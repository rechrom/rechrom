#![allow(non_snake_case)]
use foundation::{ECaptionSide, To};
use layoutng_assembly::{
    block_break_token::BlockBreakToken,
    internal::{
        block_node::BlockNode,
        table_layout_algorithm_types::{TableGroupedChildren, TableGroupedChildrenIterator},
    },
};

// cpp: layoutng_table/table_child_iterator.h:26-84
pub struct TableChildIterator<'a> {
    grouped_children: Option<&'a TableGroupedChildren>,
    break_token: Option<&'a BlockBreakToken>,
    section_iterator: Option<TableGroupedChildrenIterator>,
    child_token_idx: usize,
    caption_idx: usize,
    section_idx: u32,
}

// cpp: layoutng_table/table_child_iterator.h:33-57
pub struct Entry {
    node: BlockNode,
    token: *const BlockBreakToken,
    section_index: u32,
}
impl Entry {
    pub fn new(node: BlockNode, token: *const BlockBreakToken, section_index: u32) -> Self {
        Self {
            node,
            token,
            section_index,
        }
    }
    pub fn GetNode(&self) -> BlockNode {
        self.node.clone()
    }
    pub fn GetBreakToken(&self) -> *const BlockBreakToken {
        self.token
    }
    pub fn GetSectionIndex(&self) -> u32 {
        debug_assert!(!self.node.IsTableCaption());
        self.section_index
    }
    pub fn is_non_null(&self) -> bool {
        self.node.is_non_null()
    }
}

impl<'a> TableChildIterator<'a> {
    // cpp: layoutng_table/table_child_iterator.cc:12-48
    pub fn new(
        grouped_children: &'a TableGroupedChildren,
        break_token: Option<&'a BlockBreakToken>,
    ) -> Self {
        let mut result = Self {
            grouped_children: Some(grouped_children),
            break_token,
            section_iterator: None,
            child_token_idx: 0,
            caption_idx: 0,
            section_idx: 0,
        };
        if let Some(token) = result.break_token {
            if token.ChildBreakTokens().is_empty() {
                if token.HasSeenAllChildren() {
                    result.grouped_children = None;
                    return result;
                }
                result.break_token = None;
            }
        }
        if !grouped_children.captions.is_empty() {
            while result.caption_idx < grouped_children.captions.len() {
                if grouped_children.captions[result.caption_idx]
                    .Style()
                    .CaptionSide()
                    == ECaptionSide::kTop
                {
                    return result;
                }
                result.caption_idx += 1;
            }
            result.caption_idx = 0;
        }
        result.section_iterator = Some(grouped_children.begin());
        result
    }

    // cpp: layoutng_table/table_child_iterator.cc:50-88
    pub fn NextChild(&mut self) -> Entry {
        let mut current_token = std::ptr::null();
        let mut current_child = BlockNode::null();
        if let Some(token) = self.break_token {
            let tokens = token.ChildBreakTokens();
            if self.child_token_idx < tokens.len() {
                current_token = To::<BlockBreakToken>(tokens[self.child_token_idx].Get());
                self.child_token_idx += 1;
                current_child = unsafe { &*current_token }.InputNode();
                while self.CurrentChild() != current_child {
                    self.AdvanceChild();
                    debug_assert!(self.CurrentChild().is_non_null());
                }
                if self.child_token_idx == tokens.len() {
                    if token.HasSeenAllChildren() {
                        self.grouped_children = None;
                    }
                    self.break_token = None;
                }
            }
        } else {
            current_child = self.CurrentChild();
        }
        let section_idx = self.section_idx;
        self.AdvanceChild();
        Entry::new(current_child, current_token, section_idx)
    }

    // cpp: layoutng_table/table_child_iterator.cc:90-114
    fn CurrentChild(&self) -> BlockNode {
        let Some(grouped) = self.grouped_children else {
            return BlockNode::null();
        };
        let Some(iterator) = &self.section_iterator else {
            debug_assert!(
                grouped.captions[self.caption_idx].Style().CaptionSide() == ECaptionSide::kTop
            );
            return grouped.captions[self.caption_idx].clone();
        };
        if !iterator.Equals(&grouped.end()) {
            return iterator.Dereference();
        }
        if self.caption_idx < grouped.captions.len() {
            debug_assert!(
                grouped.captions[self.caption_idx].Style().CaptionSide() == ECaptionSide::kBottom
            );
            return grouped.captions[self.caption_idx].clone();
        }
        BlockNode::null()
    }

    // cpp: layoutng_table/table_child_iterator.cc:116-161
    fn AdvanceChild(&mut self) {
        let Some(grouped) = self.grouped_children else {
            return;
        };
        if self.section_iterator.is_none() {
            self.caption_idx += 1;
            while self.caption_idx < grouped.captions.len() {
                if grouped.captions[self.caption_idx].Style().CaptionSide() == ECaptionSide::kTop {
                    return;
                }
                self.caption_idx += 1;
            }
            self.caption_idx = 0;
            self.section_iterator = Some(grouped.begin());
            if !self
                .section_iterator
                .as_ref()
                .unwrap()
                .Equals(&grouped.end())
            {
                return;
            }
        } else {
            let iterator = self.section_iterator.as_mut().unwrap();
            if !iterator.Equals(&grouped.end()) {
                iterator.Increment();
                self.section_idx = self.section_idx.wrapping_add(1);
                if !iterator.Equals(&grouped.end()) {
                    return;
                }
            } else {
                self.caption_idx += 1;
            }
        }
        while self.caption_idx < grouped.captions.len() {
            if grouped.captions[self.caption_idx].Style().CaptionSide() == ECaptionSide::kBottom {
                return;
            }
            self.caption_idx += 1;
        }
    }
}
