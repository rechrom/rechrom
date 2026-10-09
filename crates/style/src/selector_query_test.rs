// Test-only DOM owns stable node identities; unexpected checker operations
// panic so a new production dependency cannot silently pass these tests.
#![allow(unused_variables)]
use super::*;
use crate::resolver::match_flags::{MatchFlag, MatchFlags};
use crate::selector_checker::*;
use foundation::Persistent;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::computed_style_constants::PseudoId;

struct Backend {
    nodes: Vec<Rc<usize>>,
    parents: Vec<Option<usize>>,
    children: Vec<Vec<usize>>,
    tags: Vec<AtomicString>,
    classes: Vec<Vec<AtomicString>>,
    ids: Vec<AtomicString>,
    attrs: Vec<Vec<(QualifiedName, AtomicString)>>,
    quirks: bool,
    has_depth: Cell<u32>,
    nth_depth: Cell<u32>,
}
impl Backend {
    fn new(parents: &[Option<usize>], classes: &[&str], ids: &[&str]) -> Rc<Self> {
        let mut children = vec![vec![]; parents.len()];
        for (i, &p) in parents.iter().enumerate() {
            if let Some(p) = p {
                children[p].push(i);
            }
        }
        Rc::new(Self {
            nodes: (0..parents.len()).map(Rc::new).collect(),
            parents: parents.to_vec(),
            children,
            tags: (0..parents.len())
                .map(|_| AtomicString::from_str("div"))
                .collect(),
            classes: classes
                .iter()
                .map(|s| s.split_whitespace().map(AtomicString::from_str).collect())
                .collect(),
            ids: ids
                .iter()
                .map(|s| {
                    if s.is_empty() {
                        AtomicString::default()
                    } else {
                        AtomicString::from_str(s)
                    }
                })
                .collect(),
            attrs: vec![vec![]; parents.len()],
            quirks: false,
            has_depth: Cell::new(0),
            nth_depth: Cell::new(0),
        })
    }
    fn previous(&self, index: usize) -> Option<Rc<usize>> {
        let siblings = &self.children[self.parents[index]?];
        let pos = siblings.iter().position(|&s| s == index)?;
        pos.checked_sub(1).map(|p| self.nodes[siblings[p]].clone())
    }
    fn next(&self, index: usize) -> Option<Rc<usize>> {
        let siblings = &self.children[self.parents[index]?];
        let pos = siblings.iter().position(|&s| s == index)?;
        siblings.get(pos + 1).map(|&p| self.nodes[p].clone())
    }
    fn bloom(&self, index: usize) -> u32 {
        let mut bits = 0;
        for class in &self.classes[index] {
            bits |= crate::selector_filter::FilterForString(class);
        }
        for (name, _) in &self.attrs[index] {
            if !self.IsExcludedAttribute(
                name,
                AttributesToExcludeHashesFor::kExcludeAllLazilySynchronizedAttributes,
            ) {
                bits |= crate::selector_filter::FilterForAttribute(name);
            }
        }
        for &child in &self.children[index] {
            bits |= self.bloom(child);
        }
        bits
    }
}
impl SelectorQueryFilterBackend for Backend {
    fn IsExcludedAttribute(
        &self,
        name: &QualifiedName,
        policy: AttributesToExcludeHashesFor,
    ) -> bool {
        matches!(name.LocalName().Utf8().as_str(), "id" | "class" | "style")
    }
}
impl SelectorQueryBackend for Backend {
    type QueryNthIndexCache = ();
    fn QueryParentNode(&self, node: &usize) -> Option<Rc<usize>> {
        self.parents[*node].map(|p| self.nodes[p].clone())
    }
    fn QueryPreviousSiblingNode(&self, node: &usize) -> Option<Rc<usize>> {
        self.previous(*node)
    }
    fn QueryFirstChildElement(&self, node: &usize) -> Option<Rc<usize>> {
        self.children[*node].first().map(|&c| self.nodes[c].clone())
    }
    fn QueryNextSiblingElement(&self, node: &usize) -> Option<Rc<usize>> {
        self.next(*node)
    }
    fn QueryIsShadowRoot(&self, node: &usize) -> bool {
        false
    }
    fn QueryIsHTMLDocument(&self, node: &usize) -> bool {
        true
    }
    fn QueryInQuirksMode(&self, node: &usize) -> bool {
        self.quirks
    }
    fn QueryIsInTreeScope(&self, node: &usize) -> bool {
        true
    }
    fn QueryTreeScope(&self, node: &usize) -> Rc<usize> {
        self.nodes[0].clone()
    }
    fn QuerySubtreeBloomFilter(&self, e: &usize) -> u32 {
        self.bloom(*e)
    }
    fn QueryContainsMultipleElementsWithId(&self, scope: &usize, id: &AtomicString) -> bool {
        self.ids.iter().filter(|s| *s == id).count() > 1
    }
    fn QueryAllElementsById(&self, scope: &usize, id: &AtomicString) -> Vec<Rc<usize>> {
        (1..self.nodes.len())
            .filter(|&i| &self.ids[i] == id)
            .map(|i| self.nodes[i].clone())
            .collect()
    }
    fn QueryElementById(&self, scope: &usize, id: &AtomicString) -> Option<Rc<usize>> {
        self.QueryAllElementsById(scope, id).into_iter().next()
    }
    fn EnterQueryHasCacheScope(&self, root: &usize) -> () {
        self.has_depth.set(self.has_depth.get() + 1);
    }
    fn EnterQueryNthIndexCache(&self, root: &usize) -> () {
        self.nth_depth.set(self.nth_depth.get() + 1);
    }
    fn ExitQueryNthIndexCache(&self, cache: ()) {
        self.nth_depth.set(self.nth_depth.get() - 1);
    }
}
impl SelectorCheckerBackend for Backend {
    type Element = usize;
    type ContainerNode = usize;
    type TreeScope = usize;
    type StyleScope = ();
    type StyleScopeFrame = ();
    type StyleScopeActivations = ();
    type StyleScopeActivation = ();
    type ElementResolveContext = ();
    type PartNames = ();
    type CustomScrollbar = ();
    type ScrollbarPart = ();
    type StyleRequest = ();
    type Attribute = (QualifiedName, AtomicString);
    type ViewTransition = ();
    type Node = usize;
    type HasArgumentContext = ();
    type HasTraversal = ();
    type HasCacheScope = ();
    type HasCacheContext = ();
    type HasFastRejectFilter = ();
    fn ResolveUltimateOriginatingElementOrSelf(
        &self,
        c: &Self::ElementResolveContext,
    ) -> Rc<Self::Element> {
        panic!("unexpected test dependency: ResolveUltimateOriginatingElementOrSelf")
    }
    fn ResolvePseudoElement(&self, c: &Self::ElementResolveContext) -> Option<Rc<Self::Element>> {
        panic!("unexpected test dependency: ResolvePseudoElement")
    }
    fn ResolvePseudoElementAncestors(
        &self,
        c: &Self::ElementResolveContext,
    ) -> Vec<Rc<Self::Element>> {
        panic!("unexpected test dependency: ResolvePseudoElementAncestors")
    }
    fn RequestScrollbar(&self, r: &Self::StyleRequest) -> Option<Rc<Self::CustomScrollbar>> {
        panic!("unexpected test dependency: RequestScrollbar")
    }
    fn RequestPseudoArgument(&self, r: &Self::StyleRequest) -> AtomicString {
        panic!("unexpected test dependency: RequestPseudoArgument")
    }
    fn RequestPseudoIdentList(&self, r: &Self::StyleRequest) -> Vec<AtomicString> {
        panic!("unexpected test dependency: RequestPseudoIdentList")
    }
    fn RequestScrollbarPart(&self, r: &Self::StyleRequest) -> Self::ScrollbarPart {
        panic!("unexpected test dependency: RequestScrollbarPart")
    }
    fn NoScrollbarPart(&self) -> Self::ScrollbarPart {
        ()
    }
    fn TreeScopeShadowHost(&self, s: &Self::TreeScope) -> Option<Rc<Self::Element>> {
        None
    }
    fn TreeScopeIsShadowRoot(&self, s: &Self::TreeScope) -> bool {
        false
    }
    fn ParentTreeScope(&self, s: &Self::TreeScope) -> Option<Rc<Self::TreeScope>> {
        panic!("unexpected test dependency: ParentTreeScope")
    }
    fn ElementTreeScope(&self, e: &Self::Element) -> Rc<Self::TreeScope> {
        self.nodes[0].clone()
    }
    fn ParentElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        self.parents[*e]
            .filter(|&p| p != 0)
            .map(|p| self.nodes[p].clone())
    }
    fn ParentOrShadowHostElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        self.ParentElement(e)
    }
    fn PreviousSibling(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        self.previous(*e)
    }
    fn NextSibling(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        self.next(*e)
    }
    fn PreviousSiblingWithTagName(
        &self,
        e: &Self::Element,
        name: &QualifiedName,
    ) -> Option<Rc<Self::Element>> {
        panic!("unexpected test dependency: PreviousSiblingWithTagName")
    }
    fn NextSiblingWithTagName(
        &self,
        e: &Self::Element,
        name: &QualifiedName,
    ) -> Option<Rc<Self::Element>> {
        panic!("unexpected test dependency: NextSiblingWithTagName")
    }
    fn AssignedSlot(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        panic!("unexpected test dependency: AssignedSlot")
    }
    fn IsSlotSupportingAssignment(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: IsSlotSupportingAssignment")
    }
    fn InQuirksMode(&self, e: &Self::Element) -> bool {
        self.quirks
    }
    fn IsLink(&self, e: &Self::Element) -> bool {
        false
    }
    fn IsPseudoElement(&self, e: &Self::Element) -> bool {
        false
    }
    fn IsScrollButtonPseudoElement(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: IsScrollButtonPseudoElement")
    }
    fn ParentComputedStyle(&self, e: &Self::Element) -> Option<Persistent<ComputedStyle>> {
        panic!("unexpected test dependency: ParentComputedStyle")
    }
    fn ScrollButtonPseudoIdFromArgument(
        &self,
        argument: &AtomicString,
        style: &ComputedStyle,
    ) -> PseudoId {
        panic!("unexpected test dependency: ScrollButtonPseudoIdFromArgument")
    }
    fn ElementPseudoId(&self, e: &Self::Element) -> PseudoId {
        panic!("unexpected test dependency: ElementPseudoId")
    }
    fn TransitionForElement(&self, e: &Self::Element) -> Option<Rc<Self::ViewTransition>> {
        panic!("unexpected test dependency: TransitionForElement")
    }
    fn CSSLogicalCombinationPseudoEnabled(&self) -> bool {
        true
    }
    fn ParentElementOrShadowRoot(&self, e: &Self::Element) -> Option<Rc<Self::ContainerNode>> {
        panic!("unexpected test dependency: ParentElementOrShadowRoot")
    }
    fn SetChildrenAffectedByDirectAdjacentRules(&self, node: &Self::ContainerNode) {
        panic!("unexpected test dependency: SetChildrenAffectedByDirectAdjacentRules")
    }
    fn SetChildrenAffectedByIndirectAdjacentRules(&self, node: &Self::ContainerNode) {
        panic!("unexpected test dependency: SetChildrenAffectedByIndirectAdjacentRules")
    }
    fn OwnerShadowHost(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        panic!("unexpected test dependency: OwnerShadowHost")
    }
    fn ActivationMatchFlags(&self, activations: &Self::StyleScopeActivations) -> MatchFlags {
        panic!("unexpected test dependency: ActivationMatchFlags")
    }
    fn ActivationVector(
        &self,
        activations: &Self::StyleScopeActivations,
    ) -> Vec<Rc<Self::StyleScopeActivation>> {
        panic!("unexpected test dependency: ActivationVector")
    }
    fn ActivationRoot(
        &self,
        activation: &Self::StyleScopeActivation,
    ) -> Option<Rc<Self::ContainerNode>> {
        panic!("unexpected test dependency: ActivationRoot")
    }
    fn ActivationProximity(&self, activation: &Self::StyleScopeActivation) -> u32 {
        panic!("unexpected test dependency: ActivationProximity")
    }
    fn SynchronizeAttribute(&self, e: &Self::Element, name: &AtomicString) {
        ()
    }
    fn CouldHaveAttribute(&self, e: &Self::Element, name: &QualifiedName) -> bool {
        self.attrs[*e].iter().any(|a| &a.0 == name)
    }
    fn CSSAttributeValueCaseSensitiveNonHTMLEnabled(&self) -> bool {
        true
    }
    fn IsHTMLElement(&self, e: &Self::Element) -> bool {
        true
    }
    fn IsInHTMLDocument(&self, e: &Self::Element) -> bool {
        true
    }
    fn AttributesWithoutUpdate(&self, e: &Self::Element) -> Vec<Rc<Self::Attribute>> {
        self.attrs[*e].iter().cloned().map(Rc::new).collect()
    }
    fn AttributeMatchesName(&self, a: &Self::Attribute, name: &QualifiedName) -> bool {
        &a.0 == name
    }
    fn AttributeMatchesNameCaseInsensitive(
        &self,
        a: &Self::Attribute,
        name: &QualifiedName,
    ) -> bool {
        a.0.LocalName().ToAsciiLower() == name.LocalName().ToAsciiLower()
            && a.0.NamespaceURI() == name.NamespaceURI()
    }
    fn AttributeValue(&self, a: &Self::Attribute) -> AtomicString {
        a.1.clone()
    }
    fn CouldHaveClass(&self, e: &Self::Element, class: &AtomicString) -> bool {
        self.classes[*e].contains(class)
    }
    fn HasClass(&self, e: &Self::Element) -> bool {
        !self.classes[*e].is_empty()
    }
    fn ClassNamesContain(&self, e: &Self::Element, class: &AtomicString) -> bool {
        self.classes[*e].contains(class)
    }
    fn HasID(&self, e: &Self::Element) -> bool {
        !self.ids[*e].IsNull()
    }
    fn IdForStyleResolution(&self, e: &Self::Element) -> AtomicString {
        self.ids[*e].clone()
    }
    fn ElementIs(&self, e: &Self::Element, class: ElementClass) -> bool {
        panic!("unexpected test dependency: ElementIs")
    }
    fn SetElementFlag(&self, e: &Self::Element, flag: ElementInvalidationFlag) {
        panic!("unexpected test dependency: SetElementFlag")
    }
    fn SetContainerFlag(&self, n: &Self::ContainerNode, flag: ContainerInvalidationFlag) {
        panic!("unexpected test dependency: SetContainerFlag")
    }
    fn FeatureEnabled(&self, e: &Self::Element, feature: SelectorRuntimeFeature) -> bool {
        panic!("unexpected test dependency: FeatureEnabled")
    }
    fn ElementLocalName(&self, e: &Self::Element) -> AtomicString {
        self.tags[*e].clone()
    }
    fn ElementNamespaceURI(&self, e: &Self::Element) -> AtomicString {
        AtomicString::from_str("http://www.w3.org/1999/xhtml")
    }
    fn ElementTagQName(&self, e: &Self::Element) -> QualifiedName {
        QualifiedName::new(
            AtomicString::default(),
            self.tags[*e].clone(),
            self.ElementNamespaceURI(e),
        )
    }
    fn FrameFocusedAndActive(&self, e: &Self::Element) -> Option<bool> {
        panic!("unexpected test dependency: FrameFocusedAndActive")
    }
    fn MenuOwnerItems(&self, e: &Self::Element) -> Vec<Rc<Self::Element>> {
        panic!("unexpected test dependency: MenuOwnerItems")
    }
    fn CorrespondingSVGElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        panic!("unexpected test dependency: CorrespondingSVGElement")
    }
    fn HasShadowRoot(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: HasShadowRoot")
    }
    fn ShadowRootIsUserAgent(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: ShadowRootIsUserAgent")
    }
    fn ContainingShadowRootIsUserAgent(&self, e: &Self::Element) -> Option<bool> {
        panic!("unexpected test dependency: ContainingShadowRootIsUserAgent")
    }
    fn UltimateOriginatingElement(&self, e: &Self::Element) -> Rc<Self::Element> {
        panic!("unexpected test dependency: UltimateOriginatingElement")
    }
    fn ShadowPseudoId(&self, e: &Self::Element) -> AtomicString {
        panic!("unexpected test dependency: ShadowPseudoId")
    }
    fn ElementPseudoIdForStyling(&self, e: &Self::Element) -> PseudoId {
        panic!("unexpected test dependency: ElementPseudoIdForStyling")
    }
    fn ElementPseudoArgument(&self, e: &Self::Element) -> AtomicString {
        panic!("unexpected test dependency: ElementPseudoArgument")
    }
    fn ElementViewTransitionClasses(&self, e: &Self::Element) -> Vec<AtomicString> {
        panic!("unexpected test dependency: ElementViewTransitionClasses")
    }
    fn PartNamesContain(&self, p: &Self::PartNames, name: &AtomicString) -> bool {
        panic!("unexpected test dependency: PartNamesContain")
    }
    fn TreeScopeRoot(&self, s: &Self::TreeScope) -> Rc<Self::ContainerNode> {
        self.nodes[0].clone()
    }
    fn ContainerAsElement(&self, n: &Self::ContainerNode) -> Option<Rc<Self::Element>> {
        (*n != 0).then(|| self.nodes[*n].clone())
    }
    fn ContainerShadowHost(&self, n: &Self::ContainerNode) -> Option<Rc<Self::Element>> {
        panic!("unexpected test dependency: ContainerShadowHost")
    }
    fn ElementAsContainer(&self, e: &Self::Element) -> Rc<Self::ContainerNode> {
        self.nodes[*e].clone()
    }
    fn DocumentElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        panic!("unexpected test dependency: DocumentElement")
    }
    fn FlatTreeParentElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        panic!("unexpected test dependency: FlatTreeParentElement")
    }
    fn ParentElementOrDocumentFragment(
        &self,
        e: &Self::Element,
    ) -> Option<Rc<Self::ContainerNode>> {
        self.parents[*e].map(|p| self.nodes[p].clone())
    }
    fn IsFinishedParsingChildren(&self, n: &Self::ContainerNode) -> bool {
        true
    }
    fn FirstChildElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        panic!("unexpected test dependency: FirstChildElement")
    }
    fn FirstChildNode(&self, e: &Self::Element) -> Option<Rc<Self::Node>> {
        panic!("unexpected test dependency: FirstChildNode")
    }
    fn NextSiblingNode(&self, n: &Self::Node) -> Option<Rc<Self::Node>> {
        panic!("unexpected test dependency: NextSiblingNode")
    }
    fn NodeIsElement(&self, n: &Self::Node) -> bool {
        panic!("unexpected test dependency: NodeIsElement")
    }
    fn TextNodeData(&self, n: &Self::Node) -> Option<AtomicString> {
        panic!("unexpected test dependency: TextNodeData")
    }
    fn NumDescendantInputs(&self, e: &Self::Element) -> usize {
        panic!("unexpected test dependency: NumDescendantInputs")
    }
    fn SlottedButton(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        panic!("unexpected test dependency: SlottedButton")
    }
    fn CssTarget(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        panic!("unexpected test dependency: CssTarget")
    }
    fn ProbeForcePseudoState(&self, e: &Self::Element, pseudo: PseudoType) -> bool {
        false
    }
    fn NthIndex(
        &self,
        e: &Self::Element,
        kind: PseudoType,
        list: Option<&CSSSelectorList>,
        checker: &SelectorChecker<Self>,
        context: &SelectorCheckingContext<'_, Self>,
    ) -> u32 {
        self.parents[*e].map_or(1, |p| {
            self.children[p]
                .iter()
                .position(|&child| child == *e)
                .unwrap() as u32
                + 1
        })
    }
    fn NavigationLocationMatches(
        &self,
        location: &dyn crate::css_selector::CSSSelectorNavigationLocation,
        e: &Self::Element,
    ) -> bool {
        panic!("unexpected test dependency: NavigationLocationMatches")
    }
    fn OwnerDataListElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        panic!("unexpected test dependency: OwnerDataListElement")
    }
    fn OwnerSelectElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        panic!("unexpected test dependency: OwnerSelectElement")
    }
    fn ActiveOption(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        panic!("unexpected test dependency: ActiveOption")
    }
    fn InterestInvoker(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        panic!("unexpected test dependency: InterestInvoker")
    }
    fn ScrollMarkerSelected(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        panic!("unexpected test dependency: ScrollMarkerSelected")
    }
    fn HasScrollMarkerGroupData(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: HasScrollMarkerGroupData")
    }
    fn CompareLayoutPreorder(&self, a: &Self::Element, b: &Self::Element) -> i32 {
        panic!("unexpected test dependency: CompareLayoutPreorder")
    }
    fn NavigationSourceElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        panic!("unexpected test dependency: NavigationSourceElement")
    }
    fn ComputeInheritedLanguage(&self, e: &Self::Element) -> AtomicString {
        panic!("unexpected test dependency: ComputeInheritedLanguage")
    }
    fn VttLanguage(&self, e: &Self::Element) -> AtomicString {
        panic!("unexpected test dependency: VttLanguage")
    }
    fn SlotAssignmentDirty(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: SlotAssignmentDirty")
    }
    fn RecalcSlotAssignments(&self, e: &Self::Element) {
        panic!("unexpected test dependency: RecalcSlotAssignments")
    }
    fn CachedDirectionIsRtl(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: CachedDirectionIsRtl")
    }
    fn PopoverInvoker(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        panic!("unexpected test dependency: PopoverInvoker")
    }
    fn OwningMenuElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        panic!("unexpected test dependency: OwningMenuElement")
    }
    fn InternalsHasState(&self, e: &Self::Element, state: &AtomicString) -> bool {
        panic!("unexpected test dependency: InternalsHasState")
    }
    fn GetPseudoElement(&self, e: &Self::Element, pseudo: PseudoId) -> Option<Rc<Self::Element>> {
        panic!("unexpected test dependency: GetPseudoElement")
    }
    fn FocusedElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        panic!("unexpected test dependency: FocusedElement")
    }
    fn FocusVisibleOption(&self, e: &Self::Element) -> Option<bool> {
        panic!("unexpected test dependency: FocusVisibleOption")
    }
    fn TransitionScope(&self, t: &Self::ViewTransition) -> Rc<Self::Element> {
        panic!("unexpected test dependency: TransitionScope")
    }
    fn TransitionMatchOnlyChild(
        &self,
        t: &Self::ViewTransition,
        id: PseudoId,
        argument: &AtomicString,
    ) -> bool {
        panic!("unexpected test dependency: TransitionMatchOnlyChild")
    }
    fn TransitionMatchActive(&self, t: &Self::ViewTransition) -> bool {
        panic!("unexpected test dependency: TransitionMatchActive")
    }
    fn TransitionMatchActiveType(&self, t: &Self::ViewTransition, idents: &[AtomicString]) -> bool {
        panic!("unexpected test dependency: TransitionMatchActiveType")
    }
    fn ScrollbarEnabled(&self, s: &Self::CustomScrollbar) -> bool {
        panic!("unexpected test dependency: ScrollbarEnabled")
    }
    fn ScrollbarHoveredPart(&self, s: &Self::CustomScrollbar) -> Self::ScrollbarPart {
        panic!("unexpected test dependency: ScrollbarHoveredPart")
    }
    fn ScrollbarPressedPart(&self, s: &Self::CustomScrollbar) -> Self::ScrollbarPart {
        panic!("unexpected test dependency: ScrollbarPressedPart")
    }
    fn ScrollbarPartBits(&self, p: &Self::ScrollbarPart) -> u32 {
        panic!("unexpected test dependency: ScrollbarPartBits")
    }
    fn ScrollbarHorizontal(&self, s: &Self::CustomScrollbar) -> bool {
        panic!("unexpected test dependency: ScrollbarHorizontal")
    }
    fn ScrollbarNativeThemeHasButtons(&self, s: &Self::CustomScrollbar) -> bool {
        panic!("unexpected test dependency: ScrollbarNativeThemeHasButtons")
    }
    fn ScrollbarCornerVisible(&self, s: &Self::CustomScrollbar) -> bool {
        panic!("unexpected test dependency: ScrollbarCornerVisible")
    }
    fn InPseudoHasChecking(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: InPseudoHasChecking")
    }
    fn ExitHasCacheScope(&self, scope: Self::HasCacheScope) {
        self.has_depth.set(self.has_depth.get() - 1)
    }
    fn EnterHasCacheScope(
        &self,
        e: &Self::Element,
        within_selector_checking: bool,
    ) -> Self::HasCacheScope {
        self.has_depth.set(self.has_depth.get() + 1);
        ()
    }
    fn CreateHasArgumentContext(
        &self,
        selector: CSSSelectorComplex<'_>,
        scope: Option<&Self::ContainerNode>,
        shadow: bool,
    ) -> Self::HasArgumentContext {
        panic!("unexpected test dependency: CreateHasArgumentContext")
    }
    fn HasInvalidShadowTraversal(&self, c: &Self::HasArgumentContext) -> bool {
        panic!("unexpected test dependency: HasInvalidShadowTraversal")
    }
    fn HasLeftmostRelation(&self, c: &Self::HasArgumentContext) -> RelationType {
        panic!("unexpected test dependency: HasLeftmostRelation")
    }
    fn HasDepthLimit(&self, c: &Self::HasArgumentContext) -> i32 {
        panic!("unexpected test dependency: HasDepthLimit")
    }
    fn HasDepthFixed(&self, c: &Self::HasArgumentContext) -> bool {
        panic!("unexpected test dependency: HasDepthFixed")
    }
    fn HasAdjacentDistanceLimit(&self, c: &Self::HasArgumentContext) -> i32 {
        panic!("unexpected test dependency: HasAdjacentDistanceLimit")
    }
    fn HasAdjacentDistanceFixed(&self, c: &Self::HasArgumentContext) -> bool {
        panic!("unexpected test dependency: HasAdjacentDistanceFixed")
    }
    fn HasSiblingCombinatorAtRightmost(&self, c: &Self::HasArgumentContext) -> bool {
        panic!("unexpected test dependency: HasSiblingCombinatorAtRightmost")
    }
    fn HasSiblingCombinatorBetweenChildOrDescendant(&self, c: &Self::HasArgumentContext) -> bool {
        panic!("unexpected test dependency: HasSiblingCombinatorBetweenChildOrDescendant")
    }
    fn HasSiblingsAffectedFlags(&self, c: &Self::HasArgumentContext) -> u8 {
        panic!("unexpected test dependency: HasSiblingsAffectedFlags")
    }
    fn SetSiblingsAffectedByHasFlags(&self, e: &Self::Element, flags: u8) {
        panic!("unexpected test dependency: SetSiblingsAffectedByHasFlags")
    }
    fn CreateHasTraversal(
        &self,
        e: &Self::Element,
        c: &Self::HasArgumentContext,
    ) -> Self::HasTraversal {
        panic!("unexpected test dependency: CreateHasTraversal")
    }
    fn HasTraversalCurrent(&self, it: &Self::HasTraversal) -> Option<Rc<Self::Element>> {
        panic!("unexpected test dependency: HasTraversalCurrent")
    }
    fn HasTraversalDepth(&self, it: &Self::HasTraversal) -> i32 {
        panic!("unexpected test dependency: HasTraversalDepth")
    }
    fn HasTraversalNext(&self, it: &mut Self::HasTraversal) {
        panic!("unexpected test dependency: HasTraversalNext")
    }
    fn CreateHasCacheContext(
        &self,
        e: &Self::Element,
        c: &Self::HasArgumentContext,
    ) -> Self::HasCacheContext {
        panic!("unexpected test dependency: CreateHasCacheContext")
    }
    fn HasCacheAllowed(&self, c: &Self::HasCacheContext) -> bool {
        panic!("unexpected test dependency: HasCacheAllowed")
    }
    fn HasCacheGetResult(&self, c: &Self::HasCacheContext, e: &Self::Element) -> u8 {
        panic!("unexpected test dependency: HasCacheGetResult")
    }
    fn HasCacheAlreadyChecked(&self, c: &Self::HasCacheContext, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: HasCacheAlreadyChecked")
    }
    fn HasCacheSetChecked(&self, c: &Self::HasCacheContext, e: &Self::Element) {
        panic!("unexpected test dependency: HasCacheSetChecked")
    }
    fn HasCacheSetMatched(&self, c: &Self::HasCacheContext, e: &Self::Element) -> u8 {
        panic!("unexpected test dependency: HasCacheSetMatched")
    }
    fn HasCacheSetAllTraversedChecked(
        &self,
        c: &Self::HasCacheContext,
        e: &Self::Element,
        depth: i32,
    ) {
        panic!("unexpected test dependency: HasCacheSetAllTraversedChecked")
    }
    fn HasCacheEnsureFastRejectFilter(
        &self,
        c: &Self::HasCacheContext,
        e: &Self::Element,
    ) -> (Rc<Self::HasFastRejectFilter>, bool) {
        panic!("unexpected test dependency: HasCacheEnsureFastRejectFilter")
    }
    fn HasBloomAllocated(&self, f: &Self::HasFastRejectFilter) -> bool {
        panic!("unexpected test dependency: HasBloomAllocated")
    }
    fn HasAllocateBloom(&self, f: &Self::HasFastRejectFilter) {
        panic!("unexpected test dependency: HasAllocateBloom")
    }
    fn HasAddElementIdentifierHashes(&self, f: &Self::HasFastRejectFilter, e: &Self::Element) {
        panic!("unexpected test dependency: HasAddElementIdentifierHashes")
    }
    fn HasFastReject(&self, f: &Self::HasFastRejectFilter, c: &Self::HasArgumentContext) -> bool {
        panic!("unexpected test dependency: HasFastReject")
    }
    fn StyleScopeParent(&self, s: &Self::StyleScope) -> Option<Rc<Self::StyleScope>> {
        panic!("unexpected test dependency: StyleScopeParent")
    }
    fn StyleScopeFrom(&self, s: &Self::StyleScope) -> Option<Rc<CSSSelectorList>> {
        panic!("unexpected test dependency: StyleScopeFrom")
    }
    fn StyleScopeTo(&self, s: &Self::StyleScope) -> Option<Rc<CSSSelectorList>> {
        panic!("unexpected test dependency: StyleScopeTo")
    }
    fn ElementTriggersScope(&self, e: &Self::Element, s: &Self::StyleScope) -> bool {
        panic!("unexpected test dependency: ElementTriggersScope")
    }
    fn StyleScopeFrameElement(&self, f: &Self::StyleScopeFrame) -> Rc<Self::Element> {
        panic!("unexpected test dependency: StyleScopeFrameElement")
    }
    fn ParentStyleScopeFrame(
        &self,
        f: &Self::StyleScopeFrame,
        e: &Self::Element,
    ) -> Option<Rc<Self::StyleScopeFrame>> {
        panic!("unexpected test dependency: ParentStyleScopeFrame")
    }
    fn InsertScopeActivationCacheEntry(
        &self,
        f: &Self::StyleScopeFrame,
        s: &Self::StyleScope,
    ) -> Option<Rc<Self::StyleScopeActivations>> {
        panic!("unexpected test dependency: InsertScopeActivationCacheEntry")
    }
    fn SetScopeActivationCacheEntry(
        &self,
        f: &Self::StyleScopeFrame,
        s: &Self::StyleScope,
        a: Rc<Self::StyleScopeActivations>,
    ) {
        panic!("unexpected test dependency: SetScopeActivationCacheEntry")
    }
    fn CreateActivation(
        &self,
        root: Option<Rc<Self::ContainerNode>>,
        proximity: u32,
    ) -> Rc<Self::StyleScopeActivation> {
        panic!("unexpected test dependency: CreateActivation")
    }
    fn CreateActivations(
        &self,
        vector: Vec<Rc<Self::StyleScopeActivation>>,
        flags: MatchFlags,
    ) -> Rc<Self::StyleScopeActivations> {
        panic!("unexpected test dependency: CreateActivations")
    }
    fn SpatialNavigationFocused(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: SpatialNavigationFocused")
    }
    fn HasDatalist(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: HasDatalist")
    }
    fn IsSubmenuOpen(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: IsSubmenuOpen")
    }
    fn UsesMenuList(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: UsesMenuList")
    }
    fn IsMultiSelectFocused(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: IsMultiSelectFocused")
    }
    fn IsResourceTarget(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: IsResourceTarget")
    }
    fn CachedImageIsAnimated(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: CachedImageIsAnimated")
    }
    fn IsPlaceholderVisible(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: IsPlaceholderVisible")
    }
    fn MatchesToolFormActive(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: MatchesToolFormActive")
    }
    fn IsUnboundedElementActive(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: IsUnboundedElementActive")
    }
    fn MatchesToolSubmitActive(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: MatchesToolSubmitActive")
    }
    fn IsDragged(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: IsDragged")
    }
    fn HasFocusWithin(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: HasFocusWithin")
    }
    fn IsFiltered(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: IsFiltered")
    }
    fn HasInterest(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: HasInterest")
    }
    fn HasFlattenedAssignedNodes(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: HasFlattenedAssignedNodes")
    }
    fn IsHovered(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: IsHovered")
    }
    fn IsActive(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: IsActive")
    }
    fn MatchesEnabled(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: MatchesEnabled")
    }
    fn IsMediaDocument(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: IsMediaDocument")
    }
    fn MatchesDefault(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: MatchesDefault")
    }
    fn MatchesDisabled(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: MatchesDisabled")
    }
    fn MatchesReadOnly(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: MatchesReadOnly")
    }
    fn MatchesReadWrite(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: MatchesReadWrite")
    }
    fn IsOptionalFormControl(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: IsOptionalFormControl")
    }
    fn IsRequiredFormControl(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: IsRequiredFormControl")
    }
    fn MatchesUserInvalid(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: MatchesUserInvalid")
    }
    fn MatchesUserValid(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: MatchesUserValid")
    }
    fn MatchesValidity(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: MatchesValidity")
    }
    fn IsValid(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: IsValid")
    }
    fn ShouldAppearChecked(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: ShouldAppearChecked")
    }
    fn ShouldAppearIndeterminate(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: ShouldAppearIndeterminate")
    }
    fn OptionSelected(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: OptionSelected")
    }
    fn ScrollMarkerIsSelected(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: ScrollMarkerIsSelected")
    }
    fn IsTextField(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: IsTextField")
    }
    fn IsDialogInTopLayer(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: IsDialogInTopLayer")
    }
    fn IsPopoverInTopLayer(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: IsPopoverInTopLayer")
    }
    fn IsPopover(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: IsPopover")
    }
    fn PopoverOpen(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: PopoverOpen")
    }
    fn MatchesOverscrollOpen(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: MatchesOverscrollOpen")
    }
    fn HasOpenAttribute(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: HasOpenAttribute")
    }
    fn PopupIsVisible(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: PopupIsVisible")
    }
    fn IsPickerVisible(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: IsPickerVisible")
    }
    fn FullscreenFlag(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: FullscreenFlag")
    }
    fn ContainsFullScreenElement(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: ContainsFullScreenElement")
    }
    fn Granted(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: Granted")
    }
    fn IsPictureInPicture(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: IsPictureInPicture")
    }
    fn MediaPaused(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: MediaPaused")
    }
    fn MediaSeeking(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: MediaSeeking")
    }
    fn MediaBuffering(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: MediaBuffering")
    }
    fn MediaStalled(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: MediaStalled")
    }
    fn MediaMuted(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: MediaMuted")
    }
    fn VideoPersistent(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: VideoPersistent")
    }
    fn ContainsPersistentVideo(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: ContainsPersistentVideo")
    }
    fn IsXrOverlay(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: IsXrOverlay")
    }
    fn IsFullscreenElement(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: IsFullscreenElement")
    }
    fn IsInRange(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: IsInRange")
    }
    fn IsOutOfRange(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: IsOutOfRange")
    }
    fn VttIsPastNode(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: VttIsPastNode")
    }
    fn IsDefined(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: IsDefined")
    }
    fn GetComputedStyle(&self, e: &Self::Element) -> Option<Persistent<ComputedStyle>> {
        panic!("unexpected test dependency: GetComputedStyle")
    }
    fn HasEffectiveAppearance(&self, style: &ComputedStyle) -> bool {
        panic!("unexpected test dependency: HasEffectiveAppearance")
    }
    fn PageIsActive(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: PageIsActive")
    }
    fn DidAttachInternals(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: DidAttachInternals")
    }
    fn DialogIsModal(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: DialogIsModal")
    }
    fn IsAutofilled(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: IsAutofilled")
    }
    fn IsPreviewed(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: IsPreviewed")
    }
    fn AutofillStateIsPreviewed(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: AutofillStateIsPreviewed")
    }
    fn IsFocused(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: IsFocused")
    }
    fn AccessibilityAlwaysShowFocus(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: AccessibilityAlwaysShowFocus")
    }
    fn MayTriggerVirtualKeyboard(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: MayTriggerVirtualKeyboard")
    }
    fn LastFocusTypeIsMouse(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: LastFocusTypeIsMouse")
    }
    fn HadKeyboardEvent(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: HadKeyboardEvent")
    }
    fn AffectedByMultipleHas(&self, e: &Self::Element) -> bool {
        panic!("unexpected test dependency: AffectedByMultipleHas")
    }
}

