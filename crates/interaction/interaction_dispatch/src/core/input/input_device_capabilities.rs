// Copyright 2015 The Chromium Authors
// Use of this source code is governed by a BSD-style license in Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/input/input_device_capabilities.h:16-65
// Source: third_party/blink/renderer/core/input/input_device_capabilities.cc:11-32
use crate::core::events::ui_event::InputDeviceCapabilitiesRef;
use std::{cell::RefCell, rc::Rc};
pub struct InputDeviceCapabilities {
    fires_touch_events: bool,
}
impl InputDeviceCapabilities {
    pub fn Create(fires_touch_events: bool) -> Rc<Self> {
        Rc::new(Self { fires_touch_events })
    }
    pub fn firesTouchEvents(&self) -> bool {
        self.fires_touch_events
    }
}
impl crate::core::events::ui_event::InputDeviceCapabilities for InputDeviceCapabilities {
    fn firesTouchEvents(&self) -> bool {
        self.fires_touch_events
    }
}
#[derive(Default)]
pub struct InputDeviceCapabilitiesConstants {
    fires_touch_events: RefCell<Option<InputDeviceCapabilitiesRef>>,
    doesnt_fire_touch_events: RefCell<Option<InputDeviceCapabilitiesRef>>,
}
impl InputDeviceCapabilitiesConstants {
    pub fn FiresTouchEvents(&self, fires_touch: bool) -> InputDeviceCapabilitiesRef {
        let slot = if fires_touch {
            &self.fires_touch_events
        } else {
            &self.doesnt_fire_touch_events
        };
        let mut slot = slot.borrow_mut();
        slot.get_or_insert_with(|| InputDeviceCapabilities::Create(fires_touch))
            .clone()
    }
}
