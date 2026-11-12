//! ReplaceSelectionCommand's native live-DOM ports reuse the same elementary
//! commands, marker controller and CompositeEditCommand undo owner as editing.
use super::*;
use interaction_default::core::editing::commands::replace_selection_command::{
    self as source, GeneralReplacementContext, ReplaceSelectionCommand, ReplaceSelectionOptions,
    ReplacementApplyError, ReplacementFacts, ReplacementFragment, ReplacementFragmentDOM,
    ReplacementRange, TrivialReplacementContext,
};
impl ReplacementFacts for StructuralCommand {
    fn IsStyledInlineHTMLElement(&self, node: u64) -> EditResult<bool> {
        if self.NodeFacts(node).block {
            return Ok(false);
        }
        self.dom
            .frame
            .upgrade()
            .ok_or(EditingAborted)?
            .ElementIsStyledSpanOrHtmlEquivalent(node)
            .map_err(|_| EditingAborted)
    }
    fn LayoutCanHaveChildren(&self, node: u64) -> bool {
        matches!(
            self.dom.Query(Query::ReplacementCanHaveChildren(node)),
            Some(ResultValue::ReplacementCanHaveChildren(true))
        )
    }
    fn InNoQuirksMode(&self) -> bool {
        self.dom
            .document
            .borrow()
            .GetDocument()
            .GetCompatibilityMode()
            == dom::persistent_document::CompatibilityMode::kNoQuirks
    }
    fn TextData(&self, node: u64) -> Vec<u16> {
        SimpleEditingDOM::Text(&*self.dom, &node).to_vec()
    }
    fn CharacterAtVisibleBoundary(&self, position: &Position<u64>, before: bool) -> u16 {
        let Some(position) = CommandDOM::Native(position) else {
            return 0;
        };
        match self.dom.Query(Query::ReplacementBoundaryCharacter {
            position: EditingPositionWithAffinity {
                position,
                affinity: EditingTextAffinity::Downstream,
            },
            before,
        }) {
            Some(ResultValue::ReplacementBoundaryCharacter(c)) => c,
            _ => 0,
        }
    }
}
impl TrivialReplacementContext for StructuralCommand {
    fn Text(&self, node: u64) -> Vec<u16> {
        ReplacementFacts::TextData(self, node)
    }
    fn ElementToSplitToAvoidPastingIntoInlineElementsWithStyle(
        &self,
        p: &Position<u64>,
    ) -> EditResult<Option<u64>> {
        source::ElementToSplitToAvoidPastingIntoInlineElementsWithStyle(self, p)
    }
    fn IsTabSpanElementTextNode(&self, node: u64) -> bool {
        if !PositionTree::IsText(self, &node) {
            return false;
        }
        let Some(parent) = PositionTree::Parent(self, &node) else {
            return false;
        };
        let candidate = self
            .dom
            .Read(parent, |d, n, _| {
                n.IsHTMLElement("span")
                    && n.Children().first().is_some_and(|first| {
                        d.Node(*first).Type() == DOMNodeType::kText
                            && d.CharacterDataUTF16Slice(*first).contains(&9)
                    })
            })
            .unwrap_or(false);
        if !candidate {
            return false;
        }
        self.UpdateStyleAndLayout();
        self.dom
            .Read(parent, |d, _, index| {
                d.ResolvedStyleFor(index)
                    .and_then(|s| s.ComputedStyle())
                    .is_some_and(|s| {
                        s.WhiteSpace() == layoutng_style::css::white_space::EWhiteSpace::kPre
                    })
            })
            .unwrap_or(false)
    }
    fn ShouldRemoveEndBR(&self, node: u64) -> bool {
        let original = self.QueryPosition(
            &Position::BeforeNode(node),
            StructuralPositionQuery::Visible,
        );
        source::ShouldRemoveEndBR(self, node, &original)
    }
    fn RemoveNodeAndPruneAncestors(&self, node: u64) -> EditResult {
        source::RemoveNodeAndPruneAncestors(self, node)
    }
    fn NormalizeNbspInInsertedRange(&self, range: &ReplacementRange) -> EditResult {
        source::NormalizeNbspInInsertedRange(self, range)
    }
    fn NormalizeNbspForPasteEnabled(&self) -> bool {
        foundation::RuntimeEnabledFeatures::NormalizeNbspForPasteAndDropEnabled()
            && self
                .composite
                .EndingSelection()
                .0
                .anchor
                .node_id
                .is_some_and(|n| {
                    if foundation::RuntimeEnabledFeatures::NormalizeNbspRichTextOnlyEnabled() {
                        self.NodeFacts(n).richly_editable
                    } else {
                        self.Editable(n)
                    }
                })
    }
    fn SetEndingReplacementSelection(&self, start: &Position<u64>, end: &Position<u64>) {
        let (Some(start), Some(end)) = (CommandDOM::Native(start), CommandDOM::Native(end)) else {
            return;
        };
        let selection = ImeTypingSelection(DOMSelection {
            anchor: start,
            focus: end,
            affinity: EditingTextAffinity::Downstream,
        });
        self.composite.SetEndingSelection(selection.clone());
        if foundation::RuntimeEnabledFeatures::EditingUseDomPositionApiEnabled() {
            self.composite.SetEndingDomSelection(selection);
        }
    }
}
impl ReplacementFragmentDOM for StructuralCommand {
    fn IsInterchangeNewline(&self, node: u64) -> bool {
        self.Tag(node, "br") && self.AttributeEquals(node, "class", "Apple-interchange-newline")
    }
    fn RemoveFragmentNode(&self, node: u64) -> EditResult {
        let mut owner = self.dom.document.borrow_mut();
        let d = owner.GetDocumentMut();
        let index = d.FindNodeById(node).ok_or(EditingAborted)?;
        if d.IsConnectedToDocument(index) {
            return Err(EditingAborted);
        }
        d.Remove(index);
        Ok(())
    }
}
/// Source command entry, sharing preparation and general/trivial execution.
pub(crate) fn ApplyReplacement(
    host: &Host<'_, '_, '_, '_>,
    fragment: u64,
    options: ReplaceSelectionOptions,
    command: &Rc<CompositeEditCommand<ImeTypingSelection>>,
) -> Result<Vec<u16>, ReplacementApplyError> {
    let ctx = Context(host, command);
    let selection = host
        .Frame()
        .CreateVisibleSelection(host.Frame().Current())
        .ok_or(EditingAborted)?;
    let range = host
        .Range(&ImeTypingSelection(selection))
        .ok_or(EditingAborted)?;
    let start = Position::new(range.start.node_id, range.start.offset);
    let end = Position::new(range.end.node_id, range.end.offset);
    let mut fragment = ReplacementFragment::new(fragment);
    let preparation = FragmentPreparation { ctx: &ctx, host };
    let root = ctx
        .RootEditable(*start.AnchorNode().ok_or(EditingAborted)?)
        .ok_or(EditingAborted)?;
    fragment.Prepare(&preparation, root)?;
    let mut replacement = ReplaceSelectionCommand::new(options);
    replacement.input_type =
        interaction_dispatch::core::events::input_event::InputType::kInsertFromPaste;
    replacement.DoApply(&ctx, &fragment, &start, &end)?;
    Ok(replacement.input_event_data)
}

