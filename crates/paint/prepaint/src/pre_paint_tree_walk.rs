#![allow(non_snake_case)]

use layout::fragment::ResidentBoxGeometry as _;
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::sync::Arc;

/// Flat transform list shared by a node, its contexts and every descendant
/// that adds no transform of its own; cloning only bumps a reference count.
pub type FlatTransforms = Arc<[TransformMatrix]>;

use foundation::{PaintInvalidationReason, TrustedIdentityHashMap as HashMap};

use layout::fragment::FragmentKind;
use layout::paint::{FragmentPaintData, TextCombinePaintData};
use paint_input_types::PaintResources;
use layout::input::PaintStyle;
use layout::input::{Display, FloatSide, NodeKind, Overflow, Position};
use paint_input_types::layout_input::{
    BorderLineStyle, Edges, Offset, PaintClipPathChild, PaintImage, PaintPathCommand, PaintPathVerb,
    Size, TransformMatrix,
};
use paint_input_types::paint_input::{
    BackgroundBox, ClipPathLength, ClipPathPaint, ClipPathRadiusKind, ClipPathReferenceBox,
    ClipPathShapeRadius, PaintBlendMode, PaintCornerRadii, PaintCornerRadius, PaintFilterOperation,
    PaintFilterType, PaintMaskImageState, PaintMaskLayer, PaintReflectionDirection, PaintTransform,
    SvgFilterPrimitiveKind,
};
use layout::paint::{PaintFragmentCursor, PaintFragmentLease};

use crate::background_geometry::{ResolveBackgroundTile, ResolveBackgroundTileSize};
use crate::block_paint_invalidator::BlockPaintInvalidator;
use crate::box_paint_invalidator::{
    BackgroundInvalidation, BackgroundInvalidationType, BoxGeometryMatchesPrevious,
    BoxPaintInvalidator, ViewBackgroundRects,
};
use crate::compositing_reason_finder::CompositingReasonFinder;
use crate::cull_rect_updater::CullRectUpdater;
use crate::fragment_data::{PrePaintFragmentData, PrePaintWalkRecord};
use crate::object_paint_properties::{
    ObjectPaintPropertySlot, PaintPropertyNodeKind, PaintPropertySlotNodeRef,
};
use crate::nine_piece_geometry::NinePieceImageGeometry;
use crate::object_paint_invalidator::ObjectPaintInvalidator;
use crate::paint_invalidator::{PaintInvalidator, PaintInvalidatorContext};
use crate::table_cell_paint_invalidator::TableCellPaintInvalidator;
use crate::contoured_border_geometry::{
    ConicRoundedRectPath, ExpandCornerRadiiEdges, IncludedBorderEdges,
    OutsetCornerRadiiWithCorrection, ResolveCornerRadii, UniformCornerRadius,
};
use paint_model::geometry_mapper::{
    MapRectToRoot, MultiplyTransforms, ResolveLocalTransformAroundReferenceBox,
    ResolvePaintTransform, ResolveTransformReferenceRect, TranslationTransform,
};
use crate::paint_order::ClassifyFragmentPaintLayer;
use paint_model::paint_property_tree::{
    ClipPaintPropertyNode, EffectAnimationState, EffectPaintPropertyNode, PaintBackfaceVisibility,
    PaintCompositorStickyConstraint, PaintPropertyChangeType, PaintPropertyKey,
    PaintPropertyNodeStore, PaintPropertyOwner, PaintPropertyRole,
    ScrollPaintPropertyNode, TransformAnimationState,
    TransformPaintPropertyNode,
};
use paint_model::{DisplayMaskLayer, ResolveDocumentImageContainerKey};

mod paint_property_tree_builder;

use paint_property_tree_builder::PaintPropertyTreeBuilder;

/// Read-only source for one PrePaint pass. Topology always comes from source
/// PhysicalFragment/FragmentItem links.
///
/// The second field is the fragment's FragmentData paint sidecar once this
/// visit resolved it. Chromium's PrePaintTreeWalk reaches each FragmentData
/// once per visit through FragmentDataIterator (core/paint/
/// pre_paint_tree_walk.cc WalkLayoutObject) and then reads it directly; it
/// does not repeat the PhysicalBoxFragment::GetFragmentData lookup for every
/// field it reads.
#[derive(Clone, Copy)]
pub struct PaintFragmentSource<'a>(PaintFragmentCursor<'a>, Option<*const FragmentPaintData>);

/// A LocalFrameView's actual resident LayoutView and its lifetime lease.
/// The embedding supplies the owner fragment, never a copied child tree.
#[derive(Clone)]
pub struct EmbeddedLocalFrameView {
    owner_client: u64,
    root: PaintFragmentCursor<'static>,
    lease: PaintFragmentLease,
    cross_origin_to_parent: bool,
    blocking_wheel_targets: Arc<[u64]>,
}

impl EmbeddedLocalFrameView {
    pub fn new(
        owner: PaintFragmentCursor<'_>,
        layout: &layout::paint::LayoutPaintOutput,
        cross_origin_to_parent: bool,
    ) -> Self {
        assert!(owner
            .ViewReplacedContent()
            .is_some_and(|content| content.is_layout_embedded_content));
        assert_ne!(owner.paint.DisplayItemClientId(), 0);
        // Exactly the same source-owner lease used by ResidentFragment. It
        // outlives every cursor installed in the cross-document PrePaint tree.
        Self {
            owner_client: owner.paint.DisplayItemClientId(),
            root: unsafe {
                std::mem::transmute::<PaintFragmentCursor<'_>, PaintFragmentCursor<'static>>(
                    layout.PaintCursor(),
                )
            },
            lease: layout.PaintLease(),
            cross_origin_to_parent,
            blocking_wheel_targets: Arc::from([]),
        }
    }

    pub fn Root(&self) -> PaintFragmentCursor<'_> {
        self.root
    }
    /// Whether this frame is cross-origin to its parent frame
    /// (FramePainter paints it privacy-preserving).
    /// The DisplayItemClient id of the LayoutEmbeddedContent that embeds it.
    pub fn OwnerClient(&self) -> u64 {
        self.owner_client
    }
    pub fn CrossOriginToParent(&self) -> bool {
        self.cross_origin_to_parent
    }
    pub fn WithBlockingWheelEventHandlerTargets(mut self, targets: Arc<[u64]>) -> Self {
        self.blocking_wheel_targets = targets;
        self
    }
}

impl<'a> PaintFragmentSource<'a> {
    pub fn FromCursor(cursor: PaintFragmentCursor<'a>) -> Self {
        Self(cursor, None)
    }

    /// This source with its FragmentData sidecar resolved once for the
    /// visit (FragmentDataIterator). The sidecar address is stable for the
    /// publication the cursor belongs to.
    pub(crate) fn Resolved(self) -> Self {
        if self.1.is_some() {
            return self;
        }
        // PaintDataMut keeps the sidecar's mutable provenance for
        // ReadPrePaintInvalidationForVisit below.
        let data = unsafe { self.0.PaintDataMut() } as *mut FragmentPaintData;
        Self(self.0, Some(data as *const FragmentPaintData))
    }

    /// Source: pre_paint_tree_walk.cc:418-436. Read the visited object's
    /// dirty bits through the FragmentData this visit already reached
    /// (Resolved), without resolving the sidecar a second time.
    pub(crate) unsafe fn ReadPrePaintInvalidationForVisit(self) -> Self {
        let resolved = self.Resolved();
        let data = resolved.1.expect("resolved above") as *mut FragmentPaintData;
        unsafe { resolved.0.ReadPrePaintInvalidationForResolvedVisit(data) };
        resolved
    }

    pub fn PaintData(self) -> &'a FragmentPaintData {
        match self.1 {
            // The sidecar lives on the owner's FragmentData for the whole
            // publication this cursor's lease keeps alive.
            Some(data) => unsafe { &*data },
            None => self.0.PaintData(),
        }
    }

    pub fn Children(self) -> impl Iterator<Item = Self> + 'a {
        self.0.Children().into_iter().map(|child| Self(child, None))
    }

    pub fn Cursor(self) -> PaintFragmentCursor<'a> {
        self.0
    }
}

impl<'a> std::ops::Deref for PaintFragmentSource<'a> {
    type Target = FragmentPaintData;

    fn deref(&self) -> &Self::Target {
        match self.1 {
            Some(data) => unsafe { &*data },
            None => &self.0,
        }
    }
}
use paint_model::PaintRect;

// Source: paint_property_tree_builder.cc:935-937,1286-1288. IDs come from the
// original FragmentData/box owner, never a dump-local property label.
fn TransformCompositorState(
    fragment: &FragmentPaintData,
    role: PaintPropertyRole,
) -> paint_model::paint_property_tree::TransformCompositorState {
    use foundation::{CompositorElementIdFromUniqueObjectId, CompositorElementIdNamespace as Ns};
    let mut value = paint_model::paint_property_tree::TransformCompositorState::default();
    value.is_for_svg_child = role == PaintPropertyRole::SvgLocalTransform
        && fragment.paint.SourceKind() != NodeKind::kSvgRoot;
    value.is_frame_paint_offset_translation =
        role == PaintPropertyRole::PaintOffsetTranslation && fragment.paint.IsLayoutView();
    let ns = match role {
        PaintPropertyRole::StickyTranslation => Some(Ns::kStickyTranslation),
        PaintPropertyRole::ElementCanvasTransform => Some(Ns::kElementCanvasTransform),
        PaintPropertyRole::Translate => Some(Ns::kTranslateTransform),
        PaintPropertyRole::Rotate => Some(Ns::kRotateTransform),
        PaintPropertyRole::Scale => Some(Ns::kScaleTransform),
        PaintPropertyRole::Transform | PaintPropertyRole::SvgLocalTransform => {
            Some(Ns::kPrimaryTransform)
        }
        _ => None,
    };
    if let Some(ns) = ns {
        if fragment.ViewPropertyFragmentUniqueId() != 0 {
            value.compositor_element_id = CompositorElementIdFromUniqueObjectId(
                fragment.ViewPropertyFragmentUniqueId(),
                ns,
            );
        }
    }
    value
}

// Source: paint_property_tree_builder.cc:889-1045. Blink resolves Layout's
// sticky client identities against the resident Paint property tree before it
// publishes the constraint to cc. Keep that resolution on the Paint side of
// the PaintArtifact boundary.
fn ScrollElementIdForStickyClient(
    parent: &Arc<TransformPaintPropertyNode>,
    owner_client_id: u64,
) -> foundation::CompositorElementId {
    let mut transform = Some(parent.clone());
    while let Some(node) = transform {
        let found = paint_model::property_cell::with_access(|access| {
            let mut scroll = node.ScrollNode(access);
            while let Some(candidate) = scroll {
                let state = candidate.state(access);
                if state.owner_client_id == owner_client_id {
                    return Some(state.compositor.compositor_element_id);
                }
                scroll = state.parent.as_ref();
            }
            None
        });
        if let Some(element_id) = found {
            return element_id;
        }
        transform = paint_model::property_cell::with_access(|access| node.Parent(access).cloned());
    }
    Default::default()
}

fn StickyElementIdForClient(
    parent: &Arc<TransformPaintPropertyNode>,
    owner_client_id: u64,
) -> foundation::CompositorElementId {
    paint_model::property_cell::with_access(|access| {
        let mut transform = Some(parent);
        while let Some(node) = transform {
            let state = node.state(access);
            if state
                .sticky_constraint
                .is_some_and(|constraint| constraint.owner_client_id == owner_client_id)
            {
                return state.compositor.compositor_element_id;
            }
            transform = state.parent.as_ref();
        }
        Default::default()
    })
}

fn BuildCompositorStickyConstraint(
    sticky: layout::paint::StickyPositionPaintData,
    parent: &Arc<TransformPaintPropertyNode>,
    pixel_snap_offset: Offset,
) -> Option<PaintCompositorStickyConstraint> {
    let mut result = PaintCompositorStickyConstraint::default();
    if let Some(axis) = sticky.horizontal {
        result.anchored[0] = axis.min_inset.is_some();
        result.anchored[1] = axis.max_inset.is_some();
        result.offsets[0] = axis.min_inset.unwrap_or_default() as f32;
        result.offsets[1] = axis.max_inset.unwrap_or_default() as f32;
        result.constraint_box_rect.x = axis.constraining_offset;
        result.constraint_box_rect.width = axis.constraining_size;
        result.sticky_box_rect.x = axis.scroll_container_relative_sticky_box_offset;
        result.sticky_box_rect.width = axis.scroll_container_relative_sticky_box_size;
        result.containing_block_rect.x = axis.scroll_container_relative_containing_block_offset;
        result.containing_block_rect.width = axis.scroll_container_relative_containing_block_size;
        result.x_scroll_ancestor =
            ScrollElementIdForStickyClient(parent, axis.scroll_container_client_id);
        result.nearest_shifting_sticky_box =
            StickyElementIdForClient(parent, axis.nearest_sticky_box_client_id);
        result.nearest_shifting_containing_block =
            StickyElementIdForClient(parent, axis.nearest_containing_block_client_id);
    }
    if let Some(axis) = sticky.vertical {
        result.anchored[2] = axis.min_inset.is_some();
        result.anchored[3] = axis.max_inset.is_some();
        result.offsets[2] = axis.min_inset.unwrap_or_default() as f32;
        result.offsets[3] = axis.max_inset.unwrap_or_default() as f32;
        result.constraint_box_rect.y = axis.constraining_offset;
        result.constraint_box_rect.height = axis.constraining_size;
        result.sticky_box_rect.y = axis.scroll_container_relative_sticky_box_offset;
        result.sticky_box_rect.height = axis.scroll_container_relative_sticky_box_size;
        result.containing_block_rect.y = axis.scroll_container_relative_containing_block_offset;
        result.containing_block_rect.height = axis.scroll_container_relative_containing_block_size;
        result.y_scroll_ancestor =
            ScrollElementIdForStickyClient(parent, axis.scroll_container_client_id);
        if axis.nearest_sticky_box_client_id != 0 {
            result.nearest_shifting_sticky_box =
                StickyElementIdForClient(parent, axis.nearest_sticky_box_client_id);
        }
        if axis.nearest_containing_block_client_id != 0 {
            result.nearest_shifting_containing_block =
                StickyElementIdForClient(parent, axis.nearest_containing_block_client_id);
        }
    }
    result.pixel_snap_offset = [pixel_snap_offset.x as f32, pixel_snap_offset.y as f32];
    // Source: paint_property_tree_builder.cc:991-1007. Match gfx's rounding
    // workaround for negative half-integral LayoutUnit values.
    if pixel_snap_offset.x == -0.5 {
        result.pixel_snap_offset[0] += 0.001;
    }
    if pixel_snap_offset.y == -0.5 {
        result.pixel_snap_offset[1] += 0.001;
    }
    (foundation::CompositorElementId::IsValidInternalValue(
        result.x_scroll_ancestor.GetInternalValue(),
    ) || foundation::CompositorElementId::IsValidInternalValue(
        result.y_scroll_ancestor.GetInternalValue(),
    ))
    .then_some(result)
}

// Source: paint_property_tree_builder.cc:2246-2261,2301,2687-2688.
fn EffectCompositorState(
    fragment: &FragmentPaintData,
    role: PaintPropertyRole,
    has_direct_reasons: bool,
    backdrop_mask: bool,
    is_in_tainted_subtree: bool,
    is_in_drawable_canvas_subtree: bool,
) -> paint_model::paint_property_tree::EffectCompositorState {
    use foundation::{CompositorElementIdFromUniqueObjectId, CompositorElementIdNamespace as Ns};
    let mut value = paint_model::paint_property_tree::EffectCompositorState::default();
    value.is_in_tainted_subtree = is_in_tainted_subtree;
    value.is_in_drawable_canvas_subtree = is_in_drawable_canvas_subtree;
    if fragment.ViewPropertyFragmentUniqueId() != 0 {
        let namespace = match role {
            PaintPropertyRole::Filter => Ns::kEffectFilter,
            PaintPropertyRole::Mask => Ns::kEffectMask,
            _ => {
                if has_direct_reasons {
                    Ns::kPrimaryEffect
                } else {
                    Ns::kPrimary
                }
            }
        };
        value.compositor_element_id = CompositorElementIdFromUniqueObjectId(
            fragment.ViewPropertyFragmentUniqueId(),
            namespace,
        );
        if backdrop_mask {
            value.backdrop_mask_element_id = CompositorElementIdFromUniqueObjectId(
                fragment.ViewPropertyFragmentUniqueId(),
                Ns::kEffectMask,
            );
        }
    }
    value
}
use crate::paint_shader_resolver::ResolvePaintShader;

#[derive(Clone, Default)]
pub struct PropertyTreeState {
    pub transform_id: u64,
    pub clip_id: u64,
    pub effect_id: u64,
    pub scroll_id: u64,
    pub nodes: paint_model::paint_property_tree::PropertyTreeState,
}

/// Direct reference to Chromium's canonical FragmentData sidecar. The source
/// lease retained by PrePaint keeps this address alive across incremental
/// frames; no FragmentPaintData fields are copied into Paint.
#[doc(hidden)]
pub enum ResidentFragment {
    Borrowed(*const FragmentPaintData),
    /// Hand-built PaintTreeNode values are used only by focused package tests.
    /// Keeping their fixture data here prevents a test helper from returning a
    /// dangling reference; production construction always uses `Borrowed`.
    Fixture(Box<FragmentPaintData>),
}

impl ResidentFragment {
    fn FromSource(source: &FragmentPaintData) -> Self {
        Self::Borrowed(source as *const FragmentPaintData)
    }
}

impl std::ops::Deref for ResidentFragment {
    type Target = FragmentPaintData;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(pointer) => {
                // The pointer names the stable sidecar on its owner's
                // FragmentData. A PaintFragmentLease is retained for every tree
                // that can reach it.
                unsafe { &**pointer }
            }
            Self::Fixture(fragment) => fragment,
        }
    }
}

impl std::ops::DerefMut for ResidentFragment {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Borrowed(pointer) => {
                // FragmentPaintData lives in the owner FragmentData's
                // sidecar UnsafeCell; PrePaint is the single
                // lifecycle writer while this lease is active.
                unsafe { &mut *(*pointer as *mut FragmentPaintData) }
            }
            Self::Fixture(fragment) => fragment,
        }
    }
}

pub struct PaintTreeNode<'a> {
    // This cursor borrows the exact source retained by PaintFragmentLease.
    source_cursor: Option<PaintFragmentCursor<'static>>,
    // Same boxed resource Paint nodes, indexed during the existing effects walk.
    svg_resource_nodes: Option<Box<HashMap<u64, *const PaintTreeNode<'static>>>>,

    pub fragment: Option<ResidentFragment>,
    pub source_lifetime: std::marker::PhantomData<&'a FragmentPaintData>,
    physical_index: usize,
    /// The ScopedDisplayItemFragment this node paints in: traversal state the
    /// PrePaint walk derives from the paint parent (see DisplayItemFragment).
    display_item_fragment: Option<u32>,
    /// The fragmentainer (column / page area) this node is inside: traversal
    /// state the walk derives from the paint parent, as PrePaintTreeWalk
    /// carries `fragmentainer_idx` (see FragmentainerInstanceId).
    fragmentainer_instance_id: Option<u64>,
    cache: Option<NodeCache>,
    pub absolute_offset: Offset,
    pub paint_offset: Offset,
    /// Source: PaintPropertyTreeBuilderFragmentContext::current.
    /// Propagated to PaintInvalidator for the composited subpixel movement
    /// optimization; zero means the optimization is not applicable.
    pub directly_composited_container_paint_offset_subpixel_delta: Offset,
    // Pixel snapping belongs to the local property space. This delta only
    // bridges a real paint-space reset; scrolling stays in ScrollTranslation.
    pub paint_snap_offset: Offset,
    // Blink snaps paint geometry to physical pixels. LayoutNG coordinates in
    // this crate remain CSS pixels, so retain the root device scale while
    // applying the same snap instead of rounding to whole CSS pixels.
    pub pixel_snap_scale: f64,
    pub root_clip: Option<PaintRect>,
    /// Chromium's InnerBorderRadiusClip precedes OverflowClip and remains a
    /// distinct persistent property node.
    pub local_inner_border_radius_clip: Option<PaintRect>,
    pub local_inner_border_radius_radii: PaintCornerRadii,
    pub local_clip: Option<PaintRect>,
    pub local_css_clip: Option<PaintRect>,
    pub local_clip_radius: f64,
    pub local_clip_radii: PaintCornerRadii,
    pub local_clip_path: Vec<PaintPathCommand>,
    pub local_clip_path_children: Vec<PaintClipPathChild>,
    pub clip_path_even_odd: bool,
    pub clip_path_error: Option<String>,
    pub clip: Option<PaintRect>,
    pub cull_rect: Option<PaintRect>,
    pub self_cull_rect: Option<PaintRect>,
    pub(crate) cull_decision_inputs: crate::cull_rect_updater::CullRectDecisionInputs,
    pub transforms: FlatTransforms,
    pub local_transforms: Vec<TransformMatrix>,
    /// ObjectPaintProperties::PaintOffsetTranslation. Keeping the source slot
    /// separate lets ScopedPaintState emit this one node as a raw drawing
    /// translation when it is entered inside a DrawingRecorder.
    pub paint_offset_translation: Option<Arc<TransformPaintPropertyNode>>,
    pub opacity: f32,
    pub scroll_offset: Offset,
    /// CSS transforms on non-root ancestors make background-attachment:fixed
    /// behave as scroll. Root transforms are deliberately excluded: Chromium
    /// keeps the viewport attachment and applies the root transform to it.
    pub fixed_background_acts_as_scroll: bool,
    pub is_paint_layer: bool,
    /// True when the PaintLayer admission facts below came from the source
    /// Layout/PaintLayer owner. Standalone test fragments have no such owner
    /// and retain their existing fragment-driven traversal.
    pub paint_layer_facts_are_authoritative: bool,
    pub is_self_painting_layer: bool,
    pub invisible_for_position_visibility: bool,
    pub has_ancestor_invisible_for_position_visibility: bool,
    pub has_visible_content: bool,
    pub has_self_painting_layer_descendant: bool,
    pub is_under_svg_hidden_container: bool,
    pub supports_subsequence_caching: bool,
    pub self_or_descendant_needs_repaint: bool,
    pub paint_layer_is_fragmented: bool,
    pub known_to_clip_subtree_to_padding_box: bool,
    pub child_paint_blocked_by_display_lock: bool,
    pub is_stacking_context: bool,
    pub is_stacked: bool,
    pub is_root: bool,
    /// LocalFrameView owns a separate PaintLayer tree, painted by FramePainter.
    pub is_embedded_local_frame_root: bool,
    pub embedded_frame_is_cross_origin: bool,
    pub escapes_ancestor_geometry: bool,
    pub physical_ancestors: Vec<*const PaintTreeNode<'a>>,
    pub stacking_level: i32,
    pub is_z_order_list_visible: bool,
    pub is_in_top_or_view_transition_layer: bool,
    pub is_direct_layout_view_child: bool,
    pub can_contain_absolute_position_objects: bool,
    pub can_contain_fixed_position_objects: bool,
    pub has_overlay_overflow_controls: bool,
    /// PaintLayer::NeedsReorderOverlayOverflowControls(). The stacking-node
    /// rebuild mutates this paint-lifecycle bit before any foreground pass.
    pub needs_reorder_overlay_overflow_controls: Cell<bool>,
    /// PaintLayer::PreviousPaintResult for the current traversal. Persistent
    /// cache reuse is carried by SubsequenceMarkers at the recording boundary.
    pub previous_paint_result: Cell<paint_model::PaintResult>,
    /// PaintLayerStackingNode's canonical derived lists. PaintWalk publishes
    /// the exact lists it draws; hit testing only borrows them.
    pub(crate) hit_test_layer_order:
        RefCell<Option<crate::resident_hit_test::ResidentPaintLayerOrder>>,
    pub paint_layer_order_parent_unique_id: Option<u64>,
    pub paint_layer_order: i32,
    pub applies_transform: bool,
    pub applies_clip: bool,
    pub applies_root_clip: bool,
    pub applies_overflow_clip: bool,
    pub applies_css_clip: bool,
    pub applies_clip_path: bool,
    pub applies_effect: bool,
    pub applies_opacity: bool,
    pub applies_blend: bool,
    pub applies_filter: bool,
    pub applies_backdrop_filter: bool,
    pub applies_mask: bool,
    pub paints_outline: bool,
    pub needs_float_phase: bool,
    pub needs_outline_phase: bool,
    pub needs_overflow_controls_phase: bool,
    // Upward result of TableCellPaintInvalidator. Chromium mutates already
    // visited row/section/table clients through parent contexts; the retained
    // physical fragment tree applies the same result while unwinding.
    table_cell_geometry_changed: bool,
    pub outline_rects: Vec<PaintRect>,
    pub mask_layers: Vec<DisplayMaskLayer>,
    /// The independent SaveLayerFilter drawing-record payload. Effect property
    /// nodes own only their canonical compositor operations, never this list.
    pub recording_filters: Vec<PaintFilterOperation>,
    pub properties: PropertyTreeState,
    /// ObjectPaintProperties::LocalBorderBoxProperties().PreTransform().
    /// Canvas background painting keeps the document element's clip/effect
    /// state while replacing its CSS transform with this parent space.
    pub pre_transform: Arc<TransformPaintPropertyNode>,
    pub contents_properties: PropertyTreeState,
    /// The composited fixed background paints in the LayoutView transform,
    /// while its BackgroundClip remains in this box's local transform space.
    pub fixed_background_properties: Option<paint_model::paint_property_tree::PropertyTreeState>,
    /// ScrollableAreaPainter's optional border-box clip for controls that
    /// extend outside the pixel-snapped border box.
    pub overflow_controls_properties: Option<paint_model::paint_property_tree::PropertyTreeState>,
    /// Overlay scrollbars animate opacity independently from page contents.
    /// These are ObjectPaintProperties' horizontal/vertical scrollbar effects.
    pub horizontal_scrollbar_properties:
        Option<paint_model::paint_property_tree::PropertyTreeState>,
    pub vertical_scrollbar_properties: Option<paint_model::paint_property_tree::PropertyTreeState>,
    /// Mask source uses a separate DstIn effect and the pre-MaskClip output
    /// clip, matching PaintLayerPainter::PaintFragmentWithPhase(kMask).
    pub mask_properties: Option<paint_model::paint_property_tree::PropertyTreeState>,
    pub children: Vec<Box<PaintTreeNode<'a>>>,
}

/// Source: `cull_rect_updater.cc:44-48,51-64`.
///
/// A changed cull rect exposes potentially unrecorded pixels only when the
/// previous traversal reported that it may have clipped this layer.
fn CullRectChangeNeedsRepaint(
    previous_result: paint_model::PaintResult,
    old_self: Option<Option<PaintRect>>,
    old_contents: Option<Option<PaintRect>>,
    new_self: Option<PaintRect>,
    new_contents: Option<PaintRect>,
) -> bool {
    previous_result == paint_model::PaintResult::kMayBeClippedByCullRect
        && (old_self.is_some_and(|old| old != new_self)
            || old_contents.is_some_and(|old| old != new_contents))
}

// Cull-aware painters may omit individual border sides, shadows, images, or
// descendants from a DrawingDisplayItem while the layer reports
// kMayBeClippedByCullRect.  When the sliding cull window moves, rejecting only
// the cached PaintLayer subsequence is insufficient: the freshly walked layer
// can otherwise move one of those incomplete drawing records into the new
// artifact.  Chromium's CullRectUpdater marks the PaintLayer for repaint and
// invalidates its display-item client.  Rechrom has separate projected client
// records, so raise each client owned by this fragment at the same boundary.
fn InvalidateClientsForCullRectChange(fragment: &mut FragmentPaintData) {
    use crate::object_paint_invalidator::ObjectPaintInvalidator;
    let reason = PaintInvalidationReason::kLayout;
    // CullRectUpdater: layer.SetNeedsRepaint() on a repainting cull change.
    fragment.paint.SetPaintingLayerNeedsRepaint();
    ObjectPaintInvalidator::InvalidateDisplayItemClient(fragment, reason);
    ObjectPaintInvalidator::InvalidatePaintLayerClient(fragment, reason);
    if fragment.paint.SelectionClientId() != 0 {
        ObjectPaintInvalidator::InvalidateSelectionClient(fragment, reason);
    }
}

impl<'a> Default for PaintTreeNode<'a> {
    fn default() -> Self {
        Self::WithProperties(PropertyTreeState::default())
    }
}

impl<'a> PaintTreeNode<'a> {
    /// PaintLayer::SelfOrDescendantNeedsRepaint() as PaintLayerPainter reads
    /// it at paint time: the live bits on the source PaintLayer, which
    /// SetNeedsRepaint / MarkPaintingContainerChainForNeedsRepaint set when a
    /// client was invalidated and the committed Paint clears. A fragment
    /// without a source layer always repaints; a hand-built node without a
    /// fragment keeps its own bit.
    // cpp: third_party/blink/renderer/core/paint/paint_layer.h:451-457
    // cpp: third_party/blink/renderer/core/paint/paint_layer_painter.cc:412-424
    pub fn SelfOrDescendantNeedsRepaint(&self) -> bool {
        match self.fragment.as_deref() {
            Some(fragment) => fragment.paint.PaintLayerSelfOrDescendantNeedsRepaint(),
            None => self.self_or_descendant_needs_repaint,
        }
    }

    /// A node whose property slots start at `properties`. PrePaint seeds a
    /// built node with its parent's state instead of referencing the shared
    /// root sentinels and replacing them moments later, which cost contended
    /// reference-count traffic on those process-wide nodes for every object.
    pub(crate) fn WithProperties(properties: PropertyTreeState) -> Self {
        let pre_transform = properties.nodes.transform.clone();
        let contents_properties = properties.clone();
        Self {
            source_cursor: None,
            svg_resource_nodes: None,

            fragment: None,
            source_lifetime: std::marker::PhantomData,
            physical_index: 0,
            display_item_fragment: None,
            fragmentainer_instance_id: None,
            cache: None,
            absolute_offset: Offset::default(),
            paint_offset: Offset::default(),
            directly_composited_container_paint_offset_subpixel_delta: Offset::default(),
            paint_snap_offset: Offset::default(),
            pixel_snap_scale: 1.0,
            root_clip: None,
            local_inner_border_radius_clip: None,
            local_inner_border_radius_radii: PaintCornerRadii::default(),
            local_clip: None,
            local_css_clip: None,
            local_clip_radius: 0.0,
            local_clip_radii: PaintCornerRadii::default(),
            local_clip_path: Vec::new(),
            local_clip_path_children: Vec::new(),
            clip_path_even_odd: false,
            clip_path_error: None,
            clip: None,
            cull_rect: None,
            self_cull_rect: None,
            cull_decision_inputs: Default::default(),
            transforms: FlatTransforms::default(),
            local_transforms: Vec::new(),
            paint_offset_translation: None,
            opacity: 1.0,
            scroll_offset: Offset::default(),
            fixed_background_acts_as_scroll: false,
            is_paint_layer: false,
            paint_layer_facts_are_authoritative: false,
            is_self_painting_layer: false,
            invisible_for_position_visibility: false,
            has_ancestor_invisible_for_position_visibility: false,
            has_visible_content: false,
            has_self_painting_layer_descendant: false,
            is_under_svg_hidden_container: false,
            supports_subsequence_caching: false,
            self_or_descendant_needs_repaint: true,
            paint_layer_is_fragmented: false,
            known_to_clip_subtree_to_padding_box: false,
            child_paint_blocked_by_display_lock: false,
            is_stacking_context: false,
            is_stacked: false,
            is_root: false,
            is_embedded_local_frame_root: false,
            embedded_frame_is_cross_origin: false,
            escapes_ancestor_geometry: false,
            physical_ancestors: Vec::new(),
            stacking_level: 0,
            is_z_order_list_visible: false,
            is_in_top_or_view_transition_layer: false,
            is_direct_layout_view_child: false,
            can_contain_absolute_position_objects: false,
            can_contain_fixed_position_objects: false,
            has_overlay_overflow_controls: false,
            needs_reorder_overlay_overflow_controls: Cell::new(false),
            // Source: paint_layer.cc:187. A new layer is conservative until
            // its first complete PaintLayerPainter traversal finishes.
            previous_paint_result: Cell::new(paint_model::PaintResult::kMayBeClippedByCullRect),
            hit_test_layer_order: RefCell::new(None),
            paint_layer_order_parent_unique_id: None,
            paint_layer_order: 0,
            applies_transform: false,
            applies_clip: false,
            applies_root_clip: false,
            applies_overflow_clip: false,
            applies_css_clip: false,
            applies_clip_path: false,
            applies_effect: false,
            applies_opacity: false,
            applies_blend: false,
            applies_filter: false,
            applies_backdrop_filter: false,
            applies_mask: false,
            paints_outline: true,
            // Hand-built trees remain conservative; the completed PrePaint
            // postorder replaces these hints with actual subtree requirements.
            needs_float_phase: true,
            needs_outline_phase: true,
            needs_overflow_controls_phase: true,
            table_cell_geometry_changed: false,
            outline_rects: Vec::new(),
            mask_layers: Vec::new(),
            recording_filters: Vec::new(),
            properties,
            pre_transform,
            contents_properties,
            fixed_background_properties: None,
            overflow_controls_properties: None,
            horizontal_scrollbar_properties: None,
            vertical_scrollbar_properties: None,
            mask_properties: None,
            children: Vec::new(),
        }
    }
}

impl PaintTreeNode<'_> {
    pub fn SourceCursor(&self) -> Option<PaintFragmentCursor<'_>> {
        self.source_cursor
    }

    /// The paint side of Chromium's FragmentData for this node's occurrence
    /// (core/paint/fragment_data.h), which PrePaint keeps current for every
    /// visited and retained node. Painters read paint offset, paint
    /// properties, local border box / contents properties and cull rects
    /// from here instead of the node's copies. None for fixture nodes
    /// without a result state.
    pub fn FragmentData(&self) -> Option<&PrePaintFragmentData> {
        self.source_cursor?.PrePaintData::<PrePaintFragmentData>()
    }

    /// The active ScopedDisplayItemFragment of this node's display items.
    /// Chromium keeps it as paint traversal state: a fragmentainer opens its
    /// break token's SequenceNumber()+1 (box_fragment_painter.cc:394-403), a
    /// layer box its index in LayoutBox::PhysicalFragments()
    /// (paint_layer_painter.cc:776-778), an inline item its FragmentId()
    /// (inline_box_fragment_painter.cc:63-64), and everything else inherits
    /// its paint parent's scope. The PrePaint walk records it per node;
    /// hand-built nodes outside a walk read the fragment's value.
    pub fn DisplayItemFragment(&self) -> u32 {
        match self.display_item_fragment {
            Some(fragment) => fragment,
            None => self
                .fragment
                .as_deref()
                .map_or(0, |fragment| fragment.ViewDisplayItemFragment()),
        }
    }

    /// The fragmentainer identity this node is inside (0 outside any): a
    /// column or page area is its own (fragment_tree::FragmentainerInstanceId),
    /// everything else inherits its paint parent's. The PrePaint walk records
    /// it per node; hand-built nodes outside a walk read the fragment's own.
    pub fn FragmentainerInstanceId(&self) -> u64 {
        match self.fragmentainer_instance_id {
            Some(fragmentainer) => fragmentainer,
            None => self
                .fragment
                .as_deref()
                .and_then(|fragment| fragment.ViewOwnFragmentainerInstanceId())
                .unwrap_or(0),
        }
    }

    /// The paint resources of the document this node's fragment belongs to
    /// (Chromium reads them from the LayoutObject's Document / LocalFrameView).
    /// Production nodes resolve them through their source result; hand-built
    /// fixture nodes without a source keep their fixture value.
    pub fn DocumentResources(
        &self,
    ) -> Option<&Arc<paint_input_types::PaintResources>> {
        match self.source_cursor {
            Some(cursor) => cursor.DocumentResources(),
            None => self.fragment.as_deref()?.resources.as_ref(),
        }
    }
    pub fn SvgResourceNode(&self, client: u64) -> Option<&PaintTreeNode<'static>> {
        let pointer = *self.svg_resource_nodes.as_ref()?.get(&client)?;
        // Directory rebuilt whenever topology changes; root owns every box.
        unsafe { pointer.as_ref() }
    }
    pub fn SetFragment(&mut self, source: &FragmentPaintData) {
        self.fragment = Some(ResidentFragment::FromSource(source));
    }

    /// Rebind a retained node to its replacement source: the sidecar slot and
    /// the cursor through which it reads its document's paint resources.
    pub(crate) fn BindSource(&mut self, source: PaintFragmentSource<'_>) {
        self.fragment = Some(ResidentFragment::FromSource(&source));
        // Same opaque retained lifetime as ResidentFragment; the caller keeps
        // this result's PaintFragmentLease until replacement has been rebound.
        self.source_cursor = Some(unsafe {
            std::mem::transmute::<PaintFragmentCursor<'_>, PaintFragmentCursor<'static>>(
                source.Cursor(),
            )
        });
    }

    #[doc(hidden)]
    pub fn SetOwnedFragmentForTesting(&mut self, source: &FragmentPaintData) {
        self.fragment = Some(ResidentFragment::Fixture(Box::new(source.clone())));
    }

    /// Exact resolved flat-space source of ObjectPaintProperties::MaskClip.
    /// The property node adds paint_snap_offset; flat replay uses this same
    /// source rectangle before that projection, avoiding round-trip drift.
    pub fn ResolvedMaskClipRect(&self) -> Option<PaintRect> {
        ResolveFlatMaskClipRect(self.fragment.as_deref()?, self.paint_offset, &self.mask_layers)
    }
}

/// The flat-space source rect of ObjectPaintProperties::MaskClip for an
/// occurrence at the flat `paint_offset` with its resolved `mask_layers`
/// (PaintTreeNode::ResolvedMaskClipRect).
pub fn ResolveFlatMaskClipRect(
    fragment: &FragmentPaintData,
    paint_offset: Offset,
    mask_layers: &[DisplayMaskLayer],
) -> Option<PaintRect> {
    let is_svg_effect_child = IsSvgEffectChild(fragment.paint.SourceKind());
    if !is_svg_effect_child {
        let bounds = fragment.paint.CssMaskBoundingBox()?;
        return Some(PaintRect {
            x: paint_offset.x + bounds.offset.x,
            y: paint_offset.y + bounds.offset.y,
            width: bounds.size.width,
            height: bounds.size.height,
        });
    }
    let mut rect = mask_layers.first().map(|layer| layer.clip_rect);
    for layer in mask_layers.iter().skip(1) {
        rect = Some(Union(rect.as_ref().unwrap().to_owned(), &layer.clip_rect));
    }
    // SVGMaskPainter::ResourceBoundsForSVGChild unions every live mask
    // resource with VisualRectInLocalSVGCoordinates. Chromium fragment ink
    // overflow is the boundary projection of that visual rect; keep it in
    // the same paint space as the resolved mask regions.
    if is_svg_effect_child {
        if let Some(visual) = fragment
            .ViewInkOverflow()
            .or(fragment.ViewContentsInkOverflow())
            .or(fragment.ViewSelfInkOverflow())
        {
            let visual = PaintRect {
                x: paint_offset.x + visual.offset.x,
                y: paint_offset.y + visual.offset.y,
                width: visual.size.width,
                height: visual.size.height,
            };
            rect = Some(rect.map_or(visual, |rect| Union(rect, &visual)));
        }
    }
    rect
}

impl PaintTreeNode<'_> {
    /// The paint offset Paint records this occurrence with, in the flat
    /// replay space every paint_record op, DrawingRecorder visual rect, hit
    /// test rect and chunk bound uses (the space of `paint_offset`): the
    /// occurrence's FragmentData::PaintOffset() (fragment_data.h:32-38, LOCAL
    /// to its LocalBorderBoxProperties transform, LayoutUnit-rounded) moved
    /// back by the flat-to-property delta `paint_snap_offset`. The snapping
    /// helpers below project into property space and back with the same
    /// delta. Fixture nodes without FragmentData use their own flat offset.
    ///
    /// Paint stays flat until the flat whole-recording replay
    /// (PaintRecordForArtifact) and the per-item PaintChunksToCcLayer
    /// consumer agree on Chromium's local space; then this becomes
    /// FragmentData::PaintOffset() and the delta disappears.
    pub fn FlatPaintOffset(&self) -> Offset {
        self.FlatPaintOffsetOf(self.FragmentData())
    }

    /// FlatPaintOffset with this occurrence's FragmentData already resolved.
    pub fn FlatPaintOffsetOf(&self, data: Option<&PrePaintFragmentData>) -> Offset {
        let Some(data) = data else {
            return self.paint_offset;
        };
        let local = data.PaintOffset();
        let flat = Offset {
            x: local.left.ToDouble() - self.paint_snap_offset.x,
            y: local.top.ToDouble() - self.paint_snap_offset.y,
        };
        // PaintPropertyTreeBuilder stores round(paint_offset + delta) in
        // LayoutUnit (FragmentDataPaintOffset); only that rounding may
        // separate the two copies.
        debug_assert!(
            (flat.x - self.paint_offset.x).abs() <= 1.0 / 128.0 + 1e-9
                && (flat.y - self.paint_offset.y).abs() <= 1.0 / 128.0 + 1e-9,
            "FragmentData PaintOffset is stale for its paint node"
        );
        flat
    }

    pub fn PixelSnappedRect(&self, rect: &PaintRect) -> PaintRect {
        PixelSnappedRectInPropertySpace(rect, self.paint_snap_offset, self.pixel_snap_scale)
    }

    pub fn EnclosingRect(&self, rect: PaintRect) -> PaintRect {
        // DrawingRecorder bounds are enclosed in property space, where
        // Chromium encloses the local rect, and returned in the flat replay
        // space of the recorded ops (see FlatPaintOffset).
        EnclosingRectInPropertySpace(rect, self.paint_snap_offset)
    }

    pub fn PixelSnappedOffset(&self, offset: Offset) -> Offset {
        Offset {
            x: SnapToPixel(offset.x + self.paint_snap_offset.x, self.pixel_snap_scale)
                - self.paint_snap_offset.x,
            y: SnapToPixel(offset.y + self.paint_snap_offset.y, self.pixel_snap_scale)
                - self.paint_snap_offset.y,
        }
    }

    pub fn RoundedPaintY(&self, y: f64) -> f64 {
        // third_party/blink/renderer/core/paint/text_fragment_painter.cc
        // ComputePhysicalBoxGeometry rounds the CSS LayoutUnit line top before
        // projection into device pixels. LayoutUnit::Round is floor(value +
        // 0.5), including negative half values; DPR is deliberately irrelevant.
        (y + 0.5).floor()
    }
}

use paint_reuse::pre_paint_state_compare as state_compare;

/// The physical parent's actual builder context. A dirty bit permits work;
/// these values decide whether that work must propagate into clean children.
#[derive(Clone)]
struct CanvasContentsContext {
    layout_object: usize,
    properties: PropertyTreeState,
}
#[derive(Clone)]
struct InheritedContext {
    inside_blocking_wheel_event_handler: bool,
    blocking_wheel_event_handler_changed: bool,
    offset: Offset,
    paint_space: Offset,
    clip: Option<PaintRect>,
    cull: Option<PaintRect>,
    transforms: FlatTransforms,
    opacity: f32,
    scroll: Offset,
    fixed_background_acts_as_scroll: bool,
    properties: PropertyTreeState,
    canvas_contents: Option<CanvasContentsContext>,
    rendering_context_id: u64,
    should_flatten_inherited_transform: bool,
    can_inherit_backface_visibility: bool,
    directly_composited_container_paint_offset_subpixel_delta: Offset,
    subtree_should_use_infinite_cull_rect: bool,
    root: bool,
    flex_or_grid_item: bool,
}

#[derive(Clone)]
struct ChildContext {
    inside_blocking_wheel_event_handler: bool,
    blocking_wheel_event_handler_changed: bool,
    origin: Offset,
    paint_space: Offset,
    clip: Option<PaintRect>,
    cull: Option<PaintRect>,
    transforms: FlatTransforms,
    opacity: f32,
    scroll: Offset,
    fixed_background_acts_as_scroll: bool,
    properties: PropertyTreeState,
    canvas_contents: Option<CanvasContentsContext>,
    rendering_context_id: u64,
    should_flatten_inherited_transform: bool,
    can_inherit_backface_visibility: bool,
    directly_composited_container_paint_offset_subpixel_delta: Offset,
    subtree_should_use_infinite_cull_rect: bool,
    flex_or_grid_items: bool,
    svg: Option<TransformMatrix>,
}

/// The context an occurrence's last PaintPropertyTreeBuilder run handed its
/// children (UpdateForChildren's output), kept on its FragmentData.
///
/// Source: paint_property_tree_builder.cc ObjectPaintPropertyTreeBuilder::
/// UpdateForSelf / UpdateForChildren. For an object that needs no property
/// update and no forced subtree update, every Update* step leaves its
/// existing ObjectPaintProperties node alone and only advances the context
/// past it (context_.current.transform = properties_->Transform(), the
/// paint offset from fragment_data_.PaintOffset(), the containing-block
/// contexts). The nodes this context names are those same live nodes; the
/// flat values beside them (origin, paint space, flat clip and cull, the
/// flat transform list, scroll anchor, SVG matrix) are Rechrom's flat
/// accumulation, which Chromium's context does not carry and which the
/// children already store in their FlatReplayData. Reusing the advanced
/// context therefore stands in for re-deriving it from the existing nodes.
///
/// Valid while `generation` is the walk's `child_context_generation`: a
/// property value changed in place outside a walk (a retained scroll, a
/// composited animation update: PropertyValuesChangedOutsideWalk) or a
/// change of the viewport nodes the root inherits bumps it, as such a change
/// reaches the flat values and cull rects only through a build. The direct
/// transform update keeps
/// the contexts it rewrites current. Replacing LocalBorderBoxProperties
/// outside a build drops it (PrePaintFragmentData).
pub(crate) struct CachedChildContext {
    context: ChildContext,
    generation: u64,
}

struct RootFixedPositionContext {
    offset: Offset,
    paint_space: Offset,
    transforms: FlatTransforms,
    clip: Option<PaintRect>,
    properties: PropertyTreeState,
    scroll_translation: Option<Arc<TransformPaintPropertyNode>>,
}
/// The PaintPropertyTreeBuilder output contexts of one visit. Walk-local:
/// the PaintTreeNode that carries it lives for the visit only (Chromium's
/// PaintPropertyTreeBuilderContext is a stack value of PrePaintTreeWalk::
/// Walk, pre_paint_tree_walk.cc:1470-1590).
struct NodeCache {
    /// The context this object was built with.
    inherited: InheritedContext,
    /// The context its children inherit (UpdateForChildren's output).
    children: ChildContext,
    /// The maximum change the in-place `Update` of this object's own
    /// property slots reported (PaintPropertyTreeBuilder `properties_changed_`).
    own_change: PaintPropertyChangeType,
    /// The ObjectPaintProperties slots this build registered.
    own_keys: Vec<(u64, PaintPropertyKey)>,
}

/// The effect state one paint-state occurrence of a LayoutObject gives its
/// logical (out-of-flow) children: its border-box effect node and id and its
/// accumulated opacity.
#[derive(Clone)]
struct LogicalEffectContext {
    fragment_instance_id: u64,
    fragmentainer: u64,
    effect: Arc<EffectPaintPropertyNode>,
    effect_id: u64,
    opacity: f32,
}

/// The slot ids a builder state names are its nodes' ids (the resident
/// ObjectPaintProperties nodes), and its scroll id is the nearest scroll.
fn SyncPropertyIds(state: &mut PropertyTreeState) {
    state.transform_id = state.nodes.transform.id;
    state.clip_id = state.nodes.clip.id;
    state.effect_id = state.nodes.effect.id;
    state.scroll_id = state.nodes.nearest_scroll().map_or(0, |scroll| scroll.id);
}

fn HasOwnPropertyRole(node: &PaintTreeNode<'_>, role: PaintPropertyRole) -> bool {
    node.HasOwnPropertyRole(role)
}

impl PaintTreeNode<'_> {
    pub fn HasOwnPaintProperties(&self) -> bool {
        self.cache
            .as_ref()
            .is_some_and(|cache| !cache.own_keys.is_empty())
    }

    /// ObjectPaintProperties slot query for Paint consumers that must
    /// distinguish an own node from an inherited final PropertyTreeState.
    pub fn HasOwnPropertyRole(&self, role: PaintPropertyRole) -> bool {
        self.cache
            .as_ref()
            .is_some_and(|cache| cache.own_keys.iter().any(|(_, key)| key.role == role))
    }
}

// Source: paint_property_tree_builder.cc:2632-2633,2713-2730. Chromium
// compares only ObjectPaintProperties::Filter's own operation list. An
// ancestor's inherited tainted-subtree bit must not make this object look as
// though its own Filter changed taint.
fn OwnFilterOriginTainted(node: &PaintTreeNode<'_>) -> bool {
    HasOwnPropertyRole(node, PaintPropertyRole::Filter)
        && paint_model::property_cell::with_access(|access| {
            node.properties.nodes.effect.state(access).filters.origin_tainted()
        })
}

fn FilterTaintChanged(was_origin_tainted: bool, current: &PaintTreeNode<'_>) -> bool {
    was_origin_tainted != OwnFilterOriginTainted(current)
}

pub use crate::PrePaintUpdateStats;

/// Canonical ObjectPaintProperties snapshots changed by Blink's pending
/// transform/opacity update path. The lifecycle identity is retained; these
/// values are ready for PaintArtifactCompositor's direct update methods.
#[derive(Clone, Debug, Default)]
pub struct DirectCompositedAnimationPropertyUpdate {
    pub transforms: Vec<Arc<TransformPaintPropertyNode>>,
    pub effects: Vec<Arc<EffectPaintPropertyNode>>,
    pub bindings: Vec<CompositedAnimationPropertyBinding>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompositedAnimationProperty {
    Transform,
    Opacity,
}

#[derive(Clone, Copy, Debug)]
pub struct CompositedAnimationPropertyBinding {
    pub node_id: u64,
    pub property: CompositedAnimationProperty,
    pub element_id: foundation::CompositorElementId,
}

pub struct PrePaintTreeWalk {
    pub next_property_id: u64,
    pub property_node_store: PaintPropertyNodeStore,
    visual_viewport_state: Option<paint_model::PaintVisualViewportState>,
    visual_viewport_scroll_offset:
        Option<(foundation::CompositorElementId, foundation::gfx::Vector2dF)>,
    /// The slot keys `RegisterProperty` handed out in this pass.
    property_keys: BTreeMap<u64, PaintPropertyKey>,
    /// The Layout result the last committed walk visited and the root of its
    /// fragment tree. Every value PrePaint produced lives in that result's
    /// FragmentData; the lease keeps the root cursor (and the occurrences the
    /// between-walk queries reach from it) alive.
    retained_source_lease: Option<PaintFragmentLease>,
    retained_root: Option<PaintFragmentCursor<'static>>,
    /// The same pair for the walk in progress until CommitWalk.
    pending_source_lease: Option<PaintFragmentLease>,
    pending_root: Option<PaintFragmentCursor<'static>>,
    /// The walk's lease names a different Paint publication state than the
    /// last committed walk's: no FragmentData of the preceding state is
    /// reachable, so the walk visits every object.
    source_owner_changed: bool,
    /// The walk's lease names a replacement LayoutResult (or a different
    /// state); every visited object re-reads the wheel-handler registry.
    source_result_changed: bool,
    /// Walk-scoped: no subtree may be skipped (first walk, a replacement
    /// publication state, a viewport or document resource change).
    full_walk: bool,
    /// A walk visited this document at least once.
    walked_once: bool,
    viewport: Option<PaintRect>,
    paint_invalidator: PaintInvalidator,
    viewport_properties: PropertyTreeState,
    root_fixed_position: Option<RootFixedPositionContext>,
    local_frame_views: HashMap<u64, EmbeddedLocalFrameView>,
    root_blocking_wheel_targets: Arc<[u64]>,
    active_blocking_wheel_targets: Arc<[u64]>,
    retained_local_frame_leases: Vec<PaintFragmentLease>,
    local_frame_sources_changed: bool,
    local_frame_depth: u32,
    /// The ScopedDisplayItemFragment of the object being visited
    /// (core/paint/box_fragment_painter.cc:394-403,
    /// core/paint/paint_layer_painter.cc:776-778). Set by the parent before
    /// each child visit; `None` outside a walk.
    display_item_fragment_scope: Option<u32>,
    /// The fragmentainer identity of the object being visited
    /// (PrePaintTreeWalk's `fragmentainer_idx`; 0 outside any). Set by the
    /// parent before each child visit; `None` outside a walk.
    fragmentainer_scope: Option<u64>,
    completed_local_frame_prepaint: HashSet<u64>,
    // Source: core/paint/paint_property_tree_builder.cc:4692-4701.
    requires_main_thread_for_background_attachment_fixed: Option<bool>,
    // LayoutView::BackgroundRect changes mark every registered fixed
    // background object for full background invalidation in Chromium.
    view_background_geometry_changed: bool,
    /// The walk root's LayoutView background rects, read once per walk, and
    /// the address of the root's FragmentPaintData they belong to.
    view_background_rects: Option<(usize, ViewBackgroundRects)>,
    /// The (slot id, key) of every property slot registered by the builds
    /// in progress (BuildNode drains its own range into NodeCache::own_keys),
    /// and the change each in-place slot update reported.
    own_key_log: Vec<(u64, PaintPropertyKey)>,
    own_change_log: Vec<PaintPropertyChangeType>,
    /// Decision D2: the logical out-of-flow effect parent is found in the
    /// builder context. Every visited paint-state occurrence of a
    /// LayoutObject (by node id) records the effect state its logical
    /// children inherit; the entries of clean occurrences stay valid.
    logical_effect_contexts: HashMap<u64, Vec<LogicalEffectContext>>,
    /// Walk-scoped: node ids whose logical effect context this walk created
    /// or changed; their out-of-flow logical children are fixed up after it.
    logical_effects_changed: HashSet<u64>,
    /// Node ids of the paint-state-establishing physical ancestors of the
    /// object being visited (0 for one that establishes none).
    physical_ancestor_ids: Vec<u64>,
    /// DisplayItemClient ids of the objects on the walk's stack.
    walk_ancestors: Vec<u64>,
    /// Walk-scoped: objects the walk must reach even when no dirty bit leads
    /// to them, with their ancestors: the deferred transform updates the
    /// direct path could not apply (LocalFrameView::ExecuteAllPendingUpdates),
    /// the embedded local frame
    /// owners and the display-locked objects with a pending child walk.
    must_visit_clients: HashSet<u64>,
    /// The local frame owners and their ancestors the last walk reached.
    local_frame_ancestors: HashSet<u64>,
    next_local_frame_ancestors: HashSet<u64>,
    /// DisplayLockContext::SetNeedsPrePaintSubtreeWalk: display-locked
    /// objects whose blocked children must be walked with a forced context
    /// once the lock opens, with the clients of their ancestors.
    pending_display_lock_walks: HashMap<u64, Vec<u64>>,
    /// Walk-scoped: the pending child walks of the preceding walks; an entry
    /// the walk does not reach (its object left the tree) is dropped.
    previous_display_lock_walks: HashMap<u64, Vec<u64>>,
    /// The document's paint resources at the last walk, and those of each
    /// embedded local frame by owner client.
    last_document_resources: Option<Arc<PaintResources>>,
    last_frame_resources: HashMap<u64, Arc<PaintResources>>,
    /// Whether the last outline pass found an inline outline continuation
    /// group, and whether this walk visited an object of one. Only grouped
    /// inline outlines are prepared (every other outline is collected at
    /// paint time from the live fragment, as BoxFragmentPainter::
    /// PaintOutline reads LayoutObject::OutlineRects in Chromium).
    inline_outline_groups: bool,
    visited_inline_outline: bool,
    /// Root-to-node transform products of property nodes seen this walk.
    /// The Arc keeps each key's allocation alive for the walk.
    transform_world_memo: HashMap<usize, (Arc<TransformPaintPropertyNode>, TransformMatrix)>,
    pub stats: PrePaintUpdateStats,
    intersection_observation_state:
        foundation::intersection_observation_state::IntersectionObservationState,
    /// Visited objects whose paint flags this walk consumed. Chromium clears
    /// them once per object after WalkLayoutObject handled every fragment of
    /// it (pre_paint_tree_walk.cc:1584-1585); PrePaint reads the bits at each
    /// fragment's visit, so they are cleared after the walk.
    deferred_paint_flag_clears: Vec<*const FragmentPaintData>,
    /// Walks that visited objects (and so cleared their paint flags), and
    /// that count when Paint last recorded an artifact.
    consuming_walks: u64,
    recorded_consuming_walks: u64,
    /// Slots a visited object no longer owns (ObjectPaintProperties::Clear*,
    /// object_paint_properties.h:86); released at the end of the walk unless
    /// another object registered the same key.
    released_property_keys: Vec<PaintPropertyKey>,
    /// DisplayItemClient id -> the visited occurrence carrying it, kept by
    /// the walk for the committed paint's client validation
    /// (committed_paint_clients; paint_controller.cc:127-143).
    pub(crate) committed_clients: crate::committed_paint_clients::CommittedClientDirectory,
    /// The validity generation of the FragmentData child contexts
    /// (CachedChildContext); bumped when property values changed between
    /// walks.
    child_context_generation: u64,
    /// Walk-scoped: the walk root's inherited context (the viewport nodes)
    /// changed value since the walk began.
    root_context_changed: bool,
    /// Walk-scoped: objects the walk must build although nothing dirtied
    /// them (deferred transform updates the direct path could not apply).
    must_build_clients: HashSet<u64>,
}

impl Default for PrePaintTreeWalk {
    fn default() -> Self {
        Self {
            next_property_id: 1,
            property_node_store: Default::default(),
            visual_viewport_state: None,
            visual_viewport_scroll_offset: None,
            property_keys: BTreeMap::new(),
            retained_source_lease: None,
            retained_root: None,
            pending_source_lease: None,
            pending_root: None,
            source_owner_changed: false,
            source_result_changed: false,
            full_walk: true,
            walked_once: false,
            viewport: None,
            paint_invalidator: Default::default(),
            viewport_properties: PropertyTreeState::default(),
            root_fixed_position: None,
            local_frame_views: HashMap::default(),
            root_blocking_wheel_targets: Arc::from([]),
            active_blocking_wheel_targets: Arc::from([]),
            retained_local_frame_leases: Vec::new(),
            local_frame_sources_changed: false,
            local_frame_depth: 0,
            display_item_fragment_scope: None,
            fragmentainer_scope: None,
            completed_local_frame_prepaint: HashSet::new(),
            requires_main_thread_for_background_attachment_fixed: None,
            view_background_geometry_changed: false,
            view_background_rects: None,
            own_key_log: Vec::new(),
            own_change_log: Vec::new(),
            logical_effect_contexts: HashMap::default(),
            logical_effects_changed: HashSet::new(),
            physical_ancestor_ids: Vec::new(),
            walk_ancestors: Vec::new(),
            must_visit_clients: HashSet::new(),
            local_frame_ancestors: HashSet::new(),
            next_local_frame_ancestors: HashSet::new(),
            pending_display_lock_walks: HashMap::default(),
            previous_display_lock_walks: HashMap::default(),
            last_document_resources: None,
            last_frame_resources: HashMap::default(),
            inline_outline_groups: true,
            visited_inline_outline: false,
            transform_world_memo: HashMap::default(),
            stats: PrePaintUpdateStats::default(),
            intersection_observation_state: Default::default(),
            deferred_paint_flag_clears: Vec::new(),
            consuming_walks: 0,
            recorded_consuming_walks: 0,
            released_property_keys: Vec::new(),
            committed_clients: Default::default(),
            child_context_generation: 0,
            root_context_changed: true,
            must_build_clients: HashSet::new(),
        }
    }
}

fn Intersect(mut a: PaintRect, b: &PaintRect) -> PaintRect {
    let right = (a.x + a.width).min(b.x + b.width);
    let bottom = (a.y + a.height).min(b.y + b.height);
    a.x = a.x.max(b.x);
    a.y = a.y.max(b.y);
    a.width = (right - a.x).max(0.0);
    a.height = (bottom - a.y).max(0.0);
    a
}

pub fn SvgViewBoxTransform(fragment: &FragmentPaintData) -> Option<TransformMatrix> {
    (fragment.paint.SourceKind() == NodeKind::kSvgRoot)
        .then(|| fragment.paint.SvgViewport().map(|v| v.content_transform))
        .flatten()
}

// Source: core/paint/svg_root_painter.cc:22-65,
// SVGRootPainter::PixelSnappedSize/TransformToPixelSnappedBorderBox. The
// caller supplies context_.current.paint_offset after PaintOffsetTranslation,
// so this consumes only the retained subpixel accumulation. DOM geometry uses
// SvgViewBoxTransform above and intentionally remains unsnapped.
pub fn SvgTransformToPixelSnappedBorderBox(
    fragment: &FragmentPaintData,
    paint_offset: Offset,
    paint_snap_offset: Offset,
    pixel_snap_scale: f64,
) -> Option<TransformMatrix> {
    if fragment.paint.SourceKind() != NodeKind::kSvgRoot {
        return None;
    }
    let viewport = fragment.paint.SvgViewport()?;
    let pixel_snap_scale = if pixel_snap_scale.is_finite() && pixel_snap_scale > 0.0 {
        pixel_snap_scale
    } else {
        1.0
    };
    // Chromium snaps this rect after effective zoom/device scale has entered
    // layout coordinates. Rechrom retains CSS paint coordinates, so project
    // through the same artifact-to-device scale and property-space offset as
    // every other pixel-snapped paint rect before returning to local space.
    let snap = |value: f64, offset: f64| SnapToPixel(value + offset, pixel_snap_scale) - offset;
    let snapped_x = snap(paint_offset.x, paint_snap_offset.x);
    let snapped_y = snap(paint_offset.y, paint_snap_offset.y);
    let snap_size = |size: f64, location: f64, offset: f64| {
        let mut snapped = SnapToPixel(location + offset + size, pixel_snap_scale)
            - SnapToPixel(location + offset, pixel_snap_scale);
        // foundation/blink_geometry/geometry/layout_unit.h:807-815.
        // Keep a non-trivial LayoutUnit extent from disappearing when both
        // edges round to the same CSS pixel.
        if snapped == 0.0 && size.abs() * pixel_snap_scale > 4.0 / 64.0 {
            snapped = size.signum() / pixel_snap_scale;
        }
        snapped
    };
    let snapped_width = snap_size(fragment.ViewSize().width, paint_offset.x, paint_snap_offset.x);
    let snapped_height = snap_size(fragment.ViewSize().height, paint_offset.y, paint_snap_offset.y);
    let mut offset_to_border_box = TranslationTransform(snapped_x, snapped_y);
    if fragment.ViewSize().width != 0.0 && fragment.ViewSize().height != 0.0 {
        let scale = if viewport.is_document_element {
            Some((
                snapped_width / fragment.ViewSize().width,
                snapped_height / fragment.ViewSize().height,
            ))
        } else if viewport.inline_root_pixel_snapping_scale_adjustment
            && (fragment.ViewSize().width > snapped_width || fragment.ViewSize().height > snapped_height)
        {
            let scale_x = snapped_width / fragment.ViewSize().width;
            let scale_y = snapped_height / fragment.ViewSize().height;
            let uniform_scale = scale_x.min(scale_y);
            let scaled_width = fragment.ViewSize().width * uniform_scale;
            let scaled_height = fragment.ViewSize().height * uniform_scale;
            if snapped_width - scaled_width > 1.0 || snapped_height - scaled_height > 1.0 {
                Some((scale_x, scale_y))
            } else {
                Some((uniform_scale, uniform_scale))
            }
        } else {
            None
        };
        if let Some((scale_x, scale_y)) = scale {
            let mut scale = TransformMatrix::default();
            scale.values[0] = scale_x;
            scale.values[5] = scale_y;
            offset_to_border_box = MultiplyTransforms(&offset_to_border_box, &scale);
        }
    }
    Some(MultiplyTransforms(
        &offset_to_border_box,
        &viewport.content_transform,
    ))
}

// Geometry shared by painting and DOM box queries. Clips, effects and logical
// paint parenting do not change the physical fragment's coordinate mapping.
#[derive(Clone, Copy)]
pub struct FragmentGeometry {
    pub absolute_offset: Offset,
    pub paint_offset: Offset,
    pub paint_space_origin: Offset,
    pub applies_transform: bool,
    pub applies_paint_offset_translation: bool,
    pub directly_composited_container_paint_offset_subpixel_delta: Offset,
    /// Paint offset captured immediately before StickyTranslation. Chromium
    /// records this in CompositorStickyConstraint while the transform matrix
    /// receives the rounded sum of this value and StickyPositionOffset.
    pub sticky_pixel_snap_offset: Offset,
    pub resets_paint_space: bool,
}

// Source: paint_property_tree_builder.cc:558-578. The local-subframe
// LayoutView branch remains unsupported until the frame-tree relationship is
// an explicit Layout boundary fact.
fn NeedsIsolationNodes(fragment: &FragmentPaintData) -> bool {
    fragment.paint.PaintLayerClientId() != 0
        && (fragment.paint.ShouldApplyPaintContainment()
            || (fragment.paint.ShouldApplyStyleContainment()
                && fragment.paint.ShouldApplyLayoutContainment()))
}

fn IsSvgLayoutChild(kind: NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::kSvgGroup
            | NodeKind::kSvgHiddenContainer
            | NodeKind::kSvgResourceClipper
            | NodeKind::kSvgResourceMasker
            | NodeKind::kSvgResourceGradient
            | NodeKind::kSvgResourcePattern
            | NodeKind::kSvgResourceFilter
            | NodeKind::kSvgResourceMarker
            | NodeKind::kSvgForeignObject
            | NodeKind::kSvgText
            | NodeKind::kSvgInline
            | NodeKind::kSvgTSpan
            | NodeKind::kSvgTextPath
            | NodeKind::kSvgShape
            | NodeKind::kSvgViewportContainer
    )
}

fn HasReferenceFilterOrReflection(style: &PaintStyle) -> bool {
    style.has_reference_filter_or_reflection()
}

fn HasSupportedDirectCompositingReasons(fragment: &FragmentPaintData, _style: &PaintStyle) -> bool {
    !CompositingReasonFinder::DirectReasonsForPaintProperties(fragment).IsEmpty()
}

fn HasTransformPayload(style: &PaintStyle) -> bool {
    style.has_transform_payload()
}

fn TransformPayloadForRole(style: &PaintStyle, role: PaintPropertyRole) -> Option<PaintTransform> {
    match role {
        PaintPropertyRole::Translate => style.translate(),
        PaintPropertyRole::Rotate => style.rotate(),
        PaintPropertyRole::Scale => style.scale(),
        PaintPropertyRole::Offset => style.offset_transform.clone(),
        PaintPropertyRole::Transform => style.transform(),
        _ => None,
    }
}

// Source: paint_property_tree_builder.cc:1326-1410,1601-1714. These roles are
// created independently and in this exact order. A direct reason or active
// animation retains an identity slot even when the current style value is
// absent, matching NeedsIndividualTransform/NeedsTransform.
fn TransformPropertyRoles(
    fragment: &FragmentPaintData,
    style: &PaintStyle,
) -> Vec<PaintPropertyRole> {
    if IsSvgLayoutChild(fragment.paint.SourceKind()) {
        return Vec::new();
    }
    let mut roles = Vec::new();
    for (role, has_style_value, has_active_animation) in [
        (
            PaintPropertyRole::Translate,
            style.has_translate(),
            style.active_translate_animation(),
        ),
        (
            PaintPropertyRole::Rotate,
            style.has_rotate(),
            style.active_rotate_animation(),
        ),
        (
            PaintPropertyRole::Scale,
            style.has_scale(),
            style.active_scale_animation(),
        ),
    ] {
        if has_style_value
            || has_active_animation
            || !CompositingReasonFinder::TransformReasons(fragment, role, false).IsEmpty()
        {
            roles.push(role);
        }
    }
    if style.offset_transform.is_some() {
        roles.push(PaintPropertyRole::Offset);
    }
    if style.has_transform()
        || style.active_transform_animation()
        || style.preserves_3d()
        || style.backface_visibility_hidden()
        || !CompositingReasonFinder::TransformReasons(fragment, PaintPropertyRole::Transform, false)
            .IsEmpty()
    {
        roles.push(PaintPropertyRole::Transform);
    }
    roles
}

// Source: paint_property_tree_builder.cc:609-702. Canvas draw-element,
// anchor-position scroll adjustment, resizer-only overflow controls,
// frozen embedded frames, local-subframe isolation and direct compositing
// reasons not represented by the typed boundary remain explicitly
// unsupported; no style/display proxy is substituted for them.
fn NeedsPaintOffsetTranslation(fragment: &FragmentPaintData, style: Option<&PaintStyle>) -> bool {
    if !fragment.paint.IsBoxModelObject() || IsSvgLayoutChild(fragment.paint.SourceKind()) {
        return false;
    }
    if fragment.paint.IsLayoutView() {
        return true;
    }
    let Some(style) = style else {
        return false;
    };
    if NeedsIsolationNodes(fragment)
        || HasTransformPayload(style)
        || fragment.ViewHasScrollContainer()
        || fragment.paint.StickyConstraint().is_some()
        || fragment.paint.StickyOffset().x != 0.0
        || fragment.paint.StickyOffset().y != 0.0
        || fragment.paint.NeedsAnchorPositionScrollAdjustment()
        || fragment.ViewScrollbarsPaintAsOverlay().is_some()
        || fragment.paint.SourceKind() == NodeKind::kSvgRoot
        || (fragment.paint.PaintLayerClientId() != 0
            && (HasReferenceFilterOrReflection(style) || style.has_backdrop_filters()))
        || fragment.paint.FixedToView()
    {
        return true;
    }
    if HasSupportedDirectCompositingReasons(fragment, style) || style.backface_visibility_hidden() {
        return !fragment.paint.IsLayoutInline() || fragment.paint.PaintLayerIsSelfPainting();
    }
    false
}

// Source: cull_rect_updater.cc:129-140 and
// cc/paint/filter_operations.cc:90-159. Reference and reflection filters keep
// their moving-pixel classification even when their raster owner is
// explicitly unsupported; they must never fall back to a guessed finite
// outset.
fn HasFilterThatMovesPixels(filters: &[PaintFilterOperation]) -> bool {
    filters.iter().any(|filter| {
        matches!(
            filter.r#type,
            PaintFilterType::kBlur
                | PaintFilterType::kDropShadow
                | PaintFilterType::kReference
                | PaintFilterType::kBoxReflect
        )
    })
}

fn HasPerspective(matrix: &TransformMatrix) -> bool {
    matrix.values[3] != 0.0
        || matrix.values[7] != 0.0
        || matrix.values[11] != 0.0
        || matrix.values[15] != 1.0
}

fn IsScaleOrTranslation(matrix: &TransformMatrix) -> bool {
    let v = matrix.values;
    v[1] == 0.0
        && v[2] == 0.0
        && v[3] == 0.0
        && v[4] == 0.0
        && v[6] == 0.0
        && v[7] == 0.0
        && v[8] == 0.0
        && v[9] == 0.0
        && v[11] == 0.0
        && v[15] == 1.0
}

// Source: paint_property_tree_builder.cc:704-736.
fn CanPropagateSubpixelAccumulation(
    fragment: &FragmentPaintData,
    style: &PaintStyle,
) -> (bool, bool) {
    if fragment.paint.PaintLayerClientId() == 0 {
        return (true, true);
    }
    let direct = CompositingReasonFinder::DirectReasonsForPaintProperties(fragment);
    if !direct
        .Intersection(foundation::CompositingReasonCombos::kPreventingSubpixelAccumulationReasons)
        .IsEmpty()
        || style.active_transform_animation()
        || style.active_scale_animation()
        || style.active_rotate_animation()
    {
        return (false, false);
    }
    let translate = style.translate();
    let rotate = style.rotate();
    let scale = style.scale();
    let transform = style.transform();
    let matrix = [
        translate.as_ref(),
        rotate.as_ref(),
        scale.as_ref(),
        style.offset_transform.as_ref(),
        transform.as_ref(),
    ]
    .into_iter()
    .flatten()
    .fold(TransformMatrix::default(), |matrix, transform| {
        MultiplyTransforms(&matrix, &ResolvePaintTransform(transform, fragment.ViewSize()))
    });
    if !IsScaleOrTranslation(&matrix) {
        return (false, false);
    }
    (matrix.values[0] == 1.0, matrix.values[5] == 1.0)
}

pub fn ResolveFragmentGeometry(
    fragment: &FragmentPaintData,
    // The fragment's offset in its paint-tree parent
    // (PaintFragmentCursor::ViewOffsetInContainer).
    link_offset: Offset,
    parent_offset: Offset,
    mut paint_space_origin: Offset,
    transforms: &mut Vec<TransformMatrix>,
    mut property_space_delta: Offset,
    inherited_directly_composited_subpixel_delta: Offset,
) -> FragmentGeometry {
    let absolute_offset = Offset {
        x: parent_offset.x + link_offset.x,
        y: parent_offset.y + link_offset.y,
    };
    let mut geometry = FragmentGeometry {
        absolute_offset,
        paint_offset: Offset {
            x: absolute_offset.x - paint_space_origin.x,
            y: absolute_offset.y - paint_space_origin.y,
        },
        paint_space_origin,
        applies_transform: false,
        applies_paint_offset_translation: false,
        directly_composited_container_paint_offset_subpixel_delta:
            inherited_directly_composited_subpixel_delta,
        sticky_pixel_snap_offset: Offset::default(),
        resets_paint_space: false,
    };
    let style: Option<&PaintStyle> = fragment.paint.HasSource().then_some(&fragment.paint.style);
    let establishes = fragment.ViewEstablishesPaintState();
    let position = if fragment.paint.HasSource() {
        fragment.paint.Position()
    } else {
        Position::kStatic
    };
    // NeedsStickyTranslation is driven by sticky constraints, not by the
    // current offset. At scroll origin the offset is commonly zero, but the
    // identity property node is still required for impl-side scrolling.
    let has_sticky_translation = position == Position::kSticky
        && (fragment.paint.StickyConstraint().is_some()
            || fragment.paint.StickyOffset().x != 0.0
            || fragment.paint.StickyOffset().y != 0.0);
    let has_paint_offset_translation = NeedsPaintOffsetTranslation(fragment, style);
    let text_combine = fragment.paint.TextCombine();
    if establishes
        && (has_paint_offset_translation
            || text_combine.is_some()
            || style.is_some_and(|style| {
                has_sticky_translation
                    || fragment.paint.SvgLocalTransform().is_some()
                    || !TransformPropertyRoles(fragment, style).is_empty()
                    || fragment
                        .paint
                        .SvgShape()
                        .is_some_and(|shape| shape.local_transform.is_some())
            }))
    {
        geometry.applies_transform = true;
        if has_paint_offset_translation {
            // Source: paint_property_tree_builder.cc:738-803. Blink uses
            // ToRoundedVector2d on the LayoutUnit paint offset: this is CSS
            // layout-pixel rounding, independent of device scale. Retain the
            // fractional accumulation for local painting; SVGRootPainter
            // consumes that residual in its ReplacedContentTransform.
            let translation_offset = Offset {
                x: SnapToPixel(geometry.paint_offset.x + property_space_delta.x, 1.0)
                    - property_space_delta.x,
                y: SnapToPixel(geometry.paint_offset.y + property_space_delta.y, 1.0)
                    - property_space_delta.y,
            };
            let translation = TranslationTransform(translation_offset.x, translation_offset.y);
            transforms.push(translation);
            paint_space_origin.x += translation_offset.x;
            paint_space_origin.y += translation_offset.y;
            geometry.paint_offset.x -= translation_offset.x;
            geometry.paint_offset.y -= translation_offset.y;
            if NeedsIsolationNodes(fragment) {
                geometry.paint_offset = Offset::default();
                geometry.directly_composited_container_paint_offset_subpixel_delta =
                    Offset::default();
            } else if let Some(style) = style {
                let can_propagate = CanPropagateSubpixelAccumulation(fragment, style);
                if !can_propagate.0 {
                    geometry.paint_offset.x = 0.0;
                    geometry
                        .directly_composited_container_paint_offset_subpixel_delta
                        .x = 0.0;
                }
                if !can_propagate.1 {
                    geometry.paint_offset.y = 0.0;
                    geometry
                        .directly_composited_container_paint_offset_subpixel_delta
                        .y = 0.0;
                }
            }
            property_space_delta = Offset::default();
            geometry.applies_paint_offset_translation = true;
            geometry.resets_paint_space = true;
        }
        if has_sticky_translation {
            if !fragment.paint.StickyOffset().x.is_finite()
                || !fragment.paint.StickyOffset().y.is_finite()
            {
                panic!("sticky offset must be finite");
            }
            // Source: paint_property_tree_builder.cc:917-940, 4143-4157.
            // StickyTranslation owns the pixel-rounded total, while local
            // painting retains the remainder. Use Blink's floor(x + .5)
            // rounding, including its negative half-integral behavior.
            geometry.sticky_pixel_snap_offset = geometry.paint_offset;
            let total = Offset {
                x: geometry.paint_offset.x + fragment.paint.StickyOffset().x,
                y: geometry.paint_offset.y + fragment.paint.StickyOffset().y,
            };
            let rounded = Offset {
                x: SnapToPixel(total.x, 1.0),
                y: SnapToPixel(total.y, 1.0),
            };
            let translation = TranslationTransform(rounded.x, rounded.y);
            transforms.push(translation);
            geometry.paint_offset = Offset {
                x: total.x - rounded.x,
                y: total.y - rounded.y,
            };
        }
        if let Some(style) = style.filter(|_| fragment.paint.SvgLocalTransform().is_none()) {
            // Blink snaps the layout paint offset in the current property
            // parent, whose ScrollTranslation remains a separate ancestor.
            let reference_rect = ResolveTransformReferenceRect(
                style.transform_reference_box(),
                fragment.ViewSize(),
                fragment.Borders(),
                fragment.Padding(),
            );
            let transform_origin = style.transform_origin();
            for role in TransformPropertyRoles(fragment, style) {
                let payload = TransformPayloadForRole(style, role);
                let resolved =
                    payload
                        .as_ref()
                        .map_or_else(TransformMatrix::default, |transform| {
                            ResolveLocalTransformAroundReferenceBox(
                                transform,
                                Offset {
                                    x: geometry.paint_offset.x + property_space_delta.x,
                                    y: geometry.paint_offset.y + property_space_delta.y,
                                },
                                reference_rect,
                                transform_origin.as_ref(),
                            )
                        });
                transforms.push(MultiplyTransforms(
                    &TranslationTransform(-property_space_delta.x, -property_space_delta.y),
                    &resolved,
                ));
            }
        }
        if let Some(transform) = fragment.paint.SvgLocalTransform() {
            transforms.push(transform);
            paint_space_origin = geometry.absolute_offset;
            geometry.paint_offset = Offset::default();
            geometry.resets_paint_space = true;
        }
        if let Some(local_transform) = fragment
            .paint
            .SvgShape()
            .and_then(|shape| shape.local_transform)
        {
            transforms.push(local_transform);
        }
        if let Some(text_combine) = &text_combine {
            transforms.push(TextCombineTransform(text_combine, geometry.paint_offset));
        }
    }
    geometry.paint_space_origin = paint_space_origin;
    geometry
}

// layout_text_combine.cc:189-212. Keep the anonymous wrapper's canvas
// transform as a real PrePaint transform; canonical record and raster replay
// already consume this typed matrix path.
fn TextCombineTransform(data: &TextCombinePaintData, paint_offset: Offset) -> TransformMatrix {
    let around = |matrix: TransformMatrix, pivot: Offset| {
        MultiplyTransforms(
            &TranslationTransform(pivot.x, pivot.y),
            &MultiplyTransforms(&matrix, &TranslationTransform(-pivot.x, -pivot.y)),
        )
    };
    let mut output = TransformMatrix::default();
    if data.synthetic_oblique {
        let mut skew = TransformMatrix::default();
        skew.values[1] = (-15.0_f64).to_radians().tan();
        let inline_offset = data
            .scale_x
            .map_or(0.0, |scale_x| data.inline_spacing / scale_x);
        output = MultiplyTransforms(
            &output,
            &around(
                skew,
                Offset {
                    x: paint_offset.x + inline_offset,
                    y: paint_offset.y + data.text_top_offset,
                },
            ),
        );
    }
    if let Some(scale_x) = data.scale_x {
        let mut scale = TransformMatrix::default();
        scale.values[0] = scale_x;
        output = MultiplyTransforms(&output, &around(scale, paint_offset));
    }
    output
}

pub fn ChildGeometryOrigin(fragment: &FragmentPaintData, absolute_offset: Offset) -> Offset {
    if fragment.ViewEstablishesPaintState()
        && (fragment.paint.ScrollOffset().x != 0.0 || fragment.paint.ScrollOffset().y != 0.0)
    {
        Offset {
            x: absolute_offset.x - fragment.paint.ScrollOffset().x,
            y: absolute_offset.y - fragment.paint.ScrollOffset().y,
        }
    } else {
        absolute_offset
    }
}

pub fn ResolveChildGeometryOrigin(
    child: &FragmentPaintData,
    // The child's offset in this parent (PaintFragmentCursor::ViewOffsetInContainer).
    child_link_offset: Offset,
    child_origin: Offset,
    paint_space_origin: Offset,
) -> Offset {
    if child.paint.HasSource()
        && matches!(
            child.paint.SourceKind(),
            NodeKind::kSvgShape | NodeKind::kSvgGroup | NodeKind::kSvgViewportContainer
        )
    {
        Offset {
            // SVG child geometry is expressed in the replaced content's local
            // coordinates. Chromium resets the paint offset after installing
            // ReplacedContentTransform, so descendants start at the snapped
            // content origin instead of reintroducing the root's residual.
            x: paint_space_origin.x - child_link_offset.x,
            y: paint_space_origin.y - child_link_offset.y,
        }
    } else {
        child_origin
    }
}

// Blink UpdateOverflowClip/UpdateInnerBorderRadiusClip uses paint_offset in
// context_.current.transform space (paint_property_tree_builder.cc:3148,3290).
// Convert flat geometry only into the property parent:
// P * (clip + delta) == F * clip. The same delta places paint-offset snapping
// in that parent's space. Comparing the complete linear parts proves this is a pure
// translation even across a CSS/SVG paint-space reset; summing scroll offsets
// would be wrong when an intervening scale changes their coordinate space.
impl PrePaintTreeWalk {
    /// FlatToPropertySpaceOffset with each property node's root-to-node
    /// product memoized for this walk. Property nodes are immutable
    /// snapshots, and the product is formed in the same order (parent world
    /// times own matrix), so results are bit-identical to the unmemoized
    /// root-first loop.
    pub(crate) fn FlatToPropertySpaceOffsetMemo(
        &mut self,
        flat: &[TransformMatrix],
        property: &Arc<TransformPaintPropertyNode>,
    ) -> Option<Offset> {
        let world = self.TransformWorld(property);
        FlatToWorldSpaceOffset(flat, &world)
    }

    fn TransformWorld(&mut self, node: &Arc<TransformPaintPropertyNode>) -> TransformMatrix {
        let key = Arc::as_ptr(node) as usize;
        if let Some((_, world)) = self.transform_world_memo.get(&key) {
            return *world;
        }
        let (parent, matrix) = paint_model::property_cell::with_access(|access| {
            let state = node.state(access);
            (state.parent.clone(), state.matrix)
        });
        let parent_world = match parent.as_ref() {
            Some(parent) => self.TransformWorld(parent),
            None => TransformMatrix::default(),
        };
        let world = MultiplyTransforms(&parent_world, &matrix);
        self.transform_world_memo.insert(key, (Arc::clone(node), world));
        world
    }
}

pub fn FlatToWorldSpaceOffset(
    flat: &[TransformMatrix],
    property_world: &TransformMatrix,
) -> Option<Offset> {
    let mut flat_world = TransformMatrix::default();
    for matrix in flat {
        flat_world = MultiplyTransforms(&flat_world, matrix);
    }
    let f = &flat_world.values;
    let p = &property_world.values;
    if !f.iter().chain(p.iter()).all(|value| value.is_finite())
        || (0..16).any(|index| index != 12 && index != 13 && f[index] != p[index])
        || [2, 3, 6, 7, 8, 9, 11, 14]
            .iter()
            .any(|&index| p[index] != 0.0)
        || p[10] != 1.0
        || p[15] != 1.0
    {
        return None;
    }
    let determinant = p[0] * p[5] - p[4] * p[1];
    if !determinant.is_finite() || determinant.abs() < 1e-12 {
        return None;
    }
    let dx = f[12] - p[12];
    let dy = f[13] - p[13];
    let offset = Offset {
        x: (p[5] * dx - p[4] * dy) / determinant,
        y: (p[0] * dy - p[1] * dx) / determinant,
    };
    (offset.x.is_finite() && offset.y.is_finite()).then_some(offset)
}
fn PropertySpaceRect(mut rect: PaintRect, delta: Offset) -> PaintRect {
    rect.x += delta.x;
    rect.y += delta.y;
    rect
}
pub fn SnapToPixel(value: f64, scale: f64) -> f64 {
    let scale = if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    };
    (value * scale + 0.5).floor() / scale
}

fn PixelSnappedRect(rect: &PaintRect, scale: f64) -> PaintRect {
    let left = SnapToPixel(rect.x, scale);
    let top = SnapToPixel(rect.y, scale);
    let right = SnapToPixel(rect.x + rect.width, scale);
    let bottom = SnapToPixel(rect.y + rect.height, scale);
    PaintRect {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
    }
}

pub fn EnclosingRectInPropertySpace(rect: PaintRect, delta: Offset) -> PaintRect {
    let local = PaintRect {
        x: rect.x + delta.x,
        y: rect.y + delta.y,
        ..rect
    }
    .enclosing();
    PaintRect {
        x: local.x - delta.x,
        y: local.y - delta.y,
        ..local
    }
}

pub fn PixelSnappedRectInPropertySpace(rect: &PaintRect, delta: Offset, scale: f64) -> PaintRect {
    let mut snapped = PixelSnappedRect(&PropertySpaceRect(*rect, delta), scale);
    snapped.x -= delta.x;
    snapped.y -= delta.y;
    snapped
}

fn EnclosingMaskClipRectInPropertySpace(rect: PaintRect, delta: Offset) -> PaintRect {
    PropertySpaceRect(rect, delta).enclosing()
}

fn Union(a: PaintRect, b: &PaintRect) -> PaintRect {
    let left = a.x.min(b.x);
    let top = a.y.min(b.y);
    let right = (a.x + a.width).max(b.x + b.width);
    let bottom = (a.y + a.height).max(b.y + b.height);
    PaintRect {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
    }
}

fn ResolveReflectionOffset(filter: &PaintFilterOperation, basis: f64) -> f64 {
    filter
        .reflection_offset_calculation
        .as_ref()
        .map(|calculation| {
            calculation.Evaluate(basis as f32, &foundation::EvaluationInput::default()) as f64
        })
        .unwrap_or_else(|| {
            filter.reflection_offset + basis * filter.reflection_offset_percentage / 100.0
        })
}

// Source: paint_property_tree_builder.cc:2894-2926.
fn ReplacedElementAlwaysClipsToContentBox(node: &FragmentPaintData) -> bool {
    node.ViewReplacedContent()
        .is_some_and(|replaced| !replaced.respects_css_overflow)
}

fn NeedsOverflowClipForReplacedContents(node: &FragmentPaintData) -> bool {
    let Some(replaced) = node.ViewReplacedContent() else {
        return false;
    };
    if replaced.is_svg_root {
        return replaced.clips_to_content_box;
    }
    if node.paint.style.border_radii().is_some() {
        return true;
    }
    if replaced.is_image && !replaced.is_media {
        return false;
    }
    if replaced.is_layout_embedded_content && !replaced.is_embedded_object {
        return false;
    }
    true
}

// Source: paint_property_tree_builder.cc:2928-2952. The LocalFrame
// ClipsContent exception remains an explicit unsupported frame-tree input.
fn NeedsOverflowClip(node: &FragmentPaintData) -> bool {
    if !node.paint.HasSource() || !node.ViewEstablishesPaintState() {
        return false;
    }
    if let Some(replaced) = node.ViewReplacedContent() {
        if ReplacedElementAlwaysClipsToContentBox(node) || replaced.clips_to_content_box {
            return NeedsOverflowClipForReplacedContents(node);
        }
    }
    if node.paint.SourceKind() == NodeKind::kSvgViewportContainer {
        return node.paint.SvgViewport().is_some_and(|v| v.clip.is_some());
    }
    if !node.paint.IsLayoutBox() {
        return false;
    }
    if node.paint.IsOverscrollContainer() {
        return true;
    }
    node.paint.ShouldClipOverflowAlongEitherAxis()
}

// Source: paint_property_tree_builder.cc:3015-3034. Border-shape has no
// boundary representation yet and is handled as unsupported rather than
// being inferred from border-radius geometry.
fn NeedsInnerBorderRadiusClip(node: &FragmentPaintData) -> bool {
    if node.ViewReplacedContent().is_some() && ReplacedElementAlwaysClipsToContentBox(node) {
        return false;
    }
    node.paint.style.border_radii().is_some()
        && node.paint.IsLayoutBox()
        && NeedsOverflowClip(node)
        && node.paint.ShouldClipOverflowAlongBothAxes()
}

fn ClipsOverflow(node: &FragmentPaintData) -> bool {
    NeedsOverflowClip(node)
}

fn ValidateFilters(filters: &[PaintFilterOperation]) {
    for filter in filters {
        if !filter.amount.is_finite()
            || !filter.offset.x.is_finite()
            || !filter.offset.y.is_finite()
            || !filter.blur_radius.is_finite()
            || filter.blur_radius < 0.0
            || !filter.color_matrix.iter().all(|value| value.is_finite())
        {
            panic!("filter geometry must be finite and nonnegative");
        }
        match filter.r#type {
            PaintFilterType::kBlur
            | PaintFilterType::kBrightness
            | PaintFilterType::kContrast
            | PaintFilterType::kSaturate => {
                if filter.amount < 0.0 {
                    panic!("filter amount must be nonnegative");
                }
            }
            PaintFilterType::kGrayscale
            | PaintFilterType::kInvert
            | PaintFilterType::kOpacity
            | PaintFilterType::kSepia => {
                if filter.amount < 0.0 || filter.amount > 1.0 {
                    panic!("filter amount must be in [0, 1]");
                }
            }
            PaintFilterType::kReference
            | PaintFilterType::kHueRotate
            | PaintFilterType::kLuminanceToAlpha
            | PaintFilterType::kColorMatrix
            | PaintFilterType::kConvolveMatrix
            | PaintFilterType::kComponentTransfer
            | PaintFilterType::kTurbulence
            | PaintFilterType::kDropShadow
            | PaintFilterType::kBoxReflect => {}
        }
    }
}
fn ResolveFilterImages(
    node: &PaintTreeNode<'_>,
    filters: &[PaintFilterOperation],
) -> Vec<PaintFilterOperation> {
    let fragment = node
        .fragment
        .as_deref()
        .expect("filter paint tree node has a fragment");
    let mut output = filters.to_vec();
    for filter in &mut output {
        if filter.r#type == PaintFilterType::kBoxReflect {
            // BoxReflectionUtils builds a fragment-local matrix from the
            // pixel-snapped stitched size. The first-fragment paint offset
            // affects rounding of the size, but is not part of the matrix.
            let reference = NinePieceImageGeometry::ReflectionReferenceRect(node);
            let stitched_origin =
                fragment
                    .ViewStitchedDecoration()
                    .as_ref()
                    .map_or(node.paint_offset, |stitched| Offset {
                        x: node.paint_offset.x + stitched.fragment_origin.x
                            - stitched.fragment_offset.x,
                        y: node.paint_offset.y + stitched.fragment_origin.y
                            - stitched.fragment_offset.y,
                    });
            // PhysicalRect::PixelSnappedSize rounds LayoutUnit coordinates;
            // device scale belongs to the later compositor transform.
            let snap_size = |size: f64, origin: f64| {
                let result = (origin.fract() + size).round() - origin.fract().round();
                if result == 0.0 && size.abs() > 4.0 / 64.0 {
                    size.signum()
                } else {
                    result
                }
            };
            let snapped_width = snap_size(reference.width, stitched_origin.x);
            let snapped_height = snap_size(reference.height, stitched_origin.y);
            let (basis, edge) = match filter.reflection_direction {
                PaintReflectionDirection::kAbove => (snapped_height, 0.0),
                PaintReflectionDirection::kBelow => (snapped_height, 2.0 * snapped_height),
                PaintReflectionDirection::kLeft => (snapped_width, 0.0),
                PaintReflectionDirection::kRight => (snapped_width, 2.0 * snapped_width),
            };
            let gap = ResolveReflectionOffset(filter, basis);
            filter.reflection_offset = match filter.reflection_direction {
                PaintReflectionDirection::kAbove | PaintReflectionDirection::kLeft => edge - gap,
                PaintReflectionDirection::kBelow | PaintReflectionDirection::kRight => edge + gap,
            };
            filter.reflection_offset_percentage = 0.0;
            filter.reflection_offset_calculation = None;
            if let Some(image) = filter.reflection_mask_image.take() {
                filter.reflection_mask_layers =
                    NinePieceImageGeometry::ResolveReflectionMask(node, &image);
            }
            continue;
        }
        let Some(graph) = filter.reference_graph.as_mut() else {
            continue;
        };
        let graph = Arc::make_mut(graph);
        for primitive in &mut graph.primitives {
            let SvgFilterPrimitiveKind::kImage {
                resource_id,
                width,
                height,
                orientation,
                pixels,
                ..
            } = &mut primitive.kind
            else {
                continue;
            };
            let Some(id) = *resource_id else { continue };
            let Some(resources) = node.DocumentResources() else {
                graph
                    .unsupported
                    .get_or_insert("feImage bitmap has no PaintResources".into());
                continue;
            };
            let Some(image) = resources.images.iter().find(|image| image.id == id) else {
                graph
                    .unsupported
                    .get_or_insert("feImage bitmap resource is missing".into());
                continue;
            };
            if let Some(bitmap) = image.BitmapPixels() {
                *width = image.width;
                *height = image.height;
                *orientation = image.orientation;
                *pixels = Arc::clone(bitmap)
            } else {
                graph.unsupported.get_or_insert(
                    "feImage document image recording is not raster-resolved".into(),
                );
            }
        }
    }
    // Keep masked reflection explicit. Raster consumes the resolved private
    // record layers; a pending mask stays an empty/transparent reflected
    // branch and can never become the visually much stronger unmasked form.
    output
}

fn ResolveClipLength(value: ClipPathLength, reference: f64, pixel_scale: f64) -> f64 {
    let resolved = value.pixels * pixel_scale + value.percentage * reference / 100.0;
    if !resolved.is_finite() {
        panic!("clip-path geometry must be finite");
    }
    resolved
}

fn ResolveShapeRadius(
    radius: ClipPathShapeRadius,
    center: Offset,
    reference_box: Size,
    axis: Option<bool>,
    pixel_scale: f64,
) -> f64 {
    use ClipPathRadiusKind::*;
    let dx0 = center.x;
    let dx1 = reference_box.width - center.x;
    let dy0 = center.y;
    let dy1 = reference_box.height - center.y;
    let closest_corner = dx0.min(dx1).hypot(dy0.min(dy1));
    let farthest_corner = dx0.max(dx1).hypot(dy0.max(dy1));
    let result = match radius.kind {
        kValue => ResolveClipLength(
            radius.value,
            match axis {
                Some(true) => reference_box.width,
                Some(false) => reference_box.height,
                None => reference_box.width.hypot(reference_box.height) / 2.0_f64.sqrt(),
            },
            pixel_scale,
        ),
        kClosestCorner => closest_corner,
        kFarthestCorner => farthest_corner,
        kClosestSide => match axis {
            Some(true) => center.x.abs().min((reference_box.width - center.x).abs()),
            Some(false) => center.y.abs().min((reference_box.height - center.y).abs()),
            None => center
                .x
                .abs()
                .min((reference_box.width - center.x).abs())
                .min(center.y.abs())
                .min((reference_box.height - center.y).abs()),
        },
        kFarthestSide => match axis {
            Some(true) => center.x.max((reference_box.width - center.x).abs()),
            Some(false) => center.y.max((reference_box.height - center.y).abs()),
            None => center
                .x
                .max((reference_box.width - center.x).abs())
                .max(center.y)
                .max((reference_box.height - center.y).abs()),
        },
    };
    result.max(0.0)
}

fn EllipsePath(
    origin: Offset,
    center: Offset,
    radius_x: f64,
    radius_y: f64,
) -> (PaintRect, Vec<PaintPathCommand>) {
    let rect = PaintRect {
        x: origin.x + center.x - radius_x,
        y: origin.y + center.y - radius_y,
        width: radius_x * 2.0,
        height: radius_y * 2.0,
    };
    let radii = PaintCornerRadii {
        top_left: paint_input_types::paint_input::PaintCornerRadius {
            x: radius_x,
            y: radius_y,
        },
        top_right: paint_input_types::paint_input::PaintCornerRadius {
            x: radius_x,
            y: radius_y,
        },
        bottom_right: paint_input_types::paint_input::PaintCornerRadius {
            x: radius_x,
            y: radius_y,
        },
        bottom_left: paint_input_types::paint_input::PaintCornerRadius {
            x: radius_x,
            y: radius_y,
        },
    };
    let path = ConicRoundedRectPath(&rect, &radii);
    (rect, path)
}

fn RoundedPolygonPath(points: &[Offset], requested_radius: f64) -> Vec<PaintPathCommand> {
    const MIN_ROUNDING_THRESHOLD: f64 = 0.5;
    let mut output = Vec::new();
    if points.is_empty() {
        return output;
    }
    fn line_to(output: &mut Vec<PaintPathCommand>, point: Offset) {
        output.push(PaintPathCommand {
            verb: PaintPathVerb::kLineTo,
            point,
            ..PaintPathCommand::default()
        });
    }
    if requested_radius <= MIN_ROUNDING_THRESHOLD || points.len() < 3 {
        output.push(PaintPathCommand {
            point: points[0],
            ..PaintPathCommand::default()
        });
        for &point in &points[1..] {
            line_to(&mut output, point);
        }
        output.push(PaintPathCommand {
            verb: PaintPathVerb::kClose,
            ..PaintPathCommand::default()
        });
        return output;
    }
    #[derive(Clone, Copy)]
    struct Edge {
        unit: Offset,
        length: f64,
        valid: bool,
    }
    let edge = |from: Offset, to: Offset| {
        let dx = to.x - from.x;
        let dy = to.y - from.y;
        let length = dx.hypot(dy);
        Edge {
            unit: if length > MIN_ROUNDING_THRESHOLD {
                Offset {
                    x: dx / length,
                    y: dy / length,
                }
            } else {
                Offset::default()
            },
            length,
            valid: length > MIN_ROUNDING_THRESHOLD,
        }
    };
    output.push(PaintPathCommand {
        point: Offset {
            x: 0.5 * (points[points.len() - 1].x + points[0].x),
            y: 0.5 * (points[points.len() - 1].y + points[0].y),
        },
        ..PaintPathCommand::default()
    });
    let mut incoming = edge(points[points.len() - 1], points[0]);
    for index in 0..points.len() {
        let current = points[index];
        let next = points[(index + 1) % points.len()];
        let outgoing = edge(current, next);
        let cos_interior = (-(incoming.unit.x * outgoing.unit.x
            + incoming.unit.y * outgoing.unit.y))
            .clamp(-1.0, 1.0);
        let interior = cos_interior.acos();
        let tan_half = (interior * 0.5).tan();
        let radius = if !incoming.valid
            || !outgoing.valid
            || interior <= f64::EPSILON
            || (interior - std::f64::consts::PI).abs() <= f64::EPSILON
            || !tan_half.is_finite()
            || tan_half <= f64::EPSILON
        {
            0.0
        } else {
            requested_radius
                .min(tan_half * incoming.length * 0.5)
                .min(tan_half * outgoing.length * 0.5)
        };
        if radius > f64::EPSILON {
            let tangent_distance = radius / tan_half;
            line_to(
                &mut output,
                Offset {
                    x: current.x - incoming.unit.x * tangent_distance,
                    y: current.y - incoming.unit.y * tangent_distance,
                },
            );
            output.push(PaintPathCommand {
                verb: PaintPathVerb::kConicTo,
                control1: current,
                point: Offset {
                    x: current.x + outgoing.unit.x * tangent_distance,
                    y: current.y + outgoing.unit.y * tangent_distance,
                },
                conic_weight: (interior * 0.5).sin(),
                ..PaintPathCommand::default()
            });
        } else {
            line_to(&mut output, current);
        }
        incoming = outgoing;
    }
    output.push(PaintPathCommand {
        verb: PaintPathVerb::kClose,
        ..PaintPathCommand::default()
    });
    output
}

// cpp: core/style/basic_shapes.cc:136-320; style_path.cc:53-64.
// Source: clip_path_clipper.cc:478-488; style_shape.cc:151-177.
fn ResolveComputedShapeClipPath(
    style: &PaintStyle,
    reference: PaintRect,
    svg_effect_child: bool,
    output: &mut Vec<PaintPathCommand>,
) -> Result<(PaintRect, bool), &'static str> {
    use layout_style::style::{
        basic_shapes::ShapeType,
        clip_path_operation::{ClipPathOperation, OperationType},
        shape_clip_path_operation::ShapeClipPathOperation,
        style_shape::StyleShape,
    };
    let source = style
        .Source()
        .ok_or("compiled clip-path requires its resident style owner")?;
    let operation = source
        .ClipPath()
        .and_then(|raw| unsafe { raw.as_ref() })
        .filter(|operation| operation.GetType() == OperationType::kShape)
        .ok_or("compiled clip-path requires its shape operation")?;
    let operation =
        unsafe { &*(operation as *const dyn ClipPathOperation as *const ShapeClipPathOperation) };
    let shape = operation.GetBasicShape();
    if shape.GetType() != ShapeType::kStyleShapeType {
        return Err("compiled shape clip-path has an invalid style owner");
    }
    let shape = unsafe {
        &*(shape as *const dyn layout_style::style::basic_shapes::BasicShape as *const StyleShape)
    };
    let zoom = source.EffectiveZoom().max(f32::MIN_POSITIVE);
    let reference_scale = if svg_effect_child { zoom } else { 1. };
    let rect = foundation::gfx::RectF::new(
        foundation::gfx::PointF::new(
            reference.x as f32 * reference_scale,
            reference.y as f32 * reference_scale,
        ),
        foundation::gfx::SizeF::new(
            reference.width as f32 * reference_scale,
            reference.height as f32 * reference_scale,
        ),
    );
    let mut sink = paint_model::basic_shape_commands::BasicShapeCommands::default();
    paint_model::style_shape_path::BuildStyleShapePath(
        shape,
        rect,
        if svg_effect_child { 1. / zoom } else { 1. },
        &mut sink,
    );
    let even_odd = sink.even_odd;
    let resolved = sink.Finish();
    let commands = paint_model::basic_shape_commands::IntoPaintCommands(&resolved);
    let bounds = resolved.Bounds();
    let bounds = PaintRect {
        x: f64::from(bounds.x()),
        y: f64::from(bounds.y()),
        width: f64::from(bounds.width()),
        height: f64::from(bounds.height()),
    };
    output.extend(commands);
    Ok((bounds, even_odd))
}

fn ResolveClipPath(
    input: &ClipPathPaint,
    origin: Offset,
    reference_box: Size,
    pixel_scale: f64,
    output: &mut Vec<PaintPathCommand>,
) -> PaintRect {
    if let Some(inset) = input.inset {
        let length = ResolveClipLength;
        let top = length(inset.top, reference_box.height, pixel_scale);
        let right = length(inset.right, reference_box.width, pixel_scale);
        let bottom = length(inset.bottom, reference_box.height, pixel_scale);
        let left = length(inset.left, reference_box.width, pixel_scale);
        let rect = PaintRect {
            x: origin.x + left,
            y: origin.y + top,
            width: (reference_box.width - left - right).max(0.0),
            height: (reference_box.height - top - bottom).max(0.0),
        };
        let radius = |value: paint_input_types::paint_input::ClipPathRadius| {
            paint_input_types::paint_input::PaintCornerRadius {
                x: length(value.x, reference_box.width, pixel_scale).max(0.0),
                y: length(value.y, reference_box.height, pixel_scale).max(0.0),
            }
        };
        let mut radii = PaintCornerRadii {
            top_left: radius(inset.top_left_radius),
            top_right: radius(inset.top_right_radius),
            bottom_right: radius(inset.bottom_right_radius),
            bottom_left: radius(inset.bottom_left_radius),
        };
        // FloatRoundedRect::ConstrainRadii: use one common scale so adjacent
        // corner radii never exceed the inset rectangle's corresponding side.
        let mut scale = 1.0_f64;
        for (available, sum) in [
            (rect.width, radii.top_left.x + radii.top_right.x),
            (rect.width, radii.bottom_left.x + radii.bottom_right.x),
            (rect.height, radii.top_left.y + radii.bottom_left.y),
            (rect.height, radii.top_right.y + radii.bottom_right.y),
        ] {
            if sum > 0.0 {
                scale = scale.min(available / sum);
            }
        }
        if scale < 1.0 {
            for corner in [
                &mut radii.top_left,
                &mut radii.top_right,
                &mut radii.bottom_right,
                &mut radii.bottom_left,
            ] {
                corner.x *= scale;
                corner.y *= scale;
            }
        }
        output.extend(ConicRoundedRectPath(&rect, &radii));
        return rect;
    }
    if let Some(circle) = input.circle {
        let center = Offset {
            x: ResolveClipLength(circle.center_x, reference_box.width, pixel_scale),
            y: ResolveClipLength(circle.center_y, reference_box.height, pixel_scale),
        };
        let radius = ResolveShapeRadius(circle.radius, center, reference_box, None, pixel_scale);
        let (rect, path) = EllipsePath(origin, center, radius, radius);
        output.extend(path);
        return rect;
    }
    if let Some(ellipse) = input.ellipse {
        let center = Offset {
            x: ResolveClipLength(ellipse.center_x, reference_box.width, pixel_scale),
            y: ResolveClipLength(ellipse.center_y, reference_box.height, pixel_scale),
        };
        let radius_x = ResolveShapeRadius(
            ellipse.radius_x,
            center,
            reference_box,
            Some(true),
            pixel_scale,
        );
        let radius_y = ResolveShapeRadius(
            ellipse.radius_y,
            center,
            reference_box,
            Some(false),
            pixel_scale,
        );
        let (rect, path) = EllipsePath(origin, center, radius_x, radius_y);
        output.extend(path);
        return rect;
    }
    if input.polygon_rounding_radius.is_some() && input.commands.is_empty() {
        return PaintRect {
            x: origin.x,
            y: origin.y,
            width: 0.0,
            height: 0.0,
        };
    }
    if input.commands.is_empty() || input.commands[0].verb != PaintPathVerb::kMoveTo {
        panic!("clip path must start with move-to");
    }
    let mut left = f64::INFINITY;
    let mut top = f64::INFINITY;
    let mut right = f64::NEG_INFINITY;
    let mut bottom = f64::NEG_INFINITY;
    fn resolve(
        mut point: Offset,
        origin: Offset,
        left: &mut f64,
        top: &mut f64,
        right: &mut f64,
        bottom: &mut f64,
    ) -> Offset {
        if !point.x.is_finite() || !point.y.is_finite() {
            panic!("clip path coordinates must be finite");
        }
        point.x += origin.x;
        point.y += origin.y;
        *left = (*left).min(point.x);
        *top = (*top).min(point.y);
        *right = (*right).max(point.x);
        *bottom = (*bottom).max(point.y);
        point
    }
    output.reserve(input.commands.len());
    let mut has_geometry = false;
    if !input.percentage_commands.is_empty()
        && input.percentage_commands.len() != input.commands.len()
    {
        panic!("clip path percentage geometry is invalid");
    }
    for (index, source) in input.commands.iter().enumerate() {
        let mut command = *source;
        let percentage = if input.percentage_commands.is_empty() {
            PaintPathCommand {
                verb: command.verb,
                ..Default::default()
            }
        } else {
            input.percentage_commands[index]
        };
        let with_percentage = |pixels: Offset, fractions: Offset| Offset {
            x: pixels.x * input.path_zoom * pixel_scale + fractions.x * reference_box.width,
            y: pixels.y * input.path_zoom * pixel_scale + fractions.y * reference_box.height,
        };
        match command.verb {
            PaintPathVerb::kMoveTo | PaintPathVerb::kLineTo => {
                command.point = resolve(
                    with_percentage(command.point, percentage.point),
                    origin,
                    &mut left,
                    &mut top,
                    &mut right,
                    &mut bottom,
                );
                has_geometry |= command.verb == PaintPathVerb::kLineTo;
            }
            PaintPathVerb::kQuadraticTo | PaintPathVerb::kConicTo => {
                command.control1 = resolve(
                    with_percentage(command.control1, percentage.control1),
                    origin,
                    &mut left,
                    &mut top,
                    &mut right,
                    &mut bottom,
                );
                command.point = resolve(
                    with_percentage(command.point, percentage.point),
                    origin,
                    &mut left,
                    &mut top,
                    &mut right,
                    &mut bottom,
                );
                has_geometry = true;
            }
            PaintPathVerb::kCubicTo => {
                command.control1 = resolve(
                    with_percentage(command.control1, percentage.control1),
                    origin,
                    &mut left,
                    &mut top,
                    &mut right,
                    &mut bottom,
                );
                command.control2 = resolve(
                    with_percentage(command.control2, percentage.control2),
                    origin,
                    &mut left,
                    &mut top,
                    &mut right,
                    &mut bottom,
                );
                command.point = resolve(
                    with_percentage(command.point, percentage.point),
                    origin,
                    &mut left,
                    &mut top,
                    &mut right,
                    &mut bottom,
                );
                has_geometry = true;
            }
            PaintPathVerb::kClose => {
                has_geometry = true;
            }
        }
        output.push(command);
    }
    if !has_geometry || !left.is_finite() || !top.is_finite() {
        panic!("clip path has no drawable geometry");
    }
    let bounds = PaintRect {
        x: left,
        y: top,
        width: (right - left).max(0.0),
        height: (bottom - top).max(0.0),
    };
    if let Some(radius) = input.polygon_rounding_radius {
        let points: Vec<_> = output
            .iter()
            .filter(|command| command.verb != PaintPathVerb::kClose)
            .map(|command| command.point)
            .collect();
        output.clear();
        output.extend(RoundedPolygonPath(
            &points,
            ResolveClipLength(radius, 0.0, pixel_scale),
        ));
    }
    bounds
}

fn TransformClipPoint(
    point: Offset,
    transform: paint_input_types::paint_input::SvgClipTransform,
) -> Offset {
    let [a, b, c, d, e, f] = transform.values;
    Offset {
        x: a * point.x + c * point.y + e,
        y: b * point.x + d * point.y + f,
    }
}
fn MultiplyClipTransform(
    a: paint_input_types::paint_input::SvgClipTransform,
    b: paint_input_types::paint_input::SvgClipTransform,
) -> paint_input_types::paint_input::SvgClipTransform {
    let [a0, a1, a2, a3, a4, a5] = a.values;
    let [b0, b1, b2, b3, b4, b5] = b.values;
    paint_input_types::paint_input::SvgClipTransform {
        values: [
            a0 * b0 + a2 * b1,
            a1 * b0 + a3 * b1,
            a0 * b2 + a2 * b3,
            a1 * b2 + a3 * b3,
            a0 * b4 + a2 * b5 + a4,
            a1 * b4 + a3 * b5 + a5,
        ],
        percentages: [
            a0 * b.percentages[0] + a2 * b.percentages[2] + a.percentages[0],
            a0 * b.percentages[1] + a2 * b.percentages[3] + a.percentages[1],
            a1 * b.percentages[0] + a3 * b.percentages[2] + a.percentages[2],
            a1 * b.percentages[1] + a3 * b.percentages[3] + a.percentages[3],
        ],
    }
}
fn ResolveClipTransformPercentages(
    mut transform: paint_input_types::paint_input::SvgClipTransform,
    width: f64,
    height: f64,
) -> paint_input_types::paint_input::SvgClipTransform {
    transform.values[4] += transform.percentages[0] * width + transform.percentages[1] * height;
    transform.values[5] += transform.percentages[2] * width + transform.percentages[3] * height;
    transform.percentages = [0.0; 4];
    transform
}
fn ResolveSvgClipGeometry(
    geometry: &paint_input_types::paint_input::SvgClipGeometry,
    width: f64,
    height: f64,
) -> Vec<PaintPathCommand> {
    use paint_input_types::paint_input::{SvgClipGeometry as G, SvgClipLength};
    let x = |v: SvgClipLength| v.value + v.percentage * width;
    let y = |v: SvgClipLength| v.value + v.percentage * height;
    let diagonal = ((width * width + height * height) * 0.5).sqrt();
    let r = |v: SvgClipLength| v.value + v.percentage * diagonal;
    let ellipse_path = |cx: f64, cy: f64, rx: f64, ry: f64| {
        if rx <= 0.0 || ry <= 0.0 {
            return Vec::new();
        }
        let w = std::f64::consts::FRAC_1_SQRT_2;
        let mut out = vec![PaintPathCommand {
            verb: PaintPathVerb::kMoveTo,
            point: Offset { x: cx + rx, y: cy },
            ..Default::default()
        }];
        for (control1, point) in [
            (
                Offset {
                    x: cx + rx,
                    y: cy + ry,
                },
                Offset { x: cx, y: cy + ry },
            ),
            (
                Offset {
                    x: cx - rx,
                    y: cy + ry,
                },
                Offset { x: cx - rx, y: cy },
            ),
            (
                Offset {
                    x: cx - rx,
                    y: cy - ry,
                },
                Offset { x: cx, y: cy - ry },
            ),
            (
                Offset {
                    x: cx + rx,
                    y: cy - ry,
                },
                Offset { x: cx + rx, y: cy },
            ),
        ] {
            out.push(PaintPathCommand {
                verb: PaintPathVerb::kConicTo,
                control1,
                point,
                conic_weight: w,
                ..Default::default()
            });
        }
        out.push(PaintPathCommand {
            verb: PaintPathVerb::kClose,
            ..Default::default()
        });
        out
    };
    let poly = |points: Vec<Offset>, close: bool| {
        let mut out = Vec::new();
        if let Some(first) = points.first() {
            out.push(PaintPathCommand {
                verb: PaintPathVerb::kMoveTo,
                point: *first,
                ..Default::default()
            });
            out.extend(points.iter().skip(1).map(|point| PaintPathCommand {
                verb: PaintPathVerb::kLineTo,
                point: *point,
                ..Default::default()
            }));
            if close {
                out.push(PaintPathCommand {
                    verb: PaintPathVerb::kClose,
                    ..Default::default()
                });
            }
        }
        out
    };
    match geometry {
        G::kCircle { cx, cy, r: radius } => ellipse_path(x(*cx), y(*cy), r(*radius), r(*radius)),
        G::kEllipse { cx, cy, rx, ry } => ellipse_path(x(*cx), y(*cy), x(*rx), y(*ry)),
        G::kRect {
            x: left,
            y: top,
            width: w,
            height: h,
            rx,
            ry,
        } => {
            let rect = PaintRect {
                x: x(*left),
                y: y(*top),
                width: x(*w).max(0.0),
                height: y(*h).max(0.0),
            };
            let mut radius_x = rx
                .map(x)
                .or_else(|| ry.map(y))
                .unwrap_or(0.0)
                .max(0.0)
                .min(rect.width * 0.5);
            let mut radius_y = ry
                .map(y)
                .or_else(|| rx.map(x))
                .unwrap_or(0.0)
                .max(0.0)
                .min(rect.height * 0.5);
            if radius_x == 0.0 || radius_y == 0.0 {
                radius_x = 0.0;
                radius_y = 0.0;
            }
            let radius = PaintCornerRadius {
                x: radius_x,
                y: radius_y,
            };
            ConicRoundedRectPath(
                &rect,
                &PaintCornerRadii {
                    top_left: radius,
                    top_right: radius,
                    bottom_right: radius,
                    bottom_left: radius,
                },
            )
        }
        G::kPolygon { points, close } => poly(
            points
                .iter()
                .map(|(px, py)| Offset {
                    x: x(*px),
                    y: y(*py),
                })
                .collect(),
            *close,
        ),
        G::kLine { x1, y1, x2, y2 } => poly(
            vec![
                Offset {
                    x: x(*x1),
                    y: y(*y1),
                },
                Offset {
                    x: x(*x2),
                    y: y(*y2),
                },
            ],
            false,
        ),
    }
}
fn ResolveSvgClipViewport(
    viewport: paint_input_types::paint_input::SvgClipViewport,
    reference_width: f64,
    reference_height: f64,
) -> paint_input_types::paint_input::SvgClipTransform {
    use paint_input_types::paint_input::{SvgClipPreserveAspectRatio as P, SvgClipTransform};
    let resolve = |length: paint_input_types::paint_input::SvgClipLength, basis| {
        length.value + length.percentage * basis
    };
    let width = resolve(viewport.width, reference_width).max(0.0);
    let height = resolve(viewport.height, reference_height).max(0.0);
    let [x, y, view_width, view_height] = viewport.view_box;
    if width == 0.0 || height == 0.0 || view_width <= 0.0 || view_height <= 0.0 {
        return SvgClipTransform {
            values: [0.0; 6],
            percentages: [0.0; 4],
        };
    }
    let sx = width / view_width;
    let sy = height / view_height;
    let (sx, sy, tx, ty) = match viewport.preserve {
        P::kNone => (sx, sy, -x * sx, -y * sy),
        P::kMeet { align_x, align_y } | P::kSlice { align_x, align_y } => {
            let scale = if matches!(viewport.preserve, P::kSlice { .. }) {
                sx.max(sy)
            } else {
                sx.min(sy)
            };
            (
                scale,
                scale,
                (width - view_width * scale) * align_x - x * scale,
                (height - view_height * scale) * align_y - y * scale,
            )
        }
    };
    SvgClipTransform {
        values: [sx, 0.0, 0.0, sy, tx, ty],
        percentages: [0.0; 4],
    }
}

pub fn ResolveSvgClipText(
    text: &paint_input_types::paint_input::SvgClipText,
    fonts: &[paint_input_types::layout_input::FontFace],
    geometry_size: (f64, f64),
    cursor: &mut Offset,
) -> Result<Vec<Vec<PaintPathCommand>>, String> {
    use font_engine::{ClipTextFace, ClipTextRequest, OutlineVerb, TextDirection};
    use paint_input_types::paint_input::SvgClipTextAnchor;
    let variation_storage = fonts
        .iter()
        .map(|font| {
            font.variations
                .iter()
                .map(|value| (value.tag, value.value))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let range_storage = fonts
        .iter()
        .map(|font| {
            font.unicode_ranges
                .iter()
                .map(|range| (range.start, range.end))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let faces = fonts
        .iter()
        .enumerate()
        .map(|(index, font)| ClipTextFace {
            bytes: &font.bytes,
            face_index: font.face_index,
            family: &font.family,
            weight: font.weight,
            italic: font.italic,
            variations: &variation_storage[index],
            unicode_ranges: &range_storage[index],
        })
        .collect::<Vec<_>>();
    let glyphs = font_engine::ShapeClipText(
        &ClipTextRequest {
            text: &text.text,
            families: &text.font_families,
            size: text.font_size,
            weight: text.font_weight,
            italic: text.italic,
            direction: if text.rtl {
                TextDirection::kRtl
            } else {
                TextDirection::kLtr
            },
            language: &text.language,
        },
        &faces,
    )?;
    let resolve = |values: &[paint_input_types::paint_input::SvgClipLength],
                   index: usize,
                   basis: f64| {
        values
            .get(index)
            .or_else(|| values.last())
            .map(|value| value.value + value.percentage * basis)
    };
    if let Some(x) = resolve(&text.x, 0, geometry_size.0) {
        cursor.x = x;
    }
    if let Some(y) = resolve(&text.y, 0, geometry_size.1) {
        cursor.y = y;
    }
    cursor.x += resolve(&text.dx, 0, geometry_size.0).unwrap_or(0.0);
    cursor.y += resolve(&text.dy, 0, geometry_size.1).unwrap_or(0.0);
    let advances = glyphs
        .iter()
        .map(|glyph| {
            let character = text
                .text
                .get(glyph.cluster as usize..)
                .and_then(|tail| tail.chars().next());
            glyph.advance
                + text.letter_spacing
                + if character.is_some_and(char::is_whitespace) {
                    text.word_spacing
                } else {
                    0.0
                }
        })
        .collect::<Vec<_>>();
    let total = advances.iter().sum::<f64>();
    let anchor = match text.anchor {
        SvgClipTextAnchor::kStart => 0.0,
        SvgClipTextAnchor::kMiddle => total * 0.5,
        SvgClipTextAnchor::kEnd => total,
    };
    let mut path_points = text.text_path.as_ref().map(|path| path.points.clone());
    if text
        .text_path
        .as_ref()
        .is_some_and(|path| path.reverse_direction)
    {
        path_points.as_mut().unwrap().reverse();
    }
    let path_lengths = path_points.as_ref().map(|points| {
        let mut lengths = vec![0.0];
        for pair in points.windows(2) {
            lengths.push(
                lengths.last().copied().unwrap()
                    + ((pair[1].x - pair[0].x).powi(2) + (pair[1].y - pair[0].y).powi(2)).sqrt(),
            );
        }
        lengths
    });
    let path_total = path_lengths
        .as_ref()
        .and_then(|values| values.last())
        .copied()
        .unwrap_or(0.0);
    let mut advance_cursor = text.text_path.as_ref().map_or(cursor.x, |path| {
        path.start_offset.value + path.start_offset.percentage * path_total
    }) - anchor;
    let mut result = Vec::new();
    for (index, glyph) in glyphs.into_iter().enumerate() {
        if let Some(x) = resolve(&text.x, index, geometry_size.0) {
            cursor.x = x;
            advance_cursor = x - anchor;
        }
        if let Some(y) = resolve(&text.y, index, geometry_size.1) {
            cursor.y = y;
        }
        if index > 0 {
            advance_cursor += resolve(&text.dx, index, geometry_size.0).unwrap_or(0.0);
            cursor.y += resolve(&text.dy, index, geometry_size.1).unwrap_or(0.0);
        }
        let rotation = text
            .rotate
            .get(index)
            .or_else(|| text.rotate.last())
            .copied()
            .unwrap_or(0.0)
            .to_radians();
        let (mut angle, mut origin_x, mut origin_y) = (
            rotation,
            advance_cursor + glyph.x_offset,
            cursor.y + glyph.y_offset,
        );
        if let (Some(points), Some(lengths)) = (&path_points, &path_lengths) {
            let distance = advance_cursor + advances[index] * 0.5;
            if distance < 0.0 || distance > path_total {
                advance_cursor += advances[index];
                continue;
            }
            let segment = lengths
                .windows(2)
                .position(|pair| distance <= pair[1])
                .unwrap_or(lengths.len().saturating_sub(2));
            let span = (lengths[segment + 1] - lengths[segment]).max(f64::MIN_POSITIVE);
            let t = (distance - lengths[segment]) / span;
            let dx = points[segment + 1].x - points[segment].x;
            let dy = points[segment + 1].y - points[segment].y;
            angle += dy.atan2(dx);
            origin_x = points[segment].x + dx * t - glyph.advance * 0.5 + glyph.x_offset;
            origin_y = points[segment].y + dy * t + cursor.y + glyph.y_offset;
        }
        let (sin, cos) = angle.sin_cos();
        let point = |x: f64, y: f64| Offset {
            x: origin_x + cos * x - sin * y,
            y: origin_y + sin * x + cos * y,
        };
        let commands = glyph
            .commands
            .into_iter()
            .map(|command| {
                let mut paint = PaintPathCommand::default();
                paint.verb = match command.verb {
                    OutlineVerb::MoveTo => PaintPathVerb::kMoveTo,
                    OutlineVerb::LineTo => PaintPathVerb::kLineTo,
                    OutlineVerb::QuadraticTo => PaintPathVerb::kQuadraticTo,
                    OutlineVerb::CubicTo => PaintPathVerb::kCubicTo,
                    OutlineVerb::Close => PaintPathVerb::kClose,
                };
                if paint.verb != PaintPathVerb::kClose {
                    match paint.verb {
                        PaintPathVerb::kQuadraticTo => {
                            paint.control1 = point(command.values[0], command.values[1]);
                            paint.point = point(command.values[2], command.values[3]);
                        }
                        PaintPathVerb::kCubicTo => {
                            paint.control1 = point(command.values[0], command.values[1]);
                            paint.control2 = point(command.values[2], command.values[3]);
                            paint.point = point(command.values[4], command.values[5]);
                        }
                        _ => paint.point = point(command.values[0], command.values[1]),
                    }
                }
                paint
            })
            .collect::<Vec<_>>();
        if !commands.is_empty() {
            result.push(commands);
        }
        advance_cursor += advances[index];
    }
    cursor.x = advance_cursor + anchor;
    Ok(result)
}
pub fn ResolveReferenceClipPath(
    graph: &paint_input_types::paint_input::SvgClipPathGraph,
    reference: PaintRect,
    paint_offset: Offset,
    effective_zoom: f64,
    svg_owner: bool,
    resource_viewport: Option<Size>,
    fonts: &[paint_input_types::layout_input::FontFace],
    runtime_error: &mut Option<String>,
    output: &mut Vec<PaintPathCommand>,
    children: &mut Vec<PaintClipPathChild>,
) -> PaintRect {
    use paint_input_types::paint_input::{SvgClipPathUnits, SvgClipTransform, SvgClipTransformStep};
    let first_output_child = children.len();
    if graph.children.iter().any(|child| child.text.is_some()) {
        runtime_error
            .get_or_insert_with(|| "SVG text clip-path requires mask-image fallback".into());
    }
    let unit = match graph.units {
        SvgClipPathUnits::kObjectBoundingBox => SvgClipTransform {
            values: [
                reference.width,
                0.0,
                0.0,
                reference.height,
                reference.x,
                reference.y,
            ],
            percentages: [0.0; 4],
        },
        SvgClipPathUnits::kUserSpaceOnUse => {
            let zoom = if svg_owner { 1.0 } else { effective_zoom };
            SvgClipTransform {
                values: [zoom, 0.0, 0.0, zoom, paint_offset.x, paint_offset.y],
                percentages: [0.0; 4],
            }
        }
    };
    let geometry_size = match graph.units {
        SvgClipPathUnits::kObjectBoundingBox => (1.0, 1.0),
        SvgClipPathUnits::kUserSpaceOnUse => {
            let zoom = if svg_owner {
                1.0
            } else {
                effective_zoom.max(f64::MIN_POSITIVE)
            };
            resource_viewport.map_or(
                (reference.width / zoom, reference.height / zoom),
                |viewport| (viewport.width, viewport.height),
            )
        }
    };
    let resource_transform =
        ResolveClipTransformPercentages(graph.transform, geometry_size.0, geometry_size.1);
    // LayoutSVGResourceClipper::CalculateClipTransform starts with the
    // resource transform, then applies the object-bounding-box translation
    // and scale. User-space clips instead enter the consumer paint space.
    let resource = match graph.units {
        SvgClipPathUnits::kObjectBoundingBox => MultiplyClipTransform(resource_transform, unit),
        SvgClipPathUnits::kUserSpaceOnUse => MultiplyClipTransform(unit, resource_transform),
    };
    let mut bounds: Option<(f64, f64, f64, f64)> = None;
    let mut text_cursor = Offset::default();
    for child in &graph.children {
        let mut transform = resource;
        for step in &child.transforms {
            let step = match step {
                SvgClipTransformStep::kMatrix(matrix) => {
                    ResolveClipTransformPercentages(*matrix, geometry_size.0, geometry_size.1)
                }
                SvgClipTransformStep::kViewport(viewport) => {
                    ResolveSvgClipViewport(*viewport, geometry_size.0, geometry_size.1)
                }
            };
            transform = MultiplyClipTransform(transform, step);
        }
        let geometry_commands;
        let text_commands;
        let command_groups: Vec<&[PaintPathCommand]> = if let Some(text) = child.text.as_ref() {
            match ResolveSvgClipText(text, fonts, geometry_size, &mut text_cursor) {
                Ok(commands) => {
                    text_commands = commands;
                    text_commands.iter().map(Vec::as_slice).collect()
                }
                Err(error) => {
                    runtime_error.get_or_insert(error);
                    Vec::new()
                }
            }
        } else if let Some(geometry) = child.geometry.as_ref() {
            geometry_commands = ResolveSvgClipGeometry(geometry, geometry_size.0, geometry_size.1);
            vec![&geometry_commands]
        } else {
            vec![&child.commands]
        };
        for commands in command_groups {
            let mut resolved_child = Vec::with_capacity(commands.len());
            for command in commands {
                let mut command = command.clone();
                if command.verb != PaintPathVerb::kClose {
                    command.point = TransformClipPoint(command.point, transform);
                    if matches!(
                        command.verb,
                        PaintPathVerb::kQuadraticTo
                            | PaintPathVerb::kConicTo
                            | PaintPathVerb::kCubicTo
                    ) {
                        command.control1 = TransformClipPoint(command.control1, transform)
                    }
                    if command.verb == PaintPathVerb::kCubicTo {
                        command.control2 = TransformClipPoint(command.control2, transform)
                    }
                    let mut points = vec![command.point];
                    if matches!(
                        command.verb,
                        PaintPathVerb::kQuadraticTo
                            | PaintPathVerb::kConicTo
                            | PaintPathVerb::kCubicTo
                    ) {
                        points.push(command.control1);
                    }
                    if command.verb == PaintPathVerb::kCubicTo {
                        points.push(command.control2);
                    }
                    for point in points {
                        if point.x.is_finite() && point.y.is_finite() {
                            bounds = Some(bounds.map_or(
                                (point.x, point.y, point.x, point.y),
                                |(l, t, r, b)| {
                                    (
                                        l.min(point.x),
                                        t.min(point.y),
                                        r.max(point.x),
                                        b.max(point.y),
                                    )
                                },
                            ))
                        }
                    }
                }
                resolved_child.push(command.clone());
                output.push(command)
            }
            if !resolved_child.is_empty() {
                children.push(PaintClipPathChild {
                    commands: resolved_child,
                    even_odd: child.even_odd,
                });
            }
        }
    }
    if children.len().saturating_sub(first_output_child) > 1 {
        // LayoutSVGResourceClipper::AsPath unions multiple shapes with
        // SkPathOps and falls back to a mask when that is not possible. The
        // local compositor has no path-union or clip mask path yet.
        runtime_error
            .get_or_insert_with(|| "SVG clip-path union/mask fallback is unsupported".into());
    }
    bounds.map_or(
        PaintRect {
            x: reference.x,
            y: reference.y,
            width: 0.0,
            height: 0.0,
        },
        |(l, t, r, b)| PaintRect {
            x: l,
            y: t,
            width: r - l,
            height: b - t,
        },
    )
}

fn IsSvgEffectChild(kind: NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::kSvgGroup
            | NodeKind::kSvgResourceMarker
            | NodeKind::kSvgViewportContainer
            | NodeKind::kSvgText
            | NodeKind::kSvgInline
            | NodeKind::kSvgTSpan
            | NodeKind::kSvgTextPath
            | NodeKind::kSvgShape
    )
}

/// Resolve GeometryBoxUtils/SVGResources' reference-box choice where final
/// fragment geometry is available. The returned origin and size are fed into
/// the unchanged typed BasicShape resolver below.
fn ClipPathReferenceRect(
    fragment: &FragmentPaintData,
    paint_offset: Offset,
    reference_box: ClipPathReferenceBox,
) -> PaintRect {
    let paint = &fragment.paint;
    if IsSvgEffectChild(paint.SourceKind()) {
        let fill = paint.SvgShape().map_or(
            PaintRect {
                x: paint_offset.x,
                y: paint_offset.y,
                width: fragment.ViewSize().width,
                height: fragment.ViewSize().height,
            },
            |shape| PaintRect {
                x: paint_offset.x + shape.bounds_offset.x,
                y: paint_offset.y + shape.bounds_offset.y,
                width: shape.bounds_width,
                height: shape.bounds_height,
            },
        );
        return match reference_box {
            // LayoutSVGShape::StrokeBoundingBox owns this geometry. Callers
            // reject stroke-box until that exact Layout result crosses the
            // package boundary; never reconstruct it from stroke width here.
            ClipPathReferenceBox::kStrokeBox => fill,
            ClipPathReferenceBox::kViewBox => paint.SvgViewBox().map_or(
                PaintRect {
                    x: paint_offset.x,
                    y: paint_offset.y,
                    width: fragment.ViewSize().width,
                    height: fragment.ViewSize().height,
                },
                |view| PaintRect {
                    x: paint_offset.x + view.x,
                    y: paint_offset.y + view.y,
                    width: view.width,
                    height: view.height,
                },
            ),
            // CSS layout-box keywords on an SVG child use its object bounding
            // box in Blink's ReferenceBoxForEffects path.
            _ => fill,
        };
    }

    let border = IncludedBorderEdges(fragment);
    let border_box = PaintRect {
        x: paint_offset.x,
        y: paint_offset.y,
        width: fragment.ViewSize().width,
        height: fragment.ViewSize().height,
    };
    match reference_box {
        ClipPathReferenceBox::kPaddingBox => PaintRect {
            x: border_box.x + border.left,
            y: border_box.y + border.top,
            width: (border_box.width - border.left - border.right).max(0.0),
            height: (border_box.height - border.top - border.bottom).max(0.0),
        },
        ClipPathReferenceBox::kContentBox | ClipPathReferenceBox::kFillBox => PaintRect {
            x: border_box.x + border.left + fragment.Padding().left,
            y: border_box.y + border.top + fragment.Padding().top,
            width: fragment.ViewContentSize().width,
            height: fragment.ViewContentSize().height,
        },
        ClipPathReferenceBox::kMarginBox => {
            let margin = paint.Margin();
            PaintRect {
                x: border_box.x - margin.left,
                y: border_box.y - margin.top,
                width: border_box.width + margin.left + margin.right,
                height: border_box.height + margin.top + margin.bottom,
            }
        }
        ClipPathReferenceBox::kHalfBorderBox => PaintRect {
            x: border_box.x + border.left * 0.5,
            y: border_box.y + border.top * 0.5,
            width: (border_box.width - (border.left + border.right) * 0.5).max(0.0),
            height: (border_box.height - (border.top + border.bottom) * 0.5).max(0.0),
        },
        // stroke-box/view-box have no distinct CSS box for an HTML element.
        ClipPathReferenceBox::kBorderBox
        | ClipPathReferenceBox::kStrokeBox
        | ClipPathReferenceBox::kViewBox => border_box,
    }
}

/// Source: ClipPathClipper::RoundedReferenceBox and
/// ContouredBorderGeometry::ContouredBorder. Non-round ContouredRect paths
/// remain explicit until the paint path model carries corner curvature.
fn GeometryBoxClipPathRadii(
    fragment: &FragmentPaintData,
    reference_box: ClipPathReferenceBox,
) -> Result<PaintCornerRadii, &'static str> {
    if IsSvgEffectChild(fragment.paint.SourceKind()) {
        return Ok(PaintCornerRadii::default());
    }
    let style = &fragment.paint.style;
    if style
        .border_corner_curvatures()
        .0
        .iter()
        .any(|curvature| *curvature != 2.0)
        && style.border_radii().is_some()
    {
        return Err("clip-path geometry-box with non-round corner-shape");
    }
    let border = IncludedBorderEdges(fragment);
    let inset = match reference_box {
        ClipPathReferenceBox::kPaddingBox => border,
        ClipPathReferenceBox::kContentBox | ClipPathReferenceBox::kFillBox => Edges {
            top: border.top + fragment.Padding().top,
            right: border.right + fragment.Padding().right,
            bottom: border.bottom + fragment.Padding().bottom,
            left: border.left + fragment.Padding().left,
        },
        ClipPathReferenceBox::kHalfBorderBox => Edges {
            top: border.top * 0.5,
            right: border.right * 0.5,
            bottom: border.bottom * 0.5,
            left: border.left * 0.5,
        },
        _ => Edges::default(),
    };
    let radii = ResolveCornerRadii(style, &fragment.BorderSides(), fragment.ViewSize(), inset);
    if reference_box == ClipPathReferenceBox::kMarginBox {
        return Ok(OutsetCornerRadiiWithCorrection(
            radii,
            fragment.paint.Margin(),
            fragment.ViewSize(),
        ));
    }
    Ok(radii)
}

fn MaskBox(
    fragment: &FragmentPaintData,
    absolute_offset: Offset,
    box_type: BackgroundBox,
) -> PaintRect {
    let box_type = UsedMaskGeometryBox(box_type);
    let mut rect = PaintRect {
        x: absolute_offset.x,
        y: absolute_offset.y,
        width: fragment.ViewSize().width,
        height: fragment.ViewSize().height,
    };
    if let Some(table) = fragment.ViewTable() {
        rect = PaintRect {
            x: absolute_offset.x + table.grid_offset.x,
            y: absolute_offset.y + table.grid_offset.y,
            width: table.grid_size.width,
            height: table.grid_size.height,
        };
    } else if let Some(fieldset) = fragment.ViewFieldset() {
        // FieldsetPainter::PaintMask contracts the paint rect by the same
        // legend-derived outsets used for its decoration background. Resolve
        // CSS mask-origin/mask-clip against that contracted border box while
        // the immutable fragment geometry is still available.
        rect.x += fieldset.border_outsets.left;
        rect.y += fieldset.border_outsets.top;
        rect.width =
            (rect.width - fieldset.border_outsets.left - fieldset.border_outsets.right).max(0.0);
        rect.height =
            (rect.height - fieldset.border_outsets.top - fieldset.border_outsets.bottom).max(0.0);
    }
    if box_type == BackgroundBox::kBorderBox {
        return rect;
    }
    let border = IncludedBorderEdges(fragment);
    rect = PaintRect {
        x: rect.x + border.left,
        y: rect.y + border.top,
        width: (rect.width - border.left - border.right).max(0.0),
        height: (rect.height - border.top - border.bottom).max(0.0),
    };
    if box_type == BackgroundBox::kPaddingBox {
        return rect;
    }
    PaintRect {
        x: rect.x + fragment.Padding().left,
        y: rect.y + fragment.Padding().top,
        width: fragment.ViewContentSize().width,
        height: fragment.ViewContentSize().height,
    }
}

fn MaskPositioningBox(
    fragment: &FragmentPaintData,
    absolute_offset: Offset,
    box_type: BackgroundBox,
) -> PaintRect {
    let box_type = UsedMaskGeometryBox(box_type);
    let Some(stitched) = fragment.ViewStitchedDecoration() else {
        return MaskBox(fragment, absolute_offset, box_type);
    };
    let mut rect = PaintRect {
        x: absolute_offset.x + stitched.fragment_origin.x - stitched.fragment_offset.x,
        y: absolute_offset.y + stitched.fragment_origin.y - stitched.fragment_offset.y,
        width: stitched.size.width,
        height: stitched.size.height,
    };
    if box_type == BackgroundBox::kBorderBox {
        return rect;
    }
    rect = PaintRect {
        x: rect.x + fragment.Borders().left,
        y: rect.y + fragment.Borders().top,
        width: (rect.width - fragment.Borders().left - fragment.Borders().right).max(0.0),
        height: (rect.height - fragment.Borders().top - fragment.Borders().bottom).max(0.0),
    };
    if box_type == BackgroundBox::kPaddingBox {
        return rect;
    }
    PaintRect {
        x: rect.x + fragment.Padding().left,
        y: rect.y + fragment.Padding().top,
        width: (rect.width - fragment.Padding().left - fragment.Padding().right).max(0.0),
        height: (rect.height - fragment.Padding().top - fragment.Padding().bottom).max(0.0),
    }
}

fn SvgMaskReferenceBox(
    fragment: &FragmentPaintData,
    absolute_offset: Offset,
    box_type: BackgroundBox,
) -> Result<PaintRect, &'static str> {
    let reference_box = match box_type {
        BackgroundBox::kBorderBox
        | BackgroundBox::kPaddingBox
        | BackgroundBox::kContentBox
        | BackgroundBox::kFillBox => ClipPathReferenceBox::kFillBox,
        BackgroundBox::kStrokeBox => ClipPathReferenceBox::kStrokeBox,
        BackgroundBox::kViewBox => ClipPathReferenceBox::kViewBox,
        BackgroundBox::kText
        | BackgroundBox::kNoClip
        | BackgroundBox::kBorderArea
        | BackgroundBox::kBorderAreaText => return Err("mask-origin geometry is invalid for SVG"),
    };
    if matches!(
        reference_box,
        ClipPathReferenceBox::kFillBox | ClipPathReferenceBox::kStrokeBox
    ) && fragment.paint.SvgShape().is_none()
    {
        return Err("SVG container mask reference-box geometry");
    }
    if reference_box == ClipPathReferenceBox::kStrokeBox {
        // LayoutSVGShape::StrokeBoundingBox is path-, cap-, join- and
        // transform-aware. The old PrePaint stroke-width expansion duplicated
        // Layout policy and was not exact for every shape.
        return Err("exact SVG stroke-box geometry");
    }
    Ok(ClipPathReferenceRect(
        fragment,
        absolute_offset,
        reference_box,
    ))
}

fn SvgMaskPaintingArea(
    fragment: &FragmentPaintData,
    absolute_offset: Offset,
    clip: BackgroundBox,
    positioning: PaintRect,
) -> Result<(PaintRect, bool), &'static str> {
    match clip {
        BackgroundBox::kText | BackgroundBox::kNoClip => {
            let visual = fragment
                .ViewInkOverflow()
                .or(fragment.ViewContentsInkOverflow())
                .or(fragment.ViewSelfInkOverflow())
                .ok_or("source SVG visual-overflow geometry")?;
            Ok((
                PaintRect {
                    x: absolute_offset.x + visual.offset.x,
                    y: absolute_offset.y + visual.offset.y,
                    width: visual.size.width,
                    height: visual.size.height,
                },
                false,
            ))
        }
        BackgroundBox::kContentBox | BackgroundBox::kFillBox | BackgroundBox::kPaddingBox => {
            Ok((positioning, true))
        }
        BackgroundBox::kBorderBox
        | BackgroundBox::kStrokeBox
        | BackgroundBox::kBorderArea
        | BackgroundBox::kBorderAreaText => Ok((
            SvgMaskReferenceBox(fragment, absolute_offset, BackgroundBox::kStrokeBox)?,
            true,
        )),
        BackgroundBox::kViewBox => Ok((
            SvgMaskReferenceBox(fragment, absolute_offset, BackgroundBox::kViewBox)?,
            true,
        )),
    }
}

fn MaskClipRadii(fragment: &FragmentPaintData, box_type: BackgroundBox) -> PaintCornerRadii {
    let box_type = UsedMaskGeometryBox(box_type);
    let paint = &fragment.paint;
    let mut inset = Edges::default();
    if box_type != BackgroundBox::kBorderBox {
        inset = IncludedBorderEdges(fragment);
    }
    if box_type == BackgroundBox::kContentBox {
        inset.top += fragment.Padding().top;
        inset.right += fragment.Padding().right;
        inset.bottom += fragment.Padding().bottom;
        inset.left += fragment.Padding().left;
    }
    let outer_size = fragment.ViewTable().map_or_else(
        || {
            fragment
                .ViewFieldset()
                .map_or(fragment.ViewSize(), |fieldset| Size {
                    width: (fragment.ViewSize().width
                        - fieldset.border_outsets.left
                        - fieldset.border_outsets.right)
                        .max(0.0),
                    height: (fragment.ViewSize().height
                        - fieldset.border_outsets.top
                        - fieldset.border_outsets.bottom)
                        .max(0.0),
                })
        },
        |table| table.grid_size,
    );
    ResolveCornerRadii(&paint.style, &fragment.BorderSides(), outer_size, inset)
}

fn UsedMaskGeometryBox(box_type: BackgroundBox) -> BackgroundBox {
    match box_type {
        BackgroundBox::kPaddingBox => BackgroundBox::kPaddingBox,
        BackgroundBox::kContentBox => BackgroundBox::kContentBox,
        BackgroundBox::kBorderBox
        | BackgroundBox::kFillBox
        | BackgroundBox::kStrokeBox
        | BackgroundBox::kViewBox
        | BackgroundBox::kNoClip
        | BackgroundBox::kText
        | BackgroundBox::kBorderArea
        | BackgroundBox::kBorderAreaText => BackgroundBox::kBorderBox,
    }
}

fn FindMaskImage(resources: Option<&PaintResources>, id: u64) -> &PaintImage {
    let Some(resources) = resources else {
        panic!("mask paint has no PaintResources");
    };
    let images = &resources.images;
    let Some(found) = images.iter().find(|image| image.id == id) else {
        panic!("mask image resource id is missing");
    };
    if found.width == 0
        || found.height == 0
        || !found.resolution_scale.is_finite()
        || found.resolution_scale <= 0.0
        || found
            .BitmapPixels()
            .is_some_and(|pixels| pixels.len() != found.width as usize * found.height as usize * 4)
    {
        panic!("mask image resource is invalid");
    }
    found
}

fn ResolveMaskLayers(
    fragment: &FragmentPaintData,
    resources: Option<&PaintResources>,
    absolute_offset: Offset,
    property_space_delta: Offset,
    pixel_snap_scale: f64,
    layers: &[PaintMaskLayer],
) -> Vec<DisplayMaskLayer> {
    let mut output = Vec::with_capacity(layers.len());
    for layer in layers {
        if let Some(reason) = layer.source_error.as_ref() {
            std::panic::panic_any(foundation::UnsupportedLayout::new(format!(
                "unsupported SVG mask resource: {reason}"
            )));
        }
        if let Some(reference) = &layer.svg_reference {
            let object = ClipPathReferenceRect(
                fragment,
                absolute_offset,
                if IsSvgEffectChild(fragment.paint.SourceKind()) {
                    ClipPathReferenceBox::kFillBox
                } else {
                    ClipPathReferenceBox::kBorderBox
                },
            );
            let mut resolved = DisplayMaskLayer {
                clip_rect: object,
                tile_rect: object,
                mode: layer.mode,
                composite: layer.composite,
                source_error: layer.source_error.clone(),
                ..Default::default()
            };
            if let Some(graph) = reference.graph.as_deref() {
                use paint_input_types::paint_input::SvgMaskUnits;
                if IsSvgEffectChild(fragment.paint.SourceKind())
                    && fragment.paint.SvgShape().is_none()
                    && (graph.mask_units == SvgMaskUnits::kObjectBoundingBox
                        || graph.content_units == SvgMaskUnits::kObjectBoundingBox)
                {
                    resolved.source_error =
                        Some("SVG group object bounding box for local mask".into());
                }
                if resolved.source_error.is_none() {
                    if let Some(viewport) = reference.viewport {
                        let zoom = if IsSvgEffectChild(fragment.paint.SourceKind()) {
                            1.0
                        } else {
                            f64::from(fragment.paint.EffectiveZoom())
                        };
                        let (region, shader) = crate::paint_shader_resolver::ResolveLocalSvgMask(
                            graph,
                            object,
                            absolute_offset,
                            zoom,
                            viewport,
                            resources,
                        );
                        resolved.clip_rect = region;
                        resolved.tile_rect = region;
                        resolved.paint_shader = shader;
                    } else {
                        resolved.source_error = Some("missing SVG mask resource viewport".into());
                    }
                }
            }
            if let Some(reason) = resolved.source_error.as_ref() {
                // Validate before culling: a zero-sized SVG group must not turn
                // unsupported resource content into a successful empty mask.
                std::panic::panic_any(foundation::UnsupportedLayout::new(format!(
                    "unsupported SVG mask resource: {reason}"
                )));
            }
            output.push(resolved);
            continue;
        }
        if !layer.image.position.x.is_finite() || !layer.image.position.y.is_finite() {
            panic!("mask position must be finite");
        }
        let svg_effect_child = IsSvgEffectChild(fragment.paint.SourceKind());
        let (positioning, clip, has_clip) = if svg_effect_child {
            let positioning = SvgMaskReferenceBox(fragment, absolute_offset, layer.image.origin)
                .unwrap_or_else(|reason| {
                    std::panic::panic_any(foundation::UnsupportedLayout::new(format!(
                        "unsupported SVG mask image geometry: {reason}"
                    )))
                });
            let (clip, has_clip) =
                SvgMaskPaintingArea(fragment, absolute_offset, layer.image.clip, positioning)
                    .unwrap_or_else(|reason| {
                        std::panic::panic_any(foundation::UnsupportedLayout::new(format!(
                            "unsupported SVG mask image geometry: {reason}"
                        )))
                    });
            (positioning, clip, has_clip)
        } else {
            let positioning = MaskPositioningBox(fragment, absolute_offset, layer.image.origin);
            if layer.image.clip == BackgroundBox::kNoClip {
                let bounds = fragment
                    .paint
                    .CssMaskBoundingBox()
                    .expect("source CSS mask bounding box");
                (
                    positioning,
                    PaintRect {
                        x: absolute_offset.x + bounds.offset.x,
                        y: absolute_offset.y + bounds.offset.y,
                        width: bounds.size.width,
                        height: bounds.size.height,
                    },
                    false,
                )
            } else {
                (
                    positioning,
                    MaskBox(fragment, absolute_offset, layer.image.clip),
                    true,
                )
            }
        };
        // Source: BoxPainterBase::PaintFillLayer clips CSS padding/content
        // mask boxes through ToPixelSnappedRect, while border-box image
        // geometry is likewise painted into BackgroundImageGeometry's
        // SnappedDestRect. Keep the clip in the same property-space pixel
        // phase as the already-snapped tile; an unsnapped clip would apply
        // fractional coverage a second time at the mask's top/left edges.
        let clip = if !svg_effect_child && has_clip {
            PixelSnappedRectInPropertySpace(&clip, property_space_delta, pixel_snap_scale)
        } else {
            clip
        };
        let image =
            if layer.image_state == PaintMaskImageState::kReady && layer.image.shader.is_none() {
                Some(FindMaskImage(resources, layer.image.resource_id))
            } else {
                None
            };
        let image_orientation = image.map_or(image_resource::ImageOrientation::kDefault, |image| {
            if fragment.paint.style.respect_image_orientation() {
                image.orientation
            } else {
                image_resource::ImageOrientation::kDefault
            }
        });
        let tile_size = ResolveBackgroundTileSize(
            &layer.image,
            &positioning,
            image,
            image_orientation,
            f64::from(fragment.paint.EffectiveZoom()),
        );
        let tile_geometry = ResolveBackgroundTile(&layer.image, &positioning, tile_size);
        let tile = if layer.image.shader.is_some()
            || (image.is_some() && !tile_geometry.repeat_x && !tile_geometry.repeat_y)
        {
            // Source: background_image_geometry.cc SetNoRepeatX/Y and
            // box_painter_base.cc DrawTiledBackground. A single bitmap tile
            // paints into SnappedDestRect; keeping the unsnapped mask origin
            // here makes software playback filter the already-scaled image a
            // second time at the layout-unit phase.
            PixelSnappedRectInPropertySpace(
                &tile_geometry.tile_rect,
                property_space_delta,
                pixel_snap_scale,
            )
        } else {
            tile_geometry.tile_rect
        };
        let bitmap_tile_scale = if let Some(image) = image {
            let (width, height) =
                image_orientation.oriented_size(f64::from(image.width), f64::from(image.height));
            Offset {
                x: tile.width / width,
                y: tile.height / height,
            }
        } else {
            tile_geometry.tile_scale
        };
        let mut resolved = DisplayMaskLayer {
            resource_id: image.map_or(0, |image| image.id),
            image_interpolation_quality: fragment.paint.style.image_interpolation_quality(),
            image_orientation,
            document_image_container_key: image.and_then(|image| {
                ResolveDocumentImageContainerKey(
                    image,
                    Size {
                        width: tile.width,
                        height: tile.height,
                    },
                    f64::from(fragment.paint.EffectiveZoom()),
                    resources,
                    (!fragment.paint.style.color_scheme_flags_is_normal())
                        .then_some(u8::from(fragment.paint.style.used_dark_color_scheme())),
                )
            }),
            clip_rect: clip,
            clip_radii: if svg_effect_child {
                PaintCornerRadii::default()
            } else {
                MaskClipRadii(fragment, layer.image.clip)
            },
            has_clip,
            source_rect: image.map_or_else(
                || {
                    if layer.image.shader.is_some() {
                        // Generated images without intrinsic dimensions are
                        // created at BackgroundImageGeometry::TileSize.
                        PaintRect {
                            x: 0.0,
                            y: 0.0,
                            width: tile_size.width,
                            height: tile_size.height,
                        }
                    } else {
                        PaintRect::default()
                    }
                },
                |image| {
                    let (width, height) = image_orientation
                        .oriented_size(f64::from(image.width), f64::from(image.height));
                    PaintRect {
                        x: 0.0,
                        y: 0.0,
                        width,
                        height,
                    }
                },
            ),
            tile_rect: tile,
            repeat_x: tile_geometry.repeat_x,
            repeat_y: tile_geometry.repeat_y,
            repeat_rule_x: tile_geometry.rule_x,
            repeat_rule_y: tile_geometry.rule_y,
            tile_scale: bitmap_tile_scale,
            tile_spacing: tile_geometry.tile_spacing,
            mode: layer.mode,
            composite: layer.composite,
            ..Default::default()
        };
        if let Some(shader) = layer.image.shader.as_deref() {
            resolved.paint_shader = Some(ResolvePaintShader(
                shader,
                resources,
                Offset {
                    x: tile.x,
                    y: tile.y,
                },
                Some(Size {
                    width: tile.width,
                    height: tile.height,
                }),
            ));
            resolved.tile_rect = tile;
        }
        output.push(resolved);
    }
    output
}

// core/paint/paint_property_tree_builder.cc, UpdateEffect: mask resources
// suppress the whole element until every image has finished loading. An error
// is terminal and contributes transparent black through its retained mask layer.
pub fn MaskAdjustedOpacity(style: &PaintStyle) -> f32 {
    let mask_images = style.mask_images();
    // Without mask layers neither suppression condition can hold.
    if mask_images.is_empty() {
        return style.opacity();
    }
    let any_loading = mask_images
        .iter()
        .any(|layer| layer.image_state == PaintMaskImageState::kLoading);
    let all_invalid = !mask_images.is_empty()
        && mask_images
            .iter()
            .all(|layer| layer.image_state != PaintMaskImageState::kReady);
    let has_backdrop = style.has_backdrop_filters();
    if any_loading || (has_backdrop && all_invalid) {
        0.0
    } else {
        style.opacity()
    }
}

/// The effect state `node` gives its logical children when it is a
/// paint-state occurrence of a LayoutObject (ResolveLogicalParentOccurrence's
/// candidates), inside `fragmentainer`.
fn LogicalEffectContextOf(node: &PaintTreeNode<'_>, fragmentainer: u64) -> Option<LogicalEffectContext> {
    let fragment = node.fragment.as_deref()?;
    (fragment.ViewNodeId() != 0
        && fragment.paint.HasSource()
        && fragment.ViewEstablishesPaintState())
    .then(|| LogicalEffectContext {
        fragment_instance_id: fragment.ViewFragmentInstanceId(),
        fragmentainer,
        effect: node.properties.nodes.effect.clone(),
        effect_id: node.properties.effect_id,
        opacity: node.opacity,
    })
}


// LayoutSVGHiddenContainer::Paint is a subtree boundary. Preserve its
// descendants in immutable Layout snapshots for resource consumers, but do
// not create ordinary paint owners, properties, or reparentable paint layers.
// Source PrePaint also walks hidden resource subtrees, with independent
// property context. Ordinary Paint skips their entry; resource Paint borrows it.
fn OrdinaryPaintSourceChildren<'a>(
    fragment: PaintFragmentSource<'a>,
) -> impl Iterator<Item = PaintFragmentSource<'a>> {
    fragment.Children()
}

/// Source: PrePaintTreeWalk::PrePaintTreeWalkContext and the parts of
/// PaintPropertyTreeBuilderContext that cross from a parent's visit to a
/// child's (pre_paint_tree_walk.h:60-140, paint_property_tree_builder.h).
/// A stack value of one Walk; nothing of it is retained.
struct WalkContext {
    /// The tree builder context the object is built with.
    inherited: InheritedContext,
    /// The parent's PaintInvalidatorContext.
    invalidation: PaintInvalidatorContext,
    /// force_subtree_update_reasons with isolation piercing: an ancestor
    /// added or removed one of its property nodes or changed its filter
    /// taint, so no descendant may be skipped. In-place value changes do
    /// not force (see WalkObject).
    force_subtree: bool,
    /// The context the parent hands its children differs from the one its
    /// previous build handed them (ChildContextDigest), or the inherited
    /// context has no previous value to compare with.
    context_changed: bool,
    /// The parent's contents cull rect before this walk rebuilt it: the
    /// previous cull rect of a child without its own stored rects.
    old_parent_contents_cull: Option<Option<PaintRect>>,
    /// The auto dark mode pause an SVG root ancestor established.
    auto_dark_mode_paused: bool,
    /// The parent's SVG viewBox matrix, which the child's flat replay opens
    /// as its leading local transform.
    leading_svg_matrix: Option<TransformMatrix>,
}

#[derive(Clone, Copy, Default)]
struct PhaseNeeds {
    float: bool,
    outline: bool,
    overflow_controls: bool,
}

impl PhaseNeeds {
    fn Merge(&mut self, child: PhaseNeeds) {
        self.float |= child.float;
        self.outline |= child.outline;
        self.overflow_controls |= child.overflow_controls;
    }
}

/// What one occurrence's visit (or skipped subtree) hands its parent.
#[derive(Clone, Copy, Default)]
struct VisitResult {
    /// Upward result of TableCellPaintInvalidator (Chromium mutates the
    /// already visited row/section/table clients through parent contexts).
    table_cell_geometry_changed: bool,
    is_paint_layer: bool,
    /// The subtree's paint phase needs (MarkPaintPhaseNeeds).
    phase: PhaseNeeds,
}

impl VisitResult {
    /// The result of a subtree the walk did not enter, from the values its
    /// root's FragmentData kept.
    fn Stored(data: &PrePaintFragmentData) -> Self {
        let flat = data.FlatReplay();
        Self {
            table_cell_geometry_changed: false,
            is_paint_layer: data.WalkRecord().is_paint_layer,
            phase: PhaseNeeds {
                float: !flat.skip_float_phase,
                outline: !flat.skip_outline_phase,
                overflow_controls: !flat.skip_overflow_controls_phase,
            },
        }
    }
}

// cpp: core/paint/paint_invalidator.cc:33-53
// cpp: core/paint/paint_layer_painter.cc:800-809
// Empty phases need neither a subtree walk nor replay geometry scopes. A
// self-painting child owns its phase flags and must not make the ancestor
// layer visit empty phases, so only non-layer children are aggregated.
fn OwnPhaseNeeds(fragment: &FragmentPaintData, is_paint_layer: bool) -> PhaseNeeds {
    let style = &fragment.paint.style;
    PhaseNeeds {
        float: fragment.paint.HasSource() && fragment.paint.Floating() != FloatSide::kNone,
        outline: !is_paint_layer
            && (fragment.paint.FocusedImageMapArea().is_some()
                || fragment
                    .paint
                    .FormControl()
                    .is_some_and(|control| control.auto_focus_ring)
                || (style.outline_style() != BorderLineStyle::kNone
                    && !(style.outline_width() <= 0.0)
                    && !(style.outline_color().alpha <= 0.0))),
        overflow_controls: fragment
            .ViewScrollbarsPaintAsOverlay()
            .is_some_and(|overlay| overlay && is_paint_layer),
    }
}

/// The values of an occurrence's previous visit the builder and the
/// invalidators compare against, read from its FragmentData before this
/// visit's build replaces them (Chromium reads fragment_data_.PaintOffset()
/// and the existing ObjectPaintProperties nodes the same way).
struct PreviousVisit {
    record: PrePaintWalkRecord,
    /// The exact flat paint offset (PaintInvalidator's old paint offset).
    flat_paint_offset: Offset,
    had_paint_offset_translation: bool,
    self_cull_rect: Option<PaintRect>,
    contents_cull_rect: Option<PaintRect>,
    /// Its own Filter's operations were origin-tainted
    /// (paint_property_tree_builder.cc:2632-2633,2713-2730).
    filter_origin_tainted: bool,
    /// The roles of its ObjectPaintProperties slots.
    roles: Vec<PaintPropertyRole>,
    gap_rules: Option<Arc<layout::paint::GapRulePaintData>>,
}

impl PreviousVisit {
    fn Of(data: &PrePaintFragmentData, parent_contents_cull: Option<Option<PaintRect>>) -> Self {
        let record = *data.WalkRecord();
        let flat = data.FlatReplay();
        let (self_cull_rect, contents_cull_rect) = if record.layer_cull_rects {
            (data.GetCullRect().Rect(), data.GetContentsCullRect().Rect())
        } else {
            match flat.Rare().and_then(|rare| rare.non_layer_cull_rects) {
                Some(rects) => rects,
                None => {
                    let cull = parent_contents_cull.flatten();
                    (cull, cull)
                }
            }
        };
        let properties = data.PaintProperties();
        Self {
            record,
            flat_paint_offset: flat.paint_offset,
            had_paint_offset_translation: properties
                .is_some_and(|properties| properties.PaintOffsetTranslation().is_some()),
            self_cull_rect,
            contents_cull_rect,
            filter_origin_tainted: properties
                .and_then(|properties| properties.Filter())
                .is_some_and(|filter| {
                    paint_model::property_cell::with_access(|access| {
                        filter.state(access).filters.origin_tainted()
                    })
                }),
            roles: properties.map_or_else(Vec::new, |properties| {
                properties
                    .Slots()
                    .filter_map(ObjectPaintPropertySlot::Role)
                    .collect()
            }),
            gap_rules: data.PreviousGapRules().cloned(),
        }
    }
}

/// The (cull rect, contents cull rect) an occurrence's last visit stored, as
/// PreviousVisit reads them: a PaintLayer object's FragmentData rects, or a
/// non-layer object's own flat rects, or else its paint parent's previous
/// contents cull rect.
fn StoredCullRects(
    data: &PrePaintFragmentData,
    parent_contents_cull: Option<Option<PaintRect>>,
) -> (Option<PaintRect>, Option<PaintRect>) {
    if data.WalkRecord().layer_cull_rects {
        (data.GetCullRect().Rect(), data.GetContentsCullRect().Rect())
    } else {
        match data
            .FlatReplay()
            .Rare()
            .and_then(|rare| rare.non_layer_cull_rects)
        {
            Some(rects) => rects,
            None => {
                let cull = parent_contents_cull.flatten();
                (cull, cull)
            }
        }
    }
}

/// The paint parent and the single occurrence of the object `chain[0]`,
/// found from `root` through the fragments of its LayoutObject ancestors
/// `chain[1..]` and the fragments that have no client (line boxes).
/// Chromium reaches the LayoutObject through its pointer; None when the
/// object has no occurrence there or more than one (IsFragmented).
fn FindPendingTransformOccurrence<'a>(
    root: PaintFragmentCursor<'a>,
    chain: &[u64],
) -> Option<(PaintFragmentCursor<'a>, PaintFragmentCursor<'a>)> {
    fn visit<'a>(
        node: PaintFragmentCursor<'a>,
        chain: &[u64],
        found: &mut Option<(PaintFragmentCursor<'a>, PaintFragmentCursor<'a>)>,
        count: &mut usize,
    ) {
        for child in node.Children() {
            if *count > 1 {
                return;
            }
            let Some(fragment) = child.PaintDataOption() else {
                continue;
            };
            let client = fragment.paint.DisplayItemClientId();
            if client == chain[0] {
                *count += 1;
                found.get_or_insert((node, child));
            } else if client == 0 || chain[1..].contains(&client) {
                visit(child, chain, found, count);
            }
        }
    }
    if chain.is_empty() {
        return None;
    }
    let mut found = None;
    let mut count = 0;
    visit(root, chain, &mut found, &mut count);
    if count == 1 { found } else { None }
}

/// Old flat transform list -> its replacement, for one direct transform
/// update. Most updates touch a handful of distinct lists (each object that
/// adds own entries starts a new one), so a short linear memo avoids hashing;
/// a large subtree with many such objects moves to the map. Each entry keeps
/// the old list alive, so its address is not reused while the memo lives.
#[derive(Default)]
struct FlatTransformMemo {
    linear: Vec<(FlatTransforms, FlatTransforms)>,
    // Keyed by aligned addresses: the SipHash map, not the identity one.
    map: std::collections::HashMap<usize, (FlatTransforms, FlatTransforms)>,
}

impl FlatTransformMemo {
    const LINEAR: usize = 16;

    fn Get(&self, list: &FlatTransforms) -> Option<&FlatTransforms> {
        if self.map.is_empty() {
            self.linear
                .iter()
                .find(|(old, _)| Arc::ptr_eq(old, list))
                .map(|(_, new)| new)
        } else {
            self.map
                .get(&(Arc::as_ptr(list).cast::<TransformMatrix>() as usize))
                .map(|(_, new)| new)
        }
    }

    fn Insert(&mut self, old: FlatTransforms, new: FlatTransforms) {
        if self.map.is_empty() && self.linear.len() < Self::LINEAR {
            self.linear.push((old, new));
            return;
        }
        for (linear_old, linear_new) in self.linear.drain(..) {
            self.map.insert(
                Arc::as_ptr(&linear_old).cast::<TransformMatrix>() as usize,
                (linear_old, linear_new),
            );
        }
        self.map
            .insert(Arc::as_ptr(&old).cast::<TransformMatrix>() as usize, (old, new));
    }

    /// Replaces `old` at `index` of `list` with `new`; whether it did.
    fn Replace(
        &mut self,
        list: &mut FlatTransforms,
        index: usize,
        old: &TransformMatrix,
        new: &TransformMatrix,
    ) -> bool {
        if list.get(index) != Some(old) {
            return false;
        }
        if let Some(replaced) = self.Get(list) {
            *list = replaced.clone();
            return true;
        }
        let mut values = list.to_vec();
        values[index] = *new;
        let replaced = FlatTransforms::from(values);
        self.Insert(std::mem::replace(list, replaced.clone()), replaced);
        true
    }
}

// Indices of PrePaintUpdateStats::clean_rejects (crate::PRE_PAINT_CLEAN_REJECT_FIELDS).
const CLEAN_REJECT_FORCED: usize = 0;
const CLEAN_REJECT_OWN_PROPERTY: usize = 1;
const CLEAN_REJECT_OWN_LAYOUT: usize = 2;
const CLEAN_REJECT_WALK_STATE: usize = 3;
const CLEAN_REJECT_RECORD_OWNER: usize = 4;
const CLEAN_REJECT_RECORD_GEOMETRY: usize = 5;
const CLEAN_REJECT_CACHED_CONTEXT: usize = 6;
const CLEAN_REJECT_SUBPIXEL_DELTA: usize = 7;

// Indices of PrePaintUpdateStats::skip_rejects (crate::PRE_PAINT_SKIP_REJECT_FIELDS).
const SKIP_REJECT_FORCED: usize = 0;
const SKIP_REJECT_OWN: usize = 1;
const SKIP_REJECT_DESCENDANTS: usize = 2;
const SKIP_REJECT_WALK_STATE: usize = 3;
const SKIP_REJECT_RECORD_OWNER: usize = 4;
const SKIP_REJECT_RECORD_GEOMETRY: usize = 5;
const SKIP_REJECT_WHEEL_INSIDE: usize = 6;
const SKIP_REJECT_NO_DATA: usize = 7;

/// The PhysicalFragment (or FragmentItem and container) a cursor names, as
/// addresses: the identity a skipped subtree's root must keep (a clean
/// FragmentData keeps naming the fragment the walk visited).
fn WalkedOwner(cursor: PaintFragmentCursor<'_>) -> (usize, usize) {
    use layout::output::FragmentCursor;
    match cursor.Cursor() {
        FragmentCursor::Physical { fragment, .. } => (fragment as *const _ as usize, 0),
        FragmentCursor::Inline {
            item, container, ..
        } => (item as *const _ as usize, container as *const _ as usize),
        #[allow(unreachable_patterns)]
        _ => (0, 0),
    }
}

/// A fingerprint of the context a builder hands its children (ChildContext
/// without the wheel-handler bits, which the skip test reads directly): what
/// PrePaintTreeWalk::NeedsTreeBuilderContextUpdate compares. Property nodes
/// enter by identity, as Chromium's PaintPropertyTreeBuilderFragmentContext
/// names them: a node updated in place keeps it, and the children's
/// FragmentData keeps naming the live node with its new value. The flat
/// values (origin, paint space, clip, cull, transform list, scroll anchor,
/// SVG matrix) are Rechrom's flat accumulation, which the children store in
/// their FlatReplayData and cull rects; they enter by value, so a subtree is
/// revisited only as far as these values actually differ.
fn ChildContextDigest(context: &ChildContext) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    let h = &mut hasher;
    fn offset(h: &mut impl Hasher, value: Offset) {
        value.x.to_bits().hash(h);
        value.y.to_bits().hash(h);
    }
    fn rect(h: &mut impl Hasher, value: Option<PaintRect>) {
        match value {
            None => 0u8.hash(h),
            Some(rect) => {
                1u8.hash(h);
                rect.x.to_bits().hash(h);
                rect.y.to_bits().hash(h);
                rect.width.to_bits().hash(h);
                rect.height.to_bits().hash(h);
            }
        }
    }
    fn matrix(h: &mut impl Hasher, value: &TransformMatrix) {
        for v in value.values {
            v.to_bits().hash(h);
        }
    }
    fn state(h: &mut impl Hasher, value: &PropertyTreeState) {
        value.transform_id.hash(h);
        value.clip_id.hash(h);
        value.effect_id.hash(h);
        value.scroll_id.hash(h);
        (Arc::as_ptr(&value.nodes.transform) as usize).hash(h);
        (Arc::as_ptr(&value.nodes.clip) as usize).hash(h);
        (Arc::as_ptr(&value.nodes.effect) as usize).hash(h);
    }
    offset(h, context.origin);
    offset(h, context.paint_space);
    rect(h, context.clip);
    rect(h, context.cull);
    context.transforms.len().hash(h);
    for value in context.transforms.iter() {
        matrix(h, value);
    }
    // The accumulated opacity is not hashed: no descendant stores a value
    // derived from it (its Effect nodes name the live ancestor effect), as
    // Chromium's context carries no opacity.
    offset(h, context.scroll);
    context.fixed_background_acts_as_scroll.hash(h);
    state(h, &context.properties);
    // The two values the children's builders read from the current effect
    // node itself (PaintPropertyTreeBuilder's is_in_tainted_subtree and
    // drawable-canvas inputs): a change of either in place reaches them
    // like a context change.
    paint_model::property_cell::with_access(|a| {
        let effect = &context.properties.nodes.effect;
        (effect.IsInTaintedSubtree(a), effect.IsInDrawableCanvasSubtree(a))
    })
    .hash(h);
    match &context.canvas_contents {
        None => 0u8.hash(h),
        Some(canvas) => {
            1u8.hash(h);
            canvas.layout_object.hash(h);
            state(h, &canvas.properties);
        }
    }
    context.rendering_context_id.hash(h);
    context.should_flatten_inherited_transform.hash(h);
    context.can_inherit_backface_visibility.hash(h);
    offset(h, context.directly_composited_container_paint_offset_subpixel_delta);
    context.subtree_should_use_infinite_cull_rect.hash(h);
    context.flex_or_grid_items.hash(h);
    match &context.svg {
        None => 0u8.hash(h),
        Some(svg) => {
            1u8.hash(h);
            matrix(h, svg);
        }
    }
    hasher.finish()
}

type InlineOutlineKey = (u64, u64);

/// An occurrence of an inline continuation outline group (OutlinePainter
/// paints the united outline of the inline's fragments once).
fn InlineOutlineCandidate(fragment: &FragmentPaintData) -> bool {
    fragment.ViewNodeId() != 0
        && fragment.ViewKind() == FragmentKind::kBox
        && fragment.paint.HasSource()
        && fragment.ViewEstablishesPaintState()
        && fragment.paint.Display() == Display::kInline
        && fragment.paint.style.visible()
        && fragment.paint.style.outline_width() > 0.0
        && fragment.paint.style.outline_style() != BorderLineStyle::kNone
        && fragment.paint.style.outline_color().alpha > 0.0
}

/// One occurrence's AddSelfOutlineRects in the flat paint space, appended.
fn AppendSelfOutlineRects(fragment: &FragmentPaintData, paint_offset: Offset, rects: &mut Vec<PaintRect>) {
    if let Some(outline) = fragment.ViewSelfOutline() {
        rects.extend(outline.rects.iter().map(|rect| PaintRect {
            x: paint_offset.x + rect.offset.x,
            y: paint_offset.y + rect.offset.y,
            width: rect.size.width,
            height: rect.size.height,
        }));
    }
}

/// Chromium's stable AutoDarkModeSVGSizeThreshold feature scopes the graphics
/// context while an SVG root paints its children. The derived flag lives on
/// each immutable PaintStyle snapshot so reparented PaintLayer descendants and
/// ordinary children observe the same state without depending on the paint
/// walk's call-stack shape. Image auto-dark admission ignores this flag.
///
/// Whether `paint`'s stored state already is `inherited_paused`.
fn AutoDarkModeStateIsCurrent(fragment: &FragmentPaintData, inherited_paused: bool) -> bool {
    let paint = &fragment.paint;
    paint.style.auto_dark_mode_paused() == inherited_paused
        && !(paint.style.Source().is_none()
            && paint.selection_style.auto_dark_mode_paused() != inherited_paused)
        && !paint
            .svg_text_decoration_style
            .as_ref()
            .is_some_and(|style| style.auto_dark_mode_paused() != inherited_paused)
}

/// Stores `inherited_paused` on the occurrence's style payloads (assigning
/// only on change: they are shared copy-on-write owners) and returns the
/// state its children inherit.
fn ApplyAutoDarkModeState(
    fragment: &mut FragmentPaintData,
    paint_offset: Offset,
    paint_snap_offset: Offset,
    resources: Option<&Arc<PaintResources>>,
    inherited_paused: bool,
) -> bool {
    const MAX_IMAGE_LENGTH: f64 = 64.0;
    let child_paused = if fragment.paint.SourceKind() == NodeKind::kSvgRoot
        && fragment.paint.style.force_dark()
    {
        let scale = resources
            .map_or(1.0, |resources| resources.ArtifactToDeviceScale())
            .max(f64::MIN_POSITIVE);
        let snapped = PixelSnappedRectInPropertySpace(
            &PaintRect {
                x: paint_offset.x,
                y: paint_offset.y,
                width: fragment.ViewSize().width,
                height: fragment.ViewSize().height,
            },
            paint_snap_offset,
            scale,
        );
        let layout_zoom = resources
            .map_or(1.0, |resources| resources.layout_zoom_factor)
            .max(f64::MIN_POSITIVE);
        snapped.width / layout_zoom > MAX_IMAGE_LENGTH
            || snapped.height / layout_zoom > MAX_IMAGE_LENGTH
    } else {
        inherited_paused
    };
    let paint = &fragment.paint;
    let style_changed = paint.style.auto_dark_mode_paused() != inherited_paused;
    // A live-source selection style is resolved in Paint and takes the
    // originating style's pause (PaintProperties::SelectionPaintStyle).
    let selection_changed = paint.style.Source().is_none()
        && paint.selection_style.auto_dark_mode_paused() != inherited_paused;
    let decoration_changed = paint
        .svg_text_decoration_style
        .as_ref()
        .is_some_and(|style| style.auto_dark_mode_paused() != inherited_paused);
    if style_changed {
        fragment.paint.style.SetAutoDarkModePaused(inherited_paused);
    }
    if selection_changed {
        fragment.paint.selection_style.SetAutoDarkModePaused(inherited_paused);
    }
    if decoration_changed {
        if let Some(style) = &mut fragment.paint.svg_text_decoration_style {
            style.SetAutoDarkModePaused(inherited_paused);
        }
    }
    child_paused
}

/// The auto dark mode state of a subtree the walk did not enter: an object
/// whose stored state equals the inherited one kept its size, offset and
/// style payload, so its whole subtree holds the state of the last walk.
/// Only a subtree whose inherited state changed is updated.
fn PropagateAutoDarkModeOverCursor(cursor: PaintFragmentCursor<'_>, inherited_paused: bool) {
    if AutoDarkModeStateIsCurrent(cursor.PaintData(), inherited_paused) {
        return;
    }
    let (paint_offset, paint_snap_offset) = cursor
        .PrePaintData::<PrePaintFragmentData>()
        .map_or((Offset::default(), Offset::default()), |data| {
            (data.FlatReplay().paint_offset, data.FlatReplay().paint_snap_offset)
        });
    // SAFETY: PrePaint is the single lifecycle writer of the walked source;
    // no reference into this payload is live.
    let fragment = unsafe { cursor.PaintDataMut() };
    let child_paused = ApplyAutoDarkModeState(
        fragment,
        paint_offset,
        paint_snap_offset,
        cursor.DocumentResources(),
        inherited_paused,
    );
    for child in cursor.Children() {
        PropagateAutoDarkModeOverCursor(child, child_paused);
    }
}

/// The outermost effect node the occurrence's ObjectPaintProperties own at
/// the top of its border-box effect chain (Effect, else Filter).
fn OutermostOwnEffect(data: &PrePaintFragmentData) -> Option<Arc<EffectPaintPropertyNode>> {
    let properties = data.PaintProperties()?;
    properties.Effect().or_else(|| properties.Filter()).cloned()
}

/// Reparents `effect` to `parent` in place when it names another parent.
fn ReparentEffect(
    effect: &Arc<EffectPaintPropertyNode>,
    parent: &Arc<EffectPaintPropertyNode>,
    store: &mut PaintPropertyNodeStore,
) -> bool {
    let state = paint_model::property_cell::with_access(|access| {
        let state = effect.state(access);
        (!state
            .parent
            .as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, parent)))
        .then(|| {
            let mut state = state.clone();
            state.parent = Some(parent.clone());
            state
        })
    });
    let Some(state) = state else {
        return false;
    };
    paint_model::property_cell::with_writer(|writer| store.UpdateEffectNode(writer, effect, state));
    true
}

/// Decision D2 for an out-of-flow occurrence the walk advances without a
/// build: whether its last build already took `effect`, its logical parent
/// occurrence's effect, in the state RebindLogicalEffect would leave (the
/// outermost own effect parented to it, else LocalBorderBoxProperties and
/// the kept child context naming it; a fix-up after that build replaced
/// LocalBorderBoxProperties and dropped the kept context).
fn LogicalEffectIsCurrent(
    cursor: PaintFragmentCursor<'_>,
    effect: &Arc<EffectPaintPropertyNode>,
) -> bool {
    let Some(data) = cursor
        .PrePaintData::<PrePaintFragmentData>()
        .filter(|data| data.WasWalked())
    else {
        return false;
    };
    if let Some(outermost) = OutermostOwnEffect(data) {
        return paint_model::property_cell::with_access(|access| {
            outermost
                .state(access)
                .parent
                .as_ref()
                .is_some_and(|parent| Arc::ptr_eq(parent, effect))
        });
    }
    data.LocalBorderBoxPropertiesRef()
        .is_some_and(|state| Arc::ptr_eq(&state.effect, effect))
        && data
            .CachedChildContext()
            .is_none_or(|cached| Arc::ptr_eq(&cached.context.properties.nodes.effect, effect))
}

/// Decision D2 fix-up of one out-of-flow occurrence whose logical parent
/// occurrence gives it `parent_effect`. The builder parents an out-of-flow
/// object's effects to its logical parent occurrence when the walk visited
/// that occurrence before it (PrePaintTreeWalk::LogicalOutOfFlowEffectContext);
/// otherwise this applies it: the outermost own effect node is updated in
/// place with the logical parent, and an object owning none, with the
/// descendants that inherit its effect, takes the logical parent's node in
/// its LocalBorderBoxProperties (Chromium builds them from
/// `context.current_effect`, paint_property_tree_builder.cc:2955-2981,
/// which already follows the logical parent). Returns whether it changed.
fn RebindLogicalEffect(
    cursor: PaintFragmentCursor<'_>,
    parent_effect: &Arc<EffectPaintPropertyNode>,
    store: &mut PaintPropertyNodeStore,
) -> bool {
    let Some(data) = cursor.PrePaintData::<PrePaintFragmentData>() else {
        return false;
    };
    if let Some(outermost) = OutermostOwnEffect(data) {
        return ReparentEffect(&outermost, parent_effect, store);
    }
    let Some(old) = data
        .LocalBorderBoxPropertiesRef()
        .map(|state| state.effect.clone())
    else {
        return false;
    };
    if Arc::ptr_eq(&old, parent_effect) {
        return false;
    }
    RebindInheritedEffect(cursor, &old, parent_effect, store);
    true
}

/// RebindLogicalEffect for an occurrence inheriting `old`: it and every
/// descendant that inherits the same effect take `new`; an own outermost
/// effect parented to `old` is reparented in place.
fn RebindInheritedEffect(
    cursor: PaintFragmentCursor<'_>,
    old: &Arc<EffectPaintPropertyNode>,
    new: &Arc<EffectPaintPropertyNode>,
    store: &mut PaintPropertyNodeStore,
) {
    let Some(data) = cursor
        .PrePaintData::<PrePaintFragmentData>()
        .filter(|data| data.WasWalked())
    else {
        return;
    };
    if let Some(outermost) = OutermostOwnEffect(data) {
        let parented_to_old = paint_model::property_cell::with_access(|access| {
            outermost
                .state(access)
                .parent
                .as_ref()
                .is_some_and(|parent| Arc::ptr_eq(parent, old))
        });
        if parented_to_old {
            ReparentEffect(&outermost, new, store);
        }
        return;
    }
    let Some(mut state) = data.LocalBorderBoxPropertiesRef().cloned() else {
        return;
    };
    if !Arc::ptr_eq(&state.effect, old) {
        return;
    }
    // Scrollbar effects are parented to the border-box effect.
    let scrollbars: Vec<Arc<EffectPaintPropertyNode>> = data
        .PaintProperties()
        .map(|properties| {
            [
                properties.HorizontalScrollbarEffect(),
                properties.VerticalScrollbarEffect(),
            ]
            .into_iter()
            .flatten()
            .cloned()
            .collect()
        })
        .unwrap_or_default();
    state.effect = new.clone();
    // SAFETY: single PrePaint writer after the walk; no live reference.
    unsafe { cursor.PrePaintDataMut::<PrePaintFragmentData>() }.SetLocalBorderBoxProperties(state);
    for scrollbar in &scrollbars {
        let parented_to_old = paint_model::property_cell::with_access(|access| {
            scrollbar
                .state(access)
                .parent
                .as_ref()
                .is_some_and(|parent| Arc::ptr_eq(parent, old))
        });
        if parented_to_old {
            ReparentEffect(scrollbar, new, store);
        }
    }
    for child in cursor.Children() {
        RebindInheritedEffect(child, old, new, store);
    }
}

/// PaintController commit of client `id` on a payload whose client state is
/// detached from a live source client (paint_controller.cc:127-143 validates
/// a live client through its own pointer, done by the Layout commit).
fn CommitDetachedClient(f: &mut FragmentPaintData, id: u64) {
    if f.paint.DisplayItemClientId() == id
        && f.paint.display_item_client_is_cacheable
        && !f.paint.HasLiveDisplayItemClient()
    {
        f.paint
            .display_item_client_invalidation_reason
            .replace(PaintInvalidationReason::kNone);
        f.paint.display_item_client_is_just_created.set(false);
    }
    if f.paint.SelectionClientId() == id
        && f.paint.selection_client_is_cacheable
        && !f.paint.HasLiveSelectionClient()
    {
        f.paint
            .selection_client_invalidation_reason
            .replace(PaintInvalidationReason::kNone);
    }
    let live_scroll_client = f.paint.HasLiveScrollClient(id);
    let scroll_matches = !live_scroll_client
        && f.ViewScrollContainer().is_some_and(|scroll| {
            scroll.scrolling_background_client_id == id
                && scroll.scrolling_background_client_is_cacheable
        });
    if let Some(scroll) = scroll_matches
        .then(|| f.paint.scroll_container.as_mut())
        .flatten()
    {
        if scroll.scrolling_background_client_id == id
            && scroll.scrolling_background_client_is_cacheable
        {
            scroll
                .scrolling_background_client_invalidation_reason
                .replace(PaintInvalidationReason::kNone);
            scroll
                .scrolling_background_client_is_just_created
                .set(false);
        }
    }
    let bars_match = !live_scroll_client && f.paint.scrollbars.as_ref().is_some_and(|bars| {
        (bars.corner_client_id == id && bars.corner_client_is_cacheable)
            || [&bars.horizontal, &bars.vertical]
                .into_iter()
                .flatten()
                .any(|axis| {
                    axis.display_item_client_id == id && axis.display_item_client_is_cacheable
                })
    });
    if let Some(bars) = bars_match.then(|| f.paint.scrollbars.as_mut()).flatten() {
        let bars = Arc::make_mut(bars);
        for axis in [&mut bars.horizontal, &mut bars.vertical]
            .into_iter()
            .flatten()
        {
            if axis.display_item_client_id == id && axis.display_item_client_is_cacheable {
                axis.display_item_client_invalidation_reason
                    .replace(PaintInvalidationReason::kNone);
                axis.display_item_client_is_just_created.set(false);
            }
        }
        if bars.corner_client_id == id && bars.corner_client_is_cacheable {
            bars.corner_client_invalidation_reason
                .replace(PaintInvalidationReason::kNone);
            bars.corner_client_is_just_created.set(false);
        }
    }
}

/// An `Arc` slice holding `values`, reusing `old` when it already does.
fn SharedSlice<T: Clone + PartialEq>(old: Option<&Arc<[T]>>, values: &[T]) -> Option<Arc<[T]>> {
    if values.is_empty() {
        return None;
    }
    match old {
        Some(old) if **old == *values => Some(old.clone()),
        _ => Some(Arc::from(values)),
    }
}

/// Copies the node's flat replay values (the PrePaint copies the paint walk
/// still reads through the node) onto the occurrence's FragmentData.
fn WriteFlatReplayDataTo(node: &PaintTreeNode<'_>, data: &mut PrePaintFragmentData) {
    // CullRectUpdater stores cull rects only for PaintLayer objects
    // (SetFragmentCullRects); another object keeps PrePaint's rects here
    // only when they are not its paint parent's contents cull rect.
    let has_layer = node
        .source_cursor
        .is_some_and(|cursor| cursor.OwnPaintLayer().is_some());
    let inherited_cull = node.cache.as_ref().map(|cache| cache.inherited.cull);
    let non_layer_cull_rects = (!has_layer
        && inherited_cull
            .is_none_or(|cull| cull != node.self_cull_rect || cull != node.cull_rect))
    .then_some((node.self_cull_rect, node.cull_rect));
    let has_mask_clip_inputs = !node.mask_layers.is_empty()
        || data.HasPropertySlot(ObjectPaintPropertySlot::MaskClip)
        || data.HasPropertySlot(ObjectPaintPropertySlot::Mask);
    let flat = data.FlatReplayMut();
    flat.paint_offset = node.paint_offset;
    flat.paint_snap_offset = node.paint_snap_offset;
    flat.clip = node.clip;
    if !Arc::ptr_eq(&flat.transforms, &node.transforms) {
        flat.transforms = node.transforms.clone();
    }
    flat.scroll_offset = node.scroll_offset;
    flat.applies_transform = node.applies_transform;
    flat.fixed_background_acts_as_scroll = node.fixed_background_acts_as_scroll;
    flat.applies_root_clip = node.applies_root_clip;
    flat.paints_outline = node.paints_outline;
    flat.skip_float_phase = !node.needs_float_phase;
    flat.skip_outline_phase = !node.needs_outline_phase;
    flat.skip_overflow_controls_phase = !node.needs_overflow_controls_phase;
    let old = flat.Rare();
    let rare = crate::fragment_data::FlatReplayRareData {
        local_transforms: SharedSlice(
            old.and_then(|rare| rare.local_transforms.as_ref()),
            &node.local_transforms,
        ),
        root_clip: node.root_clip,
        clip_path_commands: SharedSlice(
            old.and_then(|rare| rare.clip_path_commands.as_ref()),
            &node.local_clip_path,
        ),
        fixed_background_transform: node
            .fixed_background_properties
            .as_ref()
            .map(|state| state.transform.clone()),
        non_layer_cull_rects,
        outline_rects: SharedSlice(
            old.and_then(|rare| rare.outline_rects.as_ref()),
            &node.outline_rects,
        ),
        mask_layers: SharedSlice(
            old.and_then(|rare| rare.mask_layers.as_ref()),
            &node.mask_layers,
        ),
        // Read by the MaskClip flat replay scope and SVGMaskPainter only.
        mask_clip_rect: if has_mask_clip_inputs {
            node.ResolvedMaskClipRect()
        } else {
            None
        },
        pre_transform: (!Arc::ptr_eq(&node.pre_transform, &node.properties.nodes.transform))
            .then(|| node.pre_transform.clone()),
    };
    flat.SetRare(rare);
}

/// Nanoseconds between two profiling checkpoints; zero when not profiling.
fn ProfileNs(from: Option<std::time::Instant>, to: Option<std::time::Instant>) -> u64 {
    match (from, to) {
        (Some(from), Some(to)) => to.duration_since(from).as_nanos().min(u64::MAX as u128) as u64,
        _ => 0,
    }
}

// ViewportGeometry is a host/CSS-pixel input, while the fragment tree and
// paint property nodes live in LocalFrame::LayoutZoomFactor coordinates.
// Keeping the conversion at the PrePaint boundary also lets CSSOM and media
// queries continue to consume the unscaled host viewport.
fn ArtifactViewport(resources: Option<&PaintResources>) -> Option<PaintRect> {
    resources.and_then(|resources| {
        let viewport = resources.viewport?;
        let zoom = resources.layout_zoom_factor.max(f64::MIN_POSITIVE);
        Some(PaintRect {
            x: 0.0,
            y: 0.0,
            width: viewport.size.width as f64 * zoom,
            height: viewport.size.height as f64 * zoom,
        })
    })
}

impl PrePaintTreeWalk {
    pub fn IntersectionObservationState(
        &self,
    ) -> foundation::intersection_observation_state::IntersectionObservationState {
        self.intersection_observation_state
    }
    /// The standalone LayoutView supplies the outer layout viewport. Paint's
    /// retained Page viewport nodes supply Chromium's scale-1 visual viewport;
    /// PAC receives references to both owners and only translates them to cc.
    pub fn SetVisualViewportState(&mut self, state: paint_model::PaintVisualViewportState) {
        assert!(state.scale.is_finite() && state.scale > 0.);
        self.visual_viewport_state = Some(state);
    }
    pub fn SetVisualViewportScrollOffset(
        &mut self,
        element: foundation::CompositorElementId,
        offset: foundation::gfx::Vector2dF,
    ) {
        self.visual_viewport_scroll_offset = Some((element, offset));
    }
    pub fn ExportCompositorViewport(
        &self,
        root: &FragmentPaintData,
    ) -> Option<paint_model::PaintCompositorViewport> {
        if !root.paint.IsLayoutView() {
            return None;
        }
        Some(paint_model::PaintCompositorViewport {
            mode: paint_model::PaintCompositorViewportMode::SingleLayoutViewport,
            viewport_clip: self.property_node_store.viewport_clip()?.clone(),
            overscroll_elasticity_transform: self
                .property_node_store
                .overscroll_elasticity_transform()?
                .clone(),
            page_scale: self.property_node_store.page_scale()?.clone(),
            visual_scroll_translation: self
                .property_node_store
                .visual_viewport_scroll_translation()?
                .clone(),
            layout_scroll_translation: self
                .property_node_store
                .root_scroller_scroll_translation()
                .cloned(),
            scroll_translation_nodes: self
                .property_node_store
                .compositor_scroll_translations()
                .cloned()
                .collect(),
        })
    }
    /// Source: pre_paint_tree_walk.cc:270-282,725-730. Whether the object is
    /// inside a blocking wheel event handler region: a changed registry
    /// (or a replacement LayoutResult) re-reads the handler targets,
    /// otherwise the parent's bit or the stored one stands.
    fn InsideBlockingWheelEventHandler(
        &self,
        fragment: &FragmentPaintData,
        inside_parent: bool,
        handler_changed: bool,
    ) -> bool {
        let should_update = handler_changed
            || (self.source_result_changed
                && (fragment.paint.LayoutObjectIdentity().is_some()
                    || !self.active_blocking_wheel_targets.is_empty()));
        inside_parent
            || if should_update {
                self.active_blocking_wheel_targets
                    .binary_search(&fragment.ViewNodeId())
                    .is_ok()
                    || (fragment.paint.IsLayoutView()
                        && self.active_blocking_wheel_targets.binary_search(&0).is_ok())
            } else {
                fragment.paint.InsideBlockingWheelEventHandler()
            }
    }

    /// Returns the PRE_PAINT_SKIP_REJECT_FIELDS index of the first failing
    /// condition group when the subtree must be visited.
    ///
    /// Source: PrePaintTreeWalk::Walk (pre_paint_tree_walk.cc:1477-1482)
    /// with NeedsTreeBuilderContextUpdate, ObjectRequiresPrePaint and
    /// ContextRequiresChildPrePaint (pre_paint_tree_walk.cc:398-450): the walk
    /// returns before it builds a context for the object when the parent's
    /// context forces nothing and neither the object nor a descendant carries
    /// a dirty bit. The object's FragmentData then keeps every value of its
    /// last visit. `force_subtree` / `context_changed` are the parent's
    /// force_subtree_update_reasons; the walked owner, link offset, size and
    /// scopes are the values Chromium keeps on the persistent LayoutObject and
    /// FragmentData (a cache-hit child the parent's relayout moved, or a
    /// fragment the replacement LayoutResult did not reuse, is visited).
    #[allow(clippy::too_many_arguments)]
    fn CanSkipSubtree(
        &self,
        fragment: &PaintFragmentSource<'_>,
        data: &PrePaintFragmentData,
        force_subtree: bool,
        context_changed: bool,
        base: &ChildContext,
        parent_invalidation: &PaintInvalidatorContext,
        display_item_fragment: u32,
        fragmentainer: u64,
    ) -> Result<(), usize> {
        let record = data.WalkRecord();
        let invalidation = fragment.pre_paint_invalidation;
        let client = fragment.paint.DisplayItemClientId();
        if self.full_walk || force_subtree || context_changed || !record.walked {
            return Err(SKIP_REJECT_FORCED);
        }
        if invalidation.own() {
            return Err(SKIP_REJECT_OWN);
        }
        if invalidation.descendants() {
            return Err(SKIP_REJECT_DESCENDANTS);
        }
        if base.blocking_wheel_event_handler_changed
            || base.canvas_contents.is_some()
            || parent_invalidation.NeedsSubtreeWalk()
            || fragment.paint.IsSVGHiddenContainer()
            || (fragment.paint.FixedToView() && fragment.ViewEstablishesPaintState())
            || self.local_frame_views.contains_key(&client)
            || self.must_visit_clients.contains(&client)
        {
            return Err(SKIP_REJECT_WALK_STATE);
        }
        let owner = WalkedOwner(fragment.Cursor());
        if owner == (0, 0) || record.walked_owner != owner {
            return Err(SKIP_REJECT_RECORD_OWNER);
        }
        if record.link_offset != fragment.Cursor().ViewOffsetInContainer()
            || record.size != fragment.ViewSize()
            || record.display_item_fragment != display_item_fragment
            || record.fragmentainer != fragmentainer
        {
            return Err(SKIP_REJECT_RECORD_GEOMETRY);
        }
        if self.InsideBlockingWheelEventHandler(
            fragment,
            base.inside_blocking_wheel_event_handler,
            false,
        ) != fragment.paint.InsideBlockingWheelEventHandler()
        {
            return Err(SKIP_REJECT_WHEEL_INSIDE);
        }
        Ok(())
    }

    /// Source: PrePaintTreeWalk::WalkInternal and WalkChildren
    /// (pre_paint_tree_walk.cc:650-760,1517-1582) for one visited fragment
    /// occurrence: PaintPropertyTreeBuilder updates the occurrence's
    /// FragmentData (PaintOffset, ObjectPaintProperties in place,
    /// LocalBorderBoxProperties, cull rects), PaintInvalidator compares the
    /// previous values Layout and the FragmentData kept, and the children
    /// are walked with a context derived on the stack. The PaintTreeNode the
    /// builder returns is that context and is dropped with the visit.
    fn WalkObject(&mut self, fragment: PaintFragmentSource<'_>, mut ctx: WalkContext) -> VisitResult {
        let profile = browser_tracing::enabled();
        let checkpoint = || profile.then(std::time::Instant::now);
        let t_start = checkpoint();
        // FragmentDataIterator: reach this object's FragmentData once.
        // Source: pre_paint_tree_walk.cc:418-436. Read this object's dirty
        // bits as of this visit, through that same FragmentData.
        let fragment = unsafe { fragment.ReadPrePaintInvalidationForVisit() };
        let cursor = fragment.Cursor();
        let client = fragment.paint.DisplayItemClientId();
        let display_item_fragment = self.display_item_fragment_scope.unwrap_or(0);
        let fragmentainer = self.fragmentainer_scope.unwrap_or(0);
        // Decision D2: an out-of-flow object takes its effect state from its
        // logical parent (RebindLogicalEffect fixes up the occurrences the
        // walk reaches before their logical parent).
        let logical_effect = self.LogicalOutOfFlowEffectContext(&fragment);
        let has_logical_effect = logical_effect.is_some();
        if let Some(logical) = logical_effect {
            ctx.inherited.properties.nodes.effect = logical.effect;
            ctx.inherited.properties.effect_id = logical.effect_id;
            ctx.inherited.opacity = logical.opacity;
        }
        // Source pre_paint_tree_walk.cc:270-282,725-730. The source registry
        // invalidates the affected native subtree; this walk updates its
        // resident LayoutObject/FragmentData before paint invalidation.
        ctx.inherited.blocking_wheel_event_handler_changed |= fragment
            .pre_paint_invalidation
            .blocking_wheel_event_handler_changed;
        let inside = self.InsideBlockingWheelEventHandler(
            &fragment,
            ctx.inherited.inside_blocking_wheel_event_handler,
            ctx.inherited.blocking_wheel_event_handler_changed,
        );
        // Source: paint_property_tree_builder.cc UpdateForSelf /
        // UpdateForChildren without NeedsPaintPropertyUpdate or a forced
        // subtree update: the builder only advances the context past the
        // object's existing nodes. Such an object is visited for a
        // descendant's dirty bit or its own paint invalidation check.
        // Decision D2 on the context-only path: the logical parent's effect
        // must be the one the last build took, which the occurrence's nodes
        // and kept child context name.
        let clean = if has_logical_effect
            && !LogicalEffectIsCurrent(cursor, &ctx.inherited.properties.nodes.effect)
        {
            Err(CLEAN_REJECT_WALK_STATE)
        } else {
            self.CleanVisitChildContext(&fragment, &ctx, inside, display_item_fragment, fragmentainer)
        };
        match clean {
            Ok((base, invalidate)) => {
                return self.WalkCleanObject(
                    fragment,
                    ctx,
                    base,
                    invalidate,
                    inside,
                    display_item_fragment,
                    fragmentainer,
                    t_start,
                );
            }
            Err(reason) => self.stats.clean_rejects[reason] += 1,
        }
        let mut resident = ResidentFragment::FromSource(&fragment);
        // The payload is shared copy-on-write with the previous publication.
        // Its stored bit and the LayoutObject's are written together, so an
        // equal bit needs no update.
        let blocking_wheel_changed = resident.paint.InsideBlockingWheelEventHandler() != inside
            && resident.paint.UpdateInsideBlockingWheelEventHandler(inside);
        if ctx.inherited.blocking_wheel_event_handler_changed || blocking_wheel_changed {
            resident.paint.InvalidatePaintForHitTest();
        }
        if blocking_wheel_changed {
            resident
                .pre_paint_invalidation
                .should_check_for_paint_invalidation = true;
        }
        drop(resident);
        // The previous visit's values, before the builder replaces them.
        let previous = cursor
            .PrePaintData::<PrePaintFragmentData>()
            .filter(|data| data.WasWalked())
            .map(|data| PreviousVisit::Of(data, ctx.old_parent_contents_cull));
        let old_paint_offset = previous.as_ref().map(|previous| previous.flat_paint_offset);
        let old_self_cull_rect = previous.as_ref().map(|previous| previous.self_cull_rect);
        let old_contents_cull_rect = previous.as_ref().map(|previous| previous.contents_cull_rect);
        // PaintLayer::PreviousPaintResult (paint_layer.h:400-405), written by
        // PaintLayerPainter::Paint (paint_layer_painter.cc:589-596); only a
        // painting-layer object has one.
        let old_previous_paint_result = previous
            .as_ref()
            .filter(|previous| previous.record.is_paint_layer)
            .and_then(|_| cursor.OwnPaintLayer())
            .map_or(paint_model::PaintResult::kMayBeClippedByCullRect, |layer| {
                layer.PreviousPaintResult()
            });
        let mut invalidation_context = ctx.invalidation;
        let should_run_invalidation =
            PaintInvalidator::UpdateContext(&fragment, &mut invalidation_context);
        PaintInvalidator::UpdatePaintingLayer(cursor, &mut invalidation_context);
        self.stats.invalidation_nodes += should_run_invalidation as usize;
        let t_admitted = checkpoint();
        self.stats.profile_admission_ns += ProfileNs(t_start, t_admitted);
        let inherited = ctx.inherited;
        let mut node = PaintPropertyTreeBuilder::new(self)
            .with_previous_paint_result(old_previous_paint_result)
            .with_link_offset(cursor.ViewOffsetInContainer())
            .with_source_cursor(cursor)
            .BuildNode(
                &fragment,
                inherited.offset,
                inherited.paint_space,
                inherited.clip,
                inherited.cull,
                inherited.transforms.clone(),
                inherited.opacity,
                inherited.scroll,
                inherited.fixed_background_acts_as_scroll,
                inherited.properties.clone(),
                inherited.canvas_contents.clone(),
                inherited.rendering_context_id,
                inherited.should_flatten_inherited_transform,
                inherited.can_inherit_backface_visibility,
                inherited.directly_composited_container_paint_offset_subpixel_delta,
                inherited.subtree_should_use_infinite_cull_rect,
                old_paint_offset,
                previous
                    .as_ref()
                    .is_some_and(|previous| previous.had_paint_offset_translation),
                old_self_cull_rect,
                old_contents_cull_rect,
                inherited.root,
                inherited.flex_or_grid_item,
            );
        node.display_item_fragment = Some(display_item_fragment);
        node.fragmentainer_instance_id = Some(fragmentainer);
        node.source_cursor = Some(unsafe {
            std::mem::transmute::<PaintFragmentCursor<'_>, PaintFragmentCursor<'static>>(cursor)
        });
        // The parent's SVG viewBox matrix opens this object's flat replay.
        if let Some(svg) = ctx.leading_svg_matrix {
            node.local_transforms.insert(0, svg);
            node.applies_transform = true;
        }
        let cull_change_needs_repaint = CullRectChangeNeedsRepaint(
            old_previous_paint_result,
            old_self_cull_rect,
            old_contents_cull_rect,
            node.self_cull_rect,
            node.cull_rect,
        );
        if cull_change_needs_repaint {
            self.stats.cull_change_repaints += 1;
            InvalidateClientsForCullRectChange(
                node.fragment
                    .as_mut()
                    .expect("PrePaint node must own a fragment"),
            );
        }
        node.previous_paint_result.set(old_previous_paint_result);
        node.self_or_descendant_needs_repaint |= cull_change_needs_repaint;
        let own_change = node
            .cache
            .as_ref()
            .map(|cache| cache.own_change)
            .unwrap_or_default();
        CullRectUpdater::PaintPropertiesChanged(
            cursor,
            &fragment,
            own_change,
            node.cull_decision_inputs,
            old_previous_paint_result,
        );
        // ObjectPaintProperties::Clear*: a slot this object no longer owns is
        // released with its role.
        if let Some(previous) = previous.as_ref().filter(|_| client != 0) {
            let kept = &node.cache.as_ref().unwrap().own_keys;
            for role in &previous.roles {
                if !kept.iter().any(|(_, key)| key.role == *role) {
                    self.released_property_keys.push(PaintPropertyKey {
                        owner: PaintPropertyOwner::Fragment {
                            client,
                            fragment: previous.record.display_item_fragment,
                        },
                        role: *role,
                    });
                }
            }
        }
        // Source: cull_rect_updater.cc:67-89 (SetFragmentContentsCullRect,
        // reached from UpdateForSelf at :393-431) and
        // paint_layer_scrollable_area.cc:1919-1920 (DidUpdateCullRect).
        if old_contents_cull_rect.is_none_or(|old| old != node.cull_rect) {
            fragment
                .paint
                .SetLastCullRectUpdateScrollPosition(fragment.paint.ScrollOffset());
        }
        let t_built = checkpoint();
        self.stats.profile_build_ns += ProfileNs(t_admitted, t_built);
        if let Some(context) = LogicalEffectContextOf(&node, fragmentainer) {
            let entries = self
                .logical_effect_contexts
                .entry(fragment.ViewNodeId())
                .or_default();
            match entries
                .iter_mut()
                .find(|entry| entry.fragment_instance_id == context.fragment_instance_id)
            {
                Some(entry) => {
                    if !Arc::ptr_eq(&entry.effect, &context.effect)
                        || entry.effect_id != context.effect_id
                        || entry.fragmentainer != context.fragmentainer
                    {
                        self.logical_effects_changed.insert(fragment.ViewNodeId());
                    }
                    *entry = context;
                }
                None => {
                    self.logical_effects_changed.insert(fragment.ViewNodeId());
                    entries.push(context);
                }
            }
        }
        // Chromium runs invalidation after property construction:
        // UpdateForObjectLocation may have dirtied the object.
        let needs_invalidation_after_properties =
            PaintInvalidator::UpdateContext(&fragment, &mut invalidation_context);
        self.stats.invalidation_nodes +=
            (needs_invalidation_after_properties && !should_run_invalidation) as usize;
        let should_run_invalidation = needs_invalidation_after_properties;
        let old_paint_offset = PaintInvalidator::UpdateFromTreeBuilderContext(
            node.directly_composited_container_paint_offset_subpixel_delta,
            old_paint_offset,
            node.paint_offset,
        );
        // The invalidators read the previous box geometry from the source
        // LayoutBox (PreviousBoxGeometry); no previous fragment is retained.
        let mut table_cell_geometry_changed = fragment.paint.Display() == Display::kTableCell
            && should_run_invalidation
            && TableCellPaintInvalidator::CellGeometryChanged(
                &fragment,
                None,
                old_paint_offset,
                node.paint_offset,
            );
        if should_run_invalidation {
            let old_gap_rules = previous.as_ref().and_then(|previous| previous.gap_rules.clone());
            let new_paint_offset = node.paint_offset;
            self.InvalidatePaintForVisit(
                fragment,
                node.fragment
                    .as_mut()
                    .expect("PrePaint node must own a fragment"),
                old_gap_rules.as_deref(),
                old_paint_offset,
                new_paint_offset,
                invalidation_context,
            );
        }
        let t_invalidated = checkpoint();
        self.stats.profile_invalidation_ns += ProfileNs(t_built, t_invalidated);
        // This visit's final cull rects, also on the occurrence's FragmentData.
        unsafe {
            CullRectUpdater::SetFragmentCullRects(cursor, node.self_cull_rect, node.cull_rect)
        };
        // The context the children inherit (WalkChildren).
        {
            let children = &mut node.cache.as_mut().unwrap().children;
            children.inside_blocking_wheel_event_handler = inside;
            children.blocking_wheel_event_handler_changed =
                inherited.blocking_wheel_event_handler_changed;
        }
        if inherited.root {
            let base = &node.cache.as_ref().unwrap().children;
            self.CaptureRootFixedPositionContext(base, Some(&node));
        }
        // Source: force_subtree_update_reasons. PaintPropertyTreeBuilder forces
        // the subtree when one of this object's nodes was added or removed
        // (UpdateForChildren, paint_property_tree_builder.cc:4736-4751) or
        // its filter taint changed (:2728-2730); both are taken as
        // isolation-piercing here. A node whose value changed in place keeps
        // its identity: the descendants' FragmentData still names the same
        // live nodes, so Chromium does not revisit them (only GeometryMapper
        // caches are cleared). Every other dependency of the children on
        // this object (paint offset, which Chromium forces at :4155-4166,
        // the current transform/clip/effect/scroll nodes, the flat values
        // they store) is in the child context: a context unlike the one the
        // children were built with updates the children, which decide for
        // their own (NeedsTreeBuilderContextUpdate).
        let mut force_children = ctx.force_subtree
            || own_change >= PaintPropertyChangeType::NodeAddedOrRemoved
            || FilterTaintChanged(
                previous
                    .as_ref()
                    .is_some_and(|previous| previous.filter_origin_tainted),
                &node,
            );
        let digest = ChildContextDigest(&node.cache.as_ref().unwrap().children);
        let children_context_changed = previous
            .as_ref()
            .is_none_or(|previous| previous.record.child_context_digest != digest);
        let child_auto_dark_mode_paused = {
            let paint_offset = node.paint_offset;
            let paint_snap_offset = node.paint_snap_offset;
            ApplyAutoDarkModeState(
                node.fragment
                    .as_deref_mut()
                    .expect("PrePaint node must own a fragment"),
                paint_offset,
                paint_snap_offset,
                cursor.DocumentResources(),
                ctx.auto_dark_mode_paused,
            )
        };
        let record = PrePaintWalkRecord {
            walked: true,
            walked_owner: WalkedOwner(cursor),
            link_offset: cursor.ViewOffsetInContainer(),
            size: fragment.ViewSize(),
            display_item_fragment,
            fragmentainer,
            is_paint_layer: node.is_paint_layer,
            layer_cull_rects: cursor.OwnPaintLayer().is_some(),
            applies_opacity: node.applies_opacity,
            cull_decision_inputs: [
                node.cull_decision_inputs.has_filter_that_moves_pixels,
                node.cull_decision_inputs.has_perspective,
                node.cull_decision_inputs.has_active_transform_animation,
                node.cull_decision_inputs.has_non_composited_transform,
            ],
            child_context_digest: digest,
            context_generation: self.child_context_generation,
        };
        if InlineOutlineCandidate(&fragment) {
            self.visited_inline_outline = true;
        }
        let mut phase = OwnPhaseNeeds(&fragment, node.is_paint_layer);
        // Source: core/paint/pre_paint_tree_walk.cc:1517-1535. Display
        // locking blocks only the child walk: the children keep their
        // FragmentData. A context change they could not take is a pending
        // child walk (DisplayLockContext::SetNeedsPrePaintSubtreeWalk),
        // forced once the lock opens.
        let blocked = fragment
            .pre_paint_invalidation
            .child_pre_paint_blocked_by_display_lock;
        let was_pending = self.previous_display_lock_walks.remove(&client).is_some();
        if blocked {
            if force_children || children_context_changed || was_pending {
                self.pending_display_lock_walks
                    .insert(client, self.walk_ancestors.clone());
            }
            for child in OrdinaryPaintSourceChildren(fragment) {
                if let Some(data) = child
                    .Cursor()
                    .PrePaintData::<PrePaintFragmentData>()
                    .filter(|data| data.WasWalked())
                {
                    let result = VisitResult::Stored(data);
                    if !result.is_paint_layer {
                        phase.Merge(result.phase);
                    }
                }
            }
            self.stats.profile_children_setup_ns += ProfileNs(t_invalidated, checkpoint());
            // The blocked children were not built with this context.
            return self.FinishVisit(
                &mut node,
                fragment,
                record,
                phase,
                table_cell_geometry_changed,
                None,
            );
        }
        if was_pending {
            force_children = true;
        }
        let old_children_contents_cull = previous
            .as_ref()
            .map(|previous| previous.contents_cull_rect);
        // The context this build hands its children, kept on the
        // FragmentData for a later visit that needs no property update.
        let base = node.cache.as_ref().unwrap().children.clone();
        let owner_paint_offset = node.paint_offset;
        let has_children = self.WalkChildren(
            fragment,
            node.fragment
                .as_mut()
                .expect("PrePaint node must own a fragment"),
            owner_paint_offset,
            &base,
            invalidation_context,
            force_children,
            children_context_changed,
            old_children_contents_cull,
            child_auto_dark_mode_paused,
            display_item_fragment,
            fragmentainer,
            &mut phase,
            &mut table_cell_geometry_changed,
            t_invalidated,
        );
        let t_looped = checkpoint();
        let child_context = has_children.then(|| {
            Box::new(CachedChildContext {
                context: base,
                generation: self.child_context_generation,
            })
        });
        let result = self.FinishVisit(
            &mut node,
            fragment,
            record,
            phase,
            table_cell_geometry_changed,
            child_context,
        );
        self.stats.profile_tail_ns += ProfileNs(t_looped, checkpoint());
        result
    }

    /// Source: PaintInvalidator::InvalidatePaint (paint_invalidator.cc:
    /// 217-290) for a visited object UpdateContext admitted, after the
    /// builder updated its PaintOffset or advanced the context past it:
    /// the box and object invalidators compare the previous values the
    /// LayoutBox and FragmentData kept (`old_paint_offset`, the LayoutBox
    /// gap baseline `old_gap_rules`) with the current ones, invalidate the
    /// object's clients and save the current geometry for the next walk.
    /// `resident` is the object's resident fragment.
    #[allow(clippy::too_many_arguments)]
    fn InvalidatePaintForVisit(
        &mut self,
        fragment: PaintFragmentSource<'_>,
        resident: &mut ResidentFragment,
        old_gap_rules: Option<&layout::paint::GapRulePaintData>,
        old_paint_offset: Option<Offset>,
        new_paint_offset: Offset,
        invalidation_context: PaintInvalidatorContext,
    ) {
        let cursor = fragment.Cursor();
        let background_invalidation = (fragment.ViewKind() == FragmentKind::kBox).then(|| {
            let fragment_data: &FragmentPaintData = &fragment;
            let view_background_rects = self
                .view_background_rects
                .filter(|(owner, _)| {
                    *owner == fragment_data as *const FragmentPaintData as usize
                })
                .map(|(_, rects)| rects);
            let mut background =
                BoxPaintInvalidator::ComputeBackgroundInvalidationWithViewRects(
                    &fragment,
                    Some(cursor),
                    None,
                    old_gap_rules,
                    invalidation_context,
                    view_background_rects,
                );
            if self.view_background_geometry_changed
                && fragment.paint.IsBackgroundAttachmentFixedObject()
            {
                background = BackgroundInvalidation {
                    kind: BackgroundInvalidationType::Full,
                    invalidate_both_spaces: true,
                };
            }
            background
        });
        let reason = ObjectPaintInvalidator::ComputePaintInvalidationReason(
            &fragment,
            old_paint_offset,
            new_paint_offset,
            invalidation_context,
        );
        // Source: object_paint_invalidator.cc:49-53. The reason above read
        // the previous visibility; save the current one on the
        // LayoutObject for the next walk.
        fragment.paint.UpdateSourcePreviousVisibilityVisible();
        let reason = if fragment.ViewKind() == FragmentKind::kBox {
            if fragment.paint.Display() == Display::kTableCell {
                TableCellPaintInvalidator::ComputePaintInvalidationReason(
                    &fragment,
                    None,
                    old_paint_offset,
                    new_paint_offset,
                    invalidation_context,
                    reason,
                )
            } else if fragment.paint.IsLayoutBlock() {
                BlockPaintInvalidator::ComputePaintInvalidationReason(
                    &fragment,
                    None,
                    old_paint_offset,
                    new_paint_offset,
                    invalidation_context,
                    reason,
                )
            } else {
                BoxPaintInvalidator::ComputePaintInvalidationReason(
                    &fragment,
                    None,
                    old_paint_offset,
                    new_paint_offset,
                    invalidation_context,
                    reason,
                )
            }
        } else {
            reason
        };
        let resident_fragment = &mut **resident;
        let reason = background_invalidation.map_or(reason, |background| {
            BoxPaintInvalidator::InvalidateBackground(resident_fragment, background, reason)
        });
        ObjectPaintInvalidator::InvalidatePaintWithComputedReason(resident_fragment, reason);
        // Source: box_paint_invalidator.cc:416-427. Every invalidator of
        // this object has read the previous box geometry; save the
        // current one for the next walk ("must be at the end").
        if fragment.ViewKind() == FragmentKind::kBox {
            resident_fragment.paint.SaveSourcePreviousBoxGeometries();
        }
        self.paint_invalidator
            .TrackPendingDelayedPaintInvalidation(resident_fragment);
        // PaintInvalidator.cc:278-285. Visibility:hidden clients still
        // invalidate observations even when they do not invalidate paint.
        let invalidation = resident_fragment.pre_paint_invalidation;
        let client_reason = resident_fragment
            .paint
            .DisplayItemClientState()
            .invalidation_reason
            .unwrap_or(foundation::PaintInvalidationReason::kNone);
        // PrePaintTreeWalk.cc:123-132: any actual paint invalidation
        // requires the next scroll/visibility observation.
        if client_reason != foundation::PaintInvalidationReason::kNone {
            self.intersection_observation_state = self.intersection_observation_state.max(
                foundation::intersection_observation_state::IntersectionObservationState::ScrollAndVisibilityOnly,
            );
        }
        if invalidation.should_check_layout_for_paint_invalidation
            && (foundation::IsLayoutPaintInvalidationReason(client_reason)
                || client_reason == foundation::PaintInvalidationReason::kJustCreated
                || !resident_fragment.paint.style.visible())
        {
            self.intersection_observation_state =
                foundation::intersection_observation_state::IntersectionObservationState::Desired;
        }
    }

    /// Whether the visited object `fragment` takes the builder's
    /// context-only path, and the context its children inherit then.
    ///
    /// Source: PaintPropertyTreeBuilder (paint_property_tree_builder.cc
    /// UpdateForSelf / UpdateForChildren) with neither NeedsPaintPropertyUpdate
    /// nor force_subtree_update_reasons, and PrePaintTreeWalk::WalkInternal
    /// (pre_paint_tree_walk.cc:650-760): the object is visited for a
    /// descendant's dirty bit (NeedsTreeBuilderContextUpdate for the
    /// descendant) or for its own paint invalidation only, so its own
    /// FragmentData stays as its last build left it and the context it hands
    /// its children is the one that build produced. The conditions are those
    /// under which the walk would skip the object's whole subtree
    /// (CanSkipSubtree), except the descendant and paint-check bits, plus the inputs a build
    /// reads beside the context: the previous paint result of a
    /// non-composited transform (ShouldUseInfiniteCullRect) and the composited
    /// container's subpixel movement, which a rebuild of an unmoved object
    /// resets.
    ///
    /// Paint invalidation is not a reason to build. Chromium marks the whole
    /// container chain of a paint-dirtied object ShouldCheckForPaintInvalidation
    /// (LayoutObject::SetShouldCheckForPaintInvalidation*,
    /// layout_object.cc), and a relaid-out object
    /// ShouldCheckLayoutForPaintInvalidation (ClearNeedsLayout); neither is
    /// NeedsPaintPropertyUpdate, so the builder only advances its context past
    /// the object (ObjectRequiresTreeBuilderContext without a property update)
    /// and PaintInvalidator::InvalidatePaint runs on the unchanged
    /// PaintOffset. The second value says whether that invalidation runs.
    /// Neither is the PhysicalFragment the last visit walked: Chromium keys
    /// FragmentData on the LayoutObject, so a box the replacement LayoutResult
    /// re-created with the same link offset, size and saved box geometry keeps
    /// its FragmentData, as one without layout change does; the dirty bits
    /// Layout set (NeedsPaintPropertyUpdate) decide whether it is rebuilt.
    ///
    /// The error is the PRE_PAINT_CLEAN_REJECT_FIELDS index of the first
    /// failing condition group.
    fn CleanVisitChildContext(
        &self,
        fragment: &PaintFragmentSource<'_>,
        ctx: &WalkContext,
        inside: bool,
        display_item_fragment: u32,
        fragmentainer: u64,
    ) -> Result<(Option<ChildContext>, bool), usize> {
        let invalidation = fragment.pre_paint_invalidation;
        let client = fragment.paint.DisplayItemClientId();
        if self.full_walk
            || ctx.force_subtree
            || ctx.context_changed
            || (ctx.inherited.root && (self.local_frame_depth != 0 || self.root_context_changed))
        {
            return Err(CLEAN_REJECT_FORCED);
        }
        if invalidation.needs_paint_property_update
            || invalidation.blocking_wheel_event_handler_changed
        {
            return Err(CLEAN_REJECT_OWN_PROPERTY);
        }
        let is_box = fragment.ViewKind() == FragmentKind::kBox && fragment.paint.HasSource();
        if invalidation.subtree_should_check_for_paint_invalidation
            || invalidation.subtree_should_do_full_paint_invalidation
            || (invalidation.should_check_layout_for_paint_invalidation && !is_box)
        {
            return Err(CLEAN_REJECT_OWN_LAYOUT);
        }
        if invalidation.child_pre_paint_blocked_by_display_lock
            || ctx.inherited.blocking_wheel_event_handler_changed
            || ctx.inherited.canvas_contents.is_some()
            || ctx.invalidation.NeedsSubtreeWalk()
            || fragment.paint.IsSVGHiddenContainer()
            || (fragment.paint.FixedToView() && fragment.ViewEstablishesPaintState())
            || self.local_frame_views.contains_key(&client)
            || self.must_build_clients.contains(&client)
            || self.previous_display_lock_walks.contains_key(&client)
            || inside != fragment.paint.InsideBlockingWheelEventHandler()
        {
            return Err(CLEAN_REJECT_WALK_STATE);
        }
        let data = fragment
            .Cursor()
            .PrePaintData::<PrePaintFragmentData>()
            .filter(|data| data.WasWalked())
            .ok_or(CLEAN_REJECT_RECORD_GEOMETRY)?;
        let record = data.WalkRecord();
        if record.link_offset != fragment.Cursor().ViewOffsetInContainer()
            || record.size != fragment.ViewSize()
            || record.display_item_fragment != display_item_fragment
            || record.fragmentainer != fragmentainer
            // has_non_composited_transform: the infinite-cull decision reads
            // the PaintLayer's previous paint result.
            || record.cull_decision_inputs[3]
        {
            return Err(CLEAN_REJECT_RECORD_GEOMETRY);
        }
        let owner = WalkedOwner(fragment.Cursor());
        let owner_replaced = record.walked_owner != owner;
        // Only a box fragment's FragmentData is the LayoutObject's across
        // relayout (a FragmentItem's is keyed on its inline container).
        if owner == (0, 0)
            || (owner_replaced && !(is_box && owner.1 == 0 && record.walked_owner.1 == 0))
        {
            return Err(CLEAN_REJECT_RECORD_OWNER);
        }
        let cached = data
            .CachedChildContext()
            .filter(|cached| cached.generation == self.child_context_generation);
        // A leaf kept no child context (its last build walked no child); its
        // own values are current while that build's generation is, and the
        // subpixel movement a rebuild would reset is the one it inherits.
        let leaf = cached.is_none()
            && !ctx.inherited.root
            && record.context_generation == self.child_context_generation
            && OrdinaryPaintSourceChildren(*fragment).next().is_none();
        if cached.is_none() && !leaf {
            return Err(CLEAN_REJECT_CACHED_CONTEXT);
        }
        let delta = cached.map_or(
            ctx.inherited
                .directly_composited_container_paint_offset_subpixel_delta,
            |cached| {
                cached
                    .context
                    .directly_composited_container_paint_offset_subpixel_delta
            },
        );
        if delta.x != 0.0 || delta.y != 0.0 {
            return Err(CLEAN_REJECT_SUBPIXEL_DELTA);
        }
        // A replaced or relaid-out box keeps the values its last build
        // derived from its geometry only while that geometry is the one its
        // last paint invalidation saved (SavePreviousBoxGeometriesIfNeeded).
        if (owner_replaced || invalidation.should_check_layout_for_paint_invalidation)
            && !BoxGeometryMatchesPrevious(fragment)
        {
            return Err(CLEAN_REJECT_RECORD_GEOMETRY);
        }
        // PaintInvalidator::UpdateContext: whether InvalidatePaint runs for
        // the object. A table cell's invalidation also propagates its
        // geometry change to the row and table (CellGeometryChanged), which
        // the build path runs.
        let mut probe = ctx.invalidation;
        let invalidate = PaintInvalidator::UpdateContext(fragment, &mut probe);
        if invalidate && fragment.paint.Display() == Display::kTableCell {
            return Err(CLEAN_REJECT_WALK_STATE);
        }
        Ok((cached.map(|cached| cached.context.clone()), invalidate))
    }

    /// Source: PrePaintTreeWalk::WalkInternal and WalkChildren
    /// (pre_paint_tree_walk.cc:650-760,1517-1582) for an object admitted by
    /// CleanVisitChildContext: the builder's context-only path. No property
    /// node, PaintOffset, LocalBorderBoxProperties, cull rect or flat replay
    /// value of the object changes and no PaintTreeNode is built; the
    /// painting layer and the paint phase needs of its subtree are updated,
    /// PaintInvalidator::InvalidatePaint runs on the unchanged PaintOffset
    /// when `invalidate` (UpdateContext admitted it), and the children are
    /// walked with `base`.
    #[allow(clippy::too_many_arguments)]
    fn WalkCleanObject(
        &mut self,
        fragment: PaintFragmentSource<'_>,
        ctx: WalkContext,
        mut base: Option<ChildContext>,
        invalidate: bool,
        inside: bool,
        display_item_fragment: u32,
        fragmentainer: u64,
        t_start: Option<std::time::Instant>,
    ) -> VisitResult {
        let profile = t_start.is_some();
        let checkpoint = || profile.then(std::time::Instant::now);
        self.stats.clean_visits += 1;
        let cursor = fragment.Cursor();
        let mut invalidation_context = ctx.invalidation;
        // The subtree flags and the painting layer advance for the children;
        // CleanVisitChildContext already probed whether InvalidatePaint runs.
        let _ = PaintInvalidator::UpdateContext(&fragment, &mut invalidation_context);
        PaintInvalidator::UpdatePaintingLayer(cursor, &mut invalidation_context);
        self.stats.invalidation_nodes += invalidate as usize;
        // The first step of every build (BuildNode).
        fragment
            .paint
            .UpdateSourcePaintLayerDescendantDependentFlags();
        let (record, paint_offset, paint_snap_offset, old_children_contents_cull, old_gap_rules) = {
            let data = cursor
                .PrePaintData::<PrePaintFragmentData>()
                .expect("a clean visit has FragmentData");
            let flat = data.FlatReplay();
            (
                *data.WalkRecord(),
                flat.paint_offset,
                flat.paint_snap_offset,
                Some(StoredCullRects(data, ctx.old_parent_contents_cull).1),
                if invalidate {
                    data.PreviousGapRules().cloned()
                } else {
                    None
                },
            )
        };
        // The values of the context a build takes from the walk rather than
        // from the object: the wheel-handler state (WalkChildren) and the
        // accumulated opacity, which no digest covers.
        // A leaf (`base` None) hands no child a context.
        if let Some(base) = base.as_mut() {
            base.inside_blocking_wheel_event_handler = inside;
            base.blocking_wheel_event_handler_changed =
                ctx.inherited.blocking_wheel_event_handler_changed;
            base.opacity = ctx.inherited.opacity
                * if fragment.ViewEstablishesPaintState() && fragment.paint.HasSource() {
                    MaskAdjustedOpacity(&fragment.paint.style)
                } else {
                    1.0
                };
            if ctx.inherited.root {
                self.CaptureRootFixedPositionContext(base, None);
            }
        }
        let mut owner = ResidentFragment::FromSource(&fragment);
        let t_admitted = checkpoint();
        self.stats.profile_admission_ns += ProfileNs(t_start, t_admitted);
        if invalidate {
            // The PaintOffset the builder would recompute is the stored one:
            // the context, link offset and size are those of the last build.
            self.InvalidatePaintForVisit(
                fragment,
                &mut owner,
                old_gap_rules.as_deref(),
                Some(paint_offset),
                paint_offset,
                invalidation_context,
            );
        }
        if InlineOutlineCandidate(&fragment) {
            self.visited_inline_outline = true;
        }
        let t_invalidated = checkpoint();
        self.stats.profile_invalidation_ns += ProfileNs(t_admitted, t_invalidated);
        let child_auto_dark_mode_paused = ApplyAutoDarkModeState(
            &mut owner,
            paint_offset,
            paint_snap_offset,
            cursor.DocumentResources(),
            ctx.auto_dark_mode_paused,
        );
        let mut phase = OwnPhaseNeeds(&fragment, record.is_paint_layer);
        let mut table_cell_geometry_changed = false;
        if let Some(base) = base.as_ref() {
            self.WalkChildren(
                fragment,
                &mut owner,
                paint_offset,
                base,
                invalidation_context,
                false,
                false,
                old_children_contents_cull,
                child_auto_dark_mode_paused,
                display_item_fragment,
                fragmentainer,
                &mut phase,
                &mut table_cell_geometry_changed,
                t_invalidated,
            );
        }
        let t_looped = checkpoint();
        if let Some(slot) = cursor.PrePaintSlot() {
            // SAFETY: the walk is the occurrence's single PrePaint writer
            // here; no reference into the slot is live.
            let data = unsafe { slot.Ensure::<PrePaintFragmentData>() };
            // The FragmentData now describes the fragment this visit walked
            // (a replacement LayoutResult's box with unchanged geometry).
            let walked_owner = WalkedOwner(cursor);
            if record.walked_owner != walked_owner {
                data.SetWalkRecord(PrePaintWalkRecord {
                    walked_owner,
                    ..record
                });
            }
            let flat = data.FlatReplayMut();
            flat.skip_float_phase = !phase.float;
            flat.skip_outline_phase = !phase.outline;
            flat.skip_overflow_controls_phase = !phase.overflow_controls;
            data.SetPreviousGapRules(fragment.ViewGapRules());
            if let (Some(cached), Some(base)) = (data.CachedChildContextMut(), base.as_ref()) {
                cached.context.opacity = base.opacity;
                cached.context.inside_blocking_wheel_event_handler = inside;
            }
            self.RegisterCommittedClients(cursor, &*fragment, data);
        }
        self.deferred_paint_flag_clears
            .push(&*fragment as *const FragmentPaintData);
        self.stats.profile_tail_ns += ProfileNs(t_looped, checkpoint());
        VisitResult {
            table_cell_geometry_changed,
            is_paint_layer: record.is_paint_layer,
            phase,
        }
    }

    /// Source: PrePaintTreeWalk::WalkChildren (pre_paint_tree_walk.cc:
    /// 1517-1582): the children of the visited object `fragment` (and the
    /// LayoutView of an embedded local frame it owns) are walked with the
    /// context `base` it hands them, or returned from before their own
    /// context is built (CanSkipSubtree). `owner` is the object's resident
    /// fragment, which table cell invalidation updates. Returns whether the
    /// object has a child.
    #[allow(clippy::too_many_arguments)]
    fn WalkChildren(
        &mut self,
        fragment: PaintFragmentSource<'_>,
        owner: &mut ResidentFragment,
        owner_paint_offset: Offset,
        base: &ChildContext,
        invalidation_context: PaintInvalidatorContext,
        force_children: bool,
        children_context_changed: bool,
        old_children_contents_cull: Option<Option<PaintRect>>,
        child_auto_dark_mode_paused: bool,
        display_item_fragment: u32,
        fragmentainer: u64,
        phase: &mut PhaseNeeds,
        table_cell_geometry_changed: &mut bool,
        t_setup: Option<std::time::Instant>,
    ) -> bool {
        let profile = t_setup.is_some();
        let checkpoint = || profile.then(std::time::Instant::now);
        let client = fragment.paint.DisplayItemClientId();
        let embedded_view = self
            .local_frame_views
            .get(&client)
            .filter(|_| fragment.ViewCanPaintFragmentOccurrence())
            .cloned();
        let embedded_source = embedded_view
            .as_ref()
            .map(|view| PaintFragmentSource::FromCursor(view.root));
        let ordinary_children = OrdinaryPaintSourceChildren(fragment).count();
        self.physical_ancestor_ids.push(if fragment.ViewEstablishesPaintState() {
            fragment.ViewNodeId()
        } else {
            0
        });
        self.walk_ancestors.push(client);
        if embedded_view.is_some() {
            self.next_local_frame_ancestors
                .extend(self.walk_ancestors.iter().copied());
        }
        let t_loop = checkpoint();
        self.stats.profile_children_setup_ns += ProfileNs(t_setup, t_loop);
        let mut recursion_ns = 0;
        for (index, child) in OrdinaryPaintSourceChildren(fragment)
            .chain(embedded_source)
            .enumerate()
        {
            let child = child.Resolved();
            let is_local_frame = index == ordinary_children && embedded_view.is_some();
            // A child paints in this object's scope unless it opens its own;
            // an embedded frame's root starts its document's walk at 0.
            let child_display_item_fragment = child
                .ViewOwnDisplayItemFragment()
                .unwrap_or(if is_local_frame { 0 } else { display_item_fragment });
            let child_fragmentainer = child
                .ViewOwnFragmentainerInstanceId()
                .unwrap_or(if is_local_frame { 0 } else { fragmentainer });
            if !is_local_frame {
                // pre_paint_tree_walk.cc:1477-1482: the child's walk returns
                // before it builds the child's context.
                let child = unsafe { child.ReadPrePaintInvalidationForVisit() };
                let stored = child.Cursor().PrePaintData::<PrePaintFragmentData>();
                let verdict = match stored {
                    Some(data) => self.CanSkipSubtree(
                        &child,
                        data,
                        force_children,
                        children_context_changed,
                        base,
                        &invalidation_context,
                        child_display_item_fragment,
                        child_fragmentainer,
                    ),
                    None => Err(SKIP_REJECT_NO_DATA),
                };
                let skipped = match verdict {
                    Ok(()) => stored,
                    Err(reason) => {
                        self.stats.skip_rejects[reason] += 1;
                        None
                    }
                };
                if let Some(data) = skipped {
                    self.stats.skipped_subtrees += 1;
                    PropagateAutoDarkModeOverCursor(child.Cursor(), child_auto_dark_mode_paused);
                    let result = VisitResult::Stored(data);
                    if !result.is_paint_layer {
                        phase.Merge(result.phase);
                    }
                    continue;
                }
            }
            let child_inherited = if is_local_frame {
                self.ContextForFrame(
                    &fragment,
                    owner_paint_offset,
                    base,
                    embedded_view.as_ref().unwrap(),
                )
            } else {
                self.ContextForChild(&child, base)
            };
            let leading_svg_matrix = base.svg;
            let child_context = WalkContext {
                inherited: child_inherited,
                invalidation: invalidation_context,
                force_subtree: force_children,
                // An embedded frame's LayoutView has no previous context of
                // its owner's to compare with.
                context_changed: is_local_frame || children_context_changed,
                old_parent_contents_cull: old_children_contents_cull,
                auto_dark_mode_paused: child_auto_dark_mode_paused,
                leading_svg_matrix,
            };
            let frame_fixed_context = is_local_frame.then(|| self.root_fixed_position.take());
            let previous_wheel_targets = is_local_frame.then(|| {
                std::mem::replace(
                    &mut self.active_blocking_wheel_targets,
                    embedded_view
                        .as_ref()
                        .unwrap()
                        .blocking_wheel_targets
                        .clone(),
                )
            });
            self.local_frame_depth += u32::from(is_local_frame);
            self.display_item_fragment_scope = Some(child_display_item_fragment);
            self.fragmentainer_scope = Some(child_fragmentainer);
            let t_child = checkpoint();
            let result = self.WalkObject(child, child_context);
            recursion_ns += ProfileNs(t_child, checkpoint());
            self.local_frame_depth -= u32::from(is_local_frame);
            if let Some(previous) = previous_wheel_targets {
                self.active_blocking_wheel_targets = previous;
            }
            if is_local_frame {
                self.completed_local_frame_prepaint.insert(client);
            }
            if let Some(fixed) = frame_fixed_context {
                self.root_fixed_position = fixed;
            }
            if result.table_cell_geometry_changed {
                *table_cell_geometry_changed =
                    TableCellPaintInvalidator::InvalidateAncestorAndPropagate(&mut **owner);
            }
            if !result.is_paint_layer {
                phase.Merge(result.phase);
            }
        }
        self.walk_ancestors.pop();
        self.physical_ancestor_ids.pop();
        self.stats.profile_child_loop_ns +=
            ProfileNs(t_loop, checkpoint()).saturating_sub(recursion_ns);
        ordinary_children != 0 || embedded_view.is_some()
    }

    /// The end of one visit: the occurrence's flat replay values, walk record,
    /// the context its children were walked with and the LayoutBox gap
    /// baseline go to its FragmentData (the builder already stored
    /// PaintOffset, ObjectPaintProperties and LocalBorderBoxProperties), and
    /// the object's consumed paint flags are cleared after the walk
    /// (pre_paint_tree_walk.cc:1584-1585: once every fragment read them;
    /// a display-locked object keeps its descendant bits).
    fn FinishVisit(
        &mut self,
        node: &mut PaintTreeNode<'static>,
        fragment: PaintFragmentSource<'_>,
        record: PrePaintWalkRecord,
        phase: PhaseNeeds,
        table_cell_geometry_changed: bool,
        child_context: Option<Box<CachedChildContext>>,
    ) -> VisitResult {
        node.needs_float_phase = phase.float;
        node.needs_outline_phase = phase.outline;
        node.needs_overflow_controls_phase = phase.overflow_controls;
        if let Some(slot) = fragment.Cursor().PrePaintSlot() {
            // SAFETY: the walk is the occurrence's single PrePaint writer
            // here; no reference into the slot is live.
            let data = unsafe { slot.Ensure::<PrePaintFragmentData>() };
            WriteFlatReplayDataTo(node, data);
            data.SetWalkRecord(record);
            data.SetPreviousGapRules(fragment.ViewGapRules());
            data.SetCachedChildContext(child_context);
            self.RegisterCommittedClients(fragment.Cursor(), &*fragment, data);
        }
        self.deferred_paint_flag_clears
            .push(&*fragment as *const FragmentPaintData);
        VisitResult {
            table_cell_geometry_changed,
            is_paint_layer: node.is_paint_layer,
            phase,
        }
    }

    pub(crate) fn viewport_cull_for_special_paint(&self) -> Option<PaintRect> {
        self.viewport
    }
    pub(crate) fn viewport_contents_clip_for_special_paint(
        &self,
    ) -> std::sync::Arc<paint_model::paint_property_tree::ClipPaintPropertyNode> {
        self.root_fixed_position
            .as_ref()
            .expect("resident root fixed-position context")
            .properties
            .nodes
            .clip
            .clone()
    }
    pub(crate) fn viewport_properties_for_special_paint(
        &self,
    ) -> paint_model::paint_property_tree::PropertyTreeState {
        self.viewport_properties.nodes.clone()
    }
    /// Install current native views before entering the root lifecycle. Old
    /// leases stay live until the existing tree has been rebound and retained.
    pub fn SetLocalFrameViews(&mut self, views: impl IntoIterator<Item = EmbeddedLocalFrameView>) {
        let views: HashMap<_, _> = views
            .into_iter()
            .map(|view| (view.owner_client, view))
            .collect();
        self.local_frame_sources_changed |= views.len() != self.local_frame_views.len()
            || views.iter().any(|(owner, view)| {
                self.local_frame_views.get(owner).is_none_or(|old| {
                    !old.lease.PtrEq(&view.lease)
                        || old.cross_origin_to_parent != view.cross_origin_to_parent
                })
            });
        self.local_frame_views = views;
    }
    pub fn SetBlockingWheelEventHandlerTargets(&mut self, targets: Arc<[u64]>) {
        self.root_blocking_wheel_targets = targets;
    }

    pub fn LocalFramePrePaintCompleted(&self, owner_client: u64) -> bool {
        self.completed_local_frame_prepaint.contains(&owner_client)
    }

    pub fn LocalFrameViews(&self) -> impl Iterator<Item = &EmbeddedLocalFrameView> {
        self.local_frame_views.values()
    }

    /// The same-Page child LocalFrameView embedded by the object whose
    /// DisplayItemClient is `owner_client` (LayoutEmbeddedContent::
    /// ChildFrameView, as EmbeddedContentPainter reaches it).
    pub fn LocalFrameView(&self, owner_client: u64) -> Option<&EmbeddedLocalFrameView> {
        self.local_frame_views.get(&owner_client)
    }

    // Source: pre_paint_tree_walk.cc:1538-1575 and
    // PaintPropertyTreeBuilder::SetupContextForFrame, source.cc:192-226.
    fn ContextForFrame(
        &self,
        owner: &FragmentPaintData,
        owner_paint_offset: Offset,
        base: &ChildContext,
        view: &EmbeddedLocalFrameView,
    ) -> InheritedContext {
        let replaced = owner.ViewReplacedContent().unwrap();
        let mut properties = base.properties.clone();
        if view.cross_origin_to_parent {
            let candidate = paint_model::property_cell::with_access(|access| {
                let mut effect = Some(&properties.nodes.effect);
                let mut candidate = None;
                while let Some(current) = effect {
                    if current.HasReferenceFilter(access) {
                        candidate = current.Parent(access).cloned();
                    }
                    effect = current.Parent(access);
                }
                candidate
            });
            if let Some(effect) = candidate {
                properties.effect_id = effect.id;
                properties.nodes.effect = effect;
            }
        }
        InheritedContext {
            inside_blocking_wheel_event_handler: base.inside_blocking_wheel_event_handler,
            blocking_wheel_event_handler_changed: base.blocking_wheel_event_handler_changed,
            offset: Offset {
                x: (owner_paint_offset.x + replaced.offset.x).round() + base.paint_space.x,
                y: (owner_paint_offset.y + replaced.offset.y).round() + base.paint_space.y,
            },
            paint_space: base.paint_space,
            clip: base.clip,
            cull: base.cull,
            transforms: base.transforms.clone(),
            opacity: base.opacity,
            scroll: Offset::default(),
            fixed_background_acts_as_scroll: false,
            properties,
            canvas_contents: None,
            rendering_context_id: 0,
            should_flatten_inherited_transform: true,
            can_inherit_backface_visibility: true,
            directly_composited_container_paint_offset_subpixel_delta: Offset::default(),
            subtree_should_use_infinite_cull_rect: base.subtree_should_use_infinite_cull_rect,
            root: true,
            flex_or_grid_item: false,
        }
    }

    /// `root` is the walk root's build; a root that took the context-only
    /// path (only the top document's, see CleanVisitChildContext) has none.
    fn CaptureRootFixedPositionContext(
        &mut self,
        base: &ChildContext,
        root: Option<&PaintTreeNode<'_>>,
    ) {
        // Blink paint_property_tree_builder.cc UpdateForChildren creates the
        // LayoutView OverflowClip before UpdateOutOfFlowContext. The latter
        // copies the current state, restoring only the pre-scroll transform.
        // Keep the resolved scrollbar-excluding clip and effect, rather than
        // replacing them with the full host viewport or a nested box's clip.
        // Capture on both traversal paths so retained geometry and scrollbar
        // add/remove/resize updates use the same root-owned context.
        // Source: core/paint/paint_property_tree_builder.cc:3837-3854. Keep
        // LayoutView's actual ScrollTranslation while restoring the fixed
        // context's pre-scroll transform; this non-owning source-node reference
        // is represented by an immutable source Arc lease.
        let scroll_translation = paint_model::property_cell::with_access(|access| {
            let mut scroll_translation = Some(&base.properties.nodes.transform);
            while let Some(node) = scroll_translation {
                if node.ScrollNode(access).is_some() && node.id != 0 {
                    break;
                }
                scroll_translation = node.Parent(access);
            }
            scroll_translation.cloned()
        });
        let mut properties = base.properties.clone();
        let (offset, paint_space, transforms) = if self.local_frame_depth != 0 {
            let root = root.expect("an embedded frame's LayoutView is built");
            // Fixed descendants are rooted in this LocalFrame's pre-scroll
            // LayoutView state, not the top document's visual viewport.
            properties.transform_id = root.properties.transform_id;
            properties.nodes.transform = root.properties.nodes.transform.clone();
            (
                root.absolute_offset,
                Offset {
                    x: root.absolute_offset.x - root.paint_offset.x,
                    y: root.absolute_offset.y - root.paint_offset.y,
                },
                root.transforms.clone(),
            )
        } else {
            properties.transform_id = self.viewport_properties.transform_id;
            properties.nodes.transform = self.viewport_properties.nodes.transform.clone();
            (Offset::default(), Offset::default(), FlatTransforms::default())
        };
        self.root_fixed_position = Some(RootFixedPositionContext {
            offset,
            paint_space,
            transforms,
            clip: base.clip,
            properties,
            scroll_translation,
        });
    }

    fn ContextForChild(
        &self,
        child: &PaintFragmentSource<'_>,
        base: &ChildContext,
    ) -> InheritedContext {
        // cpp: paint_property_tree_builder.cc:4519-4532.
        if child.paint.IsSVGHiddenContainer() {
            return InheritedContext {
                inside_blocking_wheel_event_handler: base.inside_blocking_wheel_event_handler,
                blocking_wheel_event_handler_changed: base.blocking_wheel_event_handler_changed,
                offset: Offset::default(),
                paint_space: Offset::default(),
                clip: None,
                cull: None,
                transforms: FlatTransforms::default(),
                opacity: 1.,
                scroll: Offset::default(),
                fixed_background_acts_as_scroll: false,
                properties: PropertyTreeState::default(),
                canvas_contents: None,
                rendering_context_id: 0,
                should_flatten_inherited_transform: true,
                can_inherit_backface_visibility: true,
                directly_composited_container_paint_offset_subpixel_delta: Offset::default(),
                subtree_should_use_infinite_cull_rect: true,
                root: false,
                flex_or_grid_item: false,
            };
        }
        if child.paint.FixedToView() && child.ViewEstablishesPaintState() {
            // Blink switches fixed-to-view descendants to the LayoutView's
            // fixed-position context. Its transform is above document scroll,
            // but its clip still excludes classic root scrollbar gutters.
            let fixed = self
                .root_fixed_position
                .as_ref()
                .expect("root fixed-position context must precede descendants");
            let properties = fixed.properties.clone();
            return InheritedContext {
                inside_blocking_wheel_event_handler: base.inside_blocking_wheel_event_handler,
                blocking_wheel_event_handler_changed: base.blocking_wheel_event_handler_changed,
                offset: fixed.offset,
                paint_space: fixed.paint_space,
                clip: fixed.clip,
                cull: fixed.clip,
                transforms: fixed.transforms.clone(),
                opacity: base.opacity,
                scroll: Offset::default(),
                fixed_background_acts_as_scroll: base.fixed_background_acts_as_scroll,
                properties,
                canvas_contents: base.canvas_contents.clone(),
                rendering_context_id: 0,
                should_flatten_inherited_transform: true,
                can_inherit_backface_visibility: true,
                directly_composited_container_paint_offset_subpixel_delta: Offset::default(),
                subtree_should_use_infinite_cull_rect: base.subtree_should_use_infinite_cull_rect,
                root: false,
                flex_or_grid_item: base.flex_or_grid_items,
            };
        }
        let properties = base.properties.clone();
        InheritedContext {
            inside_blocking_wheel_event_handler: base.inside_blocking_wheel_event_handler,
            blocking_wheel_event_handler_changed: base.blocking_wheel_event_handler_changed,
            offset: ResolveChildGeometryOrigin(
                child,
                child.Cursor().ViewOffsetInContainer(),
                base.origin,
                base.paint_space,
            ),
            paint_space: base.paint_space,
            clip: base.clip,
            cull: base.cull,
            transforms: base.transforms.clone(),
            opacity: base.opacity,
            scroll: base.scroll,
            fixed_background_acts_as_scroll: base.fixed_background_acts_as_scroll,
            properties,
            canvas_contents: base.canvas_contents.clone(),
            rendering_context_id: base.rendering_context_id,
            should_flatten_inherited_transform: base.should_flatten_inherited_transform,
            can_inherit_backface_visibility: base.can_inherit_backface_visibility,
            directly_composited_container_paint_offset_subpixel_delta: base
                .directly_composited_container_paint_offset_subpixel_delta,
            subtree_should_use_infinite_cull_rect: base.subtree_should_use_infinite_cull_rect,
            root: false,
            flex_or_grid_item: base.flex_or_grid_items,
        }
    }
    /// The ObjectPaintProperties slot `role` of the fragment being built: its
    /// resident node id (or a new one) for the `Update*Property` that
    /// follows. A fragment without a source DisplayItemClient has no slot.
    fn RegisterProperty(&mut self, fragment: &FragmentPaintData, role: PaintPropertyRole) -> u64 {
        let client = fragment.paint.DisplayItemClientId();
        if client == 0 {
            return self.property_node_store.SlotId(None);
        }
        // BuildNode runs while UpdateNode visits this fragment's node, so the
        // walk's scope is the node's ScopedDisplayItemFragment.
        let owner = PaintPropertyOwner::Fragment {
            client,
            fragment: self
                .display_item_fragment_scope
                .unwrap_or_else(|| fragment.ViewDisplayItemFragment()),
        };
        // Chromium client creation is the lifetime boundary, including
        // allocator address reuse. The owner key is the process-unique client
        // id (DisplayItemClient::Id), never a recycled pointer, so slots the
        // store holds for it are this same client's own.
        let key = PaintPropertyKey { owner, role };
        let id = self.property_node_store.SlotId(Some(key));
        self.property_keys.insert(id, key);
        self.own_key_log.push((id, key));
        id
    }

    /// The slot key `RegisterProperty` gave the node id `id` in this pass.
    fn PropertySlotKey(&self, id: u64) -> Option<PaintPropertyKey> {
        self.property_keys.get(&id).copied()
    }

    /// Decision D2. Source: paint_property_tree_builder.cc, where
    /// `context.current_effect` follows the LayoutObject tree while the
    /// containing-block contexts carry an out-of-flow object's transform and
    /// clip. Rechrom walks physical fragments, so an absolute or fixed
    /// object whose logical parent occurrence is not a physical ancestor
    /// takes that occurrence's recorded effect state instead of its physical
    /// parent's.
    fn LogicalOutOfFlowEffectContext(
        &self,
        fragment: &FragmentPaintData,
    ) -> Option<LogicalEffectContext> {
        let position = fragment.paint.Position();
        if position != Position::kAbsolute && position != Position::kFixed {
            return None;
        }
        let logical_id = fragment.paint.LogicalParentNodeId()?;
        if self.physical_ancestor_ids.contains(&logical_id) {
            return None;
        }
        self.LogicalEffectContextFor(fragment, logical_id, self.fragmentainer_scope.unwrap_or(0))
    }

    /// The logical effect context the occurrence `logical_id` gives an
    /// out-of-flow child inside `fragmentainer`: its only recorded
    /// occurrence, preferring the child's fragmentainer.
    fn LogicalEffectContextFor(
        &self,
        fragment: &FragmentPaintData,
        logical_id: u64,
        fragmentainer: u64,
    ) -> Option<LogicalEffectContext> {
        let candidates = self.logical_effect_contexts.get(&logical_id)?;
        let own = fragment.ViewFragmentInstanceId();
        let unique = |same_fragmentainer: bool| {
            let mut matches = candidates.iter().filter(|candidate| {
                (own == 0 || candidate.fragment_instance_id != own)
                    && (!same_fragmentainer || candidate.fragmentainer == fragmentainer)
            });
            let first = matches.next()?;
            matches.next().is_none().then(|| first.clone())
        };
        unique(true).or_else(|| unique(false))
    }
}

macro_rules! update_property {
    ($name:ident, $node:ty, $update:ident) => {
        impl PrePaintTreeWalk {
            /// Source: paint_property_tree_builder.cc `OnUpdate*` around
            /// ObjectPaintProperties::Update*: updates the registered slot
            /// of `node.id` in place (or creates it) and records the change
            /// for the object being built.
            pub(crate) fn $name(&mut self, node: $node) -> Arc<$node> {
                let key = self.PropertySlotKey(node.id);
                let store = &mut self.property_node_store;
                let (node, change) = paint_model::property_cell::with_writer(|writer| {
                    store.$update(writer, key, node)
                });
                self.own_change_log.push(change);
                node
            }
        }
    };
}
update_property!(UpdateTransformProperty, TransformPaintPropertyNode, UpdateTransform);
update_property!(UpdateClipProperty, ClipPaintPropertyNode, UpdateClip);
update_property!(UpdateEffectProperty, EffectPaintPropertyNode, UpdateEffect);
update_property!(UpdateScrollProperty, ScrollPaintPropertyNode, UpdateScroll);

impl PrePaintTreeWalk {
    /// Source: PrePaintTreeWalk::WalkTree (pre_paint_tree_walk.cc:82-136).
    ///
    /// PrePaint runs in DocumentLifecycle::kInPrePaint, where no LayoutObject
    /// is created or destroyed; the walk, the builder and the invalidators
    /// read each visited object's LayoutObject and ComputedStyle directly.
    /// Open the same resident-source scope the paint tree walk opens
    /// (LocalFrameView::PaintTree) for the whole pass. The objects whose
    /// transform change PaintLayer::StyleDidChange deferred
    /// (LocalFrameView::ExecuteAllPendingUpdates, local_frame_view.cc:
    /// 5498-5525) have their Transform node updated in place before the
    /// walk (DirectlyUpdateTransformMatrix); one the direct path cannot
    /// update is built by the walk from the current style.
    pub fn Walk(&mut self, root: PaintFragmentSource<'_>, source_lease: PaintFragmentLease) {
        let _resident_sources = layout::paint::ResidentPaintSourceScope::new();
        self.WalkTree(root, source_lease);
    }

    fn WalkTree(&mut self, root: PaintFragmentSource<'_>, source_lease: PaintFragmentLease) {
        // Property changes up to here are not the walk's own (see
        // root_context_changed below).
        let walk_start_epoch = paint_model::property_cell::PropertyChangeEpoch();
        self.active_blocking_wheel_targets = self.root_blocking_wheel_targets.clone();
        // Layout keeps its Paint publication state across a replacement
        // LayoutResult (LayoutPaintState::AdoptReplacementResult), as Chromium
        // keeps FragmentData on the LayoutObject across relayout
        // (core/layout/layout_box.cc:2523-2561,2620-2661). A different state
        // has fresh FragmentData: every object is visited.
        let (source_owner_changed, source_result_changed) =
            match self.retained_source_lease.as_ref() {
                Some(retained) => (
                    !retained.SameSidecarOwner(&source_lease),
                    !retained.PtrEq(&source_lease),
                ),
                None => (true, true),
            };
        let source_owner_changed = source_owner_changed || self.local_frame_sources_changed;
        self.source_owner_changed = source_owner_changed;
        self.source_result_changed = source_result_changed || source_owner_changed;
        if source_owner_changed {
            // The committed-client directory names occurrences of the
            // previous state's FragmentData; this walk visits every object
            // and registers them again.
            self.committed_clients.Clear();
        }
        self.pending_source_lease = Some(source_lease);
        // The lease above keeps this root alive until CommitWalk retains it.
        self.pending_root = Some(unsafe {
            std::mem::transmute::<PaintFragmentCursor<'_>, PaintFragmentCursor<'static>>(
                root.Cursor(),
            )
        });
        self.completed_local_frame_prepaint.clear();
        self.paint_invalidator.BeginPass();
        // Source: LocalFrameView::ExecuteAllPendingUpdates. Taking the set
        // makes each deferred object's style current on its sidecars; the
        // updates run once the walk's environment is known (below).
        self.must_visit_clients.clear();
        self.must_build_clients.clear();
        let pending_transform_updates = root.Cursor().TakePendingTransformUpdateClientChains();
        self.must_visit_clients
            .extend(self.local_frame_ancestors.iter().copied());
        self.previous_display_lock_walks = std::mem::take(&mut self.pending_display_lock_walks);
        for (client, ancestors) in &self.previous_display_lock_walks {
            self.must_visit_clients.insert(*client);
            self.must_visit_clients.extend(ancestors.iter().copied());
        }
        self.next_local_frame_ancestors.clear();
        self.walk_ancestors.clear();
        let root = unsafe { root.ReadPrePaintInvalidationForVisit() };
        self.requires_main_thread_for_background_attachment_fixed = root
            .paint
            .RequiresMainThreadScrollingForBackgroundAttachmentFixed();
        self.stats = PrePaintUpdateStats::default();
        self.stats.source_result_replaced = self.source_result_changed && !source_owner_changed;
        self.transform_world_memo.clear();
        self.intersection_observation_state = Default::default();
        // The slots registered by this pass's builds (RegisterProperty); the
        // builders update the resident slot nodes in place. Objects the walk
        // does not visit keep their FragmentData's ObjectPaintProperties.
        self.property_keys.clear();
        self.released_property_keys.clear();
        self.own_key_log.clear();
        self.own_change_log.clear();
        self.logical_effects_changed.clear();
        self.visited_inline_outline = false;
        self.property_node_store.BeginUpdate();
        let viewport = ArtifactViewport(root.Cursor().DocumentResources().map(|resources| &**resources));
        let environment_changed = viewport != self.viewport;
        let viewport_clip_key = PaintPropertyKey {
            owner: PaintPropertyOwner::Viewport,
            role: PaintPropertyRole::ViewportClip,
        };
        if environment_changed || (viewport.is_some() && self.viewport_properties.clip_id == 0) {
            if viewport.is_none() {
                // The frame view no longer has a viewport clip.
                self.released_property_keys.push(viewport_clip_key);
            }
            self.viewport = viewport;
            // Blink updates the viewport-owned clip node in place. Its value
            // changes as the window resizes, but the property slot remains the
            // same node so descendants keep their clip ancestry.
            self.viewport_properties = PropertyTreeState::default();
            if let Some(rect) = viewport {
                let id = self.property_node_store.SlotId(Some(viewport_clip_key));
                let clip =
                    ClipPaintPropertyNode::new(Default::default(), id, paint_model::paint_property_tree::ClipPaintPropertyNodeState {
                        layout_clip_rects: Default::default(),
                        parent: Some(self.viewport_properties.nodes.clip.clone()),
                        local_transform_space: PropertyTreeState::default().nodes.transform,
                        rect: Some(rect),
                        radii: PaintCornerRadii::default(),
                        clip_path: None,
                        clip_path_error: None,
                        pixel_moving_filter: None,
                    })
                    .with_layout_clips();
                let store = &mut self.property_node_store;
                let (clip, _) = paint_model::property_cell::with_writer(|writer| {
                    store.UpdateClip(writer, Some(viewport_clip_key), clip)
                });
                self.viewport_properties.clip_id = clip.id;
                self.viewport_properties.nodes.clip = clip;
            }
        }
        // FragmentData cull rectangles live in their corresponding property
        // state. The initial viewport rectangle is in viewport-properties
        // space; CullRectUpdater maps it once at each descendant boundary.
        let visible_root_cull = viewport;
        if self.viewport_properties.clip_id != 0 {
            self.property_keys
                .insert(self.viewport_properties.clip_id, viewport_clip_key);
        }
        // Source: PrePaintTreeWalk::WalkTree and
        // VisualViewportPaintPropertyTreeBuilder::Update. Chromium updates
        // the visual viewport before walking the LayoutView, then starts the
        // current, absolute and fixed transform contexts at the visual
        // viewport scroll translation.
        if let Some(viewport) = viewport {
            let (scroll_offset, visual_state) =
                (self.visual_viewport_scroll_offset, self.visual_viewport_state);
            let store = &mut self.property_node_store;
            let (_, _, visual_scroll_translation) =
                paint_model::property_cell::with_writer(|writer| {
                    store.ensure_single_layout_viewport_nodes(
                        writer,
                        viewport,
                        scroll_offset,
                        visual_state,
                    )
                });
            self.viewport_properties.transform_id = visual_scroll_translation.id;
            self.viewport_properties.nodes.transform = visual_scroll_translation;
        }
        // A document resource change (an image or font that loaded, a new
        // device scale) reaches every object that paints with it; Chromium
        // marks those objects (ImageChanged, SetNeedsPaintPropertyUpdate).
        // PrePaint compares the catalogs and visits every object instead.
        let resources = root.Cursor().DocumentResources().cloned();
        let mut resources_changed = match (&self.last_document_resources, &resources) {
            (None, None) => false,
            (Some(a), Some(b)) if Arc::ptr_eq(a, b) => false,
            (Some(a), Some(b)) => !layout::paint::SamePaintResourcesFor(a, b, u8::MAX),
            _ => true,
        };
        self.last_document_resources = resources;
        let frame_resources: Vec<(u64, Option<Arc<PaintResources>>)> = self
            .local_frame_views
            .iter()
            .map(|(owner, view)| (*owner, view.root.DocumentResources().cloned()))
            .collect();
        for (owner, current) in frame_resources {
            let changed = match (self.last_frame_resources.get(&owner), &current) {
                (None, None) => false,
                (Some(a), Some(b)) if Arc::ptr_eq(a, b) => false,
                (Some(a), Some(b)) => !layout::paint::SamePaintResourcesFor(a, b, u8::MAX),
                _ => true,
            };
            resources_changed |= changed;
            match current {
                Some(current) => {
                    self.last_frame_resources.insert(owner, current);
                }
                None => {
                    self.last_frame_resources.remove(&owner);
                }
            }
        }
        let local_frame_views = &self.local_frame_views;
        self.last_frame_resources
            .retain(|owner, _| local_frame_views.contains_key(owner));
        self.full_walk =
            !self.walked_once || source_owner_changed || environment_changed || resources_changed;
        // The walk root inherits the viewport nodes, which the visual
        // viewport update above changes in place; every build maps its cull
        // rects from them, so no earlier child context stays current.
        self.root_context_changed =
            paint_model::property_cell::PropertyChangeEpoch() != walk_start_epoch;
        if self.root_context_changed {
            self.child_context_generation = self.child_context_generation.wrapping_add(1);
        }
        // Source: LocalFrameView::RunPrePaintLifecyclePhase runs
        // ExecuteAllPendingUpdates before PrePaintTreeWalk::WalkTree.
        if !pending_transform_updates.is_empty() {
            self.ExecuteAllPendingUpdates(root.Cursor(), &pending_transform_updates);
        }
        let properties = self.viewport_properties.clone();
        let context = InheritedContext {
            inside_blocking_wheel_event_handler: false,
            blocking_wheel_event_handler_changed: false,
            offset: Offset::default(),
            paint_space: Offset::default(),
            clip: viewport,
            // Blink's CullRect::ChangedEnough keeps an expanded sliding
            // window until the visible viewport approaches its edge.
            cull: visible_root_cull,
            transforms: FlatTransforms::default(),
            opacity: 1.0,
            scroll: Offset::default(),
            fixed_background_acts_as_scroll: false,
            properties,
            canvas_contents: None,
            rendering_context_id: 0,
            should_flatten_inherited_transform: true,
            can_inherit_backface_visibility: true,
            directly_composited_container_paint_offset_subpixel_delta: Offset::default(),
            subtree_should_use_infinite_cull_rect: false,
            root: true,
            flex_or_grid_item: false,
        };
        // LayoutView::BackgroundRect() and PreviousBackgroundRect() read
        // once per walk, as ComputeViewBackgroundInvalidation reads them
        // (box_paint_invalidator.cc:257-259).
        let root_data: &FragmentPaintData = &root;
        let view_background_rects = ViewBackgroundRects::Read(root_data);
        self.view_background_rects = Some((
            root_data as *const FragmentPaintData as usize,
            view_background_rects,
        ));
        self.view_background_geometry_changed =
            BoxPaintInvalidator::ViewBackgroundGeometryChangedWithRects(view_background_rects, None);
        // LayoutView's own scope (its layer fragment index); a publication
        // walk starts at 0. The document root is inside no fragmentainer
        // unless it is one.
        let root_display_item_fragment = root.ViewOwnDisplayItemFragment().unwrap_or(0);
        let root_fragmentainer = root.ViewOwnFragmentainerInstanceId().unwrap_or(0);
        // Source: pre_paint_tree_walk.cc:418-437. With no dirty bit on the
        // LayoutView and none below it, no object requires PrePaint.
        let entirely_clean = !self.full_walk
            && self.local_frame_views.is_empty()
            && self.must_visit_clients.is_empty()
            && !root.pre_paint_invalidation.own()
            && !root.pre_paint_invalidation.descendants()
            && root
                .Cursor()
                .PrePaintData::<PrePaintFragmentData>()
                .is_some_and(|data| {
                    let record = data.WalkRecord();
                    record.walked
                        && record.walked_owner == WalkedOwner(root.Cursor())
                        && record.display_item_fragment == root_display_item_fragment
                        && record.fragmentainer == root_fragmentainer
                        && record.size == root.ViewSize()
                });
        if entirely_clean {
            self.pending_display_lock_walks = std::mem::take(&mut self.previous_display_lock_walks);
            self.view_background_rects = None;
            self.stats.skipped_subtrees = 1;
            self.next_property_id = self.property_node_store.next_id();
            return;
        }
        self.display_item_fragment_scope = Some(root_display_item_fragment);
        self.fragmentainer_scope = Some(root_fragmentainer);
        // This walk visits objects and clears their paint flags.
        self.consuming_walks = self.consuming_walks.wrapping_add(1);
        self.WalkObject(
            root,
            WalkContext {
                inherited: context,
                invalidation: PaintInvalidatorContext::default(),
                force_subtree: false,
                context_changed: self.root_context_changed,
                old_parent_contents_cull: None,
                auto_dark_mode_paused: false,
                leading_svg_matrix: None,
            },
        );
        self.view_background_rects = None;
        self.display_item_fragment_scope = None;
        self.fragmentainer_scope = None;
        // Source: pre_paint_tree_walk.cc:1584-1585. Clear each visited
        // object's consumed flags (children before parents, as recorded).
        for data in std::mem::take(&mut self.deferred_paint_flag_clears) {
            // Every recorded sidecar is owned by the source lease this walk
            // holds (pending_source_lease).
            unsafe { &*data }.paint.ClearSourcePaintFlags();
        }
        // Source: pre_paint_tree_walk.cc:106-107. Each visited object's flags
        // were cleared above; schedule the delayed targets for the next pass.
        self.paint_invalidator
            .ProcessPendingDelayedPaintInvalidations();
        // Source: cull_rect_updater.cc:279-280,353-354, after the walk.
        CullRectUpdater::FinishUpdate(root.Cursor());
        self.local_frame_ancestors = std::mem::take(&mut self.next_local_frame_ancestors);
        // DisplayLockContext: a pending child walk of an object the walk did
        // not reach went with the object.
        self.previous_display_lock_walks.clear();
        // Decision D2: out-of-flow occurrences the walk reached before their
        // logical parent occurrence (or did not reach while that occurrence's
        // effect changed) take the logical parent's effect now.
        if !self.logical_effects_changed.is_empty() {
            let mut trace = browser_tracing::span("prepaint", "PrePaint.LogicalEffectFixups");
            let fixups = self.FixupLogicalOutOfFlowEffects(root.Cursor());
            trace.set("logical_effect_fixups", fixups as f64);
            self.stats.post_walk |= fixups != 0;
        }
        if self.inline_outline_groups || self.visited_inline_outline {
            let _trace = browser_tracing::span("prepaint", "PrePaint.PrepareOutlines");
            self.inline_outline_groups = self.PrepareOutlines(root.Cursor());
            self.stats.post_walk = true;
        }
        browser_tracing::instant(
            "prepaint",
            "PrePaint.Repaint",
            &[("cull_change_repaints", self.stats.cull_change_repaints as f64)],
        );
        {
            let mut admission = browser_tracing::span("prepaint", "PrePaint.Admission");
            admission.set("computed_nodes", self.stats.computed_nodes as f64);
            admission.set("skipped_subtrees", self.stats.skipped_subtrees as f64);
            admission.set("clean_visits", self.stats.clean_visits as f64);
            admission.set(
                "direct_transform_updates",
                self.stats.direct_transform_updates as f64,
            );
            admission.set("invalidation_nodes", self.stats.invalidation_nodes as f64);
            admission.set("full_walk", self.full_walk as u8 as f64);
            admission.set(
                "source_result_replaced",
                self.stats.source_result_replaced as u8 as f64,
            );
        }
        // Source: ObjectPaintProperties::Clear* and FragmentData destruction.
        // A role a visited object lost is released here unless another
        // object registered the same key in this pass; a slot whose
        // FragmentData Layout destroyed goes with its last reference.
        {
            if !self.released_property_keys.is_empty() {
                let registered: HashSet<PaintPropertyKey> =
                    self.property_keys.values().copied().collect();
                let released: HashSet<PaintPropertyKey> = self
                    .released_property_keys
                    .drain(..)
                    .filter(|key| !registered.contains(key))
                    .collect();
                if !released.is_empty() {
                    self.property_node_store
                        .RetainLiveSlots(|key| !released.contains(key));
                }
            }
            // The logical effect contexts of occurrences that are gone hold
            // the last references to their effects (besides the slot map).
            for entries in self.logical_effect_contexts.values_mut() {
                entries.retain(|entry| Arc::strong_count(&entry.effect) > 2);
            }
            self.logical_effect_contexts
                .retain(|_, entries| !entries.is_empty());
            self.property_node_store.ReleaseUnreferencedFragmentSlots();
            self.committed_clients.Prune();
            self.intersection_observation_state = self
                .intersection_observation_state
                .max(self.property_node_store.intersection_observation_state);
            self.next_property_id = self.property_node_store.next_id();
        }
        self.walked_once = true;
    }

    /// A paint property value changed in place outside a walk (the retained
    /// scroll update, a direct composited animation update). The flat values
    /// of the child contexts kept on FragmentData (CachedChildContext) reach
    /// such a value only through a build, so none of them stays current.
    pub fn PropertyValuesChangedOutsideWalk(&mut self) {
        self.child_context_generation = self.child_context_generation.wrapping_add(1);
    }

    /// The walk's results are in use: Paint recorded from them or the
    /// lifecycle reached PrePaintClean. The Layout result the walk visited
    /// stays alive for the queries that read its FragmentData between walks
    /// (hit testing, special painting, the direct property updates); no
    /// PrePaint tree is retained.
    pub fn CommitWalk(&mut self) {
        if let Some(lease) = self.pending_source_lease.take() {
            self.retained_source_lease = Some(lease);
        }
        if let Some(root) = self.pending_root.take() {
            self.retained_root = Some(root);
        }
        self.retained_local_frame_leases = self
            .local_frame_views
            .values()
            .map(|view| view.lease.clone())
            .collect();
        self.local_frame_sources_changed = false;
    }

    /// The root of the Layout result the last committed walk visited.
    pub fn RetainedRoot(&self) -> Option<PaintFragmentCursor<'_>> {
        self.retained_root
    }

    /// Every occurrence a PrePaint walk produced FragmentData for, below and
    /// including `root`, in fragment tree order; an embedded local frame's
    /// LayoutView follows its owner's children (FramePainter's order).
    pub fn VisitWalkedOccurrences<'a>(
        &'a self,
        root: PaintFragmentCursor<'a>,
        visit: &mut dyn FnMut(PaintFragmentCursor<'a>, &'a PrePaintFragmentData),
    ) {
        let Some(data) = root
            .PrePaintData::<PrePaintFragmentData>()
            .filter(|data| data.WasWalked())
        else {
            return;
        };
        visit(root, data);
        for child in root.Children() {
            self.VisitWalkedOccurrences(child, visit);
        }
        if let Some(view) = self
            .local_frame_views
            .get(&root.PaintData().paint.DisplayItemClientId())
        {
            self.VisitWalkedOccurrences(view.Root(), visit);
        }
    }

    /// Joins the outline of each inline continuation group onto its first
    /// occurrence (OutlinePainter paints the united outline once). Returns
    /// whether any group exists. Only grouped inline outlines are prepared;
    /// Chromium collects every other outline at paint time
    /// (BoxFragmentPainter::PaintObject -> FragmentPainter::PaintOutline ->
    /// AddOutlineRects, core/paint/fragment_painter.cc).
    fn PrepareOutlines(&self, root: PaintFragmentCursor<'_>) -> bool {
        let mut groups: BTreeMap<InlineOutlineKey, Vec<PaintFragmentCursor<'_>>> = BTreeMap::new();
        self.VisitWalkedOccurrences(root, &mut |cursor, data| {
            let fragment = cursor.PaintData();
            if InlineOutlineCandidate(fragment) {
                groups
                    .entry((fragment.ViewNodeId(), data.WalkRecord().fragmentainer))
                    .or_default()
                    .push(cursor);
            }
        });
        let any_group = !groups.is_empty();
        for occurrences in groups.into_values() {
            let mut outline_rects = Vec::new();
            for occurrence in &occurrences {
                let paint_offset = occurrence
                    .PrePaintData::<PrePaintFragmentData>()
                    .map_or(Offset::default(), |data| data.FlatReplay().paint_offset);
                AppendSelfOutlineRects(occurrence.PaintData(), paint_offset, &mut outline_rects);
            }
            let united: Option<Arc<[PaintRect]>> =
                (!outline_rects.is_empty()).then(|| Arc::from(outline_rects));
            for (index, occurrence) in occurrences.iter().enumerate() {
                // SAFETY: the post-walk pass is the occurrence's single
                // PrePaint writer; no reference into the slot is live.
                let data = unsafe { occurrence.PrePaintDataMut::<PrePaintFragmentData>() };
                let flat = data.FlatReplayMut();
                let rects = if index == 0 { united.clone() } else { None };
                let unchanged = flat.paints_outline == (index == 0)
                    && flat.Rare().and_then(|rare| rare.outline_rects.as_ref()).map(|rects| &**rects)
                        == rects.as_deref();
                if !unchanged {
                    flat.SetOutline(index == 0, rects);
                }
            }
        }
        any_group
    }

    /// Decision D2 fix-up over the walked tree (see RebindLogicalEffect)
    /// for the out-of-flow occurrences whose logical parent's effect context
    /// this walk created or changed. Returns the number of fix-ups.
    fn FixupLogicalOutOfFlowEffects(&mut self, root: PaintFragmentCursor<'_>) -> usize {
        let mut ancestors = Vec::new();
        let mut fixups = 0;
        self.FixupLogicalOutOfFlowEffectsIn(root, &mut ancestors, &mut fixups);
        fixups
    }

    fn FixupLogicalOutOfFlowEffectsIn(
        &mut self,
        cursor: PaintFragmentCursor<'_>,
        ancestors: &mut Vec<u64>,
        fixups: &mut usize,
    ) {
        let Some(fragmentainer) = cursor
            .PrePaintData::<PrePaintFragmentData>()
            .filter(|data| data.WasWalked())
            .map(|data| data.WalkRecord().fragmentainer)
        else {
            return;
        };
        let fragment = cursor.PaintData();
        let position = fragment.paint.Position();
        if position == Position::kAbsolute || position == Position::kFixed {
            if let Some(logical_id) = fragment.paint.LogicalParentNodeId().filter(|id| {
                self.logical_effects_changed.contains(id) && !ancestors.contains(id)
            }) {
                if let Some(context) =
                    self.LogicalEffectContextFor(fragment, logical_id, fragmentainer)
                {
                    if RebindLogicalEffect(cursor, &context.effect, &mut self.property_node_store) {
                        *fixups += 1;
                    }
                }
            }
        }
        ancestors.push(if fragment.ViewEstablishesPaintState() {
            fragment.ViewNodeId()
        } else {
            0
        });
        for child in cursor.Children() {
            self.FixupLogicalOutOfFlowEffectsIn(child, ancestors, fixups);
        }
        let frame_root = self
            .local_frame_views
            .get(&fragment.paint.DisplayItemClientId())
            .map(|view| view.root);
        if let Some(frame_root) = frame_root {
            self.FixupLogicalOutOfFlowEffectsIn(frame_root, ancestors, fixups);
        }
        ancestors.pop();
    }

    /// Source: LocalFrameView::ExecuteAllPendingUpdates
    /// (local_frame_view.cc:5498-5525), run before the walk, with
    /// PaintPropertyTreeBuilder::DirectlyUpdateTransformMatrix
    /// (paint_property_tree_builder.cc:4842-4888) for each object
    /// ScheduleDeferredTransformNodeUpdate deferred. Each chain is the
    /// object's DisplayItemClient id followed by those of its LayoutObject
    /// ancestors. An object the direct path cannot update (the PrePaint half
    /// of CanDoDeferredTransformNodeUpdate, 4953-5002, an occurrence the
    /// FragmentData does not describe, or a cull rect that left its expanded
    /// window) is built by the walk from its current style, as the
    /// SetNeedsPaintPropertyUpdate that Chromium's caller falls back to.
    fn ExecuteAllPendingUpdates(&mut self, root: PaintFragmentCursor<'_>, chains: &[Vec<u64>]) {
        let mut trace = browser_tracing::span("prepaint", "PrePaint.ExecutePendingUpdates");
        trace.set("transform_updates", chains.len() as f64);
        let mut rejected = 0usize;
        let mut rejection = 0u8;
        for chain in chains {
            let Some(&client) = chain.first() else {
                continue;
            };
            // A walk that visits every object builds each from its style.
            let outcome = if self.full_walk {
                Err(1)
            } else {
                self.DirectlyUpdateTransformMatrix(root, chain)
            };
            match outcome {
                Ok(updated) => self.stats.direct_transform_updates += usize::from(updated),
                Err(code) => {
                    rejected += 1;
                    rejection = code;
                    self.must_visit_clients.extend(chain.iter().copied());
                    self.must_build_clients.insert(client);
                }
            }
        }
        trace.set("updated", self.stats.direct_transform_updates as f64);
        trace.set("rejected", rejected as f64);
        trace.set("rejection", rejection as f64);
    }

    /// PaintPropertyTreeBuilder::DirectlyUpdateTransformMatrix for the object
    /// `chain[0]`, over its FragmentData. Ok(false): the Transform node
    /// already had the value. Err: the walk must build the object.
    ///
    /// The object's Transform slot is updated in place from the current
    /// style; nothing else of its property state depends on the `transform`
    /// value (Layout schedules the deferred update only when the transform
    /// alone changed). Rechrom's flat values carry the matrix too: the
    /// object's own flat matrix, its flat list and that of every descendant
    /// occurrence, and the child contexts they keep, take the new value
    /// (ReplaceFlatTransformInSubtree), as a build would write them.
    fn DirectlyUpdateTransformMatrix(
        &mut self,
        root: PaintFragmentCursor<'_>,
        chain: &[u64],
    ) -> Result<bool, u8> {
        let (parent, cursor) = FindPendingTransformOccurrence(root, chain).ok_or(1u8)?;
        let fragment = cursor.PaintDataOption().ok_or(2u8)?;
        let data = cursor
            .PrePaintData::<PrePaintFragmentData>()
            .filter(|data| data.WasWalked())
            .ok_or(2u8)?;
        // The FragmentData describes this fragment, outside any fragmentation
        // scope (IsFragmented, 4974-4978).
        let record = *data.WalkRecord();
        if record.walked_owner != WalkedOwner(cursor)
            || record.link_offset != cursor.ViewOffsetInContainer()
            || record.size != fragment.ViewSize()
            || record.display_item_fragment != 0
            || record.fragmentainer != 0
            || fragment.paint.LivePaintLayerFacts().is_fragmented
        {
            return Err(1);
        }
        if !fragment.paint.HasSource() || !fragment.ViewEstablishesPaintState() {
            return Err(2);
        }
        // The cull rects below are mapped from the paint parent's context,
        // which a fixed-to-view object does not inherit.
        if fragment.paint.FixedToView() || fragment.paint.IsSVGHiddenContainer() {
            return Err(2);
        }
        let style = &fragment.paint.style;
        // SVG children (4959-4962) and canvas-rendered subtrees (4993-5000).
        if IsSvgLayoutChild(fragment.paint.SourceKind())
            || fragment.paint.SvgLocalTransform().is_some()
            || (fragment.paint.CanvasDrawElementEnabled() && fragment.paint.IsInCanvasSubtree())
        {
            return Err(3);
        }
        // The object's own flat matrices end with Transform
        // (ResolveFragmentGeometry); a text-combine matrix follows it and an
        // element canvas transform is not in the list.
        if fragment.paint.IsTextCombine() || fragment.paint.ElementCanvasTransformInput().is_some() {
            return Err(4);
        }
        let flat = data.FlatReplay();
        if !flat.applies_transform {
            return Err(4);
        }
        // Cannot directly update properties that were not created
        // (4980-4983): the existing ObjectPaintProperties Transform, the same
        // live node the keyed store holds for the slot.
        let client = fragment.paint.DisplayItemClientId();
        let properties = data.PaintProperties().ok_or(4u8)?;
        let current = properties.Transform().cloned().ok_or(4u8)?;
        let key = PaintPropertyKey {
            owner: PaintPropertyOwner::Fragment {
                client,
                fragment: record.display_item_fragment,
            },
            role: PaintPropertyRole::Transform,
        };
        if !self
            .property_node_store
            .TransformOwnerForKey(key)
            .is_some_and(|node| Arc::ptr_eq(node, &current))
        {
            return Err(4);
        }
        // `transform: none` keeps the Transform slot only for an animation,
        // 3D context, backface or direct compositing reason (NeedsTransform,
        // 1388-1410); the full build would remove it otherwise.
        if !style.has_transform() {
            let _direct_reasons = CompositingReasonFinder::DirectReasonsScope();
            if !TransformPropertyRoles(fragment, style).contains(&PaintPropertyRole::Transform) {
                return Err(4);
            }
        }
        // An Effect allocated for a 2D scale transform (4985-4992).
        if properties.Effect().is_some_and(|effect| {
            paint_model::property_cell::with_access(|access| {
                effect.state(access).NeedsEffectFor2DScaleTransform()
            })
        }) {
            return Err(5);
        }
        // The builder derives the node's backface visibility from whether the
        // transform has 3D operations; keep the direct update to values that
        // cannot change it.
        let (old_matrix, old_backface, is_running_animation_on_compositor) =
            paint_model::property_cell::with_access(|access| {
                let state = current.state(access);
                (
                    state.matrix,
                    state.backface_visibility,
                    current.HasActiveTransformAnimation(access),
                )
            });
        if style.has_3d_transform_operation() || old_backface == PaintBackfaceVisibility::Visible {
            return Err(6);
        }
        // The matrix is the last entry of the object's flat list and of its
        // own flat matrices, and its children's lists extend that prefix.
        let flat_index = flat.transforms.len().checked_sub(1).ok_or(7u8)?;
        let own_last = flat
            .Rare()
            .and_then(|rare| rare.local_transforms.as_ref())
            .and_then(|locals| locals.last());
        if flat.transforms[flat_index] != old_matrix || own_last != Some(&old_matrix) {
            return Err(7);
        }
        if let Some(cached) = data
            .CachedChildContext()
            .filter(|cached| cached.generation == self.child_context_generation)
        {
            if cached.context.transforms.get(flat_index) != Some(&old_matrix) {
                return Err(7);
            }
        }
        // The context the paint parent handed this object (ContextForChild):
        // its contents cull rect and property state, for the cull update.
        let (parent_cull, parent_state, parent_subtree_infinite) = parent
            .PrePaintData::<PrePaintFragmentData>()
            .and_then(PrePaintFragmentData::CachedChildContext)
            .filter(|cached| cached.generation == self.child_context_generation)
            .map(|cached| {
                (
                    cached.context.cull,
                    cached.context.properties.nodes.clone(),
                    cached.context.subtree_should_use_infinite_cull_rect,
                )
            })
            .ok_or(9u8)?;

        // ComputedStyle::ApplyTransform with kIncludeTransformOperations,
        // kExcludeTransformOrigin, kExcludeMotionPath and
        // kExcludeIndependentTransformProperties, around the reference box,
        // as UpdateIndividualTransform resolves it in the builder
        // (ComputeReferenceBox, 4849; ResolveFragmentGeometry).
        let reference_rect = ResolveTransformReferenceRect(
            style.transform_reference_box(),
            fragment.ViewSize(),
            fragment.Borders(),
            fragment.Padding(),
        );
        let transform_origin = style.transform_origin();
        let paint_snap_offset = flat.paint_snap_offset;
        let resolved = TransformPayloadForRole(style, PaintPropertyRole::Transform)
            .as_ref()
            .map_or_else(TransformMatrix::default, |transform| {
                ResolveLocalTransformAroundReferenceBox(
                    transform,
                    Offset {
                        x: flat.paint_offset.x + paint_snap_offset.x,
                        y: flat.paint_offset.y + paint_snap_offset.y,
                    },
                    reference_rect,
                    transform_origin.as_ref(),
                )
            });
        let matrix = MultiplyTransforms(
            &TranslationTransform(-paint_snap_offset.x, -paint_snap_offset.y),
            &resolved,
        );
        let store = &mut self.property_node_store;
        let change = paint_model::property_cell::with_writer(|writer| {
            store.DirectlyUpdateTransform(
                writer,
                &current,
                matrix,
                [0.0; 3],
                TransformAnimationState {
                    is_running_animation_on_compositor,
                },
            )
        });
        if change == PaintPropertyChangeType::Unchanged {
            return Ok(false);
        }
        // 4873-4875.
        self.intersection_observation_state = self
            .intersection_observation_state
            .max(foundation::intersection_observation_state::IntersectionObservationState::Desired);
        // FragmentData values outside the walk changed: the retained scroll
        // coverage captured from them is stale (PaintEngine).
        self.stats.post_walk = true;
        self.ReplaceFlatTransformInSubtree(cursor, flat_index, &old_matrix, &matrix);
        // CullRectUpdater::PaintPropertiesChanged (cull_rect_updater.cc:
        // 558-620): a simple-value transform change updates this object's
        // cull rects. Descendant cull rects are relative to its contents cull
        // rect, so they stay valid while it does; a rect that moved past its
        // expanded window needs the walk (repaint and descendant updates).
        if change >= PaintPropertyChangeType::ChangedOnlySimpleValues {
            let data = cursor
                .PrePaintData::<PrePaintFragmentData>()
                .ok_or(8u8)?;
            let (old_self, old_contents) = StoredCullRects(data, Some(parent_cull));
            let previous_paint_result = if record.is_paint_layer {
                cursor
                    .OwnPaintLayer()
                    .map_or(paint_model::PaintResult::kMayBeClippedByCullRect, |layer| {
                        layer.PreviousPaintResult()
                    })
            } else {
                paint_model::PaintResult::kMayBeClippedByCullRect
            };
            let [moves_pixels, perspective, active_animation, non_composited] =
                record.cull_decision_inputs;
            let infinite = crate::cull_rect_updater::ShouldUseInfiniteCullRect(
                fragment,
                parent_subtree_infinite,
                moves_pixels,
                perspective,
                active_animation,
                non_composited,
                previous_paint_result,
            );
            if infinite.use_infinite {
                if old_self.is_some() || old_contents.is_some() {
                    return Err(8);
                }
            } else {
                let expansion_ratio = crate::cull_rect_updater::ExpansionRatio(
                    cursor.DocumentResources().map(|resources| &**resources),
                );
                let local_state = data.LocalBorderBoxProperties();
                let contents_state = data.ContentsProperties();
                let (self_cull, _) = CullRectUpdater::ComputeFragmentCullRect(
                    parent_cull,
                    Some(old_self),
                    &self.viewport_properties.nodes,
                    &parent_state,
                    &local_state,
                    expansion_ratio,
                );
                let (contents_cull, _) = CullRectUpdater::ComputeFragmentContentsCullRect(
                    self_cull,
                    Some(old_contents),
                    &self.viewport_properties.nodes,
                    &local_state,
                    &contents_state,
                    expansion_ratio,
                );
                if self_cull != old_self || contents_cull != old_contents {
                    return Err(8);
                }
            }
        }
        Ok(true)
    }

    /// The deferred transform update of the object `cursor`: its own matrix
    /// at `index` of the flat lists changes from `old` to `new`. The flat
    /// list of the object and of every walked descendant occurrence (an
    /// embedded frame's included), which all extend that prefix, the
    /// object's own flat matrix and the child contexts the occurrences keep
    /// carry the resident property node's new value; a rewritten context
    /// records the digest its build would record, so the children it was
    /// handed to stay clean. Lists shared between occurrences stay shared.
    fn ReplaceFlatTransformInSubtree(
        &self,
        cursor: PaintFragmentCursor<'_>,
        index: usize,
        old: &TransformMatrix,
        new: &TransformMatrix,
    ) {
        let mut memo = FlatTransformMemo::default();
        self.ReplaceFlatTransformIn(cursor, index, old, new, &mut memo, true);
    }

    fn ReplaceFlatTransformIn(
        &self,
        cursor: PaintFragmentCursor<'_>,
        index: usize,
        old: &TransformMatrix,
        new: &TransformMatrix,
        memo: &mut FlatTransformMemo,
        is_owner: bool,
    ) {
        if !cursor
            .PrePaintData::<PrePaintFragmentData>()
            .is_some_and(|data| data.WasWalked())
        {
            return;
        }
        {
            // SAFETY: before the walk PrePaint is the occurrence's single
            // PrePaint writer; no reference into the slot is live.
            let data = unsafe { cursor.PrePaintDataMut::<PrePaintFragmentData>() };
            let flat = data.FlatReplayMut();
            memo.Replace(&mut flat.transforms, index, old, new);
            if is_owner {
                if let Some(locals) = flat
                    .RareMut()
                    .and_then(|rare| rare.local_transforms.as_mut())
                {
                    if locals.last() == Some(old) {
                        let mut values = locals.to_vec();
                        if let Some(last) = values.last_mut() {
                            *last = *new;
                        }
                        *locals = Arc::from(values);
                    }
                }
            }
            let generation = self.child_context_generation;
            let mut digest = None;
            if let Some(cached) = data.CachedChildContextMut() {
                if cached.generation == generation
                    && memo.Replace(&mut cached.context.transforms, index, old, new)
                {
                    digest = Some(ChildContextDigest(&cached.context));
                }
            }
            if let Some(digest) = digest {
                let mut record = *data.WalkRecord();
                record.child_context_digest = digest;
                data.SetWalkRecord(record);
            }
        }
        for child in cursor.Children() {
            self.ReplaceFlatTransformIn(child, index, old, new, memo, false);
        }
        if let Some(view) = self
            .local_frame_views
            .get(&cursor.PaintData().paint.DisplayItemClientId())
        {
            self.ReplaceFlatTransformIn(view.Root(), index, old, new, memo, false);
        }
    }

    /// Source: LocalFrameView::ExecuteAllPendingUpdates and
    /// PaintPropertyTreeBuilder::DirectlyUpdate{TransformMatrix,OpacityValue}.
    ///
    /// Layout has already refreshed the canonical FragmentData sidecars for
    /// these objects. Update only their existing ObjectPaintProperties slots;
    /// topology, compositing-reason, or owner changes reject this path so the
    /// caller can run the ordinary complete PrePaint walk.
    /// Source: object_paint_properties.h:503-522 and
    /// paint_property_tree_builder.cc DirectlyUpdateCompositedOpacityValue:
    /// a running compositor opacity animation.
    const OPACITY_ANIMATION: EffectAnimationState = EffectAnimationState {
        is_running_opacity_animation_on_compositor: true,
        is_running_filter_animation_on_compositor: false,
        is_running_backdrop_filter_animation_on_compositor: false,
    };

    pub fn DirectlyUpdateCompositedAnimationProperties(
        &mut self,
        targeted_nodes: &[u64],
    ) -> Option<DirectCompositedAnimationPropertyUpdate> {
        let mut trace = browser_tracing::span("prepaint", "PrePaint.DirectAnimationPropertyUpdate");
        let targeted = targeted_nodes.iter().copied().collect::<BTreeSet<_>>();
        trace.set("targeted_nodes", targeted.len() as f64);
        let Some(root) = self.retained_root.filter(|_| !targeted.is_empty()) else {
            trace.set("rejection", 1.0);
            return None;
        };

        let mut matched = BTreeSet::new();
        let mut active = BTreeSet::new();
        let mut transform_proposals = BTreeMap::new();
        let mut effect_proposals = BTreeMap::new();
        let mut rejection = 0u8;

        // The targeted objects' occurrences, through the committed walk's
        // FragmentData (Chromium reaches each pending LayoutObject directly).
        let mut occurrences = Vec::new();
        self.VisitWalkedOccurrences(root, &mut |cursor, data| {
            if targeted.contains(&cursor.PaintData().ViewNodeId()) {
                occurrences.push((cursor, data));
            }
        });
        for (cursor, data) in occurrences {
            let fragment = cursor.PaintData();
            trace.set("fragment_opacity", fragment.paint.style.opacity() as f64);
            matched.insert(fragment.ViewNodeId());
            if !fragment.paint.HasSource() || !fragment.ViewEstablishesPaintState() {
                continue;
            }
            let style = &fragment.paint.style;
            let client = fragment.paint.DisplayItemClientId();
            if client == 0 {
                rejection = 2;
                break;
            }
            let owner = PaintPropertyOwner::Fragment {
                client,
                fragment: data.WalkRecord().display_item_fragment,
            };
            let flat = data.FlatReplay();
            let local_paint_offset = Offset {
                x: flat.paint_offset.x + flat.paint_snap_offset.x,
                y: flat.paint_offset.y + flat.paint_snap_offset.y,
            };

            let active_transform_roles = [
                (
                    PaintPropertyRole::Translate,
                    style.active_translate_animation(),
                ),
                (PaintPropertyRole::Rotate, style.active_rotate_animation()),
                (PaintPropertyRole::Scale, style.active_scale_animation()),
                (
                    PaintPropertyRole::Transform,
                    style.active_transform_animation(),
                ),
            ];
            let reference_rect = ResolveTransformReferenceRect(
                style.transform_reference_box(),
                fragment.ViewSize(),
                fragment.Borders(),
                fragment.Padding(),
            );
            let transform_origin = style.transform_origin();
            for (role, is_active) in active_transform_roles {
                if !is_active {
                    continue;
                }
                active.insert(fragment.ViewNodeId());
                if IsSvgLayoutChild(fragment.paint.SourceKind())
                    || !flat.applies_transform
                    || !TransformPropertyRoles(fragment, style).contains(&role)
                {
                    rejection = 2;
                    break;
                }
                let key = PaintPropertyKey { owner, role };
                if transform_proposals.contains_key(&key) {
                    continue;
                }
                let Some(current) = self.property_node_store.TransformOwnerForKey(key) else {
                    rejection = 2;
                    break;
                };
                if !paint_model::property_cell::with_access(|access| current.HasActiveTransformAnimation(access))
                    || paint_model::property_cell::with_access(|access| current.state(access).direct_compositing_reasons)
                        != CompositingReasonFinder::TransformReasons(fragment, role, false)
                {
                    rejection = 3;
                    break;
                }
                let resolved = TransformPayloadForRole(style, role).as_ref().map_or_else(
                    TransformMatrix::default,
                    |transform| {
                        ResolveLocalTransformAroundReferenceBox(
                            transform,
                            local_paint_offset,
                            reference_rect,
                            transform_origin.as_ref(),
                        )
                    },
                );
                let matrix = MultiplyTransforms(
                    &TranslationTransform(-flat.paint_snap_offset.x, -flat.paint_snap_offset.y),
                    &resolved,
                );
                let change = paint_model::property_cell::with_access(|access| {
                    current.DirectTransformAndOriginChange(
                        access,
                        matrix,
                        [0.0; 3],
                        TransformAnimationState {
                            is_running_animation_on_compositor: true,
                        },
                    )
                });
                if !matches!(
                    change,
                    PaintPropertyChangeType::Unchanged
                        | PaintPropertyChangeType::ChangedOnlyCompositedValues
                ) {
                    rejection = 4;
                    break;
                }
                transform_proposals.insert(key, (current.clone(), matrix, fragment.ViewNodeId()));
            }
            if rejection != 0 {
                break;
            }

            if style.active_opacity_animation() {
                active.insert(fragment.ViewNodeId());
                let key = PaintPropertyKey {
                    owner,
                    role: PaintPropertyRole::Effect,
                };
                if !effect_proposals.contains_key(&key) {
                    let Some(current) = self.property_node_store.EffectOwnerForKey(key) else {
                        rejection = 2;
                        break;
                    };
                    let mut reasons = CompositingReasonFinder::EffectReasons(fragment);
                    reasons.PutAll(CompositingReasonFinder::AdditionalEffectCompositingTrigger(
                        fragment,
                    ));
                    if paint_model::property_cell::with_access(|access| {
                        !current.HasActiveOpacityAnimation(access)
                            || current.state(access).direct_compositing_reasons != reasons
                    })
                    {
                        rejection = 3;
                        break;
                    }
                    let opacity = MaskAdjustedOpacity(style);
                    let change = paint_model::property_cell::with_access(|access| {
                        current.ComputeOpacityChange(access, opacity, Self::OPACITY_ANIMATION)
                    });
                    if !matches!(
                        change,
                        PaintPropertyChangeType::Unchanged
                            | PaintPropertyChangeType::ChangedOnlyCompositedValues
                    ) {
                        rejection = 4;
                        break;
                    }
                    effect_proposals.insert(key, (current.clone(), opacity, fragment.ViewNodeId()));
                }
            }
        }

        if rejection == 0 && matched != targeted {
            rejection = 5;
        }
        if rejection == 0 && active != targeted {
            rejection = 6;
        }
        trace.set("matched_targets", matched.len() as f64);
        trace.set("active_targets", active.len() as f64);
        trace.set("transform_updates", transform_proposals.len() as f64);
        trace.set("effect_updates", effect_proposals.len() as f64);
        trace.set("rejection", rejection as f64);
        if rejection != 0 || (transform_proposals.is_empty() && effect_proposals.is_empty()) {
            return None;
        }

        // Every proposal was admitted: update the resident slot nodes in
        // place; the update carries the same live nodes.
        self.PropertyValuesChangedOutsideWalk();
        let mut update = DirectCompositedAnimationPropertyUpdate::default();
        for (_, (node, matrix, node_id)) in transform_proposals {
            let store = &mut self.property_node_store;
            paint_model::property_cell::with_writer(|writer| {
                store.DirectlyUpdateTransform(
                    writer,
                    &node,
                    matrix,
                    [0.0; 3],
                    TransformAnimationState {
                        is_running_animation_on_compositor: true,
                    },
                )
            });
            update.transforms.push(node);
            let element_id = paint_model::property_cell::with_access(|access| {
                update.transforms.last().unwrap().state(access).compositor.compositor_element_id
            });
            if foundation::CompositorElementId::IsValidInternalValue(element_id.GetInternalValue())
            {
                update.bindings.push(CompositedAnimationPropertyBinding {
                    node_id,
                    property: CompositedAnimationProperty::Transform,
                    element_id,
                });
            }
        }
        for (_, (node, opacity, node_id)) in effect_proposals {
            let store = &mut self.property_node_store;
            paint_model::property_cell::with_writer(|writer| {
                store.DirectlyUpdateOpacity(writer, &node, opacity, Self::OPACITY_ANIMATION)
            });
            update.effects.push(node);
            let element_id = paint_model::property_cell::with_access(|access| {
                update.effects.last().unwrap().state(access).compositor.compositor_element_id
            });
            if foundation::CompositorElementId::IsValidInternalValue(element_id.GetInternalValue())
            {
                update.bindings.push(CompositedAnimationPropertyBinding {
                    node_id,
                    property: CompositedAnimationProperty::Opacity,
                    element_id,
                });
            }
        }
        trace.set("admitted", 1.0);
        Some(update)
    }

    /// Objects whose full invalidation remains delayed after this PrePaint
    /// pass. The walk has already re-armed them on their LayoutObjects.
    pub fn PendingDelayedPaintInvalidations(&self) -> &HashSet<u64> {
        self.paint_invalidator.PendingDelayedPaintInvalidations()
    }

    /// Record that Paint recorded an artifact from the current PrePaint
    /// state. See `PrePaintConsumedSinceRecording`.
    pub fn MarkPrePaintRecorded(&mut self) {
        self.recorded_consuming_walks = self.consuming_walks;
    }

    /// LayoutView's PaintLayer::SelfOrDescendantNeedsRepaint() as
    /// LocalFrameView::PaintTree reads it: the live Layout-owned layer bits.
    /// Every client invalidation since the last committed Paint marked its
    /// painting layer's container chain, and only that commit clears them
    /// (PaintLayer::ClearNeedsRepaintRecursively).
    // cpp: third_party/blink/renderer/core/frame/local_frame_view.cc:3185-3188
    // cpp: third_party/blink/renderer/core/paint/paint_layer.cc:2688-2785
    pub fn RootNeedsRepaint(&self, root: PaintFragmentCursor<'_>) -> bool {
        !root
            .PrePaintData::<PrePaintFragmentData>()
            .is_some_and(|data| data.WasWalked() && data.WalkRecord().is_paint_layer)
            || root.PaintData().paint.PaintLayerSelfOrDescendantNeedsRepaint()
    }

    pub fn HasLocalFrameViews(&self) -> bool {
        !self.local_frame_views.is_empty() || !self.retained_local_frame_leases.is_empty()
    }

    /// Whether a PrePaint walk visited (and therefore cleared the paint
    /// flags of) any object since the last recorded Paint. A walk consumes
    /// LayoutObject dirty bits (pre_paint_tree_walk.cc:1584-1585), so after
    /// a PrePaint-only lifecycle (hit-test preparation, a composited
    /// animation fallback) clean bits no longer prove that the published
    /// artifact reflects the source: it was recorded before that walk.
    pub fn PrePaintConsumedSinceRecording(&self) -> bool {
        self.consuming_walks != self.recorded_consuming_walks
    }

    pub fn CompositedAnimationPropertyBindings(
        &self,
        targeted_nodes: &[u64],
    ) -> Vec<CompositedAnimationPropertyBinding> {
        fn collect_transform(
            node_id: u64,
            node: Option<&Arc<TransformPaintPropertyNode>>,
            seen: &mut HashSet<u64>,
            output: &mut Vec<CompositedAnimationPropertyBinding>,
        ) {
            paint_model::property_cell::with_access(|access| {
            let mut node = node;
            while let Some(current) = node {
                let element_id = current.state(access).compositor.compositor_element_id;
                let internal = element_id.GetInternalValue();
                if current.HasActiveTransformAnimation(access)
                    && foundation::CompositorElementId::IsValidInternalValue(internal)
                    && seen.insert(internal)
                {
                    output.push(CompositedAnimationPropertyBinding {
                        node_id,
                        property: CompositedAnimationProperty::Transform,
                        element_id,
                    });
                }
                node = current.Parent(access);
            }
            })
        }

        fn collect_effect(
            node_id: u64,
            node: Option<&Arc<EffectPaintPropertyNode>>,
            seen: &mut HashSet<u64>,
            output: &mut Vec<CompositedAnimationPropertyBinding>,
        ) {
            paint_model::property_cell::with_access(|access| {
            let mut node = node;
            while let Some(current) = node {
                let element_id = current.state(access).compositor.compositor_element_id;
                let internal = element_id.GetInternalValue();
                if current.HasActiveOpacityAnimation(access)
                    && foundation::CompositorElementId::IsValidInternalValue(internal)
                    && seen.insert(internal)
                {
                    output.push(CompositedAnimationPropertyBinding {
                        node_id,
                        property: CompositedAnimationProperty::Opacity,
                        element_id,
                    });
                }
                node = current.Parent(access);
            }
            })
        }

        let Some(root) = self.retained_root else {
            return Vec::new();
        };
        let targeted = targeted_nodes.iter().copied().collect::<HashSet<_>>();
        let mut matched = HashSet::new();
        let mut output = Vec::new();
        let mut seen_transforms = HashSet::new();
        let mut seen_effects = HashSet::new();
        // The targeted objects' occurrences and their LocalBorderBoxProperties
        // (fragment_data.h:137-142); `applies_transform` / `applies_opacity`
        // are the builder's facts the walk recorded for the occurrence.
        self.VisitWalkedOccurrences(root, &mut |cursor, data| {
            let node_id = cursor.PaintData().ViewNodeId();
            if !targeted.contains(&node_id) {
                return;
            }
            matched.insert(node_id);
            let Some(state) = data.LocalBorderBoxPropertiesRef() else {
                return;
            };
            let applies_transform = data.FlatReplay().applies_transform;
            let applies_opacity = data.WalkRecord().applies_opacity;
            browser_tracing::instant(
                "prepaint",
                "PrePaint.CompositedAnimationTarget",
                &[
                    ("node_id", node_id as f64),
                    ("present", 1.0),
                    ("applies_transform", applies_transform as u8 as f64),
                    ("applies_opacity", applies_opacity as u8 as f64),
                    (
                        "active_transform",
                        paint_model::property_cell::with_access(|access| {
                            state.transform.HasActiveTransformAnimation(access)
                        }) as u8 as f64,
                    ),
                    (
                        "active_opacity",
                        paint_model::property_cell::with_access(|access| {
                            state.effect.HasActiveOpacityAnimation(access)
                        }) as u8 as f64,
                    ),
                ],
            );
            if applies_transform {
                collect_transform(node_id, Some(&state.transform), &mut seen_transforms, &mut output);
            }
            if applies_opacity {
                collect_effect(node_id, Some(&state.effect), &mut seen_effects, &mut output);
            }
        });
        for node_id in targeted.difference(&matched) {
            browser_tracing::instant(
                "prepaint",
                "PrePaint.CompositedAnimationTarget",
                &[
                    ("node_id", *node_id as f64),
                    ("present", 0.0),
                    ("applies_transform", 0.0),
                    ("applies_opacity", 0.0),
                    ("active_transform", 0.0),
                    ("active_opacity", 0.0),
                ],
            );
        }
        output
    }

    /// PaintController validation changes client metadata even without a new
    /// layout snapshot. Update the occurrence carrying each committed client
    /// in detached payload state (live source clients were validated through
    /// ValidateLiveClientsForCommittedPaint or the Layout commit). Each id is
    /// resolved through the committed-client directory, as
    /// ~PaintController reaches its clients through their pointers
    /// (paint_controller.cc:127-143); nothing walks the fragment tree.
    pub fn CommitClients(&mut self, clients: impl IntoIterator<Item = u64>) {
        for id in clients {
            let Some(cursor) = self.CommittedClientOccurrence(id) else {
                continue;
            };
            if cursor.PaintDataOption().is_none() {
                continue;
            }
            // SAFETY: the visiting walk's lease keeps the payload alive and
            // PrePaint is its single lifecycle writer here.
            let f = unsafe { cursor.PaintDataMut() };
            CommitDetachedClient(f, id);
        }
    }

}

#[cfg(test)]
mod tests {
    use super::{
        ClipsOverflow, CullRectChangeNeedsRepaint, EnclosingMaskClipRectInPropertySpace,
        FilterTaintChanged, HasFilterThatMovesPixels, HasPerspective,
        InvalidateClientsForCullRectChange, NeedsOverflowClip, NeedsPaintOffsetTranslation,
        OwnFilterOriginTainted, PaintPropertyRole, PaintPropertyTreeBuilder, PaintRect,
        PaintTreeNode, PixelSnappedRect, PrePaintTreeWalk, PropertyTreeState,
        ResolveClipPath, ResolveFragmentGeometry, ResolveMaskLayers,
        SvgTransformToPixelSnappedBorderBox, TransformPropertyRoles,
    };
    use layout::fragment::FragmentKind;
    use layout::paint::{
        FieldsetPaintData, FragmentPaintData, ScrollContainerPaintData, ScrollbarPaintAxis,
        ScrollbarPaintData, SelfOutlinePaintData,
    };
    use paint_input_types::{FragmentPaintRect, PaintResources};
    use layout::input::{Display, FloatSide, NodeKind, Overflow, Position};
    use paint_input_types::layout_input::{
        BorderLineStyle, Offset, PaintPathCommand, PaintPathVerb, Size, SvgShapeData,
        SvgShapeGeometry, TransformMatrix,
    };
    use paint_input_types::layout_input_types::Color;
    use paint_input_types::paint_input::{
        BackgroundBox, BackgroundImageLayer, ClipPathCircle, ClipPathLength, ClipPathPaint,
        ClipPathRadiusKind, ClipPathShapeRadius, PaintColorStop, PaintCornerRadii,
        PaintFilterOperation, PaintFilterType, PaintMaskImageState, PaintMaskLayer, PaintShader,
    };
    use std::sync::Arc;

    #[test]
    fn cull_rect_change_invalidates_fragment_drawing_clients() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.paint_layer_client_invalidation_reason =
            Some(foundation::PaintInvalidationReason::kNone);
        fragment.paint.selection_client_id = 42;
        fragment.paint.selection_client_invalidation_reason =
            Some(foundation::PaintInvalidationReason::kNone);

        InvalidateClientsForCullRectChange(&mut fragment);

        assert_eq!(
            fragment.paint.display_item_client_invalidation_reason,
            Some(foundation::PaintInvalidationReason::kLayout)
        );
        assert_eq!(
            fragment.paint.paint_layer_client_invalidation_reason,
            Some(foundation::PaintInvalidationReason::kLayout)
        );
        assert_eq!(
            fragment.paint.selection_client_invalidation_reason,
            Some(foundation::PaintInvalidationReason::kLayout)
        );

        fragment.paint.display_item_client_invalidation_reason =
            Some(foundation::PaintInvalidationReason::kJustCreated);
        InvalidateClientsForCullRectChange(&mut fragment);
        assert_eq!(
            fragment.paint.display_item_client_invalidation_reason,
            Some(foundation::PaintInvalidationReason::kJustCreated)
        );
    }

    #[test]
    fn incomplete_previous_paint_repaints_when_cull_rect_changes() {
        let old = PaintRect {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        };
        let expanded = PaintRect { width: 20.0, ..old };
        assert!(CullRectChangeNeedsRepaint(
            paint_model::PaintResult::kMayBeClippedByCullRect,
            Some(Some(old)),
            Some(Some(old)),
            Some(expanded),
            Some(old),
        ));
        assert!(!CullRectChangeNeedsRepaint(
            paint_model::PaintResult::kFullyPainted,
            Some(Some(old)),
            Some(Some(old)),
            Some(expanded),
            Some(old),
        ));
    }

    #[test]
    fn pixel_snapping_uses_physical_pixels() {
        let rect = PaintRect {
            x: 20.0,
            y: 20.5,
            width: 272.0,
            height: 14.0,
        };
        assert_eq!(PixelSnappedRect(&rect, 1.0).y, 21.0);
        assert_eq!(PixelSnappedRect(&rect, 2.0), rect);
    }

    #[test]
    fn text_line_top_snap_precedes_transform_subpixel_projection() {
        let mut node = PaintTreeNode::default();
        // Chromium rounds the CSS LayoutUnit line top, before device scaling.
        node.paint_snap_offset.y = -0.5;
        assert_eq!(node.RoundedPaintY(184.5), 185.0);
        assert_eq!(node.RoundedPaintY(-1.5), -1.0);

        // DPR does not participate in LayoutUnit::Round.
        node.pixel_snap_scale = 2.0;
        assert_eq!(node.RoundedPaintY(184.5), 185.0);
        assert_eq!(node.RoundedPaintY(-1.5), -1.0);
        assert_eq!(node.RoundedPaintY(114.75), 115.0);

        // Fractional DPR likewise leaves CSS rounding unchanged.
        node.pixel_snap_scale = 1.25;
        assert_eq!(node.RoundedPaintY(58.0), 58.0);
        assert_eq!(node.RoundedPaintY(276.4), 276.0);
    }

    #[test]
    fn moving_pixel_filters_and_perspective_require_infinite_cull() {
        let mut filter = PaintFilterOperation::default();
        filter.r#type = PaintFilterType::kBrightness;
        assert!(!HasFilterThatMovesPixels(&[filter.clone()]));
        filter.r#type = PaintFilterType::kBlur;
        assert!(HasFilterThatMovesPixels(&[filter]));

        let mut matrix = TransformMatrix::default();
        assert!(!HasPerspective(&matrix));
        matrix.values[11] = -0.01;
        assert!(HasPerspective(&matrix));
    }

    #[test]
    fn moving_filter_is_infinite_locally_without_setting_the_subtree_flag() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.has_source = true;
        fragment.paint.establishes_paint_state = true;
        fragment.paint.is_box_model_object = true;
        fragment.paint.paint_layer_state_is_authoritative = true;
        fragment.paint.paint_layer_client_id = 41;
        fragment.paint.paint_layer_is_self_painting = true;
        fragment.paint.effective_zoom = 2.0;
        fragment.paint.paint_layer_filter_reference_box =
            Some(paint_input_types::FragmentPaintRect {
                offset: Offset { x: 3.0, y: 4.0 },
                size: Size {
                    width: 40.0,
                    height: 50.0,
                },
            });
        fragment.paint.paint_layer_filter_viewport = Some(Size {
            width: 100.0,
            height: 80.0,
        });
        fragment.paint.overflow_x = Overflow::kHidden;
        fragment.paint.overflow_y = Overflow::kHidden;
        fragment.size = Size {
            width: 20.0,
            height: 20.0,
        };
        let mut blur = PaintFilterOperation::default();
        blur.r#type = PaintFilterType::kBlur;
        fragment.paint.style.filters.push(blur);
        let cull = PaintRect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        let mut walk = PrePaintTreeWalk::default();
        let node = PaintPropertyTreeBuilder::new(&mut walk).BuildNode(
            &fragment,
            Offset::default(),
            Offset::default(),
            Some(cull),
            Some(cull),
            Vec::new(),
            1.0,
            Offset::default(),
            false,
            PropertyTreeState::default(),
            None,
            0,
            true,
            true,
            Offset::default(),
            false,
            None,
            false,
            None,
            None,
            false,
            false,
        );
        assert!(node.cull_rect.is_none());
        assert!(node.self_cull_rect.is_none());
        assert!(
            !node
                .cache
                .as_ref()
                .unwrap()
                .children
                .subtree_should_use_infinite_cull_rect
        );
        assert_eq!(
            node.properties.nodes.effect.filter_reference_box,
            Some(PaintRect {
                x: 1.5,
                y: 2.0,
                width: 20.0,
                height: 25.0,
            })
        );
        assert_eq!(
            node.properties.nodes.effect.filter_viewport,
            Some(PaintRect {
                x: 0.0,
                y: 0.0,
                width: 50.0,
                height: 40.0,
            })
        );
    }

    #[test]
    fn layout_view_gets_a_paint_offset_translation_slot_at_zero_offset() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.has_source = true;
        fragment.paint.establishes_paint_state = true;
        fragment.paint.is_box_model_object = true;
        fragment.paint.is_layout_view = true;
        assert!(NeedsPaintOffsetTranslation(
            &fragment,
            Some(&fragment.paint.style)
        ));

        let mut transforms = Vec::new();
        let geometry = ResolveFragmentGeometry(
            &fragment,
            fragment.offset,
            Offset::default(),
            Offset::default(),
            &mut transforms,
            Offset::default(),
            Offset::default(),
        );
        assert!(geometry.applies_paint_offset_translation);
        assert_eq!(transforms.len(), 1);
        assert_eq!(transforms[0], Default::default());
    }

    #[test]
    fn paint_offset_translation_rounds_in_css_pixels_at_high_dpr() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.has_source = true;
        fragment.paint.establishes_paint_state = true;
        fragment.paint.is_box_model_object = true;
        fragment.paint.is_layout_view = true;
        fragment.resources = Some(Arc::new(PaintResources {
            device_pixel_ratio: 2.0,
            ..Default::default()
        }));

        let mut transforms = Vec::new();
        let geometry = ResolveFragmentGeometry(
            &fragment,
            fragment.offset,
            Offset { x: 0.3, y: 0.0 },
            Offset::default(),
            &mut transforms,
            Offset::default(),
            Offset::default(),
        );
        assert_eq!(transforms[0].values[12], 0.0);
        assert_eq!(geometry.paint_offset.x, 0.3);
    }

    #[test]
    fn svg_root_transform_snaps_residual_and_scales_only_document_root() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.source_kind = NodeKind::kSvgRoot;
        fragment.size = Size {
            width: 100.0,
            height: 99.984375,
        };
        fragment.paint.svg_viewport = Some(Default::default());
        let paint_offset = Offset { x: 0.0, y: 0.5 };

        let inline =
            SvgTransformToPixelSnappedBorderBox(&fragment, paint_offset, Offset::default(), 1.0)
                .unwrap();
        assert_eq!(inline.values[12], 0.0);
        assert_eq!(inline.values[13], 1.0);
        assert_eq!(inline.values[0], 1.0);
        assert_eq!(inline.values[5], 1.0);

        fragment
            .paint
            .svg_viewport
            .as_mut()
            .unwrap()
            .is_document_element = true;
        let document =
            SvgTransformToPixelSnappedBorderBox(&fragment, paint_offset, Offset::default(), 1.0)
                .unwrap();
        assert_eq!(document.values[12], 0.0);
        assert_eq!(document.values[13], 1.0);
        assert_eq!(document.values[0], 1.0);
        assert_eq!(document.values[5], 99.0 / 99.984375);
    }

    #[test]
    fn inline_svg_pixel_snap_adjustment_chooses_uniform_or_nonuniform_scale() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.source_kind = NodeKind::kSvgRoot;
        fragment.paint.svg_viewport = Some(Default::default());
        fragment
            .paint
            .svg_viewport
            .as_mut()
            .unwrap()
            .inline_root_pixel_snapping_scale_adjustment = true;
        fragment.size = Size {
            width: 100.4,
            height: 99.8,
        };

        let uniform = SvgTransformToPixelSnappedBorderBox(
            &fragment,
            Offset::default(),
            Offset::default(),
            1.0,
        )
        .unwrap();
        assert_eq!(uniform.values[0], 100.0 / 100.4);
        assert_eq!(uniform.values[5], 100.0 / 100.4);

        fragment.size = Size {
            width: 1000.4,
            height: 10.4,
        };
        let nonuniform = SvgTransformToPixelSnappedBorderBox(
            &fragment,
            Offset::default(),
            Offset::default(),
            1.0,
        )
        .unwrap();
        assert_eq!(nonuniform.values[0], 1000.0 / 1000.4);
        assert_eq!(nonuniform.values[5], 10.0 / 10.4);
    }

    #[test]
    fn svg_root_transform_snaps_in_artifact_device_space() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.source_kind = NodeKind::kSvgRoot;
        fragment.size = Size {
            width: 100.0,
            height: 100.0,
        };
        fragment.paint.svg_viewport = Some(Default::default());

        let transform = SvgTransformToPixelSnappedBorderBox(
            &fragment,
            Offset { x: 0.0, y: 1314.5 },
            Offset::default(),
            2.0,
        )
        .unwrap();

        assert_eq!(transform.values[13], 1314.5);

        let property_space = SvgTransformToPixelSnappedBorderBox(
            &fragment,
            Offset { x: 0.3, y: 0.3 },
            Offset { x: 0.1, y: 0.1 },
            2.0,
        )
        .unwrap();
        assert_eq!(property_space.values[12], 0.4);
        assert_eq!(property_space.values[13], 0.4);

        fragment.size.width = 0.04;
        fragment
            .paint
            .svg_viewport
            .as_mut()
            .unwrap()
            .is_document_element = true;
        let tiny_document = SvgTransformToPixelSnappedBorderBox(
            &fragment,
            Offset::default(),
            Offset::default(),
            2.0,
        )
        .unwrap();
        assert_eq!(tiny_document.values[0] * fragment.ViewSize().width, 0.5);
    }

    #[test]
    fn sticky_translation_rounds_total_and_retains_local_remainder() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.has_source = true;
        fragment.paint.establishes_paint_state = true;
        fragment.paint.is_box_model_object = true;
        fragment.paint.position = Position::kSticky;
        fragment.paint.sticky_constraint = Some(Default::default());
        fragment.paint.sticky_offset.x = 0.5;

        let mut transforms = Vec::new();
        let geometry = ResolveFragmentGeometry(
            &fragment,
            fragment.offset,
            Offset { x: 0.25, y: 0.0 },
            Offset::default(),
            &mut transforms,
            Offset::default(),
            Offset::default(),
        );
        assert_eq!(transforms.len(), 2);
        assert_eq!(transforms[1].values[12], 1.0);
        assert_eq!(geometry.sticky_pixel_snap_offset.x, 0.25);
        assert_eq!(geometry.paint_offset.x, -0.25);

        fragment.paint.sticky_offset.x = 0.0;
        transforms.clear();
        let geometry = ResolveFragmentGeometry(
            &fragment,
            fragment.offset,
            Offset { x: -0.5, y: 0.0 },
            Offset::default(),
            &mut transforms,
            Offset::default(),
            Offset::default(),
        );
        assert_eq!(transforms[1].values[12], 0.0);
        assert_eq!(geometry.sticky_pixel_snap_offset.x, -0.5);
        assert_eq!(geometry.paint_offset.x, -0.5);
    }

    #[test]
    fn independent_transform_properties_keep_source_order_and_slots() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.has_source = true;
        fragment.paint.establishes_paint_state = true;
        fragment.paint.is_box_model_object = true;
        fragment.paint.style.translate = Some(Default::default());
        fragment.paint.style.rotate = Some(Default::default());
        fragment.paint.style.scale = Some(Default::default());
        fragment.paint.style.offset_transform = Some(Default::default());
        fragment.paint.style.transform = Some(Default::default());

        assert_eq!(
            TransformPropertyRoles(&fragment, &fragment.paint.style),
            vec![
                PaintPropertyRole::Translate,
                PaintPropertyRole::Rotate,
                PaintPropertyRole::Scale,
                PaintPropertyRole::Offset,
                PaintPropertyRole::Transform,
            ]
        );
        let mut transforms = Vec::new();
        let geometry = ResolveFragmentGeometry(
            &fragment,
            fragment.offset,
            Offset::default(),
            Offset::default(),
            &mut transforms,
            Offset::default(),
            Offset::default(),
        );
        assert!(geometry.applies_paint_offset_translation);
        assert_eq!(transforms.len(), 6);

        let mut walk = PrePaintTreeWalk::default();
        let node = PaintPropertyTreeBuilder::new(&mut walk).BuildNode(
            &fragment,
            Offset::default(),
            Offset::default(),
            None,
            None,
            Vec::new(),
            1.0,
            Offset::default(),
            false,
            PropertyTreeState::default(),
            None,
            0,
            true,
            true,
            Offset::default(),
            false,
            None,
            false,
            None,
            None,
            false,
            false,
        );
        assert!(Arc::ptr_eq(
            &node.pre_transform,
            node.paint_offset_translation
                .as_ref()
                .expect("CSS transform properties follow paint-offset translation"),
        ));
        assert!(!Arc::ptr_eq(
            &node.pre_transform,
            &node.properties.nodes.transform,
        ));
    }

    #[test]
    fn will_change_effects_retain_identity_nodes_and_additional_transform_reason() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.has_source = true;
        fragment.paint.establishes_paint_state = true;
        fragment.paint.is_box_model_object = true;
        fragment.paint.display_item_client_id = 1;
        fragment.paint.style.will_change_opacity = true;
        fragment.paint.style.will_change_filter = true;
        fragment.paint.style.will_change_transform = true;
        fragment.paint.style.will_change_transform_property = true;

        let mut walk = PrePaintTreeWalk::default();
        let node = PaintPropertyTreeBuilder::new(&mut walk).BuildNode(
            &fragment,
            Offset::default(),
            Offset::default(),
            None,
            None,
            Vec::new(),
            1.0,
            Offset::default(),
            false,
            PropertyTreeState::default(),
            None,
            0,
            true,
            true,
            Offset::default(),
            false,
            None,
            false,
            None,
            None,
            false,
            false,
        );
        let roles: Vec<_> = node
            .cache
            .as_ref()
            .unwrap()
            .own_keys
            .iter()
            .map(|(_, key)| key.role)
            .collect();
        assert!(roles.contains(&PaintPropertyRole::Effect));
        assert!(roles.contains(&PaintPropertyRole::Filter));
        let filter = &node.properties.nodes.effect;
        assert!(filter
            .direct_compositing_reasons
            .Has(foundation::CompositingReason::kWillChangeFilter));
        assert!(filter
            .direct_compositing_reasons
            .Has(foundation::CompositingReason::kWillChangeTransform));
        let effect = filter.parent.as_ref().expect("filter parent effect");
        assert!(effect
            .direct_compositing_reasons
            .Has(foundation::CompositingReason::kWillChangeOpacity));
        assert!(effect
            .direct_compositing_reasons
            .Has(foundation::CompositingReason::kWillChangeTransform));
    }

    #[test]
    fn printing_does_not_create_a_backdrop_filter_effect() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.has_source = true;
        fragment.paint.establishes_paint_state = true;
        fragment.paint.is_box_model_object = true;
        fragment.paint.is_printing = true;
        fragment.paint.display_item_client_id = 1;
        fragment
            .paint
            .style
            .backdrop_filters
            .push(Default::default());

        let mut walk = PrePaintTreeWalk::default();
        let node = PaintPropertyTreeBuilder::new(&mut walk).BuildNode(
            &fragment,
            Offset::default(),
            Offset::default(),
            None,
            None,
            Vec::new(),
            1.0,
            Offset::default(),
            false,
            PropertyTreeState::default(),
            None,
            0,
            true,
            true,
            Offset::default(),
            false,
            None,
            false,
            None,
            None,
            false,
            false,
        );

        assert_eq!(node.properties.nodes.effect.id, 0);
        assert!(!node.applies_backdrop_filter);
        assert!(node
            .cache
            .as_ref()
            .unwrap()
            .own_keys
            .iter()
            .all(|(_, key)| key.role != PaintPropertyRole::BackdropFilterClip
                && key.role != PaintPropertyRole::Effect));
    }

    #[test]
    fn backdrop_effect_appends_foreground_filters_but_not_box_reflection() {
        use paint_model::cc::paint::filter_operation::FilterType;

        let mut fragment = FragmentPaintData::default();
        fragment.paint.has_source = true;
        fragment.paint.establishes_paint_state = true;
        fragment.paint.is_box_model_object = true;
        fragment.paint.display_item_client_id = 1;
        fragment.size = Size {
            width: 40.0,
            height: 30.0,
        };
        fragment
            .paint
            .style
            .backdrop_filters
            .push(PaintFilterOperation {
                r#type: PaintFilterType::kOpacity,
                amount: 0.25,
                ..Default::default()
            });
        fragment.paint.style.filters.extend([
            PaintFilterOperation {
                r#type: PaintFilterType::kGrayscale,
                amount: 0.5,
                ..Default::default()
            },
            PaintFilterOperation {
                r#type: PaintFilterType::kBoxReflect,
                ..Default::default()
            },
        ]);

        let mut walk = PrePaintTreeWalk::default();
        let node = PaintPropertyTreeBuilder::new(&mut walk).BuildNode(
            &fragment,
            Offset::default(),
            Offset::default(),
            None,
            None,
            Vec::new(),
            1.0,
            Offset::default(),
            false,
            PropertyTreeState::default(),
            None,
            0,
            true,
            true,
            Offset::default(),
            false,
            None,
            false,
            None,
            None,
            false,
            false,
        );

        let foreground = &node.properties.nodes.effect;
        let backdrop = foreground
            .parent
            .as_ref()
            .expect("foreground filter must follow the primary backdrop effect");
        let operations = backdrop
            .backdrop_filters
            .canonical()
            .expect("box reflection belongs only to the foreground chain");
        assert_eq!(operations.size(), 2);
        assert_eq!(operations.at(0).r#type(), FilterType::OPACITY);
        assert_eq!(operations.at(1).r#type(), FilterType::GRAYSCALE);
        assert_eq!(
            backdrop.filter_reference_box,
            Some(PaintRect {
                width: 40.0,
                height: 30.0,
                ..PaintRect::default()
            })
        );
        let foreground_operations = foreground
            .filters
            .canonical()
            .expect("resolved box reflection is a canonical reference filter");
        assert_eq!(foreground_operations.size(), 2);
        assert_eq!(foreground_operations.at(0).r#type(), FilterType::GRAYSCALE);
        assert_eq!(foreground_operations.at(1).r#type(), FilterType::REFERENCE);
    }

    #[test]
    fn transform_cull_optimization_consumes_the_retained_paint_result() {
        let cull = PaintRect {
            width: 100.0,
            height: 100.0,
            ..PaintRect::default()
        };
        let build = |active_animation, previous_paint_result| {
            let mut fragment = FragmentPaintData::default();
            fragment.paint.has_source = true;
            fragment.paint.establishes_paint_state = true;
            fragment.paint.is_box_model_object = true;
            fragment.paint.style.transform = Some(Default::default());
            fragment.paint.style.active_transform_animation = active_animation;
            fragment.size = Size {
                width: 20.0,
                height: 20.0,
            };
            let mut walk = PrePaintTreeWalk::default();
            PaintPropertyTreeBuilder::new(&mut walk)
                .with_previous_paint_result(previous_paint_result)
                .BuildNode(
                    &fragment,
                    Offset::default(),
                    Offset::default(),
                    Some(cull),
                    Some(cull),
                    Vec::new(),
                    1.0,
                    Offset::default(),
                    false,
                    PropertyTreeState::default(),
                    None,
                    0,
                    true,
                    true,
                    Offset::default(),
                    false,
                    None,
                    false,
                    None,
                    None,
                    false,
                    false,
                )
        };

        let incomplete = build(false, paint_model::PaintResult::kMayBeClippedByCullRect);
        assert!(incomplete.cull_rect.is_some());
        let fully_painted = build(false, paint_model::PaintResult::kFullyPainted);
        assert!(fully_painted.cull_rect.is_none());
        assert!(
            !fully_painted
                .cache
                .as_ref()
                .unwrap()
                .children
                .subtree_should_use_infinite_cull_rect
        );
        let active = build(true, paint_model::PaintResult::kMayBeClippedByCullRect);
        assert!(active.cull_rect.is_none());
        assert!(
            !active
                .cache
                .as_ref()
                .unwrap()
                .children
                .subtree_should_use_infinite_cull_rect
        );
    }

    #[test]
    fn canvas_subtree_rejects_tainted_filters_and_foreground_taint_propagates() {
        let build = |mut fragment: FragmentPaintData| {
            fragment.paint.has_source = true;
            fragment.paint.establishes_paint_state = true;
            fragment.paint.is_box_model_object = true;
            fragment.paint.display_item_client_id = 1;
            fragment.size = Size {
                width: 40.0,
                height: 30.0,
            };
            let mut walk = PrePaintTreeWalk::default();
            PaintPropertyTreeBuilder::new(&mut walk).BuildNode(
                &fragment,
                Offset::default(),
                Offset::default(),
                None,
                None,
                Vec::new(),
                1.0,
                Offset::default(),
                false,
                PropertyTreeState::default(),
                None,
                0,
                true,
                true,
                Offset::default(),
                false,
                None,
                false,
                None,
                None,
                false,
                false,
            )
        };
        let mut untainted = FragmentPaintData::default();
        untainted.paint.style.filters.push(PaintFilterOperation {
            r#type: PaintFilterType::kDropShadow,
            ..Default::default()
        });
        let untainted = build(untainted);
        assert!(!OwnFilterOriginTainted(&untainted));

        let shadow = PaintFilterOperation {
            r#type: PaintFilterType::kDropShadow,
            uses_current_color: true,
            ..Default::default()
        };

        let mut foreground = FragmentPaintData::default();
        foreground.paint.style.filters.push(shadow.clone());
        let foreground = build(foreground);
        assert!(foreground.properties.nodes.effect.filters.origin_tainted());
        assert!(
            foreground
                .properties
                .nodes
                .effect
                .compositor
                .is_in_tainted_subtree
        );
        assert!(OwnFilterOriginTainted(&foreground));
        assert!(FilterTaintChanged(false, &foreground));
        assert!(FilterTaintChanged(true, &untainted));
        let mut inherited_only = PaintTreeNode::default();
        inherited_only.properties.nodes.effect = foreground.properties.nodes.effect.clone();
        assert!(!OwnFilterOriginTainted(&inherited_only));

        let mut canvas_foreground = FragmentPaintData::default();
        canvas_foreground.paint.canvas_draw_element_enabled = true;
        canvas_foreground.paint.is_in_canvas_subtree = true;
        canvas_foreground.paint.style.filters.push(shadow.clone());
        let canvas_foreground = build(canvas_foreground);
        assert!(canvas_foreground.properties.nodes.effect.filters.is_empty());
        assert!(
            !canvas_foreground
                .properties
                .nodes
                .effect
                .compositor
                .has_filter_info
        );
        assert!(
            !canvas_foreground
                .properties
                .nodes
                .effect
                .compositor
                .is_in_tainted_subtree
        );

        let mut drawable_canvas = FragmentPaintData::default();
        drawable_canvas
            .paint
            .additional_compositing_reasons
            .Put(foundation::CompositingReason::kCanvasChild);
        drawable_canvas
            .paint
            .style
            .filters
            .push(PaintFilterOperation {
                r#type: PaintFilterType::kOpacity,
                amount: 0.5,
                ..Default::default()
            });
        drawable_canvas.paint.scrollbars = Some(Arc::new(ScrollbarPaintData {
            horizontal: Some(ScrollbarPaintAxis::default()),
            uses_overlay_scrollbars: true,
            ..Default::default()
        }));
        let drawable_canvas = build(drawable_canvas);
        assert!(
            drawable_canvas
                .properties
                .nodes
                .effect
                .compositor
                .is_in_drawable_canvas_subtree
        );
        let scrollbar_effect = drawable_canvas
            .horizontal_scrollbar_properties
            .as_ref()
            .expect("overlay canvas scrollbar effect")
            .effect
            .as_ref();
        assert!(scrollbar_effect.compositor.is_in_drawable_canvas_subtree);
        assert!(!scrollbar_effect
            .direct_compositing_reasons
            .Has(foundation::CompositingReason::kActiveOpacityAnimation));

        let mut canvas_backdrop = FragmentPaintData::default();
        canvas_backdrop.paint.canvas_draw_element_enabled = true;
        canvas_backdrop.paint.is_in_canvas_subtree = true;
        canvas_backdrop.paint.style.backdrop_filters.push(shadow);
        let canvas_backdrop = build(canvas_backdrop);
        assert!(canvas_backdrop
            .properties
            .nodes
            .effect
            .backdrop_filters
            .is_empty());
        assert!(
            !canvas_backdrop
                .properties
                .nodes
                .effect
                .compositor
                .has_backdrop_filter_info
        );
    }

    #[test]
    fn svg_children_do_not_get_html_paint_offset_translation() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.has_source = true;
        fragment.paint.is_box_model_object = true;
        fragment.paint.source_kind = NodeKind::kSvgForeignObject;
        fragment.paint.style.transform = Some(Default::default());
        assert!(!NeedsPaintOffsetTranslation(
            &fragment,
            Some(&fragment.paint.style)
        ));
    }

    #[test]
    fn svg_foreign_object_mask_clip_uses_css_descendant_overflow_bounds() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.source_kind = NodeKind::kSvgForeignObject;
        // Layout exports LocalBoundingBoxIncludingSelfPaintingDescendants for
        // a no-clip foreignObject mask. The negative origin and oversized
        // extent represent descendant overflow outside the element itself.
        fragment.paint.css_mask_bounding_box = Some(FragmentPaintRect {
            offset: Offset { x: -8.0, y: -6.0 },
            size: Size {
                width: 140.0,
                height: 120.0,
            },
        });
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        node.paint_offset = Offset { x: 20.0, y: 30.0 };

        assert_eq!(
            node.ResolvedMaskClipRect(),
            Some(PaintRect {
                x: 12.0,
                y: 24.0,
                width: 140.0,
                height: 120.0,
            })
        );
    }

    #[test]
    fn svg_clip_path_does_not_reapply_parent_paint_offset() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.has_source = true;
        fragment.paint.establishes_paint_state = true;
        fragment.paint.source_kind = NodeKind::kSvgShape;
        fragment.paint.style.clip_path =
            Some(paint_input_types::paint_input::ClipPathPaint {
                inset: Some(Default::default()),
                reference_box:
                    paint_input_types::paint_input::ClipPathReferenceBox::kFillBox,
                ..Default::default()
            });
        fragment.size = Size {
            width: 100.0,
            height: 60.0,
        };

        let mut walk = PrePaintTreeWalk::default();
        let node = PaintPropertyTreeBuilder::new(&mut walk).BuildNode(
            &fragment,
            Offset { x: 40.0, y: 30.0 },
            Offset::default(),
            None,
            None,
            Vec::new(),
            1.0,
            Offset::default(),
            false,
            PropertyTreeState::default(),
            None,
            0,
            true,
            true,
            Offset::default(),
            false,
            None,
            false,
            None,
            None,
            false,
            false,
        );

        assert_eq!(
            node.properties.nodes.clip.rect,
            Some(PaintRect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 60.0,
            })
        );
    }

    #[test]
    fn svg_clip_path_stroke_box_requires_exact_layout_geometry() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.has_source = true;
        fragment.paint.establishes_paint_state = true;
        fragment.paint.source_kind = NodeKind::kSvgShape;
        fragment.paint.style.clip_path =
            Some(paint_input_types::paint_input::ClipPathPaint {
                inset: Some(Default::default()),
                reference_box:
                    paint_input_types::paint_input::ClipPathReferenceBox::kStrokeBox,
                ..Default::default()
            });
        fragment.size = Size {
            width: 100.0,
            height: 60.0,
        };

        let mut walk = PrePaintTreeWalk::default();
        let node = PaintPropertyTreeBuilder::new(&mut walk).BuildNode(
            &fragment,
            Offset::default(),
            Offset::default(),
            None,
            None,
            Vec::new(),
            1.0,
            Offset::default(),
            false,
            PropertyTreeState::default(),
            None,
            0,
            true,
            true,
            Offset::default(),
            false,
            None,
            false,
            None,
            None,
            false,
            false,
        );

        assert_eq!(
            node.properties.nodes.clip.clip_path_error.as_deref(),
            Some("exact SVG stroke-box geometry is not exported by Layout")
        );
    }

    #[test]
    fn geometry_box_clip_path_uses_contoured_border_geometry() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.has_source = true;
        fragment.paint.establishes_paint_state = true;
        fragment.paint.style.clip_path =
            Some(paint_input_types::paint_input::ClipPathPaint {
                geometry_box_only: true,
                reference_box:
                    paint_input_types::paint_input::ClipPathReferenceBox::kBorderBox,
                ..Default::default()
            });
        fragment.size = Size {
            width: 100.0,
            height: 60.0,
        };

        let mut walk = PrePaintTreeWalk::default();
        let node = PaintPropertyTreeBuilder::new(&mut walk).BuildNode(
            &fragment,
            Offset::default(),
            Offset::default(),
            None,
            None,
            Vec::new(),
            1.0,
            Offset::default(),
            false,
            PropertyTreeState::default(),
            None,
            0,
            true,
            true,
            Offset::default(),
            false,
            None,
            false,
            None,
            None,
            false,
            false,
        );

        assert!(node.properties.nodes.clip.clip_path_error.is_none());
        assert!(!node.local_clip_path.is_empty());
        assert_eq!(
            node.properties.nodes.clip.rect,
            Some(PaintRect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 60.0,
            })
        );
    }

    #[test]
    fn geometry_box_clip_path_rejects_unrepresented_corner_curvature() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.has_source = true;
        fragment.paint.establishes_paint_state = true;
        fragment.paint.style.border_radii = Some(PaintCornerRadii {
            top_left: paint_input_types::paint_input::PaintCornerRadius {
                x: 8.0,
                y: 8.0,
            },
            ..Default::default()
        });
        fragment.paint.style.border_corner_curvatures() =
            paint_input_types::paint_input::PaintCornerCurvatures([4.0, 2.0, 2.0, 2.0]);
        fragment.paint.style.clip_path =
            Some(paint_input_types::paint_input::ClipPathPaint {
                geometry_box_only: true,
                ..Default::default()
            });
        fragment.size = Size {
            width: 100.0,
            height: 60.0,
        };

        let mut walk = PrePaintTreeWalk::default();
        let node = PaintPropertyTreeBuilder::new(&mut walk).BuildNode(
            &fragment,
            Offset::default(),
            Offset::default(),
            None,
            None,
            Vec::new(),
            1.0,
            Offset::default(),
            false,
            PropertyTreeState::default(),
            None,
            0,
            true,
            true,
            Offset::default(),
            false,
            None,
            false,
            None,
            None,
            false,
            false,
        );

        assert_eq!(
            node.properties.nodes.clip.clip_path_error.as_deref(),
            Some("clip-path geometry-box with non-round corner-shape")
        );
    }

    #[test]
    fn unavailable_shape_clip_path_does_not_become_none_or_a_fabricated_path() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.has_source = true;
        fragment.paint.establishes_paint_state = true;
        fragment.paint.style.clip_path =
            Some(paint_input_types::paint_input::ClipPathPaint {
                reference_error: Some("clip-path shape()/ray() is unsupported".into()),
                ..Default::default()
            });
        fragment.size = Size {
            width: 100.0,
            height: 60.0,
        };

        let mut walk = PrePaintTreeWalk::default();
        let node = PaintPropertyTreeBuilder::new(&mut walk).BuildNode(
            &fragment,
            Offset::default(),
            Offset::default(),
            None,
            None,
            Vec::new(),
            1.0,
            Offset::default(),
            false,
            PropertyTreeState::default(),
            None,
            0,
            true,
            true,
            Offset::default(),
            false,
            None,
            false,
            None,
            None,
            false,
            false,
        );

        assert!(node.applies_clip_path);
        assert_eq!(
            node.properties.nodes.clip.clip_path_error.as_deref(),
            Some("clip-path shape()/ray() is unsupported")
        );
    }

    #[test]
    fn direct_reason_on_layout_inline_requires_self_painting_layer() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.has_source = true;
        fragment.paint.is_box_model_object = true;
        fragment.paint.is_layout_inline = true;
        fragment.paint.style.will_change_transform = true;
        fragment.paint.style.will_change_transform_property = true;
        assert!(!NeedsPaintOffsetTranslation(
            &fragment,
            Some(&fragment.paint.style)
        ));
        fragment.paint.paint_layer_is_self_painting = true;
        assert!(NeedsPaintOffsetTranslation(
            &fragment,
            Some(&fragment.paint.style)
        ));
    }

    #[test]
    fn svg_root_overflow_clip_follows_computed_overflow() {
        let mut svg = FragmentPaintData::default();
        svg.paint.has_source = true;
        svg.paint.establishes_paint_state = true;
        svg.paint.source_kind = NodeKind::kSvgRoot;
        svg.paint.is_layout_box = true;

        // Chromium's NeedsOverflowClip(LayoutSVGRoot) permits outermost SVG
        // ink overflow when the computed overflow axes are visible.
        assert!(!ClipsOverflow(&svg));
        svg.paint.overflow_x = Overflow::kHidden;
        svg.paint.overflow_y = Overflow::kHidden;
        svg.paint.should_clip_overflow_along_either_axis = true;
        svg.paint.should_clip_overflow_along_both_axes = true;
        assert!(ClipsOverflow(&svg));
    }

    #[test]
    fn replaced_image_without_rounding_does_not_create_redundant_overflow_clip() {
        let mut image = FragmentPaintData::default();
        image.paint.has_source = true;
        image.paint.establishes_paint_state = true;
        image.paint.is_layout_box = true;
        image.paint.replaced_content =
            Some(layout::paint::ReplacedContentPaintData {
                respects_css_overflow: false,
                clips_to_content_box: true,
                is_image: true,
                ..Default::default()
            });
        assert!(!NeedsOverflowClip(&image));
    }

    #[test]
    fn inner_border_radius_clip_precedes_rectangular_overflow_clip() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.has_source = true;
        fragment.paint.establishes_paint_state = true;
        fragment.paint.is_box_model_object = true;
        fragment.paint.is_layout_box = true;
        fragment.paint.should_clip_overflow_along_either_axis = true;
        fragment.paint.should_clip_overflow_along_both_axes = true;
        fragment.paint.display_item_client_id = 1;
        fragment.paint.overflow_x = Overflow::kHidden;
        fragment.paint.overflow_y = Overflow::kHidden;
        fragment.size = Size {
            width: 100.0,
            height: 80.0,
        };
        let radius = paint_input_types::paint_input::PaintCornerRadius { x: 8.0, y: 8.0 };
        fragment.paint.style.border_radii = Some(PaintCornerRadii {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        });

        let mut walk = PrePaintTreeWalk::default();
        let node = PaintPropertyTreeBuilder::new(&mut walk).BuildNode(
            &fragment,
            Offset::default(),
            Offset::default(),
            None,
            None,
            Vec::new(),
            1.0,
            Offset::default(),
            false,
            PropertyTreeState::default(),
            None,
            0,
            true,
            true,
            Offset::default(),
            false,
            None,
            false,
            None,
            None,
            false,
            false,
        );
        let roles: Vec<_> = node
            .cache
            .as_ref()
            .unwrap()
            .own_keys
            .iter()
            .map(|(_, key)| key.role)
            .collect();
        let inner = roles
            .iter()
            .position(|role| *role == PaintPropertyRole::InnerBorderRadiusClip)
            .unwrap();
        let overflow = roles
            .iter()
            .position(|role| *role == PaintPropertyRole::OverflowClip)
            .unwrap();
        assert!(inner < overflow);
        assert!(node.local_inner_border_radius_clip.is_some());
        assert!(node.local_inner_border_radius_radii.HasRadius());
        assert!(!node.contents_properties.nodes.clip.radii.HasRadius());
        assert!(node
            .contents_properties
            .nodes
            .clip
            .parent
            .as_ref()
            .is_some_and(|clip| clip.radii.HasRadius()));
    }

    #[test]
    fn overflow_controls_outside_border_get_their_own_clip_slot() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.has_source = true;
        fragment.paint.establishes_paint_state = true;
        fragment.paint.is_box_model_object = true;
        fragment.paint.is_layout_box = true;
        fragment.paint.display_item_client_id = 1;
        fragment.size = Size {
            width: 100.0,
            height: 80.0,
        };
        for scrollbars in [
            ScrollbarPaintData {
                vertical: Some(ScrollbarPaintAxis {
                    track_offset: Offset { x: 95.0, y: 0.0 },
                    track_size: Size {
                        width: 10.0,
                        height: 80.0,
                    },
                    ..Default::default()
                }),
                ..Default::default()
            },
            // PaintLayerScrollableArea::ScrollCornerAndResizerRect() falls
            // back to the pointer resizer when there is no scroll corner.
            ScrollbarPaintData {
                resizer_pointer_offset: Offset { x: 95.0, y: 65.0 },
                resizer_pointer_size: Size {
                    width: 10.0,
                    height: 15.0,
                },
                ..Default::default()
            },
        ] {
            fragment.paint.scrollbars = Some(Arc::new(scrollbars));
            let mut walk = PrePaintTreeWalk::default();
            let node = PaintPropertyTreeBuilder::new(&mut walk).BuildNode(
                &fragment,
                Offset::default(),
                Offset::default(),
                None,
                None,
                Vec::new(),
                1.0,
                Offset::default(),
                false,
                PropertyTreeState::default(),
                None,
                0,
                true,
                true,
                Offset::default(),
                false,
                None,
                false,
                None,
                None,
                false,
                false,
            );
            assert!(node.overflow_controls_properties.is_some());
            assert!(node
                .cache
                .as_ref()
                .unwrap()
                .own_keys
                .iter()
                .any(|(_, key)| { key.role == PaintPropertyRole::OverflowControlsClip }));
        }
    }

    #[test]
    fn scroll_translation_uses_position_and_child_paint_offset_uses_origin() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.has_source = true;
        fragment.paint.establishes_paint_state = true;
        fragment.paint.is_box_model_object = true;
        fragment.paint.is_layout_box = true;
        fragment.paint.scroll_offset = Offset { x: 7.0, y: 11.0 };
        fragment.paint.scroll_container = Some(ScrollContainerPaintData {
            compositor_element_id: Default::default(),
            scrollable_axes: [true, true],
            scroll_origin: Offset { x: 40.0, y: 30.0 },
            scroll_position: Offset { x: 47.0, y: 41.0 },
            background_needs_repaint_on_scroll: Some(false),
            prefers_non_composited_scrolling: Some(false),
            scroll_snap_type: Some(Default::default()),
            container_rect: Default::default(),
            scrolling_background_visual_rect: Default::default(),
            minimum_scroll_offset: Offset::default(),
            maximum_scroll_offset: Offset { x: 100.0, y: 80.0 },
            user_scrollable_horizontal: true,
            user_scrollable_vertical: true,
            scrolling_background_client_id: 0,
            scrolling_background_client_invalidation_reason: None,
            scrolling_background_client_is_cacheable: true,
            scrolling_background_client_is_just_created: std::cell::Cell::new(false),
        });

        let mut walk = PrePaintTreeWalk::default();
        let node = PaintPropertyTreeBuilder::new(&mut walk).BuildNode(
            &fragment,
            Offset::default(),
            Offset::default(),
            None,
            None,
            Vec::new(),
            1.0,
            Offset::default(),
            false,
            PropertyTreeState::default(),
            None,
            0,
            true,
            true,
            Offset::default(),
            false,
            None,
            false,
            None,
            None,
            false,
            false,
        );
        let translation = &node.contents_properties.nodes.transform.matrix;
        assert_eq!(
            (translation.values[12], translation.values[13]),
            (-47.0, -41.0)
        );

        let child_context = &node.cache.as_ref().unwrap().children;
        assert_eq!(child_context.paint_space, Offset { x: -40.0, y: -30.0 });
        let mut child = FragmentPaintData::default();
        child.offset = Offset { x: 5.0, y: 6.0 };
        let mut transforms = child_context.transforms.to_vec();
        let geometry = ResolveFragmentGeometry(
            &child,
            child_context.origin,
            child_context.paint_space,
            &mut transforms,
            Offset::default(),
            Offset::default(),
        );
        assert_eq!(geometry.paint_offset, Offset { x: 45.0, y: 36.0 });
        assert_eq!(
            (
                geometry.paint_offset.x + translation.values[12],
                geometry.paint_offset.y + translation.values[13],
            ),
            (-2.0, -5.0)
        );
    }

    #[test]
    fn scroll_node_uses_source_composited_scrolling_preference() {
        fn build(
            prefers_non_composited: bool,
            force_preferred: bool,
        ) -> Box<PaintTreeNode<'static>> {
            let mut fragment = FragmentPaintData::default();
            fragment.paint.has_source = true;
            fragment.paint.establishes_paint_state = true;
            fragment.paint.is_box_model_object = true;
            fragment.paint.is_layout_box = true;
            fragment.paint.should_clip_overflow_along_either_axis = true;
            fragment.paint.should_clip_overflow_along_both_axes = true;
            fragment.paint.overflow_x = Overflow::kAuto;
            fragment.paint.overflow_y = Overflow::kAuto;
            fragment.size = Size {
                width: 100.0,
                height: 80.0,
            };
            fragment.paint.scroll_size = Size {
                width: 200.0,
                height: 160.0,
            };
            fragment.paint.style.will_change_opacity = force_preferred;
            fragment.paint.scroll_container =
                Some(layout::paint::ScrollContainerPaintData {
                    compositor_element_id: Default::default(),
                    scrollable_axes: [true, true],
                    scroll_origin: Offset::default(),
                    scroll_position: fragment.paint.ScrollOffset(),
                    background_needs_repaint_on_scroll: Some(false),
                    prefers_non_composited_scrolling: Some(prefers_non_composited),
                    scroll_snap_type: Some(Default::default()),
                    container_rect: Default::default(),
                    scrolling_background_visual_rect: Default::default(),
                    minimum_scroll_offset: Offset::default(),
                    maximum_scroll_offset: Offset { x: 100.0, y: 80.0 },
                    user_scrollable_horizontal: true,
                    user_scrollable_vertical: true,
                    scrolling_background_client_id: 0,
                    scrolling_background_client_invalidation_reason: Some(
                        foundation::PaintInvalidationReason::kNone,
                    ),
                    scrolling_background_client_is_cacheable: true,
                    scrolling_background_client_is_just_created: std::cell::Cell::new(false),
                });

            let mut walk = PrePaintTreeWalk::default();
            Box::new(PaintPropertyTreeBuilder::new(&mut walk).BuildNode(
                &fragment,
                Offset::default(),
                Offset::default(),
                None,
                None,
                Vec::new(),
                1.0,
                Offset::default(),
                false,
                PropertyTreeState::default(),
                None,
                0,
                true,
                true,
                Offset::default(),
                false,
                None,
                false,
                None,
                None,
                false,
                false,
            ))
        }

        let not_preferred = build(true, false);
        assert_eq!(
            not_preferred
                .contents_properties
                .nodes
                .nearest_scroll()
                .expect("scroll node")
                .compositor
                .composited_scrolling_preference,
            paint_model::paint_property_tree::PaintCompositedScrollingPreference::NotPreferred
        );

        let preferred = build(true, true);
        assert_eq!(
            preferred
                .contents_properties
                .nodes
                .nearest_scroll()
                .expect("scroll node")
                .compositor
                .composited_scrolling_preference,
            paint_model::paint_property_tree::PaintCompositedScrollingPreference::Preferred
        );
    }

    #[test]
    fn clip_shapes_resolve_percentages_and_polygon_rounding() {
        let circle = ClipPathPaint {
            circle: Some(ClipPathCircle {
                center_x: ClipPathLength {
                    pixels: 0.0,
                    percentage: 50.0,
                },
                center_y: ClipPathLength {
                    pixels: 0.0,
                    percentage: 50.0,
                },
                radius: ClipPathShapeRadius {
                    kind: ClipPathRadiusKind::kValue,
                    value: ClipPathLength {
                        pixels: 0.0,
                        percentage: 25.0,
                    },
                },
            }),
            ..ClipPathPaint::default()
        };
        let mut path = Vec::new();
        let bounds = ResolveClipPath(
            &circle,
            Offset { x: 10.0, y: 20.0 },
            Size {
                width: 100.0,
                height: 100.0,
            },
            1.0,
            &mut path,
        );
        assert_eq!(
            bounds,
            PaintRect {
                x: 35.0,
                y: 45.0,
                width: 50.0,
                height: 50.0
            }
        );
        assert_eq!(
            path.iter()
                .filter(|command| command.verb == PaintPathVerb::kConicTo)
                .count(),
            4
        );

        let mut polygon = ClipPathPaint {
            polygon_rounding_radius: Some(ClipPathLength {
                pixels: 10.0,
                percentage: 0.0,
            }),
            ..ClipPathPaint::default()
        };
        polygon.commands = [
            (PaintPathVerb::kMoveTo, Offset { x: 0.0, y: 0.0 }),
            (PaintPathVerb::kLineTo, Offset { x: 100.0, y: 0.0 }),
            (PaintPathVerb::kLineTo, Offset { x: 100.0, y: 100.0 }),
            (PaintPathVerb::kLineTo, Offset { x: 0.0, y: 100.0 }),
        ]
        .into_iter()
        .map(|(verb, point)| PaintPathCommand {
            verb,
            point,
            ..PaintPathCommand::default()
        })
        .chain([PaintPathCommand {
            verb: PaintPathVerb::kClose,
            ..PaintPathCommand::default()
        }])
        .collect();
        path.clear();
        ResolveClipPath(
            &polygon,
            Offset::default(),
            Size {
                width: 100.0,
                height: 100.0,
            },
            1.0,
            &mut path,
        );
        assert_eq!(path[0].point, Offset { x: 0.0, y: 50.0 });
        assert_eq!(
            path.iter()
                .filter(|command| command.verb == PaintPathVerb::kConicTo)
                .count(),
            4
        );
    }

    #[test]
    fn svg_bitmap_mask_retains_orientation_and_no_clip_painting_area() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.source_kind = NodeKind::kSvgShape;
        fragment.paint.svg_shape = Some(Arc::new(SvgShapeData {
            geometry: SvgShapeGeometry::kRectangle,
            bounds_offset: Offset { x: 3.0, y: 4.0 },
            bounds_width: 10.0,
            bounds_height: 20.0,
            ..SvgShapeData::default()
        }));
        fragment.paint.ink_overflow = Some(FragmentPaintRect {
            offset: Offset { x: 1.0, y: 2.0 },
            size: Size {
                width: 30.0,
                height: 40.0,
            },
        });
        fragment.paint.style.respect_image_orientation = true;
        fragment.resources = Some(Arc::new(PaintResources {
            fonts: Default::default(),
            images: vec![image_resource::PaintImage {
                id: 9,
                width: 2,
                height: 1,
                orientation: image_resource::ImageOrientation::kRotate90,
                content: image_resource::PaintImageContent::Bitmap(vec![255_u8; 2 * 1 * 4].into()),
                ..Default::default()
            }].into(),
            canvases: Default::default(),
            device_pixel_ratio: 1.0,
            preferred_color_scheme: 0,
            layout_zoom_factor: 1.0,
            base_background_color: Default::default(),
            paints_opaque_base_background: false,
            should_clear_document_background: true,
            viewport: None,
        }));
        let layer = PaintMaskLayer {
            image_state: PaintMaskImageState::kReady,
            image: BackgroundImageLayer {
                resource_id: 9,
                origin: BackgroundBox::kFillBox,
                clip: BackgroundBox::kNoClip,
                ..BackgroundImageLayer::default()
            },
            ..PaintMaskLayer::default()
        };

        let resolved = ResolveMaskLayers(
            &fragment,
            fragment.resources.as_deref(),
            Offset { x: 10.0, y: 20.0 },
            Offset::default(),
            1.0,
            &[layer],
        );
        assert_eq!(resolved.len(), 1);
        assert_eq!(
            resolved[0].image_orientation,
            image_resource::ImageOrientation::kRotate90
        );
        assert_eq!(
            resolved[0].source_rect,
            PaintRect {
                width: 1.0,
                height: 2.0,
                ..PaintRect::default()
            }
        );
        assert!(!resolved[0].has_clip);
        assert_eq!(
            resolved[0].clip_rect,
            PaintRect {
                x: 11.0,
                y: 22.0,
                width: 30.0,
                height: 40.0,
            }
        );
    }

    #[test]
    fn generated_mask_image_retains_its_tile_as_the_classifier_source_rect() {
        let mut fragment = FragmentPaintData::default();
        fragment.size = Size {
            width: 40.0,
            height: 30.0,
        };
        let layer = PaintMaskLayer {
            image: BackgroundImageLayer {
                shader: Some(Arc::new(PaintShader {
                    stops: vec![
                        PaintColorStop {
                            offset: 0.0,
                            ..Default::default()
                        },
                        PaintColorStop {
                            offset: 1.0,
                            ..Default::default()
                        },
                    ],
                    ..PaintShader::default()
                })),
                can_render: true,
                ..BackgroundImageLayer::default()
            },
            ..PaintMaskLayer::default()
        };

        let resolved = ResolveMaskLayers(
            &fragment,
            fragment.resources.as_deref(),
            Offset::default(),
            Offset::default(),
            1.0,
            &[layer],
        );
        assert_eq!(resolved.len(), 1);
        assert!(resolved[0].paint_shader.is_some());
        assert_eq!(resolved[0].source_rect.width, resolved[0].tile_rect.width);
        assert_eq!(resolved[0].source_rect.height, resolved[0].tile_rect.height);
        assert!(resolved[0].source_rect.width > 0.0);
        assert!(resolved[0].source_rect.height > 0.0);
    }

    #[test]
    fn css_mask_clip_is_pixel_snapped_in_property_space() {
        let mut fragment = FragmentPaintData::default();
        fragment.size = Size {
            width: 160.0,
            height: 28.0,
        };
        let layer = PaintMaskLayer {
            image_state: PaintMaskImageState::kInvalid,
            image: BackgroundImageLayer {
                origin: BackgroundBox::kBorderBox,
                clip: BackgroundBox::kBorderBox,
                ..BackgroundImageLayer::default()
            },
            ..PaintMaskLayer::default()
        };

        let resolved = ResolveMaskLayers(
            &fragment,
            fragment.resources.as_deref(),
            Offset {
                x: 534.328125,
                y: 145.4375,
            },
            Offset::default(),
            1.0,
            &[layer],
        );

        assert_eq!(
            resolved[0].clip_rect,
            PaintRect {
                x: 534.0,
                y: 145.0,
                width: 160.0,
                height: 28.0,
            }
        );
    }

    #[test]
    fn css_mask_property_clip_encloses_fractional_border_box() {
        assert_eq!(
            EnclosingMaskClipRectInPropertySpace(
                PaintRect {
                    x: 534.328125,
                    y: 145.4375,
                    width: 160.0,
                    height: 28.0,
                },
                Offset::default(),
            ),
            PaintRect {
                x: 534.0,
                y: 145.0,
                width: 161.0,
                height: 29.0,
            }
        );
    }

    #[test]
    fn css_no_clip_mask_uses_source_maximum_region_without_clipping() {
        let mut fragment = FragmentPaintData::default();
        fragment.size = Size {
            width: 100.0,
            height: 80.0,
        };
        fragment.paint.css_mask_bounding_box = Some(FragmentPaintRect {
            offset: Offset { x: -7.0, y: -9.0 },
            size: Size {
                width: 140.0,
                height: 120.0,
            },
        });
        let layer = PaintMaskLayer {
            image_state: PaintMaskImageState::kInvalid,
            image: BackgroundImageLayer {
                origin: BackgroundBox::kContentBox,
                clip: BackgroundBox::kNoClip,
                ..BackgroundImageLayer::default()
            },
            ..PaintMaskLayer::default()
        };

        let resolved = ResolveMaskLayers(
            &fragment,
            fragment.resources.as_deref(),
            Offset { x: 20.0, y: 30.0 },
            Offset::default(),
            1.0,
            &[layer],
        );
        assert_eq!(resolved.len(), 1);
        assert!(!resolved[0].has_clip);
        assert_eq!(
            resolved[0].clip_rect,
            PaintRect {
                x: 13.0,
                y: 21.0,
                width: 140.0,
                height: 120.0,
            }
        );

        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        node.paint_offset = Offset { x: 20.0, y: 30.0 };
        node.mask_layers = resolved;
        assert_eq!(
            node.ResolvedMaskClipRect(),
            Some(PaintRect {
                x: 13.0,
                y: 21.0,
                width: 140.0,
                height: 120.0,
            })
        );
    }

    #[test]
    fn fieldset_css_mask_uses_the_legend_contracted_border_box() {
        let mut fragment = FragmentPaintData::default();
        fragment.size = Size {
            width: 100.0,
            height: 80.0,
        };
        fragment.paint.fieldset = Some(Arc::new(FieldsetPaintData {
            border_outsets: paint_input_types::layout_input::Edges {
                top: 7.0,
                right: 4.0,
                bottom: 3.0,
                left: 6.0,
            },
            ..FieldsetPaintData::default()
        }));
        fragment.paint.border.top = 4.0;
        fragment.paint.border.right = 3.0;
        fragment.paint.border.bottom = 5.0;
        fragment.paint.border.left = 2.0;
        fragment.paint.padding.top = 8.0;
        fragment.paint.padding.right = 7.0;
        fragment.paint.padding.bottom = 9.0;
        fragment.paint.padding.left = 6.0;
        fragment.content_size = Size {
            width: 72.0,
            height: 44.0,
        };
        let layer = PaintMaskLayer {
            image_state: PaintMaskImageState::kInvalid,
            image: BackgroundImageLayer {
                origin: BackgroundBox::kBorderBox,
                clip: BackgroundBox::kContentBox,
                ..BackgroundImageLayer::default()
            },
            ..PaintMaskLayer::default()
        };

        let resolved = ResolveMaskLayers(
            &fragment,
            fragment.resources.as_deref(),
            Offset { x: 20.0, y: 30.0 },
            Offset::default(),
            1.0,
            &[layer],
        );

        assert_eq!(resolved.len(), 1);
        assert_eq!(
            resolved[0].clip_rect,
            PaintRect {
                x: 34.0,
                y: 49.0,
                width: 72.0,
                height: 44.0,
            }
        );
    }
}
