use font_engine::Font;
use foundation::{AtomicString, LayoutUnit, Length, PhysicalRect, PhysicalSize, WritingMode};
use layoutng_fragment_tree::block_break_token::BlockBreakToken;
use layoutng_fragment_tree::box_fragment_builder::BoxFragmentBuilder;
use layoutng_fragment_tree::break_token::BreakToken;
use layoutng_fragment_tree::fragment_items_builder::FragmentItemsBuilder;
use layoutng_fragment_tree::inline_break_token::InlineBreakToken;
use layoutng_fragment_tree::layout_result::LayoutResult;
use layoutng_fragment_tree::physical_box_fragment::PhysicalBoxFragment;
use layoutng_geometry::geometry::box_strut::BoxStrut;
use layoutng_geometry::geometry::logical_size::LogicalSize;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::natural_sizing_info::PhysicalNaturalSizingInfo;

use super::algorithm_forward::{
    BlockLayoutAlgorithm, GridLanesSizingSubtreeRequest, GridLanesSubgriddedItemsRequest,
    InlineChildLayoutContext, PreviousInflowPosition,
};
use super::block_node::BlockNode;
use super::column_spanner_path::ColumnSpannerPath;
use super::constraint_space::ConstraintSpace;
use super::exclusions::exclusion_space::ExclusionSpace;
use super::fragment_item_with_offset_fwd::FragmentItemWithOffsetList;
use super::grid_item::GridItems;
use super::inline_node::InlineNode;
use super::layout_algorithm::LayoutAlgorithmParams;
use super::layout_box::LayoutBox;
use super::layout_input_node::{LayoutInputNode, MinMaxSizesFloatInput};
use super::layout_object::LayoutObject;
use super::min_max_sizes::MinMaxSizesResult;
use super::positioned_float::PositionedFloat;
use super::shapes::shape::Shape;
use super::svg_layout_info::{SVGLayoutInfo, SVGLayoutResult};
use super::svg_text_attributes_request::{
    SvgTextAttributesBuildRequest, SvgTextAttributesBuildResult,
};
use super::table_node::TableNode;
use super::unpositioned_float::UnpositionedFloat;

// cpp: layoutng/internal/layout_algorithm_set.h:53-60
#[derive(Clone, Copy, Default)]
pub struct LayoutAlgorithmEntry {
    pub layout: Option<fn(&LayoutAlgorithmParams) -> *const LayoutResult>,
    pub measure: Option<fn(&LayoutAlgorithmParams, &MinMaxSizesFloatInput) -> MinMaxSizesResult>,
}

// cpp: layoutng/internal/layout_algorithm_set.h:62-85
#[derive(Clone, Copy, Default)]
pub struct InlineLayoutSupport {
    pub is_block_level: Option<fn(&InlineNode) -> bool>,
    // cpp: layoutng/internal/layout_algorithm_set.h:66-74
    // C++ forward-declares the block-owned concrete types. Rust cannot name
    // them here without a Cargo cycle, so the callback ABI carries their
    // addresses and the block package restores the types at its boundary.
    pub layout_block_child:
        Option<fn(*mut BlockLayoutAlgorithm, &InlineNode) -> *const LayoutResult>,
    pub reuse_fragments: Option<
        fn(
            *mut BlockLayoutAlgorithm,
            InlineNode,
            *mut PreviousInflowPosition,
            *mut *const InlineBreakToken,
        ) -> bool,
    >,
    pub layout: Option<
        fn(
            &InlineNode,
            &ConstraintSpace,
            *const BreakToken,
            *const ColumnSpannerPath,
            *mut InlineChildLayoutContext,
        ) -> *const LayoutResult,
    >,
    pub measure: Option<
        fn(&InlineNode, WritingMode, &ConstraintSpace, &MinMaxSizesFloatInput) -> MinMaxSizesResult,
    >,
}

// cpp: layoutng/internal/layout_algorithm_set.h:87-99
#[derive(Clone, Copy, Default)]
pub struct TableLayoutSupport {
    pub inline_size: Option<fn(&TableNode, &ConstraintSpace, &BoxStrut) -> LayoutUnit>,
    pub caption_block_size: Option<fn(&TableNode, &ConstraintSpace) -> LayoutUnit>,
    // C++ returns a borrowed reference with host-owned lifetime.
    pub borders: Option<fn(&TableNode) -> *const BoxStrut>,
    pub finalize_cell: Option<fn(LayoutUnit, *mut BoxFragmentBuilder)>,
}

