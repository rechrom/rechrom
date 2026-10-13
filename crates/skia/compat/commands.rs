//! Browser-independent raster commands and borrowed resource views.
//! This is the explicitly local Rust command interface, not a translation of Chromium's
//! DisplayItemList or an assertion of C++ SkCanvas API/ABI equivalence.
#![allow(non_camel_case_types)]
use std::sync::Arc;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Offset {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransformMatrix {
    pub values: [f64; 16],
}
impl Default for TransformMatrix {
    fn default() -> Self {
        Self {
            values: [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        }
    }
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintPathVerb {
    kMoveTo,
    kLineTo,
    kQuadraticTo,
    kConicTo,
    kCubicTo,
    kClose,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaintPathCommand {
    pub verb: PaintPathVerb,
    pub control1: Offset,
    pub control2: Offset,
    pub point: Offset,
    pub conic_weight: f64,
}
impl Default for PaintPathCommand {
    fn default() -> Self {
        Self {
            verb: PaintPathVerb::kMoveTo,
            control1: Offset::default(),
            control2: Offset::default(),
            point: Offset::default(),
            conic_weight: 1.0,
        }
    }
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BorderLineStyle {
    kNone,
    kHidden,
    kSolid,
    kDashed,
    kDotted,
    kDouble,
    kGroove,
    kRidge,
    kInset,
    kOutset,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextDecorationStyle {
    kSolid,
    kDouble,
    kDotted,
    kDashed,
    kWavy,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontSmoothing {
    kAuto,
    kNone,
    kAntialiased,
    kSubpixelAntialiased,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FontVariation {
    pub tag: u32,
    pub value: f32,
}

#[derive(Clone, Default, PartialEq)]
pub struct PaintGlyph {
    pub id: u32,
    pub character_index: u32,
    pub canvas_rotation: u8,
    pub offset: Offset,
    pub advance: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PaintCornerRadius {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PaintCornerRadii {
    pub top_left: PaintCornerRadius,
    pub top_right: PaintCornerRadius,
    pub bottom_right: PaintCornerRadius,
    pub bottom_left: PaintCornerRadius,
}
impl PaintCornerRadii {
    #[allow(non_snake_case)]
    pub fn HasRadius(&self) -> bool {
        (self.top_left.x > 0.0 && self.top_left.y > 0.0)
            || (self.top_right.x > 0.0 && self.top_right.y > 0.0)
            || (self.bottom_right.x > 0.0 && self.bottom_right.y > 0.0)
            || (self.bottom_left.x > 0.0 && self.bottom_left.y > 0.0)
    }
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintBlendMode {
    kNormal,
    kMultiply,
    kScreen,
    kOverlay,
    kDarken,
    kLighten,
    kColorDodge,
    kColorBurn,
    kHardLight,
    kSoftLight,
    kDifference,
    kExclusion,
    kHue,
    kSaturation,
    kColor,
    kLuminosity,
    kPlusLighter,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintFilterType {
    kReference,
    kBlur,
    kBrightness,
    kContrast,
    kGrayscale,
    kHueRotate,
    kInvert,
    kOpacity,
    kSaturate,
    kSepia,
    kDropShadow,
    kBoxReflect,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaintReflectionDirection {
    kAbove,
    #[default]
    kBelow,
    kLeft,
    kRight,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaintFilterOperation {
    pub r#type: PaintFilterType,
    pub amount: f64,
    pub offset: Offset,
    pub blur_radius: f64,
    pub color: Color,
    pub reflection_direction: PaintReflectionDirection,
    pub reflection_offset: f64,
    pub reflection_has_mask: bool,
    pub reflection_mask: Option<crate::compat::layer_filters::RenderReflectionMask>,
}
impl Default for PaintFilterOperation {
    fn default() -> Self {
        Self {
            r#type: PaintFilterType::kBlur,
            amount: 0.0,
            offset: Offset::default(),
            blur_radius: 0.0,
            color: Color::default(),
            reflection_direction: PaintReflectionDirection::kBelow,
            reflection_offset: 0.0,
            reflection_has_mask: false,
            reflection_mask: None,
        }
    }
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SvgStrokeLineCap {
    kButt,
    kRound,
    kSquare,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SvgStrokeLineJoin {
    kMiter,
    kRound,
    kBevel,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintShaderKind {
    kLinearGradient,
    kRadialGradient,
    kConicGradient,
    kPattern,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintSpreadMethod {
    kPad,
    kReflect,
    kRepeat,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintRadialShape {
    kCircle,
    kEllipse,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintRadialExtent {
    kExplicit,
    kClosestSide,
    kClosestCorner,
    kFarthestSide,
    kFarthestCorner,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaintColorInterpolationSpace {
    #[default]
    kNone,
    kSRGB,
    kSRGBLegacy,
    kSRGBLinear,
    kDisplayP3,
    kDisplayP3Linear,
    kHSL,
    kHWB,
    kA98RGB,
    kProPhotoRGB,
    kRec2020,
    kRec2100Linear,
    kLab,
    kLch,
    kOklab,
    kOklch,
    kXYZD50,
    kXYZD65,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaintHueInterpolationMethod {
    #[default]
    kShorter,
    kLonger,
    kIncreasing,
    kDecreasing,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PaintColorStop {
    pub offset: f64,
    pub color: Color,
    pub source_color_space: PaintColorInterpolationSpace,
    pub source_components: [f32; 4],
    pub missing_components: u8,
    pub offset_length: f64,
    pub is_hint: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SvgPatternGeometry {
    kPath,
    kRectangle,
    kEllipse,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SvgPatternPrimitive {
    pub geometry: SvgPatternGeometry,
    pub bounds_offset: Offset,
    pub bounds_width: f64,
    pub bounds_height: f64,
    pub rectangle_radius_x: f64,
    pub rectangle_radius_y: f64,
    pub path: Vec<PaintPathCommand>,
    pub fill: Option<Color>,
    pub stroke: Option<Color>,
    pub fill_server: Option<PaintShader>,
    pub stroke_server: Option<PaintShader>,
    pub stroke_width: f64,
    pub stroke_dash_array: Vec<f64>,
    pub stroke_dash_offset: f64,
    pub stroke_line_cap: SvgStrokeLineCap,
    pub stroke_line_join: SvgStrokeLineJoin,
    pub stroke_miter_limit: f64,
    pub fill_even_odd: bool,
    pub non_scaling_stroke: bool,
    pub antialias: bool,
    pub transform: TransformMatrix,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SvgPatternImage {
    pub resource_id: u64,
    pub image_interpolation_quality: ImageInterpolationQuality,
    pub destination: PaintRect,
    pub source: PaintRect,
    pub transform: TransformMatrix,
    /// Raster supplies exact document-image pixels from the command's
    /// ContainerKey. Bitmap images keep this empty and use ResourceContext.
    pub raster_width: u32,
    pub raster_height: u32,
    pub rgba8: Arc<Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SvgPatternMask {
    pub commands: Vec<SvgPatternCommand>,
    pub content_transform: TransformMatrix,
    pub clip_rect: PaintRect,
    pub mode: PaintMaskMode,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SvgPatternCommand {
    SaveLayerOpacity(f32),
    SaveLayerFilter(Vec<PaintFilterOperation>),
    SaveLayerReferenceFilter(Arc<crate::compat::layer_filters::RenderSvgFilterGraph>),
    SaveLayerBlend(PaintBlendMode),
    SaveLayerMask(Arc<SvgPatternMask>),
    Restore,
    DrawShape(SvgPatternPrimitive),
    DrawGlyph(SvgPatternPrimitive),
    DrawImage(SvgPatternImage),
}

#[derive(Clone, Debug, PartialEq)]
pub struct SvgPatternData {
    pub commands: Vec<SvgPatternCommand>,
    pub content_transform: TransformMatrix,
    pub opacity: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaintShader {
    pub kind: PaintShaderKind,
    pub spread: PaintSpreadMethod,
    pub stops: Vec<PaintColorStop>,
    pub start: Offset,
    pub end: Offset,
    pub linear_angle: Option<f64>,
    pub linear_corner: Option<[i8; 2]>,
    pub linear_start_offset: f64,
    pub linear_end_offset: f64,
    pub center: Offset,
    pub center_offset: Offset,
    pub focal: Offset,
    pub radius: f64,
    pub radius_y: Option<f64>,
    pub radius_offset: f64,
    pub radius_y_offset: f64,
    pub radial_shape: PaintRadialShape,
    pub radial_extent: PaintRadialExtent,
    pub radial_start_offset: f64,
    pub radial_end_offset: f64,
    pub focal_radius: f64,
    pub start_angle: f64,
    pub end_angle: f64,
    pub rotation_angle: f64,
    pub resource_id: u64,
    pub tile_offset: Offset,
    pub tile_width: f64,
    pub tile_height: f64,
    pub unit_coordinates: bool,
    pub object_bounding_box_coordinates: bool,
    pub interpolate_premultiplied: bool,
    pub color_interpolation_space: PaintColorInterpolationSpace,
    pub hue_interpolation_method: PaintHueInterpolationMethod,
    pub transform: TransformMatrix,
    pub pattern: Option<std::sync::Arc<SvgPatternData>>,
}
impl Default for PaintShader {
    fn default() -> Self {
        Self {
            kind: PaintShaderKind::kLinearGradient,
            spread: PaintSpreadMethod::kPad,
            stops: Vec::new(),
            start: Offset::default(),
            end: Offset::default(),
            linear_angle: None,
            linear_corner: None,
            linear_start_offset: 0.0,
            linear_end_offset: 1.0,
            center: Offset::default(),
            center_offset: Offset::default(),
            focal: Offset::default(),
            radius: 0.0,
            radius_y: None,
            radius_offset: 0.0,
            radius_y_offset: 0.0,
            radial_shape: PaintRadialShape::kEllipse,
            radial_extent: PaintRadialExtent::kExplicit,
            radial_start_offset: 0.0,
            radial_end_offset: 1.0,
            focal_radius: 0.0,
            start_angle: 0.0,
            end_angle: 360.0,
            rotation_angle: 0.0,
            resource_id: 0,
            tile_offset: Offset::default(),
            tile_width: 0.0,
            tile_height: 0.0,
            unit_coordinates: false,
            object_bounding_box_coordinates: false,
            interpolate_premultiplied: true,
            color_interpolation_space: PaintColorInterpolationSpace::kNone,
            hue_interpolation_method: PaintHueInterpolationMethod::kShorter,
            transform: TransformMatrix::default(),
            pattern: None,
        }
    }
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackgroundRepeatRule {
    kRepeat,
    kNoRepeat,
    kRound,
    kSpace,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintMaskMode {
    kAlpha,
    kLuminance,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintMaskComposite {
    kAdd,
    kSubtract,
    kIntersect,
    kExclude,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PaintRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandKind {
    kSave,
    kRestore,
    kConcat,
    kClipRect,
    kClipRoundedRect,
    kClipOutRoundedRect,
    kClipPath,
    kClipOutRect,
    kSaveLayer,
    kSaveLayerAlpha,
    kSaveLayerBlend,
    kSaveLayerFilter,
    kSaveLayerDstIn,
    kBeginMask,
    kEndMask,
    kDrawRect,
    kDrawMask,
    kDrawRoundedRect,
    kDrawDoubleRoundedRect,
    kDrawEllipse,
    kStrokeEllipse,
    kDrawPath,
    kStrokePath,
    kDrawGradientRect,
    kDrawTiledGradient,
    kStrokeRect,
    kStrokeLine,
    kStrokeWavyLine,
    kDrawGlyphRun,
    kDrawImageRect,
    kDrawTiledImage,
    kDrawBoxShadow,
    kDrawScrollbarTrack,
    kDrawScrollbarThumb,
    kDrawScrollbarButton,
    kDrawScrollbarCorner,
}

/// cc::PaintFlags::FilterQuality carried by immutable raster commands.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ImageInterpolationQuality {
    kNone = 0,
    kLow = 1,
    #[default]
    kMedium = 2,
}

/// Effective EXIF orientation for one image draw. The resource bytes stay in
/// encoded storage order; source rectangles use this oriented coordinate space.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ImageOrientation {
    #[default]
    kDefault = 1,
    kFlipHorizontal = 2,
    kRotate180 = 3,
    kFlipVertical = 4,
    kTranspose = 5,
    kRotate90 = 6,
    kTransverse = 7,
    kRotate270 = 8,
}

impl ImageOrientation {
    pub fn uses_width_as_height(self) -> bool {
        matches!(
            self,
            Self::kTranspose | Self::kRotate90 | Self::kTransverse | Self::kRotate270
        )
    }
}

#[derive(Clone, PartialEq)]
pub struct MaskLayer {
    pub resource_id: u64,
    pub image_interpolation_quality: ImageInterpolationQuality,
    pub paint_shader: Option<PaintShader>,
    pub clip_rect: PaintRect,
    pub clip_radii: PaintCornerRadii,
    pub source_rect: PaintRect,
    pub tile_rect: PaintRect,
    pub repeat_x: bool,
    pub repeat_y: bool,
    pub repeat_rule_x: BackgroundRepeatRule,
    pub repeat_rule_y: BackgroundRepeatRule,
    pub tile_scale: Offset,
    pub tile_spacing: Size,
    pub mode: PaintMaskMode,
    pub composite: PaintMaskComposite,
}
impl Default for MaskLayer {
    fn default() -> Self {
        Self {
            resource_id: 0,
            image_interpolation_quality: ImageInterpolationQuality::kMedium,
            paint_shader: None,
            clip_rect: PaintRect::default(),
            clip_radii: PaintCornerRadii::default(),
            source_rect: PaintRect::default(),
            tile_rect: PaintRect::default(),
            repeat_x: false,
            repeat_y: false,
            repeat_rule_x: BackgroundRepeatRule::kNoRepeat,
            repeat_rule_y: BackgroundRepeatRule::kNoRepeat,
            tile_scale: Offset { x: 1.0, y: 1.0 },
            tile_spacing: Size::default(),
            mode: PaintMaskMode::kAlpha,
            composite: PaintMaskComposite::kAdd,
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct DrawCommand {
    pub r#type: CommandKind,
    pub rect: PaintRect,
    pub source_rect: PaintRect,
    pub inner_rect: PaintRect,
    pub tile_rect: PaintRect,
    pub color: Color,
    pub line_style: BorderLineStyle,
    pub decoration_style: TextDecorationStyle,
    pub stroke_width: f64,
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
    pub filters: Vec<PaintFilterOperation>,
    pub mask_layers: Vec<MaskLayer>,
    pub round_cap: bool,
    pub resource_id: u64,
    pub image_interpolation_quality: ImageInterpolationQuality,
    pub image_orientation: ImageOrientation,
    pub repeat_x: bool,
    pub repeat_y: bool,
    pub tile_scale: Offset,
    pub tile_phase: Offset,
    pub tile_spacing: Size,
    pub shadow_offset: Offset,
    pub blur_radius: f64,
    pub spread: f64,
    pub inset: bool,
    pub shadow_has_opaque_background: bool,
    pub is_text_decoration: bool,
    pub stroke_glyphs: bool,
    pub opacity: f32,
    pub transform: TransformMatrix,
    pub font_face_index: u32,
    pub font_variations: Vec<FontVariation>,
    pub font_size: f64,
    pub text_line_top: f64,
    pub text_line_height: f64,
    pub text_blob_origin: Offset,
    pub synthetic_bold: bool,
    pub synthetic_italic: bool,
    pub font_smoothing: FontSmoothing,
    pub glyphs: Vec<PaintGlyph>,
}
impl Default for DrawCommand {
    fn default() -> Self {
        Self {
            r#type: CommandKind::kDrawRect,
            rect: PaintRect::default(),
            source_rect: PaintRect::default(),
            inner_rect: PaintRect::default(),
            tile_rect: PaintRect::default(),
            color: Color::default(),
            line_style: BorderLineStyle::kNone,
            decoration_style: TextDecorationStyle::kSolid,
            stroke_width: 0.0,
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
            filters: Vec::new(),
            mask_layers: Vec::new(),
            round_cap: false,
            resource_id: 0,
            image_interpolation_quality: ImageInterpolationQuality::kMedium,
            image_orientation: ImageOrientation::kDefault,
            repeat_x: false,
            repeat_y: false,
            tile_scale: Offset { x: 1.0, y: 1.0 },
            tile_phase: Offset::default(),
            tile_spacing: Size::default(),
            shadow_offset: Offset::default(),
            blur_radius: 0.0,
            spread: 0.0,
            inset: false,
            shadow_has_opaque_background: false,
            is_text_decoration: false,
            stroke_glyphs: false,
            opacity: 1.0,
            transform: TransformMatrix::default(),
            font_face_index: 0,
            font_variations: Vec::new(),
            font_size: 16.0,
            text_line_top: 0.0,
            text_line_height: 0.0,
            text_blob_origin: Offset::default(),
            synthetic_bold: false,
            synthetic_italic: false,
            font_smoothing: FontSmoothing::kAuto,
            glyphs: Vec::new(),
        }
    }
}

/// An already resolved shaping face. The exact bytes, collection index, and
/// variation coordinates identify the raster face. Names and style metadata
/// are descriptive and must never trigger a second platform-family selection.
/// Typeface metadata borrows strings/font bytes from the caller.
pub struct FontFace<'a> {
    pub family: &'a str,
    pub native_family: &'a str,
    pub weight: f64,
    pub italic: bool,
    pub bytes: &'a [u8],
    /// Optional retained immutable backing. Implementations must expose the same
    /// stable immutable full slice for their lifetime. Its view must match `bytes`;
    /// borrowed/reassigned views safely fall back to an owned exact-byte copy.
    pub bytes_owner: Option<std::sync::Arc<dyn AsRef<[u8]> + Send + Sync>>,
    pub face_index: u32,
    pub variations: Vec<FontVariation>,
}
impl FontFace<'_> {
    pub(crate) fn shared_bytes(&self) -> Option<&std::sync::Arc<dyn AsRef<[u8]> + Send + Sync>> {
        self.bytes_owner.as_ref().filter(|owner| {
            let view: &[u8] = owner.as_ref().as_ref();
            view.as_ptr() == self.bytes.as_ptr() && view.len() == self.bytes.len()
        })
    }
}
// The raster cache outlives a borrowed resource snapshot. Keep exact source
// bytes, not a pointer or family name, so data replacement and allocator address
// reuse cannot return outlines from an earlier font-context generation.
#[cfg(any(target_os = "macos", test))]
pub(crate) struct FontDataIdentity {
    bytes: Box<[u8]>,
    face_index: u32,
}
#[cfg(any(target_os = "macos", test))]
impl FontDataIdentity {
    pub(crate) fn new(face: &FontFace<'_>) -> Self {
        Self {
            bytes: face.bytes.into(),
            face_index: face.face_index,
        }
    }

    pub(crate) fn matches(&self, face: &FontFace<'_>) -> bool {
        self.face_index == face.face_index && self.bytes.as_ref() == face.bytes
    }

    pub(crate) fn byte_len(&self) -> usize {
        self.bytes.len()
    }
}

/// SkTypeface_Mac's variation argument conversion uses the last coordinate for
/// an overspecified axis (CSS Fonts 4). Canonical order also makes equivalent
/// shape/paint coordinates share a strike regardless of declaration order.
pub(crate) fn canonical_font_variations(
    variations: &[FontVariation],
) -> std::borrow::Cow<'_, [FontVariation]> {
    if variations.windows(2).all(|pair| pair[0].tag < pair[1].tag) {
        return std::borrow::Cow::Borrowed(variations);
    }
    let values: std::collections::BTreeMap<_, _> = variations
        .iter()
        .map(|axis| (axis.tag, axis.value))
        .collect();
    std::borrow::Cow::Owned(
        values
            .into_iter()
            .map(|(tag, value)| FontVariation { tag, value })
            .collect(),
    )
}

pub struct Image<'a> {
    pub id: u64,
    pub width: u32,
    pub height: u32,
    pub rgba8: &'a [u8],
    rgba8_owner: Option<&'a std::sync::Arc<Vec<u8>>>,
}
impl<'a> Image<'a> {
    pub fn borrowed(id: u64, width: u32, height: u32, rgba8: &'a [u8]) -> Self {
        Self {
            id,
            width,
            height,
            rgba8,
            rgba8_owner: None,
        }
    }
    pub fn shared(id: u64, width: u32, height: u32, rgba8: &'a std::sync::Arc<Vec<u8>>) -> Self {
        Self {
            id,
            width,
            height,
            rgba8,
            rgba8_owner: Some(rgba8),
        }
    }
    pub(crate) fn shared_pixels(&self) -> Option<&std::sync::Arc<Vec<u8>>> {
        // Public byte views may be reassigned. Only the complete owner view
        // proves immutable identity; unrelated borrowed bytes use exact comparison.
        self.rgba8_owner.filter(|owner| {
            owner.as_ptr() == self.rgba8.as_ptr() && owner.len() == self.rgba8.len()
        })
    }
}
#[derive(Default)]
pub struct ResourceCatalog<'a> {
    pub fonts: Vec<FontFace<'a>>,
    pub images: Vec<Image<'a>>,
}
#[derive(Default)]
pub struct ResourceContext<'a> {
    pub resources: Option<ResourceCatalog<'a>>,
}

#[cfg(test)]
mod font_identity_tests {
    use super::*;

    fn face(bytes: &[u8]) -> FontFace<'_> {
        FontFace {
            family: "alias",
            native_family: "Arial",
            weight: 400.0,
            italic: false,
            bytes,
            bytes_owner: None,
            face_index: 0,
            variations: vec![],
        }
    }

    #[test]
    fn exact_source_identity_survives_aliases_and_rejects_replaced_bytes() {
        let mut bytes = vec![1, 2, 3];
        let identity = FontDataIdentity::new(&face(&bytes));
        assert_eq!(identity.byte_len(), bytes.len());
        let copy = bytes.clone();
        let mut alias = face(&copy);
        alias.family = "different alias";
        alias.native_family = "a missing platform family";
        alias.weight = 700.0;
        alias.italic = true;
        assert!(identity.matches(&alias));
        alias.face_index = 1;
        assert!(!identity.matches(&alias));
        alias.face_index = 0x10000;
        assert!(
            !identity.matches(&alias),
            "named-instance index is identity"
        );
        bytes[0] = 4;
        assert!(
            !identity.matches(&face(&bytes)),
            "same allocation, new bytes"
        );
    }

    #[test]
    fn variation_identity_is_order_independent_and_last_declaration_wins() {
        let wght = u32::from_be_bytes(*b"wght");
        let opsz = u32::from_be_bytes(*b"opsz");
        let input = [
            FontVariation {
                tag: wght,
                value: 400.0,
            },
            FontVariation {
                tag: opsz,
                value: 12.0,
            },
            FontVariation {
                tag: wght,
                value: 700.0,
            },
        ];
        let expected = [
            FontVariation {
                tag: opsz,
                value: 12.0,
            },
            FontVariation {
                tag: wght,
                value: 700.0,
            },
        ];
        let actual = canonical_font_variations(&input);
        assert!(actual.as_ref() == expected);
        assert!(matches!(
            canonical_font_variations(&expected),
            std::borrow::Cow::Borrowed(_)
        ));
    }
}
