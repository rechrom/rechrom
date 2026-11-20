//! Fragment publication identity state and SVG child expansion.
//!
//! These types and traversals are shared by sibling assembly-only fragment
//! owners, but remain physically and symbolically owned by `layout_fragment`.

#![allow(non_snake_case)]

use super::*;

#[derive(Default)]
pub(super) struct PaintPopulationStats {
    pub(super) property_export_ns: u64,
    pub(super) sidecar_store_ns: u64,
    pub(super) physical_fragments: usize,
    pub(super) inline_items: usize,
    /// Retained-slot bindings, split by whether the slot was last published
    /// from this exact immutable fragment owner (a LayoutNG cache hit).
    pub(super) retained_physical: usize,
    pub(super) retained_physical_same_owner: usize,
    pub(super) retained_inline: usize,
    pub(super) retained_inline_same_owner: usize,
    /// Local (non-recursive) export time per binding class.
    pub(super) retained_same_owner_local_ns: u64,
    pub(super) retained_other_owner_local_ns: u64,
    pub(super) exported_local_ns: u64,
    /// Synchronize + identity + retained-properties check, before binding.
    pub(super) prefix_ns: u64,
    /// PrePaint input certification of each bound slot.
    pub(super) certify_ns: u64,
}

impl PaintPopulationStats {
    pub(super) fn CountBinding(&mut self, retained: bool, same_owner: bool, physical: bool) {
        if !retained {
            return;
        }
        if physical {
            self.retained_physical += 1;
            self.retained_physical_same_owner += same_owner as usize;
        } else {
            self.retained_inline += 1;
            self.retained_inline_same_owner += same_owner as usize;
        }
    }

    pub(super) fn RecordPrefix(
        &mut self,
        started: Option<std::time::Instant>,
        ended: Option<std::time::Instant>,
    ) {
        if let (Some(started), Some(ended)) = (started, ended) {
            self.prefix_ns = self
                .prefix_ns
                .saturating_add(ended.duration_since(started).as_nanos().min(u64::MAX as u128) as u64);
        }
    }

    pub(super) fn RecordLocal(&mut self, timing: (Option<std::time::Instant>, bool, bool)) {
        let (Some(started), retained, same_owner) = timing else {
            return;
        };
        let elapsed = started.elapsed().as_nanos().min(u64::MAX as u128) as u64;
        let total = match (retained, same_owner) {
            (true, true) => &mut self.retained_same_owner_local_ns,
            (true, false) => &mut self.retained_other_owner_local_ns,
            (false, _) => &mut self.exported_local_ns,
        };
        *total = total.saturating_add(elapsed);
    }
}

