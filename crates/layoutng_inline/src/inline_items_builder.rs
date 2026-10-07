// C++: layoutng_inline/inline_items_builder.h/.cc. The two source template
// instantiations share this Rust generic and retain their mapping modes.
#![allow(non_snake_case)]

use font_engine::text::native::character::Character;
use font_engine::FontHeight;
use foundation::{
    kNotFound, EOrder, ETextTransform, HeapVector, IsA, MakeGarbageCollected, Member, String,
    StringBuilder, StringView, TextDirection, TextEmphasisMark, To, UChar, UnicodeBidi, Visitor,
};
use layoutng::internal::form_node_metadata::HTMLAreaElement;
use layoutng::internal::inline_item::{CollapseType, InlineItem, InlineItemType, InlineItems};
use layoutng::internal::inline_node_data::InlineNodeData;
use layoutng::internal::layout_block_flow::LayoutBlockFlow;
use layoutng::internal::layout_inline::LayoutInline;
use layoutng::internal::layout_object::LayoutObject;
use layoutng::internal::layout_text::LayoutText;
use layoutng::internal::svg_inline_node_data::SvgTextChunkOffsets;
use layoutng::internal::text_item_type::TextItemType;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::computed_style_constants::EVerticalAlign;

use crate::empty_offset_mapping_builder::{
    EmptyOffsetMappingBuilder, SourceNodeScope as EmptySourceNodeScope,
};
use crate::offset_mapping_builder::{
    OffsetMappingBuilder, SourceNodeScope as OffsetSourceNodeScope,
};
use crate::transformed_string::TransformedString;

// cpp: layoutng_inline/inline_items_builder.h:149-159
pub struct BidiContext {
    pub node: Member<LayoutObject>,
    pub enter: UChar,
    pub exit: UChar,
}

impl BidiContext {
    // cpp: layoutng_inline/inline_items_builder.cc:1834-1839
    pub fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.node);
    }
}

// cpp: layoutng_inline/inline_items_builder.h:161-175
pub struct BoxInfo {
    pub style: Member<ComputedStyle>,
    pub item_index: u32,
    pub should_create_box_fragment: bool,
    pub text_metrics: FontHeight,
}

#[allow(non_snake_case)]
impl BoxInfo {
    // cpp: layoutng_inline/inline_items_builder.cc:227-237
    pub fn new(item_index: u32, item: &InlineItem) -> Self {
        let style = item.Style();
        debug_assert!(!style.is_null());
        Self {
            style: Member::from_ptr(style as *mut ComputedStyle),
            item_index,
            should_create_box_fragment: item.ShouldCreateBoxFragment(),
            text_metrics: unsafe { &*style }.GetFontHeight(unsafe { &*style }.GetFontBaseline()),
        }
    }

    // cpp: layoutng_inline/inline_items_builder.h:170-170
    pub fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.style);
    }

    // cpp: layoutng_inline/inline_items_builder.cc:239-261
    pub fn ShouldCreateBoxFragmentForChild(&self, child: &BoxInfo) -> bool {
        let child_style = unsafe { &*child.style.Get() };
        if child_style.MayHaveMargin() {
            return true;
        }
        if child_style.VerticalAlign() != EVerticalAlign::kBaseline {
            return true;
        }
        if self.text_metrics != child.text_metrics {
            return true;
        }
        false
    }

    // cpp: layoutng_inline/inline_items_builder.cc:263-270
    pub fn SetShouldCreateBoxFragment(&mut self, items: &mut InlineItems) {
        debug_assert!(!self.should_create_box_fragment);
        self.should_create_box_fragment = true;
        unsafe { &mut *items[self.item_index as usize].Get() }.SetShouldCreateBoxFragment();
    }
}

// cpp: layoutng_inline/inline_items_builder.cc:151-160
fn AppendItem(
    items: *mut InlineItems,
    type_: InlineItemType,
    start: u32,
    end: u32,
    layout_object: *mut LayoutObject,
) -> *mut InlineItem {
    let item = MakeGarbageCollected(InlineItem::new(type_, start, end, layout_object));
    unsafe { &mut *items }.push_back(Member::from_ptr(item));
    item
}

// cpp: layoutng_inline/inline_items_builder.cc:214-216
fn IsNonOrc16BitCharacter(character: UChar) -> bool {
    character >= 0x100 && character != 0xfffc
}

// Keep code units intact, including a surrogate that is appended before its
// partner. The generic Display append path is only for host-facing text.
fn AppendCodeUnit(text: &mut StringBuilder, character: UChar) {
    text.AppendCodeUnit(character);
}

// cpp: layoutng_inline/inline_items_builder.cc:202-212
fn LastItemToCollapseWith(items: *mut InlineItems) -> *mut InlineItem {
    for item in unsafe { &*items }.iter().rev() {
        let item = item.Get();
        if unsafe { &*item }.EndCollapseType() != CollapseType::kOpaqueToCollapsing {
            return item;
        }
    }
    std::ptr::null_mut()
}

// cpp: layoutng_inline/inline_items_builder.cc:218-224
fn GetCollapsedSpaceChar(style: *const ComputedStyle) -> UChar {
    if !style.is_null()
        && (unsafe { &*style }.TextTransform() & ETextTransform::kFullWidth)
            != ETextTransform::kNone
    {
        0x3000
    } else {
        0x20
    }
}

// cpp: layoutng_inline/inline_items_builder.cc:55-60
const DISABLE_FORCED_BREAK_IN_RUBY_COLUMN: bool = true;

// The source's East Asian width branch is compile-time disabled. Its active
// slow path only removes a newline adjacent to U+200B. That character always
// requires 16-bit storage, so the outer Is8Bit guard cannot change the result.
// cpp: layoutng_inline/inline_items_builder.cc:89-148
fn ShouldRemoveNewline(
    before: &StringBuilder,
    space_index: u32,
    _before_style: *const ComputedStyle,
    after: &StringView,
    _after_style: *const ComputedStyle,
) -> bool {
    let before_text = before.ToString();
    let before_units = before_text.Span16().unwrap_or_default();
    debug_assert!(
        space_index as usize == before_units.len()
            || (space_index as usize) < before_units.len()
                && before_units[space_index as usize] == 0x20
    );
    if space_index != 0 && before_units[space_index as usize - 1] == 0x200b {
        return true;
    }
    if !after.IsEmpty() && after[0] == 0x200b {
        return true;
    }
    false
}

// cpp: layoutng_inline/inline_items_builder.cc:163-171
fn ShouldIgnore(c: UChar) -> bool {
    c == 0x0d || c == 0x0c
}

// cpp: layoutng_inline/inline_items_builder.cc:173-185
fn IsControlItemCharacter(c: UChar) -> bool {
    c == 0x0a || c == 0x09 || c == 0x200c || ShouldIgnore(c)
}

// cpp: layoutng_inline/inline_items_builder.cc:1062-1062,1129-1129
fn FindControlItemCharacter(string: &StringView, start: u32) -> u32 {
    (start..string.length())
        .find(|&index| IsControlItemCharacter(string[index as usize]))
        .unwrap_or(kNotFound)
}

// The source feature snapshot defines both switches as constexpr true. Keep
// their foundation ownership explicit until that crate exports these methods.
unsafe extern "Rust" {
    // cpp: foundation/layout_features/runtime_enabled_features.h:21-21,40-40
    fn OffsetMappingReuseFullWidthSpaceFixEnabled() -> bool;
    fn CollapseZeroWidthSpaceWhenReuseItemEnabled() -> bool;
    // cpp: foundation/layout_features/runtime_enabled_features.h:16-16
    fn CSSLineClampLineBreakingEllipsisEnabled() -> bool;
    // cpp: foundation/blink_base/wtf/text/string_builder.h:94-111,159-169
    fn StringBuilderCapacity(builder: &StringBuilder) -> u32;
    fn StringBuilderReserveCapacity(builder: &mut StringBuilder, capacity: u32);
    fn StringBuilderReserve16BitCapacity(builder: &mut StringBuilder, capacity: u32);
    // cpp: foundation/blink_base/wtf/text/string_view.h:258-260
    fn StringViewIs8Bit(view: &StringView) -> bool;
}

// cpp: layoutng_inline/inline_items_builder.cc:190-201
fn MoveToEndOfCollapsibleSpaces(string: &StringView, offset: &mut u32, c: &mut UChar) -> bool {
    debug_assert_eq!(*c, string[*offset as usize]);
    debug_assert!(Character::IsCollapsibleSpace(*c));
    let mut space_run_has_newline = *c == 0x0a;
    *offset += 1;
    while *offset < string.length() {
        *c = string[*offset as usize];
        space_run_has_newline |= *c == 0x0a;
        if !Character::IsCollapsibleSpace(*c) {
            break;
        }
        *offset += 1;
    }
    space_run_has_newline
}

