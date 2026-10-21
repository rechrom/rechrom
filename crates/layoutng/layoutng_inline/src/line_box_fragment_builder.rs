// C++: layoutng_inline/line_box_fragment_builder.h/.cc.
// PhysicalLineBoxFragment::Create is defined in the separate
// physical_line_box_fragment_builder.cc source file.
#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};

use font_engine::FontHeight;
use foundation::{
    IsLtr, LayoutUnit, MakeGarbageCollected, TextDirection, To, ToLineWritingMode,
    WritingDirectionMode,
};
use layoutng::internal::block_node::BlockNode;
use layoutng::internal::constraint_space::ConstraintSpace;
use layoutng::internal::exclusions::exclusion_space::ExclusionSpace;
use layoutng::internal::inline_node::InlineNode;
use layoutng::internal::layout_algorithm::LayoutAlgorithmCoreBuilder;
use layoutng::internal::layout_box::LayoutBox;
use layoutng::internal::layout_utils::{BlockStaticPositionEdge, InlineStaticPositionEdge};
use layoutng::internal::unpositioned_list_marker::UnpositionedListMarker;
use layoutng_fragment_tree::fragment_builder::FragmentBuilder;
use layoutng_fragment_tree::inline_break_token::InlineBreakToken;
use layoutng_fragment_tree::layout_result::LayoutResult;
use layoutng_fragment_tree::logical_line_container::LogicalLineContainer;
use layoutng_fragment_tree::logical_line_item::LogicalLineItems;
use layoutng_fragment_tree::physical_fragment::PhysicalFragment;
use layoutng_fragment_tree::physical_line_box_fragment::{LineBoxType, PhysicalLineBoxFragment};
use layoutng_geometry::geometry::logical_offset::LogicalOffset;
use layoutng_geometry::geometry::static_position::{BlockEdge, LogicalStaticPosition};
use layoutng_style::style::computed_style::ComputedStyle;

use crate::line_utils::{ComputeRelativeOffsetForInline, ComputeRelativeOffsetForOOFInInline};

// cpp: layoutng_inline/line_box_fragment_builder.h:23-123
// C++ public inheritance places FragmentBuilder first. The remaining fields
// are accessible to the source's friend implementations after shared assembly.
#[repr(C)]
pub struct LineBoxFragmentBuilder {
    pub(crate) base_: FragmentBuilder,
    pub(crate) line_box_bfc_block_offset_: Option<LayoutUnit>,
    pub(crate) clearance_after_line_: Option<LayoutUnit>,
    pub(crate) trim_block_end_by_: Option<LayoutUnit>,
    pub(crate) annotation_block_offset_adjustment_: LayoutUnit,
    pub(crate) metrics_: FontHeight,
    pub(crate) hang_inline_size_: LayoutUnit,
    pub(crate) line_box_type_: LineBoxType,
    pub(crate) base_direction_: TextDirection,
}

const _: () = assert!(std::mem::offset_of!(LineBoxFragmentBuilder, base_) == 0);

impl Deref for LineBoxFragmentBuilder {
    type Target = FragmentBuilder;
    fn deref(&self) -> &Self::Target {
        &self.base_
    }
}

impl DerefMut for LineBoxFragmentBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base_
    }
}

impl LayoutAlgorithmCoreBuilder<InlineBreakToken> for LineBoxFragmentBuilder {
    fn PreviousBreakToken(&self) -> *const InlineBreakToken {
        LineBoxFragmentBuilder::PreviousBreakToken(self)
    }

    fn GetExclusionSpace(&mut self) -> &mut ExclusionSpace {
        FragmentBuilder::GetExclusionSpace(&mut self.base_)
    }
}

// The PhysicalLineBoxFragment::Create definition belongs to
// physical_line_box_fragment_builder.cc, which is not yet translated.
unsafe extern "Rust" {
    fn PhysicalLineBoxFragmentCreateFromInline(
        builder: &mut LineBoxFragmentBuilder,
    ) -> *const PhysicalLineBoxFragment;
}

impl LineBoxFragmentBuilder {
    // cpp: layoutng_inline/line_box_fragment_builder.h:27-42
    pub fn new(
        node: InlineNode,
        style: *const ComputedStyle,
        space: &ConstraintSpace,
        writing_direction: WritingDirectionMode,
        break_token: *const InlineBreakToken,
    ) -> Self {
        Self {
            base_: FragmentBuilder::new(
                node.into(),
                style,
                space,
                WritingDirectionMode::new(writing_direction.GetWritingMode(), TextDirection::kLtr),
                break_token.cast(),
            ),
            line_box_bfc_block_offset_: None,
            clearance_after_line_: None,
            trim_block_end_by_: None,
            annotation_block_offset_adjustment_: LayoutUnit::default(),
            metrics_: FontHeight::Empty(),
            hang_inline_size_: LayoutUnit::default(),
            line_box_type_: LineBoxType::kNormalLineBox,
            base_direction_: TextDirection::kLtr,
        }
    }

