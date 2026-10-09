/*
 * (C) 1999-2003 Lars Knoll (knoll@kde.org)
 * (C) 2002-2003 Dirk Mueller (mueller@kde.org)
 * Copyright (C) 2002, 2006, 2008, 2012, 2013 Apple Inc. All rights reserved.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Library General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Library General Public License for more details.
 *
 * You should have received a copy of the GNU Library General Public License
 * along with this library; see the file COPYING.LIB.  If not, write to
 * the Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
 * Boston, MA 02110-1301, USA.
 */

// cpp: third_party/blink/renderer/core/css/style_rule.h:50-179,196-1023
// cpp: third_party/blink/renderer/core/css/style_rule.cc:101-113,331-527,542-586,604-1270,1276-1298
// GC Trace/destructor/allocation layout is handled by Rust. Runtime segments
// are marked individually below. Missing source classes enter through required
// typed operations with no defaults; no selector, CSSValue or rule is invented.
// Clone/renesting dispatch, clone helpers, average size, and mixin binding
// hashing/equality are implemented here. External clone constructors and
// selector renesting remain explicit required dependencies until assembled.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use crate::css_markup::SerializeIdentifierTo;
use crate::css_property_names::CSSPropertyID;
use crate::parser::css_at_rule_id::CSSAtRuleID;
use crate::parser::css_nesting_type::CSSNestingType;
use crate::parser::css_parser_impl::CSSRuleOrdering;
use foundation::{AtomicString, String};
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;

// cpp: style_rule.h:58-92
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuleType {
    kCharset = 0,
    kStyle = 1,
    kImport = 2,
    kMedia = 3,
    kFontFace = 4,
    kFontPaletteValues = 5,
    kFontFeatureValues = 6,
    kFontFeature = 7,
    kPage = 8,
    kPageMargin = 9,
    kProperty = 10,
    kNavigation = 11,
    kKeyframes = 12,
    kKeyframe = 13,
    kLayerBlock = 14,
    kLayerStatement = 15,
    kNestedDeclarations = 16,
    kFunctionDeclarations = 17,
    kNamespace = 18,
    kContainer = 19,
    kCounterStyle = 20,
    kScope = 21,
    kSupports = 22,
    kStartingStyle = 23,
    kViewTransition = 24,
    kFunction = 25,
    kMixin = 26,
    kResult = 27,
    kPrivate = 28,
    kApplyMixin = 29,
    kContents = 30,
    kPositionTry = 31,
    kCustomMedia = 32,
    kLocation = 33,
}
// cpp: style_rule.h:98
pub type LayerName = Vec<AtomicString>;

// CSSSelectorList and CSSSelector are required real dependencies. The methods
// correspond to the selector/list calls in style_rule.h:287-308. A production
// implementation must own the actual flattened selectors and preserve identity.
pub trait StyleRuleSelectorList {
    type CSSSelector;
    fn FirstSelector(&self) -> &Self::CSSSelector;
    fn SelectorAt(&self, index: usize) -> &Self::CSSSelector;
    fn MutableSelectorAt(&mut self, index: usize) -> &mut Self::CSSSelector;
    fn SelectorIndex(&self, selector: &Self::CSSSelector) -> usize;
    fn NextSelector(&self, selector: &Self::CSSSelector) -> Option<&Self::CSSSelector>;
    fn SelectorsText(&self) -> String;
}
// Actual CSSPropertyValueSet operations read by the rule classes. The existing
// base-only module cannot satisfy this contract without the real property store.
pub trait StyleRulePropertySet: Sized {
    type CSSValue;
    fn IsMutable(&self) -> bool;
    fn MutableCopy(&self) -> Rc<Self>;
    fn HasFailedOrCanceledSubresources(&self) -> bool;
    fn GetPropertyCSSValue(&self, id: CSSPropertyID) -> Option<Rc<Self::CSSValue>>;
}

// Source dependencies missing from this file's source pair, with no defaults.
// Import/namespace/keyframe classes have their own mapped source files; their
// handles are mandatory actual bodies, not empty stand-in rule implementations.
pub trait StyleRuleDependencies: Sized {
    type SelectorList: StyleRuleSelectorList;
    type CSSPropertyValueSet: StyleRulePropertySet;
    type CSSLazyParsingState;
    type MixinParameterBindings;
    type StyleScope;
    type MediaQuerySet;
    type ContainerQuerySet;
    type ContainerQuery;
    type ConditionalExpNode;
    type NavigationQuery;
    type CSSSyntaxDefinition: Clone;
    type CSSVariableData;
    type CSSPrivateVariable;
    type ExecutionContext;
    type StyleRuleImport;
    type StyleRuleFontPaletteValues;
    type StyleRuleFontFeatureValues;
    type StyleRuleFontFeature;
    type StyleRuleKeyframes;
    type StyleRuleKeyframe;
    type StyleRuleNestedDeclarations;
    type StyleRuleFunctionDeclarations;
    type StyleRuleNamespace;
    type StyleRuleCounterStyle;
    type StyleRuleViewTransition;
    type StyleRulePositionTry;
    type StyleRuleLocation;

    // css_parser_impl ParseDeclarationListForLazyStyle with state sheet/context.
    fn ParseDeclarationListForLazyStyle(
        state: &Self::CSSLazyParsingState,
        offset: usize,
    ) -> Rc<Self::CSSPropertyValueSet>;
    fn ParseCustomPropertyName(text: &String) -> String;
    fn ConsumeSupportsCondition(context: &Self::ExecutionContext, text: &String) -> bool;
    fn ContainerQuerySetToString(set: &Self::ContainerQuerySet) -> String;
    fn ParseContainerQuerySet(
        context: &Self::ExecutionContext,
        text: &String,
    ) -> Option<Rc<Self::ContainerQuerySet>>;
    fn SingleContainerQuery(set: &Self::ContainerQuerySet) -> Option<&Self::ContainerQuery>;
    fn ParseContainerCondition(
        context: &Self::ExecutionContext,
        text: &String,
    ) -> Option<Rc<Self::ConditionalExpNode>>;
    fn SerializeContainerCondition(node: &Self::ConditionalExpNode) -> String;
    fn ContainerQuerySelectorName(query: &Self::ContainerQuery) -> AtomicString;
    fn NewContainerQuery(
        name: AtomicString,
        node: Rc<Self::ConditionalExpNode>,
    ) -> Rc<Self::ContainerQuery>;
    fn NewContainerQuerySet(queries: Vec<Rc<Self::ContainerQuery>>) -> Rc<Self::ContainerQuerySet>;
    fn ParseNavigationQuery(text: &String) -> Option<Rc<Self::NavigationQuery>>;
    fn CopyPrivateVariable(variable: &Self::CSSPrivateVariable) -> Rc<Self::CSSPrivateVariable>;
}

// Copy/size operations are separate so users of the mutation-only rule API do
// not need to implement dependencies that they never invoke.
pub trait StyleRulePropertySetClone: StyleRulePropertySet {
    fn ImmutableCopyIfNeeded(this: &Rc<Self>) -> Rc<Self>;
    fn AverageSizeInBytes() -> usize;
}

// Required external constructors/accessors, each matching a call in Clone.
// CopyFoo invokes that class's actual copy constructor; in particular it must
// allocate a fresh rule, not return the supplied Rc. Selector operations must
// preserve Renest's identity comparisons and nested selector-list recursion.
pub trait StyleRuleCloneDependencies:
    StyleRuleDependencies<StyleRuleFontFeatureValues: Clone, StyleRuleFontFeature: Clone>
{
    type KeyframeOffset: Clone;
    fn RenestStyleSelectors(
        list: &Self::SelectorList,
        parent: Option<Rc<StyleRule<Self>>>,
    ) -> Self::SelectorList;
    fn RenestPageSelectors(
        list: &Rc<Self::SelectorList>,
        parent: Option<Rc<StyleRule<Self>>>,
    ) -> Rc<Self::SelectorList>;
    fn CopySelectors(list: &Self::SelectorList) -> Self::SelectorList;
    // One implicit parent selector, null parent, with both list-end bits set.
    fn DummyNestingSelectors() -> Self::SelectorList;
    fn CloneStyleScope(
        scope: &Self::StyleScope,
        parent: Option<Rc<StyleRule<Self>>>,
    ) -> Rc<Self::StyleScope>;
    fn ScopeRuleForNesting(scope: &Self::StyleScope) -> Option<Rc<StyleRule<Self>>>;
    fn CopyContainerQuerySet(set: &Self::ContainerQuerySet) -> Rc<Self::ContainerQuerySet>;
    fn NestedDeclarationsNestingType(rule: &Self::StyleRuleNestedDeclarations) -> CSSNestingType;
    fn NestedDeclarationsInnerRule(rule: &Self::StyleRuleNestedDeclarations) -> &StyleRule<Self>;
    fn NewNestedDeclarations(
        nesting: CSSNestingType,
        inner: Rc<StyleRule<Self>>,
    ) -> Rc<Self::StyleRuleNestedDeclarations>;
    fn Keyframes(rule: &Self::StyleRuleKeyframes) -> &[Rc<Self::StyleRuleKeyframe>];
    fn KeyframesName(rule: &Self::StyleRuleKeyframes) -> AtomicString;
    fn KeyframesVersion(rule: &Self::StyleRuleKeyframes) -> u32;
    fn KeyframesIsVendorPrefixed(rule: &Self::StyleRuleKeyframes) -> bool;
    fn NewKeyframes(
        keys: Vec<Rc<Self::StyleRuleKeyframe>>,
        name: AtomicString,
        version: u32,
        prefixed: bool,
    ) -> Rc<Self::StyleRuleKeyframes>;
    fn KeyframeKeys(rule: &Self::StyleRuleKeyframe) -> &[Self::KeyframeOffset];
    fn KeyframeProperties(rule: &Self::StyleRuleKeyframe) -> Rc<Self::CSSPropertyValueSet>;
    fn NewKeyframe(
        keys: Vec<Self::KeyframeOffset>,
        properties: Rc<Self::CSSPropertyValueSet>,
    ) -> Rc<Self::StyleRuleKeyframe>;
    fn CounterStyleName(rule: &Self::StyleRuleCounterStyle) -> AtomicString;
    fn CounterStyleProperties(rule: &Self::StyleRuleCounterStyle) -> Rc<Self::CSSPropertyValueSet>;
    fn NewCounterStyle(
        name: AtomicString,
        properties: Rc<Self::CSSPropertyValueSet>,
    ) -> Rc<Self::StyleRuleCounterStyle>;
    fn PositionTryName(rule: &Self::StyleRulePositionTry) -> AtomicString;
    fn PositionTryProperties(rule: &Self::StyleRulePositionTry) -> Rc<Self::CSSPropertyValueSet>;
    fn NewPositionTry(
        name: AtomicString,
        properties: Rc<Self::CSSPropertyValueSet>,
    ) -> Rc<Self::StyleRulePositionTry>;
    fn CopyImport(rule: &Self::StyleRuleImport) -> Rc<Self::StyleRuleImport>;
    fn CopyFontPaletteValues(
        rule: &Self::StyleRuleFontPaletteValues,
    ) -> Rc<Self::StyleRuleFontPaletteValues>;
    fn CopyFunctionDeclarations(
        rule: &Self::StyleRuleFunctionDeclarations,
    ) -> Rc<Self::StyleRuleFunctionDeclarations>;
    fn CopyNamespace(rule: &Self::StyleRuleNamespace) -> Rc<Self::StyleRuleNamespace>;
    fn CopyViewTransition(
        rule: &Self::StyleRuleViewTransition,
    ) -> Rc<Self::StyleRuleViewTransition>;
    fn CopyLocation(rule: &Self::StyleRuleLocation) -> Rc<Self::StyleRuleLocation>;
}

impl<B: crate::css_property_value_set::CSSPropertyValueSetBackend> StyleRulePropertySet
    for crate::css_property_value_set::CSSPropertyValueSetRuleHandle<B>
{
    type CSSValue = crate::css_value::CSSValue<B>;
    fn IsMutable(&self) -> bool {
        crate::css_property_value_set::CSSPropertyValueSetRuleAdapter::IsMutable(self)
    }
    fn MutableCopy(&self) -> Rc<Self> {
        use crate::css_property_value_set::CSSPropertyValueSetRuleAdapter;
        Self::FromMutable(CSSPropertyValueSetRuleAdapter::MutableCopy(self))
    }
    fn HasFailedOrCanceledSubresources(&self) -> bool {
        crate::css_property_value_set::CSSPropertyValueSetRuleAdapter::HasFailedOrCanceledSubresources(self)
    }
    fn GetPropertyCSSValue(&self, id: CSSPropertyID) -> Option<Rc<Self::CSSValue>> {
        crate::css_property_value_set::CSSPropertyValueSetRuleAdapter::GetPropertyCSSValue(self, id)
    }
}
impl<B: crate::css_property_value_set::CSSPropertyValueSetBackend> StyleRulePropertySetClone
    for crate::css_property_value_set::CSSPropertyValueSetRuleHandle<B>
{
    fn ImmutableCopyIfNeeded(this: &Rc<Self>) -> Rc<Self> {
        use crate::css_property_value_set::ImmutableCSSPropertyValueSet;
        match this.as_ref() {
            Self::Immutable(_) => Rc::clone(this),
            Self::Mutable(set) => {
                let set = set.borrow();
                Self::FromImmutable(ImmutableCSSPropertyValueSet::Create(
                    set.Properties(),
                    set.CssParserMode(),
                    false,
                ))
            }
        }
    }
    fn AverageSizeInBytes() -> usize {
        crate::css_property_value_set::CSSPropertyValueSet::<B>::AverageSizeInBytes()
    }
}

// cpp: mixin_parameter_bindings.h:29-46,58-132
// Variable equality/hash and syntax are the real CSSVariableData and
// CSSSyntaxDefinition operations; StringHash is StringImpl::GetHash.
pub trait MixinParameterBindingsDependencies {
    type CSSVariableData: PartialEq;
    type CSSSyntaxDefinition: PartialEq;
    type ContainerQuerySet;
    fn VariableHash(value: &Self::CSSVariableData) -> u32;
    fn StringHash(key: &String) -> u32;
}
pub struct MixinBinding<D: MixinParameterBindingsDependencies> {
    pub value: Option<Rc<D::CSSVariableData>>,
    pub default_value: Option<Rc<D::CSSVariableData>>,
    pub syntax: D::CSSSyntaxDefinition,
}
impl<D: MixinParameterBindingsDependencies> PartialEq for MixinBinding<D> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
            && self.default_value == other.default_value
            && self.syntax == other.syntax
    }
}
pub struct MixinCQDependentValue<D: MixinParameterBindingsDependencies> {
    pub data: Rc<D::CSSVariableData>,
    pub container_queries: Rc<D::ContainerQuerySet>,
}
pub struct MixinParameterBindings<D: MixinParameterBindingsDependencies> {
    bindings_: HashMap<String, MixinBinding<D>>,
    base_locals_: HashMap<String, Rc<D::CSSVariableData>>,
    conditional_override_locals_: HashMap<String, Vec<MixinCQDependentValue<D>>>,
    parent_mixin_: Option<Rc<Self>>,
    hash_: u32,
}
impl<D: MixinParameterBindingsDependencies> MixinParameterBindings<D> {
    pub fn new(
        bindings: HashMap<String, MixinBinding<D>>,
        base_locals: HashMap<String, Rc<D::CSSVariableData>>,
        conditional_override_locals: HashMap<String, Vec<MixinCQDependentValue<D>>>,
        parent_mixin: Option<Rc<Self>>,
    ) -> Self {
        let mut result = Self {
            bindings_: bindings,
            base_locals_: base_locals,
            conditional_override_locals_: conditional_override_locals,
            parent_mixin_: parent_mixin,
            hash_: 0,
        };
        result.hash_ = result.ComputeHash();
        result
    }
    // cpp: style_rule.cc:1276-1288
    pub fn ComputeHash(&self) -> u32 {
        let mut hash = self
            .parent_mixin_
            .as_ref()
            .map_or(1234, |parent| parent.GetHash());
        for (key, value) in &self.bindings_ {
            hash = foundation::HashInts(
                hash,
                foundation::HashInts(
                    D::StringHash(key),
                    value
                        .value
                        .as_ref()
                        .map_or(5678, |value| D::VariableHash(value)),
                ),
            );
        }
        for (key, value) in &self.base_locals_ {
            hash = foundation::HashInts(
                hash,
                foundation::HashInts(D::StringHash(key) ^ 4321, D::VariableHash(value)),
            );
        }
        hash
    }
    pub fn GetHash(&self) -> u32 {
        self.hash_
    }
    pub fn GetBindings(&self) -> &HashMap<String, MixinBinding<D>> {
        &self.bindings_
    }
    pub fn GetBaseLocals(&self) -> &HashMap<String, Rc<D::CSSVariableData>> {
        &self.base_locals_
    }
    pub fn GetConditionalOverrideLocals(&self) -> &HashMap<String, Vec<MixinCQDependentValue<D>>> {
        &self.conditional_override_locals_
    }
    pub fn GetParentMixin(&self) -> Option<&Rc<Self>> {
        self.parent_mixin_.as_ref()
    }
}
impl<D: MixinParameterBindingsDependencies> PartialEq for MixinParameterBindings<D> {
    // cpp: style_rule.cc:1290-1298. HeapHashMap's Member values in base_locals
    // compare pointer identity; Binding and parent use ValuesEquivalent.
    // Conditional overrides deliberately participate in neither equality nor hash.
    fn eq(&self, other: &Self) -> bool {
        if self.bindings_ != other.bindings_ {
            return false;
        }
        if self.base_locals_.len() != other.base_locals_.len()
            || self.base_locals_.iter().any(|(key, value)| {
                other
                    .base_locals_
                    .get(key)
                    .is_none_or(|other| !Rc::ptr_eq(value, other))
            })
        {
            return false;
        }
        self.parent_mixin_ == other.parent_mixin_
    }
}

