// Copyright (C) 2004, 2005, 2006, 2007, 2008 Apple Inc. All rights reserved.
// This library is free software under the GNU Library General Public License,
// version 2 or (at your option) any later version; see Chromium COPYING.LIB.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/editing/ime/input_method_controller.h:56-78,89-117,186-230
// Source: third_party/blink/renderer/core/editing/ime/input_method_controller.cc:351-516,563-598,611-717,767-808,892-1203,1280-1292,1358-1375,1377-1440,1483-1490,2051-2068
// Geometry preparation remains at each annotated source synchronization point;
// the embedding synchronizes lifecycle after any preceding listener mutations.

use super::super::commands::typing_command::TextCompositionType;
use super::active_edit_context::ActiveEditContext;
use crate::core::html::forms::text_field_input_type::TextString;
use interaction_dispatch::core::{
    dom::events::{event::EventTargetRef, event_dispatch_result::DispatchEventResult},
    events::input_event::InputType,
};
use std::{
    cell::{Cell, RefCell},
    ops::Deref,
    rc::Rc,
};

// Source: input_method_controller.h:60-63; .cc:351.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ConfirmCompositionBehavior {
    kDoNotKeepSelection,
    kKeepSelection,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TypingContinuation {
    kContinue,
    kEnd,
}

// PlainTextRange is the source's small offset value, never a copied tree.
#[derive(Clone, Copy, Debug)]
pub struct PlainTextRange {
    pub start: i32,
    pub end: i32,
}
impl PlainTextRange {
    pub fn len(self) -> i32 {
        self.end - self.start
    }
}

/// The DOM's existing live Range owner. No new backing text is retained here.
pub trait CompositionRange {
    fn RootEditableNodeId(&self) -> Option<u64>;
    fn Collapsed(&self) -> bool;
    fn IsConnected(&self) -> bool;
    fn CollapseAtDocumentStart(&self);
    fn Text(&self) -> TextString;
    fn PlainTextEmittingOriginalText(&self) -> TextString;
    fn PlainTextRange(&self) -> Option<PlainTextRange>;
    fn DOMEndpoints(&self) -> Option<(CompositionEndpoint, CompositionEndpoint)>;
    fn SetStart(&self, endpoint: &CompositionEndpoint);
    fn SetEnd(&self, endpoint: &CompositionEndpoint);
}
pub type CompositionRangeRef = Rc<dyn CompositionRange>;

pub trait CompositionNode {
    /// Canonical DOM node identity; endpoints retain the source document owner.
    fn NodeId(&self) -> u64;
}
pub struct CompositionEndpoint {
    pub node: Rc<dyn CompositionNode>,
    pub offset: usize,
}
pub struct UndoCompositionSelection {
    pub anchor_is_text: bool,
    pub most_forward_anchor: CompositionEndpoint,
    pub focus: CompositionEndpoint,
}

/// Scope destruction publishes Chromium's already scoped input/compositionend
/// events. It does not buffer the normal synchronous beforeinput/update calls.
pub trait ScopedCompositionEvents {}
pub trait RevealCompositionSelectionScope {}

pub trait CompositionEvents {
    fn FocusedElement(&self) -> Option<EventTargetRef>;
    fn DispatchComposition(
        &self,
        target: &EventTargetRef,
        kind: &'static str,
        text: &[u16],
        scoped: bool,
    );
    fn DispatchBeforeInput(
        &self,
        target: &EventTargetRef,
        input_type: InputType,
        text: &[u16],
        composing: bool,
    ) -> DispatchEventResult;
    fn EventQueueScope(&self) -> Box<dyn ScopedCompositionEvents + '_>;
    fn EventsAreScoped(&self) -> bool;
    fn RevealSelectionScope(&self) -> Box<dyn RevealCompositionSelectionScope + '_>;
}

