//! Compositor-owned scroll gesture and offset state.
//!
//! This is the small Rechrom counterpart of cc's `InputHandler` plus the
//! mutable part of its `ScrollTree`.  It deliberately has no DOM, event
//! dispatch, Page, thread, or native-window dependency.  A committed paint
//! tree supplies immutable node geometry; wheel gestures latch their
//! main/compositor disposition here and update only compositor state.

use interaction::input_event::{WheelEvent, WheelPhase};
use layoutng_assembly::{
    fragment_tree::FragmentNode,
    internal::layout_input::{Offset, Overflow},
};
use paint::paint_engine::PaintRect;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ScrollNode {
    pub id: u64,
    pub parent_id: Option<u64>,
    pub container_rect: PaintRect,
    pub committed_y: f64,
    pub maximum_y: f64,
    pub user_scrollable_vertical: bool,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ScrollTreeSnapshot {
    pub nodes: Vec<ScrollNode>,
    root_id: Option<u64>,
}

impl ScrollTreeSnapshot {
    pub fn new(nodes: Vec<ScrollNode>) -> Self {
        // The viewport scroller is the outermost user-scrollable node.  The
        // largest-range fallback preserves older artifacts which did not keep
        // the parent link, while normal retained property trees take the
        // parentless branch.
        let root_id = nodes
            .iter()
            .filter(|node| {
                node.parent_id.is_none() && node.user_scrollable_vertical && node.maximum_y > 0.0
            })
            .max_by(|a, b| a.maximum_y.total_cmp(&b.maximum_y))
            .or_else(|| {
                nodes
                    .iter()
                    .filter(|node| node.user_scrollable_vertical && node.maximum_y > 0.0)
                    .max_by(|a, b| a.maximum_y.total_cmp(&b.maximum_y))
            })
            .map(|node| node.id);
        Self { nodes, root_id }
    }

    pub fn root(&self) -> Option<ScrollNode> {
        let id = self.root_id?;
        self.nodes.iter().find(|node| node.id == id).copied()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct WheelUpdate {
    pub blocked_on_main: bool,
    pub desired: Option<f64>,
    pub did_scroll: bool,
    pub active_changed: bool,
}

/// Mutable state belonging to the active compositor tree.
#[derive(Default)]
pub(crate) struct CompositorScrollTree {
    root: Option<ScrollNode>,
    pending_delta_y: f64,
    gesture_blocked_on_main: Option<bool>,
    active: bool,
}

impl CompositorScrollTree {
    pub fn reset_document(&mut self) {
        *self = Self::default();
    }

    pub fn root(&self) -> Option<ScrollNode> {
        self.root
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn current_position(&self) -> Option<(u64, f64)> {
        let root = self.root?;
        Some((
            root.id,
            (root.committed_y + self.pending_delta_y).clamp(0.0, root.maximum_y),
        ))
    }

    pub fn pending_delta_y(&self) -> f64 {
        self.pending_delta_y
    }

    pub fn visual_offset_for(&self, root: ScrollNode) -> f64 {
        if self.root.is_some_and(|active| active.id == root.id) {
            (root.committed_y + self.pending_delta_y).clamp(0.0, root.maximum_y)
        } else {
            root.committed_y.clamp(0.0, root.maximum_y)
        }
    }

    /// Activate the immutable tree while retaining the live compositor offset
    /// for a property node with stable identity.
    pub fn activate(&mut self, root: Option<ScrollNode>, retained: Option<(u64, f64)>) {
        self.root = root;
        self.pending_delta_y = match (root, retained) {
            (Some(root), Some((id, offset))) if id == root.id => {
                offset.clamp(0.0, root.maximum_y) - root.committed_y
            }
            _ => 0.0,
        };
        if root.is_none() {
            self.active = false;
            self.gesture_blocked_on_main = None;
        }
    }

    pub fn apply_wheel(
        &mut self,
        phase: WheelPhase,
        delta_y: f64,
        compositor_allowed: bool,
        candidate_blocks_main: bool,
    ) -> WheelUpdate {
        let blocks_main = !compositor_allowed || candidate_blocks_main;
        let blocked_on_main = match phase {
            WheelPhase::kBegan => {
                self.gesture_blocked_on_main = Some(blocks_main);
                blocks_main
            }
            WheelPhase::kChanged | WheelPhase::kEnded | WheelPhase::kCancelled => {
                self.gesture_blocked_on_main.unwrap_or(blocks_main)
            }
            WheelPhase::kNone => blocks_main,
        };
        let was_active = self.active;
        let mut desired = None;
        let mut did_scroll = false;
        if !blocked_on_main {
            if let Some(root) = self.root {
                let old = (root.committed_y + self.pending_delta_y).clamp(0.0, root.maximum_y);
                let next = (old + delta_y).clamp(0.0, root.maximum_y);
                desired = Some(next);
                if next != old {
                    self.pending_delta_y += next - old;
                    did_scroll = true;
                }
            }
        }
        match phase {
            WheelPhase::kBegan | WheelPhase::kChanged if did_scroll && !blocked_on_main => {
                self.active = true;
            }
            WheelPhase::kEnded | WheelPhase::kCancelled => self.active = false,
            _ => {}
        }
        if matches!(phase, WheelPhase::kEnded | WheelPhase::kCancelled) {
            self.gesture_blocked_on_main = None;
        }
        WheelUpdate {
            blocked_on_main,
            desired,
            did_scroll,
            active_changed: was_active != self.active,
        }
    }
}

/// Resolve the main-thread fallback against immutable committed geometry.
/// Ordinary compositor scrolling uses `CompositorScrollTree`; this function
/// keeps the same target, boundary and scroll-chain policy for gestures which
/// must return to the Page owner after cancelable event dispatch.
pub fn ComputeMainThreadScrollUpdates(
    fragments: &FragmentNode,
    wheel: &WheelEvent,
    viewport_height: f64,
    current_offset: &impl Fn(u64, Offset) -> Offset,
) -> Vec<(u64, Offset)> {
    let mut updates: Vec<(u64, Offset)> = Vec::new();
    for delta in [
        Offset {
            x: wheel.delta.x,
            y: 0.0,
        },
        Offset {
            x: 0.0,
            y: wheel.delta.y,
        },
    ] {
        let constrained = wheel.DefaultScrollDelta();
        let delta = if delta.x != 0.0 {
            Offset {
                x: constrained.x,
                y: 0.0,
            }
        } else {
            Offset {
                x: 0.0,
                y: constrained.y,
            }
        };
        if delta.x == 0.0 && delta.y == 0.0 {
            continue;
        }
        let mut remaining = delta;
        if let Some(chain) = ScrollChain(
            fragments,
            wheel.position,
            Offset::default(),
            delta,
            viewport_height,
            true,
            current_offset,
        ) {
            let mut visited = Vec::new();
            for (id, before, maximum) in chain {
                if visited.contains(&id) {
                    continue;
                }
                visited.push(id);
                let offset = Offset {
                    x: (before.x + remaining.x).clamp(0.0, maximum.x),
                    y: (before.y + remaining.y).clamp(0.0, maximum.y),
                };
                remaining.x -= offset.x - before.x;
                remaining.y -= offset.y - before.y;
                if let Some((_, merged)) = updates.iter_mut().find(|(node, _)| *node == id) {
                    if delta.x != 0.0 {
                        merged.x = offset.x;
                    }
                    if delta.y != 0.0 {
                        merged.y = offset.y;
                    }
                } else if offset != before {
                    updates.push((id, offset));
                }
                if std::env::var_os("BROWSER_APP_TRACE_INPUT").is_some() {
                    eprintln!(
                        "scroll-target node={id} before={before:?} after={offset:?} maximum={maximum:?} delta={delta:?}"
                    );
                }
                if remaining.x == 0.0 && remaining.y == 0.0 {
                    break;
                }
            }
        }
    }
    updates
}

fn ScrollChain(
    fragment: &FragmentNode,
    point: Offset,
    parent: Offset,
    delta: Offset,
    viewport_height: f64,
    root: bool,
    current_offset: &impl Fn(u64, Offset) -> Offset,
) -> Option<Vec<(u64, Offset, Offset)>> {
    if fragment.paint.hidden {
        return None;
    }
    let origin = Offset {
        x: parent.x + fragment.offset.x,
        y: parent.y + fragment.offset.y,
    };
    let inside_x = point.x >= origin.x && point.x < origin.x + fragment.size.width;
    let inside_y = point.y >= origin.y && point.y < origin.y + fragment.size.height;
    if !root
        && ((fragment.paint.overflow_x != Overflow::kVisible && !inside_x)
            || (fragment.paint.overflow_y != Overflow::kVisible && !inside_y))
    {
        return None;
    }
    let offset = current_offset(fragment.node_id, fragment.paint.scroll_offset);
    let mut child_origin = origin;
    if fragment.paint.establishes_paint_state {
        child_origin.x -= offset.x;
        child_origin.y -= offset.y;
    }
    let mut chain = None;
    for child in fragment.children.iter().rev() {
        if let Some(value) = ScrollChain(
            child,
            point,
            child_origin,
            delta,
            viewport_height,
            false,
            current_offset,
        ) {
            chain = Some(value);
            break;
        }
    }
    let user_scrollable = |overflow, horizontal| {
        fragment.paint.scroll_container.map_or_else(
            || {
                matches!(overflow, Overflow::kAuto | Overflow::kScroll)
                    || (root && overflow == Overflow::kVisible)
            },
            |container| {
                if horizontal {
                    container.user_scrollable_horizontal
                } else {
                    container.user_scrollable_vertical
                }
            },
        )
    };
    let maximum = Offset {
        x: if user_scrollable(fragment.paint.overflow_x, true) {
            (fragment.paint.scroll_size.width - fragment.size.width).max(0.0)
        } else {
            0.0
        },
        y: if user_scrollable(fragment.paint.overflow_y, false) {
            if root {
                (fragment
                    .content_size
                    .height
                    .max(fragment.paint.scroll_size.height)
                    - viewport_height)
                    .max(0.0)
            } else {
                (fragment.paint.scroll_size.height - fragment.size.height).max(0.0)
            }
        } else {
            0.0
        },
    };
    let can_move = (delta.x != 0.0 && (offset.x + delta.x).clamp(0.0, maximum.x) != offset.x)
        || (delta.y != 0.0 && (offset.y + delta.y).clamp(0.0, maximum.y) != offset.y);
    if (root || inside_x && inside_y)
        && fragment.node_id != 0
        && (root || fragment.paint.establishes_paint_state)
        && can_move
    {
        chain
            .get_or_insert_with(Vec::new)
            .push((fragment.node_id, offset, maximum));
    }
    chain
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root() -> ScrollNode {
        ScrollNode {
            id: 7,
            parent_id: None,
            container_rect: PaintRect::default(),
            committed_y: 20.0,
            maximum_y: 100.0,
            user_scrollable_vertical: true,
        }
    }

    #[test]
    fn wheel_gesture_latches_main_thread_disposition() {
        let mut tree = CompositorScrollTree::default();
        tree.activate(Some(root()), None);
        let began = tree.apply_wheel(WheelPhase::kBegan, 10.0, true, true);
        assert!(began.blocked_on_main);
        let changed = tree.apply_wheel(WheelPhase::kChanged, 10.0, true, false);
        assert!(changed.blocked_on_main);
        assert_eq!(tree.current_position(), Some((7, 20.0)));
        tree.apply_wheel(WheelPhase::kEnded, 0.0, true, false);
        let next = tree.apply_wheel(WheelPhase::kBegan, 10.0, true, false);
        assert!(next.did_scroll);
        assert_eq!(tree.current_position(), Some((7, 30.0)));
    }

    #[test]
    fn activation_retains_offset_for_the_same_scroll_node() {
        let mut tree = CompositorScrollTree::default();
        tree.activate(Some(root()), None);
        tree.apply_wheel(WheelPhase::kBegan, 35.0, true, false);
        let retained = tree.current_position();
        let mut committed = root();
        committed.committed_y = 30.0;
        tree.activate(Some(committed), retained);
        assert_eq!(tree.current_position(), Some((7, 55.0)));
    }

    #[test]
    fn snapshot_selects_the_outer_scroll_node_as_viewport_scroller() {
        let mut nested = root();
        nested.id = 8;
        nested.parent_id = Some(7);
        nested.maximum_y = 1000.0;
        let tree = ScrollTreeSnapshot::new(vec![nested, root()]);
        assert_eq!(tree.root().map(|node| node.id), Some(7));
    }
}
