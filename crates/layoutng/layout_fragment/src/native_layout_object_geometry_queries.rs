use super::*;

impl LayoutObject {
    // cpp: layoutng/internal/layout_object_core_services.cc:417-433
    pub fn ContainingScrollContainer(&self, axis: PhysicalAxis) -> *const LayoutBox {
        LayoutObjectContainingScrollContainer(self, axis)
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:471-487
    pub fn LocalToAncestorPoint(
        &self,
        local_point: &gfx::PointF,
        ancestor: *const LayoutBoxModelObject,
        mode: MapCoordinatesFlags,
    ) -> gfx::PointF {
        LayoutObjectLocalToAncestorPoint(self, local_point, ancestor, mode)
    }
}

// cpp: layoutng/internal/layout_object_core_services.cc:417-433
#[inline(never)]
pub(super) fn LayoutObjectContainingScrollContainer(
    object: &LayoutObject,
    axis: PhysicalAxis,
) -> *const LayoutBox {
    object.CheckIsNotDestroyed();
    let mut ancestor = object.Parent();
    while !ancestor.is_null() {
        let box_ = DynamicTo::<LayoutBox>(ancestor);
        if !box_.is_null() {
            let box_ = unsafe { &*box_ };
            let style = box_.StyleRef();
            // Chromium walks actual scroll owners before filtering axes. The
            // merged html/LayoutView retains authored overflow; its native
            // viewport ownership already includes propagation.
            if box_.IsScrollContainer()
                && (box_.IsLayoutView()
                    || (axis == PhysicalAxis::kVertical && style.IsOverflowValueScrollableY())
                    || (axis == PhysicalAxis::kHorizontal && style.IsOverflowValueScrollableX()))
            {
                return box_ as *const LayoutBox;
            }
        }
        ancestor = unsafe { &*ancestor }.Parent();
    }
    std::ptr::null()
}

// cpp: layoutng/internal/layout_object_core_services.cc:471-487
#[inline(never)]
pub(super) fn LayoutObjectLocalToAncestorPoint(
    object: &LayoutObject,
    local_point: &gfx::PointF,
    ancestor: *const LayoutBoxModelObject,
    mode: MapCoordinatesFlags,
) -> gfx::PointF {
    object.CheckIsNotDestroyed();
    let mut point = *local_point;
    let mut current: *const LayoutObject = object;
    while !current.is_null() && current != ancestor as *const LayoutObject {
        let container = unsafe { &*current }.Container();
        if container.is_null() {
            break;
        }
        let offset = unsafe { &*current }.OffsetFromContainer(container, mode);
        point.Offset(offset.left.ToFloat(), offset.top.ToFloat());
        current = container;
    }
    point
}
