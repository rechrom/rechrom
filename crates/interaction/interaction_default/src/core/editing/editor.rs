// Copyright (C) 2005, 2006, 2007, 2008, 2009 Apple Inc. All rights reserved.
// This library is free software under the GNU Library General Public License,
// version 2 or (at your option) any later version; see Chromium COPYING.LIB.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/editing/editor.cc:114-134,145-214,256-319,486-559
// Corresponding declarations: editor.h:83-86,105-109,128-138,166,198-210.

use crate::core::html::forms::text_field_input_type::TextString;
use interaction_dispatch::core::dom::events::event::EventRef;
use interaction_dispatch::core::events::input_event::InputType;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

pub use super::commands::edit_command::PasswordEchoBehavior;

/// Immutable selection values retain the owning DOM positions. Implementations
/// do not create a flattened text or geometry tree at this boundary.
pub trait EditorSelection<S: Clone> {
    fn IsNone(&self, selection: &S) -> bool;
    fn IsRangeForEditing(&self, selection: &S) -> bool;
    fn IsContentEditable(&self, selection: &S) -> bool;
    fn IsRichlyEditable(&self, selection: &S) -> bool;
    fn Current(&self) -> S;
    fn EventTargetIsTextControl(&self, event: &EventRef) -> bool;
    fn CurrentSelectionBelongsToEventTarget(&self, event: &EventRef, selection: &S) -> bool;
    fn EventTargetSavedSelection(&self, event: &EventRef) -> S;
    fn CreateVisibleSelection(&self, selection: &S) -> S;
    fn HasRootEditableElementForSelection(&self, selection: &S) -> bool;
    fn RevealFocusedOrMainFrameSelection(&self, selection: &S);
    fn RevealSelectionAfterEditingOperation(&self);
}
// Source: editor.cc:216-242. The flat visible-selection query retains its
// original geometry owner. ImageDocument and password-reveal facts belong to
// their source owners and must not be inferred from URL or password text.
pub struct VisibleSelectionCopyFacts {
    pub is_range: bool,
    pub starts_in_unrevealed_password: bool,
}
pub trait EditorCopyContext {
    fn ImageElementFromImageDocument(&self) -> bool;
    fn IsSelectionAvailable(&self) -> bool;
    fn UpdateStyleAndLayoutForEditing(&self);
    fn VisibleSelectionInFlatTree(&self) -> VisibleSelectionCopyFacts;
}

/// The translated command owners consume these exact source command inputs.
/// Editor retains command selection/order; command execution stays with its
/// existing TypingCommand/DeleteSelectionCommand/ReplaceSelectionCommand owner.
pub trait EditorCommands<S: Clone> {
    type Fragment: Clone;
    type DataTransfer: Clone;
    type TypingStyle;
    fn HandleTextInputEvent(
        &self,
        text: &[u16],
        triggering_keyboard_event: Option<&EventRef>,
    ) -> bool;
    fn InsertText(
        &self,
        text: &[u16],
        selection: &S,
        select_inserted: bool,
        password_echo: PasswordEchoBehavior,
        composition_confirm: bool,
        input_type: InputType,
        transfer: Option<&Self::DataTransfer>,
    ) -> bool;
    fn InsertLineBreak(&self) -> bool;
    fn InsertParagraphSeparator(&self) -> bool;
    fn CreateFragmentFromText(&self, text: &[u16]) -> Option<Self::Fragment>;
    fn ReplaceSelection(
        &self,
        fragment: &Self::Fragment,
        options: ReplaceSelectionCommandOptions,
        password_echo: PasswordEchoBehavior,
        input_type: InputType,
        transfer: Option<&Self::DataTransfer>,
    );
}

