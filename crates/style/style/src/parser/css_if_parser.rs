// Copyright 2025 The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Source-aligned parser for CSS Values 5 `if()` conditions.
//!
//! Maps `css_if_parser.{h,cc}` and delegates its three stable leaf grammars to
//! the existing supports, media-query and container-query parsers.
#![allow(non_snake_case)]

use super::{
    conditional_parser::{AtIdent, ConditionalParser, ConsumeAnyValue, ConsumeIfIdent, ParseMode},
    css_parser_context::CSSParserContext,
    css_parser_impl::{CSSParserImpl, SupportsResult},
    css_parser_token::{BlockType, CSSParserTokenType::*},
    css_parser_token_stream::{
        BlockGuard, CSSParserTokenStream, RestoringBlockGuard, TokenStreamTokenizer,
    },
    css_supports_parser::CSSSupportsParser,
    media_query_parser::MediaQueryParser,
};
use crate::{
    if_condition::IfCondition, media_queries::media_query_backend::MediaQueryFeatureFlags,
    parser::container_query_parser::ContainerQueryParser,
    production_container_parser::ProductionContainerBackend, production_style_sheet::Backend,
};
use std::rc::Rc;

pub struct CSSIfParser<'a> {
    context: &'a CSSParserContext<Backend>,
}

impl<'a> CSSIfParser<'a> {
    pub fn new(context: &'a CSSParserContext<Backend>) -> Self {
        Self { context }
    }

    pub fn ConsumeIfCondition<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<IfCondition>> {
        if AtIdent(stream.Peek(), "else") {
            stream.ConsumeIncludingWhitespace();
            return Some(Rc::new(IfCondition::Else));
        }
        self.ConsumeCondition(stream, ParseMode::kNormal)
    }

