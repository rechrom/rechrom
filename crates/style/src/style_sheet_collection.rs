// Copyright 1999-2026 The Chromium Authors and other contributors.
// Use of this source code is governed by the license in Chromium's
// third_party/blink/renderer/core/css/style_sheet_collection.h.
// Source ledger: comments/blanks stripped; braces retained.
// style_sheet_collection.h: physical 136, effective 63, mapped 33,
// omitted 30, production pending 0.
// style_sheet_collection.cc: physical 217, effective 140, mapped 105,
// omitted 35, production pending 0.
// Omitted: header/namespace/access/GC/copy-delete/default-destructor scaffolding,
// static helper forward declaration, DCHECK/debug constructor verification,
// GC Trace and test-only h:116-119. All production control is mapped below.
// Direct dependency mapped here: MixinMap::AllocateMapIdentifier/Merge,
// mixin_map.cc:12-16,18-24; the existing MixinMap representation is reused.
// h mapped: 77,82-87,89,92-94,96-102,104-105,107,110,113,121,123-128,130-132
// h omitted: 30-31,33-41,43,45-47,73-76,78-80,91,115-119,134,136
// cc mapped: 50-54,60-63,65-68,76-79,86,88-89,91,93-94,98-101,103-108,119-120,126,128-132,134-137,139-141,143-149
// cc mapped: 151-153,155-161,163-168,170-171,174-176,178-183,185-187,189-192,194-199,201-205,210-212,214-215
// cc omitted: 29,31-41,43,45-48,110-117,121-125,142,173,188,213,217
// Canonical RuleSet shared-mutability integration remains with the existing
// StyleSheetContents/StyleEngine owners. This module never clones a RuleSet or
// creates a second rule representation. RuleSetDiff::Clone retains the same
// underlying diff allocation, matching a C++ Member<RuleSetDiff> copy.

#![allow(non_snake_case, non_camel_case_types)]

