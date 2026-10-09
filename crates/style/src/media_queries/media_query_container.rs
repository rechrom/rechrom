// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Typed collaborators for container style/anchored query evaluation.
//! All resolver operations are required: owning StyleCascade/StyleResolver and
//! container values supply their real computed values and conversion flags.
//! No declaration tokenization or substitute string comparison lives here.
use super::media_query_evaluator::{
    EvalStyleRange, MediaQueryEvaluationBackend, MediaQueryEvaluator,
};
use super::media_query_exp::{MediaQueryExp, MediaQueryExpValue};
use super::media_values::MediaValues;
use crate::css_numeric_literal_value::CSSNumericLiteralValue;
use crate::kleene_value::{KleeneAnd, KleeneValue};
use crate::resolver::media_query_result::MediaQueryResultFlags;
use foundation::AtomicString;
use std::rc::Rc;

/// Source CSSToLengthConversionData::Flag bits, not media-query UnitFlags.
// cpp: css_to_length_conversion_data.h:293-342.
pub struct StyleQueryConversionFlags;
impl StyleQueryConversionFlags {
    pub const EM: u32 = 1 << 0;
    pub const ROOT_FONT: u32 = 1 << 1;
    pub const GLYPH: u32 = 1 << 2;
    pub const VIEWPORT: u32 = 1 << 3;
    pub const SMALL_LARGE_VIEWPORT: u32 = 1 << 4;
    pub const DYNAMIC_VIEWPORT: u32 = 1 << 5;
    pub const CONTAINER: u32 = 1 << 6;
    pub const LINE_HEIGHT: u32 = 1 << 14;
    pub const ROOT_LINE_HEIGHT: u32 = 1 << 15;
    pub const SIBLING: u32 = 1 << 19;
}
// cpp: media_query_evaluator.cc:1745-1779.
pub fn ConversionFlagsToUnitFlags(flags: u32) -> u32 {
    use StyleQueryConversionFlags as F;
    let mut units = 0;
    for (conversion, unit) in [
        (F::EM | F::GLYPH, MediaQueryExpValue::<()>::kFontRelative),
        (
            F::ROOT_FONT | F::ROOT_LINE_HEIGHT,
            MediaQueryExpValue::<()>::kRootRelative,
        ),
        (
            F::DYNAMIC_VIEWPORT,
            MediaQueryExpValue::<()>::kDynamicViewport,
        ),
        (
            F::VIEWPORT | F::SMALL_LARGE_VIEWPORT,
            MediaQueryExpValue::<()>::kStaticViewport,
        ),
        (F::CONTAINER, MediaQueryExpValue::<()>::kContainer),
        (F::SIBLING, MediaQueryExpValue::<()>::kTreeCounting),
        (
            F::LINE_HEIGHT,
            MediaQueryExpValue::<()>::kLineHeightRelative,
        ),
    ] {
        if flags & conversion != 0 {
            units |= unit;
        }
    }
    units
}

/// The owner clones the container's ComputedStyle, updates line height, and
/// creates its CSSParserContext before returning this coercion context.
pub trait StyleRangeContext<V, U> {
    fn CoerceReference(&mut self, value: &U) -> Option<CSSNumericLiteralValue>;
    fn CoerceBound(&mut self, value: &V) -> Option<CSSNumericLiteralValue>;
    fn TakeLengthConversionFlags(&mut self) -> u32;
}
pub enum StyleQueryComputedValue<C, D> {
    Unparsed(Option<Rc<D>>),
    Registered(Option<Rc<C>>),
}
pub struct StyleQueryComputedResult<C, D> {
    pub value: StyleQueryComputedValue<C, D>,
    pub conversion_flags: u32,
    pub has_random: bool,
}
pub trait StyleQueryResolver<V, U> {
    type ComputedValue: PartialEq;
    type VariableData;
    type RangeContext: StyleRangeContext<V, U>;
    fn RangeQueriesEnabled(&self) -> bool;
    fn PrepareStyleRange(&self) -> Self::RangeContext;
    fn ReferenceHasRandomFunctions(&self, value: &U) -> bool;
    fn ValueHasRandomFunctions(&self, value: &V) -> bool;
    fn IsCascadeDependentKeyword(&self, value: &V) -> bool;
    /// None represents CSSInitialValue, as used by boolean style queries.
    fn ComputeValue(
        &self,
        property: &AtomicString,
        specified: Option<&V>,
    ) -> StyleQueryComputedResult<Self::ComputedValue, Self::VariableData>;
    fn ComputedVariableData(&self, property: &AtomicString) -> Option<Rc<Self::VariableData>>;
    fn VariableHasRandomFunctions(&self, data: &Self::VariableData) -> bool;
    fn EqualsIgnoringAttrTainting(&self, a: &Self::VariableData, b: &Self::VariableData) -> bool;
    fn ComputedPropertyValue(&self, property: &AtomicString) -> Option<Rc<Self::ComputedValue>>;
}
fn Value<V>(value: &MediaQueryExpValue<V>) -> &V {
    match value {
        MediaQueryExpValue::Value(value) => value,
        _ => unreachable!("style query requires its typed CSSValue"),
    }
}
fn Boolean(value: bool) -> KleeneValue {
    if value {
        KleeneValue::kTrue
    } else {
        KleeneValue::kFalse
    }
}