/// Deletion is a separate command capability; text-event owners do not need
/// unrelated deletion ports to execute Editor::HandleTextEvent.
pub trait EditorDeletionCommands<S: Clone>: EditorCommands<S> {
    type Position;
    fn DeleteSelection(&self, options: DeleteSelectionCommandInput<'_, Self::Position>);
}

// Source: editor.cc:267-275. The receiving command gets every source option,
// including the reference move position retained by the initiating operation.
pub struct DeleteSelectionCommandInput<'a, P> {
    pub smart_delete: bool,
    pub merge_blocks_after_delete: bool,
    pub expand_for_special_elements: bool,
    pub sanitize_markup: bool,
    pub input_type: InputType,
    pub reference_move_position: &'a P,
}
// Source: editor.cc:293-301.
pub struct ReplaceSelectionCommandOptions {
    pub prevent_nesting: bool,
    pub sanitize_fragment: bool,
    pub select_replacement: bool,
    pub smart_replace: bool,
    pub match_style: bool,
}

/// A token acquired by the facade at editor.cc:158, after event dispatch. All
/// selection/command inputs below borrow that generation's resident geometry.
pub struct PreparedEditorGeometry;

pub struct TextEventData<F, D> {
    pub event: EventRef,
    pub data: TextString,
    pub is_drop: bool,
    pub is_incremental_insertion: bool,
    pub is_paste: bool,
    pub is_line_break: bool,
    pub is_composition: bool,
    pub smart_replace: bool,
    pub match_style: bool,
    pub pasting_fragment: Option<F>,
    pub data_transfer: Option<D>,
    // editor.cc:183-196; WebKeyboardEvent's confirmation fact is Android-only.
    #[cfg(target_os = "android")]
    pub confirmed_physical_keyboard: bool,
}
/// The source paste branch requires only the retained replacement-command
/// owner. Input/IME ports need not rebuild Editor's unrelated command surface.
pub trait EditorPasteCommands<F, D> {
    type Error;
    fn ReplacePasteFragment(
        &self,
        fragment: &F,
        smart: bool,
        match_style: bool,
        transfer: Option<&D>,
    ) -> Result<(), Self::Error>;
    fn ReplacePasteText(&self, text: &[u16], smart: bool) -> Result<(), Self::Error>;
}
pub fn HandlePasteTextEvent<F, D, C: EditorPasteCommands<F, D>>(
    commands: &C,
    event: &TextEventData<F, D>,
) -> Result<(), C::Error> {
    if let Some(fragment) = &event.pasting_fragment {
        commands.ReplacePasteFragment(
            fragment,
            event.smart_replace,
            event.match_style,
            event.data_transfer.as_ref(),
        )
    } else {
        commands.ReplacePasteText(&event.data, event.smart_replace)
    }
}
impl<'scope, S: Clone, C: EditorCommands<S>> EditorPasteCommands<C::Fragment, C::DataTransfer>
    for Editor<'scope, S, C>
{
    type Error = std::convert::Infallible;
    fn ReplacePasteFragment(
        &self,
        fragment: &C::Fragment,
        smart: bool,
        match_style: bool,
        transfer: Option<&C::DataTransfer>,
    ) -> Result<(), Self::Error> {
        self.ReplaceSelectionWithFragment(
            Some(fragment),
            false,
            smart,
            match_style,
            InputType::kInsertFromPaste,
            PasswordEchoBehavior::kDoNotEcho,
            transfer,
        );
        Ok(())
    }
    fn ReplacePasteText(&self, text: &[u16], smart: bool) -> Result<(), Self::Error> {
        self.ReplaceSelectionWithText(
            text,
            false,
            smart,
            InputType::kInsertFromPaste,
            PasswordEchoBehavior::kDoNotEcho,
        );
        Ok(())
    }
}

