//! ApplyStyleCommand's native leaves share the existing elementary command,
//! compiled CSS declaration and CompositeEditCommand undo owners.
use super::*;
use crate::adapter::page_editing_visible_position::apply_style_geometry::{
    ApplyStyleGeometryQuery as GQ, ApplyStyleGeometryResult as GR,
};
use cssom::editing_style::{
    ApplyStyleExtraction as Extraction, ApplyStyleOperation as Op, ApplyStyleResult as Out,
};
use interaction_default::core::editing::commands::{
    apply_style_command::{
        ApplyStyleContext, ApplyStyleQuery as Q, ApplyStyleResult as R, StyleChange, StyleConflict,
        StyleExtraction,
    },
    replacement_style::ReplacementStyleContext,
    simple_edit_commands::MergeIdenticalElementsCommand,
};
impl StructuralCommand {
    fn ApplyDeclarationQuery(&self, q: Op) -> EditResult<Out> {
        self.dom
            .frame
            .upgrade()
            .ok_or(EditingAborted)?
            .ApplyEditingStyle(q)
            .map_err(|_| EditingAborted)
    }
    fn ApplyGeometry(&self, q: GQ) -> Option<GR> {
        match self.dom.Query(Query::ApplyStyleGeometry(q))? {
            ResultValue::ApplyStyleGeometry(r) => Some(r),
            _ => None,
        }
    }
    fn WithApplyDeclaration(
        &self,
        style: &NativeEditingStyle,
        declaration: cssom::CSSInlineStyle,
    ) -> NativeEditingStyle {
        let mut style = style.clone();
        style.declaration = declaration;
        style
    }
}
impl ApplyStyleContext for StructuralCommand {
    fn ApplyStyleQuery(&self, q: Q<Self::Style>) -> EditResult<R<Self::Style>> {
        let empty = || {
            self.ApplyDeclarationQuery(Op::Empty).and_then(|r| match r {
                Out::Declaration(d) => Ok(NativeEditingStyle::new(d)),
                _ => Err(EditingAborted),
            })
        };
        Ok(match q {
            Q::Empty => R::Style(empty()?),
            Q::Count(s) => match self.ApplyDeclarationQuery(Op::Count {
                declaration: s.declaration,
            })? {
                Out::Count(n) => R::Count(n),
                _ => return Err(EditingAborted),
            },
            Q::Extract(s, k) => match self.ApplyDeclarationQuery(Op::Extract {
                declaration: s.declaration.clone(),
                kind: match k {
                    StyleExtraction::Block => Extraction::Block,
                    StyleExtraction::Direction => Extraction::Direction,
                    StyleExtraction::Container => Extraction::Container,
                },
            })? {
                Out::Extract {
                    extracted,
                    remaining,
                } => R::Extract {
                    extracted: self.WithApplyDeclaration(&s, extracted),
                    remaining: self.WithApplyDeclaration(&s, remaining),
                },
                _ => return Err(EditingAborted),
            },
            Q::Merge(base, overriding) => {
                let declaration = match self.ApplyDeclarationQuery(Op::Merge {
                    base: base.declaration.clone(),
                    overriding: overriding.declaration.clone(),
                })? {
                    Out::Declaration(d) => d,
                    _ => return Err(EditingAborted),
                };
                let mut result = self.WithApplyDeclaration(&base, declaration);
                for (p, v, tag) in overriding.original_html_equivalent_tags {
                    result.RecordOriginalHtmlEquivalentTag(p, v, tag);
                }
                R::Style(result)
            }
            Q::RemoveProperties(s, properties) => {
                match self.ApplyDeclarationQuery(Op::RemoveProperties {
                    declaration: s.declaration.clone(),
                    properties,
                })? {
                    Out::Declaration(d) => R::Style(self.WithApplyDeclaration(&s, d)),
                    _ => return Err(EditingAborted),
                }
            }
            Q::RemovePresentProperties(s, present) => {
                match self.ApplyDeclarationQuery(Op::RemovePresentProperties {
                    declaration: s.declaration.clone(),
                    present: present.declaration,
                })? {
                    Out::Declaration(d) => R::Style(self.WithApplyDeclaration(&s, d)),
                    _ => return Err(EditingAborted),
                }
            }
            Q::Conflicts {
                node,
                style,
                matching,
                extract,
                preserve_direction,
            } => match self.ApplyDeclarationQuery(Op::Conflicts {
                node_id: node,
                declaration: style.declaration.clone(),
                matching,
                extract,
                preserve_direction,
            })? {
                Out::Conflict(r) => R::Conflict(StyleConflict {
                    implicit_element: r.implicit_element,
                    attributes: r.attributes,
                    properties: r.properties,
                    extracted: self.WithApplyDeclaration(&style, r.extracted),
                }),
                _ => return Err(EditingAborted),
            },
            Q::Present(node, s) => match self.ApplyDeclarationQuery(Op::Present {
                node_id: node,
                declaration: s.declaration,
            })? {
                Out::Bool(v) => R::Bool(v),
                _ => return Err(EditingAborted),
            },
            Q::Change(node, s) => match self.ApplyDeclarationQuery(Op::Change {
                node_id: node,
                declaration: s.declaration.clone(),
                originals: s.original_html_equivalent_tags.clone(),
                css: self.dom.editor.ShouldStyleWithCss(),
            })? {
                Out::Change(change) => R::Change(StyleChange {
                    style: self.WithApplyDeclaration(&s, change.declaration),
                    tags: change.tags,
                    font_attributes: change.font_attributes,
                }),
                _ => return Err(EditingAborted),
            },
            Q::FontDelta(s) => match self.ApplyDeclarationQuery(Op::FontDelta {
                declaration: s.declaration.clone(),
            })? {
                Out::FontDelta { delta, declaration } => R::FontDelta {
                    delta,
                    style: self.WithApplyDeclaration(&s, declaration),
                },
                _ => return Err(EditingAborted),
            },
            Q::FontSize(node) => {
                match self.ApplyDeclarationQuery(Op::FontSize { node_id: node })? {
                    Out::Number(n) => R::Number(n),
                    _ => return Err(EditingAborted),
                }
            }
            Q::SetFontSize(s, pixels) => match self.ApplyDeclarationQuery(Op::SetFontSize {
                declaration: s.declaration.clone(),
                pixels,
            })? {
                Out::Declaration(d) => R::Style(self.WithApplyDeclaration(&s, d)),
                _ => return Err(EditingAborted),
            },
            Q::Bidi(node) => match self.ApplyDeclarationQuery(Op::Bidi { node_id: node })? {
                Out::Bidi {
                    unicode_bidi,
                    direction,
                } => R::Bidi {
                    unicode_bidi,
                    direction,
                },
                _ => return Err(EditingAborted),
            },
            Q::StyleDirection(s) => match self.ApplyDeclarationQuery(Op::StyleDirection {
                declaration: s.declaration,
            })? {
                Out::Direction(d) => R::Direction(d),
                _ => return Err(EditingAborted),
            },
            Q::NormalBidi(s) => match self.ApplyDeclarationQuery(Op::NormalBidi {
                declaration: s.declaration.clone(),
            })? {
                Out::Declaration(d) => R::Style(self.WithApplyDeclaration(&s, d)),
                _ => return Err(EditingAborted),
            },
        })
    }
    fn InlineStyleForApply(&self, node: u64) -> EditResult<Self::Style> {
        if let Some(d) = ReplacementStyleContext::InlineStyle(self, node)? {
            Ok(NativeEditingStyle::new(d))
        } else {
            match self.ApplyStyleQuery(Q::Empty)? {
                R::Style(s) => Ok(s),
                _ => Err(EditingAborted),
            }
        }
    }
    fn SetApplyInlineStyle(&self, node: u64, style: &Self::Style) -> EditResult {
        ReplacementStyleContext::SetInlineStyle(self, node, &style.declaration)
    }
    fn SetApplyAttribute(
        &self,
        node: u64,
        name: &QualifiedName,
        value: Option<Rc<[u16]>>,
    ) -> EditResult {
        self.Apply(Rc::new(SetNodeAttributeCommand::new(
            self.dom.clone(),
            node,
            name.clone(),
            value.map(CommandAttributeValue::Text),
        )))
    }
    fn RemoveApplyAttribute(&self, node: u64, name: &str) -> EditResult {
        self.Apply(Rc::new(SetNodeAttributeCommand::new(
            self.dom.clone(),
            node,
            QualifiedName::new(
                Default::default(),
                foundation::AtomicString::from_str(name),
                Default::default(),
            ),
            None,
        )))
    }
    fn ApplyHasAttribute(&self, node: u64, name: &str) -> bool {
        self.dom
            .Read(node, |_, n, _| n.FindAttribute(name).is_some())
            .unwrap_or(false)
    }
    fn ApplyAttributeCount(&self, node: u64) -> usize {
        self.dom
            .Read(node, |_, n, _| n.AttributesUTF16().len())
            .unwrap_or(0)
    }
    fn ApplyIsHTMLElement(&self, node: u64) -> bool {
        self.dom
            .Read(node, |_, n, _| {
                n.Type() == DOMNodeType::kElement && n.Namespace() == dom::DOMNamespace::kHTML
            })
            .unwrap_or(false)
    }
    fn ApplySameTag(&self, a: u64, b: u64) -> bool {
        let owner = self.dom.document.borrow();
        let d = owner.GetDocument();
        match (d.FindNodeById(a), d.FindNodeById(b)) {
            (Some(a), Some(b)) => d.Node(a).TagQName() == d.Node(b).TagQName(),
            _ => false,
        }
    }
    fn ApplyIdenticalElements(&self, a: u64, b: u64) -> bool {
        ReplacementStyleContext::AreIdenticalElements(self, a, b)
    }
    fn ReplaceApplyElementWithSpan(&self, node: u64) -> EditResult<u64> {
        ReplacementStyleContext::ReplaceWithSpan(self, node)
    }
    fn MergeApplyIdenticalElements(&self, left: u64, right: u64) -> EditResult {
        self.Apply(Rc::new(MergeIdenticalElementsCommand::new(
            self.dom.clone(),
            left,
            right,
        )))
    }
    fn CreateApplyQualifiedElement(&self, name: &QualifiedName) -> EditResult<u64> {
        let node = self.dom.CreateQualified(name);
        self.dom.Read(node, |_, _, _| node).ok_or(EditingAborted)
    }
    fn SetApplyStyleSelection(&self, start: &Position<u64>, end: &Position<u64>) {
        interaction_default::core::editing::commands::replace_selection_command::TrivialReplacementContext::SetEndingReplacementSelection(self,start,end)
    }
    fn ApplyTextIndex(&self, root: u64, p: &Position<u64>) -> EditResult<usize> {
        match self.ApplyGeometry(GQ::TextIndex {
            root,
            position: CommandDOM::Native(p).ok_or(EditingAborted)?,
        }) {
            Some(GR::Index(n)) => Ok(n),
            _ => Err(EditingAborted),
        }
    }
    fn ApplyPositionAtIndex(&self, root: u64, index: usize) -> EditResult<Position<u64>> {
        match self.ApplyGeometry(GQ::PositionAtIndex { root, index }) {
            Some(GR::Position(p)) => Ok(p.map(CommandDOM::Source).unwrap_or_default()),
            _ => Err(EditingAborted),
        }
    }
    fn ApplyVisiblyContained(&self, node: u64, start: &Position<u64>, end: &Position<u64>) -> bool {
        let (Some(start), Some(end)) = (CommandDOM::Native(start), CommandDOM::Native(end)) else {
            return false;
        };
        matches!(
            self.ApplyGeometry(GQ::VisiblyContained { node, start, end }),
            Some(GR::Bool(true))
        )
    }
    fn ApplyDistinctCandidate(&self, p: &Position<u64>, forward: bool) -> Position<u64> {
        CommandDOM::Native(p)
            .and_then(|position| {
                match self.ApplyGeometry(GQ::DistinctCandidate { position, forward }) {
                    Some(GR::Position(p)) => p.map(CommandDOM::Source),
                    _ => None,
                }
            })
            .unwrap_or_default()
    }
    fn ApplyTextData(&self, node: u64) -> Vec<u16> {
        SimpleEditingDOM::Text(&*self.dom, &node).to_vec()
    }
    fn ApplyHasRenderedDescendants(&self, node: u64) -> bool {
        matches!(
            self.ApplyGeometry(GQ::HasRenderedDescendants(node)),
            Some(GR::Bool(true))
        )
    }
    fn CreateApplyDefaultParagraph(&self) -> EditResult<u64> {
        self.CreateElement(
            interaction_default::core::editing::editor::DefaultParagraphElementTag(
                self.dom.editor.DefaultParagraphSeparator(),
            ),
        )
    }
}