// cpp: layoutng/internal/layout_algorithm_set.h:101-117
#[derive(Clone, Copy, Default)]
pub struct FragmentainerLayoutSupport {
    pub empty_page: Option<
        fn(
            &BlockNode,
            &ConstraintSpace,
            u32,
            &PhysicalBoxFragment,
            *mut bool,
        ) -> *const PhysicalBoxFragment,
    >,
    pub empty_column: Option<
        fn(&BlockNode, &ConstraintSpace, &PhysicalBoxFragment) -> *const PhysicalBoxFragment,
    >,
    pub layout_repeatable_root:
        Option<fn(&BlockNode, &ConstraintSpace, *const BlockBreakToken) -> *const LayoutResult>,
    pub finish_repeatable_root: Option<fn(&BlockNode)>,
}

// cpp: layoutng/internal/layout_algorithm_set.h:119-123
#[derive(Clone, Copy, Default)]
pub struct OutOfFlowLayoutSupport {
    pub run: Option<fn(&mut BoxFragmentBuilder)>,
}

// cpp: layoutng/internal/layout_algorithm_set.h:125-137
#[derive(Clone, Copy, Default)]
pub struct FloatLayoutSupport {
    pub margin_box_inline_size: Option<fn(*mut UnpositionedFloat) -> LayoutUnit>,
    pub position: Option<fn(*mut UnpositionedFloat, *mut ExclusionSpace) -> PositionedFloat>,
    pub create_shape: Option<fn(&LayoutBox, LogicalSize, LayoutUnit) -> Option<Box<dyn Shape>>>,
    pub update_shape_outside: Option<fn(&BlockNode, &LayoutResult, &ConstraintSpace)>,
}

// cpp: layoutng/internal/layout_algorithm_set.h:139-145
#[derive(Clone, Copy, Default)]
pub struct ReplacedSizingSupport {
    pub natural_sizing_info: Option<fn(&LayoutInputNode) -> PhysicalNaturalSizingInfo>,
}

// cpp: layoutng/internal/layout_algorithm_set.h:147-153
#[derive(Clone, Copy, Default)]
pub struct SimplifiedLayoutSupport {
    pub layout: Option<fn(&LayoutAlgorithmParams, &LayoutResult) -> *const LayoutResult>,
}

// cpp: layoutng/internal/layout_algorithm_set.h:155-166
#[derive(Clone, Copy, Default)]
pub struct ListLayoutSupport {
    pub update_marker_text: Option<fn(&mut LayoutObject)>,
    pub marker_occupies_whole_line: Option<fn(&LayoutObject) -> bool>,
    pub symbol_marker_text: Option<fn(*const LayoutObject) -> *const LayoutObject>,
    pub symbol_width: Option<fn(&ComputedStyle, &AtomicString) -> LayoutUnit>,
    pub outside_marker_inline_offset: Option<fn(&LayoutObject, LayoutUnit) -> LayoutUnit>,
}

// cpp: layoutng/internal/layout_algorithm_set.h:168-175
#[derive(Clone, Copy, Default)]
pub struct FormLayoutSupport {
    pub intrinsic_inline_size: Option<fn(&LayoutBox) -> LayoutUnit>,
    pub intrinsic_block_size: Option<fn(&LayoutBox, bool) -> LayoutUnit>,
}

// cpp: layoutng/internal/layout_algorithm_set.h:177-187
#[derive(Clone, Copy, Default)]
pub struct MathMLSupport {
    pub axis_height: Option<fn(&ComputedStyle) -> LayoutUnit>,
    pub fraction_rule_thickness: Option<fn(&ComputedStyle) -> LayoutUnit>,
    pub radical_metrics: Option<fn(&ComputedStyle, bool, *mut LayoutUnit, *mut LayoutUnit)>,
    pub table_baseline: Option<fn(&ComputedStyle, LayoutUnit) -> LayoutUnit>,
}

// cpp: layoutng/internal/layout_algorithm_set.h:189-194
#[derive(Clone, Copy)]
pub struct SvgRootSizingInfo {
    pub container_width: LayoutUnit,
    pub container_height: LayoutUnit,
    pub embedded_through_frame: bool,
    pub logical_size_scale_factor: f64,
}

