#![allow(non_snake_case)]

use foundation::{DynamicTo, RuntimeEnabledFeatures, To};
use layoutng::internal::block_node::BlockNode;
use layoutng::internal::layout_input_node::LayoutInputNode;
use layoutng_fragment_tree::block_break_token::BlockBreakToken;
use layoutng_fragment_tree::break_token::BreakToken;
use layoutng_fragment_tree::inline_break_token::InlineBreakToken;

// STACK_ALLOCATED maps to a value type. The iterator holds node handles and
// borrowed GC token pointers, and does not own either layout tree.
// cpp: layoutng_block/block_child_iterator.h:31-59
pub struct BlockChildIterator {
    next_unstarted_child_: BlockNode,
    tracked_child_: BlockNode,
    container_break_token_: *const BlockBreakToken,
    child_token_idx_: u32,
    child_idx_: Option<u32>,
    did_handle_first_child_: bool,
    is_ifc_: bool,
}

// cpp: layoutng_block/block_child_iterator.h:61-90
pub struct BlockChildIteratorEntry {
    pub block_node: BlockNode,
    pub token: *const BreakToken,
    pub index: Option<u32>,
    pub is_unstarted_ifc: bool,
}

impl Default for BlockChildIteratorEntry {
    fn default() -> Self {
        Self {
            block_node: BlockNode::null(),
            token: std::ptr::null(),
            index: None,
            is_unstarted_ifc: false,
        }
    }
}

impl BlockChildIteratorEntry {
    pub fn new_block(node: &BlockNode, token: *const BreakToken, index: Option<u32>) -> Self {
        Self {
            block_node: node.clone(),
            token,
            index,
            is_unstarted_ifc: false,
        }
    }

    pub fn new_inline(token: *const InlineBreakToken, is_unstarted_ifc: bool) -> Self {
        Self {
            block_node: BlockNode::null(),
            token: token as *const BreakToken,
            index: None,
            is_unstarted_ifc,
        }
    }

    pub fn AtEnd(&self) -> bool {
        !self.block_node.is_non_null() && self.token.is_null() && !self.is_unstarted_ifc
    }
}

impl PartialEq for BlockChildIteratorEntry {
    fn eq(&self, other: &Self) -> bool {
        self.block_node.EqualsBlockNode(&other.block_node)
            && self.token == other.token
            && self.index == other.index
            && self.is_unstarted_ifc == other.is_unstarted_ifc
    }
}

impl Eq for BlockChildIteratorEntry {}

impl BlockChildIterator {
    // cpp: layoutng_block/block_child_iterator.h:36-38
    // cpp: layoutng_block/block_child_iterator.cc:15-47
    pub fn new(
        first_child: LayoutInputNode,
        container_break_token: *const BlockBreakToken,
        calculate_child_idx: bool,
    ) -> Self {
        let is_ifc = first_child.is_non_null() && first_child.IsInline();
        debug_assert!(!calculate_child_idx || !is_ifc);
        let mut result = Self {
            next_unstarted_child_: BlockNode::null(),
            tracked_child_: BlockNode::null(),
            container_break_token_: container_break_token,
            child_token_idx_: 0,
            child_idx_: None,
            did_handle_first_child_: false,
            is_ifc_: is_ifc,
        };
        if result.is_ifc_ {
            return result;
        }
        let first_block_child = BlockNode::from(first_child);
        result.next_unstarted_child_ = first_block_child.clone();
        if calculate_child_idx {
            result.child_idx_ = Some(0);
            result.tracked_child_ = first_block_child;
        }
        if !result.container_break_token_.is_null() {
            let container = unsafe { &*result.container_break_token_ };
            let child_break_tokens = container.ChildBreakTokens();
            if !child_break_tokens.is_empty() || container.HasSeenAllChildren() {
                result.next_unstarted_child_ = BlockNode::null();
            }
            if child_break_tokens.is_empty() {
                result.container_break_token_ = std::ptr::null();
            }
        }
        result
    }

