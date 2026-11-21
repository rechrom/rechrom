//! Lazy initialization of one resident fragment's Paint sidecar.
//!
//! Chromium keeps FragmentData on the persistent LayoutObject and brings it
//! up to date only when PrePaintTreeWalk reaches the object
//! (core/paint/pre_paint_tree_walk.cc, GetOrCreateFragmentData and
//! WalkInternal). Layout publication therefore no longer walks the fragment
//! tree to export Paint data: LayoutPaintState binds a fragment's sidecar the
//! first time a cursor of the current publication reaches it and calls the
//! initializers below once for that binding.

#![allow(non_snake_case)]

use super::*;
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

/// Whether slots last exported with the document resources `previous` stay
/// valid under `resources` (the facts a retained projection derived from
/// them: nine-piece images, fonts, canvases, device scale, layout zoom, base
/// background). The viewport is read live and does not invalidate a slot.
pub(crate) fn PaintResourcesAreCompatible(
    previous: Option<&Arc<PaintResources>>,
    resources: &Arc<PaintResources>,
) -> bool {
    let Some(previous) = previous else {
        return false;
    };
    Arc::ptr_eq(previous, resources)
        || (previous.fonts == resources.fonts
            && previous.images == resources.images
            && previous.canvases == resources.canvases
            && previous.device_pixel_ratio == resources.device_pixel_ratio
            && previous.preferred_color_scheme == resources.preferred_color_scheme
            && previous.layout_zoom_factor == resources.layout_zoom_factor
            && previous.base_background_color == resources.base_background_color
            && previous.paints_opaque_base_background == resources.paints_opaque_base_background
            && previous.should_clear_document_background
                == resources.should_clear_document_background)
}

/// The object's dirty bits after PaintInvalidator's readiness step.
/// Source: core/paint/paint_invalidator.cc:222. LayoutBox uses this point to
/// promote a delayed full invalidation once it intersects the visible
/// viewport (layout_box.cc:2317-2340) before the bits are consumed.
pub(super) fn ReadyPrePaintInvalidation(object: *const LayoutObject) -> PrePaintInvalidation {
    if object.is_null() {
        return PrePaintInvalidation::default();
    }
    unsafe { &*object }
        .GetMutableForPainting()
        .EnsureIsReadyForPaintInvalidation();
    ExportPrePaintInvalidation(object)
}

/// Whether a retained slot's projection is still current: the object still
/// owns the ComputedStyle the projection was exported from (a tree-builder
/// transaction can install a replacement style on a reused LayoutObject) and
/// neither it nor a descendant carries PrePaint dirty bits.
fn RetainedProjectionIsCurrent(
    object: *const LayoutObject,
    effective_style: *const StyleComputedStyle,
    invalidation: PrePaintInvalidation,
    output: &FragmentPaintData,
) -> bool {
    if object.is_null() {
        return true;
    }
    let expected = if effective_style.is_null() {
        unsafe { &*object }.StyleRef() as *const StyleComputedStyle
    } else {
        effective_style
    };
    let exported_from_expected = output.paint.style.IsExportedFrom(expected)
        || (output.paint.style.IsExportedFrom(std::ptr::null())
            && !foundation::IsManagedLayoutAddress(expected));
    exported_from_expected && !invalidation.own() && !invalidation.descendants()
}

/// Rewrite `output` from its source object unless the retained projection is
/// still current. Returns whether the retained payload was kept.
fn InitializeProperties(
    object: *const LayoutObject,
    fragment_data: *mut crate::fragment_data::FragmentData,
    effective_style: *const StyleComputedStyle,
    retained: bool,
    exported_styles: Option<crate::internal::layout_input::ExportedStyleCacheKey>,
    output: &mut FragmentPaintData,
) -> bool {
    // The single readiness step of this binding; exporting the projection
    // below reads no dirty bit.
    let invalidation = ReadyPrePaintInvalidation(object);
    if retained && RetainedProjectionIsCurrent(object, effective_style, invalidation, output) {
        output.pre_paint_invalidation = invalidation;
        return true;
    }
    let properties = ExportProperties(
        object,
        fragment_data,
        effective_style,
        LayoutUpdateKind::Paint,
        exported_styles,
    );
    // The binding recorded this slot's resident owner; a fresh payload keeps it.
    let resident_owner = output.resident_owner;
    *output = FragmentPaintData::new(properties);
    output.resident_owner = resident_owner;
    output.pre_paint_invalidation = invalidation;
    false
}

