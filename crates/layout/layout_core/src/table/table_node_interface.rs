#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};

use super::super::block_node::BlockNode;
use super::super::layout_box::LayoutBox;
use super::super::layout_input_node::LayoutInputNode;

// The non-inline members (GetTableBordersStrut, GetTableBorders,
// ComputeCaptionBlockSize, GetColumnConstraints, ComputeTableInlineSize,
// AllowColumnPercentages) are table algorithms owned by //src/layoutng_table
// (Chromium core/layout/table/table_node.cc). Only the table package calls
// them, so they are its `TableNodeAlgorithms` extension trait
// (`layout_algo_table::algorithms`). Base reaches the three it needs during
// layout through the `TableLayoutSupport` registry slots.

// cpp: layoutng/internal/table_node.h:15-18
// TableNode adds no data; the BlockNode base remains at offset zero.
#[repr(C)]
pub struct TableNode {
    pub base: BlockNode,
}

impl TableNode {
    // cpp: layoutng/internal/table_node.h:18-18
    pub fn new(box_: *mut LayoutBox) -> Self {
        Self {
            base: BlockNode::new(box_),
        }
    }

    // cpp: layoutng/internal/table_node.h:44-47
    pub fn AllowFrom(node: &LayoutInputNode) -> bool {
        node.IsTable()
    }
}

impl Clone for TableNode {
    fn clone(&self) -> Self {
        Self::new(self.GetLayoutBox())
    }
}

impl Deref for TableNode {
    type Target = BlockNode;

    fn deref(&self) -> &BlockNode {
        &self.base
    }
}

impl DerefMut for TableNode {
    fn deref_mut(&mut self) -> &mut BlockNode {
        &mut self.base
    }
}