/// Call-scoped constructor ports. The source owner controls all traversal and
/// preparation decisions; these leaves read/mutate the canonical native DOM.
pub(super) fn PrepareMovingFragment(
    host: &Host<'_, '_, '_, '_>,
    ctx: &StructuralCommand,
    fragment: Option<u64>,
    root: u64,
) -> EditResult<ReplacementFragment> {
    use source::ReplacementPreparationContext;
    let preparation = FragmentPreparation { ctx, host };
    let fragment = match fragment {
        Some(fragment) => fragment,
        None => preparation.CreateEmptyFragment()?,
    };
    let mut fragment = ReplacementFragment::new(fragment);
    fragment.Prepare(&preparation, root)?;
    Ok(fragment)
}
struct FragmentPreparation<'a, 'host, 'scope, 'listener, 'frame> {
    ctx: &'a StructuralCommand,
    host: &'a Host<'host, 'scope, 'listener, 'frame>,
}
impl PositionTree for FragmentPreparation<'_, '_, '_, '_, '_> {
    type Node = u64;
    fn Parent(&self, n: &u64) -> Option<u64> {
        self.ctx.Parent(n)
    }
    fn ChildAt(&self, n: &u64, i: usize) -> Option<u64> {
        self.ctx.ChildAt(n, i)
    }
    fn ChildCount(&self, n: &u64) -> usize {
        self.ctx.ChildCount(n)
    }
    fn NodeIndex(&self, n: &u64) -> usize {
        self.ctx.NodeIndex(n)
    }
    fn PreviousSibling(&self, n: &u64) -> Option<u64> {
        self.ctx.PreviousSibling(n)
    }
    fn NextSibling(&self, n: &u64) -> Option<u64> {
        self.ctx.NextSibling(n)
    }
    fn IsText(&self, n: &u64) -> bool {
        PositionTree::IsText(self.ctx, n)
    }
    fn CharacterDataLength(&self, n: &u64) -> Option<usize> {
        self.ctx.CharacterDataLength(n)
    }
}
impl ReplacementFragmentDOM for FragmentPreparation<'_, '_, '_, '_, '_> {
    fn IsInterchangeNewline(&self, node: u64) -> bool {
        self.ctx.IsInterchangeNewline(node)
    }
    fn RemoveFragmentNode(&self, node: u64) -> EditResult {
        SimpleEditingDOM::Remove(&*self.ctx.dom, &node)
            .then_some(())
            .ok_or(EditingAborted)
    }
}
impl source::ReplacementPreparationContext for FragmentPreparation<'_, '_, '_, '_, '_> {
    fn UpdateStyleAndLayout(&self) {
        self.ctx.UpdateStyleAndLayout();
    }
    fn IsRichlyEditable(&self, node: u64) -> bool {
        self.ctx.NodeFacts(node).richly_editable
    }
    fn IsNodeRendered(&self, node: u64) -> bool {
        let facts = self.ctx.NodeFacts(node);
        facts.has_layout && facts.visible
    }
    fn IsTableStructure(&self, node: u64) -> bool {
        self.ctx.NodeFacts(node).table_structure
    }
    fn FragmentTextContent(&self, node: u64) -> Vec<u16> {
        self.ctx
            .dom
            .Read(node, |d, _, i| d.TextContentUTF16(i).to_vec())
            .unwrap_or_default()
    }
    fn HasBeforeTextInsertedListener(&self, root: u64) -> bool {
        self.host
            .commands
            .dispatcher
            .HasAttributeEventListener(root, "webkitBeforeTextInserted")
    }
    fn EnclosingTextControl(&self, node: u64) -> Option<(u64, bool)> {
        self.ctx
            .dom
            .Read(node, |d, _, i| {
                let mut node = Some(i);
                while let Some(i) = node {
                    let n = d.Node(i);
                    if d.Node(i).IsHTMLElement("textarea")
                        || d.InputType(i).is_some_and(
                            dom::persistent_document::DOMInputType::IsTextFieldInputType,
                        )
                    {
                        return Some((n.Id(), n.IsHTMLElement("textarea")));
                    }
                    node = n.Parent().or_else(|| d.OwnerShadowHost(i));
                }
                None
            })
            .flatten()
    }
    fn BeforeTextInserted(&self, root: u64, text: &[u16]) -> EditResult<Vec<u16>> {
        use source::ReplacementPreparationContext;
        let mut text = text.to_vec();
        let Some((control, textarea)) = self.EnclosingTextControl(root) else {
            return Ok(text);
        };
        let (value, max, focused) = self
            .ctx
            .dom
            .Read(control, |d, _, i| {
                (
                    d.TextControlInnerEditorValueUTF16(i),
                    d.TextControlMaxLength(i),
                    self.host
                        .commands
                        .selection
                        .interaction
                        .borrow()
                        .focused_node_id
                        == Some(control),
                )
            })
            .ok_or(EditingAborted)?;
        let needs_selection=focused&&(!textarea||max>=0&&interaction_default::core::html::forms::html_text_area_element::ComputeLengthForAPIValue(&value)+interaction_default::core::html::forms::html_text_area_element::ComputeLengthForAPIValue(&text)>=max as usize);
        let selected = if needs_selection {
            self.UpdateStyleAndLayout();
            let selection = self.host.Frame().Current().ok_or(EditingAborted)?;
            let range = self
                .host
                .Range(&ImeTypingSelection(selection))
                .ok_or(EditingAborted)?;
            match self.ctx.dom.Query(Query::Text {
                range,
                original: false,
            }) {
                Some(ResultValue::Text(text)) => text,
                _ => return Err(EditingAborted),
            }
        } else {
            Vec::new()
        };
        if textarea {
            interaction_default::core::html::forms::html_text_area_element::FilterBeforeTextInserted(
                &mut text, &value, max, focused, focused.then_some(selected.as_slice()),
            );
        } else {
            interaction_default::core::html::forms::text_field_input_type::FilterBeforeTextInserted(
                &mut text,
                value.len(),
                if focused { selected.len() } else { 0 },
                max,
            );
        }
        // Rechrom has no ChromeClient autofill client. Filtering is the native
        // default-handler operation; no author event propagation occurs here.
        self.ctx.dom.Alive().then_some(text).ok_or(EditingAborted)
    }
    fn CreateFragmentFromText(&self, text: &[u16]) -> EditResult<u64> {
        self.host
            .commands
            .dispatcher
            .CreatePasteTextFragment(text)
            .map(|f| f.root)
            .map_err(|_| EditingAborted)
    }
    fn CreateEmptyFragment(&self) -> EditResult<u64> {
        let mut owner = self.ctx.dom.document.borrow_mut();
        let d = owner.GetDocumentMut();
        let node = d.CreateDocumentFragment();
        Ok(d.Node(node).Id())
    }
    fn CreateFragmentElement(&self, tag: &str) -> EditResult<u64> {
        self.ctx.CreateElement(tag)
    }
    fn AppendFragmentChild(&self, node: u64, parent: u64) -> EditResult {
        SimpleEditingDOM::AppendChild(&*self.ctx.dom, &parent, &node)
            .then_some(())
            .ok_or(EditingAborted)
    }
    fn AppendTextOrBr(&self, text: &[u16], fragment: u64) -> EditResult {
        let mut owner = self.ctx.dom.document.borrow_mut();
        let d = owner.GetDocumentMut();
        let node = d.FindNodeById(fragment).ok_or(EditingAborted)?;
        d.AppendTextOrBrUTF16(node, text);
        Ok(())
    }
    fn TestRenderingText(&self, holder: u64) -> EditResult<Vec<u16>> {
        match self
            .ctx
            .dom
            .Query(Query::ReplacementTestRenderingText(holder))
        {
            Some(ResultValue::Text(text)) => Ok(text),
            _ => Err(EditingAborted),
        }
    }
}

