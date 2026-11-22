// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
//! The same Range::ProcessContents algorithms, parameterized only by the
//! canonical arena's primitive borrow/removal transaction.
//! Source: range.cc:529-823,858-1022.
use super::*;
use crate::tree_mutation::TreeMutation;

fn DeleteOffsets<T: TreeMutation>(
    tree: &mut T,
    node: usize,
    start: usize,
    end: usize,
) -> Result<(), DOMRangeError> {
    let character = tree.Read(|d| {
        matches!(
            d.Node(node).Type(),
            DOMNodeType::kText | DOMNodeType::kComment | DOMNodeType::kProcessingInstruction
        )
    });
    if character {
        tree.Write(|d| {
            d.ReplaceData(
                node,
                start,
                end.min(d.RangeContentsLength(node)) - start,
                "",
            )
        })?;
    } else {
        let children = tree.Read(|d| d.Node(node).Children()[start..end].to_vec());
        for child in children {
            tree.RemoveNode(child);
        }
    }
    Ok(())
}
fn DeleteAncestorsAndSiblings<T: TreeMutation>(
    tree: &mut T,
    container: usize,
    common: usize,
    forward: bool,
) {
    let ancestors = tree.Read(|d| {
        let mut ancestors = Vec::new();
        let mut current = d.Node(container).Parent();
        while let Some(ancestor) = current {
            if ancestor == common {
                break;
            }
            ancestors.push(ancestor);
            current = d.Node(ancestor).Parent();
        }
        ancestors
    });
    let mut descendant = container;
    for ancestor in ancestors {
        let mut siblings = tree.Read(|d| {
            let index = d.RangeNodeIndex(descendant);
            let children = d.Node(ancestor).Children();
            if forward {
                children[index + 1..].to_vec()
            } else {
                children[..index].to_vec()
            }
        });
        if !forward {
            siblings.reverse();
        }
        for sibling in siblings {
            if tree.Read(|d| d.Node(sibling).Parent() == Some(ancestor)) {
                tree.RemoveNode(sibling);
            }
        }
        descendant = ancestor;
    }
}
pub(crate) fn DeleteContents<T: TreeMutation>(
    tree: &mut T,
    range: &DOMRangeHandle,
) -> Result<(), DOMRangeError> {
    let snapshot = tree
        .Read(|d| d.Range(range))
        .ok_or(DOMRangeError::WrongDocument)?;
    if snapshot.Collapsed() {
        return Ok(());
    }
    let (start, end) = tree.Read(|d| {
        Ok::<_, DOMRangeError>((
            d.FindNodeById(snapshot.start.node_id)
                .ok_or(DOMRangeError::MissingNode)?,
            d.FindNodeById(snapshot.end.node_id)
                .ok_or(DOMRangeError::MissingNode)?,
        ))
    })?;
    if start == end {
        return DeleteOffsets(tree, start, snapshot.start.offset, snapshot.end.offset);
    }
    let (common, partial_start, partial_end, process_start, process_end) = tree.Read(|d| {
        let mut common = start;
        while !d.RangeContainsNode(common, end) {
            common = d
                .Node(common)
                .Parent()
                .ok_or(DOMRangeError::WrongDocument)?;
        }
        let highest = |mut node| {
            if node == common {
                return None;
            }
            while d.Node(node).Parent() != Some(common) {
                node = d.Node(node).Parent().unwrap();
            }
            Some(node)
        };
        let partial_start = highest(start);
        let partial_end = highest(end);
        let process_start = if let Some(partial) = partial_start {
            d.RangeNextSibling(partial)
        } else {
            d.Node(common)
                .Children()
                .get(snapshot.start.offset)
                .copied()
        };
        let process_end =
            partial_end.or_else(|| d.Node(common).Children().get(snapshot.end.offset).copied());
        Ok::<_, DOMRangeError>((
            common,
            partial_start,
            partial_end,
            process_start,
            process_end,
        ))
    })?;
    if start != common && tree.Read(|d| d.RangeContainsNode(common, start)) {
        let length = tree.Read(|d| d.RangeContentsLength(start));
        DeleteOffsets(tree, start, snapshot.start.offset, length)?;
        DeleteAncestorsAndSiblings(tree, start, common, true);
    }
    if end != common && tree.Read(|d| d.RangeContainsNode(common, end)) {
        DeleteOffsets(tree, end, 0, snapshot.end.offset)?;
        DeleteAncestorsAndSiblings(tree, end, common, false);
    }
    tree.Write(|d| {
        if let Some(partial) = partial_start.filter(|&node| d.RangeContainsNode(common, node)) {
            d.SetRangeStart(
                range,
                DOMBoundaryPoint {
                    node_id: d.Node(common).Id(),
                    offset: d.RangeNodeIndex(partial) + 1,
                },
            )?;
        } else if let Some(partial) = partial_end.filter(|&node| d.RangeContainsNode(common, node))
        {
            d.SetRangeStart(
                range,
                DOMBoundaryPoint {
                    node_id: d.Node(common).Id(),
                    offset: d.RangeNodeIndex(partial),
                },
            )?;
        }
        let mut data = range.data.borrow_mut();
        data.end = data.start;
        Ok::<_, DOMRangeError>(())
    })?;
    let nodes = tree.Read(|d| {
        let mut nodes = Vec::new();
        let mut node = process_start;
        while let Some(current) = node {
            if Some(current) == process_end {
                break;
            }
            nodes.push(current);
            node = d.RangeNextSibling(current);
        }
        nodes
    });
    for node in nodes {
        tree.RemoveNode(node);
    }
    Ok(())
}

