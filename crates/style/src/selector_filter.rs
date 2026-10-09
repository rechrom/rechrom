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
// The original batch maps SelectorFilter::Mark and constants. The DOM-backed
// SelectorFilter parent-stack and ancestor-collection routines remain pending;
// subject/single collection dependencies are mapped below.

// Direct SelectorQuery dependencies; the rest of SelectorFilter above remains
// separately pending. cpp: selector_filter.cc:196-276 (subject/single helpers).
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
// These are SelectorFilter's constants; module scope retains them until the
// owning class can be mapped with the genuine DOM parent-stack type.
pub const kFilterSize: u32 = 8192;
pub const kFilterMask: u32 = kFilterSize - 1;

// cpp: selector_filter.cc:45
// Salts distinguish hashes of identical names used in different selectors.
pub(crate) const kTagNameSalt: u32 = 1;
pub(crate) const kIdSalt: u32 = 3;
pub(crate) const kClassSalt: u32 = 5;
pub(crate) const kAttributeSalt: u32 = 7;

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
