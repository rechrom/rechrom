//! Direct compositing reasons used by Paint property nodes.
//!
//! Source: Chromium revision 6c1d401fcca5e1b0030563a90c2f2bba168e0c15,
//! `third_party/blink/renderer/core/paint/compositing/compositing_reason_finder.{h,cc}`.

use foundation::{CompositingReason, CompositingReasonCombos, CompositingReasons};
use layoutng_assembly::fragment_tree::{FragmentKind, FragmentPaintData};
use layoutng_assembly::internal::layout_input::{NodeKind, PaintStyle};
use paint_model::paint_property_tree::PaintPropertyRole;

pub struct CompositingReasonFinder;

impl CompositingReasonFinder {
    /// Source: `compositing_reason_finder.cc:76-141`.
    fn CompositingReasonsForWillChange(style: &PaintStyle) -> CompositingReasons {
        let mut reasons = CompositingReasons::new();
        if style.subtree_will_change_contents() {
            return reasons;
        }
        for (enabled, reason) in [
            (
                style.will_change_backdrop_filter(),
                CompositingReason::kWillChangeBackdropFilter,
            ),
            (
                style.will_change_clip_path(),
                CompositingReason::kWillChangeClipPath,
            ),
            (
                style.will_change_filter(),
                CompositingReason::kWillChangeFilter,
            ),
            (style.will_change_mask(), CompositingReason::kWillChangeMask),
            (
                style.will_change_mix_blend_mode(),
                CompositingReason::kWillChangeMixBlendMode,
            ),
            (
                style.will_change_opacity(),
                CompositingReason::kWillChangeOpacity,
            ),
            (
                style.will_change_rotate(),
                CompositingReason::kWillChangeRotate,
            ),
            (
                style.will_change_scale(),
                CompositingReason::kWillChangeScale,
            ),
            (
                style.will_change_translate(),
                CompositingReason::kWillChangeTranslate,
            ),
            (
                style.will_change_transform_property()
                    || style.will_change_perspective()
                    || style.will_change_transform_style(),
                CompositingReason::kWillChangeTransform,
            ),
        ] {
            if enabled {
                reasons.Put(reason);
            }
        }
        if reasons.IsEmpty() && style.will_change_other() {
            reasons.Put(CompositingReason::kWillChangeOther);
        }
        reasons
    }

    /// Source: `compositing_reason_finder.cc:338-348,570-609`.
    fn CompositingReasonsForAnimation(fragment: &FragmentPaintData) -> CompositingReasons {
        let style = &fragment.paint.style;
        let mut reasons = CompositingReasons::new();
        if style.subtree_will_change_contents() {
            return reasons;
        }
        let is_svg_child = matches!(
            fragment.paint.source_kind,
            NodeKind::kSvgGroup
                | NodeKind::kSvgHiddenContainer
                | NodeKind::kSvgResourceClipper
                | NodeKind::kSvgResourceMasker
                | NodeKind::kSvgForeignObject
                | NodeKind::kSvgText
                | NodeKind::kSvgInline
                | NodeKind::kSvgTSpan
                | NodeKind::kSvgTextPath
                | NodeKind::kSvgShape
        );
        let supports_transform_animation = if is_svg_child {
            !fragment.paint.svg_hidden_container
                && !fragment.paint.IsLayoutInline()
                && fragment.kind == FragmentKind::kBox
        } else {
            fragment.kind == FragmentKind::kBox
        };
        for (enabled, reason) in [
            (
                style.active_transform_animation() && supports_transform_animation,
                CompositingReason::kActiveTransformAnimation,
            ),
            (
                style.active_scale_animation() && supports_transform_animation,
                CompositingReason::kActiveScaleAnimation,
            ),
            (
                style.active_rotate_animation() && supports_transform_animation,
                CompositingReason::kActiveRotateAnimation,
            ),
            (
                style.active_translate_animation() && supports_transform_animation,
                CompositingReason::kActiveTranslateAnimation,
            ),
            (
                style.active_opacity_animation(),
                CompositingReason::kActiveOpacityAnimation,
            ),
            (
                style.active_filter_animation(),
                CompositingReason::kActiveFilterAnimation,
            ),
            (
                style.active_backdrop_filter_animation(),
                CompositingReason::kActiveBackdropFilterAnimation,
            ),
        ] {
            if enabled {
                reasons.Put(reason);
            }
        }
        reasons
    }

