#![allow(non_snake_case)]

use std::ops::{AddAssign, Deref, DerefMut};

use foundation::{
    HeapHashMap, HeapVector, LayoutUnit, Member, PhysicalOffset, PhysicalSize,
    RuntimeEnabledFeatures, Traceable, Visitor, WritingDirectionMode,
};
use layoutng_fragment_tree::block_break_token::BlockBreakToken;
use layoutng_fragment_tree::physical_fragment::{OofData, PhysicalFragment};
use layoutng_geometry::geometry::logical_offset::LogicalOffset;
use layoutng_geometry::geometry::static_position::{LogicalStaticPosition, PhysicalStaticPosition};
use layoutng_geometry::geometry::writing_mode_converter::WritingModeConverter;

use super::block_node::BlockNode;
use super::layout_box::LayoutBox;
use super::layout_inline::LayoutInline;
use super::layout_object::LayoutObject;

// cpp: layoutng/internal/oof_positioned_node.h:30-81
#[derive(Clone)]
pub struct OofContainingBlock<OffsetType> {
    offset_: OffsetType,
    relative_offset_: OffsetType,
    fragment_: Member<PhysicalFragment>,
    clipped_container_block_offset_: LayoutUnit,
    is_inside_column_spanner_: bool,
}

impl<OffsetType: Default> Default for OofContainingBlock<OffsetType> {
    fn default() -> Self {
        Self {
            offset_: OffsetType::default(),
            relative_offset_: OffsetType::default(),
            fragment_: Member::default(),
            clipped_container_block_offset_: LayoutUnit::Min(),
            is_inside_column_spanner_: false,
        }
    }
}

impl<OffsetType: Copy> OofContainingBlock<OffsetType> {
    // cpp: layoutng/internal/oof_positioned_node.h:37-47
    pub fn new(
        offset: OffsetType,
        relative_offset: OffsetType,
        fragment: *const PhysicalFragment,
        clipped_container_block_offset: Option<LayoutUnit>,
        is_inside_column_spanner: bool,
    ) -> Self {
        Self {
            offset_: offset,
            relative_offset_: relative_offset,
            fragment_: Member::from_ptr(fragment as *mut PhysicalFragment),
            clipped_container_block_offset_: clipped_container_block_offset
                .unwrap_or(LayoutUnit::Min()),
            is_inside_column_spanner_: is_inside_column_spanner,
        }
    }

    // cpp: layoutng/internal/oof_positioned_node.h:49-68
    pub fn Offset(&self) -> OffsetType {
        self.offset_
    }

    pub fn RelativeOffset(&self) -> OffsetType {
        self.relative_offset_
    }

    pub fn Fragment(&self) -> *const PhysicalFragment {
        self.fragment_.Get()
    }

    pub fn ClippedContainerBlockOffset(&self) -> Option<LayoutUnit> {
        if self.clipped_container_block_offset_ == LayoutUnit::Min() {
            None
        } else {
            Some(self.clipped_container_block_offset_)
        }
    }

    pub fn IsInsideColumnSpanner(&self) -> bool {
        self.is_inside_column_spanner_
    }

    pub fn IsFragmentedInsideClippedContainer(&self) -> bool {
        self.clipped_container_block_offset_ != LayoutUnit::Min()
    }

    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.fragment_);
    }
}

impl OofContainingBlock<LogicalOffset> {
    // cpp: layoutng/internal/oof_positioned_node.h:50-56
    pub fn IncreaseBlockOffset(&mut self, block_offset: LayoutUnit) {
        self.offset_.block_offset += block_offset;
    }

    pub fn IncreaseInlineOffset(&mut self, inline_offset: LayoutUnit) {
        self.offset_.inline_offset += inline_offset;
    }
}

// cpp: layoutng/internal/oof_positioned_node.h:91-121
#[derive(Clone)]
pub struct OofInlineContainer<OffsetType> {
    container_: Member<LayoutInline>,
    relative_offset_: OffsetType,
}

impl<OffsetType: Default> Default for OofInlineContainer<OffsetType> {
    fn default() -> Self {
        Self {
            container_: Member::default(),
            relative_offset_: OffsetType::default(),
        }
    }
}

impl<OffsetType: Copy + Default + AddAssign> OofInlineContainer<OffsetType> {
    // cpp: layoutng/internal/oof_positioned_node.h:98-100
    pub fn new(container: *const LayoutInline, relative_offset: OffsetType) -> Self {
        Self {
            container_: Member::from_ptr(container as *mut LayoutInline),
            relative_offset_: relative_offset,
        }
    }

