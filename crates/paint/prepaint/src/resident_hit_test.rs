//! Borrowed hit-test access to the committed FragmentData/ObjectPaintProperties.
//!
//! This cursor does not expose the retained PrePaint tree, its child storage,
//! or a mutable owner. Lookup uses the existing DisplayItemClient directory
//! and verifies source identity; hit testing must prepare lifecycle first.

use layoutng_assembly::fragment_tree::ResidentBoxGeometry as _;
use foundation::PhysicalOffset;
use layoutng_assembly::{
    fragment_tree::FragmentPaintData,
    layout_engine::{FragmentCursor, PaintFragmentCursor},
};
use paint_model::{
    paint_property_tree::{
        ClipPaintPropertyNode, EffectPaintPropertyNode, PaintPropertyKey, PaintPropertyNodeStore,
        PaintPropertyOwner, PaintPropertyRole, PropertyTreeState, TransformPaintPropertyNode,
    },
    PaintRect,
};

use crate::pre_paint_tree_walk::{PaintTreeNode, PrePaintTreeWalk};
use std::{cell::Ref, collections::HashMap};

/// Source PaintLayerStackingNode vectors. These are references to resident
/// derived Paint owners, not copies of fragments or a hit-test-only tree.
pub(crate) struct ResidentPaintLayerOrder {
    layers: Vec<*const ()>,
    controls_after: HashMap<u64, Vec<*const ()>>,
}

impl PaintTreeNode<'_> {
    pub fn HasResidentLayerOrder(&self) -> bool {
        self.hit_test_layer_order.borrow().is_some()
    }

    /// Publish the source stacking collector's pointers into their canonical
    /// resident owner. This is an internal Paint assembly operation.
    ///
    /// # Safety
    /// Every pointer must be a PaintTreeNode in this retained tree. Its Box
    /// address must remain stationary until PrePaint clears this ordering
    /// before a topology mutation is observable to a query.
    pub unsafe fn PublishResidentLayerOrder(
        &self,
        layers: Vec<*const ()>,
        controls_after: HashMap<u64, Vec<*const ()>>,
    ) {
        *self.hit_test_layer_order.borrow_mut() = Some(ResidentPaintLayerOrder {
            layers,
            controls_after,
        });
    }
}

#[derive(Clone, Copy)]
pub struct ResidentLayerSelection {
    pub negative_z: bool,
    pub normal_flow: bool,
    pub positive_z: bool,
}

/// Borrowed iterator over the canonical source lists. Iterator values retain
/// only the existing Paint owner's lifetime and native property slots.
pub struct ResidentPaintLayerOrderCursor<'a> {
    order: Ref<'a, Option<ResidentPaintLayerOrder>>,
    slots: &'a PaintPropertyNodeStore,
    controls_after: Option<u64>,
    selection: ResidentLayerSelection,
    front: usize,
    back: usize,
    reverse: bool,
}
impl<'a> ResidentPaintLayerOrderCursor<'a> {
    fn At(&self, index: usize) -> &'a PaintTreeNode<'static> {
        let order = self
            .order
            .as_ref()
            .expect("source layer order was prepared");
        let nodes = self
            .controls_after
            .map_or(order.layers.as_slice(), |child| {
                order
                    .controls_after
                    .get(&child)
                    .map(Vec::as_slice)
                    .unwrap_or(&[])
            });
        // The borrowed PaintEngine retains these exact boxed PrePaint owners;
        // topology mutation cannot occur until this cursor has been released.
        unsafe { &*nodes[index].cast::<PaintTreeNode<'static>>() }
    }
    fn Includes(&self, layer: &PaintTreeNode<'_>) -> bool {
        if !layer.is_stacked {
            self.selection.normal_flow
        } else if layer.stacking_level < 0 {
            self.selection.negative_z
        } else {
            self.selection.positive_z
        }
    }
    fn Take(&mut self, reverse: bool) -> Option<ResidentPaintHitTest<'a>> {
        while self.front < self.back {
            let index = if reverse {
                self.back -= 1;
                self.back
            } else {
                let index = self.front;
                self.front += 1;
                index
            };
            let node = self.At(index);
            if self.Includes(node) {
                return Some(ResidentPaintHitTest {
                    node,
                    slots: self.slots,
                });
            }
        }
        None
    }
}
impl<'a> Iterator for ResidentPaintLayerOrderCursor<'a> {
    type Item = ResidentPaintHitTest<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.Take(self.reverse)
    }
}
impl DoubleEndedIterator for ResidentPaintLayerOrderCursor<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.Take(!self.reverse)
    }
}