// cpp: layoutng_inline/inline_items_builder.h:41-47,178-208
// C++ has two explicit template instantiations. The Rust generic keeps their
// state types distinct and delegates offset mapping to its owning module.
pub struct InlineItemsBuilderTemplate<MappingBuilder> {
    block_flow_: *mut LayoutBlockFlow,
    items_: *mut InlineItems,
    text_: StringBuilder,
    mapping_builder_: MappingBuilder,
    boxes_: HeapVector<BoxInfo>,
    bidi_context_: HeapVector<BidiContext>,
    text_chunk_offsets_: *const SvgTextChunkOffsets,
    ruby_text_nesting_level_: u32,
    is_text_combine_: bool,
    has_bidi_controls_: bool,
    has_floats_: bool,
    has_out_of_flow_positioned_: bool,
    has_initial_letter_box_: bool,
    has_ruby_: bool,
    has_text_emphasis_: bool,
    is_block_level_: bool,
    has_unicode_bidi_plain_text_: bool,
    is_bisect_line_break_disabled_: bool,
    is_score_line_break_disabled_: bool,
    has_non_orc_16bit_: bool,
}

// cpp: layoutng_inline/inline_items_builder.h:300-301
pub type InlineItemsBuilder = InlineItemsBuilderTemplate<EmptyOffsetMappingBuilder>;
// cpp: layoutng_inline/inline_items_builder.h:300-303
pub type InlineItemsBuilderForOffsetMapping = InlineItemsBuilderTemplate<OffsetMappingBuilder>;

// C++ explicitly instantiates this template for exactly two mapping builders.
// Each Rust mapping implementation supplies the same compile-time selection.
pub trait InlineItemsMappingMode: Default {
    type SourceNodeScope;
    const NEEDS_BOX_INFO: bool;
    const REUSE_REACHABLE: bool;
    fn EnterSourceNodeScope(&mut self, node: *const LayoutObject) -> Self::SourceNodeScope;
    fn AppendIdentityMapping(&mut self, length: u32);
    fn RevertIdentityMapping1(&mut self);
    fn AppendCollapsedMapping(&mut self, length: u32);
    fn AppendVariableMapping(&mut self, source_length: u32, target_length: u32);
    fn CollapseTrailingSpace(&mut self, space_length: u32);
    fn RestoreTrailingCollapsibleSpace(&mut self, text: &LayoutText, offset: u32);
}

impl InlineItemsMappingMode for EmptyOffsetMappingBuilder {
    type SourceNodeScope = EmptySourceNodeScope;
    const NEEDS_BOX_INFO: bool = true;
    const REUSE_REACHABLE: bool = true;
    fn EnterSourceNodeScope(&mut self, node: *const LayoutObject) -> Self::SourceNodeScope {
        EmptySourceNodeScope::new(self, node.cast())
    }
    fn AppendIdentityMapping(&mut self, length: u32) {
        EmptyOffsetMappingBuilder::AppendIdentityMapping(self, length);
    }
    fn RevertIdentityMapping1(&mut self) {
        EmptyOffsetMappingBuilder::RevertIdentityMapping1(self);
    }
    fn AppendCollapsedMapping(&mut self, length: u32) {
        EmptyOffsetMappingBuilder::AppendCollapsedMapping(self, length);
    }
    fn AppendVariableMapping(&mut self, source_length: u32, target_length: u32) {
        EmptyOffsetMappingBuilder::AppendVariableMapping(self, source_length, target_length);
    }
    fn CollapseTrailingSpace(&mut self, space_length: u32) {
        EmptyOffsetMappingBuilder::CollapseTrailingSpace(self, space_length);
    }
    fn RestoreTrailingCollapsibleSpace(&mut self, text: &LayoutText, offset: u32) {
        EmptyOffsetMappingBuilder::RestoreTrailingCollapsibleSpace(self, text, offset);
    }
}

// cpp: layoutng_inline/inline_items_builder.h:296-303
// cpp: layoutng_inline/inline_items_builder.cc:37-41,1813-1833
impl InlineItemsMappingMode for OffsetMappingBuilder {
    type SourceNodeScope = OffsetSourceNodeScope;
    const NEEDS_BOX_INFO: bool = false;
    const REUSE_REACHABLE: bool = false;
    fn EnterSourceNodeScope(&mut self, node: *const LayoutObject) -> Self::SourceNodeScope {
        OffsetSourceNodeScope::new(self, node)
    }
    fn AppendIdentityMapping(&mut self, length: u32) {
        OffsetMappingBuilder::AppendIdentityMapping(self, length);
    }
    fn RevertIdentityMapping1(&mut self) {
        OffsetMappingBuilder::RevertIdentityMapping1(self);
    }
    fn AppendCollapsedMapping(&mut self, length: u32) {
        OffsetMappingBuilder::AppendCollapsedMapping(self, length);
    }
    fn AppendVariableMapping(&mut self, source_length: u32, target_length: u32) {
        OffsetMappingBuilder::AppendVariableMapping(self, source_length, target_length);
    }
    fn CollapseTrailingSpace(&mut self, space_offset: u32) {
        OffsetMappingBuilder::CollapseTrailingSpace(self, space_offset);
    }
    fn RestoreTrailingCollapsibleSpace(&mut self, text: &LayoutText, offset: u32) {
        OffsetMappingBuilder::RestoreTrailingCollapsibleSpace(self, text, offset);
    }
}

#[allow(non_snake_case)]
impl<MappingBuilder: InlineItemsMappingMode> InlineItemsBuilderTemplate<MappingBuilder> {
    // cpp: layoutng_inline/inline_items_builder.h:50-55
    // cpp: layoutng_inline/inline_items_builder.cc:26-35
    pub fn new(
        block_flow: *mut LayoutBlockFlow,
        items: *mut InlineItems,
        _previous_text_content: &String,
        chunk_offsets: *const SvgTextChunkOffsets,
    ) -> Self {
        Self {
            block_flow_: block_flow,
            items_: items,
            text_: StringBuilder::new(),
            mapping_builder_: MappingBuilder::default(),
            boxes_: HeapVector::default(),
            bidi_context_: HeapVector::default(),
            text_chunk_offsets_: chunk_offsets,
            ruby_text_nesting_level_: 0,
            is_text_combine_: unsafe { &*block_flow }.IsLayoutTextCombine(),
            has_bidi_controls_: false,
            has_floats_: false,
            has_out_of_flow_positioned_: false,
            has_initial_letter_box_: false,
            has_ruby_: false,
            has_text_emphasis_: false,
            is_block_level_: true,
            has_unicode_bidi_plain_text_: false,
            is_bisect_line_break_disabled_: false,
            is_score_line_break_disabled_: false,
            has_non_orc_16bit_: false,
        }
    }

    // cpp: layoutng_inline/inline_items_builder.h:57-57
    pub fn GetLayoutBlockFlow(&self) -> *mut LayoutBlockFlow {
        self.block_flow_
    }

    // cpp: layoutng_inline/inline_items_builder.cc:49-52
    pub fn ToString(&self) -> String {
        self.text_.ToString()
    }

    // cpp: layoutng_inline/inline_items_builder.h:62-69
    pub fn HasBidiControls(&self) -> bool {
        self.has_bidi_controls_
    }
    pub fn IsBlockLevel(&self) -> bool {
        self.is_block_level_
    }
    pub fn HasUnicodeBidiPlainText(&self) -> bool {
        self.has_unicode_bidi_plain_text_
    }

    // cpp: layoutng_inline/inline_items_builder.h:138-141
    pub fn GetOffsetMappingBuilder(&mut self) -> &mut MappingBuilder {
        &mut self.mapping_builder_
    }
    pub fn ShouldAbort(&self) -> bool {
        false
    }

    // cpp: layoutng_inline/inline_items_builder.cc:37-41
    pub fn NeedsBoxInfo() -> bool {
        MappingBuilder::NEEDS_BOX_INFO
    }

    // cpp: layoutng_inline/inline_items_builder.cc:300-310
    fn AppendEmptyTextItem(&mut self, layout_object: *mut LayoutText) {
        debug_assert!(!layout_object.is_null());
        let offset = self.text_.ToString().length();
        let item = AppendItem(
            self.items_,
            InlineItemType::kText,
            offset,
            offset,
            layout_object.cast(),
        );
        let item = unsafe { &mut *item };
        item.SetEndCollapseType(CollapseType::kOpaqueToCollapsing);
        item.SetIsEmptyItem(true);
        item.SetIsBlockLevel(true);
    }

    // cpp: layoutng_inline/inline_items_builder.cc:326-331
    fn DidAppendForcedBreak(&mut self) {
        self.is_bisect_line_break_disabled_ = true;
    }

    // cpp: layoutng_inline/inline_items_builder.cc:333-340
    fn DidAppendTextReusing(&mut self, item: &InlineItem) {
        self.is_block_level_ &= item.IsBlockLevel();
        if item.IsForcedLineBreak() {
            self.DidAppendForcedBreak();
        }
    }

