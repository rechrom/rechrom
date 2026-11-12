//! Call-scoped TypingCommand deletion ports. Geometry is queried against the
//! current resident owner; every elementary mutation is applied synchronously
//! through the existing StructuralCommand and CompositeEditCommand undo owner.
use super::super::super::page_frame_selection::{
    TypingDeletionGeometryQuery as GeometryQuery, TypingDeletionGeometryResult as GeometryResult,
};
use super::*;
use interaction_default::core::editing::{
    commands::{
        paragraph_editing_context::{EditResult, EditingAborted},
        typing_command::{kKillRing, kSmartDelete},
        typing_deletion::{
            BackwardDeletionGeometry, DeletionGeometryAction, TypingDeletionContext,
        },
    },
    editor::EditorKillRingContext,
    frame_selection::SelectionValue,
    text_granularity::TextGranularity,
    visible_selection::VisibleSelection,
};

impl SelectionValue for ImeTypingSelection {
    fn IsNone(&self) -> bool {
        self.0.anchor.node_id.is_none()
    }
}
impl Host<'_, '_, '_, '_> {
    fn DeletionGeometry(&self, query: GeometryQuery) -> EditResult<GeometryResult> {
        let settings = self
            .Frame()
            .InteractionTextSettings()
            .ok_or(EditingAborted)?;
        match self.Frame().CompositionText(Query::TypingDeletion {
            query,
            behavior: settings.editing_behavior,
            ui_locale: settings.ui_locale,
        }) {
            Some(ResultValue::TypingDeletion(result)) => Ok(result),
            _ => Err(EditingAborted),
        }
    }
    fn DeletionAction(
        &self,
        result: GeometryResult,
    ) -> EditResult<DeletionGeometryAction<ImeTypingSelection>> {
        match result {
            GeometryResult::Action(action) => Ok(match action {
                DeletionGeometryAction::None => DeletionGeometryAction::None,
                DeletionGeometryAction::Select(selection) => {
                    DeletionGeometryAction::Select(ImeTypingSelection(selection))
                }
                DeletionGeometryAction::Delete {
                    selection,
                    after_undo,
                } => DeletionGeometryAction::Delete {
                    selection: ImeTypingSelection(selection),
                    after_undo: ImeTypingSelection(after_undo),
                },
            }),
            _ => Err(EditingAborted),
        }
    }
}
impl EditorKillRingContext<ImeTypingSelection> for Host<'_, '_, '_, '_> {
    fn NeedsLayoutTreeUpdate(&self) -> bool {
        self.commands
            .document
            .borrow()
            .GetDocument()
            .Lifecycle()
            .GetState()
            < dom::document_lifecycle::LifecycleState::StyleClean
    }
    fn PlainText(&self, selection: &ImeTypingSelection) -> foundation::String {
        let range = self
            .Range(selection)
            .expect("valid native kill-ring selection");
        match self.Frame().CompositionText(Query::Text {
            range,
            original: false,
        }) {
            Some(ResultValue::Text(text)) => foundation::String::from_utf16(&text),
            _ => panic!("native TextIterator unavailable for kill ring"),
        }
    }
}
impl TypingDeletionContext<ImeTypingSelection> for Host<'_, '_, '_, '_> {
    type Node = u64;
    type ScrollAnchorSuppression = layoutng_assembly::layout_engine::SuppressScrollAnchorScope;
    fn HasFrame(&self) -> bool {
        self.Frame().IsAvailable()
    }
    fn IsSelectionDirectional(&self) -> bool {
        self.Frame().SelectionIsDirectional()
    }
    fn UpdateStyleAndLayoutForEditing(&self) {
        self.Frame().PrepareEditing();
    }
    fn RawDOMSelection(&self, selection: &ImeTypingSelection) -> VisibleSelection<u64> {
        structural::RawDeletionSelection(selection)
    }
    fn SelectionAnchorContainer(&self, selection: &ImeTypingSelection) -> Option<u64> {
        let position = selection.0.anchor;
        let node = position.node_id?;
        ReadNode(&self.commands.document, node, |document, _, index| {
            if matches!(
                position.anchor,
                layoutng_assembly::EditingPositionAnchorType::BeforeAnchor
                    | layoutng_assembly::EditingPositionAnchorType::AfterAnchor
            ) {
                document
                    .Node(index)
                    .Parent()
                    .map(|parent| document.Node(parent).Id())
            } else {
                Some(node)
            }
        })
        .flatten()
    }
    fn ShouldUndoOfDeleteSelectText(&self) -> bool {
        self.Frame()
            .InteractionTextSettings()
            .expect("native editing settings")
            .editing_behavior
            .ShouldUndoOfDeleteSelectText()
    }
    fn SuppressScrollAnchoring(&self, anchor: Option<&u64>) -> Self::ScrollAnchorSuppression {
        self.Frame().SuppressScrollAnchoringForNode(anchor.copied())
    }
    fn BreakOutOfEmptyMailBlockquotedParagraph(
        &self,
        command: &Rc<CompositeEditCommand<ImeTypingSelection>>,
    ) -> EditResult<bool> {
        structural::DeletionStructural(
            self,
            command,
            structural::DeletionStructuralOperation::MailBlockquote,
        )
    }
    fn BreakOutOfEmptyListItem(
        &self,
        command: &Rc<CompositeEditCommand<ImeTypingSelection>>,
    ) -> EditResult<bool> {
        structural::DeletionStructural(
            self,
            command,
            structural::DeletionStructuralOperation::EmptyList,
        )
    }
    fn MakeEditableRootEmpty(
        &self,
        command: &Rc<CompositeEditCommand<ImeTypingSelection>>,
    ) -> EditResult<bool> {
        structural::DeletionStructural(
            self,
            command,
            structural::DeletionStructuralOperation::EmptyRoot,
        )
    }
    fn BeginBackwardDeletionGeometry(
        &self,
        ending: &ImeTypingSelection,
        directional: bool,
        granularity: TextGranularity,
        kill_ring: bool,
    ) -> EditResult<BackwardDeletionGeometry<u64>> {
        match self.DeletionGeometry(GeometryQuery::BeginBackward {
            ending: ending.0,
            directional,
            granularity,
            kill_ring,
        })? {
            GeometryResult::Backward(geometry) => Ok(geometry),
            _ => Err(EditingAborted),
        }
    }
    fn BackwardDeletionNeedsEmptyRoot(
        &self,
        geometry: &BackwardDeletionGeometry<u64>,
    ) -> EditResult<bool> {
        match self.DeletionGeometry(GeometryQuery::NeedsEmptyRoot(geometry.clone()))? {
            GeometryResult::NeedsEmptyRoot(empty) => Ok(empty),
            _ => Err(EditingAborted),
        }
    }
    fn FinishBackwardDeletionGeometry(
        &self,
        geometry: BackwardDeletionGeometry<u64>,
        ending: &ImeTypingSelection,
        starting: &ImeTypingSelection,
        granularity: TextGranularity,
    ) -> EditResult<DeletionGeometryAction<ImeTypingSelection>> {
        self.DeletionAction(self.DeletionGeometry(GeometryQuery::FinishBackward {
            geometry,
            ending: ending.0,
            starting: starting.0,
            granularity,
        })?)
    }
    fn ForwardDeletionGeometry(
        &self,
        ending: &ImeTypingSelection,
        starting: &ImeTypingSelection,
        directional: bool,
        granularity: TextGranularity,
        kill_ring: bool,
    ) -> EditResult<DeletionGeometryAction<ImeTypingSelection>> {
        self.DeletionAction(self.DeletionGeometry(GeometryQuery::Forward {
            ending: ending.0,
            starting: starting.0,
            directional,
            granularity,
            kill_ring,
        })?)
    }
    fn AddToKillRing(&self, selection: &ImeTypingSelection) -> EditResult {
        self.commands
            .selection
            .editor
            .ime_typing
            .editor
            .AddToKillRing(selection, self)
            .map_err(|_| EditingAborted)
    }
    fn NotifyAccessibilityOfDeletionInTextField(&self, selection: &ImeTypingSelection) {
        self.Frame()
            .NotifyAccessibilityOfDeletionInTextField(selection.0);
    }
    fn DeleteSelectionIfRange(
        &self,
        selection: &ImeTypingSelection,
        command: &Rc<CompositeEditCommand<ImeTypingSelection>>,
        smart: bool,
        expand_special: bool,
    ) -> EditResult<ImeTypingSelection> {
        structural::DeleteWithExpansion(self, selection, command, smart, expand_special)
            .ok_or(EditingAborted)
    }
}

