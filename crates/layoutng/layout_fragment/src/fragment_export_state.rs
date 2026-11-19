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
}
impl FragmentOwners {
    /// Certify one bound slot after all of its children were populated.
    /// Returns whether its subtree changed for the parent's observation.
    pub(super) fn certify(
        &mut self,
        output: *mut FragmentPaintData,
        descendant_changed: bool,
    ) -> bool {
        match self.certification.as_mut() {
            Some(tracker) => {
                tracker.ObserveProductionNode(unsafe { &mut *output }, descendant_changed)
            }
            None => false,
        }
    }

    pub(super) fn insert(&mut self, source: *const PhysicalBoxFragment, instance: u64) -> bool {
        self.instances.entry(source).or_insert(instance);
        self.paint.insert(source)
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
