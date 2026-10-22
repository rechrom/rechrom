//! Main-thread wheel default action against immutable committed geometry.
//!
//! Compositor scrolling stays in cc's InputHandler. This owner handles the
//! route selected for blocking listeners and main-thread scrolling reasons,
//! and returns typed DOM scroll-offset updates without mutating Page, Layout,
//! Paint or Compositor.

use crate::input_event::WheelEvent;
use layoutng_assembly::{
    fragment_tree::FragmentNode,
    internal::layout_input::{Offset, Overflow},
};

struct ScrollChainEntry {
    id: u64,
    before: Offset,
    minimum: Offset,
    maximum: Offset,
    allows_x_chaining: bool,
    allows_y_chaining: bool,
}

/// Resolve a blocked/cancelable wheel's default action against the current
/// immutable fragment snapshot. The caller supplies live DOM offsets so a
/// retained fragment does not become a second mutable scroll-state owner.
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
            // LayoutNG's scroll container is the canonical, pixel-snapped
            // range used by Blink and cc. Re-deriving a larger root maximum
            // from unsnapped fragment geometry makes the main-thread route
            // disagree with the compositor at the bottom edge.
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
