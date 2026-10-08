#![allow(non_camel_case_types)]

pub use foundation::style_constants::ECursor;
use std::sync::Arc;

use super::layout_input::{
    BorderLineStyle, Edges, ObjectFit, Offset, PaintPathCommand, PaintShadow, PaintStyle,
    SvgShapeData, TextDecorationPaint, TransformMatrix,
};
use super::layout_input_types::Color;

// cpp: layoutng/internal/paint_input.h:7-15
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ClipPathPaint {
    pub commands: Vec<PaintPathCommand>,
    pub percentage_commands: Vec<PaintPathCommand>,
    pub even_odd: bool,
}

// cpp: layoutng/internal/paint_input.h:17-36
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintFilterType {
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
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaintFilterOperation {
    pub r#type: PaintFilterType,
    pub amount: f64,
    pub offset: Offset,
    pub blur_radius: f64,
    pub color: Color,
}
impl Default for PaintFilterOperation {
    fn default() -> Self {
        Self {
            r#type: PaintFilterType::kBlur,
            amount: 0.0,
            offset: Offset::default(),
            blur_radius: 0.0,
            color: Color::default(),
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

// cpp: layoutng/internal/paint_input.h:56-64
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PaintColorStop {
    pub offset: f64,
    pub color: Color,
    pub offset_length: f64,
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
    pub transform: TransformMatrix,
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
            transform: TransformMatrix::default(),
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
    pub shader: Option<Arc<PaintShader>>,
    pub size_mode: BackgroundSizeMode,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub width_percentage: Option<f64>,
    pub height_percentage: Option<f64>,
    pub position: Offset,
    pub position_offset: Offset,
    pub repeat: BackgroundRepeat,
    pub repeat_rule_x: Option<BackgroundRepeatRule>,
    pub repeat_rule_y: Option<BackgroundRepeatRule>,
    pub origin: BackgroundBox,
    pub clip: BackgroundBox,
    pub blend_mode: PaintBlendMode,
}
impl Default for BackgroundImageLayer {
    fn default() -> Self {
        Self {
            resource_id: 0,
            source_url: String::new(),
            shader: None,
            size_mode: BackgroundSizeMode::kAuto,
            width: None,
            height: None,
            width_percentage: None,
            height_percentage: None,
            position: Offset::default(),
            position_offset: Offset::default(),
            repeat: BackgroundRepeat::kRepeat,
            repeat_rule_x: None,
            repeat_rule_y: None,
            origin: BackgroundBox::kPaddingBox,
            clip: BackgroundBox::kBorderBox,
            blend_mode: PaintBlendMode::kNormal,
        }
    }
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
#[derive(Clone, PartialEq)]
pub struct PaintMaskLayer {
    pub image: BackgroundImageLayer,
    pub mode: PaintMaskMode,
    pub composite: PaintMaskComposite,
}
impl Default for PaintMaskLayer {
    fn default() -> Self {
        Self {
            image: BackgroundImageLayer::default(),
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
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NinePieceImagePaint {
    pub resource_id: u64,
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
#[derive(Clone, PartialEq)]
pub struct SvgMarkerPrimitive {
    pub shape: SvgShapeData,
    pub fill: Option<Color>,
    pub stroke: Option<Color>,
    pub fill_server: Option<SvgPaintServer>,
    pub stroke_server: Option<SvgPaintServer>,
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
impl Default for SvgMarkerPrimitive {
    fn default() -> Self {
        Self {
            shape: SvgShapeData::default(),
            fill: None,
            stroke: None,
            fill_server: None,
            stroke_server: None,
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
    pub primitives: Vec<SvgMarkerPrimitive>,
}

// cpp: layoutng/internal/paint_input.h:282-351
#[derive(Clone, PartialEq)]
pub struct PaintStyleData {
    pub color: Color,
    pub accent_color: Option<Color>,
    pub clip_path: Option<ClipPathPaint>,
    pub filters: Vec<PaintFilterOperation>,
    pub mask_images: Vec<PaintMaskLayer>,
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
    pub cursor: ECursor,
    pub z_index: Option<i32>,
    pub transform_origin: Option<PaintTransformOrigin>,
    pub transform: Option<PaintTransform>,
    pub will_change_transform: bool,
    pub image_resource_id: Option<u64>,
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
            accent_color: None,
            clip_path: None,
            filters: Vec::new(),
            mask_images: Vec::new(),
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
            cursor: ECursor::kAuto,
            z_index: None,
            transform_origin: None,
            transform: None,
            will_change_transform: false,
            image_resource_id: None,
            object_fit: ObjectFit::kFill,
            object_position: Offset { x: 0.5, y: 0.5 },
            object_position_offset: Offset::default(),
        }
    }
}

// cpp: layoutng/internal/paint_input.cc:7-7
impl Default for PaintStyle {
    fn default() -> Self {
        Self {
            data: Arc::new(PaintStyleData::default()),
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
