/*
 * Copyright (C) 1999 Lars Knoll (knoll@kde.org)
 * Copyright (C) 2004-2013 Apple Inc. and Chromium contributors.
 * GNU Library General Public License version 2 or later; see COPYING.LIB.
 */
// cpp: third_party/blink/renderer/core/css/selector_checker.h
// cpp: third_party/blink/renderer/core/css/selector_checker.cc
// Source: /Users/zhenghuaiyu/chromium/src/third_party/blink/renderer/core/css/
// Source commit: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
// Source ledger (physical / effective / mapped / omitted / pending):
//   selector_checker.h:  602 / 218 / 167 / 51 / 0.
//   selector_checker.cc: 4079 / 2474 / 2360 / 114 / 0.
// Effective excludes copyright/comments/blank/preprocessor/include/namespace
// lines and lines consisting only of braces/parentheses/semicolons. Every
// remaining production declaration/statement in these two files is mapped.
// Header omissions: 53-57,60,62,108-109,134,136,273,275,356,358,382,398,
// 436-441,520,523,555-562,574-597 (only effective lines within these ranges).
// These are C++ forward/access/allocation/deleted/friend/debug boilerplate;
// MatchForPseudoContent/Shadow have no definitions in the source; the
// EasySelectorChecker definitions belong to selector_checker-inl.h, outside
// these two source files, so their isolated declarations are omitted here.
// Implementation omissions: C++ access/allocation boilerplate at 269,271,306,
// 1782,1784,1829; all DCHECK statements with their continuation lines; debug
// sections 519-522,876-884,1434-1441,1523-1529; UseCounter statements at
// 668-672,2323-2324,3131-3135 and the metrics-only conditional at 3472-3475.
// Source-local helper bodies 118-376 and all production implementation
// 377-4079 are mapped. DOM, CheckPseudoHasArgumentContext/TraversalIterator,
// cache/filter and StyleScope owner objects remain required typed collaborators.
// The checker drives all matching, traversal, early-break and cache-propagation
// branches itself. No pending seam, default result or production stub remains.
#![allow(non_snake_case, non_camel_case_types)]

