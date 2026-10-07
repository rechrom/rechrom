use std::sync::Arc;

#[path = "pre_paint_revision.rs"]
pub mod pre_paint_revision;
pub use pre_paint_revision::SamePrePaintInput;

use layoutng::internal::form_control_types::FormControlType;
use layoutng::internal::layout_input::{
    BorderLineStyle, BoxDecorationBreak, Display, Edges, FloatSide, FontFace, FontSmoothing,
    FontVariation, ListStyleType, NodeKind, Offset, Overflow, PaintImage, PaintStyle, Position,
    Size, SvgShapeData, SvgViewBoxData, TextDirection, TransformMatrix, ViewportGeometry,
    WritingMode,
};
use layoutng::internal::layout_input_types::Color;
use layoutng_style::style::appearance::AppearanceValue;
pub use foundation::graphics_types::graphics::paint::display_item_client_types::RasterEffectOutset;

/// A native physical ink rect, relative to the owning fragment's origin.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FragmentPaintRect {
    pub offset: Offset,
    pub size: Size,
}

// The C++ definitions are public API; all layoutng::* names above retain their
// owner in the later layoutng crate. The Cargo cycle needs an interface split.

// cpp: layoutng_fragment_tree/fragment_tree.h:7
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum FragmentKind {
    kBox,
    kLine,
    kText,
    kGeneratedText,
}

impl Default for FragmentKind {
    fn default() -> Self {
        Self::kBox
    }
}

// cpp: layoutng_fragment_tree/fragment_tree.h:12-17
#[derive(Clone, PartialEq)]
pub struct PaintResources {
    pub fonts: Vec<FontFace>,
    pub images: Vec<PaintImage>,
    pub device_pixel_ratio: f64,
    pub viewport: Option<ViewportGeometry>,
}

impl Default for PaintResources {
    fn default() -> Self {
        Self {
            fonts: Vec::new(),
            images: Vec::new(),
            device_pixel_ratio: 1.0,
            viewport: None,
        }
    }
}

// cpp: layoutng_fragment_tree/fragment_tree.h:19-23
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FragmentBoxSides {
    pub top: bool,
    pub right: bool,
    pub bottom: bool,
    pub left: bool,
}

impl Default for FragmentBoxSides {
    fn default() -> Self {
        Self {
            top: true,
            right: true,
            bottom: true,
            left: true,
        }
    }
}

#[allow(non_snake_case)]
impl FragmentBoxSides {
    // cpp: layoutng_fragment_tree/fragment_tree.h:24
    pub fn HasAllSides(&self) -> bool {
        self.top && self.right && self.bottom && self.left
    }
}

// cpp: layoutng_fragment_tree/fragment_tree.h:25
// Defaulted C++ equality maps to the derived fieldwise comparison above.

// C++ nests these payloads under PaintProperties. Rust names retain that owner
// while keeping each type directly addressable by the public API.
// cpp: layoutng_fragment_tree/fragment_tree.h:74-81
#[derive(Clone, Default, PartialEq)]
pub struct StitchedDecorationData {
    pub fragment_origin: Offset,
    pub fragment_offset: Offset,
    pub size: Size,
}

// cpp: layoutng_fragment_tree/fragment_tree.h:115-118
#[derive(Clone, Default, PartialEq)]
pub struct ReplacedContentPaintData {
    pub offset: Offset,
    pub size: Size,
}

// cpp: layoutng_fragment_tree/fragment_tree.h:123-130
#[derive(Clone, Default, PartialEq)]
pub struct PaintGlyph {
    pub id: u32,
    pub character_index: u32,
    pub canvas_rotation: u8,
    pub offset: Offset,
    pub advance: f64,
}

// cpp: layoutng_fragment_tree/fragment_tree.h:131
// Defaulted C++ equality maps to the derived fieldwise comparison above.

// cpp: layoutng_fragment_tree/fragment_tree.h:133-153
#[derive(Clone, PartialEq)]
pub struct PaintGlyphRun {
    pub font_face_index: u32,
    pub font_variations: Vec<FontVariation>,
    pub font_size: f64,
    pub ascent: f64,
    pub baseline: f64,
    pub underline_position: f64,
    pub underline_thickness: f64,
    pub strikeout_position: f64,
    pub strikeout_thickness: f64,
    pub horizontal: bool,
    pub rtl: bool,
    pub synthetic_bold: bool,
    pub synthetic_italic: bool,
    pub font_smoothing: FontSmoothing,
    pub writing_mode: WritingMode,
    pub glyphs: Vec<PaintGlyph>,
}

