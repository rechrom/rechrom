#![allow(non_snake_case, non_upper_case_globals)]

//! PaintLayerPaintOrderIterator over Layout's PaintLayer and its
//! PaintLayerStackingNode, as PaintLayerPainter::PaintChildren reads them.
//!
//! Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
//! Source: third_party/blink/renderer/core/paint/paint_layer_paint_order_iterator.h:29-89
//! Source: third_party/blink/renderer/core/paint/paint_layer_paint_order_iterator.cc:39-80
//!
//! The z-order lists are the translated stacking node's, rebuilt by Layout's
//! PaintLayer::UpdateDescendantDependentFlags before PrePaint
//! (paint_layer.cc:864-882); normal-flow children are the PaintLayer's own
//! FirstChild()/NextSibling() links. Each listed PaintLayer is mapped to its
//! fragment occurrence(s) below the painting layer through
//! `LayerOccurrences`, which also carries the non-layer property ancestors
//! the flat replay stream re-enters.

use layout::fragment::ResidentBoxGeometry as _;
use layout::input::Position;
use foundation::TrustedIdentityHashMap;
use layout::paint::FragmentPaintData;
use layout::views::paint_layer::{PaintLayer, PaintLayers};
use paint_record::paint_object::{PaintObject, PaintWalkContext, ResolvedFragment};

// Source: paint_layer_paint_order_iterator.h:29-37 (PaintLayerIteration).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PaintLayerIteration(u32);

impl PaintLayerIteration {
    pub(crate) const kNegativeZOrderChildren: Self = Self(1);
    pub(crate) const kNormalFlowChildren: Self = Self(1 << 1);
    pub(crate) const kPositiveZOrderChildren: Self = Self(1 << 2);
    pub(crate) const kNormalFlowAndPositiveZOrderChildren: Self = Self((1 << 1) | (1 << 2));
    pub(crate) const kAllChildren: Self = Self(1 | (1 << 1) | (1 << 2));
}

// Source: paint_layer_paint_order_iterator.h:48-89. No Clone/Copy, matching
// the deleted source copy operations. The debug-only
// PaintLayerListMutationDetector (paint_layer.h:877-894) is folded in: it
// saves the root's mutation-allowed state for the iterator's lifetime.
//
// Rechrom's PaintLayer derives FirstChild() from the LayoutObject subtree
// instead of a stored link, so the normal-flow cursor is seeded when the
// normal-flow pass starts (None) rather than at construction; the visited
// sequence is the source's.
pub(crate) struct PaintLayerPaintOrderIterator<'a> {
    root_: &'a PaintLayer,
    remaining_children_: u32,
    index_: usize,
    current_normal_flow_child_: Option<*mut PaintLayer>,
    #[cfg(debug_assertions)]
    previous_mutation_allowed_state_: bool,
}

impl<'a> PaintLayerPaintOrderIterator<'a> {
    // Source: paint_layer_paint_order_iterator.h:55-69.
    pub(crate) fn new(root: &'a PaintLayer, which_children: PaintLayerIteration) -> Self {
        Self {
            root_: root,
            remaining_children_: which_children.0,
            index_: 0,
            current_normal_flow_child_: None,
            #[cfg(debug_assertions)]
            previous_mutation_allowed_state_: root.SetLayerListMutationAllowed(false),
        }
    }

