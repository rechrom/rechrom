//! Page-side synchronization for the mutually exclusive compositor and
//! main-thread wheel routes.
//!
//! Ordinary precise-pixel viewport scrolling is owned by the compositor's
//! InputHandler/ScrollTree counterpart. Page retains the DOM scroll offset and
//! handles gestures assigned to the main thread, including blocking
//! listeners, non-composited nested scrollers, and programmatic/test scrolls.
use super::*;
use interaction::input_event::{WheelEvent, WheelPhase};

impl Page {
    fn ReflectCompositorScrollOffset(&mut self, target_node_id: u64, offset: Offset) {
        let Some(effect) = self
            .state
            .document
            .ApplyScrollOffset(target_node_id, offset)
        else {
            return;
        };
        browser_tracing::instant(
            "input",
            "ScrollApplied",
            &[
                ("scroll_node", effect.target_node_id as f64),
                ("before_x", effect.before.x),
                ("before_y", effect.before.y),
                ("offset_x", effect.offset.x),
                ("offset_y", effect.offset.y),
                ("delta_x", effect.offset.x - effect.before.x),
                ("delta_y", effect.offset.y - effect.before.y),
            ],
        );
        if !self
            .compositor_scroll_targets
            .contains(&effect.target_node_id)
        {
            self.compositor_scroll_targets.push(effect.target_node_id);
        }
        let mut targets = self.state.scroll_event_targets.borrow_mut();
        if !targets.contains(&effect.event_target_node_id) {
            targets.push(effect.event_target_node_id);
        }
    }

    fn CommitCompositorScrollOffsets(&mut self) {
        if self.compositor_scroll_targets.is_empty() {
            return;
        }
        let targets = std::mem::take(&mut self.compositor_scroll_targets);
        let offsets = {
            let owner = self.state.document.borrow();
            let document = owner.GetDocument();
            targets
                .into_iter()
                .filter_map(|id| {
                    document
                        .FindNodeById(id)
                        .map(|node| (id, document.ScrollOffsetFor(node)))
                })
                .collect::<Vec<_>>()
        };
        if offsets.is_empty() {
            return;
        }
        let mut pending = self.state.scroll_updates.borrow_mut();
        for (id, offset) in offsets {
            if let Some((_, current)) = pending.iter_mut().find(|(target, _)| *target == id) {
                *current = offset;
            } else {
                pending.push((id, offset));
            }
        }
        drop(pending);
        self.state.InvalidateLayoutOutput();
        if !self.state.dirty.get() {
            self.state.scroll_only.set(true);
        }
        self.state.dirty.set(true);
        self.RequestBeginFrameIfNeeded();
    }

    #[cfg(feature = "production_raster")]
    fn CompositorScrollOwner(
        &self,
        element: compositor::adapter::input_types::ElementId,
    ) -> Option<(u64, u64)> {
        let frame = self.frame.as_ref()?;
        let viewport = frame.display_items.compositor_viewport.as_ref()?;
        let element = element.GetInternalValue();
        for translation in &viewport.scroll_translation_nodes {
            let mut scroll = translation.scroll.as_deref();
            while let Some(node) = scroll {
                if node.compositor.compositor_element_id.GetInternalValue() == element {
                    let owner = node.owner_client_id;
                    let dom_node = frame
                        .display_items
                        .client_owner_node_ids
                        .get(&(owner as usize))
                        .copied()
                        .filter(|id| *id > 0)? as u64;
                    return Some((owner, dom_node));
                }
                scroll = node.parent.as_deref();
            }
        }
        None
    }

    #[cfg(feature = "production_raster")]
    fn ApplyCompositorScrollOffset(
        &mut self,
        sequence: u64,
        owner_client_id: u64,
        target_node_id: u64,
        offset: Offset,
    ) -> io::Result<()> {
        if sequence <= self.processed_compositor_scroll_sequence {
            return Ok(());
        }
        if !offset.x.is_finite() || !offset.y.is_finite() {
            return Err(io::Error::other("nonfinite compositor scroll offset"));
        }
        if !self.state.document.HasRenderableRoot() {
            self.processed_compositor_scroll_sequence = sequence;
            return Ok(());
        }
        // ScrollingCoordinator reflects an impl-thread offset against the
        // already committed layout state.  It must not enter the document
        // lifecycle from input dispatch: a dirty style tree is serviced by
        // the next normal lifecycle update.
        let range = self
            .state
            .layout_engine
            .borrow()
            .Output()
            .and_then(|output| output.Node(target_node_id))
            .filter(|node| node.PaintClientId() == owner_client_id)
            .map(|node| node.ScrollRange());
        let Some(range) = range else {
            self.processed_compositor_scroll_sequence = sequence;
            return Ok(());
        };
        let offset = Offset {
            x: offset.x.clamp(range.minimum.x, range.maximum.x),
            y: offset.y.clamp(range.minimum.y, range.maximum.y),
        };
        // ScrollingCoordinator reflects compositor deltas into Blink-visible
        // state without repainting the already moving active tree. A later
        // main commit acknowledges the final offset once the gesture ends.
        self.ReflectCompositorScrollOffset(target_node_id, offset);
        self.processed_compositor_scroll_sequence = sequence;
        Ok(())
    }

    #[cfg(feature = "production_raster")]
    pub fn ApplyCompositorWheelEffects(
        &mut self,
        effects: Vec<CompositorMainThreadEffect>,
    ) -> io::Result<()> {
        let mut requested_main_frame = false;
        for effect in effects {
            match effect {
                CompositorMainThreadEffect::RequestMainFrame => requested_main_frame = true,
                CompositorMainThreadEffect::ScrollOffsetChanged {
                    sequence,
                    element,
                    offset,
                    ..
                } => {
                    let Some((owner, target)) = self.CompositorScrollOwner(element) else {
                        continue;
                    };
                    self.ApplyCompositorScrollOffset(
                        sequence,
                        owner,
                        target,
                        Offset {
                            x: f64::from(offset.x()),
                            y: f64::from(offset.y()),
                        },
                    )?;
                }
                _ => {}
            }
        }
        // Source: ProxyMain::BeginMainFrame. The applied compositor delta and
        // the property-tree commit belong to one main-frame transaction.
        if requested_main_frame {
            self.CommitCompositorScrollOffsets();
        }
        Ok(())
    }

    #[cfg(feature = "production_raster")]
    pub(super) fn PresentCompositorOnlyFrame(&mut self) {
        let Some(frame) = self.frame.as_mut() else {
            return;
        };
        frame.sequence = frame.sequence.wrapping_add(1).max(1);
        frame.compositor_scroll_sequence = self.processed_compositor_scroll_sequence;
        self.state.client.borrow_mut().DidPresentFrame(frame);
    }

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
        // Consume the resident committed output.  Pulling layout forward here
        // would put style recalculation and layout on the wheel dispatch path.
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
        if wheel.compositor_owned {
            // InputHandler already moved the impl tree. Main receives its
            // actual (clamped) delta at BeginMainFrame, including corrections.
            if matches!(wheel.phase, WheelPhase::kEnded | WheelPhase::kCancelled) {
                self.CommitCompositorScrollOffsets();
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
            interaction::ComputeMainThreadScrollUpdates(
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
            self.CommitCompositorScrollOffsets();
            // ScrollableArea::OnScrollFinished(false) does this in Blink.
            // Resolve it at the next BeginFrame, after the final scroll
            // geometry has committed, rather than during wheel dispatch.
            self.hover_state_dirty = self.cursor_position.is_some();
        }
        Ok(())
    }
}
