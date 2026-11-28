//! The painted object as Chromium's painters receive it.
//!
//! Chromium painters take the LayoutObject / PhysicalBoxFragment being
//! painted, the FragmentData of the painted occurrence and a PaintInfo
//! (box_fragment_painter.h:43-60, scoped_paint_state.h:24-60,
//! object_painter.h:21-27). `PaintObject` is that borrowed pair for
//! paint_record: a small `Copy` view over the source fragment cursor, its
//! paint sidecar (`cursor.PaintData()`), the occurrence's
//! `PrePaintFragmentData`, the source `PaintLayer`, and a `PaintWalkContext`
//! holding the paint traversal state Chromium keeps in PaintInfo and
//! PaintLayerPainter (the inherited cull rect, the in-flight previous paint
//! result).
//!
//! Every value is read from the owner Chromium reads: the cursor and its
//! LayoutObject / PaintLayer / DocumentResources, the FragmentData
//! (PaintOffset, ObjectPaintProperties slots, LocalBorderBoxProperties,
//! PaintLayer cull rects) and, for Rechrom's flat replay stream only, the
//! FragmentData's `FlatReplayData`. The paint walk builds each one with
//! `FromCursor(cursor, walk)` (or `ResolvedFragment::Object`). Unit tests
//! build hand-made objects with `Fixture(node)`: a PrePaint node without
//! FragmentData, every value read from the node's own copy.

#![allow(non_snake_case)]

use std::cell::Cell;
use std::sync::Arc;

use layout::fragment::FragmentKind;
use layout::fragment::ResidentBoxGeometry as _;
use layout::paint::{FragmentPaintData, PaintFragmentCursor};
use layout::views::paint_layer::PaintLayer;
use paint_input_types::layout_input::{
    Offset, PaintClipPathChild, PaintPathCommand, TransformMatrix,
};
use paint_input_types::paint_input::{PaintBlendMode, PaintCornerRadii};
use paint_input_types::PaintResources;
use paint_model::paint_property_tree::{
    ClipPaintPropertyNode, PaintPropertyRole, PropertyTreeState as PaintPropertyTreeState,
    TransformPaintPropertyNode,
};
use paint_model::property_cell::with_access;
use paint_model::{DisplayMaskLayer, PaintResult};
use prepaint::fragment_data::{FlatReplayData, FlatReplayRareData, PrePaintFragmentData};
use prepaint::object_paint_properties::{ObjectPaintProperties, ObjectPaintPropertySlot};
use prepaint::pre_paint_tree_walk::{
    EnclosingRectInPropertySpace, PaintTreeNode, PixelSnappedRectInPropertySpace, SnapToPixel,
};

use crate::pre_paint_tree_walk::MaskAdjustedOpacity;
use crate::PaintRect;

/// The first resident document-order position in a fragment subtree: the
/// fragment's own source LayoutObject's (LayoutObject::LogicalTreeOrder), else
/// the smallest of its children's, else 0 (no source with a Node).
fn FirstLogicalTreeOrder(cursor: PaintFragmentCursor<'_>) -> u64 {
    FirstLogicalTreeOrderOf(cursor, cursor.PaintData())
}

/// FirstLogicalTreeOrder with the fragment's paint sidecar already resolved.
fn FirstLogicalTreeOrderOf(cursor: PaintFragmentCursor<'_>, fragment: &FragmentPaintData) -> u64 {
    let order = fragment.paint.LogicalTreeOrder();
    if order != 0 {
        return order;
    }
    let mut first = u64::MAX;
    for child in cursor.Children() {
        let child_order = FirstLogicalTreeOrder(child);
        if child_order != 0 {
            first = first.min(child_order);
        }
    }
    if first == u64::MAX {
        0
    } else {
        first
    }
}

/// A fragment cursor with its paint sidecar and FragmentData resolved once
/// (one sidecar lookup), as a Chromium painter holds its PhysicalFragment
/// and reads its FragmentData once per Paint call. Building a `PaintObject`
/// from it reads nothing again.
#[derive(Clone, Copy)]
pub struct ResolvedFragment<'a> {
    pub cursor: PaintFragmentCursor<'a>,
    pub fragment: &'a FragmentPaintData,
    pub data: Option<&'a PrePaintFragmentData>,
}

impl<'a> ResolvedFragment<'a> {
    pub fn new(cursor: PaintFragmentCursor<'a>) -> Self {
        let (fragment, data) = cursor.PaintDataAndPrePaintData::<PrePaintFragmentData>();
        Self {
            cursor,
            fragment,
            data,
        }
    }

    /// The painted object of this occurrence with `walk`
    /// (`PaintObject::FromCursor` without its lookups).
    pub fn Object<'f>(self, walk: PaintWalkContext<'a>) -> PaintObject<'a, 'f> {
        PaintObject {
            cursor: Some(self.cursor),
            fragment: Some(self.fragment),
            data: self.data,
            walk: Some(walk),
            fixture: None,
        }
    }

    /// Whether `other` is the same fragment occurrence (the same sidecar).
    pub fn IsSameFragment(self, other: ResolvedFragment<'_>) -> bool {
        std::ptr::eq(self.fragment, other.fragment)
    }
}

/// Whether nonzero `keys` are already in non-decreasing order.
fn KeysOrdered(keys: impl Iterator<Item = u64>) -> bool {
    let mut previous = 0;
    for order in keys {
        if order == 0 {
            continue;
        }
        if order < previous {
            return false;
        }
        previous = order;
    }
    true
}

