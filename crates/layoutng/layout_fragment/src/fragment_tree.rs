use std::sync::{Arc, OnceLock};

pub use layout_fragment_boundary::{
    BackgroundPaintLocation, CollapsedTableEdge, CollapsedTablePaintData, CollapsedTableSection,
    ColumnRule, ColumnRulePaintData, FieldsetPaintData, FormControlPaintData, FragmentBoxSides,
    FragmentKind, FragmentPaintRect, FrameSetPaintData, InlineGeometry, InlineGeometryRect,
    LayoutResultHandle, MathMLPaintData, MathMLPaintKind, PaintGlyph, PaintGlyphRun,
    PaintResources, PrePaintInvalidation, ReplacedContentPaintData, ScrollContainerPaintData,
    ScrollbarPaintAxis, ScrollbarPaintData, StickyAxisPaintData, StickyPositionPaintData,
    StitchedDecorationData, SvgTextPaintData, SvgViewportGeometry, TextCombineEmphasisPaintData,
    TextCombinePaintData, TextControlCaretMetrics,
};

#[path = "pre_paint_revision.rs"]
pub mod pre_paint_revision;
pub use pre_paint_revision::SamePrePaintInput;

pub use foundation::graphics_types::graphics::paint::display_item_client_types::RasterEffectOutset;
use foundation::{LayoutOffset as Offset, LayoutSize as Size};
use layoutng::internal::layout_input::{
    BorderLineStyle, BoxDecorationBreak, Display, Edges, FloatSide, ListStyleType, NodeKind,
    Overflow, OverscrollBehavior, PaintStyle, Position, SvgShapeData, SvgViewBoxData,
    TextDirection, TransformMatrix, WritingMode,
};

// The C++ definitions are public API; all layoutng::* names above retain their
// owner in the later layoutng crate. The Cargo cycle needs an interface split.

// cpp: layoutng_fragment_tree/fragment_tree.h:25
// Defaulted C++ equality maps to the derived fieldwise comparison above.

// cpp: layoutng_fragment_tree/fragment_tree.h:131
// Defaulted C++ equality maps to the derived fieldwise comparison above.

/// Assembly specialization of fragment-owned table geometry with LayoutNG's
/// immutable paint-style snapshot. Existing consumers keep the original API.
pub type TablePaintColumn = layout_fragment_boundary::TablePaintColumn<PaintStyle>;
pub type TablePaintData = layout_fragment_boundary::TablePaintData<PaintStyle>;

// The native fragment is the complete paint input. Shared payloads use Arc
// because C++ shared_ptr<const T> is shared immutable ownership.
// cpp: layoutng_fragment_tree/fragment_tree.h:32-73
// cpp: layoutng_fragment_tree/fragment_tree.h:82-114
// cpp: layoutng_fragment_tree/fragment_tree.h:119-120
// cpp: layoutng_fragment_tree/fragment_tree.h:281-301
/// CSS canvas background selected by Layout from the document element or its
/// first body element.  It is owned by the root fragment because Paint must
/// cover the viewport and must not repaint the source element's own box.
///
/// This is the package-boundary equivalent of Blink's propagated LayoutView
/// background.  Keeping the selected immutable style here avoids asking Paint
/// to rediscover DOM roles or containment rules.
pub type CanvasBackgroundPaintData =
    layout_fragment_boundary::CanvasBackgroundPaintData<PaintStyle>;

/// Assembly specializations preserve the historical non-generic API while
/// fragment-local storage and sharing remain owned by `layout_fragment`.
pub type PaintPropertiesHandle = layout_fragment_boundary::PaintPropertiesHandle<PaintProperties>;
pub type FragmentPaintData = layout_fragment_boundary::FragmentPaintData<PaintProperties>;

