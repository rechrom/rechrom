// Copyright 2018 The Chromium Authors
// Use of this source code is governed by a BSD-style license in Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/public/web/web_autofill_state.h:10
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum WebAutofillState {
    kNotFilled = 0,
    kPreviewed,
    kAutofilled,
}
