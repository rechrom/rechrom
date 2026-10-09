// Copyright 2020 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// cpp: third_party/blink/renderer/core/css/style_rule_counter_style.h/.cc
// GC tracing/destruction, export and downcast boilerplate use Rust ownership.

#![allow(non_snake_case, non_camel_case_types)]

use crate::css_property_names::CSSPropertyID;
use crate::css_property_value_set::{
    CSSPropertyValueSetBackend, CSSPropertyValueSetRuleAdapter, CSSPropertyValueSetRuleHandle,
    MutableCSSPropertyValueSet,
};
use crate::css_value::{CSSValue, CSSValueListSubclass, CSSValuePayload};
use crate::parser::at_rule_descriptors::{AtRuleDescriptorID, AtRuleDescriptorIDAsCSSPropertyID};
use crate::style_rule::RuleType;
use foundation::AtomicString;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

// cpp: counter_style.h:25-44. The dependency's complete system domain.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CounterStyleSystem {
    kCyclic,
    kFixed,
    kSymbolic,
    kAlphabetic,
    kNumeric,
    kAdditive,
    kHebrew,
    kSimpChineseInformal,
    kSimpChineseFormal,
    kTradChineseInformal,
    kTradChineseFormal,
    kKoreanHangulFormal,
    kKoreanHanjaInformal,
    kKoreanHanjaFormal,
    kLowerArmenian,
    kUpperArmenian,
    kEthiopicNumeric,
    kUnresolvedExtends,
}

// Actual CounterStyle conversion and CSSValueList::length are dependencies of
// this source pair. Both are required; the rule supplies no default algorithm.
pub trait StyleRuleCounterStyleBackend: CSSPropertyValueSetBackend {
    fn ToCounterStyleSystemEnum(value: Option<&CSSValue<Self>>) -> CounterStyleSystem;
    fn CSSValueListLength(value: &Self::CSSValueList) -> usize;
}

// cpp: style_rule_counter_style.h:14-77
pub struct StyleRuleCounterStyle<D: StyleRuleCounterStyleBackend> {
    name_: AtomicString,
    version_: Cell<i32>,
    properties_: RefCell<Rc<CSSPropertyValueSetRuleHandle<D>>>,
}
impl<D: StyleRuleCounterStyleBackend> Clone for StyleRuleCounterStyle<D> {
    // cpp: style_rule_counter_style.cc:22-23. Copy the property reference, not
    // the property store, and retain the source version value.
    fn clone(&self) -> Self {
        Self {
            name_: self.name_.clone(),
            version_: Cell::new(self.version_.get()),
            properties_: RefCell::new(self.properties_.borrow().clone()),
        }
    }
}
impl<D: StyleRuleCounterStyleBackend> StyleRuleCounterStyle<D> {
    // cpp: style_rule_counter_style.cc:16-20
    pub fn new(name: AtomicString, properties: Rc<CSSPropertyValueSetRuleHandle<D>>) -> Self {
        Self {
            name_: name,
            version_: Cell::new(0),
            properties_: RefCell::new(properties),
        }
    }
    pub fn GetType(&self) -> RuleType {
        RuleType::kCounterStyle
    }
    // cpp: style_rule_counter_style.h:20,30
    pub fn GetVersion(&self) -> i32 {
        self.version_.get()
    }
    pub fn GetName(&self) -> AtomicString {
        self.name_.clone()
    }

