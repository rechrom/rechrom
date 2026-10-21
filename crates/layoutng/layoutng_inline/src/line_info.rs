// C++: layoutng_inline/line_info.h/.cc.
#![allow(non_snake_case)]

use font_engine::fonts::shaping::shape_result_types::AdjustMidCluster;
use font_engine::text::native::character::Character;
use foundation::{
    ERubyAlign, ETextAlign, ETextAlignLast, HeapVector, LayoutUnit, Member, String, TextDirection,
    Traceable, Visitor, WtfSizeT,
};
use layoutng::internal::inline_item::InlineItemType;
use layoutng::internal::inline_item_result::InlineItemResult;
use layoutng::internal::inline_item_result::InlineItemResults;
use layoutng::internal::inline_item_text_index::InlineItemTextIndex;
use layoutng::internal::inline_node::InlineNode;
use layoutng_fragment_tree::inline_break_token::InlineBreakToken;
use layoutng_fragment_tree::inline_items_data::InlineItemsData;
use layoutng_fragment_tree::layout_result::EStatus;
use layoutng_fragment_tree::layout_result::LayoutResult;
use layoutng_geometry::geometry::bfc_offset::BfcOffset;
use layoutng_style::css::white_space::WhiteSpaceCollapse;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::computed_style_initial_values::ComputedStyleInitialValues;

use crate::inline_item_result_ruby_column::InlineItemResultRubyColumn;

// cpp: layoutng_inline/line_info.cc:16-18
fn is_hanging_space(character: u16) -> bool {
    character == 0x0020 || Character::IsOtherSpaceSeparator(i32::from(character))
}

fn ruby_base_line(result: &InlineItemResult) -> &LineInfo {
    debug_assert!(result.IsRubyColumn());
    // The layoutng owner carries a forward-declared member. The allocation is
    // the inline package's source-owned Ruby column type.
    unsafe { &(*(result.ruby_column.Get() as *const InlineItemResultRubyColumn)).base_line }
}

// cpp: layoutng_inline/line_info.cc:21-36
fn glyph_count(result: &InlineItemResult) -> WtfSizeT {
    let shape = result.shape_result.Get();
    if !shape.is_null() {
        return unsafe { &*shape }.NumGlyphs();
    }
    if !result.layout_result.Get().is_null() {
        return 1;
    }
    if result.IsRubyColumn() {
        let mut count = 0;
        for nested in ruby_base_line(result).Results() {
            count += glyph_count(nested);
        }
        return count;
    }
    0
}

// cpp: layoutng_inline/line_info.h:25-31,313-370
// The source keeps LineInfo transient during line-box construction. All
// scalar defaults below match the C++ member initializers and value defaults.
// Clone preserves the source's value copies into Ruby column line lists.
#[derive(Clone)]
pub struct LineInfo {
    items_data_: Member<InlineItemsData>,
    line_style_: Member<ComputedStyle>,
    results_: InlineItemResults,
    bfc_offset_: BfcOffset,
    break_token_: Member<InlineBreakToken>,
    parallel_flow_break_tokens_: HeapVector<Member<InlineBreakToken>>,
    block_in_inline_layout_result_: Member<LayoutResult>,
    minimum_space_shortage_: Option<LayoutUnit>,
    tallest_unbreakable_block_size_: LayoutUnit,
    available_width_: LayoutUnit,
    width_: LayoutUnit,
    hang_width_: LayoutUnit,
    text_indent_: LayoutUnit,
    annotation_block_start_adjustment_: LayoutUnit,
    initial_letter_box_block_start_adjustment_: LayoutUnit,
    initial_letter_box_block_size_: LayoutUnit,
    start_: InlineItemTextIndex,
    end_item_index_: u32,
    end_offset_for_justify_: u32,
    text_fit_scale_: f32,
    text_align_: ETextAlign,
    base_direction_: TextDirection,
    is_first_formatted_line_: bool,
    is_start_of_paragraph_: bool,
    use_first_line_style_: bool,
    is_last_line_: bool,
    has_forced_break_: bool,
    is_empty_line_: bool,
    has_line_even_if_empty_: bool,
    is_block_in_inline_: bool,
    has_overflow_: bool,
    has_trailing_spaces_: bool,
    needs_accurate_end_position_: bool,
    is_ruby_base_: bool,
    is_ruby_text_: bool,
    may_have_text_combine_or_ruby_item_: bool,
    may_have_ruby_overhang_: bool,
    allow_hang_for_alignment_: bool,
}

