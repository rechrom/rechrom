#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};

use font_engine::{Font, ShapeResultSpacing};
use foundation::{
    HeapVector, LayoutUnit, Member, String, TextDirection, TextOffsetMap, Vector, Visitor,
    WritingMode,
};
use layoutng_fragment_tree::break_token::BreakToken;
use layoutng_fragment_tree::inline_items_data::InlineItemsData;
use layoutng_fragment_tree::layout_result::LayoutResult;
use layoutng_style::style::computed_style::ComputedStyle;

use super::algorithm_forward::{InlineChildLayoutContext, OffsetMapping, TextDiffRange};
use super::column_spanner_path::ColumnSpannerPath;
use super::constraint_space::ConstraintSpace;
use super::inline_item::{InlineItem, InlineItems};
use super::inline_node_data::InlineNodeData;
use super::layout_block_flow::LayoutBlockFlow;
use super::layout_input_node::{LayoutInputNode, MinMaxSizesFloatInput};
use super::layout_text::LayoutText;
use super::min_max_sizes::MinMaxSizesResult;
use super::svg_character_data::SvgCharacterData;
use super::svg_inline_node_data::{SvgTextChunkOffsets, SvgTextContentRange};

// Non-inline implementation is in //src/layoutng_inline/inline_node.cc.
unsafe extern "Rust" {
    fn InlineNodeLayout(
        this: &InlineNode,
        space: &ConstraintSpace,
        break_token: *const BreakToken,
        spanner: *const ColumnSpannerPath,
        context: *mut InlineChildLayoutContext,
    ) -> *const LayoutResult;
    fn InlineNodeComputeMinMaxSizes(
        this: &InlineNode,
        mode: WritingMode,
        space: &ConstraintSpace,
        floats: &MinMaxSizesFloatInput,
    ) -> MinMaxSizesResult;
    fn InlineNodeInvalidatePrepareLayoutForTest(this: &mut InlineNode);
    fn InlineNodeItemsData(this: &InlineNode, first_line: bool) -> *const InlineItemsData;
    fn InlineNodeFirstLineOffsetMap(this: &InlineNode) -> *const Option<TextOffsetMap>;
    fn InlineNodeUseFirstLineStyleItemsData(this: &InlineNode) -> bool;
    fn InlineNodeIsStickyImagesQuirkForContentSize(this: &InlineNode) -> bool;
    fn InlineNodeTextContentForStickyImagesQuirk(data: &InlineItemsData) -> String;
    fn InlineNodeSetTextWithOffset(
        text: *mut LayoutText,
        new_text: String,
        diff: &TextDiffRange,
    ) -> bool;
    fn InlineNodeComputeOffsetMappingIfNeeded(this: &InlineNode) -> *const OffsetMapping;
    fn InlineNodeGetOffsetMapping(block: *mut LayoutBlockFlow) -> *const OffsetMapping;
    fn InlineNodeIsBidiEnabled(this: &InlineNode) -> bool;
    fn InlineNodeBaseDirection(this: &InlineNode) -> TextDirection;
    fn InlineNodeHasFloats(this: &InlineNode) -> bool;
    fn InlineNodeHasInitialLetterBox(this: &InlineNode) -> bool;
    fn InlineNodeHasRuby(this: &InlineNode) -> bool;
    fn InlineNodeHasTextEmphasis(this: &InlineNode) -> bool;
    fn InlineNodeIsBisectLineBreakDisabled(this: &InlineNode) -> bool;
    fn InlineNodeIsScoreLineBreakDisabled(this: &InlineNode) -> bool;
    fn InlineNodeCanContainFirstFormattedLine(this: &InlineNode) -> bool;
    fn InlineNodeCheckConsistency(this: &InlineNode);
    fn InlineNodeSvgCharacterDataList(this: &InlineNode) -> *const Vector<(u32, SvgCharacterData)>;
    fn InlineNodeSvgTextLengthRangeList(
        this: &InlineNode,
    ) -> *const HeapVector<SvgTextContentRange>;
    fn InlineNodeSvgTextPathRangeList(this: &InlineNode) -> *const HeapVector<SvgTextContentRange>;
    fn InlineNodeFontForTab(this: &InlineNode) -> *const Font;
    fn InlineNodeMinimumFontPhysicalSize(this: &InlineNode) -> Option<f32>;
    fn InlineNodeNeedsShapingForTesting(item: &InlineItem) -> bool;
    fn InlineNodePrepareLayoutIfNeeded(this: &InlineNode);
    fn InlineNodeIsPrepareLayoutFinished(this: &InlineNode) -> bool;
    fn InlineNodePrepareLayout(this: &InlineNode, previous: *mut InlineNodeData);
    fn InlineNodeCollectInlines(
        this: &InlineNode,
        data: *mut InlineNodeData,
        previous: *mut InlineNodeData,
    );
    fn InlineNodeFindSvgTextChunks(
        this: &InlineNode,
        block: &mut LayoutBlockFlow,
        data: &mut InlineNodeData,
    ) -> *const SvgTextChunkOffsets;
    fn InlineNodeSegmentText(
        this: &InlineNode,
        data: *mut InlineNodeData,
        previous: *mut InlineNodeData,
    );
    fn InlineNodeSegmentScriptRuns(
        this: &InlineNode,
        data: *mut InlineNodeData,
        previous: *mut InlineNodeData,
    );
    fn InlineNodeSegmentFontOrientation(this: &InlineNode, data: *mut InlineNodeData);
    fn InlineNodeSegmentBidiRuns(this: &InlineNode, data: *mut InlineNodeData);
    fn InlineNodeShapeText(
        this: &InlineNode,
        data: *mut InlineItemsData,
        previous_text: *const String,
        previous_items: *const InlineItems,
        override_font: *const Font,
    );
    fn InlineNodeShapeTextForFirstLineIfNeeded(this: &InlineNode, data: *mut InlineNodeData);
    fn InlineNodeShapeTextIncludingFirstLine(
        this: &InlineNode,
        data: *mut InlineNodeData,
        previous_text: *const String,
        previous_items: *const InlineItems,
    );
    fn InlineNodeAssociateItemsWithInlines(this: &InlineNode, data: *mut InlineNodeData);
    fn InlineNodeIsNGShapeCacheAllowed(
        this: &InlineNode,
        text: &String,
        font: *const Font,
        items: &InlineItems,
        spacing: &mut ShapeResultSpacing,
    ) -> bool;
    fn InlineNodeMutableData(this: &InlineNode) -> *mut InlineNodeData;
    fn InlineNodeData(this: &InlineNode) -> *const InlineNodeData;
    fn InlineNodeMaybeDirtyData(this: &InlineNode) -> *const InlineNodeData;
    fn InlineNodeEnsureData(this: &InlineNode) -> *const InlineNodeData;
    fn InlineNodeAdjustFontForTextCombineUprightAll(this: &InlineNode);
    fn InlineNodeComputeOffsetMapping(block: *mut LayoutBlockFlow, data: *mut InlineNodeData);
}