pub trait CompositionSelection {
    fn IsNone(&self) -> bool;
    fn IsRange(&self) -> bool;
    fn IsAnchorFirst(&self) -> bool;
    fn IsHandleVisible(&self) -> bool;
    fn SelectedText(&self) -> TextString;
    fn GetSelectionOffsets(&self) -> Option<PlainTextRange>;
    fn CanEdit(&self) -> bool;
    fn HasRootEditableElement(&self) -> bool;
    fn RootEditableNodeId(&self) -> Option<u64>;
    fn RootEditableTextLength(&self) -> Option<i32>;
    fn RestoreSelection(&self, range: PlainTextRange, forward: bool, show_handle: bool) -> bool;
    fn SetSelectionOffsets(&self, range: PlainTextRange, continuation: TypingContinuation) -> bool;
    fn SelectComposition(&self, range: &CompositionRangeRef);
    fn HandlingInputEvent(&self) -> Option<bool>;
    fn SetHandlingInputEvent(&self, handling: bool);
}

pub trait CompositionCommands {
    type ImeTextSpan;
    fn IsAvailable(&self) -> bool;
    fn HasDocument(&self) -> bool;
    fn NeedsIncrementalInsertion(&self, text: &[u16]) -> bool;
    fn DeleteSelection(&self);
    fn InsertCompositionText(
        &self,
        text: &[u16],
        select_inserted: bool,
        composition: TextCompositionType,
        incremental: bool,
    );
    fn HandleCompositionTextInput(&self, text: &[u16]);
    fn EditorInsertText(&self, text: &[u16]);
    fn CloseTyping(&self);
    fn UpdateOpenTypingCommandSelection(&self);
    fn ValidUndoEndingSelection(&self) -> Option<UndoCompositionSelection>;
    fn CreateCompositionRange(&self) -> CompositionRangeRef;
    fn IsTextTooLongAt(&self, range: &CompositionRangeRef) -> bool;
    fn RemoveSuggestionMarkerInRangeOnFinish(&self, range: &CompositionRangeRef);
    fn RemoveCompositionMarkers(&self);
    fn RemoveAutocorrectSuggestionMarkers(&self);
    fn AddDefaultCompositionMarker(&self, range: &CompositionRangeRef);
    fn AddImeTextSpans(&self, spans: &[Self::ImeTextSpan], root: u64, offset: i32);
    fn InvalidateCompositionAnchor(&self, range: &CompositionRangeRef);
}

/// The embedding prepares geometry at the named Chromium source line, after
/// any synchronous listener mutations. The controller never accesses rendering
/// engines; Page owns and executes this lifecycle synchronization.
pub trait PreparedCompositionGeometry {
    fn PrepareAt(&self, source_line: u32);
}

// Source: input_method_controller.h:189-194; .cc:353-383.
pub struct InputMethodController<'scope, C: CompositionCommands + 'scope> {
    state: Rc<InputMethodControllerState>,
    pub events: Rc<dyn CompositionEvents + 'scope>,
    pub selection: Rc<dyn CompositionSelection + 'scope>,
    pub commands: Rc<C>,
}

/// The fields retained by LocalFrame across native IME callbacks. Page builds
/// call-scoped event/command projections around this exact state so listener
/// reentry never has to be stored in the long-lived controller owner.
// Source: input_method_controller.h:189-194; .cc:353-383.
pub struct InputMethodControllerState {
    // Source input_method_controller.h:193; value-initialized null.
    active_edit_context: RefCell<Option<Rc<dyn ActiveEditContext>>>,
    pub has_composition: Cell<bool>,
    pub composition_range: RefCell<Option<CompositionRangeRef>>,
}

impl Default for InputMethodControllerState {
    fn default() -> Self {
        Self {
            active_edit_context: RefCell::new(None),
            has_composition: Cell::new(false),
            composition_range: RefCell::new(None),
        }
    }
}
impl InputMethodControllerState {
    /// The retained LocalFrame state is also borrowed by ClipboardCommands
    /// during synchronous paste/beforeinput listener reentry.
    pub fn GetActiveEditContext(&self) -> Option<Rc<dyn ActiveEditContext>> {
        self.active_edit_context.borrow().clone()
    }
}

