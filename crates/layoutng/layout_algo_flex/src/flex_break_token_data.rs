#![allow(non_snake_case)]

use foundation::{
    DowncastFrom, EBreakBetween, HeapVector, LayoutUnit, Member, Traceable, Visitor, WtfSizeT,
};
use layoutng_assembly::break_token_algorithm_data::{
    BreakTokenAlgorithmData, BreakTokenAlgorithmDataType, BreakTokenAlgorithmDataVirtual,
};
use layoutng_assembly::internal::layout_box::LayoutBox;

use crate::flex_line::FlexLineVector;

// cpp: layoutng_flex/flex_break_token_data.h:17-24
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FlexRowGapBreakTokenData {
    pub first_row_gap_index: WtfSizeT,
    pub row_gap_count: WtfSizeT,
}

// cpp: layoutng_flex/flex_break_token_data.h:14-49
#[derive(Clone, Default)]
pub struct FlexGapBreakTokenData {
    pub effective_gap_between_lines: LayoutUnit,
    pub total_row_gap_count: WtfSizeT,
    pub gap_data_for_rows: Vec<FlexRowGapBreakTokenData>,
}

// cpp: layoutng_flex/flex_break_token_data.h:54-64
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FlexBreakBeforeRow {
    #[default]
    kNotBreakBeforeRow,
    kAtStartOfBreakBeforeRow,
    kPastStartOfBreakBeforeRow,
}

// cpp: layoutng_flex/flex_break_token_data.h:51-120
#[repr(C)]
pub struct FlexBreakTokenData {
    base: BreakTokenAlgorithmData,
    pub flex_lines: FlexLineVector,
    pub row_break_between: Vec<EBreakBetween>,
    pub oof_children: HeapVector<Member<LayoutBox>>,
    pub intrinsic_block_size: LayoutUnit,
    pub break_before_row: FlexBreakBeforeRow,
    pub gap_data: FlexGapBreakTokenData,
}

impl FlexBreakTokenData {
    // cpp: layoutng_flex/flex_break_token_data.h:66-82
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        flex_lines: &FlexLineVector,
        row_break_between: &[EBreakBetween],
        oof_children: &HeapVector<Member<LayoutBox>>,
        intrinsic_block_size: LayoutUnit,
        break_before_row: FlexBreakBeforeRow,
        gap_data: FlexGapBreakTokenData,
    ) -> Self {
        Self {
            base: BreakTokenAlgorithmData::new(BreakTokenAlgorithmDataType::kFlexData),
            flex_lines: flex_lines.clone(),
            row_break_between: row_break_between.to_vec(),
            oof_children: oof_children.clone(),
            intrinsic_block_size,
            break_before_row,
            gap_data,
        }
    }

    // cpp: layoutng_flex/flex_break_token_data.h:84-88
    pub fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.flex_lines);
        visitor.Trace(&self.oof_children);
        BreakTokenAlgorithmDataVirtual::Trace(&self.base, visitor);
    }

    // cpp: layoutng_flex/flex_break_token_data.h:90-92
    pub fn GetTotalRowGapCount(&self) -> WtfSizeT {
        self.gap_data.total_row_gap_count
    }

    // cpp: layoutng_flex/flex_break_token_data.h:94-103
    pub fn GetFirstUnprocessedRowGapIndex(&self, line_index: Option<WtfSizeT>) -> WtfSizeT {
        assert!(!self.gap_data.gap_data_for_rows.is_empty());
        let line = line_index.unwrap_or(0) as usize;
        assert!(line < self.gap_data.gap_data_for_rows.len());
        self.gap_data.gap_data_for_rows[line].first_row_gap_index
    }
}

const _: () = assert!(std::mem::offset_of!(FlexBreakTokenData, base) == 0);

impl BreakTokenAlgorithmDataVirtual for FlexBreakTokenData {
    fn base(&self) -> &BreakTokenAlgorithmData {
        &self.base
    }
    fn GetTotalRowGapCount(&self) -> u32 {
        FlexBreakTokenData::GetTotalRowGapCount(self)
    }
    fn GetFirstUnprocessedRowGapIndex(&self, line_index: Option<u32>) -> u32 {
        FlexBreakTokenData::GetFirstUnprocessedRowGapIndex(self, line_index)
    }
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        FlexBreakTokenData::Trace(self, visitor);
    }
}

impl DowncastFrom<BreakTokenAlgorithmData> for FlexBreakTokenData {
    // cpp: layoutng_flex/flex_break_token_data.h:122-126
    fn AllowFrom(data: &BreakTokenAlgorithmData) -> bool {
        data.IsFlexType()
    }
}

impl Traceable for FlexBreakTokenData {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        FlexBreakTokenData::Trace(self, visitor);
    }
}