    // cpp: style_rule_counter_style.cc:27-56
    pub fn GetSystem(&self) -> Option<Rc<CSSValue<D>>> {
        self.GetDescriptor(CSSPropertyID::kSystem)
    }
    pub fn GetNegative(&self) -> Option<Rc<CSSValue<D>>> {
        self.GetDescriptor(CSSPropertyID::kNegative)
    }
    pub fn GetPrefix(&self) -> Option<Rc<CSSValue<D>>> {
        self.GetDescriptor(CSSPropertyID::kPrefix)
    }
    pub fn GetSuffix(&self) -> Option<Rc<CSSValue<D>>> {
        self.GetDescriptor(CSSPropertyID::kSuffix)
    }
    pub fn GetRange(&self) -> Option<Rc<CSSValue<D>>> {
        self.GetDescriptor(CSSPropertyID::kRange)
    }
    pub fn GetPad(&self) -> Option<Rc<CSSValue<D>>> {
        self.GetDescriptor(CSSPropertyID::kPad)
    }
    pub fn GetFallback(&self) -> Option<Rc<CSSValue<D>>> {
        self.GetDescriptor(CSSPropertyID::kFallback)
    }
    pub fn GetSymbols(&self) -> Option<Rc<CSSValue<D>>> {
        self.GetDescriptor(CSSPropertyID::kSymbols)
    }
    pub fn GetAdditiveSymbols(&self) -> Option<Rc<CSSValue<D>>> {
        self.GetDescriptor(CSSPropertyID::kAdditiveSymbols)
    }
    pub fn GetSpeakAs(&self) -> Option<Rc<CSSValue<D>>> {
        self.GetDescriptor(CSSPropertyID::kSpeakAs)
    }
    fn GetDescriptor(&self, id: CSSPropertyID) -> Option<Rc<CSSValue<D>>> {
        self.properties_.borrow().GetPropertyCSSValue(id)
    }

    // cpp: style_rule_counter_style.cc:77-87. Properties() alone does not
    // invalidate layout; requesting MutableStyleForInspector() always does.
    pub fn MutableStyleForInspector(&self) -> Rc<RefCell<MutableCSSPropertyValueSet<D>>> {
        self.version_.set(self.version_.get() + 1);
        self.Properties()
    }
    pub fn Properties(&self) -> Rc<RefCell<MutableCSSPropertyValueSet<D>>> {
        let current = self.properties_.borrow().clone();
        if !current.IsMutable() {
            *self.properties_.borrow_mut() =
                CSSPropertyValueSetRuleHandle::FromMutable(current.MutableCopy());
        }
        self.properties_
            .borrow()
            .MutablePropertySet()
            .expect("Properties converted the source property store to mutable")
    }

    // cpp: style_rule_counter_style.cc:89-91
    pub fn HasValidSymbols(&self) -> bool {
        Self::HasValidSymbolsForValues(
            self.GetSystem().as_deref(),
            self.GetSymbols().as_deref(),
            self.GetAdditiveSymbols().as_deref(),
        )
    }
    // cpp: style_rule_counter_style.cc:93-126. Rust names distinguish the
    // source's static and instance HasValidSymbols overloads.
    pub fn HasValidSymbolsForValues(
        system_value: Option<&CSSValue<D>>,
        symbols_value: Option<&CSSValue<D>>,
        additive_symbols_value: Option<&CSSValue<D>>,
    ) -> bool {
        let system = D::ToCounterStyleSystemEnum(system_value);
        let symbols = symbols_value.map(Self::ValueList);
        let additive_symbols = additive_symbols_value.map(Self::ValueList);
        match system {
            CounterStyleSystem::kCyclic
            | CounterStyleSystem::kFixed
            | CounterStyleSystem::kSymbolic => {
                symbols.is_some_and(|list| D::CSSValueListLength(list) != 0)
            }
            CounterStyleSystem::kAlphabetic | CounterStyleSystem::kNumeric => {
                symbols.is_some_and(|list| D::CSSValueListLength(list) > 1)
            }
            CounterStyleSystem::kAdditive => {
                additive_symbols.is_some_and(|list| D::CSSValueListLength(list) != 0)
            }
            CounterStyleSystem::kUnresolvedExtends => {
                symbols.is_none() && additive_symbols.is_none()
            }
            CounterStyleSystem::kHebrew
            | CounterStyleSystem::kSimpChineseInformal
            | CounterStyleSystem::kSimpChineseFormal
            | CounterStyleSystem::kTradChineseInformal
            | CounterStyleSystem::kTradChineseFormal
            | CounterStyleSystem::kKoreanHangulFormal
            | CounterStyleSystem::kKoreanHanjaInformal
            | CounterStyleSystem::kKoreanHanjaFormal
            | CounterStyleSystem::kLowerArmenian
            | CounterStyleSystem::kUpperArmenian
            | CounterStyleSystem::kEthiopicNumeric => true,
        }
    }
    // To<CSSValueList> in the source accepts list subclasses and requires the
    // list cast to succeed for non-null values, even for a predefined system.
    fn ValueList(value: &CSSValue<D>) -> &D::CSSValueList {
        match value.Payload() {
            CSSValuePayload::kValueListClass(list) => list,
            CSSValuePayload::kFunctionClass(value) => value.AsValueList(),
            CSSValuePayload::kImageSetClass(value) => value.AsValueList(),
            CSSValuePayload::kGridLineNamesClass(value) => value.AsValueList(),
            CSSValuePayload::kGridAutoRepeatClass(value) => value.AsValueList(),
            CSSValuePayload::kGridIntegerRepeatClass(value) => value.AsValueList(),
            CSSValuePayload::kAxisClass(value) => value.AsValueList(),
            _ => panic!("To<CSSValueList> requires a list value"),
        }
    }

