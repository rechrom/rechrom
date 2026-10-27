use font_engine::Font;
use foundation::{AtomicString, LayoutUnit, Length, PhysicalRect, PhysicalSize, WritingMode};
use layout_fragment::block_break_token::BlockBreakToken;
use layout_fragment::box_fragment_builder::BoxFragmentBuilder;
use layout_fragment::break_token::BreakToken;
use layout_fragment::fragment_items_builder::FragmentItemsBuilder;
use layout_fragment::inline_break_token::InlineBreakToken;
use layout_fragment::layout_result::LayoutResult;
use layout_fragment::physical_box_fragment::PhysicalBoxFragment;
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
    /// Stable package identity used by assembly validation and diagnostics.
    /// Empty entries are intentionally unsupported by the current host.
    pub performance_owner: Option<&'static str>,
    pub layout: Option<fn(&LayoutAlgorithmParams) -> *const LayoutResult>,
    pub measure: Option<fn(&LayoutAlgorithmParams, &MinMaxSizesFloatInput) -> MinMaxSizesResult>,
    /// Optional package-owned wrapper for work surrounding the root dispatch.
    pub root_trace: Option<fn() -> browser_tracing::Span>,
}

impl LayoutAlgorithmEntry {
    fn validate_performance_ownership(&self, slot: &str) {
        let callbacks_are_complete = self.layout.is_some() && self.measure.is_some();
        let callbacks_are_empty = self.layout.is_none() && self.measure.is_none();
        assert!(
            (self.performance_owner.is_some() && callbacks_are_complete)
                || (self.performance_owner.is_none()
                    && callbacks_are_empty
                    && self.root_trace.is_none()),
            "LayoutNG algorithm slot {slot} must install performance_owner, layout, and measure atomically"
        );
        if let Some(owner) = self.performance_owner {
            assert!(
                owner.starts_with("layout_algo_"),
                "LayoutNG algorithm slot {slot} has non-package performance owner {owner:?}"
            );
        }
    }
}

fn validate_support_performance_owner(
    slot: &str,
    performance_owner: Option<&'static str>,
    has_callbacks: bool,
) {
    assert_eq!(
        performance_owner.is_some(),
        has_callbacks,
        "LayoutNG support slot {slot} must install its performance_owner and callbacks together"
    );
    if let Some(owner) = performance_owner {
        assert!(
            owner.starts_with("layout_algo_"),
            "LayoutNG support slot {slot} has non-package performance owner {owner:?}"
        );
    }
}

// cpp: layoutng/internal/layout_algorithm_set.h:62-85
#[derive(Clone, Copy, Default)]
pub struct InlineLayoutSupport {
    pub performance_owner: Option<&'static str>,
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
    pub performance_owner: Option<&'static str>,
    pub inline_size: Option<fn(&TableNode, &ConstraintSpace, &BoxStrut) -> LayoutUnit>,
    pub caption_block_size: Option<fn(&TableNode, &ConstraintSpace) -> LayoutUnit>,
    // C++ returns a borrowed reference with host-owned lifetime.
    pub borders: Option<fn(&TableNode) -> *const BoxStrut>,
    pub finalize_cell: Option<fn(LayoutUnit, *mut BoxFragmentBuilder)>,
}

// cpp: layoutng/internal/layout_algorithm_set.h:101-117
#[derive(Clone, Copy, Default)]
pub struct FragmentainerLayoutSupport {
    pub performance_owner: Option<&'static str>,
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
    pub performance_owner: Option<&'static str>,
    pub run: Option<fn(&mut BoxFragmentBuilder)>,
}

// cpp: layoutng/internal/layout_algorithm_set.h:125-137
#[derive(Clone, Copy, Default)]
pub struct FloatLayoutSupport {
    pub performance_owner: Option<&'static str>,
    pub margin_box_inline_size: Option<fn(*mut UnpositionedFloat) -> LayoutUnit>,
    pub position: Option<fn(*mut UnpositionedFloat, *mut ExclusionSpace) -> PositionedFloat>,
    pub create_shape: Option<fn(&LayoutBox, LogicalSize, LayoutUnit) -> Option<Box<dyn Shape>>>,
    pub update_shape_outside: Option<fn(&BlockNode, &LayoutResult, &ConstraintSpace)>,
}

