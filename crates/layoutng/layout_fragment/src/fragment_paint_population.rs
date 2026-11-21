//! Chromium physical-fragment and inline-item paint population.
//!
//! This assembly-only fragment owner retains reciprocal source types while
//! placing the two recursive hot paths under `layout_fragment` symbols.

#![allow(non_snake_case)]

use super::*;
use super::layout_fragment_fragment_export_state::{PopulationReplayEntry, PopulationReplayOwner};
use crate::fragment_tree::ResidentBoxGeometry as _;

/// Store a payload field only when its value changes. The payload is shared
/// copy-on-write with the previous publication, so an identical store would
/// still deep-copy the whole PaintProperties.
macro_rules! update_paint {
    ($paint:expr, $field:ident, $value:expr) => {{
        let value = $value;
        if $paint.$field != value {
            $paint.$field = value;
        }
    }};
}

/// Store a payload field only when its value changes. The payload is shared
/// copy-on-write with the previous publication, so an identical store would
/// still deep-copy the whole PaintProperties.
macro_rules! update_paint {
    ($paint:expr, $field:ident, $value:expr) => {{
        let value = $value;
        if $paint.$field != value {
            $paint.$field = value;
        }
    }};
}

fn RetainedResourcesAreCompatible(
    previous: Option<&Arc<PaintResources>>,
    resources: &Arc<PaintResources>,
    owners: &mut FragmentOwners,
) -> bool {
    let Some(previous) = previous else {
        return false;
    };
    if Arc::ptr_eq(previous, resources) {
        return true;
    }
    let key = Arc::as_ptr(previous) as usize;
    *owners
        .retained_resource_compatibility
        .entry(key)
        .or_insert_with(|| {
            previous.fonts == resources.fonts
                && previous.images == resources.images
                && previous.canvases == resources.canvases
                && previous.device_pixel_ratio == resources.device_pixel_ratio
                && previous.preferred_color_scheme == resources.preferred_color_scheme
                && previous.layout_zoom_factor == resources.layout_zoom_factor
                && previous.base_background_color == resources.base_background_color
                && previous.paints_opaque_base_background == resources.paints_opaque_base_background
                && previous.should_clear_document_background
                    == resources.should_clear_document_background
        })
}

fn RetainedPaintProperties(
    object: *const LayoutObject,
    purpose: LayoutUpdateKind,
    effective_style: *const StyleComputedStyle,
    instance: u64,
    resources: &Arc<PaintResources>,
    paint_state: &crate::layout_engine::LayoutPaintState,
    owners: &mut FragmentOwners,
) -> Option<PrePaintInvalidation> {
    if purpose != LayoutUpdateKind::Paint {
        return None;
    }
    // Dirty bits prove the paint inputs unchanged only while the object still
    // owns the ComputedStyle the projection was exported from. A tree-builder
    // transaction can install a replacement style on a reused LayoutObject;
    // Chromium's FragmentData has no copied style to go stale, so re-export
    // whenever the owner's current style differs.
    if !object.is_null() {
        let expected = if effective_style.is_null() {
            unsafe { &*object }.StyleRef() as *const StyleComputedStyle
        } else {
            effective_style
        };
        let paint = paint_state.ResidentPaintProperties(instance)?;
        let exported_from_expected = paint.style.IsExportedFrom(expected)
            || (paint.style.IsExportedFrom(std::ptr::null())
                && !foundation::IsManagedLayoutAddress(expected));
        if !exported_from_expected {
            return None;
        }
    }
    let invalidation = if object.is_null() {
        PrePaintInvalidation::default()
    } else {
        unsafe { &*object }
            .GetMutableForPainting()
            .EnsureIsReadyForPaintInvalidation();
        let invalidation = ExportPrePaintInvalidation(object);
        if invalidation.own() || invalidation.descendants() {
            return None;
        }
        invalidation
    };
    paint_state.ResidentFragment(instance)?;
    // Every resident slot was last painted with the document resources its
    // publication (or a later Page resource update) installed on the state.
    RetainedResourcesAreCompatible(paint_state.DocumentResources(), resources, owners).then_some(invalidation)
}

// Chromium reads tree order directly from its resident LayoutObject graph.
// The exported projection is a dense document-order index, so an insertion
// shifts it for every later object even though that object's own paint inputs
// (and its source dirty bits) are unchanged. A retained FragmentData keeps its
// properties; only this position projection is re-read from the current
// document-order index.
fn RefreshRetainedTreeOrder(
    object: *const LayoutObject,
    sources: &SourceMap,
    output: &mut FragmentPaintData,
) {
    if object.is_null() {
        return;
    }
    if let Some(entry) = sources.get(&object) {
        if output.paint.logical_tree_order != entry.logical_tree_order {
            output.paint.logical_tree_order = entry.logical_tree_order;
        }
    }
}

