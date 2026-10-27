//! Engine-neutral values exposed by the fragment ownership boundary.

use foundation::{
    AppearanceValue, BorderLineStyle, FontSmoothing, FontVariation, FormControlType, LayoutEdges,
    LayoutOffset, LayoutPaintWritingMode, LayoutSize, LayoutViewportGeometry, PaintColor,
    TransformMatrix,
};
use std::cell::Cell;
use std::ops::Deref;
use std::sync::Arc;

/// A native physical ink rect, relative to the owning fragment's origin.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FragmentPaintRect {
    pub offset: LayoutOffset,
    pub size: LayoutSize,
}

/// Immutable resources and host geometry retained with a fragment result.
// cpp: layoutng_fragment_tree/fragment_tree.h:12-17
#[derive(Clone, PartialEq)]
pub struct PaintResources {
    pub fonts: font_engine::FontFaceCatalog,
    pub images: Vec<image_resource::PaintImage>,
    pub device_pixel_ratio: f64,
    pub preferred_color_scheme: u8,
    /// Page/browser zoom already applied to retained Layout coordinates.
    pub layout_zoom_factor: f64,
    pub paints_opaque_base_background: bool,
    pub viewport: Option<LayoutViewportGeometry>,
}

/// Paint-time table-column geometry. The concrete immutable style snapshot is
/// supplied by the assembly boundary so this owner does not depend on the
/// Layout engine's style implementation.
// cpp: layoutng_fragment_tree/fragment_tree.h:216-223
#[derive(Clone, PartialEq)]
pub struct TablePaintColumn<Style> {
    pub start_column: usize,
    pub span: usize,
    pub inline_offset: f64,
    pub inline_size: f64,
    pub node_id: u64,
    pub style: Style,
}

impl<Style: Default> Default for TablePaintColumn<Style> {
    fn default() -> Self {
        Self {
            start_column: 0,
            span: 1,
            inline_offset: 0.0,
            inline_size: 0.0,
            node_id: 0,
            style: Style::default(),
        }
    }
}

/// Fragment-owned table paint topology parameterized by the assembly-owned
/// immutable style representation.
// cpp: layoutng_fragment_tree/fragment_tree.h:215-229
#[derive(Clone, PartialEq)]
pub struct TablePaintData<Style> {
    pub grid_offset: LayoutOffset,
    pub grid_size: LayoutSize,
    pub columns_offset: LayoutOffset,
    pub columns_size: LayoutSize,
    pub columns: Vec<TablePaintColumn<Style>>,
}

impl<Style> Default for TablePaintData<Style> {
    fn default() -> Self {
        Self {
            grid_offset: LayoutOffset::default(),
            grid_size: LayoutSize::default(),
            columns_offset: LayoutOffset::default(),
            columns_size: LayoutSize::default(),
            columns: Vec::new(),
        }
    }
}

/// Root-canvas background selection. Layout supplies its concrete immutable
/// style snapshot without transferring style ownership into this package.
#[derive(Clone, PartialEq)]
pub struct CanvasBackgroundPaintData<Style> {
    pub style: Style,
    pub effective_zoom: f32,
}

impl Default for PaintResources {
    fn default() -> Self {
        Self {
            fonts: font_engine::FontFaceCatalog::default(),
            images: Vec::new(),
            device_pixel_ratio: 1.0,
            preferred_color_scheme: 0,
            layout_zoom_factor: 1.0,
            paints_opaque_base_background: true,
            viewport: None,
        }
    }
}

#[allow(non_snake_case)]
impl PaintResources {
    pub fn ArtifactToDeviceScale(&self) -> f64 {
        self.device_pixel_ratio / self.layout_zoom_factor.max(f64::MIN_POSITIVE)
    }
}

