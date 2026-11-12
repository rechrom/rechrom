// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: typing_command.cc:153-174,199-246,329-350,402-537,662-834,
// 1266-1279; typing_command.h:50-60,152-154.
use super::{
    composite_edit_command::CompositeEditCommand, edit_command::PasswordEchoBehavior,
    undo_step::UndoStep,
};
use interaction_dispatch::core::events::input_event::InputType;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextCompositionType {
    kTextCompositionNone,
    kTextCompositionUpdate,
    kTextCompositionConfirm,
    kTextCompositionCancel,
}
pub const kSelectInsertedText: u32 = 1 << 0;
pub const kKillRing: u32 = 1 << 1;
pub const kSmartDelete: u32 = 1 << 2;
pub trait TypingSelection: Clone + PartialEq {
    fn IsCaret(&self) -> bool;
}
pub trait TypingCommandHost<S: TypingSelection> {
    fn LastEditCommandIs(&self, command: &Rc<CompositeEditCommand<S>>) -> bool;
    fn IsValid(&self, selection: &S) -> bool;
    fn CompositeBirthSelection(&self) -> (S, S);
    fn EditingUseDomPositionApiEnabled(&self) -> bool;
    fn BeforeTextInserted(&self, text: &[u16], selection: &S) -> Option<Vec<u16>>;
    fn DispatchTextInput(&self, text: &[u16]) -> bool;
    fn PrepareForInsertion(&self);
    fn InsertTextRun(
        &self,
        text: &[u16],
        selection: &S,
        select: bool,
        incremental: bool,
        command: &Rc<CompositeEditCommand<S>>,
        echo: PasswordEchoBehavior,
        rebalance_all: bool,
    ) -> Option<S>;
    fn InsertParagraphSeparator(
        &self,
        selection: &S,
        command: &Rc<CompositeEditCommand<S>>,
    ) -> Option<S>;
    fn InsertLineBreak(&self, selection: &S, command: &Rc<CompositeEditCommand<S>>) -> Option<S>;
    fn InsertParagraphSeparatorInQuotedContent(
        &self,
        selection: &S,
        command: &Rc<CompositeEditCommand<S>>,
    ) -> Option<S>;
    fn SelectionStartsInTableStructure(
        &self,
        selection: &S,
        command: &Rc<CompositeEditCommand<S>>,
    ) -> bool;
    fn SelectionStart(&self, selection: &S) -> Option<usize>;
    fn SelectInsertedRange(&self, start: usize, end: &S) -> S;
    fn DeleteSelection(
        &self,
        selection: &S,
        command: &Rc<CompositeEditCommand<S>>,
        smart: bool,
    ) -> Option<S>;
    fn AppliedEditing(
        &self,
        selection: &S,
        command: &Rc<CompositeEditCommand<S>>,
        input_type: InputType,
        text: Option<&[u16]>,
        composing: bool,
        preserves_typing_style: bool,
    );
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum CommandType {
    InsertText,
    InsertParagraph,
    InsertLineBreak,
    InsertQuotedParagraph,
    DeleteSelection,
    DeleteKey,
    ForwardDeleteKey,
}
/// Each source command has independent open/composition/type state. The facade
/// retains only the last command pointer, matching Editor::LastEditCommand;
/// synchronous reentry cannot accidentally close or reopen a different command.
pub(super) struct TypingCommandState<S: TypingSelection> {
    pub(super) composite: Rc<CompositeEditCommand<S>>,
    pub(super) open: Cell<bool>,
    pub(super) composition: Cell<TextCompositionType>,
    pub(super) kind: Cell<CommandType>,
    pub(super) input_type: Cell<InputType>,
    pub(super) incremental: Cell<bool>,
    pub(super) text: RefCell<Rc<[u16]>>,
    pub(super) granularity: Cell<super::super::text_granularity::TextGranularity>,
    pub(super) smart_delete: Cell<bool>,
    pub(super) opened_by_backward_delete: Cell<bool>,
}
impl<S: TypingSelection> TypingCommandState<S> {
    pub(super) fn new(composite: Rc<CompositeEditCommand<S>>, kind: CommandType) -> Rc<Self> {
        composite.SetCommandKind(super::composite_edit_command::CompositeCommandKind::Typing);
        Rc::new(Self {
            composite,
            open: Cell::new(true),
            composition: Cell::new(TextCompositionType::kTextCompositionNone),
            kind: Cell::new(kind),
            input_type: Cell::new(InputType::kNone),
            incremental: Cell::new(false),
            text: RefCell::new(Rc::from([])),
            granularity: Cell::new(super::super::text_granularity::TextGranularity::kCharacter),
            smart_delete: Cell::new(false),
            opened_by_backward_delete: Cell::new(false),
        })
    }
    fn Composing(&self) -> bool {
        self.composition.get() != TextCompositionType::kTextCompositionNone
    }
    fn InputType(&self) -> InputType {
        if self.Composing() {
            InputType::kInsertCompositionText
        } else if self.input_type.get() != InputType::kNone {
            self.input_type.get()
        } else {
            match self.kind.get() {
                CommandType::InsertText => InputType::kInsertText,
                CommandType::InsertParagraph | CommandType::InsertQuotedParagraph => {
                    InputType::kInsertParagraph
                }
                CommandType::InsertLineBreak => InputType::kInsertLineBreak,
                CommandType::DeleteSelection => InputType::kDeleteContentBackward,
                CommandType::DeleteKey => {
                    DeletionInputTypeFromTextGranularity(false, self.granularity.get())
                }
                CommandType::ForwardDeleteKey => {
                    DeletionInputTypeFromTextGranularity(true, self.granularity.get())
                }
            }
        }
    }
}
pub struct TypingCommand<S: TypingSelection> {
    last: RefCell<Option<Rc<TypingCommandState<S>>>>,
}
impl<S: TypingSelection> Default for TypingCommand<S> {
    fn default() -> Self {
        Self {
            last: RefCell::new(None),
        }
    }
}
impl<S: TypingSelection> TypingCommand<S> {
    pub(super) fn LastOpen(
        &self,
        host: &dyn TypingCommandHost<S>,
    ) -> Option<Rc<TypingCommandState<S>>> {
        self.last
            .borrow()
            .clone()
            .filter(|command| command.open.get() && host.LastEditCommandIs(&command.composite))
    }
    pub fn CloseTyping(&self) {
        if let Some(command) = self.last.borrow().as_ref() {
            command.open.set(false);
        }
    }
    pub fn CloseTypingForLastCommand(&self, last: &Rc<CompositeEditCommand<S>>) {
        if let Some(command) = self
            .last
            .borrow()
            .as_ref()
            .filter(|command| Rc::ptr_eq(&command.composite, last))
        {
            command.open.set(false);
        }
    }
    pub fn EndingSelectionForLastCommand(&self, last: &Rc<CompositeEditCommand<S>>) -> Option<S> {
        self.last
            .borrow()
            .as_ref()
            .filter(|command| Rc::ptr_eq(&command.composite, last))
            .map(|command| command.composite.EndingSelection())
    }
    pub fn EndingSelection(&self) -> Option<S> {
        self.last
            .borrow()
            .as_ref()
            .map(|command| command.composite.EndingSelection())
    }
    pub fn LastUndoStep(&self) -> Option<Rc<UndoStep<S>>> {
        self.last
            .borrow()
            .as_ref()
            .map(|command| command.composite.EnsureUndoStep())
    }
    pub fn UpdateSelectionIfDifferentFromCurrentSelection(
        &self,
        host: &dyn TypingCommandHost<S>,
        selection: &S,
        update_dom: bool,
    ) {
        if let Some(command) = self.LastOpen(host) {
            if &command.composite.EndingSelection() != selection {
                command.composite.SetStartingSelection(selection.clone());
                command.composite.SetEndingSelection(selection.clone());
                if update_dom {
                    command.composite.SetStartingDomSelection(selection.clone());
                    command.composite.SetEndingDomSelection(selection.clone());
                }
            }
        }
    }
    pub(super) fn TypingAddedToOpenCommand(
        &self,
        host: &dyn TypingCommandHost<S>,
        command: &Rc<TypingCommandState<S>>,
        ending: &S,
        kind: CommandType,
        data: Option<&[u16]>,
    ) {
        command.kind.set(kind);
        command.composite.SetEndingSelection(ending.clone());
        if host.EditingUseDomPositionApiEnabled() {
            command.composite.SetEndingDomSelection(ending.clone());
        }
        let full_text = if command.incremental.get() {
            Some(command.text.borrow().clone())
        } else {
            None
        };
        let data = full_text.as_ref().map(|text| text.as_ref()).or(data);
        host.AppliedEditing(
            ending,
            &command.composite,
            command.InputType(),
            data,
            command.Composing(),
            !matches!(
                kind,
                CommandType::InsertText | CommandType::InsertQuotedParagraph
            ),
        );
        // Source AppliedEditing installs the command after selection/focus effects.
        *self.last.borrow_mut() = Some(command.clone());
    }
    pub fn InsertText(
        &self,
        host: &dyn TypingCommandHost<S>,
        text: &[u16],
        selection: &S,
        select: bool,
        composition: TextCompositionType,
        incremental: bool,
        input_type: InputType,
        echo: PasswordEchoBehavior,
    ) -> bool {
        if !host.IsValid(selection) {
            return false;
        }
        let text = if composition != TextCompositionType::kTextCompositionUpdate {
            let Some(text) = host.BeforeTextInserted(text, selection) else {
                return false;
            };
            if !host.IsValid(selection) {
                return false;
            }
            text
        } else {
            text.to_vec()
        };
        if composition == TextCompositionType::kTextCompositionConfirm
            && (!host.DispatchTextInput(&text) || !host.IsValid(selection))
        {
            return false;
        }
        if selection.IsCaret() && text.is_empty() {
            return true;
        }
        host.PrepareForInsertion();
        if !host.IsValid(selection) {
            return false;
        }
        let Some(selection_start) = host.SelectionStart(selection) else {
            return false;
        };
        let update_dom = host.EditingUseDomPositionApiEnabled();
        self.UpdateSelectionIfDifferentFromCurrentSelection(host, selection, update_dom);
        let command = self.LastOpen(host).unwrap_or_else(|| {
            let (visible, raw) = host.CompositeBirthSelection();
            let changed = &raw != selection;
            let composite = CompositeEditCommand::new(visible, raw);
            if changed {
                composite.SetStartingSelection(selection.clone());
                composite.SetEndingSelection(selection.clone());
                if update_dom {
                    composite.SetStartingDomSelection(selection.clone());
                    composite.SetEndingDomSelection(selection.clone());
                }
            }
            TypingCommandState::new(composite, CommandType::InsertText)
        });
        command.composition.set(composition);
        command.input_type.set(input_type);
        command.incremental.set(incremental);
        let text = NormalizeTextForInsertion(text);
        *command.text.borrow_mut() = Rc::from(text.as_slice());
        let mut ending = selection.clone();
        let mut offset = 0;
        // Each source run publishes through TypingAddedToOpenCommand while the
        // caller's EventQueueScope retains input-event delivery until scope exit.
        if text.is_empty() {
            let Some(next) = host.InsertTextRun(
                &[],
                &ending,
                false,
                incremental,
                &command.composite,
                echo,
                command.Composing(),
            ) else {
                return false;
            };
            ending = next;
            self.TypingAddedToOpenCommand(
                host,
                &command,
                &ending,
                CommandType::InsertText,
                Some(&[]),
            );
            return true;
        }
        for (newline, _) in text.iter().enumerate().filter(|(_, unit)| **unit == 10) {
            if newline > offset {
                let run = &text[offset..newline];
                let Some(next) = host.InsertTextRun(
                    run,
                    &ending,
                    false,
                    incremental,
                    &command.composite,
                    echo,
                    command.Composing(),
                ) else {
                    return false;
                };
                ending = next;
                self.TypingAddedToOpenCommand(
                    host,
                    &command,
                    &ending,
                    CommandType::InsertText,
                    Some(run),
                );
            }
            match CanAppendNewLineFeedToSelection(host, &ending) {
                None => return false,
                Some(false) => {}
                Some(true) => {
                    let Some(next) = host.InsertParagraphSeparator(&ending, &command.composite)
                    else {
                        return false;
                    };
                    ending = next;
                    self.TypingAddedToOpenCommand(
                        host,
                        &command,
                        &ending,
                        CommandType::InsertParagraph,
                        None,
                    );
                }
            }
            offset = newline + 1;
        }
        if offset < text.len() {
            let run = &text[offset..];
            let Some(next) = host.InsertTextRun(
                run,
                &ending,
                false,
                incremental,
                &command.composite,
                echo,
                command.Composing(),
            ) else {
                return false;
            };
            ending = next;
            self.TypingAddedToOpenCommand(
                host,
                &command,
                &ending,
                CommandType::InsertText,
                Some(run),
            );
        }
        if select {
            let selected = host.SelectInsertedRange(selection_start, &ending);
            command.composite.SetEndingSelection(selected.clone());
            if update_dom {
                command.composite.SetEndingDomSelection(selected);
            }
        }
        true
    }
    pub fn InsertLineBreak(&self, host: &dyn TypingCommandHost<S>, selection: &S) -> bool {
        self.InsertNewline(host, selection, CommandType::InsertLineBreak)
    }
    pub fn InsertParagraphSeparator(&self, host: &dyn TypingCommandHost<S>, selection: &S) -> bool {
        self.InsertNewline(host, selection, CommandType::InsertParagraph)
    }
    pub fn InsertParagraphSeparatorInQuotedContent(
        &self,
        host: &dyn TypingCommandHost<S>,
        selection: &S,
    ) -> bool {
        self.InsertNewline(host, selection, CommandType::InsertQuotedParagraph)
    }
    fn InsertNewline(
        &self,
        host: &dyn TypingCommandHost<S>,
        selection: &S,
        mut kind: CommandType,
    ) -> bool {
        if !host.IsValid(selection) {
            return false;
        }
        host.PrepareForInsertion();
        let command = self.LastOpen(host).unwrap_or_else(|| {
            let (visible, raw) = host.CompositeBirthSelection();
            TypingCommandState::new(CompositeEditCommand::new(visible, raw), kind)
        });
        command.input_type.set(InputType::kNone);
        let ending = command.composite.EndingSelection();
        if kind == CommandType::InsertQuotedParagraph
            && host.SelectionStartsInTableStructure(&ending, &command.composite)
        {
            kind = CommandType::InsertParagraph;
        }
        if kind != CommandType::InsertQuotedParagraph {
            match CanAppendNewLineFeedToSelection(host, &ending) {
                None => return false,
                Some(false) => return true,
                Some(true) => {}
            }
        }
        let result = match kind {
            CommandType::InsertLineBreak => host.InsertLineBreak(&ending, &command.composite),
            CommandType::InsertParagraph => {
                host.InsertParagraphSeparator(&ending, &command.composite)
            }
            CommandType::InsertQuotedParagraph => {
                host.InsertParagraphSeparatorInQuotedContent(&ending, &command.composite)
            }
            _ => unreachable!("source newline command kind"),
        };
        let Some(ending) = result else {
            return false;
        };
        self.TypingAddedToOpenCommand(host, &command, &ending, kind, None);
        true
    }
    // Source typing_command.cc:225-246,249-267,662-704.
    pub fn DeleteSelection(
        &self,
        host: &dyn TypingCommandHost<S>,
        selection: &S,
        smart: bool,
    ) -> bool {
        if selection.IsCaret() || !host.IsValid(selection) {
            return false;
        }
        host.PrepareForInsertion();
        self.UpdateSelectionIfDifferentFromCurrentSelection(
            host,
            selection,
            host.EditingUseDomPositionApiEnabled(),
        );
        let command = self.LastOpen(host).unwrap_or_else(|| {
            let (visible, raw) = host.CompositeBirthSelection();
            TypingCommandState::new(
                CompositeEditCommand::new(visible, raw),
                CommandType::DeleteSelection,
            )
        });
        command.input_type.set(InputType::kNone);
        let Some(ending) = host.DeleteSelection(selection, &command.composite, smart) else {
            return false;
        };
        self.TypingAddedToOpenCommand(host, &command, &ending, CommandType::DeleteSelection, None);
        true
    }
}
// Source: core/editing/commands/typing_command.h:97;
// core/editing/commands/typing_command.cc:199-204. The feature is stable in
// the pinned RuntimeEnabledFeatures.json5; the WTF owner performs normalization.
pub fn NormalizeTextForInsertion(text: Vec<u16>) -> Vec<u16> {
    match NormalizeTextForInsertionBorrowed(&text) {
        std::borrow::Cow::Borrowed(_) => text,
        std::borrow::Cow::Owned(normalized) => normalized,
    }
}

/// Borrowed String projection of the same source operation. Unchanged text
/// preserves its canonical owner; UTF-16 units are never decoded/re-encoded.
// Source: typing_command.cc:199-204; platform/wtf/text/line_ending.cc:174-188.
pub fn NormalizeTextForInsertionBorrowed(text: &[u16]) -> std::borrow::Cow<'_, [u16]> {
    crate::platform::wtf::text::line_ending::NormalizeLineEndingsToLf(text)
}

