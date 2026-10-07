#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};

use foundation::{DynamicTo, LogicalToPhysical, Traceable, Visitor, WtfSizeT};
use layoutng_assembly::internal::devtools_flex_info::DevtoolsFlexInfo;
use layoutng_assembly::internal::fragmentation_utils::FindPreviousBreakToken;
use layoutng_assembly::internal::gap::gap_geometry::GapGeometry;
use layoutng_assembly::internal::layout_block::LayoutBlock;
use layoutng_assembly::internal::layout_invalidation_reason;
use layoutng_assembly::internal::layout_node_metadata::Element;
use layoutng_assembly::internal::layout_object::{LayoutObject, LayoutObjectClass};
use layoutng_assembly::physical_box_fragment::PhysicalBoxFragment;
use layoutng_assembly::RegisterFlexStitchedRowGapIndex;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::grid_enums::GridTrackSizingDirection;

use crate::flex_break_token_data::FlexBreakTokenData;

// cpp: layoutng_flex/layout_flexible_box.h:28-61
#[repr(C)]
pub struct LayoutFlexibleBox {
    block: LayoutBlock,
}

impl LayoutFlexibleBox {
    // cpp: layoutng_flex/layout_flexible_box.cc:16
    pub fn new(element: *mut Element) -> Self {
        RegisterFlexStitchedRowGapIndex(DispatchFlexStitchedRowGapIndex);
        let block = LayoutBlock::new(element.cast());
        block.SetRuntimeClass(LayoutObjectClass::FlexibleBox);
        Self { block }
    }

    // cpp: layoutng_flex/layout_flexible_box.h:42-46
    pub fn GetName(&self) -> &'static str {
        self.CheckIsNotDestroyed();
        "LayoutFlexibleBox"
    }

    // cpp: layoutng_flex/layout_flexible_box.h:57-60
    pub fn IsFlexibleBox(&self) -> bool {
        self.CheckIsNotDestroyed();
        true
    }

    // cpp: layoutng_flex/layout_flexible_box.cc:76-78
    pub fn HasTopOverflow(&self) -> bool {
        GetOverflowConverter(self.StyleRef()).Top()
    }

    // cpp: layoutng_flex/layout_flexible_box.cc:80-82
    pub fn HasLeftOverflow(&self) -> bool {
        GetOverflowConverter(self.StyleRef()).Left()
    }

    // cpp: layoutng_flex/layout_flexible_box.cc:88-91
    pub fn SetNeedsLayoutForDevtools(&mut self) {
        self.SetNeedsLayout(&raw const layout_invalidation_reason::kDevtools);
        self.SetNeedsDevtoolsInfo(true);
    }

    // cpp: layoutng_flex/layout_flexible_box.cc:93-98
    pub fn FlexLayoutData(&self) -> *const DevtoolsFlexInfo {
        let fragment_count = self.PhysicalFragmentCount();
        debug_assert!(fragment_count >= 1);
        unsafe { &*self.GetLayoutResult(0) }.FlexLayoutData()
    }

    // cpp: layoutng_flex/layout_flexible_box.cc:100-146
    pub fn StitchedRowGapIndex(
        &self,
        fragment: &PhysicalBoxFragment,
        mut gap_index: WtfSizeT,
        absolute_flex_line_index: Option<WtfSizeT>,
    ) -> WtfSizeT {
        self.CheckIsNotDestroyed();
        assert!(!fragment.IsOnlyForNode());
        let gap_geometry = fragment.GetGapGeometry();
        if gap_geometry.is_null() {
            return gap_index;
        }
        if let Some(line_index) = absolute_flex_line_index {
            gap_index =
                GapIndexWithinColumnFlexLine(unsafe { &*gap_geometry }, gap_index, line_index);
        }
        let outgoing_break_token = fragment.GetBreakToken();
        if !outgoing_break_token.is_null() {
            let flex_data =
                DynamicTo::<FlexBreakTokenData>(unsafe { &*outgoing_break_token }.TokenData());
            if !flex_data.is_null() && !unsafe { &*flex_data }.gap_data.gap_data_for_rows.is_empty()
            {
                return unsafe { &*flex_data }
                    .GetFirstUnprocessedRowGapIndex(absolute_flex_line_index)
                    + gap_index;
            }
        } else {
            let previous_break_token = FindPreviousBreakToken(fragment);
            if !previous_break_token.is_null() {
                let flex_data =
                    DynamicTo::<FlexBreakTokenData>(unsafe { &*previous_break_token }.TokenData());
                if !flex_data.is_null() {
                    let row_data = &unsafe { &*flex_data }.gap_data.gap_data_for_rows;
                    let line = absolute_flex_line_index.unwrap_or(0) as usize;
                    if line < row_data.len() {
                        return row_data[line].first_row_gap_index
                            + row_data[line].row_gap_count
                            + gap_index;
                    }
                }
            }
        }
        gap_index
    }
}

