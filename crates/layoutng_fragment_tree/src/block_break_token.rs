use std::cell::UnsafeCell;
use std::ops::Deref;

use foundation::{
    LayoutUnit, MakeGarbageCollected, MakeGarbageCollectedWithAdditionalBytes, Member,
    RuntimeEnabledFeatures, String, StringBuilder, Visitor,
};
use layoutng::internal::block_node::BlockNode;
use layoutng::internal::layout_box::LayoutBox;
use layoutng::internal::layout_input_node::LayoutInputNode;
use layoutng_geometry::geometry::logical_offset::LogicalOffset;

use crate::box_fragment_builder::BoxFragmentBuilder;
use crate::break_token::{BreakToken, BreakTokenType};
use crate::break_token_algorithm_data::BreakTokenAlgorithmData;

// The inherited BreakToken is first, and child tokens remain in an allocated
// trailing array. The two fields mutated through the C++ const-cast wrapper
// use UnsafeCell, which is layout-transparent and retains that mutability.
// cpp: layoutng_fragment_tree/block_break_token.h:24-25
// cpp: layoutng_fragment_tree/block_break_token.h:266-277
#[repr(C)]
pub struct BlockBreakToken {
    pub(crate) base_: BreakToken,
    box_: Member<LayoutBox>,
    data_: Member<BreakTokenAlgorithmData>,
    consumed_block_size_: LayoutUnit,
    monolithic_overflow_: UnsafeCell<LayoutUnit>,
    oof_start_offset_: UnsafeCell<LogicalOffset>,
    sequence_number_: u32,
    const_num_children_: u32,
    child_break_tokens_: [Member<BreakToken>; 0],
}

// cpp: layoutng_fragment_tree/block_break_token.h:279-284
impl foundation::DowncastFrom<BreakToken> for BlockBreakToken {
    fn AllowFrom(token: &BreakToken) -> bool {
        token.IsBlockType()
    }
}

const _: () = assert!(std::mem::offset_of!(BlockBreakToken, base_) == 0);

impl Deref for BlockBreakToken {
    type Target = BreakToken;

    fn deref(&self) -> &Self::Target {
        &self.base_
    }
}

// C++'s STACK_ALLOCATED wrapper holds a mutable reference obtained from a
// const token. Rust keeps a shared reference and limits writes to UnsafeCell.
// cpp: layoutng_fragment_tree/block_break_token.h:216-224
pub struct BlockBreakTokenMutableForOofFragmentation<'a> {
    break_token_: &'a BlockBreakToken,
}

#[allow(non_snake_case)]
impl BlockBreakTokenMutableForOofFragmentation<'_> {
    // cpp: layoutng_fragment_tree/block_break_token.h:225-228
    // cpp: layoutng_out_of_flow/out_of_flow_fragment_builder.cc:267-275
    pub fn Merge(&self, new_break_token: &BlockBreakToken) {
        debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
        let overflow = new_break_token.MonolithicOverflow();
        if overflow != LayoutUnit::default() {
            debug_assert!(overflow > LayoutUnit::default());
            let current = unsafe { &mut *self.break_token_.monolithic_overflow_.get() };
            *current = (*current).max(overflow);
        }
    }

    // cpp: layoutng_fragment_tree/block_break_token.h:230-232
    pub fn SetInlineStartOffset(&self, offset: LayoutUnit) {
        unsafe { (*self.break_token_.oof_start_offset_.get()).inline_offset = offset };
    }

    // cpp: layoutng_fragment_tree/block_break_token.h:233-235
    pub fn SetBlockStartOffset(&self, offset: LayoutUnit) {
        unsafe { (*self.break_token_.oof_start_offset_.get()).block_offset = offset };
    }
}

#[allow(non_snake_case)]
impl BlockBreakToken {
    // cpp: layoutng_fragment_tree/block_break_token.h:253
    // cpp: layoutng_fragment_tree/block_break_token.cc:107-110
    fn new_simple(node: LayoutInputNode) -> Self {
        let base = BreakToken::new(BreakTokenType::kBlockBreakToken, node.clone(), 0);
        let box_ptr = node.GetLayoutBox();
        Self {
            base_: base,
            box_: Member::from_ptr(box_ptr),
            data_: Member::default(),
            consumed_block_size_: LayoutUnit::default(),
            monolithic_overflow_: UnsafeCell::new(LayoutUnit::default()),
            oof_start_offset_: UnsafeCell::new(LogicalOffset::default()),
            sequence_number_: 0,
            const_num_children_: 0,
            child_break_tokens_: [],
        }
    }

