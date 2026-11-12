//! Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
//! HTMLInputElement validity virtuals and InputType numeric constraints:
//! html_input_element.cc, input_type.cc:364-486,554-563,1295-1341;
//! number_input_type.cc:179-188; temporal subclasses' CreateStepRange.
use super::{
    base_text_input_type::{
        BaseTextInputType, NeedsToCheckDirtyFlag, PatternCache, ScriptRegexp, ScriptRegexpFactory,
    },
    email_input_type::EmailInputType,
    input_type::Type,
    listed_element::ValidityState,
    step_range::{
        AnyStepHandling, NumberTypeParser, StepDescription, StepRange, StepValueShouldBe,
    },
    temporal_input_type,
};
use crate::platform::wtf::decimal::Decimal;
use std::rc::Rc;

pub trait ValidationPatternEngine: ScriptRegexpFactory {
    fn EnsureEmailRegexp(&self) -> Rc<dyn ScriptRegexp>;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValidationRequired {
    PatternEngine,
    FocusGeometry,
    ValidationMessageClient,
}
/// Only source facts not owned by this input-type calculation. No DOM/Style/
/// Layout engine, callback accessor or copied tree crosses this boundary.
pub struct InputValidationFacts<'a> {
    pub kind: Type,
    pub value: &'a [u16],
    pub inner_editor_value: &'a [u16],
    pub required: bool,
    pub disabled_or_readonly: bool,
    pub checked: bool,
    pub radio_group_value_missing: bool,
    pub files_empty: bool,
    pub multiple: bool,
    pub dirty: bool,
    pub user_edit: bool,
    pub min_length: i32,
    pub max_length: i32,
    pub pattern: Option<Rc<[u16]>>,
    pub minimum: &'a [u16],
    pub maximum: &'a [u16],
    pub step: &'a str,
    pub value_attribute: &'a [u16],
}
fn text(kind: Type) -> bool {
    matches!(
        kind,
        Type::kText
            | Type::kSearch
            | Type::kTelephone
            | Type::kURL
            | Type::kEmail
            | Type::kPassword
    )
}
fn decimal(value: i64) -> Decimal {
    Decimal::FromComponents(value < 0, 0, value.unsigned_abs())
}
pub(super) fn parse(
    facts: &InputValidationFacts<'_>,
    value: &[u16],
    parser: &dyn NumberTypeParser<Decimal>,
) -> Decimal {
    if matches!(facts.kind, Type::kNumber | Type::kRange) {
        parser.ParseToDecimalForNumberType(&String::from_utf16_lossy(value), Decimal::Nan())
    } else {
        temporal_input_type::ParseToNumber(facts.kind, value).map_or_else(Decimal::Nan, decimal)
    }
}

/// InputType::ValidationMessage, input_type.cc:620-728. Keep resource selection
/// and step candidates in the source owner; the embedding only formats text.
pub fn ValidationMessage(
    facts: &InputValidationFacts<'_>,
    validity: ValidityState,
    parser: &dyn NumberTypeParser<Decimal>,
    locale: &dyn super::constraint_validation_message::ConstraintValidationLocale,
) -> Vec<u16> {
    use super::constraint_validation_message::{
        ConstraintValidationText as L, NumericValidationMessage, NumericValidationText as N,
    };
    let temporal = !matches!(facts.kind, Type::kNumber | Type::kRange);
    let selected = if validity.bad_input {
        Some(if temporal {
            L::BadInputDateTime
        } else {
            L::BadInputNumber
        })
    } else if validity.value_missing {
        Some(match facts.kind {
            Type::kCheckbox => L::ValueMissingCheckbox,
            Type::kRadio => L::ValueMissingRadio,
            Type::kFile if facts.multiple => L::ValueMissingMultipleFile,
            Type::kFile => L::ValueMissingFile,
            _ => L::ValueMissing,
        })
    } else if validity.type_mismatch {
        Some(match facts.kind {
            Type::kURL => L::TypeMismatchURL,
            Type::kEmail => L::TypeMismatchEmail,
            _ => L::TypeMismatch,
        })
    } else if validity.pattern_mismatch {
        Some(L::PatternMismatch)
    } else if validity.too_long {
        Some(L::TooLong {
            length: facts.value.len(),
            limit: facts.max_length,
        })
    } else if validity.too_short {
        Some(L::TooShort {
            length: facts.value.len(),
            limit: facts.min_length,
        })
    } else {
        None
    };
    if let Some(selected) = selected {
        return locale.QueryValidationText(selected, &[], &[]);
    }
    if !facts.kind.IsSteppable() {
        return Vec::new();
    }
    let range = range(facts, parser);
    let format = if !range.HasStep() || range.Step().Remainder(decimal(60000)).IsZero() {
        temporal_input_type::SecondFormat::None
    } else if range.Step().Remainder(decimal(1000)).IsZero() {
        temporal_input_type::SecondFormat::Second
    } else {
        temporal_input_type::SecondFormat::Millisecond
    };
    NumericValidationMessage(
        parse(facts, facts.value, parser),
        range,
        |kind, first, second| {
            let serialize = |value: Decimal| {
                if temporal {
                    temporal_input_type::Serialize(
                        facts.kind,
                        parser.ParseToDoubleForNumberType(&value.ToString()),
                        format,
                    )
                } else {
                    parser
                        .SerializeForNumberType(value)
                        .encode_utf16()
                        .collect()
                }
            };
            let first = locale.LocalizeValue(facts.kind, &serialize(first));
            if matches!(kind, N::LocalizedValue) {
                return first;
            }
            let second = locale.LocalizeValue(facts.kind, &serialize(second));
            locale.QueryValidationText(
                match kind {
                    N::InvalidRange => L::RangeInvalid { temporal },
                    N::ReversedRange => L::ReversedRange,
                    N::NotEqual => L::ValueNotEqual,
                    N::Underflow => L::RangeUnderflow { temporal },
                    N::Overflow => L::RangeOverflow { temporal },
                    N::LocalizedValue => unreachable!(),
                },
                &first,
                &second,
            )
        },
        locale,
    )
}

