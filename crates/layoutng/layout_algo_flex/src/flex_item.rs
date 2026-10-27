#![allow(non_snake_case)]

use foundation::{
    LayoutUnit, TextDirection, Traceable, Visitor, WritingDirectionMode, WritingMode, WtfSizeT,
};
use layoutng_assembly::internal::baseline_utils::BaselineGroup;
use layoutng_assembly::internal::block_node::BlockNode;
use layoutng_assembly::internal::min_max_sizes::MinMaxSizes;
use layoutng_geometry::geometry::box_strut::{BoxStrut, PhysicalBoxStrut};
use layoutng_style::style::computed_style_constants::ItemPosition;

// cpp: layoutng_flex/flex_item.h:20
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlexerState {
    kNone,
    kMinViolation,
    kMaxViolation,
    kFrozen,
}

// cpp: layoutng_flex/flex_item.h:22-140
pub struct FlexItem {
    pub block_node: BlockNode,
    pub item_index: WtfSizeT,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub base_content_size: LayoutUnit,
    pub hypothetical_content_size: LayoutUnit,
    pub main_axis_min_max_sizes: MinMaxSizes,
    pub main_axis_border_padding: LayoutUnit,
    pub max_content_contribution: Option<LayoutUnit>,
    pub initial_margins: PhysicalBoxStrut,
    pub initial_scrollbars: BoxStrut,
    pub main_axis_auto_margin_count: u8,
    pub alignment: ItemPosition,
    pub baseline_writing_direction: WritingDirectionMode,
    pub baseline_group: BaselineGroup,
    pub is_initial_block_size_indefinite: bool,
    pub is_used_flex_basis_indefinite: bool,
    pub depends_on_min_max_sizes: bool,
    pub is_horizontal_flow: bool,
    pub free_space_fraction: f64,
    pub state: FlexerState,
    pub flexed_content_size: LayoutUnit,
}

impl FlexItem {
    // cpp: layoutng_flex/flex_item.h:27-75
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        block_node: BlockNode,
        item_index: WtfSizeT,
        flex_grow: f32,
        flex_shrink: f32,
        base_content_size: LayoutUnit,
        main_axis_min_max_sizes: MinMaxSizes,
        main_axis_border_padding: LayoutUnit,
        max_content_contribution: Option<LayoutUnit>,
        initial_margins: PhysicalBoxStrut,
        initial_scrollbars: BoxStrut,
        main_axis_auto_margin_count: u8,
        alignment: ItemPosition,
        baseline_writing_mode: WritingMode,
        baseline_group: BaselineGroup,
        is_initial_block_size_indefinite: bool,
        is_used_flex_basis_indefinite: bool,
        depends_on_min_max_sizes: bool,
        is_horizontal_flow: bool,
    ) -> Self {
        let hypothetical_content_size =
            main_axis_min_max_sizes.ClampSizeToMinAndMax(base_content_size);
        Self {
            block_node,
            item_index,
            flex_grow,
            flex_shrink,
            base_content_size,
            hypothetical_content_size,
            main_axis_min_max_sizes,
            main_axis_border_padding,
            max_content_contribution,
            initial_margins,
            initial_scrollbars,
            main_axis_auto_margin_count,
            alignment,
            baseline_writing_direction: WritingDirectionMode::new(
                baseline_writing_mode,
                TextDirection::kLtr,
            ),
            baseline_group,
            is_initial_block_size_indefinite,
            is_used_flex_basis_indefinite,
            depends_on_min_max_sizes,
            is_horizontal_flow,
            free_space_fraction: 0.0,
            state: FlexerState::kNone,
            flexed_content_size: hypothetical_content_size,
        }
    }

    // cpp: layoutng_flex/flex_item.h:77-80
    pub fn HypotheticalMainAxisMarginBoxSize(&self) -> LayoutUnit {
        self.hypothetical_content_size + self.main_axis_border_padding + self.MainAxisMarginExtent()
    }

    // cpp: layoutng_flex/flex_item.h:82-85
    pub fn FlexBaseMarginBoxSize(&self) -> LayoutUnit {
        self.base_content_size + self.main_axis_border_padding + self.MainAxisMarginExtent()
    }

    // cpp: layoutng_flex/flex_item.h:87-89
    pub fn FlexedBorderBoxSize(&self) -> LayoutUnit {
        self.flexed_content_size + self.main_axis_border_padding
    }

    // cpp: layoutng_flex/flex_item.h:91-94
    pub fn FlexedMarginBoxSize(&self) -> LayoutUnit {
        self.flexed_content_size + self.main_axis_border_padding + self.MainAxisMarginExtent()
    }

    // cpp: layoutng_flex/flex_item.h:96-99
    pub fn MainAxisMarginExtent(&self) -> LayoutUnit {
        if self.is_horizontal_flow {
            self.initial_margins.HorizontalSum()
        } else {
            self.initial_margins.VerticalSum()
        }
    }

    // cpp: layoutng_flex/flex_item.h:100-103
    pub fn CrossAxisMarginExtent(&self) -> LayoutUnit {
        if self.is_horizontal_flow {
            self.initial_margins.VerticalSum()
        } else {
            self.initial_margins.HorizontalSum()
        }
    }

    // cpp: layoutng_flex/flex_item.h:105-107
    pub fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.block_node.GetLayoutBox());
    }
}

impl Traceable for FlexItem {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        FlexItem::Trace(self, visitor);
    }
}
