// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//!
//! Typed production activation context for retained RuleSet entries.
//!
//! RuleSet already owns the source-translated scope and container interval
//! metadata.  This module keeps those owners attached to one RuleData entry
//! and supplies the small, explicit runtime inputs that ElementRuleCollector
//! normally installs before matching it.
#![allow(non_snake_case)]

use crate::{
    container_query::ContainerQuery,
    css_selector::{GetPseudoId, MatchType, PseudoType, RelationType},
    element_rule_collector::SearchTextRequest,
    kleene_value::{KleeneOr, KleeneValue},
    persistent_selector::{PersistentSelectorMatchSession, ScopedSelectorMatch},
    production_css_value::{CSSUnparsedDeclarationValue, Value},
    production_style_sheet::{Backend, ContainerSet},
    rule_set::{RuleData, RuleSet},
    style_rule::StyleRule,
    style_scope::StyleScope,
};
use foundation::AtomicString;
use layoutng_style::style::computed_style_constants::PseudoId;
use std::rc::Rc;

pub(crate) type ProductionContainerQuery = ContainerQuery<Value, CSSUnparsedDeclarationValue>;

/// Whether the translated selector runtime can activate and match every scope
/// in this rule's nested scope chain.
///
/// Explicit scope starts are evaluated from retained selector lists. Implicit
/// starts are evaluated from the stylesheet-owner trigger set assembled beside
/// the RuleSet, corresponding to Blink's StyleScopeData::TriggersScope. Scope
/// boundaries use the same selector capability gate as ordinary rules.
pub(crate) fn CanMatchProductionScopeChain(scope: &StyleScope) -> bool {
    let supported = |list: &crate::css_selector_list::CSSSelectorList| {
        crate::production_rule_matching::ProductionUnsupportedParsedFeature(&list.CopySelectors())
            .is_none()
    };
    (scope.IsImplicit() || scope.From().is_some_and(supported))
        && scope.To().is_none_or(supported)
        && scope.Parent().is_none_or(CanMatchProductionScopeChain)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProductionRulePseudoTarget {
    Element,
    Known(PseudoId),
    Unsupported(PseudoType),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProductionPseudoRequest {
    pub id: PseudoId,
    pub argument: Option<AtomicString>,
    pub ident_list: Vec<AtomicString>,
    pub search_text: SearchTextRequest,
}
impl ProductionPseudoRequest {
    pub fn Element() -> Self {
        Self {
            id: PseudoId::kPseudoIdNone,
            argument: None,
            ident_list: Vec::new(),
            search_text: SearchTextRequest::kNone,
        }
    }
    pub fn Pseudo(id: PseudoId, argument: Option<AtomicString>) -> Self {
        Self {
            id,
            argument,
            ident_list: Vec::new(),
            search_text: SearchTextRequest::kNone,
        }
    }

    pub fn SearchText(current: SearchTextRequest) -> Self {
        Self {
            id: PseudoId::kPseudoIdSearchText,
            argument: None,
            ident_list: Vec::new(),
            search_text: current,
        }
    }
}

/// Inputs behind ElementRuleCollector's `reject_starting_styles` calculation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ProductionStartingStyleContext {
    pub is_ensuring_style: bool,
    pub has_old_style: bool,
    pub resolving_style: bool,
    pub force_starting_style: bool,
}
impl ProductionStartingStyleContext {
    pub fn InitialResolution() -> Self {
        Self {
            resolving_style: true,
            ..Default::default()
        }
    }
    pub fn AllowsStartingStyle(self) -> bool {
        !((self.is_ensuring_style || self.has_old_style || !self.resolving_style)
            && !self.force_starting_style)
    }
}

/// Stable ownership of all conditional metadata associated with one RuleData.
/// The rule handle plus selector index make this safe to retain independently
/// of temporary bucket slices.
#[derive(Clone)]
pub(crate) struct ProductionRuleContext {
    rule: Rc<StyleRule<Backend>>,
    selector_index: u32,
    pub position: u32,
    pub pseudo_target: ProductionRulePseudoTarget,
    pub is_starting_style: bool,
    pub scope: Option<Rc<StyleScope>>,
    pub container_queries: Option<Rc<ContainerSet>>,
}
impl ProductionRuleContext {
    pub fn FromRule(set: &RuleSet<Backend>, data: &RuleData<Backend>) -> Self {
        Self {
            rule: data.Rule().clone(),
            selector_index: data.SelectorIndex(),
            position: data.GetPosition(),
            pseudo_target: PseudoTarget(data),
            is_starting_style: data.IsStartingStyle(),
            scope: set.ScopeForRule(data),
            container_queries: set.ContainerQueriesForRule(data),
        }
    }

    pub fn AllowsStartingStyle(&self, context: ProductionStartingStyleContext) -> bool {
        !self.is_starting_style || context.AllowsStartingStyle()
    }

    pub fn MatchesSelector(
        &self,
        session: &PersistentSelectorMatchSession<'_>,
        element_index: usize,
        pseudo: &ProductionPseudoRequest,
        implicit_scope_root: Option<usize>,
    ) -> Option<ScopedSelectorMatch> {
        match self.pseudo_target {
            ProductionRulePseudoTarget::Element if pseudo.id != PseudoId::kPseudoIdNone => {
                return None;
            }
            ProductionRulePseudoTarget::Known(id) if pseudo.id != id => return None,
            ProductionRulePseudoTarget::Unsupported(_) => return None,
            _ => {}
        }
        let selector = self
            .rule
            .Selectors()
            .ComplexAt(self.selector_index as usize);
        session.MatchesCompiledComplexInRuleContext(
            element_index,
            selector,
            self.scope.as_deref(),
            implicit_scope_root,
            pseudo.id,
            pseudo.argument.as_ref(),
            &pseudo.ident_list,
        )
    }
}

fn PseudoTarget(data: &RuleData<Backend>) -> ProductionRulePseudoTarget {
    let mut target = None;
    for selector in data.Selector().SimpleSelectors() {
        if selector.Match() == MatchType::kPseudoElement {
            let pseudo_type = selector.GetPseudoType();
            let id = GetPseudoId(pseudo_type);
            if target.is_some() || id == PseudoId::kPseudoIdNone {
                return ProductionRulePseudoTarget::Unsupported(pseudo_type);
            } else {
                target = Some(ProductionRulePseudoTarget::Known(id));
            }
        }
        if selector.Relation() != RelationType::kSubSelector {
            break;
        }
    }
    target.unwrap_or(ProductionRulePseudoTarget::Element)
}

/// Runtime container selection and condition evaluation remain owned by the
/// embedding StyleEngine. This narrow interface receives the actual parsed
/// query object, including its selector name and immutable condition tree.
pub(crate) trait ProductionContainerEvaluator {
    fn Evaluate(&mut self, query: &ProductionContainerQuery) -> KleeneValue;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ProductionContainerEvaluation {
    pub matches: bool,
    pub sets_evaluated: usize,
    pub queries_evaluated: usize,
    pub unknown_results: usize,
}

/// Evaluates OR within each query set and AND across the nested parent chain,
/// matching ElementRuleCollector::EvaluateAndAddContainerQueries.
pub(crate) fn EvaluateContainerQueryChain(
    set: &ContainerSet,
    evaluator: &mut impl ProductionContainerEvaluator,
) -> ProductionContainerEvaluation {
    let mut report = ProductionContainerEvaluation {
        matches: true,
        ..Default::default()
    };
    let mut current = Some(set);
    while let Some(set) = current {
        report.sets_evaluated += 1;
        let mut set_result = KleeneValue::kFalse;
        for query in set.Queries() {
            report.queries_evaluated += 1;
            let result = evaluator.Evaluate(query);
            if result == KleeneValue::kUnknown {
                report.unknown_results += 1;
            }
            set_result = KleeneOr(set_result, result);
            if set_result == KleeneValue::kTrue {
                break;
            }
        }
        if set_result != KleeneValue::kTrue {
            report.matches = false;
            return report;
        }
        current = set.Parent();
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        element_rule_collector::{MatchRequest, RuleSetGroup},
        parser::css_parser_mode::CSSParserMode,
        persistent_selector::PersistentSelectorService,
        production_style_sheet::{ParseStyleSheet, ProductionMediaQueryEvaluator},
        rule_set::kRuleIsStartingStyle,
    };
    use dom::persistent_document::{DOMAttribute, DOMNamespace};
    use foundation::String as WTFString;
    use std::collections::HashMap;

    fn parse(css: &str) -> crate::production_style_sheet::ProductionStyleSheet {
        ParseStyleSheet(
            &WTFString::from_utf16(&css.encode_utf16().collect::<Vec<_>>()),
            CSSParserMode::kHTMLStandardMode,
        )
    }

    #[test]
    fn production_rule_set_builds_translated_match_request() {
        let sheet = parse(".hit{color:red}");
        let set = Rc::new(sheet.BuildRuleSet(&ProductionMediaQueryEvaluator::default()));
        let mut group = RuleSetGroup::new(0);
        group.AddRuleSet(set);
        let request: MatchRequest<'_, Backend, (), usize> = MatchRequest::new(&group, None, None);
        let bundles = request.AllRuleSets().collect::<Vec<_>>();
        assert_eq!(bundles.len(), 1);
        assert_eq!(bundles[0].style_sheet_index, 0);
        assert_eq!(
            bundles[0]
                .rule_set
                .ClassRules(&AtomicString::from_str("hit"))
                .len(),
            1
        );
    }

    #[test]
    fn typed_scope_and_pseudo_context_drive_retained_selector_checker() {
        let sheet = parse(
            ".plain::before{content:'x'} @scope (.outer) to (.limit) {.hit::before{color:red}}",
        );
        let set = sheet.BuildRuleSet(&ProductionMediaQueryEvaluator::Screen(800.0, 600.0));
        let data = &set.ClassRules(&AtomicString::from_str("hit"))[0];
        let plain = &set.ClassRules(&AtomicString::from_str("plain"))[0];
        let activation = ProductionRuleContext::FromRule(&set, data);
        assert!(activation.scope.is_some());
        assert_eq!(
            activation.pseudo_target,
            ProductionRulePseudoTarget::Known(PseudoId::kPseudoIdBefore)
        );

        let mut owner = dom::DOM::new();
        let document = owner.GetDocumentMut();
        let outer = document.CreateElement(
            DOMNamespace::kHTML,
            "div".into(),
            vec![DOMAttribute {
                local_name: "class".into(),
                value: "outer".into(),
                ..Default::default()
            }],
        );
        document.AppendChild(document.Root(), outer);
        let hit = document.CreateElement(
            DOMNamespace::kHTML,
            "span".into(),
            vec![DOMAttribute {
                local_name: "class".into(),
                value: "hit plain".into(),
                ..Default::default()
            }],
        );
        document.AppendChild(outer, hit);
        let limit = document.CreateElement(
            DOMNamespace::kHTML,
            "div".into(),
            vec![DOMAttribute {
                local_name: "class".into(),
                value: "limit".into(),
                ..Default::default()
            }],
        );
        document.AppendChild(outer, limit);
        let blocked = document.CreateElement(
            DOMNamespace::kHTML,
            "span".into(),
            vec![DOMAttribute {
                local_name: "class".into(),
                value: "hit".into(),
                ..Default::default()
            }],
        );
        document.AppendChild(limit, blocked);

        let mut selectors = PersistentSelectorService::default();
        let session = selectors.StyleMatchSession(document);
        let before = ProductionPseudoRequest::Pseudo(PseudoId::kPseudoIdBefore, None);
        assert!(
            session
                .MatchesCompiledComplexInRuleContext(
                    hit,
                    plain.Selector(),
                    None,
                    None,
                    before.id,
                    before.argument.as_ref(),
                    &before.ident_list,
                )
                .is_some(),
            "pseudo request must reach SelectorChecker independently of scope"
        );
        assert!(activation
            .MatchesSelector(&session, hit, &before, None)
            .is_some());
        assert!(activation
            .MatchesSelector(&session, blocked, &before, None)
            .is_none());
        assert!(activation
            .MatchesSelector(&session, hit, &ProductionPseudoRequest::Element(), None)
            .is_none());
    }

    #[test]
    fn production_scope_matching_accepts_implicit_and_supported_explicit_parent_chains() {
        let sheet = parse(
            "@scope (.outer){@scope (.inner){.explicit{color:red}} \
             @scope {.implicit{color:blue}}} \
             @scope (:target){.unsupported{color:green}}",
        );
        let set = sheet.BuildRuleSet(&ProductionMediaQueryEvaluator::default());
        let explicit = set
            .ScopeForRule(&set.ClassRules(&AtomicString::from_str("explicit"))[0])
            .expect("nested explicit scope");
        assert!(CanMatchProductionScopeChain(&explicit));
        assert!(explicit.Parent().is_some());

        let implicit = set
            .ScopeForRule(&set.ClassRules(&AtomicString::from_str("implicit"))[0])
            .expect("nested implicit scope");
        assert!(CanMatchProductionScopeChain(&implicit));
        assert!(implicit.IsImplicit());

        let unsupported = set
            .ScopeForRule(&set.ClassRules(&AtomicString::from_str("unsupported"))[0])
            .expect("scope with an unsupported boundary feature");
        assert!(!CanMatchProductionScopeChain(&unsupported));
    }

    #[test]
    fn starting_style_gate_matches_collector_recalc_rules() {
        // RuleSet's group recursion independently proves that @starting-style
        // adds this bit. Construct the flagged production RuleData directly so
        // this test stays focused on the collector's recalc-time gate; the
        // production stylesheet integration test covers parser ingestion.
        let sheet = parse(".hit{opacity:0}");
        let set = sheet.BuildRuleSet(&ProductionMediaQueryEvaluator::default());
        let ordinary = &set.ClassRules(&AtomicString::from_str("hit"))[0];
        let mut bloom_hashes = Vec::new();
        let starting_data = RuleData::new(
            ordinary.Rule().clone(),
            ordinary.SelectorIndex(),
            ordinary.GetPosition(),
            None,
            kRuleIsStartingStyle,
            &mut bloom_hashes,
        );
        let starting = ProductionRuleContext::FromRule(&set, &starting_data);
        assert!(starting.is_starting_style);
        assert!(starting.AllowsStartingStyle(ProductionStartingStyleContext::InitialResolution()));
        assert!(
            !starting.AllowsStartingStyle(ProductionStartingStyleContext {
                has_old_style: true,
                resolving_style: true,
                ..Default::default()
            })
        );
        assert!(
            starting.AllowsStartingStyle(ProductionStartingStyleContext {
                has_old_style: true,
                resolving_style: true,
                force_starting_style: true,
                ..Default::default()
            })
        );
    }

    struct NamedEvaluator(HashMap<std::string::String, KleeneValue>);
    impl ProductionContainerEvaluator for NamedEvaluator {
        fn Evaluate(&mut self, query: &ProductionContainerQuery) -> KleeneValue {
            self.0
                .get(&query.SelectorName().Utf8())
                .copied()
                .unwrap_or(KleeneValue::kUnknown)
        }
    }

    #[test]
    fn typed_container_parent_chain_is_and_of_each_sets_or_result() {
        let outer_queries = Rc::new(ContainerSet::new(vec![
            Rc::new(ProductionContainerQuery::new(
                AtomicString::from_str("outer-a"),
                None,
            )),
            Rc::new(ProductionContainerQuery::new(
                AtomicString::from_str("outer-b"),
                None,
            )),
        ]));
        let inner = ContainerSet::new(vec![Rc::new(ProductionContainerQuery::new(
            AtomicString::from_str("inner"),
            None,
        ))])
        .CopyWithParent(Some(outer_queries));

        let mut evaluator = NamedEvaluator(HashMap::from([
            ("inner".into(), KleeneValue::kTrue),
            ("outer-a".into(), KleeneValue::kFalse),
            ("outer-b".into(), KleeneValue::kTrue),
        ]));
        let report = EvaluateContainerQueryChain(&inner, &mut evaluator);
        assert_eq!(report.sets_evaluated, 2);
        assert_eq!(report.queries_evaluated, 3);
        assert!(report.matches);

        evaluator.0.insert("outer-b".into(), KleeneValue::kUnknown);
        let report = EvaluateContainerQueryChain(&inner, &mut evaluator);
        assert!(!report.matches);
        assert_eq!(report.unknown_results, 1);
    }
}
