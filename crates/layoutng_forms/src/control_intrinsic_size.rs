#![allow(non_snake_case)]

use foundation::{kIndefiniteSize, DynamicTo, EOverflow, LayoutUnit, To};
use layoutng_assembly::internal::form_control_sizing_service::ComputedStyleControlSizingExt;
use layoutng_assembly::internal::form_control_types::FormControlType;
use layoutng_assembly::internal::form_node_metadata::{
    FormNodeMetadataExternal, HTMLInputElement, HTMLTextAreaElement,
};
use layoutng_assembly::internal::layout_box::LayoutBox;
use layoutng_assembly::internal::layout_node_metadata::Element;
use layoutng_style::style::appearance::AppearanceValue;

use crate::control_theme_size::ThemePartIntrinsicSize;
use crate::file_upload_intrinsic_size::FileUploadControlIntrinsicInlineSize;
use crate::form_node_metadata::FormNodeMetadataProvider;
use crate::layout_text_control::{GetAvgCharWidth, HasValidAvgCharWidth, ScrollbarThickness};

// cpp: layoutng_forms/control_intrinsic_size.h:34-34
// cpp: layoutng_forms/control_intrinsic_size.cc:38-64
pub fn ControlIntrinsicInlineSize(box_: &LayoutBox) -> LayoutUnit {
    let appearance = box_.StyleRef().EffectiveAppearance();
    if appearance == AppearanceValue::kCheckbox || appearance == AppearanceValue::kRadio {
        return ThemePartIntrinsicSize(box_.StyleRef(), box_.ControlThemeForLayout(), appearance)
            .inline_size;
    }
    let element = DynamicTo::<Element>(box_.GetNode());
    if element.is_null() {
        return kIndefiniteSize;
    }
    let apply_fixed_size = box_.StyleRef().ApplyControlFixedSize(element.cast());
    let input = DynamicTo::<HTMLInputElement>(element);
    if !input.is_null() {
        let input = unsafe { &*input };
        let type_ = input.FormControlType();
        if box_.IsTextField() && apply_fixed_size {
            return TextFieldIntrinsicInlineSize(input, box_);
        }
        if type_ == FormControlType::kInputFile && apply_fixed_size {
            return FileUploadControlIntrinsicInlineSize(input, box_);
        }
        if type_ == FormControlType::kInputRange {
            return SliderIntrinsicInlineSize(box_);
        }
    }
    let textarea = DynamicTo::<HTMLTextAreaElement>(element);
    if !textarea.is_null() && apply_fixed_size {
        return TextAreaIntrinsicInlineSize(unsafe { &*textarea }, box_);
    }
    kIndefiniteSize
}

// cpp: layoutng_forms/control_intrinsic_size.h:35-35
// cpp: layoutng_forms/control_intrinsic_size.cc:66-83
pub fn ControlIntrinsicBlockSize(box_: &LayoutBox, children_have_geometry: bool) -> LayoutUnit {
    let _ = children_have_geometry;
    let appearance = box_.StyleRef().EffectiveAppearance();
    if appearance == AppearanceValue::kCheckbox || appearance == AppearanceValue::kRadio {
        return ThemePartIntrinsicSize(box_.StyleRef(), box_.ControlThemeForLayout(), appearance)
            .block_size;
    }
    if !box_.StyleRef().ApplyControlFixedSize(box_.GetNode()) {
        return kIndefiniteSize;
    }
    if box_.IsTextField() {
        return TextFieldIntrinsicBlockSize(
            unsafe { &*To::<HTMLInputElement>(box_.GetNode()) },
            box_,
        );
    }
    if box_.IsTextArea() {
        return TextAreaIntrinsicBlockSize(
            unsafe { &*To::<HTMLTextAreaElement>(box_.GetNode()) },
            box_,
        );
    }
    kIndefiniteSize
}

// cpp: layoutng_forms/control_intrinsic_size.h:36-36
// cpp: layoutng_forms/control_intrinsic_size.cc:85-98
pub fn TextAreaIntrinsicInlineSize(textarea: &HTMLTextAreaElement, box_: &LayoutBox) -> LayoutUnit {
    let style = box_.StyleRef();
    let mut scrollbar_thickness = 0;
    if style.OverflowBlockDirection() == EOverflow::kScroll
        || style.OverflowBlockDirection() == EOverflow::kAuto
    {
        scrollbar_thickness = ScrollbarThickness(box_);
    }
    LayoutUnit::from_f32((GetAvgCharWidth(style) * textarea.cols() as f32).ceil())
        + scrollbar_thickness
}

