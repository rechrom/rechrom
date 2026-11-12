//! Live native area geometry for ScrollableArea/UserScroll. No animator or
//! Interaction object crosses Layout's boundary and no scroll tree is copied.
use crate::{
    internal::{
        layout_box::LayoutBox, layout_object::LayoutObject, layout_pass_scope::LayoutPassScope,
        scroll_types::IncludeScrollbarsInRect, scrollbar_orientation::ScrollbarOrientation,
    },
    layout_engine::LayoutEngine,
    native_autoscroll_source::NativeLogicalScrollArea,
};
use foundation::{gfx, DynamicTo, PhysicalRect};
use layoutng_geometry::geometry::axis::{kPhysicalAxesHorizontal, kPhysicalAxesVertical};
#[derive(Clone, Copy)]
pub struct NativeUserScrollAreaFacts {
    pub node_id: Option<u64>,
    pub scroll_element_id: foundation::CompositorElementId,
    pub user_input_scrollable_horizontal: bool,
    pub user_input_scrollable_vertical: bool,
    pub scrollable_horizontal: bool,
    pub scrollable_vertical: bool,
    pub scrolls_overflow: bool,
    /// This extracted owner is PaintLayerScrollableArea. RootFrameViewport is
    /// a distinct source owner and must not be inferred from LayoutView.
    pub is_root_frame_viewport: bool,
    /// Relevant only for RootFrameViewport completion admission.
    pub scroll_affects_layout_viewport: bool,
}

// Source: ui/events/types/scroll_types.h:12-20.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeUserScrollGranularity {
    PrecisePixel,
    Pixel,
    Line,
    Page,
    Document,
}

// Source: scrollable_area.h:97-104.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeScrollCompletionMode {
    Finished,
    ZeroDelta,
    InterruptedByScroll,
}

// Source: scroll_types.h:169-198.
#[derive(Clone, Copy, Debug, Default)]
pub struct NativeScrollConsumption {
    pub did_scroll_x: bool,
    pub did_scroll_y: bool,
    pub unused_scroll_delta_x: f32,
    pub unused_scroll_delta_y: f32,
}

/// The existing ScrollAnimator owner supplies only its current admission
/// state. Layout retains and clamps the native ScrollableArea geometry.
// Source: scroll_animator.cc:68-81,112-160.
#[derive(Clone, Copy, Debug)]
pub struct NativeUserScrollAnimationState {
    pub enabled: bool,
    pub has_running_animation: bool,
    /// ScrollAnimator::DesiredTargetOffset. None means CurrentOffset().
    pub desired_target_offset: Option<gfx::Vector2dF>,
}

impl Default for NativeUserScrollAnimationState {
    fn default() -> Self {
        Self {
            enabled: false,
            has_running_animation: false,
            desired_target_offset: None,
        }
    }
}

/// The animation branch stays a typed request because ScrollAnimator and its
/// compositor registration are owned outside LayoutNG. Accepted/rejected
/// consumption preserves ScrollAnimator::UserScroll's synchronous contract.
#[derive(Clone, Copy, Debug)]
pub struct NativeUserScrollAnimationRequest {
    pub node_id: Option<u64>,
    pub scroll_element_id: foundation::CompositorElementId,
    pub source_type: scroll_snap::scroll_source_type::ScrollSourceType,
    pub granularity: NativeUserScrollGranularity,
    pub target_offset: gfx::Vector2dF,
    pub interrupt_previous_completion: bool,
    pub accepted_consumption: NativeScrollConsumption,
    pub rejected_consumption: NativeScrollConsumption,
    pub rejected_completion: NativeScrollCompletionMode,
}

#[derive(Clone, Copy, Debug)]
pub enum NativeUserScrollPlan {
    Completed {
        consumption: NativeScrollConsumption,
        completion: NativeScrollCompletionMode,
        /// Present only when the original native area committed immediately.
        committed_offset: Option<gfx::Vector2dF>,
        cancel_programmatic_animation: bool,
        finish_previous_animation: bool,
    },
    Animate(NativeUserScrollAnimationRequest),
}