// C++ virtual dispatch calls LayoutFlexibleBox even when the caller holds a
// LayoutObject pointer. LayoutFlexibleBox is base-first and repr(C).
// cpp: layoutng_flex/layout_flexible_box.cc:100-146
fn DispatchFlexStitchedRowGapIndex(
    object: *const LayoutObject,
    fragment: &PhysicalBoxFragment,
    gap_index: usize,
    line_index: Option<usize>,
) -> usize {
    let flex = unsafe { &*object.cast::<LayoutFlexibleBox>() };
    flex.StitchedRowGapIndex(
        fragment,
        WtfSizeT::try_from(gap_index).expect("gap index exceeds wtf_size_t"),
        line_index.map(|index| WtfSizeT::try_from(index).expect("line index exceeds wtf_size_t")),
    ) as usize
}

impl Deref for LayoutFlexibleBox {
    type Target = LayoutBlock;
    fn deref(&self) -> &Self::Target {
        &self.block
    }
}

impl DerefMut for LayoutFlexibleBox {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.block
    }
}

impl Traceable for LayoutFlexibleBox {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.block.Trace(visitor);
    }
}

// cpp: layoutng_flex/layout_flexible_box.cc:20-47
fn GetOverflowConverter(style: &ComputedStyle) -> LogicalToPhysical<bool> {
    let is_wrap_reverse = style.ResolvedIsFlexWrapReverse();
    let is_direction_reverse = style.ResolvedIsReverseFlexDirection();
    let (mut inline_start, mut inline_end, mut block_start, mut block_end) =
        (false, true, false, true);
    if style.ResolvedIsColumnFlexDirection() {
        if is_direction_reverse {
            std::mem::swap(&mut block_start, &mut block_end);
        }
        if is_wrap_reverse {
            std::mem::swap(&mut inline_start, &mut inline_end);
        }
    } else {
        if is_direction_reverse {
            std::mem::swap(&mut inline_start, &mut inline_end);
        }
        if is_wrap_reverse {
            std::mem::swap(&mut block_start, &mut block_end);
        }
    }
    LogicalToPhysical::new(
        style.GetWritingDirection(),
        inline_start,
        inline_end,
        block_start,
        block_end,
    )
}

// cpp: layoutng_flex/layout_flexible_box.cc:51-72
fn GapIndexWithinColumnFlexLine(
    gap_geometry: &GapGeometry,
    gap_index: WtfSizeT,
    absolute_flex_line_index: WtfSizeT,
) -> WtfSizeT {
    assert!(gap_geometry.IsMainDirection(GridTrackSizingDirection::kForColumns));
    let main_gaps = gap_geometry.GetMainGaps();
    let count = main_gaps.len();
    if (absolute_flex_line_index as usize) < count {
        let before = main_gaps[absolute_flex_line_index as usize].GetCrossGapBeforeStart();
        debug_assert!(gap_index >= before);
        return gap_index - before;
    }
    if count > 0 && main_gaps[count - 1].HasCrossGapsAfter() {
        let after = main_gaps[count - 1].GetCrossGapAfterStart();
        debug_assert!(gap_index >= after);
        return gap_index - after;
    }
    gap_index
}
