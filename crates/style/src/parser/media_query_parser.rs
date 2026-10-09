// Copyright 2014 The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Source commit: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
// Ledger uses C++ physical / effective / mapped / omitted / pending lines.
// Effective excludes comments, blanks, preprocessor/includes, namespaces and
// lines consisting only of braces/parentheses/semicolons.
// parser/media_query_parser.cc: 661 / 418 / 348 / 10 / 60.
// Mapped: 25-28,30-164,176-228,263-339,397-438,447-659.
// Omitted: 165-174 (GC allocation of fake context becomes required backend).
// Pending whole-source container style-range path: 229-259,341-395,441-445.
// This file completes the concrete media FeatureSet path; it does not claim
// completion of Chromium's shared container-query parsing dependencies.
// MediaQueryExpValue::Consume .cc:491-605 is a required dependency, counted
// pending in media_query_exp.rs rather than counted as translated here.
// Source ledger (third_party/blink/renderer/core/css/):
// - parser/media_query_parser.cc:30-228,263-339,397-438,447-659: media-query path.
// - media_query_exp.cc:479-488: plain-feature value wrapping.
// - parser/css_variable_parser.cc:39-43: variable-name test.
// Required backend: CSSValue parsing, runtime feature queries, range UseCounter.
// Outside this media-only path: .cc:229-259,341-395,441-445 (container style ranges).
// GC/context allocation .cc:165-174 becomes backend ownership. No parser fallback.
#![allow(non_snake_case, non_camel_case_types)]
use super::conditional_parser::{ConditionalParser, ConsumeIfDelimiter, ConsumeIfIdent, ParseMode};
use super::css_parser_token::{CSSParserToken, CSSParserTokenType::*};
use super::css_parser_token_stream::{Boundary, CSSParserTokenStream, TokenStreamTokenizer};
use super::css_tokenizer::CSSTokenizer;
use crate::media_queries::conditional_exp_node::ConditionalExpNode;
use crate::media_queries::media_query::{MediaQuery, RestrictorType};
use crate::media_queries::media_query_exp::{
    MediaQueryExp, MediaQueryExpBounds, MediaQueryExpComparison, MediaQueryExpValue,
    MediaQueryOperator,
};
use crate::media_queries::media_query_set::{MediaQuerySet, MediaQuerySetParser};
use foundation::{AtomicString, String, StringView};
use std::rc::Rc;

// Exact RuntimeEnabledFeatures dependency surface. The actual execution context
// must supply these values; there is deliberately no default implementation.
pub trait MediaQueryRuntimeFeatures {
    fn CSSCustomMediaEnabled(&self) -> bool;
    fn CSSScrolledContainerQueriesEnabled(&self) -> bool;
    fn PrefersReducedDataEnabled(&self) -> bool;
    fn ForcedColorsEnabled(&self) -> bool;
    fn MediaQueryNavigationControlsEnabled(&self) -> bool;
    fn OriginTrialsSampleAPIEnabled(&self) -> bool;
    fn ViewportSegmentsEnabled(&self) -> bool;
    fn DevicePostureEnabled(&self) -> bool;
    fn InvertedColorsEnabled(&self) -> bool;
    fn DesktopPWAsAdditionalWindowingControlsEnabled(&self) -> bool;
}
pub trait MediaQueryParserBackend: MediaQueryRuntimeFeatures {
    type Value;
    type UnparsedValue;
    // media_query_exp.cc:491-605 is a required real CSS parser dependency.
    fn ConsumeValue<T: TokenStreamTokenizer>(
        &mut self,
        feature: &AtomicString,
        stream: &mut CSSParserTokenStream<'_, T>,
        supports_element_dependent: bool,
    ) -> Option<MediaQueryExpValue<Self::Value>>;
    fn UseCountRangeSyntax(&mut self);
}