    /// Source: `compositing_reason_finder.cc:541-568`.
    pub fn PotentialCompositingReasonsFor3DTransform(style: &PaintStyle) -> CompositingReasons {
        let mut reasons = CompositingReasons::new();
        if style.has_nontrivial_3d_transform() {
            reasons.Put(CompositingReason::k3DTransform);
        } else if style.has_trivial_3d_transform() {
            reasons.Put(CompositingReason::kTrivial3DTransform);
        }
        if style.has_3d_translate() {
            reasons.Put(CompositingReason::k3DTranslate);
        }
        if style.has_3d_rotate() {
            reasons.Put(CompositingReason::k3DRotate);
        }
        if style.has_3d_scale() {
            reasons.Put(CompositingReason::k3DScale);
        }
        reasons
    }

    /// Supported portion of
    /// `CompositingReasonFinder::DirectReasonsForPaintProperties`, source
    /// `compositing_reason_finder.cc:380-502`.
    ///
    /// Frame-tree, ViewTransition, ElementCapture, PaintLayer 3D-descendant
    /// and platform overscroll branches remain unsupported at the Layout/Paint
    /// boundary and therefore contribute no guessed reasons.
    pub fn DirectReasonsForPaintProperties(fragment: &FragmentPaintData) -> CompositingReasons {
        if fragment.paint.IsPrinting() {
            return CompositingReasons::new();
        }
        let style = &fragment.paint.style;
        let mut reasons = Self::CompositingReasonsForAnimation(fragment);
        reasons.PutAll(Self::CompositingReasonsForWillChange(style));
        reasons.PutAll(Self::PotentialCompositingReasonsFor3DTransform(style));
        reasons.PutAll(fragment.paint.AdditionalCompositingReasons());
        if fragment.paint.IsGlobalRootScroller() {
            reasons.Put(CompositingReason::kRootScroller);
        }
        if fragment.paint.sticky_constraint.is_some() {
            reasons.Put(CompositingReason::kStickyPosition);
        }
        if fragment.paint.needs_anchor_position_scroll_adjustment {
            reasons.Put(CompositingReason::kAnchorPosition);
        }
        if style.has_backdrop_filters() {
            reasons.Put(CompositingReason::kBackdropFilter);
        }
        reasons
    }

    pub fn TransformReasons(
        fragment: &FragmentPaintData,
        role: PaintPropertyRole,
        has_sticky_constraint: bool,
    ) -> CompositingReasons {
        if role == PaintPropertyRole::StickyTranslation {
            return has_sticky_constraint
                .then(|| CompositingReasons::FromReason(CompositingReason::kStickyPosition))
                .unwrap_or_default();
        }
        let direct = Self::DirectReasonsForPaintProperties(fragment);
        let accepted = match role {
            PaintPropertyRole::PaintOffsetTranslation => {
                CompositingReasonCombos::kDirectReasonsForPaintOffsetTranslationProperty
            }
            PaintPropertyRole::Translate => {
                CompositingReasonCombos::kDirectReasonsForTranslateProperty
            }
            PaintPropertyRole::Rotate => CompositingReasonCombos::kDirectReasonsForRotateProperty,
            PaintPropertyRole::Scale => CompositingReasonCombos::kDirectReasonsForScaleProperty,
            PaintPropertyRole::Transform | PaintPropertyRole::SvgLocalTransform => {
                CompositingReasonCombos::kDirectReasonsForTransformProperty
            }
            PaintPropertyRole::ScrollTranslation => {
                CompositingReasonCombos::kDirectReasonsForScrollTranslationProperty
            }
            _ => CompositingReasons::new(),
        };
        direct.Intersection(accepted)
    }

    pub fn EffectReasons(fragment: &FragmentPaintData) -> CompositingReasons {
        Self::DirectReasonsForPaintProperties(fragment)
            .Intersection(CompositingReasonCombos::kDirectReasonsForEffectProperty)
    }