// Source editor_paragraph_separator.h; editor.h:233-238,263-264.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorParagraphSeparator {
    kIsDiv,
    kIsP,
}
pub fn DefaultParagraphElementTag(separator: EditorParagraphSeparator) -> &'static str {
    match separator {
        EditorParagraphSeparator::kIsDiv => "div",
        EditorParagraphSeparator::kIsP => "p",
    }
}
/// Editor's native notification leaves, source editor.cc:420-428.
pub trait EditorChangedContentsContext<N> {
    fn HandleAXEditableTextContentChanged(&self, node: Option<&N>);
    fn SpellCheckerRespondToChangedContents(&self);
    fn FrameClientDidChangeContents(&self);
}
/// public/web/web_local_frame_client.h:158-161, passed through unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyncCondition {
    kNotForced,
    kForced,
}
/// Original nullable LocalFrameClient, not a selection-paint commit hook.
pub trait EditorFrameClient {
    fn DidChangeSelection(&self, is_selection_empty: bool, force_sync: SyncCondition);
}
pub trait EditorSelectionSyncContext {
    fn GetActiveEditContext(
        &self,
    ) -> Option<Rc<dyn super::ime::active_edit_context::ActiveEditContext>>;
    fn GetFrameClient(&self) -> Option<Rc<dyn EditorFrameClient>>;
}
pub trait EditorChangedSelectionContext: EditorSelectionSyncContext {
    fn SpellCheckerRespondToChangedSelection(&self);
}
/// Editor::AddToKillRing cc:705-713. PlainText is supplied by the canonical
/// TextIterator over the original ephemeral range, after the source gate.
pub trait EditorKillRingContext<R> {
    fn NeedsLayoutTreeUpdate(&self) -> bool;
    fn PlainText(&self, range: &R) -> foundation::String;
}
/// Source editor.cc:430-441. The existing optional cache is acquired before
/// inspecting the original selection; absence never creates an AX owner.
pub trait EditorAXTextFieldChangeContext<S> {
    type Cache;
    fn ExistingAXObjectCache(&self) -> Option<Self::Cache>;
    fn SelectionEndpointsValidForCurrentDocument(&self, selection: &S) -> bool;
    fn HandleDeletionOrInsertionInTextField(
        &self,
        cache: &Self::Cache,
        selection: &S,
        is_deletion: bool,
    );
}
pub struct Editor<'scope, S: Clone, C: EditorCommands<S>> {
    pub selection: Rc<dyn EditorSelection<S> + 'scope>,
    pub commands: Rc<C>,
    state: Rc<EditorState<S, C::TypingStyle>>,
}

/// Persistent Editor ownership. A source operation may borrow its document and
/// resident geometry through a call-scoped command context while retaining this
/// exact history, typing style, mark and last-command owner across operations.
pub struct EditorState<S: Clone, T> {
    prevent_reveal_selection: Cell<u32>,
    running_exec_command: Rc<Cell<bool>>,
    should_start_new_kill_ring_sequence: Cell<bool>,
    kill_ring: RefCell<Option<Rc<dyn super::kill_ring::KillRingPlatform>>>,
    typing_style: RefCell<Option<Rc<RefCell<T>>>>,
    // editor.h:114-115,260; SetShouldStyleWithCss mutates this same slot.
    should_style_with_css: Cell<bool>,
    undo_stack: super::commands::undo_stack::UndoStack<S>,
    last_edit_command:
        RefCell<Option<Rc<super::commands::composite_edit_command::CompositeEditCommand<S>>>>,
    default_paragraph_separator: Cell<EditorParagraphSeparator>,
    // Source editor.h:262,266: the original visible-selection mark snapshot.
    mark: RefCell<Option<S>>,
    mark_is_directional: Cell<bool>,
}
impl<S: Clone, T> Default for EditorState<S, T> {
    fn default() -> Self {
        Self {
            prevent_reveal_selection: Cell::new(0),
            running_exec_command: Rc::new(Cell::new(false)),
            should_start_new_kill_ring_sequence: Cell::new(false),
            kill_ring: RefCell::new(None),
            typing_style: RefCell::new(None),
            should_style_with_css: Cell::new(false),
            undo_stack: Default::default(),
            last_edit_command: RefCell::new(None),
            default_paragraph_separator: Cell::new(EditorParagraphSeparator::kIsDiv),
            mark: RefCell::new(None),
            mark_is_directional: Cell::new(false),
        }
    }
}
impl<S: Clone, T> EditorState<S, T> {
    pub fn TryExecCommandScope(&self) -> Option<EditorExecCommandScope> {
        if self.running_exec_command.get() {
            return None;
        }
        let previous = self.running_exec_command.replace(true);
        Some(EditorExecCommandScope {
            state: self.running_exec_command.clone(),
            previous,
        })
    }
    pub fn SetMarkSelection(&self, selection: Option<S>, directional: bool) {
        *self.mark.borrow_mut() = selection;
        self.mark_is_directional.set(directional);
    }
    pub fn Mark(&self) -> Option<S> {
        self.mark.borrow().clone()
    }
    pub fn MarkIsDirectional(&self) -> bool {
        self.mark_is_directional.get()
    }
    pub fn RespondToChangedContents<N, H: EditorChangedContentsContext<N>>(
        &self,
        context: &H,
        anchor: Option<&N>,
    ) {
        context.HandleAXEditableTextContentChanged(anchor);
        context.SpellCheckerRespondToChangedContents();
        context.FrameClientDidChangeContents();
    }
    pub fn PreventRevealSelection(&self) -> bool {
        self.prevent_reveal_selection.get() != 0
    }
    pub fn IncreasePreventRevealSelection(&self) {
        self.prevent_reveal_selection
            .set(self.prevent_reveal_selection.get() + 1);
    }
    pub fn DecreasePreventRevealSelection(&self) {
        self.prevent_reveal_selection.set(
            self.prevent_reveal_selection
                .get()
                .checked_sub(1)
                .expect("balanced reveal selection scope"),
        );
    }

