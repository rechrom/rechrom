//! Source: third_party/blink/renderer/core/paint/fragment_data.h and
//! fragment_data.cc (Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15).
//!
//! `PrePaintFragmentData` is the PrePaint half of Chromium's `FragmentData`
//! for one fragment occurrence: the value type of the PrePaint sidecar slot
//! that Layout's per-occurrence paint data holds (WP1a provides the generic
//! slot; layout_base never names this type). It owns that occurrence's
//! `ObjectPaintProperties` and the cached property-tree state, written by
//! PaintPropertyTreeBuilder and CullRectUpdater and read by Paint.
//!
//! Not here: the first-fragment rare fields `layer` (PaintLayer lives on the
//! LayoutObject, WP1b), sticky constraints (Layout publishes them on the
//! paint data), `unique_id` (Layout's property fragment unique id) and
//! `additional_fragments` (one sidecar per occurrence replaces
//! FragmentDataList). `previous_gap_rules` is LayoutBox state, not
//! FragmentData.
//!
//! Replaces, per occurrence, the PaintTreeNode fields `paint_offset` (now in
//! the LOCAL space of the property tree state's transform, Chromium's
//! PaintOffset, not Rechrom's flat accumulated offset), `properties`
//! (LocalBorderBoxProperties), `contents_properties` / `pre_transform`
//! (derived below), `cull_rect` / `self_cull_rect`, and `NodeCache::own_keys`
//! (the present ObjectPaintProperties slots).

use std::sync::Arc;

use foundation::PhysicalOffset;
use paint_model::paint_property_tree::{
    ClipPaintPropertyNode, EffectPaintPropertyNode, PaintPropertyRole, PropertyTreeState,
    TransformPaintPropertyNode,
};
use paint_model::property_cell::PropertyAccess;
use paint_model::{CullRect, PaintRect};

use crate::object_paint_properties::{ObjectPaintProperties, ObjectPaintPropertySlot};

/// Source: fragment_data.h:205-235 `FragmentData::RareData`, fragment
/// specific fields only.
#[derive(Debug)]
struct RareData {
    paint_properties: Option<Box<ObjectPaintProperties>>,
    /// `TraceablePropertyTreeStateOrAlias local_border_box_properties
    /// {kUninitialized}`: `None` is uninitialized.
    local_border_box_properties: Option<PropertyTreeState>,
    cull_rect: CullRect,
    contents_cull_rect: CullRect,
}

impl Default for RareData {
    fn default() -> Self {
        Self {
            paint_properties: None,
            local_border_box_properties: None,
            cull_rect: DefaultCullRect(),
            contents_cull_rect: DefaultCullRect(),
        }
    }
}

/// Chromium's default-constructed `CullRect()` holds an empty `gfx::Rect`
/// (cull_rect.h), unlike Rechrom's `CullRect::default()`, which is infinite.
fn DefaultCullRect() -> CullRect {
    CullRect::FromRect(PaintRect::default())
}

/// Source: fragment_data.h:30-245 `FragmentData` (paint side). Not `Clone`:
/// one live owner per occurrence, as Chromium's GarbageCollected object.
#[derive(Debug, Default)]
pub struct PrePaintFragmentData {
    paint_offset: PhysicalOffset,
    rare_data: Option<Box<RareData>>,
}

impl PrePaintFragmentData {
    pub fn new() -> Self {
        Self::default()
    }

    fn EnsureRareData(&mut self) -> &mut RareData {
        self.rare_data.get_or_insert_with(Default::default)
    }

    /// Source: fragment_data.h:32-38. Physical offset of this fragment's
    /// local border box top-left from the origin of the transform node of
    /// its property tree state (LOCAL space, not the flat document offset).
    pub fn PaintOffset(&self) -> PhysicalOffset {
        self.paint_offset
    }

    pub fn SetPaintOffset(&mut self, paint_offset: PhysicalOffset) {
        self.paint_offset = paint_offset;
    }

    /// Source: fragment_data.h:104-110.
    pub fn PaintProperties(&self) -> Option<&ObjectPaintProperties> {
        self.rare_data.as_ref()?.paint_properties.as_deref()
    }