use crate::css_selector::{
    AttributeMatchType, CSSSelectorComplex, MatchType, PseudoType, QualifiedName, RelationType,
};
use crate::css_selector_list::CSSSelectorList;
pub use crate::element_rule_collector::SelectorCheckerMode as Mode;
use crate::resolver::match_flags::{MatchFlag, MatchFlags};
use foundation::{AtomicString, Persistent};
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::computed_style_constants::PseudoId;
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Impact {
    kSubject = 1,
    kNonSubject = 2,
    kBoth = 3,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchStatus {
    kSelectorMatches,
    kSelectorFailsLocally,
    kSelectorFailsAllSiblings,
    kSelectorFailsCompletely,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeaturelessMatch {
    kFeaturelessMatches,
    kFeaturelessFails,
    kFeaturelessUnknown,
}

// Typed DOM class/runtime/invalidation selectors; no stored element-state model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElementClass {
    Option,
    Input,
    Select,
    MenuOwner,
    SVG,
    Image,
    TextControl,
    Form,
    HTMLElement,
    FormControl,
    FormControlWithState,
    MenuItem,
    ScrollMarker,
    Anchor,
    VTT,
    Dialog,
    Details,
    MenuList,
    MenuBar,
    Capability,
    Media,
    Video,
    Slot,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElementInvalidationFlag {
    AffectedBySubjectHas,
    AffectedByNonSubjectHas,
    AffectedByPseudoInHas,
    AffectedByLogicalCombinationsInHas,
    AncestorsOrAncestorSiblingsAffectedByHas,
    AffectedByMultipleHas,
    StyleAffectedByEmpty,
    AffectedByFirstChildRules,
    AffectedByLastChildRules,
    ChildrenOrSiblingsAffectedByDrag,
    AncestorsOrSiblingsAffectedByFocusInHas,
    ChildrenOrSiblingsAffectedByFocus,
    AncestorsOrSiblingsAffectedByFocusVisibleInHas,
    ChildrenOrSiblingsAffectedByFocusVisible,
    ChildrenOrSiblingsAffectedByFocusWithin,
    AncestorsOrSiblingsAffectedByHoverInHas,
    ChildrenOrSiblingsAffectedByHover,
    AncestorsOrSiblingsAffectedByActiveInHas,
    ChildrenOrSiblingsAffectedByActive,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContainerInvalidationFlag {
    ChildrenAffectedByFirstChildRules,
    ChildrenAffectedByLastChildRules,
    ChildrenAffectedByForwardPositionalRules,
    ChildrenAffectedByBackwardPositionalRules,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectorRuntimeFeature {
    CSSImageAnimation,
    CustomizableCombobox,
    FilterableSelect,
    CSSLangExtendedRanges,
    GeolocationElement,
    UserMediaElement,
    InstallElement,
}

/// DOM/StyleScope/StyleRequest operations only; all selector traversal,
/// branching and result propagation for this batch remain in SelectorChecker.
pub trait SelectorCheckerBackend: Sized {
    type Element;
    type ContainerNode;
    type TreeScope;
    type StyleScope;
    type StyleScopeFrame;
    type StyleScopeActivations;
    type StyleScopeActivation;
    type ElementResolveContext;
    type PartNames;
    type CustomScrollbar;
    type ScrollbarPart;
    type StyleRequest;
    type Attribute;
    type ViewTransition;
    fn ResolveUltimateOriginatingElementOrSelf(
        &self,
        c: &Self::ElementResolveContext,
    ) -> Rc<Self::Element>;
    fn ResolvePseudoElement(&self, c: &Self::ElementResolveContext) -> Option<Rc<Self::Element>>;
    fn ResolvePseudoElementAncestors(
        &self,
        c: &Self::ElementResolveContext,
    ) -> Vec<Rc<Self::Element>>;
    fn RequestScrollbar(&self, r: &Self::StyleRequest) -> Option<Rc<Self::CustomScrollbar>>;
    fn RequestPseudoArgument(&self, r: &Self::StyleRequest) -> AtomicString;
    fn RequestPseudoIdentList(&self, r: &Self::StyleRequest) -> Vec<AtomicString>;
    fn RequestScrollbarPart(&self, r: &Self::StyleRequest) -> Self::ScrollbarPart;
    fn NoScrollbarPart(&self) -> Self::ScrollbarPart;
    fn TreeScopeShadowHost(&self, s: &Self::TreeScope) -> Option<Rc<Self::Element>>;
    fn TreeScopeIsShadowRoot(&self, s: &Self::TreeScope) -> bool;
    fn ParentTreeScope(&self, s: &Self::TreeScope) -> Option<Rc<Self::TreeScope>>;
    fn ElementTreeScope(&self, e: &Self::Element) -> Rc<Self::TreeScope>;
    fn ParentElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>>;
    fn ParentOrShadowHostElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>>;
    fn PreviousSibling(&self, e: &Self::Element) -> Option<Rc<Self::Element>>;
    fn NextSibling(&self, e: &Self::Element) -> Option<Rc<Self::Element>>;
    fn PreviousSiblingWithTagName(
        &self,
        e: &Self::Element,
        name: &QualifiedName,
    ) -> Option<Rc<Self::Element>>;
    fn NextSiblingWithTagName(
        &self,
        e: &Self::Element,
        name: &QualifiedName,
    ) -> Option<Rc<Self::Element>>;
    fn AssignedSlot(&self, e: &Self::Element) -> Option<Rc<Self::Element>>;
    fn IsSlotSupportingAssignment(&self, e: &Self::Element) -> bool;
    fn InQuirksMode(&self, e: &Self::Element) -> bool;
    fn IsLink(&self, e: &Self::Element) -> bool;
    fn IsPseudoElement(&self, e: &Self::Element) -> bool;
    fn IsScrollButtonPseudoElement(&self, e: &Self::Element) -> bool;
    fn ParentComputedStyle(&self, e: &Self::Element) -> Option<Persistent<ComputedStyle>>;
    fn ScrollButtonPseudoIdFromArgument(
        &self,
        argument: &AtomicString,
        style: &ComputedStyle,
    ) -> PseudoId;
    fn ElementPseudoId(&self, e: &Self::Element) -> PseudoId;
    fn TransitionForElement(&self, e: &Self::Element) -> Option<Rc<Self::ViewTransition>>;
    fn CSSLogicalCombinationPseudoEnabled(&self) -> bool;
    fn ParentElementOrShadowRoot(&self, e: &Self::Element) -> Option<Rc<Self::ContainerNode>>;
    fn SetChildrenAffectedByDirectAdjacentRules(&self, node: &Self::ContainerNode);
    fn SetChildrenAffectedByIndirectAdjacentRules(&self, node: &Self::ContainerNode);
    fn OwnerShadowHost(&self, e: &Self::Element) -> Option<Rc<Self::Element>>;
    fn ActivationMatchFlags(&self, activations: &Self::StyleScopeActivations) -> MatchFlags;
    fn ActivationVector(
        &self,
        activations: &Self::StyleScopeActivations,
    ) -> Vec<Rc<Self::StyleScopeActivation>>;
    fn ActivationRoot(
        &self,
        activation: &Self::StyleScopeActivation,
    ) -> Option<Rc<Self::ContainerNode>>;
    fn ActivationProximity(&self, activation: &Self::StyleScopeActivation) -> u32;
    fn SynchronizeAttribute(&self, e: &Self::Element, name: &AtomicString);
    fn CouldHaveAttribute(&self, e: &Self::Element, name: &QualifiedName) -> bool;
    fn CSSAttributeValueCaseSensitiveNonHTMLEnabled(&self) -> bool;
    fn IsHTMLElement(&self, e: &Self::Element) -> bool;
    fn IsInHTMLDocument(&self, e: &Self::Element) -> bool;
    fn AttributesWithoutUpdate(&self, e: &Self::Element) -> Vec<Rc<Self::Attribute>>;
    fn AttributeMatchesName(&self, a: &Self::Attribute, name: &QualifiedName) -> bool;
    fn AttributeMatchesNameCaseInsensitive(
        &self,
        a: &Self::Attribute,
        name: &QualifiedName,
    ) -> bool;
    fn AttributeValue(&self, a: &Self::Attribute) -> AtomicString;
    fn CouldHaveClass(&self, e: &Self::Element, class: &AtomicString) -> bool;
    fn HasClass(&self, e: &Self::Element) -> bool;
    fn ClassNamesContain(&self, e: &Self::Element, class: &AtomicString) -> bool;
    fn HasID(&self, e: &Self::Element) -> bool;
    fn IdForStyleResolution(&self, e: &Self::Element) -> AtomicString;

    type Node;
    type HasArgumentContext;
    type HasTraversal;
    type HasCacheScope;
    type HasCacheContext;
    type HasFastRejectFilter;
    fn ElementIs(&self, e: &Self::Element, class: ElementClass) -> bool;
    fn SetElementFlag(&self, e: &Self::Element, flag: ElementInvalidationFlag);
    fn SetContainerFlag(&self, n: &Self::ContainerNode, flag: ContainerInvalidationFlag);
    fn FeatureEnabled(&self, e: &Self::Element, feature: SelectorRuntimeFeature) -> bool;
    fn ElementLocalName(&self, e: &Self::Element) -> AtomicString;
    fn ElementNamespaceURI(&self, e: &Self::Element) -> AtomicString;
    fn ElementTagQName(&self, e: &Self::Element) -> QualifiedName;
    fn FrameFocusedAndActive(&self, e: &Self::Element) -> Option<bool>;
    fn MenuOwnerItems(&self, e: &Self::Element) -> Vec<Rc<Self::Element>>;
    fn CorrespondingSVGElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>>;
    fn HasShadowRoot(&self, e: &Self::Element) -> bool;
    fn ShadowRootIsUserAgent(&self, e: &Self::Element) -> bool;
    fn ContainingShadowRootIsUserAgent(&self, e: &Self::Element) -> Option<bool>;
    fn UltimateOriginatingElement(&self, e: &Self::Element) -> Rc<Self::Element>;
    fn ShadowPseudoId(&self, e: &Self::Element) -> AtomicString;
    fn ElementPseudoIdForStyling(&self, e: &Self::Element) -> PseudoId;
    fn ElementPseudoArgument(&self, e: &Self::Element) -> AtomicString;
    fn ElementViewTransitionClasses(&self, e: &Self::Element) -> Vec<AtomicString>;
    fn PartNamesContain(&self, p: &Self::PartNames, name: &AtomicString) -> bool;
    fn TreeScopeRoot(&self, s: &Self::TreeScope) -> Rc<Self::ContainerNode>;
    fn ContainerAsElement(&self, n: &Self::ContainerNode) -> Option<Rc<Self::Element>>;
    fn ContainerShadowHost(&self, n: &Self::ContainerNode) -> Option<Rc<Self::Element>>;
    fn ElementAsContainer(&self, e: &Self::Element) -> Rc<Self::ContainerNode>;
    fn DocumentElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>>;
    fn FlatTreeParentElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>>;
    fn ParentElementOrDocumentFragment(&self, e: &Self::Element)
        -> Option<Rc<Self::ContainerNode>>;
    fn IsFinishedParsingChildren(&self, n: &Self::ContainerNode) -> bool;
    fn FirstChildElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>>;
    fn FirstChildNode(&self, e: &Self::Element) -> Option<Rc<Self::Node>>;
    fn NextSiblingNode(&self, n: &Self::Node) -> Option<Rc<Self::Node>>;
    fn NodeIsElement(&self, n: &Self::Node) -> bool;
    fn TextNodeData(&self, n: &Self::Node) -> Option<AtomicString>;
    fn NumDescendantInputs(&self, e: &Self::Element) -> usize;
    fn SlottedButton(&self, e: &Self::Element) -> Option<Rc<Self::Element>>;
    fn CssTarget(&self, e: &Self::Element) -> Option<Rc<Self::Element>>;
    fn ProbeForcePseudoState(&self, e: &Self::Element, pseudo: PseudoType) -> bool;
    fn NthIndex(
        &self,
        e: &Self::Element,
        kind: PseudoType,
        list: Option<&CSSSelectorList>,
        checker: &SelectorChecker<Self>,
        context: &SelectorCheckingContext<'_, Self>,
    ) -> u32;
    fn NavigationLocationMatches(
        &self,
        location: &dyn crate::css_selector::CSSSelectorNavigationLocation,
        e: &Self::Element,
    ) -> bool;
    fn OwnerDataListElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>>;
    fn OwnerSelectElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>>;
    fn ActiveOption(&self, e: &Self::Element) -> Option<Rc<Self::Element>>;
    fn InterestInvoker(&self, e: &Self::Element) -> Option<Rc<Self::Element>>;
    fn ScrollMarkerSelected(&self, e: &Self::Element) -> Option<Rc<Self::Element>>;
    fn HasScrollMarkerGroupData(&self, e: &Self::Element) -> bool;
    fn CompareLayoutPreorder(&self, a: &Self::Element, b: &Self::Element) -> i32;
    fn NavigationSourceElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>>;
    fn ComputeInheritedLanguage(&self, e: &Self::Element) -> AtomicString;
    fn VttLanguage(&self, e: &Self::Element) -> AtomicString;
    fn SlotAssignmentDirty(&self, e: &Self::Element) -> bool;
    fn RecalcSlotAssignments(&self, e: &Self::Element);
    fn CachedDirectionIsRtl(&self, e: &Self::Element) -> bool;
    fn PopoverInvoker(&self, e: &Self::Element) -> Option<Rc<Self::Element>>;
    fn OwningMenuElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>>;
    fn InternalsHasState(&self, e: &Self::Element, state: &AtomicString) -> bool;
    fn GetPseudoElement(&self, e: &Self::Element, pseudo: PseudoId) -> Option<Rc<Self::Element>>;
    fn FocusedElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>>;
    fn FocusVisibleOption(&self, e: &Self::Element) -> Option<bool>;
    fn TransitionScope(&self, t: &Self::ViewTransition) -> Rc<Self::Element>;
    fn TransitionMatchOnlyChild(
        &self,
        t: &Self::ViewTransition,
        id: PseudoId,
        argument: &AtomicString,
    ) -> bool;
    fn TransitionMatchActive(&self, t: &Self::ViewTransition) -> bool;
    fn TransitionMatchActiveType(&self, t: &Self::ViewTransition, idents: &[AtomicString]) -> bool;
    fn ScrollbarEnabled(&self, s: &Self::CustomScrollbar) -> bool;
    fn ScrollbarHoveredPart(&self, s: &Self::CustomScrollbar) -> Self::ScrollbarPart;
    fn ScrollbarPressedPart(&self, s: &Self::CustomScrollbar) -> Self::ScrollbarPart;
    fn ScrollbarPartBits(&self, p: &Self::ScrollbarPart) -> u32;
    fn ScrollbarHorizontal(&self, s: &Self::CustomScrollbar) -> bool;
    fn ScrollbarNativeThemeHasButtons(&self, s: &Self::CustomScrollbar) -> bool;
    fn ScrollbarCornerVisible(&self, s: &Self::CustomScrollbar) -> bool;
    fn InPseudoHasChecking(&self, e: &Self::Element) -> bool;
    fn ExitHasCacheScope(&self, scope: Self::HasCacheScope);
    fn EnterHasCacheScope(
        &self,
        e: &Self::Element,
        within_selector_checking: bool,
    ) -> Self::HasCacheScope;
    fn CreateHasArgumentContext(
        &self,
        selector: CSSSelectorComplex<'_>,
        scope: Option<&Self::ContainerNode>,
        shadow: bool,
    ) -> Self::HasArgumentContext;
    fn HasInvalidShadowTraversal(&self, c: &Self::HasArgumentContext) -> bool;
    fn HasLeftmostRelation(&self, c: &Self::HasArgumentContext) -> RelationType;
    fn HasDepthLimit(&self, c: &Self::HasArgumentContext) -> i32;
    fn HasDepthFixed(&self, c: &Self::HasArgumentContext) -> bool;
    fn HasAdjacentDistanceLimit(&self, c: &Self::HasArgumentContext) -> i32;
    fn HasAdjacentDistanceFixed(&self, c: &Self::HasArgumentContext) -> bool;
    fn HasSiblingCombinatorAtRightmost(&self, c: &Self::HasArgumentContext) -> bool;
    fn HasSiblingCombinatorBetweenChildOrDescendant(&self, c: &Self::HasArgumentContext) -> bool;
    fn HasSiblingsAffectedFlags(&self, c: &Self::HasArgumentContext) -> u8;
    fn SetSiblingsAffectedByHasFlags(&self, e: &Self::Element, flags: u8);
    fn CreateHasTraversal(
        &self,
        e: &Self::Element,
        c: &Self::HasArgumentContext,
    ) -> Self::HasTraversal;
    fn HasTraversalCurrent(&self, it: &Self::HasTraversal) -> Option<Rc<Self::Element>>;
    fn HasTraversalDepth(&self, it: &Self::HasTraversal) -> i32;
    fn HasTraversalNext(&self, it: &mut Self::HasTraversal);
    fn CreateHasCacheContext(
        &self,
        e: &Self::Element,
        c: &Self::HasArgumentContext,
    ) -> Self::HasCacheContext;
    fn HasCacheAllowed(&self, c: &Self::HasCacheContext) -> bool;
    fn HasCacheGetResult(&self, c: &Self::HasCacheContext, e: &Self::Element) -> u8;
    fn HasCacheAlreadyChecked(&self, c: &Self::HasCacheContext, e: &Self::Element) -> bool;
    fn HasCacheSetChecked(&self, c: &Self::HasCacheContext, e: &Self::Element);
    fn HasCacheSetMatched(&self, c: &Self::HasCacheContext, e: &Self::Element) -> u8;
    fn HasCacheSetAllTraversedChecked(
        &self,
        c: &Self::HasCacheContext,
        e: &Self::Element,
        depth: i32,
    );
    fn HasCacheEnsureFastRejectFilter(
        &self,
        c: &Self::HasCacheContext,
        e: &Self::Element,
    ) -> (Rc<Self::HasFastRejectFilter>, bool);
    fn HasBloomAllocated(&self, f: &Self::HasFastRejectFilter) -> bool;
    fn HasAllocateBloom(&self, f: &Self::HasFastRejectFilter);
    fn HasAddElementIdentifierHashes(&self, f: &Self::HasFastRejectFilter, e: &Self::Element);
    fn HasFastReject(&self, f: &Self::HasFastRejectFilter, c: &Self::HasArgumentContext) -> bool;
    fn StyleScopeParent(&self, s: &Self::StyleScope) -> Option<Rc<Self::StyleScope>>;
    fn StyleScopeFrom(&self, s: &Self::StyleScope) -> Option<Rc<CSSSelectorList>>;
    fn StyleScopeTo(&self, s: &Self::StyleScope) -> Option<Rc<CSSSelectorList>>;
    fn ElementTriggersScope(&self, e: &Self::Element, s: &Self::StyleScope) -> bool;
    fn StyleScopeFrameElement(&self, f: &Self::StyleScopeFrame) -> Rc<Self::Element>;
    fn ParentStyleScopeFrame(
        &self,
        f: &Self::StyleScopeFrame,
        e: &Self::Element,
    ) -> Option<Rc<Self::StyleScopeFrame>>;
    fn InsertScopeActivationCacheEntry(
        &self,
        f: &Self::StyleScopeFrame,
        s: &Self::StyleScope,
    ) -> Option<Rc<Self::StyleScopeActivations>>;
    fn SetScopeActivationCacheEntry(
        &self,
        f: &Self::StyleScopeFrame,
        s: &Self::StyleScope,
        a: Rc<Self::StyleScopeActivations>,
    );
    fn CreateActivation(
        &self,
        root: Option<Rc<Self::ContainerNode>>,
        proximity: u32,
    ) -> Rc<Self::StyleScopeActivation>;
    fn CreateActivations(
        &self,
        vector: Vec<Rc<Self::StyleScopeActivation>>,
        flags: MatchFlags,
    ) -> Rc<Self::StyleScopeActivations>;
    fn SpatialNavigationFocused(&self, e: &Self::Element) -> bool;
    fn HasDatalist(&self, e: &Self::Element) -> bool;
    fn IsSubmenuOpen(&self, e: &Self::Element) -> bool;
    fn UsesMenuList(&self, e: &Self::Element) -> bool;
    fn IsMultiSelectFocused(&self, e: &Self::Element) -> bool;
    fn IsResourceTarget(&self, e: &Self::Element) -> bool;
    fn CachedImageIsAnimated(&self, e: &Self::Element) -> bool;
    fn IsPlaceholderVisible(&self, e: &Self::Element) -> bool;
    fn MatchesToolFormActive(&self, e: &Self::Element) -> bool;
    fn IsUnboundedElementActive(&self, e: &Self::Element) -> bool;
    fn MatchesToolSubmitActive(&self, e: &Self::Element) -> bool;
    fn IsDragged(&self, e: &Self::Element) -> bool;
    fn HasFocusWithin(&self, e: &Self::Element) -> bool;
    fn IsFiltered(&self, e: &Self::Element) -> bool;
    fn HasInterest(&self, e: &Self::Element) -> bool;
    fn HasFlattenedAssignedNodes(&self, e: &Self::Element) -> bool;
    fn IsHovered(&self, e: &Self::Element) -> bool;
    fn IsActive(&self, e: &Self::Element) -> bool;
    fn MatchesEnabled(&self, e: &Self::Element) -> bool;
    fn IsMediaDocument(&self, e: &Self::Element) -> bool;
    fn MatchesDefault(&self, e: &Self::Element) -> bool;
    fn MatchesDisabled(&self, e: &Self::Element) -> bool;
    fn MatchesReadOnly(&self, e: &Self::Element) -> bool;
    fn MatchesReadWrite(&self, e: &Self::Element) -> bool;
    fn IsOptionalFormControl(&self, e: &Self::Element) -> bool;
    fn IsRequiredFormControl(&self, e: &Self::Element) -> bool;
    fn MatchesUserInvalid(&self, e: &Self::Element) -> bool;
    fn MatchesUserValid(&self, e: &Self::Element) -> bool;
    fn MatchesValidity(&self, e: &Self::Element) -> bool;
    fn IsValid(&self, e: &Self::Element) -> bool;
    fn ShouldAppearChecked(&self, e: &Self::Element) -> bool;
    fn ShouldAppearIndeterminate(&self, e: &Self::Element) -> bool;
    fn OptionSelected(&self, e: &Self::Element) -> bool;
    fn ScrollMarkerIsSelected(&self, e: &Self::Element) -> bool;
    fn IsTextField(&self, e: &Self::Element) -> bool;
    fn IsDialogInTopLayer(&self, e: &Self::Element) -> bool;
    fn IsPopoverInTopLayer(&self, e: &Self::Element) -> bool;
    fn IsPopover(&self, e: &Self::Element) -> bool;
    fn PopoverOpen(&self, e: &Self::Element) -> bool;
    fn MatchesOverscrollOpen(&self, e: &Self::Element) -> bool;
    fn HasOpenAttribute(&self, e: &Self::Element) -> bool;
    fn PopupIsVisible(&self, e: &Self::Element) -> bool;
    fn IsPickerVisible(&self, e: &Self::Element) -> bool;
    fn FullscreenFlag(&self, e: &Self::Element) -> bool;
    fn ContainsFullScreenElement(&self, e: &Self::Element) -> bool;
    fn Granted(&self, e: &Self::Element) -> bool;
    fn IsPictureInPicture(&self, e: &Self::Element) -> bool;
    fn MediaPaused(&self, e: &Self::Element) -> bool;
    fn MediaSeeking(&self, e: &Self::Element) -> bool;
    fn MediaBuffering(&self, e: &Self::Element) -> bool;
    fn MediaStalled(&self, e: &Self::Element) -> bool;
    fn MediaMuted(&self, e: &Self::Element) -> bool;
    fn VideoPersistent(&self, e: &Self::Element) -> bool;
    fn ContainsPersistentVideo(&self, e: &Self::Element) -> bool;
    fn IsXrOverlay(&self, e: &Self::Element) -> bool;
    fn IsFullscreenElement(&self, e: &Self::Element) -> bool;
    fn IsInRange(&self, e: &Self::Element) -> bool;
    fn IsOutOfRange(&self, e: &Self::Element) -> bool;
    fn VttIsPastNode(&self, e: &Self::Element) -> bool;
    fn IsDefined(&self, e: &Self::Element) -> bool;
    fn GetComputedStyle(&self, e: &Self::Element) -> Option<Persistent<ComputedStyle>>;
    fn HasEffectiveAppearance(&self, style: &ComputedStyle) -> bool;
    fn PageIsActive(&self, e: &Self::Element) -> bool;
    fn DidAttachInternals(&self, e: &Self::Element) -> bool;
    fn DialogIsModal(&self, e: &Self::Element) -> bool;
    fn IsAutofilled(&self, e: &Self::Element) -> bool;
    fn IsPreviewed(&self, e: &Self::Element) -> bool;
    fn AutofillStateIsPreviewed(&self, e: &Self::Element) -> bool;
    fn IsFocused(&self, e: &Self::Element) -> bool;
    fn AccessibilityAlwaysShowFocus(&self, e: &Self::Element) -> bool;
    fn MayTriggerVirtualKeyboard(&self, e: &Self::Element) -> bool;
    fn LastFocusTypeIsMouse(&self, e: &Self::Element) -> bool;
    fn HadKeyboardEvent(&self, e: &Self::Element) -> bool;
    fn AffectedByMultipleHas(&self, e: &Self::Element) -> bool;
}

// cpp: selector_checker.h:133-270. Selector is a borrowed cursor over the
// existing flat CSSSelector storage; cloning only retains DOM object identity.
pub struct SelectorCheckingContext<'s, B: SelectorCheckerBackend> {
    pub selector: Option<CSSSelectorComplex<'s>>,
    pub tree_scope: Option<Rc<B::TreeScope>>,
    pub scope: Option<Rc<B::ContainerNode>>,
    pub style_scope: Option<Rc<B::StyleScope>>,
    pub style_scope_frame: Option<Rc<B::StyleScopeFrame>>,
    pub element: Rc<B::Element>,
    pub previous_element: Option<Rc<B::Element>>,
    pub vtt_originating_element: Option<Rc<B::Element>>,
    pub relative_anchor_element: Option<Rc<B::ContainerNode>>,
    pub pseudo_argument: Option<AtomicString>,
    pub pseudo_id: PseudoId,
    pub previously_matched_pseudo_element: PseudoId,
    pub impact: Impact,
    pub is_sub_selector: bool,
    pub in_rightmost_compound: bool,
    pub has_scrollbar_pseudo: bool,
    pub in_nested_complex_selector: bool,
    pub match_visited: bool,
    pub had_match_visited: bool,
    pub pseudo_has_in_rightmost_compound: bool,
    pub is_inside_has_pseudo_class: bool,
    pub search_text_request_is_current: bool,
    pub pseudo_element: Option<Rc<B::Element>>,
    pub pseudo_element_ancestors: Vec<Rc<B::Element>>,
}
impl<'s, B: SelectorCheckerBackend> SelectorCheckingContext<'s, B> {
    pub fn new(element: Rc<B::Element>) -> Self {
        Self {
            selector: None,
            tree_scope: None,
            scope: None,
            style_scope: None,
            style_scope_frame: None,
            element,
            previous_element: None,
            vtt_originating_element: None,
            relative_anchor_element: None,
            pseudo_argument: None,
            pseudo_id: PseudoId::kPseudoIdNone,
            previously_matched_pseudo_element: PseudoId::kPseudoIdNone,
            impact: Impact::kSubject,
            is_sub_selector: false,
            in_rightmost_compound: true,
            has_scrollbar_pseudo: false,
            in_nested_complex_selector: false,
            match_visited: false,
            had_match_visited: false,
            pseudo_has_in_rightmost_compound: true,
            is_inside_has_pseudo_class: false,
            search_text_request_is_current: false,
            pseudo_element: None,
            pseudo_element_ancestors: Vec::new(),
        }
    }
    pub fn FromElementResolveContext(backend: &B, c: &B::ElementResolveContext) -> Self {
        let mut result = Self::new(backend.ResolveUltimateOriginatingElementOrSelf(c));
        result.pseudo_element = backend.ResolvePseudoElement(c);
        result.pseudo_element_ancestors = backend.ResolvePseudoElementAncestors(c);
        result
    }
    fn Rebind<'t>(
        &self,
        selector: Option<CSSSelectorComplex<'t>>,
    ) -> SelectorCheckingContext<'t, B> {
        SelectorCheckingContext {
            selector,
            tree_scope: self.tree_scope.clone(),
            scope: self.scope.clone(),
            style_scope: self.style_scope.clone(),
            style_scope_frame: self.style_scope_frame.clone(),
            element: self.element.clone(),
            previous_element: self.previous_element.clone(),
            vtt_originating_element: self.vtt_originating_element.clone(),
            relative_anchor_element: self.relative_anchor_element.clone(),
            pseudo_argument: self.pseudo_argument.clone(),
            pseudo_id: self.pseudo_id,
            previously_matched_pseudo_element: self.previously_matched_pseudo_element,
            impact: self.impact,
            is_sub_selector: self.is_sub_selector,
            in_rightmost_compound: self.in_rightmost_compound,
            has_scrollbar_pseudo: self.has_scrollbar_pseudo,
            in_nested_complex_selector: self.in_nested_complex_selector,
            match_visited: self.match_visited,
            had_match_visited: self.had_match_visited,
            pseudo_has_in_rightmost_compound: self.pseudo_has_in_rightmost_compound,
            is_inside_has_pseudo_class: self.is_inside_has_pseudo_class,
            search_text_request_is_current: self.search_text_request_is_current,
            pseudo_element: self.pseudo_element.clone(),
            pseudo_element_ancestors: self.pseudo_element_ancestors.clone(),
        }
    }
    // cpp: selector_checker.cc:489-508
    pub fn GetElementForMatching(&self, index: usize) -> Rc<B::Element> {
        if self.pseudo_element.is_none() || index == usize::MAX {
            return self.element.clone();
        }
        assert!(index <= self.pseudo_element_ancestors.len());
        let last = self
            .pseudo_element_ancestors
            .len()
            .checked_sub(1)
            .expect("pseudo-element must have its ancestors array");
        self.pseudo_element_ancestors[index.min(last)].clone()
    }
}
impl<B: SelectorCheckerBackend> Clone for SelectorCheckingContext<'_, B> {
    fn clone(&self) -> Self {
        self.Rebind(self.selector)
    }
}