/// The painted children of a fragment in paint order: the fragment cursor's
/// children (box children, then FragmentItems; BoxFragmentPainter's
/// PaintBlockChildren / PaintInlineItems traversal), each resolved once,
/// with the children that have a document-order position stably sorted by
/// it in their own slots.
///
/// Not Chromium: Chromium paints fragment children in fragment order. The
/// sort keeps the paint order PrePaint gave its node tree
/// (RestoreLogicalSiblingOrder) until that order is retired; children
/// already in order (nearly always) are returned without sorting. The order
/// test reads each child's own LogicalTreeOrder from the sidecar already
/// resolved for painting; only a child without a source Node descends.
pub fn LogicalPaintChildren<'a>(cursor: PaintFragmentCursor<'a>) -> Vec<ResolvedFragment<'a>> {
    let mut children: Vec<ResolvedFragment<'a>> =
        cursor.Children().map(ResolvedFragment::new).collect();
    if children.len() < 2 {
        return children;
    }
    let key = |child: &ResolvedFragment<'a>| FirstLogicalTreeOrderOf(child.cursor, child.fragment);
    if KeysOrdered(children.iter().map(key)) {
        return children;
    }
    let keys: Vec<u64> = children.iter().map(key).collect();
    let slots: Vec<usize> = (0..children.len()).filter(|&index| keys[index] != 0).collect();
    let mut ordered: Vec<(u64, ResolvedFragment<'a>)> =
        slots.iter().map(|&index| (keys[index], children[index])).collect();
    // Stable, as the node tree's sort_by_key.
    ordered.sort_by_key(|(order, _)| *order);
    for (slot, (_, child)) in slots.into_iter().zip(ordered) {
        children[slot] = child;
    }
    children
}

/// The paint traversal state of one painted occurrence that Chromium keeps
/// outside FragmentData: PaintInfo's cull rect as the paint parent's
/// ScopedBoxContentsPaintState left it (paint_info.h GetCullRect,
/// scoped_paint_state.cc:139-150), and the PaintLayerPainter's in-flight
/// PaintLayer::PreviousPaintResult that ScopedBoxContentsPaintState
/// downgrades (scoped_paint_state.cc:152-164).
#[derive(Clone, Copy, Default)]
pub struct PaintWalkContext<'n> {
    /// The paint parent's contents cull rect; None is infinite.
    inherited_cull_rect: Option<PaintRect>,
    previous_paint_result: Option<&'n Cell<PaintResult>>,
}

impl<'n> PaintWalkContext<'n> {
    pub fn new(
        inherited_cull_rect: Option<PaintRect>,
        previous_paint_result: Option<&'n Cell<PaintResult>>,
    ) -> Self {
        Self {
            inherited_cull_rect,
            previous_paint_result,
        }
    }
}

/// One painted object occurrence: source fragment + FragmentData + walk state.
#[derive(Clone, Copy)]
pub struct PaintObject<'n, 'f> {
    cursor: Option<PaintFragmentCursor<'n>>,
    fragment: Option<&'n FragmentPaintData>,
    data: Option<&'n PrePaintFragmentData>,
    /// None for a fixture, whose walk-context values are the node's.
    walk: Option<PaintWalkContext<'n>>,
    /// The hand-built PrePaint node of a unit-test fixture.
    fixture: Option<&'n PaintTreeNode<'f>>,
}

impl<'n, 'f> PaintObject<'n, 'f> {
    /// A hand-built unit-test fixture: a PrePaint node outside any walk,
    /// whose every value (walk context included) is the node's own copy.
    #[doc(hidden)]
    pub fn Fixture(node: &'n PaintTreeNode<'f>) -> Self {
        Self {
            cursor: node.SourceCursor(),
            fragment: node.fragment.as_deref(),
            data: node.FragmentData(),
            walk: None,
            fixture: Some(node),
        }
    }

    /// An occurrence reached through the fragment cursor alone.
    pub fn FromCursor(cursor: PaintFragmentCursor<'n>, walk: PaintWalkContext<'n>) -> Self {
        ResolvedFragment::new(cursor).Object(walk)
    }

    /// The fixture node as the source of every value: an occurrence without
    /// FragmentData.
    fn FixtureNode(self) -> Option<&'n PaintTreeNode<'f>> {
        if self.data.is_some() {
            None
        } else {
            self.fixture
        }
    }

