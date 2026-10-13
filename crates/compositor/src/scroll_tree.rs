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
    pub owner_client_id: u64,
    pub parent_id: Option<u64>,
    pub container_rect: PaintRect,
    pub committed: Offset,
    pub minimum: Offset,
    pub maximum: Offset,
    pub user_scrollable_horizontal: bool,
    pub user_scrollable_vertical: bool,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ScrollTreeSnapshot {
    pub nodes: Vec<ScrollNode>,
    root_id: Option<u64>,
}

impl ScrollTreeSnapshot {
    pub fn new(nodes: Vec<ScrollNode>) -> Self {
        // The viewport scroller is the outermost user-scrollable node. The
        // shared ScrollRoot sentinel has id zero, so a real root may name it
        // as its parent. Never promote an arbitrary nested scroller: without
        // compositor hit testing and gesture latching that would scroll it for
        // wheel events delivered anywhere in the viewport.
        let root_id = nodes
            .iter()
            .filter(|node| {
                (node.parent_id.is_none() || node.parent_id == Some(0))
                    && ((node.user_scrollable_horizontal && node.maximum.x > node.minimum.x)
                        || (node.user_scrollable_vertical && node.maximum.y > node.minimum.y))
            })
            .max_by(|a, b| {
                ((a.maximum.x - a.minimum.x) + (a.maximum.y - a.minimum.y))
                    .total_cmp(&((b.maximum.x - b.minimum.x) + (b.maximum.y - b.minimum.y)))
            })
            .map(|node| node.id);
        Self { nodes, root_id }
    }

    pub fn root(&self) -> Option<ScrollNode> {
        let id = self.root_id?;
        self.nodes.iter().find(|node| node.id == id).copied()
    }

