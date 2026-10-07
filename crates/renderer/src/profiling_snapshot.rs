//! Diagnostic, versioned serialization of exact CPU command/resource inputs.
//! Local tooling only: not a Skia source translation or a production frame cache.
use skia::compat::commands::*;
use std::io::{self, Read, Write};
trait Codec: Sized {
    fn save(&self, w: &mut impl Write) -> io::Result<()>;
    fn load(r: &mut impl Read) -> io::Result<Self>;
}
fn save_bytes(bytes: &[u8], w: &mut impl Write) -> io::Result<()> {
    (bytes.len() as u64).save(w)?;
    w.write_all(bytes)
}
fn load_bytes(r: &mut impl Read) -> io::Result<Vec<u8>> {
    let count = u64::load(r)?;
    if count > 512 * 1024 * 1024 {
        return Err(io::Error::other("oversized resource"));
    }
    let mut bytes = vec![0; count as usize];
    r.read_exact(&mut bytes)?;
    Ok(bytes)
}
fn catalog_count(r: &mut impl Read) -> io::Result<u64> {
    let n = u64::load(r)?;
    if n > 4096 {
        return Err(io::Error::other("oversized resource catalog"));
    }
    Ok(n)
}
macro_rules! scalar { ($($t:ty),*) => { $(impl Codec for $t {
 fn save(&self,w:&mut impl Write)->io::Result<()> { w.write_all(&self.to_le_bytes()) }
 fn load(r:&mut impl Read)->io::Result<Self> { let mut bytes=[0;std::mem::size_of::<Self>()]; r.read_exact(&mut bytes)?; Ok(Self::from_le_bytes(bytes)) }
})* }; }
scalar!(u8, u32, u64, f32, f64);
impl Codec for bool {
    fn save(&self, w: &mut impl Write) -> io::Result<()> {
        (*self as u8).save(w)
    }
    fn load(r: &mut impl Read) -> io::Result<Self> {
        match u8::load(r)? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(io::Error::other("invalid bool")),
        }
    }
}
impl<T: Codec> Codec for Vec<T> {
    fn save(&self, w: &mut impl Write) -> io::Result<()> {
        (self.len() as u64).save(w)?;
        for v in self {
            v.save(w)?;
        }
        Ok(())
    }
    fn load(r: &mut impl Read) -> io::Result<Self> {
        let n = u64::load(r)?;
        if n > 1_000_000 {
            return Err(io::Error::other("oversized snapshot vector"));
        }
        (0..n).map(|_| T::load(r)).collect()
    }
}
impl<T: Codec> Codec for Option<T> {
    fn save(&self, w: &mut impl Write) -> io::Result<()> {
        self.is_some().save(w)?;
        if let Some(v) = self {
            v.save(w)?;
        }
        Ok(())
    }
    fn load(r: &mut impl Read) -> io::Result<Self> {
        if bool::load(r)? {
            Ok(Some(T::load(r)?))
        } else {
            Ok(None)
        }
    }
}
impl Codec for [f64; 16] {
    fn save(&self, w: &mut impl Write) -> io::Result<()> {
        for v in self {
            v.save(w)?;
        }
        Ok(())
    }
    fn load(r: &mut impl Read) -> io::Result<Self> {
        let mut a = [0.; 16];
        for v in &mut a {
            *v = f64::load(r)?;
        }
        Ok(a)
    }
}
impl Codec for String {
    fn save(&self, w: &mut impl Write) -> io::Result<()> {
        self.as_bytes().to_vec().save(w)
    }
    fn load(r: &mut impl Read) -> io::Result<Self> {
        String::from_utf8(Vec::<u8>::load(r)?).map_err(io::Error::other)
    }
}
impl Codec for PaintPathVerb {
    fn save(&self, w: &mut impl Write) -> io::Result<()> {
        (*self as u32).save(w)
    }
    fn load(r: &mut impl Read) -> io::Result<Self> {
        match u32::load(r)? {
            0 => Ok(Self::kMoveTo),
            1 => Ok(Self::kLineTo),
            2 => Ok(Self::kQuadraticTo),
            3 => Ok(Self::kConicTo),
            4 => Ok(Self::kCubicTo),
            5 => Ok(Self::kClose),
            _ => Err(io::Error::other("unknown snapshot enum")),
        }
    }
}
impl Codec for BorderLineStyle {
    fn save(&self, w: &mut impl Write) -> io::Result<()> {
        (*self as u32).save(w)
    }
    fn load(r: &mut impl Read) -> io::Result<Self> {
        match u32::load(r)? {
            0 => Ok(Self::kNone),
            1 => Ok(Self::kHidden),
            2 => Ok(Self::kSolid),
            3 => Ok(Self::kDashed),
            4 => Ok(Self::kDotted),
            5 => Ok(Self::kDouble),
            6 => Ok(Self::kGroove),
            7 => Ok(Self::kRidge),
            8 => Ok(Self::kInset),
            9 => Ok(Self::kOutset),
            _ => Err(io::Error::other("unknown snapshot enum")),
        }
    }
}
impl Codec for TextDecorationStyle {
    fn save(&self, w: &mut impl Write) -> io::Result<()> {
        (*self as u32).save(w)
    }
    fn load(r: &mut impl Read) -> io::Result<Self> {
        match u32::load(r)? {
            0 => Ok(Self::kSolid),
            1 => Ok(Self::kDouble),
            2 => Ok(Self::kDotted),
            3 => Ok(Self::kDashed),
            4 => Ok(Self::kWavy),
            _ => Err(io::Error::other("unknown snapshot enum")),
        }
    }
}
impl Codec for FontSmoothing {
    fn save(&self, w: &mut impl Write) -> io::Result<()> {
        (*self as u32).save(w)
    }
    fn load(r: &mut impl Read) -> io::Result<Self> {
        match u32::load(r)? {
            0 => Ok(Self::kAuto),
            1 => Ok(Self::kNone),
            2 => Ok(Self::kAntialiased),
            3 => Ok(Self::kSubpixelAntialiased),
            _ => Err(io::Error::other("unknown snapshot enum")),
        }
    }
}
impl Codec for PaintBlendMode {
    fn save(&self, w: &mut impl Write) -> io::Result<()> {
        (*self as u32).save(w)
    }
    fn load(r: &mut impl Read) -> io::Result<Self> {
        match u32::load(r)? {
            0 => Ok(Self::kNormal),
            1 => Ok(Self::kMultiply),
            2 => Ok(Self::kScreen),
            3 => Ok(Self::kOverlay),
            4 => Ok(Self::kDarken),
            5 => Ok(Self::kLighten),
            6 => Ok(Self::kColorDodge),
            7 => Ok(Self::kColorBurn),
            8 => Ok(Self::kHardLight),
            9 => Ok(Self::kSoftLight),
            10 => Ok(Self::kDifference),
            11 => Ok(Self::kExclusion),
            12 => Ok(Self::kHue),
            13 => Ok(Self::kSaturation),
            14 => Ok(Self::kColor),
            15 => Ok(Self::kLuminosity),
            16 => Ok(Self::kPlusLighter),
            _ => Err(io::Error::other("unknown snapshot enum")),
        }
    }
}
impl Codec for PaintFilterType {
    fn save(&self, w: &mut impl Write) -> io::Result<()> {
        (*self as u32).save(w)
    }
    fn load(r: &mut impl Read) -> io::Result<Self> {
        match u32::load(r)? {
            0 => Ok(Self::kBlur),
            1 => Ok(Self::kBrightness),
            2 => Ok(Self::kContrast),
            3 => Ok(Self::kGrayscale),
            4 => Ok(Self::kHueRotate),
            5 => Ok(Self::kInvert),
            6 => Ok(Self::kOpacity),
            7 => Ok(Self::kSaturate),
            8 => Ok(Self::kSepia),
            9 => Ok(Self::kDropShadow),
            _ => Err(io::Error::other("unknown snapshot enum")),
        }
    }
}
impl Codec for SvgStrokeLineCap {
    fn save(&self, w: &mut impl Write) -> io::Result<()> {
        (*self as u32).save(w)
    }
    fn load(r: &mut impl Read) -> io::Result<Self> {
        match u32::load(r)? {
            0 => Ok(Self::kButt),
            1 => Ok(Self::kRound),
            2 => Ok(Self::kSquare),
            _ => Err(io::Error::other("unknown snapshot enum")),
        }
    }
}
impl Codec for SvgStrokeLineJoin {
    fn save(&self, w: &mut impl Write) -> io::Result<()> {
        (*self as u32).save(w)
    }
    fn load(r: &mut impl Read) -> io::Result<Self> {
        match u32::load(r)? {
            0 => Ok(Self::kMiter),
            1 => Ok(Self::kRound),
            2 => Ok(Self::kBevel),
            _ => Err(io::Error::other("unknown snapshot enum")),
        }
    }
}
impl Codec for PaintShaderKind {
    fn save(&self, w: &mut impl Write) -> io::Result<()> {
        (*self as u32).save(w)
    }
    fn load(r: &mut impl Read) -> io::Result<Self> {
        match u32::load(r)? {
            0 => Ok(Self::kLinearGradient),
            1 => Ok(Self::kRadialGradient),
            2 => Ok(Self::kConicGradient),
            3 => Ok(Self::kPattern),
            _ => Err(io::Error::other("unknown snapshot enum")),
        }
    }
}
impl Codec for PaintSpreadMethod {
    fn save(&self, w: &mut impl Write) -> io::Result<()> {
        (*self as u32).save(w)
    }
    fn load(r: &mut impl Read) -> io::Result<Self> {
        match u32::load(r)? {
            0 => Ok(Self::kPad),
            1 => Ok(Self::kReflect),
            2 => Ok(Self::kRepeat),
            _ => Err(io::Error::other("unknown snapshot enum")),
        }
    }
}
impl Codec for PaintRadialShape {
    fn save(&self, w: &mut impl Write) -> io::Result<()> {
        (*self as u32).save(w)
    }
    fn load(r: &mut impl Read) -> io::Result<Self> {
        match u32::load(r)? {
            0 => Ok(Self::kCircle),
            1 => Ok(Self::kEllipse),
            _ => Err(io::Error::other("unknown snapshot enum")),
        }
    }
}
impl Codec for PaintRadialExtent {
    fn save(&self, w: &mut impl Write) -> io::Result<()> {
        (*self as u32).save(w)
    }
    fn load(r: &mut impl Read) -> io::Result<Self> {
        match u32::load(r)? {
            0 => Ok(Self::kExplicit),
            1 => Ok(Self::kClosestSide),
            2 => Ok(Self::kClosestCorner),
            3 => Ok(Self::kFarthestSide),
            4 => Ok(Self::kFarthestCorner),
            _ => Err(io::Error::other("unknown snapshot enum")),
        }
    }
}
impl Codec for BackgroundRepeatRule {
    fn save(&self, w: &mut impl Write) -> io::Result<()> {
        (*self as u32).save(w)
    }
    fn load(r: &mut impl Read) -> io::Result<Self> {
        match u32::load(r)? {
            0 => Ok(Self::kRepeat),
            1 => Ok(Self::kNoRepeat),
            2 => Ok(Self::kRound),
            3 => Ok(Self::kSpace),
            _ => Err(io::Error::other("unknown snapshot enum")),
        }
    }
}
impl Codec for PaintMaskMode {
    fn save(&self, w: &mut impl Write) -> io::Result<()> {
        (*self as u32).save(w)
    }
    fn load(r: &mut impl Read) -> io::Result<Self> {
        match u32::load(r)? {
            0 => Ok(Self::kAlpha),
            1 => Ok(Self::kLuminance),
            _ => Err(io::Error::other("unknown snapshot enum")),
        }
    }
}
impl Codec for PaintMaskComposite {
    fn save(&self, w: &mut impl Write) -> io::Result<()> {
        (*self as u32).save(w)
    }
    fn load(r: &mut impl Read) -> io::Result<Self> {
        match u32::load(r)? {
            0 => Ok(Self::kAdd),
            1 => Ok(Self::kSubtract),
            2 => Ok(Self::kIntersect),
            3 => Ok(Self::kExclude),
            _ => Err(io::Error::other("unknown snapshot enum")),
        }
    }
}
impl Codec for CommandKind {
    fn save(&self, w: &mut impl Write) -> io::Result<()> {
        (*self as u32).save(w)
    }
    fn load(r: &mut impl Read) -> io::Result<Self> {
        match u32::load(r)? {
            0 => Ok(Self::kSave),
            1 => Ok(Self::kRestore),
            2 => Ok(Self::kConcat),
            3 => Ok(Self::kClipRect),
            4 => Ok(Self::kClipRoundedRect),
            5 => Ok(Self::kClipOutRoundedRect),
            6 => Ok(Self::kClipPath),
            7 => Ok(Self::kClipOutRect),
            8 => Ok(Self::kSaveLayer),
            9 => Ok(Self::kSaveLayerAlpha),
            10 => Ok(Self::kSaveLayerBlend),
            11 => Ok(Self::kSaveLayerFilter),
            12 => Ok(Self::kBeginMask),
            13 => Ok(Self::kEndMask),
            14 => Ok(Self::kDrawRect),
            15 => Ok(Self::kDrawRoundedRect),
            16 => Ok(Self::kDrawDoubleRoundedRect),
            17 => Ok(Self::kDrawEllipse),
            18 => Ok(Self::kStrokeEllipse),
            19 => Ok(Self::kDrawPath),
            20 => Ok(Self::kStrokePath),
            21 => Ok(Self::kDrawGradientRect),
            22 => Ok(Self::kDrawTiledGradient),
            23 => Ok(Self::kStrokeRect),
            24 => Ok(Self::kStrokeLine),
            25 => Ok(Self::kStrokeWavyLine),
            26 => Ok(Self::kDrawGlyphRun),
            27 => Ok(Self::kDrawImageRect),
            28 => Ok(Self::kDrawTiledImage),
            29 => Ok(Self::kDrawBoxShadow),
            30 => Ok(Self::kDrawScrollbarTrack),
            31 => Ok(Self::kDrawScrollbarThumb),
            32 => Ok(Self::kDrawScrollbarButton),
            33 => Ok(Self::kDrawScrollbarCorner),
            _ => Err(io::Error::other("unknown snapshot enum")),
        }
    }
}
impl Codec for Color {
    fn save(&self, w: &mut impl Write) -> io::Result<()> {
        self.red.save(w)?;
        self.green.save(w)?;
        self.blue.save(w)?;
        self.alpha.save(w)?;
        Ok(())
    }
    fn load(r: &mut impl Read) -> io::Result<Self> {
        Ok(Self {
            red: <f32>::load(r)?,
            green: <f32>::load(r)?,
            blue: <f32>::load(r)?,
            alpha: <f32>::load(r)?,
        })
    }
}
impl Codec for Offset {
    fn save(&self, w: &mut impl Write) -> io::Result<()> {
        self.x.save(w)?;
        self.y.save(w)?;
        Ok(())
    }
    fn load(r: &mut impl Read) -> io::Result<Self> {
        Ok(Self {
            x: <f64>::load(r)?,
            y: <f64>::load(r)?,
        })
    }
}
impl Codec for Size {
    fn save(&self, w: &mut impl Write) -> io::Result<()> {
        self.width.save(w)?;
        self.height.save(w)?;
        Ok(())
    }
    fn load(r: &mut impl Read) -> io::Result<Self> {
        Ok(Self {
            width: <f64>::load(r)?,
            height: <f64>::load(r)?,
        })
    }
}
impl Codec for TransformMatrix {
    fn save(&self, w: &mut impl Write) -> io::Result<()> {
        self.values.save(w)?;
        Ok(())
    }
    fn load(r: &mut impl Read) -> io::Result<Self> {
        Ok(Self {
            values: <[f64; 16]>::load(r)?,
        })
    }
}
impl Codec for PaintPathCommand {
    fn save(&self, w: &mut impl Write) -> io::Result<()> {
        self.verb.save(w)?;
        self.control1.save(w)?;
        self.control2.save(w)?;
        self.point.save(w)?;
        self.conic_weight.save(w)?;
        Ok(())
    }
    fn load(r: &mut impl Read) -> io::Result<Self> {
        Ok(Self {
            verb: <PaintPathVerb>::load(r)?,
            control1: <Offset>::load(r)?,
            control2: <Offset>::load(r)?,
            point: <Offset>::load(r)?,
            conic_weight: <f64>::load(r)?,
        })
    }
}
impl Codec for FontVariation {
    fn save(&self, w: &mut impl Write) -> io::Result<()> {
        self.tag.save(w)?;
        self.value.save(w)?;
        Ok(())
    }
    fn load(r: &mut impl Read) -> io::Result<Self> {
        Ok(Self {
            tag: <u32>::load(r)?,
            value: <f32>::load(r)?,
        })
    }
}
impl Codec for PaintGlyph {
    fn save(&self, w: &mut impl Write) -> io::Result<()> {
        self.id.save(w)?;
        self.character_index.save(w)?;
        self.canvas_rotation.save(w)?;
        self.offset.save(w)?;
        self.advance.save(w)?;
        Ok(())
    }
    fn load(r: &mut impl Read) -> io::Result<Self> {
        Ok(Self {
            id: <u32>::load(r)?,
            character_index: <u32>::load(r)?,
            canvas_rotation: <u8>::load(r)?,
            offset: <Offset>::load(r)?,
            advance: <f64>::load(r)?,
        })
    }
}
impl Codec for PaintCornerRadius {
    fn save(&self, w: &mut impl Write) -> io::Result<()> {
        self.x.save(w)?;
        self.y.save(w)?;
        Ok(())
    }
    fn load(r: &mut impl Read) -> io::Result<Self> {
        Ok(Self {
            x: <f64>::load(r)?,
            y: <f64>::load(r)?,
        })
    }
}
impl Codec for PaintCornerRadii {
    fn save(&self, w: &mut impl Write) -> io::Result<()> {
        self.top_left.save(w)?;
        self.top_right.save(w)?;
        self.bottom_right.save(w)?;
        self.bottom_left.save(w)?;
        Ok(())
    }
    fn load(r: &mut impl Read) -> io::Result<Self> {
        Ok(Self {
            top_left: <PaintCornerRadius>::load(r)?,
            top_right: <PaintCornerRadius>::load(r)?,
            bottom_right: <PaintCornerRadius>::load(r)?,
            bottom_left: <PaintCornerRadius>::load(r)?,
        })
    }
}
impl Codec for PaintFilterOperation {
    fn save(&self, w: &mut impl Write) -> io::Result<()> {
        self.r#type.save(w)?;
        self.amount.save(w)?;
        self.offset.save(w)?;
        self.blur_radius.save(w)?;
        self.color.save(w)?;
        Ok(())
    }
    fn load(r: &mut impl Read) -> io::Result<Self> {
        Ok(Self {
            r#type: <PaintFilterType>::load(r)?,
            amount: <f64>::load(r)?,
            offset: <Offset>::load(r)?,
            blur_radius: <f64>::load(r)?,
            color: <Color>::load(r)?,
        })
    }
}
impl Codec for PaintColorStop {
    fn save(&self, w: &mut impl Write) -> io::Result<()> {
        self.offset.save(w)?;
        self.color.save(w)?;
        self.offset_length.save(w)?;
        Ok(())
    }
    fn load(r: &mut impl Read) -> io::Result<Self> {
        Ok(Self {
            offset: <f64>::load(r)?,
            color: <Color>::load(r)?,
            offset_length: <f64>::load(r)?,
        })
    }
}
impl Codec for PaintShader {
    fn save(&self, w: &mut impl Write) -> io::Result<()> {
        self.kind.save(w)?;
        self.spread.save(w)?;
        self.stops.save(w)?;
        self.start.save(w)?;
        self.end.save(w)?;
        self.linear_angle.save(w)?;
        self.linear_start_offset.save(w)?;
        self.linear_end_offset.save(w)?;
        self.center.save(w)?;
        self.center_offset.save(w)?;
        self.focal.save(w)?;
        self.radius.save(w)?;
        self.radius_y.save(w)?;
        self.radius_offset.save(w)?;
        self.radius_y_offset.save(w)?;
        self.radial_shape.save(w)?;
        self.radial_extent.save(w)?;
        self.radial_start_offset.save(w)?;
        self.radial_end_offset.save(w)?;
        self.focal_radius.save(w)?;
        self.start_angle.save(w)?;
        self.end_angle.save(w)?;
        self.rotation_angle.save(w)?;
        self.resource_id.save(w)?;
        self.tile_offset.save(w)?;
        self.tile_width.save(w)?;
        self.tile_height.save(w)?;
        self.unit_coordinates.save(w)?;
        self.object_bounding_box_coordinates.save(w)?;
        self.interpolate_premultiplied.save(w)?;
        self.transform.save(w)?;
        Ok(())
    }
    fn load(r: &mut impl Read) -> io::Result<Self> {
        Ok(Self {
            kind: <PaintShaderKind>::load(r)?,
            spread: <PaintSpreadMethod>::load(r)?,
            stops: <Vec<PaintColorStop>>::load(r)?,
            start: <Offset>::load(r)?,
            end: <Offset>::load(r)?,
            linear_angle: <Option<f64>>::load(r)?,
            linear_start_offset: <f64>::load(r)?,
            linear_end_offset: <f64>::load(r)?,
            center: <Offset>::load(r)?,
            center_offset: <Offset>::load(r)?,
            focal: <Offset>::load(r)?,
            radius: <f64>::load(r)?,
            radius_y: <Option<f64>>::load(r)?,
            radius_offset: <f64>::load(r)?,
            radius_y_offset: <f64>::load(r)?,
            radial_shape: <PaintRadialShape>::load(r)?,
            radial_extent: <PaintRadialExtent>::load(r)?,
            radial_start_offset: <f64>::load(r)?,
            radial_end_offset: <f64>::load(r)?,
            focal_radius: <f64>::load(r)?,
            start_angle: <f64>::load(r)?,
            end_angle: <f64>::load(r)?,
            rotation_angle: <f64>::load(r)?,
            resource_id: <u64>::load(r)?,
            tile_offset: <Offset>::load(r)?,
            tile_width: <f64>::load(r)?,
            tile_height: <f64>::load(r)?,
            unit_coordinates: <bool>::load(r)?,
            object_bounding_box_coordinates: <bool>::load(r)?,
            interpolate_premultiplied: <bool>::load(r)?,
            transform: <TransformMatrix>::load(r)?,
        })
    }
}
impl Codec for PaintRect {
    fn save(&self, w: &mut impl Write) -> io::Result<()> {
        self.x.save(w)?;
        self.y.save(w)?;
        self.width.save(w)?;
        self.height.save(w)?;
        Ok(())
    }
    fn load(r: &mut impl Read) -> io::Result<Self> {
        Ok(Self {
            x: <f64>::load(r)?,
            y: <f64>::load(r)?,
            width: <f64>::load(r)?,
            height: <f64>::load(r)?,
        })
    }
}
impl Codec for MaskLayer {
    fn save(&self, w: &mut impl Write) -> io::Result<()> {
        self.resource_id.save(w)?;
        self.paint_shader.save(w)?;
        self.clip_rect.save(w)?;
        self.clip_radii.save(w)?;
        self.source_rect.save(w)?;
        self.tile_rect.save(w)?;
        self.repeat_x.save(w)?;
        self.repeat_y.save(w)?;
        self.repeat_rule_x.save(w)?;
        self.repeat_rule_y.save(w)?;
        self.tile_scale.save(w)?;
        self.tile_spacing.save(w)?;
        self.mode.save(w)?;
        self.composite.save(w)?;
        Ok(())
    }
    fn load(r: &mut impl Read) -> io::Result<Self> {
        Ok(Self {
            resource_id: <u64>::load(r)?,
            paint_shader: <Option<PaintShader>>::load(r)?,
            clip_rect: <PaintRect>::load(r)?,
            clip_radii: <PaintCornerRadii>::load(r)?,
            source_rect: <PaintRect>::load(r)?,
            tile_rect: <PaintRect>::load(r)?,
            repeat_x: <bool>::load(r)?,
            repeat_y: <bool>::load(r)?,
            repeat_rule_x: <BackgroundRepeatRule>::load(r)?,
            repeat_rule_y: <BackgroundRepeatRule>::load(r)?,
            tile_scale: <Offset>::load(r)?,
            tile_spacing: <Size>::load(r)?,
            mode: <PaintMaskMode>::load(r)?,
            composite: <PaintMaskComposite>::load(r)?,
        })
    }
}
impl Codec for DrawCommand {
    fn save(&self, w: &mut impl Write) -> io::Result<()> {
        self.r#type.save(w)?;
        self.rect.save(w)?;
        self.source_rect.save(w)?;
        self.inner_rect.save(w)?;
        self.tile_rect.save(w)?;
        self.color.save(w)?;
        self.line_style.save(w)?;
        self.decoration_style.save(w)?;
        self.stroke_width.save(w)?;
        self.corner_radii.save(w)?;
        self.inner_corner_radii.save(w)?;
        self.dash_intervals.save(w)?;
        self.path.save(w)?;
        self.even_odd.save(w)?;
        self.inverse_winding.save(w)?;
        self.svg_line_cap.save(w)?;
        self.svg_line_join.save(w)?;
        self.dash_offset.save(w)?;
        self.dash_fit_thickness.save(w)?;
        self.miter_limit.save(w)?;
        self.antialias.save(w)?;
        self.non_scaling_stroke.save(w)?;
        self.paint_shader.save(w)?;
        self.blend_mode.save(w)?;
        self.filters.save(w)?;
        self.mask_layers.save(w)?;
        self.round_cap.save(w)?;
        self.resource_id.save(w)?;
        self.repeat_x.save(w)?;
        self.repeat_y.save(w)?;
        self.tile_spacing.save(w)?;
        self.shadow_offset.save(w)?;
        self.blur_radius.save(w)?;
        self.spread.save(w)?;
        self.inset.save(w)?;
        self.shadow_has_opaque_background.save(w)?;
        self.is_text_decoration.save(w)?;
        self.stroke_glyphs.save(w)?;
        self.opacity.save(w)?;
        self.transform.save(w)?;
        self.font_face_index.save(w)?;
        self.font_variations.save(w)?;
        self.font_size.save(w)?;
        self.text_blob_origin.save(w)?;
        self.synthetic_bold.save(w)?;
        self.synthetic_italic.save(w)?;
        self.font_smoothing.save(w)?;
        self.glyphs.save(w)?;
        Ok(())
    }
    fn load(r: &mut impl Read) -> io::Result<Self> {
        Ok(Self {
            r#type: <CommandKind>::load(r)?,
            rect: <PaintRect>::load(r)?,
            source_rect: <PaintRect>::load(r)?,
            inner_rect: <PaintRect>::load(r)?,
            tile_rect: <PaintRect>::load(r)?,
            color: <Color>::load(r)?,
            line_style: <BorderLineStyle>::load(r)?,
            decoration_style: <TextDecorationStyle>::load(r)?,
            stroke_width: <f64>::load(r)?,
            corner_radii: <PaintCornerRadii>::load(r)?,
            inner_corner_radii: <PaintCornerRadii>::load(r)?,
            dash_intervals: <Vec<f64>>::load(r)?,
            path: <Vec<PaintPathCommand>>::load(r)?,
            even_odd: <bool>::load(r)?,
            inverse_winding: <bool>::load(r)?,
            svg_line_cap: <SvgStrokeLineCap>::load(r)?,
            svg_line_join: <SvgStrokeLineJoin>::load(r)?,
            dash_offset: <f64>::load(r)?,
            dash_fit_thickness: <f64>::load(r)?,
            miter_limit: <f64>::load(r)?,
            antialias: <bool>::load(r)?,
            non_scaling_stroke: <bool>::load(r)?,
            paint_shader: <Option<PaintShader>>::load(r)?,
            blend_mode: <PaintBlendMode>::load(r)?,
            filters: <Vec<PaintFilterOperation>>::load(r)?,
            mask_layers: <Vec<MaskLayer>>::load(r)?,
            round_cap: <bool>::load(r)?,
            resource_id: <u64>::load(r)?,
            repeat_x: <bool>::load(r)?,
            repeat_y: <bool>::load(r)?,
            tile_spacing: <Size>::load(r)?,
            shadow_offset: <Offset>::load(r)?,
            blur_radius: <f64>::load(r)?,
            spread: <f64>::load(r)?,
            inset: <bool>::load(r)?,
            shadow_has_opaque_background: <bool>::load(r)?,
            is_text_decoration: <bool>::load(r)?,
            stroke_glyphs: <bool>::load(r)?,
            opacity: <f32>::load(r)?,
            transform: <TransformMatrix>::load(r)?,
            font_face_index: <u32>::load(r)?,
            font_variations: <Vec<FontVariation>>::load(r)?,
            font_size: <f64>::load(r)?,
            text_blob_origin: <Offset>::load(r)?,
            synthetic_bold: <bool>::load(r)?,
            synthetic_italic: <bool>::load(r)?,
            font_smoothing: <FontSmoothing>::load(r)?,
            glyphs: <Vec<PaintGlyph>>::load(r)?,
        })
    }
}