/// DOM/event effects selected by ScrollableArea::OnScrollFinished. The native
/// snap target store is updated before this value crosses the boundary.
#[derive(Clone, Copy, Debug)]
pub struct NativeScrollFinishedPlan {
    pub node_id: Option<u64>,
    pub scroll_element_id: foundation::CompositorElementId,
    pub reset_active_smooth_scroll_type: bool,
    pub snap_target_notification:
        Option<crate::internal::layout_scrollable_area::NativeSnapTargetNotification>,
    pub notify_anchor_viewport_tracker: bool,
    pub start_marker_animation: bool,
    pub enqueue_scrollend: bool,
    pub mark_hover_state_dirty: bool,
}
pub struct NativeUserScrollSource<'a> {
    _scope: LayoutPassScope,
    _engine: &'a mut LayoutEngine,
    native: &'a NativeLogicalScrollArea,
}
fn Inclusion(include: bool) -> IncludeScrollbarsInRect {
    if include {
        IncludeScrollbarsInRect::kIncludeScrollbars
    } else {
        IncludeScrollbarsInRect::kExcludeScrollbars
    }
}
fn Orientation(horizontal: bool) -> ScrollbarOrientation {
    if horizontal {
        ScrollbarOrientation::kHorizontalScrollbar
    } else {
        ScrollbarOrientation::kVerticalScrollbar
    }
}
#[derive(Clone, Copy, Debug)]
pub struct NativeUserScrollAreaDisposal {
    pub native_area_identity: usize,
    pub scroll_element_id: foundation::CompositorElementId,
}
impl LayoutEngine {
    pub fn TakeDisposedUserScrollAreas(&mut self) -> Vec<NativeUserScrollAreaDisposal> {
        let mut disposed = Vec::new();
        self.user_scroll_disposal_queues_.retain(|_, queue| {
            disposed.extend(std::mem::take(&mut *queue.borrow_mut()).into_iter().map(
                |(native_area_identity, scroll_element_id)| NativeUserScrollAreaDisposal {
                    native_area_identity,
                    scroll_element_id,
                },
            ));
            // The native FrameView/LayoutView releases its queue on disposal;
            // retain the exact outbox through root replacement until drained.
            std::rc::Rc::strong_count(queue) > 1
        });
        disposed
    }
    pub fn NativeUserScrollSource<'a>(
        &'a mut self,
        native: &'a NativeLogicalScrollArea,
    ) -> Option<NativeUserScrollSource<'a>> {
        if native.Area().HasBeenDisposed() {
            return None;
        }
        if let Some(queue) = native.Area().UserScrollDisposalQueue() {
            self.user_scroll_disposal_queues_
                .entry(std::rc::Rc::as_ptr(&queue) as usize)
                .or_insert(queue);
        }
        let scope = self.NativeScrollPassScope();
        Some(NativeUserScrollSource {
            _scope: scope,
            _engine: self,
            native,
        })
    }
    pub fn LayoutViewportLogicalScrollArea(&self) -> Option<NativeLogicalScrollArea> {
        let root = self.GetLayoutTree()?.Root() as *const LayoutObject as *mut LayoutObject;
        let box_ = unsafe { DynamicTo::<LayoutBox>(root).as_ref()? };
        NativeLogicalScrollArea::FromPointer(box_.GetScrollableArea())
    }
}
impl LayoutEngine {
    /// The FrameView's viewport input in the native layout coordinate space,
    /// identical to the source descriptor consumed by the viewport painter.
    pub fn VisualViewportFrameSize(&self) -> Option<gfx::Size> {
        let input = self.GetConstraints();
        let viewport = input.viewport.as_ref()?;
        Some(gfx::ToRoundedSize(gfx::SizeF::new(
            (viewport.size.width as f64 * input.layout_zoom_factor) as f32,
            (viewport.size.height as f64 * input.layout_zoom_factor) as f32,
        )))
    }
    pub fn LogicalScrollAreaForScrollbar(
        &self,
        scrollbar: &crate::internal::scrollbar::Scrollbar,
    ) -> Option<NativeLogicalScrollArea> {
        NativeLogicalScrollArea::FromPointer(scrollbar.ScrollableArea())
    }
}
impl NativeUserScrollSource<'_> {
    pub fn Facts(&self) -> NativeUserScrollAreaFacts {
        let area = self.native.Area();
        let axes = area.ScrollableAxes();
        let node = unsafe { area.GetLayoutBox().as_ref() }
            .and_then(|box_| unsafe { box_.GetNode().as_ref() });
        NativeUserScrollAreaFacts {
            node_id: node.map(|node| node.InputId()),
            scroll_element_id: area.GetScrollElementId(),
            user_input_scrollable_horizontal: area.UserInputScrollable(Orientation(true)),
            user_input_scrollable_vertical: area.UserInputScrollable(Orientation(false)),
            scrollable_horizontal: (axes & kPhysicalAxesHorizontal).is_nonzero(),
            scrollable_vertical: (axes & kPhysicalAxesVertical).is_nonzero(),
            scrolls_overflow: area.ScrollsOverflow(),
            is_root_frame_viewport: false,
            scroll_affects_layout_viewport: true,
        }
    }
    pub fn GetScrollOffset(&self) -> gfx::Vector2dF {
        self.native.Area().GetScrollOffset()
    }
    pub fn MinimumScrollOffsetInt(&self) -> gfx::Vector2d {
        self.native.Area().MinimumScrollOffsetInt()
    }
    pub fn MaximumScrollOffsetInt(&self) -> gfx::Vector2d {
        self.native.Area().MaximumScrollOffsetInt()
    }
    pub fn VisibleContentRect(&self, include_scrollbars: bool) -> gfx::Rect {
        self.native
            .Area()
            .VisibleContentRect(Inclusion(include_scrollbars))
    }
    pub fn VisibleScrollSnapportRect(&self, include_scrollbars: bool) -> PhysicalRect {
        self.native
            .Area()
            .VisibleScrollSnapportRect(Inclusion(include_scrollbars))
    }
    pub fn ScrollSize(&self, horizontal: bool) -> i32 {
        self.native.Area().ScrollSize(Orientation(horizontal))
    }
    pub fn ClampScrollOffset(&self, offset: gfx::Vector2dF) -> gfx::Vector2dF {
        self.native.Area().ClampScrollOffset(&offset)
    }
    pub fn ScrollOffsetToPosition(&self, offset: gfx::Vector2dF) -> gfx::PointF {
        self.native.Area().ScrollOffsetToPosition(&offset)
    }
    pub fn ScrollPositionToOffset(&self, position: gfx::PointF) -> gfx::Vector2dF {
        self.native.Area().ScrollPositionToOffset(&position)
    }
}

