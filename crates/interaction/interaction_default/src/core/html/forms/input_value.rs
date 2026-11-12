//! Source InputType::SanitizeValue dispatch. These are the same subclass
//! sanitizers used by script assignment, parser defaults and type changes.
use super::{
    input_type::Type,
    input_validation::{range, InputValidationFacts},
    step_range::NumberTypeParser,
};
use crate::platform::wtf::decimal::Decimal;
pub fn SanitizeValue(
    facts: &InputValidationFacts<'_>,
    parser: &dyn NumberTypeParser<Decimal>,
) -> Vec<u16> {
    match facts.kind {
        Type::kText | Type::kSearch | Type::kTelephone | Type::kPassword => {
            super::text_field_input_type::TextFieldInputType::SanitizeValueFor(facts.value)
        }
        Type::kURL => super::url_input_type::URLInputType::SanitizeValueFor(facts.value),
        Type::kEmail => super::email_input_type::EmailInputType::SanitizeValueWithMultiple(
            facts.value,
            facts.multiple,
        ),
        Type::kNumber => {
            if facts.value.is_empty()
                || parser
                    .ParseToDoubleForNumberType(&String::from_utf16_lossy(facts.value))
                    .is_finite()
            {
                facts.value.to_vec()
            } else {
                Vec::new()
            }
        }
        Type::kDate | Type::kDateTimeLocal | Type::kMonth | Type::kTime | Type::kWeek => {
            if super::temporal_input_type::ParseToNumber(facts.kind, facts.value).is_some() {
                facts.value.to_vec()
            } else {
                Vec::new()
            }
        }
        Type::kRange => {
            let range = range(facts, parser);
            let parsed = parser.ParseToDecimalForNumberType(
                &String::from_utf16_lossy(facts.value),
                range.DefaultValue(),
            );
            parser
                .SerializeForNumberType(range.ClampValue(parsed))
                .encode_utf16()
                .collect()
        }
        _ => facts.value.to_vec(),
    }
}
