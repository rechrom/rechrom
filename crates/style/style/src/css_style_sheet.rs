/*
 * (C) 1999-2003 Lars Knoll (knoll@kde.org)
 * Copyright (C) 2004, 2006, 2007, 2012 Apple Inc. All rights reserved.
 * This library is free software under the GNU Library General Public License,
 * version 2 or (at your option) any later version. See COPYING.LIB.
 */
// cpp: third_party/blink/renderer/core/css/css_style_sheet.h
// cpp: third_party/blink/renderer/core/css/css_style_sheet.cc
// Source: /Users/zhenghuaiyu/chromium/src at
// 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
// Source ledger: effective excludes blank/comment/preprocessor/include/
// namespace/pure-brace lines. Production declarations and continuation lines
// count; omitted is the effective subset of the exact exclusions below.
// css_style_sheet.h: physical 368; effective 216; mapped 182; omitted 34;
// pending 0. Mapped .h:43-362, except the exclusions below and the effective
// filter. Public API, inline accessors and mutation-scope bodies are included.
// Omitted .h:51,54,68,102-104,208,213-214,232,236-237,262,265-266,288-306:
// friend-only forward declarations/visibility, wrapper/stack/test macros,
// deleted copy operations, default sheet destructor and GC Trace declaration.
// RuleMutationScope/InspectorMutationScope destructors are mapped, not omitted.
// css_style_sheet.cc: physical 769; effective 469; mapped 434; omitted 35;
// pending 0. Mapped .cc:63-81,96-753, except the exclusions below and the
// effective filter. All CSSStyleSheet method bodies in those ranges are mapped.
// Omitted .cc:67-70,84-92,140,227,232,242-243,253-254,269,404,439-440,
// 498-499,628,635,755-767: GC Trace, debug-only parent checker/DCHECKs and
// default sheet destructor. Production CHECK_NE/CHECK_GT remain assertions.
// DOM/CSSOM/platform/parser/probe/style-engine dependency bodies are required
// typed trait calls with no defaults; their files are not claimed translated.
// Ownership follows Member/WeakMember for nodes, documents and adopted scopes.
// CSSOM caches are weak while their wrappers retain the owner strongly: all
// observable wrapper identities survive, and cache-only Rc cycles are avoided.
// StyleSheetContents and StyleRuleBase are the existing implementations.

#![allow(non_snake_case, non_camel_case_types)]

use crate::css_resource_fetch_restriction::ResourceFetchRestriction;
use crate::media_queries::media_query_exp::MediaQueryExpValue;
use crate::parser::css_nesting_type::CSSNestingType;
use crate::parser::css_parser_context::{
    CSSParserContext, CSSParserContextPlatform, DocumentHandle, DocumentSnapshot,
    SingleOwnerDocument,
};
use crate::parser::css_parser_impl::ParseSheetResult;
use crate::parser::css_parser_mode::CSSDeferPropertyParsing;
use crate::resolver::media_query_result::MediaQueryResultFlags;
use crate::style_rule::{
    StyleRule, StyleRuleBase, StyleRuleCloneDependencies, StyleRulePropertySetClone,
};
use crate::style_sheet_contents::{StyleSheetContents, StyleSheetContentsBackend};
use foundation::String;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

pub type SheetURL<B> =
    <<B as StyleSheetContentsBackend>::Platform as CSSParserContextPlatform>::URL;
pub type SheetTextEncoding<B> =
    <<B as StyleSheetContentsBackend>::Platform as CSSParserContextPlatform>::TextEncoding;

// cpp: css_style_sheet.h:61-64
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CSSImportRules {
    kAllow,
    kIgnoreWithWarning,
}
// cpp: css_style_sheet.h:223-229
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mutation {
    kSheet,
    kRules,
}

/// DOMExceptionCode values used by this source pair. Security exceptions use
/// the separate ThrowSecurityError operation, matching ExceptionState.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CSSStyleSheetExceptionCode {
    kNotAllowedError,
    kSyntaxError,
    kInvalidStateError,
    kHierarchyRequestError,
    kIndexSizeError,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CSSStyleSheetOwnerKind {
    kHTMLLink,
    kHTMLStyle,
    kSVGStyle,
    kProcessingInstruction,
    kOther,
}
pub enum CSSStyleSheetInitMedia<B: CSSStyleSheetBackend> {
    MediaList(Rc<B::MediaQuerySet>),
    String(String),
}

/// Actual CSSRule operations used by CSSStyleSheet; its implementation owns
/// the existing rule payload and retains its parent strongly while attached.
pub trait CSSStyleSheetRule<B: CSSStyleSheetBackend> {
    fn parentStyleSheet(&self) -> Option<Rc<CSSStyleSheet<B>>>;
    fn SetParentStyleSheet(&self, sheet: Option<Rc<CSSStyleSheet<B>>>);
    fn Reattach(&self, rule: Rc<StyleRuleBase<B>>);
}

