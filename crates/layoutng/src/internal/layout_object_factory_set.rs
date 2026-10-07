use layoutng_style::style::computed_style::{ComputedStyle, ComputedStyleBuilder};
use layoutng_style::style::style_difference::StyleDifference;

use super::layout_block::LayoutBlock;
use super::layout_block_flow::LayoutBlockFlow;
use super::layout_box::LayoutBox;
use super::layout_input::NativeNodeConstructionData;
use super::layout_node_metadata::{Element, Node};
use super::layout_object::LayoutObject;

// cpp: layoutng/internal/layout_object_factory_set.h:19-43
pub type LayoutObjectFactory = fn(&mut Node, &ComputedStyle) -> *mut LayoutObject;
pub type ElementMetadataFactory =
    fn(&NativeNodeConstructionData, *const ComputedStyle) -> *mut Element;
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
pub type MulticolObjectUpdater =
    fn(&mut LayoutBlockFlow, &StyleDifference, *const ComputedStyle, &ComputedStyle);
pub type InlineObjectStyleUpdater = fn(&mut LayoutBlockFlow, *const ComputedStyle, &ComputedStyle);
pub type AnonymousTextCombineStyleUpdater = fn(&mut LayoutObject, &mut ComputedStyleBuilder);
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

// C++ derived table vtables live in the table package. This typed callback
// record preserves those overrides without a Cargo dependency cycle. Every
// entry is required when the table object owner is installed.
#[derive(Clone, Copy)]
pub struct TableObjectVirtuals {
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
    pub border_outsets: fn(&LayoutBox) -> layoutng_geometry::geometry::box_strut::PhysicalBoxStrut,
    pub padding_outsets: fn(&LayoutBox) -> layoutng_geometry::geometry::box_strut::PhysicalBoxStrut,
    pub sticky_container: fn(&LayoutBox) -> *mut LayoutBlock,
    pub invalidate_after_measure: fn(&LayoutBox),
    pub intrinsic_borders:
        fn(&LayoutBox) -> *const layoutng_geometry::geometry::box_strut::BoxStrut,
    pub set_intrinsic_borders:
        fn(&mut LayoutBox, &layoutng_geometry::geometry::box_strut::BoxStrut),
    pub stitched_size: fn(&LayoutBox) -> foundation::PhysicalSize,
    pub physical_location: fn(&LayoutBox) -> foundation::PhysicalOffset,
    pub overflow_clip_rect:
        fn(&LayoutBox, foundation::OverlayScrollbarClipBehavior) -> foundation::PhysicalRect,
    pub update_from_element: fn(&mut LayoutObject),
}

// cpp: layoutng/internal/layout_object_factory_set.h:45-85
#[derive(Clone, Copy, Default)]
pub struct LayoutObjectFactorySet {
    pub table_virtuals: Option<TableObjectVirtuals>,
    pub block_ruby: Option<LayoutObjectFactory>,
    pub inline_box: Option<LayoutObjectFactory>,
    pub inline_line_break: Option<LayoutObjectFactory>,
    pub inline_text: Option<LayoutObjectFactory>,
    pub inline_word_break: Option<LayoutObjectFactory>,
    pub replaced: Option<LayoutObjectFactory>,
    pub flex: Option<LayoutObjectFactory>,
    pub grid: Option<LayoutObjectFactory>,
    pub grid_lanes: Option<LayoutObjectFactory>,
    pub table: Option<LayoutObjectFactory>,
    pub custom: Option<LayoutObjectFactory>,
    pub forms: Option<LayoutObjectFactory>,
    pub frameset: Option<LayoutObjectFactory>,
    pub list: Option<LayoutObjectFactory>,
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
    pub svg_block_text_scale: Option<SvgBlockTextScale>,
    pub update_svg_block_style: Option<SvgBlockStyleUpdater>,
    pub svg_group_add_child: Option<SvgGroupChildInserter>,
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
    pub update_multicol: Option<MulticolObjectUpdater>,
    pub invalidate_column_spanners: Option<ColumnSpannerInvalidator>,

    pub forms_metadata: Option<ElementMetadataFactory>,
    pub frameset_metadata: Option<ElementMetadataFactory>,
    pub mathml_metadata: Option<ElementMetadataFactory>,
}
