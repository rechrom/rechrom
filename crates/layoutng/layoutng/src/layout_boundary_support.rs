#![allow(non_snake_case)]

use layout_fragment::fragment_tree::LayoutResultHandle;
use layout_fragment::layout_result::LayoutResult;
use layoutng_style::style::computed_style::ComputedStyle as StyleComputedStyle;
use std::sync::Arc;

use foundation::{TrustedIdentityHashMap, TrustedIdentityHashSet};
pub use layout_commit::LayoutUpdateKind;

use crate::internal::constraint_space::ConstraintSpace as LayoutConstraintSpace;
use crate::internal::layout_input::{ConstraintSpace, ExportedStyleCacheKey};
use crate::internal::paint_input::PaintStyleData;
use crate::internal::layout_node_metadata::Node;
use crate::internal::layout_object::LayoutObject;

pub(crate) type LayoutBoundaryHashMap<K, V> = TrustedIdentityHashMap<K, V>;
pub(crate) type LayoutBoundaryHashSet<K> = TrustedIdentityHashSet<K>;

/// Chromium's retained FragmentData/FragmentItem identity expressed without
/// borrowing a particular immutable LayoutResult generation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum FragmentIdentityKey {
    FragmentData(u64),
    InlineItem {
        container: u64,
        client: u64,
        fragment: u32,
        kind: u8,
        start: u32,
        end: u32,
        index: u32,
    },
    PhysicalFallback {
        client: u64,
        fragment: u32,
        kind: u8,
        preorder: u32,
    },
}

// cpp: layoutng/layout_boundary_support.h:15-23
pub trait LayoutBoundaryEnvironment {
    fn ReusesPreparedFonts(&self) -> bool;
    fn Commit(&mut self, root: &mut LayoutObject);
}

/// Immutable boundary inputs which remain valid while LayoutObject style
/// ownership is unchanged. Physical fragments are commonly rebuilt for a new
/// viewport constraint, but their source/source ComputedStyle pair is not.
#[derive(Clone, Copy)]
pub(crate) struct LayoutBoundarySourceEntry {
    pub(crate) node: *const Node,
    pub(crate) logical_tree_order: u64,
}

#[derive(Clone, Copy)]
pub(crate) struct LayoutBoundaryStyleSourceEntry {
    pub(crate) resources: *const crate::internal::layout_input::StyleResourceData,
}

/// The persistent PaintStyle projection cache of one Layout boundary, keyed
/// by (ComputedStyle, StyleResourceData) identity. Fragment PaintStyles
/// resolve their projection lazily (on first read, possibly during PrePaint
/// or Paint), so the memoized values live in a per-thread registry addressed
/// by this owner's id rather than in a map moved through publication.
/// `clear()` keeps the original clearing points: it advances the generation,
/// so entries keyed by addresses of replaced styles are never read again,
/// even by a pending projection resolved after the clear.
pub(crate) struct ExportedStyleCacheOwner {
    key: ExportedStyleCacheKey,
}

type ExportedStyleMap = LayoutBoundaryHashMap<(usize, usize), Arc<PaintStyleData>>;

thread_local! {
    static EXPORTED_STYLES: std::cell::RefCell<
        std::collections::HashMap<u64, (u64, ExportedStyleMap)>,
    > = std::cell::RefCell::new(std::collections::HashMap::new());
}

impl Default for ExportedStyleCacheOwner {
    fn default() -> Self {
        static NEXT_OWNER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        Self {
            key: ExportedStyleCacheKey {
                owner: NEXT_OWNER.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
                generation: 1,
            },
        }
    }
}

impl ExportedStyleCacheOwner {
    /// The current generation, registered on this (the publishing document)
    /// thread so projections resolved on it can memoize their values.
    pub(crate) fn Key(&self) -> ExportedStyleCacheKey {
        let key = self.key;
        let _ = EXPORTED_STYLES.try_with(|registry| {
            if let Ok(mut registry) = registry.try_borrow_mut() {
                let (generation, styles) = registry
                    .entry(key.owner)
                    .or_insert_with(|| (key.generation, ExportedStyleMap::default()));
                if *generation != key.generation {
                    *generation = key.generation;
                    styles.clear();
                }
            }
        });
        key
    }

    /// Memoized projections of the current generation (trace metric).
    pub(crate) fn len(&self) -> usize {
        let key = self.key;
        EXPORTED_STYLES
            .try_with(|registry| {
                let registry = registry.try_borrow().ok()?;
                let (generation, styles) = registry.get(&key.owner)?;
                (*generation == key.generation).then_some(styles.len())
            })
            .ok()
            .flatten()
            .unwrap_or(0)
    }

    pub(crate) fn clear(&mut self) {
        self.key.generation += 1;
        self.Unregister();
    }

    fn Unregister(&self) {
        let owner = self.key.owner;
        let _ = EXPORTED_STYLES.try_with(|registry| {
            if let Ok(mut registry) = registry.try_borrow_mut() {
                registry.remove(&owner);
            }
        });
    }
}

impl Drop for ExportedStyleCacheOwner {
    fn drop(&mut self) {
        self.Unregister();
    }
}

/// The memoized projection for `entry` in the cache generation `key`.
pub(crate) fn LookupExportedStyle(
    key: ExportedStyleCacheKey,
    entry: (usize, usize),
) -> Option<Arc<PaintStyleData>> {
    EXPORTED_STYLES
        .try_with(|registry| {
            let registry = registry.try_borrow().ok()?;
            let (generation, styles) = registry.get(&key.owner)?;
            (*generation == key.generation)
                .then(|| styles.get(&entry).cloned())
                .flatten()
        })
        .ok()
        .flatten()
}

