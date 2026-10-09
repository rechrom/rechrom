// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Typed CSS calculations over the existing tokenizer/stream. Percentages stay
//! symbolic until a used-value basis is available; no CSS string evaluation.
#![allow(non_snake_case)]
use crate::{
    css_numeric_literal_value::{CSSNumericLiteralValue, IsLength},
    css_primitive_value::UnitType,
    parser::{
        css_parser_token::CSSParserTokenType::*,
        css_parser_token_stream::{BlockGuard, CSSParserTokenStream, TokenStreamTokenizer},
    },
};
use foundation::{
    CSSValueID, CalculationExpressionNode as CalculationNode, CalculationOperator as CalculationOp,
    PixelsAndPercent,
};
use std::sync::Arc;

// cpp: css_math_expression_node.cc:75-156,1570-1629.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CalculationResultCategory {
    Number,
    Length,
    Percent,
    LengthFunction,
    Angle,
    Time,
    Frequency,
    Resolution,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CSSMathOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Min,
    Max,
    Clamp,
    RoundNearest,
    RoundUp,
    RoundDown,
    RoundToZero,
    Mod,
    Rem,
    Hypot,
    Abs,
    Sign,
    Log,
    Exp,
    Sqrt,
    Pow,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Atan2,
}
#[derive(Clone, Debug, PartialEq)]
pub enum CSSMathExpressionNode {
    Numeric(CSSNumericLiteralValue),
    Operation {
        operator: CSSMathOperator,
        operands: Vec<Self>,
        category: CalculationResultCategory,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MathError {
    Invalid,
    UnsupportedFunction,
    UnsupportedTypedArithmetic,
    MissingLengthContext,
    MissingPercentageBasis,
    DepthLimit,
}
/// CSSLengthResolver boundary: callers provide actual zoom/font/viewport and
/// container conversion and record dependencies as each relative unit is read.
pub trait CSSMathLengthResolver {
    fn ComputeLength(&mut self, value: f64, unit: UnitType) -> Result<f64, MathError>;
}
impl<F: FnMut(f64, UnitType) -> Result<f64, MathError>> CSSMathLengthResolver for F {
    fn ComputeLength(&mut self, value: f64, unit: UnitType) -> Result<f64, MathError> {
        self(value, unit)
    }
}
pub fn UnitCategory(unit: UnitType) -> Option<CalculationResultCategory> {
    use CalculationResultCategory::*;
    use UnitType::*;
    Some(if IsLength(unit) || unit == kUserUnits {
        Length
    } else {
        match unit {
            kNumber | kInteger => Number,
            kPercentage => Percent,
            kDegrees | kRadians | kGradians | kTurns => Angle,
            kSeconds | kMilliseconds => Time,
            kHertz | kKilohertz => Frequency,
            kDotsPerPixel | kX | kDotsPerInch | kDotsPerCentimeter => Resolution,
            _ => return None,
        }
    })
}
fn AddCategory(
    a: CalculationResultCategory,
    b: CalculationResultCategory,
) -> Option<CalculationResultCategory> {
    use CalculationResultCategory::*;
    if a == b {
        Some(a)
    } else if matches!(a, Length | Percent | LengthFunction)
        && matches!(b, Length | Percent | LengthFunction)
    {
        Some(LengthFunction)
    } else {
        None
    }
}
impl CSSMathExpressionNode {
    pub fn Category(&self) -> CalculationResultCategory {
        match self {
            Self::Numeric(n) => UnitCategory(n.GetType()).expect("validated math unit"),
            Self::Operation { category, .. } => *category,
        }
    }
    fn Operation(operator: CSSMathOperator, operands: Vec<Self>) -> Result<Self, MathError> {
        use CSSMathOperator::*;
        use CalculationResultCategory::{Angle, Number};
        let tree_depth = 1 + operands.iter().map(Self::Depth).max().unwrap_or(0);
        if tree_depth > 100 {
            return Err(MathError::DepthLimit);
        }
        let a = operands.first().ok_or(MathError::Invalid)?.Category();
        let category = match operator {
            Add | Subtract | Min | Max | Clamp => operands
                .iter()
                .skip(1)
                .try_fold(a, |c, n| AddCategory(c, n.Category()))
                .ok_or(MathError::Invalid)?,
            Multiply => {
                let b = operands[1].Category();
                if a == Number {
                    b
                } else if b == Number {
                    a
                } else {
                    return Err(MathError::UnsupportedTypedArithmetic);
                }
            }
            Divide => {
                if operands[1].Category() != Number {
                    return Err(MathError::UnsupportedTypedArithmetic);
                }
                a
            }
            RoundNearest | RoundUp | RoundDown | RoundToZero | Mod | Rem => {
                if operands.len() != 2 {
                    return Err(MathError::Invalid);
                }
                AddCategory(a, operands[1].Category()).ok_or(MathError::Invalid)?
            }
            Hypot => operands
                .iter()
                .skip(1)
                .try_fold(a, |c, n| AddCategory(c, n.Category()))
                .ok_or(MathError::Invalid)?,
            Abs => {
                if operands.len() != 1 {
                    return Err(MathError::Invalid);
                }
                a
            }
            Sign => {
                if operands.len() != 1 {
                    return Err(MathError::Invalid);
                }
                Number
            }
            Log => {
                if !(1..=2).contains(&operands.len())
                    || operands.iter().any(|n| n.Category() != Number)
                {
                    return Err(MathError::Invalid);
                }
                Number
            }
            Exp | Sqrt => {
                if operands.len() != 1 || a != Number {
                    return Err(MathError::Invalid);
                }
                Number
            }
            Pow => {
                if operands.len() != 2 || operands.iter().any(|n| n.Category() != Number) {
                    return Err(MathError::Invalid);
                }
                Number
            }
            Sin | Cos | Tan => {
                if operands.len() != 1 || !matches!(a, Number | Angle) {
                    return Err(MathError::Invalid);
                }
                Number
            }
            Asin | Acos | Atan => {
                if operands.len() != 1 || a != Number {
                    return Err(MathError::Invalid);
                }
                Angle
            }
            Atan2 => {
                if operands.len() != 2 || operands[1].Category() != a {
                    return Err(MathError::Invalid);
                }
                Angle
            }
        };
        Ok(Self::Operation {
            operator,
            operands,
            category,
        })
    }
    fn Depth(&self) -> usize {
        match self {
            Self::Numeric(_) => 1,
            Self::Operation { operands, .. } => {
                1 + operands.iter().map(Self::Depth).max().unwrap_or(0)
            }
        }
    }
    pub fn CanonicalUnit(&self) -> Option<UnitType> {
        use CalculationResultCategory::*;
        Some(match self.Category() {
            Number => UnitType::kNumber,
            Length => UnitType::kPixels,
            Percent => UnitType::kPercentage,
            Angle => UnitType::kDegrees,
            Time => UnitType::kSeconds,
            Frequency => UnitType::kHertz,
            Resolution => UnitType::kDotsPerPixel,
            LengthFunction => return None,
        })
    }
    // cpp: css_math_expression_node.cc:1405-1455,3447-3559. Canonical numeric
    // conversion does not resolve mixed percentages without a supplied basis.
    pub fn ComputeValue(
        &self,
        resolver: &mut impl CSSMathLengthResolver,
        percentage_basis: Option<f64>,
    ) -> Result<f64, MathError> {
        let resolve_percent = self.HasMixedPercentageDependency();
        if resolve_percent && percentage_basis.is_none() {
            return Err(MathError::MissingPercentageBasis);
        }
        self.Evaluate(resolver, percentage_basis, resolve_percent)
    }
    // Chromium propagates percentage dependencies independently of the result
    // category: sign() returns Number and atan2() returns Angle even when their
    // operands still contain a mixed length/percentage computation.
    fn HasMixedPercentageDependency(&self) -> bool {
        self.Category() == CalculationResultCategory::LengthFunction
            || matches!(self, Self::Operation { operands, .. }
                if operands.iter().any(Self::HasMixedPercentageDependency))
    }
    fn HasPercentage(&self) -> bool {
        self.Category() == CalculationResultCategory::Percent
            || matches!(self, Self::Operation { operands, .. }
                if operands.iter().any(Self::HasPercentage))
    }
    // cpp: css_math_expression_node.h:445-447,819-821. Existing numeric and
    // operation owners query canonical value without a length context.
    pub fn GetValueIfKnown(&self) -> Option<f64> {
        self.ComputeValue(&mut |_, _| Err(MathError::MissingLengthContext), None)
            .ok()
    }
    fn Evaluate(
        &self,
        resolver: &mut impl CSSMathLengthResolver,
        basis: Option<f64>,
        resolve_percent: bool,
    ) -> Result<f64, MathError> {
        use CSSMathOperator::*;
        use UnitType::*;
        match self {
            Self::Numeric(n) => {
                let v = n.DoubleValue();
                Ok(match n.GetType() {
                    u if IsLength(u) || u == kUserUnits => resolver.ComputeLength(v, u)?,
                    kPercentage if resolve_percent => {
                        v * basis.ok_or(MathError::MissingPercentageBasis)? / 100.0
                    }
                    unit => {
                        v * crate::css_primitive_value::ConversionToCanonicalUnitsScaleFactor(unit)
                    }
                })
            }
            Self::Operation {
                operator, operands, ..
            } => {
                let values = operands
                    .iter()
                    .map(|n| n.Evaluate(resolver, basis, resolve_percent))
                    .collect::<Result<Vec<_>, _>>()?;
                // css_math_expression_node.cc:3707-3712: an operation with
                // any NaN argument must retain NaN through nested evaluation.
                if let Some(value) = values.iter().find(|value| value.is_nan()) {
                    return Ok(*value);
                }
                Ok(match operator {
                    Add => values[0] + values[1],
                    Subtract => values[0] - values[1],
                    Multiply => values[0] * values[1],
                    Divide => values[0] / values[1],
                    Min => values.into_iter().reduce(CSSMin).unwrap(),
                    Max => values.into_iter().reduce(CSSMax).unwrap(),
                    Clamp => CSSMax(values[0], CSSMin(values[1], values[2])),
                    RoundNearest | RoundUp | RoundDown | RoundToZero | Mod | Rem => {
                        EvaluateSteppedValueFunction(
                            CalculationOpFor(*operator),
                            values[0],
                            values[1],
                        )
                    }
                    Hypot => values.into_iter().fold(0.0, f64::hypot),
                    Abs => values[0].abs(),
                    Sign => EvaluateSignFunction(values[0]),
                    Log => {
                        if values.len() == 2 {
                            values[0].log2() / values[1].log2()
                        } else {
                            values[0].ln()
                        }
                    }
                    Exp => values[0].exp(),
                    Sqrt => values[0].sqrt(),
                    Pow => values[0].powf(values[1]),
                    Sin | Cos | Tan | Asin | Acos | Atan | Atan2 => {
                        let mut a = values[0];
                        if matches!(operator, Sin | Cos | Tan)
                            && operands[0].Category() == CalculationResultCategory::Number
                        {
                            a = a.to_degrees();
                        }
                        EvaluateTrigonometricFunction(
                            CalculationOpFor(*operator),
                            a,
                            (values.len() == 2).then(|| values[1]),
                        )
                    }
                })
            }
        }
    }
    // cpp: css_math_expression_node.cc:1375-1402,3133-3277. Platform's native
    // immutable calculation tree retains comparison functions and percentages.
    pub fn ToCalculationExpression(
        &self,
        resolver: &mut impl CSSMathLengthResolver,
    ) -> Result<Arc<CalculationNode>, MathError> {
        use CSSMathOperator::*;
        Ok(match self {
            Self::Numeric(n) => {
                let value = n.DoubleValue() as f32;
                Arc::new(match self.Category() {
                    CalculationResultCategory::Number => CalculationNode::Number(value),
                    CalculationResultCategory::Percent => CalculationNode::PixelsAndPercent(
                        PixelsAndPercent::new(0.0, value, false, true),
                    ),
                    CalculationResultCategory::Length => {
                        CalculationNode::PixelsAndPercent(PixelsAndPercent::from_pixels(
                            resolver.ComputeLength(n.DoubleValue(), n.GetType())? as f32,
                        ))
                    }
                    CalculationResultCategory::Angle => {
                        CalculationNode::PixelsAndPercent(PixelsAndPercent::from_pixels(
                            (n.DoubleValue()
                                * crate::css_primitive_value::ConversionToCanonicalUnitsScaleFactor(
                                    n.GetType(),
                                )) as f32,
                        ))
                    }
                    CalculationResultCategory::Time
                    | CalculationResultCategory::Frequency
                    | CalculationResultCategory::Resolution => CalculationNode::Number(
                        (n.DoubleValue()
                            * crate::css_primitive_value::ConversionToCanonicalUnitsScaleFactor(
                                n.GetType(),
                            )) as f32,
                    ),
                    CalculationResultCategory::LengthFunction => return Err(MathError::Invalid),
                })
            }
            Self::Operation {
                operator, operands, ..
            } => {
                // Chromium eagerly simplifies canonical scalar functions in
                // double before lowering. A tiny sign() or large atan2() input
                // must not underflow/overflow merely because this subtree is
                // inside a calculation with percentages.
                if !self.HasPercentage() {
                    if let Some(value) = self.GetValueIfKnown() {
                        let node = match self.Category() {
                            CalculationResultCategory::Number
                            | CalculationResultCategory::Time
                            | CalculationResultCategory::Frequency
                            | CalculationResultCategory::Resolution => {
                                Some(CalculationNode::Number(value as f32))
                            }
                            CalculationResultCategory::Angle => {
                                Some(CalculationNode::PixelsAndPercent(
                                    PixelsAndPercent::from_pixels(value as f32),
                                ))
                            }
                            _ => None,
                        };
                        if let Some(node) = node {
                            return Ok(Arc::new(node));
                        }
                    }
                }
                let mut nodes = operands
                    .iter()
                    .map(|n| n.ToCalculationExpression(resolver))
                    .collect::<Result<Vec<_>, _>>()?;
                let op = match operator {
                    Add => CalculationOp::kAdd,
                    Subtract => CalculationOp::kSubtract,
                    Multiply => CalculationOp::kMultiply,
                    Divide => {
                        nodes[1] = CalculationNode::CreateSimplified(
                            vec![nodes[1].clone()],
                            CalculationOp::kInvert,
                        );
                        CalculationOp::kMultiply
                    }
                    Min => CalculationOp::kMin,
                    Max => CalculationOp::kMax,
                    Clamp => CalculationOp::kClamp,
                    _ => CalculationOpFor(*operator),
                };
                CalculationNode::CreateSimplified(nodes, op)
            }
        })
    }
    pub fn CssText(&self) -> std::string::String {
        use CSSMathOperator::*;
        match self {
            Self::Numeric(n) => {
                crate::media_queries::media_query_exp::MediaQueryExpSerialization::CssText(n).Utf8()
            }
            Self::Operation {
                operator, operands, ..
            } => {
                let texts = operands.iter().map(Self::CssText).collect::<Vec<_>>();
                match operator {
                    Min | Max | Clamp => format!(
                        "{}({})",
                        match operator {
                            Min => "min",
                            Max => "max",
                            _ => "clamp",
                        },
                        texts.join(", ")
                    ),
                    RoundNearest | RoundUp | RoundDown | RoundToZero => format!(
                        "round({}{})",
                        match operator {
                            RoundNearest => "",
                            RoundUp => "up, ",
                            RoundDown => "down, ",
                            RoundToZero => "to-zero, ",
                            _ => unreachable!(),
                        },
                        texts.join(", ")
                    ),
                    Mod | Rem | Hypot | Log | Pow | Atan2 => format!(
                        "{}({})",
                        match operator {
                            Mod => "mod",
                            Rem => "rem",
                            Hypot => "hypot",
                            Log => "log",
                            Pow => "pow",
                            Atan2 => "atan2",
                            _ => unreachable!(),
                        },
                        texts.join(", ")
                    ),
                    Abs | Sign | Exp | Sqrt | Sin | Cos | Tan | Asin | Acos | Atan => format!(
                        "{}({})",
                        match operator {
                            Abs => "abs",
                            Sign => "sign",
                            Exp => "exp",
                            Sqrt => "sqrt",
                            Sin => "sin",
                            Cos => "cos",
                            Tan => "tan",
                            Asin => "asin",
                            Acos => "acos",
                            Atan => "atan",
                            _ => unreachable!(),
                        },
                        texts[0]
                    ),
                    _ => format!(
                        "({} {} {})",
                        texts[0],
                        match operator {
                            Add => "+",
                            Subtract => "-",
                            Multiply => "*",
                            _ => "/",
                        },
                        texts[1]
                    ),
                }
            }
        }
    }
}
fn CalculationOpFor(operator: CSSMathOperator) -> CalculationOp {
    use CSSMathOperator::*;
    match operator {
        Add => CalculationOp::kAdd,
        Subtract => CalculationOp::kSubtract,
        Multiply => CalculationOp::kMultiply,
        Divide => CalculationOp::kInvert,
        Min => CalculationOp::kMin,
        Max => CalculationOp::kMax,
        Clamp => CalculationOp::kClamp,
        RoundNearest => CalculationOp::kRoundNearest,
        RoundUp => CalculationOp::kRoundUp,
        RoundDown => CalculationOp::kRoundDown,
        RoundToZero => CalculationOp::kRoundToZero,
        Mod => CalculationOp::kMod,
        Rem => CalculationOp::kRem,
        Hypot => CalculationOp::kHypot,
        Abs => CalculationOp::kAbs,
        Sign => CalculationOp::kSign,
        Log => CalculationOp::kLog,
        Exp => CalculationOp::kExp,
        Sqrt => CalculationOp::kSqrt,
        Pow => CalculationOp::kPow,
        Sin => CalculationOp::kSin,
        Cos => CalculationOp::kCos,
        Tan => CalculationOp::kTan,
        Asin => CalculationOp::kAsin,
        Acos => CalculationOp::kAcos,
        Atan => CalculationOp::kAtan,
        Atan2 => CalculationOp::kAtan2,
    }
}
fn CSSMin(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a == 0.0 && b == 0.0 {
        if a.is_sign_negative() || b.is_sign_negative() {
            -0.0
        } else {
            0.0
        }
    } else {
        a.min(b)
    }
}
fn CSSMax(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a == 0.0 && b == 0.0 {
        if a.is_sign_positive() || b.is_sign_positive() {
            0.0
        } else {
            -0.0
        }
    } else {
        a.max(b)
    }
}
pub fn IsBasicMathFunction(id: Option<CSSValueID>) -> bool {
    matches!(
        id,
        Some(
            CSSValueID::kCalc
                | CSSValueID::kWebkitCalc
                | CSSValueID::kMin
                | CSSValueID::kMax
                | CSSValueID::kClamp
                | CSSValueID::kRound
                | CSSValueID::kMod
                | CSSValueID::kRem
                | CSSValueID::kHypot
                | CSSValueID::kAbs
                | CSSValueID::kSign
                | CSSValueID::kLog
                | CSSValueID::kExp
                | CSSValueID::kSqrt
                | CSSValueID::kPow
                | CSSValueID::kSin
                | CSSValueID::kCos
                | CSSValueID::kTan
                | CSSValueID::kAsin
                | CSSValueID::kAcos
                | CSSValueID::kAtan
                | CSSValueID::kAtan2
        )
    )
}
// cpp: css_math_expression_node.cc:4708-4781,4839-4902,5069-5270.
pub fn ConsumeMathFunction<T: TokenStreamTokenizer>(
    stream: &mut CSSParserTokenStream<'_, T>,
) -> Result<CSSMathExpressionNode, MathError> {
    let node = Function(stream, 0)?;
    stream.ConsumeWhitespace();
    Ok(node)
}
fn Function<T: TokenStreamTokenizer>(
    stream: &mut CSSParserTokenStream<'_, T>,
    depth: usize,
) -> Result<CSSMathExpressionNode, MathError> {
    use CSSMathOperator::*;
    if depth > 100 {
        return Err(MathError::DepthLimit);
    }
    let id = stream.Peek().FunctionId();
    if !IsBasicMathFunction(id) {
        return Err(MathError::UnsupportedFunction);
    }
    let mut operands = Vec::new();
    let mut guard = BlockGuard::new(stream);
    guard.ConsumeWhitespace();
    let round_operator = if id == Some(CSSValueID::kRound) {
        let operator = match guard.Peek().Id() {
            CSSValueID::kNearest => Some(RoundNearest),
            CSSValueID::kUp => Some(RoundUp),
            CSSValueID::kDown => Some(RoundDown),
            CSSValueID::kToZero => Some(RoundToZero),
            _ => None,
        };
        if operator.is_some() {
            guard.ConsumeIncludingWhitespace();
            if guard.Peek().GetType() != kCommaToken {
                return Err(MathError::Invalid);
            }
            guard.ConsumeIncludingWhitespace();
        }
        operator.unwrap_or(RoundNearest)
    } else {
        RoundNearest
    };
    while !guard.AtEnd() {
        if !operands.is_empty() {
            if guard.Peek().GetType() != kCommaToken {
                return Err(MathError::Invalid);
            }
            guard.ConsumeIncludingWhitespace();
        }
        // clamp's none represents an unbounded endpoint, with the category of
        // the center operand. Replace it after validating the argument count.
        if id == Some(CSSValueID::kClamp) && guard.Peek().Id() == CSSValueID::kNone {
            guard.ConsumeIncludingWhitespace();
            operands.push(None);
        } else {
            operands.push(Some(Sum(&mut guard, depth + 1)?.0));
        }
    }
    let node = match id.unwrap() {
        CSSValueID::kCalc | CSSValueID::kWebkitCalc => {
            if operands.len() != 1 {
                return Err(MathError::Invalid);
            }
            operands.pop().flatten().ok_or(MathError::Invalid)?
        }
        CSSValueID::kClamp => {
            if operands.len() != 3 || operands[1].is_none() {
                return Err(MathError::Invalid);
            }
            // none bounds are removed rather than fabricated numeric values.
            let center = operands[1].take().unwrap();
            let low = operands[0].take();
            let high = operands[2].take();
            match (low, high) {
                (None, None) => center,
                (None, Some(h)) => CSSMathExpressionNode::Operation(Min, vec![center, h])?,
                (Some(l), None) => CSSMathExpressionNode::Operation(Max, vec![l, center])?,
                (Some(l), Some(h)) => CSSMathExpressionNode::Operation(Clamp, vec![l, center, h])?,
            }
        }
        CSSValueID::kMin | CSSValueID::kMax => CSSMathExpressionNode::Operation(
            if id == Some(CSSValueID::kMin) {
                Min
            } else {
                Max
            },
            operands
                .into_iter()
                .collect::<Option<Vec<_>>>()
                .ok_or(MathError::Invalid)?,
        )?,
        CSSValueID::kRound => {
            if operands.len() == 1
                && operands[0]
                    .as_ref()
                    .is_some_and(|n| n.Category() == CalculationResultCategory::Number)
            {
                operands.push(Some(CSSMathExpressionNode::Numeric(
                    CSSNumericLiteralValue::Create(1.0, UnitType::kNumber),
                )));
            }
            if operands.len() != 2 {
                return Err(MathError::Invalid);
            }
            CSSMathExpressionNode::Operation(
                round_operator,
                operands
                    .into_iter()
                    .collect::<Option<Vec<_>>>()
                    .ok_or(MathError::Invalid)?,
            )?
        }
        CSSValueID::kMod | CSSValueID::kRem | CSSValueID::kPow | CSSValueID::kAtan2 => {
            if operands.len() != 2 {
                return Err(MathError::Invalid);
            }
            CSSMathExpressionNode::Operation(
                match id.unwrap() {
                    CSSValueID::kMod => Mod,
                    CSSValueID::kRem => Rem,
                    CSSValueID::kPow => Pow,
                    _ => Atan2,
                },
                operands
                    .into_iter()
                    .collect::<Option<Vec<_>>>()
                    .ok_or(MathError::Invalid)?,
            )?
        }
        CSSValueID::kHypot => {
            if operands.is_empty() {
                return Err(MathError::Invalid);
            }
            CSSMathExpressionNode::Operation(
                Hypot,
                operands
                    .into_iter()
                    .collect::<Option<Vec<_>>>()
                    .ok_or(MathError::Invalid)?,
            )?
        }
        CSSValueID::kLog => {
            if !(1..=2).contains(&operands.len()) {
                return Err(MathError::Invalid);
            }
            CSSMathExpressionNode::Operation(
                Log,
                operands
                    .into_iter()
                    .collect::<Option<Vec<_>>>()
                    .ok_or(MathError::Invalid)?,
            )?
        }
        CSSValueID::kAbs
        | CSSValueID::kSign
        | CSSValueID::kExp
        | CSSValueID::kSqrt
        | CSSValueID::kSin
        | CSSValueID::kCos
        | CSSValueID::kTan
        | CSSValueID::kAsin
        | CSSValueID::kAcos
        | CSSValueID::kAtan => {
            if operands.len() != 1 {
                return Err(MathError::Invalid);
            }
            CSSMathExpressionNode::Operation(
                match id.unwrap() {
                    CSSValueID::kAbs => Abs,
                    CSSValueID::kSign => Sign,
                    CSSValueID::kExp => Exp,
                    CSSValueID::kSqrt => Sqrt,
                    CSSValueID::kSin => Sin,
                    CSSValueID::kCos => Cos,
                    CSSValueID::kTan => Tan,
                    CSSValueID::kAsin => Asin,
                    CSSValueID::kAcos => Acos,
                    _ => Atan,
                },
                operands
                    .into_iter()
                    .collect::<Option<Vec<_>>>()
                    .ok_or(MathError::Invalid)?,
            )?
        }
        _ => unreachable!(),
    };
    drop(guard);
    Ok(node)
}
fn Term<T: TokenStreamTokenizer>(
    s: &mut CSSParserTokenStream<'_, T>,
    depth: usize,
) -> Result<(CSSMathExpressionNode, bool), MathError> {
    if depth > 100 {
        return Err(MathError::DepthLimit);
    }
    let node = if s.Peek().GetType() == kLeftParenthesisToken {
        let mut g = BlockGuard::new(s);
        g.ConsumeWhitespace();
        let n = Sum(&mut g, depth + 1)?.0;
        if !g.AtEnd() {
            return Err(MathError::Invalid);
        }
        drop(g);
        n
    } else if s.Peek().GetType() == kFunctionToken {
        // Keep whitespace information outside the block before consuming it.
        return FunctionTerm(s, depth);
    } else {
        let t = s.Peek().clone();
        let (v, u) = match t.GetType() {
            kNumberToken => (t.NumericValue(), UnitType::kNumber),
            kPercentageToken => (t.NumericValue(), UnitType::kPercentage),
            kDimensionToken => (t.NumericValue(), t.GetUnitType()),
            kIdentToken => (
                match t.Id() {
                    CSSValueID::kInfinity => f64::INFINITY,
                    CSSValueID::kNegativeInfinity => f64::NEG_INFINITY,
                    CSSValueID::kNan => f64::NAN,
                    CSSValueID::kPi => std::f64::consts::PI,
                    CSSValueID::kE => std::f64::consts::E,
                    _ => return Err(MathError::Invalid),
                },
                UnitType::kNumber,
            ),
            _ => return Err(MathError::Invalid),
        };
        UnitCategory(u).ok_or(MathError::Invalid)?;
        s.Consume();
        CSSMathExpressionNode::Numeric(CSSNumericLiteralValue::Create(v, u))
    };
    let ws = s.Peek().GetType() == kWhitespaceToken;
    s.ConsumeWhitespace();
    Ok((node, ws))
}
// Function uses BlockGuard but deliberately leaves following whitespace intact
// for the additive grammar's required whitespace on both sides of +/-.
fn FunctionTerm<T: TokenStreamTokenizer>(
    s: &mut CSSParserTokenStream<'_, T>,
    depth: usize,
) -> Result<(CSSMathExpressionNode, bool), MathError> {
    let node = Function(s, depth)?;
    let ws = s.Peek().GetType() == kWhitespaceToken;
    s.ConsumeWhitespace();
    Ok((node, ws))
}
fn Product<T: TokenStreamTokenizer>(
    s: &mut CSSParserTokenStream<'_, T>,
    depth: usize,
) -> Result<(CSSMathExpressionNode, bool), MathError> {
    let (mut node, mut ws) = Term(s, depth)?;
    let mut count = depth;
    loop {
        let op = if s.Peek().GetType() == kDelimiterToken {
            match s.Peek().Delimiter() {
                42 => CSSMathOperator::Multiply,
                47 => CSSMathOperator::Divide,
                _ => break,
            }
        } else {
            break;
        };
        count += 1;
        if count > 100 {
            return Err(MathError::DepthLimit);
        }
        s.ConsumeIncludingWhitespace();
        let (rhs, w) = Term(s, depth)?;
        ws = w;
        node = CSSMathExpressionNode::Operation(op, vec![node, rhs])?;
    }
    Ok((node, ws))
}
fn Sum<T: TokenStreamTokenizer>(
    s: &mut CSSParserTokenStream<'_, T>,
    depth: usize,
) -> Result<(CSSMathExpressionNode, bool), MathError> {
    let (mut node, mut ws) = Product(s, depth)?;
    let mut count = depth;
    loop {
        let op = if s.Peek().GetType() == kDelimiterToken {
            match s.Peek().Delimiter() {
                43 => CSSMathOperator::Add,
                45 => CSSMathOperator::Subtract,
                _ => break,
            }
        } else {
            break;
        };
        if !ws {
            return Err(MathError::Invalid);
        }
        count += 1;
        if count > 100 {
            return Err(MathError::DepthLimit);
        }
        s.Consume();
        if s.Peek().GetType() != kWhitespaceToken {
            return Err(MathError::Invalid);
        }
        s.ConsumeWhitespace();
        let (rhs, w) = Product(s, depth)?;
        ws = w;
        node = CSSMathExpressionNode::Operation(op, vec![node, rhs])?;
    }
    Ok((node, ws))
}

// Chromium CSSMathExpressionOperation evaluates double values. The foundation
// helpers implement the float instantiations for the native calculation tree;
// retain double precision here until the actual platform lowering boundary.
// cpp: platform/geometry/math_functions.h:24-283; ui/gfx/geometry/sin_cos_degrees.h.
fn nearest_multiples(mut a: f64, mut b: f64) -> (f64, f64) {
    let is_negative = a < 0.0;
    a = a.abs();
    b = b.abs();
    let mut c = -(a % b);
    let mut lower = a + c;
    if a.abs() > c.abs() {
        std::mem::swap(&mut a, &mut c);
    }
    if a.abs() > b.abs() {
        std::mem::swap(&mut a, &mut b);
    }
    if b.abs() > c.abs() {
        std::mem::swap(&mut b, &mut c);
    }
    let mut upper = a + b + c;
    if is_negative {
        std::mem::swap(&mut lower, &mut upper);
        lower = -lower;
        upper = -upper;
    }
    (lower, upper)
}

// cpp: foundation/blink_geometry/geometry/math_functions.h:279-283
fn EvaluateSignFunction(value: f64) -> f64 {
    if value == 0.0 || value.is_nan() {
        value
    } else if value > 0.0 {
        1.0
    } else {
        -1.0
    }
}

// cpp: foundation/blink_geometry/geometry/math_functions.h:151-166
fn EvaluateRoundDownFunction(a: f64, b: f64) -> f64 {
    let (lower, _) = nearest_multiples(a, b);
    if !a.is_infinite() && b.is_infinite() {
        if a == 0.0 {
            a
        } else if a.is_sign_negative() {
            f64::NEG_INFINITY
        } else {
            0.0
        }
    } else {
        lower
    }
}

// cpp: foundation/blink_geometry/geometry/math_functions.h:168-275
fn EvaluateSteppedValueFunction(op: CalculationOp, a: f64, b: f64) -> f64 {
    use CalculationOp::*;
    assert!(matches!(
        op,
        kRoundNearest | kRoundUp | kRoundDown | kRoundToZero | kMod | kRem
    ));
    if b == 0.0 || (a.is_infinite() && b.is_infinite()) {
        return f64::NAN;
    }
    if matches!(op, kRoundNearest | kRoundUp | kRoundDown | kRoundToZero)
        && (a % b == 0.0 || (a.is_infinite() && !b.is_infinite()))
    {
        return a;
    }
    if matches!(op, kMod | kRem) && a.is_infinite() {
        return f64::NAN;
    }
    let (mut lower, mut upper) = nearest_multiples(a, b);
    match op {
        kRoundNearest => {
            if !a.is_infinite() && b.is_infinite() {
                return 0.0f64.copysign(a);
            }
            let a_is_negative = a < 0.0;
            if a_is_negative {
                std::mem::swap(&mut lower, &mut upper);
            }
            let distance = (a % b).abs();
            let half_b = b.abs() / 2.0;
            if distance < half_b || (a_is_negative && distance == half_b) {
                lower
            } else {
                upper
            }
        }
        kRoundUp => {
            if !a.is_infinite() && b.is_infinite() {
                if a == 0.0 {
                    a
                } else if a.is_sign_negative() {
                    -0.0
                } else {
                    f64::INFINITY
                }
            } else {
                upper
            }
        }
        kRoundDown => EvaluateRoundDownFunction(a, b),
        kRoundToZero => {
            if !a.is_infinite() && b.is_infinite() {
                0.0f64.copysign(a)
            } else if upper.abs() < lower.abs() {
                upper
            } else {
                lower
            }
        }
        kMod => {
            if b.is_infinite() && a.is_sign_negative() != b.is_sign_negative() {
                return f64::NAN;
            }
            let result = a % b;
            if result == 0.0 {
                0.0f64.copysign(b)
            } else if result.is_sign_negative() != b.is_sign_negative() {
                result + b
            } else {
                result
            }
        }
        kRem => a % b,
        _ => unreachable!(),
    }
}

// C++: src/foundation/gfx_geometry/sin_cos_degrees.h:25-102.
// cpp: foundation/gfx_geometry/sin_cos_degrees.h:25-102
fn sin_cos_degrees(mut degrees: f64) -> (f64, f64) {
    if degrees > -90_000_000.0 && degrees < 90_000_000.0 {
        let n45 = degrees / 45.0;
        let mut octant = n45 as i32;
        if octant as f64 == n45 {
            let half_sqrt2 = std::f64::consts::SQRT_2 / 2.0;
            return [
                (0.0, 1.0),
                (half_sqrt2, half_sqrt2),
                (1.0, 0.0),
                (half_sqrt2, -half_sqrt2),
                (0.0, -1.0),
                (-half_sqrt2, -half_sqrt2),
                (-1.0, 0.0),
                (-half_sqrt2, half_sqrt2),
            ][(octant & 7) as usize];
        }
        if degrees < 0.0 {
            octant -= 1;
        }
        degrees -= octant as f64 * 45.0;
        if octant & 1 != 0 {
            degrees = 45.0 - degrees;
        }
        let radians = degrees.to_radians();
        let mut sine = radians.sin();
        let mut cosine = radians.cos();
        if (octant + 1) & 2 != 0 {
            std::mem::swap(&mut sine, &mut cosine);
        }
        if octant & 4 != 0 {
            sine = -sine;
        }
        if (octant + 2) & 4 != 0 {
            cosine = -cosine;
        }
        return (sine, cosine);
    }
    let radians = (degrees % 360.0).to_radians();
    (radians.sin(), radians.cos())
}

// cpp: foundation/blink_geometry/geometry/math_functions.h:109-149
fn EvaluateTrigonometricFunction(op: CalculationOp, a: f64, b: Option<f64>) -> f64 {
    use CalculationOp::*;
    match op {
        kSin => sin_cos_degrees(a as f64).0 as f64,
        kCos => sin_cos_degrees(a as f64).1 as f64,
        kTan => {
            if a > -90_000_000.0 && a < 90_000_000.0 {
                let n45 = a / 45.0;
                let octant = n45 as i32;
                if octant as f64 == n45 {
                    return [
                        0.0,
                        1.0,
                        f64::INFINITY,
                        -1.0,
                        0.0,
                        1.0,
                        f64::NEG_INFINITY,
                        -1.0,
                    ][(octant & 7) as usize];
                }
            }
            a.to_radians().tan()
        }
        kAsin => a.asin().to_degrees(),
        kAcos => a.acos().to_degrees(),
        kAtan => a.atan().to_degrees(),
        kAtan2 => a
            .atan2(b.expect("atan2 requires a second operand"))
            .to_degrees(),
        _ => panic!("operator is not trigonometric: {op:?}"),
    }
}
