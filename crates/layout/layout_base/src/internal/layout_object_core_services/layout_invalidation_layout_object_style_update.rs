use super::*;
use foundation::Length;
use layout_style::style::computed_style::ComputedStyleBuilder;
use layout_style::style::computed_style_base::{
    ComputedStyleBase, FieldDifference, IsAtShadowBoundary,
};
use layout_style::style::computed_style_constants::PseudoId;

use crate::block_virtual_dispatch::{
    DispatchLayoutObjectAnonymousHasStylePropagationOverride,
    DispatchLayoutObjectUpdateAnonymousChildStyle,
};
unsafe extern "Rust" {
    fn LayoutObjectAdjustStyleDifferenceProvider(
        object: *const LayoutObject,
        difference: StyleDifference,
    ) -> StyleDifference;
    fn LayoutObjectApplyPseudoElementStyleChangesProvider(
        object: *mut LayoutObject,
        old_style: *const ComputedStyle,
    );
    fn LayoutObjectApplyFirstLineChangesProvider(
        object: *mut LayoutObject,
        old_style: *const ComputedStyle,
    );
}

impl LayoutObject {
    // The C++ default argument is a wrapper in Rust.
    // cpp: layoutng/internal/layout_object.h:2203-2205
    #[inline]
    pub fn SetStyle(&mut self, style: *const ComputedStyle) {
        self.SetStyleWithChanges(style, ApplyStyleChanges::kYes);
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:164-182
    #[inline]
    pub fn SetStyleWithChanges(
        &mut self,
        style: *const ComputedStyle,
        apply_changes: ApplyStyleChanges,
    ) {
        LayoutObjectSetStyleWithChanges(self, style, apply_changes, None)
    }

    /// LayoutObject::SetStyle for a retained-object restyle whose caller has
    /// already computed `ComputedStyleBase::FieldInvalidationDiff` for this
    /// style replacement (old canonical style -> new canonical style; the
    /// resident pair may differ from it only by an identical retained font
    /// binding, so `field_diff` equals or covers the resident pair's diff).
    /// Chromium's SetStyle classifies a replacement once
    /// (core/layout/layout_object.cc:2991-3030); this keeps that single
    /// classification instead of re-diffing every field group here.
    #[inline]
    pub(crate) fn SetStyleWithFieldDiff(&mut self, style: *const ComputedStyle, field_diff: u64) {
        LayoutObjectSetStyleWithChanges(self, style, ApplyStyleChanges::kYes, Some(field_diff))
    }

    // cpp: layoutng/internal/layout_object.h:2208-2209
    #[inline]
    pub fn SetPseudoElementStyle(&mut self, owner: &LayoutObject) {
        self.SetPseudoElementStyleWithMatchParentSize(owner, false);
    }

    // cpp: core/layout/layout_object.cc:3031-3099
    #[inline(never)]
    pub fn SetPseudoElementStyleWithMatchParentSize(
        &mut self,
        owner: &LayoutObject,
        match_parent_size: bool,
    ) {
        self.CheckIsNotDestroyed();
        self.InheritIsInDetachedNonDomTree(owner);
        let pseudo_style = owner.StyleRef();
        debug_assert!(
            [
                PseudoId::kPseudoIdCheckMark,
                PseudoId::kPseudoIdBefore,
                PseudoId::kPseudoIdAfter,
                PseudoId::kPseudoIdExpandIcon,
                PseudoId::kPseudoIdPickerIcon,
                PseudoId::kPseudoIdInterestButton,
                PseudoId::kPseudoIdMarker,
                PseudoId::kPseudoIdFirstLetter,
                PseudoId::kPseudoIdScrollMarkerGroup,
                PseudoId::kPseudoIdScrollMarker,
                PseudoId::kPseudoIdScrollButtonBlockStart,
                PseudoId::kPseudoIdScrollButtonInlineStart,
                PseudoId::kPseudoIdScrollButtonInlineEnd,
                PseudoId::kPseudoIdScrollButtonBlockEnd,
            ]
            .contains(&pseudo_style.StyleType())
                || pseudo_style.IsPageMarginBox()
        );

        if self.IsImage() || self.IsQuote() {
            let mut builder = ComputedStyleBuilder::from_initial_and_parent(
                unsafe { &*ComputedStyle::GetInitialStyleSingleton() },
                pseudo_style,
                IsAtShadowBoundary::kNotAtShadowBoundary,
            );
            if match_parent_size {
                debug_assert!(self.IsImage());
                builder.SetWidth(&Length::Percent(100.0));
                builder.SetHeight(&Length::Percent(100.0));
            }
            self.SetStyle(builder.TakeStyle());
            return;
        }

        let parent = self.Parent();
        if self.IsText() && !parent.is_null() && unsafe { &*parent }.IsInitialLetterBox() {
            std::panic::panic_any(UnsupportedLayout::new(
                "initial-letter pseudo text style requires StyleForInitialLetterText",
            ));
        }

        if self.IsText() && !parent.is_null() && unsafe { &*parent }.IsLayoutTextCombine() {
            let mut builder = ComputedStyleBuilder::from_initial_and_parent(
                unsafe { &*ComputedStyle::GetInitialStyleSingleton() },
                pseudo_style,
                IsAtShadowBoundary::kNotAtShadowBoundary,
            );
            let factories = LayoutObjectFactoryScope::Objects();
            let adjust = unsafe { factories.as_ref() }
                .and_then(|factories| factories.adjust_combined_text_style)
                .unwrap_or_else(|| {
                    std::panic::panic_any(UnsupportedLayout::new(
                        "inline combined-text style adjustment is not installed",
                    ))
                });
            adjust(&mut builder);
            self.SetStyle(builder.TakeStyle());
            return;
        }

        self.SetStyle(pseudo_style);
    }

    // Source: core/layout/layout_object.cc:3361-3388. Rechrom has no
    // Document draggable-region state or InputMethodController host; retain
    // the complete PaintLayer visibility invalidation and do not synthesize
    // either host notification.
    #[inline]
    pub fn StyleWillChange(
        &mut self,
        _difference: StyleDifference,
        old_style: *const ComputedStyle,
        new_style: &ComputedStyle,
        _context: &mut StyleChangeContext,
    ) {
        self.CheckIsNotDestroyed();
        debug_assert!(!self.IsText());
        if old_style.is_null() || unsafe { &*old_style }.Visibility() == new_style.Visibility() {
            return;
        }
        let layer = self.EnclosingLayer();
        if !layer.is_null() {
            unsafe { &mut *layer }.DirtyVisibleContentStatus();
        }
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:197-203
    #[inline]
    pub fn StyleDidChange(
        &mut self,
        difference: StyleDifference,
        _old_style: *const ComputedStyle,
        _new_style: &ComputedStyle,
        _context: &StyleChangeContext,
    ) {
        self.CheckIsNotDestroyed();
        debug_assert!(!difference.NeedsFullLayout());
        if difference.disable_scroll_anchoring() {
            self.SetScrollAnchorDisablingStyleChanged(true);
        }
    }

    pub fn AnonymousHasStylePropagationOverrideBase(&mut self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    // cpp: layoutng/internal/layout_object.h:3398-3401
    pub fn AnonymousHasStylePropagationOverride(&mut self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectAnonymousHasStylePropagationOverride(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2854-2857
    pub fn UpdateAnonymousChildStyleBase(
        &self,
        _child: *const LayoutObject,
        _builder: &mut ComputedStyleBuilder,
    ) {
        self.CheckIsNotDestroyed();
    }

    // cpp: layoutng/internal/layout_object.h:2854-2857
    #[inline(never)]
    pub fn UpdateAnonymousChildStyle(
        &self,
        child: *const LayoutObject,
        builder: &mut ComputedStyleBuilder,
    ) {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectUpdateAnonymousChildStyle(self, child, builder) }
    }

    // cpp: layoutng/internal/layout_object.h:3565-3565
    #[inline(never)]
    pub(crate) fn AdjustStyleDifference(&self, difference: StyleDifference) -> StyleDifference {
        unsafe { LayoutObjectAdjustStyleDifferenceProvider(self, difference) }
    }

    // cpp: layoutng/internal/layout_object.h:3576-3576
    #[inline(never)]
    pub(crate) fn ApplyPseudoElementStyleChanges(&mut self, old_style: *const ComputedStyle) {
        unsafe { LayoutObjectApplyPseudoElementStyleChangesProvider(self, old_style) }
    }

    // cpp: layoutng/internal/layout_object.h:3577-3577
    #[inline(never)]
    pub(crate) fn ApplyFirstLineChanges(&mut self, old_style: *const ComputedStyle) {
        unsafe { LayoutObjectApplyFirstLineChangesProvider(self, old_style) }
    }
}

// cpp: layoutng/internal/layout_object_core_services.cc:34-37
fn same_transform_operation<T: PartialEq>(left: *mut T, right: *mut T) -> bool {
    ValuesEquivalent(left, right)
}

// cpp: layoutng/internal/layout_object_core_services.cc:39-103
#[inline(never)]
fn transform_style_difference(
    old_style: &ComputedStyle,
    new_style: &ComputedStyle,
    field_diff: u64,
) -> StyleDifference {
    // ComputedStyle::VisualInvalidationDiff marks every field in the generated
    // kScrollAnchor group as an anchoring suppression trigger.  Layout's
    // retained SetStyle path deliberately projects only the invalidation bits
    // it owns locally, but it must not drop this source bit: size, margin,
    // padding, inset and transform changes on the selected anchor otherwise
    // let a later layout resurrect a pre-clamp scroll offset.
    //
    // Source: core/style/computed_style.cc:899-904;
    //         core/layout/layout_object.cc:3521-3523.
    let transform_property_changed = old_style.Transform() != new_style.Transform();
    let offset_anchor_changed = old_style.OffsetAnchor() != new_style.OffsetAnchor();
    let offset_path_changed = !ValuesEquivalent(old_style.OffsetPath(), new_style.OffsetPath());
    let rotate_changed = !same_transform_operation(old_style.Rotate(), new_style.Rotate());
    let offset_distance_changed = old_style.OffsetDistance() != new_style.OffsetDistance();
    let offset_position_changed = old_style.OffsetPosition() != new_style.OffsetPosition();
    let perspective_origin_changed = old_style.PerspectiveOrigin() != new_style.PerspectiveOrigin();
    let offset_rotate_changed = old_style.OffsetRotate() != new_style.OffsetRotate();
    let perspective_changed = old_style.Perspective() != new_style.Perspective();
    let scale_changed = !same_transform_operation(old_style.Scale(), new_style.Scale());
    let translate_changed = !same_transform_operation(old_style.Translate(), new_style.Translate());
    let transform_origin_changed = old_style.GetTransformOrigin() != new_style.GetTransformOrigin();
    let transform_box_changed = old_style.TransformBox() != new_style.TransformBox();
    let transform_other_changed = offset_anchor_changed
        || offset_path_changed
        || rotate_changed
        || offset_distance_changed
        || offset_position_changed
        || perspective_origin_changed
        || offset_rotate_changed
        || perspective_changed
        || scale_changed
        || translate_changed
        || transform_origin_changed
        || transform_box_changed;
    let has_transform_field_changed = transform_property_changed
        || offset_path_changed
        || rotate_changed
        || offset_position_changed
        || scale_changed
        || translate_changed
        || old_style.HasCurrentRotateAnimation() != new_style.HasCurrentRotateAnimation()
        || old_style.HasCurrentScaleAnimation() != new_style.HasCurrentScaleAnimation()
        || old_style.HasCurrentTransformAnimation() != new_style.HasCurrentTransformAnimation()
        || old_style.HasCurrentTranslateAnimation() != new_style.HasCurrentTranslateAnimation();

    let mut diff = StyleDifference::default();
    diff.set_transform_data_changed(
        transform_property_changed
            || offset_anchor_changed
            || offset_path_changed
            || rotate_changed
            || offset_distance_changed
            || offset_position_changed
            || offset_rotate_changed
            || scale_changed
            || translate_changed
            || transform_origin_changed
            || transform_box_changed,
    );
    if transform_other_changed
        || (has_transform_field_changed && old_style.HasTransform() != new_style.HasTransform())
    {
        diff.set_transform_changed(true);
    } else if transform_property_changed {
        diff.set_only_transform_property_changed(true);
        diff.set_transform_changed(true);
    }
    diff.set_disable_scroll_anchoring(
        field_diff & FieldDifference::kScrollAnchor.bits() != 0 || diff.transform_changed(),
    );
    diff
}

/// Whether SetStyle invalidates this object's DisplayItemClient.
///
/// Source: core/layout/layout_object.cc:3076-3078,3109-3120,3150-3160 with
/// AdjustStyleDifference (2878-2918) and AdjustForCompositableAnimationPaint
/// (257-275). Chromium invalidates on NeedsNormalPaintInvalidation, on the
/// simple invalidation AdjustStyleDifference derives from
/// text_decoration_or_color_changed (taken here without its narrowing
/// predicate, which only skips work), and on background-color / clip-path
/// changes without a compositor paint worklet. A transform or opacity change
/// is property-only.
///
/// The PaintLayer::StyleDidChange repaint branches for z-index
/// (paint_layer.cc:2512-2518) and moving-pixel filters (2429-2433), the
/// non-isolated blend-mode descendant bookkeeping, and the layout-driven
/// geometry branches are not separately translated on this retained path;
/// those differences keep the full object invalidation.
#[allow(non_snake_case)]
fn StyleChangeNeedsPaintInvalidation(diff: &StyleDifference) -> bool {
    diff.NeedsNormalPaintInvalidation()
        || diff.NeedsSimplePaintInvalidation()
        || diff.text_decoration_or_color_changed()
        || diff.background_color_changed()
        || diff.clip_path_changed()
        || diff.NeedsFullLayout()
        || diff.NeedsPositionedLayout()
        || diff.needs_reshape()
        || diff.needs_recompute_visual_overflow()
        || diff.z_index_changed()
        || diff.filter_changed()
        || diff.mask_changed()
        || diff.blend_mode_changed()
        || diff.border_radius_changed()
        || diff.border_shape_changed()
        || diff.needs_box_paint_property_update()
}

/// The document collaborators of ComputedStyle::VisualInvalidationDiff seen
/// from a retained LayoutObject: this product installs no LayoutWorklet or
/// CSSPaintImageGenerator, so a paint() image is unpainted and invalidates
/// (computed_style.cc DiffNeedsNormalPaintInvalidation), and registered
/// worklet property lists never reach the equality callbacks.
struct LayoutObjectVisualInvalidationContext;

impl layout_style::style::visual_invalidation_diff::VisualInvalidationDiffContext
    for LayoutObjectVisualInvalidationContext
{
    fn LayoutWorkletDefinition(
        &self,
        _name: &foundation::AtomicString,
    ) -> layout_style::style::visual_invalidation_diff::LayoutWorkletDefinition<'_> {
        layout_style::style::visual_invalidation_diff::LayoutWorkletDefinition::FeatureDisabled
    }
    fn PaintInvalidationProperties(
        &self,
        _image: &layout_style::style::style_image::StyleImage,
    ) -> layout_style::style::visual_invalidation_diff::PaintInvalidationProperties<'_> {
        layout_style::style::visual_invalidation_diff::PaintInvalidationProperties::Unpainted
    }
    fn NativePropertiesEqual(
        &self,
        _properties: &[foundation::CSSPropertyID],
        _current: &ComputedStyle,
        _other: &ComputedStyle,
    ) -> bool {
        false
    }
    fn CustomPropertiesEqual(
        &self,
        _properties: &[foundation::AtomicString],
        _current: &ComputedStyle,
        _other: &ComputedStyle,
    ) -> bool {
        false
    }
    fn Potential3DCompositingReasons(
        &self,
        style: &ComputedStyle,
    ) -> foundation::CompositingReasons {
        style.Potential3DCompositingReasons()
    }
}

// Source: core/paint/paint_layer_painter.cc:167-190.
//
// PaintLayer::StyleDidChange compares this predicate before and after a style
// replacement. Crossing the near-zero-opacity boundary changes the
// `effectively_invisible` bit inherited by every descendant PaintChunk, so a
// normal invalidation of only this LayoutObject is insufficient: cached child
// subsequences must be recorded again as well.
fn painted_output_invisible(style: &ComputedStyle) -> bool {
    if style.HasNonInitialBackdropFilter()
        || style.HasWillChangeProperty(foundation::CSSPropertyID::kOpacity)
        || style.HasCurrentOpacityAnimation()
    {
        return false;
    }
    style.Opacity() < 0.0004
}

// Source: core/paint/paint_property_tree_builder.cc:4803-4810
// (PaintPropertyTreeBuilder::ScheduleDeferredTransformNodeUpdate) with the
// LayoutObject half of CanDoDeferredTransformNodeUpdate (4953-4977). The
// remaining conditions read ObjectPaintProperties (an existing Transform
// node, no Effect for a 2D scale transform) and the canvas subtree; PrePaint
// owns those and checks them when it executes the update, running its
// ordinary walk for this object when one fails.
fn ScheduleDeferredTransformNodeUpdate(object: &LayoutObject) -> bool {
    if object.NeedsPaintPropertyUpdate() || object.DescendantNeedsPaintPropertyUpdate() {
        return false;
    }
    if object.IsSVGChild() || !object.IsBox() || object.IsFragmented() {
        return false;
    }
    let Some(view) = (unsafe { object.View().as_ref() }) else {
        return false;
    };
    view.AddPendingTransformUpdate(object);
    true
}

// cpp: layoutng/internal/layout_object_core_services.cc:164-182
#[inline(never)]
pub(super) fn LayoutObjectSetStyleWithChanges(
    object: &mut LayoutObject,
    style: *const ComputedStyle,
    apply_changes: ApplyStyleChanges,
    field_diff_hint: Option<u64>,
) {
    object.CheckIsNotDestroyed();
    if object.StylePtr() == style {
        return;
    }
    assert!(!style.is_null());
    if apply_changes == ApplyStyleChanges::kNo {
        let old_style = object.StylePtr();
        object.SetStyleInternal(style);
        object.UpdateMaskImageObservers(old_style, style);
        return;
    }
    let old_style = object.StylePtr();
    let mut difference = StyleDifference::default();
    let mut visual_difference = StyleDifference::default();
    let mut painted_output_invisible_changed = false;
    if !old_style.is_null() {
        let old_style_ref = unsafe { &*old_style };
        let new_style_ref = unsafe { &*style };
        painted_output_invisible_changed =
            painted_output_invisible(old_style_ref) != painted_output_invisible(new_style_ref);
        // ComputedStyle::VisualInvalidationDiff computes the generated field
        // diff once (computed_style.cc:749-760); reuse it for the projected
        // difference and the visual diff instead of comparing twice.
        let field_diff = field_diff_hint.unwrap_or_else(|| {
            ComputedStyleBase::FieldInvalidationDiff(old_style_ref, new_style_ref)
        });
        difference = transform_style_difference(old_style_ref, new_style_ref, field_diff);
        // Source: core/layout/layout_object.cc:3018-3020. Chromium starts
        // SetStyle() with ComputedStyle::VisualInvalidationDiff(). The layout
        // boundary classifies geometry separately (the projected difference
        // above), so the visual diff decides only this object's paint
        // invalidation and paint-property update.
        //
        // The retained StyleDidChange chain translates the paint-property
        // consequences of transform and opacity (PaintLayer::StyleDidChange,
        // paint_layer.cc:2464-2493) but not yet every other branch Chromium
        // spreads over LayoutBox/LayoutBoxModelObject/PaintLayer. A style
        // change confined to those two property groups therefore follows
        // Chromium exactly; every other difference keeps the full object
        // invalidation, whose PrePaint visit rebuilds the remaining state.
        const PROPERTY_ONLY_FIELDS: u64 = FieldDifference::kTransformData.bits()
            | FieldDifference::kTransformProperty.bits()
            | FieldDifference::kTransformOther.bits()
            | FieldDifference::kHasTransform.bits()
            | FieldDifference::kOpacity.bits()
            | FieldDifference::kScrollAnchor.bits();
        let property_only = field_diff != 0 && field_diff & !PROPERTY_ONLY_FIELDS == 0;
        if field_diff != 0 {
            visual_difference = old_style_ref.VisualInvalidationDiffWithFieldDiff(
                new_style_ref,
                field_diff,
                &LayoutObjectVisualInvalidationContext,
            );
        }
        if old_style_ref != new_style_ref
            && (!property_only || StyleChangeNeedsPaintInvalidation(&visual_difference))
        {
            difference.SetNeedsNormalPaintInvalidation();
        }
    }
    let mut context = StyleChangeContext::default();
    unsafe {
        DispatchLayoutObjectStyleWillChange(object, difference, old_style, &*style, &mut context);
    }
    object.SetStyleInternal(style);
    object.UpdateMaskImageObservers(old_style, style);
    unsafe {
        DispatchLayoutObjectStyleDidChange(
            object,
            difference,
            old_style,
            &*object.StylePtr(),
            &context,
        );
    }
    // Source: core/layout/layout_object.cc LayoutObject::SetStyle, the
    // `diff.needs_recompute_visual_overflow` branch after StyleDidChange
    // (skipped, like the rest of the tail, for an unparented object). An
    // ink-overflow-only change such as box-shadow or outline needs no
    // layout: the painting layer recomputes the object's visual overflow in
    // PaintLayer::UpdateDescendantDependentFlags before PrePaint compares
    // the new visual rect.
    if visual_difference.needs_recompute_visual_overflow() && !object.Parent().is_null() {
        object.InvalidateVisualOverflow();
    }
    // Source: core/layout/layout_object.cc:3258-3269. Normal style paint
    // invalidation is recorded on the LayoutObject and exported to PrePaint;
    // PaintController then rejects the resident cached drawing for this
    // client. Geometry/layout invalidation remains owned by the existing
    // managed-object update path.
    if difference.NeedsNormalPaintInvalidation() {
        if object.IsSVGRoot() {
            object.SetShouldDoFullPaintInvalidation();
        } else {
            object.SetShouldDoFullPaintInvalidationWithoutLayoutChange(
                foundation::PaintInvalidationReason::kStyle,
            );
        }
    }
    // Source: core/paint/paint_layer.cc:2463-2473 (PaintLayer::StyleDidChange).
    // A change of only the `transform` property schedules the deferred
    // transform node update instead of a full paint-property update; PrePaint
    // then rewrites the resident Transform node in place
    // (LocalFrameView::ExecuteAllPendingUpdates).
    let mut needs_full_transform_update = visual_difference.transform_changed();
    let mut deferred_transform_update = false;
    if needs_full_transform_update
        && visual_difference.only_transform_property_changed()
        && object.HasLayer()
        && ScheduleDeferredTransformNodeUpdate(object)
    {
        needs_full_transform_update = false;
        deferred_transform_update = true;
    }
    if deferred_transform_update {
        let layer = object.EnclosingLayer();
        if !layer.is_null() {
            unsafe { &mut *layer }.SetNeedsDescendantDependentFlagsUpdate();
        }
    }
    // Source: core/layout/layout_object.cc:3173-3184 (no PaintLayer) and
    // core/paint/paint_layer.cc:2486-2493 (PaintLayer::StyleDidChange). A
    // property-only change rebuilds this object's paint property nodes;
    // PaintPropertyTreeBuilder requests a repaint itself when nodes are added
    // or removed, so the cached drawings stay valid.
    if !object.IsText()
        && (needs_full_transform_update
            || visual_difference.opacity_changed()
            || visual_difference.z_index_changed()
            || visual_difference.filter_changed()
            || visual_difference.clip_property_changed()
            || visual_difference.blend_mode_changed()
            || visual_difference.mask_changed()
            || visual_difference.compositing_reasons_changed())
    {
        object.SetNeedsPaintPropertyUpdate();
        if object.HasLayer() {
            let layer = object.EnclosingLayer();
            if !layer.is_null() {
                unsafe { &mut *layer }.MarkAncestorChainForFlagsUpdate(
                    crate::internal::paint_layer::DescendantDependentFlagsUpdateFlag::kNeedsDescendantDependentUpdate,
                );
            }
        }
    }
    // Source: core/paint/paint_layer.cc:2525-2538. The ancestor layer's
    // opacity scope stamped descendant chunks in the previous artifact; make
    // the subtree repaint when that scope starts or stops suppressing output.
    if painted_output_invisible_changed {
        object.SetSubtreeShouldDoFullPaintInvalidation(
            foundation::PaintInvalidationReason::kSubtree,
        );
    }
}
