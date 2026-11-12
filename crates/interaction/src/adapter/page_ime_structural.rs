//! Call-scoped native DOM/visible-unit ports for source structural commands.
//! Simple commands retain this same live Document/FrameSelection and emitter
//! through CompositeEditCommand's existing undo owner.
use super::*;
#[path = "page_list_indent.rs"]
mod list_indent;
#[path = "page_replace_selection.rs"]
mod replacement;
#[path = "page_apply_style.rs"]
mod replacement_apply_style;
#[path = "page_replacement_style.rs"]
mod replacement_style;
pub(super) use list_indent::ApplyListIndent;
#[path = "page_link_format.rs"]
mod link_format;
#[path = "page_style_command_context.rs"]
mod style_commands;
use foundation::QualifiedName;
use interaction_default::core::editing::{
    commands::{
        delete_selection_command::DeleteSelectionCommand,
        insert_line_break_command::InsertLineBreak,
        insert_paragraph_separator_command::InsertParagraphSeparatorCommand,
        paragraph_editing_context::{EditResult, EditingAborted},
        simple_edit_commands::{
            AppendNodeCommand, DeleteFromTextNodeCommand, InsertIntoTextNodeCommand,
            InsertNodeBeforeCommand, RemoveNodeCommand, SetCharacterDataCommand,
            SetNodeAttributeCommand, SimpleEditCommand, SimpleEditingDOM, SplitElementCommand,
            SplitTextNodeCommand,
        },
        structural_editing_context::{
            StructuralBooleanQuery, StructuralEditingContext, StructuralNodeFacts,
            StructuralPositionQuery,
        },
    },
    position::{Position, PositionAnchorType, PositionDOM, PositionTree},
};
use layoutng_assembly::{EditingPositionAnchorType, EditingPositionWithAffinity};
pub(super) use link_format::ApplyLinkFormat;
pub(super) use replacement::ApplyReplacement;
pub(super) use style_commands::ExecuteStyle;

