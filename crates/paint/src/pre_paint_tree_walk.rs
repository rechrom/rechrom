#![allow(non_snake_case)]

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;

use layoutng_assembly::fragment_tree::{FragmentKind, FragmentNode};
use layoutng_assembly::internal::layout_input::{
    BorderLineStyle, Display, Edges, FloatSide, NodeKind, Offset, Overflow, PaintImage,
    PaintPathCommand, PaintPathVerb, Position, Size, TransformMatrix,
};
use layoutng_assembly::internal::paint_input::{
    BackgroundBox, ClipPathPaint, PaintBlendMode, PaintCornerRadii, PaintFilterOperation,
    PaintFilterType, PaintMaskLayer, PaintStyleData,
};

use crate::background_geometry::{ResolveBackgroundTile, ResolveBackgroundTileSize};
use crate::border_shape_utils::{
    ExpandCornerRadiiEdges, IncludedBorderEdges, ResolveCornerRadii, UniformCornerRadius,
};
use crate::geometry_mapper::{
    MapRectToRoot, MultiplyTransforms, ResolveLocalTransformAroundOrigin, TranslationTransform,
};
use crate::paint_engine::DisplayMaskLayer;
use crate::paint_property_tree::{
    ClipPaintPropertyNode, EffectPaintPropertyNode, PaintPropertyKey, PaintPropertyNodeStore,
    PaintPropertyOwner, PaintPropertyRole, PropertyNodeUpdater, ScrollPaintPropertyNode,
    TransformPaintPropertyNode,
};
use crate::PaintRect;

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
    // Flat replay folds ancestor scroll into geometry. Pixel snapping belongs
    // to the stable border-box property space, before that translation.
    pub(crate) paint_snap_offset: Offset,
    pub root_clip: Option<PaintRect>,
    pub local_clip: Option<PaintRect>,
    pub local_clip_radius: f64,
    pub local_clip_radii: PaintCornerRadii,
    pub local_clip_path: Vec<PaintPathCommand>,
    pub clip_path_even_odd: bool,
    pub clip: Option<PaintRect>,
    pub cull_rect: Option<PaintRect>,
    pub self_cull_rect: Option<PaintRect>,
    pub transforms: Vec<TransformMatrix>,
    pub local_transforms: Vec<TransformMatrix>,
    pub opacity: f32,
    pub scroll_offset: Offset,
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
    pub applies_clip_path: bool,
    pub applies_effect: bool,
    pub applies_compositing_layer: bool,
    pub applies_overlap_compositing_layer: bool,
    pub applies_opacity: bool,
    pub applies_blend: bool,
    pub applies_filter: bool,
    pub applies_mask: bool,
    pub paints_outline: bool,
    pub needs_float_phase: bool,
    pub needs_outline_phase: bool,
    pub needs_overflow_controls_phase: bool,
    pub outline_rects: Vec<PaintRect>,
    pub mask_layers: Vec<DisplayMaskLayer>,
    pub properties: PropertyTreeState,
    pub contents_properties: PropertyTreeState,
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
            root_clip: None,
            local_clip: None,
            local_clip_radius: 0.0,
            local_clip_radii: PaintCornerRadii::default(),
            local_clip_path: Vec::new(),
            clip_path_even_odd: false,
            clip: None,
            cull_rect: None,
            self_cull_rect: None,
            transforms: Vec::new(),
            local_transforms: Vec::new(),
            opacity: 1.0,
            scroll_offset: Offset::default(),
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
            applies_clip_path: false,
            applies_effect: false,
            applies_compositing_layer: false,
            applies_overlap_compositing_layer: false,
            applies_opacity: false,
            applies_blend: false,
            applies_filter: false,
            applies_mask: false,
            paints_outline: true,
            // Hand-built trees remain conservative; the completed PrePaint
            // postorder replaces these hints with actual subtree requirements.
            needs_float_phase: true,
            needs_outline_phase: true,
            needs_overflow_controls_phase: true,
            outline_rects: Vec::new(),
            mask_layers: Vec::new(),
            properties: PropertyTreeState::default(),
            contents_properties: PropertyTreeState::default(),
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
        let mut layers = self.mask_layers.iter();
        let mut rect = layers.next()?.clip_rect;
        for layer in layers {
            rect = Union(rect, &layer.clip_rect);
        }
        Some(rect)
    }

    pub(crate) fn PixelSnappedRect(&self, rect: &PaintRect) -> PaintRect {
        PixelSnappedRectInPropertySpace(rect, self.paint_snap_offset)
    }

    pub(crate) fn EnclosingRect(&self, rect: PaintRect) -> PaintRect {
        // DrawingRecorder bounds are enclosed in the same local paint space
        // as its pixels. The flat adapter folds ancestor scroll into geometry;
        // enclosing there would leak fractional scroll into raster bounds.
        EnclosingRectInPropertySpace(rect, self.paint_snap_offset)
    }

    pub(crate) fn PixelSnappedOffset(&self, offset: Offset) -> Offset {
        Offset {
            x: (offset.x + self.paint_snap_offset.x + 0.5).floor() - self.paint_snap_offset.x,
            y: (offset.y + self.paint_snap_offset.y + 0.5).floor() - self.paint_snap_offset.y,
        }
    }

    pub(crate) fn RoundedPaintY(&self, y: f64) -> f64 {
        (y + self.paint_snap_offset.y).round() - self.paint_snap_offset.y
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
    properties: PropertyTreeState,
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
    properties: PropertyTreeState,
    flex_or_grid_items: bool,
    svg: Option<TransformMatrix>,
}
struct NodeCache {
    inherited: InheritedContext,
    children: ChildContext,
    revision: u64,
    subtree_revision: u64,
    raw_properties: PropertyTreeState,
    raw_contents: PropertyTreeState,
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
    // Drawing records keep their local coordinate anchor while the current
    // ScrollTranslation advances. Native retained-geometry/coverage proofs
    // admit this projection; raw caches and recorded properties share it.
    pub(crate) scroll_recording_basis: Option<BTreeMap<PaintPropertyOwner, Offset>>,
    logical_tree_changed: bool,
    viewport: Option<PaintRect>,
    viewport_properties: PropertyTreeState,
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
            scroll_recording_basis: None,
            logical_tree_changed: false,
            viewport: None,
            viewport_properties: PropertyTreeState::default(),
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
    if fragment.paint.source_kind != NodeKind::kSvgRoot || fragment.paint.svg_view_box.is_none() {
        return None;
    }
    let view_box = fragment.paint.svg_view_box.as_ref().unwrap();
    let viewport_width = fragment.content_size.width;
    let viewport_height = fragment.content_size.height;
    if !(view_box.width > 0.0)
        || !(view_box.height > 0.0)
        || !(viewport_width > 0.0)
        || !(viewport_height > 0.0)
    {
        return None;
    }
    let scale_x = viewport_width / view_box.width;
    let scale_y = viewport_height / view_box.height;
    let mut transform = TransformMatrix::default();
    if view_box.preserve_none {
        transform.values[0] = scale_x;
        transform.values[5] = scale_y;
        transform.values[12] =
            fragment.paint.border.left + fragment.paint.padding.left - view_box.x * scale_x;
        transform.values[13] =
            fragment.paint.border.top + fragment.paint.padding.top - view_box.y * scale_y;
        return Some(transform);
    }
    let scale = if view_box.slice {
        scale_x.max(scale_y)
    } else {
        scale_x.min(scale_y)
    };
    transform.values[0] = scale;
    transform.values[5] = scale;
    let alignment_offset = |spare: f64, alignment: i8| {
        if alignment < 0 {
            0.0
        } else if alignment > 0 {
            spare
        } else {
            spare * 0.5
        }
    };
    transform.values[12] = fragment.paint.border.left
        + fragment.paint.padding.left
        + alignment_offset(viewport_width - view_box.width * scale, view_box.align_x)
        - view_box.x * scale;
    transform.values[13] = fragment.paint.border.top
        + fragment.paint.padding.top
        + alignment_offset(viewport_height - view_box.height * scale, view_box.align_y)
        - view_box.y * scale;
    Some(transform)
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
    let has_sticky_translation = position == Position::kSticky
        && (fragment.paint.sticky_offset.x != 0.0 || fragment.paint.sticky_offset.y != 0.0);
    let has_svg_viewport_translation = fragment.paint.source_kind == NodeKind::kSvgRoot
        && (geometry.paint_offset.x != 0.0 || geometry.paint_offset.y != 0.0);
    if establishes
        && style.is_some_and(|style| {
            has_svg_viewport_translation
                || has_sticky_translation
                || style.transform.is_some()
                || style.will_change_transform
                || fragment
                    .paint
                    .svg_shape
                    .as_ref()
                    .is_some_and(|shape| shape.local_transform.is_some())
        })
    {
        let style = style.unwrap();
        geometry.applies_transform = true;
        if has_svg_viewport_translation {
            let translation = TranslationTransform(
                (geometry.paint_offset.x + property_space_delta.x + 0.5).floor()
                    - property_space_delta.x,
                (geometry.paint_offset.y + property_space_delta.y + 0.5).floor()
                    - property_space_delta.y,
            );
            transforms.push(translation);
            paint_space_origin = geometry.absolute_offset;
            geometry.paint_offset = Offset::default();
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
        if let Some(transform) = style.transform.as_ref() {
            // Blink snaps the layout paint offset in the current property
            // parent, whose ScrollTranslation remains a separate ancestor.
            // Flat replay already folds that scroll into geometry. Snap before
            // folding it, then convert back; snapping the flat offset first
            // leaks fractional scroll into the CSS Transform's local matrix.
            let resolved = ResolveLocalTransformAroundOrigin(
                transform,
                Offset {
                    x: geometry.paint_offset.x + property_space_delta.x,
                    y: geometry.paint_offset.y + property_space_delta.y,
                },
                fragment.size,
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
        if let Some(local_transform) = fragment
            .paint
            .svg_shape
            .as_ref()
            .and_then(|shape| shape.local_transform)
        {
            transforms.push(local_transform);
        }
    }
    geometry.paint_space_origin = paint_space_origin;
    geometry
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
            NodeKind::kSvgShape | NodeKind::kSvgGroup
        )
    {
        Offset {
            x: paint_space_origin.x - child.offset.x,
            y: paint_space_origin.y - child.offset.y,
        }
    } else {
        child_origin
    }
}

// Blink UpdateOverflowClip/UpdateInnerBorderRadiusClip uses paint_offset in
// context_.current.transform space (paint_property_tree_builder.cc:3148,3290).
// Standalone ChildGeometryOrigin already folds scroll into flat paint_offset,
// while the property tree also has the real ScrollTranslation. Convert only
// into the property parent: P * (clip + delta) == F * clip. The same delta
// places paint-offset snapping in that parent's space. Comparing the complete linear parts proves this is a pure
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
fn PixelSnappedRect(rect: &PaintRect) -> PaintRect {
    let left = (rect.x + 0.5).floor();
    let top = (rect.y + 0.5).floor();
    let right = (rect.x + rect.width + 0.5).floor();
    let bottom = (rect.y + rect.height + 0.5).floor();
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

fn PixelSnappedRectInPropertySpace(rect: &PaintRect, delta: Offset) -> PaintRect {
    let mut snapped = PixelSnappedRect(&PropertySpaceRect(*rect, delta));
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
fn FilterCullOutset(filters: &[PaintFilterOperation]) -> f64 {
    let mut outset = 0.0;
    for filter in filters {
        if filter.r#type == PaintFilterType::kBlur {
            outset += filter.amount * 3.0;
        } else if filter.r#type == PaintFilterType::kDropShadow {
            outset += filter.blur_radius * 3.0 + filter.offset.x.abs().max(filter.offset.y.abs());
        }
    }
    outset
}

// cpp: paint/pre_paint_tree_walk.cc:108-118
fn ClipsOverflow(node: &FragmentNode) -> bool {
    if !node.paint.has_source || !node.paint.establishes_paint_state {
        return false;
    }
    if node.paint.source_kind == NodeKind::kSvgRoot {
        return true;
    }
    node.paint.overflow_x != Overflow::kVisible || node.paint.overflow_y != Overflow::kVisible
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
            PaintFilterType::kHueRotate | PaintFilterType::kDropShadow => {}
        }
    }
}

