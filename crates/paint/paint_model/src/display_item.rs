//! The recorded paint operation of a DrawingDisplayItem's PaintRecord: the
//! backend-neutral op the Paint recorder emits and cc/paint replays. Blink
//! keeps the display-item vocabulary in platform/graphics/paint (this
//! package); the recorder (core/paint) and cc/paint both name it.
//! Source: third_party/blink/renderer/platform/graphics/paint/
//!         drawing_display_item.h (a PaintRecord of cc::PaintOps).

use std::sync::Arc;

use foundation::ETextDecorationSkipInk;
use paint_input_types::layout_input::{
    BorderLineStyle, FontSmoothing, FontVariation, Offset, PaintPathCommand, Size,
    TextDecorationStyle, TransformMatrix, WritingMode,
};
use paint_input_types::layout_input_types::Color;
use paint_input_types::paint_input::{
    BackgroundRepeatRule, InterpolationQuality, PaintBlendMode, PaintCornerRadii,
    PaintImageTileRule, PaintShader, SvgStrokeLineCap, SvgStrokeLineJoin,
};
use paint_input_types::{PaintGlyph, PaintGlyphRun};

use crate::paint_op::PaintOpType as DisplayItemType;
use crate::property_filter_operations::PropertyFilterOperations;
use crate::{DisplayMaskLayer, PaintPhase, PaintRect};

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
    pub text_shadow_filters: Vec<paint_input_types::layout_input::PaintShadow>,
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