impl NativeUserScrollSource<'_> {
    fn UserScrollStep(&self, granularity: NativeUserScrollGranularity, horizontal: bool) -> f32 {
        use crate::internal::scroll_types::IncludeScrollbarsInRect;
        use crate::internal::scrollbar_orientation::ScrollbarOrientation;
        let area = self.native.Area();
        let orientation = if horizontal {
            ScrollbarOrientation::kHorizontalScrollbar
        } else {
            ScrollbarOrientation::kVerticalScrollbar
        };
        match granularity {
            NativeUserScrollGranularity::PrecisePixel | NativeUserScrollGranularity::Pixel => 1.0,
            NativeUserScrollGranularity::Line => {
                (f64::from(scroll_snap::scroll_utils::kPixelsPerLineStep) * area.ScaleFromDIP())
                    as f32
            }
            NativeUserScrollGranularity::Page => {
                let size = area
                    .VisibleScrollSnapportRect(IncludeScrollbarsInRect::kExcludeScrollbars)
                    .PixelSnappedSize();
                scroll_snap::scroll_utils::ScrollUtils::CalculatePageStep(if horizontal {
                    size.width()
                } else {
                    size.height()
                }) as f32
            }
            NativeUserScrollGranularity::Document => area.ScrollSize(orientation) as f32,
        }
    }

    fn Consumption(consumed: gfx::Vector2dF, unused: gfx::Vector2dF) -> NativeScrollConsumption {
        NativeScrollConsumption {
            did_scroll_x: consumed.x() != 0.0,
            did_scroll_y: consumed.y() != 0.0,
            unused_scroll_delta_x: unused.x(),
            unused_scroll_delta_y: unused.y(),
        }
    }

    /// ScrollAnimator::ComputeDeltaToConsume admission used by
    /// RootFrameViewport before it delegates the completion owner.
    // Source: scroll_animator.cc:68-81; root_frame_viewport.cc:561-583.
    pub fn WillConsumeUserScroll(
        &self,
        granularity: NativeUserScrollGranularity,
        delta: gfx::Vector2dF,
        animation: NativeUserScrollAnimationState,
    ) -> bool {
        use crate::internal::scrollbar_orientation::ScrollbarOrientation;
        let area = self.native.Area();
        let pixel_delta = gfx::Vector2dF::new(
            delta.x() * self.UserScrollStep(granularity, true),
            delta.y() * self.UserScrollStep(granularity, false),
        );
        let scrollable_axis_delta = gfx::Vector2dF::new(
            if area.UserInputScrollable(ScrollbarOrientation::kHorizontalScrollbar) {
                pixel_delta.x()
            } else {
                0.0
            },
            if area.UserInputScrollable(ScrollbarOrientation::kVerticalScrollbar) {
                pixel_delta.y()
            } else {
                0.0
            },
        );
        let desired = animation
            .desired_target_offset
            .unwrap_or_else(|| area.GetScrollOffset());
        let target = area.ClampScrollOffset(&(desired + scrollable_axis_delta));
        target != desired
    }

    /// ScrollableArea::UserScroll with the source native area retained here.
    /// The no-animation branch commits the original PaintLayerScrollableArea;
    /// the animation branch returns a request for the existing animator owner.
    // Source: scrollable_area.cc:226-274; scroll_animator_base.cc:44-78;
    // scroll_animator.cc:68-81,93-160.
    pub fn UserScroll(
        &mut self,
        granularity: NativeUserScrollGranularity,
        delta: gfx::Vector2dF,
        source_type: scroll_snap::scroll_source_type::ScrollSourceType,
        animation: NativeUserScrollAnimationState,
    ) -> NativeUserScrollPlan {
        use crate::internal::scroll_types::ScrollOffset;
        use crate::internal::scrollbar_orientation::ScrollbarOrientation;
        let area = self.native.Area();
        let pixel_delta = gfx::Vector2dF::new(
            delta.x() * self.UserScrollStep(granularity, true),
            delta.y() * self.UserScrollStep(granularity, false),
        );
        let scrollable_axis_delta = gfx::Vector2dF::new(
            if area.UserInputScrollable(ScrollbarOrientation::kHorizontalScrollbar) {
                pixel_delta.x()
            } else {
                0.0
            },
            if area.UserInputScrollable(ScrollbarOrientation::kVerticalScrollbar) {
                pixel_delta.y()
            } else {
                0.0
            },
        );
        let unscrollable_axis_delta = pixel_delta - scrollable_axis_delta;
        let current = area.GetScrollOffset();
        let desired = animation.desired_target_offset.unwrap_or(current);
        let desired_target = area.ClampScrollOffset(&(desired + scrollable_axis_delta));
        let delta_to_consume = desired_target - desired;
        if delta_to_consume.x() == 0.0 && delta_to_consume.y() == 0.0 {
            return NativeUserScrollPlan::Completed {
                consumption: Self::Consumption(ScrollOffset::default(), pixel_delta),
                completion: NativeScrollCompletionMode::ZeroDelta,
                committed_offset: None,
                cancel_programmatic_animation: false,
                finish_previous_animation: false,
            };
        }

        if !animation.enabled || granularity == NativeUserScrollGranularity::PrecisePixel {
            let target = area.ClampScrollOffset(&(current + scrollable_axis_delta));
            let consumed = target - current;
            let unused = (scrollable_axis_delta - consumed) + unscrollable_axis_delta;
            unsafe { &mut *self.native.AreaPointer() }.UpdateScrollOffset(
                target,
                layoutng_style::style::scroll_enums::mojom::blink::ScrollType::kUser,
            );
            return NativeUserScrollPlan::Completed {
                consumption: Self::Consumption(consumed, unused),
                completion: if consumed.x() == 0.0 && consumed.y() == 0.0 {
                    NativeScrollCompletionMode::ZeroDelta
                } else {
                    NativeScrollCompletionMode::Finished
                },
                committed_offset: Some(target),
                cancel_programmatic_animation: true,
                finish_previous_animation: animation.has_running_animation,
            };
        }

        NativeUserScrollPlan::Animate(NativeUserScrollAnimationRequest {
            node_id: self.Facts().node_id,
            scroll_element_id: area.GetScrollElementId(),
            source_type,
            granularity,
            target_offset: desired_target,
            interrupt_previous_completion: animation.has_running_animation,
            // ScrollAnimator latches both axes while an animation owns the
            // request, then adds only the axes rejected by CSS overflow.
            accepted_consumption: NativeScrollConsumption {
                did_scroll_x: true,
                did_scroll_y: true,
                unused_scroll_delta_x: unscrollable_axis_delta.x(),
                unused_scroll_delta_y: unscrollable_axis_delta.y(),
            },
            rejected_consumption: Self::Consumption(ScrollOffset::default(), pixel_delta),
            rejected_completion: NativeScrollCompletionMode::Finished,
        })
    }

    /// Updates the native snap target store and returns the remaining source
    /// effects to the DOM/EventHandler owners in source order.
    // Source: scrollable_area.cc:1162-1187.
    pub fn OnScrollFinished(
        &mut self,
        enqueue_scrollend: bool,
    ) -> Option<NativeScrollFinishedPlan> {
        let area = self.native.Area();
        if area.HasBeenDisposed() || area.GetLayoutBox().is_null() {
            return None;
        }
        let node_id = unsafe { area.GetLayoutBox().as_ref() }
            .and_then(|box_| unsafe { box_.GetNode().as_ref() })
            .map(|node| node.InputId());
        let snap_target_notification = enqueue_scrollend
            .then(|| unsafe { &mut *self.native.AreaPointer() }.ScrollFinishedSnapTargets())
            .flatten();
        Some(NativeScrollFinishedPlan {
            node_id,
            scroll_element_id: area.GetScrollElementId(),
            reset_active_smooth_scroll_type: enqueue_scrollend,
            snap_target_notification,
            notify_anchor_viewport_tracker: enqueue_scrollend && node_id.is_some(),
            start_marker_animation: enqueue_scrollend && node_id.is_some(),
            enqueue_scrollend: enqueue_scrollend && node_id.is_some(),
            mark_hover_state_dirty: true,
        })
    }
}