/// Only calls owned by DOM, platform, CSSOM, parser, style-engine or bindings
/// classes enter this boundary. Sheet decisions and mutation logic remain here.
/// Contents' CSSStyleSheet client is exactly this type, not a parallel model.
pub trait CSSStyleSheetBackend:
    StyleSheetContentsBackend<
        CSSStyleSheet = CSSStyleSheet<Self>,
        Document: 'static,
        TreeScope: 'static,
    > + StyleRuleCloneDependencies<CSSPropertyValueSet: StyleRulePropertySetClone>
{
    type CSSRule: CSSStyleSheetRule<Self> + 'static;
    type CSSImportRule;
    type CSSStyleSheetInit;
    type Element: 'static;
    type MediaList: 'static;
    type TextPosition: Clone;
    type ExceptionState;
    type ScriptState;
    type ScriptPromise;

    fn CSSStyleSheetInitBaseURLEnabled() -> bool;
    fn ConstructableStylesheetCacheEnabled() -> bool;
    fn InitBaseURL(options: &Self::CSSStyleSheetInit) -> String;
    fn InitMedia(options: &Self::CSSStyleSheetInit) -> CSSStyleSheetInitMedia<Self>;
    fn InitAlternate(options: &Self::CSSStyleSheetInit) -> bool;
    fn InitDisabled(options: &Self::CSSStyleSheetInit) -> bool;
    fn DocumentParserSnapshot(document: &Rc<Self::Document>) -> DocumentSnapshot<Self::Platform>;
    fn IsAdScriptExecutingInDocument(document: &Self::Document) -> bool;
    fn IsURLValid(url: &SheetURL<Self>) -> bool;
    fn IsURLNull(url: &SheetURL<Self>) -> bool;
    fn URLString(url: &SheetURL<Self>) -> String;
    fn ClientReferrerString() -> String;
    fn DefaultReferrerPolicy() -> <Self::Platform as CSSParserContextPlatform>::ReferrerPolicy;
    fn MinimumTextPosition() -> Self::TextPosition;
    fn ParentOrShadowHostElement(node: &Self::Node) -> Option<Rc<Self::Element>>;
    fn NodeAsElement(node: &Self::Node) -> Option<Rc<Self::Element>>;
    fn ElementRelAttribute(element: &Self::Element) -> String;
    fn NodeOwnerKind(node: &Self::Node) -> CSSStyleSheetOwnerKind;
    fn NodeIsInShadowTree(node: &Self::Node) -> bool;
    fn LinkIsEnabledViaScript(node: &Self::Node) -> bool;
    fn NodeSheetLoaded(node: &Self::Node) -> bool;
    fn NodeSetToPendingState(node: &Self::Node);
    fn DocumentIsActive(document: &Self::Document) -> bool;
    fn TreeScopeRootIsConnected(scope: &Self::TreeScope) -> bool;
    fn InvalidateMatchedPropertiesCache(document: &Self::Document);
    fn DidMutateStyleSheet(document: &Self::Document, sheet: &CSSStyleSheet<Self>);
    fn DidReplaceStyleSheetText(
        document: Option<&Self::Document>,
        sheet: &CSSStyleSheet<Self>,
        text: &String,
    );
    fn FindStyleSheetContents(
        document: &Self::Document,
        text: &String,
        context: &CSSParserContext<Self::Platform>,
    ) -> Option<Rc<StyleSheetContents<Self>>>;
    fn AddStyleSheetContents(
        document: &Self::Document,
        text: &String,
        contents: Rc<StyleSheetContents<Self>>,
    );
    fn AddJavaScriptWarning(document: &Self::Document, message: &String);
    fn CreateMediaQuerySet() -> Rc<Self::MediaQuerySet>;
    fn ParseMediaQuerySet(text: &String, document: &Self::Document) -> Rc<Self::MediaQuerySet>;
    fn EvaluateMediaQueries(
        evaluator: &Self::MediaQueryEvaluator,
        queries: &Self::MediaQuerySet,
        flags: &mut MediaQueryResultFlags,
    ) -> bool;
    /// MediaList retains its owner strongly and reads/writes MediaQueries live.
    fn NewMediaList(owner: Rc<CSSStyleSheet<Self>>) -> Rc<Self::MediaList>;
    /// Upcast preserves the import rule's allocation and CSSOM identity.
    fn CSSImportRuleAsCSSRule(rule: Rc<Self::CSSImportRule>) -> Rc<Self::CSSRule>;
    fn CreateCSSOMWrapper(
        rule: Rc<StyleRuleBase<Self>>,
        index: usize,
        sheet: Rc<CSSStyleSheet<Self>>,
        trigger_use_counters: bool,
    ) -> Rc<Self::CSSRule>;
    fn ParseRule(
        context: &CSSParserContext<Self::Platform>,
        contents: &Rc<StyleSheetContents<Self>>,
        nesting: CSSNestingType,
        parent_rule_for_nesting: Option<&StyleRule<Self>>,
        text: &String,
    ) -> Option<Rc<StyleRuleBase<Self>>>;
    fn ThrowDOMException(
        state: &mut Self::ExceptionState,
        code: CSSStyleSheetExceptionCode,
        message: &String,
    );
    fn ThrowSecurityError(state: &mut Self::ExceptionState, message: &String);
    fn EmptyPromise() -> Self::ScriptPromise;
    fn ToResolvedPromise(
        state: &Self::ScriptState,
        sheet: Rc<CSSStyleSheet<Self>>,
    ) -> Self::ScriptPromise;
}

// cpp: css_style_sheet.cc:63-81 (Trace omitted). This is a live view, not a
// snapshot, and retains the CSSStyleSheet while externally held.
pub struct StyleSheetCSSRuleList<B: CSSStyleSheetBackend> {
    style_sheet_: Rc<CSSStyleSheet<B>>,
}
impl<B: CSSStyleSheetBackend> StyleSheetCSSRuleList<B> {
    pub fn new(sheet: Rc<CSSStyleSheet<B>>) -> Rc<Self> {
        Rc::new(Self {
            style_sheet_: sheet,
        })
    }
    pub fn length(&self) -> usize {
        self.style_sheet_.length()
    }
    pub fn Item(&self, index: usize, trigger_use_counters: bool) -> Option<Rc<B::CSSRule>> {
        self.style_sheet_.item(index, trigger_use_counters)
    }
    pub fn GetStyleSheet(&self) -> Rc<CSSStyleSheet<B>> {
        self.style_sheet_.clone()
    }
}

