#![allow(non_snake_case)]
use foundation::{DowncastFrom, LayoutUnit, Traceable, Visitor};
use layoutng_assembly::break_token_algorithm_data::{
    BreakTokenAlgorithmData, BreakTokenAlgorithmDataType, BreakTokenAlgorithmDataVirtual,
};
// cpp: layoutng_table/table_row_break_token_data.h:12-21
#[repr(C)]
pub struct TableRowBreakTokenData {
    base: BreakTokenAlgorithmData,
    pub previous_consumed_row_block_size: LayoutUnit,
}
impl TableRowBreakTokenData {
    pub fn new(previous_consumed_row_block_size: LayoutUnit) -> Self {
        Self {
            base: BreakTokenAlgorithmData::new(BreakTokenAlgorithmDataType::kTableRowData),
            previous_consumed_row_block_size,
        }
    }
}
const _: () = assert!(std::mem::offset_of!(TableRowBreakTokenData, base) == 0);
// cpp: layoutng_table/table_row_break_token_data.h:23-28
impl DowncastFrom<BreakTokenAlgorithmData> for TableRowBreakTokenData {
    fn AllowFrom(data: &BreakTokenAlgorithmData) -> bool {
        data.IsTableRowType()
    }
}
impl BreakTokenAlgorithmDataVirtual for TableRowBreakTokenData {
    fn base(&self) -> &BreakTokenAlgorithmData {
        &self.base
    }
}
impl Traceable for TableRowBreakTokenData {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        BreakTokenAlgorithmDataVirtual::Trace(self, visitor);
    }
}
