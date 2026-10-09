//! Wheel-listener regions exported with committed input geometry.
//!
//! This is input metadata, not Page state. The owner supplies the current DOM,
//! immutable fragments, listener targets and frame identity; the resolver owns
//! descendant expansion, geometry projection and cache validation.

use dom::{persistent_document::DOMNodeType, Document};
use layoutng_assembly::fragment_tree::FragmentNode;
use paint::paint_engine::{FragmentClientRectsByNode, PaintRect};
use std::collections::HashSet;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlockingWheelEventRegions {
    pub covers_viewport: bool,
    pub rects: Vec<PaintRect>,
}

#[derive(Default)]
pub struct WheelEventRegionResolver {
    cached: Option<(u64, Vec<u64>, BlockingWheelEventRegions)>,
}

impl WheelEventRegionResolver {
    pub fn Resolve(
        &mut self,
        frame_sequence: u64,
        mut listener_targets: Vec<u64>,
        document: &Document,
        fragments: Option<&FragmentNode>,
    ) -> BlockingWheelEventRegions {
        listener_targets.sort_unstable();
        if let Some((cached_sequence, cached_targets, cached_regions)) = &self.cached {
            if *cached_sequence == frame_sequence && *cached_targets == listener_targets {
                return cached_regions.clone();
            }
        }
        let regions = ResolveBlockingWheelEventRegions(&listener_targets, document, fragments);
        self.cached = Some((frame_sequence, listener_targets, regions.clone()));
        regions
    }

    pub fn Invalidate(&mut self) {
        self.cached = None;
    }
}

fn ResolveBlockingWheelEventRegions(
    listener_targets: &[u64],
    document: &Document,
    fragments: Option<&FragmentNode>,
) -> BlockingWheelEventRegions {
    if listener_targets.is_empty() {
        return BlockingWheelEventRegions::default();
    }
    let Some(fragments) = fragments else {
        return BlockingWheelEventRegions {
            covers_viewport: true,
            rects: Vec::new(),
        };
    };
    let mut node_ids = HashSet::new();
    let mut covers_viewport = false;
    fn Collect(document: &Document, index: usize, ids: &mut HashSet<u64>) {
        let node = document.Node(index);
        ids.insert(node.Id());
        for child in node.Children() {
            Collect(document, *child, ids);
        }
    }
    for &target in listener_targets {
        if target == 0 {
            covers_viewport = true;
            break;
        }
        let Some(index) = document.FindNodeById(target) else {
            continue;
        };
        if document.Node(index).Type() == DOMNodeType::kDocument {
            covers_viewport = true;
            break;
        }
        Collect(document, index, &mut node_ids);
    }
    if covers_viewport {
        return BlockingWheelEventRegions {
            covers_viewport: true,
            rects: Vec::new(),
        };
    }
    let by_node = FragmentClientRectsByNode(fragments);
    let rects = node_ids
        .into_iter()
        .filter_map(|id| by_node.get(&id))
        .flatten()
        .copied()
        .filter(|rect| !rect.is_empty())
        .collect();
    BlockingWheelEventRegions {
        covers_viewport: false,
        rects,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_listener_covers_viewport_without_fragment_scan() {
        let owner = dom::DOM::new();
        let document = owner.GetDocument();
        let document_id = document.Node(document.Root()).Id();
        let regions = ResolveBlockingWheelEventRegions(
            &[document_id],
            &document,
            Some(&FragmentNode::default()),
        );
        assert!(regions.covers_viewport);
        assert!(regions.rects.is_empty());
    }
}
