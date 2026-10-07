// C++: layoutng_inline/inline_item_result.cc.
#![allow(non_snake_case)]

use foundation::{Member, String, StringView, Visitor};
use layoutng::internal::inline_item::{InlineItem, InlineItemType};
use layoutng::internal::inline_item_result::{InlineItemResult, InlineItemResults};
use layoutng::internal::text_fit_scale::TextFitBlockScale;
use layoutng::internal::text_offset_range::TextOffsetRange;

use crate::inline_item_result_ruby_column::InlineItemResultRubyColumn;

// cpp: layoutng_inline/inline_item_result.cc:15-26
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineItemResultNew(
    item: &InlineItem,
    index: u32,
    text_offset: &TextOffsetRange,
    break_anywhere_if_overflow: bool,
    should_create_line_box: bool,
    has_unpositioned_floats: bool,
) -> InlineItemResult {
    InlineItemResult {
        item: Member::from_ptr(item as *const InlineItem as *mut InlineItem),
        item_index: index,
        text_offset: *text_offset,
        break_anywhere_if_overflow,
        should_create_line_box,
        has_unpositioned_floats,
        ..InlineItemResult::default()
    }
}

// cpp: layoutng_inline/inline_item_result.cc:28-33
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineItemResultShapeHyphen(result: &mut InlineItemResult) {
    debug_assert!(!result.hyphen.IsPresent());
    let item = result.item.Get();
    debug_assert!(!item.is_null());
    let style = unsafe { &*item }.Style();
    debug_assert!(!style.is_null());
    result.hyphen.Shape(unsafe { &*style });
}

// cpp: layoutng_inline/inline_item_result.cc:35-60
#[cfg(debug_assertions)]
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineItemResultCheckConsistency(
    result: &InlineItemResult,
    allow_null_shape_result: bool,
) {
    let item = result.item.Get();
    debug_assert!(!item.is_null());
    result.text_offset.AssertValid();
    debug_assert!(result.text_offset.start >= unsafe { &*item }.StartOffset());
    if unsafe { &*item }.Type() == InlineItemType::kOpenRubyColumn {
        return;
    }
    debug_assert!(result.text_offset.end <= unsafe { &*item }.EndOffset());
    if unsafe { &*item }.Type() == InlineItemType::kText {
        if result.Length() == 0 {
            debug_assert!(result.shape_result.Get().is_null());
            return;
        }
        let shape = result.shape_result.Get();
        if allow_null_shape_result && shape.is_null() {
            return;
        }
        debug_assert!(!shape.is_null());
        debug_assert_eq!(result.Length(), unsafe { &*shape }.NumCharacters());
        debug_assert_eq!(result.StartOffset(), unsafe { &*shape }.StartIndex());
        debug_assert_eq!(result.EndOffset(), unsafe { &*shape }.EndIndex());
    }
}

// cpp: layoutng_inline/inline_item_result.cc:62-71
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineItemResultTrace(result: &InlineItemResult, visitor: &mut Visitor<'_>) {
    visitor.Trace(&result.item);
    visitor.Trace(&result.shape_result);
    visitor.Trace(&result.hyphen);
    visitor.Trace(&result.layout_result);
    visitor.Trace(&result.ruby_column);
    visitor.Trace(&result.positioned_float);
    visitor.Trace(&result.exclusion_space_before_position_float);
    visitor.Trace(&result.text_fit_scale);
}

// cpp: layoutng_inline/inline_item_result.cc:73-114
pub fn InlineItemResultToString(
    result: &InlineItemResult,
    ifc_text_content: &String,
    indent: &String,
) -> String {
    let item = unsafe { &*result.item.Get() };
    let mut builder = String::from("");
    builder.push_string(indent);
    builder.push_str("InlineItemResult ");
    builder.push_str(InlineItem::InlineItemTypeToString(item.Type()));
    builder.push_str(" ");

    if item.Type() == InlineItemType::kText {
        let text = StringView::from(ifc_text_content)
            .Substring(result.TextOffset().start, result.TextOffset().Length())
            .EncodeForDebugging();
        builder.push_string(&text);
    } else if result.IsRubyColumn() {
        let layout_object = item.GetLayoutObject();
        if !layout_object.is_null() {
            builder.push_string(&unsafe { &*layout_object }.ToString());
        } else {
            builder.push_str("(anonymous)");
        }
        builder.push_str(", base_line: [\n");
        let mut child_indent = indent.clone();
        child_indent.push_str("\t");

        // The layoutng owner header stores only a forward-declared GC member.
        // The pointed-to allocation is the source-owned inline crate type.
        let ruby_column =
            unsafe { &*(result.ruby_column.Get() as *const InlineItemResultRubyColumn) };
        for child in ruby_column.base_line.Results() {
            builder.push_string(&child.ToString(ifc_text_content, &child_indent));
            builder.push_str("\n");
        }
        for (index, annotation_line) in ruby_column.annotation_line_list.iter().enumerate() {
            builder.push_string(indent);
            builder.push_str("], annotation_line_list[");
            builder.push_string(&String::Number(index));
            builder.push_str("]: [\n");
            for child in annotation_line.Results() {
                builder.push_string(&child.ToString(ifc_text_content, &child_indent));
                builder.push_str("\n");
            }
        }
        builder.push_string(indent);
        builder.push_str("]");
    } else {
        let layout_object = item.GetLayoutObject();
        if !layout_object.is_null() {
            builder.push_string(&unsafe { &*layout_object }.ToString());
        }
    }
    builder
}

// cpp: layoutng_inline/inline_item_result.cc:116-139
#[unsafe(no_mangle)]
pub extern "Rust" fn FindTextScaleInternalFromInline(
    line_items: &InlineItemResults,
    start_index: u32,
    initial_nesting_level: u32,
) -> TextFitBlockScale {
    let mut level = initial_nesting_level;
    for result in line_items.iter().skip(start_index as usize) {
        let item_type = unsafe { &*result.item.Get() }.Type();
        if item_type == InlineItemType::kOpenTag {
            level += 1;
        } else if item_type == InlineItemType::kCloseTag {
            if level == 0 {
                break;
            }
            level -= 1;
        } else if item_type == InlineItemType::kText && level == 0 {
            let scale = result.text_fit_scale.Get();
            if !scale.is_null() {
                return TextFitBlockScale {
                    paint_scale: unsafe { &*scale }.scale,
                    scaled_font: unsafe { &*scale }.font.Get(),
                };
            }
            break;
        }
    }
    TextFitBlockScale {
        paint_scale: 1.0,
        scaled_font: std::ptr::null(),
    }
}