// Chromium grows and shrinks a LayoutObject's FragmentData list while
// PrePaint walks its physical fragments (core/paint/pre_paint_tree_walk.cc,
// GetOrCreateFragmentData). This publication walk is Rechrom's equivalent
// source-fragment traversal, so establish the same one-entry-per-physical-
// fragment invariant before asking a fragment for its resident data.
fn SynchronizeBoxFragmentData(fragment: &PhysicalFragment) {
    if !fragment.IsBox() {
        return;
    }
    let object = fragment.GetLayoutObject();
    if object.is_null() {
        return;
    }
    let layout_box = DynamicTo::<LayoutBox>(object);
    let Some(layout_box) = (unsafe { layout_box.as_ref() }) else {
        return;
    };
    let physical_count = layout_box.PhysicalFragmentCount() as u32;
    debug_assert_ne!(physical_count, 0);

    let owner = unsafe { &*object };
    let old_count = owner.GetMutableForPainting().FragmentList().size();
    if old_count == physical_count {
        return;
    }

    let mut mutable = owner.GetMutableForPainting();
    mutable.FragmentCountChanged();
    if old_count < physical_count {
        while mutable.FragmentList().size() < physical_count {
            mutable.FragmentList().AppendNewFragment();
        }
        // Chromium forces paint-property construction for newly added entries.
        mutable.SetOnlyThisNeedsPaintPropertyUpdate();
    } else {
        mutable.FragmentList().Shrink(physical_count);
    }
}

// Source: PrePaintTreeWalk::GetOrCreateFragmentData and
// AccompaniedFragmentIterator::Advance (one entry per inline container, not
// per line box). Reuse the existing publication traversal; do not perform a
// second same-object-item walk to discover the final count.
fn SynchronizeInlineFragmentData(item: &FragmentItem, container: u64, owners: &mut FragmentOwners) {
    let object = item.GetLayoutObject();
    let Some(owner) = (unsafe { object.as_ref() }) else {
        return;
    };
    if !owner.IsLayoutInline() {
        return;
    }
    let entry = owners
        .inline_fragment_data
        .entry(object)
        .or_insert((container, 1));
    if entry.0 != container {
        entry.0 = container;
        entry.1 += 1;
    }
    let count = entry.1;
    let mut mutable = owner.GetMutableForPainting();
    let previous_count = mutable.FragmentList().size();
    if previous_count < count || (item.IsLastForNode() && previous_count > count) {
        mutable.FragmentCountChanged();
        while mutable.FragmentList().size() < count {
            mutable.FragmentList().AppendNewFragment();
        }
        if item.IsLastForNode() && mutable.FragmentList().size() > count {
            mutable.FragmentList().Shrink(count);
        }
        mutable.SetOnlyThisNeedsPaintPropertyUpdate();
    }
}

