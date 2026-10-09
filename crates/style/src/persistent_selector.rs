// Real PersistentDocument collaborator for the translated SelectorQuery and
// SelectorChecker. Arena indices only name live DOM nodes; they never cache
// element attributes or tree edges. Rc handles are interned by the owning
// service, so identity survives repeated queries and mutations.
#![allow(non_snake_case, unused_variables)]
use crate::css_selector::*;
use crate::css_selector_list::CSSSelectorList;
use crate::parser::css_selector_parser::{CSSSelectorParser, SelectorParserContext, SelectorParserOptions};
use crate::resolver::match_flags::MatchFlags;
use crate::selector_checker::*;
use crate::selector_filter::*;
use crate::selector_query::*;
use dom::persistent_document::{
    CompatibilityMode, DOMAttribute, DOMNamespace, DOMNodeType, DOMOwnerHandle,
};
use dom::user_interaction_state::UserInteractionState;
use dom::Document;
use foundation::{AtomicString, Persistent};
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::computed_style_constants::PseudoId;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

pub struct HasArgument {
    relation: RelationType,
    depth: i32,
    adjacent: i32,
    sibling_right: bool,
    sibling_between: bool,
    shadow: bool,
    hashes: HashSet<u32>,
}
pub struct HasTraversal {
    nodes: Vec<(Rc<usize>, i32)>,
    position: usize,
}
pub struct HasCache {
    results: RefCell<HashMap<usize, u8>>,
    hashes: HashSet<u32>,
}
pub struct HasFilter {
    allocated: Cell<bool>,
    hashes: RefCell<HashSet<u32>>,
}
// cpp: style_scope_frame.h:34-124; rule_set.cc:1068-1075.
// These records own the compiled boundaries and parent chain for a rule.
// Frame caches live only for one match request, so DOM mutation cannot leave
// stale activations in the document's retained selector service.
pub struct PersistentStyleScope {
    from: Option<Rc<CSSSelectorList>>,
    to: Option<Rc<CSSSelectorList>>,
    parent: Option<Rc<Self>>,
    implicit_root: Option<usize>,
}
pub struct PersistentStyleScopeFrame {
    element: Rc<usize>,
    parent: Option<Rc<Self>>,
    data: RefCell<HashMap<usize, Rc<PersistentStyleScopeActivations>>>,
}
pub struct PersistentStyleScopeActivation {
    root: Option<Rc<usize>>,
    proximity: u32,
}
pub struct PersistentStyleScopeActivations {
    vector: Vec<Rc<PersistentStyleScopeActivation>>,
    flags: MatchFlags,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScopedSelectorMatch {
    pub specificity: u32,
    pub proximity: u32,
}
pub struct PersistentSelectorBackend<'d> {
    document: &'d Document,
    nodes: Vec<Rc<usize>>,
    interaction: UserInteractionState,
    forced: HashMap<u64, Vec<PseudoType>>,
    indeterminate: HashSet<u64>,
    frame_active: bool,
    slot_assignment_dirty: HashSet<u64>,
    vtt_elements: HashSet<u64>,
    has_depth: Cell<u32>,
    query_depth: Cell<u32>,
    nth_depth: Cell<u32>,
    element_flags: RefCell<HashMap<usize, Vec<ElementInvalidationFlag>>>,
    container_flags: RefCell<HashMap<usize, Vec<ContainerInvalidationFlag>>>,
}
impl<'d> PersistentSelectorBackend<'d> {
    fn new(
        document: &'d Document,
        nodes: Vec<Rc<usize>>,
        interaction: UserInteractionState,
        forced: HashMap<u64, Vec<PseudoType>>,
        indeterminate: HashSet<u64>,
        frame_active: bool,
        slot_assignment_dirty: HashSet<u64>,
        vtt_elements: HashSet<u64>,
    ) -> Rc<Self> {
        Rc::new(Self {
            document,
            nodes,
            interaction,
            forced,
            indeterminate,
            frame_active,
            slot_assignment_dirty,
            vtt_elements,
            has_depth: Cell::new(0),
            query_depth: Cell::new(0),
            nth_depth: Cell::new(0),
            element_flags: RefCell::new(HashMap::new()),
            container_flags: RefCell::new(HashMap::new()),
        })
    }
    fn handle(&self, index: usize) -> Rc<usize> {
        self.nodes[index].clone()
    }
    fn element(&self, index: usize) -> Option<Rc<usize>> {
        (self.document.Node(index).Type() == DOMNodeType::kElement).then(|| self.handle(index))
    }
    fn attr(&self, index: usize, name: &str) -> Option<&str> {
        self.document
            .Node(index)
            .FindAttribute(name)
            .map(|v| v.value.as_str())
    }
    fn scope(&self, index: usize) -> usize {
        let mut i = index;
        while let Some(p) = self.document.Node(i).Parent() {
            i = p;
        }
        i
    }
    fn sibling(&self, index: usize, forward: bool, elements: bool) -> Option<Rc<usize>> {
        let siblings = self
            .document
            .Node(self.document.Node(index).Parent()?)
            .Children();
        let position = siblings.iter().position(|&v| v == index)?;
        let candidates: Box<dyn Iterator<Item = &usize>> = if forward {
            Box::new(siblings[position + 1..].iter())
        } else {
            Box::new(siblings[..position].iter().rev())
        };
        candidates
            .filter(|&&v| !elements || self.document.Node(v).Type() == DOMNodeType::kElement)
            .next()
            .map(|&v| self.handle(v))
    }
    fn descendants(&self, index: usize, out: &mut Vec<Rc<usize>>) {
        for &child in self.document.Node(index).Children() {
            if let Some(e) = self.element(child) {
                out.push(e);
            }
            self.descendants(child, out);
        }
    }
    fn class_contains(&self, index: usize, class: &AtomicString) -> bool {
        self.attr(index, "class").is_some_and(|v| {
            v.split_ascii_whitespace().any(|v| {
                if self.InQuirksMode(&index) {
                    v.eq_ignore_ascii_case(&class.Utf8())
                } else {
                    v == class.Utf8()
                }
            })
        })
    }
    fn namespace(namespace: DOMNamespace) -> AtomicString {
        AtomicString::from_str(match namespace {
            DOMNamespace::kHTML => "http://www.w3.org/1999/xhtml",
            DOMNamespace::kSVG => "http://www.w3.org/2000/svg",
            DOMNamespace::kMathML => "http://www.w3.org/1998/Math/MathML",
            DOMNamespace::kNone => "",
        })
    }
    fn bloom(&self, index: usize) -> u32 {
        let mut bits = 0;
        if let Some(classes) = self.attr(index, "class") {
            for c in classes.split_ascii_whitespace() {
                let atom = AtomicString::from_str(c);
                bits |= FilterForString(&if self.InQuirksMode(&index) {
                    atom.ToAsciiLower()
                } else {
                    atom
                });
            }
        }
        for a in self.document.Node(index).Attributes() {
            let name = QualifiedName::new(
                AtomicString::from_str(&a.prefix),
                AtomicString::from_str(&a.local_name),
                AtomicString::from_str(&a.namespace_uri),
            );
            if !self.IsExcludedAttribute(
                &name,
                AttributesToExcludeHashesFor::kExcludeAllLazilySynchronizedAttributes,
            ) {
                bits |= FilterForAttribute(&name);
            }
        }
        for &child in self.document.Node(index).Children() {
            bits |= self.bloom(child);
        }
        bits
    }
    fn tag_matches(&self, index: usize, name: &QualifiedName) -> bool {
        self.ElementTagQName(&index) == *name
    }
    fn identifier_hashes(&self, index: usize) -> HashSet<u32> {
        let mut hashes = HashSet::new();
        hashes.insert(self.ElementLocalName(&index).Hash().wrapping_mul(13));
        if let Some(id) = self.attr(index, "id") {
            hashes.insert(self.IdForStyleResolution(&index).Hash().wrapping_mul(17));
        }
        if let Some(classes) = self.attr(index, "class") {
            for c in classes.split_ascii_whitespace() {
                let atom = AtomicString::from_str(c);
                hashes.insert(
                    if self.InQuirksMode(&index) {
                        atom.ToAsciiLower()
                    } else {
                        atom
                    }
                    .Hash()
                    .wrapping_mul(19),
                );
            }
        }
        for a in self.document.Node(index).Attributes() {
            hashes.insert(
                AtomicString::from_str(&a.local_name)
                    .ToAsciiLower()
                    .Hash()
                    .wrapping_mul(23),
            );
        }
        hashes
    }
    fn has_walk(&self, index: usize, depth: i32, limit: i32, out: &mut Vec<(Rc<usize>, i32)>) {
        out.push((self.handle(index), depth));
        if depth < limit {
            for &child in self.document.Node(index).Children() {
                if self.element(child).is_some() {
                    self.has_walk(child, depth + 1, limit, out);
                }
            }
        }
    }
}
impl SelectorQueryFilterBackend for PersistentSelectorBackend<'_> {
    fn IsExcludedAttribute(
        &self,
        name: &QualifiedName,
        policy: AttributesToExcludeHashesFor,
    ) -> bool {
        // PersistentDocument synchronizes all stored attributes eagerly; only
        // the standard identifiers excluded by Blink remain excluded.
        matches!(name.LocalName().Utf8().as_str(), "id" | "class" | "style")
    }
}
impl SelectorQueryBackend for PersistentSelectorBackend<'_> {
    type QueryNthIndexCache = ();
    fn QueryParentNode(&self, node: &usize) -> Option<Rc<usize>> {
        self.document.Node(*node).Parent().map(|v| self.handle(v))
    }
    fn QueryPreviousSiblingNode(&self, node: &usize) -> Option<Rc<usize>> {
        self.sibling(*node, false, false)
    }
    fn QueryFirstChildElement(&self, node: &usize) -> Option<Rc<usize>> {
        self.document
            .Node(*node)
            .Children()
            .iter()
            .find_map(|&v| self.element(v))
    }
    fn QueryNextSiblingElement(&self, node: &usize) -> Option<Rc<usize>> {
        self.sibling(*node, true, true)
    }
    fn QueryIsShadowRoot(&self, node: &usize) -> bool {
        self.TreeScopeIsShadowRoot(node)
    }
    fn QueryIsHTMLDocument(&self, node: &usize) -> bool {
        self.IsInHTMLDocument(node)
    }
    fn QueryInQuirksMode(&self, node: &usize) -> bool {
        self.InQuirksMode(node)
    }
    fn QueryIsInTreeScope(&self, node: &usize) -> bool {
        self.document.Node(self.scope(*node)).Type() == DOMNodeType::kDocument
    }
    fn QueryTreeScope(&self, node: &usize) -> Rc<usize> {
        self.handle(self.scope(*node))
    }
    fn QuerySubtreeBloomFilter(&self, e: &usize) -> u32 {
        self.bloom(*e)
    }
    fn QueryContainsMultipleElementsWithId(&self, scope: &usize, id: &AtomicString) -> bool {
        self.QueryAllElementsById(scope, id).len() > 1
    }
    fn QueryAllElementsById(&self, scope: &usize, id: &AtomicString) -> Vec<Rc<usize>> {
        let mut out = vec![];
        self.descendants(*scope, &mut out);
        out.retain(|v| self.IdForStyleResolution(v) == *id);
        out
    }
    fn QueryElementById(&self, scope: &usize, id: &AtomicString) -> Option<Rc<usize>> {
        self.QueryAllElementsById(scope, id).into_iter().next()
    }
    fn EnterQueryHasCacheScope(&self, root: &usize) -> bool {
        self.query_depth.set(self.query_depth.get() + 1);
        false
    }
    fn EnterQueryNthIndexCache(&self, root: &usize) {
        self.nth_depth.set(self.nth_depth.get() + 1);
    }
    fn ExitQueryNthIndexCache(&self, cache: ()) {
        self.nth_depth.set(self.nth_depth.get() - 1);
    }
}
impl SelectorCheckerBackend for PersistentSelectorBackend<'_> {
    type Element = usize;
    type ContainerNode = usize;
    type TreeScope = usize;
    type StyleScope = PersistentStyleScope;
    type StyleScopeFrame = PersistentStyleScopeFrame;
    type StyleScopeActivations = PersistentStyleScopeActivations;
    type StyleScopeActivation = PersistentStyleScopeActivation;
    type ElementResolveContext = ();
    type PartNames = ();
    type CustomScrollbar = ();
    type ScrollbarPart = ();
    type StyleRequest = ();
    type Attribute = DOMAttribute;
    type ViewTransition = ();
    type Node = usize;
    type HasArgumentContext = HasArgument;
    type HasTraversal = HasTraversal;
    type HasCacheScope = bool;
    type HasCacheContext = HasCache;
    type HasFastRejectFilter = HasFilter;
    fn ResolveUltimateOriginatingElementOrSelf(
        &self,
        c: &Self::ElementResolveContext,
    ) -> Rc<Self::Element> {
        panic!("Chromium DOM/runtime collaborator pending: ResolveUltimateOriginatingElementOrSelf")
    }
    fn ResolvePseudoElement(&self, c: &Self::ElementResolveContext) -> Option<Rc<Self::Element>> {
        panic!("Chromium DOM/runtime collaborator pending: ResolvePseudoElement")
    }
    fn ResolvePseudoElementAncestors(
        &self,
        c: &Self::ElementResolveContext,
    ) -> Vec<Rc<Self::Element>> {
        panic!("Chromium DOM/runtime collaborator pending: ResolvePseudoElementAncestors")
    }
    fn RequestScrollbar(&self, r: &Self::StyleRequest) -> Option<Rc<Self::CustomScrollbar>> {
        panic!("Chromium DOM/runtime collaborator pending: RequestScrollbar")
    }
    fn RequestPseudoArgument(&self, r: &Self::StyleRequest) -> AtomicString {
        panic!("Chromium DOM/runtime collaborator pending: RequestPseudoArgument")
    }
    fn RequestPseudoIdentList(&self, r: &Self::StyleRequest) -> Vec<AtomicString> {
        panic!("Chromium DOM/runtime collaborator pending: RequestPseudoIdentList")
    }
    fn RequestScrollbarPart(&self, r: &Self::StyleRequest) -> Self::ScrollbarPart {
        panic!("Chromium DOM/runtime collaborator pending: RequestScrollbarPart")
    }
    fn NoScrollbarPart(&self) -> Self::ScrollbarPart {
        ()
    }
    fn TreeScopeShadowHost(&self, s: &Self::TreeScope) -> Option<Rc<Self::Element>> {
        None
    }
    fn TreeScopeIsShadowRoot(&self, s: &Self::TreeScope) -> bool {
        self.document.Node(*s).Type() == DOMNodeType::kDocumentFragment
            && self.attr(*s, "shadowrootmode").is_some()
    }
    fn ParentTreeScope(&self, s: &Self::TreeScope) -> Option<Rc<Self::TreeScope>> {
        None
    }
    fn ElementTreeScope(&self, e: &Self::Element) -> Rc<Self::TreeScope> {
        self.handle(self.scope(*e))
    }
    fn ParentElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        self.document
            .Node(*e)
            .Parent()
            .and_then(|v| self.element(v))
    }
    fn ParentOrShadowHostElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        self.ParentElement(e)
    }
    fn PreviousSibling(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        self.sibling(*e, false, true)
    }
    fn NextSibling(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        self.sibling(*e, true, true)
    }
    fn PreviousSiblingWithTagName(
        &self,
        e: &Self::Element,
        name: &QualifiedName,
    ) -> Option<Rc<Self::Element>> {
        let mut current = self.PreviousSibling(e);
        while let Some(v) = current {
            if self.tag_matches(*v, name) {
                return Some(v);
            }
            current = self.PreviousSibling(&v);
        }
        None
    }
    fn NextSiblingWithTagName(
        &self,
        e: &Self::Element,
        name: &QualifiedName,
    ) -> Option<Rc<Self::Element>> {
        let mut current = self.NextSibling(e);
        while let Some(v) = current {
            if self.tag_matches(*v, name) {
                return Some(v);
            }
            current = self.NextSibling(&v);
        }
        None
    }
    fn AssignedSlot(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        panic!("Chromium DOM/runtime collaborator pending: AssignedSlot")
    }
    fn IsSlotSupportingAssignment(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: IsSlotSupportingAssignment")
    }
    fn InQuirksMode(&self, e: &Self::Element) -> bool {
        self.document.GetCompatibilityMode() == CompatibilityMode::kQuirks
    }
    fn IsLink(&self, e: &Self::Element) -> bool {
        matches!(self.document.Node(*e).Name(), "a" | "area" | "link")
            && self.attr(*e, "href").is_some()
    }
    fn IsPseudoElement(&self, e: &Self::Element) -> bool {
        self.document.Node(*e).Type() != DOMNodeType::kElement
    }
    fn IsScrollButtonPseudoElement(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: IsScrollButtonPseudoElement")
    }
    fn ParentComputedStyle(&self, e: &Self::Element) -> Option<Persistent<ComputedStyle>> {
        panic!("Chromium DOM/runtime collaborator pending: ParentComputedStyle")
    }
    fn ScrollButtonPseudoIdFromArgument(
        &self,
        argument: &AtomicString,
        style: &ComputedStyle,
    ) -> PseudoId {
        panic!("Chromium DOM/runtime collaborator pending: ScrollButtonPseudoIdFromArgument")
    }
    fn ElementPseudoId(&self, e: &Self::Element) -> PseudoId {
        PseudoId::kPseudoIdNone
    }
    fn TransitionForElement(&self, e: &Self::Element) -> Option<Rc<Self::ViewTransition>> {
        panic!("Chromium DOM/runtime collaborator pending: TransitionForElement")
    }
    fn CSSLogicalCombinationPseudoEnabled(&self) -> bool {
        true
    }
    fn ParentElementOrShadowRoot(&self, e: &Self::Element) -> Option<Rc<Self::ContainerNode>> {
        self.document
            .Node(*e)
            .Parent()
            .filter(|&p| self.document.Node(p).Type() == DOMNodeType::kElement)
            .map(|p| self.handle(p))
    }
    fn SetChildrenAffectedByDirectAdjacentRules(&self, node: &Self::ContainerNode) {
        self.container_flags
            .borrow_mut()
            .entry(*node)
            .or_default()
            .push(ContainerInvalidationFlag::ChildrenAffectedByForwardPositionalRules);
    }
    fn SetChildrenAffectedByIndirectAdjacentRules(&self, node: &Self::ContainerNode) {
        self.container_flags
            .borrow_mut()
            .entry(*node)
            .or_default()
            .push(ContainerInvalidationFlag::ChildrenAffectedByForwardPositionalRules);
    }
    fn OwnerShadowHost(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        panic!("Chromium DOM/runtime collaborator pending: OwnerShadowHost")
    }
    fn ActivationMatchFlags(&self, activations: &Self::StyleScopeActivations) -> MatchFlags {
        activations.flags
    }
    fn ActivationVector(
        &self,
        activations: &Self::StyleScopeActivations,
    ) -> Vec<Rc<Self::StyleScopeActivation>> {
        activations.vector.clone()
    }
    fn ActivationRoot(
        &self,
        activation: &Self::StyleScopeActivation,
    ) -> Option<Rc<Self::ContainerNode>> {
        activation.root.clone()
    }
    fn ActivationProximity(&self, activation: &Self::StyleScopeActivation) -> u32 {
        activation.proximity
    }
    fn SynchronizeAttribute(&self, e: &Self::Element, name: &AtomicString) { // PersistentDocument::SetAttribute commits attribute values synchronously.
    }
    fn CouldHaveAttribute(&self, e: &Self::Element, name: &QualifiedName) -> bool {
        self.document
            .Node(*e)
            .Attributes()
            .iter()
            .any(|a| self.AttributeMatchesName(a, name))
    }
    fn CSSAttributeValueCaseSensitiveNonHTMLEnabled(&self) -> bool {
        true
    }
    fn IsHTMLElement(&self, e: &Self::Element) -> bool {
        self.document.Node(*e).Namespace() == DOMNamespace::kHTML
    }
    fn IsInHTMLDocument(&self, e: &Self::Element) -> bool {
        self.document
            .Node(self.document.Root())
            .Children()
            .iter()
            .any(|&v| self.document.Node(v).IsHTMLElement("html"))
    }
    fn AttributesWithoutUpdate(&self, e: &Self::Element) -> Vec<Rc<Self::Attribute>> {
        self.document
            .Node(*e)
            .Attributes()
            .iter()
            .cloned()
            .map(Rc::new)
            .collect()
    }
    fn AttributeMatchesName(&self, a: &Self::Attribute, name: &QualifiedName) -> bool {
        AtomicString::from_str(&a.local_name) == *name.LocalName()
            && (name.NamespaceURI().IsNull() && a.namespace_uri.is_empty()
                || name.NamespaceURI() == &AtomicString::from_str("*")
                || AtomicString::from_str(&a.namespace_uri) == *name.NamespaceURI())
    }
    fn AttributeMatchesNameCaseInsensitive(
        &self,
        a: &Self::Attribute,
        name: &QualifiedName,
    ) -> bool {
        a.local_name.eq_ignore_ascii_case(&name.LocalName().Utf8())
            && (name.NamespaceURI().IsNull() && a.namespace_uri.is_empty()
                || name.NamespaceURI() == &AtomicString::from_str("*")
                || AtomicString::from_str(&a.namespace_uri) == *name.NamespaceURI())
    }
    fn AttributeValue(&self, a: &Self::Attribute) -> AtomicString {
        AtomicString::from_str(&a.value)
    }
    fn CouldHaveClass(&self, e: &Self::Element, class: &AtomicString) -> bool {
        self.class_contains(*e, class)
    }
    fn HasClass(&self, e: &Self::Element) -> bool {
        self.attr(*e, "class").is_some_and(|v| !v.is_empty())
    }
    fn ClassNamesContain(&self, e: &Self::Element, class: &AtomicString) -> bool {
        self.class_contains(*e, class)
    }
    fn HasID(&self, e: &Self::Element) -> bool {
        self.attr(*e, "id").is_some_and(|v| !v.is_empty())
    }
    fn IdForStyleResolution(&self, e: &Self::Element) -> AtomicString {
        self.attr(*e, "id")
            .filter(|v| !v.is_empty())
            .map(|v| {
                let a = AtomicString::from_str(v);
                if self.InQuirksMode(e) {
                    a.ToAsciiLower()
                } else {
                    a
                }
            })
            .unwrap_or_default()
    }
    fn ElementIs(&self, e: &Self::Element, class: ElementClass) -> bool {
        let node = self.document.Node(*e);
        let name = node.Name();
        use ElementClass::*;
        match class {
            Option => node.IsHTMLElement("option"),
            Input => node.IsHTMLElement("input"),
            Select => node.IsHTMLElement("select"),
            SVG => node.Namespace() == DOMNamespace::kSVG,
            Image => node.IsHTMLElement("img"),
            TextControl => node.IsHTMLElement("textarea") || self.IsTextField(e),
            Form => node.IsHTMLElement("form"),
            HTMLElement => node.Namespace() == DOMNamespace::kHTML,
            FormControl | FormControlWithState => {
                node.Namespace() == DOMNamespace::kHTML
                    && matches!(
                        name,
                        "button"
                            | "fieldset"
                            | "input"
                            | "object"
                            | "output"
                            | "select"
                            | "textarea"
                            | "option"
                            | "optgroup"
                    )
            }
            Anchor => matches!(name, "a" | "area"),
            Dialog => node.IsHTMLElement("dialog"),
            Details => node.IsHTMLElement("details"),
            Media => matches!(name, "audio" | "video"),
            Video => node.IsHTMLElement("video"),
            Slot => node.IsHTMLElement("slot"),
            VTT => self.vtt_elements.contains(&node.Id()),
            MenuItem => node.IsHTMLElement("menuitem"),
            _ => panic!("DOM class collaborator pending: {:?}", class),
        }
    }
    fn SetElementFlag(&self, e: &Self::Element, flag: ElementInvalidationFlag) {
        self.element_flags
            .borrow_mut()
            .entry(*e)
            .or_default()
            .push(flag);
    }
    fn SetContainerFlag(&self, n: &Self::ContainerNode, flag: ContainerInvalidationFlag) {
        self.container_flags
            .borrow_mut()
            .entry(*n)
            .or_default()
            .push(flag);
    }
    fn FeatureEnabled(&self, e: &Self::Element, feature: SelectorRuntimeFeature) -> bool {
        match feature {
            SelectorRuntimeFeature::CSSLangExtendedRanges => true,
            _ => panic!(
                "runtime feature not admitted by capability preflight: {:?}",
                feature
            ),
        }
    }
    fn ElementLocalName(&self, e: &Self::Element) -> AtomicString {
        AtomicString::from_str(self.document.Node(*e).Name())
    }
    fn ElementNamespaceURI(&self, e: &Self::Element) -> AtomicString {
        Self::namespace(self.document.Node(*e).Namespace())
    }
    fn ElementTagQName(&self, e: &Self::Element) -> QualifiedName {
        QualifiedName::new(
            AtomicString::default(),
            self.ElementLocalName(e),
            self.ElementNamespaceURI(e),
        )
    }
    fn FrameFocusedAndActive(&self, e: &Self::Element) -> Option<bool> {
        Some(self.frame_active)
    }
    fn MenuOwnerItems(&self, e: &Self::Element) -> Vec<Rc<Self::Element>> {
        panic!("Chromium DOM/runtime collaborator pending: MenuOwnerItems")
    }
    fn CorrespondingSVGElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        panic!("Chromium DOM/runtime collaborator pending: CorrespondingSVGElement")
    }
    fn HasShadowRoot(&self, e: &Self::Element) -> bool {
        self.document
            .Node(*e)
            .Children()
            .iter()
            .any(|&v| self.TreeScopeIsShadowRoot(&v))
    }
    fn ShadowRootIsUserAgent(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: ShadowRootIsUserAgent")
    }
    fn ContainingShadowRootIsUserAgent(&self, e: &Self::Element) -> Option<bool> {
        panic!("Chromium DOM/runtime collaborator pending: ContainingShadowRootIsUserAgent")
    }
    fn UltimateOriginatingElement(&self, e: &Self::Element) -> Rc<Self::Element> {
        panic!("Chromium DOM/runtime collaborator pending: UltimateOriginatingElement")
    }
    fn ShadowPseudoId(&self, e: &Self::Element) -> AtomicString {
        panic!("Chromium DOM/runtime collaborator pending: ShadowPseudoId")
    }
    fn ElementPseudoIdForStyling(&self, e: &Self::Element) -> PseudoId {
        PseudoId::kPseudoIdNone
    }
    fn ElementPseudoArgument(&self, e: &Self::Element) -> AtomicString {
        panic!("Chromium DOM/runtime collaborator pending: ElementPseudoArgument")
    }
    fn ElementViewTransitionClasses(&self, e: &Self::Element) -> Vec<AtomicString> {
        panic!("Chromium DOM/runtime collaborator pending: ElementViewTransitionClasses")
    }
    fn PartNamesContain(&self, p: &Self::PartNames, name: &AtomicString) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: PartNamesContain")
    }
    fn TreeScopeRoot(&self, s: &Self::TreeScope) -> Rc<Self::ContainerNode> {
        self.handle(*s)
    }
    fn ContainerAsElement(&self, n: &Self::ContainerNode) -> Option<Rc<Self::Element>> {
        self.element(*n)
    }
    fn ContainerShadowHost(&self, n: &Self::ContainerNode) -> Option<Rc<Self::Element>> {
        self.TreeScopeIsShadowRoot(n).then(|| self.document.Node(*n).Parent().and_then(|host| self.element(host))).flatten()
    }
    fn ElementAsContainer(&self, e: &Self::Element) -> Rc<Self::ContainerNode> {
        self.handle(*e)
    }
    fn DocumentElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        let root = self.document.Root();
        self.QueryFirstChildElement(&root)
    }
    fn FlatTreeParentElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        self.ParentElement(e)
    }
    fn ParentElementOrDocumentFragment(
        &self,
        e: &Self::Element,
    ) -> Option<Rc<Self::ContainerNode>> {
        self.document
            .Node(*e)
            .Parent()
            .filter(|&p| {
                matches!(
                    self.document.Node(p).Type(),
                    DOMNodeType::kElement | DOMNodeType::kDocumentFragment
                )
            })
            .map(|p| self.handle(p))
    }
    fn IsFinishedParsingChildren(&self, n: &Self::ContainerNode) -> bool {
        true
    }
    fn FirstChildElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        self.QueryFirstChildElement(e)
    }
    fn FirstChildNode(&self, e: &Self::Element) -> Option<Rc<Self::Node>> {
        self.document
            .Node(*e)
            .Children()
            .first()
            .map(|&v| self.handle(v))
    }
    fn NextSiblingNode(&self, n: &Self::Node) -> Option<Rc<Self::Node>> {
        self.sibling(*n, true, false)
    }
    fn NodeIsElement(&self, n: &Self::Node) -> bool {
        self.document.Node(*n).Type() == DOMNodeType::kElement
    }
    fn TextNodeData(&self, n: &Self::Node) -> Option<AtomicString> {
        (self.document.Node(*n).Type() == DOMNodeType::kText)
            .then(|| AtomicString::from_str(self.document.Node(*n).Data()))
    }
    fn NumDescendantInputs(&self, e: &Self::Element) -> usize {
        panic!("Chromium DOM/runtime collaborator pending: NumDescendantInputs")
    }
    fn SlottedButton(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        panic!("Chromium DOM/runtime collaborator pending: SlottedButton")
    }
    fn CssTarget(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        panic!("Chromium DOM/runtime collaborator pending: CssTarget")
    }
    fn ProbeForcePseudoState(&self, e: &Self::Element, pseudo: PseudoType) -> bool {
        self.forced
            .get(&self.document.Node(*e).Id())
            .is_some_and(|v| v.contains(&pseudo))
    }
    fn NthIndex(
        &self,
        e: &Self::Element,
        kind: PseudoType,
        list: Option<&CSSSelectorList>,
        checker: &SelectorChecker<Self>,
        context: &SelectorCheckingContext<'_, Self>,
    ) -> u32 {
        let mut siblings = if let Some(parent) = self.document.Node(*e).Parent() {
            self.document
                .Node(parent)
                .Children()
                .iter()
                .filter_map(|&v| self.element(v))
                .collect::<Vec<_>>()
        } else {
            vec![self.handle(*e)]
        };
        if matches!(
            kind,
            PseudoType::kPseudoNthOfType | PseudoType::kPseudoNthLastOfType
        ) {
            let q = self.ElementTagQName(e);
            siblings.retain(|v| self.ElementTagQName(v) == q);
        }
        if let Some(list) = list {
            siblings.retain(|v| {
                list.ComplexSelectors().any(|selector| {
                    let mut sub = SelectorCheckingContext::new(v.clone());
                    sub.selector = Some(selector);
                    sub.scope = context.scope.clone();
                    checker.Match(&sub, &mut MatchResult::default())
                })
            });
        }
        if matches!(
            kind,
            PseudoType::kPseudoNthLastChild | PseudoType::kPseudoNthLastOfType
        ) {
            siblings.reverse();
        }
        siblings
            .iter()
            .position(|v| **v == *e)
            .map(|v| v as u32 + 1)
            .unwrap_or(0)
    }
    fn NavigationLocationMatches(
        &self,
        location: &dyn crate::css_selector::CSSSelectorNavigationLocation,
        e: &Self::Element,
    ) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: NavigationLocationMatches")
    }
    fn OwnerDataListElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        panic!("Chromium DOM/runtime collaborator pending: OwnerDataListElement")
    }
    fn OwnerSelectElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        panic!("Chromium DOM/runtime collaborator pending: OwnerSelectElement")
    }
    fn ActiveOption(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        panic!("Chromium DOM/runtime collaborator pending: ActiveOption")
    }
    fn InterestInvoker(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        panic!("Chromium DOM/runtime collaborator pending: InterestInvoker")
    }
    fn ScrollMarkerSelected(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        panic!("Chromium DOM/runtime collaborator pending: ScrollMarkerSelected")
    }
    fn HasScrollMarkerGroupData(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: HasScrollMarkerGroupData")
    }
    fn CompareLayoutPreorder(&self, a: &Self::Element, b: &Self::Element) -> i32 {
        panic!("Chromium DOM/runtime collaborator pending: CompareLayoutPreorder")
    }
    fn NavigationSourceElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        panic!("Chromium DOM/runtime collaborator pending: NavigationSourceElement")
    }
    fn ComputeInheritedLanguage(&self, e: &Self::Element) -> AtomicString {
        let mut current = Some(*e);
        while let Some(v) = current {
            if let Some(lang) = self.attr(v, "lang") {
                return AtomicString::from_str(lang);
            }
            current = self.document.Node(v).Parent();
        }
        AtomicString::default()
    }
    fn VttLanguage(&self, e: &Self::Element) -> AtomicString {
        panic!("Chromium DOM/runtime collaborator pending: VttLanguage")
    }
    fn SlotAssignmentDirty(&self, e: &Self::Element) -> bool {
        self.slot_assignment_dirty
            .contains(&self.document.Node(*e).Id())
    }
    fn RecalcSlotAssignments(&self, e: &Self::Element) {
        panic!("Chromium DOM/runtime collaborator pending: RecalcSlotAssignments")
    }
    fn CachedDirectionIsRtl(&self, e: &Self::Element) -> bool {
        let mut current = Some(*e);
        while let Some(v) = current {
            if let Some(dir) = self.attr(v, "dir") {
                return dir.eq_ignore_ascii_case("rtl");
            }
            current = self.document.Node(v).Parent();
        }
        false
    }
    fn PopoverInvoker(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        panic!("Chromium DOM/runtime collaborator pending: PopoverInvoker")
    }
    fn OwningMenuElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        panic!("Chromium DOM/runtime collaborator pending: OwningMenuElement")
    }
    fn InternalsHasState(&self, e: &Self::Element, state: &AtomicString) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: InternalsHasState")
    }
    fn GetPseudoElement(&self, e: &Self::Element, pseudo: PseudoId) -> Option<Rc<Self::Element>> {
        panic!("Chromium DOM/runtime collaborator pending: GetPseudoElement")
    }
    fn FocusedElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        self.interaction
            .focused_node_id
            .and_then(|id| self.document.FindNodeById(id))
            .and_then(|v| self.element(v))
    }
    fn FocusVisibleOption(&self, e: &Self::Element) -> Option<bool> {
        Some(self.interaction.focus_visible_node_id == Some(self.document.Node(*e).Id()))
    }
    fn TransitionScope(&self, t: &Self::ViewTransition) -> Rc<Self::Element> {
        panic!("Chromium DOM/runtime collaborator pending: TransitionScope")
    }
    fn TransitionMatchOnlyChild(
        &self,
        t: &Self::ViewTransition,
        id: PseudoId,
        argument: &AtomicString,
    ) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: TransitionMatchOnlyChild")
    }
    fn TransitionMatchActive(&self, t: &Self::ViewTransition) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: TransitionMatchActive")
    }
    fn TransitionMatchActiveType(&self, t: &Self::ViewTransition, idents: &[AtomicString]) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: TransitionMatchActiveType")
    }
    fn ScrollbarEnabled(&self, s: &Self::CustomScrollbar) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: ScrollbarEnabled")
    }
    fn ScrollbarHoveredPart(&self, s: &Self::CustomScrollbar) -> Self::ScrollbarPart {
        panic!("Chromium DOM/runtime collaborator pending: ScrollbarHoveredPart")
    }
    fn ScrollbarPressedPart(&self, s: &Self::CustomScrollbar) -> Self::ScrollbarPart {
        panic!("Chromium DOM/runtime collaborator pending: ScrollbarPressedPart")
    }
    fn ScrollbarPartBits(&self, p: &Self::ScrollbarPart) -> u32 {
        panic!("Chromium DOM/runtime collaborator pending: ScrollbarPartBits")
    }
    fn ScrollbarHorizontal(&self, s: &Self::CustomScrollbar) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: ScrollbarHorizontal")
    }
    fn ScrollbarNativeThemeHasButtons(&self, s: &Self::CustomScrollbar) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: ScrollbarNativeThemeHasButtons")
    }
    fn ScrollbarCornerVisible(&self, s: &Self::CustomScrollbar) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: ScrollbarCornerVisible")
    }
    fn InPseudoHasChecking(&self, e: &Self::Element) -> bool {
        self.has_depth.get() > 0
    }
    fn ExitHasCacheScope(&self, scope: Self::HasCacheScope) {
        if scope {
            self.has_depth.set(self.has_depth.get() - 1);
        } else {
            self.query_depth.set(self.query_depth.get() - 1);
        }
    }
    fn EnterHasCacheScope(
        &self,
        e: &Self::Element,
        within_selector_checking: bool,
    ) -> Self::HasCacheScope {
        if within_selector_checking {
            self.has_depth.set(self.has_depth.get() + 1);
        } else {
            self.query_depth.set(self.query_depth.get() + 1);
        }
        within_selector_checking
    }
    fn CreateHasArgumentContext(
        &self,
        selector: CSSSelectorComplex<'_>,
        scope: Option<&Self::ContainerNode>,
        shadow: bool,
    ) -> Self::HasArgumentContext {
        let mut result = HasArgument {
            relation: RelationType::kSubSelector,
            depth: 0,
            adjacent: 0,
            sibling_right: false,
            sibling_between: false,
            shadow,
            hashes: HashSet::new(),
        };
        let mut child_seen = false;
        let mut sibling_left = false;
        for simple in selector.SimpleSelectors() {
            match simple.Match() {
                MatchType::kId => {
                    result.hashes.insert(simple.Value().Hash().wrapping_mul(17));
                }
                MatchType::kClass => {
                    result.hashes.insert(simple.Value().Hash().wrapping_mul(19));
                }
                MatchType::kTag => {
                    result
                        .hashes
                        .insert(simple.TagQName().LocalName().Hash().wrapping_mul(13));
                }
                _ => {}
            }
            use RelationType::*;
            let relation = simple.Relation();
            match relation {
                kRelativeChild | kChild => {
                    if relation == kRelativeChild {
                        result.relation = relation;
                    }
                    if sibling_left {
                        result.sibling_between = true;
                        sibling_left = false;
                    }
                    child_seen = true;
                    if result.depth != i32::MAX {
                        result.depth += 1;
                    }
                    result.adjacent = 0;
                }
                kRelativeDescendant | kDescendant => {
                    if relation == kRelativeDescendant {
                        result.relation = relation;
                    }
                    if sibling_left {
                        result.sibling_between = true;
                        sibling_left = false;
                    }
                    child_seen = true;
                    result.depth = i32::MAX;
                    result.adjacent = 0;
                }
                kRelativeDirectAdjacent | kDirectAdjacent => {
                    if relation == kRelativeDirectAdjacent {
                        result.relation = relation;
                    }
                    if child_seen {
                        sibling_left = true;
                    } else {
                        result.sibling_right = true;
                    }
                    if result.adjacent != i32::MAX {
                        result.adjacent += 1;
                    }
                }
                kRelativeIndirectAdjacent | kIndirectAdjacent => {
                    if relation == kRelativeIndirectAdjacent {
                        result.relation = relation;
                    }
                    if child_seen {
                        sibling_left = true;
                    } else {
                        result.sibling_right = true;
                    }
                    result.adjacent = i32::MAX;
                }
                _ => {}
            }
        }
        result
    }
    fn HasInvalidShadowTraversal(&self, c: &Self::HasArgumentContext) -> bool {
        c.shadow
            && !matches!(
                c.relation,
                RelationType::kRelativeChild | RelationType::kRelativeDescendant
            )
    }
    fn HasLeftmostRelation(&self, c: &Self::HasArgumentContext) -> RelationType {
        c.relation
    }
    fn HasDepthLimit(&self, c: &Self::HasArgumentContext) -> i32 {
        c.depth
    }
    fn HasDepthFixed(&self, c: &Self::HasArgumentContext) -> bool {
        c.depth != i32::MAX
    }
    fn HasAdjacentDistanceLimit(&self, c: &Self::HasArgumentContext) -> i32 {
        c.adjacent
    }
    fn HasAdjacentDistanceFixed(&self, c: &Self::HasArgumentContext) -> bool {
        c.adjacent != i32::MAX
    }
    fn HasSiblingCombinatorAtRightmost(&self, c: &Self::HasArgumentContext) -> bool {
        c.sibling_right
    }
    fn HasSiblingCombinatorBetweenChildOrDescendant(&self, c: &Self::HasArgumentContext) -> bool {
        c.sibling_between
    }
    fn HasSiblingsAffectedFlags(&self, c: &Self::HasArgumentContext) -> u8 {
        if c.adjacent == 0 {
            0
        } else if c.depth == 0 {
            1
        } else {
            2
        }
    }
    fn SetSiblingsAffectedByHasFlags(&self, e: &Self::Element, flags: u8) {
        self.element_flags
            .borrow_mut()
            .entry(*e)
            .or_default()
            .push(ElementInvalidationFlag::AncestorsOrAncestorSiblingsAffectedByHas);
    }
    fn CreateHasTraversal(
        &self,
        e: &Self::Element,
        c: &Self::HasArgumentContext,
    ) -> Self::HasTraversal {
        let mut nodes = vec![];
        if c.adjacent == 0 {
            for &child in self.document.Node(*e).Children() {
                if self.element(child).is_some() {
                    self.has_walk(child, 1, c.depth, &mut nodes);
                }
            }
        } else {
            let mut distance = 1;
            let mut current = self.NextSibling(e);
            while let Some(v) = current {
                if c.adjacent == i32::MAX || distance == c.adjacent {
                    self.has_walk(*v, 0, c.depth, &mut nodes);
                } else {
                    nodes.push((v.clone(), 0));
                }
                if distance == c.adjacent {
                    break;
                }
                current = self.NextSibling(&v);
                distance += 1;
            }
        }
        nodes.reverse();
        HasTraversal { nodes, position: 0 }
    }
    fn HasTraversalCurrent(&self, it: &Self::HasTraversal) -> Option<Rc<Self::Element>> {
        it.nodes.get(it.position).map(|v| v.0.clone())
    }
    fn HasTraversalDepth(&self, it: &Self::HasTraversal) -> i32 {
        it.nodes[it.position].1
    }
    fn HasTraversalNext(&self, it: &mut Self::HasTraversal) {
        it.position += 1;
    }
    fn CreateHasCacheContext(
        &self,
        e: &Self::Element,
        c: &Self::HasArgumentContext,
    ) -> Self::HasCacheContext {
        HasCache {
            results: RefCell::new(HashMap::new()),
            hashes: c.hashes.clone(),
        }
    }
    fn HasCacheAllowed(&self, c: &Self::HasCacheContext) -> bool {
        true
    }
    fn HasCacheGetResult(&self, c: &Self::HasCacheContext, e: &Self::Element) -> u8 {
        c.results.borrow().get(e).copied().unwrap_or(0)
    }
    fn HasCacheAlreadyChecked(&self, c: &Self::HasCacheContext, e: &Self::Element) -> bool {
        self.HasCacheGetResult(c, e) & 1 != 0
    }
    fn HasCacheSetChecked(&self, c: &Self::HasCacheContext, e: &Self::Element) {
        let mut results = c.results.borrow_mut();
        *results.entry(*e).or_default() |= 1;
    }
    fn HasCacheSetMatched(&self, c: &Self::HasCacheContext, e: &Self::Element) -> u8 {
        let mut results = c.results.borrow_mut();
        let old = results.get(e).copied().unwrap_or(0);
        *results.entry(*e).or_default() |= 3;
        old
    }
    fn HasCacheSetAllTraversedChecked(
        &self,
        c: &Self::HasCacheContext,
        e: &Self::Element,
        depth: i32,
    ) {
        self.HasCacheSetChecked(c, e);
    }
    fn HasCacheEnsureFastRejectFilter(
        &self,
        c: &Self::HasCacheContext,
        e: &Self::Element,
    ) -> (Rc<Self::HasFastRejectFilter>, bool) {
        (
            Rc::new(HasFilter {
                allocated: Cell::new(false),
                hashes: RefCell::new(HashSet::new()),
            }),
            true,
        )
    }
    fn HasBloomAllocated(&self, f: &Self::HasFastRejectFilter) -> bool {
        f.allocated.get()
    }
    fn HasAllocateBloom(&self, f: &Self::HasFastRejectFilter) {
        f.allocated.set(true);
    }
    fn HasAddElementIdentifierHashes(&self, f: &Self::HasFastRejectFilter, e: &Self::Element) {
        f.hashes.borrow_mut().extend(self.identifier_hashes(*e));
    }
    fn HasFastReject(&self, f: &Self::HasFastRejectFilter, c: &Self::HasArgumentContext) -> bool {
        !c.hashes.is_subset(&f.hashes.borrow())
    }
    fn StyleScopeParent(&self, s: &Self::StyleScope) -> Option<Rc<Self::StyleScope>> {
        s.parent.clone()
    }
    fn StyleScopeFrom(&self, s: &Self::StyleScope) -> Option<Rc<CSSSelectorList>> {
        s.from.clone()
    }
    fn StyleScopeTo(&self, s: &Self::StyleScope) -> Option<Rc<CSSSelectorList>> {
        s.to.clone()
    }
    fn ElementTriggersScope(&self, e: &Self::Element, s: &Self::StyleScope) -> bool {
        s.implicit_root == Some(*e)
    }
    fn StyleScopeFrameElement(&self, f: &Self::StyleScopeFrame) -> Rc<Self::Element> {
        f.element.clone()
    }
    fn ParentStyleScopeFrame(
        &self,
        f: &Self::StyleScopeFrame,
        e: &Self::Element,
    ) -> Option<Rc<Self::StyleScopeFrame>> {
        f.parent.as_ref().filter(|p| *p.element == *e).cloned()
    }
    fn InsertScopeActivationCacheEntry(
        &self,
        f: &Self::StyleScopeFrame,
        s: &Self::StyleScope,
    ) -> Option<Rc<Self::StyleScopeActivations>> {
        f.data.borrow().get(&(s as *const _ as usize)).cloned()
    }
    fn SetScopeActivationCacheEntry(
        &self,
        f: &Self::StyleScopeFrame,
        s: &Self::StyleScope,
        a: Rc<Self::StyleScopeActivations>,
    ) {
        f.data.borrow_mut().insert(s as *const _ as usize, a);
    }
    fn CreateActivation(
        &self,
        root: Option<Rc<Self::ContainerNode>>,
        proximity: u32,
    ) -> Rc<Self::StyleScopeActivation> {
        Rc::new(PersistentStyleScopeActivation { root, proximity })
    }
    fn CreateActivations(
        &self,
        vector: Vec<Rc<Self::StyleScopeActivation>>,
        flags: MatchFlags,
    ) -> Rc<Self::StyleScopeActivations> {
        Rc::new(PersistentStyleScopeActivations { vector, flags })
    }
    fn SpatialNavigationFocused(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: SpatialNavigationFocused")
    }
    fn HasDatalist(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: HasDatalist")
    }
    fn IsSubmenuOpen(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: IsSubmenuOpen")
    }
    fn UsesMenuList(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: UsesMenuList")
    }
    fn IsMultiSelectFocused(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: IsMultiSelectFocused")
    }
    fn IsResourceTarget(&self, e: &Self::Element) -> bool {
        self.ElementIs(e, ElementClass::SVG)
            && self.attr(*e, "id").is_some_and(|v| !v.is_empty())
            && self
                .forced
                .get(&self.document.Node(*e).Id())
                .is_some_and(|v| v.contains(&PseudoType::kPseudoTarget))
    }
    fn CachedImageIsAnimated(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: CachedImageIsAnimated")
    }
    fn IsPlaceholderVisible(&self, e: &Self::Element) -> bool {
        self.attr(*e, "placeholder").is_some_and(|v| !v.is_empty())
            && self.document.ControlValue(*e).is_empty()
    }
    fn MatchesToolFormActive(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: MatchesToolFormActive")
    }
    fn IsUnboundedElementActive(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: IsUnboundedElementActive")
    }
    fn MatchesToolSubmitActive(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: MatchesToolSubmitActive")
    }
    fn IsDragged(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: IsDragged")
    }
    fn HasFocusWithin(&self, e: &Self::Element) -> bool {
        let Some(id) = self.interaction.focused_node_id else {
            return false;
        };
        let mut current = self.document.FindNodeById(id);
        while let Some(v) = current {
            if v == *e {
                return true;
            }
            current = self.document.Node(v).Parent();
        }
        false
    }
    fn IsFiltered(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: IsFiltered")
    }
    fn HasInterest(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: HasInterest")
    }
    fn HasFlattenedAssignedNodes(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: HasFlattenedAssignedNodes")
    }
    fn IsHovered(&self, e: &Self::Element) -> bool {
        self.interaction.hovered_node_id == Some(self.document.Node(*e).Id())
    }
    fn IsActive(&self, e: &Self::Element) -> bool {
        self.interaction.pressed_node_id == Some(self.document.Node(*e).Id())
    }
    fn MatchesEnabled(&self, e: &Self::Element) -> bool {
        self.ElementIs(e, ElementClass::FormControl) && !self.MatchesDisabled(e)
    }
    fn IsMediaDocument(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: IsMediaDocument")
    }
    fn MatchesDefault(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: MatchesDefault")
    }
    fn MatchesDisabled(&self, e: &Self::Element) -> bool {
        if !self.ElementIs(e, ElementClass::FormControl) {
            return false;
        }
        if self.attr(*e, "disabled").is_some() {
            return true;
        }
        let mut current = self.document.Node(*e).Parent();
        while let Some(p) = current {
            let parent = self.document.Node(p);
            if parent.IsHTMLElement("optgroup")
                && parent.FindAttribute("disabled").is_some()
                && self.document.Node(*e).IsHTMLElement("option")
            {
                return true;
            }
            if parent.IsHTMLElement("fieldset") && parent.FindAttribute("disabled").is_some() {
                let legend = parent
                    .Children()
                    .iter()
                    .copied()
                    .find(|&v| self.document.Node(v).IsHTMLElement("legend"));
                let mut inside = false;
                let mut ancestor = Some(*e);
                while let Some(v) = ancestor {
                    if Some(v) == legend {
                        inside = true;
                        break;
                    }
                    if v == p {
                        break;
                    }
                    ancestor = self.document.Node(v).Parent();
                }
                if !inside {
                    return true;
                }
            }
            current = parent.Parent();
        }
        false
    }
    fn MatchesReadOnly(&self, e: &Self::Element) -> bool {
        !self.MatchesReadWrite(e)
    }
    fn MatchesReadWrite(&self, e: &Self::Element) -> bool {
        if self.IsTextField(e) || self.document.Node(*e).IsHTMLElement("textarea") {
            !self.MatchesDisabled(e) && self.attr(*e, "readonly").is_none()
        } else {
            let mut current = Some(*e);
            while let Some(v) = current {
                if let Some(value) = self.attr(v, "contenteditable") {
                    return value.is_empty()
                        || value.eq_ignore_ascii_case("true")
                        || value.eq_ignore_ascii_case("plaintext-only");
                }
                current = self.document.Node(v).Parent();
            }
            false
        }
    }
    fn IsOptionalFormControl(&self, e: &Self::Element) -> bool {
        self.ElementIs(e, ElementClass::FormControl) && !self.IsRequiredFormControl(e)
    }
    fn IsRequiredFormControl(&self, e: &Self::Element) -> bool {
        (self.document.Node(*e).IsHTMLElement("textarea")
            || self.document.Node(*e).IsHTMLElement("select")
            || self.document.Node(*e).IsHTMLElement("input")
                && !matches!(
                    self.attr(*e, "type").unwrap_or("text"),
                    "hidden" | "button" | "submit" | "reset" | "image" | "range" | "color"
                ))
            && self.attr(*e, "required").is_some()
    }
    fn MatchesUserInvalid(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: MatchesUserInvalid")
    }
    fn MatchesUserValid(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: MatchesUserValid")
    }
    fn MatchesValidity(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: MatchesValidity")
    }
    fn IsValid(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: IsValid")
    }
    fn ShouldAppearChecked(&self, e: &Self::Element) -> bool {
        self.document.ControlChecked(*e)
    }
    fn ShouldAppearIndeterminate(&self, e: &Self::Element) -> bool {
        self.indeterminate.contains(&self.document.Node(*e).Id())
            || self.document.Node(*e).IsHTMLElement("progress") && self.attr(*e, "value").is_none()
    }
    fn OptionSelected(&self, e: &Self::Element) -> bool {
        self.attr(*e, "selected").is_some()
    }
    fn ScrollMarkerIsSelected(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: ScrollMarkerIsSelected")
    }
    fn IsTextField(&self, e: &Self::Element) -> bool {
        self.document.Node(*e).IsHTMLElement("input")
            && matches!(
                self.attr(*e, "type").unwrap_or("text"),
                "text" | "search" | "email" | "url" | "tel" | "password" | "number"
            )
    }
    fn IsDialogInTopLayer(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: IsDialogInTopLayer")
    }
    fn IsPopoverInTopLayer(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: IsPopoverInTopLayer")
    }
    fn IsPopover(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: IsPopover")
    }
    fn PopoverOpen(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: PopoverOpen")
    }
    fn MatchesOverscrollOpen(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: MatchesOverscrollOpen")
    }
    fn HasOpenAttribute(&self, e: &Self::Element) -> bool {
        self.attr(*e, "open").is_some()
    }
    fn PopupIsVisible(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: PopupIsVisible")
    }
    fn IsPickerVisible(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: IsPickerVisible")
    }
    fn FullscreenFlag(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: FullscreenFlag")
    }
    fn ContainsFullScreenElement(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: ContainsFullScreenElement")
    }
    fn Granted(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: Granted")
    }
    fn IsPictureInPicture(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: IsPictureInPicture")
    }
    fn MediaPaused(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: MediaPaused")
    }
    fn MediaSeeking(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: MediaSeeking")
    }
    fn MediaBuffering(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: MediaBuffering")
    }
    fn MediaStalled(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: MediaStalled")
    }
    fn MediaMuted(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: MediaMuted")
    }
    fn VideoPersistent(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: VideoPersistent")
    }
    fn ContainsPersistentVideo(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: ContainsPersistentVideo")
    }
    fn IsXrOverlay(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: IsXrOverlay")
    }
    fn IsFullscreenElement(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: IsFullscreenElement")
    }
    fn IsInRange(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: IsInRange")
    }
    fn IsOutOfRange(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: IsOutOfRange")
    }
    fn VttIsPastNode(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: VttIsPastNode")
    }
    fn IsDefined(&self, e: &Self::Element) -> bool {
        self.IsHTMLElement(e) && !self.document.Node(*e).Name().contains("-")
    }
    fn GetComputedStyle(&self, e: &Self::Element) -> Option<Persistent<ComputedStyle>> {
        panic!("Chromium DOM/runtime collaborator pending: GetComputedStyle")
    }
    fn HasEffectiveAppearance(&self, style: &ComputedStyle) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: HasEffectiveAppearance")
    }
    fn PageIsActive(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: PageIsActive")
    }
    fn DidAttachInternals(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: DidAttachInternals")
    }
    fn DialogIsModal(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: DialogIsModal")
    }
    fn IsAutofilled(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: IsAutofilled")
    }
    fn IsPreviewed(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: IsPreviewed")
    }
    fn AutofillStateIsPreviewed(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: AutofillStateIsPreviewed")
    }
    fn IsFocused(&self, e: &Self::Element) -> bool {
        self.interaction.focused_node_id == Some(self.document.Node(*e).Id())
    }
    fn AccessibilityAlwaysShowFocus(&self, e: &Self::Element) -> bool {
        self.interaction.focus_visible_node_id == Some(self.document.Node(*e).Id())
    }
    fn MayTriggerVirtualKeyboard(&self, e: &Self::Element) -> bool {
        self.IsTextField(e) || self.document.Node(*e).IsHTMLElement("textarea")
    }
    fn LastFocusTypeIsMouse(&self, e: &Self::Element) -> bool {
        self.interaction.pressed_node_id.is_some()
    }
    fn HadKeyboardEvent(&self, e: &Self::Element) -> bool {
        self.interaction.focus_visible_node_id.is_some()
    }
    fn AffectedByMultipleHas(&self, e: &Self::Element) -> bool {
        self.element_flags
            .borrow()
            .get(e)
            .is_some_and(|v| v.contains(&ElementInvalidationFlag::AffectedByMultipleHas))
    }
}

