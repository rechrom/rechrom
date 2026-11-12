// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/frame/scroll_options.idl:7-11

// Source: scroll_options.idl:7. This is the typed enum produced by the source
// bindings as V8ScrollBehavior::Enum; string conversion stays in the bindings.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollBehavior {
    #[default]
    kAuto,
    kInstant,
    kSmooth,
}

// Source: scroll_options.idl:9-11.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScrollOptions {
    pub behavior: ScrollBehavior,
}