// cpp: layoutng_fragment_tree/fragment_tree.h:7
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum FragmentKind {
    #[default]
    kBox,
    kLine,
    kText,
    kGeneratedText,
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

#[derive(Clone, Default, PartialEq)]
pub struct StitchedDecorationData {
    pub fragment_origin: LayoutOffset,
    pub fragment_offset: LayoutOffset,
    pub size: LayoutSize,
}

#[derive(Clone, PartialEq)]
pub struct ReplacedContentPaintData {
    pub offset: LayoutOffset,
    pub size: LayoutSize,
    /// LayoutReplaced::ClipsToContentBox(). Ordinary images clip regardless
    /// of computed overflow; replaced elements that opt into CSS overflow use
    /// the property-tree overflow clip instead.
    pub clips_to_content_box: bool,
}

impl Default for ReplacedContentPaintData {
    fn default() -> Self {
        Self {
            offset: LayoutOffset::default(),
            size: LayoutSize::default(),
            clips_to_content_box: true,
        }
    }
}

#[derive(Clone, Default, PartialEq)]
pub struct PaintGlyph {
    pub id: u32,
    pub character_index: u32,
    pub canvas_rotation: u8,
    pub offset: LayoutOffset,
    pub advance: f64,
}

/// Immutable native shaping output retained by Paint. The glyph vector is
/// shared so crossing the Layout/Fragment boundary never copies a run.
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
    pub writing_mode: LayoutPaintWritingMode,
    pub glyphs: Arc<Vec<PaintGlyph>>,
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
            writing_mode: LayoutPaintWritingMode::kHorizontalTb,
            glyphs: Arc::new(Vec::new()),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum MathMLPaintKind {
    #[default]
    kFraction,
    kOperator,
    kRadical,
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

/// Native LayoutTextCombine emphasis state carried across the Layout/Paint
/// boundary. PrePaint owns the corresponding transform property node.
#[derive(Clone, Default, PartialEq)]
pub struct TextCombineEmphasisPaintData {
    pub glyph_runs: Vec<PaintGlyphRun>,
    pub origin: LayoutOffset,
    pub color: PaintColor,
}

#[derive(Clone, Default, PartialEq)]
pub struct TextCombinePaintData {
    pub scale_x: Option<f64>,
    pub inline_spacing: f64,
    pub text_top_offset: f64,
    pub synthetic_oblique: bool,
    pub font_size: f64,
    pub emphasis: Option<TextCombineEmphasisPaintData>,
}

#[derive(Clone, Default, PartialEq)]
pub struct FrameSetPaintData {
    pub column_sizes: Vec<f64>,
    pub row_sizes: Vec<f64>,
    pub column_allows_border: Vec<bool>,
    pub row_allows_border: Vec<bool>,
    pub border_thickness: f64,
    pub has_border_color: bool,
}

/// Font geometry required to position a caret even when the editor has no glyphs.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextControlCaretMetrics {
    pub font_height: f64,
    pub ascent: f64,
    pub line_height: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BackgroundPaintLocation {
    #[default]
    BorderBox,
    Contents,
    Both,
}

impl BackgroundPaintLocation {
    pub fn PaintsBorderBox(self) -> bool {
        matches!(self, Self::BorderBox | Self::Both)
    }

    pub fn PaintsContents(self) -> bool {
        matches!(self, Self::Contents | Self::Both)
    }
}

/// Native dirty-bit facts consumed by the fragment-local PrePaint tracker.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PrePaintInvalidation {
    pub needs_paint_property_update: bool,
    pub descendant_needs_paint_property_update: bool,
    pub should_check_for_paint_invalidation: bool,
    pub should_check_layout_for_paint_invalidation: bool,
    pub descendant_should_check_layout_for_paint_invalidation: bool,
    pub subtree_should_check_for_paint_invalidation: bool,
}

impl PrePaintInvalidation {
    pub fn own(self) -> bool {
        self.needs_paint_property_update
            || self.should_check_for_paint_invalidation
            || self.should_check_layout_for_paint_invalidation
            || self.subtree_should_check_for_paint_invalidation
    }

    pub fn descendants(self) -> bool {
        self.descendant_needs_paint_property_update
            || self.descendant_should_check_layout_for_paint_invalidation
            || self.subtree_should_check_for_paint_invalidation
    }
}

#[derive(Clone, Default, PartialEq)]
pub struct InlineGeometryRect {
    pub container_fragment_instance_id: u64,
    pub rect: FragmentPaintRect,
}

#[derive(Clone, Default, PartialEq)]
pub struct InlineGeometry {
    pub node_id: u64,
    pub effective_zoom: f32,
    pub local_bounds: FragmentPaintRect,
    pub rects: Vec<InlineGeometryRect>,
}

/// Shared immutable paint payload. The payload itself stays generic so the
/// fragment owner can retain and share it without importing LayoutNG's
/// concrete style and property graph.
#[derive(Clone, PartialEq)]
pub struct PaintPropertiesHandle<Paint>(Arc<Paint>);

impl<Paint: Default> Default for PaintPropertiesHandle<Paint> {
    fn default() -> Self {
        Self(Arc::new(Paint::default()))
    }
}

impl<Paint> From<Paint> for PaintPropertiesHandle<Paint> {
    fn from(value: Paint) -> Self {
        Self(Arc::new(value))
    }
}

impl<Paint> Deref for PaintPropertiesHandle<Paint> {
    type Target = Paint;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl<Paint: Clone> std::ops::DerefMut for PaintPropertiesHandle<Paint> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::make_mut(&mut self.0)
    }
}

