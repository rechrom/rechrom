#![allow(non_camel_case_types)]

pub use foundation::style_constants::ECursor;
pub use foundation::InterpolationQuality;
use std::sync::{Arc, OnceLock};

use super::layout_input::{
    BorderLineStyle, Edges, ObjectFit, Offset, PaintImage, PaintPathCommand, PaintShadow,
    PaintStyle, Size, SvgShapeData, SvgViewBoxData, TextDecorationPaint, TransformMatrix,
};
use super::layout_input_types::Color;

/// One CSS length-percentage retained until the clip reference box is known.
/// Percentages use CSS percentage units (50.0 means 50%), matching `Length`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ClipPathLength {
    pub pixels: f64,
    pub percentage: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ClipPathRadius {
    pub x: ClipPathLength,
    pub y: ClipPathLength,
}

/// The immutable equivalent of Blink's `BasicShapeInset`.  Keeping the
/// semantic shape here lets PrePaint resolve it against the actual fragment
/// reference box, just as `ShapeClipPathOperation::GetPath()` does in Blink.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ClipPathInset {
    pub top: ClipPathLength,
    pub right: ClipPathLength,
    pub bottom: ClipPathLength,
    pub left: ClipPathLength,
    pub top_left_radius: ClipPathRadius,
    pub top_right_radius: ClipPathRadius,
    pub bottom_right_radius: ClipPathRadius,
    pub bottom_left_radius: ClipPathRadius,
}

