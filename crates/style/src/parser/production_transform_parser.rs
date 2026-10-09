// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Typed Transform/Translate/Rotate/Scale/Perspective consumers.
#![allow(non_snake_case)]
use super::*;
use crate::css_math_expression_node::CalculationResultCategory as C;
use crate::css_math_function_value::ValueRange as R;

pub(super) fn IsTransformProperty(id: CSSPropertyID) -> bool {
    matches!(
        id,
        CSSPropertyID::kTransform
            | CSSPropertyID::kTranslate
            | CSSPropertyID::kRotate
            | CSSPropertyID::kScale
            | CSSPropertyID::kPerspective
    )
}
#[derive(Clone, Copy)]
enum Argument {
    Number,
    NumberOrPercent,
    Angle(bool),
    Length(bool, bool),
}
fn ConsumeArgument<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
    argument: Argument,
) -> Result<Rc<Value>, PropertyParseError> {
    if IsMathFunction(stream) {
        let (categories, range): (&[C], R) = match argument {
            Argument::Number => (&[C::Number], R::All),
            Argument::NumberOrPercent => (&[C::Number, C::Percent], R::All),
            Argument::Angle(_) => (&[C::Angle], R::All),
            Argument::Length(true, false) => (&[C::Length, C::Percent, C::LengthFunction], R::All),
            Argument::Length(true, true) => {
                (&[C::Length, C::Percent, C::LengthFunction], R::NonNegative)
            }
            Argument::Length(false, false) => (&[C::Length], R::All),
            Argument::Length(false, true) => (&[C::Length], R::NonNegative),
        };
        return ConsumeMath(id, stream, categories, range);
    }
    match argument {
        Argument::Number => {
            ConsumeLiteral(id, stream, mode, Grammar::Number { nonnegative: false })
        }
        Argument::NumberOrPercent if stream.Peek().GetType() == kPercentageToken => {
            Ok(values::numeric(
                stream.ConsumeIncludingWhitespace().NumericValue() / 100.0,
                UnitType::kNumber,
            ))
        }
        Argument::NumberOrPercent => ConsumeArgument(id, stream, mode, Argument::Number),
        Argument::Length(percent, nonnegative) => ConsumeLiteral(
            id,
            stream,
            mode,
            Grammar::Length {
                percent,
                nonnegative,
                quirks: false,
                keywords: &[],
            },
        ),
        Argument::Angle(zero) => {
            let token = stream.Peek();
            if token.GetType() == kNumberToken && zero && token.NumericValue() == 0.0 {
                stream.ConsumeIncludingWhitespace();
                return Ok(values::numeric(0.0, UnitType::kDegrees));
            }
            let unit = token.GetUnitType();
            if token.GetType() != kDimensionToken
                || !matches!(
                    unit,
                    UnitType::kDegrees
                        | UnitType::kRadians
                        | UnitType::kGradians
                        | UnitType::kTurns
                )
            {
                return Err(invalid(id));
            }
            Ok(values::numeric(
                stream.ConsumeIncludingWhitespace().NumericValue(),
                unit,
            ))
        }
    }
}
// ConsumeAngle optional probe reused by motion path consumers. A math
// value of a different category restores the stream, preserving distance probes.
pub(super) fn ConsumeAngle<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Option<Rc<Value>>, PropertyParseError> {
    if IsMathFunction(s) {
        return ConsumeAnimationNumericMath(id, s, C::Angle, R::All);
    }
    if !MatchesArgument(s, Argument::Angle(false)) {
        return Ok(None);
    }
    ConsumeArgument(id, s, mode, Argument::Angle(false)).map(Some)
}
fn MatchesArgument<T: TokenStreamTokenizer>(stream: &mut Stream<T>, argument: Argument) -> bool {
    if IsMathFunction(stream) {
        return true;
    }
    let token = stream.Peek();
    match argument {
        Argument::Number => token.GetType() == kNumberToken,
        Argument::NumberOrPercent => matches!(token.GetType(), kNumberToken | kPercentageToken),
        Argument::Angle(zero) => {
            token.GetType() == kDimensionToken
                && matches!(
                    token.GetUnitType(),
                    UnitType::kDegrees
                        | UnitType::kRadians
                        | UnitType::kGradians
                        | UnitType::kTurns
                )
                || zero && token.GetType() == kNumberToken && token.NumericValue() == 0.0
        }
        Argument::Length(percent, _) => {
            token.GetType() == kDimensionToken
                && crate::css_numeric_literal_value::IsLength(token.GetUnitType())
                || percent && token.GetType() == kPercentageToken
                || token.GetType() == kNumberToken && token.NumericValue() == 0.0
        }
    }
}
// cpp: css_parsing_utils.cc:746-772; longhands_custom.cc:8248-8273.
fn ConsumePerspective<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
    alias: bool,
) -> Result<Rc<Value>, PropertyParseError> {
    if stream.Peek().Id() == CSSValueID::kNone {
        return Ok(values::identifier(stream.ConsumeIncludingWhitespace().Id()));
    }
    if alias && (stream.Peek().GetType() == kNumberToken && stream.Peek().NumericValue() != 0.0) {
        let n = stream.Peek().NumericValue();
        if n < 0.0 {
            return Err(invalid(id));
        }
        stream.ConsumeIncludingWhitespace();
        return Ok(values::numeric(n, UnitType::kPixels));
    }
    if alias && IsMathFunction(stream) {
        let value = ConsumeMath(id, stream, &[C::Length, C::Number], R::NonNegative)?;
        let CSSValuePayload::kMathFunctionClass(math) = value.Payload() else {
            unreachable!()
        };
        if math.Category() == C::Number {
            // CSSMathFunctionValue::GetValueIfKnown, with no relative-length
            // context. The core typed arithmetic admits constant number trees.
            let mut no_lengths = |_: f64, _: UnitType| {
                Err(crate::css_math_expression_node::MathError::MissingLengthContext)
            };
            let known = math.ComputeValue(&mut no_lengths, None).map_err(|_| {
                unsupported(id, "ConsumePerspective GetValueIfKnown length context")
            })?;
            return Ok(values::numeric(
                crate::css_value_clamping_utils::CSSValueClampingUtils::ClampDouble(known),
                UnitType::kPixels,
            ));
        }
        return Ok(value);
    }
    ConsumeArgument(id, stream, mode, Argument::Length(false, true))
}
// cpp: css_parsing_utils.cc:9446-9608. CSS arguments require commas.
fn ConsumeTransformFunction<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
    alias: bool,
) -> Result<Rc<Value>, PropertyParseError> {
    use CSSValueID::*;
    let function = stream.Peek().FunctionId().ok_or_else(|| invalid(id))?;
    let (arguments, optional): (Vec<Argument>, bool) = match function {
        kRotate | kRotateX | kRotateY | kRotateZ | kSkewX | kSkewY => {
            (vec![Argument::Angle(true)], false)
        }
        kSkew => (vec![Argument::Angle(true), Argument::Angle(true)], true),
        kScaleX | kScaleY | kScaleZ => (vec![Argument::NumberOrPercent], false),
        kScale => (vec![Argument::NumberOrPercent; 2], true),
        kScale3d => (vec![Argument::NumberOrPercent; 3], false),
        kTranslateX | kTranslateY => (vec![Argument::Length(true, false)], false),
        kTranslate => (vec![Argument::Length(true, false); 2], true),
        kTranslateZ => (vec![Argument::Length(false, false)], false),
        kTranslate3d => (
            vec![
                Argument::Length(true, false),
                Argument::Length(true, false),
                Argument::Length(false, false),
            ],
            false,
        ),
        kRotate3d => (
            vec![
                Argument::Number,
                Argument::Number,
                Argument::Number,
                Argument::Angle(true),
            ],
            false,
        ),
        kMatrix => (vec![Argument::Number; 6], false),
        kMatrix3d => (vec![Argument::Number; 16], false),
        kPerspective => (vec![], false),
        _ => return Err(invalid(id)),
    };
    let mut values_ = Vec::new();
    {
        let mut guard = RestoringBlockGuard::new(stream);
        guard.ConsumeWhitespace();
        if guard.AtEnd() {
            return Err(invalid(id));
        }
        if function == kPerspective {
            values_.push(ConsumePerspective(id, &mut guard, mode, alias)?);
        } else {
            for (index, argument) in arguments.iter().enumerate() {
                if index > 0 {
                    if guard.Peek().GetType() != kCommaToken {
                        if optional {
                            break;
                        }
                        return Err(invalid(id));
                    }
                    guard.ConsumeIncludingWhitespace();
                }
                values_.push(ConsumeArgument(id, &mut guard, mode, *argument)?);
            }
        }
        if !guard.AtEnd() {
            return Err(invalid(id));
        }
        guard.Release();
    }
    stream.ConsumeWhitespace();
    Ok(values::function(function, values_))
}
fn Known(value: &Value) -> Option<f64> {
    match value.Payload() {
        CSSValuePayload::kNumericLiteralClass(n) => Some(n.DoubleValue()),
        _ => None,
    }
}
fn HasPercentage(value: &Value) -> bool {
    match value.Payload() {
        CSSValuePayload::kNumericLiteralClass(n) => n.GetType() == UnitType::kPercentage,
        CSSValuePayload::kMathFunctionClass(m) => {
            matches!(m.Category(), C::Percent | C::LengthFunction)
        }
        _ => false,
    }
}
// cpp: longhands_custom.cc:8525-8590; css_parsing_utils.cc:3434-3459.
fn ConsumeRotate<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    let mut angle = if MatchesArgument(stream, Argument::Angle(false)) && !IsMathFunction(stream) {
        Some(ConsumeArgument(id, stream, mode, Argument::Angle(false))?)
    } else {
        None
    };
    // Math category must be determined by the shared typed expression parser.
    if angle.is_none() && IsMathFunction(stream) {
        let save = stream.Save();
        let value = ConsumeMath(id, stream, &[C::Angle, C::Number], R::All)?;
        if matches!(value.Payload(), CSSValuePayload::kMathFunctionClass(math) if math.Category() == C::Angle)
        {
            angle = Some(value);
        } else {
            stream.Peek();
            stream.Restore(save);
        }
    }
    let mut dimensions = Vec::new();
    let named = stream.Peek().Id();
    if matches!(named, CSSValueID::kX | CSSValueID::kY | CSSValueID::kZ) {
        stream.ConsumeIncludingWhitespace();
        for axis in [CSSValueID::kX, CSSValueID::kY, CSSValueID::kZ] {
            dimensions.push(values::numeric(
                if axis == named { 1.0 } else { 0.0 },
                UnitType::kNumber,
            ));
        }
    } else if MatchesArgument(stream, Argument::Number) {
        for _ in 0..3 {
            dimensions.push(ConsumeArgument(id, stream, mode, Argument::Number)?);
        }
    }
    let mut negative_axis = false;
    if let Some(numbers) = dimensions
        .iter()
        .map(|v| Known(v))
        .collect::<Option<Vec<_>>>()
        .filter(|n| n.len() == 3)
    {
        for index in 0..3 {
            if numbers[index] < 0.0 && (0..3).all(|other| other == index || numbers[other] == 0.0) {
                dimensions[index] = values::numeric(1.0, UnitType::kNumber);
                negative_axis = true;
                break;
            }
        }
    }
    if angle.is_none() {
        angle = Some(ConsumeArgument(id, stream, mode, Argument::Angle(false))?);
    }
    let mut angle = angle.unwrap();
    if negative_axis {
        angle = match angle.Payload() {
            CSSValuePayload::kNumericLiteralClass(n) => {
                values::numeric(-n.DoubleValue(), n.GetType())
            }
            CSSValuePayload::kMathFunctionClass(m) => values::math(
                crate::css_math_expression_node::CSSMathExpressionNode::Operation {
                    operator: crate::css_math_expression_node::CSSMathOperator::Multiply,
                    operands: vec![
                        m.expression.clone(),
                        crate::css_math_expression_node::CSSMathExpressionNode::Numeric(
                            crate::css_numeric_literal_value::CSSNumericLiteralValue::Create(
                                -1.0,
                                UnitType::kNumber,
                            ),
                        ),
                    ],
                    category: C::Angle,
                },
                R::All,
            ),
            _ => return Err(invalid(id)),
        };
    }
    let mut list = Vec::new();
    if !dimensions.is_empty() {
        let axis = values::CSSAxisValue::new(dimensions);
        if axis.axis_name != CSSValueID::kZ {
            list.push(Rc::new(Value::new(CSSValuePayload::kAxisClass(axis))));
        }
    }
    list.push(angle);
    Ok(values::list(list, values::ListSeparator::Space))
}
pub(super) fn Consume<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
    alias: bool,
) -> Result<Rc<Value>, PropertyParseError> {
    if stream.Peek().Id() == CSSValueID::kNone {
        return Ok(values::identifier(stream.ConsumeIncludingWhitespace().Id()));
    }
    if id == CSSPropertyID::kPerspective {
        return ConsumePerspective(id, stream, mode, alias);
    }
    if id == CSSPropertyID::kRotate {
        return ConsumeRotate(id, stream, mode);
    }
    if id == CSSPropertyID::kTransform {
        let mut functions = Vec::new();
        loop {
            if stream.Peek().GetType() != kFunctionToken {
                break;
            }
            functions.push(ConsumeTransformFunction(id, stream, mode, alias)?);
        }
        if functions.is_empty() {
            return Err(invalid(id));
        }
        return Ok(values::list(functions, values::ListSeparator::Space));
    }
    let arguments = if id == CSSPropertyID::kTranslate {
        [
            Argument::Length(true, false),
            Argument::Length(true, false),
            Argument::Length(false, false),
        ]
    } else {
        [Argument::NumberOrPercent; 3]
    };
    let x = ConsumeArgument(id, stream, mode, arguments[0])?;
    let mut list = vec![x.clone()];
    if MatchesArgument(stream, arguments[1]) {
        let y = ConsumeArgument(id, stream, mode, arguments[1])?;
        let mut z = if MatchesArgument(stream, arguments[2]) {
            Some(ConsumeArgument(id, stream, mode, arguments[2])?)
        } else {
            None
        };
        if id == CSSPropertyID::kTranslate {
            if z.as_ref().and_then(|v| Known(v)) == Some(0.0) {
                z = None;
            }
            if Known(&y) == Some(0.0) && !HasPercentage(&y) && z.is_none() {
                return Ok(values::list(list, values::ListSeparator::Space));
            }
            list.push(y);
            if let Some(z) = z {
                list.push(z);
            }
        } else if z.as_ref().map_or(false, |z| Known(z) != Some(1.0)) {
            list.push(y);
            list.push(z.unwrap());
        } else if Known(&x).is_none() || Known(&y).is_none() || Known(&x) != Known(&y) {
            list.push(y);
        }
    }
    Ok(values::list(list, values::ListSeparator::Space))
}