fn PopulateInlineItemLocalPaintData(
    item: &FragmentItem,
    fragment_items: &layout_fragment::fragment_items::FragmentItems,
    parent_offset: PhysicalOffset,
    sources: &SourceMap,
    style_sources: &StyleSourceMap,
    resources: &Arc<PaintResources>,
    purpose: LayoutUpdateKind,
    paint_state: &crate::layout_engine::LayoutPaintState,
    paint_state_owners: &mut FragmentOwners,
    container_fragment_instance_id: u64,
    fragmentainer_instance_id: u64,
    next_fragment_instance_id: &mut u64,
    next_fragmentainer_instance_id: &mut u64,
) -> (*mut FragmentPaintData, bool) {
    let prefix_started = browser_tracing::enabled().then(std::time::Instant::now);
    let object = item.GetLayoutObject();
    if purpose == LayoutUpdateKind::Paint {
        SynchronizeInlineFragmentData(item, container_fragment_instance_id, paint_state_owners);
    }
    let item_index =
        unsafe { (item as *const FragmentItem).offset_from(fragment_items.Items().as_ptr()) };
    assert!(item_index >= 0);
    let (identity_kind, identity_start, identity_end) = match item.Type() {
        ItemType::kText => (0, item.StartOffset(), item.EndOffset()),
        ItemType::kGeneratedText => (1, item.StartOffset(), item.EndOffset()),
        ItemType::kLine => (2, 0, 0),
        ItemType::kBox => (3, 0, 0),
        ItemType::kInvalid => unreachable!("invalid inline item"),
    };
    let uses_first_line_style = item.UsesFirstLineStyle();
    let effective_style = if uses_first_line_style {
        if item.Type() == ItemType::kLine && !object.is_null() {
            unsafe { &*object }.FirstLineStyleRef() as *const StyleComputedStyle
        } else {
            item.Style() as *const StyleComputedStyle
        }
    } else {
        std::ptr::null()
    };
    let display_item_client_id = if object.is_null() {
        0
    } else {
        unsafe { &*(object as *const DisplayItemClient) }.Id() as u64
    };
    let identity_key = FragmentIdentityKey::InlineItem {
        container: container_fragment_instance_id,
        client: display_item_client_id,
        fragment: item.FragmentId(),
        kind: identity_kind,
        start: identity_start,
        end: identity_end,
        index: item_index as u32,
    };
    let fragment_instance_id =
        paint_state_owners.fragment_instance(identity_key, next_fragment_instance_id);
    let retained_invalidation = RetainedPaintProperties(
        object,
        purpose,
        effective_style,
        fragment_instance_id,
        resources,
        paint_state,
        paint_state_owners,
    );
    let store_started = browser_tracing::enabled().then(std::time::Instant::now);
    paint_state_owners.population_stats.RecordPrefix(prefix_started, store_started);
    let binding =
        paint_state.BindInline(fragment_items, item_index as usize, fragment_instance_id);
    paint_state_owners.begin_replay_entry(PopulationReplayEntry {
        owner: PopulationReplayOwner::Inline(fragment_items as *const _, item_index as usize),
        slot: binding.data as usize,
        instance: fragment_instance_id,
        key: identity_key,
        object,
        effective_style,
        paint_state_owner: None,
        inline_container: (purpose == LayoutUpdateKind::Paint).then_some(container_fragment_instance_id),
        len: 1,
        replayable: false,
    });
    let replay_entry = paint_state_owners.replay_next.entries.len() - 1;
    let (output, slot_retained) = (binding.data, binding.retained);
    let slot = output;
    // Atomic inline boxes populate their own box children here.
    let mut descendant_changed = false;
    paint_state_owners
        .population_stats
        .CountBinding(retained_invalidation.is_some(), binding.same_owner, false);
    let local_timing = (
        store_started,
        retained_invalidation.is_some(),
        binding.same_owner,
    );
    if let Some(started) = store_started {
        paint_state_owners.population_stats.sidecar_store_ns = paint_state_owners
            .population_stats
            .sidecar_store_ns
            .saturating_add(started.elapsed().as_nanos().min(u64::MAX as u128) as u64);
    }
    let output = unsafe { &mut *output };
    // Same (FragmentItems, index) owner: the immutable item, its line and its
    // container-relative offsets are those last published. Atomic inline
    // boxes keep the full path because they recurse into and register their
    // own box fragment.
    let reused_item = (retained_invalidation.is_some()
        && binding.same_owner
        && item.Type() != ItemType::kBox)
        .then(|| {
            (
                output.kind,
                output.offset,
                output.size,
                output.content_size,
                output.text_start,
                output.text_end,
                output.text_paint_timing_non_whitespace,
            )
        });
    if let Some(invalidation) = retained_invalidation {
        assert!(
            slot_retained,
            "retained paint properties have no resident slot"
        );
        output.prepare_for_publication();
        output.pre_paint_invalidation = invalidation;
        RefreshRetainedTreeOrder(object, sources, output);
    } else {
        let export_started = browser_tracing::enabled().then(std::time::Instant::now);
        let properties = ExportProperties(
            object,
            std::ptr::null_mut(),
            sources,
            style_sources,
            effective_style,
            resources,
            purpose,
            Some(&mut paint_state_owners.exported_styles),
        );
        if let Some(started) = export_started {
            paint_state_owners.population_stats.property_export_ns = paint_state_owners
                .population_stats
                .property_export_ns
                .saturating_add(started.elapsed().as_nanos().min(u64::MAX as u128) as u64);
        }
        // The binding recorded this slot's resident owner; a fresh payload keeps it.
        let resident_owner = output.resident_owner;
        let certified = std::mem::take(&mut output.certified);
        *output = FragmentPaintData::new(properties);
        output.resident_owner = resident_owner;
        output.certified = certified;
        if purpose == LayoutUpdateKind::Paint && !object.is_null() {
            // ExportProperties has made the object ready for invalidation.
            output.pre_paint_invalidation = ExportPrePaintInvalidation(object);
        }
    }
    paint_state_owners.population_stats.inline_items += 1;
    output.fragment_instance_id = fragment_instance_id;
    output.fragmentainer_instance_id = fragmentainer_instance_id;
    output.node_id = InputId(object);
    // cpp: core/paint/box_fragment_painter.cc:2025-2026,2071-2072
    // cpp: core/paint/inline_box_fragment_painter.cc:63-64
    update_paint!(output.paint, display_item_fragment, item.FragmentId());
    if let Some((
        kind,
        offset,
        size,
        content_size,
        text_start,
        text_end,
        text_paint_timing_non_whitespace,
    )) = reused_item
    {
        output.kind = kind;
        output.offset = offset;
        output.size = size;
        output.content_size = content_size;
        output.text_start = text_start;
        output.text_end = text_end;
        output.text_paint_timing_non_whitespace = text_paint_timing_non_whitespace;
        if kind == FragmentKind::kLine {
            output.node_id = 0;
        }
        // Text line geometry (core/paint/text_fragment_painter.cc:69-103) is
        // read live by Paint from the containing line and container
        // FragmentData, so a retained item carries no line-top snap.
        let entry = &mut paint_state_owners.replay_next.entries[replay_entry];
        entry.replayable = !output.pre_paint_invalidation.own()
            && !output.pre_paint_invalidation.descendants();
        paint_state_owners.population_stats.RecordLocal(local_timing);
        return (slot, false);
    }
    let rect = *item.RectInContainerFragment();
    output.offset = Offset {
        x: Number(rect.offset.left - parent_offset.left),
        y: Number(rect.offset.top - parent_offset.top),
    };
    output.size = Size {
        width: Number(rect.size.width),
        height: Number(rect.size.height),
    };
    {
        output.content_size = output.size;
    }
    // Box fragments with a live style resolve nine-piece images in Paint
    // (ResidentBoxGeometry::BorderImage); only detached projections store them.
    if purpose == LayoutUpdateKind::Paint
        && item.Type() == ItemType::kBox
        && !object.is_null()
        && output.paint.style.Source().is_none()
    {
        let style = if effective_style.is_null() {
            unsafe { &*object }.StyleRef()
        } else {
            unsafe { &*effective_style }
        };
        let borders = output.Borders();
        ExportNinePieceImages(style, output.size, resources, borders, &mut output.paint);
    }
    if purpose == LayoutUpdateKind::Paint {
        ExportInlineInkOverflow(item, &mut output.paint);
    }
    match item.Type() {
        ItemType::kText | ItemType::kGeneratedText => {
            output.kind = if item.Type() == ItemType::kText {
                FragmentKind::kText
            } else {
                FragmentKind::kGeneratedText
            };
        }
        ItemType::kLine => {
            output.kind = FragmentKind::kLine;
            output.node_id = 0;
        }
        ItemType::kBox => {
            output.kind = FragmentKind::kBox;
            let box_fragment = item.BoxFragment();
            if !box_fragment.is_null() {
                let box_fragment = unsafe { &*box_fragment };
                {
                    ExportBoxPaintGeometry(box_fragment, &mut output.paint);
                }
                if purpose == LayoutUpdateKind::Paint && LiveScrollContainer(box_fragment).is_some() {
                    output.paint_cull_scroll_offset = Some(output.paint.ScrollOffset());
                }
                let owner_fragment = item.PostLayoutBoxFragment();
                let owner_fragment = if owner_fragment.is_null() {
                    box_fragment as *const PhysicalBoxFragment
                } else {
                    owner_fragment
                };
                output.paint.establishes_paint_state =
                    paint_state_owners.insert(owner_fragment, output.fragment_instance_id);
                paint_state_owners
                    .instances
                    .entry(box_fragment as *const PhysicalBoxFragment)
                    .or_insert(output.fragment_instance_id);
                {
                    output.content_size = ExportContentSize(box_fragment);
                }
                if item.DescendantsCount() <= 1 {
                    let nested_items = box_fragment.Items();
                    for child in box_fragment.Children() {
                        if !child.is_present() || (!nested_items.is_null() && child.IsLineBox()) {
                            continue;
                        }
                        descendant_changed |= PopulatePhysicalFragmentPaintData(
                            child,
                            child.Offset(),
                            sources,
                            style_sources,
                            resources,
                            purpose,
                            paint_state,
                            paint_state_owners,
                            fragmentainer_instance_id,
                            item.FragmentId(),
                            next_fragment_instance_id,
                            next_fragmentainer_instance_id,
                        );
                    }
                    if !nested_items.is_null() {
                        let span = unsafe { &*nested_items }.Items();
                        let mut index = 0;
                        while index < span.len() {
                            let changed;
                            (index, changed) = PopulateInlineItemPaintData(
                                span,
                                unsafe { &*nested_items },
                                index,
                                span.len(),
                                PhysicalOffset::default(),
                                sources,
                                style_sources,
                                resources,
                                purpose,
                                paint_state,
                                paint_state_owners,
                                output.fragment_instance_id,
                                fragmentainer_instance_id,
                                next_fragment_instance_id,
                                next_fragmentainer_instance_id,
                            );
                            descendant_changed |= changed;
                        }
                    }
                }
                descendant_changed |= PopulateSVGChildPaintData(
                    object,
                    sources,
                    style_sources,
                    resources,
                    purpose,
                    paint_state,
                    paint_state_owners,
                    fragmentainer_instance_id,
                    item.FragmentId(),
                    next_fragment_instance_id,
                    next_fragmentainer_instance_id,
                );
            }
        }
        ItemType::kInvalid => unreachable!("invalid inline item"),
    }
    paint_state_owners.population_stats.RecordLocal(local_timing);
    (slot, descendant_changed)
}

