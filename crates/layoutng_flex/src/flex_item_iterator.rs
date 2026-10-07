#![allow(non_snake_case)]

use foundation::{To, WtfSizeT};
use layoutng_assembly::block_break_token::BlockBreakToken;

use crate::flex_line::{FlexItemData, FlexLineVector};

// cpp: layoutng_flex/flex_item_iterator.h:22-81
pub struct FlexItemIterator {
    next_unstarted_item_: *mut FlexItemData,
    flex_lines_: *const FlexLineVector,
    break_token_: *const BlockBreakToken,
    is_column_: bool,
    child_token_idx_: usize,
    flex_line_idx_: usize,
    flex_item_idx_: usize,
    next_item_idx_for_line_: Vec<usize>,
}

// cpp: layoutng_flex/flex_item_iterator.h:83-106
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FlexItemIteratorEntry {
    pub flex_item: *mut FlexItemData,
    pub flex_item_idx: WtfSizeT,
    pub flex_line_idx: WtfSizeT,
    pub token: *const BlockBreakToken,
}

impl FlexItemIteratorEntry {
    pub fn new(
        flex_item: *mut FlexItemData,
        flex_item_idx: WtfSizeT,
        flex_line_idx: WtfSizeT,
        token: *const BlockBreakToken,
    ) -> Self {
        Self {
            flex_item,
            flex_item_idx,
            flex_line_idx,
            token,
        }
    }
}

impl FlexItemIterator {
    // cpp: layoutng_flex/flex_item_iterator.cc:11-37
    pub fn new(
        flex_lines: &FlexLineVector,
        break_token: *const BlockBreakToken,
        is_column: bool,
    ) -> Self {
        let mut result = Self {
            next_unstarted_item_: std::ptr::null_mut(),
            flex_lines_: flex_lines,
            break_token_: break_token,
            is_column_: is_column,
            child_token_idx_: 0,
            flex_line_idx_: 0,
            flex_item_idx_: 0,
            next_item_idx_for_line_: Vec::new(),
        };
        if !flex_lines.is_empty() {
            debug_assert!(!flex_lines[0].line_items_data.is_empty());
            result.next_unstarted_item_ =
                &flex_lines[0].line_items_data[0] as *const FlexItemData as *mut FlexItemData;
            result.flex_item_idx_ += 1;
        }
        if !break_token.is_null() {
            let token = unsafe { &*break_token };
            let child_break_tokens = token.ChildBreakTokens();
            if !child_break_tokens.is_empty() || token.HasSeenAllChildren() {
                result.next_unstarted_item_ = std::ptr::null_mut();
                result.flex_item_idx_ = 0;
            }
            if child_break_tokens.is_empty() {
                result.break_token_ = std::ptr::null();
            }
        }
        result
    }

    // cpp: layoutng_flex/flex_item_iterator.cc:39-108
    pub fn NextItem(&mut self, broke_before_row: bool) -> FlexItemIteratorEntry {
        debug_assert!(!self.is_column_ || !broke_before_row);
        let mut current_child_break_token: *const BlockBreakToken = std::ptr::null();
        let mut current_item = self.next_unstarted_item_;
        let mut current_item_idx: usize = 0;
        let mut current_line_idx: usize = 0;
        if !self.break_token_.is_null() {
            debug_assert!(self.next_unstarted_item_.is_null());
            let child_break_tokens = unsafe { &*self.break_token_ }.ChildBreakTokens();
            if self.child_token_idx_ < child_break_tokens.len() {
                current_child_break_token =
                    To::<BlockBreakToken>(child_break_tokens[self.child_token_idx_].Get());
                self.child_token_idx_ += 1;
                debug_assert!(!current_child_break_token.is_null());
                current_item = self.FindNextItem(current_child_break_token);
                if self.is_column_ {
                    while self.next_item_idx_for_line_.len() <= self.flex_line_idx_ {
                        self.next_item_idx_for_line_.push(0);
                    }
                    self.next_item_idx_for_line_[self.flex_line_idx_] = self.flex_item_idx_;
                }
                current_item_idx = self.flex_item_idx_.wrapping_sub(1);
                current_line_idx = self.flex_line_idx_;
                if self.child_token_idx_ == child_break_tokens.len() {
                    let current_token = unsafe { &*current_child_break_token };
                    if !self.is_column_
                        && (current_item_idx != 0
                            || !current_token.IsBreakBefore()
                            || !broke_before_row)
                    {
                        self.break_token_ = std::ptr::null();
                        self.NextLine();
                    } else if !unsafe { &*self.break_token_ }.HasSeenAllChildren() {
                        if self.is_column_ {
                            self.flex_line_idx_ = 0;
                            self.flex_item_idx_ = self.next_item_idx_for_line_[0];
                        }
                        self.next_unstarted_item_ = self.FindNextItem(std::ptr::null());
                        self.break_token_ = std::ptr::null();
                    }
                }
            }
        } else {
            current_item_idx = self.flex_item_idx_.wrapping_sub(1);
            current_line_idx = self.flex_line_idx_;
            if !self.next_unstarted_item_.is_null() {
                self.next_unstarted_item_ = self.FindNextItem(std::ptr::null());
            }
        }
        FlexItemIteratorEntry::new(
            current_item,
            current_item_idx as WtfSizeT,
            current_line_idx as WtfSizeT,
            current_child_break_token,
        )
    }

