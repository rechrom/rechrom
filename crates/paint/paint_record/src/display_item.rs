use std::sync::Arc;

use layoutng_assembly::fragment_tree::{PaintGlyph, RasterEffectOutset};
use layoutng_assembly::internal::layout_input::{
    BorderLineStyle, FontSmoothing, FontVariation, Offset, PaintPathCommand, Size,
    TextDecorationStyle, TransformMatrix, WritingMode,
};
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::paint_input::{
    BackgroundRepeatRule, InterpolationQuality, PaintBlendMode, PaintCornerRadii,
    PaintFilterOperation, PaintImageTileRule, PaintShader, SvgStrokeLineCap, SvgStrokeLineJoin,
};
use paint_chunk::{PaintChunkBackgroundColorInfo, PaintHitTestOpaqueness};
use paint_model::PaintPhase;
use paint_model::PaintRect;

use crate::PaintOpType as DisplayItemType;

pub use paint_model::DisplayMaskLayer;

// cpp: paint/paint_engine.h:102-203
#[derive(Clone, PartialEq)]
pub struct DisplayItem {
    pub r#type: DisplayItemType,
    pub phase: PaintPhase,
    pub node_id: u64,
    pub fragment_instance_id: u64,
    pub rect: PaintRect,
    /// DrawTextBlobOp's retained blob bounds, independent of decoration ink.
    /// None keeps unsupported text geometry conservative.
    pub glyph_ink_rect: Option<PaintRect>,
    pub source_rect: PaintRect,
    pub inner_rect: PaintRect,
    pub tile_rect: PaintRect,
    pub color: Color,
    pub line_style: BorderLineStyle,
    pub decoration_style: TextDecorationStyle,
    pub stroke_width: f64,
    pub corner_radius: f64,
    pub inner_corner_radius: f64,
    pub corner_radii: PaintCornerRadii,
    pub inner_corner_radii: PaintCornerRadii,
    pub dash_intervals: Vec<f64>,
    pub path: Vec<PaintPathCommand>,
    pub even_odd: bool,
    pub inverse_winding: bool,
    pub svg_line_cap: SvgStrokeLineCap,
    pub svg_line_join: SvgStrokeLineJoin,
    pub dash_offset: f64,
    pub dash_fit_thickness: f64,
    pub miter_limit: f64,
    pub antialias: bool,
    pub non_scaling_stroke: bool,
    pub paint_shader: Option<PaintShader>,
    pub blend_mode: PaintBlendMode,
    pub svg_marker_resource_id: u64,
    pub filters: Vec<PaintFilterOperation>,
    /// Geometry needed only by the self-contained flat PaintRecord replay.
    /// Normal page compositing reads the same values from the Effect node.
    pub filter_reference_box: Option<PaintRect>,
    pub filter_reference_scale: f64,
    pub mask_layers: Vec<DisplayMaskLayer>,
    pub round_cap: bool,
    pub resource_id: u64,
    pub image_interpolation_quality: InterpolationQuality,
    /// Effective orientation for this image use after CSS image-orientation.
    /// Source rectangles are expressed in the oriented image coordinate space.
    pub image_orientation: image_resource::ImageOrientation,
    pub document_image_container_key: Option<image_resource::ContainerKey>,
    pub repeat_x: bool,
    pub repeat_y: bool,
    pub background_repeat_x: BackgroundRepeatRule,
    pub background_repeat_y: BackgroundRepeatRule,
    pub tile_rule_x: PaintImageTileRule,
    pub tile_rule_y: PaintImageTileRule,
    pub tile_scale: Offset,
    pub tile_phase: Offset,
    pub tile_spacing: Size,
    pub shadow_offset: Offset,
    pub blur_radius: f64,
    pub spread: f64,
    pub inset: bool,
    pub is_shadow: bool,
    pub shadow_has_opaque_background: bool,
    pub is_text_decoration: bool,
    pub stroke_glyphs: bool,
    pub skip_ink: bool,
    pub opacity: f32,
    pub transform: TransformMatrix,
    pub font_face_index: u32,
    pub font_variations: Vec<FontVariation>,
    pub font_size: f64,
    pub baseline: f64,
    pub text_line_top: f64,
    pub text_line_height: f64,
    pub text_blob_origin: Offset,
    pub horizontal: bool,
    pub at_start: bool,
    pub rtl: bool,
    pub synthetic_bold: bool,
    pub synthetic_italic: bool,
    pub font_smoothing: FontSmoothing,
    pub writing_mode: WritingMode,
    /// SkTextBlob-like immutable glyph payload. Cached DrawingDisplayItems
    /// share it across paint artifacts instead of cloning every glyph during
    /// PaintController commit.
    pub glyphs: Arc<Vec<PaintGlyph>>,
}

