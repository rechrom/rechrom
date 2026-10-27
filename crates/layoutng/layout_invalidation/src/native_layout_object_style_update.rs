use super::*;
use layoutng_style::style::computed_style::ComputedStyleBuilder;

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

    // Keep pseudo-element inheritance visible as its own invalidation stage;
    // the nested style transaction retains a separate frame below.
    // cpp: layoutng/internal/layout_object_core_services.cc:184-188
    #[inline(never)]
    pub fn SetPseudoElementStyleWithMatchParentSize(
        &mut self,
        owner: &LayoutObject,
        _match_parent_size: bool,
    ) {
        self.CheckIsNotDestroyed();
        self.InheritIsInDetachedNonDomTree(owner);
        self.SetStyle(owner.StyleRef());
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:190-195
    #[inline]
    pub fn StyleWillChange(
        &mut self,
        _difference: StyleDifference,
        _old_style: *const ComputedStyle,
        _new_style: &ComputedStyle,
        _context: &mut StyleChangeContext,
    ) {
        self.CheckIsNotDestroyed();
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
        difference = transform_style_difference(unsafe { &*old_style }, unsafe { &*style });
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
}
