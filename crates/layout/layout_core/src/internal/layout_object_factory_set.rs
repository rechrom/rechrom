use layout_style::style::computed_style::{ComputedStyle, ComputedStyleBuilder};
use layout_style::style::style_difference::StyleDifference;

use super::layout_block::LayoutBlock;
use super::layout_block_flow::LayoutBlockFlow;
use super::layout_box::LayoutBox;
use super::layout_input::NodeConstructionData;
use super::layout_node_metadata::{Element, Node};
use super::layout_object::LayoutObject;
use super::layout_replaced::LayoutReplaced;

// cpp: layoutng/internal/layout_object_factory_set.h:19-43
pub type LayoutObjectFactory = fn(&mut Node, &ComputedStyle) -> *mut LayoutObject;
pub type ElementMetadataFactory = fn(&NodeConstructionData, *const ComputedStyle) -> *mut Element;
pub type AnonymousLayoutBlockFactory = fn() -> *mut LayoutBlock;
pub type TableChildInserter = fn(&mut LayoutObject, &mut LayoutObject, *mut LayoutObject);
pub type InlineTextChildInserter =
    fn(&mut LayoutObject, &mut LayoutObject, *mut LayoutObject) -> bool;
pub type InnerEditorChildRemoval = fn(&mut LayoutBlockFlow, &mut LayoutObject);
// Rust's base-first object representation dispatches these out-of-package
// virtual overrides through the installed forms module.
pub type InnerEditorAllowsInlineChildren = fn(&LayoutBlockFlow) -> bool;
pub type InnerEditorChildInserter = fn(&mut LayoutBlockFlow, *mut LayoutObject, *mut LayoutObject);
pub type InnerEditorStyleUpdater = fn(
    &mut LayoutBlockFlow,
    StyleDifference,
    *const ComputedStyle,
    &ComputedStyle,
    &super::layout_object::StyleChangeContext,
);
pub type FieldsetChildInserter = fn(&mut LayoutBlockFlow, *mut LayoutObject, *mut LayoutObject);
pub type FieldsetTreeInserter = fn(&mut LayoutBlockFlow);
pub type FieldsetAnonymousStyleUpdater =
    fn(&LayoutBlockFlow, *const LayoutObject, &mut ComputedStyleBuilder);
pub type FieldsetContentBox = fn(&mut LayoutBlockFlow) -> *mut LayoutBox;
pub type FieldsetScrollExtent = fn(&LayoutBlockFlow) -> foundation::LayoutUnit;
pub type CustomLayoutChildFactory = fn(&mut LayoutBox);
pub type InlineObjectStyleUpdater = fn(&mut LayoutBlockFlow, *const ComputedStyle, &ComputedStyle);
pub type AnonymousTextCombineStyleUpdater = fn(&mut LayoutObject, &mut ComputedStyleBuilder);
pub type CombinedTextStyleAdjuster = fn(&mut ComputedStyleBuilder);
#[derive(Clone, Copy)]
pub struct TextFragmentSelectionFacts {
    pub node_length: Option<u32>,
    pub state: super::selection_state::SelectionState,
    pub start: u32,
    pub length: u32,
}
pub type TextFragmentSelectionFactsQuery =
    fn(&super::layout_text::LayoutText) -> TextFragmentSelectionFacts;
pub type TextFragmentSelectionQuery = fn(&super::layout_text::LayoutText) -> bool;
pub type TextCombineBoundingBoxAdjuster =
    fn(&LayoutObject, &foundation::PhysicalRect) -> foundation::PhysicalRect;
// InlineCursor belongs to the reciprocal fragment package, so the factory
// boundary carries its borrowed pointer opaquely and the inline owner restores
// the concrete type without moving or retaining it.
pub type TextCombineContentsInkOverflowRecalculator =
    fn(&LayoutObject, *const ()) -> foundation::PhysicalRect;