    // cpp: layoutng_fragment_tree/block_break_token.h:27-32
    // cpp: layoutng_fragment_tree/block_break_token.cc:56-64
    // cpp: layoutng_fragment_tree/block_break_token.cc:66-105
    pub fn Create(builder: &mut BoxFragmentBuilder) -> *mut Self {
        let num_children = builder.child_break_tokens_.len() as u32;
        let additional_bytes = num_children as usize * std::mem::size_of::<Member<BreakToken>>();
        // Foundation's GC allocator must initialize the fixed header and all
        // trailing members before registering the object for tracing.
        unsafe {
            MakeGarbageCollectedWithAdditionalBytes(additional_bytes, |ptr: *mut Self| {
                let mut base =
                    BreakToken::new(BreakTokenType::kBlockBreakToken, builder.node_.clone(), 0);
                let box_ptr = builder.node_.GetLayoutBox();
                base.set_bit(
                    BreakToken::SEEN_ALL_CHILDREN_BIT,
                    builder.has_seen_all_children_,
                );
                base.set_bit(BreakToken::COLUMN_SPANNER_BIT, builder.FoundColumnSpanner());
                base.set_bit(BreakToken::AT_BLOCK_END_BIT, builder.is_at_block_end_);
                base.set_bit(
                    BreakToken::UNPOSITIONED_LIST_MARKER_BIT,
                    builder.GetUnpositionedListMarker().is_present(),
                );
                let data = std::mem::take(&mut builder.break_token_data_);
                ptr.write(Self {
                    base_: base,
                    box_: Member::from_ptr(box_ptr),
                    data_: Member::from_ptr(data),
                    consumed_block_size_: builder.consumed_block_size_,
                    monolithic_overflow_: UnsafeCell::new(builder.monolithic_overflow_),
                    oof_start_offset_: UnsafeCell::new(LogicalOffset::default()),
                    sequence_number_: builder.sequence_number_,
                    const_num_children_: num_children,
                    child_break_tokens_: [],
                });

                // Out-of-flow children must precede in-flow children. The
                // C++ stable_sort comparator orders by !is_oof; sort_by_key
                // is stable and uses the same key.
                if RuntimeEnabledFeatures::FragmentedOofInCbEnabled() {
                    builder.child_break_tokens_.sort_by_key(|token| {
                        let token_ptr = token.Get();
                        let is_oof = !token_ptr.is_null()
                            && (&*token_ptr)
                                .as_block()
                                .is_some_and(|block| block.InputNode().IsOutOfFlowPositioned());
                        !is_oof
                    });
                }

                let children =
                    std::ptr::addr_of_mut!((*ptr).child_break_tokens_) as *mut Member<BreakToken>;
                for i in 0..num_children as usize {
                    children
                        .add(i)
                        .write(builder.child_break_tokens_[i].clone());
                }
            })
        }
    }

    // cpp: layoutng_fragment_tree/block_break_token.h:41-51
    pub fn CreateBreakBeforeAtDefaultOffset(
        node: LayoutInputNode,
        is_forced_break: bool,
    ) -> *mut Self {
        Self::CreateBreakBefore(node, is_forced_break, LogicalOffset::default())
    }

    pub fn CreateBreakBefore(
        node: LayoutInputNode,
        is_forced_break: bool,
        oof_start_offset: LogicalOffset,
    ) -> *mut Self {
        let token = MakeGarbageCollected(Self::new_simple(node.clone()));
        let token = unsafe { &mut *token };
        token.base_.set_bit(BreakToken::BREAK_BEFORE_BIT, true);
        token
            .base_
            .set_bit(BreakToken::FORCED_BREAK_BIT, is_forced_break);
        *token.oof_start_offset_.get_mut() = oof_start_offset;
        token
            .base_
            .set_bit(BreakToken::UNPOSITIONED_LIST_MARKER_BIT, node.IsListItem());
        token
    }

    // cpp: layoutng_fragment_tree/block_break_token.h:53-57
    // cpp: layoutng_fragment_tree/block_break_token.cc:19-25
    pub fn CreateRepeated(node: &BlockNode, sequence_number: u32) -> *mut Self {
        let token = MakeGarbageCollected(Self::new_simple(node.clone().into()));
        let token = unsafe { &mut *token };
        token.sequence_number_ = sequence_number;
        token.base_.set_bit(BreakToken::REPEATED_BIT, true);
        token
    }

