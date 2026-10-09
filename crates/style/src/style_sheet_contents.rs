/*
 * (C) 1999-2003 Lars Knoll (knoll@kde.org)
 * Copyright (C) 2004, 2006, 2007, 2012 Apple Inc. All rights reserved.
 * This library is free software under the GNU Library General Public License,
 * version 2 or (at your option) any later version. See COPYING.LIB.
 */
// Source ledger: effective = nonblank source lines after stripping comments,
// minus the exact boilerplate/debug-only exclusions below (braces are counted).
// style_sheet_contents.h: physical 344; effective 152; mapped 152; remaining 0.
// style_sheet_contents.cc: physical 995; effective 713; mapped 713; remaining 0.
// Mapped: .h:61-339; .cc:43-128,132-978 (excluding the omitted lines below).
// Omitted .h:1-60,67-68,258,281,283-284,298,340-344: include/class wrapper,
// deleted ctor/assignment, destructor, DCHECK, GC Trace and type alias.
// Omitted .cc:1-42,130-131,979-995: includes, destructor and GC Trace.
// Omitted .cc debug-only checks:115,172,196,209,226,257,280-281,375-376,
// 421,504,523,538,687,710-711,749,761,768-770,775-776.
// Source NOTREACHED at .cc:649 is mapped to unreachable!, and is counted.
// External bodies are mandatory typed dependencies with no defaults; their
// source files are not claimed as translated by this ledger. RuleSet building,
// diff tracking, import loading, resource response, Document, Node and client
// callbacks remain those classes' operations. No synthetic RuleSet is returned.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use crate::parser::css_parser_context::{
    CSSParserContext, CSSParserContextPlatform, DocumentHandle, SingleOwnerDocument,
};
use crate::parser::css_parser_impl::{CSSParserImpl, CSSParserImplSheetBackend, ParseSheetResult};
use crate::parser::css_parser_mode::{CSSDeferPropertyParsing, IsQuirksModeBehavior};
use crate::resolver::media_query_result::{MediaQueryResultFlags, MediaQuerySetResult};
use crate::rule_set::{RuleSet, RuleSetBackend};
use crate::style_rule::{
    ReplaceStyleRuleInVector, StyleRule, StyleRuleBase, StyleRuleCloneDependencies,
    StyleRuleFontFace, StyleRuleMedia, StyleRulePropertySet, StyleRulePropertySetClone,
};
use foundation::{AtomicString, String};
use std::cell::{Cell, Ref, RefCell};
use std::collections::HashMap;
use std::rc::{Rc, Weak};

// platform/loader/fetch/render_blocking_behavior.h:12-19
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum RenderBlockingBehavior {
    kUnset,
    kBlocking,
    kNonBlocking,
    kNonBlockingDynamic,
    kPotentiallyBlocking,
    kInBodyParserBlocking,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MIMETypeCheck {
    kLax,
    kStrict,
}

pub type RuleSetHandle<B> = Rc<RefCell<RuleSet<B>>>;

/// Only calls owned by other Chromium classes enter through this boundary.
/// The caller must assemble the actual parser, loader, diff and style engine.
pub trait StyleSheetContentsBackend: RuleSetBackend + 'static {
    type Platform: CSSParserContextPlatform;
    type ParserBackend: CSSParserImplSheetBackend<
        Platform = Self::Platform,
        StyleRuleBase = StyleRuleBase<Self>,
        StyleSheetContents = Rc<StyleSheetContents<Self>>,
    >;
    type CSSStyleSheet: 'static;
    type Node: 'static;
    type TreeScope: PartialEq;
    type Resource: 'static;
    type RuleSetDiff;

    fn ImportMediaQueries(rule: &Self::StyleRuleImport) -> Option<Rc<Self::MediaQuerySet>>;
    fn ImportStyleSheet(rule: &Self::StyleRuleImport) -> Option<Rc<StyleSheetContents<Self>>>;
    fn ImportParentStyleSheet(rule: &Self::StyleRuleImport)
        -> Option<Rc<StyleSheetContents<Self>>>;
    fn SetImportParentStyleSheet(
        rule: &Self::StyleRuleImport,
        parent: &Rc<StyleSheetContents<Self>>,
    );
    fn ClearImportParentStyleSheet(rule: &Self::StyleRuleImport);
    fn RequestImportStyleSheet(rule: &Self::StyleRuleImport);
    fn ImportIsLoading(rule: &Self::StyleRuleImport) -> bool;
    fn ImportIsSupported(rule: &Self::StyleRuleImport) -> bool;
    fn NamespacePrefix(rule: &Self::StyleRuleNamespace) -> AtomicString;
    fn NamespaceURI(rule: &Self::StyleRuleNamespace) -> AtomicString;
    fn MediaQueriesForRule(rule: &StyleRuleMedia<Self>) -> Option<Rc<Self::MediaQuerySet>>;
    fn CounterStyleHasFailedOrCanceledSubresources(rule: &Self::StyleRuleCounterStyle) -> bool;

    fn OwnerDocument(client: &Self::CSSStyleSheet) -> Option<Rc<Self::Document>>;
    fn OwnerNode(client: &Self::CSSStyleSheet) -> Option<Rc<Self::Node>>;
    fn IsConstructed(client: &Self::CSSStyleSheet) -> bool;
    fn ClientLoadCompleted(client: &Self::CSSStyleSheet) -> bool;
    fn SheetLoaded(client: &Self::CSSStyleSheet) -> bool;
    fn SetClientToPendingState(client: &Self::CSSStyleSheet);
    fn IsAdoptedByTreeScope(client: &Self::CSSStyleSheet, scope: &Self::TreeScope) -> bool;
    fn NodeTreeScope(node: &Self::Node) -> &Self::TreeScope;
    fn NodeIsConnected(node: &Self::Node) -> bool;
    fn NodeDocument(node: &Self::Node) -> Rc<Self::Document>;
    fn NotifyLoadedSheetAndAllCriticalSubresources(node: &Self::Node, error_occurred: bool);
    fn SetNeedsActiveStyleUpdate(document: &Self::Document, scope: &Self::TreeScope);
    fn RemoveFontFaceRule(document: &Self::Document, rule: &StyleRuleFontFace<Self>);
    fn ParserDocument(document: &Rc<Self::Document>) -> DocumentHandle<Self::Platform>;

    fn TraceParseAuthorStyleSheet(resource: &Self::Resource);
    fn ResourceErrorOccurred(resource: &Self::Resource) -> bool;
    fn ResourceIsCorsSameOrigin(resource: &Self::Resource) -> bool;
    fn ResourceSheetText(
        resource: &Self::Resource,
        context: &CSSParserContext<Self::Platform>,
        check: MIMETypeCheck,
    ) -> String;
    fn ResourceSourceMapHeader(resource: &Self::Resource) -> String;
    fn ResourceDeprecatedSourceMapHeader(resource: &Self::Resource) -> String;

    fn MediaEvaluatorDocument(medium: &Self::MediaQueryEvaluator) -> Option<&Self::Document>;
    fn AddRulesFromSheet(
        rule_set: &mut RuleSet<Self>,
        sheet: &StyleSheetContents<Self>,
        medium: &Self::MediaQueryEvaluator,
        mixins: &MixinMap<Self>,
    );
    fn CompactRulesIfNeeded(rule_set: &mut RuleSet<Self>);
    fn NewRuleSetDiff(old: RuleSetHandle<Self>) -> Self::RuleSetDiff;
    fn AddRuleDiff(diff: &mut Self::RuleSetDiff, rule: Rc<StyleRuleBase<Self>>);
    fn MarkDiffUnrepresentable(diff: &mut Self::RuleSetDiff);
    fn NewRuleSetCleared(diff: &mut Self::RuleSetDiff);
    fn NewRuleSetCreated(diff: &mut Self::RuleSetDiff, rules: RuleSetHandle<Self>);
}