    // cpp: layoutng_inline/line_box_fragment_builder.h:44
    // cpp: layoutng_inline/line_box_fragment_builder.cc:91-110
    pub fn Reset(&mut self) {
        self.base_.children_.Shrink(0);
        self.base_.child_break_tokens_.Shrink(0);
        self.base_.last_inline_break_token_ = std::ptr::null();
        self.base_.oof_positioned_candidates_.Shrink(0);
        self.base_.unpositioned_list_marker_ = UnpositionedListMarker::default();

        self.base_.annotation_overflow_ = LayoutUnit::default();
        self.base_.bfc_block_offset_ = None;
        self.line_box_bfc_block_offset_ = None;
        self.base_.is_pushed_by_floats_ = false;
        self.base_.subtree_modified_margin_strut_ = false;

        self.base_.size_.inline_size = LayoutUnit::default();
        self.metrics_ = FontHeight::Empty();
        self.line_box_type_ = LineBoxType::kNormalLineBox;

        self.base_.has_floating_descendants_for_paint_ = false;
        self.base_
            .has_descendant_that_depends_on_percentage_block_size_ = false;
    }

    // cpp: layoutng_inline/line_box_fragment_builder.h:46-48
    pub fn PreviousBreakToken(&self) -> *const InlineBreakToken {
        self.base_.previous_break_token_.cast()
    }

    // cpp: layoutng_inline/line_box_fragment_builder.h:50-52
    pub fn LineHeight(&self) -> LayoutUnit {
        self.metrics_.LineHeight().ClampNegativeToZero()
    }

    // cpp: layoutng_inline/line_box_fragment_builder.h:54-60
    pub fn SetInlineSize(&mut self, inline_size: LayoutUnit) {
        self.base_.size_.inline_size = inline_size;
    }
    pub fn SetHangInlineSize(&mut self, hang_inline_size: LayoutUnit) {
        self.hang_inline_size_ = hang_inline_size;
    }

    // cpp: layoutng_inline/line_box_fragment_builder.h:62-63
    // cpp: layoutng_inline/line_box_fragment_builder.cc:112-114
    pub fn SetIsEmptyLineBox(&mut self) {
        self.line_box_type_ = LineBoxType::kEmptyLineBox;
    }

    // cpp: layoutng_inline/line_box_fragment_builder.h:65-71
    pub fn LineBoxBfcBlockOffset(&self) -> Option<LayoutUnit> {
        self.line_box_bfc_block_offset_
    }
    pub fn SetLineBoxBfcBlockOffset(&mut self, offset: LayoutUnit) {
        debug_assert!(self.base_.bfc_block_offset_.is_some());
        self.line_box_bfc_block_offset_ = Some(offset);
    }

    // cpp: layoutng_inline/line_box_fragment_builder.h:73-82
    pub fn TrimBlockEndBy(&self) -> &Option<LayoutUnit> {
        &self.trim_block_end_by_
    }
    pub fn SetTrimBlockEndBy(&mut self, trim_block_end_by: LayoutUnit) {
        self.trim_block_end_by_ = Some(trim_block_end_by);
    }
    pub fn SetAnnotationBlockOffsetAdjustment(&mut self, adjustment: LayoutUnit) {
        self.annotation_block_offset_adjustment_ = adjustment;
    }

    // cpp: layoutng_inline/line_box_fragment_builder.h:84-89
    pub fn Metrics(&self) -> &FontHeight {
        &self.metrics_
    }
    pub fn SetMetrics(&mut self, metrics: &FontHeight) {
        self.metrics_ = *metrics;
    }
    pub fn SetBaseDirection(&mut self, direction: TextDirection) {
        self.base_direction_ = direction;
    }

    // cpp: layoutng_inline/line_box_fragment_builder.h:91-95
    pub fn SetBreakToken(&mut self, break_token: *const InlineBreakToken) {
        self.base_.break_token_ = break_token.cast();
    }