    fn ConsumeCondition<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
        mode: ParseMode,
    ) -> Option<Rc<IfCondition>> {
        if ConsumeIfIdent(stream, "not") {
            return self
                .ConsumeInner(stream)
                .map(|node| Rc::new(IfCondition::Not(node)));
        }
        let mut result = self.ConsumeInner(stream);
        if AtIdent(stream.Peek(), "and") {
            while result.is_some() && ConsumeIfIdent(stream, "and") {
                result = Some(Rc::new(IfCondition::And(
                    result?,
                    self.ConsumeInner(stream)?,
                )));
            }
        } else if result.is_some() && AtIdent(stream.Peek(), "or") && mode != ParseMode::kWithoutOr
        {
            while result.is_some() && ConsumeIfIdent(stream, "or") {
                result = Some(Rc::new(IfCondition::Or(
                    result?,
                    self.ConsumeInner(stream)?,
                )));
            }
        }
        result
    }

    fn ConsumeInner<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<IfCondition>> {
        if stream.Peek().GetType() == kLeftParenthesisToken {
            let mut guard = RestoringBlockGuard::new(stream);
            guard.ConsumeWhitespace();
            let node = self.ConsumeCondition(&mut guard, ParseMode::kNormal);
            if let Some(node) = node {
                if guard.Release() {
                    drop(guard);
                    stream.ConsumeWhitespace();
                    return Some(Rc::new(IfCondition::Nested(node)));
                }
            }
        } else if stream.Peek().GetType() == kFunctionToken {
            let savepoint = stream.Save();
            if let Some(function) = self.ConsumeFunction(stream) {
                stream.ConsumeWhitespace();
                return Some(function);
            }
            stream.Restore(savepoint);
        }
        self.ConsumeGeneralEnclosed(stream)
    }

    fn ConsumeFunction<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<IfCondition>> {
        let id = stream.Peek().FunctionId();
        if id == Some(foundation::CSSValueID::kSupports) {
            {
                let mut guard = RestoringBlockGuard::new(stream);
                guard.ConsumeWhitespace();
                let mut parser = CSSParserImpl::<Backend>::new(Some(self.context), None);
                let result = CSSSupportsParser::ConsumeSupportsCondition(&mut guard, &mut parser);
                if result != SupportsResult::kParseFailure && guard.Release() {
                    return Some(Rc::new(IfCondition::Supports(
                        result == SupportsResult::kSupported,
                    )));
                }
            }
            {
                let mut guard = RestoringBlockGuard::new(stream);
                guard.ConsumeWhitespace();
                let mut parser = CSSParserImpl::<Backend>::new(Some(self.context), None);
                if guard.Peek().GetType() == kIdentToken
                    && parser.ConsumeSupportsDeclaration(&mut guard)
                    && guard.Release()
                {
                    return Some(Rc::new(IfCondition::Supports(true)));
                }
            }
            return None;
        }
        if id == Some(foundation::CSSValueID::kMedia) {
            {
                let mut guard = RestoringBlockGuard::new(stream);
                guard.ConsumeWhitespace();
                let mut parser = MediaQueryParser::new(MediaQueryFeatureFlags::default());
                let condition = ConditionalParser::ConsumeCondition(
                    &mut parser,
                    &mut guard,
                    ParseMode::kNormal,
                );
                if let Some(condition) = condition {
                    if guard.Release() {
                        return Some(Rc::new(IfCondition::MediaFromCondition(condition)));
                    }
                }
            }
            // `ConsumeFeature` does not restore its stream, so Chromium first
            // tries a media condition and then retries the direct feature
            // grammar from the function boundary.
            {
                let mut guard = RestoringBlockGuard::new(stream);
                guard.ConsumeWhitespace();
                let mut parser = MediaQueryParser::new(MediaQueryFeatureFlags::default());
                let condition = ConditionalParser::ConsumeLeaf(&mut parser, &mut guard);
                if let Some(condition) = condition {
                    if guard.Release() {
                        return Some(Rc::new(IfCondition::MediaFromCondition(condition)));
                    }
                }
            }
            return None;
        }
        if id == Some(foundation::CSSValueID::kStyle) {
            let mut guard = RestoringBlockGuard::new(stream);
            guard.ConsumeWhitespace();
            let mut parser =
                ContainerQueryParser::new(ProductionContainerBackend::new(self.context));
            let condition = parser.ConsumeStyleFeatureQuery(&mut guard);
            if let Some(condition) = condition {
                if guard.Release() {
                    return Some(Rc::new(IfCondition::Style(condition)));
                }
            }
            return None;
        }
        None
    }

    fn ConsumeGeneralEnclosed<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<IfCondition>> {
        if !matches!(
            stream.Peek().GetType(),
            kLeftParenthesisToken | kFunctionToken
        ) {
            return None;
        }
        let start = stream.Offset();
        {
            let mut guard = BlockGuard::new(stream);
            guard.ConsumeWhitespace();
            ConsumeAnyValue(&mut guard);
            if !guard.AtEnd() {
                return None;
            }
        }
        let end = stream.Offset();
        let text = stream.StringRangeAt(start, end - start).ToString();
        stream.ConsumeWhitespace();
        Some(Rc::new(IfCondition::Unknown(text)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{
        css_parser_context::{SecureContextMode, StrictCSSParserContext},
        css_tokenizer::CSSTokenizer,
    };
    use foundation::{String, StringView};

    fn parse(source: &str) -> Rc<IfCondition> {
        let context = StrictCSSParserContext::<Backend>(SecureContextMode::kInsecureContext);
        let text = String::from(source);
        let mut stream = CSSParserTokenStream::<CSSTokenizer>::new(StringView::from(&text), 0);
        let result = CSSIfParser::new(&context)
            .ConsumeIfCondition(&mut stream)
            .unwrap_or_else(|| panic!("if condition must parse: {source}"));
        assert!(stream.AtEnd(), "if condition must consume: {source}");
        result
    }

    #[test]
    fn retains_typed_supports_media_and_style_owners() {
        assert!(matches!(
            *parse("supports(display:block)"),
            IfCondition::Supports(true)
        ));
        assert!(matches!(
            *parse("supports(font-tech(unknown))"),
            IfCondition::Supports(false)
        ));
        assert!(matches!(
            *parse("media(width > 500px)"),
            IfCondition::Media(_)
        ));
        assert!(matches!(
            *parse("style(--theme: dark)"),
            IfCondition::Style(_)
        ));
        assert!(matches!(*parse("else"), IfCondition::Else));
    }

    #[test]
    fn preserves_boolean_ast_and_general_enclosed_unknown() {
        assert!(matches!(
            *parse("(not supports(display:none)) or media(width > 500px)"),
            IfCondition::Or(_, _)
        ));
        assert!(matches!(
            *parse("future-test(anything)"),
            IfCondition::Unknown(_)
        ));
    }
}
