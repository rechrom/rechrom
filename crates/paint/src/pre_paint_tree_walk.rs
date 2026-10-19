#![allow(non_snake_case)]

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;

use layoutng_assembly::fragment_tree::{FragmentKind, FragmentNode, TextCombinePaintData};
use layoutng_assembly::internal::layout_input::{
    BorderLineStyle, Display, Edges, FloatSide, NodeKind, Offset, Overflow, PaintClipPathChild,
    PaintImage, PaintPathCommand, PaintPathVerb, Position, Size, TransformMatrix,
};
use layoutng_assembly::internal::paint_input::{
    BackgroundBox, ClipPathLength, ClipPathPaint, ClipPathRadiusKind, ClipPathReferenceBox,
    ClipPathShapeRadius, PaintBlendMode, PaintCornerRadii, PaintCornerRadius, PaintFilterOperation,
    PaintFilterType, PaintMaskImageState, PaintMaskLayer, PaintReflectionDirection, PaintStyleData,
    SvgFilterPrimitiveKind,
};

use crate::background_geometry::{ResolveBackgroundTile, ResolveBackgroundTileSize};
use crate::border_shape_utils::{
    ConicRoundedRectPath, ExpandCornerRadiiEdges, IncludedBorderEdges, ResolveCornerRadii,
    UniformCornerRadius,
};
use crate::geometry_mapper::{
    MapRectToRoot, MultiplyTransforms, ResolveLocalTransformAroundReferenceBox,
    ResolveTransformReferenceRect, TranslationTransform,
};
use crate::nine_piece_image_painter::NinePieceImagePainter;
use crate::paint_engine::{DisplayMaskLayer, ResolveDocumentImageContainerKey};
use crate::paint_order::ClassifyFragmentPaintLayer;
use crate::paint_property_tree::{
    ClipPaintPropertyNode, EffectPaintPropertyNode, PaintBackfaceVisibility,
    PaintCompositorStickyConstraint, PaintPropertyKey, PaintPropertyNodeStore, PaintPropertyOwner,
    PaintPropertyRole, PropertyNodeUpdater, ScrollPaintPropertyNode, TransformPaintPropertyNode,
};
use crate::PaintRect;

// Source: paint_property_tree_builder.cc:935-937,1286-1288. IDs come from the
// original FragmentData/box owner, never a dump-local property label.
fn TransformCompositorState(
    fragment: &FragmentNode,
    role: PaintPropertyRole,
) -> crate::paint_property_tree::TransformCompositorState {
    use foundation::{CompositorElementIdFromUniqueObjectId, CompositorElementIdNamespace as Ns};
    let mut value = crate::paint_property_tree::TransformCompositorState::default();
    value.is_for_svg_child = role == PaintPropertyRole::SvgLocalTransform
        && fragment.paint.source_kind != NodeKind::kSvgRoot;
    value.is_frame_paint_offset_translation =
        role == PaintPropertyRole::PaintOffsetTranslation && fragment.paint.is_layout_view;
    let ns = match role {
        PaintPropertyRole::StickyTranslation => Some(Ns::kStickyTranslation),
        PaintPropertyRole::SvgLocalTransform => Some(Ns::kPrimaryTransform),
        _ => None,
    };
    if let Some(ns) = ns {
        if fragment.paint.property_fragment_unique_id != 0 {
            value.compositor_element_id = CompositorElementIdFromUniqueObjectId(
                fragment.paint.property_fragment_unique_id,
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
        let mut scroll = node.scroll.clone();
        while let Some(candidate) = scroll {
            if candidate.owner_client_id == owner_client_id {
                return candidate.compositor.compositor_element_id;
            }
            scroll = candidate.parent.clone();
        }
        transform = node.parent.clone();
    }
    Default::default()
}

fn StickyElementIdForClient(
    parent: &Arc<TransformPaintPropertyNode>,
    owner_client_id: u64,
) -> foundation::CompositorElementId {
    let mut transform = Some(parent.clone());
    while let Some(node) = transform {
        if node
            .sticky_constraint
            .is_some_and(|constraint| constraint.owner_client_id == owner_client_id)
        {
            return node.compositor.compositor_element_id;
        }
        transform = node.parent.clone();
    }
    Default::default()
}

fn BuildCompositorStickyConstraint(
    sticky: layoutng_assembly::fragment_tree::StickyPositionPaintData,
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
    fragment: &FragmentNode,
    role: PaintPropertyRole,
    has_direct_reasons: bool,
    backdrop_mask: bool,
) -> crate::paint_property_tree::EffectCompositorState {
    use foundation::{CompositorElementIdFromUniqueObjectId, CompositorElementIdNamespace as Ns};
    let mut value = crate::paint_property_tree::EffectCompositorState::default();
    if fragment.paint.property_fragment_unique_id != 0 {
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
            fragment.paint.property_fragment_unique_id,
            namespace,
        );
        if backdrop_mask {
            value.backdrop_mask_element_id = CompositorElementIdFromUniqueObjectId(
                fragment.paint.property_fragment_unique_id,
                Ns::kEffectMask,
            );
        }
    }
    value
}

#[cfg(feature = "translation_in_progress")]
use crate::paint_shader_resolver::ResolvePaintShader;

// cpp: paint/pre_paint_tree_walk.h:11-16
#[derive(Clone, Default)]
pub struct PropertyTreeState {
    pub transform_id: u64,
    pub clip_id: u64,
    pub effect_id: u64,
    pub scroll_id: u64,
    pub nodes: crate::paint_property_tree::PropertyTreeState,
}

// cpp: paint/pre_paint_tree_walk.h:18-80
pub struct PaintTreeNode<'a> {
    pub fragment: Option<Arc<FragmentNode>>,
    pub(crate) source_lifetime: std::marker::PhantomData<&'a FragmentNode>,
    physical_index: usize,
    cache: Option<NodeCache>,
    pub absolute_offset: Offset,
    pub paint_offset: Offset,
    // Pixel snapping belongs to the local property space. This delta only
    // bridges a real paint-space reset; scrolling stays in ScrollTranslation.
    pub(crate) paint_snap_offset: Offset,
    // Blink snaps paint geometry to physical pixels. LayoutNG coordinates in
    // this crate remain CSS pixels, so retain the root device scale while
    // applying the same snap instead of rounding to whole CSS pixels.
    pub(crate) pixel_snap_scale: f64,
    pub root_clip: Option<PaintRect>,
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
    pub transforms: Vec<TransformMatrix>,
    pub local_transforms: Vec<TransformMatrix>,
    pub opacity: f32,
    pub scroll_offset: Offset,
    /// CSS transforms on non-root ancestors make background-attachment:fixed
    /// behave as scroll. Root transforms are deliberately excluded: Chromium
    /// keeps the viewport attachment and applies the root transform to it.
    pub fixed_background_acts_as_scroll: bool,
    pub is_paint_layer: bool,
    pub is_stacking_context: bool,
    pub is_root: bool,
    pub escapes_ancestor_geometry: bool,
    pub physical_ancestors: Vec<*const PaintTreeNode<'a>>,
    pub stacking_level: i32,
    pub applies_transform: bool,
    pub applies_clip: bool,
    pub applies_root_clip: bool,
    pub applies_overflow_clip: bool,
    pub applies_css_clip: bool,
    pub applies_clip_path: bool,
    pub applies_effect: bool,
    pub applies_compositing_layer: bool,
    pub applies_overlap_compositing_layer: bool,
    pub applies_opacity: bool,
    pub applies_blend: bool,
    pub applies_filter: bool,
    pub applies_backdrop_filter: bool,
    pub applies_mask: bool,
    pub paints_outline: bool,
    pub needs_float_phase: bool,
    pub needs_outline_phase: bool,
    pub needs_overflow_controls_phase: bool,
    pub outline_rects: Vec<PaintRect>,
    pub mask_layers: Vec<DisplayMaskLayer>,
    /// The independent SaveLayerFilter drawing-record payload. Effect property
    /// nodes own only their canonical compositor operations, never this list.
    pub recording_filters: Vec<PaintFilterOperation>,
    pub properties: PropertyTreeState,
    pub contents_properties: PropertyTreeState,
    /// The composited fixed background paints in the LayoutView transform,
    /// while its BackgroundClip remains in this box's local transform space.
    pub fixed_background_properties: Option<crate::paint_property_tree::PropertyTreeState>,
    /// Mask source uses a separate DstIn effect and the pre-MaskClip output
    /// clip, matching PaintLayerPainter::PaintFragmentWithPhase(kMask).
    pub mask_properties: Option<crate::paint_property_tree::PropertyTreeState>,
    pub children: Vec<Box<PaintTreeNode<'a>>>,
}

impl<'a> Default for PaintTreeNode<'a> {
    fn default() -> Self {
        Self {
            fragment: None,
            source_lifetime: std::marker::PhantomData,
            physical_index: 0,
            cache: None,
            absolute_offset: Offset::default(),
            paint_offset: Offset::default(),
            paint_snap_offset: Offset::default(),
            pixel_snap_scale: 1.0,
            root_clip: None,
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
            transforms: Vec::new(),
            local_transforms: Vec::new(),
            opacity: 1.0,
            scroll_offset: Offset::default(),
            fixed_background_acts_as_scroll: false,
            is_paint_layer: false,
            is_stacking_context: false,
            is_root: false,
            escapes_ancestor_geometry: false,
            physical_ancestors: Vec::new(),
            stacking_level: 0,
            applies_transform: false,
            applies_clip: false,
            applies_root_clip: false,
            applies_overflow_clip: false,
            applies_css_clip: false,
            applies_clip_path: false,
            applies_effect: false,
            applies_compositing_layer: false,
            applies_overlap_compositing_layer: false,
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
            outline_rects: Vec::new(),
            mask_layers: Vec::new(),
            recording_filters: Vec::new(),
            properties: PropertyTreeState::default(),
            contents_properties: PropertyTreeState::default(),
            fixed_background_properties: None,
            mask_properties: None,
            children: Vec::new(),
        }
    }
}

impl PaintTreeNode<'_> {
    pub(crate) fn SetFragment(&mut self, source: &FragmentNode) {
        // Do not keep a layout-root Rc or clone the source subtree. Painters use
        // this node's resident children, not FragmentNode.children.
        let FragmentNode {
            inline_geometry: _,
            fragment_instance_id,
            fragmentainer_instance_id,
            node_id,
            kind,
            offset,
            size,
            content_size,
            text_start,
            text_end,
            paint,
            pre_paint_revision,
            pre_paint_subtree_revision,
            pre_paint_owner_retained,
            children: _,
        } = source;
        self.fragment = Some(Arc::new(FragmentNode {
            inline_geometry: Vec::new(),
            fragment_instance_id: *fragment_instance_id,
            fragmentainer_instance_id: *fragmentainer_instance_id,
            node_id: *node_id,
            kind: *kind,
            offset: *offset,
            size: *size,
            content_size: *content_size,
            text_start: *text_start,
            text_end: *text_end,
            paint: paint.clone(),
            pre_paint_revision: *pre_paint_revision,
            pre_paint_subtree_revision: *pre_paint_subtree_revision,
            pre_paint_owner_retained: *pre_paint_owner_retained,
            children: Vec::new(),
        }));
    }

    /// Exact resolved flat-space source of ObjectPaintProperties::MaskClip.
    /// The property node adds paint_snap_offset; flat replay uses this same
    /// source rectangle before that projection, avoiding round-trip drift.
    pub(crate) fn ResolvedMaskClipRect(&self) -> Option<PaintRect> {
        let mut rect = self.mask_layers.first().map(|layer| layer.clip_rect);
        for layer in self.mask_layers.iter().skip(1) {
            rect = Some(Union(rect.as_ref().unwrap().to_owned(), &layer.clip_rect));
        }
        if let Some(mask_box) = self
            .fragment
            .as_deref()
            .and_then(|fragment| fragment.paint.style.mask_box_image.as_ref())
        {
            let bounds =
                crate::nine_piece_image_painter::NinePieceImagePainter::Bounds(self, mask_box);
            rect = Some(rect.map_or(bounds, |rect| Union(rect, &bounds)));
        }
        rect
    }

    pub(crate) fn PixelSnappedRect(&self, rect: &PaintRect) -> PaintRect {
        PixelSnappedRectInPropertySpace(rect, self.paint_snap_offset, self.pixel_snap_scale)
    }

    pub(crate) fn EnclosingRect(&self, rect: PaintRect) -> PaintRect {
        // DrawingRecorder bounds are enclosed in the same local paint space
        // as its pixels. The delta accounts for a real paint-space reset.
        EnclosingRectInPropertySpace(rect, self.paint_snap_offset)
    }

    pub(crate) fn PixelSnappedOffset(&self, offset: Offset) -> Offset {
        Offset {
            x: SnapToPixel(offset.x + self.paint_snap_offset.x, self.pixel_snap_scale)
                - self.paint_snap_offset.x,
            y: SnapToPixel(offset.y + self.paint_snap_offset.y, self.pixel_snap_scale)
                - self.paint_snap_offset.y,
        }
    }

    pub(crate) fn RoundedPaintY(&self, y: f64) -> f64 {
        SnapToPixel(y + self.paint_snap_offset.y, self.pixel_snap_scale) - self.paint_snap_offset.y
    }
}

#[path = "pre_paint_state_compare.rs"]
mod state_compare;

/// The physical parent's actual builder context. A dirty bit permits work;
/// these values decide whether that work must propagate into clean children.
#[derive(Clone)]
struct InheritedContext {
    offset: Offset,
    paint_space: Offset,
    clip: Option<PaintRect>,
    cull: Option<PaintRect>,
    transforms: Vec<TransformMatrix>,
    opacity: f32,
    scroll: Offset,
    fixed_background_acts_as_scroll: bool,
    properties: PropertyTreeState,
    rendering_context_id: u64,
    should_flatten_inherited_transform: bool,
    can_inherit_backface_visibility: bool,
    root: bool,
    flex_or_grid_item: bool,
}
#[derive(Clone)]
struct ChildContext {
    origin: Offset,
    paint_space: Offset,
    clip: Option<PaintRect>,
    cull: Option<PaintRect>,
    transforms: Vec<TransformMatrix>,
    opacity: f32,
    scroll: Offset,
    fixed_background_acts_as_scroll: bool,
    properties: PropertyTreeState,
    rendering_context_id: u64,
    should_flatten_inherited_transform: bool,
    can_inherit_backface_visibility: bool,
    flex_or_grid_items: bool,
    svg: Option<TransformMatrix>,
}
struct RootFixedPositionContext {
    clip: Option<PaintRect>,
    properties: PropertyTreeState,
    scroll_translation: Option<Arc<TransformPaintPropertyNode>>,
}
struct NodeCache {
    inherited: InheritedContext,
    children: ChildContext,
    revision: u64,
    subtree_revision: u64,
    raw_properties: PropertyTreeState,
    raw_contents: PropertyTreeState,
    raw_fixed_background: Option<crate::paint_property_tree::PropertyTreeState>,
    raw_mask: Option<crate::paint_property_tree::PropertyTreeState>,
    raw_opacity: f32,
    raw_compositing: bool,
    own_keys: Vec<(u64, PaintPropertyKey)>,
    subtree_keys: Vec<(u64, PaintPropertyKey)>,
    nodes: usize,
    computed_pass: u64,
    // Physical links survive logical OOF ownership moves. Boxes stay owned by
    // retained_tree; topology changes must use the ordinary ownership update.
    physical_children: Vec<*mut PaintTreeNode<'static>>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PrePaintUpdateStats {
    pub computed_nodes: usize,
    pub reused_nodes: usize,
    pub skipped_subtrees: usize,
    pub post_walk: bool,
}

// cpp: paint/pre_paint_tree_walk.h:82-101
pub struct PrePaintTreeWalk {
    pub(crate) next_property_id: u64,
    pub(crate) property_node_store: PaintPropertyNodeStore,
    // Completed geometry/cull state belongs to the resident paint lifecycle,
    // never to the published drawing artifact or a borrowed fragment tree.
    pub(crate) retained_scroll_state: Option<crate::scroll_paint_update::RetainedScrollPaintState>,
    property_keys: BTreeMap<u64, PaintPropertyKey>,
    reset_owners: BTreeSet<PaintPropertyOwner>,
    // The caller proved that this exact rooted native tree and exported
    // fragments survived a scroll-only update without Build/Layout/Export.
    retained_native_owner_lifetimes: bool,
    retained_tree: Option<Box<PaintTreeNode<'static>>>,
    retained_tree_valid: bool,
    logical_tree_changed: bool,
    viewport: Option<PaintRect>,
    viewport_properties: PropertyTreeState,
    root_fixed_position: Option<RootFixedPositionContext>,
    // Source: core/paint/paint_property_tree_builder.cc:4692-4701.
    requires_main_thread_for_background_attachment_fixed: Option<bool>,
    prior_property_keys: BTreeMap<u64, PaintPropertyKey>,
    needs_post_update: bool,
    pub(crate) stats: PrePaintUpdateStats,
    client_nodes: HashMap<u64, Vec<*mut PaintTreeNode<'static>>>,
    resident_nodes: Vec<*mut PaintTreeNode<'static>>,
    walk_generation: u64,
}

impl Default for PrePaintTreeWalk {
    fn default() -> Self {
        Self {
            next_property_id: 1,
            property_node_store: Default::default(),
            retained_scroll_state: None,
            property_keys: BTreeMap::new(),
            reset_owners: BTreeSet::new(),
            retained_native_owner_lifetimes: false,
            retained_tree: None,
            retained_tree_valid: false,
            logical_tree_changed: false,
            viewport: None,
            viewport_properties: PropertyTreeState::default(),
            root_fixed_position: None,
            requires_main_thread_for_background_attachment_fixed: None,
            prior_property_keys: BTreeMap::new(),
            needs_post_update: false,
            stats: PrePaintUpdateStats::default(),
            client_nodes: HashMap::new(),
            resident_nodes: Vec::new(),
            walk_generation: 0,
        }
    }
}

