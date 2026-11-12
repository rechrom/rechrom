// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/events/composition_event_init.idl:1-9
use super::ui_event_init::UIEventInit;

// Source: composition_event_init.idl:7-9
#[derive(Clone, Default)]
pub struct CompositionEventInit {
    pub ui_event_init: UIEventInit,
    pub data: String,
    pub data_utf16: Option<std::rc::Rc<[u16]>>,
}
