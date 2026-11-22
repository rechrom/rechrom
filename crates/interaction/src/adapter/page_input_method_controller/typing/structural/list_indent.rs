//! Native leaves for the source list/indent composite command families.
use super::*;
use interaction_default::core::editing::commands::{
    composite_edit_command::ParagraphMoveHost,
    editor_command_table::EditingCommandType,
    indent_outdent_command::{IndentOutdentCommand, IndentType},
    insert_list_command::{InsertListCommand, ListEditingContext, ListType},
    paragraph_cloning::ParagraphCloningContext,
    replace_selection_command::TrivialReplacementContext,
    simple_edit_commands::MergeIdenticalElementsCommand,
};
struct MovingParagraphHost<'a, 'host, 'scope, 'listener, 'frame> {
    host: &'a Host<'host, 'scope, 'listener, 'frame>,
    context: &'a StructuralCommand,
}
impl ParagraphMoveHost for MovingParagraphHost<'_, '_, '_, '_, '_> {
    fn CreateFragment(
        &self,
        start: &Position<u64>,
        end: &Position<u64>,
        constraining_ancestor: Option<u64>,
    ) -> EditResult<Option<u64>> {
        let selection = DOMSelection {
            anchor: CommandDOM::Native(start).ok_or(EditingAborted)?,
            focus: CommandDOM::Native(end).ok_or(EditingAborted)?,
            affinity: EditingTextAffinity::Downstream,
        };
        let markup = self
            .host
            .Frame()
            .SerializeParagraphForMove(selection, constraining_ancestor)
            .map_err(|_| EditingAborted)?;
        let sanitizer = self.host.Frame().MarkupSanitizationHost();
        crate::adapter::page_markup_sanitization::CreateStrictlyProcessedFragmentFromMarkupWithContext(
            &*sanitizer, &self.context.dom.document, &markup, 0, markup.len(), "",
        ).map(|fragment| fragment.map(|fragment| fragment.root)).map_err(|_| EditingAborted)
    }
    fn PrepareFragment(&self, fragment: Option<u64>, root: u64) -> EditResult<interaction_default::core::editing::commands::replace_selection_command::ReplacementFragment>{
        replacement::PrepareMovingFragment(self.host, self.context, fragment, root)
    }
    fn RangeLength(&self, start: &Position<u64>, end: &Position<u64>) -> EditResult<usize> {
        match self.context.dom.Query(Query::MovingParagraphRangeLength {
            start: CommandDOM::Native(start).ok_or(EditingAborted)?,
            end: CommandDOM::Native(end).ok_or(EditingAborted)?,
        }) {
            Some(ResultValue::Length(length)) => Ok(length),
            _ => Err(EditingAborted),
        }
    }
    fn PositionForIndex(&self, index: usize) -> Position<u64> {
        match self
            .context
            .dom
            .Query(Query::MovingParagraphPositionForIndex(index))
        {
            Some(ResultValue::StructuralPosition(p)) => CommandDOM::Source(p.position),
            _ => Position::default(),
        }
    }
    fn DocumentElement(&self) -> Option<u64> {
        let owner = self.context.dom.document.borrow();
        let document = owner.GetDocument();
        Some(document.Node(document.DocumentElement()?).Id())
    }
}
impl ParagraphCloningContext for StructuralCommand {
    fn CloneNode(&self, node: u64, deep: bool) -> EditResult<u64> {
        let clone = self.dom.FreshId();
        if !self.dom.Mutation(DOMMutation {
            mutation_type: DOMMutationType::kCloneNode,
            target_node_id: node,
            child_node_id: clone,
            bool_value: deep,
            ..Default::default()
        }) {
            return Err(EditingAborted);
        }
        self.dom.Read(clone, |_, _, _| clone).ok_or(EditingAborted)
    }
    fn SetEndingRange(&self, start: &Position<u64>, end: &Position<u64>) {
        self.SetEndingReplacementSelection(start, end);
    }
}
impl ListEditingContext for StructuralCommand {
    fn IsInlineElement(&self, node: u64) -> bool {
        self.dom
            .Read(node, |document, n, index| {
                n.Type() == DOMNodeType::kElement
                    && document
                        .ResolvedStyleFor(index)
                        .and_then(|s| s.ComputedStyle())
                        .is_some_and(|s| {
                            matches!(
                                s.Display(),
                                foundation::EDisplay::kInline | foundation::EDisplay::kRuby
                            )
                        })
            })
            .unwrap_or(false)
    }
    fn EndingRange(&self) -> (Position<u64>, Position<u64>) {
        let selection = self.composite.EndingSelection().0;
        let first = CommandDOM::Source(selection.anchor);
        let second = CommandDOM::Source(selection.focus);
        if self.Compare(&first, &second) == Some(std::cmp::Ordering::Greater) {
            (second, first)
        } else {
            (first, second)
        }
    }
    fn EndingDomRange(&self) -> (Position<u64>, Position<u64>) {
        let selection = self.composite.EndingDomSelection().0;
        let first = CommandDOM::Source(selection.anchor);
        let second = CommandDOM::Source(selection.focus);
        if self.Compare(&first, &second) == Some(std::cmp::Ordering::Greater) {
            (second, first)
        } else {
            (first, second)
        }
    }
    fn EditingUseDomPositionApiEnabled(&self) -> bool {
        foundation::RuntimeEnabledFeatures::EditingUseDomPositionApiEnabled()
    }
    fn IndexForVisiblePosition(&self, position: &Position<u64>) -> Option<(u64, usize)> {
        match self
            .dom
            .Query(Query::VisiblePositionIndex(CommandDOM::Native(position)?))?
        {
            ResultValue::VisiblePositionIndex(value) => value,
            _ => None,
        }
    }
    fn VisiblePositionForIndex(&self, scope: u64, index: usize) -> Position<u64> {
        match self
            .dom
            .Query(Query::PositionForVisibleIndex { scope, index })
        {
            Some(ResultValue::StructuralPosition(p)) => CommandDOM::Source(p.position),
            _ => Position::default(),
        }
    }
    fn MergeIdenticalElements(&self, first: u64, second: u64) -> EditResult {
        if self.NextSibling(&first) != Some(second) {
            self.Remove(second)?;
            if let Some(next) = self.NextSibling(&first) {
                self.InsertBefore(second, next)?;
            } else {
                self.Append(second, self.Parent(&first).ok_or(EditingAborted)?)?;
            }
        }
        self.Apply(Rc::new(MergeIdenticalElementsCommand::new(
            self.dom.clone(),
            first,
            second,
        )))
    }
    fn SetBlockquoteStyle(&self, node: u64) -> EditResult {
        self.Apply(Rc::new(SetNodeAttributeCommand::new(
            self.dom.clone(),
            node,
            QualifiedName::new(
                Default::default(),
                foundation::AtomicString::from_str("style"),
                Default::default(),
            ),
            Some(CommandAttributeValue::Text(
                "margin: 0 0 0 40px; border: none; padding: 0px;"
                    .encode_utf16()
                    .collect::<Vec<_>>()
                    .into(),
            )),
        )))
    }
    fn EnsureNodeVisibility(&self, node: u64) -> EditResult<bool> {
        // editing_utilities.cc:201-214. This port is called for the new,
        // attribute-free BR placeholder and preserves CSS !important.
        let (hidden, none) = self
            .dom
            .Read(node, |document, _, index| {
                document
                    .ResolvedStyleFor(index)
                    .and_then(|s| s.ComputedStyle())
                    .map(|s| {
                        (
                            s.Visibility() == foundation::style_constants::EVisibility::kHidden,
                            s.Display() == foundation::style_constants::EDisplay::kNone,
                        )
                    })
            })
            .flatten()
            .ok_or(EditingAborted)?;
        if !hidden && !none {
            return Ok(false);
        }
        let mut style = String::new();
        if hidden {
            style.push_str("visibility: visible !important;");
        }
        if none {
            style.push_str("display: inline !important;");
        }
        self.Apply(Rc::new(SetNodeAttributeCommand::new(
            self.dom.clone(),
            node,
            QualifiedName::new(
                Default::default(),
                foundation::AtomicString::from_str("style"),
                Default::default(),
            ),
            Some(CommandAttributeValue::Text(
                style.encode_utf16().collect::<Vec<_>>().into(),
            )),
        )))?;
        Ok(true)
    }
}
pub(crate) fn ApplyListIndent(
    host: &Host<'_, '_, '_, '_>,
    command: EditingCommandType,
    composite: &Rc<CompositeEditCommand<ImeTypingSelection>>,
) -> EditResult {
    let c = Context(host, composite);
    let moving = MovingParagraphHost { host, context: &c };
    c.UpdateStyleAndLayout();
    let range = host
        .Range(&composite.EndingSelection())
        .ok_or(EditingAborted)?;
    let start = Position::new(range.start.node_id, range.start.offset);
    let end = Position::new(range.end.node_id, range.end.offset);
    match command {
        EditingCommandType::InsertOrderedList => {
            InsertListCommand::new(ListType::Ordered).Apply(&c, &moving, start, end)
        }
        EditingCommandType::InsertUnorderedList => {
            InsertListCommand::new(ListType::Unordered).Apply(&c, &moving, start, end)
        }
        EditingCommandType::Indent => {
            IndentOutdentCommand::new(IndentType::Indent).Apply(&c, &moving, start, end)
        }
        EditingCommandType::Outdent => {
            IndentOutdentCommand::new(IndentType::Outdent).Apply(&c, &moving, start, end)
        }
        _ => Err(EditingAborted),
    }
}