#[inline(never)]
fn PopulateInlineItemPaintData(
    items: &[FragmentItem],
    fragment_items: &layout_fragment::fragment_items::FragmentItems,
    index: usize,
    limit: usize,
    parent_offset: PhysicalOffset,
    sources: &SourceMap,
    style_sources: &StyleSourceMap,
    resources: &Arc<PaintResources>,
    purpose: LayoutUpdateKind,
    paint_state: &crate::layout_engine::LayoutPaintState,
    paint_state_owners: &mut FragmentOwners,
    container_fragment_instance_id: u64,
    fragmentainer_instance_id: u64,
    next_fragment_instance_id: &mut u64,
    next_fragmentainer_instance_id: &mut u64,
) -> (usize, bool) {
    assert!(index < limit);
    let item = &items[index];
    let subtree_size = 1_usize.max(item.DescendantsCount() as usize);
    let subtree_end = limit.min(index + subtree_size);
    let replay_begin = (paint_state_owners.replay_next.entries.len(), paint_state_owners.replay_blocked);
    let (slot, mut descendant_changed) = PopulateInlineItemLocalPaintData(
        item,
        fragment_items,
        parent_offset,
        sources,
        style_sources,
        resources,
        purpose,
        paint_state,
        paint_state_owners,
        container_fragment_instance_id,
        fragmentainer_instance_id,
        next_fragment_instance_id,
        next_fragmentainer_instance_id,
    );
    let mut child_index = index + 1;
    while child_index < subtree_end {
        let changed;
        (child_index, changed) = PopulateInlineItemPaintData(
            items,
            fragment_items,
            child_index,
            subtree_end,
            *item.OffsetInContainerFragment(),
            sources,
            style_sources,
            resources,
            purpose,
            paint_state,
            paint_state_owners,
            container_fragment_instance_id,
            fragmentainer_instance_id,
            next_fragment_instance_id,
            next_fragmentainer_instance_id,
        );
        descendant_changed |= changed;
    }
    paint_state_owners.finish_replay_entry(replay_begin);
    (
        subtree_end,
        paint_state_owners.certify(slot, descendant_changed),
    )
}


pub(super) fn ReplayCheckEnabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("RECHROM_EXPORT_REPLAY_CHECK").is_some())
}

fn ReplayOwnerMatches(slot: &FragmentPaintData, owner: PopulationReplayOwner) -> bool {
    match owner {
        PopulationReplayOwner::Physical(fragment) => {
            slot.resident_owner
                == layout_fragment_boundary::ResidentFragmentOwner::Physical(fragment as usize)
        }
        PopulationReplayOwner::Inline(items, index) => {
            slot.resident_owner
                == layout_fragment_boundary::ResidentFragmentOwner::InlineItem {
                    items: items as usize,
                    index,
                }
        }
    }
}