// cpp: layoutng/internal/inline_node.h:39-202
#[repr(C)]
#[derive(Clone)]
pub struct InlineNode {
    pub(crate) base: LayoutInputNode,
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
    // cpp: layoutng/internal/inline_node.h:48-68
    pub fn Layout(
        &self,
        space: &ConstraintSpace,
        break_token: *const BreakToken,
        spanner: *const ColumnSpannerPath,
        context: *mut InlineChildLayoutContext,
    ) -> *const LayoutResult {
        unsafe { InlineNodeLayout(self, space, break_token, spanner, context) }
    }
    pub fn ComputeMinMaxSizes(
        &self,
        mode: WritingMode,
        space: &ConstraintSpace,
        floats: &MinMaxSizesFloatInput,
    ) -> MinMaxSizesResult {
        unsafe { InlineNodeComputeMinMaxSizes(self, mode, space, floats) }
    }
    pub fn InvalidatePrepareLayoutForTest(&mut self) {
        unsafe { InlineNodeInvalidatePrepareLayoutForTest(self) }
    }
    pub fn ItemsData(&self, first_line: bool) -> &InlineItemsData {
        unsafe { &*InlineNodeItemsData(self, first_line) }
    }
    pub fn FirstLineOffsetMap(&self) -> &Option<TextOffsetMap> {
        unsafe { &*InlineNodeFirstLineOffsetMap(self) }
    }
    pub fn UseFirstLineStyleItemsData(&self) -> bool {
        unsafe { InlineNodeUseFirstLineStyleItemsData(self) }
    }

