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
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum FormControlType {
    kButtonButton = 0,
    kButtonSubmit = 1,
    kButtonReset = 2,
    kButtonPopover = 3,
    kFieldset = 4,
    kInputButton = 5,
    kInputCheckbox = 6,
    kInputColor = 7,
    kInputDate = 8,
    kInputDatetimeLocal = 9,
    kInputEmail = 10,
    kInputFile = 11,
    kInputHidden = 12,
    kInputImage = 13,
    kInputMonth = 14,
    kInputNumber = 15,
    kInputPassword = 16,
    kInputRadio = 17,
    kInputRange = 18,
    kInputReset = 19,
    kInputSearch = 20,
    kInputSubmit = 21,
    kInputTelephone = 22,
    kInputText = 23,
    kInputTime = 24,
    kInputUrl = 25,
    kInputWeek = 26,
    kOutput = 27,
    kSelectOne = 28,
    kSelectMultiple = 29,
    kTextArea = 30,
}