    // cpp: layoutng/internal/oof_positioned_node.h:102-115
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.container_);
    }

    pub fn Container(&self) -> *const LayoutInline {
        self.container_.Get()
    }

    pub fn RelativeOffset(&self) -> OffsetType {
        if RuntimeEnabledFeatures::FragmentedOofInCbEnabled() {
            OffsetType::default()
        } else {
            self.relative_offset_
        }
    }

    pub fn IncreaseRelativeOffset(&mut self, increase: OffsetType) {
        debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
        self.relative_offset_ += increase;
    }
}

// cpp: layoutng/internal/oof_positioned_node.h:129-155
#[derive(Clone)]
pub struct MulticolWithPendingOofs<OffsetType> {
    pub multicol_offset: OffsetType,
    pub fixedpos_containing_block: OofContainingBlock<OffsetType>,
    pub fixedpos_inline_container: OofInlineContainer<OffsetType>,
}

impl<OffsetType: Copy + Default + AddAssign> Default for MulticolWithPendingOofs<OffsetType> {
    fn default() -> Self {
        Self {
            multicol_offset: OffsetType::default(),
            fixedpos_containing_block: OofContainingBlock::default(),
            fixedpos_inline_container: OofInlineContainer::default(),
        }
    }
}

impl<OffsetType: Copy + Default + AddAssign> MulticolWithPendingOofs<OffsetType> {
    // cpp: layoutng/internal/oof_positioned_node.h:144-155
    pub fn new(
        multicol_offset: OffsetType,
        fixedpos_containing_block: OofContainingBlock<OffsetType>,
        fixedpos_inline_container: OofInlineContainer<OffsetType>,
    ) -> Self {
        debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
        Self {
            multicol_offset,
            fixedpos_containing_block,
            fixedpos_inline_container,
        }
    }

    // cpp: layoutng/internal/oof_positioned_node.h:157-161
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.fixedpos_containing_block);
        visitor.Trace(&self.fixedpos_inline_container);
    }
}

pub trait OofStaticPosition<OffsetType>: Copy {
    fn SetOffset(&mut self, offset: OffsetType);
    fn IncreaseOffset(&mut self, increase: OffsetType);
}

impl OofStaticPosition<PhysicalOffset> for PhysicalStaticPosition {
    fn SetOffset(&mut self, offset: PhysicalOffset) {
        self.offset = offset;
    }

    fn IncreaseOffset(&mut self, increase: PhysicalOffset) {
        self.offset += increase;
    }
}

impl OofStaticPosition<LogicalOffset> for LogicalStaticPosition {
    fn SetOffset(&mut self, offset: LogicalOffset) {
        self.offset = offset;
    }

    fn IncreaseOffset(&mut self, increase: LogicalOffset) {
        self.offset += increase;
    }
}

// The source instantiates this template for physical and logical coordinates.
// cpp: layoutng/internal/oof_positioned_node.h:175-248
#[repr(C)]
#[derive(Clone)]
pub struct OofPositionedNode<OffsetType, StaticPositionType> {
    box_: Member<LayoutBox>,
    break_token_: Member<BlockBreakToken>,
    static_position_: StaticPositionType,
    inline_container_: OofInlineContainer<OffsetType>,
    is_for_fragmentation_: bool,
    requires_content_before_breaking_: bool,
}

