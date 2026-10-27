/*
 * Copyright (C) 1999 Lars Knoll (knoll@kde.org)
 *           (C) 2004-2005 Allan Sandfeld Jensen (kde@carewolf.com)
 * Copyright (C) 2006, 2007 Nicholas Shanks (webkit@nickshanks.com)
 * Copyright (C) 2005, 2006, 2007, 2008, 2009, 2010, 2011 Apple Inc. All rights
 * reserved.
 * Copyright (C) 2007 Alexey Proskuryakov <ap@webkit.org>
 * Copyright (C) 2007, 2008 Eric Seidel <eric@webkit.org>
 * Copyright (C) 2008, 2009 Torch Mobile Inc. All rights reserved.
 * (http://www.torchmobile.com/)
 * Copyright (c) 2011, Code Aurora Forum. All rights reserved.
 * Copyright (C) Research In Motion Limited 2011. All rights reserved.
 * Copyright (C) 2012 Google Inc. All rights reserved.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Library General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Library General Public License for more details.
 *
 * You should have received a copy of the GNU Library General Public License
 * along with this library; see the file COPYING.LIB.  If not, write to
 * the Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
 * Boston, MA 02110-1301, USA.
 */

// cpp: third_party/blink/renderer/core/css/selector_filter.h:90-93,153-154
// cpp: third_party/blink/renderer/core/css/selector_filter.cc:45
// PersistentSelectorService supplies retained DOM identifiers and parent edges;
// this module owns the translated parent stack and ancestor rejection filter.

// Direct SelectorQuery dependencies: selector_filter.cc:196-276 (subject/single helpers).
// Source commit: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
// Selected dependency ledger (physical / effective / mapped / omitted / pending):
//   selector_filter.cc:196-276: 81 / 56 / 56 / 0 / 0.
//   element.h:916-930:         15 / 10 / 10 / 0 / 0.
//   element.h:2126-2143:       18 /  7 /  7 / 0 / 0.
// Same effective-line exclusions as selector_query.rs. IsExcludedAttribute
// is Element's required registry-aware operation; no substitute policy exists.

use foundation::WtfSizeT;

// cpp: selector_filter.h:90-93
#[derive(Clone, Copy, Debug)]
pub struct Mark {
    pub parent_stack_size: WtfSizeT,
    pub set_bits_size: WtfSizeT,
}

// cpp: selector_filter.h:153-154
// Module scope shares the source constants with the production DOM adapter.
pub const kFilterSize: u32 = 8192;
pub const kFilterMask: u32 = kFilterSize - 1;

// cpp: selector_filter.cc:45
// Salts distinguish hashes of identical names used in different selectors.
pub const kTagNameSalt: u32 = 1;
pub const kIdSalt: u32 = 3;
pub const kClassSalt: u32 = 5;
pub const kAttributeSalt: u32 = 7;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttributesToExcludeHashesFor {
    kExcludeStandardAttributesOnly,
    kExcludeAllLazilySynchronizedAttributes,
    kExcludeLowercaseLazilySynchronizedAttributes,
}
/// Requires Element's actual defined-name/lazy-attribute registry policy.
pub trait SelectorQueryFilterBackend {
    fn IsExcludedAttribute(
        &self,
        name: &crate::css_selector::QualifiedName,
        policy: AttributesToExcludeHashesFor,
    ) -> bool;
}
// cpp: element.h:916-930. Same two bit positions from the AtomicString hash.
pub fn FilterForString(value: &foundation::AtomicString) -> u32 {
    let hash = value.Hash();
    (1u32 << (hash & 31)) | (1u32 << ((hash >> 5) & 31))
}
pub fn FilterForAttribute(name: &crate::css_selector::QualifiedName) -> u32 {
    FilterForString(&name.LocalName().ToAsciiUpper())
}
pub fn CollectSubjectIdentifierHashes<B: SelectorQueryFilterBackend>(
    selector: crate::css_selector::CSSSelectorComplex<'_>,
    policy: AttributesToExcludeHashesFor,
    filter: &mut u32,
    backend: &B,
) {
    for current in selector.SimpleSelectors() {
        CollectSingleSelectorIdentifierHashes(current, policy, filter, backend);
        if current.Relation() != crate::css_selector::RelationType::kSubSelector {
            break;
        }
    }
}
pub fn CollectSingleSelectorIdentifierHashes<B: SelectorQueryFilterBackend>(
    current: &crate::css_selector::CSSSelector,
    policy: AttributesToExcludeHashesFor,
    filter: &mut u32,
    backend: &B,
) {
    use crate::css_selector::{MatchType, PseudoType};
    match current.Match() {
        MatchType::kClass => {
            if !current.Value().empty() {
                *filter |= FilterForString(&current.Value());
            }
        }
        MatchType::kAttributeExact
        | MatchType::kAttributeSet
        | MatchType::kAttributeList
        | MatchType::kAttributeContain
        | MatchType::kAttributeBegin
        | MatchType::kAttributeEnd
        | MatchType::kAttributeHyphen => {
            if !backend.IsExcludedAttribute(&current.Attribute(), policy) {
                *filter |= FilterForAttribute(&current.Attribute());
            }
        }
        MatchType::kPseudoClass
            if matches!(
                current.GetPseudoType(),
                PseudoType::kPseudoIs | PseudoType::kPseudoWhere | PseudoType::kPseudoParent
            ) =>
        {
            let list = current.SelectorListOrParent();
            if current.GetPseudoType() != PseudoType::kPseudoParent || list.is_some() {
                let mut intersection = u32::MAX;
                if let Some(list) = list {
                    for selector in list.ComplexSelectors() {
                        let mut sub = 0;
                        CollectSubjectIdentifierHashes(selector, policy, &mut sub, backend);
                        intersection &= sub;
                    }
                }
                *filter |= intersection;
            }
        }
        _ => {}
    }
}