fn class(name: &str, relation: RelationType) -> CSSSelector {
    let mut s = CSSSelector::default();
    s.SetMatch(MatchType::kClass);
    s.SetValue(AtomicString::from_str(name), false);
    s.SetRelation(relation);
    s
}
fn id(name: &str, relation: RelationType) -> CSSSelector {
    let mut s = CSSSelector::default();
    s.SetMatch(MatchType::kId);
    s.SetValue(AtomicString::from_str(name), false);
    s.SetRelation(relation);
    s
}
fn pseudo(p: PseudoType, relation: RelationType) -> CSSSelector {
    let mut s = CSSSelector::default();
    s.SetMatch(MatchType::kPseudoClass);
    s.SetPseudoType(p);
    s.SetRelation(relation);
    s
}
fn query(b: &Backend, mut selectors: Vec<CSSSelector>) -> SelectorQuery {
    selectors.last_mut().unwrap().SetLastInComplexSelector(true);
    SelectorQuery::new(CSSSelectorList::AdoptSelectorVector(selectors), b)
}
fn result(q: &SelectorQuery, b: &Rc<Backend>, root: usize, expected: &[usize]) {
    let actual = q.QueryAll(b.clone(), b.nodes[root].clone());
    assert_eq!(actual.iter().map(|e| **e).collect::<Vec<_>>(), expected);
    assert_eq!(
        q.QueryFirst(b.clone(), b.nodes[root].clone())
            .as_deref()
            .copied(),
        expected.first().copied()
    );
    let mut slow = vec![];
    q.ExecuteSlow::<Backend, false>(b, &b.nodes[root], &mut slow);
    assert_eq!(actual, slow, "DFA must agree with genuine SelectorChecker");
    assert_eq!((b.has_depth.get(), b.nth_depth.get()), (0, 0));
}
#[test]
fn excludes_root_and_keeps_tree_order_with_bloom_pruning() {
    let b = Backend::new(
        &[None, Some(0), Some(1), Some(2), Some(1), Some(4), Some(1)],
        &["", "target", "miss", "target", "miss", "", "target"],
        &[""; 7],
    );
    let q = query(&b, vec![class("target", RelationType::kSubSelector)]);
    result(&q, &b, 1, &[3, 6]);
    let all = q.QueryAll(b.clone(), b.nodes[1].clone());
    assert_eq!(all.len(), 2);
    assert!(SelectorQuery::LastQueryStats().skipped_subtree > 0);
    assert!(q.Matches(b.clone(), b.nodes[1].clone()));
    assert_eq!(
        q.Closest(b.clone(), b.nodes[5].clone()).as_deref(),
        Some(&1)
    );
}
#[test]
fn sibling_states_reset_for_children_and_strict_combinators_recheck() {
    let b = Backend::new(
        &[
            None,
            Some(0),
            Some(1),
            Some(2),
            Some(3),
            Some(1),
            Some(5),
            Some(1),
            Some(7),
            Some(1),
            Some(9),
        ],
        &["", "", "a", "b", "c", "gap", "c", "b", "c", "b", "c"],
        &[""; 11],
    );
    // .a ~ .b .c matches descendants of later siblings, not children of .a.
    result(
        &query(
            &b,
            vec![
                class("c", RelationType::kDescendant),
                class("b", RelationType::kIndirectAdjacent),
                class("a", RelationType::kSubSelector),
            ],
        ),
        &b,
        1,
        &[8, 10],
    );
    // .a + .b > .c cannot jump across the gap sibling.
    result(
        &query(
            &b,
            vec![
                class("c", RelationType::kChild),
                class("b", RelationType::kDirectAdjacent),
                class("a", RelationType::kSubSelector),
            ],
        ),
        &b,
        1,
        &[],
    );
}
#[test]
fn ancestors_before_root_and_closest_scope_use_real_checker() {
    let b = Backend::new(
        &[None, Some(0), Some(1), Some(2), Some(3), Some(2)],
        &["", "a", "b", "c", "c", "c"],
        &[""; 6],
    );
    result(
        &query(
            &b,
            vec![
                class("c", RelationType::kDescendant),
                class("a", RelationType::kSubSelector),
            ],
        ),
        &b,
        2,
        &[3, 4, 5],
    );
    result(
        &query(
            &b,
            vec![
                class("c", RelationType::kChild),
                class("b", RelationType::kSubSelector),
            ],
        ),
        &b,
        2,
        &[3, 5],
    );
    let q = query(
        &b,
        vec![pseudo(PseudoType::kPseudoScope, RelationType::kSubSelector)],
    );
    assert!(q.Matches(b.clone(), b.nodes[3].clone()));
    assert_eq!(
        q.Closest(b.clone(), b.nodes[3].clone()).as_deref(),
        Some(&3)
    );
    result(&q, &b, 2, &[]);
}
#[test]
fn nth_child_counts_siblings_skipped_by_bloom() {
    let b = Backend::new(
        &[None, Some(0), Some(1), Some(1), Some(1), Some(1), Some(1)],
        &["", "", "miss", "x", "miss", "x", "x"],
        &[""; 7],
    );
    let mut nth = pseudo(PseudoType::kPseudoNthChild, RelationType::kSubSelector);
    nth.SetNth(2, 0, None);
    result(
        &query(&b, vec![nth, class("x", RelationType::kSubSelector)]),
        &b,
        1,
        &[3, 5],
    );
    let mut nth = pseudo(PseudoType::kPseudoNthChild, RelationType::kSubSelector);
    nth.SetNth(-1, 3, None);
    result(
        &query(&b, vec![nth, class("x", RelationType::kSubSelector)]),
        &b,
        1,
        &[3],
    );
}
#[test]
fn id_anchor_limits_traversal_and_duplicate_ancestor_ids_avoid_duplicates() {
    let b = Backend::new(
        &[None, Some(0), Some(1), Some(2), Some(3), Some(1), Some(5)],
        &["", "", "", "target", "target", "", "target"],
        &["", "", "anchor", "anchor", "", "", ""],
    );
    let q = query(
        &b,
        vec![
            class("target", RelationType::kDescendant),
            id("anchor", RelationType::kSubSelector),
        ],
    );
    result(&q, &b, 1, &[3, 4]);
    let q = query(&b, vec![id("anchor", RelationType::kSubSelector)]);
    result(&q, &b, 1, &[2, 3]);
    assert!(q.QueryFirst(b.clone(), b.nodes[5].clone()).is_none());
    let q = query(&b, vec![id("missing", RelationType::kSubSelector)]);
    result(&q, &b, 1, &[]);
}
#[test]
fn comma_list_scan_filters_pseudo_elements_and_deduplicates() {
    let b = Backend::new(
        &[None, Some(0), Some(1), Some(1), Some(1)],
        &["", "", "a b", "b", ""],
        &[""; 5],
    );
    let mut a = class("a", RelationType::kSubSelector);
    a.SetLastInComplexSelector(true);
    let mut b_selector = class("b", RelationType::kSubSelector);
    b_selector.SetLastInComplexSelector(true);
    let mut before = CSSSelector::default();
    before.SetMatch(MatchType::kPseudoElement);
    before.SetPseudoType(PseudoType::kPseudoBefore);
    before.SetLastInComplexSelector(true);
    let q = SelectorQuery::new(
        CSSSelectorList::AdoptSelectorVector(vec![before.clone(), a, b_selector]),
        &*b,
    );
    result(&q, &b, 1, &[2, 3]);
    let q = SelectorQuery::new(CSSSelectorList::AdoptSelectorVector(vec![before]), &*b);
    result(&q, &b, 1, &[]);
    assert!(!q.Matches(b.clone(), b.nodes[2].clone()));
}
#[test]
fn compound_filter_intersects_logical_alternatives_and_excludes_lazy_attributes() {
    let b = Backend::new(&[None], &[""], &[""]);
    let mut a = class("a", RelationType::kSubSelector);
    a.SetLastInComplexSelector(true);
    let mut ab = class("a", RelationType::kSubSelector);
    ab.SetLastInComplexSelector(true);
    let alternatives = CSSSelectorList::AdoptSelectorVector(vec![a, ab]);
    let mut logical = pseudo(PseudoType::kPseudoIs, RelationType::kSubSelector);
    logical.SetSelectorList(Some(alternatives));
    let q = query(&b, vec![logical]);
    assert_eq!(
        q.compounds_[0].selector_filter,
        crate::selector_filter::FilterForString(&AtomicString::from_str("a"))
    );
    assert!(q.need_full_check_);
}

