//! Blink-style paint property references for the supported standalone input.
//!
//! Sources: platform/graphics/paint/property_tree_state.h and the transform,
//! clip, effect and scroll_paint_property_node.h files. These are paint trees,
//! not compositor layer trees. Numeric IDs are dump-local labels; reference
//! equality, as in Blink, determines whether states share property nodes.
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::atomic::{AtomicI32, AtomicU64, AtomicU8, Ordering};
use std::sync::{Arc, OnceLock};

use crate::paint_clip_path::PaintClipPath;
use crate::property_filter_operations::PropertyFilterOperations;
use crate::PaintRect;
use layoutng_assembly::fragment_tree::StickyPositionPaintData;
pub use layoutng_assembly::internal::layout_input::OverscrollBehavior as LayoutOverscrollBehavior;
use layoutng_assembly::internal::layout_input::{Offset, TransformMatrix};
pub use layoutng_assembly::internal::paint_input::PaintBlendMode;
use layoutng_assembly::internal::paint_input::PaintCornerRadii;

/// Source: paint_property_node.h. Ordinary retained updates use the source's
/// non-running-animation default; explicit effect comparisons can preserve the
/// composited-value category while a compositor animation is running.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum PaintPropertyChangeType {
    #[default]
    Unchanged,
    ChangedOnlyCompositedValues,
    ChangedOnlySimpleValues,
    ChangedOnlyValues,
    NodeAddedOrRemoved,
}
impl PaintPropertyChangeType {
    /// Source: paint_property_node.cc:89-102.
    pub const fn AsString(self) -> &'static str {
        match self {
            Self::Unchanged => "unchanged",
            Self::ChangedOnlyCompositedValues => "composited-values",
            Self::ChangedOnlySimpleValues => "simple-values",
            Self::ChangedOnlyValues => "values",
            Self::NodeAddedOrRemoved => "node-add-remove",
        }
    }
}
impl fmt::Display for PaintPropertyChangeType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.AsString())
    }
}

/// A separately owned, persistent node entity. Blink mutates its GC node in
/// place; Rust retains immutable frame states referring to this same entity.
/// Its allocation, rather than a client hash or dump ID, defines identity.
#[derive(Debug)]
pub struct PaintPropertyNodeIdentity {
    // Chromium: third_party/blink/renderer/platform/graphics/paint/
    // paint_property_node.h:91-97,228-230. The pair is one mutable cache slot
    // on the actual Paint property node, keyed by the cc property-tree
    // sequence. Packing both i32 values makes concurrent readers observe one
    // complete source state without adding a compositor-owned side table.
    cc_sequence_and_node_id: AtomicU64,
    // Chromium: paint_property_node.h:191-230. These are mutable members of
    // the resident Paint node even when the property state is reached through
    // a const PaintArtifact. Keeping them on the persistent identity allocation
    // preserves that ownership across immutable Rust value revisions.
    changed: AtomicU8,
    changed_sequence_number: AtomicI32,
    _private: (),
}

impl PaintPropertyNodeIdentity {
    const INVALID_CC_NODE_ID: i32 = -1;

    fn pack_cc_mapping(sequence_number: i32, node_id: i32) -> u64 {
        ((sequence_number as u32 as u64) << 32) | node_id as u32 as u64
    }

    fn cc_node_id(&self, sequence_number: i32) -> i32 {
        let mapping = self.cc_sequence_and_node_id.load(Ordering::Relaxed);
        let mapped_sequence = (mapping >> 32) as u32 as i32;
        if mapped_sequence == sequence_number {
            mapping as u32 as i32
        } else {
            Self::INVALID_CC_NODE_ID
        }
    }

    fn set_cc_node_id(&self, sequence_number: i32, node_id: i32) {
        self.cc_sequence_and_node_id.store(
            Self::pack_cc_mapping(sequence_number, node_id),
            Ordering::Relaxed,
        );
    }

    fn node_changed(&self) -> PaintPropertyChangeType {
        match self.changed.load(Ordering::Relaxed) {
            0 => PaintPropertyChangeType::Unchanged,
            1 => PaintPropertyChangeType::ChangedOnlyCompositedValues,
            2 => PaintPropertyChangeType::ChangedOnlySimpleValues,
            3 => PaintPropertyChangeType::ChangedOnlyValues,
            4 => PaintPropertyChangeType::NodeAddedOrRemoved,
            _ => unreachable!("PaintPropertyChangeType is a bounded enum"),
        }
    }

    fn add_changed(&self, changed: PaintPropertyChangeType) {
        self.changed.fetch_max(changed as u8, Ordering::Relaxed);
    }

    fn clear_changed(&self, sequence_number: i32) {
        debug_assert_ne!(
            self.changed_sequence_number.load(Ordering::Relaxed),
            sequence_number
        );
        self.changed
            .store(PaintPropertyChangeType::Unchanged as u8, Ordering::Relaxed);
        self.changed_sequence_number
            .store(sequence_number, Ordering::Relaxed);
    }
}

#[derive(Clone, Debug)]
pub struct PaintPropertyNodeLifecycle {
    pub identity: Arc<PaintPropertyNodeIdentity>,
    pub revision: u64,
    pub changed: PaintPropertyChangeType,
}
impl Default for PaintPropertyNodeLifecycle {
    fn default() -> Self {
        Self {
            identity: Arc::new(PaintPropertyNodeIdentity {
                cc_sequence_and_node_id: AtomicU64::new(
                    PaintPropertyNodeIdentity::pack_cc_mapping(
                        0,
                        PaintPropertyNodeIdentity::INVALID_CC_NODE_ID,
                    ),
                ),
                changed: AtomicU8::new(PaintPropertyChangeType::NodeAddedOrRemoved as u8),
                changed_sequence_number: AtomicI32::new(0),
                _private: (),
            }),
            revision: 0,
            changed: PaintPropertyChangeType::NodeAddedOrRemoved,
        }
    }
}
impl PartialEq for PaintPropertyNodeLifecycle {
    fn eq(&self, other: &Self) -> bool {
        self.same_node(other) && self.revision == other.revision
    }
}
impl PaintPropertyNodeLifecycle {
    pub fn same_node(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.identity, &other.identity)
    }
    pub fn cc_node_id(&self, sequence_number: i32) -> i32 {
        self.identity.cc_node_id(sequence_number)
    }
    pub fn set_cc_node_id(&self, sequence_number: i32, node_id: i32) {
        self.identity.set_cc_node_id(sequence_number, node_id);
    }
    fn root() -> Self {
        let result = Self {
            changed: PaintPropertyChangeType::Unchanged,
            ..Self::default()
        };
        result
            .identity
            .changed
            .store(PaintPropertyChangeType::Unchanged as u8, Ordering::Relaxed);
        result
    }
    fn updated(&self, changed: PaintPropertyChangeType) -> Self {
        self.identity.add_changed(changed);
        Self {
            identity: self.identity.clone(),
            revision: self.revision + u64::from(changed != PaintPropertyChangeType::Unchanged),
            changed,
        }
    }

    pub fn node_changed(&self) -> PaintPropertyChangeType {
        self.identity.node_changed()
    }
    pub fn changed_sequence_number(&self) -> i32 {
        self.identity
            .changed_sequence_number
            .load(Ordering::Relaxed)
    }
    pub fn clear_changed(&self, sequence_number: i32) {
        self.identity.clear_changed(sequence_number);
    }
    /// Source: paint_property_node.h:82-85. A successful direct compositor
    /// update downgrades only the exact simple-values marker.
    pub fn CompositorSimpleValuesUpdated(&self) {
        let _ = self.identity.changed.compare_exchange(
            PaintPropertyChangeType::ChangedOnlySimpleValues as u8,
            PaintPropertyChangeType::ChangedOnlyCompositedValues as u8,
            Ordering::Relaxed,
            Ordering::Relaxed,
        );
    }
}

/// Source: paint_property_node.cc:75-87. Rust production nodes are unaliased;
/// alias support is tracked separately from this common diagnostic projection.
pub fn PaintPropertyNodeToJSON<T>(
    parent: Option<&Arc<T>>,
    lifecycle: &PaintPropertyNodeLifecycle,
) -> serde_json::Map<String, serde_json::Value> {
    let mut json = serde_json::Map::new();
    if let Some(parent) = parent {
        json.insert(
            "parent".into(),
            serde_json::Value::String(format!("{:p}", Arc::as_ptr(parent))),
        );
    }
    let changed = lifecycle.node_changed();
    if changed != PaintPropertyChangeType::Unchanged {
        json.insert(
            "changed".into(),
            serde_json::Value::String(changed.AsString().into()),
        );
    }
    json
}