impl Default for LineInfo {
    fn default() -> Self {
        Self {
            items_data_: Member::default(),
            line_style_: Member::default(),
            results_: InlineItemResults::default(),
            bfc_offset_: BfcOffset::default(),
            break_token_: Member::default(),
            parallel_flow_break_tokens_: HeapVector::default(),
            block_in_inline_layout_result_: Member::default(),
            minimum_space_shortage_: None,
            tallest_unbreakable_block_size_: LayoutUnit::default(),
            available_width_: LayoutUnit::default(),
            width_: LayoutUnit::default(),
            hang_width_: LayoutUnit::default(),
            text_indent_: LayoutUnit::default(),
            annotation_block_start_adjustment_: LayoutUnit::default(),
            initial_letter_box_block_start_adjustment_: LayoutUnit::default(),
            initial_letter_box_block_size_: LayoutUnit::default(),
            start_: InlineItemTextIndex::default(),
            end_item_index_: 0,
            end_offset_for_justify_: 0,
            text_fit_scale_: 1.0,
            text_align_: ETextAlign::kLeft,
            base_direction_: TextDirection::kLtr,
            is_first_formatted_line_: false,
            is_start_of_paragraph_: false,
            use_first_line_style_: false,
            is_last_line_: false,
            has_forced_break_: false,
            is_empty_line_: false,
            has_line_even_if_empty_: false,
            is_block_in_inline_: false,
            has_overflow_: false,
            has_trailing_spaces_: false,
            needs_accurate_end_position_: false,
            is_ruby_base_: false,
            is_ruby_text_: false,
            may_have_text_combine_or_ruby_item_: false,
            may_have_ruby_overhang_: false,
            allow_hang_for_alignment_: false,
        }
    }
}

impl LineInfo {
    // cpp: layoutng_inline/line_info.h:34-34
    // cpp: layoutng_inline/line_info.cc:41-48
    pub fn Trace(&self, visitor: &mut Visitor<'_>) {
        // InlineItemResult owns a source-defined Trace method but is not yet
        // wired to foundation::Traceable in the shared assembly.
        for result in &self.results_ {
            result.Trace(visitor);
        }
        visitor.Trace(&self.items_data_);
        visitor.Trace(&self.line_style_);
        visitor.Trace(&self.break_token_);
        visitor.Trace(&self.parallel_flow_break_tokens_);
        visitor.Trace(&self.block_in_inline_layout_result_);
    }

    // cpp: layoutng_inline/line_info.h:35-35
    // cpp: layoutng_inline/line_info.cc:50-94
    pub fn Reset(&mut self) {
        self.items_data_ = Member::default();
        self.line_style_ = Member::default();
        self.results_.Shrink(0);
        self.bfc_offset_ = BfcOffset::default();
        self.break_token_ = Member::default();
        self.parallel_flow_break_tokens_.Shrink(0);
        self.block_in_inline_layout_result_ = Member::default();
        self.available_width_ = LayoutUnit::default();
        self.width_ = LayoutUnit::default();
        self.hang_width_ = LayoutUnit::default();
        self.text_indent_ = LayoutUnit::default();
        self.annotation_block_start_adjustment_ = LayoutUnit::default();
        self.initial_letter_box_block_start_adjustment_ = LayoutUnit::default();
        self.initial_letter_box_block_size_ = LayoutUnit::default();
        self.start_ = InlineItemTextIndex {
            item_index: 0,
            text_offset: 0,
        };
        self.end_item_index_ = 0;
        self.end_offset_for_justify_ = 0;
        self.text_fit_scale_ = 1.0;
        self.text_align_ = ETextAlign::kLeft;
        self.base_direction_ = TextDirection::kLtr;
        self.use_first_line_style_ = false;
        self.is_last_line_ = false;
        self.has_forced_break_ = false;
        self.is_empty_line_ = false;
        self.has_line_even_if_empty_ = false;
        self.is_block_in_inline_ = false;
        self.has_overflow_ = false;
        self.has_trailing_spaces_ = false;
        self.needs_accurate_end_position_ = false;
        self.is_ruby_base_ = false;
        self.is_ruby_text_ = false;
        self.may_have_text_combine_or_ruby_item_ = false;
        self.may_have_ruby_overhang_ = false;
        self.allow_hang_for_alignment_ = false;
        self.is_start_of_paragraph_ = false;
        // The source deliberately does not reset minimum_space_shortage_,
        // tallest_unbreakable_block_size_, or is_first_formatted_line_.
    }