impl NativeUserScrollSource<'_> {
    pub fn PlanSnapForUserScroll(
        &mut self,
        kind: crate::internal::layout_scrollable_area::NativeUserSnapScrollKind,
        direction: foundation::PhysicalDirection,
        line_step: f32,
        animation: crate::native_scroll_source::NativeSnapAnimationState,
    ) -> (bool, crate::native_scroll_source::NativeLayoutSnapPlan) {
        unsafe { &mut *self.native.AreaPointer() }
            .PlanSnapForUserScroll(kind, direction, line_step, animation)
    }
}

impl LayoutEngine {
    pub fn UserScrollLayoutBoxFactsForNode(&self, node_id: u64) -> (bool, bool) {
        let Some(object) = self
            .GetLayoutTree()
            .and_then(|tree| tree.OutputNode(node_id))
        else {
            return (false, false);
        };
        let box_ = unsafe { DynamicTo::<LayoutBox>(object as *const LayoutObject).as_ref() };
        (
            box_.is_some(),
            box_.is_some_and(|box_| box_.IsScrollContainer()),
        )
    }
}
impl NativeUserScrollSource<'_> {
    /// RootFrameViewport.cc:221-251: intersection and scroll-padding resolve
    /// against the same native style after the effective viewport is known.
    pub fn RootVisibleScrollSnapportRect(
        &self,
        visual_offset: gfx::Vector2dF,
        visual_size: gfx::Size,
        include_scrollbars: bool,
    ) -> PhysicalRect {
        use foundation::length_functions::MinimumValueForLength;
        use foundation::{LayoutUnit, PhysicalOffset, PhysicalSize};
        use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;
        let area = self.native.Area();
        let size = area
            .VisibleContentRect(Inclusion(include_scrollbars))
            .size();
        let physical_size = |size: gfx::Size| {
            PhysicalSize::new(
                LayoutUnit::from_signed(size.width()),
                LayoutUnit::from_signed(size.height()),
            )
        };
        let frame = PhysicalRect::new(
            PhysicalOffset::FromVector2dFRound(&area.GetScrollOffset()),
            physical_size(size),
        );
        let mut visual = PhysicalRect::new(
            PhysicalOffset::FromVector2dFRound(&(area.GetScrollOffset() + visual_offset)),
            physical_size(visual_size),
        );
        visual.Intersect(&frame);
        if let Some(box_) = unsafe { area.GetLayoutBox().as_ref() } {
            let style = box_.StyleRef();
            visual.ContractEdges(
                MinimumValueForLength(style.ScrollPaddingTop(), visual.Height()),
                MinimumValueForLength(style.ScrollPaddingRight(), visual.Width()),
                MinimumValueForLength(style.ScrollPaddingBottom(), visual.Height()),
                MinimumValueForLength(style.ScrollPaddingLeft(), visual.Width()),
            );
        }
        visual
    }
}