    pub fn PaintPropertiesMut(&mut self) -> Option<&mut ObjectPaintProperties> {
        self.rare_data.as_mut()?.paint_properties.as_deref_mut()
    }

    /// Source: fragment_data.h:111-118.
    pub fn EnsurePaintProperties(&mut self) -> &mut ObjectPaintProperties {
        self.EnsureRareData()
            .paint_properties
            .get_or_insert_with(Default::default)
    }

    /// Source: fragment_data.h:119-122. Dropping the ObjectPaintProperties
    /// releases every slot node (artifacts still naming one keep it alive).
    pub fn ClearPaintProperties(&mut self) {
        if let Some(rare_data) = &mut self.rare_data {
            rare_data.paint_properties = None;
        }
    }

    /// Whether this occurrence owns any paint property node (replaces
    /// `PaintTreeNode::HasOwnPaintProperties`).
    pub fn HasPaintProperties(&self) -> bool {
        self.PaintProperties()
            .is_some_and(ObjectPaintProperties::HasAnyNode)
    }

    /// Whether the slot of `role` holds a node of this occurrence (replaces
    /// `PaintTreeNode::HasOwnPropertyRole`).
    pub fn HasPropertyRole(&self, role: PaintPropertyRole) -> bool {
        self.PaintProperties()
            .is_some_and(|properties| properties.HasRole(role))
    }

    pub fn HasPropertySlot(&self, slot: ObjectPaintPropertySlot) -> bool {
        self.PaintProperties()
            .is_some_and(|properties| properties.HasSlot(slot))
    }

    /// Source: fragment_data.h:137-142 and fragment_data.cc:145-162: the
    /// cached local border box state, or the root state when it was never
    /// set (Chromium's NOTREACHED / DumpWithoutCrashing fallback).
    pub fn LocalBorderBoxProperties(&self) -> PropertyTreeState {
        match self.LocalBorderBoxPropertiesRef() {
            Some(state) => state.clone(),
            None => self.LocalBorderBoxPropertiesFallback(),
        }
    }

    /// The cached local border box state without the fallback, borrowed.
    pub fn LocalBorderBoxPropertiesRef(&self) -> Option<&PropertyTreeState> {
        self.rare_data
            .as_ref()?
            .local_border_box_properties
            .as_ref()
    }

    #[cold]
    #[inline(never)]
    fn LocalBorderBoxPropertiesFallback(&self) -> PropertyTreeState {
        debug_assert!(false, "LocalBorderBoxProperties read before it was set");
        PropertyTreeState::default()
    }

    /// One node of `LocalBorderBoxProperties()`, without cloning the others.
    fn LocalBorderBoxNode<T>(&self, pick: impl Fn(&PropertyTreeState) -> &Arc<T>) -> Arc<T> {
        match self.LocalBorderBoxPropertiesRef() {
            Some(state) => pick(state).clone(),
            None => pick(&self.LocalBorderBoxPropertiesFallback()).clone(),
        }
    }

    /// Source: fragment_data.h:143-146.
    pub fn HasLocalBorderBoxProperties(&self) -> bool {
        self.LocalBorderBoxPropertiesRef().is_some()
    }

    /// Source: fragment_data.h:147-151.
    pub fn ClearLocalBorderBoxProperties(&mut self) {
        if let Some(rare_data) = &mut self.rare_data {
            rare_data.local_border_box_properties = None;
        }
    }

    /// Source: fragment_data.h:152-155.
    pub fn SetLocalBorderBoxProperties(&mut self, state: PropertyTreeState) {
        self.EnsureRareData().local_border_box_properties = Some(state);
    }

    /// Source: fragment_data.h:157-162.
    pub fn SetCullRect(&mut self, cull_rect: CullRect) {
        self.EnsureRareData().cull_rect = cull_rect;
    }

    pub fn GetCullRect(&self) -> CullRect {
        self.rare_data
            .as_ref()
            .map_or_else(DefaultCullRect, |rare_data| rare_data.cull_rect)
    }

    /// Source: fragment_data.h:163-168.
    pub fn SetContentsCullRect(&mut self, contents_cull_rect: CullRect) {
        self.EnsureRareData().contents_cull_rect = contents_cull_rect;
    }

