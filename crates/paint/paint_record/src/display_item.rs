use std::sync::Arc;

use foundation::ETextDecorationSkipInk;
use layoutng_assembly::fragment_tree::{PaintGlyph, PaintGlyphRun, RasterEffectOutset};
use layoutng_assembly::internal::layout_input::{
    BorderLineStyle, FontSmoothing, FontVariation, Offset, PaintPathCommand, Size,
    TextDecorationStyle, TransformMatrix, WritingMode,
};
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::paint_input::{
    BackgroundRepeatRule, InterpolationQuality, PaintBlendMode, PaintCornerRadii,
    PaintImageTileRule, PaintShader, SvgStrokeLineCap, SvgStrokeLineJoin,
};
use paint_chunk::PaintChunkBackgroundColorInfo;
use paint_model::property_filter_operations::PropertyFilterOperations;
use paint_model::PaintPhase;
use paint_model::PaintRect;

use crate::PaintOpType as DisplayItemType;

pub use crate::scrollbar_display_item::ScrollbarDisplayItemData;
pub use paint_model::DisplayMaskLayer;

/// The optional SVG PaintFlags pass supplied to DecorationLinePainter.
/// HTML is the null-flags path; SVG fill and stroke must remain distinct
/// because the latter strokes the decoration's already-expanded outline.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DecorationPaintPass {
    #[default]
    Html,
    SvgFill,
    SvgStroke,
}

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
    /// DecorationGeometry::Thickness(), kept separate from an SVG stroke
    /// pass's own PaintFlags stroke width.
    pub decoration_thickness: f64,
    pub decoration_dash_intervals: Vec<f64>,
    pub decoration_dash_offset: f64,
    pub decoration_round_cap: bool,
    /// DecorationLinePainter's resolved wave. A zero wavelength denotes a
    /// legacy/default wave and is resolved by the compositor lowering boundary.
    pub wavy_wavelength: f64,
    pub wavy_control_point_distance: f64,
    pub wavy_phase: f64,
    pub wavy_offset: f64,
    pub decoration_paint_pass: DecorationPaintPass,
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
    /// GraphicsContext::FillRect with SkBlendMode::kSrc. CSS blend modes stay
    /// independent because source replacement is a canvas-clear policy.
    pub source_overwrite: bool,
    pub svg_marker_resource_id: u64,
    /// cc::CustomDataOp payload used by nested canvas snapshot substitution.
    pub custom_data_id: u32,
    /// Canonical FilterEffectBuilder output owned by PrePaint. Flat records
    /// replay this same value; Raster must not rebuild it from authored style.
    pub canonical_filters: PropertyFilterOperations,
    /// ScopedTextShadowPainter's parallel shadow-only filter branches. The
    /// decorated source geometry is drawn once inside this layer; Raster
    /// merges these branches before compositing the shadow result.
    pub text_shadow_filters: Vec<layoutng_assembly::internal::layout_input::PaintShadow>,
    pub mask_layers: Vec<DisplayMaskLayer>,
    pub round_cap: bool,
    pub resource_id: u64,
    pub image_interpolation_quality: InterpolationQuality,
    pub dynamic_range_limit: foundation::DynamicRangeLimit,
    /// PaintImage::DecodingMode selected for this concrete image use.
    pub image_decoding_mode: image_resource::ImageDecodingMode,
    /// Effective orientation for this image use after CSS image-orientation.
    /// Source rectangles are expressed in the oriented image coordinate space.
    pub image_orientation: image_resource::ImageOrientation,
    /// PaintFlags::useDarkModeForImage for the raster-side image decoder.
    pub use_dark_mode_for_image: bool,
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
    pub skip_ink: ETextDecorationSkipInk,
    /// Bounds and shaping inputs consumed by TextPainter::ClipDecorationLine.
    /// Runs remain shared in Paint and are projected into the backend-neutral
    /// raster command only at the package boundary.
    pub decoration_ink_bounds: Option<PaintRect>,
    pub decoration_ink_baseline: f64,
    pub decoration_text_origin: Offset,
    pub decoration_text_fit_scale: f64,
    pub decoration_ink_runs: Arc<Vec<PaintGlyphRun>>,
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
            decoration_thickness: 0.0,
            decoration_dash_intervals: Vec::new(),
            decoration_dash_offset: 0.0,
            decoration_round_cap: false,
            wavy_wavelength: 0.0,
            wavy_control_point_distance: 0.0,
            wavy_phase: 0.0,
            wavy_offset: 0.0,
            decoration_paint_pass: DecorationPaintPass::Html,
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
            source_overwrite: false,
            svg_marker_resource_id: 0,
            custom_data_id: 0,
            canonical_filters: Default::default(),
            text_shadow_filters: Vec::new(),
            mask_layers: Vec::new(),
            round_cap: false,
            resource_id: 0,
            image_interpolation_quality: InterpolationQuality::kInterpolationMedium,
            dynamic_range_limit: foundation::DynamicRangeLimit::default(),
            image_decoding_mode: Default::default(),
            image_orientation: Default::default(),
            use_dark_mode_for_image: false,
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
            skip_ink: ETextDecorationSkipInk::kNone,
            decoration_ink_bounds: None,
            decoration_ink_baseline: 0.0,
            decoration_text_origin: Offset::default(),
            decoration_text_fit_scale: 1.0,
            decoration_ink_runs: Arc::new(Vec::new()),
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
    ForeignLayer,
}

