// Copyright 2026 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// cpp: third_party/blink/renderer/core/css/parser/css_parser_local_context.h
// cpp: third_party/blink/renderer/core/css/parser/css_parser_local_context.cc

#![allow(non_snake_case)]

use crate::css_property_name::CSSPropertyName;
use crate::css_property_names::{IsPropertyAlias, ResolveCSSPropertyID};
use crate::properties::css_property::CSSProperty;
use foundation::{AtomicString, CSSPropertyID, CSSValueID};

/// Local context carried while parsing one property value.
///
/// C++ stores the same values in a stack-only class. Rust ownership makes the
/// stack-only lifetime explicit without changing the parser state machine.
#[derive(Clone, Debug)]
pub struct CSSParserLocalContext {
    current_shorthand_: CSSPropertyID,
    unresolved_property_name_: Option<CSSPropertyName>,
    functions_stack_: Vec<CSSValueID>,
    custom_function_name_: AtomicString,
    custom_function_count_: usize,
    random_value_count_: usize,
}

impl Default for CSSParserLocalContext {
    fn default() -> Self {
        Self {
            current_shorthand_: CSSPropertyID::kInvalid,
            unresolved_property_name_: None,
            functions_stack_: Vec::new(),
            custom_function_name_: AtomicString::default(),
            custom_function_count_: 0,
            random_value_count_: 0,
        }
    }
}

impl CSSParserLocalContext {
    // cpp: css_parser_local_context.h:21-81
    pub fn CreateWithoutPropertyForCSSOM() -> Self {
        Self::default()
    }
    pub fn CreateWithoutPropertyForSyntaxParsing() -> Self {
        Self::default()
    }
    pub fn CreateWithoutPropertyForIdent() -> Self {
        Self::default()
    }
    pub fn CreateWithoutPropertyForTest() -> Self {
        Self::default()
    }
    pub fn CreateWithoutPropertyForAtRules() -> Self {
        Self::default()
    }
    pub fn CreateWithoutPropertyForSelectors() -> Self {
        Self::default()
    }
    pub fn CreateWithoutPropertyForCanvas() -> Self {
        Self::default()
    }
    pub fn CreateWithoutPropertyForInspector() -> Self {
        Self::default()
    }

    // cpp: css_parser_local_context.h:83-105
    pub fn new(property_name: CSSPropertyName, current_shorthand: CSSPropertyID) -> Self {
        Self::WithCustomFunction(property_name, current_shorthand, AtomicString::default(), 0)
    }

    pub fn WithCustomFunction(
        property_name: CSSPropertyName,
        current_shorthand: CSSPropertyID,
        custom_function_name: AtomicString,
        custom_function_count: usize,
    ) -> Self {
        Self {
            current_shorthand_: current_shorthand,
            unresolved_property_name_: Some(property_name),
            functions_stack_: Vec::new(),
            custom_function_name_: custom_function_name,
            custom_function_count_: custom_function_count,
            random_value_count_: 0,
        }
    }

    pub fn SetCurrentShorthand(&mut self, current_shorthand: CSSPropertyID) {
        self.current_shorthand_ = current_shorthand;
    }
    pub fn SetUnresolvedProperty(&mut self, property_name: CSSPropertyName) {
        self.unresolved_property_name_ = Some(property_name);
    }
    pub fn SetCustomFunctionName(&mut self, custom_function_name: AtomicString) {
        self.custom_function_name_ = custom_function_name;
    }
    pub fn IncrementRandomValueCount(&mut self) {
        self.random_value_count_ += 1;
    }
    pub fn SetRandomValueCount(&mut self, count: usize) {
        self.random_value_count_ = count;
    }

    pub fn UseAliasParsing(&self) -> bool {
        self.unresolved_property_name_
            .as_ref()
            .is_some_and(|name| !name.IsCustomProperty() && IsPropertyAlias(name.Id()))
    }
    pub fn CurrentShorthand(&self) -> CSSPropertyID {
        self.current_shorthand_
    }
    pub fn UnresolvedPropertyName(&self) -> Option<&CSSPropertyName> {
        self.unresolved_property_name_.as_ref()
    }
    pub fn RandomValueCount(&self) -> usize {
        self.random_value_count_
    }
    pub fn CurrentRandomValueIndex(&self) -> usize {
        self.random_value_count_ + 1
    }