impl Default for SvgRootSizingInfo {
    fn default() -> Self {
        Self {
            container_width: LayoutUnit::default(),
            container_height: LayoutUnit::default(),
            embedded_through_frame: false,
            logical_size_scale_factor: 1.0,
        }
    }
}

// cpp: layoutng/internal/layout_algorithm_set.h:196-215
#[derive(Clone, Copy, Default)]
pub struct SvgLayoutSupport {
    // Rust callback for the C++ LayoutObject::UpdateSVGLayout virtual override.
    pub update_object: Option<fn(&mut LayoutObject, &SVGLayoutInfo) -> SVGLayoutResult>,
    pub root_natural_dimensions: Option<fn(&LayoutObject) -> PhysicalNaturalSizingInfo>,
    pub root_sizing_info: Option<fn(&LayoutObject) -> SvgRootSizingInfo>,
    pub layout_root: Option<fn(&mut LayoutObject, &PhysicalRect)>,
    pub resolve_length: Option<fn(&Length, &ComputedStyle, f32) -> f32>,
    pub screen_font_scale: Option<fn(&LayoutObject) -> f32>,
    pub compute_scaled_font: Option<fn(&LayoutObject, &mut f32) -> *const Font>,
    pub build_text_attributes:
        Option<for<'a> fn(&SvgTextAttributesBuildRequest<'a>, &mut SvgTextAttributesBuildResult)>,
    // The C++ callback receives a const builder and its mutable items_ field
    // simultaneously. Raw pointers retain that aliasing boundary for the
    // optional SVG package without manufacturing overlapping Rust references.
    pub layout_text: Option<
        fn(
            *const InlineNode,
            *const FragmentItemsBuilder,
            *mut FragmentItemWithOffsetList,
        ) -> PhysicalSize,
    >,
}

// cpp: layoutng/internal/layout_algorithm_set.h:217-225
#[derive(Clone, Copy, Default)]
pub struct GridLanesLayoutSupport {
    // cpp: layoutng/internal/layout_algorithm_set.h:217-225
    // The request types belong to the optional Grid Lanes package. Their
    // addresses cross this package boundary unchanged; that package's
    // installer owns the concrete casts when it is translated.
    pub build_sizing_subtree: Option<fn(*const GridLanesSizingSubtreeRequest)>,
    pub construct_subgridded_items:
        Option<fn(*const GridLanesSubgriddedItemsRequest) -> *mut GridItems>,
}

// cpp: layoutng/internal/layout_algorithm_set.h:227-256
#[derive(Clone, Copy, Default)]
pub struct LayoutAlgorithmSet {
    pub block: LayoutAlgorithmEntry,
    pub inline_support: InlineLayoutSupport,
    pub flex: LayoutAlgorithmEntry,
    pub table: LayoutAlgorithmEntry,
    pub table_row: LayoutAlgorithmEntry,
    pub table_section: LayoutAlgorithmEntry,
    pub custom: LayoutAlgorithmEntry,
    pub mathml: LayoutAlgorithmEntry,
    pub grid: LayoutAlgorithmEntry,
    pub grid_lanes: LayoutAlgorithmEntry,
    pub replaced: LayoutAlgorithmEntry,
    pub fieldset: LayoutAlgorithmEntry,
    pub frameset: LayoutAlgorithmEntry,
    pub multicol: LayoutAlgorithmEntry,
    pub paged: LayoutAlgorithmEntry,
    pub table_support: TableLayoutSupport,
    pub fragmentainer_support: FragmentainerLayoutSupport,
    pub out_of_flow_support: OutOfFlowLayoutSupport,
    pub float_support: FloatLayoutSupport,
    pub replaced_sizing: ReplacedSizingSupport,
    pub simplified_support: SimplifiedLayoutSupport,
    pub list_support: ListLayoutSupport,
    pub forms_support: FormLayoutSupport,
    pub mathml_support: MathMLSupport,
    pub svg_support: SvgLayoutSupport,
    pub grid_lanes_support: GridLanesLayoutSupport,
}

// cpp: layoutng/internal/layout_algorithm_set.h:258-259
// FullLayoutAlgorithms is defined in //src/main, outside the selected packages.
