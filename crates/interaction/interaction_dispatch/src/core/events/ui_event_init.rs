// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/events/ui_event_init.idl:1-11

use crate::core::dom::events::event_init::EventInit;

use super::ui_event::{AbstractViewRef, InputDeviceCapabilitiesRef};

// Source: ui_event_init.idl:7-11
#[derive(Clone, Default)]
pub struct UIEventInit {
    pub event_init: EventInit,
    pub view: Option<AbstractViewRef>,
    pub detail: i32,
    pub source_capabilities: Option<InputDeviceCapabilitiesRef>,
}