    // cpp: layoutng_inline/line_info.h:47-49
    // cpp: layoutng_inline/line_info.cc:96-109
    pub fn SetLineStyle(
        &mut self,
        node: &InlineNode,
        items_data: &InlineItemsData,
        use_first_line_style: bool,
    ) {
        self.use_first_line_style_ = use_first_line_style;
        self.items_data_ = Member::from_ptr(items_data as *const _ as *mut InlineItemsData);
        let box_ptr = node.GetLayoutBox();
        self.line_style_ = Member::from_ptr(
            unsafe { &*box_ptr }.StyleRefForLine(self.use_first_line_style_) as *const _
                as *mut ComputedStyle,
        );
        self.needs_accurate_end_position_ = self.ComputeNeedsAccurateEndPosition();
        self.annotation_block_start_adjustment_ = LayoutUnit::default();
        self.initial_letter_box_block_start_adjustment_ = LayoutUnit::default();
        self.initial_letter_box_block_size_ = LayoutUnit::default();
    }

    // cpp: layoutng_inline/line_info.h:37-40
    pub fn ItemsData(&self) -> &InlineItemsData {
        let data = self.items_data_.Get();
        debug_assert!(!data.is_null());
        unsafe { &*data }
    }

    // cpp: layoutng_inline/line_info.h:43-46
    pub fn LineStyle(&self) -> &ComputedStyle {
        let style = self.line_style_.Get();
        debug_assert!(!style.is_null());
        unsafe { &*style }
    }

    // cpp: layoutng_inline/line_info.h:50-50
    pub fn OverrideLineStyle(&mut self, style: &ComputedStyle) {
        self.line_style_ = Member::from_ptr(style as *const ComputedStyle as *mut ComputedStyle);
    }

    // cpp: layoutng_inline/line_info.h:54-55
    pub fn IsFirstFormattedLine(&self) -> bool {
        self.is_first_formatted_line_
    }
    pub fn SetIsFirstFormattedLine(&mut self, value: bool) {
        self.is_first_formatted_line_ = value;
    }

    // cpp: layoutng_inline/line_info.h:59-60
    pub fn IsStartOfParagraph(&self) -> bool {
        self.is_start_of_paragraph_
    }
    pub fn SetIsStartOfParagraph(&mut self, value: bool) {
        self.is_start_of_paragraph_ = value;
    }

    // cpp: layoutng_inline/line_info.h:67-67
    pub fn UseFirstLineStyle(&self) -> bool {
        self.use_first_line_style_
    }

    // cpp: layoutng_inline/line_info.h:71-72
    pub fn IsLastLine(&self) -> bool {
        self.is_last_line_
    }
    pub fn SetIsLastLine(&mut self, value: bool) {
        self.is_last_line_ = value;
    }

    // cpp: layoutng_inline/line_info.h:77-78
    pub fn HasForcedBreak(&self) -> bool {
        self.has_forced_break_
    }
    pub fn SetHasForcedBreak(&mut self) {
        self.has_forced_break_ = true;
    }

    // cpp: layoutng_inline/line_info.h:83-84
    pub fn IsEmptyLine(&self) -> bool {
        self.is_empty_line_
    }
    pub fn SetIsEmptyLine(&mut self) {
        self.is_empty_line_ = true;
    }

    // cpp: layoutng_inline/line_info.h:87-88
    pub fn HasLineEvenIfEmpty(&self) -> bool {
        self.has_line_even_if_empty_
    }
    pub fn SetHasLineEvenIfEmpty(&mut self) {
        self.has_line_even_if_empty_ = true;
    }

    // cpp: layoutng_inline/line_info.h:91-92
    pub fn IsBlockInInline(&self) -> bool {
        self.is_block_in_inline_
    }
    pub fn SetIsBlockInInline(&mut self) {
        self.is_block_in_inline_ = true;
    }

    // cpp: layoutng_inline/line_info.h:94-97
    pub fn IsRubyBase(&self) -> bool {
        self.is_ruby_base_
    }
    pub fn SetIsRubyBase(&mut self) {
        self.is_ruby_base_ = true;
    }
    pub fn IsRubyText(&self) -> bool {
        self.is_ruby_text_
    }
    pub fn SetIsRubyText(&mut self) {
        self.is_ruby_text_ = true;
    }

    // cpp: layoutng_inline/line_info.h:99-101
    pub fn MutableResults(&mut self) -> &mut InlineItemResults {
        &mut self.results_
    }
    pub fn Results(&self) -> &InlineItemResults {
        &self.results_
    }

    // cpp: layoutng_inline/line_info.h:103-109
    pub fn GetBreakToken(&self) -> *const InlineBreakToken {
        self.break_token_.Get()
    }
    pub fn SetBreakToken(&mut self, token: *const InlineBreakToken) {
        self.break_token_ = Member::from_ptr(token as *mut InlineBreakToken);
    }
    pub fn IsEndParagraph(&self) -> bool {
        self.GetBreakToken().is_null() || self.HasForcedBreak()
    }

