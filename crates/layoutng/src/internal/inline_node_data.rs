#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};

use foundation::{Member, Visitor};
use layoutng_fragment_tree::inline_items_data::{
    InlineItemsData, InlineItemsDataType, TraceInlineNodeDataAfterDispatch,
};

use super::layout_input::TextDirection;
use super::svg_inline_node_data::SvgInlineNodeData;

// C++ uses one-bit fields to pack these flags beside the base data type.
// Rust keeps named booleans so clients can set the same state without relying
// on the implementation-defined C++ bitfield layout.
// cpp: layoutng/internal/inline_node_data.h:18-19
// cpp: layoutng/internal/inline_node_data.h:74-120
#[repr(C)]
pub struct InlineNodeData {
    pub base_: InlineItemsData,
    pub has_non_orc_16bit_: bool,
    pub is_bidi_enabled_: bool,
    pub base_direction_: bool,
    pub has_floats_: bool,
    pub has_out_of_flow_positioned_: bool,
    pub has_initial_letter_box_: bool,
    pub has_ruby_: bool,
    pub has_text_emphasis_: bool,
    pub is_block_level_: bool,
    pub changes_may_affect_earlier_lines_: bool,
    pub is_bisect_line_break_disabled_: bool,
    pub is_score_line_break_disabled_: bool,
    pub first_line_items_: Member<InlineItemsData>,
    pub svg_node_data_: Member<SvgInlineNodeData>,
}

impl Default for InlineNodeData {
    // cpp: layoutng/internal/inline_node_data.h:20
    fn default() -> Self {
        Self {
            base_: InlineItemsData::new(InlineItemsDataType::kNodeData),
            has_non_orc_16bit_: false,
            is_bidi_enabled_: false,
            base_direction_: false,
            has_floats_: false,
            has_out_of_flow_positioned_: false,
            has_initial_letter_box_: false,
            has_ruby_: false,
            has_text_emphasis_: false,
            is_block_level_: false,
            changes_may_affect_earlier_lines_: false,
            is_bisect_line_break_disabled_: false,
            is_score_line_break_disabled_: false,
            first_line_items_: Member::default(),
            svg_node_data_: Member::default(),
        }
    }
}

impl Deref for InlineNodeData {
    type Target = InlineItemsData;

    fn deref(&self) -> &Self::Target {
        &self.base_
    }
}

impl DerefMut for InlineNodeData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base_
    }
}

#[allow(non_snake_case)]
impl InlineNodeData {
    // cpp: layoutng/internal/inline_node_data.h:22-23
    pub fn HasNonOrc16BitCharacters(&self) -> bool {
        self.has_non_orc_16bit_
    }

    // cpp: layoutng/internal/inline_node_data.h:24
    pub fn IsBidiEnabled(&self) -> bool {
        self.is_bidi_enabled_
    }

    // cpp: layoutng/internal/inline_node_data.h:25-27
    pub fn BaseDirection(&self) -> TextDirection {
        if self.base_direction_ {
            TextDirection::kRtl
        } else {
            TextDirection::kLtr
        }
    }

    // cpp: layoutng/internal/inline_node_data.h:28
    // Defined in //src/layoutng_inline/inline_items_data_algorithm.cc.
    pub fn DisableBidi(&mut self) {
        unsafe { InlineNodeDataDisableBidi(self) }
    }

    // cpp: layoutng/internal/inline_node_data.h:30
    pub fn HasFloats(&self) -> bool {
        self.has_floats_
    }

    // cpp: layoutng/internal/inline_node_data.h:31
    pub fn HasOutOfFlowPositioned(&self) -> bool {
        self.has_out_of_flow_positioned_
    }

    // cpp: layoutng/internal/inline_node_data.h:32-34
    pub fn HasFloatingOrOutOfFlowPositioned(&self) -> bool {
        self.HasFloats() || self.HasOutOfFlowPositioned()
    }

    // cpp: layoutng/internal/inline_node_data.h:35
    pub fn HasInitialLetterBox(&self) -> bool {
        self.has_initial_letter_box_
    }

    // cpp: layoutng/internal/inline_node_data.h:36
    pub fn HasRuby(&self) -> bool {
        self.has_ruby_
    }

    // cpp: layoutng/internal/inline_node_data.h:37
    pub fn HasTextEmphasis(&self) -> bool {
        self.has_text_emphasis_
    }

    // cpp: layoutng/internal/inline_node_data.h:39
    pub fn IsBlockLevel(&self) -> bool {
        self.is_block_level_
    }

    // cpp: layoutng/internal/inline_node_data.h:42-44
    pub fn IsBisectLineBreakDisabled(&self) -> bool {
        self.is_bisect_line_break_disabled_
    }

    // cpp: layoutng/internal/inline_node_data.h:48-50
    pub fn IsScoreLineBreakDisabled(&self) -> bool {
        self.is_score_line_break_disabled_
    }

    // cpp: layoutng/internal/inline_node_data.h:52
    pub fn HasFirstLineItems(&self) -> bool {
        !self.first_line_items_.Get().is_null()
    }

    // cpp: layoutng/internal/inline_node_data.h:54-57
    pub fn ItemsData(&self, is_first_line: bool) -> &InlineItemsData {
        if is_first_line && self.HasFirstLineItems() {
            unsafe { &*self.first_line_items_.Get() }
        } else {
            &self.base_
        }
    }

    // cpp: layoutng/internal/inline_node_data.h:59
    // cpp: layoutng_fragment_tree/inline_items_data.cc:29-33
    pub fn TraceAfterDispatch(&self, visitor: &mut Visitor) {
        TraceInlineNodeDataAfterDispatch(
            &self.base_,
            &self.first_line_items_,
            &self.svg_node_data_,
            visitor,
        );
    }

    // C++ keeps this private for its friend builder; a later crate owns the
    // body of DisableBidi and must be able to set the same direction bit.
    // cpp: layoutng/internal/inline_node_data.h:62-64
    pub fn SetBaseDirection(&mut self, direction: TextDirection) {
        self.base_direction_ = direction == TextDirection::kRtl;
    }

    // cpp: layoutng/internal/inline_node_data.h:124-129
    pub fn AllowFrom(value: &InlineItemsData) -> bool {
        value.IsNodeData()
    }
}

unsafe extern "Rust" {
    fn InlineNodeDataDisableBidi(data: &mut InlineNodeData);
}
