#![allow(non_snake_case)]

use font_engine::fonts::shaping::text_width::ComputeTextWidth;
use foundation::{LayoutUnit, String as BlinkString, StringView};
use layoutng_assembly::internal::block_node::BlockNode;
use layoutng_assembly::internal::constraint_space_builder::ConstraintSpaceBuilder;
use layoutng_assembly::internal::form_node_metadata::{FormNodeMetadataExternal, HTMLInputElement};
use layoutng_assembly::internal::layout_box::LayoutBox;
use layoutng_assembly::internal::layout_input_node::MinMaxSizesFloatInput;
use layoutng_assembly::internal::length_utils::SizeType;

use crate::form_node_metadata::FormNodeMetadataProvider;

// cpp: layoutng_forms/file_upload_intrinsic_size.h:9-9
// cpp: layoutng_forms/file_upload_intrinsic_size.cc:36-69
pub fn FileUploadControlIntrinsicInlineSize(
    input: &HTMLInputElement,
    box_: &LayoutBox,
) -> LayoutUnit {
    // cpp: layoutng_forms/file_upload_intrinsic_size.cc:38-44
    const K_AFTER_BUTTON_SPACING: i32 = 4;
    const K_DEFAULT_WIDTH_NUM_CHARS: i32 = 34;
    const K_CHARACTER: u16 = '0' as u16;

    // C++ constructs String from a one-element UChar span, so retain its
    // 16-bit storage width before passing a StringView to the font engine.
    // cpp: layoutng_forms/file_upload_intrinsic_size.cc:45-48
    let character_as_string = BlinkString::from_utf16(&[K_CHARACTER]);
    let character_view = StringView::from_blink_string_range(&character_as_string, 0, 1);
    let min_default_label_width = K_DEFAULT_WIDTH_NUM_CHARS as f32
        * ComputeTextWidth(&character_view, box_.StyleRef().GetFont());

    // cpp: layoutng_forms/file_upload_intrinsic_size.cc:50-52
    let label = FormNodeMetadataProvider::FileNoFileLabelForLayout(input);
    let label_view = StringView::from_blink_string_range(&label, 0, label.length());
    let mut default_label_width = ComputeTextWidth(&label_view, box_.StyleRef().GetFont());

    // cpp: layoutng_forms/file_upload_intrinsic_size.cc:53-66
    let button = input.UploadButton();
    if !button.is_null() {
        let button_box = unsafe { &*button }.GetLayoutBox();
        if !button_box.is_null() {
            let button_style = unsafe { &*button_box }.StyleRef();
            let mode = button_style.GetWritingMode();
            // The C++ overload takes WritingMode directly. The translated
            // builder names that overload new_without_parent_space.
            let builder = ConstraintSpaceBuilder::new_without_parent_space(
                mode,
                button_style.GetWritingDirection(),
                true,
                true,
                false,
            );
            let max = BlockNode::new(button_box)
                .ComputeMinMaxSizes(
                    mode,
                    SizeType::kIntrinsic,
                    &builder.ToConstraintSpace(),
                    MinMaxSizesFloatInput::default(),
                )
                .sizes
                .max_size;
            default_label_width +=
                max.ToFloat() + K_AFTER_BUTTON_SPACING as f32 * box_.StyleRef().EffectiveZoom();
        }
    }

    // cpp: layoutng_forms/file_upload_intrinsic_size.cc:67-68
    LayoutUnit::from_f32(min_default_label_width.max(default_label_width).ceil())
}