impl NativeUserScrollSource<'_> {
    pub fn ViewportScrollbarWidths(&self) -> (i32, i32) {
        (
            self.native.Area().VerticalScrollbarWidthDefault(),
            self.native.Area().HorizontalScrollbarHeightDefault(),
        )
    }
}

impl LayoutEngine {
    pub fn TakeNativeScrollbarActivities(
        &mut self,
    ) -> Vec<crate::internal::layout_scrollable_area::NativeScrollbarActivity> {
        if let Some(root) = self
            .GetLayoutTree()
            .map(|tree| tree.Root() as *const LayoutObject)
        {
            let view = unsafe {
                foundation::DynamicTo::<crate::internal::layout_view::LayoutView>(root).as_ref()
            };
            if let Some(view) = view {
                let disposals = view.ScrollAreaDisposalQueue();
                self.user_scroll_disposal_queues_
                    .entry(std::rc::Rc::as_ptr(&disposals) as usize)
                    .or_insert(disposals);
                let queue = view.ScrollbarActivityQueue();
                self.scrollbar_activity_queues_
                    .entry(std::rc::Rc::as_ptr(&queue) as usize)
                    .or_insert(queue);
            }
        }
        let mut result = Vec::new();
        self.scrollbar_activity_queues_.retain(|_, queue| {
            result.append(&mut queue.borrow_mut());
            std::rc::Rc::strong_count(queue) > 1
        });
        result
    }
}

