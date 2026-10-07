#![allow(non_snake_case)]

use font_engine::{Font, RunSegmenterRange, ShapeResult};
use foundation::{
    DirectionFromLevel, HeapVector, MakeGarbageCollected, Member, String, TextDirection, To,
    Visitor,
};
use layoutng_style::style::computed_style::ComputedStyle;

use super::inline_item_segment::InlineItemSegment;
use super::layout_inline::LayoutInline;
use super::layout_object::LayoutObject;
use super::style_variant::StyleVariant;
use super::text_item_type::TextItemType;

// Non-inline definitions are owned by //src/layoutng_inline/inline_item.cc.
unsafe extern "Rust" {
    fn InlineItemNew(
        type_: InlineItemType,
        start: u32,
        end: u32,
        layout_object: *mut LayoutObject,
    ) -> InlineItem;
    fn InlineItemCloneAdjusted(
        item: &InlineItem,
        adjusted_start: u32,
        adjusted_end: u32,
        shape_result: *const ShapeResult,
    ) -> InlineItem;
    fn InlineItemClone(item: &InlineItem) -> InlineItem;
    fn InlineItemTypeToStringFromInline(type_: InlineItemType) -> &'static str;
    fn InlineItemIndexInItems(item: &InlineItem, items: &[Member<InlineItem>]) -> u32;
    fn InlineItemFontWithSvgScaling(item: &InlineItem) -> *const Font;
    fn InlineItemSplit(items: &mut InlineItems, index: u32, offset: u32);
    fn InlineItemSetSegmentData(range: &RunSegmenterRange, items: &mut InlineItems);
    fn InlineItemSetBidiLevelForItems(
        items: &mut InlineItems,
        index: u32,
        end_offset: u32,
        level: u8,
        num_out_of_flow: u32,
    ) -> u32;
    fn InlineItemUpdateIndex(items: &mut [Member<InlineItem>]);
    #[cfg(feature = "expensive_dchecks")]
    fn InlineItemCheckIndex(items: &mut [Member<InlineItem>]);
    #[cfg(debug_assertions)]
    fn InlineItemCheckTextType(item: &InlineItem, text_content: &String);
    fn InlineItemToString(item: &InlineItem) -> String;
    fn InlineItemTrace(item: &InlineItem, visitor: &mut Visitor);
    fn InlineItemComputeBoxProperties(item: &mut InlineItem);
}

// cpp: layoutng/internal/inline_item.h:31-52
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InlineItemType {
    kText,
    kControl,
    kAtomicInline,
    kBlockInInline,
    kOpenTag,
    kCloseTag,
    kFloating,
    kOutOfFlowPositioned,
    kInitialLetterBox,
    kListMarker,
    kBidiControl,
    kOpenRubyColumn,
    kCloseRubyColumn,
    kRubyLinePlaceholder,
}

// cpp: layoutng/internal/inline_item.h:54-62
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollapseType {
    kNotCollapsible,
    kOpaqueToCollapsing,
    kCollapsible,
    kCollapsed,
}

// cpp: layoutng/internal/inline_item.h:21-21
pub type InlineItems = HeapVector<Member<InlineItem>>;

// C++ stores the small flags in bitfields. Separate Rust scalars keep the
// named state and branching order; this record is not a C++ ABI boundary.
// cpp: layoutng/internal/inline_item.h:27-326
pub struct InlineItem {
    pub(crate) start_offset_: u32,
    pub(crate) end_offset_: u32,
    pub(crate) shape_result_: Member<ShapeResult>,
    pub(crate) layout_object_: Member<LayoutObject>,
    pub(crate) index_: u32,
    pub(crate) type_: InlineItemType,
    pub(crate) text_type_: u8,
    pub(crate) style_variant_: u8,
    pub(crate) end_collapse_type_: CollapseType,
    pub(crate) bidi_level_: u8,
    pub(crate) segment_data_: u32,
    pub(crate) is_empty_item_: bool,
    pub(crate) is_block_level_: bool,
    pub(crate) is_end_collapsible_newline_: bool,
    pub(crate) is_generated_for_line_break_: bool,
    pub(crate) is_unsafe_to_reuse_shape_result_: bool,
}