fn CanAppendNewLineFeedToSelection<S: TypingSelection>(
    host: &dyn TypingCommandHost<S>,
    selection: &S,
) -> Option<bool> {
    let text = host.BeforeTextInserted(&[10], selection)?;
    host.IsValid(selection).then_some(!text.is_empty())
}

// Source typing_command.cc:179-193,246-320,617-650,853-1264.
use super::{
    break_blockquote_command::BreakBlockquoteContext,
    editing_commands_utilities::DeletionInputTypeFromTextGranularity,
    editor_command_input_type::SelectionModifyDirection as D,
    editor_command_target_ranges::{
        BackwardDeletionContext, NarrowSelectionToBackwardDeletionUnit,
    },
    paragraph_editing_context::EditResult,
    replace_selection_command::ReplacementFacts,
};
use crate::core::editing::visible_position::DownstreamVisiblePositionContext;
use crate::core::editing::{
    editing_boundaries::EditableBoundaryContext,
    frame_selection::SelectionValue,
    position::{Position, PositionDOM, PositionTree, PositionWithAffinity, TextAffinity},
    selection_modifier::{
        NoXPosForVerticalArrowNavigation, SelectionModifier, SelectionModifierContext,
        SelectionModifyAlteration,
    },
    text_granularity::TextGranularity as G,
    visible_navigation,
    visible_position::{VisiblePosition, VisiblePositionContext},
    visible_selection::VisibleSelection,
    visible_units::EditingBoundaryCrossingRule as Rule,
    visible_units_commands::{self, CommandPositionContext},
};
use interaction_dispatch::core::events::input_event::InputType as I;
type GeoNode<C, S> = <<C as TypingDeletionGeometryContext<S>>::Geometry as PositionTree>::Node;

