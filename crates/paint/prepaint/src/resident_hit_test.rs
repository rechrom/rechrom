//! Borrowed hit-test access to the committed FragmentData/ObjectPaintProperties.
//!
//! This cursor exposes no mutable owner. Lookup reads the queried fragment
//! occurrence's FragmentData (or, for a PaintLayer, its LayoutObject's first
//! fragment through Layout) and verifies the walked owner identity; hit
//! testing must prepare lifecycle first.

use layout::fragment::ResidentBoxGeometry as _;
use foundation::{Member, PhysicalOffset};
use layout::output::FragmentCursor;
use layout::paint::{FragmentPaintData, PaintFragmentCursor};
use layout::views::paint_layer::PaintLayer;
use paint_model::{
    paint_property_tree::{
        ClipPaintPropertyNode, EffectPaintPropertyNode, PaintPropertyRole, PropertyTreeState,
        TransformPaintPropertyNode,
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

/// One committed occurrence for hit testing: its fragment cursor and the
/// FragmentData the last PrePaint walk produced for it.
#[derive(Clone, Copy)]
pub struct ResidentPaintHitTest<'a> {
    cursor: PaintFragmentCursor<'a>,
    data: &'a PrePaintFragmentData,
}

/// A query cursor at the lifetime of the PrePaint borrow. The committed
/// walk's Layout lease keeps every FragmentData a lookup reaches alive while
/// PrePaint is borrowed, as it kept the nodes of the former retained tree.
unsafe fn AtWalkLifetime<'a>(cursor: PaintFragmentCursor<'_>) -> PaintFragmentCursor<'a> {
    unsafe { std::mem::transmute::<PaintFragmentCursor<'_>, PaintFragmentCursor<'a>>(cursor) }
}

/// Whether the occurrence's FragmentData was walked for exactly the live
/// Layout owner `query` names.
///
/// Chromium hit tests read committed FragmentData/ObjectPaintProperties only
/// after LocalFrameView::UpdateLifecycleToPrePaintClean
/// (core/frame/local_frame_view.cc; DocumentLifecycle::kPrePaintClean,
/// core/dom/document_lifecycle.h), and then trust them without any
/// per-object freshness test: PrePaintTreeWalk consumed and cleared the
/// LayoutObject dirty bits of every object it visited
/// (pre_paint_tree_walk.cc:1584-1585). Freshness is that lifecycle gate,
/// which Rechrom's callers hold. This proves owner identity: the walk
/// recorded the same PhysicalFragment / FragmentItem the query names, so a
/// relayout that replaced the fragment rejects the stale FragmentData.
/// Container offsets are not compared: call-scoped cursors rebound with
/// ForPhysicalFragment carry no link offset.
fn WalkedSameSourceOwner(data: &PrePaintFragmentData, query: PaintFragmentCursor<'_>) -> bool {
    let owner = match query.Cursor() {
        FragmentCursor::Physical { fragment, .. } => fragment as *const _ as usize,
        FragmentCursor::Inline { item, .. } => item as *const _ as usize,
        #[allow(unreachable_patterns)]
        _ => return false,
    };
    owner != 0 && data.WalkRecord().walked_owner.0 == owner
}

/// Whether the walked object changed what a hit test reads after the
/// PrePaint walk that produced its FragmentData.
///
/// PrePaint clears each visited object's flags (pre_paint_tree_walk.cc:
/// 1584-1585; PaintProperties::ClearSourcePaintFlags), so a live bit means a
/// mutation since that walk: stale FragmentData even when the lifecycle gate
/// was not rewound. Only the bits that nothing but a walk turns into
/// hit-test state are checked (PrePaintInvalidation::own_hit_test_inputs).
/// Hand-built fixtures have no live source.
fn ChangedSinceWalk(fragment: &FragmentPaintData) -> bool {
    fragment
        .paint
        .LivePrePaintInvalidation()
        .is_some_and(|bits| bits.own_hit_test_inputs())
}