    // Source: paint_layer_paint_order_iterator.h:73-79.
    pub(crate) fn LayersPaintingOverlayOverflowControlsAfter(
        &self,
        layer: &PaintLayer,
    ) -> Option<&'a PaintLayers> {
        self.root_
            .StackingNode()?
            .LayersPaintingOverlayOverflowControlsAfter(layer as *const PaintLayer)
    }

    // Source: paint_layer_paint_order_iterator.cc:39-80.
    pub(crate) fn Next(&mut self) -> Option<&'a PaintLayer> {
        if self.remaining_children_ & PaintLayerIteration::kNegativeZOrderChildren.0 != 0 {
            if let Some(node) = self.root_.StackingNode() {
                let list = node.NegZOrderList();
                if self.index_ < list.len() {
                    let layer = list[self.index_].Get();
                    self.index_ += 1;
                    // The stacking node's Members name live PaintLayers of
                    // this tree; the lifecycle keeps them through Paint.
                    return Some(unsafe { &*layer });
                }
            }
            self.index_ = 0;
            self.remaining_children_ &= !PaintLayerIteration::kNegativeZOrderChildren.0;
        }

        if self.remaining_children_ & PaintLayerIteration::kNormalFlowChildren.0 != 0 {
            let root = self.root_;
            let mut current = *self
                .current_normal_flow_child_
                .get_or_insert_with(|| root.FirstChild());
            // PaintLayer links name live layers of this LayoutObject tree.
            while let Some(child) = unsafe { current.as_ref() } {
                current = child.NextSibling();
                self.current_normal_flow_child_ = Some(current);
                if child.GetLayoutObject().IsStacked() {
                    continue;
                }
                return Some(child);
            }
            // We reset the iterator in case we reuse it.
            self.current_normal_flow_child_ = None;
            self.remaining_children_ &= !PaintLayerIteration::kNormalFlowChildren.0;
        }

        if self.remaining_children_ & PaintLayerIteration::kPositiveZOrderChildren.0 != 0 {
            if let Some(node) = self.root_.StackingNode() {
                let list = node.PosZOrderList();
                if self.index_ < list.len() {
                    let layer = list[self.index_].Get();
                    self.index_ += 1;
                    return Some(unsafe { &*layer });
                }
            }
            self.index_ = 0;
            self.remaining_children_ &= !PaintLayerIteration::kPositiveZOrderChildren.0;
        }

        None
    }
}

#[cfg(debug_assertions)]
impl Drop for PaintLayerPaintOrderIterator<'_> {
    fn drop(&mut self) {
        self.root_
            .SetLayerListMutationAllowed(self.previous_mutation_allowed_state_);
    }
}

const kNoOccurrence: u32 = u32::MAX;

// ---------------------------------------------------------------------------
// The paint walk's layer occurrences, read from the fragment cursor.
// ---------------------------------------------------------------------------

/// PaintLayerPainter(*child) paints each FragmentData of the child's
/// LayoutObject (paint_layer_painter.cc:752-786, FragmentDataIterator). One
/// such occurrence below the painting layer occurrence: the child's fragment
/// cursor, and the fragments between the painting layer and it whose
/// property scopes the flat replay stream re-enters (Chromium paints the
/// child in its LocalBorderBoxProperties and needs none of them).
#[derive(Clone, Copy)]
pub(crate) struct LayerOccurrence<'s, 'a> {
    pub(crate) layer: ResolvedFragment<'a>,
    /// Root first. For an out-of-flow child whose logical (DOM) parent is not
    /// one of its physical (containing-block) ancestors, the logical chain:
    /// the effects follow the DOM, the clips the containing block.
    pub(crate) property_ancestors: &'s [ResolvedFragment<'a>],
    /// The child's logical parent is not a physical ancestor.
    pub(crate) escapes_ancestor_geometry: bool,
    /// How many leading property ancestors the child re-enters the
    /// escapable geometry (overflow clips) of: for an escaping child, those
    /// up to its last physical ancestor; else all.
    pub(crate) uses_inherited_geometry: usize,
    /// Whether the painting layer occurrence is a physical ancestor of the
    /// child.
    pub(crate) painting_layer_is_physical_ancestor: bool,
}

impl LayerOccurrence<'_, '_> {
    /// An escaping child none of whose physical ancestors is the painting
    /// layer or below it: the painting layer suspends its own escapable
    /// geometry around the child.
    pub(crate) fn EscapesCurrentGeometry(&self) -> bool {
        self.escapes_ancestor_geometry
            && !self.painting_layer_is_physical_ancestor
            && self.uses_inherited_geometry == 0
    }
}

struct CursorOccurrence<'a> {
    cursor: ResolvedFragment<'a>,
    ancestors: (u32, u32),
    escapes: bool,
    uses: u32,
    painting_layer_is_physical_ancestor: bool,
    next: u32,
}

/// One recorded fragment occurrence of an out-of-flow child's logical
/// parent: its node id, cursor and physical path below the painting layer.
struct LogicalParentOccurrence<'a> {
    node_id: u64,
    cursor: ResolvedFragment<'a>,
    path: (u32, u32),
}