    // cpp: layoutng_inline/line_box_fragment_builder.h:97-110
    // cpp: layoutng_inline/line_box_fragment_builder.cc:116-130
    pub fn PropagateChildrenData(&mut self, container: &mut LogicalLineContainer) {
        self.PropagateChildrenDataFromLineItems(unsafe { &mut *container.BaseLine() });
        for annotation in container.AnnotationLineList() {
            let line_items = unsafe { &mut *annotation.get() };
            if !line_items.WasPropagated() {
                self.PropagateChildrenDataFromLineItems(line_items);
                line_items.SetPropagated();
            }
        }
        debug_assert!(!self.base_.HasOutOfFlowPositionedDescendants());
        self.base_.MoveOutOfFlowDescendantCandidatesToDescendants();
    }
    // cpp: layoutng_inline/line_box_fragment_builder.cc:132-162
    pub(crate) fn PropagateChildrenDataFromLineItems(&mut self, children: &mut LogicalLineItems) {
        let mut index = 0usize;
        while index < children.size() as usize {
            let child = &mut children[index];
            let layout_result = child.layout_result.Get();
            if !layout_result.is_null() {
                let fragment = child.GetPhysicalFragment();
                let child_style = unsafe { &*fragment }.Style();
                let mut child_offset = *child.Offset();
                child_offset -=
                    ComputeRelativeOffsetForInline(self.base_.GetConstraintSpace(), child_style);
                let relative_offset = ComputeRelativeOffsetForOOFInInline(
                    self.base_.GetConstraintSpace(),
                    child_style,
                );
                self.base_.PropagateFromLayoutResultAndFragment(
                    unsafe { &*layout_result },
                    child_offset,
                    relative_offset,
                    std::ptr::null(),
                );
                if child.children_count != 0 {
                    index += child.children_count as usize - 1;
                }
                index += 1;
                continue;
            }
            let out_of_flow = child.out_of_flow_positioned_box.Get();
            if !out_of_flow.is_null() {
                let node = BlockNode::new(To::<LayoutBox>(out_of_flow));
                let line_height = self.LineHeight();
                self.base_.AddOutOfFlowInlineChildCandidate(
                    node,
                    child.Offset(),
                    child.container_writing_direction,
                    line_height,
                );
                child.out_of_flow_positioned_box.Clear();
            }
            index += 1;
        }
    }
    // cpp: layoutng_inline/line_box_fragment_builder.cc:164-173
    pub fn ToLineBoxFragment(&mut self) -> *const LayoutResult {
        self.base_.Finalize();
        let writing_mode = ToLineWritingMode(self.base_.GetWritingMode());
        self.base_.writing_direction_.SetWritingMode(writing_mode);
        let fragment = unsafe { PhysicalLineBoxFragmentCreateFromInline(self) };
        MakeGarbageCollected(LayoutResultFromLineBoxFragmentBuilder(
            fragment.cast(),
            self,
        ))
    }

    // cpp: layoutng_inline/line_box_fragment_builder.h:102-104
    pub fn SetClearanceAfterLine(&mut self, clearance: LayoutUnit) {
        self.clearance_after_line_ = Some(clearance);
    }
}

// cpp: layoutng_inline/line_box_fragment_builder.cc:24-65
#[unsafe(no_mangle)]
pub extern "Rust" fn FragmentBuilderAddOutOfFlowInlineChildCandidate(
    builder: &mut FragmentBuilder,
    child: BlockNode,
    child_offset: &LogicalOffset,
    inline_container_writing_direction: WritingDirectionMode,
    line_box_block_size: LayoutUnit,
) {
    debug_assert!(builder.node_.IsInline() || unsafe { &*builder.layout_object_ }.IsLayoutInline());
    let mut static_position = LogicalStaticPosition::from_offset(*child_offset);
    static_position.inline_edge = InlineStaticPositionEdge(
        &child,
        std::ptr::null(),
        inline_container_writing_direction,
        !IsLtr(inline_container_writing_direction.Direction()),
    );
    static_position.block_edge =
        BlockStaticPositionEdge(&child, std::ptr::null(), inline_container_writing_direction);
    match static_position.block_edge {
        BlockEdge::kBlockCenter => {
            static_position.offset.block_offset += line_box_block_size / 2;
        }
        BlockEdge::kBlockEnd => {
            static_position.offset.block_offset += line_box_block_size;
        }
        BlockEdge::kBlockStart => {}
    }
    builder.AddOutOfFlowChildCandidateDefault(&child, &static_position);
}

// cpp: layoutng_inline/line_box_fragment_builder.cc:67-89
pub fn LayoutResultFromLineBoxFragmentBuilder(
    physical_fragment: *const PhysicalFragment,
    builder: &mut LineBoxFragmentBuilder,
) -> LayoutResult {
    let result = LayoutResult::from_fragment_builder(physical_fragment, &mut builder.base_);
    debug_assert_eq!(
        builder.base_.bfc_block_offset_.is_some(),
        builder.line_box_bfc_block_offset_.is_some()
    );
    if builder.base_.bfc_block_offset_ != builder.line_box_bfc_block_offset_ {
        result.EnsureRareData().SetLineBoxBfcBlockOffset(
            builder
                .line_box_bfc_block_offset_
                .expect("line BFC block offset"),
        );
    }
    if builder.annotation_block_offset_adjustment_ != LayoutUnit::default() {
        result.EnsureRareData().annotation_block_offset_adjustment_ =
            builder.annotation_block_offset_adjustment_;
    }
    if let Some(clearance) = builder.clearance_after_line_ {
        result.EnsureRareData().clearance_after_line_ = clearance;
    }
    if let Some(trim) = builder.trim_block_end_by_ {
        result.EnsureRareData().trim_block_end_by_ = trim;
    }
    result
}