// cpp: paint/pre_paint_tree_walk.cc:19-27
fn Intersect(mut a: PaintRect, b: &PaintRect) -> PaintRect {
    let right = (a.x + a.width).min(b.x + b.width);
    let bottom = (a.y + a.height).min(b.y + b.height);
    a.x = a.x.max(b.x);
    a.y = a.y.max(b.y);
    a.width = (right - a.x).max(0.0);
    a.height = (bottom - a.y).max(0.0);
    a
}

// cpp: paint/pre_paint_tree_walk.cc:29-32
// Chromium desktop defaults: features.cc kCullRectPixelDistanceToExpand,
// kCullRectChangedEnoughDistance; CullRectUpdater::ExpansionRatio and
// CullRect::MinimumLocalPixelDistanceToExpand/IsSmallScroller. Record ahead of
// scrolling, while tile selection still limits raster to needed content.
fn ScrollCullExpansion(fragment: &FragmentNode, root_scroller: bool) -> f64 {
    let ratio = fragment
        .paint
        .resources
        .as_ref()
        .map_or(1.0, |resources| resources.device_pixel_ratio);
    let small =
        !root_scroller && fragment.size.width * fragment.size.height <= 100_000.0 * ratio * ratio;
    if small {
        2.0 * 512.0 * ratio
    } else {
        4000.0 * ratio
    }
}

fn Expand(rect: PaintRect, amount: f64) -> PaintRect {
    PaintRect {
        x: rect.x - amount,
        y: rect.y - amount,
        width: rect.width + 2.0 * amount,
        height: rect.height + 2.0 * amount,
    }
}

// cpp: paint/pre_paint_tree_walk.cc:34-77
pub(crate) fn SvgViewBoxTransform(fragment: &FragmentNode) -> Option<TransformMatrix> {
    (fragment.paint.source_kind == NodeKind::kSvgRoot)
        .then(|| fragment.paint.svg_viewport.map(|v| v.content_transform))
        .flatten()
}

// Geometry shared by painting and DOM box queries. Clips, effects and logical
// paint parenting do not change the physical fragment's coordinate mapping.
#[derive(Clone, Copy)]
pub(crate) struct FragmentGeometry {
    pub absolute_offset: Offset,
    pub paint_offset: Offset,
    pub paint_space_origin: Offset,
    pub applies_transform: bool,
    pub resets_paint_space: bool,
}

