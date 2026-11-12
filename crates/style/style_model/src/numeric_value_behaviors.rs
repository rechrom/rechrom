//! Dispatch traits borrow the same lower numeric/math payloads.
use crate::css_value::{CSSValueCustomHash, CSSValueRandom, CSSValueSubclass};
use foundation::{HashInts, String};
use layoutng_style::css::{
    css_math_function_value::CSSMathFunctionValue,
    css_numeric_literal_value::CSSNumericLiteralValue,
};
// cpp: css_numeric_literal_value.cc:222-323,397-412
impl CSSValueSubclass for CSSNumericLiteralValue {
    fn CustomCSSText(&self) -> String {
        CSSNumericLiteralValue::CssText(self)
    }
    fn Equals(&self, other: &Self) -> bool {
        self == other
    }
}
impl CSSValueCustomHash for CSSNumericLiteralValue {
    fn CustomHash(&self) -> u32 {
        let bits = self.DoubleValue().to_bits();
        HashInts(
            self.GetType() as u32,
            HashInts((bits >> 32) as u32, bits as u32),
        )
    }
}
impl CSSValueSubclass for CSSMathFunctionValue {
    fn CustomCSSText(&self) -> String {
        self.CssText()
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
