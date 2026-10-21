use foundation::{Member, Visitor};

use super::exclusion_area::{ExclusionArea, ExclusionAreaPtrArray};

// cpp: layoutng/internal/exclusions/shape_exclusions.h:15-34
#[derive(Default)]
pub struct ShapeExclusions {
    pub line_left_shapes: ExclusionAreaPtrArray,
    pub line_right_shapes: ExclusionAreaPtrArray,
}

impl Clone for ShapeExclusions {
    // cpp: layoutng/internal/exclusions/shape_exclusions.h:25-27
    fn clone(&self) -> Self {
        Self {
            line_left_shapes: self
                .line_left_shapes
                .iter()
                .map(|area| Member::<ExclusionArea>::from_ptr(area.Get()))
                .collect(),
            line_right_shapes: self
                .line_right_shapes
                .iter()
                .map(|area| Member::<ExclusionArea>::from_ptr(area.Get()))
                .collect(),
        }
    }
}

#[allow(non_snake_case)]
impl ShapeExclusions {
    // cpp: layoutng/internal/exclusions/shape_exclusions.h:28-31
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.line_left_shapes);
        visitor.Trace(&self.line_right_shapes);
    }
}