#[derive(Clone)]
struct CommandDOM {
    document: InteractionDocument,
    emit: InteractionDOMMutationEmitter,
    frame: std::rc::Weak<PageFrameSelection>,
    selections: Rc<SelectionState>,
    document_identity: u128,
    editor: Rc<NativeEditorState>,
    markers: Option<super::super::super::page_frame_selection::CompositionMarkerOwner>,
}
#[derive(Clone)]
enum CommandAttributeValue {
    Text(Rc<[u16]>),
    Inline(cssom::CSSInlineStyle),
}
impl CommandDOM {
    fn Alive(&self) -> bool {
        let owner = self.document.borrow();
        owner.GetDocument().IsActive()
            && owner.GetDocument().RootHandle().Identity() == self.document_identity
    }
    fn Query(&self, query: Query) -> Option<ResultValue> {
        if !self.Alive() {
            return None;
        }
        self.frame.upgrade()?.CompositionText(query)
    }
    fn Read<T>(
        &self,
        node: u64,
        read: impl FnOnce(&dom::Document, &dom::persistent_document::DOMNode, usize) -> T,
    ) -> Option<T> {
        ReadNode(&self.document, node, read)
    }
    fn Mutation(&self, mutation: DOMMutation) -> bool {
        if !self.Alive() {
            return false;
        }
        (self.emit)(&mutation);
        self.Alive()
    }
    fn Native(position: &Position<u64>) -> Option<EditingPosition> {
        Some(EditingPosition::WithAnchor(
            *position.AnchorNode()?,
            position.OffsetInAnchor(),
            match position.AnchorType() {
                PositionAnchorType::OffsetInAnchor => EditingPositionAnchorType::OffsetInAnchor,
                PositionAnchorType::BeforeAnchor => EditingPositionAnchorType::BeforeAnchor,
                PositionAnchorType::AfterAnchor => EditingPositionAnchorType::AfterAnchor,
                PositionAnchorType::AfterChildren => EditingPositionAnchorType::AfterChildren,
            },
        ))
    }
    fn Source(position: EditingPosition) -> Position<u64> {
        let Some(node) = position.node_id else {
            return Position::default();
        };
        match position.anchor {
            EditingPositionAnchorType::OffsetInAnchor => Position::new(node, position.offset),
            EditingPositionAnchorType::BeforeAnchor => Position::BeforeNode(node),
            EditingPositionAnchorType::AfterAnchor => Position::AfterNode(node),
            EditingPositionAnchorType::AfterChildren => Position::AfterChildren(node),
        }
    }
    fn FreshId(&self) -> u64 {
        self.document.borrow().GetDocument().NextNodeId()
    }
    fn DocumentId(&self) -> u64 {
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        document.Node(document.Root()).Id()
    }
    fn Create(&self, tag: &str) -> u64 {
        self.CreateQualified(&QualifiedName::new(
            Default::default(),
            foundation::AtomicString::from_str(tag),
            foundation::AtomicString::from_str("http://www.w3.org/1999/xhtml"),
        ))
    }
    fn CreateQualified(&self, tag: &QualifiedName) -> u64 {
        let node = self.FreshId();
        self.Mutation(DOMMutation {
            mutation_type: DOMMutationType::kCreateElement,
            target_node_id: self.DocumentId(),
            child_node_id: node,
            qualified_name: Some(tag.clone()),
            name: tag.LocalName().Utf8(),
            namespace_uri: tag.NamespaceURI().Utf8(),
            ..Default::default()
        });
        node
    }
}
impl PositionTree for CommandDOM {
    type Node = u64;
    fn Parent(&self, node: &u64) -> Option<u64> {
        self.Read(*node, |d, n, _| n.Parent().map(|i| d.Node(i).Id()))
            .flatten()
    }
    fn ChildAt(&self, node: &u64, index: usize) -> Option<u64> {
        self.Read(*node, |d, n, _| {
            n.Children().get(index).map(|i| d.Node(*i).Id())
        })
        .flatten()
    }
    fn ChildCount(&self, node: &u64) -> usize {
        self.Read(*node, |_, n, _| n.Children().len()).unwrap_or(0)
    }
    fn NodeIndex(&self, node: &u64) -> usize {
        let Some(parent) = PositionTree::Parent(self, node) else {
            return 0;
        };
        self.Read(parent, |d, n, _| {
            n.Children()
                .iter()
                .position(|i| d.Node(*i).Id() == *node)
                .unwrap_or(0)
        })
        .unwrap_or(0)
    }
    fn PreviousSibling(&self, node: &u64) -> Option<u64> {
        PositionTree::ChildAt(
            self,
            &PositionTree::Parent(self, node)?,
            PositionTree::NodeIndex(self, node).checked_sub(1)?,
        )
    }
    fn NextSibling(&self, node: &u64) -> Option<u64> {
        PositionTree::ChildAt(
            self,
            &PositionTree::Parent(self, node)?,
            PositionTree::NodeIndex(self, node) + 1,
        )
    }
    fn IsText(&self, node: &u64) -> bool {
        self.Read(*node, |_, n, _| n.Type() == DOMNodeType::kText)
            .unwrap_or(false)
    }
    fn CharacterDataLength(&self, node: &u64) -> Option<usize> {
        self.Read(*node, |d, n, i| {
            matches!(
                n.Type(),
                DOMNodeType::kText | DOMNodeType::kComment | DOMNodeType::kProcessingInstruction
            )
            .then(|| d.CharacterDataUTF16(i).len())
        })
        .flatten()
    }
}
impl PositionDOM for CommandDOM {
    fn EditingIgnoresContent(&self, node: &u64) -> bool {
        self.Read(*node, |d, _, i| d.EditingIgnoresContent(i))
            .unwrap_or(false)
    }
    fn IsDisplayInsideTable(&self, node: &u64) -> bool {
        self.Read(*node, |d, _, i| d.IsHTMLTableElementForEditing(i))
            .unwrap_or(false)
    }
    fn IsConnected(&self, node: &u64) -> bool {
        self.Read(*node, |d, _, i| d.IsConnectedToDocument(i))
            .unwrap_or(false)
    }
    fn DocumentIdentity(&self, node: &u64) -> u128 {
        self.Read(*node, |d, _, i| d.NodeDocumentHandle(i).Identity())
            .unwrap_or(0)
    }
    fn LastDOMOffsetForEditing(&self, node: &u64) -> usize {
        self.Read(*node, |d, _, i| d.LastOffsetForEditing(i))
            .unwrap_or(0)
    }
}
impl SimpleEditingDOM for CommandDOM {
    type Node = u64;
    type AttributeValue = CommandAttributeValue;
    fn Parent(&self, node: &u64) -> Option<u64> {
        PositionTree::Parent(self, node)
    }
    fn NextSibling(&self, node: &u64) -> Option<u64> {
        PositionTree::NextSibling(self, node)
    }
    fn FirstChild(&self, node: &u64) -> Option<u64> {
        PositionTree::ChildAt(self, node, 0)
    }
    fn LastChild(&self, node: &u64) -> Option<u64> {
        PositionTree::ChildAt(
            self,
            node,
            PositionTree::ChildCount(self, node).checked_sub(1)?,
        )
    }
    fn Children(&self, node: &u64) -> Vec<u64> {
        (0..PositionTree::ChildCount(self, node))
            .filter_map(|i| PositionTree::ChildAt(self, node, i))
            .collect()
    }
    fn IsEditable(&self, node: &u64) -> bool {
        let owner = self.document.borrow();
        foundation::editable::IsEditable(
            &dom::persistent_document::DOMEditableFacts(owner.GetDocument()),
            *node,
            Default::default(),
        )
    }
    fn InActiveDocument(&self, node: &u64) -> bool {
        PositionDOM::IsConnected(self, node)
    }
    fn HasFrame(&self, _node: &u64) -> bool {
        self.Alive() && self.frame.upgrade().is_some()
    }
    fn HasDocumentElement(&self, _node: &u64) -> bool {
        self.document
            .borrow()
            .GetDocument()
            .DocumentElement()
            .is_some()
    }
    fn IsPlaceholderBreak(&self, node: &u64) -> bool {
        self.Read(*node, |_, n, _| n.IsHTMLElement("br"))
            .unwrap_or(false)
    }
    fn TextAreaEmptyPlaceholderBreakEnabled(&self) -> bool {
        foundation::RuntimeEnabledFeatures::TextAreaEmptyPlaceholderBreakEnabled()
    }
    fn NonEmptyBlockquotesOnOutdentingEnabled(&self) -> bool {
        // runtime_enabled_features.json5:4310-4311 (stable).
        true
    }
    fn UpdateStyleAndLayoutTree(&self) {
        if let Some(frame) = self.frame.upgrade() {
            frame.PrepareEditing();
        }
    }
    fn UpdateStyleAndLayout(&self) {
        if let Some(frame) = self.frame.upgrade() {
            frame.PrepareEditing();
        }
    }
    fn AppendChild(&self, parent: &u64, node: &u64) -> bool {
        self.Mutation(DOMMutation {
            mutation_type: DOMMutationType::kAppendChild,
            target_node_id: *parent,
            child_node_id: *node,
            ..Default::default()
        })
    }
    fn InsertBefore(&self, parent: &u64, node: &u64, reference: Option<&u64>) -> bool {
        self.Mutation(DOMMutation {
            mutation_type: DOMMutationType::kInsertBefore,
            target_node_id: *parent,
            child_node_id: *node,
            before_node_id: reference.copied().unwrap_or(0),
            ..Default::default()
        })
    }
    fn Remove(&self, node: &u64) -> bool {
        let Some(parent) = PositionTree::Parent(self, node) else {
            return true;
        };
        self.Mutation(DOMMutation {
            mutation_type: DOMMutationType::kRemoveChild,
            target_node_id: parent,
            child_node_id: *node,
            ..Default::default()
        })
    }
    fn CreateText(&self, text: &[u16]) -> u64 {
        let node = self.FreshId();
        self.Mutation(DOMMutation {
            mutation_type: DOMMutationType::kCreateText,
            target_node_id: self.DocumentId(),
            child_node_id: node,
            utf16_value: Some(Rc::from(text)),
            ..Default::default()
        });
        node
    }
    fn CreateSpan(&self) -> u64 {
        self.Create("span")
    }
    fn CloneWithoutChildren(&self, node: &u64) -> u64 {
        let clone = self.FreshId();
        self.Mutation(DOMMutation {
            mutation_type: DOMMutationType::kCloneNode,
            target_node_id: *node,
            child_node_id: clone,
            bool_value: false,
            ..Default::default()
        });
        clone
    }
    fn Text(&self, node: &u64) -> Rc<[u16]> {
        self.Read(*node, |d, _, i| d.CharacterDataUTF16(i).to_vec().into())
            .unwrap_or_else(|| Rc::from([]))
    }
    fn ReplaceText(&self, node: &u64, offset: usize, count: usize, text: &[u16]) -> bool {
        self.Mutation(DOMMutation {
            mutation_type: DOMMutationType::kReplaceCharacterData,
            target_node_id: *node,
            offset,
            count,
            utf16_value: Some(Rc::from(text)),
            ..Default::default()
        })
    }
    fn MoveMarkers(&self, from: &u64, length: usize, to: &u64) {
        if let Some(markers) = &self.markers {
            let (ranges, invalidated) = markers.MoveMarkers(*from, length, *to);
            self.selections.SetDocumentCompositionMarkers(ranges);
            if let Some(frame) = self.frame.upgrade() {
                for node in invalidated {
                    frame.InvalidateCompositionMarkerPaint(node);
                }
            }
        }
    }
    fn Attribute(&self, node: &u64, name: &QualifiedName) -> Option<Self::AttributeValue> {
        self.Read(*node, |_, n, _| {
            n.AttributeValueUTF16(&name.LocalName().Utf8())
                .map(|value| {
                    if name.LocalName().EqualsUtf8("style") {
                        CommandAttributeValue::Inline(n.InlineStyle())
                    } else {
                        CommandAttributeValue::Text(value)
                    }
                })
        })
        .flatten()
    }
    fn SetAttribute(&self, node: &u64, name: &QualifiedName, value: Option<&Self::AttributeValue>) {
        self.Mutation(DOMMutation {
            mutation_type: if value.is_some() {
                DOMMutationType::kSetAttribute
            } else {
                DOMMutationType::kRemoveAttribute
            },
            target_node_id: *node,
            qualified_name: Some(name.clone()),
            // DOMMutation retains the QualifiedName as the authoritative
            // identity and the scalar fields as mutation/lifecycle metadata.
            // Keep both projections synchronized just like Element's
            // QualifiedName-backed AttributeModificationParams.
            name: name.LocalName().Utf8(),
            prefix: name.Prefix().Utf8(),
            namespace_uri: name.NamespaceURI().Utf8(),
            utf16_value: value.map(|value| match value {
                CommandAttributeValue::Text(text) => text.clone(),
                CommandAttributeValue::Inline(style) => Rc::from(style.SourceTextUTF16()),
            }),
            inline_style: value.and_then(|value| match value {
                CommandAttributeValue::Inline(style) => Some(style.clone()),
                _ => None,
            }),
            ..Default::default()
        });
    }
    fn ShouldEchoPassword(&self, _behavior: PasswordEchoBehavior) -> bool {
        false
    }
    fn MomentarilyRevealLastTypedCharacterIfSecure(&self, _node: &u64, _offset: usize) {
        unreachable!("generic editable text has no password echo capability")
    }
}

