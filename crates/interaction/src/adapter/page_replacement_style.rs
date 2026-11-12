//! Native leaves for replacement's style and markup simplification commands.
//! Style owns declaration algebra; these ports only route compiled owners and
//! execute source elementary mutations on the canonical Document/undo owner.
use super::*;
use cssom::editing_style::{ReplacementStyleOperation as Op, ReplacementStyleResult as Out};
use interaction_default::core::editing::commands::{
    replace_node_with_span_command::{ReplaceNodeWithSpanCommand, SpanReplacementDOM},
    replacement_style::{ReplacementStyleContext, ReplacementWhiteSpaceFacts},
    simplify_markup_command::SimplifyMarkupContext,
};
impl StructuralCommand {
    fn ReplacementStyleQuery(&self, operation: Op) -> EditResult<Out> {
        self.dom
            .frame
            .upgrade()
            .ok_or(EditingAborted)?
            .ReplacementEditingStyle(operation)
            .map_err(|_| EditingAborted)
    }
}
impl SpanReplacementDOM for CommandDOM {
    fn IsConnected(&self, node: &u64) -> bool {
        PositionDOM::IsConnected(self, node)
    }
    fn Attributes(&self, node: &u64) -> Vec<(QualifiedName, CommandAttributeValue)> {
        self.Read(*node, |_, n, _| {
            n.AttributesUTF16()
                .iter()
                .map(|attribute| {
                    let name = QualifiedName::new(
                        foundation::AtomicString::from_utf16(&attribute.name.prefix),
                        foundation::AtomicString::from_utf16(&attribute.name.local_name),
                        foundation::AtomicString::from_utf16(&attribute.name.namespace_uri),
                    );
                    let value = if attribute.name.local_name.as_ref() == [115, 116, 121, 108, 101]
                        && attribute.name.namespace_uri.is_empty()
                    {
                        CommandAttributeValue::Inline(n.InlineStyle())
                    } else {
                        CommandAttributeValue::Text(attribute.value.clone())
                    };
                    (name, value)
                })
                .collect()
        })
        .unwrap_or_default()
    }
}
impl ReplacementStyleContext for StructuralCommand {
    type Declaration = cssom::CSSInlineStyle;
    fn IsStyledElement(&self, node: u64) -> bool {
        self.IsElement(node)
    }
    fn IsStyleSpan(&self, node: u64) -> bool {
        self.dom
            .Read(node, |_, n, _| {
                n.IsHTMLElement("span")
                    && (n.Attributes().is_empty()
                        || n.Attributes().len() == 1 && n.FindAttribute("style").is_some())
            })
            .unwrap_or(false)
    }
    fn IsEmptyFontTag(&self, node: u64) -> bool {
        self.dom
            .Read(node, |_, n, _| {
                n.IsHTMLElement("font")
                    && (n.Attributes().is_empty()
                        || n.Attributes().len() == 1 && n.FindAttribute("style").is_some())
            })
            .unwrap_or(false)
    }
    fn AreIdenticalElements(&self, a: u64, b: u64) -> bool {
        if !self.Editable(a) || !self.Editable(b) {
            return false;
        }
        let owner = self.dom.document.borrow();
        let d = owner.GetDocument();
        let (Some(a), Some(b)) = (d.FindNodeById(a), d.FindNodeById(b)) else {
            return false;
        };
        let (a, b) = (d.Node(a), d.Node(b));
        a.Type() == DOMNodeType::kElement
            && b.Type() == DOMNodeType::kElement
            && a.TagQName() == b.TagQName()
            && a.AttributesUTF16().len() == b.AttributesUTF16().len()
            && a.AttributesUTF16()
                .iter()
                .all(|attr| b.AttributesUTF16().contains(attr))
    }
    fn InlineStyle(&self, node: u64) -> EditResult<Option<Self::Declaration>> {
        match self.ReplacementStyleQuery(Op::Inline { node_id: node })? {
            Out::Inline(style) => Ok(style),
            _ => Err(EditingAborted),
        }
    }
    fn PropertyCount(&self, declaration: &Self::Declaration) -> EditResult<usize> {
        match self.ReplacementStyleQuery(Op::PropertyCount {
            declaration: declaration.clone(),
        })? {
            Out::Count(count) => Ok(count),
            _ => Err(EditingAborted),
        }
    }
    fn ImplicitStyleConflicts(
        &self,
        node: u64,
        declaration: &Self::Declaration,
    ) -> EditResult<(bool, Vec<QualifiedName>)> {
        match self.ReplacementStyleQuery(Op::ImplicitConflicts {
            node_id: node,
            declaration: declaration.clone(),
        })? {
            Out::ImplicitConflicts {
                replace_with_span,
                attributes,
            } => Ok((replace_with_span, attributes)),
            _ => Err(EditingAborted),
        }
    }
    fn RemoveStyleFromContext(
        &self,
        node: u64,
        context: Option<u64>,
        declaration: &Self::Declaration,
        rules: bool,
    ) -> EditResult<Self::Declaration> {
        match self.ReplacementStyleQuery(Op::RemoveContext {
            node_id: node,
            context_id: context,
            declaration: declaration.clone(),
            include_rules: rules,
        })? {
            Out::Declaration(style) => Ok(style),
            _ => Err(EditingAborted),
        }
    }
    fn SetInlineStyle(&self, node: u64, declaration: &Self::Declaration) -> EditResult {
        self.Apply(Rc::new(SetNodeAttributeCommand::new(
            self.dom.clone(),
            node,
            QualifiedName::new(
                Default::default(),
                foundation::AtomicString::from_str("style"),
                Default::default(),
            ),
            Some(CommandAttributeValue::Inline(declaration.clone())),
        )))
    }
    fn RemoveAttribute(&self, node: u64, name: &QualifiedName) -> EditResult {
        self.Apply(Rc::new(SetNodeAttributeCommand::new(
            self.dom.clone(),
            node,
            name.clone(),
            None,
        )))
    }
    fn ReplaceWithSpan(&self, node: u64) -> EditResult<u64> {
        let command = Rc::new(ReplaceNodeWithSpanCommand::new(self.dom.clone(), node));
        self.Apply(command.clone())?;
        command.SpanElement().ok_or(EditingAborted)
    }
    fn WhiteSpaceFacts(
        &self,
        node: u64,
        include_author_rules: bool,
    ) -> EditResult<ReplacementWhiteSpaceFacts> {
        match self.ReplacementStyleQuery(Op::WhiteSpaceFacts {
            node_id: node,
            include_author_rules,
        })? {
            Out::WhiteSpaceFacts(facts) => Ok(ReplacementWhiteSpaceFacts {
                has_inline_white_space: facts.has_inline_white_space,
                has_author_white_space: facts.has_author_white_space,
                collapse_breaks_and_wrap: facts.collapse_breaks_and_wrap,
            }),
            _ => Err(EditingAborted),
        }
    }
    fn WhiteSpaceNormal(&self) -> EditResult<Self::Declaration> {
        match self.ReplacementStyleQuery(Op::WhiteSpaceNormal)? {
            Out::Declaration(style) => Ok(style),
            _ => Err(EditingAborted),
        }
    }
    fn InsertionStyleMatchesAttribute(&self, node: u64, span: u64) -> EditResult<bool> {
        match self.ReplacementStyleQuery(Op::InsertionStyleMatchesAttribute {
            node_id: node,
            span_id: span,
        })? {
            Out::Bool(same) => Ok(same),
            _ => Err(EditingAborted),
        }
    }
    fn MoveParagraphsPreserveInlineStructure(&self) -> bool {
        foundation::RuntimeEnabledFeatures::MoveParagraphsPreserveInlineStructureEnabled()
    }
    fn DocumentElement(&self) -> Option<u64> {
        let owner = self.dom.document.borrow();
        let d = owner.GetDocument();
        d.DocumentElement().map(|i| d.Node(i).Id())
    }
    fn FragmentInsertBefore(&self, node: u64, reference: u64) -> EditResult {
        let parent = self.Parent(&reference).ok_or(EditingAborted)?;
        SimpleEditingDOM::InsertBefore(&*self.dom, &parent, &node, Some(&reference))
            .then_some(())
            .ok_or(EditingAborted)
    }
}
impl SimplifyMarkupContext for StructuralCommand {
    fn InlineAlwaysCreateLineBoxes(&self, node: u64) -> Option<bool> {
        match self.dom.Query(Query::SimplifyMarkupInline(node)) {
            Some(ResultValue::SimplifyMarkupInline(value)) => value,
            _ => None,
        }
    }
    fn ComputedStylesDiffer(&self, a: u64, b: u64) -> EditResult<Option<bool>> {
        match self.ReplacementStyleQuery(Op::VisualDifference {
            node_id: a,
            other_id: b,
        })? {
            Out::VisualDifference(value) => Ok(value),
            _ => Err(EditingAborted),
        }
    }
    fn RemoveAssumingEditable(&self, node: u64) -> EditResult {
        self.Apply(Rc::new(RemoveNodeCommand::new(
            self.dom.clone(),
            node,
            true,
        )))
    }
    fn InsertBeforeAssumingEditable(&self, node: u64, reference: u64) -> EditResult {
        self.Apply(Rc::new(InsertNodeBeforeCommand::new(
            self.dom.clone(),
            node,
            reference,
            true,
        )))
    }
}
