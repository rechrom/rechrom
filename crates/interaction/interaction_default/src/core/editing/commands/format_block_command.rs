// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! format_block_command.{h,cc}. Derived range formatting uses the shared
//! ApplyBlockElementCommand lifecycle and markup-preserving paragraph cloning.
use super::{
    apply_block_element_command::{self, BlockElementFormatter},
    apply_style_command::ApplyStyleContext,
    insert_list_command::ListEditingContext,
    paragraph_cloning::{self, AtEdge, Contains},
    paragraph_editing_context::{EditResult, EditingAborted},
    structural_editing_context::StructuralPositionQuery as Q,
};
use crate::core::editing::position::Position;
use std::cell::Cell;

pub const FORMAT_BLOCK_TAGS: &[&str] = &[
    "address",
    "article",
    "aside",
    "blockquote",
    "dd",
    "div",
    "dl",
    "dt",
    "footer",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "header",
    "hgroup",
    "main",
    "nav",
    "p",
    "pre",
    "section",
];
pub trait FormatBlockContext: ListEditingContext + ApplyStyleContext {
    fn CopyFormatBlockStyle(&self, from: u64, to: u64) -> EditResult;
}
pub struct FormatBlockCommand {
    tag: String,
    did_apply: Cell<bool>,
}
impl FormatBlockCommand {
    /// editor_command.cc:671-686. All accepted qualified names are the source
    /// HTML block tag set; syntactically valid unsupported names also fail.
    pub fn FromCommandValue(value: &str) -> Option<Self> {
        let value = value.to_lowercase();
        let value = if value.starts_with('<') && value.ends_with('>') {
            &value[1..value.len() - 1]
        } else {
            &value
        };
        FORMAT_BLOCK_TAGS.contains(&value).then(|| Self {
            tag: value.into(),
            did_apply: Cell::new(false),
        })
    }
    pub fn DidApply(&self) -> bool {
        self.did_apply.get()
    }
    pub fn Apply<C: FormatBlockContext>(
        &self,
        c: &C,
        start: Position<u64>,
        end: Position<u64>,
    ) -> EditResult {
        apply_block_element_command::ApplyBlockElement(c, self, start, end)
    }
}
fn IsFormatBlock<C: ListEditingContext>(c: &C, node: u64) -> bool {
    FORMAT_BLOCK_TAGS.iter().any(|tag| c.Tag(node, tag))
}
fn EnclosingBlockToSplitTreeTo<C: ListEditingContext>(c: &C, start: u64) -> u64 {
    let mut last_block = start;
    let mut runner = Some(start);
    while let Some(node) = runner {
        if !c.Editable(node) {
            return last_block;
        }
        let parent = c.Parent(&node);
        if c.NodeFacts(node).table_cell
            || c.Tag(node, "body")
            || parent.is_none_or(|parent| !c.Editable(parent))
            || IsFormatBlock(c, node)
        {
            return node;
        }
        if c.NodeFacts(node).block {
            last_block = node;
        }
        if c.Tag(node, "ol") || c.Tag(node, "ul") {
            return parent.filter(|parent| c.Editable(*parent)).unwrap_or(node);
        }
        runner = parent;
    }
    last_block
}
impl<C: FormatBlockContext> BlockElementFormatter<C> for FormatBlockCommand {
    fn CreateBlockElement(&self, c: &C) -> EditResult<u64> {
        c.CreateElement(&self.tag)
    }
    fn FormatSelection(&self, c: &C, start: &Position<u64>, end: &Position<u64>) -> EditResult {
        let result = apply_block_element_command::FormatSelection(c, self, start, end);
        self.did_apply.set(true);
        result
    }
    fn FormatRange(
        &self,
        c: &C,
        start: &Position<u64>,
        end: &Position<u64>,
        end_of_selection: &Position<u64>,
        block: &mut Option<u64>,
        _next: &mut Option<C::Tracked>,
    ) -> EditResult {
        let visible_end = c.QueryPosition(end, Q::Visible);
        let mut reference = visible_end.AnchorNode().copied();
        while reference.is_some_and(|node| !c.NodeFacts(node).block_flow) {
            reference = reference.and_then(|node| c.Parent(&node));
        }
        let root = start
            .ComputeContainerNode(c)
            .and_then(|node| c.RootEditable(node));
        let (Some(reference), Some(root)) = (reference, root) else {
            return Ok(());
        };
        let start_node = *start.AnchorNode().ok_or(EditingAborted)?;
        let split_to = EnclosingBlockToSplitTreeTo(c, start_node);
        let outer = if start_node == split_to {
            start_node
        } else {
            super::composite_edit_command::SplitTreeToNode(c, start_node, split_to)?
        };
        let mut insertion = outer;
        c.UpdateStyleAndLayout();
        if IsFormatBlock(c, reference)
            && AtEdge(c, start, Q::BlockStart)
            && (AtEdge(c, end, Q::BlockEnd)
                || c.ApplyVisiblyContained(reference, start, end_of_selection))
            && reference != root
            && !Contains(c, reference, root)
        {
            if c.Tag(reference, &self.tag) {
                return Ok(());
            }
            insertion = reference;
        }
        let target = if let Some(block) = *block {
            block
        } else {
            let element = self.CreateBlockElement(c)?;
            c.InsertBefore(element, insertion)?;
            *block = Some(element);
            c.UpdateStyleAndLayout();
            element
        };
        let last = c
            .ChildCount(&target)
            .checked_sub(1)
            .and_then(|offset| c.ChildAt(&target, offset))
            .map(Position::AfterNode)
            .unwrap_or_default();
        let was_end = AtEdge(c, &last, Q::ParagraphEnd);
        let from = c.QueryPosition(start, Q::Visible);
        let to = c.QueryPosition(end, Q::Visible);
        if from.IsNull() || to.IsNull() {
            return Err(EditingAborted);
        }
        paragraph_cloning::MoveParagraphWithClones(c, &from, &to, target, outer)?;
        if !last.IsValidFor(c, c.DocumentIdentity(&target)) {
            return Err(EditingAborted);
        }
        if outer != insertion {
            c.CopyFormatBlockStyle(insertion, target)?;
        }
        c.UpdateStyleAndLayout();
        if was_end && !AtEdge(c, &last, Q::ParagraphEnd) && !AtEdge(c, &last, Q::ParagraphStart) {
            let br = c.CreateElement("br")?;
            super::composite_edit_command::InsertNodeAt(c, br, &last)?;
        }
        Ok(())
    }
}