// cpp: layoutng/internal/layout_algorithm_set.h:139-145
#[derive(Clone, Copy, Default)]
pub struct ReplacedSizingSupport {
    pub performance_owner: Option<&'static str>,
    pub natural_sizing_info: Option<fn(&LayoutInputNode) -> PhysicalNaturalSizingInfo>,
}

// cpp: layoutng/internal/layout_algorithm_set.h:147-153
#[derive(Clone, Copy, Default)]
pub struct SimplifiedLayoutSupport {
    pub performance_owner: Option<&'static str>,
    pub layout: Option<fn(&LayoutAlgorithmParams, &LayoutResult) -> *const LayoutResult>,
}

// cpp: layoutng/internal/layout_algorithm_set.h:155-166
#[derive(Clone, Copy, Default)]
pub struct ListLayoutSupport {
    pub performance_owner: Option<&'static str>,
    pub update_marker_text: Option<fn(&mut LayoutObject)>,
    pub marker_occupies_whole_line: Option<fn(&LayoutObject) -> bool>,
    pub symbol_marker_text: Option<fn(*const LayoutObject) -> *const LayoutObject>,
    pub symbol_width: Option<fn(&ComputedStyle, &AtomicString) -> LayoutUnit>,
    pub outside_marker_inline_offset: Option<fn(&LayoutObject, LayoutUnit) -> LayoutUnit>,
}

// cpp: layoutng/internal/layout_algorithm_set.h:168-175
#[derive(Clone, Copy, Default)]
pub struct FormLayoutSupport {
    pub performance_owner: Option<&'static str>,
    pub intrinsic_inline_size: Option<fn(&LayoutBox) -> LayoutUnit>,
    pub intrinsic_block_size: Option<fn(&LayoutBox, bool) -> LayoutUnit>,
}