// mixin_map.h:32-62. Values retain the original StyleRuleBase allocation;
// every entry is a Mixin variant. This preserves rule identity without copying.
pub struct MixinMap<B: StyleSheetContentsBackend> {
    pub mixins: HashMap<AtomicString, Rc<StyleRuleBase<B>>>,
    pub media_query_result_flags: MediaQueryResultFlags,
    pub media_query_set_results: Vec<MediaQuerySetResult>,
    pub map_identifier: Option<u64>,
}
impl<B: StyleSheetContentsBackend> MixinMap<B> {
    pub fn new() -> Self {
        Self {
            mixins: HashMap::new(),
            media_query_result_flags: MediaQueryResultFlags::default(),
            media_query_set_results: vec![],
            map_identifier: None,
        }
    }
    pub fn HasMixins(&self) -> bool {
        !self.mixins.is_empty() || !self.media_query_set_results.is_empty()
    }
}
impl<B: StyleSheetContentsBackend> Default for MixinMap<B> {
    fn default() -> Self {
        Self::new()
    }
}

// Weak sets model HeapHashSet<WeakMember<CSSStyleSheet>>; scripts can detach or
// drop clients during callbacks. Snapshot upgrades keep them alive for a loop.
struct WeakClientSet<C> {
    clients: Vec<Weak<C>>,
}
impl<C> WeakClientSet<C> {
    fn new() -> Self {
        Self { clients: vec![] }
    }
    fn snapshot(&self) -> Vec<Rc<C>> {
        self.clients.iter().filter_map(Weak::upgrade).collect()
    }
    fn len(&self) -> usize {
        self.clients.iter().filter(|c| c.strong_count() > 0).count()
    }
    fn contains(&self, client: &Rc<C>) -> bool {
        self.clients
            .iter()
            .any(|c| c.ptr_eq(&Rc::downgrade(client)))
    }
    fn insert(&mut self, client: &Rc<C>) {
        self.clients.retain(|c| c.strong_count() > 0);
        if !self.contains(client) {
            self.clients.push(Rc::downgrade(client));
        }
    }
    fn erase(&mut self, client: &Rc<C>) {
        self.clients
            .retain(|c| c.strong_count() > 0 && !c.ptr_eq(&Rc::downgrade(client)));
    }
}