    /// The occurrence's flat replay values (FragmentData FlatReplayData).
    fn Flat(self) -> Option<&'n FlatReplayData> {
        self.data.map(PrePaintFragmentData::FlatReplay)
    }

    fn FlatRare(self) -> Option<&'n FlatReplayRareData> {
        self.Flat()?.Rare()
    }

    // -----------------------------------------------------------------
    // Source object and FragmentData.
    // -----------------------------------------------------------------

    /// The borrowed fragment cursor of the painted object (the walk handle).
    pub fn SourceCursor(self) -> Option<PaintFragmentCursor<'n>> {
        self.cursor
    }

    /// The fragment's paint sidecar (`cursor.PaintData()`): the paint view of
    /// the PhysicalBoxFragment / FragmentItem and its LayoutObject.
    pub fn Fragment(self) -> Option<&'n FragmentPaintData> {
        self.fragment
    }

    /// FragmentData of this occurrence (core/paint/fragment_data.h).
    pub fn FragmentData(self) -> Option<&'n PrePaintFragmentData> {
        self.data
    }

    /// `To<LayoutBoxModelObject>(object).Layer()` of the source object.
    pub fn PaintLayer(self) -> Option<&'n PaintLayer> {
        self.cursor?.OwnPaintLayer()
    }

    /// Document / LocalFrameView paint resources of the source object.
    pub fn DocumentResources(self) -> Option<&'n Arc<PaintResources>> {
        match self.cursor {
            Some(cursor) => cursor.DocumentResources(),
            None => self.fragment?.resources.as_ref(),
        }
    }

    /// The active ScopedDisplayItemFragment (box_fragment_painter.cc:394-403,
    /// paint_layer_painter.cc:776-778, inline_box_fragment_painter.cc:63-64):
    /// Layout resolves it on the paint sidecar from the scope the fragment
    /// cursor walk carries (layout_output.rs PhysicalDisplayItemFragment).
    pub fn DisplayItemFragment(self) -> u32 {
        if let Some(node) = self.FixtureNode() {
            return node.DisplayItemFragment();
        }
        self.fragment
            .map_or(0, |fragment| fragment.ViewDisplayItemFragment())
    }

    /// Whether ObjectPaintProperties holds the slot of `role`
    /// (object_paint_properties.h `HasNode`).
    pub fn HasOwnPropertyRole(self, role: PaintPropertyRole) -> bool {
        let Some(data) = self.data else {
            return self.fixture.is_some_and(|node| node.HasOwnPropertyRole(role));
        };
        data.HasPropertyRole(role)
    }

    /// A clip slot's rect and radii in the flat replay space. The builder
    /// placed the flat rect into the node's property space by adding the
    /// paint_snap_offset (PropertySpaceRect); remove it again, as
    /// PaintContext's FlatClipRect does.
    fn FlatSlotClip(self, clip: &ClipPaintPropertyNode) -> Option<(PaintRect, PaintCornerRadii)> {
        let delta = self.paint_snap_offset();
        with_access(|access| {
            let mut rect = clip.PaintClipRect(access)?;
            rect.x -= delta.x;
            rect.y -= delta.y;
            Some((rect, clip.PaintClipRadii(access)))
        })
    }

    /// One flat clip slot of ObjectPaintProperties, or the node's fixture copy
    /// when the occurrence has no FragmentData.
    fn FlatClipSlotOr(
        self,
        slot: impl FnOnce(&'n ObjectPaintProperties) -> Option<&'n Arc<ClipPaintPropertyNode>>,
        node_value: impl FnOnce(&'n PaintTreeNode<'f>) -> (Option<PaintRect>, PaintCornerRadii),
    ) -> (Option<PaintRect>, PaintCornerRadii) {
        let Some(data) = self.data else {
            return self
                .fixture
                .map_or((None, PaintCornerRadii::default()), node_value);
        };
        match data.PaintProperties().and_then(slot) {
            Some(clip) => match self.FlatSlotClip(clip) {
                Some((rect, radii)) => (Some(rect), radii),
                None => (None, PaintCornerRadii::default()),
            },
            None => (None, PaintCornerRadii::default()),
        }
    }

    /// An `applies_*` flag from the ObjectPaintProperties slots, or the node's
    /// fixture copy when the occurrence has no FragmentData.
    fn SlotFlagOr(
        self,
        node_value: impl FnOnce(&'n PaintTreeNode<'f>) -> bool,
        derive: impl FnOnce(Option<&'n ObjectPaintProperties>) -> bool,
    ) -> bool {
        let Some(data) = self.data else {
            return self.fixture.is_some_and(node_value);
        };
        derive(data.PaintProperties())
    }

    // -----------------------------------------------------------------
    // Paint offset and snapping in the flat replay space.
    // -----------------------------------------------------------------

    /// FragmentData::PaintOffset() in the flat replay space: the LOCAL
    /// offset (fragment_data.h:32-38, LayoutUnit-rounded) moved back by the
    /// flat-to-property delta `paint_snap_offset`.
    pub fn FlatPaintOffset(self) -> Offset {
        self.FlatPaintOffsetOf(self.data)
    }

    /// FlatPaintOffset with this occurrence's FragmentData already resolved.
    pub fn FlatPaintOffsetOf(self, data: Option<&PrePaintFragmentData>) -> Offset {
        let Some(data) = data else {
            return self.fixture.map_or_else(Offset::default, |node| node.paint_offset);
        };
        let local = data.PaintOffset();
        let delta = self.paint_snap_offset();
        Offset {
            x: local.left.ToDouble() - delta.x,
            y: local.top.ToDouble() - delta.y,
        }
    }

    pub fn PixelSnappedRect(self, rect: &PaintRect) -> PaintRect {
        PixelSnappedRectInPropertySpace(rect, self.paint_snap_offset(), self.pixel_snap_scale())
    }

    /// DrawingRecorder bounds are enclosed in property space, where Chromium
    /// encloses the local rect, and returned in the flat replay space of the
    /// recorded ops (see FlatPaintOffset).
    pub fn EnclosingRect(self, rect: PaintRect) -> PaintRect {
        EnclosingRectInPropertySpace(rect, self.paint_snap_offset())
    }

    pub fn PixelSnappedOffset(self, offset: Offset) -> Offset {
        let delta = self.paint_snap_offset();
        let scale = self.pixel_snap_scale();
        Offset {
            x: SnapToPixel(offset.x + delta.x, scale) - delta.x,
            y: SnapToPixel(offset.y + delta.y, scale) - delta.y,
        }
    }

    /// ObjectPaintProperties::MaskClip source rect in the flat replay space,
    /// as PrePaint resolved it from the exact flat paint offset.
    pub fn ResolvedMaskClipRect(self) -> Option<PaintRect> {
        if let Some(node) = self.FixtureNode() {
            return node.ResolvedMaskClipRect();
        }
        self.FlatRare().and_then(|rare| rare.mask_clip_rect)
    }

    // -----------------------------------------------------------------
    // Tree access.
    // -----------------------------------------------------------------

    /// The painted children of this object in paint order: the fragment
    /// cursor's children (box children and FragmentItems, BoxFragmentPainter's
    /// traversal; LogicalPaintChildren), or, for a fixture, the node's
    /// children. Each walked child paints with this object's contents cull
    /// rect.
    pub fn Children(self) -> PaintObjectChildren<'n, 'f> {
        let inherited_cull_rect = if self.walk.is_some() {
            self.cull_rect()
        } else {
            None
        };
        let source = match (self.fixture, self.cursor) {
            (Some(node), _) => ChildSource::Nodes(node.children.iter()),
            (None, Some(cursor)) => ChildSource::Fragments(LogicalPaintChildren(cursor).into_iter()),
            _ => ChildSource::Fragments(Vec::new().into_iter()),
        };
        PaintObjectChildren {
            source,
            inherited_cull_rect,
        }
    }

    // -----------------------------------------------------------------
    // Values re-sourced from Layout, FragmentData and PaintLayer. Every one
    // reads the node's copy only for a fixture without FragmentData.
    // -----------------------------------------------------------------

    /// Device scale used to snap paint geometry: the LocalFrameView's
    /// DocumentResources scale PaintPropertyTreeBuilder snaps with.
    pub fn pixel_snap_scale(self) -> f64 {
        if let Some(node) = self.FixtureNode() {
            return node.pixel_snap_scale;
        }
        self.DocumentResources()
            .map_or(1.0, |resources| resources.ArtifactToDeviceScale())
            .max(f64::MIN_POSITIVE)
    }

    /// ObjectPaintProperties::InnerBorderRadiusClip in the flat replay space.
    pub fn local_inner_border_radius_clip(self) -> Option<PaintRect> {
        self.InnerBorderRadiusClip().0
    }
    pub fn local_inner_border_radius_radii(self) -> PaintCornerRadii {
        self.InnerBorderRadiusClip().1
    }
    fn InnerBorderRadiusClip(self) -> (Option<PaintRect>, PaintCornerRadii) {
        self.FlatClipSlotOr(
            |properties| properties.InnerBorderRadiusClip(),
            |node| {
                (
                    node.local_inner_border_radius_clip,
                    node.local_inner_border_radius_radii,
                )
            },
        )
    }

    /// ObjectPaintProperties::OverflowClip in the flat replay space.
    pub fn local_clip(self) -> Option<PaintRect> {
        self.OverflowClip().0
    }
    pub fn local_clip_radii(self) -> PaintCornerRadii {
        self.OverflowClip().1
    }
    fn OverflowClip(self) -> (Option<PaintRect>, PaintCornerRadii) {
        self.FlatClipSlotOr(
            |properties| properties.OverflowClip(),
            |node| (node.local_clip, node.local_clip_radii),
        )
    }

    /// ObjectPaintProperties::CssClip in the flat replay space.
    pub fn local_css_clip(self) -> Option<PaintRect> {
        self.FlatClipSlotOr(
            |properties| properties.CssClip(),
            |node| (node.local_css_clip, PaintCornerRadii::default()),
        )
        .0
    }

    /// The fill rule of ClipPathClip's path: PaintClipPath::from_authored
    /// builds the single authored path, or one path per reference child, with
    /// the rule PrePaint resolved (the first child's for a reference graph).
    pub fn clip_path_even_odd(self) -> bool {
        self.SlotFlagOr(|node| node.clip_path_even_odd, |properties| {
            properties
                .and_then(ObjectPaintProperties::ClipPathClip)
                .is_some_and(|clip| {
                    with_access(|access| {
                        clip.ClipPath(access)
                            .and_then(|path| path.paths().first())
                            .is_some_and(|path| path.is_even_odd())
                    })
                })
        })
    }

    /// (cull rect, contents cull rect) of this occurrence. A PaintLayer
    /// object reads its FragmentData's (FragmentData::GetCullRect /
    /// GetContentsCullRect, fragment_data.h:157-168, written by
    /// CullRectUpdater). Another object paints with the PaintInfo cull rect
    /// its paint parent passed down (the walk context), unless PrePaint
    /// computed rects of its own for it (FlatReplayData). None is infinite.
    fn CullRects(self) -> (Option<PaintRect>, Option<PaintRect>) {
        let Some(data) = self.data else {
            return self
                .fixture
                .map_or((None, None), |node| (node.self_cull_rect, node.cull_rect));
        };
        if self.PaintLayer().is_some() {
            (data.GetCullRect().Rect(), data.GetContentsCullRect().Rect())
        } else if let Some(rects) = data
            .FlatReplay()
            .Rare()
            .and_then(|rare| rare.non_layer_cull_rects)
        {
            rects
        } else {
            match self.walk {
                Some(walk) => (walk.inherited_cull_rect, walk.inherited_cull_rect),
                // A fixture outside the walk: the node carries the walk's
                // cull rect.
                None => {
                    let node = self.fixture.expect("fixture PaintObject");
                    (node.self_cull_rect, node.cull_rect)
                }
            }
        }
    }

    /// The contents cull rect (FragmentData::GetContentsCullRect for a
    /// PaintLayer object, else the walk's PaintInfo cull rect).
    pub fn cull_rect(self) -> Option<PaintRect> {
        self.CullRects().1
    }

    /// The cull rect (FragmentData::GetCullRect for a PaintLayer object,
    /// else the walk's PaintInfo cull rect).
    pub fn self_cull_rect(self) -> Option<PaintRect> {
        self.CullRects().0
    }

    /// ObjectPaintProperties::PaintOffsetTranslation (object_paint_properties.h).
    pub fn paint_offset_translation(self) -> Option<&'n Arc<TransformPaintPropertyNode>> {
        let Some(data) = self.data else {
            return self.fixture?.paint_offset_translation.as_ref();
        };
        data.PaintProperties()
            .and_then(ObjectPaintProperties::PaintOffsetTranslation)
    }

    /// Whether the object paints as its own PaintLayer: PrePaint's
    /// ClassifyFragmentPaintLayer over the source PaintLayer
    /// (`HasSelfPaintingLayer()`, layout_box_model_object.h) of a box
    /// fragment establishing paint state, plus a masked object (the builder
    /// isolates every Mask effect, SVG children included).
    pub fn is_paint_layer(self) -> bool {
        let (Some(data), Some(fragment)) = (self.data, self.fragment) else {
            return self.fixture.is_some_and(|node| node.is_paint_layer);
        };
        if !fragment.paint.PaintLayerStateIsAuthoritative() {
            if let Some(node) = self.fixture {
                return node.is_paint_layer;
            }
        }
        (fragment.ViewKind() == FragmentKind::kBox
            && fragment.ViewEstablishesPaintState()
            && self.PaintLayer().is_some_and(PaintLayer::IsSelfPaintingLayer))
            || data.HasPropertySlot(ObjectPaintPropertySlot::Mask)
    }

    /// PaintLayer::IsSelfPaintingLayer (paint_layer.h:210) of the source
    /// object; without a layer, Layout's exported fact for a source-backed
    /// object, else the PrePaint layer classification.
    pub fn is_self_painting_layer(self) -> bool {
        let (Some(_), Some(fragment)) = (self.data, self.fragment) else {
            return self.fixture.is_some_and(|node| node.is_self_painting_layer);
        };
        match self.PaintLayer() {
            Some(layer) => layer.IsSelfPaintingLayer(),
            None if fragment.paint.PaintLayerStateIsAuthoritative() => {
                fragment.paint.PaintLayerIsSelfPainting()
            }
            None => self.is_paint_layer(),
        }
    }

    /// `layer.GetLayoutObject().IsFragmented()` of the source PaintLayer
    /// (layout_object.h IsFragmented: more than one FragmentData).
    pub fn paint_layer_is_fragmented(self) -> bool {
        let (Some(_), Some(fragment)) = (self.data, self.fragment) else {
            return self.fixture.is_some_and(|node| node.paint_layer_is_fragmented);
        };
        match self.PaintLayer() {
            Some(layer) => layer.GetLayoutObject().IsFragmented(),
            None => fragment.paint.PaintLayerIsFragmented(),
        }
    }

    /// Whether the painted object is the LayoutView (`IsA<LayoutView>`,
    /// box_fragment_painter.cc PaintBoxDecorationBackground): the root of
    /// every PrePaint walk, a local frame's included.
    pub fn is_root(self) -> bool {
        let (Some(_), Some(fragment)) = (self.data, self.fragment) else {
            return self.fixture.is_some_and(|node| node.is_root);
        };
        fragment.paint.IsLayoutView()
    }

    /// PaintLayer::NeedsReorderOverlayOverflowControls (paint_layer.h:
    /// 427-431), set by the owning PaintLayerStackingNode's rebuild and read
    /// by PaintLayerPainter for the same decision.
    pub fn needs_reorder_overlay_overflow_controls(self) -> bool {
        match (self.data, self.PaintLayer()) {
            (Some(_), Some(layer)) => layer.NeedsReorderOverlayOverflowControls(),
            _ => self
                .fixture
                .is_some_and(|node| node.needs_reorder_overlay_overflow_controls.get()),
        }
    }

    /// ObjectPaintProperties::OverflowClip.
    pub fn applies_overflow_clip(self) -> bool {
        self.SlotFlagOr(|node| node.applies_overflow_clip, |properties| {
            properties.is_some_and(|properties| properties.OverflowClip().is_some())
        })
    }
    /// ObjectPaintProperties::CssClip.
    pub fn applies_css_clip(self) -> bool {
        self.SlotFlagOr(|node| node.applies_css_clip, |properties| {
            properties.is_some_and(|properties| properties.CssClip().is_some())
        })
    }
    /// ObjectPaintProperties::ClipPathClip.
    pub fn applies_clip_path(self) -> bool {
        self.SlotFlagOr(|node| node.applies_clip_path, |properties| {
            properties.is_some_and(|properties| properties.ClipPathClip().is_some())
        })
    }
    /// ObjectPaintProperties::Effect or Filter (the builder creates Effect for
    /// opacity, blending, backdrop filter, mask or an effect reason).
    pub fn applies_effect(self) -> bool {
        self.SlotFlagOr(|node| node.applies_effect, |properties| {
            properties.is_some_and(|properties| {
                properties.Effect().is_some() || properties.Filter().is_some()
            })
        })
    }
    /// The Effect slot with a non-opaque style opacity (the value the
    /// SaveLayerAlpha replays, as PaintContext's ObjectReplayScopes reads it).
    pub fn applies_opacity(self) -> bool {
        self.SlotFlagOr(|node| node.applies_opacity, |properties| {
            properties.is_some_and(|properties| properties.Effect().is_some())
                && self
                    .fragment
                    .is_some_and(|fragment| MaskAdjustedOpacity(&fragment.paint.style) != 1.0)
        })
    }
    /// The Effect slot with a blend mode or isolating descendant blending
    /// (NeedsCssBlendEffect).
    pub fn applies_blend(self) -> bool {
        self.SlotFlagOr(|node| node.applies_blend, |properties| {
            properties
                .and_then(ObjectPaintProperties::Effect)
                .is_some_and(|effect| {
                    with_access(|access| {
                        let state = effect.state(access);
                        state.blend_mode != PaintBlendMode::kNormal || state.isolates_blending
                    })
                })
        })
    }
    /// ObjectPaintProperties::Filter.
    pub fn applies_filter(self) -> bool {
        self.SlotFlagOr(|node| node.applies_filter, |properties| {
            properties.is_some_and(|properties| properties.Filter().is_some())
        })
    }
    /// The BackdropFilterClip slot, created exactly for a backdrop filter.
    pub fn applies_backdrop_filter(self) -> bool {
        self.SlotFlagOr(
            |node| node.applies_backdrop_filter,
            |properties| properties.is_some_and(|properties| properties.BackdropFilterClip().is_some()),
        )
    }
    /// ObjectPaintProperties::Mask.
    pub fn applies_mask(self) -> bool {
        self.SlotFlagOr(|node| node.applies_mask, |properties| {
            properties.is_some_and(|properties| properties.Mask().is_some())
        })
    }

    /// FragmentData::LocalBorderBoxProperties (fragment_data.h:137-142).
    pub fn properties(self) -> &'n PaintPropertyTreeState {
        match self.data.and_then(|data| data.LocalBorderBoxPropertiesRef()) {
            Some(state) => state,
            None => {
                &self
                    .fixture
                    .expect("PaintObject without LocalBorderBoxProperties")
                    .properties
                    .nodes
            }
        }
    }

    /// FragmentData::PreTransform (fragment_data.cc:55-65).
    pub fn pre_transform(self) -> Arc<TransformPaintPropertyNode> {
        match self.data {
            Some(data) if data.HasLocalBorderBoxProperties() => {
                with_access(|access| data.PreTransform(access))
            }
            _ => match self.fixture {
                Some(node) => node.pre_transform.clone(),
                None => PaintPropertyTreeState::default().transform,
            },
        }
    }

    /// FragmentData::ContentsProperties (fragment_data.h:170-177).
    pub fn contents_properties(self) -> PaintPropertyTreeState {
        match self.data {
            Some(data) if data.HasLocalBorderBoxProperties() => data.ContentsProperties(),
            _ => self
                .fixture
                .map_or_else(PaintPropertyTreeState::default, |node| {
                    node.contents_properties.nodes.clone()
                }),
        }
    }

    /// The composited fixed background state: the BackgroundClip slot over
    /// the LocalBorderBoxProperties effect, in the frame's viewport transform
    /// (BoxFragmentPainter::PaintCompositeBackgroundAttachmentFixed). That
    /// transform is the LayoutView's, which the builder carried down as walk
    /// context; FlatReplayData keeps it for the occurrence.
    pub fn fixed_background_properties(self) -> Option<PaintPropertyTreeState> {
        let Some(data) = self.data else {
            return self.fixture?.fixed_background_properties.clone();
        };
        match (
            data.PaintProperties().and_then(ObjectPaintProperties::BackgroundClip),
            self.FlatRare()
                .and_then(|rare| rare.fixed_background_transform.as_ref()),
        ) {
            (Some(clip), Some(transform)) => Some(PaintPropertyTreeState {
                transform: transform.clone(),
                clip: clip.clone(),
                effect: data.LocalBorderBoxProperties().effect,
            }),
            _ => None,
        }
    }

    /// ScrollableAreaPainter::PaintOverflowControls: the
    /// ObjectPaintProperties::OverflowControlsClip entered over the box's
    /// LocalBorderBoxProperties. `None` when the box has no such clip.
    pub fn overflow_controls_properties(self) -> Option<PaintPropertyTreeState> {
        let Some(data) = self.data else {
            return self.fixture?.overflow_controls_properties.clone();
        };
        data.PaintProperties()
            .and_then(ObjectPaintProperties::OverflowControlsClip)
            .map(|clip| {
                let mut state = data.LocalBorderBoxProperties();
                state.clip = clip.clone();
                state
            })
    }

    /// ScrollableAreaPainter::PaintScrollbar: the Horizontal/
    /// VerticalScrollbarEffect, entered inside the overflow-controls clip
    /// scope. `None` when the axis has no effect node.
    pub fn horizontal_scrollbar_properties(self) -> Option<PaintPropertyTreeState> {
        self.ScrollbarProperties(true)
    }
    pub fn vertical_scrollbar_properties(self) -> Option<PaintPropertyTreeState> {
        self.ScrollbarProperties(false)
    }
    fn ScrollbarProperties(self, horizontal: bool) -> Option<PaintPropertyTreeState> {
        let node_value = |node: &'n PaintTreeNode<'f>| {
            if horizontal {
                node.horizontal_scrollbar_properties.as_ref()
            } else {
                node.vertical_scrollbar_properties.as_ref()
            }
        };
        let Some(data) = self.data else {
            return self.fixture.and_then(node_value).cloned();
        };
        data.PaintProperties().and_then(|properties| {
            let effect = if horizontal {
                properties.HorizontalScrollbarEffect()
            } else {
                properties.VerticalScrollbarEffect()
            }?
            .clone();
            let mut state = data.LocalBorderBoxProperties();
            if let Some(clip) = properties.OverflowControlsClip() {
                state.clip = clip.clone();
            }
            state.effect = effect;
            Some(state)
        })
    }

    /// PaintLayerPainter::PaintFragmentWithPhase(kMask)
    /// (paint_layer_painter.cc:722-729): the LocalBorderBoxProperties with the
    /// Mask effect and its output clip. `None` without a Mask slot.
    pub fn mask_properties(self) -> Option<PaintPropertyTreeState> {
        let Some(data) = self.data else {
            return self.fixture?.mask_properties.clone();
        };
        data.PaintProperties()
            .and_then(ObjectPaintProperties::Mask)
            .map(|mask| {
                let output_clip = with_access(|access| mask.state(access).output_clip.clone())
                    .expect("Mask effect has an output clip");
                PaintPropertyTreeState {
                    transform: self.properties().transform.clone(),
                    clip: output_clip,
                    effect: mask.clone(),
                }
            })
    }

    // -----------------------------------------------------------------
    // Walk context (PaintInfo / PaintLayerPainter traversal state).
    // -----------------------------------------------------------------

    /// PaintLayerPainter's in-flight PaintLayer::PreviousPaintResult: seeded
    /// per traversal and downgraded by ScopedBoxContentsPaintState before
    /// PaintLayerPainter writes the layer (and not written at all under
    /// kOmitCompositingInfo). The walk lends the cell of the layer being
    /// painted; an object reached through the cursor alone has none.
    pub fn previous_paint_result(self) -> Option<&'n Cell<PaintResult>> {
        match self.walk {
            Some(walk) => walk.previous_paint_result,
            None => self.fixture.map(|node| &node.previous_paint_result),
        }
    }

    // -----------------------------------------------------------------
    // Flat replay values (FragmentData FlatReplayData): Rechrom's flat
    // replay stream only; Chromium paints in the local property space.
    // -----------------------------------------------------------------

    /// Flat-to-property-space delta of the paint offset.
    pub fn paint_snap_offset(self) -> Offset {
        let Some(flat) = self.Flat() else {
            return self.fixture.map_or_else(Offset::default, |node| node.paint_snap_offset);
        };
        flat.paint_snap_offset
    }
    /// The walk root's inherited flat clip (the frame-view viewport clip of
    /// the main frame, the owner's accumulated clip of a local frame). The
    /// frame-level ViewportClip is not an ObjectPaintProperties slot
    /// (ObjectPaintPropertySlot::FromRole).
    pub fn root_clip(self) -> Option<PaintRect> {
        if self.data.is_none() {
            return self.fixture.and_then(|node| node.root_clip);
        }
        self.FlatRare().and_then(|rare| rare.root_clip)
    }
    pub fn applies_root_clip(self) -> bool {
        let Some(flat) = self.Flat() else {
            return self.fixture.is_some_and(|node| node.applies_root_clip);
        };
        flat.applies_root_clip
    }
    /// The builder's authored clip-path commands in the flat replay space; the
    /// ClipPathClip node holds them translated into property space.
    pub fn local_clip_path(self) -> &'n [PaintPathCommand] {
        if self.data.is_none() {
            return self.fixture.map_or(&[][..], |node| node.local_clip_path.as_slice());
        }
        self.FlatRare()
            .and_then(|rare| rare.clip_path_commands.as_deref())
            .unwrap_or(&[])
    }
    /// The reference children of a clip-path graph. Only the fixture replay
    /// scopes read them (PaintContext ObjectReplayScopes::FromPrePaintCopies);
    /// an occurrence with FragmentData reads the ClipPathClip slot.
    pub fn local_clip_path_children(self) -> &'n [PaintClipPathChild] {
        self.FixtureNode()
            .map_or(&[][..], |node| node.local_clip_path_children.as_slice())
    }
    /// The accumulated flat transform list (root first).
    pub fn transforms(self) -> &'n [TransformMatrix] {
        let Some(flat) = self.Flat() else {
            return self.fixture.map_or(&[][..], |node| &node.transforms[..]);
        };
        &flat.transforms[..]
    }
    /// The object's own flat matrices, replayed as kConcat items.
    pub fn local_transforms(self) -> &'n [TransformMatrix] {
        if self.data.is_none() {
            return self.fixture.map_or(&[][..], |node| node.local_transforms.as_slice());
        }
        self.FlatRare()
            .and_then(|rare| rare.local_transforms.as_deref())
            .unwrap_or(&[])
    }
    /// Whether the flat replay opens this object's own transform scope. An
    /// SVG child may lead with its container's matrix, which has no slot of
    /// its own, so this stays with the flat matrices.
    pub fn applies_transform(self) -> bool {
        let Some(flat) = self.Flat() else {
            return self.fixture.is_some_and(|node| node.applies_transform);
        };
        flat.applies_transform
    }
    /// The flat scroll anchor the fixed background positioning area adds.
    pub fn scroll_offset(self) -> Offset {
        let Some(flat) = self.Flat() else {
            return self.fixture.map_or_else(Offset::default, |node| node.scroll_offset);
        };
        flat.scroll_offset
    }
    /// CSS transforms on non-root ancestors make background-attachment:fixed
    /// behave as scroll.
    pub fn fixed_background_acts_as_scroll(self) -> bool {
        let Some(flat) = self.Flat() else {
            return self
                .fixture
                .is_some_and(|node| node.fixed_background_acts_as_scroll);
        };
        flat.fixed_background_acts_as_scroll
    }
    /// The united outline of an inline continuation group, joined across its
    /// occurrences by PrepareOutlines (Chromium's LayoutInline OutlineRects
    /// over every FragmentItem of the inline, which the Layout facade does
    /// not expose yet).
    pub fn paints_outline(self) -> bool {
        let Some(flat) = self.Flat() else {
            return self.fixture.is_none_or(|node| node.paints_outline);
        };
        flat.paints_outline
    }
    /// Whether this object's non-layer subtree paints the float phase (the
    /// walk's phase pruning, FlatReplayData).
    pub fn needs_float_phase(self) -> bool {
        let Some(flat) = self.Flat() else {
            return self.fixture.is_none_or(|node| node.needs_float_phase);
        };
        !flat.skip_float_phase
    }
    /// Whether this object's non-layer subtree paints an outline phase.
    pub fn needs_outline_phase(self) -> bool {
        let Some(flat) = self.Flat() else {
            return self.fixture.is_none_or(|node| node.needs_outline_phase);
        };
        !flat.skip_outline_phase
    }
    /// Whether this object's non-layer subtree paints overlay overflow
    /// controls.
    pub fn needs_overflow_controls_phase(self) -> bool {
        let Some(flat) = self.Flat() else {
            return self
                .fixture
                .is_none_or(|node| node.needs_overflow_controls_phase);
        };
        !flat.skip_overflow_controls_phase
    }
    pub fn outline_rects(self) -> &'n [PaintRect] {
        if self.data.is_none() {
            return self.fixture.map_or(&[][..], |node| node.outline_rects.as_slice());
        }
        self.FlatRare()
            .and_then(|rare| rare.outline_rects.as_deref())
            .unwrap_or(&[])
    }
    /// PrePaint's resolved mask layers. PrePaint still needs them for the
    /// MaskClip rect of an SVG child (ResolvedMaskClipRect), and resolving
    /// them again here would repeat the SVG mask shader resolution.
    pub fn mask_layers(self) -> &'n [DisplayMaskLayer] {
        if self.data.is_none() {
            return self.fixture.map_or(&[][..], |node| node.mask_layers.as_slice());
        }
        self.FlatRare()
            .and_then(|rare| rare.mask_layers.as_deref())
            .unwrap_or(&[])
    }
}

