// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Source apply_style_command.{h,cc}. Native nodes and the captured opaque
//! compiled declaration retain their original owners through all command phases.
use super::{
    paragraph_editing_context::{EditResult, EditingAborted},
    structural_editing_context::{StructuralEditingContext, StructuralPositionQuery as Q},
};
use crate::core::editing::{
    position::{Position, PositionDOM, PositionTree},
    text_offset_mapping::{self, NextNode, NextSkippingChildren},
};
use foundation::{CSSPropertyID as P, CSSValueID as V, QualifiedName, TextDirection};
use std::cmp::Ordering;
#[derive(Clone, Copy)]
pub enum StyleExtraction {
    Block,
    Direction,
    Container,
}
pub enum ApplyStyleQuery<S> {
    Empty,
    Count(S),
    Extract(S, StyleExtraction),
    Merge(S, S),
    RemoveProperties(S, Vec<P>),
    RemovePresentProperties(S, S),
    Conflicts {
        node: u64,
        style: S,
        matching: bool,
        extract: bool,
        preserve_direction: bool,
    },
    Present(u64, S),
    Change(u64, S),
    FontDelta(S),
    FontSize(u64),
    SetFontSize(S, f32),
    Bidi(u64),
    StyleDirection(S),
    NormalBidi(S),
}
pub struct StyleConflict<S> {
    pub implicit_element: bool,
    pub attributes: Vec<QualifiedName>,
    pub properties: Vec<P>,
    pub extracted: S,
}
pub struct StyleChange<S> {
    pub style: S,
    pub tags: Vec<QualifiedName>,
    pub font_attributes: Vec<(QualifiedName, std::rc::Rc<[u16]>)>,
}
pub enum ApplyStyleResult<S> {
    Style(S),
    Count(usize),
    Extract {
        extracted: S,
        remaining: S,
    },
    Conflict(StyleConflict<S>),
    Bool(bool),
    Change(StyleChange<S>),
    FontDelta {
        delta: f32,
        style: S,
    },
    Number(f32),
    Bidi {
        unicode_bidi: V,
        direction: Option<TextDirection>,
    },
    Direction(Option<Option<TextDirection>>),
}
/// Operations remain element-local and synchronous. The host routes declaration
/// queries to Style; it never decides the command's split/pushdown/run order.
pub trait ApplyStyleContext: StructuralEditingContext {
    fn ApplyStyleQuery(
        &self,
        query: ApplyStyleQuery<Self::Style>,
    ) -> EditResult<ApplyStyleResult<Self::Style>>;
    fn InlineStyleForApply(&self, node: u64) -> EditResult<Self::Style>;
    fn SetApplyInlineStyle(&self, node: u64, style: &Self::Style) -> EditResult;
    fn SetApplyAttribute(
        &self,
        node: u64,
        name: &QualifiedName,
        value: Option<std::rc::Rc<[u16]>>,
    ) -> EditResult;
    fn RemoveApplyAttribute(&self, node: u64, name: &str) -> EditResult;
    fn ApplyHasAttribute(&self, node: u64, name: &str) -> bool;
    fn ApplyAttributeCount(&self, node: u64) -> usize;
    fn ApplyIsHTMLElement(&self, node: u64) -> bool;
    fn ApplySameTag(&self, a: u64, b: u64) -> bool;
    fn ApplyIdenticalElements(&self, a: u64, b: u64) -> bool;
    fn ReplaceApplyElementWithSpan(&self, node: u64) -> EditResult<u64>;
    fn MergeApplyIdenticalElements(&self, left: u64, right: u64) -> EditResult;
    fn CreateApplyQualifiedElement(&self, name: &QualifiedName) -> EditResult<u64>;
    fn SetApplyStyleSelection(&self, start: &Position<u64>, end: &Position<u64>);
    fn ApplyTextIndex(&self, root: u64, p: &Position<u64>) -> EditResult<usize>;
    fn ApplyPositionAtIndex(&self, root: u64, index: usize) -> EditResult<Position<u64>>;
    fn ApplyVisiblyContained(&self, node: u64, start: &Position<u64>, end: &Position<u64>) -> bool;
    fn ApplyDistinctCandidate(&self, p: &Position<u64>, forward: bool) -> Position<u64>;
    fn ApplyTextData(&self, node: u64) -> Vec<u16>;
    fn ApplyHasRenderedDescendants(&self, node: u64) -> bool;
    fn CreateApplyDefaultParagraph(&self) -> EditResult<u64>;
}
fn Style<C: ApplyStyleContext>(c: &C, q: ApplyStyleQuery<C::Style>) -> EditResult<C::Style> {
    match c.ApplyStyleQuery(q)? {
        ApplyStyleResult::Style(s) => Ok(s),
        _ => Err(EditingAborted),
    }
}
fn Count<C: ApplyStyleContext>(c: &C, s: &C::Style) -> EditResult<usize> {
    match c.ApplyStyleQuery(ApplyStyleQuery::Count(s.clone()))? {
        ApplyStyleResult::Count(n) => Ok(n),
        _ => Err(EditingAborted),
    }
}
fn Extract<C: ApplyStyleContext>(
    c: &C,
    s: &C::Style,
    k: StyleExtraction,
) -> EditResult<(C::Style, C::Style)> {
    match c.ApplyStyleQuery(ApplyStyleQuery::Extract(s.clone(), k))? {
        ApplyStyleResult::Extract {
            extracted,
            remaining,
        } => Ok((extracted, remaining)),
        _ => Err(EditingAborted),
    }
}
fn Contains<C: PositionTree<Node = u64>>(c: &C, parent: u64, node: u64) -> bool {
    parent == node || text_offset_mapping::IsDescendantOf(c, &node, &parent)
}
fn First<C: PositionDOM<Node = u64>>(c: &C, n: u64) -> Position<u64> {
    Position::FirstPositionInOrBeforeNode(c, &n)
}
fn Last<C: PositionDOM<Node = u64>>(c: &C, n: u64) -> Position<u64> {
    Position::LastPositionInOrAfterNode(c, &n)
}
fn Before<C: PositionTree<Node = u64>>(c: &C, n: u64) -> Position<u64> {
    Position::BeforeNode(n).ToOffsetInAnchor(c)
}
fn After<C: PositionTree<Node = u64>>(c: &C, n: u64) -> Position<u64> {
    Position::AfterNode(n).ToOffsetInAnchor(c)
}
fn Less<C: ApplyStyleContext>(c: &C, a: &Position<u64>, b: &Position<u64>) -> bool {
    c.Compare(a, b) == Some(Ordering::Less)
}
fn AttrName(name: &str) -> QualifiedName {
    QualifiedName::new(
        Default::default(),
        foundation::AtomicString::from_str(name),
        Default::default(),
    )
}
fn Children<C: PositionTree<Node = u64>>(c: &C, n: u64) -> Vec<u64> {
    (0..c.ChildCount(&n))
        .filter_map(|i| c.ChildAt(&n, i))
        .collect()
}
fn RemovePreserving<C: ApplyStyleContext>(c: &C, n: u64) -> EditResult {
    for child in Children(c, n) {
        c.Remove(child)?;
        c.InsertBefore(child, n)?;
    }
    c.Remove(n)
}
fn NoAttrsOrEmptyStyle<C: ApplyStyleContext>(c: &C, n: u64) -> EditResult<bool> {
    Ok(c.ApplyAttributeCount(n) == 0
        || (c.ApplyAttributeCount(n) == 1
            && c.ApplyHasAttribute(n, "style")
            && Count(c, &c.InlineStyleForApply(n)?)? == 0))
}
fn StyleSpan<C: ApplyStyleContext>(c: &C, n: u64) -> bool {
    c.Tag(n, "span")
        && (c.ApplyAttributeCount(n) == 0
            || c.ApplyAttributeCount(n) == 1 && c.ApplyHasAttribute(n, "style"))
}
fn UnstyledSpan<C: ApplyStyleContext>(c: &C, n: u64) -> EditResult<bool> {
    Ok(c.Tag(n, "span") && NoAttrsOrEmptyStyle(c, n)?)
}
fn AllEditable<C: ApplyStyleContext>(c: &C, n: u64) -> bool {
    let past = NextSkippingChildren(c, &n, None);
    let mut node = Some(n);
    while let Some(n) = node.filter(|n| Some(*n) != past) {
        if !c.Editable(n) {
            return false;
        }
        node = NextNode(c, &n, None);
    }
    true
}
#[derive(Clone, Copy, PartialEq)]
enum Removal {
    IfNeeded,
    Always,
    None,
}
pub struct ApplyStyleCommand<S: Clone> {
    style: S,
    start: Position<u64>,
    end: Position<u64>,
    styled: Option<u64>,
    remove_only: bool,
    force_block: bool,
    remove_format: bool,
}
pub fn ApplyStyleRange<C: ApplyStyleContext>(
    c: &C,
    style: &C::Style,
    start: &Position<u64>,
    end: &Position<u64>,
) -> EditResult {
    ApplyStyleCommand {
        style: style.clone(),
        start: start.clone(),
        end: end.clone(),
        styled: None,
        remove_only: false,
        force_block: false,
        remove_format: false,
    }
    .DoApply(c)
}
pub fn ApplyStyledElement<C: ApplyStyleContext>(
    c: &C,
    element: u64,
    start: &Position<u64>,
    end: &Position<u64>,
) -> EditResult {
    let empty = Style(c, ApplyStyleQuery::Empty)?;
    ApplyStyleCommand {
        style: empty,
        start: c.QueryPosition(start, Q::Forward),
        end: c.QueryPosition(end, Q::Backward),
        styled: Some(element),
        remove_only: false,
        force_block: false,
        remove_format: false,
    }
    .DoApply(c)
}
/// ApplyStyleCommand's remove-only styled-element constructor, used by Unlink.
pub fn RemoveStyledElement<C: ApplyStyleContext>(
    c: &C,
    element: u64,
    start: &Position<u64>,
    end: &Position<u64>,
) -> EditResult {
    ApplyStyleCommand {
        style: Style(c, ApplyStyleQuery::Empty)?,
        start: c.QueryPosition(start, Q::Forward),
        end: c.QueryPosition(end, Q::Backward),
        styled: Some(element),
        remove_only: true,
        force_block: false,
        remove_format: false,
    }
    .DoApply(c)
}
/// remove_format_command.cc:83-101 passes a predicate to the same ApplyStyle.
pub fn RemoveFormatRange<C: ApplyStyleContext>(
    c: &C,
    style: C::Style,
    start: &Position<u64>,
    end: &Position<u64>,
) -> EditResult {
    ApplyStyleCommand {
        style,
        start: c.QueryPosition(start, Q::Forward),
        end: c.QueryPosition(end, Q::Backward),
        styled: None,
        remove_only: true,
        force_block: false,
        remove_format: true,
    }
    .DoApply(c)
}
impl<S: Clone> ApplyStyleCommand<S> {
    fn Update<C: ApplyStyleContext<Style = S>>(
        &mut self,
        c: &C,
        start: Position<u64>,
        end: Position<u64>,
    ) {
        c.UpdateStyleAndLayout();
        c.SetApplyStyleSelection(&start, &end);
        self.start = start;
        self.end = end;
    }
    pub fn DoApply<C: ApplyStyleContext<Style = S>>(&mut self, c: &C) -> EditResult {
        if self.start.IsNull() || self.end.IsNull() {
            return Err(EditingAborted);
        }
        if self.force_block {
            return self.ApplyBlockStyle(c, &self.style.clone());
        }
        let (block, inline) = Extract(c, &self.style, StyleExtraction::Block)?;
        self.style = inline;
        if Count(c, &block)? != 0 {
            self.ApplyBlockStyle(c, &block)?;
        }
        if Count(c, &self.style)? != 0 || self.styled.is_some() || self.remove_format {
            self.ApplyRelativeFontStyleChange(c)?;
            self.ApplyInlineStyle(c, &self.style.clone())?;
        }
        Ok(())
    }
    fn Sorted<C: ApplyStyleContext<Style = S>>(&self, c: &C) -> (Position<u64>, Position<u64>) {
        if Less(c, &self.end, &self.start) {
            (self.end.clone(), self.start.clone())
        } else {
            (self.start.clone(), self.end.clone())
        }
    }
    fn Full<C: ApplyStyleContext<Style = S>>(
        &self,
        c: &C,
        n: u64,
        start: &Position<u64>,
        end: &Position<u64>,
    ) -> bool {
        c.UpdateStyleAndLayout();
        !Less(c, &First(c, n), start) && !Less(c, end, &c.QueryPosition(&Last(c, n), Q::Backward))
    }
    fn Change<C: ApplyStyleContext<Style = S>>(
        &self,
        c: &C,
        style: &S,
        node: u64,
    ) -> EditResult<StyleChange<S>> {
        match c.ApplyStyleQuery(ApplyStyleQuery::Change(node, style.clone()))? {
            ApplyStyleResult::Change(change) => Ok(change),
            _ => Err(EditingAborted),
        }
    }
    fn StyledToRemove<C: ApplyStyleContext<Style = S>>(&self, c: &C, node: u64) -> bool {
        self.styled
            .is_some_and(|original| c.ApplySameTag(original, node))
            || self.remove_format
                && [
                    "acronym", "b", "bdo", "big", "cite", "code", "dfn", "em", "font", "i", "ins",
                    "kbd", "nobr", "q", "s", "samp", "small", "strike", "strong", "sub", "sup",
                    "tt", "u", "var",
                ]
                .iter()
                .any(|tag| c.Tag(node, tag))
    }
    fn Conflict<C: ApplyStyleContext<Style = S>>(
        &self,
        c: &C,
        style: &S,
        node: u64,
        mode: Removal,
        extract: bool,
    ) -> EditResult<StyleConflict<S>> {
        match c.ApplyStyleQuery(ApplyStyleQuery::Conflicts {
            node,
            style: style.clone(),
            matching: mode == Removal::Always,
            extract,
            preserve_direction: extract,
        })? {
            ApplyStyleResult::Conflict(r) => Ok(r),
            _ => Err(EditingAborted),
        }
    }
    fn RemoveFromElement<C: ApplyStyleContext<Style = S>>(
        &self,
        c: &C,
        style: &S,
        node: u64,
        mode: Removal,
        extracted: Option<&mut S>,
    ) -> EditResult<bool> {
        c.UpdateStyleAndLayout();
        if c.Parent(&node).is_none_or(|n| !c.Editable(n)) {
            return Ok(false);
        }
        let mut extracted = extracted;
        if self.StyledToRemove(c, node) {
            if mode == Removal::None {
                return Ok(true);
            }
            if let Some(out) = extracted.as_deref_mut() {
                *out = Style(
                    c,
                    ApplyStyleQuery::Merge(out.clone(), c.InlineStyleForApply(node)?),
                )?;
            }
            RemovePreserving(c, node)?;
            return Ok(true);
        }
        let result = self.Conflict(c, style, node, mode, extracted.is_some())?;
        if mode == Removal::None {
            return Ok(result.implicit_element
                || !result.attributes.is_empty()
                || !result.properties.is_empty());
        }
        if let Some(out) = extracted {
            *out = Style(c, ApplyStyleQuery::Merge(out.clone(), result.extracted))?;
        }
        let removed = result.implicit_element
            || !result.attributes.is_empty()
            || !result.properties.is_empty();
        if result.implicit_element {
            if NoAttrsOrEmptyStyle(c, node)? {
                RemovePreserving(c, node)?;
            } else {
                c.ReplaceApplyElementWithSpan(node)?;
            }
        } else {
            for name in result.attributes {
                c.SetApplyAttribute(node, &name, None)?;
            }
            if (c.Tag(node, "font") || c.Tag(node, "span")) && NoAttrsOrEmptyStyle(c, node)? {
                RemovePreserving(c, node)?;
            }
        }
        if !c.IsConnected(&node) {
            return Ok(removed);
        }
        if !result.properties.is_empty() {
            let style = Style(
                c,
                ApplyStyleQuery::RemoveProperties(c.InlineStyleForApply(node)?, result.properties),
            )?;
            c.SetApplyInlineStyle(node, &style)?;
            if UnstyledSpan(c, node)? {
                RemovePreserving(c, node)?;
            }
        }
        Ok(removed)
    }
    fn Surround<C: ApplyStyleContext<Style = S>>(
        &self,
        c: &C,
        start: u64,
        end: u64,
        wrapper: u64,
        merge: bool,
    ) -> EditResult {
        c.InsertBefore(wrapper, start)?;
        c.UpdateStyleAndLayout();
        let mut node = Some(start);
        while let Some(n) = node {
            let next = c.NextSibling(&n);
            if c.Editable(n) {
                c.Remove(n)?;
                c.Append(n, wrapper)?;
            }
            if n == end {
                break;
            }
            node = next;
        }
        if !merge {
            return Ok(());
        }
        let next = c.NextSibling(&wrapper);
        let previous = c.PreviousSibling(&wrapper);
        if let Some(next) = next
            .filter(|n| c.IsElement(*n) && c.Editable(*n) && c.ApplyIdenticalElements(wrapper, *n))
        {
            c.MergeApplyIdenticalElements(wrapper, next)?;
        }
        if let Some(previous) = previous.filter(|n| c.IsElement(*n) && c.Editable(*n)) {
            if let Some(merged) = c.NextSibling(&previous).filter(|n| {
                c.IsElement(*n) && c.Editable(*n) && c.ApplyIdenticalElements(previous, *n)
            }) {
                c.MergeApplyIdenticalElements(previous, merged)?;
            }
        }
        Ok(())
    }
    fn ApplyChange<C: ApplyStyleContext<Style = S>>(
        &self,
        c: &C,
        mut start: u64,
        mut end: u64,
        change: StyleChange<S>,
        add_styled: bool,
    ) -> EditResult {
        if !c.IsConnected(&start) || !c.IsConnected(&end) {
            return Ok(());
        }
        let mut font = None;
        let mut style_container = None;
        let mut container = Some(start);
        while let Some(n) = container.filter(|_| start == end) {
            if c.Tag(n, "font") {
                font = Some(n);
            }
            if c.ApplyIsHTMLElement(n)
                && (c.Tag(n, "span")
                    || style_container.is_none_or(|x| !c.Tag(x, "span")) && c.ChildCount(&n) != 0)
            {
                style_container = Some(n);
            }
            if c.ChildCount(&n) == 0 {
                break;
            }
            start = c.ChildAt(&n, 0).ok_or(EditingAborted)?;
            end = c.ChildAt(&n, c.ChildCount(&n) - 1).ok_or(EditingAborted)?;
            container = Some(start);
        }
        if !change.font_attributes.is_empty() {
            let node = match font {
                Some(n) => n,
                None => c.CreateElement("font")?,
            };
            for (name, value) in change.font_attributes {
                c.SetApplyAttribute(node, &name, Some(value))?;
            }
            if font.is_none() {
                self.Surround(c, start, end, node, true)?;
            }
        }
        if Count(c, &change.style)? != 0 {
            if let Some(container) = style_container {
                let merged = Style(
                    c,
                    ApplyStyleQuery::Merge(c.InlineStyleForApply(container)?, change.style),
                )?;
                c.SetApplyInlineStyle(container, &merged)?;
            } else {
                let span = c.CreateElement("span")?;
                c.SetApplyInlineStyle(span, &change.style)?;
                self.Surround(c, start, end, span, true)?;
            }
        }
        // StyleChange returns outside-to-inside tags; source surrounds in the
        // reverse order, leaving each later source tag outside the same run.
        for name in change.tags.iter().rev() {
            let node = c.CreateApplyQualifiedElement(name)?;
            self.Surround(c, start, end, node, true)?;
        }
        if add_styled {
            if let Some(element) = self.styled {
                let clone = c.CloneWithoutChildren(element)?;
                self.Surround(c, start, end, clone, true)?;
            }
        }
        Ok(())
    }
    fn PositionForChange<C: ApplyStyleContext<Style = S>>(
        &self,
        c: &C,
        start: u64,
    ) -> EditResult<(Position<u64>, Option<u64>)> {
        if c.IsElement(start) {
            return Ok((First(c, start), None));
        }
        let dummy = c.CreateElement("span")?;
        super::composite_edit_command::InsertNodeAt(c, dummy, &Position::BeforeNode(start))?;
        Ok((Position::BeforeNode(dummy), Some(dummy)))
    }
    fn AddInlineIfNeeded<C: ApplyStyleContext<Style = S>>(
        &self,
        c: &C,
        style: &S,
        start: u64,
        end: u64,
    ) -> EditResult {
        if !c.IsConnected(&start) || !c.IsConnected(&end) {
            return Ok(());
        }
        let (p, dummy) = self.PositionForChange(c, start)?;
        c.UpdateStyleAndLayout();
        let change = self.Change(c, style, *p.AnchorNode().ok_or(EditingAborted)?)?;
        if let Some(dummy) = dummy {
            c.Remove(dummy)?;
        }
        self.ApplyChange(c, start, end, change, false)
    }
    fn PushStyle<C: ApplyStyleContext<Style = S>>(
        &self,
        c: &C,
        node: u64,
        style: &S,
    ) -> EditResult {
        c.UpdateStyleAndLayout();
        let facts = c.NodeFacts(node);
        if Count(c, style)? == 0 || !facts.has_layout || c.Tag(node, "iframe") {
            return Ok(());
        }
        let style = if c.ApplyIsHTMLElement(node) {
            Style(
                c,
                ApplyStyleQuery::Merge(style.clone(), c.InlineStyleForApply(node)?),
            )?
        } else {
            style.clone()
        };
        if c.ApplyIsHTMLElement(node) && (facts.block_flow || c.ChildCount(&node) != 0) {
            return c.SetApplyInlineStyle(node, &style);
        }
        if c.IsText(&node) && facts.insignificant_text {
            return Ok(());
        }
        self.AddInlineIfNeeded(c, &style, node, node)
    }
    fn HighestConflicting<C: ApplyStyleContext<Style = S>>(
        &self,
        c: &C,
        style: &S,
        node: u64,
    ) -> EditResult<Option<u64>> {
        let mut root = c.RootEditable(node);
        let mut n = Some(node);
        while let Some(current) = n {
            if c.NodeFacts(current).table_cell {
                root = Some(current);
                break;
            }
            if Some(current) == root {
                break;
            }
            n = c.Parent(&current);
        }
        let mut highest = None;
        let mut n = Some(node);
        while let Some(current) = n {
            if c.ApplyIsHTMLElement(current)
                && self.RemoveFromElement(c, style, current, Removal::None, None)?
            {
                highest = Some(current);
            }
            if Some(current) == root {
                break;
            }
            n = c.Parent(&current);
        }
        Ok(highest)
    }
    fn PushDown<C: ApplyStyleContext<Style = S>>(
        &self,
        c: &C,
        style: &S,
        target: u64,
    ) -> EditResult {
        let Some(highest) = self.HighestConflicting(c, style, target)? else {
            return Ok(());
        };
        let (_, filtered) = Extract(c, style, StyleExtraction::Container)?;
        let mut current = highest;
        let mut elements = Vec::new();
        while current != target && Contains(c, current, target) {
            let children = Children(c, current);
            let styled = self.StyledToRemove(c, current);
            if styled {
                elements.push(current);
            }
            let mut pushed = Style(c, ApplyStyleQuery::Empty)?;
            if c.ApplyIsHTMLElement(current) {
                self.RemoveFromElement(
                    c,
                    if c.NodeFacts(current).block {
                        &filtered
                    } else {
                        style
                    },
                    current,
                    Removal::IfNeeded,
                    Some(&mut pushed),
                )?;
            }
            let mut next = None;
            for child in children {
                if c.Parent(&child).is_none() {
                    continue;
                }
                if !Contains(c, child, target) {
                    for original in &elements {
                        let wrapper = c.CloneWithoutChildren(*original)?;
                        c.RemoveApplyAttribute(wrapper, "style")?;
                        c.RemoveApplyAttribute(*original, "id")?;
                        if c.Tag(*original, "a") {
                            c.RemoveApplyAttribute(*original, "name")?;
                        }
                        self.Surround(c, child, child, wrapper, true)?;
                    }
                }
                if child != target || styled {
                    self.PushStyle(c, child, &pushed)?;
                }
                if Contains(c, child, target) {
                    next = Some(child);
                }
            }
            let Some(n) = next else {
                break;
            };
            current = n;
        }
        Ok(())
    }
    fn RemoveInline<C: ApplyStyleContext<Style = S>>(
        &mut self,
        c: &C,
        style: &S,
        start: &Position<u64>,
        end: &Position<u64>,
    ) -> EditResult {
        c.UpdateStyleAndLayout();
        let mut push_start = c.QueryPosition(start, Q::Forward);
        if push_start.ComputeContainerNode(c).is_some_and(|n| {
            c.IsText(&n)
                && c.CharacterDataLength(&n) == Some(push_start.ComputeOffsetInContainerNode(c))
        }) {
            push_start = c.ApplyDistinctCandidate(&push_start, true);
        }
        c.UpdateStyleAndLayout();
        let mut push_end = c.QueryPosition(end, Q::Backward);
        if push_end
            .ComputeContainerNode(c)
            .is_some_and(|n| c.IsText(&n) && push_end.ComputeOffsetInContainerNode(c) == 0)
        {
            push_end = c.ApplyDistinctCandidate(&push_end, false);
        }
        if let Some(n) = push_start.AnchorNode() {
            self.PushDown(c, style, *n)?;
        }
        if let Some(n) = push_end.AnchorNode() {
            self.PushDown(c, style, *n)?;
        }
        let mut start = if start.IsConnected(c) {
            start.clone()
        } else {
            push_start
        };
        let mut end = if end.IsConnected(c) {
            end.clone()
        } else {
            push_end
        };
        if start.IsNull() || end.IsNull() {
            return Ok(());
        }
        let original_start = start.clone();
        let original_end = end.clone();
        let (_, filtered) = Extract(c, style, StyleExtraction::Container)?;
        let mut node = start.AnchorNode().copied();
        while let Some(n) = node {
            let next = if !c.EditingIgnoresContent(&n) {
                NextNode(c, &n, None)
            } else if end.AnchorNode().is_none_or(|end| !Contains(c, n, *end)) {
                NextSkippingChildren(c, &n, None)
            } else {
                None
            };
            if c.ApplyIsHTMLElement(n) && self.Full(c, n, &original_start, &original_end) {
                let before = PreviousPostOrder(c, n);
                let after = NextNode(c, &n, None);
                let children = if self.StyledToRemove(c, n) {
                    Children(c, n)
                } else {
                    Vec::new()
                };
                let mut pushed = Style(c, ApplyStyleQuery::Empty)?;
                self.RemoveFromElement(
                    c,
                    if c.NodeFacts(n).block {
                        &filtered
                    } else {
                        style
                    },
                    n,
                    Removal::IfNeeded,
                    (!children.is_empty()).then_some(&mut pushed),
                )?;
                if !c.IsConnected(&n) {
                    if start.AnchorNode() == Some(&n) {
                        if start == end {
                            start = after.map(|n| Before(c, n)).unwrap_or_default();
                            end = start.clone();
                        } else {
                            start = after.map(|n| First(c, n)).unwrap_or_default();
                        }
                    }
                    if end.AnchorNode() == Some(&n) {
                        end = before.map(|n| Last(c, n)).unwrap_or_default();
                    }
                }
                if !children.is_empty() {
                    let pushed = Style(
                        c,
                        ApplyStyleQuery::RemovePresentProperties(pushed, style.clone()),
                    )?;
                    for child in children {
                        self.PushStyle(c, child, &pushed)?;
                    }
                }
            }
            if original_end.AnchorNode() == Some(&n) {
                break;
            }
            node = next;
        }
        self.Update(c, start, end);
        Ok(())
    }
    fn ValidTextCaret<C: ApplyStyleContext<Style = S>>(&self, c: &C, p: &Position<u64>) -> bool {
        p.IsOffsetInAnchor()
            && p.ComputeContainerNode(c).is_some_and(|n| {
                let f = c.NodeFacts(n);
                c.IsText(&n)
                    && p.ComputeOffsetInContainerNode(c) > f.caret_min
                    && p.ComputeOffsetInContainerNode(c) < f.caret_max
            })
    }
    fn SplitContaining<C: ApplyStyleContext<Style = S>>(
        &self,
        c: &C,
        text: u64,
        offset: usize,
    ) -> EditResult {
        c.SplitText(text, offset)?;
        let Some(mut parent) = c.Parent(&text).filter(|n| c.IsElement(*n)) else {
            return Ok(());
        };
        if c.Parent(&parent)
            .is_none_or(|n| !c.IsElement(n) || !c.Editable(n))
        {
            return Ok(());
        }
        let facts = c.NodeFacts(parent);
        if !facts.has_layout || !facts.inline {
            let dummy = c.CreateElement("span")?;
            for child in Children(c, parent) {
                c.Remove(child)?;
                c.Append(child, dummy)?;
            }
            c.Append(dummy, parent)?;
            parent = dummy;
        }
        c.SplitElement(parent, text)
    }
    fn SplitStart<C: ApplyStyleContext<Style = S>>(&mut self, c: &C, element: bool) -> EditResult {
        let node = self.start.ComputeContainerNode(c).ok_or(EditingAborted)?;
        let offset = self.start.ComputeOffsetInContainerNode(c);
        let end = if self.end.IsOffsetInAnchor() && self.end.ComputeContainerNode(c) == Some(node) {
            Position::new(node, self.end.ComputeOffsetInContainerNode(c) - offset)
        } else {
            self.end.clone()
        };
        if element {
            self.SplitContaining(c, node, offset)?;
        } else {
            c.SplitText(node, offset)?;
        }
        self.Update(
            c,
            if element {
                Position::BeforeNode(node)
            } else {
                Position::FirstPositionInNode(node)
            },
            end,
        );
        Ok(())
    }
    fn SplitEnd<C: ApplyStyleContext<Style = S>>(&mut self, c: &C, element: bool) -> EditResult {
        let node = self.end.ComputeContainerNode(c).ok_or(EditingAborted)?;
        let offset = self.end.ComputeOffsetInContainerNode(c);
        let same = self.start.ComputeContainerNode(c) == Some(node);
        if element {
            self.SplitContaining(c, node, offset)?;
        } else {
            c.SplitText(node, offset)?;
        }
        let previous = if element {
            c.Parent(&node)
                .and_then(|p| c.PreviousSibling(&p))
                .and_then(|p| {
                    c.ChildCount(&p)
                        .checked_sub(1)
                        .and_then(|i| c.ChildAt(&p, i))
                })
        } else {
            c.PreviousSibling(&node)
        };
        let Some(previous) = previous.filter(|n| c.IsText(n)) else {
            return Ok(());
        };
        let start = if same {
            Position::new(previous, self.start.ComputeOffsetInContainerNode(c))
        } else {
            self.start.clone()
        };
        let end = if element {
            Position::AfterNode(previous)
        } else {
            Position::LastPositionInNode(c, &previous)
        };
        self.Update(c, start, end);
        Ok(())
    }
    fn DummyAncestor<C: ApplyStyleContext<Style = S>>(
        &self,
        c: &C,
        p: &Position<u64>,
    ) -> Option<u64> {
        let mut node = p.AnchorNode().copied();
        while let Some(n) = node {
            if StyleSpan(c, n) {
                return c.Parent(&n);
            }
            node = c.Parent(&n);
        }
        None
    }
    fn Cleanup<C: ApplyStyleContext<Style = S>>(&self, c: &C, parent: Option<u64>) -> EditResult {
        if let Some(parent) = parent {
            for child in Children(c, parent) {
                if UnstyledSpan(c, child)? {
                    RemovePreserving(c, child)?;
                }
            }
        }
        Ok(())
    }
    fn MergeStart<C: ApplyStyleContext<Style = S>>(&mut self, c: &C) -> EditResult {
        let Some(mut node) = self.start.ComputeContainerNode(c) else {
            return Ok(());
        };
        if self.start.ComputeOffsetInContainerNode(c) != 0 {
            return Ok(());
        }
        if c.IsText(&node) || c.EditingIgnoresContent(&node) {
            if c.PreviousSibling(&node).is_some() {
                return Ok(());
            }
            node = c.Parent(&node).ok_or(EditingAborted)?;
        }
        if !c.IsElement(node) {
            return Ok(());
        }
        if let Some(previous) = c
            .PreviousSibling(&node)
            .filter(|p| c.ApplyIdenticalElements(node, *p))
        {
            let first = c.ChildAt(&node, 0).ok_or(EditingAborted)?;
            c.MergeApplyIdenticalElements(previous, node)?;
            let adjustment = c.NodeIndex(&first);
            let end = Position::new(
                *self.end.AnchorNode().ok_or(EditingAborted)?,
                self.end.ComputeEditingOffset(c)
                    + if self.end.AnchorNode() == Some(&node) {
                        adjustment
                    } else {
                        0
                    },
            );
            self.Update(c, Position::new(node, adjustment), end);
        }
        Ok(())
    }
    fn MergeEnd<C: ApplyStyleContext<Style = S>>(&mut self, c: &C) -> EditResult {
        let Some(mut node) = self.end.ComputeContainerNode(c) else {
            return Ok(());
        };
        if c.IsText(&node) || c.EditingIgnoresContent(&node) {
            if self.end.ComputeOffsetInContainerNode(c)
                < c.CharacterDataLength(&node).unwrap_or(c.ChildCount(&node))
                || self
                    .end
                    .AnchorNode()
                    .and_then(|n| c.NextSibling(n))
                    .is_some()
            {
                return Ok(());
            }
            node = c.Parent(&node).ok_or(EditingAborted)?;
        }
        if !c.IsElement(node) || c.Tag(node, "br") {
            return Ok(());
        }
        if let Some(next) = c
            .NextSibling(&node)
            .filter(|n| c.ApplyIdenticalElements(node, *n))
        {
            let first = c.ChildAt(&next, 0);
            c.MergeApplyIdenticalElements(node, next)?;
            let offset = first
                .map(|n| c.NodeIndex(&n))
                .unwrap_or(c.ChildCount(&next));
            let start = if self.start.ComputeContainerNode(c) == Some(node) {
                Position::new(next, self.start.ComputeOffsetInContainerNode(c))
            } else {
                self.start.clone()
            };
            self.Update(c, start, Position::new(next, offset));
        }
        Ok(())
    }
    fn Bidi<C: ApplyStyleContext<Style = S>>(
        &self,
        c: &C,
        node: u64,
    ) -> EditResult<(V, Option<TextDirection>)> {
        match c.ApplyStyleQuery(ApplyStyleQuery::Bidi(node))? {
            ApplyStyleResult::Bidi {
                unicode_bidi,
                direction,
            } => Ok((unicode_bidi, direction)),
            _ => Err(EditingAborted),
        }
    }
    fn SplitBidi<C: ApplyStyleContext<Style = S>>(
        &self,
        c: &C,
        node: u64,
        before: bool,
        allowed: Option<TextDirection>,
    ) -> EditResult<Option<u64>> {
        let Some(block) = c.EnclosingBlock(&First(c, node)) else {
            return Ok(None);
        };
        let mut highest = None;
        let mut next_highest = None;
        let mut value = V::kInvalid;
        let mut runner = c.Parent(&node);
        while let Some(n) = runner.filter(|n| *n != block) {
            if c.IsElement(n) {
                let (v, _) = self.Bidi(c, n)?;
                if v != V::kInvalid && v != V::kNormal {
                    value = v;
                    next_highest = highest;
                    highest = Some(n);
                }
            }
            runner = c.Parent(&n);
        }
        let Some(mut highest) = highest else {
            return Ok(None);
        };
        let mut unsplit = None;
        if allowed.is_some()
            && matches!(value, V::kEmbed | V::kIsolate)
            && c.ApplyIsHTMLElement(highest)
            && self.Bidi(c, highest)?.1 == allowed
        {
            if let Some(next) = next_highest {
                unsplit = Some(highest);
                highest = next;
            } else {
                return Ok(Some(highest));
            }
        }
        let mut current = node;
        loop {
            let parent = c.Parent(&current).ok_or(EditingAborted)?;
            if let Some(at) = if before {
                c.PreviousSibling(&current).map(|_| current)
            } else {
                c.NextSibling(&current)
            } {
                c.SplitElement(parent, at)?;
            }
            if parent == highest {
                break;
            }
            current = parent;
        }
        Ok(unsplit)
    }
    fn RemoveEmbedding<C: ApplyStyleContext<Style = S>>(
        &self,
        c: &C,
        node: u64,
        unsplit: Option<u64>,
    ) -> EditResult {
        let block = c.EnclosingBlock(&First(c, node));
        let mut runner = c.Parent(&node);
        while let Some(n) = runner.filter(|n| Some(*n) != block && Some(*n) != unsplit) {
            runner = c.Parent(&n);
            if !c.IsElement(n) {
                continue;
            }
            let (v, _) = self.Bidi(c, n)?;
            if v == V::kInvalid || v == V::kNormal {
                continue;
            }
            if c.ApplyHasAttribute(n, "dir") {
                c.RemoveApplyAttribute(n, "dir")?;
            } else {
                let style = Style(c, ApplyStyleQuery::NormalBidi(c.InlineStyleForApply(n)?))?;
                c.SetApplyInlineStyle(n, &style)?;
                if UnstyledSpan(c, n)? {
                    RemovePreserving(c, n)?;
                }
            }
        }
        Ok(())
    }
    fn EmbeddingAncestor<C: ApplyStyleContext<Style = S>>(
        &self,
        c: &C,
        node: u64,
    ) -> EditResult<Option<u64>> {
        let block = c.EnclosingBlock(&First(c, node));
        let mut runner = Some(node);
        while let Some(n) = runner.filter(|n| Some(*n) != block) {
            if c.ApplyIsHTMLElement(n) && matches!(self.Bidi(c, n)?.0, V::kEmbed | V::kIsolate) {
                return Ok(Some(n));
            }
            runner = c.Parent(&n);
        }
        Ok(None)
    }
    fn ApplyInlineStyle<C: ApplyStyleContext<Style = S>>(
        &mut self,
        c: &C,
        style: &S,
    ) -> EditResult {
        c.UpdateStyleAndLayout();
        let (s, e) = self.Sorted(c);
        self.start = s;
        self.end = e;
        if self.start.IsNull() || self.end.IsNull() {
            return Ok(());
        }
        let split_start = self.ValidTextCaret(c, &self.start);
        let mut start_dummy = None;
        if split_start {
            let split = self
                .start
                .AnchorNode()
                .and_then(|n| c.Parent(n))
                .filter(|n| c.ApplyIsHTMLElement(*n))
                .map(|n| self.RemoveFromElement(c, style, n, Removal::None, None))
                .transpose()?
                .unwrap_or(false);
            self.SplitStart(c, split)?;
            if split {
                start_dummy = self.DummyAncestor(c, &self.start);
            }
        }
        let split_end = self.ValidTextCaret(c, &self.end);
        let mut end_dummy = None;
        if split_end {
            let split = self
                .end
                .AnchorNode()
                .and_then(|n| c.Parent(n))
                .filter(|n| c.ApplyIsHTMLElement(*n))
                .map(|n| self.RemoveFromElement(c, style, n, Removal::None, None))
                .transpose()?
                .unwrap_or(false);
            self.SplitEnd(c, split)?;
            if split {
                end_dummy = self.DummyAncestor(c, &self.end);
            }
        }
        let remove_start = c.QueryPosition(&self.start, Q::Backward);
        let direction = match c.ApplyStyleQuery(ApplyStyleQuery::StyleDirection(style.clone()))? {
            ApplyStyleResult::Direction(d) => d,
            _ => return Err(EditingAborted),
        };
        let mut without = None;
        let mut embedding = None;
        if let Some(direction) = direction {
            let start_unsplit = self.SplitBidi(
                c,
                *self.start.AnchorNode().ok_or(EditingAborted)?,
                true,
                direction,
            )?;
            let end_unsplit = self.SplitBidi(
                c,
                *self.end.AnchorNode().ok_or(EditingAborted)?,
                false,
                direction,
            )?;
            self.RemoveEmbedding(
                c,
                *self.start.AnchorNode().ok_or(EditingAborted)?,
                start_unsplit,
            )?;
            self.RemoveEmbedding(
                c,
                *self.end.AnchorNode().ok_or(EditingAborted)?,
                end_unsplit,
            )?;
            let embedding_start = if let Some(n) =
                start_unsplit.filter(|n| self.Full(c, *n, &remove_start, &self.end))
            {
                After(c, n)
            } else {
                remove_start.clone()
            };
            let embedding_end = if let Some(n) =
                end_unsplit.filter(|n| self.Full(c, *n, &remove_start, &self.end))
            {
                c.QueryPosition(&Before(c, n), Q::Forward)
            } else {
                self.end.clone()
            };
            if embedding_end != remove_start || embedding_end != self.end {
                let (a, b) = Extract(c, style, StyleExtraction::Direction)?;
                if !Less(c, &embedding_end, &embedding_start) {
                    self.RemoveInline(c, &a, &embedding_start, &embedding_end)?;
                }
                embedding = Some(a);
                without = Some(b);
            }
        }
        self.RemoveInline(
            c,
            without.as_ref().unwrap_or(style),
            &remove_start,
            &self.end.clone(),
        )?;
        if self.start.IsNull()
            || self.end.IsNull()
            || !self.start.IsConnected(c)
            || !self.end.IsConnected(c)
        {
            return Ok(());
        }
        if split_start {
            self.MergeStart(c)?;
        }
        if split_end {
            self.MergeEnd(c)?;
        }
        c.UpdateStyleAndLayout();
        let mut apply = style.clone();
        if direction.is_some() {
            let start_embed =
                self.EmbeddingAncestor(c, *self.start.AnchorNode().ok_or(EditingAborted)?)?;
            let end_embed =
                self.EmbeddingAncestor(c, *self.end.AnchorNode().ok_or(EditingAborted)?)?;
            if start_embed.is_some() || end_embed.is_some() {
                let start = start_embed
                    .map(|n| After(c, n))
                    .unwrap_or(self.start.clone());
                let end = end_embed.map(|n| Before(c, n)).unwrap_or(self.end.clone());
                if embedding.is_none() {
                    let (a, b) = Extract(c, style, StyleExtraction::Direction)?;
                    embedding = Some(a);
                    without = Some(b);
                }
                self.FixRange(c, embedding.as_ref().ok_or(EditingAborted)?, &start, &end)?;
                apply = without.ok_or(EditingAborted)?;
            }
        }
        c.UpdateStyleAndLayout();
        self.FixRange(c, &apply, &self.start.clone(), &self.end.clone())?;
        self.Cleanup(c, start_dummy)?;
        if start_dummy != end_dummy {
            self.Cleanup(c, end_dummy)?;
        }
        Ok(())
    }
    fn FixRange<C: ApplyStyleContext<Style = S>>(
        &self,
        c: &C,
        style: &S,
        start: &Position<u64>,
        end: &Position<u64>,
    ) -> EditResult {
        let mut first = *start.AnchorNode().ok_or(EditingAborted)?;
        if start.ComputeEditingOffset(c) >= c.NodeFacts(first).caret_max {
            let Some(next) = NextNode(c, &first, None) else {
                return Ok(());
            };
            first = next;
            if Less(c, end, &First(c, first)) {
                return Ok(());
            }
        }
        let end_anchor = *end.AnchorNode().ok_or(EditingAborted)?;
        let mut past = Some(end_anchor);
        if end.ComputeEditingOffset(c) >= c.NodeFacts(end_anchor).caret_max {
            past = NextSkippingChildren(c, &end_anchor, None);
        }
        if start == end && c.Tag(end_anchor, "br") {
            past = NextNode(c, &end_anchor, None);
        }
        let root = c.RootEditable(first);
        if Some(first) != root {
            while let Some(parent) = c.Parent(&first).filter(|n| Some(*n) != root) {
                if !c.ApplyVisiblyContained(parent, start, end) {
                    break;
                }
                first = parent;
            }
        }
        self.ApplyToNodeRange(c, style, first, past)
    }
    fn ShouldApply<C: ApplyStyleContext<Style = S>>(
        &self,
        c: &C,
        style: &S,
        first: u64,
        past: Option<u64>,
    ) -> EditResult<bool> {
        let mut node = Some(first);
        while let Some(n) = node.filter(|n| Some(*n) != past) {
            node = NextNode(c, &n, None);
            if c.ChildCount(&n) != 0 {
                continue;
            }
            let present = match c.ApplyStyleQuery(ApplyStyleQuery::Present(n, style.clone()))? {
                ApplyStyleResult::Bool(v) => v,
                _ => return Err(EditingAborted),
            };
            if !present {
                return Ok(true);
            }
            if let Some(styled) = self.styled {
                let mut parent = c.Parent(&n);
                let mut found = false;
                while let Some(p) = parent {
                    if c.ApplySameTag(p, styled) {
                        found = true;
                        break;
                    }
                    parent = c.Parent(&p);
                }
                if !found {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
    fn ApplyToNodeRange<C: ApplyStyleContext<Style = S>>(
        &self,
        c: &C,
        style: &S,
        first: u64,
        past: Option<u64>,
    ) -> EditResult {
        if self.remove_only {
            return Ok(());
        }
        c.UpdateStyleAndLayout();
        let mut runs = Vec::new();
        let mut node = Some(first);
        while let Some(n) = node.filter(|n| Some(*n) != past) {
            node = NextNode(c, &n, None);
            let facts = c.NodeFacts(n);
            if !facts.has_layout || !c.Editable(n) {
                continue;
            }
            if !facts.richly_editable && c.ApplyIsHTMLElement(n) {
                if past.is_some_and(|p| text_offset_mapping::IsDescendantOf(c, &p, &n)) {
                    break;
                }
                node = NextSkippingChildren(c, &n, None);
                let merged = Style(
                    c,
                    ApplyStyleQuery::Merge(c.InlineStyleForApply(n)?, style.clone()),
                )?;
                c.SetApplyInlineStyle(n, &merged)?;
                continue;
            }
            if facts.block {
                continue;
            }
            if c.ChildCount(&n) != 0 {
                if past.is_some_and(|p| Contains(c, n, p))
                    || !AllEditable(c, n)
                    || c.Parent(&n).is_none_or(|p| !c.Editable(p))
                {
                    continue;
                }
                if c.EditingIgnoresContent(&n) {
                    node = NextSkippingChildren(c, &n, None);
                    continue;
                }
            }
            let mut end = n;
            while let Some(next) = c.NextSibling(&end).filter(|s| {
                Some(*s) != past
                    && !past.is_some_and(|p| Contains(c, *s, p))
                    && (!c.NodeFacts(*s).block || c.Tag(*s, "br"))
                    && AllEditable(c, *s)
            }) {
                end = next;
            }
            node = NextSkippingChildren(c, &end, None);
            if self.ShouldApply(c, style, n, node)? {
                runs.push((n, end, node, Position::default(), None, None));
            }
        }
        for run in &mut runs {
            let mut n = Some(run.0);
            while let Some(current) = n.filter(|n| Some(*n) != run.2 && c.IsConnected(n)) {
                n = if c.EditingIgnoresContent(&current) {
                    NextSkippingChildren(c, &current, None)
                } else {
                    NextNode(c, &current, None)
                };
                if !c.ApplyIsHTMLElement(current) {
                    continue;
                }
                let previous = c.PreviousSibling(&current);
                let next = c.NextSibling(&current);
                let parent = c.Parent(&current);
                self.RemoveFromElement(c, style, current, Removal::Always, None)?;
                if !c.IsConnected(&current) {
                    if run.0 == current {
                        run.0 = previous
                            .and_then(|n| c.NextSibling(&n))
                            .or_else(|| parent.and_then(|p| c.ChildAt(&p, 0)))
                            .ok_or(EditingAborted)?;
                    }
                    if run.1 == current {
                        run.1 = next
                            .and_then(|n| c.PreviousSibling(&n))
                            .or_else(|| {
                                parent.and_then(|p| {
                                    c.ChildCount(&p)
                                        .checked_sub(1)
                                        .and_then(|i| c.ChildAt(&p, i))
                                })
                            })
                            .ok_or(EditingAborted)?;
                    }
                }
            }
            if c.IsConnected(&run.0) && c.IsConnected(&run.1) {
                let (p, dummy) = self.PositionForChange(c, run.0)?;
                run.3 = p;
                run.4 = dummy;
            }
        }
        c.UpdateStyleAndLayout();
        for run in &mut runs {
            if let Some(node) = run.3.AnchorNode() {
                run.5 = Some(self.Change(c, style, *node)?);
            }
        }
        for run in runs {
            if let Some(dummy) = run.4 {
                c.Remove(dummy)?;
            }
            if c.IsConnected(&run.0) && c.IsConnected(&run.1) {
                if let Some(change) = run.5 {
                    self.ApplyChange(c, run.0, run.1, change, true)?;
                }
            }
        }
        Ok(())
    }
    fn JoinText<C: ApplyStyleContext<Style = S>>(&mut self, c: &C, parent: u64) -> EditResult {
        let mut start = self.start.clone();
        let mut end = self.end.clone();
        let texts: Vec<_> = Children(c, parent)
            .into_iter()
            .filter(|n| c.IsText(n))
            .collect();
        for text in texts {
            let Some(next) = c.NextSibling(&text).filter(|n| c.IsText(n)) else {
                continue;
            };
            let length = c.CharacterDataLength(&text).unwrap_or(0);
            if self.start.IsOffsetInAnchor() && self.start.ComputeContainerNode(c) == Some(next) {
                start = Position::new(text, length + self.start.ComputeOffsetInContainerNode(c));
            }
            if self.end.IsOffsetInAnchor() && self.end.ComputeContainerNode(c) == Some(next) {
                end = Position::new(text, length + self.end.ComputeOffsetInContainerNode(c));
            }
            c.InsertText(text, length, &c.ApplyTextData(next))?;
            c.Remove(next)?;
        }
        self.Update(c, start, end);
        Ok(())
    }
    fn Number<C: ApplyStyleContext<Style = S>>(&self, c: &C, node: u64) -> EditResult<f32> {
        match c.ApplyStyleQuery(ApplyStyleQuery::FontSize(node))? {
            ApplyStyleResult::Number(n) => Ok(n),
            _ => Err(EditingAborted),
        }
    }
    fn ApplyRelativeFontStyleChange<C: ApplyStyleContext<Style = S>>(
        &mut self,
        c: &C,
    ) -> EditResult {
        let delta = match c.ApplyStyleQuery(ApplyStyleQuery::FontDelta(self.style.clone()))? {
            ApplyStyleResult::FontDelta { delta, style } => {
                self.style = style;
                delta
            }
            _ => return Err(EditingAborted),
        };
        if delta == 0.0 {
            return Ok(());
        }
        let (start, end) = self.Sorted(c);
        self.start = start;
        self.end = end;
        if let Some(parent) = self
            .start
            .AnchorNode()
            .filter(|n| c.IsText(n))
            .and_then(|n| c.Parent(n))
        {
            self.JoinText(c, parent)?;
        }
        if self.start.IsNull() || self.end.IsNull() {
            return Ok(());
        }
        if let Some(parent) = self
            .end
            .AnchorNode()
            .filter(|n| c.IsText(n))
            .and_then(|n| c.Parent(n))
            .filter(|p| self.start.AnchorNode().and_then(|n| c.Parent(n)) != Some(*p))
        {
            self.JoinText(c, parent)?;
        }
        if self.ValidTextCaret(c, &self.start) {
            self.SplitStart(c, false)?;
        }
        if self.ValidTextCaret(c, &self.end) {
            self.SplitEnd(c, false)?;
        }
        let past = self.end.NodeAsRangePastLastNode(c);
        let start = c.QueryPosition(&self.start, Q::Backward);
        let Some(mut first) = start.AnchorNode().copied() else {
            return Ok(());
        };
        if Some(first) == past {
            return Ok(());
        }
        if c.IsText(&first) && start.ComputeOffsetInContainerNode(c) >= c.NodeFacts(first).caret_max
        {
            let Some(next) = NextNode(c, &first, None) else {
                return Ok(());
            };
            first = next;
        }
        let mut sizes = std::collections::HashMap::new();
        let mut node = Some(first);
        while let Some(n) = node.filter(|n| Some(*n) != past) {
            sizes.insert(n, self.Number(c, n)?);
            node = NextNode(c, &n, None);
        }
        let mut unstyled = Vec::new();
        let mut last_styled = None;
        let mut node = Some(first);
        while let Some(n) = node.filter(|n| Some(*n) != past) {
            let next = NextNode(c, &n, None);
            let element = if c.ApplyIsHTMLElement(n) {
                if !self.Full(c, n, &self.start, &self.end) {
                    node = next;
                    continue;
                }
                n
            } else if c.IsText(&n) && c.NodeFacts(n).has_layout && c.Parent(&n) != last_styled {
                let span = c.CreateElement("span")?;
                self.Surround(c, n, n, span, false)?;
                span
            } else {
                node = next;
                continue;
            };
            last_styled = Some(n);
            let inline = c.InlineStyleForApply(element)?;
            let original = self.Number(c, n)?;
            let desired = (sizes.get(&n).copied().unwrap_or(original) + delta).max(0.1);
            let stripped = Style(
                c,
                ApplyStyleQuery::RemoveProperties(inline.clone(), vec![P::kFontSize]),
            )?;
            let current = if Count(c, &inline)? != Count(c, &stripped)? {
                c.SetApplyInlineStyle(element, &stripped)?;
                self.Number(c, n)?
            } else {
                original
            };
            let result = if current != desired {
                Style(c, ApplyStyleQuery::SetFontSize(inline, desired))?
            } else {
                stripped
            };
            c.SetApplyInlineStyle(element, &result)?;
            if Count(c, &result)? == 0 {
                c.RemoveApplyAttribute(element, "style")?;
                if UnstyledSpan(c, element)? {
                    unstyled.push(element);
                }
            }
            node = next;
        }
        for span in unstyled {
            RemovePreserving(c, span)?;
        }
        Ok(())
    }
    fn NewParagraphIfNecessary<C: ApplyStyleContext<Style = S>>(
        &self,
        c: &C,
        p: &Position<u64>,
    ) -> EditResult<Option<u64>> {
        let visible = c.QueryPosition(p, Q::Visible);
        if visible.IsNull() {
            return Err(EditingAborted);
        }
        let start = c.QueryPosition(&visible, Q::ParagraphStart);
        let end = c.QueryPosition(&visible, Q::ParagraphEnd);
        let next = c.QueryPosition(&end, Q::Next);
        let visible_end = if next.IsNull() { end.clone() } else { next };
        let upstream_start = c.QueryPosition(&start, Q::Backward);
        let upstream_end = c.QueryPosition(&visible_end, Q::Backward);
        if Less(c, p, &upstream_start) {
            return Ok(None);
        }
        let mut move_contents = true;
        if let Some(start_node) = upstream_start
            .AnchorNode()
            .filter(|n| c.NodeFacts(**n).block)
        {
            if c.RootEditable(*start_node) == Some(*start_node) {
                move_contents = c.ApplyHasRenderedDescendants(*start_node);
            } else if let Some(end_node) =
                upstream_end.AnchorNode().filter(|n| c.NodeFacts(**n).block)
            {
                if !text_offset_mapping::IsDescendantOf(c, end_node, start_node) {
                    return Ok(None);
                }
            } else if c.EnclosingBlock(&upstream_end) != Some(*start_node)
                || c.QueryPosition(&visible_end, Q::NextWithinEditable)
                    .IsNull()
            {
                return Ok(None);
            }
        }
        if end.IsNull() {
            return Ok(None);
        }
        let block = c.CreateApplyDefaultParagraph()?;
        let placeholder = c.CreateElement("br")?;
        c.Append(placeholder, block)?;
        super::composite_edit_command::InsertNodeAt(c, block, &upstream_start)?;
        if !move_contents {
            return Ok(Some(block));
        }
        let end_was_br = end.AnchorNode().is_some_and(|n| c.Tag(*n, "br"));
        c.UpdateStyleAndLayout();
        let destination = c.QueryPosition(&Position::FirstPositionInNode(block), Q::Visible);
        if destination.IsNull() {
            return Err(EditingAborted);
        }
        let origin = c.QueryPosition(p, Q::Visible);
        if origin.IsNull() {
            return Err(EditingAborted);
        }
        let first = c.QueryPosition(&origin, Q::ParagraphStart);
        let last = c.QueryPosition(&origin, Q::ParagraphEnd);
        let source = c.EnclosingBlock(&first).ok_or(EditingAborted)?;
        let _ =
            super::composite_edit_command::MoveParagraph(c, &first, &last, &destination, source)?;
        if let Some(last) = c
            .ChildCount(&block)
            .checked_sub(1)
            .and_then(|i| c.ChildAt(&block, i))
            .filter(|n| c.Tag(*n, "br") && !end_was_br)
        {
            c.Remove(last)?;
        }
        Ok(Some(block))
    }
    fn ApplyBlockStyle<C: ApplyStyleContext<Style = S>>(&mut self, c: &C, style: &S) -> EditResult {
        c.UpdateStyleAndLayout();
        let (start, end) = self.Sorted(c);
        let start = c.QueryPosition(&start, Q::Visible);
        let end = c.QueryPosition(&end, Q::Visible);
        if start.IsNull() || end.IsNull() || !start.IsConnected(c) || !end.IsConnected(c) {
            return Ok(());
        }
        let mut scope = *start.AnchorNode().ok_or(EditingAborted)?;
        while let Some(parent) = c.Parent(&scope) {
            scope = parent;
        }
        let start_index = c.ApplyTextIndex(scope, &start.ToOffsetInAnchor(c))?;
        let end_index = c.ApplyTextIndex(scope, &end.ToOffsetInAnchor(c))?;
        let mut paragraph = c.QueryPosition(&start, Q::ParagraphStart);
        let beyond = c.QueryPosition(&c.QueryPosition(&end, Q::ParagraphEnd), Q::Next);
        let beyond = if beyond.IsNull() {
            None
        } else {
            Some(c.Track(&beyond)?)
        };
        while !paragraph.IsNull() {
            if let Some(beyond) = &beyond {
                let p = c.Tracked(beyond);
                if !p.IsNull() && !Less(c, &paragraph, &p) {
                    break;
                }
            }
            let next = c.QueryPosition(&c.QueryPosition(&paragraph, Q::ParagraphEnd), Q::Next);
            let next = if next.IsNull() {
                None
            } else {
                Some(c.Track(&next)?)
            };
            let change = self.Change(c, style, *paragraph.AnchorNode().ok_or(EditingAborted)?)?;
            if Count(c, &change.style)? != 0 || self.remove_only {
                let mut block = c.EnclosingBlock(&paragraph);
                if !self.remove_only && paragraph.AnchorNode().is_some_and(|n| c.Editable(*n)) {
                    if let Some(new) = self.NewParagraphIfNecessary(c, &paragraph)? {
                        block = Some(new);
                    }
                }
                if let Some(block) = block.filter(|n| c.ApplyIsHTMLElement(*n)) {
                    let conflict = self.Conflict(c, style, block, Removal::IfNeeded, false)?;
                    let stripped = Style(
                        c,
                        ApplyStyleQuery::RemoveProperties(
                            c.InlineStyleForApply(block)?,
                            conflict.properties,
                        ),
                    )?;
                    c.SetApplyInlineStyle(block, &stripped)?;
                    if !self.remove_only {
                        let merged = Style(c, ApplyStyleQuery::Merge(change.style, stripped))?;
                        c.SetApplyInlineStyle(block, &merged)?;
                    }
                }
                c.UpdateStyleAndLayout();
            }
            paragraph = next
                .as_ref()
                .map(|p| c.QueryPosition(&c.Tracked(p), Q::Visible))
                .unwrap_or_default();
        }
        c.UpdateStyleAndLayout();
        let start = c.ApplyPositionAtIndex(scope, start_index)?;
        let end = c.ApplyPositionAtIndex(scope, end_index)?;
        if !start.IsNull() && !end.IsNull() {
            self.Update(c, start, end);
        }
        Ok(())
    }
}
fn PreviousPostOrder<C: PositionTree<Node = u64>>(c: &C, node: u64) -> Option<u64> {
    if let Some(last) = c
        .ChildCount(&node)
        .checked_sub(1)
        .and_then(|i| c.ChildAt(&node, i))
    {
        return Some(last);
    }
    if let Some(previous) = c.PreviousSibling(&node) {
        return Some(previous);
    }
    let mut current = node;
    loop {
        let parent = c.Parent(&current)?;
        if let Some(previous) = c.PreviousSibling(&parent) {
            return Some(previous);
        }
        current = parent;
    }
}
