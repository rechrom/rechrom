//! Default wheel handling belongs to Page, after cancelable event dispatch.
//! Route axes independently through real scroll boxes, consuming their available
//! range before handing the remaining displacement to the ancestor.
use super::*;
use interaction::input_event::{WheelEvent, WheelPhase};

impl Page {
    pub fn ScrollWheelDefault(&mut self, wheel: &WheelEvent) -> io::Result<()> {
        self.ApplyWheelDefaultWithoutLifecycle(wheel)?;
        self.UpdateFrameIfNeeded()
    }
    pub(super) fn ApplyWheelDefaultWithoutLifecycle(
        &mut self,
        wheel: &WheelEvent,
    ) -> io::Result<()> {
        if !wheel.delta.x.is_finite() || !wheel.delta.y.is_finite() {
            return Ok(());
        }
        let updates = {
            let Some(frame) = self.frame.as_ref() else {
                return Ok(());
            };
            let owner = self.state.document.borrow();
            let document = owner.GetDocument();
            let current_offset = |id, fallback| {
                document
                    .FindNodeById(id)
                    .map_or(fallback, |index| document.ScrollOffsetFor(index))
            };
            let viewport_height = self.state.constraints.borrow().available_size.height;
            scroll_updates(&frame.fragments, wheel, viewport_height, &current_offset)
        };
        let scrolled = !updates.is_empty();
        for (target_node_id, offset) in updates {
            let mutation = PageMutation::ScrollMutation(page_mutation::ScrollMutation {
                target_node_id,
                offset,
            });
            if let Some(scripts) = &mut self.scripts {
                scripts.ApplyMutation(mutation)?;
            } else {
                self.state.ApplyMutation(mutation);
            }
        }
        if scrolled {
            self.wheel_scroll_active = true;
        }
        if matches!(wheel.phase, WheelPhase::kEnded | WheelPhase::kCancelled)
            && std::mem::take(&mut self.wheel_scroll_active)
        {
            // ScrollableArea::OnScrollFinished(false) does this in Blink.
            // Resolve it at the next BeginFrame, after the final scroll
            // geometry has committed, rather than during wheel dispatch.
            self.hover_state_dirty = self.cursor_position.is_some();
        }
        Ok(())
    }
}

fn scroll_chain(
    fragment: &FragmentNode,
    point: Offset,
    parent: Offset,
    delta: Offset,
    viewport_height: f64,
    root: bool,
    current_offset: &impl Fn(u64, Offset) -> Offset,
) -> Option<Vec<(u64, Offset, Offset)>> {
    use layoutng_assembly::internal::layout_input::Overflow;
    if fragment.paint.hidden {
        return None;
    }
    let origin = Offset {
        x: parent.x + fragment.offset.x,
        y: parent.y + fragment.offset.y,
    };
    let inside_x = point.x >= origin.x && point.x < origin.x + fragment.size.width;
    let inside_y = point.y >= origin.y && point.y < origin.y + fragment.size.height;
    // A clipped-away descendant must not capture wheel input. Visible
    // overflow can still contain a hit descendant outside the parent's box.
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
        if let Some(value) = scroll_chain(
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
    // The LayoutView owns viewport overflow after the HTML/body propagation
    // rules have run. Prefer its exported scroll-container policy over the
    // root fragment's raw CSS overflow.
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
    // Only real scroll boxes accept wheel input; at a directional boundary
    // keep walking out to an ancestor. In particular, horizontal overflow
    // cannot swallow a vertical gesture (nor can overflow:hidden/clip).
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

fn scroll_updates(
    fragments: &FragmentNode,
    wheel: &WheelEvent,
    viewport_height: f64,
    current_offset: &impl Fn(u64, Offset) -> Offset,
) -> Vec<(u64, Offset)> {
    let mut updates: Vec<(u64, Offset)> = Vec::new();
    // Route axes independently and pass unconsumed displacement out through
    // the same ancestor chain. A coalesced gesture must not lose its remainder
    // when an inner scroller reaches its limit partway through the delta.
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
        // A stationary or railed-away axis cannot select a scroll target.
        // Avoid the second fragment-tree walk for ordinary vertical gestures.
        if delta.x == 0.0 && delta.y == 0.0 {
            continue;
        }
        let mut remaining = delta;
        if let Some(chain) = scroll_chain(
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
                    eprintln!("scroll-target node={id} before={before:?} after={offset:?} maximum={maximum:?} delta={delta:?}");
                }
                if remaining.x == 0.0 && remaining.y == 0.0 {
                    break;
                }
            }
        }
    }
    updates
}
