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
    BackgroundInvalidation, BackgroundInvalidationType, BoxPaintInvalidator,
};
use crate::compositing_reason_finder::CompositingReasonFinder;
use crate::cull_rect_updater::CullRectUpdater;
use crate::nine_piece_geometry::NinePieceImageGeometry;
use crate::object_paint_invalidator::ObjectPaintInvalidator;
use crate::paint_invalidator::{PaintInvalidator, PaintInvalidatorContext};
use crate::table_cell_paint_invalidator::TableCellPaintInvalidator;
use paint_model::contoured_border_geometry::{
    ConicRoundedRectPath, ExpandCornerRadiiEdges, IncludedBorderEdges,
    OutsetCornerRadiiWithCorrection, ResolveCornerRadii, UniformCornerRadius,
};
use paint_model::geometry_mapper::{
    MapRectToRoot, MultiplyTransforms, ResolveLocalTransformAroundReferenceBox,
    ResolvePaintTransform, ResolveTransformReferenceRect, TranslationTransform,
};
use paint_model::paint_order::ClassifyFragmentPaintLayer;
use paint_model::paint_property_tree::{
    ClipPaintPropertyNode, EffectAnimationState, EffectPaintPropertyNode, PaintBackfaceVisibility,
    PaintCompositorStickyConstraint, PaintPropertyChangeType, PaintPropertyKey,
    PaintPropertyNodeStore, PaintPropertyOwner, PaintPropertyRole, PaintPropertyTrees,
    ScrollPaintPropertyNode, TransformAnimationState,
    TransformPaintPropertyNode,
};
use paint_model::{DisplayMaskLayer, ResolveDocumentImageContainerKey};

mod paint_property_tree_builder;

use paint_property_tree_builder::PaintPropertyTreeBuilder;

/// Read-only source for one PrePaint pass. Topology always comes from source
/// PhysicalFragment/FragmentItem links.
#[derive(Clone, Copy)]
pub struct PaintFragmentSource<'a>(PaintFragmentCursor<'a>);

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
    pub fn WithBlockingWheelEventHandlerTargets(mut self, targets: Arc<[u64]>) -> Self {
        self.blocking_wheel_targets = targets;
        self
    }
}

impl<'a> PaintFragmentSource<'a> {
    pub fn FromCursor(cursor: PaintFragmentCursor<'a>) -> Self {
        Self(cursor)
    }

    pub fn PaintData(self) -> &'a FragmentPaintData {
        self.0.PaintData()
    }

    pub fn Children(self) -> impl Iterator<Item = Self> + 'a {
        self.0.Children().into_iter().map(Self)
    }

    pub fn Cursor(self) -> PaintFragmentCursor<'a> {
        self.0
    }
}

impl<'a> std::ops::Deref for PaintFragmentSource<'a> {
    type Target = FragmentPaintData;

