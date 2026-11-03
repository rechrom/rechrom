// Paint publication and retained-paint refresh live with fragment
// ownership. LayoutEngine keeps only its stable public entry points.

use super::*;

pub(super) trait EnginePaintUpdate {
    fn populate_paint_data(&mut self) -> bool;
    fn refresh_targeted_paint_data(&mut self) -> bool;
    fn refresh_targeted_paint_inputs(&mut self) -> bool;
    fn refresh_scroll_cull_data(&mut self, fragments: &mut LayoutResultHandle) -> bool;
}

impl EnginePaintUpdate for LayoutEngine {
    #[inline(never)]
    fn populate_paint_data(&mut self) -> bool {
        let mut trace = layout_fragment_boundary::trace::span(
            layout_fragment_boundary::trace::TracePhase::PopulatePaintData,
        );
        if self.tree_.is_none() {
            trace.set("rejected", 1.0);
            return false;
        }
        let root = self.RootPointer();
        let root_box = DynamicTo::<LayoutBox>(root);
        if root_box.is_null()
            || unsafe { &*root }.NeedsLayout()
            || unsafe { &*root_box }.GetLayoutResults().is_empty()
            || unsafe { &*root_box }.GetLayoutResult(0).is_null()
        {
            trace.set("needs_layout", 1.0);
            return false;
        }
        let assembly = LayoutAssembly {
            algorithms: self.algorithms_,
            objects: self.objects_,
            boundary: self.boundary_,
        };
        let fragments = assembly.PublishLayoutResult(
            unsafe { &mut *root },
            &self.constraints_,
            &mut self.boundary_cache_,
        );
        let mut inputs = std::mem::take(&mut self.pre_paint_inputs_);
        {
            let mut trace = layout_fragment_boundary::trace::span(
                layout_fragment_boundary::trace::TracePhase::CertifyPaintInputs,
            );
            UpdatePrePaintInputs(&mut inputs, &fragments, &self.objects_);
            let stats = inputs.last_stats();
            trace.set("nodes", stats.nodes as f64);
            trace.set("new_nodes", stats.new_nodes as f64);
        }
        let reuse = self
            .scroll_paint_index_
            .take()
            .or(self.scroll_paint_index_spare_.take());
        self.scroll_paint_index_ = Some({
            let _pass = LayoutPassScope::new(&self.algorithms_, &self.objects_);
            ScrollPaintPropertyIndex::new(
                unsafe { &mut *root },
                &fragments,
                self.CachedScrollIndexObjects(),
                reuse,
            )
        });
        self.pending_scroll_ids_.clear();
        self.fragments = Some(fragments);
        self.output_kind = LayoutUpdateKind::Paint;
        self.pre_paint_inputs_ = inputs;
        trace.set("published", 1.0);
        true
    }

    #[inline(never)]
    fn refresh_targeted_paint_data(&mut self) -> bool {
        if self.output_kind != LayoutUpdateKind::Paint {
            return false;
        }
        let mut trace = layout_fragment_boundary::trace::span(
            layout_fragment_boundary::trace::TracePhase::RefreshTargetedPaintData,
        );
        let Some(fragments) = &mut self.fragments else {
            trace.set("rejected", 1.0);
            return false;
        };
        let root = self
            .tree_
            .as_ref()
            .map(LayoutObjectTree::Root)
            .expect("targeted paint update requires a layout tree");
        let root_box = DynamicTo::<LayoutBox>(root as *const _ as *mut LayoutObject);
        if root_box.is_null()
            || root.NeedsLayout()
            || unsafe { &*root_box }.GetLayoutResults().is_empty()
            || unsafe { &*root_box }.GetLayoutResult(0).is_null()
        {
            trace.set("needs_layout", 1.0);
            return false;
        }
        // Chromium updates FragmentData on the live LayoutResult owner. Keep
        // the same result-local sidecar and its stable per-fragment addresses.
        UpdatePrePaintInputs(&mut self.pre_paint_inputs_, fragments, &self.objects_);
        trace.set("published", 1.0);
        true
    }