// Typed boundary for CSSStyleSheet::Contents()->NotifyRuleChanged. There is no
// default implementation: mutations with a parent must issue the real event.
pub trait StyleRuleChangeObserver<R: ?Sized> {
    fn NotifyRuleChanged(&mut self, rule: &R);
}

// cpp: style_rule.h:56-180 (tag is represented by the enum's real payload)
pub enum StyleRuleBase<D: StyleRuleDependencies> {
    Charset(StyleRuleCharset),
    Style(Rc<StyleRule<D>>),
    Import(Rc<D::StyleRuleImport>),
    Media(StyleRuleMedia<D>),
    FontFace(StyleRuleFontFace<D>),
    FontPaletteValues(Rc<D::StyleRuleFontPaletteValues>),
    FontFeatureValues(Rc<D::StyleRuleFontFeatureValues>),
    FontFeature(Rc<D::StyleRuleFontFeature>),
    Page(StyleRulePage<D>),
    PageMargin(StyleRulePageMargin<D>),
    Property(StyleRuleProperty<D>),
    Navigation(StyleRuleNavigation<D>),
    Keyframes(Rc<D::StyleRuleKeyframes>),
    Keyframe(Rc<D::StyleRuleKeyframe>),
    LayerBlock(StyleRuleLayerBlock<D>),
    LayerStatement(StyleRuleLayerStatement),
    NestedDeclarations(Rc<D::StyleRuleNestedDeclarations>),
    FunctionDeclarations(Rc<D::StyleRuleFunctionDeclarations>),
    Namespace(Rc<D::StyleRuleNamespace>),
    Container(StyleRuleContainer<D>),
    CounterStyle(Rc<D::StyleRuleCounterStyle>),
    Scope(StyleRuleScope<D>),
    Supports(StyleRuleSupports<D>),
    StartingStyle(StyleRuleStartingStyle<D>),
    ViewTransition(Rc<D::StyleRuleViewTransition>),
    Function(StyleRuleFunction<D>),
    Mixin(StyleRuleMixin<D>),
    Result(StyleRuleResult<D>),
    Private(StyleRulePrivate<D>),
    ApplyMixin(StyleRuleApplyMixin<D>),
    Contents(StyleRuleContentsStatement<D>),
    PositionTry(Rc<D::StyleRulePositionTry>),
    CustomMedia(StyleRuleCustomMedia<D>),
    Location(Rc<D::StyleRuleLocation>),
}
impl<D: StyleRuleDependencies> StyleRuleBase<D> {
    // Both views retain the same allocation as selector parent references and
    // RuleSet's typed table; extracting a typed rule never creates a copy.
    pub fn AsStyle(&self) -> Option<&StyleRule<D>> {
        match self {
            Self::Style(rule) => Some(rule),
            _ => None,
        }
    }
    pub fn StyleRef(&self) -> Option<Rc<StyleRule<D>>> {
        match self {
            Self::Style(rule) => Some(Rc::clone(rule)),
            _ => None,
        }
    }
    // cpp: style_rule.h:101
    pub fn GetType(&self) -> RuleType {
        match self {
            Self::Charset(_) => RuleType::kCharset,
            Self::Style(_) => RuleType::kStyle,
            Self::Import(_) => RuleType::kImport,
            Self::Media(_) => RuleType::kMedia,
            Self::FontFace(_) => RuleType::kFontFace,
            Self::FontPaletteValues(_) => RuleType::kFontPaletteValues,
            Self::FontFeatureValues(_) => RuleType::kFontFeatureValues,
            Self::FontFeature(_) => RuleType::kFontFeature,
            Self::Page(_) => RuleType::kPage,
            Self::PageMargin(_) => RuleType::kPageMargin,
            Self::Property(_) => RuleType::kProperty,
            Self::Navigation(_) => RuleType::kNavigation,
            Self::Keyframes(_) => RuleType::kKeyframes,
            Self::Keyframe(_) => RuleType::kKeyframe,
            Self::LayerBlock(_) => RuleType::kLayerBlock,
            Self::LayerStatement(_) => RuleType::kLayerStatement,
            Self::NestedDeclarations(_) => RuleType::kNestedDeclarations,
            Self::FunctionDeclarations(_) => RuleType::kFunctionDeclarations,
            Self::Namespace(_) => RuleType::kNamespace,
            Self::Container(_) => RuleType::kContainer,
            Self::CounterStyle(_) => RuleType::kCounterStyle,
            Self::Scope(_) => RuleType::kScope,
            Self::Supports(_) => RuleType::kSupports,
            Self::StartingStyle(_) => RuleType::kStartingStyle,
            Self::ViewTransition(_) => RuleType::kViewTransition,
            Self::Function(_) => RuleType::kFunction,
            Self::Mixin(_) => RuleType::kMixin,
            Self::Result(_) => RuleType::kResult,
            Self::Private(_) => RuleType::kPrivate,
            Self::ApplyMixin(_) => RuleType::kApplyMixin,
            Self::Contents(_) => RuleType::kContents,
            Self::PositionTry(_) => RuleType::kPositionTry,
            Self::CustomMedia(_) => RuleType::kCustomMedia,
            Self::Location(_) => RuleType::kLocation,
        }
    }
    // cpp: style_rule.h:103-149
    pub fn IsCharsetRule(&self) -> bool {
        self.GetType() == RuleType::kCharset
    }
    // cpp: style_rule.h:103-149
    pub fn IsContainerRule(&self) -> bool {
        self.GetType() == RuleType::kContainer
    }
    // cpp: style_rule.h:103-149
    pub fn IsCounterStyleRule(&self) -> bool {
        self.GetType() == RuleType::kCounterStyle
    }
    // cpp: style_rule.h:103-149
    pub fn IsFontFaceRule(&self) -> bool {
        self.GetType() == RuleType::kFontFace
    }
    // cpp: style_rule.h:103-149
    pub fn IsFontPaletteValuesRule(&self) -> bool {
        self.GetType() == RuleType::kFontPaletteValues
    }
    // cpp: style_rule.h:103-149
    pub fn IsFontFeatureValuesRule(&self) -> bool {
        self.GetType() == RuleType::kFontFeatureValues
    }
    // cpp: style_rule.h:103-149
    pub fn IsFontFeatureRule(&self) -> bool {
        self.GetType() == RuleType::kFontFeature
    }
    // cpp: style_rule.h:103-149
    pub fn IsKeyframesRule(&self) -> bool {
        self.GetType() == RuleType::kKeyframes
    }
    // cpp: style_rule.h:103-149
    pub fn IsKeyframeRule(&self) -> bool {
        self.GetType() == RuleType::kKeyframe
    }
    // cpp: style_rule.h:103-149
    pub fn IsLayerBlockRule(&self) -> bool {
        self.GetType() == RuleType::kLayerBlock
    }
    // cpp: style_rule.h:103-149
    pub fn IsLayerStatementRule(&self) -> bool {
        self.GetType() == RuleType::kLayerStatement
    }
    // cpp: style_rule.h:103-149
    pub fn IsFunctionDeclarationsRule(&self) -> bool {
        self.GetType() == RuleType::kFunctionDeclarations
    }
    // cpp: style_rule.h:103-149
    pub fn IsNestedDeclarationsRule(&self) -> bool {
        self.GetType() == RuleType::kNestedDeclarations
    }
    // cpp: style_rule.h:103-149
    pub fn IsNamespaceRule(&self) -> bool {
        self.GetType() == RuleType::kNamespace
    }
    // cpp: style_rule.h:103-149
    pub fn IsMediaRule(&self) -> bool {
        self.GetType() == RuleType::kMedia
    }
    // cpp: style_rule.h:103-149
    pub fn IsPageRule(&self) -> bool {
        self.GetType() == RuleType::kPage
    }
    // cpp: style_rule.h:103-149
    pub fn IsPageRuleMargin(&self) -> bool {
        self.GetType() == RuleType::kPageMargin
    }
    // cpp: style_rule.h:103-149
    pub fn IsPropertyRule(&self) -> bool {
        self.GetType() == RuleType::kProperty
    }
    // cpp: style_rule.h:103-149
    pub fn IsLocationRule(&self) -> bool {
        self.GetType() == RuleType::kLocation
    }
    // cpp: style_rule.h:103-149
    pub fn IsNavigationRule(&self) -> bool {
        self.GetType() == RuleType::kNavigation
    }
    // cpp: style_rule.h:103-149
    pub fn IsStyleRule(&self) -> bool {
        self.GetType() == RuleType::kStyle
    }
    // cpp: style_rule.h:103-149
    pub fn IsScopeRule(&self) -> bool {
        self.GetType() == RuleType::kScope
    }
    // cpp: style_rule.h:103-149
    pub fn IsSupportsRule(&self) -> bool {
        self.GetType() == RuleType::kSupports
    }
    // cpp: style_rule.h:103-149
    pub fn IsImportRule(&self) -> bool {
        self.GetType() == RuleType::kImport
    }
    // cpp: style_rule.h:103-149
    pub fn IsStartingStyleRule(&self) -> bool {
        self.GetType() == RuleType::kStartingStyle
    }
    // cpp: style_rule.h:103-149
    pub fn IsViewTransitionRule(&self) -> bool {
        self.GetType() == RuleType::kViewTransition
    }
    // cpp: style_rule.h:103-149
    pub fn IsFunctionRule(&self) -> bool {
        self.GetType() == RuleType::kFunction
    }
    // cpp: style_rule.h:103-149
    pub fn IsMixinRule(&self) -> bool {
        self.GetType() == RuleType::kMixin
    }
    // cpp: style_rule.h:103-149
    pub fn IsResultRule(&self) -> bool {
        self.GetType() == RuleType::kResult
    }
    // cpp: style_rule.h:103-149
    pub fn IsPrivateRule(&self) -> bool {
        self.GetType() == RuleType::kPrivate
    }
    // cpp: style_rule.h:103-149
    pub fn IsApplyMixinRule(&self) -> bool {
        self.GetType() == RuleType::kApplyMixin
    }
    // cpp: style_rule.h:103-149
    pub fn IsContentsRule(&self) -> bool {
        self.GetType() == RuleType::kContents
    }
    // cpp: style_rule.h:103-149
    pub fn IsPositionTryRule(&self) -> bool {
        self.GetType() == RuleType::kPositionTry
    }
    // cpp: style_rule.h:103-149
    pub fn IsCustomMediaRule(&self) -> bool {
        self.GetType() == RuleType::kCustomMedia
    }
    // cpp: style_rule.h:139-142; deliberately differs from the condition cast.
    pub fn IsConditionRule(&self) -> bool {
        matches!(
            self.GetType(),
            RuleType::kContainer
                | RuleType::kMedia
                | RuleType::kSupports
                | RuleType::kStartingStyle
        )
    }
    // cpp: style_rule.h:908-918
    pub fn AsGroup(&self) -> Option<&StyleRuleGroup<D>> {
        match self {
            Self::Media(rule) => Some(&rule.condition_.group_),
            Self::Supports(rule) => Some(&rule.condition_.group_),
            Self::Container(rule) => Some(&rule.condition_.group_),
            Self::LayerBlock(rule) => Some(&rule.group_),
            Self::Scope(rule) => Some(&rule.group_),
            Self::StartingStyle(rule) => Some(&rule.group_),
            Self::Function(rule) => Some(&rule.group_),
            Self::Page(rule) => Some(&rule.group_),
            Self::Navigation(rule) => Some(&rule.condition_.group_),
            Self::Mixin(rule) => Some(&rule.group_),
            Self::Result(rule) => Some(&rule.group_),
            Self::ApplyMixin(rule) => Some(&rule.group_),
            Self::Contents(rule) => Some(&rule.group_),
            _ => None,
        }
    }
    // cpp: style_rule.h:1019-1023; Navigation qualifies, StartingStyle does not.
    pub fn AsCondition(&self) -> Option<&StyleRuleCondition<D>> {
        match self {
            Self::Media(rule) => Some(&rule.condition_),
            Self::Supports(rule) => Some(&rule.condition_),
            Self::Container(rule) => Some(&rule.condition_),
            Self::Navigation(rule) => Some(&rule.condition_),
            _ => None,
        }
    }
    // cpp: style_rule.cc:929-939
    pub fn LayerNameAsString(name_parts: &LayerName) -> String {
        LayerNameAsString(name_parts)
    }
}
// cpp: style_rule.cc:604-624
pub fn CloneRules<D: StyleRuleCloneDependencies>(
    old_rules: &[Rc<StyleRuleBase<D>>],
    new_parent: Option<Rc<StyleRule<D>>>,
    bindings: Option<Rc<D::MixinParameterBindings>>,
) -> Vec<Rc<StyleRuleBase<D>>>
where
    D::CSSPropertyValueSet: StyleRulePropertySetClone,
{
    old_rules
        .iter()
        .map(|rule| rule.Clone(new_parent.clone(), bindings.clone()))
        .collect()
}
// cpp: style_rule.cc:613-624. CopyWithRules is a concrete copy constructor of
// the selected group class, supplied at each dispatch branch below.
fn CloneGroupRule<D: StyleRuleCloneDependencies, R>(
    old_rules: &[Rc<StyleRuleBase<D>>],
    parent: Option<Rc<StyleRule<D>>>,
    bindings: Option<Rc<D::MixinParameterBindings>>,
    construct: impl FnOnce(Vec<Rc<StyleRuleBase<D>>>) -> R,
) -> R
where
    D::CSSPropertyValueSet: StyleRulePropertySetClone,
{
    construct(CloneRules(old_rules, parent, bindings))
}
// cpp: style_rule.cc:626-638
fn CloneSelectorListWithDummyFallback<D: StyleRuleCloneDependencies>(
    parent: Option<&Rc<StyleRule<D>>>,
) -> D::SelectorList {
    match parent {
        Some(parent) => D::CopySelectors(&parent.selectors_),
        None => D::DummyNestingSelectors(),
    }
}

