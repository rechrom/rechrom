// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// cpp: third_party/blink/renderer/core/css/css_global_rule_set.h
// cpp: third_party/blink/renderer/core/css/css_global_rule_set.cc
// Source commit: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
// Source ledger (physical / effective / mapped / omitted / pending):
//   css_global_rule_set.h: 66 / 35 / 21 / 14 / 0.
//   css_global_rule_set.cc: 100 / 80 / 66 / 14 / 0.
// Effective lines exclude comments and blanks; scaffolding is explicitly
// omitted below. Every production declaration/statement is mapped.
// h mapped: 26,30-35,37-44,51,54,58,60-62.
// h omitted: 5-6,8,10,12-13,24-25,27-28,46,48,64,66.
// cc mapped: 17-31,33-39,41-59,61-64,66-67,69-70,72,74,76-78,80-82,84-85,87-93.
// cc omitted: 5,7-13,15,95-98,100.
// Omissions: preprocessor/namespace/forward/access/GC class scaffolding,
// deleted copying and Trace. UA rules/groups are collected by the real
// CSSDefaultStyleSheets rather than reconstructed in this document owner.

use crate::css_default_style_sheets::{CSSDefaultStyleSheets, CSSDefaultStyleSheetsBackend};
use crate::rule_set::{kRuleHasNoSpecialState, ApplyMixinsStack, RuleSet};
use crate::style_engine::{StyleEngine, StyleEngineBackend};
use crate::style_rule::StyleRule;
use crate::style_sheet_contents::{MixinMap, RuleSetHandle};
use std::cell::RefCell;
use std::rc::Rc;

/// Document-owned selector lists, media evaluator and engine are external.
/// Rule ingestion, UA feature collection and engine feature collection use
/// their existing native implementations rather than backend body forwarding.
pub trait CSSGlobalRuleSetBackend: CSSDefaultStyleSheetsBackend {
    type EngineBackend: StyleEngineBackend<
        Document = Self::Document,
        RuleFeatureSet = Self::RuleFeatureSet,
    >;
    fn WatchedCallbackSelectors(document: &Self::Document) -> Option<Vec<Rc<StyleRule<Self>>>>;
    fn DocumentRulesSelectors(document: &Self::Document) -> Vec<Rc<StyleRule<Self>>>;
    fn MediaEvaluatorForDocumentFrame(document: &Self::Document) -> Self::MediaQueryEvaluator;
    fn DocumentStyleEngine(document: &Self::Document) -> Rc<StyleEngine<Self::EngineBackend>>;
    fn ClearFeatures(features: &mut Self::RuleFeatureSet);
}

// cpp: css_global_rule_set.h:51-61
// Reuse the canonical mutable RuleSet handle. Consumers that still require
// Rc<RuleSet> instead of RuleSetHandle have the pre-existing shared-mutability
// integration gap; no RuleSet clone or alternate rule model is introduced.
pub struct CSSGlobalRuleSet<B: CSSGlobalRuleSetBackend> {
    features_: B::RuleFeatureSet,
    watched_selectors_rule_set_: Option<RuleSetHandle<B>>,
    document_rules_selectors_rule_set_: Option<RuleSetHandle<B>>,
    has_fullscreen_ua_style_: bool,
    is_dirty_: bool,
}