impl foundation::Traceable for InlineItem {
    // cpp: layoutng_inline/inline_item.cc:309-312
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        InlineItem::Trace(self, visitor);
    }
}

impl InlineItem {
    // The out-of-line constructor is in //src/layoutng_inline/inline_item.cc.
    // This source-owned initializer keeps private fields inside the shared
    // LayoutNG/inline assembly instead of duplicating InlineItem's type.
    // cpp: layoutng_inline/inline_item.cc:71-83
    pub(crate) fn new_base_for_inline(
        type_: InlineItemType,
        start: u32,
        end: u32,
        layout_object: *mut LayoutObject,
    ) -> Self {
        Self {
            start_offset_: start,
            end_offset_: end,
            shape_result_: Member::default(),
            layout_object_: Member::from_ptr(layout_object),
            index_: 0,
            type_: type_,
            text_type_: TextItemType::kNormal as u8,
            style_variant_: StyleVariant::kStandard as u8,
            end_collapse_type_: CollapseType::kNotCollapsible,
            bidi_level_: 0,
            segment_data_: 0,
            is_empty_item_: false,
            is_block_level_: false,
            is_end_collapsible_newline_: false,
            is_generated_for_line_break_: false,
            is_unsafe_to_reuse_shape_result_: false,
        }
    }

    // cpp: layoutng_inline/inline_item.cc:85-108
    pub(crate) fn clone_adjusted_base_for_inline(
        other: &Self,
        start: u32,
        end: u32,
        shape_result: *const ShapeResult,
    ) -> Self {
        Self {
            start_offset_: start,
            end_offset_: end,
            shape_result_: Member::from_ptr(shape_result as *mut ShapeResult),
            layout_object_: other.layout_object_,
            index_: other.index_,
            type_: other.type_,
            text_type_: other.text_type_,
            style_variant_: other.style_variant_,
            end_collapse_type_: other.end_collapse_type_,
            bidi_level_: other.bidi_level_,
            segment_data_: other.segment_data_,
            is_empty_item_: other.is_empty_item_,
            is_block_level_: other.is_block_level_,
            is_end_collapsible_newline_: other.is_end_collapsible_newline_,
            is_generated_for_line_break_: other.is_generated_for_line_break_,
            is_unsafe_to_reuse_shape_result_: other.is_unsafe_to_reuse_shape_result_,
        }
    }
}

impl Clone for InlineItem {
    fn clone(&self) -> Self {
        unsafe { InlineItemClone(self) }
    }
}

impl InlineItem {
    // cpp: layoutng/internal/inline_item.h:66-78
    pub fn new(
        type_: InlineItemType,
        start: u32,
        end: u32,
        layout_object: *mut LayoutObject,
    ) -> Self {
        unsafe { InlineItemNew(type_, start, end, layout_object) }
    }

    pub fn clone_adjusted(
        &self,
        adjusted_start: u32,
        adjusted_end: u32,
        shape_result: *const ShapeResult,
    ) -> Self {
        unsafe { InlineItemCloneAdjusted(self, adjusted_start, adjusted_end, shape_result) }
    }

    // cpp: layoutng/internal/inline_item.h:80-82
    pub fn Type(&self) -> InlineItemType {
        self.type_
    }

