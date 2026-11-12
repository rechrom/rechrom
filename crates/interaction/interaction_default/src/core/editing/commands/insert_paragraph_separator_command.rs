// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Source insert_paragraph_separator_command.h:36-68; .cc:70-697.
use super::{
    composite_edit_command::{
        AppendPlaceholder, BreakOutOfEmptyListItem, InsertAfter, InsertNodeAt,
        MoveRemainingSiblings, SplitTreeToNode,
    },
    delete_selection_command::DeleteSelectionCommand,
    editing_commands_utilities::EnclosingListChild,
    insert_line_break_command::InsertLineBreak,
    paragraph_editing_context::{EditResult, EditingAborted},
    structural_editing_context::{StructuralEditingContext, StructuralPositionQuery},
};
use crate::core::editing::{
    commands::editing_commands_utilities::IsBlockEdge,
    commands::structural_editing_support::{LineBreakAt, ParentAfter},
    editing_utilities::IsMailQuote,
    position::Position,
    text_offset_mapping,
    visible_units_paragraph::IsParagraphEdge,
};

fn CloneHierarchy<C: StructuralEditingContext>(
    ctx: &C,
    ancestors: &[u64],
    block: u64,
) -> EditResult<u64> {
    // insert_paragraph_separator_command.cc:225-245.
    let mut parent = block;
    for ancestor in ancestors.iter().rev() {
        let child = ctx.CloneWithoutChildren(*ancestor)?;
        ctx.RemoveId(child);
        ctx.Append(child, parent)?;
        parent = child;
    }
    Ok(parent)
}
fn AncestorsInsideBlock<C: StructuralEditingContext>(ctx: &C, node: u64, block: u64) -> Vec<u64> {
    let mut result = Vec::new();
    if node != block {
        let mut parent = ctx.Parent(&node);
        while let Some(current) = parent.filter(|current| *current != block) {
            result.push(current);
            parent = ctx.Parent(&current);
        }
    }
    result
}

