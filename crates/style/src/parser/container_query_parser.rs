// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// cpp: container_query_parser.h:26-81; .cc:24-314.
// Pending .cc:277-278 ContainerSelector feature-requirement assembly.
// Reuses ConditionalParser and MediaQueryParser::ConsumeFeatureWithSet.
#![allow(non_snake_case)]
use super::{
    conditional_parser::{ConditionalParser, ConsumeIfIdent, ParseMode},
    css_parser_token::CSSParserTokenType::*,
    css_parser_token_stream::{CSSParserTokenStream, RestoringBlockGuard, TokenStreamTokenizer},
    css_tokenizer::CSSTokenizer,
    media_query_parser::{MediaQueryFeatureSet, MediaQueryParser, MediaQueryParserBackend},
};
use crate::{
    container_query::{ContainerQuery, ContainerQuerySet},
    media_queries::conditional_exp_node::ConditionalExpNode,
};
use foundation::{AtomicString, CSSValueID, String, StringView};
use std::rc::Rc;
pub trait ContainerQueryParserBackend: MediaQueryParserBackend + Clone {
    fn ContainerNameOnlyEnabled(&self) -> bool;
    fn CommaSeparatedContainerQueriesEnabled(&self) -> bool;
    fn CountStyleContainerQuery(&self);
    fn ConsumeStyleFeatureRange<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<ConditionalExpNode<Self::Value, Self::UnparsedValue>>>;
}
#[derive(Clone, Copy)]
enum FeatureSet {
    Size,
    Style,
    State,
    Anchored,
}
impl<B: ContainerQueryParserBackend> MediaQueryFeatureSet<B> for FeatureSet {
    fn IsAllowed(&self, name: &AtomicString, b: &B) -> bool {
        match self {
            Self::Size => matches!(
                name.Utf8().as_str(),
                "width"
                    | "min-width"
                    | "max-width"
                    | "height"
                    | "min-height"
                    | "max-height"
                    | "inline-size"
                    | "min-inline-size"
                    | "max-inline-size"
                    | "block-size"
                    | "min-block-size"
                    | "max-block-size"
                    | "aspect-ratio"
                    | "min-aspect-ratio"
                    | "max-aspect-ratio"
                    | "orientation"
            ),
            Self::Style => name.length() >= 3 && name.at(0) == 45 && name.at(1) == 45,
            Self::State => {
                matches!(name.Utf8().as_str(), "stuck" | "snapped" | "scrollable")
                    || name == "scrolled" && b.CSSScrolledContainerQueriesEnabled()
            }
            Self::Anchored => name == "fallback",
        }
    }
    fn IsAllowedWithoutValue(&self, name: &AtomicString, _: &B) -> bool {
        match self {
            Self::Size => matches!(
                name.Utf8().as_str(),
                "width" | "height" | "inline-size" | "block-size" | "aspect-ratio" | "orientation"
            ),
            _ => true,
        }
    }
    fn IsAllowedWithValue(&self, _: &AtomicString, _: &B) -> bool {
        true
    }
    fn IsRangeTypeFeature(&self, name: &AtomicString) -> bool {
        matches!(self, Self::Size)
            && matches!(
                name.Utf8().as_str(),
                "width" | "height" | "inline-size" | "block-size" | "aspect-ratio" | "orientation"
            )
    }
    fn IsCaseSensitive(&self) -> bool {
        matches!(self, Self::Style)
    }
    fn SupportsElementDependent(&self) -> bool {
        true
    }
    fn SupportsStyleRange(&self) -> bool {
        matches!(self, Self::Style)
    }
    fn ConsumeStyleRange<T: TokenStreamTokenizer>(
        &self,
        b: &mut B,
        s: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<ConditionalExpNode<B::Value, B::UnparsedValue>>> {
        b.ConsumeStyleFeatureRange(s)
    }
}
pub struct ContainerQueryParser<B: ContainerQueryParserBackend> {
    backend: B,
}
impl<B: ContainerQueryParserBackend> ContainerQueryParser<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }
    pub fn ParseCondition<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<ConditionalExpNode<B::Value, B::UnparsedValue>>> {
        stream.ConsumeWhitespace();
        self.ConsumeCondition(stream, ParseMode::kNormal)
    }
    pub fn ParseConditionString(
        &mut self,
        text: &String,
    ) -> Option<Rc<ConditionalExpNode<B::Value, B::UnparsedValue>>> {
        let mut stream = CSSParserTokenStream::<CSSTokenizer>::new(StringView::from(text), 0);
        let result = self.ParseCondition(&mut stream);
        stream.AtEnd().then_some(result).flatten()
    }
    fn ConsumeFeature<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
        set: FeatureSet,
    ) -> Option<Rc<ConditionalExpNode<B::Value, B::UnparsedValue>>> {
        MediaQueryParser::new(self.backend.clone()).ConsumeFeatureWithSet(stream, &set)
    }
    fn ConsumeFeatureQuery<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
        set: FeatureSet,
    ) -> Option<Rc<ConditionalExpNode<B::Value, B::UnparsedValue>>> {
        stream.EnsureLookAhead();
        let savepoint = stream.Save();
        if let Some(feature) = self.ConsumeFeature(stream, set) {
            return Some(feature);
        }
        stream.Restore(savepoint);
        ExpressionParser { parent: self, set }.ConsumeCondition(stream, ParseMode::kNormal)
    }
    fn ConsumeContainerQuery<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<ContainerQuery<B::Value, B::UnparsedValue>>> {
        let mut name = AtomicString::default();
        if stream.Peek().GetType() == kIdentToken {
            let id = stream.Peek().Id();
            let value = stream.Peek().Value().ToString().Utf8().to_ascii_lowercase();
            if !matches!(
                id,
                CSSValueID::kNone
                    | CSSValueID::kInitial
                    | CSSValueID::kInherit
                    | CSSValueID::kUnset
                    | CSSValueID::kRevert
                    | CSSValueID::kRevertLayer
                    | CSSValueID::kDefault
            ) && !matches!(value.as_str(), "not" | "and" | "or")
            {
                name =
                    AtomicString::from_utf16(stream.ConsumeIncludingWhitespace().Value().Span16());
            }
        }
        let query = self.ParseCondition(stream);
        if query.is_some() || !name.IsNull() && self.backend.ContainerNameOnlyEnabled() {
            Some(Rc::new(ContainerQuery::new(name, query)))
        } else {
            None
        }
    }
    pub fn ParseContainerQuerySet<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<ContainerQuerySet<B::Value, B::UnparsedValue>>> {
        stream.ConsumeWhitespace();
        let mut queries = Vec::new();
        loop {
            queries.push(self.ConsumeContainerQuery(stream)?);
            stream.ConsumeWhitespace();
            if stream.AtEnd()
                || !self.backend.CommaSeparatedContainerQueriesEnabled()
                || stream.Peek().GetType() != kCommaToken
            {
                break;
            }
            stream.ConsumeIncludingWhitespace();
        }
        Some(Rc::new(ContainerQuerySet::new(queries)))
    }
    pub fn ParseContainerQuerySetString(
        &mut self,
        text: &String,
    ) -> Option<Rc<ContainerQuerySet<B::Value, B::UnparsedValue>>> {
        let mut stream = CSSParserTokenStream::<CSSTokenizer>::new(StringView::from(text), 0);
        let result = self.ParseContainerQuerySet(&mut stream);
        stream.AtEnd().then_some(result).flatten()
    }
}
impl<B: ContainerQueryParserBackend> ConditionalParser for ContainerQueryParser<B> {
    type Value = B::Value;
    type UnparsedValue = B::UnparsedValue;
    fn ConsumeLeaf<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<ConditionalExpNode<B::Value, B::UnparsedValue>>> {
        let savepoint = stream.Save();
        stream.ConsumeWhitespace();
        let result = self.ConsumeFeature(stream, FeatureSet::Size);
        if result.is_some() && stream.AtEnd() {
            stream.ConsumeWhitespace();
            result
        } else {
            stream.Restore(savepoint);
            None
        }
    }
    fn ConsumeFunction<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<ConditionalExpNode<B::Value, B::UnparsedValue>>> {
        let (set, name) = match stream.Peek().FunctionId() {
            Some(CSSValueID::kStyle) => (FeatureSet::Style, "style"),
            Some(CSSValueID::kScrollState) => (FeatureSet::State, "scroll-state"),
            Some(CSSValueID::kAnchored) => (FeatureSet::Anchored, "anchored"),
            _ => return None,
        };
        let mut guard = RestoringBlockGuard::new(stream);
        guard.ConsumeWhitespace();
        let query = self.ConsumeFeatureQuery(&mut guard, set)?;
        if !guard.Release() {
            return None;
        }
        if matches!(set, FeatureSet::Style) {
            self.backend.CountStyleContainerQuery();
        }
        ConditionalExpNode::Function(Some(query), &AtomicString::from_str(name))
    }
}
struct ExpressionParser<'a, B: ContainerQueryParserBackend> {
    parent: &'a mut ContainerQueryParser<B>,
    set: FeatureSet,
}
impl<B: ContainerQueryParserBackend> ConditionalParser for ExpressionParser<'_, B> {
    type Value = B::Value;
    type UnparsedValue = B::UnparsedValue;
    fn ConsumeLeaf<T: TokenStreamTokenizer>(
        &mut self,
        s: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<ConditionalExpNode<B::Value, B::UnparsedValue>>> {
        self.parent.ConsumeFeature(s, self.set)
    }
    fn ConsumeFunction<T: TokenStreamTokenizer>(
        &mut self,
        _: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<ConditionalExpNode<B::Value, B::UnparsedValue>>> {
        None
    }
}