// This is SelectorChecker::MatchResult, distinct from the existing aggregate
// resolver::match_result::MatchResult; no second aggregate match model is made.
pub struct MatchResult<B: SelectorCheckerBackend> {
    pub dynamic_pseudo: PseudoId,
    pub pseudo_ancestor_index: usize,
    pub custom_highlight_name: Option<AtomicString>,
    pub has_argument_leftmost_compound_matches: Option<Rc<RefCell<Vec<Rc<B::Element>>>>>,
    pub proximity: u32,
    pub flags: MatchFlags,
}
impl<B: SelectorCheckerBackend> Default for MatchResult<B> {
    fn default() -> Self {
        Self {
            dynamic_pseudo: PseudoId::kPseudoIdNone,
            pseudo_ancestor_index: usize::MAX,
            custom_highlight_name: None,
            has_argument_leftmost_compound_matches: None,
            proximity: u32::MAX,
            flags: 0,
        }
    }
}
impl<B: SelectorCheckerBackend> MatchResult<B> {
    pub fn SetFlag(&mut self, flag: MatchFlag) {
        self.flags |= flag as MatchFlags;
    }
    pub fn HasFlag(&self, flag: MatchFlag) -> bool {
        self.flags & flag as MatchFlags != 0
    }
    pub fn DescendToNextPseudoElement(&mut self) {
        self.pseudo_ancestor_index = if self.pseudo_ancestor_index == usize::MAX {
            0
        } else {
            self.pseudo_ancestor_index + 1
        };
    }
}
// cpp: selector_checker.h:351-386. Safe separate mutable references let callers
// access the outer result while retaining RAII propagation on every return.
pub struct SubResult<'p, B: SelectorCheckerBackend> {
    pub result: MatchResult<B>,
    parent: &'p mut MatchResult<B>,
}
impl<'p, B: SelectorCheckerBackend> SubResult<'p, B> {
    pub fn new(parent: &'p mut MatchResult<B>) -> Self {
        let mut result = MatchResult::default();
        result.pseudo_ancestor_index = parent.pseudo_ancestor_index;
        Self { result, parent }
    }
    pub fn PropagatePseudoAncestorIndex(&mut self) {
        let index = self.result.pseudo_ancestor_index;
        if index != usize::MAX {
            self.parent.pseudo_ancestor_index = if self.parent.pseudo_ancestor_index == usize::MAX {
                index
            } else {
                index.max(self.parent.pseudo_ancestor_index)
            };
        }
    }
    fn Parts(&mut self) -> (&mut MatchResult<B>, &mut MatchResult<B>) {
        (&mut self.result, self.parent)
    }
}
impl<B: SelectorCheckerBackend> Drop for SubResult<'_, B> {
    fn drop(&mut self) {
        self.parent.flags |= self.result.flags;
        self.PropagatePseudoAncestorIndex();
    }
}
struct DynamicPseudoScope<'p, B: SelectorCheckerBackend> {
    result: &'p mut MatchResult<B>,
    saved: PseudoId,
}
impl<'p, B: SelectorCheckerBackend> DynamicPseudoScope<'p, B> {
    fn new(result: &'p mut MatchResult<B>) -> Self {
        let saved = result.dynamic_pseudo;
        result.dynamic_pseudo = PseudoId::kPseudoIdNone;
        Self { result, saved }
    }
}
impl<B: SelectorCheckerBackend> Deref for DynamicPseudoScope<'_, B> {
    type Target = MatchResult<B>;
    fn deref(&self) -> &Self::Target {
        self.result
    }
}
impl<B: SelectorCheckerBackend> DerefMut for DynamicPseudoScope<'_, B> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.result
    }
}
impl<B: SelectorCheckerBackend> Drop for DynamicPseudoScope<'_, B> {
    fn drop(&mut self) {
        self.result.dynamic_pseudo = self.saved;
    }
}