/// Fragment-local geometry, identity, and revision state parameterized by its
/// assembly-owned heavy paint payload.
// cpp: layoutng_fragment_tree/fragment_tree.h:303-322
#[derive(Clone)]
pub struct FragmentPaintData<Paint> {
    pub inline_geometry: Vec<InlineGeometry>,
    pub paint_cull_scroll_offset: Option<LayoutOffset>,
    pub pre_paint_revision: u64,
    pub pre_paint_subtree_revision: u64,
    pub pre_paint_owner_retained: bool,
    pub fragment_instance_id: u64,
    pub fragmentainer_instance_id: u64,
    pub node_id: u64,
    pub kind: FragmentKind,
    pub offset: LayoutOffset,
    pub size: LayoutSize,
    pub content_size: LayoutSize,
    pub text_start: Option<u32>,
    pub text_end: Option<u32>,
    pub paint: PaintPropertiesHandle<Paint>,
}

impl<Paint> FragmentPaintData<Paint> {
    pub fn new(paint: Paint) -> Self {
        Self {
            inline_geometry: Vec::new(),
            paint_cull_scroll_offset: None,
            pre_paint_revision: 0,
            pre_paint_subtree_revision: 0,
            pre_paint_owner_retained: false,
            fragment_instance_id: 0,
            fragmentainer_instance_id: 0,
            node_id: 0,
            kind: FragmentKind::kBox,
            offset: LayoutOffset::default(),
            size: LayoutSize::default(),
            content_size: LayoutSize::default(),
            text_start: None,
            text_end: None,
            paint: paint.into(),
        }
    }
}

impl<Paint: Default> Default for FragmentPaintData<Paint> {
    fn default() -> Self {
        Self::new(Paint::default())
    }
}

impl<Paint> crate::revision::FragmentRevisionNode for FragmentPaintData<Paint> {
    fn pre_paint_revision(&self) -> u64 {
        self.pre_paint_revision
    }

    fn set_pre_paint_revision(&mut self, revision: u64) {
        self.pre_paint_revision = revision;
    }

    fn pre_paint_subtree_revision(&self) -> u64 {
        self.pre_paint_subtree_revision
    }

    fn set_pre_paint_subtree_revision(&mut self, revision: u64) {
        self.pre_paint_subtree_revision = revision;
    }
}

/// Fragmentation metadata needed to paint one collapsed-border table section.
/// It contains no table algorithm or native LayoutObject state.
// cpp: layoutng_fragment_tree/fragment_tree.h:202-209
#[derive(Clone, Default, PartialEq)]
pub struct CollapsedTableSection {
    pub offset: LayoutOffset,
    pub size: LayoutSize,
    pub start_row: usize,
    pub row_offsets: Vec<f64>,
    pub start_row_fragmented: bool,
    pub end_row_fragmented: bool,
}

// cpp: layoutng_fragment_tree/fragment_tree.h:179-184
#[derive(Clone, Default, PartialEq)]
pub struct FieldsetPaintData {
    pub border_outsets: LayoutEdges,
    pub has_legend: bool,
    pub legend_cutout_offset: LayoutOffset,
    pub legend_cutout_size: LayoutSize,
}

// cpp: layoutng_fragment_tree/fragment_tree.h:186-189
#[derive(Clone, PartialEq)]
pub struct ColumnRule {
    pub offset: LayoutOffset,
    pub size: LayoutSize,
    pub color: PaintColor,
    pub line_style: BorderLineStyle,
    pub stroke_width: f64,
}

impl Default for ColumnRule {
    fn default() -> Self {
        Self {
            offset: LayoutOffset::default(),
            size: LayoutSize::default(),
            color: PaintColor::default(),
            line_style: BorderLineStyle::kNone,
            stroke_width: 0.0,
        }
    }
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
    pub color: PaintColor,
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
            color: PaintColor::default(),
            box_order: 0,
        }
    }
}