#[derive(Default)]
pub(super) struct FragmentOwners {
    pub(super) paint: LayoutBoundaryHashSet<*const PhysicalBoxFragment>,
    pub(super) instances: LayoutBoundaryHashMap<*const PhysicalBoxFragment, u64>,
    pub(super) exported_styles: ExportedStyleCache,
    pub(super) exported_selection_styles: ExportedSelectionStyleCache,
    pub(super) debug_names: DebugNameCache,
    pub(super) fragment_instance_ids: LayoutBoundaryHashMap<FragmentIdentityKey, u64>,
    // Exact source identities from the preceding LayoutResult. LayoutNG may
    // carry immutable child fragments into a replacement root result during
    // constraint-only layout. Reuse their FragmentData identity while pruning
    // every source pointer not visited by the new result.
    pub(super) prior_fragment_instance_ids: LayoutBoundaryHashMap<FragmentIdentityKey, u64>,
    pub(super) retained_resource_compatibility: LayoutBoundaryHashMap<usize, bool>,
    pub(super) new_fragment_instances: usize,
    pub(super) population_stats: PaintPopulationStats,
    /// GetOrCreateFragmentData's per-object inline-container progression for
    /// this publication. Values are only (last native container, entry count),
    /// never a retained fragment list or another layout tree.
    pub(super) inline_fragment_data: LayoutBoundaryHashMap<*const LayoutObject, (u64, u32)>,
    /// The engine's PrePaint input tracker, lent for a Paint publication.
    pub(super) certification:
        Option<layout_fragment::fragment_tree::pre_paint_revision::PrePaintRevisionTracker>,
    /// The previous publication's slots (replay source) and this one's.
    pub(super) replay_previous: PopulationReplay,
    pub(super) replay_next: PopulationReplay,
    /// Finished entries that do not qualify; an ancestor qualifies only if
    /// none were finished inside its subtree.
    pub(super) replay_blocked: usize,
    /// The FragmentData of the box whose inline items are being populated.
    pub(super) inline_container_data: usize,
    /// Diagnostics: replayed subtrees / slots, check-mode mismatches.
    pub(super) replay_stats: [usize; 3],
    /// Diagnostics: why a box slot does not qualify (first reason).
    pub(super) replay_reasons: [usize; 8],
}
impl FragmentOwners {
    /// Certify one bound slot after all of its children were populated.
    /// Returns whether its subtree changed for the parent's observation.
    pub(super) fn certify(
        &mut self,
        output: *mut FragmentPaintData,
        descendant_changed: bool,
    ) -> bool {
        let Some(tracker) = self.certification.as_mut() else {
            return false;
        };
        let started = browser_tracing::enabled().then(std::time::Instant::now);
        let changed = tracker.ObserveProductionNode(unsafe { &mut *output }, descendant_changed);
        if let Some(started) = started {
            self.population_stats.certify_ns = self
                .population_stats
                .certify_ns
                .saturating_add(started.elapsed().as_nanos().min(u64::MAX as u128) as u64);
        }
        changed
    }

    /// certify for a slot population replayed unchanged.
    pub(super) fn certify_replayed(&mut self, output: *mut FragmentPaintData) -> bool {
        let Some(tracker) = self.certification.as_mut() else {
            return false;
        };
        let started = browser_tracing::enabled().then(std::time::Instant::now);
        let changed = tracker.ReplayProductionNode(unsafe { &mut *output });
        if let Some(started) = started {
            self.population_stats.certify_ns = self
                .population_stats
                .certify_ns
                .saturating_add(started.elapsed().as_nanos().min(u64::MAX as u128) as u64);
        }
        changed
    }

    pub(super) fn insert(&mut self, source: *const PhysicalBoxFragment, instance: u64) -> bool {
        self.instances.entry(source).or_insert(instance);
        self.paint.insert(source)
    }

    /// The instance `key` resolves to in this publication, without
    /// claiming it (see fragment_instance).
    pub(super) fn peek_instance(&self, key: &FragmentIdentityKey) -> Option<u64> {
        self.fragment_instance_ids
            .get(key)
            .or_else(|| self.prior_fragment_instance_ids.get(key))
            .copied()
    }

    /// Start recording a bound slot (finish_replay_entry closes it).
    pub(super) fn begin_replay_entry(&mut self, entry: PopulationReplayEntry) -> (usize, usize) {
        let index = self.replay_next.entries.len();
        self.replay_next.entries.push(entry);
        (index, self.replay_blocked)
    }

    /// Close a recorded slot after its subtree; `begin` from begin_replay_entry.
    pub(super) fn finish_replay_entry(&mut self, begin: (usize, usize)) {
        let (index, blocked) = begin;
        let len = self.replay_next.entries.len() - index;
        let entry = &mut self.replay_next.entries[index];
        let own = entry.replayable;
        entry.len = len;
        entry.replayable = own && self.replay_blocked == blocked;
        if !own {
            self.replay_blocked += 1;
        }
        let instance = entry.instance;
        self.replay_next.index.insert(instance, index);
    }

