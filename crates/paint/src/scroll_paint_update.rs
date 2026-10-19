//! Retained paint coverage and scroll-only paint-property updates.
//!
//! Like CullRectUpdater/UseCachedSubsequence, the old painted region stays in
//! contents coordinates. Updating a scroll translation never moves that region.
#![allow(non_snake_case)]

use crate::{
    display_item_id::DisplayItemIdType,
    geometry_mapper::{MapRectToRoot, MultiplyTransforms},
    paint_engine::{DisplayItem, PaintArtifact, PaintPhase},
    paint_property_tree::{
        ClipPaintPropertyNode, EffectPaintPropertyNode, PaintPropertyNodeStore, PaintPropertyOwner,
        PaintPropertyTrees, PropertyTreeState, TransformPaintPropertyNode,
    },
    pre_paint_tree_walk::{PaintTreeNode, PrePaintTreeWalk},
    PaintRect,
};
use layoutng_assembly::{
    fragment_tree::{FragmentNode, StickyPositionPaintData},
    internal::layout_input::{Offset, Size, TransformMatrix},
    internal::paint_input::BackgroundAttachment,
};
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

#[derive(Clone)]
struct FragmentCoverage {
    offset: Offset,
    size: Size,
    sticky_offset: Offset,
    paint_offset: Offset,
    // None means painting was uncullable, not an empty recorded region.
    // Keep coverage in the exact root coordinate basis used during recording.
    // A retained scroll changes only translation, even below a fixed rotation.
    recorded_world: TransformMatrix,
    properties: PropertyTreeState,
    controls: Option<OverflowControlGeometry>,
}

/// Only overflow controls need flat paint geometry during a retained scroll.
/// Ordinary descendants retain their property-space coverage, not another tree.
#[derive(Clone)]
struct OverflowControlGeometry {
    paint_snap_offset: Offset,
    properties: crate::pre_paint_tree_walk::PropertyTreeState,
    contents_properties: crate::pre_paint_tree_walk::PropertyTreeState,
    transforms: Vec<TransformMatrix>,
}

#[derive(Clone)]
struct MaskOutput {
    effect_id: u64,
    transform: Arc<TransformPaintPropertyNode>,
    recorded_rect: PaintRect,
    recorded_world: TransformMatrix,
}

#[derive(Clone)]
pub struct RetainedScrollPaintState {
    fragments: Vec<FragmentCoverage>,
    // Recorded identity, never traversal order or a borrowed fragment address.
    // Logical out-of-flow reparenting can differ from native fragment order.
    fragment_indices: HashMap<(u64, u64, u64, u32), usize>,
    masks: Vec<MaskOutput>,
    coverage_groups: Vec<CoverageGroup>,
    /// Whether Paint folded a live scroll offset into background drawing
    /// commands. Property-driven backgrounds keep their existing records.
    has_scroll_dependent_background: bool,
}

