// C++: layoutng_inline/layout_text.cc. Non-inline methods of the LayoutText
// type owned by //src/layoutng; linked in the shared assembly.
#![allow(non_snake_case)]

use font_engine::text::native::character_break_iterator::LengthOfGraphemeCluster;
use font_engine::Font;
use foundation::{
    DynamicTo, ETextSecurity, ETextTransform, String, StringView, TextOffsetMap, To, Visitor,
};
use layoutng::internal::inline_item_span::InlineItemSpan;
use layoutng::internal::layout_invalidation_reason;
use layoutng::internal::layout_node_metadata::{Node, Text};
use layoutng::internal::layout_object::{LayoutObject, StyleChangeContext};
use layoutng::internal::layout_text::LayoutText;
use layoutng::internal::layout_text_combine::LayoutTextCombine;
use layoutng::internal::variable_length_transform_result::VariableLengthTransformResult;
use layoutng_fragment_tree::fragment_items::FragmentItems;
use layoutng_fragment_tree::inline_cursor::InlineCursor;
use layoutng_fragment_tree::inline_items_data::InlineItemsData;
use layoutng_geometry::geometry::logical_offset::LogicalOffset;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::style_difference::StyleDifference;
use std::ffi::c_char;

// cpp: layoutng_inline/layout_text.cc:31-41
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextNew(node: *mut Node, text: String) -> LayoutText {
    LayoutText::new_base_for_inline(node, text)
}

// cpp: layoutng_inline/layout_text.cc:43-46
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextTrace(this: &LayoutText, visitor: &mut Visitor<'_>) {
    this.inline_items_.Trace(visitor);
    LayoutObject::Trace(this, visitor);
}

// cpp: layoutng_inline/layout_text.cc:48-50
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextScaledFont(this: &LayoutText) -> *const Font {
    if this.RuntimeClass() == layoutng::internal::layout_object::LayoutObjectClass::SvgInlineText {
        let objects = layoutng::internal::layout_pass_scope::LayoutObjectFactoryScope::Objects();
        let scaled = unsafe { objects.as_ref() }
            .and_then(|objects| objects.svg_inline_text_scaled_font)
            .expect("SVG inline text scaled font is not installed");
        return scaled(this);
    }
    this.StyleRef().GetFont()
}

// cpp: layoutng_inline/layout_text.cc:52-55
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextIsWordBreak(this: &LayoutText) -> bool {
    this.CheckIsNotDestroyed();
    false
}

// cpp: layoutng_inline/layout_text.cc:57-81
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextStyleDidChange(
    this: &mut LayoutText,
    diff: StyleDifference,
    old_style: *const ComputedStyle,
    new_style: &ComputedStyle,
    _context: &StyleChangeContext,
) {
    this.CheckIsNotDestroyed();
    if diff.NeedsFullLayout() {
        this.SetNeedsLayoutAndIntrinsicWidthsRecalc(unsafe {
            std::ptr::addr_of!(layout_invalidation_reason::kStyleChange)
        });
    }
    let old_transform = if old_style.is_null() {
        ETextTransform::kNone
    } else {
        unsafe { &*old_style }.TextTransform()
    };
    let old_security = if old_style.is_null() {
        ETextSecurity::kNone
    } else {
        unsafe { &*old_style }.TextSecurity()
    };
    if old_transform != new_style.TextTransform()
        || old_security != new_style.TextSecurity()
        || (!old_style.is_null()
            && old_transform != ETextTransform::kNone
            && unsafe { &*old_style }.Locale() != new_style.Locale())
    {
        this.TransformAndSecureOriginalText();
    }
    if diff.needs_reshape() {
        this.valid_ng_items_ = false;
        this.SetNeedsCollectInlines();
    }
    this.SetIsHorizontalWritingMode(new_style.IsHorizontalWritingMode());
}

// cpp: layoutng_inline/layout_text.cc:83-88
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextWillBeDestroyed(this: &mut LayoutText) {
    this.CheckIsNotDestroyed();
    this.RemoveAndDestroyTextBoxes();
    this.valid_ng_items_ = false;
    this.Remove();
}

// cpp: layoutng_inline/layout_text.cc:90-97
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextRemoveAndDestroyTextBoxes(this: &mut LayoutText) {
    this.CheckIsNotDestroyed();
    if this.FirstInlineFragmentItemIndex() != 0 {
        FragmentItems::LayoutObjectWillBeDestroyed(this);
        this.first_fragment_item_index_ = 0;
    }
    this.DeleteTextBoxes();
}

