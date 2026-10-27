//! Fragment publication culling, identity state, and SVG child expansion.
//!
//! These types and traversals are shared by sibling assembly-only fragment
//! owners, but remain physically and symbolically owned by `layout_fragment`.

#![allow(non_snake_case)]

use super::*;

#[derive(Clone, Copy)]
pub(super) struct PaintCullRect {
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
}

impl PaintCullRect {
    pub(super) fn intersects_vertical_range(&self, top: f64, bottom: f64) -> bool {
        top < self.bottom && self.top < bottom
    }

    pub(super) fn intersects_horizontal_range(&self, left: f64, right: f64) -> bool {
        left < self.right && self.left < right
    }
}

pub(super) fn AddPhysicalOffsets(a: PhysicalOffset, b: PhysicalOffset) -> PhysicalOffset {
    PhysicalOffset {
        left: a.left + b.left,
        top: a.top + b.top,
    }
}

pub(super) fn SubtractScrollOffset(offset: PhysicalOffset, scroll: Offset) -> PhysicalOffset {
    PhysicalOffset {
        left: offset.left - LayoutUnit::from_f64(scroll.x),
        top: offset.top - LayoutUnit::from_f64(scroll.y),
    }
}

pub(super) fn ShouldCullPhysicalFragment(
    fragment: &PhysicalFragment,
    absolute_offset: PhysicalOffset,
    cull: PaintCullRect,
    owners: &FragmentOwners,
) -> bool {
    if !fragment.IsBox() {
        return false;
    }
    let object = fragment.GetLayoutObject();
    if !object.is_null() && owners.uncullable_objects.contains(&object) {
        return false;
    }
    let box_fragment = To::<PhysicalBoxFragment>(fragment as *const PhysicalFragment);
    let box_fragment = unsafe { &*box_fragment };
    let overflow = if box_fragment.IsInkOverflowComputed() {
        box_fragment.InkOverflowRect()
    } else {
        box_fragment.ScrollableOverflow()
    };
    let left = Number(absolute_offset.left + overflow.offset.left);
    let top = Number(absolute_offset.top + overflow.offset.top);
    let right = left + Number(overflow.size.width);
    let bottom = top + Number(overflow.size.height);
    !(cull.intersects_horizontal_range(left, right) && cull.intersects_vertical_range(top, bottom))
}

#[inline(never)]
pub(super) fn BuildPaintCullRect(
    root: &mut LayoutObject,
    space: &ConstraintSpace,
) -> Option<PaintCullRect> {
    if space.printing {
        return None;
    }
    let viewport = space.viewport?;
    let root_box = DynamicTo::<LayoutBox>(root as *mut LayoutObject);
    if root_box.is_null() {
        return None;
    }
    let area = unsafe { &*root_box }.GetScrollableArea();
    let (scroll_x, scroll_y) = if area.is_null() {
        (0.0, 0.0)
    } else {
        let offset = unsafe { &*area }.GetScrollOffset();
        (offset.x() as f64, offset.y() as f64)
    };
    let zoom = space.layout_zoom_factor.max(f64::MIN_POSITIVE);
    // CullRectUpdater::ExpansionRatio is the document DPR with Chromium's
    // default dpr_coef=1. GeometryMapper additionally carries page zoom into
    // the local-space expansion. Keep the boundary projection's interest
    // range in that same local coordinate system.
    let expansion_ratio = space.device_pixel_ratio.max(f64::MIN_POSITIVE);
    let outset = 4000.0 * expansion_ratio * zoom;
    Some(PaintCullRect {
        left: scroll_x - outset,
        top: scroll_y - outset,
        right: scroll_x + viewport.size.width as f64 * zoom + outset,
        bottom: scroll_y + viewport.size.height as f64 * zoom + outset,
    })
}

pub(super) fn MarkUncullablePaintAncestors(
    objects: &[*mut LayoutObject],
) -> LayoutBoundaryHashSet<*const LayoutObject> {
    crate::layout_fragment_native::mark_uncullable_paint_ancestors(objects)
}

#[derive(Default)]
pub(super) struct FragmentOwners {
    pub(super) paint: LayoutBoundaryHashSet<*const PhysicalBoxFragment>,
    pub(super) instances: LayoutBoundaryHashMap<*const PhysicalBoxFragment, u64>,
    pub(super) exported_styles: ExportedStyleCache,
    pub(super) debug_names: DebugNameCache,
    pub(super) uncullable_objects: Arc<LayoutBoundaryHashSet<*const LayoutObject>>,
    pub(super) fragment_instance_ids: LayoutBoundaryHashMap<(u8, usize), u64>,
    // Exact native identities from the preceding LayoutResult. LayoutNG may
    // carry immutable child fragments into a replacement root result during
    // constraint-only layout. Reuse their FragmentData identity while pruning
    // every native pointer not visited by the new result.
    pub(super) prior_fragment_instance_ids: LayoutBoundaryHashMap<(u8, usize), u64>,
}
impl FragmentOwners {
    pub(super) fn insert(&mut self, native: *const PhysicalBoxFragment, instance: u64) -> bool {
        self.instances.entry(native).or_insert(instance);
        self.paint.insert(native)
    }

    pub(super) fn fragment_instance(
        &mut self,
        kind: u8,
        native: usize,
        next_fragment_instance_id: &mut u64,
    ) -> u64 {
        let key = (kind, native);
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
                    child_offset,
                    None,
                    false,
                    sources,
                    style_sources,
                    resources,
                    purpose,
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
