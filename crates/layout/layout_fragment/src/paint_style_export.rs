//! Layout's `PaintStyle` publication over the ComputedStyle -> paint-value
//! projection owned by `layout_style_paint`. Only the wrappers that build Layout's
//! retained `PaintStyle` or read Style resource bindings stay here; the pure
//! projection items are re-exported at their established path.

#![allow(non_snake_case)]

use std::sync::Arc;

use layout_style::style::computed_style::ComputedStyle;

pub use layout_style_paint::paint_style_export::*;

use crate::internal::paint_input::{
    PaintFilterOperation, PaintFilterType, PaintStyleData, PaintTransform,
};

#[inline(never)]
pub(crate) fn ExportGeometryStyle(
    style: &ComputedStyle,
) -> crate::internal::layout_input::PaintStyle {
    let mut data = PaintStyleData::default();
    ExportGeometryProperties(
        style,
        &mut data,
        !foundation::IsManagedLayoutAddress(style as *const _),
    );
    crate::internal::layout_input::PaintStyle::FromSource(style, data)
}

/// Retain the canonical Chromium ComputedStyle across Layout -> Paint. The
/// accompanying data contains only values still awaiting direct-source
/// migration plus fragment/resource results.
#[inline(never)]
pub(crate) fn ExportStyle(
    style: &ComputedStyle,
    image_resource_id: Option<u64>,
) -> crate::internal::layout_input::PaintStyle {
    let copy_canonical_values = !foundation::IsManagedLayoutAddress(style as *const _);
    crate::internal::layout_input::PaintStyle::FromSource(
        style,
        ExportForPaint(style, image_resource_id, copy_canonical_values),
    )
}

pub(crate) fn ExportDetachedStyle(
    style: &ComputedStyle,
    image_resource_id: Option<u64>,
) -> crate::internal::layout_input::PaintStyle {
    crate::internal::layout_input::PaintStyle::FromData(Export(style, image_resource_id))
}

/// Preserve document-resolved SVG resources that cannot live in Layout's
/// source ComputedStyle object graph. URI identity still comes from `style`;
/// only the immutable server snapshot is carried from StyleEngine's existing
/// renderer-neutral projection.
#[inline(never)]
pub(crate) fn ExportStyleWithResourceBindings(
    style: &ComputedStyle,
    image_resource_id: Option<u64>,
    resources: &crate::style_projection::StyleResourceBindings<'_>,
    clip_resources: &crate::internal::paint_input::SvgClipResourceBinding,
) -> crate::internal::layout_input::PaintStyle {
    let mut output = ExportStyle(style, image_resource_id);
    output.svg_mask_resource = resources.svg_mask_resource.clone();
    output.svg_mask_bindings = MaskReferences(style);
    for reference in &mut output.svg_mask_bindings {
        let entry = (reference.scope.document_id == clip_resources.snapshot.version.document_id)
            .then(|| {
                clip_resources.snapshot.entries.iter().find(|entry| {
                    entry.scope_id == reference.scope.tree_scope_id && entry.id == reference.id
                })
            })
            .flatten();
        if let Some(graph) = entry.and_then(|entry| entry.mask_graph.as_ref()) {
            reference.graph = Some(Arc::clone(graph));
            reference.target_id = entry.map(|entry| entry.target_node_id);
        }
    }
    output.svg_fill_server = resources.svg_fill_server.clone();
    output.svg_stroke_server = resources.svg_stroke_server.clone();
    output.svg_markers = resources.svg_markers.to_vec();
    output.replaced_image_shader = resources.replaced_image_shader.clone();
    output.svg_filter_graphs = Arc::clone(resources.svg_filter_graphs);
    if let Some(clip) = output.clip_path.as_mut() {
        if let Some(url) = clip.reference_url.as_deref() {
            if let Some(id) = url.strip_prefix('#') {
                let scope = clip.reference_scope.unwrap_or(clip_resources.scope);
                let entry =
                    (scope.document_id == clip_resources.snapshot.version.document_id)
                        .then(|| {
                            clip_resources.snapshot.entries.iter().find(|entry| {
                                entry.scope_id == scope.tree_scope_id && entry.id == id
                            })
                        })
                        .flatten();
                if let Some(graph) = entry.and_then(|entry| entry.graph.as_ref()) {
                    clip.reference_target_id = entry.map(|entry| entry.target_node_id);
                    clip.reference_graph = Some(Arc::clone(graph));
                    clip.reference_error = graph.unsupported.clone();
                } else {
                    // cpp: ClipPathClipper::PathBasedClipInternal. A missing
                    // or wrong-kind resource supplies no clip. A present,
                    // empty graph above remains a real empty clip instead.
                    output.clip_path = None;
                }
            } else {
                // External fetch/loading ownership is not represented by this
                // local resource snapshot. Keep it explicitly unsupported.
                clip.reference_error = Some("external clip-path IRI".into());
            }
        }
    }
    let fill_paint = output.svg_fill;
    let stroke_paint = output.svg_stroke;
    let graphs = Arc::clone(resources.svg_filter_graphs);
    // Both CSS filter properties use the same document-resolved SVG resource
    // graph.  Keeping only the foreground projection turns a valid local
    // backdrop-filter url() into an unprojected marker at the Paint boundary,
    // even though Style already produced the canonical graph.
    let project_filters = |filters: &mut Vec<PaintFilterOperation>| {
        for filter in filters {
            if filter.r#type != PaintFilterType::kReference {
                continue;
            }
            let Some(id) = filter
                .reference_url
                .as_deref()
                .and_then(|url| url.strip_prefix('#'))
            else {
                continue;
            };
            if let Some(graph) = graphs.iter().find(|graph| graph.id == id) {
                let mut graph = graph.clone();
                graph.fill_paint = fill_paint;
                graph.stroke_paint = stroke_paint;
                filter.reference_error = graph.unsupported.clone();
                filter.reference_graph = Some(Arc::new(graph));
            }
        }
    };
    if output.Source().is_none() {
        project_filters(&mut output.filters);
        project_filters(&mut output.backdrop_filters);
        output.backdrop_filter_has_reference = output.backdrop_filters.iter().any(|filter| {
            filter.r#type == PaintFilterType::kReference && filter.reference_graph.is_none()
        });
    }
    output
}

#[inline(never)]
pub(crate) fn ExportStyleWithInputResources(
    style: &ComputedStyle,
    resources: &crate::internal::layout_input::StyleResourceData,
) -> crate::internal::layout_input::PaintStyle {
    let bindings = crate::style_projection::StyleResourceBindings::FromData(resources);
    ExportStyleWithResourceBindings(
        style,
        resources.image_resource_id,
        &bindings,
        &resources.svg_clip_resources,
    )
}

/// Motion-path is resolved against the actual containing LayoutBox, so its
/// final matrix cannot be reconstructed from ComputedStyle alone at the
/// immutable Layout -> Paint boundary. Preserve only UpdateOffset's matrix;
/// independent and legacy transform properties keep their own slots.
#[inline(never)]
pub(crate) fn SetResolvedOffsetTransform(
    output: &mut crate::internal::layout_input::PaintStyle,
    matrix: foundation::gfx::Transform,
) {
    let mut transform = PaintTransform::default();
    PushMatrix(&mut transform, matrix);
    output.offset_transform = Some(transform);
}