#[derive(Clone)]
enum ChildSource<'n, 'f> {
    /// A fixture's node children.
    Nodes(std::slice::Iter<'n, Box<PaintTreeNode<'f>>>),
    /// The fragment cursor's children, each resolved once.
    Fragments(std::vec::IntoIter<ResolvedFragment<'n>>),
}

/// Iterator over a painted object's children.
#[derive(Clone)]
pub struct PaintObjectChildren<'n, 'f> {
    source: ChildSource<'n, 'f>,
    inherited_cull_rect: Option<PaintRect>,
}

impl<'n, 'f> PaintObjectChildren<'n, 'f> {
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn Child(&self, node: Option<&'n PaintTreeNode<'f>>, child: Option<ResolvedFragment<'n>>) -> PaintObject<'n, 'f> {
        match (node, child) {
            (Some(node), _) => PaintObject::Fixture(node),
            (None, Some(child)) => {
                child.Object(PaintWalkContext::new(self.inherited_cull_rect, None))
            }
            (None, None) => unreachable!("a paint child has a node or a cursor"),
        }
    }
}

impl<'n, 'f> Iterator for PaintObjectChildren<'n, 'f> {
    type Item = PaintObject<'n, 'f>;

    fn next(&mut self) -> Option<Self::Item> {
        let (node, child) = match &mut self.source {
            ChildSource::Nodes(nodes) => (Some(&**nodes.next()?), None),
            ChildSource::Fragments(fragments) => (None, Some(fragments.next()?)),
        };
        Some(self.Child(node, child))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.source {
            ChildSource::Nodes(nodes) => nodes.size_hint(),
            ChildSource::Fragments(fragments) => fragments.size_hint(),
        }
    }
}

impl DoubleEndedIterator for PaintObjectChildren<'_, '_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let (node, child) = match &mut self.source {
            ChildSource::Nodes(nodes) => (Some(&**nodes.next_back()?), None),
            ChildSource::Fragments(fragments) => (None, Some(fragments.next_back()?)),
        };
        Some(self.Child(node, child))
    }
}

impl ExactSizeIterator for PaintObjectChildren<'_, '_> {}
