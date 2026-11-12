// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/events/focus_event_init.idl:1-9

use super::ui_event_init::UIEventInit;
use crate::core::dom::events::event::EventTargetRef;

// Source: focus_event_init.idl:7-9
#[derive(Clone, Default)]
pub struct FocusEventInit {
    pub ui_event_init: UIEventInit,
    pub related_target: Option<EventTargetRef>,
}