struct StructuralCommand {
    dom: Rc<CommandDOM>,
    composite: Rc<CompositeEditCommand<ImeTypingSelection>>,
}
impl StructuralCommand {
    fn Apply(&self, command: Rc<dyn SimpleEditCommand>) -> EditResult {
        self.composite.ApplyCommandToComposite(command)
    }
}
impl PositionTree for StructuralCommand {
    type Node = u64;
    fn Parent(&self, n: &u64) -> Option<u64> {
        PositionTree::Parent(&*self.dom, n)
    }
    fn ChildAt(&self, n: &u64, i: usize) -> Option<u64> {
        PositionTree::ChildAt(&*self.dom, n, i)
    }
    fn ChildCount(&self, n: &u64) -> usize {
        PositionTree::ChildCount(&*self.dom, n)
    }
    fn NodeIndex(&self, n: &u64) -> usize {
        PositionTree::NodeIndex(&*self.dom, n)
    }
    fn PreviousSibling(&self, n: &u64) -> Option<u64> {
        PositionTree::PreviousSibling(&*self.dom, n)
    }
    fn NextSibling(&self, n: &u64) -> Option<u64> {
        PositionTree::NextSibling(&*self.dom, n)
    }
    fn IsText(&self, n: &u64) -> bool {
        PositionTree::IsText(&*self.dom, n)
    }
    fn CharacterDataLength(&self, n: &u64) -> Option<usize> {
        PositionTree::CharacterDataLength(&*self.dom, n)
    }
}
impl PositionDOM for StructuralCommand {
    fn EditingIgnoresContent(&self, n: &u64) -> bool {
        self.dom.EditingIgnoresContent(n)
    }
    fn IsDisplayInsideTable(&self, n: &u64) -> bool {
        self.dom.IsDisplayInsideTable(n)
    }
    fn IsConnected(&self, n: &u64) -> bool {
        PositionDOM::IsConnected(&*self.dom, n)
    }
    fn DocumentIdentity(&self, n: &u64) -> u128 {
        self.dom.DocumentIdentity(n)
    }
    fn LastDOMOffsetForEditing(&self, n: &u64) -> usize {
        self.dom.LastDOMOffsetForEditing(n)
    }
}
impl StructuralEditingContext for StructuralCommand {
    type Tracked = DOMRangeHandle;
    type Style = NativeEditingStyle;
    fn CaptureStyle(&self, position: &Position<u64>, in_effect: bool) -> EditResult<Self::Style> {
        use cssom::editing_style::EditingStyleOperation as Op;
        let frame = self.dom.frame.upgrade().ok_or(EditingAborted)?;
        let style = frame
            .EditingStyle(Op::Capture {
                node_id: position.ComputeContainerNode(self).ok_or(EditingAborted)?,
                in_effect,
            })
            .map_err(|_| EditingAborted)?;
        let mut style = NativeEditingStyle::new(style);
        if in_effect {
            let mut current = position.AnchorNode().copied();
            while let Some(node) = current {
                if let Some((name, tag)) = self
                    .dom
                    .Read(node, |_, node, _| {
                        (node.Type() == DOMNodeType::kElement
                            && node.Namespace() == dom::DOMNamespace::kHTML)
                            .then(|| (node.Name().to_owned(), node.TagQName().clone()))
                    })
                    .flatten()
                {
                    if let Some((property, value)) = NativeEditingStyle::HtmlEquivalent(&name) {
                        style.RecordOriginalHtmlEquivalentTag(property, value, tag);
                    }
                }
                if !self.Editable(node) {
                    break;
                }
                current = self.Parent(&node);
            }
        }
        Ok(style)
    }
    fn MergeTypingStyle(&self, mut style: Self::Style) -> EditResult<Self::Style> {
        use cssom::editing_style::EditingStyleOperation as Op;
        let frame = self.dom.frame.upgrade().ok_or(EditingAborted)?;
        let typing = self
            .dom
            .editor
            .TypingStyle()
            .map(|style| style.borrow().clone());
        if let Some(typing) = typing {
            style.declaration = frame
                .EditingStyle(Op::Merge {
                    base: style.declaration,
                    override_style: typing.declaration,
                })
                .map_err(|_| EditingAborted)?;
            for (property, value, tag) in typing.original_html_equivalent_tags {
                style.RecordOriginalHtmlEquivalentTag(property, value, tag);
            }
        }
        Ok(style)
    }
    fn PrepareStyle(
        &self,
        style: &Self::Style,
        position: &Position<u64>,
    ) -> EditResult<Option<Self::Style>> {
        let mut result = style.clone();
        result.declaration = self
            .dom
            .frame
            .upgrade()
            .ok_or(EditingAborted)?
            .EditingStyle(cssom::editing_style::EditingStyleOperation::Prepare {
                node_id: position.ComputeContainerNode(self).ok_or(EditingAborted)?,
                declaration: style.declaration.clone(),
            })
            .map_err(|_| EditingAborted)?;
        Ok((!result.declaration.SourceTextValue().empty()).then_some(result))
    }
    fn RemoveStyleAddedByElement(
        &self,
        style: &Self::Style,
        element: u64,
    ) -> EditResult<Self::Style> {
        let mut result = style.clone();
        result.declaration = self
            .dom
            .frame
            .upgrade()
            .ok_or(EditingAborted)?
            .EditingStyle(
                cssom::editing_style::EditingStyleOperation::RemoveStyleAddedByElement {
                    node_id: element,
                    declaration: style.declaration.clone(),
                },
            )
            .map_err(|_| EditingAborted)?;
        Ok(result)
    }
    fn SetTypingStyle(&self, style: Option<Self::Style>) {
        self.dom
            .editor
            .SetTypingStyle(style.map(|s| Rc::new(RefCell::new(s))));
    }
    fn ApplyStyle(
        &self,
        style: &Self::Style,
        position: &Position<u64>,
    ) -> EditResult<Position<u64>> {
        use cssom::editing_style::{ApplyStyleOperation, ApplyStyleResult};
        let application = self
            .dom
            .frame
            .upgrade()
            .ok_or(EditingAborted)?
            .ApplyEditingStyle(ApplyStyleOperation::Change {
                node_id: position.ComputeContainerNode(self).ok_or(EditingAborted)?,
                declaration: style.declaration.clone(),
                originals: style.original_html_equivalent_tags.clone(),
                css: self.dom.editor.ShouldStyleWithCss(),
            })
            .map_err(|_| EditingAborted)?;
        let ApplyStyleResult::Change(application) = application else {
            return Err(EditingAborted);
        };
        let mut insertion = position.clone();
        for tag in application.tags {
            let node = self.dom.CreateQualified(&tag);
            if self.dom.Read(node, |_, _, _| ()).is_none() {
                return Err(EditingAborted);
            }
            interaction_default::core::editing::commands::composite_edit_command::InsertNodeAt(
                self, node, &insertion,
            )?;
            insertion = Position::FirstPositionInNode(node);
        }
        if !application.font_attributes.is_empty() {
            let font = self.CreateElement("font")?;
            for (name, value) in application.font_attributes {
                self.Apply(Rc::new(SetNodeAttributeCommand::new(
                    self.dom.clone(),
                    font,
                    name,
                    Some(CommandAttributeValue::Text(value)),
                )))?;
            }
            interaction_default::core::editing::commands::composite_edit_command::InsertNodeAt(
                self, font, &insertion,
            )?;
            insertion = Position::FirstPositionInNode(font);
        }
        if !application.declaration.SourceTextValue().empty() {
            let span = self.CreateElement("span")?;
            let name = QualifiedName::new(
                Default::default(),
                foundation::AtomicString::from_str("style"),
                Default::default(),
            );
            self.Apply(Rc::new(SetNodeAttributeCommand::new(
                self.dom.clone(),
                span,
                name,
                Some(CommandAttributeValue::Inline(application.declaration)),
            )))?;
            interaction_default::core::editing::commands::composite_edit_command::InsertNodeAt(
                self, span, &insertion,
            )?;
            insertion = Position::FirstPositionInNode(span);
        }
        self.SetTypingStyle(Some(style.clone()));
        Ok(insertion)
    }
    fn AttributeEquals(&self, node: u64, name: &str, value: &str) -> bool {
        self.dom
            .Read(node, |_, node, _| {
                node.FindAttribute(name)
                    .is_some_and(|attribute| attribute.value == value)
            })
            .unwrap_or(false)
    }
    fn IsElement(&self, node: u64) -> bool {
        self.dom
            .Read(node, |_, node, _| node.Type() == DOMNodeType::kElement)
            .unwrap_or(false)
    }
    fn UpdateStyleAndLayout(&self) {
        SimpleEditingDOM::UpdateStyleAndLayout(&*self.dom)
    }
    fn PositionBoolean(
        &self,
        position: &Position<u64>,
        other: Option<&Position<u64>>,
        query: StructuralBooleanQuery,
    ) -> bool {
        let Some(position) = CommandDOM::Native(position) else {
            return false;
        };
        matches!(
            self.dom.Query(Query::StructuralBoolean {
                position,
                other: other.and_then(CommandDOM::Native),
                query
            }),
            Some(ResultValue::StructuralBoolean(true))
        )
    }
    fn TextCodeUnit(&self, node: u64, offset: usize) -> Option<u16> {
        self.dom
            .Read(node, |document, _, index| {
                document.CharacterDataUTF16(index).get(offset).copied()
            })
            .flatten()
    }
    fn OriginalPlainText(
        &self,
        start: &Position<u64>,
        end: &Position<u64>,
    ) -> EditResult<Vec<u16>> {
        let start = start.ToOffsetInAnchor(self);
        let end = end.ToOffsetInAnchor(self);
        match self.dom.Query(Query::Text {
            range: DOMStaticRange {
                start: DOMBoundaryPoint {
                    node_id: *start.AnchorNode().ok_or(EditingAborted)?,
                    offset: start.OffsetInAnchor(),
                },
                end: DOMBoundaryPoint {
                    node_id: *end.AnchorNode().ok_or(EditingAborted)?,
                    offset: end.OffsetInAnchor(),
                },
            },
            original: true,
        }) {
            Some(ResultValue::Text(text)) => Ok(text),
            _ => Err(EditingAborted),
        }
    }
    fn ReplaceText(&self, node: u64, offset: usize, count: usize, text: &[u16]) -> EditResult {
        self.Apply(Rc::new(SetCharacterDataCommand::new(
            self.dom.clone(),
            node,
            offset,
            count,
            Rc::from(text),
            PasswordEchoBehavior::kDoNotEcho,
        )))
    }
    fn SetStartingSelection(&self, start: &Position<u64>, end: &Position<u64>) {
        let selection = self.composite.StartingSelection().0;
        let forward = self
            .Compare(
                &CommandDOM::Source(selection.anchor),
                &CommandDOM::Source(selection.focus),
            )
            .is_none_or(|order| order != std::cmp::Ordering::Greater);
        let (Some(start), Some(end)) = (CommandDOM::Native(start), CommandDOM::Native(end)) else {
            return;
        };
        if let Some(ResultValue::CanonicalSelection(selection)) =
            self.dom.Query(Query::CanonicalSelection(DOMSelection {
                anchor: if forward { start } else { end },
                focus: if forward { end } else { start },
                affinity: EditingTextAffinity::Downstream,
            }))
        {
            self.composite
                .SetStartingSelection(ImeTypingSelection(selection));
        }
    }
    fn QueryPosition(&self, p: &Position<u64>, query: StructuralPositionQuery) -> Position<u64> {
        let Some(position) = CommandDOM::Native(p) else {
            return Position::default();
        };
        match self.dom.Query(Query::StructuralPosition {
            position: EditingPositionWithAffinity {
                position,
                affinity: EditingTextAffinity::Downstream,
            },
            query,
        }) {
            Some(ResultValue::StructuralPosition(p)) => CommandDOM::Source(p.position),
            _ => Position::default(),
        }
    }
    fn NodeFacts(&self, node: u64) -> StructuralNodeFacts {
        match self.dom.Query(Query::StructuralNode(node)) {
            Some(ResultValue::StructuralNode(facts)) => facts,
            _ => Default::default(),
        }
    }
    fn EnclosingBlock(&self, p: &Position<u64>) -> Option<u64> {
        match self
            .dom
            .Query(Query::StructuralBlock(CommandDOM::Native(p)?))?
        {
            ResultValue::StructuralBlock(node) => node,
            _ => None,
        }
    }
    fn RootEditable(&self, node: u64) -> Option<u64> {
        let owner = self.dom.document.borrow();
        foundation::editable::RootEditableElement(
            &dom::persistent_document::DOMEditableFacts(owner.GetDocument()),
            node,
            Default::default(),
        )
    }
    fn Editable(&self, node: u64) -> bool {
        SimpleEditingDOM::IsEditable(&*self.dom, &node)
    }
    fn Tag(&self, node: u64, tag: &str) -> bool {
        self.dom
            .Read(node, |_, n, _| n.IsHTMLElement(tag))
            .unwrap_or(false)
    }
    fn HasAttributes(&self, node: u64) -> bool {
        self.dom
            .Read(node, |_, n, _| !n.Attributes().is_empty())
            .unwrap_or(false)
    }
    fn Compare(&self, a: &Position<u64>, b: &Position<u64>) -> Option<std::cmp::Ordering> {
        let a = a.ToOffsetInAnchor(self);
        let b = b.ToOffsetInAnchor(self);
        self.dom
            .document
            .borrow()
            .GetDocument()
            .CompareBoundaryPoints(
                DOMBoundaryPoint {
                    node_id: *a.AnchorNode()?,
                    offset: a.OffsetInAnchor(),
                },
                DOMBoundaryPoint {
                    node_id: *b.AnchorNode()?,
                    offset: b.OffsetInAnchor(),
                },
            )
            .ok()
    }
    fn Track(&self, p: &Position<u64>) -> EditResult<Self::Tracked> {
        let p = p.ToOffsetInAnchor(self);
        let point = DOMBoundaryPoint {
            node_id: *p.AnchorNode().ok_or(EditingAborted)?,
            offset: p.OffsetInAnchor(),
        };
        self.dom
            .document
            .borrow_mut()
            .GetDocumentMut()
            .CreateRange(point, point)
            .map_err(|_| EditingAborted)
    }
    fn Tracked(&self, p: &Self::Tracked) -> Position<u64> {
        self.dom
            .document
            .borrow()
            .GetDocument()
            .Range(p)
            .map_or_else(Position::default, |range| {
                Position::new(range.start.node_id, range.start.offset)
            })
    }
    fn CreateDefaultParagraphElement(&self) -> EditResult<u64> {
        self.CreateElement(
            interaction_default::core::editing::editor::DefaultParagraphElementTag(
                self.dom.editor.DefaultParagraphSeparator(),
            ),
        )
    }
    fn CreateElement(&self, tag: &str) -> EditResult<u64> {
        let node = self.dom.Create(tag);
        self.dom.Read(node, |_, _, _| node).ok_or(EditingAborted)
    }
    fn CloneWithoutChildren(&self, node: u64) -> EditResult<u64> {
        Ok(SimpleEditingDOM::CloneWithoutChildren(&*self.dom, &node))
    }
    fn RemoveId(&self, node: u64) {
        let name = QualifiedName::new(
            Default::default(),
            foundation::AtomicString::from_str("id"),
            Default::default(),
        );
        let _ = self.Apply(Rc::new(SetNodeAttributeCommand::new(
            self.dom.clone(),
            node,
            name,
            None,
        )));
    }
    fn Append(&self, node: u64, parent: u64) -> EditResult {
        self.Apply(Rc::new(AppendNodeCommand::new(
            self.dom.clone(),
            parent,
            node,
        )))
    }
    fn InsertBefore(&self, node: u64, reference: u64) -> EditResult {
        self.Apply(Rc::new(InsertNodeBeforeCommand::new(
            self.dom.clone(),
            node,
            reference,
            false,
        )))
    }
    fn Remove(&self, node: u64) -> EditResult {
        self.Apply(Rc::new(RemoveNodeCommand::new(
            self.dom.clone(),
            node,
            false,
        )))
    }
    fn SplitText(&self, right: u64, offset: usize) -> EditResult {
        self.Apply(Rc::new(SplitTextNodeCommand::new(
            self.dom.clone(),
            right,
            offset,
        )))
    }
    fn SplitElement(&self, right: u64, at: u64) -> EditResult {
        self.Apply(Rc::new(SplitElementCommand::new(
            self.dom.clone(),
            right,
            at,
        )))
    }
    fn DeleteText(&self, node: u64, offset: usize, count: usize) -> EditResult {
        if count == 0 {
            return Ok(());
        }
        self.Apply(Rc::new(DeleteFromTextNodeCommand::new(
            self.dom.clone(),
            node,
            offset,
            count,
        )))
    }
    fn InsertText(&self, node: u64, offset: usize, text: &[u16]) -> EditResult {
        if text.is_empty() {
            return Ok(());
        }
        self.Apply(Rc::new(InsertIntoTextNodeCommand::new(
            self.dom.clone(),
            node,
            offset,
            Rc::from(text),
            PasswordEchoBehavior::kDoNotEcho,
        )))
    }
    fn SetEndingPosition(&self, p: &Position<u64>) {
        if let Some(position) = CommandDOM::Native(p) {
            let selection = ImeTypingSelection(DOMSelection {
                anchor: position,
                focus: position,
                affinity: EditingTextAffinity::Downstream,
            });
            self.composite.SetEndingSelection(selection.clone());
            if foundation::RuntimeEnabledFeatures::EditingUseDomPositionApiEnabled() {
                self.composite.SetEndingDomSelection(selection);
            }
        }
    }
}
fn Context(
    host: &Host<'_, '_, '_, '_>,
    command: &Rc<CompositeEditCommand<ImeTypingSelection>>,
) -> StructuralCommand {
    StructuralCommand {
        dom: Rc::new(CommandDOM {
            document: host.commands.document.clone(),
            emit: host.commands.emit.clone(),
            frame: Rc::downgrade(host.commands.selection.frame.as_ref().unwrap()),
            selections: host.commands.selection.selections.clone(),
            document_identity: host
                .commands
                .document
                .borrow()
                .GetDocument()
                .RootHandle()
                .Identity(),
            editor: host.commands.selection.editor.ime_typing.editor.clone(),
            markers: host
                .commands
                .selection
                .editor
                .ime_typing
                .markers
                .borrow()
                .as_ref()
                .map(|(_, owner)| owner.clone()),
        }),
        composite: command.clone(),
    }
}
fn End(ctx: &StructuralCommand, p: Position<u64>) -> Option<ImeTypingSelection> {
    let p = CommandDOM::Native(&p)?;
    ctx.SetEndingPosition(&CommandDOM::Source(p));
    Some(ImeTypingSelection(DOMSelection {
        anchor: p,
        focus: p,
        affinity: EditingTextAffinity::Downstream,
    }))
}
pub(super) fn Replace(
    host: &Host<'_, '_, '_, '_>,
    text: &[u16],
    selection: &ImeTypingSelection,
    select: bool,
    command: &Rc<CompositeEditCommand<ImeTypingSelection>>,
) -> Option<ImeTypingSelection> {
    let ctx = Context(host, command);
    let range = host.Range(selection)?;
    let start = Position::new(range.start.node_id, range.start.offset);
    let end = Position::new(range.end.node_id, range.end.offset);
    let position = if start != end {
        DeleteSelectionCommand::Apply(&ctx, (start, end)).ok()?
    } else {
        start
    };
    let position = position.ToOffsetInAnchor(&ctx);
    let node = position.ComputeContainerNode(&ctx)?;
    let offset = position.ComputeOffsetInContainerNode(&ctx);
    let (mut start, mut end) = if PositionTree::IsText(&ctx, &node) {
        ctx.InsertText(node, offset, text).ok()?;
        (
            Position::new(node, offset),
            Position::new(node, offset + text.len()),
        )
    } else if !text.is_empty() {
        let text_node = SimpleEditingDOM::CreateText(&*ctx.dom, text);
        interaction_default::core::editing::commands::composite_edit_command::InsertNodeAt(
            &ctx, text_node, &position,
        )
        .ok()?;
        (
            Position::new(text_node, 0),
            Position::new(text_node, text.len()),
        )
    } else {
        (position.clone(), position)
    };
    // insert_text_command.cc:428-458 applies the retained typing style to
    // the inserted range. The same ApplyStyleCommand handles style removal,
    // relative font sizes, splitting and its resulting selection.
    if !text.is_empty() {
        let typing = ctx
            .dom
            .editor
            .TypingStyle()
            .map(|style| style.borrow().clone());
        if let Some(style) = typing {
            use interaction_default::core::editing::commands::apply_style_command::{
                ApplyStyleContext, ApplyStyleRange,
            };
            ctx.SetApplyStyleSelection(&start, &end);
            ctx.UpdateStyleAndLayout();
            let prepared = ctx.PrepareStyle(&style, &end).ok()?;
            ctx.SetTypingStyle(prepared.clone());
            if let Some(style) = prepared {
                ApplyStyleRange(&ctx, &style, &start, &end).ok()?;
                let selection = command.EndingSelection();
                start = CommandDOM::Source(selection.0.anchor);
                end = CommandDOM::Source(selection.0.focus);
            }
        }
    }
    let start = CommandDOM::Native(&start)?;
    let end = CommandDOM::Native(&end)?;
    *host.commands.ending.borrow_mut() = Some((
        DOMBoundaryPoint {
            node_id: start.node_id?,
            offset: start.offset,
        },
        DOMBoundaryPoint {
            node_id: end.node_id?,
            offset: end.offset,
        },
    ));
    Some(ImeTypingSelection(DOMSelection {
        anchor: if select { start } else { end },
        focus: end,
        affinity: EditingTextAffinity::Downstream,
    }))
}
pub(super) fn Delete(
    host: &Host<'_, '_, '_, '_>,
    selection: &ImeTypingSelection,
    command: &Rc<CompositeEditCommand<ImeTypingSelection>>,
    smart: bool,
) -> Option<ImeTypingSelection> {
    let ctx = Context(host, command);
    End(
        &ctx,
        DeleteSelectionCommand::ApplyWithSmartDelete(
            &ctx,
            (
                CommandDOM::Source(selection.0.anchor),
                CommandDOM::Source(selection.0.focus),
            ),
            smart,
        )
        .ok()?,
    )
}
pub(super) fn DeleteWithExpansion(
    host: &Host<'_, '_, '_, '_>,
    selection: &ImeTypingSelection,
    command: &Rc<CompositeEditCommand<ImeTypingSelection>>,
    smart: bool,
    expand: bool,
) -> Option<ImeTypingSelection> {
    let context = Context(host, command);
    let ending = DeleteSelectionCommand::ApplyWithTypingOptions(
        &context,
        (
            CommandDOM::Source(selection.0.anchor),
            CommandDOM::Source(selection.0.focus),
        ),
        smart,
        expand,
    )
    .ok()?;
    End(&context, ending)
}
pub(super) fn Paragraph(
    host: &Host<'_, '_, '_, '_>,
    selection: &ImeTypingSelection,
    command: &Rc<CompositeEditCommand<ImeTypingSelection>>,
    line_break: bool,
) -> Option<ImeTypingSelection> {
    let ctx = Context(host, command);
    let range = (
        CommandDOM::Source(selection.0.anchor),
        CommandDOM::Source(selection.0.focus),
    );
    let position = if line_break {
        InsertLineBreak(&ctx, range)
    } else {
        InsertParagraphSeparatorCommand::Apply(&ctx, range)
    };
    End(&ctx, position.ok()?)
}
pub(super) fn QuotedParagraph(
    host: &Host<'_, '_, '_, '_>,
    selection: &ImeTypingSelection,
    command: &Rc<CompositeEditCommand<ImeTypingSelection>>,
) -> Option<ImeTypingSelection> {
    let ctx = Context(host, command);
    let position=interaction_default::core::editing::commands::break_blockquote_command::BreakBlockquoteCommand::InsertParagraphSeparatorInQuotedContent(&ctx,(CommandDOM::Source(selection.0.anchor),CommandDOM::Source(selection.0.focus))).ok()?;
    match position {
        Some(position) => End(&ctx, position),
        None => Some(selection.clone()),
    }
}
pub(super) fn SelectionStartsInTableStructure(
    host: &Host<'_, '_, '_, '_>,
    selection: &ImeTypingSelection,
    command: &Rc<CompositeEditCommand<ImeTypingSelection>>,
) -> bool {
    let ctx = Context(host, command);
    interaction_default::core::editing::commands::break_blockquote_command::BreakBlockquoteCommand::SelectionStartsInTableStructure(&ctx,(CommandDOM::Source(selection.0.anchor),CommandDOM::Source(selection.0.focus)))
}

