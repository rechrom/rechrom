use font_engine::ShapeResultView;
use foundation::{LayoutUnit, Member, Visitor};
use layoutng_geometry::geometry::box_strut::BoxStrut;

// cpp: layoutng/internal/mathml_paint_info.h:22-53
pub struct MathMLPaintInfo {
    pub operator_character: u16,
    pub operator_shape_result_view: Member<ShapeResultView>,
    pub operator_inline_size: LayoutUnit,
    pub operator_ascent: LayoutUnit,
    pub operator_descent: LayoutUnit,
    pub radical_base_margins: BoxStrut,
    pub radical_operator_inline_offset: Option<LayoutUnit>,
}

#[allow(non_snake_case)]
impl MathMLPaintInfo {
    // cpp: layoutng/internal/mathml_paint_info.h:46-46
    pub const NON_CHARACTER: u16 = 0xFFFF;

    // cpp: layoutng/internal/mathml_paint_info.h:24-38
    pub fn new(
        operator_character: u16,
        operator_shape_result_view: *const ShapeResultView,
        operator_inline_size: LayoutUnit,
        operator_ascent: LayoutUnit,
        operator_descent: LayoutUnit,
        radical_base_margins: BoxStrut,
        radical_operator_inline_offset: Option<LayoutUnit>,
    ) -> Self {
        Self {
            operator_character,
            operator_shape_result_view: Member::from_ptr(
                operator_shape_result_view as *mut ShapeResultView,
            ),
            operator_inline_size,
            operator_ascent,
            operator_descent,
            radical_base_margins,
            radical_operator_inline_offset,
        }
    }

    pub fn new_without_radical(
        operator_character: u16,
        operator_shape_result_view: *const ShapeResultView,
        operator_inline_size: LayoutUnit,
        operator_ascent: LayoutUnit,
        operator_descent: LayoutUnit,
    ) -> Self {
        Self::new(
            operator_character,
            operator_shape_result_view,
            operator_inline_size,
            operator_ascent,
            operator_descent,
            BoxStrut::default(),
            None,
        )
    }

    // cpp: layoutng/internal/mathml_paint_info.h:40-42
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.operator_shape_result_view);
    }

    // cpp: layoutng/internal/mathml_paint_info.h:43-45
    pub fn IsRadicalOperator(&self) -> bool {
        self.radical_operator_inline_offset.is_some()
    }
}
