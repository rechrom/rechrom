//! Wheel-listener regions exported with committed input geometry.
//!
//! This is input metadata, not Page state. The owner supplies the current DOM,
//! immutable fragments, listener targets and frame identity; the resolver owns
//! descendant expansion, geometry projection and cache validation.

use crate::interaction_engine::InteractionFragmentSource;
use dom::{persistent_document::DOMNodeType, Document};
use layoutng_assembly::layout_engine::NativePaintFragmentCursor;
use paint::paint_engine::{NativeFragmentClientRectsByNode, PaintRect};
use std::collections::HashSet;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlockingWheelEventRegions {
    pub covers_viewport: bool,
    /// Client-space regions whose fragments move with the root scroll node.
    pub scrolling_rects: Vec<PaintRect>,
    /// Client-space regions in a fixed-to-view subtree. These must not be
    /// sampled with the compositor's pending root-scroll displacement.
    pub fixed_rects: Vec<PaintRect>,
}

#[derive(Default)]
pub struct WheelEventRegionResolver {
    cached: Option<(u64, Vec<u64>, BlockingWheelEventRegions)>,
}

impl WheelEventRegionResolver {
    pub fn ResolveNative(
        &mut self,
        frame_sequence: u64,
        mut listener_targets: Vec<u64>,
        document: &Document,
        fragments: Option<NativePaintFragmentCursor<'_>>,
    ) -> BlockingWheelEventRegions {
        listener_targets.sort_unstable();
        if let Some((cached_sequence, cached_targets, cached_regions)) = &self.cached {
            if *cached_sequence == frame_sequence && *cached_targets == listener_targets {
                return cached_regions.clone();
            }
        }
        let regions = ResolveBlockingWheelEventRegionsSource(
            &listener_targets,
            document,
            fragments.map(InteractionFragmentSource::from),
        );
        self.cached = Some((frame_sequence, listener_targets, regions.clone()));
        regions
    }

    pub fn Invalidate(&mut self) {
        self.cached = None;
    }
}

fn ResolveBlockingWheelEventRegionsSource(
    listener_targets: &[u64],
    document: &Document,
    fragments: Option<InteractionFragmentSource<'_>>,
) -> BlockingWheelEventRegions {
    if listener_targets.is_empty() {
        return BlockingWheelEventRegions::default();
    }
    let Some(fragments) = fragments else {
        return BlockingWheelEventRegions {
            covers_viewport: true,
            scrolling_rects: Vec::new(),
            fixed_rects: Vec::new(),
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
            scrolling_rects: Vec::new(),
            fixed_rects: Vec::new(),
        };
    }
    let by_node = NativeFragmentClientRectsByNode(fragments.Native());
    // Fixed descendants are painted in the viewport transform space. Keep
    // that attachment across this input metadata boundary: compositor scroll
    // can then move ordinary event regions without moving fixed controls.
    fn CollectHitNodes(
        fragment: InteractionFragmentSource<'_>,
        inherited: bool,
        hit_nodes: &mut HashSet<u64>,
        fixed_nodes: &mut HashSet<u64>,
    ) {
        // Resource definitions retain measurable descendant geometry, but
        // LayoutSVGHiddenContainer::NodeAtPoint rejects the entire subtree.
        if fragment.paint.svg_hidden_container {
            return;
        }
        hit_nodes.insert(fragment.node_id);
        let fixed = inherited || fragment.paint.fixed_to_view;
        if fixed {
            fixed_nodes.insert(fragment.node_id);
        }
        for child in fragment.Children() {
            CollectHitNodes(child, fixed, hit_nodes, fixed_nodes);
        }
    }
    let mut hit_nodes = HashSet::new();
    let mut fixed_nodes = HashSet::new();
    CollectHitNodes(fragments, false, &mut hit_nodes, &mut fixed_nodes);
    let mut scrolling_rects = Vec::new();
    let mut fixed_rects = Vec::new();
    for id in node_ids {
        if !hit_nodes.contains(&id) {
            continue;
        }
        let Some(node_rects) = by_node.get(&id) else {
            continue;
        };
        let output = if fixed_nodes.contains(&id) {
            &mut fixed_rects
        } else {
            &mut scrolling_rects
        };
        output.extend(node_rects.iter().copied().filter(|rect| !rect.is_empty()));
    }
    BlockingWheelEventRegions {
        covers_viewport: false,
        scrolling_rects,
        fixed_rects,
    }
}
