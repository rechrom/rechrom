//! Convert paint inputs into independent Skia raster commands.
use raster_commands::commands as target;
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
impl ToSkia for foundation::ETextDecorationSkipInk {
    type Output = target::TextDecorationSkipInk;
    fn to_skia(&self) -> Self::Output {
        match self {
            Self::kNone => target::TextDecorationSkipInk::kNone,
            Self::kAll => target::TextDecorationSkipInk::kAll,
            Self::kAuto => target::TextDecorationSkipInk::kAuto,
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
impl ToSkia for layoutng_assembly::fragment_tree::PaintGlyphRun {
    type Output = target::TextInkSkipRun;
    fn to_skia(&self) -> Self::Output {
        target::TextInkSkipRun {
            font_face_index: self.font_face_index,
            font_variations: self.font_variations.to_skia(),
            font_size: self.font_size,
            baseline: self.baseline,
            synthetic_bold: self.synthetic_bold,
            synthetic_italic: self.synthetic_italic,
            glyphs: self.glyphs.as_ref().to_vec().to_skia(),
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
            Self::kReference => target::PaintFilterType::kReference,
            Self::kBlur => target::PaintFilterType::kBlur,
            Self::kBrightness => target::PaintFilterType::kBrightness,
            Self::kContrast => target::PaintFilterType::kContrast,
            Self::kGrayscale => target::PaintFilterType::kGrayscale,
            Self::kHueRotate => target::PaintFilterType::kHueRotate,
            Self::kLuminanceToAlpha => target::PaintFilterType::kLuminanceToAlpha,
            Self::kColorMatrix => target::PaintFilterType::kColorMatrix,
            Self::kInvert => target::PaintFilterType::kInvert,
            Self::kOpacity => target::PaintFilterType::kOpacity,
            Self::kSaturate => target::PaintFilterType::kSaturate,
            Self::kSepia => target::PaintFilterType::kSepia,
            Self::kDropShadow => target::PaintFilterType::kDropShadow,
            Self::kConvolveMatrix => target::PaintFilterType::kConvolveMatrix,
            Self::kComponentTransfer => target::PaintFilterType::kComponentTransfer,
            Self::kTurbulence => target::PaintFilterType::kTurbulence,
            Self::kBoxReflect => target::PaintFilterType::kBoxReflect,
        }
    }
}
impl ToSkia for layoutng_assembly::internal::paint_input::PaintFilterOperation {
    type Output = target::PaintFilterOperation;
    fn to_skia(&self) -> Self::Output {
        debug_assert!(self.reflection_offset_calculation.is_none());
        target::PaintFilterOperation {
            r#type: self.r#type.to_skia(),
            amount: self.amount.to_skia(),
            color_matrix: self.color_matrix.clone(),
            canvas_filter: self
                .canvas_filter
                .as_ref()
                .map(crate::svg_filter_graph::translate_canvas_filter),
            offset: self.offset.to_skia(),
            blur_radius: self.blur_radius.to_skia(),
            color: self.color.to_skia(),
            reflection_direction: match self.reflection_direction {
                layoutng_assembly::internal::paint_input::PaintReflectionDirection::kAbove => {
                    target::PaintReflectionDirection::kAbove
                }
                layoutng_assembly::internal::paint_input::PaintReflectionDirection::kBelow => {
                    target::PaintReflectionDirection::kBelow
                }
                layoutng_assembly::internal::paint_input::PaintReflectionDirection::kLeft => {
                    target::PaintReflectionDirection::kLeft
                }
                layoutng_assembly::internal::paint_input::PaintReflectionDirection::kRight => {
                    target::PaintReflectionDirection::kRight
                }
            },
            reflection_offset: self.reflection_offset,
            reflection_has_mask: self.reflection_has_mask,
            reflection_mask: self.reflection_has_mask.then(|| {
                let (layers, error) = match crate::reflection_mask::CompileReflectionMaskLayers(
                    &self.reflection_mask_layers,
                ) {
                    Ok(layers) => (layers, None),
                    Err(error) => (Vec::new(), Some(error.to_string())),
                };
                raster_commands::filter_graph::RenderReflectionMask {
                    // A specified but pending/empty mask is transparent. It
                    // must not degrade into an unmasked reflected branch.
                    layers,
                    error,
                }
            }),
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
impl ToSkia for layoutng_assembly::internal::paint_input::PaintColorInterpolationSpace {
    type Output = target::PaintColorInterpolationSpace;
    fn to_skia(&self) -> Self::Output {
        match self {
            Self::kNone => target::PaintColorInterpolationSpace::kNone,
            Self::kSRGB => target::PaintColorInterpolationSpace::kSRGB,
            Self::kSRGBLegacy => target::PaintColorInterpolationSpace::kSRGBLegacy,
            Self::kSRGBLinear => target::PaintColorInterpolationSpace::kSRGBLinear,
            Self::kDisplayP3 => target::PaintColorInterpolationSpace::kDisplayP3,
            Self::kDisplayP3Linear => target::PaintColorInterpolationSpace::kDisplayP3Linear,
            Self::kHSL => target::PaintColorInterpolationSpace::kHSL,
            Self::kHWB => target::PaintColorInterpolationSpace::kHWB,
            Self::kA98RGB => target::PaintColorInterpolationSpace::kA98RGB,
            Self::kProPhotoRGB => target::PaintColorInterpolationSpace::kProPhotoRGB,
            Self::kRec2020 => target::PaintColorInterpolationSpace::kRec2020,
            Self::kRec2100Linear => target::PaintColorInterpolationSpace::kRec2100Linear,
            Self::kLab => target::PaintColorInterpolationSpace::kLab,
            Self::kLch => target::PaintColorInterpolationSpace::kLch,
            Self::kOklab => target::PaintColorInterpolationSpace::kOklab,
            Self::kOklch => target::PaintColorInterpolationSpace::kOklch,
            Self::kXYZD50 => target::PaintColorInterpolationSpace::kXYZD50,
            Self::kXYZD65 => target::PaintColorInterpolationSpace::kXYZD65,
        }
    }
}
impl ToSkia for layoutng_assembly::internal::paint_input::PaintHueInterpolationMethod {
    type Output = target::PaintHueInterpolationMethod;
    fn to_skia(&self) -> Self::Output {
        match self {
            Self::kShorter => target::PaintHueInterpolationMethod::kShorter,
            Self::kLonger => target::PaintHueInterpolationMethod::kLonger,
            Self::kIncreasing => target::PaintHueInterpolationMethod::kIncreasing,
            Self::kDecreasing => target::PaintHueInterpolationMethod::kDecreasing,
        }
    }
}
impl ToSkia for layoutng_assembly::internal::paint_input::PaintColorStop {
    type Output = target::PaintColorStop;
    fn to_skia(&self) -> Self::Output {
        target::PaintColorStop {
            offset: self.offset.to_skia(),
            color: self.color.to_skia(),
            source_color_space: self.source_color_space.to_skia(),
            source_components: self.source_components,
            missing_components: self.missing_components,
            offset_length: self.offset_length.to_skia(),
            is_hint: self.is_hint,
        }
    }
}
impl ToSkia for layoutng_assembly::internal::paint_input::SvgMarkerPrimitive {
    type Output = target::SvgPatternPrimitive;
    fn to_skia(&self) -> Self::Output {
        use layoutng_assembly::internal::layout_input::SvgShapeGeometry;
        target::SvgPatternPrimitive {
            geometry: match self.shape.geometry {
                SvgShapeGeometry::kPath => target::SvgPatternGeometry::kPath,
                SvgShapeGeometry::kRectangle => target::SvgPatternGeometry::kRectangle,
                SvgShapeGeometry::kEllipse => target::SvgPatternGeometry::kEllipse,
            },
            bounds_offset: self.shape.bounds_offset.to_skia(),
            bounds_width: self.shape.bounds_width,
            bounds_height: self.shape.bounds_height,
            rectangle_radius_x: self.shape.rectangle_radius_x,
            rectangle_radius_y: self.shape.rectangle_radius_y,
            path: self.shape.path.to_skia(),
            fill: self.fill.to_skia(),
            stroke: self.stroke.to_skia(),
            fill_server: self.fill_server.to_skia(),
            stroke_server: self.stroke_server.to_skia(),
            stroke_width: self.stroke_width,
            stroke_dash_array: self.stroke_dash_array.clone(),
            stroke_dash_offset: self.stroke_dash_offset,
            stroke_line_cap: self.stroke_line_cap.to_skia(),
            stroke_line_join: self.stroke_line_join.to_skia(),
            stroke_miter_limit: self.stroke_miter_limit,
            fill_even_odd: self.fill_even_odd,
            non_scaling_stroke: self.non_scaling_stroke,
            antialias: self.antialias,
            apply_srgb_to_linear_color_filter: self.apply_srgb_to_linear_color_filter,
            transform: self.transform.unwrap_or_default().to_skia(),
        }
    }
}
impl ToSkia for layoutng_assembly::internal::paint_input::SvgPatternImage {
    type Output = target::SvgPatternImage;
    fn to_skia(&self) -> Self::Output {
        target::SvgPatternImage {
            resource_id: self.resource_id,
            image_interpolation_quality: self.image_interpolation_quality.to_skia(),
            dynamic_range_limit: self.dynamic_range_limit.to_skia(),
            destination: target::PaintRect {
                x: self.destination_offset.x,
                y: self.destination_offset.y,
                width: self.destination_width,
                height: self.destination_height,
            },
            source: target::PaintRect {
                x: self.source_offset.x,
                y: self.source_offset.y,
                width: self.source_width,
                height: self.source_height,
            },
            transform: self.transform.to_skia(),
            raster_width: 0,
            raster_height: 0,
            rgba8: Vec::new().into(),
        }
    }
}
impl ToSkia for layoutng_assembly::internal::paint_input::SvgPatternCommand {
    type Output = target::SvgPatternCommand;
    fn to_skia(&self) -> Self::Output {
        match self {
            Self::SaveLayerOpacity(opacity) => {
                target::SvgPatternCommand::SaveLayerOpacity(*opacity)
            }
            Self::SaveLayerFilter(filters) => {
                target::SvgPatternCommand::SaveLayerFilter(filters.to_skia())
            }
            Self::SaveLayerReferenceFilter(filter) => {
                let [x, y, width, height] = filter
                    .reference
                    .expect("unresolved SVG pattern reference filter reached raster conversion");
                let graph = crate::svg_filter_graph::TranslateSvgFilterGraph(
                    &filter.graph,
                    paint::paint_engine::PaintRect {
                        x,
                        y,
                        width,
                        height,
                    },
                    1.0,
                )
                .unwrap_or_else(|error| {
                    panic!("unsupported SVG pattern reference filter: {error}")
                });
                target::SvgPatternCommand::SaveLayerReferenceFilter(std::sync::Arc::new(graph))
            }
            Self::SaveLayerBlend(mode) => target::SvgPatternCommand::SaveLayerBlend(mode.to_skia()),
            Self::SaveLayerMask(mask) => {
                let [x, y, width, height] = mask
                    .resolved_clip
                    .expect("unresolved SVG pattern mask reached raster conversion");
                target::SvgPatternCommand::SaveLayerMask(std::sync::Arc::new(
                    target::SvgPatternMask {
                        commands: mask.commands.to_skia(),
                        content_transform: mask.content_transform.to_skia(),
                        clip_rect: target::PaintRect {
                            x,
                            y,
                            width,
                            height,
                        },
                        mode: mask.mode.to_skia(),
                    },
                ))
            }
            Self::SaveLayerClip(_) => {
                panic!("unresolved SVG pattern clipPath reached raster conversion")
            }
            Self::Restore => target::SvgPatternCommand::Restore,
            Self::DrawShape(shape) => target::SvgPatternCommand::DrawShape(shape.to_skia()),
            Self::DrawGlyph(glyph) => target::SvgPatternCommand::DrawGlyph(glyph.to_skia()),
            Self::DrawText(_) => panic!("unresolved SVG pattern text reached raster conversion"),
            Self::DrawImage(image) => target::SvgPatternCommand::DrawImage(image.to_skia()),
        }
    }
}
impl ToSkia for layoutng_assembly::internal::paint_input::SvgPatternData {
    type Output = target::SvgPatternData;
    fn to_skia(&self) -> Self::Output {
        target::SvgPatternData {
            commands: self.commands.to_skia(),
            content_transform: self.content_transform.to_skia(),
            opacity: self.opacity,
        }
    }
}
impl ToSkia for layoutng_assembly::internal::paint_input::PaintShader {
    type Output = target::PaintShader;
    fn to_skia(&self) -> Self::Output {
        target::PaintShader {
            flags_alpha: 1.,
            kind: self.kind.to_skia(),
            apply_dark_mode: self.apply_dark_mode,
            spread: self.spread.to_skia(),
            stops: self.stops.to_skia(),
            start: self.start.to_skia(),
            end: self.end.to_skia(),
            linear_angle: self.linear_angle.to_skia(),
            linear_corner: self.linear_corner,
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
            color_interpolation_space: self.color_interpolation_space.to_skia(),
            hue_interpolation_method: self.hue_interpolation_method.to_skia(),
            transform: self.transform.to_skia(),
            pattern: self
                .pattern
                .as_ref()
                .map(|pattern| std::sync::Arc::new(pattern.as_ref().to_skia())),
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
impl ToSkia for layoutng_assembly::internal::paint_input::InterpolationQuality {
    type Output = target::ImageInterpolationQuality;
    fn to_skia(&self) -> Self::Output {
        match self {
            Self::kInterpolationNone => target::ImageInterpolationQuality::kNone,
            Self::kInterpolationLow => target::ImageInterpolationQuality::kLow,
            Self::kInterpolationMedium => target::ImageInterpolationQuality::kMedium,
        }
    }
}
impl ToSkia for foundation::DynamicRangeLimit {
    type Output = target::DynamicRangeLimit;
    fn to_skia(&self) -> Self::Output {
        // cc::PaintFlags stores each explicit mixture weight in seven bits.
        let quantize = |value: f32| ((0.5 + 127.0 * value) as u32) as f32 / 127.0;
        target::DynamicRangeLimit {
            standard_mix: quantize(self.standard_mix),
            constrained_high_mix: quantize(self.constrained_high_mix),
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
            Self::kCustomData => target::CommandKind::kCustomData,
        }
    }
}
impl ToSkia for image_resource::ImageOrientation {
    type Output = target::ImageOrientation;
    fn to_skia(&self) -> Self::Output {
        match self {
            Self::kDefault => target::ImageOrientation::kDefault,
            Self::kFlipHorizontal => target::ImageOrientation::kFlipHorizontal,
            Self::kRotate180 => target::ImageOrientation::kRotate180,
            Self::kFlipVertical => target::ImageOrientation::kFlipVertical,
            Self::kTranspose => target::ImageOrientation::kTranspose,
            Self::kRotate90 => target::ImageOrientation::kRotate90,
            Self::kTransverse => target::ImageOrientation::kTransverse,
            Self::kRotate270 => target::ImageOrientation::kRotate270,
        }
    }
}
impl ToSkia for paint::paint_engine::DisplayMaskLayer {
    type Output = target::MaskLayer;
    fn to_skia(&self) -> Self::Output {
        assert!(
            self.source_error.is_none(),
            "unsupported SVG mask resource: {:?}",
            self.source_error
        );
        target::MaskLayer {
            resource_id: self.resource_id.to_skia(),
            image_interpolation_quality: self.image_interpolation_quality.to_skia(),
            dynamic_range_limit: self.dynamic_range_limit.to_skia(),
            image_orientation: self.image_orientation.to_skia(),
            use_dark_mode_for_image: self.use_dark_mode_for_image,
            paint_shader: self.paint_shader.to_skia(),
            clip_rect: self.clip_rect.to_skia(),
            clip_radii: self.clip_radii.to_skia(),
            has_clip: self.has_clip.to_skia(),
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
            flags_alpha: 1.,
            r#type: self.r#type.to_skia(),
            rect: self.rect.to_skia(),
            source_rect: self.source_rect.to_skia(),
            inner_rect: self.inner_rect.to_skia(),
            tile_rect: self.tile_rect.to_skia(),
            color: self.color.to_skia(),
            line_style: self.line_style.to_skia(),
            decoration_style: self.decoration_style.to_skia(),
            stroke_width: self.stroke_width.to_skia(),
            corner_radius: self.corner_radius.to_skia(),
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
            apply_srgb_to_linear_color_filter: false,
            paint_shader: self.paint_shader.to_skia(),
            blend_mode: self.blend_mode.to_skia(),
            source_overwrite: self.source_overwrite,
            // Paint DisplayItems carry PrePaint's canonical filter value and
            // paint playback consumes it before command projection.
            filters: Vec::new(),
            mask_layers: self.mask_layers.to_skia(),
            round_cap: self.round_cap.to_skia(),
            resource_id: self.resource_id.to_skia(),
            image_interpolation_quality: self.image_interpolation_quality.to_skia(),
            dynamic_range_limit: self.dynamic_range_limit.to_skia(),
            image_orientation: self.image_orientation.to_skia(),
            use_dark_mode_for_image: self.use_dark_mode_for_image.to_skia(),
            repeat_x: self.repeat_x.to_skia(),
            repeat_y: self.repeat_y.to_skia(),
            tile_scale: self.tile_scale.to_skia(),
            tile_phase: self.tile_phase.to_skia(),
            tile_spacing: self.tile_spacing.to_skia(),
            shadow_offset: self.shadow_offset.to_skia(),
            blur_radius: self.blur_radius.to_skia(),
            spread: self.spread.to_skia(),
            inset: self.inset.to_skia(),
            shadow_has_opaque_background: self.shadow_has_opaque_background.to_skia(),
            is_text_decoration: self.is_text_decoration.to_skia(),
            stroke_glyphs: self.stroke_glyphs.to_skia(),
            decoration_thickness: self.decoration_thickness,
            skip_ink: self.skip_ink.to_skia(),
            decoration_ink_bounds: self.decoration_ink_bounds.to_skia(),
            decoration_ink_baseline: self.decoration_ink_baseline,
            decoration_text_origin: self.decoration_text_origin.to_skia(),
            decoration_text_fit_scale: self.decoration_text_fit_scale,
            decoration_ink_runs: self.decoration_ink_runs.as_ref().to_vec().to_skia(),
            opacity: self.opacity.to_skia(),
            transform: self.transform.to_skia(),
            font_face_index: self.font_face_index.to_skia(),
            font_variations: self.font_variations.to_skia(),
            font_size: self.font_size.to_skia(),
            text_line_top: self.text_line_top.to_skia(),
            text_line_height: self.text_line_height.to_skia(),
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
                    resolved_family: &f.resolved_family,
                    weight: f.weight,
                    italic: f.italic,
                    bytes: &f.bytes,
                    bytes_owner: Some(std::sync::Arc::new(f.bytes.clone())),
                    face_index: f.face_index,
                    variations: f.variations.to_skia(),
                })
                .collect(),
            images: r
                .images
                .iter()
                .filter_map(|i| {
                    i.BitmapPixels().map(|pixels| {
                        let alpha_type = match i.alpha_type {
                            image_resource::PaintImageAlphaType::Unpremultiplied => {
                                target::ImageAlphaType::Unpremultiplied
                            }
                            image_resource::PaintImageAlphaType::Premultiplied => {
                                target::ImageAlphaType::Premultiplied
                            }
                        };
                        target::Image::shared_with_alpha_type(
                            i.id, i.width, i.height, alpha_type, pixels,
                        )
                    })
                })
                .chain(r.canvases.iter().map(|canvas| {
                    // CanvasRenderingContext publishes premultiplied front-
                    // buffer RGBA8, matching StaticBitmapImage's paint path.
                    target::Image::shared_with_alpha_type(
                        canvas.resource_id,
                        canvas.width,
                        canvas.height,
                        target::ImageAlphaType::Premultiplied,
                        &canvas.pixels,
                    )
                }))
                .collect(),
        }),
    }
}

#[cfg(test)]
mod font_identity_tests {
    use super::*;
    use foundation::ETextDecorationSkipInk;
    use layoutng_assembly::fragment_tree::{PaintGlyph, PaintGlyphRun, PaintResources};
    use layoutng_assembly::internal::layout_input::{FontFace, FontVariation, Offset};
    use paint::paint_engine::{DisplayItem, DisplayItemType, PaintArtifact};
    use std::sync::Arc;

    #[test]
    fn paint_to_raster_command_preserves_image_decode_flags() {
        let command = DisplayItem {
            r#type: DisplayItemType::kDrawImageRect,
            use_dark_mode_for_image: true,
            dynamic_range_limit: foundation::DynamicRangeLimit::from_mix(0.25, 0.5),
            ..Default::default()
        }
        .to_skia();
        assert!(command.use_dark_mode_for_image);
        assert_eq!(command.dynamic_range_limit.standard_mix, 32.0 / 127.0);
        assert_eq!(
            command.dynamic_range_limit.constrained_high_mix,
            64.0 / 127.0
        );

        let mask = paint::paint_engine::DisplayMaskLayer {
            use_dark_mode_for_image: true,
            dynamic_range_limit: foundation::DynamicRangeLimit::from_mix(0.75, 0.125),
            ..Default::default()
        }
        .to_skia();
        assert!(mask.use_dark_mode_for_image);
        assert_eq!(mask.dynamic_range_limit.standard_mix, 95.0 / 127.0);
        assert_eq!(mask.dynamic_range_limit.constrained_high_mix, 16.0 / 127.0);

        let mut target = compositor_core::runtime::paint::TargetColorParams::default();
        target.hdr_headroom = Some(4.0);
        let limited = compositor_core::runtime::paint::LimitTargetDynamicRange(
            compositor_core::runtime::paint::CpuDynamicRangeLimit::new(0.25, 0.5),
            target,
        );
        assert_eq!(limited.hdr_headroom, Some(1.5));
    }

    #[test]
    fn paint_to_raster_command_preserves_source_replacement() {
        let command = DisplayItem {
            r#type: DisplayItemType::kDrawRect,
            source_overwrite: true,
            ..Default::default()
        }
        .to_skia();
        assert!(command.source_overwrite);
    }

    #[test]
    fn paint_to_skia_preserves_exact_face_and_glyph_identity() {
        let font = FontFace {
            family: "CSS alias".into(),
            resolved_family: "metadata must not select a face".into(),
            bytes: Vec::from(b"selected immutable font data".as_slice()).into(),
            face_index: 0x20003,
            variations: vec![FontVariation {
                tag: u32::from_be_bytes(*b"wght"),
                value: 550.0,
            }],
            ..Default::default()
        };
        let source_bytes = font.bytes.clone();
        let item = DisplayItem {
            r#type: DisplayItemType::kDrawGlyphRun,
            font_face_index: 0,
            font_variations: vec![FontVariation {
                tag: u32::from_be_bytes(*b"opsz"),
                value: 12.0,
            }],
            synthetic_bold: true,
            synthetic_italic: true,
            glyphs: std::sync::Arc::new(vec![PaintGlyph {
                id: 513,
                character_index: 4,
                offset: Offset { x: 2.5, y: -1.0 },
                ..Default::default()
            }]),
            ..Default::default()
        };
        let artifact = PaintArtifact {
            resources: Some(Arc::new(PaintResources {
                fonts: vec![font].into(),
                ..Default::default()
            })),
            items: Arc::new(vec![item]),
            ..Default::default()
        };
        let converted = resources(&artifact);
        let face = &converted.resources.as_ref().unwrap().fonts[0];
        assert_eq!(face.bytes, source_bytes.as_ref());
        let retained = face.bytes_owner.as_ref().expect("production font owner");
        let view: &[u8] = retained.as_ref().as_ref();
        assert_eq!(view.as_ptr(), source_bytes.as_ptr());
        assert_eq!(view.len(), source_bytes.len());
        assert_eq!(
            face.bytes.as_ptr(),
            source_bytes.as_ptr(),
            "do not copy the font snapshot"
        );
        assert_eq!(face.face_index, 0x20003);
        assert_eq!(face.variations[0].tag, u32::from_be_bytes(*b"wght"));
        assert_eq!(face.variations[0].value, 550.0);
        let command = artifact.items[0].to_skia();
        assert_eq!(command.font_face_index, 0);
        assert_eq!(command.font_variations[0].tag, u32::from_be_bytes(*b"opsz"));
        assert_eq!(command.font_variations[0].value, 12.0);
        assert_eq!(command.glyphs[0].id, 513);
        assert_eq!(command.glyphs[0].character_index, 4);
        assert_eq!(command.glyphs[0].offset.x, 2.5);
        assert_eq!(command.glyphs[0].offset.y, -1.0);
        assert!(command.synthetic_bold && command.synthetic_italic);
    }

    #[test]
    fn paint_to_skia_preserves_decoration_ink_inputs_and_fallback_runs() {
        let fallback_runs = Arc::new(vec![
            PaintGlyphRun {
                font_face_index: 2,
                font_size: 18.0,
                baseline: 14.0,
                glyphs: Arc::new(vec![PaintGlyph {
                    id: 84,
                    character_index: 3,
                    offset: Offset { x: 7.0, y: 0.0 },
                    ..Default::default()
                }]),
                ..Default::default()
            },
            PaintGlyphRun {
                font_face_index: 5,
                font_size: 19.0,
                baseline: 14.0,
                synthetic_italic: true,
                glyphs: Arc::new(vec![PaintGlyph {
                    id: 91,
                    character_index: 4,
                    offset: Offset { x: 17.5, y: -0.25 },
                    ..Default::default()
                }]),
                ..Default::default()
            },
        ]);
        let command = DisplayItem {
            is_text_decoration: true,
            decoration_thickness: 2.25,
            skip_ink: ETextDecorationSkipInk::kAll,
            decoration_ink_bounds: Some(paint::paint_engine::PaintRect {
                x: 4.0,
                y: 31.0,
                width: 80.0,
                height: 3.0,
            }),
            decoration_ink_baseline: 28.0,
            decoration_text_origin: Offset { x: 4.0, y: 14.0 },
            decoration_text_fit_scale: 0.875,
            decoration_ink_runs: fallback_runs,
            ..Default::default()
        }
        .to_skia();

        assert_eq!(command.skip_ink, target::TextDecorationSkipInk::kAll);
        assert_eq!(command.decoration_thickness, 2.25);
        assert_eq!(command.decoration_ink_bounds.unwrap().y, 31.0);
        assert_eq!(command.decoration_ink_baseline, 28.0);
        assert_eq!(command.decoration_text_origin.x, 4.0);
        assert_eq!(command.decoration_text_fit_scale, 0.875);
        assert_eq!(command.decoration_ink_runs.len(), 2);
        assert_eq!(command.decoration_ink_runs[0].font_face_index, 2);
        assert_eq!(command.decoration_ink_runs[0].glyphs[0].id, 84);
        assert_eq!(command.decoration_ink_runs[1].font_face_index, 5);
        assert!(command.decoration_ink_runs[1].synthetic_italic);
        assert_eq!(command.decoration_ink_runs[1].glyphs[0].offset.x, 17.5);
    }

    #[test]
    fn font_data_replacement_keeps_old_paint_snapshot_independent() {
        let old = PaintArtifact {
            resources: Some(Arc::new(PaintResources {
                fonts: vec![FontFace {
                    family: "same family".into(),
                    bytes: Vec::from(b"old font data".as_slice()).into(),
                    ..Default::default()
                }]
                .into(),
                ..Default::default()
            })),
            ..Default::default()
        };
        let mut new = old.clone();
        Arc::make_mut(new.resources.as_mut().unwrap()).fonts[0].bytes =
            Vec::from(b"new font data".as_slice()).into();
        let old_skia = resources(&old);
        let new_skia = resources(&new);
        let old_face = &old_skia.resources.as_ref().unwrap().fonts[0];
        let new_face = &new_skia.resources.as_ref().unwrap().fonts[0];
        assert_eq!(old_face.family, new_face.family);
        assert_eq!(old_face.bytes, b"old font data");
        assert_eq!(new_face.bytes, b"new font data");
        assert_ne!(old_face.bytes.as_ptr(), new_face.bytes.as_ptr());
    }

    fn render_selected_source(bytes: font_engine::SharedFontBytes, face_index: u32) -> Vec<u8> {
        use font_engine::{OpenTypeFont, TextDirection};
        let font = OpenTypeFont::new_shared(bytes.clone(), face_index, 24.0, &[]);
        let shaped = font.ShapeRun("Exact face", TextDirection::kLtr, "en", "Latn");
        assert!(!shaped.glyphs.is_empty());
        assert!(shaped.glyphs.iter().all(|glyph| glyph.id != 0));
        let mut x = 0.0;
        let glyphs = Arc::new(
            shaped
                .glyphs
                .iter()
                .map(|glyph| {
                    let painted = PaintGlyph {
                        id: glyph.id,
                        character_index: glyph.cluster,
                        offset: Offset {
                            x: x + glyph.x_offset,
                            y: -glyph.y_offset,
                        },
                        ..Default::default()
                    };
                    x += glyph.x_advance;
                    painted
                })
                .collect(),
        );
        let artifact = PaintArtifact {
            resources: Some(Arc::new(PaintResources {
                fonts: vec![FontFace {
                    family: "CSS alias".into(),
                    resolved_family: "a nonexistent platform font family 314159".into(),
                    bytes,
                    face_index,
                    ..Default::default()
                }]
                .into(),
                ..Default::default()
            })),
            items: Arc::new(vec![DisplayItem {
                r#type: DisplayItemType::kDrawGlyphRun,
                font_size: 24.0,
                color: layoutng_assembly::internal::layout_input_types::Color {
                    red: 0.0,
                    green: 0.0,
                    blue: 0.0,
                    alpha: 1.0,
                },
                text_blob_origin: Offset { x: 5.0, y: 32.0 },
                glyphs,
                ..Default::default()
            }]),
            ..Default::default()
        };
        let render = |artifact: &PaintArtifact| crate::RasterizePaintArtifact(artifact, 180, 48);
        let mislabeled = render(&artifact);
        assert!(mislabeled
            .chunks_exact(4)
            .any(|pixel| pixel != [255, 255, 255, 255]));
        let mut without_metadata = artifact.clone();
        let face = &mut Arc::make_mut(without_metadata.resources.as_mut().unwrap()).fonts[0];
        face.resolved_family.clear();
        face.family.clear();
        assert_eq!(mislabeled, render(&without_metadata));
        mislabeled
    }

    #[test]
    fn shaped_glyphs_render_selected_bytes_even_with_unresolvable_family_metadata() {
        render_selected_source(
            include_bytes!("../../../assets/fonts/Roboto-Regular.ttf").into(),
            0,
        );
    }

    #[test]
    fn shaped_collection_face_rasters_like_the_same_standalone_face() {
        let reference = include_bytes!("../../../assets/fonts/Roboto-Regular.ttf");
        let other = include_bytes!("../../../assets/fonts/Roboto-Regular.ttf");
        let mut collection = Vec::from(b"ttcf".as_slice());
        collection.extend_from_slice(&0x00010000u32.to_be_bytes());
        collection.extend_from_slice(&2u32.to_be_bytes());
        collection.extend_from_slice(&[0; 8]);
        for (index, source) in [other.as_slice(), reference.as_slice()]
            .into_iter()
            .enumerate()
        {
            while collection.len() % 4 != 0 {
                collection.push(0);
            }
            let start = collection.len();
            collection[12 + index * 4..16 + index * 4]
                .copy_from_slice(&(start as u32).to_be_bytes());
            collection.extend_from_slice(source);
            let tables = u16::from_be_bytes(source[4..6].try_into().unwrap()) as usize;
            for table in 0..tables {
                let at = start + 12 + table * 16 + 8;
                let offset = u32::from_be_bytes(collection[at..at + 4].try_into().unwrap());
                collection[at..at + 4].copy_from_slice(&(offset + start as u32).to_be_bytes());
            }
        }
        let selected = render_selected_source(collection.into(), 1);
        assert_eq!(selected, render_selected_source(reference.into(), 0));
    }
}

#[cfg(test)]
mod canvas_resource_tests {
    use super::*;
    use canvas::CanvasPaintResource;
    use layoutng_assembly::fragment_tree::PaintResources;
    use paint::paint_engine::PaintArtifact;
    use std::sync::Arc;

    #[test]
    fn canvas_snapshot_enters_raster_catalog_as_premultiplied_image() {
        let pixels = Arc::new(vec![10, 20, 30, 40]);
        let artifact = PaintArtifact {
            resources: Some(Arc::new(PaintResources {
                canvases: vec![CanvasPaintResource {
                    dom_node_id: 9,
                    resource_id: 1 << 63,
                    revision: 4,
                    width: 1,
                    height: 1,
                    origin_clean: true,
                    is_clear: false,
                    pixels: pixels.clone(),
                }],
                ..Default::default()
            })),
            ..Default::default()
        };

        let converted = resources(&artifact);
        let image = &converted.resources.unwrap().images[0];
        assert_eq!(image.id, 1 << 63);
        assert_eq!(image.alpha_type, target::ImageAlphaType::Premultiplied);
        assert!(Arc::ptr_eq(image.shared_pixels().unwrap(), &pixels));
    }
}
