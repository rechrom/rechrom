//! Native command ports over the existing frame/editor owners.
use super::{EditorCommandError, EventDispatcher};
use crate::adapter::event::EventTargetResolver;
use crate::adapter::page_editing_visible_position::{
    EditorSelectionGeometryQuery as EQ, EditorSelectionGeometryResult as ER,
};
use crate::adapter::page_frame_selection::{
    CompositionTextQuery as Q, CompositionTextResult as R, FrameSelectionValue as S,
};
use interaction_default::core::editing::commands::{
    editor_command_table::{EditingCommandType, EditorCommandSource},
    editor_selection_commands::{self, MarkCommandContext, TransposeCommandContext},
};
use interaction_default::core::editing::set_selection_options::SetSelectionOptions;
use interaction_default::core::editing::{
    commands::{
        editor_command_input_type::SelectionModifyDirection, editor_command_table::ExecuteFunction,
    },
    frame_selection_modify::{
        FrameSelectionCommandContext, PreparedSelectionModification, SelectionModificationState,
    },
    selection_modifier::SelectionModifyAlteration,
    text_granularity::TextGranularity,
};
use interaction_dispatch::core::dom::events::{
    event::{Event, EventInterface, EventRef},
    event_dispatch_result::DispatchEventResult,
};
use layoutng_assembly::{editing_state::DOMSelection, EditingPosition, EditingTextAffinity};