    // cpp: layoutng_inline/line_info.h:111-117
    pub fn ParallelFlowBreakTokens(&mut self) -> &mut HeapVector<Member<InlineBreakToken>> {
        &mut self.parallel_flow_break_tokens_
    }
    pub fn PropagateParallelFlowBreakToken(&mut self, token: *const InlineBreakToken) {
        debug_assert!(unsafe { &*token }.IsInParallelBlockFlow());
        self.parallel_flow_break_tokens_
            .push(Member::from_ptr(token as *mut InlineBreakToken));
    }

    // cpp: layoutng_inline/line_info.h:120-130
    pub fn MinimumSpaceShortage(&self) -> Option<LayoutUnit> {
        self.minimum_space_shortage_
    }
    pub fn PropagateMinimumSpaceShortage(&mut self, shortage: LayoutUnit) {
        debug_assert!(shortage > LayoutUnit::default());
        self.minimum_space_shortage_ = Some(match self.minimum_space_shortage_ {
            Some(previous) => previous.min(shortage),
            None => shortage,
        });
    }

    // cpp: layoutng_inline/line_info.h:131-137
    pub fn TallestUnbreakableBlockSize(&self) -> LayoutUnit {
        self.tallest_unbreakable_block_size_
    }
    pub fn PropagateTallestUnbreakableBlockSize(&mut self, size: LayoutUnit) {
        debug_assert!(size >= LayoutUnit::default());
        self.tallest_unbreakable_block_size_ = size;
    }

    // cpp: layoutng_inline/line_info.h:139-140
    pub fn SetTextIndent(&mut self, indent: LayoutUnit) {
        self.text_indent_ = indent;
    }
    pub fn TextIndent(&self) -> LayoutUnit {
        self.text_indent_
    }

    // cpp: layoutng_inline/line_info.h:142-142
    pub fn TextAlign(&self) -> ETextAlign {
        self.text_align_
    }

    // cpp: layoutng_inline/line_info.h:147-148
    pub fn GetBfcOffset(&self) -> BfcOffset {
        self.bfc_offset_
    }
    pub fn AvailableWidth(&self) -> LayoutUnit {
        self.available_width_
    }

    // cpp: layoutng_inline/line_info.h:150-165
    pub fn Width(&self) -> LayoutUnit {
        self.width_.ClampNegativeToZero()
    }
    pub fn WidthForAlignment(&self) -> LayoutUnit {
        self.width_ - self.HangWidthForAlignment()
    }
    pub fn HangWidth(&self) -> LayoutUnit {
        self.hang_width_
    }
    pub fn HangWidthForAlignment(&self) -> LayoutUnit {
        if self.allow_hang_for_alignment_ {
            self.hang_width_
        } else {
            LayoutUnit::default()
        }
    }

    // cpp: layoutng_inline/line_info.h:178-183
    pub fn HasTrailingSpaces(&self) -> bool {
        self.has_trailing_spaces_
    }
    pub fn SetHasTrailingSpaces(&mut self) {
        self.has_trailing_spaces_ = true;
    }
    pub fn HasOverflow(&self) -> bool {
        self.has_overflow_
    }
    pub fn SetHasOverflow(&mut self, value: bool) {
        self.has_overflow_ = value;
    }
    pub fn SetHasOverflowDefault(&mut self) {
        self.SetHasOverflow(true);
    }

    // cpp: layoutng_inline/line_info.h:188-192
    pub fn SetBfcOffset(&mut self, offset: &BfcOffset) {
        self.bfc_offset_ = *offset;
    }
    pub fn SetWidth(&mut self, available_width: LayoutUnit, width: LayoutUnit) {
        self.available_width_ = available_width;
        self.width_ = width;
    }

    // cpp: layoutng_inline/line_info.h:195-197
    pub fn Start(&self) -> &InlineItemTextIndex {
        &self.start_
    }
    pub fn StartOffset(&self) -> u32 {
        self.start_.text_offset
    }
    pub fn SetStart(&mut self, index: &InlineItemTextIndex) {
        self.start_ = *index;
    }

    // cpp: layoutng_inline/line_info.h:211-217
    pub fn InflowEndOffset(&self) -> u32 {
        self.InflowEndOffsetInternal(false)
    }
    pub fn InflowEndOffsetWithoutForcedBreak(&self) -> u32 {
        self.InflowEndOffsetInternal(true)
    }

    // cpp: layoutng_inline/line_info.h:220-226
    pub fn EndOffsetForJustify(&self) -> u32 {
        debug_assert_eq!(self.text_align_, ETextAlign::kJustify);
        self.end_offset_for_justify_
    }
    pub fn EndItemIndex(&self) -> u32 {
        self.end_item_index_
    }
    pub fn SetEndItemIndex(&mut self, index: u32) {
        self.end_item_index_ = index;
    }

