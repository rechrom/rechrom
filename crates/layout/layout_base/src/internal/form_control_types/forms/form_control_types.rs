//! Form-algorithm control classification shared with the assembly facade.

// cpp: layoutng/internal/form_control_types.h:7-9
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum AutofillState {
    kNotFilled = 0,
    kPreviewed = 1,
    kAutofilled = 2,
}

// cpp: layoutng/internal/form_control_types.h:11-42
pub use foundation::FormControlType;
