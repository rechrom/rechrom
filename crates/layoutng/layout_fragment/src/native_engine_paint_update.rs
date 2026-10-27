// Native paint publication and retained-paint refresh live with fragment
// ownership. LayoutEngine keeps only its stable public entry points.

use super::*;

pub(super) trait NativeEnginePaintUpdate {
    fn populate_paint_data(&mut self) -> bool;
    fn refresh_targeted_paint_data(&mut self) -> bool;
    fn refresh_targeted_paint_inputs(&mut self) -> bool;
    fn refresh_scroll_cull_data(&mut self, fragments: &mut LayoutResultHandle) -> bool;
}

impl NativeEnginePaintUpdate for LayoutEngine {
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
            trace.set("native_needs_layout", 1.0);
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
            UpdateNativePrePaintInputs(&mut inputs, &fragments, &self.objects_);
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
            trace.set("native_needs_layout", 1.0);
            return false;
        }
        UpdateNativePrePaintInputs(&mut self.pre_paint_inputs_, fragments, &self.objects_);
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
        let _pass = LayoutPassScope::new(&self.algorithms_, &self.objects_);
        if !crate::internal::boundary::layout_boundary::RefreshRetainedPaintInputs(
            unsafe { &mut *root },
            NativePaintFragmentCursor::FromHandle(fragments),
            &self.pending_paint_ids_,
            &self.pending_inline_ink_ids_,
        ) {
            trace.set("rejected", 1.0);
            return false;
        }
        UpdateNativePrePaintInputs(&mut self.pre_paint_inputs_, fragments, &self.objects_);
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
        let assembly = LayoutAssembly {
            algorithms: self.algorithms_,
            objects: self.objects_,
            boundary: self.boundary_,
        };
        let indexed = {
            let mut heap = LayoutHeapScope::new();
            let _pass = LayoutPassScope::new(&self.algorithms_, &self.objects_);
            // A coverage fallback can follow the ordinary indexed property
            // update in the same frame. In that case there are deliberately no
            // pending IDs left: the native scroll properties are already
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
            assembly.RefreshScrollPaintProperties(unsafe { &mut *root }, fragments, true);
        }
        let published = assembly.PublishLayoutResult(
            unsafe { &mut *root },
            &self.constraints_,
            &mut self.boundary_cache_,
        );
        {
            let _phase = layout_fragment_boundary::trace::span(
                layout_fragment_boundary::trace::TracePhase::UpdatePrePaintInputs,
            );
            UpdateNativePrePaintInputs(&mut self.pre_paint_inputs_, &published, &self.objects_);
        }
        let reuse = self
            .scroll_paint_index_
            .take()
            .or(self.scroll_paint_index_spare_.take());
        self.scroll_paint_index_ = Some({
            let _phase = layout_fragment_boundary::trace::span(
                layout_fragment_boundary::trace::TracePhase::BuildScrollPaintIndex,
            );
            let _pass = LayoutPassScope::new(&self.algorithms_, &self.objects_);
            ScrollPaintPropertyIndex::new(
                unsafe { &mut *root },
                &published,
                self.CachedScrollIndexObjects(),
                reuse,
            )
        });
        *fragments = published;
        self.pending_scroll_ids_.clear();
        true
    }
}