    // cpp: layoutng/internal/inline_node.h:70-103
    pub fn IsStickyImagesQuirkForContentSize(&self) -> bool {
        unsafe { InlineNodeIsStickyImagesQuirkForContentSize(self) }
    }
    pub fn TextContentForStickyImagesQuirk(data: &InlineItemsData) -> String {
        unsafe { InlineNodeTextContentForStickyImagesQuirk(data) }
    }
    pub fn SetTextWithOffset(
        text: *mut LayoutText,
        new_text: String,
        diff: &TextDiffRange,
    ) -> bool {
        unsafe { InlineNodeSetTextWithOffset(text, new_text, diff) }
    }
    pub fn ComputeOffsetMappingIfNeeded(&self) -> *const OffsetMapping {
        unsafe { InlineNodeComputeOffsetMappingIfNeeded(self) }
    }
    pub fn GetOffsetMapping(block: *mut LayoutBlockFlow) -> *const OffsetMapping {
        unsafe { InlineNodeGetOffsetMapping(block) }
    }
    pub fn IsBidiEnabled(&self) -> bool {
        unsafe { InlineNodeIsBidiEnabled(self) }
    }
    pub fn BaseDirection(&self) -> TextDirection {
        unsafe { InlineNodeBaseDirection(self) }
    }

    // cpp: layoutng/internal/inline_node.h:105-139
    pub fn HasFloats(&self) -> bool {
        unsafe { InlineNodeHasFloats(self) }
    }
    pub fn HasInitialLetterBox(&self) -> bool {
        unsafe { InlineNodeHasInitialLetterBox(self) }
    }
    pub fn HasRuby(&self) -> bool {
        unsafe { InlineNodeHasRuby(self) }
    }
    pub fn HasTextEmphasis(&self) -> bool {
        unsafe { InlineNodeHasTextEmphasis(self) }
    }
    pub fn IsBisectLineBreakDisabled(&self) -> bool {
        unsafe { InlineNodeIsBisectLineBreakDisabled(self) }
    }
    pub fn IsScoreLineBreakDisabled(&self) -> bool {
        unsafe { InlineNodeIsScoreLineBreakDisabled(self) }
    }
    pub fn CanContainFirstFormattedLine(&self) -> bool {
        unsafe { InlineNodeCanContainFirstFormattedLine(self) }
    }
    pub fn CheckConsistency(&self) {
        unsafe { InlineNodeCheckConsistency(self) }
    }
    pub fn SvgCharacterDataList(&self) -> &Vector<(u32, SvgCharacterData)> {
        unsafe { &*InlineNodeSvgCharacterDataList(self) }
    }
    pub fn SvgTextLengthRangeList(&self) -> &HeapVector<SvgTextContentRange> {
        unsafe { &*InlineNodeSvgTextLengthRangeList(self) }
    }
    pub fn SvgTextPathRangeList(&self) -> &HeapVector<SvgTextContentRange> {
        unsafe { &*InlineNodeSvgTextPathRangeList(self) }
    }
    pub fn FontForTab(&self) -> &Font {
        unsafe { &*InlineNodeFontForTab(self) }
    }
    pub fn MinimumFontPhysicalSize(&self) -> Option<f32> {
        unsafe { InlineNodeMinimumFontPhysicalSize(self) }
    }
    pub fn NeedsShapingForTesting(item: &InlineItem) -> bool {
        unsafe { InlineNodeNeedsShapingForTesting(item) }
    }
    pub fn PrepareLayoutIfNeeded(&self) {
        unsafe { InlineNodePrepareLayoutIfNeeded(self) }
    }
}

