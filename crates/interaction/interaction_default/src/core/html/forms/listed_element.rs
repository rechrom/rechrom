//! Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
//! listed_element.cc:436-510,582-665; validity_state.cc:31-79.
//! Retained control storage is supplied by the DOM owner. This module owns the
//! validation decision/dispatch order, independent of an execution thread.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ValidityState {
    pub value_missing: bool,
    pub type_mismatch: bool,
    pub pattern_mismatch: bool,
    pub too_long: bool,
    pub too_short: bool,
    pub range_underflow: bool,
    pub range_overflow: bool,
    pub step_mismatch: bool,
    pub bad_input: bool,
    pub custom_error: bool,
}
impl ValidityState {
    pub fn valid(&self) -> bool {
        !(self.type_mismatch
            || self.step_mismatch
            || self.range_underflow
            || self.range_overflow
            || self.too_long
            || self.too_short
            || self.pattern_mismatch
            || self.value_missing
            || self.bad_input
            || self.custom_error)
    }
}
pub trait ListedElementValidationOperations {
    fn WillValidate(&self) -> bool;
    fn Valid(&self) -> bool;
    fn DispatchInvalid(&self) -> bool;
    fn IsConnected(&self) -> bool;
    fn DocumentIdentity(&self) -> [u64; 2];
}
pub fn CheckValidity(
    operations: &dyn ListedElementValidationOperations,
    collect_unhandled: bool,
) -> super::html_form_element::FormControlValidity {
    if !operations.WillValidate() || operations.Valid() {
        return super::html_form_element::FormControlValidity {
            valid: true,
            unhandled: false,
        };
    }
    let document = operations.DocumentIdentity();
    let canceled = operations.DispatchInvalid();
    super::html_form_element::FormControlValidity {
        valid: false,
        unhandled: collect_unhandled
            && !canceled
            && operations.IsConnected()
            && operations.DocumentIdentity() == document,
    }
}
