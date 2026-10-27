//! Fragment-owned refresh of resident native boundary indices.
//!
//! Included by the native layout boundary. Keeping both the style-source scan
//! and derived sticky/uncullable indices here makes targeted update cache
//! maintenance visible as fragment work in traces and sampling profiles.

use super::*;

/// A compatible Element::RecalcOwnStyle update preserves LayoutObject and
/// metadata identities. Keep the topology-dependent native indices resident
/// while dropping style values derived from changed objects. Blink keeps
/// these identities in one object graph across SetStyle;
/// this is the boundary equivalent for Rechrom's fragment-local Paint data.
#[inline(never)]
pub(super) fn refresh_compatible_node_input_cache(
    cache: &mut LayoutBoundaryCache,
    update: &NodeInputUpdate,
) {
    if cache.objects.is_empty() || update.objects.is_empty() {
        return;
    }

    refresh_compatible_paint_cache(cache);
}

/// A paint-only SetStyle keeps LayoutObject and Node identities resident.
/// Discard values keyed by the old style pointers, then rebuild the small
/// style-derived indices without throwing away the document-order object
/// index used by scroll paint-property and cull-rect updates.
#[inline(never)]
pub(super) fn refresh_compatible_paint_cache(cache: &mut LayoutBoundaryCache) {
    let mut trace = layout_fragment_boundary::trace::span(
        layout_fragment_boundary::trace::TracePhase::RefreshBoundaryCache,
    );
    trace.set("objects", cache.objects.len() as f64);
    trace.set("sources", cache.source_objects.len() as f64);
    if cache.objects.is_empty() {
        return;
    }

    cache.styles.clear();

    // A native ComputedStyle can be shared by several source nodes. Rebuild
    // this small pointer index in document order so removing one changed source
    // cannot orphan another source which still owns the old native style.
    cache.style_sources.clear();
    for &object in cache.source_objects.iter() {
        let current = unsafe { &*object };
        let node = current.GetNode();
        if node.is_null() {
            continue;
        }
        let input_node = unsafe { &*node };
        cache
            .style_sources
            .entry(current.StyleRef())
            .or_insert(input_node.InputStyle());
        let first_line_style = input_node.InputFirstLineStyle();
        if !first_line_style.is_null() {
            if let Some(first_line_data) = input_node.InputFirstLineStyleData() {
                cache
                    .style_sources
                    .entry(first_line_style)
                    .or_insert(first_line_data.as_ref());
            }
        }
    }

    cache.sticky_objects = Arc::new(
        cache
            .objects
            .iter()
            .copied()
            .filter(|object| {
                unsafe { &**object }
                    .StyleRef()
                    .HasStickyConstrainedPosition()
            })
            .collect(),
    );
    cache.uncullable_objects = Arc::new(MarkUncullablePaintAncestors(&cache.objects));
}