/// Box fragments with a live style resolve nine-piece images in Paint
/// (ResidentBoxGeometry::BorderImage); only detached projections store them.
fn ExportDetachedNinePieceImages(
    object: *const LayoutObject,
    effective_style: *const StyleComputedStyle,
    resources: Option<&Arc<PaintResources>>,
    output: &mut FragmentPaintData,
) {
    let Some(resources) = resources else {
        return;
    };
    if object.is_null() || output.paint.style.Source().is_some() {
        return;
    }
    let style = if effective_style.is_null() {
        unsafe { &*object }.StyleRef()
    } else {
        unsafe { &*effective_style }
    };
    let size = output.ViewSize();
    let borders = output.Borders();
    let stitched_size = output.ViewStitchedDecoration().map(|stitched| stitched.size);
    ExportNinePieceImages(
        style,
        size,
        stitched_size,
        resources,
        borders,
        &mut output.paint,
    );
}

/// A rebound scroll container's contents cull is recomputed by the PrePaint
/// walk that binds it; record the offset on the scroll owner
/// (cull_rect_updater.cc:67-89,193-206).
fn RecordRebindScrollOrigin(box_fragment: &PhysicalBoxFragment, output: &FragmentPaintData) {
    if LiveScrollContainer(box_fragment).is_some() {
        output
            .paint
            .SetLastCullRectUpdateScrollPosition(output.paint.ScrollOffset());
    }
}