    fn deref(&self) -> &Self::Target {
        &self.0
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

/// `previous_client_id` is the DisplayItemClient id PrePaint recorded for the
/// previous fragment; the previous snapshot's LayoutObject may be gone.
fn RetainImagePaintedRect(
    current: &FragmentPaintData,
    previous: Option<&FragmentPaintData>,
    previous_client_id: Option<u64>,
) {
    if !current.paint.ReplacedIsImage() {
        return;
    }
    let previous = previous
        .filter(|_| previous_client_id == Some(current.paint.DisplayItemClientId()));
    if current.paint.image_last_paint_rect.get().is_none() {
        if let Some(retained) =
            previous.and_then(|previous| previous.paint.image_last_paint_rect.get())
        {
            current.paint.image_last_paint_rect.set(Some(retained));
        }
    }
    if current.paint.image_last_painted_id.get() == image_resource::PaintImageId::INVALID {
        if let Some(previous) = previous {
            current
                .paint
                .image_last_painted_id
                .set(previous.paint.image_last_painted_id.get());
        }
    }
}

#[inline]
fn PreviousFragment(
    pointer: Option<*const FragmentPaintData>,
) -> Option<&'static FragmentPaintData> {
    pointer.map(|pointer| {
        // The previous retained tree's PaintFragmentLease stays resident for
        // the duration of Walk. Keep a reference to that canonical sidecar;
        // copying FragmentPaintData here duplicated Layout/Paint input solely
        // to satisfy a temporary borrow in invalidation.
        unsafe { &*pointer }
    })
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
    /// Resident physical links remain stable while this retained tree is
    /// borrowed. Logical paint ownership can reparent OOF nodes, so clients
    /// validating Layout's physical revision paths must use this directory.
    pub fn PhysicalChildPointers(&self) -> Option<&[*mut PaintTreeNode<'static>]> {
        self.cache
            .as_ref()
            .map(|cache| cache.physical_children.as_slice())
    }

    /// PrePaint-owned (revision, subtree revision) of this resident node:
    /// the revision changes whenever PrePaint rebuilds, shifts or refreshes
    /// the node and is kept while it is reused; the subtree revision also
    /// changes with any descendant's. Values are process-unique per pass,
    /// unrelated to Layout's fragment revisions, so a consumer compares two
    /// recorded values of the same node (Blink's retained FragmentData /
    /// ObjectPaintProperties identity), never a recorded value with a
    /// source fragment's.
    pub fn RecordedRevisions(&self) -> Option<(u64, u64)> {
        self.cache
            .as_ref()
            .map(|cache| (cache.revision, cache.subtree_revision))
    }

    pub fn SourceCursor(&self) -> Option<PaintFragmentCursor<'_>> {
        self.source_cursor
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
        let fragment = self.fragment.as_deref()?;
        let is_svg_effect_child = IsSvgEffectChild(fragment.paint.SourceKind());
        if !is_svg_effect_child {
            let bounds = fragment.paint.CssMaskBoundingBox()?;
            return Some(PaintRect {
                x: self.paint_offset.x + bounds.offset.x,
                y: self.paint_offset.y + bounds.offset.y,
                width: bounds.size.width,
                height: bounds.size.height,
            });
        }
        let mut rect = self.mask_layers.first().map(|layer| layer.clip_rect);
        for layer in self.mask_layers.iter().skip(1) {
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
                    x: self.paint_offset.x + visual.offset.x,
                    y: self.paint_offset.y + visual.offset.y,
                    width: visual.size.width,
                    height: visual.size.height,
                };
                rect = Some(rect.map_or(visual, |rect| Union(rect, &visual)));
            }
        }
        rect
    }

    pub fn PixelSnappedRect(&self, rect: &PaintRect) -> PaintRect {
        PixelSnappedRectInPropertySpace(rect, self.paint_snap_offset, self.pixel_snap_scale)
    }

    pub fn EnclosingRect(&self, rect: PaintRect) -> PaintRect {
        // DrawingRecorder bounds are enclosed in the same local paint space
        // as its pixels. The delta accounts for a real paint-space reset.
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
struct RootFixedPositionContext {
    offset: Offset,
    paint_space: Offset,
    transforms: FlatTransforms,
    clip: Option<PaintRect>,
    properties: PropertyTreeState,
    scroll_translation: Option<Arc<TransformPaintPropertyNode>>,
}
struct NodeCache {
    inherited: InheritedContext,
    children: ChildContext,
    /// PrePaint-owned generations (PrePaintTreeWalk::revision_generation):
    /// `revision` changes whenever this node is rebuilt, shifted or
    /// refreshed for invalidation only, and is kept while it is reused;
    /// `subtree_revision` also changes when any descendant's does. Zero is
    /// a pending child walk (DisplayLockContext::SetNeedsPrePaintSubtreeWalk).
    revision: u64,
    subtree_revision: u64,
    /// The document paint resources this node was last proven against, and
    /// the resource usage (paint_resource_usage::PaintResourceUsage) of this
    /// node and of its whole subtree. Every node of the subtree paints the
    /// same under `resources` as under the value it was built with.
    resources: Option<Arc<PaintResources>>,
    resource_usage: u8,
    subtree_resource_usage: u8,
    /// The values this node compared, recorded after its comparisons; the
    /// next lifecycle compares the paired fragment against them.
    record: PrePaintPrevious,
    /// This node's fragment values as of its last visit: the paint payload
    /// (shared, not copied) and the fragment's own scalar fields. The paint
    /// invalidators read them as the previous fragment.
    previous_fragment: Option<FragmentPaintData>,
    /// The maximum change the in-place `Update` of this object's own
    /// property slots reported when it was last built (Chromium's
    /// PaintPropertyTreeBuilder `properties_changed_`). The node states
    /// above name the live slot nodes, so an old/new state comparison alone
    /// cannot see a value change.
    own_change: PaintPropertyChangeType,
    own_keys: Vec<(u64, PaintPropertyKey)>,
    subtree_keys: Vec<(u64, PaintPropertyKey)>,
    nodes: usize,
    computed_pass: u64,
    // Physical links survive logical OOF ownership moves. Boxes stay owned by
    // retained_tree; topology changes must use the ordinary ownership update.
    physical_children: Vec<*mut PaintTreeNode<'static>>,
    contains_local_frames: bool,
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
    // Completed geometry/cull state belongs to the resident paint lifecycle,
    // never to the published drawing artifact or a borrowed fragment tree.
    property_keys: BTreeMap<u64, PaintPropertyKey>,
    pub retained_tree: Option<Box<PaintTreeNode<'static>>>,
    retained_source_lease: Option<PaintFragmentLease>,
    pending_source_lease: Option<PaintFragmentLease>,
    source_owner_changed: bool,
    retained_tree_valid: bool,
    logical_tree_changed: bool,
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
    /// The ScopedDisplayItemFragment of the node UpdateNode is visiting
    /// (core/paint/box_fragment_painter.cc:394-403,
    /// core/paint/paint_layer_painter.cc:776-778). Set by the parent before
    /// each child visit; `None` outside a walk.
    display_item_fragment_scope: Option<u32>,
    /// The fragmentainer identity of the node UpdateNode is visiting
    /// (PrePaintTreeWalk's `fragmentainer_idx`; 0 outside any). Set by the
    /// parent before each child visit; `None` outside a walk.
    fragmentainer_scope: Option<u64>,
    completed_local_frame_prepaint: HashSet<u64>,
    // Source: core/paint/paint_property_tree_builder.cc:4692-4701.
    requires_main_thread_for_background_attachment_fixed: Option<bool>,
    // LayoutView::BackgroundRect changes mark every registered fixed
    // background object for full background invalidation in Chromium.
    view_background_geometry_changed: bool,
    /// The (slot id, key) of every property slot registered by the builds
    /// in progress (BuildNode drains its own range into NodeCache::own_keys),
    /// and the change each in-place slot update reported.
    own_key_log: Vec<(u64, PaintPropertyKey)>,
    own_change_log: Vec<PaintPropertyChangeType>,
    /// Decision D2: the logical out-of-flow effect parent is found in the
    /// builder context. Every visited paint-state occurrence of a
    /// LayoutObject (by node id) records the effect state its logical
    /// children inherit; the complete set is rebuilt from the final tree in
    /// every post-update pass (GatherFinalEffects).
    logical_effect_contexts: HashMap<u64, Vec<LogicalEffectContext>>,
    /// Node ids of the paint-state-establishing physical ancestors of the
    /// node UpdateNode is visiting (0 for one that establishes none).
    physical_ancestor_ids: Vec<u64>,
    needs_post_update: bool,
    /// Root-to-node transform products of property nodes seen this walk.
    /// The Arc keeps each key's allocation alive for the walk.
    transform_world_memo: HashMap<usize, (Arc<TransformPaintPropertyNode>, TransformMatrix)>,
    /// RECHROM_PREPAINT_SHIFT_CHECK: rebuild offset-only nodes anyway and
    /// compare them with the shifted retained node (diagnostic).
    shift_check: bool,
    pub stats: PrePaintUpdateStats,
    intersection_observation_state:
        foundation::intersection_observation_state::IntersectionObservationState,
    pub(crate) client_nodes: HashMap<u64, Vec<*mut PaintTreeNode<'static>>>,
    resident_nodes: Vec<*mut PaintTreeNode<'static>>,
    walk_generation: u64,
    /// This pass's PrePaint revision generation (NodeCache::revision): a
    /// process-unique nonzero value, so a node whose revision equals it
    /// changed in this pass.
    revision_generation: u64,
    /// SamePaintResourcesFor results of this pass by (recorded, current,
    /// usage); the Arcs keep both keys alive for the pass.
    resource_memo: Vec<(Arc<PaintResources>, Arc<PaintResources>, u8, bool)>,
    /// Walks that visited objects (and so cleared their paint flags), and
    /// that count when Paint last recorded an artifact.
    consuming_walks: u64,
    recorded_consuming_walks: u64,
}

impl Default for PrePaintTreeWalk {
    fn default() -> Self {
        Self {
            next_property_id: 1,
            property_node_store: Default::default(),
            visual_viewport_state: None,
            visual_viewport_scroll_offset: None,
            property_keys: BTreeMap::new(),
            retained_tree: None,
            retained_source_lease: None,
            pending_source_lease: None,
            source_owner_changed: false,
            retained_tree_valid: false,
            logical_tree_changed: false,
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
            own_key_log: Vec::new(),
            own_change_log: Vec::new(),
            logical_effect_contexts: HashMap::default(),
            physical_ancestor_ids: Vec::new(),
            needs_post_update: false,
            transform_world_memo: HashMap::default(),
            shift_check: std::env::var_os("RECHROM_PREPAINT_SHIFT_CHECK").is_some(),
            stats: PrePaintUpdateStats::default(),
            intersection_observation_state: Default::default(),
            client_nodes: HashMap::default(),
            resident_nodes: Vec::new(),
            walk_generation: 0,
            revision_generation: 0,
            resource_memo: Vec::new(),
            consuming_walks: 0,
            recorded_consuming_walks: 0,
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
        || fragment.ViewScrollContainer().is_some()
        || fragment.paint.StickyConstraint().is_some()
        || fragment.paint.StickyOffset().x != 0.0
        || fragment.paint.StickyOffset().y != 0.0
        || fragment.paint.NeedsAnchorPositionScrollAdjustment()
        || fragment.ViewScrollbars().is_some()
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
fn SnapToPixel(value: f64, scale: f64) -> f64 {
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

fn PixelSnappedRectInPropertySpace(rect: &PaintRect, delta: Offset, scale: f64) -> PaintRect {
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

// Rechrom boundary adaptation for Chromium's missable-child traversal
// (`pre_paint_tree_walk.cc:769-959`). Layout resolves the exact logical parent
// fragment occurrence while it still owns the physical fragment graph. Paint
// consumes only that stable identity; it must not invent a second geometric
// parent-selection path when the boundary leaves the relationship unsupported.
// This block is not claimed as a line-for-line translation in the coverage
// ledger.
fn GatherFragmentInstances<'a>(
    node: &mut PaintTreeNode<'a>,
    fragment_instances: &mut HashMap<u64, *mut PaintTreeNode<'a>>,
) {
    let node_pointer = node as *mut PaintTreeNode<'a>;
    let fragment = node
        .fragment
        .as_deref()
        .expect("PrePaint node must own a fragment");
    if fragment.ViewFragmentInstanceId() != 0 {
        if fragment_instances
            .insert(fragment.ViewFragmentInstanceId(), node_pointer)
            .is_some()
        {
            panic!("FragmentTree contains duplicate fragment_instance_id values");
        }
    }
    for child in &mut node.children {
        GatherFragmentInstances(child, fragment_instances);
    }
}

struct DetachedPaintNode<'a> {
    logical_parent: *mut PaintTreeNode<'a>,
    node: Box<PaintTreeNode<'a>>,
}

/// The occurrence of an out-of-flow child's logical parent LayoutObject:
/// the nearest physical ancestor occurrence of it that establishes paint
/// state, else its only such occurrence (preferring the child's
/// fragmentainer). Chromium's PrePaintTreeWalk reaches the containing
/// LayoutObject through its context stack; Rechrom's walk follows physical
/// fragments, so PrePaint resolves the occurrence here instead of the Layout
/// export resolving every slot each publication.
fn ResolveLogicalParentOccurrence<'a>(
    child: *const PaintTreeNode<'a>,
    logical_id: u64,
    physical_ancestors: &[*const PaintTreeNode<'a>],
    fragment_instances: &HashMap<u64, *mut PaintTreeNode<'a>>,
) -> Option<*mut PaintTreeNode<'a>> {
    let establishes = |node: &PaintTreeNode<'a>| {
        node.fragment.as_deref().is_some_and(|candidate| {
            candidate.ViewNodeId() == logical_id
                && candidate.paint.HasSource()
                && candidate.ViewEstablishesPaintState()
        })
    };
    for &ancestor in physical_ancestors.iter().rev() {
        let node = unsafe { &*ancestor };
        if node.fragment.as_deref().is_some_and(|candidate| {
            candidate.ViewNodeId() == logical_id && candidate.ViewEstablishesPaintState()
        }) {
            return Some(ancestor as *mut PaintTreeNode<'a>);
        }
    }
    // The enclosing fragmentainer is the walk's context of each node.
    let child_fragmentainer = unsafe { &*child }.FragmentainerInstanceId();
    let unique = |same_fragmentainer: bool| {
        let mut matches = fragment_instances.values().copied().filter(|&candidate| {
            let node = unsafe { &*candidate };
            candidate as *const PaintTreeNode<'a> != child
                && establishes(node)
                && (!same_fragmentainer || node.FragmentainerInstanceId() == child_fragmentainer)
        });
        let first = matches.next()?;
        matches.next().is_none().then_some(first)
    };
    unique(true).or_else(|| unique(false))
}

fn DetachLogicallyReparentedOutOfFlowNodes<'a>(
    parent: &mut PaintTreeNode<'a>,
    fragment_instances: &HashMap<u64, *mut PaintTreeNode<'a>>,
    physical_ancestors: &mut Vec<*const PaintTreeNode<'a>>,
    detached: &mut Vec<DetachedPaintNode<'a>>,
) {
    physical_ancestors.push(parent as *const PaintTreeNode<'a>);
    let mut index = 0;
    while index < parent.children.len() {
        let child = parent.children[index].as_mut();
        let child_pointer = child as *const PaintTreeNode<'a>;
        let fragment = child
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        let logical_id = fragment.paint.LogicalParentNodeId();
        let position = fragment.paint.Position();
        let is_out_of_flow = position == Position::kAbsolute || position == Position::kFixed;
        let logical_parent = if is_out_of_flow {
            logical_id.and_then(|logical_id| {
                ResolveLogicalParentOccurrence(
                    child_pointer,
                    logical_id,
                    physical_ancestors,
                    fragment_instances,
                )
            })
        } else {
            None
        };
        let logical_parent_is_physical_ancestor = logical_parent.is_some_and(|pointer| {
            physical_ancestors.contains(&(pointer as *const PaintTreeNode<'a>))
        });
        if is_out_of_flow
            && logical_id.is_some()
            && !logical_parent_is_physical_ancestor
            && logical_parent.is_some()
            && logical_parent != Some(child as *mut PaintTreeNode<'a>)
        {
            child.escapes_ancestor_geometry = true;
            child.physical_ancestors = physical_ancestors.clone();
            detached.push(DetachedPaintNode {
                logical_parent: logical_parent.unwrap(),
                node: parent.children.remove(index),
            });
            continue;
        }
        DetachLogicallyReparentedOutOfFlowNodes(
            child,
            fragment_instances,
            physical_ancestors,
            detached,
        );
        index += 1;
    }
    physical_ancestors.pop();
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

/// The number of effect nodes this object owns at the top of its border-box
/// effect chain (Effect/Mask, then Filter).
fn LocalEffectCount(node: &PaintTreeNode<'_>) -> usize {
    usize::from(
        node.applies_opacity
            || node.applies_blend
            || node.applies_backdrop_filter
            || node.applies_mask,
    ) + usize::from(node.applies_filter)
}

/// Decision D2 fix-up. The builder parents an out-of-flow object's effects
/// to its logical parent occurrence when that occurrence was visited before
/// it (PrePaintTreeWalk::LogicalOutOfFlowEffectContext). An occurrence the
/// physical walk reached only later is applied here, once: the outermost
/// own effect node is updated in place with the logical parent, inheriting
/// descendants take the logical parent's node, and the recorded builder
/// contexts follow so the next pass admits the subtree against them.
fn RebindLogicalEffectState(
    node: &mut PaintTreeNode<'_>,
    parent_opacity: f32,
    parent_effect_id: u64,
    parent_effect: Arc<EffectPaintPropertyNode>,
    store: &mut PaintPropertyNodeStore,
) {
    let fragment = node
        .fragment
        .as_deref()
        .expect("PrePaint node must own a fragment");
    let style: Option<&PaintStyle> = fragment.paint.HasSource().then_some(&fragment.paint.style);
    let local_opacity = if fragment.ViewEstablishesPaintState() {
        style.map_or(1.0, MaskAdjustedOpacity)
    } else {
        1.0
    };
    node.opacity = parent_opacity * local_opacity;
    let local_effect_count = LocalEffectCount(node);
    if local_effect_count == 0 {
        node.properties.effect_id = parent_effect_id;
        node.properties.nodes.effect = parent_effect.clone();
    } else {
        let (outermost, state) = paint_model::property_cell::with_access(|access| {
            let mut outermost = node.properties.nodes.effect.clone();
            for _ in 1..local_effect_count {
                outermost = outermost
                    .Parent(access)
                    .expect("local effect parent")
                    .clone();
            }
            let state = outermost.state(access);
            let reparent = !state
                .parent
                .as_ref()
                .is_some_and(|parent| Arc::ptr_eq(parent, &parent_effect));
            let state = reparent.then(|| {
                let mut state = state.clone();
                state.parent = Some(parent_effect.clone());
                state
            });
            (outermost, state)
        });
        if let Some(state) = state {
            paint_model::property_cell::with_writer(|writer| {
                store.UpdateEffectNode(writer, &outermost, state)
            });
        }
    }
    if let Some(fixed) = &mut node.fixed_background_properties {
        fixed.effect = node.properties.nodes.effect.clone();
    }
    // A mask effect's parent is this object's own content effect, whose
    // identity is unchanged. Scrollbar effects are parented to the border-box
    // effect, which is the logical parent's when the object owns none.
    for scrollbar in [
        &node.horizontal_scrollbar_properties,
        &node.vertical_scrollbar_properties,
    ] {
        let Some(scrollbar) = scrollbar else {
            continue;
        };
        let state = paint_model::property_cell::with_access(|access| {
            let state = scrollbar.effect.state(access);
            (!state
                .parent
                .as_ref()
                .is_some_and(|parent| Arc::ptr_eq(parent, &node.properties.nodes.effect)))
            .then(|| {
                let mut state = state.clone();
                state.parent = Some(node.properties.nodes.effect.clone());
                state
            })
        });
        if let Some(state) = state {
            paint_model::property_cell::with_writer(|writer| {
                store.UpdateEffectNode(writer, &scrollbar.effect, state)
            });
        }
    }
    node.contents_properties.effect_id = node.properties.effect_id;
    node.contents_properties.nodes.effect = node.properties.nodes.effect.clone();
    if let Some(cache) = node.cache.as_mut() {
        cache.inherited.opacity = parent_opacity;
        cache.inherited.properties.effect_id = parent_effect_id;
        cache.inherited.properties.nodes.effect = parent_effect.clone();
        cache.children.opacity = node.opacity;
        cache.children.properties.effect_id = node.properties.effect_id;
        cache.children.properties.nodes.effect = node.properties.nodes.effect.clone();
    }
    for child in &mut node.children {
        RebindLogicalEffectState(
            child,
            node.opacity,
            node.properties.effect_id,
            node.properties.nodes.effect.clone(),
            store,
        );
    }
}

fn FirstLogicalTreeOrder(node: &PaintTreeNode<'_>) -> u64 {
    let order = node
        .fragment
        .as_deref()
        .expect("PrePaint node must own a fragment")
        .paint
        .LogicalTreeOrder();
    if order != 0 {
        return order;
    }
    let mut first = u64::MAX;
    for child in &node.children {
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

fn InsertInLogicalTreeOrder<'a>(parent: &mut PaintTreeNode<'a>, node: Box<PaintTreeNode<'a>>) {
    let order = FirstLogicalTreeOrder(&node);
    if order == 0 {
        parent.children.push(node);
        return;
    }
    let insertion = parent.children.iter().position(|candidate| {
        let candidate_order = FirstLogicalTreeOrder(candidate);
        candidate_order != 0 && candidate_order > order
    });
    parent
        .children
        .insert(insertion.unwrap_or(parent.children.len()), node);
}

fn RestoreLogicalSiblingOrder(parent: &mut PaintTreeNode<'_>) -> bool {
    let mut changed = false;
    for child in &mut parent.children {
        changed |= RestoreLogicalSiblingOrder(child);
    }
    // Physical children usually already follow logical order. Preserve their
    // Box owners and anonymous slots without allocating/rebuilding three Vecs.
    let mut previous = 0;
    let already_ordered = parent.children.iter().all(|child| {
        let order = FirstLogicalTreeOrder(child);
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
        return changed;
    }
    let mut ordered_slots = Vec::new();
    let mut ordered_children = Vec::new();
    let mut slots: Vec<Option<Box<PaintTreeNode<'_>>>> = std::mem::take(&mut parent.children)
        .into_iter()
        .map(Some)
        .collect();
    for (index, slot) in slots.iter_mut().enumerate() {
        if FirstLogicalTreeOrder(slot.as_ref().unwrap()) == 0 {
            continue;
        }
        ordered_slots.push(index);
        ordered_children.push(slot.take().unwrap());
    }
    ordered_children.sort_by_key(|child| FirstLogicalTreeOrder(child));
    for (index, child) in ordered_slots.into_iter().zip(ordered_children) {
        slots[index] = Some(child);
    }
    parent.children = slots.into_iter().map(Option::unwrap).collect();
    true
}

/// Moves logically reparented out-of-flow subtrees under their logical
/// parent occurrence (paint order) and applies the D2 fix-up to those the
/// builder could not parent logically. Returns whether the tree differs
/// from the physical tree and the number of fix-ups.
fn RestoreLogicalOutOfFlowParents(
    root: &mut PaintTreeNode<'_>,
    store: &mut PaintPropertyNodeStore,
) -> (bool, usize) {
    let mut fragment_instances = HashMap::default();
    GatherFragmentInstances(root, &mut fragment_instances);
    let mut physical_ancestors = Vec::new();
    let mut detached = Vec::new();
    DetachLogicallyReparentedOutOfFlowNodes(
        root,
        &fragment_instances,
        &mut physical_ancestors,
        &mut detached,
    );
    let reparented = !detached.is_empty();
    let mut fixups = 0;
    for mut entry in detached {
        // Pointers target boxed nodes: moving a Box owner never moves its
        // allocation. All detached owners stay live through this loop.
        let logical_parent = unsafe { &mut *entry.logical_parent };
        // The builder context already named this logical parent's effect
        // state (the common case: the logical parent precedes the
        // out-of-flow object in the physical walk).
        let built_logically = entry.node.cache.as_ref().is_some_and(|cache| {
            Arc::ptr_eq(
                &cache.inherited.properties.nodes.effect,
                &logical_parent.properties.nodes.effect,
            ) && cache.inherited.opacity == logical_parent.opacity
        });
        if !built_logically {
            fixups += 1;
            RebindLogicalEffectState(
                &mut entry.node,
                logical_parent.opacity,
                logical_parent.properties.effect_id,
                logical_parent.properties.nodes.effect.clone(),
                store,
            );
        }
        InsertInLogicalTreeOrder(logical_parent, entry.node);
    }
    (RestoreLogicalSiblingOrder(root) || reparented, fixups)
}

type InlineOutlineKey = (u64, u64);

fn GatherInlineOutlineGroups<'a>(
    node: &mut PaintTreeNode<'a>,
    groups: &mut BTreeMap<InlineOutlineKey, Vec<*mut PaintTreeNode<'a>>>,
) {
    let fragment = node
        .fragment
        .as_deref()
        .expect("PrePaint node must own a fragment");
    if fragment.ViewNodeId() != 0
        && fragment.ViewKind() == FragmentKind::kBox
        && fragment.paint.HasSource()
        && fragment.ViewEstablishesPaintState()
        && fragment.paint.Display() == Display::kInline
        && fragment.paint.style.visible()
        && fragment.paint.style.outline_width() > 0.0
        && fragment.paint.style.outline_style() != BorderLineStyle::kNone
        && fragment.paint.style.outline_color().alpha > 0.0
    {
        groups
            .entry((fragment.ViewNodeId(), node.FragmentainerInstanceId()))
            .or_default()
            .push(node as *mut PaintTreeNode<'a>);
    }
    for child in &mut node.children {
        GatherInlineOutlineGroups(child, groups);
    }
}

fn PrepareSelfOutlineRects(node: &mut PaintTreeNode<'_>) {
    let fragment = node
        .fragment
        .as_deref()
        .expect("PrePaint node must own a fragment");
    if let Some(outline) = fragment.ViewSelfOutline() {
        node.outline_rects
            .extend(outline.rects.iter().map(|rect| PaintRect {
                x: node.paint_offset.x + rect.offset.x,
                y: node.paint_offset.y + rect.offset.y,
                width: rect.size.width,
                height: rect.size.height,
            }));
    }
    for child in &mut node.children {
        PrepareSelfOutlineRects(child);
    }
}

/// Chromium's stable AutoDarkModeSVGSizeThreshold feature scopes the graphics
/// context while an SVG root paints its children. The derived flag lives on
/// each immutable PaintStyle snapshot so reparented PaintLayer descendants and
/// ordinary retained children observe the same state without depending on walk
/// call-stack shape. Image auto-dark admission intentionally ignores this flag.
fn PropagateAutoDarkModeState(node: &mut PaintTreeNode<'_>, inherited_paused: bool) {
    const MAX_IMAGE_LENGTH: f64 = 64.0;
    let child_paused = {
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        if fragment.paint.SourceKind() == NodeKind::kSvgRoot && fragment.paint.style.force_dark() {
            let snapped = node.PixelSnappedRect(&PaintRect {
                x: node.paint_offset.x,
                y: node.paint_offset.y,
                width: fragment.ViewSize().width,
                height: fragment.ViewSize().height,
            });
            let layout_zoom = node
                .DocumentResources()
                .map_or(1.0, |resources| resources.layout_zoom_factor)
                .max(f64::MIN_POSITIVE);
            snapped.width / layout_zoom > MAX_IMAGE_LENGTH
                || snapped.height / layout_zoom > MAX_IMAGE_LENGTH
        } else {
            inherited_paused
        }
    };
    {
        // The style payloads are shared copy-on-write owners: assign only on
        // change, or every walk deep-copies each node's PaintProperties and
        // PaintStyleData to store the same value.
        let fragment = node
            .fragment
            .as_deref_mut()
            .expect("PrePaint node must own a fragment");
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
    }
    for child in &mut node.children {
        PropagateAutoDarkModeState(child, child_paused);
    }
}

fn PrepareOutlines(root: &mut PaintTreeNode<'_>) {
    PrepareSelfOutlineRects(root);
    let mut groups = BTreeMap::new();
    GatherInlineOutlineGroups(root, &mut groups);
    for (_unused_key, occurrences) in groups {
        let owner_pointer = occurrences[0];
        let mut outline_rects = Vec::new();
        for occurrence_pointer in occurrences {
            // The pointers refer to distinct boxed nodes that remain owned by
            // the tree for this whole pass.
            outline_rects.append(unsafe { &mut (*occurrence_pointer).outline_rects });
            if occurrence_pointer != owner_pointer {
                unsafe { (*occurrence_pointer).paints_outline = false };
            }
        }
        unsafe { (*owner_pointer).outline_rects = outline_rects };
    }
}

fn MarkPaintPhaseNeeds(node: &mut PaintTreeNode<'_>) {
    let fragment = node
        .fragment
        .as_deref()
        .expect("PrePaint node must own a fragment");
    // cpp: core/paint/paint_invalidator.cc:33-53
    // cpp: core/paint/paint_layer_painter.cc:800-809
    // Empty phases need neither a subtree walk nor replay geometry scopes.
    // Aggregate after logical reparenting, preserving PaintInvalidator's
    // current-painting-layer boundary: a self-painting child owns its phase
    // flags and must not make the ancestor layer visit empty phases.
    node.needs_float_phase =
        fragment.paint.HasSource() && fragment.paint.Floating() != FloatSide::kNone;
    let style = &fragment.paint.style;
    node.needs_outline_phase = !node.is_paint_layer
        && (fragment.paint.FocusedImageMapArea().is_some()
            || fragment
                .paint
                .FormControl()
                .is_some_and(|control| control.auto_focus_ring)
            || (style.outline_style() != BorderLineStyle::kNone
                && !(style.outline_width() <= 0.0)
                && !(style.outline_color().alpha <= 0.0)));
    node.needs_overflow_controls_phase = fragment
        .ViewScrollbars()
        .is_some_and(|scrollbars| scrollbars.should_paint_as_overlay && node.is_paint_layer);
    for child in &mut node.children {
        MarkPaintPhaseNeeds(child);
        if !child.is_paint_layer {
            node.needs_float_phase |= child.needs_float_phase;
            node.needs_outline_phase |= child.needs_outline_phase;
            node.needs_overflow_controls_phase |= child.needs_overflow_controls_phase;
        }
    }
}

/// The SVG resource directory and, for decision D2, the effect state every
/// paint-state occurrence gives its logical children, from the final tree.
fn GatherFinalEffects(
    node: &PaintTreeNode<'_>,
    logical_effects: &mut HashMap<u64, Vec<LogicalEffectContext>>,
    resources: &mut HashMap<u64, *const PaintTreeNode<'static>>,
) {
    let fragment = node
        .fragment
        .as_deref()
        .expect("PrePaint node must own a fragment");
    if fragment.paint.IsSVGHiddenContainer() {
        resources.insert(
            fragment.paint.DisplayItemClientId(),
            node as *const PaintTreeNode<'_> as *const PaintTreeNode<'static>,
        );
    }
    if let Some(context) = LogicalEffectContextOf(node, node.FragmentainerInstanceId()) {
        logical_effects
            .entry(fragment.ViewNodeId())
            .or_default()
            .push(context);
    }
    for child in &node.children {
        GatherFinalEffects(child, logical_effects, resources);
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

/// Clears the logical-tree escape marks DetachLogicallyReparentedOutOfFlowNodes
/// set in the preceding post-update pass.
fn ResetLogicalEscapes(node: &mut PaintTreeNode<'_>) {
    node.escapes_ancestor_geometry = false;
    node.physical_ancestors.clear();
    for child in &mut node.children {
        ResetLogicalEscapes(child);
    }
}


/// Undo ownership moves before an incremental physical-tree update. All Box
/// allocations remain live until every escaped owner has been put back.
fn RestorePhysicalOwnership(root: &mut PaintTreeNode<'static>) {
    fn detach(
        node: &mut PaintTreeNode<'static>,
        detached: &mut Vec<(*const PaintTreeNode<'static>, Box<PaintTreeNode<'static>>)>,
    ) {
        let mut index = 0;
        while index < node.children.len() {
            detach(&mut node.children[index], detached);
            if node.children[index].escapes_ancestor_geometry {
                let mut child = node.children.remove(index);
                let parent = *child
                    .physical_ancestors
                    .last()
                    .expect("escaped node retains its physical parent");
                child.escapes_ancestor_geometry = false;
                child.physical_ancestors.clear();
                detached.push((parent, child));
            } else {
                index += 1;
            }
        }
    }
    fn order(node: &mut PaintTreeNode<'static>) {
        node.children.sort_by_key(|child| child.physical_index);
        for child in &mut node.children {
            order(child);
        }
    }
    let mut detached = Vec::new();
    detach(root, &mut detached);
    for (parent, child) in detached {
        // Physical parents point into the still-owned boxed tree or into a
        // detached Box kept alive by this loop. Moving a Box does not move it.
        unsafe { &mut *(parent as *mut PaintTreeNode<'static>) }
            .children
            .push(child);
    }
    order(root);
}

fn ResetPostPaintData(node: &mut PaintTreeNode<'_>) {
    // Full PrePaint postwalk includes topology/stacking-style changes. Impl
    // scroll-only property refresh does not clear these source ordering lists.
    *node.hit_test_layer_order.get_mut() = None;
    node.outline_rects.clear();
    node.paints_outline = true;
    for child in &mut node.children {
        ResetPostPaintData(child);
    }
}

/// Propagates the invalidation results produced by this PrePaint pass to the
/// PaintLayer subsequence admission bit consumed by PaintLayerPainter.
///
/// Chromium mutates PaintLayer::needs_repaint_ while ObjectPaintInvalidator
/// walks LayoutObjects, then PaintLayer::SelfOrDescendantNeedsRepaint() folds
/// that state through the layer tree before cached-subsequence lookup.  The
/// immutable Rechrom boundary snapshots the Layout-owned bit before PrePaint
/// computes fragment-client invalidation, so a newly attached LayoutText could
/// otherwise remain hidden behind its ancestor's cached subsequence.  Fold the
/// resident client results back into the same per-layer admission fact after
/// the complete child walk.
// cpp: core/paint/paint_invalidator.cc:230-269
// cpp: core/paint/paint_layer.h:451-457
fn PropagateSubsequenceRepaint(node: &mut PaintTreeNode<'_>) -> bool {
    PropagateSubsequenceRepaintCounting(node, &mut [0; 3])
}

/// `counts`: [paint layers, layers already flagged by the source PaintLayer,
/// layers newly flagged from invalidated clients].
fn PropagateSubsequenceRepaintCounting(
    node: &mut PaintTreeNode<'_>,
    counts: &mut [usize; 3],
) -> bool {
    if node.is_paint_layer {
        // PaintLayerPainter reads PaintLayer::SelfOrDescendantNeedsRepaint()
        // at paint time (paint_layer_painter.cc:412-424); the bit is cleared
        // after each committed paint. A retained PrePaint node must not keep
        // the value captured when it was last built, or a layer flagged once
        // (e.g. at first paint) never reuses its subsequence again.
        if let Some(fragment) = node.fragment.as_deref() {
            node.self_or_descendant_needs_repaint =
                fragment.paint.PaintLayerSelfOrDescendantNeedsRepaint();
        }
        counts[0] += 1;
        counts[1] += node.self_or_descendant_needs_repaint as usize;
    }
    let client_needs_repaint = node.fragment.as_deref().is_some_and(|fragment| {
        [
            fragment.paint.DisplayItemClientState().invalidation_reason,
            fragment.paint.SelectionClientState().invalidation_reason,
            fragment.paint.PaintLayerClientState().invalidation_reason,
        ]
        .into_iter()
        .flatten()
        // A client stays kJustCreated until it first produces a display item
        // (paint_controller.cc:127-143 validates only used clients). Chromium
        // never consults that state for subsequence admission: the object's
        // first PrePaint invalidation (kAppeared on insertion) marked its
        // painting layer in InvalidatePaintWithComputedReason. Counting it
        // here would keep every ancestor layer of a non-painting object out
        // of subsequence reuse on every frame.
        .any(|reason| {
            reason != PaintInvalidationReason::kNone
                && reason != PaintInvalidationReason::kJustCreated
        })
    });
    // Every descendant PaintLayer needs its own admission bit updated. `any`
    // would stop after the first dirty sibling and leave later layers eligible
    // to reuse stale subsequences even though their clients were invalidated.
    let mut descendant_needs_repaint = false;
    for child in &mut node.children {
        descendant_needs_repaint |= PropagateSubsequenceRepaintCounting(child, counts);
    }
    let subtree_needs_repaint = client_needs_repaint || descendant_needs_repaint;
    if node.is_paint_layer && subtree_needs_repaint {
        counts[2] += !node.self_or_descendant_needs_repaint as usize;
        node.self_or_descendant_needs_repaint = true;
    }
    subtree_needs_repaint
}

fn SameSourceIdentity(a: &FragmentPaintData, b: &FragmentPaintData) -> bool {
    a.ViewNodeId() == b.ViewNodeId()
        && a.ViewFragmentInstanceId() == b.ViewFragmentInstanceId()
        && a.ViewKind() == b.ViewKind()
        && a.paint.DisplayItemClientId() == b.paint.DisplayItemClientId()
        && a.ViewDisplayItemFragment() == b.ViewDisplayItemFragment()
}

// Chromium's display-lock path retains children because their LayoutObject and
// FragmentData owners remain resident. Rechrom may do that only while the same
// Paint sidecar lease remains current.
fn CanRetainDisplayLockedChildren(
    child_pre_paint_blocked_by_display_lock: bool,
    source_owner_changed: bool,
) -> bool {
    child_pre_paint_blocked_by_display_lock && !source_owner_changed
}

fn RequiresFullSourceWalk(source_owner_changed: bool, retained_tree_valid: bool) -> bool {
    // Chromium retains LayoutObject/FragmentData owners across PrePaint. A
    // replacement LayoutResult in Rechrom owns a different sidecar allocation,
    // so no retained raw FragmentPaintData pointer may cross this boundary.
    // Rebuild while the preceding lease is still alive. UpdateNode then visits
    // old/new nodes in lockstep once: O(n), without a recursive compatibility
    // proof at every descendant.
    // Retained-node admission reads only the live dirty bits and PrePaint's
    // own records (UpdateNode), so no Layout certification gates it.
    source_owner_changed || !retained_tree_valid
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

fn SamePhaseStructure(a: &FragmentPaintData, b: &FragmentPaintData) -> bool {
    let phase = |f: &FragmentPaintData| {
        (
            f.paint.HasSource(),
            f.paint.Floating(),
            f.paint
                .FormControl()
                .is_some_and(|control| control.auto_focus_ring),
            f.paint.FocusedImageMapArea().is_some(),
            f.ViewScrollbars().is_some(),
            f.paint.Display(),
            f.ViewEstablishesPaintState(),
            f.paint.IsSVGHiddenContainer(),
            f.paint.style.visible(),
            f.paint.style.outline_style(),
            f.paint.style.outline_width(),
            f.paint.style.outline_color().alpha,
        )
    };
    phase(a) == phase(b)
}

fn SamePhaseInputs(a: &FragmentPaintData, b: &FragmentPaintData) -> bool {
    SamePhaseStructure(a, b) && a.ViewSize() == b.ViewSize()
}

/// Values of one fragment that this walk compares across lifecycles. Each
/// retained node records them after its own comparisons, so the next
/// lifecycle compares against the node's record instead of dereferencing the
/// previous LayoutResult's sidecar copy (Chromium likewise compares against
/// previous values kept on its persistent owners). Enums are stored as their
/// `repr(i32)` discriminants; `kind` is `None` until recorded.
/// DisplayItemClient ids of one fragment (see ClientDirectoryIds).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ClientIds {
    display_item: u64,
    paint_layer: u64,
    /// Scroll corner, horizontal and vertical scrollbar clients.
    scrollbars: Option<(u64, Option<u64>, Option<u64>)>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct PrePaintPrevious {
    kind: Option<FragmentKind>,
    node_id: u64,
    fragment_instance_id: u64,
    display_item_client_id: u64,
    display_item_fragment: u32,
    size: foundation::LayoutSize,
    offset: foundation::LayoutOffset,
    content_size: foundation::LayoutSize,
    /// Boolean inputs of SamePhaseStructure, as bits.
    phase_flags: u32,
    floating: i32,
    display: i32,
    outline_style: i32,
    outline_width: f64,
    outline_alpha: f32,
    position: i32,
    logical_parent_node_id: Option<u64>,
    logical_tree_order: u64,
    /// DisplayItemClient ids compared by SameClientDirectory.
    clients: ClientIds,
    /// The selection and scrolling-background DisplayItemClients, which
    /// SameClientDirectory does not compare but whose recorded ids prove
    /// them not newly created (HasUnrecordedJustCreatedClient).
    selection_client_id: u64,
    scrolling_background_client_id: u64,
}

fn PrePaintPreviousOf(f: &FragmentPaintData, link_offset: Offset) -> PrePaintPrevious {
    let p = &f.paint;
    let (visible, outline_style, outline_width, outline_alpha) = p.style.PhaseStyleValues();
    let phase_flags = u32::from(p.HasSource())
        | u32::from(
            p.FormControl()
                .is_some_and(|control| control.auto_focus_ring),
        ) << 1
        | u32::from(p.FocusedImageMapArea().is_some()) << 2
        | u32::from(p.scrollbars.is_some()) << 3
        | u32::from(f.ViewEstablishesPaintState()) << 4
        | u32::from(p.IsSVGHiddenContainer()) << 5
        | u32::from(visible) << 6;
    PrePaintPrevious {
        kind: Some(f.ViewKind()),
        node_id: f.ViewNodeId(),
        fragment_instance_id: f.ViewFragmentInstanceId(),
        display_item_client_id: p.DisplayItemClientId(),
        display_item_fragment: f.ViewDisplayItemFragment(),
        size: f.ViewSize(),
        offset: link_offset,
        content_size: f.ViewContentSize(),
        phase_flags,
        floating: p.Floating() as i32,
        display: p.Display() as i32,
        outline_style: outline_style as i32,
        outline_width,
        outline_alpha,
        position: p.Position() as i32,
        logical_parent_node_id: p.LogicalParentNodeId(),
        logical_tree_order: p.LogicalTreeOrder(),
        clients: ClientDirectoryIds(f),
        selection_client_id: p.SelectionClientId(),
        scrolling_background_client_id: f
            .ViewScrollContainer()
            .map_or(0, |scroll| scroll.scrolling_background_client_id),
    }
}

/// The previous-fragment view a retained node keeps for the paint
/// invalidators: the shared paint payload (an Arc, not a copy) and the
/// fragment's scalar fields. CSSOM inline rectangles are not paint inputs.
fn PreviousFragmentSnapshot(f: &FragmentPaintData) -> FragmentPaintData {
    let mut snapshot = FragmentPaintData {
        // Document resources are read from the document, never the slot.
        resources: None,
        paint: f.paint.clone(),
        ..*f
    };
    // Gap rules are read live from the resident box; keep the value this
    // fragment painted with as the comparison baseline
    // (ShouldInvalidateGapDecorations).
    let gap_rules = f.ViewGapRules();
    if snapshot.paint.gap_rules != gap_rules {
        snapshot.paint.gap_rules = gap_rules;
    }
    snapshot
}

/// The record of a retained node; empty if it has none.
/// Whether a clean retained subtree pairs one-to-one, by source identity and
/// child order, with the replacement source subtree. Read-only.
fn CleanSubtreeRebindable(node: &PaintTreeNode<'static>, source: PaintFragmentSource<'_>) -> bool {
    if node.fragment.is_none() || !SamePreviousSourceIdentity(node, &source) {
        return false;
    }
    let mut sources = OrdinaryPaintSourceChildren(source);
    for child in &node.children {
        match sources.next() {
            Some(child_source) if CleanSubtreeRebindable(child, child_source) => {}
            _ => return false,
        }
    }
    sources.next().is_none()
}

/// Move every node of a clean subtree, proven by CleanSubtreeRebindable, to
/// the replacement owner: its sidecar slot and its source cursor are the only
/// references into a LayoutResult. Geometry, properties and caches are kept,
/// as Chromium keeps a clean FragmentData across relayout. Returns the nodes.
fn RebindCleanSubtree(node: &mut PaintTreeNode<'static>, source: PaintFragmentSource<'_>) -> usize {
    node.fragment = Some(ResidentFragment::FromSource(&source));
    node.source_cursor = Some(unsafe {
        std::mem::transmute::<PaintFragmentCursor<'_>, PaintFragmentCursor<'static>>(
            source.Cursor(),
        )
    });
    let mut nodes = 1;
    for (child, child_source) in node.children.iter_mut().zip(OrdinaryPaintSourceChildren(source)) {
        nodes += RebindCleanSubtree(child, child_source);
    }
    nodes
}

/// A retained subtree the walk did not descend into keeps the
/// ScopedDisplayItemFragment values recorded at its last visit. When its
/// root's scope changed, re-derive the inherited scopes below it through the
/// physical links: a fragment opening its own scope keeps it and an embedded
/// frame root starts at 0 (box_fragment_painter.cc:394-403,
/// paint_layer_painter.cc:776-778).
fn RefreshInheritedDisplayItemFragments(node: &mut PaintTreeNode<'static>) {
    let scope = node.DisplayItemFragment();
    let fragmentainer = node.FragmentainerInstanceId();
    let Some(children) = node.cache.as_ref().map(|cache| cache.physical_children.clone()) else {
        return;
    };
    for child in children {
        // Physical links stay valid while the retained tree owns every box.
        let child = unsafe { &mut *child };
        let value = child
            .fragment
            .as_deref()
            .and_then(|fragment| fragment.ViewOwnDisplayItemFragment())
            .unwrap_or(if child.is_embedded_local_frame_root { 0 } else { scope });
        // The enclosing fragmentainer is inherited the same way.
        let child_fragmentainer = child
            .fragment
            .as_deref()
            .and_then(|fragment| fragment.ViewOwnFragmentainerInstanceId())
            .unwrap_or(if child.is_embedded_local_frame_root { 0 } else { fragmentainer });
        if child.display_item_fragment != Some(value)
            || child.fragmentainer_instance_id != Some(child_fragmentainer)
        {
            child.display_item_fragment = Some(value);
            child.fragmentainer_instance_id = Some(child_fragmentainer);
            RefreshInheritedDisplayItemFragments(child);
        }
    }
}

/// A node without own property nodes takes its clip, cull rects and flat
/// transforms, and passes them to its children, unchanged from its parent.
/// Only then is an offset-only update a pure translation.
fn ShiftPreservesClips(node: &PaintTreeNode<'_>) -> bool {
    let Some(cache) = node.cache.as_ref() else {
        return false;
    };
    let inherited = &cache.inherited;
    // Paint layers are admitted: RefreshLivePaintLayerFacts re-reads every
    // live layer fact the builder reads. SVG children are admitted: their own
    // local matrices are in SVG user space and keep their values when the
    // container moves (ShiftKeepsSvgPrefix checks the container's matrix).
    // An SVG root still rebuilds: its viewBox matrix maps to its paint offset.
    cache.children.svg.is_none()
        && node.fixed_background_properties.is_none()
        && node.overflow_controls_properties.is_none()
        && node.horizontal_scrollbar_properties.is_none()
        && node.vertical_scrollbar_properties.is_none()
        && node.mask_properties.is_none()
        && node.clip == inherited.clip
        && node.cull_rect == inherited.cull
        && node.self_cull_rect == inherited.cull
        && node.transforms.starts_with(&inherited.transforms)
        && node.transforms.len() - inherited.transforms.len() <= node.local_transforms.len()
        && cache.children.clip == inherited.clip
        && cache.children.cull == inherited.cull
        && (Arc::ptr_eq(&cache.children.transforms, &node.transforms)
            || cache.children.transforms == node.transforms)
}

/// A node with local transforms may lead with its SVG container's matrix,
/// which the parent inserts only into newly built children. Shifting keeps
/// that leading matrix, so the container's matrix must be unchanged.
fn ShiftKeepsSvgPrefix(node: &PaintTreeNode<'_>, old: &InheritedContext, new: &InheritedContext) -> bool {
    node.local_transforms.is_empty() || old.transforms.last() == new.transforms.last()
}

/// The flat transform list of a shifted node: the parent's new list followed
/// by the node's own matrices, which are independent of its position.
fn ShiftedTransforms(old_node: &FlatTransforms, old_inherited_len: usize, inherited: &FlatTransforms) -> FlatTransforms {
    let own = &old_node[old_inherited_len..];
    if own.is_empty() {
        inherited.clone()
    } else {
        let mut list = Vec::with_capacity(inherited.len() + own.len());
        list.extend_from_slice(inherited);
        list.extend_from_slice(own);
        FlatTransforms::from(list)
    }
}

/// The object's only own property node is a plain Effect (opacity/blend, no
/// filter, backdrop or mask) that BuildNode derived from the inherited
/// transform, clip and effect alone, it is linked to the resident inherited
/// nodes the object moves under, and the parent facts the builder copies
/// into it are unchanged. Such an effect keeps its value and its links when
/// the object moves (Chromium updates only the moved ancestor's slots in
/// place), so the moved object keeps its own property state.
fn RebasableOwnEffect(node: &PaintTreeNode<'_>, inherited: &InheritedContext) -> bool {
    let Some(cache) = node.cache.as_ref() else {
        return false;
    };
    let [(_, key)] = cache.own_keys.as_slice() else {
        return false;
    };
    if key.role != PaintPropertyRole::Effect {
        return false;
    }
    let old = &cache.inherited.properties;
    let new = &inherited.properties;
    let own = &node.properties;
    let effect_node = &own.nodes.effect;
    let contents = &node.contents_properties.nodes;
    let children = &cache.children.properties.nodes;
    paint_model::property_cell::with_access(|access| {
        let effect = effect_node.state(access);
        let new_parent = new.nodes.effect.state(access);
        let old_parent = old.nodes.effect.state(access);
        own.effect_id == effect_node.id
            && own.effect_id != old.effect_id
            && Arc::ptr_eq(&own.nodes.transform, &new.nodes.transform)
            && Arc::ptr_eq(&own.nodes.clip, &new.nodes.clip)
            && effect
                .parent
                .as_ref()
                .is_some_and(|parent| Arc::ptr_eq(parent, &new.nodes.effect))
            && Arc::ptr_eq(&effect.local_transform_space, &new.nodes.transform)
            && effect
                .output_clip
                .as_ref()
                .is_some_and(|clip| Arc::ptr_eq(clip, &new.nodes.clip))
            && effect.filters.is_empty()
            && effect.backdrop_filters.is_empty()
            && effect.backdrop_filter_clip.is_none()
            && effect.backdrop_filter_bounds.is_none()
            && effect.filter_reference_box.is_none()
            && !effect.has_mask
            && !effect.is_mask
            && Arc::ptr_eq(&contents.effect, effect_node)
            && Arc::ptr_eq(&children.effect, effect_node)
            && Arc::ptr_eq(&contents.clip, &own.nodes.clip)
            && Arc::ptr_eq(&contents.transform, &own.nodes.transform)
            && Arc::ptr_eq(&children.clip, &own.nodes.clip)
            && Arc::ptr_eq(&children.transform, &own.nodes.transform)
            && new_parent.compositor.is_in_tainted_subtree
                == old_parent.compositor.is_in_tainted_subtree
            && new_parent.compositor.is_in_drawable_canvas_subtree
                == old_parent.compositor.is_in_drawable_canvas_subtree
            && new_parent.self_or_ancestor_participates_in_view_transition
                == old_parent.self_or_ancestor_participates_in_view_transition
    })
}

/// Own property nodes of a moved object under its new inherited state.
/// Chromium updates only the moved ancestor's ObjectPaintProperties in
/// place, so the object's own resident nodes keep their identity, value and
/// links. This proves that case: every link of an own node reaches either
/// another own node or the inherited top node, which must be the same
/// resident node in the old and new inherited states. Any other link
/// rejects the shift and the object is rebuilt (updating its slots).
struct OwnNodeRebase<'a> {
    access: &'a paint_model::property_cell::PropertyAccess,
    own_ids: &'a [(u64, PaintPropertyKey)],
    inherited: &'a paint_model::paint_property_tree::PropertyTreeState,
    inherited_scroll: Option<Arc<ScrollPaintPropertyNode>>,
    /// Own nodes already proven, by allocation.
    proven: Vec<usize>,
}

impl OwnNodeRebase<'_> {
    fn is_own(&self, id: u64) -> bool {
        self.own_ids.iter().any(|(own, _)| *own == id)
    }

    fn proven<T>(&mut self, node: &Arc<T>, prove: impl FnOnce(&mut Self) -> bool) -> bool {
        let key = Arc::as_ptr(node) as *const () as usize;
        if self.proven.contains(&key) {
            return true;
        }
        let proven = prove(self);
        if proven {
            self.proven.push(key);
        }
        proven
    }

    fn transform(&mut self, node: &Arc<TransformPaintPropertyNode>) -> bool {
        if Arc::ptr_eq(node, &self.inherited.transform) {
            return true;
        }
        if !self.is_own(node.id) {
            return false;
        }
        let links = node.state(self.access).links();
        self.proven(node, |this| {
            links.parent.as_ref().is_none_or(|parent| this.transform(parent))
                && links.scroll.as_ref().is_none_or(|scroll| this.scroll(scroll))
                && links
                    .scroll_parent_scroll_translation
                    .as_ref()
                    .is_none_or(|translation| this.transform(translation))
        })
    }

    fn clip(&mut self, node: &Arc<ClipPaintPropertyNode>) -> bool {
        if Arc::ptr_eq(node, &self.inherited.clip) {
            return true;
        }
        if !self.is_own(node.id) {
            return false;
        }
        let state = node.state(self.access);
        let (parent, transform, filter) = (
            state.parent.clone(),
            state.local_transform_space.clone(),
            state.pixel_moving_filter.clone(),
        );
        self.proven(node, |this| {
            parent.as_ref().is_none_or(|parent| this.clip(parent))
                && this.transform(&transform)
                && filter.as_ref().is_none_or(|filter| this.effect(filter))
        })
    }

    fn effect(&mut self, node: &Arc<EffectPaintPropertyNode>) -> bool {
        if Arc::ptr_eq(node, &self.inherited.effect) {
            return true;
        }
        if !self.is_own(node.id) {
            return false;
        }
        let links = node.state(self.access).links();
        if links.canvas_child_state.is_some() {
            return false;
        }
        self.proven(node, |this| {
            links.parent.as_ref().is_none_or(|parent| this.effect(parent))
                && this.transform(&links.local_transform_space)
                && links.output_clip.as_ref().is_none_or(|clip| this.clip(clip))
                && links
                    .backdrop_filter_clip
                    .as_ref()
                    .is_none_or(|clip| this.clip(clip))
        })
    }

    fn scroll(&mut self, node: &Arc<ScrollPaintPropertyNode>) -> bool {
        if self
            .inherited_scroll
            .as_ref()
            .is_some_and(|inherited| Arc::ptr_eq(node, inherited))
        {
            return true;
        }
        if !self.is_own(node.id) {
            return false;
        }
        let state = node.state(self.access);
        let (parent, overflow_clip) = (state.parent.clone(), state.overflow_clip.clone());
        self.proven(node, |this| {
            parent.as_ref().is_none_or(|parent| this.scroll(parent))
                && overflow_clip.as_ref().is_none_or(|clip| this.clip(clip))
        })
    }

    fn state(&mut self, state: &PropertyTreeState) -> bool {
        self.transform(&state.nodes.transform)
            && self.clip(&state.nodes.clip)
            && self.effect(&state.nodes.effect)
    }
}

/// The object's own states under the new inherited state (unchanged nodes).
struct RebasedOwnStates {
    properties: PropertyTreeState,
    contents: PropertyTreeState,
    children: PropertyTreeState,
    pre_transform: Option<Arc<TransformPaintPropertyNode>>,
}

/// Offset-only move of an object that owns property nodes, under an
/// ancestor whose property space moved with it (its paint offset inside
/// that space is unchanged, so its own node values are unchanged): its flat
/// clip moves rigidly with the inherited clip, its own resident nodes keep
/// their links (OwnNodeRebase), and its cull rects are recomputed as the
/// builder computes them.
fn RebaseOwnStates(
    node: &PaintTreeNode<'_>,
    inherited: &InheritedContext,
    delta: ShiftDelta,
) -> Option<RebasedOwnStates> {
    let cache = node.cache.as_ref()?;
    let old = &cache.inherited;
    let (old_nodes, new_nodes) = (&old.properties.nodes, &inherited.properties.nodes);
    if delta.offset != delta.paint_space
        || cache.own_keys.is_empty()
        || node.fixed_background_properties.is_some()
        || node.overflow_controls_properties.is_some()
        || node.horizontal_scrollbar_properties.is_some()
        || node.vertical_scrollbar_properties.is_some()
        || node.mask_properties.is_some()
        || inherited.clip != MovedRect(old.clip, delta.offset)
        || !node.transforms.starts_with(&old.transforms)
        || !cache.children.transforms.starts_with(&old.transforms)
        || !Arc::ptr_eq(&old_nodes.transform, &new_nodes.transform)
        || !Arc::ptr_eq(&old_nodes.clip, &new_nodes.clip)
        || !Arc::ptr_eq(&old_nodes.effect, &new_nodes.effect)
    {
        return None;
    }
    let inherited_scroll = new_nodes.nearest_scroll();
    paint_model::property_cell::with_access(|access| {
        let mut rebase = OwnNodeRebase {
            access,
            own_ids: &cache.own_keys,
            inherited: new_nodes,
            inherited_scroll,
            proven: Vec::new(),
        };
        (rebase.state(&node.properties)
            && rebase.state(&node.contents_properties)
            && rebase.state(&cache.children.properties))
        .then(|| RebasedOwnStates {
            properties: node.properties.clone(),
            contents: node.contents_properties.clone(),
            children: cache.children.properties.clone(),
            pre_transform: rebase
                .transform(&node.pre_transform)
                .then(|| node.pre_transform.clone()),
        })
    })
}

fn MovedRect(rect: Option<PaintRect>, offset: Offset) -> Option<PaintRect> {
    rect.map(|mut rect| {
        rect.x += offset.x;
        rect.y += offset.y;
        rect
    })
}

/// Trace-only classification of a rebuilt node for build_kind_ns.
fn BuildKind(node: &PaintTreeNode<'_>, fragment: &FragmentPaintData) -> usize {
    use PaintPropertyRole as R;
    let roles: Vec<PaintPropertyRole> = node
        .cache
        .as_ref()
        .map(|cache| cache.own_keys.iter().map(|(_, key)| key.role).collect())
        .unwrap_or_default();
    let is_transform = |role: &PaintPropertyRole| {
        matches!(
            role,
            R::PaintOffsetTranslation
                | R::Transform
                | R::ReplacedContentTransform
                | R::SvgLocalTransform
        )
    };
    let is_effect = |role: &PaintPropertyRole| matches!(role, R::Effect | R::Filter | R::Mask);
    if IsSvgLayoutChild(fragment.paint.SourceKind()) || !node.local_transforms.is_empty() {
        3
    } else if roles.iter().any(is_transform) {
        0
    } else if !roles.is_empty() && !roles.iter().any(is_effect) {
        1
    } else if !roles.is_empty() {
        2
    } else if node.is_paint_layer {
        4
    } else {
        5
    }
}

/// How far an offset-only update moved a subtree and its property space.
#[derive(Clone, Copy)]
struct ShiftDelta {
    offset: Offset,
    paint_space: Offset,
}

// Move a retained node (proven by ShiftPreservesClips) by `delta` under its
// new inherited context.
// Returns whether the new cull rects need a repaint (CullRectUpdater), as
// the rebuild path decides it; the node's repaint bit is raised already.
thread_local! {
    /// Trace only: ShiftRetainedNode time by part (refresh layer facts,
    /// property/context update, remainder).
    static SHIFT_PART_NS: std::cell::Cell<[u64; 3]> = const { std::cell::Cell::new([0; 3]) };
}

fn AddShiftPart(index: usize, from: Option<std::time::Instant>) -> Option<std::time::Instant> {
    let from = from?;
    let now = std::time::Instant::now();
    SHIFT_PART_NS.with(|parts| {
        let mut value = parts.get();
        value[index] += now.duration_since(from).as_nanos() as u64;
        parts.set(value);
    });
    Some(now)
}

pub fn TakeShiftPartNs() -> [u64; 3] {
    SHIFT_PART_NS.with(|parts| parts.replace([0; 3]))
}

fn ShiftRetainedNode(
    node: &mut PaintTreeNode<'static>,
    delta: ShiftDelta,
    inherited: &InheritedContext,
    previous_paint_result: paint_model::PaintResult,
    rebased: Option<RebasedOwnStates>,
    viewport: &paint_model::paint_property_tree::PropertyTreeState,
) -> bool {
    if let Some(rebased) = rebased {
        return RebaseRetainedNode(node, delta, inherited, previous_paint_result, rebased, viewport);
    }
    let t = browser_tracing::enabled().then(std::time::Instant::now);
    let cull_change_needs_repaint = CullRectChangeNeedsRepaint(
        previous_paint_result,
        Some(node.self_cull_rect),
        Some(node.cull_rect),
        inherited.cull,
        inherited.cull,
    );
    RefreshLivePaintLayerFacts(node);
    let t = AddShiftPart(0, t);
    node.self_or_descendant_needs_repaint |= cull_change_needs_repaint;
    // Paint offsets are relative to the property space, which moves by
    // `paint_space` when the parent's own PaintOffsetTranslation moved. An
    // SVG local transform resets the paint space to the object's own
    // position, so its paint offset stays zero and its children's space
    // moves with the object.
    let resets_paint_space = node
        .fragment
        .as_deref()
        .is_some_and(|fragment| fragment.paint.SvgLocalTransform().is_some());
    node.absolute_offset.x += delta.offset.x;
    node.absolute_offset.y += delta.offset.y;
    if !resets_paint_space {
        node.paint_offset.x += delta.offset.x - delta.paint_space.x;
        node.paint_offset.y += delta.offset.y - delta.paint_space.y;
    }
    let children_paint_space = if resets_paint_space { delta.offset } else { delta.paint_space };
    node.clip = inherited.clip;
    node.cull_rect = inherited.cull;
    node.self_cull_rect = inherited.cull;
    let old_inherited_len = node
        .cache
        .as_ref()
        .expect("shifted node has builder state")
        .inherited
        .transforms
        .len();
    node.transforms = ShiftedTransforms(&node.transforms, old_inherited_len, &inherited.transforms);
    let shifted_transforms = node.transforms.clone();
    // Without own nodes the object paints in its parent's property state.
    // An own plain effect (RebasableOwnEffect) is a resident node already
    // linked to that same state's nodes, so it keeps its property state.
    let properties = {
        let cache = node.cache.as_ref().expect("shifted node has builder state");
        if cache.own_keys.is_empty() {
            inherited.properties.clone()
        } else {
            node.properties.clone()
        }
    };
    node.properties = properties.clone();
    node.contents_properties = properties.clone();
    let cache = node.cache.as_mut().expect("shifted node has builder state");
    cache.children.properties = properties;
    cache.inherited = inherited.clone();
    let t = AddShiftPart(1, t);
    cache.children.origin.x += delta.offset.x;
    cache.children.origin.y += delta.offset.y;
    cache.children.paint_space.x += children_paint_space.x;
    cache.children.paint_space.y += children_paint_space.y;
    cache.children.clip = inherited.clip;
    cache.children.cull = inherited.cull;
    cache.children.transforms = shifted_transforms;
    AddShiftPart(2, t);
    cull_change_needs_repaint
}

/// ShiftRetainedNode for an object with own property nodes (RebaseOwnStates).
fn RebaseRetainedNode(
    node: &mut PaintTreeNode<'static>,
    delta: ShiftDelta,
    inherited: &InheritedContext,
    previous_paint_result: paint_model::PaintResult,
    rebased: RebasedOwnStates,
    viewport: &paint_model::paint_property_tree::PropertyTreeState,
) -> bool {
    let fragment = node.fragment.as_deref().expect("shifted node keeps its fragment");
    let inputs = node.cull_decision_inputs;
    let infinite = crate::cull_rect_updater::ShouldUseInfiniteCullRect(
        fragment,
        inherited.subtree_should_use_infinite_cull_rect,
        inputs.has_filter_that_moves_pixels,
        inputs.has_perspective,
        inputs.has_active_transform_animation,
        inputs.has_non_composited_transform,
        previous_paint_result,
    );
    let (self_cull_rect, cull_rect) = if infinite.use_infinite {
        (None, None)
    } else {
        let expansion_ratio = crate::cull_rect_updater::ExpansionRatio(
            node.DocumentResources().map(|resources| &**resources),
        );
        let (self_cull, _) = CullRectUpdater::ComputeFragmentCullRect(
            inherited.cull,
            Some(node.self_cull_rect),
            viewport,
            &inherited.properties.nodes,
            &rebased.properties.nodes,
            expansion_ratio,
        );
        let (contents_cull, _) = CullRectUpdater::ComputeFragmentContentsCullRect(
            self_cull,
            Some(node.cull_rect),
            viewport,
            &rebased.properties.nodes,
            &rebased.contents.nodes,
            expansion_ratio,
        );
        (self_cull, contents_cull)
    };
    let cull_change_needs_repaint = CullRectChangeNeedsRepaint(
        previous_paint_result,
        Some(node.self_cull_rect),
        Some(node.cull_rect),
        self_cull_rect,
        cull_rect,
    );
    RefreshLivePaintLayerFacts(node);
    node.self_or_descendant_needs_repaint |= cull_change_needs_repaint;
    // The paint offset inside the moved property space is unchanged.
    node.absolute_offset.x += delta.offset.x;
    node.absolute_offset.y += delta.offset.y;
    node.clip = MovedRect(node.clip, delta.offset);
    node.self_cull_rect = self_cull_rect;
    node.cull_rect = cull_rect;
    let cache = node.cache.as_mut().expect("shifted node has builder state");
    let old_inherited_len = cache.inherited.transforms.len();
    node.transforms = ShiftedTransforms(&node.transforms, old_inherited_len, &inherited.transforms);
    cache.children.transforms =
        ShiftedTransforms(&cache.children.transforms, old_inherited_len, &inherited.transforms);
    if let Some(pre_transform) = rebased.pre_transform {
        node.pre_transform = pre_transform;
    }
    node.properties = rebased.properties;
    node.contents_properties = rebased.contents;
    cache.children.properties = rebased.children;
    cache.inherited = inherited.clone();
    cache.children.origin.x += delta.offset.x;
    cache.children.origin.y += delta.offset.y;
    cache.children.paint_space.x += delta.offset.x;
    cache.children.paint_space.y += delta.offset.y;
    cache.children.clip = MovedRect(cache.children.clip, delta.offset);
    cache.children.cull = cull_rect;
    cull_change_needs_repaint
}

/// Re-read the live PaintLayer facts the builder reads on every visit
/// (paint_property_tree_builder.rs BuildNode); a retained node would
/// otherwise keep the values of its last build.
fn RefreshLivePaintLayerFacts(node: &mut PaintTreeNode<'static>) {
    let Some(fragment) = node.fragment.as_deref() else {
        return;
    };
    let facts = fragment.paint.LivePaintLayerFacts();
    node.paint_layer_facts_are_authoritative = facts.authoritative;
    if facts.authoritative {
        node.is_self_painting_layer = facts.is_self_painting;
    } else {
        node.is_self_painting_layer = node.is_paint_layer;
    }
    node.invisible_for_position_visibility = facts.invisible_for_position_visibility;
    node.has_ancestor_invisible_for_position_visibility =
        facts.has_ancestor_invisible_for_position_visibility;
    node.has_visible_content = facts.has_visible_content;
    node.has_self_painting_layer_descendant = facts.has_self_painting_layer_descendant;
    node.is_under_svg_hidden_container = facts.is_under_svg_hidden_container;
    node.supports_subsequence_caching = facts.supports_subsequence_caching;
    node.self_or_descendant_needs_repaint = facts.self_or_descendant_needs_repaint;
    node.paint_layer_is_fragmented = facts.is_fragmented;
    node.known_to_clip_subtree_to_padding_box = facts.known_to_clip_subtree_to_padding_box;
    node.child_paint_blocked_by_display_lock = facts.child_paint_blocked_by_display_lock;
}

/// The fields ShiftRetainedNode writes, for the verification mode.
struct SavedShiftFields {
    absolute_offset: Offset,
    paint_offset: Offset,
    clip: Option<PaintRect>,
    cull_rect: Option<PaintRect>,
    self_cull_rect: Option<PaintRect>,
    transforms: FlatTransforms,
    properties: PropertyTreeState,
    contents_properties: PropertyTreeState,
    self_or_descendant_needs_repaint: bool,
    layer_facts: [bool; 11],
    inherited: InheritedContext,
    children: ChildContext,
}

fn LayerFacts(node: &PaintTreeNode<'_>) -> [bool; 11] {
    [
        node.paint_layer_facts_are_authoritative,
        node.is_self_painting_layer,
        node.invisible_for_position_visibility,
        node.has_ancestor_invisible_for_position_visibility,
        node.has_visible_content,
        node.has_self_painting_layer_descendant,
        node.is_under_svg_hidden_container,
        node.supports_subsequence_caching,
        node.paint_layer_is_fragmented,
        node.known_to_clip_subtree_to_padding_box,
        node.child_paint_blocked_by_display_lock,
    ]
}

fn SetLayerFacts(node: &mut PaintTreeNode<'_>, facts: [bool; 11]) {
    [
        node.paint_layer_facts_are_authoritative,
        node.is_self_painting_layer,
        node.invisible_for_position_visibility,
        node.has_ancestor_invisible_for_position_visibility,
        node.has_visible_content,
        node.has_self_painting_layer_descendant,
        node.is_under_svg_hidden_container,
        node.supports_subsequence_caching,
        node.paint_layer_is_fragmented,
        node.known_to_clip_subtree_to_padding_box,
        node.child_paint_blocked_by_display_lock,
    ] = facts;
}

impl SavedShiftFields {
    fn Save(node: &PaintTreeNode<'_>) -> Self {
        let cache = node.cache.as_ref().expect("cached node");
        Self {
            absolute_offset: node.absolute_offset,
            paint_offset: node.paint_offset,
            clip: node.clip,
            cull_rect: node.cull_rect,
            self_cull_rect: node.self_cull_rect,
            transforms: node.transforms.clone(),
            properties: node.properties.clone(),
            contents_properties: node.contents_properties.clone(),
            self_or_descendant_needs_repaint: node.self_or_descendant_needs_repaint,
            layer_facts: LayerFacts(node),
            inherited: cache.inherited.clone(),
            children: cache.children.clone(),
        }
    }

    fn Restore(self, node: &mut PaintTreeNode<'_>) {
        node.absolute_offset = self.absolute_offset;
        node.paint_offset = self.paint_offset;
        node.clip = self.clip;
        node.cull_rect = self.cull_rect;
        node.self_cull_rect = self.self_cull_rect;
        node.transforms = self.transforms;
        node.properties = self.properties;
        node.contents_properties = self.contents_properties;
        node.self_or_descendant_needs_repaint = self.self_or_descendant_needs_repaint;
        SetLayerFacts(node, self.layer_facts);
        let cache = node.cache.as_mut().expect("cached node");
        cache.inherited = self.inherited;
        cache.children = self.children;
    }
}

/// Verification mode: the first geometry field of a shifted node that
/// differs from the rebuilt node.
fn ShiftFieldMismatch(a: &PaintTreeNode<'_>, b: &PaintTreeNode<'_>) -> Option<&'static str> {
    macro_rules! first_diff {
        ($($field:ident),* $(,)?) => {{
            $(if a.$field != b.$field { return Some(stringify!($field)); })*
            None
        }};
    }
    // A newly built SVG child receives its container's matrix as a leading
    // local transform only after it returns to the parent (UpdateNode's
    // child loop); the shifted node already carries it.
    if a.local_transforms != b.local_transforms {
        let expected = b.cache.as_ref().and_then(|cache| cache.inherited.transforms.last());
        let prefixed = a.local_transforms.len() == b.local_transforms.len() + 1
            && a.local_transforms[1..] == b.local_transforms[..]
            && expected == a.local_transforms.first();
        if !prefixed {
            return Some("local_transforms");
        }
        if !a.applies_transform {
            return Some("applies_transform");
        }
    } else if a.applies_transform != b.applies_transform {
        return Some("applies_transform");
    }
    first_diff!(
        absolute_offset,
        paint_offset,
        directly_composited_container_paint_offset_subpixel_delta,
        paint_snap_offset,
        pixel_snap_scale,
        root_clip,
        local_inner_border_radius_clip,
        local_inner_border_radius_radii,
        local_clip,
        local_css_clip,
        local_clip_radius,
        local_clip_radii,
        local_clip_path,
        clip_path_even_odd,
        clip,
        cull_rect,
        self_cull_rect,
        transforms,
        scroll_offset,
        fixed_background_acts_as_scroll,
        is_paint_layer,
        paint_layer_facts_are_authoritative,
        is_self_painting_layer,
        invisible_for_position_visibility,
        has_ancestor_invisible_for_position_visibility,
        has_visible_content,
        has_self_painting_layer_descendant,
        is_under_svg_hidden_container,
        supports_subsequence_caching,
        self_or_descendant_needs_repaint,
        paint_layer_is_fragmented,
        known_to_clip_subtree_to_padding_box,
        child_paint_blocked_by_display_lock,
        is_stacking_context,
        is_stacked,
        is_root,
        stacking_level,
        is_z_order_list_visible,
        is_in_top_or_view_transition_layer,
        is_direct_layout_view_child,
        can_contain_absolute_position_objects,
        can_contain_fixed_position_objects,
        has_overlay_overflow_controls,
        paint_layer_order_parent_unique_id,
        paint_layer_order,
        applies_clip,
        applies_root_clip,
        applies_overflow_clip,
        applies_css_clip,
        applies_clip_path,
        applies_effect,
        applies_opacity,
        applies_blend,
        applies_filter,
        applies_backdrop_filter,
        applies_mask,
        mask_layers,
        overflow_controls_properties,
    )
}

/// Verification mode: the child context a shifted node hands down must equal
/// the one the builder computed.
fn ShiftChildrenMismatch(shifted: &PaintTreeNode<'_>, built: &PaintTreeNode<'_>) -> Option<&'static str> {
    let a = &shifted.cache.as_ref()?.children;
    let b = &built.cache.as_ref()?.children;
    if a.origin != b.origin {
        return Some("children.origin");
    }
    if a.paint_space != b.paint_space {
        return Some("children.paint_space");
    }
    if a.clip != b.clip {
        return Some("children.clip");
    }
    if a.cull != b.cull {
        return Some("children.cull");
    }
    if a.transforms != b.transforms {
        return Some("children.transforms");
    }
    if a.opacity != b.opacity || a.scroll != b.scroll || a.svg != b.svg {
        return Some("children.opacity_scroll_svg");
    }
    if a.directly_composited_container_paint_offset_subpixel_delta
        != b.directly_composited_container_paint_offset_subpixel_delta
        || a.subtree_should_use_infinite_cull_rect != b.subtree_should_use_infinite_cull_rect
        || a.flex_or_grid_items != b.flex_or_grid_items
    {
        return Some("children.flags");
    }
    None
}

/// Nanoseconds between two profiling checkpoints; zero when not profiling.
fn ProfileNs(from: Option<std::time::Instant>, to: Option<std::time::Instant>) -> u64 {
    match (from, to) {
        (Some(from), Some(to)) => to.duration_since(from).as_nanos().min(u64::MAX as u128) as u64,
        _ => 0,
    }
}

fn RecordOf(node: &PaintTreeNode<'_>) -> PrePaintPrevious {
    node.cache.as_ref().map_or_else(PrePaintPrevious::default, |cache| cache.record)
}

/// SameSourceIdentity of the retained node `previous` (by its record) and
/// the paired `new` fragment. The enclosing fragmentainer is not part of it:
/// the walk pairs a node only under a parent of the same identity, so the
/// fragmentainer ancestor (itself identified by owner and break-token
/// sequence) is already the same, as in PrePaintTreeWalk's
/// `fragmentainer_idx` context.
fn SamePreviousSourceIdentity(previous: &PaintTreeNode<'_>, new: &FragmentPaintData) -> bool {
    let recorded = RecordOf(previous);
    recorded.kind == Some(new.ViewKind())
        && recorded.node_id == new.ViewNodeId()
        && recorded.fragment_instance_id == new.ViewFragmentInstanceId()
        && recorded.display_item_client_id == new.paint.DisplayItemClientId()
        && recorded.display_item_fragment == new.ViewDisplayItemFragment()
}

/// SamePhaseInputs against the recorded previous values.
fn SamePreviousPhaseInputs(recorded: &PrePaintPrevious, current: &PrePaintPrevious) -> bool {
    recorded.phase_flags == current.phase_flags
        && recorded.floating == current.floating
        && recorded.display == current.display
        && recorded.outline_style == current.outline_style
        && recorded.outline_width == current.outline_width
        && recorded.outline_alpha == current.outline_alpha
        && recorded.size == current.size
}

/// The logical-tree inputs SameGeometry and the invalidation-only refresh
/// compare, against the recorded previous values.
fn SamePreviousLogicalPosition(recorded: &PrePaintPrevious, current: &PrePaintPrevious) -> bool {
    recorded.position == current.position
        && recorded.logical_parent_node_id == current.logical_parent_node_id
        && recorded.logical_tree_order == current.logical_tree_order
}

/// The DisplayItemClient ids of a fragment's paint payload: the object, its
/// PaintLayer and, for a scroller, the scroll corner and scrollbar clients.
fn ClientDirectoryIds(fragment: &FragmentPaintData) -> ClientIds {
    let paint = &fragment.paint;
    ClientIds {
        display_item: paint.DisplayItemClientId(),
        paint_layer: paint.PaintLayerClientId(),
        scrollbars: fragment.ViewScrollbars().map(|bars| {
            (
                bars.corner_client_id,
                bars.horizontal.as_ref().map(|axis| axis.display_item_client_id),
                bars.vertical.as_ref().map(|axis| axis.display_item_client_id),
            )
        }),
    }
}

/// The DisplayItemClient ids PrePaint recorded for a retained node: its
/// client directory plus the selection and scrolling-background clients.
fn RecordedClientIds(recorded: &PrePaintPrevious) -> [u64; 7] {
    let (corner, horizontal, vertical) = recorded.clients.scrollbars.unwrap_or((0, None, None));
    [
        recorded.clients.display_item,
        recorded.clients.paint_layer,
        recorded.selection_client_id,
        recorded.scrolling_background_client_id,
        corner,
        horizontal.unwrap_or(0),
        vertical.unwrap_or(0),
    ]
}

/// Whether a live client is new to PrePaint's record. Chromium's
/// DisplayItemClient::IsJustCreated holds until a paint commit validates the
/// client (display_item_client.h:71-74), so a client PrePaint walked but
/// Paint never committed (for example a culled one) is still just created.
/// Client ids are process-unique tokens: a just-created client whose id the
/// retained node recorded is that same client lifetime, not a new one.
fn IsNewClient(recorded: &[u64; 7], id: u64, just_created: bool) -> bool {
    just_created && !recorded.contains(&id)
}

/// Whether any live DisplayItemClient of `fragment` (object, selection,
/// PaintLayer, scrolling background, scroll corner and scrollbars) is just
/// created and absent from the retained node's `recorded` client ids.
fn HasUnrecordedJustCreatedClient(recorded: &PrePaintPrevious, fragment: &FragmentPaintData) -> bool {
    let recorded = RecordedClientIds(recorded);
    let paint = &fragment.paint;
    IsNewClient(
        &recorded,
        paint.DisplayItemClientId(),
        paint.DisplayItemClientState().is_just_created,
    ) || IsNewClient(
        &recorded,
        paint.SelectionClientId(),
        paint.SelectionClientState().is_just_created,
    ) || IsNewClient(
        &recorded,
        paint.PaintLayerClientId(),
        paint.PaintLayerClientState().is_just_created,
    ) || fragment.ViewScrollContainer().is_some_and(|scroll| {
        IsNewClient(
            &recorded,
            scroll.scrolling_background_client_id,
            paint
                .ScrollClientState(
                    scroll.scrolling_background_client_id,
                    scroll.scrolling_background_client_invalidation_reason,
                    scroll.scrolling_background_client_is_cacheable,
                    scroll.scrolling_background_client_is_just_created.get(),
                )
                .is_just_created,
        )
    }) || fragment.ViewScrollbars().is_some_and(|bars| {
        IsNewClient(
            &recorded,
            bars.corner_client_id,
            paint
                .ScrollClientState(
                    bars.corner_client_id,
                    bars.corner_client_invalidation_reason,
                    bars.corner_client_is_cacheable,
                    bars.corner_client_is_just_created.get(),
                )
                .is_just_created,
        ) || [&bars.horizontal, &bars.vertical]
            .into_iter()
            .flatten()
            .any(|axis| {
                IsNewClient(
                    &recorded,
                    axis.display_item_client_id,
                    paint
                        .ScrollClientState(
                            axis.display_item_client_id,
                            axis.display_item_client_invalidation_reason,
                            axis.display_item_client_is_cacheable,
                            axis.display_item_client_is_just_created.get(),
                        )
                        .is_just_created,
                )
            })
    })
}

/// Whether `b` keeps the client directory of the retained node `previous`.
/// The previous ids come from the node's record. Equal ids name the same
/// live DisplayItemClient, so its cacheability is the same object state;
/// only a just-created client the record does not name is new
/// (IsNewClient).
fn SameClientDirectory(previous: &PaintTreeNode<'_>, b: &FragmentPaintData) -> bool {
    let y = &b.paint;
    let recorded = RecordOf(previous);
    if recorded.clients != ClientDirectoryIds(b) {
        return false;
    }
    let recorded = RecordedClientIds(&recorded);
    let (y_item, y_layer) = (y.DisplayItemClientState(), y.PaintLayerClientState());
    let mut created = IsNewClient(&recorded, y.DisplayItemClientId(), y_item.is_just_created)
        || IsNewClient(&recorded, y.PaintLayerClientId(), y_layer.is_just_created);
    if let Some(bars) = &y.scrollbars {
        created |= IsNewClient(
            &recorded,
            bars.corner_client_id,
            y.ScrollClientState(
                bars.corner_client_id,
                bars.corner_client_invalidation_reason,
                bars.corner_client_is_cacheable,
                bars.corner_client_is_just_created.get(),
            )
            .is_just_created,
        );
        for axis in [&bars.horizontal, &bars.vertical].into_iter().flatten() {
            created |= IsNewClient(
                &recorded,
                axis.display_item_client_id,
                y.ScrollClientState(
                    axis.display_item_client_id,
                    axis.display_item_client_invalidation_reason,
                    axis.display_item_client_is_cacheable,
                    axis.display_item_client_is_just_created.get(),
                )
                .is_just_created,
            );
        }
    }
    !created
}

/// Source: `pre_paint_tree_walk.cc:430-437`.
#[allow(non_snake_case)]
fn ObjectRequiresTreeBuilderContext(object: &FragmentPaintData) -> bool {
    let invalidation = object.pre_paint_invalidation;
    invalidation.needs_paint_property_update
        || invalidation.should_check_layout_for_paint_invalidation
        || (!invalidation.child_pre_paint_blocked_by_display_lock
            && (invalidation.descendant_needs_paint_property_update
                || invalidation.descendant_should_check_layout_for_paint_invalidation))
}

/// Chromium still visits objects marked for paint-invalidation checking, but
/// PaintPropertyTreeBuilder retains their FragmentData property slots when
/// NeedsPaintPropertyUpdate is clear. The projected Rust tree mirrors those
/// stable slots: refresh the shallow source while retaining completed
/// geometry/property/phase data only when the direct layout inputs also prove
/// unchanged. Any property update or geometry/topology change falls back to
/// the complete builder comparison.
fn CanRefreshInvalidationOnlySource(
    previous: &PaintTreeNode<'_>,
    new: &FragmentPaintData,
    current: &PrePaintPrevious,
) -> bool {
    let invalidation = new.pre_paint_invalidation;
    if !invalidation.own()
        || ObjectRequiresTreeBuilderContext(new)
        || !SamePreviousSourceIdentity(previous, new)
    {
        return false;
    }
    let recorded = &RecordOf(previous);
    SamePreviousPhaseInputs(recorded, current)
        && recorded.offset == current.offset
        && recorded.content_size == current.content_size
        && SamePreviousLogicalPosition(recorded, current)
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
    /// The persisted property trees as published with this pass's artifact.
    pub fn PublishedTrees(&mut self) -> Arc<paint_model::paint_property_tree::FrozenPropertyTrees> {
        self.property_node_store.published_trees()
    }
    pub fn IntersectionObservationState(
        &self,
    ) -> foundation::intersection_observation_state::IntersectionObservationState {
        self.intersection_observation_state
    }
    fn RebindResidentPropertySnapshots(&mut self, trees: &mut PaintPropertyTrees) {
        let mut refresh = |state: &mut paint_model::paint_property_tree::PropertyTreeState| {
            trees.clear_state_change_markers(state);
        };
        refresh(&mut self.viewport_properties.nodes);
        if let Some(fixed) = &mut self.root_fixed_position {
            refresh(&mut fixed.properties.nodes);
        }
        for &pointer in &self.resident_nodes {
            // RetainTree rebuilds this directory whenever owners change. All
            // boxed allocations remain owned by retained_tree.
            let node = unsafe { &mut *pointer };
            refresh(&mut node.properties.nodes);
            refresh(&mut node.contents_properties.nodes);
            if let Some(fixed) = &mut node.fixed_background_properties {
                refresh(fixed);
            }
            if let Some(mask) = &mut node.mask_properties {
                refresh(mask);
            }
            if let Some(scrollbar) = &mut node.horizontal_scrollbar_properties {
                refresh(scrollbar);
            }
            if let Some(scrollbar) = &mut node.vertical_scrollbar_properties {
                refresh(scrollbar);
            }
            if let Some(cache) = &mut node.cache {
                refresh(&mut cache.inherited.properties.nodes);
                refresh(&mut cache.children.properties.nodes);
            }
        }
        drop(refresh);
        if let Some(fixed) = &mut self.root_fixed_position {
            if let Some(scroll_translation) = &fixed.scroll_translation {
                fixed.scroll_translation = trees.transform_for(scroll_translation).cloned();
            }
        }
    }

    /// Blink mutates a property node in its stable slot. The Rust property
    /// store publishes immutable revisions, so direct scroll publication must
    /// move every resident reference to the new revision at the same boundary.
    pub fn CommitDirectScrollPropertySnapshots(
        &mut self,
        store: PaintPropertyNodeStore,
        trees: &mut PaintPropertyTrees,
    ) {
        self.RebindResidentPropertySnapshots(trees);
        self.property_node_store = store;
        self.property_node_store.invalidate_published();
    }

    fn ClipContextCovers(old: &Option<PaintRect>, new: &Option<PaintRect>) -> bool {
        paint_model::cull::ClipContextCovers(*old, *new)
    }

    fn CullContextCovers(old: &Option<PaintRect>, new: &Option<PaintRect>) -> bool {
        paint_model::cull::CullContextCovers(*old, *new)
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
    fn SameCanvasContents(
        &self,
        a: &Option<CanvasContentsContext>,
        b: &Option<CanvasContentsContext>,
    ) -> bool {
        match (a, b) {
            (None, None) => true,
            (Some(a), Some(b)) => {
                a.layout_object == b.layout_object
                    && state_compare::SameBuilderStructure(
                        &a.properties.nodes,
                        &b.properties.nodes,
                    )
            }
            _ => false,
        }
    }

    fn SameContext(&self, a: &InheritedContext, b: &InheritedContext) -> bool {
        self.SameContextImpl(a, b, true)
    }

    /// The offset delta when two inherited contexts differ only by where the
    /// parent placed this subtree (Chromium updates only PaintOffset for an
    /// object without its own paint properties).
    /// Also admits a moved transformed ancestor: the flat transform values a
    /// descendant inherits change while their structure stays the same.
    fn OffsetOnlyContextDelta(&self, a: &InheritedContext, b: &InheritedContext) -> Option<ShiftDelta> {
        ((a.offset != b.offset || a.transforms != b.transforms)
            && self.SameContextImpl(a, b, false))
        .then(|| ShiftDelta {
            offset: Offset {
                x: b.offset.x - a.offset.x,
                y: b.offset.y - a.offset.y,
            },
            paint_space: Offset {
                x: b.paint_space.x - a.paint_space.x,
                y: b.paint_space.y - a.paint_space.y,
            },
        })
    }

    fn SameContextImpl(&self, a: &InheritedContext, b: &InheritedContext, exact: bool) -> bool {
        let cull_covers = Self::CullContextCovers(&a.cull, &b.cull);
        a.inside_blocking_wheel_event_handler == b.inside_blocking_wheel_event_handler
            && (!exact || a.offset == b.offset)
            && (exact || a.transforms.len() == b.transforms.len())
            && (a.paint_space == b.paint_space
                || (!exact
                    && b.paint_space.x - a.paint_space.x == b.offset.x - a.offset.x
                    && b.paint_space.y - a.paint_space.y == b.offset.y - a.offset.y))
            // A shrinking inherited clip/cull cannot expose content omitted
            // by the retained node. The updated ancestor still publishes its
            // exact current clip; descendants may keep their larger recorded
            // coverage and rebind to that same property slot, matching Blink's
            // mutable property-node and cull-hysteresis behavior.
            && (Self::ClipContextCovers(&a.clip, &b.clip) || cull_covers)
            && cull_covers
            && (!exact
                || Arc::ptr_eq(&a.transforms, &b.transforms)
                || a.transforms == b.transforms)
            && a.opacity == b.opacity
            && a.scroll == b.scroll
            && a.fixed_background_acts_as_scroll == b.fixed_background_acts_as_scroll
            && a.rendering_context_id == b.rendering_context_id
            && a.should_flatten_inherited_transform == b.should_flatten_inherited_transform
            && a.can_inherit_backface_visibility == b.can_inherit_backface_visibility
            && a.directly_composited_container_paint_offset_subpixel_delta
                == b.directly_composited_container_paint_offset_subpixel_delta
            && a.subtree_should_use_infinite_cull_rect
                == b.subtree_should_use_infinite_cull_rect
            && a.root == b.root
            && a.flex_or_grid_item == b.flex_or_grid_item
            && self.SameCanvasContents(&a.canvas_contents, &b.canvas_contents)
            && state_compare::SameBuilderStructure(
                &a.properties.nodes,
                &b.properties.nodes,
            )
    }

    /// CullRectUpdater changes composited transform values and the sliding
    /// cull window without changing descendant geometry or property slots.
    /// Chromium retains those FragmentData records and repaints only content
    /// newly admitted by the cull window.  The flat transform/scroll vectors
    /// here are builder inputs mirrored by the semantic property slots below;
    /// their values therefore do not make an otherwise clean descendant a new
    /// object. Offsets and topology still have to match, while the retained
    /// clip/cull window may be larger than the exact successor window.
    fn SameRetainedContextStructure(&self, a: &InheritedContext, b: &InheritedContext) -> bool {
        a.inside_blocking_wheel_event_handler == b.inside_blocking_wheel_event_handler
            && a.offset == b.offset
            && a.paint_space == b.paint_space
            && (Self::ClipContextCovers(&a.clip, &b.clip)
                || Self::CullContextCovers(&a.cull, &b.cull))
            && a.transforms.len() == b.transforms.len()
            && a.opacity == b.opacity
            && a.fixed_background_acts_as_scroll == b.fixed_background_acts_as_scroll
            && a.rendering_context_id == b.rendering_context_id
            && a.should_flatten_inherited_transform == b.should_flatten_inherited_transform
            && a.can_inherit_backface_visibility == b.can_inherit_backface_visibility
            && a.directly_composited_container_paint_offset_subpixel_delta
                == b.directly_composited_container_paint_offset_subpixel_delta
            && a.subtree_should_use_infinite_cull_rect == b.subtree_should_use_infinite_cull_rect
            && a.root == b.root
            && a.flex_or_grid_item == b.flex_or_grid_item
            && self.SameCanvasContents(&a.canvas_contents, &b.canvas_contents)
            && state_compare::SameBuilderStructure(
                &a.properties.nodes,
                &b.properties.nodes,
            )
    }

    fn SameGeometry(
        &self,
        a: &PaintTreeNode<'_>,
        b: &PaintTreeNode<'_>,
        current: &PrePaintPrevious,
    ) -> bool {
        let new = b.cache.as_ref().expect("new geometry");
        let bf = b.fragment.as_deref().unwrap();
        if IsNewClient(
            &RecordedClientIds(&RecordOf(a)),
            bf.paint.DisplayItemClientId(),
            bf.paint.DisplayItemClientState().is_just_created,
        ) || !SamePreviousSourceIdentity(a, bf)
        {
            return false;
        }
        let recorded = RecordOf(a);
        if !SamePreviousPhaseInputs(&recorded, current)
            || !SamePreviousLogicalPosition(&recorded, current)
        {
            return false;
        }
        macro_rules! same { ($($field:ident),* $(,)?) => { true $(&& a.$field == b.$field)* }; }
        same!(
            absolute_offset,
            paint_offset,
            directly_composited_container_paint_offset_subpixel_delta,
            paint_snap_offset,
            pixel_snap_scale,
            root_clip,
            local_inner_border_radius_clip,
            local_inner_border_radius_radii,
            local_clip,
            local_css_clip,
            local_clip_radius,
            local_clip_radii,
            local_clip_path,
            clip_path_even_odd,
            clip,
            cull_rect,
            self_cull_rect,
            transforms,
            local_transforms,
            scroll_offset,
            fixed_background_acts_as_scroll,
            is_paint_layer,
            paint_layer_facts_are_authoritative,
            is_self_painting_layer,
            invisible_for_position_visibility,
            has_ancestor_invisible_for_position_visibility,
            has_visible_content,
            has_self_painting_layer_descendant,
            is_under_svg_hidden_container,
            supports_subsequence_caching,
            self_or_descendant_needs_repaint,
            paint_layer_is_fragmented,
            known_to_clip_subtree_to_padding_box,
            child_paint_blocked_by_display_lock,
            is_stacking_context,
            is_stacked,
            is_root,
            stacking_level,
            is_z_order_list_visible,
            is_in_top_or_view_transition_layer,
            is_direct_layout_view_child,
            can_contain_absolute_position_objects,
            can_contain_fixed_position_objects,
            has_overlay_overflow_controls,
            paint_layer_order_parent_unique_id,
            paint_layer_order,
            applies_transform,
            applies_clip,
            applies_root_clip,
            applies_overflow_clip,
            applies_css_clip,
            applies_clip_path,
            applies_effect,
            applies_opacity,
            applies_blend,
            applies_filter,
            applies_backdrop_filter,
            applies_mask,
            mask_layers,
            overflow_controls_properties
        ) && a.opacity == b.opacity
            && new.own_change == PaintPropertyChangeType::Unchanged
            && state_compare::SameBuilderState(
                &a.properties.nodes,
                &b.properties.nodes,
            )
            && state_compare::SameBuilderState(
                &a.contents_properties.nodes,
                &b.contents_properties.nodes,
            )
            && match (&a.mask_properties, &b.mask_properties) {
                (None, None) => true,
                (Some(a), Some(b)) => state_compare::SameBuilderState(
                    a,
                    b,
                ),
                _ => false,
            }
            && match (&a.fixed_background_properties, &b.fixed_background_properties) {
                (None, None) => true,
                (Some(a), Some(b)) => state_compare::SameBuilderState(
                    a,
                    b,
                ),
                _ => false,
            }
            && match (&a.horizontal_scrollbar_properties, &b.horizontal_scrollbar_properties) {
                (None, None) => true,
                (Some(a), Some(b)) => state_compare::SameBuilderState(
                    a,
                    b,
                ),
                _ => false,
            }
            && match (&a.vertical_scrollbar_properties, &b.vertical_scrollbar_properties) {
                (None, None) => true,
                (Some(a), Some(b)) => state_compare::SameBuilderState(
                    a,
                    b,
                ),
                _ => false,
            }
    }

    /// The first rule by which the retained `node` cannot stand for
    /// `fragment`'s own PrePaint state, or None. Source:
    /// pre_paint_tree_walk.cc:418-437 visits an object for its own dirty
    /// bits (PrePaintInvalidation::own, read live from the LayoutObject).
    /// Chromium also keeps, on the same LayoutObject/FragmentData, values
    /// that change without dirtying this object; PrePaint compares its own
    /// records of them: the link offset (a parent's relayout moves a
    /// cache-hit child), the size, the source identity, the live
    /// DisplayItemClients, and the document resources the object paints
    /// with. Index: 0 own bits, 1 offset, 2 size, 3 identity, 4 owners,
    /// 5 resources.
    fn RetainedSelfChange(
        &mut self,
        node: &PaintTreeNode<'_>,
        fragment: PaintFragmentSource<'_>,
    ) -> Option<usize> {
        let Some(cache) = node.cache.as_ref() else {
            return Some(3);
        };
        let f: &FragmentPaintData = &fragment;
        if f.pre_paint_invalidation.own() {
            return Some(0);
        }
        if cache.record.offset != fragment.Cursor().ViewOffsetInContainer() {
            return Some(1);
        }
        if cache.record.size != f.ViewSize() {
            return Some(2);
        }
        if !SamePreviousSourceIdentity(node, f) {
            return Some(3);
        }
        // Recorded client ids name the same live clients; a client the
        // record does not name must not be newly created.
        if !SameClientDirectory(node, f) || HasUnrecordedJustCreatedClient(&cache.record, f) {
            return Some(4);
        }
        let usage = || {
            cache.resource_usage
                | layout::paint::PaintResourceUsage(f)
        };
        if !self.SameDocumentResources(
            cache.resources.as_ref(),
            fragment.Cursor().DocumentResources(),
            usage,
        ) {
            return Some(5);
        }
        None
    }

    /// The first rule by which `node`'s retained descendants cannot stand
    /// for `fragment`'s, or None; RetainedSelfChange must already hold.
    /// Source: pre_paint_tree_walk.cc:418-437 descends for descendant dirty
    /// bits; a display-locked child walk left pending (subtree revision 0)
    /// and the document resources of the whole subtree are the remaining
    /// inputs. Index: 6 descendant bits, 7 pending child walk, 8 resources.
    fn RetainedDescendantChange(
        &mut self,
        node: &PaintTreeNode<'_>,
        fragment: PaintFragmentSource<'_>,
    ) -> Option<usize> {
        let Some(cache) = node.cache.as_ref() else {
            return Some(7);
        };
        if fragment.pre_paint_invalidation.descendants() {
            return Some(6);
        }
        if cache.subtree_revision == 0 {
            return Some(7);
        }
        if !self.SameDocumentResources(
            cache.resources.as_ref(),
            fragment.Cursor().DocumentResources(),
            || cache.subtree_resource_usage,
        ) {
            return Some(8);
        }
        None
    }

    fn RetainedSelfUnchanged(
        &mut self,
        node: &PaintTreeNode<'_>,
        fragment: PaintFragmentSource<'_>,
    ) -> bool {
        self.RetainedSelfChange(node, fragment).is_none()
    }

    fn RetainedDescendantsUnchanged(
        &mut self,
        node: &PaintTreeNode<'_>,
        fragment: PaintFragmentSource<'_>,
    ) -> bool {
        self.RetainedDescendantChange(node, fragment).is_none()
    }

    /// Whether a fragment last proven against `recorded` document paint
    /// resources paints the same under `current`, for its resource `usage`
    /// (evaluated only when the two values are different allocations).
    fn SameDocumentResources(
        &mut self,
        recorded: Option<&Arc<PaintResources>>,
        current: Option<&Arc<PaintResources>>,
        usage: impl FnOnce() -> u8,
    ) -> bool {
        let (recorded, current) = match (recorded, current) {
            (None, None) => return true,
            (Some(a), Some(b)) if Arc::ptr_eq(a, b) => return true,
            (Some(a), Some(b)) => (a, b),
            _ => return false,
        };
        let usage = usage();
        if let Some(&(_, _, _, same)) = self.resource_memo.iter().find(|(a, b, u, _)| {
            *u == usage && Arc::ptr_eq(a, recorded) && Arc::ptr_eq(b, current)
        }) {
            return same;
        }
        let same = layout::paint::SamePaintResourcesFor(
            recorded, current, usage,
        );
        self.resource_memo
            .push((recorded.clone(), current.clone(), usage, same));
        same
    }

    fn UpdateNode(
        &mut self,
        fragment: PaintFragmentSource<'_>,
        mut inherited: InheritedContext,
        parent_invalidation_context: PaintInvalidatorContext,
        mut previous: Option<Box<PaintTreeNode<'static>>>,
        force: bool,
    ) -> Box<PaintTreeNode<'static>> {
        let profile = browser_tracing::enabled();
        let checkpoint = || profile.then(std::time::Instant::now);
        let t_start = checkpoint();
        // Decision D2: an out-of-flow object takes its effect state from its
        // logical parent before admission, so a retained subtree is compared
        // against, and a rebuilt one parented to, the logical effect.
        if let Some(logical) = self.LogicalOutOfFlowEffectContext(&fragment) {
            inherited.properties.nodes.effect = logical.effect;
            inherited.properties.effect_id = logical.effect_id;
            inherited.opacity = logical.opacity;
        }
        // Source pre_paint_tree_walk.cc:270-282,725-730. The source registry
        // invalidates the affected native subtree; this existing walk updates
        // its resident LayoutObject/FragmentData before paint invalidation.
        inherited.blocking_wheel_event_handler_changed |= fragment
            
            .pre_paint_invalidation
            .blocking_wheel_event_handler_changed;
        let has_handler = self
            .active_blocking_wheel_targets
            .binary_search(&fragment.ViewNodeId())
            .is_ok()
            || (fragment.paint.IsLayoutView()
                && self.active_blocking_wheel_targets.binary_search(&0).is_ok());
        let should_update = inherited.blocking_wheel_event_handler_changed
            || (self.source_owner_changed
                && (fragment.paint.LayoutObjectIdentity().is_some()
                    || !self.active_blocking_wheel_targets.is_empty()));
        let inside = inherited.inside_blocking_wheel_event_handler
            || if should_update {
                has_handler
            } else {
                fragment.paint.InsideBlockingWheelEventHandler()
            };
        let mut resident = ResidentFragment::FromSource(&fragment);
        // The payload is shared copy-on-write with the previous publication.
        // Its stored bit and the LayoutObject's are written together (and an
        // export copies the object's), so an equal bit needs no update.
        let blocking_wheel_changed = resident.paint.InsideBlockingWheelEventHandler() != inside
            && resident.paint.UpdateInsideBlockingWheelEventHandler(inside);
        if inherited.blocking_wheel_event_handler_changed || blocking_wheel_changed {
            resident.paint.InvalidatePaintForHitTest();
        }
        if blocking_wheel_changed {
            resident
                
                .pre_paint_invalidation
                .should_check_for_paint_invalidation = true;
        }
        let old_paint_offset = previous.as_ref().map(|node| node.paint_offset);
        let old_had_paint_offset_translation = previous.as_ref().is_some_and(|node| {
            HasOwnPropertyRole(node, PaintPropertyRole::PaintOffsetTranslation)
        });
        let old_self_cull_rect = previous.as_ref().map(|node| node.self_cull_rect);
        let old_contents_cull_rect = previous.as_ref().map(|node| node.cull_rect);
        let old_previous_paint_result = previous
            .as_ref()
            .map_or(paint_model::PaintResult::kMayBeClippedByCullRect, |node| {
                node.previous_paint_result.get()
            });
        let was_filter_origin_tainted = previous.as_deref().is_some_and(OwnFilterOriginTainted);
        let mut invalidation_context = parent_invalidation_context;
        let should_run_invalidation =
            PaintInvalidator::UpdateContext(&fragment, &mut invalidation_context);
        let same_context = !force
            && previous.as_ref().is_some_and(|node| {
                node.cache
                    .as_ref()
                    .is_some_and(|cache| self.SameContext(&cache.inherited, &inherited))
            });
        let retained_context = !force
            && !same_context
            && previous.as_ref().is_some_and(|node| {
                node.cache.as_ref().is_some_and(|cache| {
                    self.SameRetainedContextStructure(&cache.inherited, &inherited)
                })
            });
        // Source: pre_paint_tree_walk.cc:1498-1508 and
        // paint_property_tree_builder.cc:3769-3827. A mutable composited
        // transform (including ScrollTranslation) keeps its property slot and
        // does not dirty clean descendants. CullRectUpdater's sliding window
        // is the coverage proof: descend only when the successor exposes area
        // that the retained subtree did not record. Fixed-background geometry
        // is the exception because its paint position is scroll-relative.
        let retained_geometry_covered = retained_context
            && previous.as_ref().is_some_and(|node| {
                node.cache.as_ref().is_some_and(|cache| {
                    Self::CullContextCovers(&cache.inherited.cull, &inherited.cull)
                        && (!inherited.fixed_background_acts_as_scroll
                            || cache.inherited.scroll == inherited.scroll)
                })
            });
        self.stats.same_context_nodes += same_context as usize;
        self.stats.retained_context_nodes += retained_context as usize;
        self.stats.retained_geometry_covered_nodes += retained_geometry_covered as usize;
        self.stats.invalidation_nodes += should_run_invalidation as usize;
        // PrePaint's own admission (RetainedSelfChange /
        // RetainedDescendantChange): the live dirty bits plus this node's
        // records, never a Layout revision.
        let self_change = previous
            .as_deref()
            .map(|node| self.RetainedSelfChange(node, fragment));
        let self_unchanged = self_change == Some(None);
        let descendant_change = if self_unchanged {
            previous
                .as_deref()
                .and_then(|node| self.RetainedDescendantChange(node, fragment))
        } else {
            None
        };
        let subtree_unchanged = self_unchanged && descendant_change.is_none();
        if let Some(Some(rule)) = self_change {
            self.stats.admission_rejects[rule] += 1;
        }
        if let Some(rule) = descendant_change {
            self.stats.admission_rejects[rule] += 1;
        }
        // A replacement source owner forces this complete walk. Clean-subtree
        // admission is therefore available only while the resident sidecar
        // itself remains the owner of every raw FragmentPaintData pointer.
        let source_tree_compatible = !self.source_owner_changed;
        self.stats.self_unchanged_nodes += self_unchanged as usize;
        self.stats.subtree_unchanged_nodes += subtree_unchanged as usize;
        self.stats.source_tree_compatible_nodes += source_tree_compatible as usize;
        let subtree_clean = !self
            .local_frame_views
            .contains_key(&fragment.paint.DisplayItemClientId())
            && previous
                .as_ref()
                .is_none_or(|node| !node.cache.as_ref().unwrap().contains_local_frames)
            && (same_context || retained_geometry_covered)
            && !should_run_invalidation
            && subtree_unchanged
            // A replacement LayoutResult republishes the same stable sidecar
            // slots; a clean subtree whose source topology is unchanged only
            // needs its two owner references moved (RebindCleanSubtree).
            && (source_tree_compatible
                || previous
                    .as_deref()
                    .is_some_and(|node| CleanSubtreeRebindable(node, fragment)));
        if subtree_clean {
            let mut node = previous.unwrap();
            if !source_tree_compatible {
                self.stats.rebound_nodes += RebindCleanSubtree(&mut node, fragment);
            }
            // Same-slot immutable snapshots need the same reference update that
            // Blink gets from retaining a pointer to its mutable property node.
            // CullRect::ChangedEnough keeps the old finite sliding window while
            // it still covers the successor. Preserve that window in the
            // retained builder context; widening it to an infinite cull rect is
            // not a Chromium CullRectUpdater operation and defeats culling for
            // the complete clean subtree.
            let mut retained_inherited = inherited;
            if retained_context {
                let old_cull = node.cache.as_ref().unwrap().inherited.cull;
                if Self::CullContextCovers(&old_cull, &retained_inherited.cull) {
                    retained_inherited.cull = old_cull;
                }
            }
            node.cache.as_mut().unwrap().inherited = retained_inherited;
            let cache = node.cache.as_ref().unwrap();
            self.property_keys
                .extend(cache.subtree_keys.iter().copied());
            self.stats.reused_nodes += cache.nodes;
            self.stats.skipped_subtrees += 1;
            self.stats.profile_admission_ns += ProfileNs(t_start, checkpoint());
            return node;
        }
        // A replacement LayoutResult republishes every fragment-local sidecar
        // at a new address, but Layout's revisions are keyed by the stable
        // FragmentData identity. An unchanged revision therefore proves the
        // same inputs Chromium would find on its retained FragmentData; only
        // the borrowed sidecar pointer has to move to the new owner.
        // The values this lifecycle compares and then records on the slot
        // (PrePaintPreviousOf), computed once per visited node.
        let current_record =
            PrePaintPreviousOf(&fragment, fragment.Cursor().ViewOffsetInContainer());
        let previous_display_item_client_id =
            previous.as_ref().map(|node| RecordOf(node).display_item_client_id);
        // The previous lifecycle's fragment, as the retained node recorded
        // it, instead of the previous LayoutResult's sidecar copy. Owned
        // here: the rebuild path overwrites the retained node before the
        // invalidators below read it.
        let old_snapshot = previous
            .as_mut()
            .and_then(|node| node.cache.as_mut())
            .and_then(|cache| cache.previous_fragment.take());
        let old_fragment = old_snapshot
            .as_ref()
            .map(|fragment| fragment as *const FragmentPaintData);
        // self_unchanged already proves the source identity and client
        // directory that a replacement owner's rebind requires.
        let self_clean = (same_context || retained_geometry_covered) && self_unchanged;
        let previous_resource_usage = previous
            .as_ref()
            .and_then(|node| node.cache.as_ref())
            .map(|cache| cache.resource_usage);
        // Both the invalidation-only and same-geometry proofs compare the old
        // sidecar's style projection with the new one. After a replacement
        // LayoutResult the old projection may reference a ComputedStyle that
        // has since been released and its address reused, so only the
        // identity-based rebind above may cross owners.
        let invalidation_only = same_context
            && !self.source_owner_changed
            && previous.as_ref().is_some_and(|node| {
                SameClientDirectory(node, &fragment)
                    && CanRefreshInvalidationOnlySource(node, &fragment, &current_record)
            });
        let t_admitted = checkpoint();
        self.stats.profile_admission_ns += ProfileNs(t_start, t_admitted);
        let retained_before = (
            self.stats.reused_nodes,
            self.stats.shifted_nodes,
            self.stats.invalidation_only_nodes,
        );
        // Offset-only update: this object is unchanged and owns no paint
        // property nodes, and its parent only moved it. Shift the retained
        // geometry instead of rebuilding it (PaintPropertyTreeBuilder's
        // UpdateForObjectLocation path updates just PaintOffset).
        let shift_check = self.shift_check;
        let mut rebased_own_states = None;
        let shift_delta = if !self_clean && !invalidation_only && !force && self_unchanged {
            previous.as_ref().and_then(|node| {
                let cache = node.cache.as_ref()?;
                if (self.source_owner_changed
                    && !(SamePreviousSourceIdentity(node, &fragment)
                        && SameClientDirectory(node, &fragment)))
                    || !ShiftKeepsSvgPrefix(node, &cache.inherited, &inherited)
                {
                    return None;
                }
                if (cache.own_keys.is_empty() || RebasableOwnEffect(node, &inherited))
                    && ShiftPreservesClips(node)
                {
                    return self.OffsetOnlyContextDelta(&cache.inherited, &inherited);
                }
                let delta = self.OffsetOnlyContextDelta(&cache.inherited, &inherited)?;
                rebased_own_states = Some(RebaseOwnStates(node, &inherited, delta)?);
                Some(delta)
            })
        } else {
            None
        };
        if profile && shift_delta.is_none() && !self_clean && !invalidation_only {
            // Why this node is rebuilt rather than shifted (trace only).
            let reason = match previous.as_ref().and_then(|node| node.cache.as_ref().map(|c| (node, c))) {
                None => 0,
                _ if force => 0,
                _ if !self_unchanged => 1,
                Some((node, cache))
                    if !(cache.own_keys.is_empty() || RebasableOwnEffect(node, &inherited)) =>
                {
                    2
                }
                Some((node, _))
                    if self.source_owner_changed
                        && !(SamePreviousSourceIdentity(node, &fragment)
                            && SameClientDirectory(node, &fragment)) =>
                {
                    3
                }
                Some((node, _)) if node.is_paint_layer => 4,
                Some((node, cache))
                    if !node.local_transforms.is_empty()
                        || cache.children.svg.is_some()
                        || IsSvgLayoutChild(fragment.paint.SourceKind()) =>
                {
                    5
                }
                Some((node, _)) if !ShiftPreservesClips(node) => 6,
                Some(_) => 7,
            };
            self.stats.shift_rejects[reason] += 1;
        }
        let mut node = if let Some(delta) = shift_delta.filter(|_| !shift_check) {
            let t_shift = checkpoint();
            self.stats.shifted_nodes += 1;
            let mut node = previous.take().unwrap();
            node.BindSource(fragment);
            if ShiftRetainedNode(
                &mut node,
                delta,
                &inherited,
                old_previous_paint_result,
                rebased_own_states.take(),
                &self.viewport_properties.nodes,
            ) {
                self.stats.cull_change_repaints += 1;
                InvalidateClientsForCullRectChange(
                    node.fragment.as_mut().expect("shifted node keeps its fragment"),
                );
            }
            // As UpdateForObjectLocation: a changed paint offset admits paint
            // invalidation for this object.
            if delta.offset != delta.paint_space {
                node.fragment
                    .as_mut()
                    .expect("shifted node keeps its fragment")
                    .pre_paint_invalidation
                    .should_check_for_paint_invalidation = true;
            }
            self.needs_post_update = true;
            // Shifted geometry is a new PrePaint state of this node.
            node.cache.as_mut().unwrap().revision = self.revision_generation;
            self.stats.shift_branch_ns += ProfileNs(t_shift, checkpoint());
            node
        } else if self_clean {
            self.stats.reused_nodes += 1;
            let mut node = previous.take().unwrap();
            if self.source_owner_changed {
                self.stats.rebound_nodes += 1;
                node.BindSource(fragment);
            }
            node
        } else if invalidation_only {
            // Blink visited this object to run paint invalidation, while
            // retaining its FragmentData property slots. Refresh our shallow
            // source and revisions without constructing a speculative second
            // PaintTreeNode solely to rediscover the same property geometry.
            self.stats.invalidation_only_nodes += 1;
            let mut node = previous.take().unwrap();
            node.BindSource(fragment);
            let cache = node.cache.as_mut().unwrap();
            cache.inherited = inherited.clone();
            cache.revision = self.revision_generation;
            node
        } else {
            let mut built = PaintPropertyTreeBuilder::new(self)
                .with_previous_paint_result(old_previous_paint_result)
                .with_link_offset(fragment.Cursor().ViewOffsetInContainer())
                .with_source_cursor(fragment.Cursor())
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
                    old_had_paint_offset_translation,
                    old_self_cull_rect,
                    old_contents_cull_rect,
                    inherited.root,
                    inherited.flex_or_grid_item,
                );
            let cull_change_needs_repaint = CullRectChangeNeedsRepaint(
                old_previous_paint_result,
                old_self_cull_rect,
                old_contents_cull_rect,
                built.self_cull_rect,
                built.cull_rect,
            );
            if cull_change_needs_repaint {
                self.stats.cull_change_repaints += 1;
                InvalidateClientsForCullRectChange(
                    &mut built
                        .fragment
                        .as_mut()
                        .expect("PrePaint node must own a fragment"),
                );
            }
            built.previous_paint_result.set(old_previous_paint_result);
            if let Some(mut old) = previous.take() {
                if let Some(delta) = shift_delta.filter(|_| shift_check) {
                    // Verification: the shifted retained node must equal the
                    // node the builder just produced. Restore it afterwards.
                    let saved = SavedShiftFields::Save(&old);
                    ShiftRetainedNode(
                        &mut old,
                        delta,
                        &inherited,
                        old_previous_paint_result,
                        rebased_own_states.take(),
                        &self.viewport_properties.nodes,
                    );
                    self.stats.shifted_nodes += 1;
                    let field_mismatch = ShiftFieldMismatch(&old, &built);
                    // Compare against the built node as the parent completes
                    // it: a leading SVG container matrix (already verified by
                    // ShiftFieldMismatch) is inserted after it returns.
                    let built_local = (built.local_transforms.clone(), built.applies_transform);
                    if field_mismatch.is_none() {
                        built.local_transforms = old.local_transforms.clone();
                        built.applies_transform = old.applies_transform;
                    }
                    let same_geometry = self.SameGeometry(&old, &built, &current_record);
                    (built.local_transforms, built.applies_transform) = built_local;
                    let mismatch = if let Some(field) = field_mismatch {
                        Some(field)
                    } else if !same_geometry {
                        let bf = built.fragment.as_deref().unwrap();
                        let recorded = RecordOf(&old);
                        Some(if IsNewClient(
                            &RecordedClientIds(&recorded),
                            bf.paint.DisplayItemClientId(),
                            bf.paint.DisplayItemClientState().is_just_created,
                        ) {
                            "SG.just_created"
                        } else if !SamePreviousSourceIdentity(&old, bf) {
                            "SG.identity"
                        } else if !SamePreviousPhaseInputs(&recorded, &current_record) {
                            "SG.phase_inputs"
                        } else if !SamePreviousLogicalPosition(&recorded, &current_record) {
                            "SG.logical_position"
                        } else if old.opacity != built.opacity {
                            "SG.opacity"
                        } else if !state_compare::SameBuilderState(
                            &old.properties.nodes,
                            &built.properties.nodes,
                        ) {
                            "SG.properties"
                        } else if !state_compare::SameBuilderState(
                            &old.contents_properties.nodes,
                            &built.contents_properties.nodes,
                        ) {
                            "SG.contents_properties"
                        } else {
                            "SG.other_field"
                        })
                    } else {
                        ShiftChildrenMismatch(&old, &built)
                    };
                    if mismatch.is_some() {
                        browser_tracing::instant(
                            "prepaint",
                            "PrePaint.ShiftMismatchValues",
                            &[
                                ("shift_abs_x", old.absolute_offset.x),
                                ("built_abs_x", built.absolute_offset.x),
                                ("shift_po_x", old.paint_offset.x),
                                ("built_po_x", built.paint_offset.x),
                                ("delta_x", delta.offset.x),
                            ],
                        );
                    }
                    if let Some(field) = mismatch {
                        self.stats.shift_check_mismatches += 1;
                        let name = built
                            .fragment
                            .as_deref()
                            .map_or_else(
                                || Arc::<str>::from("?"),
                                |f| f.paint.DisplayItemClientDebugName(),
                            );
                        browser_tracing::instant_with_text(
                            "prepaint",
                            "PrePaint.ShiftMismatch",
                            &format!("{name}: {field}"),
                            &[],
                        );
                    }
                    saved.Restore(&mut old);
                }
                let same_geometry =
                    !force
                        && !self.source_owner_changed
                        && self.SameGeometry(&old, &built, &current_record);
                if same_geometry {
                    self.stats.same_geometry_rebuilds += 1;
                    // Bit 2 of the recorded phase flags is the previous
                    // focused image-map area (PrePaintPreviousOf).
                    let focused_image_map_phase_changed =
                        built.fragment.as_deref().is_some_and(|fragment| {
                            (RecordOf(&old).phase_flags & (1 << 2) != 0)
                                != fragment.paint.FocusedImageMapArea().is_some()
                        });
                    self.needs_post_update |= focused_image_map_phase_changed;
                    // Only drawing content changed. Keep the already-finalized
                    // properties and group/phase results; clean descendants do
                    // not need a canonicalization or logical-tree pass.
                    built.properties = old.properties.clone();
                    built.contents_properties = old.contents_properties.clone();
                    built.fixed_background_properties = old.fixed_background_properties.clone();
                    built.horizontal_scrollbar_properties =
                        old.horizontal_scrollbar_properties.clone();
                    built.vertical_scrollbar_properties = old.vertical_scrollbar_properties.clone();
                    built.mask_properties = old.mask_properties.clone();
                    built.opacity = old.opacity;
                    built.outline_rects = std::mem::take(&mut old.outline_rects);
                    built.paints_outline = old.paints_outline;
                    built.needs_float_phase = old.needs_float_phase;
                    built.needs_outline_phase = old.needs_outline_phase;
                    built.needs_overflow_controls_phase = old.needs_overflow_controls_phase;
                    let old_cache = old.cache.as_ref().unwrap();
                    let new_cache = built.cache.as_mut().unwrap();
                    // Keep the child context computed from this lifecycle
                    // pass. Stable property slots make old/new builder states
                    // equivalent even when a mutable ScrollTranslation value
                    // changed, but the flat transform and cull vectors used by
                    // Paint's MapRectToRoot must carry that new value. Reuse
                    // only the finalized property snapshot inside the context.
                    new_cache.children.properties = old_cache.children.properties.clone();
                    new_cache.own_keys = old_cache.own_keys.clone();
                    // The completed node snapshots were retained above. If a
                    // sibling forces the post pass, restore this node from its
                    // raw builder cache just like any other retained node.
                    new_cache.computed_pass = old_cache.computed_pass;
                } else {
                    self.stats.property_geometry_changes += 1;
                    self.stats.property_geometry_changes_with_own_keys +=
                        (!old.cache.as_ref().unwrap().own_keys.is_empty()) as usize;
                    self.stats.property_geometry_changes_needing_property_update += fragment
                        
                        .pre_paint_invalidation
                        .needs_paint_property_update
                        as usize;
                    self.needs_post_update = true;
                }
                built.self_or_descendant_needs_repaint |= cull_change_needs_repaint;
                built.children = std::mem::take(&mut old.children);
                built.physical_index = old.physical_index;
                // Source retains FragmentData/property roots across a shallow
                // update (paint_property_tree_builder.cc:4519-4532). The
                // directory points to these same stable boxes; move its owner
                // before replacing the root payload. A topology change still
                // rebuilds it during the existing final-effects walk below.
                built.svg_resource_nodes = old.svg_resource_nodes.take();
                // Update dirty ancestors in place, preserving every Box address
                // referenced by retained physical/logical ancestry.
                *old = built;
                old
            } else {
                self.needs_post_update = true;
                Box::new(built)
            }
        };
        let t_built = checkpoint();
        // Source: cull_rect_updater.cc:67-89 (SetFragmentContentsCullRect,
        // reached from UpdateForSelf at :393-431) and
        // paint_layer_scrollable_area.cc:1919-1920 (DidUpdateCullRect). When
        // this update produced a new contents cull rect, record on the
        // PaintLayerScrollableArea the scroll position it was computed for;
        // HasScrolledEnough (cull_rect_updater.cc:193-206) measures from it.
        if old_contents_cull_rect.is_none_or(|old| old != node.cull_rect) {
            fragment
                .paint
                .SetLastCullRectUpdateScrollPosition(fragment.paint.ScrollOffset());
        }
        self.stats.profile_build_ns += ProfileNs(t_admitted, t_built);
        if profile
            && retained_before
                == (
                    self.stats.reused_nodes,
                    self.stats.shifted_nodes,
                    self.stats.invalidation_only_nodes,
                )
        {
            let kind = BuildKind(&node, &fragment);
            self.stats.build_kind_ns[kind] += ProfileNs(t_admitted, t_built);
            self.stats.build_kind_nodes[kind] += 1;
        }
        // LayoutImage::last_paint_rect_ belongs to the retained LayoutImage,
        // not to one immutable PhysicalFragment generation. Rechrom keeps the
        // same lifecycle value on the fragment's shared Paint payload, so
        // carry it across a rebuilt fragment only when the source
        // DisplayItemClient identity is unchanged.
        // cpp: core/layout/layout_image.h:202-203
        // cpp: core/layout/layout_image.cc:411-421
        RetainImagePaintedRect(
            node.fragment
                .as_deref()
                .expect("PrePaint node must own a fragment"),
            PreviousFragment(old_fragment),
            previous_display_item_client_id,
        );
        {
            // This node is now proven (RetainedSelfChange) or rebuilt
            // against the current document resources. Its usage follows its
            // own inputs, so an unchanged node keeps the recorded one.
            let resources = fragment.Cursor().DocumentResources().cloned();
            let usage = match previous_resource_usage {
                Some(usage) if self_unchanged => usage,
                _ => layout::paint::PaintResourceUsage(
                    &fragment,
                ),
            };
            let cache = node.cache.as_mut().unwrap();
            cache.inherited = inherited.clone();
            cache.resources = resources;
            cache.resource_usage = usage;
        }
        if let Some(context) =
            LogicalEffectContextOf(&node, self.fragmentainer_scope.unwrap_or(0))
        {
            let entries = self
                .logical_effect_contexts
                .entry(fragment.ViewNodeId())
                .or_default();
            match entries
                .iter_mut()
                .find(|entry| entry.fragment_instance_id == context.fragment_instance_id)
            {
                Some(entry) => *entry = context,
                None => entries.push(context),
            }
        }
        // Chromium runs invalidation after property construction. The initial
        // admission above is only a clean-subtree proof: UpdateForObjectLocation
        // may have dirtied a previously clean object through an ancestor move.
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
        node.table_cell_geometry_changed = fragment.paint.Display() == Display::kTableCell
            && should_run_invalidation
            && TableCellPaintInvalidator::CellGeometryChanged(
                &fragment,
                PreviousFragment(old_fragment),
                old_paint_offset,
                node.paint_offset,
            );
        if should_run_invalidation {
            let background_invalidation = (fragment.ViewKind() == FragmentKind::kBox).then(|| {
                let mut background = BoxPaintInvalidator::ComputeBackgroundInvalidation(
                    &fragment,
                    Some(fragment.Cursor()),
                    PreviousFragment(old_fragment),
                    invalidation_context,
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
                node.paint_offset,
                invalidation_context,
            );
            // Source: object_paint_invalidator.cc:49-53. The reason above read
            // the previous visibility from the slot snapshot; save the current
            // one on the LayoutObject for the next walk.
            fragment.paint.UpdateSourcePreviousVisibilityVisible();
            let reason = if fragment.ViewKind() == FragmentKind::kBox {
                if fragment.paint.Display() == Display::kTableCell {
                    TableCellPaintInvalidator::ComputePaintInvalidationReason(
                        &fragment,
                        PreviousFragment(old_fragment),
                        old_paint_offset,
                        node.paint_offset,
                        invalidation_context,
                        reason,
                    )
                } else if fragment.paint.IsLayoutBlock() {
                    BlockPaintInvalidator::ComputePaintInvalidationReason(
                        &fragment,
                        PreviousFragment(old_fragment),
                        old_paint_offset,
                        node.paint_offset,
                        invalidation_context,
                        reason,
                    )
                } else {
                    BoxPaintInvalidator::ComputePaintInvalidationReason(
                        &fragment,
                        PreviousFragment(old_fragment),
                        old_paint_offset,
                        node.paint_offset,
                        invalidation_context,
                        reason,
                    )
                }
            } else {
                reason
            };
            let resident_fragment = &mut **node
                .fragment
                .as_mut()
                .expect("PrePaint node must own a fragment");
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
            // requires the next scroll/visibility observation, even when the
            // geometry did not change.
            if client_reason != foundation::PaintInvalidationReason::kNone {
                self.intersection_observation_state=self.intersection_observation_state.max(foundation::intersection_observation_state::IntersectionObservationState::ScrollAndVisibilityOnly);
            }
            if invalidation.should_check_layout_for_paint_invalidation
                && (foundation::IsLayoutPaintInvalidationReason(client_reason)
                    || client_reason == foundation::PaintInvalidationReason::kJustCreated
                    || !resident_fragment.paint.style.visible())
            {
                self.intersection_observation_state=foundation::intersection_observation_state::IntersectionObservationState::Desired;
            }
        }
        let t_invalidated = checkpoint();
        self.stats.profile_invalidation_ns += ProfileNs(t_built, t_invalidated);
        // Same opaque retained lifetime as ResidentFragment; the caller keeps
        // this result's PaintFragmentLease until replacement has been rebound.
        node.source_cursor = Some(unsafe {
            std::mem::transmute::<PaintFragmentCursor<'_>, PaintFragmentCursor<'static>>(
                fragment.Cursor(),
            )
        });
        let mut base = node.cache.as_ref().unwrap().children.clone();
        base.inside_blocking_wheel_event_handler = inside;
        base.blocking_wheel_event_handler_changed = inherited.blocking_wheel_event_handler_changed;
        node.cache
            .as_mut()
            .unwrap()
            .children
            .inside_blocking_wheel_event_handler = inside;
        node.cache
            .as_mut()
            .unwrap()
            .children
            .blocking_wheel_event_handler_changed = inherited.blocking_wheel_event_handler_changed;
        let own_keys = node.cache.as_ref().unwrap().own_keys.clone();
        self.property_keys.extend(own_keys.iter().copied());
        if inherited.root {
            self.CaptureRootFixedPositionContext(&base, &node);
        }
        // This node's comparisons are complete: record the values the next
        // lifecycle compares against (RetainedSelfChange reads them), also
        // for a display-locked node whose children are retained below.
        // Children record theirs in their own UpdateNode, after the identity
        // check below has read them.
        let snapshot = node.fragment.as_deref().map(PreviousFragmentSnapshot);
        if let Some(cache) = node.cache.as_mut() {
            cache.record = current_record;
            cache.previous_fragment = snapshot;
        }
        // Source: core/paint/pre_paint_tree_walk.cc:1517-1535.
        // Display locking blocks only the child walk. WalkInternal has already
        // updated this object, so retain its resident children exactly as they
        // were and keep their property slots live. A zero subtree revision is
        // the local equivalent of DisplayLockContext::SetNeedsPrePaintSubtreeWalk:
        // it prevents an ancestor clean-subtree fast path from hiding the work
        // after the lock opens.
        let child_pre_paint_blocked_by_display_lock = fragment
            
            .pre_paint_invalidation
            .child_pre_paint_blocked_by_display_lock;
        if CanRetainDisplayLockedChildren(
            child_pre_paint_blocked_by_display_lock,
            self.source_owner_changed,
        ) {
            let mut keys = own_keys;
            let mut nodes = 1;
            let mut subtree_resource_usage = node.cache.as_ref().unwrap().resource_usage;
            for child in &node.children {
                let child_cache = child
                    .cache
                    .as_ref()
                    .expect("resident child has builder state");
                keys.extend(child_cache.subtree_keys.iter().copied());
                nodes += child_cache.nodes;
                subtree_resource_usage |= child_cache.subtree_resource_usage;
            }
            self.property_keys.extend(keys.iter().copied());
            self.stats.reused_nodes += nodes.saturating_sub(1);
            self.stats.skipped_subtrees += usize::from(nodes > 1);
            let physical_children = node
                .children
                .iter_mut()
                .map(|child| &mut **child as *mut _)
                .collect();
            let cache = node.cache.as_mut().unwrap();
            cache.subtree_revision = 0;
            cache.subtree_resource_usage = subtree_resource_usage;
            cache.subtree_keys = keys;
            cache.nodes = nodes;
            cache.physical_children = physical_children;
            self.stats.profile_children_setup_ns += ProfileNs(t_invalidated, checkpoint());
            // Source: pre_paint_tree_walk.cc:1517-1531,1584-1585. The child
            // walk is blocked, but this object was visited; ClearPaintFlags
            // keeps the descendant bits of a display-locked object.
            fragment.paint.ClearSourcePaintFlags();
            return node;
        }
        // Chromium's locked subtree keeps pointers to resident LayoutObjects.
        // Rechrom's equivalent references a result-local immutable sidecar. If
        // Layout published an incompatible replacement tree while the lock was
        // active, retaining the old children would require retaining both
        // leases. Walk the replacement children instead so RetainTree can
        // safely release the preceding lease; property/invalidation work is
        // still skipped by the source's display-lock dirty-bit projection.
        let embedded_view = self
            .local_frame_views
            .get(&fragment.paint.DisplayItemClientId())
            .filter(|_| fragment.ViewCanPaintFragmentOccurrence())
            .cloned();
        let embedded_source = embedded_view
            .as_ref()
            .map(|view| PaintFragmentSource::FromCursor(view.root));
        let old_children = std::mem::take(&mut node.children);
        // Source: paint_property_tree_builder.cc:2728-2730. A changed filter
        // taint alters descendant Effect state even across paint-property
        // isolation. Reuse the existing piercing `force` channel for the
        // descendants; the current node has already been rebuilt above.
        let force_descendants = force || FilterTaintChanged(was_filter_origin_tainted, &node);
        let same_children = old_children.len()
            == OrdinaryPaintSourceChildren(fragment).count()
                + usize::from(embedded_source.is_some())
            && old_children
                .iter()
                .zip(OrdinaryPaintSourceChildren(fragment).chain(embedded_source))
                .all(|(old, source)| SamePreviousSourceIdentity(old, &source));
        if !same_children {
            self.needs_post_update = true;
        }
        // Chromium keeps PhysicalFragment/FragmentData identity across cull
        // rect updates.  A newly exposed fragment may be inserted before the
        // retained range, so child position is not object identity.  Preserve
        // the allocation fast path when the sequence is unchanged; otherwise
        // move resident nodes by the occurrence identity read live from the
        // resident fragment (ResidentBoxGeometry::ViewFragmentInstanceId).
        // Hand-built trees use id zero and deliberately retain the positional
        // behavior only while their topology is unchanged.
        let (mut positional_children, mut children_by_instance) = if same_children {
            (Some(old_children.into_iter()), HashMap::default())
        } else {
            let mut children_by_instance = HashMap::default();
            children_by_instance.reserve(old_children.len());
            for child in old_children {
                // The instance id recorded when this child was last visited.
                let instance_id = RecordOf(&child).fragment_instance_id;
                if instance_id != 0 {
                    let replaced = children_by_instance.insert(instance_id, child);
                    debug_assert!(
                        replaced.is_none(),
                        "FragmentTree contains duplicate fragment_instance_id values"
                    );
                }
            }
            (None, children_by_instance)
        };
        let mut keys = own_keys;
        let mut nodes = 1;
        // This node's ScopedDisplayItemFragment, set by the caller before
        // this visit; no child has been visited yet.
        let display_item_fragment = self.display_item_fragment_scope.unwrap_or(0);
        // Likewise the fragmentainer this node is inside (PrePaintTreeWalk's
        // `fragmentainer_idx` context).
        let fragmentainer = self.fragmentainer_scope.unwrap_or(0);
        let ordinary_children = OrdinaryPaintSourceChildren(fragment).count();
        let mut contains_local_frames = embedded_source.is_some();
        let t_loop = checkpoint();
        self.stats.profile_children_setup_ns += ProfileNs(t_invalidated, t_loop);
        let mut recursion_ns = 0;
        self.physical_ancestor_ids.push(if fragment.ViewEstablishesPaintState() {
            fragment.ViewNodeId()
        } else {
            0
        });
        for (index, child) in OrdinaryPaintSourceChildren(fragment)
            .chain(embedded_source)
            .enumerate()
        {
            let previous = if let Some(children) = positional_children.as_mut() {
                children.next()
            } else {
                // The child's live occurrence identity (FragmentData /
                // (client, FragmentId) / fragmentainer), as recorded above.
                let instance_id = child.ViewFragmentInstanceId();
                if instance_id != 0 {
                    children_by_instance.remove(&instance_id)
                } else {
                    None
                }
            };
            let is_local_frame = index == ordinary_children && embedded_view.is_some();
            let child_context = if is_local_frame {
                self.ContextForFrame(&node, &base, embedded_view.as_ref().unwrap())
            } else {
                self.ContextForChild(&child, &base)
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
            // A child paints in this node's scope unless it opens its own; an
            // embedded frame's root starts its document's walk at 0.
            let child_display_item_fragment = child
                .ViewOwnDisplayItemFragment()
                .unwrap_or(if is_local_frame { 0 } else { display_item_fragment });
            self.display_item_fragment_scope = Some(child_display_item_fragment);
            // A column or page area opens its own fragmentainer; everything
            // else is inside this node's. An embedded frame starts outside any.
            let child_fragmentainer = child
                .ViewOwnFragmentainerInstanceId()
                .unwrap_or(if is_local_frame { 0 } else { fragmentainer });
            self.fragmentainer_scope = Some(child_fragmentainer);
            let t_child = checkpoint();
            let mut child_node = self.UpdateNode(
                child,
                child_context,
                invalidation_context,
                previous,
                force_descendants,
            );
            recursion_ns += ProfileNs(t_child, checkpoint());
            self.local_frame_depth -= u32::from(is_local_frame);
            if let Some(previous) = previous_wheel_targets {
                self.active_blocking_wheel_targets = previous;
            }
            child_node.is_embedded_local_frame_root = is_local_frame;
            child_node.embedded_frame_is_cross_origin =
                is_local_frame && embedded_view.as_ref().unwrap().cross_origin_to_parent;
            if is_local_frame {
                self.completed_local_frame_prepaint
                    .insert(fragment.paint.DisplayItemClientId());
            }
            if let Some(fixed) = frame_fixed_context {
                self.root_fixed_position = fixed;
            }
            contains_local_frames |= child_node.cache.as_ref().unwrap().contains_local_frames;
            if child_node.table_cell_geometry_changed {
                node.table_cell_geometry_changed =
                    TableCellPaintInvalidator::InvalidateAncestorAndPropagate(
                        &mut **node
                            .fragment
                            .as_mut()
                            .expect("PrePaint node must own a fragment"),
                    );
            }
            if let Some(svg) = base.svg {
                // A reused node already contains this leading geometry scope.
                // A newly built node needs it once, never once per paint pass.
                if child_node
                    .cache
                    .as_ref()
                    .is_some_and(|cache| cache.computed_pass == self.walk_generation)
                {
                    child_node.local_transforms.insert(0, svg);
                    child_node.applies_transform = true;
                }
            }
            child_node.physical_index = index;
            let scope_changed = child_node
                .display_item_fragment
                .is_some_and(|old| old != child_display_item_fragment);
            child_node.display_item_fragment = Some(child_display_item_fragment);
            let fragmentainer_changed = child_node
                .fragmentainer_instance_id
                .is_some_and(|old| old != child_fragmentainer);
            child_node.fragmentainer_instance_id = Some(child_fragmentainer);
            if scope_changed || fragmentainer_changed {
                RefreshInheritedDisplayItemFragments(&mut child_node);
            }
            let cache = child_node.cache.as_ref().unwrap();
            keys.extend(cache.subtree_keys.iter().copied());
            nodes += cache.nodes;
            node.children.push(child_node);
        }
        self.physical_ancestor_ids.pop();
        let t_looped = checkpoint();
        self.stats.profile_child_loop_ns +=
            ProfileNs(t_loop, t_looped).saturating_sub(recursion_ns);
        let physical_children = node
            .children
            .iter_mut()
            .map(|child| &mut **child as *mut _)
            .collect();
        let generation = self.revision_generation;
        let mut child_walk_pending = false;
        let mut child_changed = false;
        let mut subtree_resource_usage = node.cache.as_ref().unwrap().resource_usage;
        for child in &node.children {
            let cache = child.cache.as_ref().expect("resident child has builder state");
            child_walk_pending |= cache.subtree_revision == 0;
            // Generations are process-unique: a child subtree that changed
            // in this pass carries exactly this pass's generation.
            child_changed |= cache.subtree_revision == generation;
            subtree_resource_usage |= cache.subtree_resource_usage;
        }
        let cache = node.cache.as_mut().unwrap();
        // PrePaint's subtree revision: kept while this node and every child
        // subtree were kept and the child list is the same; otherwise this
        // pass's generation. Zero keeps a pending child walk visible to the
        // ancestors' admission.
        cache.subtree_revision = if child_walk_pending {
            0
        } else if same_children
            && !child_changed
            && cache.revision != generation
            && cache.subtree_revision != 0
        {
            cache.subtree_revision
        } else {
            generation
        };
        cache.subtree_resource_usage = subtree_resource_usage;
        cache.subtree_keys = keys;
        cache.nodes = nodes;
        cache.physical_children = physical_children;
        cache.contains_local_frames = contains_local_frames;
        self.stats.profile_tail_ns += ProfileNs(t_looped, checkpoint());
        // Source: pre_paint_tree_walk.cc:1584-1585. Clear the flags this
        // visit consumed after the children were walked. PrePaint read them
        // from the slot's snapshot, so a second walk in the same lifecycle
        // still sees this publication's inputs; LayoutObject bits now mean
        // "changed since the last PrePaint", as in Chromium.
        fragment.paint.ClearSourcePaintFlags();
        node
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

    // Source: pre_paint_tree_walk.cc:1538-1575 and
    // PaintPropertyTreeBuilder::SetupContextForFrame, source.cc:192-226.
    fn ContextForFrame(
        &self,
        owner: &PaintTreeNode<'_>,
        base: &ChildContext,
        view: &EmbeddedLocalFrameView,
    ) -> InheritedContext {
        let replaced = owner
            .fragment
            .as_deref()
            .unwrap()
            .ViewReplacedContent()
            .unwrap();
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
                x: (owner.paint_offset.x + replaced.offset.x).round() + base.paint_space.x,
                y: (owner.paint_offset.y + replaced.offset.y).round() + base.paint_space.y,
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

    fn CaptureRootFixedPositionContext(&mut self, base: &ChildContext, root: &PaintTreeNode<'_>) {
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
    /// object whose logical parent occurrence (ResolveLogicalParentOccurrence)
    /// is not a physical ancestor takes that occurrence's recorded effect
    /// state instead of its physical parent's.
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
        let candidates = self.logical_effect_contexts.get(&logical_id)?;
        let own = fragment.ViewFragmentInstanceId();
        let fragmentainer = self.fragmentainer_scope.unwrap_or(0);
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
    pub fn Walk(
        &mut self,
        root: PaintFragmentSource<'_>,
        source_lease: PaintFragmentLease,
    ) -> Box<PaintTreeNode<'static>> {
        self.active_blocking_wheel_targets = self.root_blocking_wheel_targets.clone();
        let source_owner_changed = self
            .retained_source_lease
            .as_ref()
            .is_none_or(|retained| !retained.PtrEq(&source_lease));
        let source_owner_changed = source_owner_changed || self.local_frame_sources_changed;
        self.source_owner_changed = source_owner_changed;
        self.pending_source_lease = Some(source_lease);
        self.completed_local_frame_prepaint.clear();
        self.paint_invalidator.BeginPass();
        if source_owner_changed {
            // Sliced inline strips depend on every fragment of the object;
            // resolve them before any node reads them.
            crate::inline_stitched_decoration::PopulateInlineStitchedDecorations(root.Cursor());
        }
        self.requires_main_thread_for_background_attachment_fixed = root
            .paint
            .RequiresMainThreadScrollingForBackgroundAttachmentFixed();
        // Chromium clears changed bits on the persistent Paint property-node
        // identities after PaintArtifactCompositor consumes the artifact.
        // The compositor adapter does the same through ClearChangedToRoot().
        // Do not clone and rebind the resident graph here: that would preserve
        // a second snapshot-local marker model which Chromium does not have.
        self.stats = PrePaintUpdateStats::default();
        self.transform_world_memo.clear();
        self.intersection_observation_state = Default::default();
        self.walk_generation = self
            .walk_generation
            .checked_add(1)
            .expect("PrePaint pass exhausted");
        // Process-unique, so no retained node of any walk already holds it.
        static PRE_PAINT_REVISION: std::sync::atomic::AtomicU64 =
            std::sync::atomic::AtomicU64::new(0);
        self.revision_generation =
            PRE_PAINT_REVISION.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        self.resource_memo.clear();
        // Every slot this pass builds or retains registers its key again; the
        // builders update the resident slot nodes in place.
        self.property_keys.clear();
        self.own_key_log.clear();
        self.own_change_log.clear();
        self.property_node_store.BeginUpdate();
        let viewport = ArtifactViewport(root.Cursor().DocumentResources().map(|resources| &**resources));
        let environment_changed = viewport != self.viewport;
        let viewport_clip_key = PaintPropertyKey {
            owner: PaintPropertyOwner::Viewport,
            role: PaintPropertyRole::ViewportClip,
        };
        if environment_changed || (viewport.is_some() && self.viewport_properties.clip_id == 0) {
            self.viewport = viewport;
            // Blink updates the viewport-owned clip node in place. Its value
            // changes as the window resizes, but the property slot remains the
            // same node so descendants do not acquire a new clip ancestry on
            // every source resize sample.
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
        // The root ScrollTranslation already lives in the descendant chain,
        // so no scroll offset is added to this initial window.
        let visible_root_cull = viewport;
        if self.viewport_properties.clip_id != 0 {
            self.property_keys
                .insert(self.viewport_properties.clip_id, viewport_clip_key);
        }
        // Source: PrePaintTreeWalk::WalkTree and
        // VisualViewportPaintPropertyTreeBuilder::Update. Chromium updates
        // the visual viewport before walking the LayoutView, then starts the
        // current, absolute and fixed transform contexts at the visual
        // viewport scroll translation. Keep the same resident nodes in the
        // inherited state so LayoutView's paint-offset/scroll translations
        // and every page-content chunk are descendants of overscroll and page
        // scale. Publishing the nodes as compositor viewport metadata alone
        // does not establish that ancestry.
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
        let properties = self.viewport_properties.clone();
        let context = InheritedContext {
            inside_blocking_wheel_event_handler: false,
            blocking_wheel_event_handler_changed: false,
            offset: Offset::default(),
            paint_space: Offset::default(),
            clip: viewport,
            // Blink's CullRect::ChangedEnough keeps an expanded sliding
            // window until the visible viewport approaches its edge. This
            // lets incremental resize update the exact viewport clip without
            // repainting clean descendants which already have coverage.
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
        let mut previous = self.retained_tree.take();
        self.view_background_geometry_changed = BoxPaintInvalidator::ViewBackgroundGeometryChanged(
            &root,
            previous.as_ref().and_then(|node| node.fragment.as_deref()),
        );
        // A viewport value change dirties the viewport clip and LayoutView,
        // not every descendant, while the same LayoutResult sidecar remains
        // resident. A replacement owner is different: retained nodes contain
        // raw references into the preceding sidecar, so Chromium's in-place
        // FragmentData update cannot be emulated by identity-only rebinding.
        // Force one linear old/new walk before releasing that preceding lease.
        let force = RequiresFullSourceWalk(source_owner_changed, self.retained_tree_valid);
        // Source: pre_paint_tree_walk.cc:418-437. With no dirty bit on the
        // LayoutView and none below it, no object requires PrePaint. This is
        // UpdateNode's clean-subtree admission applied to the root.
        let entirely_clean = self.local_frame_views.is_empty()
            && self.retained_local_frame_leases.is_empty()
            && !force
            && previous.as_deref().is_some_and(|node| {
                node.cache
                    .as_ref()
                    .is_some_and(|cache| self.SameContext(&cache.inherited, &context))
                    && self.RetainedSelfUnchanged(node, root)
                    && self.RetainedDescendantsUnchanged(node, root)
            });
        if entirely_clean {
            let tree = previous.unwrap();
            let cache = tree.cache.as_ref().unwrap();
            self.stats.reused_nodes = cache.nodes;
            self.stats.skipped_subtrees = 1;
            self.stats.self_unchanged_nodes = 1;
            self.stats.subtree_unchanged_nodes = 1;
            self.property_keys
                .extend(cache.subtree_keys.iter().copied());
            self.resource_memo.clear();
            self.next_property_id = self.property_node_store.next_id();
            return tree;
        }
        // Chromium's FragmentData outlives a relayout, so PrePaint visits only
        // dirty objects. When the sole reason for the forced walk is a
        // replacement sidecar owner, keep the per-node revision proofs and
        // rebind clean nodes in one linear walk instead of rebuilding every
        // property node; subtree skipping stays disabled so no node keeps a
        // pointer into the released owner.
        let rebind = force
            && source_owner_changed
            && self.retained_tree_valid
            && previous.is_some();
        self.stats.rebind_walk = rebind;
        let force = force && !rebind;
        self.needs_post_update = force || previous.is_none() || self.logical_tree_changed;
        if self.logical_tree_changed {
            if let Some(tree) = &mut previous {
                RestorePhysicalOwnership(tree);
            }
        }
        // LayoutView's own scope (its layer fragment index); a publication
        // walk starts at 0.
        let root_display_item_fragment = root.ViewOwnDisplayItemFragment().unwrap_or(0);
        self.display_item_fragment_scope = Some(root_display_item_fragment);
        // The document root is inside no fragmentainer unless it is one.
        let root_fragmentainer = root.ViewOwnFragmentainerInstanceId().unwrap_or(0);
        self.fragmentainer_scope = Some(root_fragmentainer);
        // This walk visits objects and clears their paint flags.
        self.consuming_walks = self.consuming_walks.wrapping_add(1);
        let mut tree = self.UpdateNode(
            root,
            context,
            PaintInvalidatorContext::default(),
            previous,
            force,
        );
        // Source: pre_paint_tree_walk.cc:106-107. Each visited object's flags
        // were cleared in its UpdateNode; schedule the delayed targets for
        // the next pass.
        self.paint_invalidator
            .ProcessPendingDelayedPaintInvalidations();
        self.display_item_fragment_scope = None;
        self.fragmentainer_scope = None;
        let scope_changed = tree
            .display_item_fragment
            .is_some_and(|old| old != root_display_item_fragment)
            || tree
                .fragmentainer_instance_id
                .is_some_and(|old| old != root_fragmentainer);
        tree.display_item_fragment = Some(root_display_item_fragment);
        tree.fragmentainer_instance_id = Some(root_fragmentainer);
        if scope_changed {
            RefreshInheritedDisplayItemFragments(&mut tree);
        }
        if self.needs_post_update {
            self.stats.post_walk = true;
            ResetLogicalEscapes(&mut tree);
            {
                let mut trace = browser_tracing::span("prepaint", "PrePaint.RestoreLogicalTree");
                let (changed, fixups) =
                    RestoreLogicalOutOfFlowParents(&mut tree, &mut self.property_node_store);
                self.logical_tree_changed = changed;
                trace.set("logical_effect_fixups", fixups as f64);
            }
            let mut svg_resources = HashMap::default();
            {
                let _trace = browser_tracing::span("prepaint", "PrePaint.GatherFinalEffects");
                self.logical_effect_contexts.clear();
                GatherFinalEffects(&tree, &mut self.logical_effect_contexts, &mut svg_resources);
                tree.svg_resource_nodes = Some(Box::new(svg_resources));
            }
            {
                let _trace = browser_tracing::span("prepaint", "PrePaint.ResetPostPaintData");
                ResetPostPaintData(&mut tree);
            }
            {
                let _trace = browser_tracing::span("prepaint", "PrePaint.PrepareOutlines");
                PrepareOutlines(&mut tree);
            }
            {
                let _trace = browser_tracing::span("prepaint", "PrePaint.MarkPaintPhaseNeeds");
                MarkPaintPhaseNeeds(&mut tree);
            }
        }
        {
            let mut admission = browser_tracing::span("prepaint", "PrePaint.Admission");
            admission.set("same_context", self.stats.same_context_nodes as f64);
            admission.set("retained_context", self.stats.retained_context_nodes as f64);
            admission.set(
                "retained_covered",
                self.stats.retained_geometry_covered_nodes as f64,
            );
            // Retained nodes admitted (or rejected) by each rule of the
            // dirty-bit admission (UpdateNode).
            admission.set("self_unchanged", self.stats.self_unchanged_nodes as f64);
            admission.set(
                "subtree_unchanged",
                self.stats.subtree_unchanged_nodes as f64,
            );
            admission.set("reject_own_bits", self.stats.admission_rejects[0] as f64);
            admission.set("reject_offset", self.stats.admission_rejects[1] as f64);
            admission.set("reject_size", self.stats.admission_rejects[2] as f64);
            admission.set("reject_identity", self.stats.admission_rejects[3] as f64);
            admission.set("reject_owners", self.stats.admission_rejects[4] as f64);
            admission.set("reject_resources", self.stats.admission_rejects[5] as f64);
            admission.set(
                "reject_descendant_bits",
                self.stats.admission_rejects[6] as f64,
            );
            admission.set(
                "reject_pending_child_walk",
                self.stats.admission_rejects[7] as f64,
            );
            admission.set(
                "reject_subtree_resources",
                self.stats.admission_rejects[8] as f64,
            );
            // What the admitted nodes became.
            admission.set("skipped_subtrees", self.stats.skipped_subtrees as f64);
            admission.set("shifted", self.stats.shifted_nodes as f64);
            admission.set(
                "invalidation_only",
                self.stats.invalidation_only_nodes as f64,
            );
            self.resource_memo.clear();
            admission.set(
                "source_tree_compatible",
                self.stats.source_tree_compatible_nodes as f64,
            );
            admission.set("invalidation_nodes", self.stats.invalidation_nodes as f64);
        }
        PropagateAutoDarkModeState(&mut tree, false);
        let mut repaint_counts = [0; 3];
        PropagateSubsequenceRepaintCounting(&mut tree, &mut repaint_counts);
        browser_tracing::instant(
            "prepaint",
            "PrePaint.Repaint",
            &[
                ("paint_layers", repaint_counts[0] as f64),
                ("source_needs_repaint", repaint_counts[1] as f64),
                ("client_propagated", repaint_counts[2] as f64),
                ("cull_change_repaints", self.stats.cull_change_repaints as f64),
            ],
        );
        // Every slot of a built or retained object registered its key in
        // this pass; the others belong to objects that no longer need them.
        {
            let live: HashSet<PaintPropertyKey> = self.property_keys.values().copied().collect();
            self.property_node_store
                .RetainLiveSlots(|key| live.contains(key));
            self.intersection_observation_state = self
                .intersection_observation_state
                .max(self.property_node_store.intersection_observation_state);
            self.next_property_id = self.property_node_store.next_id();
        }
        tree
    }

    pub fn RetainTree(&mut self, mut tree: Box<PaintTreeNode<'static>>) {
        if self.stats.post_walk {
            // Chromium's DisplayItemClient directory is resident across
            // lifecycle updates. Rebuild the contents without discarding the
            // per-client storage: a resize commonly preserves the same client
            // set, so dropping every bucket here only recreates identical
            // allocations on the next frame.
            for nodes in self.client_nodes.values_mut() {
                nodes.clear();
            }
            self.resident_nodes.clear();
            fn collect(
                node: &mut PaintTreeNode<'static>,
                clients: &mut HashMap<u64, Vec<*mut PaintTreeNode<'static>>>,
                nodes: &mut Vec<*mut PaintTreeNode<'static>>,
            ) {
                let pointer = node as *mut PaintTreeNode<'static>;
                nodes.push(pointer);
                let f = node.fragment.as_deref().unwrap();
                let mut add = |id| {
                    if id != 0 {
                        clients.entry(id).or_default().push(pointer);
                    }
                };
                add(f.paint.DisplayItemClientId());
                add(f.paint.SelectionClientId());
                add(f.paint.PaintLayerClientId());
                if let Some(scroll) = f.ViewScrollContainer() {
                    add(scroll.scrolling_background_client_id);
                }
                if let Some(bars) = f.ViewScrollbars() {
                    if let Some(axis) = &bars.horizontal {
                        add(axis.display_item_client_id);
                    }
                    if let Some(axis) = &bars.vertical {
                        add(axis.display_item_client_id);
                    }
                    add(bars.corner_client_id);
                }
                for child in &mut node.children {
                    collect(child, clients, nodes);
                }
            }
            collect(&mut tree, &mut self.client_nodes, &mut self.resident_nodes);
            self.client_nodes.retain(|_, nodes| !nodes.is_empty());
        }
        self.retained_tree = Some(tree);
        if let Some(lease) = self.pending_source_lease.take() {
            self.retained_source_lease = Some(lease);
        }
        self.retained_local_frame_leases = self
            .local_frame_views
            .values()
            .map(|view| view.lease.clone())
            .collect();
        self.local_frame_sources_changed = false;
        self.retained_tree_valid = true;
    }

    /// Source: LocalFrameView::ExecuteAllPendingUpdates and
    /// PaintPropertyTreeBuilder::DirectlyUpdate{TransformMatrix,OpacityValue}.
    ///
    /// Layout has already refreshed the canonical FragmentData sidecars for
    /// these objects. Update only their existing ObjectPaintProperties slots;
    /// topology, compositing-reason, or owner changes reject this path so the
    /// caller can run the ordinary complete PrePaint walk.
    pub fn DirectlyUpdateCompositedAnimationProperties(
        &mut self,
        targeted_nodes: &[u64],
    ) -> Option<DirectCompositedAnimationPropertyUpdate> {
        let mut trace = browser_tracing::span("prepaint", "PrePaint.DirectAnimationPropertyUpdate");
        let targeted = targeted_nodes.iter().copied().collect::<BTreeSet<_>>();
        trace.set("targeted_nodes", targeted.len() as f64);
        if targeted.is_empty() || !self.retained_tree_valid || self.retained_tree.is_none() {
            trace.set("rejection", 1.0);
            return None;
        }

        let mut matched = BTreeSet::new();
        let mut active = BTreeSet::new();
        let mut transform_proposals = BTreeMap::new();
        let mut effect_proposals = BTreeMap::new();
        let mut rejection = 0u8;

        for &pointer in &self.resident_nodes {
            // RetainTree owns every pointer in this directory. It is rebuilt
            // whenever topology changes and no Box moves during this method.
            let node = unsafe { &*pointer };
            let Some(fragment) = node.fragment.as_deref() else {
                continue;
            };
            if !targeted.contains(&fragment.ViewNodeId()) {
                continue;
            }
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
                fragment: node.DisplayItemFragment(),
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
                    || !node.applies_transform
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
                            Offset {
                                x: node.paint_offset.x + node.paint_snap_offset.x,
                                y: node.paint_offset.y + node.paint_snap_offset.y,
                            },
                            reference_rect,
                            transform_origin.as_ref(),
                        )
                    },
                );
                let matrix = MultiplyTransforms(
                    &TranslationTransform(-node.paint_snap_offset.x, -node.paint_snap_offset.y),
                    &resolved,
                );
                let mut next = (**current).clone();
                let change = next.DirectlyUpdateTransformAndOrigin(
                    matrix,
                    [0.0; 3],
                    TransformAnimationState {
                        is_running_animation_on_compositor: true,
                    },
                );
                if !matches!(
                    change,
                    PaintPropertyChangeType::Unchanged
                        | PaintPropertyChangeType::ChangedOnlyCompositedValues
                ) {
                    rejection = 4;
                    break;
                }
                transform_proposals.insert(key, (Arc::new(next), change, fragment.ViewNodeId()));
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
                    let mut next = (**current).clone();
                    let change = next.DirectlyUpdateOpacity(
                        MaskAdjustedOpacity(style),
                        EffectAnimationState {
                            is_running_opacity_animation_on_compositor: true,
                            ..Default::default()
                        },
                    );
                    if !matches!(
                        change,
                        PaintPropertyChangeType::Unchanged
                            | PaintPropertyChangeType::ChangedOnlyCompositedValues
                    ) {
                        rejection = 4;
                        break;
                    }
                    effect_proposals.insert(key, (Arc::new(next), change, fragment.ViewNodeId()));
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

        let mut update = DirectCompositedAnimationPropertyUpdate::default();
        for (key, (snapshot, change, node_id)) in transform_proposals {
            self.property_node_store
                .PublishDirectTransformSnapshot(key, snapshot.clone(), change);
            update.transforms.push(snapshot);
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
        for (key, (snapshot, change, node_id)) in effect_proposals {
            self.property_node_store
                .PublishDirectEffectSnapshot(key, snapshot.clone(), change);
            update.effects.push(snapshot);
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

    /// Whether a PrePaint walk visited (and therefore cleared the paint
    /// flags of) any object since the last recorded Paint. A walk consumes
    /// LayoutObject dirty bits (pre_paint_tree_walk.cc:1584-1585), so after
    /// a PrePaint-only lifecycle (hit-test preparation, a composited
    /// animation fallback) clean bits no longer prove that the published
    /// artifact reflects the source: it was recorded before that walk.
    pub fn PrePaintConsumedSinceRecording(&self) -> bool {
        self.consuming_walks != self.recorded_consuming_walks
    }

    /// Return the compositor element ids created by the completed PrePaint
    /// walk for CSS transform/opacity animation targets. Chromium creates the
    /// KeyframeModel and its paint-property node in the same main-frame
    /// commit. Rechrom keeps those owners in separate packages, so Page uses
    /// this typed identity list to preserve that ordering without exposing the
    /// resident Paint tree.
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

        fn walk(
            node: &PaintTreeNode<'_>,
            targeted: &HashSet<u64>,
            matched: &mut HashSet<u64>,
            seen_transforms: &mut HashSet<u64>,
            seen_effects: &mut HashSet<u64>,
            output: &mut Vec<CompositedAnimationPropertyBinding>,
        ) {
            let node_id = node
                .fragment
                .as_deref()
                .map_or(0, |fragment| fragment.ViewNodeId());
            if targeted.contains(&node_id) {
                matched.insert(node_id);
                browser_tracing::instant(
                    "prepaint",
                    "PrePaint.CompositedAnimationTarget",
                    &[
                        ("node_id", node_id as f64),
                        ("present", 1.0),
                        ("applies_transform", node.applies_transform as u8 as f64),
                        ("applies_opacity", node.applies_opacity as u8 as f64),
                        (
                            "active_transform",
                            paint_model::property_cell::with_access(|access| {
                                node.properties.nodes.transform.HasActiveTransformAnimation(access)
                            }) as u8 as f64,
                        ),
                        (
                            "active_opacity",
                            paint_model::property_cell::with_access(|access| {
                                node.properties.nodes.effect.HasActiveOpacityAnimation(access)
                            }) as u8 as f64,
                        ),
                    ],
                );
                if node.applies_transform {
                    collect_transform(
                        node_id,
                        Some(&node.properties.nodes.transform),
                        seen_transforms,
                        output,
                    );
                }
                if node.applies_opacity {
                    collect_effect(
                        node_id,
                        Some(&node.properties.nodes.effect),
                        seen_effects,
                        output,
                    );
                }
            }
            for child in &node.children {
                walk(
                    child,
                    targeted,
                    matched,
                    seen_transforms,
                    seen_effects,
                    output,
                );
            }
        }

        let Some(tree) = self.retained_tree.as_deref() else {
            return Vec::new();
        };
        let targeted = targeted_nodes.iter().copied().collect::<HashSet<_>>();
        let mut matched = HashSet::new();
        let mut output = Vec::new();
        walk(
            tree,
            &targeted,
            &mut matched,
            &mut HashSet::new(),
            &mut HashSet::new(),
            &mut output,
        );
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
    /// layout snapshot. Update only resident sources for the committed clients.
    pub fn CommitClients(&mut self, clients: impl IntoIterator<Item = u64>) {
        for id in clients {
            let Some(nodes) = self.client_nodes.get(&id) else {
                continue;
            };
            for &pointer in nodes {
                // This directory is rebuilt before publication whenever owners
                // are added/removed. Box addresses survive local updates.
                let node = unsafe { &mut *pointer };
                let f = node.fragment.as_deref_mut().unwrap();
                // Live source clients were validated by the Layout commit's
                // PaintController cycle; only detached payload state is local.
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
                                axis.display_item_client_id == id
                                    && axis.display_item_client_is_cacheable
                            })
                });
                if let Some(bars) = bars_match.then(|| f.paint.scrollbars.as_mut()).flatten() {
                    let changes = (bars.corner_client_id == id && bars.corner_client_is_cacheable)
                        || [&bars.horizontal, &bars.vertical]
                            .into_iter()
                            .flatten()
                            .any(|axis| {
                                axis.display_item_client_id == id
                                    && axis.display_item_client_is_cacheable
                            });
                    if changes {
                        let bars = Arc::make_mut(bars);
                        for axis in [&mut bars.horizontal, &mut bars.vertical]
                            .into_iter()
                            .flatten()
                        {
                            if axis.display_item_client_id == id
                                && axis.display_item_client_is_cacheable
                            {
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
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CanRetainDisplayLockedChildren, ClipsOverflow, CullRectChangeNeedsRepaint,
        EnclosingMaskClipRectInPropertySpace, FilterTaintChanged, HasFilterThatMovesPixels,
        HasPerspective, InvalidateClientsForCullRectChange, MarkPaintPhaseNeeds, NeedsOverflowClip,
        NeedsPaintOffsetTranslation, ObjectRequiresTreeBuilderContext, OwnFilterOriginTainted,
        PaintPropertyRole, PaintPropertyTreeBuilder, PaintRect, PaintTreeNode, PixelSnappedRect,
        PrePaintTreeWalk, PrepareOutlines, PropagateAutoDarkModeState, PropagateSubsequenceRepaint,
        PropertyTreeState, RequiresFullSourceWalk, ResidentFragment, ResolveClipPath,
        ResolveFragmentGeometry, ResolveMaskLayers, RetainImagePaintedRect, SamePhaseInputs,
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
    fn new_descendant_client_invalidates_the_ancestor_layer_subsequence() {
        let mut layer = PaintTreeNode::default();
        layer.is_paint_layer = true;
        layer.self_or_descendant_needs_repaint = false;
        layer.fragment = Some(ResidentFragment::FromSource(&FragmentPaintData::default()));

        let mut child_fragment = FragmentPaintData::default();
        child_fragment.paint.display_item_client_invalidation_reason =
            Some(foundation::PaintInvalidationReason::kJustCreated);
        let mut child = PaintTreeNode::default();
        child.self_or_descendant_needs_repaint = false;
        child.fragment = Some(ResidentFragment::FromSource(&child_fragment));
        layer.children.push(Box::new(child));

        assert!(PropagateSubsequenceRepaint(&mut layer));
        assert!(layer.self_or_descendant_needs_repaint);
    }

    #[test]
    fn repaint_propagation_visits_dirty_sibling_layers_after_the_first_match() {
        fn dirty_leaf() -> Box<PaintTreeNode<'static>> {
            let mut fragment = FragmentPaintData::default();
            fragment.paint.display_item_client_invalidation_reason =
                Some(foundation::PaintInvalidationReason::kJustCreated);
            let mut leaf = PaintTreeNode::default();
            leaf.fragment = Some(ResidentFragment::FromSource(&fragment));
            Box::new(leaf)
        }

        let mut root = PaintTreeNode::default();
        root.is_paint_layer = true;
        root.self_or_descendant_needs_repaint = false;
        root.fragment = Some(ResidentFragment::FromSource(&FragmentPaintData::default()));

        let mut first_layer = PaintTreeNode::default();
        first_layer.is_paint_layer = true;
        first_layer.self_or_descendant_needs_repaint = false;
        first_layer.fragment = Some(ResidentFragment::FromSource(&FragmentPaintData::default()));
        first_layer.children.push(dirty_leaf());

        let mut later_layer = PaintTreeNode::default();
        later_layer.is_paint_layer = true;
        later_layer.self_or_descendant_needs_repaint = false;
        later_layer.fragment = Some(ResidentFragment::FromSource(&FragmentPaintData::default()));
        later_layer.children.push(dirty_leaf());

        root.children.push(Box::new(first_layer));
        root.children.push(Box::new(later_layer));

        assert!(PropagateSubsequenceRepaint(&mut root));
        assert!(root.children[0].self_or_descendant_needs_repaint);
        assert!(
            root.children[1].self_or_descendant_needs_repaint,
            "later dirty PaintLayers must not retain a stale cached subsequence"
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
    fn tree_builder_context_respects_descendant_dirty_bits_and_display_lock() {
        let mut fragment = FragmentPaintData::default();
        fragment
            
            .pre_paint_invalidation
            .descendant_needs_paint_property_update = true;
        assert!(ObjectRequiresTreeBuilderContext(&fragment));

        fragment
            
            .pre_paint_invalidation
            .child_pre_paint_blocked_by_display_lock = true;
        assert!(!ObjectRequiresTreeBuilderContext(&fragment));

        fragment
            
            .pre_paint_invalidation
            .should_check_layout_for_paint_invalidation = true;
        assert!(ObjectRequiresTreeBuilderContext(&fragment));
    }

    #[test]
    fn display_locked_children_do_not_outlive_an_incompatible_source_lease() {
        assert!(CanRetainDisplayLockedChildren(true, false));
        assert!(!CanRetainDisplayLockedChildren(true, true));
        assert!(!CanRetainDisplayLockedChildren(false, true));
    }

    #[test]
    fn replacement_source_owner_forces_follow_up_outline_walks() {
        // The regression was a retained ResidentFragment pointer surviving a
        // replacement LayoutResult lease. A later animation repaint entered
        // SamePhaseInputs::outline_style after the old sidecar was freed.
        // Every replacement round must therefore take the full linear walk,
        // even when revisions are otherwise cacheable.
        let mut resident_source: Option<Box<FragmentPaintData>> = None;
        let mut resident_node: Option<Box<PaintTreeNode<'static>>> = None;
        for revision in [1, 2, 3, u64::MAX] {
            assert!(RequiresFullSourceWalk(true, true));

            let mut source = Box::new(FragmentPaintData::default());
            source.paint.has_source = true;
            source.paint.style.outline_style = if revision % 2 == 0 {
                BorderLineStyle::kDashed
            } else {
                BorderLineStyle::kSolid
            };
            source.paint.style.outline_width = revision as f64;
            let node = PaintPropertyTreeBuilder::new(&mut PrePaintTreeWalk::default()).BuildNode(
                &source,
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
            // Destroy the old retained node before its source owner, then
            // install the replacement pair. This is the order enforced by
            // Walk/RetainTree around the lease boundary.
            resident_node = Some(Box::new(node));
            resident_source = Some(source);
            let current = resident_node
                .as_deref()
                .unwrap()
                .fragment
                .as_deref()
                .unwrap();
            assert_eq!(
                current.paint.style.outline_style(),
                resident_source
                    .as_deref()
                    .unwrap()
                    .paint
                    .style
                    .outline_style()
            );
            assert!(SamePhaseInputs(current, current));
        }
        assert!(!RequiresFullSourceWalk(false, true));
        assert!(RequiresFullSourceWalk(false, false));
    }

    #[test]
    fn inline_outline_group_joins_exact_source_rects_in_paint_space() {
        fn occurrence(local_rects: Vec<FragmentPaintRect>) -> FragmentPaintData {
            let mut fragment = FragmentPaintData::default();
            fragment.node_id = 17;
            fragment.fragmentainer_instance_id = 3;
            fragment.kind = FragmentKind::kBox;
            fragment.paint.has_source = true;
            fragment.paint.establishes_paint_state = true;
            fragment.paint.display = Display::kInline;
            fragment.paint.style.SetVisibleForTesting(true);
            fragment.paint.style.outline_width = 2.0;
            fragment.paint.style.outline_style = BorderLineStyle::kSolid;
            fragment.paint.style.outline_color = Color {
                alpha: 1.0,
                ..Color::default()
            };
            fragment.paint.self_outline = Some(SelfOutlinePaintData {
                rects: local_rects,
                width: 2.0,
                offset: 0.0,
            });
            fragment
        }

        // AddSelfOutlineRects returns the complete inline continuation group
        // relative to its one source outline owner. Non-owner occurrences
        // export authoritative empty geometry.
        let first = occurrence(vec![
            FragmentPaintRect {
                offset: Offset { x: 1.0, y: 2.0 },
                size: Size {
                    width: 10.0,
                    height: 20.0,
                },
            },
            FragmentPaintRect {
                offset: Offset { x: 203.0, y: 204.0 },
                size: Size {
                    width: 30.0,
                    height: 40.0,
                },
            },
        ]);
        let second = occurrence(Vec::new());
        let mut owner = PaintTreeNode::default();
        owner.SetOwnedFragmentForTesting(&first);
        owner.paint_offset = Offset { x: 100.0, y: 200.0 };
        let mut child = PaintTreeNode::default();
        child.SetOwnedFragmentForTesting(&second);
        child.paint_offset = Offset { x: 300.0, y: 400.0 };
        owner.children.push(Box::new(child));

        PrepareOutlines(&mut owner);

        assert_eq!(
            owner.outline_rects,
            vec![
                PaintRect {
                    x: 101.0,
                    y: 202.0,
                    width: 10.0,
                    height: 20.0,
                },
                PaintRect {
                    x: 303.0,
                    y: 404.0,
                    width: 30.0,
                    height: 40.0,
                },
            ]
        );
        assert!(!owner.children[0].paints_outline);
        assert!(owner.children[0].outline_rects.is_empty());
    }

    #[test]
    fn rebuilt_image_fragment_retains_painted_rect_for_the_same_client() {
        let painted_rect = FragmentPaintRect {
            offset: Offset { x: 10.0, y: 20.0 },
            size: Size {
                width: 30.0,
                height: 40.0,
            },
        };
        let mut previous = FragmentPaintData::default();
        previous.paint.replaced_is_image = true;
        previous.paint.display_item_client_id = 7;
        previous.paint.image_last_paint_rect.set(Some(painted_rect));
        previous
            .paint
            .image_last_painted_id
            .set(image_resource::PaintImageId(42));

        let mut current = FragmentPaintData::default();
        current.paint.replaced_is_image = true;
        current.paint.display_item_client_id = 7;
        RetainImagePaintedRect(&current, Some(&previous), Some(7));
        assert_eq!(
            current.paint.image_last_paint_rect.get(),
            Some(painted_rect)
        );
        assert_eq!(
            current.paint.image_last_painted_id.get(),
            image_resource::PaintImageId(42)
        );

        let mut different_client = FragmentPaintData::default();
        different_client.paint.replaced_is_image = true;
        different_client.paint.display_item_client_id = 8;
        RetainImagePaintedRect(&different_client, Some(&previous), Some(7));
        assert_eq!(different_client.paint.image_last_paint_rect.get(), None);
        assert_eq!(
            different_client.paint.image_last_painted_id.get(),
            image_resource::PaintImageId::INVALID
        );
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
    fn committed_resident_clients_become_cache_valid() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.display_item_client_id = 11;
        fragment.paint.display_item_client_is_cacheable = true;
        fragment.paint.display_item_client_invalidation_reason =
            Some(foundation::PaintInvalidationReason::kLayout);
        fragment.paint.selection_client_id = 12;
        fragment.paint.selection_client_is_cacheable = true;
        fragment.paint.selection_client_invalidation_reason =
            Some(foundation::PaintInvalidationReason::kSelection);

        let mut root = PaintTreeNode::default();
        root.fragment = Some(ResidentFragment::FromSource(&fragment));
        let mut walk = PrePaintTreeWalk::default();
        walk.stats.post_walk = true;
        walk.RetainTree(Box::new(root));
        walk.CommitClients([11, 12]);

        let paint = &walk
            .retained_tree
            .as_ref()
            .unwrap()
            .fragment
            .as_deref()
            .unwrap()
            .paint;
        assert_eq!(
            paint.display_item_client_invalidation_reason,
            Some(foundation::PaintInvalidationReason::kNone)
        );
        assert_eq!(
            paint.selection_client_invalidation_reason,
            Some(foundation::PaintInvalidationReason::kNone)
        );
    }

    #[test]
    fn phase_needs_stop_at_self_painting_layer_boundaries() {
        let mut child_fragment = FragmentPaintData::default();
        child_fragment.paint.has_source = true;
        child_fragment.paint.floating = FloatSide::kLeft;
        child_fragment.paint.scrollbars = Some(Arc::new(ScrollbarPaintData {
            should_paint_as_overlay: true,
            ..ScrollbarPaintData::default()
        }));

        let mut outlined_fragment = FragmentPaintData::default();
        outlined_fragment.paint.has_source = true;
        outlined_fragment.paint.style.outline_style =
            paint_input_types::layout_input::BorderLineStyle::kSolid;
        outlined_fragment.paint.style.outline_width = 1.0;
        outlined_fragment.paint.style.outline_color.alpha = 1.0;
        let mut outlined = PaintTreeNode::default();
        outlined.fragment = Some(ResidentFragment::FromSource(&outlined_fragment));

        let mut child = PaintTreeNode::default();
        child.fragment = Some(ResidentFragment::FromSource(&child_fragment));
        child.is_paint_layer = true;
        child.children.push(Box::new(outlined));

        let mut root = PaintTreeNode::default();
        root.fragment = Some(ResidentFragment::FromSource(&FragmentPaintData::default()));
        root.is_paint_layer = true;
        root.children.push(Box::new(child));

        MarkPaintPhaseNeeds(&mut root);
        let child = &root.children[0];
        assert!(child.needs_float_phase);
        assert!(child.needs_outline_phase);
        assert!(child.needs_overflow_controls_phase);
        assert!(!root.needs_float_phase);
        assert!(!root.needs_outline_phase);
        assert!(!root.needs_overflow_controls_phase);
    }

    #[test]
    fn phase_needs_propagate_through_non_layer_descendants() {
        let mut child_fragment = FragmentPaintData::default();
        child_fragment.paint.has_source = true;
        child_fragment.paint.floating = FloatSide::kLeft;

        let mut child = PaintTreeNode::default();
        child.fragment = Some(ResidentFragment::FromSource(&child_fragment));

        let mut root = PaintTreeNode::default();
        root.fragment = Some(ResidentFragment::FromSource(&FragmentPaintData::default()));
        root.is_paint_layer = true;
        root.children.push(Box::new(child));

        MarkPaintPhaseNeeds(&mut root);
        assert!(root.needs_float_phase);
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
    fn large_svg_pauses_descendant_auto_dark_and_nested_icon_reenables_it() {
        let mut outer_fragment = FragmentPaintData::default();
        outer_fragment.paint.source_kind = NodeKind::kSvgRoot;
        outer_fragment.paint.style.force_dark = true;
        outer_fragment.size = Size {
            width: 200.0,
            height: 100.0,
        };
        let mut outer = PaintTreeNode::default();
        outer.SetOwnedFragmentForTesting(&outer_fragment);

        let mut direct_fragment = FragmentPaintData::default();
        direct_fragment.paint.source_kind = NodeKind::kSvgShape;
        direct_fragment.paint.style.force_dark = true;
        direct_fragment.paint.selection_style.force_dark = true;
        let mut direct = PaintTreeNode::default();
        direct.SetOwnedFragmentForTesting(&direct_fragment);

        let mut nested_fragment = FragmentPaintData::default();
        nested_fragment.paint.source_kind = NodeKind::kSvgRoot;
        nested_fragment.paint.style.force_dark = true;
        nested_fragment.size = Size {
            width: 32.0,
            height: 32.0,
        };
        let mut nested = PaintTreeNode::default();
        nested.SetOwnedFragmentForTesting(&nested_fragment);

        let mut nested_child_fragment = FragmentPaintData::default();
        nested_child_fragment.paint.source_kind = NodeKind::kSvgShape;
        nested_child_fragment.paint.style.force_dark = true;
        let mut nested_child = PaintTreeNode::default();
        nested_child.SetOwnedFragmentForTesting(&nested_child_fragment);
        nested.children.push(Box::new(nested_child));

        outer.children.push(Box::new(direct));
        outer.children.push(Box::new(nested));
        PropagateAutoDarkModeState(&mut outer, false);

        assert!(!outer
            .fragment
            .as_deref()
            .unwrap()
            .paint
            .style
            .auto_dark_mode_paused());
        let direct = outer.children[0].fragment.as_deref().unwrap();
        assert!(direct.paint.style.auto_dark_mode_paused());
        assert!(direct.paint.selection_style.auto_dark_mode_paused());
        let nested = outer.children[1].as_ref();
        assert!(nested
            .fragment
            .as_deref()
            .unwrap()
            .paint
            .style
            .auto_dark_mode_paused());
        assert!(!nested.children[0]
            .fragment
            .as_deref()
            .unwrap()
            .paint
            .style
            .auto_dark_mode_paused());
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
