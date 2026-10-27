pub use style_resolve::media_queries::media_query_evaluator::*;

#[cfg(test)]
mod tests {
    use super::super::conditional_exp_node::ConditionalExpNode;
    use super::super::forced_colors::ForcedColors;
    use super::super::media_query_exp::MediaQueryOperator;
    use super::super::media_query_exp::{MediaQueryExp, MediaQueryExpValue};
    use super::super::media_values::MediaValues;
    use super::super::media_values::{
        CSSValueIDToForcedColors, CSSValueIDToPreferredColorScheme, CSSValueIDToPreferredContrast,
    };
    use super::super::media_values_cached::{MediaValuesCached, MediaValuesCachedData};
    use super::super::preferred_color_scheme::PreferredColorScheme;
    use super::super::preferred_contrast::PreferredContrast;
    use super::super::web_preferences::PointerType;
    use super::*;
    use crate::kleene_value::KleeneValue;
    use foundation::{CSSValueID, LayoutUnit, String};
    #[test]
    fn negative_query_rules_precision_and_nan_match_source() {
        use MediaQueryOperator::*;
        for op in [kNone, kEq, kLt, kLe, kGt, kGe] {
            assert_eq!(CompareValue(-20_i32, -10_i32, op), matches!(op, kGt | kGe));
            assert_eq!(
                CompareDoubleValue(-20.0, -10.0, op),
                matches!(op, kGt | kGe)
            );
            assert!(!CompareDoubleValue(f64::NAN, 10.0, op));
            assert!(!CompareDoubleValue(10.0, f64::NAN, op));
            assert_eq!(ReverseOperator(ReverseOperator(op)), op);
        }
        let epsilon = f64::from(LayoutUnit::Epsilon());
        assert!(CompareDoubleValue(10.0, 10.0 + epsilon, kEq));
        assert!(!CompareDoubleValue(10.0, 10.0 + epsilon * 2.0, kEq));
        assert!(CompareDoubleValue(10.0, 10.0 + epsilon, kGe));
        assert!(CompareDoubleValue(10.0, 10.0 - epsilon, kLe));
        assert!(CompareDoubleValue(10.0, 10.0 + epsilon / 2.0, kLt));
        assert!(!CompareDoubleValue(f64::INFINITY, f64::INFINITY, kEq));
        assert!(CompareValue(f64::INFINITY, f64::INFINITY, kEq));
    }
    #[test]
    fn color_scheme_boolean_and_identifier_queries_match_source() {
        let boolean = MediaQueryExpValue::<f64>::default();
        let numeric = MediaQueryExpValue::FromValue(std::rc::Rc::new(1.0));
        let ratio = MediaQueryExpValue::FromRatio(std::rc::Rc::new(1.0), std::rc::Rc::new(1.0));
        for scheme in [PreferredColorScheme::kDark, PreferredColorScheme::kLight] {
            let values = MediaValuesCached::new(&MediaValuesCachedData {
                preferred_color_scheme: scheme,
                ..Default::default()
            });
            assert!(PrefersColorSchemeMediaFeatureEval(
                &boolean,
                MediaQueryOperator::kNone,
                &values
            ));
            assert!(!PrefersColorSchemeMediaFeatureEval(
                &numeric,
                MediaQueryOperator::kNone,
                &values
            ));
            assert!(!PrefersColorSchemeMediaFeatureEval(
                &ratio,
                MediaQueryOperator::kNone,
                &values
            ));
            for id in [
                CSSValueID::kDark,
                CSSValueID::kLight,
                CSSValueID::kInvalid,
                CSSValueID::kNone,
            ] {
                let value = MediaQueryExpValue::<f64>::FromId(id);
                assert_eq!(
                    PrefersColorSchemeMediaFeatureEval(&value, MediaQueryOperator::kNone, &values),
                    (scheme == PreferredColorScheme::kDark && id == CSSValueID::kDark)
                        || (scheme == PreferredColorScheme::kLight && id == CSSValueID::kLight)
                );
            }
        }
        assert_eq!(PreferredColorScheme::kDark as i32, 0);
        assert_eq!(PreferredColorScheme::kLight as i32, 1);
        assert_eq!(
            CSSValueIDToPreferredColorScheme(CSSValueID::kLight),
            PreferredColorScheme::kLight
        );
        assert_eq!(
            CSSValueIDToPreferredContrast(CSSValueID::kCustom),
            PreferredContrast::kCustom
        );
        assert_eq!(
            CSSValueIDToForcedColors(CSSValueID::kActive),
            ForcedColors::kActive
        );
    }
    #[test]
    fn boolean_context_and_capability_masks_match_source() {
        let boolean = MediaQueryExpValue::<f64>::default();
        let values = MediaValuesCached::default();
        assert!(!PrefersContrastMediaFeatureEval(
            &boolean,
            MediaQueryOperator::kNone,
            &values
        ));
        assert!(!ForcedColorsMediaFeatureEval(
            &boolean,
            MediaQueryOperator::kNone,
            &values
        ));
        assert!(!PointerMediaFeatureEval(
            &boolean,
            MediaQueryOperator::kNone,
            &values
        ));
        assert!(!AnyPointerMediaFeatureEval(
            &boolean,
            MediaQueryOperator::kNone,
            &values
        ));
        assert!(!HoverMediaFeatureEval(
            &boolean,
            MediaQueryOperator::kNone,
            &values
        ));
        assert!(!AnyHoverMediaFeatureEval(
            &boolean,
            MediaQueryOperator::kNone,
            &values
        ));
        let values = MediaValuesCached::new(&MediaValuesCachedData {
            preferred_contrast: PreferredContrast::kMore,
            forced_colors: ForcedColors::kActive,
            prefers_reduced_motion: true,
            primary_pointer_type: PointerType::kPointerFineType,
            available_pointer_types: 1 | 2 | 4,
            available_hover_types: 1 | 2,
            ..Default::default()
        });
        assert!(PrefersContrastMediaFeatureEval(
            &boolean,
            MediaQueryOperator::kNone,
            &values
        ));
        assert!(ForcedColorsMediaFeatureEval(
            &boolean,
            MediaQueryOperator::kNone,
            &values
        ));
        assert!(PrefersReducedMotionMediaFeatureEval(
            &boolean,
            MediaQueryOperator::kNone,
            &values
        ));
        let no_preference = MediaQueryExpValue::<f64>::FromId(CSSValueID::kNoPreference);
        assert!(!PrefersReducedMotionMediaFeatureEval(
            &no_preference,
            MediaQueryOperator::kNone,
            &values
        ));
        for id in [CSSValueID::kNone, CSSValueID::kCoarse, CSSValueID::kFine] {
            assert!(AnyPointerMediaFeatureEval(
                &MediaQueryExpValue::<f64>::FromId(id),
                MediaQueryOperator::kNone,
                &values
            ));
        }
        for id in [CSSValueID::kNone, CSSValueID::kHover] {
            assert!(AnyHoverMediaFeatureEval(
                &MediaQueryExpValue::<f64>::FromId(id),
                MediaQueryOperator::kNone,
                &values
            ));
        }
    }
    struct ContainerValues {
        cached: MediaValuesCached,
        width: Option<f64>,
    }
    impl MediaValues for ContainerValues {
        type Document = dom::Document;
        fn GetDocument(&self) -> Option<&dom::Document> {
            None
        }
        fn DeviceWidth(&self) -> i32 {
            self.cached.DeviceWidth()
        }
        fn DeviceHeight(&self) -> i32 {
            self.cached.DeviceHeight()
        }
        fn DevicePixelRatio(&self) -> f32 {
            self.cached.DevicePixelRatio()
        }
        fn DeviceSupportsHDR(&self) -> bool {
            self.cached.DeviceSupportsHDR()
        }
        fn ColorBitsPerComponent(&self) -> i32 {
            self.cached.ColorBitsPerComponent()
        }
        fn MonochromeBitsPerComponent(&self) -> i32 {
            self.cached.MonochromeBitsPerComponent()
        }
        fn InvertedColors(&self) -> bool {
            self.cached.InvertedColors()
        }
        fn PrimaryPointerType(&self) -> super::super::web_preferences::PointerType {
            self.cached.PrimaryPointerType()
        }
        fn AvailablePointerTypes(&self) -> i32 {
            self.cached.AvailablePointerTypes()
        }
        fn PrimaryHoverType(&self) -> super::super::web_preferences::HoverType {
            self.cached.PrimaryHoverType()
        }
        fn OutputDeviceUpdateAbilityType(
            &self,
        ) -> super::super::web_preferences::OutputDeviceUpdateAbilityType {
            self.cached.OutputDeviceUpdateAbilityType()
        }
        fn AvailableHoverTypes(&self) -> i32 {
            self.cached.AvailableHoverTypes()
        }
        fn ThreeDEnabled(&self) -> bool {
            self.cached.ThreeDEnabled()
        }
        fn MediaType(&self) -> foundation::String {
            self.cached.MediaType()
        }
        fn DisplayMode(&self) -> super::super::display_mode::DisplayMode {
            self.cached.DisplayMode()
        }
        fn WindowShowState(&self) -> super::super::window_show_state::WindowShowState {
            self.cached.WindowShowState()
        }
        fn Resizable(&self) -> bool {
            self.cached.Resizable()
        }
        fn StrictMode(&self) -> bool {
            self.cached.StrictMode()
        }
        fn HasValues(&self) -> bool {
            self.cached.HasValues()
        }
        fn ColorGamut(&self) -> super::super::color_space_gamut::ColorSpaceGamut {
            self.cached.ColorGamut()
        }
        fn GetPreferredColorScheme(
            &self,
        ) -> super::super::preferred_color_scheme::PreferredColorScheme {
            self.cached.GetPreferredColorScheme()
        }
        fn GetPreferredContrast(&self) -> super::super::preferred_contrast::PreferredContrast {
            self.cached.GetPreferredContrast()
        }
        fn PrefersReducedMotion(&self) -> bool {
            self.cached.PrefersReducedMotion()
        }
        fn PrefersReducedData(&self) -> bool {
            self.cached.PrefersReducedData()
        }
        fn PrefersReducedTransparency(&self) -> bool {
            self.cached.PrefersReducedTransparency()
        }
        fn GetForcedColors(&self) -> super::super::forced_colors::ForcedColors {
            self.cached.GetForcedColors()
        }
        fn GetNavigationControls(&self) -> super::super::navigation_controls::NavigationControls {
            self.cached.GetNavigationControls()
        }
        fn GetHorizontalViewportSegments(&self) -> i32 {
            self.cached.GetHorizontalViewportSegments()
        }
        fn GetVerticalViewportSegments(&self) -> i32 {
            self.cached.GetVerticalViewportSegments()
        }
        fn GetDevicePosture(&self) -> super::super::device_posture_provider::DevicePostureType {
            self.cached.GetDevicePosture()
        }
        fn GetScripting(&self) -> super::super::scripting::Scripting {
            self.cached.GetScripting()
        }
        fn ViewportWidth(&self) -> f64 {
            self.cached.ViewportWidth()
        }
        fn ViewportHeight(&self) -> f64 {
            self.cached.ViewportHeight()
        }
        fn ComputeLength(&self, value: f64, unit: crate::css_primitive_value::UnitType) -> f64 {
            MediaValues::ComputeLength(&self.cached, value, unit)
        }
        fn GetWritingMode(&self) -> foundation::WritingMode {
            foundation::WritingMode::kVerticalRl
        }
        fn Width(&self) -> Option<f64> {
            self.width
        }
        fn Height(&self) -> Option<f64> {
            Some(80.0)
        }
        fn StuckHorizontal(&self) -> super::super::container_state::ContainerStuckPhysical {
            super::super::container_state::ContainerStuckPhysical::kRight
        }
        fn StuckVertical(&self) -> super::super::container_state::ContainerStuckPhysical {
            super::super::container_state::ContainerStuckPhysical::kTop
        }
        fn StuckInline(&self) -> super::super::container_state::ContainerStuckLogical {
            super::super::container_state::ContainerStuckLogical::kStart
        }
        fn StuckBlock(&self) -> super::super::container_state::ContainerStuckLogical {
            super::super::container_state::ContainerStuckLogical::kEnd
        }
        fn SnappedFlags(&self) -> u32 {
            1
        }
        fn ScrollableHorizontal(&self) -> u32 {
            2
        }
        fn ScrollableVertical(&self) -> u32 {
            1
        }
        fn ScrollableInline(&self) -> u32 {
            1
        }
        fn ScrollableBlock(&self) -> u32 {
            2
        }
        fn ScrolledHorizontal(&self) -> super::super::container_state::ContainerScrolled {
            super::super::container_state::ContainerScrolled::kEnd
        }
        fn ScrolledVertical(&self) -> super::super::container_state::ContainerScrolled {
            super::super::container_state::ContainerScrolled::kStart
        }
        fn ScrolledInline(&self) -> super::super::container_state::ContainerScrolled {
            super::super::container_state::ContainerScrolled::kStart
        }
        fn ScrolledBlock(&self) -> super::super::container_state::ContainerScrolled {
            super::super::container_state::ContainerScrolled::kEnd
        }
    }
    #[test]
    fn typed_container_axes_state_and_missing_dimensions_match_source() {
        use super::super::media_query_exp::{MediaQueryExpBounds, MediaQueryExpComparison};
        use crate::css_numeric_literal_value::CSSNumericLiteralValue;
        use crate::css_primitive_value::UnitType;
        let feature = |name: &str, value: MediaQueryExpValue<CSSNumericLiteralValue>| {
            ConditionalExpNode::Feature(MediaQueryExp::CreateWithBounds(
                &foundation::AtomicString::from_str(name),
                &MediaQueryExpBounds::FromRight(&MediaQueryExpComparison::FromValue(&value)),
            ))
        };
        let values = ContainerValues {
            cached: MediaValuesCached::default(),
            width: Some(50.0),
        };
        let evaluator = crate::media_queries::media_query_backend::ForMediaValues(&values);
        let px = |v| {
            MediaQueryExpValue::FromValue(std::rc::Rc::new(CSSNumericLiteralValue::Create(
                v,
                UnitType::kPixels,
            )))
        };
        assert_eq!(
            evaluator.EvalNode(&feature("inline-size", px(80.0)), None, None),
            KleeneValue::kTrue
        );
        assert_eq!(
            evaluator.EvalNode(&feature("block-size", px(50.0)), None, None),
            KleeneValue::kTrue
        );
        for (name, id, expected) in [
            ("stuck", CSSValueID::kTop, true),
            ("stuck", CSSValueID::kBottom, false),
            ("stuck", CSSValueID::kBlockEnd, true),
            ("snapped", CSSValueID::kBlock, true),
            ("snapped", CSSValueID::kInline, false),
            ("snapped", CSSValueID::kBoth, false),
            ("scrollable", CSSValueID::kRight, true),
            ("scrollable", CSSValueID::kLeft, false),
            ("scrollable", CSSValueID::kInlineStart, true),
            ("scrolled", CSSValueID::kTop, true),
            ("scrolled", CSSValueID::kBlockEnd, true),
            ("scrolled", CSSValueID::kBottom, false),
        ] {
            assert_eq!(
                evaluator.EvalNode(&feature(name, MediaQueryExpValue::FromId(id)), None, None),
                if expected {
                    KleeneValue::kTrue
                } else {
                    KleeneValue::kFalse
                },
                "{name}: {id:?}"
            );
        }
        for name in ["stuck", "snapped", "scrollable", "scrolled"] {
            assert_eq!(
                evaluator.EvalNode(&feature(name, MediaQueryExpValue::Invalid), None, None),
                KleeneValue::kTrue
            );
        }
        let absent = ContainerValues {
            cached: MediaValuesCached::default(),
            width: None,
        };
        assert_eq!(
            crate::media_queries::media_query_backend::ForMediaValues(&absent).EvalNode(
                &feature("block-size", px(50.0)),
                None,
                None
            ),
            KleeneValue::kUnknown
        );
        assert_eq!(
            crate::media_queries::media_query_backend::ForMediaValues(&absent).EvalNode(
                &feature("inline-size", px(80.0)),
                None,
                None
            ),
            KleeneValue::kTrue
        );
    }
    #[test]
    fn style_numeric_type_zero_coercion_and_reverse_comparisons_match_source() {
        use crate::css_numeric_literal_value::CSSNumericLiteralValue as N;
        use crate::css_primitive_value::UnitType::*;
        use MediaQueryOperator::*;
        assert_eq!(
            EvalStyleRange(
                &N::Create(0.0, kNumber),
                &N::Create(1.0, kPixels),
                kLt,
                false
            ),
            KleeneValue::kTrue
        );
        assert_eq!(
            EvalStyleRange(
                &N::Create(1.0, kPixels),
                &N::Create(0.0, kNumber),
                kLt,
                true
            ),
            KleeneValue::kTrue
        );
        assert_eq!(
            EvalStyleRange(
                &N::Create(1.0, kNumber),
                &N::Create(1.0, kPixels),
                kEq,
                false
            ),
            KleeneValue::kFalse
        );
        for unit in [kPercentage, kDegrees, kSeconds, kDotsPerPixel] {
            assert_eq!(
                EvalStyleRange(&N::Create(1.0, unit), &N::Create(1.0, unit), kEq, false),
                KleeneValue::kTrue
            );
        }
    }
    struct TypeOnlyBackend;
    impl MediaQueryEvaluationBackend for TypeOnlyBackend {
        type Document = dom::Document;
        type Value = ();
        type UnparsedValue = ();
        type ResultFlags = ();
        type CustomMediaRulesMap = ();
        fn EvalFeature(
            &self,
            _: &MediaQueryExp<()>,
            _: Option<&mut ()>,
            _: Option<&()>,
        ) -> KleeneValue {
            unreachable!("type-only constructor must not request feature reads")
        }
    }
    struct FrameValues(Option<MediaValuesCached>);
    impl MediaQueryFrameValues for FrameValues {
        type Document = dom::Document;
        fn CreateDynamicIfFrameExists(&self) -> Option<&dyn MediaValues<Document = dom::Document>> {
            self.0
                .as_ref()
                .map(|values| values as &dyn MediaValues<Document = dom::Document>)
        }
    }
    #[test]
    fn constructors_preserve_type_only_dynamic_and_absent_frame_reads() {
        let type_only = MediaQueryEvaluator::FromMediaType(String::from("SCREEN"), TypeOnlyBackend);
        assert!(type_only.MediaTypeMatch(&String::from("screen")));
        assert!(type_only.GetDocument().is_none());
        let frame = FrameValues(Some(MediaValuesCached::new(&MediaValuesCachedData {
            media_type: String::from("print"),
            ..Default::default()
        })));
        let dynamic = MediaQueryEvaluator::FromFrame(&frame, TypeOnlyBackend);
        assert!(dynamic.MediaTypeMatch(&String::from("PRINT")));
        assert!(std::ptr::eq(
            dynamic.GetMediaValues(),
            frame.CreateDynamicIfFrameExists().unwrap()
        ));
        let absent = FrameValues(None);
        let evaluator = MediaQueryEvaluator::FromFrame(&absent, TypeOnlyBackend);
        assert!(evaluator.MediaType().empty());
        assert!(evaluator.GetDocument().is_none());
        assert!(!evaluator.MediaTypeMatch(&String::from("screen")));
        assert!(evaluator.MediaTypeMatch(&String::from("all")));
    }
}
