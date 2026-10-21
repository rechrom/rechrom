#![allow(non_snake_case)]

use super::algorithm_forward::InlineItemResultRubyColumn;
use font_engine::ShapeResultView;
use foundation::{HeapVector, LayoutUnit, Member, String, Visitor};
use layoutng_fragment_tree::layout_result::LayoutResult;
use layoutng_geometry::geometry::box_strut::LineBoxStrut;

use super::exclusions::exclusion_space::ExclusionSpace;
use super::hyphen_result::HyphenResult;
use super::inline_item::{InlineItem, InlineItemType};
use super::inline_item_text_index::InlineItemTextIndex;
use super::positioned_float::PositionedFloat;
use super::text_fit_scale::{TextFitBlockScale, TextFitScale};
use super::text_offset_range::TextOffsetRange;

// The non-inline definitions live in //src/layoutng_inline. These signatures
// retain that package boundary without copying the line-break algorithm here.
unsafe extern "Rust" {
    fn InlineItemResultNew(
        item: &InlineItem,
        index: u32,
        text_offset: &TextOffsetRange,
        break_anywhere_if_overflow: bool,
        should_create_line_box: bool,
        has_unpositioned_floats: bool,
    ) -> InlineItemResult;
    fn InlineItemResultShapeHyphen(result: &mut InlineItemResult);
    fn InlineItemResultTrace(result: &InlineItemResult, visitor: &mut Visitor);
    #[cfg(debug_assertions)]
    fn InlineItemResultCheckConsistency(result: &InlineItemResult, allow_null_shape_result: bool);
    fn InlineItemResultToString(
        result: &InlineItemResult,
        ifc_text_content: &String,
        indent: &String,
    ) -> String;
    fn FindTextScaleInternalFromInline(
        line_items: &InlineItemResults,
        start_index: u32,
        initial_nesting_level: u32,
    ) -> TextFitBlockScale;
}

// cpp: layoutng/internal/inline_item_result.h:42-70
#[derive(Clone)]
pub struct OptionalPositionedFloat {
    has_value_: bool,
    value_: PositionedFloat,
}

impl Default for OptionalPositionedFloat {
    fn default() -> Self {
        Self {
            has_value_: false,
            value_: PositionedFloat::default(),
        }
    }
}

impl OptionalPositionedFloat {
    // cpp: layoutng/internal/inline_item_result.h:48-53
    pub fn Assign(&mut self, value: PositionedFloat) -> &mut Self {
        self.has_value_ = true;
        self.value_ = value;
        self
    }

    // cpp: layoutng/internal/inline_item_result.h:54-61
    pub fn HasValue(&self) -> bool {
        self.has_value_
    }

    pub fn Get(&self) -> *const PositionedFloat {
        debug_assert!(self.has_value_);
        &self.value_
    }

    pub fn GetMut(&mut self) -> *mut PositionedFloat {
        debug_assert!(self.has_value_);
        &mut self.value_
    }

    // cpp: layoutng/internal/inline_item_result.h:63-63
    pub fn Trace(&self, visitor: &mut Visitor) {
        self.value_.Trace(visitor);
    }
}

// cpp: layoutng/internal/inline_item_result.h:37-201
#[derive(Clone)]
pub struct InlineItemResult {
    pub item: Member<InlineItem>,
    pub item_index: u32,
    pub text_offset: TextOffsetRange,
    pub inline_size: LayoutUnit,
    pub spacing_before: LayoutUnit,
    pub pending_end_overhang: LayoutUnit,
    pub shape_result: Member<ShapeResultView>,
    pub hyphen: HyphenResult,
    pub layout_result: Member<LayoutResult>,
    pub ruby_column: Member<InlineItemResultRubyColumn>,
    pub positioned_float: OptionalPositionedFloat,
    pub exclusion_space_before_position_float: ExclusionSpace,
    pub margins: LineBoxStrut,
    pub borders: LineBoxStrut,
    pub padding: LineBoxStrut,
    pub text_fit_scale: Member<TextFitScale>,
    pub may_break_inside: bool,
    pub can_break_after: bool,
    pub has_only_pre_wrap_trailing_spaces: bool,
    pub has_only_bidi_trailing_spaces: bool,
    pub break_anywhere_if_overflow: bool,
    pub should_create_line_box: bool,
    pub has_unpositioned_floats: bool,
    pub is_hyphenated: bool,
}

