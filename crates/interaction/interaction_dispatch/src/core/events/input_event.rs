// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/events/input_event.h:1-137
// Source: third_party/blink/renderer/core/events/input_event.cc:1-260
// Rust modules replace includes/guards/namespaces and wrapper annotations.

pub use super::input_event_init::InputEventInit;
pub use super::input_event_range::{
    LiveRangeRef, RangeConversionError, RangeExceptionState, StaticRangeRef,
};
use super::ui_event::{UIEvent, UIEventInterface};
use super::ui_event_init::UIEventInit;
use crate::core::dom::events::event::{Event, EventClock, EventInterface};
use crate::core::dom::events::event_dispatch_result::DispatchEventResult;
use crate::core::dom::events::event_dispatcher::{EventDispatchHook, EventDispatcher};
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

// Source: input_event.h:15,112,127
// InputEvent retains DataTransfer unchanged and never reads its mutable model.
pub type DataTransferRef = Rc<crate::core::clipboard::data_transfer::DataTransfer>;

// Source: input_event.h:25-74
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum InputType {
    kNone,
    kInsertText,
    kInsertLineBreak,
    kInsertParagraph,
    kInsertOrderedList,
    kInsertUnorderedList,
    kInsertHorizontalRule,
    kInsertFromPaste,
    kInsertFromDrop,
    kInsertFromYank,
    kInsertTranspose,
    kInsertReplacementText,
    kInsertCompositionText,
    kInsertLink,
    kDeleteWordBackward,
    kDeleteWordForward,
    kDeleteSoftLineBackward,
    kDeleteSoftLineForward,
    kDeleteHardLineBackward,
    kDeleteHardLineForward,
    kDeleteContentBackward,
    kDeleteContentForward,
    kDeleteByCut,
    kDeleteByDrag,
    kHistoryUndo,
    kHistoryRedo,
    kFormatBold,
    kFormatItalic,
    kFormatUnderline,
    kFormatStrikeThrough,
    kFormatSuperscript,
    kFormatSubscript,
    kFormatJustifyCenter,
    kFormatJustifyFull,
    kFormatJustifyRight,
    kFormatJustifyLeft,
    kFormatIndent,
    kFormatOutdent,
    kFormatRemove,
    kFormatSetBlockTextDirection,
    kNumberOfInputTypes,
}

// Source: input_event.h:76-79
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventIsComposing {
    kNotComposing,
    kIsComposing,
}

// Source: input_event.h:17-18,124-132
pub struct InputEvent {
    ui_event: UIEvent,
    input_type_: InputType,
    data_: Option<Rc<[u16]>>,
    data_transfer_: Option<DataTransferRef>,
    is_composing_: bool,
    // This is the same live Range vector retained through dispatch. The hook
    // shares only its owner, releasing all event borrows before listener entry.
    ranges_: Rc<RefCell<Vec<LiveRangeRef>>>,
}

// Source: input_event.cc:20-82
const kInputTypeStringNameMap: [(InputType, &str); InputType::kNumberOfInputTypes as usize] = [
    (InputType::kNone, ""),
    (InputType::kInsertText, "insertText"),
    (InputType::kInsertLineBreak, "insertLineBreak"),
    (InputType::kInsertParagraph, "insertParagraph"),
    (InputType::kInsertOrderedList, "insertOrderedList"),
    (InputType::kInsertUnorderedList, "insertUnorderedList"),
    (InputType::kInsertHorizontalRule, "insertHorizontalRule"),
    (InputType::kInsertFromPaste, "insertFromPaste"),
    (InputType::kInsertFromDrop, "insertFromDrop"),
    (InputType::kInsertFromYank, "insertFromYank"),
    (InputType::kInsertTranspose, "insertTranspose"),
    (InputType::kInsertReplacementText, "insertReplacementText"),
    (InputType::kInsertCompositionText, "insertCompositionText"),
    (InputType::kInsertLink, "insertLink"),
    (InputType::kDeleteWordBackward, "deleteWordBackward"),
    (InputType::kDeleteWordForward, "deleteWordForward"),
    (InputType::kDeleteSoftLineBackward, "deleteSoftLineBackward"),
    (InputType::kDeleteSoftLineForward, "deleteSoftLineForward"),
    (InputType::kDeleteHardLineBackward, "deleteHardLineBackward"),
    (InputType::kDeleteHardLineForward, "deleteHardLineForward"),
    (InputType::kDeleteContentBackward, "deleteContentBackward"),
    (InputType::kDeleteContentForward, "deleteContentForward"),
    (InputType::kDeleteByCut, "deleteByCut"),
    (InputType::kDeleteByDrag, "deleteByDrag"),
    (InputType::kHistoryUndo, "historyUndo"),
    (InputType::kHistoryRedo, "historyRedo"),
    (InputType::kFormatBold, "formatBold"),
    (InputType::kFormatItalic, "formatItalic"),
    (InputType::kFormatUnderline, "formatUnderline"),
    (InputType::kFormatStrikeThrough, "formatStrikeThrough"),
    (InputType::kFormatSuperscript, "formatSuperscript"),
    (InputType::kFormatSubscript, "formatSubscript"),
    (InputType::kFormatJustifyCenter, "formatJustifyCenter"),
    (InputType::kFormatJustifyFull, "formatJustifyFull"),
    (InputType::kFormatJustifyRight, "formatJustifyRight"),
    (InputType::kFormatJustifyLeft, "formatJustifyLeft"),
    (InputType::kFormatIndent, "formatIndent"),
    (InputType::kFormatOutdent, "formatOutdent"),
    (InputType::kFormatRemove, "formatRemove"),
    (
        InputType::kFormatSetBlockTextDirection,
        "formatSetBlockTextDirection",
    ),
];