impl interaction_default::core::editing::commands::typing_deletion_structural::TypingDeletionStructuralContext for StructuralCommand {
    fn SetFrameSelectionBeforeRootRemoval(&self, position: &Position<u64>) {
        if let Some(position) = CommandDOM::Native(position) {
            if let Some(frame) = self.dom.frame.upgrade() {
                frame.SetSelection(Some(DOMSelection {anchor:position, focus:position, affinity:EditingTextAffinity::Downstream}), SetSelectionOptions {is_directional:self.composite.SelectionIsDirectional(), ..Default::default()});
            }
        }
    }
}

pub(super) fn RawDeletionSelection(
    selection: &ImeTypingSelection,
) -> interaction_default::core::editing::visible_selection::VisibleSelection<u64> {
    interaction_default::core::editing::visible_selection::VisibleSelection {
        anchor: CommandDOM::Source(selection.0.anchor),
        focus: CommandDOM::Source(selection.0.focus),
        affinity: if selection.0.affinity == EditingTextAffinity::Upstream {
            interaction_default::core::editing::position::TextAffinity::Upstream
        } else {
            interaction_default::core::editing::position::TextAffinity::Downstream
        },
    }
}
pub(super) enum DeletionStructuralOperation {
    MailBlockquote,
    EmptyList,
    EmptyRoot,
}
pub(super) fn DeletionStructural(
    host: &Host<'_, '_, '_, '_>,
    command: &Rc<CompositeEditCommand<ImeTypingSelection>>,
    operation: DeletionStructuralOperation,
) -> EditResult<bool> {
    use interaction_default::core::editing::commands::typing_deletion_structural as source;
    let context = Context(host, command);
    let ending = command.EndingSelection();
    if ending.0.anchor != ending.0.focus {
        return Ok(false);
    }
    let position = CommandDOM::Source(ending.0.anchor);
    match operation {
        DeletionStructuralOperation::MailBlockquote => source::BreakOutOfEmptyMailBlockquotedParagraph(&context,&position),
        DeletionStructuralOperation::EmptyList => Ok(interaction_default::core::editing::commands::composite_edit_command::BreakOutOfEmptyListItem(&context,&position)?.is_some()),
        DeletionStructuralOperation::EmptyRoot => source::MakeEditableRootEmpty(&context,&position),
    }
}
