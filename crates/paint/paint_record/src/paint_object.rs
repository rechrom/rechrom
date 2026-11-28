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
//! FragmentData's `FlatReplayData`. A PaintObject is built three ways:
//!
//! - `FromCursor(cursor, walk)`: the paint walk's occurrence; no PrePaint
//!   node at all.
//! - `Walked(node, walk)`: the same occurrence under
//!   RECHROM_PREPAINT_FRAGMENT_DATA_CHECK. The values come from the cursor,
//!   FragmentData and walk context exactly as for `FromCursor`; the resident
//!   PrePaint node is attached only as the equivalence oracle (`Verify*`).
//! - `new(node)`: hand-built fixtures (no FragmentData: every value is the
//!   node's copy) and the synthetic nodes of scroll_paint_update / special
//!   paint, whose walk-context values (inherited cull rect, previous paint
//!   result) are the node's.

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

/// RECHROM_PREPAINT_FRAGMENT_DATA_CHECK (the PrePaint FragmentData proof
/// switch), read once.
fn SourceCheckEnabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("RECHROM_PREPAINT_FRAGMENT_DATA_CHECK").is_some())
}

/// RECHROM_PREPAINT_FRAGMENT_DATA_CHECK: the paint walk also lends the
/// resident PrePaint nodes to its objects as their equivalence oracle and
/// checks its cursor traversal against the node traversal.
#[doc(hidden)]
pub fn FragmentDataCheckEnabled() -> bool {
    SourceCheckEnabled()
}

/// The first resident document-order position in a fragment subtree: the
/// fragment's own source LayoutObject's (LayoutObject::LogicalTreeOrder), else
/// the smallest of its children's, else 0 (no source with a Node).
fn FirstLogicalTreeOrder(cursor: PaintFragmentCursor<'_>) -> u64 {
    let order = cursor.PaintData().paint.LogicalTreeOrder();
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

/// The painted children of a fragment in paint order: the fragment cursor's
/// children (box children, then FragmentItems; BoxFragmentPainter's
/// PaintBlockChildren / PaintInlineItems traversal), with the children that
/// have a document-order position stably sorted by it in their own slots.
///
/// Not Chromium: Chromium paints fragment children in fragment order. The
/// sort keeps the paint order PrePaint gave its node tree
/// (RestoreLogicalSiblingOrder) until that order is retired; children
/// already in order (nearly always) are returned without sorting.
pub fn LogicalPaintChildren<'a>(cursor: PaintFragmentCursor<'a>) -> Vec<PaintFragmentCursor<'a>> {
    let mut children: Vec<PaintFragmentCursor<'a>> = cursor.Children().collect();
    if children.len() < 2 {
        return children;
    }
    let keys: Vec<u64> = children.iter().map(|child| FirstLogicalTreeOrder(*child)).collect();
    let mut previous = 0;
    let already_ordered = keys.iter().all(|&order| {
        if order == 0 {
            return true;
        }
        if order < previous {
            return false;
        }
        previous = order;
        true
    });
    if already_ordered {
        return children;
    }
    let slots: Vec<usize> = (0..children.len()).filter(|&index| keys[index] != 0).collect();
    let mut ordered: Vec<(u64, PaintFragmentCursor<'a>)> =
        slots.iter().map(|&index| (keys[index], children[index])).collect();
    // Stable, as the node tree's sort_by_key.
    ordered.sort_by_key(|(order, _)| *order);
    for (slot, (_, child)) in slots.into_iter().zip(ordered) {
        children[slot] = child;
    }
    children
}

/// The equivalence proof of a re-sourced accessor whose production value
/// changes source in this view: checked in debug builds and, in any build,
/// under RECHROM_PREPAINT_FRAGMENT_DATA_CHECK.
#[inline]
fn VerifySource(accessor: &str, same: impl FnOnce() -> bool) {
    if cfg!(debug_assertions) || SourceCheckEnabled() {
        assert!(
            same(),
            "PaintObject::{accessor}: the re-sourced value differs from its PaintTreeNode copy"
        );
    }
}

/// The equivalence check of a re-sourced accessor whose production callers
/// already read the FragmentData value (so a difference only shows a stale
/// node copy), or whose node copy is written outside the paint lifecycle:
/// checked only under RECHROM_PREPAINT_FRAGMENT_DATA_CHECK.
#[inline]
fn VerifySourceUnderCheck(accessor: &str, same: impl FnOnce() -> bool) {
    if SourceCheckEnabled() {
        assert!(
            same(),
            "PaintObject::{accessor}: the re-sourced value differs from its PaintTreeNode copy"
        );
    }
}