pub struct SelectorChecker<B: SelectorCheckerBackend> {
    backend: Rc<B>,
    scrollbar_: Option<Rc<B::CustomScrollbar>>,
    part_names_: Option<Rc<B::PartNames>>,
    pseudo_argument_: AtomicString,
    pseudo_ident_list_: Vec<AtomicString>,
    scrollbar_part_: B::ScrollbarPart,
    mode_: Mode,
    is_ua_rule_: bool,
}
impl<B: SelectorCheckerBackend> SelectorChecker<B> {
    // cpp: selector_checker.h:89-108
    pub fn new(backend: Rc<B>, mode: Mode) -> Self {
        let part = backend.NoScrollbarPart();
        Self {
            backend,
            scrollbar_: None,
            part_names_: None,
            pseudo_argument_: AtomicString::default(),
            pseudo_ident_list_: Vec::new(),
            scrollbar_part_: part,
            mode_: mode,
            is_ua_rule_: false,
        }
    }
    pub fn ForStyleRequest(
        backend: Rc<B>,
        parts: Option<Rc<B::PartNames>>,
        request: &B::StyleRequest,
        mode: Mode,
        ua: bool,
    ) -> Self {
        Self {
            scrollbar_: backend.RequestScrollbar(request),
            part_names_: parts,
            pseudo_argument_: backend.RequestPseudoArgument(request),
            pseudo_ident_list_: backend.RequestPseudoIdentList(request),
            scrollbar_part_: backend.RequestScrollbarPart(request),
            backend,
            mode_: mode,
            is_ua_rule_: ua,
        }
    }
    fn Selector<'s>(&self, c: &SelectorCheckingContext<'s, B>) -> CSSSelectorComplex<'s> {
        c.selector.expect("selector checking requires a selector")
    }
    // ShadowHost's source-local helper cc:364-370 is required by this batch.
    fn ShadowHost(&self, c: &SelectorCheckingContext<'_, B>) -> Option<Rc<B::Element>> {
        c.tree_scope
            .as_deref()
            .and_then(|scope| self.backend.TreeScopeShadowHost(scope))
    }
    pub fn IsAtShadowHost(&self, c: &SelectorCheckingContext<'_, B>) -> bool {
        self.ShadowHost(c)
            .is_some_and(|h| Rc::ptr_eq(&h, &c.element))
    }
    fn ParentElement(&self, c: &SelectorCheckingContext<'_, B>) -> Option<Rc<B::Element>> {
        if c.tree_scope.is_none() {
            return self.backend.ParentElement(&c.element);
        }
        if self.IsAtShadowHost(c) {
            return None;
        }
        self.backend.ParentOrShadowHostElement(&c.element)
    }
    fn PreviousSiblingElement(&self, c: &SelectorCheckingContext<'_, B>) -> Option<Rc<B::Element>> {
        if self.IsAtShadowHost(c) {
            None
        } else {
            self.backend.PreviousSibling(&c.element)
        }
    }
    fn FindSlotElementInScope(&self, c: &SelectorCheckingContext<'_, B>) -> Option<Rc<B::Element>> {
        let mut slot = self.backend.AssignedSlot(&c.element);
        let Some(scope) = c.tree_scope.as_ref() else {
            return slot;
        };
        while let Some(current) = slot {
            if Rc::ptr_eq(&self.backend.ElementTreeScope(&current), scope) {
                return Some(current);
            }
            slot = self.backend.AssignedSlot(&current);
        }
        None
    }
    fn ShouldMatchHoverOrActive(&self, c: &SelectorCheckingContext<'_, B>) -> bool {
        if !self.backend.InQuirksMode(&c.element)
            || c.is_sub_selector
            || self.backend.IsLink(&c.element)
        {
            return true;
        }
        let mut selector = self.Selector(c);
        while selector.Relation() == RelationType::kSubSelector {
            let Some(next) = selector.NextSimpleSelector() else {
                break;
            };
            selector = next;
            if selector.Match() != MatchType::kPseudoClass
                || !matches!(
                    selector.GetPseudoType(),
                    PseudoType::kPseudoHover | PseudoType::kPseudoActive
                )
            {
                return true;
            }
        }
        false
    }
    fn Impacts(c: &SelectorCheckingContext<'_, B>, impact: Impact) -> bool {
        c.impact as u8 & impact as u8 != 0
    }
    fn ImpactsSubject(c: &SelectorCheckingContext<'_, B>) -> bool {
        Self::Impacts(c, Impact::kSubject)
    }
    fn ImpactsNonSubject(c: &SelectorCheckingContext<'_, B>) -> bool {
        Self::Impacts(c, Impact::kNonSubject)
    }
    fn IsFirstChild(&self, e: &B::Element) -> bool {
        self.backend.PreviousSibling(e).is_none()
    }
    fn IsLastChild(&self, e: &B::Element) -> bool {
        self.backend.NextSibling(e).is_none()
    }
    fn IsFirstOfType(&self, e: &B::Element, t: &QualifiedName) -> bool {
        self.backend.PreviousSiblingWithTagName(e, t).is_none()
    }
    fn IsLastOfType(&self, e: &B::Element, t: &QualifiedName) -> bool {
        self.backend.NextSiblingWithTagName(e, t).is_none()
    }
    fn DisallowMatchVisited(c: &mut SelectorCheckingContext<'_, B>) {
        c.had_match_visited |= c.match_visited;
        c.match_visited = false;
    }
    // cpp: selector_checker.cc:510-528
    pub fn Match(&self, c: &SelectorCheckingContext<'_, B>, r: &mut MatchResult<B>) -> bool {
        if c.vtt_originating_element.is_some() && self.Selector(c).IsLastInComplexSelector() {
            return false;
        }
        self.MatchSelector(c, r) == MatchStatus::kSelectorMatches
    }
    pub fn MatchWithoutResult(&self, c: &SelectorCheckingContext<'_, B>) -> bool {
        self.Match(c, &mut MatchResult::default())
    }
    fn MatchScrollButton(
        &self,
        e: &B::Element,
        c: &SelectorCheckingContext<'_, B>,
        r: &mut MatchResult<B>,
    ) -> bool {
        if !self.backend.IsPseudoElement(e) {
            r.dynamic_pseudo = PseudoId::kPseudoIdScrollButton;
            return true;
        }
        if !self.backend.IsScrollButtonPseudoElement(e) {
            return false;
        }
        let style = self
            .backend
            .ParentComputedStyle(e)
            .expect("scroll button requires parent style");
        let id = self
            .backend
            .ScrollButtonPseudoIdFromArgument(&self.Selector(c).Argument(), unsafe {
                &*style.Get()
            });
        id == PseudoId::kPseudoIdScrollButton || self.backend.ElementPseudoId(e) == id
    }
    fn NeedsScopeActivation(&self, c: &SelectorCheckingContext<'_, B>) -> bool {
        c.style_scope.is_some()
            && (self.Selector(c).IsScopeContaining() || self.Selector(c).IsLastInComplexSelector())
    }
    fn GetTransitionForScope(&self, e: &B::Element) -> Option<Rc<B::ViewTransition>> {
        if self.backend.IsPseudoElement(e) {
            None
        } else {
            self.backend.TransitionForElement(e)
        }
    }
    // cpp: selector_checker.cc:578-853. Featureless matching is tri-state;
    // negative logical selectors never turn an unknown into a match.
    fn MatchesShadowHostInComplexSelector(
        &self,
        c: &SelectorCheckingContext<'_, B>,
        r: &mut MatchResult<B>,
    ) -> FeaturelessMatch {
        let mut sub = c.clone();
        let mut matched = FeaturelessMatch::kFeaturelessMatches;
        while let Some(selector) = sub.selector {
            if selector.Relation() != RelationType::kSubSelector {
                return FeaturelessMatch::kFeaturelessUnknown;
            }
            let mut result = SubResult::new(r);
            match self.MatchShadowHost(&sub, &mut result.result) {
                FeaturelessMatch::kFeaturelessMatches => (),
                FeaturelessMatch::kFeaturelessFails => {
                    matched = FeaturelessMatch::kFeaturelessFails
                }
                FeaturelessMatch::kFeaturelessUnknown => {
                    return FeaturelessMatch::kFeaturelessUnknown
                }
            }
            sub.selector = selector.NextSimpleSelector();
        }
        matched
    }
    fn MatchesShadowHostInList(
        &self,
        c: &SelectorCheckingContext<'_, B>,
        list: &CSSSelectorList,
        r: &mut MatchResult<B>,
    ) -> FeaturelessMatch {
        let mut sub = c.Rebind(None);
        sub.is_sub_selector = true;
        sub.in_nested_complex_selector = true;
        sub.pseudo_id = PseudoId::kPseudoIdNone;
        sub.pseudo_element = None;
        let mut matched = FeaturelessMatch::kFeaturelessUnknown;
        for selector in list.ComplexSelectors() {
            sub.selector = Some(selector);
            match self.MatchesShadowHostInComplexSelector(&sub, r) {
                FeaturelessMatch::kFeaturelessMatches => {
                    return FeaturelessMatch::kFeaturelessMatches
                }
                FeaturelessMatch::kFeaturelessFails => {
                    matched = FeaturelessMatch::kFeaturelessFails
                }
                FeaturelessMatch::kFeaturelessUnknown => (),
            }
        }
        matched
    }
    fn MatchShadowHost(
        &self,
        c: &SelectorCheckingContext<'_, B>,
        r: &mut MatchResult<B>,
    ) -> FeaturelessMatch {
        let selector = self.Selector(c);

        match selector.GetPseudoType() {
            PseudoType::kPseudoIs | PseudoType::kPseudoWhere | PseudoType::kPseudoAny => selector
                .SelectorListOrParent()
                .map_or(FeaturelessMatch::kFeaturelessUnknown, |list| {
                    self.MatchesShadowHostInList(c, &list, r)
                }),
            PseudoType::kPseudoNot => {
                let result = selector
                    .SelectorListOrParent()
                    .map_or(FeaturelessMatch::kFeaturelessUnknown, |list| {
                        self.MatchesShadowHostInList(c, &list, r)
                    });
                match result {
                    FeaturelessMatch::kFeaturelessMatches => FeaturelessMatch::kFeaturelessFails,
                    FeaturelessMatch::kFeaturelessFails => FeaturelessMatch::kFeaturelessMatches,
                    FeaturelessMatch::kFeaturelessUnknown => FeaturelessMatch::kFeaturelessUnknown,
                }
            }
            PseudoType::kPseudoParent => {
                if let Some(list) = selector.SelectorListOrParent() {
                    self.MatchesShadowHostInList(c, &list, r)
                } else {
                    if self.CheckPseudoScope(c, r) {
                        FeaturelessMatch::kFeaturelessMatches
                    } else {
                        FeaturelessMatch::kFeaturelessFails
                    }
                }
            }
            PseudoType::kPseudoHostContext | PseudoType::kPseudoHost => {
                if self.CheckPseudoHost(c, r) {
                    FeaturelessMatch::kFeaturelessMatches
                } else {
                    FeaturelessMatch::kFeaturelessFails
                }
            }
            PseudoType::kPseudoScope => {
                if self.CheckPseudoScope(c, r) {
                    FeaturelessMatch::kFeaturelessMatches
                } else {
                    FeaturelessMatch::kFeaturelessFails
                }
            }
            PseudoType::kPseudoHas => {
                if self.CheckPseudoHas(c, r) {
                    FeaturelessMatch::kFeaturelessMatches
                } else {
                    FeaturelessMatch::kFeaturelessFails
                }
            }
            _ => FeaturelessMatch::kFeaturelessUnknown,
        }
    }
    // cpp: selector_checker.cc:861-947. SubResult and dynamic-pseudo scopes
    // preserve flag/index propagation and restoration on every early return.
    pub fn MatchSelector(
        &self,
        c: &SelectorCheckingContext<'_, B>,
        r: &mut MatchResult<B>,
    ) -> MatchStatus {
        use MatchStatus::*;
        if self.NeedsScopeActivation(c) {
            return self.MatchForScopeActivation(c, r);
        }
        let mut sub = SubResult::new(r);
        let selector = self.Selector(c);
        let covered =
            selector.IsCoveredByBucketing() && !c.is_sub_selector && c.tree_scope.is_none();
        if !covered && !self.CheckOne(c, &mut sub.result) {
            return kSelectorFailsLocally;
        }
        sub.PropagatePseudoAncestorIndex();
        let (child, r) = sub.Parts();
        if child.dynamic_pseudo != PseudoId::kPseudoIdNone || c.pseudo_element.is_some() {
            r.dynamic_pseudo = child.dynamic_pseudo;
            r.custom_highlight_name = child.custom_highlight_name.take();
        }
        if selector.IsLastInComplexSelector()
            || !matches!(
                selector.Relation(),
                RelationType::kSubSelector | RelationType::kPseudoChild
            )
        {
            if !self.backend.CSSLogicalCombinationPseudoEnabled()
                && c.pseudo_id != PseudoId::kPseudoIdNone
                && c.pseudo_id != r.dynamic_pseudo
            {
                return kSelectorFailsCompletely;
            }
            if c.pseudo_element.is_some()
                && (r.pseudo_ancestor_index == usize::MAX
                    || r.pseudo_ancestor_index < c.pseudo_element_ancestors.len() - 1)
            {
                return kSelectorFailsCompletely;
            }
        }
        if selector.IsLastInComplexSelector() {
            return kSelectorMatches;
        }
        if selector.Relation() == RelationType::kSubSelector {
            self.MatchForSubSelector(c, r)
        } else {
            let mut reset = DynamicPseudoScope::new(r);
            self.MatchForRelation(c, &mut reset)
        }
    }
    fn PrepareNextContextForRelation<'s>(
        &self,
        c: &SelectorCheckingContext<'s, B>,
    ) -> SelectorCheckingContext<'s, B> {
        c.Rebind(self.Selector(c).NextSimpleSelector())
    }
    fn MatchForSubSelector(
        &self,
        c: &SelectorCheckingContext<'_, B>,
        r: &mut MatchResult<B>,
    ) -> MatchStatus {
        use MatchStatus::*;
        let mut next = self.PrepareNextContextForRelation(c);
        if next.pseudo_element.is_some()
            && r.pseudo_ancestor_index != usize::MAX
            && r.pseudo_ancestor_index > next.pseudo_element_ancestors.len()
        {
            return kSelectorFailsLocally;
        }
        let dynamic = r.dynamic_pseudo;
        next.has_scrollbar_pseudo = dynamic != PseudoId::kPseudoIdNone
            && (self.scrollbar_.is_some()
                || dynamic == PseudoId::kPseudoIdScrollbarCorner
                || dynamic == PseudoId::kPseudoIdResizer);
        if c.in_rightmost_compound
            && dynamic != PseudoId::kPseudoIdNone
            && c.pseudo_element.is_none()
            && c.pseudo_id == PseudoId::kPseudoIdNone
        {
            if !next.has_scrollbar_pseudo && dynamic == PseudoId::kPseudoIdScrollbar {
                return kSelectorFailsCompletely;
            }
            return kSelectorMatches;
        }
        next.previously_matched_pseudo_element = dynamic;
        next.is_sub_selector = true;
        self.MatchSelector(&next, r)
    }
    fn MatchForScopeActivation(
        &self,
        c: &SelectorCheckingContext<'_, B>,
        r: &mut MatchResult<B>,
    ) -> MatchStatus {
        use MatchStatus::*;
        let scope = c
            .style_scope
            .as_deref()
            .expect("scope activation requires a StyleScope");
        let mut next = c.clone();
        next.is_sub_selector = true;
        let activations = self.EnsureActivations(c, scope);
        if Self::ImpactsSubject(c) {
            r.flags |= self.backend.ActivationMatchFlags(&activations);
        }
        let vector = self.backend.ActivationVector(&activations);
        if vector.is_empty() {
            return kSelectorFailsCompletely;
        }
        for activation in vector.into_iter().rev() {
            next.match_visited = c.match_visited;
            next.impact = c.impact;
            next.style_scope = None;
            next.scope = self.backend.ActivationRoot(&activation);
            assert!(!self.NeedsScopeActivation(&next));
            if self.MatchSelector(&next, r) == kSelectorMatches {
                r.proximity = self.backend.ActivationProximity(&activation);
                return kSelectorMatches;
            }
        }
        kSelectorFailsLocally
    }
    fn RecordRelativeAnchor(&self, c: &SelectorCheckingContext<'_, B>, r: &mut MatchResult<B>) {
        r.has_argument_leftmost_compound_matches
            .as_ref()
            .expect("relative selector requires has() matches vector")
            .borrow_mut()
            .push(c.element.clone());
    }
    // cpp: selector_checker.cc:1075-1291. All parent/sibling/slot/shadow loops
    // and the four distinct failure statuses remain in Rust.
    fn MatchForRelation(
        &self,
        c: &SelectorCheckingContext<'_, B>,
        r: &mut MatchResult<B>,
    ) -> MatchStatus {
        use MatchStatus::*;
        use RelationType::*;
        let mut next = self.PrepareNextContextForRelation(c);
        let relation = self.Selector(c).Relation();
        if self.backend.IsLink(&c.element) || !matches!(relation, kDescendant | kChild) {
            Self::DisallowMatchVisited(&mut next);
        }
        next.in_rightmost_compound = false;
        next.impact = Impact::kNonSubject;
        next.is_sub_selector = false;
        next.previous_element = Some(c.element.clone());
        next.pseudo_id = PseudoId::kPseudoIdNone;
        next.pseudo_element = None;
        match relation {
            kRelativeDescendant | kDescendant => {
                if relation == kRelativeDescendant {
                    self.RecordRelativeAnchor(c, r);
                }
                let mut parent = self.ParentElement(&next);
                while let Some(element) = parent {
                    next.element = element;
                    let status = self.MatchSelector(&next, r);
                    if matches!(status, kSelectorMatches | kSelectorFailsCompletely) {
                        return status;
                    }
                    if self.backend.IsLink(&next.element) {
                        Self::DisallowMatchVisited(&mut next);
                    }
                    parent = self.ParentElement(&next);
                }
                kSelectorFailsCompletely
            }
            kRelativeChild | kChild => {
                if relation == kRelativeChild {
                    self.RecordRelativeAnchor(c, r);
                }
                let Some(parent) = self.ParentElement(&next) else {
                    return kSelectorFailsCompletely;
                };
                next.element = parent;
                let status = self.MatchSelector(&next, r);
                if status == kSelectorFailsLocally {
                    kSelectorFailsAllSiblings
                } else {
                    status
                }
            }
            kRelativeDirectAdjacent | kDirectAdjacent => {
                if relation == kRelativeDirectAdjacent {
                    self.RecordRelativeAnchor(c, r);
                }
                if self.mode_ == Mode::kResolvingStyle {
                    if let Some(parent) = self.backend.ParentElementOrShadowRoot(&c.element) {
                        self.backend
                            .SetChildrenAffectedByDirectAdjacentRules(&parent);
                    }
                }
                let Some(previous) = self.PreviousSiblingElement(c) else {
                    return kSelectorFailsAllSiblings;
                };
                next.element = previous;
                self.MatchSelector(&next, r)
            }
            kRelativeIndirectAdjacent | kIndirectAdjacent => {
                if relation == kRelativeIndirectAdjacent {
                    self.RecordRelativeAnchor(c, r);
                }
                if self.mode_ == Mode::kResolvingStyle {
                    if let Some(parent) = self.backend.ParentElementOrShadowRoot(&c.element) {
                        self.backend
                            .SetChildrenAffectedByIndirectAdjacentRules(&parent);
                    }
                }
                let mut previous = self.PreviousSiblingElement(c);
                while let Some(element) = previous {
                    next.element = element;
                    let status = self.MatchSelector(&next, r);
                    if matches!(
                        status,
                        kSelectorMatches | kSelectorFailsAllSiblings | kSelectorFailsCompletely
                    ) {
                        return status;
                    }
                    previous = self.PreviousSiblingElement(&next);
                }
                kSelectorFailsAllSiblings
            }
            kPseudoChild => {
                if c.pseudo_id != PseudoId::kPseudoIdNone {
                    next.pseudo_id = PseudoId::kPseudoIdNone;
                    next.pseudo_element = c.pseudo_element.clone();
                } else {
                    let pseudo = c
                        .pseudo_element
                        .as_deref()
                        .expect("pseudo-child requires a pseudo-element");
                    next.pseudo_id = PseudoId::kPseudoIdNone;
                    next.pseudo_element = self
                        .backend
                        .ParentElement(pseudo)
                        .filter(|parent| self.backend.IsPseudoElement(parent));
                }
                self.MatchSelector(&next, r)
            }
            kUAShadow => {
                if c.tree_scope.as_ref().is_some_and(|scope| {
                    self.backend.TreeScopeIsShadowRoot(scope)
                        && Rc::ptr_eq(scope, &self.backend.ElementTreeScope(&c.element))
                }) {
                    return kSelectorFailsCompletely;
                }
                let Some(mut host) = self.backend.OwnerShadowHost(&c.element) else {
                    return kSelectorFailsCompletely;
                };
                if let Some(origin) = c.vtt_originating_element.as_ref() {
                    host = origin.clone();
                }
                next.element = host;
                if c.tree_scope.as_ref().is_some_and(|scope| {
                    !Rc::ptr_eq(scope, &self.backend.ElementTreeScope(&next.element))
                }) && !self.Selector(&next).CrossesTreeScopes()
                {
                    return kSelectorFailsCompletely;
                }
                self.MatchSelector(&next, r)
            }
            kShadowSlot => {
                if self.backend.IsSlotSupportingAssignment(&c.element) {
                    return kSelectorFailsCompletely;
                }
                let Some(slot) = self.FindSlotElementInScope(c) else {
                    return kSelectorFailsCompletely;
                };
                next.element = slot;
                self.MatchSelector(&next, r)
            }
            kShadowPart => {
                let scope = c
                    .tree_scope
                    .as_ref()
                    .expect("shadow part requires tree scope");
                loop {
                    let Some(host) = self.backend.OwnerShadowHost(&next.element) else {
                        return kSelectorFailsCompletely;
                    };
                    next.element = host;
                    let host_scope = if self.Selector(&next).IsDeeplyHostPseudoClass()
                        && Rc::ptr_eq(&self.backend.ElementTreeScope(&c.element), scope)
                    {
                        self.backend
                            .ParentTreeScope(scope)
                            .expect("host tree scope requires a parent scope")
                    } else {
                        scope.clone()
                    };
                    if Rc::ptr_eq(&self.backend.ElementTreeScope(&next.element), &host_scope) {
                        return self.MatchSelector(&next, r);
                    }
                }
            }
            kSubSelector => unreachable!("subselectors are handled by MatchForSubSelector"),
        }
    }
    // cpp: selector_checker.cc:1401-1472
    fn AnyAttributeMatches(
        &self,
        e: &B::Element,
        kind: MatchType,
        selector: CSSSelectorComplex<'_>,
    ) -> bool {
        let name = selector.Attribute();
        self.backend.SynchronizeAttribute(e, name.LocalName());
        if !self.backend.CouldHaveAttribute(e, &name) {
            return false;
        }
        // The source's value reference is never used for an attribute-set test.
        let value = if kind == MatchType::kAttributeSet {
            AtomicString::default()
        } else {
            selector.Value().clone()
        };
        let insensitive = selector.AttributeMatch() == AttributeMatchType::kCaseInsensitive
            || (selector.LegacyCaseInsensitiveMatch()
                && (!self.backend.CSSAttributeValueCaseSensitiveNonHTMLEnabled()
                    || self.backend.IsHTMLElement(e))
                && self.backend.IsInHTMLDocument(e));
        for attribute in self.backend.AttributesWithoutUpdate(e) {
            if !self.backend.AttributeMatchesName(&attribute, &name) {
                if self.backend.IsHTMLElement(e) || !self.backend.IsInHTMLDocument(e) {
                    continue;
                }
                if !self
                    .backend
                    .AttributeMatchesNameCaseInsensitive(&attribute, &name)
                {
                    continue;
                }
            }
            if AttributeValueMatches(
                &self.backend.AttributeValue(&attribute),
                kind,
                &value,
                insensitive,
            ) {
                return true;
            }
            if name.NamespaceURI() != &AtomicString::from_str("*") {
                return false;
            }
        }
        false
    }
    fn GetCandidateElement(
        &self,
        c: &SelectorCheckingContext<'_, B>,
        r: &MatchResult<B>,
    ) -> Rc<B::Element> {
        if self.backend.CSSLogicalCombinationPseudoEnabled() {
            c.pseudo_element
                .clone()
                .unwrap_or_else(|| c.element.clone())
        } else {
            c.GetElementForMatching(r.pseudo_ancestor_index)
        }
    }
    // cpp: selector_checker.cc:1484-1557. Simple selector dispatch.
    fn CheckOne(&self, c: &SelectorCheckingContext<'_, B>, r: &mut MatchResult<B>) -> bool {
        let element = &c.element;
        let selector = self.Selector(c);

        if self.IsAtShadowHost(c) && selector.Match() != MatchType::kPseudoElement {
            if !selector.IsHostPseudoClass()
                && selector.SelectorListOrParent().is_none()
                && selector.GetPseudoType() != PseudoType::kPseudoScope
            {
                return false;
            }
            return self.MatchShadowHost(c, r) == FeaturelessMatch::kFeaturelessMatches;
        }
        if self.backend.CSSLogicalCombinationPseudoEnabled() {
            if c.pseudo_id != PseudoId::kPseudoIdNone {
                return self.CheckVirtualPseudo(c, r);
            } else if c.pseudo_element.is_some()
                && !matches!(
                    selector.Match(),
                    MatchType::kPseudoElement | MatchType::kPseudoClass
                )
            {
                return false;
            }
        }
        match selector.Match() {
            MatchType::kTag => self.MatchesTagName(element, selector.TagQName()),
            MatchType::kUniversalTag => self.MatchesUniversalTagName(element, selector.TagQName()),
            MatchType::kClass => {
                if !self.backend.CouldHaveClass(element, &selector.Value()) {
                    return false;
                }
                self.backend.HasClass(element)
                    && self.backend.ClassNamesContain(element, &selector.Value())
            }
            MatchType::kId => {
                self.backend.HasID(element)
                    && self.backend.IdForStyleResolution(element) == *selector.Value()
            }
            MatchType::kAttributeExact
            | MatchType::kAttributeSet
            | MatchType::kAttributeHyphen
            | MatchType::kAttributeList
            | MatchType::kAttributeContain
            | MatchType::kAttributeBegin
            | MatchType::kAttributeEnd => {
                self.AnyAttributeMatches(element, selector.Match(), selector)
            }
            MatchType::kPseudoClass => self.CheckPseudoClass(c, r),
            MatchType::kPseudoElement => self.CheckPseudoElement(c, r),
            _ => unreachable!("unexpected selector match type"),
        }
    }
}