    pub fn GetContentsCullRect(&self) -> CullRect {
        self.rare_data
            .as_ref()
            .map_or_else(DefaultCullRect, |rare_data| rare_data.contents_cull_rect)
    }

    /// Source: fragment_data.h:170-177 `ContentsProperties`: the state that
    /// paints this fragment's contents (overflow clip, scroll translation).
    pub fn ContentsProperties(&self) -> PropertyTreeState {
        PropertyTreeState {
            transform: self.ContentsTransform(),
            clip: self.ContentsClip(),
            effect: self.ContentsEffect(),
        }
    }

    /// Source: fragment_data.cc:55-65 `PreTransform`: the parent of the
    /// outermost CSS transform node, else the local border box transform.
    pub fn PreTransform(&self, access: &PropertyAccess) -> Arc<TransformPaintPropertyNode> {
        if let Some(properties) = self.PaintProperties() {
            if let Some(transform) = properties.CSSTransformPropertiesOutsideToInside().next() {
                let parent = transform.Parent(access);
                debug_assert!(parent.is_some());
                if let Some(parent) = parent {
                    return parent.clone();
                }
            }
        }
        self.LocalBorderBoxNode(|state| &state.transform)
    }

    /// Source: fragment_data.cc:67-87 `ContentsTransform`. Chromium first
    /// returns the TransformIsolationNode alias; paint_model has no parent
    /// alias node kind and Rechrom builds no isolation nodes, so that step
    /// is absent until both exist.
    pub fn ContentsTransform(&self) -> Arc<TransformPaintPropertyNode> {
        if let Some(properties) = self.PaintProperties() {
            if let Some(node) = properties
                .ScrollTranslation()
                .or_else(|| properties.ReplacedContentTransform())
                .or_else(|| properties.Perspective())
                .or_else(|| properties.ContentTranslation())
            {
                return node.clone();
            }
        }
        self.LocalBorderBoxNode(|state| &state.transform)
    }

    /// The earlier Chromium name of `ContentsTransform` (the transform
    /// after the scroll translation), kept for callers translated from it.
    pub fn PostScrollTranslation(&self) -> Arc<TransformPaintPropertyNode> {
        self.ContentsTransform()
    }

    /// Source: fragment_data.cc:89-109 `PreClip`.
    pub fn PreClip(&self, access: &PropertyAccess) -> Arc<ClipPaintPropertyNode> {
        if let Some(properties) = self.PaintProperties() {
            if let Some(clip) = properties
                .ClipPathClip()
                .or_else(|| properties.MaskClip())
                .or_else(|| properties.CssClip())
                .or_else(|| properties.PixelMovingFilterClipExpander())
            {
                let parent = clip.Parent(access);
                debug_assert!(parent.is_some());
                if let Some(parent) = parent {
                    return parent.clone();
                }
            }
        }
        self.LocalBorderBoxNode(|state| &state.clip)
    }

    /// Source: fragment_data.cc:111-121 `ContentsClip` (without the
    /// ClipIsolationNode alias step, see `ContentsTransform`).
    pub fn ContentsClip(&self) -> Arc<ClipPaintPropertyNode> {
        if let Some(properties) = self.PaintProperties() {
            if let Some(node) = properties
                .OverflowClip()
                .or_else(|| properties.InnerBorderRadiusClip())
            {
                return node.clone();
            }
        }
        self.LocalBorderBoxNode(|state| &state.clip)
    }

    /// Source: fragment_data.cc:123-135 `PreEffect`.
    pub fn PreEffect(&self, access: &PropertyAccess) -> Arc<EffectPaintPropertyNode> {
        if let Some(properties) = self.PaintProperties() {
            if let Some(effect) = properties.Effect().or_else(|| properties.Filter()) {
                let parent = effect.Parent(access);
                debug_assert!(parent.is_some());
                if let Some(parent) = parent {
                    return parent.clone();
                }
            }
        }
        self.LocalBorderBoxNode(|state| &state.effect)
    }

    /// Source: fragment_data.cc:137-143 `ContentsEffect` (without the
    /// EffectIsolationNode alias step, see `ContentsTransform`).
    pub fn ContentsEffect(&self) -> Arc<EffectPaintPropertyNode> {
        self.LocalBorderBoxNode(|state| &state.effect)
    }
}