pub(crate) fn ResolveFragmentGeometry(
    fragment: &FragmentNode,
    parent_offset: Offset,
    mut paint_space_origin: Offset,
    transforms: &mut Vec<TransformMatrix>,
    mut property_space_delta: Offset,
) -> FragmentGeometry {
    let absolute_offset = Offset {
        x: parent_offset.x + fragment.offset.x,
        y: parent_offset.y + fragment.offset.y,
    };
    let mut geometry = FragmentGeometry {
        absolute_offset,
        paint_offset: Offset {
            x: absolute_offset.x - paint_space_origin.x,
            y: absolute_offset.y - paint_space_origin.y,
        },
        paint_space_origin,
        applies_transform: false,
        resets_paint_space: false,
    };
    let style = fragment.paint.has_source.then_some(&*fragment.paint.style);
    let establishes = fragment.paint.establishes_paint_state;
    let position = if fragment.paint.has_source {
        fragment.paint.position
    } else {
        Position::kStatic
    };
    // NeedsStickyTranslation is driven by sticky constraints, not by the
    // current offset. At scroll origin the offset is commonly zero, but the
    // identity property node is still required for impl-side scrolling.
    let has_sticky_translation = position == Position::kSticky
        && (fragment.paint.sticky_constraint.is_some()
            || fragment.paint.sticky_offset.x != 0.0
            || fragment.paint.sticky_offset.y != 0.0);
    let has_svg_viewport_translation = fragment.paint.source_kind == NodeKind::kSvgRoot
        && (geometry.paint_offset.x != 0.0 || geometry.paint_offset.y != 0.0);
    let text_combine = fragment.paint.text_combine.as_ref();
    if establishes
        && (text_combine.is_some()
            || style.is_some_and(|style| {
                has_svg_viewport_translation
                    || has_sticky_translation
                    || fragment.paint.svg_local_transform.is_some()
                    || style.transform.is_some()
                    || style.will_change_transform
                    || style.preserves_3d
                    || style.backface_visibility_hidden
                    || fragment
                        .paint
                        .svg_shape
                        .as_ref()
                        .is_some_and(|shape| shape.local_transform.is_some())
            }))
    {
        geometry.applies_transform = true;
        if has_svg_viewport_translation {
            // cpp: paint_property_tree_builder.cc
            // FragmentPaintPropertyTreeBuilder::UpdateForPaintOffsetTranslation
            // cpp: paint/svg_root_painter.cc
            // SVGRootPainter::TransformToPixelSnappedBorderBox
            //
            // Keep the residual paint offset for the SVG root's own box, but
            // establish its replaced-content coordinate space on a physical
            // pixel. SVG descendants paint at the snapped origin below.
            let scale = fragment
                .paint
                .resources
                .as_ref()
                .map_or(1.0, |resources| resources.ArtifactToDeviceScale())
                .max(f64::MIN_POSITIVE);
            let translation_offset = Offset {
                x: SnapToPixel(geometry.paint_offset.x + property_space_delta.x, scale)
                    - property_space_delta.x,
                y: SnapToPixel(geometry.paint_offset.y + property_space_delta.y, scale)
                    - property_space_delta.y,
            };
            let translation = TranslationTransform(translation_offset.x, translation_offset.y);
            transforms.push(translation);
            paint_space_origin.x += translation_offset.x;
            paint_space_origin.y += translation_offset.y;
            geometry.paint_offset.x -= translation_offset.x;
            geometry.paint_offset.y -= translation_offset.y;
            property_space_delta = Offset::default();
            geometry.resets_paint_space = true;
        }
        if has_sticky_translation {
            if !fragment.paint.sticky_offset.x.is_finite()
                || !fragment.paint.sticky_offset.y.is_finite()
            {
                panic!("sticky offset must be finite");
            }
            let translation = TranslationTransform(
                fragment.paint.sticky_offset.x,
                fragment.paint.sticky_offset.y,
            );
            transforms.push(translation);
        }
        if let Some(style) = style.filter(|style| {
            style.transform.is_some() && fragment.paint.svg_local_transform.is_none()
        }) {
            let transform = style.transform.as_ref().unwrap();
            // Blink snaps the layout paint offset in the current property
            // parent, whose ScrollTranslation remains a separate ancestor.
            let reference_rect = ResolveTransformReferenceRect(
                style.transform_reference_box,
                fragment.size,
                fragment.paint.border,
                fragment.paint.padding,
            );
            let resolved = ResolveLocalTransformAroundReferenceBox(
                transform,
                Offset {
                    x: geometry.paint_offset.x + property_space_delta.x,
                    y: geometry.paint_offset.y + property_space_delta.y,
                },
                reference_rect,
                style.transform_origin.as_ref(),
            );
            transforms.push(MultiplyTransforms(
                &TranslationTransform(-property_space_delta.x, -property_space_delta.y),
                &resolved,
            ));
            paint_space_origin = geometry.absolute_offset;
            geometry.paint_offset = Offset::default();
            geometry.resets_paint_space = true;
        }
        if let Some(transform) = fragment.paint.svg_local_transform {
            transforms.push(transform);
            paint_space_origin = geometry.absolute_offset;
            geometry.paint_offset = Offset::default();
            geometry.resets_paint_space = true;
        }
        if let Some(local_transform) = fragment
            .paint
            .svg_shape
            .as_ref()
            .and_then(|shape| shape.local_transform)
        {
            transforms.push(local_transform);
        }
        if let Some(text_combine) = text_combine {
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

pub(crate) fn ChildGeometryOrigin(fragment: &FragmentNode, absolute_offset: Offset) -> Offset {
    if fragment.paint.establishes_paint_state
        && (fragment.paint.scroll_offset.x != 0.0 || fragment.paint.scroll_offset.y != 0.0)
    {
        Offset {
            x: absolute_offset.x - fragment.paint.scroll_offset.x,
            y: absolute_offset.y - fragment.paint.scroll_offset.y,
        }
    } else {
        absolute_offset
    }
}

pub(crate) fn ResolveChildGeometryOrigin(
    child: &FragmentNode,
    child_origin: Offset,
    paint_space_origin: Offset,
) -> Offset {
    if child.paint.has_source
        && matches!(
            child.paint.source_kind,
            NodeKind::kSvgShape | NodeKind::kSvgGroup | NodeKind::kSvgViewportContainer
        )
    {
        Offset {
            // SVG child geometry is expressed in the replaced content's local
            // coordinates. Chromium resets the paint offset after installing
            // ReplacedContentTransform, so descendants start at the snapped
            // content origin instead of reintroducing the root's residual.
            x: paint_space_origin.x - child.offset.x,
            y: paint_space_origin.y - child.offset.y,
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
fn FlatToPropertySpaceOffset(
    flat: &[TransformMatrix],
    property: &Arc<TransformPaintPropertyNode>,
) -> Option<Offset> {
    let mut parents = Vec::new();
    let mut current = Some(property);
    while let Some(node) = current {
        parents.push(node.matrix);
        current = node.parent.as_ref();
    }
    let mut property_world = TransformMatrix::default();
    for matrix in parents.iter().rev() {
        property_world = MultiplyTransforms(&property_world, matrix);
    }
    FlatToWorldSpaceOffset(flat, &property_world)
}

pub(crate) fn FlatToWorldSpaceOffset(
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
// cpp: paint/pre_paint_tree_walk.cc:79-85
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

pub(crate) fn EnclosingRectInPropertySpace(rect: PaintRect, delta: Offset) -> PaintRect {
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

// cpp: paint/pre_paint_tree_walk.cc:87-93
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

// cpp: paint/pre_paint_tree_walk.cc:95-106
fn FilterCullOutset(filters: &[PaintFilterOperation], size: Size) -> f64 {
    let mut outset = 0.0;
    for filter in filters {
        if filter.r#type == PaintFilterType::kBlur {
            outset += filter.amount * 3.0;
        } else if filter.r#type == PaintFilterType::kDropShadow {
            outset += filter.blur_radius * 3.0 + filter.offset.x.abs().max(filter.offset.y.abs());
        } else if filter.r#type == PaintFilterType::kReference {
            let Some(graph) = filter.reference_graph.as_ref() else {
                continue;
            };
            let unit = |value: f64, x_axis: bool| {
                if graph.primitive_units
                    == layoutng_assembly::internal::paint_input::SvgFilterUnits::kObjectBoundingBox
                {
                    value * if x_axis { size.width } else { size.height }
                } else {
                    value
                }
            };
            let mut graph_outset = size.width.max(size.height) * 0.1;
            for primitive in &graph.primitives {
                graph_outset = graph_outset.max(match &primitive.kind {
                    SvgFilterPrimitiveKind::kGaussianBlur {
                        sigma_x, sigma_y, ..
                    } => 3.0 * unit(*sigma_x, true).abs().max(unit(*sigma_y, false).abs()),
                    SvgFilterPrimitiveKind::kOffset { dx, dy } => {
                        unit(*dx, true).abs().max(unit(*dy, false).abs())
                    }
                    SvgFilterPrimitiveKind::kMorphology {
                        radius_x, radius_y, ..
                    } => unit(*radius_x, true)
                        .abs()
                        .max(unit(*radius_y, false).abs()),
                    SvgFilterPrimitiveKind::kDisplacementMap { scale, .. } => {
                        unit(*scale, true).abs().max(unit(*scale, false).abs())
                    }
                    SvgFilterPrimitiveKind::kConvolveMatrix {
                        order_x,
                        order_y,
                        target_x,
                        target_y,
                        kernel_unit_x,
                        kernel_unit_y,
                        ..
                    } => ((order_x.saturating_sub(1).max(*target_x) as f64)
                        * unit(*kernel_unit_x, true))
                    .abs()
                    .max(
                        ((order_y.saturating_sub(1).max(*target_y) as f64)
                            * unit(*kernel_unit_y, false))
                        .abs(),
                    ),
                    _ => 0.0,
                });
            }
            outset += graph_outset;
        } else if filter.r#type == PaintFilterType::kBoxReflect {
            let basis = match filter.reflection_direction {
                PaintReflectionDirection::kAbove | PaintReflectionDirection::kBelow => size.height,
                PaintReflectionDirection::kLeft | PaintReflectionDirection::kRight => size.width,
            };
            let gap = ResolveReflectionOffset(filter, basis);
            // MapRect unions the original and reflected boxes.  The existing
            // scalar cull representation must conservatively cover either
            // physical axis.
            outset += basis + gap.abs();
        }
    }
    outset
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

// cpp: paint/pre_paint_tree_walk.cc:108-118
fn ClipsOverflow(node: &FragmentNode) -> bool {
    if !node.paint.has_source || !node.paint.establishes_paint_state {
        return false;
    }
    if node.paint.source_kind == NodeKind::kSvgViewportContainer {
        return node.paint.svg_viewport.is_some_and(|v| v.clip.is_some());
    }
    node.paint.svg_viewport.is_some_and(|v| v.disabled)
        || node.paint.overflow_x != Overflow::kVisible
        || node.paint.overflow_y != Overflow::kVisible
}

// cpp: paint/pre_paint_tree_walk.cc:120-146
fn ValidateFilters(filters: &[PaintFilterOperation]) {
    for filter in filters {
        if !filter.amount.is_finite()
            || !filter.offset.x.is_finite()
            || !filter.offset.y.is_finite()
            || !filter.blur_radius.is_finite()
            || filter.blur_radius < 0.0
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
            let reference = NinePieceImagePainter::ReflectionReferenceRect(node);
            let stitched_origin =
                fragment
                    .paint
                    .stitched_decoration
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
                    NinePieceImagePainter::ResolveReflectionMask(node, &image);
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
                pixels,
                ..
            } = &mut primitive.kind
            else {
                continue;
            };
            let Some(id) = *resource_id else { continue };
            let Some(resources) = fragment.paint.resources.as_ref() else {
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
                *pixels = Arc::clone(bitmap)
            } else {
                graph.unsupported.get_or_insert(
                    "feImage document image recording is not raster-resolved".into(),
                );
            }
        }
    }
    // Keep masked reflection explicit. Layer lowering rejects this typed state
    // and flat replay skips only the reflection branch; neither path may
    // reinterpret it as the visually much stronger unmasked operation.
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
        top_left: layoutng_assembly::internal::paint_input::PaintCornerRadius {
            x: radius_x,
            y: radius_y,
        },
        top_right: layoutng_assembly::internal::paint_input::PaintCornerRadius {
            x: radius_x,
            y: radius_y,
        },
        bottom_right: layoutng_assembly::internal::paint_input::PaintCornerRadius {
            x: radius_x,
            y: radius_y,
        },
        bottom_left: layoutng_assembly::internal::paint_input::PaintCornerRadius {
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
        let radius = |value: layoutng_assembly::internal::paint_input::ClipPathRadius| {
            layoutng_assembly::internal::paint_input::PaintCornerRadius {
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
    transform: layoutng_assembly::internal::paint_input::SvgClipTransform,
) -> Offset {
    let [a, b, c, d, e, f] = transform.values;
    Offset {
        x: a * point.x + c * point.y + e,
        y: b * point.x + d * point.y + f,
    }
}
fn MultiplyClipTransform(
    a: layoutng_assembly::internal::paint_input::SvgClipTransform,
    b: layoutng_assembly::internal::paint_input::SvgClipTransform,
) -> layoutng_assembly::internal::paint_input::SvgClipTransform {
    let [a0, a1, a2, a3, a4, a5] = a.values;
    let [b0, b1, b2, b3, b4, b5] = b.values;
    layoutng_assembly::internal::paint_input::SvgClipTransform {
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
    mut transform: layoutng_assembly::internal::paint_input::SvgClipTransform,
    width: f64,
    height: f64,
) -> layoutng_assembly::internal::paint_input::SvgClipTransform {
    transform.values[4] += transform.percentages[0] * width + transform.percentages[1] * height;
    transform.values[5] += transform.percentages[2] * width + transform.percentages[3] * height;
    transform.percentages = [0.0; 4];
    transform
}
fn ResolveSvgClipGeometry(
    geometry: &layoutng_assembly::internal::paint_input::SvgClipGeometry,
    width: f64,
    height: f64,
) -> Vec<PaintPathCommand> {
    use layoutng_assembly::internal::paint_input::{SvgClipGeometry as G, SvgClipLength};
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
    viewport: layoutng_assembly::internal::paint_input::SvgClipViewport,
    reference_width: f64,
    reference_height: f64,
) -> layoutng_assembly::internal::paint_input::SvgClipTransform {
    use layoutng_assembly::internal::paint_input::{
        SvgClipPreserveAspectRatio as P, SvgClipTransform,
    };
    let resolve = |length: layoutng_assembly::internal::paint_input::SvgClipLength, basis| {
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

pub(crate) fn ResolveSvgClipText(
    text: &layoutng_assembly::internal::paint_input::SvgClipText,
    fonts: &[layoutng_assembly::internal::layout_input::FontFace],
    geometry_size: (f64, f64),
    cursor: &mut Offset,
) -> Result<Vec<Vec<PaintPathCommand>>, String> {
    use font_engine::{ClipTextFace, ClipTextRequest, OutlineVerb, TextDirection};
    use layoutng_assembly::internal::paint_input::SvgClipTextAnchor;
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
    let resolve = |values: &[layoutng_assembly::internal::paint_input::SvgClipLength],
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
pub(crate) fn ResolveReferenceClipPath(
    graph: &layoutng_assembly::internal::paint_input::SvgClipPathGraph,
    reference: PaintRect,
    paint_offset: Offset,
    effective_zoom: f64,
    svg_owner: bool,
    resource_viewport: Option<Size>,
    fonts: &[layoutng_assembly::internal::layout_input::FontFace],
    runtime_error: &mut Option<String>,
    output: &mut Vec<PaintPathCommand>,
    children: &mut Vec<PaintClipPathChild>,
) -> PaintRect {
    use layoutng_assembly::internal::paint_input::{
        SvgClipPathUnits, SvgClipTransform, SvgClipTransformStep,
    };
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
    fragment: &FragmentNode,
    paint_offset: Offset,
    reference_box: ClipPathReferenceBox,
) -> PaintRect {
    let paint = &fragment.paint;
    if IsSvgEffectChild(paint.source_kind) {
        let fill = paint.svg_shape.as_ref().map_or(
            PaintRect {
                x: paint_offset.x,
                y: paint_offset.y,
                width: fragment.size.width,
                height: fragment.size.height,
            },
            |shape| PaintRect {
                x: paint_offset.x + shape.bounds_offset.x,
                y: paint_offset.y + shape.bounds_offset.y,
                width: shape.bounds_width,
                height: shape.bounds_height,
            },
        );
        return match reference_box {
            ClipPathReferenceBox::kStrokeBox => {
                let has_stroke = paint.style.svg_stroke.is_some()
                    || paint.style.svg_stroke_server.is_some()
                    || paint.style.svg_stroke_reference.is_some();
                let mut outset = if has_stroke {
                    paint.style.svg_stroke_width.max(0.0) * 0.5
                } else {
                    0.0
                };
                if outset > 0.0
                    && paint.svg_shape.as_ref().is_some_and(|shape| {
                        shape.geometry
                            == layoutng_assembly::internal::layout_input::SvgShapeGeometry::kPath
                    })
                {
                    let shape = paint.svg_shape.as_ref().unwrap();
                    let drawing_segments = shape
                        .path
                        .iter()
                        .filter(|command| {
                            !matches!(command.verb, PaintPathVerb::kMoveTo | PaintPathVerb::kClose)
                        })
                        .count();
                    if drawing_segments > 1
                        && paint.style.svg_stroke_line_join
                            == layoutng_assembly::internal::paint_input::SvgStrokeLineJoin::kMiter
                    {
                        let miter = paint.style.svg_stroke_miter_limit.max(1.0);
                        if miter < 2.0_f64.sqrt()
                            && paint.style.svg_stroke_line_cap
                                == layoutng_assembly::internal::paint_input::SvgStrokeLineCap::kSquare
                        {
                            outset *= 2.0_f64.sqrt();
                        } else {
                            outset *= miter;
                        }
                    } else if paint.style.svg_stroke_line_cap
                        == layoutng_assembly::internal::paint_input::SvgStrokeLineCap::kSquare
                    {
                        outset *= 2.0_f64.sqrt();
                    }
                }
                PaintRect {
                    x: fill.x - outset,
                    y: fill.y - outset,
                    width: fill.width + outset * 2.0,
                    height: fill.height + outset * 2.0,
                }
            }
            ClipPathReferenceBox::kViewBox => paint.svg_view_box.map_or(
                PaintRect {
                    x: paint_offset.x,
                    y: paint_offset.y,
                    width: fragment.size.width,
                    height: fragment.size.height,
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

    let border = IncludedBorderEdges(paint);
    let border_box = PaintRect {
        x: paint_offset.x,
        y: paint_offset.y,
        width: fragment.size.width,
        height: fragment.size.height,
    };
    match reference_box {
        ClipPathReferenceBox::kPaddingBox => PaintRect {
            x: border_box.x + border.left,
            y: border_box.y + border.top,
            width: (border_box.width - border.left - border.right).max(0.0),
            height: (border_box.height - border.top - border.bottom).max(0.0),
        },
        ClipPathReferenceBox::kContentBox | ClipPathReferenceBox::kFillBox => PaintRect {
            x: border_box.x + border.left + paint.padding.left,
            y: border_box.y + border.top + paint.padding.top,
            width: fragment.content_size.width,
            height: fragment.content_size.height,
        },
        ClipPathReferenceBox::kMarginBox => PaintRect {
            x: border_box.x - paint.margin.left,
            y: border_box.y - paint.margin.top,
            width: border_box.width + paint.margin.left + paint.margin.right,
            height: border_box.height + paint.margin.top + paint.margin.bottom,
        },
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

// cpp: paint/pre_paint_tree_walk.cc:220-241
fn MaskBox(fragment: &FragmentNode, absolute_offset: Offset, box_type: BackgroundBox) -> PaintRect {
    let box_type = UsedMaskGeometryBox(box_type);
    let paint = &fragment.paint;
    let mut rect = PaintRect {
        x: absolute_offset.x,
        y: absolute_offset.y,
        width: fragment.size.width,
        height: fragment.size.height,
    };
    if let Some(table) = paint.table.as_ref() {
        rect = PaintRect {
            x: absolute_offset.x + table.grid_offset.x,
            y: absolute_offset.y + table.grid_offset.y,
            width: table.grid_size.width,
            height: table.grid_size.height,
        };
    }
    if box_type == BackgroundBox::kBorderBox {
        return rect;
    }
    let border = IncludedBorderEdges(paint);
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
        x: rect.x + paint.padding.left,
        y: rect.y + paint.padding.top,
        width: fragment.content_size.width,
        height: fragment.content_size.height,
    }
}

// cpp: paint/pre_paint_tree_walk.cc:243-269
fn MaskPositioningBox(
    fragment: &FragmentNode,
    absolute_offset: Offset,
    box_type: BackgroundBox,
) -> PaintRect {
    let box_type = UsedMaskGeometryBox(box_type);
    let Some(stitched) = fragment.paint.stitched_decoration.as_ref() else {
        return MaskBox(fragment, absolute_offset, box_type);
    };
    let paint = &fragment.paint;
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
        x: rect.x + paint.border.left,
        y: rect.y + paint.border.top,
        width: (rect.width - paint.border.left - paint.border.right).max(0.0),
        height: (rect.height - paint.border.top - paint.border.bottom).max(0.0),
    };
    if box_type == BackgroundBox::kPaddingBox {
        return rect;
    }
    PaintRect {
        x: rect.x + paint.padding.left,
        y: rect.y + paint.padding.top,
        width: (rect.width - paint.padding.left - paint.padding.right).max(0.0),
        height: (rect.height - paint.padding.top - paint.padding.bottom).max(0.0),
    }
}

// cpp: paint/pre_paint_tree_walk.cc:271-286
fn MaskClipRadii(fragment: &FragmentNode, box_type: BackgroundBox) -> PaintCornerRadii {
    let box_type = UsedMaskGeometryBox(box_type);
    let paint = &fragment.paint;
    let mut inset = Edges::default();
    if box_type != BackgroundBox::kBorderBox {
        inset = IncludedBorderEdges(paint);
    }
    if box_type == BackgroundBox::kContentBox {
        inset.top += paint.padding.top;
        inset.right += paint.padding.right;
        inset.bottom += paint.padding.bottom;
        inset.left += paint.padding.left;
    }
    let outer_size = paint
        .table
        .as_ref()
        .map_or(fragment.size, |table| table.grid_size);
    ResolveCornerRadii(&paint.style, &paint.border_sides, outer_size, inset)
}

fn UsedMaskGeometryBox(box_type: BackgroundBox) -> BackgroundBox {
    match box_type {
        BackgroundBox::kPaddingBox => BackgroundBox::kPaddingBox,
        BackgroundBox::kContentBox => BackgroundBox::kContentBox,
        BackgroundBox::kBorderBox
        | BackgroundBox::kText
        | BackgroundBox::kBorderArea
        | BackgroundBox::kBorderAreaText => BackgroundBox::kBorderBox,
    }
}

// cpp: paint/pre_paint_tree_walk.cc:288-304
fn FindMaskImage(fragment: &FragmentNode, id: u64) -> &PaintImage {
    let Some(resources) = fragment.paint.resources.as_ref() else {
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

// cpp: paint/pre_paint_tree_walk.cc:306-358
#[cfg(feature = "translation_in_progress")]
fn ResolveMaskLayers(
    fragment: &FragmentNode,
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
                if IsSvgEffectChild(fragment.paint.source_kind) {
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
                use layoutng_assembly::internal::paint_input::SvgMaskUnits;
                if IsSvgEffectChild(fragment.paint.source_kind)
                    && fragment.paint.svg_shape.is_none()
                    && (graph.mask_units == SvgMaskUnits::kObjectBoundingBox
                        || graph.content_units == SvgMaskUnits::kObjectBoundingBox)
                {
                    resolved.source_error =
                        Some("SVG group object bounding box for local mask".into());
                }
                if resolved.source_error.is_none() {
                    if let Some(viewport) = reference.viewport {
                        let zoom = if IsSvgEffectChild(fragment.paint.source_kind) {
                            1.0
                        } else {
                            f64::from(fragment.paint.effective_zoom)
                        };
                        let (region, shader) = crate::paint_shader_resolver::ResolveLocalSvgMask(
                            graph,
                            object,
                            absolute_offset,
                            zoom,
                            viewport,
                            fragment.paint.resources.as_deref(),
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
        let positioning = MaskPositioningBox(fragment, absolute_offset, layer.image.origin);
        let clip = MaskBox(fragment, absolute_offset, layer.image.clip);
        let image =
            if layer.image_state == PaintMaskImageState::kReady && layer.image.shader.is_none() {
                Some(FindMaskImage(fragment, layer.image.resource_id))
            } else {
                None
            };
        let tile_size = ResolveBackgroundTileSize(
            &layer.image,
            &positioning,
            image,
            f64::from(fragment.paint.effective_zoom),
        );
        let tile_geometry = ResolveBackgroundTile(&layer.image, &positioning, tile_size);
        let tile = if layer.image.shader.is_some() {
            PixelSnappedRectInPropertySpace(
                &tile_geometry.tile_rect,
                property_space_delta,
                pixel_snap_scale,
            )
        } else {
            tile_geometry.tile_rect
        };
        let bitmap_tile_scale = if let Some(image) = image {
            Offset {
                x: tile.width / image.width as f64,
                y: tile.height / image.height as f64,
            }
        } else {
            tile_geometry.tile_scale
        };
        let mut resolved = DisplayMaskLayer {
            resource_id: image.map_or(0, |image| image.id),
            image_interpolation_quality: fragment.paint.style.image_interpolation_quality,
            document_image_container_key: image.and_then(|image| {
                ResolveDocumentImageContainerKey(
                    image,
                    Size {
                        width: tile.width,
                        height: tile.height,
                    },
                    f64::from(fragment.paint.effective_zoom),
                    fragment.paint.resources.as_deref(),
                    (!fragment.paint.style.color_scheme_flags_is_normal)
                        .then_some(u8::from(fragment.paint.style.used_dark_color_scheme)),
                )
            }),
            clip_rect: clip,
            clip_radii: MaskClipRadii(fragment, layer.image.clip),
            source_rect: image.map_or(PaintRect::default(), |image| PaintRect {
                x: 0.0,
                y: 0.0,
                width: image.width as f64,
                height: image.height as f64,
            }),
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
                fragment.paint.resources.as_deref(),
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

// Most sources have one physical occurrence. Store it inline rather than
// allocating a Vec for every source on every paint traversal.
struct LogicalParentOccurrences<'a> {
    first: *mut PaintTreeNode<'a>,
    remaining: Vec<*mut PaintTreeNode<'a>>,
}
impl<'a> LogicalParentOccurrences<'a> {
    fn iter(&self) -> impl Iterator<Item = *mut PaintTreeNode<'a>> + '_ {
        std::iter::once(self.first).chain(self.remaining.iter().copied())
    }
}

// cpp: paint/pre_paint_tree_walk.cc:360-378
fn GatherLogicalParents<'a>(
    node: &mut PaintTreeNode<'a>,
    source_nodes: &mut HashMap<u64, LogicalParentOccurrences<'a>>,
    fragment_instances: &mut HashMap<u64, *mut PaintTreeNode<'a>>,
) {
    let node_pointer = node as *mut PaintTreeNode<'a>;
    let fragment = node
        .fragment
        .as_deref()
        .expect("PrePaint node must own a fragment");
    if fragment.fragment_instance_id != 0 {
        if fragment_instances
            .insert(fragment.fragment_instance_id, node_pointer)
            .is_some()
        {
            panic!("FragmentTree contains duplicate fragment_instance_id values");
        }
    }
    if fragment.node_id != 0 && fragment.paint.has_source && fragment.paint.establishes_paint_state
    {
        match source_nodes.entry(fragment.node_id) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(LogicalParentOccurrences {
                    first: node_pointer,
                    remaining: Vec::new(),
                });
            }
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                entry.get_mut().remaining.push(node_pointer);
            }
        }
    }
    for child in &mut node.children {
        GatherLogicalParents(child, source_nodes, fragment_instances);
    }
}

// cpp: paint/pre_paint_tree_walk.cc:380-383
struct DetachedPaintNode<'a> {
    logical_parent: *mut PaintTreeNode<'a>,
    node: Box<PaintTreeNode<'a>>,
}

// cpp: paint/pre_paint_tree_walk.cc:385-396
fn DistanceSquaredToFragment(candidate: &PaintTreeNode<'_>, child: &PaintTreeNode<'_>) -> f64 {
    let left = candidate.absolute_offset.x;
    let top = candidate.absolute_offset.y;
    let fragment = candidate
        .fragment
        .as_deref()
        .expect("PrePaint node must own a fragment");
    let right = left + fragment.size.width;
    let bottom = top + fragment.size.height;
    let x = child.absolute_offset.x;
    let y = child.absolute_offset.y;
    let dx = if x < left {
        left - x
    } else if x > right {
        x - right
    } else {
        0.0
    };
    let dy = if y < top {
        top - y
    } else if y > bottom {
        y - bottom
    } else {
        0.0
    };
    dx * dx + dy * dy
}

// cpp: paint/pre_paint_tree_walk.cc:398-422
fn ResolveLogicalParentOccurrence<'a>(
    child: &PaintTreeNode<'a>,
    candidates: &LogicalParentOccurrences<'a>,
) -> Option<*mut PaintTreeNode<'a>> {
    let fragmentainer = child
        .fragment
        .as_deref()
        .expect("PrePaint node must own a fragment")
        .fragmentainer_instance_id;
    // Reuse the same stable boxed pointers and candidate order, without a
    // temporary Vec of matching fragmentainers for each lookup.
    let has_same = candidates.iter().any(|candidate| {
        unsafe { &*candidate }
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment")
            .fragmentainer_instance_id
            == fragmentainer
    });
    let mut eligible = candidates.iter().filter(|&candidate| {
        !has_same
            || unsafe { &*candidate }
                .fragment
                .as_deref()
                .expect("PrePaint node must own a fragment")
                .fragmentainer_instance_id
                == fragmentainer
    });
    let mut smallest = eligible.next()?;
    for candidate in eligible {
        let candidate_ref = unsafe { &*candidate };
        let smallest_ref = unsafe { &*smallest };
        let candidate_distance = DistanceSquaredToFragment(candidate_ref, child);
        let smallest_distance = DistanceSquaredToFragment(smallest_ref, child);
        let candidate_precedes = if candidate_distance != smallest_distance {
            candidate_distance < smallest_distance
        } else {
            candidate_ref
                .fragment
                .as_deref()
                .expect("PrePaint node must own a fragment")
                .fragment_instance_id
                < smallest_ref
                    .fragment
                    .as_deref()
                    .expect("PrePaint node must own a fragment")
                    .fragment_instance_id
        };
        if candidate_precedes {
            smallest = candidate;
        }
    }
    Some(smallest)
}

// cpp: paint/pre_paint_tree_walk.cc:424-477
fn DetachLogicallyReparentedOutOfFlowNodes<'a>(
    parent: &mut PaintTreeNode<'a>,
    source_nodes: &HashMap<u64, LogicalParentOccurrences<'a>>,
    fragment_instances: &HashMap<u64, *mut PaintTreeNode<'a>>,
    physical_ancestors: &mut Vec<*const PaintTreeNode<'a>>,
    detached: &mut Vec<DetachedPaintNode<'a>>,
) {
    physical_ancestors.push(parent as *const PaintTreeNode<'a>);
    let mut index = 0;
    while index < parent.children.len() {
        let child = parent.children[index].as_mut();
        let fragment = child
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        let logical_id = fragment.paint.logical_parent_node_id;
        let logical_instance_id = fragment.paint.logical_parent_fragment_instance_id;
        let mut logical_parent: Option<*mut PaintTreeNode<'a>> = None;
        if let Some(logical_instance_id) = logical_instance_id {
            let Some(&exact) = fragment_instances.get(&logical_instance_id) else {
                panic!("logical_parent_fragment_instance_id does not name a Fragment");
            };
            logical_parent = Some(exact);
            if let Some(logical_id) = logical_id {
                let logical_parent_fragment = unsafe { &*exact }
                    .fragment
                    .as_deref()
                    .expect("PrePaint node must own a fragment");
                if logical_parent_fragment.node_id != logical_id {
                    panic!("logical parent node id and fragment instance id disagree");
                }
            }
        } else if let Some(logical_id) = logical_id {
            if let Some(candidates) = source_nodes.get(&logical_id) {
                logical_parent = ResolveLogicalParentOccurrence(child, candidates);
            }
        }
        let logical_parent_is_physical_ancestor = logical_parent.is_some_and(|pointer| {
            physical_ancestors.contains(&(pointer as *const PaintTreeNode<'a>))
        });
        let position = fragment.paint.position;
        let is_out_of_flow = position == Position::kAbsolute || position == Position::kFixed;
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
            source_nodes,
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
pub(crate) fn MaskAdjustedOpacity(style: &PaintStyleData) -> f32 {
    let any_loading = style
        .mask_images
        .iter()
        .any(|layer| layer.image_state == PaintMaskImageState::kLoading);
    let all_invalid = !style.mask_images.is_empty()
        && style
            .mask_images
            .iter()
            .all(|layer| layer.image_state != PaintMaskImageState::kReady);
    let has_backdrop = !style.backdrop_filters.is_empty() || style.backdrop_filter_has_reference;
    if any_loading || (has_backdrop && all_invalid) {
        0.0
    } else {
        style.opacity
    }
}

// cpp: paint/pre_paint_tree_walk.cc:479-496
fn RebindLogicalEffectState(
    node: &mut PaintTreeNode<'_>,
    parent_opacity: f32,
    parent_effect_id: u64,
    parent_effect: Arc<EffectPaintPropertyNode>,
) {
    let fragment = node
        .fragment
        .as_deref()
        .expect("PrePaint node must own a fragment");
    let style = fragment.paint.has_source.then_some(&*fragment.paint.style);
    let local_opacity = if fragment.paint.establishes_paint_state {
        style.map_or(1.0, MaskAdjustedOpacity)
    } else {
        1.0
    };
    node.opacity = parent_opacity * local_opacity;
    if !node.applies_effect {
        node.properties.effect_id = parent_effect_id;
    }
    // Geometry still follows the physical ancestry. Effects inherit from the
    // restored logical ancestry; rebuild only this effect's parent reference.
    let local_effect_count = usize::from(
        node.applies_opacity
            || node.applies_blend
            || node.applies_backdrop_filter
            || node.applies_mask,
    ) + usize::from(node.applies_filter);
    fn ReparentLocalEffect(
        effect: &Arc<EffectPaintPropertyNode>,
        count: usize,
        parent: Arc<EffectPaintPropertyNode>,
    ) -> Arc<EffectPaintPropertyNode> {
        if count == 0 {
            return parent;
        }
        let mut state = (**effect).clone();
        state.parent = Some(ReparentLocalEffect(
            state.parent.as_ref().expect("local effect parent"),
            count - 1,
            parent,
        ));
        Arc::new(state)
    }
    node.properties.nodes.effect = ReparentLocalEffect(
        &node.properties.nodes.effect,
        local_effect_count,
        parent_effect,
    );
    if let Some(fixed) = &mut node.fixed_background_properties {
        fixed.effect = node.properties.nodes.effect.clone();
    }
    if let Some(mask) = &mut node.mask_properties {
        let parent_id = mask.effect.parent.as_ref().expect("Mask content effect").id;
        let mut content = node.properties.nodes.effect.clone();
        while content.id != parent_id {
            content = content
                .parent
                .as_ref()
                .expect("Mask parent belongs to content effect chain")
                .clone();
        }
        let mut effect = (*mask.effect).clone();
        effect.parent = Some(content);
        mask.effect = Arc::new(effect);
    }
    node.contents_properties.effect_id = node.properties.effect_id;
    node.contents_properties.nodes.effect = node.properties.nodes.effect.clone();
    for child in &mut node.children {
        RebindLogicalEffectState(
            child,
            node.opacity,
            node.properties.effect_id,
            node.properties.nodes.effect.clone(),
        );
    }
}

// cpp: paint/pre_paint_tree_walk.cc:498-508
fn FirstLogicalTreeOrder(node: &PaintTreeNode<'_>) -> u64 {
    let order = node
        .fragment
        .as_deref()
        .expect("PrePaint node must own a fragment")
        .paint
        .logical_tree_order;
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

// cpp: paint/pre_paint_tree_walk.cc:510-524
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

// cpp: paint/pre_paint_tree_walk.cc:526-552
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

// cpp: paint/pre_paint_tree_walk.cc:554-568
fn RestoreLogicalOutOfFlowParents(root: &mut PaintTreeNode<'_>) -> bool {
    let mut source_nodes = HashMap::new();
    let mut fragment_instances = HashMap::new();
    GatherLogicalParents(root, &mut source_nodes, &mut fragment_instances);
    let mut physical_ancestors = Vec::new();
    let mut detached = Vec::new();
    DetachLogicallyReparentedOutOfFlowNodes(
        root,
        &source_nodes,
        &fragment_instances,
        &mut physical_ancestors,
        &mut detached,
    );
    let reparented = !detached.is_empty();
    for mut entry in detached {
        // Pointers target boxed nodes: moving a Box owner never moves its
        // allocation. All detached owners stay live through this loop.
        let logical_parent = unsafe { &mut *entry.logical_parent };
        RebindLogicalEffectState(
            &mut entry.node,
            logical_parent.opacity,
            logical_parent.properties.effect_id,
            logical_parent.properties.nodes.effect.clone(),
        );
        InsertInLogicalTreeOrder(logical_parent, entry.node);
    }
    RestoreLogicalSiblingOrder(root) || reparented
}

// cpp: paint/pre_paint_tree_walk.cc:570
type InlineOutlineKey = (u64, u64);

// cpp: paint/pre_paint_tree_walk.cc:572-588
fn GatherInlineOutlineGroups<'a>(
    node: &mut PaintTreeNode<'a>,
    groups: &mut BTreeMap<InlineOutlineKey, Vec<*mut PaintTreeNode<'a>>>,
) {
    let fragment = node
        .fragment
        .as_deref()
        .expect("PrePaint node must own a fragment");
    if fragment.node_id != 0
        && fragment.kind == FragmentKind::kBox
        && fragment.paint.has_source
        && fragment.paint.establishes_paint_state
        && fragment.paint.display == Display::kInline
        && fragment.paint.style.visible
        && fragment.paint.style.outline_width > 0.0
        && fragment.paint.style.outline_style != BorderLineStyle::kNone
        && fragment.paint.style.outline_color.alpha > 0.0
    {
        groups
            .entry((fragment.node_id, fragment.fragmentainer_instance_id))
            .or_default()
            .push(node as *mut PaintTreeNode<'a>);
    }
    for child in &mut node.children {
        GatherInlineOutlineGroups(child, groups);
    }
}

// cpp: paint/pre_paint_tree_walk.cc:590-605
fn PrepareInlineOutlines(root: &mut PaintTreeNode<'_>) {
    let mut groups = BTreeMap::new();
    GatherInlineOutlineGroups(root, &mut groups);
    for (_unused_key, occurrences) in groups {
        let owner_pointer = occurrences[0];
        unsafe { (*owner_pointer).outline_rects.reserve(occurrences.len()) };
        for occurrence_pointer in occurrences {
            let occurrence = unsafe { &*occurrence_pointer };
            let fragment = occurrence
                .fragment
                .as_deref()
                .expect("PrePaint node must own a fragment");
            let rect = PaintRect {
                x: occurrence.paint_offset.x,
                y: occurrence.paint_offset.y,
                width: fragment.size.width,
                height: fragment.size.height,
            };
            unsafe { (*owner_pointer).outline_rects.push(rect) };
            if occurrence_pointer != owner_pointer {
                unsafe { (*occurrence_pointer).paints_outline = false };
            }
        }
    }
}

// cpp: paint/pre_paint_tree_walk.cc:607-619
fn MarkOverlapCompositingLayers(node: &mut PaintTreeNode<'_>) -> bool {
    let fragment = node
        .fragment
        .as_deref()
        .expect("PrePaint node must own a fragment");
    // cpp: core/paint/paint_layer_painter.cc:800-809
    // Empty phases need neither a subtree walk nor replay geometry scopes.
    // Aggregate after logical reparenting, preserving all new cull decisions.
    node.needs_float_phase =
        fragment.paint.has_source && fragment.paint.floating != FloatSide::kNone;
    let style = &*fragment.paint.style;
    node.needs_outline_phase = fragment
        .paint
        .form_control
        .as_ref()
        .is_some_and(|control| control.auto_focus_ring)
        || (style.outline_style != BorderLineStyle::kNone
            && !(style.outline_width <= 0.0)
            && !(style.outline_color.alpha <= 0.0));
    node.needs_overflow_controls_phase = fragment.paint.scrollbars.is_some();
    let mut has_composited_descendant = false;
    for child in &mut node.children {
        has_composited_descendant |= MarkOverlapCompositingLayers(child);
        node.needs_float_phase |= child.needs_float_phase;
        node.needs_outline_phase |= child.needs_outline_phase;
        node.needs_overflow_controls_phase |= child.needs_overflow_controls_phase;
    }
    if node.applies_overflow_clip && has_composited_descendant {
        node.applies_overlap_compositing_layer = true;
        has_composited_descendant = false;
    }
    node.applies_compositing_layer || has_composited_descendant
}

fn GatherFinalEffects(
    node: &PaintTreeNode<'_>,
    effects: &mut BTreeMap<u64, Arc<EffectPaintPropertyNode>>,
) {
    // Include only this owner's slots, not inherited physical effect snapshots.
    let count = usize::from(
        node.applies_opacity
            || node.applies_blend
            || node.applies_backdrop_filter
            || node.applies_mask,
    ) + usize::from(node.applies_filter);
    let mut effect = node.properties.nodes.effect.clone();
    for _ in 0..count {
        effects.insert(effect.id, effect.clone());
        effect = effect.parent.as_ref().expect("local effect parent").clone();
    }
    if let Some(mask) = &node.mask_properties {
        effects.insert(mask.effect.id, mask.effect.clone());
    }
    for child in &node.children {
        GatherFinalEffects(child, effects);
    }
}

// Update snapshots only after logical effect reparenting. Physical transform
// and clip chains remain intact; updating the temporary physical effect parent
// first would incorrectly report a change on every out-of-flow paint.
fn PersistPropertyStates(node: &mut PaintTreeNode<'_>, updater: &mut PropertyNodeUpdater<'_>) {
    for state in [&mut node.properties, &mut node.contents_properties] {
        updater.state(&mut state.nodes);
        state.transform_id = state.nodes.transform.id;
        state.clip_id = state.nodes.clip.id;
        state.effect_id = state.nodes.effect.id;
        state.scroll_id = state.nodes.nearest_scroll().map_or(0, |s| s.id);
    }
    if let Some(fixed) = &mut node.fixed_background_properties {
        updater.state(fixed);
    }
    if let Some(mask) = &mut node.mask_properties {
        updater.state(mask);
    }
    for child in &mut node.children {
        PersistPropertyStates(child, updater);
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

fn RestoreBuilderStates(node: &mut PaintTreeNode<'_>) {
    let cache = node
        .cache
        .as_ref()
        .expect("resident node has builder state");
    node.properties = cache.raw_properties.clone();
    node.contents_properties = cache.raw_contents.clone();
    node.fixed_background_properties = cache.raw_fixed_background.clone();
    node.mask_properties = cache.raw_mask.clone();
    node.opacity = cache.raw_opacity;
    node.applies_compositing_layer = cache.raw_compositing;
    node.escapes_ancestor_geometry = false;
    node.physical_ancestors.clear();
    for child in &mut node.children {
        RestoreBuilderStates(child);
    }
}

fn ResetPostPaintData(node: &mut PaintTreeNode<'_>) {
    node.outline_rects.clear();
    node.paints_outline = true;
    node.applies_overlap_compositing_layer = false;
    for child in &mut node.children {
        ResetPostPaintData(child);
    }
}

fn SameSourceIdentity(a: &FragmentNode, b: &FragmentNode) -> bool {
    a.node_id == b.node_id
        && a.fragment_instance_id == b.fragment_instance_id
        && a.fragmentainer_instance_id == b.fragmentainer_instance_id
        && a.kind == b.kind
        && a.paint.display_item_client_id == b.paint.display_item_client_id
        && a.paint.display_item_fragment == b.paint.display_item_fragment
}

// LayoutSVGHiddenContainer::Paint is a subtree boundary. Preserve its
// descendants in immutable Layout snapshots for resource consumers, but do
// not create ordinary paint owners, properties, or reparentable paint layers.
pub(crate) fn OrdinaryPaintChildren(
    fragment: &FragmentNode,
) -> impl Iterator<Item = &FragmentNode> {
    fragment
        .children
        .iter()
        .filter(|child| !fragment.paint.svg_hidden_container && !child.paint.svg_hidden_container)
}

fn SamePhaseInputs(a: &FragmentNode, b: &FragmentNode) -> bool {
    let phase = |f: &FragmentNode| {
        (
            f.paint.has_source,
            f.paint.floating,
            f.paint
                .form_control
                .as_ref()
                .is_some_and(|control| control.auto_focus_ring),
            f.paint.scrollbars.is_some(),
            f.paint.display,
            f.paint.establishes_paint_state,
            f.paint.svg_hidden_container,
            f.paint.style.visible,
            f.paint.style.outline_style,
            f.paint.style.outline_width,
            f.paint.style.outline_color.alpha,
            f.size,
        )
    };
    phase(a) == phase(b)
}

fn SameClientDirectory(a: &FragmentNode, b: &FragmentNode) -> bool {
    let x = &a.paint;
    let y = &b.paint;
    if (
        x.display_item_client_id,
        x.display_item_client_is_cacheable,
        x.paint_layer_client_id,
        x.paint_layer_client_is_cacheable,
    ) != (
        y.display_item_client_id,
        y.display_item_client_is_cacheable,
        y.paint_layer_client_id,
        y.paint_layer_client_is_cacheable,
    ) {
        return false;
    }
    let mut created = y.display_item_client_is_just_created || y.paint_layer_client_is_just_created;
    match (&x.scrollbars, &y.scrollbars) {
        (None, None) => {}
        (Some(x), Some(y)) => {
            if (x.corner_client_id, x.corner_client_is_cacheable)
                != (y.corner_client_id, y.corner_client_is_cacheable)
            {
                return false;
            }
            created |= y.corner_client_is_just_created;
            for (x, y) in [(&x.horizontal, &y.horizontal), (&x.vertical, &y.vertical)] {
                match (x, y) {
                    (None, None) => {}
                    (Some(x), Some(y)) => {
                        if (x.display_item_client_id, x.display_item_client_is_cacheable)
                            != (y.display_item_client_id, y.display_item_client_is_cacheable)
                        {
                            return false;
                        }
                        created |= y.display_item_client_is_just_created;
                    }
                    _ => return false,
                }
            }
        }
        _ => return false,
    }
    !created || b.pre_paint_owner_retained
}

// cpp: local_frame_view.cc:1083-1096,1138-1146; pre_paint_tree_walk.cc.
// ViewportGeometry is a host/CSS-pixel input, while the fragment tree and
// paint property nodes live in LocalFrame::LayoutZoomFactor coordinates.
// Keeping the conversion at the PrePaint boundary also lets CSSOM and media
// queries continue to consume the unscaled host viewport.
fn ArtifactViewport(root: &FragmentNode) -> Option<PaintRect> {
    root.paint.resources.as_ref().and_then(|resources| {
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

struct ContentUpdate {
    node: *mut PaintTreeNode<'static>,
    source: Option<Arc<FragmentNode>>,
    revision: u64,
    subtree_revision: u64,
}

impl PrePaintTreeWalk {
    /// The existing standalone LayoutView is explicitly a single-layout-
    /// viewport producer. No visual viewport/PageScale/overscroll node is
    /// synthesized from dimensions or guessed later by CompositorEngine.
    pub(crate) fn ExportCompositorViewport(
        &self,
        root: &FragmentNode,
    ) -> Option<crate::paint_engine::PaintCompositorViewport> {
        if !root.paint.is_layout_view {
            return None;
        }
        Some(crate::paint_engine::PaintCompositorViewport {
            mode: crate::paint_engine::PaintCompositorViewportMode::SingleLayoutViewport,
            viewport_clip: self.property_node_store.viewport_clip()?.clone(),
            layout_scroll_translation: self
                .property_node_store
                .layout_view_scroll_translation(
                    root.paint.display_item_client_id,
                    root.paint.display_item_fragment,
                )
                .cloned(),
            scroll_translation_nodes: self
                .property_node_store
                .compositor_scroll_translations()
                .cloned()
                .collect(),
        })
    }
    fn SameContext(&self, a: &InheritedContext, b: &InheritedContext) -> bool {
        a.offset == b.offset
            && a.paint_space == b.paint_space
            && a.clip == b.clip
            && a.cull == b.cull
            && a.transforms == b.transforms
            && a.opacity == b.opacity
            && a.scroll == b.scroll
            && a.fixed_background_acts_as_scroll == b.fixed_background_acts_as_scroll
            && a.rendering_context_id == b.rendering_context_id
            && a.should_flatten_inherited_transform == b.should_flatten_inherited_transform
            && a.can_inherit_backface_visibility == b.can_inherit_backface_visibility
            && a.root == b.root
            && a.flex_or_grid_item == b.flex_or_grid_item
            && state_compare::SameBuilderState(
                &a.properties.nodes,
                &b.properties.nodes,
                &self.property_keys,
                &self.prior_property_keys,
            )
    }

    fn SameGeometry(&self, a: &PaintTreeNode<'_>, b: &PaintTreeNode<'_>) -> bool {
        let old = a.cache.as_ref().expect("cached geometry");
        let new = b.cache.as_ref().expect("new geometry");
        let af = a.fragment.as_deref().unwrap();
        let bf = b.fragment.as_deref().unwrap();
        if (bf.paint.display_item_client_is_just_created && !bf.pre_paint_owner_retained)
            || !SameSourceIdentity(af, bf)
            || !SamePhaseInputs(af, bf)
            || af.paint.position != bf.paint.position
            || af.paint.logical_parent_node_id != bf.paint.logical_parent_node_id
            || af.paint.logical_parent_fragment_instance_id
                != bf.paint.logical_parent_fragment_instance_id
            || af.paint.logical_tree_order != bf.paint.logical_tree_order
        {
            return false;
        }
        macro_rules! same { ($($field:ident),* $(,)?) => { true $(&& a.$field == b.$field)* }; }
        same!(
            absolute_offset,
            paint_offset,
            paint_snap_offset,
            pixel_snap_scale,
            root_clip,
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
            is_stacking_context,
            is_root,
            stacking_level,
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
            mask_layers
        ) && old.raw_opacity == new.raw_opacity
            && old.raw_compositing == new.raw_compositing
            && state_compare::SameBuilderState(
                &old.raw_properties.nodes,
                &new.raw_properties.nodes,
                &self.property_keys,
                &self.prior_property_keys,
            )
            && state_compare::SameBuilderState(
                &old.raw_contents.nodes,
                &new.raw_contents.nodes,
                &self.property_keys,
                &self.prior_property_keys,
            )
            && match (&old.raw_mask, &new.raw_mask) {
                (None, None) => true,
                (Some(a), Some(b)) => state_compare::SameBuilderState(
                    a,
                    b,
                    &self.property_keys,
                    &self.prior_property_keys,
                ),
                _ => false,
            }
            && match (&old.raw_fixed_background, &new.raw_fixed_background) {
                (None, None) => true,
                (Some(a), Some(b)) => state_compare::SameBuilderState(
                    a,
                    b,
                    &self.property_keys,
                    &self.prior_property_keys,
                ),
                _ => false,
            }
    }

    /// Preserve completed geometry/property/phase data when the exact builder
    /// comparison proves that only source drawing content changed. Walking
    /// the physical links avoids dismantling an unchanged logical OOF tree.
    fn UpdateContentInPlace(
        &mut self,
        fragment: &FragmentNode,
        inherited: InheritedContext,
        node: &mut PaintTreeNode<'static>,
        updates: &mut Vec<ContentUpdate>,
    ) -> bool {
        let cache = node.cache.as_ref().unwrap();
        if !self.SameContext(&cache.inherited, &inherited) {
            return false;
        }
        if fragment.pre_paint_subtree_revision != 0
            && cache.subtree_revision == fragment.pre_paint_subtree_revision
        {
            self.property_keys
                .extend(cache.subtree_keys.iter().copied());
            self.stats.reused_nodes += cache.nodes;
            self.stats.skipped_subtrees += 1;
            return true;
        }
        let physical_children = cache.physical_children.clone();
        if physical_children.len() != OrdinaryPaintChildren(fragment).count()
            || !physical_children
                .iter()
                .zip(OrdinaryPaintChildren(fragment))
                .all(|(&pointer, source)| {
                    // These boxes remain owned even after logical reparenting.
                    // A failed topology proof returns before any box is removed.
                    SameSourceIdentity(unsafe { &*pointer }.fragment.as_deref().unwrap(), source)
                })
        {
            return false;
        }
        let self_clean =
            fragment.pre_paint_revision != 0 && cache.revision == fragment.pre_paint_revision;
        let mut source = None;
        if self_clean {
            self.stats.reused_nodes += 1;
        } else {
            if !SameClientDirectory(node.fragment.as_deref().unwrap(), fragment) {
                return false;
            }
            let next_id = self.next_property_id;
            let mut built = self.BuildNode(
                fragment,
                inherited.offset,
                inherited.paint_space,
                inherited.clip,
                inherited.cull,
                inherited.transforms.clone(),
                inherited.opacity,
                inherited.scroll,
                inherited.fixed_background_acts_as_scroll,
                inherited.properties.clone(),
                inherited.rendering_context_id,
                inherited.should_flatten_inherited_transform,
                inherited.can_inherit_backface_visibility,
                inherited.root,
                inherited.flex_or_grid_item,
            );
            let same = self.SameGeometry(node, &built);
            // The admission above excludes RegisterProperty's owner reset.
            // None of the speculative raw nodes are retained; restore their
            // directory/allocator state on success and rejection alike.
            self.property_keys.split_off(&next_id);
            self.next_property_id = next_id;
            if !same {
                return false;
            }
            source = built.fragment.take();
        }
        let cache = node.cache.as_ref().unwrap();
        let base = cache.children.clone();
        self.property_keys.extend(cache.own_keys.iter().copied());
        if inherited.root {
            self.CaptureRootFixedPositionContext(&base);
        }
        for (&pointer, child) in physical_children
            .iter()
            .zip(OrdinaryPaintChildren(fragment))
        {
            let context = self.ContextForChild(child, &base);
            // Physical links form a tree independently of the logical owners;
            // each child allocation is uniquely updated along this traversal.
            if !self.UpdateContentInPlace(child, context, unsafe { &mut *pointer }, updates) {
                return false;
            }
        }
        updates.push(ContentUpdate {
            node,
            source,
            revision: fragment.pre_paint_revision,
            subtree_revision: fragment.pre_paint_subtree_revision,
        });
        true
    }

    fn UpdateNode(
        &mut self,
        fragment: &FragmentNode,
        inherited: InheritedContext,
        mut previous: Option<Box<PaintTreeNode<'static>>>,
        force: bool,
    ) -> Box<PaintTreeNode<'static>> {
        let same_context = !force
            && previous.as_ref().is_some_and(|node| {
                node.cache
                    .as_ref()
                    .is_some_and(|cache| self.SameContext(&cache.inherited, &inherited))
            });
        let subtree_clean = same_context
            && fragment.pre_paint_subtree_revision != 0
            && previous.as_ref().is_some_and(|node| {
                node.cache.as_ref().unwrap().subtree_revision == fragment.pre_paint_subtree_revision
            });
        if subtree_clean {
            let node = previous.unwrap();
            let cache = node.cache.as_ref().unwrap();
            self.property_keys
                .extend(cache.subtree_keys.iter().copied());
            self.stats.reused_nodes += cache.nodes;
            self.stats.skipped_subtrees += 1;
            return node;
        }
        let self_clean = same_context
            && fragment.pre_paint_revision != 0
            && previous.as_ref().is_some_and(|node| {
                node.cache.as_ref().unwrap().revision == fragment.pre_paint_revision
            });
        let mut node = if self_clean {
            self.stats.reused_nodes += 1;
            previous.take().unwrap()
        } else {
            let mut built = self.BuildNode(
                fragment,
                inherited.offset,
                inherited.paint_space,
                inherited.clip,
                inherited.cull,
                inherited.transforms.clone(),
                inherited.opacity,
                inherited.scroll,
                inherited.fixed_background_acts_as_scroll,
                inherited.properties.clone(),
                inherited.rendering_context_id,
                inherited.should_flatten_inherited_transform,
                inherited.can_inherit_backface_visibility,
                inherited.root,
                inherited.flex_or_grid_item,
            );
            if let Some(mut old) = previous.take() {
                if !force && self.SameGeometry(&old, &built) {
                    // Only drawing content changed. Keep the already-finalized
                    // properties and group/phase results; clean descendants do
                    // not need a canonicalization or logical-tree pass.
                    built.properties = old.properties.clone();
                    built.contents_properties = old.contents_properties.clone();
                    built.fixed_background_properties = old.fixed_background_properties.clone();
                    built.mask_properties = old.mask_properties.clone();
                    built.opacity = old.opacity;
                    built.outline_rects = std::mem::take(&mut old.outline_rects);
                    built.paints_outline = old.paints_outline;
                    built.applies_overlap_compositing_layer = old.applies_overlap_compositing_layer;
                    built.needs_float_phase = old.needs_float_phase;
                    built.needs_outline_phase = old.needs_outline_phase;
                    built.needs_overflow_controls_phase = old.needs_overflow_controls_phase;
                    let old_cache = old.cache.as_ref().unwrap();
                    let new_cache = built.cache.as_mut().unwrap();
                    new_cache.raw_properties = old_cache.raw_properties.clone();
                    new_cache.raw_contents = old_cache.raw_contents.clone();
                    new_cache.raw_fixed_background = old_cache.raw_fixed_background.clone();
                    new_cache.raw_mask = old_cache.raw_mask.clone();
                    new_cache.children = old_cache.children.clone();
                    new_cache.own_keys = old_cache.own_keys.clone();
                } else {
                    self.needs_post_update = true;
                }
                built.children = std::mem::take(&mut old.children);
                built.physical_index = old.physical_index;
                // Update dirty ancestors in place, preserving every Box address
                // referenced by retained physical/logical ancestry.
                *old = *built;
                old
            } else {
                self.needs_post_update = true;
                built
            }
        };
        let base = node.cache.as_ref().unwrap().children.clone();
        let own_keys = node.cache.as_ref().unwrap().own_keys.clone();
        self.property_keys.extend(own_keys.iter().copied());
        if inherited.root {
            self.CaptureRootFixedPositionContext(&base);
        }
        let old_children = std::mem::take(&mut node.children);
        let same_children = old_children.len() == OrdinaryPaintChildren(fragment).count()
            && old_children
                .iter()
                .zip(OrdinaryPaintChildren(fragment))
                .all(|(old, source)| SameSourceIdentity(old.fragment.as_deref().unwrap(), source));
        if !same_children {
            self.needs_post_update = true;
        }
        let mut previous_children = old_children.into_iter();
        let mut keys = own_keys;
        let mut nodes = 1;
        for (index, child) in OrdinaryPaintChildren(fragment).enumerate() {
            let previous = previous_children
                .next()
                .filter(|old| SameSourceIdentity(old.fragment.as_deref().unwrap(), child));
            let child_context = self.ContextForChild(child, &base);
            let mut child_node = self.UpdateNode(child, child_context, previous, force);
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
            let cache = child_node.cache.as_ref().unwrap();
            keys.extend(cache.subtree_keys.iter().copied());
            nodes += cache.nodes;
            node.children.push(child_node);
        }
        let physical_children = node
            .children
            .iter_mut()
            .map(|child| &mut **child as *mut _)
            .collect();
        let cache = node.cache.as_mut().unwrap();
        cache.subtree_revision = fragment.pre_paint_subtree_revision;
        cache.subtree_keys = keys;
        cache.nodes = nodes;
        cache.physical_children = physical_children;
        node
    }

    pub(crate) fn SetRetainedNativeOwnerLifetimes(&mut self, retained: bool) {
        self.retained_native_owner_lifetimes = retained;
    }

    fn CaptureRootFixedPositionContext(&mut self, base: &ChildContext) {
        // Blink paint_property_tree_builder.cc UpdateForChildren creates the
        // LayoutView OverflowClip before UpdateOutOfFlowContext. The latter
        // copies the current state, restoring only the pre-scroll transform.
        // Keep the resolved scrollbar-excluding clip and effect, rather than
        // replacing them with the full host viewport or a nested box's clip.
        // Capture on both traversal paths so retained geometry and scrollbar
        // add/remove/resize updates use the same root-owned context.
        // Source: core/paint/paint_property_tree_builder.cc:820-826. Keep the
        // actual LayoutView ScrollTranslation before restoring the fixed
        // context's pre-scroll transform; this is a non-owning source node
        // reference represented by an immutable native Arc lease.
        let mut scroll_translation = Some(base.properties.nodes.transform.clone());
        while let Some(node) = &scroll_translation {
            if node.scroll.is_some() && node.id != 0 {
                break;
            }
            scroll_translation = node.parent.clone();
        }
        let mut properties = base.properties.clone();
        properties.transform_id = self.viewport_properties.transform_id;
        properties.nodes.transform = self.viewport_properties.nodes.transform.clone();
        self.root_fixed_position = Some(RootFixedPositionContext {
            clip: base.clip,
            properties,
            scroll_translation,
        });
    }

    fn ContextForChild(&self, child: &FragmentNode, base: &ChildContext) -> InheritedContext {
        if child.paint.fixed_to_view && child.paint.establishes_paint_state {
            // Blink switches fixed-to-view descendants to the LayoutView's
            // fixed-position context. Its transform is above document scroll,
            // but its clip still excludes classic root scrollbar gutters.
            let fixed = self
                .root_fixed_position
                .as_ref()
                .expect("root fixed-position context must precede descendants");
            return InheritedContext {
                offset: Offset::default(),
                paint_space: Offset::default(),
                clip: fixed.clip,
                cull: fixed.clip,
                transforms: Vec::new(),
                opacity: base.opacity,
                scroll: Offset::default(),
                fixed_background_acts_as_scroll: base.fixed_background_acts_as_scroll,
                properties: fixed.properties.clone(),
                rendering_context_id: 0,
                should_flatten_inherited_transform: true,
                can_inherit_backface_visibility: true,
                root: false,
                flex_or_grid_item: base.flex_or_grid_items,
            };
        }
        InheritedContext {
            offset: ResolveChildGeometryOrigin(child, base.origin, base.paint_space),
            paint_space: base.paint_space,
            clip: base.clip,
            cull: base.cull,
            transforms: base.transforms.clone(),
            opacity: base.opacity,
            scroll: base.scroll,
            fixed_background_acts_as_scroll: base.fixed_background_acts_as_scroll,
            properties: base.properties.clone(),
            rendering_context_id: base.rendering_context_id,
            should_flatten_inherited_transform: base.should_flatten_inherited_transform,
            can_inherit_backface_visibility: base.can_inherit_backface_visibility,
            root: false,
            flex_or_grid_item: base.flex_or_grid_items,
        }
    }
    fn RegisterProperty(&mut self, fragment: &FragmentNode, role: PaintPropertyRole, id: u64) {
        let client = fragment.paint.display_item_client_id;
        if client == 0 {
            return;
        }
        let owner = PaintPropertyOwner::NativeFragment {
            client,
            fragment: fragment.paint.display_item_fragment,
        };
        // Native client creation is the lifetime boundary, including allocator
        // address reuse. Never infer identity from a recycled pointer alone.
        if !self.retained_native_owner_lifetimes
            && !fragment.pre_paint_owner_retained
            && fragment.paint.display_item_client_is_just_created
            && self.reset_owners.insert(owner)
        {
            self.property_node_store.clear_owner(owner);
        }
        self.property_keys
            .insert(id, PaintPropertyKey { owner, role });
    }
    // cpp: paint/pre_paint_tree_walk.h:86
    // cpp: paint/pre_paint_tree_walk.cc:623-640
    #[cfg(feature = "translation_in_progress")]
    pub fn Walk(&mut self, root: &FragmentNode) -> Box<PaintTreeNode<'static>> {
        self.requires_main_thread_for_background_attachment_fixed = root
            .paint
            .requires_main_thread_scrolling_for_background_attachment_fixed;
        // Blink clears property change flags between lifecycle passes. Keep
        // old artifacts immutable and refresh only the resident snapshots;
        // this maintenance never recomputes geometry or dirties clean nodes.
        if self.retained_tree.is_some() {
            if let Some((store, mut trees)) = self.property_node_store.cleared_change_markers() {
                let mut refresh = |state: &mut crate::paint_property_tree::PropertyTreeState| {
                    trees.clear_state_change_markers(state);
                };
                for &pointer in &self.resident_nodes {
                    // RetainTree rebuilds this directory whenever owners change.
                    // All boxed allocations are still owned by retained_tree.
                    let node = unsafe { &mut *pointer };
                    refresh(&mut node.properties.nodes);
                    refresh(&mut node.contents_properties.nodes);
                    if let Some(fixed) = &mut node.fixed_background_properties {
                        refresh(fixed);
                    }
                    if let Some(mask) = &mut node.mask_properties {
                        refresh(mask);
                    }
                }
                self.property_node_store = store;
            }
        }
        self.stats = PrePaintUpdateStats::default();
        self.walk_generation = self
            .walk_generation
            .checked_add(1)
            .expect("PrePaint pass exhausted");
        self.reset_owners.clear();
        self.prior_property_keys = std::mem::take(&mut self.property_keys);
        let viewport = ArtifactViewport(root);
        let environment_changed = viewport != self.viewport;
        if environment_changed || (viewport.is_some() && self.viewport_properties.clip_id == 0) {
            self.viewport = viewport;
            self.viewport_properties = PropertyTreeState::default();
            if let Some(rect) = viewport {
                let id = self.next_property_id;
                self.next_property_id += 1;
                self.viewport_properties.clip_id = id;
                self.viewport_properties.nodes.clip = Arc::new(
                    ClipPaintPropertyNode {
                        geometry_mapper_cache: Default::default(),
                        layout_clip_rects: Default::default(),
                        lifecycle: Default::default(),
                        id,
                        parent: Some(self.viewport_properties.nodes.clip.clone()),
                        local_transform_space: PropertyTreeState::default().nodes.transform,
                        rect: Some(rect),
                        radii: PaintCornerRadii::default(),
                        clip_path: None,
                        clip_path_error: None,
                        pixel_moving_filter: None,
                    }
                    .with_layout_clips(),
                );
            }
        }
        if self.viewport_properties.clip_id != 0 {
            self.property_keys.insert(
                self.viewport_properties.clip_id,
                PaintPropertyKey {
                    owner: PaintPropertyOwner::Viewport,
                    role: PaintPropertyRole::ViewportClip,
                },
            );
        }
        let context = InheritedContext {
            offset: Offset::default(),
            paint_space: Offset::default(),
            clip: viewport,
            cull: viewport.map(|rect| Expand(rect, ScrollCullExpansion(root, true))),
            transforms: Vec::new(),
            opacity: 1.0,
            scroll: Offset::default(),
            fixed_background_acts_as_scroll: false,
            properties: self.viewport_properties.clone(),
            rendering_context_id: 0,
            should_flatten_inherited_transform: true,
            can_inherit_backface_visibility: true,
            root: true,
            flex_or_grid_item: false,
        };
        let mut previous = self.retained_tree.take();
        let force =
            !self.retained_tree_valid || environment_changed || root.pre_paint_revision == 0;
        let entirely_clean = !force
            && root.pre_paint_subtree_revision != 0
            && previous.as_ref().is_some_and(|node| {
                node.cache.as_ref().is_some_and(|cache| {
                    cache.subtree_revision == root.pre_paint_subtree_revision
                        && self.SameContext(&cache.inherited, &context)
                })
            });
        if entirely_clean {
            let tree = previous.unwrap();
            let cache = tree.cache.as_ref().unwrap();
            self.stats.reused_nodes = cache.nodes;
            self.stats.skipped_subtrees = 1;
            self.property_keys
                .extend(cache.subtree_keys.iter().copied());
            return tree;
        }
        if !force {
            if let Some(tree) = &mut previous {
                let original_keys = self.property_keys.clone();
                let mut updates = Vec::new();
                if self.UpdateContentInPlace(root, context.clone(), tree, &mut updates) {
                    for update in updates {
                        // All candidates passed before any resident source or
                        // revision is committed. Box ownership stays intact.
                        let node = unsafe { &mut *update.node };
                        if let Some(source) = update.source {
                            node.fragment = Some(source);
                        }
                        let cache = node.cache.as_mut().unwrap();
                        cache.revision = update.revision;
                        cache.subtree_revision = update.subtree_revision;
                    }
                    self.needs_post_update = false;
                    return previous.unwrap();
                }
                // A geometry/phase/topology change needs the full existing
                // ownership pass. Discard the staged source updates and keys;
                // its attempted work must not count as skipped geometry.
                self.stats = PrePaintUpdateStats::default();
                self.property_keys = original_keys;
            }
        }
        self.needs_post_update = force || previous.is_none() || self.logical_tree_changed;
        if self.logical_tree_changed {
            if let Some(tree) = &mut previous {
                RestorePhysicalOwnership(tree);
            }
        }
        let mut tree = self.UpdateNode(root, context, previous, force);
        if self.needs_post_update {
            self.stats.post_walk = true;
            RestoreBuilderStates(&mut tree);
            self.logical_tree_changed = RestoreLogicalOutOfFlowParents(&mut tree);
            let mut effects = BTreeMap::new();
            GatherFinalEffects(&tree, &mut effects);
            let mut updater = self
                .property_node_store
                .updater(&self.property_keys, &effects);
            PersistPropertyStates(&mut tree, &mut updater);
            updater.finish();
            ResetPostPaintData(&mut tree);
            PrepareInlineOutlines(&mut tree);
            MarkOverlapCompositingLayers(&mut tree);
        }
        tree
    }

    pub(crate) fn RetainTree(&mut self, mut tree: Box<PaintTreeNode<'static>>) {
        if self.stats.post_walk {
            self.client_nodes.clear();
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
                add(f.paint.display_item_client_id);
                add(f.paint.paint_layer_client_id);
                if let Some(bars) = &f.paint.scrollbars {
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
        }
        self.retained_tree = Some(tree);
        self.retained_tree_valid = true;
    }

    /// PaintController validation changes client metadata even without a new
    /// layout snapshot. Update only resident sources for the committed clients.
    pub(crate) fn CommitClients(&mut self, clients: impl IntoIterator<Item = u64>) {
        for id in clients {
            let Some(nodes) = self.client_nodes.get(&id) else {
                continue;
            };
            for &pointer in nodes {
                // This directory is rebuilt before publication whenever owners
                // are added/removed. Box addresses survive local updates.
                let node = unsafe { &mut *pointer };
                let f = Arc::make_mut(node.fragment.as_mut().unwrap());
                if f.paint.display_item_client_id == id && f.paint.display_item_client_is_cacheable
                {
                    f.paint.display_item_client_is_just_created = false;
                }
                if f.paint.paint_layer_client_id == id && f.paint.paint_layer_client_is_cacheable {
                    f.paint.paint_layer_client_is_just_created = false;
                }
                if let Some(bars) = &mut f.paint.scrollbars {
                    let bars = Arc::make_mut(bars);
                    for axis in [&mut bars.horizontal, &mut bars.vertical]
                        .into_iter()
                        .flatten()
                    {
                        if axis.display_item_client_id == id
                            && axis.display_item_client_is_cacheable
                        {
                            axis.display_item_client_is_just_created = false;
                        }
                    }
                    if bars.corner_client_id == id && bars.corner_client_is_cacheable {
                        bars.corner_client_is_just_created = false;
                    }
                }
            }
        }
    }

    // cpp: paint/pre_paint_tree_walk.h:89-99
    // cpp: paint/pre_paint_tree_walk.cc:642-936
    #[cfg(feature = "translation_in_progress")]
    fn BuildNode(
        &mut self,
        fragment: &FragmentNode,
        parent_offset: Offset,
        paint_space_origin: Offset,
        parent_clip: Option<PaintRect>,
        parent_cull_rect: Option<PaintRect>,
        transforms: Vec<TransformMatrix>,
        parent_opacity: f32,
        scroll_offset: Offset,
        inherited_fixed_background_acts_as_scroll: bool,
        parent_properties: PropertyTreeState,
        inherited_rendering_context_id: u64,
        inherited_should_flatten_transform: bool,
        inherited_can_inherit_backface: bool,
        is_root: bool,
        is_flex_or_grid_item: bool,
    ) -> Box<PaintTreeNode<'static>> {
        self.stats.computed_nodes += 1;
        let own_key_start = self.next_property_id;
        let inherited = InheritedContext {
            offset: parent_offset,
            paint_space: paint_space_origin,
            clip: parent_clip,
            cull: parent_cull_rect,
            transforms: transforms.clone(),
            opacity: parent_opacity,
            scroll: scroll_offset,
            fixed_background_acts_as_scroll: inherited_fixed_background_acts_as_scroll,
            properties: parent_properties.clone(),
            rendering_context_id: inherited_rendering_context_id,
            should_flatten_inherited_transform: inherited_should_flatten_transform,
            can_inherit_backface_visibility: inherited_can_inherit_backface,
            root: is_root,
            flex_or_grid_item: is_flex_or_grid_item,
        };
        let mut node = Box::new(PaintTreeNode::default());
        node.SetFragment(fragment);
        node.is_root = is_root;
        let local_establishes_fixed_background_containing_block = !is_root
            && fragment.paint.establishes_paint_state
            && fragment.paint.has_source
            && (fragment.paint.style.transform.is_some()
                || fragment.paint.style.will_change_transform);
        node.fixed_background_acts_as_scroll = inherited_fixed_background_acts_as_scroll
            || local_establishes_fixed_background_containing_block;
        let inherited_transform_count = transforms.len();
        let initial_paint_offset = Offset {
            x: parent_offset.x + fragment.offset.x - paint_space_origin.x,
            y: parent_offset.y + fragment.offset.y - paint_space_origin.y,
        };
        let mut transforms = transforms;
        let geometry_property_delta =
            FlatToPropertySpaceOffset(&transforms, &parent_properties.nodes.transform)
                .unwrap_or_default();
        let geometry = ResolveFragmentGeometry(
            fragment,
            parent_offset,
            paint_space_origin,
            &mut transforms,
            geometry_property_delta,
        );
        let paint_space_origin = geometry.paint_space_origin;
        node.absolute_offset = geometry.absolute_offset;
        node.paint_offset = geometry.paint_offset;
        node.pixel_snap_scale = fragment
            .paint
            .resources
            .as_ref()
            .map_or(1.0, |resources| resources.ArtifactToDeviceScale())
            .max(f64::MIN_POSITIVE);
        node.clip = parent_clip;
        node.cull_rect = parent_cull_rect;
        node.self_cull_rect = parent_cull_rect;
        node.transforms = transforms;
        node.scroll_offset = scroll_offset;
        node.properties = parent_properties.clone();
        node.applies_clip = is_root && parent_clip.is_some();
        node.applies_root_clip = node.applies_clip;
        if node.applies_root_clip {
            node.root_clip = parent_clip;
        }

        let style: Option<&PaintStyleData> =
            fragment.paint.has_source.then_some(&*fragment.paint.style);
        let establishes = fragment.paint.establishes_paint_state;
        node.opacity = parent_opacity
            * if establishes {
                style.map_or(1.0, MaskAdjustedOpacity)
            } else {
                1.0
            };
        let position = if fragment.paint.has_source {
            fragment.paint.position
        } else {
            Position::kStatic
        };
        let layer = ClassifyFragmentPaintLayer(fragment, is_root, is_flex_or_grid_item);
        node.stacking_level = layer.stacking_level;
        node.is_paint_layer = layer.is_paint_layer;
        node.is_stacking_context = layer.is_stacking_context;

        if geometry.applies_transform {
            node.properties.transform_id = self.next_property_id;
            self.next_property_id += 1;
            node.applies_transform = true;
            node.local_transforms
                .extend_from_slice(&node.transforms[inherited_transform_count..]);
            // Distinct source roles retain their identities when a preceding
            // sticky/viewport translation is added or removed. Assigning slots
            // by flat replay index would accidentally recycle another role.
            let mut source_roles = Vec::new();
            if fragment.paint.source_kind == NodeKind::kSvgRoot
                && (initial_paint_offset.x != 0.0 || initial_paint_offset.y != 0.0)
            {
                source_roles.push(PaintPropertyRole::PaintOffsetTranslation);
            }
            if position == Position::kSticky
                && (fragment.paint.sticky_constraint.is_some()
                    || fragment.paint.sticky_offset.x != 0.0
                    || fragment.paint.sticky_offset.y != 0.0)
            {
                source_roles.push(PaintPropertyRole::StickyTranslation);
            }
            if style.is_some_and(|style| style.transform.is_some())
                && fragment.paint.svg_local_transform.is_none()
            {
                source_roles.push(PaintPropertyRole::Transform);
            }
            if fragment.paint.svg_local_transform.is_some() {
                source_roles.push(PaintPropertyRole::SvgLocalTransform);
            }
            if fragment
                .paint
                .svg_shape
                .as_ref()
                .is_some_and(|shape| shape.local_transform.is_some())
            {
                // The native SVG affine matrix is a distinct existing replay
                // input. Mixed CSS+SVG transform lowering is still adapted;
                // it is not claimed to match LocalToSVGParentTransform fully.
                source_roles.push(PaintPropertyRole::SvgLocalTransform);
            }
            if fragment.paint.text_combine.is_some() {
                source_roles.push(PaintPropertyRole::TextCombineTransform);
            }
            debug_assert_eq!(source_roles.len(), node.local_transforms.len());
            let mut operations: Vec<_> = source_roles
                .into_iter()
                .zip(node.local_transforms.iter().copied())
                .collect();
            if style.is_some_and(|style| {
                style.transform.is_none()
                    && (style.will_change_transform
                        || style.preserves_3d
                        || style.backface_visibility_hidden)
            }) {
                // NeedsTransform creates a transform slot even with identity
                // matrix. The compositing reason belongs to that slot, never
                // to an incidental sticky or viewport translation.
                let index = operations
                    .iter()
                    .position(|(role, _)| *role == PaintPropertyRole::SvgLocalTransform)
                    .unwrap_or(operations.len());
                operations.insert(
                    index,
                    (PaintPropertyRole::Transform, TransformMatrix::default()),
                );
            }
            let mut flat_transform_context = node.transforms[..inherited_transform_count].to_vec();
            let mut current_rendering_context_id = inherited_rendering_context_id;
            for (index, (role, matrix)) in operations.iter().enumerate() {
                // UpdatePaintOffsetTranslation/UpdateTransform establish a
                // local paint space. Express this reset matrix in its actual
                // property-parent space, so P_parent*M_property
                // equals the existing F_parent*M_flat. Constant sticky/SVG
                // local/text-combine matrices and will-change identity slots
                // do not reset paint_offset and keep their scroll anchor.
                let resets_paint_space = *role == PaintPropertyRole::PaintOffsetTranslation
                    || (*role == PaintPropertyRole::Transform
                        && style.is_some_and(|style| style.transform.is_some()));
                let property_matrix = if resets_paint_space {
                    FlatToPropertySpaceOffset(
                        &flat_transform_context,
                        &node.properties.nodes.transform,
                    )
                    .map(|delta| {
                        MultiplyTransforms(&TranslationTransform(delta.x, delta.y), matrix)
                    })
                    .unwrap_or(*matrix)
                } else {
                    *matrix
                };
                let id = if index == 0 {
                    node.properties.transform_id
                } else {
                    let id = self.next_property_id;
                    self.next_property_id += 1;
                    id
                };
                let sticky_constraint = (*role == PaintPropertyRole::StickyTranslation)
                    .then_some(fragment.paint.sticky_constraint)
                    .flatten();
                let parent_transform = node.properties.nodes.transform.clone();
                let mut compositor = TransformCompositorState(fragment, *role);
                if let Some(sticky) = sticky_constraint {
                    compositor.sticky_constraint = BuildCompositorStickyConstraint(
                        sticky,
                        &parent_transform,
                        node.paint_snap_offset,
                    );
                }
                let reasons = if compositor.sticky_constraint.is_some() {
                    vec!["StickyPosition"]
                } else if *role == PaintPropertyRole::Transform
                    && style.is_some_and(|style| style.will_change_transform)
                {
                    vec!["WillChangeTransform"]
                } else {
                    Vec::new()
                };
                let is_css_transform = *role == PaintPropertyRole::Transform;
                let style_3d = style.filter(|_| is_css_transform);
                let establishes_3d_context = style_3d
                    .is_some_and(|style| style.preserves_3d && current_rendering_context_id == 0);
                let rendering_context_id = if establishes_3d_context {
                    id
                } else {
                    current_rendering_context_id
                };
                let backface_visibility =
                    if style_3d.is_some_and(|style| style.backface_visibility_hidden) {
                        PaintBackfaceVisibility::Hidden
                    } else if style_3d.is_some_and(|style| {
                        establishes_3d_context
                            || !inherited_can_inherit_backface
                            || style.has_3d_transform_operation
                    }) {
                        PaintBackfaceVisibility::Visible
                    } else {
                        PaintBackfaceVisibility::Inherited
                    };
                self.RegisterProperty(fragment, *role, id);
                node.properties.nodes.transform = Arc::new(TransformPaintPropertyNode {
                    compositor,
                    geometry_mapper_cache: Default::default(),
                    scroll_parent_scroll_translation: if *role
                        == PaintPropertyRole::PaintOffsetTranslation
                        && fragment.paint.fixed_to_view
                    {
                        self.root_fixed_position
                            .as_ref()
                            .and_then(|context| context.scroll_translation.clone())
                    } else {
                        None
                    },
                    lifecycle: Default::default(),
                    id,
                    parent: Some(parent_transform),
                    matrix: property_matrix,
                    origin: [0.0; 3],
                    flattens_inherited_transform: inherited_should_flatten_transform,
                    rendering_context_id,
                    backface_visibility,
                    scroll: None,
                    sticky_constraint,
                    direct_compositing_reasons: reasons,
                });
                current_rendering_context_id = rendering_context_id;
                flat_transform_context.push(*matrix);
            }
        }
        // Unsupported non-affine transforms are rejected by raster lowering;
        // only a proven coordinate translation changes these property payloads.
        let property_space_delta = if geometry.applies_transform {
            FlatToPropertySpaceOffset(&node.transforms, &node.properties.nodes.transform)
                .unwrap_or_default()
        } else {
            // Neither flat transforms nor the property transform changed;
            // reuse the already validated projection for ordinary fragments.
            geometry_property_delta
        };
        node.paint_snap_offset = property_space_delta;
        if let Some(style) = style.filter(|_| establishes) {
            if !style.filters.is_empty() {
                ValidateFilters(&style.filters);
                if let Some(cull_rect) = node.cull_rect {
                    node.cull_rect = Some(Expand(
                        cull_rect,
                        FilterCullOutset(&style.filters, fragment.size),
                    ));
                }
            }
            node.applies_compositing_layer = style.will_change_transform;
            node.applies_opacity = MaskAdjustedOpacity(style) != 1.0;
            node.applies_blend =
                style.blend_mode != PaintBlendMode::kNormal || style.isolate_blending;
            node.applies_filter = !style.filters.is_empty();
            if !style.backdrop_filters.is_empty() {
                ValidateFilters(&style.backdrop_filters);
            }
            node.applies_backdrop_filter =
                !style.backdrop_filters.is_empty() || style.backdrop_filter_has_reference;
            if !style.mask_images.is_empty() {
                node.mask_layers = ResolveMaskLayers(
                    fragment,
                    node.paint_offset,
                    property_space_delta,
                    node.pixel_snap_scale,
                    &style.mask_images,
                );
            } else {
                node.mask_layers.clear();
            }
            node.applies_mask = !node.mask_layers.is_empty() || style.mask_box_image.is_some();
            if node.applies_mask {
                if let (Some(cull_rect), Some(mask_bounds)) =
                    (node.cull_rect, node.ResolvedMaskClipRect())
                {
                    if let Some(mapped) = MapRectToRoot(mask_bounds, &node.transforms) {
                        node.cull_rect = Some(Intersect(cull_rect, &mapped));
                    }
                }
            }
        }
        if establishes {
            if let Some(style) = style {
                if let Some(clip_path) = style.clip_path.as_ref() {
                    let reference =
                        ClipPathReferenceRect(fragment, node.paint_offset, clip_path.reference_box);
                    node.clip_path_error = clip_path.reference_error.clone();
                    let path_bounds = if let Some(graph) = clip_path.reference_graph.as_ref() {
                        if node.clip_path_error.is_none() {
                            node.clip_path_error = graph.unsupported.clone()
                        }
                        node.clip_path_even_odd =
                            graph.children.first().is_some_and(|child| child.even_odd);
                        ResolveReferenceClipPath(
                            graph,
                            reference,
                            node.paint_offset,
                            f64::from(fragment.paint.effective_zoom),
                            IsSvgEffectChild(fragment.paint.source_kind),
                            clip_path.reference_viewport,
                            fragment
                                .paint
                                .resources
                                .as_ref()
                                .map_or(&[], |resources| resources.fonts.as_slice()),
                            &mut node.clip_path_error,
                            &mut node.local_clip_path,
                            &mut node.local_clip_path_children,
                        )
                    } else if clip_path.reference_url.is_some() {
                        PaintRect {
                            x: reference.x,
                            y: reference.y,
                            width: 0.0,
                            height: 0.0,
                        }
                    } else {
                        node.clip_path_even_odd = clip_path.even_odd;
                        ResolveClipPath(
                            clip_path,
                            Offset {
                                x: reference.x,
                                y: reference.y,
                            },
                            Size {
                                width: reference.width,
                                height: reference.height,
                            },
                            if IsSvgEffectChild(fragment.paint.source_kind) {
                                1.0 / f64::from(fragment.paint.effective_zoom)
                                    .max(f64::MIN_POSITIVE)
                            } else {
                                1.0
                            },
                            &mut node.local_clip_path,
                        )
                    };
                    if let Some(mapped) = MapRectToRoot(path_bounds, &node.transforms) {
                        node.clip = Some(node.clip.map_or(mapped, |clip| Intersect(clip, &mapped)));
                        node.cull_rect = Some(
                            node.cull_rect
                                .map_or(mapped, |cull_rect| Intersect(cull_rect, &mapped)),
                        );
                        node.self_cull_rect = Some(
                            node.self_cull_rect
                                .map_or(mapped, |self_cull| Intersect(self_cull, &mapped)),
                        );
                    }
                    node.properties.clip_id = self.next_property_id;
                    self.next_property_id += 1;
                    self.RegisterProperty(
                        fragment,
                        PaintPropertyRole::ClipPathClip,
                        node.properties.clip_id,
                    );
                    node.properties.nodes.clip = Arc::new(
                        ClipPaintPropertyNode {
                            geometry_mapper_cache: Default::default(),
                            layout_clip_rects: Default::default(),
                            lifecycle: Default::default(),
                            id: node.properties.clip_id,
                            parent: Some(node.properties.nodes.clip.clone()),
                            local_transform_space: node.properties.nodes.transform.clone(),
                            rect: Some(PropertySpaceRect(path_bounds, property_space_delta)),
                            radii: PaintCornerRadii::default(),
                            clip_path: Some(Arc::new(
                                crate::paint_clip_path::PaintClipPath::from_authored(
                                    &node.local_clip_path,
                                    &node.local_clip_path_children,
                                    node.clip_path_even_odd,
                                    property_space_delta,
                                ),
                            )),
                            clip_path_error: node.clip_path_error.clone(),
                            pixel_moving_filter: None,
                        }
                        .with_layout_clips(),
                    );
                    node.applies_clip = true;
                    node.applies_clip_path = true;
                }
            }
        }

        // UpdateEffect/UpdateMask use the ancestor output clip; MaskClip
        // restricts the contents only, never the mask source itself.
        let mask_output_clip = node.properties.nodes.clip.clone();
        node.applies_effect = node.applies_compositing_layer
            || node.applies_opacity
            || node.applies_blend
            || node.applies_filter
            || node.applies_backdrop_filter
            || node.applies_mask;
        if node.applies_opacity
            || node.applies_blend
            || node.applies_backdrop_filter
            || node.applies_mask
        {
            let style = style.expect("effect must have source style");
            let backdrop_filter_clip = if node.applies_backdrop_filter {
                let id = self.next_property_id;
                self.next_property_id += 1;
                self.RegisterProperty(fragment, PaintPropertyRole::BackdropFilterClip, id);
                Some(Arc::new(
                    ClipPaintPropertyNode {
                        geometry_mapper_cache: Default::default(),
                        layout_clip_rects: Default::default(),
                        lifecycle: Default::default(),
                        id,
                        parent: Some(node.properties.nodes.clip.clone()),
                        local_transform_space: node.properties.nodes.transform.clone(),
                        rect: Some(PropertySpaceRect(
                            node.PixelSnappedRect(&PaintRect {
                                x: node.paint_offset.x,
                                y: node.paint_offset.y,
                                width: fragment.size.width,
                                height: fragment.size.height,
                            }),
                            property_space_delta,
                        )),
                        radii: ResolveCornerRadii(
                            style,
                            &fragment.paint.border_sides,
                            fragment.size,
                            Edges::default(),
                        ),
                        clip_path: None,
                        clip_path_error: None,
                        pixel_moving_filter: None,
                    }
                    .with_layout_clips(),
                ))
            } else {
                None
            };
            node.properties.effect_id = self.next_property_id;
            self.next_property_id += 1;
            self.RegisterProperty(
                fragment,
                PaintPropertyRole::Effect,
                node.properties.effect_id,
            );
            node.properties.nodes.effect = Arc::new(
                EffectPaintPropertyNode {
                    compositor: EffectCompositorState(
                        fragment,
                        PaintPropertyRole::Effect,
                        style.will_change_transform,
                        node.applies_mask && node.applies_backdrop_filter,
                    ),

                    self_or_ancestor_participates_in_view_transition: false,
                    lifecycle: Default::default(),
                    id: node.properties.effect_id,
                    parent: Some(node.properties.nodes.effect.clone()),
                    local_transform_space: node.properties.nodes.transform.clone(),
                    output_clip: Some(node.properties.nodes.clip.clone()),
                    filter_reference_box: (!style.backdrop_filters.is_empty()).then(|| {
                        PropertySpaceRect(
                            NinePieceImagePainter::ReferenceRect(&node),
                            property_space_delta,
                        )
                    }),
                    filter_reference_scale: if IsSvgEffectChild(fragment.paint.source_kind) {
                        1.0
                    } else {
                        f64::from(fragment.paint.effective_zoom)
                    },
                    opacity: MaskAdjustedOpacity(style),
                    blend_mode: style.blend_mode,
                    filters: Default::default(),
                    backdrop_filters:
                        crate::property_filter_operations::PropertyFilterOperations::from_authored(
                            &style.backdrop_filters,
                        ),
                    backdrop_filter_has_reference: style.backdrop_filter_has_reference,
                    backdrop_filter_bounds: backdrop_filter_clip.as_ref().map(|clip| {
                        crate::paint_clip_path::rounded_bounds(
                            clip.rect.expect("source finite backdrop bounds"),
                            clip.radii,
                        )
                    }),
                    backdrop_filter_clip,
                    isolates_blending: style.isolate_blending,
                    has_mask: false,
                    is_mask: false,
                    // Source: core/paint/compositing/compositing_reason_finder.cc:461-462;
                    // core/paint/paint_property_tree_builder.cc:2232-2240.
                    // BackdropFilter is a direct effect reason: decompositing it
                    // would restrict backdrop reads to individual raster tiles.
                    // The additional effect trigger also carries transform hints.
                    direct_compositing_reasons: {
                        let mut reasons = Vec::new();
                        if !style.backdrop_filters.is_empty() {
                            reasons.push("BackdropFilter");
                        }
                        if style.will_change_transform {
                            reasons.push("WillChangeTransform");
                        }
                        reasons
                    },
                }
                .with_filter_presence()
                .with_inherited_view_transition(),
            );
        }
        if node.applies_mask {
            // CSS masks establish a stacking context and isolated layer even
            // when a standalone native owner has no exported PaintLayer yet.
            node.is_paint_layer = true;
            node.is_stacking_context = true;
            let mask_id = self.next_property_id;
            self.next_property_id += 1;
            self.RegisterProperty(fragment, PaintPropertyRole::Mask, mask_id);
            let mask_effect = Arc::new(
                EffectPaintPropertyNode {
                    compositor: EffectCompositorState(
                        fragment,
                        PaintPropertyRole::Mask,
                        false,
                        false,
                    ),
                    backdrop_filter_bounds: None,
                    self_or_ancestor_participates_in_view_transition: false,
                    lifecycle: Default::default(),
                    id: mask_id,
                    parent: Some(node.properties.nodes.effect.clone()),
                    local_transform_space: node.properties.nodes.transform.clone(),
                    output_clip: Some(mask_output_clip.clone()),
                    filter_reference_box: None,
                    filter_reference_scale: 1.0,
                    opacity: 1.0,
                    blend_mode: PaintBlendMode::kNormal,
                    filters: Default::default(),
                    backdrop_filters: Default::default(),
                    backdrop_filter_has_reference: false,
                    backdrop_filter_clip: None,
                    isolates_blending: false,
                    has_mask: false,
                    is_mask: true,
                    direct_compositing_reasons: Vec::new(),
                }
                .with_filter_presence()
                .with_inherited_view_transition(),
            );
            node.mask_properties = Some(crate::paint_property_tree::PropertyTreeState {
                transform: node.properties.nodes.transform.clone(),
                clip: mask_output_clip.clone(),
                effect: mask_effect,
            });
            let mask_bounds = node.ResolvedMaskClipRect().expect("resolved MaskClip");
            let clip_id = self.next_property_id;
            self.next_property_id += 1;
            self.RegisterProperty(fragment, PaintPropertyRole::MaskClip, clip_id);
            node.properties.clip_id = clip_id;
            node.properties.nodes.clip = Arc::new(
                ClipPaintPropertyNode {
                    geometry_mapper_cache: Default::default(),
                    layout_clip_rects: Default::default(),
                    lifecycle: Default::default(),
                    id: clip_id,
                    parent: Some(mask_output_clip),
                    local_transform_space: node.properties.nodes.transform.clone(),
                    rect: Some(PropertySpaceRect(mask_bounds, property_space_delta)),
                    radii: PaintCornerRadii::default(),
                    clip_path: None,
                    clip_path_error: None,
                    pixel_moving_filter: None,
                }
                .with_layout_clips(),
            );
        }
        // Blink UpdateCssClip runs after UpdateEffect and before UpdateFilter.
        // Unlike overflow clipping, the legacy positioned-box clip applies to
        // the box's own background as well as all descendants.
        if establishes {
            if let Some(css_clip) = fragment.paint.css_clip {
                let local = node.PixelSnappedRect(&PaintRect {
                    x: node.paint_offset.x + css_clip.offset.x,
                    y: node.paint_offset.y + css_clip.offset.y,
                    width: css_clip.size.width.max(0.0),
                    height: css_clip.size.height.max(0.0),
                });
                node.local_css_clip = Some(local);
                if let Some(mapped) = MapRectToRoot(local, &node.transforms) {
                    node.clip = Some(node.clip.map_or(mapped, |clip| Intersect(clip, &mapped)));
                    node.cull_rect = Some(
                        node.cull_rect
                            .map_or(mapped, |cull_rect| Intersect(cull_rect, &mapped)),
                    );
                    node.self_cull_rect = Some(
                        node.self_cull_rect
                            .map_or(mapped, |self_cull| Intersect(self_cull, &mapped)),
                    );
                }
                node.properties.clip_id = self.next_property_id;
                self.next_property_id += 1;
                self.RegisterProperty(
                    fragment,
                    PaintPropertyRole::CssClip,
                    node.properties.clip_id,
                );
                node.properties.nodes.clip = Arc::new(
                    ClipPaintPropertyNode {
                        geometry_mapper_cache: Default::default(),
                        layout_clip_rects: Default::default(),
                        lifecycle: Default::default(),
                        id: node.properties.clip_id,
                        parent: Some(node.properties.nodes.clip.clone()),
                        local_transform_space: node.properties.nodes.transform.clone(),
                        rect: Some(PropertySpaceRect(local, property_space_delta)),
                        radii: PaintCornerRadii::default(),
                        clip_path: None,
                        clip_path_error: None,
                        pixel_moving_filter: None,
                    }
                    .with_layout_clips(),
                );
                node.applies_clip = true;
                node.applies_css_clip = true;
            }
        }
        if node.applies_filter {
            let style = style.expect("filter must have source style");
            node.recording_filters = ResolveFilterImages(&node, &style.filters);
            node.properties.effect_id = self.next_property_id;
            self.next_property_id += 1;
            self.RegisterProperty(
                fragment,
                PaintPropertyRole::Filter,
                node.properties.effect_id,
            );
            // UpdateFilter follows UpdateEffect and has its own persistent
            // ObjectPaintProperties slot (paint_property_tree_builder.cc:2630).
            node.properties.nodes.effect = Arc::new(
                EffectPaintPropertyNode {
                    compositor: EffectCompositorState(
                        fragment,
                        PaintPropertyRole::Filter,
                        style.will_change_transform,
                        false,
                    ),
                    backdrop_filter_bounds: None,
                    self_or_ancestor_participates_in_view_transition: false,
                    lifecycle: Default::default(),
                    id: node.properties.effect_id,
                    parent: Some(node.properties.nodes.effect.clone()),
                    local_transform_space: node.properties.nodes.transform.clone(),
                    output_clip: Some(node.properties.nodes.clip.clone()),
                    filter_reference_box: Some(PropertySpaceRect(
                        NinePieceImagePainter::ReferenceRect(&node),
                        property_space_delta,
                    )),
                    filter_reference_scale: if IsSvgEffectChild(fragment.paint.source_kind) {
                        1.0
                    } else {
                        f64::from(fragment.paint.effective_zoom)
                    },
                    opacity: 1.0,
                    blend_mode: PaintBlendMode::kNormal,
                    filters:
                        crate::property_filter_operations::PropertyFilterOperations::from_authored(
                            &node.recording_filters,
                        ),
                    backdrop_filters: Default::default(),
                    backdrop_filter_has_reference: false,
                    backdrop_filter_clip: None,
                    isolates_blending: false,
                    has_mask: false,
                    is_mask: false,
                    direct_compositing_reasons: if style.will_change_transform {
                        vec!["WillChangeTransform"]
                    } else {
                        Vec::new()
                    },
                }
                .with_filter_presence()
                .with_inherited_view_transition(),
            );
            if style.filters.iter().any(|filter| {
                matches!(
                    filter.r#type,
                    PaintFilterType::kBlur
                        | PaintFilterType::kDropShadow
                        | PaintFilterType::kBoxReflect
                )
            }) {
                node.properties.clip_id = self.next_property_id;
                self.next_property_id += 1;
                self.RegisterProperty(
                    fragment,
                    PaintPropertyRole::PixelMovingFilterClipExpander,
                    node.properties.clip_id,
                );
                node.properties.nodes.clip = Arc::new(
                    ClipPaintPropertyNode {
                        geometry_mapper_cache: Default::default(),
                        layout_clip_rects: Default::default(),
                        lifecycle: Default::default(),
                        id: node.properties.clip_id,
                        parent: Some(node.properties.nodes.clip.clone()),
                        local_transform_space: node.properties.nodes.transform.clone(),
                        rect: None,
                        radii: PaintCornerRadii::default(),
                        clip_path: None,
                        clip_path_error: None,
                        pixel_moving_filter: Some(node.properties.nodes.effect.clone()),
                    }
                    .with_layout_clips(),
                );
            }
        }
        // Chromium's composited background-attachment:fixed path records the
        // viewport-sized background in the LayoutView transform, then clips
        // it with a node whose local transform space is this element.  The
        // clip is deliberately not installed into the descendant context.
        if fragment.paint.can_composite_background_attachment_fixed {
            let style = style.expect("composited fixed background requires style");
            let layer = style
                .background_images
                .first()
                .expect("composited fixed background requires one image layer");
            let border = IncludedBorderEdges(&fragment.paint);
            let mut local = PaintRect {
                x: node.paint_offset.x,
                y: node.paint_offset.y,
                width: fragment.size.width,
                height: fragment.size.height,
            };
            if matches!(
                layer.clip,
                BackgroundBox::kPaddingBox | BackgroundBox::kContentBox
            ) {
                local.x += border.left;
                local.y += border.top;
                local.width = (local.width - border.left - border.right).max(0.0);
                local.height = (local.height - border.top - border.bottom).max(0.0);
            }
            if layer.clip == BackgroundBox::kContentBox {
                local.x += fragment.paint.padding.left;
                local.y += fragment.paint.padding.top;
                local.width =
                    (local.width - fragment.paint.padding.left - fragment.paint.padding.right)
                        .max(0.0);
                local.height =
                    (local.height - fragment.paint.padding.top - fragment.paint.padding.bottom)
                        .max(0.0);
            }
            let id = self.next_property_id;
            self.next_property_id += 1;
            self.RegisterProperty(fragment, PaintPropertyRole::BackgroundClip, id);
            let clip = Arc::new(
                ClipPaintPropertyNode {
                    geometry_mapper_cache: Default::default(),
                    layout_clip_rects: Default::default(),
                    lifecycle: Default::default(),
                    id,
                    parent: Some(node.properties.nodes.clip.clone()),
                    local_transform_space: node.properties.nodes.transform.clone(),
                    rect: Some(PropertySpaceRect(
                        node.PixelSnappedRect(&local),
                        property_space_delta,
                    )),
                    radii: PaintCornerRadii::default(),
                    clip_path: None,
                    clip_path_error: None,
                    pixel_moving_filter: None,
                }
                .with_layout_clips(),
            );
            node.fixed_background_properties =
                Some(crate::paint_property_tree::PropertyTreeState {
                    transform: self.viewport_properties.nodes.transform.clone(),
                    clip,
                    effect: node.properties.nodes.effect.clone(),
                });
        }
        node.self_cull_rect = node.cull_rect;
        // LocalBorderBoxProperties excludes this box's overflow clip and
        // scroll translation. ContentsProperties includes both (Blink
        // paint_property_tree_builder.cc UpdateForChildren).
        let border_box_properties = node.properties.clone();
        let mut scroll_node = None;
        if ClipsOverflow(fragment) || fragment.paint.scroll_container.is_some() {
            let mut clip_inset = IncludedBorderEdges(&fragment.paint);
            if let Some(outsets) = fragment.paint.overflow_clip_margin_outsets {
                clip_inset = Edges {
                    top: -outsets.top,
                    right: -outsets.right,
                    bottom: -outsets.bottom,
                    left: -outsets.left,
                };
            }
            let mut local = PaintRect {
                x: node.paint_offset.x + clip_inset.left,
                y: node.paint_offset.y + clip_inset.top,
                width: (fragment.size.width - clip_inset.left - clip_inset.right).max(0.0),
                height: (fragment.size.height - clip_inset.top - clip_inset.bottom).max(0.0),
            };
            if let Some(scrollbars) = fragment.paint.scrollbars.as_ref() {
                if !scrollbars.uses_overlay_scrollbars {
                    if let Some(vertical) = scrollbars.vertical.as_ref() {
                        let thickness = vertical.track_size.width;
                        if vertical.track_offset.x <= clip_inset.left {
                            local.x += thickness;
                        }
                        local.width = (local.width - thickness).max(0.0);
                    }
                    if let Some(horizontal) = scrollbars.horizontal.as_ref() {
                        let thickness = horizontal.track_size.height;
                        if horizontal.track_offset.y <= clip_inset.top {
                            local.y += thickness;
                        }
                        local.height = (local.height - thickness).max(0.0);
                    }
                }
            }
            // Source: core/paint/paint_property_tree_builder.cc:3286-3307.
            // Layout mapping retains the unsnapped clip independently of the
            // painting clip. Scroll container output is the authoritative
            // physical clip geometry when present.
            let mut precise_layout_clip = local;
            local = node.PixelSnappedRect(&local);
            if let Some(scroll) = fragment.paint.scroll_container {
                precise_layout_clip = PaintRect {
                    x: node.paint_offset.x + scroll.container_rect.offset.x,
                    y: node.paint_offset.y + scroll.container_rect.offset.y,
                    width: scroll.container_rect.size.width,
                    height: scroll.container_rect.size.height,
                };
                local = node.PixelSnappedRect(&precise_layout_clip);
            }
            if let Some(viewport) = fragment.paint.svg_viewport {
                if let Some(clip) = viewport.clip {
                    local = PaintRect {
                        x: clip.offset.x,
                        y: clip.offset.y,
                        width: clip.size.width,
                        height: clip.size.height,
                    };
                    precise_layout_clip = local;
                }
            }
            node.local_clip = Some(local);
            if let Some(style) =
                style.filter(|_| fragment.paint.source_kind != NodeKind::kSvgViewportContainer)
            {
                if let Some(outsets) = fragment.paint.overflow_clip_margin_outsets {
                    node.local_clip_radii = ExpandCornerRadiiEdges(
                        ResolveCornerRadii(
                            style,
                            &fragment.paint.border_sides,
                            fragment.size,
                            Edges::default(),
                        ),
                        outsets,
                        Size {
                            width: local.width,
                            height: local.height,
                        },
                    );
                } else {
                    node.local_clip_radii = ResolveCornerRadii(
                        style,
                        &fragment.paint.border_sides,
                        Size {
                            width: local.width + clip_inset.left + clip_inset.right,
                            height: local.height + clip_inset.top + clip_inset.bottom,
                        },
                        clip_inset,
                    );
                }
                node.local_clip_radius = UniformCornerRadius(&node.local_clip_radii);
            }
            if let Some(mapped) = MapRectToRoot(local, &node.transforms) {
                node.clip = Some(node.clip.map_or(mapped, |clip| Intersect(clip, &mapped)));
                let scrolling_contents = fragment.paint.scroll_container.is_some()
                    || matches!(
                        fragment.paint.overflow_x,
                        Overflow::kScroll | Overflow::kAuto
                    )
                    || matches!(
                        fragment.paint.overflow_y,
                        Overflow::kScroll | Overflow::kAuto
                    );
                let content_cull = if scrolling_contents {
                    Expand(mapped, ScrollCullExpansion(fragment, is_root))
                } else {
                    mapped
                };
                node.cull_rect = Some(node.cull_rect.map_or(content_cull, |cull_rect| {
                    Intersect(cull_rect, &content_cull)
                }));
            }
            node.properties.clip_id = self.next_property_id;
            self.next_property_id += 1;
            self.RegisterProperty(
                fragment,
                PaintPropertyRole::OverflowClip,
                node.properties.clip_id,
            );
            let mut excluding_overlay = precise_layout_clip;
            if let Some(scrollbars) = fragment
                .paint
                .scrollbars
                .as_ref()
                .filter(|bars| bars.uses_overlay_scrollbars)
            {
                if let Some(vertical) = &scrollbars.vertical {
                    if vertical.track_offset.x <= clip_inset.left {
                        excluding_overlay.x += vertical.track_size.width;
                    }
                    excluding_overlay.width =
                        (excluding_overlay.width - vertical.track_size.width).max(0.0);
                }
                if let Some(horizontal) = &scrollbars.horizontal {
                    if horizontal.track_offset.y <= clip_inset.top {
                        excluding_overlay.y += horizontal.track_size.height;
                    }
                    excluding_overlay.height =
                        (excluding_overlay.height - horizontal.track_size.height).max(0.0);
                }
            }
            node.properties.nodes.clip = Arc::new(
                ClipPaintPropertyNode {
                    geometry_mapper_cache: Default::default(),
                    layout_clip_rects: Default::default(),
                    lifecycle: Default::default(),
                    id: node.properties.clip_id,
                    parent: Some(node.properties.nodes.clip.clone()),
                    local_transform_space: node.properties.nodes.transform.clone(),
                    rect: Some(PropertySpaceRect(local, property_space_delta)),
                    radii: node.local_clip_radii,
                    clip_path: None,
                    clip_path_error: None,
                    pixel_moving_filter: None,
                }
                .with_layout_clips()
                .with_layout_clip_values(
                    Some(PropertySpaceRect(precise_layout_clip, property_space_delta)),
                    None,
                    (fragment.paint.source_kind != NodeKind::kSvgViewportContainer)
                        .then_some(PropertySpaceRect(excluding_overlay, property_space_delta)),
                ),
            );
            node.applies_clip = true;
            node.applies_overflow_clip = true;
            if fragment.paint.source_kind != NodeKind::kSvgViewportContainer
                && (fragment.paint.scroll_container.is_some()
                    || matches!(
                        fragment.paint.overflow_x,
                        Overflow::kScroll | Overflow::kAuto
                    )
                    || matches!(
                        fragment.paint.overflow_y,
                        Overflow::kScroll | Overflow::kAuto
                    ))
            {
                node.properties.scroll_id = self.next_property_id;
                self.next_property_id += 1;
                self.RegisterProperty(
                    fragment,
                    PaintPropertyRole::Scroll,
                    node.properties.scroll_id,
                );
                scroll_node = Some(Arc::new(ScrollPaintPropertyNode {
                    // Source: core/paint/paint_property_tree_builder.cc:3614-3635.
                    // The remaining fields still await their original Layout
                    // facts; PaintInput rejects incomplete non-root scrolls.
                    compositor: {
                        let mut state =
                            crate::paint_property_tree::ScrollCompositorState::default();
                        state.prevent_scroll_axis_locking =
                            fragment.paint.prevent_scroll_axis_locking;
                        // Source: core/paint/paint_property_tree_builder.cc:3436-3439.
                        if self.requires_main_thread_for_background_attachment_fixed == Some(true) {
                            state.main_thread_repaint_reasons |= 1;
                        }
                        state.overscroll_behavior = [
                            fragment.paint.overscroll_behavior_x,
                            fragment.paint.overscroll_behavior_y,
                        ];
                        if let Some(scroll) = fragment.paint.scroll_container {
                            state.compositor_element_id = scroll.compositor_element_id;
                            state.snap_container_data_absent =
                                scroll.scroll_snap_type_is_none == Some(true);
                            // Source: core/paint/paint_property_tree_builder.cc:3440-3455;
                            // cc/input/main_thread_scrolling_reason.h:25-37.
                            if scroll.background_needs_repaint_on_scroll == Some(true) {
                                state.main_thread_repaint_reasons |= 1 << 3;
                            }
                            if !(scroll.user_scrollable_horizontal
                                || scroll.user_scrollable_vertical)
                            {
                                state.main_thread_repaint_reasons |= 1 << 2;
                            }
                            for (behavior, scrollable) in state
                                .overscroll_behavior
                                .iter_mut()
                                .zip(scroll.scrollable_axes)
                            {
                                if !scrollable {
                                    *behavior = Default::default();
                                }
                            }
                        }
                        state
                    },
                    lifecycle: Default::default(),
                    id: node.properties.scroll_id,
                    owner_client_id: fragment.paint.display_item_client_id,
                    parent: parent_properties.nodes.nearest_scroll(),
                    overflow_clip: Some(node.properties.nodes.clip.clone()),
                    container_rect: PropertySpaceRect(local, property_space_delta),
                    contents_rect: PaintRect {
                        x: local.x + property_space_delta.x,
                        y: local.y + property_space_delta.y,
                        width: fragment.paint.scroll_size.width.max(local.width),
                        height: fragment.paint.scroll_size.height.max(local.height),
                    },
                    minimum_scroll_offset: fragment
                        .paint
                        .scroll_container
                        .map_or(Offset::default(), |scroll| scroll.minimum_scroll_offset),
                    maximum_scroll_offset: fragment.paint.scroll_container.map_or_else(
                        || Offset {
                            x: (fragment.paint.scroll_size.width - local.width).max(0.0),
                            y: (fragment.paint.scroll_size.height - local.height).max(0.0),
                        },
                        |scroll| scroll.maximum_scroll_offset,
                    ),
                    user_scrollable_horizontal: fragment.paint.scroll_container.map_or_else(
                        || {
                            matches!(
                                fragment.paint.overflow_x,
                                Overflow::kScroll | Overflow::kAuto
                            )
                        },
                        |scroll| scroll.user_scrollable_horizontal,
                    ),
                    user_scrollable_vertical: fragment.paint.scroll_container.map_or_else(
                        || {
                            matches!(
                                fragment.paint.overflow_y,
                                Overflow::kScroll | Overflow::kAuto
                            )
                        },
                        |scroll| scroll.user_scrollable_vertical,
                    ),
                }));
            }
        }

        // Source: core/paint/paint_property_tree_builder.cc
        // UpdateScrollTranslation. Descendant display items stay in the
        // scroller's local coordinate space; the ScrollTranslation property
        // alone moves them. Baking the offset into child geometry and also
        // publishing the property applies every committed scroll twice.
        let geometry_scroll = fragment.paint.scroll_offset;
        let child_origin = node.absolute_offset;
        let mut child_scroll = node.scroll_offset;
        if fragment.paint.establishes_paint_state
            && (geometry_scroll.x != 0.0 || geometry_scroll.y != 0.0)
        {
            let local_scroll = geometry_scroll;
            child_scroll.x += local_scroll.x;
            child_scroll.y += local_scroll.y;
        }
        let children_are_flex_or_grid_items = fragment.paint.has_source
            && matches!(
                fragment.paint.display,
                Display::kFlex
                    | Display::kInlineFlex
                    | Display::kGrid
                    | Display::kInlineGrid
                    | Display::kGridLanes
            );
        let svg_view_box = SvgViewBoxTransform(fragment);
        let mut child_properties = node.properties.clone();
        let mut child_transforms = node.transforms.clone();
        let preserves_3d = style.is_some_and(|style| style.preserves_3d);
        let mut child_rendering_context_id = if preserves_3d {
            node.properties.nodes.transform.rendering_context_id
        } else {
            0
        };
        let pre_perspective_should_flatten = !preserves_3d;
        // This is captured before Perspective: perspective changes flattening
        // for descendant geometry, but not backface inheritance.
        let child_can_inherit_backface = pre_perspective_should_flatten;
        let mut child_should_flatten = pre_perspective_should_flatten;
        if let Some((depth, origin)) = style.and_then(|style| {
            style
                .perspective
                .map(|depth| (depth, style.perspective_origin))
        }) {
            if !depth.is_finite() || depth <= 0.0 {
                panic!("used perspective depth must be finite and positive");
            }
            let pivot = Offset {
                x: node.paint_offset.x
                    + property_space_delta.x
                    + origin.pixels.x
                    + fragment.size.width * origin.percentages.x / 100.0,
                y: node.paint_offset.y
                    + property_space_delta.y
                    + origin.pixels.y
                    + fragment.size.height * origin.percentages.y / 100.0,
            };
            let mut perspective = TransformMatrix::default();
            perspective.values[11] = -1.0 / depth;
            let matrix = MultiplyTransforms(
                &TranslationTransform(pivot.x, pivot.y),
                &MultiplyTransforms(&perspective, &TranslationTransform(-pivot.x, -pivot.y)),
            );
            let id = self.next_property_id;
            self.next_property_id += 1;
            self.RegisterProperty(fragment, PaintPropertyRole::Perspective, id);
            child_properties.nodes.transform = Arc::new(TransformPaintPropertyNode {
                compositor: TransformCompositorState(fragment, PaintPropertyRole::Perspective),
                geometry_mapper_cache: Default::default(),
                scroll_parent_scroll_translation: None,
                lifecycle: Default::default(),
                id,
                parent: Some(child_properties.nodes.transform.clone()),
                matrix,
                origin: [0.0; 3],
                flattens_inherited_transform: pre_perspective_should_flatten,
                rendering_context_id: child_rendering_context_id,
                backface_visibility: PaintBackfaceVisibility::Inherited,
                scroll: None,
                sticky_constraint: None,
                direct_compositing_reasons: Vec::new(),
            });
            child_properties.transform_id = id;
            child_transforms.push(matrix);
            // Chromium's perspective node itself must not flatten, otherwise
            // its child 3D transforms never receive perspective.
            child_should_flatten = false;
        }
        // Chromium inserts ReplacedContentTransform after Perspective and
        // before ScrollTranslation, so the content transform scrolls with the
        // replaced element rather than wrapping its scroll node.
        if let Some(matrix) = svg_view_box {
            let id = self.next_property_id;
            self.next_property_id += 1;
            self.RegisterProperty(fragment, PaintPropertyRole::ReplacedContentTransform, id);
            child_properties.nodes.transform = Arc::new(TransformPaintPropertyNode {
                compositor: TransformCompositorState(
                    fragment,
                    PaintPropertyRole::ReplacedContentTransform,
                ),
                geometry_mapper_cache: Default::default(),
                scroll_parent_scroll_translation: None,
                lifecycle: Default::default(),
                id,
                parent: Some(child_properties.nodes.transform.clone()),
                matrix,
                origin: [0.0; 3],
                flattens_inherited_transform: child_should_flatten,
                rendering_context_id: child_rendering_context_id,
                backface_visibility: PaintBackfaceVisibility::Inherited,
                scroll: None,
                sticky_constraint: None,
                direct_compositing_reasons: Vec::new(),
            });
            child_properties.transform_id = id;
            child_should_flatten = true;
            child_rendering_context_id = 0;
        }
        // UpdateScrollTranslation in Blink records the negative local offset
        // and associates a scroll node with that transform. Overflow:hidden
        // may have a translation without a scroll node.
        if scroll_node.is_some()
            || (establishes && (geometry_scroll.x != 0.0 || geometry_scroll.y != 0.0))
        {
            let id = self.next_property_id;
            self.next_property_id += 1;
            self.RegisterProperty(fragment, PaintPropertyRole::ScrollTranslation, id);
            child_properties.nodes.transform = Arc::new(TransformPaintPropertyNode {
                compositor: TransformCompositorState(
                    fragment,
                    PaintPropertyRole::ScrollTranslation,
                ),
                geometry_mapper_cache: Default::default(),
                scroll_parent_scroll_translation: None,
                lifecycle: Default::default(),
                id,
                parent: Some(child_properties.nodes.transform.clone()),
                matrix: TranslationTransform(-geometry_scroll.x, -geometry_scroll.y),
                origin: [0.0; 3],
                flattens_inherited_transform: child_should_flatten,
                rendering_context_id: child_rendering_context_id,
                backface_visibility: PaintBackfaceVisibility::Inherited,
                scroll: scroll_node,
                sticky_constraint: None,
                direct_compositing_reasons: Vec::new(),
            });
            child_properties.transform_id = id;
            child_transforms.push(TranslationTransform(-geometry_scroll.x, -geometry_scroll.y));
        }
        node.contents_properties = child_properties.clone();
        node.properties = border_box_properties;
        if let Some(svg) = svg_view_box {
            child_transforms.push(svg);
        }
        let own_keys: Vec<_> = self
            .property_keys
            .range(own_key_start..self.next_property_id)
            .map(|(&id, &key)| (id, key))
            .collect();
        node.cache = Some(NodeCache {
            inherited,
            children: ChildContext {
                origin: child_origin,
                paint_space: paint_space_origin,
                clip: node.clip,
                cull: node.cull_rect,
                transforms: child_transforms,
                opacity: node.opacity,
                scroll: child_scroll,
                fixed_background_acts_as_scroll: node.fixed_background_acts_as_scroll,
                properties: child_properties,
                rendering_context_id: child_rendering_context_id,
                should_flatten_inherited_transform: child_should_flatten,
                can_inherit_backface_visibility: child_can_inherit_backface,
                flex_or_grid_items: children_are_flex_or_grid_items,
                svg: svg_view_box,
            },
            revision: fragment.pre_paint_revision,
            subtree_revision: fragment.pre_paint_subtree_revision,
            raw_properties: node.properties.clone(),
            raw_contents: node.contents_properties.clone(),
            raw_fixed_background: node.fixed_background_properties.clone(),
            raw_mask: node.mask_properties.clone(),
            raw_opacity: node.opacity,
            raw_compositing: node.applies_compositing_layer,
            subtree_keys: own_keys.clone(),
            own_keys,
            nodes: 1,
            computed_pass: self.walk_generation,
            physical_children: Vec::new(),
        });
        node
    }
}

#[cfg(test)]
mod tests {
    use super::{ClipsOverflow, PaintRect, PixelSnappedRect, ResolveClipPath};
    use layoutng_assembly::fragment_tree::FragmentNode;
    use layoutng_assembly::internal::layout_input::{
        NodeKind, Offset, Overflow, PaintPathCommand, PaintPathVerb, Size,
    };
    use layoutng_assembly::internal::paint_input::{
        ClipPathCircle, ClipPathLength, ClipPathPaint, ClipPathRadiusKind, ClipPathShapeRadius,
    };

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
    fn svg_root_overflow_clip_follows_computed_overflow() {
        let mut svg = FragmentNode::default();
        svg.paint.has_source = true;
        svg.paint.establishes_paint_state = true;
        svg.paint.source_kind = NodeKind::kSvgRoot;

        // Chromium's NeedsOverflowClip(LayoutSVGRoot) permits outermost SVG
        // ink overflow when the computed overflow axes are visible.
        assert!(!ClipsOverflow(&svg));
        svg.paint.overflow_x = Overflow::kHidden;
        svg.paint.overflow_y = Overflow::kHidden;
        assert!(ClipsOverflow(&svg));
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
}
