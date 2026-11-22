// Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: spin_button_element.cc:218-255. Borrow native geometry/style once.
use super::*;
use crate::internal::map_coordinates_flags::MapCoordinatesFlags;
impl LayoutEngine {
    pub fn SpinButtonGeometry(
        &self,
        node_id: u64,
        absolute: foundation::gfx::PointF,
    ) -> Option<foundation::spin_button_geometry::SpinButtonGeometry> {
        let _scope = self.NativeScrollPassScope();
        let object = self.tree_.as_ref()?.OutputNode(node_id)?;
        let box_ = DynamicTo::<LayoutBox>(object as *const LayoutObject as *mut LayoutObject);
        if box_.is_null() {
            return None;
        }
        let box_ = unsafe { &*box_ };
        let local = box_.AbsoluteToLocalPointF(&absolute, MapCoordinatesFlags::default());
        let direction = unsafe { object.Style().as_ref() }.map_or_else(
            || {
                foundation::WritingDirectionMode::new(
                    foundation::WritingMode::kHorizontalTb,
                    foundation::TextDirection::kLtr,
                )
            },
            |style| style.GetWritingDirection(),
        );
        Some(foundation::spin_button_geometry::SpinButtonGeometry {
            local: foundation::gfx::point_conversions::ToRoundedPoint(local),
            size: box_.StitchedSize(),
            line_over: direction.LineOver(),
        })
    }
    pub fn InvalidateSpinButtonPaint(&mut self, node_id: u64) -> bool {
        let _scope = self.NativeScrollPassScope();
        let Some(object) = self
            .tree_
            .as_mut()
            .and_then(|tree| tree.OutputNodeMut(node_id))
        else {
            return false;
        };
        object.SetShouldDoFullPaintInvalidation();
        true
    }
}
