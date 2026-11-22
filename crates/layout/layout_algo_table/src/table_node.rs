#![allow(non_snake_case)]
#[allow(unused_imports)]
use crate::algorithms::*;
use layout_base::internal::table_node::TableNode;

// cpp: layoutng_table/table_node.cc:60-81
pub fn TableNodeAllowColumnPercentagesFromTable(
    node: &TableNode,
    layout_pass: bool,
) -> bool {
    if node.Style().LogicalWidth().HasMaxContent() {
        return false;
    }
    if layout_pass {
        return true;
    }
    let mut block = unsafe { &*node.GetLayoutBox() }.ContainingBlock();
    while !unsafe { &*block }.IsLayoutView() {
        let object = unsafe { &*block };
        if object.IsTableCell() || object.IsFlexibleBox() || object.IsLayoutGridOrGridLanes() {
            return false;
        }
        block = object.ContainingBlock();
    }
    true
}

// cpp: layoutng_table/table_node.cc:13-30
pub fn TableNodeGetTableBordersFromTable(
    node: &TableNode,
) -> *const layout_base::internal::table_borders::TableBorders {
    let _trace = browser_tracing::span("layout", "layout_algo_table.ResolveBorders");
    use crate::layout_table::LayoutTable;
    use foundation::To;
    use layout_base::internal::table_borders::TableBorders;
    let table = unsafe { &mut *To::<LayoutTable>(node.GetLayoutBox()) };
    let mut borders = table.GetCachedTableBorders();
    if borders.is_null() {
        borders = TableBorders::ComputeTableBorders(node);
        table.SetCachedTableBorders(borders);
    } else {
        #[cfg(debug_assertions)]
        {
            let duplicate = TableBorders::ComputeTableBorders(node);
            debug_assert!(unsafe { &*duplicate }.Equals(unsafe { &*borders }));
        }
    }
    borders
}
// cpp: layoutng_table/table_node.cc:32-34
pub fn TableNodeGetTableBordersStrutFromTable(
    node: &TableNode,
) -> *const layout_geometry::geometry::box_strut::BoxStrut {
    unsafe { &*TableNodeGetTableBordersFromTable(node) }.TableBorder()
}
// cpp: layoutng_table/table_node.cc:36-49
pub fn TableNodeGetColumnConstraintsFromTable(
    node: &TableNode,
    grouped: &layout_base::internal::table_layout_algorithm_types::TableGroupedChildren,
    border_padding: &layout_geometry::geometry::box_strut::BoxStrut,
) -> Option<std::sync::Arc<layout_base::internal::table_layout_algorithm_types::Columns>> {
    let _trace = browser_tracing::span("layout", "layout_algo_table.ResolveColumnConstraints");
    let table =
        unsafe { &mut *foundation::To::<crate::layout_table::LayoutTable>(node.GetLayoutBox()) };
    let mut columns = table.CloneCachedTableColumnConstraints();
    if columns.is_none() {
        columns = Some(crate::table_layout_utils::ComputeColumnConstraints(
            node,
            grouped,
            unsafe { &*node.GetTableBorders() },
            border_padding,
        ));
        table.SetCachedTableColumnConstraints(columns.clone());
    }
    columns
}

// cpp: layoutng_table/table_node.cc:47-51
pub fn TableNodeComputeTableInlineSizeFromTable(
    node: &TableNode,
    space: &layout_base::internal::constraint_space::ConstraintSpace,
    border_padding: &layout_geometry::geometry::box_strut::BoxStrut,
) -> foundation::LayoutUnit {
    let _trace = browser_tracing::span("layout", "layout_algo_table.ComputeInlineSize");
    crate::table_layout_support::ComputeTableInlineSize(node, space, border_padding)
}
// cpp: layoutng_table/table_node.cc:53-58
pub fn TableNodeComputeCaptionBlockSizeFromTable(
    node: &TableNode,
    space: &layout_base::internal::constraint_space::ConstraintSpace,
) -> foundation::LayoutUnit {
    let _trace = browser_tracing::span("layout", "layout_algo_table.ComputeCaptionBlockSize");
    let geometry = layout_base::internal::length_utils::CalculateInitialFragmentGeometry(
        space,
        node,
        std::ptr::null(),
        false,
    );
    crate::table_layout_support::ComputeTableCaptionBlockSize(node, &geometry, space)
}
