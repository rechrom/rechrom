use foundation::LayoutUnit;
use layoutng_geometry::geometry::logical_rect::LogicalRect;

// cpp: layoutng/internal/shapes/shape.h:41-55
#[derive(Clone, Copy, Debug, Default)]
pub struct LineSegment {
    pub logical_left: LayoutUnit,
    pub logical_right: LayoutUnit,
    pub is_valid: bool,
}

impl LineSegment {
    // cpp: layoutng/internal/shapes/shape.h:47-50
    pub fn new(logical_left: f32, logical_right: f32) -> Self {
        Self {
            logical_left: LayoutUnit::from_f32(logical_left),
            logical_right: LayoutUnit::from_f32(logical_right),
            is_valid: true,
        }
    }
}

// cpp: layoutng/internal/shapes/shape.h:95-95
#[derive(Default)]
pub struct ShapeBase {
    margin_: f32,
}

// C++'s pure virtual shape interface becomes an object-safe trait. Concrete
// shape owners embed ShapeBase to retain the base's margin state.
// cpp: layoutng/internal/shapes/shape.h:57-96
#[allow(non_snake_case)]
pub trait Shape {
    fn base(&self) -> &ShapeBase;
    fn base_mut(&mut self) -> &mut ShapeBase;

    // cpp: layoutng/internal/shapes/shape.h:69-72
    fn ShapeMarginLogicalBoundingBox(&self) -> LogicalRect;
    fn IsEmpty(&self) -> bool;
    fn GetExcludedInterval(
        &self,
        logical_top: LayoutUnit,
        logical_height: LayoutUnit,
    ) -> LineSegment;

    // cpp: layoutng/internal/shapes/shape.h:73-77
    fn LineOverlapsShapeMarginBounds(&self, line_top: LayoutUnit, line_height: LayoutUnit) -> bool {
        self.LineOverlapsBoundingBox(line_top, line_height, &self.ShapeMarginLogicalBoundingBox())
    }

    // cpp: layoutng/internal/shapes/shape.h:78-81
    fn SetShapeMargin(&mut self, margin: f32) {
        self.base_mut().margin_ = margin;
    }

    fn ShapeMargin(&self) -> f32 {
        self.base().margin_
    }

    // cpp: layoutng/internal/shapes/shape.h:84-93
    fn LineOverlapsBoundingBox(
        &self,
        line_top: LayoutUnit,
        line_height: LayoutUnit,
        rect: &LogicalRect,
    ) -> bool {
        if rect.IsEmpty() {
            return false;
        }
        let rect_line_top = rect.offset.block_offset;
        (line_top < rect.BlockEndOffset() && line_top + line_height > rect_line_top)
            || (line_height == LayoutUnit::default() && line_top == rect_line_top)
    }
}
