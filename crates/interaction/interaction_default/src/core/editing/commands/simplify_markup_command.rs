// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! simplify_markup_command.cc:50-163. The temporary work list contains original
//! native element identities; styles and the live DOM are never copied.
use super::{
    paragraph_editing_context::EditResult, structural_editing_context::StructuralEditingContext,
};
use crate::core::editing::{position::PositionTree, text_offset_mapping::NextNode};
pub trait SimplifyMarkupContext: StructuralEditingContext {
    fn InlineAlwaysCreateLineBoxes(&self, node: u64) -> Option<bool>;
    fn ComputedStylesDiffer(&self, a: u64, b: u64) -> EditResult<Option<bool>>;
    fn RemoveAssumingEditable(&self, node: u64) -> EditResult;
    fn InsertBeforeAssumingEditable(&self, node: u64, reference: u64) -> EditResult;
}
pub struct SimplifyMarkupCommand;
impl SimplifyMarkupCommand {
    pub fn Apply<C: SimplifyMarkupContext>(c: &C, first: u64, past: Option<u64>) -> EditResult {
        let root = c.Parent(&first);
        let mut remove = Vec::new();
        let mut node = Some(first);
        while let Some(current) = node.filter(|n| Some(*n) != past) {
            node = NextNode(c, &current, None);
            if c.ChildCount(&current) != 0
                || c.IsText(&current) && c.NextSibling(&current).is_some()
            {
                continue;
            }
            let Some(start) = c.Parent(&current) else {
                continue;
            };
            if c.ComputedStylesDiffer(start, start)?.is_none() {
                continue;
            }
            let mut parent = Some(start);
            let mut top = None;
            while let Some(current) = parent.filter(|n| Some(*n) != root) {
                if c.Parent(&current) != root && IsRemovableBlock(c, current) {
                    remove.push(current);
                }
                parent = c.Parent(&current);
                let Some(next) = parent else {
                    break;
                };
                if c.InlineAlwaysCreateLineBoxes(next) != Some(false) {
                    continue;
                }
                if c.ChildCount(&next) != 1 {
                    top = None;
                    break;
                }
                if c.ComputedStylesDiffer(next, start)? == Some(false) {
                    top = Some(next);
                }
            }
            if let Some(top) = top {
                let mut ancestor = Some(start);
                while let Some(current) = ancestor.filter(|n| *n != top) {
                    remove.push(current);
                    ancestor = c.Parent(&current);
                }
            }
        }
        let mut i = 0;
        while i < remove.len() {
            let Some(pruned) = Self::PruneSubsequentAncestorsToRemove(c, &remove, i)? else {
                i += 1;
                continue;
            };
            RemovePreservingChildren(c, remove[i])?;
            i += pruned + 1;
        }
        Ok(())
    }
    fn PruneSubsequentAncestorsToRemove<C: SimplifyMarkupContext>(
        c: &C,
        nodes: &[u64],
        start: usize,
    ) -> EditResult<Option<usize>> {
        let mut past = start + 1;
        while past < nodes.len() && c.Parent(&nodes[past - 1]) == Some(nodes[past]) {
            past += 1;
        }
        let highest = nodes[past - 1];
        if c.Parent(&highest).is_none() {
            return Ok(None);
        }
        if past == start + 1 {
            return Ok(Some(0));
        }
        c.RemoveAssumingEditable(nodes[start])?;
        c.InsertBeforeAssumingEditable(nodes[start], highest)?;
        c.RemoveAssumingEditable(highest)?;
        Ok(Some(past - start - 1))
    }
}
fn IsRemovableBlock<C: StructuralEditingContext>(c: &C, node: u64) -> bool {
    c.Tag(node, "div")
        && !c.HasAttributes(node)
        && c.Parent(&node)
            .is_none_or(|parent| c.ChildCount(&parent) <= 1)
}
fn RemovePreservingChildren<C: SimplifyMarkupContext>(c: &C, node: u64) -> EditResult {
    let children: Vec<_> = (0..c.ChildCount(&node))
        .filter_map(|i| c.ChildAt(&node, i))
        .collect();
    for child in children {
        c.RemoveAssumingEditable(child)?;
        c.InsertBeforeAssumingEditable(child, node)?;
    }
    c.RemoveAssumingEditable(node)
}