struct OwnedFont {
    family: String,
    native_family: String,
    weight: f64,
    italic: bool,
    bytes: Vec<u8>,
    face_index: u32,
    variations: Vec<FontVariation>,
}
struct OwnedImage {
    id: u64,
    width: u32,
    height: u32,
    rgba8: Vec<u8>,
}
/// Complete replay inputs, independent of Page, URL transports and script clocks.
pub struct Snapshot {
    commands: Vec<DrawCommand>,
    fonts: Vec<OwnedFont>,
    images: Vec<OwnedImage>,
}
impl Snapshot {
    pub fn capture(list: &paint::paint_engine::PaintArtifact) -> Self {
        use crate::convert::ToSkia;
        let resources = crate::convert::resources(list);
        Self {
            commands: list.items.iter().map(ToSkia::to_skia).collect(),
            fonts: resources.resources.as_ref().map_or_else(Vec::new, |r| {
                r.fonts
                    .iter()
                    .map(|f| OwnedFont {
                        family: f.family.into(),
                        native_family: f.native_family.into(),
                        weight: f.weight,
                        italic: f.italic,
                        bytes: f.bytes.into(),
                        face_index: f.face_index,
                        variations: f.variations.clone(),
                    })
                    .collect()
            }),
            images: resources.resources.as_ref().map_or_else(Vec::new, |r| {
                r.images
                    .iter()
                    .map(|i| OwnedImage {
                        id: i.id,
                        width: i.width,
                        height: i.height,
                        rgba8: i.rgba8.into(),
                    })
                    .collect()
            }),
        }
    }
    pub fn save(&self, path: &std::path::Path) -> io::Result<()> {
        let mut w = io::BufWriter::new(std::fs::File::create(path)?);
        w.write_all(b"QQRS0001")?;
        self.commands.save(&mut w)?;
        (self.fonts.len() as u64).save(&mut w)?;
        for f in &self.fonts {
            f.family.save(&mut w)?;
            f.native_family.save(&mut w)?;
            f.weight.save(&mut w)?;
            f.italic.save(&mut w)?;
            save_bytes(&f.bytes, &mut w)?;
            f.face_index.save(&mut w)?;
            f.variations.save(&mut w)?;
        }
        (self.images.len() as u64).save(&mut w)?;
        for i in &self.images {
            i.id.save(&mut w)?;
            i.width.save(&mut w)?;
            i.height.save(&mut w)?;
            save_bytes(&i.rgba8, &mut w)?;
        }
        w.flush()
    }
    pub fn load(path: &std::path::Path) -> io::Result<Self> {
        if path.as_os_str() == "-" {
            Self::read_from(std::io::stdin().lock())
        } else {
            Self::read_from(std::fs::File::open(path)?)
        }
    }
    /// Decode before timing; e.g. gzip -dc SCENE.snap.gz | probe --load -.
    pub fn read_from(reader: impl Read) -> io::Result<Self> {
        let mut r = io::BufReader::new(reader);
        let mut header = [0u8; 8];
        r.read_exact(&mut header)?;
        if &header != b"QQRS0001" {
            return Err(io::Error::other("snapshot version mismatch"));
        }
        let commands = Vec::<DrawCommand>::load(&mut r)?;
        let fonts = (0..catalog_count(&mut r)?)
            .map(|_| {
                Ok(OwnedFont {
                    family: String::load(&mut r)?,
                    native_family: String::load(&mut r)?,
                    weight: f64::load(&mut r)?,
                    italic: bool::load(&mut r)?,
                    bytes: load_bytes(&mut r)?,
                    face_index: u32::load(&mut r)?,
                    variations: Vec::<FontVariation>::load(&mut r)?,
                })
            })
            .collect::<io::Result<Vec<_>>>()?;
        let images = (0..catalog_count(&mut r)?)
            .map(|_| {
                Ok(OwnedImage {
                    id: u64::load(&mut r)?,
                    width: u32::load(&mut r)?,
                    height: u32::load(&mut r)?,
                    rgba8: load_bytes(&mut r)?,
                })
            })
            .collect::<io::Result<Vec<_>>>()?;
        Ok(Self {
            commands,
            fonts,
            images,
        })
    }
    pub fn command_count(&self) -> usize {
        self.commands.len()
    }
    /// Resource fingerprint evidence; file SHA-256 remains the scene identity.
    pub fn write_resource_manifest(&self, w: &mut impl Write) -> io::Result<()> {
        let hash = |bytes: &[u8]| {
            bytes.iter().fold(0xcbf29ce484222325u64, |h, b| {
                (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
            })
        };
        for (index, font) in self.fonts.iter().enumerate() {
            writeln!(w,"snapshot-font index={index} family={:?} native={:?} face={} bytes={} fnv1a64={:016x}",font.family,font.native_family,font.face_index,font.bytes.len(),hash(&font.bytes))?;
        }
        for image in &self.images {
            writeln!(
                w,
                "snapshot-image id={} width={} height={} bytes={} fnv1a64={:016x}",
                image.id,
                image.width,
                image.height,
                image.rgba8.len(),
                hash(&image.rgba8)
            )?;
        }
        Ok(())
    }
    /// CPU replay into the ordinary window format, including all save/restore states.
    /// Conversion and loading are excluded; this measures only Skia replay.
    pub fn replay(
        &self,
        width: u32,
        height: u32,
        scale: f64,
        pixels: &mut [u32],
    ) -> io::Result<()> {
        let resources = ResourceContext {
            resources: Some(ResourceCatalog {
                fonts: self
                    .fonts
                    .iter()
                    .map(|f| FontFace {
                        family: &f.family,
                        native_family: &f.native_family,
                        weight: f.weight,
                        italic: f.italic,
                        bytes: &f.bytes,
                        face_index: f.face_index,
                        variations: f.variations.clone(),
                    })
                    .collect(),
                images: self
                    .images
                    .iter()
                    .map(|i| Image::borrowed(i.id, i.width, i.height, &i.rgba8))
                    .collect(),
            }),
        };
        let expected = (width as usize)
            .checked_mul(height as usize)
            .filter(|n| *n > 0)
            .ok_or_else(|| io::Error::other("invalid dimensions"))?;
        if pixels.len() != expected
            || !scale.is_finite()
            || scale <= 0.
            || !cfg!(target_endian = "little")
        {
            return Err(io::Error::other("invalid target"));
        }
        let bytes = expected
            .checked_mul(4)
            .filter(|n| *n <= isize::MAX as usize)
            .ok_or_else(|| io::Error::other("invalid target size"))?;
        let ptr = std::ptr::NonNull::new(pixels.as_mut_ptr().cast()).unwrap();
        // SAFETY: initialized exclusive caller slice outlives the canvas. The inert
        // owner has no allocator responsibilities; storage cannot escape this call.
        let storage = unsafe { skia::PixelStorage::from_external(ptr, bytes, Box::new(())) }
            .ok_or_else(|| io::Error::other("invalid storage"))?;
        let mut canvas = skia::cpu::canvas::Canvas::make_raster_direct_with_format(
            &resources,
            width,
            height,
            width as usize * 4,
            storage,
            skia::PixelFormat::Bgrx8888,
        )
        .ok_or_else(|| io::Error::other("invalid target"))?;
        canvas.set_scale(scale);
        let totals = std::env::var_os("RENDERER_TRACE_TOTALS").is_some();
        let items = std::env::var_os("RENDERER_TRACE_ITEMS").is_some();
        let trace = totals || items;
        let mut buckets = vec![(std::time::Duration::ZERO, 0usize); 256];
        for (index, command) in self.commands.iter().enumerate() {
            let start = trace.then(std::time::Instant::now);
            canvas.replay_item(command, &resources);
            if let Some(start) = start {
                let b = &mut buckets[command.r#type as usize];
                let elapsed = start.elapsed();
                b.0 += elapsed;
                b.1 += 1;
                if items && elapsed.as_secs_f64() > 0.0001 {
                    eprintln!(
                        "snapshot-item index={index} kind={:?} ms={:.6} rect={:?} radii={:?}",
                        command.r#type,
                        elapsed.as_secs_f64() * 1000.,
                        command.rect,
                        command.corner_radii
                    );
                }
            }
        }
        if totals {
            let mut kinds: Vec<_> = buckets
                .iter()
                .enumerate()
                .filter(|(_, b)| b.1 > 0)
                .collect();
            kinds.sort_by_key(|(_, b)| std::cmp::Reverse(b.0));
            for (kind, b) in kinds {
                eprintln!(
                    "snapshot-kind kind={kind} count={} ms={:.6}",
                    b.1,
                    b.0.as_secs_f64() * 1000.
                );
            }
        }
        drop(canvas.finish_direct().into_external_owner());
        Ok(())
    }
}
