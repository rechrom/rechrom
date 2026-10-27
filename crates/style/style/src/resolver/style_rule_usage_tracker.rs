//! Translation of Chromium resolver/style_rule_usage_tracker.{h,cc}.
//! Source: /Users/zhenghuaiyu/chromium/src/third_party/blink/renderer/core/css/resolver/
//! Chromium commit: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
//! Ledger (physical / effective / mapped / omitted / pending):
//! style_rule_usage_tracker.h: 38 / 15 / 11 / 4 / 0.
//! style_rule_usage_tracker.cc: 55 / 31 / 28 / 3 / 0.
//! Effective excludes copyright/comments, blanks, preprocessing/includes,
//! namespaces and pure bracket/punctuation lines. Header omissions: forward
//! declaration 14, access labels 17,27 and Oilpan Trace declaration 25.
//! cc omissions: Oilpan Trace function/visits 50-52. All remaining declarations
//! and production cc:13-48 map. Actual CSSStyleSheet and StyleRule identities
//! remain strongly owned; StyleRuleIdentitySet is reused. TakeDelta swaps only
//! the pending map, while permanent membership survives and per-sheet append
//! order is retained. This source version has no ToString method.
#![allow(non_snake_case)]
use crate::css_style_sheet::{CSSStyleSheet, CSSStyleSheetBackend};
use crate::rule_set::StyleRuleIdentitySet;
use crate::style_rule::StyleRule;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

/// Strong ownership plus pointer identity, equivalent to the source's
/// Member<const CSSStyleSheet> hash key. This is not a separate sheet model.
pub struct StyleSheetIdentity<B: CSSStyleSheetBackend>(Rc<CSSStyleSheet<B>>);
impl<B: CSSStyleSheetBackend> StyleSheetIdentity<B> {
    pub fn new(sheet: Rc<CSSStyleSheet<B>>) -> Self {
        Self(sheet)
    }
    pub fn Get(&self) -> &Rc<CSSStyleSheet<B>> {
        &self.0
    }
}
impl<B: CSSStyleSheetBackend> Clone for StyleSheetIdentity<B> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<B: CSSStyleSheetBackend> PartialEq for StyleSheetIdentity<B> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
impl<B: CSSStyleSheetBackend> Eq for StyleSheetIdentity<B> {}
impl<B: CSSStyleSheetBackend> Hash for StyleSheetIdentity<B> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.0).hash(state);
    }
}

pub type RuleListByStyleSheet<B> = HashMap<StyleSheetIdentity<B>, Vec<Rc<StyleRule<B>>>>;

pub struct StyleRuleUsageTracker<B: CSSStyleSheetBackend> {
    used_rules_: HashMap<StyleSheetIdentity<B>, StyleRuleIdentitySet<B>>,
    used_rules_delta_: RuleListByStyleSheet<B>,
}
impl<B: CSSStyleSheetBackend> Default for StyleRuleUsageTracker<B> {
    fn default() -> Self {
        Self {
            used_rules_: HashMap::new(),
            used_rules_delta_: HashMap::new(),
        }
    }
}
impl<B: CSSStyleSheetBackend> StyleRuleUsageTracker<B> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn TakeDelta(&mut self) -> RuleListByStyleSheet<B> {
        // Move the whole delta out. The permanent used identity sets survive,
        // so a rule is never reported again by this tracker after TakeDelta.
        std::mem::take(&mut self.used_rules_delta_)
    }
    fn InsertToUsedRulesMap(
        &mut self,
        parent_sheet: &StyleSheetIdentity<B>,
        rule: &Rc<StyleRule<B>>,
    ) -> bool {
        let set = self.used_rules_.entry(parent_sheet.clone()).or_default();
        if set.Contains(rule) {
            return false;
        }
        set.Insert(rule.clone());
        true
    }
    pub fn Track(&mut self, parent_sheet: Option<Rc<CSSStyleSheet<B>>>, rule: Rc<StyleRule<B>>) {
        let Some(parent_sheet) = parent_sheet else {
            return;
        };
        let parent_sheet = StyleSheetIdentity::new(parent_sheet);
        if !self.InsertToUsedRulesMap(&parent_sheet, &rule) {
            return;
        }
        self.used_rules_delta_
            .entry(parent_sheet)
            .or_default()
            .push(rule);
    }
}
