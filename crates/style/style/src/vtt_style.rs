//! StyleEngine's compiled WebVTT stylesheet clients. The TextTrack retains the
//! original CSSStyleSheet/parsed backing; this cache owns its normal RuleSet
//! projection and consults the native media dependency checks before reuse.
use crate::{
    media_queries::media_values_cached::MediaValuesCachedData,
    production_style_sheet::{Backend, ProductionMediaQueryEvaluator, ProductionStyleSheet},
    resolver::production_style_resolver::ProductionResolverRuleSetInput,
    rule_set::RuleSet,
};
use std::{
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};
struct Entry {
    parsed: Weak<ProductionStyleSheet>,
    rules: Rc<RuleSet<Backend>>,
}
#[derive(Default)]
pub(crate) struct VTTStyleSheetRules {
    entries: RefCell<HashMap<(u64, u64), Entry>>,
}
impl VTTStyleSheetRules {
    pub(crate) fn Dispose(&self) {
        self.entries.borrow_mut().clear();
    }
    pub fn Prune(&self) {
        self.entries
            .borrow_mut()
            .retain(|_, e| e.parsed.strong_count() > 0);
    }
    pub fn RuleSetForSheet(
        &self,
        sheet: &cssom::CSSStyleSheet,
        media: &MediaValuesCachedData,
    ) -> Rc<RuleSet<Backend>> {
        let key = (sheet.Identity(), sheet.Revision());
        let evaluator = ProductionMediaQueryEvaluator::FromCachedData(media);
        if let Some(existing) = self.entries.borrow().get(&key) {
            if existing.parsed.strong_count() > 0
                && !existing.rules.DidMediaQueryResultsChange(&evaluator)
            {
                return existing.rules.clone();
            }
        }
        let parsed = sheet
            .ParsedBacking::<ProductionStyleSheet>()
            .expect("VTT stylesheet retains its original compiled owner");
        assert_eq!(
            parsed.contents.ParserContext().ResourceFetchRestriction(),
            crate::css_resource_fetch_restriction::ResourceFetchRestriction::kOnlyDataUrls
        );
        let rules = Rc::new(parsed.BuildRuleSet(&evaluator));
        self.entries.borrow_mut().insert(
            key,
            Entry {
                parsed: Rc::downgrade(&parsed),
                rules: rules.clone(),
            },
        );
        rules
    }
    pub fn InputForRules(&self, rules: &Rc<RuleSet<Backend>>) -> ProductionResolverRuleSetInput {
        let entries = self.entries.borrow();
        let (&(sheet_identity, sheet_revision), _) = entries
            .iter()
            .find(|(_, e)| Rc::ptr_eq(&e.rules, rules))
            .expect("VTT MatchRequest retains the compiled sheet's RuleSet identity");
        ProductionResolverRuleSetInput {
            rule_set: rules.clone(),
            implicit_scope_trigger: None,
            sheet_identity,
            sheet_revision,
            tree_scope_root: None,
        }
    }
}
/// The original StyleEngine text_tracks_ weak membership. The adapter holds
/// only a weak reference to that track and borrows its current sheet vector.
pub trait TextTrackStyleOwner {
    fn IsAlive(&self) -> bool;
    fn OwnerNodeId(&self) -> Option<u64>;
    fn VisitStyleSheets(&self, visitor: &mut dyn FnMut(&Rc<cssom::CSSStyleSheet>));
}
impl VTTStyleSheetRules {
    pub fn ClearRuleSet(&self, sheet: &cssom::CSSStyleSheet) {
        self.entries
            .borrow_mut()
            .remove(&(sheet.Identity(), sheet.Revision()));
    }
}