#[path = "selector_query_cache_test.rs"]
mod cache;

struct AttributeContext;
impl crate::css_selector::CSSSelectorAttributeContext for AttributeContext {
    fn IsCaseSensitiveAttribute(&self, attribute: &QualifiedName) -> bool {
        true
    }
}
fn attribute(name: &str, value: &str, sensitivity: AttributeMatchType) -> CSSSelector {
    CSSSelector::FromAttribute(
        MatchType::kAttributeExact,
        QualifiedName::new(
            AtomicString::default(),
            AtomicString::from_str(name),
            AtomicString::default(),
        ),
        sensitivity,
        Some(AtomicString::from_str(value)),
        &AttributeContext,
    )
}
#[test]
fn exact_attributes_keep_case_flags_and_lazy_attribute_policy() {
    let mut b = Backend::new(&[None, Some(0), Some(1), Some(1)], &[""; 4], &[""; 4]);
    let backend = Rc::get_mut(&mut b).unwrap();
    let name = QualifiedName::new(
        AtomicString::default(),
        AtomicString::from_str("data-v"),
        AtomicString::default(),
    );
    backend.attrs[2].push((name.clone(), AtomicString::from_str("YES")));
    backend.attrs[3].push((name, AtomicString::from_str("yes")));
    let q = query(
        &b,
        vec![attribute(
            "data-v",
            "yes",
            AttributeMatchType::kCaseInsensitive,
        )],
    );
    result(&q, &b, 1, &[2, 3]);
    let q = query(
        &b,
        vec![attribute(
            "data-v",
            "yes",
            AttributeMatchType::kCaseSensitiveAlways,
        )],
    );
    result(&q, &b, 1, &[3]);
    let q = query(
        &b,
        vec![attribute(
            "style",
            "x",
            AttributeMatchType::kCaseSensitiveAlways,
        )],
    );
    assert_eq!(q.compounds_[0].selector_filter, 0);
    q.FillMissingData(&*b, &1);
    assert!(q.compounds_[0].needs_synchronize_attribute.get());
}
