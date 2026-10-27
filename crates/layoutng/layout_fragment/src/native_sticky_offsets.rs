//! Native sticky-position property refresh.
//!
//! This source is compiled at the reciprocal LayoutNG assembly boundary. The
//! fragment package owns the complete object/axis loop so its trace owner and
//! sampling frame agree even when block or scroll code initiates the refresh.

use foundation::{gfx, DynamicTo};
use layoutng_geometry::geometry::axis::{
    kPhysicalAxesHorizontal, kPhysicalAxesVertical, PhysicalAxes, PhysicalAxis,
};

use crate::internal::{
    layout_box::LayoutBox, layout_box_model_object::LayoutBoxModelObject,
    layout_node_metadata::Element, layout_object::LayoutObject,
};

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
            // native owner; missing/changed owners still build real constraints.
            let can_reuse = reuse_constraints
                && !cached.is_null()
                && unsafe { &*cached }.containing_scroll_container.Get() as *const LayoutBox
                    == scroll_container;
            if !can_reuse {
                // PaintLayer's containing-scroll query records whether a
                // viewport-fixed ancestor separates this sticky box from root
                // scrolling. Preserve that native constraint input here.
                let mut ancestor = box_model.Parent();
                let mut is_fixed_to_view = false;
                while !ancestor.is_null()
                    && ancestor as *const LayoutObject != scroll_container as *const LayoutObject
                {
                    let ancestor_box = DynamicTo::<LayoutBox>(ancestor);
                    if !ancestor_box.is_null()
                        && unsafe { &*ancestor_box }.IsFixedToView(std::ptr::null())
                    {
                        is_fixed_to_view = true;
                        break;
                    }
                    ancestor = unsafe { &*ancestor }.Parent();
                }
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