    // cpp: layoutng_fragment_tree/block_break_token.h:59-77
    // cpp: layoutng_fragment_tree/block_break_token.cc:27-38
    pub fn CreateForBreakInRepeatedFragment(
        node: &BlockNode,
        sequence_number: u32,
        consumed_block_size: LayoutUnit,
        is_at_block_end: bool,
    ) -> *mut Self {
        let token = MakeGarbageCollected(Self::new_simple(node.clone().into()));
        let token = unsafe { &mut *token };
        token.sequence_number_ = sequence_number;
        token.consumed_block_size_ = consumed_block_size;
        token
            .base_
            .set_bit(BreakToken::AT_BLOCK_END_BIT, is_at_block_end);
        token
            .base_
            .set_bit(BreakToken::REPEATED_ACTUAL_BREAK_BIT, true);
        token
    }

    // cpp: layoutng_fragment_tree/block_break_token.h:79-86
    pub fn ConsumedBlockSize(&self) -> LayoutUnit {
        self.consumed_block_size_
    }

    // cpp: layoutng_fragment_tree/block_break_token.h:88-89
    pub fn InputNode(&self) -> BlockNode {
        BlockNode::new(self.box_.Get())
    }

    // cpp: layoutng_fragment_tree/block_break_token.h:91-100
    pub fn SequenceNumber(&self) -> u32 {
        debug_assert!(
            self.base_.bit(BreakToken::REPEATED_ACTUAL_BREAK_BIT) || !self.IsBreakBefore()
        );
        self.sequence_number_
    }

    // cpp: layoutng_fragment_tree/block_break_token.h:102-112
    pub fn MonolithicOverflow(&self) -> LayoutUnit {
        debug_assert!(!self.base_.bit(BreakToken::REPEATED_ACTUAL_BREAK_BIT));
        unsafe { *self.monolithic_overflow_.get() }
    }

    // cpp: layoutng_fragment_tree/block_break_token.h:114-117
    pub fn TokenData(&self) -> *const BreakTokenAlgorithmData {
        debug_assert!(!self.base_.bit(BreakToken::REPEATED_ACTUAL_BREAK_BIT));
        self.data_.Get()
    }

    // cpp: layoutng_fragment_tree/block_break_token.h:119-133
    pub fn OofBlockStartOffset(&self) -> LayoutUnit {
        debug_assert!(self.InputNode().IsOutOfFlowPositioned());
        unsafe { (*self.oof_start_offset_.get()).block_offset }
    }

    // cpp: layoutng_fragment_tree/block_break_token.h:135-143
    pub fn OofInlineStartOffset(&self) -> LayoutUnit {
        debug_assert!(self.InputNode().IsOutOfFlowPositioned());
        unsafe { (*self.oof_start_offset_.get()).inline_offset }
    }

    // cpp: layoutng_fragment_tree/block_break_token.h:145-149
    pub fn IsBreakBefore(&self) -> bool {
        self.base_.bit(BreakToken::BREAK_BEFORE_BIT)
    }

    // cpp: layoutng_fragment_tree/block_break_token.h:151
    pub fn IsForcedBreak(&self) -> bool {
        self.base_.bit(BreakToken::FORCED_BREAK_BIT)
    }

    // cpp: layoutng_fragment_tree/block_break_token.h:153-157
    pub fn IsRepeated(&self) -> bool {
        self.base_.bit(BreakToken::REPEATED_BIT)
    }

    // cpp: layoutng_fragment_tree/block_break_token.h:159-162
    pub fn IsCausedByColumnSpanner(&self) -> bool {
        debug_assert!(!self.base_.bit(BreakToken::REPEATED_ACTUAL_BREAK_BIT));
        self.base_.bit(BreakToken::COLUMN_SPANNER_BIT)
    }

    // cpp: layoutng_fragment_tree/block_break_token.h:164-171
    pub fn HasSeenAllChildren(&self) -> bool {
        debug_assert!(!self.base_.bit(BreakToken::REPEATED_ACTUAL_BREAK_BIT));
        self.base_.bit(BreakToken::SEEN_ALL_CHILDREN_BIT)
    }

    // cpp: layoutng_fragment_tree/block_break_token.h:173-195
    pub fn IsAtBlockEnd(&self) -> bool {
        self.base_.bit(BreakToken::AT_BLOCK_END_BIT)
    }