    // cpp: layoutng_inline/inline_items_builder.cc:343-582
    pub fn AppendTextReusing(
        &mut self,
        original_data: &InlineNodeData,
        layout_text: *mut LayoutText,
    ) -> bool {
        // The OffsetMappingBuilder specialization is source NOTREACHED().
        assert!(MappingBuilder::REUSE_REACHABLE);
        debug_assert!(!layout_text.is_null());
        let layout_text_ref = unsafe { &*layout_text };
        let items = layout_text_ref.GetInlineItems().Items();
        let old_item0 = unsafe { &*items[0].Get() };
        if old_item0.Length() == 0 {
            return false;
        }

        if (old_item0.Index() as usize) < original_data.items.len() {
            for i in (1..=old_item0.Index() as usize).rev() {
                let prev_item = unsafe { &*original_data.items[i - 1].Get() };
                if prev_item.Length() > 0 || prev_item.Type() == InlineItemType::kControl {
                    break;
                }
                if prev_item.Type() == InlineItemType::kBlockInInline {
                    return false;
                }
            }
        }

        let original_string = &original_data.text_content;
        let original_units = original_string.Span16().unwrap_or_default();
        let new_style = layout_text_ref.StyleRef();
        let collapse_spaces = new_style.ShouldCollapseWhiteSpaces();
        let mut preserve_newlines = new_style.ShouldPreserveBreaks();
        if preserve_newlines && self.is_text_combine_ {
            preserve_newlines = false;
        }

        let last_item = LastItemToCollapseWith(self.items_);
        if !last_item.is_null() {
            let last = unsafe { &*last_item };
            if collapse_spaces {
                match last.EndCollapseType() {
                    CollapseType::kCollapsible => {
                        match original_units[old_item0.StartOffset() as usize] {
                            0x20 => return false,
                            0x3000 => {
                                if unsafe { OffsetMappingReuseFullWidthSpaceFixEnabled() }
                                    && GetCollapsedSpaceChar(new_style) == 0x3000
                                {
                                    return false;
                                }
                            }
                            0x0a => {
                                if preserve_newlines {
                                    return false;
                                }
                            }
                            0x200b => {
                                if unsafe { CollapseZeroWidthSpaceWhenReuseItemEnabled() }
                                    && old_item0.TextType() == TextItemType::kFlowControl
                                {
                                    return false;
                                }
                            }
                            _ => {}
                        }
                        if last.IsEndCollapsibleNewline() {
                            let view = StringView::from(original_string)
                                .Substring(old_item0.StartOffset(), old_item0.Length());
                            if ShouldRemoveNewline(
                                &self.text_,
                                last.EndOffset() - 1,
                                last.Style(),
                                &view,
                                new_style,
                            ) {
                                return false;
                            }
                        }
                    }
                    CollapseType::kNotCollapsible => {
                        let source_text = layout_text_ref.TransformedText();
                        let source_units = source_text.Span16().unwrap_or_default();
                        if !source_units.is_empty()
                            && Character::IsCollapsibleSpace(source_units[0])
                        {
                            if original_units[old_item0.StartOffset() as usize] != 0x20 {
                                return false;
                            }
                            let mut offset = 0u32;
                            let mut c = source_units[0];
                            let source_view = StringView::from(source_text);
                            let contains_newline =
                                MoveToEndOfCollapsibleSpaces(&source_view, &mut offset, &mut c);
                            if contains_newline
                                && ShouldRemoveNewline(
                                    &self.text_,
                                    self.text_.ToString().length(),
                                    last.Style(),
                                    &source_view.Substring(offset, source_view.length() - offset),
                                    new_style,
                                )
                            {
                                return false;
                            }
                        }
                    }
                    CollapseType::kCollapsed => {
                        self.RestoreTrailingCollapsibleSpace(last_item);
                        return false;
                    }
                    CollapseType::kOpaqueToCollapsing => unreachable!(),
                }
            } else if last.EndCollapseType() == CollapseType::kCollapsed {
                self.RestoreTrailingCollapsibleSpace(last_item);
                return false;
            }
            debug_assert!(!last.Style().is_null());
            if !unsafe { &*last.Style() }.ShouldWrapLine() && new_style.ShouldWrapLine() {
                return false;
            }
        } else if collapse_spaces {
            let leading_char = original_units[old_item0.StartOffset() as usize];
            let full_width_fix_applies = unsafe { OffsetMappingReuseFullWidthSpaceFixEnabled() }
                && GetCollapsedSpaceChar(new_style) == 0x3000;
            if leading_char == 0x20 || (leading_char == 0x3000 && full_width_fix_applies) {
                return false;
            }
        }

        if preserve_newlines
            && (!self.bidi_context_.is_empty() || layout_text_ref.HasBidiControlInlineItems())
            && layout_text_ref
                .TransformedText()
                .Span16()
                .unwrap_or_default()
                .contains(&0x0a)
        {
            return false;
        }
        if old_item0.StartOffset() > 0
            && self.ShouldInsertBreakOpportunityAfterLeadingPreservedSpaces(
                &StringView::from(layout_text_ref.TransformedText()),
                new_style,
                0,
            )
        {
            return false;
        }

        for item_ptr in items {
            let item = unsafe { &*item_ptr.Get() };
            if self.text_.ToString().length() == 0 && item.Length() == 0 && collapse_spaces {
                continue;
            }
            if item.IsGeneratedForLineBreak() {
                let text = self.text_.ToString();
                let units = text.Span16().unwrap_or_default();
                if units.is_empty() {
                    continue;
                }
                let mut index = units.len() as isize - 1;
                while index >= 0 && units[index as usize] == 0x20 {
                    index -= 1;
                }
                if index >= 0 && units[index as usize] != 0x0a {
                    continue;
                }
            }

            let start = self.text_.ToString().length();
            self.has_non_orc_16bit_ |= original_data.HasNonOrc16BitCharacters();
            let start_old = item.StartOffset() as usize;
            let end_old = (item.StartOffset() + item.Length()) as usize;
            self.text_
                .AppendString(&String::from_utf16(&original_units[start_old..end_old]));
            if item.StartOffset() == start {
                let copied = MakeGarbageCollected(item.clone());
                unsafe { &mut *self.items_ }.push_back(Member::from_ptr(copied));
                self.DidAppendTextReusing(item);
                continue;
            }

            let end = start + item.Length();
            let mut adjusted_shape_result = std::ptr::null();
            if !item.TextShapeResult().is_null() {
                debug_assert_eq!(item.Type(), InlineItemType::kText);
                adjusted_shape_result =
                    unsafe { &*item.TextShapeResult() }.CopyAdjustedOffset(start);
                debug_assert!(!adjusted_shape_result.is_null());
            }
            let adjusted_item =
                MakeGarbageCollected(item.clone_adjusted(start, end, adjusted_shape_result));
            unsafe { &mut *self.items_ }.push_back(Member::from_ptr(adjusted_item));
            debug_assert_eq!(start, unsafe { &*adjusted_item }.StartOffset());
            debug_assert_eq!(end, unsafe { &*adjusted_item }.EndOffset());
            if !unsafe { &*adjusted_item }.TextShapeResult().is_null() {
                let shape = unsafe { &*unsafe { &*adjusted_item }.TextShapeResult() };
                debug_assert_eq!(start, shape.StartIndex());
                debug_assert_eq!(end, shape.EndIndex());
            }
            debug_assert_eq!(item.IsEmptyItem(), unsafe { &*adjusted_item }.IsEmptyItem());
            self.DidAppendTextReusing(unsafe { &*adjusted_item });
        }
        true
    }

    // cpp: layoutng_inline/inline_items_builder.cc:585-626
    pub fn AppendText(
        &mut self,
        layout_text: *mut LayoutText,
        previous_data: *const InlineNodeData,
    ) {
        debug_assert!(!layout_text.is_null());
        if !previous_data.is_null() && unsafe { &*layout_text }.HasValidInlineItems() {
            if self.AppendTextReusing(unsafe { &*previous_data }, layout_text) {
                return;
            }
        }
        if unsafe { &*layout_text }.IsWordBreak() {
            let _scope = self
                .mapping_builder_
                .EnterSourceNodeScope(layout_text.cast());
            if self.is_text_combine_ {
                self.Append(InlineItemType::kText, 0x200b, layout_text.cast());
                return;
            }
            self.AppendBreakOpportunity(layout_text.cast());
            return;
        }
        if !unsafe { &*layout_text }.HasVariableLengthTransform() {
            let transformed = TransformedString::new(StringView::from(
                unsafe { &*layout_text }.TransformedText(),
            ));
            self.AppendTextTransformed(&transformed, unsafe { &*layout_text });
            return;
        }
        let result = unsafe { &*layout_text }.GetVariableLengthTransformResult();
        let transformed = unsafe { &*layout_text }.TransformedText().clone();
        let length_map = result
            .offset_map
            .CreateLengthMap(result.original_length, transformed.length());
        assert!(transformed.length() as usize == length_map.len() || length_map.is_empty());
        let transformed =
            TransformedString::with_length_map(StringView::from(&transformed), &length_map);
        self.AppendTextTransformed(&transformed, unsafe { &*layout_text });
    }

    // cpp: layoutng_inline/inline_items_builder.cc:628-633
    pub fn AppendTextString(&mut self, string: &String, layout_object: *mut LayoutText) {
        let transformed = TransformedString::new(StringView::from(string));
        self.AppendTextTransformed(&transformed, unsafe { &*layout_object });
    }