/// Chromium FeatureSet boundary shared by media and container consumers.
/// Implementations preserve each grammar's case and element-dependent rules.
pub trait MediaQueryFeatureSet<B: MediaQueryParserBackend> {
    fn IsAllowed(&self, name: &AtomicString, runtime: &B) -> bool;
    fn IsAllowedWithoutValue(&self, name: &AtomicString, runtime: &B) -> bool;
    fn IsAllowedWithValue(&self, name: &AtomicString, runtime: &B) -> bool;
    fn IsRangeTypeFeature(&self, name: &AtomicString) -> bool;
    fn IsCaseSensitive(&self) -> bool;
    fn SupportsElementDependent(&self) -> bool;
    fn SupportsStyleRange(&self) -> bool;
    fn ConsumeStyleRange<T: TokenStreamTokenizer>(
        &self,
        backend: &mut B,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<ConditionalExpNode<B::Value, B::UnparsedValue>>>;
}
struct MediaFeatureSet;
impl<B: MediaQueryParserBackend> MediaQueryFeatureSet<B> for MediaFeatureSet {
    fn IsAllowed(&self, n: &AtomicString, b: &B) -> bool {
        IsAllowed(n, b)
    }
    fn IsAllowedWithoutValue(&self, n: &AtomicString, b: &B) -> bool {
        IsAllowedWithoutValue(n, b)
    }
    fn IsAllowedWithValue(&self, n: &AtomicString, b: &B) -> bool {
        IsAllowedWithValue(n, b)
    }
    fn IsRangeTypeFeature(&self, n: &AtomicString) -> bool {
        IsRangeTypeFeature(n)
    }
    fn IsCaseSensitive(&self) -> bool {
        false
    }
    fn SupportsElementDependent(&self) -> bool {
        false
    }
    fn SupportsStyleRange(&self) -> bool {
        false
    }
    fn ConsumeStyleRange<T: TokenStreamTokenizer>(
        &self,
        _: &mut B,
        _: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<ConditionalExpNode<B::Value, B::UnparsedValue>>> {
        unreachable!("media features have no style ranges")
    }
}

// cpp: parser/css_variable_parser.cc:39-43
fn IsValidVariableName(name: &AtomicString) -> bool {
    name.length() >= 3 && name.at(0) == 45 && name.at(1) == 45
}
// cpp: parser/media_query_parser.cc:30-48
fn IsAllowed(feature: &AtomicString, runtime: &impl MediaQueryRuntimeFeatures) -> bool {
    !(matches!(
        feature.Utf8().as_str(),
        "inline-size"
            | "min-inline-size"
            | "max-inline-size"
            | "block-size"
            | "min-block-size"
            | "max-block-size"
            | "stuck"
            | "snapped"
            | "scrollable"
    ) || (feature == "scrolled" && runtime.CSSScrolledContainerQueriesEnabled())
        || (IsValidVariableName(feature) && !runtime.CSSCustomMediaEnabled()))
}
// cpp: parser/media_query_parser.cc:50-111
fn IsAllowedWithoutValue(feature: &AtomicString, runtime: &impl MediaQueryRuntimeFeatures) -> bool {
    matches!(
        feature.Utf8().as_str(),
        "monochrome"
            | "color"
            | "color-index"
            | "grid"
            | "height"
            | "width"
            | "device-height"
            | "device-width"
            | "orientation"
            | "aspect-ratio"
            | "device-aspect-ratio"
            | "hover"
            | "any-hover"
            | "-webkit-transform-3d"
            | "pointer"
            | "any-pointer"
            | "-webkit-device-pixel-ratio"
            | "resolution"
            | "display-mode"
            | "scan"
            | "color-gamut"
            | "prefers-color-scheme"
            | "prefers-contrast"
            | "prefers-reduced-motion"
            | "overflow-inline"
            | "overflow-block"
            | "update"
            | "prefers-reduced-transparency"
            | "scripting"
    ) || (feature == "prefers-reduced-data" && runtime.PrefersReducedDataEnabled())
        || (feature == "forced-colors" && runtime.ForcedColorsEnabled())
        || (feature == "navigation-controls" && runtime.MediaQueryNavigationControlsEnabled())
        || (feature == "origin-trial-test" && runtime.OriginTrialsSampleAPIEnabled())
        || (matches!(
            feature.Utf8().as_str(),
            "horizontal-viewport-segments" | "vertical-viewport-segments"
        ) && runtime.ViewportSegmentsEnabled())
        || (feature == "device-posture" && runtime.DevicePostureEnabled())
        || (feature == "inverted-colors" && runtime.InvertedColorsEnabled())
        || IsValidVariableName(feature)
        || (runtime.DesktopPWAsAdditionalWindowingControlsEnabled()
            && matches!(feature.Utf8().as_str(), "display-state" | "resizable"))
}
// cpp: parser/media_query_parser.cc:113-117
fn IsAllowedWithValue(feature: &AtomicString, runtime: &impl MediaQueryRuntimeFeatures) -> bool {
    !runtime.CSSCustomMediaEnabled() || !IsValidVariableName(feature)
}
// cpp: parser/media_query_parser.cc:119-135
fn IsRangeTypeFeature(feature: &AtomicString) -> bool {
    matches!(
        feature.Utf8().as_str(),
        "height"
            | "width"
            | "device-height"
            | "device-width"
            | "aspect-ratio"
            | "device-aspect-ratio"
            | "resolution"
            | "color"
            | "color-index"
            | "monochrome"
            | "-webkit-device-pixel-ratio"
            | "horizontal-viewport-segments"
            | "vertical-viewport-segments"
    )
}
// cpp: parser/media_query_parser.cc:178-185
fn IsRestrictorOrLogicalOperator(token: &CSSParserToken) -> bool {
    ["not", "and", "or", "only", "layer"]
        .iter()
        .any(|word| token.Value().ToString().Utf8().eq_ignore_ascii_case(word))
}
// cpp: parser/media_query_parser.cc:187-195
fn ConsumeUntilCommaInclusive<T: TokenStreamTokenizer>(
    stream: &mut CSSParserTokenStream<'_, T>,
) -> bool {
    stream.SkipUntilPeekedTypeIs(&[kCommaToken]);
    if stream.Peek().GetType() != kCommaToken {
        return false;
    }
    stream.ConsumeIncludingWhitespace();
    true
}
// cpp: parser/media_query_parser.cc:197-219
fn IsComparisonDelimiter(c: u16) -> bool {
    matches!(c, 60 | 62 | 61)
}
fn SkipUntilComparisonOrColon<T: TokenStreamTokenizer>(stream: &mut CSSParserTokenStream<'_, T>) {
    while !stream.AtEnd() {
        stream.SkipUntilPeekedTypeIs(&[kDelimiterToken, kColonToken]);
        if stream.AtEnd() {
            return;
        }
        if stream.Peek().GetType() == kDelimiterToken {
            if IsComparisonDelimiter(stream.Peek().Delimiter()) {
                return;
            }
            stream.Consume();
        } else {
            debug_assert_eq!(stream.Peek().GetType(), kColonToken);
            return;
        }
    }
}

pub struct MediaQueryParser<B: MediaQueryParserBackend> {
    backend: B,
}
impl<B: MediaQueryParserBackend> MediaQueryParser<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }
    pub fn Backend(&self) -> &B {
        &self.backend
    }
    // cpp: parser/media_query_parser.cc:137-163
    pub fn ParseMediaQuerySet(
        &mut self,
        text: StringView,
    ) -> MediaQuerySet<B::Value, B::UnparsedValue> {
        let mut stream = CSSParserTokenStream::<CSSTokenizer>::new(text, 0);
        self.ParseMediaQuerySetFromStream(&mut stream)
    }
    pub fn ParseMediaQuerySetFromStream<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> MediaQuerySet<B::Value, B::UnparsedValue> {
        self.ParseImpl(stream, false)
    }
    pub fn ParseMediaCondition<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> MediaQuerySet<B::Value, B::UnparsedValue> {
        self.ParseImpl(stream, true)
    }
    pub fn ParseCustomMediaDefinition<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> MediaQuerySet<B::Value, B::UnparsedValue> {
        let mut boundary = Boundary::new(stream, kSemicolonToken);
        self.ParseMediaQuerySetFromStream(&mut boundary)
    }
    // cpp: parser/media_query_parser.cc:263-282
    fn ConsumeRestrictor<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> RestrictorType {
        if ConsumeIfIdent(stream, "not") {
            RestrictorType::kNot
        } else if ConsumeIfIdent(stream, "only") {
            RestrictorType::kOnly
        } else {
            RestrictorType::kNone
        }
    }
    fn ConsumeType<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<AtomicString> {
        if stream.Peek().GetType() != kIdentToken || IsRestrictorOrLogicalOperator(stream.Peek()) {
            return None;
        }
        let value = stream.ConsumeIncludingWhitespace().Value().ToString();
        Some(AtomFromString(&value))
    }
    // cpp: parser/media_query_parser.cc:284-312
    pub fn ConsumeComparison<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> MediaQueryOperator {
        use MediaQueryOperator::*;
        if stream.Peek().GetType() != kDelimiterToken
            || !IsComparisonDelimiter(stream.Peek().Delimiter())
        {
            return kNone;
        }
        match stream.Peek().Delimiter() {
            61 => {
                stream.ConsumeIncludingWhitespace();
                kEq
            }
            c @ (60 | 62) => {
                stream.Consume();
                if ConsumeIfDelimiter(stream, 61) {
                    if c == 60 {
                        kLe
                    } else {
                        kGe
                    }
                } else {
                    stream.ConsumeWhitespace();
                    if c == 60 {
                        kLt
                    } else {
                        kGt
                    }
                }
            }
            _ => unreachable!(),
        }
    }
    // cpp: parser/media_query_parser.cc:314-339; .h:95-100 (media features are case insensitive).
    fn ConsumeAllowedName<T: TokenStreamTokenizer, F: MediaQueryFeatureSet<B>>(
        &self,
        stream: &mut CSSParserTokenStream<'_, T>,
        set: &F,
    ) -> Option<AtomicString> {
        if stream.Peek().GetType() != kIdentToken {
            return None;
        }
        let mut name = AtomFromString(&stream.Peek().Value().ToString());
        if !set.IsCaseSensitive() {
            name = name.ToAsciiLower();
        }
        if !set.IsAllowed(&name, &self.backend) {
            return None;
        }
        stream.ConsumeIncludingWhitespace();
        Some(name)
    }
    fn ConsumeRangeContextFeatureName<T: TokenStreamTokenizer, F: MediaQueryFeatureSet<B>>(
        &self,
        stream: &mut CSSParserTokenStream<'_, T>,
        set: &F,
    ) -> Option<AtomicString> {
        self.ConsumeAllowedName(stream, set)
            .filter(|n| set.IsRangeTypeFeature(n))
    }
    // cpp: parser/media_query_parser.cc:397-438,447-564 (concrete media FeatureSet).
    pub fn ConsumeFeatureWithSet<T: TokenStreamTokenizer, F: MediaQueryFeatureSet<B>>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
        set: &F,
    ) -> Option<Rc<ConditionalExpNode<B::Value, B::UnparsedValue>>> {
        use MediaQueryOperator::*;
        let start = stream.Save();
        if let Some(name) = self.ConsumeAllowedName(stream, set) {
            if stream.AtEnd() && set.IsAllowedWithoutValue(&name, &self.backend) {
                let exp = if self.backend.CSSCustomMediaEnabled()
                    && IsValidVariableName(&name)
                    && !set.IsAllowedWithValue(&name, &self.backend)
                {
                    MediaQueryExp::CreateCustomMedia(&name)
                } else {
                    MediaQueryExp::CreateWithBounds(&name, &MediaQueryExpBounds::default())
                };
                return Some(Rc::new(ConditionalExpNode::Feature(exp)));
            }
            if stream.Peek().GetType() == kColonToken
                && set.IsAllowedWithValue(&name, &self.backend)
            {
                stream.ConsumeIncludingWhitespace();
                // media_query_exp.cc:479-488: empty values are delegated too.
                if let Some(value) =
                    self.backend
                        .ConsumeValue(&name, stream, set.SupportsElementDependent())
                {
                    if stream.AtEnd() {
                        return Some(Self::Feature(
                            &name,
                            MediaQueryExpComparison::default(),
                            MediaQueryExpComparison::FromValue(&value),
                        ));
                    }
                }
            }
        }
        stream.Restore(start);
        if set.SupportsStyleRange() {
            return set.ConsumeStyleRange(&mut self.backend, stream);
        }
        if let Some(name) = self.ConsumeRangeContextFeatureName(stream, set) {
            if !stream.AtEnd() {
                let op = Self::ConsumeComparison(stream);
                if op != kNone {
                    if let Some(value) =
                        self.backend
                            .ConsumeValue(&name, stream, set.SupportsElementDependent())
                    {
                        if stream.AtEnd() {
                            self.backend.UseCountRangeSyntax();
                            return Some(Self::Feature(
                                &name,
                                MediaQueryExpComparison::default(),
                                MediaQueryExpComparison::new(&value, op),
                            ));
                        }
                    }
                }
            }
        }
        stream.Restore(start);
        SkipUntilComparisonOrColon(stream);
        if stream.AtEnd() {
            return None;
        }
        let offset_after_value1 = stream.LookAheadOffset();
        let op1 = Self::ConsumeComparison(stream);
        if op1 == kNone {
            return None;
        }
        let name = self.ConsumeRangeContextFeatureName(stream, set)?;
        stream.ConsumeWhitespace();
        let after_feature_name = stream.Save();
        stream.Restore(start);
        let value1 = self
            .backend
            .ConsumeValue(&name, stream, set.SupportsElementDependent())?;
        if stream.LookAheadOffset() != offset_after_value1 {
            return None;
        }
        stream.Restore(after_feature_name);
        if stream.AtEnd() {
            self.backend.UseCountRangeSyntax();
            return Some(Self::Feature(
                &name,
                MediaQueryExpComparison::new(&value1, op1),
                MediaQueryExpComparison::default(),
            ));
        }
        let op2 = Self::ConsumeComparison(stream);
        if !((matches!(op1, kLt | kLe) && matches!(op2, kLt | kLe))
            || (matches!(op1, kGt | kGe) && matches!(op2, kGt | kGe)))
        {
            return None;
        }
        let value2 = self
            .backend
            .ConsumeValue(&name, stream, set.SupportsElementDependent())?;
        self.backend.UseCountRangeSyntax();
        Some(Self::Feature(
            &name,
            MediaQueryExpComparison::new(&value1, op1),
            MediaQueryExpComparison::new(&value2, op2),
        ))
    }
    fn Feature(
        name: &AtomicString,
        left: MediaQueryExpComparison<B::Value>,
        right: MediaQueryExpComparison<B::Value>,
    ) -> Rc<ConditionalExpNode<B::Value, B::UnparsedValue>> {
        Rc::new(ConditionalExpNode::Feature(
            MediaQueryExp::CreateWithBounds(name, &MediaQueryExpBounds::new(&left, &right)),
        ))
    }
    // cpp: parser/media_query_parser.cc:600-628
    fn ConsumeQuery<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<MediaQuery<B::Value, B::UnparsedValue>> {
        let savepoint = stream.Save();
        let restrictor = Self::ConsumeRestrictor(stream);
        if let Some(media_type) = Self::ConsumeType(stream) {
            let text = StringFromAtom(&media_type);
            if !ConsumeIfIdent(stream, "and") {
                return Some(MediaQuery::new(restrictor, text, None));
            }
            let node = self.ConsumeCondition(stream, ParseMode::kWithoutOr)?;
            return Some(MediaQuery::new(restrictor, text, Some(node)));
        }
        stream.Restore(savepoint);
        self.ConsumeCondition(stream, ParseMode::kNormal)
            .map(|node| MediaQuery::new(RestrictorType::kNone, String::from("all"), Some(node)))
    }
    // cpp: parser/media_query_parser.cc:584-598,630-655
    fn ParseImpl<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
        condition_only: bool,
    ) -> MediaQuerySet<B::Value, B::UnparsedValue> {
        stream.ConsumeWhitespace();
        if stream.AtEnd() {
            return MediaQuerySet::Create();
        }
        if condition_only {
            let query = self
                .ConsumeCondition(stream, ParseMode::kNormal)
                .map(|node| MediaQuery::new(RestrictorType::kNone, String::from("all"), Some(node)))
                .unwrap_or_else(MediaQuery::CreateNotAll);
            return MediaQuerySet::FromQueries(vec![Rc::new(query)]);
        }
        let mut queries = Vec::new();
        loop {
            let query = self.ConsumeQuery(stream);
            let ok = query.is_some() && (stream.AtEnd() || stream.Peek().GetType() == kCommaToken);
            queries.push(Rc::new(if ok {
                query.unwrap()
            } else {
                MediaQuery::CreateNotAll()
            }));
            if stream.AtEnd() || !ConsumeUntilCommaInclusive(stream) {
                break;
            }
        }
        MediaQuerySet::FromQueries(queries)
    }
}
impl<B: MediaQueryParserBackend> ConditionalParser for MediaQueryParser<B> {
    type Value = B::Value;
    type UnparsedValue = B::UnparsedValue;
    // cpp: parser/media_query_parser.cc:566-582
    fn ConsumeLeaf<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<ConditionalExpNode<Self::Value, Self::UnparsedValue>>> {
        stream.ConsumeWhitespace();
        let feature = self.ConsumeFeatureWithSet(stream, &MediaFeatureSet);
        if feature.is_some() {
            stream.ConsumeWhitespace();
        }
        feature
    }
    fn ConsumeFunction<T: TokenStreamTokenizer>(
        &mut self,
        _stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<ConditionalExpNode<Self::Value, Self::UnparsedValue>>> {
        None
    }
}
impl<B: MediaQueryParserBackend> MediaQuerySetParser<B::Value, B::UnparsedValue>
    for MediaQueryParser<B>
{
    fn ParseMediaQuerySet(&mut self, text: &String) -> MediaQuerySet<B::Value, B::UnparsedValue> {
        self.ParseMediaQuerySet(StringView::from(text))
    }
}
fn AtomFromString(text: &String) -> AtomicString {
    if text.IsNull() {
        AtomicString::default()
    } else if let Some(bytes) = text.Span8() {
        AtomicString::from_latin1(bytes)
    } else {
        AtomicString::from_utf16(text.Span16().unwrap())
    }
}
fn StringFromAtom(atom: &AtomicString) -> String {
    if atom.IsNull() {
        String::default()
    } else if atom.Is8Bit() {
        String::from_latin1(atom.Span8())
    } else {
        String::from_utf16(atom.Span16())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kleene_value::KleeneValue;
    use crate::media_queries::media_query_evaluator::{
        MediaQueryEvaluationBackend, MediaQueryEvaluator,
    };
    use crate::media_queries::media_query_exp::MediaQueryExpSerialization;
    use foundation::CSSValueID;
    use std::cell::Cell;

    // A constrained CSSValue fixture, used only to isolate the source parser
    // state machine. Production callers must implement the real CSS consumer.
    struct FixtureValue(String);
    impl MediaQueryExpSerialization for FixtureValue {
        fn CssText(&self) -> String {
            self.0.clone()
        }
    }
    struct FixtureBackend {
        range_count: usize,
        eval_count: Cell<usize>,
    }
    impl FixtureBackend {
        fn new() -> Self {
            Self {
                range_count: 0,
                eval_count: Cell::new(0),
            }
        }
    }
    impl MediaQueryRuntimeFeatures for FixtureBackend {
        fn CSSCustomMediaEnabled(&self) -> bool {
            true
        }
        fn CSSScrolledContainerQueriesEnabled(&self) -> bool {
            true
        }
        fn PrefersReducedDataEnabled(&self) -> bool {
            true
        }
        fn ForcedColorsEnabled(&self) -> bool {
            true
        }
        fn MediaQueryNavigationControlsEnabled(&self) -> bool {
            true
        }
        fn OriginTrialsSampleAPIEnabled(&self) -> bool {
            true
        }
        fn ViewportSegmentsEnabled(&self) -> bool {
            true
        }
        fn DevicePostureEnabled(&self) -> bool {
            true
        }
        fn InvertedColorsEnabled(&self) -> bool {
            true
        }
        fn DesktopPWAsAdditionalWindowingControlsEnabled(&self) -> bool {
            true
        }
    }
    impl MediaQueryParserBackend for FixtureBackend {
        type Value = FixtureValue;
        type UnparsedValue = FixtureValue;
        fn ConsumeValue<T: TokenStreamTokenizer>(
            &mut self,
            feature: &AtomicString,
            stream: &mut CSSParserTokenStream<'_, T>,
            element_dependent: bool,
        ) -> Option<MediaQueryExpValue<FixtureValue>> {
            assert!(!element_dependent);
            if feature == "prefers-color-scheme"
                && matches!(stream.Peek().Id(), CSSValueID::kLight | CSSValueID::kDark)
            {
                return Some(MediaQueryExpValue::FromId(
                    stream.ConsumeIncludingWhitespace().Id(),
                ));
            }
            if feature != "width" || stream.Peek().GetType() != kDimensionToken {
                return None;
            }
            if stream.Peek().GetUnitType() != crate::css_primitive_value::UnitType::kPixels {
                return None;
            }
            let value = stream.ConsumeIncludingWhitespace().NumericValue();
            Some(MediaQueryExpValue::FromValue(Rc::new(FixtureValue(
                String::from(format!("{value}px").as_str()),
            ))))
        }
        fn UseCountRangeSyntax(&mut self) {
            self.range_count += 1;
        }
    }
    impl MediaQueryEvaluationBackend for FixtureBackend {
        type Value = FixtureValue;
        type UnparsedValue = FixtureValue;
        type ResultFlags = usize;
        type CustomMediaRulesMap = ();
        fn EvalFeature(
            &self,
            feature: &MediaQueryExp<FixtureValue>,
            flags: Option<&mut usize>,
            _: Option<&()>,
        ) -> KleeneValue {
            self.eval_count.set(self.eval_count.get() + 1);
            if let Some(flags) = flags {
                *flags += 1;
            }
            match feature.MediaFeature().Utf8().as_str() {
                "width" => KleeneValue::kFalse,
                "prefers-color-scheme" => KleeneValue::kTrue,
                "--fixture" => KleeneValue::kUnknown,
                _ => panic!("test leaf not specified"),
            }
        }
    }
    fn parse(text: &str) -> MediaQuerySet<FixtureValue> {
        MediaQueryParser::new(FixtureBackend::new()).ParseMediaQuerySet(StringView::from(text))
    }
    #[test]
    fn media_query_parser_lists_recover_at_top_level_commas() {
        for (input, expected) in [
            ("", ""),
            ("SCREEN, only print", "screen, only print"),
            ("screen and, print", "not all, print"),
            ("screen,", "screen, not all"),
            (",screen", "not all, screen"),
            ("layer, all", "not all, all"),
            ("(unknown: 1, 2), screen", "(unknown: 1, 2), screen"),
            (
                "screen and (width: 1px) or (width: 2px), print",
                "not all, print",
            ),
            ("only (width: 1px), print", "not all, print"),
            ("(width < = 1px), print", "(width < = 1px), print"),
        ] {
            assert_eq!(parse(input).MediaText().Utf8(), expected, "{input}");
        }
    }
    #[test]
    fn media_query_parser_range_bounds_and_unknown_fallback() {
        for (input, expected) in [
            ("(WIDTH<=10px)", "(width <= 10px)"),
            ("(10px < width <= 20px)", "(10px < width <= 20px)"),
            ("(20px >= width > 10px)", "(20px >= width > 10px)"),
            ("(width)", "(width)"),
            ("(--fixture)", "(--fixture)"),
            (
                "not (prefers-color-scheme:dark)",
                "not (prefers-color-scheme: dark)",
            ),
            ("(10px < width > 20px)", "(10px < width > 20px)"),
        ] {
            assert_eq!(parse(input).MediaText().Utf8(), expected, "{input}");
        }
        let range = parse("(10px < width <= 20px)");
        let mut expressions = Vec::new();
        range.QueryVector()[0].CollectExpressions(&mut expressions);
        assert_eq!(expressions.len(), 1);
        assert_eq!(expressions[0].Bounds().left.op, MediaQueryOperator::kLt);
        assert_eq!(expressions[0].Bounds().right.op, MediaQueryOperator::kLe);
        let mut expressions = Vec::new();
        parse("(10px < width > 20px)").QueryVector()[0].CollectExpressions(&mut expressions);
        assert!(expressions.is_empty()); // general-enclosed retains Unknown, never a false feature.
    }
    #[test]
    fn media_query_parser_tri_state_short_circuit_and_restrictors() {
        let evaluator =
            MediaQueryEvaluator::FromMediaType(String::from("screen"), FixtureBackend::new());
        for (input, expected) in [
            ("", true),
            ("screen", true),
            ("only print", false),
            ("not print", true),
            ("not all", false),
            ("not (unknown: x)", false),
            ("not (--fixture)", false),
            ("not (width)", true),
            ("(unknown: x) or (prefers-color-scheme: dark)", true),
            ("(width) and (prefers-color-scheme: dark)", false),
            ("print, (prefers-color-scheme: dark)", true),
        ] {
            assert_eq!(evaluator.Eval(&parse(input)), expected, "{input}");
        }
        let mut visits = 0;
        evaluator.EvalSet(
            &parse("(width) and (prefers-color-scheme: dark)"),
            Some(&mut visits),
            None,
        );
        assert_eq!(visits, 1);
        visits = 0;
        evaluator.EvalSet(
            &parse("(prefers-color-scheme: dark) or (width)"),
            Some(&mut visits),
            None,
        );
        assert_eq!(visits, 1);
        let mut expressions = Vec::new();
        parse("(width) and (prefers-color-scheme: dark)").QueryVector()[0]
            .CollectExpressions(&mut expressions);
        assert_eq!(expressions.len(), 2); // collector returns Unknown to prevent short-circuiting.
    }
    #[test]
    fn media_query_parser_custom_definition_boundary_and_set_copy_identity() {
        let mut parser = MediaQueryParser::new(FixtureBackend::new());
        let mut stream =
            CSSParserTokenStream::<CSSTokenizer>::new(StringView::from("screen; print"), 0);
        assert_eq!(
            parser
                .ParseCustomMediaDefinition(&mut stream)
                .MediaText()
                .Utf8(),
            "screen"
        );
        assert_eq!(stream.Peek().GetType(), kSemicolonToken);
        let queries = Rc::new(parse("screen, screen, print"));
        assert!(queries
            .CopyAndAdd(&String::from("SCREEN"), &mut parser)
            .is_none());
        let unchanged = queries
            .CopyAndRemove(&String::from("screen, print"), &mut parser)
            .unwrap();
        assert!(Rc::ptr_eq(&queries, &unchanged));
        assert_eq!(
            queries
                .CopyAndRemove(&String::from("SCREEN"), &mut parser)
                .unwrap()
                .MediaText()
                .Utf8(),
            "print"
        );
        assert!(queries
            .CopyAndRemove(&String::from("speech"), &mut parser)
            .is_none());
    }
}