impl Default for PaintGlyphRun {
    fn default() -> Self {
        Self {
            font_face_index: 0,
            font_variations: Vec::new(),
            font_size: 16.0,
            ascent: 0.0,
            baseline: 0.0,
            underline_position: 0.0,
            underline_thickness: 1.0,
            strikeout_position: 0.0,
            strikeout_thickness: 1.0,
            horizontal: true,
            rtl: false,
            synthetic_bold: false,
            synthetic_italic: false,
            font_smoothing: FontSmoothing::kAuto,
            writing_mode: WritingMode::kHorizontalTb,
            glyphs: Vec::new(),
        }
    }
}

// cpp: layoutng_fragment_tree/fragment_tree.h:155
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum MathMLPaintKind {
    kFraction,
    kOperator,
    kRadical,
}

impl Default for MathMLPaintKind {
    fn default() -> Self {
        Self::kFraction
    }
}

// cpp: layoutng_fragment_tree/fragment_tree.h:154-167
#[derive(Clone, Default, PartialEq)]
pub struct MathMLPaintData {
    pub kind: MathMLPaintKind,
    pub axis_height: f64,
    pub rule_thickness: f64,
    pub vertical_gap: f64,
    pub operator_inline_size: f64,
    pub operator_ascent: f64,
    pub operator_descent: f64,
    pub radical_margin_inline_start: f64,
    pub radical_margin_inline_end: f64,
    pub radical_operator_inline_offset: f64,
    pub operator_glyph_runs: Vec<PaintGlyphRun>,
}

// cpp: layoutng_fragment_tree/fragment_tree.h:171-178
#[derive(Clone, Default, PartialEq)]
pub struct FrameSetPaintData {
    pub column_sizes: Vec<f64>,
    pub row_sizes: Vec<f64>,
    pub column_allows_border: Vec<bool>,
    pub row_allows_border: Vec<bool>,
    pub border_thickness: f64,
    pub has_border_color: bool,
}

// cpp: layoutng_fragment_tree/fragment_tree.h:179-184
#[derive(Clone, Default, PartialEq)]
pub struct FieldsetPaintData {
    pub border_outsets: Edges,
    pub has_legend: bool,
    pub legend_cutout_offset: Offset,
    pub legend_cutout_size: Size,
}

// cpp: layoutng_fragment_tree/fragment_tree.h:186-189
#[derive(Clone, Default, PartialEq)]
pub struct ColumnRule {
    pub offset: Offset,
    pub size: Size,
}

// cpp: layoutng_fragment_tree/fragment_tree.h:185-191
#[derive(Clone, Default, PartialEq)]
pub struct ColumnRulePaintData {
    pub rules: Vec<ColumnRule>,
}

// cpp: layoutng_fragment_tree/fragment_tree.h:193-201
#[derive(Clone, PartialEq)]
pub struct CollapsedTableEdge {
    pub exists: bool,
    pub can_paint: bool,
    pub width: f64,
    pub style: BorderLineStyle,
    pub style_rank: u8,
    pub color: Color,
    pub box_order: usize,
}

impl Default for CollapsedTableEdge {
    fn default() -> Self {
        Self {
            exists: false,
            can_paint: false,
            width: 0.0,
            style: BorderLineStyle::kNone,
            style_rank: 0,
            color: Color::default(),
            box_order: 0,
        }
    }
}

// cpp: layoutng_fragment_tree/fragment_tree.h:202-209
#[derive(Clone, Default, PartialEq)]
pub struct CollapsedTableSection {
    pub offset: Offset,
    pub size: Size,
    pub start_row: usize,
    pub row_offsets: Vec<f64>,
    pub start_row_fragmented: bool,
    pub end_row_fragmented: bool,
}

// cpp: layoutng_fragment_tree/fragment_tree.h:192-214
#[derive(Clone, Default, PartialEq)]
pub struct CollapsedTablePaintData {
    pub edges_per_row: usize,
    pub edges: Vec<CollapsedTableEdge>,
    pub columns: Vec<f64>,
    pub sections: Vec<CollapsedTableSection>,
}

// cpp: layoutng_fragment_tree/fragment_tree.h:216-223
#[derive(Clone, PartialEq)]
pub struct TablePaintColumn {
    pub start_column: usize,
    pub span: usize,
    pub inline_offset: f64,
    pub inline_size: f64,
    pub node_id: u64,
    pub style: PaintStyle,
}