pub(crate) fn InsertNode<T: TreeMutation>(
    tree: &mut T,
    range: &DOMRangeHandle,
    node_id: u64,
) -> Result<(), DOMRangeError> {
    tree.Read(|d| d.ValidateRangeInsertion(range, node_id))?;
    let (node, parent, mut reference) = tree.Write(|d| {
        let snapshot = d.Range(range).unwrap();
        let start = d.FindNodeById(snapshot.start.node_id).unwrap();
        let node = d.FindNodeById(node_id).unwrap();
        let is_text = d.Node(start).Type() == DOMNodeType::kText;
        let parent = if is_text {
            d.Node(start).Parent().unwrap()
        } else {
            start
        };
        let reference = if is_text {
            Some(d.SplitText(start, snapshot.start.offset)?)
        } else {
            d.Node(start).Children().get(snapshot.start.offset).copied()
        };
        Ok::<_, DOMRangeError>((node, parent, reference))
    })?;
    if reference == Some(node) {
        reference = tree.Read(|d| d.RangeNextSibling(node));
    }
    tree.RemoveNode(node);
    let (fragment, children, new_offset) = tree.Read(|d| {
        let fragment = d.Node(node).Type() == DOMNodeType::kDocumentFragment;
        let children = if fragment {
            d.Node(node).Children().to_vec()
        } else {
            vec![node]
        };
        let offset = reference
            .map(|node| d.RangeNodeIndex(node))
            .unwrap_or(d.Node(parent).Children().len())
            + children.len();
        (fragment, children, offset)
    });
    if fragment {
        let removal_scope = tree.Write(|d| {
            if !children.is_empty() {
                d.CaptureChildListMutation(node, &[], &children, None, None);
            }
            d.SuppressCapturedChildListMutations(node)
        });
        for &child in &children {
            tree.RemoveNode(child);
        }
        drop(removal_scope);
        let insertion_scope = tree.Write(|d| {
            let index = reference
                .map(|child| d.RangeNodeIndex(child))
                .unwrap_or(d.Node(parent).Children().len());
            let previous = index
                .checked_sub(1)
                .map(|index| d.Node(parent).Children()[index]);
            if !children.is_empty() {
                d.CaptureChildListMutation(parent, &children, &[], previous, reference);
            }
            d.SuppressCapturedChildListMutations(parent)
        });
        for child in children {
            tree.Write(|d| {
                if let Some(reference) = reference {
                    d.InsertBefore(parent, child, reference);
                } else {
                    d.AppendChild(parent, child);
                }
            });
        }
        drop(insertion_scope);
    } else {
        for child in children {
            tree.Write(|d| {
                if let Some(reference) = reference {
                    d.InsertBefore(parent, child, reference);
                } else {
                    d.AppendChild(parent, child);
                }
            });
        }
    }
    tree.Write(|d| {
        if d.Range(range).unwrap().Collapsed() {
            d.SetRangeEnd(
                range,
                DOMBoundaryPoint {
                    node_id: d.Node(parent).Id(),
                    offset: new_offset,
                },
            )?;
        }
        Ok(())
    })
}