    // cpp: css_parser_local_context.cc:120-155
    pub fn CustomFunctionName(&self) -> AtomicString {
        if self.custom_function_name_.IsNull() {
            return AtomicString::from_str("");
        }
        AtomicString::from_str(&format!("{};", self.custom_function_name_.Utf8()))
    }

    pub fn CustomFunctionNameAndCnt(&self) -> AtomicString {
        if self.custom_function_name_.IsNull() {
            return AtomicString::from_str("");
        }
        AtomicString::from_str(&format!(
            "{}{};",
            self.custom_function_name_.Utf8(),
            self.custom_function_count_
        ))
    }

    pub fn PropertyName(&self) -> AtomicString {
        let Some(unresolved) = self.unresolved_property_name_.as_ref() else {
            return AtomicString::from_str("");
        };
        if unresolved.Id() == CSSPropertyID::kInvalid {
            return AtomicString::from_str("");
        }
        if self.current_shorthand_ != CSSPropertyID::kInvalid {
            return CSSPropertyName::new(self.current_shorthand_)
                .ToAtomicString()
                .clone();
        }
        if unresolved.IsCustomProperty() {
            return unresolved.ToAtomicString().clone();
        }
        CSSPropertyName::new(ResolveCSSPropertyID(unresolved.Id()))
            .ToAtomicString()
            .clone()
    }

    // cpp: css_parser_local_context.cc:12-96
    pub fn PercentagesDependOnUsedValue(&self) -> bool {
        let Some(property_name) = self.unresolved_property_name_.as_ref() else {
            return false;
        };
        let id = property_name.Id();
        if id == CSSPropertyID::kInvalid || id == CSSPropertyID::kVariable {
            return false;
        }
        if let Some(current_function_id) = self.functions_stack_.last().copied() {
            return match current_function_id {
                CSSValueID::kCrossFade
                | CSSValueID::kWebkitCrossFade
                | CSSValueID::kConicGradient
                | CSSValueID::kRadialGradient
                | CSSValueID::kWebkitRadialGradient
                | CSSValueID::kWebkitGradient
                | CSSValueID::kInset
                | CSSValueID::kXywh
                | CSSValueID::kRect
                | CSSValueID::kCircle
                | CSSValueID::kEllipse
                | CSSValueID::kPolygon
                | CSSValueID::kShape
                | CSSValueID::kScale3d
                | CSSValueID::kScaleZ
                | CSSValueID::kTranslate
                | CSSValueID::kTranslateX
                | CSSValueID::kTranslateY
                | CSSValueID::kTranslate3d
                | CSSValueID::kRepeat
                | CSSValueID::kRay
                | CSSValueID::kView => true,

                CSSValueID::kAttr
                | CSSValueID::kAlpha
                | CSSValueID::kBlur
                | CSSValueID::kBrightness
                | CSSValueID::kColor
                | CSSValueID::kColorMix
                | CSSValueID::kColorStop
                | CSSValueID::kContrast
                | CSSValueID::kDropShadow
                | CSSValueID::kDynamicRangeLimitMix
                | CSSValueID::kGrayscale
                | CSSValueID::kHsl
                | CSSValueID::kHsla
                | CSSValueID::kHueRotate
                | CSSValueID::kHwb
                | CSSValueID::kIf
                | CSSValueID::kInvert
                | CSSValueID::kLab
                | CSSValueID::kLch
                | CSSValueID::kLinear
                | CSSValueID::kMatrix
                | CSSValueID::kMatrix3d
                | CSSValueID::kOklab
                | CSSValueID::kOklch
                | CSSValueID::kOpacity
                | CSSValueID::kPaletteMix
                | CSSValueID::kPath
                | CSSValueID::kPerspective
                | CSSValueID::kRgb
                | CSSValueID::kRgba
                | CSSValueID::kRotate
                | CSSValueID::kRotate3d
                | CSSValueID::kRotateX
                | CSSValueID::kRotateY
                | CSSValueID::kRotateZ
                | CSSValueID::kSaturate
                | CSSValueID::kScale
                | CSSValueID::kScaleX
                | CSSValueID::kScaleY
                | CSSValueID::kSepia
                | CSSValueID::kSkew
                | CSSValueID::kSkewX
                | CSSValueID::kSkewY
                | CSSValueID::kTranslateZ => false,
                _ => panic!("function must declare percentage dependency"),
            };
        }
        CSSProperty::Get(ResolveCSSPropertyID(id)).PercentagesDependOnUsedValue()
    }