    // cpp: style_rule_counter_style.cc:128-150. Despite the method's name,
    // the source returns true only for an unequal, acceptable replacement.
    pub fn NewValueInvalidOrEqual(
        &self,
        descriptor_id: AtRuleDescriptorID,
        new_value: Option<&CSSValue<D>>,
    ) -> bool {
        let original_value = self.GetDescriptor(AtRuleDescriptorIDAsCSSPropertyID(descriptor_id));
        if ValuesEquivalent(original_value.as_deref(), new_value) {
            return false;
        }
        match descriptor_id {
            AtRuleDescriptorID::System => {
                D::ToCounterStyleSystemEnum(self.GetSystem().as_deref())
                    == D::ToCounterStyleSystemEnum(new_value)
            }
            AtRuleDescriptorID::Symbols => Self::HasValidSymbolsForValues(
                self.GetSystem().as_deref(),
                new_value,
                self.GetAdditiveSymbols().as_deref(),
            ),
            AtRuleDescriptorID::AdditiveSymbols => Self::HasValidSymbolsForValues(
                self.GetSystem().as_deref(),
                self.GetSymbols().as_deref(),
                new_value,
            ),
            _ => true,
        }
    }
    // cpp: style_rule_counter_style.cc:152-156. Validation belongs to the
    // caller: this setter always requests the mutable store and bumps version.
    pub fn SetDescriptorValue(
        &self,
        descriptor_id: AtRuleDescriptorID,
        new_value: Rc<CSSValue<D>>,
    ) {
        self.MutableStyleForInspector().borrow_mut().SetProperty(
            AtRuleDescriptorIDAsCSSPropertyID(descriptor_id),
            new_value,
            false,
        );
    }
    // cpp: style_rule_counter_style.h:59-67
    pub fn SetName(&mut self, name: AtomicString) {
        self.name_ = name;
        self.version_.set(self.version_.get() + 1);
    }
    pub fn HasFailedOrCanceledSubresources(&self) -> bool {
        false
    }
}
// cpp: style_rule_counter_style.cc:58-75. Version tracks mutation and is not
// content; all ten descriptors use CSSValue's actual payload equality.
impl<D: StyleRuleCounterStyleBackend> PartialEq for StyleRuleCounterStyle<D> {
    fn eq(&self, other: &Self) -> bool {
        if std::ptr::eq(self, other) {
            return true;
        }
        self.name_ == other.name_
            && ValuesEquivalent(self.GetSystem().as_deref(), other.GetSystem().as_deref())
            && ValuesEquivalent(
                self.GetNegative().as_deref(),
                other.GetNegative().as_deref(),
            )
            && ValuesEquivalent(self.GetPrefix().as_deref(), other.GetPrefix().as_deref())
            && ValuesEquivalent(self.GetSuffix().as_deref(), other.GetSuffix().as_deref())
            && ValuesEquivalent(self.GetRange().as_deref(), other.GetRange().as_deref())
            && ValuesEquivalent(self.GetPad().as_deref(), other.GetPad().as_deref())
            && ValuesEquivalent(
                self.GetFallback().as_deref(),
                other.GetFallback().as_deref(),
            )
            && ValuesEquivalent(self.GetSymbols().as_deref(), other.GetSymbols().as_deref())
            && ValuesEquivalent(
                self.GetAdditiveSymbols().as_deref(),
                other.GetAdditiveSymbols().as_deref(),
            )
            && ValuesEquivalent(self.GetSpeakAs().as_deref(), other.GetSpeakAs().as_deref())
    }
}
// cpp: base/memory/values_equivalent.h. Preserve null and pointer shortcuts.
fn ValuesEquivalent<V: PartialEq>(first: Option<&V>, second: Option<&V>) -> bool {
    match (first, second) {
        (None, None) => true,
        (Some(a), Some(b)) => std::ptr::eq(a, b) || a == b,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css_property_name::CSSPropertyName;
    use crate::css_property_value::{CSSPropertyValue, CSSPropertyValueBackend};
    use crate::css_property_value_set::{CSSPropertyValueSet, ImmutableCSSPropertyValueSet};
    use crate::css_value::*;
    use crate::parser::css_parser_mode::CSSParserMode;
    use foundation::{CSSValueID, Length, String};

    // Only tested dependencies have fixture behavior. Unrelated child methods
    // deliberately panic so these tests cannot pass through fabricated defaults.
    #[derive(PartialEq)]
    enum Fixture {
        System(CounterStyleSystem, u32),
        List(Vec<u32>),
        Text(&'static str),
    }
    impl CSSValueSubclass for Fixture {
        fn CustomCSSText(&self) -> String {
            panic!("unused fixture serialization")
        }
        fn Equals(&self, other: &Self) -> bool {
            self == other
        }
    }
    impl CSSValueCustomHash for Fixture {
        fn CustomHash(&self) -> u32 {
            panic!("unused fixture hash")
        }
    }
    impl CSSValueSubresources for Fixture {
        fn HasFailedOrCanceledSubresources(&self) -> bool {
            panic!("unused fixture resources")
        }
    }
    impl CSSValueRandom for Fixture {
        fn HasRandomFunctions(&self) -> bool {
            panic!("unused fixture random functions")
        }
    }
    impl CSSValueUrl<()> for Fixture {
        fn ReResolveUrl(&self, _: &()) {
            panic!("unused fixture URL")
        }
    }
    impl CSSValueListUrls for Fixture {
        fn MayContainUrl(&self) -> bool {
            panic!("unused fixture list URL")
        }
    }
    impl CSSValueListSubclass<Backend> for Fixture {
        fn AsValueList(&self) -> &Fixture {
            assert!(matches!(self, Self::List(_)));
            self
        }
    }
    impl CSSValuePairSubclass<Backend> for Fixture {
        fn AsValuePair(&self) -> &Fixture {
            panic!("unused fixture pair")
        }
    }
    impl CSSValueTreeScope<Backend> for Fixture {
        fn PopulateWithTreeScope<'a>(&'a self, _: Option<&'a ()>) -> &'a CSSValue<Backend> {
            panic!("unused fixture tree scope")
        }
    }
    struct Backend;
    macro_rules! fixture_types {
        ($($name:ident),* $(,)?) => { $(type $name = Fixture;)* };
    }
    impl CSSValueDispatch for Backend {
        type Document = ();
        type TreeScope = ();
        fixture_types!(
            CSSNumericLiteralValue,
            CSSMathFunctionValue,
            CSSIdentifierValue,
            CSSScopedKeywordValue,
            CSSColor,
            CSSUnresolvedColorValue,
            CSSColorMixValue,
            CSSAlphaColorValue,
            CSSContrastColorValue,
            CSSCounterValue,
            CSSCounterContentValue,
            CSSQuadValue,
            CSSCustomIdentValue,
            CSSStringValue,
            CSSURIValue,
            CSSURLPatternValue,
            CSSValuePair,
            CSSLightDarkValuePair,
            CSSParamValuePair,
            CSSScrollValue,
            CSSViewValue,
            CSSRatioValue,
            CSSRelativeColorValue,
            CSSBasicShapeCircleValue,
            CSSBasicShapeEllipseValue,
            CSSBasicShapePolygonValue,
            CSSBasicShapeInsetValue,
            CSSBasicShapeRectValue,
            CSSBasicShapeXYWHValue,
            CSSPathValue,
            CSSShapeValue,
            CSSImageValue,
            CSSCursorImageValue,
            CSSCrossfadeValue,
            CSSPaintValue,
            CSSLinearGradientValue,
            CSSRadialGradientValue,
            CSSConicGradientValue,
            CSSConstantGradientValue,
            CSSColorImageValue,
            CSSLinearTimingFunctionValue,
            CSSCubicBezierTimingFunctionValue,
            CSSStepsTimingFunctionValue,
            CSSProgressValue,
            CSSBorderImageSliceValue,
            CSSDynamicRangeLimitMixValue,
            CSSFontFeatureValue,
            CSSFontFaceSrcValue,
            CSSFontFamilyValue,
            CSSFontStyleRangeValue,
            CSSFontVariationValue,
            CSSAlternateValue,
            CSSInheritedValue,
            CSSInitialValue,
            CSSUnsetValue,
            CSSRevertValue,
            CSSRevertLayerValue,
            CSSRevertRuleValue,
            CSSReflectValue,
            CSSShadowValue,
            CSSUnicodeRangeValue,
            CSSGridTemplateAreasValue,
            CSSPaletteMixValue,
            CSSRayValue,
            CSSUnparsedDeclarationValue,
            CSSPendingSubstitutionValue,
            CSSPendingSystemFontValue,
            CSSInvalidVariableValue,
            CSSCyclicVariableValue,
            CSSFlipRevertValue,
            CSSLayoutFunctionValue,
            CSSContentDistributionValue,
            CSSKeyframeShorthandValue,
            CSSInitialColorValue,
            CSSImageSetOptionValue,
            CSSImageSetTypeValue,
            CSSRepeatStyleValue,
            CSSSuperellipseValue,
            CSSSymbolsValue,
            CSSTriggerAttachmentValue,
            CSSRepeatValue,
            CSSValueList,
            CSSFunctionValue,
            CSSImageSetValue,
            CSSBracketedValueList,
            CSSGridAutoRepeatValue,
            CSSGridIntegerRepeatValue,
            CSSAxisValue
        );
        fn CreateIdentifierFromLength(_: &Length) -> Rc<CSSValue<Self>> {
            panic!("unused fixture length identifier")
        }
        fn CreatePrimitiveFromLength(_: &Length, _: f32) -> Rc<CSSValue<Self>> {
            panic!("unused fixture length primitive")
        }
    }
    impl CSSPropertyValueBackend for Backend {
        fn MatchingShorthandsForLonghand(_: CSSPropertyID) -> Vec<CSSPropertyID> {
            panic!("counter style descriptors have no shorthand provenance")
        }
        fn IsAffectedByAll(_: CSSPropertyID) -> bool {
            panic!("unused fixture all metadata")
        }
    }
    impl CSSPropertyValueSetBackend for Backend {
        type CSSStyleDeclaration = ();
        type ExecutionContext = ();
        fn ShorthandForProperty(id: CSSPropertyID) -> Vec<CSSPropertyID> {
            assert!(DESCRIPTORS
                .iter()
                .any(|&descriptor| AtRuleDescriptorIDAsCSSPropertyID(descriptor) == id));
            vec![]
        }
        fn IsInSameLogicalPropertyGroupWithDifferentMappingLogic(
            _: CSSPropertyID,
            _: CSSPropertyID,
        ) -> bool {
            panic!("counter style descriptors have no logical mapping")
        }
        fn SerializeShorthand(_: &CSSPropertyValueSet<Self>, _: CSSPropertyID) -> String {
            panic!("unused fixture shorthand serialization")
        }
        fn AsText(_: &CSSPropertyValueSet<Self>) -> String {
            panic!("unused fixture serialization")
        }
        fn CreateIdentifier(_: CSSValueID) -> Rc<CSSValue<Self>> {
            panic!("unused fixture identifier")
        }
        fn DeclarationPropertyValueSet(_: &()) -> Option<&CSSPropertyValueSet<Self>> {
            panic!("unused fixture declaration")
        }
        fn DeclarationPropertyMatches(_: &(), _: CSSPropertyID, _: &CSSValue<Self>) -> bool {
            panic!("unused fixture declaration comparison")
        }
        fn NewCSSStyleDeclaration(_: Option<&()>, _: &MutableCSSPropertyValueSet<Self>) -> Rc<()> {
            panic!("unused fixture declaration construction")
        }
    }
    impl StyleRuleCounterStyleBackend for Backend {
        fn ToCounterStyleSystemEnum(value: Option<&CSSValue<Self>>) -> CounterStyleSystem {
            match value.map(CSSValue::Payload) {
                None => CounterStyleSystem::kSymbolic,
                Some(CSSValuePayload::kIdentifierClass(Fixture::System(system, _))) => *system,
                _ => panic!("fixture system conversion requires its system payload"),
            }
        }
        fn CSSValueListLength(value: &Fixture) -> usize {
            match value {
                Fixture::List(items) => items.len(),
                _ => panic!("fixture length requires its list payload"),
            }
        }
    }
    const DESCRIPTORS: [AtRuleDescriptorID; 10] = [
        AtRuleDescriptorID::System,
        AtRuleDescriptorID::Negative,
        AtRuleDescriptorID::Prefix,
        AtRuleDescriptorID::Suffix,
        AtRuleDescriptorID::Range,
        AtRuleDescriptorID::Pad,
        AtRuleDescriptorID::Fallback,
        AtRuleDescriptorID::Symbols,
        AtRuleDescriptorID::AdditiveSymbols,
        AtRuleDescriptorID::SpeakAs,
    ];
    fn system(system: CounterStyleSystem, parameter: u32) -> Rc<CSSValue<Backend>> {
        Rc::new(CSSValue::new(CSSValuePayload::kIdentifierClass(
            Fixture::System(system, parameter),
        )))
    }
    fn list(length: usize) -> Rc<CSSValue<Backend>> {
        Rc::new(CSSValue::new(CSSValuePayload::kValueListClass(
            Fixture::List(vec![1; length]),
        )))
    }
    fn text(text: &'static str) -> Rc<CSSValue<Backend>> {
        Rc::new(CSSValue::new(CSSValuePayload::kStringClass(Fixture::Text(
            text,
        ))))
    }
    fn immutable(values: &[Rc<CSSValue<Backend>>]) -> Rc<CSSPropertyValueSetRuleHandle<Backend>> {
        let properties = DESCRIPTORS
            .iter()
            .zip(values)
            .map(|(&descriptor, value)| {
                CSSPropertyValue::new(
                    &CSSPropertyName::new(AtRuleDescriptorIDAsCSSPropertyID(descriptor)),
                    value.clone(),
                    false,
                    false,
                    0,
                    false,
                )
            })
            .collect::<Vec<_>>();
        CSSPropertyValueSetRuleHandle::FromImmutable(ImmutableCSSPropertyValueSet::Create(
            &properties,
            CSSParserMode::kCSSCounterStyleRuleMode,
            false,
        ))
    }
    type Rule = StyleRuleCounterStyle<Backend>;

    #[test]
    fn covers_every_system_symbol_requirement_and_list_subclass_cast() {
        use CounterStyleSystem::*;
        let systems = [
            kCyclic,
            kFixed,
            kSymbolic,
            kAlphabetic,
            kNumeric,
            kAdditive,
            kUnresolvedExtends,
            kHebrew,
            kSimpChineseInformal,
            kSimpChineseFormal,
            kTradChineseInformal,
            kTradChineseFormal,
            kKoreanHangulFormal,
            kKoreanHanjaInformal,
            kKoreanHanjaFormal,
            kLowerArmenian,
            kUpperArmenian,
            kEthiopicNumeric,
        ];
        let lists = [None, Some(list(0)), Some(list(1)), Some(list(2))];
        for algorithm in systems {
            let system = system(algorithm, 0);
            for (i, symbols) in lists.iter().enumerate() {
                for (j, additive) in lists.iter().enumerate() {
                    let expected = match algorithm {
                        kCyclic | kFixed | kSymbolic => i >= 2,
                        kAlphabetic | kNumeric => i == 3,
                        kAdditive => j >= 2,
                        kUnresolvedExtends => i == 0 && j == 0,
                        _ => true,
                    };
                    assert_eq!(
                        Rule::HasValidSymbolsForValues(
                            Some(&system),
                            symbols.as_deref(),
                            additive.as_deref()
                        ),
                        expected,
                        "{algorithm:?}, symbols={i}, additive={j}"
                    );
                }
            }
        }
        assert!(!Rule::HasValidSymbolsForValues(None, None, None));
        assert!(Rule::HasValidSymbolsForValues(None, Some(&list(1)), None));
        let subclasses = [
            CSSValuePayload::kFunctionClass(Fixture::List(vec![1])),
            CSSValuePayload::kImageSetClass(Fixture::List(vec![1])),
            CSSValuePayload::kGridLineNamesClass(Fixture::List(vec![1])),
            CSSValuePayload::kGridAutoRepeatClass(Fixture::List(vec![1])),
            CSSValuePayload::kGridIntegerRepeatClass(Fixture::List(vec![1])),
            CSSValuePayload::kAxisClass(Fixture::List(vec![1])),
        ];
        for payload in subclasses {
            assert!(Rule::HasValidSymbolsForValues(
                None,
                Some(&CSSValue::new(payload)),
                None
            ));
        }
    }

    #[test]
    fn preserves_descriptor_identity_copy_on_mutation_versions_equality_and_setter_semantics() {
        use CounterStyleSystem::*;
        let values = [
            system(kFixed, 1),
            text("negative"),
            text("prefix"),
            text("suffix"),
            text("range"),
            text("pad"),
            text("fallback"),
            list(1),
            list(1),
            text("speak-as"),
        ];
        let properties = immutable(&values);
        let mut rule = Rule::new(AtomicString::from_str("example"), properties.clone());
        let snapshot = rule.clone();
        assert_eq!(rule.GetType(), RuleType::kCounterStyle);
        assert_eq!(rule.GetVersion(), 0);
        assert_eq!(rule.GetName(), AtomicString::from_str("example"));
        assert!(!rule.HasFailedOrCanceledSubresources());
        assert!(rule.HasValidSymbols());
        assert!(rule == rule && rule == snapshot);
        let getters: [fn(&Rule) -> Option<Rc<CSSValue<Backend>>>; 10] = [
            Rule::GetSystem,
            Rule::GetNegative,
            Rule::GetPrefix,
            Rule::GetSuffix,
            Rule::GetRange,
            Rule::GetPad,
            Rule::GetFallback,
            Rule::GetSymbols,
            Rule::GetAdditiveSymbols,
            Rule::GetSpeakAs,
        ];
        for (getter, value) in getters.into_iter().zip(&values) {
            assert!(Rc::ptr_eq(&getter(&rule).unwrap(), value));
        }
        let mutable = rule.Properties();
        assert_eq!(rule.GetVersion(), 0);
        assert!(Rc::ptr_eq(&mutable, &rule.Properties()));
        assert!(!properties.IsMutable());
        assert!(!Rc::ptr_eq(&mutable, &snapshot.Properties()));
        let shared = rule.clone();
        assert!(Rc::ptr_eq(&mutable, &shared.Properties()));
        assert!(Rc::ptr_eq(&mutable, &rule.MutableStyleForInspector()));
        assert_eq!(rule.GetVersion(), 1);
        assert_eq!(shared.GetVersion(), 0);
        assert!(rule == shared);

        // Equality examines all ten descriptors independently, through payload
        // equality even when two CSSValues occupy distinct allocations.
        for i in 0..DESCRIPTORS.len() {
            let mut equivalent = values.clone();
            equivalent[i] = match i {
                0 => system(kFixed, 1),
                7 | 8 => list(1),
                _ => text(match i {
                    1 => "negative",
                    2 => "prefix",
                    3 => "suffix",
                    4 => "range",
                    5 => "pad",
                    6 => "fallback",
                    9 => "speak-as",
                    _ => unreachable!(),
                }),
            };
            let equal = Rule::new(rule.GetName(), immutable(&equivalent));
            assert!(rule == equal);
            equivalent[i] = text("changed descriptor");
            let different = Rule::new(rule.GetName(), immutable(&equivalent));
            assert!(
                rule != different,
                "descriptor {i} must participate in equality"
            );
        }
        assert!(!rule.NewValueInvalidOrEqual(AtRuleDescriptorID::Suffix, Some(&text("suffix"))));
        assert!(!rule.NewValueInvalidOrEqual(AtRuleDescriptorID::System, Some(&system(kFixed, 1))));
        assert!(rule.NewValueInvalidOrEqual(AtRuleDescriptorID::System, Some(&system(kFixed, 2))));
        assert!(
            !rule.NewValueInvalidOrEqual(AtRuleDescriptorID::System, Some(&system(kNumeric, 1)))
        );
        assert!(!rule.NewValueInvalidOrEqual(AtRuleDescriptorID::Symbols, Some(&list(0))));
        assert!(rule.NewValueInvalidOrEqual(AtRuleDescriptorID::Symbols, Some(&list(2))));
        assert!(rule.NewValueInvalidOrEqual(AtRuleDescriptorID::AdditiveSymbols, Some(&list(0))));
        assert!(rule.NewValueInvalidOrEqual(AtRuleDescriptorID::Prefix, None));
        assert!(!ValuesEquivalent(Some(values[1].as_ref()), None));
        assert!(ValuesEquivalent::<CSSValue<Backend>>(None, None));
        let suffix = text("changed suffix");
        rule.SetDescriptorValue(AtRuleDescriptorID::Suffix, suffix.clone());
        assert_eq!(rule.GetVersion(), 2);
        assert!(Rc::ptr_eq(&rule.GetSuffix().unwrap(), &suffix));
        assert!(Rc::ptr_eq(&shared.GetSuffix().unwrap(), &suffix));
        assert!(rule == shared && rule != snapshot);
        rule.SetDescriptorValue(AtRuleDescriptorID::Suffix, suffix);
        assert_eq!(rule.GetVersion(), 3);
        rule.SetName(rule.GetName());
        assert_eq!(rule.GetVersion(), 4);
        assert!(rule == shared);
        rule.SetName(AtomicString::from_str("renamed"));
        assert_eq!(rule.GetVersion(), 5);
        assert!(rule != shared);
        // The setter performs no validation; its caller must use the probe.
        rule.SetDescriptorValue(AtRuleDescriptorID::System, system(kAdditive, 0));
        assert_eq!(rule.GetVersion(), 6);
        assert!(!rule.NewValueInvalidOrEqual(AtRuleDescriptorID::AdditiveSymbols, Some(&list(0))));
        assert!(!rule.NewValueInvalidOrEqual(AtRuleDescriptorID::AdditiveSymbols, None));
        assert!(rule.NewValueInvalidOrEqual(AtRuleDescriptorID::AdditiveSymbols, Some(&list(2))));
        rule.SetDescriptorValue(AtRuleDescriptorID::System, system(kUnresolvedExtends, 0));
        assert!(!rule.NewValueInvalidOrEqual(AtRuleDescriptorID::Symbols, Some(&list(2))));
        let empty = Rule::new(AtomicString::from_str("empty"), immutable(&[]));
        assert!(!empty.NewValueInvalidOrEqual(AtRuleDescriptorID::Prefix, None));
        assert!(!empty.HasValidSymbols());
    }
}
