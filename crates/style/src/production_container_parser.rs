// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Concrete owners for container_query_parser and media_query_parser.cc:223-255,341-395.
#![allow(non_snake_case)]
use crate::{
    css_value::{CSSValuePayload, CSSValueSubclass},
    media_queries::{
        conditional_exp_node::ConditionalExpNode, media_query_backend::MediaQueryFeatureFlags,
        media_query_exp::*,
    },
    parser::{
        container_query_parser::ContainerQueryParserBackend,
        css_parser_context::CSSParserContext,
        css_parser_token::{BlockType, CSSParserTokenType::*},
        css_parser_token_stream::{BlockGuard, CSSParserTokenStream, TokenStreamTokenizer},
        media_query_parser::{
            MediaQueryParser, MediaQueryParserBackend, MediaQueryRuntimeFeatures,
        },
        production_property_parser::ValidateVariableTokensWithArgumentGrammar,
    },
    production_css_value as values,
    production_style_sheet::{Backend, Feature},
};
use foundation::{AtomicString, String};
use std::rc::Rc;
pub type ContainerSet =
    crate::container_query::ContainerQuerySet<values::Value, values::CSSUnparsedDeclarationValue>;
pub type ContainerCondition =
    ConditionalExpNode<values::Value, values::CSSUnparsedDeclarationValue>;
#[derive(Clone, Copy)]
pub struct ProductionContainerBackend<'a> {
    pub context: &'a CSSParserContext<Backend>,
    pub flags: MediaQueryFeatureFlags,
}
impl<'a> ProductionContainerBackend<'a> {
    pub fn new(context: &'a CSSParserContext<Backend>) -> Self {
        Self {
            context,
            flags: MediaQueryFeatureFlags {
                scrolled_container_queries: true,
                ..Default::default()
            },
        }
    }
    fn ConsumeUnparsed<T: TokenStreamTokenizer>(
        &self,
        s: &mut CSSParserTokenStream<'_, T>,
        allow_important: bool,
        comparison_ends: bool,
    ) -> Option<Rc<values::Value>> {
        let mut tokens = Vec::new();
        let mut annotation = None;
        CollectComponentTokens(s, comparison_ends, &mut tokens, Some(&mut annotation));
        if tokens.is_empty() {
            return None;
        }
        if let Some(index) = annotation {
            let suffix = tokens[index + 1..]
                .iter()
                .filter(|t| !matches!(t.token.GetType(), kWhitespaceToken | kCommentToken))
                .collect::<Vec<_>>();
            if !allow_important
                || suffix.len() != 1
                || suffix[0].token.GetType() != kIdentToken
                || !suffix[0]
                    .token
                    .Value()
                    .ToString()
                    .Utf8()
                    .eq_ignore_ascii_case("important")
            {
                return None;
            }
            tokens.truncate(index);
        }
        let first = tokens
            .iter()
            .position(|t| !matches!(t.token.GetType(), kWhitespaceToken | kCommentToken))
            .unwrap_or(tokens.len());
        let last = tokens
            .iter()
            .rposition(|t| !matches!(t.token.GetType(), kWhitespaceToken | kCommentToken))
            .map_or(first, |i| i + 1);
        let tokens = tokens[first..last].to_vec();
        if !ValidateVariableTokensWithArgumentGrammar(&tokens, false, true, false) {
            return None;
        }
        if allow_important && tokens.len() == 1 && tokens[0].token.GetType() == kIdentToken {
            if let Some(v) = values::wide(tokens[0].token.Id()) {
                return Some(v);
            }
        }
        Some(values::unparsed(
            values::CSSVariableData::FromTokens(tokens, false, false),
            crate::parser::css_parser_mode::CSSParserMode::kHTMLStandardMode,
        ))
    }
}
// CSSParserTokenStream owns block boundaries. Copy its tokens and exact lexemes;
// recursion enters BlockGuard instead of discovering blocks from source text.
pub(crate) fn CollectComponentTokens<T: TokenStreamTokenizer>(
    s: &mut CSSParserTokenStream<'_, T>,
    comparison_ends: bool,
    tokens: &mut Vec<values::VariableToken>,
    mut annotation: Option<&mut Option<usize>>,
) {
    while !s.AtEnd() {
        let start = s.Offset();
        let token = s.Peek().clone();
        if comparison_ends
            && token.GetType() == kDelimiterToken
            && matches!(token.Delimiter(), 61 | 60 | 62)
        {
            break;
        }
        if token.GetType() == kDelimiterToken && token.Delimiter() == 33 {
            if let Some(index) = annotation.as_deref_mut() {
                *index = Some(tokens.len());
            }
        }
        if token.GetBlockType() == BlockType::kBlockStart {
            let mut block = BlockGuard::new(s);
            tokens.push(values::VariableToken {
                token,
                text: block
                    .StringRangeAt(start, block.Offset() - start)
                    .ToString(),
            });
            CollectComponentTokens(&mut block, false, tokens, None);
            if block.Peek().GetBlockType() == BlockType::kBlockEnd {
                let end = block.Offset();
                let length = block.LookAheadOffset() + 1 - end;
                tokens.push(values::VariableToken {
                    token: block.Peek().clone(),
                    text: block.StringRangeAt(end, length).ToString(),
                });
            }
        } else {
            s.Consume();
            tokens.push(values::VariableToken {
                token,
                text: s.StringRangeAt(start, s.Offset() - start).ToString(),
            });
        }
    }
}

