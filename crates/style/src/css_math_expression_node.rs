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
        use CalculationResultCategory::Number;
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
        if self.Category() == CalculationResultCategory::LengthFunction
            && percentage_basis.is_none()
        {
            return Err(MathError::MissingPercentageBasis);
        }
        self.Evaluate(
            resolver,
            percentage_basis,
            self.Category() == CalculationResultCategory::LengthFunction,
        )
    }
    // cpp: css_math_expression_node.h:445-447,819-821. Existing numeric and
    // operation owners query canonical value without a length context.
    pub fn GetValueIfKnown(&self) -> Option<f64> {
        self.ComputeValue(&mut |_, _| Err(MathError::MissingLengthContext), None).ok()
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
                Ok(match operator {
                    Add => values[0] + values[1],
                    Subtract => values[0] - values[1],
                    Multiply => values[0] * values[1],
                    Divide => values[0] / values[1],
                    Min => values.into_iter().reduce(CSSMin).unwrap(),
                    Max => values.into_iter().reduce(CSSMax).unwrap(),
                    Clamp => CSSMax(values[0], CSSMin(values[1], values[2])),
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
                    _ => return Err(MathError::Invalid),
                })
            }
            Self::Operation {
                operator, operands, ..
            } => {
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
