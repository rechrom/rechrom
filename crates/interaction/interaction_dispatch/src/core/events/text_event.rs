// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/events/text_event.cc:40-49,76-102; text_event.h:70-98.
use super::ui_event::{AbstractViewRef, UIEvent, UIEventInterface};
use crate::core::dom::events::event::{
    Bubbles, Cancelable, ComposedMode, Event, EventClock, EventInterface,
};
use std::ops::{Deref, DerefMut};
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TextEventInputType {
    Keyboard,
    Composition,
    IncrementalInsertion,
    LineBreak,
    Paste,
    Drop,
}
pub struct TextEvent {
    ui_event: UIEvent,
    data: std::rc::Rc<[u16]>,
    input_type: TextEventInputType,
    pasting_fragment: Option<std::rc::Rc<dyn std::any::Any>>,
    data_transfer: Option<std::rc::Rc<crate::core::clipboard::data_transfer::DataTransfer>>,
    smart_replace: bool,
    match_style: bool,
}
impl TextEvent {
    pub fn new(
        view: Option<AbstractViewRef>,
        data: &[u16],
        input_type: TextEventInputType,
        clock: &dyn EventClock,
    ) -> Self {
        Self {
            ui_event: UIEvent::new(
                "textInput",
                Bubbles::kYes,
                Cancelable::kYes,
                ComposedMode::kComposed,
                clock.Now(),
                view,
                0,
                None,
            ),
            data: data.into(),
            input_type,
            pasting_fragment: None,
            data_transfer: None,
            smart_replace: false,
            match_style: false,
        }
    }
    // text_event.cc:44-57,88-102. The opaque fragment pins the exact native
    // DocumentFragment; dispatch neither depends on DOM nor copies its tree.
    pub fn CreateForPlainTextPaste(
        view: Option<AbstractViewRef>,
        text: &[u16],
        smart: bool,
        clock: &dyn EventClock,
    ) -> Self {
        let mut event = Self::new(view, text, TextEventInputType::Paste, clock);
        event.smart_replace = smart;
        event
    }
    pub fn CreateForFragmentPaste(
        view: Option<AbstractViewRef>,
        fragment: std::rc::Rc<dyn std::any::Any>,
        smart: bool,
        match_style: bool,
        transfer: std::rc::Rc<crate::core::clipboard::data_transfer::DataTransfer>,
        clock: &dyn EventClock,
    ) -> Self {
        let mut event = Self::new(view, &[], TextEventInputType::Paste, clock);
        event.pasting_fragment = Some(fragment);
        event.smart_replace = smart;
        event.match_style = match_style;
        event.data_transfer = Some(transfer);
        event
    }
    pub fn PastingFragment<T: std::any::Any + Clone>(&self) -> Option<T> {
        self.pasting_fragment.as_ref().map(|owner| {
            owner
                .downcast_ref::<T>()
                .expect("the frame's retained DOM fragment type")
                .clone()
        })
    }
    pub fn DataTransfer(
        &self,
    ) -> Option<std::rc::Rc<crate::core::clipboard::data_transfer::DataTransfer>> {
        self.data_transfer.clone()
    }
    pub fn ShouldSmartReplace(&self) -> bool {
        self.smart_replace
    }
    pub fn ShouldMatchStyle(&self) -> bool {
        self.match_style
    }
    pub fn data(&self) -> &[u16] {
        &self.data
    }
    pub fn dataUTF16(&self) -> std::rc::Rc<[u16]> {
        self.data.clone()
    }
    pub fn IsComposition(&self) -> bool {
        self.input_type == TextEventInputType::Composition
    }
    pub fn IsIncrementalInsertion(&self) -> bool {
        self.input_type == TextEventInputType::IncrementalInsertion
    }
    pub fn IsLineBreak(&self) -> bool {
        self.input_type == TextEventInputType::LineBreak
    }
    pub fn IsPaste(&self) -> bool {
        self.input_type == TextEventInputType::Paste
    }
    pub fn IsDrop(&self) -> bool {
        self.input_type == TextEventInputType::Drop
    }
}
impl Deref for TextEvent {
    type Target = UIEvent;
    fn deref(&self) -> &UIEvent {
        &self.ui_event
    }
}
impl DerefMut for TextEvent {
    fn deref_mut(&mut self) -> &mut UIEvent {
        &mut self.ui_event
    }
}
impl EventInterface for TextEvent {
    fn AsTextEvent(&self) -> Option<&TextEvent> {
        Some(self)
    }
    fn BaseEvent(&self) -> &Event {
        self.ui_event.BaseEvent()
    }
    fn BaseEventMut(&mut self) -> &mut Event {
        self.ui_event.BaseEventMut()
    }
    fn InterfaceName(&self) -> &str {
        "TextEvent"
    }
    fn IsUIEvent(&self) -> bool {
        true
    }
}
impl UIEventInterface for TextEvent {
    fn BaseUIEvent(&self) -> &UIEvent {
        &self.ui_event
    }
    fn BaseUIEventMut(&mut self) -> &mut UIEvent {
        &mut self.ui_event
    }
}