    // cpp: layoutng_inline/line_info.h:230-238
    pub fn BaseDirection(&self) -> TextDirection {
        self.base_direction_
    }
    pub fn SetBaseDirection(&mut self, direction: TextDirection) {
        self.base_direction_ = direction;
    }
    pub fn NeedsAccurateEndPosition(&self) -> bool {
        self.needs_accurate_end_position_
    }

    // cpp: layoutng_inline/line_info.h:240-246
    pub fn BlockInInlineLayoutResult(&self) -> *const LayoutResult {
        self.block_in_inline_layout_result_.Get()
    }
    pub fn SetBlockInInlineLayoutResult(&mut self, result: *const LayoutResult) {
        self.block_in_inline_layout_result_ = Member::from_ptr(result as *mut LayoutResult);
    }

    // cpp: layoutng_inline/line_info.h:254-264
    pub fn MayHaveTextCombineOrRubyItem(&self) -> bool {
        self.may_have_text_combine_or_ruby_item_
    }
    pub fn SetHaveTextCombineOrRubyItem(&mut self) {
        self.may_have_text_combine_or_ruby_item_ = true;
    }
    pub fn MayHaveRubyOverhang(&self) -> bool {
        self.may_have_ruby_overhang_
    }
    pub fn SetMayHaveRubyOverhang(&mut self) {
        self.may_have_ruby_overhang_ = true;
    }

    // cpp: layoutng_inline/line_info.h:285-299
    pub fn SetAnnotationBlockStartAdjustment(&mut self, amount: LayoutUnit) {
        debug_assert!(!self.IsEmptyLine());
        self.annotation_block_start_adjustment_ = amount;
    }
    pub fn SetInitialLetterBlockStartAdjustment(&mut self, amount: LayoutUnit) {
        debug_assert!(amount >= LayoutUnit::default());
        debug_assert!(!self.IsEmptyLine());
        self.initial_letter_box_block_start_adjustment_ = amount;
    }
    pub fn SetInitialLetterBoxBlockSize(&mut self, block_size: LayoutUnit) {
        debug_assert!(block_size >= LayoutUnit::default());
        self.initial_letter_box_block_size_ = block_size;
    }

    // cpp: layoutng_inline/line_info.h:301-302
    pub fn SetTextFitScale(&mut self, scale: f32) {
        self.text_fit_scale_ = scale;
    }
    pub fn TextFitScale(&self) -> f32 {
        self.text_fit_scale_
    }

    // cpp: layoutng_inline/line_info.h:305-305
    // cpp: layoutng_inline/line_info.cc:111-127
    fn GetTextAlign(&self, is_last_line: bool) -> ETextAlign {
        if self.is_ruby_base_ {
            return ETextAlign::kJustify;
        }
        if self.is_ruby_text_ {
            let text_align = self.LineStyle().GetTextAlign();
            let ruby_align = self.LineStyle().RubyAlign();
            if (ruby_align == ERubyAlign::kSpaceAround
                && (text_align == ComputedStyleInitialValues::InitialTextAlign()
                    || text_align == ETextAlign::kJustify))
                || ruby_align == ERubyAlign::kSpaceBetween
            {
                return ETextAlign::kJustify;
            }
        }
        self.LineStyle().GetTextAlignForLine(is_last_line)
    }

    // cpp: layoutng_inline/line_info.h:306-306
    // cpp: layoutng_inline/line_info.cc:129-181
    fn ComputeNeedsAccurateEndPosition(&self) -> bool {
        match self.GetTextAlign(false) {
            ETextAlign::kStart => {}
            ETextAlign::kEnd
            | ETextAlign::kCenter
            | ETextAlign::kWebkitCenter
            | ETextAlign::kJustify
            | ETextAlign::kMatchParent => return true,
            ETextAlign::kLeft | ETextAlign::kWebkitLeft => {
                if self.BaseDirection() == TextDirection::kRtl {
                    return true;
                }
            }
            ETextAlign::kRight | ETextAlign::kWebkitRight => {
                if self.BaseDirection() == TextDirection::kLtr {
                    return true;
                }
            }
        }
        let mut align_last = self.LineStyle().TextAlignLast();
        if self.is_ruby_base_ {
            align_last = ETextAlignLast::kJustify;
        } else if self.is_ruby_text_
            && align_last == ComputedStyleInitialValues::InitialTextAlignLast()
        {
            align_last = ETextAlignLast::kJustify;
        }
        match align_last {
            ETextAlignLast::kStart | ETextAlignLast::kAuto => return false,
            ETextAlignLast::kEnd
            | ETextAlignLast::kCenter
            | ETextAlignLast::kJustify
            | ETextAlignLast::kMatchParent => return true,
            ETextAlignLast::kLeft => {
                if self.BaseDirection() == TextDirection::kRtl {
                    return true;
                }
            }
            ETextAlignLast::kRight => {
                if self.BaseDirection() == TextDirection::kLtr {
                    return true;
                }
            }
        }
        false
    }