// style_sheet_contents.h:290-339
pub struct StyleSheetContents<B: StyleSheetContentsBackend> {
    owner_rule: RefCell<Option<Rc<B::StyleRuleImport>>>,
    original_url: String,
    pre_import_layer_statement_rules: RefCell<Vec<Rc<StyleRuleBase<B>>>>,
    import_rules: RefCell<Vec<Rc<StyleRuleBase<B>>>>,
    namespace_rules: RefCell<Vec<Rc<StyleRuleBase<B>>>>,
    child_rules: RefCell<Vec<Rc<StyleRuleBase<B>>>>,
    namespaces: RefCell<HashMap<AtomicString, AtomicString>>,
    default_namespace: RefCell<AtomicString>,
    referenced_from_resource: RefCell<Option<Weak<B::Resource>>>,
    has_syntactically_valid_css_header: Cell<bool>,
    did_load_error_occur: Cell<bool>,
    is_mutable: Cell<bool>,
    has_font_face_rule: Cell<bool>,
    has_media_queries: Cell<bool>,
    has_single_owner_document: Cell<bool>,
    is_used_from_text_cache: Cell<bool>,
    is_used_from_resource_cache: Cell<bool>,
    parser_context: Rc<CSSParserContext<B::Platform>>,
    loading_clients: RefCell<WeakClientSet<B::CSSStyleSheet>>,
    completed_clients: RefCell<WeakClientSet<B::CSSStyleSheet>>,
    mixins: RefCell<MixinMap<B>>,
    has_cached_mixins: Cell<bool>,
    rule_set: RefCell<Option<RuleSetHandle<B>>>,
    rule_set_diff: RefCell<Option<B::RuleSetDiff>>,
    source_map_url: RefCell<String>,
    render_blocking_behavior: Cell<RenderBlockingBehavior>,
}
impl<B: StyleSheetContentsBackend> StyleSheetContents<B> {
    // cc:72-86
    pub fn new(
        context: Rc<CSSParserContext<B::Platform>>,
        original_url: String,
        owner_rule: Option<Rc<B::StyleRuleImport>>,
    ) -> Rc<Self> {
        Rc::new(Self {
            owner_rule: RefCell::new(owner_rule),
            original_url,
            pre_import_layer_statement_rules: RefCell::new(vec![]),
            import_rules: RefCell::new(vec![]),
            namespace_rules: RefCell::new(vec![]),
            child_rules: RefCell::new(vec![]),
            namespaces: RefCell::new(HashMap::new()),
            default_namespace: RefCell::new(AtomicString::from_str("*")),
            referenced_from_resource: RefCell::new(None),
            has_syntactically_valid_css_header: Cell::new(true),
            did_load_error_occur: Cell::new(false),
            is_mutable: Cell::new(false),
            has_font_face_rule: Cell::new(false),
            has_media_queries: Cell::new(false),
            has_single_owner_document: Cell::new(true),
            is_used_from_text_cache: Cell::new(false),
            is_used_from_resource_cache: Cell::new(false),
            parser_context: context,
            loading_clients: RefCell::new(WeakClientSet::new()),
            completed_clients: RefCell::new(WeakClientSet::new()),
            mixins: RefCell::new(MixinMap::new()),
            has_cached_mixins: Cell::new(false),
            rule_set: RefCell::new(None),
            rule_set_diff: RefCell::new(None),
            source_map_url: RefCell::new(String::default()),
            render_blocking_behavior: Cell::new(RenderBlockingBehavior::kUnset),
        })
    }
    // h:72-279 (inline accessors)
    pub fn ParserContext(&self) -> &Rc<CSSParserContext<B::Platform>> {
        &self.parser_context
    }
    pub fn DefaultNamespace(&self) -> AtomicString {
        self.default_namespace.borrow().clone()
    }
    pub fn Charset(&self) -> &<B::Platform as CSSParserContextPlatform>::TextEncoding {
        self.parser_context.Charset()
    }
    pub fn BaseURL(&self) -> &<B::Platform as CSSParserContextPlatform>::URL {
        self.parser_context.BaseURL()
    }
    pub fn OriginalURL(&self) -> String {
        self.original_url.clone()
    }
    pub fn IsOriginClean(&self) -> bool {
        self.parser_context.IsOriginClean()
    }
    pub fn HasSyntacticallyValidCSSHeader(&self) -> bool {
        self.has_syntactically_valid_css_header.get()
    }
    pub fn SetHasSyntacticallyValidCSSHeader(&self, value: bool) {
        self.has_syntactically_valid_css_header.set(value);
    }
    pub fn HasFontFaceRule(&self) -> bool {
        self.has_font_face_rule.get()
    }
    pub fn SetHasFontFaceRule(&self) {
        self.has_font_face_rule.set(true);
    }
    pub fn HasMediaQueries(&self) -> bool {
        self.has_media_queries.get()
    }
    pub fn HasSingleOwnerDocument(&self) -> bool {
        self.has_single_owner_document.get()
    }
    pub fn IsMutable(&self) -> bool {
        self.is_mutable.get()
    }
    pub fn IsUsedFromTextCache(&self) -> bool {
        self.is_used_from_text_cache.get()
    }
    pub fn SetIsUsedFromTextCache(&self) {
        self.is_used_from_text_cache.set(true);
    }
    pub fn IsUsedFromResourceCache(&self) -> bool {
        self.is_used_from_resource_cache.get()
    }
    pub fn SetIsUsedFromResourceCache(&self) {
        self.is_used_from_resource_cache.set(true);
    }
    pub fn DidLoadErrorOccur(&self) -> bool {
        self.did_load_error_occur.get()
    }
    pub fn SourceMapURL(&self) -> String {
        self.source_map_url.borrow().clone()
    }
    pub fn SetRenderBlocking(&self, value: RenderBlockingBehavior) {
        self.render_blocking_behavior.set(value);
    }
    pub fn GetRenderBlockingBehavior(&self) -> RenderBlockingBehavior {
        self.render_blocking_behavior.get()
    }
    pub fn OwnerRule(&self) -> Option<Rc<B::StyleRuleImport>> {
        self.owner_rule.borrow().clone()
    }
    pub fn ClearOwnerRule(&self) {
        self.owner_rule.borrow_mut().take();
    }
    pub fn ChildRules(&self) -> Ref<'_, Vec<Rc<StyleRuleBase<B>>>> {
        self.child_rules.borrow()
    }
    pub fn PreImportLayerStatementRules(&self) -> Ref<'_, Vec<Rc<StyleRuleBase<B>>>> {
        self.pre_import_layer_statement_rules.borrow()
    }
    pub fn ImportRules(&self) -> Ref<'_, Vec<Rc<StyleRuleBase<B>>>> {
        self.import_rules.borrow()
    }
    pub fn NamespaceRules(&self) -> Ref<'_, Vec<Rc<StyleRuleBase<B>>>> {
        self.namespace_rules.borrow()
    }

    // cc:136-181
    pub fn IsCacheableForResource(self: &Rc<Self>) -> bool {
        self.LoadCompleted()
            && self.import_rules.borrow().is_empty()
            && self.owner_rule.borrow().is_none()
            && !self.DidLoadErrorOccur()
            && !self.IsMutable()
            && self.HasSyntacticallyValidCSSHeader()
    }
    pub fn IsCacheableForStyleElement(&self) -> bool {
        if !self.import_rules.borrow().is_empty() {
            return false;
        }
        debug_assert!(!self.DidLoadErrorOccur());
        !self.IsMutable() && self.HasSyntacticallyValidCSSHeader()
    }
    // cc:183-216
    pub fn ParserAppendRule(self: &Rc<Self>, rule: Rc<StyleRuleBase<B>>) {
        match rule.as_ref() {
            StyleRuleBase::LayerStatement(_)
                if self.import_rules.borrow().is_empty()
                    && self.namespace_rules.borrow().is_empty()
                    && self.child_rules.borrow().is_empty() =>
            {
                self.pre_import_layer_statement_rules
                    .borrow_mut()
                    .push(rule);
                return;
            }
            StyleRuleBase::Import(import) => {
                debug_assert!(self.child_rules.borrow().is_empty());
                if B::ImportMediaQueries(import).is_some() {
                    self.SetHasMediaQueries();
                }
                self.import_rules.borrow_mut().push(rule.clone());
                B::SetImportParentStyleSheet(import, self);
                B::RequestImportStyleSheet(import);
                return;
            }
            StyleRuleBase::Namespace(namespace) => {
                debug_assert!(self.child_rules.borrow().is_empty());
                self.ParserAddNamespace(B::NamespacePrefix(namespace), B::NamespaceURI(namespace));
                self.namespace_rules.borrow_mut().push(rule);
                return;
            }
            _ => {}
        }
        self.child_rules.borrow_mut().push(rule);
    }
    // cc:218-223
    pub fn SetHasMediaQueries(&self) {
        self.has_media_queries.set(true);
        if let Some(parent) = self.ParentStyleSheet() {
            parent.SetHasMediaQueries();
        }
    }
    // cc:225-252
    pub fn RuleCount(&self) -> usize {
        self.pre_import_layer_statement_rules.borrow().len()
            + self.import_rules.borrow().len()
            + self.namespace_rules.borrow().len()
            + self.child_rules.borrow().len()
    }
    pub fn RuleAt(&self, mut index: usize) -> Rc<StyleRuleBase<B>> {
        for rules in [
            &self.pre_import_layer_statement_rules,
            &self.import_rules,
            &self.namespace_rules,
            &self.child_rules,
        ] {
            let rules = rules.borrow();
            if index < rules.len() {
                return rules[index].clone();
            }
            index -= rules.len();
        }
        panic!("StyleSheetContents::RuleAt index out of range");
    }
    // cc:254-268
    pub fn ClearRules(&self) {
        self.pre_import_layer_statement_rules.borrow_mut().clear();
        let imports = self.import_rules.borrow().clone();
        for rule in imports {
            if let StyleRuleBase::Import(import) = rule.as_ref() {
                B::ClearImportParentStyleSheet(import);
            }
        }
        self.with_diff(B::MarkDiffUnrepresentable);
        self.import_rules.borrow_mut().clear();
        self.namespace_rules.borrow_mut().clear();
        self.child_rules.borrow_mut().clear();
    }
    // cc:270-276
    pub fn ReplaceChildRuleIfExists(
        &self,
        old: &Rc<StyleRuleBase<B>>,
        new: Rc<StyleRuleBase<B>>,
        hint: usize,
    ) -> usize {
        ReplaceStyleRuleInVector(old, new, hint, &mut self.child_rules.borrow_mut())
    }
    // cc:278-372
    pub fn WrapperInsertRule(
        self: &Rc<Self>,
        rule: Rc<StyleRuleBase<B>>,
        mut index: usize,
    ) -> bool {
        debug_assert!(self.IsMutable());
        assert!(index <= self.RuleCount());
        self.NotifyRuleChanged(rule.clone());
        if !self.pre_import_layer_statement_rules.borrow().is_empty()
            && self.import_rules.borrow().is_empty()
            && self.namespace_rules.borrow().is_empty()
        {
            let mut statements = self.pre_import_layer_statement_rules.borrow_mut();
            self.child_rules
                .borrow_mut()
                .splice(0..0, statements.drain(..));
        }
        let pre_count = self.pre_import_layer_statement_rules.borrow().len();
        if index < pre_count
            || (index == pre_count && matches!(rule.as_ref(), StyleRuleBase::LayerStatement(_)))
        {
            if !matches!(rule.as_ref(), StyleRuleBase::LayerStatement(_)) {
                return false;
            }
            self.pre_import_layer_statement_rules
                .borrow_mut()
                .insert(index, rule);
            return true;
        }
        index -= pre_count;
        let import_count = self.import_rules.borrow().len();
        if index < import_count
            || (index == import_count && matches!(rule.as_ref(), StyleRuleBase::Import(_)))
        {
            let StyleRuleBase::Import(import) = rule.as_ref() else {
                return false;
            };
            if B::ImportMediaQueries(import).is_some() {
                self.SetHasMediaQueries();
            }
            self.import_rules.borrow_mut().insert(index, rule.clone());
            B::SetImportParentStyleSheet(import, self);
            B::RequestImportStyleSheet(import);
            return true;
        }
        if matches!(rule.as_ref(), StyleRuleBase::Import(_)) {
            return false;
        }
        index -= import_count;
        let namespace_count = self.namespace_rules.borrow().len();
        if index < namespace_count
            || (index == namespace_count && matches!(rule.as_ref(), StyleRuleBase::Namespace(_)))
        {
            let StyleRuleBase::Namespace(namespace) = rule.as_ref() else {
                return false;
            };
            if !self.child_rules.borrow().is_empty() {
                return false;
            }
            self.namespace_rules
                .borrow_mut()
                .insert(index, rule.clone());
            self.ParserAddNamespace(B::NamespacePrefix(namespace), B::NamespaceURI(namespace));
            return true;
        }
        if matches!(rule.as_ref(), StyleRuleBase::Namespace(_)) {
            return false;
        }
        index -= namespace_count;
        self.child_rules.borrow_mut().insert(index, rule);
        true
    }
    // cc:374-417
    pub fn WrapperDeleteRule(self: &Rc<Self>, mut index: usize) -> bool {
        debug_assert!(self.IsMutable());
        assert!(index < self.RuleCount());
        let pre_count = self.pre_import_layer_statement_rules.borrow().len();
        if index < pre_count {
            let rule = self.pre_import_layer_statement_rules.borrow()[index].clone();
            self.NotifyRuleChanged(rule);
            self.pre_import_layer_statement_rules
                .borrow_mut()
                .remove(index);
            return true;
        }
        index -= pre_count;
        let import_count = self.import_rules.borrow().len();
        if index < import_count {
            let rule = self.import_rules.borrow()[index].clone();
            self.NotifyRuleChanged(rule.clone());
            if let StyleRuleBase::Import(import) = rule.as_ref() {
                B::ClearImportParentStyleSheet(import);
            }
            self.import_rules.borrow_mut().remove(index);
            return true;
        }
        index -= import_count;
        let namespace_count = self.namespace_rules.borrow().len();
        if index < namespace_count {
            let rule = self.namespace_rules.borrow()[index].clone();
            self.NotifyRuleChanged(rule);
            if !self.child_rules.borrow().is_empty() {
                return false;
            }
            self.namespace_rules.borrow_mut().remove(index);
            return true;
        }
        index -= namespace_count;
        let rule = self.child_rules.borrow()[index].clone();
        self.NotifyRuleChanged(rule.clone());
        if let StyleRuleBase::FontFace(font) = rule.as_ref() {
            self.NotifyRemoveFontFaceRule(font);
        }
        self.child_rules.borrow_mut().remove(index);
        true
    }
    // cc:419-433
    pub fn ParserAddNamespace(&self, prefix: AtomicString, uri: AtomicString) {
        debug_assert!(!uri.IsNull());
        if prefix.IsNull() {
            *self.default_namespace.borrow_mut() = uri;
            return;
        }
        self.namespaces.borrow_mut().insert(prefix, uri);
    }
    pub fn NamespaceURIFromPrefix(&self, prefix: &AtomicString) -> AtomicString {
        self.namespaces
            .borrow()
            .get(prefix)
            .cloned()
            .unwrap_or_default()
    }
    // cc:435-472. CSSParser's ParseSheet delegates to CSSParserImpl::ParseStyleSheet.
    pub fn ParseAuthorStyleSheet(self: &Rc<Self>, resource: &B::Resource) {
        B::TraceParseAuthorStyleSheet(resource);
        let check = if IsQuirksModeBehavior(self.parser_context.Mode())
            && B::ResourceIsCorsSameOrigin(resource)
        {
            MIMETypeCheck::kLax
        } else {
            MIMETypeCheck::kStrict
        };
        let text = B::ResourceSheetText(resource, &self.parser_context, check);
        let mut map = B::ResourceSourceMapHeader(resource);
        if map.empty() {
            map = B::ResourceDeprecatedSourceMapHeader(resource);
        }
        *self.source_map_url.borrow_mut() = map;
        self.ParseString(&text, true, CSSDeferPropertyParsing::kYes);
    }
    pub fn ParseString(
        self: &Rc<Self>,
        text: &String,
        allow_import_rules: bool,
        defer: CSSDeferPropertyParsing,
    ) -> ParseSheetResult {
        let context = CSSParserContext::FromStyleSheetOwner(&self.parser_context, self.as_ref());
        CSSParserImpl::<B::ParserBackend>::ParseStyleSheet(
            text,
            &context,
            &mut self.clone(),
            defer,
            allow_import_rules,
        )
    }
    // cc:474-491
    pub fn IsLoading(&self) -> bool {
        self.import_rules
            .borrow()
            .iter()
            .any(|rule| match rule.as_ref() {
                StyleRuleBase::Import(import) => B::ImportIsLoading(import),
                _ => unreachable!(),
            })
    }
    pub fn LoadCompleted(self: &Rc<Self>) -> bool {
        if let Some(parent) = self.ParentStyleSheet() {
            return parent.LoadCompleted();
        }
        self.RootStyleSheet().loading_clients.borrow().len() == 0
    }
    // cc:493-535
    pub fn CheckLoaded(self: &Rc<Self>) {
        if self.IsLoading() {
            return;
        }
        if let Some(parent) = self.ParentStyleSheet() {
            parent.CheckLoaded();
            return;
        }
        let loading_clients = self.loading_clients.borrow().snapshot();
        for client in loading_clients {
            if B::ClientLoadCompleted(&client) {
                continue;
            }
            debug_assert!(!B::IsConstructed(&client));
            if let Some(node) = B::OwnerNode(&client) {
                if B::SheetLoaded(&client) {
                    B::NotifyLoadedSheetAndAllCriticalSubresources(&node, self.DidLoadErrorOccur());
                }
            }
        }
    }
    // cc:537-561
    pub fn NotifyLoadedSheet(self: &Rc<Self>, resource: &B::Resource) {
        self.did_load_error_occur
            .set(self.DidLoadErrorOccur() || B::ResourceErrorOccurred(resource));
        self.ClearRuleSet();
    }
    pub fn SetToPendingState(self: &Rc<Self>) {
        let root = self.RootStyleSheet();
        let loading = root.loading_clients.borrow().snapshot();
        for client in loading {
            B::SetClientToPendingState(&client);
        }
        // Chromium deliberately snapshots this sheet's completed clients.
        let completed = self.completed_clients.borrow().snapshot();
        for client in completed {
            B::SetClientToPendingState(&client);
        }
    }
    // cc:563-616,691-707
    pub fn ParentStyleSheet(&self) -> Option<Rc<Self>> {
        self.OwnerRule()
            .and_then(|rule| B::ImportParentStyleSheet(&rule))
    }
    pub fn RootStyleSheet(self: &Rc<Self>) -> Rc<Self> {
        let mut root = self.clone();
        while let Some(parent) = root.ParentStyleSheet() {
            root = parent;
        }
        root
    }
    pub fn HasSingleOwnerNode(self: &Rc<Self>) -> bool {
        self.RootStyleSheet().HasOneClient()
    }
    pub fn SingleOwnerNode(self: &Rc<Self>) -> Option<Rc<B::Node>> {
        let root = self.RootStyleSheet();
        if !root.HasOneClient() {
            return None;
        }
        root.ClientAny().and_then(|client| B::OwnerNode(&client))
    }
    pub fn SingleOwnerDocument(self: &Rc<Self>) -> Option<Rc<B::Document>> {
        self.RootStyleSheet().ClientSingleOwnerDocument()
    }
    pub fn SingleOwnerDocumentForUseCounter(
        contents: Option<&Rc<Self>>,
    ) -> Option<Rc<B::Document>> {
        contents
            .filter(|contents| contents.HasSingleOwnerNode())
            .and_then(|contents| contents.SingleOwnerDocument())
    }
    pub fn AnyOwnerDocument(self: &Rc<Self>) -> Option<Rc<B::Document>> {
        self.RootStyleSheet().ClientAnyOwnerDocument()
    }
    pub fn ClientInTreeScope(
        self: &Rc<Self>,
        scope: &B::TreeScope,
    ) -> Option<Rc<B::CSSStyleSheet>> {
        let root = self.RootStyleSheet();
        let completed = root.completed_clients.borrow().snapshot();
        let loading = root.loading_clients.borrow().snapshot();
        completed.into_iter().chain(loading).find(|client| {
            B::IsAdoptedByTreeScope(client, scope)
                || B::OwnerNode(client).is_some_and(|node| B::NodeTreeScope(&node) == scope)
        })
    }
    fn ClientAny(&self) -> Option<Rc<B::CSSStyleSheet>> {
        self.loading_clients
            .borrow()
            .snapshot()
            .into_iter()
            .next()
            .or_else(|| {
                self.completed_clients
                    .borrow()
                    .snapshot()
                    .into_iter()
                    .next()
            })
    }
    fn ClientAnyOwnerDocument(&self) -> Option<Rc<B::Document>> {
        self.ClientAny()
            .and_then(|client| B::OwnerDocument(&client))
    }
    fn ClientSingleOwnerDocument(&self) -> Option<Rc<B::Document>> {
        if self.HasSingleOwnerDocument() {
            self.ClientAnyOwnerDocument()
        } else {
            None
        }
    }
    // cc:709-764
    pub fn RegisterClient(&self, client: &Rc<B::CSSStyleSheet>) {
        debug_assert!(!self.loading_clients.borrow().contains(client));
        debug_assert!(!self.completed_clients.borrow().contains(client));
        let Some(owner) = B::OwnerDocument(client) else {
            return;
        };
        if let Some(document) = self.ClientSingleOwnerDocument() {
            if !Rc::ptr_eq(&owner, &document) {
                self.has_single_owner_document.set(false);
            }
        }
        if B::IsConstructed(client) {
            self.completed_clients.borrow_mut().insert(client);
        } else {
            self.loading_clients.borrow_mut().insert(client);
        }
    }
    pub fn UnregisterClient(&self, client: &Rc<B::CSSStyleSheet>) {
        self.loading_clients.borrow_mut().erase(client);
        self.completed_clients.borrow_mut().erase(client);
        if B::OwnerDocument(client).is_none() || self.ClientSize() > 0 {
            return;
        }
        self.has_single_owner_document.set(true);
    }
    pub fn ClientSize(&self) -> usize {
        self.loading_clients.borrow().len() + self.completed_clients.borrow().len()
    }
    pub fn HasOneClient(&self) -> bool {
        self.ClientSize() == 1
    }
    pub fn ClientLoadCompleted(&self, client: &Rc<B::CSSStyleSheet>) {
        debug_assert!(
            self.loading_clients.borrow().contains(client) || B::OwnerDocument(client).is_none()
        );
        self.loading_clients.borrow_mut().erase(client);
        if B::OwnerDocument(client).is_none() {
            return;
        }
        self.completed_clients.borrow_mut().insert(client);
    }
    pub fn ClientLoadStarted(&self, client: &Rc<B::CSSStyleSheet>) {
        debug_assert!(self.completed_clients.borrow().contains(client));
        self.completed_clients.borrow_mut().erase(client);
        self.loading_clients.borrow_mut().insert(client);
    }
    // cc:766-778
    pub fn IsReferencedFromResource(&self) -> bool {
        self.referenced_from_resource
            .borrow()
            .as_ref()
            .and_then(Weak::upgrade)
            .is_some()
    }
    pub fn SetReferencedFromResource(self: &Rc<Self>, resource: &Rc<B::Resource>) {
        debug_assert!(!self.IsReferencedFromResource());
        debug_assert!(self.IsCacheableForResource());
        *self.referenced_from_resource.borrow_mut() = Some(Rc::downgrade(resource));
    }
    pub fn ClearReferencedFromResource(self: &Rc<Self>) {
        debug_assert!(self.IsReferencedFromResource());
        debug_assert!(self.IsCacheableForResource());
        self.referenced_from_resource.borrow_mut().take();
    }
    // cc:781-876
    pub fn ExtractMixins(&self, medium: &B::MediaQueryEvaluator) -> Ref<'_, MixinMap<B>> {
        if self.has_cached_mixins.get()
            && !B::DidResultsChange(medium, &self.mixins.borrow().media_query_set_results)
        {
            return self.mixins.borrow();
        }
        let mut mixins = MixinMap::new();
        ExtractMixinsFromSheet(self, medium, &mut mixins);
        *self.mixins.borrow_mut() = mixins;
        self.has_cached_mixins.set(true);
        self.mixins.borrow()
    }
    // cc:878-912
    pub fn GetRuleSet(&self) -> RuleSetHandle<B> {
        self.rule_set
            .borrow()
            .as_ref()
            .expect("StyleSheetContents must have RuleSet")
            .clone()
    }
    pub fn HasRuleSet(&self) -> bool {
        self.rule_set.borrow().is_some()
    }
    pub fn EnsureRuleSet(
        &self,
        medium: &B::MediaQueryEvaluator,
        mixins: &MixinMap<B>,
    ) -> RuleSetHandle<B> {
        let previous = self.rule_set.borrow().clone();
        if previous.as_ref().is_some_and(|set| {
            set.borrow()
                .DependingOnOutdatedMixins(mixins.map_identifier)
        }) {
            self.rule_set.borrow_mut().take();
            self.with_diff(B::MarkDiffUnrepresentable);
        }
        let previous = self.rule_set.borrow().clone();
        if previous.as_ref().is_some_and(|set| {
            set.borrow().DidMediaQueryResultsChange(medium)
                || set
                    .borrow()
                    .DidRoutesChange(B::MediaEvaluatorDocument(medium))
        }) {
            self.rule_set.borrow_mut().take();
        }
        self.with_diff(B::NewRuleSetCleared);
        if !self.HasRuleSet() {
            let rules = Rc::new(RefCell::new(RuleSet::new()));
            *self.rule_set.borrow_mut() = Some(rules.clone());
            B::AddRulesFromSheet(&mut rules.borrow_mut(), self, medium, mixins);
            self.with_diff(|diff| B::NewRuleSetCreated(diff, rules.clone()));
            B::CompactRulesIfNeeded(&mut rules.borrow_mut());
        }
        self.GetRuleSet()
    }
    pub fn CreateUnconnectedRuleSet(
        &self,
        medium: &B::MediaQueryEvaluator,
        mixins: &MixinMap<B>,
    ) -> RuleSet<B> {
        let mut rules = RuleSet::new();
        B::AddRulesFromSheet(&mut rules, self, medium, mixins);
        B::CompactRulesIfNeeded(&mut rules);
        rules
    }
    // h:150-164; cc:926-961
    fn with_diff(&self, operation: impl FnOnce(&mut B::RuleSetDiff)) {
        if let Some(diff) = self.rule_set_diff.borrow_mut().as_mut() {
            operation(diff);
        }
    }
    pub fn NotifyRuleChanged(&self, rule: Rc<StyleRuleBase<B>>) {
        self.with_diff(|diff| B::AddRuleDiff(diff, rule));
    }
    pub fn NotifyDiffUnrepresentable(&self) {
        self.with_diff(B::MarkDiffUnrepresentable);
        self.has_cached_mixins.set(false);
    }
    pub fn GetRuleSetDiff(&self) -> Ref<'_, Option<B::RuleSetDiff>> {
        self.rule_set_diff.borrow()
    }
    pub fn ClearRuleSetDiff(&self) {
        self.rule_set_diff.borrow_mut().take();
    }
    pub fn StartMutation(&self) {
        self.is_mutable.set(true);
        if let Some(set) = self.rule_set.borrow().clone() {
            *self.rule_set_diff.borrow_mut() = Some(B::NewRuleSetDiff(set));
        }
    }
    pub fn ClearRuleSet(self: &Rc<Self>) {
        self.has_cached_mixins.set(false);
        if let Some(parent) = self.ParentStyleSheet() {
            parent.ClearRuleSet();
        }
        let Some(set) = self.rule_set.borrow().clone() else {
            return;
        };
        if set.borrow().DependingOnMixins() {
            self.with_diff(B::MarkDiffUnrepresentable);
        }
        self.rule_set.borrow_mut().take();
        self.with_diff(B::NewRuleSetCleared);
        let loading = self.loading_clients.borrow().snapshot();
        let completed = self.completed_clients.borrow().snapshot();
        SetNeedsActiveStyleUpdateForClients::<B>(&loading);
        SetNeedsActiveStyleUpdateForClients::<B>(&completed);
    }
    // cc:963-978
    fn NotifyRemoveFontFaceRule(self: &Rc<Self>, font: &StyleRuleFontFace<B>) {
        let root = self.RootStyleSheet();
        let loading = root.loading_clients.borrow().snapshot();
        let completed = root.completed_clients.borrow().snapshot();
        for client in loading.into_iter().chain(completed) {
            if let Some(node) = B::OwnerNode(&client) {
                B::RemoveFontFaceRule(&B::NodeDocument(&node), font);
            }
        }
    }
    // cc:618-689
    pub fn HasFailedOrCanceledSubresources(self: &Rc<Self>) -> bool {
        debug_assert!(self.IsCacheableForResource());
        ChildRulesHaveFailedOrCanceledSubresources::<B>(&self.child_rules.borrow())
    }
}
impl<B: StyleSheetContentsBackend + StyleRuleCloneDependencies> StyleSheetContents<B>
where
    B::CSSPropertyValueSet: StyleRulePropertySetClone,
{
    // cc:54-70
    pub fn EstimatedSizeInBytes(&self) -> usize {
        let mut size =
            std::mem::size_of::<Self>() + self.RuleCount() * StyleRule::<B>::AverageSizeInBytes();
        for rule in self.import_rules.borrow().iter() {
            if let StyleRuleBase::Import(import) = rule.as_ref() {
                if let Some(sheet) = B::ImportStyleSheet(import) {
                    size += sheet.EstimatedSizeInBytes();
                }
            }
        }
        size
    }
    // cc:88-128. Imports cannot be copied in Chromium's implementation.
    pub fn Copy(&self) -> Rc<Self> {
        assert!(
            self.import_rules.borrow().is_empty(),
            "Chromium StyleSheetContents copy requires no @import rules"
        );
        let result = Self::new(self.parser_context.clone(), self.original_url.clone(), None);
        *result.pre_import_layer_statement_rules.borrow_mut() = self
            .pre_import_layer_statement_rules
            .borrow()
            .iter()
            .map(|r| r.Clone(None, None))
            .collect();
        *result.namespace_rules.borrow_mut() = self
            .namespace_rules
            .borrow()
            .iter()
            .map(|rule| {
                let StyleRuleBase::Namespace(namespace) = rule.as_ref() else {
                    unreachable!()
                };
                Rc::new(StyleRuleBase::Namespace(B::CopyNamespace(namespace)))
            })
            .collect();
        *result.child_rules.borrow_mut() = self
            .child_rules
            .borrow()
            .iter()
            .map(|r| r.Clone(None, None))
            .collect();
        *result.namespaces.borrow_mut() = self.namespaces.borrow().clone();
        *result.default_namespace.borrow_mut() = self.DefaultNamespace();
        result
            .has_syntactically_valid_css_header
            .set(self.HasSyntacticallyValidCSSHeader());
        result.has_font_face_rule.set(self.HasFontFaceRule());
        result.has_media_queries.set(self.HasMediaQueries());
        result
    }
}
impl<B: StyleSheetContentsBackend> SingleOwnerDocument<B::Platform> for StyleSheetContents<B> {
    // CSSParserContext's static helper additionally requires a single client.
    fn SingleOwnerDocument(&self) -> Option<DocumentHandle<B::Platform>> {
        let mut parent = self.ParentStyleSheet();
        let mut root = None;
        while let Some(sheet) = parent {
            parent = sheet.ParentStyleSheet();
            root = Some(sheet);
        }
        let sheet = root.as_deref().unwrap_or(self);
        if !sheet.HasOneClient() {
            return None;
        }
        sheet
            .ClientSingleOwnerDocument()
            .map(|document| B::ParserDocument(&document))
    }
}

