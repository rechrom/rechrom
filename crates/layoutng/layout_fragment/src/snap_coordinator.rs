//! Chromium snap_coordinator.cc:43-202,250-367; scrollable_area.cc:1052-1064,
//! 1260-1264; paint_layer_scrollable_area.cc:1009-1027,2106-2139.
//! Native fragments and their live ComputedStyle owners supply the cc data.
use super::*;
use crate::internal::layout_box_model_object::LayoutBoxModelObject;
use crate::internal::layout_object::BoxQuadType;
use crate::internal::map_coordinates_flags::MapCoordinatesMode;
use foundation::WritingDirectionMode;
use layoutng_style::style::scroll_snap_data::cc as style_cc;
use scroll_snap::geometry as cc_gfx;
use scroll_snap::scroll_snap_data as cc;
use scroll_snap::scroll_utils::ScrollUtils;
use scroll_snap::snap_selection_strategy::{SnapSelectionStrategy, SnapStopAlwaysFilter};

/// Source command values, independent of DOM Event/Interaction enum storage.
#[derive(Clone, Copy, Debug)]
pub enum NativePageScrollCommand {
    Up,
    Down,
    Left,
    Right,
    InlineStart,
    InlineEnd,
    BlockStart,
    BlockEnd,
}
#[derive(Clone, Copy, Debug)]
pub enum NativeUserSnapScrollKind {
    Direction,
    Page,
    Document,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeSnapError {
    /// ColumnPseudoElement::ColumnRect has not yet crossed the native boundary.
    ColumnPseudoElementGeometry,
    /// The native pseudo attachment must supply UltimateOriginatingElement.
    OverscrollAreaOrigin,
}
/// The source SetScrollOffset call, ready for its programmatic animator owner.
/// Page routes this value; it must preserve Auto/Smooth semantics.
#[derive(Clone, Copy, Debug)]
pub struct NativePageScrollRequest {
    pub node_id: u64,
    pub scroll_element_id: CompositorElementId,
    pub source_type: scroll_snap::scroll_source_type::ScrollSourceType,
    pub offset: ScrollOffset,
    /// InitiateScrollAnimation must still cancel a running programmatic
    /// animation for a no-op request; the command result itself is false.
    pub is_noop: bool,
    pub behavior: layoutng_style::style::scroll_enums::mojom::blink::ScrollBehavior,
}
fn Point(point: gfx::PointF) -> cc_gfx::PointF {
    cc_gfx::PointF::new(point.x(), point.y())
}
fn Rect(rect: PhysicalRect) -> cc_gfx::RectF {
    cc_gfx::RectF::FromOriginSize(
        cc_gfx::PointF::new(rect.X().ToFloat(), rect.Y().ToFloat()),
        cc_gfx::SizeF::new(rect.Width().ToFloat(), rect.Height().ToFloat()),
    )
}
fn Alignment(value: style_cc::SnapAlignment) -> cc::SnapAlignment {
    match value {
        style_cc::SnapAlignment::kNone => cc::SnapAlignment::kNone,
        style_cc::SnapAlignment::kStart => cc::SnapAlignment::kStart,
        style_cc::SnapAlignment::kEnd => cc::SnapAlignment::kEnd,
        style_cc::SnapAlignment::kCenter => cc::SnapAlignment::kCenter,
    }
}
fn Flip(value: cc::SnapAlignment) -> cc::SnapAlignment {
    match value {
        cc::SnapAlignment::kStart => cc::SnapAlignment::kEnd,
        cc::SnapAlignment::kEnd => cc::SnapAlignment::kStart,
        value => value,
    }
}
fn GetPhysicalSnapType(container: &LayoutBox) -> cc::ScrollSnapType {
    let value = container.StyleRef().GetScrollSnapType();
    let horizontal = container.StyleRef().IsHorizontalWritingMode();
    let axis = match value.axis {
        style_cc::SnapAxis::kInline => {
            if horizontal {
                cc::SnapAxis::kX
            } else {
                cc::SnapAxis::kY
            }
        }
        style_cc::SnapAxis::kBlock => {
            if horizontal {
                cc::SnapAxis::kY
            } else {
                cc::SnapAxis::kX
            }
        }
        style_cc::SnapAxis::kX => cc::SnapAxis::kX,
        style_cc::SnapAxis::kY => cc::SnapAxis::kY,
        style_cc::SnapAxis::kBoth => cc::SnapAxis::kBoth,
        style_cc::SnapAxis::kPair => cc::SnapAxis::kPair,
    };
    cc::ScrollSnapType::new(
        value.is_none,
        axis,
        match value.strictness {
            style_cc::SnapStrictness::kMandatory => cc::SnapStrictness::kMandatory,
            style_cc::SnapStrictness::kProximity => cc::SnapStrictness::kProximity,
        },
    )
}
fn GetPhysicalAlignment(
    area_style: &ComputedStyle,
    container_style: &ComputedStyle,
    area_rect: PhysicalRect,
    container_rect: PhysicalRect,
    resolving: WritingDirectionMode,
) -> cc::ScrollSnapAlign {
    let align = area_style.GetScrollSnapAlign();
    let container_direction = container_style.GetWritingDirection();
    let area_direction = area_style.GetWritingDirection();
    let flip_x = if area_rect.Width() > container_rect.Width() {
        area_direction.IsFlippedX()
    } else {
        container_direction.IsFlippedX()
    };
    let flip_y = if area_rect.Height() > container_rect.Height() {
        area_direction.IsFlippedY()
    } else {
        container_direction.IsFlippedY()
    };
    let (x, y) = if resolving.IsHorizontal() {
        (
            Alignment(align.alignment_inline),
            Alignment(align.alignment_block),
        )
    } else {
        (
            Alignment(align.alignment_block),
            Alignment(align.alignment_inline),
        )
    };
    cc::ScrollSnapAlign::new(
        if flip_y { Flip(y) } else { y },
        if flip_x { Flip(x) } else { x },
    )
}
/// snap_coordinator.cc:304-367. Each source element is the native SnapArea's
/// existing Member, not an independently looked-up or copied layout tree.
fn CalculateSnapAreaData(
    area: &Element,
    container: &LayoutBox,
    axes: PhysicalAxes,
    resolving: WritingDirectionMode,
) -> Result<cc::SnapAreaData, NativeSnapError> {
    let style = unsafe { &*area.GetComputedStyle() };
    let mode = MapCoordinatesFlags::from_mode(MapCoordinatesMode::kTraverseDocumentBoundaries)
        | MapCoordinatesFlags::from_mode(MapCoordinatesMode::kIgnoreScrollOffset);
    let mut quads = foundation::Vector::<gfx::QuadF>::default();
    let Some(box_) = (unsafe { area.GetLayoutBox().as_ref() }) else {
        return Err(NativeSnapError::ColumnPseudoElementGeometry);
    };
    box_.QuadsInAncestor(
        &mut quads,
        container as *const LayoutBox as *const LayoutBoxModelObject,
        mode,
        BoxQuadType::kBorder,
    );
    let mut area_rect = PhysicalRect::default();
    for quad in quads.iter() {
        area_rect.UniteIfNonZero(&PhysicalRect::EnclosingRect(&quad.BoundingBox()));
    }
    area_rect.Expand(&style.ScrollMarginStrut());
    let mut align = GetPhysicalAlignment(
        style,
        container.StyleRef(),
        area_rect,
        container.PhysicalBorderBoxRect(),
        resolving,
    );
    if axes & kPhysicalAxesHorizontal == kPhysicalAxesNone {
        align.alignment_inline = cc::SnapAlignment::kNone;
    }
    if axes & kPhysicalAxesVertical == kPhysicalAxesNone {
        align.alignment_block = cc::SnapAlignment::kNone;
    }
    // compositor_element_id.cc:16-32,52-56. The namespace is retained verbatim;
    // remapping to cc's own namespace is reserved for cc-generated identities.
    let id = CompositorElementIdFromUniqueObjectId(
        area.InputId(),
        CompositorElementIdNamespace::kDOMNodeId,
    );
    Ok(cc::SnapAreaData::new(
        &align,
        &Rect(area_rect),
        style.ScrollSnapStop() == foundation::EScrollSnapStop::kAlways,
        area.InputEditingFacts().has_focus_within,
        scroll_snap::ElementId::new(id.GetInternalValue()),
    ))
}
impl PaintLayerScrollableArea {
    pub fn GetSnapContainerData(&self) -> Option<&cc::SnapContainerData> {
        self.snap_container_data_.as_ref()
    }
    pub fn SetTargetedSnapAreaId(&mut self, id: Option<scroll_snap::ElementId>) {
        self.targeted_snap_area_id_ = id;
    }
    /// snap_coordinator.cc:79-202. Called at the native post-layout snap phase.
    pub fn UpdateSnapContainerData(&mut self) -> Result<bool, NativeSnapError> {
        let container = unsafe { &*self.GetLayoutBox() };
        let snap_type = GetPhysicalSnapType(container);
        if snap_type.is_none {
            if self.snap_container_data_.take().is_some() {
                unsafe { &mut *self.GetLayoutBox() }.SetNeedsPaintPropertyUpdate();
                self.scrollsnapchanging_target_ids_ = None;
                self.scrollsnapchange_target_ids_ = None;
                self.snapped_query_target_ids_ = None;
                self.snap_target_removal_pending_ = true;
            }
            return Ok(false);
        }
        let mut data = cc::SnapContainerData::FromType(snap_type);
        data.set_max_position(Point(
            self.ScrollOffsetToPosition(&self.MaximumScrollOffset()),
        ));
        data.set_targeted_area_id(&self.targeted_snap_area_id_);
        let style = container.StyleRef();
        let mut rect = container.OverflowClipRect();
        rect.ContractEdges(
            MinimumValueForLength(style.ScrollPaddingTop(), rect.Height()),
            MinimumValueForLength(style.ScrollPaddingRight(), rect.Width()),
            MinimumValueForLength(style.ScrollPaddingBottom(), rect.Height()),
            MinimumValueForLength(style.ScrollPaddingLeft(), rect.Width()),
        );
        data.set_rect(&Rect(rect));
        data.set_has_horizontal_writing_mode(style.IsHorizontalWritingMode());
        if data.scroll_snap_type().strictness == cc::SnapStrictness::kProximity {
            let mut size = rect.size;
            size.ScaleFloat(1.0 / 3.0);
            data.set_proximity_range(&cc_gfx::PointF::new(
                size.width.ToFloat(),
                size.height.ToFloat(),
            ));
        }
        let old_targets = self
            .snap_container_data_
            .as_ref()
            .map(|data| *data.GetTargetSnapAreaElementIds())
            .unwrap_or_default();
        let mut new_targets = cc::TargetSnapAreaElementIds::default();
        if container.IsOverscrollAreaParent() {
            self.AddOverscrollSnapAreas(container, &mut data, &mut new_targets, &old_targets)?;
        } else {
            for fragment in container.PhysicalFragments() {
                for item in fragment.SnapAreas() {
                    if let Some(area) = unsafe { item.GetElementIfConsumed().as_ref() } {
                        let area = CalculateSnapAreaData(
                            area,
                            container,
                            item.ConsumedAxes(),
                            item.ContainerWritingDirectionMode()
                                .expect("resolved snap area writing mode"),
                        )?;
                        if old_targets.x == area.element_id {
                            new_targets.x = old_targets.x;
                        }
                        if old_targets.y == area.element_id {
                            new_targets.y = old_targets.y;
                        }
                        data.AddSnapAreaData(area);
                    }
                }
            }
        }
        data.SetTargetSnapAreaElementIds(new_targets);
        if self.snap_container_data_.as_ref() != Some(&data) {
            unsafe { &mut *self.GetLayoutBox() }.SetNeedsPaintPropertyUpdate();
            self.snap_container_data_ = Some(data);
            return Ok(true);
        }
        Ok(false)
    }
    // Source: snap_coordinator.cc:203-254. Both source snap identities refer
    // to actual native nodes; no parent/hit-test identity is substituted.
    fn AddOverscrollSnapAreas(
        &self,
        container: &LayoutBox,
        data: &mut cc::SnapContainerData,
        targets: &mut cc::TargetSnapAreaElementIds,
        old: &cc::TargetSnapAreaElementIds,
    ) -> Result<(), NativeSnapError> {
        let pseudo =
            unsafe { container.GetNode().as_ref() }.ok_or(NativeSnapError::OverscrollAreaOrigin)?;
        let origin = unsafe { pseudo.UltimateOriginatingElement().as_ref() }
            .ok_or(NativeSnapError::OverscrollAreaOrigin)?;
        let rect = container.OverflowClipRect();
        let scroll_origin = container.ScrollOrigin();
        let initial = cc::SnapAreaData {
            rect: cc_gfx::RectF::FromOriginSize(
                cc_gfx::PointF::new(
                    rect.X().ToFloat() + scroll_origin.x() as f32,
                    rect.Y().ToFloat() + scroll_origin.y() as f32,
                ),
                cc_gfx::SizeF::new(rect.Width().ToFloat(), rect.Height().ToFloat()),
            ),
            element_id: scroll_snap::ElementId::new(
                CompositorElementIdFromUniqueObjectId(
                    pseudo.InputId(),
                    CompositorElementIdNamespace::kDOMNodeId,
                )
                .GetInternalValue(),
            ),
            scroll_snap_align: cc::ScrollSnapAlign::new(
                cc::SnapAlignment::kCenter,
                cc::SnapAlignment::kCenter,
            ),
            must_snap: true,
            ..Default::default()
        };
        let mut area = CalculateSnapAreaData(
            origin,
            container,
            kPhysicalAxesBoth,
            WritingDirectionMode::new(
                foundation::WritingMode::kHorizontalTb,
                foundation::TextDirection::kLtr,
            ),
        )?;
        area.must_snap = false;
        area.scroll_snap_align =
            cc::ScrollSnapAlign::new(cc::SnapAlignment::kCenter, cc::SnapAlignment::kCenter);
        data.AddSnapAreaData(initial);
        data.AddSnapAreaData(area);
        if old.x == initial.element_id || old.x == area.element_id {
            targets.x = old.x;
        }
        if old.y == initial.element_id || old.y == area.element_id {
            targets.y = old.y;
        }
        Ok(())
    }
    /// paint_layer_scrollable_area.cc:2106-2139; type None still updates targets.
    pub fn GetSnapPositionAndSetTarget(
        &mut self,
        strategy: &dyn SnapSelectionStrategy,
    ) -> Option<gfx::PointF> {
        let data = self.snap_container_data_.as_ref()?;
        if data.size() == 0 {
            return None;
        }
        let result = data.FindSnapPosition(strategy);
        self.SetTargetSnapAreaElementIds(result.target_element_ids);
        if result.r#type == cc::SnapPositionType::kNone {
            None
        } else {
            Some(gfx::PointF::new(result.position.x(), result.position.y()))
        }
    }
    /// Source: scrollable_area.cc:1052-1064,1260-1264; area.cc:1009-1027.
    /// The returned request belongs to SetScrollOffset's programmatic animator;
    /// this source stage never substitutes an immediate offset for Auto/Smooth.
    pub fn ScrollByPageWithSnap(
        &mut self,
        direction: foundation::PhysicalDirection,
        fractional_offsets_enabled: bool,
    ) -> NativePageScrollRequest {
        use foundation::PhysicalDirection::*;
        let (dx, dy) = match direction {
            kUp => (0., -1.),
            kDown => (0., 1.),
            kLeft => (-1., 0.),
            kRight => (1., 0.),
        };
        let size = self
            .VisibleScrollSnapportRect(IncludeScrollbarsInRect::kExcludeScrollbars)
            .PixelSnappedSize();
        let current = self.ScrollPosition();
        let strategy = <dyn SnapSelectionStrategy>::CreateForPageScroll(
            Point(current),
            cc_gfx::Vector2dF::new(dx, dy),
            cc_gfx::Size::new(size.width(), size.height()),
            fractional_offsets_enabled,
            SnapStopAlwaysFilter::kIgnore,
        );
        let target = self
            .GetSnapPositionAndSetTarget(strategy.as_ref())
            .unwrap_or_else(|| {
                gfx::PointF::new(
                    current.x() + dx * ScrollUtils::CalculatePageStep(size.width()) as f32,
                    current.y() + dy * ScrollUtils::CalculatePageStep(size.height()) as f32,
                )
            });
        self.ProgrammaticScrollRequest(
            self.ScrollPositionToOffset(&target),
            scroll_snap::scroll_source_type::ScrollSourceType::kNone,
            fractional_offsets_enabled,
        )
    }
    // Source: overscroll_area_tracker.cc:34-51. Update BOTH target IDs even
    // for a zero/no-op offset before entering the programmatic animator.
    pub fn OverscrollScrollToAndSnapBothAxes(
        &mut self,
        mut offset: ScrollOffset,
        fractional_offsets_enabled: bool,
    ) -> NativePageScrollRequest {
        let position = Point(self.ScrollOffsetToPosition(&offset));
        let strategy = <dyn SnapSelectionStrategy>::CreateForEndPosition(&position, true, true);
        if let Some(position) = self.GetSnapPositionAndSetTarget(strategy.as_ref()) {
            offset = self.ScrollPositionToOffset(&position);
        }
        self.ProgrammaticScrollRequest(
            offset,
            scroll_snap::scroll_source_type::ScrollSourceType::kAbsoluteScroll,
            fractional_offsets_enabled,
        )
    }
    /// PLSA::ScrollIntoView, cc:2592-2609. Both axes snap using the
    /// original target identities before the source sequence is submitted.
    pub fn PlanScrollIntoView(
        &mut self,
        offset: ScrollOffset,
        behavior: layoutng_style::style::scroll_enums::mojom::blink::ScrollBehavior,
    ) -> NativePageScrollRequest {
        let endpoint = self.ScrollOffsetToPosition(&offset);
        let strategy =
            <dyn SnapSelectionStrategy>::CreateForEndPosition(&Point(endpoint), true, true);
        let endpoint = self
            .GetSnapPositionAndSetTarget(strategy.as_ref())
            .unwrap_or(endpoint);
        let offset = self.ScrollPositionToOffset(&endpoint);
        let mut request = self.ProgrammaticScrollRequest(
            offset,
            scroll_snap::scroll_source_type::ScrollSourceType::kAbsoluteScroll,
            foundation::RuntimeEnabledFeatures::FractionalScrollOffsetsEnabled(),
        );
        use layoutng_style::style::scroll_enums::mojom::blink::ScrollBehavior;
        let resolved = if behavior == ScrollBehavior::kAuto {
            self.ScrollBehaviorStyle()
        } else {
            behavior
        };
        request.behavior = if resolved == ScrollBehavior::kSmooth {
            ScrollBehavior::kSmooth
        } else {
            ScrollBehavior::kInstant
        };
        request
    }
    // The same SetScrollOffsetInternal/InitiateScrollAnimation source branch,
    // used after page-scroll or overscroll end-position snap selection.
    pub(super) fn ProgrammaticScrollRequest(
        &self,
        offset: ScrollOffset,
        source_type: scroll_snap::scroll_source_type::ScrollSourceType,
        fractional_offsets_enabled: bool,
    ) -> NativePageScrollRequest {
        // ScrollableArea::SetScrollOffsetInternal, cc:335-357: clamp first,
        // then resolve Auto against the same native style owner.
        let offset = self.ClampScrollOffset(&offset);
        let comparison_offset = if fractional_offsets_enabled {
            offset
        } else {
            let rounded = gfx::ToRoundedVector2d(&offset);
            ScrollOffset::new(rounded.x() as f32, rounded.y() as f32)
        };
        NativePageScrollRequest {
            node_id: unsafe { &*unsafe { &*self.GetLayoutBox() }.GetNode() }.InputId(),
            scroll_element_id: self.GetScrollElementId(),
            source_type,
            offset,
            is_noop: self.GetScrollOffset() == comparison_offset,
            // SetScrollOffsetInternal resolves Auto to style (cc:352-354);
            // InitiateScrollAnimation (cc:515-517,555-565) animates only Smooth.
            // CSS `auto` therefore takes the source without-animation branch.
            behavior: if self.ScrollBehaviorStyle()
                == layoutng_style::style::scroll_enums::mojom::blink::ScrollBehavior::kSmooth
            {
                layoutng_style::style::scroll_enums::mojom::blink::ScrollBehavior::kSmooth
            } else {
                layoutng_style::style::scroll_enums::mojom::blink::ScrollBehavior::kInstant
            },
        }
    }
}