    // cpp: css_parser_local_context.cc:98-116. Chromium only calls this in
    // DCHECK builds; keeping it available makes the generated property flag
    // invariant testable without changing release behavior.
    pub fn CheckPercentagesFlagSetOnProperty(&self) {
        let Some(property_name) = self.unresolved_property_name_.as_ref() else {
            return;
        };
        if !self.functions_stack_.is_empty()
            || !self.custom_function_name_.IsNull()
            || property_name.IsCustomProperty()
            || property_name.Id() == CSSPropertyID::kInvalid
            || ResolveCSSPropertyID(property_name.Id()) == self.current_shorthand_
        {
            return;
        }
        let property = CSSProperty::Get(ResolveCSSPropertyID(property_name.Id()));
        debug_assert!(
            property.PercentagesDependOnUsedValue() || property.PercentagesDoNotDependOnUsedValue()
        );
    }

    pub fn EnterFunction(&mut self, function_id: CSSValueID) -> FunctionLocalContext<'_> {
        self.functions_stack_.push(function_id);
        FunctionLocalContext {
            local_context_: self,
        }
    }
}

// cpp: css_parser_local_context.h:144-159
pub struct FunctionLocalContext<'a> {
    local_context_: &'a mut CSSParserLocalContext,
}

impl FunctionLocalContext<'_> {
    pub fn Context(&mut self) -> &mut CSSParserLocalContext {
        self.local_context_
    }
}

impl Drop for FunctionLocalContext<'_> {
    fn drop(&mut self) {
        debug_assert!(!self.local_context_.functions_stack_.is_empty());
        self.local_context_.functions_stack_.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_aliases_and_counters_follow_property_context() {
        let mut context = CSSParserLocalContext::new(
            CSSPropertyName::new(CSSPropertyID::kAliasWordWrap),
            CSSPropertyID::kInvalid,
        );
        assert!(context.UseAliasParsing());
        assert_eq!(context.PropertyName().Utf8(), "overflow-wrap");
        context.SetCurrentShorthand(CSSPropertyID::kBackground);
        assert_eq!(context.PropertyName().Utf8(), "background");
        assert_eq!(context.CurrentRandomValueIndex(), 1);
        context.IncrementRandomValueCount();
        assert_eq!(context.CurrentRandomValueIndex(), 2);
    }

    #[test]
    fn function_context_overrides_property_percentage_behavior_and_restores() {
        let mut context = CSSParserLocalContext::new(
            CSSPropertyName::new(CSSPropertyID::kOpacity),
            CSSPropertyID::kInvalid,
        );
        let property_answer = context.PercentagesDependOnUsedValue();
        {
            let mut scope = context.EnterFunction(CSSValueID::kTranslateX);
            assert!(scope.Context().PercentagesDependOnUsedValue());
        }
        assert_eq!(context.PercentagesDependOnUsedValue(), property_answer);
        {
            let mut scope = context.EnterFunction(CSSValueID::kColorMix);
            assert!(!scope.Context().PercentagesDependOnUsedValue());
        }
    }

    #[test]
    fn custom_function_and_custom_property_names_match_chromium_format() {
        let context = CSSParserLocalContext::WithCustomFunction(
            CSSPropertyName::custom(AtomicString::from_str("--x")),
            CSSPropertyID::kInvalid,
            AtomicString::from_str("--f"),
            3,
        );
        assert_eq!(context.CustomFunctionName().Utf8(), "--f;");
        assert_eq!(context.CustomFunctionNameAndCnt().Utf8(), "--f3;");
        assert_eq!(context.PropertyName().Utf8(), "--x");
    }
}