impl Default for TablePaintColumn {
    fn default() -> Self {
        Self {
            start_column: 0,
            span: 1,
            inline_offset: 0.0,
            inline_size: 0.0,
            node_id: 0,
            style: PaintStyle::default(),
        }
    }
}

// cpp: layoutng_fragment_tree/fragment_tree.h:215-229
#[derive(Clone, Default, PartialEq)]
pub struct TablePaintData {
    pub grid_offset: Offset,
    pub grid_size: Size,
    pub columns_offset: Offset,
    pub columns_size: Size,
    pub columns: Vec<TablePaintColumn>,
}

// cpp: layoutng_fragment_tree/fragment_tree.h:231-238
#[derive(Clone, Default, PartialEq)]
pub struct ScrollbarPaintAxis {
    pub display_item_client_id: u64,
    pub display_item_client_is_cacheable: bool,
    pub display_item_client_is_just_created: bool,
    pub track_offset: Offset,
    pub track_size: Size,
    pub thumb_offset: Offset,
    pub thumb_size: Size,
    pub horizontal: bool,
    pub has_buttons: bool,
}

// cpp: layoutng_fragment_tree/fragment_tree.h:230-248
#[derive(Clone, Default, PartialEq)]
pub struct ScrollbarPaintData {
    pub corner_client_id: u64,
    pub corner_client_is_cacheable: bool,
    pub corner_client_is_just_created: bool,
    pub horizontal: Option<ScrollbarPaintAxis>,
    pub vertical: Option<ScrollbarPaintAxis>,
    pub uses_overlay_scrollbars: bool,
    pub corner_offset: Offset,
    pub corner_size: Size,
    pub track_color: Color,
    pub thumb_color: Color,
    pub button_color: Color,
    pub corner_color: Color,
}

// cpp: layoutng_fragment_tree/fragment_tree.h:252-268
#[derive(Clone, PartialEq)]
pub struct FormControlPaintData {
    pub appearance: AppearanceValue,
    pub r#type: Option<FormControlType>,
    pub checked: bool,
    pub indeterminate: bool,
    pub disabled: bool,
    pub read_only: bool,
    pub hovered: bool,
    pub active: bool,
    pub focused: bool,
    pub auto_focus_ring: bool,
    pub autofilled: bool,
    pub value_ratio: Option<f64>,
    pub accent_color: Color,
}

impl Default for FormControlPaintData {
    fn default() -> Self {
        Self {
            appearance: AppearanceValue::kNone,
            r#type: None,
            checked: false,
            indeterminate: false,
            disabled: false,
            read_only: false,
            hovered: false,
            active: false,
            focused: false,
            auto_focus_ring: false,
            autofilled: false,
            value_ratio: None,
            accent_color: Color {
                red: 0.0,
                green: 117.0 / 255.0,
                blue: 1.0,
                alpha: 1.0,
            },
        }
    }
}

// cpp: layoutng_fragment_tree/fragment_tree.h:273-280
#[derive(Clone, PartialEq)]
pub struct SvgTextPaintData {
    pub glyph_origin: Offset,
    pub untransformed_size: Size,
    pub local_transform: TransformMatrix,
    pub scaling_factor: f64,
    pub has_transform: bool,
    pub hidden: bool,
}

impl Default for SvgTextPaintData {
    fn default() -> Self {
        Self {
            glyph_origin: Offset::default(),
            untransformed_size: Size::default(),
            local_transform: TransformMatrix::default(),
            scaling_factor: 1.0,
            has_transform: false,
            hidden: false,
        }
    }
}

/// Font geometry required to position a caret even when the editor has no glyphs.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextControlCaretMetrics {
    pub font_height: f64,
    pub ascent: f64,
    pub line_height: f64,
}

// The native fragment is the complete paint input. Shared payloads use Arc
// because C++ shared_ptr<const T> is shared immutable ownership.
// cpp: layoutng_fragment_tree/fragment_tree.h:32-73
// cpp: layoutng_fragment_tree/fragment_tree.h:82-114
// cpp: layoutng_fragment_tree/fragment_tree.h:119-120
// cpp: layoutng_fragment_tree/fragment_tree.h:281-301
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollContainerPaintData {
    pub container_rect: FragmentPaintRect,
    pub user_scrollable_horizontal: bool,
    pub user_scrollable_vertical: bool,
}