    // cpp: layoutng_fragment_tree/block_break_token.h:197-201
    pub fn HasUnpositionedListMarker(&self) -> bool {
        debug_assert!(!self.base_.bit(BreakToken::REPEATED_ACTUAL_BREAK_BIT));
        self.base_.bit(BreakToken::UNPOSITIONED_LIST_MARKER_BIT)
    }

    // cpp: layoutng_fragment_tree/block_break_token.h:203-214
    pub fn ChildBreakTokens(&self) -> &[Member<BreakToken>] {
        debug_assert!(!self.base_.bit(BreakToken::REPEATED_ACTUAL_BREAK_BIT));
        self.ChildBreakTokensInternal()
    }

    // cpp: layoutng_fragment_tree/block_break_token.h:241-243
    pub fn GetMutableForOofFragmentation(&self) -> BlockBreakTokenMutableForOofFragmentation<'_> {
        BlockBreakTokenMutableForOofFragmentation { break_token_: self }
    }

    // cpp: layoutng_fragment_tree/block_break_token.h:245
    // cpp: layoutng_fragment_tree/block_break_token.cc:112-154
    pub fn ToString(&self) -> String {
        self.ToStringWithSkipNodeInfo(false)
    }

    pub fn ToStringWithSkipNodeInfo(&self, skip_node_info: bool) -> String {
        let mut builder = StringBuilder::default();
        if !skip_node_info {
            builder.Append(&self.InputNode().ToString());
        }
        if self.IsBreakBefore() {
            if self.IsForcedBreak() {
                builder.Append(" forced");
            }
            builder.Append(" break-before");
        } else {
            builder.Append(" sequence:");
            builder.AppendNumber(self.SequenceNumber());
        }
        if self.IsRepeated() {
            builder.Append(" (repeated)");
        }
        if self.base_.bit(BreakToken::COLUMN_SPANNER_BIT) {
            builder.Append(" (caused by spanner)");
        }
        if self.base_.bit(BreakToken::SEEN_ALL_CHILDREN_BIT) {
            builder.Append(" (seen all children)");
        }
        if self.IsAtBlockEnd() {
            builder.Append(" (at block-end)");
        }

        let oof_offset = unsafe { *self.oof_start_offset_.get() };
        if oof_offset != LogicalOffset::default() {
            builder.Append(" oof-offset:");
            builder.Append(&oof_offset.ToString());
        }
        builder.Append(" consumed:");
        builder.Append(&self.ConsumedBlockSize().ToString());
        builder.Append("px");

        if !self.base_.bit(BreakToken::REPEATED_ACTUAL_BREAK_BIT)
            && self.MonolithicOverflow() != LayoutUnit::default()
        {
            builder.Append(" monolithic overflow:");
            builder.Append(&self.MonolithicOverflow().ToString());
            builder.Append("px");
        }
        builder.ToString()
    }

    // cpp: layoutng_fragment_tree/block_break_token.h:255
    // cpp: layoutng_fragment_tree/block_break_token.cc:156-167
    pub fn TraceAfterDispatch(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.box_);
        visitor.Trace(&self.data_);
        for child in self.ChildBreakTokensInternal() {
            visitor.Trace(child);
        }
        self.base_.TraceAfterDispatch(visitor);
    }

    // cpp: layoutng_fragment_tree/block_break_token.h:258-264
    fn ChildBreakTokensInternal(&self) -> &[Member<BreakToken>] {
        let ptr = self.child_break_tokens_.as_ptr();
        unsafe { std::slice::from_raw_parts(ptr, self.const_num_children_ as usize) }
    }

    // cpp: layoutng_fragment_tree/block_break_token.h:279-282
    pub fn AllowFrom(token: &BreakToken) -> bool {
        token.IsBlockType()
    }
}

// cpp: layoutng_fragment_tree/block_break_token.cc:42-52
#[repr(C)]
struct SameSizeAsBlockBreakToken {
    base: BreakToken,
    box_: Member<LayoutBox>,
    data: Member<BreakTokenAlgorithmData>,
    consumed_block_size: LayoutUnit,
    monolithic_overflow: LayoutUnit,
    oof_start_offset: LogicalOffset,
    sequence_number: u32,
    numbers: [u32; 1],
}

const _: [(); std::mem::size_of::<SameSizeAsBlockBreakToken>()] =
    [(); std::mem::size_of::<BlockBreakToken>()];
