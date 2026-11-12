//! Synchronous hit-test geometry from the resident LayoutObject/fragment owner.
//! The input expresses the request's one geometry choice; no Interaction
//! result, LayoutObject projection, or retained geometry tree is required.

use foundation::{DynamicTo, PhysicalRect};
use layoutng::internal::layout_box::LayoutBox;
use layoutng::internal::layout_box_model_object::LayoutBoxModelObject;
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;

use crate::physical_box_fragment::PhysicalBoxFragment;

#[allow(non_snake_case)]
impl PhysicalBoxFragment {
    /// Source: core/layout/layout_box_hot.cc:20-72 and
    /// core/layout/physical_box_fragment.cc:772-781. `None` means that the
    /// source cannot reject this object from geometry (including the effective
    /// root scroller), rather than an empty hit-test region.
    pub fn HitTestIntersectionBounds(&self, visual_overflow: bool) -> Option<PhysicalRect> {
        let object = self.GetLayoutObject();
        let box_ = unsafe { DynamicTo::<LayoutBox>(object).as_ref() }?;
        if box_.IsEffectiveRootScroller() {
            return None;
        }
        let model = unsafe { &*object.cast::<LayoutBoxModelObject>() };
        if visual_overflow {
            return Some(model.VisualOverflowRectIncludingFilters());
        }
        let hit_testable_overflow = box_.HasVisualOverflow()
            && (!box_.ShouldClipOverflowAlongBothAxis()
                || (box_.ShouldApplyOverflowClipMargin()
                    && box_
                        .StyleRef()
                        .OverflowClipMargin()
                        .as_ref()
                        .is_some_and(|margin| {
                            margin.GetMargin() > foundation::LayoutUnit::default()
                        })));
        if !hit_testable_overflow {
            return Some(box_.PhysicalBorderBoxRect());
        }
        let mut bounds = model.VisualOverflowRect();
        if box_.ShouldClipOverflowAlongEitherAxis() {
            bounds.Intersect(&box_.OverflowClipRect());
        }
        bounds.Unite(&box_.PhysicalBorderBoxRect());
        Some(bounds)
    }
}
