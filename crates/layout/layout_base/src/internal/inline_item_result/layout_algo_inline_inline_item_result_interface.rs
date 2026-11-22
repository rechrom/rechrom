#![allow(non_snake_case)]

use super::super::algorithm_forward::InlineItemResultRubyColumn;
use font_engine::ShapeResultView;
use foundation::{HeapVector, LayoutUnit, Member, Visitor};
use crate::layout_result::LayoutResult;
use layout_geometry::geometry::box_strut::LineBoxStrut;

use super::super::exclusions::exclusion_space::ExclusionSpace;
use super::super::hyphen_result::HyphenResult;
use super::super::inline_item::{InlineItem, InlineItemType};
use super::super::inline_item_text_index::InlineItemTextIndex;
use super::super::positioned_float::PositionedFloat;
use super::super::text_fit_scale::TextFitScale;
use super::super::text_offset_range::TextOffsetRange;

// R5: the out-of-line members (constructor, ShapeHyphen, CheckConsistency,
// ToString) and FindTextScale(Internal) are line-breaker
// code in layout_algo_inline (`inline_item_result::InlineItemResultAlgorithms`).

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

    // cpp: layoutng_inline/inline_item_result.cc:62-71
    // Record member: base's Traceable binding (layoutng_trace_bindings.rs)
    // calls it, so it stays with the declaration.
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.item);
        visitor.Trace(&self.shape_result);
        visitor.Trace(&self.hyphen);
        visitor.Trace(&self.layout_result);
        visitor.Trace(&self.ruby_column);
        visitor.Trace(&self.positioned_float);
        visitor.Trace(&self.exclusion_space_before_position_float);
        visitor.Trace(&self.text_fit_scale);
    }

}

// Preserve the source's 32-item inline capacity. Most lines fit without a
// backing allocation, which is particularly important during grid measuring.
// cpp: layoutng/internal/inline_item_result.h:204-204
pub type InlineItemResults = HeapVector<InlineItemResult, 32>;