/// Whether every slot of the previous publication's `entries` can be
/// rebound unchanged: same immutable owner, clean dirty bits, the style the
/// projection was exported from, the same identity and compatible resources
/// (exactly what RetainedPaintProperties and the reuse paths prove).
fn ReplayValid(
    entries: &[PopulationReplayEntry],
    resources: &Arc<PaintResources>,
    paint_state: &crate::layout_engine::LayoutPaintState,
    owners: &mut FragmentOwners,
) -> bool {
    // Every slot shares the document resources the previous publication
    // installed; prove them once.
    if !RetainedResourcesAreCompatible(paint_state.DocumentResources(), resources, owners) {
        return false;
    }
    for entry in entries {
        if !entry.replayable || owners.peek_instance(&entry.key) != Some(entry.instance) {
            return false;
        }
        // The recorded slot: published slots are carried, as the same shared
        // allocations, into every replacement result's resident state.
        let slot = unsafe { &*(entry.slot as *const FragmentPaintData) };
        if !ReplayOwnerMatches(slot, entry.owner)
            || slot.pre_paint_invalidation.own()
            || slot.pre_paint_invalidation.descendants()
        {
            return false;
        }
        if !entry.object.is_null() {
            let expected = if entry.effective_style.is_null() {
                unsafe { &*entry.object }.StyleRef() as *const StyleComputedStyle
            } else {
                entry.effective_style
            };
            if !(slot.paint.style.IsExportedFrom(expected)
                || (slot.paint.style.IsExportedFrom(std::ptr::null())
                    && !foundation::IsManagedLayoutAddress(expected)))
            {
                return false;
            }
        }
    }
    true
}

/// Rebind the previous publication's unchanged subtree `entries` (all
/// proven by ReplayValid) to this publication: the bookkeeping the
/// population walk does for a retained slot, without re-deriving its
/// payload. Returns whether any certified revision changed.
fn ReplaySubtree(
    entries: &[PopulationReplayEntry],
    sources: &SourceMap,
    resources: &Arc<PaintResources>,
    paint_state: &crate::layout_engine::LayoutPaintState,
    owners: &mut FragmentOwners,
    next_fragment_instance_id: &mut u64,
) -> bool {
    let mut slots = Vec::with_capacity(entries.len());
    for entry in entries {
        let instance = owners.fragment_instance(entry.key, next_fragment_instance_id);
        debug_assert_eq!(instance, entry.instance);
        let binding = match entry.owner {
            PopulationReplayOwner::Physical(fragment) => {
                owners.population_stats.physical_fragments += 1;
                paint_state.BindPhysical(unsafe { &*fragment }, instance)
            }
            PopulationReplayOwner::Inline(items, index) => {
                let items = unsafe { &*items };
                if let Some(container) = entry.inline_container {
                    SynchronizeInlineFragmentData(&items.Items()[index], container, owners);
                }
                owners.population_stats.inline_items += 1;
                paint_state.BindInline(items, index, instance)
            }
        };
        owners.population_stats.CountBinding(
            true,
            binding.same_owner,
            matches!(entry.owner, PopulationReplayOwner::Physical(_)),
        );
        let output = unsafe { &mut *binding.data };
        RefreshRetainedTreeOrder(entry.object, sources, output);
        if let Some(fragment) = entry.paint_state_owner {
            let establishes = owners.insert(fragment, instance);
            update_paint!(output.paint, establishes_paint_state, establishes);
        }
        let mut copy = entry.clone();
        copy.instance = instance;
        copy.slot = binding.data as usize;
        let begin = owners.begin_replay_entry(copy);
        slots.push((binding.data, begin));
    }
    // Certify after descendants (reverse preorder), as population does.
    let mut changed = vec![false; entries.len()];
    for i in (0..entries.len()).rev() {
        let len = entries[i].len;
        let descendant_changed = changed[i + 1..i + len].iter().any(|c| *c);
        changed[i] = if descendant_changed {
            owners.certify(slots[i].0, true)
        } else {
            owners.certify_replayed(slots[i].0)
        };
    }
    // Close the copied entries (already finished subtrees: keep their
    // recorded lengths and flags).
    for (_, (index, _)) in &slots {
        let entry = &owners.replay_next.entries[*index];
        let instance = entry.instance;
        owners.replay_next.index.insert(instance, *index);
    }
    owners.replay_stats[0] += 1;
    owners.replay_stats[1] += entries.len();
    changed.iter().any(|c| *c)
}

/// Check mode: the slot state replay would leave, captured before the
/// population walk rebuilds it.
fn ReplayCheckSnapshot(
    entries: &[PopulationReplayEntry],
    paint_state: &crate::layout_engine::LayoutPaintState,
) -> Vec<Option<(PaintPropertiesHandle, (u64, Offset, Size, Size, u64, Option<u32>, Option<u32>, bool, FragmentKind, u64, u64), PrePaintInvalidation)>> {
    entries
        .iter()
        .map(|entry| {
            let slot = paint_state.ResidentFragment(entry.instance)?;
            Some((
                slot.paint.clone(),
                (
                    slot.node_id,
                    slot.offset,
                    slot.size,
                    slot.content_size,
                    slot.fragmentainer_instance_id,
                    slot.text_start,
                    slot.text_end,
                    slot.text_paint_timing_non_whitespace,
                    slot.kind,
                    slot.pre_paint_revision,
                    slot.pre_paint_subtree_revision,
                ),
                slot.pre_paint_invalidation,
            ))
        })
        .collect()
}