// cpp: selector_checker.cc:1294-1400. Preserve UTF-16 positions and HTML-space
// token boundaries; ASCII-insensitive comparison never folds Unicode case.
fn AttributeValueMatches(
    value: &AtomicString,
    kind: MatchType,
    selector: &AtomicString,
    insensitive: bool,
) -> bool {
    fn lower(u: u16) -> u16 {
        if (0x41..=0x5a).contains(&u) {
            u + 0x20
        } else {
            u
        }
    }
    fn html_space(u: u16) -> bool {
        matches!(u, 0x09 | 0x0a | 0x0c | 0x0d | 0x20)
    }
    let value = value.utf16_units().unwrap_or_default();
    let selector = selector.utf16_units().unwrap_or_default();
    let equal = |a: &[u16], b: &[u16]| {
        a == b
            || (insensitive
                && a.len() == b.len()
                && a.iter().zip(b).all(|(a, b)| lower(*a) == lower(*b)))
    };
    let starts = |a: &[u16]| a.len() >= selector.len() && equal(&a[..selector.len()], &selector);
    match kind {
        MatchType::kAttributeExact => equal(&value, &selector),
        MatchType::kAttributeSet => true,
        MatchType::kAttributeList => {
            if selector.is_empty() || selector.iter().any(|u| html_space(*u)) {
                return false;
            }
            let mut at = 0;
            while at + selector.len() <= value.len() {
                if equal(&value[at..at + selector.len()], &selector)
                    && (at == 0 || html_space(value[at - 1]))
                    && (at + selector.len() == value.len()
                        || html_space(value[at + selector.len()]))
                {
                    return true;
                }
                at += 1;
            }
            false
        }
        MatchType::kAttributeContain => {
            !selector.is_empty()
                && value
                    .windows(selector.len())
                    .any(|part| equal(part, &selector))
        }
        MatchType::kAttributeBegin => !selector.is_empty() && starts(&value),
        MatchType::kAttributeEnd => {
            !selector.is_empty()
                && value.len() >= selector.len()
                && equal(&value[value.len() - selector.len()..], &selector)
        }
        MatchType::kAttributeHyphen => {
            starts(&value) && (value.len() == selector.len() || value[selector.len()] == 0x2d)
        }
        _ => unreachable!("non-attribute selector"),
    }
}

// cpp: selector_checker.cc:118-376. Source-local helpers stay local algorithms.
fn ascii_equal(a: &AtomicString, b: &AtomicString) -> bool {
    let a = a.utf16_units().unwrap_or_default();
    let b = b.utf16_units().unwrap_or_default();
    a.len() == b.len()
        && a.iter()
            .zip(b)
            .all(|(a, b)| ascii_lower(*a) == ascii_lower(*b))
}
fn ascii_lower(u: u16) -> u16 {
    if (65..=90).contains(&u) {
        u + 32
    } else {
        u
    }
}
fn IsValidBCP47Value(value: &AtomicString, allow_wildcards: bool) -> bool {
    let value = value.utf16_units().unwrap_or_default();
    if value.is_empty() {
        return false;
    }
    for (index, subtag) in value.split(|c| *c == 45).enumerate() {
        if subtag.is_empty() {
            return false;
        }
        if subtag == [42] {
            if !allow_wildcards {
                return false;
            }
        } else if subtag.len() > 8
            || !subtag.iter().all(|c| {
                (65..=90).contains(c)
                    || (97..=122).contains(c)
                    || (index > 0 && (48..=57).contains(c))
            })
        {
            return false;
        }
    }
    true
}
fn MatchesLangPseudoClass(language: &AtomicString, ranges: &[AtomicString]) -> bool {
    for range in ranges {
        if language.empty() {
            if !language.IsNull() && range.empty() {
                return true;
            }
            continue;
        }
        if !IsValidBCP47Value(range, true) {
            continue;
        }
        let lang: Vec<_> = language
            .utf16_units()
            .unwrap()
            .split(|c| *c == 45)
            .collect();
        let range: Vec<_> = range.utf16_units().unwrap().split(|c| *c == 45).collect();
        let same = |a: &[u16], b: &[u16]| {
            a.len() == b.len()
                && a.iter()
                    .zip(b)
                    .all(|(a, b)| ascii_lower(*a) == ascii_lower(*b))
        };
        if range[0] != [42] && !same(range[0], lang[0]) {
            continue;
        }
        let (mut r, mut l) = (1, 1);
        while r < range.len() && l < lang.len() {
            if range[r] == [42] || same(range[r], lang[l]) {
                r += 1;
                l += 1;
            } else if lang[l].len() == 1 {
                break;
            } else {
                l += 1;
            }
        }
        if r == range.len() {
            return true;
        }
    }
    false
}
impl<B: SelectorCheckerBackend> SelectorChecker<B> {
    fn IsFrameFocused(&self, e: &B::Element) -> bool {
        self.backend.FrameFocusedAndActive(e).is_some_and(|v| v)
    }
    fn MatchesTagName(&self, e: &B::Element, name: &QualifiedName) -> bool {
        let local = self.backend.ElementLocalName(e);
        if name.LocalName() != &local {
            if self.backend.IsHTMLElement(e) || !self.backend.IsInHTMLDocument(e) {
                return false;
            }
            if !ascii_equal(&local, name.LocalName()) {
                return false;
            }
        }
        name.NamespaceURI() == &AtomicString::from_str("*")
            || name.NamespaceURI() == &self.backend.ElementNamespaceURI(e)
    }
    fn MatchesUniversalTagName(&self, e: &B::Element, name: &QualifiedName) -> bool {
        name == &QualifiedName::AnyQName()
            || name.NamespaceURI() == &AtomicString::from_str("*")
            || name.NamespaceURI() == &self.backend.ElementNamespaceURI(e)
    }
    fn MatchesExternalSVGUseTarget(&self, e: &B::Element) -> bool {
        if !self.backend.ElementIs(e, ElementClass::SVG) {
            return false;
        }
        self.backend.CorrespondingSVGElement(e).map_or_else(
            || self.backend.IsResourceTarget(e),
            |corresponding| self.backend.IsResourceTarget(&corresponding),
        )
    }
    fn MatchesUAShadowElement(&self, e: &B::Element, id: &AtomicString) -> bool {
        let origin = self
            .backend
            .IsPseudoElement(e)
            .then(|| self.backend.UltimateOriginatingElement(e));
        let e = origin.as_deref().unwrap_or(e);
        self.backend.ContainingShadowRootIsUserAgent(e) == Some(true)
            && self.backend.ShadowPseudoId(e) == *id
    }
    fn MatchesAnyInList(
        &self,
        c: &SelectorCheckingContext<'_, B>,
        list: &CSSSelectorList,
        r: &mut MatchResult<B>,
    ) -> bool {
        let mut sub = c.Rebind(list.First());
        sub.is_sub_selector = true;
        sub.in_nested_complex_selector = true;
        if !self.backend.CSSLogicalCombinationPseudoEnabled() {
            sub.pseudo_id = PseudoId::kPseudoIdNone;
            sub.pseudo_element = None;
        }
        while let Some(selector) = sub.selector {
            let mut result = SubResult::new(r);
            if self.MatchSelector(&sub, &mut result.result) == MatchStatus::kSelectorMatches {
                return true;
            }
            sub.selector = selector.NextComplexSelector();
        }
        false
    }
    fn CheckPseudoNot(&self, c: &SelectorCheckingContext<'_, B>, r: &mut MatchResult<B>) -> bool {
        let list = self.Selector(c).SelectorList().expect(":not selector list");
        !self.MatchesAnyInList(c, &list, r)
    }
    fn CheckPseudoLinkTo(
        &self,
        c: &SelectorCheckingContext<'_, B>,
        r: &mut MatchResult<B>,
    ) -> bool {
        let location = self
            .Selector(c)
            .GetNavigationLocation()
            .expect("link-to location");
        self.backend
            .NavigationLocationMatches(location.as_ref(), &self.GetCandidateElement(c, r))
    }
    fn CheckPseudoAutofill(&self, p: PseudoType, e: &B::Element) -> bool {
        use PseudoType::*;
        if self.backend.ProbeForcePseudoState(e, kPseudoAutofill) {
            return true;
        }
        if !self.backend.ElementIs(e, ElementClass::FormControl) {
            return false;
        }
        match p {
            kPseudoAutofill | kPseudoWebKitAutofill => {
                self.backend.IsAutofilled(e) || self.backend.IsPreviewed(e)
            }
            kPseudoAutofillPreviewed => self.backend.AutofillStateIsPreviewed(e),
            kPseudoAutofillSelected => self.backend.IsAutofilled(e),
            _ => unreachable!("autofill pseudo type"),
        }
    }
    fn CheckPseudoHost(&self, c: &SelectorCheckingContext<'_, B>, r: &mut MatchResult<B>) -> bool {
        let selector = self.Selector(c);
        let element = c.GetElementForMatching(r.pseudo_ancestor_index);
        if c.tree_scope.is_none()
            || !self
                .ShadowHost(c)
                .is_some_and(|host| Rc::ptr_eq(&host, &element))
        {
            return false;
        }
        let Some(list) = selector.SelectorList() else {
            return true;
        };
        let mut sub = c.Rebind(list.First());
        sub.is_sub_selector = true;
        sub.pseudo_id = PseudoId::kPseudoIdNone;
        sub.pseudo_element = None;
        let scope = self.backend.ElementTreeScope(&c.element);
        sub.scope = Some(self.backend.TreeScopeRoot(&scope));
        sub.tree_scope = Some(scope);
        let mut next = Some(element);
        while let Some(element) = next {
            sub.element = element;
            let mut result = SubResult::new(r);
            if self.MatchSelector(&sub, &mut result.result) == MatchStatus::kSelectorMatches {
                return true;
            }
            sub.scope = None;
            sub.tree_scope = None;
            if selector.GetPseudoType() == PseudoType::kPseudoHost {
                break;
            }
            sub.in_rightmost_compound = false;
            sub.impact = Impact::kNonSubject;
            next = self.backend.FlatTreeParentElement(&sub.element);
        }
        false
    }
    fn CheckPseudoScope(
        &self,
        c: &SelectorCheckingContext<'_, B>,
        _r: &mut MatchResult<B>,
    ) -> bool {
        let Some(scope) = c.scope.as_ref() else {
            return false;
        };
        if let Some(element) = self.backend.ContainerAsElement(scope) {
            Rc::ptr_eq(&element, &c.element)
        } else {
            self.backend
                .DocumentElement(&c.element)
                .is_some_and(|e| Rc::ptr_eq(&e, &c.element))
        }
    }
    fn CheckVirtualPseudo(
        &self,
        c: &SelectorCheckingContext<'_, B>,
        r: &mut MatchResult<B>,
    ) -> bool {
        let selector = self.Selector(c);
        use PseudoType::*;
        match selector.Match() {
            MatchType::kPseudoClass => match selector.GetPseudoType() {
                kPseudoIs | kPseudoWhere => selector
                    .SelectorListOrParent()
                    .is_some_and(|list| self.MatchesAnyInList(c, &list, r)),
                kPseudoNot => self.CheckPseudoNot(c, r),
                _ => false,
            },
            MatchType::kPseudoElement => match selector.GetPseudoType() {
                kPseudoBefore | kPseudoAfter | kPseudoMarker | kPseudoBackdrop => {
                    c.pseudo_id == crate::css_selector::GetPseudoId(selector.GetPseudoType())
                }
                _ => false,
            },
            _ => false,
        }
    }
    pub fn MatchesActiveViewTransitionPseudoClass(&self, e: &B::Element) -> bool {
        self.GetTransitionForScope(e).is_some()
    }
    pub fn MatchesFocusPseudoClass(&self, e: &B::Element, pseudo: PseudoId) -> bool {
        let pseudo = if pseudo != PseudoId::kPseudoIdNone {
            let Some(p) = self.backend.GetPseudoElement(e, pseudo) else {
                return false;
            };
            Some(p)
        } else {
            None
        };
        let e = pseudo.as_deref().unwrap_or(e);
        self.backend
            .ProbeForcePseudoState(e, PseudoType::kPseudoFocus)
            || (self.backend.IsFocused(e) && self.IsFrameFocused(e))
    }
    pub fn MatchesFocusVisiblePseudoClass(&self, e: &B::Element) -> bool {
        let b = &self.backend;
        if b.ProbeForcePseudoState(e, PseudoType::kPseudoFocusVisible) {
            return true;
        }
        if !b.IsFocused(e) || !self.IsFrameFocused(e) {
            return false;
        }
        if !b
            .FocusedElement(e)
            .is_some_and(|focused| std::ptr::eq(focused.as_ref(), e))
            && b.HasShadowRoot(e)
            && !b.ShadowRootIsUserAgent(e)
        {
            return false;
        }
        let accessibility = b.AccessibilityAlwaysShowFocus(e);
        let option = b.FocusVisibleOption(e);
        if !accessibility && option == Some(false) {
            return false;
        }
        let always = accessibility || option == Some(true);
        let mouse = b.FrameFocusedAndActive(e).is_some_and(|v| v) && b.LastFocusTypeIsMouse(e);
        always || b.MayTriggerVirtualKeyboard(e) || !mouse || b.HadKeyboardEvent(e)
    }
    fn CheckScrollbarPseudoClass(
        &self,
        c: &SelectorCheckingContext<'_, B>,
        r: &mut MatchResult<B>,
    ) -> bool {
        use PseudoType::*;
        let p = self.Selector(c).GetPseudoType();
        let b = &self.backend;
        if p == kPseudoNot {
            return self.CheckPseudoNot(c, r);
        }
        if p == kPseudoWindowInactive {
            return !b.PageIsActive(&c.element);
        }
        let Some(scrollbar) = self.scrollbar_.as_ref() else {
            return false;
        };
        // Values are the actual ScrollbarPart bit representation; no duplicate part type.
        let part = b.ScrollbarPartBits(&self.scrollbar_part_);
        match p {
            kPseudoEnabled => b.ScrollbarEnabled(scrollbar),
            kPseudoDisabled => !b.ScrollbarEnabled(scrollbar),
            kPseudoHover | kPseudoActive => {
                let selected = if p == kPseudoHover {
                    b.ScrollbarHoveredPart(scrollbar)
                } else {
                    b.ScrollbarPressedPart(scrollbar)
                };
                let selected = b.ScrollbarPartBits(&selected);
                if part == 128 {
                    selected != 0
                } else if part == 256 {
                    matches!(selected, 4 | 16 | 8)
                } else {
                    part == selected
                }
            }
            kPseudoHorizontal => b.ScrollbarHorizontal(scrollbar),
            kPseudoVertical => !b.ScrollbarHorizontal(scrollbar),
            kPseudoDecrement => matches!(part, 1 | 32 | 4),
            kPseudoIncrement => matches!(part, 2 | 64 | 16),
            kPseudoStart => matches!(part, 1 | 2 | 4),
            kPseudoEnd => matches!(part, 32 | 64 | 16),
            kPseudoDoubleButton => false,
            kPseudoSingleButton => {
                b.ScrollbarNativeThemeHasButtons(scrollbar) && matches!(part, 1 | 64 | 4 | 16)
            }
            kPseudoNoButton => {
                !b.ScrollbarNativeThemeHasButtons(scrollbar) && matches!(part, 4 | 16)
            }
            kPseudoCornerPresent => b.ScrollbarCornerVisible(scrollbar),
            _ => false,
        }
    }
    fn CheckPseudoElement(
        &self,
        c: &SelectorCheckingContext<'_, B>,
        r: &mut MatchResult<B>,
    ) -> bool {
        use PseudoType::*;
        let selector = self.Selector(c);
        let p = selector.GetPseudoType();
        let id = crate::css_selector::GetPseudoId(p);
        let b = &self.backend;
        if id != PseudoId::kPseudoIdNone && id <= PseudoId::kLastPublicPseudoId {
            r.DescendToNextPseudoElement();
        }
        let element = self.GetCandidateElement(c, r);
        let e = element.as_ref();
        if !b.CSSLogicalCombinationPseudoEnabled() && c.in_nested_complex_selector {
            return false;
        }
        match p {
            kPseudoCue => {
                let list = selector.SelectorList().expect("cue selector list");
                let mut sub = c.Rebind(list.First());
                sub.is_sub_selector = true;
                sub.scope = None;
                sub.tree_scope = None;
                while let Some(s) = sub.selector {
                    let mut result = SubResult::new(r);
                    if self.MatchSelector(&sub, &mut result.result) == MatchStatus::kSelectorMatches
                    {
                        return true;
                    }
                    sub.selector = s.NextComplexSelector();
                }
                false
            }
            kPseudoPart => self.part_names_.as_ref().is_some_and(|parts| {
                selector
                    .IdentList()
                    .iter()
                    .all(|name| b.PartNamesContain(parts, name))
            }),
            kPseudoFileSelectorButton => self
                .MatchesUAShadowElement(e, &AtomicString::from_str("-webkit-file-upload-button")),
            kPseudoPicker => {
                *selector.Argument() == AtomicString::from_str("select")
                    && self.MatchesUAShadowElement(e, &AtomicString::from_str("picker(select)"))
            }
            kPseudoSelectListbox => {
                self.MatchesUAShadowElement(e, &AtomicString::from_str("select-listbox"))
            }
            kPseudoPlaceholder => {
                self.MatchesUAShadowElement(e, &AtomicString::from_str("-webkit-input-placeholder"))
            }
            kPseudoDetailsContent => {
                self.MatchesUAShadowElement(e, &AtomicString::from_str("details-content"))
            }
            kPseudoPermissionIcon => {
                self.MatchesUAShadowElement(e, &AtomicString::from_str("permission-icon"))
            }
            kPseudoWebKitCustomElement | kPseudoBlinkInternalElement => {
                self.MatchesUAShadowElement(e, &selector.Value())
            }
            kPseudoSlotted => {
                let list = selector.SelectorList().expect("slotted selector list");
                let mut sub = c.Rebind(list.First());
                sub.is_sub_selector = true;
                sub.scope = None;
                sub.tree_scope = None;
                sub.pseudo_id = PseudoId::kPseudoIdNone;
                sub.pseudo_element = None;
                let mut result = SubResult::new(r);
                self.MatchSelector(&sub, &mut result.result) == MatchStatus::kSelectorMatches
            }
            kPseudoHighlight => {
                r.dynamic_pseudo = PseudoId::kPseudoIdHighlight;
                if self.pseudo_argument_.IsNull() || self.pseudo_argument_ == *selector.Argument() {
                    r.custom_highlight_name = Some(selector.Argument().clone());
                    true
                } else {
                    false
                }
            }
            kPseudoViewTransition
            | kPseudoViewTransitionGroup
            | kPseudoViewTransitionGroupChildren
            | kPseudoViewTransitionImagePair
            | kPseudoViewTransitionOld
            | kPseudoViewTransitionNew => {
                if c.pseudo_id == PseudoId::kPseudoIdNone {
                    if let Some(transition) = b.TransitionForElement(e) {
                        if std::ptr::eq(b.TransitionScope(&transition).as_ref(), e) {
                            r.dynamic_pseudo = id;
                            return true;
                        }
                    }
                }
                let check = if b.IsPseudoElement(e) {
                    b.ElementPseudoId(e)
                } else {
                    c.pseudo_id
                };
                if id != check {
                    return false;
                }
                r.dynamic_pseudo = c.pseudo_id;
                if id == PseudoId::kPseudoIdViewTransition {
                    return true;
                }
                let names = selector.IdentList();
                assert!(!names.is_empty());
                let argument = if b.IsPseudoElement(e) {
                    b.ElementPseudoArgument(e)
                } else {
                    self.pseudo_argument_.clone()
                };
                if names[0] != AtomicString::from_str("*") && names[0] != argument {
                    return false;
                }
                let classes = if b.IsPseudoElement(e) {
                    b.ElementViewTransitionClasses(e)
                } else {
                    self.pseudo_ident_list_.clone()
                };
                names[1..].iter().all(|name| classes.contains(name))
            }
            kPseudoScrollbarButton
            | kPseudoScrollbarCorner
            | kPseudoScrollbarThumb
            | kPseudoScrollbarTrack
            | kPseudoScrollbarTrackPiece => {
                if id != c.pseudo_id {
                    return false;
                }
                r.dynamic_pseudo = c.pseudo_id;
                true
            }
            kPseudoOverscrollAreaParent => b.ElementPseudoIdForStyling(e) == id,
            kPseudoScrollButton => self.MatchScrollButton(e, c, r),
            _ => {
                if b.CSSLogicalCombinationPseudoEnabled() {
                    return matches!(
                        p,
                        kPseudoBefore | kPseudoAfter | kPseudoMarker | kPseudoBackdrop
                    ) && b.ElementPseudoIdForStyling(e) == id;
                }
                r.dynamic_pseudo = id;
                if let Some(pseudo) = c.pseudo_element.as_ref() {
                    if r.pseudo_ancestor_index == c.pseudo_element_ancestors.len() - 1
                        && Rc::ptr_eq(pseudo, &element)
                    {
                        r.dynamic_pseudo = PseudoId::kPseudoIdNone;
                    }
                    if r.pseudo_ancestor_index == c.pseudo_element_ancestors.len() {
                        return true;
                    }
                    return b.ElementPseudoIdForStyling(e) == id;
                }
                c.previously_matched_pseudo_element == PseudoId::kPseudoIdNone
            }
        }
    }
}