pub type SvgBlockTextScale = fn(&LayoutBlock) -> f64;
pub type SvgBlockStyleUpdater = fn(&mut LayoutBlock, f64, *const ComputedStyle, &ComputedStyle);
// Rust callback for LayoutSVGGroup::AddChild's C++ virtual override.
pub type SvgGroupChildInserter = fn(&mut LayoutBlockFlow, *mut LayoutObject, *mut LayoutObject);
pub type SvgChildAllowed = fn(&LayoutObject, *mut LayoutObject, &ComputedStyle) -> bool;
pub type SvgTextChildChange = fn(&mut LayoutBlockFlow, *mut LayoutObject, *mut LayoutObject);
pub type SvgTextChildRemoval = fn(&mut LayoutBlockFlow, *mut LayoutObject);
pub type SvgTextTreeChange = fn(&mut LayoutBlockFlow);
pub type SvgTextMetricsInvalidator = fn(&mut LayoutObject);
pub type SvgInlineTextFontUpdater = fn(&mut LayoutObject);
pub type SvgInlineTextScaledFont = fn(&LayoutObject) -> *const font_engine::Font;
pub type SvgInlineTextScalingFactor = fn(&LayoutObject) -> f32;
pub type SvgRootChildInserter = fn(&mut LayoutObject, *mut LayoutObject, *mut LayoutObject);
pub type SvgRootChildRemover = fn(&mut LayoutObject, *mut LayoutObject);
pub type SvgRootChildren =
    fn(&LayoutObject) -> *mut super::layout_object_child_list::LayoutObjectChildList;
pub type ColumnSpannerInvalidator = fn(&mut LayoutBox);
pub type ReplacedRespectsCSSOverflow = fn(&LayoutReplaced) -> bool;
pub type ReplacedImageStateUpdater = fn(&mut LayoutReplaced, f64, bool);
pub type ReplacedImageResourceUpdater =
    fn(&mut LayoutReplaced, image_resource::ImageResourceContentUpdate) -> bool;
pub type ReplacedVideoPosterResourceUpdater =
    fn(&mut LayoutReplaced, image_resource::ImageResourceContentUpdate) -> bool;
pub type ReplacedBrokenImageResourceUpdater = fn(
    &mut LayoutReplaced,
    image_resource::platform_resources::BrokenImageResourceVariants,
    f64,
) -> bool;
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ReplacedImageResourceFacts {
    pub has_image: bool,
    pub metadata: Option<image_resource::ImageResourceMetadata>,
    pub error: bool,
    pub loading: bool,
    pub device_pixel_ratio: f32,
}
pub type ReplacedImageResourceFactsQuery = fn(&LayoutReplaced) -> ReplacedImageResourceFacts;
pub type ReplacedVideoStateUpdater = fn(&mut LayoutReplaced);

/// Source quad virtual on an out-of-package SVG object. The override is
/// supplied by the object's existing factory owner, never synthesized as a CSS box.
pub type ObjectQuadEnumerator = fn(
    &LayoutObject,
    &mut foundation::Vector<foundation::gfx::QuadF>,
    *const super::layout_box_model_object::LayoutBoxModelObject,
    super::map_coordinates_flags::MapCoordinatesFlags,
    super::layout_object::BoxQuadType,
);

