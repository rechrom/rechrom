use crate::{
    css_value::CSSValueTreeScope,
    production_css_value::{ProductionCSSValueDispatch, Value},
};
pub use layoutng_style::css::css_math_function_value::*;
impl CSSValueTreeScope<ProductionCSSValueDispatch> for CSSMathFunctionValue {
    fn PopulateWithTreeScope<'a>(&'a self, _: Option<&'a ()>) -> &'a Value {
        unreachable!("basic math expressions are already scoped")
    }
}