#[derive(Clone, PartialEq)]
pub struct PaintProperties {
    /// Native LayoutObject dirty bits consumed by Chromium's
    /// PrePaintTreeWalk. They are lifecycle facts, not drawing inputs: the
    /// layout boundary snapshots them before Paint and clears them only after
    /// the corresponding artifact commits.
    pub pre_paint_invalidation: PrePaintInvalidation,
    pub property_fragment_unique_id: u64,
    pub is_layout_view: bool,
    /// Source: core/frame/local_frame_view.cc:1504-1527. Published only by the
    /// actual LayoutView after querying native fixed-background membership.
    pub requires_main_thread_scrolling_for_background_attachment_fixed: Option<bool>,
    /// PaintController snapshots the client's actual reason when appending an
    /// item (paint_controller.cc:531-545). None is an unexported external input.
    pub display_item_client_invalidation_reason: Option<foundation::PaintInvalidationReason>,
    pub display_item_client_debug_name: std::sync::Arc<str>,
    pub paint_layer_client_invalidation_reason: Option<foundation::PaintInvalidationReason>,
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
    // node ids and the native occurrence's fragment_instance_id.
    // cpp: platform/graphics/paint/display_item_client.h:33-35,71-82
    pub display_item_client_id: u64,
    /// Chromium DisplayItemClient::OwnerNodeId(), kept distinct from the
    /// layout/export node identity above. DOMNodeId is a signed 32-bit type.
    pub display_item_client_owner_node_id: i32,
    // A distinct persistent native PaintLayer DisplayItemClient, when present.
    // cpp: core/paint/paint_layer.h:168-171
    /// Native layer ownership was queried, including the authoritative no-layer result.
    /// False is reserved for standalone fragments without a native Layout owner.
    pub paint_layer_state_is_native: bool,
    pub paint_layer_client_id: u64,
    pub paint_layer_client_is_cacheable: bool,
    pub paint_layer_client_is_just_created: std::cell::Cell<bool>,
    pub paint_layer_is_self_painting: bool,
    /// Canonical LayoutObject/PaintLayer stacking metadata. Paint must not
    /// reconstruct these from the projected style because native adjustment
    /// includes flex/grid items, containment and other grouping properties.
    pub paint_layer_is_stacking_context: bool,
    pub paint_layer_stacking_level: i32,
    pub display_item_client_is_cacheable: bool,
    pub display_item_client_is_just_created: std::cell::Cell<bool>,
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
    /// Native LayoutSVGHiddenContainer semantics: retain SVG layout and
    /// resource descendants, but omit this subtree from ordinary paint/hits.
    pub svg_hidden_container: bool,
    pub painted_atomically: bool,
    pub has_collapsed_borders: bool,
    pub source_kind: NodeKind,
    pub effective_zoom: f32,
    pub style: PaintStyle,
    /// True for an element whose CSS background is painted by the root canvas
    /// painter. Borders, shadows and outlines remain ordinary box decoration.
    pub background_transfers_to_view: bool,
    /// Present only on the root LayoutView fragment.
    pub canvas_background: Option<CanvasBackgroundPaintData>,
    pub background_paint_location: BackgroundPaintLocation,
    /// LayoutObject::CanCompositeBackgroundAttachmentFixed is the canonical
    /// eligibility decision. It includes the one-layer, clip, shadow,
    /// appearance and border-radius restrictions required by BackgroundClip.
    pub can_composite_background_attachment_fixed: bool,
    pub margin: Edges,
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
    pub sticky_constraint: Option<StickyPositionPaintData>,
    pub overflow_x: Overflow,
    pub overflow_y: Overflow,
    pub overscroll_behavior_x: OverscrollBehavior,
    pub overscroll_behavior_y: OverscrollBehavior,
    /// Source: core/paint/paint_property_tree_builder.cc:3634-3635.
    pub prevent_scroll_axis_locking: bool,
    /// CSS 2.1 `clip: rect(...)` in the box's local stitched coordinate space.
    /// This is distinct from overflow clipping: it clips the box's own paint
    /// as well as descendants and is ordered between Effect and Filter.
    pub css_clip: Option<FragmentPaintRect>,
    pub overflow_clip_margin_outsets: Option<Edges>,
    pub scroll_offset: Offset,
    pub scroll_size: Size,
    /// Native CSSOM client dimensions, excluding classic scrollbar gutters.
    pub client_size: Option<Size>,
    // Native scrollable area, including viewport-propagated overflow. CSS
    // input overflow alone does not identify the LayoutView's scrolling owner.
    pub scroll_container: Option<ScrollContainerPaintData>,
    pub writing_mode: WritingMode,
    pub direction: TextDirection,
    pub first_baseline: Option<f64>,
    pub text_line_top_offset: Option<f64>,
    pub text_line_height: Option<f64>,
    pub text_combine: Option<TextCombinePaintData>,
    /// LayoutTextCombine::AdjustTextLeftForPaint, quantized as LayoutUnit.
    pub text_combine_inline_offset: f64,
    pub replaced_content: Option<ReplacedContentPaintData>,
    pub resources: Option<Arc<PaintResources>>,
    pub glyph_runs: Arc<Vec<PaintGlyphRun>>,
    pub text_control_host: Option<u64>,
    pub text_control_inner_editor: bool,
    pub text_control_empty_caret: Option<FragmentPaintRect>,
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
    pub svg_local_transform: Option<TransformMatrix>,
    pub svg_viewport: Option<SvgViewportGeometry>,
    pub svg_text: Option<Arc<SvgTextPaintData>>,
    pub table_cell_column: Option<usize>,
}

