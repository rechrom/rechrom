// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! replace_selection_command.cc:665-910,1168-1220. Structural decisions stay in
//! the editing command; opaque declarations are mutated by the Style owner.
use super::{
    paragraph_editing_context::{EditResult, EditingAborted},
    replace_selection_command::ReplacementFragmentDOM,
    replacement_markup::{ReplacementInsertedNodes, ReplacementMarkupContext},
    structural_editing_context::{StructuralEditingContext, StructuralPositionQuery},
};
use crate::core::editing::{
    position::{Position, PositionTree},
    text_offset_mapping::{NextNode, NextSkippingChildren},
};
use foundation::QualifiedName;
#[derive(Clone, Copy, Default)]
pub struct ReplacementWhiteSpaceFacts {
    pub has_inline_white_space: bool,
    pub has_author_white_space: bool,
    pub collapse_breaks_and_wrap: Option<(bool, bool)>,
}
pub trait ReplacementStyleContext: ReplacementMarkupContext {
    type Declaration: Clone;
    fn IsStyledElement(&self, node: u64) -> bool;
    fn IsStyleSpan(&self, node: u64) -> bool;
    fn IsEmptyFontTag(&self, node: u64) -> bool;
    fn AreIdenticalElements(&self, a: u64, b: u64) -> bool;
    fn InlineStyle(&self, node: u64) -> EditResult<Option<Self::Declaration>>;
    fn PropertyCount(&self, declaration: &Self::Declaration) -> EditResult<usize>;
    fn ImplicitStyleConflicts(
        &self,
        node: u64,
        declaration: &Self::Declaration,
    ) -> EditResult<(bool, Vec<QualifiedName>)>;
    fn RemoveStyleFromContext(
        &self,
        node: u64,
        context: Option<u64>,
        declaration: &Self::Declaration,
        rules: bool,
    ) -> EditResult<Self::Declaration>;
    fn SetInlineStyle(&self, node: u64, declaration: &Self::Declaration) -> EditResult;
    fn RemoveAttribute(&self, node: u64, name: &QualifiedName) -> EditResult;
    fn ReplaceWithSpan(&self, node: u64) -> EditResult<u64>;
    fn WhiteSpaceFacts(
        &self,
        node: u64,
        include_author_rules: bool,
    ) -> EditResult<ReplacementWhiteSpaceFacts>;
    fn WhiteSpaceNormal(&self) -> EditResult<Self::Declaration>;
    fn InsertionStyleMatchesAttribute(&self, node: u64, span: u64) -> EditResult<bool>;
    fn MoveParagraphsPreserveInlineStructure(&self) -> bool;
    fn DocumentElement(&self) -> Option<u64>;
    fn FragmentInsertBefore(&self, node: u64, reference: u64) -> EditResult;
}
fn Attr(name: &str) -> QualifiedName {
    QualifiedName::new(
        Default::default(),
        foundation::AtomicString::from_str(name),
        Default::default(),
    )
}
fn EnclosingQuote<C: StructuralEditingContext>(c: &C, node: u64) -> Option<u64> {
    let mut n = Some(node);
    while let Some(current) = n {
        if c.Tag(current, "blockquote") && c.AttributeEquals(current, "type", "cite") {
            return Some(current);
        }
        n = c.Parent(&current);
    }
    None
}
fn RemovePreservingChildren<C: ReplacementStyleContext>(c: &C, node: u64) -> EditResult {
    while let Some(child) = c.ChildAt(&node, 0) {
        c.Remove(child)?;
        c.InsertBefore(child, node)?;
    }
    c.Remove(node)
}
fn RemoveFragmentPreservingChildren<C: ReplacementStyleContext>(c: &C, node: u64) -> EditResult {
    while let Some(child) = c.ChildAt(&node, 0) {
        c.RemoveFragmentNode(child)?;
        c.FragmentInsertBefore(child, node)?;
    }
    c.RemoveFragmentNode(node)
}
/// Source :677-753. Only the original source nodes move through elementary
/// commands; the compiled white-space declaration remains a Style-owned value.
pub fn PreserveWhiteSpaceForNode<C: ReplacementStyleContext>(
    c: &C,
    node: u64,
    moving: bool,
) -> EditResult<Option<u64>> {
    if !moving {
        return Ok(None);
    }
    let Some(parent) = c
        .Parent(&node)
        .filter(|n| c.IsElement(*n) && c.NodeFacts(*n).block)
    else {
        return Ok(None);
    };
    if ["pre", "listing", "xmp", "plaintext", "style", "script"]
        .iter()
        .any(|tag| c.Tag(parent, tag))
    {
        return Ok(None);
    }
    if !c.WhiteSpaceFacts(parent, true)?.has_author_white_space {
        return Ok(None);
    }
    if c.IsElement(node) && c.WhiteSpaceFacts(node, false)?.has_inline_white_space {
        return Ok(None);
    }
    c.UpdateStyleAndLayout();
    let Some(parent_facts) = c.WhiteSpaceFacts(parent, false)?.collapse_breaks_and_wrap else {
        return Ok(None);
    };
    if let Some(root) = c.RootEditable(parent) {
        if c.WhiteSpaceFacts(root, false)?.collapse_breaks_and_wrap == Some(parent_facts) {
            return Ok(None);
        }
    }
    let span = c.CreateElement("span")?;
    c.SetInlineStyle(span, &c.WhiteSpaceNormal()?)?;
    c.InsertBefore(span, node)?;
    c.Remove(node)?;
    c.Append(node, span)?;
    Ok(Some(span))
}
/// Source :758-910. Retains the original inserted interval owner while native
/// callbacks can synchronously remove/replace nodes during attribute mutation.
pub fn RemoveRedundantStylesAndKeepStyleSpanInline<
    C: ReplacementStyleContext,
    N: ReplacementInsertedNodes,
