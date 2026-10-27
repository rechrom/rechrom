// Copyright 2015 The Chromium Authors. BSD-style license; see Chromium LICENSE.
// cpp: css_supports_parser.h:20-117; css_supports_parser.cc:24-315.
// The three-state Boolean operators preserve parse failures even in short-circuit
// expressions. External selector/runtime registries remain typed dependencies.
#![allow(non_snake_case)]
use super::{
    conditional_parser::{AtIdent, ConsumeAnyValue, ConsumeIfIdent},
    css_at_rule_id::{CSSAtRuleID, CssAtRuleID},
    css_parser_impl::{
        CSSParserImpl, CSSParserImplRuleBackend, CSSParserImplRuleObserver, SupportsResult,
    },
    css_parser_mode::CSSParserMode,
    css_parser_token::CSSParserTokenType::*,
    css_parser_token_stream::{CSSParserTokenStream, RestoringBlockGuard, TokenStreamTokenizer},
};
use foundation::CSSValueID;
use std::ops::{BitAnd, BitOr, Not};
impl Not for SupportsResult {
    type Output = Self;
    fn not(self) -> Self {
        match self {
            Self::kSupported => Self::kUnsupported,
            Self::kUnsupported => Self::kSupported,
            Self::kParseFailure => self,
        }
    }
}
impl BitAnd for SupportsResult {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        if self == Self::kParseFailure || rhs == Self::kParseFailure {
            Self::kParseFailure
        } else if self != Self::kSupported || rhs != Self::kSupported {
            Self::kUnsupported
        } else {
            Self::kSupported
        }
    }
}
impl BitOr for SupportsResult {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        if self == Self::kParseFailure || rhs == Self::kParseFailure {
            Self::kParseFailure
        } else if self == Self::kSupported || rhs == Self::kSupported {
            Self::kSupported
        } else {
            Self::kUnsupported
        }
    }
}
pub trait CSSSupportsBackend: CSSParserImplRuleBackend {
    fn SupportsComplexSelector<T: TokenStreamTokenizer>(
        parser: &mut CSSParserImpl<'_, Self>,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> bool;
    fn CSSSupportsAtRuleFunctionEnabled() -> bool;
    fn CSSSupportsNamedFeatureFunctionEnabled() -> bool;
    fn IsSupportedNamedFeature(id: CSSValueID) -> bool;
    fn IsBlinkFeatureEnabled(name: &foundation::String) -> bool;
}
pub struct CSSSupportsParser;
impl CSSSupportsParser {
    // cpp: .cc:24-30; .h:33.
    pub fn ConsumeSupportsCondition<B: CSSSupportsBackend, T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
        parser: &mut CSSParserImpl<'_, B>,
    ) -> SupportsResult
    where
        B::CSSParserObserver: CSSParserImplRuleObserver,
    {
        stream.ConsumeWhitespace();
        Self::Condition(stream, parser)
    }
    // cpp: .cc:43-65.
    fn Condition<B: CSSSupportsBackend, T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
        parser: &mut CSSParserImpl<'_, B>,
    ) -> SupportsResult
    where
        B::CSSParserObserver: CSSParserImplRuleObserver,
    {
        if ConsumeIfIdent(stream, "not") {
            return !Self::InParens(stream, parser);
        }
        let mut result = Self::InParens(stream, parser);
        if AtIdent(stream.Peek(), "and") {
            while ConsumeIfIdent(stream, "and") {
                result = result & Self::InParens(stream, parser);
            }
        } else if AtIdent(stream.Peek(), "or") {
            while ConsumeIfIdent(stream, "or") {
                result = result | Self::InParens(stream, parser);
            }
        }
        result
    }
    // cpp: .cc:70-102. A failed nested condition still gets the feature and
    // general-enclosed recovery attempts after RestoringBlockGuard restores it.
    fn InParens<B: CSSSupportsBackend, T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
        parser: &mut CSSParserImpl<'_, B>,
    ) -> SupportsResult
    where
        B::CSSParserObserver: CSSParserImplRuleObserver,
    {
        if stream.Peek().GetType() == kLeftParenthesisToken {
            let mut guard = RestoringBlockGuard::new(stream);
            guard.ConsumeWhitespace();
            let result = Self::Condition(&mut guard, parser);
            if result == SupportsResult::kSupported && guard.Release() {
                guard.ConsumeWhitespace();
                return result;
            }
        }
        if Self::Feature(stream, parser) {
            return SupportsResult::kSupported;
        }
        if Self::GeneralEnclosed(stream) {
            return SupportsResult::kUnsupported;
        }
        SupportsResult::kParseFailure
    }
    // cpp: .cc:110-138.
    fn Feature<B: CSSSupportsBackend, T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
        parser: &mut CSSParserImpl<'_, B>,
    ) -> bool
    where
        B::CSSParserObserver: CSSParserImplRuleObserver,
    {
        if Self::SelectorFn(stream, parser)
            || Self::FontFn(stream, true)
            || Self::FontFn(stream, false)
            || Self::AtRuleFn::<B, T>(stream)
            || Self::NamedFeatureFn::<B, T>(stream)
        {
            return true;
        }
        if parser.GetMode() == CSSParserMode::kUASheetMode && Self::BlinkFeatureFn::<B, T>(stream) {
            return true;
        }
        Self::Declaration(stream, parser)
    }
    // cpp: .cc:141-156.
    fn SelectorFn<B: CSSSupportsBackend, T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
        parser: &mut CSSParserImpl<'_, B>,
    ) -> bool
    where
        B::CSSParserObserver: CSSParserImplRuleObserver,
    {
        if stream.Peek().FunctionId() != Some(CSSValueID::kSelector) {
            return false;
        }
        let mut guard = RestoringBlockGuard::new(stream);
        guard.ConsumeWhitespace();
        if B::SupportsComplexSelector(parser, &mut guard) && guard.Release() {
            guard.ConsumeWhitespace();
            return true;
        }
        false
    }
    // cpp: .cc:158-197; css_parsing_utils.cc:6795-6867.
    fn FontFn<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
        tech: bool,
    ) -> bool {
        let function = if tech {
            CSSValueID::kFontTech
        } else {
            CSSValueID::kFontFormat
        };
        if stream.Peek().FunctionId() != Some(function) {
            return false;
        }
        let mut guard = RestoringBlockGuard::new(stream);
        guard.ConsumeWhitespace();
        let id = guard.Peek().Id();
        let supported = if tech {
            matches!(
                id,
                CSSValueID::kFeaturesOpentype
                    | CSSValueID::kFeaturesAat
                    | CSSValueID::kColorCOLRv0
                    | CSSValueID::kColorCOLRv1
                    | CSSValueID::kColorSbix
                    | CSSValueID::kColorCBDT
                    | CSSValueID::kVariations
                    | CSSValueID::kPalettes
            )
        } else {
            matches!(
                id,
                CSSValueID::kCollection
                    | CSSValueID::kOpentype
                    | CSSValueID::kTruetype
                    | CSSValueID::kWoff
                    | CSSValueID::kWoff2
            )
        };
        if supported {
            guard.ConsumeIncludingWhitespace();
            if guard.Release() {
                guard.ConsumeWhitespace();
                return true;
            }
        }
        false
    }
    // cpp: .cc:200-227.
    fn AtRuleFn<B: CSSSupportsBackend, T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> bool {
        if !B::CSSSupportsAtRuleFunctionEnabled()
            || stream.Peek().FunctionId() != Some(CSSValueID::kAtRule)
        {
            return false;
        }
        let mut guard = RestoringBlockGuard::new(stream);
        guard.ConsumeWhitespace();
        if guard.Peek().GetType() != kAtKeywordToken {
            return false;
        }
        let token = guard.ConsumeIncludingWhitespace();
        let id = CssAtRuleID(&token.Value(), B::AtRuleFeatures());
        if id == CSSAtRuleID::kCSSAtRuleInvalid {
            return false;
        }
        if guard.Release() && id != CSSAtRuleID::kCSSAtRuleCharset {
            guard.ConsumeWhitespace();
            return true;
        }
        false
    }
    // cpp: .cc:243-264. IsSupportedNamedFeature is the embedding registry.
    fn NamedFeatureFn<B: CSSSupportsBackend, T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> bool {
        if !B::CSSSupportsNamedFeatureFunctionEnabled()
            || stream.Peek().FunctionId() != Some(CSSValueID::kNamedFeature)
        {
            return false;
        }
        let mut guard = RestoringBlockGuard::new(stream);
        guard.ConsumeWhitespace();
        let id = guard.Peek().Id();
        if guard.Peek().GetType() == kIdentToken && B::IsSupportedNamedFeature(id) {
            guard.ConsumeIncludingWhitespace();
            if guard.Release() {
                guard.ConsumeWhitespace();
                return true;
            }
        }
        false
    }
    // cpp: .cc:267-280.
    fn Declaration<B: CSSSupportsBackend, T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
        parser: &mut CSSParserImpl<'_, B>,
    ) -> bool
    where
        B::CSSParserObserver: CSSParserImplRuleObserver,
    {
        if stream.Peek().GetType() != kLeftParenthesisToken {
            return false;
        }
        let mut guard = RestoringBlockGuard::new(stream);
        guard.ConsumeWhitespace();
        if guard.Peek().GetType() == kIdentToken
            && parser.ConsumeSupportsDeclaration(&mut guard)
            && guard.Release()
        {
            guard.ConsumeWhitespace();
            return true;
        }
        false
    }
    // cpp: .cc:284-297. The existing translated ConsumeAnyValue rejects bad
    // string/URL tokens and respects component-value block boundaries.
    fn GeneralEnclosed<T: TokenStreamTokenizer>(stream: &mut CSSParserTokenStream<'_, T>) -> bool {
        if !matches!(
            stream.Peek().GetType(),
            kLeftParenthesisToken | kFunctionToken
        ) {
            return false;
        }
        let mut guard = RestoringBlockGuard::new(stream);
        ConsumeAnyValue(&mut guard);
        if guard.Release() {
            guard.ConsumeWhitespace();
            return true;
        }
        false
    }
    // cpp: .cc:299-315.
    fn BlinkFeatureFn<B: CSSSupportsBackend, T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> bool {
        if stream.Peek().FunctionId() != Some(CSSValueID::kBlinkFeature) {
            return false;
        }
        let mut guard = RestoringBlockGuard::new(stream);
        guard.ConsumeWhitespace();
        if guard.Peek().GetType() == kIdentToken {
            let name = guard.ConsumeIncludingWhitespace().Value().ToString();
            if B::IsBlinkFeatureEnabled(&name) && guard.Release() {
                guard.ConsumeWhitespace();
                return true;
            }
        }
        false
    }
}
