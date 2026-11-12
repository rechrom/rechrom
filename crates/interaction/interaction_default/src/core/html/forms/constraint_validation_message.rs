// Copyright The Chromium Authors. BSD license; see Chromium LICENSE.
// Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
// input_type.cc:620-728; html_text_area_element.cc:728-750.
use super::{
    step_range::{Decimal, StepRange},
    text_field_input_type::TextString,
};

pub enum ConstraintValidationText {
    BadInputNumber,
    BadInputDateTime,
    ValueMissing,
    ValueMissingCheckbox,
    ValueMissingRadio,
    ValueMissingFile,
    ValueMissingMultipleFile,
    ValueMissingSelect,
    TypeMismatch,
    TypeMismatchURL,
    TypeMismatchEmail,
    PatternMismatch,
    TooLong { length: usize, limit: i32 },
    TooShort { length: usize, limit: i32 },
    StepMismatchCloseToLimit,
    StepMismatch,
    RangeInvalid { temporal: bool },
    ReversedRange,
    ValueNotEqual,
    RangeUnderflow { temporal: bool },
    RangeOverflow { temporal: bool },
}
/// Locale/translated-resource formatting only. Constraint decisions and numeric
/// candidates are made by the same retained InputType, never by this port.
pub trait ConstraintValidationLocale {
    fn QueryValidationText(
        &self,
        kind: ConstraintValidationText,
        first: &[u16],
        second: &[u16],
    ) -> TextString;
    /// NumberInputType/BaseTemporalInputType::LocalizeValue. The value has
    /// already been serialized by the owning input type.
    fn LocalizeValue(&self, _kind: super::input_type::Type, value: &[u16]) -> TextString {
        value.to_vec()
    }
}
#[derive(Clone, Copy)]
pub(super) enum NumericValidationText {
    InvalidRange,
    ReversedRange,
    NotEqual,
    Underflow,
    Overflow,
    LocalizedValue,
}
pub(super) fn NumericValidationMessage<D: Decimal>(
    numeric_value: D,
    range: StepRange<D>,
    mut text: impl FnMut(NumericValidationText, D, D) -> TextString,
    locale: &dyn ConstraintValidationLocale,
) -> TextString {
    use NumericValidationText::*;
    if !numeric_value.IsFinite() {
        return Vec::new();
    }
    let min = range.Minimum();
    let max = range.Maximum();
    if min > max && !range.HasReversedRange() {
        return text(InvalidRange, min, max);
    }
    if range.HasReversedRange() && numeric_value < min && numeric_value > max {
        return text(ReversedRange, min, max);
    }
    if numeric_value != min && min == max {
        return text(NotEqual, min, max);
    }
    if numeric_value < min {
        return text(Underflow, min, max);
    }
    if numeric_value > max {
        return text(Overflow, max, min);
    }
    if range.StepMismatch(numeric_value) {
        debug_assert!(range.HasStep());
        let candidate1 = range.ClampValue(numeric_value);
        let localized1 = text(LocalizedValue, candidate1, candidate1);
        let candidate2 = if candidate1 < numeric_value {
            candidate1 + range.Step()
        } else {
            candidate1 - range.Step()
        };
        if !candidate2.IsFinite() || candidate2 < min || candidate2 > max {
            return locale.QueryValidationText(
                ConstraintValidationText::StepMismatchCloseToLimit,
                &localized1,
                &[],
            );
        }
        let localized2 = text(LocalizedValue, candidate2, candidate2);
        let (first, second) = if candidate1 < candidate2 {
            (&localized1, &localized2)
        } else {
            (&localized2, &localized1)
        };
        return locale.QueryValidationText(ConstraintValidationText::StepMismatch, first, second);
    }
    Vec::new()
}
