// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//!
//! Live DOM collaborators for the translated ElementRuleCollector.
#![allow(non_snake_case)]

use crate::{
    cascade_layer_map::CascadeLayerMap,
    css_property_value_set::CSSPropertyValueSetRuleHandle,
    css_selector::CSSSelectorComplex,
    element_rule_collector::{ElementRuleCollectorRuntime, SearchTextRequest, SelectorCheckerMode},
    persistent_selector::{
        PersistentPartNames, PersistentSelectorBackend, PersistentSelectorMatchSession,
        PersistentStyleScope, PersistentStyleScopeFrame,
    },
    production_css_value::ProductionCSSValueDispatch,
    production_rule_context::{
        ProductionContainerQuery, ProductionPseudoRequest, ProductionStartingStyleContext,
    },
    production_style_sheet::{Backend, ContainerSet},
    resolver::{
        match_flags::MatchFlags,
        match_result::{MatchResult as CascadeMatchResult, TreeScope as MatchTreeScope},
        production_container_runtime::{
            ApplyContainerUnitDependencies, ProductionContainerRuntime, ProductionContainerSelector,
        },
    },
    selector_checker::{
        MatchResult as CheckerMatchResult, SelectorChecker, SelectorCheckerBackend,
        SelectorCheckingContext,
    },
    style_rule::StyleRule,
    style_scope::StyleScope,
};
use dom::{persistent_document::DOMAttribute, Document};
use foundation::{AtomicString, CSSPropertyID};
use layoutng_style::style::computed_style_constants::PseudoId;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProductionTreeScope(pub usize);
impl MatchTreeScope for ProductionTreeScope {}

#[derive(Clone, Debug)]
pub(crate) struct ProductionElementResolveContext {
    pub element: usize,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ProductionStyleRecalcContext {
    pub starting_style: ProductionStartingStyleContext,
    pub size_container: Option<usize>,
}

impl ProductionStyleRecalcContext {
    /// Mirrors StyleRecalcContext::old_style for the style request being
    /// resolved. A newly generated pseudo has no old style even when its
    /// originating element does; an existing pseudo rejects @starting-style
    /// unless DevTools supplies the separate force flag.
    pub(crate) fn ForStyleRequest(
        document: &Document,
        element: usize,
        pseudo_id: PseudoId,
        size_container: Option<usize>,
    ) -> Self {
        let resolved = document.ResolvedStyleFor(element);
        let has_old_style = match pseudo_id {
            PseudoId::kPseudoIdNone => resolved.is_some(),
            PseudoId::kPseudoIdBefore => resolved.is_some_and(|style| style.before.is_some()),
            PseudoId::kPseudoIdAfter => resolved.is_some_and(|style| style.after.is_some()),
            PseudoId::kPseudoIdFirstLetter => {
                resolved.is_some_and(|style| style.first_letter.is_some())
            }
            PseudoId::kPseudoIdPlaceholder => {
                resolved.is_some_and(|style| style.placeholder.is_some())
            }
            _ => false,
        };
        let mut starting_style = ProductionStartingStyleContext::InitialResolution();
        starting_style.has_old_style = has_old_style;
        Self {
            starting_style,
            size_container,
        }
    }

    /// Normal element recalc convenience matching StyleRequest::kForElement.
    pub(crate) fn ForElement(
        document: &Document,
        element: usize,
        size_container: Option<usize>,
    ) -> Self {
        Self::ForStyleRequest(document, element, PseudoId::kPseudoIdNone, size_container)
    }
}

#[derive(Default)]
pub(crate) struct ProductionContainerSelectorCache {
    selected: HashMap<(Option<usize>, usize), Option<usize>>,
}

#[derive(Clone)]
pub(crate) struct ProductionCheckingContext {
    element: Rc<usize>,
    pseudo_element: Option<Rc<usize>>,
    style_scope_frame: Option<Rc<PersistentStyleScopeFrame>>,
    scope: Option<Rc<usize>>,
    tree_scope: Option<Rc<ProductionTreeScope>>,
    pseudo: ProductionPseudoRequest,
    vtt_origin: Option<Rc<usize>>,
    search_text_current: bool,
    style_scope: Option<Rc<StyleScope>>,
}

pub(crate) struct ProductionElementRuleCollectorRuntime<'d> {
    selector: Rc<PersistentSelectorBackend<'d>>,
    author_layer_map: Option<Rc<CascadeLayerMap>>,
    implicit_scope_roots: HashMap<usize, Vec<usize>>,
    container_runtime: Option<&'d ProductionContainerRuntime<'d>>,
    style_scopes: RefCell<HashMap<usize, Rc<PersistentStyleScope>>>,
    tree_scopes: RefCell<HashMap<usize, Rc<ProductionTreeScope>>>,
}

