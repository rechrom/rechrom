// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/dom/events/event_modifier_init.idl:1-19

use crate::core::events::ui_event_init::UIEventInit;

// Source: event_modifier_init.idl:7-19
#[derive(Clone, Default)]
pub struct EventModifierInit {
    pub ui_event_init: UIEventInit,
    pub ctrl_key: bool,
    pub shift_key: bool,
    pub alt_key: bool,
    pub meta_key: bool,
    pub modifier_alt_graph: bool,
    pub modifier_caps_lock: bool,
    pub modifier_fn: bool,
    pub modifier_num_lock: bool,
    pub modifier_scroll_lock: bool,
    pub modifier_symbol: bool,
}
