//! Blink-style paint property references for the supported standalone input.
//!
//! Sources: platform/graphics/paint/property_tree_state.h and the transform,
//! clip, effect and scroll_paint_property_node.h files. These are paint trees,
//! not compositor layer trees. Numeric IDs are dump-local labels; reference
//! equality, as in Blink, determines whether states share property nodes.
use std::collections::BTreeMap;
use std::fmt;
use crate::property_cell::{
    MutableCell, PaintSequenceId, PropertyAccess, PropertyCell, PropertyWriter,
};
use std::sync::Arc;

use crate::paint_clip_path::PaintClipPath;
use crate::property_filter_operations::PropertyFilterOperations;
use crate::PaintRect;
use paint_input_types::layout_input::StickyPositionPaintData;
pub use paint_input_types::layout_input::OverscrollBehavior as LayoutOverscrollBehavior;
use paint_input_types::layout_input::{Offset, TransformMatrix};
pub use paint_input_types::paint_input::PaintBlendMode;
use paint_input_types::paint_input::PaintCornerRadii;

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

/// The mutable members of one live node (paint_property_node.h:214-230).
/// Nodes are updated in place, so a node and its identity allocation are one
/// to one; only hand-built test fixtures copy a node (and so share it).
#[derive(Debug)]
pub struct PaintPropertyNodeIdentity {
    // Chromium: third_party/blink/renderer/platform/graphics/paint/
    // paint_property_node.h:91-97,228-230. One mutable cache slot on the
    // actual Paint property node, keyed by the cc property-tree sequence:
    // (sequence number, cc node id).
    cc_node: MutableCell<(i32, i32)>,
    // Chromium: paint_property_node.h:191-230. These are mutable members of
    // the resident Paint node even when the property state is reached through
    // a const PaintArtifact. Keeping them on the persistent identity allocation
    // preserves that ownership across immutable Rust value revisions. They
    // are confined to the document sequence that created the node: PrePaint,
    // Paint and PaintArtifactCompositor all run there.
    changed: MutableCell<PaintPropertyChangeType>,
    changed_sequence_number: MutableCell<i32>,
}

impl PaintPropertyNodeIdentity {
    const INVALID_CC_NODE_ID: i32 = -1;

    fn new(changed: PaintPropertyChangeType) -> Self {
        if changed != PaintPropertyChangeType::Unchanged {
            crate::property_cell::NotePropertyChange();
        }
        let owner = PaintSequenceId::current();
        Self {
            cc_node: MutableCell::with_owner(owner, (0, Self::INVALID_CC_NODE_ID)),
            changed: MutableCell::with_owner(owner, changed),
            changed_sequence_number: MutableCell::with_owner(owner, 0),
        }
    }

    fn cc_node_id(&self, access: &PropertyAccess, sequence_number: i32) -> i32 {
        let (mapped_sequence, node_id) = self.cc_node.get(access);
        if mapped_sequence == sequence_number {
            node_id
        } else {
            Self::INVALID_CC_NODE_ID
        }
    }

    fn set_cc_node_id(&self, access: &PropertyAccess, sequence_number: i32, node_id: i32) {
        self.cc_node.set(access, (sequence_number, node_id));
    }

    fn node_changed(&self, access: &PropertyAccess) -> PaintPropertyChangeType {
        self.changed.get(access)
    }

    /// Source: paint_property_node.h:185-188 `AddChanged`.
    fn add_changed(&self, access: &PropertyAccess, changed: PaintPropertyChangeType) {
        if changed != PaintPropertyChangeType::Unchanged {
            crate::property_cell::NotePropertyChange();
        }
        if changed > self.changed.get(access) {
            self.changed.set(access, changed);
        }
    }

