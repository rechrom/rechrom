//! Chromium sticky-position property refresh.
//!
//! This source is compiled at the reciprocal LayoutNG assembly boundary. The
//! fragment package owns the complete object/axis loop so its trace owner and
//! sampling frame agree even when block or scroll code initiates the refresh.
//!
//! Post-layout constraint construction follows Chromium's per-scroll-container
//! queue: PaintLayerScrollableArea::UpdateAfterLayout enqueues the area on the
//! frame view when its fragments consumed sticky descendants, and
//! LocalFrameView::ExecutePendingStickyUpdates visits only those areas'
//! StickyDescendants(). No document-wide object scan exists.
//! Source: core/paint/paint_layer_scrollable_area.cc:2391-2415;
//!         core/frame/local_frame_view.cc:5542-5578.

#![allow(non_snake_case)]

use foundation::{gfx, DynamicTo};
use layoutng_geometry::geometry::axis::{
    kPhysicalAxesHorizontal, kPhysicalAxesVertical, PhysicalAxes, PhysicalAxis,
};

use crate::internal::{
    layout_box::LayoutBox, layout_box_model_object::LayoutBoxModelObject,
    layout_node_metadata::Element, layout_object::LayoutObject,
    layout_scrollable_area::PaintLayerScrollableArea,
};

// PaintLayer's containing-scroll query records whether a viewport-fixed
// ancestor separates this sticky box from root scrolling. Preserve that source
// constraint input for ComputeStickyPositionConstraints.
fn IsFixedToViewBelowScrollContainer(
    box_model: &LayoutBoxModelObject,
    scroll_container: *const LayoutBox,
) -> bool {
    let mut ancestor = box_model.Parent();
    while !ancestor.is_null()
        && ancestor as *const LayoutObject != scroll_container as *const LayoutObject
    {
        let ancestor_box = DynamicTo::<LayoutBox>(ancestor);
        if !ancestor_box.is_null() && unsafe { &*ancestor_box }.IsFixedToView(std::ptr::null()) {
            return true;
        }
        ancestor = unsafe { &*ancestor }.Parent();
    }
    false
}

impl PaintLayerScrollableArea {
    // cpp: core/paint/paint_layer_scrollable_area.cc:2406-2415
    // Enqueue ourselves for a sticky update if we have any sticky descendants.
    pub(crate) fn EnqueueForStickyUpdateIfNeeded(&self) {
        let box_ = unsafe { &*self.GetLayoutBox() };
        if box_
            .PhysicalFragments()
            .into_iter()
            .any(|fragment| fragment.HasConsumedStickyDescendants())
        {
            if let Some(view) = self.StickyUpdateView() {
                view.AddPendingStickyUpdate(self as *const Self as *mut Self);
            }
        }
    }

    // GetLayoutBox()->GetFrameView(). The root area can be allocated before
    // its LayoutView is reachable from the global input owner, so fall back
    // to the same attached-ancestor alias the snap queue uses.
    pub(crate) fn StickyUpdateView(&self) -> Option<&crate::internal::layout_view::LayoutView> {
        self.RegisteredFrameView().or_else(|| self.SnapUpdateView())
    }

    // cpp: core/paint/paint_layer_scrollable_area.cc:2391-2404
    // Each consumed StickyDescendants() list is in fragment-propagation order
    // (fragment_builder.cc:184-219): a sticky box precedes the sticky
    // descendants it propagates, so nested sticky offsets observe updated
    // ancestors. Sticky ancestors are searched only below this scroll
    // container, so queued areas are independent of each other.
    #[inline(never)]
    pub(crate) fn UpdateAllStickyConstraints(&self) {
        let scroll_container = self.GetLayoutBox() as *const LayoutBox;
        let scroll_position = self.ScrollPosition();
        for fragment in unsafe { &*scroll_container }.PhysicalFragments() {
            for item in fragment.StickyDescendants() {
                let sticky_descendant = item.GetIfConsumed();
                if sticky_descendant.is_null() {
                    continue;
                }
                let sticky_descendant = unsafe { &mut *sticky_descendant };
                let axes = item.ConsumedAxes();
                let is_fixed_to_view =
                    IsFixedToViewBelowScrollContainer(sticky_descendant, scroll_container);
                let data = sticky_descendant.ComputeStickyPositionConstraintsForLayout(
                    scroll_container,
                    is_fixed_to_view,
                    axes,
                );
                sticky_descendant.SetStickyConstraints(data);
                sticky_descendant
                    .StickyConstraints()
                    .ComputeStickyOffset(&scroll_position, axes);
            }
        }
    }
}

#[inline(never)]
pub(crate) fn update_sticky_offsets(
    objects: impl IntoIterator<Item = *mut LayoutObject>,
    reuse_constraints: bool,
) {
    // Logical preorder lets nested sticky offsets observe updated ancestors.
    for object in objects {
        let box_model = DynamicTo::<LayoutBoxModelObject>(object);
        if box_model.is_null()
            || !unsafe { &*box_model }
                .StyleRef()
                .HasStickyConstrainedPosition()
        {
            continue;
        }
        let box_model = unsafe { &mut *box_model };
        let constrained_axes = LayoutBoxModelObject::StickyConstrainedAxes(box_model.StyleRef());
        let mut update_axis = |axis: PhysicalAxis, axis_mask: PhysicalAxes| {
            if !(constrained_axes & axis_mask).is_nonzero() {
                return;
            }
            let scroll_container = box_model.ContainingScrollContainer(axis);
            if scroll_container.is_null() {
                return;
            }
            let cached = box_model.StickyConstraints().AxisData(axis);
            // cpp: core/paint/paint_layer_scrollable_area.cc:2420-2440
            // A layout-clean scroll changes the offset, not the containing
            // geometry. Keep each axis's rooted constraints only for its same
            // source owner; missing/changed owners still build real constraints.
            let can_reuse = reuse_constraints
                && !cached.is_null()
                && unsafe { &*cached }.containing_scroll_container.Get() as *const LayoutBox
                    == scroll_container;
            if !can_reuse {
                let is_fixed_to_view =
                    IsFixedToViewBelowScrollContainer(box_model, scroll_container);
                let constraints = box_model.ComputeStickyPositionConstraintsForLayout(
                    scroll_container,
                    is_fixed_to_view,
                    axis_mask,
                );
                box_model.SetStickyConstraints(constraints);
            } else {
                box_model.SetNeedsPaintPropertyUpdate();
            }
            let mut scroll_position = gfx::PointF::default();
            let area = unsafe { &*scroll_container }.GetScrollableArea();
            if !area.is_null() {
                scroll_position = unsafe { &*area }.ScrollPosition();
            } else if unsafe { &*scroll_container }.IsLayoutView() {
                let source = unsafe { &*scroll_container }.GetNode();
                if !source.is_null() && unsafe { &*source }.IsElementNode() {
                    if let Some(data) = unsafe { &*(source as *const Element) }.InputElementData() {
                        scroll_position = gfx::PointF::new(
                            data.scroll_offset.x as f32,
                            data.scroll_offset.y as f32,
                        );
                    }
                }
            }
            box_model
                .StickyConstraints()
                .ComputeStickyOffset(&scroll_position, axis_mask);
        };
        update_axis(PhysicalAxis::kHorizontal, kPhysicalAxesHorizontal);
        update_axis(PhysicalAxis::kVertical, kPhysicalAxesVertical);
    }
}
