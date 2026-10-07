#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use foundation::LayoutUnit;
use layoutng_geometry::geometry::box_strut::BoxStrut;

use super::block_node::BlockNode;
use super::constraint_space::ConstraintSpace;
use super::layout_box::LayoutBox;
use super::layout_input_node::LayoutInputNode;
use super::table_borders::TableBorders;
use super::table_layout_algorithm_types::{Columns, TableGroupedChildren};

// The method bodies are owned by //src/layoutng_table/table_node.cc. Their
// dependencies on LayoutTable and the table algorithm remain external to this
// Bazel package; these typed links preserve the class interface here.
unsafe extern "Rust" {
    fn TableNodeGetTableBordersStrutFromTable(node: &TableNode) -> *const BoxStrut;
    fn TableNodeGetTableBordersFromTable(node: &TableNode) -> *const TableBorders;
    fn TableNodeComputeCaptionBlockSizeFromTable(
        node: &TableNode,
        space: &ConstraintSpace,
    ) -> LayoutUnit;
    fn TableNodeGetColumnConstraintsFromTable(
        node: &TableNode,
        grouped_children: &TableGroupedChildren,
        border_padding: &BoxStrut,
    ) -> Option<Arc<Columns>>;
    fn TableNodeComputeTableInlineSizeFromTable(
        node: &TableNode,
        space: &ConstraintSpace,
        border_padding: &BoxStrut,
    ) -> LayoutUnit;
    fn TableNodeAllowColumnPercentagesFromTable(node: &TableNode, is_layout_pass: bool) -> bool;
}

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

    // cpp: layoutng/internal/table_node.h:20-22
    pub fn GetTableBordersStrut(&self) -> &BoxStrut {
        let borders = unsafe { TableNodeGetTableBordersStrutFromTable(self) };
        debug_assert!(!borders.is_null());
        unsafe { &*borders }
    }

    pub fn GetTableBorders(&self) -> *const TableBorders {
        unsafe { TableNodeGetTableBordersFromTable(self) }
    }

    // cpp: layoutng/internal/table_node.h:24-31
    pub fn ComputeCaptionBlockSize(&self, space: &ConstraintSpace) -> LayoutUnit {
        unsafe { TableNodeComputeCaptionBlockSizeFromTable(self, space) }
    }

    pub fn GetColumnConstraints(
        &self,
        grouped_children: &TableGroupedChildren,
        border_padding: &BoxStrut,
    ) -> Option<Arc<Columns>> {
        unsafe { TableNodeGetColumnConstraintsFromTable(self, grouped_children, border_padding) }
    }

    pub fn ComputeTableInlineSize(
        &self,
        space: &ConstraintSpace,
        border_padding: &BoxStrut,
    ) -> LayoutUnit {
        unsafe { TableNodeComputeTableInlineSizeFromTable(self, space, border_padding) }
    }

    // cpp: layoutng/internal/table_node.h:33-41
    pub fn AllowColumnPercentages(&self, is_layout_pass: bool) -> bool {
        unsafe { TableNodeAllowColumnPercentagesFromTable(self, is_layout_pass) }
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
