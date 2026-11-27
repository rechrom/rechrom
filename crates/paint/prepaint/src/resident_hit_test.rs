//! Borrowed hit-test access to the committed FragmentData/ObjectPaintProperties.
//!
//! This cursor does not expose the retained PrePaint tree, its child storage,
//! or a mutable owner. Lookup uses the existing DisplayItemClient directory
//! and verifies source identity; hit testing must prepare lifecycle first.

use layout::fragment::ResidentBoxGeometry as _;
use foundation::{Member, PhysicalOffset};
use layout::output::FragmentCursor;
use layout::paint::{FragmentPaintData, PaintFragmentCursor};
use layout::views::paint_layer::PaintLayer;
use paint_model::{
    paint_property_tree::{
        ClipPaintPropertyNode, EffectPaintPropertyNode, PaintPropertyKey, PaintPropertyNodeStore,
        PaintPropertyOwner, PaintPropertyRole, PropertyTreeState, TransformPaintPropertyNode,
    },
    PaintRect,
};

use crate::fragment_data::PrePaintFragmentData;
use crate::object_paint_properties::{ObjectPaintProperties, ObjectPaintPropertySlot};
use crate::pre_paint_tree_walk::{PaintTreeNode, PrePaintTreeWalk};
use std::{cell::Ref, collections::HashMap, sync::Arc};

/// Source PaintLayerStackingNode vectors. These are references to resident
/// derived Paint owners, not copies of fragments or a hit-test-only tree.
///
/// Like PaintLayerStackingNode's lists they are retained until dirtied
/// (z_order_lists_dirty_, paint_layer_stacking_node.h:191): PrePaint drops
/// them (ResetPostPaintData, the DirtyZOrderLists of
/// paint_layer_stacking_node.cc:67-83) before a topology or stacking change
/// is observable, and the paint walk and hit testing rebuild them only when
/// absent (UpdateZOrderLists, paint_layer_stacking_node.cc:436-439).
pub(crate) struct ResidentPaintLayerOrder {
    layers: Vec<*const ()>,
    controls_after: HashMap<u64, Vec<*const ()>>,
    paint: ResidentPaintOrderLists,
}

/// The paint walk's form of the same lists: each z-order entry's property
/// ancestors (the non-layer PrePaint nodes between the stacking root and the
/// child layer, which PaintLayerPainter re-enters), and
/// layer_to_overlay_overflow_controls_painting_after_ keyed by the target
/// layer node. Ancestors are one flattened buffer addressed by ranges, so a
/// retained list owns a fixed number of allocations however many layers it
/// names.
#[derive(Default)]
pub struct ResidentPaintOrderLists {
    ancestors: Vec<*const ()>,
    /// Parallel to `ResidentPaintLayerOrder::layers`.
    entry_ancestors: Vec<(u32, u32)>,
    controls_after: HashMap<*const (), Vec<(*const (), (u32, u32))>>,
}

impl ResidentPaintOrderLists {
    /// Append one entry's property ancestors; returns their range.
    pub fn PushAncestors(&mut self, ancestors: impl IntoIterator<Item = *const ()>) -> (u32, u32) {
        let start = self.ancestors.len() as u32;
        self.ancestors.extend(ancestors);
        (start, self.ancestors.len() as u32)
    }

    /// Record the ancestor range of the z-order entry at the next layer index.
    pub fn PushEntry(&mut self, ancestors: (u32, u32)) {
        self.entry_ancestors.push(ancestors);
    }

    /// Record a scroller whose overlay controls paint after `target`.
    pub fn PushControlsAfter(
        &mut self,
        target: *const (),
        scroller: *const (),
        ancestors: (u32, u32),
    ) {
        self.controls_after
            .entry(target)
            .or_default()
            .push((scroller, ancestors));
    }
}

/// Borrowed view of one stacking root's retained lists. Values are the
/// resident PrePaint node addresses PaintWalk published; the borrow keeps the
/// lists from being dropped or rebuilt while it is held.
pub struct ResidentPaintOrderView<'a> {
    order: Ref<'a, ResidentPaintLayerOrder>,
}