impl<C: CompositionCommands> Deref for InputMethodController<'_, C> {
    type Target = InputMethodControllerState;
    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<'scope, C: CompositionCommands + 'scope> InputMethodController<'scope, C> {
    pub fn new(
        events: Rc<dyn CompositionEvents + 'scope>,
        selection: Rc<dyn CompositionSelection + 'scope>,
        commands: Rc<C>,
    ) -> Self {
        Self::WithState(
            events,
            selection,
            commands,
            Rc::new(InputMethodControllerState::default()),
        )
    }

    pub fn WithState(
        events: Rc<dyn CompositionEvents + 'scope>,
        selection: Rc<dyn CompositionSelection + 'scope>,
        commands: Rc<C>,
        state: Rc<InputMethodControllerState>,
    ) -> Self {
        Self {
            state,
            events,
            selection,
            commands,
        }
    }
    pub fn IsAvailable(&self) -> bool {
        self.commands.IsAvailable()
    }
    // Source input_method_controller.h:148-154. Retain the same EditContext.
    pub fn GetActiveEditContext(&self) -> Option<Rc<dyn ActiveEditContext>> {
        self.state.GetActiveEditContext()
    }
    pub fn SetActiveEditContext(&self, context: Option<Rc<dyn ActiveEditContext>>) {
        *self.active_edit_context.borrow_mut() = context;
    }
    // Source: input_method_controller.cc:368-371.
    pub fn HasComposition(&self) -> bool {
        self.has_composition.get()
            && self
                .composition_range
                .borrow()
                .as_ref()
                .is_some_and(|range| !range.Collapsed() && range.IsConnected())
    }
    // Source: input_method_controller.cc:385-426.
    pub fn DispatchCompositionUpdateEvent(&self, text: &[u16]) {
        if let Some(target) = self.events.FocusedElement() {
            self.events
                .DispatchComposition(&target, "compositionupdate", text, false);
        }
    }
    pub fn DispatchCompositionEndEvent(&self, text: &[u16]) {
        debug_assert!(self.events.EventsAreScoped());
        if let Some(target) = self.events.FocusedElement() {
            self.events
                .DispatchComposition(&target, "compositionend", text, true);
        }
    }
    pub fn DispatchBeforeInputFromComposition(
        &self,
        target: Option<&EventTargetRef>,
        input_type: InputType,
        data: &[u16],
    ) {
        if let Some(target) = target {
            self.events
                .DispatchBeforeInput(target, input_type, data, true);
        }
    }
    // Source: input_method_controller.cc:436-504.
    pub fn InsertTextDuringCompositionWithEvents(
        &self,
        text: &[u16],
        select_inserted: bool,
        composition: TextCompositionType,
        geometry: &dyn PreparedCompositionGeometry,
    ) {
        debug_assert!(self.events.EventsAreScoped());
        debug_assert!(composition != TextCompositionType::kTextCompositionNone);
        if !self.commands.HasDocument() {
            return;
        }
        let Some(target) = self.events.FocusedElement() else {
            return;
        };
        self.DispatchCompositionUpdateEvent(text);
        if !self.IsAvailable() {
            return;
        }
        self.DispatchBeforeInputFromComposition(
            Some(&target),
            InputType::kInsertCompositionText,
            text,
        );
        if !self.IsAvailable() {
            return;
        }
        geometry.PrepareAt(476);
        let incremental = self.commands.NeedsIncrementalInsertion(text);
        match composition {
            TextCompositionType::kTextCompositionUpdate
            | TextCompositionType::kTextCompositionConfirm => {
                if text.is_empty() {
                    self.commands.DeleteSelection();
                }
                geometry.PrepareAt(488);
                self.commands.InsertCompositionText(
                    text,
                    select_inserted,
                    composition,
                    incremental,
                );
            }
            TextCompositionType::kTextCompositionCancel => {
                self.commands.HandleCompositionTextInput(text)
            }
            TextCompositionType::kTextCompositionNone => unreachable!(),
        }
    }
    // Source: input_method_controller.cc:506-516,1370-1375.
    pub fn Clear(&self) {
        if self.HasComposition() {
            let range = self.composition_range.borrow().as_ref().unwrap().clone();
            self.commands.RemoveSuggestionMarkerInRangeOnFinish(&range);
        }
        self.has_composition.set(false);
        let range = self.composition_range.borrow().clone();
        if let Some(range) = range {
            range.CollapseAtDocumentStart();
        }
        self.commands.RemoveCompositionMarkers();
    }
    // Source: input_method_controller.cc:563-598.
    pub fn SelectComposition(&self) {
        if !self.HasComposition() {
            return;
        }
        let range = self.composition_range.borrow().as_ref().unwrap().clone();
        let handling = self.selection.HandlingInputEvent();
        if handling.is_some() {
            self.selection.SetHandlingInputEvent(false);
        }
        self.selection.SelectComposition(&range);
        if let Some(handling) = handling {
            self.selection.SetHandlingInputEvent(handling);
        }
    }
    // Source: input_method_controller.cc:611-692.
    pub fn FinishComposingText(
        &self,
        behavior: ConfirmCompositionBehavior,
        geometry: &dyn PreparedCompositionGeometry,
    ) -> bool {
        if !self.HasComposition() {
            return false;
        }
        let range = self.composition_range.borrow().as_ref().unwrap().clone();
        let too_long = self.commands.IsTextTooLongAt(&range);
        geometry.PrepareAt(622);
        let composing = self.ComposingText();
        let _scope = self.events.EventQueueScope();
        if behavior == ConfirmCompositionBehavior::kKeepSelection {
            let handles = self.selection.IsHandleVisible();
            let offsets = self.selection.GetSelectionOffsets();
            let forward = self.selection.IsAnchorFirst();
            let _reveal = self.events.RevealSelectionScope();
            if too_long {
                let _ = self.ReplaceComposition(&self.ComposingText(), geometry);
            } else {
                self.Clear();
                self.DispatchCompositionEndEvent(&composing);
            }
            geometry.PrepareAt(651);
            let Some(offsets) = offsets else {
                return false;
            };
            return self.selection.RestoreSelection(offsets, forward, handles);
        }
        let Some(range) = range.PlainTextRange() else {
            return false;
        };
        if too_long {
            if !self.ReplaceComposition(&self.ComposingText(), geometry) {
                return false;
            }
        } else {
            self.Clear();
            self.DispatchCompositionEndEvent(&composing);
        }
        self.MoveCaret(range.end, geometry)
    }
    // Source: input_method_controller.cc:694-717.
    pub fn CommitText(
        &self,
        text: &[u16],
        spans: &[C::ImeTextSpan],
        relative_caret: i32,
        geometry: &dyn PreparedCompositionGeometry,
    ) -> bool {
        let result = if self.HasComposition() {
            self.ReplaceCompositionAndMoveCaret(text, relative_caret, spans, geometry)
        } else {
            self.InsertTextAndMoveCaret(text, relative_caret, spans, geometry)
        };
        // TODO: input_method_controller.cc:708-713 AX HandleCommitText.
        result
    }
    // Source: input_method_controller.cc:767-801.
    pub fn ReplaceComposition(
        &self,
        text: &[u16],
        geometry: &dyn PreparedCompositionGeometry,
    ) -> bool {
        debug_assert!(self.events.EventsAreScoped());
        if !self.HasComposition() {
            return false;
        }
        self.SelectComposition();
        if self.selection.IsNone() || !self.IsAvailable() {
            return false;
        }
        self.Clear();
        self.InsertTextDuringCompositionWithEvents(
            text,
            false,
            TextCompositionType::kTextCompositionConfirm,
            geometry,
        );
        if !self.IsAvailable() {
            return false;
        }
        self.DispatchCompositionEndEvent(text);
        true
    }
    // Source: input_method_controller.cc:892-922.
    pub fn ReplaceCompositionAndMoveCaret(
        &self,
        text: &[u16],
        relative_caret: i32,
        spans: &[C::ImeTextSpan],
        geometry: &dyn PreparedCompositionGeometry,
    ) -> bool {
        let Some(root) = self.selection.RootEditableNodeId() else {
            return false;
        };
        debug_assert!(self.HasComposition());
        let range = self
            .composition_range
            .borrow()
            .as_ref()
            .and_then(|range| range.PlainTextRange());
        let Some(range) = range else {
            return false;
        };
        let _scope = self.events.EventQueueScope();
        if !self.ReplaceComposition(text, geometry) {
            return false;
        }
        geometry.PrepareAt(915);
        self.commands.AddImeTextSpans(spans, root, range.start);
        self.MoveCaret(
            ComputeAbsoluteCaretPosition(range.start, text.len() as i32, relative_caret),
            geometry,
        )
    }
    // Source: input_method_controller.cc:924-964.
    pub fn InsertText(&self, text: &[u16]) -> bool {
        if let Some(target) = self.events.FocusedElement() {
            if self
                .events
                .DispatchBeforeInput(&target, InputType::kInsertText, text, false)
                != DispatchEventResult::kNotCanceled
            {
                return false;
            }
        }
        if !self.IsAvailable() {
            return false;
        }
        self.commands.EditorInsertText(text);
        true
    }
    pub fn InsertTextAndMoveCaret(
        &self,
        text: &[u16],
        relative_caret: i32,
        spans: &[C::ImeTextSpan],
        geometry: &dyn PreparedCompositionGeometry,
    ) -> bool {
        let Some(range) = self.selection.GetSelectionOffsets() else {
            return false;
        };
        let _scope = self.events.EventQueueScope();
        if (!text.is_empty() || range.len() > 0) && !self.InsertText(text) {
            return false;
        }
        geometry.PrepareAt(954);
        if let Some(root) = self.selection.RootEditableNodeId() {
            self.commands.AddImeTextSpans(spans, root, range.start);
        }
        self.MoveCaret(
            ComputeAbsoluteCaretPosition(range.start, text.len() as i32, relative_caret),
            geometry,
        )
    }
    // Source: input_method_controller.cc:966-1007.
    pub fn CancelComposition(&self, geometry: &dyn PreparedCompositionGeometry) {
        if !self.HasComposition() {
            return;
        }
        geometry.PrepareAt(972);
        let _reveal = self.events.RevealSelectionScope();
        if self.selection.IsNone() {
            return;
        }
        self.Clear();
        self.InsertTextDuringCompositionWithEvents(
            &[],
            false,
            TextCompositionType::kTextCompositionCancel,
            geometry,
        );
        if !self.IsAvailable() {
            return;
        }
        self.commands.CloseTyping();
        self.DispatchCompositionEndEvent(&[]);
    }
    pub fn DispatchCompositionStartEvent(&self, text: &[u16]) -> bool {
        let Some(target) = self.events.FocusedElement() else {
            return self.IsAvailable();
        };
        self.events
            .DispatchComposition(&target, "compositionstart", text, false);
        self.IsAvailable()
    }
    // Source: input_method_controller.cc:1009-1192.
    pub fn SetComposition(
        &self,
        text: &[u16],
        spans: &[C::ImeTextSpan],
        selection_start: i32,
        selection_end: i32,
        geometry: &dyn PreparedCompositionGeometry,
    ) {
        let _reveal = self.events.RevealSelectionScope();
        geometry.PrepareAt(1020);
        self.SelectComposition();
        geometry.PrepareAt(1026);
        if self.selection.IsNone() || self.events.FocusedElement().is_none() {
            return;
        }
        let selected =
            self.CreateSelectionRangeForSetComposition(selection_start, selection_end, text.len());
        if text.is_empty() {
            let _scope = self.events.EventQueueScope();
            if self.HasComposition() {
                let _inner_reveal = self.events.RevealSelectionScope();
                if !self.ReplaceComposition(&[], geometry) {
                    return;
                }
            } else if !self.DeleteSelection(geometry) {
                return;
            }
            geometry.PrepareAt(1074);
            if let Some(selected) = selected {
                self.SetEditableSelectionOffsets(selected, TypingContinuation::kEnd);
            }
            return;
        }
        if !self.HasComposition()
            && !self.DispatchCompositionStartEvent(&self.selection.SelectedText())
        {
            return;
        }
        self.Clear();
        let _scope = self.events.EventQueueScope();
        self.InsertTextDuringCompositionWithEvents(
            text,
            true,
            TextCompositionType::kTextCompositionUpdate,
            geometry,
        );
        if !self.IsAvailable() {
            return;
        }
        geometry.PrepareAt(1103);
        let Some(ending) = self.commands.ValidUndoEndingSelection() else {
            return;
        };
        if !ending.anchor_is_text {
            return;
        }
        self.has_composition.set(true);
        let existing = self.composition_range.borrow().clone();
        let range = existing.unwrap_or_else(|| self.commands.CreateCompositionRange());
        range.SetStart(&ending.most_forward_anchor);
        range.SetEnd(&ending.focus);
        *self.composition_range.borrow_mut() = Some(range.clone());
        // TODO: input_method_controller.cc:1137-1141 AX HandleSetComposition.
        self.commands.InvalidateCompositionAnchor(&range);
        geometry.PrepareAt(1149);
        if let Some(selected) = selected {
            self.SetEditableSelectionOffsets(selected, TypingContinuation::kContinue);
        }
        self.commands.UpdateOpenTypingCommandSelection();
        if !self.HasComposition() {
            return;
        }
        if spans.is_empty() {
            self.commands.AddDefaultCompositionMarker(&range);
            return;
        }
        if let (Some(offsets), Some(root)) = (range.PlainTextRange(), range.RootEditableNodeId()) {
            self.commands.AddImeTextSpans(spans, root, offsets.start);
        }
    }
    // Source: input_method_controller.cc:1194-1203.
    pub fn CreateSelectionRangeForSetComposition(
        &self,
        start: i32,
        end: i32,
        text_length: usize,
    ) -> Option<PlainTextRange> {
        let offsets = self.selection.GetSelectionOffsets()?;
        self.CreateRangeForSelection(offsets.start + start, offsets.start + end, text_length)
    }
    // Source: input_method_controller.cc:1280-1292.
    pub fn ComposingText(&self) -> TextString {
        if !self.HasComposition() {
            return Vec::new();
        }
        self.composition_range
            .borrow()
            .as_ref()
            .unwrap()
            .PlainTextEmittingOriginalText()
    }
    // Source: input_method_controller.cc:1358-1368.
    pub fn SetEditableSelectionOffsets(
        &self,
        offsets: PlainTextRange,
        continuation: TypingContinuation,
    ) -> bool {
        self.selection.CanEdit() && self.selection.SetSelectionOffsets(offsets, continuation)
    }
    // Source: input_method_controller.cc:1377-1415.
    pub fn CreateRangeForSelection(
        &self,
        mut start: i32,
        mut end: i32,
        text_length: usize,
    ) -> Option<PlainTextRange> {
        start = start.max(0);
        end = end.max(start);
        let mut boundary = self.selection.RootEditableTextLength()?;
        if self.HasComposition() {
            boundary -= self
                .composition_range
                .borrow()
                .as_ref()
                .unwrap()
                .Text()
                .len() as i32;
        }
        boundary += text_length as i32;
        Some(PlainTextRange {
            start: start.min(boundary),
            end: end.min(boundary),
        })
    }
    // Source: input_method_controller.cc:1417-1440.
    pub fn DeleteSelection(&self, geometry: &dyn PreparedCompositionGeometry) -> bool {
        if !self.selection.IsRange() {
            return true;
        }
        if let Some(target) = self.events.FocusedElement() {
            self.events
                .DispatchBeforeInput(&target, InputType::kDeleteContentBackward, &[], false);
            if !self.IsAvailable() {
                return false;
            }
        }
        geometry.PrepareAt(1435);
        self.commands.DeleteSelection();
        self.IsAvailable()
    }
    // Source: input_method_controller.cc:1483-1490.
    pub fn MoveCaret(&self, position: i32, geometry: &dyn PreparedCompositionGeometry) -> bool {
        geometry.PrepareAt(1484);
        let Some(range) = self.CreateRangeForSelection(position, position, 0) else {
            return false;
        };
        self.SetEditableSelectionOffsets(range, TypingContinuation::kEnd)
    }
    // Source: input_method_controller.cc:2051-2060. Rc/Drop handles 2062-2068.
    pub fn WillChangeFocus(&self, geometry: &dyn PreparedCompositionGeometry) {
        self.FinishComposingText(ConfirmCompositionBehavior::kKeepSelection, geometry);
        if !self.IsAvailable() {
            return;
        }
        self.commands.RemoveAutocorrectSuggestionMarkers();
    }
}
// Source: input_method_controller.cc:803-808.
pub fn ComputeAbsoluteCaretPosition(start: i32, length: i32, relative: i32) -> i32 {
    start + length + relative
}

