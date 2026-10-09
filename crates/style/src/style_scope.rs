// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// cpp: style_scope.h:36-84/.cc:19-20,52-154. Copy/Clone (22-50) remain pending
// until production StyleRuleCloneDependencies preserves complete rule trees.
#![allow(non_snake_case)]
use crate::{
    css_property_value_set::{CSSPropertyValueSetRuleHandle, ImmutableCSSPropertyValueSet},
    css_selector_list::CSSSelectorList,
    parser::{
        css_nesting_type::CSSNestingType,
        css_parser_impl::CSSParserImpl,
        css_parser_mode::CSSParserMode,
        css_parser_token::CSSParserTokenType::*,
        css_parser_token_stream::{
            CSSParserTokenStream, RestoringBlockGuard, TokenStreamTokenizer,
        },
        css_selector_parser::{CSSSelectorParser, SelectorParserContext, SelectorParserOptions},
    },
    production_css_value::ProductionCSSValueDispatch,
    production_style_sheet::Backend,
    style_rule::StyleRule,
};
use std::rc::Rc;
pub struct StyleScope {
    from_: Option<Rc<StyleRule<Backend>>>,
    to_: Option<Rc<CSSSelectorList>>,
    parent_: Option<Rc<Self>>,
}
impl StyleScope {
    pub fn new(from: Option<Rc<StyleRule<Backend>>>, to: Option<Rc<CSSSelectorList>>) -> Self {
        Self {
            from_: from,
            to_: to,
            parent_: None,
        }
    }
    pub fn CreateImplicit() -> Rc<Self> {
        Rc::new(Self::new(None, None))
    }
    pub fn IsImplicit(&self) -> bool {
        self.from_.is_none()
    }
    pub fn From(&self) -> Option<&CSSSelectorList> {
        self.from_.as_ref().map(|rule| rule.Selectors())
    }
    pub fn To(&self) -> Option<&CSSSelectorList> {
        self.to_.as_deref()
    }
    pub fn FromRule(&self) -> Option<Rc<StyleRule<Backend>>> {
        self.from_.clone()
    }
    pub fn ToList(&self) -> Option<Rc<CSSSelectorList>> {
        self.to_.clone()
    }
    pub fn Parent(&self) -> Option<&Self> {
        self.parent_.as_deref()
    }
    // CSSScopeifiedParentPseudoClass is stable in the mapped source commit.
    pub fn RuleForNesting(&self) -> Option<Rc<StyleRule<Backend>>> {
        None
    }
    pub fn Consume<T: TokenStreamTokenizer>(
        parser: &mut CSSParserImpl<'_, Backend>,
        s: &mut CSSParserTokenStream<'_, T>,
        nesting: CSSNestingType,
        parent: Option<Rc<StyleRule<Backend>>>,
    ) -> Option<Rc<Self>> {
        let mut from = None;
        let mut to = None;
        if s.Peek().GetType() == kLeftParenthesisToken {
            if let Some(list) = ConsumeEnclosedBoundary(parser, s, nesting, parent) {
                let properties =
                    CSSPropertyValueSetRuleHandle::FromImmutable(ImmutableCSSPropertyValueSet::<
                        ProductionCSSValueDispatch,
                    >::Create(
                        &[],
                        CSSParserMode::kHTMLStandardMode,
                        false,
                    ));
                from = Some(Rc::new(StyleRule::Create(
                    (*list).clone(),
                    properties,
                    None,
                )));
            }
        }
        s.EnsureLookAhead();
        let save = s.Save();
        if s.Peek().GetType() == kIdentToken
            && s.Peek()
                .Value()
                .ToString()
                .Utf8()
                .eq_ignore_ascii_case("to")
        {
            s.ConsumeIncludingWhitespace();
            if s.Peek().GetType() == kLeftParenthesisToken {
                to = ConsumeEnclosedBoundary(parser, s, CSSNestingType::kScope, None);
            }
            if to.is_none() {
                s.Restore(save);
            }
        }
        if from.is_none() && to.is_none() {
            None
        } else {
            Some(Rc::new(Self::new(from, to)))
        }
    }
}
fn ConsumeEnclosedBoundary<T: TokenStreamTokenizer>(
    parser: &mut CSSParserImpl<'_, Backend>,
    s: &mut CSSParserTokenStream<'_, T>,
    nesting: CSSNestingType,
    parent: Option<Rc<StyleRule<Backend>>>,
) -> Option<Rc<CSSSelectorList>> {
    let mut block = RestoringBlockGuard::new(s);
    block.ConsumeWhitespace();
    let context = SelectorParserContext {
        html: true,
        quirks: parser.GetMode() == CSSParserMode::kHTMLQuirksMode,
    };
    let options = SelectorParserOptions {
        namespace_context: parser.style_sheet_.as_ref().map(|sheet| {
            sheet.as_ref() as &dyn crate::parser::css_selector_parser::CSSSelectorNamespaceContext
        }),
        parent_rule_for_nesting: parent
            .map(|p| p as Rc<dyn crate::css_selector::CSSSelectorParentRule>),
        ..Default::default()
    };
    let mut arena = Vec::new();
    let start = block.LookAheadOffset();
    let selectors = CSSSelectorParser::ParseScopeBoundaryStream(
        &mut block, &context, nesting, &options, &mut arena,
    )
    .to_vec();
    if selectors.is_empty() || !block.Release() {
        return None;
    }
    let end = block.Offset().saturating_sub(1);
    if let Some(observer) = parser.observer_.as_deref_mut() {
        crate::parser::css_selector_parser::CSSSelectorParserObserver::ObserveSelector(
            observer, start, end,
        );
    }
    block.ConsumeWhitespace();
    Some(CSSSelectorList::AdoptSelectorVector(selectors))
}
impl crate::rule_set::RuleSetStyleScope for StyleScope {
    fn From(&self) -> Option<&CSSSelectorList> {
        StyleScope::From(self)
    }
    fn To(&self) -> Option<&CSSSelectorList> {
        StyleScope::To(self)
    }
    fn Parent(&self) -> Option<&Self> {
        StyleScope::Parent(self)
    }
}