// cpp: media_query_evaluator.cc:1783-1926.
pub fn EvalStyleFeature<V, U, S: StyleQueryResolver<V, U>>(
    feature: &MediaQueryExp<V, U>,
    mut result_flags: Option<&mut MediaQueryResultFlags>,
    resolver: &S,
) -> KleeneValue {
    let bounds = feature.Bounds();
    if bounds.IsRange() {
        debug_assert!(feature.HasStyleRange());
        if !resolver.RangeQueriesEnabled() {
            return KleeneValue::kFalse;
        }
        let mut state = resolver.PrepareStyleRange();
        if resolver.ReferenceHasRandomFunctions(feature.ReferenceValue()) {
            return KleeneValue::kUnknown;
        }
        let Some(reference) = state.CoerceReference(feature.ReferenceValue()) else {
            return KleeneValue::kFalse;
        };
        let mut result = KleeneValue::kTrue;
        for (bound, reverse) in [(&bounds.left, true), (&bounds.right, false)] {
            if !bound.IsValid() {
                continue;
            }
            let specified = Value(&bound.value);
            if resolver.ValueHasRandomFunctions(specified) {
                return KleeneValue::kUnknown;
            }
            let Some(resolved) = state.CoerceBound(specified) else {
                return KleeneValue::kFalse;
            };
            result = KleeneAnd(
                result,
                EvalStyleRange(&reference, &resolved, bound.op, reverse),
            );
        }
        if let Some(flags) = result_flags.as_deref_mut() {
            flags.unit_flags |= ConversionFlagsToUnitFlags(state.TakeLengthConversionFlags());
        }
        return result;
    }
    debug_assert_eq!(
        bounds.right.op,
        super::media_query_exp::MediaQueryOperator::kNone
    );
    let property = feature.MediaFeature();
    let explicit = bounds.right.value.IsValid();
    let specified = explicit.then(|| Value(&bounds.right.value));
    if specified.is_some_and(|value| resolver.ValueHasRandomFunctions(value)) {
        return KleeneValue::kUnknown;
    }
    if specified.is_some_and(|value| resolver.IsCascadeDependentKeyword(value)) {
        return KleeneValue::kFalse;
    }
    let computed_query = resolver.ComputeValue(property, specified);
    if computed_query.has_random {
        return KleeneValue::kUnknown;
    }
    match computed_query.value {
        StyleQueryComputedValue::Unparsed(query) => {
            let computed = resolver.ComputedVariableData(property);
            if computed
                .as_deref()
                .is_some_and(|data| resolver.VariableHasRandomFunctions(data))
            {
                return KleeneValue::kUnknown;
            }
            Boolean(match (&computed, &query) {
                (None, None) => true,
                (Some(a), Some(b)) => Rc::ptr_eq(a, b) || resolver.EqualsIgnoringAttrTainting(a, b),
                _ => false,
            })
        }
        StyleQueryComputedValue::Registered(query) => {
            if let Some(flags) = result_flags {
                flags.unit_flags |= ConversionFlagsToUnitFlags(computed_query.conversion_flags);
            }
            Boolean((query == resolver.ComputedPropertyValue(property)) == explicit)
        }
    }
}