impl PaintLayerScrollableArea {
    // Native equivalent of this Page's LocalFrameView alias, never a copied tree.
    fn SnapUpdateView(&self) -> Option<&crate::internal::layout_view::LayoutView> {
        let mut object = self.GetLayoutBox() as *mut LayoutObject;
        while let Some(owner) = unsafe { object.as_ref() } {
            if owner.IsLayoutView() {
                return Some(unsafe {
                    &*(object as *const crate::internal::layout_view::LayoutView)
                });
            }
            object = owner.Parent();
        }
        None
    }
    // PaintLayerScrollableArea::EnqueueForSnapUpdateIfNeeded, cc:2368-2388.
    pub(crate) fn EnqueueForSnapUpdateIfNeeded(&self) {
        let box_ = unsafe { &*self.GetLayoutBox() };
        if !box_.IsScrollContainer() {
            return;
        }
        if box_.IsOverscrollAreaParent()
            || box_.PhysicalFragments().into_iter().any(|fragment| {
                !fragment.SnapAreas().is_empty() || self.GetSnapContainerData().is_some()
            })
        {
            if let Some(view) = self.SnapUpdateView() {
                view.AddPendingSnapUpdate(self as *const Self as *mut Self);
            }
        }
    }
    pub(crate) fn RemovePendingSnapUpdate(&self) {
        if let Some(view) = self.SnapUpdateView() {
            view.RemovePendingSnapUpdate(self as *const Self as *mut Self);
        }
    }
}