/// Flat rects are moved into property space by `paint_snap_offset` and back,
/// so the round trip may differ in the last bits.
fn SameRect(a: &PaintRect, b: &PaintRect) -> bool {
    let close = |a: f64, b: f64| (a - b).abs() <= 1e-6 * (1.0 + a.abs().max(b.abs()));
    close(a.x, b.x) && close(a.y, b.y) && close(a.width, b.width) && close(a.height, b.height)
}

fn SameOptionalRect(a: Option<PaintRect>, b: Option<PaintRect>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => SameRect(&a, &b),
        (None, None) => true,
        _ => false,
    }
}

/// Node identity is the pointer (paint property nodes are live objects).
fn SameStateNodes(a: &PaintPropertyTreeState, b: &PaintPropertyTreeState) -> bool {
    Arc::ptr_eq(&a.transform, &b.transform)
        && Arc::ptr_eq(&a.clip, &b.clip)
        && Arc::ptr_eq(&a.effect, &b.effect)
}

fn SameOptionalState(a: Option<&PaintPropertyTreeState>, b: Option<&PaintPropertyTreeState>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => SameStateNodes(a, b),
        (None, None) => true,
        _ => false,
    }
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

    /// The context of a PaintLayer painted by its PaintLayerPainter: its
    /// cull rects are its FragmentData's (CullRectUpdater), so nothing is
    /// inherited.
    pub fn ForLayer(previous_paint_result: Option<&'n Cell<PaintResult>>) -> Self {
        Self::new(None, previous_paint_result)
    }

    /// The context of a paint child of `parent`: the child paints with the
    /// parent's contents cull rect (BoxFragmentPainter passes the
    /// ScopedBoxContentsPaintState PaintInfo to its children).
    pub fn ForChild(
        parent: PaintObject<'_, '_>,
        previous_paint_result: Option<&'n Cell<PaintResult>>,
    ) -> Self {
        Self::new(parent.cull_rect(), previous_paint_result)
    }
}

/// One painted object occurrence: source fragment + FragmentData + walk state.
#[derive(Clone, Copy)]
pub struct PaintObject<'n, 'f> {
    cursor: Option<PaintFragmentCursor<'n>>,
    fragment: Option<&'n FragmentPaintData>,
    data: Option<&'n PrePaintFragmentData>,
    /// None for a node-backed object, whose walk-context values are the
    /// node's.
    walk: Option<PaintWalkContext<'n>>,
    node: Option<&'n PaintTreeNode<'f>>,
}

impl<'n, 'f> PaintObject<'n, 'f> {
    /// A node-backed occurrence: a hand-built fixture, or a synthetic node of
    /// scroll_paint_update / special paint outside the paint walk.
    pub fn new(node: &'n PaintTreeNode<'f>) -> Self {
        Self {
            cursor: node.SourceCursor(),
            fragment: node.fragment.as_deref(),
            data: node.FragmentData(),
            walk: None,
            node: Some(node),
        }
    }

    /// The paint walk's occurrence: the values of `FromCursor(node's cursor,
    /// walk)`, with the resident PrePaint node attached as their oracle.
    pub fn Walked(node: &'n PaintTreeNode<'f>, walk: PaintWalkContext<'n>) -> Self {
        Self {
            walk: Some(walk),
            ..Self::new(node)
        }
    }

    /// An occurrence reached through the fragment cursor alone.
    pub fn FromCursor(cursor: PaintFragmentCursor<'n>, walk: PaintWalkContext<'n>) -> Self {
        Self {
            cursor: Some(cursor),
            fragment: Some(cursor.PaintData()),
            data: cursor.PrePaintData::<PrePaintFragmentData>(),
            walk: Some(walk),
            node: None,
        }
    }

    /// The PrePaint node attached as this object's equivalence oracle
    /// (RECHROM_PREPAINT_FRAGMENT_DATA_CHECK), or a fixture's node. The paint
    /// walk reads it only to check its cursor traversal; painters never use
    /// it.
    #[doc(hidden)]
    pub fn CheckNode(self) -> Option<&'n PaintTreeNode<'f>> {
        self.node
    }

