use super::*;
use foundation::Length;
use layoutng_style::style::computed_style::ComputedStyleBuilder;
use layoutng_style::style::computed_style_base::{
    ComputedStyleBase, FieldDifference, IsAtShadowBoundary,
};
use layoutng_style::style::computed_style_constants::PseudoId;

unsafe extern "Rust" {
    fn DispatchLayoutObjectAnonymousHasStylePropagationOverride(object: *mut LayoutObject) -> bool;
    fn DispatchLayoutObjectUpdateAnonymousChildStyle(
        object: *const LayoutObject,
        child: *const LayoutObject,
        builder: &mut ComputedStyleBuilder,
    );
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
        LayoutObjectSetStyleWithChanges(self, style, apply_changes)
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
    let field_diff = ComputedStyleBase::FieldInvalidationDiff(old_style, new_style);
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

// cpp: layoutng/internal/layout_object_core_services.cc:164-182
#[inline(never)]
pub(super) fn LayoutObjectSetStyleWithChanges(
    object: &mut LayoutObject,
    style: *const ComputedStyle,
    apply_changes: ApplyStyleChanges,
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
    if !old_style.is_null() {
        let old_style_ref = unsafe { &*old_style };
        let new_style_ref = unsafe { &*style };
        difference = transform_style_difference(old_style_ref, new_style_ref);
        // Source: core/layout/layout_object.cc:3102-3227. Chromium starts
        // SetStyle() with ComputedStyle::VisualInvalidationDiff(). The layout
        // boundary already classifies geometry separately, but a retained
        // LayoutObject must still invalidate its DisplayItemClient whenever
        // the complete projected style changes. Otherwise PaintController is
        // allowed to move the old drawing from the previous PaintArtifact even
        // though this fragment now carries a new color/background/effect.
        if old_style_ref != new_style_ref {
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
}