// Source: input_event.cc:84-94
fn ConvertInputTypeToString(input_type: InputType) -> &'static str {
    kInputTypeStringNameMap
        .get(input_type as usize)
        .map_or("", |entry| entry.1)
}
// Source: input_event.cc:96-103
fn ConvertStringToInputType(string_name: &str) -> InputType {
    for entry in &kInputTypeStringNameMap {
        if string_name == entry.1 {
            return entry.0;
        }
    }
    InputType::kNone
}
// Source: input_event.cc:105-107
fn InputTypeIsCancelable(input_type: InputType) -> bool {
    input_type != InputType::kInsertCompositionText
}

impl InputEvent {
    // Source: input_event.h:21-23; input_event.cc:111-121
    pub fn Create(
        event_type: impl Into<String>,
        initializer: &InputEventInit,
        exception_state: &mut RangeExceptionState,
        clock: &dyn EventClock,
    ) -> Option<Self> {
        assert!(!exception_state.HadException());
        let result = Self::WithInitializer(event_type, initializer, exception_state, clock);
        if exception_state.HadException() {
            return None;
        }
        Some(result)
    }

    // Source: input_event.h:99; input_event.cc:123-142
    pub fn WithInitializer(
        event_type: impl Into<String>,
        initializer: &InputEventInit,
        exception_state: &mut RangeExceptionState,
        clock: &dyn EventClock,
    ) -> Self {
        let mut result = Self {
            ui_event: UIEvent::WithInitializerAndClock(
                event_type,
                &initializer.ui_event_init,
                clock,
            ),
            input_type_: InputType::kNone,
            data_: None,
            data_transfer_: None,
            is_composing_: false,
            ranges_: Rc::new(RefCell::new(Vec::new())),
        };
        if let Some(input_type) = &initializer.input_type {
            result.input_type_ = ConvertStringToInputType(input_type);
        }
        if let Some(data) = &initializer.data_utf16 {
            result.data_ = data.clone();
        } else if let Some(data) = &initializer.data {
            result.data_ = data
                .as_ref()
                .map(|text| text.encode_utf16().collect::<Vec<_>>().into());
        }
        if let Some(data_transfer) = &initializer.data_transfer {
            result.data_transfer_ = data_transfer.clone();
        }
        if let Some(is_composing) = initializer.is_composing {
            result.is_composing_ = is_composing;
        }
        if let Some(ranges) = &initializer.target_ranges {
            for range in ranges {
                result
                    .ranges_
                    .borrow_mut()
                    .push(range.ToRange(exception_state));
            }
        }
        result
    }