    /// The walk context this object paints with.
    pub fn Walk(self) -> PaintWalkContext<'n> {
        self.walk.unwrap_or_default()
    }

    // -----------------------------------------------------------------
    // Sources and the oracle.
    // -----------------------------------------------------------------

    /// The node as the source of every value: an occurrence without
    /// FragmentData (hand-built fixtures).
    fn FixtureNode(self) -> Option<&'n PaintTreeNode<'f>> {
        if self.data.is_some() {
            None
        } else {
            self.node
        }
    }

    /// The node as the oracle of a value re-sourced from the occurrence.
    fn Oracle(self) -> Option<&'n PaintTreeNode<'f>> {
        if self.data.is_some() {
            self.node
        } else {
            None
        }
    }

    fn Verify(self, accessor: &str, same: impl FnOnce(&'n PaintTreeNode<'f>) -> bool) {
        if let Some(node) = self.Oracle() {
            VerifySource(accessor, || same(node));
        }
    }

    fn VerifyUnderCheck(self, accessor: &str, same: impl FnOnce(&'n PaintTreeNode<'f>) -> bool) {
        if let Some(node) = self.Oracle() {
            VerifySourceUnderCheck(accessor, || same(node));
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
        let value = self
            .fragment
            .map_or(0, |fragment| fragment.ViewDisplayItemFragment());
        self.VerifyUnderCheck("DisplayItemFragment", |node| value == node.DisplayItemFragment());
        value
    }

    /// Whether ObjectPaintProperties holds the slot of `role`
    /// (object_paint_properties.h `HasNode`).
    pub fn HasOwnPropertyRole(self, role: PaintPropertyRole) -> bool {
        let Some(data) = self.data else {
            return self.node.is_some_and(|node| node.HasOwnPropertyRole(role));
        };
        let value = data.HasPropertyRole(role);
        // The node's key log exists only for nodes built with a cache.
        self.VerifyUnderCheck("HasOwnPropertyRole", |node| {
            value == node.HasOwnPropertyRole(role)
        });
        value
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
        accessor: &str,
        slot: impl FnOnce(&'n ObjectPaintProperties) -> Option<&'n Arc<ClipPaintPropertyNode>>,
        node_value: impl Fn(&'n PaintTreeNode<'f>) -> (Option<PaintRect>, PaintCornerRadii),
    ) -> (Option<PaintRect>, PaintCornerRadii) {
        let Some(data) = self.data else {
            return self
                .node
                .map_or((None, PaintCornerRadii::default()), &node_value);
        };
        let value = match data.PaintProperties().and_then(slot) {
            Some(clip) => match self.FlatSlotClip(clip) {
                Some((rect, radii)) => (Some(rect), radii),
                None => (None, PaintCornerRadii::default()),
            },
            None => (None, PaintCornerRadii::default()),
        };
        self.Verify(accessor, |node| {
            let node_value = node_value(node);
            SameOptionalRect(value.0, node_value.0) && value.1 == node_value.1
        });
        value
    }

    /// An `applies_*` flag from the ObjectPaintProperties slots, or the node's
    /// fixture copy when the occurrence has no FragmentData.
    fn SlotFlagOr(
        self,
        accessor: &str,
        node_value: impl Fn(&'n PaintTreeNode<'f>) -> bool,
        derive: impl FnOnce(Option<&'n ObjectPaintProperties>) -> bool,
    ) -> bool {
        let Some(data) = self.data else {
            return self.node.is_some_and(&node_value);
        };
        let value = derive(data.PaintProperties());
        self.Verify(accessor, |node| value == node_value(node));
        value
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
            return self.node.map_or_else(Offset::default, |node| node.paint_offset);
        };
        let local = data.PaintOffset();
        let delta = self.paint_snap_offset();
        let flat = Offset {
            x: local.left.ToDouble() - delta.x,
            y: local.top.ToDouble() - delta.y,
        };
        // PaintPropertyTreeBuilder stores round(paint_offset + delta) in
        // LayoutUnit (FragmentDataPaintOffset); only that rounding may
        // separate the two copies.
        self.Verify("FlatPaintOffset", |node| {
            (flat.x - node.paint_offset.x).abs() <= 1.0 / 128.0 + 1e-9
                && (flat.y - node.paint_offset.y).abs() <= 1.0 / 128.0 + 1e-9
        });
        flat
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
        let value = self.FlatRare().and_then(|rare| rare.mask_clip_rect);
        self.Verify("ResolvedMaskClipRect", |node| {
            SameOptionalRect(value, node.ResolvedMaskClipRect())
        });
        value
    }

    // -----------------------------------------------------------------
    // Tree access.
    // -----------------------------------------------------------------

    /// The painted children of this object in paint order: the fragment
    /// cursor's children (box children and FragmentItems, BoxFragmentPainter's
    /// traversal; LogicalPaintChildren), or, for a node-backed object (a
    /// fixture, or the check oracle), the PrePaint node's children. Each child
    /// paints with this object's contents cull rect.
    pub fn Children(self) -> PaintObjectChildren<'n, 'f> {
        let inherited_cull_rect = if self.walk.is_some() {
            self.cull_rect()
        } else {
            None
        };
        let source = match (self.node, self.cursor) {
            (Some(node), _) => ChildSource::Nodes(node.children.iter()),
            (None, Some(cursor)) => ChildSource::Cursors(LogicalPaintChildren(cursor).into_iter()),
            _ => ChildSource::Cursors(Vec::new().into_iter()),
        };
        PaintObjectChildren {
            source,
            walked: self.walk.is_some(),
            inherited_cull_rect,
        }
    }

    /// The PrePaint node of the SVG resource subtree root registered for
    /// `client` (the root node's directory): the check oracle of a resource
    /// subtree the paint walk reaches through Layout.
    pub fn SvgResourceNode(self, client: u64) -> Option<PaintObject<'n, 'static>> {
        self.node?.SvgResourceNode(client).map(PaintObject::new)
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
        let value = self
            .DocumentResources()
            .map_or(1.0, |resources| resources.ArtifactToDeviceScale())
            .max(f64::MIN_POSITIVE);
        self.Verify("pixel_snap_scale", |node| value == node.pixel_snap_scale);
        value
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
            "local_inner_border_radius_clip",
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
            "local_clip",
            |properties| properties.OverflowClip(),
            |node| (node.local_clip, node.local_clip_radii),
        )
    }

    /// ObjectPaintProperties::CssClip in the flat replay space.
    pub fn local_css_clip(self) -> Option<PaintRect> {
        self.FlatClipSlotOr(
            "local_css_clip",
            |properties| properties.CssClip(),
            |node| (node.local_css_clip, PaintCornerRadii::default()),
        )
        .0
    }

    /// The fill rule of ClipPathClip's path: PaintClipPath::from_authored
    /// builds the single authored path, or one path per reference child, with
    /// the rule PrePaint resolved (the first child's for a reference graph).
    pub fn clip_path_even_odd(self) -> bool {
        self.SlotFlagOr("clip_path_even_odd", |node| node.clip_path_even_odd, |properties| {
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
                .node
                .map_or((None, None), |node| (node.self_cull_rect, node.cull_rect));
        };
        let value = if self.PaintLayer().is_some() {
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
                // A node-backed object outside the walk: the node carries the
                // walk's cull rect.
                None => {
                    let node = self.node.expect("node-backed PaintObject");
                    return (node.self_cull_rect, node.cull_rect);
                }
            }
        };
        self.VerifyUnderCheck("cull_rect", |node| value == (node.self_cull_rect, node.cull_rect));
        value
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
            return self.node?.paint_offset_translation.as_ref();
        };
        let value = data
            .PaintProperties()
            .and_then(ObjectPaintProperties::PaintOffsetTranslation);
        self.Verify("paint_offset_translation", |node| {
            match (value, node.paint_offset_translation.as_ref()) {
                (Some(value), Some(node_value)) => Arc::ptr_eq(value, node_value),
                (None, None) => true,
                _ => false,
            }
        });
        value
    }

    /// Whether the object paints as its own PaintLayer: PrePaint's
    /// ClassifyFragmentPaintLayer over the source PaintLayer
    /// (`HasSelfPaintingLayer()`, layout_box_model_object.h) of a box
    /// fragment establishing paint state, plus a masked object (the builder
    /// isolates every Mask effect, SVG children included).
    pub fn is_paint_layer(self) -> bool {
        let (Some(data), Some(fragment)) = (self.data, self.fragment) else {
            return self.node.is_some_and(|node| node.is_paint_layer);
        };
        if !fragment.paint.PaintLayerStateIsAuthoritative() {
            if let Some(node) = self.node {
                return node.is_paint_layer;
            }
        }
        let value = (fragment.ViewKind() == FragmentKind::kBox
            && fragment.ViewEstablishesPaintState()
            && self.PaintLayer().is_some_and(PaintLayer::IsSelfPaintingLayer))
            || data.HasPropertySlot(ObjectPaintPropertySlot::Mask);
        self.Verify("is_paint_layer", |node| value == node.is_paint_layer);
        value
    }

    /// PaintLayer::IsSelfPaintingLayer (paint_layer.h:210) of the source
    /// object; without a layer, Layout's exported fact for a source-backed
    /// object, else the PrePaint layer classification.
    pub fn is_self_painting_layer(self) -> bool {
        let (Some(_), Some(fragment)) = (self.data, self.fragment) else {
            return self.node.is_some_and(|node| node.is_self_painting_layer);
        };
        let value = match self.PaintLayer() {
            Some(layer) => layer.IsSelfPaintingLayer(),
            None if fragment.paint.PaintLayerStateIsAuthoritative() => {
                fragment.paint.PaintLayerIsSelfPainting()
            }
            None => self.is_paint_layer(),
        };
        self.Verify("is_self_painting_layer", |node| value == node.is_self_painting_layer);
        value
    }

    /// `layer.GetLayoutObject().IsFragmented()` of the source PaintLayer
    /// (layout_object.h IsFragmented: more than one FragmentData).
    pub fn paint_layer_is_fragmented(self) -> bool {
        let (Some(_), Some(fragment)) = (self.data, self.fragment) else {
            return self.node.is_some_and(|node| node.paint_layer_is_fragmented);
        };
        let value = match self.PaintLayer() {
            Some(layer) => layer.GetLayoutObject().IsFragmented(),
            None => fragment.paint.PaintLayerIsFragmented(),
        };
        self.Verify("paint_layer_is_fragmented", |node| {
            value == node.paint_layer_is_fragmented
        });
        value
    }

    /// Whether the painted object is the LayoutView (`IsA<LayoutView>`,
    /// box_fragment_painter.cc PaintBoxDecorationBackground): the root of
    /// every PrePaint walk, a local frame's included.
    pub fn is_root(self) -> bool {
        let (Some(_), Some(fragment)) = (self.data, self.fragment) else {
            return self.node.is_some_and(|node| node.is_root);
        };
        let value = fragment.paint.IsLayoutView();
        self.Verify("is_root", |node| value == node.is_root);
        value
    }

    /// PaintLayer::NeedsReorderOverlayOverflowControls (paint_layer.h:
    /// 427-431), set by the owning PaintLayerStackingNode's rebuild and read
    /// by PaintLayerPainter for the same decision. The node cell is written
    /// only by the resident hit-test collector, so it is compared under the
    /// check switch only.
    pub fn needs_reorder_overlay_overflow_controls(self) -> bool {
        let layer = match (self.data, self.PaintLayer()) {
            (Some(_), Some(layer)) => layer,
            _ => {
                return self
                    .node
                    .is_some_and(|node| node.needs_reorder_overlay_overflow_controls.get())
            }
        };
        let value = layer.NeedsReorderOverlayOverflowControls();
        self.VerifyUnderCheck("needs_reorder_overlay_overflow_controls", |node| {
            value == node.needs_reorder_overlay_overflow_controls.get()
        });
        value
    }

    /// ObjectPaintProperties::OverflowClip.
    pub fn applies_overflow_clip(self) -> bool {
        self.SlotFlagOr("applies_overflow_clip", |node| node.applies_overflow_clip, |properties| {
            properties.is_some_and(|properties| properties.OverflowClip().is_some())
        })
    }
    /// ObjectPaintProperties::CssClip.
    pub fn applies_css_clip(self) -> bool {
        self.SlotFlagOr("applies_css_clip", |node| node.applies_css_clip, |properties| {
            properties.is_some_and(|properties| properties.CssClip().is_some())
        })
    }
    /// ObjectPaintProperties::ClipPathClip.
    pub fn applies_clip_path(self) -> bool {
        self.SlotFlagOr("applies_clip_path", |node| node.applies_clip_path, |properties| {
            properties.is_some_and(|properties| properties.ClipPathClip().is_some())
        })
    }
    /// ObjectPaintProperties::Effect or Filter (the builder creates Effect for
    /// opacity, blending, backdrop filter, mask or an effect reason).
    pub fn applies_effect(self) -> bool {
        self.SlotFlagOr("applies_effect", |node| node.applies_effect, |properties| {
            properties.is_some_and(|properties| {
                properties.Effect().is_some() || properties.Filter().is_some()
            })
        })
    }
    /// The Effect slot with a non-opaque style opacity (the value the
    /// SaveLayerAlpha replays, as PaintContext's ObjectReplayScopes reads it).
    pub fn applies_opacity(self) -> bool {
        self.SlotFlagOr("applies_opacity", |node| node.applies_opacity, |properties| {
            properties.is_some_and(|properties| properties.Effect().is_some())
                && self
                    .fragment
                    .is_some_and(|fragment| MaskAdjustedOpacity(&fragment.paint.style) != 1.0)
        })
    }
    /// The Effect slot with a blend mode or isolating descendant blending
    /// (NeedsCssBlendEffect).
    pub fn applies_blend(self) -> bool {
        self.SlotFlagOr("applies_blend", |node| node.applies_blend, |properties| {
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
        self.SlotFlagOr("applies_filter", |node| node.applies_filter, |properties| {
            properties.is_some_and(|properties| properties.Filter().is_some())
        })
    }
    /// The BackdropFilterClip slot, created exactly for a backdrop filter.
    pub fn applies_backdrop_filter(self) -> bool {
        self.SlotFlagOr(
            "applies_backdrop_filter",
            |node| node.applies_backdrop_filter,
            |properties| properties.is_some_and(|properties| properties.BackdropFilterClip().is_some()),
        )
    }
    /// ObjectPaintProperties::Mask.
    pub fn applies_mask(self) -> bool {
        self.SlotFlagOr("applies_mask", |node| node.applies_mask, |properties| {
            properties.is_some_and(|properties| properties.Mask().is_some())
        })
    }

    /// FragmentData::LocalBorderBoxProperties (fragment_data.h:137-142).
    pub fn properties(self) -> &'n PaintPropertyTreeState {
        match self.data.and_then(|data| data.LocalBorderBoxPropertiesRef()) {
            Some(state) => {
                self.Verify("properties", |node| SameStateNodes(state, &node.properties.nodes));
                state
            }
            None => {
                &self
                    .node
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
                let value = with_access(|access| data.PreTransform(access));
                self.VerifyUnderCheck("pre_transform", |node| {
                    Arc::ptr_eq(&value, &node.pre_transform)
                });
                value
            }
            _ => match self.node {
                Some(node) => node.pre_transform.clone(),
                None => PaintPropertyTreeState::default().transform,
            },
        }
    }

    /// FragmentData::ContentsProperties (fragment_data.h:170-177).
    pub fn contents_properties(self) -> PaintPropertyTreeState {
        match self.data {
            Some(data) if data.HasLocalBorderBoxProperties() => {
                let value = data.ContentsProperties();
                self.VerifyUnderCheck("contents_properties", |node| {
                    SameStateNodes(&value, &node.contents_properties.nodes)
                });
                value
            }
            _ => self
                .node
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
            return self.node?.fixed_background_properties.clone();
        };
        let value = match (
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
        };
        self.Verify("fixed_background_properties", |node| {
            SameOptionalState(value.as_ref(), node.fixed_background_properties.as_ref())
        });
        value
    }

    /// ScrollableAreaPainter::PaintOverflowControls: the
    /// ObjectPaintProperties::OverflowControlsClip entered over the box's
    /// LocalBorderBoxProperties. `None` when the box has no such clip.
    pub fn overflow_controls_properties(self) -> Option<PaintPropertyTreeState> {
        let Some(data) = self.data else {
            return self.node?.overflow_controls_properties.clone();
        };
        let value = data
            .PaintProperties()
            .and_then(ObjectPaintProperties::OverflowControlsClip)
            .map(|clip| {
                let mut state = data.LocalBorderBoxProperties();
                state.clip = clip.clone();
                state
            });
        self.VerifyUnderCheck("overflow_controls_properties", |node| {
            SameOptionalState(value.as_ref(), node.overflow_controls_properties.as_ref())
        });
        value
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
            return self.node.and_then(node_value).cloned();
        };
        let value = data.PaintProperties().and_then(|properties| {
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
        });
        self.VerifyUnderCheck("scrollbar_properties", |node| {
            SameOptionalState(value.as_ref(), node_value(node))
        });
        value
    }

    /// PaintLayerPainter::PaintFragmentWithPhase(kMask)
    /// (paint_layer_painter.cc:722-729): the LocalBorderBoxProperties with the
    /// Mask effect and its output clip. `None` without a Mask slot.
    pub fn mask_properties(self) -> Option<PaintPropertyTreeState> {
        let Some(data) = self.data else {
            return self.node?.mask_properties.clone();
        };
        let value = data
            .PaintProperties()
            .and_then(ObjectPaintProperties::Mask)
            .map(|mask| {
                let output_clip = with_access(|access| mask.state(access).output_clip.clone())
                    .expect("Mask effect has an output clip");
                PaintPropertyTreeState {
                    transform: self.properties().transform.clone(),
                    clip: output_clip,
                    effect: mask.clone(),
                }
            });
        self.VerifyUnderCheck("mask_properties", |node| {
            SameOptionalState(value.as_ref(), node.mask_properties.as_ref())
        });
        value
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
            None => self.node.map(|node| &node.previous_paint_result),
        }
    }

    // -----------------------------------------------------------------
    // Flat replay values (FragmentData FlatReplayData): Rechrom's flat
    // replay stream only; Chromium paints in the local property space.
    // -----------------------------------------------------------------

    /// Flat-to-property-space delta of the paint offset.
    pub fn paint_snap_offset(self) -> Offset {
        let Some(flat) = self.Flat() else {
            return self.node.map_or_else(Offset::default, |node| node.paint_snap_offset);
        };
        let value = flat.paint_snap_offset;
        self.Verify("paint_snap_offset", |node| value == node.paint_snap_offset);
        value
    }
    /// The walk root's inherited flat clip (the frame-view viewport clip of
    /// the main frame, the owner's accumulated clip of a local frame). The
    /// frame-level ViewportClip is not an ObjectPaintProperties slot
    /// (ObjectPaintPropertySlot::FromRole).
    pub fn root_clip(self) -> Option<PaintRect> {
        if self.data.is_none() {
            return self.node.and_then(|node| node.root_clip);
        }
        let value = self.FlatRare().and_then(|rare| rare.root_clip);
        self.Verify("root_clip", |node| value == node.root_clip);
        value
    }
    pub fn applies_root_clip(self) -> bool {
        let Some(flat) = self.Flat() else {
            return self.node.is_some_and(|node| node.applies_root_clip);
        };
        let value = flat.applies_root_clip;
        self.Verify("applies_root_clip", |node| value == node.applies_root_clip);
        value
    }
    /// The builder's authored clip-path commands in the flat replay space; the
    /// ClipPathClip node holds them translated into property space.
    pub fn local_clip_path(self) -> &'n [PaintPathCommand] {
        if self.data.is_none() {
            return self.node.map_or(&[][..], |node| node.local_clip_path.as_slice());
        }
        let value = self
            .FlatRare()
            .and_then(|rare| rare.clip_path_commands.as_deref())
            .unwrap_or(&[]);
        self.Verify("local_clip_path", |node| value == node.local_clip_path.as_slice());
        value
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
            return self.node.map_or(&[][..], |node| &node.transforms[..]);
        };
        let value = &flat.transforms[..];
        self.Verify("transforms", |node| value == &node.transforms[..]);
        value
    }
    /// The object's own flat matrices, replayed as kConcat items.
    pub fn local_transforms(self) -> &'n [TransformMatrix] {
        if self.data.is_none() {
            return self.node.map_or(&[][..], |node| node.local_transforms.as_slice());
        }
        let value = self
            .FlatRare()
            .and_then(|rare| rare.local_transforms.as_deref())
            .unwrap_or(&[]);
        self.Verify("local_transforms", |node| value == node.local_transforms.as_slice());
        value
    }
    /// Whether the flat replay opens this object's own transform scope. An
    /// SVG child may lead with its container's matrix, which has no slot of
    /// its own, so this stays with the flat matrices.
    pub fn applies_transform(self) -> bool {
        let Some(flat) = self.Flat() else {
            return self.node.is_some_and(|node| node.applies_transform);
        };
        let value = flat.applies_transform;
        self.Verify("applies_transform", |node| value == node.applies_transform);
        value
    }
    /// The flat scroll anchor the fixed background positioning area adds.
    pub fn scroll_offset(self) -> Offset {
        let Some(flat) = self.Flat() else {
            return self.node.map_or_else(Offset::default, |node| node.scroll_offset);
        };
        let value = flat.scroll_offset;
        self.Verify("scroll_offset", |node| value == node.scroll_offset);
        value
    }
    /// CSS transforms on non-root ancestors make background-attachment:fixed
    /// behave as scroll.
    pub fn fixed_background_acts_as_scroll(self) -> bool {
        let Some(flat) = self.Flat() else {
            return self
                .node
                .is_some_and(|node| node.fixed_background_acts_as_scroll);
        };
        let value = flat.fixed_background_acts_as_scroll;
        self.Verify("fixed_background_acts_as_scroll", |node| {
            value == node.fixed_background_acts_as_scroll
        });
        value
    }
    /// The united outline of an inline continuation group, joined across its
    /// occurrences by PrepareOutlines (Chromium's LayoutInline OutlineRects
    /// over every FragmentItem of the inline, which the Layout facade does
    /// not expose yet).
    pub fn paints_outline(self) -> bool {
        let Some(flat) = self.Flat() else {
            return self.node.is_none_or(|node| node.paints_outline);
        };
        let value = flat.paints_outline;
        self.Verify("paints_outline", |node| value == node.paints_outline);
        value
    }
    /// Whether this object's non-layer subtree paints the float phase (the
    /// walk's phase pruning, FlatReplayData).
    pub fn needs_float_phase(self) -> bool {
        let Some(flat) = self.Flat() else {
            return self.node.is_none_or(|node| node.needs_float_phase);
        };
        let value = !flat.skip_float_phase;
        self.Verify("needs_float_phase", |node| value == node.needs_float_phase);
        value
    }
    /// Whether this object's non-layer subtree paints an outline phase.
    pub fn needs_outline_phase(self) -> bool {
        let Some(flat) = self.Flat() else {
            return self.node.is_none_or(|node| node.needs_outline_phase);
        };
        let value = !flat.skip_outline_phase;
        self.Verify("needs_outline_phase", |node| value == node.needs_outline_phase);
        value
    }
    /// Whether this object's non-layer subtree paints overlay overflow
    /// controls.
    pub fn needs_overflow_controls_phase(self) -> bool {
        let Some(flat) = self.Flat() else {
            return self
                .node
                .is_none_or(|node| node.needs_overflow_controls_phase);
        };
        let value = !flat.skip_overflow_controls_phase;
        self.Verify("needs_overflow_controls_phase", |node| {
            value == node.needs_overflow_controls_phase
        });
        value
    }
    pub fn outline_rects(self) -> &'n [PaintRect] {
        if self.data.is_none() {
            return self.node.map_or(&[][..], |node| node.outline_rects.as_slice());
        }
        let value = self
            .FlatRare()
            .and_then(|rare| rare.outline_rects.as_deref())
            .unwrap_or(&[]);
        self.Verify("outline_rects", |node| value == node.outline_rects.as_slice());
        value
    }
    /// PrePaint's resolved mask layers. PrePaint still needs them for the
    /// MaskClip rect of an SVG child (ResolvedMaskClipRect), and resolving
    /// them again here would repeat the SVG mask shader resolution.
    pub fn mask_layers(self) -> &'n [DisplayMaskLayer] {
        if self.data.is_none() {
            return self.node.map_or(&[][..], |node| node.mask_layers.as_slice());
        }
        let value = self
            .FlatRare()
            .and_then(|rare| rare.mask_layers.as_deref())
            .unwrap_or(&[]);
        self.Verify("mask_layers", |node| value == node.mask_layers.as_slice());
        value
    }
}

