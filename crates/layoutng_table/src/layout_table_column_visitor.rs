#![allow(non_snake_case)]
use foundation::HeapVector;
use layoutng_assembly::internal::block_node::BlockNode;

// cpp: layoutng_table/layout_table_column_visitor.h:22-32
pub trait ColumnVisitor {
    fn VisitCol(&mut self, column: &BlockNode, start_column_index: u32, span: u32);
    fn EnterColgroup(&mut self, colgroup: &BlockNode, start_column_index: u32);
    fn LeaveColgroup(
        &mut self,
        colgroup: &BlockNode,
        start_column_index: u32,
        span: u32,
        has_children: bool,
    );
}

// cpp: layoutng_table/layout_table_column_visitor.h:33-79
pub fn VisitLayoutTableColumn(
    columns: &HeapVector<BlockNode>,
    table_column_count: u32,
    visitor: &mut impl ColumnVisitor,
) {
    fn visit(
        column: &BlockNode,
        count: u32,
        current: &mut u32,
        visitor: &mut impl ColumnVisitor,
    ) -> u32 {
        let span = column.TableColumnSpan().min(count.wrapping_sub(*current));
        visitor.VisitCol(column, *current, span);
        *current = current.wrapping_add(span);
        span
    }
    let mut current = 0;
    for column in columns {
        if current >= table_column_count {
            break;
        }
        if column.IsTableCol() {
            visit(column, table_column_count, &mut current, visitor);
            continue;
        }
        debug_assert!(column.IsTableColgroup());
        visitor.EnterColgroup(column, current);
        let mut child = BlockNode::from(column.FirstChild());
        let start = current;
        let mut span = 0_u32;
        let has_children = child.is_non_null();
        if has_children {
            while child.is_non_null() {
                span = span.wrapping_add(visit(&child, table_column_count, &mut current, visitor));
                if current >= table_column_count {
                    break;
                }
                child = BlockNode::from(child.NextSibling());
            }
        } else {
            span = column
                .TableColumnSpan()
                .min(table_column_count.wrapping_sub(current));
            current = current.wrapping_add(span);
        }
        visitor.LeaveColgroup(column, start, span, has_children);
    }
}
