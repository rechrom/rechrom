// Copyright 2020 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/dom/focus_params.h:1-65

use std::rc::Rc;

use interaction_dispatch::core::events::ui_event::InputDeviceCapabilitiesRef;

use super::element::SelectionBehaviorOnFocus;
use crate::core::html::focus_options::FocusOptions;
use crate::core::page::focus_controller::FocusFrameRef;
use crate::public::mojom::input::focus_type::FocusType;

// Source: focus_params.h:17-31
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FocusTrigger {
    #[default]
    kScript,
    kUserGesture,
}

// Source: focus_params.h:33-62
#[derive(Clone)]
pub struct FocusParams {
    pub selection_behavior: SelectionBehaviorOnFocus,
    pub r#type: FocusType,
    pub source_capabilities: Option<InputDeviceCapabilitiesRef>,
    pub options: Rc<FocusOptions>,
    pub omit_blur_events: bool,
    pub focus_trigger: FocusTrigger,
    pub initiator_frame: Option<FocusFrameRef>,
}

impl FocusParams {
    // Source: focus_params.h:38-39
    pub fn WithTrigger(focus_trigger: FocusTrigger) -> Self {
        Self {
            focus_trigger,
            ..Self::default()
        }
    }

    // Source: focus_params.h:40-49
    pub fn new(
        selection_behavior: SelectionBehaviorOnFocus,
        focus_type: FocusType,
        source_capabilities: Option<InputDeviceCapabilitiesRef>,
        options: Rc<FocusOptions>,
        focus_trigger: FocusTrigger,
    ) -> Self {
        Self {
            selection_behavior,
            r#type: focus_type,
            source_capabilities,
            options,
            omit_blur_events: false,
            focus_trigger,
            initiator_frame: None,
        }
    }
}

// Source: focus_params.h:37,51-61
impl Default for FocusParams {
    fn default() -> Self {
        Self {
            selection_behavior: SelectionBehaviorOnFocus::kRestore,
            r#type: FocusType::kNone,
            source_capabilities: None,
            options: Rc::new(FocusOptions::default()),
            omit_blur_events: false,
            focus_trigger: FocusTrigger::kScript,
            initiator_frame: None,
        }
    }
}