impl<D: StyleRuleCloneDependencies> StyleRuleBase<D>
where
    D::CSSPropertyValueSet: StyleRulePropertySetClone,
{
    // cpp: style_rule.h:162-163; style_rule.cc:642-821
    pub fn Clone(
        &self,
        new_parent: Option<Rc<StyleRule<D>>>,
        bindings: Option<Rc<D::MixinParameterBindings>>,
    ) -> Rc<Self> {
        let immutable = |properties: &Rc<D::CSSPropertyValueSet>| {
            <D::CSSPropertyValueSet as StyleRulePropertySetClone>::ImmutableCopyIfNeeded(properties)
        };
        let result = match self {
            Self::Style(rule) => {
                let selectors = D::RenestStyleSelectors(&rule.selectors_, new_parent);
                let new_rule = Rc::new(StyleRule::Create(
                    selectors,
                    immutable(&rule.Properties()),
                    bindings.clone(),
                ));
                if let Some(children) = rule.ChildRules() {
                    for child in children.iter() {
                        new_rule.AddChildRule(
                            child.Clone(Some(Rc::clone(&new_rule)), bindings.clone()),
                        );
                    }
                }
                Self::Style(new_rule)
            }
            Self::Scope(rule) => {
                let scope = D::CloneStyleScope(rule.GetStyleScope(), new_parent);
                let children =
                    CloneRules(rule.ChildRules(), D::ScopeRuleForNesting(&scope), bindings);
                Self::Scope(StyleRuleScope::new(scope, children))
            }
            Self::LayerBlock(rule) => Self::LayerBlock(CloneGroupRule(
                rule.ChildRules(),
                new_parent,
                bindings,
                |children| StyleRuleLayerBlock::CopyWithRules(rule, children),
            )),
            Self::Container(rule) => Self::Container(StyleRuleContainer::new(
                D::CopyContainerQuerySet(rule.GetContainerQuerySet()),
                CloneRules(rule.ChildRules(), new_parent, bindings),
            )),
            Self::Media(rule) => Self::Media(CloneGroupRule(
                rule.ChildRules(),
                new_parent,
                bindings,
                |children| StyleRuleMedia::CopyWithRules(rule, children),
            )),
            Self::Location(rule) => Self::Location(D::CopyLocation(rule)),
            Self::Navigation(rule) => Self::Navigation(CloneGroupRule(
                rule.ChildRules(),
                new_parent,
                bindings,
                |children| StyleRuleNavigation::CopyWithRules(rule, children),
            )),
            Self::Supports(rule) => Self::Supports(CloneGroupRule(
                rule.ChildRules(),
                new_parent,
                bindings,
                |children| StyleRuleSupports::CopyWithRules(rule, children),
            )),
            Self::StartingStyle(rule) => Self::StartingStyle(CloneGroupRule(
                rule.ChildRules(),
                new_parent,
                bindings,
                |children| StyleRuleStartingStyle::CopyWithRules(rule, children),
            )),
            Self::Page(rule) => Self::Page(StyleRulePage::new(
                D::RenestPageSelectors(&rule.selector_list_, new_parent.clone()),
                immutable(&rule.properties_),
                CloneRules(rule.ChildRules(), new_parent, bindings),
            )),
            Self::Mixin(rule) => Self::Mixin(CloneGroupRule(
                rule.ChildRules(),
                new_parent,
                bindings,
                |children| StyleRuleMixin::CopyWithRules(rule, children),
            )),
            Self::Result(rule) => Self::Result(CloneGroupRule(
                rule.ChildRules(),
                new_parent,
                bindings,
                |children| StyleRuleResult::CopyWithRules(rule, children),
            )),
            Self::ApplyMixin(rule) => Self::ApplyMixin(CloneGroupRule(
                rule.ChildRules(),
                new_parent,
                bindings,
                |children| StyleRuleApplyMixin::CopyWithRules(rule, children),
            )),
            Self::Contents(rule) => Self::Contents(CloneGroupRule(
                rule.ChildRules(),
                new_parent,
                bindings,
                |children| StyleRuleContentsStatement::CopyWithRules(rule, children),
            )),
            Self::NestedDeclarations(rule) => {
                let nesting = D::NestedDeclarationsNestingType(rule);
                let old_inner = D::NestedDeclarationsInnerRule(rule);
                let selectors = if nesting == CSSNestingType::kScope {
                    D::CopySelectors(&old_inner.selectors_)
                } else {
                    CloneSelectorListWithDummyFallback::<D>(new_parent.as_ref())
                };
                let inner = Rc::new(StyleRule::Create(
                    selectors,
                    immutable(&old_inner.Properties()),
                    bindings,
                ));
                Self::NestedDeclarations(D::NewNestedDeclarations(nesting, inner))
            }
            Self::FunctionDeclarations(rule) => {
                Self::FunctionDeclarations(D::CopyFunctionDeclarations(rule))
            }
            Self::Function(rule) => Self::Function(StyleRuleFunction::new(
                rule.Name().clone(),
                rule.GetParameters().clone(),
                CloneRules(rule.ChildRules(), new_parent, bindings),
                rule.GetReturnType().clone(),
            )),
            Self::Property(rule) => Self::Property(StyleRuleProperty::Copy(rule)),
            Self::PageMargin(rule) => Self::PageMargin(StyleRulePageMargin::Copy(rule)),
            Self::FontFace(rule) => Self::FontFace(StyleRuleFontFace::Copy(rule)),
            Self::FontPaletteValues(rule) => {
                Self::FontPaletteValues(D::CopyFontPaletteValues(rule))
            }
            Self::FontFeatureValues(rule) => {
                Self::FontFeatureValues(Rc::new(rule.as_ref().clone()))
            }
            Self::FontFeature(rule) => Self::FontFeature(Rc::new(rule.as_ref().clone())),
            Self::Import(rule) => Self::Import(D::CopyImport(rule)),
            Self::Keyframes(rule) => {
                let keys = D::Keyframes(rule)
                    .iter()
                    .map(|key| {
                        D::NewKeyframe(
                            D::KeyframeKeys(key).to_vec(),
                            immutable(&D::KeyframeProperties(key)),
                        )
                    })
                    .collect();
                Self::Keyframes(D::NewKeyframes(
                    keys,
                    D::KeyframesName(rule),
                    D::KeyframesVersion(rule),
                    D::KeyframesIsVendorPrefixed(rule),
                ))
            }
            Self::LayerStatement(rule) => Self::LayerStatement(rule.clone()),
            Self::Namespace(rule) => Self::Namespace(D::CopyNamespace(rule)),
            Self::CounterStyle(rule) => Self::CounterStyle(D::NewCounterStyle(
                D::CounterStyleName(rule),
                immutable(&D::CounterStyleProperties(rule)),
            )),
            Self::Keyframe(rule) => Self::Keyframe(D::NewKeyframe(
                D::KeyframeKeys(rule).to_vec(),
                immutable(&D::KeyframeProperties(rule)),
            )),
            Self::Charset(_) => Self::Charset(StyleRuleCharset::new()),
            Self::ViewTransition(rule) => Self::ViewTransition(D::CopyViewTransition(rule)),
            Self::PositionTry(rule) => Self::PositionTry(D::NewPositionTry(
                D::PositionTryName(rule),
                immutable(&D::PositionTryProperties(rule)),
            )),
            Self::CustomMedia(rule) => Self::CustomMedia(StyleRuleCustomMedia {
                name_: rule.name_.clone(),
                media_query_value_: rule.media_query_value_.clone(),
                boolean_value_: rule.boolean_value_,
            }),
            Self::Private(rule) => Self::Private(StyleRulePrivate::Copy(rule)),
        };
        Rc::new(result)
    }
}

// Connect the real rule type to CSSParserImpl rule-list ordering.
impl<D: StyleRuleDependencies> CSSRuleOrdering for StyleRuleBase<D> {
    fn IsCharsetRule(&self) -> bool {
        StyleRuleBase::IsCharsetRule(self)
    }
    fn IsLayerStatementRule(&self) -> bool {
        StyleRuleBase::IsLayerStatementRule(self)
    }
    fn IsImportRule(&self) -> bool {
        StyleRuleBase::IsImportRule(self)
    }
    fn IsNamespaceRule(&self) -> bool {
        StyleRuleBase::IsNamespaceRule(self)
    }
}

// cpp: style_rule.cc:929-939
pub fn LayerNameAsString(name_parts: &LayerName) -> String {
    let mut result = Vec::new();
    for part in name_parts {
        if !result.is_empty() {
            result.push(b'.' as u16);
        }
        // AtomicString -> String retains the original UTF-16 content.
        let units: Vec<u16> = (0..part.length()).map(|i| part.at(i)).collect();
        SerializeIdentifierTo(&String::from_utf16(&units), &mut result, false);
    }
    String::from_utf16(&result)
}

// cpp: style_rule.h:837-858
pub fn ReplaceStyleRuleInVector<R>(
    old_rule: &Rc<R>,
    new_rule: Rc<R>,
    position_hint: usize,
    child_rules: &mut [Rc<R>],
) -> usize {
    if position_hint < child_rules.len() && Rc::ptr_eq(&child_rules[position_hint], old_rule) {
        child_rules[position_hint] = new_rule;
        return position_hint;
    }
    for (i, rule) in child_rules.iter_mut().enumerate() {
        if Rc::ptr_eq(rule, old_rule) {
            *rule = new_rule;
            return i;
        }
    }
    usize::MAX
}

// cpp: style_rule.h:348-359
pub struct StyleRule<D: StyleRuleDependencies> {
    selectors_: D::SelectorList,
    properties_: RefCell<Option<Rc<D::CSSPropertyValueSet>>>,
    lazy_state_: RefCell<Option<Rc<D::CSSLazyParsingState>>>,
    child_rules_: RefCell<Option<Vec<Rc<StyleRuleBase<D>>>>>,
    mixin_parameter_bindings_: Option<Rc<D::MixinParameterBindings>>,
    lazy_offset_: usize,
}
impl<D: StyleRuleDependencies> StyleRule<D> {
    // cpp: style_rule.h:207-261; style_rule.cc:482-517
    pub fn Create(
        selectors: D::SelectorList,
        properties: Rc<D::CSSPropertyValueSet>,
        mixin_parameter_bindings: Option<Rc<D::MixinParameterBindings>>,
    ) -> Self {
        Self {
            selectors_: selectors,
            properties_: RefCell::new(Some(properties)),
            lazy_state_: RefCell::new(None),
            child_rules_: RefCell::new(None),
            mixin_parameter_bindings_: mixin_parameter_bindings,
            lazy_offset_: 0,
        }
    }
    pub fn CreateLazy(
        selectors: D::SelectorList,
        lazy_state: Rc<D::CSSLazyParsingState>,
        lazy_offset: usize,
    ) -> Self {
        Self {
            selectors_: selectors,
            properties_: RefCell::new(None),
            lazy_state_: RefCell::new(Some(lazy_state)),
            child_rules_: RefCell::new(None),
            mixin_parameter_bindings_: None,
            lazy_offset_: lazy_offset,
        }
    }
    // Source intentionally allows incomplete construction until SetProperties.
    pub fn CreateWithoutProperties(selectors: D::SelectorList) -> Self {
        Self {
            selectors_: selectors,
            properties_: RefCell::new(None),
            lazy_state_: RefCell::new(None),
            child_rules_: RefCell::new(None),
            mixin_parameter_bindings_: None,
            lazy_offset_: 0,
        }
    }
    // cpp: style_rule.cc:509-517: source does NOT transfer mixin bindings.
    pub fn CreateWithSelectors(selectors: D::SelectorList, other: Self) -> Self {
        Self {
            selectors_: selectors,
            properties_: other.properties_,
            lazy_state_: other.lazy_state_,
            child_rules_: other.child_rules_,
            mixin_parameter_bindings_: None,
            lazy_offset_: other.lazy_offset_,
        }
    }
    // cpp: style_rule.h:265-268
    pub fn SetProperties(&self, properties: Rc<D::CSSPropertyValueSet>) {
        debug_assert!(self.properties_.borrow().is_none());
        *self.properties_.borrow_mut() = Some(properties);
    }
    // Borrow the actual selector storage for the CSSSelectorParentRule adapter.
    pub fn Selectors(&self) -> &D::SelectorList {
        &self.selectors_
    }
    // cpp: style_rule.h:296; style_rule.cc:477-480
    pub fn AverageSizeInBytes() -> usize
    where
        D::CSSPropertyValueSet: StyleRulePropertySetClone,
    {
        std::mem::size_of::<Self>()
            + std::mem::size_of::<<D::SelectorList as StyleRuleSelectorList>::CSSSelector>()
            + <D::CSSPropertyValueSet as StyleRulePropertySetClone>::AverageSizeInBytes()
    }
    // cpp: style_rule.h:271-291
    pub fn FirstSelector(&self) -> &<D::SelectorList as StyleRuleSelectorList>::CSSSelector {
        self.selectors_.FirstSelector()
    }
    pub fn SelectorAt(
        &self,
        index: usize,
    ) -> &<D::SelectorList as StyleRuleSelectorList>::CSSSelector {
        self.selectors_.SelectorAt(index)
    }
    pub fn MutableSelectorAt(
        &mut self,
        index: usize,
    ) -> &mut <D::SelectorList as StyleRuleSelectorList>::CSSSelector {
        self.selectors_.MutableSelectorAt(index)
    }
    pub fn SelectorIndex(
        &self,
        selector: &<D::SelectorList as StyleRuleSelectorList>::CSSSelector,
    ) -> usize {
        self.selectors_.SelectorIndex(selector)
    }
    pub fn IndexOfNextSelectorAfter(&self, index: usize) -> usize {
        let current = self.SelectorAt(index);
        match self.selectors_.NextSelector(current) {
            None => usize::MAX,
            Some(next) => self.SelectorIndex(next),
        }
    }
    pub fn SelectorsText(&self) -> String {
        self.selectors_.SelectorsText()
    }
    // cpp: style_rule.cc:520-527
    pub fn Properties(&self) -> Rc<D::CSSPropertyValueSet> {
        if self.properties_.borrow().is_none() {
            let properties = {
                let state = self.lazy_state_.borrow();
                D::ParseDeclarationListForLazyStyle(
                    state
                        .as_ref()
                        .expect("incomplete StyleRule: no lazy state or properties"),
                    self.lazy_offset_,
                )
            };
            *self.properties_.borrow_mut() = Some(properties);
            self.lazy_state_.borrow_mut().take();
        }
        Rc::clone(self.properties_.borrow().as_ref().unwrap())
    }
    // cpp: style_rule.cc:542-548
    pub fn MutableProperties(&mut self) -> Rc<D::CSSPropertyValueSet> {
        let properties = self.Properties();
        if !properties.IsMutable() {
            *self.properties_.get_mut() = Some(properties.MutableCopy());
        }
        let properties = Rc::clone(self.properties_.get_mut().as_ref().unwrap());
        debug_assert!(properties.IsMutable());
        properties
    }
    // cpp: style_rule.cc:577-586
    pub fn PropertiesHaveFailedOrCanceledSubresources(&self) -> bool {
        self.properties_
            .borrow()
            .as_ref()
            .is_some_and(|p| p.HasFailedOrCanceledSubresources())
    }
    pub fn HasParsedProperties(&self) -> bool {
        debug_assert!(self.lazy_state_.borrow().is_some() || self.properties_.borrow().is_some());
        debug_assert!(self.lazy_state_.borrow().is_none() || self.properties_.borrow().is_none());
        self.lazy_state_.borrow().is_none()
    }
    // cpp: style_rule.h:303-332
    pub fn ChildRules(&self) -> Option<Ref<'_, Vec<Rc<StyleRuleBase<D>>>>> {
        Ref::filter_map(self.child_rules_.borrow(), |children| children.as_ref()).ok()
    }
    pub fn ChildRulesMut(&mut self) -> Option<RefMut<'_, Vec<Rc<StyleRuleBase<D>>>>> {
        RefMut::filter_map(self.child_rules_.borrow_mut(), |children| children.as_mut()).ok()
    }
    pub fn GetMixinParameterBindings(&self) -> Option<&Rc<D::MixinParameterBindings>> {
        self.mixin_parameter_bindings_.as_ref()
    }
    pub fn EnsureChildRules(&mut self) {
        if self.child_rules_.get_mut().is_none() {
            *self.child_rules_.get_mut() = Some(Vec::new());
        }
    }
    pub fn AddChildRule(&self, child: Rc<StyleRuleBase<D>>) {
        self.child_rules_
            .borrow_mut()
            .get_or_insert_with(Vec::new)
            .push(child);
    }
    // cpp: style_rule.cc:550-558
    pub fn ReplaceChildRuleIfExists(
        &mut self,
        old_rule: &Rc<StyleRuleBase<D>>,
        new_rule: Rc<StyleRuleBase<D>>,
        position_hint: usize,
    ) -> usize {
        match self.child_rules_.get_mut().as_mut() {
            Some(children) => ReplaceStyleRuleInVector(old_rule, new_rule, position_hint, children),
            None => usize::MAX,
        }
    }
    // cpp: style_rule.cc:560-575
    pub fn WrapperInsertRule(
        &mut self,
        parent_sheet: Option<&mut dyn StyleRuleChangeObserver<Rc<StyleRuleBase<D>>>>,
        index: usize,
        rule: Rc<StyleRuleBase<D>>,
    ) {
        self.EnsureChildRules();
        self.child_rules_
            .get_mut()
            .as_mut()
            .unwrap()
            .insert(index, Rc::clone(&rule));
        if let Some(sheet) = parent_sheet {
            sheet.NotifyRuleChanged(&rule);
        }
    }
    pub fn WrapperRemoveRule(
        &mut self,
        parent_sheet: Option<&mut dyn StyleRuleChangeObserver<Rc<StyleRuleBase<D>>>>,
        index: usize,
    ) {
        if let Some(sheet) = parent_sheet {
            sheet.NotifyRuleChanged(
                &self
                    .child_rules_
                    .get_mut()
                    .as_ref()
                    .expect("child rules missing")[index],
            );
        }
        self.child_rules_
            .get_mut()
            .as_mut()
            .expect("child rules missing")
            .remove(index);
    }
}