    // Source: input_event.h:100-108; input_event.cc:144-161
    pub fn new(
        event_type: impl Into<String>,
        init: &UIEventInit,
        input_type: InputType,
        data: Option<String>,
        data_transfer: Option<DataTransferRef>,
        is_composing: EventIsComposing,
        ranges: Option<&[StaticRangeRef]>,
        clock: &dyn EventClock,
    ) -> Self {
        Self::newUTF16(
            event_type,
            init,
            input_type,
            data.map(|text| text.encode_utf16().collect::<Vec<_>>().into()),
            data_transfer,
            is_composing,
            ranges,
            clock,
        )
    }
    pub fn newUTF16(
        event_type: impl Into<String>,
        init: &UIEventInit,
        input_type: InputType,
        data: Option<Rc<[u16]>>,
        data_transfer: Option<DataTransferRef>,
        is_composing: EventIsComposing,
        ranges: Option<&[StaticRangeRef]>,
        clock: &dyn EventClock,
    ) -> Self {
        let result = Self {
            ui_event: UIEvent::WithInitializerAndClock(event_type, init, clock),
            input_type_: input_type,
            data_: data,
            data_transfer_: data_transfer,
            is_composing_: is_composing == EventIsComposing::kIsComposing,
            ranges_: Rc::new(RefCell::new(Vec::new())),
        };
        if let Some(ranges) = ranges {
            for range in ranges {
                let mut no_exception = RangeExceptionState::default();
                let live_range = range.ToRange(&mut no_exception);
                assert!(
                    !no_exception.HadException(),
                    "internal InputEvent range conversion must not throw"
                );
                result.ranges_.borrow_mut().push(live_range);
            }
        }
        result
    }

    // Source: input_event.h:81-84; input_event.cc:163-175
    pub fn CreateBeforeInput(
        input_type: InputType,
        data: String,
        is_composing: EventIsComposing,
        ranges: Option<&[StaticRangeRef]>,
        clock: &dyn EventClock,
    ) -> Self {
        Self::CreateBeforeInputUTF16(
            input_type,
            data.encode_utf16().collect::<Vec<_>>().into(),
            is_composing,
            ranges,
            clock,
        )
    }
    pub fn CreateBeforeInputUTF16(
        input_type: InputType,
        data: Rc<[u16]>,
        is_composing: EventIsComposing,
        ranges: Option<&[StaticRangeRef]>,
        clock: &dyn EventClock,
    ) -> Self {
        Self::CreateBeforeInputDataUTF16(input_type, Some(data), is_composing, ranges, clock)
    }
    /// Source null String remains distinct from an empty UTF-16 string.
    pub fn CreateBeforeInputDataUTF16(
        input_type: InputType,
        data: Option<Rc<[u16]>>,
        is_composing: EventIsComposing,
        ranges: Option<&[StaticRangeRef]>,
        clock: &dyn EventClock,
    ) -> Self {
        let mut init = UIEventInit::default();
        init.event_init.bubbles = true;
        init.event_init.cancelable = InputTypeIsCancelable(input_type);
        init.event_init.composed = true;
        Self::newUTF16(
            "beforeinput",
            &init,
            input_type,
            data,
            None,
            is_composing,
            ranges,
            clock,
        )
    }
    // Source: input_event.h:85-88; input_event.cc:177-189
    pub fn CreateBeforeInputWithDataTransfer(
        input_type: InputType,
        data_transfer: Option<DataTransferRef>,
        is_composing: EventIsComposing,
        ranges: Option<&[StaticRangeRef]>,
        clock: &dyn EventClock,
    ) -> Self {
        let mut init = UIEventInit::default();
        init.event_init.bubbles = true;
        init.event_init.cancelable = InputTypeIsCancelable(input_type);
        init.event_init.composed = true;
        Self::new(
            "beforeinput",
            &init,
            input_type,
            None,
            data_transfer,
            is_composing,
            ranges,
            clock,
        )
    }
    // Source: input_event.h:89-92; input_event.cc:191-203
    pub fn CreateInput(
        input_type: InputType,
        data: String,
        is_composing: EventIsComposing,
        ranges: Option<&[StaticRangeRef]>,
        clock: &dyn EventClock,
    ) -> Self {
        Self::CreateInputUTF16(
            input_type,
            data.encode_utf16().collect::<Vec<_>>().into(),
            is_composing,
            ranges,
            clock,
        )
    }
    pub fn CreateInputUTF16(
        input_type: InputType,
        data: Rc<[u16]>,
        is_composing: EventIsComposing,
        ranges: Option<&[StaticRangeRef]>,
        clock: &dyn EventClock,
    ) -> Self {
        Self::CreateInputDataUTF16(input_type, Some(data), is_composing, ranges, clock)
    }
    pub fn CreateInputDataUTF16(
        input_type: InputType,
        data: Option<Rc<[u16]>>,
        is_composing: EventIsComposing,
        ranges: Option<&[StaticRangeRef]>,
        clock: &dyn EventClock,
    ) -> Self {
        let mut init = UIEventInit::default();
        init.event_init.bubbles = true;
        init.event_init.cancelable = false;
        init.event_init.composed = true;
        Self::newUTF16(
            "input",
            &init,
            input_type,
            data,
            None,
            is_composing,
            ranges,
            clock,
        )
    }
    // Source: input_event.h:94-97; input_event.cc:205-217
    pub fn CreateInputWithDataTransfer(
        input_type: InputType,
        data_transfer: Option<DataTransferRef>,
        is_composing: EventIsComposing,
        ranges: Option<&[StaticRangeRef]>,
        clock: &dyn EventClock,
    ) -> Self {
        let mut init = UIEventInit::default();
        init.event_init.bubbles = true;
        init.event_init.cancelable = false;
        init.event_init.composed = true;
        Self::new(
            "input",
            &init,
            input_type,
            None,
            data_transfer,
            is_composing,
            ranges,
            clock,
        )
    }