// cpp: css_style_sheet.h:207-219,341-357
pub struct RuleMutationScope<B: CSSStyleSheetBackend> {
    style_sheet_: Option<Rc<CSSStyleSheet<B>>>,
}
impl<B: CSSStyleSheetBackend> RuleMutationScope<B> {
    pub fn new(sheet: &Rc<CSSStyleSheet<B>>) -> Self {
        sheet.WillMutateRules();
        Self {
            style_sheet_: Some(sheet.clone()),
        }
    }
    pub fn FromRule(rule: Option<&B::CSSRule>) -> Self {
        let style_sheet = rule.and_then(|rule| rule.parentStyleSheet());
        if let Some(sheet) = &style_sheet {
            sheet.WillMutateRules();
        }
        Self {
            style_sheet_: style_sheet,
        }
    }
}
impl<B: CSSStyleSheetBackend> Drop for RuleMutationScope<B> {
    fn drop(&mut self) {
        if let Some(sheet) = &self.style_sheet_ {
            sheet.DidMutate(Mutation::kRules);
        }
    }
}
// cpp: css_style_sheet.h:231-242; css_style_sheet.cc:303-311
pub struct InspectorMutationScope<B: CSSStyleSheetBackend> {
    style_sheet_: Rc<CSSStyleSheet<B>>,
}
impl<B: CSSStyleSheetBackend> InspectorMutationScope<B> {
    pub fn new(sheet: &Rc<CSSStyleSheet<B>>) -> Self {
        sheet.EnableRuleAccessForInspector();
        Self {
            style_sheet_: sheet.clone(),
        }
    }
}
impl<B: CSSStyleSheetBackend> Drop for InspectorMutationScope<B> {
    fn drop(&mut self) {
        self.style_sheet_.DisableRuleAccessForInspector();
    }
}

// cpp: css_style_sheet.h:310-338
pub struct CSSStyleSheet<B: CSSStyleSheetBackend> {
    contents_: RefCell<Rc<StyleSheetContents<B>>>,
    media_queries_: RefCell<Option<Rc<B::MediaQuerySet>>>,
    media_query_result_flags_: Cell<MediaQueryResultFlags>,
    title_: RefCell<String>,
    owner_node_: RefCell<Option<Rc<B::Node>>>,
    owner_parent_or_shadow_host_element_: Option<Weak<B::Element>>,
    owner_rule_: RefCell<Option<Rc<B::CSSRule>>>,
    adopted_tree_scopes_: RefCell<Vec<(Weak<B::TreeScope>, usize)>>,
    constructor_document_: RefCell<Option<Rc<B::Document>>>,
    start_position_: B::TextPosition,
    media_cssom_wrapper_: RefCell<Option<Weak<B::MediaList>>>,
    child_rule_cssom_wrappers_: RefCell<Vec<Option<Weak<B::CSSRule>>>>,
    rule_list_cssom_wrapper_: RefCell<Option<Weak<StyleSheetCSSRuleList<B>>>>,
    is_inline_stylesheet_: bool,
    is_for_css_module_script_: Cell<bool>,
    is_disabled_: Cell<bool>,
    load_completed_: Cell<bool>,
    alternate_from_constructor_: Cell<bool>,
    enable_rule_access_for_inspector_: Cell<bool>,
}

