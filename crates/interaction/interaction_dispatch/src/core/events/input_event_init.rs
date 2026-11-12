// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/events/input_event_init.idl:1-17

use super::input_event::DataTransferRef;
use super::input_event_range::StaticRangeRef;
use super::ui_event_init::UIEventInit;

// Source: input_event_init.idl:8-17
// Nested Option preserves the nullable data/dataTransfer value separately from
// presence. WebIDL defaults count as present for the defaulted members.
#[derive(Clone)]
pub struct InputEventInit {
    pub ui_event_init: UIEventInit,
    pub data: Option<Option<String>>,
    /// Exact WebIDL DOMString code units; takes precedence over UTF-8 input.
    pub data_utf16: Option<Option<std::rc::Rc<[u16]>>>,
    pub is_composing: Option<bool>,
    pub input_type: Option<String>,
    pub data_transfer: Option<Option<DataTransferRef>>,
    pub target_ranges: Option<Vec<StaticRangeRef>>,
}
impl Default for InputEventInit {
    fn default() -> Self {
        Self {
            ui_event_init: UIEventInit::default(),
            data: None,
            data_utf16: None,
            is_composing: Some(false),
            input_type: Some(String::new()),
            data_transfer: None,
            target_ranges: Some(Vec::new()),
        }
    }
}