// cpp: layoutng_inline/layout_text.cc:99-102
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextDeleteTextBoxes(this: &mut LayoutText) {
    this.CheckIsNotDestroyed();
    this.has_abstract_inline_text_box_ = false;
}

// cpp: layoutng_inline/layout_text.cc:104-108
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextClearFirstInlineFragmentItemIndex(this: &mut LayoutText) {
    this.CheckIsNotDestroyed();
    assert!(this.IsInLayoutNGInlineFormattingContext());
    this.first_fragment_item_index_ = 0;
}

// cpp: layoutng_inline/layout_text.cc:110-115
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextSetFirstInlineFragmentItemIndex(
    this: &mut LayoutText,
    index: usize,
) {
    this.CheckIsNotDestroyed();
    assert!(this.IsInLayoutNGInlineFormattingContext());
    debug_assert_ne!(index, 0);
    this.first_fragment_item_index_ = index;
}

// cpp: layoutng_inline/layout_text.cc:117-121
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextInLayoutNGInlineFormattingContextWillChange(
    this: &mut LayoutText,
    _: bool,
) {
    this.CheckIsNotDestroyed();
    this.first_fragment_item_index_ = 0;
    this.has_abstract_inline_text_box_ = false;
}

// cpp: layoutng_inline/layout_text.cc:123-126
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextHasInlineFragments(this: &LayoutText) -> bool {
    this.CheckIsNotDestroyed();
    this.IsInLayoutNGInlineFormattingContext() && this.first_fragment_item_index_ != 0
}

// cpp: layoutng_inline/layout_text.cc:128-132
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextOriginalText(this: &LayoutText) -> String {
    this.CheckIsNotDestroyed();
    let text_node = DynamicTo::<Text>(this.GetNode());
    if text_node.is_null() {
        String::default()
    } else {
        unsafe { &*text_node }.data().clone()
    }
}

// cpp: layoutng_inline/layout_text.cc:134-138
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextOriginalTextLength(this: &LayoutText) -> u32 {
    this.CheckIsNotDestroyed();
    debug_assert!(!this.IsBR());
    this.OriginalText().length()
}

// cpp: layoutng_inline/layout_text.cc:140-152
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextPlainText(this: &LayoutText) -> String {
    this.CheckIsNotDestroyed();
    if this.IsInLayoutNGInlineFormattingContext() {
        let mut result = String::from("");
        let mut cursor = InlineCursor::default();
        cursor.MoveToLayoutObject(this);
        while cursor.IsNotNull() {
            result.push_string(&cursor.Current().Text(&cursor).ToString());
            cursor.MoveToNextForSameLayoutObject();
        }
        if !result.empty() {
            return result;
        }
    }
    this.text_.clone()
}

// cpp: layoutng_inline/layout_text.cc:154-163
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextIsAllCollapsibleWhitespace(this: &LayoutText) -> bool {
    this.CheckIsNotDestroyed();
    if this.text_.empty() {
        return true;
    }
    let style = this.StyleRef();
    this.text_
        .Span16()
        .unwrap_or_default()
        .iter()
        .all(|character| style.IsCollapsibleWhiteSpace(*character))
}

// cpp: layoutng_inline/layout_text.cc:165-173
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextFirstCharacterAfterWhitespaceCollapsing(this: &LayoutText) -> i32 {
    this.CheckIsNotDestroyed();
    let mut cursor = InlineCursor::default();
    cursor.MoveToLayoutObject(this);
    if cursor.IsNull() {
        return 0;
    }
    let text = cursor.Current().Text(&cursor);
    if text.IsEmpty() {
        0
    } else {
        text.CodePointAt(0)
    }
}

// cpp: layoutng_inline/layout_text.cc:175-183
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextLastCharacterAfterWhitespaceCollapsing(this: &LayoutText) -> i32 {
    this.CheckIsNotDestroyed();
    let mut cursor = InlineCursor::default();
    cursor.MoveToLayoutObject(this);
    if cursor.IsNull() {
        return 0;
    }
    let text = cursor.Current().Text(&cursor);
    if text.IsEmpty() {
        0
    } else {
        text.CodePointAt(text.length() - 1)
    }
}

// cpp: layoutng_inline/layout_text.cc:185-198
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextPreviousCharacter(this: &LayoutText) -> u16 {
    this.CheckIsNotDestroyed();
    let mut previous = this.PreviousInPreOrderDefault();
    while !previous.is_null()
        && (unsafe { &*previous }.IsLayoutInline()
            || (unsafe { &*previous }.IsText()
                && unsafe { &*To::<LayoutText>(previous) }.HasEmptyText()))
    {
        previous = unsafe { &*previous }.PreviousInPreOrderDefault();
    }
    if !previous.is_null() && unsafe { &*previous }.IsText() {
        let value = unsafe { &*To::<LayoutText>(previous) }.TransformedText();
        if let Some(&last) = value.Span16().unwrap_or_default().last() {
            return last;
        }
    }
    0x0020
}