impl<OffsetType, StaticPositionType> OofPositionedNode<OffsetType, StaticPositionType>
where
    OffsetType: Copy + Default + AddAssign,
    StaticPositionType: OofStaticPosition<OffsetType>,
{
    // cpp: layoutng/internal/oof_positioned_node.h:180-193
    pub fn new(
        node: BlockNode,
        break_token: *const BlockBreakToken,
        static_position: StaticPositionType,
        requires_content_before_breaking: bool,
    ) -> Self {
        Self::new_with_inline(
            node,
            break_token,
            static_position,
            requires_content_before_breaking,
            OofInlineContainer::default(),
        )
    }

    pub fn new_with_inline(
        node: BlockNode,
        break_token: *const BlockBreakToken,
        static_position: StaticPositionType,
        requires_content_before_breaking: bool,
        inline_container: OofInlineContainer<OffsetType>,
    ) -> Self {
        debug_assert!(node.IsBlock());
        Self {
            box_: Member::from_ptr(node.GetLayoutBox()),
            break_token_: Member::from_ptr(break_token as *mut BlockBreakToken),
            static_position_: static_position,
            inline_container_: inline_container,
            is_for_fragmentation_: false,
            requires_content_before_breaking_: requires_content_before_breaking,
        }
    }

    // cpp: layoutng/internal/oof_positioned_node.h:196-201
    pub fn TraceAfterDispatch(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.box_);
        visitor.Trace(&self.break_token_);
        visitor.Trace(&self.inline_container_);
    }

    // cpp: layoutng/internal/oof_positioned_node.h:203-224
    pub fn Node(&self) -> BlockNode {
        BlockNode::new(self.box_.Get())
    }

    pub fn GetBreakToken(&self) -> *const BlockBreakToken {
        self.break_token_.Get()
    }

    pub fn StaticPosition(&self) -> StaticPositionType {
        self.static_position_
    }

    pub fn InlineContainer(&self) -> *const LayoutInline {
        self.inline_container_.Container()
    }

    pub fn InlineContainerInfo(&self) -> &OofInlineContainer<OffsetType> {
        &self.inline_container_
    }

    pub fn InlineContainerInfoMut(&mut self) -> &mut OofInlineContainer<OffsetType> {
        &mut self.inline_container_
    }

    pub fn IsForFragmentation(&self) -> bool {
        debug_assert!(
            !self.is_for_fragmentation_ || !RuntimeEnabledFeatures::FragmentedOofInCbEnabled()
        );
        self.is_for_fragmentation_
    }

    pub fn RequiresContentBeforeBreaking(&self) -> bool {
        self.requires_content_before_breaking_
    }

    // cpp: layoutng/internal/oof_positioned_node.h:226-238
    pub fn SetStaticPositionOffset(&mut self, offset: OffsetType) {
        self.static_position_.SetOffset(offset);
    }

    pub fn IncreaseStaticPositionOffset(&mut self, increase: OffsetType) {
        self.static_position_.IncreaseOffset(increase);
    }

    pub fn SetInlineContainer(&mut self, object: *const LayoutInline) {
        self.inline_container_ = OofInlineContainer::new(object, OffsetType::default());
    }
}

pub type PhysicalOofPositionedNode = OofPositionedNode<PhysicalOffset, PhysicalStaticPosition>;
pub type LogicalOofPositionedNode = OofPositionedNode<LogicalOffset, LogicalStaticPosition>;

// cpp: layoutng/internal/oof_positioned_node.h:284-309
#[repr(C)]
#[derive(Clone)]
pub struct PhysicalOofNodeForFragmentation {
    base_: PhysicalOofPositionedNode,
    pub containing_block: OofContainingBlock<PhysicalOffset>,
    pub fixedpos_containing_block: OofContainingBlock<PhysicalOffset>,
    pub fixedpos_inline_container: OofInlineContainer<PhysicalOffset>,
}

impl PhysicalOofNodeForFragmentation {
    // cpp: layoutng/internal/oof_positioned_node.h:294-317
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        node: BlockNode,
        static_position: PhysicalStaticPosition,
        requires_content_before_breaking: bool,
        inline_container: OofInlineContainer<PhysicalOffset>,
        containing_block: OofContainingBlock<PhysicalOffset>,
        fixedpos_containing_block: OofContainingBlock<PhysicalOffset>,
        fixedpos_inline_container: OofInlineContainer<PhysicalOffset>,
    ) -> Self {
        debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
        let mut base_ = PhysicalOofPositionedNode::new_with_inline(
            node,
            std::ptr::null(),
            static_position,
            requires_content_before_breaking,
            inline_container,
        );
        base_.is_for_fragmentation_ = true;
        Self {
            base_,
            containing_block,
            fixedpos_containing_block,
            fixedpos_inline_container,
        }
    }

    // cpp: layoutng/internal/oof_positioned_node.cc:67-73
    pub fn TraceAfterDispatch(&self, visitor: &mut Visitor) {
        self.base_.TraceAfterDispatch(visitor);
        visitor.Trace(&self.containing_block);
        visitor.Trace(&self.fixedpos_containing_block);
        visitor.Trace(&self.fixedpos_inline_container);
    }
}

impl Deref for PhysicalOofNodeForFragmentation {
    type Target = PhysicalOofPositionedNode;

    fn deref(&self) -> &Self::Target {
        &self.base_
    }
}

impl DerefMut for PhysicalOofNodeForFragmentation {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base_
    }
}

// cpp: layoutng/internal/oof_positioned_node.h:323-374
#[repr(C)]
#[derive(Clone)]
pub struct LogicalOofNodeForFragmentation {
    base_: LogicalOofPositionedNode,
    pub containing_block: OofContainingBlock<LogicalOffset>,
    pub fixedpos_containing_block: OofContainingBlock<LogicalOffset>,
    pub fixedpos_inline_container: OofInlineContainer<LogicalOffset>,
}