    pub fn InlineItemTypeToString(value: InlineItemType) -> &'static str {
        unsafe { InlineItemTypeToStringFromInline(value) }
    }

    // cpp: layoutng/internal/inline_item.h:84-114
    pub fn TextType(&self) -> TextItemType {
        match self.text_type_ {
            0 => TextItemType::kNormal,
            1 => TextItemType::kForcedLineBreak,
            2 => TextItemType::kFlowControl,
            3 => TextItemType::kSymbolMarker,
            4 => TextItemType::kLayoutGenerated,
            _ => unreachable!("invalid TextItemType bitfield"),
        }
    }

    pub fn IsForcedLineBreak(&self) -> bool {
        self.TextType() == TextItemType::kForcedLineBreak
    }
    pub fn SetTextType(&mut self, value: TextItemType) {
        self.text_type_ = value as u8;
    }
    pub fn IsFloatingOrOutOfFlowPositioned(&self) -> bool {
        self.Type() == InlineItemType::kFloating
            || self.Type() == InlineItemType::kOutOfFlowPositioned
    }
    pub fn IsSymbolMarker(&self) -> bool {
        self.TextType() == TextItemType::kSymbolMarker
    }
    pub fn SetIsSymbolMarker(&mut self) {
        debug_assert!(matches!(
            self.TextType(),
            TextItemType::kNormal | TextItemType::kSymbolMarker
        ));
        self.SetTextType(TextItemType::kSymbolMarker);
    }
    pub fn IsOpaqueForTextProcessing(&self) -> bool {
        self.Length() == 0
            || self.Type() == InlineItemType::kFloating
            || self.Type() == InlineItemType::kOutOfFlowPositioned
    }

    // cpp: layoutng/internal/inline_item.h:118-134
    pub fn Index(&self) -> u32 {
        self.index_
    }
    pub fn IndexInItems(&self, items: &[Member<InlineItem>]) -> u32 {
        unsafe { InlineItemIndexInItems(self, items) }
    }
    pub fn TextShapeResult(&self) -> *const ShapeResult {
        self.shape_result_.Get()
    }
    // cpp: layoutng_inline/inline_node.cc:1650-1650,1755-1755,1799-1799
    // Out-of-line shaping writes this source-owned GC member.
    pub fn SetTextShapeResultForInline(&mut self, result: *const ShapeResult) {
        self.shape_result_ = Member::from_ptr(result.cast_mut());
    }
    pub fn CloneTextShapeResult(&mut self) -> *mut ShapeResult {
        let original = self.shape_result_.Get();
        debug_assert!(!original.is_null());
        let clone = unsafe { &*original }.DeepCopy();
        self.shape_result_ = Member::from_ptr(clone);
        clone
    }
    pub fn IsUnsafeToReuseShapeResult(&self) -> bool {
        self.is_unsafe_to_reuse_shape_result_
    }
    pub fn SetUnsafeToReuseShapeResult(&mut self) {
        self.is_unsafe_to_reuse_shape_result_ = true;
    }
    #[cfg(debug_assertions)]
    pub fn CheckTextType(&self, text_content: &String) {
        unsafe { InlineItemCheckTextType(self, text_content) }
    }

    // cpp: layoutng/internal/inline_item.h:140-156
    pub fn IsEmptyItem(&self) -> bool {
        self.is_empty_item_
    }
    pub fn SetIsEmptyItem(&mut self, value: bool) {
        self.is_empty_item_ = value;
    }
    pub fn IsBlockLevel(&self) -> bool {
        self.is_block_level_
    }
    pub fn SetIsBlockLevel(&mut self, value: bool) {
        self.is_block_level_ = value;
    }

    // cpp: layoutng/internal/inline_item.h:160-171
    pub fn ShouldCreateBoxFragment(&self) -> bool {
        if self.Type() == InlineItemType::kOpenTag || self.Type() == InlineItemType::kCloseTag {
            let inline = To::<LayoutInline>(self.layout_object_.Get());
            return unsafe { &*inline }.ShouldCreateBoxFragment();
        }
        debug_assert_eq!(self.Type(), InlineItemType::kAtomicInline);
        false
    }

    pub fn SetShouldCreateBoxFragment(&mut self) {
        debug_assert!(matches!(
            self.Type(),
            InlineItemType::kOpenTag | InlineItemType::kCloseTag
        ));
        let inline = To::<LayoutInline>(self.layout_object_.Get());
        unsafe { &mut *inline }.SetShouldCreateBoxFragment(true);
    }

    // cpp: layoutng/internal/inline_item.h:173-181
    pub fn StartOffset(&self) -> u32 {
        self.start_offset_
    }
    pub fn EndOffset(&self) -> u32 {
        self.end_offset_
    }
    pub fn Length(&self) -> u32 {
        self.end_offset_.wrapping_sub(self.start_offset_)
    }
    pub fn Direction(&self) -> TextDirection {
        DirectionFromLevel(self.BidiLevel() as u32)
    }
    pub fn BidiLevel(&self) -> u8 {
        self.bidi_level_
    }
    pub fn BidiLevelForReorder(&self) -> u8 {
        if self.Type() != InlineItemType::kListMarker {
            self.BidiLevel()
        } else {
            0
        }
    }

    // cpp: layoutng/internal/inline_item.h:187-197
    pub fn GetLayoutObject(&self) -> *mut LayoutObject {
        self.layout_object_.Get()
    }
    pub fn IsImage(&self) -> bool {
        let object = self.GetLayoutObject();
        !object.is_null() && unsafe { &*object }.IsLayoutImage()
    }
    pub fn IsTextCombine(&self) -> bool {
        let object = self.GetLayoutObject();
        !object.is_null() && unsafe { &*object }.IsLayoutTextCombine()
    }

    // cpp: layoutng/internal/inline_item.h:199-214
    pub fn SetOffset(&mut self, start: u32, end: u32) {
        debug_assert!(end >= start);
        self.start_offset_ = start;
        self.end_offset_ = end;
        self.shape_result_ = Member::default();
    }
    pub fn SetEndOffset(&mut self, end_offset: u32) {
        debug_assert!(end_offset >= self.start_offset_);
        self.end_offset_ = end_offset;
        self.shape_result_ = Member::default();
    }

    // cpp: layoutng/internal/inline_item.h:216-228
    pub fn SetStyleVariant(&mut self, value: StyleVariant) {
        self.style_variant_ = value as u8;
    }
    pub fn GetStyleVariant(&self) -> StyleVariant {
        match self.style_variant_ {
            0 => StyleVariant::kStandard,
            1 => StyleVariant::kFirstLine,
            2 => StyleVariant::kStandardEllipsis,
            3 => StyleVariant::kFirstLineEllipsis,
            _ => unreachable!("invalid StyleVariant bitfield"),
        }
    }
    pub fn Style(&self) -> *const ComputedStyle {
        debug_assert!(!self.layout_object_.Get().is_null());
        unsafe { &*self.layout_object_.Get() }.EffectiveStyle(self.GetStyleVariant())
    }
    pub fn FontWithSvgScaling(&self) -> *const Font {
        unsafe { InlineItemFontWithSvgScaling(self) }
    }

    // cpp: layoutng/internal/inline_item.h:234-255
    pub fn EndCollapseType(&self) -> CollapseType {
        self.end_collapse_type_
    }
    pub fn SetEndCollapseType(&mut self, value: CollapseType) {
        debug_assert!(
            self.Type() == InlineItemType::kText
                || (matches!(
                    self.Type(),
                    InlineItemType::kControl | InlineItemType::kBlockInInline
                ) && value == CollapseType::kCollapsible)
                || value == CollapseType::kOpaqueToCollapsing
        );
        self.end_collapse_type_ = value;
    }
    pub fn IsCollapsibleSpaceOnly(&self) -> bool {
        self.Type() == InlineItemType::kText
            && self.end_collapse_type_ == CollapseType::kCollapsible
            && self.Length() == 1
    }
    pub fn IsGeneratedForLineBreak(&self) -> bool {
        self.is_generated_for_line_break_
    }
    pub fn SetIsGeneratedForLineBreak(&mut self) {
        self.is_generated_for_line_break_ = true;
    }

    // cpp: layoutng/internal/inline_item.h:264-273
    pub fn IsEndCollapsibleNewline(&self) -> bool {
        self.is_end_collapsible_newline_
    }
    pub fn SetEndCollapseTypeWithNewline(&mut self, value: CollapseType, is_newline: bool) {
        self.SetEndCollapseType(value);
        self.is_end_collapsible_newline_ = is_newline;
    }
    pub fn Split(items: &mut InlineItems, index: u32, offset: u32) {
        unsafe { InlineItemSplit(items, index, offset) }
    }

    // cpp: layoutng/internal/inline_item.h:276-292
    pub fn SegmentData(&self) -> u32 {
        self.segment_data_
    }
    // The out-of-line InlineNode implementation in //src/layoutng_inline
    // copies this source-owned packed field when reusing a prior text run.
    // cpp: layoutng_inline/inline_node.cc:1263-1267
    pub fn SetPackedSegmentDataForInline(&mut self, value: u32) {
        self.segment_data_ = value;
    }
    pub fn SetSegmentData(range: &RunSegmenterRange, items: &mut InlineItems) {
        unsafe { InlineItemSetSegmentData(range, items) }
    }
    pub fn CreateRunSegmenterRange(&self) -> RunSegmenterRange {
        debug_assert_eq!(self.Type(), InlineItemType::kText);
        InlineItemSegment::UnpackSegmentData(
            self.start_offset_,
            self.end_offset_,
            self.segment_data_,
        )
    }
    pub fn EqualsRunSegment(&self, other: &Self) -> bool {
        self.segment_data_ == other.segment_data_
    }

    // cpp: layoutng/internal/inline_item.h:294-310
    pub fn SetBidiLevel(&mut self, level: u8) {
        if DirectionFromLevel(level as u32) != DirectionFromLevel(self.bidi_level_ as u32) {
            self.shape_result_ = Member::default();
        }
        self.bidi_level_ = level;
    }
    pub fn SetBidiLevelForItems(
        items: &mut InlineItems,
        index: u32,
        end_offset: u32,
        level: u8,
        num_out_of_flow: u32,
    ) -> u32 {
        unsafe { InlineItemSetBidiLevelForItems(items, index, end_offset, level, num_out_of_flow) }
    }
    pub fn SetBidiLevelForItemsDefault(
        items: &mut InlineItems,
        index: u32,
        end_offset: u32,
        level: u8,
    ) -> u32 {
        Self::SetBidiLevelForItems(items, index, end_offset, level, 0)
    }
    pub fn UpdateIndex(items: &mut [Member<InlineItem>]) {
        unsafe { InlineItemUpdateIndex(items) }
    }
    #[cfg(feature = "expensive_dchecks")]
    pub fn CheckIndex(items: &mut [Member<InlineItem>]) {
        unsafe { InlineItemCheckIndex(items) }
    }

    // cpp: layoutng/internal/inline_item.h:312-320
    pub fn AssertOffset(&self, offset: u32) {
        debug_assert!(self.IsValidOffset(offset));
    }
    pub fn AssertEndOffset(&self, offset: u32) {
        debug_assert!(offset >= self.start_offset_);
        debug_assert!(offset <= self.end_offset_);
    }
    pub fn ToString(&self) -> String {
        unsafe { InlineItemToString(self) }
    }
    pub fn Trace(&self, visitor: &mut Visitor) {
        unsafe { InlineItemTrace(self, visitor) }
    }
    pub(crate) fn ComputeBoxProperties(&mut self) {
        unsafe { InlineItemComputeBoxProperties(self) }
    }

    // cpp: layoutng/internal/inline_item.h:328-337
    pub fn IsValidOffset(&self, offset: u32) -> bool {
        (offset >= self.start_offset_ && offset < self.end_offset_)
            || (self.start_offset_ == self.end_offset_ && offset == self.start_offset_)
    }
}