impl<'d> ProductionElementRuleCollectorRuntime<'d> {
    pub fn new(session: &PersistentSelectorMatchSession<'d>) -> Self {
        Self {
            selector: session.Backend(),
            author_layer_map: None,
            implicit_scope_roots: HashMap::new(),
            container_runtime: None,
            style_scopes: RefCell::new(HashMap::new()),
            tree_scopes: RefCell::new(HashMap::new()),
        }
    }

    pub fn with_author_layer_map(
        session: &PersistentSelectorMatchSession<'d>,
        author_layer_map: Option<Rc<CascadeLayerMap>>,
    ) -> Self {
        Self {
            selector: session.Backend(),
            author_layer_map,
            implicit_scope_roots: HashMap::new(),
            container_runtime: None,
            style_scopes: RefCell::new(HashMap::new()),
            tree_scopes: RefCell::new(HashMap::new()),
        }
    }

    pub fn with_production_inputs(
        session: &PersistentSelectorMatchSession<'d>,
        author_layer_map: Option<Rc<CascadeLayerMap>>,
        implicit_scope_roots: HashMap<usize, Vec<usize>>,
        container_runtime: Option<&'d ProductionContainerRuntime<'d>>,
    ) -> Self {
        Self {
            selector: session.Backend(),
            author_layer_map,
            implicit_scope_roots,
            container_runtime,
            style_scopes: RefCell::new(HashMap::new()),
            tree_scopes: RefCell::new(HashMap::new()),
        }
    }

    fn tree_scope(&self, root: usize) -> Rc<ProductionTreeScope> {
        self.tree_scopes
            .borrow_mut()
            .entry(root)
            .or_insert_with(|| Rc::new(ProductionTreeScope(root)))
            .clone()
    }

    fn node(&self, index: &usize) -> &dom::persistent_document::DOMNode {
        self.selector.document().Node(*index)
    }

    fn properties<'a>(
        set: &'a CSSPropertyValueSetRuleHandle<ProductionCSSValueDispatch>,
    ) -> std::borrow::Cow<
        'a,
        [crate::css_property_value::CSSPropertyValue<ProductionCSSValueDispatch>],
    > {
        match set {
            CSSPropertyValueSetRuleHandle::Immutable(set) => {
                std::borrow::Cow::Borrowed(set.Properties())
            }
            CSSPropertyValueSetRuleHandle::Mutable(set) => {
                std::borrow::Cow::Owned(set.borrow().Properties().to_vec())
            }
        }
    }

    fn persistent_style_scope(&self, scope: &StyleScope) -> Rc<PersistentStyleScope> {
        let identity = scope as *const StyleScope as usize;
        if let Some(scope) = self.style_scopes.borrow().get(&identity) {
            return scope.clone();
        }
        let roots = self
            .implicit_scope_roots
            .get(&identity)
            .map_or(&[][..], Vec::as_slice);
        let persistent =
            PersistentSelectorMatchSession::RuleStyleScopeWithImplicitRoots(scope, roots);
        self.style_scopes
            .borrow_mut()
            .insert(identity, persistent.clone());
        persistent
    }
}