impl InlineNode {
    // cpp: layoutng/internal/inline_node.h:160-183
    pub fn IsPrepareLayoutFinished(&self) -> bool {
        unsafe { InlineNodeIsPrepareLayoutFinished(self) }
    }
    pub fn PrepareLayout(&self, previous: *mut InlineNodeData) {
        unsafe { InlineNodePrepareLayout(self, previous) }
    }
    pub fn CollectInlines(&self, data: *mut InlineNodeData, previous: *mut InlineNodeData) {
        unsafe { InlineNodeCollectInlines(self, data, previous) }
    }
    pub fn CollectInlinesDefault(&self, data: *mut InlineNodeData) {
        self.CollectInlines(data, std::ptr::null_mut())
    }
    pub fn FindSvgTextChunks(
        &self,
        block: &mut LayoutBlockFlow,
        data: &mut InlineNodeData,
    ) -> *const SvgTextChunkOffsets {
        unsafe { InlineNodeFindSvgTextChunks(self, block, data) }
    }
    pub fn SegmentText(&self, data: *mut InlineNodeData, previous: *mut InlineNodeData) {
        unsafe { InlineNodeSegmentText(self, data, previous) }
    }
    pub fn SegmentScriptRuns(&self, data: *mut InlineNodeData, previous: *mut InlineNodeData) {
        unsafe { InlineNodeSegmentScriptRuns(self, data, previous) }
    }
    pub fn SegmentFontOrientation(&self, data: *mut InlineNodeData) {
        unsafe { InlineNodeSegmentFontOrientation(self, data) }
    }
    pub fn SegmentBidiRuns(&self, data: *mut InlineNodeData) {
        unsafe { InlineNodeSegmentBidiRuns(self, data) }
    }
    pub fn ShapeText(
        &self,
        data: *mut InlineItemsData,
        previous_text: *const String,
        previous_items: *const InlineItems,
        override_font: *const Font,
    ) {
        unsafe { InlineNodeShapeText(self, data, previous_text, previous_items, override_font) }
    }
    pub fn ShapeTextDefault(&self, data: *mut InlineItemsData) {
        self.ShapeText(data, std::ptr::null(), std::ptr::null(), std::ptr::null())
    }
    pub fn ShapeTextForFirstLineIfNeeded(&self, data: *mut InlineNodeData) {
        unsafe { InlineNodeShapeTextForFirstLineIfNeeded(self, data) }
    }
    pub fn ShapeTextIncludingFirstLine(
        &self,
        data: *mut InlineNodeData,
        previous_text: *const String,
        previous_items: *const InlineItems,
    ) {
        unsafe { InlineNodeShapeTextIncludingFirstLine(self, data, previous_text, previous_items) }
    }
    pub fn AssociateItemsWithInlines(&self, data: *mut InlineNodeData) {
        unsafe { InlineNodeAssociateItemsWithInlines(self, data) }
    }

    // cpp: layoutng/internal/inline_node.h:184-198
    pub fn IsNGShapeCacheAllowed(
        &self,
        text: &String,
        font: *const Font,
        items: &InlineItems,
        spacing: &mut ShapeResultSpacing,
    ) -> bool {
        unsafe { InlineNodeIsNGShapeCacheAllowed(self, text, font, items, spacing) }
    }
    pub fn MutableData(&self) -> *mut InlineNodeData {
        unsafe { InlineNodeMutableData(self) }
    }
    pub fn Data(&self) -> &InlineNodeData {
        unsafe { &*InlineNodeData(self) }
    }
    pub fn MaybeDirtyData(&self) -> &InlineNodeData {
        unsafe { &*InlineNodeMaybeDirtyData(self) }
    }
    pub fn EnsureData(&self) -> &InlineNodeData {
        unsafe { &*InlineNodeEnsureData(self) }
    }
    pub fn AdjustFontForTextCombineUprightAll(&self) {
        unsafe { InlineNodeAdjustFontForTextCombineUprightAll(self) }
    }
    pub fn ComputeOffsetMapping(block: *mut LayoutBlockFlow, data: *mut InlineNodeData) {
        unsafe { InlineNodeComputeOffsetMapping(block, data) }
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
