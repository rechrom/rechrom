// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Concrete numeric literal payload used by the media-query value consumer.
//! Source: css_numeric_literal_value.h:22-38,61-75; .cc:111-140,191-220,222-323.
//! Number/length/resolution literals and media-query math payload ownership are
//! mapped here. Allocation caches and unrelated numeric categories remain out of scope.
use crate::media_query_exp::MediaQueryExpSerialization;
use crate::{
    css_math_expression_node::{CSSMathExpressionNode, CalculationResultCategory, MathError},
    css_math_function_value::{CSSMathFunctionValue, ValueRange},
    css_primitive_value::{UnitType, UnitTypeToString},
};
use foundation::String;

#[derive(Clone, Debug, PartialEq)]
pub struct CSSNumericLiteralValue {
    num_: f64,
    unit_: UnitType,
    math_: Option<Box<CSSMathFunctionValue>>,
}
impl CSSNumericLiteralValue {
    pub fn Create(value: f64, unit: UnitType) -> Self {
        Self {
            num_: value,
            unit_: unit,
            math_: None,
        }
    }
    pub fn CreateMath(expression: CSSMathExpressionNode, range: ValueRange) -> Self {
        let unit = expression.CanonicalUnit().unwrap_or(UnitType::kUnknown);
        Self {
            num_: expression.GetValueIfKnown().unwrap_or(f64::NAN),
            unit_: unit,
            math_: Some(Box::new(CSSMathFunctionValue::Create(expression, range))),
        }
    }
    pub fn MathValue(&self) -> Option<&CSSMathFunctionValue> {
        self.math_.as_deref()
    }
    pub fn WithValueRange(mut self, range: ValueRange) -> Self {
        if let Some(math) = &mut self.math_ {
            math.range = range;
        }
        self
    }
    pub fn GetValueIfKnown(&self) -> Option<f64> {
        self.math_.as_ref().map_or(Some(self.num_), |value| {
            value
                .expression
                .GetValueIfKnown()
                .map(|number| value.ClampToPermittedRange(number))
        })
    }
    pub fn Resolve(
        &self,
        resolver: &mut impl crate::css_math_expression_node::CSSMathLengthResolver,
    ) -> Result<Self, MathError> {
        let Some(math) = &self.math_ else {
            return Ok(self.clone());
        };
        let unit = math.expression.CanonicalUnit().ok_or(MathError::Invalid)?;
        Ok(Self::Create(math.ComputeValue(resolver, None)?, unit))
    }
    pub fn DoubleValue(&self) -> f64 {
        self.num_
    }
    pub fn GetType(&self) -> UnitType {
        self.unit_
    }
    pub fn IsNumber(&self) -> bool {
        self.math_.as_ref().map_or(
            matches!(self.unit_, UnitType::kNumber | UnitType::kInteger),
            |value| value.Category() == CalculationResultCategory::Number,
        )
    }
    pub fn IsInteger(&self) -> bool {
        self.math_
            .as_ref()
            .map_or(self.unit_ == UnitType::kInteger, |value| {
                matches!(
                    value.range,
                    ValueRange::Integer
                        | ValueRange::NonNegativeInteger
                        | ValueRange::PositiveInteger
                )
            })
    }
    pub fn IsLength(&self) -> bool {
        self.math_.as_ref().map_or(IsLength(self.unit_), |value| {
            value.Category() == CalculationResultCategory::Length
        })
    }
    pub fn IsResolution(&self) -> bool {
        self.math_.as_ref().map_or(
            matches!(
                self.unit_,
                UnitType::kDotsPerPixel
                    | UnitType::kX
                    | UnitType::kDotsPerInch
                    | UnitType::kDotsPerCentimeter
            ),
            |value| value.Category() == CalculationResultCategory::Resolution,
        )
    }
    pub fn ComputeDotsPerPixel(&self) -> f64 {
        self.num_
            * match self.unit_ {
                UnitType::kDotsPerInch => 1.0 / 96.0,
                UnitType::kDotsPerCentimeter => 2.54 / 96.0,
                UnitType::kDotsPerPixel | UnitType::kX => 1.0,
                _ => unreachable!("resolution required"),
            }
    }
}
pub fn IsLength(unit: UnitType) -> bool {
    use UnitType::*;
    matches!(
        unit,
        kEms | kExs
            | kPixels
            | kCentimeters
            | kMillimeters
            | kInches
            | kPoints
            | kPicas
            | kQuarterMillimeters
            | kViewportWidth
            | kViewportHeight
            | kViewportInlineSize
            | kViewportBlockSize
            | kViewportMin
            | kViewportMax
            | kSmallViewportWidth
            | kSmallViewportHeight
            | kSmallViewportInlineSize
            | kSmallViewportBlockSize
            | kSmallViewportMin
            | kSmallViewportMax
            | kLargeViewportWidth
            | kLargeViewportHeight
            | kLargeViewportInlineSize
            | kLargeViewportBlockSize
            | kLargeViewportMin
            | kLargeViewportMax
            | kDynamicViewportWidth
            | kDynamicViewportHeight
            | kDynamicViewportInlineSize
            | kDynamicViewportBlockSize
            | kDynamicViewportMin
            | kDynamicViewportMax
            | kContainerWidth
            | kContainerHeight
            | kContainerInlineSize
            | kContainerBlockSize
            | kContainerMin
            | kContainerMax
            | kRems
            | kRexs
            | kRchs
            | kRics
            | kChs
            | kIcs
            | kLhs
            | kRlhs
            | kCaps
            | kRcaps
    )
}
impl MediaQueryExpSerialization for CSSNumericLiteralValue {
    fn CssText(&self) -> String {
        if let Some(math) = &self.math_ {
            return crate::css_value::CSSValueSubclass::CustomCSSText(math.as_ref());
        }
        let suffix = UnitTypeToString(self.unit_).expect("numeric literal unit");
        if self.IsInteger() {
            return String::Number(self.num_ as i32);
        }
        if self.num_.is_finite()
            && (-999999.0..=999999.0).contains(&self.num_)
            && self.num_.trunc() == self.num_
        {
            return String::from(format!("{}{suffix}", self.num_ as i32).as_str());
        }
        if !self.num_.is_finite() {
            let number = if self.num_.is_nan() {
                "NaN"
            } else if self.num_ > 0.0 {
                "infinity"
            } else {
                "-infinity"
            };
            return String::from(
                if suffix.is_empty() {
                    number.to_owned()
                } else {
                    format!("{number} * 1{suffix}")
                }
                .as_str(),
            );
        }
        // Blink FormatNumber uses %.6g; the libc formatting contract matches it.
        unsafe extern "C" {
            fn snprintf(
                buffer: *mut std::ffi::c_char,
                size: usize,
                format: *const std::ffi::c_char,
                ...
            ) -> std::ffi::c_int;
        }
        let mut buffer = [0u8; 64];
        unsafe {
            snprintf(
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                b"%.6g\0".as_ptr().cast(),
                self.num_,
            );
        }
        let length = buffer.iter().position(|&byte| byte == 0).unwrap();
        let mut text = String::from_latin1(&buffer[..length]);
        text.push_str(suffix);
        text
    }
}
