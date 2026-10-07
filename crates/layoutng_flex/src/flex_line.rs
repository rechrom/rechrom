#![allow(non_snake_case)]

use foundation::{HeapVector, LayoutUnit, Traceable, Visitor, WtfSizeT};
use layoutng_assembly::internal::block_node::BlockNode;
use layoutng_geometry::geometry::logical_offset::LogicalOffset;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::computed_style_constants::ItemPosition;

// cpp: layoutng_flex/flex_line.h:19-63
#[derive(Clone)]
pub struct FlexItemData {
    pub block_node: BlockNode,
    pub item_index: WtfSizeT,
    pub offset: LogicalOffset,
    pub alignment: ItemPosition,
    pub main_axis_final_size: LayoutUnit,
    pub margin_block_end: LayoutUnit,
    pub total_remaining_block_size: LayoutUnit,
    pub is_initial_block_size_indefinite: bool,
    pub is_used_flex_basis_indefinite: bool,
    pub has_descendant_that_depends_on_percentage_block_size: bool,
}

impl FlexItemData {
    // cpp: layoutng_flex/flex_line.h:24-45
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        block_node: BlockNode,
        item_index: WtfSizeT,
        offset: LogicalOffset,
        alignment: ItemPosition,
        main_axis_final_size: LayoutUnit,
        margin_block_end: LayoutUnit,
        total_remaining_block_size: LayoutUnit,
        is_initial_block_size_indefinite: bool,
        is_used_flex_basis_indefinite: bool,
        has_descendant_that_depends_on_percentage_block_size: bool,
    ) -> Self {
        Self {
            block_node,
            item_index,
            offset,
            alignment,
            main_axis_final_size,
            margin_block_end,
            total_remaining_block_size,
            is_initial_block_size_indefinite,
            is_used_flex_basis_indefinite,
            has_descendant_that_depends_on_percentage_block_size,
        }
    }

    // cpp: layoutng_flex/flex_line.h:47
    pub fn Style(&self) -> &ComputedStyle {
        unsafe { &*self.block_node.GetLayoutBox() }.StyleRef()
    }

    // cpp: layoutng_flex/flex_line.h:49
    pub fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.block_node.GetLayoutBox());
    }
}

impl Traceable for FlexItemData {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        FlexItemData::Trace(self, visitor);
    }
}

// cpp: layoutng_flex/flex_line.h:70-116
#[derive(Clone)]
pub struct FlexLine {
    pub item_indices: Vec<WtfSizeT>,
    pub main_axis_free_space: LayoutUnit,
    pub line_cross_size: LayoutUnit,
    pub major_baseline: LayoutUnit,
    pub minor_baseline: LayoutUnit,
    pub main_axis_auto_margin_count: u32,
    pub cross_axis_offset: LayoutUnit,
    pub effective_gap_between_items: LayoutUnit,
    pub item_offset_adjustment: LayoutUnit,
    pub has_seen_all_children: bool,
    pub line_items_data: HeapVector<FlexItemData>,
}

impl FlexLine {
    // cpp: layoutng_flex/flex_line.h:73-86
    pub fn new(
        item_indices: Vec<WtfSizeT>,
        main_axis_free_space: LayoutUnit,
        line_cross_size: LayoutUnit,
        major_baseline: LayoutUnit,
        minor_baseline: LayoutUnit,
        main_axis_auto_margin_count: u32,
    ) -> Self {
        Self {
            item_indices,
            main_axis_free_space,
            line_cross_size,
            major_baseline,
            minor_baseline,
            main_axis_auto_margin_count,
            cross_axis_offset: LayoutUnit::default(),
            effective_gap_between_items: LayoutUnit::default(),
            item_offset_adjustment: LayoutUnit::default(),
            has_seen_all_children: false,
            line_items_data: HeapVector::default(),
        }
    }

    // cpp: layoutng_flex/flex_line.h:88-90
    pub fn LineCrossEnd(&self) -> LayoutUnit {
        self.line_cross_size + self.cross_axis_offset + self.item_offset_adjustment
    }

    // cpp: layoutng_flex/flex_line.h:92
    pub fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.line_items_data);
    }
}

impl Traceable for FlexLine {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        FlexLine::Trace(self, visitor);
    }
}

// cpp: layoutng_flex/flex_line.h:119
pub type FlexLineVector = HeapVector<FlexLine, 1>;
