//! Convert paint inputs into independent Skia raster commands.
use skia::compat::commands as target;
pub trait ToSkia {
    type Output;
    fn to_skia(&self) -> Self::Output;
}
macro_rules! scalar {
    ($($t:ty),*) => {$ (impl ToSkia for $t {
        type Output = $t;
        fn to_skia(&self) -> $t { *self }
    })*};
}
scalar!(bool, u8, u32, u64, f32, f64);
impl<T: ToSkia> ToSkia for Vec<T> {
    type Output = Vec<T::Output>;
    fn to_skia(&self) -> Self::Output {
        self.iter().map(ToSkia::to_skia).collect()
    }
}
impl<T: ToSkia> ToSkia for Option<T> {
    type Output = Option<T::Output>;
    fn to_skia(&self) -> Self::Output {
        self.as_ref().map(ToSkia::to_skia)
    }
}
impl ToSkia for [f64; 16] {
    type Output = Self;
    fn to_skia(&self) -> Self {
        *self
    }
}
impl ToSkia for layoutng_assembly::internal::layout_input_types::Color {
    type Output = target::Color;
    fn to_skia(&self) -> Self::Output {
        target::Color {
            red: self.red.to_skia(),
            green: self.green.to_skia(),
            blue: self.blue.to_skia(),
            alpha: self.alpha.to_skia(),
        }
    }
}
impl ToSkia for layoutng_assembly::internal::layout_input::Offset {
    type Output = target::Offset;
    fn to_skia(&self) -> Self::Output {
        target::Offset {
            x: self.x.to_skia(),
            y: self.y.to_skia(),
        }
    }
}
impl ToSkia for layoutng_assembly::internal::layout_input::Size {
    type Output = target::Size;
    fn to_skia(&self) -> Self::Output {
        target::Size {
            width: self.width.to_skia(),
            height: self.height.to_skia(),
        }
    }
}
impl ToSkia for layoutng_assembly::internal::layout_input::TransformMatrix {
    type Output = target::TransformMatrix;
    fn to_skia(&self) -> Self::Output {
        target::TransformMatrix {
            values: self.values.to_skia(),
        }
    }
}
impl ToSkia for layoutng_assembly::internal::layout_input::PaintPathVerb {
    type Output = target::PaintPathVerb;
    fn to_skia(&self) -> Self::Output {
        match self {
            Self::kMoveTo => target::PaintPathVerb::kMoveTo,
            Self::kLineTo => target::PaintPathVerb::kLineTo,
            Self::kQuadraticTo => target::PaintPathVerb::kQuadraticTo,
            Self::kConicTo => target::PaintPathVerb::kConicTo,
            Self::kCubicTo => target::PaintPathVerb::kCubicTo,
            Self::kClose => target::PaintPathVerb::kClose,
        }
    }
}
impl ToSkia for layoutng_assembly::internal::layout_input::PaintPathCommand {
    type Output = target::PaintPathCommand;
    fn to_skia(&self) -> Self::Output {
        target::PaintPathCommand {
            verb: self.verb.to_skia(),
            control1: self.control1.to_skia(),
            control2: self.control2.to_skia(),
            point: self.point.to_skia(),
            conic_weight: self.conic_weight.to_skia(),
        }
    }
}
impl ToSkia for layoutng_assembly::internal::layout_input::BorderLineStyle {
    type Output = target::BorderLineStyle;
    fn to_skia(&self) -> Self::Output {
        match self {
            Self::kNone => target::BorderLineStyle::kNone,
            Self::kHidden => target::BorderLineStyle::kHidden,
            Self::kSolid => target::BorderLineStyle::kSolid,
            Self::kDashed => target::BorderLineStyle::kDashed,
            Self::kDotted => target::BorderLineStyle::kDotted,
            Self::kDouble => target::BorderLineStyle::kDouble,
            Self::kGroove => target::BorderLineStyle::kGroove,
            Self::kRidge => target::BorderLineStyle::kRidge,
            Self::kInset => target::BorderLineStyle::kInset,
            Self::kOutset => target::BorderLineStyle::kOutset,
        }
    }
}
impl ToSkia for layoutng_assembly::internal::layout_input::TextDecorationStyle {
    type Output = target::TextDecorationStyle;
    fn to_skia(&self) -> Self::Output {
        match self {
            Self::kSolid => target::TextDecorationStyle::kSolid,
            Self::kDouble => target::TextDecorationStyle::kDouble,
            Self::kDotted => target::TextDecorationStyle::kDotted,
            Self::kDashed => target::TextDecorationStyle::kDashed,
            Self::kWavy => target::TextDecorationStyle::kWavy,
        }
    }
}
impl ToSkia for layoutng_assembly::internal::layout_input::FontSmoothing {
    type Output = target::FontSmoothing;
    fn to_skia(&self) -> Self::Output {
        match self {
            Self::kAuto => target::FontSmoothing::kAuto,
            Self::kNone => target::FontSmoothing::kNone,
            Self::kAntialiased => target::FontSmoothing::kAntialiased,
            Self::kSubpixelAntialiased => target::FontSmoothing::kSubpixelAntialiased,
        }
    }
}
impl ToSkia for layoutng_assembly::internal::layout_input::FontVariation {
    type Output = target::FontVariation;
    fn to_skia(&self) -> Self::Output {
        target::FontVariation {
            tag: self.tag.to_skia(),
            value: self.value.to_skia(),
        }
    }
}
impl ToSkia for layoutng_assembly::fragment_tree::PaintGlyph {
    type Output = target::PaintGlyph;
    fn to_skia(&self) -> Self::Output {
        target::PaintGlyph {
            id: self.id.to_skia(),
            character_index: self.character_index.to_skia(),
            canvas_rotation: self.canvas_rotation.to_skia(),
            offset: self.offset.to_skia(),
            advance: self.advance.to_skia(),
        }
    }
}
impl ToSkia for layoutng_assembly::internal::paint_input::PaintCornerRadius {
    type Output = target::PaintCornerRadius;
    fn to_skia(&self) -> Self::Output {
        target::PaintCornerRadius {
            x: self.x.to_skia(),
            y: self.y.to_skia(),
        }
    }
}
impl ToSkia for layoutng_assembly::internal::paint_input::PaintCornerRadii {
    type Output = target::PaintCornerRadii;
    fn to_skia(&self) -> Self::Output {
        target::PaintCornerRadii {
            top_left: self.top_left.to_skia(),
            top_right: self.top_right.to_skia(),
            bottom_right: self.bottom_right.to_skia(),
            bottom_left: self.bottom_left.to_skia(),
        }
    }
}
impl ToSkia for layoutng_assembly::internal::paint_input::PaintBlendMode {
    type Output = target::PaintBlendMode;
    fn to_skia(&self) -> Self::Output {
        match self {
            Self::kNormal => target::PaintBlendMode::kNormal,
            Self::kMultiply => target::PaintBlendMode::kMultiply,
            Self::kScreen => target::PaintBlendMode::kScreen,
            Self::kOverlay => target::PaintBlendMode::kOverlay,
            Self::kDarken => target::PaintBlendMode::kDarken,
            Self::kLighten => target::PaintBlendMode::kLighten,
            Self::kColorDodge => target::PaintBlendMode::kColorDodge,
            Self::kColorBurn => target::PaintBlendMode::kColorBurn,
            Self::kHardLight => target::PaintBlendMode::kHardLight,
            Self::kSoftLight => target::PaintBlendMode::kSoftLight,
            Self::kDifference => target::PaintBlendMode::kDifference,
            Self::kExclusion => target::PaintBlendMode::kExclusion,
            Self::kHue => target::PaintBlendMode::kHue,
            Self::kSaturation => target::PaintBlendMode::kSaturation,
            Self::kColor => target::PaintBlendMode::kColor,
            Self::kLuminosity => target::PaintBlendMode::kLuminosity,
            Self::kPlusLighter => target::PaintBlendMode::kPlusLighter,
        }
    }
}
impl ToSkia for layoutng_assembly::internal::paint_input::PaintFilterType {
    type Output = target::PaintFilterType;
    fn to_skia(&self) -> Self::Output {
        match self {
            Self::kBlur => target::PaintFilterType::kBlur,
            Self::kBrightness => target::PaintFilterType::kBrightness,
            Self::kContrast => target::PaintFilterType::kContrast,
            Self::kGrayscale => target::PaintFilterType::kGrayscale,
            Self::kHueRotate => target::PaintFilterType::kHueRotate,
            Self::kInvert => target::PaintFilterType::kInvert,
            Self::kOpacity => target::PaintFilterType::kOpacity,
            Self::kSaturate => target::PaintFilterType::kSaturate,
            Self::kSepia => target::PaintFilterType::kSepia,
            Self::kDropShadow => target::PaintFilterType::kDropShadow,
        }
    }
}
impl ToSkia for layoutng_assembly::internal::paint_input::PaintFilterOperation {
    type Output = target::PaintFilterOperation;
    fn to_skia(&self) -> Self::Output {
        target::PaintFilterOperation {
            r#type: self.r#type.to_skia(),
            amount: self.amount.to_skia(),
            offset: self.offset.to_skia(),
            blur_radius: self.blur_radius.to_skia(),
            color: self.color.to_skia(),
        }
    }
}
impl ToSkia for layoutng_assembly::internal::paint_input::SvgStrokeLineCap {
    type Output = target::SvgStrokeLineCap;
    fn to_skia(&self) -> Self::Output {
        match self {
            Self::kButt => target::SvgStrokeLineCap::kButt,
            Self::kRound => target::SvgStrokeLineCap::kRound,
            Self::kSquare => target::SvgStrokeLineCap::kSquare,
        }
    }
}
impl ToSkia for layoutng_assembly::internal::paint_input::SvgStrokeLineJoin {
    type Output = target::SvgStrokeLineJoin;
    fn to_skia(&self) -> Self::Output {
        match self {
            Self::kMiter => target::SvgStrokeLineJoin::kMiter,
            Self::kRound => target::SvgStrokeLineJoin::kRound,
            Self::kBevel => target::SvgStrokeLineJoin::kBevel,
        }
    }
}
impl ToSkia for layoutng_assembly::internal::paint_input::PaintShaderKind {
    type Output = target::PaintShaderKind;
    fn to_skia(&self) -> Self::Output {
        match self {
            Self::kLinearGradient => target::PaintShaderKind::kLinearGradient,
            Self::kRadialGradient => target::PaintShaderKind::kRadialGradient,
            Self::kConicGradient => target::PaintShaderKind::kConicGradient,
            Self::kPattern => target::PaintShaderKind::kPattern,
        }
    }
}
impl ToSkia for layoutng_assembly::internal::paint_input::PaintSpreadMethod {
    type Output = target::PaintSpreadMethod;
    fn to_skia(&self) -> Self::Output {
        match self {
            Self::kPad => target::PaintSpreadMethod::kPad,
            Self::kReflect => target::PaintSpreadMethod::kReflect,
            Self::kRepeat => target::PaintSpreadMethod::kRepeat,
        }
    }
}
impl ToSkia for layoutng_assembly::internal::paint_input::PaintRadialShape {
    type Output = target::PaintRadialShape;
    fn to_skia(&self) -> Self::Output {
        match self {
            Self::kCircle => target::PaintRadialShape::kCircle,
            Self::kEllipse => target::PaintRadialShape::kEllipse,
        }
    }
}
impl ToSkia for layoutng_assembly::internal::paint_input::PaintRadialExtent {
    type Output = target::PaintRadialExtent;
    fn to_skia(&self) -> Self::Output {
        match self {
            Self::kExplicit => target::PaintRadialExtent::kExplicit,
            Self::kClosestSide => target::PaintRadialExtent::kClosestSide,
            Self::kClosestCorner => target::PaintRadialExtent::kClosestCorner,
            Self::kFarthestSide => target::PaintRadialExtent::kFarthestSide,
            Self::kFarthestCorner => target::PaintRadialExtent::kFarthestCorner,
        }
    }
}
impl ToSkia for layoutng_assembly::internal::paint_input::PaintColorStop {
    type Output = target::PaintColorStop;
    fn to_skia(&self) -> Self::Output {
        target::PaintColorStop {
            offset: self.offset.to_skia(),
            color: self.color.to_skia(),
            offset_length: self.offset_length.to_skia(),
        }
    }
}
impl ToSkia for layoutng_assembly::internal::paint_input::PaintShader {
    type Output = target::PaintShader;
    fn to_skia(&self) -> Self::Output {
        target::PaintShader {
            kind: self.kind.to_skia(),
            spread: self.spread.to_skia(),
            stops: self.stops.to_skia(),
            start: self.start.to_skia(),
            end: self.end.to_skia(),
            linear_angle: self.linear_angle.to_skia(),
            linear_start_offset: self.linear_start_offset.to_skia(),
            linear_end_offset: self.linear_end_offset.to_skia(),
            center: self.center.to_skia(),
            center_offset: self.center_offset.to_skia(),
            focal: self.focal.to_skia(),
            radius: self.radius.to_skia(),
            radius_y: self.radius_y.to_skia(),
            radius_offset: self.radius_offset.to_skia(),
            radius_y_offset: self.radius_y_offset.to_skia(),
            radial_shape: self.radial_shape.to_skia(),
            radial_extent: self.radial_extent.to_skia(),
            radial_start_offset: self.radial_start_offset.to_skia(),
            radial_end_offset: self.radial_end_offset.to_skia(),
            focal_radius: self.focal_radius.to_skia(),
            start_angle: self.start_angle.to_skia(),
            end_angle: self.end_angle.to_skia(),
            rotation_angle: self.rotation_angle.to_skia(),
            resource_id: self.resource_id.to_skia(),
            tile_offset: self.tile_offset.to_skia(),
            tile_width: self.tile_width.to_skia(),
            tile_height: self.tile_height.to_skia(),
            unit_coordinates: self.unit_coordinates.to_skia(),
            object_bounding_box_coordinates: self.object_bounding_box_coordinates.to_skia(),
            interpolate_premultiplied: self.interpolate_premultiplied.to_skia(),
            transform: self.transform.to_skia(),
        }
    }
}
impl ToSkia for layoutng_assembly::internal::paint_input::BackgroundRepeatRule {
    type Output = target::BackgroundRepeatRule;
    fn to_skia(&self) -> Self::Output {
        match self {
            Self::kRepeat => target::BackgroundRepeatRule::kRepeat,
            Self::kNoRepeat => target::BackgroundRepeatRule::kNoRepeat,
            Self::kRound => target::BackgroundRepeatRule::kRound,
            Self::kSpace => target::BackgroundRepeatRule::kSpace,
        }
    }
}
impl ToSkia for layoutng_assembly::internal::paint_input::PaintMaskMode {
    type Output = target::PaintMaskMode;
    fn to_skia(&self) -> Self::Output {
        match self {
            Self::kAlpha => target::PaintMaskMode::kAlpha,
            Self::kLuminance => target::PaintMaskMode::kLuminance,
        }
    }
}
impl ToSkia for layoutng_assembly::internal::paint_input::PaintMaskComposite {
    type Output = target::PaintMaskComposite;
    fn to_skia(&self) -> Self::Output {
        match self {
            Self::kAdd => target::PaintMaskComposite::kAdd,
            Self::kSubtract => target::PaintMaskComposite::kSubtract,
            Self::kIntersect => target::PaintMaskComposite::kIntersect,
            Self::kExclude => target::PaintMaskComposite::kExclude,
        }
    }
}
impl ToSkia for paint::paint_engine::PaintRect {
    type Output = target::PaintRect;
    fn to_skia(&self) -> Self::Output {
        target::PaintRect {
            x: self.x.to_skia(),
            y: self.y.to_skia(),
            width: self.width.to_skia(),
            height: self.height.to_skia(),
        }
    }
}
impl ToSkia for paint::paint_engine::DisplayItemType {
    type Output = target::CommandKind;
    fn to_skia(&self) -> Self::Output {
        match self {
            Self::kSave => target::CommandKind::kSave,
            Self::kRestore => target::CommandKind::kRestore,
            Self::kConcat => target::CommandKind::kConcat,
            Self::kClipRect => target::CommandKind::kClipRect,
            Self::kClipRoundedRect => target::CommandKind::kClipRoundedRect,
            Self::kClipOutRoundedRect => target::CommandKind::kClipOutRoundedRect,
            Self::kClipPath => target::CommandKind::kClipPath,
            Self::kClipOutRect => target::CommandKind::kClipOutRect,
            Self::kSaveLayer => target::CommandKind::kSaveLayer,
            Self::kSaveLayerAlpha => target::CommandKind::kSaveLayerAlpha,
            Self::kSaveLayerBlend => target::CommandKind::kSaveLayerBlend,
            Self::kSaveLayerFilter => target::CommandKind::kSaveLayerFilter,
            Self::kSaveLayerDstIn => target::CommandKind::kSaveLayerDstIn,
            Self::kBeginMask => target::CommandKind::kBeginMask,
            Self::kEndMask => target::CommandKind::kEndMask,
            Self::kDrawRect => target::CommandKind::kDrawRect,
            Self::kDrawMask => target::CommandKind::kDrawMask,
            Self::kDrawRoundedRect => target::CommandKind::kDrawRoundedRect,
            Self::kDrawDoubleRoundedRect => target::CommandKind::kDrawDoubleRoundedRect,
            Self::kDrawEllipse => target::CommandKind::kDrawEllipse,
            Self::kStrokeEllipse => target::CommandKind::kStrokeEllipse,
            Self::kDrawPath => target::CommandKind::kDrawPath,
            Self::kStrokePath => target::CommandKind::kStrokePath,
            Self::kDrawGradientRect => target::CommandKind::kDrawGradientRect,
            Self::kDrawTiledGradient => target::CommandKind::kDrawTiledGradient,
            Self::kStrokeRect => target::CommandKind::kStrokeRect,
            Self::kStrokeLine => target::CommandKind::kStrokeLine,
            Self::kStrokeWavyLine => target::CommandKind::kStrokeWavyLine,
            Self::kDrawGlyphRun => target::CommandKind::kDrawGlyphRun,
            Self::kDrawImageRect => target::CommandKind::kDrawImageRect,
            Self::kDrawTiledImage => target::CommandKind::kDrawTiledImage,
            Self::kDrawBoxShadow => target::CommandKind::kDrawBoxShadow,
            Self::kDrawScrollbarTrack => target::CommandKind::kDrawScrollbarTrack,
            Self::kDrawScrollbarThumb => target::CommandKind::kDrawScrollbarThumb,
            Self::kDrawScrollbarButton => target::CommandKind::kDrawScrollbarButton,
            Self::kDrawScrollbarCorner => target::CommandKind::kDrawScrollbarCorner,
        }
    }
}
impl ToSkia for paint::paint_engine::DisplayMaskLayer {
    type Output = target::MaskLayer;
    fn to_skia(&self) -> Self::Output {
        target::MaskLayer {
            resource_id: self.resource_id.to_skia(),
            paint_shader: self.paint_shader.to_skia(),
            clip_rect: self.clip_rect.to_skia(),
            clip_radii: self.clip_radii.to_skia(),
            source_rect: self.source_rect.to_skia(),
            tile_rect: self.tile_rect.to_skia(),
            repeat_x: self.repeat_x.to_skia(),
            repeat_y: self.repeat_y.to_skia(),
            repeat_rule_x: self.repeat_rule_x.to_skia(),
            repeat_rule_y: self.repeat_rule_y.to_skia(),
            tile_scale: self.tile_scale.to_skia(),
            tile_spacing: self.tile_spacing.to_skia(),
            mode: self.mode.to_skia(),
            composite: self.composite.to_skia(),
        }
    }
}
impl ToSkia for paint::paint_engine::DisplayItem {
    type Output = target::DrawCommand;
    fn to_skia(&self) -> Self::Output {
        target::DrawCommand {
            r#type: self.r#type.to_skia(),
            rect: self.rect.to_skia(),
            source_rect: self.source_rect.to_skia(),
            inner_rect: self.inner_rect.to_skia(),
            tile_rect: self.tile_rect.to_skia(),
            color: self.color.to_skia(),
            line_style: self.line_style.to_skia(),
            decoration_style: self.decoration_style.to_skia(),
            stroke_width: self.stroke_width.to_skia(),
            corner_radii: self.corner_radii.to_skia(),
            inner_corner_radii: self.inner_corner_radii.to_skia(),
            dash_intervals: self.dash_intervals.to_skia(),
            path: self.path.to_skia(),
            even_odd: self.even_odd.to_skia(),
            inverse_winding: self.inverse_winding.to_skia(),
            svg_line_cap: self.svg_line_cap.to_skia(),
            svg_line_join: self.svg_line_join.to_skia(),
            dash_offset: self.dash_offset.to_skia(),
            dash_fit_thickness: self.dash_fit_thickness.to_skia(),
            miter_limit: self.miter_limit.to_skia(),
            antialias: self.antialias.to_skia(),
            non_scaling_stroke: self.non_scaling_stroke.to_skia(),
            paint_shader: self.paint_shader.to_skia(),
            blend_mode: self.blend_mode.to_skia(),
            filters: self.filters.to_skia(),
            mask_layers: self.mask_layers.to_skia(),
            round_cap: self.round_cap.to_skia(),
            resource_id: self.resource_id.to_skia(),
            repeat_x: self.repeat_x.to_skia(),
            repeat_y: self.repeat_y.to_skia(),
            tile_spacing: self.tile_spacing.to_skia(),
            shadow_offset: self.shadow_offset.to_skia(),
            blur_radius: self.blur_radius.to_skia(),
            spread: self.spread.to_skia(),
            inset: self.inset.to_skia(),
            shadow_has_opaque_background: self.shadow_has_opaque_background.to_skia(),
            is_text_decoration: self.is_text_decoration.to_skia(),
            stroke_glyphs: self.stroke_glyphs.to_skia(),
            opacity: self.opacity.to_skia(),
            transform: self.transform.to_skia(),
            font_face_index: self.font_face_index.to_skia(),
            font_variations: self.font_variations.to_skia(),
            font_size: self.font_size.to_skia(),
            text_blob_origin: self.text_blob_origin.to_skia(),
            synthetic_bold: self.synthetic_bold.to_skia(),
            synthetic_italic: self.synthetic_italic.to_skia(),
            font_smoothing: self.font_smoothing.to_skia(),
            glyphs: self.glyphs.to_skia(),
        }
    }
}

pub fn resources(list: &paint::paint_engine::PaintArtifact) -> target::ResourceContext<'_> {
    target::ResourceContext {
        resources: list.resources.as_ref().map(|r| target::ResourceCatalog {
            fonts: r
                .fonts
                .iter()
                .map(|f| target::FontFace {
                    family: &f.family,
                    native_family: &f.native_family,
                    weight: f.weight,
                    italic: f.italic,
                    bytes: &f.bytes,
                    face_index: f.face_index,
                    variations: f.variations.to_skia(),
                })
                .collect(),
            images: r
                .images
                .iter()
                .filter_map(|i| {
                    i.BitmapPixels()
                        .map(|pixels| target::Image::shared(i.id, i.width, i.height, pixels))
                })
                .collect(),
        }),
    }
}