    // Source: input_event.h:110; input_event.cc:219-221
    pub fn inputType(&self) -> &'static str {
        ConvertInputTypeToString(self.input_type_)
    }
    // Source: input_event.h:111-113
    pub fn data(&self) -> Option<&[u16]> {
        self.data_.as_deref()
    }
    pub fn dataUTF16(&self) -> Option<Rc<[u16]>> {
        self.data_.clone()
    }
    pub fn dataTransfer(&self) -> Option<DataTransferRef> {
        self.data_transfer_.clone()
    }
    pub fn isComposing(&self) -> bool {
        self.is_composing_
    }

    // Source: input_event.h:114-116; input_event.cc:223-228
    pub fn getTargetRanges(&self) -> Vec<StaticRangeRef> {
        let mut static_ranges = Vec::new();
        for range in self.ranges_.borrow().iter() {
            static_ranges.push(range.CreateStaticRange());
        }
        static_ranges
    }
    // Source: input_event.h:118; input_event.cc:230-232
    pub fn IsInputEvent(&self) -> bool {
        true
    }

    // Source: input_event.h:122; input_event.cc:234-238
    // Rc/Drop retain and release DataTransfer and each live Range owner.

    // Source: input_event.h:120; input_event.cc:240-257
    pub fn DispatchHook(&self) -> InputEventDispatchHook {
        InputEventDispatchHook {
            ranges: self.ranges_.clone(),
        }
    }
}

// Source: input_event.cc:240-257
// Captures the exact live Range owner while releasing EventRef's borrow before
// listener dispatch. Clearing after Dispatch preserves DOM-mutation tracking
// during listeners and stops the author-held event retaining live ranges.
pub struct InputEventDispatchHook {
    ranges: Rc<RefCell<Vec<LiveRangeRef>>>,
}
impl EventDispatchHook for InputEventDispatchHook {
    fn DispatchEvent(&self, dispatcher: &mut EventDispatcher<'_>) -> DispatchEventResult {
        let result = dispatcher.Dispatch();
        self.ranges.borrow_mut().clear();
        result
    }
}
impl Deref for InputEvent {
    type Target = UIEvent;
    fn deref(&self) -> &UIEvent {
        &self.ui_event
    }
}
impl DerefMut for InputEvent {
    fn deref_mut(&mut self) -> &mut UIEvent {
        &mut self.ui_event
    }
}
impl EventInterface for InputEvent {
    fn PrepareDispatchHook<'a>(
        &self,
        _nested: &'a dyn crate::core::events::mouse_event::MouseNestedDispatch,
    ) -> Box<dyn crate::core::dom::events::event_dispatcher::EventDispatchHook + 'a> {
        Box::new(self.DispatchHook())
    }
    fn AsInputEvent(&self) -> Option<&InputEvent> {
        Some(self)
    }
    fn AsInputEventMut(&mut self) -> Option<&mut InputEvent> {
        Some(self)
    }
    fn BaseEvent(&self) -> &Event {
        self.ui_event.BaseEvent()
    }
    fn BaseEventMut(&mut self) -> &mut Event {
        self.ui_event.BaseEventMut()
    }
    fn InterfaceName(&self) -> &str {
        "InputEvent"
    }
    fn IsUIEvent(&self) -> bool {
        true
    }
    fn IsInputEvent(&self) -> bool {
        true
    }
}
impl UIEventInterface for InputEvent {
    fn BaseUIEvent(&self) -> &UIEvent {
        &self.ui_event
    }
    fn BaseUIEventMut(&mut self) -> &mut UIEvent {
        &mut self.ui_event
    }
}