>(
    c: &C,
    inserted: &mut N,
    moving: bool,
) -> EditResult {
    let past = inserted.PastLastLeaf(c);
    let mut node = inserted.FirstNodeInserted();
    while let Some(current) = node.filter(|n| Some(*n) != past) {
        node = NextNode(c, &current, None);
        if !c.IsStyledElement(current) {
            if let Some(span) = PreserveWhiteSpaceForNode(c, current, moving)? {
                node = NextSkippingChildren(c, &span, None);
            }
            continue;
        }
        let mut element = current;
        let mut inline = c.InlineStyle(element)?;
        let mut new_inline = inline.clone();
        let mut without_parent = None;
        if let Some(style) = inline.as_ref() {
            if c.IsHTMLElement(element) {
                let (replace, attributes) = c.ImplicitStyleConflicts(element, style)?;
                if replace {
                    let original = element;
                    element = c.ReplaceWithSpan(element)?;
                    inline = c.InlineStyle(element)?;
                    inserted.DidReplaceNode(original, element);
                } else {
                    for attribute in attributes {
                        c.RemoveAttribute(element, &attribute)?;
                    }
                }
            }
            let context = c.Parent(&element).filter(|n| c.IsElement(*n));
            c.UpdateStyleAndLayout();
            if context.and_then(|n| EnclosingQuote(c, n)).is_some() {
                new_inline = Some(c.RemoveStyleFromContext(
                    element,
                    c.DocumentElement(),
                    new_inline.as_ref().ok_or(EditingAborted)?,
                    true,
                )?);
            }
            if c.MoveParagraphsPreserveInlineStructure() && moving && c.IsStyleSpan(element) {
                if let Some(original) = inline.as_ref() {
                    without_parent =
                        Some(c.RemoveStyleFromContext(element, context, original, false)?);
                }
            }
            new_inline = Some(c.RemoveStyleFromContext(
                element,
                context,
                new_inline.as_ref().ok_or(EditingAborted)?,
                true,
            )?);
        }
        let unwrap = without_parent
            .as_ref()
            .map(|s| c.PropertyCount(s))
            .transpose()?
            .is_some_and(|n| n == 0);
        let nested = moving
            && c.Tag(element, "span")
            && c.Parent(&element).is_some_and(|n| c.Tag(n, "span"));
        let preserve = c.MoveParagraphsPreserveInlineStructure() && moving && !unwrap && !nested;
        if inline.is_none()
            || new_inline
                .as_ref()
                .map(|s| c.PropertyCount(s))
                .transpose()?
                .is_some_and(|n| n == 0)
            || unwrap
        {
            if c.IsStyleSpan(element) && !preserve || c.IsEmptyFontTag(element) {
                inserted.WillRemoveNodePreservingChildren(c, element);
                RemovePreservingChildren(c, element)?;
                continue;
            }
            c.RemoveAttribute(element, &Attr("style"))?;
        } else if c.PropertyCount(new_inline.as_ref().unwrap())?
            != c.PropertyCount(inline.as_ref().unwrap())?
        {
            c.SetInlineStyle(element, new_inline.as_ref().unwrap())?;
        }
        c.UpdateStyleAndLayout();
        if let Some(parent) = c.Parent(&element) {
            let visible = |n, start| {
                c.QueryPosition(
                    &if start {
                        Position::FirstPositionInNode(n)
                    } else {
                        Position::LastPositionInNode(c, &n)
                    },
                    StructuralPositionQuery::Visible,
                )
            };
            if c.IsHTMLElement(element)
                && c.NodeFacts(element).block
                && !c.NodeFacts(element).table_cell
                && c.AreIdenticalElements(element, parent)
                && visible(parent, true) == visible(element, true)
                && visible(parent, false) == visible(element, false)
            {
                inserted.WillRemoveNodePreservingChildren(c, element);
                RemovePreservingChildren(c, element)?;
                continue;
            }
            if c.NodeFacts(parent).richly_editable && c.NodeFacts(element).richly_editable {
                c.RemoveAttribute(element, &Attr("contenteditable"))?;
            }
        }
        if let Some(span) = PreserveWhiteSpaceForNode(c, current, moving)? {
            node = NextSkippingChildren(c, &span, None);
        }
    }
    Ok(())
}
/// Source :1168-1220. The source's final CSS text equality is read inside Style;
/// it never reparses CSS or reconstructs the incoming fragment in Interaction.
pub fn HandleStyleSpansBeforeInsertion<C: ReplacementStyleContext>(
    c: &C,
    insertion: &Position<u64>,
    fragment: u64,
    moving: bool,
    from_paste: bool,
) -> EditResult {
    let Some(top) = c.ChildAt(&fragment, 0).filter(|n| c.Tag(*n, "span")) else {
        return Ok(());
    };
    if EnclosingQuote(c, top).is_some() {
        return Ok(());
    }
    let Some(mut node) = insertion.AnchorNode().copied() else {
        return Ok(());
    };
    if c.NodeFacts(node).inline {
        let Some(block) = c.EnclosingBlock(insertion) else {
            return Ok(());
        };
        node = block;
    }
    let follow = c.ChildAt(&node, 0).is_some()
        && !(c.ChildCount(&node) == 1 && c.ChildAt(&node, 0).is_some_and(|n| c.Tag(n, "br")))
        && (c.Tag(node, "li") || c.NodeFacts(node).table_cell || c.Tag(node, "pre"));
    if !from_paste && follow {
        return RemoveFragmentPreservingChildren(c, top);
    }
    if c.MoveParagraphsPreserveInlineStructure() && moving {
        return Ok(());
    }
    let insertion = insertion.ToOffsetInAnchor(c);
    if c.InsertionStyleMatchesAttribute(
        insertion.ComputeContainerNode(c).ok_or(EditingAborted)?,
        top,
    )? {
        RemoveFragmentPreservingChildren(c, top)?;
    }
    Ok(())
}