/// A semantic item containing a PaintRecord range in the flat replay ABI.
/// Save/restore and multiple draw ops inside a recorder count as one item.
#[derive(Clone, Debug, PartialEq)]
pub struct RecordedDisplayItem {
    // Source: display_item.h:260-265; paint_controller.cc:531-545.
    pub is_cacheable: bool,
    pub is_tombstone: bool,
    pub paint_invalidation_reason: Option<foundation::PaintInvalidationReason>,
    pub scrollbar: Option<ScrollbarDisplayItemData>,
    pub foreign_layer: Option<crate::ForeignLayerDisplayItemData>,
    pub kind: RecordedDisplayItemKind,
    pub id: paint_model::display_item_id::DisplayItemId,
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

impl RecordedDisplayItem {
    /// Source: `display_item.cc:40-66`.
    ///
    /// The resident artifacts are explicit because Rechrom stores a drawing's
    /// PaintRecord in the artifact's flat PaintOp backing instead of in the
    /// heterogeneous DisplayItem allocation itself.
    pub fn EqualsForUnderInvalidation(
        &self,
        artifact: &crate::PaintRecording,
        other: &Self,
        other_artifact: &crate::PaintRecording,
    ) -> bool {
        assert!(!self.IsTombstone());
        if self.id != other.id
            || self.kind != other.kind
            || self.raster_effect_outset != other.raster_effect_outset
            || self.draws_content != other.draws_content
        {
            return false;
        }

        if self.visual_rect != other.visual_rect
            // Change of empty visual rect doesn't matter.
            && self.visual_rect.is_empty()
            && other.visual_rect.is_empty()
            // Visual rect of a DrawingDisplayItem not drawing content doesn't matter.
            && (!self.IsDrawing() || self.draws_content)
        {
            return false;
        }

        match self.kind {
            RecordedDisplayItemKind::Drawing => {
                self.EqualsDrawingRecordFastPath(artifact, other, other_artifact)
            }
            RecordedDisplayItemKind::ForeignLayer => {
                self.EqualsForeignLayerForUnderInvalidation(other)
            }
            RecordedDisplayItemKind::Scrollbar => self.EqualsScrollbarForUnderInvalidation(other),
        }
    }

    // Source: display_item.h:206.
    pub fn GetId(&self) -> paint_model::display_item_id::DisplayItemId {
        self.id
    }

    // Source: display_item.h:208-211.
    pub fn ClientId(&self) -> u64 {
        self.id.client_id
    }

    // Source: display_item.h:213-225.
    pub fn VisualRect(&self) -> PaintRect {
        self.visual_rect
    }

    pub fn GetRasterEffectOutset(&self) -> RasterEffectOutset {
        self.raster_effect_outset
    }

    pub fn GetType(&self) -> paint_model::display_item_id::DisplayItemIdType {
        self.id.r#type
    }

    pub fn Fragment(&self) -> u32 {
        self.id.fragment
    }

    // Source: display_item.h:226.
    pub fn SetFragment(&mut self, fragment: u32) {
        self.id.fragment = fragment;
    }