/// Check mode: compare what the population walk produced with the replay
/// prediction; returns the number of mismatching slots.
fn ReplayCheckCompare(
    entries: &[PopulationReplayEntry],
    before: Vec<Option<(PaintPropertiesHandle, (u64, Offset, Size, Size, u64, Option<u32>, Option<u32>, bool, FragmentKind, u64, u64), PrePaintInvalidation)>>,
    paint_state: &crate::layout_engine::LayoutPaintState,
) -> usize {
    let mut mismatches = 0;
    for (entry, before) in entries.iter().zip(before) {
        let (Some(slot), Some((mut paint, fields, invalidation))) =
            (paint_state.ResidentFragment(entry.instance), before)
        else {
            mismatches += 1;
            continue;
        };
        update_paint!(paint, establishes_paint_state, slot.paint.establishes_paint_state);
        update_paint!(paint, logical_tree_order, slot.paint.logical_tree_order);
        let now = (
            slot.node_id,
            slot.offset,
            slot.size,
            slot.content_size,
            slot.fragmentainer_instance_id,
            slot.text_start,
            slot.text_end,
            slot.text_paint_timing_non_whitespace,
            slot.kind,
            slot.pre_paint_revision,
            slot.pre_paint_subtree_revision,
        );
        if paint != slot.paint || fields != now || invalidation != slot.pre_paint_invalidation {
            mismatches += 1;
            if mismatches == 1 {
                browser_tracing::instant_with_text(
                    "layout",
                    "layout_fragment.ReplayMismatch",
                    &format!(
                        "instance {} paint {} fields {} invalidation {}",
                        entry.instance,
                        paint != slot.paint,
                        fields != now,
                        invalidation != slot.pre_paint_invalidation
                    ),
                    &[],
                );
            }
        }
    }
    mismatches
}