pub trait FallbackQueryResolver<V> {
    type Fallback: Clone;
    fn AnchoredFallback(&self) -> Self::Fallback;
    fn IsNone(&self, value: &Self::Fallback) -> bool;
    fn PositionAreaIsNone(&self, value: &Self::Fallback) -> bool;
    /// Uses the actual containing block and container writing directions.
    fn ToPhysicalPositionArea(&self, value: &Self::Fallback) -> Self::Fallback;
    fn ConvertSinglePositionTryFallback(
        &self,
        value: &V,
        allow_any_keyword_in_position_area: bool,
    ) -> Self::Fallback;
    fn Matches(&self, actual: &Self::Fallback, query: &Self::Fallback) -> bool;
}
// cpp: media_query_evaluator.cc:1570-1605.
pub fn EvalFallbackFeature<V, R: FallbackQueryResolver<V>>(
    value: &MediaQueryExpValue<V>,
    resolver: &R,
) -> bool {
    let fallback = resolver.AnchoredFallback();
    if !value.IsValid() {
        return !resolver.IsNone(&fallback);
    }
    if value.IsId() {
        assert_eq!(value.Id(), foundation::CSSValueID::kNone);
        return resolver.IsNone(&fallback);
    }
    if resolver.IsNone(&fallback) {
        return false;
    }
    let query = resolver.ConvertSinglePositionTryFallback(Value(value), true);
    let physical = |value: R::Fallback| {
        if resolver.PositionAreaIsNone(&value) {
            value
        } else {
            resolver.ToPhysicalPositionArea(&value)
        }
    };
    resolver.Matches(&physical(fallback), &physical(query))
}