/// Live animator facts for the same native ScrollableArea identity. The caller
/// supplies these from animator lifecycle; the snap engine selects the action.
#[derive(Clone, Copy, Debug)]
pub struct NativeSnapAnimationState {
    pub scroll_element_id: CompositorElementId,
    pub has_running_animation: bool,
    pub programmatic_target: Option<ScrollOffset>,
}
#[derive(Clone, Copy, Debug)]
pub struct NativePendingSnapTarget {
    pub node_id: u64,
    pub scroll_element_id: CompositorElementId,
}
/// Values selected by PaintLayerScrollableArea::UpdateSnappedTargets. The DOM
/// event queue and Style snapped-query supplement consume these decisions.
#[derive(Clone, Copy, Debug)]
pub struct NativeSnapTargetNotification {
    pub target: NativePendingSnapTarget,
    pub targets: Option<cc::TargetSnapAreaElementIds>,
    pub enqueue_scrollsnapchange: bool,
    pub enqueue_scrollsnapchanging: bool,
}
/// Source PerformSnapping's decided effects. Cancellation and target-event
/// delivery remain synchronous operations of the receiving animator/DOM owners.
#[derive(Clone, Copy, Debug)]
pub struct NativeLayoutSnapPlan {
    pub target: NativePendingSnapTarget,
    pub request: Option<NativePageScrollRequest>,
    pub cancel_user_animation: bool,
    pub cancel_programmatic_animation: bool,
    update_snapped_targets: bool,
    snap_targets: Option<cc::TargetSnapAreaElementIds>,
    enqueue_scrollsnapchange: bool,
    pub target_notification: Option<NativeSnapTargetNotification>,
}
impl PaintLayerScrollableArea {
    pub(crate) fn PendingSnapTarget(&self) -> NativePendingSnapTarget {
        NativePendingSnapTarget {
            node_id: unsafe { &*unsafe { &*self.GetLayoutBox() }.GetNode() }.InputId(),
            scroll_element_id: self.GetScrollElementId(),
        }
    }
    /// ScrollableArea::SnapAfterLayout/SelectScrollBehaviorForSnapReason/
    /// PerformSnapping, scrollable_area.cc:1267-1357. Native data and strategy
    /// owners retain target selection; the animator is an explicit live input.
    pub(crate) fn PlanSnapAfterLayout(
        &mut self,
        animation: NativeSnapAnimationState,
    ) -> NativeLayoutSnapPlan {
        let strategy =
            <dyn SnapSelectionStrategy>::CreateForTargetElement(Point(self.ScrollPosition()));
        self.PlanNativeSnapping(
            strategy.as_ref(),
            animation,
            scroll_snap::scroll_source_type::ScrollSourceType::kStationaryScroll,
            true,
        )
        .1
    }
    /// ScrollableArea.cc:1218-1264. Only native geometry constructs strategy;
    /// the Page adapter supplies the ChromeClient's actual line-step scalar.
    pub fn PlanSnapForUserScroll(
        &mut self,
        kind: NativeUserSnapScrollKind,
        direction: foundation::PhysicalDirection,
        line_step: f32,
        animation: NativeSnapAnimationState,
    ) -> (bool, NativeLayoutSnapPlan) {
        use foundation::PhysicalDirection::*;
        let (dx, dy) = match direction {
            kUp => (0., -1.),
            kDown => (0., 1.),
            kLeft => (-1., 0.),
            kRight => (1., 0.),
        };
        let current = Point(self.ScrollPosition());
        let fractional = foundation::RuntimeEnabledFeatures::FractionalScrollOffsetsEnabled();
        let (strategy, source_type): (Box<dyn SnapSelectionStrategy>, _) = match kind {
            NativeUserSnapScrollKind::Direction => (
                <dyn SnapSelectionStrategy>::CreateForDirection(
                    current,
                    cc_gfx::Vector2dF::new(dx * line_step, dy * line_step),
                    fractional,
                    SnapStopAlwaysFilter::kIgnore,
                ),
                scroll_snap::scroll_source_type::ScrollSourceType::kRelativeScroll,
            ),
            NativeUserSnapScrollKind::Page => {
                let size = self
                    .VisibleScrollSnapportRect(IncludeScrollbarsInRect::kExcludeScrollbars)
                    .PixelSnappedSize();
                (
                    <dyn SnapSelectionStrategy>::CreateForPageScroll(
                        current,
                        cc_gfx::Vector2dF::new(dx, dy),
                        cc_gfx::Size::new(size.width(), size.height()),
                        fractional,
                        SnapStopAlwaysFilter::kIgnore,
                    ),
                    scroll_snap::scroll_source_type::ScrollSourceType::kRelativeScroll,
                )
            }
            NativeUserSnapScrollKind::Document => {
                let end = cc_gfx::PointF::new(
                    current.x()
                        + dx * self.ScrollSize(ScrollbarOrientation::kHorizontalScrollbar) as f32,
                    current.y()
                        + dy * self.ScrollSize(ScrollbarOrientation::kVerticalScrollbar) as f32,
                );
                (
                    <dyn SnapSelectionStrategy>::CreateForEndPosition(&end, dx != 0., dy != 0.),
                    scroll_snap::scroll_source_type::ScrollSourceType::kAbsoluteScroll,
                )
            }
        };
        self.PlanNativeSnapping(strategy.as_ref(), animation, source_type, false)
    }
    /// ScrollableArea.cc:1282-1363. The returned plan retains cancellation,
    /// target comparison and animation request decisions in the native owner.
    fn PlanNativeSnapping(
        &mut self,
        strategy: &dyn SnapSelectionStrategy,
        animation: NativeSnapAnimationState,
        source_type: scroll_snap::scroll_source_type::ScrollSourceType,
        layout_snap: bool,
    ) -> (bool, NativeLayoutSnapPlan) {
        assert_eq!(animation.scroll_element_id, self.GetScrollElementId());
        let previous = self
            .GetSnapContainerData()
            .map(|data| *data.GetTargetSnapAreaElementIds());
        let mut plan = NativeLayoutSnapPlan {
            target: self.PendingSnapTarget(),
            request: None,
            cancel_user_animation: false,
            cancel_programmatic_animation: false,
            update_snapped_targets: true,
            snap_targets: previous,
            enqueue_scrollsnapchange: false,
            target_notification: None,
        };
        let Some(position) = self.GetSnapPositionAndSetTarget(strategy) else {
            self.FinishSnapTargetPlan(&mut plan);
            return (false, plan);
        };
        use layoutng_style::style::scroll_enums::mojom::blink::ScrollBehavior;
        let targets = *self
            .GetSnapContainerData()
            .unwrap()
            .GetTargetSnapAreaElementIds();
        plan.snap_targets = Some(targets);
        let behavior = if !layout_snap {
            ScrollBehavior::kSmooth
        } else if foundation::RuntimeEnabledFeatures::AuthorSpecifiedLayoutScrollSnapBehaviorEnabled(
        ) && Some(targets) != previous
        {
            self.ScrollBehaviorStyle()
        } else {
            ScrollBehavior::kInstant
        };
        let offset = self.ScrollPositionToOffset(&position);
        if behavior == ScrollBehavior::kSmooth && animation.programmatic_target == Some(offset) {
            plan.update_snapped_targets = false;
            return (true, plan);
        }
        if self.scrollsnapchanging_target_ids_.is_none() {
            self.scrollsnapchanging_target_ids_ = Some(targets);
        }
        plan.cancel_user_animation = true;
        plan.cancel_programmatic_animation = true;
        let request = self.ProgrammaticScrollRequest(
            offset,
            source_type,
            foundation::RuntimeEnabledFeatures::FractionalScrollOffsetsEnabled(),
        );
        // Snap reason, rather than CSS auto, selected this behavior.
        let request = NativePageScrollRequest {
            behavior: if behavior == ScrollBehavior::kSmooth {
                ScrollBehavior::kSmooth
            } else {
                ScrollBehavior::kInstant
            },
            ..request
        };
        plan.update_snapped_targets = request.is_noop;
        plan.request = Some(request);
        self.FinishSnapTargetPlan(&mut plan);
        (true, plan)
    }
    // PaintLayerScrollableArea.cc:3445-3469. Dependency-driven snapped-query
    // snapshot creation and DOM event delivery receive these typed targets;
    // source target comparison/storage remains with this ScrollableArea.
    fn FinishSnapTargetPlan(&mut self, plan: &mut NativeLayoutSnapPlan) {
        if !plan.update_snapped_targets {
            return;
        }
        let Some(data) = self.GetSnapContainerData() else {
            return;
        };
        let targets = *data.GetTargetSnapAreaElementIds();
        plan.snap_targets = Some(targets);
        if self.scrollsnapchange_target_ids_ != Some(targets) {
            self.scrollsnapchange_target_ids_ = Some(targets);
            self.snapped_query_target_ids_ = Some(targets);
            plan.enqueue_scrollsnapchange = true;
        }
        plan.target_notification = Some(NativeSnapTargetNotification {
            target: plan.target,
            targets: Some(targets),
            enqueue_scrollsnapchange: plan.enqueue_scrollsnapchange,
            enqueue_scrollsnapchanging: false,
        });
    }
    // PaintLayerScrollableArea.cc:2064-2091, no layout/snap-selection work.
    pub fn UpdateFocusDataForSnapAreas(&mut self) {
        let box_ = unsafe { &*self.GetLayoutBox() };
        let Some(data) = self.snap_container_data_.as_mut() else {
            return;
        };
        let indices: std::collections::BTreeMap<_, _> = (0..data.size())
            .map(|i| (data.at(i).element_id, i))
            .collect();
        for fragment in box_.PhysicalFragments() {
            for item in fragment.SnapAreas() {
                if let Some(element) = unsafe { item.GetElementIfConsumed().as_ref() } {
                    let id = CompositorElementIdFromUniqueObjectId(
                        element.InputId(),
                        CompositorElementIdNamespace::kDOMNodeId,
                    );
                    let id = scroll_snap::ElementId::new(id.GetInternalValue());
                    let index = *indices
                        .get(&id)
                        .expect("same source SnapArea belongs to snap data");
                    data.UpdateSnapAreaFocus(index, element.InputEditingFacts().has_focus_within);
                }
            }
        }
    }
}

