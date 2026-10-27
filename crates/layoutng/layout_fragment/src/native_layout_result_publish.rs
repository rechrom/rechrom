//! Fragment-owned publication of a native LayoutResult paint snapshot.
//!
//! Included at the reciprocal native boundary. The physical source and
//! out-of-line symbol keep publication self time attributable to
//! `layout_fragment` while translated helper types remain available.

use super::*;

#[inline(never)]
pub(super) fn populate_layout_result_paint_data(
    root: &mut LayoutObject,
    result: &LayoutResult,
    space: &ConstraintSpace,
    purpose: LayoutUpdateKind,
    cache: &mut LayoutBoundaryCache,
) -> LayoutResultHandle {
    let mut trace = layout_fragment_boundary::trace::span(
        layout_fragment_boundary::trace::TracePhase::PopulateLayoutResultPaintData,
    );
    trace.set("cache_warm", (!cache.objects.is_empty()) as u8 as f64);
    let result_identity = result as *const LayoutResult as usize;
    let result_changed = cache.fragment_identity_result != result_identity;
    let prior_fragment_instance_ids = if result_changed {
        cache.fragment_identity_result = result_identity;
        std::mem::take(&mut cache.fragment_instance_ids)
    } else {
        LayoutBoundaryHashMap::default()
    };
    if cache.objects.is_empty() {
        let _phase = layout_fragment_boundary::trace::span(
            layout_fragment_boundary::trace::TracePhase::IndexObjects,
        );
        EnsureLayoutBoundaryObjectIndex(root, cache);
    }
    let objects = &cache.objects;
    let sources = &cache.sources;
    let style_sources = &cache.style_sources;
    trace.set("objects", objects.len() as f64);
    trace.set("sources", sources.len() as f64);
    {
        let _phase = layout_fragment_boundary::trace::span(
            layout_fragment_boundary::trace::TracePhase::UpdateStickyOffsets,
        );
        UpdateStickyOffsets(objects.iter().copied(), false);
    }
    let resources = Arc::new(PaintResources {
        fonts: if purpose == LayoutUpdateKind::Paint {
            space.fonts.clone()
        } else {
            Vec::new().into()
        },
        images: if purpose == LayoutUpdateKind::Paint {
            space.images.clone()
        } else {
            Vec::new()
        },
        device_pixel_ratio: space.device_pixel_ratio,
        preferred_color_scheme: space.preferred_color_scheme,
        layout_zoom_factor: space.layout_zoom_factor,
        paints_opaque_base_background: !space.printing,
        viewport: space.viewport,
    });
    trace.set("retained_candidates", 0.0);
    let mut paint_state_owners = FragmentOwners {
        exported_styles: std::mem::take(&mut cache.styles),
        debug_names: std::mem::take(&mut cache.debug_names),
        uncullable_objects: Arc::clone(&cache.uncullable_objects),
        fragment_instance_ids: std::mem::take(&mut cache.fragment_instance_ids),
        prior_fragment_instance_ids,
        ..FragmentOwners::default()
    };
    let mut next_fragment_instance_id = cache.next_fragment_instance_id.max(1);
    let mut next_fragmentainer_instance_id = 1;
    let paint_cull = {
        let _phase = layout_fragment_boundary::trace::span(
            layout_fragment_boundary::trace::TracePhase::BuildPaintCullRect,
        );
        (purpose == LayoutUpdateKind::Paint)
            .then(|| BuildPaintCullRect(root, space))
            .flatten()
    };
    trace.set("culled", paint_cull.is_some() as u8 as f64);
    {
        let _phase = layout_fragment_boundary::trace::span(
            layout_fragment_boundary::trace::TracePhase::PopulateFragmentPaintData,
        );
        PopulatePhysicalFragmentPaintData(
            result.GetPhysicalFragment(),
            PhysicalOffset::default(),
            PhysicalOffset::default(),
            paint_cull,
            true,
            sources,
            style_sources,
            &resources,
            purpose,
            &mut paint_state_owners,
            0,
            0,
            &mut next_fragment_instance_id,
            &mut next_fragmentainer_instance_id,
        )
    }
    let native =
        crate::layout_engine::NativePaintFragmentCursor::FromRoot(result.GetPhysicalFragment());
    {
        let _phase = layout_fragment_boundary::trace::span(
            layout_fragment_boundary::trace::TracePhase::ResolveLogicalParents,
        );
        ResolveNativeLogicalParentOccurrences(native);
    }
    if purpose == LayoutUpdateKind::Paint {
        let _phase = layout_fragment_boundary::trace::span(
            layout_fragment_boundary::trace::TracePhase::PopulateGlobalPaintData,
        );
        ExportCanvasBackgroundNative(root, objects, native);
        ExportGlobalMainThreadScrollFactsNative(root, objects, native);
        PopulateNativeInlineStitchedDecorations(native);
    }
    cache.styles = paint_state_owners.exported_styles;
    cache.debug_names = paint_state_owners.debug_names;
    cache.fragment_instance_ids = paint_state_owners.fragment_instance_ids;
    cache.next_fragment_instance_id = next_fragment_instance_id;
    unsafe { LayoutResultHandle::FromNative(result) }
}