    // cpp: layoutng_inline/line_info.h:203-203
    // cpp: layoutng_inline/line_info.cc:183-200
    pub fn InflowStartOffset(&self) -> u32 {
        for result in self.Results() {
            let item = unsafe { &*result.item.Get() };
            if matches!(
                item.Type(),
                InlineItemType::kText | InlineItemType::kControl | InlineItemType::kAtomicInline
            ) && item.Length() > 0
            {
                return result.StartOffset();
            } else if result.IsRubyColumn() {
                let base_line = ruby_base_line(result);
                let start_offset = base_line.InflowStartOffset();
                if start_offset != base_line.EndTextOffset() {
                    return start_offset;
                }
            }
        }
        self.EndTextOffset()
    }

    // cpp: layoutng_inline/line_info.h:207-207
    // cpp: layoutng_inline/line_info.cc:202-210
    pub fn End(&self) -> InlineItemTextIndex {
        let break_token = self.GetBreakToken();
        if !break_token.is_null() {
            return *unsafe { &*break_token }.Start();
        }
        let data = self.ItemsData();
        if self.end_item_index_ != 0 && (self.end_item_index_ as usize) < data.items.len() {
            return InlineItemTextIndex {
                item_index: self.end_item_index_,
                text_offset: unsafe { &*data.items[self.end_item_index_ as usize].Get() }
                    .StartOffset(),
            };
        }
        data.End()
    }

    // cpp: layoutng_inline/line_info.h:208-208
    // cpp: layoutng_inline/line_info.cc:212-220
    pub fn EndTextOffset(&self) -> u32 {
        let break_token = self.GetBreakToken();
        if !break_token.is_null() {
            return unsafe { &*break_token }.StartTextOffset();
        }
        let data = self.ItemsData();
        if self.end_item_index_ != 0 && (self.end_item_index_ as usize) < data.items.len() {
            return unsafe { &*data.items[self.end_item_index_ as usize].Get() }.StartOffset();
        }
        data.text_content.length()
    }

    // cpp: layoutng_inline/line_info.h:311-311
    // cpp: layoutng_inline/line_info.cc:222-248
    fn InflowEndOffsetInternal(&self, skip_forced_break: bool) -> u32 {
        for result in self.Results().iter().rev() {
            let item_ptr = result.item.Get();
            debug_assert!(!item_ptr.is_null());
            let item = unsafe { &*item_ptr };
            if skip_forced_break {
                if item.Type() == InlineItemType::kControl
                    && self
                        .ItemsData()
                        .text_content
                        .Span16()
                        .expect("non-null text")[item.StartOffset() as usize]
                        == 0x000a
                {
                    continue;
                } else if item.Type() == InlineItemType::kText && item.Length() == 0 {
                    continue;
                }
            }
            if matches!(
                item.Type(),
                InlineItemType::kText | InlineItemType::kControl | InlineItemType::kAtomicInline
            ) {
                return result.EndOffset();
            } else if result.IsRubyColumn() {
                let base_line = ruby_base_line(result);
                let end_offset = base_line.InflowEndOffsetInternal(skip_forced_break);
                if end_offset != base_line.StartOffset() {
                    return end_offset;
                }
            }
        }
        self.StartOffset()
    }

    // cpp: layoutng_inline/line_info.h:228-228
    // cpp: layoutng_inline/line_info.cc:250-259
    pub fn GlyphCountIsGreaterThan(&self, limit: WtfSizeT) -> bool {
        let mut count = 0;
        for result in self.Results() {
            count += glyph_count(result);
            if count > limit {
                return true;
            }
        }
        false
    }

    // cpp: layoutng_inline/line_info.h:186-186
    // cpp: layoutng_inline/line_info.cc:261-268
    pub fn IsHyphenated(&self) -> bool {
        for result in self.Results().iter().rev() {
            if result.Length() != 0 {
                return result.is_hyphenated;
            }
        }
        false
    }

    // cpp: layoutng_inline/line_info.h:250-250
    // cpp: layoutng_inline/line_info.cc:270-275
    pub fn HasUnsuccessfulBlockInInline(&self) -> bool {
        let result = self.BlockInInlineLayoutResult();
        !result.is_null() && unsafe { &*result }.Status() != EStatus::kSuccess
    }