impl PaintLayerScrollableArea {
    pub(crate) fn TakeSnapTargetRemovalNotification(&mut self) -> Option<NativeLayoutSnapPlan> {
        if !std::mem::take(&mut self.snap_target_removal_pending_) {
            return None;
        }
        Some(NativeLayoutSnapPlan {
            target: self.PendingSnapTarget(),
            request: None,
            cancel_user_animation: false,
            cancel_programmatic_animation: false,
            update_snapped_targets: true,
            snap_targets: None,
            enqueue_scrollsnapchange: true,
            target_notification: Some(NativeSnapTargetNotification {
                target: self.PendingSnapTarget(),
                targets: None,
                enqueue_scrollsnapchange: true,
                enqueue_scrollsnapchanging: false,
            }),
        })
    }
}

impl PaintLayerScrollableArea {
    // ScrollableArea::OnScrollFinished, cc:1162-1169; same native target store.
    pub fn ScrollFinishedSnapTargets(&mut self) -> Option<NativeSnapTargetNotification> {
        let mut plan = NativeLayoutSnapPlan {
            target: self.PendingSnapTarget(),
            request: None,
            cancel_user_animation: false,
            cancel_programmatic_animation: false,
            update_snapped_targets: true,
            snap_targets: None,
            enqueue_scrollsnapchange: false,
            target_notification: None,
        };
        self.FinishSnapTargetPlan(&mut plan);
        plan.target_notification
    }
}

