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
    /// Apply cc's scroll-and-scale delta batch before the main rendering
    /// opportunity. The host verifies document identity; Page owns DOM lookup
    /// and publishes this acknowledgement only with a completed paint frame.
    pub fn ApplyCompositorScrollDelta(
        &mut self,
        sequence: u64,
        owner_client_id: u64,
        delta: Offset,
    ) -> io::Result<()> {
        if sequence <= self.processed_compositor_scroll_sequence {
            return Ok(());
        }
        if !delta.x.is_finite() || !delta.y.is_finite() {
            return Err(io::Error::other("nonfinite compositor scroll delta"));
        }
        // ScrollingCoordinator ignores callbacks for deleted scroll owners.
        // Never force measurement before a replacement document has a root.
        if !self.state.document.HasRenderableRoot() {
            self.processed_compositor_scroll_sequence = sequence;
            return Ok(());
        }
        self.state.EnsureLayoutOutput();
        let range = self
            .state
            .layout_engine
            .borrow()
            .Output()
            .map(|output| output.Root())
            .filter(|node| node.PaintClientId() == owner_client_id)
            .map(|node| node.ScrollRange());
        let Some(range) = range else {
            self.processed_compositor_scroll_sequence = sequence;
            return Ok(());
        };
        let (target_node_id, accumulated) = {
            let owner = self.state.document.borrow();
            let document = owner.GetDocument();
            let root = document
                .Node(document.Root())
                .Children()
                .iter()
                .copied()
                .find(|&index| document.Node(index).IsHTMLElement("html"))
                .ok_or_else(|| io::Error::other("compositor scroll has no document root"))?;
            let current = document.ScrollOffsetFor(root);
            (
                document.Node(root).Id(),
                Offset {
                    x: current.x + delta.x,
                    y: current.y + delta.y,
                },
            )
        };
        // Blink reflects the impl batch through its real scroll range. The
        // next delta-bearing main frame waits for this commit to activate,
        // matching the non-overlapping cc main/pending scheduler branch.
        let offset = Offset {
            x: accumulated.x.clamp(range.minimum.x, range.maximum.x),
            y: accumulated.y.clamp(range.minimum.y, range.maximum.y),
        };
        let mutation = PageMutation::ScrollMutation(page_mutation::ScrollMutation {
            target_node_id,
            offset,
        });
        if let Some(scripts) = &mut self.scripts {
            scripts.ApplyMutation(mutation)?;
        } else {
            self.state.ApplyMutation(mutation);
        }
        self.processed_compositor_scroll_sequence = sequence;
        self.RequestBeginFrameIfNeeded();
        Ok(())
    }

    pub fn ScrollWheelDefault(&mut self, wheel: &WheelEvent) -> io::Result<()> {
        self.ApplyWheelDefaultWithoutLifecycle(wheel, wheel.target_node_id)?;
        self.UpdateFrameIfNeeded()
    }
    pub(super) fn ApplyWheelDefaultWithoutLifecycle(
        &mut self,
        wheel: &WheelEvent,
        target_node_id: Option<u64>,
    ) -> io::Result<()> {
        if wheel.compositor_handled {
            // InputHandler already moved the impl tree. Main receives its
            // actual (clamped) delta at BeginMainFrame, including corrections.
            if matches!(wheel.phase, WheelPhase::kEnded | WheelPhase::kCancelled) {
                if std::mem::take(&mut self.wheel_scroll_active) {
                    self.hover_state_dirty = self.cursor_position.is_some();
                }
            } else if wheel.delta.x != 0.0 || wheel.delta.y != 0.0 {
                self.wheel_scroll_active = true;
            }
            return Ok(());
        }
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
            let mut targeted_wheel = wheel.clone();
            targeted_wheel.target_node_id = target_node_id;
            compositor::ComputeMainThreadScrollUpdates(
                &frame.fragments,
                &targeted_wheel,
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