    // cpp: layoutng_flex/flex_item_iterator.cc:110-146
    fn FindNextItem(&mut self, item_break_token: *const BlockBreakToken) -> *mut FlexItemData {
        while self.flex_line_idx_ < unsafe { &*self.flex_lines_ }.len() {
            let flex_line = &unsafe { &*self.flex_lines_ }[self.flex_line_idx_];
            if !flex_line.has_seen_all_children || !item_break_token.is_null() {
                while self.flex_item_idx_ < flex_line.line_items_data.len() {
                    let flex_item = &flex_line.line_items_data[self.flex_item_idx_]
                        as *const FlexItemData
                        as *mut FlexItemData;
                    self.flex_item_idx_ += 1;
                    if item_break_token.is_null()
                        || unsafe { &*flex_item }.block_node
                            == unsafe { &*item_break_token }.InputNode()
                    {
                        return flex_item;
                    }
                }
            }
            if self.is_column_
                && item_break_token.is_null()
                && self.flex_line_idx_ == self.next_item_idx_for_line_.len().wrapping_sub(1)
            {
                break;
            }
            self.flex_line_idx_ += 1;
            self.AdjustItemIndexForNewLine();
        }
        if !item_break_token.is_null() {
            debug_assert!(self.is_column_);
            self.flex_line_idx_ = 0;
            self.flex_item_idx_ = self.next_item_idx_for_line_[0];
            return self.FindNextItem(item_break_token);
        }
        std::ptr::null_mut()
    }

    // cpp: layoutng_flex/flex_item_iterator.cc:148-155
    pub fn NextLine(&mut self) {
        if self.flex_item_idx_ == 0 {
            return;
        }
        self.flex_line_idx_ += 1;
        self.AdjustItemIndexForNewLine();
        if self.break_token_.is_null() {
            self.next_unstarted_item_ = self.FindNextItem(std::ptr::null());
        }
    }

    // cpp: layoutng_flex/flex_item_iterator.cc:157-162
    fn AdjustItemIndexForNewLine(&mut self) {
        self.flex_item_idx_ = self
            .next_item_idx_for_line_
            .get(self.flex_line_idx_)
            .copied()
            .unwrap_or(0);
    }

    // cpp: layoutng_flex/flex_item_iterator.h:45
    pub fn HasMoreBreakTokens(&self) -> bool {
        !self.break_token_.is_null()
    }

    // cpp: layoutng_flex/flex_item_iterator.h:49-52
    pub fn HasNextItemInLine(&self, line_idx: WtfSizeT) -> bool {
        debug_assert!((line_idx as usize) < unsafe { &*self.flex_lines_ }.len());
        line_idx as usize == self.flex_line_idx_ && !self.next_unstarted_item_.is_null()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flex_line::FlexLine;
    use foundation::LayoutUnit;
    use layoutng_assembly::internal::block_node::BlockNode;
    use layoutng_geometry::geometry::logical_offset::LogicalOffset;
    use layoutng_style::style::computed_style_constants::ItemPosition;

    fn line(indices: &[u32]) -> FlexLine {
        let mut result = FlexLine::new(
            indices.to_vec(),
            LayoutUnit::default(),
            LayoutUnit::from_signed(10),
            LayoutUnit::default(),
            LayoutUnit::default(),
            0,
        );
        for index in indices {
            result.line_items_data.push(FlexItemData::new(
                BlockNode::null(),
                *index,
                LogicalOffset::default(),
                ItemPosition::kNormal,
                LayoutUnit::from_signed(10),
                LayoutUnit::default(),
                LayoutUnit::default(),
                false,
                false,
                false,
            ));
        }
        result
    }

    #[test]
    fn traverses_items_in_line_order_without_break_token() {
        let mut lines = FlexLineVector::new();
        lines.push(line(&[0, 1]));
        lines.push(line(&[2]));
        let mut iterator = FlexItemIterator::new(&lines, std::ptr::null(), false);
        let first = iterator.NextItem(false);
        let second = iterator.NextItem(false);
        let third = iterator.NextItem(false);
        let end = iterator.NextItem(false);
        assert_eq!((first.flex_line_idx, first.flex_item_idx), (0, 0));
        assert_eq!((second.flex_line_idx, second.flex_item_idx), (0, 1));
        assert_eq!((third.flex_line_idx, third.flex_item_idx), (1, 0));
        assert!(
            !first.flex_item.is_null() && !second.flex_item.is_null() && !third.flex_item.is_null()
        );
        assert!(end.flex_item.is_null());
    }
}
