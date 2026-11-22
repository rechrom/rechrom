#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};

use font_engine::{Font, ShapeResultSpacing};
use foundation::{
    HeapVector, LayoutUnit, Member, String, TextDirection, TextOffsetMap, Vector, Visitor,
    WritingMode, UnsupportedLayout,
};
use crate::break_token::BreakToken;
use crate::inline_items_data::InlineItemsData;
use crate::layout_result::LayoutResult;
use layout_style::style::computed_style::ComputedStyle;

use super::super::algorithm_forward::{InlineChildLayoutContext, OffsetMapping, TextDiffRange};
use super::super::column_spanner_path::ColumnSpannerPath;
use super::super::constraint_space::ConstraintSpace;
use super::super::inline_item::{InlineItem, InlineItems};
use super::super::inline_node_data::InlineNodeData;
use super::super::layout_block_flow::LayoutBlockFlow;
use super::super::layout_input::TextDirection as InputTextDirection;
use super::super::layout_input_node::{LayoutInputNode, MinMaxSizesFloatInput};
use super::super::layout_pass_scope::LayoutPassScope;
use super::super::layout_text::LayoutText;
use super::super::min_max_sizes::MinMaxSizesResult;
use super::super::svg_character_data::SvgCharacterData;
use super::super::svg_inline_node_data::{SvgTextChunkOffsets, SvgTextContentRange};

// R5 split of the snapshot's layoutng_inline/inline_node.cc members:
// - The one-line InlineNodeData readers (Chromium declares them inline in
//   core/layout/inline/inline_node.h, so every includer, core/layout/*.cc and
//   SVG text included, calls them directly) are defined below with the
//   declaration base owns.
// - GetOffsetMapping is called by base editing, selection and OffsetMapping
//   code, also outside a layout pass, so it goes through the
//   `InlineLayoutSupport::get_offset_mapping` slot (resident support).
// - Everything else (Layout, ComputeMinMaxSizes, PrepareLayout, CollectInlines,
//   segmentation, shaping, ...) is inline-algorithm code that only the inline
//   package and block's inline extension call:
//   `layout_algo_inline::inline_node::InlineNodeAlgorithms`.

// cpp: layoutng/internal/inline_node.h:39-202
#[repr(C)]
#[derive(Clone)]
pub struct InlineNode {
    pub base: LayoutInputNode,
}

// cpp: layoutng/internal/inline_node.h:205-211
impl foundation::DowncastFrom<LayoutInputNode> for InlineNode {
    fn AllowFrom(node: &LayoutInputNode) -> bool {
        node.IsInline()
    }
}

const _: () = assert!(std::mem::offset_of!(InlineNode, base) == 0);

// cpp: layoutng/internal/inline_node.h:205-208
// Moving a successful DynamicTo<InlineNode> result keeps the existing input
// node; it must not run InlineNode's side-effecting constructor again.
impl From<LayoutInputNode> for InlineNode {
    fn from(node: LayoutInputNode) -> Self {
        assert!(node.IsInline());
        Self { base: node }
    }
}

// cpp: layoutng/internal/inline_node.h:39-42
impl From<InlineNode> for LayoutInputNode {
    fn from(node: InlineNode) -> Self {
        node.base
    }
}

impl InlineNode {
    pub fn AsLayoutInputNode(&self) -> &LayoutInputNode {
        &self.base
    }
}

// cpp: layoutng/internal/inline_node.h:141-152
pub struct FloatingObject {
    pub float_style: Member<ComputedStyle>,
    pub style: Member<ComputedStyle>,
    pub float_inline_max_size_with_margin: LayoutUnit,
}

impl FloatingObject {
    // cpp: layoutng/internal/inline_node.h:144-147
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.float_style);
        visitor.Trace(&self.style);
    }
}

impl PartialEq for FloatingObject {
    // cpp: layoutng/internal/inline_node.h:212-213
    fn eq(&self, other: &Self) -> bool {
        self.float_style.Get() == other.float_style.Get()
            && self.style.Get() == other.style.Get()
            && self.float_inline_max_size_with_margin == other.float_inline_max_size_with_margin
    }
}

impl InlineNode {
    // cpp: layoutng/internal/inline_node.h:44-44
    pub fn null() -> Self {
        Self {
            base: LayoutInputNode::null(),
        }
    }
    // cpp: layoutng/internal/inline_node.h:205-208
    pub fn AllowFrom(node: &LayoutInputNode) -> bool {
        node.IsInline()
    }
}

