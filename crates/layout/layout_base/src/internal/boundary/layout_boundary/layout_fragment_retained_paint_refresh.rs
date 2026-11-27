//! Fragment-owned refresh of retained paint data after a paint-only input update.
//!
//! Included by the source layout boundary so reciprocal translated types remain
//! available while sampling symbols resolve to `layout_fragment`.

use super::*;
use crate::fragment_tree::ResidentBoxGeometry as _;

/// Refresh local paint inputs of the restyled source objects, retaining layout
/// geometry, shaping, physical fragment identities and resource ownership.
/// Special geometry clients retain ordinary export until its proof is available.
///
/// Source: core/layout/layout_object.cc:2970-3120 (LayoutObject::SetStyle /
/// StyleDidChange). Chromium updates only the restyled object: its
/// FragmentData is reached from the object itself and painters read the new
/// ComputedStyle through it. Each changed object's bound occurrence sidecars
/// are therefore visited directly from its FragmentData list; no document
/// walk is involved.
#[inline(never)]
pub(super) fn refresh_retained_paint_inputs(
    tree: &crate::internal::layout_object_builder::LayoutObjectTree,
    fragments: &crate::layout_result_handle::LayoutResultHandle,
    changed: &HashSet<u64>,
    inline_ink_changes: &HashSet<u64>,
) -> bool {
    let Some(state) = fragments.PaintState::<crate::layout_engine::LayoutPaintState>() else {
        return false;
    };
    let mut objects = Vec::new();
    for &id in changed {
        for object in tree.InputObjects(id) {
            if !object.is_null() && !objects.contains(&object) {
                objects.push(object);
            }
        }
    }
    for &object in &objects {
        let client_id = unsafe { &*(object as *const DisplayItemClient) }.Id() as u64;
        let mut special = false;
        let visited = state.VisitBoundObjectPaintData(unsafe { &*object }, &mut |data| {
            if data.paint.DisplayItemClientId() == client_id
                && (data.ViewHasCollapsedBorders()
                    || data.ViewSvgText().is_some()
                    || data.paint.IsTextCombine())
            {
                special = true;
            }
        });
        if !visited || special {
            return false;
        }
    }
    // Source: core/layout/layout_object.cc:2289-2302 (InvalidateVisualOverflow,
    // reached from LayoutObject::SetStyle when
    // StyleDifference::NeedsRecomputeVisualOverflow()). A decoration change
    // keeps shaping resident: invalidate the text's FragmentItem ink overflow
    // and mark its painting layer. PaintLayer::UpdateDescendantDependentFlags
    // (paint_layer.cc:386-514), run immediately before PrePaint, recomputes it
    // through RecalcVisualOverflow and FragmentItem::RecalcInkOverflow
    // (fragment_item.cc:946-1078) before any painter reads it.
    for &id in inline_ink_changes {
        for object in tree.InputObjects(id) {
            if !object.is_null() {
                unsafe { &mut *object }.InvalidateVisualOverflow();
            }
        }
    }
    let mut retained_style_trace = layout_commit::fragment_trace::span(
        layout_commit::fragment_trace::TracePhase::RefreshRetainedPaintStyle,
    );
    retained_style_trace.set("objects", objects.len() as f64);
    for &object in &objects {
        let client_id = unsafe { &*(object as *const DisplayItemClient) }.Id() as u64;
        // Only the style-derived slice changes on a paint-only restyle;
        // the rest of the sidecar is the retained FragmentData.
        let current = ExportRetainedPaintStyle(object);
        let live_style_facts = current.live_style_facts;
        let invalidation = ReadyPrePaintInvalidation(object);
        state.VisitBoundObjectPaintData(unsafe { &*object }, &mut |node| {
            if node.paint.DisplayItemClientId() != client_id {
                return;
            }
            node.paint.background_is_known_to_be_obscured = false;
            node.paint.BindSourceObject(object);
            node.pre_paint_invalidation = invalidation;
            node.paint.style = current.style.clone();
            // FragmentData identity is owned by this retained physical
            // fragment. ExportPaintProperties has no physical-fragment
            // context and therefore reports the LayoutObject's first
            // FragmentData; do not collapse fragmented boxes back to it.
            node.paint.BindLiveStyleFacts(live_style_facts);
            node.paint.background_fill_layer_count = current.background_fill_layer_count;
            node.paint.has_background = current.has_background;
            node.paint.background_any_local_attachment_image =
                current.background_any_local_attachment_image;
            node.paint.background_any_default_attachment_image =
                current.background_any_default_attachment_image;
            node.paint.background_any_layer_uses_content_box =
                current.background_any_layer_uses_content_box;
        });
    }
    true
}

/// Make the style a deferred transform update reads current on every
/// sidecar of `object`'s FragmentData.
///
/// Source: paint_property_tree_builder.cc:4842-4888
/// (PaintPropertyTreeBuilder::DirectlyUpdateTransformMatrix reads
/// `object.StyleRef()`). PaintLayer::StyleDidChange deferred this object's
/// transform change (paint_layer.cc:2463-2473), so no PrePaint visit rebinds
/// its sidecar through the current publication: after a Layout publication
/// the sidecar keeps an earlier publication's binding, which the paint-only
/// refresh above skips, and the retained PrePaint node would keep resolving
/// the matrix from the previous ComputedStyle.
pub(super) fn refresh_deferred_transform_paint_style(
    state: &crate::layout_engine::LayoutPaintState,
    object: &LayoutObject,
) {
    let client_id = unsafe { &*(object as *const LayoutObject as *const DisplayItemClient) }.Id() as u64;
    let current = ExportRetainedPaintStyle(object);
    let live_style_facts = current.live_style_facts;
    state.VisitObjectPaintData(object, &mut |node| {
        if node.paint.DisplayItemClientId() != client_id {
            return;
        }
        node.paint.BindSourceObject(object);
        node.paint.style = current.style.clone();
        node.paint.BindLiveStyleFacts(live_style_facts);
    });
}