impl LogicalOofNodeForFragmentation {
    // cpp: layoutng/internal/oof_positioned_node.h:333-357
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        node: BlockNode,
        static_position: LogicalStaticPosition,
        requires_content_before_breaking: bool,
        inline_container: OofInlineContainer<LogicalOffset>,
        containing_block: OofContainingBlock<LogicalOffset>,
        fixedpos_containing_block: OofContainingBlock<LogicalOffset>,
        fixedpos_inline_container: OofInlineContainer<LogicalOffset>,
    ) -> Self {
        debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
        let mut base_ = LogicalOofPositionedNode::new_with_inline(
            node,
            std::ptr::null(),
            static_position,
            requires_content_before_breaking,
            inline_container,
        );
        base_.is_for_fragmentation_ = true;
        Self {
            base_,
            containing_block,
            fixedpos_containing_block,
            fixedpos_inline_container,
        }
    }

    // cpp: layoutng/internal/oof_positioned_node.h:359-369
    pub fn from_oof(oof_node: &LogicalOofPositionedNode) -> Self {
        debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
        let mut base_ = LogicalOofPositionedNode::new_with_inline(
            oof_node.Node(),
            std::ptr::null(),
            oof_node.StaticPosition(),
            oof_node.RequiresContentBeforeBreaking(),
            oof_node.InlineContainerInfo().clone(),
        );
        base_.is_for_fragmentation_ = true;
        Self {
            base_,
            containing_block: OofContainingBlock::default(),
            fixedpos_containing_block: OofContainingBlock::default(),
            fixedpos_inline_container: OofInlineContainer::default(),
        }
    }

    // cpp: layoutng/internal/oof_positioned_node.cc:10-12
    pub fn CssContainingBlock(&self) -> *const LayoutObject {
        unsafe { &*self.base_.box_.Get() }.Container()
    }

    // cpp: layoutng/internal/oof_positioned_node.h:368-373
    pub fn AllowFrom(oof_node: &LogicalOofPositionedNode) -> bool {
        oof_node.IsForFragmentation()
    }

    // cpp: layoutng/internal/oof_positioned_node.cc:75-81
    pub fn TraceAfterDispatch(&self, visitor: &mut Visitor) {
        self.base_.TraceAfterDispatch(visitor);
        visitor.Trace(&self.containing_block);
        visitor.Trace(&self.fixedpos_containing_block);
        visitor.Trace(&self.fixedpos_inline_container);
    }
}

impl Traceable for LogicalOofNodeForFragmentation {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.TraceAfterDispatch(visitor);
    }
}

impl Deref for LogicalOofNodeForFragmentation {
    type Target = LogicalOofPositionedNode;

    fn deref(&self) -> &Self::Target {
        &self.base_
    }
}

impl DerefMut for LogicalOofNodeForFragmentation {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base_
    }
}

impl OofPositionedNode<PhysicalOffset, PhysicalStaticPosition> {
    // cpp: layoutng/internal/oof_positioned_node.cc:15-27
    pub fn Trace(&self, visitor: &mut Visitor) {
        if self.is_for_fragmentation_ {
            let descendant = self as *const _ as *const PhysicalOofNodeForFragmentation;
            unsafe { &*descendant }.TraceAfterDispatch(visitor);
        } else {
            self.TraceAfterDispatch(visitor);
        }
    }
}

impl OofPositionedNode<LogicalOffset, LogicalStaticPosition> {
    // cpp: layoutng/internal/oof_positioned_node.cc:29-39
    pub fn Trace(&self, visitor: &mut Visitor) {
        if self.is_for_fragmentation_ {
            let descendant = self as *const _ as *const LogicalOofNodeForFragmentation;
            unsafe { &*descendant }.TraceAfterDispatch(visitor);
        } else {
            self.TraceAfterDispatch(visitor);
        }
    }
}

// cpp: layoutng/internal/oof_positioned_node.h:382-424
#[repr(C)]
pub struct FragmentedOofData {
    base_: OofData,
    pub oof_positioned_fragmentainer_descendants: HeapVector<PhysicalOofNodeForFragmentation>,
    pub multicols_with_pending_oofs:
        HeapHashMap<Member<LayoutBox>, Member<MulticolWithPendingOofs<PhysicalOffset>>>,
}