// C++ derived table vtables live in the table package. This typed callback
// record preserves those overrides without a Cargo dependency cycle. Every
// entry is required when the table object owner is installed.
#[derive(Clone, Copy)]
pub struct TableObjectVirtuals {
    pub focusgroup_facts: fn(
        &LayoutObject,
    ) -> Option<
        foundation::table_focusgroup::TableFocusgroupFacts<*const LayoutObject>,
    >,
    pub quads_in_ancestor: ObjectQuadEnumerator,
    pub add_child: fn(&mut LayoutObject, *mut LayoutObject, *mut LayoutObject),
    pub remove_child: fn(&mut LayoutObject, *mut LayoutObject),
    pub style_did_change: fn(
        &mut LayoutObject,
        StyleDifference,
        *const ComputedStyle,
        &ComputedStyle,
        &super::layout_object::StyleChangeContext,
    ),
    pub inserted_into_tree: fn(&mut LayoutObject),
    pub will_be_removed_from_tree: fn(&mut LayoutObject),
    pub virtual_children:
        fn(&LayoutObject) -> *mut super::layout_object_child_list::LayoutObjectChildList,
    pub create_anonymous: fn(&LayoutBox, *const LayoutObject) -> *mut LayoutBox,
    pub border_outsets: fn(&LayoutBox) -> layout_geometry::geometry::box_strut::PhysicalBoxStrut,
    pub padding_outsets: fn(&LayoutBox) -> layout_geometry::geometry::box_strut::PhysicalBoxStrut,
    pub sticky_container: fn(&LayoutBox) -> *mut LayoutBlock,
    pub invalidate_after_measure: fn(&LayoutBox),
    pub intrinsic_borders:
        fn(&LayoutBox) -> *const layout_geometry::geometry::box_strut::BoxStrut,
    pub set_intrinsic_borders:
        fn(&mut LayoutBox, &layout_geometry::geometry::box_strut::BoxStrut),
    pub stitched_size: fn(&LayoutBox) -> foundation::PhysicalSize,
    pub physical_location: fn(&LayoutBox) -> foundation::PhysicalOffset,
    pub overflow_clip_rect:
        fn(&LayoutBox, foundation::OverlayScrollbarClipBehavior) -> foundation::PhysicalRect,
    pub computed_row_span: fn(&LayoutBox) -> u32,
    pub update_from_element: fn(&mut LayoutObject),
    // cpp: core/editing/element_inner_text.cc:180-211
    // Whether a LayoutTableRow is followed by another row of its table (next
    // row, or a later section with rows). Base editing (InnerTextLayoutFacts)
    // calls it on a table row after layout; the object's own FactorySet()
    // carries the record, so no layout pass needs to be active.
    pub inner_text_row_needs_newline: fn(&LayoutObject) -> bool,
    // cpp: core/editing/iterators/text_iterator.cc:628-640,689-695
    // For a LayoutTableCell / LayoutTableRow: (has table, is the table's
    // first cell, table is inline). Called by base editing's
    // EditingTextIteratorObjectFacts like `inner_text_row_needs_newline`.
    pub text_iterator_table_facts: fn(&LayoutObject) -> (bool, bool, bool),
}

// C++ LayoutGrid overrides these LayoutObject/LayoutBox virtuals; the class
// lives in the grid package. Filled by `layout_algo_grid::registration::Register`
// and read through the object's own factory set (`LayoutObject::FactorySet()`).
// cpp: layoutng_grid/layout_grid.h:87-96,108-109
#[derive(Clone, Copy)]
pub struct GridObjectVirtuals {
    pub add_child: fn(*mut LayoutObject, *mut LayoutObject, *mut LayoutObject),
    pub remove_child: fn(*mut LayoutObject, *mut LayoutObject),
    pub style_did_change: fn(
        *mut LayoutObject,
        StyleDifference,
        *const ComputedStyle,
        &ComputedStyle,
        &super::layout_object::StyleChangeContext,
    ),
    pub stitched_row_gap_index: StitchedRowGapIndex,
    pub adjust_anchor_containing_block: fn(
        &LayoutBox,
        *const super::grid_layout_data::GridLayoutData,
        &ComputedStyle,
        &layout_geometry::geometry::logical_rect::LogicalRect,
        &LayoutBox,
    ) -> layout_geometry::geometry::logical_rect::LogicalRect,
}

// LayoutObject::StitchedRowGapIndex, overridden by LayoutFlexibleBox and
// LayoutGrid. cpp: layoutng/internal/layout_object.h:2023-2029
pub type StitchedRowGapIndex = fn(
    *const LayoutObject,
    &crate::physical_box_fragment::PhysicalBoxFragment,
    usize,
    Option<usize>,
) -> usize;