// cpp: style_rule.h:398-422; style_rule.cc:895-921
pub struct StyleRuleGroup<D: StyleRuleDependencies> {
    type_: RuleType,
    child_rules_: Vec<Rc<StyleRuleBase<D>>>,
}
impl<D: StyleRuleDependencies> StyleRuleGroup<D> {
    fn new(type_: RuleType, rules: Vec<Rc<StyleRuleBase<D>>>) -> Self {
        Self {
            type_,
            child_rules_: rules,
        }
    }
    pub fn GetType(&self) -> RuleType {
        self.type_
    }
    pub fn ChildRules(&self) -> &Vec<Rc<StyleRuleBase<D>>> {
        &self.child_rules_
    }
    pub fn ChildRulesMut(&mut self) -> &mut Vec<Rc<StyleRuleBase<D>>> {
        &mut self.child_rules_
    }
    pub fn ReplaceChildRuleIfExists(
        &mut self,
        old_rule: &Rc<StyleRuleBase<D>>,
        new_rule: Rc<StyleRuleBase<D>>,
        position_hint: usize,
    ) -> usize {
        ReplaceStyleRuleInVector(old_rule, new_rule, position_hint, &mut self.child_rules_)
    }
    pub fn WrapperInsertRule(
        &mut self,
        parent_sheet: Option<&mut dyn StyleRuleChangeObserver<Rc<StyleRuleBase<D>>>>,
        index: usize,
        rule: Rc<StyleRuleBase<D>>,
    ) {
        self.child_rules_.insert(index, Rc::clone(&rule));
        if let Some(sheet) = parent_sheet {
            sheet.NotifyRuleChanged(&rule);
        }
    }
    pub fn WrapperRemoveRule(
        &mut self,
        parent_sheet: Option<&mut dyn StyleRuleChangeObserver<Rc<StyleRuleBase<D>>>>,
        index: usize,
    ) {
        if let Some(sheet) = parent_sheet {
            sheet.NotifyRuleChanged(&self.child_rules_[index]);
        }
        self.child_rules_.remove(index);
    }
}
// cpp: style_rule.h:509-523; style_rule.cc:1020-1027
pub struct StyleRuleCondition<D: StyleRuleDependencies> {
    group_: StyleRuleGroup<D>,
    condition_text_: String,
}
impl<D: StyleRuleDependencies> StyleRuleCondition<D> {
    fn new(type_: RuleType, rules: Vec<Rc<StyleRuleBase<D>>>) -> Self {
        Self {
            group_: StyleRuleGroup::new(type_, rules),
            condition_text_: String::default(),
        }
    }
    fn with_text(
        type_: RuleType,
        condition_text: String,
        rules: Vec<Rc<StyleRuleBase<D>>>,
    ) -> Self {
        Self {
            group_: StyleRuleGroup::new(type_, rules),
            condition_text_: condition_text,
        }
    }
    pub fn ConditionText(&self) -> String {
        self.condition_text_.clone()
    }
    pub fn Group(&self) -> &StyleRuleGroup<D> {
        &self.group_
    }
    pub fn GroupMut(&mut self) -> &mut StyleRuleGroup<D> {
        &mut self.group_
    }
}

// cpp: style_rule.h:362-397,468-508; style_rule.cc:823-881,978-1013
// The base pointer changes to a MutableCopy only when the real store is immutable.
fn EnsureMutable<P: StyleRulePropertySet>(properties: &mut Rc<P>) -> Rc<P> {
    if !properties.IsMutable() {
        *properties = properties.MutableCopy();
    }
    debug_assert!(properties.IsMutable());
    Rc::clone(properties)
}

// cpp: style_rule.h:362-374; style_rule.cc:867-879
pub struct StyleRuleFontFace<D: StyleRuleDependencies> {
    properties_: Rc<D::CSSPropertyValueSet>,
}
impl<D: StyleRuleDependencies> StyleRuleFontFace<D> {
    pub fn new(properties: Rc<D::CSSPropertyValueSet>) -> Self {
        Self {
            properties_: properties,
        }
    }
    pub fn Copy(other: &Self) -> Self {
        Self::new(other.properties_.MutableCopy())
    }
    pub fn Properties(&self) -> &D::CSSPropertyValueSet {
        &self.properties_
    }
    pub fn MutableProperties(&mut self) -> Rc<D::CSSPropertyValueSet> {
        EnsureMutable(&mut self.properties_)
    }
}
// cpp: style_rule.h:376-393; style_rule.cc:823-860
pub struct StyleRuleProperty<D: StyleRuleDependencies> {
    name_: String,
    properties_: Rc<D::CSSPropertyValueSet>,
}
impl<D: StyleRuleDependencies> StyleRuleProperty<D> {
    pub fn new(name: String, properties: Rc<D::CSSPropertyValueSet>) -> Self {
        Self {
            name_: name,
            properties_: properties,
        }
    }
    pub fn Copy(other: &Self) -> Self {
        Self::new(other.name_.clone(), other.properties_.MutableCopy())
    }
    pub fn Properties(&self) -> &D::CSSPropertyValueSet {
        &self.properties_
    }
    pub fn MutableProperties(&mut self) -> Rc<D::CSSPropertyValueSet> {
        EnsureMutable(&mut self.properties_)
    }
    pub fn GetName(&self) -> &String {
        &self.name_
    }
    pub fn GetSyntax(
        &self,
    ) -> Option<Rc<<D::CSSPropertyValueSet as StyleRulePropertySet>::CSSValue>> {
        self.properties_.GetPropertyCSSValue(CSSPropertyID::kSyntax)
    }
    pub fn Inherits(
        &self,
    ) -> Option<Rc<<D::CSSPropertyValueSet as StyleRulePropertySet>::CSSValue>> {
        self.properties_
            .GetPropertyCSSValue(CSSPropertyID::kInherits)
    }
    pub fn GetInitialValue(
        &self,
    ) -> Option<Rc<<D::CSSPropertyValueSet as StyleRulePropertySet>::CSSValue>> {
        self.properties_
            .GetPropertyCSSValue(CSSPropertyID::kInitialValue)
    }
    pub fn SetNameText(
        &mut self,
        _execution_context: Option<&D::ExecutionContext>,
        name_text: &String,
    ) -> bool {
        debug_assert!(!name_text.IsNull());
        let name = D::ParseCustomPropertyName(name_text);
        if name.IsNull() {
            return false;
        }
        self.name_ = name;
        true
    }
}
// cpp: style_rule.h:423-435; style_rule.cc:886-888
pub struct StyleRuleScope<D: StyleRuleDependencies> {
    group_: StyleRuleGroup<D>,
    style_scope_: Rc<D::StyleScope>,
}
impl<D: StyleRuleDependencies> StyleRuleScope<D> {
    pub fn new(style_scope: Rc<D::StyleScope>, rules: Vec<Rc<StyleRuleBase<D>>>) -> Self {
        Self {
            group_: StyleRuleGroup::new(RuleType::kScope, rules),
            style_scope_: style_scope,
        }
    }
    pub fn GetStyleScope(&self) -> &D::StyleScope {
        &self.style_scope_
    }
}
// cpp: style_rule.h:437-453; style_rule.cc:941-949,955-957
pub struct StyleRuleLayerBlock<D: StyleRuleDependencies> {
    group_: StyleRuleGroup<D>,
    name_: LayerName,
}
impl<D: StyleRuleDependencies> StyleRuleLayerBlock<D> {
    pub fn new(name: LayerName, rules: Vec<Rc<StyleRuleBase<D>>>) -> Self {
        Self {
            group_: StyleRuleGroup::new(RuleType::kLayerBlock, rules),
            name_: name,
        }
    }
    pub fn CopyWithRules(other: &Self, rules: Vec<Rc<StyleRuleBase<D>>>) -> Self {
        Self::new(other.name_.clone(), rules)
    }
    pub fn GetName(&self) -> &LayerName {
        &self.name_
    }
    pub fn GetNameAsString(&self) -> String {
        LayerNameAsString(&self.name_)
    }
}
// cpp: style_rule.h:454-467; style_rule.cc:959-963,970-976
#[derive(Clone)]
pub struct StyleRuleLayerStatement {
    names_: Vec<LayerName>,
}
impl StyleRuleLayerStatement {
    pub fn new(names: Vec<LayerName>) -> Self {
        Self { names_: names }
    }
    pub fn GetNames(&self) -> &Vec<LayerName> {
        &self.names_
    }
    pub fn GetNamesAsStrings(&self) -> Vec<String> {
        self.names_.iter().map(LayerNameAsString).collect()
    }
}
// cpp: style_rule.h:468-489; style_rule.cc:978-990
pub struct StyleRulePage<D: StyleRuleDependencies> {
    group_: StyleRuleGroup<D>,
    properties_: Rc<D::CSSPropertyValueSet>,
    selector_list_: Rc<D::SelectorList>,
}
impl<D: StyleRuleDependencies> StyleRulePage<D> {
    pub fn new(
        selector_list: Rc<D::SelectorList>,
        properties: Rc<D::CSSPropertyValueSet>,
        child_rules: Vec<Rc<StyleRuleBase<D>>>,
    ) -> Self {
        Self {
            group_: StyleRuleGroup::new(RuleType::kPage, child_rules),
            properties_: properties,
            selector_list_: selector_list,
        }
    }
    pub fn Selector(&self) -> &<D::SelectorList as StyleRuleSelectorList>::CSSSelector {
        self.selector_list_.FirstSelector()
    }
    pub fn SelectorList(&self) -> &D::SelectorList {
        &self.selector_list_
    }
    pub fn Properties(&self) -> &D::CSSPropertyValueSet {
        &self.properties_
    }
    pub fn MutableProperties(&mut self) -> Rc<D::CSSPropertyValueSet> {
        EnsureMutable(&mut self.properties_)
    }
    pub fn WrapperAdoptSelectorList(&mut self, selectors: Rc<D::SelectorList>) {
        self.selector_list_ = selectors;
    }
}
// cpp: style_rule.h:491-507; style_rule.cc:998-1013
pub struct StyleRulePageMargin<D: StyleRuleDependencies> {
    id_: CSSAtRuleID,
    properties_: Rc<D::CSSPropertyValueSet>,
}
impl<D: StyleRuleDependencies> StyleRulePageMargin<D> {
    pub fn new(id: CSSAtRuleID, properties: Rc<D::CSSPropertyValueSet>) -> Self {
        Self {
            id_: id,
            properties_: properties,
        }
    }
    pub fn Copy(other: &Self) -> Self {
        Self::new(other.id_, other.properties_.MutableCopy())
    }
    pub fn ID(&self) -> CSSAtRuleID {
        self.id_
    }
    pub fn Properties(&self) -> &D::CSSPropertyValueSet {
        &self.properties_
    }
    pub fn MutableProperties(&mut self) -> Rc<D::CSSPropertyValueSet> {
        EnsureMutable(&mut self.properties_)
    }
}
// cpp: style_rule.h:526-543; style_rule.cc:1029-1035
pub struct StyleRuleMedia<D: StyleRuleDependencies> {
    condition_: StyleRuleCondition<D>,
    media_queries_: Option<Rc<D::MediaQuerySet>>,
}
impl<D: StyleRuleDependencies> StyleRuleMedia<D> {
    pub fn new(media: Option<Rc<D::MediaQuerySet>>, rules: Vec<Rc<StyleRuleBase<D>>>) -> Self {
        Self {
            condition_: StyleRuleCondition::new(RuleType::kMedia, rules),
            media_queries_: media,
        }
    }
    pub fn CopyWithRules(other: &Self, rules: Vec<Rc<StyleRuleBase<D>>>) -> Self {
        Self::new(other.media_queries_.clone(), rules)
    }
    pub fn MediaQueries(&self) -> Option<&D::MediaQuerySet> {
        self.media_queries_.as_deref()
    }
    pub fn SetMediaQueries(&mut self, media_queries: Option<Rc<D::MediaQuerySet>>) {
        self.media_queries_ = media_queries;
    }
}
// cpp: style_rule.h:545-563; style_rule.cc:1042-1069
pub struct StyleRuleSupports<D: StyleRuleDependencies> {
    condition_: StyleRuleCondition<D>,
    condition_is_supported_: bool,
}
impl<D: StyleRuleDependencies> StyleRuleSupports<D> {
    pub fn new(
        condition_text: String,
        condition_is_supported: bool,
        rules: Vec<Rc<StyleRuleBase<D>>>,
    ) -> Self {
        Self {
            condition_: StyleRuleCondition::with_text(RuleType::kSupports, condition_text, rules),
            condition_is_supported_: condition_is_supported,
        }
    }
    pub fn CopyWithRules(other: &Self, rules: Vec<Rc<StyleRuleBase<D>>>) -> Self {
        Self::new(
            other.condition_.condition_text_.clone(),
            other.condition_is_supported_,
            rules,
        )
    }
    pub fn ConditionIsSupported(&self) -> bool {
        self.condition_is_supported_
    }
    pub fn SetConditionText(
        &mut self,
        execution_context: &D::ExecutionContext,
        parent_contents: Option<&mut dyn StyleRuleChangeObserver<Self>>,
        value: String,
    ) {
        let supported = D::ConsumeSupportsCondition(execution_context, &value);
        self.condition_.condition_text_ = value;
        self.condition_is_supported_ = supported;
        if let Some(contents) = parent_contents {
            contents.NotifyRuleChanged(self);
        }
    }
}
// cpp: style_rule.h:565-588; style_rule.cc:1071-1123
pub struct StyleRuleContainer<D: StyleRuleDependencies> {
    condition_: StyleRuleCondition<D>,
    container_query_set_: Rc<D::ContainerQuerySet>,
}
impl<D: StyleRuleDependencies> StyleRuleContainer<D> {
    pub fn new(
        container_query_set: Rc<D::ContainerQuerySet>,
        rules: Vec<Rc<StyleRuleBase<D>>>,
    ) -> Self {
        Self {
            condition_: StyleRuleCondition::with_text(
                RuleType::kContainer,
                D::ContainerQuerySetToString(&container_query_set),
                rules,
            ),
            container_query_set_: container_query_set,
        }
    }
    pub fn CopyWithRules(other: &Self, rules: Vec<Rc<StyleRuleBase<D>>>) -> Self {
        Self::new(Rc::clone(&other.container_query_set_), rules)
    }
    pub fn GetContainerQuerySet(&self) -> &D::ContainerQuerySet {
        &self.container_query_set_
    }
    pub fn SetConditionText(
        &mut self,
        execution_context: &D::ExecutionContext,
        parent_contents: Option<&mut dyn StyleRuleChangeObserver<Self>>,
        value: String,
    ) {
        if let Some(set) = D::ParseContainerQuerySet(execution_context, &value) {
            self.condition_.condition_text_ = D::ContainerQuerySetToString(&set);
            self.container_query_set_ = set;
            if let Some(contents) = parent_contents {
                contents.NotifyRuleChanged(self);
            }
        }
    }
    pub fn SetQueryText(
        &mut self,
        execution_context: &D::ExecutionContext,
        parent_contents: Option<&mut dyn StyleRuleChangeObserver<Self>>,
        value: String,
    ) {
        let Some(query) = D::SingleContainerQuery(&self.container_query_set_) else {
            return;
        };
        if let Some(exp_node) = D::ParseContainerCondition(execution_context, &value) {
            self.condition_.condition_text_ = D::SerializeContainerCondition(&exp_node);
            let selector_name = D::ContainerQuerySelectorName(query);
            let queries = vec![D::NewContainerQuery(selector_name, exp_node)];
            self.container_query_set_ = D::NewContainerQuerySet(queries);
            if let Some(contents) = parent_contents {
                contents.NotifyRuleChanged(self);
            }
        }
    }
}
// cpp: style_rule.h:590-609; style_rule.cc:1130-1140,1147-1160
pub struct StyleRuleNavigation<D: StyleRuleDependencies> {
    condition_: StyleRuleCondition<D>,
    navigation_query_: Rc<D::NavigationQuery>,
}
impl<D: StyleRuleDependencies> StyleRuleNavigation<D> {
    pub fn new(query: Rc<D::NavigationQuery>, rules: Vec<Rc<StyleRuleBase<D>>>) -> Self {
        Self {
            condition_: StyleRuleCondition::new(RuleType::kNavigation, rules),
            navigation_query_: query,
        }
    }
    pub fn CopyWithRules(other: &Self, rules: Vec<Rc<StyleRuleBase<D>>>) -> Self {
        Self::new(Rc::clone(&other.navigation_query_), rules)
    }
    pub fn GetNavigationQuery(&self) -> &D::NavigationQuery {
        &self.navigation_query_
    }
    pub fn SetConditionText(
        &mut self,
        _execution_context: Option<&D::ExecutionContext>,
        parent_contents: Option<&mut dyn StyleRuleChangeObserver<Self>>,
        value: String,
    ) {
        if let Some(query) = D::ParseNavigationQuery(&value) {
            self.navigation_query_ = query;
            if let Some(contents) = parent_contents {
                contents.NotifyRuleChanged(self);
            }
        }
        // The source intentionally does not assign condition_text_ here.
    }
}
// cpp: style_rule.h:610-623; style_rule.cc:1162-1164
pub struct StyleRuleStartingStyle<D: StyleRuleDependencies> {
    group_: StyleRuleGroup<D>,
}
impl<D: StyleRuleDependencies> StyleRuleStartingStyle<D> {
    pub fn new(rules: Vec<Rc<StyleRuleBase<D>>>) -> Self {
        Self {
            group_: StyleRuleGroup::new(RuleType::kStartingStyle, rules),
        }
    }
    pub fn CopyWithRules(_other: &Self, rules: Vec<Rc<StyleRuleBase<D>>>) -> Self {
        Self::new(rules)
    }
}
// cpp: style_rule.h:624-634
pub struct StyleRuleCharset;
impl StyleRuleCharset {
    pub fn new() -> Self {
        Self
    }
}