impl<B: CSSStyleSheetBackend> CSSStyleSheet<B> {
    // cpp: css_style_sheet.cc:96-102. Static use-counter helper requires exactly
    // one owner node/client, unlike Contents' ordinary instance document getter.
    pub fn SingleOwnerDocument(sheet: Option<&Self>) -> Option<Rc<B::Document>> {
        let contents = sheet.map(Self::Contents);
        StyleSheetContents::<B>::SingleOwnerDocumentForUseCounter(contents.as_ref())
    }
    // cpp: css_style_sheet.cc:104-121
    pub fn Create(
        document: Rc<B::Document>,
        options: &B::CSSStyleSheetInit,
        exception_state: &mut B::ExceptionState,
    ) -> Option<Rc<Self>> {
        if !B::CSSStyleSheetInitBaseURLEnabled() {
            let base = B::DocumentParserSnapshot(&document).base_url;
            return Some(Self::CreateWithBaseURL(document, base, options));
        }
        let base_url_text = B::InitBaseURL(options);
        let base = B::DocumentParserSnapshot(&document).base_url;
        if base_url_text.IsNull() {
            return Some(Self::CreateWithBaseURL(document, base, options));
        }
        let base_url = B::Platform::ResolveURL(&base, &base_url_text, None);
        if !B::IsURLValid(&base_url) {
            B::ThrowDOMException(
                exception_state,
                CSSStyleSheetExceptionCode::kNotAllowedError,
                &String::from("The 'baseURL' provided in CSSStyleSheetInit is invalid."),
            );
            return None;
        }
        Some(Self::CreateWithBaseURL(document, base_url, options))
    }
    // cpp: css_style_sheet.cc:123-135. The overload's ExceptionState is unused.
    pub fn CreateWithBaseURL(
        document: Rc<B::Document>,
        base_url: SheetURL<B>,
        options: &B::CSSStyleSheetInit,
    ) -> Rc<Self> {
        let parser_context = Rc::new(CSSParserContext::FromDocumentWithBaseURL(
            &B::DocumentParserSnapshot(&document),
            base_url,
        ));
        if B::IsAdScriptExecutingInDocument(&document) {
            parser_context.SetIsAdRelated();
        }
        let contents = StyleSheetContents::new(parser_context, String::default(), None);
        Self::newConstructed(contents, document, options)
    }
    // cpp: css_style_sheet.cc:137-143
    pub fn CreateInlineFromContents(
        contents: Rc<StyleSheetContents<B>>,
        owner_node: Rc<B::Node>,
        start_position: Option<B::TextPosition>,
    ) -> Rc<Self> {
        Self::newWithOwnerNode(contents, owner_node, true, start_position)
    }
    // cpp: css_style_sheet.cc:145-166
    pub fn InlineParserContext(
        document: &Rc<B::Document>,
        base_url: &SheetURL<B>,
        encoding: Option<SheetTextEncoding<B>>,
    ) -> Rc<CSSParserContext<B::Platform>> {
        let snapshot = B::DocumentParserSnapshot(document);
        let base = if B::IsURLNull(base_url) {
            snapshot.base_url.clone()
        } else {
            base_url.clone()
        };
        let referrer =
            B::Platform::MakeReferrer(B::ClientReferrerString(), B::DefaultReferrerPolicy());
        let parser_context = Rc::new(CSSParserContext::FromDocumentWithOptions(
            &snapshot,
            base,
            true,
            referrer,
            encoding.unwrap_or_else(B::Platform::EmptyTextEncoding),
            ResourceFetchRestriction::kNone,
        ));
        if B::IsAdScriptExecutingInDocument(document) {
            parser_context.SetIsAdRelated();
        }
        parser_context
    }
    // cpp: css_style_sheet.cc:168-179
    pub fn CreateInline(
        owner_node: Rc<B::Node>,
        base_url: &SheetURL<B>,
        start_position: Option<B::TextPosition>,
        encoding: Option<SheetTextEncoding<B>>,
    ) -> Rc<Self> {
        let document = B::NodeDocument(&owner_node);
        let parser_context = Self::InlineParserContext(&document, base_url, encoding);
        let contents = StyleSheetContents::new(parser_context, B::URLString(base_url), None);
        Self::newWithOwnerNode(contents, owner_node, true, start_position)
    }
    fn allocate(
        contents: Rc<StyleSheetContents<B>>,
        owner_node: Option<Rc<B::Node>>,
        owner_rule: Option<Rc<B::CSSRule>>,
        is_inline: bool,
        start_position: B::TextPosition,
    ) -> Rc<Self> {
        let owner_parent = owner_node
            .as_ref()
            .and_then(|node| B::ParentOrShadowHostElement(node))
            .map(|element| Rc::downgrade(&element));
        Rc::new(Self {
            contents_: RefCell::new(contents),
            media_queries_: RefCell::new(None),
            media_query_result_flags_: Cell::new(MediaQueryResultFlags::default()),
            title_: RefCell::new(String::default()),
            owner_node_: RefCell::new(owner_node),
            owner_parent_or_shadow_host_element_: owner_parent,
            owner_rule_: RefCell::new(owner_rule),
            adopted_tree_scopes_: RefCell::new(Vec::new()),
            constructor_document_: RefCell::new(None),
            start_position_: start_position,
            media_cssom_wrapper_: RefCell::new(None),
            child_rule_cssom_wrappers_: RefCell::new(Vec::new()),
            rule_list_cssom_wrapper_: RefCell::new(None),
            is_inline_stylesheet_: is_inline,
            is_for_css_module_script_: Cell::new(false),
            is_disabled_: Cell::new(false),
            load_completed_: Cell::new(false),
            alternate_from_constructor_: Cell::new(false),
            enable_rule_access_for_inspector_: Cell::new(false),
        })
    }
    // cpp: css_style_sheet.cc:181-187
    pub fn new(
        contents: Rc<StyleSheetContents<B>>,
        owner_rule: Option<Rc<B::CSSImportRule>>,
    ) -> Rc<Self> {
        let sheet = Self::allocate(
            contents,
            None,
            owner_rule.map(B::CSSImportRuleAsCSSRule),
            false,
            B::MinimumTextPosition(),
        );
        sheet.Contents().RegisterClient(&sheet);
        sheet
    }
    // cpp: css_style_sheet.cc:189-214
    pub fn newConstructed(
        contents: Rc<StyleSheetContents<B>>,
        document: Rc<B::Document>,
        options: &B::CSSStyleSheetInit,
    ) -> Rc<Self> {
        let sheet = Self::new(contents, None);
        sheet.SetConstructorDocument(document.clone());
        sheet.ClearOwnerNode();
        sheet.ClearOwnerRule();
        sheet.Contents().RegisterClient(&sheet);
        let queries = match B::InitMedia(options) {
            CSSStyleSheetInitMedia::MediaList(queries) => queries,
            CSSStyleSheetInitMedia::String(text) => B::ParseMediaQuerySet(&text, &document),
        };
        sheet.SetMediaQueries(Some(queries));
        if B::InitAlternate(options) {
            sheet.SetAlternateFromConstructor(true);
        }
        if B::InitDisabled(options) {
            sheet.setDisabled(true);
        }
        sheet
    }
    // cpp: css_style_sheet.cc:216-230
    pub fn newWithOwnerNode(
        contents: Rc<StyleSheetContents<B>>,
        owner_node: Rc<B::Node>,
        is_inline: bool,
        start_position: Option<B::TextPosition>,
    ) -> Rc<Self> {
        let sheet = Self::allocate(
            contents,
            Some(owner_node),
            None,
            is_inline,
            start_position.unwrap_or_else(B::MinimumTextPosition),
        );
        sheet.Contents().RegisterClient(&sheet);
        sheet
    }
    // cpp: css_style_sheet.cc:234-249
    pub fn WillMutateRules(self: &Rc<Self>) {
        if !self.IsContentsShared() {
            let contents = self.Contents();
            contents.StartMutation();
            contents.ClearRuleSet();
            return;
        }
        let copy = self.Contents().Copy();
        self.SetContents(copy);
        self.Contents().StartMutation();
    }
    // cpp: css_style_sheet.cc:251-283
    pub fn DidMutate(&self, mutation: Mutation) {
        let Some(document) = self.OwnerDocument() else {
            return;
        };
        if !B::DocumentIsActive(&document) {
            return;
        }
        let mut invalidate_matched_properties_cache = false;
        if let Some(owner) = self.ownerNode().filter(|owner| B::NodeIsConnected(owner)) {
            B::SetNeedsActiveStyleUpdate(&document, B::NodeTreeScope(&owner));
            invalidate_matched_properties_cache = true;
        } else {
            let scopes: Vec<_> = {
                let mut adopted = self.adopted_tree_scopes_.borrow_mut();
                adopted.retain(|(scope, _)| scope.strong_count() > 0);
                adopted
                    .iter()
                    .filter_map(|(scope, _)| scope.upgrade())
                    .collect()
            };
            for scope in scopes {
                if !B::TreeScopeRootIsConnected(&scope) {
                    continue;
                }
                B::SetNeedsActiveStyleUpdate(&document, &scope);
                invalidate_matched_properties_cache = true;
            }
        }
        if mutation == Mutation::kRules {
            if invalidate_matched_properties_cache {
                B::InvalidateMatchedPropertiesCache(&document);
            }
            B::DidMutateStyleSheet(&document, self);
        }
    }
    // cpp: css_style_sheet.cc:285-290
    pub fn EnableRuleAccessForInspector(&self) {
        self.enable_rule_access_for_inspector_.set(true);
    }
    pub fn DisableRuleAccessForInspector(&self) {
        self.enable_rule_access_for_inspector_.set(false);
    }
    // cpp: css_style_sheet.cc:292-301. QuietMutationScope is a separate owner.
    pub fn BeginQuietMutation(self: &Rc<Self>) {
        if self.IsContentsShared() {
            self.SetContents(self.Contents().Copy());
        }
    }
    pub fn EndQuietMutation(self: &Rc<Self>, original_contents: Rc<StyleSheetContents<B>>) {
        assert!(!Rc::ptr_eq(&self.Contents(), &original_contents));
        self.SetContents(original_contents);
    }
    // cpp: css_style_sheet.cc:313-317
    pub fn IsContentsShared(&self) -> bool {
        let contents = self.Contents();
        contents.IsUsedFromTextCache()
            || contents.IsUsedFromResourceCache()
            || contents.IsReferencedFromResource()
    }
    // cpp: css_style_sheet.cc:319-324
    pub fn SetContents(self: &Rc<Self>, contents: Rc<StyleSheetContents<B>>) {
        self.Contents().UnregisterClient(self);
        *self.contents_.borrow_mut() = contents;
        self.Contents().RegisterClient(self);
        self.ReattachChildRuleCSSOMWrappers();
    }
    // cpp: css_style_sheet.cc:326-333
    pub fn ReattachChildRuleCSSOMWrappers(&self) {
        let wrappers: Vec<_> = self
            .child_rule_cssom_wrappers_
            .borrow()
            .iter()
            .enumerate()
            .filter_map(|(i, wrapper)| {
                wrapper
                    .as_ref()
                    .and_then(Weak::upgrade)
                    .map(|wrapper| (i, wrapper))
            })
            .collect();
        for (index, wrapper) in wrappers {
            wrapper.Reattach(self.Contents().RuleAt(index));
        }
    }
    // cpp: css_style_sheet.cc:335-341
    pub fn DetachCSSOMWrappers(&self) {
        let wrappers: Vec<_> = self
            .child_rule_cssom_wrappers_
            .borrow()
            .iter()
            .filter_map(|wrapper| wrapper.as_ref().and_then(Weak::upgrade))
            .collect();
        for wrapper in wrappers {
            wrapper.SetParentStyleSheet(None);
        }
    }
    // cpp: css_style_sheet.cc:343-350
    pub fn setDisabled(&self, disabled: bool) {
        if disabled == self.is_disabled_.get() {
            return;
        }
        self.is_disabled_.set(disabled);
        self.DidMutate(Mutation::kSheet);
    }
    // cpp: css_style_sheet.cc:352-359
    pub fn MatchesMediaQueries(&self, evaluator: &B::MediaQueryEvaluator) -> bool {
        let mut flags = MediaQueryResultFlags::default();
        self.media_query_result_flags_.set(flags);
        let queries = self.MediaQueries();
        let Some(queries) = queries else {
            return true;
        };
        let result = B::EvaluateMediaQueries(evaluator, &queries, &mut flags);
        self.media_query_result_flags_.set(flags);
        result
    }
    // cpp: css_style_sheet.cc:361-380. Weak entries compare pointer identity,
    // not TreeScope structural equality, and duplicate adopted entries count.
    pub fn AddedAdoptedToTreeScope(&self, scope: &Rc<B::TreeScope>) {
        let mut adopted = self.adopted_tree_scopes_.borrow_mut();
        adopted.retain(|(scope, _)| scope.strong_count() > 0);
        let weak = Rc::downgrade(scope);
        if let Some((_, count)) = adopted.iter_mut().find(|(scope, _)| scope.ptr_eq(&weak)) {
            *count = count.wrapping_add(1);
        } else {
            adopted.push((weak, 1));
        }
    }
    pub fn RemovedAdoptedFromTreeScope(&self, scope: &B::TreeScope) {
        let mut adopted = self.adopted_tree_scopes_.borrow_mut();
        adopted.retain(|(scope, _)| scope.strong_count() > 0);
        let Some(index) = adopted
            .iter()
            .position(|(weak, _)| std::ptr::eq(weak.as_ptr(), scope))
        else {
            return;
        };
        assert!(adopted[index].1 > 0);
        adopted[index].1 -= 1;
        if adopted[index].1 == 0 {
            adopted.remove(index);
        }
    }
    pub fn IsAdoptedByTreeScope(&self, scope: &B::TreeScope) -> bool {
        self.adopted_tree_scopes_
            .borrow()
            .iter()
            .any(|(weak, _)| weak.strong_count() > 0 && std::ptr::eq(weak.as_ptr(), scope))
    }
    // cpp: css_style_sheet.cc:382-389
    pub fn HasViewportDependentMediaQueries(&self) -> bool {
        self.media_query_result_flags_.get().is_viewport_dependent
    }
    pub fn HasDynamicViewportDependentMediaQueries(&self) -> bool {
        self.media_query_result_flags_.get().unit_flags & MediaQueryExpValue::<()>::kDynamicViewport
            != 0
    }
    // cpp: css_style_sheet.cc:391-412
    pub fn length(&self) -> usize {
        self.Contents().RuleCount()
    }
    pub fn item(
        self: &Rc<Self>,
        index: usize,
        trigger_use_counters: bool,
    ) -> Option<Rc<B::CSSRule>> {
        let rule_count = self.length();
        if index >= rule_count {
            return None;
        }
        if self.child_rule_cssom_wrappers_.borrow().is_empty() {
            self.child_rule_cssom_wrappers_
                .borrow_mut()
                .resize_with(rule_count, || None);
        }
        if let Some(wrapper) = self.child_rule_cssom_wrappers_.borrow()[index]
            .as_ref()
            .and_then(Weak::upgrade)
        {
            return Some(wrapper);
        }
        let wrapper = B::CreateCSSOMWrapper(
            self.Contents().RuleAt(index),
            index,
            self.clone(),
            trigger_use_counters,
        );
        self.child_rule_cssom_wrappers_.borrow_mut()[index] = Some(Rc::downgrade(&wrapper));
        Some(wrapper)
    }
    // cpp: css_style_sheet.cc:414-420. Keep the weak owner-parent element.
    pub fn ClearOwnerNode(self: &Rc<Self>) {
        self.DidMutate(Mutation::kSheet);
        if self.ownerNode().is_some() {
            self.Contents().UnregisterClient(self);
        }
        self.owner_node_.borrow_mut().take();
    }
    // cpp: css_style_sheet.cc:422-428
    pub fn CanAccessRules(&self) -> bool {
        self.enable_rule_access_for_inspector_.get() || self.Contents().IsOriginClean()
    }
    pub fn rules(
        self: &Rc<Self>,
        exception_state: &mut B::ExceptionState,
    ) -> Option<Rc<StyleSheetCSSRuleList<B>>> {
        self.cssRules(exception_state)
    }
    // cpp: css_style_sheet.cc:430-488
    pub fn insertRule(
        self: &Rc<Self>,
        rule_string: &String,
        index: usize,
        exception_state: &mut B::ExceptionState,
    ) -> usize {
        if !self.CanAccessRules() {
            B::ThrowSecurityError(
                exception_state,
                &String::from("Cannot access StyleSheet to insertRule"),
            );
            return 0;
        }
        if index > self.length() {
            B::ThrowDOMException(
                exception_state,
                CSSStyleSheetExceptionCode::kIndexSizeError,
                &String::from(
                    format!(
                        "The index provided ({index}) is larger than the maximum index ({}).",
                        self.length()
                    )
                    .as_str(),
                ),
            );
            return 0;
        }
        let contents = self.Contents();
        let context =
            CSSParserContext::FromStyleSheetOwner(contents.ParserContext(), self.as_ref());
        let Some(rule) = B::ParseRule(
            &context,
            &contents,
            CSSNestingType::kNone,
            None,
            rule_string,
        ) else {
            let mut text: Vec<u16> = "Failed to parse the rule '".encode_utf16().collect();
            text.extend_from_slice(rule_string.Span16().unwrap_or_default());
            text.extend("'.".encode_utf16());
            B::ThrowDOMException(
                exception_state,
                CSSStyleSheetExceptionCode::kSyntaxError,
                &String::from_utf16(&text),
            );
            return 0;
        };
        let _mutation_scope = RuleMutationScope::new(self);
        if rule.IsImportRule() && self.IsConstructed() {
            B::ThrowDOMException(
                exception_state,
                CSSStyleSheetExceptionCode::kSyntaxError,
                &String::from("Can't insert @import rules into a constructed stylesheet."),
            );
            return 0;
        }
        if !self.Contents().WrapperInsertRule(rule.clone(), index) {
            let (code, text) = if rule.IsNamespaceRule() {
                (
                    CSSStyleSheetExceptionCode::kInvalidStateError,
                    "Failed to insert the rule",
                )
            } else {
                (
                    CSSStyleSheetExceptionCode::kHierarchyRequestError,
                    "Failed to insert the rule.",
                )
            };
            B::ThrowDOMException(exception_state, code, &String::from(text));
            return 0;
        }
        if !self.child_rule_cssom_wrappers_.borrow().is_empty() {
            self.child_rule_cssom_wrappers_
                .borrow_mut()
                .insert(index, None);
        }
        index
    }
    // cpp: css_style_sheet.cc:490-530
    pub fn deleteRule(self: &Rc<Self>, index: usize, exception_state: &mut B::ExceptionState) {
        if !self.CanAccessRules() {
            B::ThrowSecurityError(
                exception_state,
                &String::from("Cannot access StyleSheet to deleteRule"),
            );
            return;
        }
        if index >= self.length() {
            let text = if self.length() != 0 {
                String::from(
                    format!(
                        "The index provided ({index}) is larger than the maximum index ({}).",
                        self.length() - 1
                    )
                    .as_str(),
                )
            } else {
                String::from("Style sheet is empty (length 0).")
            };
            B::ThrowDOMException(
                exception_state,
                CSSStyleSheetExceptionCode::kIndexSizeError,
                &text,
            );
            return;
        }
        let _mutation_scope = RuleMutationScope::new(self);
        if !self.Contents().WrapperDeleteRule(index) {
            B::ThrowDOMException(
                exception_state,
                CSSStyleSheetExceptionCode::kInvalidStateError,
                &String::from("Failed to delete rule"),
            );
            return;
        }
        if !self.child_rule_cssom_wrappers_.borrow().is_empty() {
            let wrapper = self.child_rule_cssom_wrappers_.borrow()[index]
                .as_ref()
                .and_then(Weak::upgrade);
            if let Some(wrapper) = wrapper {
                wrapper.SetParentStyleSheet(None);
            }
            self.child_rule_cssom_wrappers_.borrow_mut().remove(index);
        }
    }
    // cpp: css_style_sheet.cc:532-554
    pub fn addRule(
        self: &Rc<Self>,
        selector: &String,
        style: &String,
        index: i32,
        exception_state: &mut B::ExceptionState,
    ) -> i32 {
        let mut text = selector.Span16().unwrap_or_default().to_vec();
        text.extend(" { ".encode_utf16());
        text.extend_from_slice(style.Span16().unwrap_or_default());
        if !style.empty() {
            text.push(b' ' as u16);
        }
        text.push(b'}' as u16);
        // C++ converts the signed IE extension index to unsigned insertRule.
        self.insertRule(
            &String::from_utf16(&text),
            index as u32 as usize,
            exception_state,
        );
        -1
    }
    pub fn addRuleAtEnd(
        self: &Rc<Self>,
        selector: &String,
        style: &String,
        exception_state: &mut B::ExceptionState,
    ) -> i32 {
        self.addRule(selector, style, self.length() as i32, exception_state)
    }
    // cpp: css_style_sheet.cc:556-583. Parsing and resolution are synchronous.
    pub fn replace(
        self: &Rc<Self>,
        script_state: &B::ScriptState,
        text: &String,
        exception_state: &mut B::ExceptionState,
    ) -> B::ScriptPromise {
        if !self.IsConstructed() {
            B::ThrowDOMException(
                exception_state,
                CSSStyleSheetExceptionCode::kNotAllowedError,
                &String::from("Can't call replace on non-constructed CSSStyleSheets."),
            );
            return B::EmptyPromise();
        }
        self.SetText(text, CSSImportRules::kIgnoreWithWarning);
        B::DidReplaceStyleSheetText(self.OwnerDocument().as_deref(), self, text);
        B::ToResolvedPromise(script_state, self.clone())
    }
    pub fn replaceSync(self: &Rc<Self>, text: &String, exception_state: &mut B::ExceptionState) {
        if !self.IsConstructed() {
            B::ThrowDOMException(
                exception_state,
                CSSStyleSheetExceptionCode::kNotAllowedError,
                &String::from("Can't call replaceSync on non-constructed CSSStyleSheets."),
            );
            return;
        }
        self.SetText(text, CSSImportRules::kIgnoreWithWarning);
        B::DidReplaceStyleSheetText(self.OwnerDocument().as_deref(), self, text);
    }
    // cpp: css_style_sheet.cc:585-595
    pub fn cssRules(
        self: &Rc<Self>,
        exception_state: &mut B::ExceptionState,
    ) -> Option<Rc<StyleSheetCSSRuleList<B>>> {
        if !self.CanAccessRules() {
            B::ThrowSecurityError(exception_state, &String::from("Cannot access rules"));
            return None;
        }
        if let Some(wrapper) = self
            .rule_list_cssom_wrapper_
            .borrow()
            .as_ref()
            .and_then(Weak::upgrade)
        {
            return Some(wrapper);
        }
        let wrapper = StyleSheetCSSRuleList::new(self.clone());
        *self.rule_list_cssom_wrapper_.borrow_mut() = Some(Rc::downgrade(&wrapper));
        Some(wrapper)
    }
    // cpp: css_style_sheet.cc:597-632
    pub fn href(&self) -> String {
        self.Contents().OriginalURL()
    }
    pub fn BaseURL(&self) -> SheetURL<B> {
        self.Contents().BaseURL().clone()
    }
    pub fn IsLoading(&self) -> bool {
        self.Contents().IsLoading()
    }
    pub fn media(self: &Rc<Self>) -> Rc<B::MediaList> {
        if self.MediaQueries().is_none() {
            self.SetMediaQueries(Some(B::CreateMediaQuerySet()));
        }
        if let Some(wrapper) = self
            .media_cssom_wrapper_
            .borrow()
            .as_ref()
            .and_then(Weak::upgrade)
        {
            return wrapper;
        }
        let wrapper = B::NewMediaList(self.clone());
        *self.media_cssom_wrapper_.borrow_mut() = Some(Rc::downgrade(&wrapper));
        wrapper
    }
    pub fn parentStyleSheet(&self) -> Option<Rc<Self>> {
        self.ownerRule().and_then(|rule| rule.parentStyleSheet())
    }
    pub fn OwnerDocument(&self) -> Option<Rc<B::Document>> {
        if let Some(parent) = self.parentStyleSheet() {
            return parent.OwnerDocument();
        }
        if self.IsConstructed() {
            return self.ConstructorDocument();
        }
        self.ownerNode().map(|node| B::NodeDocument(&node))
    }
    // cpp: css_style_sheet.cc:634-657
    pub fn SheetLoaded(self: &Rc<Self>) -> bool {
        let owner = self
            .ownerNode()
            .expect("SheetLoaded requires an owner node");
        self.SetLoadCompleted(B::NodeSheetLoaded(&owner));
        self.load_completed_.get()
    }
    pub fn SetToPendingState(self: &Rc<Self>) {
        self.SetLoadCompleted(false);
        B::NodeSetToPendingState(
            &self
                .ownerNode()
                .expect("SetToPendingState requires an owner node"),
        );
    }
    pub fn SetLoadCompleted(self: &Rc<Self>, completed: bool) {
        if completed == self.load_completed_.get() {
            return;
        }
        self.load_completed_.set(completed);
        if completed {
            self.Contents().ClientLoadCompleted(self);
        } else {
            self.Contents().ClientLoadStarted(self);
        }
    }
    // cpp: css_style_sheet.cc:659-709
    pub fn SetText(self: &Rc<Self>, text: &String, import_rules: CSSImportRules) {
        self.DetachCSSOMWrappers();
        self.child_rule_cssom_wrappers_.borrow_mut().clear();
        let use_constructed_cache = B::ConstructableStylesheetCacheEnabled()
            && self.IsConstructed()
            && self.OwnerDocument().is_some()
            && import_rules == CSSImportRules::kIgnoreWithWarning;
        if use_constructed_cache {
            let document = self
                .OwnerDocument()
                .expect("constructed cache requires owner document");
            if let Some(cached) =
                B::FindStyleSheetContents(&document, text, self.Contents().ParserContext())
            {
                self.SetContents(cached);
                self.DidMutate(Mutation::kSheet);
                return;
            }
            let current = self.Contents();
            self.SetContents(StyleSheetContents::new(
                current.ParserContext().clone(),
                current.OriginalURL(),
                None,
            ));
            let contents = self.Contents();
            let parse_result = contents.ParseString(text, false, CSSDeferPropertyParsing::kNo);
            if parse_result == ParseSheetResult::kHasUnallowedImportRule {
                self.WarnUnallowedImports();
            }
            self.DidMutate(Mutation::kSheet);
            if parse_result == ParseSheetResult::kSucceeded && contents.IsCacheableForStyleElement()
            {
                B::AddStyleSheetContents(&document, text, contents);
            }
            return;
        }
        let _mutation_scope = RuleMutationScope::new(self);
        self.Contents().ClearRules();
        let allow_imports = import_rules == CSSImportRules::kAllow;
        if self
            .Contents()
            .ParseString(text, allow_imports, CSSDeferPropertyParsing::kNo)
            == ParseSheetResult::kHasUnallowedImportRule
            && import_rules == CSSImportRules::kIgnoreWithWarning
        {
            self.WarnUnallowedImports();
        }
    }
    fn WarnUnallowedImports(&self) {
        let document = self
            .OwnerDocument()
            .expect("import warning requires owner document");
        B::AddJavaScriptWarning(&document, &String::from("@import rules are not allowed here. See https://github.com/WICG/construct-stylesheets/issues/119#issuecomment-588352418."));
    }
    // cpp: css_style_sheet.cc:711-724
    pub fn SetAlternateFromConstructor(&self, alternate: bool) {
        self.alternate_from_constructor_.set(alternate);
    }
    pub fn IsAlternate(&self) -> bool {
        if let Some(owner) = self.ownerNode() {
            return B::NodeAsElement(&owner).is_some_and(|element| {
                let rel = B::ElementRelAttribute(&element);
                let alternate: Vec<u16> = "alternate".encode_utf16().collect();
                rel.Span16()
                    .unwrap_or_default()
                    .windows(alternate.len())
                    .any(|candidate| candidate == alternate)
            });
        }
        self.alternate_from_constructor_.get()
    }
    // cpp: css_style_sheet.cc:726-753
    pub fn CanBeActivated(&self, current_preferrable_name: &String) -> bool {
        if self.disabled() {
            return false;
        }
        let owner = self.ownerNode();
        if let Some(owner) = &owner {
            if B::NodeIsInShadowTree(owner)
                && matches!(
                    B::NodeOwnerKind(owner),
                    CSSStyleSheetOwnerKind::kHTMLStyle | CSSStyleSheetOwnerKind::kSVGStyle
                )
            {
                return true;
            }
        }
        let check_title = owner.as_ref().is_none_or(|owner| {
            let kind = B::NodeOwnerKind(owner);
            kind == CSSStyleSheetOwnerKind::kProcessingInstruction
                || kind != CSSStyleSheetOwnerKind::kHTMLLink
                || !B::LinkIsEnabledViaScript(owner)
        });
        let title = self.title();
        if check_title && !title.empty() && title != *current_preferrable_name {
            return false;
        }
        if self.IsAlternate() && title.empty() {
            return false;
        }
        true
    }