// cpp: layoutng/internal/layout_object_factory_set.h:45-85
#[derive(Clone, Copy, Default)]
pub struct LayoutObjectFactorySet {
    pub svg_context_reference_box: Option<fn(&LayoutObject) -> foundation::gfx::RectF>,
    pub svg_marker_paints: Option<fn(&LayoutObject) -> Vec<super::paint_input::SvgMarkerPaint>>,
    /// Injected platform Path construction; SVG Layout owns the returned path.
    pub svg_shape_path: Option<
        fn(
            &super::layout_input::SvgShapeData,
        ) -> std::sync::Arc<dyn foundation::native_path::NativePath>,
    >,

    pub table_virtuals: Option<TableObjectVirtuals>,
    pub svg_quads_in_ancestor: Option<ObjectQuadEnumerator>,
    pub block_ruby: Option<LayoutObjectFactory>,
    pub inline_box: Option<LayoutObjectFactory>,
    pub inline_line_break: Option<LayoutObjectFactory>,
    pub inline_text: Option<LayoutObjectFactory>,
    pub inline_word_break: Option<LayoutObjectFactory>,
    pub replaced: Option<LayoutObjectFactory>,
    pub replaced_respects_css_overflow: Option<ReplacedRespectsCSSOverflow>,
    pub replaced_image_state_updater: Option<ReplacedImageStateUpdater>,
    pub replaced_image_resource_updater: Option<ReplacedImageResourceUpdater>,
    pub replaced_video_poster_resource_updater: Option<ReplacedVideoPosterResourceUpdater>,
    pub replaced_broken_image_resource_updater: Option<ReplacedBrokenImageResourceUpdater>,
    pub replaced_image_resource_facts: Option<ReplacedImageResourceFactsQuery>,
    pub replaced_video_state_updater: Option<ReplacedVideoStateUpdater>,
    pub flex: Option<LayoutObjectFactory>,
    /// LayoutFlexibleBox::StitchedRowGapIndex (layoutng_flex/layout_flexible_box.h:33-37).
    pub flex_stitched_row_gap_index: Option<StitchedRowGapIndex>,
    pub grid: Option<LayoutObjectFactory>,
    pub grid_virtuals: Option<GridObjectVirtuals>,
    pub grid_lanes: Option<LayoutObjectFactory>,
    pub table: Option<LayoutObjectFactory>,
    pub custom: Option<LayoutObjectFactory>,
    pub forms: Option<LayoutObjectFactory>,
    pub frameset: Option<LayoutObjectFactory>,
    pub list: Option<LayoutObjectFactory>,
    /// Original derived ListItem/InlineListItem::SubtreeDidChange virtual.
    pub list_subtree_did_change: Option<fn(&mut LayoutObject)>,
    pub mathml: Option<LayoutObjectFactory>,
    pub svg: Option<LayoutObjectFactory>,