impl<'n, 'f> From<&'n PaintTreeNode<'f>> for PaintObject<'n, 'f> {
    fn from(node: &'n PaintTreeNode<'f>) -> Self {
        Self::new(node)
    }
}

#[derive(Clone)]
enum ChildSource<'n, 'f> {
    /// The paint walk's resident children.
    Nodes(std::slice::Iter<'n, Box<PaintTreeNode<'f>>>),
    /// The fragment cursor's children.
    Cursors(std::vec::IntoIter<PaintFragmentCursor<'n>>),
}

/// Iterator over a painted object's children.
#[derive(Clone)]
pub struct PaintObjectChildren<'n, 'f> {
    source: ChildSource<'n, 'f>,
    /// Whether the parent carries a walk context its children derive theirs
    /// from (else the children are node-backed, as their parent).
    walked: bool,
    inherited_cull_rect: Option<PaintRect>,
}

impl<'n, 'f> PaintObjectChildren<'n, 'f> {
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn Child(&self, node: Option<&'n PaintTreeNode<'f>>, cursor: Option<PaintFragmentCursor<'n>>) -> PaintObject<'n, 'f> {
        match (node, cursor) {
            (Some(node), _) if self.walked => PaintObject::Walked(
                node,
                PaintWalkContext::new(self.inherited_cull_rect, Some(&node.previous_paint_result)),
            ),
            (Some(node), _) => PaintObject::new(node),
            (None, Some(cursor)) => PaintObject::FromCursor(
                cursor,
                PaintWalkContext::new(self.inherited_cull_rect, None),
            ),
            (None, None) => unreachable!("a paint child has a node or a cursor"),
        }
    }
}

impl<'n, 'f> Iterator for PaintObjectChildren<'n, 'f> {
    type Item = PaintObject<'n, 'f>;

    fn next(&mut self) -> Option<Self::Item> {
        let (node, cursor) = match &mut self.source {
            ChildSource::Nodes(nodes) => (Some(&**nodes.next()?), None),
            ChildSource::Cursors(cursors) => (None, Some(cursors.next()?)),
        };
        Some(self.Child(node, cursor))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.source {
            ChildSource::Nodes(nodes) => nodes.size_hint(),
            ChildSource::Cursors(cursors) => cursors.size_hint(),
        }
    }
}

impl DoubleEndedIterator for PaintObjectChildren<'_, '_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let (node, cursor) = match &mut self.source {
            ChildSource::Nodes(nodes) => (Some(&**nodes.next_back()?), None),
            ChildSource::Cursors(cursors) => (None, Some(cursors.next_back()?)),
        };
        Some(self.Child(node, cursor))
    }
}

impl ExactSizeIterator for PaintObjectChildren<'_, '_> {}
