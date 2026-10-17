// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// cpp: selector_checker-inl.h:15-343. This checker only accepts the source's
// nonrecursive selector subset; every other selector stays with SelectorChecker.
#![allow(non_snake_case)]
use crate::css_selector::{
    AttributeMatchType, CSSSelector, CSSSelectorComplex, GetPseudoId, MatchType, PseudoType,
    QualifiedName, RelationType,
};
use crate::selector_checker::SelectorCheckerBackend;
use foundation::AtomicString;
use layoutng_style::style::computed_style_constants::PseudoId;
use std::rc::Rc;

pub(crate) struct EasySelectorChecker;
impl EasySelectorChecker {
    pub(crate) fn IsEasy(selector: CSSSelectorComplex<'_>) -> bool {
        use MatchType::*;
        use PseudoType::*;
        use RelationType::*;
        let mut descendant = false;
        let mut pseudo = false;
        for current in selector.SimpleSelectors() {
            if !current.IsLastInComplexSelector()
                && !matches!(current.Relation(), kSubSelector | kDescendant | kChild)
            {
                return false;
            }
            if current.Relation() == kDescendant {
                descendant = true;
            } else if current.Relation() == kChild && descendant {
                return false;
            }
            if current.IsCoveredByBucketing() {
                continue;
            }
            match current.Match() {
                kTag | kUniversalTag | kId | kClass => (),
                kAttributeExact | kAttributeSet => {
                    if *current.Attribute().Prefix() == AtomicString::from_str("*") {
                        return false;
                    }
                }
                kPseudoClass => {
                    if current.GetPseudoType() != kPseudoNot {
                        return false;
                    }
                    let Some(list) = current.SelectorList() else {
                        return false;
                    };
                    let mut complexes = list.ComplexSelectors();
                    let Some(only) = complexes.next() else {
                        return false;
                    };
                    if complexes.next().is_some()
                        || !only.First().IsLastInComplexSelector()
                        || only.First().Match() != kTag
                    {
                        return false;
                    }
                }
                kPseudoElement => {
                    if !matches!(
                        current.GetPseudoType(),
                        kPseudoBefore
                            | kPseudoAfter
                            | kPseudoMarker
                            | kPseudoScrollbar
                            | kPseudoSelection
                    ) || pseudo
                    {
                        return false;
                    }
                    pseudo = true;
                }
                _ => return false,
            }
        }
        true
    }
    pub(crate) fn Match<B: SelectorCheckerBackend>(
        backend: &B,
        selector: CSSSelectorComplex<'_>,
        mut element: Rc<B::Element>,
        pseudo_element: Option<&B::Element>,
        pseudo_id: PseudoId,
        dynamic: &mut PseudoId,
    ) -> bool {
        debug_assert!(Self::IsEasy(selector));
        let mut current = Some(selector);
        let mut rewind = None;
        while let Some(selector) = current {
            let simple = selector.First();
            if simple.IsCoveredByBucketing()
                || Self::MatchOne(
                    backend,
                    simple,
                    &element,
                    pseudo_element,
                    pseudo_id,
                    dynamic,
                )
            {
                if simple.Relation() == RelationType::kDescendant {
                    rewind = selector.NextSimpleSelector();
                    let Some(parent) = backend.ParentElement(&element) else {
                        return false;
                    };
                    element = parent;
                } else if simple.Relation() == RelationType::kChild {
                    debug_assert!(rewind.is_none());
                    let Some(parent) = backend.ParentElement(&element) else {
                        return false;
                    };
                    element = parent;
                }
                current = selector.NextSimpleSelector();
            } else if rewind.is_some() {
                current = rewind;
                let Some(parent) = backend.ParentElement(&element) else {
                    return false;
                };
                element = parent;
            } else {
                return false;
            }
        }
        true
    }
    fn MatchesTagName<B: SelectorCheckerBackend>(
        b: &B,
        name: &QualifiedName,
        element: &B::Element,
    ) -> bool {
        let actual = b.ElementTagQName(element);
        if actual.NamespaceURI() != name.NamespaceURI()
            && *name.NamespaceURI() != AtomicString::from_str("*")
        {
            return false;
        }
        actual.LocalName() == name.LocalName()
            || (!b.IsHTMLElement(element)
                && b.IsInHTMLDocument(element)
                && actual.LocalName().ToAsciiUpper() == name.LocalName().ToAsciiUpper())
    }
    fn MatchOne<B: SelectorCheckerBackend>(
        b: &B,
        selector: &CSSSelector,
        element: &B::Element,
        pseudo: Option<&B::Element>,
        pseudo_id: PseudoId,
        dynamic: &mut PseudoId,
    ) -> bool {
        use MatchType::*;
        match selector.Match() {
            kTag => Self::MatchesTagName(b, &selector.TagQName(), element),
            kPseudoClass => !Self::MatchesTagName(
                b,
                &selector
                    .SelectorList()
                    .unwrap()
                    .First()
                    .unwrap()
                    .First()
                    .TagQName(),
                element,
            ),
            kUniversalTag => {
                b.ElementNamespaceURI(element) == *selector.TagQName().NamespaceURI()
                    || *selector.TagQName().NamespaceURI() == AtomicString::from_str("*")
            }
            kClass => {
                b.CouldHaveClass(element, &selector.Value())
                    && b.HasClass(element)
                    && b.ClassNamesContain(element, &selector.Value())
            }
            kId => b.HasID(element) && b.IdForStyleResolution(element) == *selector.Value(),
            kAttributeSet | kAttributeExact => {
                let name = selector.Attribute();
                b.SynchronizeAttribute(element, name.LocalName());
                if selector.Match() == kAttributeExact && !b.CouldHaveAttribute(element, &name) {
                    return false;
                }
                let insensitive = selector.AttributeMatch() == AttributeMatchType::kCaseInsensitive
                    || (selector.LegacyCaseInsensitiveMatch()
                        && (!b.CSSAttributeValueCaseSensitiveNonHTMLEnabled()
                            || b.IsHTMLElement(element))
                        && b.IsInHTMLDocument(element));
                for attribute in b.AttributesWithoutUpdate(element).iter() {
                    if b.AttributeMatchesName(&attribute, &name)
                        || (!b.IsHTMLElement(element)
                            && b.IsInHTMLDocument(element)
                            && b.AttributeMatchesNameCaseInsensitive(&attribute, &name))
                    {
                        if selector.Match() == kAttributeSet {
                            return true;
                        }
                        let value = b.AttributeValue(&attribute);
                        return value == *selector.Value()
                            || insensitive
                                && value.ToAsciiLower() == selector.Value().ToAsciiLower();
                    }
                }
                false
            }
            kPseudoElement => {
                let id = GetPseudoId(selector.GetPseudoType());
                if let Some(pseudo) = pseudo {
                    if b.ParentElement(pseudo)
                        .is_some_and(|parent| b.IsPseudoElement(&parent))
                    {
                        return false;
                    }
                    b.ElementPseudoIdForStyling(pseudo) == id
                } else if pseudo_id != PseudoId::kPseudoIdNone {
                    pseudo_id == id
                } else {
                    *dynamic = id;
                    true
                }
            }
            _ => unreachable!("IsEasy excludes this selector"),
        }
    }
}