/// Source: paint_property_node.cc:66-73. Concrete virtual ToJSON output is
/// serialized directly; Rust has no DCHECK-only debug-name prefix.
pub fn PaintPropertyNodeToString(json: &serde_json::Value) -> String {
    json.to_string()
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaintBackfaceVisibility {
    #[default]
    Inherited,
    Visible,
    Hidden,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TransformPaintPropertyNode {
    /// Source: transform_paint_property_node.h:110-134. These remain source
    /// state fields even when a feature does not populate them on this node.
    pub compositor: TransformCompositorState,
    /// Source: transform_paint_property_node.h:472-485. Owns only the lazy
    /// source cache allocation; cloning a node starts a new empty cache slot.
    pub geometry_mapper_cache: crate::opaque_paint_cache_slot::OpaquePaintCacheSlot,
    pub lifecycle: PaintPropertyNodeLifecycle,
    pub id: u64,
    pub parent: Option<Arc<Self>>,
    pub matrix: TransformMatrix,
    // The standalone geometry mapper has already folded transform-origin into
    // the matrix. Keeping origin zero preserves that resolved transformation.
    pub origin: [f64; 3],
    pub flattens_inherited_transform: bool,
    pub rendering_context_id: u64,
    pub backface_visibility: PaintBackfaceVisibility,
    pub scroll: Option<Arc<ScrollPaintPropertyNode>>,
    /// Source: transform_paint_property_node.h:221-223. A fixed/overscroll
    /// paint-offset node can select a non-ancestor source scroll translation.
    pub scroll_parent_scroll_translation: Option<Arc<TransformPaintPropertyNode>>,
    /// Layout's immutable sticky geometry. The compositor resolves its scroll
    /// and ancestor identities through the property chain when applying an
    /// impl-side scroll delta.
    pub sticky_constraint: Option<StickyPositionPaintData>,
    pub direct_compositing_reasons: foundation::CompositingReasons,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TransformCompositorState {
    pub sticky_constraint: Option<PaintCompositorStickyConstraint>,
    pub compositor_element_id: foundation::CompositorElementId,
    pub visible_frame_element_id: foundation::CompositorElementId,
    pub in_subtree_of_page_scale: bool,
    pub animation_is_axis_aligned: bool,
    pub is_frame_paint_offset_translation: bool,
    pub is_for_svg_child: bool,
    pub anchor_position_scroll_data: Option<PaintAnchorPositionScrollData>,
}
impl Default for TransformCompositorState {
    fn default() -> Self {
        Self {
            sticky_constraint: None,
            compositor_element_id: Default::default(),
            visible_frame_element_id: Default::default(),
            in_subtree_of_page_scale: true,
            animation_is_axis_aligned: false,
            is_frame_paint_offset_translation: false,
            is_for_svg_child: false,
            anchor_position_scroll_data: None,
        }
    }
}

/// Source: transform_paint_property_node.h:101-105.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TransformAnimationState {
    pub is_running_animation_on_compositor: bool,
}

impl TransformPaintPropertyNode {
    fn ResolvedMatrix(&self) -> foundation::gfx::Transform {
        foundation::gfx::Transform::ColMajor(&self.matrix.values)
    }

    // Source: transform_paint_property_node.h:187-250,293-429. Source-owned
    // predicates stay here; compositor adapters project their results.
    pub fn IsIdentityOr2dTranslation(&self) -> bool {
        let matrix = self.ResolvedMatrix();
        matrix.IsIdentityOrTranslation() && matrix.GetColMajor()[14] == 0.0
    }
    pub fn IsIdentity(&self) -> bool {
        self.ResolvedMatrix().IsIdentity()
    }
    pub fn Get2dTranslation(&self) -> [f32; 2] {
        debug_assert!(self.IsIdentityOr2dTranslation());
        let translation = self.ResolvedMatrix().To2dTranslation();
        [translation.x(), translation.y()]
    }
    pub fn Matrix(&self) -> &TransformMatrix {
        &self.matrix
    }
    pub fn MatrixWithOriginApplied(&self) -> foundation::gfx::Transform {
        let mut result = self.ResolvedMatrix();
        result.ApplyTransformOrigin(
            self.origin[0] as f32,
            self.origin[1] as f32,
            self.origin[2] as f32,
        );
        result
    }
    pub fn Origin(&self) -> [f64; 3] {
        self.origin
    }
    pub fn ScrollNode(&self) -> Option<&Arc<ScrollPaintPropertyNode>> {
        self.scroll.as_ref()
    }
    pub fn ScrollParentScrollTranslation(&self) -> Option<&Arc<TransformPaintPropertyNode>> {
        self.scroll_parent_scroll_translation.as_ref()
    }
    pub fn IsAffectedByOuterViewportBoundsDelta(&self) -> bool {
        self.direct_compositing_reasons
            .Has(foundation::CompositingReason::kAffectedByOuterViewportBoundsDelta)
    }
    pub fn IsAffectedBySafeAreaBottom(&self) -> bool {
        self.direct_compositing_reasons
            .Has(foundation::CompositingReason::kAffectedBySafeAreaBottom)
    }
    pub fn IsInSubtreeOfPageScale(&self) -> bool {
        self.compositor.in_subtree_of_page_scale
    }
    pub fn GetStickyConstraint(&self) -> Option<&PaintCompositorStickyConstraint> {
        self.compositor.sticky_constraint.as_ref()
    }
    pub fn GetAnchorPositionScrollData(&self) -> Option<&PaintAnchorPositionScrollData> {
        self.compositor.anchor_position_scroll_data.as_ref()
    }
    pub fn FlattensInheritedTransform(&self) -> bool {
        self.flattens_inherited_transform
    }
    pub fn HasDirectCompositingReasons(&self) -> bool {
        !self.direct_compositing_reasons.IsEmpty()
    }
    pub fn HasDirectCompositingReasonsOtherThan3dTransform(&self) -> bool {
        !self
            .direct_compositing_reasons
            .Difference(foundation::CompositingReasons::FromReasons(&[
                foundation::CompositingReason::k3DTransform,
                foundation::CompositingReason::k3DScale,
                foundation::CompositingReason::k3DRotate,
                foundation::CompositingReason::k3DTranslate,
                foundation::CompositingReason::kTrivial3DTransform,
            ]))
            .IsEmpty()
    }
    pub fn HasActiveTransformAnimation(&self) -> bool {
        self.direct_compositing_reasons.HasAny(&[
            foundation::CompositingReason::kActiveTransformAnimation,
            foundation::CompositingReason::kActiveScaleAnimation,
            foundation::CompositingReason::kActiveRotateAnimation,
            foundation::CompositingReason::kActiveTranslateAnimation,
        ])
    }
    pub fn RequiresCompositingForFixedPosition(&self) -> bool {
        self.direct_compositing_reasons
            .Has(foundation::CompositingReason::kFixedPosition)
    }
    pub fn RequiresCompositingForFixedPositionOnly(&self) -> bool {
        self.RequiresCompositingForFixedPosition()
            && self
                .direct_compositing_reasons
                .Difference(foundation::CompositingReasonCombos::kFixedPositionReasons)
                .IsEmpty()
    }
    pub fn RequiresCompositingForFixedToViewport(&self) -> bool {
        self.direct_compositing_reasons
            .Has(foundation::CompositingReason::kUndoOverscroll)
    }
    pub fn RequiresCompositingForStickyPosition(&self) -> bool {
        self.direct_compositing_reasons
            .Has(foundation::CompositingReason::kStickyPosition)
    }
    pub fn RequiresCompositingForStickyPositionOnly(&self) -> bool {
        self.direct_compositing_reasons
            == foundation::CompositingReasons::FromReasons(&[
                foundation::CompositingReason::kStickyPosition,
            ])
    }
    pub fn RequiresCompositingForAnchorPosition(&self) -> bool {
        self.direct_compositing_reasons
            .Has(foundation::CompositingReason::kAnchorPosition)
    }
    pub fn DirectCompositingReasonsForDebugging(&self) -> foundation::CompositingReasons {
        self.direct_compositing_reasons
    }
    pub fn TransformAnimationIsAxisAligned(&self) -> bool {
        self.compositor.animation_is_axis_aligned
    }
    pub fn RequiresCompositingForRootScroller(&self) -> bool {
        self.direct_compositing_reasons
            .Has(foundation::CompositingReason::kRootScroller)
    }
    pub fn RequiresCompositingForWillChangeTransform(&self) -> bool {
        self.direct_compositing_reasons.HasAny(&[
            foundation::CompositingReason::kWillChangeTransform,
            foundation::CompositingReason::kWillChangeScale,
            foundation::CompositingReason::kWillChangeRotate,
            foundation::CompositingReason::kWillChangeTranslate,
        ])
    }
    pub fn UsesCompositedScrolling(&self) -> bool {
        self.direct_compositing_reasons
            .Has(foundation::CompositingReason::kOverflowScrolling)
    }
    pub fn RequiresCullRectExpansion(&self) -> bool {
        foundation::CompositingReasonCombos::kRequiresCullRectExpansion
            .iter_reasons()
            .any(|reason| self.direct_compositing_reasons.Has(reason))
    }
    pub fn GetCompositorElementId(&self) -> foundation::CompositorElementId {
        self.compositor.compositor_element_id
    }
    pub fn GetVisibleFrameElementId(&self) -> foundation::CompositorElementId {
        self.compositor.visible_frame_element_id
    }
    pub fn IsFramePaintOffsetTranslation(&self) -> bool {
        self.compositor.is_frame_paint_offset_translation
    }
    pub fn DelegatesToParentForBackface(&self) -> bool {
        self.backface_visibility == PaintBackfaceVisibility::Inherited
    }
    pub fn RenderingContextId(&self) -> u64 {
        self.rendering_context_id
    }
    pub fn HasRenderingContext(&self) -> bool {
        self.rendering_context_id != 0
    }
    pub fn IsForSVGChild(&self) -> bool {
        self.compositor.is_for_svg_child
    }

    /// Source: transform_paint_property_node.h:173-185.
    pub fn Update(
        &mut self,
        parent: Arc<TransformPaintPropertyNode>,
        mut state: TransformPaintPropertyNode,
        animation_state: TransformAnimationState,
    ) -> PaintPropertyChangeType {
        state.parent = Some(parent);
        let change = transform_state_change_with_animation(self, &state, animation_state);
        let auxiliary_changed = self.sticky_constraint != state.sticky_constraint;
        if change != PaintPropertyChangeType::Unchanged || auxiliary_changed {
            let lifecycle = if change == PaintPropertyChangeType::Unchanged {
                self.lifecycle.clone()
            } else {
                self.lifecycle.updated(change)
            };
            let id = self.id;
            state.geometry_mapper_cache = Default::default();
            *self = state;
            self.lifecycle = lifecycle;
            self.id = id;
        }
        change
    }

    /// Source: transform_paint_property_node.cc:124-134.
    pub fn DirectlyUpdateTransformAndOrigin(
        &mut self,
        matrix: TransformMatrix,
        origin: [f64; 3],
        animation_state: TransformAnimationState,
    ) -> PaintPropertyChangeType {
        let mut state = self.clone();
        state.matrix = matrix;
        state.origin = origin;
        let change = compute_transform_change(self, &state, animation_state);
        self.matrix = state.matrix;
        self.origin = state.origin;
        if change != PaintPropertyChangeType::Unchanged {
            self.lifecycle = self.lifecycle.updated(change);
            self.geometry_mapper_cache = Default::default();
        }
        change
    }

    /// Source: transform_paint_property_node.cc:151-164. Supported producer
    /// nodes are unaliased; associated Scroll changes are cleared recursively.
    pub fn ClearChangedToRoot(&self, sequence_number: i32) {
        let mut current = Some(self);
        while let Some(node) = current {
            if node.lifecycle.changed_sequence_number() == sequence_number {
                break;
            }
            node.lifecycle.clear_changed(sequence_number);
            if let Some(scroll) = node.ScrollNode() {
                scroll.ClearChangedToRoot(sequence_number);
            }
            current = node.parent.as_deref();
        }
    }

    /// Source: transform_paint_property_node.cc:166-177.
    pub fn CanMergeForFixedPosition(&self, other: &Self) -> bool {
        self.direct_compositing_reasons == other.direct_compositing_reasons
            && self.RequiresCompositingForFixedPositionOnly()
            && other.RequiresCompositingForFixedPositionOnly()
            && match (
                self.ScrollParentScrollTranslation(),
                other.ScrollParentScrollTranslation(),
            ) {
                (None, None) => true,
                (Some(a), Some(b)) => a.lifecycle.same_node(&b.lifecycle),
                _ => false,
            }
            && match (&self.parent, &other.parent) {
                (None, None) => true,
                (Some(a), Some(b)) => a.lifecycle.same_node(&b.lifecycle),
                _ => false,
            }
    }

    /// Source: transform_paint_property_node.cc:221-267. The non-2d matrix
    /// diagnostic uses the source row-major matrix spelling because the full
    /// gfx decomposition formatter is not part of paint_model.
    pub fn ToJSON(&self) -> serde_json::Value {
        let mut json = PaintPropertyNodeToJSON(self.parent.as_ref(), &self.lifecycle);
        if self.IsIdentityOr2dTranslation() {
            let translation = self.Get2dTranslation();
            if translation != [0.0, 0.0] {
                json.insert(
                    "translation2d".into(),
                    serde_json::Value::String(format!("{},{}", translation[0], translation[1])),
                );
            }
        } else {
            let values = self.ResolvedMatrix().GetColMajor();
            let rows = (0..4)
                .map(|row| {
                    (0..4)
                        .map(|column| values[column * 4 + row].to_string())
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .collect::<Vec<_>>()
                .join(", ");
            json.insert("matrix".into(), serde_json::Value::String(rows));
            json.insert(
                "origin".into(),
                serde_json::Value::String(format!(
                    "{},{},{}",
                    self.origin[0], self.origin[1], self.origin[2]
                )),
            );
        }
        if !self.FlattensInheritedTransform() {
            json.insert(
                "flattensInheritedTransform".into(),
                serde_json::Value::Bool(false),
            );
        }
        if !self.IsInSubtreeOfPageScale() {
            json.insert(
                "in_subtree_of_page_scale".into(),
                serde_json::Value::Bool(false),
            );
        }
        if self.backface_visibility != PaintBackfaceVisibility::Inherited {
            json.insert(
                "backface".into(),
                serde_json::Value::String(
                    if self.backface_visibility == PaintBackfaceVisibility::Visible {
                        "visible"
                    } else {
                        "hidden"
                    }
                    .into(),
                ),
            );
        }
        if self.HasRenderingContext() {
            json.insert(
                "renderingContextId".into(),
                serde_json::Value::String(format!("{:x}", self.RenderingContextId())),
            );
        }
        if self.HasDirectCompositingReasons() {
            json.insert(
                "directCompositingReasons".into(),
                serde_json::Value::String(self.direct_compositing_reasons.to_string()),
            );
        }
        let compositor_id = self.GetCompositorElementId().GetInternalValue();
        if compositor_id != 0 {
            json.insert(
                "compositorElementId".into(),
                serde_json::Value::String(format!("({compositor_id})")),
            );
        }
        if let Some(scroll) = self.ScrollNode() {
            json.insert(
                "scroll".into(),
                serde_json::Value::String(format!("{:p}", Arc::as_ptr(scroll))),
            );
        }
        if let Some(scroll_parent) = self.ScrollParentScrollTranslation() {
            json.insert(
                "scroll_parent_scroll_translation".into(),
                serde_json::Value::String(format!("{:p}", Arc::as_ptr(scroll_parent))),
            );
        }
        serde_json::Value::Object(json)
    }
    pub fn ToString(&self) -> String {
        PaintPropertyNodeToString(&self.ToJSON())
    }
}
/// Source value: cc/trees/sticky_position_constraint.h:23-68. Paint owns the
/// builder's produced compositor constraint, distinct from Layout constraints.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PaintCompositorStickyConstraint {
    pub anchored: [bool; 4],
    pub offsets: [f32; 4],
    pub constraint_box_rect: PaintRect,
    pub sticky_box_rect: PaintRect,
    pub containing_block_rect: PaintRect,
    pub pixel_snap_offset: [f32; 2],
    pub x_scroll_ancestor: foundation::CompositorElementId,
    pub y_scroll_ancestor: foundation::CompositorElementId,
    pub nearest_shifting_sticky_box: foundation::CompositorElementId,
    pub nearest_shifting_containing_block: foundation::CompositorElementId,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PaintAnchorPositionScrollData {
    pub adjustment_container_ids: Vec<foundation::CompositorElementId>,
    pub accumulated_scroll_origin: [i32; 2],
    pub needs_scroll_adjustment_in_x: bool,
    pub needs_scroll_adjustment_in_y: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClipPaintPropertyNode {
    /// Source: clip_paint_property_node.h:249-258.
    pub geometry_mapper_cache: crate::opaque_paint_cache_slot::OpaquePaintCacheSlot,
    /// Source: clip_paint_property_node.h:88-125,188-217. Layout and paint
    /// clips are distinct source values, including compositor animation bounds.
    pub layout_clip_rects: ClipLayoutRects,
    pub lifecycle: PaintPropertyNodeLifecycle,
    pub id: u64,
    pub parent: Option<Arc<Self>>,
    pub local_transform_space: Arc<TransformPaintPropertyNode>,
    pub rect: Option<PaintRect>,
    pub radii: PaintCornerRadii,
    /// None is no path clip. A present source empty path clips everything.
    /// SVG children retain their own fill rules in the source shape owner.
    pub clip_path: Option<Arc<PaintClipPath>>,
    pub clip_path_error: Option<String>,
    /// UpdatePixelMovingFilterClipExpander retains the filter whose input
    /// bounds must be expanded; an infinite rect alone loses this relationship.
    pub pixel_moving_filter: Option<Arc<EffectPaintPropertyNode>>,
}

impl ClipPaintPropertyNode {
    /// Source: clip_paint_property_node.h:88-99,110-125. Normal source clip
    /// creation supplies equal precise/paint input until the builder supplies
    /// its unsnapped layout value or an explicit animation expansion.
    pub fn with_layout_clips(mut self) -> Self {
        self.layout_clip_rects =
            ClipLayoutRects::new(self.rect, self.radii.HasRadius(), None, None);
        self
    }
    pub fn with_layout_clip_values(
        mut self,
        precise: Option<PaintRect>,
        expanded: Option<PaintRect>,
        excluding_overlay: Option<PaintRect>,
    ) -> Self {
        self.layout_clip_rects =
            ClipLayoutRects::new(precise, self.radii.HasRadius(), expanded, excluding_overlay);
        self
    }
    pub fn clip_path_child_count(&self) -> usize {
        self.clip_path
            .as_ref()
            .map_or(0, |shape| shape.paths().len())
    }
    pub fn single_clip_path(&self) -> Option<&crate::paint_clip_path::PaintPath> {
        self.clip_path.as_ref()?.single_path().map(Arc::as_ref)
    }
    pub fn clip_path_verb_count(&self) -> usize {
        self.clip_path.as_ref().map_or(0, |shape| {
            shape.paths().iter().map(|p| p.verb_count()).sum()
        })
    }

    // Source: clip_paint_property_node.h:183-232. The node owns these source
    // queries; compositor adapters only convert their value types.
    pub fn LocalTransformSpace(&self) -> &Arc<TransformPaintPropertyNode> {
        &self.local_transform_space
    }
    pub fn PaintClipRect(&self) -> Option<PaintRect> {
        self.rect
    }
    pub fn PaintClipRadii(&self) -> PaintCornerRadii {
        self.radii
    }
    pub fn LayoutClipRect(&self) -> ClipLayoutRect {
        self.layout_clip_rects.expanded
    }
    pub fn PreciseLayoutClipRect(&self) -> ClipLayoutRect {
        self.layout_clip_rects.precise
    }
    pub fn IsForCompositeClipPathAnimation(&self) -> bool {
        self.layout_clip_rects.expanded != self.layout_clip_rects.precise
    }
    pub fn LayoutClipRectExcludingOverlayScrollbars(&self) -> ClipLayoutRect {
        self.layout_clip_rects
            .excluding_overlay
            .unwrap_or(self.layout_clip_rects.precise)
    }
    pub fn ClipPath(&self) -> Option<&Arc<PaintClipPath>> {
        self.clip_path.as_ref()
    }
    pub fn ClipPathEquals(&self, path: Option<&Arc<PaintClipPath>>) -> bool {
        self.clip_path.as_ref() == path
    }
    pub fn PixelMovingFilter(&self) -> Option<&Arc<EffectPaintPropertyNode>> {
        self.pixel_moving_filter.as_ref()
    }

    /// Source: clip_paint_property_node.h:163-178. The incoming node is the
    /// source `State`; resident identity stays on this node, and a changed
    /// revision starts with an empty GeometryMapper cache.
    pub fn Update(
        &mut self,
        parent: Arc<ClipPaintPropertyNode>,
        mut state: ClipPaintPropertyNode,
    ) -> PaintPropertyChangeType {
        state.parent = Some(parent);
        let change = clip_state_change(self, &state);
        if change != PaintPropertyChangeType::Unchanged {
            let lifecycle = self.lifecycle.updated(change);
            let id = self.id;
            state.geometry_mapper_cache = Default::default();
            *self = state;
            self.lifecycle = lifecycle;
            self.id = id;
        }
        change
    }

    /// Source: clip_paint_property_node.cc:70-81. Supported snapshots are
    /// unaliased; each clip also clears its local transform chain.
    pub fn ClearChangedToRoot(&self, sequence_number: i32) {
        let mut current = Some(self);
        while let Some(node) = current {
            if node.lifecycle.changed_sequence_number() == sequence_number {
                break;
            }
            node.lifecycle.clear_changed(sequence_number);
            node.LocalTransformSpace()
                .ClearChangedToRoot(sequence_number);
            current = node.parent.as_deref();
        }
    }

    /// Source: clip_paint_property_node.cc:83-114.
    pub fn ToJSON(&self) -> serde_json::Value {
        fn rect_string(rect: Option<PaintRect>) -> String {
            let rect = rect.unwrap_or_else(|| {
                let infinite = foundation::infinite_int_rect::InfiniteIntRect();
                PaintRect {
                    x: f64::from(infinite.x()),
                    y: f64::from(infinite.y()),
                    width: f64::from(infinite.width()),
                    height: f64::from(infinite.height()),
                }
            });
            format!("{},{} {}x{}", rect.x, rect.y, rect.width, rect.height)
        }

        let mut json = PaintPropertyNodeToJSON(self.parent.as_ref(), &self.lifecycle);
        json.insert(
            "localTransformSpace".into(),
            serde_json::Value::String(format!("{:p}", Arc::as_ptr(&self.local_transform_space))),
        );
        json.insert(
            "rect".into(),
            serde_json::Value::String(rect_string(self.PaintClipRect())),
        );
        if let Some(excluding) = self.layout_clip_rects.excluding_overlay {
            if excluding != self.layout_clip_rects.expanded {
                json.insert(
                    "rectExcludingOverlayScrollbars".into(),
                    serde_json::Value::String(rect_string(excluding.rect)),
                );
            }
        }
        if self.ClipPath().is_some() {
            json.insert("hasClipPath".into(), serde_json::Value::Bool(true));
        }
        if self.IsForCompositeClipPathAnimation() {
            json.insert(
                "isForCompositeClipPathAnimation".into(),
                serde_json::Value::Bool(true),
            );
            json.insert(
                "expandedLayoutClipRect".into(),
                serde_json::Value::String(rect_string(self.LayoutClipRect().rect)),
            );
            json.insert(
                "preciseLayoutClipRect".into(),
                serde_json::Value::String(rect_string(self.PreciseLayoutClipRect().rect)),
            );
        }
        if let Some(filter) = self.PixelMovingFilter() {
            json.insert(
                "pixelMovingFilter".into(),
                serde_json::Value::String(format!("{:p}", Arc::as_ptr(filter))),
            );
        }
        serde_json::Value::Object(json)
    }
    pub fn ToString(&self) -> String {
        PaintPropertyNodeToString(&self.ToJSON())
    }
}

/// Source: clip_paint_property_node.h:110-125,188-217. None is the source
/// infinite FloatClipRect sentinel; each optional overlay rect is unrounded.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ClipLayoutRect {
    pub rect: Option<PaintRect>,
    pub has_radius: bool,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ClipLayoutRects {
    pub precise: ClipLayoutRect,
    pub expanded: ClipLayoutRect,
    pub excluding_overlay: Option<ClipLayoutRect>,
}
impl ClipLayoutRects {
    pub fn new(
        precise: Option<PaintRect>,
        rounded: bool,
        expanded: Option<PaintRect>,
        excluding_overlay: Option<PaintRect>,
    ) -> Self {
        Self {
            precise: ClipLayoutRect {
                rect: precise,
                has_radius: rounded,
            },
            expanded: ClipLayoutRect {
                rect: expanded.or(precise),
                has_radius: rounded && expanded.is_none(),
            },
            excluding_overlay: excluding_overlay.map(|rect| ClipLayoutRect {
                rect: Some(rect),
                has_radius: false,
            }),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EffectPaintPropertyNode {
    pub compositor: EffectCompositorState,
    /// Source: effect_paint_property_node.h:97-105,261-269.
    pub backdrop_filter_bounds: Option<Arc<crate::paint_clip_path::PaintPath>>,
    /// Source: effect_paint_property_node.h:427-429; builder.cc:2263-2264,
    /// 2493-2498,2690-2691. The producer publishes inherited participation.
    pub self_or_ancestor_participates_in_view_transition: bool,
    pub lifecycle: PaintPropertyNodeLifecycle,
    pub id: u64,
    pub parent: Option<Arc<Self>>,
    pub local_transform_space: Arc<TransformPaintPropertyNode>,
    pub output_clip: Option<Arc<ClipPaintPropertyNode>>,
    /// Filter reference box in `local_transform_space`.  This is independent
    /// of `output_clip`: SVG filterUnits/primitiveUnits and the filter region
    /// are resolved from the owning border box, while the output clip belongs
    /// to the surrounding property tree.
    pub filter_reference_box: Option<PaintRect>,
    /// Chromium PaintLayer filter viewport mapped into local_transform_space.
    /// Reference filters resolve userSpaceOnUse percentages against this box,
    /// independently of visual-overflow reference geometry.
    pub filter_viewport: Option<PaintRect>,
    /// Origin of the filter's local user space in local_transform_space.
    /// This remains present when SVG foreignObject deliberately has no
    /// viewport override.
    pub filter_space_origin: Offset,
    /// SVG userSpaceOnUse values are raw resource coordinates. HTML/CSS
    /// owners map them through EffectiveZoom before the property transform;
    /// source SVG owners use one.
    pub filter_reference_scale: f64,
    pub opacity: f32,
    pub blend_mode: PaintBlendMode,
    pub filters: PropertyFilterOperations,
    /// Filters applied to pixels already present in the parent render target.
    /// They are never folded into the foreground `filters` chain.
    pub backdrop_filters: PropertyFilterOperations,
    pub backdrop_filter_has_reference: bool,
    /// Border-box backdrop output clip. It is separate from `output_clip` so
    /// foreground descendants keep their ordinary overflow semantics.
    pub backdrop_filter_clip: Option<Arc<ClipPaintPropertyNode>>,
    pub isolates_blending: bool,
    // Legacy standalone wrapper metadata; formal Mask nodes use is_mask.
    pub has_mask: bool,
    /// ObjectPaintProperties::Mask has internal SkBlendMode::kDstIn. This is
    /// independent of the CSS blend-mode input, which cannot express DstIn.
    pub is_mask: bool,
    pub direct_compositing_reasons: foundation::CompositingReasons,
}

/// Source: effect_paint_property_node.h:82-88. Callers performing a direct
/// compositor update can preserve a composited-only change; ordinary retained
/// PrePaint updates use the all-false source default.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EffectAnimationState {
    pub is_running_opacity_animation_on_compositor: bool,
    pub is_running_filter_animation_on_compositor: bool,
    pub is_running_backdrop_filter_animation_on_compositor: bool,
}

/// Source: effect_paint_property_node.h:90-105,146-176. Value fields belong to the
/// published node, not a compositor-side registry or inferred feature flags.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EffectCompositorState {
    /// Source: effect_paint_property_node.h:90-95,248-250. The producer's reference
    /// filter output rectangle; absent when this state owns no FilterInfo.
    pub filter_output_bounds: Option<PaintRect>,
    pub has_filter_info: bool,
    pub has_backdrop_filter_info: bool,
    pub compositor_element_id: foundation::CompositorElementId,
    pub backdrop_mask_element_id: foundation::CompositorElementId,
    pub view_transition_element_resource_id: Option<([u64; 2], u32, bool)>,
    pub restriction_target_id: [u64; 2],
    pub needs_effect_for_2d_scale_transform: bool,
    /// Source: effect_paint_property_node.h:174. Foreground filter taint is
    /// inherited through the effect tree independently of raster operations.
    pub is_in_tainted_subtree: bool,
    pub is_in_drawable_canvas_subtree: bool,
    pub canvas_child_state: Option<Arc<PaintCanvasChildState>>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PaintCanvasChildState {
    pub id: i32,
    pub paint_state: CanvasChildPaintState,
    pub content_state: PropertyTreeState,
}

/// Source: platform/graphics/canvas_child_paint_state.h:22-42. Geometry stays
/// in source units; image ids are the source signed PaintImage ids.
#[derive(Clone, Debug)]
pub struct CanvasChildPaintState {
    pub effective_zoom: f32,
    pub transform_origin: [f32; 3],
    pub box_size: [f32; 2],
    pub reference_box_offset: Offset,
    pub canvas_content_size: [f32; 2],
    pub canvas_device_pixel_content_box: [i32; 2],
    pub canvas_node_id: i32,
    pub canvas_child_node_id: i32,
    pub animated_image_frame_index_map: Option<Arc<BTreeMap<i32, usize>>>,
}
impl PartialEq for CanvasChildPaintState {
    fn eq(&self, other: &Self) -> bool {
        // Source: canvas_child_paint_state.cc:12-27. The child id is owned by
        // EffectPaintPropertyNode::CanvasChildState and is intentionally not
        // part of this value comparison.
        self.effective_zoom == other.effective_zoom
            && self.transform_origin == other.transform_origin
            && self.box_size == other.box_size
            && self.reference_box_offset == other.reference_box_offset
            && self.canvas_content_size == other.canvas_content_size
            && self.canvas_device_pixel_content_box == other.canvas_device_pixel_content_box
            && self.canvas_node_id == other.canvas_node_id
            && self.animated_image_frame_index_map == other.animated_image_frame_index_map
    }
}
impl Default for CanvasChildPaintState {
    fn default() -> Self {
        Self {
            effective_zoom: 1.0,
            transform_origin: [0.0; 3],
            box_size: [0.0; 2],
            reference_box_offset: Offset::default(),
            canvas_content_size: [0.0; 2],
            canvas_device_pixel_content_box: [0; 2],
            canvas_node_id: 0,
            canvas_child_node_id: 0,
            animated_image_frame_index_map: None,
        }
    }
}
impl CanvasChildPaintState {
    /// Source: canvas_child_paint_state.cc:64-78.
    pub fn GetCanvasGridScaleFactor(&self, canvas_size: [i32; 2]) -> [f32; 2] {
        if canvas_size != self.canvas_device_pixel_content_box
            && self.canvas_content_size[0] > 0.0
            && self.canvas_content_size[1] > 0.0
        {
            [
                canvas_size[0] as f32 / self.canvas_content_size[0],
                canvas_size[1] as f32 / self.canvas_content_size[1],
            ]
        } else {
            [1.0, 1.0]
        }
    }

    /// Source: canvas_child_paint_state.cc:29-62.
    pub fn GetElementTransform(
        &self,
        canvas_size: [i32; 2],
        draw_transform: &foundation::gfx::Transform,
        element_canvas_transform_enabled: bool,
    ) -> foundation::gfx::Transform {
        fn scale(x: f32, y: f32) -> foundation::gfx::Transform {
            let mut values = TransformMatrix::default().values;
            values[0] = f64::from(x);
            values[5] = f64::from(y);
            foundation::gfx::Transform::ColMajor(&values)
        }

        let physical_to_canvas_grid = self.GetCanvasGridScaleFactor(canvas_size);
        let physical_to_css = 1.0 / self.effective_zoom;
        let canvas_grid_to_css_x = physical_to_css / physical_to_canvas_grid[0];
        let canvas_grid_to_css_y = physical_to_css / physical_to_canvas_grid[1];

        let mut css_transform = scale(canvas_grid_to_css_x, canvas_grid_to_css_y);
        css_transform.PreConcat(draw_transform);
        css_transform.PreConcat(&scale(
            1.0 / canvas_grid_to_css_x,
            1.0 / canvas_grid_to_css_y,
        ));
        if element_canvas_transform_enabled {
            return css_transform;
        }

        let mut result = foundation::gfx::Transform::default();
        result.Translate3d(
            -self.transform_origin[0],
            -self.transform_origin[1],
            -self.transform_origin[2],
        );
        result.PreConcat(&css_transform);
        result.Translate3d(
            self.transform_origin[0],
            self.transform_origin[1],
            self.transform_origin[2],
        );
        result
    }
}

impl EffectPaintPropertyNode {
    /// Current producer publishes FilterInfo only at its actual nonempty filter
    /// construction sites; retain this ownership fact on the published node.
    pub fn with_filter_presence(mut self) -> Self {
        self.compositor.has_filter_info = !self.filters.is_empty();
        self.compositor.has_backdrop_filter_info = !self.backdrop_filters.is_empty();
        self
    }
    pub fn with_inherited_view_transition(mut self) -> Self {
        self.self_or_ancestor_participates_in_view_transition |= self
            .parent
            .as_ref()
            .is_some_and(|parent| parent.self_or_ancestor_participates_in_view_transition);
        self
    }
    /// Used by the producer creating a participating view-transition effect;
    /// descendants subsequently inherit this same source-owned value.
    pub fn with_view_transition_participation(mut self, participates: bool) -> Self {
        self.self_or_ancestor_participates_in_view_transition = participates;
        self.with_inherited_view_transition()
    }

    pub fn LocalTransformSpace(&self) -> &Arc<TransformPaintPropertyNode> {
        &self.local_transform_space
    }
    pub fn OutputClip(&self) -> Option<&Arc<ClipPaintPropertyNode>> {
        self.output_clip.as_ref()
    }

    // Source: effect_paint_property_node.h:236-433. These predicates belong to
    // the Paint property node. Compositor adapters project their results; they
    // must not reconstruct a second, subtly different effect-state model.
    pub fn Filter(&self) -> Option<&crate::cc::paint::filter_operations::FilterOperations> {
        self.compositor.has_filter_info.then(|| {
            self.filters
                .canonical()
                .expect("published FilterInfo must own canonical operations")
        })
    }

    pub fn BackdropFilter(&self) -> Option<&crate::cc::paint::filter_operations::FilterOperations> {
        self.compositor.has_backdrop_filter_info.then(|| {
            self.backdrop_filters
                .canonical()
                .expect("published BackdropFilterInfo must own canonical operations")
        })
    }

    pub fn HasReferenceFilter(&self) -> bool {
        self.Filter()
            .is_some_and(|operations| operations.HasReferenceFilter())
            || self
                .BackdropFilter()
                .is_some_and(|operations| operations.HasReferenceFilter())
    }

    pub fn HasFilterThatMovesPixels(&self) -> bool {
        self.Filter()
            .is_some_and(|operations| operations.HasFilterThatMovesPixels())
    }

    pub fn HasRealEffects(&self) -> bool {
        self.opacity != 1.0
            || self.blend_mode != PaintBlendMode::kNormal
            || self.Filter().is_some()
            || self.BackdropFilter().is_some()
    }

    pub fn IsOpacityOnly(&self) -> bool {
        self.blend_mode == PaintBlendMode::kNormal
            && self.Filter().is_none()
            && self.BackdropFilter().is_none()
    }

    pub fn HasDirectCompositingReasons(&self) -> bool {
        !self.direct_compositing_reasons.IsEmpty()
    }

    pub fn RequiresCompositingForUnboundedElement(&self) -> bool {
        self.direct_compositing_reasons
            .Has(foundation::CompositingReason::kUnboundedElement)
    }

    pub fn RequiresCompositingForBackdropFilterMask(&self) -> bool {
        self.direct_compositing_reasons
            .Has(foundation::CompositingReason::kBackdropFilterMask)
    }

    pub fn RequiresCompositingForCanvasChild(&self) -> bool {
        self.direct_compositing_reasons
            .Has(foundation::CompositingReason::kCanvasChild)
    }

    pub fn FlattensAtLeafOf3DScene(&self) -> bool {
        self.direct_compositing_reasons
            .Has(foundation::CompositingReason::kTransform3DSceneLeaf)
    }

    pub fn HasActiveOpacityAnimation(&self) -> bool {
        self.direct_compositing_reasons
            .Has(foundation::CompositingReason::kActiveOpacityAnimation)
    }

    pub fn HasActiveFilterAnimation(&self) -> bool {
        self.direct_compositing_reasons
            .Has(foundation::CompositingReason::kActiveFilterAnimation)
    }

    pub fn HasActiveBackdropFilterAnimation(&self) -> bool {
        self.direct_compositing_reasons
            .Has(foundation::CompositingReason::kActiveBackdropFilterAnimation)
    }

    pub fn RequiresCompositingForWillChangeOpacity(&self) -> bool {
        self.direct_compositing_reasons
            .Has(foundation::CompositingReason::kWillChangeOpacity)
    }

    pub fn RequiresCompositingForWillChangeFilter(&self) -> bool {
        self.direct_compositing_reasons
            .Has(foundation::CompositingReason::kWillChangeFilter)
    }

    pub fn RequiresCompositingForWillChangeBackdropFilter(&self) -> bool {
        self.direct_compositing_reasons
            .Has(foundation::CompositingReason::kWillChangeBackdropFilter)
    }

    pub fn MayHaveOpacity(&self) -> bool {
        self.opacity != 1.0
            || self.HasActiveOpacityAnimation()
            || self.RequiresCompositingForWillChangeOpacity()
    }

    pub fn MayHaveFilter(&self) -> bool {
        self.Filter().is_some()
            || self.HasActiveFilterAnimation()
            || self.RequiresCompositingForWillChangeFilter()
    }

    pub fn MayHaveBackdropFilter(&self) -> bool {
        self.BackdropFilter().is_some()
            || self.HasActiveBackdropFilterAnimation()
            || self.RequiresCompositingForWillChangeBackdropFilter()
    }

    pub fn NeedsPixelMovingFilterClipExpander(&self) -> bool {
        self.HasActiveFilterAnimation()
            || self.RequiresCompositingForWillChangeFilter()
            || self.HasFilterThatMovesPixels()
    }

    pub fn MayHaveBackdropEffect(&self) -> bool {
        self.blend_mode != PaintBlendMode::kNormal || self.MayHaveBackdropFilter()
    }

    pub fn HasValidViewTransitionElementResourceId(&self) -> bool {
        self.compositor
            .view_transition_element_resource_id
            .is_some_and(|(_, local_id, _)| local_id != 0)
    }

    pub fn HasElementCaptureId(&self) -> bool {
        self.compositor.restriction_target_id != [0, 0]
    }

    pub fn HasCanvasChildState(&self) -> bool {
        self.compositor
            .canvas_child_state
            .as_ref()
            .is_some_and(|state| state.id != 0)
    }

    pub fn CanvasChildId(&self) -> i32 {
        self.compositor
            .canvas_child_state
            .as_ref()
            .map_or(0, |state| state.id)
    }

    pub fn canvas_child_paint_state(&self) -> Option<&CanvasChildPaintState> {
        self.compositor
            .canvas_child_state
            .as_ref()
            .map(|state| &state.paint_state)
    }

    pub fn CanvasChildContentTransform(&self) -> &Arc<TransformPaintPropertyNode> {
        &self
            .compositor
            .canvas_child_state
            .as_ref()
            .expect("source HasCanvasChildState")
            .content_state
            .transform
    }

    pub fn CanvasChildContentClip(&self) -> &Arc<ClipPaintPropertyNode> {
        &self
            .compositor
            .canvas_child_state
            .as_ref()
            .expect("source HasCanvasChildState")
            .content_state
            .clip
    }

    pub fn CanvasChildContentEffect(&self) -> &Arc<EffectPaintPropertyNode> {
        &self
            .compositor
            .canvas_child_state
            .as_ref()
            .expect("source HasCanvasChildState")
            .content_state
            .effect
    }

    pub fn DrawsContent(&self) -> bool {
        self.MayHaveFilter()
            || self.MayHaveBackdropEffect()
            || self.HasValidViewTransitionElementResourceId()
            || self.HasElementCaptureId()
    }

    pub fn IsInTaintedSubtree(&self) -> bool {
        self.compositor.is_in_tainted_subtree
    }

    pub fn IsInDrawableCanvasSubtree(&self) -> bool {
        self.compositor.is_in_drawable_canvas_subtree
    }

    pub fn NeedsEffectFor2DScaleTransform(&self) -> bool {
        self.compositor.needs_effect_for_2d_scale_transform
    }

    /// Source: effect_paint_property_node.cc:112-123.
    pub fn IsOpacityChangeSimple(
        opacity: f32,
        new_opacity: f32,
        direct_compositing_reasons: foundation::CompositingReasons,
        new_direct_compositing_reasons: foundation::CompositingReasons,
    ) -> bool {
        opacity != new_opacity
            && ((opacity != 1.0 && new_opacity != 1.0)
                || (direct_compositing_reasons
                    .Has(foundation::CompositingReason::kActiveOpacityAnimation)
                    && new_direct_compositing_reasons
                        .Has(foundation::CompositingReason::kActiveOpacityAnimation)))
    }

    /// Source: effect_paint_property_node.cc:195-217.
    pub fn ComputeOpacityChange(
        &self,
        new_opacity: f32,
        animation_state: EffectAnimationState,
    ) -> PaintPropertyChangeType {
        use PaintPropertyChangeType::*;
        let opacity_changed = self.opacity != new_opacity;
        let opacity_change_is_simple = Self::IsOpacityChangeSimple(
            self.opacity,
            new_opacity,
            self.direct_compositing_reasons,
            self.direct_compositing_reasons,
        );
        if opacity_changed && !opacity_change_is_simple {
            debug_assert!(!animation_state.is_running_opacity_animation_on_compositor);
            return ChangedOnlyValues;
        }
        if opacity_change_is_simple && !animation_state.is_running_opacity_animation_on_compositor {
            return ChangedOnlySimpleValues;
        }
        if opacity_changed {
            return ChangedOnlyCompositedValues;
        }
        Unchanged
    }

    /// Source: effect_paint_property_node.h:216-227. The incoming value is the
    /// source `State`; resident identity and dump-local id stay on this node.
    pub fn Update(
        &mut self,
        parent: Arc<EffectPaintPropertyNode>,
        mut state: EffectPaintPropertyNode,
        animation_state: EffectAnimationState,
    ) -> PaintPropertyChangeType {
        state.parent = Some(parent);
        let change = effect_state_change_with_animation(self, &state, animation_state);
        if change != PaintPropertyChangeType::Unchanged {
            let lifecycle = self.lifecycle.updated(change);
            let id = self.id;
            *self = state;
            self.lifecycle = lifecycle;
            self.id = id;
        }
        change
    }

    /// Source: effect_paint_property_node.cc:219-227. Rust publishes a new
    /// immutable value revision while retaining the same resident node identity.
    pub fn DirectlyUpdateOpacity(
        &mut self,
        opacity: f32,
        animation_state: EffectAnimationState,
    ) -> PaintPropertyChangeType {
        let change = self.ComputeOpacityChange(opacity, animation_state);
        self.opacity = opacity;
        if change != PaintPropertyChangeType::Unchanged {
            self.lifecycle = self.lifecycle.updated(change);
        }
        change
    }

    /// Source: effect_paint_property_node.cc:181-193. Supported snapshots are
    /// unaliased; local transform and output clip dependencies are included.
    pub fn ClearChangedToRoot(&self, sequence_number: i32) {
        let mut current = Some(self);
        while let Some(node) = current {
            if node.lifecycle.changed_sequence_number() == sequence_number {
                break;
            }
            node.lifecycle.clear_changed(sequence_number);
            node.LocalTransformSpace()
                .ClearChangedToRoot(sequence_number);
            if let Some(clip) = node.OutputClip() {
                clip.ClearChangedToRoot(sequence_number);
            }
            current = node.parent.as_deref();
        }
    }

    /// Source: effect_paint_property_node.cc:257-282. This is the source Paint
    /// node diagnostic object consumed by the compositor debug projection.
    pub fn ToJSON(&self) -> serde_json::Value {
        use serde_json::Value;

        fn blend_mode_name(mode: PaintBlendMode) -> &'static str {
            match mode {
                PaintBlendMode::kNormal => "SrcOver",
                PaintBlendMode::kMultiply => "Multiply",
                PaintBlendMode::kScreen => "Screen",
                PaintBlendMode::kOverlay => "Overlay",
                PaintBlendMode::kDarken => "Darken",
                PaintBlendMode::kLighten => "Lighten",
                PaintBlendMode::kColorDodge => "ColorDodge",
                PaintBlendMode::kColorBurn => "ColorBurn",
                PaintBlendMode::kHardLight => "HardLight",
                PaintBlendMode::kSoftLight => "SoftLight",
                PaintBlendMode::kDifference => "Difference",
                PaintBlendMode::kExclusion => "Exclusion",
                PaintBlendMode::kHue => "Hue",
                PaintBlendMode::kSaturation => "Saturation",
                PaintBlendMode::kColor => "Color",
                PaintBlendMode::kLuminosity => "Luminosity",
                PaintBlendMode::kPlusLighter => "Plus",
            }
        }

        let mut json = PaintPropertyNodeToJSON(self.parent.as_ref(), &self.lifecycle);
        json.insert(
            "localTransformSpace".into(),
            Value::String(format!("{:p}", Arc::as_ptr(&self.local_transform_space))),
        );
        json.insert(
            "outputClip".into(),
            Value::String(
                self.output_clip
                    .as_ref()
                    .map_or_else(|| "0x0".into(), |clip| format!("{:p}", Arc::as_ptr(clip))),
            ),
        );
        if let Some(filter) = self.Filter() {
            json.insert("filter".into(), Value::String(filter.ToString()));
        }
        if let Some(filter) = self.BackdropFilter() {
            json.insert("backdrop_filter".into(), Value::String(filter.ToString()));
        }
        if self.opacity != 1.0 {
            json.insert("opacity".into(), Value::from(self.opacity));
        }
        if self.blend_mode != PaintBlendMode::kNormal {
            json.insert(
                "blendMode".into(),
                Value::String(blend_mode_name(self.blend_mode).into()),
            );
        }
        if !self.direct_compositing_reasons.IsEmpty() {
            json.insert(
                "directCompositingReasons".into(),
                Value::String(self.direct_compositing_reasons.to_string()),
            );
        }
        let compositor_id = self.compositor.compositor_element_id.GetInternalValue();
        if compositor_id != 0 {
            json.insert(
                "compositorElementId".into(),
                Value::String(format!("({compositor_id})")),
            );
        }
        if self.compositor.is_in_drawable_canvas_subtree {
            json.insert("is_in_drawable_canvas_subtree".into(), Value::Bool(true));
        }
        Value::Object(json)
    }

    pub fn ToString(&self) -> String {
        PaintPropertyNodeToString(&self.ToJSON())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScrollPaintPropertyNode {
    pub compositor: ScrollCompositorState,
    pub lifecycle: PaintPropertyNodeLifecycle,
    pub id: u64,
    /// Stable source owner identity used to connect Layout's sticky scroll
    /// ancestor to this Paint property node without exposing either owner.
    pub owner_client_id: u64,
    pub parent: Option<Arc<Self>>,
    pub overflow_clip: Option<Arc<ClipPaintPropertyNode>>,
    pub container_rect: PaintRect,
    pub contents_rect: PaintRect,
    pub minimum_scroll_offset: Offset,
    pub maximum_scroll_offset: Offset,
    pub user_scrollable_horizontal: bool,
    pub user_scrollable_vertical: bool,
}
// Source: scroll_paint_property_node.h:52-83. Chromium optional/feature fields
// stay on the producer node and retain their constructor defaults until set.
#[derive(Clone, Debug, PartialEq)]
pub struct ScrollCompositorState {
    pub compositor_element_id: foundation::CompositorElementId,
    pub prevent_scroll_axis_locking: bool,
    pub prevent_viewport_scrolling_from_inner: bool,
    pub max_scroll_offset_affected_by_page_scale: bool,
    pub composited_scrolling_preference: PaintCompositedScrollingPreference,
    pub main_thread_repaint_reasons: u8,
    pub overscroll_behavior: [layoutng_assembly::internal::layout_input::OverscrollBehavior; 2],
    pub scrolling_contents_cull_rect: PaintRect,
    /// The producer queried scroll-snap state and proved that no source
    /// SnapContainerData exists. False means a non-none snap payload is still
    /// required before this node can enter cc.
    pub snap_container_data_absent: bool,
    /// Chromium's ScrollPaintPropertyNode::State carries optional
    /// cc::SnapContainerData. Layout publishes its computed type across the
    /// package boundary and compositor owns the cc value constructed from it.
    pub snap_container_data: Option<PaintScrollSnapType>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintScrollSnapAxis {
    Both,
    X,
    Y,
    Block,
    Inline,
    Pair,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintScrollSnapStrictness {
    Proximity,
    Mandatory,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaintScrollSnapType {
    pub axis: PaintScrollSnapAxis,
    pub strictness: PaintScrollSnapStrictness,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaintCompositedScrollingPreference {
    #[default]
    Default,
    Preferred,
    NotPreferred,
}
impl Default for ScrollCompositorState {
    fn default() -> Self {
        let r = foundation::infinite_int_rect::InfiniteIntRect();
        Self {
            compositor_element_id: Default::default(),
            prevent_scroll_axis_locking: false,
            prevent_viewport_scrolling_from_inner: false,
            max_scroll_offset_affected_by_page_scale: false,
            composited_scrolling_preference: Default::default(),
            main_thread_repaint_reasons: 0,
            overscroll_behavior: [Default::default(); 2],
            scrolling_contents_cull_rect: PaintRect {
                x: r.x() as f64,
                y: r.y() as f64,
                width: r.width() as f64,
                height: r.height() as f64,
            },
            snap_container_data_absent: false,
            snap_container_data: None,
        }
    }
}

impl ScrollPaintPropertyNode {
    // Source: scroll_paint_property_node.h:127-211. Both referenced runtime
    // features are stable at the pinned Chromium revision, so ContentsRect and
    // ScrollingContentsCullRect use their enabled source branches.
    pub fn OverscrollBehavior(&self) -> [LayoutOverscrollBehavior; 2] {
        self.compositor.overscroll_behavior
    }
    pub fn ContainerRect(&self) -> PaintRect {
        self.container_rect
    }
    pub fn ContentsRect(&self) -> PaintRect {
        self.contents_rect
    }
    pub fn OverflowClipNode(&self) -> Option<&Arc<ClipPaintPropertyNode>> {
        self.overflow_clip.as_ref()
    }
    pub fn UserScrollableHorizontal(&self) -> bool {
        self.user_scrollable_horizontal
    }
    pub fn UserScrollableVertical(&self) -> bool {
        self.user_scrollable_vertical
    }
    pub fn UserScrollable(&self) -> bool {
        self.UserScrollableHorizontal() || self.UserScrollableVertical()
    }
    pub fn PreventViewportScrollingFromInner(&self) -> bool {
        self.compositor.prevent_viewport_scrolling_from_inner
    }
    pub fn MaxScrollOffsetAffectedByPageScale(&self) -> bool {
        self.compositor.max_scroll_offset_affected_by_page_scale
    }
    pub fn PreventScrollAxisLocking(&self) -> bool {
        self.compositor.prevent_scroll_axis_locking
    }
    pub fn GetCompositedScrollingPreference(&self) -> PaintCompositedScrollingPreference {
        self.compositor.composited_scrolling_preference
    }
    pub fn GetMainThreadRepaintReasons(&self) -> u8 {
        self.compositor.main_thread_repaint_reasons
    }
    pub fn RequiresMainThreadForBackgroundAttachmentFixed(&self) -> bool {
        self.GetMainThreadRepaintReasons() & 1 != 0
    }
    pub fn GetCompositorElementId(&self) -> foundation::CompositorElementId {
        self.compositor.compositor_element_id
    }
    pub fn ScrollingContentsCullRect(&self) -> PaintRect {
        self.compositor.scrolling_contents_cull_rect
    }

    /// Source: scroll_paint_property_node.h:199-204. Direct cull-rect updates
    /// always publish the source's simple-values marker, even for equal input.
    pub fn SetScrollingContentsCullRect(&mut self, rect: PaintRect) {
        self.lifecycle = self
            .lifecycle
            .updated(PaintPropertyChangeType::ChangedOnlySimpleValues);
        self.compositor.scrolling_contents_cull_rect = rect;
    }

    /// Source: scroll_paint_property_node.h:109-120. Layout-only association
    /// fields are carried through the package boundary without changing the
    /// Chromium property-node change classification.
    pub fn Update(
        &mut self,
        parent: Arc<ScrollPaintPropertyNode>,
        mut state: ScrollPaintPropertyNode,
    ) -> PaintPropertyChangeType {
        state.parent = Some(parent);
        let change = scroll_state_change(self, &state);
        let auxiliary_changed = self.owner_client_id != state.owner_client_id
            || self.minimum_scroll_offset != state.minimum_scroll_offset
            || self.maximum_scroll_offset != state.maximum_scroll_offset;
        if change != PaintPropertyChangeType::Unchanged || auxiliary_changed {
            let lifecycle = if change == PaintPropertyChangeType::Unchanged {
                self.lifecycle.clone()
            } else {
                self.lifecycle.updated(change)
            };
            let id = self.id;
            *self = state;
            self.lifecycle = lifecycle;
            self.id = id;
        }
        change
    }

    /// Source: scroll_paint_property_node.cc:73-79.
    pub fn ClearChangedToRoot(&self, sequence_number: i32) {
        let mut current = Some(self);
        while let Some(node) = current {
            if node.lifecycle.changed_sequence_number() == sequence_number {
                break;
            }
            node.lifecycle.clear_changed(sequence_number);
            current = node.parent.as_deref();
        }
    }

    /// Source: scroll_paint_property_node.cc:81-142. Snap-container output is
    /// intentionally absent until the producer can publish the complete cc
    /// value; supported non-root snapshots prove that absence explicitly.
    pub fn ToJSON(&self) -> serde_json::Value {
        fn rect_string(rect: PaintRect) -> String {
            format!("{},{} {}x{}", rect.x, rect.y, rect.width, rect.height)
        }
        fn overscroll_name(value: LayoutOverscrollBehavior) -> &'static str {
            match value {
                LayoutOverscrollBehavior::kNone => "none",
                LayoutOverscrollBehavior::kAuto => "auto",
                LayoutOverscrollBehavior::kChain => "chain",
                LayoutOverscrollBehavior::kContain => "contain",
            }
        }
        fn repaint_reasons(bits: u8) -> String {
            [
                "Has background-attachment:fixed",
                "Not opaque for text and LCD text",
                "Prefer non-composited scrolling",
                "Background needs repaint on scroll",
            ]
            .into_iter()
            .enumerate()
            .filter_map(|(index, name)| (bits & (1 << index) != 0).then_some(name))
            .collect::<Vec<_>>()
            .join(", ")
        }

        let mut json = PaintPropertyNodeToJSON(self.parent.as_ref(), &self.lifecycle);
        if self.ContainerRect().width != 0.0 && self.ContainerRect().height != 0.0 {
            json.insert(
                "containerRect".into(),
                serde_json::Value::String(rect_string(self.ContainerRect())),
            );
        }
        if self.ContentsRect().width != 0.0 && self.ContentsRect().height != 0.0 {
            json.insert(
                "contentsRect".into(),
                serde_json::Value::String(rect_string(self.ContentsRect())),
            );
        }
        if let Some(clip) = self.OverflowClipNode() {
            json.insert(
                "overflowClipNode".into(),
                serde_json::Value::String(format!("{:p}", Arc::as_ptr(clip))),
            );
        }
        if self.UserScrollable() {
            let value = if self.UserScrollableHorizontal() {
                if self.UserScrollableVertical() {
                    "both"
                } else {
                    "horizontal"
                }
            } else {
                "vertical"
            };
            json.insert(
                "userScrollable".into(),
                serde_json::Value::String(value.into()),
            );
        }
        if self.GetMainThreadRepaintReasons() != 0 {
            json.insert(
                "mainThreadReasons".into(),
                serde_json::Value::String(repaint_reasons(self.GetMainThreadRepaintReasons())),
            );
        }
        if self.MaxScrollOffsetAffectedByPageScale() {
            json.insert(
                "maxScrollOffsetAffectedByPageScale".into(),
                serde_json::Value::String("true".into()),
            );
        }
        let compositor_id = self.GetCompositorElementId().GetInternalValue();
        if compositor_id != 0 {
            json.insert(
                "compositorElementId".into(),
                serde_json::Value::String(format!("({compositor_id})")),
            );
        }
        let overscroll = self.OverscrollBehavior();
        if overscroll[0] != LayoutOverscrollBehavior::kAuto {
            json.insert(
                "overscroll-behavior-x".into(),
                serde_json::Value::String(overscroll_name(overscroll[0]).into()),
            );
        }
        if overscroll[1] != LayoutOverscrollBehavior::kAuto {
            json.insert(
                "overscroll-behavior-y".into(),
                serde_json::Value::String(overscroll_name(overscroll[1]).into()),
            );
        }
        if self.PreventScrollAxisLocking() {
            json.insert(
                "preventScrollAxisLock".into(),
                serde_json::Value::Bool(true),
            );
        }
        let infinite = foundation::infinite_int_rect::InfiniteIntRect();
        let cull = self.ScrollingContentsCullRect();
        let is_infinite = cull.x == f64::from(infinite.x())
            && cull.y == f64::from(infinite.y())
            && cull.width == f64::from(infinite.width())
            && cull.height == f64::from(infinite.height());
        json.insert(
            "contents_cull_rect".into(),
            serde_json::Value::String(if is_infinite {
                "Inf".into()
            } else {
                rect_string(cull)
            }),
        );
        serde_json::Value::Object(json)
    }
    pub fn ToString(&self) -> String {
        PaintPropertyNodeToString(&self.ToJSON())
    }
}

/// Complete inherited state. Scroll is reached through transform.scroll,
/// matching Blink PropertyTreeState's three references.
#[derive(Clone, Debug)]
pub struct PropertyTreeState {
    pub transform: Arc<TransformPaintPropertyNode>,
    pub clip: Arc<ClipPaintPropertyNode>,
    pub effect: Arc<EffectPaintPropertyNode>,
}

impl PartialEq for PropertyTreeState {
    fn eq(&self, other: &Self) -> bool {
        self.transform
            .lifecycle
            .same_node(&other.transform.lifecycle)
            && self.clip.lifecycle.same_node(&other.clip.lifecycle)
            && self.effect.lifecycle.same_node(&other.effect.lifecycle)
    }
}
impl Eq for PropertyTreeState {}
impl fmt::Display for PropertyTreeState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.ToString())
    }
}

impl Default for PropertyTreeState {
    fn default() -> Self {
        // Blink Root() references shared sentinel nodes, rather than allocating
        // distinct identity nodes for each default state.
        static ROOT: OnceLock<PropertyTreeState> = OnceLock::new();
        ROOT.get_or_init(|| {
            let infinite = foundation::infinite_int_rect::InfiniteIntRect();
            let infinite = PaintRect {
                x: infinite.x() as f64,
                y: infinite.y() as f64,
                width: infinite.width() as f64,
                height: infinite.height() as f64,
            };
            // ScrollRoot is a sentinel, not a real scroll. Its nullable overflow
            // clip also avoids manufacturing a cyclic root clip relationship.
            let scroll = Arc::new(ScrollPaintPropertyNode {
                compositor: Default::default(),
                lifecycle: PaintPropertyNodeLifecycle::root(),
                id: 0,
                owner_client_id: 0,
                parent: None,
                overflow_clip: None,
                container_rect: infinite,
                contents_rect: infinite,
                minimum_scroll_offset: Offset::default(),
                maximum_scroll_offset: Offset::default(),
                user_scrollable_horizontal: false,
                user_scrollable_vertical: false,
            });
            let transform = Arc::new(TransformPaintPropertyNode {
                compositor: TransformCompositorState {
                    in_subtree_of_page_scale: false,
                    ..Default::default()
                },
                geometry_mapper_cache: Default::default(),
                scroll_parent_scroll_translation: None,
                lifecycle: PaintPropertyNodeLifecycle::root(),
                id: 0,
                parent: None,
                matrix: TransformMatrix::default(),
                origin: [0.0; 3],
                flattens_inherited_transform: false,
                rendering_context_id: 0,
                backface_visibility: PaintBackfaceVisibility::Inherited,
                scroll: Some(scroll),
                sticky_constraint: None,
                direct_compositing_reasons: Default::default(),
            });
            let clip = Arc::new(
                ClipPaintPropertyNode {
                    geometry_mapper_cache: Default::default(),
                    layout_clip_rects: Default::default(),
                    lifecycle: PaintPropertyNodeLifecycle::root(),
                    id: 0,
                    parent: None,
                    local_transform_space: transform.clone(),
                    rect: None,
                    radii: PaintCornerRadii::default(),
                    clip_path: None,
                    clip_path_error: None,
                    pixel_moving_filter: None,
                }
                .with_layout_clips(),
            );
            let effect = Arc::new(
                EffectPaintPropertyNode {
                    compositor: Default::default(),
                    backdrop_filter_bounds: None,
                    self_or_ancestor_participates_in_view_transition: false,
                    lifecycle: PaintPropertyNodeLifecycle::root(),
                    id: 0,
                    parent: None,
                    local_transform_space: transform.clone(),
                    output_clip: None,
                    filter_reference_box: None,
                    filter_viewport: None,
                    filter_space_origin: Offset::default(),
                    filter_reference_scale: 1.0,
                    opacity: 1.0,
                    blend_mode: PaintBlendMode::kNormal,
                    filters: Default::default(),
                    backdrop_filters: Default::default(),
                    backdrop_filter_has_reference: false,
                    backdrop_filter_clip: None,
                    isolates_blending: false,
                    has_mask: false,
                    is_mask: false,
                    direct_compositing_reasons: Default::default(),
                }
                .with_inherited_view_transition(),
            );
            PropertyTreeState {
                transform,
                clip,
                effect,
            }
        })
        .clone()
    }
}

impl PropertyTreeState {
    /// Source: property_tree_state.h:275-280.
    pub fn Root() -> Self {
        Self::default()
    }
    pub fn Transform(&self) -> &Arc<TransformPaintPropertyNode> {
        &self.transform
    }
    pub fn SetTransform(&mut self, node: Arc<TransformPaintPropertyNode>) {
        self.transform = node;
    }
    pub fn Clip(&self) -> &Arc<ClipPaintPropertyNode> {
        &self.clip
    }
    pub fn SetClip(&mut self, node: Arc<ClipPaintPropertyNode>) {
        self.clip = node;
    }
    pub fn Effect(&self) -> &Arc<EffectPaintPropertyNode> {
        &self.effect
    }
    pub fn SetEffect(&mut self, node: Arc<EffectPaintPropertyNode>) {
        self.effect = node;
    }

    /// Source: property_tree_state.h:142-152,288-301.
    pub fn ClearChangedToRoot(&self, sequence_number: i32) {
        self.Transform().ClearChangedToRoot(sequence_number);
        self.Clip().ClearChangedToRoot(sequence_number);
        self.Effect().ClearChangedToRoot(sequence_number);
    }
    pub fn Changed(&self, change: PaintPropertyChangeType, relative_to: &Self) -> bool {
        crate::property_node_geometry::PropertyStateChanged(self, change, relative_to)
    }
    pub fn ChangedToRoot(&self, change: PaintPropertyChangeType) -> bool {
        self.Changed(change, &Self::Root())
    }

    /// Source: property_tree_state.cc:198-200,212-218.
    pub fn ToString(&self) -> String {
        format!(
            "t:{:p} c:{:p} e:{:p}",
            Arc::as_ptr(self.Transform()),
            Arc::as_ptr(self.Clip()),
            Arc::as_ptr(self.Effect())
        )
    }
    pub fn ToJSON(&self) -> serde_json::Value {
        serde_json::json!({
            "transform": self.Transform().ToJSON(),
            "clip": self.Clip().ToJSON(),
            "effect": self.Effect().ToJSON(),
        })
    }

    pub fn same_nodes(&self, other: &Self) -> bool {
        self == other
    }
    fn same_snapshots(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.transform, &other.transform)
            && Arc::ptr_eq(&self.clip, &other.clip)
            && Arc::ptr_eq(&self.effect, &other.effect)
    }
    /// Frame-state comparison is intentionally separate from node identity.
    /// A stable node can have new values (including changed ancestor values).
    pub fn same_values(&self, other: &Self) -> bool {
        if Arc::ptr_eq(&self.transform, &other.transform)
            && Arc::ptr_eq(&self.clip, &other.clip)
            && Arc::ptr_eq(&self.effect, &other.effect)
        {
            return true;
        }
        // Compare each shared graph node once. Recursive derived equality can
        // revisit filter/clip cross-links exponentially in a nested subtree.
        let a = PaintPropertyTrees::from_states([self]);
        let b = PaintPropertyTrees::from_states([other]);
        same_directory(&a.transforms, &b.transforms, |a, b| {
            a.lifecycle == b.lifecycle
                && transform_state_change(a, b) == PaintPropertyChangeType::Unchanged
        }) && same_directory(&a.clips, &b.clips, |a, b| {
            a.lifecycle == b.lifecycle
                && clip_state_change(a, b) == PaintPropertyChangeType::Unchanged
        }) && same_directory(&a.effects, &b.effects, |a, b| {
            a.lifecycle == b.lifecycle
                && effect_state_change(a, b) == PaintPropertyChangeType::Unchanged
        }) && same_directory(&a.scrolls, &b.scrolls, |a, b| {
            a.lifecycle == b.lifecycle
                && scroll_state_change(a, b) == PaintPropertyChangeType::Unchanged
        })
    }
    pub fn changed_to_root(&self) -> PaintPropertyChangeType {
        let trees = PaintPropertyTrees::from_states([self]);
        trees
            .transforms
            .values()
            .map(|n| n.lifecycle.node_changed())
            .chain(trees.clips.values().map(|n| n.lifecycle.node_changed()))
            .chain(trees.effects.values().map(|n| n.lifecycle.node_changed()))
            .chain(trees.scrolls.values().map(|n| n.lifecycle.node_changed()))
            .max()
            .unwrap_or_default()
    }
    pub fn nearest_scroll(&self) -> Option<Arc<ScrollPaintPropertyNode>> {
        let mut transform = Some(self.transform.clone());
        while let Some(node) = transform {
            if let Some(scroll) = &node.scroll {
                return Some(scroll.clone());
            }
            transform = node.parent.clone();
        }
        None
    }
}

/// A deduplicated directory suitable for exporting the complete referenced
/// ancestry of one paint artifact. Per-type roots all have ID zero.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct PaintPropertyTrees {
    pub transforms: BTreeMap<u64, Arc<TransformPaintPropertyNode>>,
    pub clips: BTreeMap<u64, Arc<ClipPaintPropertyNode>>,
    pub effects: BTreeMap<u64, Arc<EffectPaintPropertyNode>>,
    pub scrolls: BTreeMap<u64, Arc<ScrollPaintPropertyNode>>,
    transforms_by_identity: BTreeMap<usize, Arc<TransformPaintPropertyNode>>,
    clips_by_identity: BTreeMap<usize, Arc<ClipPaintPropertyNode>>,
    effects_by_identity: BTreeMap<usize, Arc<EffectPaintPropertyNode>>,
    scrolls_by_identity: BTreeMap<usize, Arc<ScrollPaintPropertyNode>>,
}

impl PaintPropertyTrees {
    fn identity(lifecycle: &PaintPropertyNodeLifecycle) -> usize {
        Arc::as_ptr(&lifecycle.identity) as usize
    }
    pub fn transform_for(
        &self,
        node: &TransformPaintPropertyNode,
    ) -> Option<&Arc<TransformPaintPropertyNode>> {
        self.transforms_by_identity
            .get(&Self::identity(&node.lifecycle))
    }
    pub fn clip_for(&self, node: &ClipPaintPropertyNode) -> Option<&Arc<ClipPaintPropertyNode>> {
        self.clips_by_identity.get(&Self::identity(&node.lifecycle))
    }
    pub fn effect_for(
        &self,
        node: &EffectPaintPropertyNode,
    ) -> Option<&Arc<EffectPaintPropertyNode>> {
        self.effects_by_identity
            .get(&Self::identity(&node.lifecycle))
    }
    pub fn scroll_for(
        &self,
        node: &ScrollPaintPropertyNode,
    ) -> Option<&Arc<ScrollPaintPropertyNode>> {
        self.scrolls_by_identity
            .get(&Self::identity(&node.lifecycle))
    }
    pub fn transform_snapshots(&self) -> impl Iterator<Item = &Arc<TransformPaintPropertyNode>> {
        self.transforms_by_identity.values()
    }
    pub fn clip_snapshots(&self) -> impl Iterator<Item = &Arc<ClipPaintPropertyNode>> {
        self.clips_by_identity.values()
    }
    pub fn effect_snapshots(&self) -> impl Iterator<Item = &Arc<EffectPaintPropertyNode>> {
        self.effects_by_identity.values()
    }
    pub fn scroll_snapshots(&self) -> impl Iterator<Item = &Arc<ScrollPaintPropertyNode>> {
        self.scrolls_by_identity.values()
    }
    fn insert_transform(&mut self, node: Arc<TransformPaintPropertyNode>) {
        self.transforms_by_identity
            .insert(Self::identity(&node.lifecycle), node.clone());
        self.transforms.entry(node.id).or_insert(node);
    }
    fn insert_clip(&mut self, node: Arc<ClipPaintPropertyNode>) {
        self.clips_by_identity
            .insert(Self::identity(&node.lifecycle), node.clone());
        self.clips.entry(node.id).or_insert(node);
    }
    fn insert_effect(&mut self, node: Arc<EffectPaintPropertyNode>) {
        self.effects_by_identity
            .insert(Self::identity(&node.lifecycle), node.clone());
        self.effects.entry(node.id).or_insert(node);
    }
    fn insert_scroll(&mut self, node: Arc<ScrollPaintPropertyNode>) {
        self.scrolls_by_identity
            .insert(Self::identity(&node.lifecycle), node.clone());
        self.scrolls.entry(node.id).or_insert(node);
    }
    /// Clear every referenced ancestor, including fresh nodes without a
    /// source-owner slot. Blink's PropertyTreeState::ClearChangedToRoot walks
    /// the actual state graph; the owner directory is not the complete graph.
    pub fn clear_state_change_markers(&mut self, state: &mut PropertyTreeState) {
        let mut rebind = RetainedScrollSnapshotRebinder {
            translations: BTreeMap::new(),
            trees: std::mem::take(self),
            changed: PaintPropertyChangeType::Unchanged,
        };
        state.transform = rebind.transform(&state.transform);
        state.clip = rebind.clip(&state.clip);
        state.effect = rebind.effect(&state.effect);
        *self = rebind.trees;
    }

    pub fn from_states<'a>(states: impl IntoIterator<Item = &'a PropertyTreeState>) -> Self {
        let mut trees = Self::default();
        for state in states {
            trees.add_transform(&state.transform);
            trees.add_clip(&state.clip);
            trees.add_effect(&state.effect);
        }
        trees
    }
    fn add_transform(&mut self, node: &Arc<TransformPaintPropertyNode>) {
        if self.transform_for(node).is_some() {
            return;
        }
        self.insert_transform(node.clone());
        if let Some(parent) = &node.parent {
            self.add_transform(parent);
        }
        if let Some(scroll) = &node.scroll {
            self.add_scroll(scroll);
        }
        if let Some(parent) = &node.scroll_parent_scroll_translation {
            self.add_transform(parent);
        }
    }
    fn add_clip(&mut self, node: &Arc<ClipPaintPropertyNode>) {
        if self.clip_for(node).is_some() {
            return;
        }
        self.insert_clip(node.clone());
        if let Some(parent) = &node.parent {
            self.add_clip(parent);
        }
        if let Some(filter) = &node.pixel_moving_filter {
            self.add_effect(filter);
        }
        self.add_transform(&node.local_transform_space);
    }
    fn add_effect(&mut self, node: &Arc<EffectPaintPropertyNode>) {
        if self.effect_for(node).is_some() {
            return;
        }
        self.insert_effect(node.clone());
        if let Some(parent) = &node.parent {
            self.add_effect(parent);
        }
        self.add_transform(&node.local_transform_space);
        if let Some(clip) = &node.output_clip {
            self.add_clip(clip);
        }
        if let Some(clip) = &node.backdrop_filter_clip {
            self.add_clip(clip);
        }
        if let Some(child) = &node.compositor.canvas_child_state {
            self.add_transform(&child.content_state.transform);
            self.add_clip(&child.content_state.clip);
            self.add_effect(&child.content_state.effect);
        }
    }
    fn add_scroll(&mut self, node: &Arc<ScrollPaintPropertyNode>) {
        if self.scroll_for(node).is_some() {
            return;
        }
        self.insert_scroll(node.clone());
        if let Some(parent) = &node.parent {
            self.add_scroll(parent);
        }
        if let Some(clip) = &node.overflow_clip {
            self.add_clip(clip);
        }
    }
}

/// ObjectPaintProperties owns one slot per property kind and fragment. The
/// source-specific transform slots mirror ObjectPaintProperties roles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PaintPropertyRole {
    ViewportClip,
    OverscrollElasticityTransform,
    PageScale,
    VisualViewportScroll,
    VisualViewportScrollTranslation,
    PaintOffsetTranslation,
    StickyTranslation,
    TextCombineTransform,
    Translate,
    Rotate,
    Scale,
    Offset,
    Transform,
    Perspective,
    SvgLocalTransform,
    ClipPathClip,
    CssClip,
    BackgroundClip,
    BackdropFilterClip,
    MaskClip,
    Effect,
    Mask,
    Filter,
    PixelMovingFilterClipExpander,
    OverflowControlsClip,
    InnerBorderRadiusClip,
    InnerBorderShapeClip,
    OverflowClip,
    HorizontalScrollbarEffect,
    VerticalScrollbarEffect,
    Scroll,
    ScrollTranslation,
    ReplacedContentTransform,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PaintPropertyOwner {
    Viewport,
    Fragment { client: u64, fragment: u32 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PaintPropertyKey {
    pub owner: PaintPropertyOwner,
    pub role: PaintPropertyRole,
}

/// Persistent Create-or-Update slots; source: ObjectPaintProperties::Update /
/// Clear in core/paint/object_paint_properties.h. The map is scoped to one
/// retained paint lifecycle, never global and never keyed by DOM order.
/// Old frames keep immutable snapshots; identity allocations are retained by
/// both snapshots. Removing a slot and later adding it creates a new entity.
#[derive(Clone, Debug)]
pub struct PaintPropertyNodeStore {
    next_id: u64,
    visual_viewport_unique_id: Option<u64>,
    transforms: BTreeMap<PaintPropertyKey, Arc<TransformPaintPropertyNode>>,
    clips: BTreeMap<PaintPropertyKey, Arc<ClipPaintPropertyNode>>,
    effects: BTreeMap<PaintPropertyKey, Arc<EffectPaintPropertyNode>>,
    scrolls: BTreeMap<PaintPropertyKey, Arc<ScrollPaintPropertyNode>>,
    // A cleared published graph stays cleared until an updater marks a node.
    // This avoids scanning all resident slots on entirely clean paint passes.
    change_markers_cleared: bool,
    pub changed: PaintPropertyChangeType,
}
impl Default for PaintPropertyNodeStore {
    fn default() -> Self {
        Self {
            next_id: 1,
            visual_viewport_unique_id: None,
            transforms: BTreeMap::new(),
            clips: BTreeMap::new(),
            effects: BTreeMap::new(),
            scrolls: BTreeMap::new(),
            change_markers_cleared: true,
            changed: Default::default(),
        }
    }
}
impl PaintPropertyNodeStore {
    pub fn compositor_scroll_translations(
        &self,
    ) -> impl Iterator<Item = &Arc<TransformPaintPropertyNode>> {
        self.transforms.iter().filter_map(|(key, node)| {
            // Source: local_frame_view.cc:3504-3518. Chromium publishes this
            // list from scrollable_areas_with_scroll_node_ and CHECKs that
            // every corresponding fragment owns both Scroll() and
            // ScrollTranslation(). A transform carrying only scroll-position
            // geometry (for example hidden overflow without a Paint scroll
            // node) is not a compositor scroll translation and must not enter
            // PropertyTreeManager::EnsureCompositorScrollNode().
            (key.role == PaintPropertyRole::ScrollTranslation && node.scroll.is_some())
                .then_some(node)
        })
    }
    pub fn root_scroller_scroll_translation(&self) -> Option<&Arc<TransformPaintPropertyNode>> {
        self.transforms.iter().find_map(|(key, node)| {
            (key.role == PaintPropertyRole::ScrollTranslation
                && node.RequiresCompositingForRootScroller())
            .then_some(node)
        })
    }

    /// Source: core/frame/visual_viewport.cc:96-318 and
    /// local_frame_view.cc:3480-3530. The Page's active visual viewport owns a
    /// stable page-scale node and inner viewport scroll node even at scale 1.
    /// The Paint artifact carries these same retained nodes into PAC; the
    /// compositor must not synthesize a second viewport graph from dimensions.
    pub fn ensure_single_layout_viewport_nodes(
        &mut self,
        viewport: PaintRect,
    ) -> (
        Arc<TransformPaintPropertyNode>,
        Arc<TransformPaintPropertyNode>,
        Arc<TransformPaintPropertyNode>,
    ) {
        let roots = PropertyTreeState::default();
        let unique_id = *self
            .visual_viewport_unique_id
            .get_or_insert_with(foundation::NewUniqueObjectId);
        let page_scale_element_id = foundation::CompositorElementIdFromUniqueObjectId(
            unique_id,
            foundation::CompositorElementIdNamespace::kPrimary,
        );
        let scroll_element_id = foundation::CompositorElementIdFromUniqueObjectId(
            unique_id,
            foundation::CompositorElementIdNamespace::kScroll,
        );

        // Source: VisualViewport::UpdatePaintPropertyNodesIfNeeded. On
        // platforms with root elastic overscroll, this stable identity node
        // sits immediately above page scale. PAC publishes this same node to
        // cc as ViewportPropertyIds::overscroll_elasticity_transform.
        let overscroll_key = PaintPropertyKey {
            owner: PaintPropertyOwner::Viewport,
            role: PaintPropertyRole::OverscrollElasticityTransform,
        };
        let mut overscroll = TransformPaintPropertyNode {
            compositor: TransformCompositorState {
                in_subtree_of_page_scale: false,
                ..Default::default()
            },
            geometry_mapper_cache: Default::default(),
            lifecycle: Default::default(),
            id: 0,
            parent: Some(roots.transform.clone()),
            matrix: TransformMatrix::default(),
            origin: [0.0; 3],
            flattens_inherited_transform: false,
            rendering_context_id: 0,
            backface_visibility: Default::default(),
            scroll: None,
            scroll_parent_scroll_translation: None,
            sticky_constraint: None,
            direct_compositing_reasons: Default::default(),
        };
        let overscroll = if let Some(old) = self.transforms.get(&overscroll_key) {
            let change = transform_state_change(old, &overscroll);
            if change == PaintPropertyChangeType::Unchanged {
                old.clone()
            } else {
                overscroll.id = old.id;
                overscroll.lifecycle = old.lifecycle.updated(change);
                self.changed = self.changed.max(change);
                self.change_markers_cleared = false;
                Arc::new(overscroll)
            }
        } else {
            overscroll.id = self.next_id;
            self.next_id += 1;
            self.changed = PaintPropertyChangeType::NodeAddedOrRemoved;
            self.change_markers_cleared = false;
            Arc::new(overscroll)
        };
        self.transforms.insert(overscroll_key, overscroll.clone());

        let page_scale_key = PaintPropertyKey {
            owner: PaintPropertyOwner::Viewport,
            role: PaintPropertyRole::PageScale,
        };
        let mut page_scale = TransformPaintPropertyNode {
            compositor: TransformCompositorState {
                compositor_element_id: page_scale_element_id,
                in_subtree_of_page_scale: false,
                ..Default::default()
            },
            geometry_mapper_cache: Default::default(),
            lifecycle: Default::default(),
            id: 0,
            parent: Some(overscroll.clone()),
            matrix: TransformMatrix::default(),
            origin: [0.0; 3],
            flattens_inherited_transform: false,
            rendering_context_id: 0,
            backface_visibility: Default::default(),
            scroll: None,
            scroll_parent_scroll_translation: None,
            sticky_constraint: None,
            direct_compositing_reasons: foundation::CompositingReasons::FromReasons(&[
                foundation::CompositingReason::kViewport,
            ]),
        };
        let page_scale = if let Some(old) = self.transforms.get(&page_scale_key) {
            let change = transform_state_change(old, &page_scale);
            if change == PaintPropertyChangeType::Unchanged {
                old.clone()
            } else {
                page_scale.id = old.id;
                page_scale.lifecycle = old.lifecycle.updated(change);
                self.changed = self.changed.max(change);
                self.change_markers_cleared = false;
                Arc::new(page_scale)
            }
        } else {
            page_scale.id = self.next_id;
            self.next_id += 1;
            self.changed = PaintPropertyChangeType::NodeAddedOrRemoved;
            self.change_markers_cleared = false;
            Arc::new(page_scale)
        };
        self.transforms.insert(page_scale_key, page_scale.clone());

        let visual_scroll_key = PaintPropertyKey {
            owner: PaintPropertyOwner::Viewport,
            role: PaintPropertyRole::VisualViewportScroll,
        };
        let mut visual_scroll = ScrollPaintPropertyNode {
            compositor: ScrollCompositorState {
                compositor_element_id: scroll_element_id,
                max_scroll_offset_affected_by_page_scale: true,
                scrolling_contents_cull_rect: viewport,
                snap_container_data_absent: true,
                ..Default::default()
            },
            lifecycle: Default::default(),
            id: 0,
            owner_client_id: 0,
            parent: Some(
                roots
                    .transform
                    .scroll
                    .as_ref()
                    .expect("root transform owns the root scroll node")
                    .clone(),
            ),
            overflow_clip: None,
            container_rect: viewport,
            contents_rect: viewport,
            minimum_scroll_offset: Offset::default(),
            maximum_scroll_offset: Offset::default(),
            user_scrollable_horizontal: true,
            user_scrollable_vertical: true,
        };
        let visual_scroll = if let Some(old) = self.scrolls.get(&visual_scroll_key) {
            let change = scroll_state_change(old, &visual_scroll);
            if change == PaintPropertyChangeType::Unchanged {
                old.clone()
            } else {
                visual_scroll.id = old.id;
                visual_scroll.lifecycle = old.lifecycle.updated(change);
                self.changed = self.changed.max(change);
                self.change_markers_cleared = false;
                Arc::new(visual_scroll)
            }
        } else {
            visual_scroll.id = self.next_id;
            self.next_id += 1;
            self.changed = PaintPropertyChangeType::NodeAddedOrRemoved;
            self.change_markers_cleared = false;
            Arc::new(visual_scroll)
        };
        self.scrolls
            .insert(visual_scroll_key, visual_scroll.clone());

        let visual_translation_key = PaintPropertyKey {
            owner: PaintPropertyOwner::Viewport,
            role: PaintPropertyRole::VisualViewportScrollTranslation,
        };
        let mut visual_translation = TransformPaintPropertyNode {
            compositor: Default::default(),
            geometry_mapper_cache: Default::default(),
            lifecycle: Default::default(),
            id: 0,
            parent: Some(page_scale.clone()),
            matrix: TransformMatrix::default(),
            origin: [0.0; 3],
            flattens_inherited_transform: false,
            rendering_context_id: 0,
            backface_visibility: Default::default(),
            scroll: Some(visual_scroll),
            scroll_parent_scroll_translation: None,
            sticky_constraint: None,
            direct_compositing_reasons: foundation::CompositingReasons::FromReasons(&[
                foundation::CompositingReason::kViewport,
            ]),
        };
        let visual_translation = if let Some(old) = self.transforms.get(&visual_translation_key) {
            let change = transform_state_change(old, &visual_translation);
            if change == PaintPropertyChangeType::Unchanged {
                old.clone()
            } else {
                visual_translation.id = old.id;
                visual_translation.lifecycle = old.lifecycle.updated(change);
                self.changed = self.changed.max(change);
                self.change_markers_cleared = false;
                Arc::new(visual_translation)
            }
        } else {
            visual_translation.id = self.next_id;
            self.next_id += 1;
            self.changed = PaintPropertyChangeType::NodeAddedOrRemoved;
            self.change_markers_cleared = false;
            Arc::new(visual_translation)
        };
        self.transforms
            .insert(visual_translation_key, visual_translation.clone());
        (overscroll, page_scale, visual_translation)
    }

    pub fn overscroll_elasticity_transform(&self) -> Option<&Arc<TransformPaintPropertyNode>> {
        self.transforms.get(&PaintPropertyKey {
            owner: PaintPropertyOwner::Viewport,
            role: PaintPropertyRole::OverscrollElasticityTransform,
        })
    }

    pub fn page_scale(&self) -> Option<&Arc<TransformPaintPropertyNode>> {
        self.transforms.get(&PaintPropertyKey {
            owner: PaintPropertyOwner::Viewport,
            role: PaintPropertyRole::PageScale,
        })
    }

    pub fn visual_viewport_scroll_translation(&self) -> Option<&Arc<TransformPaintPropertyNode>> {
        self.transforms.get(&PaintPropertyKey {
            owner: PaintPropertyOwner::Viewport,
            role: PaintPropertyRole::VisualViewportScrollTranslation,
        })
    }
    /// ScrollPosition values encoded by the real recording-coordinate
    /// anchors, indexed by the same source owner/role slots used by the
    /// property builder. No new layer identity.
    pub fn scroll_offsets(&self) -> BTreeMap<PaintPropertyOwner, Offset> {
        self.transforms
            .iter()
            .filter(|(key, _)| key.role == PaintPropertyRole::ScrollTranslation)
            .map(|(key, node)| {
                (
                    key.owner,
                    Offset {
                        x: -node.matrix.values[12],
                        y: -node.matrix.values[13],
                    },
                )
            })
            .collect()
    }

    /// Current sticky values carried by the source property owner. Recording
    /// basis changes may rebase scroll while preserving these existing slots.
    pub fn sticky_offsets(
        &self,
    ) -> BTreeMap<PaintPropertyOwner, (Offset, StickyPositionPaintData)> {
        self.transforms
            .iter()
            .filter_map(|(key, node)| {
                (key.role == PaintPropertyRole::StickyTranslation)
                    .then(|| {
                        node.sticky_constraint.map(|constraint| {
                            (
                                key.owner,
                                (
                                    Offset {
                                        x: node.matrix.values[12],
                                        y: node.matrix.values[13],
                                    },
                                    constraint,
                                ),
                            )
                        })
                    })
                    .flatten()
            })
            .collect()
    }

    /// Retained source geometry has already proved unchanged owners/topology,
    /// styles, resources and fragment geometry. Update existing real scroll
    /// translations without constructing a new PrePaint tree. The caller must
    /// prove retained recording coverage before publishing this staged store.
    ///
    /// Source: paint_property_tree_builder.cc:3769-3827 and
    /// property_tree_manager.cc:138-176. This backend retains immutable Blink
    /// snapshots; it does not claim a cc composited-value update. Rebinding a
    /// snapshot reference preserves its entity and revision. Per-pass change
    /// markers clear on unchanged values, as in the ordinary property updater.
    pub fn retained_scroll_update(
        &self,
        offsets: &BTreeMap<PaintPropertyOwner, Offset>,
        sticky: &BTreeMap<PaintPropertyOwner, (Offset, StickyPositionPaintData)>,
    ) -> Option<(Self, PaintPropertyTrees)> {
        let mut translations = BTreeMap::new();
        for (owner, offset) in offsets {
            if !offset.x.is_finite() || !offset.y.is_finite() {
                return None;
            }
            if *offset != Offset::default()
                && !self.transforms.contains_key(&PaintPropertyKey {
                    owner: *owner,
                    role: PaintPropertyRole::ScrollTranslation,
                })
            {
                return None;
            }
        }
        for (key, node) in &self.transforms {
            if key.role != PaintPropertyRole::ScrollTranslation {
                continue;
            }
            let Some(offset) = offsets.get(&key.owner) else {
                return None;
            };
            let current_offset = Offset {
                x: -node.matrix.values[12],
                y: -node.matrix.values[13],
            };
            // Chromium updates the ScrollTranslation belonging to the
            // ScrollableArea whose offset changed. Other translation slots can
            // describe hidden overflow or a non-composited scroller; their
            // unchanged presence must not reject an otherwise valid retained
            // update for the active scroller.
            if *offset == current_offset {
                continue;
            }
            // Preserve the previous scroll-only admission: translations without
            // real ScrollNodes (e.g. hidden overflow) require ordinary PrePaint.
            if !matches!(key.owner, PaintPropertyOwner::Fragment { .. })
                || node.scroll.as_ref().is_none_or(|scroll| scroll.id == 0)
                || node.origin != [0.0; 3]
            {
                return None;
            }
            let mut linear = node.matrix.values;
            linear[12] = 0.0;
            linear[13] = 0.0;
            if linear != TransformMatrix::default().values {
                return None;
            }
            let mut matrix = node.matrix;
            matrix.values[12] = -offset.x;
            matrix.values[13] = -offset.y;
            translations.insert(
                PaintPropertyTrees::identity(&node.lifecycle),
                (matrix, node.sticky_constraint),
            );
        }
        // FragmentPaintPropertyTreeBuilder::UpdateStickyTranslation uses
        // Layout's resolved offset and constraint data. Do not recompute sticky
        // geometry in Paint, or treat its scroll-dependent position as relayout.
        for owner in sticky.keys() {
            if !self.transforms.contains_key(&PaintPropertyKey {
                owner: *owner,
                role: PaintPropertyRole::StickyTranslation,
            }) {
                return None;
            }
        }
        for (key, node) in &self.transforms {
            if key.role != PaintPropertyRole::StickyTranslation {
                continue;
            }
            let Some((offset, constraint)) = sticky.get(&key.owner) else {
                return None;
            };
            let Some(before) = node.sticky_constraint else {
                return None;
            };
            let current_offset = Offset {
                x: node.matrix.values[12],
                y: node.matrix.values[13],
            };
            if *offset == current_offset && before == *constraint {
                continue;
            }
            if !offset.x.is_finite()
                || !offset.y.is_finite()
                || node.origin != [0.0; 3]
                || !before.same_geometry(constraint)
            {
                return None;
            }
            let mut linear = node.matrix.values;
            linear[12] = 0.0;
            linear[13] = 0.0;
            if linear != TransformMatrix::default().values {
                return None;
            }
            let mut matrix = node.matrix;
            matrix.values[12] = offset.x;
            matrix.values[13] = offset.y;
            translations.insert(
                PaintPropertyTrees::identity(&node.lifecycle),
                (matrix, Some(*constraint)),
            );
        }
        Some(self.staged_snapshots(translations))
    }

    /// Clear the preceding paint pass's change markers in new immutable
    /// snapshots. As with Blink's ClearChangedToRoot, this changes no property
    /// values, node entities or revisions; old paint artifacts retain their
    /// original markers. The caller rebinds published states using the trees.
    pub fn cleared_change_markers(&self) -> Option<(Self, PaintPropertyTrees)> {
        if self.changed == PaintPropertyChangeType::Unchanged
            && (self.change_markers_cleared
                || (self
                    .transforms
                    .values()
                    .all(|n| n.lifecycle.changed == PaintPropertyChangeType::Unchanged)
                    && self
                        .clips
                        .values()
                        .all(|n| n.lifecycle.changed == PaintPropertyChangeType::Unchanged)
                    && self
                        .effects
                        .values()
                        .all(|n| n.lifecycle.changed == PaintPropertyChangeType::Unchanged)
                    && self
                        .scrolls
                        .values()
                        .all(|n| n.lifecycle.changed == PaintPropertyChangeType::Unchanged)))
        {
            return None;
        }
        Some(self.staged_snapshots(BTreeMap::new()))
    }

    fn staged_snapshots(
        &self,
        translations: BTreeMap<usize, (TransformMatrix, Option<StickyPositionPaintData>)>,
    ) -> (Self, PaintPropertyTrees) {
        let mut rebind = RetainedScrollSnapshotRebinder {
            translations,
            trees: PaintPropertyTrees::default(),
            changed: PaintPropertyChangeType::Unchanged,
        };
        // Empty directories still expose the real shared per-type sentinels.
        let roots = PropertyTreeState::default();
        rebind.transform(&roots.transform);
        rebind.clip(&roots.clip);
        rebind.effect(&roots.effect);
        let transforms = self
            .transforms
            .iter()
            .map(|(key, node)| (*key, rebind.transform(node)))
            .collect();
        let clips = self
            .clips
            .iter()
            .map(|(key, node)| (*key, rebind.clip(node)))
            .collect();
        let effects = self
            .effects
            .iter()
            .map(|(key, node)| (*key, rebind.effect(node)))
            .collect();
        let scrolls = self
            .scrolls
            .iter()
            .map(|(key, node)| (*key, rebind.scroll(node)))
            .collect();
        let store = Self {
            next_id: self.next_id,
            visual_viewport_unique_id: self.visual_viewport_unique_id,
            transforms,
            clips,
            effects,
            scrolls,
            change_markers_cleared: rebind.changed == PaintPropertyChangeType::Unchanged,
            changed: rebind.changed,
        };
        (store, rebind.trees)
    }

    pub fn viewport_clip(&self) -> Option<&Arc<ClipPaintPropertyNode>> {
        self.clips.get(&PaintPropertyKey {
            owner: PaintPropertyOwner::Viewport,
            role: PaintPropertyRole::ViewportClip,
        })
    }

    pub fn is_scroll_only_update_from(&self, old: &Self) -> bool {
        let reject = |kind: &str,
                      key: &PaintPropertyKey,
                      before: u64,
                      after: u64,
                      same_identity: bool,
                      change: PaintPropertyChangeType| {
            if std::env::var_os("BROWSER_PROFILE_INPUT").is_some() {
                eprintln!(
                    "scroll-property-reject kind={} key={:?} old_id={} new_id={} same_identity={} change={:?}",
                    kind, key, before, after, same_identity, change
                );
            }
            false
        };
        if self.transforms.len() != old.transforms.len()
            || self.clips.len() != old.clips.len()
            || self.effects.len() != old.effects.len()
            || self.scrolls.len() != old.scrolls.len()
        {
            if std::env::var_os("BROWSER_PROFILE_INPUT").is_some() {
                eprintln!(
                    "scroll-property-reject kind=slot-count old={:?} new={:?}",
                    (
                        old.transforms.len(),
                        old.clips.len(),
                        old.effects.len(),
                        old.scrolls.len()
                    ),
                    (
                        self.transforms.len(),
                        self.clips.len(),
                        self.effects.len(),
                        self.scrolls.len()
                    )
                );
            }
            return false;
        }
        for (key, node) in &self.transforms {
            let Some(before) = old.transforms.get(key) else {
                return reject(
                    "transform-slot",
                    key,
                    0,
                    node.id,
                    false,
                    PaintPropertyChangeType::NodeAddedOrRemoved,
                );
            };
            let change = transform_state_change(before, node);
            let same_identity = node.lifecycle.same_node(&before.lifecycle);
            if !same_identity
                || !(change == PaintPropertyChangeType::Unchanged
                    || (key.role == PaintPropertyRole::ScrollTranslation
                        && node.scroll.as_ref().is_some_and(|scroll| scroll.id != 0)
                        && change == PaintPropertyChangeType::ChangedOnlySimpleValues))
            {
                if std::env::var_os("BROWSER_PROFILE_INPUT").is_some() {
                    eprintln!(
                        "scroll-transform-values old={:?} new={:?}",
                        before.matrix.values, node.matrix.values
                    );
                }
                return reject("transform", key, before.id, node.id, same_identity, change);
            }
        }
        macro_rules! check_nodes {
            ($map:ident, $change:ident) => {
                for (key, node) in &self.$map {
                    let Some(before) = old.$map.get(key) else {
                        return reject(
                            stringify!($map),
                            key,
                            0,
                            node.id,
                            false,
                            PaintPropertyChangeType::NodeAddedOrRemoved,
                        );
                    };
                    let change = $change(before, node);
                    let same_identity = node.lifecycle.same_node(&before.lifecycle);
                    if !same_identity || change != PaintPropertyChangeType::Unchanged {
                        return reject(
                            stringify!($map),
                            key,
                            before.id,
                            node.id,
                            same_identity,
                            change,
                        );
                    }
                }
            };
        }
        check_nodes!(clips, clip_state_change);
        check_nodes!(effects, effect_state_change);
        check_nodes!(scrolls, scroll_state_change);
        true
    }
    pub fn clear_owner(&mut self, owner: PaintPropertyOwner) {
        self.transforms.retain(|k, _| k.owner != owner);
        self.clips.retain(|k, _| k.owner != owner);
        self.effects.retain(|k, _| k.owner != owner);
        self.scrolls.retain(|k, _| k.owner != owner);
    }
    pub fn updater<'a>(
        &'a mut self,
        keys: &BTreeMap<u64, PaintPropertyKey>,
        prior_keys: &BTreeMap<u64, PaintPropertyKey>,
        effect_sources: &'a BTreeMap<usize, Arc<EffectPaintPropertyNode>>,
    ) -> PropertyNodeUpdater<'a> {
        // Raw builder ids name nodes created in this pass. Retained clean
        // descendants can still hold the stable id assigned to the same
        // owner/role slot in an earlier pass, so index both forms before the
        // store is mutably borrowed. Once a rebuilt ancestor updates a slot,
        // every stale descendant reference can then resolve to that slot's
        // canonical current snapshot instead of reintroducing its old value.
        let mut stable_transform_keys = BTreeMap::new();
        for (key, node) in &self.transforms {
            stable_transform_keys.insert(node.id, *key);
        }
        let mut stable_clip_keys = BTreeMap::new();
        for (key, node) in &self.clips {
            stable_clip_keys.insert(node.id, *key);
        }
        let mut stable_effect_keys = BTreeMap::new();
        for (key, node) in &self.effects {
            stable_effect_keys.insert(node.id, *key);
        }
        let mut stable_scroll_keys = BTreeMap::new();
        for (key, node) in &self.scrolls {
            stable_scroll_keys.insert(node.id, *key);
        }
        self.changed = PaintPropertyChangeType::Unchanged;
        // A partial update cannot certify that every retained graph is clean.
        self.change_markers_cleared = false;
        let seen = self
            .transforms
            .keys()
            .chain(self.scrolls.keys())
            .filter(|key| {
                key.owner == PaintPropertyOwner::Viewport
                    && matches!(
                        key.role,
                        PaintPropertyRole::OverscrollElasticityTransform
                            | PaintPropertyRole::PageScale
                            | PaintPropertyRole::VisualViewportScroll
                            | PaintPropertyRole::VisualViewportScrollTranslation
                    )
            })
            .copied()
            .collect();
        PropertyNodeUpdater {
            store: self,
            raw_keys: keys.clone(),
            prior_raw_keys: prior_keys.clone(),
            stable_transform_keys,
            stable_clip_keys,
            stable_effect_keys,
            stable_scroll_keys,
            effect_sources,
            seen,
            transforms: BTreeMap::new(),
            clips: BTreeMap::new(),
            effects: BTreeMap::new(),
            scrolls: BTreeMap::new(),
        }
    }
    pub fn next_id(&self) -> u64 {
        self.next_id
    }
}

/// Rebinds property references carried by cached PaintArtifact data to the
/// immutable snapshots published by the current PrePaint pass.
///
/// Source: ObjectPaintProperties and PaintController::AppendSubsequenceByMoving.
/// Chromium mutates each owned property node in place, so a cached PaintChunk
/// automatically observes its current parents and cross-tree links. Rechrom's
/// immutable snapshots need the equivalent publication step. Owned slots bind
/// by source node identity; unowned wrapper nodes are cloned recursively so
/// their complete ancestry observes those current slots. Numeric node ids are
/// dump-local labels and deliberately do not participate in identity.
pub struct ArtifactPropertyStateRebinder {
    current_transforms_by_identity: BTreeMap<usize, Arc<TransformPaintPropertyNode>>,
    current_clips_by_identity: BTreeMap<usize, Arc<ClipPaintPropertyNode>>,
    current_effects_by_identity: BTreeMap<usize, Arc<EffectPaintPropertyNode>>,
    current_scrolls_by_identity: BTreeMap<usize, Arc<ScrollPaintPropertyNode>>,
    rebound_transforms: BTreeMap<usize, Arc<TransformPaintPropertyNode>>,
    rebound_clips: BTreeMap<usize, Arc<ClipPaintPropertyNode>>,
    rebound_effects: BTreeMap<usize, Arc<EffectPaintPropertyNode>>,
    rebound_scrolls: BTreeMap<usize, Arc<ScrollPaintPropertyNode>>,
    visiting_effects: BTreeSet<usize>,
}

impl ArtifactPropertyStateRebinder {
    pub fn new(store: &PaintPropertyNodeStore) -> Self {
        let mut result = Self {
            current_transforms_by_identity: BTreeMap::new(),
            current_clips_by_identity: BTreeMap::new(),
            current_effects_by_identity: BTreeMap::new(),
            current_scrolls_by_identity: BTreeMap::new(),
            rebound_transforms: BTreeMap::new(),
            rebound_clips: BTreeMap::new(),
            rebound_effects: BTreeMap::new(),
            rebound_scrolls: BTreeMap::new(),
            visiting_effects: BTreeSet::new(),
        };
        for node in store.transforms.values() {
            result
                .current_transforms_by_identity
                .insert(PaintPropertyTrees::identity(&node.lifecycle), node.clone());
        }
        for node in store.clips.values() {
            result
                .current_clips_by_identity
                .insert(PaintPropertyTrees::identity(&node.lifecycle), node.clone());
        }
        for node in store.effects.values() {
            result
                .current_effects_by_identity
                .insert(PaintPropertyTrees::identity(&node.lifecycle), node.clone());
        }
        for node in store.scrolls.values() {
            result
                .current_scrolls_by_identity
                .insert(PaintPropertyTrees::identity(&node.lifecycle), node.clone());
        }
        result
    }

    pub fn RebindState(&mut self, state: &mut PropertyTreeState) {
        state.transform = self.RebindTransform(&state.transform);
        state.clip = self.clip(&state.clip);
        state.effect = self.effect(&state.effect);
    }

    pub fn RebindTransform(
        &mut self,
        input: &Arc<TransformPaintPropertyNode>,
    ) -> Arc<TransformPaintPropertyNode> {
        if input.id == 0 {
            return PropertyTreeState::default().transform;
        }
        let identity = PaintPropertyTrees::identity(&input.lifecycle);
        if let Some(node) = self.rebound_transforms.get(&identity) {
            return node.clone();
        }
        let source = self
            .current_transforms_by_identity
            .get(&identity)
            .cloned()
            .unwrap_or_else(|| input.clone());
        let source_identity = PaintPropertyTrees::identity(&source.lifecycle);
        let parent = source
            .parent
            .as_ref()
            .map(|parent| self.RebindTransform(parent));
        let scroll = source.scroll.as_ref().map(|scroll| self.scroll(scroll));
        let scroll_parent_scroll_translation = source
            .scroll_parent_scroll_translation
            .as_ref()
            .map(|parent| self.RebindTransform(parent));
        let node = if same_snapshot_option(&parent, &source.parent)
            && same_snapshot_option(&scroll, &source.scroll)
            && same_snapshot_option(
                &scroll_parent_scroll_translation,
                &source.scroll_parent_scroll_translation,
            ) {
            source
        } else {
            let mut node = (*source).clone();
            node.geometry_mapper_cache = Default::default();
            node.parent = parent;
            node.scroll = scroll;
            node.scroll_parent_scroll_translation = scroll_parent_scroll_translation;
            Arc::new(node)
        };
        self.rebound_transforms
            .insert(source_identity, node.clone());
        self.rebound_transforms.insert(identity, node.clone());
        node
    }

    fn clip(&mut self, input: &Arc<ClipPaintPropertyNode>) -> Arc<ClipPaintPropertyNode> {
        if input.id == 0 {
            return PropertyTreeState::default().clip;
        }
        let identity = PaintPropertyTrees::identity(&input.lifecycle);
        if let Some(node) = self.rebound_clips.get(&identity) {
            return node.clone();
        }
        let source = self
            .current_clips_by_identity
            .get(&identity)
            .cloned()
            .unwrap_or_else(|| input.clone());
        let source_identity = PaintPropertyTrees::identity(&source.lifecycle);
        let parent = source.parent.as_ref().map(|parent| self.clip(parent));
        let local_transform_space = self.RebindTransform(&source.local_transform_space);
        let pixel_moving_filter = source
            .pixel_moving_filter
            .as_ref()
            .map(|effect| self.effect(effect));
        let node = if same_snapshot_option(&parent, &source.parent)
            && Arc::ptr_eq(&local_transform_space, &source.local_transform_space)
            && same_snapshot_option(&pixel_moving_filter, &source.pixel_moving_filter)
        {
            source
        } else {
            let mut node = (*source).clone();
            node.geometry_mapper_cache = Default::default();
            node.parent = parent;
            node.local_transform_space = local_transform_space;
            node.pixel_moving_filter = pixel_moving_filter;
            Arc::new(node)
        };
        self.rebound_clips.insert(source_identity, node.clone());
        self.rebound_clips.insert(identity, node.clone());
        node
    }

    fn effect(&mut self, input: &Arc<EffectPaintPropertyNode>) -> Arc<EffectPaintPropertyNode> {
        if input.id == 0 {
            return PropertyTreeState::default().effect;
        }
        let identity = PaintPropertyTrees::identity(&input.lifecycle);
        if let Some(node) = self.rebound_effects.get(&identity) {
            return node.clone();
        }
        // A cross-tree wrapper may name an earlier immutable revision of the
        // effect currently being rebuilt. A strong-Arc cycle cannot represent
        // Chromium's in-place back-reference, so preserve that already-valid
        // inner snapshot while the outer snapshot is in flight.
        if !self.visiting_effects.insert(identity) {
            return input.clone();
        }
        let source = self
            .current_effects_by_identity
            .get(&identity)
            .cloned()
            .unwrap_or_else(|| input.clone());
        let source_identity = PaintPropertyTrees::identity(&source.lifecycle);
        let parent = source.parent.as_ref().map(|parent| self.effect(parent));
        let local_transform_space = self.RebindTransform(&source.local_transform_space);
        let output_clip = source.output_clip.as_ref().map(|clip| self.clip(clip));
        let backdrop_filter_clip = source
            .backdrop_filter_clip
            .as_ref()
            .map(|clip| self.clip(clip));
        let canvas_child_state = source.compositor.canvas_child_state.as_ref().map(|child| {
            let mut rebound = (**child).clone();
            self.RebindState(&mut rebound.content_state);
            if rebound.content_state.same_snapshots(&child.content_state) {
                child.clone()
            } else {
                Arc::new(rebound)
            }
        });
        let node = if same_snapshot_option(&parent, &source.parent)
            && Arc::ptr_eq(&local_transform_space, &source.local_transform_space)
            && same_snapshot_option(&output_clip, &source.output_clip)
            && same_snapshot_option(&backdrop_filter_clip, &source.backdrop_filter_clip)
            && same_snapshot_option(&canvas_child_state, &source.compositor.canvas_child_state)
        {
            source
        } else {
            let mut node = (*source).clone();
            node.parent = parent;
            node.local_transform_space = local_transform_space;
            node.output_clip = output_clip;
            node.backdrop_filter_clip = backdrop_filter_clip;
            node.compositor.canvas_child_state = canvas_child_state;
            Arc::new(node)
        };
        self.rebound_effects.insert(source_identity, node.clone());
        self.rebound_effects.insert(identity, node.clone());
        self.visiting_effects.remove(&identity);
        node
    }

    fn scroll(&mut self, input: &Arc<ScrollPaintPropertyNode>) -> Arc<ScrollPaintPropertyNode> {
        if input.id == 0 {
            return PropertyTreeState::default()
                .transform
                .scroll
                .as_ref()
                .expect("root transform owns root scroll")
                .clone();
        }
        let identity = PaintPropertyTrees::identity(&input.lifecycle);
        if let Some(node) = self.rebound_scrolls.get(&identity) {
            return node.clone();
        }
        let source = self
            .current_scrolls_by_identity
            .get(&identity)
            .cloned()
            .unwrap_or_else(|| input.clone());
        let source_identity = PaintPropertyTrees::identity(&source.lifecycle);
        let parent = source.parent.as_ref().map(|parent| self.scroll(parent));
        let overflow_clip = source.overflow_clip.as_ref().map(|clip| self.clip(clip));
        let node = if same_snapshot_option(&parent, &source.parent)
            && same_snapshot_option(&overflow_clip, &source.overflow_clip)
        {
            source
        } else {
            let mut node = (*source).clone();
            node.parent = parent;
            node.overflow_clip = overflow_clip;
            Arc::new(node)
        };
        self.rebound_scrolls.insert(source_identity, node.clone());
        self.rebound_scrolls.insert(identity, node.clone());
        node
    }
}

// One pass over the resident property graph, not the fragment tree. Memoized
// per-type IDs name existing directory entries; no new node identity is made.
struct RetainedScrollSnapshotRebinder {
    translations: BTreeMap<usize, (TransformMatrix, Option<StickyPositionPaintData>)>,
    trees: PaintPropertyTrees,
    changed: PaintPropertyChangeType,
}
impl RetainedScrollSnapshotRebinder {
    fn transform(
        &mut self,
        input: &Arc<TransformPaintPropertyNode>,
    ) -> Arc<TransformPaintPropertyNode> {
        if let Some(node) = self.trees.transform_for(input) {
            return node.clone();
        }
        let parent = input.parent.as_ref().map(|parent| self.transform(parent));
        let scroll = input.scroll.as_ref().map(|scroll| self.scroll(scroll));
        let scroll_parent = input
            .scroll_parent_scroll_translation
            .as_ref()
            .map(|parent| self.transform(parent));
        let (matrix, sticky_constraint) = self
            .translations
            .get(&PaintPropertyTrees::identity(&input.lifecycle))
            .copied()
            .unwrap_or((input.matrix, input.sticky_constraint));
        let node = if matrix == input.matrix
            && sticky_constraint == input.sticky_constraint
            && same_snapshot_option(&parent, &input.parent)
            && same_snapshot_option(&scroll, &input.scroll)
            && same_snapshot_option(&scroll_parent, &input.scroll_parent_scroll_translation)
            && input.lifecycle.node_changed() == PaintPropertyChangeType::Unchanged
        {
            input.clone()
        } else {
            let change = if sticky_constraint != input.sticky_constraint {
                PaintPropertyChangeType::ChangedOnlyValues
            } else if matrix != input.matrix {
                PaintPropertyChangeType::ChangedOnlySimpleValues
            } else {
                PaintPropertyChangeType::Unchanged
            };
            self.changed = self.changed.max(change);
            // Clone only retained payloads. The old parent/cross-tree/lifecycle
            // references would otherwise be cloned and immediately overwritten.
            Arc::new(TransformPaintPropertyNode {
                compositor: input.compositor.clone(),
                geometry_mapper_cache: Default::default(),
                scroll_parent_scroll_translation: scroll_parent,
                lifecycle: input.lifecycle.updated(change),
                id: input.id,
                parent,
                matrix,
                origin: input.origin,
                flattens_inherited_transform: input.flattens_inherited_transform,
                rendering_context_id: input.rendering_context_id,
                backface_visibility: input.backface_visibility,
                scroll,
                sticky_constraint,
                direct_compositing_reasons: input.direct_compositing_reasons.clone(),
            })
        };
        self.trees.insert_transform(node.clone());
        node
    }
    fn clip(&mut self, input: &Arc<ClipPaintPropertyNode>) -> Arc<ClipPaintPropertyNode> {
        if let Some(node) = self.trees.clip_for(input) {
            return node.clone();
        }
        let parent = input.parent.as_ref().map(|parent| self.clip(parent));
        let transform = self.transform(&input.local_transform_space);
        let filter = input
            .pixel_moving_filter
            .as_ref()
            .map(|filter| self.effect(filter));
        let node = if same_snapshot_option(&parent, &input.parent)
            && Arc::ptr_eq(&transform, &input.local_transform_space)
            && same_snapshot_option(&filter, &input.pixel_moving_filter)
            && input.lifecycle.node_changed() == PaintPropertyChangeType::Unchanged
        {
            input.clone()
        } else {
            Arc::new(ClipPaintPropertyNode {
                geometry_mapper_cache: Default::default(),
                layout_clip_rects: input.layout_clip_rects,
                lifecycle: input.lifecycle.updated(PaintPropertyChangeType::Unchanged),
                id: input.id,
                parent,
                local_transform_space: transform,
                rect: input.rect,
                radii: input.radii,
                clip_path: input.clip_path.clone(),
                clip_path_error: input.clip_path_error.clone(),
                pixel_moving_filter: filter,
            })
        };
        self.trees.insert_clip(node.clone());
        node
    }
    fn effect(&mut self, input: &Arc<EffectPaintPropertyNode>) -> Arc<EffectPaintPropertyNode> {
        if let Some(node) = self.trees.effect_for(input) {
            return node.clone();
        }
        let parent = input.parent.as_ref().map(|parent| self.effect(parent));
        let transform = self.transform(&input.local_transform_space);
        let clip = input.output_clip.as_ref().map(|clip| self.clip(clip));
        let backdrop_clip = input
            .backdrop_filter_clip
            .as_ref()
            .map(|clip| self.clip(clip));
        let canvas_child_state = input.compositor.canvas_child_state.as_ref().map(|child| {
            let mut rebound = (**child).clone();
            rebound.content_state.transform = self.transform(&rebound.content_state.transform);
            rebound.content_state.clip = self.clip(&rebound.content_state.clip);
            rebound.content_state.effect = self.effect(&rebound.content_state.effect);
            if rebound.content_state.same_snapshots(&child.content_state) {
                child.clone()
            } else {
                Arc::new(rebound)
            }
        });
        let node = if same_snapshot_option(&parent, &input.parent)
            && Arc::ptr_eq(&transform, &input.local_transform_space)
            && same_snapshot_option(&clip, &input.output_clip)
            && same_snapshot_option(&backdrop_clip, &input.backdrop_filter_clip)
            && same_snapshot_option(&canvas_child_state, &input.compositor.canvas_child_state)
            && input.lifecycle.node_changed() == PaintPropertyChangeType::Unchanged
        {
            input.clone()
        } else {
            let mut compositor = input.compositor.clone();
            compositor.canvas_child_state = canvas_child_state;
            Arc::new(EffectPaintPropertyNode {
                compositor,
                backdrop_filter_bounds: input.backdrop_filter_bounds.clone(),
                self_or_ancestor_participates_in_view_transition: input
                    .self_or_ancestor_participates_in_view_transition,
                lifecycle: input.lifecycle.updated(PaintPropertyChangeType::Unchanged),
                id: input.id,
                parent,
                local_transform_space: transform,
                output_clip: clip,
                filter_reference_box: input.filter_reference_box,
                filter_viewport: input.filter_viewport,
                filter_space_origin: input.filter_space_origin,
                filter_reference_scale: input.filter_reference_scale,
                opacity: input.opacity,
                blend_mode: input.blend_mode,
                filters: input.filters.clone(),
                backdrop_filters: input.backdrop_filters.clone(),
                backdrop_filter_has_reference: input.backdrop_filter_has_reference,
                backdrop_filter_clip: backdrop_clip,
                isolates_blending: input.isolates_blending,
                has_mask: input.has_mask,
                is_mask: input.is_mask,
                direct_compositing_reasons: input.direct_compositing_reasons.clone(),
            })
        };
        self.trees.insert_effect(node.clone());
        node
    }
    fn scroll(&mut self, input: &Arc<ScrollPaintPropertyNode>) -> Arc<ScrollPaintPropertyNode> {
        if let Some(node) = self.trees.scroll_for(input) {
            return node.clone();
        }
        let parent = input.parent.as_ref().map(|parent| self.scroll(parent));
        let clip = input.overflow_clip.as_ref().map(|clip| self.clip(clip));
        let node = if same_snapshot_option(&parent, &input.parent)
            && same_snapshot_option(&clip, &input.overflow_clip)
            && input.lifecycle.node_changed() == PaintPropertyChangeType::Unchanged
        {
            input.clone()
        } else {
            Arc::new(ScrollPaintPropertyNode {
                compositor: input.compositor.clone(),
                lifecycle: input.lifecycle.updated(PaintPropertyChangeType::Unchanged),
                id: input.id,
                owner_client_id: input.owner_client_id,
                parent,
                overflow_clip: clip,
                container_rect: input.container_rect,
                contents_rect: input.contents_rect,
                minimum_scroll_offset: input.minimum_scroll_offset,
                maximum_scroll_offset: input.maximum_scroll_offset,
                user_scrollable_horizontal: input.user_scrollable_horizontal,
                user_scrollable_vertical: input.user_scrollable_vertical,
            })
        };
        self.trees.insert_scroll(node.clone());
        node
    }
}

pub struct PropertyNodeUpdater<'a> {
    store: &'a mut PaintPropertyNodeStore,
    raw_keys: BTreeMap<u64, PaintPropertyKey>,
    prior_raw_keys: BTreeMap<u64, PaintPropertyKey>,
    stable_transform_keys: BTreeMap<u64, PaintPropertyKey>,
    stable_clip_keys: BTreeMap<u64, PaintPropertyKey>,
    stable_effect_keys: BTreeMap<u64, PaintPropertyKey>,
    stable_scroll_keys: BTreeMap<u64, PaintPropertyKey>,
    effect_sources: &'a BTreeMap<usize, Arc<EffectPaintPropertyNode>>,
    seen: BTreeSet<PaintPropertyKey>,
    transforms: BTreeMap<usize, Arc<TransformPaintPropertyNode>>,
    clips: BTreeMap<usize, Arc<ClipPaintPropertyNode>>,
    effects: BTreeMap<usize, Arc<EffectPaintPropertyNode>>,
    scrolls: BTreeMap<usize, Arc<ScrollPaintPropertyNode>>,
}

fn same_directory<T>(
    a: &BTreeMap<u64, Arc<T>>,
    b: &BTreeMap<u64, Arc<T>>,
    same: impl Fn(&T, &T) -> bool,
) -> bool {
    a.len() == b.len()
        && a.iter()
            .zip(b)
            .all(|((ka, a), (kb, b))| ka == kb && same(a, b))
}
fn same_snapshot_option<T>(a: &Option<Arc<T>>, b: &Option<Arc<T>>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => Arc::ptr_eq(a, b),
        _ => false,
    }
}
impl TransformPaintPropertyNode {
    fn same_snapshot_references(&self, other: &Self) -> bool {
        same_snapshot_option(&self.parent, &other.parent)
            && same_snapshot_option(&self.scroll, &other.scroll)
            && same_snapshot_option(
                &self.scroll_parent_scroll_translation,
                &other.scroll_parent_scroll_translation,
            )
            // Layout sticky geometry is an input association outside Chromium
            // TransformPaintPropertyNode::State. Preserve a new boundary value
            // without promoting the source lifecycle category.
            && self.sticky_constraint == other.sticky_constraint
    }
}
impl ClipPaintPropertyNode {
    fn same_snapshot_references(&self, other: &Self) -> bool {
        same_snapshot_option(&self.parent, &other.parent)
            && Arc::ptr_eq(&self.local_transform_space, &other.local_transform_space)
            && same_snapshot_option(&self.pixel_moving_filter, &other.pixel_moving_filter)
    }
}
impl EffectPaintPropertyNode {
    fn same_snapshot_references(&self, other: &Self) -> bool {
        same_snapshot_option(&self.parent, &other.parent)
            && Arc::ptr_eq(&self.local_transform_space, &other.local_transform_space)
            && same_snapshot_option(&self.output_clip, &other.output_clip)
            && same_snapshot_option(&self.backdrop_filter_clip, &other.backdrop_filter_clip)
            && same_snapshot_option(
                &self.compositor.canvas_child_state,
                &other.compositor.canvas_child_state,
            )
    }
}
impl ScrollPaintPropertyNode {
    fn same_snapshot_references(&self, other: &Self) -> bool {
        same_snapshot_option(&self.parent, &other.parent)
            && same_snapshot_option(&self.overflow_clip, &other.overflow_clip)
            // These are Rechrom package-boundary associations, not fields in
            // Chromium ScrollPaintPropertyNode::State. Keep a new value when
            // they change without promoting the source lifecycle category.
            && self.owner_client_id == other.owner_client_id
            && self.minimum_scroll_offset == other.minimum_scroll_offset
            && self.maximum_scroll_offset == other.maximum_scroll_offset
    }
}

// Each snapshot's parent and cross-tree links are canonicalized before its
// local state is compared. Compare node entities, not old ancestor values:
// Blink SetParent/State::ComputeChange compares pointers in these fields.
fn same_optional<T>(
    a: &Option<Arc<T>>,
    b: &Option<Arc<T>>,
    identity: impl Fn(&T) -> &PaintPropertyNodeLifecycle,
) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => identity(a).same_node(identity(b)),
        _ => false,
    }
}
/// Source: transform_paint_property_node.cc:18-70.
fn compute_transform_change(
    a: &TransformPaintPropertyNode,
    b: &TransformPaintPropertyNode,
    animation_state: TransformAnimationState,
) -> PaintPropertyChangeType {
    use PaintPropertyChangeType::*;
    let matrix_changed = a.matrix != b.matrix;
    let origin_changed = a.origin != b.origin;
    if !matrix_changed && !origin_changed {
        return Unchanged;
    }
    if animation_state.is_running_animation_on_compositor {
        return if origin_changed {
            ChangedOnlySimpleValues
        } else {
            ChangedOnlyCompositedValues
        };
    }
    if a.RequiresCompositingForStickyPosition() || a.RequiresCompositingForAnchorPosition() {
        debug_assert!(a.ResolvedMatrix().Preserves2dAxisAlignment());
        debug_assert!(b.ResolvedMatrix().Preserves2dAxisAlignment());
        return ChangedOnlyCompositedValues;
    }
    if matrix_changed {
        let a_matrix = a.ResolvedMatrix();
        let b_matrix = b.ResolvedMatrix();
        let keeps_axis_alignment = (a_matrix.Preserves2dAxisAlignment()
            && b_matrix.Preserves2dAxisAlignment())
            || (a_matrix.InverseOrIdentity() * b_matrix).Preserves2dAxisAlignment();
        if !keeps_axis_alignment {
            return ChangedOnlyValues;
        }
    }
    ChangedOnlySimpleValues
}

/// Source: transform_paint_property_node.cc:72-105 and
/// transform_paint_property_node.h:173-185. Parent participates through the
/// enclosing SetParent result, represented here by resident identity.
fn transform_state_change_with_animation(
    a: &TransformPaintPropertyNode,
    b: &TransformPaintPropertyNode,
    animation_state: TransformAnimationState,
) -> PaintPropertyChangeType {
    use PaintPropertyChangeType::*;
    if !same_optional(&a.parent, &b.parent, |n| &n.lifecycle)
        || !same_optional(&a.scroll, &b.scroll, |n| &n.lifecycle)
        || !same_optional(
            &a.scroll_parent_scroll_translation,
            &b.scroll_parent_scroll_translation,
            |n| &n.lifecycle,
        )
        || a.flattens_inherited_transform != b.flattens_inherited_transform
        || a.compositor.in_subtree_of_page_scale != b.compositor.in_subtree_of_page_scale
        || a.compositor.animation_is_axis_aligned != b.compositor.animation_is_axis_aligned
        || a.compositor.is_frame_paint_offset_translation
            != b.compositor.is_frame_paint_offset_translation
        || a.compositor.is_for_svg_child != b.compositor.is_for_svg_child
        || a.rendering_context_id != b.rendering_context_id
        || a.backface_visibility != b.backface_visibility
        || a.direct_compositing_reasons != b.direct_compositing_reasons
        || a.compositor.compositor_element_id != b.compositor.compositor_element_id
        || a.compositor.sticky_constraint != b.compositor.sticky_constraint
        || a.compositor.anchor_position_scroll_data != b.compositor.anchor_position_scroll_data
        || a.compositor.visible_frame_element_id != b.compositor.visible_frame_element_id
    {
        return ChangedOnlyValues;
    }
    compute_transform_change(a, b, animation_state)
}

fn transform_state_change(
    a: &TransformPaintPropertyNode,
    b: &TransformPaintPropertyNode,
) -> PaintPropertyChangeType {
    transform_state_change_with_animation(a, b, TransformAnimationState::default())
}
fn clip_state_change(
    a: &ClipPaintPropertyNode,
    b: &ClipPaintPropertyNode,
) -> PaintPropertyChangeType {
    if same_optional(&a.parent, &b.parent, |n| &n.lifecycle)
        && a.local_transform_space
            .lifecycle
            .same_node(&b.local_transform_space.lifecycle)
        && a.rect == b.rect
        && a.radii == b.radii
        && a.layout_clip_rects == b.layout_clip_rects
        && a.clip_path == b.clip_path
        && a.clip_path_error == b.clip_path_error
        && same_optional(&a.pixel_moving_filter, &b.pixel_moving_filter, |n| {
            &n.lifecycle
        })
    {
        PaintPropertyChangeType::Unchanged
    } else {
        PaintPropertyChangeType::ChangedOnlyValues
    }
}
// Source: effect_paint_property_node.cc:17-123. Filter and backdrop-filter
// changes retain the source's animation-aware composited-value downgrade.
fn filter_state_change(
    a: &EffectPaintPropertyNode,
    b: &EffectPaintPropertyNode,
    is_running_filter_animation_on_compositor: bool,
) -> PaintPropertyChangeType {
    use PaintPropertyChangeType::*;
    if !a.compositor.has_filter_info && !b.compositor.has_filter_info {
        return Unchanged;
    }
    if a.compositor.has_filter_info != b.compositor.has_filter_info
        || a.compositor.filter_output_bounds != b.compositor.filter_output_bounds
        || a.filters.origin_tainted() != b.filters.origin_tainted()
    {
        return ChangedOnlyValues;
    }
    if a.Filter() != b.Filter() {
        return if is_running_filter_animation_on_compositor {
            ChangedOnlyCompositedValues
        } else {
            ChangedOnlyValues
        };
    }
    Unchanged
}

fn backdrop_filter_state_change(
    a: &EffectPaintPropertyNode,
    b: &EffectPaintPropertyNode,
    is_running_backdrop_filter_animation_on_compositor: bool,
) -> PaintPropertyChangeType {
    use PaintPropertyChangeType::*;
    if !a.compositor.has_backdrop_filter_info && !b.compositor.has_backdrop_filter_info {
        return Unchanged;
    }
    if a.compositor.has_backdrop_filter_info != b.compositor.has_backdrop_filter_info
        || a.backdrop_filter_bounds != b.backdrop_filter_bounds
        || a.compositor.backdrop_mask_element_id != b.compositor.backdrop_mask_element_id
    {
        return ChangedOnlyValues;
    }
    if a.BackdropFilter() != b.BackdropFilter() {
        return if is_running_backdrop_filter_animation_on_compositor {
            ChangedOnlyCompositedValues
        } else {
            ChangedOnlyValues
        };
    }
    Unchanged
}

fn effect_state_change_with_animation(
    a: &EffectPaintPropertyNode,
    b: &EffectPaintPropertyNode,
    animation_state: EffectAnimationState,
) -> PaintPropertyChangeType {
    use PaintPropertyChangeType::*;
    if !same_optional(&a.parent, &b.parent, |n| &n.lifecycle)
        || !a
            .local_transform_space
            .lifecycle
            .same_node(&b.local_transform_space.lifecycle)
        || !same_optional(&a.output_clip, &b.output_clip, |n| &n.lifecycle)
        || a.blend_mode != b.blend_mode
        || a.filter_reference_box != b.filter_reference_box
        || a.filter_viewport != b.filter_viewport
        || a.filter_space_origin != b.filter_space_origin
        || a.filter_reference_scale != b.filter_reference_scale
        || a.backdrop_filter_has_reference != b.backdrop_filter_has_reference
        || a.compositor.compositor_element_id != b.compositor.compositor_element_id
        || a.compositor.view_transition_element_resource_id
            != b.compositor.view_transition_element_resource_id
        || a.compositor.restriction_target_id != b.compositor.restriction_target_id
        || a.compositor.needs_effect_for_2d_scale_transform
            != b.compositor.needs_effect_for_2d_scale_transform
        || a.compositor.is_in_tainted_subtree != b.compositor.is_in_tainted_subtree
        || a.compositor.is_in_drawable_canvas_subtree != b.compositor.is_in_drawable_canvas_subtree
        || a.compositor.canvas_child_state != b.compositor.canvas_child_state
        || !same_optional(&a.backdrop_filter_clip, &b.backdrop_filter_clip, |n| {
            &n.lifecycle
        })
        || a.isolates_blending != b.isolates_blending
        || a.has_mask != b.has_mask
        || a.is_mask != b.is_mask
        || a.self_or_ancestor_participates_in_view_transition
            != b.self_or_ancestor_participates_in_view_transition
        || a.direct_compositing_reasons != b.direct_compositing_reasons
    {
        return ChangedOnlyValues;
    }

    let opacity_changed = a.opacity != b.opacity;
    let opacity_change_is_simple = EffectPaintPropertyNode::IsOpacityChangeSimple(
        a.opacity,
        b.opacity,
        a.direct_compositing_reasons,
        b.direct_compositing_reasons,
    );
    if opacity_changed && !opacity_change_is_simple {
        return ChangedOnlyValues;
    }

    let filter_changed = filter_state_change(
        a,
        b,
        animation_state.is_running_filter_animation_on_compositor,
    );
    if filter_changed == ChangedOnlyValues {
        return ChangedOnlyValues;
    }
    let backdrop_filter_changed = backdrop_filter_state_change(
        a,
        b,
        animation_state.is_running_backdrop_filter_animation_on_compositor,
    );
    if backdrop_filter_changed == ChangedOnlyValues {
        return ChangedOnlyValues;
    }

    if opacity_change_is_simple && !animation_state.is_running_opacity_animation_on_compositor {
        return ChangedOnlySimpleValues;
    }
    if opacity_changed || filter_changed != Unchanged || backdrop_filter_changed != Unchanged {
        return ChangedOnlyCompositedValues;
    }
    Unchanged
}

fn effect_state_change(
    a: &EffectPaintPropertyNode,
    b: &EffectPaintPropertyNode,
) -> PaintPropertyChangeType {
    effect_state_change_with_animation(a, b, EffectAnimationState::default())
}
fn scroll_state_change(
    a: &ScrollPaintPropertyNode,
    b: &ScrollPaintPropertyNode,
) -> PaintPropertyChangeType {
    use PaintPropertyChangeType::*;
    // Source: scroll_paint_property_node.cc:31-55. Parent is compared by the
    // enclosing Update's SetParent and therefore participates in the maximum.
    if !same_optional(&a.parent, &b.parent, |n| &n.lifecycle)
        || !same_optional(&a.overflow_clip, &b.overflow_clip, |n| &n.lifecycle)
        || a.container_rect != b.container_rect
        || a.contents_rect != b.contents_rect
        || a.user_scrollable_horizontal != b.user_scrollable_horizontal
        || a.user_scrollable_vertical != b.user_scrollable_vertical
        || a.compositor.prevent_viewport_scrolling_from_inner
            != b.compositor.prevent_viewport_scrolling_from_inner
        || a.compositor.max_scroll_offset_affected_by_page_scale
            != b.compositor.max_scroll_offset_affected_by_page_scale
        || a.compositor.composited_scrolling_preference
            != b.compositor.composited_scrolling_preference
        || a.compositor.main_thread_repaint_reasons != b.compositor.main_thread_repaint_reasons
        || a.compositor.compositor_element_id != b.compositor.compositor_element_id
        || a.compositor.overscroll_behavior != b.compositor.overscroll_behavior
        || a.compositor.snap_container_data_absent != b.compositor.snap_container_data_absent
        || a.compositor.snap_container_data != b.compositor.snap_container_data
        || a.compositor.prevent_scroll_axis_locking != b.compositor.prevent_scroll_axis_locking
    {
        return ChangedOnlyValues;
    }
    if a.compositor.scrolling_contents_cull_rect != b.compositor.scrolling_contents_cull_rect {
        return ChangedOnlySimpleValues;
    }
    Unchanged
}

macro_rules! update_snapshot {
    ($self:ident, $node:ident, $key:expr, $old:ident, $change:ident, $map:ident) => {{
        if let Some(key) = $key {
            $self.seen.insert(key);
            if let Some($old) = $self.store.$map.get(&key) {
                let changed = $change($old, &$node);
                $node.id = $old.id;
                $node.lifecycle = $old.lifecycle.updated(changed);
            } else {
                $node.id = $self.store.next_id;
                $self.store.next_id += 1;
                $node.lifecycle = Default::default();
            }
            $self.store.changed = $self.store.changed.max($node.lifecycle.changed);
            if $node.lifecycle.changed != PaintPropertyChangeType::Unchanged {
                $self.store.change_markers_cleared = false;
            }
            let node = if let Some(old) = $self.store.$map.get(&key).filter(|old| {
                $node.lifecycle.changed == PaintPropertyChangeType::Unchanged
                    && old.lifecycle.node_changed() == PaintPropertyChangeType::Unchanged
                    && $node.same_snapshot_references(old)
            }) {
                old.clone()
            } else {
                Arc::new($node)
            };
            $self.store.$map.insert(key, node.clone());
            node
        } else {
            // No source owner (unsupported hand-built input): keep fresh
            // entities instead of inventing cross-frame client identities.
            $node.id = $self.store.next_id;
            $self.store.next_id += 1;
            $node.lifecycle = Default::default();
            $self.store.changed = PaintPropertyChangeType::NodeAddedOrRemoved;
            Arc::new($node)
        }
    }};
}
impl PropertyNodeUpdater<'_> {
    fn transform_key(&self, input: &TransformPaintPropertyNode) -> Option<PaintPropertyKey> {
        self.stable_transform_keys
            .get(&input.id)
            .copied()
            .filter(|key| {
                self.store
                    .transforms
                    .get(key)
                    .is_some_and(|node| input.lifecycle.same_node(&node.lifecycle))
            })
            .or_else(|| self.raw_keys.get(&input.id).copied())
            .or_else(|| self.prior_raw_keys.get(&input.id).copied())
    }

    fn clip_key(&self, input: &ClipPaintPropertyNode) -> Option<PaintPropertyKey> {
        self.stable_clip_keys
            .get(&input.id)
            .copied()
            .filter(|key| {
                self.store
                    .clips
                    .get(key)
                    .is_some_and(|node| input.lifecycle.same_node(&node.lifecycle))
            })
            .or_else(|| self.raw_keys.get(&input.id).copied())
            .or_else(|| self.prior_raw_keys.get(&input.id).copied())
    }

    fn effect_key(&self, input: &EffectPaintPropertyNode) -> Option<PaintPropertyKey> {
        self.stable_effect_keys
            .get(&input.id)
            .copied()
            .filter(|key| {
                self.store
                    .effects
                    .get(key)
                    .is_some_and(|node| input.lifecycle.same_node(&node.lifecycle))
            })
            .or_else(|| self.raw_keys.get(&input.id).copied())
            .or_else(|| self.prior_raw_keys.get(&input.id).copied())
    }

    fn scroll_key(&self, input: &ScrollPaintPropertyNode) -> Option<PaintPropertyKey> {
        self.stable_scroll_keys
            .get(&input.id)
            .copied()
            .filter(|key| {
                self.store
                    .scrolls
                    .get(key)
                    .is_some_and(|node| input.lifecycle.same_node(&node.lifecycle))
            })
            .or_else(|| self.raw_keys.get(&input.id).copied())
            .or_else(|| self.prior_raw_keys.get(&input.id).copied())
    }

    pub fn state(&mut self, state: &mut PropertyTreeState) {
        state.transform = self.transform(&state.transform);
        state.clip = self.clip(&state.clip);
        state.effect = self.effect(&state.effect);
    }
    fn transform(
        &mut self,
        input: &Arc<TransformPaintPropertyNode>,
    ) -> Arc<TransformPaintPropertyNode> {
        if input.id == 0 {
            return input.clone();
        }
        let identity = PaintPropertyTrees::identity(&input.lifecycle);
        if let Some(node) = self.transforms.get(&identity) {
            return node.clone();
        }
        // A retained clean subtree can still reference the preceding raw
        // snapshot of an ancestor whose value was rebuilt earlier in this
        // pass (most commonly ScrollTranslation during CullRectUpdater). The
        // source owner/role is the property slot identity. Once that slot has
        // been updated, every stale raw reference must bind to the new
        // canonical snapshot instead of replaying the old value over it.
        let key = self.transform_key(input);
        if let Some(key) = key {
            if self.seen.contains(&key) {
                return self
                    .store
                    .transforms
                    .get(&key)
                    .expect("seen transform property has a canonical snapshot")
                    .clone();
            }
        }
        let mut node = (**input).clone();
        node.parent = node.parent.as_ref().map(|p| self.transform(p));
        node.scroll = node.scroll.as_ref().map(|s| self.scroll(s));
        node.scroll_parent_scroll_translation = node
            .scroll_parent_scroll_translation
            .as_ref()
            .map(|s| self.transform(s));
        let node = update_snapshot!(self, node, key, old, transform_state_change, transforms);
        self.transforms.insert(identity, node.clone());
        node
    }
    fn clip(&mut self, input: &Arc<ClipPaintPropertyNode>) -> Arc<ClipPaintPropertyNode> {
        if input.id == 0 {
            return input.clone();
        }
        let identity = PaintPropertyTrees::identity(&input.lifecycle);
        if let Some(node) = self.clips.get(&identity) {
            return node.clone();
        }
        let key = self.clip_key(input);
        if let Some(key) = key {
            if self.seen.contains(&key) {
                return self
                    .store
                    .clips
                    .get(&key)
                    .expect("seen clip property has a canonical snapshot")
                    .clone();
            }
        }
        let mut node = (**input).clone();
        node.parent = node.parent.as_ref().map(|p| self.clip(p));
        node.local_transform_space = self.transform(&node.local_transform_space);
        node.pixel_moving_filter = node.pixel_moving_filter.as_ref().map(|f| self.effect(f));
        let node = update_snapshot!(self, node, key, old, clip_state_change, clips);
        self.clips.insert(identity, node.clone());
        node
    }
    fn effect(&mut self, input: &Arc<EffectPaintPropertyNode>) -> Arc<EffectPaintPropertyNode> {
        if input.id == 0 {
            return input.clone();
        }
        let identity = PaintPropertyTrees::identity(&input.lifecycle);
        if let Some(node) = self.effects.get(&identity) {
            return node.clone();
        }
        let key = self.effect_key(input);
        if let Some(key) = key {
            if self.seen.contains(&key) {
                return self
                    .store
                    .effects
                    .get(&key)
                    .expect("seen effect property has a canonical snapshot")
                    .clone();
            }
        }
        // A physical clip expander may still hold a pre-rebind effect snapshot.
        // Resolve its real node entity to the final logical effect ancestry.
        let source = self.effect_sources.get(&identity).unwrap_or(input);
        let mut node = (**source).clone();
        node.parent = node.parent.as_ref().map(|p| self.effect(p));
        node.local_transform_space = self.transform(&node.local_transform_space);
        node.output_clip = node.output_clip.as_ref().map(|c| self.clip(c));
        node.backdrop_filter_clip = node.backdrop_filter_clip.as_ref().map(|c| self.clip(c));
        if let Some(child) = node.compositor.canvas_child_state.as_ref() {
            let mut child = (**child).clone();
            self.state(&mut child.content_state);
            node.compositor.canvas_child_state = Some(Arc::new(child));
        }
        let node = update_snapshot!(self, node, key, old, effect_state_change, effects);
        self.effects.insert(identity, node.clone());
        node
    }
    fn scroll(&mut self, input: &Arc<ScrollPaintPropertyNode>) -> Arc<ScrollPaintPropertyNode> {
        if input.id == 0 {
            return input.clone();
        }
        let identity = PaintPropertyTrees::identity(&input.lifecycle);
        if let Some(node) = self.scrolls.get(&identity) {
            return node.clone();
        }
        let key = self.scroll_key(input);
        if let Some(key) = key {
            if self.seen.contains(&key) {
                return self
                    .store
                    .scrolls
                    .get(&key)
                    .expect("seen scroll property has a canonical snapshot")
                    .clone();
            }
        }
        let mut node = (**input).clone();
        node.parent = node.parent.as_ref().map(|p| self.scroll(p));
        node.overflow_clip = node.overflow_clip.as_ref().map(|c| self.clip(c));
        let node = update_snapshot!(self, node, key, old, scroll_state_change, scrolls);
        self.scrolls.insert(identity, node.clone());
        node
    }
    pub fn finish(self) {
        let before = self.store.transforms.len()
            + self.store.clips.len()
            + self.store.effects.len()
            + self.store.scrolls.len();
        self.store.transforms.retain(|k, _| self.seen.contains(k));
        self.store.clips.retain(|k, _| self.seen.contains(k));
        self.store.effects.retain(|k, _| self.seen.contains(k));
        self.store.scrolls.retain(|k, _| self.seen.contains(k));
        let after = self.store.transforms.len()
            + self.store.clips.len()
            + self.store.effects.len()
            + self.store.scrolls.len();
        if before != after {
            self.store.changed = PaintPropertyChangeType::NodeAddedOrRemoved;
        }
        self.store.change_markers_cleared =
            self.store.changed == PaintPropertyChangeType::Unchanged;
    }
}