    pub anonymous_flex: Option<AnonymousLayoutBlockFactory>,
    pub anonymous_grid: Option<AnonymousLayoutBlockFactory>,
    pub anonymous_grid_lanes: Option<AnonymousLayoutBlockFactory>,
    pub anonymous_mathml: Option<AnonymousLayoutBlockFactory>,
    pub insert_table_child: Option<TableChildInserter>,
    pub insert_inline_text_child: Option<InlineTextChildInserter>,
    pub remove_inner_editor_child: Option<InnerEditorChildRemoval>,
    pub inner_editor_allows_inline_children: Option<InnerEditorAllowsInlineChildren>,
    pub inner_editor_add_child: Option<InnerEditorChildInserter>,
    pub inner_editor_style_did_change: Option<InnerEditorStyleUpdater>,
    pub fieldset_add_child: Option<FieldsetChildInserter>,
    pub fieldset_inserted_into_tree: Option<FieldsetTreeInserter>,
    pub fieldset_update_anonymous_child_style: Option<FieldsetAnonymousStyleUpdater>,
    pub fieldset_content_layout_box: Option<FieldsetContentBox>,
    pub fieldset_scroll_width: Option<FieldsetScrollExtent>,
    pub fieldset_scroll_height: Option<FieldsetScrollExtent>,
    pub create_custom_child: Option<CustomLayoutChildFactory>,
    pub update_inline_style: Option<InlineObjectStyleUpdater>,
    pub update_anonymous_text_combine_style: Option<AnonymousTextCombineStyleUpdater>,
    pub adjust_combined_text_style: Option<CombinedTextStyleAdjuster>,
    pub is_text_fragment_selected: Option<TextFragmentSelectionQuery>,
    pub text_fragment_selection_facts: Option<TextFragmentSelectionFactsQuery>,
    pub adjust_text_combine_bounding_box: Option<TextCombineBoundingBoxAdjuster>,
    pub recalc_text_combine_contents_ink_overflow:
        Option<TextCombineContentsInkOverflowRecalculator>,
    pub svg_block_text_scale: Option<SvgBlockTextScale>,
    pub update_svg_block_style: Option<SvgBlockStyleUpdater>,
    pub svg_group_add_child: Option<SvgGroupChildInserter>,
    pub svg_container_style_did_change: Option<InnerEditorStyleUpdater>,
    pub svg_shape_style_did_change: Option<InnerEditorStyleUpdater>,
    pub svg_is_child_allowed: Option<SvgChildAllowed>,
    pub svg_text_add_child: Option<SvgTextChildChange>,
    pub svg_text_remove_child: Option<SvgTextChildRemoval>,
    pub svg_text_inserted_into_tree: Option<SvgTextTreeChange>,
    pub svg_text_will_be_removed_from_tree: Option<SvgTextTreeChange>,
    pub svg_text_needs_metrics_update: Option<SvgTextMetricsInvalidator>,
    pub svg_inline_text_update_scaled_font: Option<SvgInlineTextFontUpdater>,
    pub svg_inline_text_scaled_font: Option<SvgInlineTextScaledFont>,
    pub svg_inline_text_scaling_factor: Option<SvgInlineTextScalingFactor>,
    pub svg_root_add_child: Option<SvgRootChildInserter>,
    pub svg_root_remove_child: Option<SvgRootChildRemover>,
    pub svg_root_children: Option<SvgRootChildren>,
    pub invalidate_column_spanners: Option<ColumnSpannerInvalidator>,

    pub forms_metadata: Option<ElementMetadataFactory>,
    pub frameset_metadata: Option<ElementMetadataFactory>,
    pub mathml_metadata: Option<ElementMetadataFactory>,
}

impl LayoutObjectFactorySet {
    /// Reject an object factory installed without the virtual overrides its
    /// objects dispatch to. A missing override would otherwise surface only
    /// when the first such object reaches that virtual call.
    pub fn validate_virtual_overrides(&self) {
        let grid_objects = self.grid.is_some() || self.anonymous_grid.is_some();
        assert_eq!(
            grid_objects,
            self.grid_virtuals.is_some(),
            "LayoutGrid factories and grid_virtuals must be installed together"
        );
        let flex_objects = self.flex.is_some() || self.anonymous_flex.is_some();
        assert_eq!(
            flex_objects,
            self.flex_stitched_row_gap_index.is_some(),
            "LayoutFlexibleBox factories and flex_stitched_row_gap_index must be installed together"
        );
        if self.svg.is_some() {
            assert!(
                self.svg_root_children.is_some()
                    && self.svg_inline_text_scaled_font.is_some()
                    && self.svg_inline_text_scaling_factor.is_some(),
                "the SVG factory requires svg_root_children and the SVG inline text font overrides"
            );
        }
    }
}