    // cpp: layoutng_inline/inline_items_builder.cc:635-681
    fn AppendTextTransformed(
        &mut self,
        transformed: &TransformedString<'_>,
        layout_object: &LayoutText,
    ) {
        let string = transformed.View();
        let layout_object = layout_object as *const LayoutText as *mut LayoutText;
        if string.IsEmpty() {
            self.AppendEmptyTextItem(layout_object);
            return;
        }
        let capacity = unsafe { StringBuilderCapacity(&self.text_) };
        let estimated_length = self.text_.ToString().length() + string.length();
        if capacity != 0 && estimated_length > capacity {
            let new_capacity = estimated_length.max(capacity * 2);
            if unsafe { StringViewIs8Bit(string) } {
                unsafe { StringBuilderReserveCapacity(&mut self.text_, new_capacity) };
            } else {
                unsafe { StringBuilderReserve16BitCapacity(&mut self.text_, new_capacity) };
            }
        }
        let _scope = self
            .mapping_builder_
            .EnterSourceNodeScope(layout_object.cast());
        let style = unsafe { &*layout_object }.StyleRef();
        let should_not_preserve_newline = unsafe { &*layout_object }.IsSVGInlineText()
            || self.is_text_combine_
            || self.ruby_text_nesting_level_ > 0;
        self.RestoreTrailingCollapsibleSpaceIfRemoved();
        if !self.text_chunk_offsets_.is_null()
            && self.AppendTextChunks(transformed, unsafe { &*layout_object })
        {
            return;
        }
        if style.ShouldPreserveWhiteSpaces() {
            self.AppendPreserveWhitespace(transformed, style, layout_object);
        } else if style.ShouldPreserveBreaks() && !should_not_preserve_newline {
            self.AppendPreserveNewline(transformed, style, layout_object);
        } else {
            self.AppendCollapseWhitespace(transformed, style, layout_object);
        }
    }

    // cpp: layoutng_inline/inline_items_builder.cc:682-718
    fn AppendTextChunks(
        &mut self,
        transformed: &TransformedString<'_>,
        layout_text: &LayoutText,
    ) -> bool {
        let offsets = unsafe { &*self.text_chunk_offsets_ };
        let key = Member::from_ptr(layout_text as *const _ as *mut LayoutText);
        let Some(chunk_offsets) = offsets.get(&key) else {
            return false;
        };
        let style = layout_text.StyleRef();
        let should_collapse_space = style.ShouldCollapseWhiteSpaces();
        let length = transformed.View().length();
        let mut start = 0u32;
        let layout_text_ptr = layout_text as *const _ as *mut LayoutText;
        for &offset in chunk_offsets {
            debug_assert!(offset <= length);
            if start < offset {
                let part = transformed.Substring(start, offset - start);
                if !should_collapse_space {
                    self.AppendPreserveWhitespace(&part, style, layout_text_ptr);
                } else {
                    self.AppendCollapseWhitespace(&part, style, layout_text_ptr);
                }
            }
            self.ExitAndEnterSvgTextChunk(layout_text_ptr);
            start = offset;
        }
        if start >= length {
            return true;
        }
        let part = transformed.Substring(start, length - start);
        if !should_collapse_space {
            self.AppendPreserveWhitespace(&part, style, layout_text_ptr);
        } else {
            self.AppendCollapseWhitespace(&part, style, layout_text_ptr);
        }
        true
    }

    // cpp: layoutng_inline/inline_items_builder.cc:1220-1237
    fn ExitAndEnterSvgTextChunk(&mut self, layout_text: *mut LayoutText) {
        debug_assert!(unsafe { &*self.block_flow_ }.IsSVGText());
        debug_assert!(!self.text_chunk_offsets_.is_null());
        if self.bidi_context_.is_empty() {
            return;
        }
        let _scope = self.mapping_builder_.EnterSourceNodeScope(std::ptr::null());
        let exits: Vec<UChar> = self
            .bidi_context_
            .iter()
            .rev()
            .map(|bidi| bidi.exit)
            .collect();
        for exit in exits {
            self.AppendOpaqueCharacter(InlineItemType::kBidiControl, exit, layout_text.cast());
        }
        let enters: Vec<UChar> = self.bidi_context_.iter().map(|bidi| bidi.enter).collect();
        for enter in enters {
            self.AppendOpaqueCharacter(InlineItemType::kBidiControl, enter, layout_text.cast());
        }
    }

    // cpp: layoutng_inline/inline_items_builder.cc:1452-1459
    pub fn EnterBidiContext(&mut self, node: *mut LayoutObject, enter: UChar, exit: UChar) {
        self.AppendOpaqueCharacter(InlineItemType::kBidiControl, enter, std::ptr::null_mut());
        self.bidi_context_.push_back(BidiContext {
            node: Member::from_ptr(node),
            enter,
            exit,
        });
        self.has_bidi_controls_ = true;
    }

    // cpp: layoutng_inline/inline_items_builder.cc:1462-1470
    pub fn EnterBidiContextWithStyle(
        &mut self,
        node: *mut LayoutObject,
        style: &ComputedStyle,
        ltr_enter: UChar,
        rtl_enter: UChar,
        exit: UChar,
    ) {
        self.EnterBidiContext(
            node,
            if foundation::IsLtr(style.Direction()) {
                ltr_enter
            } else {
                rtl_enter
            },
            exit,
        );
    }

    // cpp: layoutng_inline/inline_items_builder.cc:1239-1247
    fn EnterSvgTextChunk(&mut self, style: &ComputedStyle) {
        if !unsafe { &*self.block_flow_ }.IsSVGText() || self.text_chunk_offsets_.is_null() {
            return;
        }
        self.EnterBidiContextWithStyle(std::ptr::null_mut(), style, 0x2066, 0x2067, 0x2069);
    }

    // cpp: layoutng_inline/inline_items_builder.cc:1473-1525
    pub fn EnterBlock(&mut self, style: &ComputedStyle) {
        if unsafe { CSSLineClampLineBreakingEllipsisEnabled() } && style.HasLineClamp() {
            self.is_score_line_break_disabled_ = true;
        }
        if style.RtlOrdering() == EOrder::kLogical {
            self.EnterSvgTextChunk(style);
            match style.GetUnicodeBidi() {
                UnicodeBidi::kNormal | UnicodeBidi::kEmbed | UnicodeBidi::kIsolate => {
                    if style.Direction() == TextDirection::kRtl {
                        self.has_bidi_controls_ = true;
                    }
                }
                UnicodeBidi::kBidiOverride | UnicodeBidi::kIsolateOverride => {
                    self.EnterBidiContextWithStyle(
                        std::ptr::null_mut(),
                        style,
                        0x202d,
                        0x202e,
                        0x202c,
                    );
                }
                UnicodeBidi::kPlaintext => {
                    self.has_bidi_controls_ = true;
                    self.has_unicode_bidi_plain_text_ = true;
                }
            }
        } else {
            debug_assert_eq!(style.RtlOrdering(), EOrder::kVisual);
            self.EnterBidiContextWithStyle(std::ptr::null_mut(), style, 0x202d, 0x202e, 0x202c);
        }
        if style.GetTextEmphasisMark() != TextEmphasisMark::kNone {
            self.has_text_emphasis_ = true;
        }
        if style.IsDisplayListItem() && !style.ListStyleType().Get().is_null() {
            self.is_block_level_ = false;
        }
    }