impl MediaQueryExpSerialization for values::Value {
    fn CssText(&self) -> String {
        self.CssText()
    }
}
impl MediaQueryExpSerialization for values::CSSUnparsedDeclarationValue {
    fn CssText(&self) -> String {
        self.CustomCSSText()
    }
}
impl MediaQueryRuntimeFeatures for ProductionContainerBackend<'_> {
    fn CSSCustomMediaEnabled(&self) -> bool {
        self.flags.CSSCustomMediaEnabled()
    }
    fn CSSScrolledContainerQueriesEnabled(&self) -> bool {
        self.flags.CSSScrolledContainerQueriesEnabled()
    }
    fn PrefersReducedDataEnabled(&self) -> bool {
        self.flags.PrefersReducedDataEnabled()
    }
    fn ForcedColorsEnabled(&self) -> bool {
        self.flags.ForcedColorsEnabled()
    }
    fn MediaQueryNavigationControlsEnabled(&self) -> bool {
        self.flags.MediaQueryNavigationControlsEnabled()
    }
    fn OriginTrialsSampleAPIEnabled(&self) -> bool {
        self.flags.OriginTrialsSampleAPIEnabled()
    }
    fn ViewportSegmentsEnabled(&self) -> bool {
        self.flags.ViewportSegmentsEnabled()
    }
    fn DevicePostureEnabled(&self) -> bool {
        self.flags.DevicePostureEnabled()
    }
    fn InvertedColorsEnabled(&self) -> bool {
        self.flags.InvertedColorsEnabled()
    }
    fn DesktopPWAsAdditionalWindowingControlsEnabled(&self) -> bool {
        self.flags.DesktopPWAsAdditionalWindowingControlsEnabled()
    }
}
impl MediaQueryParserBackend for ProductionContainerBackend<'_> {
    type Value = values::Value;
    type UnparsedValue = values::CSSUnparsedDeclarationValue;
    fn ConsumeValue<T: TokenStreamTokenizer>(
        &mut self,
        feature: &AtomicString,
        s: &mut CSSParserTokenStream<'_, T>,
        element: bool,
    ) -> Option<MediaQueryExpValue<Self::Value>> {
        if feature.length() >= 3 && feature.at(0) == 45 && feature.at(1) == 45 {
            return self
                .ConsumeUnparsed(s, true, false)
                .map(MediaQueryExpValue::FromValue);
        }
        let value = self.flags.ConsumeValue(feature, s, element)?;
        Some(match value {
            MediaQueryExpValue::Invalid => MediaQueryExpValue::Invalid,
            MediaQueryExpValue::Id(id) => MediaQueryExpValue::Id(id),
            MediaQueryExpValue::Value(v) => {
                MediaQueryExpValue::Value(values::numeric(v.DoubleValue(), v.GetType()))
            }
            MediaQueryExpValue::Ratio(v) => MediaQueryExpValue::FromRatio(
                values::numeric(v.0.DoubleValue(), v.0.GetType()),
                values::numeric(v.1.DoubleValue(), v.1.GetType()),
            ),
        })
    }
    fn UseCountRangeSyntax(&mut self) {
        self.context.CountWebFeature(Feature::ContainerRangeSyntax);
    }
}
impl ContainerQueryParserBackend for ProductionContainerBackend<'_> {
    fn ContainerNameOnlyEnabled(&self) -> bool {
        true
    }
    fn CommaSeparatedContainerQueriesEnabled(&self) -> bool {
        true
    }
    fn CountStyleContainerQuery(&self) {
        self.context.CountWebFeature(Feature::ContainerStyleQuery);
    }
    fn ConsumeStyleFeatureRange<T: TokenStreamTokenizer>(
        &mut self,
        s: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<ContainerCondition>> {
        let start = s.Save();
        let result = (|| {
            let v1 = self.ConsumeUnparsed(s, false, true)?;
            if s.AtEnd() {
                return None;
            }
            let op1 = MediaQueryParser::<Self>::ConsumeComparison(s);
            if op1 == MediaQueryOperator::kNone {
                return None;
            }
            let v2 = self.ConsumeUnparsed(s, false, true)?;
            let reference = match v2.Payload() {
                CSSValuePayload::kUnparsedDeclarationClass(v) => Rc::new(v.clone()),
                _ => return None,
            };
            let left = MediaQueryExpComparison::new(&MediaQueryExpValue::FromValue(v1), op1);
            let right = if s.AtEnd() {
                MediaQueryExpComparison::default()
            } else {
                let op2 = MediaQueryParser::<Self>::ConsumeComparison(s);
                if op2 == MediaQueryOperator::kNone || ((op2 as i32) - (op1 as i32)).abs() > 1 {
                    return None;
                }
                let v3 = self.ConsumeUnparsed(s, false, true)?;
                if !s.AtEnd() {
                    return None;
                }
                MediaQueryExpComparison::new(&MediaQueryExpValue::FromValue(v3), op2)
            };
            Some(Rc::new(ConditionalExpNode::Feature(
                MediaQueryExp::CreateStyleRange(
                    reference,
                    &MediaQueryExpBounds::new(&left, &right),
                    true,
                )?,
            )))
        })();
        if result.is_none() {
            s.Restore(start);
        }
        result
    }
}