#[derive(Clone)]
struct CoverageGroup {
    properties: PropertyTreeState,
    recorded_world: TransformMatrix,
    // Fast group proofs. If either enclosure contains the corresponding
    // mapped output, every member passes the exact per-fragment predicate.
    recorded_visible_intersection: Option<PaintRect>,
    recorded_cull_intersection: PaintRect,
    members: Vec<(PaintRect, Option<PaintRect>)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollPaintInvalidation {
    MissingCoverage,
    FragmentChanged,
    CoverageExceeded,
    PropertyChanged,
    UnsupportedTransform,
    ScrollbarChanged,
    CaretNeedsPaint,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ScrollPropertyUpdate {
    pub updated_chunks: usize,
    pub updated_scrollbar_records: usize,
}

fn world_transform(
    node: &Arc<TransformPaintPropertyNode>,
    worlds: &mut BTreeMap<u64, TransformMatrix>,
) -> TransformMatrix {
    if let Some(matrix) = worlds.get(&node.id) {
        return *matrix;
    }
    let parent = node
        .parent
        .as_ref()
        .map_or_else(TransformMatrix::default, |parent| {
            world_transform(parent, worlds)
        });
    let matrix = MultiplyTransforms(&parent, &node.matrix);
    worlds.insert(node.id, matrix);
    matrix
}

fn property_world(
    state: &PropertyTreeState,
    worlds: &mut BTreeMap<u64, TransformMatrix>,
) -> Option<TransformMatrix> {
    let matrix = world_transform(&state.transform, worlds);
    let m = &matrix.values;
    (m.iter().all(|v| v.is_finite())
        && [2, 3, 6, 7, 8, 9, 11, 14].iter().all(|&i| m[i] == 0.0)
        && m[10] == 1.0
        && m[15] == 1.0
        && m[0] * m[5] - m[1] * m[4] != 0.0)
        .then_some(matrix)
}

/// Keep the recorded region fixed, as CullRectUpdater does for a retained
/// scroll translation. Prove the complete affine linear map is unchanged;
/// then root-space regions differ by translation only. No inverse AABB of a
/// rotated rectangle is used as a claim of recorded source coverage.
fn rebase_rect(
    rect: PaintRect,
    from: &TransformMatrix,
    to: &TransformMatrix,
) -> Option<(PaintRect, Offset)> {
    let a = &from.values;
    let b = &to.values;
    if !a.iter().chain(b).all(|value| value.is_finite())
        || (0..16).any(|i| i != 12 && i != 13 && a[i] != b[i])
    {
        return None;
    }
    let dx = b[12] - a[12];
    let dy = b[13] - a[13];
    let result = PaintRect {
        x: rect.x + dx,
        y: rect.y + dy,
        ..rect
    };
    let roundoff = Offset {
        x: 16.0
            * f64::EPSILON
            * (rect.x.abs() + a[12].abs() + b[12].abs() + result.x.abs() + rect.width.abs()),
        y: 16.0
            * f64::EPSILON
            * (rect.y.abs() + a[13].abs() + b[13].abs() + result.y.abs() + rect.height.abs()),
    };
    [
        result.x,
        result.y,
        result.width,
        result.height,
        roundoff.x,
        roundoff.y,
    ]
    .iter()
    .all(|value| value.is_finite())
    .then_some((result, roundoff))
}

fn identity(node: &PaintTreeNode<'_>) -> (u64, u64, u64, u32) {
    let f = node
        .fragment
        .as_deref()
        .expect("PrePaint owns a real fragment");
    (
        f.node_id,
        f.fragment_instance_id,
        f.paint.display_item_client_id,
        f.paint.display_item_fragment,
    )
}

fn gather<'a, 'f>(node: &'a PaintTreeNode<'f>, nodes: &mut Vec<&'a PaintTreeNode<'f>>) {
    nodes.push(node);
    for child in &node.children {
        gather(child, nodes);
    }
}

fn intersection(a: PaintRect, b: PaintRect) -> PaintRect {
    let x = a.x.max(b.x);
    let y = a.y.max(b.y);
    PaintRect {
        x,
        y,
        width: (a.x + a.width).min(b.x + b.width).max(x) - x,
        height: (a.y + a.height).min(b.y + b.height).max(y) - y,
    }
}

fn mask_outputs(nodes: &[&PaintTreeNode<'_>]) -> Option<BTreeMap<u64, PaintRect>> {
    let mut outputs = BTreeMap::new();
    for node in nodes {
        if !node.applies_mask {
            continue;
        }
        let mut masks = node.mask_layers.iter();
        let mut rect = masks.next()?.clip_rect;
        for mask in masks {
            // Same rectangle union as PrePaint's mask cull, including empty
            // layers. This is an output enclosure, never an opaque-fill claim.
            let x = rect.x.min(mask.clip_rect.x);
            let y = rect.y.min(mask.clip_rect.y);
            rect = PaintRect {
                x,
                y,
                width: (rect.x + rect.width).max(mask.clip_rect.x + mask.clip_rect.width) - x,
                height: (rect.y + rect.height).max(mask.clip_rect.y + mask.clip_rect.height) - y,
            };
        }
        let mask = node.mask_properties.as_ref()?;
        let content_effect = mask.effect.parent.as_ref()?;
        outputs.insert(content_effect.id, MapRectToRoot(rect, &node.transforms)?);
    }
    Some(outputs)
}

fn effect_mask_output(
    effect: &Arc<EffectPaintPropertyNode>,
    own: &BTreeMap<u64, PaintRect>,
    cache: &mut BTreeMap<u64, Option<PaintRect>>,
) -> Option<Option<PaintRect>> {
    if let Some(bounds) = cache.get(&effect.id) {
        return Some(*bounds);
    }
    let mut bounds = match &effect.parent {
        Some(parent) => effect_mask_output(parent, own, cache)?,
        None => None,
    };
    if let Some(&mask) = own.get(&effect.id) {
        bounds = Some(bounds.map_or(mask, |parent| intersection(parent, mask)));
    } else if effect.has_mask {
        // Legacy standalone wrapper without a formal source is not reusable.
        return None;
    }
    cache.insert(effect.id, bounds);
    Some(bounds)
}

#[derive(Clone, Copy, Default)]
struct ClipOutput {
    visible: Option<PaintRect>,
    hard: Option<PaintRect>,
}

fn clip_output(
    clip: &Arc<ClipPaintPropertyNode>,
    viewport: Option<&Arc<ClipPaintPropertyNode>>,
    worlds: &mut BTreeMap<u64, TransformMatrix>,
    cache: &mut BTreeMap<u64, ClipOutput>,
) -> Option<ClipOutput> {
    if let Some(bounds) = cache.get(&clip.id) {
        return Some(*bounds);
    }
    if clip.pixel_moving_filter.is_some() {
        return None;
    }
    let mut output = match &clip.parent {
        Some(parent) => clip_output(parent, viewport, worlds, cache)?,
        None => ClipOutput::default(),
    };
    if let Some(rect) = clip.rect {
        let world = world_transform(&clip.local_transform_space, worlds);
        let rect = MapRectToRoot(rect, &[world])?;
        output.visible = Some(
            output
                .visible
                .map_or(rect, |parent| intersection(parent, rect)),
        );
        // Identify the actual viewport-owned property entity, never a guessed
        // numeric ID or a rectangle which merely happens to match the viewport.
        if !viewport.is_some_and(|viewport| clip.lifecycle.same_node(&viewport.lifecycle)) {
            output.hard = Some(
                output
                    .hard
                    .map_or(rect, |parent| intersection(parent, rect)),
            );
        }
    } else if clip.clip_path.is_some() {
        return None;
    }
    cache.insert(clip.id, output);
    Some(output)
}

fn visible_output(
    state: &PropertyTreeState,
    masks: &BTreeMap<u64, PaintRect>,
    mask_cache: &mut BTreeMap<u64, Option<PaintRect>>,
    viewport: Option<&Arc<ClipPaintPropertyNode>>,
    worlds: &mut BTreeMap<u64, TransformMatrix>,
    clip_cache: &mut BTreeMap<u64, ClipOutput>,
    fringe: f64,
) -> Option<Option<PaintRect>> {
    // BorderBoxProperties excludes this owner's overflow clip, unlike
    // node.clip (the geometry used for its children). Its background, border
    // and shadow need the real border-box chain's complete visible region.
    let clips = clip_output(&state.clip, viewport, worlds, clip_cache)?;
    let mut visible = clips.visible;
    if let Some(rect) = &mut visible {
        if !rect.is_empty() {
            rect.x -= fringe;
            rect.y -= fringe;
            rect.width += 2.0 * fringe;
            rect.height += 2.0 * fringe;
        }
    }
    // Expand viewport sampling before applying every real non-viewport hard
    // output enclosure. Clip/mask exterior is known transparent; it does not
    // require newly recorded source pixels. Rounded/path bounding rectangles
    // are conservative enclosures, never claims that their corners are filled.
    if let Some(hard) = clips.hard {
        visible = Some(visible.map_or(hard, |clip| intersection(clip, hard)));
    }
    if let Some(mask) = effect_mask_output(&state.effect, masks, mask_cache)? {
        visible = Some(visible.map_or(mask, |clip| intersection(clip, mask)));
    }
    Some(visible)
}

impl RetainedScrollPaintState {
    pub(crate) fn capture(
        tree: &PaintTreeNode<'_>,
        store: &PaintPropertyNodeStore,
    ) -> Option<Self> {
        let mut nodes = Vec::new();
        gather(tree, &mut nodes);
        let mut fragments = Vec::with_capacity(nodes.len());
        let mut fragment_indices = HashMap::with_capacity(nodes.len());
        let mut worlds = BTreeMap::new();
        let masks = mask_outputs(&nodes)?;
        let mut mask_cache = BTreeMap::new();
        let mut clip_cache = BTreeMap::new();
        let mut retained_masks = Vec::new();
        let mut coverage_groups = Vec::<CoverageGroup>::new();
        let mut coverage_group_indices: HashMap<((usize, usize, usize), [u64; 16]), usize> =
            HashMap::new();
        let has_scroll_dependent_background = nodes
            .iter()
            .any(|node| has_scroll_dependent_background(node));
        for node in &nodes {
            if !node.applies_mask {
                continue;
            }
            let effect_id = node.mask_properties.as_ref()?.effect.parent.as_ref()?.id;
            let world = property_world(&node.properties.nodes, &mut worlds)?;
            retained_masks.push(MaskOutput {
                effect_id,
                transform: node.properties.nodes.transform.clone(),
                recorded_rect: *masks.get(&effect_id)?,
                recorded_world: world,
            });
        }
        for node in nodes {
            let f = node.fragment.as_deref()?;
            // Pixel-moving effects are retained property-tree effects. Their
            // unsupported coverage mapping is isolated below to that state;
            // it must not disable retained scrolling for unrelated layers.
            let raw_world = world_transform(&node.properties.nodes.transform, &mut worlds);
            // A singular transform has no drawable area. Keep its retained
            // fragment/property identity, but omit it from the coverage proof.
            // One scale(0) menu or animation must not disable retained scroll
            // painting for the whole document.
            let world = property_world(&node.properties.nodes, &mut worlds);
            let recorded_visible = world.and_then(|_| {
                visible_output(
                    &node.properties.nodes,
                    &masks,
                    &mut mask_cache,
                    store.viewport_clip(),
                    &mut worlds,
                    &mut clip_cache,
                    0.0,
                )
            });
            let identity = identity(node);
            if fragment_indices.insert(identity, fragments.len()).is_some() {
                return None;
            }
            if let (Some(world), Some(recorded_visible)) = (world, recorded_visible) {
                if f.paint.has_source && f.paint.style.visible && !f.paint.hidden {
                    if let Some(recorded_cull) = node.self_cull_rect {
                        let key = (
                            state_cache_key(&node.properties.nodes),
                            world.values.map(f64::to_bits),
                        );
                        if let Some(&index) = coverage_group_indices.get(&key) {
                            let group = &mut coverage_groups[index];
                            group.recorded_cull_intersection =
                                intersection(group.recorded_cull_intersection, recorded_cull);
                            group.recorded_visible_intersection =
                                match (group.recorded_visible_intersection, recorded_visible) {
                                    (Some(a), Some(b)) => Some(intersection(a, b)),
                                    _ => None,
                                };
                            group.members.push((recorded_cull, recorded_visible));
                        } else {
                            coverage_group_indices.insert(key, coverage_groups.len());
                            coverage_groups.push(CoverageGroup {
                                properties: node.properties.nodes.clone(),
                                recorded_world: world,
                                recorded_visible_intersection: recorded_visible,
                                recorded_cull_intersection: recorded_cull,
                                members: vec![(recorded_cull, recorded_visible)],
                            });
                        }
                    }
                }
            }
            fragments.push(FragmentCoverage {
                offset: f.offset,
                size: f.size,
                sticky_offset: f.paint.sticky_offset,
                paint_offset: node.paint_offset,
                recorded_world: world.unwrap_or(raw_world),
                properties: node.properties.nodes.clone(),
                controls: f
                    .paint
                    .scrollbars
                    .as_ref()
                    .map(|_| OverflowControlGeometry {
                        paint_snap_offset: node.paint_snap_offset,
                        properties: node.properties.clone(),
                        contents_properties: node.contents_properties.clone(),
                        transforms: node.transforms.clone(),
                    }),
            });
        }
        Some(Self {
            fragments,
            fragment_indices,
            masks: retained_masks,
            coverage_groups,
            has_scroll_dependent_background,
        })
    }
}

// Validate current exported inputs while placing them in the recorded slots.
// Unknown, duplicate or missing identities still reject the whole update.
// ScrollPaintReuseRejected stages: 1 missing identity, 2 duplicate identity,
// 3 changed geometry/unowned sticky placement, 4 scroll-dependent background,
// 5 property owner/constraint change.
fn collect_current_fragments<'a>(
    f: &'a FragmentNode,
    retained: &RetainedScrollPaintState,
    nodes: &mut [Option<&'a FragmentNode>],
    offsets: &mut BTreeMap<PaintPropertyOwner, Offset>,
    sticky: &mut BTreeMap<PaintPropertyOwner, (Offset, StickyPositionPaintData)>,
    collected: &mut usize,
) -> bool {
    let key = (
        f.node_id,
        f.fragment_instance_id,
        f.paint.display_item_client_id,
        f.paint.display_item_fragment,
    );
    let Some(&index) = retained.fragment_indices.get(&key) else {
        browser_tracing::instant(
            "paint",
            "ScrollPaintReuseRejected",
            &[
                ("stage", 1.0),
                ("node_id", f.node_id as f64),
                ("fragment_instance_id", f.fragment_instance_id as f64),
                (
                    "display_item_client_id",
                    f.paint.display_item_client_id as f64,
                ),
            ],
        );
        return false;
    };
    if nodes[index].is_some() {
        browser_tracing::instant(
            "paint",
            "ScrollPaintReuseRejected",
            &[("stage", 2.0), ("node_id", f.node_id as f64)],
        );
        return false;
    }
    let old = &retained.fragments[index];
    let unowned_sticky_change = f.paint.sticky_offset != old.sticky_offset
        && (f.paint.sticky_constraint.is_none() || f.paint.display_item_client_id == 0);
    if f.offset != old.offset || f.size != old.size || unowned_sticky_change {
        browser_tracing::instant(
            "paint",
            "ScrollPaintReuseRejected",
            &[
                ("stage", 3.0),
                ("node_id", f.node_id as f64),
                ("offset_changed", (f.offset != old.offset) as u8 as f64),
                ("size_changed", (f.size != old.size) as u8 as f64),
                (
                    "sticky_changed",
                    (f.paint.sticky_offset != old.sticky_offset) as u8 as f64,
                ),
                ("old_sticky_y", old.sticky_offset.y),
                ("new_sticky_y", f.paint.sticky_offset.y),
            ],
        );
        return false;
    }
    nodes[index] = Some(f);
    *collected += 1;
    if f.paint.display_item_client_id != 0 {
        let owner = PaintPropertyOwner::NativeFragment {
            client: f.paint.display_item_client_id,
            fragment: f.paint.display_item_fragment,
        };
        if let Some(constraint) = f.paint.sticky_constraint {
            let current = (f.paint.sticky_offset, constraint);
            if sticky
                .insert(owner, current)
                .is_some_and(|old| old != current)
            {
                return false;
            }
        }
        if offsets
            .insert(owner, f.paint.scroll_offset)
            .is_some_and(|old| old != f.paint.scroll_offset)
        {
            return false;
        }
    }
    // LayoutSVGHiddenContainer remains in Layout snapshots for resource users,
    // but PrePaint creates no ordinary owners beneath that boundary. Traverse
    // the same owner-selected children when validating retained identities.
    crate::pre_paint_tree_walk::OrdinaryPaintChildren(f)
        .all(|child| collect_current_fragments(child, retained, nodes, offsets, sticky, collected))
}

fn has_scroll_dependent_background(node: &PaintTreeNode<'_>) -> bool {
    let Some(fragment) = node.fragment.as_deref() else {
        return false;
    };
    if !fragment.paint.has_source || fragment.paint.background_transfers_to_view {
        return false;
    }
    fragment
        .paint
        .style
        .background_images
        .iter()
        .any(|layer| match layer.attachment {
            BackgroundAttachment::kScroll => false,
            BackgroundAttachment::kLocal => fragment.paint.scroll_container.is_some(),
            BackgroundAttachment::kFixed => {
                let independently_recorded =
                    fragment.paint.can_composite_background_attachment_fixed
                        && !node.fixed_background_acts_as_scroll
                        && fragment.paint.background_paint_location.PaintsBorderBox()
                        && fragment.paint.style.background_images.len() == 1
                        && (layer.shader.is_some() || layer.resource_id != 0);
                !node.fixed_background_acts_as_scroll && !independently_recorded
            }
        })
}

fn contains(outer: PaintRect, inner: PaintRect, roundoff: Offset) -> bool {
    inner.is_empty()
        || (inner.x >= outer.x - roundoff.x
            && inner.y >= outer.y - roundoff.y
            && inner.x + inner.width <= outer.x + outer.width + roundoff.x
            && inner.y + inner.height <= outer.y + outer.height + roundoff.y)
}

fn refreshed(
    state: &PropertyTreeState,
    trees: &mut PaintPropertyTrees,
) -> Option<PropertyTreeState> {
    if trees.transform_for(&state.transform).is_none()
        || trees.clip_for(&state.clip).is_none()
        || trees.effect_for(&state.effect).is_none()
    {
        // Fresh property nodes without native-owner slots still belong to the
        // retained graph. Rebind their actual ancestors through the updated
        // scroll snapshots; do not mistake an incomplete directory for loss.
        let mut referenced = state.clone();
        trees.clear_state_change_markers(&mut referenced);
    }
    let transform = trees.transform_for(&state.transform)?.clone();
    let clip = trees.clip_for(&state.clip)?.clone();
    let effect = trees.effect_for(&state.effect)?.clone();
    (transform.lifecycle.same_node(&state.transform.lifecycle)
        && clip.lifecycle.same_node(&state.clip.lifecycle)
        && effect.lifecycle.same_node(&state.effect.lifecycle))
    .then_some(PropertyTreeState {
        transform,
        clip,
        effect,
    })
}

fn refreshed_cached<'a>(
    state: &PropertyTreeState,
    trees: &mut PaintPropertyTrees,
    cache: &'a mut HashMap<(usize, usize, usize), PropertyTreeState>,
) -> Option<&'a PropertyTreeState> {
    let key = state_cache_key(state);
    // Coverage only reads the mapped state. Clone its three Arc references
    // once when publishing a chunk, rather than for every fragment validation.
    Some(match cache.entry(key) {
        std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
        std::collections::hash_map::Entry::Vacant(entry) => entry.insert(refreshed(state, trees)?),
    })
}

fn state_cache_key(state: &PropertyTreeState) -> (usize, usize, usize) {
    (
        Arc::as_ptr(&state.transform) as usize,
        Arc::as_ptr(&state.clip) as usize,
        Arc::as_ptr(&state.effect) as usize,
    )
}

/// Retained records require the native unchanged-layout proof and prove
/// current coverage and real property/scrollbar semantics before publication.
/// Rejection leaves the artifact untouched and requests ordinary paint.
pub(crate) fn TryUpdateScrollPropertiesWithPrePaint(
    fragments: &FragmentNode,
    artifact: &mut PaintArtifact,
    prepaint: &mut PrePaintTreeWalk,
) -> Result<ScrollPropertyUpdate, ScrollPaintInvalidation> {
    use ScrollPaintInvalidation::*;
    let started = std::env::var_os("BROWSER_PROFILE_INPUT").map(|_| std::time::Instant::now());
    if artifact.caret.is_some_and(|caret| caret.visible) {
        return Err(CaretNeedsPaint);
    }
    let retained = prepaint
        .retained_scroll_state
        .as_ref()
        .ok_or(MissingCoverage)?;
    if artifact.recorded_properties.len() != artifact.chunks.len() {
        return Err(MissingCoverage);
    }
    if let Some((index, chunk)) = artifact
        .chunks
        .iter()
        .enumerate()
        .find(|(_, chunk)| !chunk.can_match_old_chunk())
    {
        if std::env::var_os("BROWSER_PROFILE_INPUT").is_some() {
            eprintln!("scroll-paint-reject stage=chunk-cache index={} id={:?} cacheable={} just_created={}",
                index, chunk.id, chunk.is_cacheable, chunk.client_is_just_created);
        }
        return Err(PropertyChanged);
    }
    // This path is admitted only by Page's retained native-layout proof. No
    // property builder or temporary PaintTreeNode is needed for clean content.
    // Verify current geometry independently and carry Layout's resolved sticky
    // values into its existing property slot before validating recorded coverage.
    let mut nodes = vec![None; retained.fragments.len()];
    let mut offsets = BTreeMap::new();
    let mut sticky = BTreeMap::new();
    let mut collected = 0;
    let collect_trace = browser_tracing::span("paint", "ScrollPaint.CollectFragments");
    if !collect_current_fragments(
        fragments,
        retained,
        &mut nodes,
        &mut offsets,
        &mut sticky,
        &mut collected,
    ) || collected != retained.fragments.len()
    {
        return Err(FragmentChanged);
    }
    drop(collect_trace);
    // These layers are recorded in the owning box-decoration record. Until
    // they are promoted to Chromium's dedicated scrolling/fixed background
    // chunk, their phase is a function of the current scroll offsets. Reject
    // the property-only path so Paint re-records exact geometry instead of
    // reusing a tile phase from the preceding scroll position.
    let recorded_offsets = prepaint.property_node_store.scroll_offsets();
    let scroll_changed = offsets.iter().any(|(owner, current)| {
        recorded_offsets
            .get(owner)
            .is_some_and(|recorded| recorded != current)
    });
    if scroll_changed && retained.has_scroll_dependent_background {
        browser_tracing::instant("paint", "ScrollPaintReuseRejected", &[("stage", 4.0)]);
        return Err(FragmentChanged);
    }
    let property_trace = browser_tracing::span("paint", "ScrollPaint.RebindPropertyTrees");
    let (updated_store, mut trees) = prepaint
        .property_node_store
        .retained_scroll_update(&offsets, &sticky)
        .ok_or_else(|| {
            browser_tracing::instant("paint", "ScrollPaintReuseRejected", &[("stage", 5.0)]);
            PropertyChanged
        })?;
    drop(property_trace);
    let artifact_to_device_scale = fragments
        .paint
        .resources
        .as_ref()
        .map_or(1.0, |resources| resources.ArtifactToDeviceScale());
    if !artifact_to_device_scale.is_finite() || artifact_to_device_scale <= 0.0 {
        return Err(UnsupportedTransform);
    }
    let mut worlds = BTreeMap::new();
    let mut masks = BTreeMap::new();
    for mask in &retained.masks {
        let transform = trees
            .transform_for(&mask.transform)
            .ok_or(PropertyChanged)?;
        if !transform.lifecycle.same_node(&mask.transform.lifecycle) {
            return Err(PropertyChanged);
        }
        let world = world_transform(transform, &mut worlds);
        let (rect, _) = rebase_rect(mask.recorded_rect, &mask.recorded_world, &world)
            .ok_or(UnsupportedTransform)?;
        masks.insert(mask.effect_id, rect);
    }
    let mut mask_cache = BTreeMap::new();
    let mut clip_cache = BTreeMap::new();
    // Fragments and chunks often share the exact immutable state. Retain one
    // frame-local mapping, including the lifecycle checks, for both consumers.
    let mut refreshed_states = HashMap::new();
    // GeometryMapper maps once per property-tree state. Capture grouped the
    // equivalent retained client predicates during Paint, so the common path
    // also proves their shared cull enclosure once instead of redoing the same
    // map/rebase for every Fragment.
    let coverage_fragments: usize = retained
        .coverage_groups
        .iter()
        .map(|group| group.members.len())
        .sum();
    let mut coverage_trace = browser_tracing::span("paint", "ScrollPaint.ValidateCoverage");
    for group in &retained.coverage_groups {
        let state = refreshed_cached(&group.properties, &mut trees, &mut refreshed_states)
            .ok_or(PropertyChanged)?
            .clone();
        let world = property_world(&state, &mut worlds).ok_or(UnsupportedTransform)?;
        let exact = visible_output(
            &state,
            &masks,
            &mut mask_cache,
            updated_store.viewport_clip(),
            &mut worlds,
            &mut clip_cache,
            0.0,
        )
        .ok_or(UnsupportedTransform)?
        .ok_or(CoverageExceeded)?;
        let fringe = visible_output(
            &state,
            &masks,
            &mut mask_cache,
            updated_store.viewport_clip(),
            &mut worlds,
            &mut clip_cache,
            (1.0 / artifact_to_device_scale).max(1.0),
        )
        .ok_or(UnsupportedTransform)?
        .ok_or(CoverageExceeded)?;
        let exact =
            rebase_rect(exact, &world, &group.recorded_world).ok_or(UnsupportedTransform)?;
        let fringe =
            rebase_rect(fringe, &world, &group.recorded_world).ok_or(UnsupportedTransform)?;
        if group
            .recorded_visible_intersection
            .is_some_and(|visible| contains(visible, exact.0, exact.1))
            || contains(group.recorded_cull_intersection, fringe.0, fringe.1)
        {
            continue;
        }
        // Mixed exact/cull coverage cannot be represented by one intersection.
        // Preserve the original member predicate as the uncommon fallback.
        for &(covered, visible) in &group.members {
            if visible.is_some_and(|visible| contains(visible, exact.0, exact.1)) {
                continue;
            }
            if !contains(covered, fringe.0, fringe.1) {
                if std::env::var_os("BROWSER_PROFILE_INPUT").is_some() {
                    eprintln!("scroll-paint-reject stage=coverage old_cull={covered:?} old_visible={visible:?} required={:?} world={:?}",
                        fringe.0, world.values);
                }
                return Err(CoverageExceeded);
            }
        }
    }
    coverage_trace.set("fragments", coverage_fragments as f64);
    coverage_trace.set("property_states", retained.coverage_groups.len() as f64);
    coverage_trace.set("recorded_spaces", retained.coverage_groups.len() as f64);
    drop(coverage_trace);
    // Many semantic chunks share the exact immutable property snapshots. Map
    // those snapshots once, including the lifecycle checks. Arc addresses are
    // only frame-local memo keys: the artifact holds every source snapshot live.
    // They do not become persistent property or chunk identities.
    let chunks_trace = browser_tracing::span("paint", "ScrollPaint.MapChunkProperties");
    let properties: Vec<_> = artifact.chunks.iter().enumerate().map(|(index, chunk)| {
        let state = &chunk.properties;
        let properties = refreshed_cached(state, &mut trees, &mut refreshed_states).ok_or_else(|| {
            if std::env::var_os("BROWSER_PROFILE_INPUT").is_some() {
                eprintln!("scroll-paint-reject stage=chunk-property-map index={} id={:?} transform={} clip={} effect={}",
                    index, chunk.id, chunk.properties.transform.id, chunk.properties.clip.id, chunk.properties.effect.id);
            }
            PropertyChanged
        })?;
        Ok(properties.clone())
    }).collect::<Result<_, _>>()?;
    drop(chunks_trace);

    // Scrollbar thumb geometry is real scroll-dependent recorded content.
    // Re-record only overflow controls, retaining every other drawing record.
    // Translate the tiny new records back into their original flat coordinate
    // basis so old wrapper ops and recorded_properties remain authoritative.
    // Only overflow-control IDs can be produced here. Build their directory
    // lazily, without allocating entries for the page's ordinary records.
    let mut old_record_indices: Option<BTreeMap<(u64, u32, u8), usize>> = None;
    let mut replacements: Vec<(usize, Vec<DisplayItem>)> = Vec::new();
    let scrollbar_trace = browser_tracing::span("paint", "ScrollPaint.UpdateScrollbars");
    for (old, f) in retained.fragments.iter().zip(&nodes) {
        let f = f.ok_or(FragmentChanged)?;
        if f.paint.scrollbars.is_none() || f.paint.hidden {
            continue;
        }
        let geometry = old.controls.as_ref().ok_or(ScrollbarChanged)?;
        // Record controls in the original flat coordinate basis. Chunk
        // properties/scroll translations are refreshed separately below.
        let mut node = PaintTreeNode::default();
        node.paint_offset = old.paint_offset;
        node.paint_snap_offset = geometry.paint_snap_offset;
        node.properties = geometry.properties.clone();
        node.contents_properties = geometry.contents_properties.clone();
        node.SetFragment(f);
        let mut controls = PaintArtifact::default();
        let context = RefCell::new(crate::paint_context::PaintContext::WithCaret(
            &mut controls,
            None,
        ));
        crate::scrollable_area_painter::ScrollableAreaPainter::new(&node, &context)
            .PaintOverflowControls(&crate::paint_info::PaintInfo::new(
                &context,
                PaintPhase::kOverlayOverflowControls,
            ));
        drop(context);
        for record in &controls.display_items {
            let key = (record.id.client_id, record.id.fragment, record.id.r#type.0);
            let indices = old_record_indices.get_or_insert_with(|| {
                artifact
                    .display_items
                    .iter()
                    .enumerate()
                    .filter(|(_, record)| {
                        [
                            DisplayItemIdType::kScrollbarHorizontal,
                            DisplayItemIdType::kScrollbarVertical,
                            DisplayItemIdType::kScrollCorner,
                        ]
                        .contains(&record.id.r#type)
                    })
                    .map(|(i, record)| {
                        (
                            (record.id.client_id, record.id.fragment, record.id.r#type.0),
                            i,
                        )
                    })
                    .collect()
            });
            let Some(&index) = indices.get(&key) else {
                // A previously culled offscreen scrollbar has no old record.
                // It may remain unrecorded only while it cannot be displayed.
                let original_root = MapRectToRoot(record.visual_rect, &geometry.transforms)
                    .ok_or(UnsupportedTransform)?;
                let state = refreshed_cached(&old.properties, &mut trees, &mut refreshed_states)
                    .ok_or(PropertyChanged)?;
                let current_world =
                    property_world(state, &mut worlds).ok_or(UnsupportedTransform)?;
                let (rect, _) = rebase_rect(original_root, &old.recorded_world, &current_world)
                    .ok_or(UnsupportedTransform)?;
                let output = visible_output(
                    state,
                    &masks,
                    &mut mask_cache,
                    updated_store.viewport_clip(),
                    &mut worlds,
                    &mut clip_cache,
                    0.0,
                )
                .ok_or(UnsupportedTransform)?;
                let visible = output.is_none_or(|clip| !intersection(rect, clip).is_empty());
                if visible {
                    return Err(ScrollbarChanged);
                }
                continue;
            };
            let before = &artifact.display_items[index];
            if record.record_end - record.record_begin != before.record_end - before.record_begin {
                return Err(ScrollbarChanged);
            }
            let mut ops = controls.items[record.record_begin..record.record_end].to_vec();
            for (op, before) in ops
                .iter_mut()
                .zip(&artifact.items[before.record_begin..before.record_end])
            {
                if op.r#type != before.r#type {
                    return Err(ScrollbarChanged);
                }
                op.phase = before.phase;
            }
            replacements.push((index, ops));
        }
    }
    drop(scrollbar_trace);
    let publish_trace = browser_tracing::span("paint", "ScrollPaint.PublishProperties");
    let mut result = ScrollPropertyUpdate::default();
    for (chunk, properties) in artifact.chunks.iter_mut().zip(properties) {
        if !std::sync::Arc::ptr_eq(&chunk.properties.transform, &properties.transform)
            || !std::sync::Arc::ptr_eq(&chunk.properties.clip, &properties.clip)
            || !std::sync::Arc::ptr_eq(&chunk.properties.effect, &properties.effect)
        {
            result.updated_chunks += 1;
        }
        chunk.properties = properties;
        chunk.is_moved_from_cached_subsequence = true;
    }
    for record in &mut artifact.display_items {
        if let Some(old) = record.scroll_translation.as_ref() {
            record.scroll_translation = trees.transform_for(old).cloned();
        }
    }
    for (index, ops) in replacements {
        let record = &artifact.display_items[index];
        if &artifact.items[record.record_begin..record.record_end] != ops.as_slice() {
            let old_ops = &mut std::sync::Arc::make_mut(&mut artifact.items)
                [record.record_begin..record.record_end];
            old_ops.clone_from_slice(&ops);
            result.updated_scrollbar_records += 1;
        }
    }
    if result.updated_scrollbar_records != 0 {
        artifact.recording_revision = foundation::NewUniqueObjectId();
    }
    if let Some(viewport) = artifact.compositor_viewport.as_mut() {
        viewport.viewport_clip = updated_store
            .viewport_clip()
            .expect("source retained viewport clip")
            .clone();
        if let Some(old) = &viewport.layout_scroll_translation {
            viewport.layout_scroll_translation = Some(
                trees
                    .transform_for(old)
                    .expect("source retained layout scroll slot")
                    .clone(),
            );
        }
        viewport.scroll_translation_nodes = updated_store
            .compositor_scroll_translations()
            .cloned()
            .collect();
    }
    prepaint.property_node_store = updated_store;
    drop(publish_trace);
    if let Some(started) = started {
        eprintln!("scroll-prepaint-direct ms={:.3} checked_fragments={} rebuilt_prepaint_nodes=0 updated_chunks={} updated_scrollbar_records={}",
            started.elapsed().as_secs_f64() * 1000.0, retained.fragments.len(),
            result.updated_chunks, result.updated_scrollbar_records);
    }
    Ok(result)
}

#[cfg(test)]
mod coverage_tests {
    use super::*;

    #[test]
    fn retained_fragment_slots_preserve_identity_and_geometry_guards() {
        fn fragment(id: u64) -> FragmentNode {
            let mut f = FragmentNode::default();
            f.node_id = id;
            f.fragment_instance_id = id + 10;
            f.paint.display_item_client_id = id + 20;
            f
        }
        fn accepts(retained: &RetainedScrollPaintState, f: &FragmentNode) -> bool {
            let mut nodes = vec![None; retained.fragments.len()];
            let mut offsets = BTreeMap::new();
            let mut collected = 0;
            collect_current_fragments(
                f,
                retained,
                &mut nodes,
                &mut offsets,
                &mut BTreeMap::new(),
                &mut collected,
            ) && collected == retained.fragments.len()
        }
        let mut root = fragment(1);
        root.children = vec![fragment(2), fragment(3)];
        // Recorded paint order may differ from native order (logical OOF parent).
        let order = [&root.children[1], &root, &root.children[0]];
        let fragments: Vec<_> = order
            .iter()
            .map(|f| FragmentCoverage {
                offset: f.offset,
                size: f.size,
                sticky_offset: f.paint.sticky_offset,
                paint_offset: Offset::default(),
                recorded_world: TransformMatrix::default(),
                properties: PropertyTreeState::default(),
                controls: None,
            })
            .collect();
        let fragment_indices = order
            .iter()
            .enumerate()
            .map(|(i, f)| {
                (
                    (
                        f.node_id,
                        f.fragment_instance_id,
                        f.paint.display_item_client_id,
                        f.paint.display_item_fragment,
                    ),
                    i,
                )
            })
            .collect();
        let retained = RetainedScrollPaintState {
            fragments,
            fragment_indices,
            masks: Vec::new(),
            coverage_groups: Vec::new(),
            has_scroll_dependent_background: false,
        };
        assert!(accepts(&retained, &root));
        let mut with_resource = root.clone();
        let mut hidden = fragment(4);
        hidden.paint.svg_hidden_container = true;
        hidden.children.push(fragment(5));
        with_resource.children.push(hidden);
        assert!(accepts(&retained, &with_resource));
        with_resource
            .children
            .last_mut()
            .unwrap()
            .paint
            .svg_hidden_container = false;
        assert!(!accepts(&retained, &with_resource));
        let mut current = root.clone();
        current.children.reverse();
        current.paint.scroll_offset.y = 64.5;
        assert!(accepts(&retained, &current));
        current.children.pop();
        assert!(!accepts(&retained, &current));
        let mut current = root.clone();
        current.children.push(root.children[0].clone());
        assert!(!accepts(&retained, &current));
        let mut current = root.clone();
        current.children[0].fragment_instance_id += 1;
        assert!(!accepts(&retained, &current));
        let mut current = root.clone();
        current.children[0].size.width += 1.0;
        assert!(!accepts(&retained, &current));
        let mut current = root.clone();
        current.children[0].paint.sticky_offset.y += 1.0;
        assert!(!accepts(&retained, &current));
        current.children[0].paint.sticky_constraint = Some(Default::default());
        assert!(accepts(&retained, &current)); // Store subsequently validates the real slot/geometry.
    }

    #[test]
    fn retained_affine_coverage_uses_translation_without_inverse_aabb() {
        let mut recorded = TransformMatrix::default();
        recorded.values[0] = 0.8;
        recorded.values[1] = 0.6;
        recorded.values[4] = -0.6;
        recorded.values[5] = 0.8;
        recorded.values[12] = 900.0;
        recorded.values[13] = 2000.0;
        let mut scrolled = recorded;
        scrolled.values[13] -= 64.5;
        let cull = PaintRect {
            x: 50.0,
            y: 100.0,
            width: 300.0,
            height: 400.0,
        };
        let (current, _) = rebase_rect(cull, &recorded, &scrolled).unwrap();
        assert_eq!(current, PaintRect { y: 35.5, ..cull });
        let (back, error) = rebase_rect(current, &scrolled, &recorded).unwrap();
        assert_eq!(back, cull);
        assert!(contains(cull, back, error));
        let too_far = PaintRect {
            y: current.y - 1.0,
            ..current
        };
        let (required, error) = rebase_rect(too_far, &scrolled, &recorded).unwrap();
        assert!(!contains(cull, required, error));
        scrolled.values[0] += 0.01;
        assert!(rebase_rect(current, &scrolled, &recorded).is_none());
    }
}