    pub fn FilterReasons(fragment: &FragmentPaintData) -> CompositingReasons {
        Self::DirectReasonsForPaintProperties(fragment)
            .Intersection(CompositingReasonCombos::kDirectReasonsForFilterProperty)
    }

    pub fn AdditionalEffectCompositingTrigger(fragment: &FragmentPaintData) -> CompositingReasons {
        Self::DirectReasonsForPaintProperties(fragment)
            .Intersection(CompositingReasonCombos::kAdditionalEffectCompositingTrigger)
    }

    /// Source: `compositing_reason_finder.cc:504-539`.
    pub fn ShouldForcePreferCompositingToLCDText(
        fragment: &FragmentPaintData,
        reasons_except_scrolling: CompositingReasons,
    ) -> bool {
        if fragment.paint.CanvasDrawElementEnabled() && fragment.paint.IsInCanvasSubtree() {
            return false;
        }
        if !reasons_except_scrolling.IsEmpty() {
            return true;
        }
        let style = &fragment.paint.style;
        if style.will_change_scroll_position() || style.backface_visibility_hidden() {
            return true;
        }
        // Every Page is an independent local root in Rechrom. The source's
        // owner/document/body fallback is only needed for embedded frame trees.
        fragment.paint.IsLayoutView()
    }

    /// Overlay scrollbar opacity is always compositor animated.
    /// Source: `paint_property_tree_builder.cc:3706-3726`.
    pub fn ActiveOpacityAnimation() -> CompositingReasons {
        CompositingReasons::FromReason(CompositingReason::kActiveOpacityAnimation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn property_roles_receive_only_their_source_reason_subset() {
        let mut fragment = FragmentPaintData::default();
        let style = &mut *fragment.paint.style;
        style.will_change_transform_property = true;
        style.will_change_translate = true;
        style.will_change_rotate = true;
        style.will_change_scale = true;
        style.will_change_opacity = true;
        style.active_filter_animation = true;

        let transform = CompositingReasonFinder::TransformReasons(
            &fragment,
            PaintPropertyRole::Transform,
            false,
        );
        assert!(transform.Has(CompositingReason::kWillChangeTransform));
        assert!(!transform.Has(CompositingReason::kWillChangeOpacity));

        let translate = CompositingReasonFinder::TransformReasons(
            &fragment,
            PaintPropertyRole::Translate,
            false,
        );
        assert!(translate.Has(CompositingReason::kWillChangeTranslate));
        assert!(!translate.Has(CompositingReason::kWillChangeRotate));
        let rotate =
            CompositingReasonFinder::TransformReasons(&fragment, PaintPropertyRole::Rotate, false);
        assert!(rotate.Has(CompositingReason::kWillChangeRotate));
        let scale =
            CompositingReasonFinder::TransformReasons(&fragment, PaintPropertyRole::Scale, false);
        assert!(scale.Has(CompositingReason::kWillChangeScale));

        let effect = CompositingReasonFinder::EffectReasons(&fragment);
        assert!(effect.Has(CompositingReason::kWillChangeOpacity));
        assert!(!effect.Has(CompositingReason::kWillChangeTransform));

        let filter = CompositingReasonFinder::FilterReasons(&fragment);
        assert!(filter.Has(CompositingReason::kActiveFilterAnimation));

        fragment
            .paint
            .additional_compositing_reasons
            .Put(CompositingReason::kOverflowScrolling);
        let scroll = CompositingReasonFinder::TransformReasons(
            &fragment,
            PaintPropertyRole::ScrollTranslation,
            false,
        );
        assert!(scroll.Has(CompositingReason::kOverflowScrolling));
    }

    #[test]
    fn lcd_preference_honors_canvas_and_scroll_hints() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.canvas_draw_element_enabled = true;
        fragment.paint.is_in_canvas_subtree = true;
        assert!(
            !CompositingReasonFinder::ShouldForcePreferCompositingToLCDText(
                &fragment,
                CompositingReasons::FromReason(CompositingReason::kWillChangeOpacity),
            )
        );

        fragment.paint.is_in_canvas_subtree = false;
        fragment.paint.style.will_change_scroll_position = true;
        assert!(
            CompositingReasonFinder::ShouldForcePreferCompositingToLCDText(
                &fragment,
                CompositingReasons::new(),
            )
        );
    }
}