    #[inline(never)]
    fn refresh_targeted_paint_inputs(&mut self) -> bool {
        if self.output_kind != LayoutUpdateKind::Paint {
            return false;
        }
        let mut trace = layout_fragment_boundary::trace::span(
            layout_fragment_boundary::trace::TracePhase::RefreshTargetedPaintInputs,
        );
        if self.tree_.is_none() {
            return false;
        }
        let root = self.RootPointer();
        if unsafe { &*root }.NeedsLayout() {
            return false;
        }
        let Some(fragments) = &mut self.fragments else {
            return false;
        };
        // Targeted updates mutate the canonical FragmentData sidecars in
        // place; publishing a copied snapshot would break retained identity.
        let _pass = LayoutPassScope::new(&self.algorithms_, &self.objects_);
        if !crate::internal::boundary::layout_boundary::RefreshRetainedPaintInputs(
            unsafe { &mut *root },
            PaintFragmentCursor::FromHandle(fragments),
            &self.pending_paint_ids_,
            &self.pending_inline_ink_ids_,
        ) {
            trace.set("rejected", 1.0);
            return false;
        }
        UpdatePrePaintInputs(&mut self.pre_paint_inputs_, fragments, &self.objects_);
        trace.set("updated_sources", self.pending_paint_ids_.len() as f64);
        self.pending_paint_ids_.clear();
        self.pending_inline_ink_ids_.clear();
        trace.set("published", 1.0);
        true
    }

    #[inline(never)]
    fn refresh_scroll_cull_data(&mut self, fragments: &mut LayoutResultHandle) -> bool {
        let mut trace = layout_fragment_boundary::trace::span(
            layout_fragment_boundary::trace::TracePhase::RefreshScrollCullData,
        );
        let Some(tree) = self.tree_.as_ref() else {
            return false;
        };
        if !tree.HasRoot() {
            return false;
        }
        let root = self.RootPointer();
        let indexed = {
            let mut heap = LayoutHeapScope::new();
            let _pass = LayoutPassScope::new(&self.algorithms_, &self.objects_);
            // A coverage fallback can follow the ordinary indexed property
            // update in the same frame. In that case there are deliberately no
            // pending IDs left: the source scroll properties are already
            // current and only cull state needs refreshing.
            let refreshed = self.pending_scroll_ids_.is_empty()
                || self.scroll_paint_index_.as_ref().is_some_and(|index| {
                    index.refresh(unsafe { &mut *root }, &self.pending_scroll_ids_)
                });
            heap.DeferCollection();
            refreshed
        };
        trace.set("indexed_refresh", indexed as u8 as f64);
        if !indexed {
            let assembly = LayoutAssembly {
                algorithms: self.algorithms_,
                objects: self.objects_,
                boundary: self.boundary_,
            };
            assembly.RefreshScrollPaintProperties(unsafe { &mut *root }, fragments, true);
            let reuse = self
                .scroll_paint_index_
                .take()
                .or(self.scroll_paint_index_spare_.take());
            self.scroll_paint_index_ = Some({
                let _phase = layout_fragment_boundary::trace::span(
                    layout_fragment_boundary::trace::TracePhase::BuildScrollPaintIndex,
                );
                ScrollPaintPropertyIndex::new(
                    unsafe { &mut *root },
                    fragments,
                    self.CachedScrollIndexObjects(),
                    reuse,
                )
            });
        }

        // Source: core/paint/cull_rect_updater.cc:67-89,193-206.
        // Chromium retains the FragmentData graph and commits the scroll
        // position used by the new contents CullRect. The Layout boundary must
        // do the same: republishing the complete LayoutResult here would create
        // a second, non-source fragment-projection lifecycle.
        unsafe fn commit_scroll_origins(
            cursor: PaintFragmentCursor<'_>,
            changed: &HashSet<u64>,
        ) -> usize {
            let children = cursor.Children();
            let fragment = unsafe { cursor.PaintDataMut() };
            let mut committed = 0;
            if fragment.paint.scroll_container.is_some()
                && (changed.is_empty() || changed.contains(&fragment.node_id))
            {
                let origin = Some(fragment.paint.scroll_offset);
                if fragment.paint_cull_scroll_offset != origin {
                    fragment.paint_cull_scroll_offset = origin;
                    committed += 1;
                }
            }
            for child in children {
                committed += unsafe { commit_scroll_origins(child, changed) };
            }
            committed
        }
        let committed = unsafe {
            commit_scroll_origins(
                PaintFragmentCursor::FromHandle(fragments),
                &self.pending_scroll_ids_,
            )
        };
        trace.set("committed_origins", committed as f64);
        {
            let _phase = layout_fragment_boundary::trace::span(
                layout_fragment_boundary::trace::TracePhase::UpdatePrePaintInputs,
            );
            UpdatePrePaintInputs(&mut self.pre_paint_inputs_, fragments, &self.objects_);
        }
        self.pending_scroll_ids_.clear();
        true
    }
}