impl Default for FragmentedOofData {
    // cpp: layoutng/internal/oof_positioned_node.h:387-389
    fn default() -> Self {
        debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
        Self {
            base_: OofData::default(),
            oof_positioned_fragmentainer_descendants: HeapVector::default(),
            multicols_with_pending_oofs: HeapHashMap::default(),
        }
    }
}

impl FragmentedOofData {
    // cpp: layoutng/internal/oof_positioned_node.h:391-396
    pub fn HasOutOfFlowPositionedFragmentainerDescendants(fragment: &PhysicalFragment) -> bool {
        let data = fragment.GetFragmentedOofData();
        !data.is_null()
            && !unsafe { &*data }
                .oof_positioned_fragmentainer_descendants
                .is_empty()
    }

    // cpp: layoutng/internal/oof_positioned_node.h:398-401
    pub fn NeedsOOFPositionedInfoPropagation(&self) -> bool {
        !self.oof_positioned_fragmentainer_descendants.is_empty()
            || !self.multicols_with_pending_oofs.is_empty()
    }

    // cpp: layoutng/internal/oof_positioned_node.h:403-414
    pub fn OutOfFlowPositionedFragmentainerDescendants(
        fragment: &PhysicalFragment,
    ) -> *mut [PhysicalOofNodeForFragmentation] {
        let data = fragment.GetFragmentedOofData();
        if data.is_null()
            || unsafe { &*data }
                .oof_positioned_fragmentainer_descendants
                .is_empty()
        {
            return std::ptr::slice_from_raw_parts_mut(
                std::ptr::NonNull::<PhysicalOofNodeForFragmentation>::dangling().as_ptr(),
                0,
            );
        }
        let descendants = unsafe {
            &mut (*(data as *mut FragmentedOofData)).oof_positioned_fragmentainer_descendants
        };
        std::ptr::slice_from_raw_parts_mut(descendants.as_mut_ptr(), descendants.len())
    }

    // cpp: layoutng/internal/oof_positioned_node.h:416-421
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.oof_positioned_fragmentainer_descendants);
        visitor.Trace(&self.multicols_with_pending_oofs);
        self.base_.Trace(visitor);
    }
}

impl Deref for FragmentedOofData {
    type Target = OofData;

    fn deref(&self) -> &Self::Target {
        &self.base_
    }
}

impl DerefMut for FragmentedOofData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base_
    }
}

// cpp: layoutng/internal/oof_positioned_node.h:426-431
pub fn RelativeInsetToPhysical(
    relative_inset: LogicalOffset,
    writing_direction: WritingDirectionMode,
) -> PhysicalOffset {
    relative_inset.ConvertToPhysical(
        writing_direction,
        PhysicalSize::default(),
        PhysicalSize::default(),
    )
}

// cpp: layoutng/internal/oof_positioned_node.h:433-438
pub fn RelativeInsetToLogical(
    relative_inset: PhysicalOffset,
    writing_direction: WritingDirectionMode,
) -> LogicalOffset {
    WritingModeConverter::new(writing_direction, PhysicalSize::default())
        .ToLogicalOffset(relative_inset, PhysicalSize::default())
}

// cpp: layoutng/internal/oof_positioned_node.cc:41-52
pub fn LogicalOofPositionedNodeToPhysical(
    logical: &LogicalOofPositionedNode,
    converter: &WritingModeConverter,
) -> PhysicalOofPositionedNode {
    let inline_container = OofInlineContainer::new(
        logical.InlineContainer(),
        converter.ToPhysicalOffset(
            logical.InlineContainerInfo().RelativeOffset(),
            PhysicalSize::default(),
        ),
    );
    PhysicalOofPositionedNode::new_with_inline(
        logical.Node(),
        logical.GetBreakToken(),
        logical.StaticPosition().ConvertToPhysical(converter),
        logical.RequiresContentBeforeBreaking(),
        inline_container,
    )
}

// cpp: layoutng/internal/oof_positioned_node.cc:54-65
pub fn PhysicalOofPositionedNodeToLogical(
    physical: &PhysicalOofPositionedNode,
    converter: &WritingModeConverter,
) -> LogicalOofPositionedNode {
    let inline_container = OofInlineContainer::new(
        physical.InlineContainer(),
        converter.ToLogicalOffset(
            physical.InlineContainerInfo().RelativeOffset(),
            PhysicalSize::default(),
        ),
    );
    LogicalOofPositionedNode::new_with_inline(
        physical.Node(),
        physical.GetBreakToken(),
        physical.StaticPosition().ConvertToLogical(converter),
        physical.RequiresContentBeforeBreaking(),
        inline_container,
    )
}