/// Read-only source geometry, borrowed only during one synchronous query.
/// Effects and DOM mutation never execute while this geometry is borrowed.
pub trait TypingDeletionGeometryContext<S: TypingSelection + SelectionValue> {
    type Geometry: SelectionModifierContext;
    fn Geometry(&self) -> &Self::Geometry;
    fn DOMGeometry(&self) -> &Self::Geometry;
    fn RawDOMSelection(&self, selection: &S) -> VisibleSelection<GeoNode<Self, S>>;
    fn ConvertRawDOMSelectionToFlat(&self, selection: &S) -> VisibleSelection<GeoNode<Self, S>>;
    fn ComputeVisibleDOMSelection(&self, selection: &S) -> VisibleSelection<GeoNode<Self, S>>;
    fn BuildDOMSelection(&self, selection: &VisibleSelection<GeoNode<Self, S>>) -> S;
    fn EnclosingTableCell(&self, position: &Position<GeoNode<Self, S>>)
        -> Option<GeoNode<Self, S>>;
    fn TableElementJustBefore(
        &self,
        position: &VisiblePosition<GeoNode<Self, S>>,
    ) -> Option<GeoNode<Self, S>>;
    fn TableElementJustAfter(
        &self,
        position: &VisiblePosition<GeoNode<Self, S>>,
    ) -> Option<GeoNode<Self, S>>;
    fn CaretMinOffset(&self, node: &GeoNode<Self, S>) -> usize;
    fn AfterBlockIfBeforeAnonymousPlaceholder(
        &self,
        position: &Position<GeoNode<Self, S>>,
    ) -> Option<Position<GeoNode<Self, S>>>;
}