// cpp: selector_filter.h:95-177; selector_filter.cc:277-319.
// Element identities and identifiers are supplied by the retained DOM adapter.
pub struct SelectorFilter {
    parents: Vec<usize>,
    set_bits: Vec<u16>,
    parent_marks: Vec<Mark>,
    bits: [u64; 128],
}
impl Default for SelectorFilter {
    fn default() -> Self {
        Self {
            parents: Vec::new(),
            set_bits: Vec::new(),
            parent_marks: Vec::new(),
            bits: [0; 128],
        }
    }
}
impl SelectorFilter {
    pub fn PushParent(&mut self, parent: usize, hashes: impl IntoIterator<Item = u32>) {
        self.parent_marks.push(Mark {
            parent_stack_size: self.parents.len() as WtfSizeT,
            set_bits_size: self.set_bits.len() as WtfSizeT,
        });
        self.parents.push(parent);
        for hash in hashes {
            let bit = hash & kFilterMask;
            let word = &mut self.bits[bit as usize / 64];
            let mask = 1u64 << (bit % 64);
            if *word & mask == 0 {
                *word |= mask;
                self.set_bits.push(bit as u16);
            }
        }
    }
    pub fn Parents(&self) -> &[usize] {
        &self.parents
    }
    pub fn PopToParentCount(&mut self, count: usize) {
        if count == self.parents.len() {
            return;
        }
        let mark = self.parent_marks[count];
        while self.set_bits.len() > mark.set_bits_size as usize {
            let bit = self.set_bits.pop().unwrap() as usize;
            self.bits[bit / 64] &= !(1u64 << (bit % 64));
        }
        self.parents.truncate(mark.parent_stack_size as usize);
        self.parent_marks.truncate(mark.parent_stack_size as usize);
    }
    pub fn ParentStackIsConsistent(&self, parent: Option<usize>) -> bool {
        self.parents.last().copied() == parent
    }
    pub fn FastRejectSelector(&self, hashes: &[u16]) -> bool {
        hashes.iter().any(|&hash| {
            let bit = u32::from(hash) & kFilterMask;
            self.bits[bit as usize / 64] & (1u64 << (bit % 64)) == 0
        })
    }
}

// The persistent DOM eagerly synchronizes attributes. Standard identifiers
// are handled separately, just as Element::IsExcludedAttribute does in Blink.
pub(crate) struct ProductionFilterAttributes;
impl SelectorQueryFilterBackend for ProductionFilterAttributes {
    fn IsExcludedAttribute(
        &self,
        name: &crate::css_selector::QualifiedName,
        _: AttributesToExcludeHashesFor,
    ) -> bool {
        matches!(name.LocalName().Utf8().as_str(), "id" | "class" | "style")
    }
}

