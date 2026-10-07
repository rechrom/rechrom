// C++: layoutng_inline/inline_item_result_ruby_column.h.
#![allow(non_snake_case)]

use foundation::{HeapVector, LayoutUnit, Member, RubyPosition, Traceable, Visitor};
use layoutng_fragment_tree::inline_break_token::RubyBreakTokenData;

use crate::line_info::LineInfo;

// cpp: layoutng_inline/inline_item_result_ruby_column.h:12-49
pub struct InlineItemResultRubyColumn {
    pub base_line: LineInfo,
    pub annotation_line_list: HeapVector<LineInfo, 1>,
    pub position_list: Vec<RubyPosition>,
    pub is_continuation: bool,
    pub start_ruby_break_token: Member<RubyBreakTokenData>,
    pub end_ruby_break_token: Member<RubyBreakTokenData>,
    pub last_base_glyph_spacing: LayoutUnit,
    pub end_overhang: LayoutUnit,
}

impl Default for InlineItemResultRubyColumn {
    fn default() -> Self {
        Self {
            base_line: LineInfo::default(),
            annotation_line_list: HeapVector::default(),
            position_list: Vec::new(),
            is_continuation: false,
            start_ruby_break_token: Member::default(),
            end_ruby_break_token: Member::default(),
            last_base_glyph_spacing: LayoutUnit::default(),
            end_overhang: LayoutUnit::default(),
        }
    }
}

impl InlineItemResultRubyColumn {
    // cpp: layoutng_inline/inline_item_result_ruby_column.h:14-19
    pub fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.base_line);
        visitor.Trace(&self.annotation_line_list);
        visitor.Trace(&self.start_ruby_break_token);
        visitor.Trace(&self.end_ruby_break_token);
    }
}

impl Traceable for InlineItemResultRubyColumn {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        InlineItemResultRubyColumn::Trace(self, visitor);
    }
}