/// The original SelectionModifier and native node/position values survive
/// short geometry borrows. No Document, LayoutOutput or copied tree is stored.
struct BackwardDeletionGeometryState<N> {
    modifier: SelectionModifier<N>,
    visible_start: VisiblePosition<N>,
    previous: VisiblePosition<N>,
    cell: Option<N>,
    needs_list_breakout: bool,
}
pub struct BackwardDeletionGeometry<N>(Rc<RefCell<Option<BackwardDeletionGeometryState<N>>>>);
impl<N> Clone for BackwardDeletionGeometry<N> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<N> BackwardDeletionGeometry<N> {
    pub fn NeedsListBreakout(&self) -> bool {
        self.0
            .borrow()
            .as_ref()
            .expect("live source deletion geometry")
            .needs_list_breakout
    }
}
pub enum DeletionGeometryAction<S> {
    None,
    Select(S),
    Delete { selection: S, after_undo: S },
}

/// Call-scoped command leaves. Each geometry stage lends the current resident
/// output and releases it before an effect can synchronously mutate the DOM.
/// Composite/Editor/undo identity stays on the original command owner.
pub trait TypingDeletionContext<S: TypingSelection + SelectionValue> {
    type Node: Clone + PartialEq;
    type ScrollAnchorSuppression;
    fn HasFrame(&self) -> bool;
    fn IsSelectionDirectional(&self) -> bool;
    fn UpdateStyleAndLayoutForEditing(&self);
    fn RawDOMSelection(&self, selection: &S) -> VisibleSelection<Self::Node>;
    fn SelectionAnchorContainer(&self, selection: &S) -> Option<Self::Node>;
    fn ShouldUndoOfDeleteSelectText(&self) -> bool;
    fn SuppressScrollAnchoring(&self, anchor: Option<&Self::Node>)
        -> Self::ScrollAnchorSuppression;
    fn BreakOutOfEmptyMailBlockquotedParagraph(
        &self,
        command: &Rc<CompositeEditCommand<S>>,
    ) -> EditResult<bool>;
    fn BreakOutOfEmptyListItem(&self, command: &Rc<CompositeEditCommand<S>>) -> EditResult<bool>;
    fn MakeEditableRootEmpty(&self, command: &Rc<CompositeEditCommand<S>>) -> EditResult<bool>;
    fn BeginBackwardDeletionGeometry(
        &self,
        ending: &S,
        directional: bool,
        granularity: G,
        kill_ring: bool,
    ) -> EditResult<BackwardDeletionGeometry<Self::Node>>;
    fn BackwardDeletionNeedsEmptyRoot(
        &self,
        geometry: &BackwardDeletionGeometry<Self::Node>,
    ) -> EditResult<bool>;
    fn FinishBackwardDeletionGeometry(
        &self,
        geometry: BackwardDeletionGeometry<Self::Node>,
        ending: &S,
        starting: &S,
        granularity: G,
    ) -> EditResult<DeletionGeometryAction<S>>;
    fn ForwardDeletionGeometry(
        &self,
        ending: &S,
        starting: &S,
        directional: bool,
        granularity: G,
        kill_ring: bool,
    ) -> EditResult<DeletionGeometryAction<S>>;
    fn AddToKillRing(&self, selection: &S) -> EditResult;
    fn NotifyAccessibilityOfDeletionInTextField(&self, selection: &S);
    fn DeleteSelectionIfRange(
        &self,
        selection: &S,
        command: &Rc<CompositeEditCommand<S>>,
        smart: bool,
        expand_special: bool,
    ) -> EditResult<S>;
}
fn start<C: VisiblePositionContext + EditableBoundaryContext>(
    ctx: &C,
    s: &VisibleSelection<C::Node>,
) -> Position<C::Node> {
    if ctx.ComparePositions(&s.anchor, &s.focus) == Some(std::cmp::Ordering::Greater) {
        s.focus.clone()
    } else {
        s.anchor.clone()
    }
}
fn end<C: VisiblePositionContext + EditableBoundaryContext>(
    ctx: &C,
    s: &VisibleSelection<C::Node>,
) -> Position<C::Node> {
    if ctx.ComparePositions(&s.anchor, &s.focus) == Some(std::cmp::Ordering::Greater) {
        s.anchor.clone()
    } else {
        s.focus.clone()
    }
}
fn visible<C: VisiblePositionContext>(ctx: &C, p: Position<C::Node>) -> VisiblePosition<C::Node> {
    VisiblePosition::Create(
        ctx,
        PositionWithAffinity {
            position: p,
            affinity: TextAffinity::Downstream,
        },
    )
}
fn visible_start<C: VisiblePositionContext + EditableBoundaryContext>(
    ctx: &C,
    s: &VisibleSelection<C::Node>,
) -> VisiblePosition<C::Node> {
    VisiblePosition::Create(
        ctx,
        PositionWithAffinity {
            position: start(ctx, s),
            affinity: if range(s) {
                TextAffinity::Downstream
            } else {
                s.affinity
            },
        },
    )
}
fn selection_visible_end<C: VisiblePositionContext + EditableBoundaryContext>(
    ctx: &C,
    s: &VisibleSelection<C::Node>,
) -> VisiblePosition<C::Node> {
    VisiblePosition::Create(
        ctx,
        PositionWithAffinity {
            position: end(ctx, s),
            affinity: if range(s) {
                TextAffinity::Upstream
            } else {
                s.affinity
            },
        },
    )
}
fn range<N: Clone + PartialEq>(s: &VisibleSelection<N>) -> bool {
    !s.anchor.IsNull() && s.anchor != s.focus
}
fn caret<N: Clone + PartialEq>(s: &VisibleSelection<N>) -> bool {
    !s.anchor.IsNull() && s.anchor == s.focus
}
impl<S: TypingSelection + SelectionValue> TypingCommand<S> {
    /// Static DeleteKeyPressed/ForwardDeleteKeyPressed entry points retain the
    /// source asymmetry: backward character deletion only coalesces DeleteKey;
    /// forward character deletion can reuse any still-open TypingCommand.
    pub fn DeleteKeyPressed<C: TypingDeletionContext<S>>(
        &self,
        host: &dyn TypingCommandHost<S>,
        ctx: &C,
        current: &S,
        options: u32,
        granularity: G,
        forward: bool,
    ) -> EditResult {
        let reusable = if granularity == G::kCharacter {
            self.LastOpen(host)
                .filter(|command| forward || command.kind.get() == CommandType::DeleteKey)
        } else {
            None
        };
        let created_command = reusable.is_none();
        if !created_command {
            self.UpdateSelectionIfDifferentFromCurrentSelection(
                host,
                current,
                host.EditingUseDomPositionApiEnabled(),
            );
        }
        let command = reusable.unwrap_or_else(|| {
            let (visible, raw) = host.CompositeBirthSelection();
            let composite = CompositeEditCommand::new(visible, raw);
            composite.SetSelectionIsDirectional(ctx.IsSelectionDirectional());
            TypingCommandState::new(
                composite,
                if forward {
                    CommandType::ForwardDeleteKey
                } else {
                    CommandType::DeleteKey
                },
            )
        });
        command.input_type.set(I::kNone);
        if created_command {
            command.smart_delete.set(options & kSmartDelete != 0);
            command.granularity.set(granularity);
            if !forward {
                command.opened_by_backward_delete.set(true);
            }
        }
        let ending = command.composite.EndingSelection();
        if created_command && (SelectionValue::IsNone(&ending) || !host.IsValid(&ending)) {
            return Ok(());
        }
        let anchor = ctx.SelectionAnchorContainer(&ending);
        let _suppression = if created_command {
            Some(ctx.SuppressScrollAnchoring(anchor.as_ref()))
        } else {
            None
        };
        if forward {
            self.ForwardDeleteInternal(host, ctx, &command, granularity, options & kKillRing != 0)
        } else {
            self.BackwardDeleteInternal(host, ctx, &command, granularity, options & kKillRing != 0)
        }
    }
    fn PublishDeletion<C: TypingDeletionContext<S>>(
        &self,
        host: &dyn TypingCommandHost<S>,
        ctx: &C,
        command: &Rc<TypingCommandState<S>>,
        selection: S,
        forward: bool,
    ) {
        if !ctx.HasFrame() {
            return;
        }
        self.TypingAddedToOpenCommand(
            host,
            command,
            &selection,
            if forward {
                CommandType::ForwardDeleteKey
            } else {
                CommandType::DeleteKey
            },
            None,
        );
    }
    fn DeleteInternalSelection<C: TypingDeletionContext<S>>(
        &self,
        host: &dyn TypingCommandHost<S>,
        ctx: &C,
        command: &Rc<TypingCommandState<S>>,
        to_delete: &S,
        after_undo: &S,
        kill_ring: bool,
        forward: bool,
    ) -> EditResult {
        let raw = ctx.RawDOMSelection(to_delete);
        if raw.anchor.IsNull() || caret(&raw) {
            return Ok(());
        }
        if kill_ring {
            ctx.AddToKillRing(to_delete)?;
        }
        if ctx.ShouldUndoOfDeleteSelectText()
            && (forward || command.opened_by_backward_delete.get())
        {
            command.composite.SetStartingSelection(after_undo.clone());
            if host.EditingUseDomPositionApiEnabled() {
                command
                    .composite
                    .SetStartingDomSelection(after_undo.clone());
            }
        }
        if !forward {
            ctx.NotifyAccessibilityOfDeletionInTextField(to_delete);
        }
        let expand = range(&ctx.RawDOMSelection(&command.composite.EndingSelection()));
        let ending = ctx.DeleteSelectionIfRange(
            to_delete,
            &command.composite,
            command.smart_delete.get(),
            expand,
        )?;
        command.smart_delete.set(false);
        self.PublishDeletion(host, ctx, command, ending, forward);
        Ok(())
    }
    fn ApplyDeletionGeometry<C: TypingDeletionContext<S>>(
        &self,
        host: &dyn TypingCommandHost<S>,
        ctx: &C,
        command: &Rc<TypingCommandState<S>>,
        action: DeletionGeometryAction<S>,
        kill_ring: bool,
        forward: bool,
    ) -> EditResult {
        match action {
            DeletionGeometryAction::None => Ok(()),
            DeletionGeometryAction::Select(selection) => {
                command.composite.SetEndingSelection(selection.clone());
                if host.EditingUseDomPositionApiEnabled() {
                    command.composite.SetEndingDomSelection(selection.clone());
                }
                self.PublishDeletion(host, ctx, command, selection, forward);
                Ok(())
            }
            DeletionGeometryAction::Delete {
                selection,
                after_undo,
            } => self.DeleteInternalSelection(
                host,
                ctx,
                command,
                &selection,
                &after_undo,
                kill_ring,
                forward,
            ),
        }
    }
    fn BackwardDeleteInternal<C: TypingDeletionContext<S>>(
        &self,
        host: &dyn TypingCommandHost<S>,
        ctx: &C,
        command: &Rc<TypingCommandState<S>>,
        granularity: G,
        kill_ring: bool,
    ) -> EditResult {
        if !ctx.HasFrame() {
            return Ok(());
        }
        let ending = command.composite.EndingSelection();
        let raw = ctx.RawDOMSelection(&ending);
        if range(&raw) {
            return self
                .DeleteInternalSelection(host, ctx, command, &ending, &ending, kill_ring, false);
        }
        assert!(caret(&raw));
        if ctx.BreakOutOfEmptyMailBlockquotedParagraph(&command.composite)? {
            self.PublishDeletion(
                host,
                ctx,
                command,
                command.composite.EndingSelection(),
                false,
            );
        }
        command.smart_delete.set(false);
        ctx.UpdateStyleAndLayoutForEditing();
        let geometry = ctx.BeginBackwardDeletionGeometry(
            &command.composite.EndingSelection(),
            command.composite.SelectionIsDirectional(),
            granularity,
            kill_ring,
        )?;
        if geometry.NeedsListBreakout() && ctx.BreakOutOfEmptyListItem(&command.composite)? {
            self.PublishDeletion(
                host,
                ctx,
                command,
                command.composite.EndingSelection(),
                false,
            );
            return Ok(());
        }
        if ctx.BackwardDeletionNeedsEmptyRoot(&geometry)?
            && ctx.MakeEditableRootEmpty(&command.composite)?
        {
            self.PublishDeletion(
                host,
                ctx,
                command,
                command.composite.EndingSelection(),
                false,
            );
            return Ok(());
        }
        let action = ctx.FinishBackwardDeletionGeometry(
            geometry,
            &command.composite.EndingSelection(),
            &command.composite.StartingSelection(),
            granularity,
        )?;
        self.ApplyDeletionGeometry(host, ctx, command, action, kill_ring, false)
    }
    fn ForwardDeleteInternal<C: TypingDeletionContext<S>>(
        &self,
        host: &dyn TypingCommandHost<S>,
        ctx: &C,
        command: &Rc<TypingCommandState<S>>,
        granularity: G,
        kill_ring: bool,
    ) -> EditResult {
        if !ctx.HasFrame() {
            return Ok(());
        }
        let ending = command.composite.EndingSelection();
        let raw = ctx.RawDOMSelection(&ending);
        if range(&raw) {
            return self
                .DeleteInternalSelection(host, ctx, command, &ending, &ending, kill_ring, true);
        }
        assert!(caret(&raw));
        command.smart_delete.set(false);
        ctx.UpdateStyleAndLayoutForEditing();
        let action = ctx.ForwardDeletionGeometry(
            &ending,
            &command.composite.StartingSelection(),
            command.composite.SelectionIsDirectional(),
            granularity,
            kill_ring,
        )?;
        self.ApplyDeletionGeometry(host, ctx, command, action, kill_ring, true)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AnonymousPlaceholderFacts<N> {
    pub is_br: bool,
    pub has_layout_sibling: bool,
    pub parent_is_anonymous: bool,
    pub previous_parent_non_pseudo_node: Option<N>,
}
pub trait AnonymousPlaceholderContext: PositionDOM {
    fn AnonymousPlaceholderFacts(
        &self,
        node: &Self::Node,
    ) -> Option<AnonymousPlaceholderFacts<Self::Node>>;
}
/// Source typing_command.cc:179-193. Layout returns only actual resident facts;
/// this source owner decides whether the special placeholder case applies.
pub fn AfterBlockIfBeforeAnonymousPlaceholder<C: AnonymousPlaceholderContext>(
    ctx: &C,
    position: &Position<C::Node>,
) -> Option<Position<C::Node>> {
    if !position.IsBeforeAnchor() {
        return None;
    }
    let facts = ctx.AnonymousPlaceholderFacts(position.AnchorNode()?)?;
    if !facts.is_br || facts.has_layout_sibling || !facts.parent_is_anonymous {
        return None;
    }
    Some(Position::AfterNode(facts.previous_parent_non_pseudo_node?))
}

// The following source geometry stages execute under a short resident borrow.
pub fn ModifierForDeletion<
    S: TypingSelection + SelectionValue,
    C: TypingDeletionGeometryContext<S>,
>(
    ctx: &C,
    ending: &S,
    directional: bool,
    granularity: G,
    kill_ring: bool,
    forward: bool,
) -> SelectionModifier<GeoNode<C, S>>
where
    C::Geometry: BackwardDeletionContext + CommandPositionContext,
{
    let mut modifier = SelectionModifier::new(
        ctx.ConvertRawDOMSelectionToFlat(ending),
        NoXPosForVerticalArrowNavigation(),
    );
    modifier.SetSelectionIsDirectional(directional);
    let direction = if forward { D::Forward } else { D::Backward };
    modifier.Modify(
        ctx.Geometry(),
        SelectionModifyAlteration::Extend,
        direction,
        granularity,
    );
    if kill_ring && caret(&modifier.Selection(ctx.Geometry())) && granularity != G::kCharacter {
        modifier.Modify(
            ctx.Geometry(),
            SelectionModifyAlteration::Extend,
            direction,
            G::kCharacter,
        );
    }
    modifier
}

pub fn BeginBackwardDeletionGeometry<
    S: TypingSelection + SelectionValue,
    C: TypingDeletionGeometryContext<S>,
>(
    ctx: &C,
    ending_selection: &S,
    directional: bool,
    granularity: G,
    kill_ring: bool,
) -> BackwardDeletionGeometry<GeoNode<C, S>>
where
    C::Geometry: BackwardDeletionContext + CommandPositionContext,
{
    let modifier = ModifierForDeletion(
        ctx,
        ending_selection,
        directional,
        granularity,
        kill_ring,
        false,
    );
    let ending = ctx.ComputeVisibleDOMSelection(ending_selection);
    let visible_start = visible_start(ctx.DOMGeometry(), &ending);
    let previous = visible_navigation::PreviousPositionOf(
        ctx.DOMGeometry(),
        visible_start.DeepEquivalent(),
        Rule::CannotCross,
    );
    let cell = ctx.EnclosingTableCell(visible_start.DeepEquivalent());
    let needs_list_breakout =
        previous.IsNull() || cell != ctx.EnclosingTableCell(previous.DeepEquivalent());
    BackwardDeletionGeometry(Rc::new(RefCell::new(Some(BackwardDeletionGeometryState {
        modifier,
        visible_start,
        previous,
        cell,
        needs_list_breakout,
    }))))
}
pub fn BackwardDeletionNeedsEmptyRoot<
    S: TypingSelection + SelectionValue,
    C: TypingDeletionGeometryContext<S>,
>(
    ctx: &C,
    geometry: &BackwardDeletionGeometry<GeoNode<C, S>>,
) -> bool
where
    C::Geometry: BackwardDeletionContext + CommandPositionContext,
{
    let geometry = geometry.0.borrow();
    let geometry = geometry.as_ref().expect("live source deletion geometry");
    geometry.previous.IsNull()
        && visible_navigation::NextPositionOf(
            ctx.DOMGeometry(),
            &geometry.visible_start.ToPositionWithAffinity(),
            Rule::CannotCross,
        )
        .IsNull()
}
pub fn FinishBackwardDeletionGeometry<
    S: TypingSelection + SelectionValue,
    C: TypingDeletionGeometryContext<S>,
>(
    ctx: &C,
    geometry: BackwardDeletionGeometry<GeoNode<C, S>>,
    ending_selection: &S,
    starting_selection: &S,
    granularity: G,
) -> DeletionGeometryAction<S>
where
    C::Geometry: BackwardDeletionContext + CommandPositionContext,
{
    let BackwardDeletionGeometryState {
        mut modifier,
        visible_start,
        previous,
        cell,
        ..
    } = geometry
        .0
        .borrow_mut()
        .take()
        .expect("source deletion geometry consumed once");
    if cell.as_ref().is_some_and(|cell| {
        visible_start.DeepEquivalent()
            == visible(
                ctx.DOMGeometry(),
                Position::FirstPositionInNode(cell.clone()),
            )
            .DeepEquivalent()
    }) {
        return DeletionGeometryAction::None;
    }
    if ctx
        .Geometry()
        .Behavior()
        .ShouldMergeContentWithTablesOnBackspace()
        && visible_units_commands::IsStartOfParagraph(ctx.DOMGeometry(), &visible_start)
        && ctx.TableElementJustBefore(&previous).is_some()
    {
        if ctx.TableElementJustAfter(&visible_start).is_some() {
            return DeletionGeometryAction::None;
        }
        modifier.Modify(
            ctx.Geometry(),
            SelectionModifyAlteration::Extend,
            D::Backward,
            granularity,
        );
    } else if let Some(table) = ctx.TableElementJustBefore(&visible_start) {
        let result = ctx.BuildDOMSelection(&VisibleSelection {
            anchor: Position::BeforeNode(table),
            focus: start(ctx.DOMGeometry(), &ctx.RawDOMSelection(ending_selection)),
            affinity: TextAffinity::Downstream,
        });
        return DeletionGeometryAction::Select(result);
    }
    let mut selection = modifier.Selection(ctx.Geometry());
    if granularity == G::kCharacter {
        if caret(&selection) {
            if let Some(after) = ctx.AfterBlockIfBeforeAnonymousPlaceholder(&selection.anchor) {
                selection.focus = after;
            }
        } else {
            selection = NarrowSelectionToBackwardDeletionUnit(ctx.DOMGeometry(), &selection);
        }
    }
    let to_delete = ctx.BuildDOMSelection(&selection);
    let starting = ctx.RawDOMSelection(starting_selection);
    if !range(&starting) || selection.anchor != start(ctx.DOMGeometry(), &starting) {
        return DeletionGeometryAction::Delete {
            selection: to_delete.clone(),
            after_undo: to_delete,
        };
    }
    let after_undo = ctx.BuildDOMSelection(&VisibleSelection {
        anchor: end(ctx.DOMGeometry(), &starting),
        focus: visible(ctx.DOMGeometry(), selection.focus)
            .DeepEquivalent()
            .clone(),
        affinity: TextAffinity::Downstream,
    });
    DeletionGeometryAction::Delete {
        selection: to_delete,
        after_undo,
    }
}

pub fn ForwardDeletionGeometry<
    S: TypingSelection + SelectionValue,
    C: TypingDeletionGeometryContext<S>,
>(
    ctx: &C,
    ending_selection: &S,
    starting_selection: &S,
    directional: bool,
    granularity: G,
    kill_ring: bool,
) -> DeletionGeometryAction<S>
where
    C::Geometry: BackwardDeletionContext + CommandPositionContext,
{
    let raw = ctx.RawDOMSelection(ending_selection);
    let mut modifier = ModifierForDeletion(
        ctx,
        ending_selection,
        directional,
        granularity,
        kill_ring,
        true,
    );
    let ending = ctx.ComputeVisibleDOMSelection(ending_selection);
    let mut downstream_end = DownstreamVisiblePositionContext::MostForwardCaretPosition(
        ctx.DOMGeometry(),
        &end(ctx.DOMGeometry(), &raw),
    );
    let visible_end = selection_visible_end(ctx.DOMGeometry(), &ending);
    if ctx
        .EnclosingTableCell(visible_end.DeepEquivalent())
        .is_some_and(|cell| {
            visible_end.DeepEquivalent()
                == visible(
                    ctx.DOMGeometry(),
                    Position::LastPositionInNode(ctx.DOMGeometry(), &cell),
                )
                .DeepEquivalent()
        })
    {
        return DeletionGeometryAction::None;
    }
    if visible_end.DeepEquivalent()
        == visible_units_commands::EndOfParagraph(ctx.DOMGeometry(), &visible_end).DeepEquivalent()
    {
        let next = visible_navigation::NextPositionOf(
            ctx.DOMGeometry(),
            &visible_end.ToPositionWithAffinity(),
            Rule::CannotCross,
        );
        downstream_end = DownstreamVisiblePositionContext::MostForwardCaretPosition(
            ctx.DOMGeometry(),
            next.DeepEquivalent(),
        );
    }
    if let Some(container) = downstream_end.ComputeContainerNode(ctx.DOMGeometry()) {
        if ctx.DOMGeometry().IsDisplayInsideTable(&container)
            && downstream_end.ComputeOffsetInContainerNode(ctx.DOMGeometry())
                <= ctx.CaretMinOffset(&container)
        {
            let result = ctx.BuildDOMSelection(&VisibleSelection {
                anchor: end(ctx.DOMGeometry(), &raw),
                focus: Position::AfterNode(container),
                affinity: TextAffinity::Downstream,
            });
            return DeletionGeometryAction::Select(result);
        }
    }
    let modified = modifier.Selection(ctx.Geometry());
    if granularity == G::kParagraphBoundary
        && caret(&modified)
        && visible_units_commands::IsEndOfParagraph(
            ctx.DOMGeometry(),
            &selection_visible_end(ctx.DOMGeometry(), &modified),
        )
    {
        modifier.Modify(
            ctx.Geometry(),
            SelectionModifyAlteration::Extend,
            D::Forward,
            G::kCharacter,
        );
    }
    let selection = modifier.Selection(ctx.Geometry());
    let to_delete = ctx.BuildDOMSelection(&selection);
    let starting = ctx.RawDOMSelection(starting_selection);
    if !range(&starting)
        || DownstreamVisiblePositionContext::MostBackwardCaretPosition(
            ctx.DOMGeometry(),
            &selection.anchor,
        ) != start(ctx.DOMGeometry(), &starting)
    {
        return DeletionGeometryAction::Delete {
            selection: to_delete.clone(),
            after_undo: to_delete,
        };
    }
    let extent = end(ctx.DOMGeometry(), &starting);
    let selected_end = end(ctx.DOMGeometry(), &selection);
    let focus = if extent.ComputeContainerNode(ctx.DOMGeometry())
        != selected_end.ComputeContainerNode(ctx.DOMGeometry())
    {
        selection.focus.clone()
    } else {
        let selected_start = start(ctx.DOMGeometry(), &selection);
        let extra = if selected_start.ComputeContainerNode(ctx.DOMGeometry())
            == selected_end.ComputeContainerNode(ctx.DOMGeometry())
        {
            selected_end.ComputeOffsetInContainerNode(ctx.DOMGeometry())
                - selected_start.ComputeOffsetInContainerNode(ctx.DOMGeometry())
        } else {
            selected_end.ComputeOffsetInContainerNode(ctx.DOMGeometry())
        };
        Position::new(
            extent
                .ComputeContainerNode(ctx.DOMGeometry())
                .expect("nonnull source undo extent"),
            extent.ComputeOffsetInContainerNode(ctx.DOMGeometry()) + extra,
        )
    };
    let after_undo = ctx.BuildDOMSelection(&VisibleSelection {
        anchor: start(ctx.DOMGeometry(), &starting),
        focus,
        affinity: TextAffinity::Downstream,
    });
    DeletionGeometryAction::Delete {
        selection: to_delete,
        after_undo,
    }
}

// Source typing_command.cc:855-898.
pub trait TypingDeletionStructuralContext: BreakBlockquoteContext + ReplacementFacts {
    /// Source sets the shared FrameSelection before removing root contents.
    fn SetFrameSelectionBeforeRootRemoval(&self, position: &Position<u64>);
}

pub fn MakeEditableRootEmpty<C: TypingDeletionStructuralContext>(
    ctx: &C,
    ending: &Position<u64>,
) -> EditResult<bool> {
    let Some(root) = ending.AnchorNode().and_then(|node| ctx.RootEditable(*node)) else {
        return Ok(false);
    };
    let Some(first) = ctx.ChildAt(&root, 0) else {
        return Ok(false);
    };
    if ctx.ChildCount(&root) == 1 && ctx.Tag(first, "br") && ctx.NodeFacts(root).block_flow {
        return Ok(false);
    }
    let start = Position::FirstPositionInNode(root);
    ctx.SetFrameSelectionBeforeRootRemoval(&start);
    ctx.SetEndingPosition(&start);
    // RemoveAllChildrenIfPossible preserves the next original sibling across
    // synchronous mutation and stops if reentry moved it out of the container.
    let mut child = Some(first);
    while let Some(node) = child {
        let next = ctx.NextSibling(&node);
        ctx.Remove(node)?;
        if next.is_some_and(|next| ctx.Parent(&next) != Some(root)) {
            break;
        }
        child = next;
    }
    if ctx.ChildCount(&root) != 0 {
        return Ok(false);
    }
    ctx.UpdateStyleAndLayout();
    let facts = ctx.BlockPlaceholderFacts(root);
    if facts.block_flow && (facts.height_zero || facts.empty_list_item) {
        let br = ctx.CreateElement("br")?;
        ctx.Append(br, root)?;
    }
    Ok(true)
}
