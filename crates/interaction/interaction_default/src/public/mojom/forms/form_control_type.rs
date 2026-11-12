// Copyright 2023 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/public/mojom/forms/form_control_type.mojom:1-47
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum FormControlType {
    kButtonButton,
    kButtonSubmit,
    kButtonReset,
    kButtonPopover,
    kFieldset,
    kInputButton,
    kInputCheckbox,
    kInputColor,
    kInputDate,
    kInputDatetimeLocal,
    kInputEmail,
    kInputFile,
    kInputHidden,
    kInputImage,
    kInputMonth,
    kInputNumber,
    kInputPassword,
    kInputRadio,
    kInputRange,
    kInputReset,
    kInputSearch,
    kInputSubmit,
    kInputTelephone,
    kInputText,
    kInputTime,
    kInputUrl,
    kInputWeek,
    kOutput,
    kSelectOne,
    kSelectMultiple,
    kTextArea,
}
