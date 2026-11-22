//! ComputedStyle::VisualInvalidationDiff, Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
//! computed_style.cc:749-914,964-1186,1258-1283. Worklet definitions and
//! CSSPropertyEquality remain the original narrow document/style collaborators.
use super::{
    computed_style::ComputedStyle,
    computed_style_base::{ComputedStyleBase, FieldDifference as F},
    style_difference::StyleDifference,
    style_image::StyleImage,
};
use foundation::{AtomicString, CSSPropertyID, CompositingReasons, EPosition, EVisibility};
#[derive(Clone, Copy)]
pub struct InvalidationProperties<'a> {
    pub native: &'a [CSSPropertyID],
    pub custom: &'a [AtomicString],
}
#[derive(Clone, Copy)]
pub struct LayoutInvalidationDefinition<'a> {
    pub own: InvalidationProperties<'a>,
    pub child: InvalidationProperties<'a>,
}
/// A feature-disabled product cannot produce registered layout() styles. Keep
/// that admission distinct from a live registry with no matching definition.
pub enum LayoutWorkletDefinition<'a> {
    FeatureDisabled,
    UnsupportedProduct,
    NotRegistered,
    Invalid,
    Registered(LayoutInvalidationDefinition<'a>),
}
pub enum PaintInvalidationProperties<'a> {
    UnsupportedProduct,
    Unpainted,
    Registered(InvalidationProperties<'a>),
}
#[derive(Debug)]
pub enum UnsupportedStyleInvalidationCapability {
    LayoutWorklet,
    PaintWorklet,
}
pub trait VisualInvalidationDiffContext {
    fn LayoutWorkletDefinition(&self, name: &AtomicString) -> LayoutWorkletDefinition<'_>;
    fn PaintInvalidationProperties(&self, image: &StyleImage) -> PaintInvalidationProperties<'_>;
    fn NativePropertiesEqual(
        &self,
        properties: &[CSSPropertyID],
        current: &ComputedStyle,
        other: &ComputedStyle,
    ) -> bool;
    fn CustomPropertiesEqual(
        &self,
        properties: &[AtomicString],
        current: &ComputedStyle,
        other: &ComputedStyle,
    ) -> bool;
    fn Potential3DCompositingReasons(&self, style: &ComputedStyle) -> CompositingReasons;
}
impl ComputedStyle {
    pub fn VisualInvalidationDiff(
        &self,
        other: &Self,
        context: &dyn VisualInvalidationDiffContext,
    ) -> StyleDifference {
        let mut diff = StyleDifference::default();
        let field_diff = ComputedStyleBase::FieldInvalidationDiff(self, other);
        let changed = |flag: F| field_diff & flag.bits() != 0;
        if self.DiffNeedsReshape(other, field_diff) {
            diff.set_needs_reshape(true);
            diff.SetNeedsFullLayout();
            diff.SetNeedsNormalPaintInvalidation();
        }
        if self.IsStackingContextWithoutContainment() != other.IsStackingContextWithoutContainment()
        {
            diff.SetNeedsFullLayout();
            diff.SetNeedsNormalPaintInvalidation();
            diff.set_z_index_changed(true);
        }
        if (!diff.NeedsFullLayout() || !diff.NeedsNormalPaintInvalidation())
            && self.DiffNeedsFullLayoutAndPaintInvalidation(other, field_diff)
        {
            diff.SetNeedsFullLayout();
            diff.SetNeedsNormalPaintInvalidation();
        }
        if !diff.NeedsFullLayout() {
            if self.DiffNeedsFullLayoutWithContext(other, field_diff, context) {
                diff.SetNeedsFullLayout();
            } else if changed(F::kInset) && self.GetPosition() != EPosition::kStatic {
                diff.SetNeedsPositionedLayout();
            }
        }
        if !diff.NeedsNormalPaintInvalidation()
            && self.DiffNeedsNormalPaintInvalidationWithContext(other, field_diff, context)
        {
            diff.SetNeedsNormalPaintInvalidation();
        }
        if self.DiffNeedsRecomputeVisualOverflow(other, field_diff) {
            diff.set_needs_recompute_visual_overflow(true);
        }
        if self.DiffCompositingReasonsWithContext(other, field_diff, context) {
            diff.set_compositing_reasons_changed(true);
        }
        if changed(F::kAXStyle) {
            diff.set_ax_style_changed(true);
        }
        if changed(F::kBackgroundColor) {
            diff.set_background_color_changed(true);
        }
        if changed(F::kBlendMode) {
            diff.set_blend_mode_changed(true);
        }
        if changed(F::kBorderRadius) {
            diff.set_border_radius_changed(true);
        }
        if changed(F::kBorderShape) {
            diff.set_border_shape_changed(true);
        }
        if changed(F::kClip) {
            let has_clip = self.HasOutOfFlowPosition() && !self.HasAutoClip();
            let other_has_clip = other.HasOutOfFlowPosition() && !other.HasAutoClip();
            if has_clip != other_has_clip || (has_clip && self.Clip() != other.Clip()) {
                diff.set_clip_property_changed(true);
            }
        }
        if changed(F::kClipPath) {
            diff.set_clip_path_changed(true);
        }
        if changed(F::kBoxPaintProperty) {
            diff.set_needs_box_paint_property_update(true);
        }
        if changed(F::kColor) {
            diff.set_ax_style_changed(true);
            diff.set_text_decoration_or_color_changed(true);
        }
        if changed(F::kFilterData) {
            diff.set_filter_changed(true);
        }
        if changed(F::kInert) && self.IsInert() != other.IsInert() {
            diff.set_ax_visibility_or_inert_changed(true);
        }
        if changed(F::kMask) {
            diff.set_mask_changed(true);
        }
        if changed(F::kOpacity) {
            diff.set_opacity_changed(true);
        }
        if changed(F::kScrollbarColor) && self.UsedScrollbarColor() != other.UsedScrollbarColor() {
            diff.SetNeedsNormalPaintInvalidation();
        }
        if changed(F::kScrollbarStyle)
            && (self.HasPseudoElementStyle(
                super::computed_style_constants::PseudoId::kPseudoIdScrollbar,
            ) != other.HasPseudoElementStyle(
                super::computed_style_constants::PseudoId::kPseudoIdScrollbar,
            ) || self.UsesStandardScrollbarStyle() != other.UsesStandardScrollbarStyle())
        {
            diff.SetNeedsFullLayout();
            diff.SetNeedsNormalPaintInvalidation();
        }
        if changed(F::kTextDecoration) {
            diff.set_ax_style_changed(true);
            diff.set_text_decoration_or_color_changed(true);
        }
        if changed(F::kTransformData) {
            diff.set_transform_data_changed(true);
        }
        if changed(F::kTransformOther) {
            diff.set_transform_changed(true);
        } else if changed(F::kHasTransform) && self.HasTransform() != other.HasTransform() {
            diff.set_transform_changed(true);
        } else if changed(F::kTransformProperty) {
            diff.set_only_transform_property_changed(true);
            diff.set_transform_changed(true);
        }
        if changed(F::kVisibility) {
            if (self.Visibility() == EVisibility::kCollapse)
                != (other.Visibility() == EVisibility::kCollapse)
            {
                diff.SetNeedsFullLayout();
            }
            diff.set_ax_visibility_or_inert_changed(true);
        }
        if changed(F::kZIndex) {
            diff.set_z_index_changed(true);
        }
        if diff.text_decoration_or_color_changed() {
            if self.HasFilter() && self.Filter().UsesCurrentColor() {
                diff.set_filter_changed(true);
            }
            if self.HasBackdropFilter() && self.BackdropFilter().UsesCurrentColor() {
                diff.set_compositing_reasons_changed(true);
            }
        }
        if changed(F::kScrollAnchor) || diff.transform_changed() {
            diff.set_disable_scroll_anchoring(true);
        }
        diff
    }
    fn WorkletPropertiesChanged(
        &self,
        other: &Self,
        properties: InvalidationProperties<'_>,
        context: &dyn VisualInvalidationDiffContext,
    ) -> bool {
        !context.NativePropertiesEqual(properties.native, self, other)
            || !context.CustomPropertiesEqual(properties.custom, self, other)
    }
    fn LayoutWorkletPropertiesChanged(
        &self,
        other: &Self,
        name: &AtomicString,
        child: bool,
        context: &dyn VisualInvalidationDiffContext,
    ) -> bool {
        match context.LayoutWorkletDefinition(name) {
            LayoutWorkletDefinition::FeatureDisabled
            | LayoutWorkletDefinition::NotRegistered
            | LayoutWorkletDefinition::Invalid => false,
            LayoutWorkletDefinition::UnsupportedProduct => {
                std::panic::panic_any(UnsupportedStyleInvalidationCapability::LayoutWorklet)
            }
            LayoutWorkletDefinition::Registered(definition) => self.WorkletPropertiesChanged(
                other,
                if child {
                    definition.child
                } else {
                    definition.own
                },
                context,
            ),
        }
    }
    fn DiffNeedsFullLayoutWithContext(
        &self,
        other: &Self,
        field_diff: u64,
        context: &dyn VisualInvalidationDiffContext,
    ) -> bool {
        if self.DiffNeedsFullLayoutBeforeWorklets(other, field_diff) {
            return true;
        }
        if self.IsDisplayLayoutCustom()
            && self.LayoutWorkletPropertiesChanged(
                other,
                self.DisplayLayoutCustomName(),
                false,
                context,
            )
        {
            return true;
        }
        if !self.DisplayLayoutCustomParentName().empty()
            && self.LayoutWorkletPropertiesChanged(
                other,
                self.DisplayLayoutCustomParentName(),
                true,
                context,
            )
        {
            return true;
        }
        self.DiffNeedsFullLayoutAfterWorklets(other, field_diff)
    }
    fn DiffNeedsNormalPaintInvalidationWithContext(
        &self,
        other: &Self,
        field_diff: u64,
        context: &dyn VisualInvalidationDiffContext,
    ) -> bool {
        let changed = |flag: F| field_diff & flag.bits() != 0;
        if changed(F::kPaint) {
            return true;
        }
        if changed(F::kAccentColor) && self.AccentColorResolved() != other.AccentColorResolved() {
            return true;
        }
        if changed(F::kOutline) && !self.OutlineVisuallyEqual(other) {
            return true;
        }
        if changed(F::kBackground)
            && !self
                .BackgroundInternal()
                .VisuallyEqual(other.BackgroundInternal())
        {
            return true;
        }
        if changed(F::kCurrentcolor)
            && (self.GetCurrentColor(None) != other.GetCurrentColor(None)
                || self.GetInternalVisitedCurrentColor(None)
                    != other.GetInternalVisitedCurrentColor(None))
            && self.HasPropertyDependingOnCurrentColor()
        {
            return true;
        }
        if changed(F::kBorderVisual) && !self.BorderVisuallyEqual(other) {
            return true;
        }
        if changed(F::kBorderOutlineVisitedColor) && self.BorderOutlineVisitedColorChanged(other) {
            return true;
        }
        if let Some(images) = unsafe { self.PaintImagesInternal().Get().as_ref() } {
            for image in images.Images() {
                // Source issue835589: linked paint targets bypass worklet property checks.
                if self.InsideLink() != foundation::EInsideLink::kNotInsideLink {
                    continue;
                }
                let image = unsafe { &*image.Get() };
                match context.PaintInvalidationProperties(image) {
                    PaintInvalidationProperties::Unpainted => return true,
                    PaintInvalidationProperties::UnsupportedProduct => {
                        std::panic::panic_any(UnsupportedStyleInvalidationCapability::PaintWorklet)
                    }
                    PaintInvalidationProperties::Registered(properties) => {
                        if self.WorkletPropertiesChanged(other, properties, context) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
    fn DiffCompositingReasonsWithContext(
        &self,
        other: &Self,
        field_diff: u64,
        context: &dyn VisualInvalidationDiffContext,
    ) -> bool {
        self.DiffNon3DCompositingReasonsChanged(other, field_diff)
            || context.Potential3DCompositingReasons(self)
                != context.Potential3DCompositingReasons(other)
    }
}