    pub(super) fn fragment_instance(
        &mut self,
        key: FragmentIdentityKey,
        next_fragment_instance_id: &mut u64,
    ) -> u64 {
        if let Some(instance) = self.fragment_instance_ids.get(&key) {
            return *instance;
        }
        let instance = if let Some(instance) = self.prior_fragment_instance_ids.remove(&key) {
            instance
        } else {
            self.new_fragment_instances += 1;
            let instance = *next_fragment_instance_id;
            *next_fragment_instance_id += 1;
            instance
        };
        self.fragment_instance_ids.insert(key, instance);
        instance
    }
}

#[inline(never)]
pub(super) fn PopulateSVGChildPaintData(
    object: *const LayoutObject,
    sources: &SourceMap,
    style_sources: &StyleSourceMap,
    resources: &Arc<PaintResources>,
    purpose: LayoutUpdateKind,
    paint_state: &crate::layout_engine::LayoutPaintState,
    paint_state_owners: &mut FragmentOwners,
    fragmentainer_instance_id: u64,
    display_item_fragment: u32,
    next_fragment_instance_id: &mut u64,
    next_fragmentainer_instance_id: &mut u64,
) -> bool {
    let Some(owner) = (unsafe { object.as_ref() }) else {
        return false;
    };
    if !(owner.IsSVGRoot() || (owner.IsSVGContainer() && !owner.IsSVGForeignObject())) {
        return false;
    }
    let mut changed = false;
    let mut child = owner.SlowFirstChild();
    while let Some(layout_child) = unsafe { child.as_ref() } {
        let child_box = DynamicTo::<LayoutBox>(child);
        if let Some(child_box) = unsafe { child_box.as_ref() } {
            let child_offset = child_box.PhysicalLocation();
            for child_fragment in child_box.PhysicalFragments() {
                changed |= PopulatePhysicalFragmentPaintData(
                    child_fragment,
                    child_offset,
                    sources,
                    style_sources,
                    resources,
                    purpose,
                    paint_state,
                    paint_state_owners,
                    fragmentainer_instance_id,
                    display_item_fragment,
                    next_fragment_instance_id,
                    next_fragmentainer_instance_id,
                );
            }
        }
        child = layout_child.NextSibling();
    }
    changed
}

/// The fragment-local paint slots the last Paint publication bound, in
/// preorder, so the next publication can rebind an unchanged subtree
/// without re-deriving it (see TryReplaySubtree in fragment population).
/// Chromium keeps FragmentData on the persistent LayoutObject and walks only
/// dirty objects; a LayoutNG cache hit hands the replacement result the very
/// same immutable fragments, so their published slots stay valid.
#[derive(Default)]
pub(crate) struct PopulationReplay {
    pub(crate) entries: Vec<PopulationReplayEntry>,
    /// Slot instance -> its entry.
    pub(crate) index: LayoutBoundaryHashMap<u64, usize>,
}

#[derive(Clone, Copy)]
pub(crate) enum PopulationReplayOwner {
    Physical(*const PhysicalFragment),
    Inline(*const layout_fragment::fragment_items::FragmentItems, usize),
}

#[derive(Clone)]
pub(crate) struct PopulationReplayEntry {
    pub(crate) owner: PopulationReplayOwner,
    /// The bound slot (a stable, shared FragmentPaintData allocation).
    pub(crate) slot: usize,
    pub(crate) instance: u64,
    pub(crate) key: FragmentIdentityKey,
    pub(crate) object: *const LayoutObject,
    pub(crate) effective_style: *const StyleComputedStyle,
    /// Box fragment registered as establishing paint state.
    pub(crate) paint_state_owner: Option<*const PhysicalBoxFragment>,
    /// Inline item: its container's slot instance (fragment-count progression).
    pub(crate) inline_container: Option<u64>,
    /// Text item: its line top in the container and the container's
    /// FragmentData, whose paint offset an ancestor move changes.
    pub(crate) text_snap: Option<(LayoutUnit, *const crate::fragment_data::FragmentData)>,
    /// Entries in this slot's subtree, itself included.
    pub(crate) len: usize,
    /// While recording: this slot alone qualifies; once finished: the whole
    /// subtree qualifies.
    pub(crate) replayable: bool,
}