    // Source: display_item.h:246-258.
    pub fn IsDrawing(&self) -> bool {
        self.GetType().IsDrawingType()
    }

    pub fn IsForeignLayer(&self) -> bool {
        self.GetType().IsForeignLayerType()
    }

    pub fn IsScrollbar(&self) -> bool {
        matches!(
            self.GetType(),
            paint_model::display_item_id::DisplayItemIdType::kScrollbarHorizontal
                | paint_model::display_item_id::DisplayItemIdType::kScrollbarVertical
        )
    }

    // Source: display_item.h:260-268.
    pub fn GetPaintInvalidationReason(&self) -> Option<foundation::PaintInvalidationReason> {
        self.paint_invalidation_reason
    }

    pub fn SetPaintInvalidationReason(&mut self, reason: foundation::PaintInvalidationReason) {
        self.paint_invalidation_reason = Some(reason);
        self.is_cacheable = reason != foundation::PaintInvalidationReason::kUncacheable;
    }

    pub fn IsCacheable(&self) -> bool {
        self.paint_invalidation_reason
            .map_or(self.is_cacheable, |reason| {
                reason != foundation::PaintInvalidationReason::kUncacheable
            })
    }

    // Source: display_item.h:272-276.
    pub fn IsTombstone(&self) -> bool {
        self.is_tombstone
    }

    // Source: display_item.h:317-325. A single-item move keeps identity and
    // invalidation fields available to diagnostics and raster invalidation.
    pub fn CreateTombstone(&mut self) {
        // MoveItem's memcpy transfers subclass ownership without incrementing
        // scoped_refptr counts, and Destruct() intentionally skips the old
        // tombstone. Rust must explicitly release the old owner slot after the
        // destination clone has retained it.
        self.foreign_layer = None;
        self.draws_content = false;
        self.is_tombstone = true;
    }

    // Source: display_item_list.h:225-231. A moved subsequence is never read
    // for raster invalidation, so Blink zeroes every old slot rather than
    // preserving the single-item tombstone fields.
    pub fn CreateSubsequenceTombstone(&mut self) {
        self.is_cacheable = true;
        self.is_tombstone = true;
        self.paint_invalidation_reason = Some(foundation::PaintInvalidationReason::kNone);
        self.scrollbar = None;
        self.foreign_layer = None;
        self.kind = RecordedDisplayItemKind::Drawing;
        self.id = Default::default();
        self.visual_rect = Default::default();
        self.visual_rect_is_accurate = false;
        self.draws_content = false;
        self.raster_effect_outset = RasterEffectOutset::kNone;
        self.record_rect_known_to_be_opaque = Default::default();
        self.record_background = Default::default();
        self.record_has_text = false;
        self.record_begin = 0;
        self.record_end = 0;
        self.scroll_translation = None;
    }

    // Source: display_item.h:278-286; display_item.cc:203-240.
    pub fn IsSubsequenceTombstone(&self) -> bool {
        self.is_tombstone && self.id.client_id == 0
    }

    pub fn IdAsString(&self, paint_artifact: &crate::PaintRecording) -> String {
        if self.IsSubsequenceTombstone() {
            return "SUBSEQUENCE TOMBSTONE".into();
        }
        if self.IsTombstone() {
            return format!("TOMBSTONE {}", paint_artifact.IdAsString(&self.id));
        }
        paint_artifact.IdAsString(&self.id)
    }