// cpp: style_rule.h:637-647
pub struct StyleRuleFunctionParameter<D: StyleRuleDependencies> {
    pub name: String,
    pub r#type: D::CSSSyntaxDefinition,
    pub default_value: Option<Rc<D::CSSVariableData>>,
}
impl<D: StyleRuleDependencies> Clone for StyleRuleFunctionParameter<D> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            r#type: self.r#type.clone(),
            default_value: self.default_value.clone(),
        }
    }
}
// cpp: style_rule.h:635-690; style_rule.cc:1170-1178
pub struct StyleRuleFunction<D: StyleRuleDependencies> {
    group_: StyleRuleGroup<D>,
    name_: AtomicString,
    parameters_: Vec<StyleRuleFunctionParameter<D>>,
    return_type_: D::CSSSyntaxDefinition,
}
impl<D: StyleRuleDependencies> StyleRuleFunction<D> {
    pub fn new(
        name: AtomicString,
        parameters: Vec<StyleRuleFunctionParameter<D>>,
        child_rules: Vec<Rc<StyleRuleBase<D>>>,
        return_type: D::CSSSyntaxDefinition,
    ) -> Self {
        Self {
            group_: StyleRuleGroup::new(RuleType::kFunction, child_rules),
            name_: name,
            parameters_: parameters,
            return_type_: return_type,
        }
    }
    pub fn Name(&self) -> &AtomicString {
        &self.name_
    }
    pub fn GetParameters(&self) -> &Vec<StyleRuleFunctionParameter<D>> {
        &self.parameters_
    }
    pub fn GetReturnType(&self) -> &D::CSSSyntaxDefinition {
        &self.return_type_
    }
}
// cpp: style_rule.h:692-714; style_rule.cc:1185-1197
pub struct StyleRuleMixin<D: StyleRuleDependencies> {
    group_: StyleRuleGroup<D>,
    name_: AtomicString,
    parameters_: Vec<StyleRuleFunctionParameter<D>>,
}
impl<D: StyleRuleDependencies> StyleRuleMixin<D> {
    pub fn new(
        name: AtomicString,
        parameters: Vec<StyleRuleFunctionParameter<D>>,
        child_rules: Vec<Rc<StyleRuleBase<D>>>,
    ) -> Self {
        Self {
            group_: StyleRuleGroup::new(RuleType::kMixin, child_rules),
            name_: name,
            parameters_: parameters,
        }
    }
    pub fn CopyWithRules(other: &Self, child_rules: Vec<Rc<StyleRuleBase<D>>>) -> Self {
        Self::new(other.name_.clone(), other.parameters_.clone(), child_rules)
    }
    pub fn GetName(&self) -> &AtomicString {
        &self.name_
    }
    pub fn GetParameters(&self) -> &Vec<StyleRuleFunctionParameter<D>> {
        &self.parameters_
    }
}
// cpp: style_rule.h:715-724; style_rule.cc:1204-1210
pub struct StyleRuleResult<D: StyleRuleDependencies> {
    group_: StyleRuleGroup<D>,
}
impl<D: StyleRuleDependencies> StyleRuleResult<D> {
    pub fn new(child_rules: Vec<Rc<StyleRuleBase<D>>>) -> Self {
        Self {
            group_: StyleRuleGroup::new(RuleType::kResult, child_rules),
        }
    }
    pub fn CopyWithRules(_other: &Self, child_rules: Vec<Rc<StyleRuleBase<D>>>) -> Self {
        Self::new(child_rules)
    }
}
// cpp: style_rule.h:725-742; style_rule.cc:1215-1228
pub struct StyleRulePrivate<D: StyleRuleDependencies> {
    private_variables_: Vec<Rc<D::CSSPrivateVariable>>,
}
impl<D: StyleRuleDependencies> StyleRulePrivate<D> {
    pub fn new(private_variables: Vec<Rc<D::CSSPrivateVariable>>) -> Self {
        Self {
            private_variables_: private_variables,
        }
    }
    pub fn Copy(other: &Self) -> Self {
        Self::new(
            other
                .private_variables_
                .iter()
                .map(|variable| D::CopyPrivateVariable(variable))
                .collect(),
        )
    }
    pub fn GetPrivateVariables(&self) -> &Vec<Rc<D::CSSPrivateVariable>> {
        &self.private_variables_
    }
}
// cpp: style_rule.h:744-782; style_rule.cc:1235-1241
pub struct StyleRuleApplyMixin<D: StyleRuleDependencies> {
    group_: StyleRuleGroup<D>,
    name_: AtomicString,
    arguments_: Vec<Option<Rc<D::CSSVariableData>>>,
    has_contents_block_: bool,
}
impl<D: StyleRuleDependencies> StyleRuleApplyMixin<D> {
    pub fn WithContents(
        name: AtomicString,
        arguments: Vec<Option<Rc<D::CSSVariableData>>>,
        child_rules: Vec<Rc<StyleRuleBase<D>>>,
    ) -> Self {
        Self {
            group_: StyleRuleGroup::new(RuleType::kApplyMixin, child_rules),
            name_: name,
            arguments_: arguments,
            has_contents_block_: true,
        }
    }
    pub fn WithoutContents(
        name: AtomicString,
        arguments: Vec<Option<Rc<D::CSSVariableData>>>,
    ) -> Self {
        Self {
            group_: StyleRuleGroup::new(RuleType::kApplyMixin, Vec::new()),
            name_: name,
            arguments_: arguments,
            has_contents_block_: false,
        }
    }
    pub fn CopyWithRules(other: &Self, child_rules: Vec<Rc<StyleRuleBase<D>>>) -> Self {
        Self {
            group_: StyleRuleGroup::new(RuleType::kApplyMixin, child_rules),
            name_: other.name_.clone(),
            arguments_: other.arguments_.clone(),
            has_contents_block_: other.has_contents_block_,
        }
    }
    pub fn GetName(&self) -> &AtomicString {
        &self.name_
    }
    pub fn GetArguments(&self) -> &Vec<Option<Rc<D::CSSVariableData>>> {
        &self.arguments_
    }
    pub fn HasContentsBlock(&self) -> bool {
        self.has_contents_block_
    }
}
// cpp: style_rule.h:793-803; style_rule.cc:1248-1251
pub struct StyleRuleContentsStatement<D: StyleRuleDependencies> {
    group_: StyleRuleGroup<D>,
}
impl<D: StyleRuleDependencies> StyleRuleContentsStatement<D> {
    pub fn new(child_rules: Vec<Rc<StyleRuleBase<D>>>) -> Self {
        Self {
            group_: StyleRuleGroup::new(RuleType::kContents, child_rules),
        }
    }
    pub fn CopyWithRules(_other: &Self, child_rules: Vec<Rc<StyleRuleBase<D>>>) -> Self {
        Self::new(child_rules)
    }
}
// cpp: style_rule.h:806-833; style_rule.cc:1258-1269
pub struct StyleRuleCustomMedia<D: StyleRuleDependencies> {
    name_: AtomicString,
    media_query_value_: Option<Rc<D::MediaQuerySet>>,
    boolean_value_: bool,
}
impl<D: StyleRuleDependencies> StyleRuleCustomMedia<D> {
    pub fn FromMediaQuery(name: AtomicString, media_query_set: Rc<D::MediaQuerySet>) -> Self {
        Self {
            name_: name,
            media_query_value_: Some(media_query_set),
            boolean_value_: false,
        }
    }
    pub fn FromBoolean(name: AtomicString, value: bool) -> Self {
        Self {
            name_: name,
            media_query_value_: None,
            boolean_value_: value,
        }
    }
    pub fn GetName(&self) -> String {
        AtomicStringAsString(&self.name_)
    }
    pub fn IsMediaQueryValue(&self) -> bool {
        self.media_query_value_.is_some()
    }
    pub fn IsBooleanValue(&self) -> bool {
        !self.IsMediaQueryValue()
    }
    pub fn GetMediaQueryValue(&self) -> &D::MediaQuerySet {
        assert!(self.IsMediaQueryValue());
        self.media_query_value_.as_deref().unwrap()
    }
    pub fn GetBooleanValue(&self) -> bool {
        assert!(self.IsBooleanValue());
        self.boolean_value_
    }
    pub fn SetMediaQueries(&mut self, media_queries: Rc<D::MediaQuerySet>) {
        self.media_query_value_ = Some(media_queries);
    }
}

impl<D: StyleRuleDependencies> std::ops::Deref for StyleRuleScope<D> {
    type Target = StyleRuleGroup<D>;
    fn deref(&self) -> &Self::Target {
        &self.group_
    }
}
impl<D: StyleRuleDependencies> std::ops::DerefMut for StyleRuleScope<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.group_
    }
}
impl<D: StyleRuleDependencies> std::ops::Deref for StyleRuleLayerBlock<D> {
    type Target = StyleRuleGroup<D>;
    fn deref(&self) -> &Self::Target {
        &self.group_
    }
}
impl<D: StyleRuleDependencies> std::ops::DerefMut for StyleRuleLayerBlock<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.group_
    }
}
impl<D: StyleRuleDependencies> std::ops::Deref for StyleRulePage<D> {
    type Target = StyleRuleGroup<D>;
    fn deref(&self) -> &Self::Target {
        &self.group_
    }
}
impl<D: StyleRuleDependencies> std::ops::DerefMut for StyleRulePage<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.group_
    }
}
impl<D: StyleRuleDependencies> std::ops::Deref for StyleRuleStartingStyle<D> {
    type Target = StyleRuleGroup<D>;
    fn deref(&self) -> &Self::Target {
        &self.group_
    }
}
impl<D: StyleRuleDependencies> std::ops::DerefMut for StyleRuleStartingStyle<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.group_
    }
}
impl<D: StyleRuleDependencies> std::ops::Deref for StyleRuleFunction<D> {
    type Target = StyleRuleGroup<D>;
    fn deref(&self) -> &Self::Target {
        &self.group_
    }
}
impl<D: StyleRuleDependencies> std::ops::DerefMut for StyleRuleFunction<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.group_
    }
}
impl<D: StyleRuleDependencies> std::ops::Deref for StyleRuleMixin<D> {
    type Target = StyleRuleGroup<D>;
    fn deref(&self) -> &Self::Target {
        &self.group_
    }
}
impl<D: StyleRuleDependencies> std::ops::DerefMut for StyleRuleMixin<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.group_
    }
}
impl<D: StyleRuleDependencies> std::ops::Deref for StyleRuleResult<D> {
    type Target = StyleRuleGroup<D>;
    fn deref(&self) -> &Self::Target {
        &self.group_
    }
}
impl<D: StyleRuleDependencies> std::ops::DerefMut for StyleRuleResult<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.group_
    }
}
impl<D: StyleRuleDependencies> std::ops::Deref for StyleRuleApplyMixin<D> {
    type Target = StyleRuleGroup<D>;
    fn deref(&self) -> &Self::Target {
        &self.group_
    }
}
impl<D: StyleRuleDependencies> std::ops::DerefMut for StyleRuleApplyMixin<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.group_
    }
}
impl<D: StyleRuleDependencies> std::ops::Deref for StyleRuleContentsStatement<D> {
    type Target = StyleRuleGroup<D>;
    fn deref(&self) -> &Self::Target {
        &self.group_
    }
}
impl<D: StyleRuleDependencies> std::ops::DerefMut for StyleRuleContentsStatement<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.group_
    }
}
impl<D: StyleRuleDependencies> std::ops::Deref for StyleRuleMedia<D> {
    type Target = StyleRuleCondition<D>;
    fn deref(&self) -> &Self::Target {
        &self.condition_
    }
}
impl<D: StyleRuleDependencies> std::ops::DerefMut for StyleRuleMedia<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.condition_
    }
}
impl<D: StyleRuleDependencies> std::ops::Deref for StyleRuleSupports<D> {
    type Target = StyleRuleCondition<D>;
    fn deref(&self) -> &Self::Target {
        &self.condition_
    }
}
impl<D: StyleRuleDependencies> std::ops::DerefMut for StyleRuleSupports<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.condition_
    }
}
impl<D: StyleRuleDependencies> std::ops::Deref for StyleRuleContainer<D> {
    type Target = StyleRuleCondition<D>;
    fn deref(&self) -> &Self::Target {
        &self.condition_
    }
}
impl<D: StyleRuleDependencies> std::ops::DerefMut for StyleRuleContainer<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.condition_
    }
}
impl<D: StyleRuleDependencies> std::ops::Deref for StyleRuleNavigation<D> {
    type Target = StyleRuleCondition<D>;
    fn deref(&self) -> &Self::Target {
        &self.condition_
    }
}
impl<D: StyleRuleDependencies> std::ops::DerefMut for StyleRuleNavigation<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.condition_
    }
}
impl<D: StyleRuleDependencies> std::ops::Deref for StyleRuleCondition<D> {
    type Target = StyleRuleGroup<D>;
    fn deref(&self) -> &Self::Target {
        &self.group_
    }
}
impl<D: StyleRuleDependencies> std::ops::DerefMut for StyleRuleCondition<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.group_
    }
}

