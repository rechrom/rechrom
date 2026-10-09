// Copyright 2025 The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Source commit: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
// Ledger uses C++ physical / effective / mapped / omitted / pending lines.
// Effective excludes comments, blanks, preprocessor/includes, namespaces and
// lines consisting only of braces/parentheses/semicolons.
// parser/conditional_parser.h: 60 / 15 / 10 / 5 / 0.
// parser/conditional_parser.cc: 100 / 55 / 52 / 3 / 0.
// Header omitted: 13-14,21-23,36 (forward/stack/access).
// Implementation omitted: 13-15 (C++ using declarations).
// Source ledger (third_party/blink/renderer/core/css/):
// - parser/conditional_parser.h:25-67; .cc:17-98: complete parser runtime.
// - properties/css_parsing_utils.h:1042-1064: identifier/delimiter consumers.
// - properties/css_parsing_utils.cc:922-936,1063-1087: any-value consumers.
// Omitted: includes/STACK_ALLOCATED; no runtime branches omitted.
// Required leaf/function parsing is the source's abstract-method boundary.
#![allow(non_snake_case, non_camel_case_types)]
use super::css_parser_token::{BlockType, CSSParserToken, CSSParserTokenType::*};
use super::css_parser_token_stream::{
    BlockGuard, CSSParserTokenStream, RestoringBlockGuard, TokenStreamTokenizer,
};
use crate::media_queries::conditional_exp_node::ConditionalExpNode;
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ParseMode {
    kNormal,
    kWithoutOr,
}

pub(crate) fn AtIdent(token: &CSSParserToken, ident: &str) -> bool {
    token.GetType() == kIdentToken && token.Value().ToString().Utf8().eq_ignore_ascii_case(ident)
}
pub(crate) fn ConsumeIfIdent<T: TokenStreamTokenizer>(
    stream: &mut CSSParserTokenStream<'_, T>,
    ident: &str,
) -> bool {
    if !AtIdent(stream.Peek(), ident) {
        return false;
    }
    stream.ConsumeIncludingWhitespace();
    true
}
pub(crate) fn ConsumeIfDelimiter<T: TokenStreamTokenizer>(
    stream: &mut CSSParserTokenStream<'_, T>,
    c: u16,
) -> bool {
    if stream.Peek().GetType() != kDelimiterToken || stream.Peek().Delimiter() != c {
        return false;
    }
    stream.ConsumeIncludingWhitespace();
    true
}
fn IsTokenAllowedForAnyValue(token: &CSSParserToken) -> bool {
    match token.GetType() {
        kBadStringToken | kEOFToken | kBadUrlToken => false,
        kRightParenthesisToken | kRightBracketToken | kRightBraceToken => {
            token.GetBlockType() == BlockType::kBlockEnd
        }
        _ => true,
    }
}
pub(crate) fn ConsumeAnyValue<T: TokenStreamTokenizer>(stream: &mut CSSParserTokenStream<'_, T>) {
    while !stream.AtEnd() {
        if stream.Peek().GetBlockType() == BlockType::kBlockStart {
            let mut guard = RestoringBlockGuard::new(stream);
            ConsumeAnyValue(&mut guard);
            if !guard.Release() {
                return;
            }
        } else if IsTokenAllowedForAnyValue(stream.Peek()) {
            stream.Consume();
        } else {
            return;
        }
    }
}

pub trait ConditionalParser {
    type Value;
    type UnparsedValue;
    fn ConsumeLeaf<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<ConditionalExpNode<Self::Value, Self::UnparsedValue>>>;
    fn ConsumeFunction<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<ConditionalExpNode<Self::Value, Self::UnparsedValue>>>;

    // cpp: parser/conditional_parser.cc:17-38
    fn ConsumeCondition<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
        mode: ParseMode,
    ) -> Option<Rc<ConditionalExpNode<Self::Value, Self::UnparsedValue>>> {
        if ConsumeIfIdent(stream, "not") {
            return ConditionalExpNode::Not(self.ConsumeInner(stream));
        }
        let mut result = self.ConsumeInner(stream);
        if AtIdent(stream.Peek(), "and") {
            while result.is_some() && ConsumeIfIdent(stream, "and") {
                result = ConditionalExpNode::And(result, self.ConsumeInner(stream));
            }
        } else if result.is_some() && AtIdent(stream.Peek(), "or") && mode != ParseMode::kWithoutOr
        {
            while result.is_some() && ConsumeIfIdent(stream, "or") {
                result = ConditionalExpNode::Or(result, self.ConsumeInner(stream));
            }
        }
        result
    }
    // cpp: parser/conditional_parser.cc:40-65
    fn ConsumeInner<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<ConditionalExpNode<Self::Value, Self::UnparsedValue>>> {
        if stream.Peek().GetType() == kLeftParenthesisToken {
            {
                let mut guard = RestoringBlockGuard::new(stream);
                guard.ConsumeWhitespace();
                let savepoint = guard.Save();
                let mut node = self.ConsumeLeaf(&mut guard);
                if node.is_none() {
                    guard.Restore(savepoint);
                    node = self.ConsumeCondition(&mut guard, ParseMode::kNormal);
                }
                if node.is_some() && guard.Release() {
                    drop(guard);
                    stream.ConsumeWhitespace();
                    return ConditionalExpNode::Nested(node);
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
    // cpp: parser/conditional_parser.cc:67-98
    fn ConsumeGeneralEnclosed<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<ConditionalExpNode<Self::Value, Self::UnparsedValue>>> {
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
        Some(Rc::new(ConditionalExpNode::Unknown(text)))
    }
}