/// Whether `object` stops the layer collection as a stacking context
/// (PaintLayerStackingNode::CollectLayers does not enter one,
/// paint_layer_stacking_node.cc:328-401): the classification
/// ClassifyFragmentPaintLayer gives a painting layer, a frame's LayoutView,
/// and a masked object (CSS masks isolate a stacking context).
pub(crate) fn IsStackingContextObject(object: PaintObject<'_, '_>) -> bool {
    let Some(fragment) = object.Fragment() else {
        return false;
    };
    object.is_paint_layer()
        && (fragment.paint.IsLayoutView()
            || fragment.paint.PaintLayerIsStackingContext()
            || object.applies_mask())
}

/// An out-of-flow positioned object's logical parent node id
/// (LayoutObject::Parent() up to the first object with a Node).
fn OutOfFlowLogicalParentId(fragment: &FragmentPaintData) -> Option<u64> {
    if !fragment.paint.HasSource() {
        return None;
    }
    let position = fragment.paint.Position();
    if position != Position::kAbsolute && position != Position::kFixed {
        return None;
    }
    fragment.paint.LogicalParentNodeId()
}

/// The out-of-flow PaintLayers in `layer`'s PaintLayer subtree, not entering
/// stacking contexts: the layers whose logical paint parent may be below
/// `layer` while their physical fragment is not.
fn CollectOutOfFlowLayers<'a>(layer: &'a PaintLayer, out: &mut Vec<&'a PaintLayer>) {
    let mut child = layer.FirstChild();
    // PaintLayer links name live layers of this LayoutObject tree.
    while let Some(current) = unsafe { child.as_ref() } {
        let object = current.GetLayoutObject();
        if object.IsOutOfFlowPositioned() {
            out.push(current);
        }
        if !object.IsStackingContext() {
            CollectOutOfFlowLayers(current, out);
        }
        child = current.NextSibling();
    }
}

/// PaintLayer -> fragment occurrences below one painting layer occurrence.
///
/// Built once per painted layer by one walk of the fragment subtree its
/// iterator can name (direct child layers; for a layer owning a
/// PaintLayerStackingNode, every layer up to nested stacking contexts, the
/// domain of PaintLayerStackingNode::CollectLayers). The walk follows the
/// physical fragments; an out-of-flow child whose logical (DOM) parent is not
/// one of its physical ancestors takes the logical parent's path instead, as
/// PrePaint's paint order placed it (RestoreLogicalOutOfFlowParents). A
/// fragmented PaintLayer keeps all its occurrences in fragment order.
pub(crate) struct LayerOccurrences<'a> {
    /// Every fragment is resolved once, when the collection walk reaches it;
    /// the painters of the listed layers and of their property ancestors
    /// reuse that resolution.
    ancestors: Vec<ResolvedFragment<'a>>,
    occurrences: Vec<CursorOccurrence<'a>>,
    layers: TrustedIdentityHashMap<*const PaintLayer, (u32, u32)>,
    has_escaping_layer: bool,
}

impl<'a> LayerOccurrences<'a> {
    /// `through_layers`: also descend through non-stacking-context layers
    /// (only a layer owning a stacking node lists such descendants).
    /// `resolve_escaping`: also resolve the escaping out-of-flow layers of
    /// the logical subtree (for HasEscapingLayer) when not `through_layers`.
    pub(crate) fn Collect(
        root: ResolvedFragment<'a>,
        root_layer: &'a PaintLayer,
        through_layers: bool,
        resolve_escaping: bool,
    ) -> Self {
        let mut trace = browser_tracing::span("paint_walk", "PaintWalk.CollectLayerOccurrences");
        let mut out_of_flow = Vec::new();
        if through_layers || resolve_escaping {
            CollectOutOfFlowLayers(root_layer, &mut out_of_flow);
        }
        let wanted: Vec<u64> = out_of_flow
            .iter()
            .filter_map(|&layer| {
                root.cursor
                    .ForLayoutObject(layer.GetLayoutObject())
                    .and_then(|cursor| OutOfFlowLogicalParentId(cursor.PaintData()))
            })
            .collect();
        let mut occurrences = Self {
            ancestors: Vec::new(),
            occurrences: Vec::new(),
            layers: TrustedIdentityHashMap::default(),
            has_escaping_layer: false,
        };
        let mut logical_parents = Vec::new();
        occurrences.Visit(
            root,
            &mut Vec::new(),
            through_layers || !wanted.is_empty(),
            &wanted,
            &mut logical_parents,
        );
        if !wanted.is_empty() {
            occurrences.ResolveEscaping(root, &out_of_flow, &logical_parents);
        }
        trace.set("occurrences", occurrences.occurrences.len() as f64);
        occurrences
    }