#[cfg(test)]
mod retained_scroll_update_tests {
    use super::*;

    fn property_key(client: u64, role: PaintPropertyRole) -> PaintPropertyKey {
        PaintPropertyKey {
            owner: PaintPropertyOwner::Fragment {
                client,
                fragment: 0,
            },
            role,
        }
    }

    fn child_effect(
        id: u64,
        parent: Arc<EffectPaintPropertyNode>,
        transform: Arc<TransformPaintPropertyNode>,
    ) -> Arc<EffectPaintPropertyNode> {
        let mut effect = (*PropertyTreeState::default().effect).clone();
        effect.id = id;
        effect.lifecycle = Default::default();
        effect.parent = Some(parent);
        effect.local_transform_space = transform;
        Arc::new(effect)
    }

    #[test]
    fn artifact_rebinder_recurses_through_current_owner_snapshot() {
        let root = PropertyTreeState::default();
        let old_parent = child_effect(11, root.effect.clone(), root.transform.clone());
        let mut current_parent = (*old_parent).clone();
        current_parent.opacity = 0.75;
        current_parent.lifecycle = old_parent
            .lifecycle
            .updated(PaintPropertyChangeType::ChangedOnlyValues);
        let current_parent = Arc::new(current_parent);
        let old_child = child_effect(12, old_parent.clone(), root.transform.clone());
        let mut current_child = (*old_child).clone();
        current_child.opacity = 0.5;
        current_child.lifecycle = old_child
            .lifecycle
            .updated(PaintPropertyChangeType::ChangedOnlyValues);
        current_child.parent = Some(old_parent);
        let current_child = Arc::new(current_child);

        let mut store = PaintPropertyNodeStore::default();
        store.effects.insert(
            property_key(11, PaintPropertyRole::Effect),
            current_parent.clone(),
        );
        store.effects.insert(
            property_key(12, PaintPropertyRole::Effect),
            current_child.clone(),
        );
        let mut state = PropertyTreeState {
            effect: old_child,
            ..root
        };
        ArtifactPropertyStateRebinder::new(&store).RebindState(&mut state);

        assert!(state.effect.lifecycle.same_node(&current_child.lifecycle));
        assert_eq!(state.effect.opacity, 0.5);
        assert!(Arc::ptr_eq(
            state.effect.parent.as_ref().unwrap(),
            &current_parent
        ));
    }