    // cpp: layoutng_inline/line_info.h:145-145
    // cpp: layoutng_inline/line_info.cc:277-291
    pub fn UpdateTextAlign(&mut self) {
        self.text_align_ = self.GetTextAlign(self.IsLastLine());
        self.allow_hang_for_alignment_ = true;
        if self.HasTrailingSpaces() {
            let mut end_offset = self.end_offset_for_justify_;
            self.hang_width_ = self.ComputeTrailingSpaceWidth(Some(&mut end_offset));
            self.end_offset_for_justify_ = end_offset;
            return;
        }
        self.hang_width_ = LayoutUnit::default();
        if self.text_align_ == ETextAlign::kJustify {
            self.end_offset_for_justify_ = self.InflowEndOffset();
        }
    }

    // cpp: layoutng_inline/line_info.h:309-310
    // cpp: layoutng_inline/line_info.cc:293-418
    fn ComputeTrailingSpaceWidth(&self, mut end_offset_out: Option<&mut u32>) -> LayoutUnit {
        if !self.has_trailing_spaces_ {
            if let Some(out) = end_offset_out.as_deref_mut() {
                *out = self.InflowEndOffset();
            }
            return LayoutUnit::default();
        }

        let mut trailing_spaces_width = LayoutUnit::default();
        for result in self.Results().iter().rev() {
            let item_ptr = result.item.Get();
            debug_assert!(!item_ptr.is_null());
            let item = unsafe { &*item_ptr };

            if item.EndCollapseType()
                == layoutng::internal::inline_item::CollapseType::kOpaqueToCollapsing
            {
                continue;
            }
            debug_assert!(!matches!(
                item.Type(),
                InlineItemType::kFloating
                    | InlineItemType::kOutOfFlowPositioned
                    | InlineItemType::kBidiControl
            ));

            let mut trailing_item_width = LayoutUnit::default();
            let mut will_continue = false;
            let mut end_offset = result.EndOffset();
            if item.Type() == InlineItemType::kControl || result.has_only_pre_wrap_trailing_spaces {
                trailing_item_width = result.inline_size;
                will_continue = true;
            } else if item.Type() == InlineItemType::kText {
                if result.Length() == 0 {
                    debug_assert_eq!(result.inline_size, LayoutUnit::default());
                    continue;
                }
                let text = &self.ItemsData().text_content;
                let units = text.Span16().expect("non-null IFC text");
                debug_assert!(end_offset >= 1);
                if is_hanging_space(units[(end_offset - 1) as usize]) {
                    loop {
                        end_offset -= 1;
                        if end_offset <= result.StartOffset()
                            || !is_hanging_space(units[(end_offset - 1) as usize])
                        {
                            break;
                        }
                    }
                    if end_offset == result.StartOffset() {
                        trailing_item_width = result.inline_size;
                        will_continue = true;
                    } else {
                        // Match the source's deliberately unreshaped estimate
                        // at offsets that are not safe-to-break.
                        debug_assert_eq!(item.Direction(), self.BaseDirection());
                        let shape_view = result.shape_result.Get();
                        let shape_ptr = unsafe { &*shape_view }.CreateShapeResult();
                        let shape = unsafe { &*shape_ptr };
                        let end_position = shape.PositionForOffset(
                            end_offset - shape.StartIndex(),
                            AdjustMidCluster::kToEnd,
                        );
                        trailing_item_width = if self.BaseDirection() == TextDirection::kRtl {
                            LayoutUnit::from_f32(end_position)
                        } else {
                            LayoutUnit::from_f32(shape.Width() - end_position)
                        };
                    }
                }
            }

            if trailing_item_width != LayoutUnit::default() {
                let style = unsafe { &*item.Style() };
                let collapse = style.GetWhiteSpaceCollapse();
                if collapse == WhiteSpaceCollapse::kCollapse
                    || collapse == WhiteSpaceCollapse::kPreserveBreaks
                {
                    trailing_spaces_width += trailing_item_width;
                } else if collapse == WhiteSpaceCollapse::kPreserve && style.ShouldWrapLine() {
                    if trailing_spaces_width == LayoutUnit::default()
                        && (self.HasForcedBreak() || self.IsLastLine())
                    {
                        let item_end = self.width_ - trailing_spaces_width;
                        let actual_hang_width = trailing_item_width
                            .min(item_end - self.available_width_)
                            .ClampNegativeToZero();
                        if actual_hang_width != trailing_item_width {
                            will_continue = false;
                        }
                        trailing_spaces_width += actual_hang_width;
                    } else {
                        trailing_spaces_width += trailing_item_width;
                    }
                } else {
                    // Preserve without wrapping and break-spaces do not hang.
                    if will_continue {
                        end_offset = item.EndOffset();
                        will_continue = false;
                    }
                }
            } else {
                trailing_spaces_width += trailing_item_width;
            }

            if !will_continue {
                if let Some(out) = end_offset_out.as_deref_mut() {
                    *out = end_offset;
                }
                return trailing_spaces_width;
            }
        }

        if let Some(out) = end_offset_out.as_deref_mut() {
            *out = self.StartOffset();
        }
        trailing_spaces_width
    }