pub struct PersistentSelectorService {
    cache: SelectorQueryCache,
    owner: Option<(DOMOwnerHandle, CompatibilityMode)>,
    nodes: Vec<Rc<usize>>,
    pub interaction: UserInteractionState,
    pub forced_pseudo_states: HashMap<u64, Vec<PseudoType>>,
    pub indeterminate_controls: HashSet<u64>,
    pub frame_focused_and_active: bool,
    pub slot_assignment_dirty: HashSet<u64>,
    pub vtt_elements: HashSet<u64>,
}
#[derive(Debug)]
pub enum PersistentSelectorError {
    Syntax(SelectorQuerySyntaxError),
    Unsupported(std::string::String),
}
impl From<SelectorQuerySyntaxError> for PersistentSelectorError {
    fn from(value: SelectorQuerySyntaxError) -> Self {
        Self::Syntax(value)
    }
}
impl Default for PersistentSelectorService {
    fn default() -> Self {
        Self {
            cache: SelectorQueryCache::default(),
            owner: None,
            nodes: vec![],
            interaction: Default::default(),
            forced_pseudo_states: HashMap::new(),
            indeterminate_controls: HashSet::new(),
            frame_focused_and_active: true,
            slot_assignment_dirty: HashSet::new(),
            vtt_elements: HashSet::new(),
        }
    }
}
impl PersistentSelectorService {
    fn prepare<'d>(
        &mut self,
        document: &'d Document,
        selector: &str,
    ) -> Result<(Rc<SelectorQuery>, Rc<PersistentSelectorBackend<'d>>), PersistentSelectorError>
    {
        let text = foundation::String::from(selector);
        if let Some(feature) = CSSSelectorParser::UnsupportedFeature(&text) {
            return Err(PersistentSelectorError::Unsupported(format!(
                "selector backend pending: :{}",
                feature
            )));
        }
        let owner = (document.RootHandle(), document.GetCompatibilityMode());
        if self.owner.map(|previous| previous.0) != Some(owner.0) {
            self.nodes.clear();
        }
        if self.owner != Some(owner) {
            self.owner = Some(owner);
            self.cache.Invalidate();
        }
        while self.nodes.len() < document.NodeCount() {
            self.nodes.push(Rc::new(self.nodes.len()));
        }
        let backend = PersistentSelectorBackend::new(
            document,
            self.nodes.clone(),
            self.interaction,
            self.forced_pseudo_states.clone(),
            self.indeterminate_controls.clone(),
            self.frame_focused_and_active,
            self.slot_assignment_dirty.clone(),
            self.vtt_elements.clone(),
        );
        let context = SelectorParserContext {
            html: backend.IsInHTMLDocument(&document.Root()),
            quirks: backend.InQuirksMode(&document.Root()),
        };
        let query = self.cache.AddForSelectorParser(
            &AtomicString::from_str(selector),
            &context,
            &*backend,
        )?;
        Ok((query, backend))
    }
    // cpp: element_rule_collector.cc:551-572; selector_checker.cc:1035-1072,
    // 3897-4081. All activation traversal and limit matching stay in the
    // translated checker; this method supplies the live DOM owners.
    pub fn MatchingScopedSpecificity(
        &mut self,
        document: &Document,
        index: usize,
        list: &CSSSelectorList,
        scopes: &[cssom::CSSStyleScope],
        owner_node_id: u64,
    ) -> Result<Option<ScopedSelectorMatch>, PersistentSelectorError> {
        use crate::parser::css_nesting_type::CSSNestingType;
        let (_, b) = self.prepare(document, "*")?;
        if let Some(feature) = CSSSelectorParser::UnsupportedParsedFeature(&list.CopySelectors()) {
            return Err(PersistentSelectorError::Unsupported(format!("scoped selector backend pending: :{}", feature)));
        }
        let context = SelectorParserContext { html: true, quirks: b.InQuirksMode(&index) };
        // cpp: scoped_style_resolver.cc:446-478. Ownerless sheets have no
        // implicit activation; documentElement is not an invented fallback.
        let implicit_root = document.FindNodeById(owner_node_id)
            .and_then(|i| document.Node(i).Parent())
            .and_then(|i| b.element(i)).map(|i| *i);
        let mut parent = None;
        for record in scopes {
            let parse = |text: &str, nesting| -> Result<Rc<CSSSelectorList>, PersistentSelectorError> {
                let text = foundation::String::from(text);
                let list = CSSSelectorList::AdoptSelectorVector(CSSSelectorParser::ParseScopeBoundary(
                    &text, &context, nesting, &SelectorParserOptions::default()));
                if !list.IsValid() { return Err(PersistentSelectorError::Unsupported("invalid compiled scope boundary".into())); }
                if let Some(feature) = CSSSelectorParser::UnsupportedParsedFeature(&list.CopySelectors()) {
                    return Err(PersistentSelectorError::Unsupported(format!("scope boundary backend pending: :{}", feature)));
                }
                Ok(list)
            };
            let from = match &record.root {
                cssom::CSSScopeRoot::ImplicitStylesheetOwner => None,
                cssom::CSSScopeRoot::ExplicitSelectorList(text) => Some(parse(text, if parent.is_some() { CSSNestingType::kScope } else { CSSNestingType::kNone })?),
            };
            let to = record.limit.as_deref().map(|text| parse(text, CSSNestingType::kScope)).transpose()?;
            parent = Some(Rc::new(PersistentStyleScope { from, to, parent, implicit_root }));
        }
        let mut ancestors = Vec::new();
        let mut ancestor = Some(b.handle(index));
        while let Some(e) = ancestor {
            ancestor = b.ParentOrShadowHostElement(&e);
            ancestors.push(e);
        }
        let mut frame = None;
        for element in ancestors.into_iter().rev() {
            frame = Some(Rc::new(PersistentStyleScopeFrame { element, parent: frame, data: RefCell::new(HashMap::new()) }));
        }
        let checker = SelectorChecker::new(b.clone(), Mode::kResolvingStyle);
        let mut c = SelectorCheckingContext::new(b.handle(index));
        c.tree_scope = Some(b.ElementTreeScope(&index));
        c.style_scope = parent;
        c.style_scope_frame = frame;
        let mut best: Option<ScopedSelectorMatch> = None;
        for selector in list.ComplexSelectors() {
            c.selector = Some(selector);
            let mut result = MatchResult::default();
            if checker.Match(&c, &mut result) {
                let matched = ScopedSelectorMatch { specificity: selector.Specificity(), proximity: result.proximity };
                if best.is_none_or(|old| (matched.specificity, std::cmp::Reverse(matched.proximity)) > (old.specificity, std::cmp::Reverse(old.proximity))) { best = Some(matched); }
            }
        }
        Ok(best)
    }
    pub fn Specificity(
        &mut self,
        document: &Document,
        selector: &str,
    ) -> Result<u32, PersistentSelectorError> {
        let (q, _) = self.prepare(document, selector)?;
        Ok(q.Specificity())
    }
    pub fn MatchingSpecificity(
        &mut self,
        document: &Document,
        index: usize,
        selector: &str,
    ) -> Result<Option<u32>, PersistentSelectorError> {
        let (q, b) = self.prepare(document, selector)?;
        Ok(q.MatchingSpecificity(b.clone(), b.handle(index)))
    }
    pub fn Matches(
        &mut self,
        document: &Document,
        index: usize,
        selector: &str,
    ) -> Result<bool, PersistentSelectorError> {
        let (q, b) = self.prepare(document, selector)?;
        Ok(q.Matches(b.clone(), b.handle(index)))
    }
    pub fn Closest(
        &mut self,
        document: &Document,
        index: usize,
        selector: &str,
    ) -> Result<Option<usize>, PersistentSelectorError> {
        let (q, b) = self.prepare(document, selector)?;
        Ok(q.Closest(b.clone(), b.handle(index)).map(|v| *v))
    }
    pub fn QueryFirst(
        &mut self,
        document: &Document,
        index: usize,
        selector: &str,
    ) -> Result<Option<usize>, PersistentSelectorError> {
        let (q, b) = self.prepare(document, selector)?;
        Ok(q.QueryFirst(b.clone(), b.handle(index)).map(|v| *v))
    }
    pub fn QueryAll(
        &mut self,
        document: &Document,
        index: usize,
        selector: &str,
    ) -> Result<Vec<usize>, PersistentSelectorError> {
        let (q, b) = self.prepare(document, selector)?;
        Ok(q.QueryAll(b.clone(), b.handle(index))
            .into_iter()
            .map(|v| *v)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn element(d: &mut Document, parent: usize, tag: &str, id: &str, class: &str) -> usize {
        let i = d.CreateElementDefault(DOMNamespace::kHTML, tag.into());
        for (name, value) in [("id", id), ("class", class)] {
            if !value.is_empty() {
                d.SetAttribute(
                    i,
                    DOMAttribute {
                        local_name: name.into(),
                        value: value.into(),
                        ..Default::default()
                    },
                );
            }
        }
        d.AppendChild(parent, i);
        i
    }
    #[test]
    fn mapped_queries_read_real_dom_order_scope_and_mutations() {
        let mut owner = dom::DOM::new();
        let d = owner.GetDocumentMut();
        let html = element(d, d.Root(), "html", "", "");
        let root = element(d, html, "main", "root", "");
        let a = element(d, root, "p", "duplicate", "one two");
        d.AppendText(root, "between");
        let b = element(d, root, "p", "duplicate", "two");
        let section = element(d, root, "section", "section", "");
        let leaf = element(d, section, "span", "leaf", "leaf");
        let mut service = PersistentSelectorService::default();
        assert_eq!(
            service.QueryFirst(d, d.Root(), "#duplicate").unwrap(),
            Some(a)
        );
        assert_eq!(service.QueryAll(d, root, "#duplicate").unwrap(), vec![a, b]);
        assert_eq!(service.QueryFirst(d, root, "#root").unwrap(), None);
        assert_eq!(service.QueryAll(d, root, ":scope > p").unwrap(), vec![a, b]);
        assert!(service
            .Matches(d, a, "main > p.one.two:first-child")
            .unwrap());
        assert!(service.Matches(d, b, "p + p:nth-child(2)").unwrap());
        assert_eq!(service.Closest(d, leaf, "main").unwrap(), Some(root));
        assert!(service
            .Matches(d, a, ":is(#duplicate,.absent):not(.absent):where(p)")
            .unwrap());
        assert!(service.Matches(d, root, ":has(> section)").unwrap());
        assert!(!service.Matches(d, root, ":has(> .leaf)").unwrap());
        assert!(service.Matches(d, root, ":has(> section .leaf)").unwrap());
        assert!(service.Matches(d, a, "*").unwrap());
        assert!(!service.Matches(d, a, "p::before").unwrap());
        let identity = service.nodes[a].clone();
        d.SetAttribute(
            a,
            DOMAttribute {
                local_name: "class".into(),
                value: "changed".into(),
                ..Default::default()
            },
        );
        d.Remove(a);
        d.AppendChild(root, a);
        assert_eq!(service.QueryAll(d, root, "p").unwrap(), vec![b, a]);
        assert!(!service.Matches(d, a, ".one").unwrap());
        assert!(service.Matches(d, a, ".changed:last-child").unwrap());
        assert!(Rc::ptr_eq(&identity, &service.nodes[a]));
        assert!(service.QueryFirst(d, root, "p >").is_err());
        assert!(service.QueryAll(d, root, "").is_err());
    }
}