struct ExtendedContext<'a, 'listener, 'frame>(&'a EventDispatcher<'listener, 'frame>, u128);
impl MarkCommandContext for ExtendedContext<'_, '_, '_> {
    type Selection = DOMSelection;
    type Error = EditorCommandError;
    fn CurrentVisibleSelection(&self) -> Option<DOMSelection> {
        self.0
            .FrameSelection()
            .CreateVisibleSelection(self.0.FrameSelection().Current())
    }
    fn MarkVisibleSelection(&self) -> Option<DOMSelection> {
        self.0.FrameSelection().CreateVisibleSelection(
            self.0
                .EditingOwners()
                .ime_typing
                .editor
                .Mark()
                .map(|s| s.0),
        )
    }
    fn MarkIsDirectional(&self) -> bool {
        self.0
            .EditingOwners()
            .ime_typing
            .editor
            .MarkIsDirectional()
    }
    fn SetMarkFromCurrentSelection(&self) {
        self.0
            .EditingOwners()
            .ime_typing
            .editor
            .SetMarkSelection(
                self.CurrentVisibleSelection()
                    .map(crate::adapter::page_input_method_controller::ImeTypingSelection),
                self.0.FrameSelection().SelectionIsDirectional(),
            );
    }
    fn UnionMarkAndSelection(
        &self,
        mark: DOMSelection,
        selection: DOMSelection,
    ) -> Result<Option<DOMSelection>, EditorCommandError> {
        match self
            .0
            .EditorSelectionGeometry(EQ::Union { selection, mark })?
        {
            ER::Selection(selection) => Ok(selection),
            _ => Err(EditorCommandError::RequiredPort("mark selection union")),
        }
    }
    fn SetSelection(&self, selection: DOMSelection, options: SetSelectionOptions) {
        self.0
            .FrameSelection()
            .SetSelection(Some(selection), options);
    }
    fn DeleteSelectionWithKillRing(&self) -> Result<(), EditorCommandError> {
        crate::adapter::page_input_method_controller::DeleteEditorSelection(self.0)
            .map_err(EditorCommandError::Aborted)
    }
    fn UpdateStyleAndLayout(&self) {
        self.0.FrameSelection().PrepareEditing();
    }
}
impl TransposeCommandContext for ExtendedContext<'_, '_, '_> {
    type Range = dom::DOMStaticRange;
    type Error = EditorCommandError;
    fn PrepareForEditing(&self) {
        self.0.FrameSelection().PrepareEditing();
    }
    fn ComputeRangeAndText(
        &self,
    ) -> Result<Option<(dom::DOMStaticRange, [u16; 2])>, EditorCommandError> {
        let Some(selection) = self.0.FrameSelection().Current() else {
            return Ok(None);
        };
        let ER::Range(range) = self.0.EditorSelectionGeometry(EQ::Transpose(selection))? else {
            return Err(EditorCommandError::RequiredPort("transpose range"));
        };
        let Some(range) = range else {
            return Ok(None);
        };
        let Some(R::Text(text)) = self.0.FrameSelection().CompositionText(Q::Text {
            range,
            original: false,
        }) else {
            return Err(EditorCommandError::RequiredPort("transpose PlainText"));
        };
        Ok(<[u16; 2]>::try_from(text).ok().map(|text| (range, text)))
    }
    fn DispatchBeforeInput(
        &self,
        range: &dom::DOMStaticRange,
        text: &[u16; 2],
    ) -> Result<bool, EditorCommandError> {
        let Some(target) = self
            .0
            .KeyboardTargets()
            .EventTargetNodeForDocument(self.0.KeyboardState().borrow().focused_node_id)
        else {
            return Ok(true);
        };
        let range =
            super::editor_target_ranges::StaticRangeForInputEvent(self.0.document.clone(), *range)
                .map_err(|_| EditorCommandError::RequiredPort("transpose StaticRange"))?;
        let ranges = [range];
        let mut event = super::NativeInputEvent::CreateBeforeInputUTF16(
            super::NativeInputType::kInsertTranspose,
            std::rc::Rc::from(text.as_slice()),
            super::EventIsComposing::kNotComposing,
            Some(&ranges),
            self.0.EventClock(),
        );
        event.BaseEventMut().SetTrusted(true);
        Ok(self
            .0
            .DispatchCanonicalEventTarget(&target, EventRef::new(event))
            .0
            == DispatchEventResult::kNotCanceled)
    }
    fn IsDocumentAttached(&self) -> bool {
        self.0.FrameSelection().IsAvailable()
            && self
                .0
                .document
                .borrow()
                .GetDocument()
                .RootHandle()
                .Identity()
                == self.1
    }
    fn SelectRangeIfDifferent(&self, range: &dom::DOMStaticRange) {
        let selection = DOMSelection {
            anchor: EditingPosition::new(range.start.node_id, range.start.offset),
            focus: EditingPosition::new(range.end.node_id, range.end.offset),
            affinity: EditingTextAffinity::Downstream,
        };
        if self
            .0
            .FrameSelection()
            .CreateVisibleSelection(Some(selection))
            != self.CurrentVisibleSelection()
        {
            self.0
                .FrameSelection()
                .SetSelectionAndEndTyping(Some(selection));
        }
    }
    fn ReplaceSelection(&self, text: &[u16; 2]) -> Result<(), EditorCommandError> {
        if !self.IsDocumentAttached() {
            return Ok(());
        }
        crate::adapter::page_input_method_controller::ReplaceEditorSelectionText(
            self.0,
            text,
            super::NativeInputType::kInsertTranspose,
        )
        .map_err(EditorCommandError::RequiredPort)
    }
}