// Required concrete CSSOM constructors and parent/use-counter operations invoked
// by StyleRuleBase's real switch. Each method returns an actual CSSRule; there is
// no generic fallback constructor or empty wrapper. Implementations belong to
// the respective CSSOM files and their actual sheet/document dependencies.
pub trait StyleRuleCSSOMBackend<D: StyleRuleDependencies> {
    type CSSRule;
    type CSSStyleSheet;
    fn CountCSSPageRule(&mut self, sheet: &Self::CSSStyleSheet);
    fn SetParentRule(&mut self, rule: &mut Self::CSSRule, parent: &Self::CSSRule);
    fn NewCSSStyleRule(
        &mut self,
        rule: &StyleRule<D>,
        sheet: Option<&Self::CSSStyleSheet>,
        position_hint: usize,
    ) -> Self::CSSRule;
    fn NewCSSPageRule(
        &mut self,
        rule: &StyleRulePage<D>,
        sheet: Option<&Self::CSSStyleSheet>,
    ) -> Self::CSSRule;
    fn NewCSSPageMarginRule(
        &mut self,
        rule: &StyleRulePageMargin<D>,
        sheet: Option<&Self::CSSStyleSheet>,
    ) -> Self::CSSRule;
    fn NewCSSLocationRule(
        &mut self,
        rule: &D::StyleRuleLocation,
        sheet: Option<&Self::CSSStyleSheet>,
    ) -> Self::CSSRule;
    fn NewCSSNavigationRule(
        &mut self,
        rule: &StyleRuleNavigation<D>,
        sheet: Option<&Self::CSSStyleSheet>,
    ) -> Self::CSSRule;
    fn NewCSSPropertyRule(
        &mut self,
        rule: &StyleRuleProperty<D>,
        sheet: Option<&Self::CSSStyleSheet>,
    ) -> Self::CSSRule;
    fn NewCSSFontFaceRule(
        &mut self,
        rule: &StyleRuleFontFace<D>,
        sheet: Option<&Self::CSSStyleSheet>,
    ) -> Self::CSSRule;
    fn NewCSSFontPaletteValuesRule(
        &mut self,
        rule: &D::StyleRuleFontPaletteValues,
        sheet: Option<&Self::CSSStyleSheet>,
    ) -> Self::CSSRule;
    fn NewCSSFontFeatureValuesRule(
        &mut self,
        rule: &D::StyleRuleFontFeatureValues,
        sheet: Option<&Self::CSSStyleSheet>,
    ) -> Self::CSSRule;
    fn NewCSSMediaRule(
        &mut self,
        rule: &StyleRuleMedia<D>,
        sheet: Option<&Self::CSSStyleSheet>,
    ) -> Self::CSSRule;
    fn NewCSSNestedDeclarationsRule(
        &mut self,
        rule: &D::StyleRuleNestedDeclarations,
        sheet: Option<&Self::CSSStyleSheet>,
    ) -> Self::CSSRule;
    fn NewCSSFunctionDeclarationsRule(
        &mut self,
        rule: &D::StyleRuleFunctionDeclarations,
        sheet: Option<&Self::CSSStyleSheet>,
    ) -> Self::CSSRule;
    fn NewCSSFunctionRule(
        &mut self,
        rule: &StyleRuleFunction<D>,
        sheet: Option<&Self::CSSStyleSheet>,
    ) -> Self::CSSRule;
    fn NewCSSScopeRule(
        &mut self,
        rule: &StyleRuleScope<D>,
        sheet: Option<&Self::CSSStyleSheet>,
    ) -> Self::CSSRule;
    fn NewCSSSupportsRule(
        &mut self,
        rule: &StyleRuleSupports<D>,
        sheet: Option<&Self::CSSStyleSheet>,
    ) -> Self::CSSRule;
    fn NewCSSImportRule(
        &mut self,
        rule: &D::StyleRuleImport,
        sheet: Option<&Self::CSSStyleSheet>,
    ) -> Self::CSSRule;
    fn NewCSSKeyframesRule(
        &mut self,
        rule: &D::StyleRuleKeyframes,
        sheet: Option<&Self::CSSStyleSheet>,
    ) -> Self::CSSRule;
    fn NewCSSLayerBlockRule(
        &mut self,
        rule: &StyleRuleLayerBlock<D>,
        sheet: Option<&Self::CSSStyleSheet>,
    ) -> Self::CSSRule;
    fn NewCSSLayerStatementRule(
        &mut self,
        rule: &StyleRuleLayerStatement,
        sheet: Option<&Self::CSSStyleSheet>,
    ) -> Self::CSSRule;
    fn NewCSSNamespaceRule(
        &mut self,
        rule: &D::StyleRuleNamespace,
        sheet: Option<&Self::CSSStyleSheet>,
    ) -> Self::CSSRule;
    fn NewCSSContainerRule(
        &mut self,
        rule: &StyleRuleContainer<D>,
        sheet: Option<&Self::CSSStyleSheet>,
    ) -> Self::CSSRule;
    fn NewCSSCounterStyleRule(
        &mut self,
        rule: &D::StyleRuleCounterStyle,
        sheet: Option<&Self::CSSStyleSheet>,
    ) -> Self::CSSRule;
    fn NewCSSStartingStyleRule(
        &mut self,
        rule: &StyleRuleStartingStyle<D>,
        sheet: Option<&Self::CSSStyleSheet>,
    ) -> Self::CSSRule;
    fn NewCSSViewTransitionRule(
        &mut self,
        rule: &D::StyleRuleViewTransition,
        sheet: Option<&Self::CSSStyleSheet>,
    ) -> Self::CSSRule;
    fn NewCSSPositionTryRule(
        &mut self,
        rule: &D::StyleRulePositionTry,
        sheet: Option<&Self::CSSStyleSheet>,
    ) -> Self::CSSRule;
    fn NewCSSCustomMediaRule(
        &mut self,
        rule: &StyleRuleCustomMedia<D>,
        sheet: Option<&Self::CSSStyleSheet>,
    ) -> Self::CSSRule;
    fn NewCSSMixinRule(
        &mut self,
        rule: &StyleRuleMixin<D>,
        sheet: Option<&Self::CSSStyleSheet>,
    ) -> Self::CSSRule;
    fn NewCSSResultRule(
        &mut self,
        rule: &StyleRuleResult<D>,
        sheet: Option<&Self::CSSStyleSheet>,
    ) -> Self::CSSRule;
    fn NewCSSPrivateRule(
        &mut self,
        rule: &StyleRulePrivate<D>,
        sheet: Option<&Self::CSSStyleSheet>,
    ) -> Self::CSSRule;
    fn NewCSSApplyMixinRule(
        &mut self,
        rule: &StyleRuleApplyMixin<D>,
        sheet: Option<&Self::CSSStyleSheet>,
    ) -> Self::CSSRule;
    fn NewCSSContentsRule(
        &mut self,
        rule: &StyleRuleContentsStatement<D>,
        sheet: Option<&Self::CSSStyleSheet>,
    ) -> Self::CSSRule;
}
impl<D: StyleRuleDependencies> StyleRuleBase<D> {
    // cpp: style_rule.cc:101-113; separate Rust names for source overloads.
    pub fn CreateCSSOMWrapper<B: StyleRuleCSSOMBackend<D>>(
        &self,
        backend: &mut B,
        position_hint: usize,
        parent_sheet: Option<&B::CSSStyleSheet>,
        trigger_use_counters: bool,
    ) -> B::CSSRule {
        self.CreateCSSOMWrapperInternal(
            backend,
            position_hint,
            parent_sheet,
            None,
            trigger_use_counters,
        )
    }
    pub fn CreateCSSOMWrapperForRule<B: StyleRuleCSSOMBackend<D>>(
        &self,
        backend: &mut B,
        position_hint: usize,
        parent_rule: Option<&B::CSSRule>,
        trigger_use_counters: bool,
    ) -> B::CSSRule {
        self.CreateCSSOMWrapperInternal(
            backend,
            position_hint,
            None,
            parent_rule,
            trigger_use_counters,
        )
    }
    // cpp: style_rule.cc:331-475
    fn CreateCSSOMWrapperInternal<B: StyleRuleCSSOMBackend<D>>(
        &self,
        backend: &mut B,
        position_hint: usize,
        parent_sheet: Option<&B::CSSStyleSheet>,
        parent_rule: Option<&B::CSSRule>,
        trigger_use_counters: bool,
    ) -> B::CSSRule {
        let mut rule = match self {
            Self::Style(payload) => backend.NewCSSStyleRule(payload, parent_sheet, position_hint),
            Self::Page(payload) => {
                if trigger_use_counters {
                    if let Some(sheet) = parent_sheet {
                        backend.CountCSSPageRule(sheet);
                    }
                }
                backend.NewCSSPageRule(payload, parent_sheet)
            }
            Self::PageMargin(payload) => backend.NewCSSPageMarginRule(payload, parent_sheet),
            Self::Location(payload) => backend.NewCSSLocationRule(payload, parent_sheet),
            Self::Navigation(payload) => backend.NewCSSNavigationRule(payload, parent_sheet),
            Self::Property(payload) => backend.NewCSSPropertyRule(payload, parent_sheet),
            Self::FontFace(payload) => backend.NewCSSFontFaceRule(payload, parent_sheet),
            Self::FontPaletteValues(payload) => {
                backend.NewCSSFontPaletteValuesRule(payload, parent_sheet)
            }
            Self::FontFeatureValues(payload) => {
                backend.NewCSSFontFeatureValuesRule(payload, parent_sheet)
            }
            Self::Media(payload) => backend.NewCSSMediaRule(payload, parent_sheet),
            Self::NestedDeclarations(payload) => {
                backend.NewCSSNestedDeclarationsRule(payload, parent_sheet)
            }
            Self::FunctionDeclarations(payload) => {
                backend.NewCSSFunctionDeclarationsRule(payload, parent_sheet)
            }
            Self::Function(payload) => backend.NewCSSFunctionRule(payload, parent_sheet),
            Self::Scope(payload) => backend.NewCSSScopeRule(payload, parent_sheet),
            Self::Supports(payload) => backend.NewCSSSupportsRule(payload, parent_sheet),
            Self::Import(payload) => backend.NewCSSImportRule(payload, parent_sheet),
            Self::Keyframes(payload) => backend.NewCSSKeyframesRule(payload, parent_sheet),
            Self::LayerBlock(payload) => backend.NewCSSLayerBlockRule(payload, parent_sheet),
            Self::LayerStatement(payload) => {
                backend.NewCSSLayerStatementRule(payload, parent_sheet)
            }
            Self::Namespace(payload) => backend.NewCSSNamespaceRule(payload, parent_sheet),
            Self::Container(payload) => backend.NewCSSContainerRule(payload, parent_sheet),
            Self::CounterStyle(payload) => backend.NewCSSCounterStyleRule(payload, parent_sheet),
            Self::StartingStyle(payload) => backend.NewCSSStartingStyleRule(payload, parent_sheet),
            Self::ViewTransition(payload) => {
                backend.NewCSSViewTransitionRule(payload, parent_sheet)
            }
            Self::PositionTry(payload) => backend.NewCSSPositionTryRule(payload, parent_sheet),
            Self::CustomMedia(payload) => backend.NewCSSCustomMediaRule(payload, parent_sheet),
            Self::Mixin(payload) => backend.NewCSSMixinRule(payload, parent_sheet),
            Self::Result(payload) => backend.NewCSSResultRule(payload, parent_sheet),
            Self::Private(payload) => backend.NewCSSPrivateRule(payload, parent_sheet),
            Self::ApplyMixin(payload) => backend.NewCSSApplyMixinRule(payload, parent_sheet),
            Self::Contents(payload) => backend.NewCSSContentsRule(payload, parent_sheet),
            Self::FontFeature(_) | Self::Keyframe(_) | Self::Charset(_) => {
                panic!("NOTREACHED: rule type has no CSSOM wrapper")
            }
        };
        if let Some(parent) = parent_rule {
            backend.SetParentRule(&mut rule, parent);
        }
        rule
    }
}