impl InlineNode {
    // cpp: layoutng_inline/inline_node.cc:85-87 (inline_node.h accessor)
    pub fn ItemsData(&self, first_line: bool) -> &InlineItemsData {
        self.Data().ItemsData(first_line)
    }
    // cpp: layoutng_inline/inline_node.cc:89-95
    pub fn FirstLineOffsetMap(&self) -> &Option<TextOffsetMap> {
        static EMPTY: Option<TextOffsetMap> = None;
        if self.Data().HasFirstLineItems() {
            return self.Data().ItemsData(true).OffsetMap();
        }
        &EMPTY
    }
    // cpp: layoutng_inline/inline_node.cc:2400-2404
    pub fn UseFirstLineStyleItemsData(&self) -> bool {
        let box_ptr = self.GetLayoutBox();
        !box_ptr.is_null()
            && unsafe { &*box_ptr }.HasFirstLineStylesForLayout()
            && self.Data().HasFirstLineItems()
    }

    // cpp: layoutng_inline/inline_node.cc:1128-1145 (body in layout_algo_inline)
    // Reached through the resident inline support: editing and selection
    // query offset mappings after layout, outside a layout pass.
    pub fn GetOffsetMapping(block: *mut LayoutBlockFlow) -> *const OffsetMapping {
        let callback = LayoutPassScope::ResidentInlineSupport()
            .get_offset_mapping
            .unwrap_or_else(|| {
                std::panic::panic_any(UnsupportedLayout::new(
                    "inline layout module is not installed",
                ))
            });
        callback(block)
    }

    // cpp: layoutng_inline/inline_node.cc:97-119 (inline_node.h accessors)
    pub fn IsBidiEnabled(&self) -> bool {
        self.Data().IsBidiEnabled()
    }
    pub fn BaseDirection(&self) -> TextDirection {
        match self.Data().BaseDirection() {
            InputTextDirection::kLtr => TextDirection::kLtr,
            InputTextDirection::kRtl => TextDirection::kRtl,
        }
    }
    pub fn HasFloats(&self) -> bool {
        self.Data().HasFloats()
    }
    pub fn HasInitialLetterBox(&self) -> bool {
        self.Data().HasInitialLetterBox()
    }
    pub fn HasRuby(&self) -> bool {
        self.Data().HasRuby()
    }
    pub fn HasTextEmphasis(&self) -> bool {
        self.Data().HasTextEmphasis()
    }
    pub fn IsBisectLineBreakDisabled(&self) -> bool {
        self.Data().IsBisectLineBreakDisabled()
    }
    pub fn IsScoreLineBreakDisabled(&self) -> bool {
        self.Data().IsScoreLineBreakDisabled()
    }
    pub fn CanContainFirstFormattedLine(&self) -> bool {
        let block = self.GetLayoutBlockFlow();
        debug_assert!(!block.is_null());
        unsafe { &*block }.CanContainFirstFormattedLine()
    }

    // cpp: layoutng_inline/inline_node.cc:2417-2433
    pub fn SvgCharacterDataList(&self) -> &Vector<(u32, SvgCharacterData)> {
        debug_assert!(self.IsSvgText());
        &unsafe { &*self.Data().svg_node_data_.Get() }.character_data_list
    }
    pub fn SvgTextLengthRangeList(&self) -> &HeapVector<SvgTextContentRange> {
        debug_assert!(self.IsSvgText());
        &unsafe { &*self.Data().svg_node_data_.Get() }.text_length_range_list
    }
    pub fn SvgTextPathRangeList(&self) -> &HeapVector<SvgTextContentRange> {
        debug_assert!(self.IsSvgText());
        &unsafe { &*self.Data().svg_node_data_.Get() }.text_path_range_list
    }

    // cpp: layoutng_inline/inline_node.cc:658-660
    pub fn IsPrepareLayoutFinished(&self) -> bool {
        let data = unsafe { &*self.GetLayoutBlockFlow() }.GetInlineNodeData();
        !data.is_null() && !unsafe { &*data }.text_content.IsNull()
    }

    // cpp: layoutng_inline/inline_node.cc:121-130 (inline_node.h accessors)
    pub fn MutableData(&self) -> *mut InlineNodeData {
        unsafe { &*self.GetLayoutBlockFlow() }.GetInlineNodeData()
    }
    pub fn Data(&self) -> &InlineNodeData {
        debug_assert!(self.IsPrepareLayoutFinished());
        let block = self.GetLayoutBlockFlow();
        debug_assert!(!unsafe { &*block }.NeedsCollectInlines());
        unsafe { &*(&*block).GetInlineNodeData() }
    }
}

impl Deref for InlineNode {
    type Target = LayoutInputNode;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl DerefMut for InlineNode {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