    fn Visit(
        &mut self,
        parent: ResolvedFragment<'a>,
        path: &mut Vec<ResolvedFragment<'a>>,
        through_layers: bool,
        wanted: &[u64],
        logical_parents: &mut Vec<LogicalParentOccurrence<'a>>,
    ) {
        for child in parent.cursor.Children() {
            let child = ResolvedFragment::new(child);
            let fragment = child.fragment;
            if !wanted.is_empty()
                && fragment.paint.HasSource()
                && fragment.ViewEstablishesPaintState()
                && wanted.contains(&fragment.ViewNodeId())
            {
                let start = self.ancestors.len() as u32;
                self.ancestors.extend_from_slice(path);
                logical_parents.push(LogicalParentOccurrence {
                    node_id: fragment.ViewNodeId(),
                    cursor: child,
                    path: (start, self.ancestors.len() as u32),
                });
            }
            let object = child.Object(PaintWalkContext::default());
            if object.is_paint_layer() {
                if let Some(layer) = child.cursor.OwnPaintLayer() {
                    self.Push(layer, child, path, path.len() as u32, true, false);
                }
                if !through_layers || IsStackingContextObject(object) {
                    continue;
                }
            }
            path.push(child);
            self.Visit(child, path, through_layers, wanted, logical_parents);
            path.pop();
        }
    }