impl LayoutEngine {
    pub fn NativeScrollCompositingOwners(&self) -> Vec<NativeLogicalScrollArea> {
        let Some(root) = self
            .GetLayoutTree()
            .map(|tree| tree.Root() as *const LayoutObject)
        else {
            return Vec::new();
        };
        let Some(view) = (unsafe {
            foundation::DynamicTo::<crate::internal::layout_view::LayoutView>(root).as_ref()
        }) else {
            return Vec::new();
        };
        view.RegisteredScrollableAreas()
            .into_iter()
            .filter_map(|area| NativeLogicalScrollArea::FromPointer(area.Get()))
            .collect()
    }
}
impl NativeUserScrollSource<'_> {
    pub fn PublishUsesCompositedScrolling(&mut self, value: bool) {
        unsafe { &mut *self.native.AreaPointer() }.PublishUsesCompositedScrolling(value);
    }
}

impl LayoutEngine {
    /// ContainerQueryEvaluator.cc:94-103: original containing scroll container,
    /// including OOF containment semantics, rather than a DOM ancestor search.
    pub fn SnappedQueryScrollSourceForNode(&self, node: u64) -> Option<NativeLogicalScrollArea> {
        let _scope = self.NativeScrollPassScope();
        let object = self.GetLayoutTree()?.OutputNode(node)?;
        let box_ = unsafe { object.ContainingScrollContainerAnyAxisDefault().as_ref() }?;
        NativeLogicalScrollArea::FromPointer(box_.GetScrollableArea())
    }
}
impl NativeUserScrollSource<'_> {
    pub fn SnappedQueryTargetNodeIds(&self) -> (Option<u64>, Option<u64>) {
        let decode = |id: scroll_snap::ElementId| {
            if !id.IsValid() {
                return None;
            }
            assert_eq!(
                foundation::NamespaceFromCompositorElementId(foundation::CompositorElementId::new(
                    id.GetInternalValue()
                )),
                foundation::CompositorElementIdNamespace::kDOMNodeId
            );
            Some(
                id.GetInternalValue()
                    >> (foundation::compositor_element_id::kCompositorNamespaceBitCount
                        + foundation::compositor_element_id::kCompositorReservedBitCount),
            )
        };
        self.native
            .Area()
            .SnappedQueryTargetIds()
            .map(|ids| (decode(ids.x), decode(ids.y)))
            .unwrap_or((None, None))
    }
}