impl Default for InlineItemResult {
    // cpp: layoutng/internal/inline_item_result.h:72-72
    fn default() -> Self {
        Self {
            item: Member::default(),
            item_index: 0,
            text_offset: TextOffsetRange::default(),
            inline_size: LayoutUnit::default(),
            spacing_before: LayoutUnit::default(),
            pending_end_overhang: LayoutUnit::default(),
            shape_result: Member::default(),
            hyphen: HyphenResult::default(),
            layout_result: Member::default(),
            ruby_column: Member::default(),
            positioned_float: OptionalPositionedFloat::default(),
            exclusion_space_before_position_float: ExclusionSpace::default(),
            margins: LineBoxStrut::default(),
            borders: LineBoxStrut::default(),
            padding: LineBoxStrut::default(),
            text_fit_scale: Member::default(),
            may_break_inside: false,
            can_break_after: false,
            has_only_pre_wrap_trailing_spaces: false,
            has_only_bidi_trailing_spaces: false,
            break_anywhere_if_overflow: false,
            should_create_line_box: false,
            has_unpositioned_floats: false,
            is_hyphenated: false,
        }
    }
}

impl InlineItemResult {
    // cpp: layoutng/internal/inline_item_result.h:73-78
    pub fn new(
        item: &InlineItem,
        index: u32,
        text_offset: &TextOffsetRange,
        break_anywhere_if_overflow: bool,
        should_create_line_box: bool,
        has_unpositioned_floats: bool,
    ) -> Self {
        unsafe {
            InlineItemResultNew(
                item,
                index,
                text_offset,
                break_anywhere_if_overflow,
                should_create_line_box,
                has_unpositioned_floats,
            )
        }
    }

    // cpp: layoutng/internal/inline_item_result.h:80-87
    pub fn TextOffset(&self) -> &TextOffsetRange {
        &self.text_offset
    }
    pub fn StartOffset(&self) -> u32 {
        self.text_offset.start
    }
    pub fn EndOffset(&self) -> u32 {
        self.text_offset.end
    }
    pub fn Length(&self) -> u32 {
        self.text_offset.Length()
    }

    pub fn Start(&self) -> InlineItemTextIndex {
        InlineItemTextIndex {
            item_index: self.item_index,
            text_offset: self.StartOffset(),
        }
    }

    pub fn End(&self) -> InlineItemTextIndex {
        InlineItemTextIndex {
            item_index: self.item_index,
            text_offset: self.EndOffset(),
        }
    }

    // cpp: layoutng/internal/inline_item_result.h:89-96
    pub fn IsEmptyText(&self) -> bool {
        self.Length() == 0 && unsafe { &*self.item.Get() }.Type() == InlineItemType::kText
    }

    pub fn IsRubyColumn(&self) -> bool {
        !self.ruby_column.Get().is_null()
    }

    // cpp: layoutng/internal/inline_item_result.h:99-110
    pub fn ShapeHyphen(&mut self) {
        unsafe { InlineItemResultShapeHyphen(self) }
    }
    pub fn Trace(&self, visitor: &mut Visitor) {
        unsafe { InlineItemResultTrace(self, visitor) }
    }

    #[cfg(debug_assertions)]
    pub fn CheckConsistency(&self, allow_null_shape_result: bool) {
        unsafe { InlineItemResultCheckConsistency(self, allow_null_shape_result) }
    }

    #[cfg(debug_assertions)]
    pub fn CheckConsistencyDefault(&self) {
        self.CheckConsistency(false)
    }

    pub fn ToString(&self, ifc_text_content: &String, indent: &String) -> String {
        unsafe { InlineItemResultToString(self, ifc_text_content, indent) }
    }

    pub fn ToStringDefaultIndent(&self, ifc_text_content: &String) -> String {
        self.ToString(ifc_text_content, &String::from(""))
    }
}

// The source's 32-item inline capacity is a storage hint. HeapVector retains
// the traced-element and sequence semantics without fixing that capacity.
// cpp: layoutng/internal/inline_item_result.h:204-204
pub type InlineItemResults = HeapVector<InlineItemResult>;

// cpp: layoutng/internal/inline_item_result.h:215-217
pub fn FindTextScaleInternal(
    line_items: &InlineItemResults,
    start_index: u32,
    initial_nesting_level: u32,
) -> TextFitBlockScale {
    unsafe { FindTextScaleInternalFromInline(line_items, start_index, initial_nesting_level) }
}

// cpp: layoutng/internal/inline_item_result.h:226-236
pub fn FindTextScale(
    should_scale: bool,
    line_items: &InlineItemResults,
    start_index: u32,
    initial_nesting_level: u32,
) -> TextFitBlockScale {
    if !should_scale {
        return TextFitBlockScale::default();
    }
    FindTextScaleInternal(line_items, start_index, initial_nesting_level)
}