use crate::active_style_sheets::{ActiveStyleSheet, RuleSetDiff};
use crate::css_style_sheet::{CSSStyleSheet, CSSStyleSheetBackend};
use crate::rule_set::RuleSet;
use crate::style_engine::{StyleEngine, StyleEngineBackend};
use crate::style_sheet_contents::{MixinMap, StyleSheetContents, StyleSheetContentsBackend};
use foundation::String;
use std::cell::{Cell, Ref, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

pub type ActiveStyleSheetVector<B> = Vec<ActiveStyleSheet<CSSStyleSheet<B>, RuleSet<B>>>;

/// DOM owners supply the existing TreeOrderedList and candidate objects. Add,
/// remove and iteration preserve real node identity and tree order; no fallback
/// insertion order or optional implementation is supplied.
pub trait StyleSheetCollectionDOMBackend:
    CSSStyleSheetBackend<RuleSetDiff: Clone + RuleSetDiff<RuleSet<Self>>> + 'static
{
    type StyleSheet;
    type TreeOrderedNodes;
    type StyleSheetCandidate;
    fn ScopeIsShadowRoot(scope: &Self::TreeScope) -> bool;
    fn ScopeDocument(scope: &Self::TreeScope) -> Rc<Self::Document>;
    fn NewTreeOrderedNodes() -> Self::TreeOrderedNodes;
    fn OrderedNodesAdd(nodes: &Self::TreeOrderedNodes, node: Rc<Self::Node>);
    fn OrderedNodesRemove(nodes: &Self::TreeOrderedNodes, node: &Rc<Self::Node>);
    fn OrderedNodesIsEmpty(nodes: &Self::TreeOrderedNodes) -> bool;
    fn OrderedNodesSnapshot(nodes: &Self::TreeOrderedNodes) -> Vec<Rc<Self::Node>>;
    fn NewStyleSheetCandidate(node: Rc<Self::Node>) -> Self::StyleSheetCandidate;
    fn CandidateIsEnabledAndLoading(candidate: &Self::StyleSheetCandidate) -> bool;
    fn CandidateSheet(candidate: &Self::StyleSheetCandidate) -> Option<Rc<Self::StyleSheet>>;
    fn CandidateCanBeActivated(
        candidate: &Self::StyleSheetCandidate,
        preferred_name: &String,
    ) -> bool;
    fn StyleSheetAsCSS(sheet: &Rc<Self::StyleSheet>) -> Rc<CSSStyleSheet<Self>>;
    fn ScopeHasAdoptedStyleSheets(scope: &Self::TreeScope) -> bool;
    fn ScopeAdoptedStyleSheets(scope: &Self::TreeScope) -> Vec<Option<Rc<CSSStyleSheet<Self>>>>;
}

/// The actual engine shares Document, TreeScope, sheet/contents, rules, mixins
/// and diff types with this collection. No StyleEngine body is delegated here.
pub trait StyleSheetCollectionBackend: StyleSheetCollectionDOMBackend {
    type EngineBackend: StyleEngineBackend<
        Document = Self::Document,
        TreeScope = Self::TreeScope,
        CSSStyleSheet = CSSStyleSheet<Self>,
        StyleSheetContents = StyleSheetContents<Self>,
        RuleSet = RuleSet<Self>,
        MixinMap = MixinMap<Self>,
        RuleSetDiff = Self::RuleSetDiff,
    >;
    fn DocumentStyleEngine(document: &Self::Document) -> Rc<StyleEngine<Self::EngineBackend>>;
}

// Direct dependency: mixin_map.cc:12-16,18-24. The source identifier belongs
// to all Documents/scopes on the renderer main thread, rather than one backend
// monomorphization. Atomic storage preserves that shared sequence in Rust.
static NEXT_MIXIN_MAP_IDENTIFIER: AtomicU64 = AtomicU64::new(0);
impl<B: StyleSheetContentsBackend> MixinMap<B> {
    pub fn AllocateMapIdentifier() -> u64 {
        NEXT_MIXIN_MAP_IDENTIFIER.fetch_add(1, Ordering::Relaxed)
    }
    pub fn Merge(&mut self, other: &Self) {
        for (name, rule) in &other.mixins {
            self.mixins.insert(name.clone(), rule.clone());
        }
        self.media_query_result_flags
            .Add(&other.media_query_result_flags);
        self.media_query_set_results
            .extend(other.media_query_set_results.iter().cloned());
    }
}

// cpp: style_sheet_collection.h:73-132
pub struct StyleSheetCollection<B: StyleSheetCollectionBackend> {
    tree_scope_: Rc<B::TreeScope>,
    style_sheets_for_style_sheet_list_: RefCell<Vec<Rc<B::StyleSheet>>>,
    style_sheet_candidate_nodes_: B::TreeOrderedNodes,
    active_style_sheets_: RefCell<ActiveStyleSheetVector<B>>,
    pending_active_style_sheets_: RefCell<ActiveStyleSheetVector<B>>,
    mixins_: RefCell<MixinMap<B>>,
    sheet_list_dirty_: Cell<bool>,
    is_shadow_tree_: bool,
}
impl<B: StyleSheetCollectionBackend> StyleSheetCollection<B> {
    // cpp: style_sheet_collection.cc:119-126
    pub fn new(scope: Rc<B::TreeScope>) -> Self {
        Self {
            is_shadow_tree_: B::ScopeIsShadowRoot(&scope),
            tree_scope_: scope,
            style_sheets_for_style_sheet_list_: RefCell::new(Vec::new()),
            style_sheet_candidate_nodes_: B::NewTreeOrderedNodes(),
            active_style_sheets_: RefCell::new(Vec::new()),
            pending_active_style_sheets_: RefCell::new(Vec::new()),
            mixins_: RefCell::new(MixinMap::new()),
            sheet_list_dirty_: Cell::new(true),
        }
    }
    // cpp: style_sheet_collection.h:82-87
    pub fn ActiveStyleSheets(&self) -> Ref<'_, ActiveStyleSheetVector<B>> {
        self.active_style_sheets_.borrow()
    }
    pub fn StyleSheetsForStyleSheetList(&self) -> Ref<'_, Vec<Rc<B::StyleSheet>>> {
        self.style_sheets_for_style_sheet_list_.borrow()
    }
    // cpp: style_sheet_collection.h:89
    pub fn MarkSheetListDirty(&self) {
        self.sheet_list_dirty_.set(true);
    }
    // cpp: style_sheet_collection.h:92-94
    pub fn GetHumanReadableName(&self) -> &'static str {
        "StyleSheetCollection"
    }
    // cpp: style_sheet_collection.h:97-102
    pub fn RemoveStyleSheetCandidateNode(&self, node: &Rc<B::Node>) {
        B::OrderedNodesRemove(&self.style_sheet_candidate_nodes_, node);
    }
    pub fn HasStyleSheetCandidateNodes(&self) -> bool {
        !B::OrderedNodesIsEmpty(&self.style_sheet_candidate_nodes_)
    }
    // cpp: style_sheet_collection.h:104-107
    pub fn IsShadowTreeStyleSheetCollection(&self) -> bool {
        self.is_shadow_tree_
    }
    pub fn Mixins(&self) -> Ref<'_, MixinMap<B>> {
        self.mixins_.borrow()
    }
    // cpp: style_sheet_collection.h:121
    fn GetDocument(&self) -> Rc<B::Document> {
        B::ScopeDocument(&self.tree_scope_)
    }
    // cpp: style_sheet_collection.cc:128-132
    pub fn AddStyleSheetCandidateNode(&self, node: Rc<B::Node>) {
        if B::NodeIsConnected(&node) {
            B::OrderedNodesAdd(&self.style_sheet_candidate_nodes_, node);
        }
    }
    // cpp: style_sheet_collection.cc:134-153
    pub fn UpdateStyleSheetList(&self) {
        if !self.sheet_list_dirty_.get() {
            return;
        }
        let mut new_list = Vec::new();
        for node in B::OrderedNodesSnapshot(&self.style_sheet_candidate_nodes_) {
            let candidate = B::NewStyleSheetCandidate(node);
            if B::CandidateIsEnabledAndLoading(&candidate) {
                continue;
            }
            if let Some(sheet) = B::CandidateSheet(&candidate) {
                new_list.push(sheet);
            }
        }
        *self.style_sheets_for_style_sheet_list_.borrow_mut() = new_list;
        self.sheet_list_dirty_.set(false);
    }
    // cpp: style_sheet_collection.cc:155-215
    pub fn PrepareUpdateActiveStyleSheets(&self, medium: &B::MediaQueryEvaluator) {
        let mut sheets = Vec::new();
        let engine = if self.is_shadow_tree_ {
            None
        } else {
            Some(B::DocumentStyleEngine(&self.GetDocument()))
        };
        let preferred = engine.as_ref().map_or_else(String::default, |engine| {
            engine.PreferredStylesheetSetName()
        });
        if let Some(engine) = &engine {
            for (_, sheet) in engine.InjectedAuthorStyleSheets().iter() {
                sheets.push(ActiveStyleSheet::new(sheet.clone(), None));
            }
        }
        for node in B::OrderedNodesSnapshot(&self.style_sheet_candidate_nodes_) {
            let candidate = B::NewStyleSheetCandidate(node);
            if B::CandidateIsEnabledAndLoading(&candidate) {
                continue;
            }
            if let Some(sheet) = B::CandidateSheet(&candidate) {
                if B::CandidateCanBeActivated(&candidate, &preferred) {
                    sheets.push(ActiveStyleSheet::new(B::StyleSheetAsCSS(&sheet), None));
                }
            }
        }
        if B::ScopeHasAdoptedStyleSheets(&self.tree_scope_) {
            for sheet in B::ScopeAdoptedStyleSheets(&self.tree_scope_)
                .into_iter()
                .flatten()
            {
                if sheet.CanBeActivated(&preferred) {
                    sheets.push(ActiveStyleSheet::new(sheet, None));
                }
            }
        }
        if let Some(engine) = &engine {
            for sheet in engine.InspectorStyleSheets().iter() {
                sheets.push(ActiveStyleSheet::new(sheet.clone(), None));
            }
        }
        let had_mixins = self.mixins_.borrow().HasMixins();
        *self.mixins_.borrow_mut() = MixinMap::new();
        for sheet in &sheets {
            let contents = sheet.style_sheet.Contents();
            let extracted = contents.ExtractMixins(medium);
            self.mixins_.borrow_mut().Merge(&extracted);
        }
        if had_mixins || self.mixins_.borrow().HasMixins() {
            self.mixins_.borrow_mut().map_identifier = Some(MixinMap::<B>::AllocateMapIdentifier());
        }
        *self.pending_active_style_sheets_.borrow_mut() = sheets;
    }
    // cpp: style_sheet_collection.cc:50-68
    pub fn FinishUpdateActiveStyleSheets(&self, effective_mixins: &MixinMap<B>) {
        let engine = B::DocumentStyleEngine(&self.GetDocument());
        let mut diffs = Vec::new();
        Self::CreateRuleSets(
            &engine,
            effective_mixins,
            &mut self.pending_active_style_sheets_.borrow_mut(),
            &mut diffs,
        );
        // Publish and empty pending before the callback. The inspector can
        // synchronously prepare the next update while ApplyRuleSetChanges runs.
        let pending = std::mem::take(&mut *self.pending_active_style_sheets_.borrow_mut());
        let old = std::mem::replace(&mut *self.active_style_sheets_.borrow_mut(), pending);
        let current = self.active_style_sheets_.borrow().clone();
        engine.ApplyRuleSetChanges(&self.tree_scope_, &old, &current, &diffs);
    }
    // cpp: style_sheet_collection.cc:76-108
    fn CreateRuleSets(
        engine: &StyleEngine<B::EngineBackend>,
        mixins: &MixinMap<B>,
        sheets: &mut ActiveStyleSheetVector<B>,
        diffs: &mut Vec<B::RuleSetDiff>,
    ) {
        // Strong contents handles preserve allocation identity throughout this
        // pass, including repeated adoption of the same sheet/contents.
        let mut seen: HashMap<usize, Rc<StyleSheetContents<B>>> = HashMap::new();
        for sheet in sheets {
            assert!(
                sheet.rule_set.is_none(),
                "CreateRuleSets may only run once per prepared batch"
            );
            let contents = sheet.style_sheet.Contents();
            let repeated = seen
                .insert(Rc::as_ptr(&contents) as usize, contents.clone())
                .is_some();
            let has_repeated_layers = repeated
                && contents.HasRuleSet()
                && contents.GetRuleSet().borrow().HasCascadeLayers();
            sheet.rule_set = if has_repeated_layers {
                engine.CreateUnconnectedRuleSet(&sheet.style_sheet, mixins)
            } else {
                engine.RuleSetForSheet(&sheet.style_sheet, mixins)
            };
            let diff = contents.GetRuleSetDiff().clone();
            if let Some(diff) = diff {
                diffs.push(diff);
                contents.ClearRuleSetDiff();
            }
        }
    }
}