    #[test]
    fn artifact_rebinder_ignores_dump_id_collision() {
        let root = PropertyTreeState::default();
        let owned = child_effect(7, root.effect.clone(), root.transform.clone());
        let mut wrapper = (*root.effect).clone();
        wrapper.id = 7;
        wrapper.lifecycle = Default::default();
        wrapper.parent = Some(root.effect.clone());
        wrapper.opacity = 0.25;
        let wrapper = Arc::new(wrapper);
        assert!(!wrapper.lifecycle.same_node(&owned.lifecycle));
        let mut store = PaintPropertyNodeStore::default();
        store
            .effects
            .insert(property_key(7, PaintPropertyRole::Effect), owned);
        let mut state = PropertyTreeState {
            effect: wrapper.clone(),
            ..root
        };
        ArtifactPropertyStateRebinder::new(&store).RebindState(&mut state);

        assert!(Arc::ptr_eq(&state.effect, &wrapper));
        assert_eq!(state.effect.opacity, 0.25);
    }

    #[test]
    fn artifact_rebinder_recurses_all_effect_cross_tree_state() {
        let root = PropertyTreeState::default();
        let mut old_transform = (*root.transform).clone();
        old_transform.id = 21;
        old_transform.lifecycle = Default::default();
        old_transform.parent = Some(root.transform.clone());
        let old_transform = Arc::new(old_transform);
        let mut current_transform = (*old_transform).clone();
        current_transform.matrix.values[12] = 8.0;
        current_transform.lifecycle = old_transform
            .lifecycle
            .updated(PaintPropertyChangeType::ChangedOnlySimpleValues);
        let current_transform = Arc::new(current_transform);

        let make_clip = |id, lifecycle: PaintPropertyNodeLifecycle| {
            let mut clip = (*root.clip).clone();
            clip.id = id;
            clip.lifecycle = lifecycle;
            clip.parent = Some(root.clip.clone());
            clip.local_transform_space = old_transform.clone();
            Arc::new(clip)
        };
        let old_output_clip = make_clip(22, Default::default());
        let old_backdrop_clip = make_clip(23, Default::default());
        let mut current_output_clip = (*old_output_clip).clone();
        current_output_clip.lifecycle = old_output_clip
            .lifecycle
            .updated(PaintPropertyChangeType::ChangedOnlyValues);
        current_output_clip.local_transform_space = current_transform.clone();
        let current_output_clip = Arc::new(current_output_clip);
        let mut current_backdrop_clip = (*old_backdrop_clip).clone();
        current_backdrop_clip.lifecycle = old_backdrop_clip
            .lifecycle
            .updated(PaintPropertyChangeType::ChangedOnlyValues);
        current_backdrop_clip.local_transform_space = current_transform.clone();
        let current_backdrop_clip = Arc::new(current_backdrop_clip);
        let old_parent = child_effect(24, root.effect.clone(), old_transform.clone());
        let mut current_parent = (*old_parent).clone();
        current_parent.lifecycle = old_parent
            .lifecycle
            .updated(PaintPropertyChangeType::ChangedOnlyValues);
        current_parent.local_transform_space = current_transform.clone();
        let current_parent = Arc::new(current_parent);

        let mut wrapper = (*root.effect).clone();
        wrapper.id = 25;
        wrapper.lifecycle = Default::default();
        wrapper.parent = Some(old_parent.clone());
        wrapper.local_transform_space = old_transform.clone();
        wrapper.output_clip = Some(old_output_clip.clone());
        wrapper.backdrop_filter_clip = Some(old_backdrop_clip.clone());
        wrapper.compositor.canvas_child_state = Some(Arc::new(PaintCanvasChildState {
            id: 9,
            paint_state: Default::default(),
            content_state: PropertyTreeState {
                transform: old_transform,
                clip: old_output_clip,
                effect: old_parent,
            },
        }));
        let wrapper = Arc::new(wrapper);

        let mut store = PaintPropertyNodeStore::default();
        store.transforms.insert(
            property_key(21, PaintPropertyRole::Transform),
            current_transform.clone(),
        );
        store.clips.insert(
            property_key(22, PaintPropertyRole::CssClip),
            current_output_clip.clone(),
        );
        store.clips.insert(
            property_key(23, PaintPropertyRole::BackdropFilterClip),
            current_backdrop_clip.clone(),
        );
        store.effects.insert(
            property_key(24, PaintPropertyRole::Effect),
            current_parent.clone(),
        );

        let mut state = PropertyTreeState {
            effect: wrapper.clone(),
            ..root
        };
        ArtifactPropertyStateRebinder::new(&store).RebindState(&mut state);

        assert!(state.effect.lifecycle.same_node(&wrapper.lifecycle));
        assert!(Arc::ptr_eq(
            state.effect.parent.as_ref().unwrap(),
            &current_parent
        ));
        assert!(Arc::ptr_eq(
            &state.effect.local_transform_space,
            &current_transform
        ));
        assert!(Arc::ptr_eq(
            state.effect.output_clip.as_ref().unwrap(),
            &current_output_clip
        ));
        assert!(Arc::ptr_eq(
            state.effect.backdrop_filter_clip.as_ref().unwrap(),
            &current_backdrop_clip
        ));
        let child = state.effect.compositor.canvas_child_state.as_ref().unwrap();
        assert!(Arc::ptr_eq(
            &child.content_state.transform,
            &current_transform
        ));
        assert!(Arc::ptr_eq(&child.content_state.clip, &current_output_clip));
        assert!(Arc::ptr_eq(&child.content_state.effect, &current_parent));
    }