#[derive(Clone, PartialEq)]
pub struct PaintProperties {
    // None means native ink overflow has not been computed. Never substitute
    // a frame rect here: text glyphs and decorations can extend outside it.
    pub self_ink_overflow: Option<FragmentPaintRect>,
    pub contents_ink_overflow: Option<FragmentPaintRect>,
    pub ink_overflow: Option<FragmentPaintRect>,
    // Baseline-adjusted, writing-mode converted glyph bounds. This remains
    // available when full text decoration/emphasis overflow is not implemented.
    pub text_glyph_ink: Option<FragmentPaintRect>,
    // BoxPainter::VisualRect uses LayoutBox::SelfVisualOverflowRect(), which
    // is a stitched LayoutBox rect rather than a physical fragment ink rect.
    pub box_self_visual_overflow: Option<FragmentPaintRect>,
    pub box_fragment_is_inline_box: bool,
    pub display_item_raster_effect_outset: RasterEffectOutset,
    // Native DisplayItemClient identity and lifecycle, independently of DOM
    // node ids and preorder-only exported fragment_instance_id.
    // cpp: platform/graphics/paint/display_item_client.h:33-35,71-82
    pub display_item_client_id: u64,
    // A distinct persistent native PaintLayer DisplayItemClient, when present.
    // cpp: core/paint/paint_layer.h:168-171
    pub paint_layer_client_id: u64,
    pub paint_layer_client_is_cacheable: bool,
    pub paint_layer_client_is_just_created: bool,
    pub paint_layer_is_self_painting: bool,
    pub display_item_client_is_cacheable: bool,
    pub display_item_client_is_just_created: bool,
    // The active ScopedDisplayItemFragment value: inline FragmentId(), layer
    // fragment index, or the containing fragmentainer's break-token identity.
    // cpp: core/layout/inline/fragment_item.h:172-188
    // cpp: core/paint/box_fragment_painter.cc:394-403
    pub display_item_fragment: u32,
    pub has_source: bool,
    pub logical_parent_node_id: Option<u64>,
    pub logical_parent_fragment_instance_id: Option<u64>,
    pub logical_tree_order: u64,
    pub establishes_paint_state: bool,
    pub hidden: bool,
    pub painted_atomically: bool,
    pub has_collapsed_borders: bool,
    pub source_kind: NodeKind,
    pub effective_zoom: f32,
    pub style: PaintStyle,
    pub border: Edges,
    pub border_sides: FragmentBoxSides,
    pub border_styles: [BorderLineStyle; 4],
    pub padding: Edges,
    pub stitched_decoration: Option<StitchedDecorationData>,
    pub box_decoration_break: BoxDecorationBreak,
    pub display: Display,
    pub position: Position,
    // A fixed-position LayoutBox whose containing block is the LayoutView.
    // PrePaint switches this box above the document scroll translation.
    pub fixed_to_view: bool,
    pub floating: FloatSide,
    pub sticky_offset: Offset,
    pub overflow_x: Overflow,
    pub overflow_y: Overflow,
    pub overflow_clip_margin_outsets: Option<Edges>,
    pub scroll_offset: Offset,
    pub scroll_size: Size,
    // Native scrollable area, including viewport-propagated overflow. CSS
    // input overflow alone does not identify the LayoutView's scrolling owner.
    pub scroll_container: Option<ScrollContainerPaintData>,
    pub writing_mode: WritingMode,
    pub direction: TextDirection,
    pub first_baseline: Option<f64>,
    pub text_line_top_offset: Option<f64>,
    pub replaced_content: Option<ReplacedContentPaintData>,
    pub resources: Option<Arc<PaintResources>>,
    pub glyph_runs: Vec<PaintGlyphRun>,
    pub list_marker_symbol: Option<ListStyleType>,
    pub list_marker_inside: bool,
    pub mathml: Option<Arc<MathMLPaintData>>,
    pub frame_set: Option<Arc<FrameSetPaintData>>,
    pub fieldset: Option<Arc<FieldsetPaintData>>,
    pub column_rules: Option<Arc<ColumnRulePaintData>>,
    pub collapsed_table: Option<Arc<CollapsedTablePaintData>>,
    pub table: Option<Arc<TablePaintData>>,
    pub scrollbars: Option<Arc<ScrollbarPaintData>>,
    pub form_control: Option<Arc<FormControlPaintData>>,
    pub text_control_caret_metrics: Option<TextControlCaretMetrics>,
    pub svg_shape: Option<Arc<SvgShapeData>>,
    pub svg_view_box: Option<SvgViewBoxData>,
    pub svg_text: Option<Arc<SvgTextPaintData>>,
    pub table_cell_column: Option<usize>,
}

