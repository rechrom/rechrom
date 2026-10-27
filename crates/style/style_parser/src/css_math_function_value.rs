// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
#![allow(non_snake_case)]
use crate::{
    css_math_expression_node::{
        CSSMathExpressionNode, CSSMathLengthResolver, CalculationResultCategory, MathError,
    },
    css_value::{CSSValueRandom, CSSValueSubclass, CSSValueTreeScope},
    production_css_value::{ProductionCSSValueDispatch, Value},
};
use foundation::{CalculationValue, Length, LengthValueRange, String};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueRange {
    All,
    NonNegative,
    Integer,
    NonNegativeInteger,
    PositiveInteger,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CSSMathFunctionValue {
    pub expression: CSSMathExpressionNode,
    pub range: ValueRange,
}
impl CSSMathFunctionValue {
    // cpp: css_math_function_value.cc:39-47,86-139,162-184.
    pub fn Create(expression: CSSMathExpressionNode, range: ValueRange) -> Self {
        Self { expression, range }
    }
    pub fn Category(&self) -> CalculationResultCategory {
        self.expression.Category()
    }
    pub fn ClampToPermittedRange(&self, value: f64) -> f64 {
        let value = if value.is_nan() { 0.0 } else { value };
        match self.range {
            ValueRange::All => value,
            ValueRange::NonNegative => value.max(0.0),
            ValueRange::Integer => (value + 0.5).floor(),
            ValueRange::NonNegativeInteger => (value.max(0.0) + 0.5).floor(),
            ValueRange::PositiveInteger => (value.max(1.0) + 0.5).floor(),
        }
    }
    pub fn ComputeValue(
        &self,
        resolver: &mut impl CSSMathLengthResolver,
        percentage_basis: Option<f64>,
    ) -> Result<f64, MathError> {
        Ok(self.ClampToPermittedRange(self.expression.ComputeValue(resolver, percentage_basis)?))
    }
    // cpp: css_math_function_value.cc:123-139,ConvertToLength; native
    // CalculationValue applies NaN censoring and range at used-value time.
    pub fn ConvertToLength(
        &self,
        resolver: &mut impl CSSMathLengthResolver,
    ) -> Result<Length, MathError> {
        use CalculationResultCategory::*;
        Ok(match self.Category() {
            Length => foundation::Length::Fixed(
                crate::css_value_clamping_utils::CSSValueClampingUtils::ClampLength(
                    self.ComputeValue(resolver, None)?,
                ),
            ),
            Percent => foundation::Length::Percent(
                crate::css_value_clamping_utils::CSSValueClampingUtils::ClampLength(
                    self.ComputeValue(resolver, None)?,
                ),
            ),
            LengthFunction => {
                foundation::Length::from_calculation_value(CalculationValue::CreateSimplified(
                    self.expression.ToCalculationExpression(resolver)?,
                    if self.range == ValueRange::NonNegative {
                        LengthValueRange::kNonNegative
                    } else {
                        LengthValueRange::kAll
                    },
                ))
            }
            _ => return Err(MathError::Invalid),
        })
    }
}
impl CSSValueSubclass for CSSMathFunctionValue {
    // cpp: css_math_function_value.cc:141-163.
    fn CustomCSSText(&self) -> String {
        let text = self.expression.CssText();
        String::from(
            if text.starts_with("min(") || text.starts_with("max(") || text.starts_with("clamp(") {
                text
            } else if text.starts_with('(') {
                format!("calc{text}")
            } else {
                format!("calc({text})")
            }
            .as_str(),
        )
    }
    fn Equals(&self, other: &Self) -> bool {
        self.expression == other.expression
    }
}
impl CSSValueRandom for CSSMathFunctionValue {
    fn HasRandomFunctions(&self) -> bool {
        false
    }
}
impl CSSValueTreeScope<ProductionCSSValueDispatch> for CSSMathFunctionValue {
    fn PopulateWithTreeScope<'a>(&'a self, _: Option<&'a ()>) -> &'a Value {
        unreachable!("basic math expressions are already scoped")
    }
}