    // cpp: layoutng_block/block_child_iterator.h:43-44
    // cpp: layoutng_block/block_child_iterator.cc:49-140
    pub fn NextChild(
        &mut self,
        previous_inline_break_token: *const InlineBreakToken,
    ) -> BlockChildIteratorEntry {
        debug_assert!(self.is_ifc_ || previous_inline_break_token.is_null());
        if self.is_ifc_ {
            let is_unstarted_ifc = !self.did_handle_first_child_;
            self.did_handle_first_child_ = true;
            if !previous_inline_break_token.is_null() {
                self.container_break_token_ = std::ptr::null();
                return BlockChildIteratorEntry::new_inline(previous_inline_break_token, false);
            }
            if !self.container_break_token_.is_null() {
                let container = unsafe { &*self.container_break_token_ };
                let tokens = container.ChildBreakTokens();
                if tokens.is_empty() && container.HasSeenAllChildren() {
                    return BlockChildIteratorEntry::default();
                }
                if (self.child_token_idx_ as usize) < tokens.len() {
                    let token = tokens[self.child_token_idx_ as usize].Get();
                    self.child_token_idx_ += 1;
                    let block_token = DynamicTo::<BlockBreakToken>(token);
                    if !block_token.is_null() {
                        let node = unsafe { &*block_token }.InputNode();
                        debug_assert!(node.IsOutOfFlowPositioned());
                        debug_assert!(RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
                        return BlockChildIteratorEntry::new_block(
                            &node,
                            block_token as *const BreakToken,
                            None,
                        );
                    }
                    let inline_token = To::<InlineBreakToken>(token);
                    return BlockChildIteratorEntry::new_inline(inline_token, false);
                }
            }
            return BlockChildIteratorEntry::new_inline(std::ptr::null(), is_unstarted_ifc);
        }

        if self.did_handle_first_child_ {
            if !self.container_break_token_.is_null() {
                let container = unsafe { &*self.container_break_token_ };
                let child_break_tokens = container.ChildBreakTokens();
                if self.child_token_idx_ as usize == child_break_tokens.len() {
                    if !container.HasSeenAllChildren() {
                        let last_token =
                            child_break_tokens[(self.child_token_idx_ - 1) as usize].Get();
                        let last_resumed_child =
                            unsafe { &*To::<BlockBreakToken>(last_token) }.InputNode();
                        self.AdvanceToNextChild(&last_resumed_child);
                    }
                    self.container_break_token_ = std::ptr::null();
                }
            } else if self.next_unstarted_child_.is_non_null() {
                let current = self.next_unstarted_child_.clone();
                self.AdvanceToNextChild(&current);
            }
        } else {
            self.did_handle_first_child_ = true;
        }

        let mut current_child_break_token: *const BlockBreakToken = std::ptr::null();
        let mut current_child_idx = None;
        let mut current_child = self.next_unstarted_child_.clone();
        if !self.container_break_token_.is_null() {
            debug_assert!(!self.next_unstarted_child_.is_non_null());
            let child_break_tokens = unsafe { &*self.container_break_token_ }.ChildBreakTokens();
            debug_assert!((self.child_token_idx_ as usize) < child_break_tokens.len());
            current_child_break_token =
                To::<BlockBreakToken>(child_break_tokens[self.child_token_idx_ as usize].Get());
            self.child_token_idx_ += 1;
            current_child = unsafe { &*current_child_break_token }.InputNode();
            if self.child_idx_.is_some() {
                while !self.tracked_child_.EqualsBlockNode(&current_child) {
                    self.tracked_child_ = self.tracked_child_.NextBlockSibling();
                    *self.child_idx_.as_mut().unwrap() += 1;
                }
                current_child_idx = self.child_idx_;
            }
        } else if self.next_unstarted_child_.is_non_null() {
            current_child_idx = self.child_idx_;
        }

        #[cfg(debug_assertions)]
        {
            let box_ = current_child.GetLayoutBox();
            if !box_.is_null() {
                let box_ = unsafe { &*box_ };
                debug_assert!(box_.IsInDetachedNonDomTree() || !box_.Parent().is_null());
            }
        }
        BlockChildIteratorEntry::new_block(
            &current_child,
            current_child_break_token as *const BreakToken,
            current_child_idx,
        )
    }

    pub fn NextChildDefault(&mut self) -> BlockChildIteratorEntry {
        self.NextChild(std::ptr::null())
    }

    // cpp: layoutng_block/block_child_iterator.h:47-47
    // cpp: layoutng_block/block_child_iterator.cc:142-146
    fn AdvanceToNextChild(&mut self, child: &BlockNode) {
        self.next_unstarted_child_ = child.NextBlockSibling();
        if let Some(child_idx) = self.child_idx_.as_mut() {
            *child_idx += 1;
        }
    }
}