/// BaseTemporalInputType::ShouldHaveSecondField and
/// DateTimeEditBuilder::NeedMillisecondField. `any` uses the default step for
/// the editor layout, unlike constraint validation's kRejectAny.
pub fn TemporalPrecision(
    kind: Type,
    value: &[u16],
    minimum: &[u16],
    step: &str,
    parser: &dyn NumberTypeParser<Decimal>,
) -> (bool, bool) {
    if !matches!(kind, Type::kTime | Type::kDateTimeLocal) {
        return (false, false);
    }
    let fields = temporal_input_type::DateTimeFieldsState::Parse(kind, value);
    let minimum = temporal_input_type::ParseToNumber(kind, minimum).map_or(decimal(0), decimal);
    let step = StepRange::ParseStep(
        AnyStepHandling::kAnyIsDefaultStep,
        StepDescription::new(
            60,
            0,
            1000,
            StepValueShouldBe::kScaledStepValueShouldBeInteger,
        ),
        step,
        parser,
    );
    let needs = |unit| {
        !minimum.Remainder(decimal(unit)).IsZero() || !step.Remainder(decimal(unit)).IsZero()
    };
    let milliseconds = fields.millisecond.is_some_and(|value| value != 0) || needs(1000);
    let seconds = fields.second.is_some_and(|value| value != 0) || milliseconds || needs(60000);
    (seconds, milliseconds)
}
/// Editor CreateStepRange(kAnyIsDefaultStep), carrying the source Decimal
/// values directly into DateTimeEditBuilder::CreateStep.
pub fn TemporalEditorStep(
    kind: Type,
    minimum: &[u16],
    value_attribute: &[u16],
    step: &str,
    parser: &dyn NumberTypeParser<Decimal>,
) -> (Decimal, Decimal) {
    let (default, base, scale, mode) = match kind {
        Type::kDate => (
            1,
            0,
            86400000,
            StepValueShouldBe::kParsedStepValueShouldBeInteger,
        ),
        Type::kMonth => (1, 0, 1, StepValueShouldBe::kParsedStepValueShouldBeInteger),
        Type::kWeek => (
            1,
            -259200000,
            604800000,
            StepValueShouldBe::kParsedStepValueShouldBeInteger,
        ),
        Type::kTime | Type::kDateTimeLocal => (
            60,
            0,
            1000,
            StepValueShouldBe::kScaledStepValueShouldBeInteger,
        ),
        _ => unreachable!("temporal type"),
    };
    let base = temporal_input_type::ParseToNumber(kind, minimum)
        .or_else(|| temporal_input_type::ParseToNumber(kind, value_attribute))
        .map_or(decimal(base), decimal);
    (
        StepRange::ParseStep(
            AnyStepHandling::kAnyIsDefaultStep,
            StepDescription::new(default, 0, scale, mode),
            step,
            parser,
        ),
        base,
    )
}
pub(super) fn range(
    facts: &InputValidationFacts<'_>,
    parser: &dyn NumberTypeParser<Decimal>,
) -> StepRange<Decimal> {
    use StepValueShouldBe::*;
    let (step, base, scale, mode, min, max, reversed) = match facts.kind {
        Type::kNumber => (
            1,
            0,
            1,
            kStepValueShouldBeReal,
            Decimal::FromComponents(true, 292, 17976931348623157),
            Decimal::FromComponents(false, 292, 17976931348623157),
            false,
        ),
        Type::kRange => (
            1,
            0,
            1,
            kStepValueShouldBeReal,
            decimal(0),
            decimal(100),
            false,
        ),
        Type::kDate => (
            1,
            0,
            86400000,
            kParsedStepValueShouldBeInteger,
            decimal(-62135596800000),
            decimal(8640000000000000),
            false,
        ),
        Type::kWeek => (
            1,
            -259200000,
            604800000,
            kParsedStepValueShouldBeInteger,
            decimal(-62135596800000),
            decimal(8639999568000000),
            false,
        ),
        Type::kMonth => (
            1,
            0,
            1,
            kParsedStepValueShouldBeInteger,
            decimal(-23628),
            decimal(3285488),
            false,
        ),
        Type::kTime => (
            60,
            0,
            1000,
            kScaledStepValueShouldBeInteger,
            decimal(0),
            decimal(86399999),
            true,
        ),
        Type::kDateTimeLocal => (
            60,
            0,
            1000,
            kScaledStepValueShouldBeInteger,
            decimal(-62135596800000),
            decimal(8640000000000000),
            false,
        ),
        _ => unreachable!("steppable type"),
    };
    let description = StepDescription::new(step, base, scale, mode);
    let minimum = parse(facts, facts.minimum, parser);
    let maximum = parse(facts, facts.maximum, parser);
    let has_min = minimum.IsFinite();
    let has_max = maximum.IsFinite();
    let step_base = if has_min {
        minimum
    } else {
        let value = parse(facts, facts.value_attribute, parser);
        if value.IsFinite() {
            value
        } else {
            decimal(base as i64)
        }
    };
    let minimum = if has_min { minimum } else { min };
    let mut maximum = if has_max { maximum } else { max };
    if facts.kind == Type::kRange && maximum < minimum {
        maximum = minimum;
    }
    let parsed_step =
        StepRange::ParseStep(AnyStepHandling::kRejectAny, description, facts.step, parser);
    StepRange::new(
        step_base,
        minimum,
        maximum,
        has_min,
        has_max,
        reversed && (has_min || has_max) && maximum < minimum,
        parsed_step,
        description,
    )
}
pub fn Evaluate(
    facts: &InputValidationFacts<'_>,
    pattern: &PatternCache,
    engine: Option<&dyn ValidationPatternEngine>,
    parser: &dyn NumberTypeParser<Decimal>,
    valid_absolute_url: impl FnOnce(&[u16]) -> bool,
) -> Result<ValidityState, ValidationRequired> {
    let mut result = ValidityState::default();
    result.value_missing = match facts.kind {
        Type::kCheckbox => super::checkbox_input_type::CheckboxInputType::ValueMissingWithState(
            facts.required,
            facts.checked,
        ),
        Type::kRadio => facts.radio_group_value_missing,
        Type::kFile => super::file_input_type::FileInputType::ValueMissingWithState(
            facts.required,
            facts.files_empty,
        ),
        Type::kRange
        | Type::kColor
        | Type::kHidden
        | Type::kButton
        | Type::kReset
        | Type::kSubmit
        | Type::kImage => false,
        _ => super::text_field_input_type::TextFieldInputType::ValueMissingWithState(
            facts.value,
            facts.required,
            facts.disabled_or_readonly,
        ),
    };
    if text(facts.kind) {
        result.too_long = BaseTextInputType::TooLongWithState(
            facts.value,
            facts.max_length,
            NeedsToCheckDirtyFlag::kCheckDirtyFlag,
            facts.dirty,
            facts.user_edit,
        );
        result.too_short = BaseTextInputType::TooShortWithState(
            facts.value,
            facts.min_length,
            NeedsToCheckDirtyFlag::kCheckDirtyFlag,
            facts.dirty,
            facts.user_edit,
        );
        if let Some(raw) = facts.pattern.as_ref().filter(|_| !facts.value.is_empty()) {
            let engine = engine.ok_or(ValidationRequired::PatternEngine)?;
            if facts.kind == Type::kEmail && facts.multiple {
                result.pattern_mismatch = EmailInputType::ParseMultipleValues(facts.value)
                    .any(|value| pattern.PatternMismatchPerValue(raw.clone(), value, engine));
            } else {
                result.pattern_mismatch =
                    pattern.PatternMismatchPerValue(raw.clone(), facts.value, engine);
            }
        }
    }
    if facts.kind == Type::kEmail && !facts.value.is_empty() {
        let regexp = engine
            .ok_or(ValidationRequired::PatternEngine)?
            .EnsureEmailRegexp();
        result.type_mismatch = EmailInputType::FindInvalidAddressWithRegexp(
            regexp.as_ref(),
            facts.value,
            facts.multiple,
        )
        .is_some();
    } else if facts.kind == Type::kURL {
        result.type_mismatch = !facts.value.is_empty() && !valid_absolute_url(facts.value);
    }
    if facts.kind.IsSteppable() {
        if !matches!(facts.kind, Type::kNumber | Type::kRange) {
            result.type_mismatch = !facts.value.is_empty()
                && temporal_input_type::ParseToNumber(facts.kind, facts.value).is_none();
        }
        let value = parse(facts, facts.value, parser);
        if value.IsFinite() {
            let range = range(facts, parser);
            let outside = value > range.Maximum() && value < range.Minimum();
            result.range_underflow = if range.HasReversedRange() {
                outside
            } else {
                value < range.Minimum()
            };
            result.range_overflow = if range.HasReversedRange() {
                outside
            } else {
                value > range.Maximum()
            };
            result.step_mismatch = range.StepMismatch(value);
        }
        if facts.kind == Type::kNumber {
            result.bad_input = !facts.inner_editor_value.is_empty()
                && !parser
                    .ParseToDoubleForNumberType(&String::from_utf16_lossy(facts.inner_editor_value))
                    .is_finite();
        }
    }
    Ok(result)
}