    /// This compositor currently owns one mutable scroll offset. Do not admit
    /// a root-only gesture when the immutable property tree contains another
    /// scrollable node: Page will target and chain that gesture on the main
    /// thread. Treating it as root scroll here moves both nodes when Page's
    /// committed nested offset arrives.
    pub fn supports_root_only_input(&self) -> bool {
        let Some(root) = self.root() else {
            return false;
        };
        !self.nodes.iter().any(|node| {
            node.id != root.id
                && ((node.user_scrollable_horizontal && node.maximum.x > node.minimum.x)
                    || (node.user_scrollable_vertical && node.maximum.y > node.minimum.y))
        })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct WheelUpdate {
    pub blocked_on_main: bool,
    pub desired: Option<Offset>,
    pub did_scroll: bool,
    pub active_changed: bool,
}

struct ScrollChainEntry {
    id: u64,
    before: Offset,
    minimum: Offset,
    maximum: Offset,
    allows_x_chaining: bool,
    allows_y_chaining: bool,
}

#[path = "synced_scroll_offset.rs"]
mod synced_scroll_offset;
use synced_scroll_offset::SyncedScrollOffset;

/// Mutable state belonging to the active compositor tree.
#[derive(Default)]
pub(crate) struct CompositorScrollTree {
    // Committed geometry and source-aligned synchronized impl offset.
    root: Option<ScrollNode>,
    offset: SyncedScrollOffset,
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
    pub fn current_position(&self) -> Option<(u64, Offset)> {
        self.offset.current_position()
    }
    pub fn pending_delta(&self) -> Offset {
        match (self.root, self.current_position()) {
            (Some(root), Some((_, position))) => Offset {
                x: position.x - root.committed.x,
                y: position.y - root.committed.y,
            },
            _ => Offset::default(),
        }
    }
    pub fn visual_offset_for(&self, root: ScrollNode) -> Offset {
        self.current_position()
            .filter(|(id, _)| *id == root.id)
            .map_or(root.committed, |(_, position)| position)
    }
    pub fn pull_delta_for_main(&mut self) -> Option<crate::MainThreadScrollDelta> {
        self.offset.pull_delta_for_main()
    }
    pub fn has_unacknowledged_batch(&self, acknowledged: u64) -> bool {
        self.offset.has_unacknowledged_batch(acknowledged)
    }
    pub fn activation_position(&self, root: ScrollNode, acknowledged: u64) -> Offset {
        self.offset.activation_position(root, acknowledged)
    }
    pub fn activate(&mut self, root: Option<ScrollNode>, acknowledged: u64) {
        let owner_changed =
            self.root.map(|root| root.owner_client_id) != root.map(|root| root.owner_client_id);
        self.offset.activate(root, acknowledged);
        self.root = root;
        if root.is_none() || owner_changed {
            self.active = false;
            self.gesture_blocked_on_main = None;
        }
    }

    pub fn apply_wheel(
        &mut self,
        phase: WheelPhase,
        delta: Offset,
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
                let old = self.visual_offset_for(root);
                let next = Offset {
                    x: (old.x
                        + if root.user_scrollable_horizontal {
                            delta.x
                        } else {
                            0.0
                        })
                    .clamp(root.minimum.x, root.maximum.x),
                    y: (old.y
                        + if root.user_scrollable_vertical {
                            delta.y
                        } else {
                            0.0
                        })
                    .clamp(root.minimum.y, root.maximum.y),
                };
                desired = Some(next);
                if next != old {
                    self.offset.set_current(next);
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
    let layout_zoom = fragments
        .paint
        .resources
        .as_ref()
        .map_or(1.0, |resources| resources.layout_zoom_factor)
        .max(f64::MIN_POSITIVE);
    let position = Offset {
        x: wheel.position.x * layout_zoom,
        y: wheel.position.y * layout_zoom,
    };
    let constrained = wheel.DefaultScrollDelta();
    let target = wheel
        .target_node_id
        .filter(|target| FragmentContainsNode(fragments, *target));
    for delta in [
        Offset {
            x: constrained.x * layout_zoom,
            y: 0.0,
        },
        Offset {
            x: 0.0,
            y: constrained.y * layout_zoom,
        },
    ] {
        if delta.x == 0.0 && delta.y == 0.0 {
            continue;
        }
        let mut remaining = delta;
        let mut bypass_ancestors_to_viewport = false;
        if let Some(chain) = ScrollChain(
            fragments,
            position,
            Offset::default(),
            delta,
            viewport_height * layout_zoom,
            true,
            target,
            &mut bypass_ancestors_to_viewport,
            current_offset,
        ) {
            let mut visited = Vec::new();
            for entry in chain {
                if visited.contains(&entry.id) {
                    continue;
                }
                visited.push(entry.id);
                let offset = Offset {
                    x: (entry.before.x + remaining.x).clamp(entry.minimum.x, entry.maximum.x),
                    y: (entry.before.y + remaining.y).clamp(entry.minimum.y, entry.maximum.y),
                };
                remaining.x -= offset.x - entry.before.x;
                remaining.y -= offset.y - entry.before.y;
                if let Some((_, merged)) = updates.iter_mut().find(|(node, _)| *node == entry.id) {
                    if delta.x != 0.0 {
                        merged.x = offset.x;
                    }
                    if delta.y != 0.0 {
                        merged.y = offset.y;
                    }
                } else if offset != entry.before {
                    updates.push((entry.id, offset));
                }
                if std::env::var_os("BROWSER_APP_TRACE_INPUT").is_some() {
                    eprintln!(
                        "scroll-target node={} before={:?} after={offset:?} minimum={:?} maximum={:?} delta={delta:?}",
                        entry.id, entry.before, entry.minimum, entry.maximum
                    );
                }
                let chaining_blocked = (delta.x != 0.0 && !entry.allows_x_chaining)
                    || (delta.y != 0.0 && !entry.allows_y_chaining);
                if chaining_blocked || (remaining.x == 0.0 && remaining.y == 0.0) {
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
    target: Option<u64>,
    bypass_ancestors_to_viewport: &mut bool,
    current_offset: &impl Fn(u64, Offset) -> Offset,
) -> Option<Vec<ScrollChainEntry>> {
    if fragment.paint.hidden {
        return None;
    }
    let origin = Offset {
        x: parent.x + fragment.offset.x,
        y: parent.y + fragment.offset.y,
    };
    let inside_x = point.x >= origin.x && point.x < origin.x + fragment.size.width;
    let inside_y = point.y >= origin.y && point.y < origin.y + fragment.size.height;
    if target.is_none()
        && !root
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
    let target_here = target == Some(fragment.node_id);
    for child in fragment.children.iter().rev() {
        let child_is_target_branch =
            target.is_some() && !target_here && FragmentContainsNode(child, target.unwrap());
        if target.is_some() && !child_is_target_branch {
            continue;
        }
        let child_chain = ScrollChain(
            child,
            point,
            child_origin,
            delta,
            viewport_height,
            false,
            target,
            bypass_ancestors_to_viewport,
            current_offset,
        );
        // A fixed-to-view subtree is parented above document scroll in the
        // paint property tree. Its own nested scrollers still consume wheel
        // input, then chaining resumes at the viewport rather than at DOM/
        // fragment ancestors which do not geometrically contain it.
        if child_is_target_branch && child.paint.fixed_to_view {
            *bypass_ancestors_to_viewport = true;
        }
        if let Some(value) = child_chain {
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
    let (minimum, maximum) = fragment.paint.scroll_container.map_or_else(
        || {
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
            (Offset::default(), maximum)
        },
        |container| {
            let mut minimum = container.minimum_scroll_offset;
            let mut maximum = container.maximum_scroll_offset;
            if !user_scrollable(fragment.paint.overflow_x, true) {
                minimum.x = 0.0;
                maximum.x = 0.0;
            }
            if !user_scrollable(fragment.paint.overflow_y, false) {
                minimum.y = 0.0;
                maximum.y = 0.0;
            }
            // The host viewport can be smaller than the LayoutView's exported
            // clip while browser controls are present. Preserve its native
            // minimum and extend only the positive block-end range.
            if root {
                maximum.y = maximum.y.max(
                    minimum.y
                        + (fragment
                            .content_size
                            .height
                            .max(fragment.paint.scroll_size.height)
                            - viewport_height)
                            .max(0.0),
                );
            }
            (minimum, maximum)
        },
    );
    let can_move = (delta.x != 0.0 && (offset.x + delta.x).clamp(minimum.x, maximum.x) != offset.x)
        || (delta.y != 0.0 && (offset.y + delta.y).clamp(minimum.y, maximum.y) != offset.y);
    let allows_x_chaining = fragment.paint.overscroll_behavior_x.AllowsScrollChaining();
    let allows_y_chaining = fragment.paint.overscroll_behavior_y.AllowsScrollChaining();
    let blocks_chain =
        (delta.x != 0.0 && user_scrollable(fragment.paint.overflow_x, true) && !allows_x_chaining)
            || (delta.y != 0.0
                && user_scrollable(fragment.paint.overflow_y, false)
                && !allows_y_chaining);
    // Once EventHandler has resolved a DOM target, that target's ancestor
    // branch is authoritative. Re-testing an untransformed fragment rect here
    // drops nested scrollers beneath transforms and fixed/sticky positioning.
    if (root || !*bypass_ancestors_to_viewport)
        && (root || target.is_some() || inside_x && inside_y)
        && fragment.node_id != 0
        && (root || fragment.paint.establishes_paint_state)
        && (can_move || blocks_chain)
    {
        chain.get_or_insert_with(Vec::new).push(ScrollChainEntry {
            id: fragment.node_id,
            before: offset,
            minimum,
            maximum,
            allows_x_chaining,
            allows_y_chaining,
        });
    }
    chain
}

fn FragmentContainsNode(fragment: &FragmentNode, node_id: u64) -> bool {
    fragment.node_id == node_id
        || fragment
            .children
            .iter()
            .any(|child| FragmentContainsNode(child, node_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn root() -> ScrollNode {
        ScrollNode {
            id: 7,
            owner_client_id: 70,
            parent_id: None,
            container_rect: PaintRect::default(),
            committed: Offset { x: 0., y: 20. },
            minimum: Offset::default(),
            maximum: Offset { x: 0., y: 100. },
            user_scrollable_horizontal: false,
            user_scrollable_vertical: true,
        }
    }
    #[test]
    fn wheel_gesture_latches_main_thread_disposition() {
        let mut tree = CompositorScrollTree::default();
        tree.activate(Some(root()), 0);
        assert!(
            tree.apply_wheel(WheelPhase::kBegan, Offset { x: 0., y: 10. }, true, true)
                .blocked_on_main
        );
        assert!(
            tree.apply_wheel(WheelPhase::kChanged, Offset { x: 0., y: 10. }, true, false)
                .blocked_on_main
        );
        assert_eq!(tree.current_position().unwrap().1.y, 20.);
        tree.apply_wheel(WheelPhase::kEnded, Offset::default(), true, false);
        assert!(
            tree.apply_wheel(WheelPhase::kBegan, Offset { x: 0., y: 10. }, true, false)
                .did_scroll
        );
        assert_eq!(tree.current_position().unwrap().1.y, 30.);
        assert_eq!(tree.pull_delta_for_main().unwrap().delta.y, 10.);
    }
    #[test]
    fn negative_range_and_document_identity_do_not_reuse_old_delta() {
        let mut tree = CompositorScrollTree::default();
        let node = ScrollNode {
            committed: Offset { x: 0., y: -20. },
            minimum: Offset { x: 0., y: -100. },
            maximum: Offset::default(),
            ..root()
        };
        tree.activate(Some(node), 0);
        tree.apply_wheel(
            WheelPhase::kChanged,
            Offset { x: 0., y: -200. },
            true,
            false,
        );
        assert_eq!(tree.current_position().unwrap().1.y, -100.);
        assert_eq!(tree.pull_delta_for_main().unwrap().delta.y, -80.);
        tree.reset_document();
        tree.activate(Some(node), 0);
        assert_eq!(tree.current_position().unwrap().1.y, -20.);
        assert!(tree.pull_delta_for_main().is_none());
    }
    #[test]
    fn snapshot_selects_outer_scroll_owner() {
        let nested = ScrollNode {
            id: 8,
            parent_id: Some(7),
            maximum: Offset { x: 0., y: 1000. },
            ..root()
        };
        assert_eq!(
            ScrollTreeSnapshot::new(vec![nested, root()])
                .root()
                .unwrap()
                .id,
            7
        );
    }
}