    #[test]
    fn compositor_scroll_translation_list_requires_a_scroll_node() {
        let root = PropertyTreeState::default();
        let owner = |client| PaintPropertyOwner::Fragment {
            client,
            fragment: 0,
        };
        let translation = |id, scroll| {
            Arc::new(TransformPaintPropertyNode {
                compositor: Default::default(),
                geometry_mapper_cache: Default::default(),
                lifecycle: Default::default(),
                id,
                parent: Some(root.transform.clone()),
                matrix: TransformMatrix::default(),
                origin: [0.0; 3],
                flattens_inherited_transform: false,
                rendering_context_id: 0,
                backface_visibility: Default::default(),
                scroll,
                scroll_parent_scroll_translation: None,
                sticky_constraint: None,
                direct_compositing_reasons: Default::default(),
            })
        };
        let with_scroll = translation(1, root.transform.scroll.clone());
        let geometry_only = translation(2, None);
        let mut store = PaintPropertyNodeStore::default();
        store.transforms.insert(
            PaintPropertyKey {
                owner: owner(1),
                role: PaintPropertyRole::ScrollTranslation,
            },
            with_scroll.clone(),
        );
        store.transforms.insert(
            PaintPropertyKey {
                owner: owner(2),
                role: PaintPropertyRole::ScrollTranslation,
            },
            geometry_only,
        );

        let published = store
            .compositor_scroll_translations()
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(published.len(), 1);
        assert!(Arc::ptr_eq(&published[0], &with_scroll));
    }

