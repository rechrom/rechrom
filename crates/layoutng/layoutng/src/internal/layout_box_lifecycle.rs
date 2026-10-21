#![allow(non_snake_case)]

use foundation::UnsupportedLayout;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::style_difference::StyleDifference;

use super::layout_box::LayoutBox;
use super::layout_box_model_object::{LayoutBoxModelObject, PaintLayerType};
use super::layout_invalidation_reason;
use super::layout_object::StyleChangeContext;
use super::layout_pass_scope::LayoutObjectFactoryScope;
use super::shapes::shape_outside_info::ShapeOutsideInfo;

impl LayoutBox {
    // LayoutBox has one C++ base, LayoutBoxModelObject, at offset zero. Its
    // pending Rust owner must retain that first-field layout for static base
    // calls to reach the same subobject without virtual redispatch.
    fn model_object_base_mut(&mut self) -> &mut LayoutBoxModelObject {
        unsafe { &mut *(self as *mut LayoutBox).cast::<LayoutBoxModelObject>() }
    }

    // cpp: layoutng/internal/layout_box_lifecycle.cc:22-31
    pub fn LayerTypeRequired(&self) -> PaintLayerType {
        self.CheckIsNotDestroyed();
        if self.IsStacked() || self.HasHiddenBackface() {
            return PaintLayerType::kNormalPaintLayer;
        }
        if self.HasNonVisibleOverflow() && !self.IsLayoutReplaced() {
            return PaintLayerType::kOverflowClipPaintLayer;
        }
        if self.IsOverscrollContainer() {
            return PaintLayerType::kForcedPaintLayer;
        }
        PaintLayerType::kNoPaintLayer
    }

    // cpp: layoutng/internal/layout_box_lifecycle.cc:33-38
    pub fn WillBeDestroyed(&mut self) {
        self.CheckIsNotDestroyed();
        ShapeOutsideInfo::RemoveInfo(self);
        self.DisassociatePhysicalFragments();
        self.model_object_base_mut().WillBeDestroyed();
    }

    // cpp: layoutng/internal/layout_box_lifecycle.cc:40-44
    pub fn InsertedIntoTree(&mut self) {
        self.CheckIsNotDestroyed();
        self.model_object_base_mut().InsertedIntoTreeBase();
        self.AddCustomLayoutChildIfNeeded();
    }

    // cpp: layoutng/internal/layout_box_lifecycle.cc:46-50
    pub fn WillBeRemovedFromTree(&mut self) {
        self.CheckIsNotDestroyed();
        self.ClearCustomLayoutChild();
        self.model_object_base_mut().WillBeRemovedFromTreeBase();
    }

    // cpp: layoutng/internal/layout_box_lifecycle.cc:52-68
    pub fn StyleWillChange(
        &mut self,
        diff: StyleDifference,
        old_style: *const ComputedStyle,
        new_style: &ComputedStyle,
        style_change_context: &mut StyleChangeContext,
    ) {
        self.CheckIsNotDestroyed();
        if !old_style.is_null() {
            if diff.NeedsFullLayout()
                && !self.Parent().is_null()
                && unsafe { &*old_style }.GetPosition() != new_style.GetPosition()
            {
                self.MarkContainerChainForLayout();
            }
            style_change_context.did_prevent_spanner_descendants =
                self.IsInsideMulticol() && self.ShouldPreventColumnSpannerDescendants();
        }
        self.model_object_base_mut().StyleWillChange(
            diff,
            old_style,
            new_style,
            style_change_context,
        );
    }

    // cpp: layoutng/internal/layout_box_lifecycle.cc:70-99
    pub fn StyleDidChange(
        &mut self,
        diff: StyleDifference,
        old_style: *const ComputedStyle,
        new_style: &ComputedStyle,
        style_change_context: &StyleChangeContext,
    ) {
        self.CheckIsNotDestroyed();
        self.model_object_base_mut().StyleDidChange(
            diff,
            old_style,
            new_style,
            style_change_context,
        );
        // Chromium layout_box.cc:852-909: changing a grid item's placement or
        // order invalidates its container's persistent placement cache.
        if !old_style.is_null() {
            let old = unsafe { &*old_style };
            let grid_changed = old.GridColumnStart() != new_style.GridColumnStart()
                || old.GridColumnEnd() != new_style.GridColumnEnd()
                || old.GridRowStart() != new_style.GridRowStart()
                || old.GridRowEnd() != new_style.GridRowEnd()
                || old.Order() != new_style.Order()
                || old.HasOutOfFlowPosition() != new_style.HasOutOfFlowPosition();
            if grid_changed {
                let container = self.ContainingBlock();
                if !container.is_null() && unsafe { &*container }.IsLayoutGrid() {
                    if !old.HasOutOfFlowPosition() || !new_style.HasOutOfFlowPosition() {
                        unsafe { &mut *container }.SetGridPlacementDirty(true);
                    }
                    unsafe { &mut *container }.SetNeedsLayoutAndIntrinsicWidthsRecalc(
                        &raw const layout_invalidation_reason::kStyleChange,
                    );
                }
            }
        }
        let axes = self.ComputeOverflowClipAxes();
        self.SetOverflowClipAxes(axes);

        if !old_style.is_null()
            && self.IsInLayoutNGInlineFormattingContext()
            && self.IsInline()
            && unsafe { &*old_style }.Direction() != new_style.Direction()
        {
            self.SetNeedsCollectInlines();
        }
        if !old_style.is_null() {
            let should_prevent_now =
                self.IsInsideMulticol() && self.ShouldPreventColumnSpannerDescendants();
            if style_change_context.did_prevent_spanner_descendants != should_prevent_now {
                let factories = LayoutObjectFactoryScope::Objects();
                let invalidate = if factories.is_null() {
                    None
                } else {
                    unsafe { &*factories }.invalidate_column_spanners
                };
                let Some(invalidate) = invalidate else {
                    std::panic::panic_any(UnsupportedLayout::new(
                        "multicol layout module is not installed",
                    ));
                };
                invalidate(self);
            }
        }
        if diff.transform_changed() && self.TransformsChangeMayRequireLayout() {
            let reason = unsafe { &layout_invalidation_reason::kStyleChange as *const _ };
            self.SetNeedsLayoutAndFullPaintInvalidation(reason);
        }
    }
}
