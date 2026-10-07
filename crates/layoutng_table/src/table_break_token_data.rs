#![allow(non_snake_case)]
use foundation::{DowncastFrom, LayoutUnit, Traceable, Visitor};
use layoutng_assembly::{
    break_token_algorithm_data::{
        BreakTokenAlgorithmData, BreakTokenAlgorithmDataType, BreakTokenAlgorithmDataVirtual,
    },
    internal::table_layout_algorithm_types::{CellBlockConstraints, Rows, Sections},
};

// cpp: layoutng_table/table_break_token_data.h:13-47
#[repr(C)]
pub struct TableBreakTokenData {
    base: BreakTokenAlgorithmData,
    pub rows: Rows,
    pub cell_block_constraints: CellBlockConstraints,
    pub sections: Sections,
    pub total_table_min_block_size: LayoutUnit,
    pub consumed_table_box_block_size: LayoutUnit,
    pub has_entered_table_box: bool,
    pub is_past_table_box: bool,
}
impl TableBreakTokenData {
    pub fn new(
        rows: &Rows,
        cell_block_constraints: &CellBlockConstraints,
        sections: &Sections,
        total_table_min_block_size: LayoutUnit,
        consumed_table_box_block_size: LayoutUnit,
        has_entered_table_box: bool,
        is_past_table_box: bool,
    ) -> Self {
        Self {
            base: BreakTokenAlgorithmData::new(BreakTokenAlgorithmDataType::kTableData),
            rows: rows.clone(),
            cell_block_constraints: cell_block_constraints.clone(),
            sections: sections.clone(),
            total_table_min_block_size,
            consumed_table_box_block_size,
            has_entered_table_box,
            is_past_table_box,
        }
    }
}
const _: () = assert!(std::mem::offset_of!(TableBreakTokenData, base) == 0);
// cpp: layoutng_table/table_break_token_data.h:49-54
impl DowncastFrom<BreakTokenAlgorithmData> for TableBreakTokenData {
    fn AllowFrom(data: &BreakTokenAlgorithmData) -> bool {
        data.IsTableType()
    }
}
impl BreakTokenAlgorithmDataVirtual for TableBreakTokenData {
    fn base(&self) -> &BreakTokenAlgorithmData {
        &self.base
    }
}
impl Traceable for TableBreakTokenData {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        BreakTokenAlgorithmDataVirtual::Trace(self, visitor);
    }
}