pub use crate::internal::layout_scrollable_area::NativePendingHistoryRestore;
impl NativeUserScrollSource<'_> {
    pub fn SetPendingHistoryRestoreScrollOffset(
        &mut self,
        state: NativePendingHistoryRestore,
        should_restore_scroll: bool,
    ) {
        unsafe { &mut *self.native.AreaPointer() }
            .SetPendingHistoryRestoreScrollOffset(state, should_restore_scroll);
    }
    pub fn PendingHistoryRestore(&self) -> Option<NativePendingHistoryRestore> {
        self.native.Area().PendingHistoryRestore().cloned()
    }
    pub fn HasPendingHistoryRestoreScrollOffset(&self) -> bool {
        self.native.Area().HasPendingHistoryRestoreScrollOffset()
    }
    /// The source static selector result contains only original node identities.
    /// Layout resolves the same native objects while this engine borrow roots
    /// them, and returns the fallback programmatic request after anchor restore.
    pub fn PendingHistoryRestoreNeedsSelectorQuery(&self) -> bool {
        self.native.Area().PendingHistoryRestoreNeedsSelectorQuery()
    }
    pub fn ApplyPendingHistoryRestoreScrollOffset(
        &mut self,
        selector_matches: &[u64],
    ) -> crate::native_scroll_source::NativeScrollResult {
        let candidates = selector_matches
            .iter()
            .filter_map(|&id| {
                self._engine
                    .GetLayoutTree()?
                    .GetLayoutObjectForNode(id)
                    .map(|object| object as *const LayoutObject as *mut LayoutObject)
            })
            .collect::<Vec<_>>();
        let area = unsafe { &mut *self.native.AreaPointer() };
        let old = area.GetScrollOffset();
        let request = area.ApplyPendingHistoryRestoreScrollOffset(&candidates);
        let mut result = crate::native_scroll_source::NativeScrollResult::default();
        if let Some(request) = request {
            result.did_scroll = !request.is_noop;
            result.programmatic_requests.push(request);
        }
        let current = area.GetScrollOffset();
        if current != old {
            result.did_scroll = true;
            let node = unsafe { &*area.GetLayoutBox() }.GetNode();
            if let Some(node) = unsafe { node.as_ref() } {
                result
                    .offsets
                    .push(crate::native_scroll_source::NativeScrollOffset {
                    node_id: node.InputId(),
                    offset: crate::internal::layout_input::Offset {
                        x: current.x() as f64,
                        y: current.y() as f64,
                    },
                    scroll_type:
                        layoutng_style::style::scroll_enums::mojom::blink::ScrollType::kAnchoring,
                });
            }
        }
        result
    }
}
impl NativeUserScrollSource<'_> {
    pub fn ContentsSize(&self) -> gfx::Size {
        self.native.Area().ContentsSize()
    }
    pub fn ScrollOffsetInt(&self) -> gfx::Vector2d {
        self.native.Area().ScrollOffsetInt()
    }
}
impl LayoutEngine {
    /// spatial_navigation.cc:445-460, source LayoutView scrollbar policy.
    pub fn SpatialViewportScrollbarModes(&self) -> Option<(bool, bool)> {
        use crate::internal::{
            layout_view::LayoutView, scrollbar_mode::mojom::blink::ScrollbarMode,
        };
        let _scope = self.NativeScrollPassScope();
        let view =
            unsafe { foundation::DynamicTo::<LayoutView>(self.GetLayoutTree()?.Root()).as_ref() }?;
        let (mut horizontal, mut vertical) = (ScrollbarMode::kAuto, ScrollbarMode::kAuto);
        view.CalculateScrollbarModes(&mut horizontal, &mut vertical, None, None);
        Some((
            horizontal == ScrollbarMode::kAlwaysOff,
            vertical == ScrollbarMode::kAlwaysOff,
        ))
    }
}