    // cpp: layoutng_inline/line_info.h:170-170
    // cpp: layoutng_inline/line_info.cc:420-427
    pub fn ComputeWidth(&self) -> LayoutUnit {
        let mut inline_size = self.TextIndent();
        for result in self.Results() {
            inline_size += result.inline_size;
        }
        inline_size
    }

    // cpp: layoutng_inline/line_info.h:172-176
    // cpp: layoutng_inline/line_info.cc:429-438
    #[cfg(debug_assertions)]
    pub fn ComputeWidthInFloat(&self) -> f32 {
        let mut inline_size = self.TextIndent().ToFloat();
        for result in self.Results() {
            inline_size += result.inline_size.ToFloat();
        }
        inline_size
    }

    // cpp: layoutng_inline/line_info.h:268-268
    // cpp: layoutng_inline/line_info.cc:496-509
    pub fn ComputeAnnotationBlockOffsetAdjustment(&self) -> LayoutUnit {
        if self.annotation_block_start_adjustment_ < LayoutUnit::default() {
            return self.annotation_block_start_adjustment_
                + self.initial_letter_box_block_start_adjustment_;
        }
        (self.annotation_block_start_adjustment_ - self.initial_letter_box_block_start_adjustment_)
            .max(LayoutUnit::default())
    }

    // cpp: layoutng_inline/line_info.h:272-272
    // cpp: layoutng_inline/line_info.cc:511-524
    pub fn ComputeBlockStartAdjustment(&self) -> LayoutUnit {
        if self.annotation_block_start_adjustment_ < LayoutUnit::default() {
            return self.annotation_block_start_adjustment_
                + self.initial_letter_box_block_start_adjustment_;
        }
        self.annotation_block_start_adjustment_
            .max(self.initial_letter_box_block_start_adjustment_)
    }

    // cpp: layoutng_inline/line_info.h:276-276
    // cpp: layoutng_inline/line_info.cc:525-536
    pub fn ComputeInitialLetterBoxBlockStartAdjustment(&self) -> LayoutUnit {
        if self.annotation_block_start_adjustment_ == LayoutUnit::default() {
            return LayoutUnit::default();
        }
        if self.annotation_block_start_adjustment_ < LayoutUnit::default() {
            return (self.initial_letter_box_block_start_adjustment_
                + self.annotation_block_start_adjustment_)
                .min(LayoutUnit::default());
        }
        (self.annotation_block_start_adjustment_ - self.initial_letter_box_block_start_adjustment_)
            .max(LayoutUnit::default())
    }

    // cpp: layoutng_inline/line_info.h:281-283
    // cpp: layoutng_inline/line_info.cc:538-546
    pub fn ComputeTotalBlockSize(
        &self,
        line_height: LayoutUnit,
        annotation_overflow_block_end: LayoutUnit,
    ) -> LayoutUnit {
        debug_assert!(annotation_overflow_block_end >= LayoutUnit::default());
        let line_height_with_annotation =
            line_height + self.annotation_block_start_adjustment_ + annotation_overflow_block_end;
        self.initial_letter_box_block_size_
            .max(line_height_with_annotation)
    }

    // cpp: layoutng_inline/line_info.h:118-118
    // cpp: layoutng_inline/line_info.cc:548-564
    pub fn RemoveParallelFlowBreakToken(&mut self, item_index: u32) {
        #[cfg(feature = "expensive_dchecks")]
        debug_assert!(self.parallel_flow_break_tokens_.windows(2).all(|pair| {
            unsafe { &*pair[0].Get() }.StartItemIndex()
                <= unsafe { &*pair[1].Get() }.StartItemIndex()
        }));
        for index in 0..self.parallel_flow_break_tokens_.len() {
            let token = unsafe { &*self.parallel_flow_break_tokens_[index].Get() };
            debug_assert!(token.IsInParallelBlockFlow());
            if token.StartItemIndex() >= item_index {
                self.parallel_flow_break_tokens_.Shrink(index as u32);
                break;
            }
        }
    }
}

impl Traceable for LineInfo {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        LineInfo::Trace(self, visitor);
    }
}

// cpp: layoutng_inline/line_info.h:375-375
// cpp: layoutng_inline/line_info.cc:566-575
impl std::fmt::Display for LineInfo {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            output,
            "LineInfo available_width_={} width_={} Results=[\n",
            self.AvailableWidth(),
            self.Width()
        )?;
        let text_content = &self.ItemsData().text_content;
        for result in self.Results() {
            write!(
                output,
                "{}\n",
                result.ToString(text_content, &String::from("\t")).Utf8()
            )?;
        }
        output.write_str("]")
    }
}