pub(in crate::adapter::page_input_method_controller) fn Delete(
    commands: &PageCompositionCommands<'_, '_, '_>,
    granularity: TextGranularity,
    forward: bool,
    kill_ring: bool,
) -> EditResult {
    let root = commands.selection.Root().ok_or(EditingAborted)?;
    let current = commands
        .selection
        .frame
        .as_ref()
        .and_then(|frame| frame.Current())
        .ok_or(EditingAborted)?;
    let host = Host { commands, root };
    let settings = host
        .Frame()
        .InteractionTextSettings()
        .ok_or(EditingAborted)?;
    let smart = settings.smart_insert_delete_enabled
        && host.Frame().SelectionControllerFlags().0 == TextGranularity::kWord;
    let options = if kill_ring { kKillRing } else { 0 } | if smart { kSmartDelete } else { 0 };
    commands
        .selection
        .editor
        .ime_typing
        .typing
        .DeleteKeyPressed(
            &host,
            &host,
            &ImeTypingSelection(current),
            options,
            granularity,
            forward,
        )
}

pub(in crate::adapter::page_input_method_controller) fn DeleteRange(
    commands: &PageCompositionCommands<'_, '_, '_>,
    forward: bool,
    kill_ring: bool,
) -> EditResult {
    let root = commands.selection.Root().ok_or(EditingAborted)?;
    let current = ImeTypingSelection(
        commands
            .selection
            .frame
            .as_ref()
            .and_then(|frame| frame.Current())
            .ok_or(EditingAborted)?,
    );
    let host = Host { commands, root };
    if kill_ring {
        TypingDeletionContext::AddToKillRing(&host, &current)?;
    }
    let settings = host
        .Frame()
        .InteractionTextSettings()
        .ok_or(EditingAborted)?;
    let smart = settings.smart_insert_delete_enabled
        && host.Frame().SelectionControllerFlags().0 == TextGranularity::kWord;
    let command = CompositeEditCommand::new(current.clone(), current.clone());
    let ending = structural::DeleteWithExpansion(&host, &current, &command, smart, true)
        .ok_or(EditingAborted)?;
    host.AppliedEditing(
        &ending,
        &command,
        if forward {
            InputType::kDeleteContentForward
        } else {
            InputType::kDeleteContentBackward
        },
        None,
        false,
        false,
    );
    Ok(())
}