    pub fn PropertiesAsJSON(&self, paint_artifact: &crate::PaintRecording) -> serde_json::Value {
        let mut properties = serde_json::Map::new();
        properties.insert(
            "id".into(),
            serde_json::Value::String(self.IdAsString(paint_artifact)),
        );
        if self.IsSubsequenceTombstone() {
            return serde_json::Value::Object(properties);
        }

        properties.insert(
            "invalidation".into(),
            serde_json::Value::String(
                foundation::PaintInvalidationReasonToString(
                    self.paint_invalidation_reason
                        .unwrap_or(foundation::PaintInvalidationReason::kNone),
                )
                .into(),
            ),
        );
        properties.insert(
            "visualRect".into(),
            serde_json::Value::String(format!(
                "{},{} {}x{}",
                self.visual_rect.x,
                self.visual_rect.y,
                self.visual_rect.width,
                self.visual_rect.height
            )),
        );
        match self.raster_effect_outset {
            RasterEffectOutset::kNone => {}
            RasterEffectOutset::kHalfPixel => {
                properties.insert("outset".into(), serde_json::json!(0.5));
            }
            RasterEffectOutset::kWholePixel => {
                properties.insert("outset".into(), serde_json::json!(1));
            }
        }

        if self.IsTombstone() {
            return serde_json::Value::Object(properties);
        }
        if self.kind == RecordedDisplayItemKind::Scrollbar {
            let scroll_translation = self
                .scroll_translation
                .as_ref()
                .map_or_else(|| "0x0".into(), |node| format!("{:p}", Arc::as_ptr(node)));
            properties.insert(
                "scrollTranslation".into(),
                serde_json::Value::String(scroll_translation),
            );
        } else if self.kind == RecordedDisplayItemKind::ForeignLayer {
            properties.insert(
                "layer".into(),
                serde_json::json!(self.GetForeignLayer().LayerId()),
            );
        }
        serde_json::Value::Object(properties)
    }

    pub fn AsDebugString(&self, paint_artifact: &crate::PaintRecording) -> String {
        serde_json::to_string_pretty(&self.PropertiesAsJSON(paint_artifact))
            .expect("DisplayItem debug properties contain only finite source values")
    }

    pub fn DrawsContent(&self) -> bool {
        self.draws_content
    }
}

// Source: display_item.cc:271-273.
impl std::fmt::Display for RecordedDisplayItem {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(output, "{{\"id\": {}}}", self.GetId())
    }
}

#[cfg(test)]
mod recorded_display_item_debug_tests {
    use super::*;
    use paint_model::display_item_id::{DisplayItemId, DisplayItemIdType};

    fn item() -> RecordedDisplayItem {
        RecordedDisplayItem {
            is_cacheable: true,
            is_tombstone: false,
            paint_invalidation_reason: Some(foundation::PaintInvalidationReason::kLayout),
            scrollbar: None,
            foreign_layer: None,
            kind: RecordedDisplayItemKind::Drawing,
            id: DisplayItemId {
                client_id: 0x2a,
                r#type: DisplayItemIdType::kCaret,
                fragment: 3,
            },
            visual_rect: PaintRect {
                x: 1.0,
                y: 2.0,
                width: 30.0,
                height: 40.0,
            },
            visual_rect_is_accurate: true,
            draws_content: true,
            raster_effect_outset: RasterEffectOutset::kHalfPixel,
            record_rect_known_to_be_opaque: PaintRect::default(),
            record_background: PaintChunkBackgroundColorInfo::default(),
            record_has_text: false,
            record_begin: 0,
            record_end: 0,
            scroll_translation: None,
        }
    }

    #[test]
    fn properties_use_artifact_identity_reason_rect_and_outset() {
        let item = item();
        let mut artifact = crate::PaintRecording::default();
        artifact.RecordDebugInfo(0x2a, Arc::<str>::from("caret-client"), 7);
        let properties = item.PropertiesAsJSON(&artifact);
        assert_eq!(properties["id"], "0x2a:caret-client:Caret:3");
        assert_eq!(properties["invalidation"], "geometry");
        assert_eq!(properties["visualRect"], "1,2 30x40");
        assert_eq!(properties["outset"], 0.5);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&item.AsDebugString(&artifact)).unwrap(),
            properties
        );
    }

    #[test]
    fn tombstone_strings_keep_source_distinction() {
        let artifact = crate::PaintRecording::default();
        let mut moved = item();
        moved.is_tombstone = true;
        assert_eq!(moved.IdAsString(&artifact), "TOMBSTONE 0x2a:Caret:3");
        moved.id.client_id = 0;
        assert_eq!(moved.IdAsString(&artifact), "SUBSEQUENCE TOMBSTONE");
        assert_eq!(
            moved.PropertiesAsJSON(&artifact).as_object().unwrap().len(),
            1
        );
    }
}

pub use layoutng_assembly::caret::CaretPaintState as CaretPosition;
pub use layoutng_assembly::editing_state::EditingPaintState;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CaretGeometry {
    pub node_id: u64,
    pub rect: PaintRect,
    pub visible: bool,
}