// cpp: selector_checker.cc:1558-2285. Cache owners are external typed objects;
// SelectorChecker retains the candidate traversal and cache propagation logic.
struct HasCheckingScopeGuard<'a, B: SelectorCheckerBackend> {
    backend: &'a B,
    scope: Option<B::HasCacheScope>,
}
impl<B: SelectorCheckerBackend> Drop for HasCheckingScopeGuard<'_, B> {
    fn drop(&mut self) {
        if let Some(scope) = self.scope.take() {
            self.backend.ExitHasCacheScope(scope);
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum EarlyBreakOnHasArgumentChecking {
    Matched,
    NextArgument,
    NoBreak,
}
impl<B: SelectorCheckerBackend> SelectorChecker<B> {
    fn SetAffectedByHasFlagsForElementAtDepth(
        &self,
        a: &B::HasArgumentContext,
        e: &B::Element,
        depth: i32,
    ) {
        if depth > 0 {
            self.backend.SetElementFlag(
                e,
                ElementInvalidationFlag::AncestorsOrAncestorSiblingsAffectedByHas,
            );
        } else {
            self.backend
                .SetSiblingsAffectedByHasFlags(e, self.backend.HasSiblingsAffectedFlags(a));
        }
    }
    fn SetAffectedByHasFlagsForHasAnchorElement(&self, a: &B::HasArgumentContext, e: &B::Element) {
        use RelationType::*;
        match self.backend.HasLeftmostRelation(a) {
            kRelativeChild | kRelativeDescendant => self.backend.SetElementFlag(
                e,
                ElementInvalidationFlag::AncestorsOrAncestorSiblingsAffectedByHas,
            ),
            kRelativeDirectAdjacent | kRelativeIndirectAdjacent => self
                .backend
                .SetSiblingsAffectedByHasFlags(e, self.backend.HasSiblingsAffectedFlags(a)),
            _ => unreachable!("has relation"),
        }
    }
    fn SetAffectedByHasFlagsForHasAnchorSiblings(&self, a: &B::HasArgumentContext, e: &B::Element) {
        let limit = self.backend.HasAdjacentDistanceLimit(a);
        if limit == 0 {
            return;
        }
        let mut distance = 1;
        let mut sibling = self.backend.NextSibling(e);
        while let Some(element) = sibling {
            if distance > limit {
                break;
            }
            self.backend
                .SetSiblingsAffectedByHasFlags(&element, self.backend.HasSiblingsAffectedFlags(a));
            sibling = self.backend.NextSibling(&element);
            distance += 1;
        }
    }
    fn SetAffectedByHasForArgumentMatchedElement(
        &self,
        a: &B::HasArgumentContext,
        anchor: &B::Element,
        matched: Rc<B::Element>,
        matched_depth: i32,
    ) {
        let mut element = matched;
        let mut depth = matched_depth;
        loop {
            if depth == 0 {
                element = self
                    .backend
                    .PreviousSibling(&element)
                    .expect("has affected iterator previous sibling");
            } else {
                let needs = if depth == matched_depth {
                    self.backend.HasSiblingCombinatorAtRightmost(a)
                } else {
                    self.backend.HasSiblingCombinatorBetweenChildOrDescendant(a)
                };
                if let Some(previous) = needs
                    .then(|| self.backend.PreviousSibling(&element))
                    .flatten()
                {
                    element = previous;
                } else {
                    depth -= 1;
                    element = self
                        .backend
                        .ParentOrShadowHostElement(&element)
                        .expect("has affected iterator parent");
                }
            }
            if std::ptr::eq(element.as_ref(), anchor) {
                break;
            }
            self.SetAffectedByHasFlagsForElementAtDepth(a, &element, depth);
        }
    }
    fn SetHasAnchorElementAsCheckedAndGetOldResult(
        &self,
        c: &SelectorCheckingContext<'_, B>,
        cache: &B::HasCacheContext,
    ) -> u8 {
        let b = &self.backend;
        let e = &c.element;
        let previous = b.HasCacheGetResult(cache, e);
        if previous & 1 != 0 {
            return previous;
        }
        let subject = c.in_rightmost_compound
            && c.scope
                .as_ref()
                .and_then(|n| b.ContainerAsElement(n))
                .is_some_and(|root| Rc::ptr_eq(&root, e));
        if !subject && b.HasCacheAlreadyChecked(cache, e) {
            if previous != 0 {
                b.HasCacheSetChecked(cache, e);
            }
            return previous | 1;
        }
        b.HasCacheSetChecked(cache, e);
        previous
    }
    fn CacheMatchedElementsAndReturnMatchedResult(
        &self,
        relation: RelationType,
        anchor: &B::Element,
        matches: &[Rc<B::Element>],
        cache: &B::HasCacheContext,
    ) -> bool {
        use RelationType::*;
        let b = &self.backend;
        let (parent, indirect) = match relation {
            kRelativeDescendant => (true, true),
            kRelativeChild => (true, false),
            kRelativeDirectAdjacent => (false, false),
            kRelativeIndirectAdjacent => (false, true),
            _ => unreachable!("relative has relation"),
        };
        let next = |e: &B::Element| {
            if parent {
                b.ParentOrShadowHostElement(e)
            } else {
                b.PreviousSibling(e)
            }
        };
        let caching = b.HasCacheAllowed(cache);
        let mut matched = false;
        for leftmost in matches {
            let mut current = next(leftmost);
            while let Some(element) = current {
                if std::ptr::eq(element.as_ref(), anchor) {
                    matched = true;
                    if !caching {
                        return true;
                    }
                }
                if caching {
                    let old = b.HasCacheSetMatched(cache, &element);
                    if indirect && old != 0 && old & 2 != 0 {
                        break;
                    }
                }
                if !indirect {
                    break;
                }
                current = next(&element);
            }
        }
        matched
    }
    fn CheckEarlyBreakForHasArgument(
        &self,
        c: &SelectorCheckingContext<'_, B>,
        a: &B::HasArgumentContext,
        cache: &B::HasCacheContext,
        update: &mut bool,
    ) -> EarlyBreakOnHasArgumentChecking {
        use EarlyBreakOnHasArgumentChecking::*;
        let b = &self.backend;
        if !b.HasCacheAllowed(cache) {
            return NoBreak;
        }
        let old = self.SetHasAnchorElementAsCheckedAndGetOldResult(c, cache);
        if old & 1 != 0 {
            if *update {
                self.SetAffectedByHasFlagsForHasAnchorSiblings(a, &c.element);
            }
            return if old & 2 != 0 { Matched } else { NextArgument };
        }
        let (filter, new) = b.HasCacheEnsureFastRejectFilter(cache, &c.element);
        if new && !b.AffectedByMultipleHas(&c.element) {
            return NoBreak;
        }
        if !b.HasBloomAllocated(&filter) {
            if *update {
                b.SetElementFlag(&c.element, ElementInvalidationFlag::AffectedByMultipleHas);
            }
            b.HasAllocateBloom(&filter);
            let mut iterator = b.CreateHasTraversal(&c.element, a);
            while let Some(element) = b.HasTraversalCurrent(&iterator) {
                b.HasAddElementIdentifierHashes(&filter, &element);
                if *update {
                    self.SetAffectedByHasFlagsForElementAtDepth(
                        a,
                        &element,
                        b.HasTraversalDepth(&iterator),
                    );
                }
                b.HasTraversalNext(&mut iterator);
            }
        }
        *update = false;
        if b.HasFastReject(&filter, a) {
            let mut last = Some(c.element.clone());
            let mut depth = 0;
            if b.HasAdjacentDistanceLimit(a) > 0 {
                last = last.and_then(|e| b.NextSibling(&e));
            }
            if b.HasDepthLimit(a) > 0 {
                last = last.and_then(|e| b.FirstChildElement(&e));
                depth = 1;
            }
            if let Some(last) = last {
                b.HasCacheSetAllTraversedChecked(cache, &last, depth);
            }
            return NextArgument;
        }
        NoBreak
    }
    fn CheckPseudoHas(&self, c: &SelectorCheckingContext<'_, B>, r: &mut MatchResult<B>) -> bool {
        let b = &self.backend;
        let e = &c.element;
        let selector = self.Selector(c);
        if self.mode_ == Mode::kResolvingStyle {
            if Self::ImpactsSubject(c) {
                b.SetElementFlag(e, ElementInvalidationFlag::AffectedBySubjectHas);
            }
            if Self::ImpactsNonSubject(c) {
                b.SetElementFlag(e, ElementInvalidationFlag::AffectedByNonSubjectHas);
            }
            if selector.ContainsPseudoInsideHasPseudoClass() {
                b.SetElementFlag(e, ElementInvalidationFlag::AffectedByPseudoInHas);
            }
            if selector.ContainsComplexLogicalCombinationsInsideHasPseudoClass() {
                b.SetElementFlag(
                    e,
                    ElementInvalidationFlag::AffectedByLogicalCombinationsInHas,
                );
            }
        }
        if b.InPseudoHasChecking(e) {
            return false;
        }
        let _scope = HasCheckingScopeGuard {
            backend: b.as_ref(),
            scope: Some(b.EnterHasCacheScope(e, true)),
        };
        let shadow = selector.HasArgumentMatchInShadowTree();
        if shadow && !b.HasShadowRoot(e) {
            return false;
        }
        let list = selector.SelectorList().expect(":has selector list");
        let mut sub = SelectorCheckingContext::new(e.clone());
        sub.tree_scope = c.tree_scope.clone();
        sub.scope = c.scope.clone();
        sub.is_inside_has_pseudo_class = true;
        sub.pseudo_has_in_rightmost_compound = c.in_rightmost_compound;
        let mut update = self.mode_ == Mode::kResolvingStyle;
        let mut cursor = list.First();
        while let Some(selector) = cursor {
            cursor = selector.NextComplexSelector();
            let a = b.CreateHasArgumentContext(selector, c.scope.as_deref(), shadow);
            if b.HasInvalidShadowTraversal(&a) {
                continue;
            }
            let relation = b.HasLeftmostRelation(&a);
            let cache = b.CreateHasCacheContext(e, &a);
            if b.HasAdjacentDistanceLimit(&a) > 0 && b.HasAdjacentDistanceFixed(&a) {
                if let Some(parent) = b.ParentElementOrShadowRoot(e) {
                    b.SetChildrenAffectedByDirectAdjacentRules(&parent);
                }
            }
            if update {
                self.SetAffectedByHasFlagsForHasAnchorElement(&a, e);
            }
            match self.CheckEarlyBreakForHasArgument(c, &a, &cache, &mut update) {
                EarlyBreakOnHasArgumentChecking::Matched => return true,
                EarlyBreakOnHasArgumentChecking::NextArgument => continue,
                EarlyBreakOnHasArgumentChecking::NoBreak => {}
            }
            sub.selector = Some(selector);
            sub.relative_anchor_element = Some(b.ElementAsContainer(e));
            let mut matched = false;
            let mut last = None;
            let mut last_depth = -1;
            let mut iterator = b.CreateHasTraversal(e, &a);
            while let Some(element) = b.HasTraversalCurrent(&iterator) {
                let depth = b.HasTraversalDepth(&iterator);
                if update {
                    self.SetAffectedByHasFlagsForElementAtDepth(&a, &element, depth);
                }
                if (b.HasDepthLimit(&a) > 0 && depth == 0)
                    || (b.HasDepthFixed(&a) && depth != b.HasDepthLimit(&a))
                {
                    b.HasTraversalNext(&mut iterator);
                    continue;
                }
                sub.element = element.clone();
                let matches = Rc::new(RefCell::new(Vec::new()));
                let mut result = SubResult::new(r);
                result.result.has_argument_leftmost_compound_matches = Some(matches.clone());
                self.MatchSelector(&sub, &mut result.result);
                last = Some(element);
                last_depth = depth;
                matched = self.CacheMatchedElementsAndReturnMatchedResult(
                    relation,
                    e,
                    &matches.borrow(),
                    &cache,
                );
                if matched {
                    break;
                }
                b.HasTraversalNext(&mut iterator);
            }
            if b.HasCacheAllowed(&cache) {
                if let Some(last) = last.as_ref() {
                    b.HasCacheSetAllTraversedChecked(&cache, last, last_depth);
                }
            }
            if !matched {
                continue;
            }
            if update {
                self.SetAffectedByHasForArgumentMatchedElement(
                    &a,
                    e,
                    last.expect("has matched candidate"),
                    last_depth,
                );
            }
            return true;
        }
        false
    }
    fn EnsureActivations(
        &self,
        c: &SelectorCheckingContext<'_, B>,
        scope: &B::StyleScope,
    ) -> Rc<B::StyleScopeActivations> {
        let b = &self.backend;
        let outer = if let Some(parent) = b.StyleScopeParent(scope) {
            self.EnsureActivations(c, &parent)
        } else {
            b.CreateActivations(vec![b.CreateActivation(c.scope.clone(), u32::MAX)], 0)
        };
        let visited = c.match_visited || c.had_match_visited;
        let frame = c.style_scope_frame.as_ref().expect("scope frame");
        self.CalculateActivations(
            c.tree_scope.clone(),
            b.StyleScopeFrameElement(frame),
            scope,
            &outer,
            if visited { None } else { Some(frame.clone()) },
            visited,
        )
    }
    fn CalculateActivations(
        &self,
        tree: Option<Rc<B::TreeScope>>,
        element: Rc<B::Element>,
        scope: &B::StyleScope,
        outer: &B::StyleScopeActivations,
        frame: Option<Rc<B::StyleScopeFrame>>,
        visited: bool,
    ) -> Rc<B::StyleScopeActivations> {
        let b = &self.backend;
        if let Some(frame) = frame.as_ref() {
            if let Some(cached) = b.InsertScopeActivationCacheEntry(frame, scope) {
                return cached;
            }
        }
        let mut vector = Vec::new();
        let mut flags = 0;
        let outer_vector = b.ActivationVector(outer);
        if let Some(first) = outer_vector.first() {
            let ceiling = b.ActivationRoot(first).and_then(|root| {
                b.ContainerAsElement(&root)
                    .or_else(|| b.ContainerShadowHost(&root))
            });
            if !ceiling.is_some_and(|e| Rc::ptr_eq(&e, &element)) {
                if let Some(parent) = b.ParentOrShadowHostElement(&element) {
                    let parent_frame = frame
                        .as_ref()
                        .and_then(|f| b.ParentStyleScopeFrame(f, &parent));
                    let parent_activations = self.CalculateActivations(
                        tree.clone(),
                        parent,
                        scope,
                        outer,
                        parent_frame,
                        visited && !b.IsLink(&element),
                    );
                    flags = b.ActivationMatchFlags(&parent_activations);
                    for activation in b.ActivationVector(&parent_activations) {
                        if !self.ElementIsScopingLimit(
                            tree.clone(),
                            scope,
                            &activation,
                            element.clone(),
                            visited,
                            &mut flags,
                        ) {
                            vector.push(b.CreateActivation(
                                b.ActivationRoot(&activation),
                                b.ActivationProximity(&activation).wrapping_add(1),
                            ));
                        }
                    }
                }
            }
            for outer_activation in outer_vector {
                let from = b.StyleScopeFrom(scope);
                let root_matches = if let Some(from) = from {
                    self.MatchesWithScope(
                        element.clone(),
                        &from,
                        tree.clone(),
                        b.ActivationRoot(&outer_activation),
                        visited,
                        &mut flags,
                    )
                } else {
                    b.ElementTriggersScope(&element, scope)
                };
                if root_matches {
                    let activation = b.CreateActivation(Some(b.ElementAsContainer(&element)), 0);
                    if !self.ElementIsScopingLimit(
                        tree.clone(),
                        scope,
                        &activation,
                        element.clone(),
                        visited,
                        &mut flags,
                    ) {
                        vector.push(activation);
                    }
                    break;
                }
            }
        }
        let activations = b.CreateActivations(vector, flags);
        if let Some(frame) = frame {
            b.SetScopeActivationCacheEntry(&frame, scope, activations.clone());
        }
        activations
    }
    fn MatchesWithScope(
        &self,
        e: Rc<B::Element>,
        list: &CSSSelectorList,
        tree: Option<Rc<B::TreeScope>>,
        scope: Option<Rc<B::ContainerNode>>,
        visited: bool,
        flags: &mut MatchFlags,
    ) -> bool {
        let mut c = SelectorCheckingContext::new(e);
        c.tree_scope = tree;
        c.scope = scope;
        c.match_visited = visited;
        c.impact = Impact::kBoth;
        c.selector = list.First();
        while let Some(selector) = c.selector {
            let mut r = MatchResult::default();
            let matched = self.MatchSelector(&c, &mut r) == MatchStatus::kSelectorMatches;
            *flags |= r.flags;
            if matched {
                return true;
            }
            c.selector = selector.NextComplexSelector();
        }
        false
    }
    fn ElementIsScopingLimit(
        &self,
        tree: Option<Rc<B::TreeScope>>,
        scope: &B::StyleScope,
        activation: &B::StyleScopeActivation,
        e: Rc<B::Element>,
        visited: bool,
        flags: &mut MatchFlags,
    ) -> bool {
        self.backend.StyleScopeTo(scope).is_some_and(|to| {
            self.MatchesWithScope(
                e,
                &to,
                tree,
                self.backend.ActivationRoot(activation),
                visited,
                flags,
            )
        })
    }
}

// cpp: selector_checker.cc:2288-3238. Native DOM predicates below read state;
// pseudo dispatch, forced-state precedence and invalidation remain Rust.
impl<B: SelectorCheckerBackend> SelectorChecker<B> {
    fn ForcedStatePair(&self, e: &B::Element, p: PseudoType, opposite: PseudoType) -> Option<bool> {
        if self.backend.ProbeForcePseudoState(e, p) {
            Some(true)
        } else if self.backend.ProbeForcePseudoState(e, opposite) {
            Some(false)
        } else {
            None
        }
    }
    fn MarkPositionalParent(&self, e: &B::Element, flag: ContainerInvalidationFlag) {
        if self.mode_ == Mode::kResolvingStyle {
            if let Some(parent) = self.backend.ParentElementOrDocumentFragment(e) {
                self.backend.SetContainerFlag(&parent, flag);
            }
        }
    }
    fn PositionalChildrenReady(&self, parent: Option<&B::ContainerNode>) -> bool {
        self.mode_ == Mode::kQueryingRules
            || parent.is_none_or(|p| self.backend.IsFinishedParsingChildren(p))
    }
    fn MarkInteractiveImpact(
        &self,
        c: &SelectorCheckingContext<'_, B>,
        e: &B::Element,
        has: ElementInvalidationFlag,
        related: ElementInvalidationFlag,
    ) {
        if self.mode_ == Mode::kResolvingStyle {
            if c.is_inside_has_pseudo_class {
                self.backend.SetElementFlag(e, has);
            } else if Self::ImpactsNonSubject(c) {
                self.backend.SetElementFlag(e, related);
            }
        }
    }
    fn CheckPseudoClass(&self, c: &SelectorCheckingContext<'_, B>, r: &mut MatchResult<B>) -> bool {
        use ContainerInvalidationFlag as N;
        use ElementClass as C;
        use ElementInvalidationFlag as E;
        use PseudoType::*;
        let b = &self.backend;
        let selector = self.Selector(c);
        let p = selector.GetPseudoType();
        let element = self.GetCandidateElement(c, r);
        let e = element.as_ref();
        if c.has_scrollbar_pseudo {
            return self.CheckScrollbarPseudoClass(c, r);
        }
        match p {
            kPseudoNot => self.CheckPseudoNot(c, r),
            kPseudoEmpty => {
                let mut empty = true;
                let mut whitespace = false;
                let mut node = b.FirstChildNode(e);
                while let Some(n) = node {
                    if b.NodeIsElement(&n) {
                        empty = false;
                        break;
                    }
                    if let Some(data) = b.TextNodeData(&n) {
                        if !data.empty() {
                            if data
                                .utf16_units()
                                .unwrap()
                                .iter()
                                .all(|u| matches!(*u, 9 | 10 | 12 | 13 | 32))
                            {
                                whitespace = true;
                            } else {
                                empty = false;
                                break;
                            }
                        }
                    }
                    node = b.NextSiblingNode(&n);
                }
                if empty && whitespace {
                    empty = false;
                }
                if self.mode_ == Mode::kResolvingStyle {
                    b.SetElementFlag(e, E::StyleAffectedByEmpty);
                }
                empty
            }
            kPseudoAnimatedImage => {
                assert!(b.FeatureEnabled(e, SelectorRuntimeFeature::CSSImageAnimation));
                b.ElementIs(e, C::Image) && b.CachedImageIsAnimated(e)
            }
            kPseudoFirstChild => {
                self.MarkPositionalParent(e, N::ChildrenAffectedByFirstChildRules);
                if self.mode_ == Mode::kResolvingStyle {
                    b.SetElementFlag(e, E::AffectedByFirstChildRules);
                }
                self.IsFirstChild(e)
            }
            kPseudoFirstOfType => {
                self.MarkPositionalParent(e, N::ChildrenAffectedByForwardPositionalRules);
                self.IsFirstOfType(e, &b.ElementTagQName(e))
            }
            kPseudoLastChild => {
                let parent = b.ParentElementOrDocumentFragment(e);
                self.MarkPositionalParent(e, N::ChildrenAffectedByLastChildRules);
                if self.mode_ == Mode::kResolvingStyle {
                    b.SetElementFlag(e, E::AffectedByLastChildRules);
                }
                self.PositionalChildrenReady(parent.as_deref()) && self.IsLastChild(e)
            }
            kPseudoLastOfType => {
                let parent = b.ParentElementOrDocumentFragment(e);
                self.MarkPositionalParent(e, N::ChildrenAffectedByBackwardPositionalRules);
                self.PositionalChildrenReady(parent.as_deref())
                    && self.IsLastOfType(e, &b.ElementTagQName(e))
            }
            kPseudoOnlyChild => {
                let id = if b.IsPseudoElement(e) {
                    b.ElementPseudoId(e)
                } else {
                    c.pseudo_id
                };
                if layoutng_style::style::computed_style_constants::IsTransitionPseudoElement(id) {
                    let Some(t) = b.TransitionForElement(e) else {
                        return false;
                    };
                    let argument = if b.IsPseudoElement(e) {
                        b.ElementPseudoArgument(e)
                    } else {
                        c.pseudo_argument
                            .clone()
                            .expect("transition pseudo argument")
                    };
                    return b.TransitionMatchOnlyChild(&t, id, &argument);
                }
                self.MarkPositionalParent(e, N::ChildrenAffectedByFirstChildRules);
                self.MarkPositionalParent(e, N::ChildrenAffectedByLastChildRules);
                if self.mode_ == Mode::kResolvingStyle {
                    b.SetElementFlag(e, E::AffectedByFirstChildRules);
                    b.SetElementFlag(e, E::AffectedByLastChildRules);
                }
                self.PositionalChildrenReady(b.ParentElementOrDocumentFragment(e).as_deref())
                    && self.IsFirstChild(e)
                    && self.IsLastChild(e)
            }
            kPseudoOnlyOfType => {
                self.MarkPositionalParent(e, N::ChildrenAffectedByForwardPositionalRules);
                self.MarkPositionalParent(e, N::ChildrenAffectedByBackwardPositionalRules);
                self.PositionalChildrenReady(b.ParentElementOrDocumentFragment(e).as_deref())
                    && self.IsFirstOfType(e, &b.ElementTagQName(e))
                    && self.IsLastOfType(e, &b.ElementTagQName(e))
            }
            kPseudoNthChild | kPseudoNthOfType | kPseudoNthLastChild | kPseudoNthLastOfType => {
                let backward = matches!(p, kPseudoNthLastChild | kPseudoNthLastOfType);
                self.MarkPositionalParent(
                    e,
                    if backward {
                        N::ChildrenAffectedByBackwardPositionalRules
                    } else {
                        N::ChildrenAffectedByForwardPositionalRules
                    },
                );
                if backward
                    && !self
                        .PositionalChildrenReady(b.ParentElementOrDocumentFragment(e).as_deref())
                {
                    return false;
                }
                let list = selector.SelectorList();
                if matches!(p, kPseudoNthChild | kPseudoNthLastChild) {
                    if let Some(list) = list.as_ref() {
                        if !self.MatchesAnyInList(c, list, r) {
                            return false;
                        }
                    }
                }
                selector.MatchNth(b.NthIndex(e, p, list.as_deref(), self, c))
            }
            kPseudoSelectContainsInput => b.ElementIs(e, C::Select) && b.NumDescendantInputs(e) > 0,
            kPseudoSelectHasSlottedButton => {
                b.ElementIs(e, C::Select) && b.SlottedButton(e).is_some()
            }
            kPseudoTarget => {
                b.ProbeForcePseudoState(e, p)
                    || b.CssTarget(e)
                        .is_some_and(|target| Rc::ptr_eq(&target, &element))
                    || self.MatchesExternalSVGUseTarget(e)
            }
            kPseudoIs | kPseudoWhere | kPseudoAny => selector
                .SelectorListOrParent()
                .is_some_and(|list| self.MatchesAnyInList(c, &list, r)),
            kPseudoParent => {
                if let Some(list) = selector.SelectorListOrParent() {
                    self.MatchesAnyInList(c, &list, r)
                } else {
                    self.CheckPseudoScope(c, r)
                }
            }
            kPseudoAutofill
            | kPseudoWebKitAutofill
            | kPseudoAutofillPreviewed
            | kPseudoAutofillSelected => self.CheckPseudoAutofill(p, e),
            kPseudoAnyLink | kPseudoWebkitAnyLink => b.IsLink(e),
            kPseudoLink => b.IsLink(e) && !c.match_visited,
            kPseudoVisited => b.IsLink(e) && c.match_visited,
            kPseudoDrag => {
                if self.mode_ == Mode::kResolvingStyle && Self::ImpactsNonSubject(c) {
                    b.SetElementFlag(e, E::ChildrenOrSiblingsAffectedByDrag);
                }
                if Self::ImpactsSubject(c) {
                    r.SetFlag(MatchFlag::kAffectedByDrag);
                }
                b.IsDragged(e)
            }
            kPseudoFocus => {
                self.MarkInteractiveImpact(
                    c,
                    e,
                    E::AncestorsOrSiblingsAffectedByFocusInHas,
                    E::ChildrenOrSiblingsAffectedByFocus,
                );
                self.MatchesFocusPseudoClass(e, c.previously_matched_pseudo_element)
            }
            kPseudoFocusVisible => {
                self.MarkInteractiveImpact(
                    c,
                    e,
                    E::AncestorsOrSiblingsAffectedByFocusVisibleInHas,
                    E::ChildrenOrSiblingsAffectedByFocusVisible,
                );
                self.MatchesFocusVisiblePseudoClass(e)
            }
            kPseudoFocusWithin => {
                self.MarkInteractiveImpact(
                    c,
                    e,
                    E::AncestorsOrSiblingsAffectedByFocusInHas,
                    E::ChildrenOrSiblingsAffectedByFocusWithin,
                );
                if Self::ImpactsSubject(c) {
                    r.SetFlag(MatchFlag::kAffectedByFocusWithin);
                }
                b.ProbeForcePseudoState(e, p) || b.HasFocusWithin(e)
            }
            kPseudoActiveOption => {
                if !b.ElementIs(e, C::Option) {
                    return false;
                }
                if b.FeatureEnabled(e, SelectorRuntimeFeature::CustomizableCombobox) {
                    if let Some(owner) = b.OwnerDataListElement(e) {
                        return b
                            .ActiveOption(&owner)
                            .is_some_and(|option| Rc::ptr_eq(&option, &element));
                    }
                }
                if b.FeatureEnabled(e, SelectorRuntimeFeature::FilterableSelect) {
                    if let Some(owner) = b.OwnerSelectElement(e) {
                        return b
                            .ActiveOption(&owner)
                            .is_some_and(|option| Rc::ptr_eq(&option, &element));
                    }
                }
                false
            }
            kPseudoFiltered => {
                assert!(
                    b.FeatureEnabled(e, SelectorRuntimeFeature::CustomizableCombobox)
                        || b.FeatureEnabled(e, SelectorRuntimeFeature::FilterableSelect)
                );
                b.ElementIs(e, C::Option) && b.IsFiltered(e)
            }
            kPseudoInterestSource => b.HasInterest(e),
            kPseudoInterestTarget => b.InterestInvoker(e).is_some(),
            kPseudoHasSlotted => b.ElementIs(e, C::Slot) && b.HasFlattenedAssignedNodes(e),
            kPseudoHover | kPseudoActive => {
                self.MarkInteractiveImpact(
                    c,
                    e,
                    if p == kPseudoHover {
                        E::AncestorsOrSiblingsAffectedByHoverInHas
                    } else {
                        E::AncestorsOrSiblingsAffectedByActiveInHas
                    },
                    if p == kPseudoHover {
                        E::ChildrenOrSiblingsAffectedByHover
                    } else {
                        E::ChildrenOrSiblingsAffectedByActive
                    },
                );
                if Self::ImpactsSubject(c) {
                    r.SetFlag(if p == kPseudoHover {
                        MatchFlag::kAffectedByHover
                    } else {
                        MatchFlag::kAffectedByActive
                    });
                }
                if !self.ShouldMatchHoverOrActive(c) {
                    return false;
                }
                b.ProbeForcePseudoState(e, p)
                    || if p == kPseudoHover {
                        b.IsHovered(e)
                    } else {
                        b.IsActive(e)
                    }
            }
            kPseudoEnabled => self
                .ForcedStatePair(e, p, kPseudoDisabled)
                .unwrap_or_else(|| b.MatchesEnabled(e)),
            kPseudoDisabled => self
                .ForcedStatePair(e, p, kPseudoEnabled)
                .unwrap_or_else(|| b.MatchesDisabled(e)),
            kPseudoReadOnly => self
                .ForcedStatePair(e, p, kPseudoReadWrite)
                .unwrap_or_else(|| b.MatchesReadOnly(e)),
            kPseudoReadWrite => self
                .ForcedStatePair(e, p, kPseudoReadOnly)
                .unwrap_or_else(|| b.MatchesReadWrite(e)),
            kPseudoOptional => self
                .ForcedStatePair(e, p, kPseudoRequired)
                .unwrap_or_else(|| b.IsOptionalFormControl(e)),
            kPseudoRequired => self
                .ForcedStatePair(e, p, kPseudoOptional)
                .unwrap_or_else(|| b.IsRequiredFormControl(e)),
            kPseudoUserInvalid => {
                self.ForcedStatePair(e, p, kPseudoUserValid)
                    .unwrap_or_else(|| {
                        b.ElementIs(e, C::FormControlWithState) && b.MatchesUserInvalid(e)
                    })
            }
            kPseudoUserValid => self
                .ForcedStatePair(e, p, kPseudoUserInvalid)
                .unwrap_or_else(|| {
                    b.ElementIs(e, C::FormControlWithState) && b.MatchesUserValid(e)
                }),
            kPseudoValid => self
                .ForcedStatePair(e, p, kPseudoInvalid)
                .unwrap_or_else(|| b.MatchesValidity(e) && b.IsValid(e)),
            kPseudoInvalid => self
                .ForcedStatePair(e, p, kPseudoValid)
                .unwrap_or_else(|| b.MatchesValidity(e) && !b.IsValid(e)),
            kPseudoInRange => self
                .ForcedStatePair(e, p, kPseudoOutOfRange)
                .unwrap_or_else(|| b.IsInRange(e)),
            kPseudoOutOfRange => self
                .ForcedStatePair(e, p, kPseudoInRange)
                .unwrap_or_else(|| b.IsOutOfRange(e)),

            kPseudoChecked => {
                if b.ProbeForcePseudoState(e, p) {
                    return true;
                }
                if b.ElementIs(e, C::Input) {
                    b.ShouldAppearChecked(e) && !b.ShouldAppearIndeterminate(e)
                } else if b.ElementIs(e, C::Option) {
                    b.OptionSelected(e)
                } else {
                    b.ElementIs(e, C::MenuItem) && b.ShouldAppearChecked(e)
                }
            }
            kPseudoTargetCurrent => {
                if b.ProbeForcePseudoState(e, p) {
                    return true;
                }
                if b.ElementIs(e, C::ScrollMarker) {
                    b.ScrollMarkerIsSelected(e)
                } else {
                    b.ElementIs(e, C::Anchor)
                        && b.HasScrollMarkerGroupData(e)
                        && b.ScrollMarkerSelected(e)
                            .is_some_and(|selected| Rc::ptr_eq(&selected, &element))
                }
            }
            kPseudoTargetBefore | kPseudoTargetAfter => {
                let mut marker = None;
                let mut selected = None;
                if b.ElementIs(e, C::ScrollMarker) && b.HasScrollMarkerGroupData(e) {
                    marker = Some(element.clone());
                    selected = b.ScrollMarkerSelected(e);
                }
                if b.ElementIs(e, C::Anchor) && b.HasScrollMarkerGroupData(e) {
                    marker = Some(element.clone());
                    selected = b.ScrollMarkerSelected(e);
                }
                if let (Some(marker), Some(selected)) = (marker, selected) {
                    let order = b.CompareLayoutPreorder(&marker, &selected);
                    order == if p == kPseudoTargetBefore { -1 } else { 1 }
                } else {
                    false
                }
            }
            kPseudoIndeterminate => b.ProbeForcePseudoState(e, p) || b.ShouldAppearIndeterminate(e),
            kPseudoRoot => b
                .DocumentElement(e)
                .is_some_and(|root| Rc::ptr_eq(&root, &element)),
            kPseudoLinkTo => self.CheckPseudoLinkTo(c, r),
            kPseudoNavigationSource => b
                .NavigationSourceElement(e)
                .is_some_and(|source| Rc::ptr_eq(&source, &element)),
            kPseudoLang => {
                let language = if b.ElementIs(e, C::VTT) {
                    b.VttLanguage(e)
                } else {
                    b.ComputeInheritedLanguage(e)
                };
                if !language.empty() && !IsValidBCP47Value(&language, false) {
                    return false;
                }
                let arguments = selector.ArgumentList().expect("lang arguments");
                if !b.FeatureEnabled(e, SelectorRuntimeFeature::CSSLangExtendedRanges) {
                    return !language.empty()
                        && AttributeValueMatches(
                            &language,
                            MatchType::kAttributeHyphen,
                            &arguments[0],
                            true,
                        );
                }
                MatchesLangPseudoClass(&language, &arguments)
            }
            kPseudoDir => {
                let argument = selector.Argument();
                let rtl = if ascii_equal(&argument, &AtomicString::from_str("ltr")) {
                    false
                } else if ascii_equal(&argument, &AtomicString::from_str("rtl")) {
                    true
                } else {
                    return false;
                };
                if self.mode_ == Mode::kQueryingRules && b.SlotAssignmentDirty(e) {
                    b.RecalcSlotAssignments(e);
                }
                b.CachedDirectionIsRtl(e) == rtl
            }
            kPseudoPopoverOpen => {
                b.ElementIs(e, C::HTMLElement) && b.IsPopover(e) && b.PopoverOpen(e)
            }
            kPseudoOpen => {
                if b.ProbeForcePseudoState(e, p) {
                    return true;
                }
                if b.ElementIs(e, C::Dialog) || b.ElementIs(e, C::Details) {
                    b.HasOpenAttribute(e)
                } else if b.ElementIs(e, C::Select) {
                    b.PopupIsVisible(e)
                } else if b.ElementIs(e, C::Input) {
                    b.IsPickerVisible(e)
                } else {
                    b.ElementIs(e, C::MenuItem) && b.IsSubmenuOpen(e)
                }
            }
            kPseudoMenulistPopoverWithMenubarAnchor | kPseudoMenulistPopoverWithMenulistAnchor => {
                if !b.ElementIs(e, C::MenuList) {
                    return false;
                }
                let Some(invoker) = b.PopoverInvoker(e) else {
                    return false;
                };
                if !b.ElementIs(&invoker, C::MenuItem) {
                    return false;
                }
                b.OwningMenuElement(&invoker).is_some_and(|owner| {
                    b.ElementIs(
                        &owner,
                        if p == kPseudoMenulistPopoverWithMenubarAnchor {
                            C::MenuBar
                        } else {
                            C::MenuList
                        },
                    )
                })
            }
            kPseudoFullscreen | kPseudoFullScreen => b.FullscreenFlag(e),
            kPseudoPermissionGranted => {
                assert!(
                    b.FeatureEnabled(e, SelectorRuntimeFeature::GeolocationElement)
                        || b.FeatureEnabled(e, SelectorRuntimeFeature::UserMediaElement)
                        || b.FeatureEnabled(e, SelectorRuntimeFeature::InstallElement)
                );
                b.ElementIs(e, C::Capability) && b.Granted(e)
            }
            kPseudoPlaying => b.ElementIs(e, C::Media) && !b.MediaPaused(e),
            kPseudoPaused => b.ElementIs(e, C::Media) && b.MediaPaused(e),
            kPseudoVolumeLocked => false,
            kPseudoXrOverlay => b.IsXrOverlay(e) && b.IsFullscreenElement(e),
            kPseudoFutureCue => b.ElementIs(e, C::VTT) && !b.VttIsPastNode(e),
            kPseudoPastCue => b.ElementIs(e, C::VTT) && b.VttIsPastNode(e),
            kPseudoScope => self.CheckPseudoScope(c, r),
            kPseudoHostContext | kPseudoHost => false,
            kPseudoSpatialNavigationFocus => {
                b.ElementIs(e, C::Option) && b.SpatialNavigationFocused(e) && self.IsFrameFocused(e)
            }
            kPseudoHasDatalist => b.ElementIs(e, C::Input) && b.HasDatalist(e),
            kPseudoHasOpenMenuitem => {
                b.ElementIs(e, C::MenuOwner)
                    && b.MenuOwnerItems(e).iter().any(|item| b.IsSubmenuOpen(item))
            }
            kPseudoIsHtml => b.IsInHTMLDocument(e),
            kPseudoListBox => b.ElementIs(e, C::Select) && !b.UsesMenuList(e),
            kPseudoMultiSelectFocus => {
                b.ElementIs(e, C::Option) && b.IsMultiSelectFocused(e) && self.IsFrameFocused(e)
            }
            kPseudoHostHasNonAutoAppearance => {
                if b.ContainingShadowRootIsUserAgent(e) != Some(true) {
                    return false;
                }
                let host = b.OwnerShadowHost(e).expect("containing shadow root host");
                b.GetComputedStyle(&host)
                    .is_some_and(|style| b.HasEffectiveAppearance(unsafe { &*style.Get() }))
            }
            kPseudoWindowInactive => {
                c.previously_matched_pseudo_element == PseudoId::kPseudoIdSelection
                    && !b.PageIsActive(e)
            }
            kPseudoState => b.DidAttachInternals(e) && b.InternalsHasState(e, &selector.Argument()),
            kPseudoHorizontal | kPseudoVertical | kPseudoDecrement | kPseudoIncrement
            | kPseudoStart | kPseudoEnd | kPseudoDoubleButton | kPseudoSingleButton
            | kPseudoNoButton | kPseudoCornerPresent => false,
            kPseudoModal => {
                b.IsFullscreenElement(e) || (b.ElementIs(e, C::Dialog) && b.DialogIsModal(e))
            }
            kPseudoHas => self.CheckPseudoHas(c, r),
            kPseudoRelativeAnchor => c
                .relative_anchor_element
                .as_ref()
                .and_then(|anchor| b.ContainerAsElement(anchor))
                .is_some_and(|anchor| Rc::ptr_eq(&anchor, &element)),
            kPseudoActiveViewTransition => self
                .GetTransitionForScope(e)
                .is_some_and(|transition| b.TransitionMatchActive(&transition)),
            kPseudoActiveViewTransitionType => {
                self.GetTransitionForScope(e).is_some_and(|transition| {
                    b.TransitionMatchActiveType(&transition, &selector.IdentList())
                })
            }
            kPseudoUnparsed => false,
            kPseudoCurrent => {
                c.previously_matched_pseudo_element == PseudoId::kPseudoIdSearchText
                    && c.search_text_request_is_current
            }
            kPseudoPlaceholderShown => {
                b.ProbeForcePseudoState(e, p)
                    || (b.ElementIs(e, C::TextControl) && b.IsPlaceholderVisible(e))
            }
            kPseudoToolFormActive => b.ElementIs(e, C::Form) && b.MatchesToolFormActive(e),
            kPseudoUnbounded => b.ElementIs(e, C::HTMLElement) && b.IsUnboundedElementActive(e),
            kPseudoToolSubmitActive => {
                b.ElementIs(e, C::FormControl) && b.MatchesToolSubmitActive(e)
            }
            kPseudoFullPageMedia => b.IsMediaDocument(e),
            kPseudoDefault => b.MatchesDefault(e),
            kPseudoTextField => b.ElementIs(e, C::Input) && b.IsTextField(e),
            kPseudoDialogInTopLayer => b.IsDialogInTopLayer(e),
            kPseudoPopoverInTopLayer => b.IsPopoverInTopLayer(e),
            kPseudoOverscrollOpen => b.MatchesOverscrollOpen(e),
            kPseudoFullScreenAncestor => b.ContainsFullScreenElement(e),
            kPseudoPictureInPicture => b.IsPictureInPicture(e),
            kPseudoSeeking => b.ElementIs(e, C::Media) && b.MediaSeeking(e),
            kPseudoBuffering => b.ElementIs(e, C::Media) && b.MediaBuffering(e),
            kPseudoStalled => b.ElementIs(e, C::Media) && b.MediaStalled(e),
            kPseudoMuted => b.ElementIs(e, C::Media) && b.MediaMuted(e),
            kPseudoVideoPersistent => b.ElementIs(e, C::Video) && b.VideoPersistent(e),
            kPseudoVideoPersistentAncestor => b.ContainsPersistentVideo(e),
            kPseudoDefined => b.IsDefined(e),
            _ => unreachable!("invalid pseudo-class selector"),
        }
    }
}