// cpp: layoutng_inline/layout_text.cc:200-206
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextSetTextInternal(this: &mut LayoutText, text: String) {
    this.CheckIsNotDestroyed();
    debug_assert!(!text.IsNull());
    this.text_ = text;
    debug_assert!(!this.IsBR() || this.text_.Span16() == Some(&[0x000a][..]));
}

// cpp: layoutng_inline/layout_text.cc:208-212
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextSetTextIfNeeded(this: &mut LayoutText, text: String) {
    this.CheckIsNotDestroyed();
    if this.text_ != text {
        this.ForceSetText(text);
    }
}

// cpp: layoutng_inline/layout_text.cc:214-218
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextForceSetText(this: &mut LayoutText, text: String) {
    this.CheckIsNotDestroyed();
    this.SetTextInternal(text);
    this.TextDidChange();
}

// cpp: layoutng_inline/layout_text.cc:220-224
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextTransformAndSecureOriginalText(this: &mut LayoutText) {
    this.CheckIsNotDestroyed();
    let original = this.OriginalText();
    if !original.IsNull() {
        this.ForceSetText(original);
    }
}

// cpp: layoutng_inline/layout_text.cc:226-252
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextTransformAndSecureText(
    this: &LayoutText,
    original: &String,
    offset_map: &mut TextOffsetMap,
) -> String {
    this.CheckIsNotDestroyed();
    let transformed =
        this.StyleRef()
            .ApplyTextTransform(original, this.PreviousCharacter(), Some(offset_map));
    let mask = match this.StyleRef().TextSecurity() {
        ETextSecurity::kNone => return transformed,
        ETextSecurity::kCircle => 0x25e6,
        ETextSecurity::kDisc => 0x2022,
        ETextSecurity::kSquare => 0x25a0,
    };
    let (secured, secure_map) = this.SecureText(&transformed, mask);
    if !secure_map.IsEmpty() {
        *offset_map = TextOffsetMap::new(
            original.length(),
            offset_map,
            transformed.length(),
            &secure_map,
            secured.length(),
        );
    }
    secured
}

// cpp: layoutng_inline/layout_text.cc:254-270
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextSecureText(
    this: &LayoutText,
    plain: &String,
    mask: u16,
) -> (String, TextOffsetMap) {
    this.CheckIsNotDestroyed();
    if plain.empty() {
        return (plain.clone(), TextOffsetMap::default());
    }
    let view = StringView::from(plain);
    let mut secured = Vec::with_capacity(plain.length() as usize);
    let mut offset_map = TextOffsetMap::default();
    let mut offset = 0;
    while offset < plain.length() {
        let cluster_size = LengthOfGraphemeCluster(&view, offset);
        offset += cluster_size;
        secured.push(mask);
        if cluster_size != 1 {
            offset_map.Append(offset, secured.len() as u32);
        }
    }
    (String::from_utf16(&secured), offset_map)
}

// cpp: layoutng_inline/layout_text.cc:272-283
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextSetVariableLengthTransformResult(
    this: &mut LayoutText,
    original_length: usize,
    offset_map: &TextOffsetMap,
) {
    this.CheckIsNotDestroyed();
    if offset_map.IsEmpty() {
        this.ClearHasVariableLengthTransform();
        return;
    }
    this.has_variable_length_transform_ = true;
    let result = VariableLengthTransformResult {
        original_length: u32::try_from(original_length).expect("text length exceeds wtf_size_t"),
        offset_map: offset_map.clone(),
    };
    unsafe { &mut *this.View() }.RegisterVariableLengthTransformResult(this, &result);
}

// cpp: layoutng_inline/layout_text.cc:285-289
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextGetVariableLengthTransformResult(
    this: &LayoutText,
) -> VariableLengthTransformResult {
    this.CheckIsNotDestroyed();
    unsafe { &*this.View() }.GetVariableLengthTransformResult(this)
}

// cpp: layoutng_inline/layout_text.cc:291-296
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextClearHasVariableLengthTransform(this: &mut LayoutText) {
    this.CheckIsNotDestroyed();
    if this.has_variable_length_transform_ {
        unsafe { &mut *this.View() }.UnregisterVariableLengthTransformResult(this);
    }
    this.has_variable_length_transform_ = false;
}