impl<B: CSSGlobalRuleSetBackend> CSSGlobalRuleSet<B>
where
    B::MediaQueryEvaluator: 'static,
{
    // cpp: css_global_rule_set.h:27,54-61
    pub fn new() -> Self {
        Self {
            features_: B::NewFeatures(),
            watched_selectors_rule_set_: None,
            document_rules_selectors_rule_set_: None,
            has_fullscreen_ua_style_: false,
            is_dirty_: true,
        }
    }
    // cpp: css_global_rule_set.h:34-47
    pub fn MarkDirty(&mut self) {
        self.is_dirty_ = true;
    }
    pub fn IsDirty(&self) -> bool {
        self.is_dirty_
    }
    pub fn GetRuleFeatureSet(&self) -> &B::RuleFeatureSet {
        &self.features_
    }
    pub fn WatchedSelectorsRuleSet(&self) -> Option<RuleSetHandle<B>> {
        self.watched_selectors_rule_set_.clone()
    }
    pub fn DocumentRulesSelectorsRuleSet(&self) -> Option<RuleSetHandle<B>> {
        self.document_rules_selectors_rule_set_.clone()
    }
    pub fn HasFullscreenUAStyle(&self) -> bool {
        self.has_fullscreen_ua_style_
    }

    // cpp: css_global_rule_set.cc:17-39
    pub fn InitWatchedSelectorsRuleSet(&mut self, document: &B::Document) {
        self.MarkDirty();
        self.watched_selectors_rule_set_ = None;
        let Some(selectors) = B::WatchedCallbackSelectors(document) else {
            return;
        };
        if selectors.is_empty() {
            return;
        }
        let rules = Rc::new(RefCell::new(RuleSet::<B>::new()));
        self.watched_selectors_rule_set_ = Some(rules.clone());
        let medium = B::MediaEvaluatorForDocumentFrame(document);
        let mut stack: ApplyMixinsStack<B> = Vec::new();
        let mixins = MixinMap::<B>::new();
        for selector in selectors {
            rules.borrow_mut().AddStyleRule(
                selector,
                None,
                &medium,
                &mixins,
                kRuleHasNoSpecialState,
                &mut stack,
                None,
                None,
                None,
            );
        }
    }
    // cpp: css_global_rule_set.cc:41-59
    pub fn UpdateDocumentRulesSelectorsRuleSet(&mut self, document: &B::Document) {
        self.MarkDirty();
        self.document_rules_selectors_rule_set_ = None;
        let selectors = B::DocumentRulesSelectors(document);
        if selectors.is_empty() {
            return;
        }
        let rules = Rc::new(RefCell::new(RuleSet::<B>::new()));
        self.document_rules_selectors_rule_set_ = Some(rules.clone());
        let medium = B::MediaEvaluatorForDocumentFrame(document);
        let mut stack: ApplyMixinsStack<B> = Vec::new();
        let mixins = MixinMap::<B>::new();
        for selector in selectors {
            rules.borrow_mut().AddStyleRule(
                selector,
                None,
                &medium,
                &mixins,
                kRuleHasNoSpecialState,
                &mut stack,
                None,
                None,
                None,
            );
        }
        rules.borrow_mut().CompactRulesIfNeeded();
    }
    // cpp: css_global_rule_set.cc:61-85
    pub fn Update(&mut self, document: &B::Document) {
        if !self.is_dirty_ {
            return;
        }
        self.is_dirty_ = false;
        B::ClearFeatures(&mut self.features_);
        let defaults = CSSDefaultStyleSheets::<B>::Instance();
        let mut defaults = defaults.borrow_mut();
        self.has_fullscreen_ua_style_ = defaults.FullscreenStyleSheet().is_some();
        defaults.CollectFeaturesTo(document, &mut self.features_);
        // Release the singleton borrow before document-owned engine callbacks.
        drop(defaults);
        if let Some(rules) = &self.watched_selectors_rule_set_ {
            B::MergeFeatures(&mut self.features_, rules.borrow().Features());
        }
        if let Some(rules) = &self.document_rules_selectors_rule_set_ {
            B::MergeFeatures(&mut self.features_, rules.borrow().Features());
        }
        B::DocumentStyleEngine(document).CollectFeaturesTo(&mut self.features_);
    }
    // cpp: css_global_rule_set.cc:87-93
    pub fn Dispose(&mut self) {
        B::ClearFeatures(&mut self.features_);
        self.watched_selectors_rule_set_ = None;
        self.document_rules_selectors_rule_set_ = None;
        self.has_fullscreen_ua_style_ = false;
        self.is_dirty_ = true;
    }
}

impl<B: CSSGlobalRuleSetBackend> Default for CSSGlobalRuleSet<B>
where
    B::MediaQueryEvaluator: 'static,
{
    fn default() -> Self {
        Self::new()
    }
}