impl ResidentPaintOrderView<'_> {
    /// The z-order list in paint order: negative z, normal flow and positive
    /// z, then overscroll-area parents and top-layer layers.
    pub fn Layers(&self) -> &[*const ()] {
        &self.order.layers
    }

    /// The property ancestors of the entry at `index` in `Layers()`.
    pub fn EntryAncestors(&self, index: usize) -> &[*const ()] {
        self.Ancestors(self.order.paint.entry_ancestors[index])
    }

    pub fn Ancestors(&self, (start, end): (u32, u32)) -> &[*const ()] {
        &self.order.paint.ancestors[start as usize..end as usize]
    }

    /// layer_to_overlay_overflow_controls_painting_after_[target]: scrollers
    /// with their ancestor ranges, in collection order.
    pub fn ControlsAfter(&self, target: *const ()) -> &[(*const (), (u32, u32))] {
        self.order
            .paint
            .controls_after
            .get(&target)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}

impl PaintTreeNode<'_> {
    /// Borrow the retained lists for the paint walk
    /// (PaintLayerPaintOrderIterator over the stacking node's PosZOrderList /
    /// NegZOrderList, paint_layer_paint_order_iterator.cc:39-79). None while
    /// dirty.
    pub fn ResidentPaintOrder(&self) -> Option<ResidentPaintOrderView<'_>> {
        Ref::filter_map(self.hit_test_layer_order.borrow(), Option::as_ref)
            .ok()
            .map(|order| ResidentPaintOrderView { order })
    }

    /// PaintLayerStackingNode::DirtyZOrderLists (paint_layer_stacking_node.cc:
    /// 67-83) for this root alone: the next paint or hit-test query rebuilds.
    pub fn DirtyZOrderLists(&mut self) {
        *self.hit_test_layer_order.get_mut() = None;
    }

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
        paint: ResidentPaintOrderLists,
    ) {
        debug_assert_eq!(layers.len(), paint.entry_ancestors.len());
        *self.hit_test_layer_order.borrow_mut() = Some(ResidentPaintLayerOrder {
            layers,
            controls_after,
            paint,
        });
    }
}

#[derive(Clone, Copy)]
pub struct ResidentLayerSelection {
    pub negative_z: bool,
    pub normal_flow: bool,
    pub positive_z: bool,
}

/// Borrowed iterator over the stacking root PaintLayer's own lists, in
/// PaintLayerPaintOrderIterator order (paint_layer_paint_order_iterator.cc:
/// 39-79): NegZOrderList, the normal-flow children (PaintLayer children that
/// are not stacked), then PosZOrderList (which also holds the overscroll
/// area parents and top-layer layers, paint_layer_stacking_node.cc:283-326).
/// For overlay overflow controls it is the one
/// LayersPaintingOverlayOverflowControlsAfter list
/// (paint_layer_stacking_node.h:117-124). The lists are Layout's
/// PaintLayerStackingNode vectors read in place; each listed PaintLayer
/// resolves to its committed FragmentData owner by LayoutObject identity.
pub struct ResidentPaintLayerOrderCursor<'a> {
    walk: &'a PrePaintTreeWalk,
    root: PaintFragmentCursor<'a>,
    first: &'a [Member<PaintLayer>],
    last: &'a [Member<PaintLayer>],
    /// The next normal-flow child from the front and from the back.
    normal_front: *const PaintLayer,
    normal_back: *const PaintLayer,
    normal_len: usize,
    front: usize,
    back: usize,
    reverse: bool,
}

/// Source paint_layer_paint_order_iterator.cc:55-63: the normal-flow
/// children are the PaintLayer children that are not stacked. Returns the
/// first such layer at or after `layer` along `next`.
fn NormalFlowChild(
    mut layer: *const PaintLayer,
    next: fn(&PaintLayer) -> *mut PaintLayer,
) -> *const PaintLayer {
    while let Some(current) = unsafe { layer.as_ref() } {
        if !current.GetLayoutObject().IsStacked() {
            return layer;
        }
        layer = next(current);
    }
    std::ptr::null()
}

impl<'a> ResidentPaintLayerOrderCursor<'a> {
    fn new(
        walk: &'a PrePaintTreeWalk,
        root: PaintFragmentCursor<'a>,
        first: &'a [Member<PaintLayer>],
        normal_flow_parent: Option<&'a PaintLayer>,
        last: &'a [Member<PaintLayer>],
        reverse: bool,
    ) -> Self {
        let (normal_front, normal_back, normal_len) = match normal_flow_parent {
            Some(parent) => {
                let front = NormalFlowChild(parent.FirstChild(), PaintLayer::NextSibling);
                let back = NormalFlowChild(parent.LastChild(), PaintLayer::PreviousSibling);
                let mut len = 0;
                let mut child = front;
                while let Some(current) = unsafe { child.as_ref() } {
                    len += 1;
                    child = NormalFlowChild(current.NextSibling(), PaintLayer::NextSibling);
                }
                (front, back, len)
            }
            None => (std::ptr::null(), std::ptr::null(), 0),
        };
        Self {
            walk,
            root,
            first,
            last,
            normal_front,
            normal_back,
            normal_len,
            front: 0,
            back: first.len() + normal_len + last.len(),
            reverse,
        }
    }