impl PaintLayerScrollableArea {
    // PaintLayerScrollableArea.cc:2051-2061; used by both native strategy and
    // ScrollingCoordinator::DidCompositorScroll's original target ids.
    pub fn SetTargetSnapAreaElementIds(&mut self, targets: cc::TargetSnapAreaElementIds) -> bool {
        let Some(data) = self.snap_container_data_.as_mut() else {
            return false;
        };
        if !data.SetTargetSnapAreaElementIds(targets) {
            return false;
        }
        unsafe { &mut *self.GetLayoutBox() }.SetNeedsPaintPropertyUpdate();
        true
    }
}

impl PaintLayerScrollableArea {
    // PLSA.cc:3475-3509,3513-3524. Impl's decided snap target identities
    // update the same rare-data store; this source owner decides notification.
    pub fn CompositorSnapChangingNotification(
        &mut self,
        targets: cc::TargetSnapAreaElementIds,
    ) -> Option<NativeSnapTargetNotification> {
        if self.GetSnapContainerData().is_none() {
            return None;
        }
        if self.scrollsnapchanging_target_ids_ == Some(targets) {
            return None;
        }
        self.scrollsnapchanging_target_ids_ = Some(targets);
        self.snapped_query_target_ids_ = Some(targets);
        Some(NativeSnapTargetNotification {
            target: self.PendingSnapTarget(),
            targets: Some(targets),
            enqueue_scrollsnapchange: false,
            enqueue_scrollsnapchanging:
                foundation::RuntimeEnabledFeatures::CSSScrollSnapChangingEventEnabled(),
        })
    }
}