pub trait ImeTextSpanContext: super::super::iterators::text_iterator::TextIteratorContext {
    type FlatContext: super::super::position::PositionDOM<Node = Self::Node>;
    fn FlatContext(&self) -> Self::FlatContext;
    fn ToFlatTreePosition(
        &self,
        flat: &Self::FlatContext,
        position: &super::super::position::Position<Self::Node>,
    ) -> super::super::position::Position<Self::Node>;
    fn ImeSpanRange(
        &self,
        root: &Self::Node,
        start: usize,
        end: usize,
    ) -> Option<(
        super::super::position::Position<Self::Node>,
        super::super::position::Position<Self::Node>,
    )>;
    fn SpellCheckingEnabledAt(
        &self,
        position: &super::super::position::Position<Self::Node>,
    ) -> bool;
}

// Source: input_method_controller.cc:810-887. Marker storage and invalidation
// stay with the original DocumentMarkerController; the host only supplies the
// live PlainTextRange and spellcheck leaves.
pub fn AddImeTextSpans<C>(
    context: &C,
    markers: &super::super::markers::document_marker_controller::DocumentMarkerController<C::Node>,
    spans: &[super::ime_text_span::ImeTextSpan],
    root: &C::Node,
    offset: usize,
) -> Vec<super::super::markers::document_marker_controller::MarkerEffect<C::Node>>
where
    C: ImeTextSpanContext,
    C::Node: Clone + Eq + std::hash::Hash + 'static,
{
    use super::super::markers::{
        document_marker_controller::{MarkerType, MarkerTypes},
        styleable_marker::{ImeTextSpanUnderlineStyle, SuggestionMarkerProperties, SuggestionType},
    };
    use super::ime_text_span::Type;
    let mut effects = Vec::new();
    for span in spans {
        let (Some(start), Some(end)) = (
            offset.checked_add(span.StartOffset()),
            offset.checked_add(span.EndOffset()),
        ) else {
            continue;
        };
        let Some((start, end)) = context.ImeSpanRange(root, start, end) else {
            continue;
        };
        match span.kind {
            Type::kComposition => {
                let mut style = span.style.clone();
                if span.interim_char_selection {
                    style.underline_style = ImeTextSpanUnderlineStyle::Solid;
                }
                effects.extend(markers.AddCompositionMarker(context, &start, &end, &style));
            }
            Type::kPreviewStylusGesture => effects.extend(markers.AddPreviewStylusGestureMarker(
                context,
                &start,
                &end,
                span.style.background_color,
            )),
            kind => {
                let kind = match kind {
                    Type::kAutocorrect => SuggestionType::Autocorrect,
                    Type::kGrammarSuggestion => SuggestionType::Grammar,
                    Type::kMisspellingSuggestion => SuggestionType::Misspelling,
                    Type::kSuggestion => SuggestionType::NotMisspelling,
                    _ => unreachable!(),
                };
                if kind == SuggestionType::Misspelling && !context.SpellCheckingEnabledAt(&start) {
                    continue;
                }
                if kind == SuggestionType::Grammar {
                    let flat = context.FlatContext();
                    let flat_start = context.ToFlatTreePosition(&flat, &start);
                    let flat_end = context.ToFlatTreePosition(&flat, &end);
                    if !markers
                        .MarkersIntersectingRange(
                            &flat,
                            &flat_start,
                            &flat_end,
                            MarkerTypes::Single(MarkerType::Spelling),
                        )
                        .is_empty()
                    {
                        continue;
                    }
                }
                effects.extend(markers.AddSuggestionMarker(
                    context,
                    &start,
                    &end,
                    &SuggestionMarkerProperties {
                        style: span.style.clone(),
                        kind,
                        remove_on_finish_composing: span.remove_on_finish_composing,
                        suggestions: span.suggestions.clone(),
                        highlight_color: span.suggestion_highlight_color,
                        hide_suggestion_menu: span.should_hide_suggestion_menu,
                    },
                ));
            }
        }
    }
    effects
}