fn AtomicStringAsString(value: &AtomicString) -> String {
    if value.IsNull() {
        return String::default();
    }
    String::from_utf16(&(0..value.length()).map(|i| value.at(i)).collect::<Vec<_>>())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    // Fixtures exercise rule ownership/control only; they do not parse CSS or
    // replace production CSSSelector/CSSValue implementations. Unused parser
    // and data dependencies are uninhabited and cannot be constructed.
    #[derive(Clone)]
    enum UnusedDependency {}
    struct SelectorFixture([u32; 3], Option<std::rc::Weak<StyleRule<Dependencies>>>);
    impl StyleRuleSelectorList for SelectorFixture {
        type CSSSelector = u32;
        fn FirstSelector(&self) -> &u32 {
            &self.0[0]
        }
        fn SelectorAt(&self, index: usize) -> &u32 {
            &self.0[index]
        }
        fn MutableSelectorAt(&mut self, index: usize) -> &mut u32 {
            &mut self.0[index]
        }
        fn SelectorIndex(&self, selector: &u32) -> usize {
            self.0
                .iter()
                .position(|slot| std::ptr::eq(slot, selector))
                .unwrap()
        }
        fn NextSelector(&self, selector: &u32) -> Option<&u32> {
            self.0.get(self.SelectorIndex(selector) + 1)
        }
        fn SelectorsText(&self) -> String {
            panic!("unmapped fixture serialization must not run")
        }
    }
    struct PropertyFixture {
        mutable: bool,
        failed: bool,
        copy_count: Rc<Cell<u32>>,
    }
    impl StyleRulePropertySet for PropertyFixture {
        type CSSValue = UnusedDependency;
        fn IsMutable(&self) -> bool {
            self.mutable
        }
        fn MutableCopy(&self) -> Rc<Self> {
            self.copy_count.set(self.copy_count.get() + 1);
            Rc::new(Self {
                mutable: true,
                failed: self.failed,
                copy_count: Rc::clone(&self.copy_count),
            })
        }
        fn HasFailedOrCanceledSubresources(&self) -> bool {
            self.failed
        }
        fn GetPropertyCSSValue(&self, _id: CSSPropertyID) -> Option<Rc<Self::CSSValue>> {
            panic!("fixture CSSValue lookup must not run")
        }
    }
    struct LazyFixture {
        calls: Rc<Cell<u32>>,
        properties: Rc<PropertyFixture>,
    }
    struct BindingFixture(u8);
    struct PrivateFixture(u8);
    struct MediaFixture(u8);
    struct NestedFixture {
        nesting: CSSNestingType,
        inner: Rc<StyleRule<Dependencies>>,
    }
    struct ScopeFixture {
        rule: Option<Rc<StyleRule<Dependencies>>>,
    }
    struct Dependencies;
    impl StyleRuleDependencies for Dependencies {
        type SelectorList = SelectorFixture;
        type CSSPropertyValueSet = PropertyFixture;
        type CSSLazyParsingState = LazyFixture;
        type MixinParameterBindings = BindingFixture;
        type StyleScope = ScopeFixture;
        type MediaQuerySet = MediaFixture;
        type ContainerQuerySet = UnusedDependency;
        type ContainerQuery = UnusedDependency;
        type ConditionalExpNode = UnusedDependency;
        type NavigationQuery = UnusedDependency;
        type CSSSyntaxDefinition = u8;
        type CSSVariableData = UnusedDependency;
        type CSSPrivateVariable = PrivateFixture;
        type ExecutionContext = UnusedDependency;
        type StyleRuleImport = UnusedDependency;
        type StyleRuleFontPaletteValues = UnusedDependency;
        type StyleRuleFontFeatureValues = UnusedDependency;
        type StyleRuleFontFeature = UnusedDependency;
        type StyleRuleKeyframes = UnusedDependency;
        type StyleRuleKeyframe = UnusedDependency;
        type StyleRuleNestedDeclarations = NestedFixture;
        type StyleRuleFunctionDeclarations = UnusedDependency;
        type StyleRuleNamespace = UnusedDependency;
        type StyleRuleCounterStyle = UnusedDependency;
        type StyleRuleViewTransition = UnusedDependency;
        type StyleRulePositionTry = UnusedDependency;
        type StyleRuleLocation = UnusedDependency;
        fn ParseDeclarationListForLazyStyle(
            state: &LazyFixture,
            offset: usize,
        ) -> Rc<PropertyFixture> {
            assert_eq!(offset, 9);
            state.calls.set(state.calls.get() + 1);
            Rc::clone(&state.properties)
        }
        fn ParseCustomPropertyName(_text: &String) -> String {
            panic!("unused name parser")
        }
        fn ConsumeSupportsCondition(_context: &UnusedDependency, _text: &String) -> bool {
            panic!("unused supports parser")
        }
        fn ContainerQuerySetToString(_set: &UnusedDependency) -> String {
            panic!("unused query set")
        }
        fn ParseContainerQuerySet(
            _context: &UnusedDependency,
            _text: &String,
        ) -> Option<Rc<UnusedDependency>> {
            panic!("unused query parser")
        }
        fn SingleContainerQuery(_set: &UnusedDependency) -> Option<&UnusedDependency> {
            panic!("unused single query")
        }
        fn ParseContainerCondition(
            _context: &UnusedDependency,
            _text: &String,
        ) -> Option<Rc<UnusedDependency>> {
            panic!("unused condition parser")
        }
        fn SerializeContainerCondition(_node: &UnusedDependency) -> String {
            panic!("unused condition serializer")
        }
        fn ContainerQuerySelectorName(_query: &UnusedDependency) -> AtomicString {
            panic!("unused query selector")
        }
        fn NewContainerQuery(
            _name: AtomicString,
            _node: Rc<UnusedDependency>,
        ) -> Rc<UnusedDependency> {
            panic!("unused query constructor")
        }
        fn NewContainerQuerySet(_queries: Vec<Rc<UnusedDependency>>) -> Rc<UnusedDependency> {
            panic!("unused query set constructor")
        }
        fn ParseNavigationQuery(_text: &String) -> Option<Rc<UnusedDependency>> {
            panic!("unused navigation parser")
        }
        fn CopyPrivateVariable(variable: &PrivateFixture) -> Rc<PrivateFixture> {
            Rc::new(PrivateFixture(variable.0))
        }
    }
    impl StyleRulePropertySetClone for PropertyFixture {
        fn ImmutableCopyIfNeeded(this: &Rc<Self>) -> Rc<Self> {
            if !this.mutable {
                return Rc::clone(this);
            }
            this.copy_count.set(this.copy_count.get() + 1);
            Rc::new(Self {
                mutable: false,
                failed: this.failed,
                copy_count: this.copy_count.clone(),
            })
        }
        fn AverageSizeInBytes() -> usize {
            99
        }
    }
    #[allow(unused_variables)]
    impl StyleRuleCloneDependencies for Dependencies {
        type KeyframeOffset = f64;
        fn RenestStyleSelectors(
            list: &Self::SelectorList,
            parent: Option<Rc<StyleRule<Self>>>,
        ) -> Self::SelectorList {
            SelectorFixture(list.0, parent.as_ref().map(Rc::downgrade))
        }
        fn RenestPageSelectors(
            list: &Rc<Self::SelectorList>,
            parent: Option<Rc<StyleRule<Self>>>,
        ) -> Rc<Self::SelectorList> {
            Rc::new(Self::RenestStyleSelectors(list, parent))
        }
        fn CopySelectors(list: &Self::SelectorList) -> Self::SelectorList {
            SelectorFixture(list.0, list.1.clone())
        }
        fn DummyNestingSelectors() -> Self::SelectorList {
            SelectorFixture([0, 0, 0], None)
        }
        fn CloneStyleScope(
            scope: &Self::StyleScope,
            parent: Option<Rc<StyleRule<Self>>>,
        ) -> Rc<Self::StyleScope> {
            Rc::new(ScopeFixture { rule: parent })
        }
        fn ScopeRuleForNesting(scope: &Self::StyleScope) -> Option<Rc<StyleRule<Self>>> {
            scope.rule.clone()
        }
        fn CopyContainerQuerySet(set: &Self::ContainerQuerySet) -> Rc<Self::ContainerQuerySet> {
            panic!("uninhabited fixture dependency")
        }
        fn NestedDeclarationsNestingType(
            rule: &Self::StyleRuleNestedDeclarations,
        ) -> CSSNestingType {
            rule.nesting
        }
        fn NestedDeclarationsInnerRule(
            rule: &Self::StyleRuleNestedDeclarations,
        ) -> &StyleRule<Self> {
            &rule.inner
        }
        fn NewNestedDeclarations(
            nesting: CSSNestingType,
            inner: Rc<StyleRule<Self>>,
        ) -> Rc<Self::StyleRuleNestedDeclarations> {
            Rc::new(NestedFixture { nesting, inner })
        }
        fn Keyframes(rule: &Self::StyleRuleKeyframes) -> &[Rc<Self::StyleRuleKeyframe>] {
            panic!("uninhabited fixture dependency")
        }
        fn KeyframesName(rule: &Self::StyleRuleKeyframes) -> AtomicString {
            panic!("uninhabited fixture dependency")
        }
        fn KeyframesVersion(rule: &Self::StyleRuleKeyframes) -> u32 {
            panic!("uninhabited fixture dependency")
        }
        fn KeyframesIsVendorPrefixed(rule: &Self::StyleRuleKeyframes) -> bool {
            panic!("uninhabited fixture dependency")
        }
        fn NewKeyframes(
            keys: Vec<Rc<Self::StyleRuleKeyframe>>,
            name: AtomicString,
            version: u32,
            prefixed: bool,
        ) -> Rc<Self::StyleRuleKeyframes> {
            panic!("uninhabited fixture dependency")
        }
        fn KeyframeKeys(rule: &Self::StyleRuleKeyframe) -> &[Self::KeyframeOffset] {
            panic!("uninhabited fixture dependency")
        }
        fn KeyframeProperties(rule: &Self::StyleRuleKeyframe) -> Rc<Self::CSSPropertyValueSet> {
            panic!("uninhabited fixture dependency")
        }
        fn NewKeyframe(
            keys: Vec<Self::KeyframeOffset>,
            properties: Rc<Self::CSSPropertyValueSet>,
        ) -> Rc<Self::StyleRuleKeyframe> {
            panic!("uninhabited fixture dependency")
        }
        fn CounterStyleName(rule: &Self::StyleRuleCounterStyle) -> AtomicString {
            panic!("uninhabited fixture dependency")
        }
        fn CounterStyleProperties(
            rule: &Self::StyleRuleCounterStyle,
        ) -> Rc<Self::CSSPropertyValueSet> {
            panic!("uninhabited fixture dependency")
        }
        fn NewCounterStyle(
            name: AtomicString,
            properties: Rc<Self::CSSPropertyValueSet>,
        ) -> Rc<Self::StyleRuleCounterStyle> {
            panic!("uninhabited fixture dependency")
        }
        fn PositionTryName(rule: &Self::StyleRulePositionTry) -> AtomicString {
            panic!("uninhabited fixture dependency")
        }
        fn PositionTryProperties(
            rule: &Self::StyleRulePositionTry,
        ) -> Rc<Self::CSSPropertyValueSet> {
            panic!("uninhabited fixture dependency")
        }
        fn NewPositionTry(
            name: AtomicString,
            properties: Rc<Self::CSSPropertyValueSet>,
        ) -> Rc<Self::StyleRulePositionTry> {
            panic!("uninhabited fixture dependency")
        }
        fn CopyImport(rule: &Self::StyleRuleImport) -> Rc<Self::StyleRuleImport> {
            panic!("uninhabited fixture dependency")
        }
        fn CopyFontPaletteValues(
            rule: &Self::StyleRuleFontPaletteValues,
        ) -> Rc<Self::StyleRuleFontPaletteValues> {
            panic!("uninhabited fixture dependency")
        }
        fn CopyFunctionDeclarations(
            rule: &Self::StyleRuleFunctionDeclarations,
        ) -> Rc<Self::StyleRuleFunctionDeclarations> {
            panic!("uninhabited fixture dependency")
        }
        fn CopyNamespace(rule: &Self::StyleRuleNamespace) -> Rc<Self::StyleRuleNamespace> {
            panic!("uninhabited fixture dependency")
        }
        fn CopyViewTransition(
            rule: &Self::StyleRuleViewTransition,
        ) -> Rc<Self::StyleRuleViewTransition> {
            panic!("uninhabited fixture dependency")
        }
        fn CopyLocation(rule: &Self::StyleRuleLocation) -> Rc<Self::StyleRuleLocation> {
            panic!("uninhabited fixture dependency")
        }
    }
    fn property(mutable: bool, failed: bool) -> Rc<PropertyFixture> {
        Rc::new(PropertyFixture {
            mutable,
            failed,
            copy_count: Rc::new(Cell::new(0)),
        })
    }
    fn layer(name: &str) -> Rc<StyleRuleBase<Dependencies>> {
        Rc::new(StyleRuleBase::LayerStatement(StyleRuleLayerStatement::new(
            vec![vec![AtomicString::from_str(name)]],
        )))
    }
    #[test]
    fn lazy_parse_and_mutable_copy_preserve_original_property_identity() {
        let properties = property(false, true);
        let calls = Rc::new(Cell::new(0));
        let lazy = Rc::new(LazyFixture {
            calls: Rc::clone(&calls),
            properties: Rc::clone(&properties),
        });
        let weak_lazy = Rc::downgrade(&lazy);
        let mut rule =
            StyleRule::<Dependencies>::CreateLazy(SelectorFixture([4, 5, 6], None), lazy, 9);
        assert!(!rule.HasParsedProperties());
        assert!(!rule.PropertiesHaveFailedOrCanceledSubresources());
        assert_eq!(calls.get(), 0);
        assert!(Rc::ptr_eq(&rule.Properties(), &properties));
        assert!(weak_lazy.upgrade().is_none());
        assert!(rule.HasParsedProperties());
        assert!(rule.PropertiesHaveFailedOrCanceledSubresources());
        assert!(Rc::ptr_eq(&rule.Properties(), &properties));
        assert_eq!(calls.get(), 1);
        let mutable = rule.MutableProperties();
        assert!(!Rc::ptr_eq(&mutable, &properties));
        assert!(mutable.IsMutable());
        assert_eq!(properties.copy_count.get(), 1);
        assert!(Rc::ptr_eq(&mutable, &rule.MutableProperties()));
        assert_eq!(properties.copy_count.get(), 1);
        assert_eq!(rule.IndexOfNextSelectorAfter(0), 1);
        assert_eq!(rule.IndexOfNextSelectorAfter(2), usize::MAX);
        *rule.MutableSelectorAt(1) = 90;
        assert_eq!(*rule.SelectorAt(1), 90);
    }
    #[test]
    fn clone_renests_each_child_under_its_new_style_parent_and_retains_typed_identity() {
        let properties = property(true, false);
        let root = Rc::new(StyleRule::<Dependencies>::Create(
            SelectorFixture([1, 2, 3], None),
            properties.clone(),
            None,
        ));
        let child = Rc::new(StyleRule::<Dependencies>::Create(
            SelectorFixture([4, 5, 6], None),
            property(false, false),
            None,
        ));
        child.AddChildRule(layer("leaf"));
        root.AddChildRule(Rc::new(StyleRuleBase::Media(StyleRuleMedia::new(
            None,
            vec![Rc::new(StyleRuleBase::Style(child.clone()))],
        ))));
        let binding = Rc::new(BindingFixture(42));
        let outer = Rc::new(StyleRule::<Dependencies>::Create(
            SelectorFixture([7, 8, 9], None),
            property(false, false),
            None,
        ));
        let cloned_base =
            StyleRuleBase::Style(root.clone()).Clone(Some(outer.clone()), Some(binding.clone()));
        let cloned = cloned_base.StyleRef().unwrap();
        assert!(std::ptr::eq(
            cloned_base.AsStyle().unwrap(),
            cloned.as_ref()
        ));
        assert!(!Rc::ptr_eq(&cloned, &root));
        assert!(Rc::ptr_eq(
            &cloned.selectors_.1.as_ref().unwrap().upgrade().unwrap(),
            &outer
        ));
        assert!(!Rc::ptr_eq(&cloned.Properties(), &properties));
        assert!(!cloned.Properties().IsMutable());
        assert!(Rc::ptr_eq(
            cloned.GetMixinParameterBindings().unwrap(),
            &binding
        ));
        let children = cloned.ChildRules().unwrap();
        let cloned_child = children[0].AsGroup().unwrap().ChildRules()[0]
            .StyleRef()
            .unwrap();
        assert!(!Rc::ptr_eq(&cloned_child, &child));
        assert!(Rc::ptr_eq(
            &cloned_child
                .selectors_
                .1
                .as_ref()
                .unwrap()
                .upgrade()
                .unwrap(),
            &cloned
        ));
        assert!(Rc::ptr_eq(&cloned_child.Properties(), &child.Properties()));
        assert!(!Rc::ptr_eq(
            &cloned_child.ChildRules().unwrap()[0],
            &child.ChildRules().unwrap()[0]
        ));
        assert!(root.selectors_.1.is_none());
        assert_eq!(
            StyleRule::<Dependencies>::AverageSizeInBytes(),
            std::mem::size_of::<StyleRule<Dependencies>>() + 4 + 99
        );
    }
    #[test]
    fn clone_nested_declarations_copies_parent_scope_or_dummy_selectors() {
        let props = property(false, false);
        let old_inner = Rc::new(StyleRule::<Dependencies>::Create(
            SelectorFixture([10, 20, 30], None),
            props.clone(),
            None,
        ));
        let parent = Rc::new(StyleRule::<Dependencies>::Create(
            SelectorFixture([40, 50, 60], None),
            property(false, false),
            None,
        ));
        for (nesting, provided_parent, expected) in [
            (CSSNestingType::kScope, Some(parent.clone()), [10, 20, 30]),
            (CSSNestingType::kNesting, Some(parent.clone()), [40, 50, 60]),
            (CSSNestingType::kNesting, None, [0, 0, 0]),
        ] {
            let base = StyleRuleBase::NestedDeclarations(Rc::new(NestedFixture {
                nesting,
                inner: old_inner.clone(),
            }));
            let cloned = base.Clone(provided_parent, None);
            let StyleRuleBase::NestedDeclarations(nested) = cloned.as_ref() else {
                unreachable!()
            };
            assert_eq!(nested.inner.selectors_.0, expected);
            assert!(!Rc::ptr_eq(&nested.inner, &old_inner));
            assert!(Rc::ptr_eq(&nested.inner.Properties(), &props));
        }
        let scoped_child = Rc::new(StyleRuleBase::Style(old_inner));
        let scope = StyleRuleBase::Scope(StyleRuleScope::new(
            Rc::new(ScopeFixture { rule: None }),
            vec![scoped_child],
        ));
        let cloned = scope.Clone(Some(parent.clone()), None);
        assert!(Rc::ptr_eq(
            &cloned.AsGroup().unwrap().ChildRules()[0]
                .StyleRef()
                .unwrap()
                .selectors_
                .1
                .as_ref()
                .unwrap()
                .upgrade()
                .unwrap(),
            &parent
        ));
    }
    struct MixinDependencies;
    impl MixinParameterBindingsDependencies for MixinDependencies {
        type CSSVariableData = u32;
        type CSSSyntaxDefinition = u8;
        type ContainerQuerySet = u8;
        fn VariableHash(value: &u32) -> u32 {
            *value
        }
        fn StringHash(key: &String) -> u32 {
            key.length() as u32
        }
    }
    fn mixin_binding(
        value: Option<u32>,
        default: Option<u32>,
        syntax: u8,
    ) -> MixinBinding<MixinDependencies> {
        MixinBinding {
            value: value.map(Rc::new),
            default_value: default.map(Rc::new),
            syntax,
        }
    }
    #[test]
    fn mixin_hash_and_equality_follow_member_value_and_parent_semantics() {
        type Bindings = MixinParameterBindings<MixinDependencies>;
        let key = String::from("--a");
        let make = |binding, local: Rc<u32>, conditional: HashMap<_, _>, parent| {
            Bindings::new(
                HashMap::from([(key.clone(), binding)]),
                HashMap::from([(key.clone(), local)]),
                conditional,
                parent,
            )
        };
        let local = Rc::new(77);
        let a = make(
            mixin_binding(None, Some(8), 2),
            local.clone(),
            HashMap::new(),
            None,
        );
        let expected = foundation::HashInts(
            foundation::HashInts(1234, foundation::HashInts(3, 5678)),
            foundation::HashInts(3 ^ 4321, 77),
        );
        assert_eq!(a.GetHash(), expected);
        let conditional = HashMap::from([(
            key.clone(),
            vec![MixinCQDependentValue {
                data: Rc::new(7),
                container_queries: Rc::new(1),
            }],
        )]);
        let b = make(
            mixin_binding(None, Some(8), 2),
            local.clone(),
            conditional,
            None,
        );
        assert!(a == b);
        assert_eq!(a.GetHash(), b.GetHash());
        let different_default = make(
            mixin_binding(None, Some(9), 2),
            local.clone(),
            HashMap::new(),
            None,
        );
        assert!(a != different_default);
        assert_eq!(a.GetHash(), different_default.GetHash());
        let different_syntax = make(
            mixin_binding(None, Some(8), 3),
            local.clone(),
            HashMap::new(),
            None,
        );
        assert!(a != different_syntax);
        let equivalent_value_different_pointer = make(
            mixin_binding(None, Some(8), 2),
            Rc::new(77),
            HashMap::new(),
            None,
        );
        assert!(a != equivalent_value_different_pointer);
        let pa = Rc::new(a);
        let pb = Rc::new(b);
        let empty =
            |parent| Bindings::new(HashMap::new(), HashMap::new(), HashMap::new(), Some(parent));
        let ca = empty(pa);
        let cb = empty(pb);
        assert!(ca == cb);
        assert_eq!(ca.GetHash(), expected);
    }
    struct Observer {
        seen: Vec<Rc<StyleRuleBase<Dependencies>>>,
    }
    impl StyleRuleChangeObserver<Rc<StyleRuleBase<Dependencies>>> for Observer {
        fn NotifyRuleChanged(&mut self, rule: &Rc<StyleRuleBase<Dependencies>>) {
            self.seen.push(Rc::clone(rule));
        }
    }
    #[test]
    fn child_mutation_uses_identity_hint_and_notifies_inserted_or_removed_rule() {
        let a = layer("same");
        let b = layer("same");
        let c = layer("next");
        let mut rule =
            StyleRule::<Dependencies>::CreateWithoutProperties(SelectorFixture([0, 1, 2], None));
        assert!(rule.ChildRules().is_none());
        assert_eq!(
            rule.ReplaceChildRuleIfExists(&a, Rc::clone(&b), 0),
            usize::MAX
        );
        rule.SetProperties(property(true, false));
        assert!(rule.HasParsedProperties());
        let mut observer = Observer { seen: Vec::new() };
        rule.WrapperInsertRule(Some(&mut observer), 0, Rc::clone(&a));
        rule.WrapperInsertRule(None, 1, Rc::clone(&b));
        assert!(Rc::ptr_eq(&observer.seen[0], &a));
        assert_eq!(rule.ReplaceChildRuleIfExists(&b, Rc::clone(&c), 0), 1);
        assert_eq!(rule.ReplaceChildRuleIfExists(&a, Rc::clone(&c), 0), 0);
        rule.WrapperRemoveRule(Some(&mut observer), 0);
        assert!(Rc::ptr_eq(&observer.seen[1], &c));
        assert_eq!(rule.ChildRules().unwrap().len(), 1);
        let mut duplicate = vec![Rc::clone(&a), Rc::clone(&a)];
        assert_eq!(
            ReplaceStyleRuleInVector(&a, Rc::clone(&b), 1, &mut duplicate),
            1
        );
        assert!(Rc::ptr_eq(&duplicate[0], &a));
        let missing = layer("same");
        assert_eq!(
            ReplaceStyleRuleInVector(&missing, c, 0, &mut duplicate),
            usize::MAX
        );
    }
    #[test]
    fn moved_style_drops_bindings_but_retains_lazy_state_and_children() {
        let properties = property(true, false);
        let binding = Rc::new(BindingFixture(8));
        let original = StyleRule::<Dependencies>::Create(
            SelectorFixture([1, 2, 3], None),
            Rc::clone(&properties),
            Some(binding),
        );
        assert_eq!(original.GetMixinParameterBindings().unwrap().0, 8);
        let child = layer("base");
        original.AddChildRule(Rc::clone(&child));
        let moved = StyleRule::CreateWithSelectors(SelectorFixture([8, 9, 10], None), original);
        assert!(moved.GetMixinParameterBindings().is_none());
        assert!(Rc::ptr_eq(&moved.Properties(), &properties));
        assert!(Rc::ptr_eq(&moved.ChildRules().unwrap()[0], &child));
        assert_eq!(*moved.FirstSelector(), 8);
    }
    #[test]
    fn inherited_classification_and_mixin_contents_remain_distinct() {
        let starting =
            StyleRuleBase::<Dependencies>::StartingStyle(StyleRuleStartingStyle::new(Vec::new()));
        assert!(starting.IsConditionRule());
        assert!(starting.AsCondition().is_none());
        assert!(starting.AsGroup().is_some());
        let media = StyleRuleBase::<Dependencies>::Media(StyleRuleMedia::new(None, Vec::new()));
        assert!(media.IsConditionRule());
        assert!(media.AsCondition().is_some());
        let empty = StyleRuleApplyMixin::<Dependencies>::WithContents(
            AtomicString::from_str("--m"),
            Vec::new(),
            Vec::new(),
        );
        let missing = StyleRuleApplyMixin::<Dependencies>::WithoutContents(
            AtomicString::from_str("--m"),
            Vec::new(),
        );
        assert!(empty.HasContentsBlock());
        assert!(!missing.HasContentsBlock());
        assert!(StyleRuleApplyMixin::CopyWithRules(&empty, Vec::new()).HasContentsBlock());
        assert!(!StyleRuleApplyMixin::CopyWithRules(&missing, Vec::new()).HasContentsBlock());
        let originals = StyleRulePrivate::<Dependencies>::new(vec![Rc::new(PrivateFixture(19))]);
        let copies = StyleRulePrivate::Copy(&originals);
        assert_eq!(copies.GetPrivateVariables()[0].0, 19);
        assert!(!Rc::ptr_eq(
            &originals.GetPrivateVariables()[0],
            &copies.GetPrivateVariables()[0]
        ));
        let mut custom =
            StyleRuleCustomMedia::<Dependencies>::FromBoolean(AtomicString::from_str("--x"), true);
        assert!(custom.IsBooleanValue() && custom.GetBooleanValue());
        custom.SetMediaQueries(Rc::new(MediaFixture(23)));
        assert!(custom.IsMediaQueryValue());
        assert_eq!(custom.GetMediaQueryValue().0, 23);
        assert!(std::panic::catch_unwind(
            std::panic::AssertUnwindSafe(|| custom.GetBooleanValue())
        )
        .is_err());
        assert_eq!(RuleType::kLocation as u8, 33);
    }
    #[test]
    fn layer_names_use_identifier_serialization_including_empty_part_separator_rule() {
        let names = vec![
            vec![AtomicString::from_str("1st"), AtomicString::from_str("a.b")],
            Vec::new(),
            vec![AtomicString::from_str(""), AtomicString::from_str("tail")],
        ];
        let statement = StyleRuleLayerStatement::new(names);
        let values: Vec<_> = statement
            .GetNamesAsStrings()
            .iter()
            .map(|s| s.Utf8())
            .collect();
        assert_eq!(values, ["\\31 st.a\\.b", "", "tail"]);
    }
    // This backend records the real switch's constructor/parent/count calls;
    // it is a dispatch observation fixture, not a production CSSOM object.
    struct WrapperFixture {
        kind: RuleType,
        parent: Option<RuleType>,
        sheet: Option<u8>,
        hint: usize,
    }
    struct WrapperBackend {
        events: Vec<&'static str>,
    }
    impl StyleRuleCSSOMBackend<Dependencies> for WrapperBackend {
        type CSSRule = WrapperFixture;
        type CSSStyleSheet = u8;
        fn CountCSSPageRule(&mut self, _sheet: &u8) {
            self.events.push("count page");
        }
        fn SetParentRule(&mut self, rule: &mut WrapperFixture, parent: &WrapperFixture) {
            self.events.push("parent");
            rule.parent = Some(parent.kind);
        }
        fn NewCSSStyleRule(
            &mut self,
            _rule: &StyleRule<Dependencies>,
            sheet: Option<&u8>,
            position_hint: usize,
        ) -> WrapperFixture {
            self.events.push("Style");
            WrapperFixture {
                kind: RuleType::kStyle,
                parent: None,
                sheet: sheet.copied(),
                hint: position_hint,
            }
        }
        fn NewCSSPageRule(
            &mut self,
            _rule: &StyleRulePage<Dependencies>,
            sheet: Option<&u8>,
        ) -> WrapperFixture {
            self.events.push("Page");
            WrapperFixture {
                kind: RuleType::kPage,
                parent: None,
                sheet: sheet.copied(),
                hint: usize::MAX,
            }
        }
        fn NewCSSPageMarginRule(
            &mut self,
            _rule: &StyleRulePageMargin<Dependencies>,
            sheet: Option<&u8>,
        ) -> WrapperFixture {
            self.events.push("PageMargin");
            WrapperFixture {
                kind: RuleType::kPageMargin,
                parent: None,
                sheet: sheet.copied(),
                hint: usize::MAX,
            }
        }
        fn NewCSSLocationRule(
            &mut self,
            _rule: &UnusedDependency,
            sheet: Option<&u8>,
        ) -> WrapperFixture {
            self.events.push("Location");
            WrapperFixture {
                kind: RuleType::kLocation,
                parent: None,
                sheet: sheet.copied(),
                hint: usize::MAX,
            }
        }
        fn NewCSSNavigationRule(
            &mut self,
            _rule: &StyleRuleNavigation<Dependencies>,
            sheet: Option<&u8>,
        ) -> WrapperFixture {
            self.events.push("Navigation");
            WrapperFixture {
                kind: RuleType::kNavigation,
                parent: None,
                sheet: sheet.copied(),
                hint: usize::MAX,
            }
        }
        fn NewCSSPropertyRule(
            &mut self,
            _rule: &StyleRuleProperty<Dependencies>,
            sheet: Option<&u8>,
        ) -> WrapperFixture {
            self.events.push("Property");
            WrapperFixture {
                kind: RuleType::kProperty,
                parent: None,
                sheet: sheet.copied(),
                hint: usize::MAX,
            }
        }
        fn NewCSSFontFaceRule(
            &mut self,
            _rule: &StyleRuleFontFace<Dependencies>,
            sheet: Option<&u8>,
        ) -> WrapperFixture {
            self.events.push("FontFace");
            WrapperFixture {
                kind: RuleType::kFontFace,
                parent: None,
                sheet: sheet.copied(),
                hint: usize::MAX,
            }
        }
        fn NewCSSFontPaletteValuesRule(
            &mut self,
            _rule: &UnusedDependency,
            sheet: Option<&u8>,
        ) -> WrapperFixture {
            self.events.push("FontPaletteValues");
            WrapperFixture {
                kind: RuleType::kFontPaletteValues,
                parent: None,
                sheet: sheet.copied(),
                hint: usize::MAX,
            }
        }
        fn NewCSSFontFeatureValuesRule(
            &mut self,
            _rule: &UnusedDependency,
            sheet: Option<&u8>,
        ) -> WrapperFixture {
            self.events.push("FontFeatureValues");
            WrapperFixture {
                kind: RuleType::kFontFeatureValues,
                parent: None,
                sheet: sheet.copied(),
                hint: usize::MAX,
            }
        }
        fn NewCSSMediaRule(
            &mut self,
            _rule: &StyleRuleMedia<Dependencies>,
            sheet: Option<&u8>,
        ) -> WrapperFixture {
            self.events.push("Media");
            WrapperFixture {
                kind: RuleType::kMedia,
                parent: None,
                sheet: sheet.copied(),
                hint: usize::MAX,
            }
        }
        fn NewCSSNestedDeclarationsRule(
            &mut self,
            _rule: &NestedFixture,
            sheet: Option<&u8>,
        ) -> WrapperFixture {
            self.events.push("NestedDeclarations");
            WrapperFixture {
                kind: RuleType::kNestedDeclarations,
                parent: None,
                sheet: sheet.copied(),
                hint: usize::MAX,
            }
        }
        fn NewCSSFunctionDeclarationsRule(
            &mut self,
            _rule: &UnusedDependency,
            sheet: Option<&u8>,
        ) -> WrapperFixture {
            self.events.push("FunctionDeclarations");
            WrapperFixture {
                kind: RuleType::kFunctionDeclarations,
                parent: None,
                sheet: sheet.copied(),
                hint: usize::MAX,
            }
        }
        fn NewCSSFunctionRule(
            &mut self,
            _rule: &StyleRuleFunction<Dependencies>,
            sheet: Option<&u8>,
        ) -> WrapperFixture {
            self.events.push("Function");
            WrapperFixture {
                kind: RuleType::kFunction,
                parent: None,
                sheet: sheet.copied(),
                hint: usize::MAX,
            }
        }
        fn NewCSSScopeRule(
            &mut self,
            _rule: &StyleRuleScope<Dependencies>,
            sheet: Option<&u8>,
        ) -> WrapperFixture {
            self.events.push("Scope");
            WrapperFixture {
                kind: RuleType::kScope,
                parent: None,
                sheet: sheet.copied(),
                hint: usize::MAX,
            }
        }
        fn NewCSSSupportsRule(
            &mut self,
            _rule: &StyleRuleSupports<Dependencies>,
            sheet: Option<&u8>,
        ) -> WrapperFixture {
            self.events.push("Supports");
            WrapperFixture {
                kind: RuleType::kSupports,
                parent: None,
                sheet: sheet.copied(),
                hint: usize::MAX,
            }
        }
        fn NewCSSImportRule(
            &mut self,
            _rule: &UnusedDependency,
            sheet: Option<&u8>,
        ) -> WrapperFixture {
            self.events.push("Import");
            WrapperFixture {
                kind: RuleType::kImport,
                parent: None,
                sheet: sheet.copied(),
                hint: usize::MAX,
            }
        }
        fn NewCSSKeyframesRule(
            &mut self,
            _rule: &UnusedDependency,
            sheet: Option<&u8>,
        ) -> WrapperFixture {
            self.events.push("Keyframes");
            WrapperFixture {
                kind: RuleType::kKeyframes,
                parent: None,
                sheet: sheet.copied(),
                hint: usize::MAX,
            }
        }
        fn NewCSSLayerBlockRule(
            &mut self,
            _rule: &StyleRuleLayerBlock<Dependencies>,
            sheet: Option<&u8>,
        ) -> WrapperFixture {
            self.events.push("LayerBlock");
            WrapperFixture {
                kind: RuleType::kLayerBlock,
                parent: None,
                sheet: sheet.copied(),
                hint: usize::MAX,
            }
        }
        fn NewCSSLayerStatementRule(
            &mut self,
            _rule: &StyleRuleLayerStatement,
            sheet: Option<&u8>,
        ) -> WrapperFixture {
            self.events.push("LayerStatement");
            WrapperFixture {
                kind: RuleType::kLayerStatement,
                parent: None,
                sheet: sheet.copied(),
                hint: usize::MAX,
            }
        }
        fn NewCSSNamespaceRule(
            &mut self,
            _rule: &UnusedDependency,
            sheet: Option<&u8>,
        ) -> WrapperFixture {
            self.events.push("Namespace");
            WrapperFixture {
                kind: RuleType::kNamespace,
                parent: None,
                sheet: sheet.copied(),
                hint: usize::MAX,
            }
        }
        fn NewCSSContainerRule(
            &mut self,
            _rule: &StyleRuleContainer<Dependencies>,
            sheet: Option<&u8>,
        ) -> WrapperFixture {
            self.events.push("Container");
            WrapperFixture {
                kind: RuleType::kContainer,
                parent: None,
                sheet: sheet.copied(),
                hint: usize::MAX,
            }
        }
        fn NewCSSCounterStyleRule(
            &mut self,
            _rule: &UnusedDependency,
            sheet: Option<&u8>,
        ) -> WrapperFixture {
            self.events.push("CounterStyle");
            WrapperFixture {
                kind: RuleType::kCounterStyle,
                parent: None,
                sheet: sheet.copied(),
                hint: usize::MAX,
            }
        }
        fn NewCSSStartingStyleRule(
            &mut self,
            _rule: &StyleRuleStartingStyle<Dependencies>,
            sheet: Option<&u8>,
        ) -> WrapperFixture {
            self.events.push("StartingStyle");
            WrapperFixture {
                kind: RuleType::kStartingStyle,
                parent: None,
                sheet: sheet.copied(),
                hint: usize::MAX,
            }
        }
        fn NewCSSViewTransitionRule(
            &mut self,
            _rule: &UnusedDependency,
            sheet: Option<&u8>,
        ) -> WrapperFixture {
            self.events.push("ViewTransition");
            WrapperFixture {
                kind: RuleType::kViewTransition,
                parent: None,
                sheet: sheet.copied(),
                hint: usize::MAX,
            }
        }
        fn NewCSSPositionTryRule(
            &mut self,
            _rule: &UnusedDependency,
            sheet: Option<&u8>,
        ) -> WrapperFixture {
            self.events.push("PositionTry");
            WrapperFixture {
                kind: RuleType::kPositionTry,
                parent: None,
                sheet: sheet.copied(),
                hint: usize::MAX,
            }
        }
        fn NewCSSCustomMediaRule(
            &mut self,
            _rule: &StyleRuleCustomMedia<Dependencies>,
            sheet: Option<&u8>,
        ) -> WrapperFixture {
            self.events.push("CustomMedia");
            WrapperFixture {
                kind: RuleType::kCustomMedia,
                parent: None,
                sheet: sheet.copied(),
                hint: usize::MAX,
            }
        }
        fn NewCSSMixinRule(
            &mut self,
            _rule: &StyleRuleMixin<Dependencies>,
            sheet: Option<&u8>,
        ) -> WrapperFixture {
            self.events.push("Mixin");
            WrapperFixture {
                kind: RuleType::kMixin,
                parent: None,
                sheet: sheet.copied(),
                hint: usize::MAX,
            }
        }
        fn NewCSSResultRule(
            &mut self,
            _rule: &StyleRuleResult<Dependencies>,
            sheet: Option<&u8>,
        ) -> WrapperFixture {
            self.events.push("Result");
            WrapperFixture {
                kind: RuleType::kResult,
                parent: None,
                sheet: sheet.copied(),
                hint: usize::MAX,
            }
        }
        fn NewCSSPrivateRule(
            &mut self,
            _rule: &StyleRulePrivate<Dependencies>,
            sheet: Option<&u8>,
        ) -> WrapperFixture {
            self.events.push("Private");
            WrapperFixture {
                kind: RuleType::kPrivate,
                parent: None,
                sheet: sheet.copied(),
                hint: usize::MAX,
            }
        }
        fn NewCSSApplyMixinRule(
            &mut self,
            _rule: &StyleRuleApplyMixin<Dependencies>,
            sheet: Option<&u8>,
        ) -> WrapperFixture {
            self.events.push("ApplyMixin");
            WrapperFixture {
                kind: RuleType::kApplyMixin,
                parent: None,
                sheet: sheet.copied(),
                hint: usize::MAX,
            }
        }
        fn NewCSSContentsRule(
            &mut self,
            _rule: &StyleRuleContentsStatement<Dependencies>,
            sheet: Option<&u8>,
        ) -> WrapperFixture {
            self.events.push("Contents");
            WrapperFixture {
                kind: RuleType::kContents,
                parent: None,
                sheet: sheet.copied(),
                hint: usize::MAX,
            }
        }
    }
    #[test]
    fn cssom_dispatch_counts_page_before_construction_and_sets_parent_after() {
        let mut backend = WrapperBackend { events: Vec::new() };
        let page = StyleRuleBase::<Dependencies>::Page(StyleRulePage::new(
            Rc::new(SelectorFixture([1, 2, 3], None)),
            property(true, false),
            Vec::new(),
        ));
        let wrapper = page.CreateCSSOMWrapper(&mut backend, 4, Some(&9), true);
        assert_eq!(backend.events, ["count page", "Page"]);
        assert_eq!(wrapper.kind, RuleType::kPage);
        assert_eq!(wrapper.sheet, Some(9));
        backend.events.clear();
        let parent = WrapperFixture {
            kind: RuleType::kMedia,
            parent: None,
            sheet: None,
            hint: 0,
        };
        let wrapper = page.CreateCSSOMWrapperForRule(&mut backend, 4, Some(&parent), true);
        assert_eq!(backend.events, ["Page", "parent"]);
        assert_eq!(wrapper.parent, Some(RuleType::kMedia));
        assert_eq!(wrapper.sheet, None);
        backend.events.clear();
        page.CreateCSSOMWrapper(&mut backend, 0, Some(&9), false);
        assert_eq!(backend.events, ["Page"]);
        let style = StyleRuleBase::<Dependencies>::Style(Rc::new(StyleRule::Create(
            SelectorFixture([1, 2, 3], None),
            property(true, false),
            None,
        )));
        let wrapper = style.CreateCSSOMWrapper(&mut backend, 77, None, false);
        assert_eq!(wrapper.kind, RuleType::kStyle);
        assert_eq!(wrapper.hint, 77);
        let charset = StyleRuleBase::<Dependencies>::Charset(StyleRuleCharset::new());
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(
            || charset.CreateCSSOMWrapper(&mut backend, 0, None, false)
        ))
        .is_err());
    }
}