// Chromium grows and shrinks a LayoutObject's FragmentData list while
// PrePaint walks its physical fragments (core/paint/pre_paint_tree_walk.cc,
// GetOrCreateFragmentData): one entry per physical fragment. The first
// binding of any of the box's fragments establishes the invariant before the
// fragment's FragmentData identity is read.
pub(crate) fn SynchronizeBoxFragmentData(owner: &LayoutObject) {
    let Some(layout_box) =
        (unsafe { DynamicTo::<LayoutBox>(owner as *const LayoutObject).as_ref() })
    else {
        return;
    };
    let physical_count = layout_box.PhysicalFragmentCount() as u32;
    if physical_count == 0 {
        return;
    }
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
// AccompaniedFragmentIterator::Advance: a LayoutInline has one FragmentData
// entry per inline container (fragmentainer) holding its items, not one per
// line box. Count the containers along the object's own item chain.
pub(crate) fn SynchronizeInlineFragmentData(object: &LayoutObject) {
    if !object.IsLayoutInline() || !object.IsInLayoutNGInlineFormattingContext() {
        return;
    }
    let mut cursor = crate::inline_cursor::InlineCursor::default();
    cursor.MoveToLayoutObject(object);
    if cursor.IsNull() {
        return;
    }
    let mut count = 1u32;
    let mut container = cursor.ContainerFragmentIndex();
    loop {
        cursor.MoveToNextForSameLayoutObject();
        if cursor.IsNull() {
            break;
        }
        let next = cursor.ContainerFragmentIndex();
        if next != container {
            container = next;
            count += 1;
        }
    }
    let mut mutable = object.GetMutableForPainting();
    if mutable.FragmentList().size() == count {
        return;
    }
    mutable.FragmentCountChanged();
    while mutable.FragmentList().size() < count {
        mutable.FragmentList().AppendNewFragment();
    }
    if mutable.FragmentList().size() > count {
        mutable.FragmentList().Shrink(count);
    }
    mutable.SetOnlyThisNeedsPaintPropertyUpdate();
}

/// Bring a physical fragment's sidecar up to date for the current
/// publication. `retained`: the slot existed and was last exported with
/// compatible document resources; `same_owner`: it was last bound to this
/// immutable fragment (a LayoutNG cache hit); `display_item_fragment`: the
/// scope the reaching walk carries for it (PaintLayerPainter /
/// BoxFragmentPainter ScopedDisplayItemFragment), when known.
#[inline(never)]
pub(crate) fn InitializePhysicalFragmentPaintData(
    fragment: &PhysicalFragment,
    output: &mut FragmentPaintData,
    retained: bool,
    same_owner: bool,
    display_item_fragment: Option<u32>,
    resources: Option<&Arc<PaintResources>>,
    exported_styles: Option<crate::internal::layout_input::ExportedStyleCacheKey>,
) {
    let object = fragment.GetLayoutObject();
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
    let kept = InitializeProperties(
        object,
        fragment_data,
        effective_style,
        retained,
        exported_styles,
        output,
    );
    // cpp: core/paint/box_fragment_painter.cc:398-403,1211-1212
    // cpp: core/paint/paint_layer_painter.cc:776-778
    if let Some(display_item_fragment) = display_item_fragment {
        update_paint!(output.paint, display_item_fragment, display_item_fragment);
    }
    if !fragment.IsBox() {
        return;
    }
    let box_fragment = unsafe { &*To::<PhysicalBoxFragment>(fragment as *const PhysicalFragment) };
    // Geometry projected solely from an unchanged immutable fragment (and
    // the unchanged style and resources proven above) is still current.
    if !(kept && same_owner) {
        ExportDetachedNinePieceImages(object, effective_style, resources, output);
        ExportBoxPaintGeometry(box_fragment, &mut output.paint);
        RecordRebindScrollOrigin(box_fragment, output);
    }
    // ViewEstablishesPaintState reads this only for SVG foreignObject / text
    // boxes: such a child of an SVG root is reached once per root fragment
    // (FragmentChildren's SVG child expansion) and establishes the paint
    // state only when that occurrence is unique.
    if let Some(owner) = unsafe { object.as_ref() } {
        if owner.IsSVGForeignObject() || owner.IsSVGText() {
            let parent = DynamicTo::<LayoutBox>(owner.Parent() as *const LayoutObject);
            let establishes = unsafe { parent.as_ref() }
                .is_none_or(|parent| parent.PhysicalFragmentCount() <= 1);
            update_paint!(output.paint, establishes_paint_state, establishes);
        }
    }
}

/// Bring the inline item `items.Items()[index]`'s sidecar up to date for the
/// current publication (see InitializePhysicalFragmentPaintData).
#[inline(never)]
pub(crate) fn InitializeInlineItemPaintData(
    items: &layout_fragment::fragment_items::FragmentItems,
    index: usize,
    output: &mut FragmentPaintData,
    retained: bool,
    same_owner: bool,
    resources: Option<&Arc<PaintResources>>,
    exported_styles: Option<crate::internal::layout_input::ExportedStyleCacheKey>,
) {
    let item = &items.Items()[index];
    let object = item.GetLayoutObject();
    let effective_style = if item.UsesFirstLineStyle() {
        if item.Type() == ItemType::kLine && !object.is_null() {
            unsafe { &*object }.FirstLineStyleRef() as *const StyleComputedStyle
        } else {
            item.Style() as *const StyleComputedStyle
        }
    } else {
        std::ptr::null()
    };
    let kept = InitializeProperties(
        object,
        std::ptr::null_mut(),
        effective_style,
        retained,
        exported_styles,
        output,
    );
    // Offsets, sizes, text ranges, kinds, node ids and display-item
    // fragments of inline items are read live from the resident item
    // (ResidentBoxGeometry); only an atomic inline's box facts remain.
    if item.Type() != ItemType::kBox || (kept && same_owner) {
        return;
    }
    let Some(box_fragment) = (unsafe { item.BoxFragment().as_ref() }) else {
        return;
    };
    ExportDetachedNinePieceImages(object, effective_style, resources, output);
    ExportBoxPaintGeometry(box_fragment, &mut output.paint);
    RecordRebindScrollOrigin(box_fragment, output);
}