pub(super) fn PopulatePhysicalFragmentPaintData(
    fragment: &PhysicalFragment,
    offset: PhysicalOffset,
    sources: &SourceMap,
    style_sources: &StyleSourceMap,
    resources: &Arc<PaintResources>,
    purpose: LayoutUpdateKind,
    paint_state: &crate::layout_engine::LayoutPaintState,
    paint_state_owners: &mut FragmentOwners,
    mut fragmentainer_instance_id: u64,
    mut display_item_fragment: u32,
    next_fragment_instance_id: &mut u64,
    next_fragmentainer_instance_id: &mut u64,
) -> bool {
    let prefix_started = browser_tracing::enabled().then(std::time::Instant::now);
    SynchronizeBoxFragmentData(fragment);
    let object = fragment.GetLayoutObject();
    let kind = if fragment.IsBox() {
        FragmentKind::kBox
    } else {
        FragmentKind::kLine
    };
    let effective_style = if fragment.UsesFirstLineStyle() {
        fragment.Style() as *const StyleComputedStyle
    } else {
        std::ptr::null()
    };
    let fragment_data = if fragment.IsBox() {
        fragment.GetFragmentData().cast_mut()
    } else {
        std::ptr::null_mut()
    };
    let property_fragment_unique_id = if object.is_null() {
        0
    } else {
        PropertyFragmentUniqueId(object, fragment_data)
    };
    let display_item_client_id = if object.is_null() {
        0
    } else {
        unsafe { &*(object as *const DisplayItemClient) }.Id() as u64
    };
    let identity_key = if property_fragment_unique_id != 0 {
        FragmentIdentityKey::FragmentData(property_fragment_unique_id)
    } else {
        FragmentIdentityKey::PhysicalFallback {
            client: display_item_client_id,
            fragment: display_item_fragment,
            kind: if fragment.IsBox() { 0 } else { 1 },
            preorder: paint_state_owners.population_stats.physical_fragments as u32,
        }
    };
    let fragment_instance_id =
        paint_state_owners.fragment_instance(identity_key, next_fragment_instance_id);
    let retained_invalidation = RetainedPaintProperties(
        object,
        purpose,
        effective_style,
        fragment_instance_id,
        resources,
        paint_state,
        paint_state_owners,
    );
    let store_started = browser_tracing::enabled().then(std::time::Instant::now);
    paint_state_owners.population_stats.RecordPrefix(prefix_started, store_started);
    let binding = paint_state.BindPhysical(fragment, fragment_instance_id);
    let replay_begin = paint_state_owners.begin_replay_entry(PopulationReplayEntry {
        owner: PopulationReplayOwner::Physical(fragment as *const PhysicalFragment),
        slot: binding.data as usize,
        instance: fragment_instance_id,
        key: identity_key,
        object,
        effective_style,
        paint_state_owner: None,
        inline_container: None,
        len: 1,
        replayable: false,
    });
    let (output, slot_retained) = (binding.data, binding.retained);
    let slot = output;
    paint_state_owners
        .population_stats
        .CountBinding(retained_invalidation.is_some(), binding.same_owner, true);
    let local_timing = (
        store_started,
        retained_invalidation.is_some(),
        binding.same_owner,
    );
    if let Some(started) = store_started {
        paint_state_owners.population_stats.sidecar_store_ns = paint_state_owners
            .population_stats
            .sidecar_store_ns
            .saturating_add(started.elapsed().as_nanos().min(u64::MAX as u128) as u64);
    }
    let output = unsafe { &mut *output };
    // A LayoutNG cache hit hands the replacement LayoutResult the same
    // immutable PhysicalFragment the slot was last published from. When the
    // owner also carries no PrePaint dirty bits, Chromium's PrePaintTreeWalk
    // does no work for it and Paint reads the unchanged fragment directly.
    // Geometry projected solely from that fragment (and the unchanged style
    // and resources proven by RetainedPaintProperties) is therefore still
    // current; publication-relative and source-object facts are re-derived.
    let reuse_fragment_geometry = retained_invalidation.is_some() && binding.same_owner;
    let mut retained_content_size = None;
    if let Some(invalidation) = retained_invalidation {
        assert!(
            slot_retained,
            "retained paint properties have no resident slot"
        );
        if reuse_fragment_geometry {
            retained_content_size = Some(output.content_size);
        }
        output.prepare_for_publication();
        output.pre_paint_invalidation = invalidation;
        RefreshRetainedTreeOrder(object, sources, output);
    } else {
        let export_started = browser_tracing::enabled().then(std::time::Instant::now);
        let properties = ExportProperties(
            object,
            fragment_data,
            sources,
            style_sources,
            effective_style,
            resources,
            purpose,
            Some(&mut paint_state_owners.exported_styles),
        );
        if let Some(started) = export_started {
            paint_state_owners.population_stats.property_export_ns = paint_state_owners
                .population_stats
                .property_export_ns
                .saturating_add(started.elapsed().as_nanos().min(u64::MAX as u128) as u64);
        }
        // The binding recorded this slot's resident owner; a fresh payload keeps it.
        let resident_owner = output.resident_owner;
        let certified = std::mem::take(&mut output.certified);
        *output = FragmentPaintData::new(properties);
        output.resident_owner = resident_owner;
        output.certified = certified;
        if purpose == LayoutUpdateKind::Paint && !object.is_null() {
            // ExportProperties has made the object ready for invalidation.
            output.pre_paint_invalidation = ExportPrePaintInvalidation(object);
        }
    }
    output.fragment_instance_id = fragment_instance_id;
    paint_state_owners.population_stats.physical_fragments += 1;
    output.node_id = InputId(object);
    update_paint!(output.paint, display_item_fragment, display_item_fragment);
    // IsHiddenForPaint is read live from the resident physical owner
    // (ResidentBoxGeometry::ViewHidden); nothing is stored here.
    output.kind = kind;
    output.offset = Offset {
        x: Number(offset.left),
        y: Number(offset.top),
    };
    output.size = Size {
        width: Number(fragment.Size().width),
        height: Number(fragment.Size().height),
    };
    {
        output.content_size = output.size;
    }
    {
        output.fragmentainer_instance_id = fragmentainer_instance_id;
    }
    if !fragment.IsBox() {
        paint_state_owners.population_stats.RecordLocal(local_timing);
        paint_state_owners.replay_next.entries[replay_begin.0].replayable = reuse_fragment_geometry
            && matches!(identity_key, FragmentIdentityKey::FragmentData(_))
            && !output.pre_paint_invalidation.own()
            && !output.pre_paint_invalidation.descendants();
        paint_state_owners.finish_replay_entry(replay_begin);
        return paint_state_owners.certify(slot, false);
    }
    let box_fragment = To::<PhysicalBoxFragment>(fragment as *const PhysicalFragment);
    let box_fragment = unsafe { &*box_fragment };
    if !reuse_fragment_geometry {
        // IsFirstForNode / CanPaintMultipleFragments are read live from the
        // resident box (ResidentBoxGeometry::ViewCanPaintFragmentOccurrence).
        // Live styles resolve nine-piece images in Paint (BorderImage).
        if purpose == LayoutUpdateKind::Paint
            && !object.is_null()
            && output.paint.style.Source().is_none()
        {
            let style = if effective_style.is_null() {
                unsafe { &*object }.StyleRef()
            } else {
                unsafe { &*effective_style }
            };
            let borders = output.Borders();
            ExportNinePieceImages(style, output.size, resources, borders, &mut output.paint);
        }
    }
    if box_fragment.IsFragmentainerBox() {
        fragmentainer_instance_id = *next_fragmentainer_instance_id;
        *next_fragmentainer_instance_id += 1;
        // cpp: core/paint/box_fragment_painter.cc:398-403,1211-1212
        let token = box_fragment.GetBreakToken();
        display_item_fragment = if token.is_null() {
            0
        } else {
            unsafe { &*token }.SequenceNumber() + 1
        };
    } else if !object.is_null() && unsafe { &*object }.HasLayer() {
        // PaintLayerPainter scopes each physical fragment using its index in
        // the owning LayoutBox, independently of exported preorder numbering.
        // cpp: core/paint/paint_layer_painter.cc:776-778
        let mut fragments =
            crate::fragment_data_iterator::AccompaniedFragmentIterator::new(unsafe { &*object });
        let mut index = 0;
        while let Some(physical) = fragments.GetPhysicalBoxFragment() {
            if std::ptr::eq(physical, box_fragment) {
                display_item_fragment = index;
                break;
            }
            index += 1;
            fragments.Advance();
        }
    }
    update_paint!(output.paint, display_item_fragment, display_item_fragment);
    output.fragmentainer_instance_id = fragmentainer_instance_id;
    if !reuse_fragment_geometry {
        ExportBoxPaintGeometry(box_fragment, &mut output.paint);
    }
    if purpose == LayoutUpdateKind::Paint && LiveScrollContainer(box_fragment).is_some() {
        output.paint_cull_scroll_offset = Some(output.paint.ScrollOffset());
    }
    let establishes_paint_state = paint_state_owners.insert(
        box_fragment as *const PhysicalBoxFragment,
        output.fragment_instance_id,
    );
    paint_state_owners.replay_next.entries[replay_begin.0].paint_state_owner =
        Some(box_fragment as *const PhysicalBoxFragment);
    update_paint!(output.paint, establishes_paint_state, establishes_paint_state);
    output.content_size = match retained_content_size {
        Some(size) if reuse_fragment_geometry => size,
        _ => ExportContentSize(box_fragment),
    };
    paint_state_owners.population_stats.RecordLocal(local_timing);
    let mut descendant_changed = false;
    // Self qualification: a retained, same-owner, clean slot with no state
    // that a population walk re-derives from live (non-fragment) inputs.
    let self_replayable = reuse_fragment_geometry
        && matches!(identity_key, FragmentIdentityKey::FragmentData(_))
        && !output.pre_paint_invalidation.own()
        && !output.pre_paint_invalidation.descendants()
        // Scroll containers and scrollbars are read live, but a scrollable
        // area's controls are positioned at PrePaint; keep those boxes on the
        // full path.
        && unsafe { DynamicTo::<LayoutBox>(box_fragment.GetLayoutObject()).as_ref() }
            .is_none_or(|layout_box| layout_box.GetScrollableArea().is_null())
        && !matches!(
            box_fragment.GetBoxType(),
            crate::physical_fragment::BoxType::kPageContainer
                | crate::physical_fragment::BoxType::kPageBorderBox
        )
        && !box_fragment.IsFragmentainerBox();
    paint_state_owners.replay_next.entries[replay_begin.0].replayable = self_replayable;
    if !self_replayable && browser_tracing::enabled() {
        let reasons = [
            !reuse_fragment_geometry,
            !matches!(identity_key, FragmentIdentityKey::FragmentData(_)),
            output.pre_paint_invalidation.own() || output.pre_paint_invalidation.descendants(),
            !(unsafe { DynamicTo::<LayoutBox>(box_fragment.GetLayoutObject()).as_ref() }
            .is_none_or(|layout_box| layout_box.GetScrollableArea().is_null())),
            false,
            matches!(
                box_fragment.GetBoxType(),
                crate::physical_fragment::BoxType::kPageContainer
                    | crate::physical_fragment::BoxType::kPageBorderBox
            ) || box_fragment.IsFragmentainerBox(),
            false,
            true,
        ];
        let first = reasons.iter().position(|r| *r).unwrap_or(7);
        paint_state_owners.replay_reasons[first] += 1;
    }
    let previous_subtree = if self_replayable {
        paint_state_owners
            .replay_previous
            .index
            .get(&output.fragment_instance_id)
            .copied()
            .filter(|index| {
                paint_state_owners.replay_previous.entries[*index].instance
                    == output.fragment_instance_id
            })
    } else {
        None
    };
    let mut replayed = false;
    let mut check: Option<(usize, usize, Vec<_>)> = None;
    if let Some(index) = previous_subtree {
        let previous = std::mem::take(&mut paint_state_owners.replay_previous);
        let len = previous.entries[index].len;
        let range = &previous.entries[index + 1..index + len];
        if !range.is_empty() && ReplayValid(range, resources, paint_state, paint_state_owners) {
            if ReplayCheckEnabled() {
                check = Some((index + 1, index + len, ReplayCheckSnapshot(range, paint_state)));
            } else {
                descendant_changed = ReplaySubtree(
                    range,
                    sources,
                    resources,
                    paint_state,
                    paint_state_owners,
                    next_fragment_instance_id,
                );
                replayed = true;
            }
        }
        paint_state_owners.replay_previous = previous;
    }
    if !replayed {
        let items = box_fragment.Items();
        for child in box_fragment.Children() {
            if !child.is_present() || (!items.is_null() && child.IsLineBox()) {
                continue;
            }
            descendant_changed |= PopulatePhysicalFragmentPaintData(
                child,
                child.Offset(),
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
        if !items.is_null() {
            let span = unsafe { &*items }.Items();
            let mut index = 0;
            while index < span.len() {
                let changed;
                (index, changed) = PopulateInlineItemPaintData(
                    span,
                    unsafe { &*items },
                    index,
                    span.len(),
                    PhysicalOffset::default(),
                    sources,
                    style_sources,
                    resources,
                    purpose,
                    paint_state,
                    paint_state_owners,
                    output.fragment_instance_id,
                    fragmentainer_instance_id,
                    next_fragment_instance_id,
                    next_fragmentainer_instance_id,
                );
                descendant_changed |= changed;
            }
        }
        descendant_changed |= PopulateSVGChildPaintData(
            object,
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
    if let Some((start, end, before)) = check {
        let previous = std::mem::take(&mut paint_state_owners.replay_previous);
        let mismatches = ReplayCheckCompare(&previous.entries[start..end], before, paint_state);
        paint_state_owners.replay_stats[2] += mismatches;
        paint_state_owners.replay_stats[0] += 1;
        paint_state_owners.replay_stats[1] += end - start;
        paint_state_owners.replay_previous = previous;
    }
    paint_state_owners.finish_replay_entry(replay_begin);
    paint_state_owners.certify(slot, descendant_changed)
}