struct MovementContext<'a, 'listener, 'frame>(
    &'a EventDispatcher<'listener, 'frame>,
    std::cell::Cell<Option<&'static str>>,
);
impl
    interaction_default::core::editing::frame_selection_select_all::FrameSelectionSelectAllContext<
        S,
    > for MovementContext<'_, '_, '_>
{
    fn SelectFocusedControl(&self) -> Result<bool, &'static str> {
        let focused = self.0.KeyboardState().borrow().focused_node_id;
        Ok(focused.is_some_and(|node| {
            crate::adapter::page_input_type::SelectAllSelectControl(&self.0.Context(), node)
        }))
    }
    fn Facts(
        &self,
    ) -> Option<interaction_default::core::editing::frame_selection_select_all::SelectAllFacts>
    {
        match self.0.FrameSelection().CompositionText(Q::SelectAllFacts {
            selection: self.0.FrameSelection().Current(),
            focused: self.0.KeyboardState().borrow().focused_node_id,
        })? {
            R::SelectAllFacts(facts) => Some(facts),
            _ => None,
        }
    }
    fn EditingIgnoresContent(&self, root: u64) -> bool {
        let owner = self.0.document.borrow();
        let document = owner.GetDocument();
        document
            .FindNodeById(root)
            .is_none_or(|node| document.EditingIgnoresContent(node))
    }
    fn DispatchSelectStart(&self, target: u64) -> DispatchEventResult {
        let Some(target) = self.0.KeyboardTargets().ResolveTarget(target) else {
            return DispatchEventResult::kNotCanceled;
        };
        self.0
            .DispatchCanonicalEventTarget(
                &target,
                EventRef::new(Event::CreateCancelableBubble(
                    "selectstart",
                    self.0.EventClock(),
                )),
            )
            .0
    }
    fn IsConnected(&self, root: u64) -> bool {
        let owner = self.0.document.borrow();
        let document = owner.GetDocument();
        document
            .FindNodeById(root)
            .is_some_and(|node| document.IsConnectedToDocument(node))
    }
    fn SelectAllChildren(&self, root: u64, canonicalize: bool) -> S {
        use layoutng_assembly::{
            editing_state::DOMSelection, EditingPosition, EditingTextAffinity,
        };
        let selection = {
            let owner = self.0.document.borrow();
            let document = owner.GetDocument();
            document.FindNodeById(root).map(|node| DOMSelection {
                anchor: EditingPosition::new(root, 0),
                focus: EditingPosition::new(root, document.Node(node).Children().len()),
                affinity: EditingTextAffinity::Downstream,
            })
        };
        if canonicalize {
            self.0.FrameSelection().PrepareEditing();
            S(self.0.FrameSelection().CreateVisibleSelection(selection))
        } else {
            S(selection)
        }
    }
    fn ShowSelectionContextMenu(&self) -> Result<(), &'static str> {
        Err("SelectAll touch context menu services")
    }
}
impl FrameSelectionCommandContext<S> for MovementContext<'_, '_, '_> {
    type AXIntentScope = ();
    fn PrepareModification(
        &self,
        selection: &S,
        state: SelectionModificationState,
        alteration: SelectionModifyAlteration,
        direction: SelectionModifyDirection,
        granularity: TextGranularity,
    ) -> Option<PreparedSelectionModification<S>> {
        let selection = selection.0?;
        let Some(settings) = self.0.FrameSelection().InteractionTextSettings() else {
            self.1.set(Some("selection movement editing environment"));
            return None;
        };
        match self.0.FrameSelection().CompositionText(Q::ModifySelection {
            selection,
            state,
            alteration,
            direction,
            granularity,
            behavior: settings.editing_behavior,
            ui_locale: settings.ui_locale,
        }) {
            Some(R::ModifiedSelection(result)) => Some(result),
            _ => {
                self.1.set(Some("resident selection movement geometry"));
                None
            }
        }
    }
    fn IsRange(&self, selection: &S) -> bool {
        selection.0.is_some_and(|s| s.anchor != s.focus)
    }
    fn CurrentVisibleSelectionIsCaret(&self) -> bool {
        self.0
            .FrameSelection()
            .CreateVisibleSelection(self.0.FrameSelection().Current())
            .is_some_and(|s| s.anchor == s.focus)
    }
    fn DispatchSelectStart(&self) -> DispatchEventResult {
        let Some(target) = self
            .0
            .FrameSelection()
            .Current()
            .and_then(|s| s.anchor.node_id)
            .and_then(|id| self.0.KeyboardTargets().ResolveTarget(id))
        else {
            return DispatchEventResult::kNotCanceled;
        };
        self.0
            .DispatchCanonicalEventTarget(
                &target,
                EventRef::new(Event::CreateCancelableBubble(
                    "selectstart",
                    self.0.EventClock(),
                )),
            )
            .0
    }
    fn CanonicalizeSelection(&self, selection: &S) -> S {
        S(self.0.FrameSelection().CreateVisibleSelection(selection.0))
    }
    fn BeginAXSelectionIntent(
        &self,
        _intent: interaction_dispatch::core::dom::events::event_dispatcher::AXSelectionIntent,
    ) -> Option<()> {
        // Native AX intent scopes are not yet exposed by the frame host;
        // FrameSelection still emits its existing accessibility change effect.
        None
    }
}
impl EventDispatcher<'_, '_> {
    fn EditorSelectionGeometry(&self, query: EQ) -> Result<ER, EditorCommandError> {
        let settings = self.FrameSelection().InteractionTextSettings().ok_or(
            EditorCommandError::RequiredPort("editor selection environment"),
        )?;
        match self
            .FrameSelection()
            .CompositionText(Q::EditorSelectionGeometry {
                query,
                behavior: settings.editing_behavior,
                ui_locale: settings.ui_locale,
            }) {
            Some(R::EditorSelectionGeometry(result)) => Ok(result),
            _ => Err(EditorCommandError::RequiredPort(
                "resident editor selection geometry",
            )),
        }
    }
    fn CanExecuteVisibleSelectionCommand(&self, source: EditorCommandSource) -> bool {
        if source == EditorCommandSource::kDom
            && self
                .input_method_state
                .as_ref()
                .is_some_and(|state| state.GetActiveEditContext().is_some())
        {
            return false;
        }
        self.FrameSelection().PrepareEditing();
        if source == EditorCommandSource::kMenuOrKeyBinding
            && !matches!(
                self.FrameSelection().CompositionText(Q::SelectionFocus {
                    selection: self.FrameSelection().Current(),
                    focused: self.KeyboardState().borrow().focused_node_id
                }),
                Some(R::SelectionFocus(true))
            )
        {
            return false;
        }
        let Some(selection) = self
            .FrameSelection()
            .CreateVisibleSelection(self.FrameSelection().Current())
        else {
            return false;
        };
        if selection.anchor != selection.focus {
            return true;
        }
        let owner = self.document.borrow();
        selection.anchor.node_id.is_some_and(|node| {
            foundation::editable::IsEditable(
                &dom::persistent_document::DOMEditableFacts(owner.GetDocument()),
                node,
                Default::default(),
            )
        })
    }
    pub(super) fn ExecuteExtendedSelectionCommand(
        &self,
        command: EditingCommandType,
        source: EditorCommandSource,
    ) -> Option<Result<bool, EditorCommandError>> {
        let execute = command.Descriptor()?.execute;
        use ExecuteFunction as F;
        let granularity = match execute {
            F::ExecuteSelectWord => Some(TextGranularity::kWord),
            F::ExecuteSelectLine => Some(TextGranularity::kLine),
            F::ExecuteSelectParagraph => Some(TextGranularity::kParagraph),
            F::ExecuteSelectSentence => Some(TextGranularity::kSentence),
            _ => None,
        };
        let marks = matches!(
            execute,
            F::ExecuteSetMark
                | F::ExecuteSelectToMark
                | F::ExecuteSwapWithMark
                | F::ExecuteDeleteToMark
        );
        if granularity.is_none() && !marks && execute != F::ExecuteTranspose {
            return None;
        }
        if !self.CanExecuteVisibleSelectionCommand(source) {
            return Some(Ok(false));
        }
        let context = ExtendedContext(
            self,
            self.document.borrow().GetDocument().RootHandle().Identity(),
        );
        if let Some(granularity) = granularity {
            return Some((|| {
                let Some(selection) = self.FrameSelection().Current() else {
                    return Ok(false);
                };
                let ER::Selection(selection) = self.EditorSelectionGeometry(EQ::Expand {
                    selection,
                    granularity,
                })?
                else {
                    return Err(EditorCommandError::RequiredPort(
                        "selection granularity expansion",
                    ));
                };
                let Some(selection) = selection else {
                    return Ok(false);
                };
                self.FrameSelection().SetSelection(
                    Some(selection),
                    SetSelectionOptions {
                        should_close_typing: true,
                        ..Default::default()
                    },
                );
                Ok(true)
            })());
        }
        if marks {
            if execute != F::ExecuteSetMark && context.MarkVisibleSelection().is_none() {
                return Some(Ok(false));
            }
            return editor_selection_commands::ExecuteMarkCommand(&context, execute);
        }
        if !crate::adapter::page_input_method_controller::CanExecuteEditingCommand(self) {
            return Some(Ok(false));
        }
        Some(editor_selection_commands::ExecuteTranspose(&context))
    }
    pub(super) fn ExecuteSelectionCommand(
        &self,
        command: ExecuteFunction,
        source: interaction_default::core::editing::commands::editor_command_table::EditorCommandSource,
    ) -> Option<Result<bool, EditorCommandError>> {
        use interaction_default::core::editing::{
            commands::editor_command_table::EditorCommandSource as Source,
            set_selection_options::SetSelectionBy,
        };
        if source == Source::kDom
            && self
                .input_method_state
                .as_ref()
                .is_some_and(|state| state.GetActiveEditContext().is_some())
            && matches!(
                command,
                ExecuteFunction::ExecuteSelectAll | ExecuteFunction::ExecuteUnselect
            )
        {
            return Some(Ok(false));
        }
        match command {
            ExecuteFunction::ExecuteSelectAll => {
                self.FrameSelection().PrepareEditing();
                let context = MovementContext(self, std::cell::Cell::new(None));
                use interaction_default::core::editing::frame_selection_select_all::{
                    EnabledSelectAllForSource, FrameSelectionSelectAllContext,
                };
                let Some(facts) = context.Facts() else {
                    return Some(Err(EditorCommandError::RequiredPort(
                        "resident SelectAll geometry",
                    )));
                };
                let by = if source == Source::kDom {
                    SetSelectionBy::kSystem
                } else {
                    SetSelectionBy::kUser
                };
                if !EnabledSelectAllForSource(&facts, by) {
                    return Some(Ok(false));
                }
                Some(
                    self.FrameSelection()
                        .SelectAll(&context, by)
                        .map(|()| true)
                        .map_err(EditorCommandError::RequiredPort),
                )
            }
            ExecuteFunction::ExecuteUnselect => {
                self.FrameSelection().PrepareEditing();
                let Some(selection) = self
                    .FrameSelection()
                    .CreateVisibleSelection(self.FrameSelection().Current())
                else {
                    return Some(Ok(false));
                };
                if selection.anchor == selection.focus {
                    let editable = {
                        let owner = self.document.borrow();
                        selection.anchor.node_id.is_some_and(|node| {
                            foundation::editable::IsEditable(
                                &dom::persistent_document::DOMEditableFacts(owner.GetDocument()),
                                node,
                                Default::default(),
                            )
                        })
                    };
                    if !editable {
                        return Some(Ok(false));
                    }
                }
                self.FrameSelection().ClearSelection();
                Some(Ok(true))
            }
            _ => None,
        }
    }
    pub(super) fn ExecuteMovementCommand(
        &self,
        command: ExecuteFunction,
    ) -> Option<Result<bool, EditorCommandError>> {
        if let Some((alteration, direction)) =
            interaction_default::core::editing::commands::move_commands::PageMovement(command)
        {
            return Some((|| {
                self.FrameSelection().PrepareEditing();
                let Some(selection) = self.FrameSelection().Current() else {
                    return Ok(false);
                };
                let settings = self.FrameSelection().InteractionTextSettings().ok_or(
                    EditorCommandError::RequiredPort("page movement environment"),
                )?;
                let ER::Selection(selection) = self.EditorSelectionGeometry(EQ::Page {
                    selection,
                    directional: self.FrameSelection().SelectionIsDirectional(),
                    focused: self.KeyboardState().borrow().focused_node_id,
                    alteration,
                    direction,
                })?
                else {
                    return Err(EditorCommandError::RequiredPort("page selection movement"));
                };
                let Some(selection) = selection else {
                    return Ok(false);
                };
                self.FrameSelection().SetSelection(Some(selection),interaction_default::core::editing::commands::move_commands::PageMovementSelectionOptions(alteration,settings.editing_behavior));
                Ok(true)
            })());
        }
        let (alteration, direction, granularity, returns_modified) =
            interaction_default::core::editing::commands::move_commands::Movement(command)?;
        self.FrameSelection().PrepareEditing();
        if !self.FrameSelection().IsAvailable() {
            return Some(Ok(false));
        }
        let context = MovementContext(self, std::cell::Cell::new(None));
        let modified = self
            .FrameSelection()
            .Modify(&context, alteration, direction, granularity);
        Some(
            context
                .1
                .get()
                .map_or(Ok(!returns_modified || modified), |error| {
                    Err(EditorCommandError::RequiredPort(error))
                }),
        )
    }
}