    fn LayerAt(&mut self, index: usize, from_back: bool) -> *const PaintLayer {
        if index < self.first.len() {
            return self.first[index].Get();
        }
        let index = index - self.first.len();
        if index < self.normal_len {
            // The front and back walks never cross: `front < back` bounds
            // the number of normal-flow children either side takes.
            return if from_back {
                let layer = self.normal_back;
                self.normal_back = NormalFlowChild(
                    unsafe { &*layer }.PreviousSibling(),
                    PaintLayer::PreviousSibling,
                );
                layer
            } else {
                let layer = self.normal_front;
                self.normal_front =
                    NormalFlowChild(unsafe { &*layer }.NextSibling(), PaintLayer::NextSibling);
                layer
            };
        }
        self.last[index - self.normal_len].Get()
    }

    fn Take(&mut self, from_back: bool) -> Option<ResidentPaintHitTest<'a>> {
        while self.front < self.back {
            let index = if from_back {
                self.back -= 1;
                self.back
            } else {
                let index = self.front;
                self.front += 1;
                index
            };
            // The lists name live PaintLayers of this Layout result: Layout
            // rebuilds them before PrePaint (PaintLayer::
            // UpdateDescendantDependentFlags, paint_layer.cc:864-882) and the
            // borrowed PaintEngine keeps the result alive for this cursor.
            let Some(layer) = (unsafe { self.LayerAt(index, from_back).as_ref() }) else {
                continue;
            };
            // A PaintLayer whose object has no committed resident owner (no
            // fragment PrePaint reached) has nothing to hit test.
            let (walk, root): (&'a PrePaintTreeWalk, PaintFragmentCursor<'a>) =
                (self.walk, self.root);
            if let Some(resident) = root
                .ForLayoutObject(layer.GetLayoutObject())
                .and_then(|cursor| walk.ResidentLayerHitTest(cursor))
            {
                return Some(resident);
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
/// per-object freshness test: PrePaintTreeWalk consumed and cleared the
/// LayoutObject dirty bits of every object it visited
/// (pre_paint_tree_walk.cc:1584-1585). Freshness is that lifecycle gate,
/// which Rechrom's callers hold (LayoutViewHitTestQuery checks
/// IsPrePaintClean and the prepared revision; Page pointer geometry runs
/// PrePaint below PrePaintClean). This lookup proves owner identity: the
/// same resident sidecar slot (checked by the caller) and the same
/// PhysicalFragment / FragmentItem and LayoutObject the walk visited. A
/// relayout that replaces the fragment therefore rejects the resident node.
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

/// Whether the walked object changed what a hit test reads after the
/// PrePaint walk that produced its resident node.
///
/// PrePaint clears each visited object's flags (pre_paint_tree_walk.cc:
/// 1584-1585; PaintProperties::ClearSourcePaintFlags), so a live bit means a
/// mutation since that walk: a stale node even when the lifecycle gate was
/// not rewound. Only the bits that nothing but a walk turns into hit-test
/// state are checked (PrePaintInvalidation::own_hit_test_inputs, which also
/// documents the excluded paint-invalidation, delayed-invalidation and
/// directly applied property-update bits). Hand-built fixtures have no live
/// source.
fn ChangedSinceWalk(fragment: &FragmentPaintData) -> bool {
    fragment
        .paint
        .LivePrePaintInvalidation()
        .is_some_and(|bits| bits.own_hit_test_inputs())
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

    /// WithResidentPaintNode for a PaintLayer query. A PaintLayer belongs to
    /// its LayoutObject (paint_layer.h), so the layer's resident node is
    /// resolved by object identity like ResidentLayerHitTest: an inline-level
    /// layer reached through its FragmentItem cursor names the node PrePaint
    /// recorded for its box fragment.
    pub fn WithResidentLayerNode<'a, T>(
        &'a self,
        source: PaintFragmentCursor<'_>,
        query: impl FnOnce(&'a PaintTreeNode<'static>) -> T,
    ) -> Option<T> {
        self.ResidentLayerHitTest(source)
            .map(|layer| query(layer.node))
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
            if !WalkedSameSourceOwner(node, query) || ChangedSinceWalk(source) {
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
            // comparison) and not changed since (ChangedSinceWalk). Otherwise
            // freshness is the caller's PrePaintClean gate.
            if node.RecordedRevisions().is_none() || ChangedSinceWalk(fragment) {
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

impl PrePaintTreeWalk {
    /// PaintLayerPaintOrderIterator over the PaintLayer `source` names
    /// (paint_layer_paint_order_iterator.cc:39-79), or, with
    /// `controls_after`, PaintLayerStackingNode::
    /// LayersPaintingOverlayOverflowControlsAfter(child layer)
    /// (paint_layer_stacking_node.h:117-124). Both read Layout's PaintLayer
    /// and its stacking node in place, the lists Paint shares. The layer is
    /// resolved by LayoutObject identity, as ResidentLayerHitTest resolves
    /// it, so an inline-level layer reached through its FragmentItem cursor
    /// names its own PaintLayer. None when `source` (or `controls_after`)
    /// has no committed resident layer.
    pub fn ResidentLayerOrder<'a>(
        &'a self,
        source: PaintFragmentCursor<'a>,
        selection: ResidentLayerSelection,
        controls_after: Option<PaintFragmentCursor<'_>>,
        reverse: bool,
    ) -> Option<ResidentPaintLayerOrderCursor<'a>> {
        self.ResidentLayerHitTest(source)?;
        let layer = source.LayerForPaintObject(source.PaintData().paint.LayoutObjectIdentity()?)?;
        let stacking_node = layer.StackingNode();
        if let Some(child) = controls_after {
            self.ResidentLayerHitTest(child)?;
            let child_layer =
                child.LayerForPaintObject(child.PaintData().paint.LayoutObjectIdentity()?)?;
            let list = stacking_node
                .and_then(|node| {
                    node.LayersPaintingOverlayOverflowControlsAfter(child_layer as *const PaintLayer)
                })
                .map_or(&[][..], |list| &list[..]);
            return Some(ResidentPaintLayerOrderCursor::new(
                self, source, list, None, &[], reverse,
            ));
        }
        // Source paint_layer_paint_order_iterator.cc:39-79: the stacking
        // node's lists exist only on a stacking context with stacked
        // descendants (paint_layer.cc:864-882); without one they are empty.
        let negative = stacking_node
            .filter(|_| selection.negative_z)
            .map_or(&[][..], |node| &node.NegZOrderList()[..]);
        let positive = stacking_node
            .filter(|_| selection.positive_z)
            .map_or(&[][..], |node| &node.PosZOrderList()[..]);
        let normal_flow = selection.normal_flow.then_some(layer);
        Some(ResidentPaintLayerOrderCursor::new(
            self,
            source,
            negative,
            normal_flow,
            positive,
            reverse,
        ))
    }
}

/// The node of an ObjectPaintProperties transform slot, borrowed from the
/// FragmentData that owns it (object_paint_properties.h:117-139).
fn TransformSlot(
    properties: &ObjectPaintProperties,
    slot: ObjectPaintPropertySlot,
) -> Option<&Arc<TransformPaintPropertyNode>> {
    use ObjectPaintPropertySlot as S;
    match slot {
        S::PaintOffsetTranslation => properties.PaintOffsetTranslation(),
        S::StickyTranslation => properties.StickyTranslation(),
        S::AnchorPositionScrollTranslation => properties.AnchorPositionScrollTranslation(),
        S::ElementCanvasTransform => properties.ElementCanvasTransform(),
        S::Translate => properties.Translate(),
        S::Rotate => properties.Rotate(),
        S::Scale => properties.Scale(),
        S::Offset => properties.Offset(),
        S::Transform => properties.Transform(),
        S::ContentTranslation => properties.ContentTranslation(),
        S::Perspective => properties.Perspective(),
        S::ReplacedContentTransform => properties.ReplacedContentTransform(),
        S::ScrollTranslation => properties.ScrollTranslation(),
        S::UnboundedWrapperTransform => properties.UnboundedWrapperTransform(),
        S::UnboundedInnerTransform => properties.UnboundedInnerTransform(),
        S::SvgLocalTransform => properties.SvgLocalTransform(),
        S::TextCombineTransform => properties.TextCombineTransform(),
        _ => None,
    }
}

/// The node of an ObjectPaintProperties clip slot, borrowed from the
/// FragmentData that owns it (object_paint_properties.h:161-174).
fn ClipSlot(
    properties: &ObjectPaintProperties,
    slot: ObjectPaintPropertySlot,
) -> Option<&Arc<ClipPaintPropertyNode>> {
    use ObjectPaintPropertySlot as S;
    match slot {
        S::ClipPathClip => properties.ClipPathClip(),
        S::MaskClip => properties.MaskClip(),
        S::CssClip => properties.CssClip(),
        S::OverflowControlsClip => properties.OverflowControlsClip(),
        S::BackgroundClip => properties.BackgroundClip(),
        S::PixelMovingFilterClipExpander => properties.PixelMovingFilterClipExpander(),
        S::InnerBorderRadiusClip => properties.InnerBorderRadiusClip(),
        S::OverflowClip => properties.OverflowClip(),
        S::CssClipFixedPosition => properties.CssClipFixedPosition(),
        S::InnerBorderShapeClip => properties.InnerBorderShapeClip(),
        S::BackdropFilterClip => properties.BackdropFilterClip(),
        _ => None,
    }
}

impl<'a> ResidentPaintHitTest<'a> {
    /// The occurrence's committed FragmentData (fragment_data.h), which
    /// PrePaint keeps current for every visited and retained object. None
    /// only for hand-built nodes, which keep their own copies.
    fn Data(self) -> Option<&'a PrePaintFragmentData> {
        self.node.FragmentData()
    }

    /// PaintLayer::NeedsReorderOverlayOverflowControls (paint_layer.h:
    /// 427-431), written by the layer's stacking node.
    pub fn NeedsReorderOverlayControls(self) -> bool {
        match self.node.SourceCursor().and_then(|cursor| cursor.OwnPaintLayer()) {
            Some(layer) => layer.NeedsReorderOverlayOverflowControls(),
            None => self.node.needs_reorder_overlay_overflow_controls.get(),
        }
    }

    pub fn LayoutObjectIdentity(
        self,
    ) -> Option<layout::paint::PaintLayoutObjectIdentity<'a>> {
        self.Fragment().paint.LayoutObjectIdentity()
    }

    /// Source: LayoutObject::HasNonZeroEffectiveOpacity(), layout_object.cc
    /// 2363-2385. Read the committed effect ancestry, including inherited
    /// effects, rather than reconstructing effective opacity from CSS.
    pub fn HasNonZeroEffectiveOpacity(self) -> bool {
        paint_model::property_cell::with_access(|access| {
            let mut effect = Some(self.LocalBorderBoxProperties().effect.as_ref());
            while let Some(current) = effect {
                if current.state(access).opacity == 0.0 {
                    return false;
                }
                effect = current.Parent(access).map(|parent| &**parent);
            }
            true
        })
    }


    /// fragment_data.h:104-110 `PaintProperties()` holds a node.
    pub fn HasPaintProperties(self) -> bool {
        match self.Data() {
            Some(data) => data.HasPaintProperties(),
            None => self.node.HasOwnPaintProperties(),
        }
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

    /// FragmentData::PaintOffset() (fragment_data.h:32-38) in the flat
    /// replay space hit testing shares with the recorded ops
    /// (`PaintTreeNode::FlatPaintOffset`).
    pub fn PaintOffset(self) -> PhysicalOffset {
        let offset = self.node.FlatPaintOffsetOf(self.Data());
        PhysicalOffset::new(
            foundation::LayoutUnit::FromFloatRound(offset.x as f32),
            foundation::LayoutUnit::FromFloatRound(offset.y as f32),
        )
    }

    /// fragment_data.h:137-142.
    pub fn LocalBorderBoxProperties(self) -> &'a PropertyTreeState {
        match self.Data().and_then(PrePaintFragmentData::LocalBorderBoxPropertiesRef) {
            Some(state) => state,
            None => &self.node.properties.nodes,
        }
    }
    /// fragment_data.h:170-177.
    pub fn ContentsProperties(self) -> PropertyTreeState {
        match self.Data() {
            Some(data) => data.ContentsProperties(),
            None => self.node.contents_properties.nodes.clone(),
        }
    }
    // Source: fragment_data.cc:89-109. Property parents keep their exact owner.
    pub fn PreClip(self) -> Arc<ClipPaintPropertyNode> {
        if let Some(data) = self.Data() {
            return paint_model::property_cell::with_access(|access| data.PreClip(access));
        }
        for role in [
            PaintPropertyRole::ClipPathClip,
            PaintPropertyRole::MaskClip,
            PaintPropertyRole::CssClip,
            PaintPropertyRole::PixelMovingFilterClipExpander,
        ] {
            if let Some(clip) = self.Clip(role) {
                return paint_model::property_cell::with_access(|access| {
                    clip.Parent(access).expect("non-root own clip has parent").clone()
                });
            }
        }
        self.node.properties.nodes.clip.clone()
    }
    // Source: fragment_data.cc:123-135.
    pub fn PreEffect(self) -> Arc<EffectPaintPropertyNode> {
        if let Some(data) = self.Data() {
            return paint_model::property_cell::with_access(|access| data.PreEffect(access));
        }
        for role in [PaintPropertyRole::Effect, PaintPropertyRole::Filter] {
            if let Some(effect) = self.slots.EffectForKey(self.Key(role)) {
                return paint_model::property_cell::with_access(|access| {
                    effect.Parent(access).expect("non-root own effect has parent").clone()
                });
            }
        }
        self.node.properties.nodes.effect.clone()
    }

    pub fn LocalBorderTransform(self) -> &'a Arc<TransformPaintPropertyNode> {
        &self.LocalBorderBoxProperties().transform
    }

    /// fragment_data.cc:67-87 `ContentsTransform`, borrowed from the slot
    /// that owns it.
    pub fn ContentsTransform(self) -> &'a Arc<TransformPaintPropertyNode> {
        let Some(data) = self.Data() else {
            return &self.node.contents_properties.nodes.transform;
        };
        data.PaintProperties()
            .and_then(|properties| {
                properties
                    .ScrollTranslation()
                    .or_else(|| properties.ReplacedContentTransform())
                    .or_else(|| properties.Perspective())
                    .or_else(|| properties.ContentTranslation())
            })
            .unwrap_or_else(|| self.LocalBorderTransform())
    }