    #[test]
    fn property_tree_state_owns_source_access_change_and_diagnostics() {
        let root = PropertyTreeState::Root();
        assert!(root.same_nodes(&PropertyTreeState::default()));
        assert!(!root.ChangedToRoot(PaintPropertyChangeType::ChangedOnlyValues));

        let mut child_transform = (*root.transform).clone();
        child_transform.id = 101;
        child_transform.parent = Some(root.transform.clone());
        child_transform.lifecycle = PaintPropertyNodeLifecycle::default()
            .updated(PaintPropertyChangeType::ChangedOnlyValues);
        child_transform.matrix.values[12] = 9.0;
        let child_transform = Arc::new(child_transform);
        let mut state = root.clone();
        state.SetTransform(child_transform.clone());
        assert!(Arc::ptr_eq(state.Transform(), &child_transform));
        assert!(state.Changed(PaintPropertyChangeType::ChangedOnlyValues, &root));
        assert!(state.ChangedToRoot(PaintPropertyChangeType::ChangedOnlyValues));

        let text = state.ToString();
        assert!(text.starts_with("t:"));
        assert!(text.contains(" c:"));
        assert!(text.contains(" e:"));
        let json = state.ToJSON();
        assert!(json["transform"].is_object());
        assert!(json["clip"].is_object());
        assert!(json["effect"].is_object());

        state.ClearChangedToRoot(17);
        assert_eq!(child_transform.lifecycle.changed_sequence_number(), 17);
        assert!(!state.ChangedToRoot(PaintPropertyChangeType::ChangedOnlyValues));
    }