    fn Push(
        &mut self,
        layer: &PaintLayer,
        cursor: ResolvedFragment<'a>,
        path: &[ResolvedFragment<'a>],
        uses: u32,
        painting_layer_is_physical_ancestor: bool,
        escapes: bool,
    ) {
        let start = self.ancestors.len() as u32;
        self.ancestors.extend_from_slice(path);
        let index = self.occurrences.len() as u32;
        self.occurrences.push(CursorOccurrence {
            cursor,
            ancestors: (start, self.ancestors.len() as u32),
            escapes,
            uses,
            painting_layer_is_physical_ancestor,
            next: kNoOccurrence,
        });
        self.has_escaping_layer |= escapes;
        match self.layers.entry(layer as *const PaintLayer) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                let tail = entry.get().1;
                self.occurrences[tail as usize].next = index;
                entry.get_mut().1 = index;
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert((index, index));
            }
        }
    }

    /// The one recorded occurrence of the logical parent `node_id` other
    /// than `child` itself (ResolveLogicalParentOccurrence's unique
    /// occurrence establishing paint state).
    fn UniqueLogicalParent<'l>(
        logical_parents: &'l [LogicalParentOccurrence<'a>],
        node_id: u64,
        child: ResolvedFragment<'_>,
    ) -> Option<&'l LogicalParentOccurrence<'a>> {
        let mut matches = logical_parents
            .iter()
            .filter(|candidate| candidate.node_id == node_id && !candidate.cursor.IsSameFragment(child));
        let first = matches.next()?;
        matches.next().is_none().then_some(first)
    }

    /// The path of an escaping child under its logical parent: the logical
    /// parent's own path, then the logical parent.
    fn LogicalPath(
        &self,
        logical_parent: Option<&LogicalParentOccurrence<'a>>,
    ) -> Vec<ResolvedFragment<'a>> {
        match logical_parent {
            Some(parent) => {
                let (start, end) = parent.path;
                let mut path = self.ancestors[start as usize..end as usize].to_vec();
                path.push(parent.cursor);
                path
            }
            // The logical parent is the painting layer itself.
            None => Vec::new(),
        }
    }

    /// Source: pre_paint_tree_walk.rs DetachLogicallyReparentedOutOfFlowNodes
    /// / ResolveLogicalParentOccurrence, the order PrePaint gives an
    /// out-of-flow occurrence whose logical parent is not a physical
    /// ancestor; and the out-of-flow layers of the logical subtree whose
    /// fragments lie outside the painting layer's fragment subtree.
    fn ResolveEscaping(
        &mut self,
        root: ResolvedFragment<'a>,
        out_of_flow: &[&'a PaintLayer],
        logical_parents: &[LogicalParentOccurrence<'a>],
    ) {
        let root_fragment = root.fragment;
        let root_is_logical_parent = |node_id: u64| {
            root_fragment.ViewNodeId() == node_id
                && root_fragment.paint.HasSource()
                && root_fragment.ViewEstablishesPaintState()
        };
        // Occurrences found below the painting layer.
        for index in 0..self.occurrences.len() {
            let cursor = self.occurrences[index].cursor;
            let Some(node_id) = OutOfFlowLogicalParentId(cursor.fragment) else {
                continue;
            };
            let (start, end) = self.occurrences[index].ancestors;
            let physical = self.ancestors[start as usize..end as usize].to_vec();
            let is_logical_parent = |candidate: &ResolvedFragment<'_>| {
                candidate.fragment.ViewNodeId() == node_id
                    && candidate.fragment.ViewEstablishesPaintState()
            };
            if is_logical_parent(&root) || physical.iter().any(is_logical_parent) {
                continue;
            }
            let Some(logical_parent) = Self::UniqueLogicalParent(logical_parents, node_id, cursor)
            else {
                continue;
            };
            let path = self.LogicalPath(Some(logical_parent));
            let uses = path
                .iter()
                .rposition(|entry| physical.iter().any(|candidate| candidate.IsSameFragment(*entry)))
                .map_or(0, |last| last + 1);
            let start = self.ancestors.len() as u32;
            self.ancestors.extend_from_slice(&path);
            let occurrence = &mut self.occurrences[index];
            occurrence.ancestors = (start, self.ancestors.len() as u32);
            occurrence.escapes = true;
            occurrence.uses = uses as u32;
            self.has_escaping_layer = true;
        }
        // Layers of the logical subtree without a fragment below the
        // painting layer: their containing block is above it.
        for &layer in out_of_flow {
            if self.layers.contains_key(&(layer as *const PaintLayer)) {
                continue;
            }
            for cursor in root.cursor.FragmentsForLayoutObject(layer.GetLayoutObject()) {
                let cursor = ResolvedFragment::new(cursor);
                if !cursor.Object(PaintWalkContext::default()).is_paint_layer() {
                    continue;
                }
                let Some(node_id) = OutOfFlowLogicalParentId(cursor.fragment) else {
                    continue;
                };
                let logical_parent = if root_is_logical_parent(node_id) {
                    None
                } else {
                    match Self::UniqueLogicalParent(logical_parents, node_id, cursor) {
                        Some(parent) => Some(parent),
                        None => continue,
                    }
                };
                let path = self.LogicalPath(logical_parent);
                self.Push(layer, cursor, &path, 0, false, true);
            }
        }
    }

    /// Whether an out-of-flow layer of the logical subtree escapes its
    /// physical ancestors (PaintLayerPainter's HasEscapingChildLayer).
    pub(crate) fn HasEscapingLayer(&self) -> bool {
        self.has_escaping_layer
    }

    /// The occurrences of `layer`, in fragment order.
    pub(crate) fn Occurrences<'s>(
        &'s self,
        layer: &PaintLayer,
    ) -> impl Iterator<Item = LayerOccurrence<'s, 'a>> + 's {
        let mut index = self
            .layers
            .get(&(layer as *const PaintLayer))
            .map_or(kNoOccurrence, |&(head, _)| head);
        std::iter::from_fn(move || {
            if index == kNoOccurrence {
                return None;
            }
            let occurrence = &self.occurrences[index as usize];
            index = occurrence.next;
            let (start, end) = occurrence.ancestors;
            Some(LayerOccurrence {
                layer: occurrence.cursor,
                property_ancestors: &self.ancestors[start as usize..end as usize],
                escapes_ancestor_geometry: occurrence.escapes,
                uses_inherited_geometry: if occurrence.escapes {
                    occurrence.uses as usize
                } else {
                    (end - start) as usize
                },
                painting_layer_is_physical_ancestor: occurrence.painting_layer_is_physical_ancestor,
            })
        })
    }
}
