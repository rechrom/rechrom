#![allow(non_snake_case)]

use foundation::{DowncastFrom, LayoutUnit, Traceable, Visitor};
use layoutng_assembly::break_token_algorithm_data::{
    BreakTokenAlgorithmData, BreakTokenAlgorithmDataType, BreakTokenAlgorithmDataVirtual,
};

// cpp: layoutng_forms/fieldset_break_token_data.h:12-16
#[repr(C)]
pub struct FieldsetBreakTokenData {
    base: BreakTokenAlgorithmData,
    pub legend_block_size_contribution: LayoutUnit,
}

impl Default for FieldsetBreakTokenData {
    fn default() -> Self {
        Self::new()
    }
}

impl FieldsetBreakTokenData {
    pub fn new() -> Self {
        Self {
            base: BreakTokenAlgorithmData::new(BreakTokenAlgorithmDataType::kFieldsetData),
            legend_block_size_contribution: LayoutUnit::new(),
        }
    }
}

const _: () = assert!(std::mem::offset_of!(FieldsetBreakTokenData, base) == 0);

impl BreakTokenAlgorithmDataVirtual for FieldsetBreakTokenData {
    fn base(&self) -> &BreakTokenAlgorithmData {
        &self.base
    }
}

// cpp: layoutng_forms/fieldset_break_token_data.h:18-23
impl DowncastFrom<BreakTokenAlgorithmData> for FieldsetBreakTokenData {
    fn AllowFrom(data: &BreakTokenAlgorithmData) -> bool {
        data.IsFieldsetType()
    }
}

impl Traceable for FieldsetBreakTokenData {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        BreakTokenAlgorithmDataVirtual::Trace(self, visitor);
    }
}