#[derive(Clone, Copy)]
pub struct ResidentPaintHitTest<'a> {
    node: &'a PaintTreeNode<'static>,
    slots: &'a PaintPropertyNodeStore,
}

/// Whether `node` was walked for exactly the live Layout owner `query` names.
///
/// Chromium hit tests read committed FragmentData/ObjectPaintProperties only
/// after LocalFrameView::UpdateLifecycleToPrePaintClean
/// (core/frame/local_frame_view.cc; DocumentLifecycle::kPrePaintClean,
/// core/dom/document_lifecycle.h), and then trust them without any
/// per-object freshness test: PrePaintTreeWalk consumed and cleared every
/// LayoutObject dirty bit (pre_paint_tree_walk.cc:107). Freshness is that
/// lifecycle gate, which Rechrom's callers hold (LayoutViewHitTestQuery
/// checks IsPrePaintClean and the prepared revision; Page pointer geometry
/// runs PrePaint below PrePaintClean). This lookup only proves owner
/// identity: the same resident sidecar slot (checked by the caller) and
/// the same PhysicalFragment / FragmentItem and LayoutObject the walk
/// visited. A relayout that replaces the fragment therefore rejects the
/// resident node; any other relayout is caught by the lifecycle gate.
///
/// The LayoutObject dirty bits are deliberately not consulted here: Rechrom
/// clears them only at the full paint client commit
/// (LayoutObject::ClearPaintFlags from layout_commit's client_commit.rs), so
/// after a PrePaint-only lifecycle (PaintEngine::PrePaintWithLifecycle, the
/// hit-test preparation path) the walked objects still carry the bits the
/// walk consumed, and LayoutView itself carries container-chain
/// ShouldCheckForPaintInvalidation. Rejecting on them would make every hit
/// test between a DOM change and the next paint fail.
fn WalkedSameSourceOwner(node: &PaintTreeNode<'_>, query: PaintFragmentCursor<'_>) -> bool {
    // Hand-built fixture nodes have no source cursor; the exact sidecar slot
    // identity checked by the caller is their only owner.
    let Some(walked) = node.SourceCursor() else {
        return true;
    };
    // Compare owner addresses only; the walked cursor is never dereferenced,
    // since it may name a fragment a later Layout publication replaced. The
    // PhysicalFragment / FragmentItem owner determines its LayoutObject.
    // Container offsets are not compared: call-scoped cursors rebound with
    // ForPhysicalFragment carry no link offset, unlike the walk's traversal.
    match (walked.Cursor(), query.Cursor()) {
        (
            FragmentCursor::Physical { fragment: walked, .. },
            FragmentCursor::Physical { fragment: query, .. },
        ) => std::ptr::eq(walked, query),
        (
            FragmentCursor::Inline { item: walked, .. },
            FragmentCursor::Inline { item: query, .. },
        ) => std::ptr::eq(walked, query),
        _ => false,
    }
}

impl PrePaintTreeWalk {
    /// Internal Paint assembly port. The facade uses this only to run the
    /// canonical PaintWalk stacking collector; it never exports this owner.
    pub fn WithResidentPaintNode<'a, T>(
        &'a self,
        source: PaintFragmentCursor<'_>,
        query: impl FnOnce(&'a PaintTreeNode<'static>) -> T,
    ) -> Option<T> {
        self.ResidentHitTest(source)
            .map(|source| query(source.node))
    }

    pub fn ResidentHitTest(
        &self,
        source: PaintFragmentCursor<'_>,
    ) -> Option<ResidentPaintHitTest<'_>> {
        let query = source;
        let source = source.PaintData();
        let candidates = self
            .client_nodes
            .get(&source.paint.DisplayItemClientId())?;
        for pointer in candidates {
            // RetainTree populates this directory after restoring resident
            // ownership. Its boxes cannot move or disappear during this borrow.
            let node = unsafe { &**pointer };
            if !node
                .fragment
                .as_deref()
                .is_some_and(|fragment| std::ptr::eq(fragment, source))
            {
                continue;
            }
            if !WalkedSameSourceOwner(node, query) {
                return None;
            }
            return Some(ResidentPaintHitTest {
                node,
                slots: &self.property_node_store,
            });
        }
        None
    }

    /// Resolve a source PaintLayer for an inline replaced fragment. LayoutNG
    /// keeps the FragmentItem and its linked PhysicalFragment as distinct
    /// paint sidecars even though both name the same DisplayItemClient and
    /// LayoutObject. Use the existing client directory and verify the object
    /// identity; this neither walks nor duplicates the resident paint tree.
    pub fn ResidentLayerHitTest(
        &self,
        source: PaintFragmentCursor<'_>,
    ) -> Option<ResidentPaintHitTest<'_>> {
        if let Some(exact) = self.ResidentHitTest(source) {
            if exact.node.is_paint_layer {
                return Some(exact);
            }
        }
        let source = source.PaintData();
        let identity = source.paint.LayoutObjectIdentity()?;
        let candidates = self
            .client_nodes
            .get(&source.paint.DisplayItemClientId())?;
        for pointer in candidates {
            let node = unsafe { &**pointer };
            if !node.is_paint_layer {
                continue;
            }
            let Some(fragment) = node.fragment.as_deref() else {
                continue;
            };
            if !fragment
                .paint
                .LayoutObjectIdentity()
                .is_some_and(|candidate| candidate.PtrEq(identity))
            {
                continue;
            }
            // The layer node is found through its LayoutObject, not through
            // the queried fragment, so there is no queried owner to compare;
            // it must have been visited by the PrePaint walk that produced
            // this resident tree (presence of its walk cache, not a revision
            // comparison). Freshness is the caller's PrePaintClean gate.
            if node.RecordedRevisions().is_none() {
                return None;
            }
            return Some(ResidentPaintHitTest {
                node,
                slots: &self.property_node_store,
            });
        }
        None
    }
}

