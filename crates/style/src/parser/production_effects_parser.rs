// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Typed stable filter, polygon/box, cursor and shape threshold consumers.
// Source audit is scoped to the direct entrypoints, filter resolver, polygon
// branch/value and geometry-box owner below. Nested math/color/image/length and
// other BasicShape collaborator bodies are NOT included; pending ledger entries
// cover them. Native polygon data is installed; GetPath is still unimplemented.
// Effective lines exclude comments/blanks/preprocessing/namespaces/using,
// access labels and punctuation-only lines; count multiline statements by line.
// Scope                         effective mapped omitted remaining
// Parser entrypoints/helpers           298    237      10        51
// Native application/converters        286    219      15        52
// Polygon value/geometry owner         108     88      20         0
// Total selected scope                692    544      45       103
// Parser: longhands_custom.cc:2276-2311,3386-3463,9266-9308;
// generated longhands.cc:2995-3000,7932-7937,15408-15413;
// css_parsing_utils.cc:546-593,813-888,1445-1450,3930-3933,3949-3954,
// 3994-4018,9170-9217. Remaining ranges: custom:3391-3429,3448,3461-3462;
// utilities:562-564,9187-9190,9193-9206. Omitted ranges: custom:3432-3440;
// utilities:838,9212 (use counters).
// Application: generated:3002-3012,5936-5949,7939-7949,15415-15423,
// 15478-15491; custom:925-931,3490-3495,3497-3502,3504-3526,4170-4175;
// style_builder_converter.cc:352-396,2750-2782;
// filter_operation_resolver.cc:47-73,128-170,174-190,192-273;
// basic_shape_functions.cc:636-653. Remaining: generated:3006-3010,
// 5940-5944,7943-7947,15482-15486; custom:3509-3520;
// converter:379-382,2757-2761; filter resolver:174-190,207-208,212-218.
// Omitted: converter:362-365,373,385-386,2753,2780;
// filter resolver:71,199,209-210,224-225,268 (metrics/diagnostics).
// Value/native: css_basic_shape_values.h:108-156; .cc:237-327;
// geometry_box_clip_path_operation.h:13-40. Omitted value .cc:240,
// 255-275,322-327 (DCHECK, allocation capacity, Rc-managed ownership trace).
// Stable defaults: CSSPolygonRounding, CSSShapeOutsidePathAndShapeSupport,
// CSSShapeOutsideRectAndXywhSupport=true; BasicShapeCornerRadius=false.
// Alternate flag branches, resource owners and cross-zoom reapplication remain
// in the ledgers. No property is declared complete by this selected-scope audit.
#![allow(non_snake_case)]
use super::*;
use crate::{
    css_math_expression_node::CalculationResultCategory as C,
    css_math_function_value::ValueRange as R,
};
use CSSValueID::*;
pub(super) fn IsEffectsProperty(id: CSSPropertyID) -> bool {
    matches!(
        id,
        CSSPropertyID::kFilter
            | CSSPropertyID::kBackdropFilter
            | CSSPropertyID::kClipPath
            | CSSPropertyID::kCursor
            | CSSPropertyID::kShapeOutside
            | CSSPropertyID::kShapeImageThreshold
    )
}
fn LengthValue<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
    percent: bool,
    nonnegative: bool,
) -> Result<Rc<Value>, PropertyParseError> {
    if IsMathFunction(stream) {
        return ConsumeMath(
            id,
            stream,
            if percent {
                &[C::Length, C::Percent, C::LengthFunction]
            } else {
                &[C::Length]
            },
            if nonnegative { R::NonNegative } else { R::All },
        );
    }
    ConsumeLiteral(
        id,
        stream,
        mode,
        Grammar::Length {
            percent,
            nonnegative,
            quirks: false,
            keywords: &[],
        },
    )
}
fn NumberValue<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    if IsMathFunction(stream) {
        return ConsumeMath(id, stream, &[C::Number, C::Percent], R::NonNegative);
    }
    if stream.Peek().GetType() == kPercentageToken {
        let number = stream.ConsumeIncludingWhitespace().NumericValue();
        if number < 0.0 {
            return Err(invalid(id));
        }
        return Ok(values::numeric(number, UnitType::kPercentage));
    }
    ConsumeLiteral(id, stream, mode, Grammar::Number { nonnegative: true })
}
// cpp: css_parsing_utils.cc:813-890,3994-4018.
fn Filter<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    if stream.Peek().Id() == kNone {
        stream.ConsumeIncludingWhitespace();
        return Ok(values::identifier(kNone));
    }
    let mut filters = Vec::new();
    loop {
        if stream.Peek().GetType() == kUrlToken || stream.Peek().FunctionId() == Some(kUrl) {
            filters.push(values::uri(
                ConsumeUrl(id, stream)?.ok_or_else(|| invalid(id))?,
            ));
            continue;
        }
        let Some(function) = stream.Peek().FunctionId() else {
            break;
        };
        if !matches!(
            function,
            kGrayscale
                | kSepia
                | kSaturate
                | kHueRotate
                | kInvert
                | kOpacity
                | kBrightness
                | kContrast
                | kBlur
                | kDropShadow
        ) {
            break;
        }
        let mut arguments = Vec::new();
        {
            let mut guard = RestoringBlockGuard::new(stream);
            guard.ConsumeWhitespace();
            if function == kDropShadow {
                let shadow = ConsumeShadow(id, &mut guard, mode)?;
                let CSSValuePayload::kValueListClass(list) = shadow.Payload() else {
                    return Err(invalid(id));
                };
                if list.values.len() != 1 {
                    return Err(invalid(id));
                }
                arguments.push(list.values[0].clone());
            } else if !guard.AtEnd() {
                let mut value = match function {
                    kBlur => LengthValue(
                        id,
                        &mut guard,
                        CSSParserMode::kHTMLStandardMode,
                        false,
                        true,
                    )?,
                    kHueRotate => {
                        if IsMathFunction(&mut guard) {
                            ConsumeMath(id, &mut guard, &[C::Angle], R::All)?
                        } else {
                            let token = guard.Peek();
                            let unit = token.GetUnitType();
                            if token.GetType() == kNumberToken && token.NumericValue() == 0.0 {
                                guard.ConsumeIncludingWhitespace();
                                values::numeric(0.0, UnitType::kDegrees)
                            } else if token.GetType() == kDimensionToken
                                && matches!(
                                    unit,
                                    UnitType::kDegrees
                                        | UnitType::kRadians
                                        | UnitType::kGradians
                                        | UnitType::kTurns
                                )
                            {
                                values::numeric(
                                    guard.ConsumeIncludingWhitespace().NumericValue(),
                                    unit,
                                )
                            } else {
                                return Err(invalid(id));
                            }
                        }
                    }
                    _ => NumberValue(id, &mut guard, mode)?,
                };
                if matches!(function, kGrayscale | kSepia | kInvert | kOpacity) {
                    if let CSSValuePayload::kNumericLiteralClass(number) = value.Payload() {
                        let maximum = if number.GetType() == UnitType::kPercentage {
                            100.0
                        } else {
                            1.0
                        };
                        if number.DoubleValue() > maximum {
                            value = values::numeric(maximum, number.GetType());
                        }
                    }
                }
                arguments.push(value);
            }
            if !guard.AtEnd() {
                return Err(invalid(id));
            }
            guard.Release();
        }
        stream.ConsumeWhitespace();
        filters.push(values::function(function, arguments));
    }
    if filters.is_empty() {
        return Err(invalid(id));
    }
    Ok(values::list(filters, values::ListSeparator::Space))
}
// cpp: css_parsing_utils.cc:546-592,9170-9214. Polygon rounding is stable.
pub(super) fn Polygon<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    let mut guard = RestoringBlockGuard::new(stream);
    guard.ConsumeWhitespace();
    let mut wind_rule = foundation::WindRule::RULE_NONZERO;
    let mut prefix = false;
    if matches!(guard.Peek().Id(), kEvenodd | kNonzero) {
        wind_rule = if guard.ConsumeIncludingWhitespace().Id() == kEvenodd {
            foundation::WindRule::RULE_EVENODD
        } else {
            wind_rule
        };
        prefix = true;
    }
    let rounding_radius = if guard.Peek().Id() == kRound {
        guard.ConsumeIncludingWhitespace();
        prefix = true;
        Some(LengthValue(id, &mut guard, mode, false, true)?)
    } else {
        None
    };
    if prefix {
        if guard.Peek().GetType() != kCommaToken {
            return Err(invalid(id));
        }
        guard.ConsumeIncludingWhitespace();
    }
    let mut coordinates = Vec::new();
    loop {
        coordinates.push(LengthValue(id, &mut guard, mode, true, false)?);
        coordinates.push(LengthValue(id, &mut guard, mode, true, false)?);
        if guard.Peek().GetType() != kCommaToken {
            break;
        }
        guard.ConsumeIncludingWhitespace();
    }
    if !guard.AtEnd() {
        return Err(invalid(id));
    }
    guard.Release();
    drop(guard);
    stream.ConsumeWhitespace();
    Ok(Rc::new(Value::new(
        CSSValuePayload::kBasicShapePolygonClass(
            crate::production_effects_value::CSSBasicShapePolygonValue {
                wind_rule,
                rounding_radius,
                coordinates,
            },
        ),
    )))
}
fn BoxValue<T: TokenStreamTokenizer>(stream: &mut Stream<T>, geometry: bool) -> Option<Rc<Value>> {
    let keyword = stream.Peek().Id();
    if matches!(keyword, kContentBox | kPaddingBox | kBorderBox | kMarginBox)
        || geometry && matches!(keyword, kFillBox | kStrokeBox | kViewBox)
    {
        stream.ConsumeIncludingWhitespace();
        Some(values::identifier(keyword))
    } else {
        None
    }
}
// cpp: longhands_custom.cc:2276-2311,9266-9308.
fn Shape<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    let clip = id == CSSPropertyID::kClipPath;
    if stream.Peek().Id() == kNone {
        stream.ConsumeIncludingWhitespace();
        return Ok(values::identifier(kNone));
    }
    if stream.Peek().GetType() == kUrlToken || stream.Peek().FunctionId() == Some(kUrl) {
        return if clip {
            Ok(values::uri(
                ConsumeUrl(id, stream)?.ok_or_else(|| invalid(id))?,
            ))
        } else {
            ConsumeContentImage(id, stream)?.ok_or_else(|| invalid(id))
        };
    }
    if !clip
        && matches!(
            stream.Peek().FunctionId(),
            Some(kLinearGradient | kRepeatingLinearGradient | kImageSet)
        )
    {
        return Err(unsupported(
            id,
            "ShapeOutside generated image StyleImage resource adapter",
        ));
    }
    let mut box_value = BoxValue(stream, clip);
    let function = stream.Peek().FunctionId();
    let shape = if function == Some(kPolygon) {
        Some(Polygon(id, stream, mode)?)
    } else if matches!(
        function,
        Some(kCircle | kEllipse | kInset | kRect | kXywh | kPath | kShape)
    ) {
        return Err(unsupported(
            id,
            "BasicShape native Path/equality owner or typed shape subtype",
        ));
    } else {
        None
    };
    if shape.is_some() && box_value.is_none() {
        box_value = BoxValue(stream, clip);
    }
    if shape.is_none() && box_value.is_none() {
        return Err(invalid(id));
    }
    let mut items = Vec::new();
    if let Some(shape) = shape {
        items.push(shape);
    }
    if let Some(box_value) = box_value {
        let CSSValuePayload::kIdentifierClass(keyword) = box_value.Payload() else {
            unreachable!()
        };
        if items.is_empty() || keyword.0 != if clip { kBorderBox } else { kMarginBox } {
            items.push(box_value);
        }
    }
    Ok(values::list(items, values::ListSeparator::Space))
}
// cpp: longhands_custom.cc:3386-3463 (image branch requires its resource owner).
fn Cursor<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    if stream.Peek().GetType() == kUrlToken
        || stream.Peek().FunctionId() == Some(kUrl)
        || stream.Peek().FunctionId() == Some(kImageSet)
    {
        return Err(unsupported(
            id,
            "CSSCursorImageValue/StyleImage resource adapter",
        ));
    }
    let mut keyword = stream.Peek().Id();
    if keyword == kHand {
        if mode != CSSParserMode::kHTMLQuirksMode {
            return Err(invalid(id));
        }
        keyword = kPointer;
    } else if !((kAuto as i32..=kWebkitZoomOut as i32).contains(&(keyword as i32))
        || matches!(keyword, kCopy | kNone))
    {
        return Err(invalid(id));
    }
    stream.ConsumeIncludingWhitespace();
    Ok(values::identifier(keyword))
}
pub(super) fn Consume<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    match id {
        CSSPropertyID::kFilter | CSSPropertyID::kBackdropFilter => Filter(id, stream, mode),
        CSSPropertyID::kClipPath | CSSPropertyID::kShapeOutside => Shape(id, stream, mode),
        CSSPropertyID::kCursor => Cursor(id, stream, mode),
        CSSPropertyID::kShapeImageThreshold => {
            let value = if IsMathFunction(stream) {
                ConsumeMath(id, stream, &[C::Number, C::Percent], R::All)?
            } else if stream.Peek().GetType() == kPercentageToken {
                values::numeric(
                    stream.ConsumeIncludingWhitespace().NumericValue(),
                    UnitType::kPercentage,
                )
            } else {
                ConsumeLiteral(id, stream, mode, Grammar::Number { nonnegative: false })?
            };
            Ok(value)
        }
        _ => Err(unsupported(id, "visual effect consumer")),
    }
}