    /// Installed by the embedding, retained across Editor::Clear just like
    /// editor.h's KillRing owner. The host service itself owns global state.
    pub fn SetKillRing(&self, ring: Rc<dyn super::kill_ring::KillRingPlatform>) {
        *self.kill_ring.borrow_mut() = Some(ring);
    }
    pub fn GetKillRing(
        &self,
    ) -> Result<
        Rc<dyn super::kill_ring::KillRingPlatform>,
        super::kill_ring::RequiredKillRingPlatform,
    > {
        self.kill_ring
            .borrow()
            .clone()
            .ok_or(super::kill_ring::RequiredKillRingPlatform::HostNotInstalled)
    }
    pub fn ExecuteYank(
        &self,
        context: &dyn super::kill_ring::EditorYankContext,
        select_inserted: bool,
    ) -> Result<bool, super::kill_ring::RequiredKillRingPlatform> {
        super::kill_ring::ExecuteYank(self.GetKillRing()?.as_ref(), context, select_inserted)
    }
    pub fn AddToKillRing<R>(
        &self,
        range: &R,
        context: &dyn EditorKillRingContext<R>,
    ) -> Result<(), super::kill_ring::RequiredKillRingPlatform> {
        let ring = self.GetKillRing()?;
        if self.should_start_new_kill_ring_sequence.get() {
            ring.StartNewSequence()?;
        }
        debug_assert!(!context.NeedsLayoutTreeUpdate());
        let text = context.PlainText(range);
        ring.Append(&text)?;
        self.should_start_new_kill_ring_sequence.set(false);
        Ok(())
    }
    pub fn SetStartNewKillRingSequence(&self, flag: bool) {
        self.should_start_new_kill_ring_sequence.set(flag);
    }
    pub fn DefaultParagraphSeparator(&self) -> EditorParagraphSeparator {
        self.default_paragraph_separator.get()
    }
    pub fn SetDefaultParagraphSeparator(&self, value: EditorParagraphSeparator) {
        self.default_paragraph_separator.set(value)
    }
    pub fn TypingStyle(&self) -> Option<Rc<RefCell<T>>> {
        self.typing_style.borrow().clone()
    }
    pub fn SetTypingStyle(&self, value: Option<Rc<RefCell<T>>>) {
        *self.typing_style.borrow_mut() = value;
    }
    pub fn ClearTypingStyle(&self) {
        self.typing_style.borrow_mut().take();
    }
    pub fn ShouldStyleWithCss(&self) -> bool {
        self.should_style_with_css.get()
    }
    pub fn SetShouldStyleWithCss(&self, value: bool) {
        self.should_style_with_css.set(value);
    }
    pub fn LastEditCommand(
        &self,
    ) -> Option<Rc<super::commands::composite_edit_command::CompositeEditCommand<S>>> {
        self.last_edit_command.borrow().clone()
    }
    pub fn SetLastEditCommand(
        &self,
        command: Option<Rc<super::commands::composite_edit_command::CompositeEditCommand<S>>>,
    ) {
        *self.last_edit_command.borrow_mut() = command;
    }
    pub fn GetUndoStack(&self) -> &super::commands::undo_stack::UndoStack<S> {
        &self.undo_stack
    }
    pub fn Clear(&self) {
        self.should_style_with_css.set(false);
        self.default_paragraph_separator
            .set(EditorParagraphSeparator::kIsDiv);
        self.last_edit_command.borrow_mut().take();
        self.undo_stack.Clear();
    }
    /// composite_edit_command.cc:2501-2535, shared by full Editor and the
    /// call-scoped native editing ports over this exact retained state.
    pub fn RegisterAppliedEditCommand<
        H: super::commands::undo_stack::UndoStackRegistrationContext<S>,
    >(
        &self,
        context: &H,
        command: &Rc<super::commands::composite_edit_command::CompositeEditCommand<S>>,
        input_type: InputType,
        update_dom: bool,
    ) {
        let last = self.LastEditCommand();
        if last.as_ref().is_some_and(|last| Rc::ptr_eq(last, command)) {
            debug_assert!(command.IsTypingCommand());
            return;
        }
        if let Some(last) = last.filter(|last| {
            last.IsDragAndDropCommand()
                && matches!(
                    input_type,
                    InputType::kDeleteByDrag | InputType::kInsertFromDrop
                )
        }) {
            if last.GetUndoStep().is_none() {
                self.undo_stack
                    .RegisterUndoStep(context, last.EnsureUndoStep());
            }
            let target = last.EnsureUndoStep();
            let step = command.EnsureUndoStep();
            target.SetEndingSelection(step.EndingSelection());
            if update_dom {
                target.SetEndingDomSelection(step.EndingDomSelection());
            }
            target.SetSelectionIsDirectional(step.SelectionIsDirectional());
            self.undo_stack.DidSetEndingSelection(context, &target);
            last.AppendCommandToUndoStep(command.clone());
        } else {
            self.SetLastEditCommand(Some(command.clone()));
            self.undo_stack
                .RegisterUndoStep(context, command.EnsureUndoStep());
        }
    }
}
impl<'scope, S: Clone, C: EditorCommands<S>> std::ops::Deref for Editor<'scope, S, C> {
    type Target = EditorState<S, C::TypingStyle>;
    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

/// Source document_exec_command.cc:126-135, AutoReset<bool>. The editing
/// transaction owner retains the flag here; no Browser/DOM mirror is needed.
pub struct EditorExecCommandScope {
    state: Rc<Cell<bool>>,
    previous: bool,
}
impl Drop for EditorExecCommandScope {
    fn drop(&mut self) {
        self.state.set(self.previous);
    }
}

impl<'scope, S: Clone, C: EditorCommands<S>> Editor<'scope, S, C> {
    // Source editor.cc:933-937; editor.h:257,280-282.
    pub fn RespondToChangedSelection<H: EditorChangedSelectionContext>(&self, context: &H) {
        context.SpellCheckerRespondToChangedSelection();
        self.SyncSelection(context, SyncCondition::kNotForced);
        self.SetStartNewKillRingSequence(true);
    }
    // Source editor.cc:939-953. Use raw FrameSelection, without a lifecycle
    // barrier or a VisibleSelection conversion. Active EditContext owns sync.
    pub fn SyncSelection(
        &self,
        context: &dyn EditorSelectionSyncContext,
        force_sync: SyncCondition,
    ) {
        if context.GetActiveEditContext().is_some() {
            return;
        }
        if let Some(client) = context.GetFrameClient() {
            client.DidChangeSelection(
                !self.selection.IsRangeForEditing(&self.selection.Current()),
                force_sync,
            );
        }
    }
    pub fn CanCopy(&self, context: &dyn EditorCopyContext) -> bool {
        if context.ImageElementFromImageDocument() {
            return true;
        }
        if !context.IsSelectionAvailable() {
            return false;
        }
        context.UpdateStyleAndLayoutForEditing();
        let selection = context.VisibleSelectionInFlatTree();
        selection.is_range && !selection.starts_in_unrevealed_password
    }
    pub fn CanCut(&self, context: &dyn EditorCopyContext) -> bool {
        self.CanCopy(context) && self.CanDelete()
    }
    pub fn CanPaste(&self) -> bool {
        self.CanEdit()
    }
    pub fn CanDelete(&self) -> bool {
        let visible = self
            .selection
            .CreateVisibleSelection(&self.selection.Current());
        self.selection.IsRangeForEditing(&visible)
            && self.selection.HasRootEditableElementForSelection(&visible)
    }
    // Source editor.cc:479-484. Reset the original command state together.
    pub fn Clear(&self) {
        self.state.Clear();
    }
    pub fn ShouldStyleWithCss(&self) -> bool {
        self.state.ShouldStyleWithCss()
    }
    pub fn SetShouldStyleWithCss(&self, value: bool) {
        self.state.SetShouldStyleWithCss(value);
    }
    pub fn new(selection: Rc<dyn EditorSelection<S> + 'scope>, commands: Rc<C>) -> Self {
        Self::WithState(selection, commands, Rc::new(EditorState::default()))
    }
    pub fn WithState(
        selection: Rc<dyn EditorSelection<S> + 'scope>,
        commands: Rc<C>,
        state: Rc<EditorState<S, C::TypingStyle>>,
    ) -> Self {
        Self {
            selection,
            commands,
            state,
        }
    }
    // Source editor.cc:963-966; editor.h:284-290.
    pub fn SetMark(&self, directional: bool) {
        self.state.SetMarkSelection(
            Some(
                self.selection
                    .CreateVisibleSelection(&self.selection.Current()),
            ),
            directional,
        );
    }
    pub fn Mark(&self) -> Option<S> {
        self.state.Mark()
    }
    pub fn MarkIsDirectional(&self) -> bool {
        self.state.MarkIsDirectional()
    }
    pub fn NotifyAccessibilityOfDeletionOrInsertionInTextField<
        H: EditorAXTextFieldChangeContext<S>,
    >(
        &self,
        context: &H,
        selection: &S,
        is_deletion: bool,
    ) {
        let Some(cache) = context.ExistingAXObjectCache() else {
            return;
        };
        if !context.SelectionEndpointsValidForCurrentDocument(selection) {
            return;
        }
        context.HandleDeletionOrInsertionInTextField(&cache, selection, is_deletion);
    }
    pub fn DefaultParagraphSeparator(&self) -> EditorParagraphSeparator {
        self.state.DefaultParagraphSeparator()
    }
    pub fn SetDefaultParagraphSeparator(&self, separator: EditorParagraphSeparator) {
        self.state.SetDefaultParagraphSeparator(separator);
    }
    pub fn RespondToChangedContents<N, H: EditorChangedContentsContext<N>>(
        &self,
        context: &H,
        anchor: Option<&N>,
    ) {
        self.state.RespondToChangedContents(context, anchor);
    }
    // Source editor.h:280-284; composite_edit_command.cc:2501-2535.
    pub fn LastEditCommand(
        &self,
    ) -> Option<Rc<super::commands::composite_edit_command::CompositeEditCommand<S>>> {
        self.state.LastEditCommand()
    }
    pub fn SetLastEditCommand(
        &self,
        command: Option<Rc<super::commands::composite_edit_command::CompositeEditCommand<S>>>,
    ) {
        self.state.SetLastEditCommand(command);
    }
    // Source editor.cc:977-981. Removal clears only the command rooted at
    // that exact element; UndoStack pruning is a separate preceding operation.
    pub fn ElementRemoved<H: super::commands::undo_step::UndoStepContext<S>>(
        &self,
        context: &H,
        element: &H::Node,
    ) {
        if self.LastEditCommand().is_some_and(|command| {
            context
                .RootEditableElement(&command.EndingSelection())
                .as_ref()
                == Some(element)
        }) {
            self.SetLastEditCommand(None);
        }
    }
    pub fn RegisterAppliedEditCommand<H: super::commands::undo_stack::UndoStackContext<S>>(
        &self,
        context: &H,
        command: &Rc<super::commands::composite_edit_command::CompositeEditCommand<S>>,
        input_type: InputType,
        update_dom: bool,
    ) {
        self.state
            .RegisterAppliedEditCommand(context, command, input_type, update_dom);
    }
    // Source editor.cc:643-657. No independent history/capability cache.
    pub fn GetUndoStack(&self) -> &super::commands::undo_stack::UndoStack<S> {
        &self.undo_stack
    }
    pub fn CanUndo(&self) -> bool {
        self.undo_stack.CanUndo()
    }
    pub fn CanRedo(&self) -> bool {
        self.undo_stack.CanRedo()
    }
    pub fn Undo<H: super::commands::undo_step::UndoStepContext<S>>(&self, host: &H) {
        self.undo_stack.Undo(host);
    }
    pub fn Redo<H: super::commands::undo_step::UndoStepContext<S>>(&self, host: &H) {
        self.undo_stack.Redo(host);
    }
    // Source editor.h:292-302. This is the single Editor typing-style owner.
    // Its guards are used only during pure source style queries, never script.
    pub fn TypingStyle(&self) -> Option<Rc<RefCell<C::TypingStyle>>> {
        self.state.TypingStyle()
    }
    pub fn SetTypingStyle(&self, style: Option<Rc<RefCell<C::TypingStyle>>>) {
        self.state.SetTypingStyle(style);
    }
    pub fn ClearTypingStyle(&self) {
        self.state.ClearTypingStyle();
    }
    pub fn IsRunningExecCommand(&self) -> bool {
        self.running_exec_command.get()
    }
    pub fn TryExecCommandScope(&self) -> Option<EditorExecCommandScope> {
        self.state.TryExecCommandScope()
    }
    pub fn RunExecCommand(&self, command: impl FnOnce() -> bool) -> bool {
        let Some(_scope) = self.TryExecCommandScope() else {
            return false;
        };
        command()
    }
    // Source: editor.cc:114-134.
    pub fn SelectionForCommand(&self, event: Option<&EventRef>) -> S {
        let selection = self.selection.Current();
        let Some(event) = event else {
            return selection;
        };
        if !self.selection.EventTargetIsTextControl(event) {
            return selection;
        }
        if !self.selection.IsNone(&selection)
            && self
                .selection
                .CurrentSelectionBelongsToEventTarget(event, &selection)
        {
            return selection;
        }
        let saved = self.selection.EventTargetSavedSelection(event);
        if self.selection.IsNone(&saved) {
            selection
        } else {
            saved
        }
    }
    // Source: editor.cc:145-201. The prepared token corresponds to the source
    // layout synchronization at 158 and is never acquired inside this engine.
    pub fn HandleTextEvent(
        &self,
        event: &TextEventData<C::Fragment, C::DataTransfer>,
        _prepared: &PreparedEditorGeometry,
    ) -> bool {
        if event.is_drop || event.is_incremental_insertion {
            return false;
        }
        if event.is_paste {
            let Ok(()) = HandlePasteTextEvent(self, event);
            return true;
        }
        if event.data == [10] {
            return if event.is_line_break {
                self.InsertLineBreak()
            } else {
                self.InsertParagraphSeparator()
            };
        }
        // Source editor.cc:181-197 uses the physical echo policy by default;
        // only Android consults its explicit physical-keyboard input fact.
        #[cfg(not(target_os = "android"))]
        let echo = PasswordEchoBehavior::kEchoIfPasswordEchoPhysicalEnabled;
        #[cfg(target_os = "android")]
        let echo = if event.confirmed_physical_keyboard {
            PasswordEchoBehavior::kEchoIfPasswordEchoPhysicalEnabled
        } else {
            PasswordEchoBehavior::kEchoIfPasswordEchoTouchEnabled
        };
        self.InsertTextWithoutSendingTextEvent(
            &event.data,
            false,
            Some(&event.event),
            event.is_composition,
            InputType::kInsertText,
            echo,
            None,
        )
    }
    // Source: editor.cc:203-214.
    pub fn CanEdit(&self) -> bool {
        self.selection.IsContentEditable(&self.selection.Current())
    }
    pub fn CanEditRichly(&self) -> bool {
        self.selection.IsRichlyEditable(&self.selection.Current())
    }
    // Source: editor.cc:256-277.
    pub fn DeleteSelectionWithSmartDelete(
        &self,
        smart: bool,
        input_type: InputType,
        reference_move_position: &C::Position,
    ) where
        C: EditorDeletionCommands<S>,
    {
        if self.selection.IsNone(&self.selection.Current()) {
            return;
        }
        self.commands.DeleteSelection(DeleteSelectionCommandInput {
            smart_delete: smart,
            merge_blocks_after_delete: true,
            expand_for_special_elements: true,
            sanitize_markup: true,
            input_type,
            reference_move_position,
        });
    }
    // Source: editor.cc:279-308.
    pub fn ReplaceSelectionWithFragment(
        &self,
        fragment: Option<&C::Fragment>,
        select: bool,
        smart: bool,
        match_style: bool,
        input_type: InputType,
        echo: PasswordEchoBehavior,
        transfer: Option<&C::DataTransfer>,
    ) {
        let selection = self.selection.Current();
        let Some(fragment) = fragment else {
            return;
        };
        if self.selection.IsNone(&selection) || !self.selection.IsContentEditable(&selection) {
            return;
        }
        self.commands.ReplaceSelection(
            fragment,
            ReplaceSelectionCommandOptions {
                prevent_nesting: true,
                sanitize_fragment: true,
                select_replacement: select,
                smart_replace: smart,
                match_style,
            },
            echo,
            input_type,
            transfer,
        );
        self.selection.RevealSelectionAfterEditingOperation();
    }
    // Source: editor.cc:310-319.
    pub fn ReplaceSelectionWithText(
        &self,
        text: &[u16],
        select: bool,
        smart: bool,
        input_type: InputType,
        echo: PasswordEchoBehavior,
    ) {
        let fragment = self.commands.CreateFragmentFromText(text);
        self.ReplaceSelectionWithFragment(
            fragment.as_ref(),
            select,
            smart,
            true,
            input_type,
            echo,
            None,
        );
    }
    // Source: editor.cc:486-489.
    pub fn InsertText(&self, text: &[u16], triggering_keyboard: Option<&EventRef>) -> bool {
        self.commands
            .HandleTextInputEvent(text, triggering_keyboard)
    }
    // Source: editor.cc:491-527.
    pub fn InsertTextWithoutSendingTextEvent(
        &self,
        text: &[u16],
        select_inserted: bool,
        triggering_event: Option<&EventRef>,
        composition: bool,
        input_type: InputType,
        echo: PasswordEchoBehavior,
        transfer: Option<&C::DataTransfer>,
    ) -> bool {
        let selection = self
            .selection
            .CreateVisibleSelection(&self.SelectionForCommand(triggering_event));
        if !self.selection.IsContentEditable(&selection) {
            return false;
        }
        if !self.commands.InsertText(
            text,
            &selection,
            select_inserted,
            echo,
            composition,
            input_type,
            transfer,
        ) {
            return false;
        }
        self.selection.RevealFocusedOrMainFrameSelection(&selection);
        true
    }
    // Source: editor.cc:529-559.
    pub fn InsertLineBreak(&self) -> bool {
        if !self.CanEdit() {
            return false;
        }
        if !self.commands.InsertLineBreak() {
            return false;
        }
        self.selection.RevealSelectionAfterEditingOperation();
        true
    }
    pub fn InsertParagraphSeparator(&self) -> bool {
        if !self.CanEdit() {
            return false;
        }
        if !self.CanEditRichly() {
            return self.InsertLineBreak();
        }
        if !self.commands.InsertParagraphSeparator() {
            return false;
        }
        self.selection.RevealSelectionAfterEditingOperation();
        true
    }
}
