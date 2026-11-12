//! CSSParser::ParseColor/ParseSystemColor projection for native form controls.
//! This boundary returns canonical Color values, never reparsed CSS strings.
use foundation::{Color, String};
use style_model::{
    css_parser_mode::CSSParserMode, css_property_names::CSSPropertyID, css_value::CSSValuePayload,
};
use style_parser::{production_css_value as value, production_property_parser as parser};
#[derive(Clone, Copy, Debug)]
pub enum RequiredInputColorMechanism {
    SystemColorLightPalette,
    UntranslatedColorConsumer,
}
pub fn Parse(value: &str, system: bool) -> Result<Option<Color>, RequiredInputColorMechanism> {
    let properties = match parser::ParseProperty(
        CSSPropertyID::kColor,
        &String::from(value),
        false,
        CSSParserMode::kHTMLStandardMode,
    ) {
        Ok(p) => p,
        Err(e) => {
            return if e.kind == parser::PropertyParseErrorKind::Invalid {
                Ok(None)
            } else {
                Err(RequiredInputColorMechanism::UntranslatedColorConsumer)
            }
        }
    };
    let Some(property) = properties.first() else {
        return Ok(None);
    };
    match property.Value().Payload() {
        CSSValuePayload::kColorClass(color) => Ok((!system).then_some(color.0)),
        CSSValuePayload::kIdentifierClass(identifier) => {
            if let Some(color) = value::NamedColor(identifier.0) {
                return Ok((!system).then_some(color));
            }
            if system && layoutng_style::css::style_color::StyleColor::IsSystemColorIncludingDeprecatedKeyword(identifier.0) {
                Err(RequiredInputColorMechanism::SystemColorLightPalette)
            } else {
                Ok(None)
            }
        }
        _ => Ok(None),
    }
}
pub fn Serialize(color: Color) -> std::string::String {
    value::color(color).CssText().Utf8()
}