    // cpp: css_style_sheet.h:106-270,308 (inline/accessor bodies)
    pub fn ownerNode(&self) -> Option<Rc<B::Node>> {
        self.owner_node_.borrow().clone()
    }
    pub fn ownerRule(&self) -> Option<Rc<B::CSSRule>> {
        self.owner_rule_.borrow().clone()
    }
    pub fn ClearOwnerRule(&self) {
        self.owner_rule_.borrow_mut().take();
    }
    pub fn OwnerParentOrShadowHostElement(&self) -> Option<Rc<B::Element>> {
        self.owner_parent_or_shadow_host_element_
            .as_ref()
            .and_then(Weak::upgrade)
    }
    pub fn title(&self) -> String {
        self.title_.borrow().clone()
    }
    pub fn SetTitle(&self, title: &String) {
        *self.title_.borrow_mut() = if title.empty() {
            String::default()
        } else {
            title.clone()
        };
    }
    pub fn disabled(&self) -> bool {
        self.is_disabled_.get()
    }
    pub fn removeRule(self: &Rc<Self>, index: usize, exception_state: &mut B::ExceptionState) {
        self.deleteRule(index, exception_state);
    }
    pub fn ItemInternal(self: &Rc<Self>, index: usize) -> Option<Rc<B::CSSRule>> {
        self.item(index, false)
    }
    pub fn MediaQueries(&self) -> Option<Rc<B::MediaQuerySet>> {
        self.media_queries_.borrow().clone()
    }
    pub fn SetMediaQueries(&self, queries: Option<Rc<B::MediaQuerySet>>) {
        *self.media_queries_.borrow_mut() = queries;
    }
    pub fn GetMediaQueryResultFlags(&self) -> MediaQueryResultFlags {
        self.media_query_result_flags_.get()
    }
    pub fn HasMediaQueryResults(&self) -> bool {
        let flags = self.GetMediaQueryResultFlags();
        flags.is_viewport_dependent || flags.is_device_dependent
    }
    pub fn ConstructorDocument(&self) -> Option<Rc<B::Document>> {
        self.constructor_document_.borrow().clone()
    }
    pub fn SetConstructorDocument(&self, document: Rc<B::Document>) {
        *self.constructor_document_.borrow_mut() = Some(document);
    }
    pub fn Contents(&self) -> Rc<StyleSheetContents<B>> {
        self.contents_.borrow().clone()
    }
    pub fn IsInline(&self) -> bool {
        self.is_inline_stylesheet_
    }
    pub fn StartPositionInSource(&self) -> B::TextPosition {
        self.start_position_.clone()
    }
    pub fn LoadCompleted(&self) -> bool {
        self.load_completed_.get()
    }
    pub fn IsConstructed(&self) -> bool {
        self.ConstructorDocument().is_some()
    }
    pub fn SetIsForCSSModuleScript(&self) {
        self.is_for_css_module_script_.set(true);
    }
    pub fn IsForCSSModuleScript(&self) -> bool {
        self.is_for_css_module_script_.get()
    }
    pub fn IsCSSStyleSheet(&self) -> bool {
        true
    }
    pub fn r#type(&self) -> String {
        String::from("text/css")
    }
    pub fn AlternateFromConstructor(&self) -> bool {
        self.alternate_from_constructor_.get()
    }
}
impl<B: CSSStyleSheetBackend> SingleOwnerDocument<B::Platform> for CSSStyleSheet<B> {
    fn SingleOwnerDocument(&self) -> Option<DocumentHandle<B::Platform>> {
        CSSStyleSheet::<B>::SingleOwnerDocument(Some(self))
            .map(|document| B::ParserDocument(&document))
    }
}