// cpp: layoutng/internal/layout_algorithm_set.h:177-187
#[derive(Clone, Copy, Default)]
pub struct MathMLSupport {
    pub performance_owner: Option<&'static str>,
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
    pub performance_owner: Option<&'static str>,
    // Pure derived SVG Layout visual geometry, in local SVG user coordinates.
    pub text_visual_rect: Option<fn(&LayoutObject) -> foundation::gfx::RectF>,
    pub viewport_size: Option<fn(&LayoutObject) -> foundation::gfx::SizeF>,
    pub local_to_parent_transform: Option<
        fn(
            &LayoutObject,
        ) -> foundation::blink_geometry::transforms::affine_transform::AffineTransform,
    >,
    pub paint_geometry: Option<
        fn(
            &LayoutObject,
        ) -> (
            Option<super::layout_input::TransformMatrix>,
            Option<layout_fragment::fragment_tree::SvgViewportGeometry>,
        ),
    >,
    // Rust callback for the C++ LayoutObject::UpdateSVGLayout virtual override.
    pub update_object: Option<fn(&mut LayoutObject, &SVGLayoutInfo) -> SVGLayoutResult>,
    pub root_natural_dimensions: Option<fn(&LayoutObject) -> PhysicalNaturalSizingInfo>,
    pub root_sizing_info: Option<fn(&LayoutObject) -> SvgRootSizingInfo>,
    pub set_root_container_size: Option<fn(&mut LayoutObject, PhysicalSize)>,
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
    pub performance_owner: Option<&'static str>,
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

impl LayoutAlgorithmSet {
    /// Reject partially installed algorithm slots before a layout pass can
    /// produce samples with no package owner. Empty slots are supported: they
    /// represent capabilities that this host intentionally did not assemble.
    pub fn validate_performance_ownership(&self) {
        for (slot, entry) in [
            ("block", &self.block),
            ("flex", &self.flex),
            ("table", &self.table),
            ("table_row", &self.table_row),
            ("table_section", &self.table_section),
            ("custom", &self.custom),
            ("mathml", &self.mathml),
            ("grid", &self.grid),
            ("grid_lanes", &self.grid_lanes),
            ("replaced", &self.replaced),
            ("fieldset", &self.fieldset),
            ("frameset", &self.frameset),
            ("multicol", &self.multicol),
            ("paged", &self.paged),
        ] {
            entry.validate_performance_ownership(slot);
        }

        validate_support_performance_owner(
            "inline_support",
            self.inline_support.performance_owner,
            self.inline_support.is_block_level.is_some()
                || self.inline_support.layout_block_child.is_some()
                || self.inline_support.reuse_fragments.is_some()
                || self.inline_support.layout.is_some()
                || self.inline_support.measure.is_some(),
        );
        validate_support_performance_owner(
            "table_support",
            self.table_support.performance_owner,
            self.table_support.inline_size.is_some()
                || self.table_support.caption_block_size.is_some()
                || self.table_support.borders.is_some()
                || self.table_support.finalize_cell.is_some(),
        );
        validate_support_performance_owner(
            "fragmentainer_support",
            self.fragmentainer_support.performance_owner,
            self.fragmentainer_support.empty_page.is_some()
                || self.fragmentainer_support.empty_column.is_some()
                || self.fragmentainer_support.layout_repeatable_root.is_some()
                || self.fragmentainer_support.finish_repeatable_root.is_some(),
        );
        validate_support_performance_owner(
            "out_of_flow_support",
            self.out_of_flow_support.performance_owner,
            self.out_of_flow_support.run.is_some(),
        );
        validate_support_performance_owner(
            "float_support",
            self.float_support.performance_owner,
            self.float_support.margin_box_inline_size.is_some()
                || self.float_support.position.is_some()
                || self.float_support.create_shape.is_some()
                || self.float_support.update_shape_outside.is_some(),
        );
        validate_support_performance_owner(
            "replaced_sizing",
            self.replaced_sizing.performance_owner,
            self.replaced_sizing.natural_sizing_info.is_some(),
        );
        validate_support_performance_owner(
            "simplified_support",
            self.simplified_support.performance_owner,
            self.simplified_support.layout.is_some(),
        );
        validate_support_performance_owner(
            "list_support",
            self.list_support.performance_owner,
            self.list_support.update_marker_text.is_some()
                || self.list_support.marker_occupies_whole_line.is_some()
                || self.list_support.symbol_marker_text.is_some()
                || self.list_support.symbol_width.is_some()
                || self.list_support.outside_marker_inline_offset.is_some(),
        );
        validate_support_performance_owner(
            "forms_support",
            self.forms_support.performance_owner,
            self.forms_support.intrinsic_inline_size.is_some()
                || self.forms_support.intrinsic_block_size.is_some(),
        );
        validate_support_performance_owner(
            "mathml_support",
            self.mathml_support.performance_owner,
            self.mathml_support.axis_height.is_some()
                || self.mathml_support.fraction_rule_thickness.is_some()
                || self.mathml_support.radical_metrics.is_some()
                || self.mathml_support.table_baseline.is_some(),
        );
        validate_support_performance_owner(
            "svg_support",
            self.svg_support.performance_owner,
            self.svg_support.text_visual_rect.is_some()
                || self.svg_support.viewport_size.is_some()
                || self.svg_support.local_to_parent_transform.is_some()
                || self.svg_support.paint_geometry.is_some()
                || self.svg_support.update_object.is_some()
                || self.svg_support.root_natural_dimensions.is_some()
                || self.svg_support.root_sizing_info.is_some()
                || self.svg_support.set_root_container_size.is_some()
                || self.svg_support.layout_root.is_some()
                || self.svg_support.resolve_length.is_some()
                || self.svg_support.screen_font_scale.is_some()
                || self.svg_support.compute_scaled_font.is_some()
                || self.svg_support.build_text_attributes.is_some()
                || self.svg_support.layout_text.is_some(),
        );
        validate_support_performance_owner(
            "grid_lanes_support",
            self.grid_lanes_support.performance_owner,
            self.grid_lanes_support.build_sizing_subtree.is_some()
                || self.grid_lanes_support.construct_subgridded_items.is_some(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_algorithm_slots_are_explicitly_unsupported() {
        LayoutAlgorithmSet::default().validate_performance_ownership();
    }

    #[test]
    #[should_panic(expected = "must install performance_owner, layout, and measure atomically")]
    fn performance_owner_cannot_be_installed_without_callbacks() {
        let mut algorithms = LayoutAlgorithmSet::default();
        algorithms.custom.performance_owner = Some("layout_algo_custom");
        algorithms.validate_performance_ownership();
    }

    #[test]
    #[should_panic(expected = "must install its performance_owner and callbacks together")]
    fn support_owner_cannot_be_installed_without_callbacks() {
        let mut algorithms = LayoutAlgorithmSet::default();
        algorithms.out_of_flow_support.performance_owner = Some("layout_algo_out_of_flow");
        algorithms.validate_performance_ownership();
    }
}

// cpp: layoutng/internal/layout_algorithm_set.h:258-259
// FullLayoutAlgorithms is defined in //src/main, outside the selected packages.
