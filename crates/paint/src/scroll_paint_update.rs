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
    fragment_tree::FragmentNode,
    internal::layout_input::{Offset, Size, TransformMatrix},
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
    } else if !clip.clip_path.is_empty() {
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
            // Pixel-moving effects need an effect-specific input cull proof.
            if !f.paint.style.filters.is_empty() {
                if std::env::var_os("BROWSER_PROFILE_INPUT").is_some() {
                    eprintln!(
                        "scroll-coverage-capture rejected=filter node={} filters={:?}",
                        f.node_id, f.paint.style.filters
                    );
                }
                return None;
            }
            let world = property_world(&node.properties.nodes, &mut worlds).or_else(|| {
                if std::env::var_os("BROWSER_PROFILE_INPUT").is_some() {
                    eprintln!("scroll-coverage-capture rejected=transform node={} source={} hidden={} transform={} world={:?}",
                        f.node_id, f.paint.has_source, f.paint.hidden, node.properties.nodes.transform.id,
                        world_transform(&node.properties.nodes.transform, &mut worlds).values);
                }
                None
            })?;
            let recorded_visible = visible_output(
                &node.properties.nodes,
                &masks,
                &mut mask_cache,
                store.viewport_clip(),
                &mut worlds,
                &mut clip_cache,
                0.0,
            )
            .or_else(|| {
                if std::env::var_os("BROWSER_PROFILE_INPUT").is_some() {
                    eprintln!(
                        "scroll-coverage-capture rejected=output node={} clip={} effect={}",
                        f.node_id, node.properties.nodes.clip.id, node.properties.nodes.effect.id
                    );
                }
                None
            })?;
            let identity = identity(node);
            if fragment_indices.insert(identity, fragments.len()).is_some() {
                return None;
            }
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
            fragments.push(FragmentCoverage {
                offset: f.offset,
                size: f.size,
                sticky_offset: f.paint.sticky_offset,
                paint_offset: node.paint_offset,
                recorded_world: world,
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
        })
    }
}

// Validate current exported inputs while placing them in the recorded slots.
// Unknown, duplicate or missing identities still reject the whole update.
fn collect_current_fragments<'a>(
    f: &'a FragmentNode,
    retained: &RetainedScrollPaintState,
    nodes: &mut [Option<&'a FragmentNode>],
    offsets: &mut BTreeMap<PaintPropertyOwner, Offset>,
    collected: &mut usize,
) -> bool {
    let key = (
        f.node_id,
        f.fragment_instance_id,
        f.paint.display_item_client_id,
        f.paint.display_item_fragment,
    );
    let Some(&index) = retained.fragment_indices.get(&key) else {
        return false;
    };
    if nodes[index].is_some() {
        return false;
    }
    let old = &retained.fragments[index];
    if f.offset != old.offset || f.size != old.size || f.paint.sticky_offset != old.sticky_offset {
        return false;
    }
    nodes[index] = Some(f);
    *collected += 1;
    if f.paint.display_item_client_id != 0 {
        let owner = PaintPropertyOwner::NativeFragment {
            client: f.paint.display_item_client_id,
            fragment: f.paint.display_item_fragment,
        };
        if offsets
            .insert(owner, f.paint.scroll_offset)
            .is_some_and(|old| old != f.paint.scroll_offset)
        {
            return false;
        }
    }
    f.children
        .iter()
        .all(|child| collect_current_fragments(child, retained, nodes, offsets, collected))
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
    if !trees.transforms.contains_key(&state.transform.id)
        || !trees.clips.contains_key(&state.clip.id)
        || !trees.effects.contains_key(&state.effect.id)
    {
        // Fresh property nodes without native-owner slots still belong to the
        // retained graph. Rebind their actual ancestors through the updated
        // scroll snapshots; do not mistake an incomplete directory for loss.
        let mut referenced = state.clone();
        trees.clear_state_change_markers(&mut referenced);
    }
    let transform = trees.transforms.get(&state.transform.id)?.clone();
    let clip = trees.clips.get(&state.clip.id)?.clone();
    let effect = trees.effects.get(&state.effect.id)?.clone();
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScrollRecordingUpdate {
    Retained,
    Fresh,
}

/// Retained records require the native unchanged-layout proof. Fresh records
/// were just painted from current certified inputs in an admitted coordinate
/// basis and recaptured their exact geometry/cull state. Both modes prove
/// current coverage and real property/scrollbar semantics before publication.
/// Rejection leaves the artifact untouched and requests ordinary paint.
pub(crate) fn TryUpdateScrollPropertiesWithPrePaint(
    fragments: &FragmentNode,
    artifact: &mut PaintArtifact,
    prepaint: &mut PrePaintTreeWalk,
    recording: ScrollRecordingUpdate,
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
    if let Some((index, chunk)) = artifact.chunks.iter().enumerate().find(|(_, chunk)| {
        recording == ScrollRecordingUpdate::Retained && !chunk.can_match_old_chunk()
    }) {
        if std::env::var_os("BROWSER_PROFILE_INPUT").is_some() {
            eprintln!("scroll-paint-reject stage=chunk-cache index={} id={:?} cacheable={} just_created={}",
                index, chunk.id, chunk.is_cacheable, chunk.client_is_just_created);
        }
        return Err(PropertyChanged);
    }
    // This path is admitted only by Page's retained native-layout proof. No
    // property builder or temporary PaintTreeNode is needed for clean content.
    // Verify current geometry independently; inspecting cached nodes here would
    // accidentally accept a changed sticky position.
    let mut nodes = vec![None; retained.fragments.len()];
    let mut offsets = BTreeMap::new();
    let mut collected = 0;
    let collect_trace = browser_tracing::span("paint", "ScrollPaint.CollectFragments");
    if !collect_current_fragments(
        fragments,
        retained,
        &mut nodes,
        &mut offsets,
        &mut collected,
    ) || collected != retained.fragments.len()
    {
        return Err(FragmentChanged);
    }
    drop(collect_trace);
    let property_trace = browser_tracing::span("paint", "ScrollPaint.RebindPropertyTrees");
    let (updated_store, mut trees) = prepaint
        .property_node_store
        .retained_scroll_update(&offsets)
        .ok_or(PropertyChanged)?;
    drop(property_trace);
    let device_scale = fragments
        .paint
        .resources
        .as_ref()
        .map_or(1.0, |resources| resources.device_pixel_ratio);
    if !device_scale.is_finite() || device_scale <= 0.0 {
        return Err(UnsupportedTransform);
    }
    let mut worlds = BTreeMap::new();
    let mut masks = BTreeMap::new();
    for mask in &retained.masks {
        let transform = trees
            .transforms
            .get(&mask.transform.id)
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
            (1.0 / device_scale).max(1.0),
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
        chunk.is_moved_from_cached_subsequence = recording == ScrollRecordingUpdate::Retained;
    }
    for record in &mut artifact.display_items {
        if let Some(old) = record.scroll_translation.as_ref() {
            record.scroll_translation = trees.transforms.get(&old.id).cloned();
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
            collect_current_fragments(f, retained, &mut nodes, &mut offsets, &mut collected)
                && collected == retained.fragments.len()
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
        let fragment_indices = fragments
            .iter()
            .enumerate()
            .map(|(i, f)| (f.identity, i))
            .collect();
        let retained = RetainedScrollPaintState {
            fragments,
            fragment_indices,
            masks: Vec::new(),
            coverage_groups: Vec::new(),
        };
        assert!(accepts(&retained, &root));
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