// cpp: layoutng_fragment_tree/fragment_tree.h:192-214
#[derive(Clone, Default, PartialEq)]
pub struct CollapsedTablePaintData {
    pub edges_per_row: usize,
    pub edges: Vec<CollapsedTableEdge>,
    pub columns: Vec<f64>,
    pub sections: Vec<CollapsedTableSection>,
}

/// Immutable geometry and DisplayItemClient identity for one scrollbar axis.
/// Color/theme selection remains on the enclosing paint payload.
// cpp: layoutng_fragment_tree/fragment_tree.h:231-238
#[derive(Clone, Default, PartialEq)]
pub struct ScrollbarPaintAxis {
    pub compositor_element_id: foundation::CompositorElementId,
    pub display_item_client_invalidation_reason: Option<foundation::PaintInvalidationReason>,
    pub needs_update_display: Option<bool>,
    pub display_item_client_id: u64,
    pub display_item_client_is_cacheable: bool,
    pub display_item_client_is_just_created: Cell<bool>,
    pub track_offset: LayoutOffset,
    pub track_size: LayoutSize,
    pub thumb_offset: LayoutOffset,
    pub thumb_size: LayoutSize,
    pub horizontal: bool,
    pub left_side_vertical: bool,
    pub has_buttons: bool,
}

// cpp: layoutng_fragment_tree/fragment_tree.h:230-248
#[derive(Clone, Default, PartialEq)]
pub struct ScrollbarPaintData {
    pub corner_client_invalidation_reason: Option<foundation::PaintInvalidationReason>,
    pub corner_client_id: u64,
    pub corner_client_is_cacheable: bool,
    pub corner_client_is_just_created: Cell<bool>,
    pub horizontal: Option<ScrollbarPaintAxis>,
    pub vertical: Option<ScrollbarPaintAxis>,
    pub uses_overlay_scrollbars: bool,
    pub minimum_thumb_length: i32,
    pub corner_offset: LayoutOffset,
    pub corner_size: LayoutSize,
    pub track_color: PaintColor,
    pub thumb_color: PaintColor,
    pub button_color: PaintColor,
    pub corner_color: PaintColor,
}

/// Fully resolved native control state consumed by Paint. DOM and style
/// objects stay on the producer side of this fragment boundary.
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
    pub accent_color: PaintColor,
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
            accent_color: PaintColor {
                red: 0.0,
                green: 117.0 / 255.0,
                blue: 1.0,
                alpha: 1.0,
            },
        }
    }
}

/// SVG text geometry after textPath/rotate layout, relative to its fragment.
// cpp: layoutng_fragment_tree/fragment_tree.h:273-280
#[derive(Clone, PartialEq)]
pub struct SvgTextPaintData {
    pub visual_rect: FragmentPaintRect,
    pub glyph_origin: LayoutOffset,
    pub untransformed_size: LayoutSize,
    pub local_transform: TransformMatrix,
    pub scaling_factor: f64,
    pub has_transform: bool,
    pub hidden: bool,
}

impl Default for SvgTextPaintData {
    fn default() -> Self {
        Self {
            visual_rect: FragmentPaintRect::default(),
            glyph_origin: LayoutOffset::default(),
            untransformed_size: LayoutSize::default(),
            local_transform: TransformMatrix::default(),
            scaling_factor: 1.0,
            has_transform: false,
            hidden: false,
        }
    }
}

