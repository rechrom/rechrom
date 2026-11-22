//! Fragment-owned publication of a source LayoutResult to Paint.
//!
//! Included at the reciprocal source boundary. The physical source and
//! out-of-line symbol keep publication self time attributable to
//! `layout_fragment` while translated helper types remain available.
//!
//! Publication does not walk the fragment tree. Chromium keeps FragmentData
//! on the persistent LayoutObjects and PrePaintTreeWalk brings each reached
//! object's FragmentData up to date (core/paint/pre_paint_tree_walk.cc);
//! LayoutPaintState likewise initializes a fragment's sidecar the first time
//! a cursor of this publication reaches it.

use super::*;

#[inline(never)]
pub(super) fn install_layout_result_paint_state(
    root: &mut LayoutObject,
    result: &LayoutResult,
    space: &ConstraintSpace,
    purpose: LayoutUpdateKind,
    cache: &mut LayoutBoundaryCache,
) -> LayoutResultHandle {
    let mut trace = layout_fragment_boundary::trace::span(
        layout_fragment_boundary::trace::TracePhase::InstallLayoutResultPaintState,
    );
    trace.set("cache_warm", (!cache.objects.is_empty()) as u8 as f64);
    let result_identity = result as *const LayoutResult as usize;
    let result_changed = cache.fragment_identity_result != result_identity;
    // A retained update of the same LayoutResult republishes the same state.
    // A replacement LayoutResult gets its own state, whose lease retains that
    // result for the handles Paint keeps; the FragmentPaintData sidecars
    // themselves stay on their LayoutObjects' FragmentData across both, as
    // Chromium's FragmentData does (core/paint/fragment_data.h), so nothing
    // is seeded or moved between the states.
    let retained_paint_state = !result_changed && cache.fragment_paint_state.is_some();
    // A topology mutation cleared the object index.
    let objects_rebuilt = cache.objects.is_empty();
    if objects_rebuilt {
        let _phase = layout_fragment_boundary::trace::span(
            layout_fragment_boundary::trace::TracePhase::IndexObjects,
        );
        EnsureLayoutBoundaryObjectIndex(root, cache);
    }
    let paint_state = if result_changed {
        let state = crate::layout_engine::LayoutPaintState::New(result);
        if let Some(previous) = &cache.fragment_paint_state {
            trace.set("previous_paint_state_refs", Arc::strong_count(previous) as f64);
            if purpose == LayoutUpdateKind::Paint {
                state.InheritFrom(previous);
            }
            // Handles of the previous result stay readable; the shared
            // sidecars now bind to this result's owners.
            previous.Supersede();
        }
        state
    } else {
        cache
            .fragment_paint_state
            .clone()
            .unwrap_or_else(|| crate::layout_engine::LayoutPaintState::New(result))
    };
    paint_state.BeginPublication();
    if result_changed {
        cache.fragment_identity_result = result_identity;
    }
    EnsureBoundaryStyleIndices(cache);
    let objects = &cache.objects;
    trace.set("objects", objects.len() as f64);
    // Sticky constraints and offsets are not refreshed here. Like Chromium's
    // LocalFrameView::ExecutePendingStickyUpdates, the layout lifecycle
    // updates only the queued scroll containers' sticky descendants once
    // after layout (sticky_layout_commit.rs); publication reads them.
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
        // Shared catalogs: publication takes a reference, never a copy.
        images: if purpose == LayoutUpdateKind::Paint {
            Arc::clone(&space.images)
        } else {
            Default::default()
        },
        canvases: if purpose == LayoutUpdateKind::Paint {
            Arc::clone(&space.canvases)
        } else {
            Default::default()
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
    // The resident sidecars were last exported with the document resources the
    // state holds (its previous publication or a later Page resource
    // update). Sidecars bound after an incompatible change re-export;
    // the values are the Document / LocalFrameView facts Chromium reads,
    // held once per result rather than on every fragment sidecar.
    let resources_compatible =
        PaintResourcesAreCompatible(paint_state.DocumentResources(), &resources);
    paint_state.InstallPublication(resources, resources_compatible, Some(cache.styles.Key()));
    // The same LayoutResult republishes the same immutable fragments, so no
    // live identity can appear or disappear.
    let topology_retained = retained_paint_state;
    paint_state.FinishPublication(topology_retained);
    trace.set("topology_retained", topology_retained as u8 as f64);
    cache.fragment_paint_state = Some(paint_state);
    handle
}