/// Blink's keyword/value radius retained until the clip reference box exists.
/// `value` is used only by `kValue`; the keyword variants are resolved from
/// the final center and box dimensions in PrePaint.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ClipPathRadiusKind {
    kValue,
    #[default]
    kClosestSide,
    kFarthestSide,
    kClosestCorner,
    kFarthestCorner,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ClipPathShapeRadius {
    pub kind: ClipPathRadiusKind,
    pub value: ClipPathLength,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ClipPathCircle {
    pub center_x: ClipPathLength,
    pub center_y: ClipPathLength,
    pub radius: ClipPathShapeRadius,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ClipPathEllipse {
    pub center_x: ClipPathLength,
    pub center_y: ClipPathLength,
    pub radius_x: ClipPathShapeRadius,
    pub radius_y: ClipPathShapeRadius,
}

/// The coordinate box remains semantic across the Style/Layout/Paint package
/// boundary. PrePaint is the first owner that has both the final fragment box
/// and SVG geometry needed to resolve it.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ClipPathReferenceBox {
    #[default]
    kBorderBox,
    kPaddingBox,
    kContentBox,
    kMarginBox,
    kFillBox,
    kStrokeBox,
    kViewBox,
    kHalfBorderBox,
}

// cpp: layoutng/internal/paint_input.h:7-15
#[derive(Clone, Debug, PartialEq)]
pub struct ClipPathPaint {
    pub reference_url: Option<String>,
    pub reference_target_id: Option<u64>,
    /// Resolved by the referenced native SVG resource's viewport owner during
    /// Layout export. Percentages do not use the consumer's bounding box.
    pub reference_viewport: Option<super::layout_input::Size>,
    pub reference_scope:
        Option<layoutng_style::style::reference_clip_path_operation::SvgClipResourceScope>,
    pub reference_graph: Option<Arc<SvgClipPathGraph>>,
    pub reference_error: Option<String>,
    pub reference_box: ClipPathReferenceBox,
    pub inset: Option<ClipPathInset>,
    pub circle: Option<ClipPathCircle>,
    pub ellipse: Option<ClipPathEllipse>,
    pub commands: Vec<PaintPathCommand>,
    pub percentage_commands: Vec<PaintPathCommand>,
    /// `Some` identifies polygon() even when its requested radius is zero.
    /// Chromium restricts this value to a length; retaining a length-percentage
    /// still preserves calc() without prematurely resolving it.
    pub polygon_rounding_radius: Option<ClipPathLength>,
    /// path() coordinates are unzoomed SVG user units. Blink applies the
    /// object's effective zoom only when constructing the final Path.
    pub path_zoom: f64,
    pub even_odd: bool,
}
impl Default for ClipPathPaint {
    fn default() -> Self {
        Self {
            reference_url: None,
            reference_target_id: None,
            reference_viewport: None,
            reference_scope: None,
            reference_graph: None,
            reference_error: None,
            reference_box: ClipPathReferenceBox::kBorderBox,
            inset: None,
            circle: None,
            ellipse: None,
            commands: Vec::new(),
            percentage_commands: Vec::new(),
            polygon_rounding_radius: None,
            path_zoom: 1.0,
            even_odd: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SvgClipPathUnits {
    kUserSpaceOnUse,
    kObjectBoundingBox,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SvgClipTransform {
    pub values: [f64; 6],
    /// Translation coefficients for reference width/height:
    /// [tx/width, tx/height, ty/width, ty/height].
    pub percentages: [f64; 4],
}
impl Default for SvgClipTransform {
    fn default() -> Self {
        Self {
            values: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            percentages: [0.0; 4],
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SvgClipLength {
    pub value: f64,
    /// Fractional SVG percentage (0.5 means 50%).
    pub percentage: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub enum SvgClipGeometry {
    kCircle {
        cx: SvgClipLength,
        cy: SvgClipLength,
        r: SvgClipLength,
    },
    kEllipse {
        cx: SvgClipLength,
        cy: SvgClipLength,
        rx: SvgClipLength,
        ry: SvgClipLength,
    },
    kRect {
        x: SvgClipLength,
        y: SvgClipLength,
        width: SvgClipLength,
        height: SvgClipLength,
        rx: Option<SvgClipLength>,
        ry: Option<SvgClipLength>,
    },
    kPolygon {
        points: Vec<(SvgClipLength, SvgClipLength)>,
        close: bool,
    },
    kLine {
        x1: SvgClipLength,
        y1: SvgClipLength,
        x2: SvgClipLength,
        y2: SvgClipLength,
    },
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SvgClipTextAnchor {
    #[default]
    kStart,
    kMiddle,
    kEnd,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SvgClipTextPath {
    pub points: Vec<Offset>,
    pub start_offset: SvgClipLength,
    pub reverse_direction: bool,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SvgClipText {
    pub text: String,
    pub x: Vec<SvgClipLength>,
    pub y: Vec<SvgClipLength>,
    pub dx: Vec<SvgClipLength>,
    pub dy: Vec<SvgClipLength>,
    pub rotate: Vec<f64>,
    pub font_families: Vec<String>,
    pub font_size: f64,
    pub font_weight: f64,
    pub italic: bool,
    pub letter_spacing: f64,
    pub word_spacing: f64,
    pub rtl: bool,
    pub language: String,
    pub anchor: SvgClipTextAnchor,
    pub text_path: Option<SvgClipTextPath>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SvgClipPreserveAspectRatio {
    kNone,
    kMeet { align_x: f64, align_y: f64 },
    kSlice { align_x: f64, align_y: f64 },
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SvgClipViewport {
    pub view_box: [f64; 4],
    pub width: SvgClipLength,
    pub height: SvgClipLength,
    pub preserve: SvgClipPreserveAspectRatio,
}
#[derive(Clone, Debug, PartialEq)]
pub enum SvgClipTransformStep {
    kMatrix(SvgClipTransform),
    kViewport(SvgClipViewport),
}
#[derive(Clone, Debug, PartialEq)]
pub struct SvgClipPathChild {
    pub commands: Vec<PaintPathCommand>,
    pub geometry: Option<SvgClipGeometry>,
    pub text: Option<SvgClipText>,
    pub even_odd: bool,
    /// Ordered SVG transform list. Viewport steps remain semantic until
    /// PrePaint knows the clip reference viewport.
    pub transforms: Vec<SvgClipTransformStep>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SvgClipPathGraph {
    pub id: String,
    pub units: SvgClipPathUnits,
    pub transform: SvgClipTransform,
    pub children: Vec<SvgClipPathChild>,
    pub unsupported: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SvgClipResourceVersion {
    pub document_id: u64,
    pub revision: u64,
}

/// The first connected element with this ID in its TreeScope. A wrong-kind
/// target deliberately has no graph; a later clipPath with the same ID cannot
/// replace it. A present graph with no children is a valid empty clip.
#[derive(Clone, Debug, PartialEq)]
pub struct SvgClipResourceEntry {
    pub scope_id: u64,
    pub target_node_id: u64,
    pub id: String,
    pub graph: Option<Arc<SvgClipPathGraph>>,
    pub mask_graph: Option<Arc<SvgMaskGraph>>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SvgClipResourceSnapshot {
    pub version: SvgClipResourceVersion,
    pub entries: Vec<SvgClipResourceEntry>,
}

/// Non-CSS resource ownership carried into the resident Layout input tree.
/// It outlives paint-only replacements of the canonical computed style.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SvgClipResourceBinding {
    pub snapshot: Arc<SvgClipResourceSnapshot>,
    pub scope: layoutng_style::style::reference_clip_path_operation::SvgClipResourceScope,
}

// cpp: layoutng/internal/paint_input.h:17-36
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
    /// `-webkit-box-reflect` is appended after the authored CSS filter list,
    /// matching PaintLayer::FilterOperationsIncludingReflection.  It remains
    /// semantic until PrePaint knows the fragment's used border-box size.
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

/// One draw from the private reflection-mask PaintRecord. The source image is
/// retained with the used container variant so Raster and Renderer do not
/// perform a second image-set/DPR selection.
#[derive(Clone, Debug, PartialEq)]
pub struct PaintReflectionMaskLayer {
    pub image: Option<PaintImage>,
    pub document_image_container_key: Option<image_resource::ContainerKey>,
    pub paint_shader: Option<PaintShader>,
    pub image_interpolation_quality: InterpolationQuality,
    pub clip_origin: Offset,
    pub clip_size: Size,
    pub source_origin: Offset,
    pub source_size: Size,
    pub tile_origin: Offset,
    pub tile_size: Size,
    pub repeat_x: bool,
    pub repeat_y: bool,
    pub tile_spacing: Size,
}

impl Default for PaintReflectionMaskLayer {
    fn default() -> Self {
        Self {
            image: None,
            document_image_container_key: None,
            paint_shader: None,
            image_interpolation_quality: InterpolationQuality::kInterpolationMedium,
            clip_origin: Offset::default(),
            clip_size: Size::default(),
            source_origin: Offset::default(),
            source_size: Size::default(),
            tile_origin: Offset::default(),
            tile_size: Size::default(),
            repeat_x: false,
            repeat_y: false,
            tile_spacing: Size::default(),
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SvgFilterLength {
    pub value: f64,
    pub percentage: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SvgFilterUnits {
    kObjectBoundingBox,
    kUserSpaceOnUse,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SvgFilterRegion {
    pub x: Option<SvgFilterLength>,
    pub y: Option<SvgFilterLength>,
    pub width: Option<SvgFilterLength>,
    pub height: Option<SvgFilterLength>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SvgFilterInput {
    kSourceGraphic,
    kSourceAlpha,
    kBackgroundImage,
    kBackgroundAlpha,
    kFillPaint,
    kStrokePaint,
    kNamed(String),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SvgFilterEdgeMode {
    kNone,
    kDuplicate,
    kWrap,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SvgFilterChannel {
    kR,
    kG,
    kB,
    kA,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SvgMorphologyOperator {
    kErode,
    kDilate,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SvgTurbulenceType {
    kFractalNoise,
    kTurbulence,
}
#[derive(Clone, Debug, PartialEq)]
pub enum SvgLightSource {
    kDistant {
        azimuth: f64,
        elevation: f64,
    },
    kPoint {
        x: f64,
        y: f64,
        z: f64,
    },
    kSpot {
        x: f64,
        y: f64,
        z: f64,
        points_at_x: f64,
        points_at_y: f64,
        points_at_z: f64,
        specular_exponent: f64,
        limiting_cone_angle: f64,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SvgLightingType {
    kDiffuse,
    kSpecular,
}
#[derive(Clone, Debug, PartialEq)]
pub enum SvgTransferFunction {
    kIdentity,
    kTable(Vec<f64>),
    kDiscrete(Vec<f64>),
    kLinear {
        slope: f64,
        intercept: f64,
    },
    kGamma {
        amplitude: f64,
        exponent: f64,
        offset: f64,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SvgColorMatrixType {
    kMatrix,
    kSaturate,
    kHueRotate,
    kLuminanceToAlpha,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SvgBlendMode {
    kNormal,
    kMultiply,
    kScreen,
    kDarken,
    kLighten,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SvgCompositeOperator {
    kOver,
    kIn,
    kOut,
    kAtop,
    kXor,
    kArithmetic,
}
#[derive(Clone, Debug, PartialEq)]
pub enum SvgFilterPrimitiveKind {
    kGaussianBlur {
        sigma_x: f64,
        sigma_y: f64,
        edge_mode: SvgFilterEdgeMode,
    },
    kColorMatrix {
        kind: SvgColorMatrixType,
        values: Vec<f64>,
    },
    kOffset {
        dx: f64,
        dy: f64,
    },
    kFlood {
        color: Color,
    },
    kBlend {
        mode: SvgBlendMode,
        input2: SvgFilterInput,
    },
    kComposite {
        operator: SvgCompositeOperator,
        input2: SvgFilterInput,
        k: [f64; 4],
    },
    kMerge {
        inputs: Vec<SvgFilterInput>,
    },
    kComponentTransfer {
        functions: [SvgTransferFunction; 4],
    },
    kMorphology {
        operator: SvgMorphologyOperator,
        radius_x: f64,
        radius_y: f64,
    },
    kDisplacementMap {
        input2: SvgFilterInput,
        scale: f64,
        x_channel: SvgFilterChannel,
        y_channel: SvgFilterChannel,
    },
    kTurbulence {
        base_x: f64,
        base_y: f64,
        octaves: u32,
        seed: f64,
        stitch: bool,
        kind: SvgTurbulenceType,
    },
    kTile,
    kImage {
        href: String,
        local_fragment: bool,
        preserve_none: bool,
        resource_id: Option<u64>,
        width: u32,
        height: u32,
        pixels: Arc<Vec<u8>>,
    },
    kConvolveMatrix {
        order_x: u32,
        order_y: u32,
        kernel: Vec<f64>,
        divisor: f64,
        bias: f64,
        target_x: u32,
        target_y: u32,
        edge_mode: SvgFilterEdgeMode,
        kernel_unit_x: f64,
        kernel_unit_y: f64,
        preserve_alpha: bool,
    },
    kLighting {
        lighting_type: SvgLightingType,
        light: Option<SvgLightSource>,
        color: Color,
        surface_scale: f64,
        constant: f64,
        specular_exponent: f64,
        /// Fixed Chromium exposes this SVG attribute but does not pass it to
        /// FELighting/Skia; retaining it prevents loss at the package boundary.
        kernel_unit_x: f64,
        kernel_unit_y: f64,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub struct SvgFilterPrimitive {
    pub input: SvgFilterInput,
    pub result: String,
    pub region: SvgFilterRegion,
    pub linear_rgb: bool,
    pub kind: SvgFilterPrimitiveKind,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SvgFilterGraph {
    pub id: String,
    pub filter_units: SvgFilterUnits,
    pub primitive_units: SvgFilterUnits,
    pub region: SvgFilterRegion,
    pub primitives: Vec<SvgFilterPrimitive>,
    pub fill_paint: Option<Color>,
    pub stroke_paint: Option<Color>,
    /// Unknown primitives, recursive href chains, and unsupported built-ins
    /// remain explicit so paint cannot reinterpret the graph as `none`.
    pub unsupported: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaintFilterOperation {
    pub r#type: PaintFilterType,
    pub amount: f64,
    pub offset: Offset,
    pub blur_radius: f64,
    pub color: Color,
    pub reference_url: Option<String>,
    pub reference_graph: Option<Arc<SvgFilterGraph>>,
    pub reference_error: Option<String>,
    pub reflection_direction: PaintReflectionDirection,
    /// Fixed and percentage components of the computed reflection offset.
    /// PrePaint resolves the percentage against the physical box dimension.
    pub reflection_offset: f64,
    pub reflection_offset_percentage: f64,
    /// Expression-form `calc()`/`min()`/`max()`/`clamp()` stays typed until
    /// PrePaint knows the physical border-box dimension used by Blink.
    pub reflection_offset_calculation: Option<Arc<foundation::CalculationValue>>,
    /// A mask must never silently degrade to an unmasked reflection.  The
    /// NinePieceImage record is exported separately once its raster owner is
    /// available; until then Raster rejects this explicit state.
    pub reflection_has_mask: bool,
    /// Native NinePieceImage before Paint knows the concrete image/container.
    pub reflection_mask_image: Option<NinePieceImagePaint>,
    /// Private reflection-mask PaintRecord lowered to source/tile draws.
    pub reflection_mask_layers: Vec<PaintReflectionMaskLayer>,
}
impl Default for PaintFilterOperation {
    fn default() -> Self {
        Self {
            r#type: PaintFilterType::kBlur,
            amount: 0.0,
            offset: Offset::default(),
            blur_radius: 0.0,
            color: Color::default(),
            reference_url: None,
            reference_graph: None,
            reference_error: None,
            reflection_direction: PaintReflectionDirection::kBelow,
            reflection_offset: 0.0,
            reflection_offset_percentage: 0.0,
            reflection_offset_calculation: None,
            reflection_has_mask: false,
            reflection_mask_image: None,
            reflection_mask_layers: Vec::new(),
        }
    }
}

// cpp: layoutng/internal/paint_input.h:38-55
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
pub enum SvgPaintComponent {
    kFill,
    kStroke,
    kMarkers,
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

// cpp: layoutng/internal/paint_input.h:56-64
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PaintColorStop {
    pub offset: f64,
    pub color: Color,
    /// Original resolved CSS color coordinates. `kNone` means `color` is the
    /// source extended-sRGB value. The explicit representation retains CSS
    /// Color 4 missing components until adjacent-stop interpolation.
    pub source_color_space: PaintColorInterpolationSpace,
    pub source_components: [f32; 4],
    pub missing_components: u8,
    pub offset_length: f64,
    pub is_hint: bool,
}

// cpp: layoutng/internal/paint_input.h:65-111
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
    /// Vector content for an SVG <pattern>. Style owns resource/href
    /// resolution, Paint resolves its coordinate systems for each use, and
    /// Raster materializes the tile. Keeping the content here avoids a
    /// document-global last-use-wins bitmap for objectBoundingBox patterns.
    pub pattern: Option<Arc<SvgPatternData>>,
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

// cpp: layoutng/internal/paint_input.h:113-116
pub type SvgPaintServerKind = PaintShaderKind;
pub type SvgSpreadMethod = PaintSpreadMethod;
pub type SvgGradientStop = PaintColorStop;
pub type SvgPaintServer = PaintShader;

// cpp: layoutng/internal/paint_input.h:118-141
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackgroundRepeat {
    kRepeat,
    kRepeatX,
    kRepeatY,
    kNoRepeat,
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
pub enum BackgroundBox {
    kBorderBox,
    kPaddingBox,
    kContentBox,
    /// The painted background is retained in an isolated layer and masked by
    /// the alpha coverage of this box's descendant text.
    kText,
    /// CSS Backgrounds 4 border-area clip.  Unlike border-box this is the
    /// geometry actually painted by the used border styles.
    kBorderArea,
    /// Union of the used border-area and descendant text geometry.
    kBorderAreaText,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackgroundSizeMode {
    kAuto,
    kExplicit,
    kContain,
    kCover,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintImageTileRule {
    kStretch,
    kRepeat,
    kRound,
    kSpace,
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

// cpp: layoutng/internal/paint_input.h:143-164
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

// cpp: layoutng/internal/paint_input.h:166-195
#[derive(Clone, PartialEq)]
pub struct BackgroundImageLayer {
    pub resource_id: u64,
    pub source_url: String,
    /// Per-use StyleImage scale (for example the selected image-set()
    /// resolution). This cannot be recovered from the resource catalog:
    /// several computed-style uses may select the same decoded resource with
    /// different resolution descriptors.
    pub image_resolution_scale: f64,
    pub shader: Option<Arc<PaintShader>>,
    pub size_mode: BackgroundSizeMode,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub width_percentage: Option<f64>,
    pub height_percentage: Option<f64>,
    /// Retains min/max/clamp and other calculated sizes that cannot be
    /// represented by a pixels-plus-percentage pair.
    pub width_calculation: Option<Arc<foundation::CalculationValue>>,
    pub height_calculation: Option<Arc<foundation::CalculationValue>>,
    pub position: Offset,
    pub position_offset: Offset,
    pub position_x_calculation: Option<Arc<foundation::CalculationValue>>,
    pub position_y_calculation: Option<Arc<foundation::CalculationValue>>,
    pub position_x_from_right: bool,
    pub position_y_from_bottom: bool,
    pub repeat: BackgroundRepeat,
    pub repeat_rule_x: Option<BackgroundRepeatRule>,
    pub repeat_rule_y: Option<BackgroundRepeatRule>,
    pub attachment: BackgroundAttachment,
    pub origin: BackgroundBox,
    pub clip: BackgroundBox,
    pub blend_mode: PaintBlendMode,
    /// The native FillLayer is the bottom layer, so its paint operation also
    /// owns the background color.  Keep this association across the immutable
    /// Style/Paint boundary: filtering out `image:none` layers must not make a
    /// higher image accidentally absorb the color.
    pub paints_background_color: bool,
}
impl Default for BackgroundImageLayer {
    fn default() -> Self {
        Self {
            resource_id: 0,
            source_url: String::new(),
            image_resolution_scale: 1.0,
            shader: None,
            size_mode: BackgroundSizeMode::kAuto,
            width: None,
            height: None,
            width_percentage: None,
            height_percentage: None,
            width_calculation: None,
            height_calculation: None,
            position: Offset::default(),
            position_offset: Offset::default(),
            position_x_calculation: None,
            position_y_calculation: None,
            position_x_from_right: false,
            position_y_from_bottom: false,
            repeat: BackgroundRepeat::kRepeat,
            repeat_rule_x: None,
            repeat_rule_y: None,
            attachment: BackgroundAttachment::kScroll,
            origin: BackgroundBox::kPaddingBox,
            clip: BackgroundBox::kBorderBox,
            blend_mode: PaintBlendMode::kNormal,
            paints_background_color: false,
        }
    }
}

// cpp: layoutng_style/style/computed_style_constants.h EFillAttachment.
// This package-owned value crosses the immutable Style/Paint boundary without
// leaking FillLayer ownership into paint.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BackgroundAttachment {
    #[default]
    kScroll,
    kLocal,
    kFixed,
}

// cpp: layoutng/internal/paint_input.h:197-203
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
/// Immutable StyleImage lifecycle needed by mask painting. A failed or `none`
/// layer still participates as transparent black; a loading layer suppresses
/// the complete masked element until the resource reaches a terminal state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaintMaskImageState {
    #[default]
    kReady,
    kLoading,
    kInvalid,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SvgMaskReference {
    pub id: String,
    pub scope: layoutng_style::style::reference_clip_path_operation::SvgClipResourceScope,
    pub graph: Option<Arc<SvgMaskGraph>>,
    pub target_id: Option<u64>,
    pub viewport: Option<Size>,
}

#[derive(Clone, PartialEq)]
pub struct PaintMaskLayer {
    pub svg_reference: Option<SvgMaskReference>,
    pub match_source: bool,
    pub source_error: Option<String>,
    pub image_state: PaintMaskImageState,
    pub image: BackgroundImageLayer,
    pub mode: PaintMaskMode,
    pub composite: PaintMaskComposite,
}
impl Default for PaintMaskLayer {
    fn default() -> Self {
        Self {
            svg_reference: None,
            match_source: false,
            source_error: None,
            image: BackgroundImageLayer::default(),
            image_state: PaintMaskImageState::kReady,
            mode: PaintMaskMode::kAlpha,
            composite: PaintMaskComposite::kAdd,
        }
    }
}

// cpp: layoutng/internal/paint_input.h:205-213
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PaintTransformOrigin {
    pub pixels: Offset,
    pub percentages: Offset,
    pub z: f64,
}

// cpp: layoutng/internal/paint_input.h:215-226
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintTransformOperationKind {
    kMatrix,
    kTranslate,
    kTranslate3D,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaintTransformOperation {
    pub kind: PaintTransformOperationKind,
    pub matrix: TransformMatrix,
    pub pixels: Offset,
    pub percentages: Offset,
    pub z: f64,
}
impl Default for PaintTransformOperation {
    fn default() -> Self {
        Self {
            kind: PaintTransformOperationKind::kMatrix,
            matrix: TransformMatrix::default(),
            pixels: Offset::default(),
            percentages: Offset::default(),
            z: 0.0,
        }
    }
}

// cpp: layoutng/internal/paint_input.h:228-240
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PaintTransform {
    pub operations: Vec<PaintTransformOperation>,
}
impl PaintTransform {
    pub fn from_matrix(value: &TransformMatrix) -> Self {
        Self {
            operations: vec![PaintTransformOperation {
                kind: PaintTransformOperationKind::kMatrix,
                matrix: *value,
                ..PaintTransformOperation::default()
            }],
        }
    }
    pub fn assign_matrix(&mut self, value: &TransformMatrix) -> &mut Self {
        self.operations = vec![PaintTransformOperation {
            kind: PaintTransformOperationKind::kMatrix,
            matrix: *value,
            ..PaintTransformOperation::default()
        }];
        self
    }
}

// cpp: layoutng/internal/paint_input.h:242-252
#[derive(Clone, Debug, PartialEq)]
pub struct NinePieceImagePaint {
    pub resource_id: u64,
    pub image_resolution_scale: f64,
    /// Source image dimensions in the coordinate space used by `slices`.
    /// Generated images use the border-image area, matching StyleImage::ImageSize.
    pub image_size: Size,
    /// Generated border images cross the Style/Paint boundary as immutable
    /// shader data. Fetched images continue to use `resource_id`.
    pub shader: Option<Arc<PaintShader>>,
    pub slices: Edges,
    pub widths: Edges,
    pub outsets: Edges,
    pub horizontal_rule: PaintImageTileRule,
    pub vertical_rule: PaintImageTileRule,
    pub fill: bool,
}
impl Default for NinePieceImagePaint {
    fn default() -> Self {
        Self {
            resource_id: 0,
            image_resolution_scale: 1.0,
            image_size: Size::default(),
            shader: None,
            slices: Edges::default(),
            widths: Edges::default(),
            outsets: Edges::default(),
            horizontal_rule: PaintImageTileRule::kStretch,
            vertical_rule: PaintImageTileRule::kStretch,
            fill: false,
        }
    }
}

// cpp: layoutng/internal/paint_input.h:254-280
#[derive(Clone, Debug, PartialEq)]
pub struct SvgPaintServerReferenceBox {
    pub offset: Offset,
    pub size: Size,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SvgMarkerPrimitive {
    pub shape: SvgShapeData,
    pub fill: Option<Color>,
    pub stroke: Option<Color>,
    pub fill_server: Option<SvgPaintServer>,
    pub stroke_server: Option<SvgPaintServer>,
    /// A context-fill/context-stroke paint server keeps the geometry of the
    /// context element. It must not be rebound to the marker child bounds.
    pub fill_server_reference_box: Option<SvgPaintServerReferenceBox>,
    pub stroke_server_reference_box: Option<SvgPaintServerReferenceBox>,
    pub stroke_width: f64,
    pub stroke_dash_array: Vec<f64>,
    pub stroke_dash_offset: f64,
    pub stroke_line_cap: SvgStrokeLineCap,
    pub stroke_line_join: SvgStrokeLineJoin,
    pub stroke_miter_limit: f64,
    pub fill_even_odd: bool,
    pub non_scaling_stroke: bool,
    pub antialias: bool,
    pub transform: Option<TransformMatrix>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SvgPatternImage {
    pub resource_id: u64,
    pub image_interpolation_quality: InterpolationQuality,
    pub destination_offset: Offset,
    pub destination_width: f64,
    pub destination_height: f64,
    pub source_offset: Offset,
    pub source_width: f64,
    pub source_height: f64,
    pub preserve_none: bool,
    pub align_x: i8,
    pub align_y: i8,
    pub slice: bool,
    pub transform: TransformMatrix,
    /// Exact document-image environment selected by Paint. Bitmap images
    /// leave this empty. Raster must never substitute another recorded
    /// container variant when the exact one has not been published yet.
    pub document_container_key: Option<image_resource::ContainerKey>,
    pub color_scheme_flags_is_normal: bool,
    pub used_dark_color_scheme: bool,
}

/// Unshaped SVG text retained until Paint owns the concrete FontFace catalog.
/// Paint replaces this command with one or more DrawGlyph outline commands;
/// raster never performs font selection or shaping.
#[derive(Clone, Debug, PartialEq)]
pub struct SvgPatternText {
    pub text: SvgClipText,
    pub paint: SvgMarkerPrimitive,
    pub reset_cursor: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SvgMaskUnits {
    kUserSpaceOnUse,
    kObjectBoundingBox,
}

/// Immutable local SVG <mask> resource. Style owns the hierarchical pattern
/// commands; Paint resolves unit geometry against the masked content bounds.
#[derive(Clone, Debug, PartialEq)]
pub struct SvgMaskGraph {
    pub id: String,
    pub mask_units: SvgMaskUnits,
    pub content_units: SvgMaskUnits,
    pub x: SvgClipLength,
    pub y: SvgClipLength,
    pub width: SvgClipLength,
    pub height: SvgClipLength,
    pub mode: PaintMaskMode,
    pub commands: Vec<SvgPatternCommand>,
    pub content_transform: TransformMatrix,
    pub resolved_clip: Option<[f64; 4]>,
    pub unsupported: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SvgPatternReferenceFilter {
    pub graph: Arc<SvgFilterGraph>,
    pub reference: Option<[f64; 4]>,
}

impl Default for SvgPatternImage {
    fn default() -> Self {
        Self {
            resource_id: 0,
            image_interpolation_quality: InterpolationQuality::kInterpolationMedium,
            destination_offset: Offset::default(),
            destination_width: 0.0,
            destination_height: 0.0,
            source_offset: Offset::default(),
            source_width: 0.0,
            source_height: 0.0,
            preserve_none: false,
            align_x: 0,
            align_y: 0,
            slice: false,
            transform: TransformMatrix::default(),
            document_container_key: None,
            color_scheme_flags_is_normal: true,
            used_dark_color_scheme: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SvgPatternCommand {
    SaveLayerOpacity(f32),
    SaveLayerFilter(Vec<PaintFilterOperation>),
    SaveLayerReferenceFilter(Arc<SvgPatternReferenceFilter>),
    SaveLayerBlend(PaintBlendMode),
    SaveLayerClip(Arc<SvgClipPathGraph>),
    SaveLayerMask(Arc<SvgMaskGraph>),
    Restore,
    DrawShape(SvgMarkerPrimitive),
    DrawText(SvgPatternText),
    DrawGlyph(SvgMarkerPrimitive),
    DrawImage(SvgPatternImage),
}

/// Immutable content inherited from the first <pattern> in an href chain
/// which owns children. Before Paint resolution `content_transform` is the
/// identity and the unit/viewBox fields are semantic. Afterwards the matrix
/// maps pattern-child user coordinates into the local tile rectangle.
#[derive(Clone, Debug, PartialEq)]
pub struct SvgPatternData {
    pub commands: Vec<SvgPatternCommand>,
    pub content_units_object_bounding_box: bool,
    pub view_box: Option<SvgViewBoxData>,
    pub content_transform: TransformMatrix,
    pub opacity: f32,
}
impl Default for SvgPatternData {
    fn default() -> Self {
        Self {
            commands: Vec::new(),
            content_units_object_bounding_box: false,
            view_box: None,
            content_transform: TransformMatrix::default(),
            opacity: 1.0,
        }
    }
}
impl Default for SvgMarkerPrimitive {
    fn default() -> Self {
        Self {
            shape: SvgShapeData::default(),
            fill: None,
            stroke: None,
            fill_server: None,
            stroke_server: None,
            fill_server_reference_box: None,
            stroke_server_reference_box: None,
            stroke_width: 1.0,
            stroke_dash_array: Vec::new(),
            stroke_dash_offset: 0.0,
            stroke_line_cap: SvgStrokeLineCap::kButt,
            stroke_line_join: SvgStrokeLineJoin::kMiter,
            stroke_miter_limit: 4.0,
            fill_even_odd: false,
            non_scaling_stroke: false,
            antialias: true,
            transform: None,
        }
    }
}
#[derive(Clone, Default, PartialEq)]
pub struct SvgMarkerInstance {
    pub resource_id: u64,
    pub transform: TransformMatrix,
    /// Marker viewport in marker-local coordinates when overflow clips the
    /// resource. It is applied after the instance transform and before child
    /// graphics, matching LayoutSVGResourceMarker's viewport clip.
    pub clip_offset: Option<Offset>,
    pub clip_size: Size,
    pub primitives: Vec<SvgMarkerPrimitive>,
}

/// The fragment-local box used to resolve percentage transforms and
/// transform-origin. Blink normalizes the SVG spellings of transform-box to
/// one of these two values before a layout box reaches Paint.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaintTransformReferenceBox {
    kContentBox,
    #[default]
    kBorderBox,
}

// cpp: layoutng/internal/paint_input.h:282-351
#[derive(Clone, PartialEq)]
pub struct PaintStyleData {
    pub color: Color,
    /// Chromium ComputedStyle::GetInterpolationQuality(), carried unchanged
    /// to every bitmap display item produced for this style.
    pub image_interpolation_quality: InterpolationQuality,
    pub text_fill_color: Color,
    pub text_stroke_color: Color,
    pub text_stroke_width: f64,
    /// Inputs to StyleEngine::ResolveColorSchemeForEmbedding. Keep these
    /// separate from resolved paint colors because SVG document images use
    /// the embedding element's computed scheme to build their own style tree.
    pub color_scheme_flags_is_normal: bool,
    pub used_dark_color_scheme: bool,
    pub accent_color: Option<Color>,
    pub clip_path: Option<ClipPathPaint>,
    pub filters: Vec<PaintFilterOperation>,
    /// Document-owned snapshots keyed by the local fragment identifier.
    pub svg_filter_graphs: Arc<Vec<SvgFilterGraph>>,
    /// Ordered CSS backdrop-filter chain. This remains distinct from `filters`:
    /// it samples the already-painted parent target before this effect content.
    pub backdrop_filters: Vec<PaintFilterOperation>,
    /// The canonical chain contains a reference/SVG filter graph which the
    /// flattened operation payload cannot represent. Consumers must reject it.
    pub backdrop_filter_has_reference: bool,
    pub mask_images: Vec<PaintMaskLayer>,
    pub svg_mask_resource: Option<Arc<SvgMaskGraph>>,
    /// Fragment-sized projection of -webkit-mask-box-image. It shares the
    /// nine-piece geometry contract with border-image but is consumed by the
    /// mask source phase rather than box decoration painting.
    pub mask_box_image: Option<NinePieceImagePaint>,
    pub blend_mode: PaintBlendMode,
    pub isolate_blending: bool,
    pub svg_fill: Option<Color>,
    pub svg_stroke: Option<Color>,
    pub svg_fill_current_color: bool,
    pub svg_stroke_current_color: bool,
    pub svg_stop_color: Color,
    pub svg_stop_current_color: bool,
    pub svg_stop_opacity: f32,
    pub svg_fill_reference: Option<String>,
    pub svg_stroke_reference: Option<String>,
    pub svg_fill_server: Option<SvgPaintServer>,
    pub svg_stroke_server: Option<SvgPaintServer>,
    pub svg_stroke_width: f64,
    pub svg_stroke_dash_array: Vec<f64>,
    pub svg_stroke_dash_offset: f64,
    pub svg_stroke_line_cap: SvgStrokeLineCap,
    pub svg_stroke_line_join: SvgStrokeLineJoin,
    pub svg_stroke_miter_limit: f64,
    pub svg_fill_even_odd: bool,
    pub svg_non_scaling_stroke: bool,
    pub svg_shape_antialias: bool,
    pub svg_paint_order: [SvgPaintComponent; 3],
    pub svg_markers: Vec<SvgMarkerInstance>,
    pub background_color: Color,
    pub background_clip: BackgroundBox,
    pub background_images: Vec<BackgroundImageLayer>,
    pub border_image: Option<NinePieceImagePaint>,
    pub box_shadows: Vec<PaintShadow>,
    pub text_shadows: Vec<PaintShadow>,
    /// Resolved decorating-box chain, in ancestor-to-descendant order. This
    /// mirrors Blink's AppliedTextDecorationVector; text-decoration is
    /// propagated by the formatting model rather than inherited as a CSS
    /// property.
    pub applied_text_decorations: Vec<TextDecorationPaint>,
    pub text_decoration: TextDecorationPaint,
    pub border_colors: [Color; 4],
    pub column_rule_width: f64,
    pub column_rule_style: BorderLineStyle,
    pub column_rule_color: Color,
    pub outline_color: Color,
    pub outline_width: f64,
    pub outline_offset: f64,
    pub outline_style: BorderLineStyle,
    pub border_radii: Option<PaintCornerRadii>,
    pub border_radii_percentages: PaintCornerRadii,
    pub border_radius: f64,
    pub opacity: f32,
    pub visible: bool,
    /// Inherited CSS pointer-events; HTML hit testing ignores only `none`.
    pub pointer_events_none: bool,
    /// Used user-select value needed by text-position hit testing and selection paint.
    pub user_select_none: bool,
    pub cursor: ECursor,
    pub z_index: Option<i32>,
    pub transform_origin: Option<PaintTransformOrigin>,
    pub transform: Option<PaintTransform>,
    pub transform_reference_box: PaintTransformReferenceBox,
    /// Parent perspective is a separate child-space transform node in Blink.
    pub perspective: Option<f64>,
    pub perspective_origin: PaintTransformOrigin,
    /// Used transform-style value after grouping properties force flattening.
    pub preserves_3d: bool,
    /// Syntactic 3D is needed for Chromium's backface inheritance reset.
    pub has_3d_transform_operation: bool,
    pub backface_visibility_hidden: bool,
    pub will_change_transform: bool,
    pub image_resource_id: Option<u64>,
    /// Foreground image for generated `content: <gradient>`. Unlike a CSS
    /// background this is painted by the replaced-content path.
    pub replaced_image_shader: Option<Arc<PaintShader>>,
    /// CSS image-orientation. The encoded resource orientation remains on
    /// PaintImage and is applied only when this use requests `from-image`.
    pub respect_image_orientation: bool,
    pub object_fit: ObjectFit,
    pub object_position: Offset,
    pub object_position_offset: Offset,
}
impl Default for PaintStyleData {
    fn default() -> Self {
        let black = Color {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            alpha: 1.0,
        };
        Self {
            color: black,
            image_interpolation_quality: InterpolationQuality::kInterpolationMedium,
            text_fill_color: black,
            text_stroke_color: black,
            text_stroke_width: 0.0,
            color_scheme_flags_is_normal: true,
            used_dark_color_scheme: false,
            accent_color: None,
            clip_path: None,
            filters: Vec::new(),
            svg_filter_graphs: Arc::new(Vec::new()),
            backdrop_filters: Vec::new(),
            backdrop_filter_has_reference: false,
            mask_images: Vec::new(),
            svg_mask_resource: None,
            mask_box_image: None,
            blend_mode: PaintBlendMode::kNormal,
            isolate_blending: false,
            svg_fill: Some(black),
            svg_stroke: None,
            svg_fill_current_color: false,
            svg_stroke_current_color: false,
            svg_stop_color: black,
            svg_stop_current_color: false,
            svg_stop_opacity: 1.0,
            svg_fill_reference: None,
            svg_stroke_reference: None,
            svg_fill_server: None,
            svg_stroke_server: None,
            svg_stroke_width: 1.0,
            svg_stroke_dash_array: Vec::new(),
            svg_stroke_dash_offset: 0.0,
            svg_stroke_line_cap: SvgStrokeLineCap::kButt,
            svg_stroke_line_join: SvgStrokeLineJoin::kMiter,
            svg_stroke_miter_limit: 4.0,
            svg_fill_even_odd: false,
            svg_non_scaling_stroke: false,
            svg_shape_antialias: true,
            svg_paint_order: [
                SvgPaintComponent::kFill,
                SvgPaintComponent::kStroke,
                SvgPaintComponent::kMarkers,
            ],
            svg_markers: Vec::new(),
            background_color: Color::default(),
            background_clip: BackgroundBox::kBorderBox,
            background_images: Vec::new(),
            border_image: None,
            box_shadows: Vec::new(),
            text_shadows: Vec::new(),
            applied_text_decorations: Vec::new(),
            text_decoration: TextDecorationPaint::default(),
            border_colors: [Color::default(); 4],
            column_rule_width: 0.0,
            column_rule_style: BorderLineStyle::kNone,
            column_rule_color: Color::default(),
            outline_color: Color::default(),
            outline_width: 0.0,
            outline_offset: 0.0,
            outline_style: BorderLineStyle::kSolid,
            border_radii: None,
            border_radii_percentages: PaintCornerRadii::default(),
            border_radius: 0.0,
            opacity: 1.0,
            visible: true,
            pointer_events_none: false,
            user_select_none: false,
            cursor: ECursor::kAuto,
            z_index: None,
            transform_origin: None,
            transform: None,
            transform_reference_box: PaintTransformReferenceBox::kBorderBox,
            perspective: None,
            perspective_origin: PaintTransformOrigin::default(),
            preserves_3d: false,
            has_3d_transform_operation: false,
            backface_visibility_hidden: false,
            will_change_transform: false,
            image_resource_id: None,
            replaced_image_shader: None,
            respect_image_orientation: true,
            object_fit: ObjectFit::kFill,
            object_position: Offset { x: 0.5, y: 0.5 },
            object_position_offset: Offset::default(),
        }
    }
}

// cpp: layoutng/internal/paint_input.cc:7-7
// base::ValuesEquivalent compares shared immutable groups by identity first.
// DerefMut below detaches before writing, so a changed group still compares all
// of its values; this is not a native-style-pointer geometry shortcut.
impl PartialEq for PaintStyle {
    fn eq(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.data, &other.data) || self.data == other.data
    }
}

impl Default for PaintStyle {
    fn default() -> Self {
        static DEFAULT_DATA: OnceLock<Arc<PaintStyleData>> = OnceLock::new();
        Self {
            data: Arc::clone(DEFAULT_DATA.get_or_init(|| Arc::new(PaintStyleData::default()))),
        }
    }
}

// cpp: layoutng/internal/paint_input.cc:15-15
// cpp: layoutng/internal/paint_input.cc:19-19
impl std::ops::Deref for PaintStyle {
    type Target = PaintStyleData;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

// cpp: layoutng/internal/paint_input.cc:9-13
// cpp: layoutng/internal/paint_input.cc:17-17
impl std::ops::DerefMut for PaintStyle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::make_mut(&mut self.data)
    }
}