impl GeneralReplacementContext for StructuralCommand {
    fn IsTextControlPlaceholderBreak(&self, node: u64) -> bool {
        self.dom
            .Read(node, |d, _, i| d.TextControlIsPlaceholderBreak(i))
            .unwrap_or(false)
    }
    fn IsInTextControl(&self, node: u64) -> bool {
        self.dom
            .Read(node, |d, _, i| {
                let mut node = Some(i);
                while let Some(i) = node {
                    if d.Node(i).IsHTMLElement("textarea")
                        || d.InputType(i).is_some_and(
                            dom::persistent_document::DOMInputType::IsTextFieldInputType,
                        )
                    {
                        return true;
                    }
                    node = d.Node(i).Parent().or_else(|| d.OwnerShadowHost(i));
                }
                false
            })
            .unwrap_or(false)
    }
    fn RebalanceReplacementWhitespace(&self, position: &Position<u64>) -> EditResult {
        interaction_default::core::editing::commands::replacement_position::RebalanceWhitespaceAt(
            self, position,
        )
    }
    fn ApplyReplacementStyleRange(
        &self,
        style: &Self::Style,
        start: &Position<u64>,
        end: &Position<u64>,
    ) -> EditResult {
        interaction_default::core::editing::commands::apply_style_command::ApplyStyleRange(
            self, style, start, end,
        )
    }
    fn ApplyReplacementStyledElement(
        &self,
        element: u64,
        start: &Position<u64>,
        end: &Position<u64>,
    ) -> EditResult {
        interaction_default::core::editing::commands::apply_style_command::ApplyStyledElement(
            self, element, start, end,
        )
    }
    fn IsPasswordField(&self, position: &Position<u64>) -> bool {
        position.AnchorNode().is_some_and(|node| {
            self.dom
                .Read(*node, |d, _, i| {
                    let mut node = Some(i);
                    while let Some(i) = node {
                        if d.InputType(i) == Some(dom::persistent_document::DOMInputType::kPassword)
                        {
                            return true;
                        }
                        node = d.Node(i).Parent().or_else(|| d.OwnerShadowHost(i));
                    }
                    false
                })
                .unwrap_or(false)
        })
    }
    fn WrapReplacementContentsInDummySpan(&self, node: u64) -> EditResult<u64> {
        self.Apply(Rc::new(interaction_default::core::editing::commands::simple_edit_commands::WrapContentsInDummySpanCommand::new(self.dom.clone(),node)))?;
        self.ChildAt(&node, 0).ok_or(EditingAborted)
    }
    fn HasLinkHref(&self, node: u64) -> bool {
        self.dom
            .Read(node, |_, n, _| n.FindAttribute("href").is_some())
            .unwrap_or(false)
    }
    fn EditingUseDomPositionApi(&self) -> bool {
        foundation::RuntimeEnabledFeatures::EditingUseDomPositionApiEnabled()
    }
    fn RawLineBreakExists(&self, p: &Position<u64>) -> bool {
        CommandDOM::Native(p).is_some_and(|p| {
            matches!(
                self.dom.Query(Query::ReplacementRawLineBreak(p)),
                Some(ResultValue::StructuralBoolean(true))
            )
        })
    }
    fn TabSpanCheckSibling(&self) -> bool {
        true
    } // Pinned source runtime feature: stable.
    fn RemoveCollapsedPlaceholderGate(&self) -> bool {
        foundation::RuntimeEnabledFeatures::RemoveCollapsedPlaceholderForContentEditableEnabled()
    }
    fn CreateTextControlPlaceholderBreak(&self, _root: u64) -> EditResult<u64> {
        let br = self.CreateElement("br")?;
        self.Apply(Rc::new(SetNodeAttributeCommand::new(
            self.dom.clone(),
            br,
            QualifiedName::new(
                Default::default(),
                foundation::AtomicString::from_str("id"),
                Default::default(),
            ),
            Some(CommandAttributeValue::Text(Rc::from(
                "placeholder-break".encode_utf16().collect::<Vec<_>>(),
            ))),
        )))?;
        Ok(br)
    }
}