    #[test]
    fn transform_node_owns_source_change_queries_and_json() {
        let root = PropertyTreeState::default();
        let mut transform = (*root.transform).clone();
        transform.id = 91;
        transform.parent = Some(root.transform.clone());
        transform.lifecycle = PaintPropertyNodeLifecycle::root();

        let mut scaled = transform.clone();
        scaled.matrix.values[0] = 2.0;
        assert_eq!(
            transform_state_change(&transform, &scaled),
            PaintPropertyChangeType::ChangedOnlySimpleValues
        );

        let mut skewed = transform.clone();
        skewed.matrix.values[4] = 0.5;
        assert_eq!(
            transform_state_change(&transform, &skewed),
            PaintPropertyChangeType::ChangedOnlyValues
        );
        assert_eq!(
            transform_state_change_with_animation(
                &transform,
                &skewed,
                TransformAnimationState {
                    is_running_animation_on_compositor: true,
                }
            ),
            PaintPropertyChangeType::ChangedOnlyCompositedValues
        );

        let mut origin_changed = transform.clone();
        origin_changed.origin = [2.0, 3.0, 0.0];
        assert_eq!(
            transform_state_change_with_animation(
                &transform,
                &origin_changed,
                TransformAnimationState {
                    is_running_animation_on_compositor: true,
                }
            ),
            PaintPropertyChangeType::ChangedOnlySimpleValues
        );

        let mut sticky = transform.clone();
        sticky
            .direct_compositing_reasons
            .Put(foundation::CompositingReason::kStickyPosition);
        let mut sticky_moved = sticky.clone();
        sticky_moved.matrix.values[12] = 10.0;
        assert_eq!(
            transform_state_change(&sticky, &sticky_moved),
            PaintPropertyChangeType::ChangedOnlyCompositedValues
        );

        let mut source_state_changed = transform.clone();
        source_state_changed.compositor.visible_frame_element_id =
            foundation::CompositorElementId::new(7);
        assert_eq!(
            transform_state_change(&transform, &source_state_changed),
            PaintPropertyChangeType::ChangedOnlyValues
        );
        let mut auxiliary_changed = transform.clone();
        auxiliary_changed.sticky_constraint = Some(StickyPositionPaintData::default());
        assert_eq!(
            transform_state_change(&transform, &auxiliary_changed),
            PaintPropertyChangeType::Unchanged
        );

        let mut translated = transform.clone();
        translated.matrix.values[12] = 4.0;
        translated.matrix.values[13] = 5.0;
        translated.flattens_inherited_transform = false;
        translated.compositor.in_subtree_of_page_scale = false;
        translated.backface_visibility = PaintBackfaceVisibility::Hidden;
        translated.rendering_context_id = 0x2a;
        translated
            .direct_compositing_reasons
            .Put(foundation::CompositingReason::kWillChangeTransform);
        let json = translated.ToJSON();
        assert_eq!(json["translation2d"], "4,5");
        assert_eq!(json["flattensInheritedTransform"], false);
        assert_eq!(json["in_subtree_of_page_scale"], false);
        assert_eq!(json["backface"], "hidden");
        assert_eq!(json["renderingContextId"], "2a");

        let mut resident = transform.clone();
        assert_eq!(
            resident.Update(
                root.transform.clone(),
                scaled,
                TransformAnimationState::default()
            ),
            PaintPropertyChangeType::ChangedOnlySimpleValues
        );
        assert_eq!(resident.id, 91);
        assert_eq!(resident.lifecycle.revision, 1);
        assert_eq!(resident.Matrix().values[0], 2.0);

        let mut direct_matrix = resident.matrix;
        direct_matrix.values[12] = 8.0;
        assert_eq!(
            resident.DirectlyUpdateTransformAndOrigin(
                direct_matrix,
                resident.origin,
                TransformAnimationState {
                    is_running_animation_on_compositor: true,
                }
            ),
            PaintPropertyChangeType::ChangedOnlyCompositedValues
        );
        assert_eq!(resident.Matrix().values[12], 8.0);
        resident.ClearChangedToRoot(8);
        assert_eq!(resident.lifecycle.changed_sequence_number(), 8);
    }

    #[test]
    fn scroll_node_owns_source_change_queries_and_json() {
        let root = PropertyTreeState::default();
        let root_scroll = root
            .transform
            .scroll
            .as_ref()
            .expect("root transform owns root scroll")
            .clone();
        let mut scroll = (*root_scroll).clone();
        scroll.id = 81;
        scroll.lifecycle = PaintPropertyNodeLifecycle::root();
        scroll.parent = Some(root_scroll.clone());
        scroll.overflow_clip = Some(root.clip.clone());
        scroll.container_rect = PaintRect {
            x: 1.0,
            y: 2.0,
            width: 30.0,
            height: 40.0,
        };
        scroll.contents_rect = PaintRect {
            x: 1.0,
            y: 2.0,
            width: 300.0,
            height: 400.0,
        };
        scroll.user_scrollable_horizontal = true;
        scroll.user_scrollable_vertical = true;
        scroll.compositor.main_thread_repaint_reasons = 1 | (1 << 3);
        scroll.compositor.max_scroll_offset_affected_by_page_scale = true;
        scroll.compositor.prevent_scroll_axis_locking = true;
        scroll.compositor.overscroll_behavior = [
            LayoutOverscrollBehavior::kContain,
            LayoutOverscrollBehavior::kNone,
        ];
        scroll.compositor.snap_container_data_absent = true;
        scroll.compositor.snap_container_data = None;
        scroll.compositor.compositor_element_id = foundation::CompositorElementId::new(42);

        assert_eq!(scroll.ContainerRect().width, 30.0);
        assert_eq!(scroll.ContentsRect().height, 400.0);
        assert!(scroll.UserScrollable());
        assert!(scroll.RequiresMainThreadForBackgroundAttachmentFixed());
        assert!(scroll.PreventScrollAxisLocking());
        assert!(scroll
            .OverflowClipNode()
            .is_some_and(|clip| Arc::ptr_eq(clip, &root.clip)));

        let mut cull_changed = scroll.clone();
        cull_changed.compositor.scrolling_contents_cull_rect = PaintRect {
            x: 3.0,
            y: 4.0,
            width: 50.0,
            height: 60.0,
        };
        assert_eq!(
            scroll_state_change(&scroll, &cull_changed),
            PaintPropertyChangeType::ChangedOnlySimpleValues
        );
        let mut auxiliary_changed = scroll.clone();
        auxiliary_changed.maximum_scroll_offset.y = 99.0;
        assert_eq!(
            scroll_state_change(&scroll, &auxiliary_changed),
            PaintPropertyChangeType::Unchanged
        );
        let mut source_changed = scroll.clone();
        source_changed
            .compositor
            .prevent_viewport_scrolling_from_inner = true;
        assert_eq!(
            scroll_state_change(&scroll, &source_changed),
            PaintPropertyChangeType::ChangedOnlyValues
        );

        let json = scroll.ToJSON();
        assert_eq!(json["containerRect"], "1,2 30x40");
        assert_eq!(json["contentsRect"], "1,2 300x400");
        assert_eq!(json["userScrollable"], "both");
        assert_eq!(json["maxScrollOffsetAffectedByPageScale"], "true");
        assert_eq!(json["overscroll-behavior-x"], "contain");
        assert_eq!(json["overscroll-behavior-y"], "none");
        assert_eq!(json["preventScrollAxisLock"], true);
        assert_eq!(
            json["mainThreadReasons"],
            "Has background-attachment:fixed, Background needs repaint on scroll"
        );

        let mut resident = scroll.clone();
        let mut next = resident.clone();
        next.user_scrollable_vertical = false;
        assert_eq!(
            resident.Update(root_scroll, next),
            PaintPropertyChangeType::ChangedOnlyValues
        );
        assert_eq!(resident.id, 81);
        assert_eq!(resident.lifecycle.revision, 1);
        resident.SetScrollingContentsCullRect(PaintRect::default());
        assert_eq!(resident.lifecycle.revision, 2);
        assert_eq!(
            resident.lifecycle.node_changed(),
            PaintPropertyChangeType::ChangedOnlyValues
        );
        resident.ClearChangedToRoot(7);
        assert_eq!(resident.lifecycle.changed_sequence_number(), 7);
    }

    #[test]
    fn clip_node_owns_source_clip_queries_and_json() {
        let root = PropertyTreeState::default();
        let mut clip = (*root.clip).clone();
        clip.rect = Some(PaintRect {
            x: 1.0,
            y: 2.0,
            width: 30.0,
            height: 40.0,
        });
        let precise = PaintRect {
            x: 3.0,
            y: 4.0,
            width: 20.0,
            height: 25.0,
        };
        let expanded = PaintRect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 120.0,
        };
        clip = clip.with_layout_clip_values(Some(precise), Some(expanded), None);
        clip.pixel_moving_filter = Some(root.effect.clone());

        assert_eq!(clip.PaintClipRect(), clip.rect);
        assert_eq!(clip.PreciseLayoutClipRect().rect, Some(precise));
        assert_eq!(clip.LayoutClipRect().rect, Some(expanded));
        assert_eq!(
            clip.LayoutClipRectExcludingOverlayScrollbars().rect,
            Some(precise)
        );
        assert!(clip.IsForCompositeClipPathAnimation());
        assert!(clip.ClipPathEquals(None));
        assert!(clip
            .PixelMovingFilter()
            .is_some_and(|filter| Arc::ptr_eq(filter, &root.effect)));

        let json = clip.ToJSON();
        assert_eq!(json["rect"], "1,2 30x40");
        assert_eq!(json["isForCompositeClipPathAnimation"], true);
        assert_eq!(json["expandedLayoutClipRect"], "0,0 100x120");
        assert_eq!(json["preciseLayoutClipRect"], "3,4 20x25");
        assert!(json["pixelMovingFilter"].is_string());

