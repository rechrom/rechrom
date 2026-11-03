// The no-layout scroll fast path mutates only retained fragment paint state.
// Keep index refresh, full-walk fallback and index reconstruction together in
// a fragment-owned sampling frame; relayout fallback remains core orchestration.

use super::*;

pub(super) struct RetainedScrollRefresh {
    pub(super) update: ScrollLayoutUpdate,
    pub(super) indexed_refresh: Option<bool>,
}

#[inline(never)]
pub(super) fn refresh_retained_scroll_geometry(
    engine: &mut LayoutEngine,
    root: *mut LayoutObject,
    fragments: &mut LayoutResultHandle,
    refresh_cull_rect: bool,
) -> Option<RetainedScrollRefresh> {
    if refresh_cull_rect {
        if !engine.RefreshScrollCullData(fragments) {
            return None;
        }
        return Some(RetainedScrollRefresh {
            update: ScrollLayoutUpdate::RefreshedLayout,
            indexed_refresh: None,
        });
    }

    let mut heap = LayoutHeapScope::new();
    let _pass = LayoutPassScope::new(&engine.algorithms_, &engine.objects_);
    let refreshed = engine
        .scroll_paint_index_
        .as_ref()
        .is_some_and(|index| index.refresh(unsafe { &mut *root }, &engine.pending_scroll_ids_));
    PrepareLayoutHeapForPointerQueries();
    if !refreshed {
        let assembly = LayoutAssembly {
            algorithms: engine.algorithms_,
            objects: engine.objects_,
            boundary: engine.boundary_,
        };
        assembly.RefreshScrollPaintProperties(unsafe { &mut *root }, fragments, true);
        let reuse = engine
            .scroll_paint_index_
            .take()
            .or(engine.scroll_paint_index_spare_.take());
        engine.scroll_paint_index_ = Some(ScrollPaintPropertyIndex::new(
            unsafe { &mut *root },
            fragments,
            engine.CachedScrollIndexObjects(),
            reuse,
        ));
    }
    engine.pending_scroll_ids_.clear();
    heap.DeferCollection();
    Some(RetainedScrollRefresh {
        update: ScrollLayoutUpdate::RetainedGeometry,
        indexed_refresh: Some(refreshed),
    })
}