// cpp: layoutng_forms/control_intrinsic_size.h:37-37
// cpp: layoutng_forms/control_intrinsic_size.cc:100-127
pub fn TextAreaIntrinsicBlockSize(textarea: &HTMLTextAreaElement, box_: &LayoutBox) -> LayoutUnit {
    let mut scrollbar_thickness = 0;
    if box_.StyleRef().OverflowInlineDirection() == EOverflow::kScroll {
        scrollbar_thickness = ScrollbarThickness(box_);
    }

    let inner_editor = textarea.InnerEditorElement();
    let editor_box = if !inner_editor.is_null() {
        unsafe { &*inner_editor }.GetLayoutBox()
    } else {
        std::ptr::null_mut()
    };
    let inner_box = if !editor_box.is_null() {
        DynamicTo::<LayoutBox>(unsafe { &*editor_box }.SlowFirstChild())
    } else {
        std::ptr::null_mut()
    };

    let target_box = if !inner_box.is_null() {
        unsafe { &*inner_box }
    } else if !editor_box.is_null() {
        unsafe { &*editor_box }
    } else {
        box_
    };
    target_box.FirstLineStyleRef().ComputedLineHeightAsFixed() * textarea.rows()
        + scrollbar_thickness
}

// cpp: layoutng_forms/control_intrinsic_size.h:38-38
// cpp: layoutng_forms/control_intrinsic_size.cc:129-170
pub fn TextFieldIntrinsicInlineSize(input: &HTMLInputElement, box_: &LayoutBox) -> LayoutUnit {
    let mut factor = 0;
    let includes_decoration = FormNodeMetadataProvider::GetSizeWithDecoration(input, &mut factor);
    if factor <= 0 {
        factor = 20;
    }

    let char_width = GetAvgCharWidth(box_.StyleRef());
    let mut float_result = char_width * factor as f32;

    let mut max_char_width = 0.0;
    let font = unsafe { &*box_.StyleRef().GetFont() };
    if HasValidAvgCharWidth(font) {
        max_char_width = unsafe { &*font.PrimaryFont() }.MaxCharWidth();
    }
    if max_char_width > char_width {
        float_result += max_char_width - char_width;
    }

    let mut result = LayoutUnit::from_f32(float_result.ceil());
    if includes_decoration {
        let spin_button = input.SpinButtonForLayout();
        let spin_box = if !spin_button.is_null() {
            unsafe { &*spin_button }.GetLayoutBox()
        } else {
            std::ptr::null_mut()
        };
        if !spin_box.is_null() {
            let spin_box = unsafe { &*spin_box };
            let logical_width = spin_box.StyleRef().LogicalWidth();
            result += spin_box.BorderPaddingInlineSize();
            if logical_width.IsPercent() {
                let value = logical_width.PercentValue();
                if value != 100.0 {
                    result += result * value / (100.0 - value);
                }
            } else if logical_width.IsFixed() {
                result += logical_width.Pixels();
            }
        }
    }
    result
}

// cpp: layoutng_forms/control_intrinsic_size.h:39-39
// cpp: layoutng_forms/control_intrinsic_size.cc:172-181
pub fn TextFieldIntrinsicBlockSize(input: &HTMLInputElement, box_: &LayoutBox) -> LayoutUnit {
    let inner_editor = input.InnerEditorElement();
    let editor_box = if !inner_editor.is_null() {
        unsafe { &*inner_editor }.GetLayoutBox()
    } else {
        std::ptr::null_mut()
    };
    let target_box = if !editor_box.is_null() {
        unsafe { &*editor_box }
    } else {
        box_
    };
    target_box.FirstLineStyleRef().ComputedLineHeightAsFixed()
}

// cpp: layoutng_forms/control_intrinsic_size.h:40-40
// cpp: layoutng_forms/control_intrinsic_size.cc:183-186
pub fn SliderIntrinsicInlineSize(box_: &LayoutBox) -> LayoutUnit {
    const K_DEFAULT_TRACK_LENGTH: i32 = 129;
    LayoutUnit::from_f32(K_DEFAULT_TRACK_LENGTH as f32 * box_.StyleRef().EffectiveZoom())
}
