//! Fragment publication identity state and SVG child expansion.
//!
//! These types and traversals are shared by sibling assembly-only fragment
//! owners, but remain physically and symbolically owned by `layout_fragment`.

#![allow(non_snake_case)]

use super::*;

#[derive(Default)]
pub(super) struct FragmentOwners {
    pub(super) paint: LayoutBoundaryHashSet<*const PhysicalBoxFragment>,
    pub(super) instances: LayoutBoundaryHashMap<*const PhysicalBoxFragment, u64>,
    pub(super) exported_styles: ExportedStyleCache,
    pub(super) debug_names: DebugNameCache,
    pub(super) fragment_instance_ids: LayoutBoundaryHashMap<(u8, usize), u64>,
    // Exact source identities from the preceding LayoutResult. LayoutNG may
    // carry immutable child fragments into a replacement root result during
    // constraint-only layout. Reuse their FragmentData identity while pruning
    // every source pointer not visited by the new result.
    pub(super) prior_fragment_instance_ids: LayoutBoundaryHashMap<(u8, usize), u64>,
}
impl FragmentOwners {
    pub(super) fn insert(&mut self, source: *const PhysicalBoxFragment, instance: u64) -> bool {
        self.instances.entry(source).or_insert(instance);
        self.paint.insert(source)
    }

    pub(super) fn fragment_instance(
        &mut self,
        kind: u8,
        source: usize,
        next_fragment_instance_id: &mut u64,
    ) -> u64 {
        let key = (kind, source);
        if let Some(instance) = self.fragment_instance_ids.get(&key) {
            return *instance;
        }
        let instance = self
            .prior_fragment_instance_ids
            .remove(&key)
            .unwrap_or_else(|| {
                let instance = *next_fragment_instance_id;
                *next_fragment_instance_id += 1;
                instance
            });
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
) {
    let Some(owner) = (unsafe { object.as_ref() }) else {
        return;
    };
    if !(owner.IsSVGRoot() || (owner.IsSVGContainer() && !owner.IsSVGForeignObject())) {
        return;
    }
    let mut child = owner.SlowFirstChild();
    while let Some(layout_child) = unsafe { child.as_ref() } {
        let child_box = DynamicTo::<LayoutBox>(child);
        if let Some(child_box) = unsafe { child_box.as_ref() } {
            let child_offset = child_box.PhysicalLocation();
            for child_fragment in child_box.PhysicalFragments() {
                PopulatePhysicalFragmentPaintData(
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
}
