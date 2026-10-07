//! Blink-style paint property references for the supported standalone input.
//!
//! Sources: platform/graphics/paint/property_tree_state.h and the transform,
//! clip, effect and scroll_paint_property_node.h files. These are paint trees,
//! not compositor layer trees. Numeric IDs are dump-local labels; reference
//! equality, as in Blink, determines whether states share property nodes.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, OnceLock};

use crate::PaintRect;
use layoutng_assembly::internal::layout_input::{Offset, PaintPathCommand, TransformMatrix};
use layoutng_assembly::internal::paint_input::{
    PaintBlendMode, PaintCornerRadii, PaintFilterOperation,
};

/// Source: paint_property_node.h. Composited-value downgrades are deliberately
/// absent from the standalone builder: it does not directly update cc nodes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum PaintPropertyChangeType {
    #[default]
    Unchanged,
    ChangedOnlyCompositedValues,
    ChangedOnlySimpleValues,
    ChangedOnlyValues,
    NodeAddedOrRemoved,
}

/// A separately owned, persistent node entity. Blink mutates its GC node in
/// place; Rust retains immutable frame states referring to this same entity.
/// Its allocation, rather than a client hash or dump ID, defines identity.
#[derive(Debug)]
pub struct PaintPropertyNodeIdentity {
    _private: (),
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
            identity: Arc::new(PaintPropertyNodeIdentity { _private: () }),
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
    fn root() -> Self {
        Self {
            changed: PaintPropertyChangeType::Unchanged,
            ..Self::default()
        }
    }
    fn updated(&self, changed: PaintPropertyChangeType) -> Self {
        Self {
            identity: self.identity.clone(),
            revision: self.revision + u64::from(changed != PaintPropertyChangeType::Unchanged),
            changed,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TransformPaintPropertyNode {
    pub lifecycle: PaintPropertyNodeLifecycle,
    pub id: u64,
    pub parent: Option<Arc<Self>>,
    pub matrix: TransformMatrix,
    // The standalone geometry mapper has already folded transform-origin into
    // the matrix. Keeping origin zero preserves that resolved transformation.
    pub origin: [f64; 3],
    pub scroll: Option<Arc<ScrollPaintPropertyNode>>,
    pub direct_compositing_reasons: Vec<&'static str>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClipPaintPropertyNode {
    pub lifecycle: PaintPropertyNodeLifecycle,
    pub id: u64,
    pub parent: Option<Arc<Self>>,
    pub local_transform_space: Arc<TransformPaintPropertyNode>,
    pub rect: Option<PaintRect>,
    pub radii: PaintCornerRadii,
    pub clip_path: Vec<PaintPathCommand>,
    pub clip_path_even_odd: bool,
    /// UpdatePixelMovingFilterClipExpander retains the filter whose input
    /// bounds must be expanded; an infinite rect alone loses this relationship.
    pub pixel_moving_filter: Option<Arc<EffectPaintPropertyNode>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EffectPaintPropertyNode {
    pub lifecycle: PaintPropertyNodeLifecycle,
    pub id: u64,
    pub parent: Option<Arc<Self>>,
    pub local_transform_space: Arc<TransformPaintPropertyNode>,
    pub output_clip: Option<Arc<ClipPaintPropertyNode>>,
    pub opacity: f32,
    pub blend_mode: PaintBlendMode,
    pub filters: Vec<PaintFilterOperation>,
    pub isolates_blending: bool,
    // Legacy standalone wrapper metadata; formal Mask nodes use is_mask.
    pub has_mask: bool,
    /// ObjectPaintProperties::Mask has internal SkBlendMode::kDstIn. This is
    /// independent of the CSS blend-mode input, which cannot express DstIn.
    pub is_mask: bool,
    pub direct_compositing_reasons: Vec<&'static str>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScrollPaintPropertyNode {
    pub lifecycle: PaintPropertyNodeLifecycle,
    pub id: u64,
    pub parent: Option<Arc<Self>>,
    pub overflow_clip: Option<Arc<ClipPaintPropertyNode>>,
    pub container_rect: PaintRect,
    pub contents_rect: PaintRect,
    pub user_scrollable_horizontal: bool,
    pub user_scrollable_vertical: bool,
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
                lifecycle: PaintPropertyNodeLifecycle::root(),
                id: 0,
                parent: None,
                overflow_clip: None,
                container_rect: infinite,
                contents_rect: infinite,
                user_scrollable_horizontal: false,
                user_scrollable_vertical: false,
            });
            let transform = Arc::new(TransformPaintPropertyNode {
                lifecycle: PaintPropertyNodeLifecycle::root(),
                id: 0,
                parent: None,
                matrix: TransformMatrix::default(),
                origin: [0.0; 3],
                scroll: Some(scroll),
                direct_compositing_reasons: Vec::new(),
            });
            let clip = Arc::new(ClipPaintPropertyNode {
                lifecycle: PaintPropertyNodeLifecycle::root(),
                id: 0,
                parent: None,
                local_transform_space: transform.clone(),
                rect: None,
                radii: PaintCornerRadii::default(),
                clip_path: Vec::new(),
                clip_path_even_odd: false,
                pixel_moving_filter: None,
            });
            let effect = Arc::new(EffectPaintPropertyNode {
                lifecycle: PaintPropertyNodeLifecycle::root(),
                id: 0,
                parent: None,
                local_transform_space: transform.clone(),
                output_clip: None,
                opacity: 1.0,
                blend_mode: PaintBlendMode::kNormal,
                filters: Vec::new(),
                isolates_blending: false,
                has_mask: false,
                is_mask: false,
                direct_compositing_reasons: Vec::new(),
            });
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
    pub fn same_nodes(&self, other: &Self) -> bool {
        self == other
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
            .map(|n| n.lifecycle.changed)
            .chain(trees.clips.values().map(|n| n.lifecycle.changed))
            .chain(trees.effects.values().map(|n| n.lifecycle.changed))
            .chain(trees.scrolls.values().map(|n| n.lifecycle.changed))
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
}

impl PaintPropertyTrees {
    /// Clear every referenced ancestor, including fresh nodes without a
    /// native-owner slot. Blink's PropertyTreeState::ClearChangedToRoot walks
    /// the actual state graph; the owner directory is not the complete graph.
    pub(crate) fn clear_state_change_markers(&mut self, state: &mut PropertyTreeState) {
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
        if self.transforms.contains_key(&node.id) {
            return;
        }
        self.transforms.insert(node.id, node.clone());
        if let Some(parent) = &node.parent {
            self.add_transform(parent);
        }
        if let Some(scroll) = &node.scroll {
            self.add_scroll(scroll);
        }
    }
    fn add_clip(&mut self, node: &Arc<ClipPaintPropertyNode>) {
        if self.clips.contains_key(&node.id) {
            return;
        }
        self.clips.insert(node.id, node.clone());
        if let Some(parent) = &node.parent {
            self.add_clip(parent);
        }
        if let Some(filter) = &node.pixel_moving_filter {
            self.add_effect(filter);
        }
        self.add_transform(&node.local_transform_space);
    }
    fn add_effect(&mut self, node: &Arc<EffectPaintPropertyNode>) {
        if self.effects.contains_key(&node.id) {
            return;
        }
        self.effects.insert(node.id, node.clone());
        if let Some(parent) = &node.parent {
            self.add_effect(parent);
        }
        self.add_transform(&node.local_transform_space);
        if let Some(clip) = &node.output_clip {
            self.add_clip(clip);
        }
    }
    fn add_scroll(&mut self, node: &Arc<ScrollPaintPropertyNode>) {
        if self.scrolls.contains_key(&node.id) {
            return;
        }
        self.scrolls.insert(node.id, node.clone());
        if let Some(parent) = &node.parent {
            self.add_scroll(parent);
        }
        if let Some(clip) = &node.overflow_clip {
            self.add_clip(clip);
        }
    }
}

/// ObjectPaintProperties owns one slot per property kind and fragment. The
/// source-specific transform slots mirror the supported ObjectPaintProperties
/// roles. Independent CSS translate/rotate/scale/offset payload is unavailable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum PaintPropertyRole {
    ViewportClip,
    PaintOffsetTranslation,
    StickyTranslation,
    Transform,
    SvgLocalTransform,
    ClipPathClip,
    MaskClip,
    Effect,
    Mask,
    Filter,
    PixelMovingFilterClipExpander,
    OverflowClip,
    Scroll,
    ScrollTranslation,
    ReplacedContentTransform,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum PaintPropertyOwner {
    Viewport,
    NativeFragment { client: u64, fragment: u32 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct PaintPropertyKey {
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
    /// The real recording coordinate anchors, indexed by the same native
    /// owner/role slots used by the property builder. No new layer identity.
    pub(crate) fn scroll_offsets(&self) -> BTreeMap<PaintPropertyOwner, Offset> {
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

    /// Retained native geometry has already proved unchanged owners/topology,
    /// styles, resources and fragment geometry. Update existing real scroll
    /// translations without constructing a new PrePaint tree. The caller must
    /// prove retained recording coverage before publishing this staged store.
    ///
    /// Source: paint_property_tree_builder.cc:3769-3827 and
    /// property_tree_manager.cc:138-176. This backend retains immutable Blink
    /// snapshots; it does not claim a cc composited-value update. Rebinding a
    /// snapshot reference preserves its entity and revision. Per-pass change
    /// markers clear on unchanged values, as in the ordinary property updater.
    pub(crate) fn retained_scroll_update(
        &self,
        offsets: &BTreeMap<PaintPropertyOwner, Offset>,
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
            let offset = offsets.get(&key.owner)?;
            // Preserve the previous scroll-only admission: translations without
            // real ScrollNodes (e.g. hidden overflow) require ordinary PrePaint.
            if !matches!(key.owner, PaintPropertyOwner::NativeFragment { .. })
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
            translations.insert(node.id, matrix);
        }
        Some(self.staged_snapshots(translations))
    }

    /// Clear the preceding paint pass's change markers in new immutable
    /// snapshots. As with Blink's ClearChangedToRoot, this changes no property
    /// values, node entities or revisions; old paint artifacts retain their
    /// original markers. The caller rebinds published states using the trees.
    pub(crate) fn cleared_change_markers(&self) -> Option<(Self, PaintPropertyTrees)> {
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
        translations: BTreeMap<u64, TransformMatrix>,
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
            transforms,
            clips,
            effects,
            scrolls,
            change_markers_cleared: rebind.changed == PaintPropertyChangeType::Unchanged,
            changed: rebind.changed,
        };
        (store, rebind.trees)
    }

    pub(crate) fn viewport_clip(&self) -> Option<&Arc<ClipPaintPropertyNode>> {
        self.clips.get(&PaintPropertyKey {
            owner: PaintPropertyOwner::Viewport,
            role: PaintPropertyRole::ViewportClip,
        })
    }

    pub(crate) fn is_scroll_only_update_from(&self, old: &Self) -> bool {
        let reject = |kind: &str,
                      key: &PaintPropertyKey,
                      before: u64,
                      after: u64,
                      same_identity: bool,
                      change: PaintPropertyChangeType| {
            if std::env::var_os("BROWSER_PROFILE_INPUT").is_some() {
                eprintln!("scroll-property-reject kind={} key={:?} old_id={} new_id={} same_identity={} change={:?}",
                    kind, key, before, after, same_identity, change);
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
    pub(crate) fn clear_owner(&mut self, owner: PaintPropertyOwner) {
        self.transforms.retain(|k, _| k.owner != owner);
        self.clips.retain(|k, _| k.owner != owner);
        self.effects.retain(|k, _| k.owner != owner);
        self.scrolls.retain(|k, _| k.owner != owner);
    }
    pub(crate) fn updater<'a>(
        &'a mut self,
        keys: &'a BTreeMap<u64, PaintPropertyKey>,
        effect_sources: &'a BTreeMap<u64, Arc<EffectPaintPropertyNode>>,
    ) -> PropertyNodeUpdater<'a> {
        self.changed = PaintPropertyChangeType::Unchanged;
        // A partial update cannot certify that every retained graph is clean.
        self.change_markers_cleared = false;
        PropertyNodeUpdater {
            store: self,
            keys,
            effect_sources,
            seen: BTreeSet::new(),
            transforms: BTreeMap::new(),
            clips: BTreeMap::new(),
            effects: BTreeMap::new(),
            scrolls: BTreeMap::new(),
        }
    }
    pub(crate) fn next_id(&self) -> u64 {
        self.next_id
    }
}

// One pass over the resident property graph, not the fragment tree. Memoized
// per-type IDs name existing directory entries; no new node identity is made.
struct RetainedScrollSnapshotRebinder {
    translations: BTreeMap<u64, TransformMatrix>,
    trees: PaintPropertyTrees,
    changed: PaintPropertyChangeType,
}
impl RetainedScrollSnapshotRebinder {
    fn transform(
        &mut self,
        input: &Arc<TransformPaintPropertyNode>,
    ) -> Arc<TransformPaintPropertyNode> {
        if let Some(node) = self.trees.transforms.get(&input.id) {
            return node.clone();
        }
        let parent = input.parent.as_ref().map(|parent| self.transform(parent));
        let scroll = input.scroll.as_ref().map(|scroll| self.scroll(scroll));
        let matrix = self
            .translations
            .get(&input.id)
            .copied()
            .unwrap_or(input.matrix);
        let node = if matrix == input.matrix
            && same_snapshot_option(&parent, &input.parent)
            && same_snapshot_option(&scroll, &input.scroll)
            && input.lifecycle.changed == PaintPropertyChangeType::Unchanged
        {
            input.clone()
        } else {
            let change = if matrix != input.matrix {
                PaintPropertyChangeType::ChangedOnlySimpleValues
            } else {
                PaintPropertyChangeType::Unchanged
            };
            self.changed = self.changed.max(change);
            // Clone only retained payloads. The old parent/cross-tree/lifecycle
            // references would otherwise be cloned and immediately overwritten.
            Arc::new(TransformPaintPropertyNode {
                lifecycle: input.lifecycle.updated(change),
                id: input.id,
                parent,
                matrix,
                origin: input.origin,
                scroll,
                direct_compositing_reasons: input.direct_compositing_reasons.clone(),
            })
        };
        self.trees.transforms.insert(input.id, node.clone());
        node
    }
    fn clip(&mut self, input: &Arc<ClipPaintPropertyNode>) -> Arc<ClipPaintPropertyNode> {
        if let Some(node) = self.trees.clips.get(&input.id) {
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
            && input.lifecycle.changed == PaintPropertyChangeType::Unchanged
        {
            input.clone()
        } else {
            Arc::new(ClipPaintPropertyNode {
                lifecycle: input.lifecycle.updated(PaintPropertyChangeType::Unchanged),
                id: input.id,
                parent,
                local_transform_space: transform,
                rect: input.rect,
                radii: input.radii,
                clip_path: input.clip_path.clone(),
                clip_path_even_odd: input.clip_path_even_odd,
                pixel_moving_filter: filter,
            })
        };
        self.trees.clips.insert(input.id, node.clone());
        node
    }
    fn effect(&mut self, input: &Arc<EffectPaintPropertyNode>) -> Arc<EffectPaintPropertyNode> {
        if let Some(node) = self.trees.effects.get(&input.id) {
            return node.clone();
        }
        let parent = input.parent.as_ref().map(|parent| self.effect(parent));
        let transform = self.transform(&input.local_transform_space);
        let clip = input.output_clip.as_ref().map(|clip| self.clip(clip));
        let node = if same_snapshot_option(&parent, &input.parent)
            && Arc::ptr_eq(&transform, &input.local_transform_space)
            && same_snapshot_option(&clip, &input.output_clip)
            && input.lifecycle.changed == PaintPropertyChangeType::Unchanged
        {
            input.clone()
        } else {
            Arc::new(EffectPaintPropertyNode {
                lifecycle: input.lifecycle.updated(PaintPropertyChangeType::Unchanged),
                id: input.id,
                parent,
                local_transform_space: transform,
                output_clip: clip,
                opacity: input.opacity,
                blend_mode: input.blend_mode,
                filters: input.filters.clone(),
                isolates_blending: input.isolates_blending,
                has_mask: input.has_mask,
                is_mask: input.is_mask,
                direct_compositing_reasons: input.direct_compositing_reasons.clone(),
            })
        };
        self.trees.effects.insert(input.id, node.clone());
        node
    }
    fn scroll(&mut self, input: &Arc<ScrollPaintPropertyNode>) -> Arc<ScrollPaintPropertyNode> {
        if let Some(node) = self.trees.scrolls.get(&input.id) {
            return node.clone();
        }
        let parent = input.parent.as_ref().map(|parent| self.scroll(parent));
        let clip = input.overflow_clip.as_ref().map(|clip| self.clip(clip));
        let node = if same_snapshot_option(&parent, &input.parent)
            && same_snapshot_option(&clip, &input.overflow_clip)
            && input.lifecycle.changed == PaintPropertyChangeType::Unchanged
        {
            input.clone()
        } else {
            Arc::new(ScrollPaintPropertyNode {
                lifecycle: input.lifecycle.updated(PaintPropertyChangeType::Unchanged),
                id: input.id,
                parent,
                overflow_clip: clip,
                container_rect: input.container_rect,
                contents_rect: input.contents_rect,
                user_scrollable_horizontal: input.user_scrollable_horizontal,
                user_scrollable_vertical: input.user_scrollable_vertical,
            })
        };
        self.trees.scrolls.insert(input.id, node.clone());
        node
    }
}

pub(crate) struct PropertyNodeUpdater<'a> {
    store: &'a mut PaintPropertyNodeStore,
    keys: &'a BTreeMap<u64, PaintPropertyKey>,
    effect_sources: &'a BTreeMap<u64, Arc<EffectPaintPropertyNode>>,
    seen: BTreeSet<PaintPropertyKey>,
    transforms: BTreeMap<u64, Arc<TransformPaintPropertyNode>>,
    clips: BTreeMap<u64, Arc<ClipPaintPropertyNode>>,
    effects: BTreeMap<u64, Arc<EffectPaintPropertyNode>>,
    scrolls: BTreeMap<u64, Arc<ScrollPaintPropertyNode>>,
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
    }
}
impl ScrollPaintPropertyNode {
    fn same_snapshot_references(&self, other: &Self) -> bool {
        same_snapshot_option(&self.parent, &other.parent)
            && same_snapshot_option(&self.overflow_clip, &other.overflow_clip)
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
fn transform_state_change(
    a: &TransformPaintPropertyNode,
    b: &TransformPaintPropertyNode,
) -> PaintPropertyChangeType {
    use PaintPropertyChangeType::*;
    if !same_optional(&a.parent, &b.parent, |n| &n.lifecycle)
        || !same_optional(&a.scroll, &b.scroll, |n| &n.lifecycle)
        || a.direct_compositing_reasons != b.direct_compositing_reasons
    {
        return ChangedOnlyValues;
    }
    if a.matrix == b.matrix && a.origin == b.origin {
        return Unchanged;
    }
    // Animation/sticky direct compositor updates are unavailable. Only the
    // unambiguous 2D translation case uses the official simple-value class;
    // other matrix updates conservatively request a full property update.
    let is_translation = |m: &TransformMatrix| {
        let mut v = m.values;
        v[12] = 0.0;
        v[13] = 0.0;
        v == TransformMatrix::default().values
    };
    if is_translation(&a.matrix) && is_translation(&b.matrix) {
        ChangedOnlySimpleValues
    } else {
        ChangedOnlyValues
    }
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
        && a.clip_path == b.clip_path
        && a.clip_path_even_odd == b.clip_path_even_odd
        && same_optional(&a.pixel_moving_filter, &b.pixel_moving_filter, |n| {
            &n.lifecycle
        })
    {
        PaintPropertyChangeType::Unchanged
    } else {
        PaintPropertyChangeType::ChangedOnlyValues
    }
}
fn effect_state_change(
    a: &EffectPaintPropertyNode,
    b: &EffectPaintPropertyNode,
) -> PaintPropertyChangeType {
    use PaintPropertyChangeType::*;
    if !same_optional(&a.parent, &b.parent, |n| &n.lifecycle)
        || !a
            .local_transform_space
            .lifecycle
            .same_node(&b.local_transform_space.lifecycle)
        || !same_optional(&a.output_clip, &b.output_clip, |n| &n.lifecycle)
        || a.blend_mode != b.blend_mode
        || a.filters != b.filters
        || a.isolates_blending != b.isolates_blending
        || a.has_mask != b.has_mask
        || a.is_mask != b.is_mask
        || a.direct_compositing_reasons != b.direct_compositing_reasons
    {
        return ChangedOnlyValues;
    }
    if a.opacity == b.opacity {
        return Unchanged;
    }
    // EffectPaintPropertyNode::State::IsOpacityChangeSimple. No fabricated
    // running compositor animation is inferred from the animation reason.
    if (a.opacity != 1.0 && b.opacity != 1.0)
        || (a
            .direct_compositing_reasons
            .contains(&"ActiveOpacityAnimation")
            && b.direct_compositing_reasons
                .contains(&"ActiveOpacityAnimation"))
    {
        ChangedOnlySimpleValues
    } else {
        ChangedOnlyValues
    }
}
fn scroll_state_change(
    a: &ScrollPaintPropertyNode,
    b: &ScrollPaintPropertyNode,
) -> PaintPropertyChangeType {
    if same_optional(&a.parent, &b.parent, |n| &n.lifecycle)
        && same_optional(&a.overflow_clip, &b.overflow_clip, |n| &n.lifecycle)
        && a.container_rect == b.container_rect
        && a.contents_rect == b.contents_rect
        && a.user_scrollable_horizontal == b.user_scrollable_horizontal
        && a.user_scrollable_vertical == b.user_scrollable_vertical
    {
        PaintPropertyChangeType::Unchanged
    } else {
        PaintPropertyChangeType::ChangedOnlyValues
    }
}

macro_rules! update_snapshot {
    ($self:ident, $node:ident, $old:ident, $change:ident, $map:ident) => {{
        let raw_id = $node.id;
        if let Some(key) = $self.keys.get(&raw_id).copied() {
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
                    && old.lifecycle.changed == PaintPropertyChangeType::Unchanged
                    && $node.same_snapshot_references(old)
            }) {
                old.clone()
            } else {
                Arc::new($node)
            };
            $self.store.$map.insert(key, node.clone());
            node
        } else {
            // No native owner (unsupported hand-built input): keep fresh
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
    pub(crate) fn state(&mut self, state: &mut PropertyTreeState) {
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
        if let Some(node) = self.transforms.get(&input.id) {
            return node.clone();
        }
        let mut node = (**input).clone();
        node.parent = node.parent.as_ref().map(|p| self.transform(p));
        node.scroll = node.scroll.as_ref().map(|s| self.scroll(s));
        let node = update_snapshot!(self, node, old, transform_state_change, transforms);
        self.transforms.insert(input.id, node.clone());
        node
    }
    fn clip(&mut self, input: &Arc<ClipPaintPropertyNode>) -> Arc<ClipPaintPropertyNode> {
        if input.id == 0 {
            return input.clone();
        }
        if let Some(node) = self.clips.get(&input.id) {
            return node.clone();
        }
        let mut node = (**input).clone();
        node.parent = node.parent.as_ref().map(|p| self.clip(p));
        node.local_transform_space = self.transform(&node.local_transform_space);
        node.pixel_moving_filter = node.pixel_moving_filter.as_ref().map(|f| self.effect(f));
        let node = update_snapshot!(self, node, old, clip_state_change, clips);
        self.clips.insert(input.id, node.clone());
        node
    }
    fn effect(&mut self, input: &Arc<EffectPaintPropertyNode>) -> Arc<EffectPaintPropertyNode> {
        if input.id == 0 {
            return input.clone();
        }
        if let Some(node) = self.effects.get(&input.id) {
            return node.clone();
        }
        // A physical clip expander may still hold a pre-rebind effect snapshot.
        // Resolve its real node entity to the final logical effect ancestry.
        let source = self.effect_sources.get(&input.id).unwrap_or(input);
        let mut node = (**source).clone();
        node.parent = node.parent.as_ref().map(|p| self.effect(p));
        node.local_transform_space = self.transform(&node.local_transform_space);
        node.output_clip = node.output_clip.as_ref().map(|c| self.clip(c));
        let node = update_snapshot!(self, node, old, effect_state_change, effects);
        self.effects.insert(input.id, node.clone());
        node
    }
    fn scroll(&mut self, input: &Arc<ScrollPaintPropertyNode>) -> Arc<ScrollPaintPropertyNode> {
        if input.id == 0 {
            return input.clone();
        }
        if let Some(node) = self.scrolls.get(&input.id) {
            return node.clone();
        }
        let mut node = (**input).clone();
        node.parent = node.parent.as_ref().map(|p| self.scroll(p));
        node.overflow_clip = node.overflow_clip.as_ref().map(|c| self.clip(c));
        let node = update_snapshot!(self, node, old, scroll_state_change, scrolls);
        self.scrolls.insert(input.id, node.clone());
        node
    }
    pub(crate) fn finish(self) {
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

    #[test]
    fn clearing_resident_states_includes_nodes_without_native_owner_slots() {
        let root = PropertyTreeState::default();
        let clip = Arc::new(ClipPaintPropertyNode {
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
            clip_path: Vec::new(),
            clip_path_even_odd: false,
            pixel_moving_filter: None,
        });
        let mut state = PropertyTreeState { clip, ..root };
        let mut store = PaintPropertyNodeStore::default();
        let keys = BTreeMap::new();
        let effects = BTreeMap::new();
        let mut updater = store.updater(&keys, &effects);
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
        let owner = PaintPropertyOwner::NativeFragment {
            client: 42,
            fragment: 0,
        };
        let key = |role| PaintPropertyKey { owner, role };
        let clean =
            || PaintPropertyNodeLifecycle::default().updated(PaintPropertyChangeType::Unchanged);
        let scroll = Arc::new(ScrollPaintPropertyNode {
            id: 1,
            lifecycle: clean(),
            parent: root.transform.scroll.clone(),
            overflow_clip: Some(root.clip.clone()),
            container_rect: PaintRect::default(),
            contents_rect: PaintRect::default(),
            user_scrollable_horizontal: false,
            user_scrollable_vertical: true,
        });
        let translation = Arc::new(TransformPaintPropertyNode {
            id: 2,
            lifecycle: clean(),
            parent: Some(root.transform.clone()),
            matrix: TransformMatrix::default(),
            origin: [0.0; 3],
            scroll: Some(scroll.clone()),
            direct_compositing_reasons: Vec::new(),
        });
        let child = Arc::new(TransformPaintPropertyNode {
            id: 3,
            lifecycle: clean(),
            parent: Some(translation.clone()),
            matrix: TransformMatrix::default(),
            origin: [0.0; 3],
            scroll: None,
            direct_compositing_reasons: Vec::new(),
        });
        let clip = Arc::new(ClipPaintPropertyNode {
            id: 4,
            lifecycle: clean().updated(PaintPropertyChangeType::ChangedOnlyValues),
            parent: Some(root.clip.clone()),
            local_transform_space: child.clone(),
            rect: None,
            radii: PaintCornerRadii::default(),
            clip_path: Vec::new(),
            clip_path_even_odd: false,
            pixel_moving_filter: None,
        });
        let effect = Arc::new(EffectPaintPropertyNode {
            id: 5,
            lifecycle: clean(),
            parent: Some(root.effect.clone()),
            local_transform_space: child.clone(),
            output_clip: Some(clip.clone()),
            opacity: 0.5,
            blend_mode: PaintBlendMode::kNormal,
            filters: Vec::new(),
            isolates_blending: false,
            has_mask: false,
            is_mask: false,
            direct_compositing_reasons: Vec::new(),
        });
        let secondary_scroll = Arc::new(ScrollPaintPropertyNode {
            id: 6,
            lifecycle: clean(),
            parent: Some(scroll.clone()),
            overflow_clip: Some(clip.clone()),
            container_rect: PaintRect::default(),
            contents_rect: PaintRect::default(),
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
                owner: PaintPropertyOwner::NativeFragment {
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
        let (updated, trees) = store.retained_scroll_update(&offsets).unwrap();
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
        let (_, unchanged) = updated.retained_scroll_update(&offsets).unwrap();
        assert_eq!(
            unchanged.transforms[&2].lifecycle.revision,
            trees.transforms[&2].lifecycle.revision
        );
        assert_eq!(
            unchanged.transforms[&2].lifecycle.changed,
            PaintPropertyChangeType::Unchanged
        );
        assert!(store.retained_scroll_update(&BTreeMap::new()).is_none());
        let mut new_owner = offsets.clone();
        new_owner.insert(
            PaintPropertyOwner::NativeFragment {
                client: 99,
                fragment: 0,
            },
            Offset { x: 0.0, y: 1.0 },
        );
        assert!(store.retained_scroll_update(&new_owner).is_none());
    }
}