    // cpp: layoutng_inline/inline_items_builder.cc:1528-1617
    pub fn EnterInline(&mut self, node: *mut LayoutInline) {
        debug_assert!(!node.is_null());
        let node_ref = unsafe { &*node };
        let style = node_ref.StyleRef();
        if style.RtlOrdering() == EOrder::kLogical {
            match style.GetUnicodeBidi() {
                UnicodeBidi::kNormal => {}
                UnicodeBidi::kEmbed => {
                    self.EnterBidiContextWithStyle(node.cast(), style, 0x202a, 0x202b, 0x202c)
                }
                UnicodeBidi::kBidiOverride => {
                    self.EnterBidiContextWithStyle(node.cast(), style, 0x202d, 0x202e, 0x202c)
                }
                UnicodeBidi::kIsolate => {
                    self.EnterBidiContextWithStyle(node.cast(), style, 0x2066, 0x2067, 0x2069)
                }
                UnicodeBidi::kPlaintext => {
                    self.has_unicode_bidi_plain_text_ = true;
                    self.EnterBidiContext(node.cast(), 0x2068, 0x2069);
                }
                UnicodeBidi::kIsolateOverride => {
                    self.EnterBidiContext(node.cast(), 0x2068, 0x2069);
                    self.EnterBidiContextWithStyle(node.cast(), style, 0x202d, 0x202e, 0x202c);
                }
            }
        }

        if style.GetTextEmphasisMark() != TextEmphasisMark::kNone {
            self.has_text_emphasis_ = true;
        }
        self.has_ruby_ |= node_ref.IsInlineRubyText();
        if node_ref.IsInlineRubyText() {
            self.ruby_text_nesting_level_ += 1;
            let _scope = self.mapping_builder_.EnterSourceNodeScope(std::ptr::null());
            let parent = node_ref.Parent();
            if parent.is_null() || !unsafe { &*parent }.IsInlineRuby() {
                self.AppendOpaqueCharacter(
                    InlineItemType::kOpenRubyColumn,
                    if foundation::IsLtr(style.Direction()) {
                        0x2066
                    } else {
                        0x2067
                    },
                    std::ptr::null_mut(),
                );
                self.AppendOpaqueItem(InlineItemType::kRubyLinePlaceholder, std::ptr::null_mut());
                self.is_score_line_break_disabled_ = true;
            } else {
                self.AppendOpaqueItem(InlineItemType::kRubyLinePlaceholder, parent);
            }
        }
        self.AppendOpaqueItem(InlineItemType::kOpenTag, node.cast());

        if Self::NeedsBoxInfo() {
            let items = unsafe { &mut *self.items_ };
            let item_index = items.len() - 1;
            let current_box = BoxInfo::new(item_index as u32, unsafe { &*items[item_index].Get() });
            self.boxes_.push_back(current_box);
            if self.boxes_.len() > 1 {
                let current_index = self.boxes_.len() - 1;
                let (parents, current) = self.boxes_.split_at_mut(current_index);
                let parent_box = &mut parents[current_index - 1];
                let current_box = &current[0];
                if !parent_box.should_create_box_fragment
                    && parent_box.ShouldCreateBoxFragmentForChild(current_box)
                {
                    parent_box.SetShouldCreateBoxFragment(items);
                }
            }
        }

        let _scope = self.mapping_builder_.EnterSourceNodeScope(std::ptr::null());
        if node_ref.IsInlineRuby() {
            self.AppendOpaqueCharacter(
                InlineItemType::kOpenRubyColumn,
                if foundation::IsLtr(style.Direction()) {
                    0x2066
                } else {
                    0x2067
                },
                node.cast(),
            );
            if DISABLE_FORCED_BREAK_IN_RUBY_COLUMN {
                self.ruby_text_nesting_level_ += 1;
            }
            self.AppendOpaqueItem(InlineItemType::kRubyLinePlaceholder, node.cast());
            self.is_score_line_break_disabled_ = true;
        } else if node_ref.IsInlineRubyText() {
            self.AppendOpaqueItem(InlineItemType::kRubyLinePlaceholder, node.cast());
        }
    }

    // cpp: layoutng_inline/inline_items_builder.cc:1725-1730
    fn Exit(&mut self, node: *mut LayoutObject) {
        while self
            .bidi_context_
            .last()
            .is_some_and(|bidi| bidi.node.Get() == node)
        {
            let exit = self.bidi_context_.last().unwrap().exit;
            self.AppendOpaqueCharacter(InlineItemType::kBidiControl, exit, std::ptr::null_mut());
            self.bidi_context_.pop();
        }
    }

    // cpp: layoutng_inline/inline_items_builder.cc:1620-1627
    pub fn ExitBlock(&mut self) {
        self.Exit(std::ptr::null_mut());
        self.RemoveTrailingCollapsibleSpaceIfExists();
    }

    // cpp: layoutng_inline/inline_items_builder.cc:1630-1722
    pub fn ExitInline(&mut self, node: *mut LayoutObject) {
        debug_assert!(!node.is_null());
        let object = unsafe { &*node };
        if object.IsInlineRuby() {
            if DISABLE_FORCED_BREAK_IN_RUBY_COLUMN {
                self.ruby_text_nesting_level_ -= 1;
            }
            let _scope = self.mapping_builder_.EnterSourceNodeScope(std::ptr::null());
            let items = unsafe { &mut *self.items_ };
            let size = items.len();
            if size >= 3
                && unsafe { &*items[size - 3].Get() }.Type() == InlineItemType::kCloseRubyColumn
                && unsafe { &*items[size - 2].Get() }.Type() == InlineItemType::kOpenRubyColumn
                && unsafe { &*items[size - 1].Get() }.Type() == InlineItemType::kRubyLinePlaceholder
            {
                let end = unsafe { &*items[size - 2].Get() }.StartOffset();
                self.text_.Resize(end);
                items.Shrink((size - 2) as u32);
                self.mapping_builder_.RevertIdentityMapping1();
            } else {
                self.AppendOpaqueCharacter(InlineItemType::kCloseRubyColumn, 0x2069, node);
            }
        } else if object.IsInlineRubyText() {
            let _scope = self.mapping_builder_.EnterSourceNodeScope(std::ptr::null());
            self.AppendOpaqueItem(InlineItemType::kRubyLinePlaceholder, node);
        }

        if Self::NeedsBoxInfo() {
            let current_box = self.boxes_.last().expect("open inline box");
            if !current_box.should_create_box_fragment {
                let open_item_index = current_box.item_index as usize;
                let items = unsafe { &mut *self.items_ };
                debug_assert!(items.len() >= open_item_index + 1);
                debug_assert_eq!(
                    unsafe { &*items[open_item_index].Get() }.Type(),
                    InlineItemType::kOpenTag,
                );
                for i in (open_item_index..items.len()).rev() {
                    let item = unsafe { &mut *items[i].Get() };
                    if i == open_item_index {
                        debug_assert_eq!(i, current_box.item_index as usize);
                        let node = unsafe { &*item.GetLayoutObject() }.GetNode();
                        if !IsA::<HTMLAreaElement>(node) {
                            item.SetShouldCreateBoxFragment();
                        }
                        break;
                    }
                    debug_assert!(i > current_box.item_index as usize);
                    if item.IsEmptyItem() || item.IsCollapsibleSpaceOnly() {
                        continue;
                    }
                    break;
                }
            }
            self.boxes_.pop();
        }

        self.AppendOpaqueItem(InlineItemType::kCloseTag, node);
        if object.IsInlineRubyText() {
            self.ruby_text_nesting_level_ -= 1;
            let _scope = self.mapping_builder_.EnterSourceNodeScope(std::ptr::null());
            let parent = object.Parent();
            if !parent.is_null() && unsafe { &*parent }.IsInlineRuby() {
                self.AppendOpaqueCharacter(InlineItemType::kCloseRubyColumn, 0x2069, parent);
                self.AppendOpaqueCharacter(
                    InlineItemType::kOpenRubyColumn,
                    if foundation::IsLtr(unsafe { &*parent }.StyleRef().Direction()) {
                        0x2066
                    } else {
                        0x2067
                    },
                    parent,
                );
                self.AppendOpaqueItem(InlineItemType::kRubyLinePlaceholder, node);
                self.is_score_line_break_disabled_ = true;
            } else {
                self.AppendOpaqueCharacter(
                    InlineItemType::kCloseRubyColumn,
                    0x2069,
                    std::ptr::null_mut(),
                );
            }
        }
        self.Exit(node);
    }

    // cpp: layoutng_inline/inline_items_builder.cc:719-780
    fn AppendTransformedString(
        &mut self,
        transformed: &TransformedString<'_>,
        _layout_text: &LayoutText,
    ) {
        let view = transformed.View();
        // StringView currently exposes UTF-16 units but not the source
        // StringImpl storage flag. Non-Latin1 units are the observable part
        // needed by the downstream bidi decision.
        self.has_non_orc_16bit_ |= view.Span16().iter().any(|&ch| ch > 0xff);
        self.text_.AppendString(&view.ToString());
        if !transformed.HasLengthMap() {
            self.mapping_builder_.AppendIdentityMapping(view.length());
            return;
        }

        let mut identity_start = kNotFound;
        let size = view.length();
        let mut i = 0u32;
        while i < size {
            let len = transformed.LengthMap()[i as usize];
            if len > 1 {
                if identity_start != kNotFound {
                    self.mapping_builder_
                        .AppendIdentityMapping(i - identity_start);
                    identity_start = kNotFound;
                }
                let mut zero_length = 0u32;
                i += 1;
                while i < size {
                    if transformed.LengthMap()[i as usize] != 0 {
                        i -= 1;
                        break;
                    }
                    zero_length += 1;
                    i += 1;
                }
                self.mapping_builder_
                    .AppendVariableMapping(len, 1 + zero_length);
            } else if len == 0 {
                assert_ne!(i, 0);
                assert_ne!(identity_start, kNotFound);
                if i - identity_start > 1 {
                    self.mapping_builder_
                        .AppendIdentityMapping(i - identity_start - 1);
                }
                identity_start = kNotFound;
                let mut zero_length = 1u32;
                i += 1;
                while i < size {
                    if transformed.LengthMap()[i as usize] != 0 {
                        i -= 1;
                        break;
                    }
                    zero_length += 1;
                    i += 1;
                }
                self.mapping_builder_
                    .AppendVariableMapping(1, 1 + zero_length);
            } else {
                debug_assert_eq!(len, 1);
                if identity_start == kNotFound {
                    identity_start = i;
                }
            }
            i += 1;
        }
        if identity_start != kNotFound {
            self.mapping_builder_
                .AppendIdentityMapping(size - identity_start);
        }
    }

