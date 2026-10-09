//! Page-side wheel synchronization and main-thread fallback after cancelable
//! DOM event dispatch.
//!
//! Ordinary precise-pixel viewport scrolling is owned by the compositor's
//! InputHandler/ScrollTree counterpart. Page retains the DOM scroll offset and
//! handles gestures which cannot use that fast path, including blocking
//! listeners, non-composited nested scrollers, and programmatic/test scrolls.
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
            compositor::ComputeMainThreadScrollUpdates(
                &frame.fragments,
                wheel,
                viewport_height,
                &current_offset,
            )
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
