//! Thin HTML-control routes to the canonical LayoutObject/ComputedStyle owner.
use super::*;
impl LayoutEngine {
    /// ListBoxSelectType::ScrollToOptionTask, select_type.cc:1720-1754.
    /// Scroll only the resident list-box ScrollableArea; parent containers
    /// deliberately do not participate in this ScrollIntoView operation.
    pub fn ScrollSelectOptionIntoView(
        &mut self,
        container: u64,
        option: u64,
    ) -> crate::native_scroll_source::NativeScrollResult {
        use crate::native_scroll_source::{
            NativeScrollResult, ScrollAlignment, ScrollIntoViewParams,
        };
        let rect = self.BoundingBoxForScrollIntoView(option);
        let Some(mut source) = self.NativeScrollSource(container) else {
            return NativeScrollResult::default();
        };
        if !source.IsScrollContainerBox() {
            return NativeScrollResult::default();
        }
        source.ScrollIntoOwnArea(
            rect,
            Default::default(),
            ScrollIntoViewParams {
                align_x: ScrollAlignment::ToEdgeIfNeeded(),
                align_y: ScrollAlignment::ToEdgeIfNeeded(),
                behavior:
                    layoutng_style::style::scroll_enums::mojom::blink::ScrollBehavior::kInstant,
                ..Default::default()
            },
        )
    }
    /// Element::DidChangeIsInCanvasSubtree, element.cc:4502-4507.
    pub fn ElementCanvasSubtreeChanged(&mut self, id: u64) {
        let _scope = LayoutPassScope::new(&self.algorithms_, &self.objects_);
        if let Some(object) = self.tree_.as_mut().and_then(|tree| tree.OutputNodeMut(id)) {
            object.SetNeedsPaintPropertyUpdate();
            object.SetShouldDoFullPaintInvalidation();
            object.SetBackgroundNeedsFullPaintInvalidation();
        }
    }

    /// RangeInputType::ListAttributeTargetChanged, range_input_type.cc:415-423.
    pub fn InvalidateRangeDataList(&mut self, host_id: u64, track_id: Option<u64>) {
        let _scope = LayoutPassScope::new(&self.algorithms_, &self.objects_);
        let Some(tree) = self.tree_.as_mut() else {
            return;
        };
        if let Some(object) = tree.OutputNodeMut(host_id) {
            object.SetSubtreeShouldDoFullPaintInvalidation(
                foundation::PaintInvalidationReason::kSubtree,
            );
        }
        if let Some(object) = track_id.and_then(|id| tree.OutputNodeMut(id)) {
            object.SetNeedsLayout(std::ptr::addr_of!(
                crate::internal::layout_invalidation_reason::kAttributeChanged
            ));
        }
    }

    pub fn SetInputShouldDoFullPaintInvalidation(&mut self, id: u64) -> bool {
        let _scope = LayoutPassScope::new(&self.algorithms_, &self.objects_);
        let Some(object) = self.tree_.as_mut().and_then(|tree| tree.OutputNodeMut(id)) else {
            return false;
        };
        object.SetShouldDoFullPaintInvalidation();
        true
    }
    pub fn SetInputNeedsLayoutIntrinsicWidthsAndPaint(&mut self, id: u64) -> bool {
        let _scope = LayoutPassScope::new(&self.algorithms_, &self.objects_);
        let Some(object) = self.tree_.as_mut().and_then(|tree| tree.OutputNodeMut(id)) else {
            return false;
        };
        object.SetNeedsLayoutAndIntrinsicWidthsRecalcAndFullPaintInvalidation(std::ptr::addr_of!(
            crate::internal::layout_invalidation_reason::kAttributeChanged
        ));
        true
    }
    pub fn SetInputModifiedStyleOutsideStyleRecalc(
        &mut self,
        id: u64,
        style: foundation::Persistent<StyleComputedStyle>,
    ) -> bool {
        let _scope = LayoutPassScope::new(&self.algorithms_, &self.objects_);
        let Some(object) = self.tree_.as_mut().and_then(|tree| tree.OutputNodeMut(id)) else {
            return false;
        };
        object.SetModifiedStyleOutsideStyleRecalc(
            style.Get(),
            crate::internal::layout_object::ApplyStyleChanges::kYes,
        );
        true
    }
}