impl<'a> ResidentPaintHitTest<'a> {
    pub fn NeedsReorderOverlayControls(self) -> bool {
        self.node.needs_reorder_overlay_overflow_controls.get()
    }

    pub fn LayoutObjectIdentity(
        self,
    ) -> Option<layoutng_assembly::layout_engine::PaintLayoutObjectIdentity<'a>> {
        self.Fragment().paint.LayoutObjectIdentity()
    }

    pub fn LayerOrder(
        self,
        selection: ResidentLayerSelection,
        controls_after: Option<u64>,
        reverse: bool,
    ) -> ResidentPaintLayerOrderCursor<'a> {
        let order = self.node.hit_test_layer_order.borrow();
        let source = order
            .as_ref()
            .expect("canonical layer order was prepared by PaintWalk");
        let length = controls_after.map_or(source.layers.len(), |child| {
            source.controls_after.get(&child).map_or(0, Vec::len)
        });
        ResidentPaintLayerOrderCursor {
            order,
            slots: self.slots,
            controls_after,
            selection,
            front: 0,
            back: length,
            reverse,
        }
    }

    /// Source: LayoutObject::HasNonZeroEffectiveOpacity(), layout_object.cc
    /// 2363-2385. Read the committed effect ancestry, including inherited
    /// effects, rather than reconstructing effective opacity from CSS.
    pub fn HasNonZeroEffectiveOpacity(self) -> bool {
        use paint_model::paint_property_tree::PropertyNodeResolver;
        let mut effect = Some(self.node.properties.nodes.effect.as_ref());
        while let Some(current) = effect {
            if current.opacity == 0.0 {
                return false;
            }
            effect = current.parent.as_ref().map(|parent| &**self.slots.effect(parent));
        }
        true
    }

    /// The published property trees this node's states resolve through.
    pub fn PublishedTrees(self) -> std::sync::Arc<paint_model::paint_property_tree::FrozenPropertyTrees> {
        self.slots.published_trees()
    }

    pub fn HasPaintProperties(self) -> bool {
        self.node.HasOwnPaintProperties()
    }

    pub fn Identity(self) -> usize {
        self.node as *const _ as usize
    }

    pub fn Fragment(self) -> &'a FragmentPaintData {
        self.node
            .fragment
            .as_deref()
            .expect("resident paint node has a source")
    }

    pub fn PaintOffset(self) -> PhysicalOffset {
        PhysicalOffset::new(
            foundation::LayoutUnit::FromFloatRound(self.node.paint_offset.x as f32),
            foundation::LayoutUnit::FromFloatRound(self.node.paint_offset.y as f32),
        )
    }

    pub fn LocalBorderBoxProperties(self) -> &'a PropertyTreeState {
        &self.node.properties.nodes
    }
    pub fn ContentsProperties(self) -> &'a PropertyTreeState {
        &self.node.contents_properties.nodes
    }
    // Source: fragment_data.cc:89-140. Property parents keep their exact owner.
    pub fn PreClip(self) -> &'a std::sync::Arc<ClipPaintPropertyNode> {
        for role in [
            PaintPropertyRole::ClipPathClip,
            PaintPropertyRole::MaskClip,
            PaintPropertyRole::CssClip,
            PaintPropertyRole::PixelMovingFilterClipExpander,
        ] {
            if let Some(clip) = self.Clip(role) {
                return paint_model::paint_property_tree::PropertyNodeResolver::clip(
                    self.slots,
                    clip.parent.as_ref().expect("non-root own clip has parent"),
                );
            }
        }
        &self.node.properties.nodes.clip
    }
    pub fn PreEffect(self) -> &'a std::sync::Arc<EffectPaintPropertyNode> {
        for role in [PaintPropertyRole::Effect, PaintPropertyRole::Filter] {
            if let Some(effect) = self.slots.EffectForKey(self.Key(role)) {
                return paint_model::paint_property_tree::PropertyNodeResolver::effect(
                    self.slots,
                    effect.parent.as_ref().expect("non-root own effect has parent"),
                );
            }
        }
        &self.node.properties.nodes.effect
    }

    pub fn LocalBorderTransform(self) -> &'a std::sync::Arc<TransformPaintPropertyNode> {
        &self.node.properties.nodes.transform
    }

    pub fn ContentsTransform(self) -> &'a std::sync::Arc<TransformPaintPropertyNode> {
        &self.node.contents_properties.nodes.transform
    }

    pub fn LocalBorderClip(self) -> &'a ClipPaintPropertyNode {
        &self.node.properties.nodes.clip
    }

    pub fn ContentsClip(self) -> &'a ClipPaintPropertyNode {
        &self.node.contents_properties.nodes.clip
    }

    pub fn PreTransform(self) -> &'a std::sync::Arc<TransformPaintPropertyNode> {
        &self.node.pre_transform
    }

    fn Key(self, role: PaintPropertyRole) -> PaintPropertyKey {
        let fragment = self.Fragment();
        PaintPropertyKey {
            owner: PaintPropertyOwner::Fragment {
                client: fragment.paint.DisplayItemClientId(),
                fragment: fragment.ViewDisplayItemFragment(),
            },
            role,
        }
    }

    pub fn Transform(
        self,
        role: PaintPropertyRole,
    ) -> Option<&'a std::sync::Arc<TransformPaintPropertyNode>> {
        self.slots.TransformOwnerForKey(self.Key(role))
    }

    pub fn Clip(self, role: PaintPropertyRole) -> Option<&'a ClipPaintPropertyNode> {
        self.slots.ClipForKey(self.Key(role))
    }

    pub fn BackgroundClip(self) -> Option<PaintRect> {
        self.node.clip
    }

    pub fn ForegroundClip(self) -> Option<PaintRect> {
        self.node.contents_properties.nodes.clip.PaintClipRect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layer(id: u64, stacked: bool, z: i32) -> Box<PaintTreeNode<'static>> {
        let mut fragment = FragmentPaintData::default();
        fragment.node_id = id;
        let mut node = Box::new(PaintTreeNode::default());
        node.SetOwnedFragmentForTesting(&fragment);
        node.is_stacked = stacked;
        node.stacking_level = z;
        node
    }

    #[test]
    fn source_layer_masks_and_overlay_reverse_order_share_the_resident_lists() {
        let negative = layer(1, true, -2);
        let normal = layer(2, false, 0);
        let positive = layer(3, true, 1);
        let front = layer(4, true, 7);
        let pointer = |node: &PaintTreeNode<'static>| node as *const _ as *const ();
        let root = layer(0, true, 0);
        *root.hit_test_layer_order.borrow_mut() = Some(ResidentPaintLayerOrder {
            layers: vec![
                pointer(&negative),
                pointer(&normal),
                pointer(&positive),
                pointer(&front),
            ],
            controls_after: HashMap::from([(77, vec![pointer(&normal), pointer(&positive)])]),
        });
        let slots = PaintPropertyNodeStore::default();
        let source = ResidentPaintHitTest {
            node: &root,
            slots: &slots,
        };
        let selection = ResidentLayerSelection {
            negative_z: false,
            normal_flow: true,
            positive_z: true,
        };
        let ids: Vec<_> = source
            .LayerOrder(selection, None, true)
            .map(|node| node.Fragment().node_id)
            .collect();
        assert_eq!(ids, [4, 3, 2]);
        // The controls map is stored in paint order. Source HitTestChildren
        // reverses its borrowed iterator exactly once.
        let ids: Vec<_> = source
            .LayerOrder(selection, Some(77), false)
            .rev()
            .map(|node| node.Fragment().node_id)
            .collect();
        assert_eq!(ids, [3, 2]);
    }
}