impl Default for PaintProperties {
    fn default() -> Self {
        Self {
            self_ink_overflow: None,
            contents_ink_overflow: None,
            ink_overflow: None,
            text_glyph_ink: None,
            box_self_visual_overflow: None,
            box_fragment_is_inline_box: false,
            display_item_raster_effect_outset: RasterEffectOutset::kNone,
            display_item_client_id: 0,
            paint_layer_client_id: 0,
            paint_layer_client_is_cacheable: false,
            paint_layer_client_is_just_created: false,
            paint_layer_is_self_painting: false,
            display_item_client_is_cacheable: false,
            display_item_client_is_just_created: false,
            display_item_fragment: 0,
            has_source: false,
            logical_parent_node_id: None,
            logical_parent_fragment_instance_id: None,
            logical_tree_order: 0,
            establishes_paint_state: false,
            hidden: false,
            painted_atomically: false,
            has_collapsed_borders: false,
            source_kind: NodeKind::kBox,
            effective_zoom: 1.0,
            style: PaintStyle::default(),
            border: Edges::default(),
            border_sides: FragmentBoxSides::default(),
            border_styles: [BorderLineStyle::kNone; 4],
            padding: Edges::default(),
            stitched_decoration: None,
            box_decoration_break: BoxDecorationBreak::kSlice,
            display: Display::kBlock,
            position: Position::kStatic,
            fixed_to_view: false,
            floating: FloatSide::kNone,
            sticky_offset: Offset::default(),
            overflow_x: Overflow::kVisible,
            overflow_y: Overflow::kVisible,
            overflow_clip_margin_outsets: None,
            scroll_offset: Offset::default(),
            scroll_size: Size::default(),
            scroll_container: None,
            writing_mode: WritingMode::kHorizontalTb,
            direction: TextDirection::kLtr,
            first_baseline: None,
            text_line_top_offset: None,
            replaced_content: None,
            resources: None,
            glyph_runs: Vec::new(),
            list_marker_symbol: None,
            list_marker_inside: false,
            mathml: None,
            frame_set: None,
            fieldset: None,
            column_rules: None,
            collapsed_table: None,
            table: None,
            scrollbars: None,
            form_control: None,
            text_control_caret_metrics: None,
            svg_shape: None,
            svg_view_box: None,
            svg_text: None,
            table_cell_column: None,
        }
    }
}

#[allow(non_snake_case)]
impl PaintProperties {
    // C++ chained assignment evaluates the right-hand assignment first.
    // cpp: layoutng_fragment_tree/fragment_tree.h:96
    pub fn SetOverflow(&mut self, value: Overflow) {
        self.overflow_y = value;
        self.overflow_x = value;
    }
}

// cpp: layoutng_fragment_tree/fragment_tree.h:303-322
#[derive(Clone)]
pub struct FragmentNode {
    /// Certified layout-output input revision. Zero means unobserved input.
    pub pre_paint_revision: u64,
    /// Includes the exact ordered child input revisions and topology.
    pub pre_paint_subtree_revision: u64,
    /// Boundary proof that the previous native owners are still live. Weak
    /// handles in LayoutEngine establish this; it is not a paint property.
    pub pre_paint_owner_retained: bool,
    pub fragment_instance_id: u64,
    pub fragmentainer_instance_id: u64,
    pub node_id: u64,
    pub kind: FragmentKind,
    pub offset: Offset,
    pub size: Size,
    pub content_size: Size,
    pub text_start: Option<u32>,
    pub text_end: Option<u32>,
    pub paint: PaintProperties,
    pub children: Vec<FragmentNode>,
}

impl Default for FragmentNode {
    fn default() -> Self {
        Self {
            pre_paint_revision: 0,
            pre_paint_subtree_revision: 0,
            pre_paint_owner_retained: false,
            fragment_instance_id: 0,
            fragmentainer_instance_id: 0,
            node_id: 0,
            kind: FragmentKind::kBox,
            offset: Offset::default(),
            size: Size::default(),
            content_size: Size::default(),
            text_start: None,
            text_end: None,
            paint: PaintProperties::default(),
            children: Vec::new(),
        }
    }
}