// cpp: selector_filter.cc:79-194,319-330. Keep sibling compounds out of the
// ancestor filter and intersect selector-list subject requirements.
pub trait SelectorFilterScope {
    fn From(&self) -> Option<&crate::css_selector_list::CSSSelectorList>;
    fn Parent(&self) -> Option<&Self>;
}

pub fn CollectIdentifierHashes<S: SelectorFilterScope>(
    selector: crate::css_selector::CSSSelectorComplex<'_>,
    scope: Option<&S>,
    backing: &mut Vec<u16>,
    subject: &mut u32,
) {
    let mut simples = selector.SimpleSelectors();
    if let Some(first) = simples.next() {
        CollectDescendantCompoundHashes(simples, first.Relation(), scope, backing);
    }
    *subject = 0;
    CollectSubjectIdentifierHashes(
        selector,
        AttributesToExcludeHashesFor::kExcludeAllLazilySynchronizedAttributes,
        subject,
        &ProductionFilterAttributes,
    );
}
fn CollectDescendantCompoundHashes<'a, S: SelectorFilterScope>(
    selectors: impl Iterator<Item = &'a crate::css_selector::CSSSelector>,
    mut relation: crate::css_selector::RelationType,
    scope: Option<&S>,
    hashes: &mut Vec<u16>,
) {
    use crate::css_selector::RelationType::*;
    let mut skip = true;
    for current in selectors {
        match relation {
            kSubSelector => {
                if !skip {
                    CollectDescendantHashes(current, scope, hashes);
                }
            }
            kDirectAdjacent | kIndirectAdjacent | kPseudoChild => skip = true,
            kShadowSlot | kDescendant | kChild | kUAShadow | kShadowPart => {
                skip = false;
                CollectDescendantHashes(current, scope, hashes);
            }
            kRelativeDescendant
            | kRelativeChild
            | kRelativeDirectAdjacent
            | kRelativeIndirectAdjacent => {
                unreachable!("relative selector is not an ancestor compound")
            }
        }
        relation = current.Relation();
    }
}
fn CollectDescendantHashes<S: SelectorFilterScope>(
    selector: &crate::css_selector::CSSSelector,
    scope: Option<&S>,
    hashes: &mut Vec<u16>,
) {
    use crate::css_selector::{MatchType::*, PseudoType::*};
    let hash = match selector.Match() {
        kId if !selector.Value().empty() => Some(selector.Value().Hash().wrapping_mul(kIdSalt)),
        kClass if !selector.Value().empty() => {
            Some(selector.Value().Hash().wrapping_mul(kClassSalt))
        }
        kTag => Some(
            selector
                .TagQName()
                .LocalName()
                .Hash()
                .wrapping_mul(kTagNameSalt),
        ),
        kAttributeExact | kAttributeSet | kAttributeList | kAttributeContain | kAttributeBegin
        | kAttributeEnd | kAttributeHyphen => (!ProductionFilterAttributes.IsExcludedAttribute(
            &selector.Attribute(),
            AttributesToExcludeHashesFor::kExcludeStandardAttributesOnly,
        ))
        .then(|| {
            selector
                .Attribute()
                .LocalName()
                .ToAsciiLower()
                .Hash()
                .wrapping_mul(kAttributeSalt)
        }),
        kPseudoClass => {
            let (list, nested_scope) = match selector.GetPseudoType() {
                kPseudoIs | kPseudoWhere | kPseudoParent => {
                    (selector.SelectorListOrParent(), scope)
                }
                kPseudoScope => (
                    scope
                        .and_then(|scope| scope.From())
                        .map(crate::css_selector::SelectorListRef::Parent),
                    scope.and_then(|scope| scope.Parent()),
                ),
                _ => (None, scope),
            };
            if let Some(list) = list {
                let mut complexes = list.ComplexSelectors();
                if let Some(only) = complexes.next() {
                    if complexes.next().is_none() {
                        CollectDescendantCompoundHashes(
                            only.SimpleSelectors(),
                            crate::css_selector::RelationType::kDescendant,
                            nested_scope,
                            hashes,
                        );
                    }
                }
            }
            None
        }
        _ => None,
    };
    if let Some(hash) = hash {
        hashes.push(hash as u16);
    }
}
