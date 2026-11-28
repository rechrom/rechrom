//! The painted object as Chromium's painters receive it.
//!
//! Chromium painters take the LayoutObject / PhysicalBoxFragment being
//! painted, the FragmentData of the painted occurrence and a PaintInfo
//! (box_fragment_painter.h:43-60, scoped_paint_state.h:24-60,
//! object_painter.h:21-27). `PaintObject` is that borrowed pair for
//! paint_record: a small `Copy` view over the source fragment cursor, its
//! paint sidecar (`cursor.PaintData()`), the occurrence's
//! `PrePaintFragmentData`, the source `PaintLayer`, and the walk state
//! Chromium keeps as paint traversal state (ScopedDisplayItemFragment).
//!
//! It is still backed by the resident PrePaint node, which is the only place
//! this package names it. The accessors under "Values re-sourced from
//! Layout, FragmentData and PaintLayer" read the owner Chromium reads
//! (ObjectPaintProperties slots, LocalBorderBoxProperties, the Layout
//! PaintLayer, DocumentResources, the LayoutObject) whenever the occurrence
//! has its FragmentData; only hand-built fixtures without one keep the
//! node's copy. Each re-sourced value is checked against the node copy (see
//! `VerifySource`) until the copies are deleted. The accessors under "Walk
//! context" are paint traversal state that still lives on the node; they
//! move to the walk's own context before the backing node goes away.

#![allow(non_snake_case)]

use std::cell::Cell;
use std::sync::Arc;

use layout::fragment::FragmentKind;
use layout::fragment::ResidentBoxGeometry as _;
use layout::paint::{FragmentPaintData, PaintFragmentCursor};
use layout::views::paint_layer::PaintLayer;
use paint_input_types::layout_input::{Offset, PaintClipPathChild, PaintPathCommand};
use paint_input_types::paint_input::{PaintBlendMode, PaintCornerRadii};
use paint_input_types::PaintResources;
use paint_model::paint_property_tree::{
    ClipPaintPropertyNode, PaintPropertyRole, PropertyTreeState as PaintPropertyTreeState,
    TransformPaintPropertyNode,
};
use paint_model::property_cell::with_access;
use paint_model::{DisplayMaskLayer, PaintResult};
use prepaint::fragment_data::PrePaintFragmentData;
use prepaint::object_paint_properties::{ObjectPaintProperties, ObjectPaintPropertySlot};
use prepaint::pre_paint_tree_walk::{FlatTransforms, PaintTreeNode};

use crate::pre_paint_tree_walk::MaskAdjustedOpacity;
use crate::PaintRect;

/// RECHROM_PREPAINT_FRAGMENT_DATA_CHECK (the PrePaint FragmentData proof
/// switch), read once.
fn SourceCheckEnabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("RECHROM_PREPAINT_FRAGMENT_DATA_CHECK").is_some())
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

/// One painted object occurrence: source fragment + FragmentData + walk state.
#[derive(Clone, Copy)]
pub struct PaintObject<'n, 'f> {
    node: &'n PaintTreeNode<'f>,
}

impl<'n, 'f> PaintObject<'n, 'f> {
    /// The resident PrePaint occurrence the paint walk is visiting.
    pub fn new(node: &'n PaintTreeNode<'f>) -> Self {
        Self { node }
    }

    /// The walk-side node, for paint_walk while its own traversal still runs
    /// over PrePaint nodes. Painters never use it.
    #[doc(hidden)]
    pub fn BackingNode(self) -> &'n PaintTreeNode<'f> {
        self.node
    }

    // -----------------------------------------------------------------
    // Source object and FragmentData.
    // -----------------------------------------------------------------

