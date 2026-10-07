use foundation::{GCedHeapVector, LayoutUnit, Visitor};

use super::layout_input_node::LayoutInputNode;

// cpp: layoutng/internal/table_fragment_data.h:16-38
#[derive(Clone)]
pub struct TableColumnGeometry {
    pub start_column: u32,
    pub span: u32,
    pub inline_offset: LayoutUnit,
    pub inline_size: LayoutUnit,
    pub node: LayoutInputNode,
}

impl PartialEq for TableColumnGeometry {
    fn eq(&self, other: &Self) -> bool {
        self.start_column == other.start_column
            && self.span == other.span
            && self.inline_offset == other.inline_offset
            && self.inline_size == other.inline_size
            && self.node == other.node
    }
}

#[allow(non_snake_case)]
impl TableColumnGeometry {
    // cpp: layoutng/internal/table_fragment_data.h:22-31
    pub fn new(
        start_column: u32,
        span: u32,
        inline_offset: LayoutUnit,
        inline_size: LayoutUnit,
        node: LayoutInputNode,
    ) -> Self {
        Self {
            start_column,
            span,
            inline_offset,
            inline_size,
            node,
        }
    }

    // cpp: layoutng/internal/table_fragment_data.h:32-32
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.node);
    }
}

// cpp: layoutng/internal/table_fragment_data.h:40-41
pub type TableColumnGeometries = Vec<TableColumnGeometry>;
pub type GCedTableColumnGeometries = GCedHeapVector<TableColumnGeometry>;

// cpp: layoutng/internal/table_fragment_data.h:43-56
#[derive(Clone, Default)]
pub struct CollapsedTableBordersGeometry {
    pub columns: Vec<LayoutUnit>,
}

#[allow(non_snake_case)]
impl CollapsedTableBordersGeometry {
    // cpp: layoutng/internal/table_fragment_data.h:50-55
    pub fn CheckSameForSimplifiedLayout(&self, other: &Self) {
        debug_assert!(self.columns == other.columns);
    }
}