/// Native SVG viewport geometry, in SVG user coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SvgViewportGeometry {
    pub viewport: FragmentPaintRect,
    pub content_transform: TransformMatrix,
    pub clip: Option<FragmentPaintRect>,
    pub disabled: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollContainerPaintData {
    pub compositor_element_id: foundation::CompositorElementId,
    pub scrollable_axes: [bool; 2],
    pub background_needs_repaint_on_scroll: Option<bool>,
    pub prefers_non_composited_scrolling: Option<bool>,
    pub scroll_snap_type_is_none: Option<bool>,
    pub container_rect: FragmentPaintRect,
    pub minimum_scroll_offset: LayoutOffset,
    pub maximum_scroll_offset: LayoutOffset,
    pub user_scrollable_horizontal: bool,
    pub user_scrollable_vertical: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StickyAxisPaintData {
    pub min_inset: Option<f64>,
    pub max_inset: Option<f64>,
    pub scroll_container_relative_containing_block_offset: f64,
    pub scroll_container_relative_containing_block_size: f64,
    pub scroll_container_relative_sticky_box_offset: f64,
    pub scroll_container_relative_sticky_box_size: f64,
    pub constraining_offset: f64,
    pub constraining_size: f64,
    pub scroll_container_client_id: u64,
    pub nearest_sticky_box_client_id: u64,
    pub nearest_containing_block_client_id: u64,
    pub current_sticky_offset: f64,
    pub current_total_sticky_box_offset: f64,
    pub current_total_containing_block_offset: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StickyPositionPaintData {
    pub owner_client_id: u64,
    pub horizontal: Option<StickyAxisPaintData>,
    pub vertical: Option<StickyAxisPaintData>,
}

impl StickyPositionPaintData {
    /// Compare retained layout geometry while ignoring resolved scroll offsets.
    pub fn same_geometry(&self, other: &Self) -> bool {
        let (mut before, mut after) = (*self, *other);
        for data in [&mut before, &mut after] {
            for axis in [&mut data.horizontal, &mut data.vertical]
                .into_iter()
                .flatten()
            {
                if ![
                    axis.current_sticky_offset,
                    axis.current_total_sticky_box_offset,
                    axis.current_total_containing_block_offset,
                ]
                .into_iter()
                .all(f64::is_finite)
                {
                    return false;
                }
                axis.current_sticky_offset = 0.0;
                axis.current_total_sticky_box_offset = 0.0;
                axis.current_total_containing_block_offset = 0.0;
            }
        }
        before == after
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boundary_defaults_match_native_fragment_semantics() {
        assert_eq!(FragmentKind::default(), FragmentKind::kBox);
        assert!(FragmentBoxSides::default().HasAllSides());
        assert_eq!(FragmentPaintRect::default().size, LayoutSize::default());
        assert_eq!(PaintResources::default().ArtifactToDeviceScale(), 1.0);
        assert!(PaintResources::default().images.is_empty());
        assert_eq!(FragmentPaintData::<u8>::default().kind, FragmentKind::kBox);
        assert_eq!(*FragmentPaintData::<u8>::default().paint, 0);
        assert_eq!(TablePaintColumn::<u8>::default().span, 1);
        assert!(TablePaintData::<u8>::default().columns.is_empty());
        assert!(PaintGlyphRun::default().glyphs.is_empty());
        assert_eq!(
            PaintGlyphRun::default().writing_mode,
            LayoutPaintWritingMode::kHorizontalTb
        );
        assert!(MathMLPaintData::default().operator_glyph_runs.is_empty());
        assert!(TextCombinePaintData::default().emphasis.is_none());
        assert_eq!(CollapsedTableSection::default().start_row, 0);
        assert!(CollapsedTablePaintData::default().edges.is_empty());
        assert_eq!(ColumnRule::default().line_style, BorderLineStyle::kNone);
        assert_eq!(
            FieldsetPaintData::default().border_outsets,
            LayoutEdges::default()
        );
        assert!(!ScrollbarPaintAxis::default()
            .display_item_client_is_just_created
            .get());
        assert!(ScrollbarPaintData::default().horizontal.is_none());
        assert_eq!(
            FormControlPaintData::default().appearance,
            AppearanceValue::kNone
        );
        assert_eq!(SvgTextPaintData::default().scaling_factor, 1.0);
        assert_eq!(
            SvgViewportGeometry::default().content_transform.values[0],
            1.0
        );
    }

    #[test]
    fn sticky_geometry_ignores_only_resolved_scroll_offsets() {
        let mut before = StickyPositionPaintData::default();
        before.horizontal = Some(StickyAxisPaintData {
            constraining_size: 20.0,
            current_sticky_offset: 4.0,
            ..Default::default()
        });
        let mut after = before;
        after.horizontal.as_mut().unwrap().current_sticky_offset = 9.0;
        assert!(before.same_geometry(&after));
        after.horizontal.as_mut().unwrap().constraining_size = 21.0;
        assert!(!before.same_geometry(&after));
    }

    #[test]
    fn prepaint_invalidation_preserves_own_and_descendant_causes() {
        let descendant = PrePaintInvalidation {
            descendant_needs_paint_property_update: true,
            ..Default::default()
        };
        assert!(!descendant.own());
        assert!(descendant.descendants());

        let subtree = PrePaintInvalidation {
            subtree_should_check_for_paint_invalidation: true,
            ..Default::default()
        };
        assert!(subtree.own());
        assert!(subtree.descendants());
    }
}