// cc:618-684. The source intentionally skips @supports and style-rule children.
fn ChildRulesHaveFailedOrCanceledSubresources<B: StyleSheetContentsBackend>(
    rules: &[Rc<StyleRuleBase<B>>],
) -> bool {
    for rule in rules {
        match rule.as_ref() {
            StyleRuleBase::Style(style) => {
                if style.PropertiesHaveFailedOrCanceledSubresources() {
                    return true;
                }
            }
            StyleRuleBase::FontFace(font) => {
                if font.Properties().HasFailedOrCanceledSubresources() {
                    return true;
                }
            }
            StyleRuleBase::Container(_)
            | StyleRuleBase::Media(_)
            | StyleRuleBase::LayerBlock(_)
            | StyleRuleBase::Navigation(_)
            | StyleRuleBase::Scope(_)
            | StyleRuleBase::StartingStyle(_) => {
                if ChildRulesHaveFailedOrCanceledSubresources::<B>(
                    rule.AsGroup().expect("group rule").ChildRules(),
                ) {
                    return true;
                }
            }
            StyleRuleBase::CounterStyle(counter) => {
                if B::CounterStyleHasFailedOrCanceledSubresources(counter) {
                    return true;
                }
            }
            StyleRuleBase::Charset(_)
            | StyleRuleBase::Import(_)
            | StyleRuleBase::Namespace(_)
            | StyleRuleBase::Mixin(_) => unreachable!("invalid child rule in cacheable stylesheet"),
            _ => {}
        }
    }
    false
}
// cc:781-793
fn MatchMediaForMixins<B: StyleSheetContentsBackend>(
    medium: &B::MediaQueryEvaluator,
    queries: Option<Rc<B::MediaQuerySet>>,
    flags: &mut MediaQueryResultFlags,
    results: &mut Vec<MediaQuerySetResult>,
) -> bool {
    let Some(queries) = queries else {
        return true;
    };
    let matched = B::EvalMedia(medium, &queries, flags);
    results.push(MediaQuerySetResult::new(queries, matched));
    matched
}
// cc:797-842
fn ExtractMixinsFromRules<B: StyleSheetContentsBackend>(
    rules: &[Rc<StyleRuleBase<B>>],
    medium: &B::MediaQueryEvaluator,
    mut mixins: Option<&mut MixinMap<B>>,
) -> bool {
    let mut found = false;
    for rule in rules {
        match rule.as_ref() {
            StyleRuleBase::Media(media) => {
                let mut flags = MediaQueryResultFlags::default();
                let mut results = vec![];
                let matched = MatchMediaForMixins::<B>(
                    medium,
                    B::MediaQueriesForRule(media),
                    &mut flags,
                    &mut results,
                );
                if ExtractMixinsFromRules::<B>(
                    media.ChildRules(),
                    medium,
                    if matched { mixins.as_deref_mut() } else { None },
                ) {
                    found |= matched;
                    if let Some(map) = mixins.as_deref_mut() {
                        map.media_query_result_flags.Add(&flags);
                        map.media_query_set_results.extend(results);
                    }
                }
            }
            StyleRuleBase::Supports(supports) if supports.ConditionIsSupported() => {
                found |= ExtractMixinsFromRules::<B>(
                    supports.ChildRules(),
                    medium,
                    mixins.as_deref_mut(),
                );
            }
            StyleRuleBase::Mixin(mixin) => {
                if let Some(map) = mixins.as_deref_mut() {
                    map.mixins.insert(mixin.GetName().clone(), rule.clone());
                }
                found = true;
            }
            _ => {}
        }
        if found && mixins.is_none() {
            return true;
        }
    }
    found
}
// cc:844-865
fn ExtractMixinsFromSheet<B: StyleSheetContentsBackend>(
    sheet: &StyleSheetContents<B>,
    medium: &B::MediaQueryEvaluator,
    mixins: &mut MixinMap<B>,
) -> bool {
    let mut found = false;
    for rule in sheet.import_rules.borrow().iter() {
        let StyleRuleBase::Import(import) = rule.as_ref() else {
            unreachable!()
        };
        let Some(child) = B::ImportStyleSheet(import) else {
            continue;
        };
        if !B::ImportIsSupported(import) {
            continue;
        }
        if !MatchMediaForMixins::<B>(
            medium,
            B::ImportMediaQueries(import),
            &mut mixins.media_query_result_flags,
            &mut mixins.media_query_set_results,
        ) {
            continue;
        }
        found |= ExtractMixinsFromSheet(&child, medium, mixins);
    }
    found |= ExtractMixinsFromRules::<B>(&sheet.child_rules.borrow(), medium, Some(mixins));
    found
}
// cc:914-924
fn SetNeedsActiveStyleUpdateForClients<B: StyleSheetContentsBackend>(
    clients: &[Rc<B::CSSStyleSheet>],
) {
    for client in clients {
        let Some(document) = B::OwnerDocument(client) else {
            continue;
        };
        let Some(node) = B::OwnerNode(client) else {
            continue;
        };
        if !B::NodeIsConnected(&node) {
            continue;
        }
        B::SetNeedsActiveStyleUpdate(&document, B::NodeTreeScope(&node));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn weak_clients_preserve_identity_and_snapshot_during_load_callbacks() {
        let first = Rc::new(7);
        let same_value_other_client = Rc::new(7);
        let mut loading = WeakClientSet::new();
        let mut completed = WeakClientSet::new();
        loading.insert(&first);
        loading.insert(&first);
        loading.insert(&same_value_other_client);
        assert_eq!(loading.len(), 2);
        let snapshot = loading.snapshot();
        for client in &snapshot {
            loading.erase(client);
            completed.insert(client);
        }
        assert_eq!(loading.len(), 0);
        assert_eq!(completed.len(), 2);
        drop(first);
        drop(same_value_other_client);
        assert_eq!(completed.len(), 2); // Snapshot protects clients from scripts.
        drop(snapshot);
        assert_eq!(completed.len(), 0);
        assert!(completed.snapshot().is_empty());
    }
}