impl ElementRuleCollectorRuntime<Backend> for ProductionElementRuleCollectorRuntime<'_> {
    type RuntimeDocument = bool;
    type Element = usize;
    type ContainerNode = usize;
    // This production adapter is already instantiated for one selected
    // TreeScope. Its retained author layer map is the part of
    // ScopedStyleResolver that ElementRuleCollector consumes.
    type ScopedStyleResolver = Rc<CascadeLayerMap>;
    type TreeScope = ProductionTreeScope;
    type ElementResolveContext = ProductionElementResolveContext;
    type StyleRecalcContext = ProductionStyleRecalcContext;
    type SelectorFilter = ();
    type StyleRequest = ProductionPseudoRequest;
    type PartNames = PersistentPartNames;
    type SelectorChecker = SelectorChecker<PersistentSelectorBackend<'static>>;
    type SelectorCheckingContext = ProductionCheckingContext;
    type CheckerMatchResult = CheckerMatchResult<PersistentSelectorBackend<'static>>;
    type StyleScopeFrame = PersistentStyleScopeFrame;
    type ContainerSelectorCache = ProductionContainerSelectorCache;
    type Attribute = DOMAttribute;
    type CSSRule = ();
    type CSSRuleCollection = Vec<()>;
    type CSSStyleSheet = ();
    type StyleSheetContents = ();
    type StyleRuleUsageTracker = ();

    fn ContextElement(&self, c: &Self::ElementResolveContext) -> Rc<Self::Element> {
        self.selector.handle(c.element)
    }
    fn ContextUltimateOriginatingElementOrSelf(
        &self,
        c: &Self::ElementResolveContext,
    ) -> Rc<Self::Element> {
        self.ContextElement(c)
    }
    fn ContextParentElement(&self, c: &Self::ElementResolveContext) -> Option<Rc<Self::Element>> {
        self.node(&c.element)
            .Parent()
            .map(|index| self.selector.handle(index))
    }
    fn ElementIsPseudoElement(&self, _: &Self::Element) -> bool {
        false
    }
    fn LayoutTraversalParentOfUltimateOriginatingElement(
        &self,
        e: &Self::Element,
    ) -> Option<Rc<Self::Element>> {
        self.node(e)
            .Parent()
            .map(|index| self.selector.handle(index))
    }
    fn ParentStackIsConsistent(&self, _: &Self::SelectorFilter, _: Option<&Self::Element>) -> bool {
        false
    }
    fn FastRejectSelector(&self, _: &Self::SelectorFilter, _: &[u16]) -> bool {
        false
    }
    fn DefaultStyleRequest(&self) -> Self::StyleRequest {
        ProductionPseudoRequest::Element()
    }
    fn RequestPseudoId(&self, r: &Self::StyleRequest) -> PseudoId {
        r.id
    }
    fn RequestPseudoArgument(&self, r: &Self::StyleRequest) -> AtomicString {
        r.argument.clone().unwrap_or_default()
    }
    fn RequestSearchText(&self, r: &Self::StyleRequest) -> SearchTextRequest {
        r.search_text
    }
    fn RecalcStyleScopeFrame(
        &self,
        _: &Self::StyleRecalcContext,
    ) -> Option<Rc<Self::StyleScopeFrame>> {
        None
    }
    fn RecalcIsEnsuringStyle(&self, c: &Self::StyleRecalcContext) -> bool {
        c.starting_style.is_ensuring_style
    }
    fn RecalcHasOldStyle(&self, c: &Self::StyleRecalcContext) -> bool {
        c.starting_style.has_old_style
    }
    fn RecalcSizeContainer(&self, c: &Self::StyleRecalcContext) -> Option<Rc<Self::Element>> {
        c.size_container.map(|node| self.selector.handle(node))
    }
    fn RecalcTrySet(
        &self,
        _: &Self::StyleRecalcContext,
    ) -> Option<Rc<<Backend as crate::style_rule::StyleRuleDependencies>::CSSPropertyValueSet>>
    {
        None
    }
    fn RecalcTryTacticsSet(
        &self,
        _: &Self::StyleRecalcContext,
    ) -> Option<Rc<<Backend as crate::style_rule::StyleRuleDependencies>::CSSPropertyValueSet>>
    {
        None
    }
    fn NewStyleScopeFrame(
        &self,
        origin: Rc<Self::Element>,
        parent: Option<Rc<Self::StyleScopeFrame>>,
    ) -> Rc<Self::StyleScopeFrame> {
        PersistentStyleScopeFrame::new(origin, parent)
    }
    fn ParentFrameOrThis(
        &self,
        frame: &Rc<Self::StyleScopeFrame>,
        origin: &Self::Element,
    ) -> Rc<Self::StyleScopeFrame> {
        if **frame.Element() == *origin {
            frame.clone()
        } else {
            PersistentStyleScopeFrame::new(self.selector.handle(*origin), Some(frame.clone()))
        }
    }
    fn FrameHasSeenImplicitScope(&self, _: &Self::StyleScopeFrame, _: &StyleScope) -> bool {
        false
    }
    fn StyleScopeIsImplicit(&self, scope: &StyleScope) -> bool {
        scope.IsImplicit()
    }
    fn NewSelectorCheckingContext(
        &self,
        c: &Self::ElementResolveContext,
    ) -> Self::SelectorCheckingContext {
        ProductionCheckingContext {
            element: self.ContextElement(c),
            pseudo_element: None,
            style_scope_frame: None,
            scope: None,
            tree_scope: None,
            pseudo: ProductionPseudoRequest::Element(),
            vtt_origin: None,
            search_text_current: false,
            style_scope: None,
        }
    }
    fn CheckingElement(&self, c: &Self::SelectorCheckingContext) -> Rc<Self::Element> {
        c.element.clone()
    }
    fn CheckingPseudoElement(
        &self,
        c: &Self::SelectorCheckingContext,
    ) -> Option<Rc<Self::Element>> {
        c.pseudo_element.clone()
    }
    fn SetCheckingStyleScopeFrame(
        &self,
        c: &mut Self::SelectorCheckingContext,
        f: Rc<Self::StyleScopeFrame>,
    ) {
        c.style_scope_frame = Some(f);
    }
    fn SetCheckingScope(
        &self,
        c: &mut Self::SelectorCheckingContext,
        scope: Option<Rc<Self::ContainerNode>>,
    ) {
        c.scope = scope;
    }
    fn SetCheckingTreeScope(
        &self,
        c: &mut Self::SelectorCheckingContext,
        scope: Option<Rc<Self::TreeScope>>,
    ) {
        c.tree_scope = scope;
    }
    fn SetCheckingPseudo(
        &self,
        c: &mut Self::SelectorCheckingContext,
        id: PseudoId,
        argument: &AtomicString,
    ) {
        c.pseudo.id = id;
        c.pseudo.argument = (!argument.IsNull()).then(|| argument.clone());
    }
    fn SetCheckingVTTOrigin(
        &self,
        c: &mut Self::SelectorCheckingContext,
        origin: Option<Rc<Self::Element>>,
    ) {
        c.vtt_origin = origin;
    }
    fn SetCheckingSearchTextCurrent(&self, c: &mut Self::SelectorCheckingContext, current: bool) {
        c.search_text_current = current;
    }
    fn SetCheckingStyleScope(
        &self,
        c: &mut Self::SelectorCheckingContext,
        scope: Option<Rc<StyleScope>>,
    ) {
        c.style_scope = scope;
    }
    fn NewSelectorChecker(
        &self,
        parts: Option<&Self::PartNames>,
        request: &Self::StyleRequest,
        mode: SelectorCheckerMode,
        ua: bool,
    ) -> Self::SelectorChecker {
        // The selector backend only borrows the live document; the checker is
        // consumed synchronously inside this runtime. Erasing that lifetime in
        // the associated type does not extend the backend's actual lifetime.
        let backend = unsafe {
            std::mem::transmute::<
                Rc<PersistentSelectorBackend<'_>>,
                Rc<PersistentSelectorBackend<'static>>,
            >(self.selector.clone())
        };
        SelectorChecker::ForPseudoRequestWithParts(
            backend,
            parts.cloned().map(Rc::new),
            mode,
            request.argument.clone().unwrap_or_default(),
            request.ident_list.clone(),
            ua,
        )
    }
    fn NewCheckerMatchResult(&self) -> Self::CheckerMatchResult {
        CheckerMatchResult::default()
    }
    fn CheckerMatch(
        &self,
        checker: &Self::SelectorChecker,
        c: &mut Self::SelectorCheckingContext,
        selector: CSSSelectorComplex<'_>,
        match_visited: bool,
        result: &mut Self::CheckerMatchResult,
    ) -> bool {
        let backend = unsafe {
            std::mem::transmute::<
                Rc<PersistentSelectorBackend<'_>>,
                Rc<PersistentSelectorBackend<'static>>,
            >(self.selector.clone())
        };
        let mut actual = SelectorCheckingContext::new(backend.handle(*c.element));
        actual.selector = Some(selector);
        actual.scope = c.scope.as_ref().map(|scope| backend.handle(**scope));
        // TreeScope comparisons in SelectorChecker are identity comparisons,
        // as in Blink. Reuse the selector session's interned arena handle;
        // constructing a fresh Rc here makes ::slotted/::part scope checks
        // fail even when both values name the same ShadowRoot.
        actual.tree_scope = c.tree_scope.as_ref().map(|scope| backend.handle(scope.0));
        actual.style_scope = c
            .style_scope
            .as_deref()
            .map(|scope| self.persistent_style_scope(scope));
        actual.style_scope_frame = c.style_scope_frame.clone();
        actual.pseudo_id = c.pseudo.id;
        actual.pseudo_argument = c.pseudo.argument.clone();
        actual.vtt_originating_element = c.vtt_origin.clone();
        actual.search_text_request_is_current = c.search_text_current;
        actual.match_visited = match_visited;
        // Chromium 6c1d401f keeps CSSLogicalCombinationPseudo disabled.  The
        // ordinary element pass must therefore retain SelectorChecker's
        // dynamic-pseudo result too: ElementRuleCollector consumes that result
        // to set MatchResult::pseudo_element_styles without transferring the
        // pseudo declarations into the originating element.  Enabling logical
        // pseudo mode here made successful production resolution silently lose
        // ::before/::after markers; compatibility fallback happened to mask it.
        backend.WithLogicalPseudoMode(false, || checker.Match(&actual, result))
    }
    fn EasySelectorMatch(
        &self,
        _: CSSSelectorComplex<'_>,
        _: &Self::Element,
        _: Option<&Self::Element>,
        _: PseudoId,
        _: &mut Self::CheckerMatchResult,
    ) -> bool {
        unreachable!("production RuleSet disables easy-selector classification")
    }
    fn MatchDynamicPseudo(&self, result: &Self::CheckerMatchResult) -> PseudoId {
        result.dynamic_pseudo
    }
    fn MatchFlags(&self, result: &Self::CheckerMatchResult) -> MatchFlags {
        result.flags
    }
    fn MatchProximity(&self, result: &Self::CheckerMatchResult) -> u32 {
        result.proximity
    }
    fn MatchCustomHighlightName(&self, result: &Self::CheckerMatchResult) -> AtomicString {
        result.custom_highlight_name.clone().unwrap_or_default()
    }
    fn GetPseudoElement(
        &self,
        element: &Self::Element,
        pseudo: PseudoId,
        _: &AtomicString,
    ) -> Option<Rc<Self::Element>> {
        let shadow_id = match pseudo {
            PseudoId::kPseudoIdPlaceholder => "-webkit-input-placeholder",
            _ => return None,
        };
        self.selector
            .document()
            .UAShadowPseudoElement(*element, shadow_id)
            .map(|pseudo| self.selector.handle(pseudo))
    }
    fn ForceStartingStyle(&self, _: &Self::Element) -> bool {
        false
    }
    fn ScopeOwnerShadowHost(&self, scope: &Self::ContainerNode) -> Option<Rc<Self::Element>> {
        self.selector
            .document()
            .ShadowHost(*scope)
            .map(|host| self.selector.handle(host))
    }
    fn ScopeTreeScope(&self, scope: &Self::ContainerNode) -> Rc<Self::TreeScope> {
        self.tree_scope(*scope)
    }
    fn ElementTreeScope(&self, e: &Self::Element) -> Rc<Self::TreeScope> {
        let scope = SelectorCheckerBackend::ElementTreeScope(self.selector.as_ref(), e);
        self.tree_scope(*scope)
    }
    fn ScopeScopedStyleResolver(
        &self,
        _: &Self::ContainerNode,
    ) -> Option<Rc<Self::ScopedStyleResolver>> {
        self.author_layer_map.clone().map(Rc::new)
    }
    fn ResolverCascadeLayerMap(
        &self,
        resolver: &Self::ScopedStyleResolver,
    ) -> Option<Rc<CascadeLayerMap>> {
        Some(resolver.clone())
    }
    fn UserCascadeLayerMap(&self, _: &Self::RuntimeDocument) -> Option<Rc<CascadeLayerMap>> {
        self.author_layer_map.clone()
    }
    fn ElementDocument(&self, element: &Self::Element) -> Rc<Self::RuntimeDocument> {
        Rc::new(self.selector.document().IsInHTMLDocument(*element))
    }
    fn AttributeOrClassBloomFilter(&self, _: &Self::Element) -> u32 {
        u32::MAX
    }
    fn ShadowPseudoId(&self, element: &Self::Element) -> AtomicString {
        SelectorCheckerBackend::ShadowPseudoId(self.selector.as_ref(), element)
    }
    fn IsVTTElement(&self, _: &Self::Element) -> bool {
        // Persistent DOM nodes do not model Blink's VTTElement subclass.
        false
    }
    fn HasID(&self, e: &Self::Element) -> bool {
        SelectorCheckerBackend::HasID(self.selector.as_ref(), e)
    }
    fn IdForStyleResolution(&self, e: &Self::Element) -> AtomicString {
        SelectorCheckerBackend::IdForStyleResolution(self.selector.as_ref(), e)
    }
    fn IsStyledElement(&self, e: &Self::Element) -> bool {
        self.node(e).Type() == dom::persistent_document::DOMNodeType::kElement
    }
    fn HasClass(&self, e: &Self::Element) -> bool {
        SelectorCheckerBackend::HasClass(self.selector.as_ref(), e)
    }
    fn ClassNames(&self, e: &Self::Element) -> Vec<AtomicString> {
        self.node(e)
            .FindAttribute("class")
            .map(|attribute| {
                attribute
                    .value
                    .split_ascii_whitespace()
                    .map(AtomicString::from_str)
                    .collect()
            })
            .unwrap_or_default()
    }
    fn IsHTMLElement(&self, e: &Self::Element) -> bool {
        SelectorCheckerBackend::IsHTMLElement(self.selector.as_ref(), e)
    }
    fn IsHTMLDocument(&self, document: &Self::RuntimeDocument) -> bool {
        *document
    }
    fn Attributes(&self, e: &Self::Element) -> Vec<Rc<Self::Attribute>> {
        self.AttributesWithoutUpdate(e)
    }
    fn AttributesWithoutStyleUpdate(&self, e: &Self::Element) -> Vec<Rc<Self::Attribute>> {
        self.AttributesWithoutUpdate(e)
    }
    fn AttributesWithoutUpdate(&self, e: &Self::Element) -> Vec<Rc<Self::Attribute>> {
        self.node(e)
            .Attributes()
            .iter()
            .cloned()
            .map(Rc::new)
            .collect()
    }
    fn AttributeLocalName(&self, a: &Self::Attribute) -> AtomicString {
        AtomicString::from_str(&a.local_name)
    }
    fn AttributeNamespaceURI(&self, a: &Self::Attribute) -> AtomicString {
        AtomicString::from_str(&a.namespace_uri)
    }
    fn AttributeValue(&self, a: &Self::Attribute) -> AtomicString {
        AtomicString::from_str(&a.value)
    }
    fn LocalName(&self, e: &Self::Element) -> AtomicString {
        SelectorCheckerBackend::ElementLocalName(self.selector.as_ref(), e)
    }
    fn LocalNameForSelectorMatching(&self, e: &Self::Element) -> AtomicString {
        self.LocalName(e)
    }
    fn InputTypeAttribute(&self, e: &Self::Element) -> AtomicString {
        self.node(e)
            .FindAttribute("type")
            .map(|attribute| AtomicString::from_str(&attribute.value))
            .unwrap_or_default()
    }
    fn IsLink(&self, e: &Self::Element) -> bool {
        SelectorCheckerBackend::IsLink(self.selector.as_ref(), e)
    }
    fn MatchesFocusPseudoClass(&self, e: &Self::Element, id: PseudoId) -> bool {
        SelectorChecker::new(self.selector.clone(), SelectorCheckerMode::kResolvingStyle)
            .MatchesFocusPseudoClass(e, id)
    }
    fn MatchesFocusVisiblePseudoClass(&self, e: &Self::Element) -> bool {
        SelectorChecker::new(self.selector.clone(), SelectorCheckerMode::kResolvingStyle)
            .MatchesFocusVisiblePseudoClass(e)
    }
    fn MatchesActiveViewTransitionPseudoClass(&self, _: &Self::Element) -> bool {
        // PersistentDocument has no active ViewTransition state.
        false
    }
    fn IsUnboundedHTMLElementActive(&self, e: &Self::Element) -> bool {
        SelectorCheckerBackend::IsActive(self.selector.as_ref(), e)
    }
    fn IsDocumentElement(&self, e: &Self::Element) -> bool {
        SelectorCheckerBackend::DocumentElement(self.selector.as_ref(), e)
            .is_some_and(|root| *root == *e)
    }
    fn NewContainerSelectorCache(&self) -> Self::ContainerSelectorCache {
        ProductionContainerSelectorCache::default()
    }
    fn ContainerQuerySetParent(&self, set: &ContainerSet) -> Option<Rc<ContainerSet>> {
        set.Parent().map(|parent| Rc::new(parent.clone()))
    }
    fn ContainerQueries(
        &self,
        set: &ContainerSet,
    ) -> Vec<Rc<<Backend as crate::style_rule::StyleRuleDependencies>::ContainerQuery>> {
        set.Queries().to_vec()
    }
    fn DetermineContainerStartingElement(
        &self,
        element: &Self::Element,
        pseudo: PseudoId,
        query: &<Backend as crate::style_rule::StyleRuleDependencies>::ContainerQuery,
        nearest: Option<Rc<Self::Element>>,
    ) -> Option<Rc<Self::Element>> {
        let runtime = self.container_runtime?;
        let selector = ProductionContainerSelector::FromQuery(query);
        runtime
            .DetermineStartingElement(*element, pseudo, &selector, nearest.map(|node| *node))
            .map(|node| self.selector.handle(node))
    }
    fn EvalAndAddContainerQuery(
        &self,
        start: Option<Rc<Self::Element>>,
        _: &Self::StyleRecalcContext,
        query: &<Backend as crate::style_rule::StyleRuleDependencies>::ContainerQuery,
        cache: &mut Self::ContainerSelectorCache,
        result: &mut CascadeMatchResult,
    ) -> bool {
        let Some(runtime) = self.container_runtime else {
            return false;
        };
        let selector = ProductionContainerSelector::FromQuery(query);
        if !selector.SelectsAnyContainer() {
            return false;
        }
        selector.SetDependencyFlags(result);
        let start = start.map(|node| *node);
        let key = (start, query as *const ProductionContainerQuery as usize);
        let container = *cache
            .selected
            .entry(key)
            .or_insert_with(|| runtime.FindContainer(start, &selector));
        let Some(container) = container else {
            return false;
        };
        if query.Query().is_none() {
            return true;
        }
        let Ok(evaluation) = runtime.Evaluate(container, query) else {
            return false;
        };
        ApplyContainerUnitDependencies(evaluation.unit_flags, result);
        evaluation.value == crate::kleene_value::KleeneValue::kTrue
    }
    fn SetContainerDependencyFlags(
        &self,
        query: &<Backend as crate::style_rule::StyleRuleDependencies>::ContainerQuery,
        result: &mut CascadeMatchResult,
    ) {
        ProductionContainerSelector::FromQuery(query).SetDependencyFlags(result)
    }
    fn PropertyIDs(
        &self,
        set: &CSSPropertyValueSetRuleHandle<ProductionCSSValueDispatch>,
    ) -> Vec<CSSPropertyID> {
        Self::properties(set)
            .iter()
            .map(|property| property.PropertyID())
            .collect()
    }
    fn PropertiesIsEmpty(
        &self,
        set: &CSSPropertyValueSetRuleHandle<ProductionCSSValueDispatch>,
    ) -> bool {
        Self::properties(set).is_empty()
    }
    fn PropertiesHasProperty(
        &self,
        set: &CSSPropertyValueSetRuleHandle<ProductionCSSValueDispatch>,
        id: CSSPropertyID,
    ) -> bool {
        Self::properties(set)
            .iter()
            .any(|property| property.PropertyID() == id)
    }
    fn CollectionLength(&self, _: &Self::CSSRuleCollection) -> usize {
        0
    }
    fn CollectionItemInternal(&self, _: &Self::CSSRuleCollection, _: usize) -> Rc<Self::CSSRule> {
        unreachable!("an empty production CSS rule collection has no item")
    }
    fn StyleRuleFromCSSRule(&self, _: &Self::CSSRule) -> Option<Rc<StyleRule<Backend>>> {
        None
    }
    fn ImportSheetFromCSSRule(&self, _: &Self::CSSRule) -> Option<Rc<Self::CSSStyleSheet>> {
        None
    }
    fn CSSRuleChildren(&self, _: &Self::CSSRule) -> Option<Rc<Self::CSSRuleCollection>> {
        None
    }
    fn NestedDeclarationsFromCSSRule(
        &self,
        _: &Self::CSSRule,
    ) -> Option<(Rc<StyleRule<Backend>>, Rc<Self::CSSRule>)> {
        None
    }
    fn SheetRuleCollection(&self, _: &Self::CSSStyleSheet) -> Rc<Self::CSSRuleCollection> {
        Rc::new(Vec::new())
    }
    fn ActiveScopeStyleSheets(&self, _: &Self::TreeScope) -> Vec<Rc<Self::CSSStyleSheet>> {
        Vec::new()
    }
    fn ActiveUserStyleSheets(&self, _: &Self::RuntimeDocument) -> Vec<Rc<Self::CSSStyleSheet>> {
        Vec::new()
    }
    fn SheetContents(&self, _: &Self::CSSStyleSheet) -> Rc<Self::StyleSheetContents> {
        Rc::new(())
    }
    fn StartOrStopSelectorMapTracking(
        &self,
        _: Option<&Self::TreeScope>,
        _: &Self::RuntimeDocument,
    ) {
    }
    fn LookupStyleSheetContentsForRule(
        &self,
        _: &StyleRule<Backend>,
    ) -> Option<Rc<Self::StyleSheetContents>> {
        None
    }
    fn ContentsClientInTreeScope(
        &self,
        _: &Self::StyleSheetContents,
        _: &Self::TreeScope,
    ) -> Option<Rc<Self::CSSStyleSheet>> {
        None
    }
    fn CreateCSSOMWrapper(&self, _: &Rc<StyleRule<Backend>>, _: usize) -> Rc<Self::CSSRule> {
        Rc::new(())
    }
    fn TrackRule(
        &self,
        _: &mut Self::StyleRuleUsageTracker,
        _: Option<Rc<Self::CSSStyleSheet>>,
        _: &Rc<StyleRule<Backend>>,
    ) {
    }
}