    // cpp: layoutng_inline/inline_items_builder.cc:783-983
    fn AppendCollapseWhitespace(
        &mut self,
        transformed: &TransformedString<'_>,
        style: *const ComputedStyle,
        layout_object: *mut LayoutText,
    ) {
        let string = transformed.View();
        debug_assert!(!string.IsEmpty());
        debug_assert!(!layout_object.is_null());
        let mut end_collapse = CollapseType::kNotCollapsible;
        let mut i = 0u32;
        let mut c = string[i as usize];
        let mut space_run_has_newline = false;
        let start_offset;

        if Character::IsCollapsibleSpace(c) {
            space_run_has_newline = MoveToEndOfCollapsibleSpaces(string, &mut i, &mut c);
            if space_run_has_newline && string.length() == 1 && unsafe { &*layout_object }.IsBR() {
                if self.is_text_combine_ || self.ruby_text_nesting_level_ > 0 {
                    let space = TransformedString::new(StringView::from(" "));
                    self.AppendTextItem(&space, layout_object);
                } else {
                    self.AppendForcedBreakCollapseWhitespace(layout_object.cast());
                }
                return;
            }

            let mut insert_space;
            let last_item = LastItemToCollapseWith(self.items_);
            if !last_item.is_null() {
                let item = unsafe { &*last_item };
                if item.EndCollapseType() == CollapseType::kNotCollapsible {
                    insert_space = true;
                } else {
                    debug_assert_eq!(item.EndCollapseType(), CollapseType::kCollapsible);
                    insert_space = false;
                    if (space_run_has_newline || item.IsEndCollapsibleNewline())
                        && item.Type() == InlineItemType::kText
                        && ShouldRemoveNewline(
                            &self.text_,
                            item.EndOffset() - 1,
                            item.Style(),
                            &string.Substring(i, string.length() - i),
                            style,
                        )
                    {
                        self.RemoveTrailingCollapsibleSpace(last_item);
                        space_run_has_newline = false;
                    } else if !unsafe { &*item.Style() }.ShouldWrapLine()
                        && unsafe { &*style }.ShouldWrapLine()
                    {
                        let is_forced_break = item.Type() == InlineItemType::kControl
                            && self.text_.ToString().Span16().unwrap_or_default()
                                [item.StartOffset() as usize]
                                == 0x0a;
                        if !is_forced_break {
                            self.AppendGeneratedBreakOpportunity(layout_object.cast());
                        }
                    }
                }
            } else {
                insert_space = false;
            }

            if space_run_has_newline
                && ShouldRemoveNewline(
                    &self.text_,
                    self.text_.ToString().length(),
                    style,
                    &string.Substring(i, string.length() - i),
                    style,
                )
            {
                insert_space = false;
                space_run_has_newline = false;
            }
            start_offset = self.text_.ToString().length();
            debug_assert!(i != 0);
            let mut collapsed_length = i;
            if insert_space {
                AppendCodeUnit(&mut self.text_, GetCollapsedSpaceChar(style));
                self.mapping_builder_.AppendIdentityMapping(1);
                collapsed_length -= 1;
            }
            if collapsed_length != 0 {
                self.mapping_builder_
                    .AppendCollapsedMapping(collapsed_length);
            }
            if i == string.length() {
                end_collapse = CollapseType::kCollapsible;
            }
        } else {
            let last_item = LastItemToCollapseWith(self.items_);
            if !last_item.is_null() {
                let item = unsafe { &*last_item };
                if item.EndCollapseType() == CollapseType::kCollapsible
                    && item.IsEndCollapsibleNewline()
                    && ShouldRemoveNewline(
                        &self.text_,
                        item.EndOffset() - 1,
                        item.Style(),
                        string,
                        style,
                    )
                {
                    self.RemoveTrailingCollapsibleSpace(last_item);
                }
            }
            start_offset = self.text_.ToString().length();
        }

        if i < string.length() {
            loop {
                debug_assert!(!Character::IsCollapsibleSpace(string[i as usize]));
                let start_of_non_space = i;
                i += 1;
                while i < string.length() {
                    c = string[i as usize];
                    if Character::IsCollapsibleSpace(c) {
                        break;
                    }
                    i += 1;
                }
                self.AppendTransformedString(
                    &transformed.Substring(start_of_non_space, i - start_of_non_space),
                    unsafe { &*layout_object },
                );
                if i == string.length() {
                    end_collapse = CollapseType::kNotCollapsible;
                    break;
                }
                debug_assert_eq!(c, string[i as usize]);
                debug_assert!(Character::IsCollapsibleSpace(c));
                let mut start_of_spaces = i;
                space_run_has_newline = MoveToEndOfCollapsibleSpaces(string, &mut i, &mut c);
                debug_assert!(start_of_spaces != 0);
                let remove_newline = space_run_has_newline
                    && ShouldRemoveNewline(
                        &self.text_,
                        self.text_.ToString().length(),
                        style,
                        &string.Substring(i, string.length() - i),
                        style,
                    );
                if remove_newline {
                    end_collapse = CollapseType::kNotCollapsible;
                    space_run_has_newline = false;
                } else {
                    AppendCodeUnit(&mut self.text_, GetCollapsedSpaceChar(style));
                    self.mapping_builder_.AppendIdentityMapping(1);
                    start_of_spaces += 1;
                    end_collapse = CollapseType::kCollapsible;
                }
                if i != start_of_spaces {
                    self.mapping_builder_
                        .AppendCollapsedMapping(i - start_of_spaces);
                }
                if i == string.length() {
                    break;
                }
            }
        }

        debug_assert!(self.text_.ToString().length() >= start_offset);
        if self.text_.ToString().length() == start_offset {
            self.AppendEmptyTextItem(layout_object);
            return;
        }
        let item = AppendItem(
            self.items_,
            InlineItemType::kText,
            start_offset,
            self.text_.ToString().length(),
            layout_object.cast(),
        );
        unsafe { &mut *item }.SetEndCollapseTypeWithNewline(end_collapse, space_run_has_newline);
        debug_assert!(!unsafe { &*item }.IsEmptyItem());
        self.is_block_level_ = false;
    }

    // cpp: layoutng_inline/inline_items_builder.cc:986-1010
    fn ShouldInsertBreakOpportunityAfterLeadingPreservedSpaces(
        &self,
        string: &StringView,
        style: &ComputedStyle,
        index: u32,
    ) -> bool {
        debug_assert!(index <= string.length());
        if self.is_text_combine_ {
            return false;
        }
        if style.ShouldCollapseWhiteSpaces()
            || !style.ShouldWrapLine()
            || string.IsEmpty()
            || index >= string.length()
            || string[index as usize] != 0x20
        {
            return false;
        }
        if index != 0 {
            return string[index as usize - 1] == 0x0a;
        }
        let text = self.text_.ToString();
        let units = text.Span16().unwrap_or_default();
        units.is_empty() || units[units.len() - 1] == 0x0a
    }

    // cpp: layoutng_inline/inline_items_builder.cc:1013-1032
    fn InsertBreakOpportunityAfterLeadingPreservedSpaces(
        &mut self,
        transformed: &TransformedString<'_>,
        style: &ComputedStyle,
        layout_object: *mut LayoutText,
        start: &mut u32,
    ) {
        let string = transformed.View();
        if self.ShouldInsertBreakOpportunityAfterLeadingPreservedSpaces(string, style, *start) {
            let mut end = *start;
            loop {
                end += 1;
                if end >= string.length() || string[end as usize] != 0x20 {
                    break;
                }
            }
            self.AppendTextItem(&transformed.Substring(*start, end - *start), layout_object);
            self.AppendGeneratedBreakOpportunity(layout_object.cast());
            *start = end;
        }
    }

    // cpp: layoutng_inline/inline_items_builder.cc:1039-1134
    fn AppendPreserveWhitespace(
        &mut self,
        transformed: &TransformedString<'_>,
        style: *const ComputedStyle,
        layout_object: *mut LayoutText,
    ) {
        debug_assert!(!style.is_null());
        let mut start = 0u32;
        self.InsertBreakOpportunityAfterLeadingPreservedSpaces(
            transformed,
            unsafe { &*style },
            layout_object,
            &mut start,
        );
        let transformed_view = transformed.View();
        let length = transformed_view.length();
        if start >= length {
            return;
        }
        if unsafe { &*layout_object }.HasNoControlItems() {
            self.AppendTextItem(&transformed.Substring(start, length - start), layout_object);
            return;
        }
        let mut control = FindControlItemCharacter(transformed_view, start);
        if control == kNotFound {
            unsafe { &mut *layout_object }.SetHasNoControlItems();
            self.AppendTextItem(&transformed.Substring(start, length - start), layout_object);
            return;
        }

        while start < length {
            if control != start {
                let end = control.min(length);
                self.AppendTextItem(&transformed.Substring(start, end - start), layout_object);
                if control >= length {
                    break;
                }
                start = control;
            }
            let c = transformed_view[start as usize];
            match c {
                0x0a => {
                    if self.is_text_combine_ || self.ruby_text_nesting_level_ > 0 {
                        start += 1;
                        self.AppendTextItem(
                            &TransformedString::new(StringView::from(" ")),
                            layout_object,
                        );
                    } else {
                        self.AppendForcedBreak(layout_object.cast());
                        start += 1;
                        self.InsertBreakOpportunityAfterLeadingPreservedSpaces(
                            transformed,
                            unsafe { &*style },
                            layout_object,
                            &mut start,
                        );
                    }
                }
                0x09 => {
                    let tab_end = ((start + 1)..length)
                        .find(|&index| transformed_view[index as usize] != 0x09)
                        .unwrap_or(length);
                    let item = self.AppendTextItemWithType(
                        InlineItemType::kControl,
                        &transformed.Substring(start, tab_end - start),
                        layout_object,
                    );
                    unsafe { &mut *item }.SetTextType(TextItemType::kFlowControl);
                    start = tab_end;
                    self.is_score_line_break_disabled_ = true;
                }
                0x200c => {
                    control = FindControlItemCharacter(transformed_view, start + 1);
                    if control == kNotFound {
                        control = length;
                    }
                    continue;
                }
                _ => {
                    debug_assert!(IsControlItemCharacter(c));
                    let item = self.Append(InlineItemType::kControl, c, layout_object.cast());
                    unsafe { &mut *item }.SetTextType(TextItemType::kFlowControl);
                    start += 1;
                }
            }
            if start >= length {
                break;
            }
            control = FindControlItemCharacter(transformed_view, start);
            if control == kNotFound {
                control = length;
            }
        }
    }