/// Container evaluation overlays style/anchored features on the owner's
/// existing media backend. The supplied resolver is required and never replaced
/// by a default result when a container feature is observable.
pub struct ContainerQueryEvaluationBackend<'a, B, S> {
    backend: B,
    resolver: &'a S,
    values: &'a dyn MediaValues,
}
impl<B, S> MediaQueryEvaluationBackend for ContainerQueryEvaluationBackend<'_, B, S>
where
    B: MediaQueryEvaluationBackend<ResultFlags = MediaQueryResultFlags>,
    S: StyleQueryResolver<B::Value, B::UnparsedValue> + FallbackQueryResolver<B::Value>,
{
    type Value = B::Value;
    type UnparsedValue = B::UnparsedValue;
    type ResultFlags = MediaQueryResultFlags;
    type CustomMediaRulesMap = B::CustomMediaRulesMap;
    fn EvalFeature(
        &self,
        feature: &MediaQueryExp<Self::Value, Self::UnparsedValue>,
        flags: Option<&mut Self::ResultFlags>,
        custom_medias: Option<&Self::CustomMediaRulesMap>,
    ) -> KleeneValue {
        assert!(
            self.values.HasValues(),
            "container query requires MediaValues"
        );
        if feature.HasStyleRange()
            || feature.HasMediaFeature() && IsVariableName(feature.MediaFeature())
        {
            return EvalStyleFeature(feature, flags, self.resolver);
        }
        if feature.HasMediaFeature() && feature.MediaFeature() == "fallback" {
            if !self.values.HasAnchoredState() {
                return KleeneValue::kUnknown;
            }
            return Boolean(EvalFallbackFeature(
                &feature.Bounds().right.value,
                self.resolver,
            ));
        }
        self.backend.EvalFeature(feature, flags, custom_medias)
    }
}
fn IsVariableName(name: &AtomicString) -> bool {
    name.length() >= 3 && name.at(0) == 45 && name.at(1) == 45
}
impl<'a, B, S> MediaQueryEvaluator<'a, ContainerQueryEvaluationBackend<'a, B, S>>
where
    B: MediaQueryEvaluationBackend<ResultFlags = MediaQueryResultFlags>,
    S: StyleQueryResolver<B::Value, B::UnparsedValue> + FallbackQueryResolver<B::Value>,
{
    pub fn FromContainerValues(values: &'a dyn MediaValues, backend: B, resolver: &'a S) -> Self {
        Self::FromMediaValues(
            values,
            ContainerQueryEvaluationBackend {
                backend,
                resolver,
                values,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::media_query_exp::{
        MediaQueryExpBounds, MediaQueryExpComparison, MediaQueryOperator,
    };
    use super::*;
    use crate::css_primitive_value::UnitType;
    use std::cell::Cell;
    #[derive(Clone)]
    struct Specified {
        numeric: Option<CSSNumericLiteralValue>,
        random: bool,
        cascade: bool,
    }
    impl Specified {
        fn number(value: f64) -> Self {
            Self {
                numeric: Some(CSSNumericLiteralValue::Create(value, UnitType::kNumber)),
                random: false,
                cascade: false,
            }
        }
    }
    struct Data {
        content: u32,
        attr_taint: bool,
        random: bool,
    }
    struct RangeContext;
    impl StyleRangeContext<Specified, Specified> for RangeContext {
        fn CoerceReference(&mut self, value: &Specified) -> Option<CSSNumericLiteralValue> {
            value.numeric.clone()
        }
        fn CoerceBound(&mut self, value: &Specified) -> Option<CSSNumericLiteralValue> {
            value.numeric.clone()
        }
        fn TakeLengthConversionFlags(&mut self) -> u32 {
            StyleQueryConversionFlags::GLYPH | StyleQueryConversionFlags::CONTAINER
        }
    }
    struct Resolver {
        range_enabled: bool,
        unparsed: bool,
        query_data: Option<Rc<Data>>,
        computed_data: Option<Rc<Data>>,
        computed: f64,
        computed_random: bool,
        fallback: (bool, u32),
        physical_conversions: Cell<u32>,
    }
    impl Default for Resolver {
        fn default() -> Self {
            Self {
                range_enabled: true,
                unparsed: false,
                query_data: None,
                computed_data: None,
                computed: 5.0,
                computed_random: false,
                fallback: (true, 0),
                physical_conversions: Cell::new(0),
            }
        }
    }
    impl StyleQueryResolver<Specified, Specified> for Resolver {
        type ComputedValue = CSSNumericLiteralValue;
        type VariableData = Data;
        type RangeContext = RangeContext;
        fn RangeQueriesEnabled(&self) -> bool {
            self.range_enabled
        }
        fn PrepareStyleRange(&self) -> RangeContext {
            RangeContext
        }
        fn ReferenceHasRandomFunctions(&self, value: &Specified) -> bool {
            value.random
        }
        fn ValueHasRandomFunctions(&self, value: &Specified) -> bool {
            value.random
        }
        fn IsCascadeDependentKeyword(&self, value: &Specified) -> bool {
            value.cascade
        }
        fn ComputeValue(
            &self,
            _: &AtomicString,
            specified: Option<&Specified>,
        ) -> StyleQueryComputedResult<CSSNumericLiteralValue, Data> {
            StyleQueryComputedResult {
                value: if self.unparsed {
                    StyleQueryComputedValue::Unparsed(self.query_data.clone())
                } else {
                    StyleQueryComputedValue::Registered(Some(Rc::new(
                        specified
                            .and_then(|v| v.numeric.clone())
                            .unwrap_or_else(|| {
                                CSSNumericLiteralValue::Create(0.0, UnitType::kNumber)
                            }),
                    )))
                },
                conversion_flags: StyleQueryConversionFlags::ROOT_FONT,
                has_random: self.computed_random,
            }
        }
        fn ComputedVariableData(&self, _: &AtomicString) -> Option<Rc<Data>> {
            self.computed_data.clone()
        }
        fn VariableHasRandomFunctions(&self, data: &Data) -> bool {
            data.random
        }
        fn EqualsIgnoringAttrTainting(&self, a: &Data, b: &Data) -> bool {
            a.content == b.content
        }
        fn ComputedPropertyValue(&self, _: &AtomicString) -> Option<Rc<CSSNumericLiteralValue>> {
            Some(Rc::new(CSSNumericLiteralValue::Create(
                self.computed,
                UnitType::kNumber,
            )))
        }
    }
    impl FallbackQueryResolver<Specified> for Resolver {
        type Fallback = (bool, u32);
        fn AnchoredFallback(&self) -> Self::Fallback {
            self.fallback
        }
        fn IsNone(&self, value: &Self::Fallback) -> bool {
            value.0 && value.1 == 0
        }
        fn PositionAreaIsNone(&self, value: &Self::Fallback) -> bool {
            value.0
        }
        fn ToPhysicalPositionArea(&self, value: &Self::Fallback) -> Self::Fallback {
            self.physical_conversions
                .set(self.physical_conversions.get() + 1);
            (true, value.1)
        }
        fn ConvertSinglePositionTryFallback(
            &self,
            value: &Specified,
            allow_any: bool,
        ) -> Self::Fallback {
            assert!(allow_any);
            (false, value.numeric.as_ref().unwrap().DoubleValue() as u32)
        }
        fn Matches(&self, actual: &Self::Fallback, query: &Self::Fallback) -> bool {
            actual == query
        }
    }
    fn plain(value: Option<Specified>) -> MediaQueryExp<Specified> {
        let value = value.map_or(MediaQueryExpValue::Invalid, |v| {
            MediaQueryExpValue::FromValue(Rc::new(v))
        });
        MediaQueryExp::CreateWithBounds(
            &AtomicString::from_str("--property"),
            &MediaQueryExpBounds::FromRight(&MediaQueryExpComparison::FromValue(&value)),
        )
    }
    fn range(reference: Specified, left: Specified, right: Specified) -> MediaQueryExp<Specified> {
        MediaQueryExp::CreateStyleRange(
            Rc::new(reference),
            &MediaQueryExpBounds::new(
                &MediaQueryExpComparison::new(
                    &MediaQueryExpValue::FromValue(Rc::new(left)),
                    MediaQueryOperator::kLt,
                ),
                &MediaQueryExpComparison::new(
                    &MediaQueryExpValue::FromValue(Rc::new(right)),
                    MediaQueryOperator::kLe,
                ),
            ),
            true,
        )
        .unwrap()
    }
    #[test]
    fn style_ranges_coerce_both_sides_and_preserve_random_unknown_and_flags() {
        let resolver = Resolver::default();
        let mut flags = MediaQueryResultFlags::default();
        assert_eq!(
            EvalStyleFeature(
                &range(
                    Specified::number(5.0),
                    Specified::number(1.0),
                    Specified::number(5.0)
                ),
                Some(&mut flags),
                &resolver
            ),
            KleeneValue::kTrue
        );
        assert_eq!(
            flags.unit_flags,
            MediaQueryExpValue::<()>::kFontRelative | MediaQueryExpValue::<()>::kContainer
        );
        assert_eq!(
            EvalStyleFeature(
                &range(
                    Specified::number(0.0),
                    Specified::number(1.0),
                    Specified::number(5.0)
                ),
                None,
                &resolver
            ),
            KleeneValue::kFalse
        );
        let mut random = Specified::number(5.0);
        random.random = true;
        assert_eq!(
            EvalStyleFeature(
                &range(random, Specified::number(1.0), Specified::number(5.0)),
                None,
                &resolver
            ),
            KleeneValue::kUnknown
        );
        let mut random = Specified::number(1.0);
        random.random = true;
        assert_eq!(
            EvalStyleFeature(
                &range(Specified::number(5.0), random, Specified::number(5.0)),
                None,
                &resolver
            ),
            KleeneValue::kUnknown
        );
        let invalid = Specified {
            numeric: None,
            random: false,
            cascade: false,
        };
        assert_eq!(
            EvalStyleFeature(
                &range(invalid, Specified::number(1.0), Specified::number(5.0)),
                None,
                &resolver
            ),
            KleeneValue::kFalse
        );
        let disabled = Resolver {
            range_enabled: false,
            ..Default::default()
        };
        assert_eq!(
            EvalStyleFeature(
                &range(
                    Specified::number(5.0),
                    Specified::number(1.0),
                    Specified::number(5.0)
                ),
                None,
                &disabled
            ),
            KleeneValue::kFalse
        );
    }
    #[test]
    fn style_equality_registered_initial_unparsed_attr_taint_and_random_match_source() {
        let mut resolver = Resolver::default();
        let mut flags = MediaQueryResultFlags::default();
        assert_eq!(
            EvalStyleFeature(
                &plain(Some(Specified::number(5.0))),
                Some(&mut flags),
                &resolver
            ),
            KleeneValue::kTrue
        );
        assert_eq!(flags.unit_flags, MediaQueryExpValue::<()>::kRootRelative);
        assert_eq!(
            EvalStyleFeature(&plain(None), None, &resolver),
            KleeneValue::kTrue
        );
        resolver.computed = 0.0;
        assert_eq!(
            EvalStyleFeature(&plain(None), None, &resolver),
            KleeneValue::kFalse
        );
        let mut cascade = Specified::number(0.0);
        cascade.cascade = true;
        assert_eq!(
            EvalStyleFeature(&plain(Some(cascade)), None, &resolver),
            KleeneValue::kFalse
        );
        resolver.computed_random = true;
        assert_eq!(
            EvalStyleFeature(&plain(Some(Specified::number(0.0))), None, &resolver),
            KleeneValue::kUnknown
        );
        resolver.computed_random = false;
        resolver.unparsed = true;
        resolver.query_data = Some(Rc::new(Data {
            content: 7,
            attr_taint: false,
            random: false,
        }));
        resolver.computed_data = Some(Rc::new(Data {
            content: 7,
            attr_taint: true,
            random: false,
        }));
        assert_ne!(
            resolver.query_data.as_ref().unwrap().attr_taint,
            resolver.computed_data.as_ref().unwrap().attr_taint
        );
        flags = MediaQueryResultFlags::default();
        assert_eq!(
            EvalStyleFeature(
                &plain(Some(Specified::number(0.0))),
                Some(&mut flags),
                &resolver
            ),
            KleeneValue::kTrue
        );
        assert_eq!(flags.unit_flags, 0);
        resolver.computed_data = Some(Rc::new(Data {
            content: 7,
            attr_taint: false,
            random: true,
        }));
        assert_eq!(
            EvalStyleFeature(&plain(Some(Specified::number(0.0))), None, &resolver),
            KleeneValue::kUnknown
        );
        resolver.computed_data = None;
        assert_eq!(
            EvalStyleFeature(&plain(Some(Specified::number(0.0))), None, &resolver),
            KleeneValue::kFalse
        );
        resolver.query_data = None;
        assert_eq!(
            EvalStyleFeature(&plain(None), None, &resolver),
            KleeneValue::kTrue
        );
    }
    #[test]
    fn anchored_fallback_compares_physical_areas_and_none_boolean() {
        let resolver = Resolver {
            fallback: (false, 7),
            ..Default::default()
        };
        assert!(EvalFallbackFeature(&MediaQueryExpValue::Invalid, &resolver));
        assert!(!EvalFallbackFeature(
            &MediaQueryExpValue::FromId(foundation::CSSValueID::kNone),
            &resolver
        ));
        assert!(EvalFallbackFeature(
            &MediaQueryExpValue::FromValue(Rc::new(Specified::number(7.0))),
            &resolver
        ));
        assert_eq!(resolver.physical_conversions.get(), 2);
        assert!(!EvalFallbackFeature(
            &MediaQueryExpValue::FromValue(Rc::new(Specified::number(8.0))),
            &resolver
        ));
        let none = Resolver::default();
        assert!(!EvalFallbackFeature(&MediaQueryExpValue::Invalid, &none));
        assert!(EvalFallbackFeature(
            &MediaQueryExpValue::FromId(foundation::CSSValueID::kNone),
            &none
        ));
        assert!(!EvalFallbackFeature(
            &MediaQueryExpValue::FromValue(Rc::new(Specified::number(7.0))),
            &none
        ));
    }
    #[test]
    fn conversion_flags_map_only_source_dependencies() {
        use StyleQueryConversionFlags as F;
        for (flag, expected) in [
            (F::EM, MediaQueryExpValue::<()>::kFontRelative),
            (F::GLYPH, MediaQueryExpValue::<()>::kFontRelative),
            (F::ROOT_LINE_HEIGHT, MediaQueryExpValue::<()>::kRootRelative),
            (F::SIBLING, MediaQueryExpValue::<()>::kTreeCounting),
            (
                F::LINE_HEIGHT,
                MediaQueryExpValue::<()>::kLineHeightRelative,
            ),
            (1 << 7, 0),
            (1 << 20, 0),
        ] {
            assert_eq!(ConversionFlagsToUnitFlags(flag), expected);
        }
    }
}