    /// The borrowed fragment cursor of the painted object (the walk handle).
    pub fn SourceCursor(self) -> Option<PaintFragmentCursor<'n>> {
        self.node.SourceCursor()
    }

    /// The fragment's paint sidecar (`cursor.PaintData()`): the paint view of
    /// the PhysicalBoxFragment / FragmentItem and its LayoutObject.
    pub fn Fragment(self) -> Option<&'n FragmentPaintData> {
        self.node.fragment.as_deref()
    }

    /// FragmentData of this occurrence (core/paint/fragment_data.h).
    pub fn FragmentData(self) -> Option<&'n PrePaintFragmentData> {
        self.node.FragmentData()
    }

    /// `To<LayoutBoxModelObject>(object).Layer()` of the source object.
    pub fn PaintLayer(self) -> Option<&'n PaintLayer> {
        self.SourceCursor()?.OwnPaintLayer()
    }

    /// Document / LocalFrameView paint resources of the source object.
    pub fn DocumentResources(self) -> Option<&'n Arc<PaintResources>> {
        self.node.DocumentResources()
    }

    /// The active ScopedDisplayItemFragment (paint traversal state).
    pub fn DisplayItemFragment(self) -> u32 {
        self.node.DisplayItemFragment()
    }

    /// The fragmentainer this occurrence paints inside (traversal state).
    pub fn FragmentainerInstanceId(self) -> u64 {
        self.node.FragmentainerInstanceId()
    }

    /// Whether ObjectPaintProperties holds the slot of `role`
    /// (object_paint_properties.h `HasNode`).
    pub fn HasOwnPropertyRole(self, role: PaintPropertyRole) -> bool {
        let Some(data) = self.FragmentData() else {
            return self.node.HasOwnPropertyRole(role);
        };
        let value = data.HasPropertyRole(role);
        // The node's key log exists only for nodes built with a cache.
        VerifySourceUnderCheck("HasOwnPropertyRole", || {
            value == self.node.HasOwnPropertyRole(role)
        });
        value
    }

    /// The FragmentData of an object with a PaintLayer: only those carry
    /// CullRectUpdater's cull rects (cull_rect_updater.cc:51-90).
    fn LayerFragmentData(self) -> Option<&'n PrePaintFragmentData> {
        let data = self.FragmentData()?;
        self.PaintLayer()?;
        Some(data)
    }

    /// A clip slot's rect and radii in the flat replay space. The builder
    /// placed the flat rect into the node's property space by adding the
    /// paint_snap_offset (PropertySpaceRect); remove it again, as
    /// PaintContext's FlatClipRect does.
    fn FlatSlotClip(self, clip: &ClipPaintPropertyNode) -> Option<(PaintRect, PaintCornerRadii)> {
        let delta = self.node.paint_snap_offset;
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
        node_value: (Option<PaintRect>, PaintCornerRadii),
    ) -> (Option<PaintRect>, PaintCornerRadii) {
        let Some(data) = self.FragmentData() else {
            return node_value;
        };
        let value = match data.PaintProperties().and_then(slot) {
            Some(clip) => match self.FlatSlotClip(clip) {
                Some((rect, radii)) => (Some(rect), radii),
                None => (None, PaintCornerRadii::default()),
            },
            None => (None, PaintCornerRadii::default()),
        };
        VerifySource(accessor, || {
            SameOptionalRect(value.0, node_value.0) && value.1 == node_value.1
        });
        value
    }

    /// An `applies_*` flag from the ObjectPaintProperties slots, or the node's
    /// fixture copy when the occurrence has no FragmentData.
    fn SlotFlagOr(
        self,
        accessor: &str,
        node_value: bool,
        derive: impl FnOnce(Option<&'n ObjectPaintProperties>) -> bool,
    ) -> bool {
        let Some(data) = self.FragmentData() else {
            return node_value;
        };
        let value = derive(data.PaintProperties());
        VerifySource(accessor, || value == node_value);
        value
    }

    // -----------------------------------------------------------------
    // Paint offset and snapping in the flat replay space.
    // -----------------------------------------------------------------

    /// FragmentData::PaintOffset() in the flat replay space.
    pub fn FlatPaintOffset(self) -> Offset {
        self.node.FlatPaintOffset()
    }

    /// FlatPaintOffset with this occurrence's FragmentData already resolved.
    pub fn FlatPaintOffsetOf(self, data: Option<&PrePaintFragmentData>) -> Offset {
        self.node.FlatPaintOffsetOf(data)
    }

    pub fn PixelSnappedRect(self, rect: &PaintRect) -> PaintRect {
        self.node.PixelSnappedRect(rect)
    }

    pub fn EnclosingRect(self, rect: PaintRect) -> PaintRect {
        self.node.EnclosingRect(rect)
    }

    pub fn PixelSnappedOffset(self, offset: Offset) -> Offset {
        self.node.PixelSnappedOffset(offset)
    }

    /// ObjectPaintProperties::MaskClip source rect in the flat replay space.
    pub fn ResolvedMaskClipRect(self) -> Option<PaintRect> {
        self.node.ResolvedMaskClipRect()
    }

    // -----------------------------------------------------------------
    // Tree access.
    // -----------------------------------------------------------------

    /// The painted children of this object in PrePaint traversal order.
    pub fn Children(self) -> PaintObjectChildren<'n, 'f> {
        PaintObjectChildren {
            inner: self.node.children.iter(),
        }
    }

    /// The painted SVG resource subtree root registered for `client`.
    pub fn SvgResourceNode(self, client: u64) -> Option<PaintObject<'n, 'static>> {
        self.node.SvgResourceNode(client).map(PaintObject::new)
    }

    // -----------------------------------------------------------------
    // Values re-sourced from Layout, FragmentData and PaintLayer. Every one
    // reads the node's copy only for a fixture without FragmentData.
    // -----------------------------------------------------------------

    /// Device scale used to snap paint geometry: the LocalFrameView's
    /// DocumentResources scale PaintPropertyTreeBuilder snaps with.
    pub fn pixel_snap_scale(self) -> f64 {
        let node_value = self.node.pixel_snap_scale;
        if self.FragmentData().is_none() {
            return node_value;
        }
        let value = self
            .DocumentResources()
            .map_or(1.0, |resources| resources.ArtifactToDeviceScale())
            .max(f64::MIN_POSITIVE);
        VerifySource("pixel_snap_scale", || value == node_value);
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
            (
                self.node.local_inner_border_radius_clip,
                self.node.local_inner_border_radius_radii,
            ),
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
            (self.node.local_clip, self.node.local_clip_radii),
        )
    }

    /// ObjectPaintProperties::CssClip in the flat replay space.
    pub fn local_css_clip(self) -> Option<PaintRect> {
        self.FlatClipSlotOr(
            "local_css_clip",
            |properties| properties.CssClip(),
            (self.node.local_css_clip, PaintCornerRadii::default()),
        )
        .0
    }

    /// The fill rule of ClipPathClip's path: PaintClipPath::from_authored
    /// builds the single authored path, or one path per reference child, with
    /// the rule PrePaint resolved (the first child's for a reference graph).
    pub fn clip_path_even_odd(self) -> bool {
        self.SlotFlagOr("clip_path_even_odd", self.node.clip_path_even_odd, |properties| {
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

    /// The contents cull rect of a PaintLayer object (FragmentData::
    /// GetContentsCullRect, fragment_data.h:163-168; infinite is None). Other
    /// objects keep the walk's PaintInfo cull rect (walk context).
    pub fn cull_rect(self) -> Option<PaintRect> {
        let node_value = self.node.cull_rect;
        let Some(data) = self.LayerFragmentData() else {
            return node_value;
        };
        let value = data.GetContentsCullRect().Rect();
        VerifySource("cull_rect", || value == node_value);
        value
    }

    /// The cull rect of a PaintLayer object (FragmentData::GetCullRect,
    /// fragment_data.h:157-162); other objects keep the walk's PaintInfo cull
    /// rect (walk context).
    pub fn self_cull_rect(self) -> Option<PaintRect> {
        let node_value = self.node.self_cull_rect;
        let Some(data) = self.LayerFragmentData() else {
            return node_value;
        };
        let value = data.GetCullRect().Rect();
        VerifySource("self_cull_rect", || value == node_value);
        value
    }

    /// ObjectPaintProperties::PaintOffsetTranslation (object_paint_properties.h).
    pub fn paint_offset_translation(self) -> Option<&'n Arc<TransformPaintPropertyNode>> {
        let node_value = self.node.paint_offset_translation.as_ref();
        let Some(data) = self.FragmentData() else {
            return node_value;
        };
        let value = data
            .PaintProperties()
            .and_then(ObjectPaintProperties::PaintOffsetTranslation);
        VerifySource("paint_offset_translation", || match (value, node_value) {
            (Some(value), Some(node_value)) => Arc::ptr_eq(value, node_value),
            (None, None) => true,
            _ => false,
        });
        value
    }

    /// Whether the object paints as its own PaintLayer: PrePaint's
    /// ClassifyFragmentPaintLayer over the source PaintLayer
    /// (`HasSelfPaintingLayer()`, layout_box_model_object.h) of a box
    /// fragment establishing paint state, plus a masked object (the builder
    /// isolates every Mask effect, SVG children included).
    pub fn is_paint_layer(self) -> bool {
        let node_value = self.node.is_paint_layer;
        let (Some(data), Some(fragment)) = (self.FragmentData(), self.Fragment()) else {
            return node_value;
        };
        if !fragment.paint.PaintLayerStateIsAuthoritative() {
            return node_value;
        }
        let value = (fragment.ViewKind() == FragmentKind::kBox
            && fragment.ViewEstablishesPaintState()
            && self.PaintLayer().is_some_and(PaintLayer::IsSelfPaintingLayer))
            || data.HasPropertySlot(ObjectPaintPropertySlot::Mask);
        VerifySource("is_paint_layer", || value == node_value);
        value
    }

    /// PaintLayer::IsSelfPaintingLayer (paint_layer.h:210) of the source
    /// object; without a layer, Layout's exported fact for a source-backed
    /// object, else the PrePaint layer classification.
    pub fn is_self_painting_layer(self) -> bool {
        let node_value = self.node.is_self_painting_layer;
        let (Some(_), Some(fragment)) = (self.FragmentData(), self.Fragment()) else {
            return node_value;
        };
        let value = match self.PaintLayer() {
            Some(layer) => layer.IsSelfPaintingLayer(),
            None if fragment.paint.PaintLayerStateIsAuthoritative() => {
                fragment.paint.PaintLayerIsSelfPainting()
            }
            None => self.is_paint_layer(),
        };
        VerifySource("is_self_painting_layer", || value == node_value);
        value
    }

    /// `layer.GetLayoutObject().IsFragmented()` of the source PaintLayer
    /// (layout_object.h IsFragmented: more than one FragmentData).
    pub fn paint_layer_is_fragmented(self) -> bool {
        let node_value = self.node.paint_layer_is_fragmented;
        let (Some(_), Some(fragment)) = (self.FragmentData(), self.Fragment()) else {
            return node_value;
        };
        let value = match self.PaintLayer() {
            Some(layer) => layer.GetLayoutObject().IsFragmented(),
            None => fragment.paint.PaintLayerIsFragmented(),
        };
        VerifySource("paint_layer_is_fragmented", || value == node_value);
        value
    }

    /// Whether the painted object is the LayoutView (`IsA<LayoutView>`,
    /// box_fragment_painter.cc PaintBoxDecorationBackground): the root of
    /// every PrePaint walk, a local frame's included.
    pub fn is_root(self) -> bool {
        let node_value = self.node.is_root;
        let (Some(_), Some(fragment)) = (self.FragmentData(), self.Fragment()) else {
            return node_value;
        };
        let value = fragment.paint.IsLayoutView();
        VerifySource("is_root", || value == node_value);
        value
    }

    /// PaintLayer::NeedsReorderOverlayOverflowControls (paint_layer.h:
    /// 427-431), set by the owning PaintLayerStackingNode's rebuild and read
    /// by PaintLayerPainter for the same decision. The node cell is written
    /// only by the resident hit-test collector, so it is compared under the
    /// check switch only.
    pub fn needs_reorder_overlay_overflow_controls(self) -> bool {
        let node_value = self.node.needs_reorder_overlay_overflow_controls.get();
        if self.FragmentData().is_none() {
            return node_value;
        }
        let Some(layer) = self.PaintLayer() else {
            return node_value;
        };
        let value = layer.NeedsReorderOverlayOverflowControls();
        VerifySourceUnderCheck("needs_reorder_overlay_overflow_controls", || {
            value == node_value
        });
        value
    }

    /// ObjectPaintProperties::OverflowClip.
    pub fn applies_overflow_clip(self) -> bool {
        self.SlotFlagOr("applies_overflow_clip", self.node.applies_overflow_clip, |properties| {
            properties.is_some_and(|properties| properties.OverflowClip().is_some())
        })
    }
    /// ObjectPaintProperties::CssClip.
    pub fn applies_css_clip(self) -> bool {
        self.SlotFlagOr("applies_css_clip", self.node.applies_css_clip, |properties| {
            properties.is_some_and(|properties| properties.CssClip().is_some())
        })
    }
    /// ObjectPaintProperties::ClipPathClip.
    pub fn applies_clip_path(self) -> bool {
        self.SlotFlagOr("applies_clip_path", self.node.applies_clip_path, |properties| {
            properties.is_some_and(|properties| properties.ClipPathClip().is_some())
        })
    }
    /// ObjectPaintProperties::Effect or Filter (the builder creates Effect for
    /// opacity, blending, backdrop filter, mask or an effect reason).
    pub fn applies_effect(self) -> bool {
        self.SlotFlagOr("applies_effect", self.node.applies_effect, |properties| {
            properties.is_some_and(|properties| {
                properties.Effect().is_some() || properties.Filter().is_some()
            })
        })
    }
    /// The Effect slot with a non-opaque style opacity (the value the
    /// SaveLayerAlpha replays, as PaintContext's ObjectReplayScopes reads it).
    pub fn applies_opacity(self) -> bool {
        self.SlotFlagOr("applies_opacity", self.node.applies_opacity, |properties| {
            properties.is_some_and(|properties| properties.Effect().is_some())
                && self
                    .Fragment()
                    .is_some_and(|fragment| MaskAdjustedOpacity(&fragment.paint.style) != 1.0)
        })
    }
    /// The Effect slot with a blend mode or isolating descendant blending
    /// (NeedsCssBlendEffect).
    pub fn applies_blend(self) -> bool {
        self.SlotFlagOr("applies_blend", self.node.applies_blend, |properties| {
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
        self.SlotFlagOr("applies_filter", self.node.applies_filter, |properties| {
            properties.is_some_and(|properties| properties.Filter().is_some())
        })
    }
    /// The BackdropFilterClip slot, created exactly for a backdrop filter.
    pub fn applies_backdrop_filter(self) -> bool {
        self.SlotFlagOr("applies_backdrop_filter", self.node.applies_backdrop_filter, |properties| {
            properties.is_some_and(|properties| properties.BackdropFilterClip().is_some())
        })
    }
    /// ObjectPaintProperties::Mask.
    pub fn applies_mask(self) -> bool {
        self.SlotFlagOr("applies_mask", self.node.applies_mask, |properties| {
            properties.is_some_and(|properties| properties.Mask().is_some())
        })
    }

    /// FragmentData::LocalBorderBoxProperties (fragment_data.h:137-142).
    pub fn properties(self) -> &'n PaintPropertyTreeState {
        let node_value = &self.node.properties.nodes;
        match self
            .FragmentData()
            .and_then(|data| data.LocalBorderBoxPropertiesRef())
        {
            Some(state) => {
                VerifySource("properties", || SameStateNodes(state, node_value));
                state
            }
            None => node_value,
        }
    }

    /// FragmentData::PreTransform (fragment_data.cc:55-65).
    pub fn pre_transform(self) -> Arc<TransformPaintPropertyNode> {
        let node_value = &self.node.pre_transform;
        match self.FragmentData() {
            Some(data) if data.HasLocalBorderBoxProperties() => {
                let value = with_access(|access| data.PreTransform(access));
                VerifySourceUnderCheck("pre_transform", || Arc::ptr_eq(&value, node_value));
                value
            }
            _ => node_value.clone(),
        }
    }

    /// FragmentData::ContentsProperties (fragment_data.h:170-177).
    pub fn contents_properties(self) -> PaintPropertyTreeState {
        let node_value = &self.node.contents_properties.nodes;
        match self.FragmentData() {
            Some(data) if data.HasLocalBorderBoxProperties() => {
                let value = data.ContentsProperties();
                VerifySourceUnderCheck("contents_properties", || {
                    SameStateNodes(&value, node_value)
                });
                value
            }
            _ => node_value.clone(),
        }
    }

    /// The composited fixed background state: the BackgroundClip slot over
    /// the LocalBorderBoxProperties effect, in the frame's viewport transform
    /// (BoxFragmentPainter::PaintCompositeBackgroundAttachmentFixed). That
    /// transform is walk context the builder carried; it stays on the node.
    pub fn fixed_background_properties(self) -> Option<PaintPropertyTreeState> {
        let node_value = self.node.fixed_background_properties.as_ref();
        let Some(data) = self.FragmentData() else {
            return node_value.cloned();
        };
        let value = match (
            data.PaintProperties().and_then(ObjectPaintProperties::BackgroundClip),
            node_value,
        ) {
            (Some(clip), Some(node_state)) => Some(PaintPropertyTreeState {
                transform: node_state.transform.clone(),
                clip: clip.clone(),
                effect: data.LocalBorderBoxProperties().effect,
            }),
            _ => None,
        };
        VerifySource("fixed_background_properties", || {
            SameOptionalState(value.as_ref(), node_value)
        });
        value
    }

    /// ScrollableAreaPainter::PaintOverflowControls: the
    /// ObjectPaintProperties::OverflowControlsClip entered over the box's
    /// LocalBorderBoxProperties. `None` when the box has no such clip.
    pub fn overflow_controls_properties(self) -> Option<PaintPropertyTreeState> {
        let node_value = self.node.overflow_controls_properties.as_ref();
        let Some(data) = self.FragmentData() else {
            return node_value.cloned();
        };
        let value = data
            .PaintProperties()
            .and_then(ObjectPaintProperties::OverflowControlsClip)
            .map(|clip| {
                let mut state = data.LocalBorderBoxProperties();
                state.clip = clip.clone();
                state
            });
        VerifySourceUnderCheck("overflow_controls_properties", || {
            SameOptionalState(value.as_ref(), node_value)
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
        let node_value = if horizontal {
            self.node.horizontal_scrollbar_properties.as_ref()
        } else {
            self.node.vertical_scrollbar_properties.as_ref()
        };
        let Some(data) = self.FragmentData() else {
            return node_value.cloned();
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
        VerifySourceUnderCheck("scrollbar_properties", || {
            SameOptionalState(value.as_ref(), node_value)
        });
        value
    }

    /// PaintLayerPainter::PaintFragmentWithPhase(kMask)
    /// (paint_layer_painter.cc:722-729): the LocalBorderBoxProperties with the
    /// Mask effect and its output clip. `None` without a Mask slot.
    pub fn mask_properties(self) -> Option<PaintPropertyTreeState> {
        let node_value = self.node.mask_properties.as_ref();
        let Some(data) = self.FragmentData() else {
            return node_value.cloned();
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
        VerifySourceUnderCheck("mask_properties", || {
            SameOptionalState(value.as_ref(), node_value)
        });
        value
    }

    // -----------------------------------------------------------------
    // Walk context: paint traversal state the PrePaint walk still carries on
    // the node (S5b-2 moves it into the walk's own context), and PrePaint
    // results that have no FragmentData / PaintLayer / Layout owner yet.
    // -----------------------------------------------------------------

    /// Flat-to-property-space delta of the paint offset.
    pub fn paint_snap_offset(self) -> Offset {
        self.node.paint_snap_offset
    }
    /// The walk root's inherited flat clip (the frame-view viewport clip of
    /// the main frame, the owner's accumulated clip of a local frame). The
    /// frame-level ViewportClip is not an ObjectPaintProperties slot
    /// (ObjectPaintPropertySlot::FromRole).
    pub fn root_clip(self) -> Option<PaintRect> {
        self.node.root_clip
    }
    pub fn applies_root_clip(self) -> bool {
        self.node.applies_root_clip
    }
    /// The builder's authored clip-path commands in the flat replay space; the
    /// ClipPathClip node holds them translated into property space.
    pub fn local_clip_path(self) -> &'n Vec<PaintPathCommand> {
        &self.node.local_clip_path
    }
    pub fn local_clip_path_children(self) -> &'n Vec<PaintClipPathChild> {
        &self.node.local_clip_path_children
    }
    pub fn transforms(self) -> &'n FlatTransforms {
        &self.node.transforms
    }
    pub fn local_transforms(self) -> &'n Vec<paint_input_types::layout_input::TransformMatrix> {
        &self.node.local_transforms
    }
    /// Whether the flat replay opens this object's own transform scope. An
    /// SVG child may lead with its container's matrix, which has no slot of
    /// its own, so this stays with the walk's flat matrices.
    pub fn applies_transform(self) -> bool {
        self.node.applies_transform
    }
    pub fn scroll_offset(self) -> Offset {
        self.node.scroll_offset
    }
    pub fn fixed_background_acts_as_scroll(self) -> bool {
        self.node.fixed_background_acts_as_scroll
    }
    /// PaintLayerPainter's in-flight PaintLayer::PreviousPaintResult: seeded
    /// per traversal and downgraded by ScopedBoxContentsPaintState before
    /// PaintLayerPainter writes the layer (and not written at all under
    /// kOmitCompositingInfo), so it is traversal state until the walk writes
    /// the PaintLayer directly.
    pub fn previous_paint_result(self) -> &'n Cell<PaintResult> {
        &self.node.previous_paint_result
    }
    /// The united outline of an inline continuation group, joined across its
    /// occurrences by PrepareOutlines (Chromium's LayoutInline OutlineRects
    /// over every FragmentItem of the inline, which the Layout facade does
    /// not expose yet).
    pub fn paints_outline(self) -> bool {
        self.node.paints_outline
    }
    pub fn outline_rects(self) -> &'n Vec<PaintRect> {
        &self.node.outline_rects
    }
    /// PrePaint's resolved mask layers. PrePaint still needs them for the
    /// MaskClip rect of an SVG child (ResolvedMaskClipRect), and resolving
    /// them again here would repeat the SVG mask shader resolution.
    pub fn mask_layers(self) -> &'n Vec<DisplayMaskLayer> {
        &self.node.mask_layers
    }
}

impl<'n, 'f> From<&'n PaintTreeNode<'f>> for PaintObject<'n, 'f> {
    fn from(node: &'n PaintTreeNode<'f>) -> Self {
        Self::new(node)
    }
}

/// Iterator over a painted object's children.
#[derive(Clone)]
pub struct PaintObjectChildren<'n, 'f> {
    inner: std::slice::Iter<'n, Box<PaintTreeNode<'f>>>,
}

impl<'n, 'f> PaintObjectChildren<'n, 'f> {
    pub fn is_empty(&self) -> bool {
        self.inner.len() == 0
    }
}

impl<'n, 'f> Iterator for PaintObjectChildren<'n, 'f> {
    type Item = PaintObject<'n, 'f>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|child| PaintObject::new(child))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl DoubleEndedIterator for PaintObjectChildren<'_, '_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|child| PaintObject::new(child))
    }
}

impl ExactSizeIterator for PaintObjectChildren<'_, '_> {}