impl Default for DisplayItem {
    fn default() -> Self {
        Self {
            r#type: DisplayItemType::kDrawRect,
            phase: PaintPhase::kForeground,
            node_id: 0,
            fragment_instance_id: 0,
            rect: PaintRect::default(),
            glyph_ink_rect: None,
            source_rect: PaintRect::default(),
            inner_rect: PaintRect::default(),
            tile_rect: PaintRect::default(),
            color: Color::default(),
            line_style: BorderLineStyle::kNone,
            decoration_style: TextDecorationStyle::kSolid,
            stroke_width: 0.0,
            corner_radius: 0.0,
            inner_corner_radius: 0.0,
            corner_radii: PaintCornerRadii::default(),
            inner_corner_radii: PaintCornerRadii::default(),
            dash_intervals: Vec::new(),
            path: Vec::new(),
            even_odd: false,
            inverse_winding: false,
            svg_line_cap: SvgStrokeLineCap::kButt,
            svg_line_join: SvgStrokeLineJoin::kMiter,
            dash_offset: 0.0,
            dash_fit_thickness: 0.0,
            miter_limit: 4.0,
            antialias: true,
            non_scaling_stroke: false,
            paint_shader: None,
            blend_mode: PaintBlendMode::kNormal,
            svg_marker_resource_id: 0,
            filters: Vec::new(),
            filter_reference_box: None,
            filter_reference_scale: 1.0,
            mask_layers: Vec::new(),
            round_cap: false,
            resource_id: 0,
            image_interpolation_quality: InterpolationQuality::kInterpolationMedium,
            image_orientation: Default::default(),
            document_image_container_key: None,
            repeat_x: false,
            repeat_y: false,
            background_repeat_x: BackgroundRepeatRule::kNoRepeat,
            background_repeat_y: BackgroundRepeatRule::kNoRepeat,
            tile_rule_x: PaintImageTileRule::kStretch,
            tile_rule_y: PaintImageTileRule::kStretch,
            tile_scale: Offset { x: 1.0, y: 1.0 },
            tile_phase: Offset::default(),
            tile_spacing: Size::default(),
            shadow_offset: Offset::default(),
            blur_radius: 0.0,
            spread: 0.0,
            inset: false,
            is_shadow: false,
            shadow_has_opaque_background: false,
            is_text_decoration: false,
            stroke_glyphs: false,
            skip_ink: false,
            opacity: 1.0,
            transform: TransformMatrix::default(),
            font_face_index: 0,
            font_variations: Vec::new(),
            font_size: 16.0,
            baseline: 0.0,
            text_line_top: 0.0,
            text_line_height: 0.0,
            text_blob_origin: Offset::default(),
            horizontal: true,
            at_start: false,
            rtl: false,
            synthetic_bold: false,
            synthetic_italic: false,
            font_smoothing: FontSmoothing::kAuto,
            writing_mode: WritingMode::kHorizontalTb,
            glyphs: Arc::new(Vec::new()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordedDisplayItemKind {
    Drawing,
    Scrollbar,
}

/// Source-owned inputs retained by ScrollbarDisplayItem until PAC chooses
/// composited or non-composited painting. The cc layer is created only at the
/// compositor boundary; Paint retains geometry and identity, not a cc owner.
#[derive(Clone, Debug, PartialEq)]
pub struct ScrollbarDisplayItemData {
    pub element_id: foundation::CompositorElementId,
    pub track_rect: PaintRect,
    pub thumb_rect: PaintRect,
    pub thumb_color: Color,
    pub horizontal: bool,
    pub overlay: bool,
    pub left_side_vertical: bool,
    pub minimum_thumb_length: i32,
    pub needs_update_display: bool,
    pub hit_test_opaqueness: PaintHitTestOpaqueness,
}

/// A semantic item containing a PaintRecord range in the flat replay ABI.
/// Save/restore and multiple draw ops inside a recorder count as one item.
#[derive(Clone, Debug, PartialEq)]
pub struct RecordedDisplayItem {
    // Source: display_item.h:260-265; paint_controller.cc:531-545.
    pub is_cacheable: bool,
    pub is_tombstone: bool,
    pub paint_invalidation_reason: Option<foundation::PaintInvalidationReason>,
    /// Immutable content versions of image resources sampled by this record.
    ///
    /// Blink normally carries an image client's invalidation through the next
    /// PaintController transaction. Rechrom also permits an older immutable
    /// PaintArtifact to be presented again, so a one-shot dirty bit is not a
    /// sufficient proof. Keeping the exact `(resource id, revision)` set on
    /// the record lets RasterInvalidator compare any two retained artifacts in
    /// either direction without retaining Paint or Page state.
    pub image_content_revisions: Vec<(image_resource::ImageId, u64)>,
    pub scrollbar_needs_update_display: Option<bool>,
    pub scrollbar: Option<ScrollbarDisplayItemData>,
    pub kind: RecordedDisplayItemKind,
    pub id: paint_model::display_item_id::DisplayItemId,
    /// The client visual rect supplied when the PaintRecord was created.
    /// ObjectPaintInvalidator compares this geometry before PaintController
    /// moves a cached DrawingDisplayItem. Keep the producer value separate
    /// from `visual_rect`, which may be tightened to the recorded operations.
    pub client_visual_rect: PaintRect,
    pub visual_rect: PaintRect,
    pub visual_rect_is_accurate: bool,
    pub draws_content: bool,
    pub raster_effect_outset: RasterEffectOutset,
    /// DrawingDisplayItem's immutable PaintRecord analysis. PaintController
    /// moves these values with a cached item instead of rescanning its record.
    pub record_rect_known_to_be_opaque: PaintRect,
    pub record_background: PaintChunkBackgroundColorInfo,
    pub record_has_text: bool,
    pub record_begin: usize,
    pub record_end: usize,
    pub scroll_translation:
        Option<Arc<paint_model::paint_property_tree::TransformPaintPropertyNode>>,
}

pub use layoutng_assembly::caret::CaretPaintState as CaretPosition;
pub use layoutng_assembly::editing_state::EditingPaintState;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CaretGeometry {
    pub node_id: u64,
    pub rect: PaintRect,
    pub visible: bool,
}

/// Drawing display items, each selecting its recorded drawing operations.
pub type DisplayItemList = Vec<RecordedDisplayItem>;