    pub fn LocalBorderClip(self) -> &'a ClipPaintPropertyNode {
        &self.LocalBorderBoxProperties().clip
    }

    /// fragment_data.cc:111-121 `ContentsClip`, borrowed from the slot that
    /// owns it.
    pub fn ContentsClip(self) -> &'a ClipPaintPropertyNode {
        let Some(data) = self.Data() else {
            return &self.node.contents_properties.nodes.clip;
        };
        data.PaintProperties()
            .and_then(|properties| {
                properties
                    .OverflowClip()
                    .or_else(|| properties.InnerBorderRadiusClip())
            })
            .unwrap_or_else(|| &self.LocalBorderBoxProperties().clip)
    }

    /// fragment_data.cc:55-65 `PreTransform`. The parent link of the
    /// outermost CSS transform node can only be read under the property
    /// access proof, so the borrowed form still reads the walk's copy.
    pub fn PreTransform(self) -> &'a Arc<TransformPaintPropertyNode> {
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

    /// The object's own transform node of `role`: its ObjectPaintProperties
    /// slot (object_paint_properties.h:117-139).
    pub fn Transform(self, role: PaintPropertyRole) -> Option<&'a Arc<TransformPaintPropertyNode>> {
        match self.Data() {
            Some(data) => TransformSlot(
                data.PaintProperties()?,
                ObjectPaintPropertySlot::FromRole(role)?,
            ),
            None => self.slots.TransformOwnerForKey(self.Key(role)),
        }
    }

    /// The object's own clip node of `role`: its ObjectPaintProperties slot
    /// (object_paint_properties.h:161-174).
    pub fn Clip(self, role: PaintPropertyRole) -> Option<&'a ClipPaintPropertyNode> {
        match self.Data() {
            Some(data) => ClipSlot(
                data.PaintProperties()?,
                ObjectPaintPropertySlot::FromRole(role)?,
            )
            .map(|clip| &**clip),
            None => self.slots.ClipForKey(self.Key(role)),
        }
    }

    /// The flat accumulated background clip the walk computed for this
    /// object (PaintLayerClipper's background rect source).
    pub fn BackgroundClip(self) -> Option<PaintRect> {
        self.node.clip
    }

    pub fn ForegroundClip(self) -> Option<PaintRect> {
        paint_model::property_cell::with_access(|a| self.ContentsClip().PaintClipRect(a))
    }
}