impl PrePaintTreeWalk {
    /// The committed FragmentData of the occurrence `source` names, read in
    /// place through the fragment cursor (no client directory).
    pub fn ResidentHitTest(
        &self,
        source: PaintFragmentCursor<'_>,
    ) -> Option<ResidentPaintHitTest<'_>> {
        let source = unsafe { AtWalkLifetime(source) };
        let data = source
            .PrePaintData::<PrePaintFragmentData>()
            .filter(|data| data.WasWalked() && data.HasLocalBorderBoxProperties())?;
        if !WalkedSameSourceOwner(data, source) || ChangedSinceWalk(source.PaintData()) {
            return None;
        }
        Some(ResidentPaintHitTest { cursor: source, data })
    }

    /// Resolve a source PaintLayer for a fragment occurrence. A PaintLayer
    /// belongs to its LayoutObject (paint_layer.h): LayoutNG keeps the
    /// FragmentItem and its linked PhysicalFragment as distinct paint
    /// occurrences of the same LayoutObject, so an inline-level layer
    /// reached through its FragmentItem cursor resolves, through Layout's
    /// LayoutObject lookup (ForPaintObject), to the box fragment the walk
    /// built the layer's properties on.
    pub fn ResidentLayerHitTest(
        &self,
        source: PaintFragmentCursor<'_>,
    ) -> Option<ResidentPaintHitTest<'_>> {
        let source = unsafe { AtWalkLifetime(source) };
        if let Some(exact) = self.ResidentHitTest(source) {
            if exact.data.WalkRecord().is_paint_layer {
                return Some(exact);
            }
        }
        let identity = source.PaintData().paint.LayoutObjectIdentity()?;
        let layer = source.ForPaintObject(identity)?;
        let data = layer
            .PrePaintData::<PrePaintFragmentData>()
            .filter(|data| {
                data.WasWalked()
                    && data.HasLocalBorderBoxProperties()
                    && data.WalkRecord().is_paint_layer
            })?;
        // The layer occurrence is found through its LayoutObject, not
        // through the queried fragment; it must have been visited by a walk
        // and not changed since (ChangedSinceWalk). Otherwise freshness is
        // the caller's PrePaintClean gate.
        if ChangedSinceWalk(layer.PaintData()) {
            return None;
        }
        Some(ResidentPaintHitTest { cursor: layer, data })
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
    /// The occurrence's committed FragmentData (fragment_data.h).
    fn Data(self) -> &'a PrePaintFragmentData {
        self.data
    }

    /// PaintLayer::NeedsReorderOverlayOverflowControls (paint_layer.h:
    /// 427-431), written by the layer's stacking node.
    pub fn NeedsReorderOverlayControls(self) -> bool {
        self.cursor
            .OwnPaintLayer()
            .is_some_and(|layer| layer.NeedsReorderOverlayOverflowControls())
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
        self.Data().HasPaintProperties()
    }

    /// The identity of the occurrence: its FragmentData's address.
    pub fn Identity(self) -> usize {
        self.data as *const PrePaintFragmentData as usize
    }

    pub fn Fragment(self) -> &'a FragmentPaintData {
        self.cursor.PaintData()
    }

    /// FragmentData::PaintOffset() (fragment_data.h:32-38) in the flat
    /// replay space hit testing shares with the recorded ops: the local
    /// offset moved back by the flat-to-property delta.
    pub fn PaintOffset(self) -> PhysicalOffset {
        let data = self.Data();
        let local = data.PaintOffset();
        let snap = data.FlatReplay().paint_snap_offset;
        let x = local.left.ToDouble() - snap.x;
        let y = local.top.ToDouble() - snap.y;
        PhysicalOffset::new(
            foundation::LayoutUnit::FromFloatRound(x as f32),
            foundation::LayoutUnit::FromFloatRound(y as f32),
        )
    }

    /// fragment_data.h:137-142. The lookup admits only occurrences whose
    /// LocalBorderBoxProperties were set.
    pub fn LocalBorderBoxProperties(self) -> &'a PropertyTreeState {
        self.Data()
            .LocalBorderBoxPropertiesRef()
            .expect("resident hit test occurrence has LocalBorderBoxProperties")
    }
    /// fragment_data.h:170-177.
    pub fn ContentsProperties(self) -> PropertyTreeState {
        self.Data().ContentsProperties()
    }
    // Source: fragment_data.cc:89-109. Property parents keep their exact owner.
    pub fn PreClip(self) -> Arc<ClipPaintPropertyNode> {
        paint_model::property_cell::with_access(|access| self.Data().PreClip(access))
    }
    // Source: fragment_data.cc:123-135.
    pub fn PreEffect(self) -> Arc<EffectPaintPropertyNode> {
        paint_model::property_cell::with_access(|access| self.Data().PreEffect(access))
    }

    pub fn LocalBorderTransform(self) -> &'a Arc<TransformPaintPropertyNode> {
        &self.LocalBorderBoxProperties().transform
    }

    /// fragment_data.cc:67-87 `ContentsTransform`, borrowed from the slot
    /// that owns it.
    pub fn ContentsTransform(self) -> &'a Arc<TransformPaintPropertyNode> {
        self.Data()
            .PaintProperties()
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
        self.Data()
            .PaintProperties()
            .and_then(|properties| {
                properties
                    .OverflowClip()
                    .or_else(|| properties.InnerBorderRadiusClip())
            })
            .unwrap_or_else(|| &self.LocalBorderBoxProperties().clip)
    }

    /// fragment_data.cc:55-65 `PreTransform`. The parent link of the
    /// outermost CSS transform node can only be read under the property
    /// access proof, so the borrowed form reads the node the walk stored
    /// beside the occurrence's flat replay values.
    pub fn PreTransform(self) -> &'a Arc<TransformPaintPropertyNode> {
        self.Data()
            .FlatReplay()
            .Rare()
            .and_then(|rare| rare.pre_transform.as_ref())
            .unwrap_or_else(|| self.LocalBorderTransform())
    }

    /// The object's own transform node of `role`: its ObjectPaintProperties
    /// slot (object_paint_properties.h:117-139).
    pub fn Transform(self, role: PaintPropertyRole) -> Option<&'a Arc<TransformPaintPropertyNode>> {
        TransformSlot(
            self.Data().PaintProperties()?,
            ObjectPaintPropertySlot::FromRole(role)?,
        )
    }

    /// The object's own clip node of `role`: its ObjectPaintProperties slot
    /// (object_paint_properties.h:161-174).
    pub fn Clip(self, role: PaintPropertyRole) -> Option<&'a ClipPaintPropertyNode> {
        ClipSlot(
            self.Data().PaintProperties()?,
            ObjectPaintPropertySlot::FromRole(role)?,
        )
        .map(|clip| &**clip)
    }

    /// The flat accumulated background clip the walk computed for this
    /// object (PaintLayerClipper's background rect source).
    pub fn BackgroundClip(self) -> Option<PaintRect> {
        self.Data().FlatReplay().clip
    }

    pub fn ForegroundClip(self) -> Option<PaintRect> {
        paint_model::property_cell::with_access(|a| self.ContentsClip().PaintClipRect(a))
    }
}
