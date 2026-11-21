//! Fragment-owned refresh of retained paint data after a paint-only input update.
//!
//! Included by the source layout boundary so reciprocal translated types remain
//! available while sampling symbols resolve to `layout_fragment`.

use super::*;
use crate::fragment_tree::ResidentBoxGeometry as _;

/// Refresh local paint inputs from existing source objects, retaining layout
/// geometry, shaping, physical fragment identities and resource ownership.
/// Special geometry clients retain ordinary export until its proof is available.
#[inline(never)]
pub(super) fn refresh_retained_paint_inputs(
    root: &mut LayoutObject,
    fragments: crate::layout_engine::PaintFragmentCursor<'_>,
    changed: &HashSet<u64>,
    inline_ink_changes: &HashSet<u64>,
) -> bool {
    let mut sources = SourceMap::default();
    let mut style_sources = StyleSourceMap::default();
    let mut clients = HashMap::new();
    let mut objects = Vec::new();
    let root_ptr: *mut LayoutObject = root;
    let mut object = root_ptr;
    let mut logical_order = 0;
    while !object.is_null() {
        objects.push(object);
        let current = unsafe { &*object };
        let node = current.GetNode();
        if !node.is_null() {
            logical_order += 1;
            sources.insert(
                object as *const _,
                SourceEntry {
                    node,
                    logical_tree_order: logical_order,
                },
            );
            style_sources.insert(
                current.StyleRef() as *const _,
                StyleSourceEntry {
                    resources: unsafe { &*node }.InputStyleResources(),
                },
            );
            if changed.contains(&unsafe { &*node }.InputId()) {
                clients.insert(
                    unsafe { &*(object as *const DisplayItemClient) }.Id() as u64,
                    object,
                );
            }
        }
        object = current.NextInPreOrder(root_ptr);
    }
    fn validate(
        node: crate::layout_engine::PaintFragmentCursor<'_>,
        clients: &HashMap<u64, *mut LayoutObject>,
    ) -> bool {
        if clients.contains_key(&node.paint.DisplayItemClientId())
            && (node.ViewHasCollapsedBorders()
                || node.ViewSvgText().is_some()
                || node.paint.IsTextCombine())
        {
            return false;
        }
        node.Children()
            .into_iter()
            .all(|child| validate(child, clients))
    }
    if !validate(fragments, &clients) {
        return false;
    }
    // Source: core/layout/layout_object.cc:2289-2302 (InvalidateVisualOverflow,
    // reached from LayoutObject::SetStyle when
    // StyleDifference::NeedsRecomputeVisualOverflow()). A decoration change
    // keeps shaping resident: invalidate the text's FragmentItem ink overflow
    // and mark its painting layer. PaintLayer::UpdateDescendantDependentFlags
    // (paint_layer.cc:386-514), run immediately before PrePaint, recomputes it
    // through RecalcVisualOverflow and FragmentItem::RecalcInkOverflow
    // (fragment_item.cc:946-1078) before any painter reads it.
    for &object in &objects {
        if inline_ink_changes.contains(&InputId(unsafe { &*object })) {
            unsafe { &mut *object }.InvalidateVisualOverflow();
        }
    }
    unsafe fn refresh(
        cursor: crate::layout_engine::PaintFragmentCursor<'_>,
        clients: &HashMap<u64, *mut LayoutObject>,
        sources: &SourceMap,
        styles: &StyleSourceMap,
    ) {
        let children = cursor.Children();
        let node = unsafe { cursor.PaintDataMut() };
        node.paint.background_is_known_to_be_obscured = false;
        node.paint.page_document_background_style = None;
        node.paint.view_background_invalidation = None;
        if let Some(&object) = clients.get(&node.paint.DisplayItemClientId()) {
            let resources = cursor
                .DocumentResources()
                .expect("completed fragment has document resources");
            let current =
                ExportPaintProperties(object, sources, styles, std::ptr::null(), resources);
            let live_style_facts = current.LiveStyleFacts();
            node.paint.BindSourceObject(object);
            // ExportPaintProperties has made the object ready for invalidation.
            node.pre_paint_invalidation = ExportPrePaintInvalidation(object);
            node.paint.style = current.style;
            // FragmentData identity is owned by this retained physical
            // fragment. ExportPaintProperties has no physical-fragment
            // context and therefore reports the LayoutObject's first
            // FragmentData; do not collapse fragmented boxes back to it.
            node.paint.BindLiveStyleFacts(live_style_facts);
            node.paint.background_fill_layer_count =
                current.background_fill_layer_count;
            node.paint.has_background = current.has_background;
            node.paint.background_any_local_attachment_image =
                current.background_any_local_attachment_image;
            node.paint.background_any_default_attachment_image =
                current.background_any_default_attachment_image;
            node.paint.background_any_layer_uses_content_box =
                current.background_any_layer_uses_content_box;
        }
        for child in children {
            unsafe { refresh(child, clients, sources, styles) };
        }
    }
    unsafe { refresh(fragments, &clients, &sources, &style_sources) };
    ExportCanvasBackground(root, fragments);
    ExportGlobalMainThreadScrollFacts(root, &objects, fragments);
    true
}
