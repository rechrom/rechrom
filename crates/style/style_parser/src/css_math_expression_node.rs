//! CSS math token grammar over the single compiled model owned by layoutng_style.
#![allow(non_snake_case)]
use crate::{
    css_numeric_literal_value::CSSNumericLiteralValue,
    css_parser_token::CSSParserTokenType::*,
    css_parser_token_stream::{BlockGuard, CSSParserTokenStream, TokenStreamTokenizer},
    css_primitive_value::UnitType,
};
use foundation::CSSValueID;
pub use layoutng_style::css::css_math_expression_node::*;
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
// helpers implement the float instantiations for the computed style calculation tree;
// retain double precision here until the actual platform lowering boundary.
// cpp: platform/geometry/math_functions.h:24-283; ui/gfx/geometry/sin_cos_degrees.h.