// cpp: layoutng_inline/layout_text.cc:298-308
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextSetNeedsLayoutAndIntrinsicWidthsRecalcAndFullPaintInvalidation(
    this: &mut LayoutText,
    reason: *const c_char,
) {
    this.CheckIsNotDestroyed();
    let parent = this.Parent();
    if !parent.is_null() && unsafe { &*parent }.IsLayoutTextCombine() {
        unsafe { &mut *parent.cast::<LayoutTextCombine>() }
            .SetNeedsLayoutAndIntrinsicWidthsRecalcAndFullPaintInvalidation(reason);
        return;
    }
    LayoutObject::SetNeedsLayoutAndIntrinsicWidthsRecalcAndFullPaintInvalidation(this, reason);
}

// cpp: layoutng_inline/layout_text.cc:310-317
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextTextDidChange(this: &mut LayoutText) {
    this.CheckIsNotDestroyed();
    if !this.Parent().is_null() {
        this.SetNeedsLayoutAndIntrinsicWidthsRecalcAndFullPaintInvalidation(unsafe {
            std::ptr::addr_of!(layout_invalidation_reason::kTextChanged)
        });
    }
    this.TextDidChangeWithoutInvalidation();
}

// cpp: layoutng_inline/layout_text.cc:319-328
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextTextDidChangeWithoutInvalidation(this: &mut LayoutText) {
    this.CheckIsNotDestroyed();
    let mut offset_map = TextOffsetMap::default();
    let original = if this.OriginalText().empty() {
        this.text_.clone()
    } else {
        this.OriginalText()
    };
    this.text_ = this.TransformAndSecureText(&original, &mut offset_map);
    this.SetVariableLengthTransformResult(original.length() as usize, &offset_map);
    this.valid_ng_items_ = false;
    this.ClearHasNoControlItems();
    this.SetNeedsCollectInlines();
}

// cpp: layoutng_inline/layout_text.cc:330-338
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextInvalidateSubtreeLayoutForFontUpdates(this: &mut LayoutText) {
    this.CheckIsNotDestroyed();
    if this.IsFontFallbackValid() {
        return;
    }
    this.valid_ng_items_ = false;
    this.SetNeedsCollectInlines();
    this.SetNeedsLayoutAndIntrinsicWidthsRecalcAndFullPaintInvalidation(unsafe {
        std::ptr::addr_of!(layout_invalidation_reason::kFontsChanged)
    });
}

// cpp: layoutng_inline/layout_text.cc:340-350
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextSetInlineItems(
    this: &mut LayoutText,
    data: *mut InlineItemsData,
    begin: usize,
    size: usize,
) {
    this.CheckIsNotDestroyed();
    let data = unsafe { &*data };
    #[cfg(debug_assertions)]
    for i in begin..begin + size {
        debug_assert_eq!(
            unsafe { &*data.items[i].Get() }.GetLayoutObject() as *const _,
            this as *const LayoutText as *const _
        );
    }
    this.valid_ng_items_ = true;
    this.inline_items_.SetItems(
        data,
        u32::try_from(begin).expect("inline item index exceeds wtf_size_t"),
        u32::try_from(size).expect("inline item count exceeds wtf_size_t"),
    );
}

// cpp: layoutng_inline/layout_text.cc:352-357
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextClearInlineItems(this: &mut LayoutText) {
    this.CheckIsNotDestroyed();
    this.has_bidi_control_items_ = false;
    this.valid_ng_items_ = false;
    this.inline_items_.Clear();
}

// cpp: layoutng_inline/layout_text.cc:359-364
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextInlineItems(this: &LayoutText) -> *const InlineItemSpan {
    this.CheckIsNotDestroyed();
    debug_assert!(this.valid_ng_items_);
    debug_assert!(!this.inline_items_.empty());
    &this.inline_items_
}

// cpp: layoutng_inline/layout_text.cc:366-370
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextSetPreviousLogicalStartingPoint(
    this: &LayoutText,
    point: &LogicalOffset,
) {
    this.CheckIsNotDestroyed();
    this.previous_logical_starting_point_.set(*point);
}

// cpp: layoutng_inline/layout_text.cc:372-378
#[cfg(debug_assertions)]
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextRecalcVisualOverflow(this: &mut LayoutText) {
    if this.IsInline() && this.IsInLayoutNGInlineFormattingContext() {
        panic!("LayoutText::RecalcVisualOverflow is unreachable in inline formatting context");
    }
    LayoutObject::RecalcVisualOverflow(this);
}
