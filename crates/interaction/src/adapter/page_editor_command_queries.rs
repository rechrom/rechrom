//! EditorCommand queries use the descriptor and the live FrameSelection/Editor.
use super::{EditorCommandError, EventDispatcher};
use crate::adapter::page_frame_selection::{CompositionTextQuery as Q, CompositionTextResult as R};
use cssom::editing_style::{
    ApplyStyleOperation as Op, ApplyStyleResult as Out, EditingStylePropertyValue,
};
use foundation::{CSSPropertyID as P, CSSValueID as V};
use interaction_default::core::editing::{
    commands::{
        editor_command_table::*,
        paragraph_editing_context::{EditResult, EditingAborted},
        style_commands::{self, StyleCommandQueryContext},
    },
    frame_selection::SelectionRevealSource,
};

pub struct CommandQuery<'a, 'listener, 'frame>(pub &'a EventDispatcher<'listener, 'frame>);
impl CommandQuery<'_, '_, '_> {
    fn Typing(&self) -> Option<cssom::CSSInlineStyle> {
        self.0
            .EditingOwners()
            .ime_typing
            .editor
            .TypingStyle()
            .map(|style| style.borrow().declaration.clone())
    }
    fn Apply(&self, operation: Op) -> EditResult<Out> {
        self.0
            .FrameSelection()
            .ApplyEditingStyle(operation)
            .map_err(|_| EditingAborted)
    }
}
impl DocumentCommandContext for CommandQuery<'_, '_, '_> {
    fn DocumentNode(&self) -> u64 {
        self.0.document.borrow().GetDocument().RootHandle().Id()
    }
    fn DocumentElement(&self) -> Option<u64> {
        let owner = self.0.document.borrow();
        let d = owner.GetDocument();
        Some(d.Node(d.DocumentElement()?).Id())
    }
    fn IsEditable(&self, node: u64) -> bool {
        foundation::editable::IsEditable(
            &dom::persistent_document::DOMEditableFacts(self.0.document.borrow().GetDocument()),
            node,
            Default::default(),
        )
    }
    fn IsHTMLElement(&self, node: u64, tag: &str) -> bool {
        self.IsTag(node, tag)
    }
    fn CreateHTMLElement(&self, tag: &str) -> Result<u64, &'static str> {
        let node = self.0.document.borrow().GetDocument().NextNodeId();
        let name = foundation::QualifiedName::new(
            Default::default(),
            foundation::AtomicString::from_str(tag),
            foundation::AtomicString::from_str("http://www.w3.org/1999/xhtml"),
        );
        (self.0.EditorCommandMutationEmitter())(&dom::dom_mutation::DOMMutation {
            mutation_type: dom::dom_mutation::DOMMutationType::kCreateElement,
            target_node_id: self.DocumentNode(),
            child_node_id: node,
            qualified_name: Some(name),
            name: tag.into(),
            namespace_uri: "http://www.w3.org/1999/xhtml".into(),
            ..Default::default()
        });
        self.0
            .document
            .borrow()
            .GetDocument()
            .FindNodeById(node)
            .map(|_| node)
            .ok_or("Document command element mutation")
    }
    fn AppendChild(&self, parent: u64, child: u64) -> Result<(), &'static str> {
        (self.0.EditorCommandMutationEmitter())(&dom::dom_mutation::DOMMutation {
            mutation_type: dom::dom_mutation::DOMMutationType::kAppendChild,
            target_node_id: parent,
            child_node_id: child,
            ..Default::default()
        });
        if StyleCommandQueryContext::Parent(self, child) == Some(parent) {
            Ok(())
        } else {
            Err("Document command append mutation")
        }
    }
}
impl StyleCommandQueryContext for CommandQuery<'_, '_, '_> {
    fn IsTag(&self, node: u64, tag: &str) -> bool {
        let owner = self.0.document.borrow();
        let d = owner.GetDocument();
        d.FindNodeById(node)
            .is_some_and(|i| d.Node(i).IsHTMLElement(tag))
    }
    fn RootEditableElement(&self, node: u64) -> Option<u64> {
        foundation::editable::RootEditableElement(
            &dom::persistent_document::DOMEditableFacts(self.0.document.borrow().GetDocument()),
            node,
            Default::default(),
        )
    }
    fn HasActiveEditContext(&self) -> bool {
        self.0
            .input_method_state
            .as_ref()
            .is_some_and(|s| s.GetActiveEditContext().is_some())
    }
    fn ShouldStyleWithCss(&self) -> bool {
        self.0
            .EditingOwners()
            .ime_typing
            .editor
            .ShouldStyleWithCss()
    }
    fn ToggleBasedOnStart(&self) -> bool {
        self.0
            .FrameSelection()
            .InteractionTextSettings()
            .is_some_and(|s| {
                s.editing_behavior
                    .ShouldToggleStyleBasedOnStartOfSelection()
            })
    }
    fn Selection(&self) -> Option<(u64, u64, bool)> {
        let selection = self
            .0
            .FrameSelection()
            .CreateVisibleSelection(self.0.FrameSelection().Current())?;
        let mut start = dom::DOMBoundaryPoint {
            node_id: selection.anchor.node_id?,
            offset: selection.anchor.offset,
        };
        let mut end = dom::DOMBoundaryPoint {
            node_id: selection.focus.node_id?,
            offset: selection.focus.offset,
        };
        let owner = self.0.document.borrow();
        let d = owner.GetDocument();
        if d.CompareBoundaryPoints(start, end).ok()? == std::cmp::Ordering::Greater {
            std::mem::swap(&mut start, &mut end);
        }
        // VisibleSelectionStartAnchor shares FrameSelection's canonical visible
        // position normalization, including a range beginning at a text end.
        let start_node = self
            .0
            .FrameSelection()
            .VisibleSelectionStartAnchor(
                &crate::adapter::page_frame_selection::FrameSelectionValue(Some(selection)),
            )
            .unwrap_or(start.node_id);
        Some((start_node, end.node_id, selection.anchor == selection.focus))
    }
    fn NextNode(&self, node: u64) -> Option<u64> {
        let owner = self.0.document.borrow();
        let d = owner.GetDocument();
        let mut index = d.FindNodeById(node)?;
        if let Some(child) = d.Node(index).Children().first() {
            return Some(d.Node(*child).Id());
        }
        while let Some(parent) = d.Node(index).Parent() {
            let children = d.Node(parent).Children();
            if let Some(next) = children
                .iter()
                .position(|child| *child == index)
                .and_then(|i| children.get(i + 1))
            {
                return Some(d.Node(*next).Id());
            }
            index = parent;
        }
        None
    }
    fn Parent(&self, node: u64) -> Option<u64> {
        let owner = self.0.document.borrow();
        let d = owner.GetDocument();
        Some(d.Node(d.Node(d.FindNodeById(node)?).Parent()?).Id())
    }
    fn IsText(&self, node: u64) -> bool {
        let owner = self.0.document.borrow();
        let d = owner.GetDocument();
        d.FindNodeById(node)
            .is_some_and(|i| d.Node(i).Type() == dom::DOMNodeType::kText)
    }
    fn IsStyledElement(&self, node: u64) -> bool {
        let owner = self.0.document.borrow();
        let d = owner.GetDocument();
        d.FindNodeById(node).is_some_and(|i| {
            d.Node(i).Type() == dom::DOMNodeType::kElement && d.ResolvedStyleFor(i).is_some()
        })
    }
    fn HasEditableLayout(&self, node: u64) -> bool {
        self.0.FrameSelection().HasLayoutObject(node)
            && foundation::editable::IsEditable(
                &dom::persistent_document::DOMEditableFacts(self.0.document.borrow().GetDocument()),
                node,
                Default::default(),
            )
    }
    fn IsBlock(&self, node: u64) -> bool {
        matches!(self.0.FrameSelection().CompositionText(Q::StructuralNode(node)), Some(R::StructuralNode(facts)) if facts.block)
    }
    fn Present(&self, node: u64, property: P, value: V, typing: bool) -> EditResult<bool> {
        let Out::Declaration(declaration) = self.Apply(Op::Create {
            properties: vec![(property, EditingStylePropertyValue::Identifier(value))],
        })?
        else {
            return Err(EditingAborted);
        };
        match self.Apply(Op::CommandPresent {
            node_id: node,
            declaration,
            typing: typing.then(|| self.Typing()).flatten(),
        })? {
            Out::Bool(value) => Ok(value),
            _ => Err(EditingAborted),
        }
    }
    fn Value(&self, node: u64, property: P) -> EditResult<String> {
        match self.Apply(Op::CommandValue {
            node_id: node,
            property,
            typing: self.Typing(),
        })? {
            Out::Text(value) => Ok(value),
            _ => Err(EditingAborted),
        }
    }
    fn Bidi(&self, node: u64) -> EditResult<(V, Option<foundation::TextDirection>)> {
        match self.Apply(Op::Bidi { node_id: node })? {
            Out::Bidi {
                unicode_bidi,
                direction,
            } => Ok((unicode_bidi, direction)),
            _ => Err(EditingAborted),
        }
    }
    fn TypingDirection(&self) -> EditResult<Option<Option<foundation::TextDirection>>> {
        let Some(declaration) = self.Typing() else {
            return Ok(None);
        };
        match self.Apply(Op::StyleDirection { declaration })? {
            Out::Direction(value) => Ok(value),
            _ => Err(EditingAborted),
        }
    }
}
impl EditorCommandQueryContext for CommandQuery<'_, '_, '_> {
    fn PasteSupported(&self) -> bool {
        super::clipboard_commands::CanReadClipboard(self.0, EditorCommandSource::kDom)
    }
    fn CanReadClipboard(&self, source: EditorCommandSource) -> bool {
        super::clipboard_commands::CanReadClipboard(self.0, source)
    }
    fn CanWriteClipboard(&self, source: EditorCommandSource) -> bool {
        super::clipboard_commands::CanWriteClipboard(self.0, source)
    }
    fn ClipboardEnabled(
        &self,
        function: EnabledFunction,
        source: EditorCommandSource,
    ) -> Result<bool, &'static str> {
        super::clipboard_commands::IsEnabled(self.0, function, source)
    }
    fn HasActiveEditContext(&self) -> bool {
        StyleCommandQueryContext::HasActiveEditContext(self)
    }
    fn SelectionHasFocus(&self) -> bool {
        matches!(
            self.0.FrameSelection().CompositionText(Q::SelectionFocus {
                selection: self.0.FrameSelection().Current(),
                focused: self.0.KeyboardState().borrow().focused_node_id
            }),
            Some(R::SelectionFocus(true))
        )
    }
    fn IsCaret(&self) -> bool {
        self.0
            .FrameSelection()
            .CreateVisibleSelection(self.0.FrameSelection().Current())
            .is_some_and(|s| s.anchor == s.focus)
    }
    fn IsRange(&self) -> bool {
        self.0
            .FrameSelection()
            .CreateVisibleSelection(self.0.FrameSelection().Current())
            .is_some_and(|s| s.anchor != s.focus)
    }
    fn CanEdit(&self) -> bool {
        self.0
            .FrameSelection()
            .VisibleSelectionRootEditableElement()
            .is_some()
    }
    fn CanEditRichly(&self) -> bool {
        self.CanEdit() && crate::adapter::page_input_method_controller::PasteCanEditRichly(self.0)
    }
    fn HasMark(&self) -> bool {
        self.0
            .EditingOwners()
            .ime_typing
            .editor
            .Mark()
            .is_some()
    }
    fn CanUndo(&self) -> bool {
        self.0
            .EditingOwners()
            .ime_typing
            .editor
            .GetUndoStack()
            .CanUndo()
    }
    fn CanRedo(&self) -> bool {
        self.0
            .EditingOwners()
            .ime_typing
            .editor
            .GetUndoStack()
            .CanRedo()
    }
    fn SelectAllEnabled(&self, source: EditorCommandSource) -> Result<bool, &'static str> {
        match self.0.FrameSelection().CompositionText(Q::SelectAllFacts { selection: self.0.FrameSelection().Current(), focused: self.0.KeyboardState().borrow().focused_node_id }) {
            Some(R::SelectAllFacts(facts)) => Ok(interaction_default::core::editing::frame_selection_select_all::EnabledSelectAllForSource(&facts, if source == EditorCommandSource::kDom { interaction_default::core::editing::set_selection_options::SetSelectionBy::kSystem } else { interaction_default::core::editing::set_selection_options::SetSelectionBy::kUser })),
            _ => Err("resident SelectAll geometry"),
        }
    }
}
impl EventDispatcher<'_, '_> {
    pub(crate) fn TidyUpHtmlStructure(&self) -> Result<(), EditorCommandError> {
        self.FrameSelection().PrepareEditing();
        TidyUpHtmlStructure(&CommandQuery(self)).map_err(EditorCommandError::RequiredPort)
    }
    pub(crate) fn EditorCommandSupported(
        &self,
        command: EditingCommandType,
        source: EditorCommandSource,
    ) -> bool {
        command
            .Descriptor()
            .is_some_and(|d| d.IsSupported(&CommandQuery(self), source))
    }
    pub(crate) fn EditorCommandEnabled(
        &self,
        command: EditingCommandType,
        source: EditorCommandSource,
    ) -> Result<bool, EditorCommandError> {
        self.FrameSelection().PrepareEditing();
        command.Descriptor().map_or(Ok(false), |d| {
            d.IsEnabled(&CommandQuery(self), source)
                .map_err(EditorCommandError::RequiredPort)
        })
    }
    pub(crate) fn EditorCommandState(
        &self,
        command: EditingCommandType,
        source: EditorCommandSource,
    ) -> Result<EditingTriState, EditorCommandError> {
        if !self.EditorCommandSupported(command, source) {
            return Ok(EditingTriState::kFalse);
        }
        self.FrameSelection().PrepareEditing();
        let state = command.Descriptor().unwrap().state;
        if state == StateFunction::StateNone {
            return Ok(EditingTriState::kFalse);
        }
        style_commands::State(&CommandQuery(self), state)
            .unwrap_or(Err(EditingAborted))
            .map_err(EditorCommandError::Aborted)
    }
    pub(crate) fn EditorCommandValue(
        &self,
        command: EditingCommandType,
        source: EditorCommandSource,
    ) -> Result<String, EditorCommandError> {
        if !self.EditorCommandSupported(command, source) {
            return Ok(String::new());
        }
        self.FrameSelection().PrepareEditing();
        let descriptor = command.Descriptor().unwrap();
        if let Some(value) = style_commands::Value(&CommandQuery(self), descriptor.value) {
            return value.map_err(EditorCommandError::Aborted);
        }
        Ok(match descriptor.value {
            ValueFunction::ValueEmpty => String::new(),
            ValueFunction::ValueFormatBlock => {
                style_commands::FormatBlockValue(&CommandQuery(self))
            }
            ValueFunction::ValueStateOrNull if descriptor.state == StateFunction::StateNone => {
                String::new()
            }
            ValueFunction::ValueStateOrNull => {
                (self.EditorCommandState(command, source)? == EditingTriState::kTrue).to_string()
            }
            ValueFunction::ValueDefaultParagraphSeparator => match self
                .EditingOwners()
                .ime_typing
                .editor
                .DefaultParagraphSeparator()
            {
                interaction_default::core::editing::editor::EditorParagraphSeparator::kIsDiv => {
                    "div".into()
                }
                _ => "p".into(),
            },
            _ => {
                return Err(EditorCommandError::RequiredPort(
                    "EditorCommand value owner",
                ))
            }
        })
    }
}