    fn clear_changed(&self, access: &PropertyAccess, sequence_number: i32) {
        debug_assert_ne!(self.changed_sequence_number.get(access), sequence_number);
        self.changed.set(access, PaintPropertyChangeType::Unchanged);
        self.changed_sequence_number.set(access, sequence_number);
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
            identity: Arc::new(PaintPropertyNodeIdentity::new(
                PaintPropertyChangeType::NodeAddedOrRemoved,
            )),
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
    pub fn cc_node_id(&self, access: &PropertyAccess, sequence_number: i32) -> i32 {
        self.identity.cc_node_id(access, sequence_number)
    }
    pub fn set_cc_node_id(&self, access: &PropertyAccess, sequence_number: i32, node_id: i32) {
        self.identity.set_cc_node_id(access, sequence_number, node_id);
    }
    fn root() -> Self {
        Self {
            identity: Arc::new(PaintPropertyNodeIdentity::new(
                PaintPropertyChangeType::Unchanged,
            )),
            revision: 0,
            changed: PaintPropertyChangeType::Unchanged,
        }
    }
    fn updated(&self, changed: PaintPropertyChangeType) -> Self {
        // Value revisions are produced on the owning document sequence; the
        // shared entity marker is raised there, as AddChanged does in Blink.
        crate::property_cell::with_access(|access| self.identity.add_changed(access, changed));
        Self {
            identity: self.identity.clone(),
            revision: self.revision + u64::from(changed != PaintPropertyChangeType::Unchanged),
            changed,
        }
    }

    /// Source: paint_property_node.h:185-188 `AddChanged`, raised on the
    /// resident node by an in-place `Update` under the caller's proof.
    pub(crate) fn AddChanged(&self, access: &PropertyAccess, changed: PaintPropertyChangeType) {
        self.identity.add_changed(access, changed);
    }

    pub fn node_changed(&self, access: &PropertyAccess) -> PaintPropertyChangeType {
        self.identity.node_changed(access)
    }
    pub fn changed_sequence_number(&self, access: &PropertyAccess) -> i32 {
        self.identity.changed_sequence_number.get(access)
    }
    pub fn clear_changed(&self, access: &PropertyAccess, sequence_number: i32) {
        self.identity.clear_changed(access, sequence_number);
    }
    /// Source: paint_property_node.h:82-85. A successful direct compositor
    /// update downgrades only the exact simple-values marker.
    pub fn CompositorSimpleValuesUpdated(&self, access: &PropertyAccess) {
        if self.identity.changed.get(access) == PaintPropertyChangeType::ChangedOnlySimpleValues {
            self.identity
                .changed
                .set(access, PaintPropertyChangeType::ChangedOnlyCompositedValues);
        }
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
    let changed = crate::property_cell::with_access(|access| lifecycle.node_changed(access));
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

/// Source: transform_paint_property_node.h:150-485. Dump id, lifecycle
/// identity and the GeometryMapper cache slot are plain fields; the node
/// `State` (with the parent link, paint_property_node.h:232) is confined to
/// the document sequence that created the node and read through a
/// `PropertyAccess`.
#[derive(Debug)]
pub struct TransformPaintPropertyNode {
    /// Source: transform_paint_property_node.h:472-485 `mutable
    /// Member<GeometryMapperTransformCache> transform_cache_`, lazily
    /// allocated and validated by the sequence's cache generation.
    pub(crate) geometry_mapper_cache: MutableCell<
        Option<Box<crate::geometry_mapper_source::geometry_mapper_transform_cache::NativeGeometryMapperTransformCache>>,
    >,
    pub lifecycle: PaintPropertyNodeLifecycle,
    pub id: u64,
    state: PropertyCell<TransformPaintPropertyNodeState>,
}

/// Source: transform_paint_property_node.h:110-134
/// `TransformPaintPropertyNode::State` plus the parent link and Rechrom's
/// Layout sticky association.
#[derive(Clone, Debug, PartialEq)]
pub struct TransformPaintPropertyNodeState {
    /// Source: transform_paint_property_node.h:110-134. These remain source
    /// state fields even when a feature does not populate them on this node.
    pub compositor: TransformCompositorState,
    pub parent: Option<Arc<TransformPaintPropertyNode>>,
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

/// Node accessors for the source predicates implemented on the node `State`:
/// each reads the state under the caller's proof.
macro_rules! state_getters {
    ($lt:lifetime; $($name:ident -> $ret:ty;)*) => {$(
        #[inline]
        pub fn $name<$lt>(&$lt self, access: &$lt PropertyAccess) -> $ret {
            self.state(access).$name()
        }
    )*};
}

/// Two optional node links name the same allocation (Chromium's `Member`
/// pointer compare). `Update` stores a state whose link changed allocation
/// even when the change classification compares node entities.
fn same_link<T>(a: &Option<Arc<T>>, b: &Option<Arc<T>>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => Arc::ptr_eq(a, b),
        _ => false,
    }
}

impl Clone for TransformPaintPropertyNode {
    /// A separate node with a copy of this node's current state and the same
    /// lifecycle markers, for hand-built test fixtures. Production code holds
    /// nodes by `Arc` and never copies one: a node is one live object.
    fn clone(&self) -> Self {
        Self::new(
            self.lifecycle.clone(),
            self.id,
            crate::property_cell::with_access(|access| self.state(access).clone()),
        )
    }
}
/// Source: paint_property_node.h: nodes are compared by pointer
/// (`Member<const Node>` / `scoped_refptr`); one live object per node.
impl PartialEq for TransformPaintPropertyNode {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
impl Eq for TransformPaintPropertyNode {}

impl TransformPaintPropertyNode {
    /// Source: transform_paint_property_node.h:150-171 `Create`. The node
    /// belongs to the calling document sequence.
    pub fn new(
        lifecycle: PaintPropertyNodeLifecycle,
        id: u64,
        state: TransformPaintPropertyNodeState,
    ) -> Self {
        let owner = PaintSequenceId::current();
        Self {
            geometry_mapper_cache: MutableCell::with_owner(owner, None),
            lifecycle,
            id,
            state: PropertyCell::with_owner(owner, state),
        }
    }
    #[inline]
    pub fn state<'a>(&'a self, access: &'a PropertyAccess) -> &'a TransformPaintPropertyNodeState {
        self.state.get(access)
    }
    /// Exclusive access to a snapshot that is not shared yet.
    #[inline]
    pub fn state_mut(&mut self) -> &mut TransformPaintPropertyNodeState {
        self.state.get_mut()
    }
    /// Source: paint_property_node.h:116-119 `Parent`.
    #[inline]
    pub fn Parent<'a>(&'a self, access: &'a PropertyAccess) -> Option<&'a Arc<Self>> {
        self.state(access).parent.as_ref()
    }

    state_getters! {'a;
        IsIdentityOr2dTranslation -> bool;
        IsIdentity -> bool;
        Get2dTranslation -> [f32; 2];
        Matrix -> &'a TransformMatrix;
        MatrixWithOriginApplied -> foundation::gfx::Transform;
        Origin -> [f64; 3];
        ScrollNode -> Option<&'a Arc<ScrollPaintPropertyNode>>;
        ScrollParentScrollTranslation -> Option<&'a Arc<TransformPaintPropertyNode>>;
        IsAffectedByOuterViewportBoundsDelta -> bool;
        IsAffectedBySafeAreaBottom -> bool;
        IsInSubtreeOfPageScale -> bool;
        GetStickyConstraint -> Option<&'a PaintCompositorStickyConstraint>;
        GetAnchorPositionScrollData -> Option<&'a PaintAnchorPositionScrollData>;
        FlattensInheritedTransform -> bool;
        HasDirectCompositingReasons -> bool;
        HasDirectCompositingReasonsOtherThan3dTransform -> bool;
        HasActiveTransformAnimation -> bool;
        RequiresCompositingForFixedPosition -> bool;
        RequiresCompositingForFixedPositionOnly -> bool;
        RequiresCompositingForFixedToViewport -> bool;
        RequiresCompositingForStickyPosition -> bool;
        RequiresCompositingForStickyPositionOnly -> bool;
        RequiresCompositingForAnchorPosition -> bool;
        DirectCompositingReasonsForDebugging -> foundation::CompositingReasons;
        TransformAnimationIsAxisAligned -> bool;
        RequiresCompositingForRootScroller -> bool;
        RequiresCompositingForWillChangeTransform -> bool;
        UsesCompositedScrolling -> bool;
        RequiresCullRectExpansion -> bool;
        GetCompositorElementId -> foundation::CompositorElementId;
        GetVisibleFrameElementId -> foundation::CompositorElementId;
        IsFramePaintOffsetTranslation -> bool;
        DelegatesToParentForBackface -> bool;
        RenderingContextId -> u64;
        HasRenderingContext -> bool;
        IsForSVGChild -> bool;
    }

    /// Source: transform_paint_property_node.h:173-185 `Update` (SetParent,
    /// State::ComputeChange, AddChanged) on this resident node, in place.
    /// The parent is the `State`'s `parent` link. Rechrom's Layout sticky
    /// association and a newer reference to the same linked node entity are
    /// stored without promoting the source change category.
    pub fn Update(
        &self,
        writer: &mut PropertyWriter,
        state: TransformPaintPropertyNodeState,
        animation_state: TransformAnimationState,
    ) -> PaintPropertyChangeType {
        let (change, store) = {
            let old = self.state(writer);
            let change = transform_node_state_change(old, &state, animation_state);
            let store = change != PaintPropertyChangeType::Unchanged
                || old.sticky_constraint != state.sticky_constraint
                || !same_link(&old.parent, &state.parent)
                || !same_link(&old.scroll, &state.scroll)
                || !same_link(
                    &old.scroll_parent_scroll_translation,
                    &state.scroll_parent_scroll_translation,
                );
            (change, store)
        };
        if store {
            drop(self.state.set(writer, state));
        }
        if change != PaintPropertyChangeType::Unchanged {
            self.lifecycle.AddChanged(writer, change);
            // Source: transform_paint_property_node.h:50-55 (AddChanged
            // override): a transform change clears both GeometryMapper caches.
            crate::property_cell::ClearTransformCache();
            crate::property_cell::ClearClipCache();
        }
        change
    }

    /// The change `DirectlyUpdateTransformAndOrigin` would report, without
    /// updating the node (the direct-update path admits all its nodes
    /// before it updates any).
    pub fn DirectTransformAndOriginChange(
        &self,
        access: &PropertyAccess,
        matrix: TransformMatrix,
        origin: [f64; 3],
        animation_state: TransformAnimationState,
    ) -> PaintPropertyChangeType {
        let old = self.state(access);
        let mut state = old.clone();
        state.matrix = matrix;
        state.origin = origin;
        compute_transform_change(old, &state, animation_state)
    }

    /// Source: transform_paint_property_node.cc:124-134, on this resident
    /// node in place.
    pub fn DirectlyUpdateTransformAndOrigin(
        &self,
        writer: &mut PropertyWriter,
        matrix: TransformMatrix,
        origin: [f64; 3],
        animation_state: TransformAnimationState,
    ) -> PaintPropertyChangeType {
        let change = self.DirectTransformAndOriginChange(writer, matrix, origin, animation_state);
        self.state.modify(writer, |state| {
            state.matrix = matrix;
            state.origin = origin;
        });
        if change != PaintPropertyChangeType::Unchanged {
            self.lifecycle.AddChanged(writer, change);
            crate::property_cell::ClearTransformCache();
            crate::property_cell::ClearClipCache();
        }
        change
    }

    /// Source: transform_paint_property_node.cc:151-164. Supported producer
    /// nodes are unaliased; associated Scroll changes are cleared recursively.
    pub fn ClearChangedToRoot(&self, access: &PropertyAccess, sequence_number: i32) {
        let mut current = Some(self);
        while let Some(node) = current {
            if node.lifecycle.changed_sequence_number(access) == sequence_number {
                break;
            }
            node.lifecycle.clear_changed(access, sequence_number);
            if let Some(scroll) = node.ScrollNode(access) {
                scroll.ClearChangedToRoot(access, sequence_number);
            }
            current = node.Parent(access).map(|parent| &**parent);
        }
    }

    /// Source: transform_paint_property_node.cc:166-177.
    pub fn CanMergeForFixedPosition(&self, access: &PropertyAccess, other: &Self) -> bool {
        self.state(access).CanMergeForFixedPosition(other.state(access))
    }

    /// Source: transform_paint_property_node.cc:221-267.
    pub fn ToJSON(&self) -> serde_json::Value {
        crate::property_cell::with_access(|access| {
            self.state(access).ToJSON(self.Parent(access), &self.lifecycle)
        })
    }
    pub fn ToString(&self) -> String {
        PaintPropertyNodeToString(&self.ToJSON())
    }
}

/// The property-node links of one transform state, cloned out of the proof
/// scope for walks that recurse across node kinds.
#[derive(Clone)]
pub struct TransformStateLinks {
    pub parent: Option<Arc<TransformPaintPropertyNode>>,
    pub scroll: Option<Arc<ScrollPaintPropertyNode>>,
    pub scroll_parent_scroll_translation: Option<Arc<TransformPaintPropertyNode>>,
}

impl TransformPaintPropertyNodeState {
    pub fn links(&self) -> TransformStateLinks {
        TransformStateLinks {
            parent: self.parent.clone(),
            scroll: self.scroll.clone(),
            scroll_parent_scroll_translation: self.scroll_parent_scroll_translation.clone(),
        }
    }
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
    fn ToJSON(
        &self,
        parent: Option<&Arc<TransformPaintPropertyNode>>,
        lifecycle: &PaintPropertyNodeLifecycle,
    ) -> serde_json::Value {
        let mut json = PaintPropertyNodeToJSON(parent, lifecycle);
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

/// Source: clip_paint_property_node.h:163-258. Dump id, lifecycle identity
/// and the GeometryMapper cache slot are plain fields; the node `State` (with
/// the parent link, paint_property_node.h:232) is confined to the document
/// sequence that created the node and read through a `PropertyAccess`.
#[derive(Debug)]
pub struct ClipPaintPropertyNode {
    /// Source: clip_paint_property_node.h:249-258 `mutable
    /// Member<GeometryMapperClipCache> geometry_mapper_clip_cache_`.
    pub(crate) geometry_mapper_cache: MutableCell<
        Option<Box<crate::geometry_mapper_source::geometry_mapper_clip_cache::GeometryMapperClipCache>>,
    >,
    pub lifecycle: PaintPropertyNodeLifecycle,
    pub id: u64,
    state: PropertyCell<ClipPaintPropertyNodeState>,
}

/// Source: clip_paint_property_node.h:88-125 `ClipPaintPropertyNode::State`
/// plus the parent link.
#[derive(Clone, Debug, PartialEq)]
pub struct ClipPaintPropertyNodeState {
    /// Source: clip_paint_property_node.h:88-125,188-217. Layout and paint
    /// clips are distinct source values, including compositor animation bounds.
    pub layout_clip_rects: ClipLayoutRects,
    pub parent: Option<Arc<ClipPaintPropertyNode>>,
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

impl Clone for ClipPaintPropertyNode {
    /// A separate node with a copy of this node's current state and the same
    /// lifecycle markers, for hand-built test fixtures. Production code holds
    /// nodes by `Arc` and never copies one: a node is one live object.
    fn clone(&self) -> Self {
        Self::new(
            self.lifecycle.clone(),
            self.id,
            crate::property_cell::with_access(|access| self.state(access).clone()),
        )
    }
}
/// Pointer identity, as for TransformPaintPropertyNode.
impl PartialEq for ClipPaintPropertyNode {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
impl Eq for ClipPaintPropertyNode {}

impl ClipPaintPropertyNode {
    /// Source: clip_paint_property_node.h:150-160 `Create`. The node belongs
    /// to the calling document sequence.
    pub fn new(
        lifecycle: PaintPropertyNodeLifecycle,
        id: u64,
        state: ClipPaintPropertyNodeState,
    ) -> Self {
        let owner = PaintSequenceId::current();
        Self {
            geometry_mapper_cache: MutableCell::with_owner(owner, None),
            lifecycle,
            id,
            state: PropertyCell::with_owner(owner, state),
        }
    }
    #[inline]
    pub fn state<'a>(&'a self, access: &'a PropertyAccess) -> &'a ClipPaintPropertyNodeState {
        self.state.get(access)
    }
    /// Exclusive access to a snapshot that is not shared yet.
    #[inline]
    pub fn state_mut(&mut self) -> &mut ClipPaintPropertyNodeState {
        self.state.get_mut()
    }
    /// Source: paint_property_node.h:116-119 `Parent`.
    #[inline]
    pub fn Parent<'a>(&'a self, access: &'a PropertyAccess) -> Option<&'a Arc<Self>> {
        self.state(access).parent.as_ref()
    }
    /// Source: clip_paint_property_node.h:88-99,110-125. Normal source clip
    /// creation supplies equal precise/paint input until the builder supplies
    /// its unsnapped layout value or an explicit animation expansion.
    pub fn with_layout_clips(mut self) -> Self {
        let state = self.state_mut();
        state.layout_clip_rects =
            ClipLayoutRects::new(state.rect, state.radii.HasRadius(), None, None);
        self
    }
    pub fn with_layout_clip_values(
        mut self,
        precise: Option<PaintRect>,
        expanded: Option<PaintRect>,
        excluding_overlay: Option<PaintRect>,
    ) -> Self {
        let state = self.state_mut();
        state.layout_clip_rects =
            ClipLayoutRects::new(precise, state.radii.HasRadius(), expanded, excluding_overlay);
        self
    }
    pub fn clip_path_child_count(&self, access: &PropertyAccess) -> usize {
        self.state(access)
            .clip_path
            .as_ref()
            .map_or(0, |shape| shape.paths().len())
    }
    pub fn single_clip_path<'a>(
        &'a self,
        access: &'a PropertyAccess,
    ) -> Option<&'a crate::paint_clip_path::PaintPath> {
        self.state(access).clip_path.as_ref()?.single_path().map(Arc::as_ref)
    }
    pub fn clip_path_verb_count(&self, access: &PropertyAccess) -> usize {
        self.state(access).clip_path.as_ref().map_or(0, |shape| {
            shape.paths().iter().map(|p| p.verb_count()).sum()
        })
    }

    // Source: clip_paint_property_node.h:183-232. The node owns these source
    // queries; compositor adapters only convert their value types.
    pub fn LocalTransformSpace<'a>(
        &'a self,
        access: &'a PropertyAccess,
    ) -> &'a Arc<TransformPaintPropertyNode> {
        &self.state(access).local_transform_space
    }
    pub fn PaintClipRect(&self, access: &PropertyAccess) -> Option<PaintRect> {
        self.state(access).rect
    }
    pub fn PaintClipRadii(&self, access: &PropertyAccess) -> PaintCornerRadii {
        self.state(access).radii
    }
    pub fn LayoutClipRect(&self, access: &PropertyAccess) -> ClipLayoutRect {
        self.state(access).layout_clip_rects.expanded
    }
    pub fn PreciseLayoutClipRect(&self, access: &PropertyAccess) -> ClipLayoutRect {
        self.state(access).layout_clip_rects.precise
    }
    pub fn IsForCompositeClipPathAnimation(&self, access: &PropertyAccess) -> bool {
        let rects = &self.state(access).layout_clip_rects;
        rects.expanded != rects.precise
    }
    pub fn LayoutClipRectExcludingOverlayScrollbars(
        &self,
        access: &PropertyAccess,
    ) -> ClipLayoutRect {
        let rects = &self.state(access).layout_clip_rects;
        rects.excluding_overlay.unwrap_or(rects.precise)
    }
    pub fn ClipPath<'a>(&'a self, access: &'a PropertyAccess) -> Option<&'a Arc<PaintClipPath>> {
        self.state(access).clip_path.as_ref()
    }
    pub fn ClipPathEquals(&self, access: &PropertyAccess, path: Option<&Arc<PaintClipPath>>) -> bool {
        self.state(access).clip_path.as_ref() == path
    }
    pub fn PixelMovingFilter<'a>(
        &'a self,
        access: &'a PropertyAccess,
    ) -> Option<&'a Arc<EffectPaintPropertyNode>> {
        self.state(access).pixel_moving_filter.as_ref()
    }

    /// Source: clip_paint_property_node.h:163-178 `Update` on this resident
    /// node, in place; the parent is the `State`'s `parent` link.
    pub fn Update(
        &self,
        writer: &mut PropertyWriter,
        state: ClipPaintPropertyNodeState,
    ) -> PaintPropertyChangeType {
        let (change, store) = {
            let old = self.state(writer);
            let change = clip_node_state_change(old, &state);
            let store = change != PaintPropertyChangeType::Unchanged
                || !same_link(&old.parent, &state.parent)
                || !Arc::ptr_eq(&old.local_transform_space, &state.local_transform_space)
                || !same_link(&old.pixel_moving_filter, &state.pixel_moving_filter);
            (change, store)
        };
        if store {
            drop(self.state.set(writer, state));
        }
        if change != PaintPropertyChangeType::Unchanged {
            self.lifecycle.AddChanged(writer, change);
            // Source: clip_paint_property_node.h:53-57 (AddChanged override).
            crate::property_cell::ClearClipCache();
        }
        change
    }

    /// Source: clip_paint_property_node.cc:70-81. Supported snapshots are
    /// unaliased; each clip also clears its local transform chain.
    pub fn ClearChangedToRoot(&self, access: &PropertyAccess, sequence_number: i32) {
        let mut current = Some(self);
        while let Some(node) = current {
            if node.lifecycle.changed_sequence_number(access) == sequence_number {
                break;
            }
            node.lifecycle.clear_changed(access, sequence_number);
            node.LocalTransformSpace(access).ClearChangedToRoot(access, sequence_number);
            current = node.Parent(access).map(|parent| &**parent);
        }
    }

    /// Source: clip_paint_property_node.cc:83-114.
    pub fn ToJSON(&self) -> serde_json::Value {
        crate::property_cell::with_access(|access| self.ToJSONIn(access))
    }
    fn ToJSONIn(&self, access: &PropertyAccess) -> serde_json::Value {
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

        let mut json = PaintPropertyNodeToJSON(self.Parent(access), &self.lifecycle);
        json.insert(
            "localTransformSpace".into(),
            serde_json::Value::String(format!("{:p}", Arc::as_ptr(self.LocalTransformSpace(access)))),
        );
        json.insert(
            "rect".into(),
            serde_json::Value::String(rect_string(self.PaintClipRect(access))),
        );
        if let Some(excluding) = self.state(access).layout_clip_rects.excluding_overlay {
            if excluding != self.state(access).layout_clip_rects.expanded {
                json.insert(
                    "rectExcludingOverlayScrollbars".into(),
                    serde_json::Value::String(rect_string(excluding.rect)),
                );
            }
        }
        if self.ClipPath(access).is_some() {
            json.insert("hasClipPath".into(), serde_json::Value::Bool(true));
        }
        if self.IsForCompositeClipPathAnimation(access) {
            json.insert(
                "isForCompositeClipPathAnimation".into(),
                serde_json::Value::Bool(true),
            );
            json.insert(
                "expandedLayoutClipRect".into(),
                serde_json::Value::String(rect_string(self.LayoutClipRect(access).rect)),
            );
            json.insert(
                "preciseLayoutClipRect".into(),
                serde_json::Value::String(rect_string(self.PreciseLayoutClipRect(access).rect)),
            );
        }
        if let Some(filter) = self.PixelMovingFilter(access) {
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

/// Source: effect_paint_property_node.h:216-433. Dump id and lifecycle
/// identity are plain fields; the node `State` (with the parent link,
/// paint_property_node.h:232) is confined to the document sequence that
/// created the node and read through a `PropertyAccess`.
#[derive(Debug)]
pub struct EffectPaintPropertyNode {
    pub lifecycle: PaintPropertyNodeLifecycle,
    pub id: u64,
    state: PropertyCell<EffectPaintPropertyNodeState>,
}

/// Source: effect_paint_property_node.h:82-176 `EffectPaintPropertyNode::State`
/// plus the parent link.
#[derive(Clone, Debug, PartialEq)]
pub struct EffectPaintPropertyNodeState {
    pub compositor: EffectCompositorState,
    /// Source: effect_paint_property_node.h:97-105,261-269.
    pub backdrop_filter_bounds: Option<Arc<crate::paint_clip_path::PaintPath>>,
    /// Source: effect_paint_property_node.h:427-429; builder.cc:2263-2264,
    /// 2493-2498,2690-2691. The producer publishes inherited participation.
    pub self_or_ancestor_participates_in_view_transition: bool,
    pub parent: Option<Arc<EffectPaintPropertyNode>>,
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

pub use foundation::canvas_child_paint_state::CanvasChildPaintState;

impl Clone for EffectPaintPropertyNode {
    /// A separate node with a copy of this node's current state and the same
    /// lifecycle markers, for hand-built test fixtures. Production code holds
    /// nodes by `Arc` and never copies one: a node is one live object.
    fn clone(&self) -> Self {
        Self::new(
            self.lifecycle.clone(),
            self.id,
            crate::property_cell::with_access(|access| self.state(access).clone()),
        )
    }
}
/// Pointer identity, as for TransformPaintPropertyNode.
impl PartialEq for EffectPaintPropertyNode {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
impl Eq for EffectPaintPropertyNode {}

impl EffectPaintPropertyNode {
    /// Source: effect_paint_property_node.h:197-214 `Create`. The node belongs
    /// to the calling document sequence.
    pub fn new(
        lifecycle: PaintPropertyNodeLifecycle,
        id: u64,
        state: EffectPaintPropertyNodeState,
    ) -> Self {
        Self {
            lifecycle,
            id,
            state: PropertyCell::with_owner(PaintSequenceId::current(), state),
        }
    }
    #[inline]
    pub fn state<'a>(&'a self, access: &'a PropertyAccess) -> &'a EffectPaintPropertyNodeState {
        self.state.get(access)
    }
    /// Exclusive access to a snapshot that is not shared yet.
    #[inline]
    pub fn state_mut(&mut self) -> &mut EffectPaintPropertyNodeState {
        self.state.get_mut()
    }
    /// Source: paint_property_node.h:116-119 `Parent`.
    #[inline]
    pub fn Parent<'a>(&'a self, access: &'a PropertyAccess) -> Option<&'a Arc<Self>> {
        self.state(access).parent.as_ref()
    }
    /// Current producer publishes FilterInfo only at its actual nonempty filter
    /// construction sites; retain this ownership fact on the published node.
    pub fn with_filter_presence(mut self) -> Self {
        let state = self.state_mut();
        state.compositor.has_filter_info = !state.filters.is_empty();
        state.compositor.has_backdrop_filter_info = !state.backdrop_filters.is_empty();
        self
    }
    pub fn with_inherited_view_transition(mut self) -> Self {
        let state = self.state_mut();
        let inherited = state.parent.as_ref().is_some_and(|parent| {
            crate::property_cell::with_access(|access| {
                parent.state(access).self_or_ancestor_participates_in_view_transition
            })
        });
        state.self_or_ancestor_participates_in_view_transition |= inherited;
        self
    }
    /// Used by the producer creating a participating view-transition effect;
    /// descendants subsequently inherit this same source-owned value.
    pub fn with_view_transition_participation(mut self, participates: bool) -> Self {
        self.state_mut().self_or_ancestor_participates_in_view_transition = participates;
        self.with_inherited_view_transition()
    }

    state_getters! {'a;
        LocalTransformSpace -> &'a Arc<TransformPaintPropertyNode>;
        OutputClip -> Option<&'a Arc<ClipPaintPropertyNode>>;
        Filter -> Option<&'a crate::cc::paint::filter_operations::FilterOperations>;
        BackdropFilter -> Option<&'a crate::cc::paint::filter_operations::FilterOperations>;
        HasReferenceFilter -> bool;
        HasFilterThatMovesPixels -> bool;
        HasRealEffects -> bool;
        IsOpacityOnly -> bool;
        HasDirectCompositingReasons -> bool;
        RequiresCompositingForUnboundedElement -> bool;
        RequiresCompositingForBackdropFilterMask -> bool;
        RequiresCompositingForCanvasChild -> bool;
        FlattensAtLeafOf3DScene -> bool;
        HasActiveOpacityAnimation -> bool;
        HasActiveFilterAnimation -> bool;
        HasActiveBackdropFilterAnimation -> bool;
        RequiresCompositingForWillChangeOpacity -> bool;
        RequiresCompositingForWillChangeFilter -> bool;
        RequiresCompositingForWillChangeBackdropFilter -> bool;
        MayHaveOpacity -> bool;
        MayHaveFilter -> bool;
        MayHaveBackdropFilter -> bool;
        NeedsPixelMovingFilterClipExpander -> bool;
        MayHaveBackdropEffect -> bool;
        HasValidViewTransitionElementResourceId -> bool;
        HasElementCaptureId -> bool;
        HasCanvasChildState -> bool;
        CanvasChildId -> i32;
        canvas_child_paint_state -> Option<&'a CanvasChildPaintState>;
        CanvasChildContentTransform -> &'a Arc<TransformPaintPropertyNode>;
        CanvasChildContentClip -> &'a Arc<ClipPaintPropertyNode>;
        CanvasChildContentEffect -> &'a Arc<EffectPaintPropertyNode>;
        DrawsContent -> bool;
        IsInTaintedSubtree -> bool;
        IsInDrawableCanvasSubtree -> bool;
        NeedsEffectFor2DScaleTransform -> bool;
    }

    /// Source: effect_paint_property_node.cc:112-123.
    pub fn IsOpacityChangeSimple(
        opacity: f32,
        new_opacity: f32,
        direct_compositing_reasons: foundation::CompositingReasons,
        new_direct_compositing_reasons: foundation::CompositingReasons,
    ) -> bool {
        EffectPaintPropertyNodeState::IsOpacityChangeSimple(
            opacity,
            new_opacity,
            direct_compositing_reasons,
            new_direct_compositing_reasons,
        )
    }

    /// Source: effect_paint_property_node.cc:195-217.
    pub fn ComputeOpacityChange(
        &self,
        access: &PropertyAccess,
        new_opacity: f32,
        animation_state: EffectAnimationState,
    ) -> PaintPropertyChangeType {
        self.state(access).ComputeOpacityChange(new_opacity, animation_state)
    }

    /// Source: effect_paint_property_node.h:216-227 `Update` on this resident
    /// node, in place; the parent is the `State`'s `parent` link.
    pub fn Update(
        &self,
        writer: &mut PropertyWriter,
        state: EffectPaintPropertyNodeState,
        animation_state: EffectAnimationState,
    ) -> PaintPropertyChangeType {
        let (change, store) = {
            let old = self.state(writer);
            let change = effect_node_state_change(old, &state, animation_state);
            let store = change != PaintPropertyChangeType::Unchanged
                || !same_link(&old.parent, &state.parent)
                || !Arc::ptr_eq(&old.local_transform_space, &state.local_transform_space)
                || !same_link(&old.output_clip, &state.output_clip)
                || !same_link(&old.backdrop_filter_clip, &state.backdrop_filter_clip)
                || !same_link(
                    &old.compositor.canvas_child_state,
                    &state.compositor.canvas_child_state,
                );
            (change, store)
        };
        if store {
            drop(self.state.set(writer, state));
        }
        if change != PaintPropertyChangeType::Unchanged {
            self.lifecycle.AddChanged(writer, change);
        }
        change
    }

    /// Source: effect_paint_property_node.cc:219-227, on this resident node
    /// in place.
    pub fn DirectlyUpdateOpacity(
        &self,
        writer: &mut PropertyWriter,
        opacity: f32,
        animation_state: EffectAnimationState,
    ) -> PaintPropertyChangeType {
        let change = self.state(writer).ComputeOpacityChange(opacity, animation_state);
        self.state.modify(writer, |state| state.opacity = opacity);
        if change != PaintPropertyChangeType::Unchanged {
            self.lifecycle.AddChanged(writer, change);
        }
        change
    }

    /// Source: effect_paint_property_node.cc:181-193. Supported snapshots are
    /// unaliased; local transform and output clip dependencies are included.
    pub fn ClearChangedToRoot(&self, access: &PropertyAccess, sequence_number: i32) {
        let mut current = Some(self);
        while let Some(node) = current {
            if node.lifecycle.changed_sequence_number(access) == sequence_number {
                break;
            }
            node.lifecycle.clear_changed(access, sequence_number);
            node.LocalTransformSpace(access).ClearChangedToRoot(access, sequence_number);
            if let Some(clip) = node.OutputClip(access) {
                clip.ClearChangedToRoot(access, sequence_number);
            }
            current = node.Parent(access).map(|parent| &**parent);
        }
    }

    /// Source: effect_paint_property_node.cc:257-282.
    pub fn ToJSON(&self) -> serde_json::Value {
        crate::property_cell::with_access(|access| {
            PaintPropertyNodeToJSONWithState(self.Parent(access), &self.lifecycle, self.state(access))
        })
    }

    pub fn ToString(&self) -> String {
        PaintPropertyNodeToString(&self.ToJSON())
    }
}

fn PaintPropertyNodeToJSONWithState(
    parent: Option<&Arc<EffectPaintPropertyNode>>,
    lifecycle: &PaintPropertyNodeLifecycle,
    state: &EffectPaintPropertyNodeState,
) -> serde_json::Value {
    state.ToJSON(parent, lifecycle)
}

/// The property-node links of one effect state, cloned out of the proof scope
/// for walks that recurse across node kinds.
#[derive(Clone)]
pub struct EffectStateLinks {
    pub parent: Option<Arc<EffectPaintPropertyNode>>,
    pub local_transform_space: Arc<TransformPaintPropertyNode>,
    pub output_clip: Option<Arc<ClipPaintPropertyNode>>,
    pub backdrop_filter_clip: Option<Arc<ClipPaintPropertyNode>>,
    pub canvas_child_state: Option<Arc<PaintCanvasChildState>>,
}

impl EffectPaintPropertyNodeState {
    pub fn links(&self) -> EffectStateLinks {
        EffectStateLinks {
            parent: self.parent.clone(),
            local_transform_space: self.local_transform_space.clone(),
            output_clip: self.output_clip.clone(),
            backdrop_filter_clip: self.backdrop_filter_clip.clone(),
            canvas_child_state: self.compositor.canvas_child_state.clone(),
        }
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
        self.is_mask || self.blend_mode != PaintBlendMode::kNormal || self.MayHaveBackdropFilter()
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

    /// Source: effect_paint_property_node.cc:257-282. This is the source Paint
    /// node diagnostic object consumed by the compositor debug projection.
    fn ToJSON(
        &self,
        parent: Option<&Arc<EffectPaintPropertyNode>>,
        lifecycle: &PaintPropertyNodeLifecycle,
    ) -> serde_json::Value {
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

        let mut json = PaintPropertyNodeToJSON(parent, lifecycle);
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
}

/// Source: scroll_paint_property_node.h:52-83,231-233. Dump id and lifecycle
/// identity are plain fields; the node `State` (with the parent link,
/// paint_property_node.h:232) is confined to the document sequence that
/// created the node and read through a `PropertyAccess`.
#[derive(Debug)]
pub struct ScrollPaintPropertyNode {
    pub lifecycle: PaintPropertyNodeLifecycle,
    pub id: u64,
    state: PropertyCell<ScrollPaintPropertyNodeState>,
}

/// Source: scroll_paint_property_node.h:52-83 `ScrollPaintPropertyNode::State`,
/// plus the parent link and Rechrom's Layout association fields.
#[derive(Clone, Debug, PartialEq)]
pub struct ScrollPaintPropertyNodeState {
    pub compositor: ScrollCompositorState,
    /// Stable source owner identity used to connect Layout's sticky scroll
    /// ancestor to this Paint property node without exposing either owner.
    pub owner_client_id: u64,
    /// DOM owner selected by DisplayItemClient::OwnerNodeId().  Scrolling
    /// Coordinator uses this retained source identity when reflecting an impl
    /// scroll; it must not depend on optional PaintController debug records.
    pub owner_node_id: u64,
    pub parent: Option<Arc<ScrollPaintPropertyNode>>,
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
    pub overscroll_behavior: [LayoutOverscrollBehavior; 2],
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

impl Clone for ScrollPaintPropertyNode {
    /// A separate node with a copy of this node's current state and the same
    /// lifecycle markers, for hand-built test fixtures. Production code holds
    /// nodes by `Arc` and never copies one: a node is one live object.
    fn clone(&self) -> Self {
        Self::new(
            self.lifecycle.clone(),
            self.id,
            crate::property_cell::with_access(|access| self.state(access).clone()),
        )
    }
}
/// Pointer identity, as for TransformPaintPropertyNode.
impl PartialEq for ScrollPaintPropertyNode {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
impl Eq for ScrollPaintPropertyNode {}

impl ScrollPaintPropertyNode {
    /// Source: scroll_paint_property_node.h:99-106 `Create`. The node belongs
    /// to the calling document sequence.
    pub fn new(
        lifecycle: PaintPropertyNodeLifecycle,
        id: u64,
        state: ScrollPaintPropertyNodeState,
    ) -> Self {
        Self {
            lifecycle,
            id,
            state: PropertyCell::with_owner(PaintSequenceId::current(), state),
        }
    }
    #[inline]
    pub fn state<'a>(&'a self, access: &'a PropertyAccess) -> &'a ScrollPaintPropertyNodeState {
        self.state.get(access)
    }
    /// Exclusive access to a snapshot that is not shared yet.
    #[inline]
    pub fn state_mut(&mut self) -> &mut ScrollPaintPropertyNodeState {
        self.state.get_mut()
    }
    /// Source: paint_property_node.h:116-119 `Parent`.
    #[inline]
    pub fn Parent<'a>(&'a self, access: &'a PropertyAccess) -> Option<&'a Arc<Self>> {
        self.state(access).parent.as_ref()
    }

    // Source: scroll_paint_property_node.h:127-211. Both referenced runtime
    // features are stable at the pinned Chromium revision, so ContentsRect and
    // ScrollingContentsCullRect use their enabled source branches.
    pub fn OverscrollBehavior(&self, access: &PropertyAccess) -> [LayoutOverscrollBehavior; 2] {
        self.state(access).compositor.overscroll_behavior
    }
    pub fn ContainerRect(&self, access: &PropertyAccess) -> PaintRect {
        self.state(access).container_rect
    }
    pub fn ContentsRect(&self, access: &PropertyAccess) -> PaintRect {
        self.state(access).contents_rect
    }
    pub fn OverflowClipNode<'a>(
        &'a self,
        access: &'a PropertyAccess,
    ) -> Option<&'a Arc<ClipPaintPropertyNode>> {
        self.state(access).overflow_clip.as_ref()
    }
    pub fn UserScrollableHorizontal(&self, access: &PropertyAccess) -> bool {
        self.state(access).user_scrollable_horizontal
    }
    pub fn UserScrollableVertical(&self, access: &PropertyAccess) -> bool {
        self.state(access).user_scrollable_vertical
    }
    pub fn UserScrollable(&self, access: &PropertyAccess) -> bool {
        self.UserScrollableHorizontal(access) || self.UserScrollableVertical(access)
    }
    pub fn PreventViewportScrollingFromInner(&self, access: &PropertyAccess) -> bool {
        self.state(access).compositor.prevent_viewport_scrolling_from_inner
    }
    pub fn MaxScrollOffsetAffectedByPageScale(&self, access: &PropertyAccess) -> bool {
        self.state(access).compositor.max_scroll_offset_affected_by_page_scale
    }
    pub fn PreventScrollAxisLocking(&self, access: &PropertyAccess) -> bool {
        self.state(access).compositor.prevent_scroll_axis_locking
    }
    pub fn GetCompositedScrollingPreference(
        &self,
        access: &PropertyAccess,
    ) -> PaintCompositedScrollingPreference {
        self.state(access).compositor.composited_scrolling_preference
    }
    pub fn GetMainThreadRepaintReasons(&self, access: &PropertyAccess) -> u8 {
        self.state(access).compositor.main_thread_repaint_reasons
    }
    pub fn RequiresMainThreadForBackgroundAttachmentFixed(&self, access: &PropertyAccess) -> bool {
        self.GetMainThreadRepaintReasons(access) & 1 != 0
    }
    pub fn GetCompositorElementId(
        &self,
        access: &PropertyAccess,
    ) -> foundation::CompositorElementId {
        self.state(access).compositor.compositor_element_id
    }
    pub fn ScrollingContentsCullRect(&self, access: &PropertyAccess) -> PaintRect {
        self.state(access).compositor.scrolling_contents_cull_rect
    }

    /// Source: scroll_paint_property_node.h:199-204. Direct cull-rect updates
    /// always publish the source's simple-values marker, even for equal input.
    pub fn SetScrollingContentsCullRect(&mut self, rect: PaintRect) {
        self.lifecycle = self
            .lifecycle
            .updated(PaintPropertyChangeType::ChangedOnlySimpleValues);
        self.state_mut().compositor.scrolling_contents_cull_rect = rect;
    }

    /// Source: scroll_paint_property_node.h:109-120. Layout-only association
    /// fields are carried through the package boundary without changing the
    /// Chromium property-node change classification.
    pub fn Update(
        &self,
        writer: &mut PropertyWriter,
        state: ScrollPaintPropertyNodeState,
    ) -> PaintPropertyChangeType {
        let (change, store) = {
            let old = self.state(writer);
            let change = scroll_node_state_change(old, &state);
            let store = change != PaintPropertyChangeType::Unchanged
                || old.owner_client_id != state.owner_client_id
                || old.owner_node_id != state.owner_node_id
                || old.minimum_scroll_offset != state.minimum_scroll_offset
                || old.maximum_scroll_offset != state.maximum_scroll_offset
                || !same_link(&old.parent, &state.parent)
                || !same_link(&old.overflow_clip, &state.overflow_clip);
            (change, store)
        };
        if store {
            drop(self.state.set(writer, state));
        }
        if change != PaintPropertyChangeType::Unchanged {
            self.lifecycle.AddChanged(writer, change);
        }
        change
    }

    /// Source: scroll_paint_property_node.cc:73-79.
    pub fn ClearChangedToRoot(&self, access: &PropertyAccess, sequence_number: i32) {
        let mut current = Some(self);
        while let Some(node) = current {
            if node.lifecycle.changed_sequence_number(access) == sequence_number {
                break;
            }
            node.lifecycle.clear_changed(access, sequence_number);
            current = node.Parent(access).map(|parent| &**parent);
        }
    }

    /// Source: scroll_paint_property_node.cc:81-142. Snap-container output is
    /// intentionally absent until the producer can publish the complete cc
    /// value; supported non-root snapshots prove that absence explicitly.
    pub fn ToJSON(&self) -> serde_json::Value {
        crate::property_cell::with_access(|access| self.ToJSONIn(access))
    }
    fn ToJSONIn(&self, access: &PropertyAccess) -> serde_json::Value {
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

        let mut json = PaintPropertyNodeToJSON(self.Parent(access), &self.lifecycle);
        if self.ContainerRect(access).width != 0.0 && self.ContainerRect(access).height != 0.0 {
            json.insert(
                "containerRect".into(),
                serde_json::Value::String(rect_string(self.ContainerRect(access))),
            );
        }
        if self.ContentsRect(access).width != 0.0 && self.ContentsRect(access).height != 0.0 {
            json.insert(
                "contentsRect".into(),
                serde_json::Value::String(rect_string(self.ContentsRect(access))),
            );
        }
        if let Some(clip) = self.OverflowClipNode(access) {
            json.insert(
                "overflowClipNode".into(),
                serde_json::Value::String(format!("{:p}", Arc::as_ptr(clip))),
            );
        }
        if self.UserScrollable(access) {
            let value = if self.UserScrollableHorizontal(access) {
                if self.UserScrollableVertical(access) {
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
        if self.GetMainThreadRepaintReasons(access) != 0 {
            json.insert(
                "mainThreadReasons".into(),
                serde_json::Value::String(repaint_reasons(self.GetMainThreadRepaintReasons(access))),
            );
        }
        if self.MaxScrollOffsetAffectedByPageScale(access) {
            json.insert(
                "maxScrollOffsetAffectedByPageScale".into(),
                serde_json::Value::String("true".into()),
            );
        }
        let compositor_id = self.GetCompositorElementId(access).GetInternalValue();
        if compositor_id != 0 {
            json.insert(
                "compositorElementId".into(),
                serde_json::Value::String(format!("({compositor_id})")),
            );
        }
        let overscroll = self.OverscrollBehavior(access);
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
        if self.PreventScrollAxisLocking(access) {
            json.insert(
                "preventScrollAxisLock".into(),
                serde_json::Value::Bool(true),
            );
        }
        let infinite = foundation::infinite_int_rect::InfiniteIntRect();
        let cull = self.ScrollingContentsCullRect(access);
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
        // Source: transform_paint_property_node.cc:117-122 and its clip, effect
        // and scroll equivalents: Root() returns DEFINE_STATIC_LOCAL sentinels of
        // the single painting thread. Rechrom paints several Pages per process,
        // one document sequence each, so the sentinels belong to the calling
        // sequence: their mutable members (changed markers, cc node ids) are
        // written by that Page's PaintArtifactCompositor only.
        thread_local! {
            static ROOT: PropertyTreeState = PropertyTreeState::CreateRoot();
        }
        ROOT.with(PropertyTreeState::clone)
    }
}

impl PropertyTreeState {
    fn CreateRoot() -> Self {
        {
            let infinite = foundation::infinite_int_rect::InfiniteIntRect();
            let infinite = PaintRect {
                x: infinite.x() as f64,
                y: infinite.y() as f64,
                width: infinite.width() as f64,
                height: infinite.height() as f64,
            };
            // ScrollRoot is a sentinel, not a real scroll. Its nullable overflow
            // clip also avoids manufacturing a cyclic root clip relationship.
            let scroll = Arc::new(ScrollPaintPropertyNode::new(
                PaintPropertyNodeLifecycle::root(),
                0,
                ScrollPaintPropertyNodeState {
                    compositor: Default::default(),
                    owner_client_id: 0,
                    owner_node_id: 0,
                    parent: None,
                    overflow_clip: None,
                    container_rect: infinite,
                    contents_rect: infinite,
                    minimum_scroll_offset: Offset::default(),
                    maximum_scroll_offset: Offset::default(),
                    user_scrollable_horizontal: false,
                    user_scrollable_vertical: false,
                },
            ));
            let transform = Arc::new(TransformPaintPropertyNode::new(PaintPropertyNodeLifecycle::root(), 0, TransformPaintPropertyNodeState {
                compositor: TransformCompositorState {
                    in_subtree_of_page_scale: false,
                    ..Default::default()
                },
                scroll_parent_scroll_translation: None,
                parent: None,
                matrix: TransformMatrix::default(),
                origin: [0.0; 3],
                flattens_inherited_transform: false,
                rendering_context_id: 0,
                backface_visibility: PaintBackfaceVisibility::Inherited,
                scroll: Some(scroll),
                sticky_constraint: None,
                direct_compositing_reasons: Default::default(),
            }));
            let clip = Arc::new(
                ClipPaintPropertyNode::new(
                    PaintPropertyNodeLifecycle::root(),
                    0,
                    ClipPaintPropertyNodeState {
                        layout_clip_rects: Default::default(),
                        parent: None,
                        local_transform_space: transform.clone(),
                        rect: None,
                        radii: PaintCornerRadii::default(),
                        clip_path: None,
                        clip_path_error: None,
                        pixel_moving_filter: None,
                    },
                )
                .with_layout_clips(),
            );
            let effect = Arc::new(
                EffectPaintPropertyNode::new(PaintPropertyNodeLifecycle::root(), 0, EffectPaintPropertyNodeState {
                    compositor: Default::default(),
                    backdrop_filter_bounds: None,
                    self_or_ancestor_participates_in_view_transition: false,
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
                })
                .with_inherited_view_transition(),
            );
            PropertyTreeState {
                transform,
                clip,
                effect,
            }
        }
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
    pub fn ClearChangedToRoot(&self, access: &PropertyAccess, sequence_number: i32) {
        self.Transform().ClearChangedToRoot(access, sequence_number);
        self.Clip().ClearChangedToRoot(access, sequence_number);
        self.Effect().ClearChangedToRoot(access, sequence_number);
    }
    pub fn Changed(
        &self,
        access: &PropertyAccess,
        change: PaintPropertyChangeType,
        relative_to: &Self,
    ) -> bool {
        crate::property_node_geometry::PropertyStateChanged(access, self, change, relative_to)
    }
    pub fn ChangedToRoot(&self, access: &PropertyAccess, change: PaintPropertyChangeType) -> bool {
        self.Changed(access, change, &Self::Root())
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
    pub fn nearest_scroll(&self) -> Option<Arc<ScrollPaintPropertyNode>> {
        crate::property_cell::with_access(|access| {
            let mut transform = Some(&self.transform);
            while let Some(node) = transform {
                if let Some(scroll) = node.ScrollNode(access) {
                    return Some(scroll.clone());
                }
                transform = node.Parent(access);
            }
            None
        })
    }
}

/// ObjectPaintProperties owns one slot per property kind and fragment. The
/// source-specific transform slots mirror ObjectPaintProperties roles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PaintPropertyRole {
    ViewportClip,
    OverscrollElasticityTransform,
    PageScale,
    VisualViewportScroll,
    VisualViewportScrollTranslation,
    PaintOffsetTranslation,
    StickyTranslation,
    ElementCanvasTransform,
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PaintPropertyOwner {
    Viewport,
    Fragment { client: u64, fragment: u32 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PaintPropertyKey {
    pub owner: PaintPropertyOwner,
    pub role: PaintPropertyRole,
}

/// Persistent Create-or-Update slots; source: ObjectPaintProperties::Update /
/// Clear in core/paint/object_paint_properties.h. The map is scoped to one
/// retained paint lifecycle, never global and never keyed by DOM order.
/// A slot holds one live node updated in place; artifacts that still name a
/// removed node keep it alive. Removing a slot and later adding it creates a
/// new node.
/// One kind of ObjectPaintProperties slot node by slot key. Reads go through
/// Deref; writes go through `insert` and `retain`.
#[derive(Debug)]
pub struct KeyedPropertyNodes<T> {
    by_key: BTreeMap<PaintPropertyKey, Arc<T>>,
}
impl<T> Clone for KeyedPropertyNodes<T> {
    /// Clones the slot map: the same live node `Arc`s.
    fn clone(&self) -> Self {
        Self {
            by_key: self.by_key.clone(),
        }
    }
}
impl<T> Default for KeyedPropertyNodes<T> {
    fn default() -> Self {
        Self {
            by_key: BTreeMap::new(),
        }
    }
}
impl<T> std::ops::Deref for KeyedPropertyNodes<T> {
    type Target = BTreeMap<PaintPropertyKey, Arc<T>>;
    fn deref(&self) -> &Self::Target {
        &self.by_key
    }
}
impl<'a, T> IntoIterator for &'a KeyedPropertyNodes<T> {
    type Item = (&'a PaintPropertyKey, &'a Arc<T>);
    type IntoIter = std::collections::btree_map::Iter<'a, PaintPropertyKey, Arc<T>>;
    fn into_iter(self) -> Self::IntoIter {
        self.by_key.iter()
    }
}
impl<T> FromIterator<(PaintPropertyKey, Arc<T>)> for KeyedPropertyNodes<T> {
    fn from_iter<I: IntoIterator<Item = (PaintPropertyKey, Arc<T>)>>(iter: I) -> Self {
        let mut nodes = Self::default();
        for (key, node) in iter {
            nodes.insert(key, node);
        }
        nodes
    }
}
impl<T> KeyedPropertyNodes<T> {
    /// Store `node` in `key`'s slot.
    pub fn insert(&mut self, key: PaintPropertyKey, node: Arc<T>) -> Option<Arc<T>> {
        self.by_key.insert(key, node)
    }
    pub fn retain(&mut self, keep: impl FnMut(&PaintPropertyKey, &mut Arc<T>) -> bool) {
        self.by_key.retain(keep);
    }
}

/// One admitted retained scroll or sticky translation value for a resident
/// slot node (PaintPropertyNodeStore::retained_scroll_update).
pub struct RetainedScrollTranslationUpdate {
    node: Arc<TransformPaintPropertyNode>,
    matrix: TransformMatrix,
    sticky_constraint: Option<StickyPositionPaintData>,
}

#[derive(Clone, Debug)]
pub struct PaintPropertyNodeStore {
    next_id: u64,
    visual_viewport_unique_id: Option<u64>,
    transforms: KeyedPropertyNodes<TransformPaintPropertyNode>,
    clips: KeyedPropertyNodes<ClipPaintPropertyNode>,
    effects: KeyedPropertyNodes<EffectPaintPropertyNode>,
    scrolls: KeyedPropertyNodes<ScrollPaintPropertyNode>,
    // A cleared published graph stays cleared until an updater marks a node.
    // This avoids scanning all resident slots on entirely clean paint passes.
    change_markers_cleared: bool,
    pub changed: PaintPropertyChangeType,
    /// Counts the updates after which PaintArtifactCompositor needs a full
    /// update (see `NoteCompositorChange`); carried with each published
    /// artifact as `PaintArtifact::property_tree_update_generation`.
    compositor_update_generation: u64,
    /// Source PaintPropertyTreeBuilder.cc:4754-4763, effective local changes.
    pub intersection_observation_state:
        foundation::intersection_observation_state::IntersectionObservationState,
    /// VisualViewport::needs_paint_property_update_ (visual_viewport.h): the
    /// inputs the viewport nodes were last updated from, with those nodes.
    viewport_nodes: Option<(ViewportNodeInputs, ViewportNodes)>,
}
type ViewportNodes = (
    Arc<TransformPaintPropertyNode>,
    Arc<TransformPaintPropertyNode>,
    Arc<TransformPaintPropertyNode>,
);
#[derive(Clone, Copy, Debug, PartialEq)]
struct ViewportNodeInputs {
    viewport: PaintRect,
    visual_offset: Option<(foundation::CompositorElementId, foundation::gfx::Vector2dF)>,
    visual_state: Option<crate::PaintVisualViewportState>,
}
impl Default for PaintPropertyNodeStore {
    fn default() -> Self {
        Self {
            next_id: 1,
            visual_viewport_unique_id: None,
            transforms: Default::default(),
            clips: Default::default(),
            effects: Default::default(),
            scrolls: Default::default(),
            change_markers_cleared: true,
            changed: Default::default(),
            compositor_update_generation: NextCompositorUpdateGeneration(),
            intersection_observation_state: Default::default(),
            viewport_nodes: None,
        }
    }
}
/// Generations are unique in the process, so artifacts of a replaced store
/// (a new document) never compare equal to an older store's.
fn NextCompositorUpdateGeneration() -> u64 {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}
macro_rules! update_slot {
    ($name:ident, $node:ty, $map:ident, $update:expr, $desired_above:expr, $direct_scroll:expr) => {
        /// Source: object_paint_properties.h:528-551 `Update<NodeType>`.
        /// Creates the slot's node (kNodeAddedOrRemoved) or updates the
        /// resident node in place (`Node::Update`) and returns it. `node` is
        /// the builder's value: its id is the slot id (`SlotId`) and its
        /// state the new `State`. A node without an owner slot (no source
        /// DisplayItemClient) is a new node on every update.
        pub fn $name(
            &mut self,
            writer: &mut PropertyWriter,
            key: Option<PaintPropertyKey>,
            node: $node,
        ) -> (Arc<$node>, PaintPropertyChangeType) {
            let added = PaintPropertyChangeType::NodeAddedOrRemoved;
            let desired_above: fn(PaintPropertyKey) -> Option<PaintPropertyChangeType> =
                $desired_above;
            let Some(key) = key else {
                self.changed = added;
                self.change_markers_cleared = false;
                self.NoteCompositorChange(added, false);
                return (Arc::new(node), added);
            };
            if let Some(slot) = self.$map.get(&key).filter(|slot| slot.id == node.id).cloned() {
                let update: fn(&$node, &mut PropertyWriter, $node) -> PaintPropertyChangeType =
                    $update;
                let direct_scroll: fn(PaintPropertyKey, &$node, &PropertyAccess) -> bool =
                    $direct_scroll;
                let change = update(&slot, writer, node);
                self.record_change(change, desired_above(key));
                self.NoteCompositorChange(change, direct_scroll(key, &slot, writer));
                return (slot, change);
            }
            let node = Arc::new(node);
            self.$map.insert(key, node.clone());
            self.record_change(added, desired_above(key));
            self.NoteCompositorChange(added, false);
            (node, added)
        }
    };
}

impl PaintPropertyNodeStore {
    /// Begins one PrePaint pass of slot updates: the pass's maximum change
    /// and intersection-observation effect start unchanged.
    pub fn BeginUpdate(&mut self) {
        self.changed = PaintPropertyChangeType::Unchanged;
        self.intersection_observation_state = Default::default();
    }

    /// The node id of `key`'s resident slot, or a new id for the node an
    /// update of that slot (or an unowned node) creates.
    pub fn SlotId(&mut self, key: Option<PaintPropertyKey>) -> u64 {
        if let Some(key) = key {
            let resident = self
                .transforms
                .get(&key)
                .map(|node| node.id)
                .or_else(|| self.clips.get(&key).map(|node| node.id))
                .or_else(|| self.effects.get(&key).map(|node| node.id))
                .or_else(|| self.scrolls.get(&key).map(|node| node.id));
            if let Some(id) = resident {
                return id;
            }
        }
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Source pp_tree_builder.cc:4754-4763: a change of a node raises the
    /// pass's change and its intersection-observation effect. Above
    /// `desired_above` (or for any change when it is None) observations are
    /// fully invalidated, otherwise only scroll/visibility tracking is.
    fn record_change(
        &mut self,
        change: PaintPropertyChangeType,
        desired_above: Option<PaintPropertyChangeType>,
    ) {
        use foundation::intersection_observation_state::IntersectionObservationState;
        if change == PaintPropertyChangeType::Unchanged {
            return;
        }
        self.changed = self.changed.max(change);
        self.change_markers_cleared = false;
        let observation = match desired_above {
            Some(threshold) if change > threshold => IntersectionObservationState::Desired,
            _ => IntersectionObservationState::ScrollAndVisibilityOnly,
        };
        self.intersection_observation_state = self.intersection_observation_state.max(observation);
    }

    /// A direct composited-value update outside a PrePaint pass raises the
    /// store's change but no intersection-observation effect.
    fn record_direct_change(&mut self, change: PaintPropertyChangeType) {
        if change != PaintPropertyChangeType::Unchanged {
            self.changed = self.changed.max(change);
            self.change_markers_cleared = false;
        }
    }

    /// Source: PaintPropertyTreeBuilder (paint_property_tree_builder.cc,
    /// `property_changed > PaintPropertyChangeType::kChangedOnlyCompositedValues`
    /// -> `LocalFrameView::SetPaintArtifactCompositorNeedsUpdate`). A change
    /// above composited values needs a full PaintArtifactCompositor update,
    /// except a simple-values change of a compositor scroll translation:
    /// Chromium's UpdateScrollAndScrollTranslation applies that one through
    /// `DirectlyUpdateScrollOffsetTransform` first (paint_property_tree_builder.cc:
    /// 3798-3827), which Rechrom runs in LayerizeEngine (decision D3), and
    /// Layerize requests the full update itself when that direct update fails.
    /// The flag of Chromium's frame view crosses the Paint -> Layerize boundary
    /// as this generation on each published artifact.
    fn NoteCompositorChange(&mut self, change: PaintPropertyChangeType, scroll_offset: bool) {
        let direct_scroll =
            scroll_offset && change == PaintPropertyChangeType::ChangedOnlySimpleValues;
        if change > PaintPropertyChangeType::ChangedOnlyCompositedValues && !direct_scroll {
            self.compositor_update_generation = NextCompositorUpdateGeneration();
        }
    }

    /// The generation of updates that need a full PaintArtifactCompositor
    /// update; a published artifact carries the value of its publication.
    pub fn CompositorUpdateGeneration(&self) -> u64 {
        self.compositor_update_generation
    }

    update_slot!(
        UpdateTransform,
        TransformPaintPropertyNode,
        transforms,
        |slot, writer, node| {
            let state = node.state.into_inner();
            // Chromium's deferred transform path classifies a sampled matrix
            // as a composited-value change once the resident node is already
            // an active transform-animation node (see transform_state_change).
            let animation_state = TransformAnimationState {
                is_running_animation_on_compositor: slot
                    .state(writer)
                    .HasActiveTransformAnimation()
                    && state.HasActiveTransformAnimation(),
            };
            slot.Update(writer, state, animation_state)
        },
        |key| {
            Some(
                if matches!(
                    key.role,
                    PaintPropertyRole::ScrollTranslation
                        | PaintPropertyRole::VisualViewportScrollTranslation
                ) {
                    PaintPropertyChangeType::ChangedOnlySimpleValues
                } else {
                    PaintPropertyChangeType::Unchanged
                },
            )
        },
        // The compositor scroll translations (compositor_scroll_translations).
        |key, node, access| {
            key.role == PaintPropertyRole::ScrollTranslation && node.ScrollNode(access).is_some()
        }
    );
    update_slot!(
        UpdateClip,
        ClipPaintPropertyNode,
        clips,
        |slot, writer, node| slot.Update(writer, node.state.into_inner()),
        |_| Some(PaintPropertyChangeType::Unchanged),
        |_, _, _| false
    );
    update_slot!(
        UpdateEffect,
        EffectPaintPropertyNode,
        effects,
        |slot, writer, node| {
            slot.Update(writer, node.state.into_inner(), EffectAnimationState::default())
        },
        |_| None,
        |_, _, _| false
    );
    update_slot!(
        UpdateScroll,
        ScrollPaintPropertyNode,
        scrolls,
        |slot, writer, node| slot.Update(writer, node.state.into_inner()),
        |_| Some(PaintPropertyChangeType::Unchanged),
        |_, _, _| false
    );

    /// An in-place `Update` of a resident effect node reached through a
    /// property state rather than its slot (the logical out-of-flow parent
    /// fix-up), with the same change bookkeeping.
    pub fn UpdateEffectNode(
        &mut self,
        writer: &mut PropertyWriter,
        node: &EffectPaintPropertyNode,
        state: EffectPaintPropertyNodeState,
    ) -> PaintPropertyChangeType {
        let change = node.Update(writer, state, EffectAnimationState::default());
        self.record_change(change, None);
        self.NoteCompositorChange(change, false);
        change
    }

    /// Source: ObjectPaintProperties::Clear* and FragmentData destruction.
    /// Removes every fragment slot `live` does not name after a PrePaint
    /// pass; artifacts that still reference a removed node keep it alive.
    /// The visual viewport's own nodes stay while the frame view lives.
    pub fn RetainLiveSlots(&mut self, live: impl Fn(&PaintPropertyKey) -> bool) {
        use foundation::intersection_observation_state::IntersectionObservationState;
        let keep = |key: &PaintPropertyKey| {
            live(key)
                || (key.owner == PaintPropertyOwner::Viewport
                    && key.role != PaintPropertyRole::ViewportClip)
        };
        let geometry_before = self.transforms.len() + self.clips.len() + self.scrolls.len();
        let before = geometry_before + self.effects.len();
        self.transforms.retain(|key, _| keep(key));
        self.clips.retain(|key, _| keep(key));
        self.effects.retain(|key, _| keep(key));
        self.scrolls.retain(|key, _| keep(key));
        let geometry_after = self.transforms.len() + self.clips.len() + self.scrolls.len();
        if before != geometry_after + self.effects.len() {
            self.changed = PaintPropertyChangeType::NodeAddedOrRemoved;
            self.NoteCompositorChange(PaintPropertyChangeType::NodeAddedOrRemoved, false);
            crate::property_cell::ClearTransformCache();
            crate::property_cell::ClearClipCache();
            let update = if geometry_before != geometry_after {
                IntersectionObservationState::Desired
            } else {
                IntersectionObservationState::ScrollAndVisibilityOnly
            };
            self.intersection_observation_state = self.intersection_observation_state.max(update);
        }
        self.change_markers_cleared = self.changed == PaintPropertyChangeType::Unchanged;
    }

    /// Borrow an ObjectPaintProperties slot by its existing owner and role.
    /// Query adapters retain neither another property graph nor a snapshot.
    pub fn TransformForKey(&self, key: PaintPropertyKey) -> Option<&TransformPaintPropertyNode> {
        self.transforms.get(&key).map(Arc::as_ref)
    }

    pub fn TransformOwnerForKey(
        &self,
        key: PaintPropertyKey,
    ) -> Option<&Arc<TransformPaintPropertyNode>> {
        self.transforms.get(&key)
    }

    pub fn EffectOwnerForKey(
        &self,
        key: PaintPropertyKey,
    ) -> Option<&Arc<EffectPaintPropertyNode>> {
        self.effects.get(&key)
    }

    /// Source: object_paint_properties.h:503-522
    /// DirectlyUpdateTransformAndOrigin / DirectlyUpdateOpacity: the slot's
    /// resident transform node is updated in place, every artifact that
    /// references it observes the new value.
    pub fn DirectlyUpdateTransform(
        &mut self,
        writer: &mut PropertyWriter,
        node: &TransformPaintPropertyNode,
        matrix: TransformMatrix,
        origin: [f64; 3],
        animation_state: TransformAnimationState,
    ) -> PaintPropertyChangeType {
        let change = node.DirectlyUpdateTransformAndOrigin(writer, matrix, origin, animation_state);
        self.record_direct_change(change);
        self.NoteCompositorChange(change, false);
        change
    }

    pub fn DirectlyUpdateOpacity(
        &mut self,
        writer: &mut PropertyWriter,
        node: &EffectPaintPropertyNode,
        opacity: f32,
        animation_state: EffectAnimationState,
    ) -> PaintPropertyChangeType {
        let change = node.DirectlyUpdateOpacity(writer, opacity, animation_state);
        self.record_direct_change(change);
        self.NoteCompositorChange(change, false);
        change
    }

    pub fn ClipForKey(&self, key: PaintPropertyKey) -> Option<&ClipPaintPropertyNode> {
        self.clips.get(&key).map(Arc::as_ref)
    }

    pub fn EffectForKey(&self, key: PaintPropertyKey) -> Option<&EffectPaintPropertyNode> {
        self.effects.get(&key).map(Arc::as_ref)
    }

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
            (key.role == PaintPropertyRole::ScrollTranslation
                && crate::property_cell::with_access(|access| node.ScrollNode(access).is_some()))
                .then_some(node)
        })
    }
    pub fn root_scroller_scroll_translation(&self) -> Option<&Arc<TransformPaintPropertyNode>> {
        self.transforms.iter().find_map(|(key, node)| {
            (key.role == PaintPropertyRole::ScrollTranslation
                && crate::property_cell::with_access(|access| {
                    node.RequiresCompositingForRootScroller(access)
                }))
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
        writer: &mut PropertyWriter,
        viewport: PaintRect,
        visual_offset: Option<(foundation::CompositorElementId, foundation::gfx::Vector2dF)>,
        visual_state: Option<crate::PaintVisualViewportState>,
    ) -> (
        Arc<TransformPaintPropertyNode>,
        Arc<TransformPaintPropertyNode>,
        Arc<TransformPaintPropertyNode>,
    ) {
        // VisualViewport::UpdatePaintPropertyNodesIfNeeded returns early
        // unless needs_paint_property_update_ (visual_viewport.cc:96-102); the
        // flag is set only when the viewport's size, scale or offset change.
        // The same resident nodes stay valid while those inputs are equal.
        let inputs = ViewportNodeInputs {
            viewport,
            visual_offset,
            visual_state,
        };
        if let Some((previous, nodes)) = &self.viewport_nodes {
            let resident = |role, node: &Arc<TransformPaintPropertyNode>| {
                self.transforms
                    .get(&PaintPropertyKey {
                        owner: PaintPropertyOwner::Viewport,
                        role,
                    })
                    .is_some_and(|current| Arc::ptr_eq(current, node))
            };
            if *previous == inputs
                && resident(PaintPropertyRole::OverscrollElasticityTransform, &nodes.0)
                && resident(PaintPropertyRole::PageScale, &nodes.1)
                && resident(PaintPropertyRole::VisualViewportScrollTranslation, &nodes.2)
            {
                return nodes.clone();
            }
        }
        let roots = PropertyTreeState::default();
        let unique_id = if let Some(state) = visual_state {
            if let Some(id) = self.visual_viewport_unique_id {
                assert_eq!(id, state.unique_id, "same native VisualViewport owner");
            }
            self.visual_viewport_unique_id = Some(state.unique_id);
            state.unique_id
        } else {
            *self
                .visual_viewport_unique_id
                .get_or_insert_with(foundation::NewUniqueObjectId)
        };
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
        let overscroll = TransformPaintPropertyNode::new(Default::default(), self.SlotId(Some(overscroll_key)), TransformPaintPropertyNodeState {
            compositor: TransformCompositorState {
                in_subtree_of_page_scale: false,
                ..Default::default()
            },
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
        });
        let (overscroll, _) = self.UpdateTransform(writer, Some(overscroll_key), overscroll);

        let page_scale_key = PaintPropertyKey {
            owner: PaintPropertyOwner::Viewport,
            role: PaintPropertyRole::PageScale,
        };
        let page_scale = TransformPaintPropertyNode::new(Default::default(), self.SlotId(Some(page_scale_key)), TransformPaintPropertyNodeState {
            compositor: TransformCompositorState {
                compositor_element_id: page_scale_element_id,
                in_subtree_of_page_scale: false,
                ..Default::default()
            },
            parent: Some(overscroll.clone()),
            matrix: {
                let mut matrix = TransformMatrix::default();
                if let Some(state) = visual_state {
                    matrix.values[0] = f64::from(state.scale);
                    matrix.values[5] = f64::from(state.scale);
                }
                matrix
            },
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
        });
        let (page_scale, _) = self.UpdateTransform(writer, Some(page_scale_key), page_scale);

        let visual_scroll_key = PaintPropertyKey {
            owner: PaintPropertyOwner::Viewport,
            role: PaintPropertyRole::VisualViewportScroll,
        };
        let visual_scroll = ScrollPaintPropertyNode::new(Default::default(), self.SlotId(Some(visual_scroll_key)), ScrollPaintPropertyNodeState {
            compositor: ScrollCompositorState {
                compositor_element_id: scroll_element_id,
                max_scroll_offset_affected_by_page_scale: true,
                scrolling_contents_cull_rect: viewport,
                snap_container_data_absent: true,
                ..Default::default()
            },
            owner_client_id: 0,
            owner_node_id: 0,
            parent: Some(
                roots
                    .transform
                    .ScrollNode(writer)
                    .expect("root transform owns the root scroll node")
                    .clone(),
            ),
            overflow_clip: None,
            container_rect: visual_state.map_or(viewport, |s| PaintRect {
                x: 0.,
                y: 0.,
                width: f64::from(s.size.width()),
                height: f64::from(s.size.height()),
            }),
            contents_rect: visual_state.map_or(viewport, |s| PaintRect {
                x: 0.,
                y: 0.,
                width: f64::from(s.contents_size.width()),
                height: f64::from(s.contents_size.height()),
            }),
            minimum_scroll_offset: Offset::default(),
            maximum_scroll_offset: visual_state.map_or_else(Offset::default, |s| Offset {
                x: f64::from(s.maximum_scroll_offset.x()),
                y: f64::from(s.maximum_scroll_offset.y()),
            }),
            user_scrollable_horizontal: true,
            user_scrollable_vertical: true,
        });
        let (visual_scroll, _) = self.UpdateScroll(writer, Some(visual_scroll_key), visual_scroll);

        let visual_translation_key = PaintPropertyKey {
            owner: PaintPropertyOwner::Viewport,
            role: PaintPropertyRole::VisualViewportScrollTranslation,
        };
        let mut visual_translation = TransformPaintPropertyNode::new(Default::default(), self.SlotId(Some(visual_translation_key)), TransformPaintPropertyNodeState {
            compositor: Default::default(),
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
        });
        // VisualViewport::UpdatePaintPropertyNodesIfNeeded: the native owner
        // supplies offset_ for this SAME existing element/translation slot.
        let source_offset = visual_state
            .map(|s| (scroll_element_id, s.offset))
            .or(visual_offset);
        if let Some((element, offset)) = source_offset {
            assert_eq!(
                element, scroll_element_id,
                "same VisualViewport source identity"
            );
            visual_translation.state_mut().matrix.values[12] = -f64::from(offset.x());
            visual_translation.state_mut().matrix.values[13] = -f64::from(offset.y());
        }
        let visual_translation =
            self.UpdateTransform(writer, Some(visual_translation_key), visual_translation).0;
        let nodes = (overscroll, page_scale, visual_translation);
        self.viewport_nodes = Some((inputs, nodes.clone()));
        nodes
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
        crate::property_cell::with_access(|access| {
        self.transforms
            .iter()
            .filter(|(key, _)| key.role == PaintPropertyRole::ScrollTranslation)
            .map(|(key, node)| {
                (
                    key.owner,
                    Offset {
                        x: -node.state(access).matrix.values[12],
                        y: -node.state(access).matrix.values[13],
                    },
                )
            })
            .collect()
        })
    }

    /// Current sticky values carried by the source property owner. Recording
    /// basis changes may rebase scroll while preserving these existing slots.
    pub fn sticky_offsets(
        &self,
    ) -> BTreeMap<PaintPropertyOwner, (Offset, StickyPositionPaintData)> {
        crate::property_cell::with_access(|access| {
        self.transforms
            .iter()
            .filter_map(|(key, node)| {
                (key.role == PaintPropertyRole::StickyTranslation)
                    .then(|| {
                        node.state(access).sticky_constraint.map(|constraint| {
                            (
                                key.owner,
                                (
                                    Offset {
                                        x: node.state(access).matrix.values[12],
                                        y: node.state(access).matrix.values[13],
                                    },
                                    constraint,
                                ),
                            )
                        })
                    })
                    .flatten()
            })
            .collect()
        })
    }

    /// Retained source geometry has already proved unchanged owners/topology,
    /// styles, resources and fragment geometry. Update existing real scroll
    /// translations without constructing a new PrePaint tree. The caller must
    /// admit the returned translation updates (ApplyRetainedScrollUpdate).
    ///
    /// Source: paint_property_tree_builder.cc:3769-3827 and
    /// property_tree_manager.cc:138-176.
    pub fn retained_scroll_update(
        &self,
        offsets: &BTreeMap<PaintPropertyOwner, Offset>,
        sticky: &BTreeMap<PaintPropertyOwner, (Offset, StickyPositionPaintData)>,
    ) -> Option<Vec<RetainedScrollTranslationUpdate>> {
        crate::property_cell::with_access(|access| {
        let mut translations = Vec::new();
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
                x: -node.state(access).matrix.values[12],
                y: -node.state(access).matrix.values[13],
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
                || node.state(access).scroll.as_ref().is_none_or(|scroll| scroll.id == 0)
                || node.state(access).origin != [0.0; 3]
            {
                return None;
            }
            let mut linear = node.state(access).matrix.values;
            linear[12] = 0.0;
            linear[13] = 0.0;
            if linear != TransformMatrix::default().values {
                return None;
            }
            let mut matrix = node.state(access).matrix;
            matrix.values[12] = -offset.x;
            matrix.values[13] = -offset.y;
            translations.push(RetainedScrollTranslationUpdate {
                node: node.clone(),
                matrix,
                sticky_constraint: node.state(access).sticky_constraint,
            });
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
            let Some(before) = node.state(access).sticky_constraint else {
                return None;
            };
            let current_offset = Offset {
                x: node.state(access).matrix.values[12],
                y: node.state(access).matrix.values[13],
            };
            if *offset == current_offset && before == *constraint {
                continue;
            }
            if !offset.x.is_finite()
                || !offset.y.is_finite()
                || node.state(access).origin != [0.0; 3]
                || !before.same_geometry(constraint)
            {
                return None;
            }
            let mut linear = node.state(access).matrix.values;
            linear[12] = 0.0;
            linear[13] = 0.0;
            if linear != TransformMatrix::default().values {
                return None;
            }
            let mut matrix = node.state(access).matrix;
            matrix.values[12] = offset.x;
            matrix.values[13] = offset.y;
            translations.push(RetainedScrollTranslationUpdate {
                node: node.clone(),
                matrix,
                sticky_constraint: Some(*constraint),
            });
        }
        Some(translations)
        })
    }

    /// Source: paint_property_tree_builder.cc:3769-3827 (UpdateScrollTranslation)
    /// and UpdateStickyTranslation: the admitted retained scroll and sticky
    /// translations are updated in place on their resident slot nodes; every
    /// artifact and PrePaint state referencing them observes the new values.
    /// A sticky constraint change is a value change, a translation-only
    /// change a simple-values change (the classification the retained
    /// scroll path has always published).
    pub fn ApplyRetainedScrollUpdate(
        &mut self,
        writer: &mut PropertyWriter,
        updates: &[RetainedScrollTranslationUpdate],
    ) {
        for update in updates {
            let (matrix_changed, sticky_changed) = {
                let state = update.node.state(writer);
                (
                    state.matrix != update.matrix,
                    state.sticky_constraint != update.sticky_constraint,
                )
            };
            let change = if sticky_changed {
                PaintPropertyChangeType::ChangedOnlyValues
            } else if matrix_changed {
                PaintPropertyChangeType::ChangedOnlySimpleValues
            } else {
                continue;
            };
            update.node.state.modify(writer, |state| {
                state.matrix = update.matrix;
                state.sticky_constraint = update.sticky_constraint;
            });
            update.node.lifecycle.AddChanged(writer, change);
            crate::property_cell::ClearTransformCache();
            crate::property_cell::ClearClipCache();
            self.record_direct_change(change);
            let scroll_offset = update.node.ScrollNode(writer).is_some();
            self.NoteCompositorChange(change, scroll_offset);
        }
    }

    pub fn viewport_clip(&self) -> Option<&Arc<ClipPaintPropertyNode>> {
        self.clips.get(&PaintPropertyKey {
            owner: PaintPropertyOwner::Viewport,
            role: PaintPropertyRole::ViewportClip,
        })
    }

    pub fn clear_owner(&mut self, owner: PaintPropertyOwner) {
        self.transforms.retain(|k, _| k.owner != owner);
        self.clips.retain(|k, _| k.owner != owner);
        self.effects.retain(|k, _| k.owner != owner);
        self.scrolls.retain(|k, _| k.owner != owner);
    }
    pub fn next_id(&self) -> u64 {
        self.next_id
    }
}

// Compare linked node entities, not ancestor values:
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
    a: &TransformPaintPropertyNodeState,
    b: &TransformPaintPropertyNodeState,
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
#[cfg(test)]
fn transform_state_change_with_animation(
    a: &TransformPaintPropertyNode,
    b: &TransformPaintPropertyNode,
    animation_state: TransformAnimationState,
) -> PaintPropertyChangeType {
    crate::property_cell::with_access(|access| {
        transform_node_state_change(a.state(access), b.state(access), animation_state)
    })
}
fn transform_node_state_change(
    a: &TransformPaintPropertyNodeState,
    b: &TransformPaintPropertyNodeState,
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

#[cfg(test)]
fn transform_state_change(
    a: &TransformPaintPropertyNode,
    b: &TransformPaintPropertyNode,
) -> PaintPropertyChangeType {
    // Chromium's deferred transform path classifies a sampled matrix as a
    // composited-value change once the resident property node is already an
    // active transform-animation node. Rechrom samples CSS keyframes on the
    // document sequence before forwarding that value through the same
    // PaintArtifactCompositor direct-update API, so preserve that change
    // category here. `transform_state_change_with_animation` still returns
    // ChangedOnlyValues for parent, scroll, compositing-reason, or any other
    // structural difference; only matrix/origin values receive the downgrade.
    let directly_sampled_active_animation = crate::property_cell::with_access(|access| {
        a.HasActiveTransformAnimation(access) && b.HasActiveTransformAnimation(access)
    });
    transform_state_change_with_animation(
        a,
        b,
        TransformAnimationState {
            is_running_animation_on_compositor: directly_sampled_active_animation,
        },
    )
}
fn clip_node_state_change(
    a: &ClipPaintPropertyNodeState,
    b: &ClipPaintPropertyNodeState,
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
    a: &EffectPaintPropertyNodeState,
    b: &EffectPaintPropertyNodeState,
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
    a: &EffectPaintPropertyNodeState,
    b: &EffectPaintPropertyNodeState,
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

#[cfg(test)]
fn effect_state_change_with_animation(
    a: &EffectPaintPropertyNode,
    b: &EffectPaintPropertyNode,
    animation_state: EffectAnimationState,
) -> PaintPropertyChangeType {
    crate::property_cell::with_access(|access| {
        effect_node_state_change(a.state(access), b.state(access), animation_state)
    })
}
fn effect_node_state_change(
    a: &EffectPaintPropertyNodeState,
    b: &EffectPaintPropertyNodeState,
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

#[cfg(test)]
fn scroll_state_change(
    a: &ScrollPaintPropertyNode,
    b: &ScrollPaintPropertyNode,
) -> PaintPropertyChangeType {
    crate::property_cell::with_access(|access| {
        scroll_node_state_change(a.state(access), b.state(access))
    })
}
fn scroll_node_state_change(
    a: &ScrollPaintPropertyNodeState,
    b: &ScrollPaintPropertyNodeState,
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

#[cfg(test)]
mod retained_scroll_update_tests {
    use super::*;

    #[test]
    fn compositor_scroll_translation_list_requires_a_scroll_node() {
        let root = PropertyTreeState::default();
        let owner = |client| PaintPropertyOwner::Fragment {
            client,
            fragment: 0,
        };
        let translation = |id, scroll| {
            Arc::new(TransformPaintPropertyNode::new(Default::default(), id, TransformPaintPropertyNodeState {
                compositor: Default::default(),
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
            }))
        };
        let with_scroll = translation(1, crate::property_cell::with_access(|a| root.transform.state(a).scroll.clone()).clone());
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
        assert!(!crate::property_cell::with_access(|a| root.ChangedToRoot(a, PaintPropertyChangeType::ChangedOnlyValues)));

        let mut child_transform = (*root.transform).clone();
        child_transform.id = 101;
        child_transform.state_mut().parent = Some(root.transform.clone());
        child_transform.lifecycle = PaintPropertyNodeLifecycle::default()
            .updated(PaintPropertyChangeType::ChangedOnlyValues);
        child_transform.state_mut().matrix.values[12] = 9.0;
        let child_transform = Arc::new(child_transform);
        let mut state = root.clone();
        state.SetTransform(child_transform.clone());
        assert!(Arc::ptr_eq(state.Transform(), &child_transform));
        assert!(crate::property_cell::with_access(|a| state.Changed(a, PaintPropertyChangeType::ChangedOnlyValues, &root)));
        assert!(crate::property_cell::with_access(|a| state.ChangedToRoot(a, PaintPropertyChangeType::ChangedOnlyValues)));

        let text = state.ToString();
        assert!(text.starts_with("t:"));
        assert!(text.contains(" c:"));
        assert!(text.contains(" e:"));
        let json = state.ToJSON();
        assert!(json["transform"].is_object());
        assert!(json["clip"].is_object());
        assert!(json["effect"].is_object());

        crate::property_cell::with_access(|a| state.ClearChangedToRoot(a, 17));
        assert_eq!(crate::property_cell::with_access(|a| child_transform.lifecycle.changed_sequence_number(a)), 17);
        assert!(!crate::property_cell::with_access(|a| state.ChangedToRoot(a, PaintPropertyChangeType::ChangedOnlyValues)));
    }

    #[test]
    fn transform_node_owns_source_change_queries_and_json() {
        let root = PropertyTreeState::default();
        let mut transform = (*root.transform).clone();
        transform.id = 91;
        transform.state_mut().parent = Some(root.transform.clone());
        transform.lifecycle = PaintPropertyNodeLifecycle::root();

        let mut scaled = transform.clone();
        scaled.state_mut().matrix.values[0] = 2.0;
        assert_eq!(
            transform_state_change(&transform, &scaled),
            PaintPropertyChangeType::ChangedOnlySimpleValues
        );

        let mut skewed = transform.clone();
        skewed.state_mut().matrix.values[4] = 0.5;
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
        origin_changed.state_mut().origin = [2.0, 3.0, 0.0];
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
            .state_mut()
            .direct_compositing_reasons
            .Put(foundation::CompositingReason::kStickyPosition);
        let mut sticky_moved = sticky.clone();
        sticky_moved.state_mut().matrix.values[12] = 10.0;
        assert_eq!(
            transform_state_change(&sticky, &sticky_moved),
            PaintPropertyChangeType::ChangedOnlyCompositedValues
        );

        let mut source_state_changed = transform.clone();
        source_state_changed.state_mut().compositor.visible_frame_element_id =
            foundation::CompositorElementId::new(7);
        assert_eq!(
            transform_state_change(&transform, &source_state_changed),
            PaintPropertyChangeType::ChangedOnlyValues
        );
        let mut auxiliary_changed = transform.clone();
        auxiliary_changed.state_mut().sticky_constraint = Some(StickyPositionPaintData::default());
        assert_eq!(
            transform_state_change(&transform, &auxiliary_changed),
            PaintPropertyChangeType::Unchanged
        );

        let mut translated = transform.clone();
        translated.state_mut().matrix.values[12] = 4.0;
        translated.state_mut().matrix.values[13] = 5.0;
        translated.state_mut().flattens_inherited_transform = false;
        translated.state_mut().compositor.in_subtree_of_page_scale = false;
        translated.state_mut().backface_visibility = PaintBackfaceVisibility::Hidden;
        translated.state_mut().rendering_context_id = 0x2a;
        translated
            .state_mut()
            .direct_compositing_reasons
            .Put(foundation::CompositingReason::kWillChangeTransform);
        let json = translated.ToJSON();
        assert_eq!(json["translation2d"], "4,5");
        assert_eq!(json["flattensInheritedTransform"], false);
        assert_eq!(json["in_subtree_of_page_scale"], false);
        assert_eq!(json["backface"], "hidden");
        assert_eq!(json["renderingContextId"], "2a");

        let mut resident = transform.clone();
        let mut scaled_state = scaled.state_mut().clone();
        scaled_state.parent = Some(root.transform.clone());
        assert_eq!(
            crate::property_cell::with_writer(|w| resident.Update(
                w,
                scaled_state,
                TransformAnimationState::default()
            )),
            PaintPropertyChangeType::ChangedOnlySimpleValues
        );
        assert_eq!(resident.id, 91);
        assert_eq!(resident.state_mut().Matrix().values[0], 2.0);

        let mut direct_matrix = resident.state_mut().matrix;
        direct_matrix.values[12] = 8.0;
        let direct_origin = resident.state_mut().origin;
        assert_eq!(
            crate::property_cell::with_writer(|w| resident.DirectlyUpdateTransformAndOrigin(
                w,
                direct_matrix,
                direct_origin,
                TransformAnimationState {
                    is_running_animation_on_compositor: true,
                }
            )),
            PaintPropertyChangeType::ChangedOnlyCompositedValues
        );
        assert_eq!(resident.state_mut().Matrix().values[12], 8.0);
        crate::property_cell::with_access(|a| resident.ClearChangedToRoot(a, 8));
        assert_eq!(crate::property_cell::with_access(|a| resident.lifecycle.changed_sequence_number(a)), 8);
    }

    #[test]
    fn scroll_node_owns_source_change_queries_and_json() {
        let root = PropertyTreeState::default();
        let root_scroll = crate::property_cell::with_access(|a| {
            root.transform.ScrollNode(a).expect("root transform owns root scroll").clone()
        });
        let mut scroll = (*root_scroll).clone();
        scroll.id = 81;
        scroll.lifecycle = PaintPropertyNodeLifecycle::root();
        scroll.state_mut().parent = Some(root_scroll.clone());
        scroll.state_mut().overflow_clip = Some(root.clip.clone());
        scroll.state_mut().container_rect = PaintRect {
            x: 1.0,
            y: 2.0,
            width: 30.0,
            height: 40.0,
        };
        scroll.state_mut().contents_rect = PaintRect {
            x: 1.0,
            y: 2.0,
            width: 300.0,
            height: 400.0,
        };
        scroll.state_mut().user_scrollable_horizontal = true;
        scroll.state_mut().user_scrollable_vertical = true;
        scroll.state_mut().compositor.main_thread_repaint_reasons = 1 | (1 << 3);
        scroll.state_mut().compositor.max_scroll_offset_affected_by_page_scale = true;
        scroll.state_mut().compositor.prevent_scroll_axis_locking = true;
        scroll.state_mut().compositor.overscroll_behavior = [
            LayoutOverscrollBehavior::kContain,
            LayoutOverscrollBehavior::kNone,
        ];
        scroll.state_mut().compositor.snap_container_data_absent = true;
        scroll.state_mut().compositor.snap_container_data = None;
        scroll.state_mut().compositor.compositor_element_id = foundation::CompositorElementId::new(42);

        crate::property_cell::with_access(|a| {
        assert_eq!(scroll.ContainerRect(a).width, 30.0);
        assert_eq!(scroll.ContentsRect(a).height, 400.0);
        assert!(scroll.UserScrollable(a));
        assert!(scroll.RequiresMainThreadForBackgroundAttachmentFixed(a));
        assert!(scroll.PreventScrollAxisLocking(a));
        assert!(scroll
            .OverflowClipNode(a)
            .is_some_and(|clip| Arc::ptr_eq(clip, &root.clip)));
        });

        let mut cull_changed = scroll.clone();
        cull_changed.state_mut().compositor.scrolling_contents_cull_rect = PaintRect {
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
        auxiliary_changed.state_mut().maximum_scroll_offset.y = 99.0;
        assert_eq!(
            scroll_state_change(&scroll, &auxiliary_changed),
            PaintPropertyChangeType::Unchanged
        );
        let mut source_changed = scroll.clone();
        source_changed
            .state_mut()
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
        let mut next = resident.state_mut().clone();
        next.user_scrollable_vertical = false;
        next.parent = Some(root_scroll);
        assert_eq!(
            crate::property_cell::with_writer(|w| resident.Update(w, next)),
            PaintPropertyChangeType::ChangedOnlyValues
        );
        assert_eq!(resident.id, 81);
        resident.SetScrollingContentsCullRect(PaintRect::default());
        assert_eq!(
            crate::property_cell::with_access(|a| resident.lifecycle.node_changed(a)),
            PaintPropertyChangeType::ChangedOnlyValues
        );
        crate::property_cell::with_access(|a| resident.ClearChangedToRoot(a, 7));
        assert_eq!(crate::property_cell::with_access(|a| resident.lifecycle.changed_sequence_number(a)), 7);
    }

    #[test]
    fn clip_node_owns_source_clip_queries_and_json() {
        let root = PropertyTreeState::default();
        let mut clip = (*root.clip).clone();
        clip.state_mut().rect = Some(PaintRect {
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
        clip.state_mut().pixel_moving_filter = Some(root.effect.clone());

        crate::property_cell::with_access(|a| {
        assert_eq!(clip.PaintClipRect(a), clip.state(a).rect);
        assert_eq!(clip.PreciseLayoutClipRect(a).rect, Some(precise));
        assert_eq!(clip.LayoutClipRect(a).rect, Some(expanded));
        assert_eq!(
            clip.LayoutClipRectExcludingOverlayScrollbars(a).rect,
            Some(precise)
        );
        assert!(clip.IsForCompositeClipPathAnimation(a));
        assert!(clip.ClipPathEquals(a, None));
        assert!(clip
            .PixelMovingFilter(a)
            .is_some_and(|filter| Arc::ptr_eq(filter, &root.effect)));
        });

        let json = clip.ToJSON();
        assert_eq!(json["rect"], "1,2 30x40");
        assert_eq!(json["isForCompositeClipPathAnimation"], true);
        assert_eq!(json["expandedLayoutClipRect"], "0,0 100x120");
        assert_eq!(json["preciseLayoutClipRect"], "3,4 20x25");
        assert!(json["pixelMovingFilter"].is_string());

        let mut resident = clip.clone();
        resident.id = 73;
        resident.lifecycle = PaintPropertyNodeLifecycle::root();
        let mut next = resident.state_mut().clone();
        next.rect = Some(PaintRect {
            x: 5.0,
            y: 6.0,
            width: 7.0,
            height: 8.0,
        });
        next.parent = Some(root.clip.clone());
        assert_eq!(
            crate::property_cell::with_writer(|w| resident.Update(w, next)),
            PaintPropertyChangeType::ChangedOnlyValues
        );
        assert_eq!(resident.id, 73);
        crate::property_cell::with_access(|a| {
            assert_eq!(resident.PaintClipRect(a).expect("updated clip rect").x, 5.0);
            assert!(resident
                .Parent(a)
                .is_some_and(|parent| Arc::ptr_eq(parent, &root.clip)));
        });
    }

    #[test]
    fn effect_node_owns_source_effect_predicates() {
        let root = PropertyTreeState::default();
        let mut effect = (*root.effect).clone();

        assert!(!effect.state_mut().HasRealEffects());
        assert!(effect.state_mut().IsOpacityOnly());
        assert!(!effect.state_mut().HasDirectCompositingReasons());
        assert!(!effect.state_mut().MayHaveOpacity());
        assert!(!effect.state_mut().MayHaveFilter());
        assert!(!effect.state_mut().MayHaveBackdropFilter());
        assert!(!effect.state_mut().MayHaveBackdropEffect());
        assert!(!effect.state_mut().DrawsContent());

        effect.state_mut().opacity = 0.5;
        assert!(effect.state_mut().HasRealEffects());
        assert!(effect.state_mut().IsOpacityOnly());
        assert!(effect.state_mut().MayHaveOpacity());

        effect.state_mut()
            .direct_compositing_reasons
            .Put(foundation::CompositingReason::kWillChangeFilter);
        assert!(effect.state_mut().HasDirectCompositingReasons());
        assert!(effect.state_mut().RequiresCompositingForWillChangeFilter());
        assert!(effect.state_mut().MayHaveFilter());
        assert!(effect.state_mut().NeedsPixelMovingFilterClipExpander());
        assert!(effect.state_mut().DrawsContent());

        effect.state_mut()
            .direct_compositing_reasons
            .Put(foundation::CompositingReason::kWillChangeBackdropFilter);
        assert!(effect.state_mut().RequiresCompositingForWillChangeBackdropFilter());
        assert!(effect.state_mut().MayHaveBackdropFilter());
        assert!(effect.state_mut().MayHaveBackdropEffect());

        effect.state_mut().compositor.view_transition_element_resource_id = Some(([1, 2], 0, false));
        assert!(!effect.state_mut().HasValidViewTransitionElementResourceId());
        effect.state_mut().compositor.view_transition_element_resource_id = Some(([1, 2], 7, false));
        assert!(effect.state_mut().HasValidViewTransitionElementResourceId());
        effect.state_mut().compositor.restriction_target_id = [1, 0];
        assert!(effect.state_mut().HasElementCaptureId());

        effect.state_mut().compositor.canvas_child_state = Some(Arc::new(PaintCanvasChildState {
            id: 0,
            paint_state: CanvasChildPaintState::default(),
            content_state: root.clone(),
        }));
        assert!(!effect.state_mut().HasCanvasChildState());
        assert_eq!(effect.state_mut().CanvasChildId(), 0);
        effect.state_mut().compositor.canvas_child_state = Some(Arc::new(PaintCanvasChildState {
            id: 9,
            paint_state: CanvasChildPaintState {
                reference_box_offset: Offset { x: 3.0, y: 4.0 },
                canvas_content_size: [100.0, 100.0],
                canvas_device_pixel_content_box: [200, 300],
                ..CanvasChildPaintState::default()
            },
            content_state: root.clone(),
        }));
        assert!(effect.state_mut().HasCanvasChildState());
        assert_eq!(effect.state_mut().CanvasChildId(), 9);
        assert_eq!(
            effect.state_mut()
                .canvas_child_paint_state()
                .expect("canvas child paint state")
                .reference_box_offset,
            Offset { x: 3.0, y: 4.0 }
        );
        let canvas_state = effect.state_mut()
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
            effect.state_mut().CanvasChildContentTransform(),
            &root.transform
        ));
        assert!(Arc::ptr_eq(effect.state_mut().CanvasChildContentClip(), &root.clip));
        assert!(Arc::ptr_eq(effect.state_mut().CanvasChildContentEffect(), &root.effect));

        effect.state_mut().blend_mode = PaintBlendMode::kMultiply;
        assert!(!effect.state_mut().IsOpacityOnly());
        assert!(effect.state_mut().MayHaveBackdropEffect());

        effect.state_mut().compositor.is_in_tainted_subtree = true;
        effect.state_mut().compositor.is_in_drawable_canvas_subtree = true;
        effect.state_mut().compositor.needs_effect_for_2d_scale_transform = true;
        assert!(effect.state_mut().IsInTaintedSubtree());
        assert!(effect.state_mut().IsInDrawableCanvasSubtree());
        assert!(effect.state_mut().NeedsEffectFor2DScaleTransform());

        effect.state_mut().compositor.compositor_element_id = foundation::CompositorElementId::new(42);
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
        crate::property_cell::with_access(|a| initial.clear_changed(a, 1));
        let simple = initial.updated(PaintPropertyChangeType::ChangedOnlySimpleValues);
        assert_eq!(
            crate::property_cell::with_access(|a| simple.node_changed(a)),
            PaintPropertyChangeType::ChangedOnlySimpleValues
        );
        crate::property_cell::with_access(|a| simple.CompositorSimpleValuesUpdated(a));
        assert_eq!(
            crate::property_cell::with_access(|a| simple.node_changed(a)),
            PaintPropertyChangeType::ChangedOnlyCompositedValues
        );
        let initial = PaintPropertyNodeLifecycle::default();
        crate::property_cell::with_access(|a| initial.clear_changed(a, 1));
        let values = initial.updated(PaintPropertyChangeType::ChangedOnlyValues);
        crate::property_cell::with_access(|a| values.CompositorSimpleValuesUpdated(a));
        assert_eq!(
            crate::property_cell::with_access(|a| values.node_changed(a)),
            PaintPropertyChangeType::ChangedOnlyValues
        );

        let root = PropertyTreeState::default();
        let mut opacity_a = (*root.effect).clone();
        opacity_a.state_mut().opacity = 0.4;
        let mut opacity_b = opacity_a.clone();
        opacity_b.state_mut().opacity = 0.5;
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
        filter_a.state_mut().filters = blur(1.0, false);
        filter_a = filter_a.with_filter_presence();
        let mut filter_b = filter_a.clone();
        filter_b.state_mut().filters = blur(2.0, false);
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

        filter_b.state_mut().filters = blur(1.0, true);
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
        backdrop_a.state_mut().backdrop_filters = blur(1.0, false);
        backdrop_a = backdrop_a.with_filter_presence();
        let mut backdrop_b = backdrop_a.clone();
        backdrop_b.state_mut().backdrop_filters = blur(2.0, false);
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
        direct.state_mut().opacity = 0.4;
        assert_eq!(
            direct.state_mut().ComputeOpacityChange(0.5, EffectAnimationState::default()),
            PaintPropertyChangeType::ChangedOnlySimpleValues
        );
        assert_eq!(
            crate::property_cell::with_writer(|w| direct.DirectlyUpdateOpacity(
                w,
                0.5,
                EffectAnimationState {
                    is_running_opacity_animation_on_compositor: true,
                    ..EffectAnimationState::default()
                }
            )),
            PaintPropertyChangeType::ChangedOnlyCompositedValues
        );
        assert_eq!(direct.state_mut().opacity, 0.5);
        assert_eq!(
            crate::property_cell::with_access(|a| direct.lifecycle.node_changed(a)),
            PaintPropertyChangeType::ChangedOnlyCompositedValues
        );
        assert_eq!(
            crate::property_cell::with_writer(|w| direct.DirectlyUpdateOpacity(
                w,
                0.5,
                EffectAnimationState::default()
            )),
            PaintPropertyChangeType::Unchanged
        );

        let mut resident = (*root.effect).clone();
        resident.id = 77;
        resident.state_mut().parent = Some(root.effect.clone());
        resident.lifecycle = PaintPropertyNodeLifecycle::root();
        let mut next_state = resident.state_mut().clone();
        next_state.opacity = 0.25;
        next_state.parent = Some(root.effect.clone());
        assert_eq!(
            crate::property_cell::with_writer(|w| resident.Update(
                w,
                next_state,
                EffectAnimationState::default()
            )),
            PaintPropertyChangeType::ChangedOnlyValues
        );
        assert_eq!(resident.state_mut().opacity, 0.25);
        assert_eq!(resident.id, 77);
        assert!(resident.state_mut()
            .parent
            .as_ref()
            .is_some_and(|parent| Arc::ptr_eq(parent, &root.effect)));
    }

    #[test]
    fn slot_update_keeps_the_resident_node_and_marks_its_change() {
        let root = PropertyTreeState::default().transform;
        let key = PaintPropertyKey {
            owner: PaintPropertyOwner::Fragment {
                client: 9,
                fragment: 0,
            },
            role: PaintPropertyRole::ScrollTranslation,
        };
        let mut store = PaintPropertyNodeStore::default();
        let node = |store: &mut PaintPropertyNodeStore, y: f64| {
            let mut state = crate::property_cell::with_access(|a| root.state(a).clone());
            state.parent = Some(root.clone());
            state.matrix.values[13] = y;
            TransformPaintPropertyNode::new(Default::default(), store.SlotId(Some(key)), state)
        };
        let first = node(&mut store, 0.0);
        let first =
            crate::property_cell::with_writer(|w| store.UpdateTransform(w, Some(key), first).0);
        let next = node(&mut store, -4000.0);
        let rebuilt =
            crate::property_cell::with_writer(|w| store.UpdateTransform(w, Some(key), next).0);
        assert!(Arc::ptr_eq(&first, &rebuilt));
        crate::property_cell::with_access(|a| {
            assert_eq!(first.Matrix(a).values[13], -4000.0);
            assert_eq!(
                first.lifecycle.node_changed(a),
                PaintPropertyChangeType::NodeAddedOrRemoved
            );
        });
        store.RetainLiveSlots(|_| false);
        assert!(store.TransformForKey(key).is_none());
        assert_eq!(store.changed, PaintPropertyChangeType::NodeAddedOrRemoved);
    }

    #[test]
    fn retained_scroll_updates_the_resident_scroll_translation_in_place() {
        let root = PropertyTreeState::default();
        let owner = PaintPropertyOwner::Fragment {
            client: 42,
            fragment: 0,
        };
        let key = |role| PaintPropertyKey { owner, role };
        let mut store = PaintPropertyNodeStore::default();
        let scroll = ScrollPaintPropertyNode::new(
            Default::default(),
            store.SlotId(Some(key(PaintPropertyRole::Scroll))),
            ScrollPaintPropertyNodeState {
                compositor: Default::default(),
                owner_client_id: 42,
                owner_node_id: 0,
                parent: crate::property_cell::with_access(|a| root.transform.state(a).scroll.clone()),
                overflow_clip: Some(root.clip.clone()),
                container_rect: PaintRect::default(),
                contents_rect: PaintRect::default(),
                minimum_scroll_offset: Offset::default(),
                maximum_scroll_offset: Offset { x: 0.0, y: 100.0 },
                user_scrollable_horizontal: false,
                user_scrollable_vertical: true,
            },
        );
        let (scroll, translation) = crate::property_cell::with_writer(|w| {
            let scroll = store.UpdateScroll(w, Some(key(PaintPropertyRole::Scroll)), scroll).0;
            let mut state = root.transform.state(w).clone();
            state.parent = Some(root.transform.clone());
            state.scroll = Some(scroll.clone());
            let translation = TransformPaintPropertyNode::new(
                Default::default(),
                store.SlotId(Some(key(PaintPropertyRole::ScrollTranslation))),
                state,
            );
            let translation = store
                .UpdateTransform(w, Some(key(PaintPropertyRole::ScrollTranslation)), translation)
                .0;
            (scroll, translation)
        });
        crate::property_cell::with_access(|a| translation.ClearChangedToRoot(a, 1));
        let offsets = BTreeMap::from([(owner, Offset { x: 0.0, y: 30.0 })]);
        let updates = store
            .retained_scroll_update(&offsets, &BTreeMap::new())
            .expect("a real scroll translation admits the retained update");
        crate::property_cell::with_writer(|w| store.ApplyRetainedScrollUpdate(w, &updates));
        assert!(Arc::ptr_eq(
            store
                .TransformOwnerForKey(key(PaintPropertyRole::ScrollTranslation))
                .unwrap(),
            &translation
        ));
        crate::property_cell::with_access(|a| {
            assert_eq!(translation.Matrix(a).values[13], -30.0);
            assert_eq!(
                translation.lifecycle.node_changed(a),
                PaintPropertyChangeType::ChangedOnlySimpleValues
            );
            assert_eq!(scroll.lifecycle.node_changed(a), PaintPropertyChangeType::Unchanged);
        });
        assert_eq!(store.scroll_offsets()[&owner], Offset { x: 0.0, y: 30.0 });
    }
}