    // cpp: layoutng_inline/inline_items_builder.cc:1137-1158
    fn AppendPreserveNewline(
        &mut self,
        transformed: &TransformedString<'_>,
        style: *const ComputedStyle,
        layout_object: *mut LayoutText,
    ) {
        let string = transformed.View();
        let mut start = 0u32;
        while start < string.length() {
            if string[start as usize] == 0x0a {
                self.AppendForcedBreakCollapseWhitespace(layout_object.cast());
                start += 1;
                continue;
            }
            let end = ((start + 1)..string.length())
                .find(|&index| string[index as usize] == 0x0a)
                .unwrap_or(string.length());
            debug_assert!(end >= start);
            self.AppendCollapseWhitespace(
                &transformed.Substring(start, end - start),
                style,
                layout_object,
            );
            start = end;
        }
    }

    // cpp: layoutng_inline/inline_items_builder.cc:273-280
    fn AppendTextItem(
        &mut self,
        transformed: &TransformedString<'_>,
        layout_object: *mut LayoutText,
    ) {
        debug_assert!(!layout_object.is_null());
        self.AppendTextItemWithType(InlineItemType::kText, transformed, layout_object);
    }

    // cpp: layoutng_inline/inline_items_builder.cc:282-295
    fn AppendTextItemWithType(
        &mut self,
        type_: InlineItemType,
        transformed: &TransformedString<'_>,
        layout_object: *mut LayoutText,
    ) -> *mut InlineItem {
        debug_assert!(!layout_object.is_null());
        let start_offset = self.text_.ToString().length();
        self.AppendTransformedString(transformed, unsafe { &*layout_object });
        let item = AppendItem(
            self.items_,
            type_,
            start_offset,
            self.text_.ToString().length(),
            layout_object.cast(),
        );
        debug_assert!(!unsafe { &*item }.IsEmptyItem());
        self.is_block_level_ = false;
        item
    }

    // cpp: layoutng_inline/inline_items_builder.cc:1248-1264
    fn Append(
        &mut self,
        type_: InlineItemType,
        character: UChar,
        layout_object: *mut LayoutObject,
    ) -> *mut InlineItem {
        debug_assert_ne!(character, 0x20);
        self.has_non_orc_16bit_ |= IsNonOrc16BitCharacter(character);
        AppendCodeUnit(&mut self.text_, character);
        self.mapping_builder_.AppendIdentityMapping(1);
        let end_offset = self.text_.ToString().length();
        let item = AppendItem(
            self.items_,
            type_,
            end_offset - 1,
            end_offset,
            layout_object,
        );
        self.is_block_level_ &= unsafe { &*item }.IsBlockLevel();
        item
    }

    // cpp: layoutng_inline/inline_items_builder.cc:1336-1350
    pub fn AppendOpaqueCharacter(
        &mut self,
        type_: InlineItemType,
        character: UChar,
        layout_object: *mut LayoutObject,
    ) -> *mut InlineItem {
        self.has_non_orc_16bit_ |= IsNonOrc16BitCharacter(character);
        AppendCodeUnit(&mut self.text_, character);
        self.mapping_builder_.AppendIdentityMapping(1);
        let end_offset = self.text_.ToString().length();
        let item = AppendItem(
            self.items_,
            type_,
            end_offset - 1,
            end_offset,
            layout_object,
        );
        unsafe { &mut *item }.SetEndCollapseType(CollapseType::kOpaqueToCollapsing);
        self.is_block_level_ &= unsafe { &*item }.IsBlockLevel();
        item
    }

    // cpp: layoutng_inline/inline_items_builder.cc:1352-1361
    pub fn AppendOpaqueItem(&mut self, type_: InlineItemType, layout_object: *mut LayoutObject) {
        let end_offset = self.text_.ToString().length();
        let item = AppendItem(self.items_, type_, end_offset, end_offset, layout_object);
        unsafe { &mut *item }.SetEndCollapseType(CollapseType::kOpaqueToCollapsing);
        self.is_block_level_ &= unsafe { &*item }.IsBlockLevel();
    }

    // cpp: layoutng_inline/inline_items_builder.cc:1208-1217
    pub fn AppendBreakOpportunity(&mut self, layout_object: *mut LayoutObject) -> *mut InlineItem {
        debug_assert!(!layout_object.is_null());
        let item = self.AppendOpaqueCharacter(InlineItemType::kControl, 0x200b, layout_object);
        unsafe { &mut *item }.SetTextType(TextItemType::kFlowControl);
        item
    }

    // cpp: layoutng_inline/inline_items_builder.cc:312-324
    fn AppendGeneratedBreakOpportunity(&mut self, layout_object: *mut LayoutObject) {
        if unsafe { &*self.block_flow_ }.IsSVGText() {
            return;
        }
        debug_assert!(!layout_object.is_null());
        let _scope = self.mapping_builder_.EnterSourceNodeScope(std::ptr::null());
        let item = self.AppendBreakOpportunity(layout_object);
        unsafe { &mut *item }.SetIsGeneratedForLineBreak();
        unsafe { &mut *item }.SetEndCollapseType(CollapseType::kOpaqueToCollapsing);
    }

    // cpp: layoutng_inline/inline_items_builder.cc:1160-1197
    fn AppendForcedBreak(&mut self, layout_object: *mut LayoutObject) {
        debug_assert!(!layout_object.is_null());
        debug_assert!(!self.is_text_combine_);
        if !self.bidi_context_.is_empty() {
            let exits: Vec<UChar> = self
                .bidi_context_
                .iter()
                .rev()
                .map(|bidi| bidi.exit)
                .collect();
            let _scope = self.mapping_builder_.EnterSourceNodeScope(std::ptr::null());
            for exit in exits {
                self.AppendOpaqueCharacter(InlineItemType::kBidiControl, exit, layout_object);
            }
        }
        let item = self.Append(InlineItemType::kControl, 0x0a, layout_object);
        unsafe { &mut *item }.SetTextType(TextItemType::kForcedLineBreak);
        unsafe { &mut *item }.SetEndCollapseTypeWithNewline(CollapseType::kCollapsible, false);

        if !self.bidi_context_.is_empty() {
            let enters: Vec<UChar> = self.bidi_context_.iter().map(|bidi| bidi.enter).collect();
            let _scope = self.mapping_builder_.EnterSourceNodeScope(std::ptr::null());
            for enter in enters {
                self.AppendOpaqueCharacter(InlineItemType::kBidiControl, enter, layout_object);
            }
        }
        self.DidAppendForcedBreak();
    }

    // cpp: layoutng_inline/inline_items_builder.cc:1199-1205
    fn AppendForcedBreakCollapseWhitespace(&mut self, layout_object: *mut LayoutObject) {
        self.RemoveTrailingCollapsibleSpaceIfExists();
        self.AppendForcedBreak(layout_object);
    }

    // cpp: layoutng_inline/inline_items_builder.cc:1266-1285
    pub fn AppendAtomicInline(&mut self, layout_object: *mut LayoutObject) {
        debug_assert!(!layout_object.is_null());
        let _scope = self.mapping_builder_.EnterSourceNodeScope(layout_object);
        self.RestoreTrailingCollapsibleSpaceIfRemoved();
        self.Append(InlineItemType::kAtomicInline, 0xfffc, layout_object);
        if let Some(current_box) = self.boxes_.last_mut() {
            if !current_box.should_create_box_fragment {
                current_box.SetShouldCreateBoxFragment(unsafe { &mut *self.items_ });
            }
        }
    }