impl NativeUserScrollSource<'_> {
    /// PaintLayerScrollableArea.cc:1316-1319, the same LayoutBox style owner.
    pub fn ScrollBehaviorStyle(
        &self,
    ) -> layoutng_style::style::scroll_enums::mojom::blink::ScrollBehavior {
        self.native.Area().ScrollBehaviorStyle()
    }
}

impl NativeUserScrollSource<'_> {
    /// ScrollableArea's programmatic helper: smooth style requests animate;
    /// every other used behavior takes the immediate source branch.
    pub fn ResolvedProgrammaticScrollBehavior(
        &self,
    ) -> layoutng_style::style::scroll_enums::mojom::blink::ScrollBehavior {
        use layoutng_style::style::scroll_enums::mojom::blink::ScrollBehavior;
        if self.ScrollBehaviorStyle() == ScrollBehavior::kSmooth {
            ScrollBehavior::kSmooth
        } else {
            ScrollBehavior::kInstant
        }
    }
}

impl NativeUserScrollSource<'_> {
    /// RootFrameViewport.cc:166-168 -> PLSA.cc:2874-2895. Style/theme and
    /// scrollbar allocation remain on their original resident Layout owner.
    pub fn DidUpdateVisualViewport(&mut self, active: bool, maximum: gfx::Vector2dF) {
        let area = unsafe { &mut *self.native.AreaPointer() };
        area.SetVisualViewportScrollability(
            active,
            maximum.x().floor() > 0.,
            maximum.y().floor() > 0.,
        );
        area.DidUpdateVisualViewport();
    }
}
