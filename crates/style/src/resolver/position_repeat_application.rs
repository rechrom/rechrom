// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Concrete positional converters and generated fill-layer application loops.
//! Transform origins reuse typed CSS math; missing length contexts stay typed Unsupported.
#![allow(non_snake_case)]
use crate::{
    css_primitive_value::UnitType, css_value::CSSValuePayload,
    media_queries::MediaValuesCachedData, production_css_value::Value,
    properties::longhand_dispatch::LonghandApplicationError,
};
use foundation::{
    BlendMode, CSSPropertyID, CSSValueID, Length, LengthPoint, LengthSize, LengthType,
};
use layoutng_style::style::{
    computed_style::{ComputedStyle, ComputedStyleBuilder},
    computed_style_constants::{
        BackgroundEdgeOrigin, CompositingOperator, EFillAttachment, EFillBox, EFillLayerType,
        EFillMaskMode, EFillRepeat, EFillSizeType,
    },
    computed_style_initial_values::ComputedStyleInitialValues,
    fill_layer::{FillLayer, FillRepeat, FillSize},
    transform_origin::TransformOrigin,
};
type Result<T = ()> = std::result::Result<T, LonghandApplicationError>;

pub fn IsPositionRepeatProperty(id: CSSPropertyID) -> bool {
    matches!(
        id,
        CSSPropertyID::kObjectPosition
            | CSSPropertyID::kPerspectiveOrigin
            | CSSPropertyID::kOffsetAnchor
            | CSSPropertyID::kOffsetPosition
            | CSSPropertyID::kTransformOrigin
            | CSSPropertyID::kBackgroundPositionX
            | CSSPropertyID::kBackgroundPositionY
            | CSSPropertyID::kWebkitMaskPositionX
            | CSSPropertyID::kWebkitMaskPositionY
            | CSSPropertyID::kBackgroundAttachment
            | CSSPropertyID::kBackgroundBlendMode
            | CSSPropertyID::kBackgroundClip
            | CSSPropertyID::kBackgroundOrigin
            | CSSPropertyID::kBackgroundSize
            | CSSPropertyID::kBackgroundRepeat
            | CSSPropertyID::kMaskRepeat
            | CSSPropertyID::kMaskSize
            | CSSPropertyID::kMaskOrigin
            | CSSPropertyID::kMaskClip
            | CSSPropertyID::kMaskComposite
            | CSSPropertyID::kMaskMode
    )
}
fn Ident(id: CSSPropertyID, value: &Value) -> Result<CSSValueID> {
    match value.Payload() {
        CSSValuePayload::kIdentifierClass(value) => Ok(value.0),
        _ => Err(LonghandApplicationError::InvalidValue(id)),
    }
}
// cpp: style_builder_converter.cc ConvertLength; css_numeric_literal_value.cc
// ComputeLengthDouble. Reuse the production literal resolver's admitted units.
// Typed math uses the same length resolver as transform application. Missing
// font/container metrics and non-transform zoom policies remain collaborators.
pub(super) fn ConvertLength(
    id: CSSPropertyID,
    value: &Value,
    b: &ComputedStyleBuilder,
    root: f32,
    media: &MediaValuesCachedData,
) -> Result<Length> {
    if b.EffectiveZoom() != 1.0
        && !matches!(value.Payload(), CSSValuePayload::kNumericLiteralClass(n) if n.GetType() == UnitType::kPercentage)
        && !matches!(
            id,
            CSSPropertyID::kTransform
                | CSSPropertyID::kTranslate
                | CSSPropertyID::kTransformOrigin
                | CSSPropertyID::kPerspectiveOrigin
                | CSSPropertyID::kWebkitTransformOriginX
                | CSSPropertyID::kWebkitTransformOriginY
                | CSSPropertyID::kWebkitPerspectiveOriginX
                | CSSPropertyID::kWebkitPerspectiveOriginY
        )
    {
        return Err(LonghandApplicationError::Unsupported(id));
    }
    let mut resolver = super::production_style_builder::MathLengthResolver(
        id,
        b.GetFontDescription().ComputedSize(),
        root,
        b.EffectiveZoom(),
        media,
    );
    if let CSSValuePayload::kMathFunctionClass(math) = value.Payload() {
        return math
            .ConvertToLength(&mut resolver)
            .map_err(|_| LonghandApplicationError::Unsupported(id));
    }
    let CSSValuePayload::kNumericLiteralClass(value) = value.Payload() else {
        return Err(LonghandApplicationError::InvalidValue(id));
    };
    if value.GetType() == UnitType::kPercentage {
        return Ok(Length::Percent(value.DoubleValue()));
    }
    use crate::css_math_expression_node::CSSMathLengthResolver;
    let pixels = resolver
        .ComputeLength(value.DoubleValue(), value.GetType())
        .map_err(|_| LonghandApplicationError::Unsupported(id))?;
    Ok(Length::Fixed(
        crate::css_value_clamping_utils::CSSValueClampingUtils::ClampLength(pixels),
    ))
}
// cpp: style_builder_converter.h:541-574 ConvertPositionLength.
pub(super) fn ConvertPositionLength(
    id: CSSPropertyID,
    value: &Value,
    horizontal: bool,
    b: &ComputedStyleBuilder,
    root: f32,
    media: &MediaValuesCachedData,
) -> Result<Length> {
    let (zero, hundred) = if horizontal {
        (CSSValueID::kLeft, CSSValueID::kRight)
    } else {
        (CSSValueID::kTop, CSSValueID::kBottom)
    };
    if let CSSValuePayload::kValuePairClass(pair) = value.Payload() {
        let length = ConvertLength(id, &pair.second, b, root, media)?;
        return match Ident(id, &pair.first)? {
            edge if edge == zero => Ok(length),
            edge if edge == hundred => Ok(length.SubtractFromOneHundredPercent()),
            _ => Err(LonghandApplicationError::InvalidValue(id)),
        };
    }
    if let CSSValuePayload::kIdentifierClass(value) = value.Payload() {
        return match value.0 {
            edge if edge == zero => Ok(Length::Percent(0)),
            edge if edge == hundred => Ok(Length::Percent(100)),
            CSSValueID::kCenter => Ok(Length::Percent(50)),
            _ => Err(LonghandApplicationError::InvalidValue(id)),
        };
    }
    ConvertLength(id, value, b, root, media)
}
// cpp: style_builder_converter.cc:2473-2504.
fn ConvertPosition(
    id: CSSPropertyID,
    value: &Value,
    b: &ComputedStyleBuilder,
    root: f32,
    media: &MediaValuesCachedData,
) -> Result<LengthPoint> {
    if let CSSValuePayload::kValuePairClass(pair) = value.Payload() {
        return Ok(LengthPoint::new(
            &ConvertPositionLength(id, &pair.first, true, b, root, media)?,
            &ConvertPositionLength(id, &pair.second, false, b, root, media)?,
        ));
    }
    let keyword = Ident(id, value)?;
    let type_ = if matches!(
        id,
        CSSPropertyID::kOffsetAnchor | CSSPropertyID::kOffsetPosition
    ) && keyword == CSSValueID::kAuto
    {
        LengthType::kAuto
    } else if id == CSSPropertyID::kOffsetPosition && keyword == CSSValueID::kNormal {
        LengthType::kNone
    } else {
        return Err(LonghandApplicationError::InvalidValue(id));
    };
    let length = Length::new(0.0, type_);
    Ok(LengthPoint::new(&length, &length))
}
// cpp: style_builder_converter.cc:3288-3308.
fn ConvertTransformOrigin(
    id: CSSPropertyID,
    value: &Value,
    b: &ComputedStyleBuilder,
    root: f32,
    media: &MediaValuesCachedData,
) -> Result<TransformOrigin> {
    let CSSValuePayload::kValueListClass(list) = value.Payload() else {
        return Err(LonghandApplicationError::InvalidValue(id));
    };
    if !(2..=3).contains(&list.values.len()) {
        return Err(LonghandApplicationError::InvalidValue(id));
    }
    let z = if list.values.len() == 3 {
        let length = ConvertLength(id, &list.values[2], b, root, media)?;
        if !length.IsFixed() {
            return Err(LonghandApplicationError::InvalidValue(id));
        }
        length.Pixels()
    } else {
        0.0
    };
    Ok(TransformOrigin::new(
        &ConvertPositionLength(id, &list.values[0], true, b, root, media)?,
        &ConvertPositionLength(id, &list.values[1], false, b, root, media)?,
        z,
    ))
}
#[derive(Clone, Copy)]
enum Field {
    Attachment,
    BlendMode,
    X,
    Y,
    Repeat,
    Size,
    Origin,
    Clip,
    Composite,
    Mode,
}
fn LayerField(id: CSSPropertyID) -> Option<(EFillLayerType, Field)> {
    use CSSPropertyID::*;
    Some(match id {
        kBackgroundAttachment => (EFillLayerType::kBackground, Field::Attachment),
        kBackgroundBlendMode => (EFillLayerType::kBackground, Field::BlendMode),
        kBackgroundClip => (EFillLayerType::kBackground, Field::Clip),
        kBackgroundOrigin => (EFillLayerType::kBackground, Field::Origin),
        kBackgroundSize => (EFillLayerType::kBackground, Field::Size),
        kBackgroundPositionX => (EFillLayerType::kBackground, Field::X),
        kBackgroundPositionY => (EFillLayerType::kBackground, Field::Y),
        kWebkitMaskPositionX => (EFillLayerType::kMask, Field::X),
        kWebkitMaskPositionY => (EFillLayerType::kMask, Field::Y),
        kBackgroundRepeat => (EFillLayerType::kBackground, Field::Repeat),
        kMaskRepeat => (EFillLayerType::kMask, Field::Repeat),
        kMaskSize => (EFillLayerType::kMask, Field::Size),
        kMaskOrigin => (EFillLayerType::kMask, Field::Origin),
        kMaskClip => (EFillLayerType::kMask, Field::Clip),
        kMaskComposite => (EFillLayerType::kMask, Field::Composite),
        kMaskMode => (EFillLayerType::kMask, Field::Mode),
        _ => return None,
    })
}
enum LayerValue {
    Attachment(EFillAttachment),
    BlendMode(BlendMode),
    Position(Length, Option<BackgroundEdgeOrigin>),
    Repeat(FillRepeat),
    Size(FillSize),
    Box(EFillBox),
    Composite(CompositingOperator),
    Mode(EFillMaskMode),
}
fn RepeatAxis(id: CSSPropertyID, value: &Value) -> Result<EFillRepeat> {
    Ok(match Ident(id, value)? {
        CSSValueID::kRepeat => EFillRepeat::kRepeatFill,
        CSSValueID::kNoRepeat => EFillRepeat::kNoRepeatFill,
        CSSValueID::kRound => EFillRepeat::kRoundFill,
        CSSValueID::kSpace => EFillRepeat::kSpaceFill,
        _ => return Err(LonghandApplicationError::InvalidValue(id)),
    })
}
// css_to_style_map.cc:72-95,109-117,145-198. Preserve native enum tags.
fn FillBox(id: CSSPropertyID, v: &Value) -> Result<EFillBox> {
    Ok(match Ident(id, v)? {
        CSSValueID::kBorder | CSSValueID::kBorderBox => EFillBox::kBorder,
        CSSValueID::kPadding | CSSValueID::kPaddingBox => EFillBox::kPadding,
        CSSValueID::kContent | CSSValueID::kContentBox => EFillBox::kContent,
        CSSValueID::kFillBox => EFillBox::kFillBox,
        CSSValueID::kStrokeBox => EFillBox::kStrokeBox,
        CSSValueID::kViewBox => EFillBox::kViewBox,
        CSSValueID::kNoClip if id == CSSPropertyID::kMaskClip => EFillBox::kNoClip,
        CSSValueID::kBorderArea if id == CSSPropertyID::kBackgroundClip => EFillBox::kBorderArea,
        CSSValueID::kText
            if matches!(
                id,
                CSSPropertyID::kMaskClip | CSSPropertyID::kBackgroundClip
            ) =>
        {
            EFillBox::kText
        }
        _ => return Err(LonghandApplicationError::InvalidValue(id)),
    })
}
fn MaskComposite(id: CSSPropertyID, v: &Value) -> Result<CompositingOperator> {
    use CompositingOperator as O;
    Ok(match Ident(id, v)? {
        CSSValueID::kAdd => O::kAdd,
        CSSValueID::kSubtract => O::kSubtract,
        CSSValueID::kIntersect => O::kIntersect,
        CSSValueID::kExclude => O::kExclude,
        CSSValueID::kClear => O::kClear,
        CSSValueID::kCopy => O::kCopy,
        CSSValueID::kSourceOver => O::kSourceOver,
        CSSValueID::kSourceIn => O::kSourceIn,
        CSSValueID::kSourceOut => O::kSourceOut,
        CSSValueID::kSourceAtop => O::kSourceAtop,
        CSSValueID::kDestinationOver => O::kDestinationOver,
        CSSValueID::kDestinationIn => O::kDestinationIn,
        CSSValueID::kDestinationOut => O::kDestinationOut,
        CSSValueID::kDestinationAtop => O::kDestinationAtop,
        CSSValueID::kXor => O::kXOR,
        CSSValueID::kPlusLighter => O::kPlusLighter,
        _ => return Err(LonghandApplicationError::InvalidValue(id)),
    })
}
fn FillLength(
    id: CSSPropertyID,
    v: &Value,
    b: &ComputedStyleBuilder,
    root: f32,
    media: &MediaValuesCachedData,
) -> Result<Length> {
    if matches!(v.Payload(),CSSValuePayload::kIdentifierClass(k) if k.0==CSSValueID::kAuto) {
        Ok(Length::Auto().clone())
    } else {
        ConvertLength(id, v, b, root, media)
    }
}
// cpp: css_to_style_map.cc:133-143,200-239. Fill edge offsets retain an origin
// flag, whereas ConvertPositionLength subtracts right/bottom from 100%.
fn MapFill(
    id: CSSPropertyID,
    field: Field,
    value: &Value,
    b: &ComputedStyleBuilder,
    root: f32,
    media: &MediaValuesCachedData,
) -> Result<LayerValue> {
    if value.IsInitialValue() {
        return Ok(InitialLayer(LayerField(id).unwrap().0, field));
    }
    match field {
        Field::Attachment => Ok(LayerValue::Attachment(match Ident(id, value)? {
            CSSValueID::kScroll => EFillAttachment::kScroll,
            CSSValueID::kLocal => EFillAttachment::kLocal,
            CSSValueID::kFixed => EFillAttachment::kFixed,
            _ => return Err(LonghandApplicationError::InvalidValue(id)),
        })),
        Field::BlendMode => Ok(LayerValue::BlendMode(match Ident(id, value)? {
            CSSValueID::kNormal => BlendMode::kNormal,
            CSSValueID::kMultiply => BlendMode::kMultiply,
            CSSValueID::kScreen => BlendMode::kScreen,
            CSSValueID::kOverlay => BlendMode::kOverlay,
            CSSValueID::kDarken => BlendMode::kDarken,
            CSSValueID::kLighten => BlendMode::kLighten,
            CSSValueID::kColorDodge => BlendMode::kColorDodge,
            CSSValueID::kColorBurn => BlendMode::kColorBurn,
            CSSValueID::kHardLight => BlendMode::kHardLight,
            CSSValueID::kSoftLight => BlendMode::kSoftLight,
            CSSValueID::kDifference => BlendMode::kDifference,
            CSSValueID::kExclusion => BlendMode::kExclusion,
            CSSValueID::kHue => BlendMode::kHue,
            CSSValueID::kSaturation => BlendMode::kSaturation,
            CSSValueID::kColor => BlendMode::kColor,
            CSSValueID::kLuminosity => BlendMode::kLuminosity,
            CSSValueID::kPlusLighter => BlendMode::kPlusLighter,
            _ => return Err(LonghandApplicationError::InvalidValue(id)),
        })),
        Field::Clip
            if id == CSSPropertyID::kBackgroundClip
                && matches!(value.Payload(), CSSValuePayload::kValuePairClass(_)) =>
        {
            let CSSValuePayload::kValuePairClass(pair) = value.Payload() else {
                unreachable!()
            };
            let a = Ident(id, &pair.first)?;
            let z = Ident(id, &pair.second)?;
            if !matches!(
                (a, z),
                (CSSValueID::kText, CSSValueID::kBorderArea)
                    | (CSSValueID::kBorderArea, CSSValueID::kText)
            ) {
                return Err(LonghandApplicationError::InvalidValue(id));
            }
            Ok(LayerValue::Box(EFillBox::kBorderAreaText))
        }
        Field::Origin | Field::Clip => Ok(LayerValue::Box(FillBox(id, value)?)),
        Field::Composite => Ok(LayerValue::Composite(MaskComposite(id, value)?)),
        Field::Mode => Ok(LayerValue::Mode(match Ident(id, value)? {
            CSSValueID::kAlpha => EFillMaskMode::kAlpha,
            CSSValueID::kLuminance => EFillMaskMode::kLuminance,
            CSSValueID::kMatchSource => EFillMaskMode::kMatchSource,
            _ => return Err(LonghandApplicationError::InvalidValue(id)),
        })),
        Field::Size => {
            let type_ = match value.Payload() {
                CSSValuePayload::kIdentifierClass(k) if k.0 == CSSValueID::kContain => {
                    EFillSizeType::kContain
                }
                CSSValuePayload::kIdentifierClass(k) if k.0 == CSSValueID::kCover => {
                    EFillSizeType::kCover
                }
                _ => EFillSizeType::kSizeLength,
            };
            let size = if type_ != EFillSizeType::kSizeLength {
                FillLayer::InitialFillSizeLength(LayerField(id).unwrap().0)
            } else if let CSSValuePayload::kValuePairClass(p) = value.Payload() {
                LengthSize::new(
                    &FillLength(id, &p.first, b, root, media)?,
                    &FillLength(id, &p.second, b, root, media)?,
                )
            } else {
                LengthSize::new(&FillLength(id, value, b, root, media)?, Length::Auto())
            };
            Ok(LayerValue::Size(FillSize::new(type_, &size)))
        }
        Field::Repeat => {
            let CSSValuePayload::kRepeatStyleClass(repeat) = value.Payload() else {
                return Err(LonghandApplicationError::InvalidValue(id));
            };
            Ok(LayerValue::Repeat(FillRepeat {
                x: RepeatAxis(id, &repeat.x)?,
                y: RepeatAxis(id, &repeat.y)?,
            }))
        }
        Field::X | Field::Y => {
            if let CSSValuePayload::kValuePairClass(pair) = value.Payload() {
                let origin = match Ident(id, &pair.first)? {
                    CSSValueID::kLeft if matches!(field, Field::X) => BackgroundEdgeOrigin::kLeft,
                    CSSValueID::kRight if matches!(field, Field::X) => BackgroundEdgeOrigin::kRight,
                    CSSValueID::kTop if matches!(field, Field::Y) => BackgroundEdgeOrigin::kTop,
                    CSSValueID::kBottom if matches!(field, Field::Y) => {
                        BackgroundEdgeOrigin::kBottom
                    }
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                };
                Ok(LayerValue::Position(
                    ConvertLength(id, &pair.second, b, root, media)?,
                    Some(origin),
                ))
            } else {
                Ok(LayerValue::Position(
                    ConvertPositionLength(id, value, matches!(field, Field::X), b, root, media)?,
                    None,
                ))
            }
        }
    }
}
fn IsSet(layer: &FillLayer, field: Field) -> bool {
    match field {
        Field::Attachment => layer.IsAttachmentSet(),
        Field::BlendMode => layer.IsBlendModeSet(),
        Field::X => layer.IsPositionXSet(),
        Field::Y => layer.IsPositionYSet(),
        Field::Repeat => layer.IsRepeatSet(),
        Field::Size => layer.IsSizeSet(),
        Field::Origin => layer.IsOriginSet(),
        Field::Clip => layer.IsClipSet(),
        Field::Composite => layer.IsCompositingOperatorSet(),
        Field::Mode => layer.IsMaskModeSet(),
    }
}
fn Clear(layer: &mut FillLayer, field: Field) {
    match field {
        Field::Attachment => layer.ClearAttachment(),
        Field::BlendMode => layer.ClearBlendMode(),
        Field::X => layer.ClearPositionX(),
        Field::Y => layer.ClearPositionY(),
        Field::Repeat => layer.ClearRepeat(),
        Field::Size => layer.ClearSize(),
        Field::Origin => layer.ClearOrigin(),
        Field::Clip => layer.ClearClip(),
        Field::Composite => layer.ClearCompositingOperator(),
        Field::Mode => layer.ClearMaskMode(),
    }
}
fn Set(layer: &mut FillLayer, field: Field, value: &LayerValue) {
    match (field, value) {
        (Field::Attachment, LayerValue::Attachment(v)) => layer.SetAttachment(*v),
        (Field::BlendMode, LayerValue::BlendMode(v)) => layer.SetBlendMode(*v),
        (Field::Repeat, LayerValue::Repeat(value)) => layer.SetRepeat(value),
        (Field::Size, LayerValue::Size(v)) => layer.SetSize(v),
        (Field::Origin, LayerValue::Box(v)) => layer.SetOrigin(*v),
        (Field::Clip, LayerValue::Box(v)) => layer.SetClip(*v),
        (Field::Composite, LayerValue::Composite(v)) => layer.SetCompositingOperator(*v),
        (Field::Mode, LayerValue::Mode(v)) => layer.SetMaskMode(*v),
        (Field::X, LayerValue::Position(value, origin)) => {
            layer.SetPositionX(value);
            if let Some(origin) = origin {
                layer.SetBackgroundXOrigin(*origin);
            }
        }
        (Field::Y, LayerValue::Position(value, origin)) => {
            layer.SetPositionY(value);
            if let Some(origin) = origin {
                layer.SetBackgroundYOrigin(*origin);
            }
        }
        _ => unreachable!("source field and mapped layer value agree"),
    }
}
// cpp: generated longhands.cc:3426-3482,3503-3559,3580-3629,
// 10971-11020,18838-18894,18915-18971. Access*/EnsureNext use native COW/GC.
fn ApplyLayers(
    b: &mut ComputedStyleBuilder,
    type_: EFillLayerType,
    field: Field,
    values: &[LayerValue],
) {
    let mut current: *mut FillLayer = if type_ == EFillLayerType::kBackground {
        b.AccessBackgroundLayers()
    } else {
        b.AccessMaskLayers()
    };
    let mut previous: *mut FillLayer = std::ptr::null_mut();
    for value in values {
        if current.is_null() {
            current = unsafe { (*previous).EnsureNext() };
        }
        let layer = unsafe { &mut *current };
        Set(layer, field, value);
        previous = current;
        current = layer.NextMut();
    }
    while let Some(layer) = unsafe { current.as_mut() } {
        Clear(layer, field);
        current = layer.NextMut();
    }
}
fn InitialLayer(type_: EFillLayerType, field: Field) -> LayerValue {
    match field {
        Field::Attachment => LayerValue::Attachment(FillLayer::InitialFillAttachment(type_)),
        Field::BlendMode => LayerValue::BlendMode(FillLayer::InitialFillBlendMode(type_)),
        Field::X => LayerValue::Position(FillLayer::InitialFillPositionX(type_), None),
        Field::Y => LayerValue::Position(FillLayer::InitialFillPositionY(type_), None),
        Field::Repeat => LayerValue::Repeat(FillLayer::InitialFillRepeat(type_)),
        Field::Size => LayerValue::Size(FillLayer::InitialFillSize(type_)),
        Field::Origin => LayerValue::Box(FillLayer::InitialFillOrigin(type_)),
        Field::Clip => LayerValue::Box(FillLayer::InitialFillClip(type_)),
        Field::Composite => LayerValue::Composite(FillLayer::InitialFillCompositingOperator(type_)),
        Field::Mode => LayerValue::Mode(FillLayer::InitialFillMaskMode(type_)),
    }
}
fn ParentLayers(parent: &ComputedStyle, type_: EFillLayerType, field: Field) -> Vec<LayerValue> {
    let mut current: *const FillLayer = if type_ == EFillLayerType::kBackground {
        parent.BackgroundLayers()
    } else {
        parent.MaskLayers()
    };
    let mut values = Vec::new();
    while let Some(layer) = unsafe { current.as_ref() } {
        if !IsSet(layer, field) {
            break;
        }
        values.push(match field {
            Field::Attachment => LayerValue::Attachment(layer.Attachment()),
            Field::BlendMode => LayerValue::BlendMode(layer.GetBlendMode()),
            Field::X => LayerValue::Position(
                layer.PositionX().clone(),
                layer
                    .IsBackgroundXOriginSet()
                    .then(|| layer.BackgroundXOrigin()),
            ),
            Field::Y => LayerValue::Position(
                layer.PositionY().clone(),
                layer
                    .IsBackgroundYOriginSet()
                    .then(|| layer.BackgroundYOrigin()),
            ),
            Field::Repeat => LayerValue::Repeat(*layer.Repeat()),
            Field::Size => LayerValue::Size(layer.Size()),
            Field::Origin => LayerValue::Box(layer.Origin()),
            Field::Clip => LayerValue::Box(layer.Clip()),
            Field::Composite => LayerValue::Composite(layer.CompositingOperator()),
            Field::Mode => LayerValue::Mode(layer.MaskMode()),
        });
        current = layer.Next();
    }
    values
}
// cpp: generated positional ApplyInitial/ApplyInherit/ApplyValue. All these
// properties are non-inherited. The production caller supplies root font/media.
pub fn Apply(
    id: CSSPropertyID,
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    value: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> Result {
    if !IsPositionRepeatProperty(id) {
        return Err(LonghandApplicationError::Unsupported(id));
    }
    let inherit = value.IsInheritedValue();
    let initial = value.IsInitialValue() || value.IsUnsetValue() || inherit && parent.is_none();
    // ApplyParentValueIfZoomChanged requires document zoom policy and parent
    // specified values, which this production application interface lacks.
    if inherit
        && !initial
        && parent.unwrap().EffectiveZoom() != b.EffectiveZoom()
        && (LayerField(id).is_none()
            || matches!(LayerField(id), Some((_, Field::X | Field::Y | Field::Size))))
    {
        return Err(LonghandApplicationError::Unsupported(id));
    }
    if let Some((type_, field)) = LayerField(id) {
        let converted = if initial {
            vec![InitialLayer(type_, field)]
        } else if inherit {
            ParentLayers(parent.unwrap(), type_, field)
        } else if let CSSValuePayload::kValueListClass(list) = value.Payload() {
            list.values
                .iter()
                .map(|v| MapFill(id, field, v, b, root, media))
                .collect::<Result<Vec<_>>>()?
        } else {
            vec![MapFill(id, field, value, b, root, media)?]
        };
        if id == CSSPropertyID::kBackgroundClip && !initial && !inherit {
            // cpp: longhands_custom.cc:1045-1071. Repeat complete list cycles
            // across the existing chain; a final partial cycle creates layers.
            if converted.is_empty() {
                return Err(LonghandApplicationError::InvalidValue(id));
            }
            let mut current = b.AccessBackgroundLayers() as *mut FillLayer;
            let mut previous: *mut FillLayer = std::ptr::null_mut();
            while !current.is_null() {
                for value in &converted {
                    if current.is_null() {
                        current = unsafe { (*previous).EnsureNext() };
                    }
                    let layer = unsafe { &mut *current };
                    Set(layer, field, value);
                    previous = current;
                    current = layer.NextMut();
                }
            }
        } else {
            ApplyLayers(b, type_, field, &converted);
        }
    } else if id == CSSPropertyID::kTransformOrigin {
        let converted = if initial {
            ComputedStyleInitialValues::InitialTransformOrigin()
        } else if inherit {
            parent.unwrap().GetTransformOrigin().clone()
        } else {
            ConvertTransformOrigin(id, value, b, root, media)?
        };
        b.SetTransformOrigin(&converted);
    } else {
        let converted = if initial {
            match id {
                CSSPropertyID::kObjectPosition => {
                    ComputedStyleInitialValues::InitialObjectPosition()
                }
                CSSPropertyID::kPerspectiveOrigin => {
                    ComputedStyleInitialValues::InitialPerspectiveOrigin()
                }
                CSSPropertyID::kOffsetAnchor => ComputedStyleInitialValues::InitialOffsetAnchor(),
                CSSPropertyID::kOffsetPosition => {
                    ComputedStyleInitialValues::InitialOffsetPosition()
                }
                _ => unreachable!(),
            }
        } else if inherit {
            let parent = parent.unwrap();
            match id {
                CSSPropertyID::kObjectPosition => parent.ObjectPosition().clone(),
                CSSPropertyID::kPerspectiveOrigin => parent.PerspectiveOrigin().clone(),
                CSSPropertyID::kOffsetAnchor => parent.OffsetAnchor().clone(),
                CSSPropertyID::kOffsetPosition => parent.OffsetPosition().clone(),
                _ => unreachable!(),
            }
        } else {
            ConvertPosition(id, value, b, root, media)?
        };
        match id {
            CSSPropertyID::kObjectPosition => b.SetObjectPosition(&converted),
            CSSPropertyID::kPerspectiveOrigin => b.SetPerspectiveOrigin(&converted),
            CSSPropertyID::kOffsetAnchor => b.SetOffsetAnchor(&converted),
            CSSPropertyID::kOffsetPosition => b.SetOffsetPosition(&converted),
            _ => unreachable!(),
        }
    }
    if inherit && !initial {
        b.SetHasExplicitInheritance();
        parent.unwrap().SetChildHasExplicitInheritance();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{
        css_parser_mode::CSSParserMode, production_property_parser::ParseProperty,
    };
    use foundation::String;
    fn initial() -> &'static ComputedStyle {
        unsafe { &*ComputedStyle::GetInitialStyleSingleton() }
    }
    fn apply(
        b: &mut ComputedStyleBuilder,
        id: CSSPropertyID,
        css: &str,
        parent: Option<&ComputedStyle>,
    ) -> Result {
        let values = ParseProperty(
            id,
            &String::from(css),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .unwrap();
        for property in values {
            Apply(
                property.PropertyID(),
                b,
                parent,
                property.Value(),
                16.0,
                &MediaValuesCachedData::default(),
            )?;
        }
        Ok(())
    }
    #[test]
    fn background_fields_preserve_layer_lists_and_clip_cycles() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut b = ComputedStyleBuilder::from_style(initial());
        apply(
            &mut b,
            CSSPropertyID::kBackgroundAttachment,
            "fixed, local, scroll",
            None,
        )
        .unwrap();
        apply(
            &mut b,
            CSSPropertyID::kBackgroundBlendMode,
            "multiply, screen",
            None,
        )
        .unwrap();
        apply(
            &mut b,
            CSSPropertyID::kBackgroundOrigin,
            "content-box, border-box",
            None,
        )
        .unwrap();
        apply(
            &mut b,
            CSSPropertyID::kBackgroundSize,
            "calc(2px + 3px) auto, cover",
            None,
        )
        .unwrap();
        assert_eq!(
            b.AccessBackgroundLayers().SizeLength().Width().Pixels(),
            5.0
        );
        assert_eq!(
            b.AccessBackgroundLayers().Attachment(),
            EFillAttachment::kFixed
        );
        let third = unsafe { &*(*b.AccessBackgroundLayers().Next()).Next() };
        assert!(
            third.IsAttachmentSet()
                && !third.IsBlendModeSet()
                && !third.IsOriginSet()
                && !third.IsSizeSet()
        );
        // BackgroundClip::ApplyValue repeats a complete two-value cycle over
        // three existing layers, creating the fourth layer for the final item.
        apply(
            &mut b,
            CSSPropertyID::kBackgroundClip,
            "text, border-area",
            None,
        )
        .unwrap();
        let first = b.AccessBackgroundLayers();
        let second = unsafe { &*first.Next() };
        let third = unsafe { &*second.Next() };
        let fourth = unsafe { &*third.Next() };
        assert_eq!(
            (first.Clip(), second.Clip(), third.Clip(), fourth.Clip()),
            (
                EFillBox::kText,
                EFillBox::kBorderArea,
                EFillBox::kText,
                EFillBox::kBorderArea
            )
        );
        assert_eq!(second.GetBlendMode(), BlendMode::kScreen);
        assert_eq!(second.Origin(), EFillBox::kBorder);
        assert!(fourth.IsClipSet() && !fourth.IsAttachmentSet());
        apply(
            &mut b,
            CSSPropertyID::kBackgroundClip,
            "border-area text",
            None,
        )
        .unwrap();
        assert_eq!(
            unsafe { &*b.AccessBackgroundLayers().Next() }.Clip(),
            EFillBox::kBorderAreaText
        );
        apply(&mut b, CSSPropertyID::kBackgroundClip, "initial", None).unwrap();
        assert_eq!(b.AccessBackgroundLayers().Clip(), EFillBox::kBorder);
        assert!(!unsafe { &*b.AccessBackgroundLayers().Next() }.IsClipSet());
        let parent = unsafe { &*b.TakeStyle() };
        let mut child = ComputedStyleBuilder::from_style(parent);
        apply(
            &mut child,
            CSSPropertyID::kBackgroundAttachment,
            "initial",
            None,
        )
        .unwrap();
        assert_eq!(
            parent.BackgroundLayers().Attachment(),
            EFillAttachment::kFixed
        );
        apply(
            &mut child,
            CSSPropertyID::kBackgroundAttachment,
            "inherit",
            Some(parent),
        )
        .unwrap();
        assert_eq!(
            unsafe { &*child.AccessBackgroundLayers().Next() }.Attachment(),
            EFillAttachment::kLocal
        );
        assert!(
            !unsafe { &*(*(*child.AccessBackgroundLayers().Next()).Next()).Next() }
                .IsAttachmentSet()
        );
    }

    #[test]
    fn native_position_points_preserve_percentages_and_far_edge_calculations() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut b = ComputedStyleBuilder::from_style(initial());
        apply(
            &mut b,
            CSSPropertyID::kObjectPosition,
            "right 2px bottom 30%",
            None,
        )
        .unwrap();
        let point = b.ObjectPosition();
        assert!(point.X().IsCalculated());
        let calculation = point.X().GetCalculationValue();
        assert_eq!(calculation.Pixels(), -2.0);
        assert_eq!(calculation.Percent(), 100.0);
        assert_eq!(point.Y().PercentValue(), 70.0);
        apply(&mut b, CSSPropertyID::kPerspectiveOrigin, "top left", None).unwrap();
        assert!(
            b.PerspectiveOrigin() == &LengthPoint::new(&Length::Percent(0), &Length::Percent(0))
        );
        apply(&mut b, CSSPropertyID::kOffsetAnchor, "auto", None).unwrap();
        assert!(b.OffsetAnchor().X().IsAuto() && b.OffsetAnchor().Y().IsAuto());
        apply(&mut b, CSSPropertyID::kOffsetPosition, "normal", None).unwrap();
        assert!(b.OffsetPosition().X().IsNone() && b.OffsetPosition().Y().IsNone());
        apply(&mut b, CSSPropertyID::kOffsetPosition, "20% 3px", None).unwrap();
        assert_eq!(b.OffsetPosition().X().PercentValue(), 20.0);
        assert_eq!(b.OffsetPosition().Y().Pixels(), 3.0);
        apply(
            &mut b,
            CSSPropertyID::kTransformOrigin,
            "left top -4px",
            None,
        )
        .unwrap();
        assert_eq!(b.GetTransformOrigin().X().PercentValue(), 0.0);
        assert_eq!(b.GetTransformOrigin().Y().PercentValue(), 0.0);
        assert_eq!(b.GetTransformOrigin().Z(), -4.0);
        apply(&mut b, CSSPropertyID::kTransformOrigin, "center", None).unwrap();
        assert_eq!(b.GetTransformOrigin().X().PercentValue(), 50.0);
        assert_eq!(b.GetTransformOrigin().Z(), 0.0);
    }
    #[test]
    fn fill_layers_keep_origins_and_clear_only_remaining_property_fields() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut b = ComputedStyleBuilder::from_style(initial());
        apply(
            &mut b,
            CSSPropertyID::kBackgroundPosition,
            "right 2px bottom 3px, left 7px top 8px",
            None,
        )
        .unwrap();
        apply(
            &mut b,
            CSSPropertyID::kBackgroundRepeat,
            "repeat-x, round space, no-repeat",
            None,
        )
        .unwrap();
        {
            let first = b.AccessBackgroundLayers();
            assert_eq!(first.PositionX().Pixels(), 2.0);
            assert_eq!(first.PositionY().Pixels(), 3.0);
            assert_eq!(first.BackgroundXOrigin(), BackgroundEdgeOrigin::kRight);
            assert_eq!(first.BackgroundYOrigin(), BackgroundEdgeOrigin::kBottom);
            assert!(first.IsBackgroundXOriginSet() && first.IsBackgroundYOriginSet());
            assert_eq!(first.Repeat().x, EFillRepeat::kRepeatFill);
            assert_eq!(first.Repeat().y, EFillRepeat::kNoRepeatFill);
            let second = unsafe { &*first.Next() };
            assert_eq!(second.PositionX().Pixels(), 7.0);
            assert_eq!(second.Repeat().x, EFillRepeat::kRoundFill);
            assert_eq!(second.Repeat().y, EFillRepeat::kSpaceFill);
            assert!(!second.Next().is_null());
        }
        apply(&mut b, CSSPropertyID::kBackgroundPositionX, "center", None).unwrap();
        let first = b.AccessBackgroundLayers();
        assert_eq!(first.PositionX().PercentValue(), 50.0);
        assert!(!first.IsBackgroundXOriginSet());
        let second = unsafe { &*first.Next() };
        assert!(!second.IsPositionXSet() && !second.IsBackgroundXOriginSet());
        assert!(second.IsPositionYSet() && second.IsRepeatSet());
        apply(&mut b, CSSPropertyID::kBackgroundRepeat, "initial", None).unwrap();
        let first = b.AccessBackgroundLayers();
        assert_eq!(first.Repeat().x, EFillRepeat::kRepeatFill);
        assert!(!unsafe { &*first.Next() }.IsRepeatSet());
        apply(
            &mut b,
            CSSPropertyID::kMaskPosition,
            "right 4px bottom 5px",
            None,
        )
        .unwrap();
        apply(&mut b, CSSPropertyID::kMaskRepeat, "repeat-y, space", None).unwrap();
        let mask = b.AccessMaskLayers();
        assert_eq!(mask.PositionX().Pixels(), 4.0);
        assert_eq!(mask.BackgroundXOrigin(), BackgroundEdgeOrigin::kRight);
        assert_eq!(mask.Repeat().x, EFillRepeat::kNoRepeatFill);
        assert_eq!(unsafe { &*mask.Next() }.Repeat().y, EFillRepeat::kSpaceFill);
    }
    #[test]
    fn initial_inheritance_and_copy_on_write_use_native_storage() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut parent = ComputedStyleBuilder::from_style(initial());
        apply(
            &mut parent,
            CSSPropertyID::kObjectPosition,
            "left 5px bottom 25%",
            None,
        )
        .unwrap();
        apply(
            &mut parent,
            CSSPropertyID::kBackgroundPosition,
            "right 2px bottom 3px, left top",
            None,
        )
        .unwrap();
        apply(
            &mut parent,
            CSSPropertyID::kBackgroundRepeat,
            "repeat-x, round space",
            None,
        )
        .unwrap();
        let parent = unsafe { &*parent.TakeStyle() };
        let mut child = ComputedStyleBuilder::from_style(parent);
        apply(
            &mut child,
            CSSPropertyID::kBackgroundRepeat,
            "no-repeat",
            None,
        )
        .unwrap();
        assert_eq!(
            parent.BackgroundLayers().Repeat().x,
            EFillRepeat::kRepeatFill
        );
        assert!(unsafe { &*parent.BackgroundLayers().Next() }.IsRepeatSet());
        assert!(!unsafe { &*child.AccessBackgroundLayers().Next() }.IsRepeatSet());
        apply(
            &mut child,
            CSSPropertyID::kBackgroundRepeat,
            "inherit",
            Some(parent),
        )
        .unwrap();
        let second = unsafe { &*child.AccessBackgroundLayers().Next() };
        assert_eq!(second.Repeat().x, EFillRepeat::kRoundFill);
        assert_eq!(second.Repeat().y, EFillRepeat::kSpaceFill);
        apply(
            &mut child,
            CSSPropertyID::kBackgroundPositionX,
            "inherit",
            Some(parent),
        )
        .unwrap();
        assert_eq!(
            child.AccessBackgroundLayers().BackgroundXOrigin(),
            BackgroundEdgeOrigin::kRight
        );
        apply(
            &mut child,
            CSSPropertyID::kObjectPosition,
            "inherit",
            Some(parent),
        )
        .unwrap();
        assert!(child.ObjectPosition() == parent.ObjectPosition());
        apply(
            &mut child,
            CSSPropertyID::kObjectPosition,
            "unset",
            Some(parent),
        )
        .unwrap();
        assert!(child.ObjectPosition() == initial().ObjectPosition());
        apply(&mut child, CSSPropertyID::kOffsetAnchor, "inherit", None).unwrap();
        assert!(child.OffsetAnchor().X().IsAuto());
    }
    #[test]
    fn unsupported_length_resolution_is_explicit_and_does_not_partially_write() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut b = ComputedStyleBuilder::from_style(initial());
        apply(&mut b, CSSPropertyID::kObjectPosition, "10% 20%", None).unwrap();
        let before = b.ObjectPosition().clone();
        assert_eq!(
            apply(&mut b, CSSPropertyID::kObjectPosition, "left 2ex", None),
            Err(LonghandApplicationError::Unsupported(
                CSSPropertyID::kObjectPosition
            ))
        );
        assert!(b.ObjectPosition() == &before);
        apply(
            &mut b,
            CSSPropertyID::kBackgroundPositionX,
            "right 2px",
            None,
        )
        .unwrap();
        let before = b.AccessBackgroundLayers().PositionX().clone();
        assert_eq!(
            apply(
                &mut b,
                CSSPropertyID::kBackgroundPositionX,
                "3px, 2ex",
                None
            ),
            Err(LonghandApplicationError::Unsupported(
                CSSPropertyID::kBackgroundPositionX
            ))
        );
        assert!(b.AccessBackgroundLayers().PositionX() == &before);
        assert!(b.AccessBackgroundLayers().Next().is_null());
        b.SetEffectiveZoom(2.0);
        assert_eq!(
            apply(&mut b, CSSPropertyID::kObjectPosition, "1px", None),
            Err(LonghandApplicationError::Unsupported(
                CSSPropertyID::kObjectPosition
            ))
        );
        assert!(apply(&mut b, CSSPropertyID::kObjectPosition, "20%", None).is_ok());
    }
}