/// Memoize a projection resolved for the cache generation `key`. Nothing is
/// stored for a generation that has since been cleared, or for an owner that
/// no longer exists or was registered on another thread.
pub(crate) fn StoreExportedStyle(
    key: ExportedStyleCacheKey,
    entry: (usize, usize),
    style: Arc<PaintStyleData>,
) {
    let _ = EXPORTED_STYLES.try_with(|registry| {
        let Ok(mut registry) = registry.try_borrow_mut() else {
            return;
        };
        if let Some((generation, styles)) = registry.get_mut(&key.owner) {
            if *generation == key.generation {
                styles.insert(entry, style);
            }
        }
    });
}

#[derive(Default)]
pub struct LayoutBoundaryCache {
    pub(crate) styles: ExportedStyleCacheOwner,
    pub(crate) objects: Vec<*mut LayoutObject>,
    pub(crate) source_objects: Arc<Vec<*mut LayoutObject>>,
    pub(crate) sticky_objects: Arc<Vec<*mut LayoutObject>>,
    pub(crate) sources: LayoutBoundaryHashMap<*const LayoutObject, LayoutBoundarySourceEntry>,
    pub(crate) style_sources:
        LayoutBoundaryHashMap<*const StyleComputedStyle, LayoutBoundaryStyleSourceEntry>,
    // Chromium fragment/item identities remain stable while a LayoutResult is
    // retained. Chromium keeps the corresponding FragmentData identity across
    // cull-rect updates; preserve the same identity in fragment-local Paint
    // data so PrePaint does not mistake a retained source owner for a new one.
    pub(crate) fragment_identity_result: usize,
    /// Chromium-style resident FragmentData state. Replacement LayoutResults
    /// rebind source owners to the same stable fragment slots.
    pub(crate) fragment_paint_state: Option<Arc<crate::layout_engine::LayoutPaintState>>,
    pub(crate) fragment_instance_ids: LayoutBoundaryHashMap<FragmentIdentityKey, u64>,
    pub(crate) next_fragment_instance_id: u64,
    /// `style_sources` / `sticky_objects` predate a compatible style
    /// replacement and must be rebuilt before their next read.
    pub(crate) style_indices_stale: bool,
    /// Present only while a Paint publication runs. Population certifies each
    /// fragment-local input as it binds it, after its children, the way
    /// PrePaintTreeWalk consumes dirty bits in one pass; no separate
    /// certification walk follows publication.
    pub(crate) pre_paint_certification:
        Option<layout_fragment::fragment_tree::pre_paint_revision::PrePaintRevisionTracker>,
    /// The last Paint publication's slots in preorder (population replay).
    pub(crate) population_replay:
        crate::internal::boundary::layout_boundary::layout_fragment_fragment_export_state::PopulationReplay,
}

impl LayoutBoundaryCache {
    /// A LayoutTreeBuilder transaction (insert, remove, reattach) changes the
    /// document-order object graph, so every index keyed by LayoutObject or
    /// ComputedStyle address is dropped. Chromium does not discard the
    /// FragmentData of objects the transaction left in place: their
    /// PaintProperties, cached display items and PrePaint state survive and
    /// are revisited only through their own dirty bits. Keep the matching
    /// resident state here. Its keys are process-unique FragmentData /
    /// DisplayItemClient ids, never addresses, so a removed object's entry is
    /// simply not found again and is dropped at the next publication.
    ///
    /// Source: core/layout/layout_object.cc (AddChild/RemoveChild mark only
    /// the affected object and its container chain);
    ///         core/paint/fragment_data.h (FragmentData lives on its object).
    pub fn ClearTopology(&mut self) {
        self.styles.clear();
        self.objects.clear();
        self.source_objects = Arc::default();
        self.sticky_objects = Arc::default();
        self.sources.clear();
        self.style_sources.clear();
        self.style_indices_stale = false;
    }

    /// A tree update that only updated existing objects in place (no object
    /// created, removed or moved) keeps the document-order object graph:
    /// keep the object and source indices, drop what is keyed by or derived
    /// from replaced styles and node facts (as a compatible style
    /// replacement does), and rebuild the style indices on their next read.
    pub fn ClearUpdatedObjects(&mut self) {
        if self.objects.is_empty() {
            return;
        }
        self.styles.clear();
        self.style_indices_stale = true;
    }

    pub fn Clear(&mut self) {
        self.styles.clear();
        self.objects.clear();
        self.source_objects = Arc::default();
        self.sticky_objects = Arc::default();
        self.sources.clear();
        self.style_sources.clear();
        self.style_indices_stale = false;
        self.fragment_identity_result = 0;
        self.fragment_paint_state = None;
        self.fragment_instance_ids.clear();
        self.next_fragment_instance_id = 0;
    }
}

// cpp: layoutng/layout_boundary_support.h:25-35
#[derive(Clone, Copy, Default)]
pub struct LayoutBoundarySupport {
    pub create_environment:
        Option<fn(&mut LayoutObject, &ConstraintSpace) -> Box<dyn LayoutBoundaryEnvironment>>,
    pub prepare_constraints:
        Option<fn(&StyleComputedStyle, &ConstraintSpace, bool) -> LayoutConstraintSpace>,
    pub prepare_tree: Option<
        fn(
            &mut LayoutObject,
            &ConstraintSpace,
            &LayoutConstraintSpace,
            bool,
            bool,
            bool,
            bool,
            Option<&[*mut LayoutObject]>,
        ),
    >,
    pub publish_layout_result: Option<
        fn(
            &mut LayoutObject,
            &LayoutResult,
            &ConstraintSpace,
            &mut LayoutBoundaryCache,
        ) -> LayoutResultHandle,
    >,
}