impl Default for PaintProperties {
    fn default() -> Self {
        static EMPTY_DEBUG_NAME: OnceLock<Arc<str>> = OnceLock::new();
        static EMPTY_GLYPH_RUNS: OnceLock<Arc<Vec<PaintGlyphRun>>> = OnceLock::new();
        Self {
            pre_paint_invalidation: PrePaintInvalidation::default(),
            property_fragment_unique_id: 0,
            is_layout_view: false,
            requires_main_thread_scrolling_for_background_attachment_fixed: None,
            display_item_client_invalidation_reason: None,
            display_item_client_debug_name: Arc::clone(
                EMPTY_DEBUG_NAME.get_or_init(|| Arc::from("")),
            ),
            paint_layer_client_invalidation_reason: None,
            self_ink_overflow: None,
            contents_ink_overflow: None,
            ink_overflow: None,
            text_glyph_ink: None,
            box_self_visual_overflow: None,
            box_fragment_is_inline_box: false,
            display_item_raster_effect_outset: RasterEffectOutset::kNone,
            display_item_client_id: 0,
            display_item_client_owner_node_id: 0,
            paint_layer_state_is_native: false,
            paint_layer_client_id: 0,
            paint_layer_client_is_cacheable: false,
            paint_layer_client_is_just_created: std::cell::Cell::new(false),
            paint_layer_is_self_painting: false,
            paint_layer_is_stacking_context: false,
            paint_layer_stacking_level: 0,
            display_item_client_is_cacheable: false,
            display_item_client_is_just_created: std::cell::Cell::new(false),
            display_item_fragment: 0,
            has_source: false,
            logical_parent_node_id: None,
            logical_parent_fragment_instance_id: None,
            logical_tree_order: 0,
            establishes_paint_state: false,
            hidden: false,
            svg_hidden_container: false,
            painted_atomically: false,
            has_collapsed_borders: false,
            source_kind: NodeKind::kBox,
            effective_zoom: 1.0,
            style: PaintStyle::default(),
            background_transfers_to_view: false,
            canvas_background: None,
            background_paint_location: BackgroundPaintLocation::BorderBox,
            can_composite_background_attachment_fixed: false,
            margin: Edges::default(),
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
            sticky_constraint: None,
            overflow_x: Overflow::kVisible,
            overflow_y: Overflow::kVisible,
            overscroll_behavior_x: Default::default(),
            overscroll_behavior_y: Default::default(),
            prevent_scroll_axis_locking: false,
            css_clip: None,
            overflow_clip_margin_outsets: None,
            scroll_offset: Offset::default(),
            scroll_size: Size::default(),
            client_size: None,
            scroll_container: None,
            writing_mode: WritingMode::kHorizontalTb,
            direction: TextDirection::kLtr,
            first_baseline: None,
            text_line_top_offset: None,
            text_line_height: None,
            text_combine: None,
            text_combine_inline_offset: 0.0,
            replaced_content: None,
            resources: None,
            glyph_runs: Arc::clone(EMPTY_GLYPH_RUNS.get_or_init(|| Arc::new(Vec::new()))),
            text_control_host: None,
            text_control_inner_editor: false,
            text_control_empty_caret: None,
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
            svg_local_transform: None,
            svg_viewport: None,
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
