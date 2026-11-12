//! Fragment-owned publication of a source LayoutResult paint snapshot.
//!
//! Included at the reciprocal source boundary. The physical source and
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
    // A retained update of the same LayoutResult mutates Chromium's canonical
    // FragmentData sidecars in place (notably for scroll paint properties).
    // A replacement LayoutResult, however, is a new immutable publication:
    // handles retained by Paint/tests must keep their previous owner bindings
    // and values. Reusing the cache Arc here let BeginPublication clear the old
    // handle's owner map before the replacement was committed.
    let retained_paint_state = !result_changed && cache.fragment_paint_state.is_some();
    let paint_state = if result_changed {
        crate::layout_engine::LayoutPaintState::New()
    } else {
        cache
            .fragment_paint_state
            .clone()
            .unwrap_or_else(crate::layout_engine::LayoutPaintState::New)
    };
    paint_state.BeginPublication();
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
    // Isolated SVG image documents paint into an image record whose canvas is
    // transparent. They must not inherit the ordinary page FrameView's opaque
    // white base backdrop; the outer page supplies whatever is behind the
    // image when the record is replayed.
    let is_document_image = space.document_image_container.is_some();
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
        canvases: if purpose == LayoutUpdateKind::Paint {
            space.canvases.clone()
        } else {
            Vec::new()
        },
        device_pixel_ratio: space.device_pixel_ratio,
        preferred_color_scheme: space.preferred_color_scheme,
        layout_zoom_factor: space.layout_zoom_factor,
        base_background_color: foundation::PaintColor {
            red: 1.0,
            green: 1.0,
            blue: 1.0,
            alpha: if is_document_image { 0.0 } else { 1.0 },
        },
        paints_opaque_base_background: !is_document_image,
        should_clear_document_background: !is_document_image,
        viewport: space.viewport,
    });
    let handle =
        unsafe { LayoutResultHandle::FromLayoutResultWithPaintState(result, paint_state.clone()) };
    let mut paint_state_owners = FragmentOwners {
        exported_styles: std::mem::take(&mut cache.styles),
        debug_names: std::mem::take(&mut cache.debug_names),
        fragment_instance_ids: std::mem::take(&mut cache.fragment_instance_ids),
        prior_fragment_instance_ids,
        ..FragmentOwners::default()
    };
    let mut next_fragment_instance_id = cache.next_fragment_instance_id.max(1);
    let mut next_fragmentainer_instance_id = 1;
    if purpose == LayoutUpdateKind::Paint {
        // Source: core/frame/local_frame_view.cc:3034 and
        // core/paint/paint_layer.cc:386-514. Chromium updates descendant
        // aggregates once from the root PaintLayer before PrePaint consumes
        // them. Doing this from every fragment repeats the same tree walk and
        // makes publication own a second layer lifecycle.
        let enclosing_layer = root.EnclosingLayer();
        if !enclosing_layer.is_null() {
            let root_layer = unsafe { &*enclosing_layer }.Root();
            if !root_layer.is_null() {
                unsafe { &mut *root_layer }.UpdateDescendantDependentFlags();
            }
        }
    }
    {
        let _phase = layout_fragment_boundary::trace::span(
            layout_fragment_boundary::trace::TracePhase::PopulateFragmentPaintData,
        );
        PopulatePhysicalFragmentPaintData(
            result.GetPhysicalFragment(),
            PhysicalOffset::default(),
            sources,
            style_sources,
            &resources,
            purpose,
            &paint_state,
            &mut paint_state_owners,
            0,
            0,
            &mut next_fragment_instance_id,
            &mut next_fragmentainer_instance_id,
        )
    }
    trace.set(
        "paint_property_export_ms",
        paint_state_owners.population_stats.property_export_ns as f64 / 1_000_000.0,
    );
    trace.set(
        "sidecar_store_ms",
        paint_state_owners.population_stats.sidecar_store_ns as f64 / 1_000_000.0,
    );
    trace.set(
        "physical_fragments",
        paint_state_owners.population_stats.physical_fragments as f64,
    );
    trace.set(
        "inline_items",
        paint_state_owners.population_stats.inline_items as f64,
    );
    let topology_retained = retained_paint_state
        && paint_state_owners.new_fragment_instances == 0
        && paint_state_owners.prior_fragment_instance_ids.is_empty();
    paint_state.FinishPublication(topology_retained);
    trace.set("topology_retained", topology_retained as u8 as f64);
    let source = crate::layout_engine::PaintFragmentCursor::FromHandle(&handle);
    {
        let _phase = layout_fragment_boundary::trace::span(
            layout_fragment_boundary::trace::TracePhase::ResolveLogicalParents,
        );
        ResolveLogicalParentOccurrences(source);
    }
    if purpose == LayoutUpdateKind::Paint {
        let _phase = layout_fragment_boundary::trace::span(
            layout_fragment_boundary::trace::TracePhase::PopulateGlobalPaintData,
        );
        ExportCanvasBackground(
            root,
            objects,
            source,
            space.should_apply_root_background_behavior,
        );
        ExportGlobalMainThreadScrollFacts(root, objects, source);
        PopulateInlineStitchedDecorations(source);
    }
    cache.styles = paint_state_owners.exported_styles;
    cache.debug_names = paint_state_owners.debug_names;
    cache.fragment_instance_ids = paint_state_owners.fragment_instance_ids;
    cache.next_fragment_instance_id = next_fragment_instance_id;
    cache.fragment_paint_state = Some(paint_state);
    handle
}
