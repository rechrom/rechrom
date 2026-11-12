//! RangeInputType::AdjustStyle, range_input_type.cc:296-299. The inherited
//! InputTypeView::AdjustStyle is empty (input_type_view.h:116).
use super::computed_style::ComputedStyleBuilder;
pub fn AdjustRangeInputStyle(builder: &mut ComputedStyleBuilder) {
    builder.SetInlineBlockBaselineEdge(foundation::EInlineBlockBaselineEdge::kBorderBox);
}