    // cpp: layoutng_inline/inline_items_builder.cc:1287-1311
    pub fn AppendBlockInInline(&mut self, layout_object: *mut LayoutObject) {
        debug_assert!(!layout_object.is_null());
        self.RemoveTrailingCollapsibleSpaceIfExists();
        let item = self.Append(InlineItemType::kBlockInInline, 0xfffc, layout_object);
        unsafe { &mut *item }.SetEndCollapseTypeWithNewline(CollapseType::kCollapsible, false);
        if self.ShouldUpdateLayoutObject() {
            let parent = unsafe { &*layout_object }.Parent();
            debug_assert!(parent.is_null() || IsA::<LayoutInline>(parent));
            if !parent.is_null() {
                let inline = To::<LayoutInline>(parent);
                unsafe { &mut *inline }.SetShouldCreateBoxFragment(true);
            }
        }
        self.is_bisect_line_break_disabled_ = true;
    }

    // cpp: layoutng_inline/inline_items_builder.cc:1314-1325
    pub fn AppendFloating(&mut self, layout_object: *mut LayoutObject) {
        self.AppendOpaqueItem(InlineItemType::kFloating, layout_object);
        self.has_floats_ = true;
        self.is_bisect_line_break_disabled_ = true;
    }

    // cpp: layoutng_inline/inline_items_builder.cc:1327-1334
    pub fn AppendOutOfFlowPositioned(&mut self, layout_object: *mut LayoutObject) {
        self.AppendOpaqueItem(InlineItemType::kOutOfFlowPositioned, layout_object);
        self.has_out_of_flow_positioned_ = true;
    }

    // cpp: layoutng_inline/inline_items_builder.cc:1363-1373
    fn RemoveTrailingCollapsibleSpaceIfExists(&mut self) {
        let item = LastItemToCollapseWith(self.items_);
        if !item.is_null() && unsafe { &*item }.EndCollapseType() == CollapseType::kCollapsible {
            self.RemoveTrailingCollapsibleSpace(item);
        }
    }

    // cpp: layoutng_inline/inline_items_builder.cc:1375-1410
    fn RemoveTrailingCollapsibleSpace(&mut self, item: *mut InlineItem) {
        debug_assert!(!item.is_null());
        let item_ref = unsafe { &*item };
        debug_assert_eq!(item_ref.EndCollapseType(), CollapseType::kCollapsible);
        debug_assert!(item_ref.Length() > 0);
        if item_ref.Type() != InlineItemType::kText {
            debug_assert!(matches!(
                item_ref.Type(),
                InlineItemType::kControl | InlineItemType::kBlockInInline
            ));
            return;
        }
        debug_assert!(item_ref.EndOffset() > item_ref.StartOffset());
        let space_offset = item_ref.EndOffset() - 1;
        let mut units = self.text_.ToString().Span16().unwrap_or_default().to_vec();
        debug_assert!(matches!(units[space_offset as usize], 0x20 | 0x3000));
        units.remove(space_offset as usize);
        self.text_ = StringBuilder::new();
        self.text_.AppendString(&String::from_utf16(&units));
        self.mapping_builder_.CollapseTrailingSpace(space_offset);

        let item_index = unsafe { &*item }.IndexInItems(unsafe { &*self.items_ });
        let new_end = unsafe { &*item }.EndOffset() - 1;
        unsafe { &mut *item }.SetEndOffset(new_end);
        unsafe { &mut *item }.SetEndCollapseType(CollapseType::kCollapsed);
        for later in unsafe { &mut *self.items_ }
            .iter_mut()
            .skip(item_index as usize + 1)
        {
            let later = unsafe { &mut *later.Get() };
            later.SetOffset(later.StartOffset() - 1, later.EndOffset() - 1);
        }
    }

    // cpp: layoutng_inline/inline_items_builder.cc:1412-1421
    fn RestoreTrailingCollapsibleSpaceIfRemoved(&mut self) {
        let item = LastItemToCollapseWith(self.items_);
        if !item.is_null() && unsafe { &*item }.EndCollapseType() == CollapseType::kCollapsed {
            self.RestoreTrailingCollapsibleSpace(item);
        }
    }

    // cpp: layoutng_inline/inline_items_builder.cc:1423-1450
    fn RestoreTrailingCollapsibleSpace(&mut self, item: *mut InlineItem) {
        debug_assert!(!item.is_null());
        debug_assert_eq!(
            unsafe { &*item }.EndCollapseType(),
            CollapseType::kCollapsed
        );
        let layout_text = To::<LayoutText>(unsafe { &*item }.GetLayoutObject());
        self.mapping_builder_.RestoreTrailingCollapsibleSpace(
            unsafe { &*layout_text },
            unsafe { &*item }.EndOffset(),
        );

        let space_char = GetCollapsedSpaceChar(unsafe { &*item }.Style());
        let insert_at = unsafe { &*item }.EndOffset() as usize;
        let mut units = self.text_.ToString().Span16().unwrap_or_default().to_vec();
        units.insert(insert_at, space_char);
        self.text_ = StringBuilder::new();
        self.text_.AppendString(&String::from_utf16(&units));

        let item_index = unsafe { &*item }.IndexInItems(unsafe { &*self.items_ });
        let new_end = unsafe { &*item }.EndOffset() + 1;
        unsafe { &mut *item }.SetEndOffset(new_end);
        unsafe { &mut *item }.SetEndCollapseType(CollapseType::kCollapsible);
        for later in unsafe { &mut *self.items_ }
            .iter_mut()
            .skip(item_index as usize + 1)
        {
            let later = unsafe { &mut *later.Get() };
            later.SetOffset(later.StartOffset() + 1, later.EndOffset() + 1);
        }
    }

    // cpp: layoutng_inline/inline_items_builder.cc:1732-1758
    pub fn DidFinishCollectInlines(&mut self, data: &mut InlineNodeData) {
        data.text_content = self.ToString();
        data.has_non_orc_16bit_ = self.has_non_orc_16bit_;
        data.is_bidi_enabled_ = self.HasBidiControls()
            || (self.has_non_orc_16bit_ && Character::MaybeBidiRtlString(&data.text_content));
        data.has_floats_ = self.has_floats_;
        data.has_out_of_flow_positioned_ = self.has_out_of_flow_positioned_;
        data.has_initial_letter_box_ = self.has_initial_letter_box_;
        data.has_ruby_ = self.has_ruby_;
        data.has_text_emphasis_ = self.has_text_emphasis_;
        data.is_block_level_ = self.IsBlockLevel();
        data.changes_may_affect_earlier_lines_ = self.HasUnicodeBidiPlainText();
        data.is_bisect_line_break_disabled_ = self.is_bisect_line_break_disabled_;
        data.is_score_line_break_disabled_ = self.is_score_line_break_disabled_;
        #[cfg(debug_assertions)]
        data.CheckConsistency();
    }

    // cpp: layoutng_inline/inline_items_builder.cc:1760-1769
    pub fn SetHasInititialLetterBox(&mut self) {
        debug_assert!(!unsafe { &*self.items_ }.is_empty());
        debug_assert!(!self.has_initial_letter_box_);
        self.has_initial_letter_box_ = true;
        self.is_bisect_line_break_disabled_ = true;
        self.is_score_line_break_disabled_ = true;
    }

    // cpp: layoutng_inline/inline_items_builder.cc:1771-1775
    pub fn SetIsSymbolMarker(&mut self) {
        let items = unsafe { &mut *self.items_ };
        debug_assert!(!items.is_empty());
        let item = items.last_mut().expect("items not empty").Get();
        unsafe { &mut *item }.SetIsSymbolMarker();
    }

    // cpp: layoutng_inline/inline_items_builder.cc:1777-1782,1818-1822
    pub fn ShouldUpdateLayoutObject(&self) -> bool {
        MappingBuilder::NEEDS_BOX_INFO
    }

    // cpp: layoutng_inline/inline_items_builder.cc:1784-1790,1829-1832
    pub fn ClearInlineFragment(&mut self, object: *mut LayoutObject) {
        if MappingBuilder::NEEDS_BOX_INFO {
            unsafe { &mut *object }.SetIsInLayoutNGInlineFormattingContext(true);
        }
    }

    // cpp: layoutng_inline/inline_items_builder.cc:1791-1805,1824-1827
    pub fn ClearNeedsLayout(&mut self, object: *mut LayoutObject) {
        if !MappingBuilder::NEEDS_BOX_INFO {
            return;
        }
        unsafe { &mut *object }.ClearNeedsCollectInlines();
        self.ClearInlineFragment(object);
        if unsafe { &*object }.IsText() {
            let text = To::<LayoutText>(object);
            unsafe { &mut *text }.ClearInlineItems();
        }
    }

    // cpp: layoutng_inline/inline_items_builder.cc:1807-1812,1830-1833
    pub fn UpdateShouldCreateBoxFragment(&mut self, object: *mut LayoutInline) {
        if MappingBuilder::NEEDS_BOX_INFO {
            unsafe { &mut *object }.UpdateShouldCreateBoxFragment();
        }
    }
}

// cpp: layoutng_inline/inline_items_builder.cc:43-47
impl<MappingBuilder> Drop for InlineItemsBuilderTemplate<MappingBuilder> {
    fn drop(&mut self) {
        debug_assert_eq!(self.bidi_context_.len(), 0);
        let items = unsafe { &*self.items_ };
        let end_offset = items
            .last()
            .map_or(0, |item| unsafe { &*item.Get() }.EndOffset());
        debug_assert_eq!(self.text_.ToString().length(), end_offset);
    }
}