        let mut resident = clip.clone();
        resident.id = 73;
        resident.lifecycle = PaintPropertyNodeLifecycle::root();
        let mut next = resident.clone();
        next.rect = Some(PaintRect {
            x: 5.0,
            y: 6.0,
            width: 7.0,
            height: 8.0,
        });
        assert_eq!(
            resident.Update(root.clip.clone(), next),
            PaintPropertyChangeType::ChangedOnlyValues
        );
        assert_eq!(resident.id, 73);
        assert_eq!(resident.lifecycle.revision, 1);
        assert_eq!(resident.PaintClipRect().expect("updated clip rect").x, 5.0);
        assert!(resident
            .parent
            .as_ref()
            .is_some_and(|parent| Arc::ptr_eq(parent, &root.clip)));
    }

    #[test]
    fn effect_node_owns_source_effect_predicates() {
        let root = PropertyTreeState::default();
        let mut effect = (*root.effect).clone();

        assert!(!effect.HasRealEffects());
        assert!(effect.IsOpacityOnly());
        assert!(!effect.HasDirectCompositingReasons());
        assert!(!effect.MayHaveOpacity());
        assert!(!effect.MayHaveFilter());
        assert!(!effect.MayHaveBackdropFilter());
        assert!(!effect.MayHaveBackdropEffect());
        assert!(!effect.DrawsContent());

        effect.opacity = 0.5;
        assert!(effect.HasRealEffects());
        assert!(effect.IsOpacityOnly());
        assert!(effect.MayHaveOpacity());

        effect
            .direct_compositing_reasons
            .Put(foundation::CompositingReason::kWillChangeFilter);
        assert!(effect.HasDirectCompositingReasons());
        assert!(effect.RequiresCompositingForWillChangeFilter());
        assert!(effect.MayHaveFilter());
        assert!(effect.NeedsPixelMovingFilterClipExpander());
        assert!(effect.DrawsContent());

        effect
            .direct_compositing_reasons
            .Put(foundation::CompositingReason::kWillChangeBackdropFilter);
        assert!(effect.RequiresCompositingForWillChangeBackdropFilter());
        assert!(effect.MayHaveBackdropFilter());
        assert!(effect.MayHaveBackdropEffect());

        effect.compositor.view_transition_element_resource_id = Some(([1, 2], 0, false));
        assert!(!effect.HasValidViewTransitionElementResourceId());
        effect.compositor.view_transition_element_resource_id = Some(([1, 2], 7, false));
        assert!(effect.HasValidViewTransitionElementResourceId());
        effect.compositor.restriction_target_id = [1, 0];
        assert!(effect.HasElementCaptureId());

        effect.compositor.canvas_child_state = Some(Arc::new(PaintCanvasChildState {
            id: 0,
            paint_state: CanvasChildPaintState::default(),
            content_state: root.clone(),
        }));
        assert!(!effect.HasCanvasChildState());
        assert_eq!(effect.CanvasChildId(), 0);
        effect.compositor.canvas_child_state = Some(Arc::new(PaintCanvasChildState {
            id: 9,
            paint_state: CanvasChildPaintState {
                reference_box_offset: Offset { x: 3.0, y: 4.0 },
                canvas_content_size: [100.0, 100.0],
                canvas_device_pixel_content_box: [200, 300],
                ..CanvasChildPaintState::default()
            },
            content_state: root.clone(),
        }));
        assert!(effect.HasCanvasChildState());
        assert_eq!(effect.CanvasChildId(), 9);
        assert_eq!(
            effect
                .canvas_child_paint_state()
                .expect("canvas child paint state")
                .reference_box_offset,
            Offset { x: 3.0, y: 4.0 }
        );
        let canvas_state = effect
            .canvas_child_paint_state()
            .expect("canvas child paint state");
        assert_eq!(
            canvas_state.GetCanvasGridScaleFactor([200, 300]),
            [1.0, 1.0]
        );
        assert_eq!(
            canvas_state.GetCanvasGridScaleFactor([100, 300]),
            [1.0, 3.0]
        );
        let mut same_value = canvas_state.clone();
        same_value.canvas_child_node_id = 99;
        assert_eq!(*canvas_state, same_value);
        let mut transform_state = canvas_state.clone();
        transform_state.effective_zoom = 2.0;
        transform_state.transform_origin = [2.0, 3.0, 0.0];
        let mut draw_values = TransformMatrix::default().values;
        draw_values[0] = 2.0;
        draw_values[5] = 3.0;
        let draw = foundation::gfx::Transform::ColMajor(&draw_values);
        let enabled = transform_state.GetElementTransform([200, 300], &draw, true);
        let around_origin = transform_state.GetElementTransform([200, 300], &draw, false);
        assert_eq!(enabled.GetColMajor()[12], 0.0);
        assert_eq!(enabled.GetColMajor()[13], 0.0);
        assert_eq!(around_origin.GetColMajor()[12], 2.0);
        assert_eq!(around_origin.GetColMajor()[13], 6.0);
        assert!(Arc::ptr_eq(
            effect.CanvasChildContentTransform(),
            &root.transform
        ));
        assert!(Arc::ptr_eq(effect.CanvasChildContentClip(), &root.clip));
        assert!(Arc::ptr_eq(effect.CanvasChildContentEffect(), &root.effect));

        effect.blend_mode = PaintBlendMode::kMultiply;
        assert!(!effect.IsOpacityOnly());
        assert!(effect.MayHaveBackdropEffect());

        effect.compositor.is_in_tainted_subtree = true;
        effect.compositor.is_in_drawable_canvas_subtree = true;
        effect.compositor.needs_effect_for_2d_scale_transform = true;
        assert!(effect.IsInTaintedSubtree());
        assert!(effect.IsInDrawableCanvasSubtree());
        assert!(effect.NeedsEffectFor2DScaleTransform());

        effect.compositor.compositor_element_id = foundation::CompositorElementId::new(42);
        let json = effect.ToJSON();
        assert_eq!(json["opacity"], serde_json::json!(0.5));
        assert_eq!(json["blendMode"], "Multiply");
        assert_eq!(json["compositorElementId"], "(42)");
        assert_eq!(json["is_in_drawable_canvas_subtree"], true);
        assert!(json["directCompositingReasons"]
            .as_str()
            .is_some_and(|reasons| reasons.contains("WillChangeFilter")));
        assert!(json["localTransformSpace"].is_string());
        assert!(json["outputClip"].is_string());
    }

    #[test]
    fn effect_change_keeps_animation_aware_source_categories() {
        use crate::cc::paint::filter_operation::FilterOperation;
        use crate::cc::paint::filter_operations::FilterOperations;

        fn blur(radius: f32, origin_tainted: bool) -> PropertyFilterOperations {
            PropertyFilterOperations::Ready {
                operations: FilterOperations::new(vec![FilterOperation::CreateBlurFilter(radius)]),
                origin_tainted,
            }
        }

        assert_eq!(
            PaintPropertyChangeType::ChangedOnlySimpleValues.to_string(),
            "simple-values"
        );
        let initial = PaintPropertyNodeLifecycle::default();
        initial.clear_changed(1);
        let simple = initial.updated(PaintPropertyChangeType::ChangedOnlySimpleValues);
        assert_eq!(
            simple.node_changed(),
            PaintPropertyChangeType::ChangedOnlySimpleValues
        );
        simple.CompositorSimpleValuesUpdated();
        assert_eq!(
            simple.node_changed(),
            PaintPropertyChangeType::ChangedOnlyCompositedValues
        );
        let initial = PaintPropertyNodeLifecycle::default();
        initial.clear_changed(1);
        let values = initial.updated(PaintPropertyChangeType::ChangedOnlyValues);
        values.CompositorSimpleValuesUpdated();
        assert_eq!(
            values.node_changed(),
            PaintPropertyChangeType::ChangedOnlyValues
        );

        let root = PropertyTreeState::default();
        let mut opacity_a = (*root.effect).clone();
        opacity_a.opacity = 0.4;
        let mut opacity_b = opacity_a.clone();
        opacity_b.opacity = 0.5;
        assert_eq!(
            effect_state_change_with_animation(
                &opacity_a,
                &opacity_b,
                EffectAnimationState::default()
            ),
            PaintPropertyChangeType::ChangedOnlySimpleValues
        );
        assert_eq!(
            effect_state_change_with_animation(
                &opacity_a,
                &opacity_b,
                EffectAnimationState {
                    is_running_opacity_animation_on_compositor: true,
                    ..EffectAnimationState::default()
                }
            ),
            PaintPropertyChangeType::ChangedOnlyCompositedValues
        );

        let mut filter_a = (*root.effect).clone();
        filter_a.filters = blur(1.0, false);
        filter_a = filter_a.with_filter_presence();
        let mut filter_b = filter_a.clone();
        filter_b.filters = blur(2.0, false);
        filter_b = filter_b.with_filter_presence();
        assert_eq!(
            effect_state_change_with_animation(
                &filter_a,
                &filter_b,
                EffectAnimationState::default()
            ),
            PaintPropertyChangeType::ChangedOnlyValues
        );
        assert_eq!(
            effect_state_change_with_animation(
                &filter_a,
                &filter_b,
                EffectAnimationState {
                    is_running_filter_animation_on_compositor: true,
                    ..EffectAnimationState::default()
                }
            ),
            PaintPropertyChangeType::ChangedOnlyCompositedValues
        );

        filter_b.filters = blur(1.0, true);
        assert_eq!(
            effect_state_change_with_animation(
                &filter_a,
                &filter_b,
                EffectAnimationState {
                    is_running_filter_animation_on_compositor: true,
                    ..EffectAnimationState::default()
                }
            ),
            PaintPropertyChangeType::ChangedOnlyValues
        );

        let mut backdrop_a = (*root.effect).clone();
        backdrop_a.backdrop_filters = blur(1.0, false);
        backdrop_a = backdrop_a.with_filter_presence();
        let mut backdrop_b = backdrop_a.clone();
        backdrop_b.backdrop_filters = blur(2.0, false);
        backdrop_b = backdrop_b.with_filter_presence();
        assert_eq!(
            effect_state_change_with_animation(
                &backdrop_a,
                &backdrop_b,
                EffectAnimationState {
                    is_running_backdrop_filter_animation_on_compositor: true,
                    ..EffectAnimationState::default()
                }
            ),
            PaintPropertyChangeType::ChangedOnlyCompositedValues
        );

        let mut direct = (*root.effect).clone();
        direct.lifecycle = PaintPropertyNodeLifecycle::root();
        direct.opacity = 0.4;
        assert_eq!(
            direct.ComputeOpacityChange(0.5, EffectAnimationState::default()),
            PaintPropertyChangeType::ChangedOnlySimpleValues
        );
        assert_eq!(
            direct.DirectlyUpdateOpacity(
                0.5,
                EffectAnimationState {
                    is_running_opacity_animation_on_compositor: true,
                    ..EffectAnimationState::default()
                }
            ),
            PaintPropertyChangeType::ChangedOnlyCompositedValues
        );
        assert_eq!(direct.opacity, 0.5);
        assert_eq!(direct.lifecycle.revision, 1);
        assert_eq!(
            direct.lifecycle.node_changed(),
            PaintPropertyChangeType::ChangedOnlyCompositedValues
        );
        assert_eq!(
            direct.DirectlyUpdateOpacity(0.5, EffectAnimationState::default()),
            PaintPropertyChangeType::Unchanged
        );
        assert_eq!(direct.lifecycle.revision, 1);

        let mut resident = (*root.effect).clone();
        resident.id = 77;
        resident.parent = Some(root.effect.clone());
        resident.lifecycle = PaintPropertyNodeLifecycle::root();
        let mut next_state = resident.clone();
        next_state.opacity = 0.25;
        assert_eq!(
            resident.Update(
                root.effect.clone(),
                next_state,
                EffectAnimationState::default()
            ),
            PaintPropertyChangeType::ChangedOnlyValues
        );
        assert_eq!(resident.opacity, 0.25);
        assert_eq!(resident.id, 77);
        assert_eq!(resident.lifecycle.revision, 1);
        assert!(resident
            .parent
            .as_ref()
            .is_some_and(|parent| Arc::ptr_eq(parent, &root.effect)));
    }

    #[test]
    fn rebuilt_slot_rebinds_retained_stable_id_reference() {
        fn transform(id: u64, y: f64) -> Arc<TransformPaintPropertyNode> {
            let root = PropertyTreeState::default().transform;
            let mut node = (*root).clone();
            let mut matrix = TransformMatrix::default();
            matrix.values[13] = y;
            node.lifecycle = Default::default();
            node.id = id;
            node.parent = Some(root);
            node.matrix = matrix;
            Arc::new(node)
        }

        let key = PaintPropertyKey {
            owner: PaintPropertyOwner::Fragment {
                client: 9,
                fragment: 0,
            },
            role: PaintPropertyRole::ScrollTranslation,
        };
        let effects = BTreeMap::new();
        let no_prior_keys = BTreeMap::new();
        let mut store = PaintPropertyNodeStore::default();

        let mut first = PropertyTreeState {
            transform: transform(10, 0.0),
            ..PropertyTreeState::default()
        };
        let mut retained_raw = first.clone();
        let mut first_keys = BTreeMap::new();
        first_keys.insert(10, key);
        let mut updater = store.updater(&first_keys, &no_prior_keys, &effects);
        updater.state(&mut first);
        updater.finish();
        let mut retained = first.clone();
        let stable_id = retained.transform.id;

        let mut rebuilt = PropertyTreeState {
            transform: transform(20, -4000.0),
            ..PropertyTreeState::default()
        };
        // Raw builder labels and stable store labels use independent
        // allocators. Exercise the collision explicitly: this fresh raw node
        // has the same number as the retained stable reference above, but it
        // belongs to a different owner slot and has a different lifecycle.
        let other_key = PaintPropertyKey {
            owner: PaintPropertyOwner::Fragment {
                client: 10,
                fragment: 0,
            },
            role: PaintPropertyRole::Transform,
        };
        let mut collided = PropertyTreeState {
            transform: transform(stable_id, 123.0),
            ..PropertyTreeState::default()
        };
        let mut rebuilt_keys = BTreeMap::new();
        rebuilt_keys.insert(20, key);
        rebuilt_keys.insert(stable_id, other_key);
        let mut updater = store.updater(&rebuilt_keys, &first_keys, &effects);
        updater.state(&mut rebuilt);
        updater.state(&mut collided);
        updater.state(&mut retained);
        updater.state(&mut retained_raw);
        updater.finish();

        assert_eq!(rebuilt.transform.id, stable_id);
        assert!(Arc::ptr_eq(&rebuilt.transform, &retained.transform));
        assert_eq!(retained.transform.matrix.values[13], -4000.0);
        assert!(Arc::ptr_eq(&rebuilt.transform, &retained_raw.transform));
        assert_eq!(retained_raw.transform.matrix.values[13], -4000.0);
        assert!(!Arc::ptr_eq(&rebuilt.transform, &collided.transform));
        assert_ne!(collided.transform.id, stable_id);
        assert_eq!(collided.transform.matrix.values[13], 123.0);
    }

    #[test]
    fn clearing_resident_states_includes_nodes_without_source_owner_slots() {
        let root = PropertyTreeState::default();
        let clip = Arc::new(
            ClipPaintPropertyNode {
                geometry_mapper_cache: Default::default(),
                layout_clip_rects: Default::default(),
                lifecycle: Default::default(),
                id: 17,
                parent: Some(root.clip.clone()),
                local_transform_space: root.transform.clone(),
                rect: Some(PaintRect {
                    x: 2.0,
                    y: 3.0,
                    width: 40.0,
                    height: 50.0,
                }),
                radii: Default::default(),
                clip_path: None,
                clip_path_error: None,
                pixel_moving_filter: None,
            }
            .with_layout_clips(),
        );
        let mut state = PropertyTreeState { clip, ..root };
        let mut store = PaintPropertyNodeStore::default();
        let keys = BTreeMap::new();
        let effects = BTreeMap::new();
        let prior_keys = BTreeMap::new();
        let mut updater = store.updater(&keys, &prior_keys, &effects);
        updater.state(&mut state);
        updater.finish();
        let previous = state.clone();
        let (_, mut trees) = store.cleared_change_markers().unwrap();
        assert!(!trees.clips.contains_key(&state.clip.id));
        trees.clear_state_change_markers(&mut state);
        assert_eq!(state.clip.id, previous.clip.id);
        assert_eq!(state.clip.rect, previous.clip.rect);
        assert!(state.clip.lifecycle.same_node(&previous.clip.lifecycle));
        assert_eq!(
            state.clip.lifecycle.revision,
            previous.clip.lifecycle.revision
        );
        assert_eq!(
            state.clip.lifecycle.changed,
            PaintPropertyChangeType::Unchanged
        );
        assert_eq!(
            previous.clip.lifecycle.changed,
            PaintPropertyChangeType::NodeAddedOrRemoved
        );
        let shared = state.clip.clone();
        trees.clear_state_change_markers(&mut state);
        assert!(Arc::ptr_eq(&shared, &state.clip));
    }

    #[test]
    fn direct_scroll_rebinds_cross_links_without_invalidating_static_nodes() {
        let root = PropertyTreeState::default();
        let owner = PaintPropertyOwner::Fragment {
            client: 42,
            fragment: 0,
        };
        let key = |role| PaintPropertyKey { owner, role };
        let clean =
            || PaintPropertyNodeLifecycle::default().updated(PaintPropertyChangeType::Unchanged);
        let scroll = Arc::new(ScrollPaintPropertyNode {
            compositor: Default::default(),
            id: 1,
            owner_client_id: 42,
            lifecycle: clean(),
            parent: root.transform.scroll.clone(),
            overflow_clip: Some(root.clip.clone()),
            container_rect: PaintRect::default(),
            contents_rect: PaintRect::default(),
            minimum_scroll_offset: Offset::default(),
            maximum_scroll_offset: Offset { x: 0.0, y: 100.0 },
            user_scrollable_horizontal: false,
            user_scrollable_vertical: true,
        });
        let translation = Arc::new(TransformPaintPropertyNode {
            compositor: Default::default(),
            geometry_mapper_cache: Default::default(),
            scroll_parent_scroll_translation: None,
            id: 2,
            lifecycle: clean(),
            parent: Some(root.transform.clone()),
            matrix: TransformMatrix::default(),
            origin: [0.0; 3],
            flattens_inherited_transform: false,
            rendering_context_id: 0,
            backface_visibility: Default::default(),
            scroll: Some(scroll.clone()),
            sticky_constraint: None,
            direct_compositing_reasons: Default::default(),
        });
        let child = Arc::new(TransformPaintPropertyNode {
            compositor: Default::default(),
            geometry_mapper_cache: Default::default(),
            scroll_parent_scroll_translation: None,
            id: 3,
            lifecycle: clean(),
            parent: Some(translation.clone()),
            matrix: TransformMatrix::default(),
            origin: [0.0; 3],
            flattens_inherited_transform: false,
            rendering_context_id: 0,
            backface_visibility: Default::default(),
            scroll: None,
            sticky_constraint: None,
            direct_compositing_reasons: Default::default(),
        });
        let clip = Arc::new(
            ClipPaintPropertyNode {
                geometry_mapper_cache: Default::default(),
                layout_clip_rects: Default::default(),
                id: 4,
                lifecycle: clean().updated(PaintPropertyChangeType::ChangedOnlyValues),
                parent: Some(root.clip.clone()),
                local_transform_space: child.clone(),
                rect: None,
                radii: PaintCornerRadii::default(),
                clip_path: None,
                clip_path_error: None,
                pixel_moving_filter: None,
            }
            .with_layout_clips(),
        );
        let effect = Arc::new(
            EffectPaintPropertyNode {
                compositor: Default::default(),
                backdrop_filter_bounds: None,
                self_or_ancestor_participates_in_view_transition: false,
                id: 5,
                lifecycle: clean(),
                parent: Some(root.effect.clone()),
                local_transform_space: child.clone(),
                output_clip: Some(clip.clone()),
                filter_reference_box: None,
                filter_viewport: None,
                filter_space_origin: Offset::default(),
                filter_reference_scale: 1.0,
                opacity: 0.5,
                blend_mode: PaintBlendMode::kNormal,
                filters: Default::default(),
                backdrop_filters: Default::default(),
                backdrop_filter_has_reference: false,
                backdrop_filter_clip: None,
                isolates_blending: false,
                has_mask: false,
                is_mask: false,
                direct_compositing_reasons: Default::default(),
            }
            .with_inherited_view_transition(),
        );
        let secondary_scroll = Arc::new(ScrollPaintPropertyNode {
            compositor: Default::default(),
            id: 6,
            owner_client_id: 43,
            lifecycle: clean(),
            parent: Some(scroll.clone()),
            overflow_clip: Some(clip.clone()),
            container_rect: PaintRect::default(),
            contents_rect: PaintRect::default(),
            minimum_scroll_offset: Offset::default(),
            maximum_scroll_offset: Offset::default(),
            user_scrollable_horizontal: false,
            user_scrollable_vertical: false,
        });
        let mut store = PaintPropertyNodeStore::default();
        // This test bypasses updater() to construct marked resident slots.
        store.change_markers_cleared = false;
        store.transforms.insert(
            key(PaintPropertyRole::ScrollTranslation),
            translation.clone(),
        );
        store
            .transforms
            .insert(key(PaintPropertyRole::Transform), child.clone());
        store
            .clips
            .insert(key(PaintPropertyRole::OverflowClip), clip.clone());
        store
            .effects
            .insert(key(PaintPropertyRole::Effect), effect.clone());
        store.scrolls.insert(key(PaintPropertyRole::Scroll), scroll);
        store.scrolls.insert(
            PaintPropertyKey {
                owner: PaintPropertyOwner::Fragment {
                    client: 43,
                    fragment: 0,
                },
                role: PaintPropertyRole::Scroll,
            },
            secondary_scroll.clone(),
        );
        let (cleared, cleared_trees) = store.cleared_change_markers().unwrap();
        assert_eq!(cleared.changed, PaintPropertyChangeType::Unchanged);
        assert_eq!(cleared.next_id, store.next_id);
        assert_eq!(
            cleared_trees.clips[&4].lifecycle.changed,
            PaintPropertyChangeType::Unchanged
        );
        assert!(cleared_trees.clips[&4].lifecycle.same_node(&clip.lifecycle));
        assert_eq!(
            cleared_trees.clips[&4].lifecycle.revision,
            clip.lifecycle.revision
        );
        assert_eq!(
            clip.lifecycle.changed,
            PaintPropertyChangeType::ChangedOnlyValues,
            "clearing published markers leaves the old artifact immutable"
        );
        assert!(Arc::ptr_eq(
            cleared_trees.effects[&5].output_clip.as_ref().unwrap(),
            &cleared_trees.clips[&4]
        ));
        assert!(cleared.cleared_change_markers().is_none());
        let offsets = BTreeMap::from([(owner, Offset { x: 0.0, y: 12.5 })]);
        let (updated, trees) = store
            .retained_scroll_update(&offsets, &BTreeMap::new())
            .unwrap();
        assert_eq!(
            translation.matrix.values[13], 0.0,
            "old snapshot stays immutable"
        );
        assert_eq!(trees.transforms[&2].matrix.values[13], -12.5);
        assert!(trees.transforms[&2]
            .lifecycle
            .same_node(&translation.lifecycle));
        assert_eq!(
            trees.transforms[&2].lifecycle.revision,
            translation.lifecycle.revision + 1
        );
        assert_eq!(
            trees.transforms[&2].lifecycle.changed,
            PaintPropertyChangeType::ChangedOnlySimpleValues
        );
        for (before, after) in [
            (&child.lifecycle, &trees.transforms[&3].lifecycle),
            (&clip.lifecycle, &trees.clips[&4].lifecycle),
            (&effect.lifecycle, &trees.effects[&5].lifecycle),
            (&secondary_scroll.lifecycle, &trees.scrolls[&6].lifecycle),
        ] {
            assert_eq!(before, after);
            assert_eq!(after.changed, PaintPropertyChangeType::Unchanged);
        }
        assert!(Arc::ptr_eq(
            trees.transforms[&3].parent.as_ref().unwrap(),
            &trees.transforms[&2]
        ));
        assert!(Arc::ptr_eq(
            &trees.clips[&4].local_transform_space,
            &trees.transforms[&3]
        ));
        assert!(Arc::ptr_eq(
            trees.effects[&5].output_clip.as_ref().unwrap(),
            &trees.clips[&4]
        ));
        assert!(Arc::ptr_eq(
            trees.scrolls[&6].overflow_clip.as_ref().unwrap(),
            &trees.clips[&4]
        ));
        assert!(updated.is_scroll_only_update_from(&store));
        assert!(
            trees.transforms.contains_key(&0)
                && trees.clips.contains_key(&0)
                && trees.effects.contains_key(&0)
                && trees.scrolls.contains_key(&0)
        );
        let (cleared_updated, cleared_trees) = updated.cleared_change_markers().unwrap();
        assert_eq!(
            cleared_trees.transforms[&2].matrix,
            trees.transforms[&2].matrix
        );
        assert!(cleared_trees.transforms[&2]
            .lifecycle
            .same_node(&trees.transforms[&2].lifecycle));
        assert_eq!(
            cleared_trees.transforms[&2].lifecycle.revision,
            trees.transforms[&2].lifecycle.revision
        );
        assert_eq!(
            cleared_trees.transforms[&2].lifecycle.changed,
            PaintPropertyChangeType::Unchanged
        );
        assert_eq!(
            trees.transforms[&2].lifecycle.changed,
            PaintPropertyChangeType::ChangedOnlySimpleValues
        );
        assert!(cleared_updated.cleared_change_markers().is_none());
        let (_, unchanged) = updated
            .retained_scroll_update(&offsets, &BTreeMap::new())
            .unwrap();
        assert_eq!(
            unchanged.transforms[&2].lifecycle.revision,
            trees.transforms[&2].lifecycle.revision
        );
        assert_eq!(
            unchanged.transforms[&2].lifecycle.changed,
            PaintPropertyChangeType::Unchanged
        );
        assert!(store
            .retained_scroll_update(&BTreeMap::new(), &BTreeMap::new())
            .is_none());
        let mut new_owner = offsets.clone();
        new_owner.insert(
            PaintPropertyOwner::Fragment {
                client: 99,
                fragment: 0,
            },
            Offset { x: 0.0, y: 1.0 },
        );
        assert!(store
            .retained_scroll_update(&new_owner, &BTreeMap::new())
            .is_none());
        let constraint = StickyPositionPaintData {
            owner_client_id: 42,
            vertical: Some(layoutng_assembly::fragment_tree::StickyAxisPaintData {
                scroll_container_client_id: 42,
                min_inset: Some(0.0),
                ..Default::default()
            }),
            ..Default::default()
        };
        let sticky_node = Arc::new(TransformPaintPropertyNode {
            compositor: Default::default(),
            geometry_mapper_cache: Default::default(),
            scroll_parent_scroll_translation: None,
            id: 7,
            parent: Some(child.clone()),
            sticky_constraint: Some(constraint),
            ..(*child).clone()
        });
        store.transforms.insert(
            key(PaintPropertyRole::StickyTranslation),
            sticky_node.clone(),
        );
        let mut current = constraint;
        current.vertical.as_mut().unwrap().current_sticky_offset = 10.0;
        current
            .vertical
            .as_mut()
            .unwrap()
            .current_total_sticky_box_offset = 10.0;
        let mut sticky = BTreeMap::from([(owner, (Offset { x: 0.0, y: 10.0 }, current))]);
        let (_, trees) = store.retained_scroll_update(&offsets, &sticky).unwrap();
        assert_eq!(trees.transforms[&7].matrix.values[13], 10.0);
        assert_eq!(trees.transforms[&7].sticky_constraint, Some(current));
        assert_eq!(sticky_node.matrix.values[13], 0.0);
        assert_eq!(sticky_node.sticky_constraint, Some(constraint));
        assert!(trees.transforms[&7]
            .lifecycle
            .same_node(&sticky_node.lifecycle));
        assert!(Arc::ptr_eq(
            trees.transforms[&7].parent.as_ref().unwrap(),
            &trees.transforms[&3]
        ));
        sticky
            .get_mut(&owner)
            .unwrap()
            .1
            .vertical
            .as_mut()
            .unwrap()
            .min_inset = Some(2.0);
        assert!(store.retained_scroll_update(&offsets, &sticky).is_none());
        assert!(store
            .retained_scroll_update(&offsets, &BTreeMap::new())
            .is_none());
    }
}