// cpp: paint/pre_paint_tree_walk.cc:148-218
fn ResolveClipPath(
    input: &ClipPathPaint,
    origin: Offset,
    reference_box: Size,
    output: &mut Vec<PaintPathCommand>,
) -> PaintRect {
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
            x: pixels.x + fractions.x * reference_box.width,
            y: pixels.y + fractions.y * reference_box.height,
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
    PaintRect {
        x: left,
        y: top,
        width: (right - left).max(0.0),
        height: (bottom - top).max(0.0),
    }
}

// cpp: paint/pre_paint_tree_walk.cc:220-241
fn MaskBox(fragment: &FragmentNode, absolute_offset: Offset, box_type: BackgroundBox) -> PaintRect {
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
        || found.rgba8.len() != found.width as usize * found.height as usize * 4
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
    layers: &[PaintMaskLayer],
) -> Vec<DisplayMaskLayer> {
    let mut output = Vec::with_capacity(layers.len());
    for layer in layers {
        if !layer.image.position.x.is_finite() || !layer.image.position.y.is_finite() {
            panic!("mask position must be finite");
        }
        let positioning = MaskPositioningBox(fragment, absolute_offset, layer.image.origin);
        let clip = MaskBox(fragment, absolute_offset, layer.image.clip);
        let image = if layer.image.shader.is_none() {
            Some(FindMaskImage(fragment, layer.image.resource_id))
        } else {
            None
        };
        let tile_size = ResolveBackgroundTileSize(&layer.image, &positioning, image);
        let tile_geometry = ResolveBackgroundTile(&layer.image, &positioning, tile_size);
        let tile = if layer.image.shader.is_some() {
            PixelSnappedRectInPropertySpace(&tile_geometry.tile_rect, property_space_delta)
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
        style.map_or(1.0, |style| style.opacity)
    } else {
        1.0
    };
    node.opacity = parent_opacity * local_opacity;
    if !node.applies_effect {
        node.properties.effect_id = parent_effect_id;
    }
    // Geometry still follows the physical ancestry. Effects inherit from the
    // restored logical ancestry; rebuild only this effect's parent reference.
    let local_effect_count =
        usize::from(node.applies_opacity || node.applies_blend || node.applies_mask)
            + usize::from(node.applies_filter);
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
    let count = usize::from(node.applies_opacity || node.applies_blend || node.applies_mask)
        + usize::from(node.applies_filter);
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

struct ContentUpdate {
    node: *mut PaintTreeNode<'static>,
    source: Option<Arc<FragmentNode>>,
    revision: u64,
    subtree_revision: u64,
}

impl PrePaintTreeWalk {
    pub(crate) fn ResetScrollRecordingBasis(&mut self) {
        if self.scroll_recording_basis.take().is_some() {
            // A different anchor changes flattened geometry. Rebuild it rather
            // than marking its old projected coordinates as newly certified.
            self.retained_tree_valid = false;
        }
    }

    pub(crate) fn PrepareScrollRecordingBasis(&mut self, root: &FragmentNode) {
        let Some(basis) = self.scroll_recording_basis.as_ref() else {
            return;
        };
        let viewport = root
            .paint
            .resources
            .as_ref()
            .and_then(|r| r.viewport)
            .map(|v| PaintRect {
                x: 0.0,
                y: 0.0,
                width: v.size.width as f64,
                height: v.size.height as f64,
            });
        if root.pre_paint_revision == 0
            || viewport != self.viewport
            || !self
                .retained_tree
                .as_ref()
                .is_some_and(|tree| SameSourceIdentity(tree.fragment.as_deref().unwrap(), root))
        {
            self.ResetScrollRecordingBasis();
            return;
        }
        if self.property_node_store.scroll_offsets() == *basis {
            return;
        }
        if let Some((store, _)) = self.property_node_store.retained_scroll_update(basis) {
            // Advance the authoritative immutable snapshot lifecycle, also
            // when projecting back to the recording view. Walk refreshes the
            // final states; raw contexts continue describing this same basis.
            self.property_node_store = store;
        } else {
            self.ResetScrollRecordingBasis();
        }
    }

    fn GeometryScrollOffset(&self, fragment: &FragmentNode) -> Offset {
        if fragment.paint.display_item_client_is_just_created && !fragment.pre_paint_owner_retained
        {
            // Native address reuse does not prove the old recording owner.
            return fragment.paint.scroll_offset;
        }
        let owner = PaintPropertyOwner::NativeFragment {
            client: fragment.paint.display_item_client_id,
            fragment: fragment.paint.display_item_fragment,
        };
        self.scroll_recording_basis
            .as_ref()
            .and_then(|basis| basis.get(&owner))
            .copied()
            .unwrap_or(fragment.paint.scroll_offset)
    }

    fn SameContext(&self, a: &InheritedContext, b: &InheritedContext) -> bool {
        a.offset == b.offset
            && a.paint_space == b.paint_space
            && a.clip == b.clip
            && a.cull == b.cull
            && a.transforms == b.transforms
            && a.opacity == b.opacity
            && a.scroll == b.scroll
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
            root_clip,
            local_clip,
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
            is_paint_layer,
            is_stacking_context,
            is_root,
            stacking_level,
            applies_transform,
            applies_clip,
            applies_root_clip,
            applies_overflow_clip,
            applies_clip_path,
            applies_effect,
            applies_opacity,
            applies_blend,
            applies_filter,
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
        if physical_children.len() != fragment.children.len()
            || !physical_children
                .iter()
                .zip(&fragment.children)
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
                inherited.properties.clone(),
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
        for (&pointer, child) in physical_children.iter().zip(&fragment.children) {
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
                inherited.properties.clone(),
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
        let old_children = std::mem::take(&mut node.children);
        let same_children = old_children.len() == fragment.children.len()
            && old_children
                .iter()
                .zip(&fragment.children)
                .all(|(old, source)| SameSourceIdentity(old.fragment.as_deref().unwrap(), source));
        if !same_children {
            self.needs_post_update = true;
        }
        let mut previous_children = old_children.into_iter();
        let mut keys = own_keys;
        let mut nodes = 1;
        for (index, child) in fragment.children.iter().enumerate() {
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

    fn ContextForChild(&self, child: &FragmentNode, base: &ChildContext) -> InheritedContext {
        if child.paint.fixed_to_view {
            // Blink switches fixed-to-view descendants to the LayoutView's
            // fixed-position context. It is above document ScrollTranslation
            // and keeps the viewport clip. The ordinary child context made
            // fixed boxes move by -scrollTop with the page.
            return InheritedContext {
                offset: Offset::default(),
                paint_space: Offset::default(),
                clip: self.viewport,
                cull: self.viewport,
                transforms: Vec::new(),
                opacity: base.opacity,
                scroll: Offset::default(),
                properties: self.viewport_properties.clone(),
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
            properties: base.properties.clone(),
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
        let viewport = root
            .paint
            .resources
            .as_ref()
            .and_then(|resources| resources.viewport)
            .map(|viewport| PaintRect {
                x: 0.0,
                y: 0.0,
                width: viewport.size.width as f64,
                height: viewport.size.height as f64,
            });
        let environment_changed = viewport != self.viewport;
        if environment_changed || (viewport.is_some() && self.viewport_properties.clip_id == 0) {
            self.viewport = viewport;
            self.viewport_properties = PropertyTreeState::default();
            if let Some(rect) = viewport {
                let id = self.next_property_id;
                self.next_property_id += 1;
                self.viewport_properties.clip_id = id;
                self.viewport_properties.nodes.clip = Arc::new(ClipPaintPropertyNode {
                    lifecycle: Default::default(),
                    id,
                    parent: Some(self.viewport_properties.nodes.clip.clone()),
                    local_transform_space: self.viewport_properties.nodes.transform.clone(),
                    rect: Some(rect),
                    radii: PaintCornerRadii::default(),
                    clip_path: Vec::new(),
                    clip_path_even_odd: false,
                    pixel_moving_filter: None,
                });
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
            properties: self.viewport_properties.clone(),
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
        parent_properties: PropertyTreeState,
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
            properties: parent_properties.clone(),
            root: is_root,
            flex_or_grid_item: is_flex_or_grid_item,
        };
        let mut node = Box::new(PaintTreeNode::default());
        node.SetFragment(fragment);
        node.is_root = is_root;
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
                style.map_or(1.0, |style| style.opacity)
            } else {
                1.0
            };
        let position = if fragment.paint.has_source {
            fragment.paint.position
        } else {
            Position::kStatic
        };
        let positioned_z_index = style.is_some_and(|style| {
            style.z_index.is_some() && (position != Position::kStatic || is_flex_or_grid_item)
        });
        node.stacking_level = if positioned_z_index {
            style.unwrap().z_index.unwrap()
        } else {
            0
        };
        node.is_paint_layer = is_root
            || fragment.paint.source_kind == NodeKind::kSvgRoot
            || (establishes
                && style.is_some_and(|style| {
                    position != Position::kStatic
                        || positioned_z_index
                        || style.transform.is_some()
                        || style.will_change_transform
                        || style.opacity != 1.0
                        || !style.filters.is_empty()
                        || style.blend_mode != PaintBlendMode::kNormal
                        || style.isolate_blending
                        || style.clip_path.is_some()
                        || !style.mask_images.is_empty()
                }));
        // Exported native PaintLayer determines whether this owner paints as
        // a layer. A line/text fragment can borrow the same LayoutObject but
        // does not become another layer owner.
        if fragment.paint.paint_layer_client_id != 0 {
            node.is_paint_layer = fragment.kind == FragmentKind::kBox
                && establishes
                && fragment.paint.paint_layer_is_self_painting;
        }
        node.is_stacking_context = is_root
            || (establishes
                && style.is_some_and(|style| {
                    positioned_z_index
                        || position == Position::kFixed
                        || position == Position::kSticky
                        || style.transform.is_some()
                        || style.will_change_transform
                        || style.opacity != 1.0
                        || !style.filters.is_empty()
                        || style.blend_mode != PaintBlendMode::kNormal
                        || style.isolate_blending
                        || style.clip_path.is_some()
                        || !style.mask_images.is_empty()
                }));

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
                && (fragment.paint.sticky_offset.x != 0.0 || fragment.paint.sticky_offset.y != 0.0)
            {
                source_roles.push(PaintPropertyRole::StickyTranslation);
            }
            if style.is_some_and(|style| style.transform.is_some()) {
                source_roles.push(PaintPropertyRole::Transform);
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
            debug_assert_eq!(source_roles.len(), node.local_transforms.len());
            let mut operations: Vec<_> = source_roles
                .into_iter()
                .zip(node.local_transforms.iter().copied())
                .collect();
            if style.is_some_and(|style| style.will_change_transform && style.transform.is_none()) {
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
            for (index, (role, matrix)) in operations.iter().enumerate() {
                // UpdatePaintOffsetTranslation/UpdateTransform establish a
                // local paint space. Their standalone replay matrix includes
                // folded ancestor scroll in the paint offset; property parents
                // already contain ScrollTranslation. Express this reset matrix
                // in its actual property-parent space, so P_parent*M_property
                // equals the existing F_parent*M_flat. Constant sticky/SVG
                // local matrices and will-change identity slots do not reset
                // paint_offset and must keep inheriting their scroll anchor.
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
                let reasons = if *role == PaintPropertyRole::Transform
                    && style.is_some_and(|style| style.will_change_transform)
                {
                    vec!["WillChangeTransform"]
                } else {
                    Vec::new()
                };
                self.RegisterProperty(fragment, *role, id);
                node.properties.nodes.transform = Arc::new(TransformPaintPropertyNode {
                    lifecycle: Default::default(),
                    id,
                    parent: Some(node.properties.nodes.transform.clone()),
                    matrix: property_matrix,
                    origin: [0.0; 3],
                    scroll: None,
                    direct_compositing_reasons: reasons,
                });
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
                    node.cull_rect = Some(Expand(cull_rect, FilterCullOutset(&style.filters)));
                }
            }
            node.applies_compositing_layer = style.will_change_transform;
            node.applies_opacity = style.opacity != 1.0;
            node.applies_blend =
                style.blend_mode != PaintBlendMode::kNormal || style.isolate_blending;
            node.applies_filter = !style.filters.is_empty();
            if !style.mask_images.is_empty() {
                node.mask_layers = ResolveMaskLayers(
                    fragment,
                    node.paint_offset,
                    property_space_delta,
                    &style.mask_images,
                );
                node.applies_mask = !node.mask_layers.is_empty();
                if node.applies_mask {
                    if let Some(cull_rect) = node.cull_rect {
                        let mut mask_bounds = node.mask_layers[0].clip_rect;
                        for mask in node.mask_layers.iter().skip(1) {
                            mask_bounds = Union(mask_bounds, &mask.clip_rect);
                        }
                        if let Some(mapped) = MapRectToRoot(mask_bounds, &node.transforms) {
                            node.cull_rect = Some(Intersect(cull_rect, &mapped));
                        }
                    }
                }
            }
        }
        if establishes {
            if let Some(style) = style {
                if let Some(clip_path) = style.clip_path.as_ref() {
                    let path_bounds = ResolveClipPath(
                        clip_path,
                        node.paint_offset,
                        fragment.size,
                        &mut node.local_clip_path,
                    );
                    node.clip_path_even_odd = clip_path.even_odd;
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
                    node.properties.nodes.clip = Arc::new(ClipPaintPropertyNode {
                        lifecycle: Default::default(),
                        id: node.properties.clip_id,
                        parent: Some(node.properties.nodes.clip.clone()),
                        local_transform_space: node.properties.nodes.transform.clone(),
                        rect: Some(PropertySpaceRect(path_bounds, property_space_delta)),
                        radii: PaintCornerRadii::default(),
                        clip_path: node
                            .local_clip_path
                            .iter()
                            .cloned()
                            .map(|mut command| {
                                if command.verb != PaintPathVerb::kClose {
                                    command.point.x += property_space_delta.x;
                                    command.point.y += property_space_delta.y;
                                    if matches!(
                                        command.verb,
                                        PaintPathVerb::kQuadraticTo
                                            | PaintPathVerb::kConicTo
                                            | PaintPathVerb::kCubicTo
                                    ) {
                                        command.control1.x += property_space_delta.x;
                                        command.control1.y += property_space_delta.y;
                                    }
                                    if command.verb == PaintPathVerb::kCubicTo {
                                        command.control2.x += property_space_delta.x;
                                        command.control2.y += property_space_delta.y;
                                    }
                                }
                                command
                            })
                            .collect(),
                        clip_path_even_odd: node.clip_path_even_odd,
                        pixel_moving_filter: None,
                    });
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
            || node.applies_mask;
        if node.applies_opacity || node.applies_blend || node.applies_mask {
            node.properties.effect_id = self.next_property_id;
            self.next_property_id += 1;
            let style = style.expect("effect must have source style");
            self.RegisterProperty(
                fragment,
                PaintPropertyRole::Effect,
                node.properties.effect_id,
            );
            node.properties.nodes.effect = Arc::new(EffectPaintPropertyNode {
                lifecycle: Default::default(),
                id: node.properties.effect_id,
                parent: Some(node.properties.nodes.effect.clone()),
                local_transform_space: node.properties.nodes.transform.clone(),
                output_clip: Some(node.properties.nodes.clip.clone()),
                opacity: style.opacity,
                blend_mode: style.blend_mode,
                filters: Vec::new(),
                isolates_blending: style.isolate_blending,
                has_mask: false,
                is_mask: false,
                // UpdateEffect's kAdditionalEffectCompositingTrigger includes
                // the source will-change:transform reason. Opacity alone does
                // not justify adding a direct compositing reason.
                direct_compositing_reasons: if style.will_change_transform {
                    vec!["WillChangeTransform"]
                } else {
                    Vec::new()
                },
            });
        }
        if node.applies_mask {
            // CSS masks establish a stacking context and isolated layer even
            // when a standalone native owner has no exported PaintLayer yet.
            node.is_paint_layer = true;
            node.is_stacking_context = true;
            let mask_id = self.next_property_id;
            self.next_property_id += 1;
            self.RegisterProperty(fragment, PaintPropertyRole::Mask, mask_id);
            let mask_effect = Arc::new(EffectPaintPropertyNode {
                lifecycle: Default::default(),
                id: mask_id,
                parent: Some(node.properties.nodes.effect.clone()),
                local_transform_space: node.properties.nodes.transform.clone(),
                output_clip: Some(mask_output_clip.clone()),
                opacity: 1.0,
                blend_mode: PaintBlendMode::kNormal,
                filters: Vec::new(),
                isolates_blending: false,
                has_mask: false,
                is_mask: true,
                direct_compositing_reasons: Vec::new(),
            });
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
            node.properties.nodes.clip = Arc::new(ClipPaintPropertyNode {
                lifecycle: Default::default(),
                id: clip_id,
                parent: Some(mask_output_clip),
                local_transform_space: node.properties.nodes.transform.clone(),
                rect: Some(PropertySpaceRect(mask_bounds, property_space_delta)),
                radii: PaintCornerRadii::default(),
                clip_path: Vec::new(),
                clip_path_even_odd: false,
                pixel_moving_filter: None,
            });
        }
        if node.applies_filter {
            let style = style.expect("filter must have source style");
            node.properties.effect_id = self.next_property_id;
            self.next_property_id += 1;
            self.RegisterProperty(
                fragment,
                PaintPropertyRole::Filter,
                node.properties.effect_id,
            );
            // UpdateFilter follows UpdateEffect and has its own persistent
            // ObjectPaintProperties slot (paint_property_tree_builder.cc:2630).
            node.properties.nodes.effect = Arc::new(EffectPaintPropertyNode {
                lifecycle: Default::default(),
                id: node.properties.effect_id,
                parent: Some(node.properties.nodes.effect.clone()),
                local_transform_space: node.properties.nodes.transform.clone(),
                output_clip: Some(node.properties.nodes.clip.clone()),
                opacity: 1.0,
                blend_mode: PaintBlendMode::kNormal,
                filters: style.filters.clone(),
                isolates_blending: false,
                has_mask: false,
                is_mask: false,
                direct_compositing_reasons: if style.will_change_transform {
                    vec!["WillChangeTransform"]
                } else {
                    Vec::new()
                },
            });
            if style.filters.iter().any(|filter| {
                matches!(
                    filter.r#type,
                    PaintFilterType::kBlur | PaintFilterType::kDropShadow
                )
            }) {
                node.properties.clip_id = self.next_property_id;
                self.next_property_id += 1;
                self.RegisterProperty(
                    fragment,
                    PaintPropertyRole::PixelMovingFilterClipExpander,
                    node.properties.clip_id,
                );
                node.properties.nodes.clip = Arc::new(ClipPaintPropertyNode {
                    lifecycle: Default::default(),
                    id: node.properties.clip_id,
                    parent: Some(node.properties.nodes.clip.clone()),
                    local_transform_space: node.properties.nodes.transform.clone(),
                    rect: None,
                    radii: PaintCornerRadii::default(),
                    clip_path: Vec::new(),
                    clip_path_even_odd: false,
                    pixel_moving_filter: Some(node.properties.nodes.effect.clone()),
                });
            }
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
            local = node.PixelSnappedRect(&local);
            if let Some(scroll) = fragment.paint.scroll_container {
                local = node.PixelSnappedRect(&PaintRect {
                    x: node.paint_offset.x + scroll.container_rect.offset.x,
                    y: node.paint_offset.y + scroll.container_rect.offset.y,
                    width: scroll.container_rect.size.width,
                    height: scroll.container_rect.size.height,
                });
            }
            node.local_clip = Some(local);
            if let Some(style) = style {
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
            node.properties.nodes.clip = Arc::new(ClipPaintPropertyNode {
                lifecycle: Default::default(),
                id: node.properties.clip_id,
                parent: Some(node.properties.nodes.clip.clone()),
                local_transform_space: node.properties.nodes.transform.clone(),
                rect: Some(PropertySpaceRect(local, property_space_delta)),
                radii: node.local_clip_radii,
                clip_path: Vec::new(),
                clip_path_even_odd: false,
                pixel_moving_filter: None,
            });
            node.applies_clip = true;
            node.applies_overflow_clip = true;
            if fragment.paint.scroll_container.is_some()
                || matches!(
                    fragment.paint.overflow_x,
                    Overflow::kScroll | Overflow::kAuto
                )
                || matches!(
                    fragment.paint.overflow_y,
                    Overflow::kScroll | Overflow::kAuto
                )
            {
                node.properties.scroll_id = self.next_property_id;
                self.next_property_id += 1;
                self.RegisterProperty(
                    fragment,
                    PaintPropertyRole::Scroll,
                    node.properties.scroll_id,
                );
                scroll_node = Some(Arc::new(ScrollPaintPropertyNode {
                    lifecycle: Default::default(),
                    id: node.properties.scroll_id,
                    parent: parent_properties.nodes.nearest_scroll(),
                    overflow_clip: Some(node.properties.nodes.clip.clone()),
                    container_rect: PropertySpaceRect(local, property_space_delta),
                    contents_rect: PaintRect {
                        x: local.x + property_space_delta.x,
                        y: local.y + property_space_delta.y,
                        width: fragment.paint.scroll_size.width.max(local.width),
                        height: fragment.paint.scroll_size.height.max(local.height),
                    },
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

        let geometry_scroll = self.GeometryScrollOffset(fragment);
        let child_origin = if establishes {
            Offset {
                x: node.absolute_offset.x - geometry_scroll.x,
                y: node.absolute_offset.y - geometry_scroll.y,
            }
        } else {
            node.absolute_offset
        };
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
                Display::kFlex | Display::kGrid | Display::kGridLanes
            );
        let svg_view_box = SvgViewBoxTransform(fragment);
        let mut child_properties = node.properties.clone();
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
                lifecycle: Default::default(),
                id,
                parent: Some(child_properties.nodes.transform.clone()),
                matrix: TranslationTransform(-geometry_scroll.x, -geometry_scroll.y),
                origin: [0.0; 3],
                scroll: scroll_node,
                direct_compositing_reasons: Vec::new(),
            });
            child_properties.transform_id = id;
        }
        // Create the SVG viewBox node before descending so grandchildren keep
        // the complete chain, rather than inheriting an ID overwritten later.
        if let Some(matrix) = svg_view_box {
            let id = self.next_property_id;
            self.next_property_id += 1;
            self.RegisterProperty(fragment, PaintPropertyRole::ReplacedContentTransform, id);
            child_properties.nodes.transform = Arc::new(TransformPaintPropertyNode {
                lifecycle: Default::default(),
                id,
                parent: Some(child_properties.nodes.transform.clone()),
                matrix,
                origin: [0.0; 3],
                scroll: None,
                direct_compositing_reasons: Vec::new(),
            });
            child_properties.transform_id = id;
        }
        node.contents_properties = child_properties.clone();
        node.properties = border_box_properties;
        let mut child_transforms = node.transforms.clone();
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
                properties: child_properties,
                flex_or_grid_items: children_are_flex_or_grid_items,
                svg: svg_view_box,
            },
            revision: fragment.pre_paint_revision,
            subtree_revision: fragment.pre_paint_subtree_revision,
            raw_properties: node.properties.clone(),
            raw_contents: node.contents_properties.clone(),
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