fn IsPhrasingContent<C: StructuralEditingContext>(ctx: &C, node: u64) -> bool {
    // insert_paragraph_separator_command.cc:70-111.
    [
        "a", "abbr", "area", "audio", "b", "bdi", "bdo", "br", "button", "canvas", "cite", "code",
        "data", "datalist", "del", "dfn", "em", "embed", "i", "iframe", "img", "input", "ins",
        "kbd", "label", "link", "map", "mark", "meta", "meter", "noscript", "object", "output",
        "picture", "progress", "q", "ruby", "s", "samp", "script", "select", "slot", "small",
        "span", "strong", "sub", "sup", "template", "textarea", "time", "u", "var", "video", "wbr",
    ]
    .iter()
    .any(|tag| ctx.Tag(node, tag))
}
pub struct InsertParagraphSeparatorCommand;
impl InsertParagraphSeparatorCommand {
    pub fn Apply<C: StructuralEditingContext>(
        ctx: &C,
        selection: (Position<u64>, Position<u64>),
    ) -> EditResult<Position<u64>> {
        Self::ApplyWithDefaultParagraph(ctx, selection, false)
    }
    pub(super) fn ApplyWithDefaultParagraph<C: StructuralEditingContext>(
        ctx: &C,
        selection: (Position<u64>, Position<u64>),
        force_default: bool,
    ) -> EditResult<Position<u64>> {
        Self::ApplyForPaste(ctx, selection, force_default, false)
    }
    pub(super) fn ApplyForPaste<C: StructuralEditingContext>(
        ctx: &C,
        selection: (Position<u64>, Position<u64>),
        force_default: bool,
        paste_quote: bool,
    ) -> EditResult<Position<u64>> {
        let mut insertion = if selection.0 != selection.1 {
            DeleteSelectionCommand::Apply(ctx, selection)?
        } else {
            selection.0
        };
        ctx.UpdateStyleAndLayout();
        insertion = ctx.QueryPosition(&insertion, StructuralPositionQuery::Visible);
        let mut block = ctx.EnclosingBlock(&insertion).ok_or(EditingAborted)?;
        let root = ctx
            .RootEditable(insertion.ComputeContainerNode(ctx).ok_or(EditingAborted)?)
            .ok_or(EditingAborted)?;
        let block_facts = ctx.NodeFacts(block);
        let initial_list_child = EnclosingListChild(ctx, &insertion);
        if ctx.Parent(&block).is_none()
            || ctx.NodeFacts(root).inline
            || IsPhrasingContent(ctx, root)
            || initial_list_child.is_some_and(|node| ctx.NodeFacts(node).inline)
            || block_facts.table_cell
            || ctx.Tag(block, "form")
            || ctx.Tag(block, "pre")
        {
            return InsertLineBreak(ctx, (insertion.clone(), insertion));
        }
        insertion = ctx.QueryPosition(&insertion, StructuralPositionQuery::Backward);
        if insertion.IsNull() {
            return Err(EditingAborted);
        }
        let style =
            if IsParagraphEdge(ctx, &insertion, true) || IsParagraphEdge(ctx, &insertion, false) {
                Some(ctx.MergeTypingStyle(ctx.CaptureStyle(&insertion, false)?)?)
            } else {
                None
            };
        if let Some(ending) = BreakOutOfEmptyListItem(ctx, &insertion)? {
            return Ok(ending);
        }
        let list_child = EnclosingListChild(ctx, &insertion);
        let first = IsBlockEdge(ctx, &insertion, true);
        let last = IsBlockEdge(ctx, &insertion, false);
        let nest = block == root;
        let default_block = force_default
            || nest
            || (last
                && ["h1", "h2", "h3", "h4", "h5"]
                    .iter()
                    .any(|tag| ctx.Tag(block, tag)));
        let new_block = if default_block {
            ctx.CreateDefaultParagraphElement()?
        } else {
            ctx.CloneWithoutChildren(block)?
        };
        // Source :373-380 moves the new unquoted paragraph outside an incoming
        // quotation while retaining the original new-block style decision.
        if last && !nest && paste_quote {
            let mut node = insertion.AnchorNode().copied();
            while let Some(n) = node {
                if IsMailQuote(ctx, n) {
                    block = n;
                }
                if n == root {
                    break;
                }
                node = ctx.Parent(&n);
            }
        }
        // insert_paragraph_separator_command.cc:367-447.
        if last {
            if nest {
                if first && !LineBreakAt(ctx, &insertion) {
                    let extra = ctx.CreateDefaultParagraphElement()?;
                    ctx.Append(extra, block)?;
                    AppendPlaceholder(ctx, extra)?;
                }
                ctx.Append(new_block, block)?;
            } else if let Some(list_child) = list_child.filter(|child| *child != block) {
                let new_item = ctx.CloneWithoutChildren(list_child)?;
                ctx.Append(new_block, new_item)?;
                InsertAfter(ctx, new_item, list_child)?;
            } else {
                let mut sibling = block;
                if ctx.Tag(new_block, "div") {
                    while ctx.NextSibling(&sibling).is_none() {
                        let Some(parent) = ctx.Parent(&sibling).filter(|parent| {
                            ctx.Tag(*parent, "div")
                                && ctx.Parent(parent).is_some()
                                && !ctx.HasAttributes(*parent)
                                && *parent != root
                        }) else {
                            break;
                        };
                        sibling = parent;
                    }
                }
                InsertAfter(ctx, new_block, sibling)?;
            }
            let ancestors =
                AncestorsInsideBlock(ctx, *insertion.AnchorNode().ok_or(EditingAborted)?, block);
            let parent = CloneHierarchy(ctx, &ancestors, new_block)?;
            AppendPlaceholder(ctx, parent)?;
            let ending = Position::FirstPositionInNode(parent);
            ctx.SetEndingPosition(&ending);
            return Ok(ending);
        }
        // insert_paragraph_separator_command.cc:449-507.
        let previous = ctx.QueryPosition(&insertion, StructuralPositionQuery::Previous);
        let different_previous_block =
            !previous.IsNull() && ctx.EnclosingBlock(&previous) != Some(block);
        if first || different_previous_block {
            let mut inserted = false;
            let reference = if first && nest {
                ctx.ChildAt(&block, 0).ok_or(EditingAborted)?
            } else if first {
                block
            } else if insertion.AnchorNode() == Some(&block) && nest {
                ctx.ChildAt(&block, insertion.ComputeEditingOffset(ctx))
                    .ok_or(EditingAborted)?
            } else {
                *insertion.AnchorNode().ok_or(EditingAborted)?
            };
            let preserved = ctx.Track(&insertion)?;
            if first && !nest {
                if let Some(list_child) = list_child.filter(|child| *child != block) {
                    let new_item = ctx.CloneWithoutChildren(list_child)?;
                    ctx.Append(new_block, new_item)?;
                    ctx.InsertBefore(new_item, list_child)?;
                    inserted = true;
                }
            }
            if !inserted {
                ctx.InsertBefore(new_block, reference)?;
            }
            let ancestors =
                AncestorsInsideBlock(ctx, *insertion.AnchorNode().ok_or(EditingAborted)?, block);
            let parent = CloneHierarchy(ctx, &ancestors, new_block)?;
            AppendPlaceholder(ctx, parent)?;
            let ending = ctx.Tracked(&preserved);
            ctx.SetEndingPosition(&ending);
            return Ok(ending);
        }
        // insert_paragraph_separator_command.cc:513-690. Split the original
        // text and ancestor chain, then move only the original right siblings.
        if IsParagraphEdge(ctx, &insertion, true) {
            let br = ctx.CreateElement("br")?;
            InsertNodeAt(ctx, br, &insertion)?;
            insertion = ParentAfter(ctx, br);
        }
        insertion = ctx.QueryPosition(&insertion, StructuralPositionQuery::Forward);
        let node = insertion.ComputeContainerNode(ctx).ok_or(EditingAborted)?;
        let mut moving = if node == block {
            insertion.ComputeNodeAfterPosition(ctx)
        } else {
            let mut split_to = node;
            if ctx.IsText(&node) {
                let offset = insertion.ComputeOffsetInContainerNode(ctx);
                let length = ctx.CharacterDataLength(&node).ok_or(EditingAborted)?;
                if offset > 0 && offset < length {
                    ctx.SplitText(node, offset)?;
                } else if offset >= length {
                    split_to = text_offset_mapping::NextNode(ctx, &node, Some(&block))
                        .ok_or(EditingAborted)?;
                }
            } else if let Some(child) = insertion.ComputeNodeAfterPosition(ctx) {
                split_to = child;
            }
            Some(SplitTreeToNode(ctx, split_to, block)?)
        };
        if nest {
            ctx.Append(new_block, block)?;
        } else if let Some(list_child) = list_child.filter(|child| *child != block) {
            let new_item = ctx.CloneWithoutChildren(list_child)?;
            ctx.Append(new_block, new_item)?;
            InsertAfter(ctx, new_item, list_child)?;
        } else {
            InsertAfter(ctx, new_block, block)?;
        }
        if moving == Some(new_block) {
            moving = None;
        }
        MoveRemainingSiblings(ctx, moving, Some(new_block), new_block)?;
        if ctx.ChildCount(&new_block) == 0 {
            AppendPlaceholder(ctx, new_block)?;
        }
        let mut ending = Position::FirstPositionInNode(new_block);
        // insert_paragraph_separator_command.cc:178-198,690. Header returns
        // deliberately drop the old heading typing style; ordinary paragraph
        // boundaries apply only the declaration difference at the new caret.
        if !["h1", "h2", "h3", "h4", "h5"]
            .iter()
            .any(|tag| ctx.Tag(block, tag))
        {
            if let Some(style) = style {
                if let Some(style) = ctx.PrepareStyle(&style, &ending)? {
                    ending = ctx.ApplyStyle(&style, &ending)?;
                }
            }
        }
        ctx.SetEndingPosition(&ending);
        Ok(ending)
    }
}
