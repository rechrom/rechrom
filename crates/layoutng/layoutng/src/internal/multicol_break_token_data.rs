use std::ops::Deref;

use foundation::LayoutUnit;
use layoutng_fragment_tree::break_token_algorithm_data::{
    BreakTokenAlgorithmData, BreakTokenAlgorithmDataType, BreakTokenAlgorithmDataVirtual,
};

// cpp: layoutng/internal/multicol_break_token_data.h:12-21
pub struct MulticolBreakTokenData {
    base: BreakTokenAlgorithmData,
    pub consumed_row_block_size: LayoutUnit,
}

#[allow(non_snake_case)]
impl MulticolBreakTokenData {
    // cpp: layoutng/internal/multicol_break_token_data.h:13-15
    pub fn new(consumed_row_block_size: LayoutUnit) -> Self {
        Self {
            base: BreakTokenAlgorithmData::new(BreakTokenAlgorithmDataType::kMulticolData),
            consumed_row_block_size,
        }
    }

    // cpp: layoutng/internal/multicol_break_token_data.h:23-28
    pub fn AllowFrom(token_data: &BreakTokenAlgorithmData) -> bool {
        token_data.IsMulticolType()
    }
}

impl Deref for MulticolBreakTokenData {
    type Target = BreakTokenAlgorithmData;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl BreakTokenAlgorithmDataVirtual for MulticolBreakTokenData {
    fn base(&self) -> &BreakTokenAlgorithmData {
        &self.base
    }
}