impl interaction_default::core::editing::commands::replacement_smart::SmartReplacementContext
    for StructuralCommand
{
    fn CharacterCodePointAtBoundary(&self, p: &Position<u64>, before: bool) -> u32 {
        let Some(position) = CommandDOM::Native(p) else {
            return 0;
        };
        match self.dom.Query(Query::ReplacementBoundaryCodePoint {
            position: EditingPositionWithAffinity {
                position,
                affinity: EditingTextAffinity::Downstream,
            },
            before,
        }) {
            Some(ResultValue::ReplacementBoundaryCodePoint(c)) => c,
            _ => 0,
        }
    }
    fn CreateReplacementText(&self, text: &[u16]) -> EditResult<u64> {
        let node = SimpleEditingDOM::CreateText(&*self.dom, text);
        self.dom.Read(node, |_, _, _| node).ok_or(EditingAborted)
    }
    fn SmartReplaceUsesMacCharacterSets(&self) -> bool {
        cfg!(target_os = "macos")
    }
}
impl interaction_default::core::editing::commands::replacement_markup::ReplacementMarkupContext
    for StructuralCommand
{
    fn IsHTMLElement(&self, node: u64) -> bool {
        self.dom
            .Read(node, |_, n, _| {
                n.Type() == DOMNodeType::kElement && n.Namespace() == dom::DOMNamespace::kHTML
            })
            .unwrap_or(false)
    }
    fn HasLocalName(&self, node: u64, name: &str) -> bool {
        self.dom
            .Read(node, |_, n, _| {
                n.Type() == DOMNodeType::kElement && n.Name() == name
            })
            .unwrap_or(false)
    }
    fn IsSVGStyleElement(&self, node: u64) -> bool {
        self.dom
            .Read(node, |_, n, _| {
                n.Namespace() == dom::DOMNamespace::kSVG && n.Name() == "style"
            })
            .unwrap_or(false)
    }
    fn NodeHasVisibleLayoutText(&self, node: u64) -> bool {
        matches!(
            self.dom.Query(Query::ReplacementVisibleText(node)),
            Some(ResultValue::ReplacementVisibleText(true))
        )
    }
}
impl interaction_default::core::editing::commands::break_blockquote_command::BreakBlockquoteContext
    for StructuralCommand
{
    fn ListItemNumber(&self, node: u64) -> Option<i32> {
        match self.dom.Query(Query::ReplacementListItemNumber(node)) {
            Some(ResultValue::ReplacementListItemNumber(n)) => n,
            _ => None,
        }
    }
    fn SetListStart(&self, node: u64, number: i32) -> EditResult {
        self.Apply(Rc::new(SetNodeAttributeCommand::new(
            self.dom.clone(),
            node,
            QualifiedName::new(
                Default::default(),
                foundation::AtomicString::from_str("start"),
                Default::default(),
            ),
            Some(CommandAttributeValue::Text(Rc::from(
                number.to_string().encode_utf16().collect::<Vec<_>>(),
            ))),
        )))
    }
    fn BlockPlaceholderFacts(
        &self,
        node: u64,
    ) -> interaction_default::core::editing::commands::break_blockquote_command::BlockPlaceholderFacts
    {
        match self.dom.Query(Query::ReplacementBlockPlaceholder(node)) {
            Some(ResultValue::ReplacementBlockPlaceholder(f)) => f,
            _ => Default::default(),
        }
    }
    fn MoveByGrapheme(&self, p: &Position<u64>, forward: bool) -> Position<u64> {
        let Some(position) = CommandDOM::Native(p) else {
            return Position::default();
        };
        match self
            .dom
            .Query(Query::ReplacementMoveGrapheme { position, forward })
        {
            Some(ResultValue::StructuralPosition(p)) => CommandDOM::Source(p.position),
            _ => Position::default(),
        }
    }
    fn RebalanceBreakWhitespace(&self, p: &Position<u64>) -> EditResult {
        interaction_default::core::editing::commands::replacement_position::RebalanceWhitespaceAt(
            self, p,
        )
    }
}
