//! Numeric payloads retain one model owner across parser and native Layout.
pub use layoutng_style::css::css_numeric_literal_value::*;
impl crate::media_query_exp::MediaQueryExpSerialization for CSSNumericLiteralValue {
    fn CssText(&self) -> foundation::String {
        CSSNumericLiteralValue::CssText(self)
    }
}
