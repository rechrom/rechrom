/*
 * (C) 1999-2003 Lars Knoll (knoll@kde.org)
 * Copyright (C) 2004, 2005, 2006, 2008, 2012 Apple Inc. All rights reserved.
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
// cpp: third_party/blink/renderer/core/css/css_property_value_set.h:160-162,172-177,204-210,242-260,269-283,361-377,482-486
// cpp: third_party/blink/renderer/core/css/css_property_value_set.cc:137-148
// cpp: third_party/blink/renderer/core/css/css_property_value_set.h:112-118,122-222,227-236,241-285,296-313,316-339,341-459,462-539
// cpp: third_party/blink/renderer/core/css/css_property_value_set.cc:49-201,210-353,370-930,938-945
// CSSPropertyValueSetState retains the exact eight-byte packed base state.
// Property storage below is backed by real CSSPropertyValue/CSSValue payloads.

use crate::css_property_names::CSSPropertyID;
use crate::parser::at_rule_descriptors::{AtRuleDescriptorID, AtRuleDescriptorIDAsCSSPropertyID};
use crate::parser::css_parser_mode::CSSParserMode;
use foundation::AtomicString;
use std::cell::Cell;
use std::sync::atomic::{AtomicU32, Ordering};

// cpp: css_property_value_set.h:283,482-486
// Direct dependencies: platform/wtf/hash_traits.h:203-223,241-243.
// Default HashTraits<unsigned> reserves 0 and unsigned(-1).
const HASH_EMPTY_VALUE: u32 = 0;
const HASH_DELETED_VALUE: u32 = u32::MAX;

// cpp: css_property_value_set.h:242,269-283
// Packed state shared by the mutable and immutable property stores.
#[repr(C)]
pub struct CSSPropertyValueSetState {
    bits_: AtomicU32,
    hash_: Cell<u32>,
}

#[allow(non_snake_case, non_upper_case_globals)]
impl CSSPropertyValueSetState {
    // cpp: css_property_value_set.h:242
    pub(crate) const kMaxArraySize: u32 = (1 << 24) - 1;

    // cpp: css_property_value_set.h:269-278
    const CSSParserModeShift: u32 = 24;
    const CSSParserModeMask: u32 = 0xF << Self::CSSParserModeShift;
    const IsMutableMask: u32 = 1 << 28;
    const ContainsCursorHandMask: u32 = 1 << 29;
    const MayHaveLogicalPropertiesMask: u32 = 1 << 30;
    const HasAllMask: u32 = 1 << 31;

    // cpp: css_property_value_set.h:244-248,283
    // Rust names distinguish the two protected constructor overloads.
    pub(crate) fn MutableBase(css_parser_mode: CSSParserMode) -> Self {
        Self {
            bits_: AtomicU32::new(Self::EncodeParserMode(css_parser_mode) | Self::IsMutableMask),
            hash_: Cell::new(HASH_EMPTY_VALUE),
        }
    }

    // cpp: css_property_value_set.h:250-260,283
    pub(crate) fn ImmutableBase(
        css_parser_mode: CSSParserMode,
        immutable_array_size: u32,
        contains_cursor_hand: bool,
    ) -> Self {
        let array_size = if immutable_array_size < Self::kMaxArraySize {
            immutable_array_size
        } else {
            Self::kMaxArraySize
        };
        Self {
            bits_: AtomicU32::new(
                array_size
                    | Self::EncodeParserMode(css_parser_mode)
                    | ((contains_cursor_hand as u32) * Self::ContainsCursorHandMask),
            ),
            hash_: Cell::new(HASH_EMPTY_VALUE),
        }
    }

    // The source's BitField::Value::encode also checks the field width.
    fn EncodeParserMode(css_parser_mode: CSSParserMode) -> u32 {
        debug_assert!((css_parser_mode as u32) < 16);
        (css_parser_mode as u32) << Self::CSSParserModeShift
    }

    // cpp: css_property_value_set.h:160-162
    pub fn CssParserMode(&self) -> CSSParserMode {
        // Safe conversion of the source static_cast; constructors are the only
        // producers of this field, and every defined enum value is represented.
        match (self.bits_.load(Ordering::Relaxed) & Self::CSSParserModeMask)
            >> Self::CSSParserModeShift
        {
            0 => CSSParserMode::kHTMLStandardMode,
            1 => CSSParserMode::kHTMLQuirksMode,
            2 => CSSParserMode::kSVGAttributeMode,
            3 => CSSParserMode::kCSSFontFaceRuleMode,
            4 => CSSParserMode::kCSSKeyframeRuleMode,
            5 => CSSParserMode::kCSSPropertyRuleMode,
            6 => CSSParserMode::kCSSFontPaletteValuesRuleMode,
            7 => CSSParserMode::kCSSPositionTryRuleMode,
            8 => CSSParserMode::kCSSFunctionDescriptorsMode,
            9 => CSSParserMode::kCSSCounterStyleRuleMode,
            10 => CSSParserMode::kUASheetMode,
            11 => CSSParserMode::kNumCSSParserModes,
            _ => unreachable!("parser mode was not produced by a constructor"),
        }
    }

    // cpp: css_property_value_set.h:172
    pub fn IsMutable(&self) -> bool {
        self.bits_.load(Ordering::Relaxed) & Self::IsMutableMask != 0
    }

    // cpp: css_property_value_set.h:173-175
    pub fn ContainsCursorHand(&self) -> bool {
        self.bits_.load(Ordering::Relaxed) & Self::ContainsCursorHandMask != 0
    }

    // cpp: css_property_value_set.h:177
    pub fn HasAllProperty(&self) -> bool {
        self.bits_.load(Ordering::Relaxed) & Self::HasAllMask != 0
    }

    // cpp: css_property_value_set.h:204-207
    pub fn GetExistingHash(&self) -> u32 {
        let hash = self.hash_.get();
        debug_assert_ne!(hash, HASH_EMPTY_VALUE);
        hash
    }

    // cpp: css_property_value_set.h:208-210
    pub fn ModifiedSinceHashing(&self) -> bool {
        self.hash_.get() == HASH_DELETED_VALUE
    }
}

// cpp: css_property_value_set.h:482-486
// Shared original cache invalidation algorithm.
#[allow(non_snake_case)]
pub(crate) fn InvalidateHashIfComputed(set: &mut CSSPropertyValueSetState) {
    if set.hash_.get() != HASH_EMPTY_VALUE {
        set.hash_.set(HASH_DELETED_VALUE);
    }
}

// cpp: css_property_value_set.h:361-377
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
pub enum SetResult {
    kParseError = 0,
    kUnchanged = 1,
    kModifiedExisting = 2,
    kChangedPropertySet = 3,
}

// cpp: css_property_value_set.cc:137-139
// Rust names distinguish the three source overloads, with their actual types.
#[allow(non_snake_case)]
pub(crate) fn GetConvertedCSSPropertyID(property_id: CSSPropertyID) -> u16 {
    property_id as u16
}

// cpp: css_property_value_set.cc:141-143
#[allow(non_snake_case)]
pub(crate) fn GetConvertedCSSPropertyIDForCustomProperty(_: &AtomicString) -> u16 {
    CSSPropertyID::kVariable as u16
}

// cpp: css_property_value_set.cc:145-148
#[allow(non_snake_case)]
pub(crate) fn GetConvertedCSSPropertyIDForDescriptor(descriptor_id: AtRuleDescriptorID) -> u16 {
    AtRuleDescriptorIDAsCSSPropertyID(descriptor_id) as u16
}

const _: () = assert!(std::mem::size_of::<CSSPropertyValueSetState>() == 8);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packed_fields_preserve_mode_and_bound_immutable_size() {
        let modes = [
            CSSParserMode::kHTMLStandardMode,
            CSSParserMode::kHTMLQuirksMode,
            CSSParserMode::kSVGAttributeMode,
            CSSParserMode::kCSSFontFaceRuleMode,
            CSSParserMode::kCSSKeyframeRuleMode,
            CSSParserMode::kCSSPropertyRuleMode,
            CSSParserMode::kCSSFontPaletteValuesRuleMode,
            CSSParserMode::kCSSPositionTryRuleMode,
            CSSParserMode::kCSSFunctionDescriptorsMode,
            CSSParserMode::kCSSCounterStyleRuleMode,
            CSSParserMode::kUASheetMode,
            CSSParserMode::kNumCSSParserModes,
        ];
        for mode in modes {
            let mutable = CSSPropertyValueSetState::MutableBase(mode);
            assert!(mutable.IsMutable());
            assert!(!mutable.ContainsCursorHand() && !mutable.HasAllProperty());
            assert_eq!(mutable.CssParserMode(), mode);
            for count in [
                0,
                1,
                CSSPropertyValueSetState::kMaxArraySize - 1,
                CSSPropertyValueSetState::kMaxArraySize,
                CSSPropertyValueSetState::kMaxArraySize + 1,
                u32::MAX,
            ] {
                let immutable = CSSPropertyValueSetState::ImmutableBase(mode, count, true);
                assert!(!immutable.IsMutable());
                assert!(immutable.ContainsCursorHand());
                assert!(!immutable.HasAllProperty());
                assert_eq!(immutable.CssParserMode(), mode);
                assert_eq!(
                    immutable.bits_.load(Ordering::Relaxed)
                        & CSSPropertyValueSetState::kMaxArraySize,
                    count.min(CSSPropertyValueSetState::kMaxArraySize)
                );
                assert_eq!(
                    immutable.bits_.load(Ordering::Relaxed)
                        & CSSPropertyValueSetState::MayHaveLogicalPropertiesMask,
                    0
                );
            }
        }
    }

    #[test]
    fn invalidation_distinguishes_unhashed_from_hashed_and_never_recovers() {
        let mut set = CSSPropertyValueSetState::MutableBase(CSSParserMode::kHTMLStandardMode);
        InvalidateHashIfComputed(&mut set);
        assert_eq!(set.hash_.get(), HASH_EMPTY_VALUE);
        assert!(!set.ModifiedSinceHashing());
        // Seed the private field to represent ComputeHash's result without
        // inventing a CSSValue implementation or exposing such a setter.
        set.hash_.set(3141592653);
        assert_eq!(set.GetExistingHash(), 3141592653);
        InvalidateHashIfComputed(&mut set);
        assert!(set.ModifiedSinceHashing());
        assert_eq!(set.GetExistingHash(), HASH_DELETED_VALUE);
        InvalidateHashIfComputed(&mut set);
        assert_eq!(set.GetExistingHash(), HASH_DELETED_VALUE);
        let fresh = CSSPropertyValueSetState::ImmutableBase(set.CssParserMode(), 0, false);
        assert!(!fresh.ModifiedSinceHashing());
        assert_eq!(fresh.hash_.get(), HASH_EMPTY_VALUE);
    }
}

use crate::css_property_name::CSSPropertyName;
use crate::css_property_value::{CSSPropertyValue, CSSPropertyValueBackend};
use crate::css_value::CSSValue;
use crate::properties::css_property::CSSProperty;
use crate::property_bitsets::kLogicalGroupProperties;
use foundation::{AddIntToHash, CSSValueID, String, StringView};
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

// Original dependencies only. No method has a default implementation and
// serialization must be supplied by the real StylePropertySerializer.
pub trait CSSPropertyValueSetBackend: CSSPropertyValueBackend {
    type CSSStyleDeclaration;
    type ExecutionContext;
    fn ShorthandForProperty(id: CSSPropertyID) -> Vec<CSSPropertyID>;
    fn IsInSameLogicalPropertyGroupWithDifferentMappingLogic(
        a: CSSPropertyID,
        b: CSSPropertyID,
    ) -> bool;
    fn SerializeShorthand(set: &CSSPropertyValueSet<Self>, id: CSSPropertyID) -> String;
    fn AsText(set: &CSSPropertyValueSet<Self>) -> String;
    fn CreateIdentifier(id: CSSValueID) -> Rc<CSSValue<Self>>;
    fn DeclarationPropertyValueSet(
        style: &Self::CSSStyleDeclaration,
    ) -> Option<&CSSPropertyValueSet<Self>>;
    fn DeclarationPropertyMatches(
        style: &Self::CSSStyleDeclaration,
        id: CSSPropertyID,
        value: &CSSValue<Self>,
    ) -> bool;
    fn NewCSSStyleDeclaration(
        context: Option<&Self::ExecutionContext>,
        set: &MutableCSSPropertyValueSet<Self>,
    ) -> Rc<Self::CSSStyleDeclaration>;
}

// A template argument in the C++ overloads becomes an explicit typed key.
// Descriptors retain their source conversion and 'all' handling.
#[derive(Clone, Copy)]
pub enum PropertyKey<'a> {
    Property(CSSPropertyID),
    Custom(&'a AtomicString),
    Descriptor(AtRuleDescriptorID),
}
impl From<CSSPropertyID> for PropertyKey<'_> {
    fn from(id: CSSPropertyID) -> Self {
        Self::Property(id)
    }
}
impl<'a> From<&'a AtomicString> for PropertyKey<'a> {
    fn from(name: &'a AtomicString) -> Self {
        Self::Custom(name)
    }
}
impl From<AtRuleDescriptorID> for PropertyKey<'_> {
    fn from(id: AtRuleDescriptorID) -> Self {
        Self::Descriptor(id)
    }
}
impl PropertyKey<'_> {
    // cpp: css_property_value_set.cc:137-180
    fn Matches<D: CSSPropertyValueSetBackend>(self, p: &CSSPropertyValue<D>) -> bool {
        match self {
            Self::Property(id) => p.PropertyID() as u16 == GetConvertedCSSPropertyID(id),
            Self::Custom(name) => {
                p.PropertyID() == CSSPropertyID::kVariable && p.CustomPropertyName() == name
            }
            Self::Descriptor(id) => {
                p.PropertyID() as u16 == GetConvertedCSSPropertyIDForDescriptor(id)
            }
        }
    }
    // cpp: css_property_value_set.cc:278-288. The custom-property overload
    // actually returns true even though the adjacent source comment says false.
    fn IsAffectedByAll<D: CSSPropertyValueSetBackend>(self) -> bool {
        match self {
            Self::Property(id) => D::IsAffectedByAll(id),
            Self::Custom(_) => true,
            Self::Descriptor(_) => false,
        }
    }
}

#[allow(non_snake_case)]
pub struct CSSPropertyValueSet<D: CSSPropertyValueSetBackend> {
    state: CSSPropertyValueSetState,
    properties: Vec<CSSPropertyValue<D>>,
}
// Source derived classes own their storage; Rust separates mutable APIs from
// the base view, and Rc retains the original GC references without fake values.
pub struct MutableCSSPropertyValueSet<D: CSSPropertyValueSetBackend> {
    set: CSSPropertyValueSet<D>,
    cssom_wrapper_: Option<Rc<D::CSSStyleDeclaration>>,
}
pub struct ImmutableCSSPropertyValueSet<D: CSSPropertyValueSetBackend>(std::marker::PhantomData<D>);
impl<D: CSSPropertyValueSetBackend> Deref for MutableCSSPropertyValueSet<D> {
    type Target = CSSPropertyValueSet<D>;
    fn deref(&self) -> &Self::Target {
        &self.set
    }
}
impl<D: CSSPropertyValueSetBackend> DerefMut for MutableCSSPropertyValueSet<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.set
    }
}

#[allow(non_snake_case)]
impl<D: CSSPropertyValueSetBackend> CSSPropertyValueSet<D> {
    // cpp: css_property_value_set.h:497-539
    pub fn PropertyCount(&self) -> usize {
        self.properties.len()
    }
    pub fn IsEmpty(&self) -> bool {
        self.PropertyCount() == 0
    }
    pub fn PropertyAt(&self, index: usize) -> &CSSPropertyValue<D> {
        &self.properties[index]
    }
    pub fn Properties(&self) -> &[CSSPropertyValue<D>] {
        &self.properties
    }
    pub fn CssParserMode(&self) -> CSSParserMode {
        self.state.CssParserMode()
    }
    pub fn IsMutable(&self) -> bool {
        self.state.IsMutable()
    }
    pub fn ContainsCursorHand(&self) -> bool {
        self.state.ContainsCursorHand()
    }
    pub fn HasAllProperty(&self) -> bool {
        self.state.HasAllProperty()
    }
    pub fn GetExistingHash(&self) -> u32 {
        self.state.GetExistingHash()
    }
    pub fn ModifiedSinceHashing(&self) -> bool {
        self.state.ModifiedSinceHashing()
    }
    fn SetBit(&mut self, mask: u32, value: bool) {
        let bits = self.state.bits_.get_mut();
        if value {
            *bits |= mask
        } else {
            *bits &= !mask
        }
    }
    fn MayHaveLogicalProperties(&self) -> bool {
        self.state.bits_.load(Ordering::Relaxed)
            & CSSPropertyValueSetState::MayHaveLogicalPropertiesMask
            != 0
    }
    // cpp: css_property_value_set.cc:182-194,900-918
    pub fn FindPropertyIndex<'a>(&self, key: impl Into<PropertyKey<'a>>) -> i32 {
        let key = key.into();
        if self.IsMutable() {
            self.properties.iter().position(|p| key.Matches(p))
        } else {
            self.properties.iter().rposition(|p| key.Matches(p))
        }
        .map_or(-1, |n| n as i32)
    }
    pub fn HasProperty(&self, id: CSSPropertyID) -> bool {
        self.FindPropertyIndex(id) != -1
    }
    // cpp: css_property_value_set.cc:327-350
    pub fn GetPropertyCSSValue<'a>(&self, key: impl Into<PropertyKey<'a>>) -> Option<&CSSValue<D>> {
        let i = self.FindPropertyIndex(key);
        if i < 0 {
            None
        } else {
            Some(self.PropertyAt(i as usize).Value())
        }
    }
    pub fn GetPropertyValueRef<'a>(
        &self,
        key: impl Into<PropertyKey<'a>>,
    ) -> Option<Rc<CSSValue<D>>> {
        let i = self.FindPropertyIndex(key);
        if i < 0 {
            None
        } else {
            Some(self.PropertyAt(i as usize).ValueRef())
        }
    }
    pub fn GetPropertyCSSValueWithHint(&self, name: &AtomicString, index: usize) -> &CSSValue<D> {
        debug_assert_eq!(name, self.PropertyAt(index).Name().ToAtomicString());
        self.PropertyAt(index).Value()
    }
    // cpp: css_property_value_set.cc:231-276
    fn SerializeShorthand(&self, key: PropertyKey<'_>) -> String {
        let id = match key {
            PropertyKey::Property(id) => id,
            _ => return String::default(),
        };
        if id == CSSPropertyID::kAll {
            if !self.HasAllProperty() {
                return String::from("");
            }
            let i = self.FindPropertyIndex(id) as usize;
            let all = self.PropertyAt(i);
            for p in &self.properties[i + 1..] {
                if p.IsAffectedByAll() && all.Value() != p.Value() {
                    return String::from("");
                }
            }
            return all.Value().CssText();
        }
        if D::ShorthandForProperty(id).is_empty() {
            return String::default();
        }
        D::SerializeShorthand(self, id)
    }
    // cpp: css_property_value_set.cc:290-323
    pub fn GetPropertyValue<'a>(&self, key: impl Into<PropertyKey<'a>>) -> String {
        let key = key.into();
        let text = self.SerializeShorthand(key);
        if !text.IsNull() {
            return text;
        }
        if self.HasAllProperty() && key.IsAffectedByAll::<D>() {
            let i = self.FindPropertyIndex(CSSPropertyID::kAll);
            if self.FindPropertyIndex(key) < i {
                return self.PropertyAt(i as usize).Value().CssText();
            }
        }
        self.GetPropertyCSSValue(key)
            .map_or_else(|| String::from(""), CSSValue::CssText)
    }
    pub fn GetPropertyValueWithHint(&self, name: &AtomicString, index: usize) -> String {
        self.GetPropertyCSSValueWithHint(name, index).CssText()
    }
    // cpp: css_property_value_set.cc:429-462
    pub fn PropertyIsImportant<'a>(&self, key: impl Into<PropertyKey<'a>>) -> bool {
        let key = key.into();
        let i = self.FindPropertyIndex(key);
        if i != -1 {
            self.PropertyAt(i as usize).IsImportant()
        } else {
            self.ShorthandIsImportant(key)
        }
    }
    pub fn PropertyIsImportantWithHint(&self, name: &AtomicString, index: usize) -> bool {
        debug_assert_eq!(name, self.PropertyAt(index).Name().ToAtomicString());
        self.PropertyAt(index).IsImportant()
    }
    pub fn ShorthandIsImportant<'a>(&self, key: impl Into<PropertyKey<'a>>) -> bool {
        let id = match key.into() {
            PropertyKey::Property(id) => id,
            _ => return false,
        };
        let longhands = D::ShorthandForProperty(id);
        !longhands.is_empty() && longhands.into_iter().all(|id| self.PropertyIsImportant(id))
    }
    // cpp: css_property_value_set.cc:464-482
    pub fn GetPropertyShorthand(&self, id: CSSPropertyID) -> CSSPropertyID {
        let i = self.FindPropertyIndex(id);
        if i == -1 {
            CSSPropertyID::kInvalid
        } else {
            self.PropertyAt(i as usize).ShorthandID()
        }
    }
    pub fn IsPropertyImplicit(&self, id: CSSPropertyID) -> bool {
        let i = self.FindPropertyIndex(id);
        i != -1 && self.PropertyAt(i as usize).IsImplicit()
    }
    // cpp: css_property_value_set.cc:75-94; hash_traits.h:396-401
    pub fn ComputeHash(&self) -> u32 {
        let mut hash = 3141592653;
        for p in &self.properties {
            AddIntToHash(
                &mut hash,
                if p.PropertyID() == CSSPropertyID::kVariable {
                    p.Name().ToAtomicString().Hash()
                } else {
                    p.PropertyID() as u32
                },
            );
            AddIntToHash(&mut hash, p.IsImportant() as u32);
            AddIntToHash(&mut hash, p.Value().Hash());
        }
        if hash == HASH_EMPTY_VALUE || hash == HASH_DELETED_VALUE {
            1
        } else {
            hash
        }
    }
    // cpp: css_property_value_set.h:198-218
    pub fn GetHash(&self) -> u32 {
        if self.state.hash_.get() == HASH_EMPTY_VALUE {
            self.state.hash_.set(self.ComputeHash());
        }
        self.state.hash_.get()
    }
    pub fn Equals(&self, other: &Self) -> bool {
        std::ptr::eq(self, other) || self.Properties() == other.Properties()
    }
    // cpp: css_property_value_set.cc:699-716
    pub fn AsText(&self) -> String {
        D::AsText(self)
    }
    pub fn HasFailedOrCanceledSubresources(&self) -> bool {
        self.properties
            .iter()
            .any(|p| p.Value().HasFailedOrCanceledSubresources())
    }
    // cpp: css_property_value_set.cc:807-826
    pub fn PropertyMatches(&self, id: CSSPropertyID, value: &CSSValue<D>) -> bool {
        let i = self.FindPropertyIndex(id);
        i != -1 && self.PropertyAt(i as usize).Value() == value
    }
    pub fn ShorthandPropertyMatches(&self, id: CSSPropertyID, style: &Self) -> bool {
        let text = D::SerializeShorthand(style, id);
        !text.empty() && text == self.SerializeShorthand(PropertyKey::Property(id))
    }
    pub fn ShorthandPropertyMatchesDeclaration(
        &self,
        id: CSSPropertyID,
        style: &D::CSSStyleDeclaration,
    ) -> bool {
        D::DeclarationPropertyValueSet(style)
            .is_some_and(|set| self.ShorthandPropertyMatches(id, set))
    }
    // cpp: css_property_value_set.cc:866-884
    pub fn MutableCopy(&self) -> MutableCSSPropertyValueSet<D> {
        MutableCSSPropertyValueSet::Copy(self)
    }
    pub fn CopyPropertiesInSet(&self, ids: &[CSSPropertyID]) -> MutableCSSPropertyValueSet<D> {
        let properties = ids
            .iter()
            .filter_map(|&id| {
                self.GetPropertyValueRef(id).map(|v| {
                    CSSPropertyValue::new(&CSSPropertyName::new(id), v, false, false, 0, false)
                })
            })
            .collect::<Vec<_>>();
        MutableCSSPropertyValueSet::FromProperties(&properties)
    }
    // cpp: css_property_value_set.cc:61-73. The Rc receiver preserves the
    // source identity return for a set that is already immutable.
    pub fn ImmutableCopyIfNeeded(self: &Rc<Self>) -> Rc<Self> {
        if !self.IsMutable() {
            return self.clone();
        }
        ImmutableCSSPropertyValueSet::Create(self.Properties(), self.CssParserMode(), false)
    }
    // cpp: third_party/blink/renderer/core/css/css_property_value_set.cc:938-945. Allocator representation differs
    // in Rust; sizeof is evaluated on the translated storage representation.
    pub fn AverageSizeInBytes() -> usize {
        std::mem::size_of::<Self>() + 4 * std::mem::size_of::<CSSPropertyValue<D>>()
    }
}

#[allow(non_snake_case)]
impl<D: CSSPropertyValueSetBackend> ImmutableCSSPropertyValueSet<D> {
    // cpp: css_property_value_set.cc:49-59,109-133; h:303-309,333-347
    pub fn Create(
        properties: &[CSSPropertyValue<D>],
        mode: CSSParserMode,
        contains_cursor_hand: bool,
    ) -> Rc<CSSPropertyValueSet<D>> {
        debug_assert!(properties.len() <= CSSPropertyValueSetState::kMaxArraySize as usize);
        let mut set = CSSPropertyValueSet {
            state: CSSPropertyValueSetState::ImmutableBase(
                mode,
                properties
                    .len()
                    .try_into()
                    .expect("checked unsigned array size"),
                contains_cursor_hand,
            ),
            properties: properties.to_vec(),
        };
        // The packed array size is capped by the source constructor.
        let size = (set.state.bits_.load(Ordering::Relaxed)
            & CSSPropertyValueSetState::kMaxArraySize) as usize;
        set.properties.truncate(size);
        let has_all = set
            .properties
            .iter()
            .any(|p| p.PropertyID() == CSSPropertyID::kAll);
        set.SetBit(CSSPropertyValueSetState::HasAllMask, has_all);
        Rc::new(set)
    }
}

#[allow(non_snake_case)]
impl<D: CSSPropertyValueSetBackend> MutableCSSPropertyValueSet<D> {
    // cpp: css_property_value_set.cc:96-107
    pub fn new(mode: CSSParserMode) -> Self {
        Self {
            set: CSSPropertyValueSet {
                state: CSSPropertyValueSetState::MutableBase(mode),
                properties: Vec::new(),
            },
            cssom_wrapper_: None,
        }
    }
    pub fn FromProperties(properties: &[CSSPropertyValue<D>]) -> Self {
        let mut set = Self::new(CSSParserMode::kHTMLStandardMode);
        for p in properties {
            let logical =
                set.MayHaveLogicalProperties() || kLogicalGroupProperties.Has(p.PropertyID());
            set.SetBit(
                CSSPropertyValueSetState::MayHaveLogicalPropertiesMask,
                logical,
            );
            if p.PropertyID() == CSSPropertyID::kAll {
                set.SetBit(CSSPropertyValueSetState::HasAllMask, true);
            }
            set.properties.push(p.clone());
        }
        set
    }
    // cpp: css_property_value_set.cc:207-229
    pub fn Copy(other: &CSSPropertyValueSet<D>) -> Self {
        let mut set = Self::new(other.CssParserMode());
        set.properties = other.properties.clone();
        let logical = if other.IsMutable() {
            other.MayHaveLogicalProperties()
        } else {
            other
                .properties
                .iter()
                .any(|p| kLogicalGroupProperties.Has(p.PropertyID()))
        };
        set.SetBit(
            CSSPropertyValueSetState::MayHaveLogicalPropertiesMask,
            logical,
        );
        set.SetBit(CSSPropertyValueSetState::HasAllMask, other.HasAllProperty());
        set
    }
    fn InvalidateHashIfComputed(&mut self) {
        InvalidateHashIfComputed(&mut self.state)
    }
    // cpp: css_property_value_set.cc:918-928
    fn FindPropertyPointer<'a>(
        &self,
        key: impl Into<PropertyKey<'a>>,
    ) -> Option<&CSSPropertyValue<D>> {
        let key = key.into();
        self.properties.iter().find(|p| key.Matches(p))
    }
    // cpp: css_property_value_set.cc:793-801
    pub fn FindCSSPropertyWithName(
        &mut self,
        name: &CSSPropertyName,
    ) -> Option<&mut CSSPropertyValue<D>> {
        let index = if name.IsCustomProperty() {
            self.FindPropertyIndex(name.ToAtomicString())
        } else {
            self.FindPropertyIndex(name.Id())
        };
        if index < 0 {
            None
        } else {
            Some(&mut self.properties[index as usize])
        }
    }
    // cpp: css_property_value_set.cc:368-385
    fn RemoveShorthandProperty(&mut self, key: PropertyKey<'_>) -> bool {
        let id = match key {
            PropertyKey::Property(id) => id,
            _ => return false,
        };
        if id == CSSPropertyID::kAll {
            self.RemovePropertiesAffectedByAll();
            let i = self.FindPropertyIndex(id);
            return self.RemovePropertyAtIndex(i, None);
        }
        let ids = D::ShorthandForProperty(id);
        if ids.is_empty() {
            return false;
        }
        self.RemovePropertiesInSet(&ids)
    }
    // cpp: css_property_value_set.cc:387-427
    pub fn RemovePropertyAtIndex(&mut self, index: i32, return_text: Option<&mut String>) -> bool {
        if index == -1 {
            if let Some(text) = return_text {
                *text = String::from("");
            }
            return false;
        }
        let index = index as usize;
        if let Some(text) = return_text {
            *text = self.PropertyAt(index).Value().CssText();
        }
        if self.PropertyAt(index).PropertyID() == CSSPropertyID::kAll {
            self.SetBit(CSSPropertyValueSetState::HasAllMask, false);
        }
        self.properties.remove(index);
        self.InvalidateHashIfComputed();
        true
    }
    pub fn RemoveProperty<'a>(
        &mut self,
        key: impl Into<PropertyKey<'a>>,
        return_text: Option<&mut String>,
    ) -> bool {
        let key = key.into();
        if self.RemoveShorthandProperty(key) {
            if let Some(text) = return_text {
                *text = String::from("");
            }
            return true;
        }
        let i = self.FindPropertyIndex(key);
        self.RemovePropertyAtIndex(i, return_text)
    }
    // cpp: css_property_value_set.cc:554-578
    fn FindInsertionPointForID(&mut self, id: CSSPropertyID) -> Option<usize> {
        let i = self.FindPropertyIndex(id);
        if i == -1 {
            return None;
        }
        let i = i as usize;
        if self.MayHaveLogicalProperties() && CSSProperty::Get(id).IsInLogicalPropertyGroup() {
            for n in (i + 1..self.PropertyCount()).rev() {
                if D::IsInSameLogicalPropertyGroupWithDifferentMappingLogic(
                    id,
                    self.PropertyAt(n).PropertyID(),
                ) {
                    self.RemovePropertyAtIndex(i as i32, None);
                    return None;
                }
            }
        }
        Some(i)
    }
    // cpp: css_property_value_set.cc:580-625
    pub fn SetLonghandProperty(&mut self, property: CSSPropertyValue<D>) -> SetResult {
        let id = property.PropertyID();
        debug_assert!(D::ShorthandForProperty(id).is_empty());
        let i = if id == CSSPropertyID::kVariable {
            let i = self.FindPropertyIndex(property.CustomPropertyName());
            if i < 0 {
                None
            } else {
                Some(i as usize)
            }
        } else {
            if id == CSSPropertyID::kAll {
                self.RemovePropertiesAffectedByAll();
            }
            self.FindInsertionPointForID(id)
        };
        if let Some(i) = i {
            if self.properties[i] == property {
                return SetResult::kUnchanged;
            }
            self.properties[i] = property;
            self.InvalidateHashIfComputed();
            return SetResult::kModifiedExisting;
        }
        let logical = self.MayHaveLogicalProperties() || kLogicalGroupProperties.Has(id);
        self.SetBit(
            CSSPropertyValueSetState::MayHaveLogicalPropertiesMask,
            logical,
        );
        if id == CSSPropertyID::kAll {
            self.SetBit(CSSPropertyValueSetState::HasAllMask, true);
        }
        self.properties.push(property);
        self.InvalidateHashIfComputed();
        SetResult::kChangedPropertySet
    }
    // cpp: css_property_value_set.cc:627-651. This streamlined overload does
    // not remove all-affected properties and does not perform equality checks.
    pub fn SetLonghandPropertyValue(&mut self, id: CSSPropertyID, value: Rc<CSSValue<D>>) {
        debug_assert!(D::ShorthandForProperty(id).is_empty());
        let property =
            CSSPropertyValue::new(&CSSPropertyName::new(id), value, false, false, 0, false);
        if let Some(i) = self.FindInsertionPointForID(id) {
            self.properties[i] = property;
        } else {
            let logical = self.MayHaveLogicalProperties() || kLogicalGroupProperties.Has(id);
            self.SetBit(
                CSSPropertyValueSetState::MayHaveLogicalPropertiesMask,
                logical,
            );
            if id == CSSPropertyID::kAll {
                self.SetBit(CSSPropertyValueSetState::HasAllMask, true);
            }
            self.properties.push(property);
        }
        self.InvalidateHashIfComputed();
    }
    // cpp: css_property_value_set.cc:653-660
    pub fn SetLonghandPropertyIdentifier(
        &mut self,
        id: CSSPropertyID,
        identifier: CSSValueID,
        important: bool,
    ) -> SetResult {
        self.SetLonghandProperty(CSSPropertyValue::new(
            &CSSPropertyName::new(id),
            D::CreateIdentifier(identifier),
            important,
            false,
            0,
            false,
        ))
    }
    // cpp: css_property_value_set.cc:522-552
    pub fn SetPropertyName(
        &mut self,
        name: &CSSPropertyName,
        value: Rc<CSSValue<D>>,
        important: bool,
    ) {
        if name.IsCustomProperty() {
            self.SetLonghandProperty(CSSPropertyValue::new(
                name, value, important, false, 0, false,
            ));
        } else {
            self.SetProperty(name.Id(), value, important);
        }
    }
    pub fn SetProperty(&mut self, id: CSSPropertyID, value: Rc<CSSValue<D>>, important: bool) {
        debug_assert_ne!(id, CSSPropertyID::kVariable);
        debug_assert_ne!(id, CSSPropertyID::kWhiteSpace);
        let ids = D::ShorthandForProperty(id);
        if ids.is_empty() {
            self.SetLonghandProperty(CSSPropertyValue::new(
                &CSSPropertyName::new(id),
                value,
                important,
                false,
                0,
                false,
            ));
            return;
        }
        self.RemovePropertiesInSet(&ids);
        for id in ids {
            self.properties.push(CSSPropertyValue::new(
                &CSSPropertyName::new(id),
                value.clone(),
                important,
                false,
                0,
                false,
            ));
        }
        self.InvalidateHashIfComputed();
    }
    // cpp: css_property_value_set.cc:680-697. SetResult is converted to bool
    // exactly as C++: every nonzero result, including kUnchanged, is true.
    pub fn AddParsedProperties(&mut self, properties: &[CSSPropertyValue<D>]) -> SetResult {
        let mut changed = SetResult::kUnchanged;
        self.properties.reserve(properties.len());
        for p in properties {
            changed = changed.max(self.SetLonghandProperty(p.clone()));
        }
        changed
    }
    pub fn AddRespectingCascade(&mut self, p: &CSSPropertyValue<D>) -> bool {
        if !self.PropertyIsImportant(p.PropertyID()) || p.IsImportant() {
            self.SetLonghandProperty(p.clone()) as i32 != 0
        } else {
            false
        }
    }
    // cpp: css_property_value_set.cc:703-708,718-724
    pub fn MergeAndOverrideOnConflict(&mut self, other: &CSSPropertyValueSet<D>) {
        for p in other.Properties() {
            self.SetLonghandProperty(p.clone());
        }
    }
    pub fn Clear(&mut self) {
        self.properties.clear();
        self.InvalidateHashIfComputed();
        self.SetBit(
            CSSPropertyValueSetState::MayHaveLogicalPropertiesMask,
            false,
        );
        self.SetBit(CSSPropertyValueSetState::HasAllMask, false);
    }
    // cpp: css_property_value_set.cc:726-767. retain preserves the source
    // stable in-place compaction order and updates the same all bit.
    pub fn RemovePropertiesInSet(&mut self, ids: &[CSSPropertyID]) -> bool {
        if self.properties.is_empty() {
            return false;
        }
        let old = self.properties.len();
        let mut removed_all = false;
        self.properties.retain(|p| {
            if ids.contains(&p.PropertyID()) {
                removed_all |= p.PropertyID() == CSSPropertyID::kAll;
                false
            } else {
                true
            }
        });
        if removed_all {
            self.SetBit(CSSPropertyValueSetState::HasAllMask, false);
        }
        if self.properties.len() != old {
            self.InvalidateHashIfComputed();
            true
        } else {
            false
        }
    }
    // cpp: css_property_value_set.cc:769-792. Unlike CSSPropertyValue's
    // IsAffectedByAll, this calls CSSProperty::Get even for kVariable.
    pub fn RemovePropertiesAffectedByAll(&mut self) -> bool {
        if self.properties.is_empty() {
            return false;
        }
        let old = self.properties.len();
        self.properties
            .retain(|p| !D::IsAffectedByAll(p.PropertyID()));
        if self.properties.len() != old {
            self.InvalidateHashIfComputed();
            true
        } else {
            false
        }
    }
    // cpp: css_property_value_set.cc:828-853
    pub fn RemoveEquivalentProperties(&mut self, style: &CSSPropertyValueSet<D>) {
        let old = self.properties.len();
        let mut removed_all = false;
        self.properties.retain(|p| {
            if p.PropertyID() != CSSPropertyID::kVariable
                && style.PropertyMatches(p.PropertyID(), p.Value())
            {
                removed_all |= p.PropertyID() == CSSPropertyID::kAll;
                false
            } else {
                true
            }
        });
        if removed_all {
            self.SetBit(CSSPropertyValueSetState::HasAllMask, false);
        }
        if self.properties.len() != old {
            self.InvalidateHashIfComputed();
        }
    }
    // cpp: css_property_value_set.cc:855-881
    pub fn RemoveEquivalentPropertiesPreservingShorthands(
        &mut self,
        style: &D::CSSStyleDeclaration,
    ) {
        let mut remove = HashSet::new();
        for p in &self.properties {
            let shorthand = p.ShorthandID();
            if shorthand != CSSPropertyID::kInvalid {
                if !remove.contains(&shorthand)
                    && self.ShorthandPropertyMatchesDeclaration(shorthand, style)
                {
                    remove.insert(shorthand);
                }
                continue;
            }
            if D::DeclarationPropertyMatches(style, p.PropertyID(), p.Value()) {
                remove.insert(p.PropertyID());
            }
        }
        for id in remove {
            self.RemoveProperty(id, None);
        }
    }
    // cpp: css_property_value_set.cc:886-899
    pub fn EnsureCSSStyleDeclaration(
        &mut self,
        context: Option<&D::ExecutionContext>,
    ) -> Rc<D::CSSStyleDeclaration> {
        if let Some(wrapper) = &self.cssom_wrapper_ {
            return wrapper.clone();
        }
        let wrapper = D::NewCSSStyleDeclaration(context, self);
        self.cssom_wrapper_ = Some(wrapper.clone());
        wrapper
    }
    pub fn IntoBase(self) -> CSSPropertyValueSet<D> {
        self.set
    }
}

// The actual CSSParser is used directly. This optional extension constrains its
// associated property/value types to this real store; no parallel parser API.
pub trait CSSPropertyValueSetParserBackend: CSSPropertyValueSetBackend {
    type Parser: crate::parser::css_parser::CSSParserBackend<
        MutablePropertySet = MutableCSSPropertyValueSet<Self>,
        CSSValue = CSSValue<Self>,
        CSSPropertyValue = CSSPropertyValue<Self>,
    >;
    // Source sheet constructor also preserves its SingleOwnerDocument.
    fn DeclarationParserContext(
        sheet: &<Self::Parser as crate::parser::css_parser::CSSParserBackend>::StyleSheetContents,
    ) -> crate::parser::css_parser_context::CSSParserContext<Self::Parser>;
}
#[allow(non_snake_case)]
impl<D: CSSPropertyValueSetParserBackend> MutableCSSPropertyValueSet<D> {
    // cpp: css_property_value_set.cc:484-520
    pub fn ParseAndSetProperty(
        &mut self,
        id: CSSPropertyID,
        value: StringView,
        important: bool,
        secure: crate::parser::css_parser_context::SecureContextMode,
        sheet: Option<
            &<D::Parser as crate::parser::css_parser::CSSParserBackend>::StyleSheetContents,
        >,
    ) -> SetResult {
        if value.IsEmpty() {
            return if self.RemoveProperty(crate::css_property_names::ResolveCSSPropertyID(id), None)
            {
                SetResult::kChangedPropertySet
            } else {
                SetResult::kUnchanged
            };
        }
        crate::parser::css_parser::CSSParser::<D::Parser>::ParseValueWithSheet(
            self, id, value, important, secure, sheet, None,
        )
    }
    pub fn ParseAndSetCustomProperty(
        &mut self,
        name: &AtomicString,
        value: StringView,
        important: bool,
        secure: crate::parser::css_parser_context::SecureContextMode,
        sheet: Option<
            &<D::Parser as crate::parser::css_parser::CSSParserBackend>::StyleSheetContents,
        >,
        animation_tainted: bool,
    ) -> SetResult {
        if value.IsEmpty() {
            return if self.RemoveProperty(name, None) {
                SetResult::kChangedPropertySet
            } else {
                SetResult::kUnchanged
            };
        }
        crate::parser::css_parser::CSSParser::<D::Parser>::ParseValueForCustomProperty(
            self,
            name,
            value,
            important,
            secure,
            sheet,
            animation_tainted,
        )
    }
    // cpp: css_property_value_set.cc:662-678
    pub fn ParseDeclarationList(
        &mut self,
        text: &String,
        secure: crate::parser::css_parser_context::SecureContextMode,
        sheet: Option<
            &<D::Parser as crate::parser::css_parser::CSSParserBackend>::StyleSheetContents,
        >,
    ) {
        self.Clear();
        let context = if let Some(sheet) = sheet {
            let context = D::DeclarationParserContext(sheet);
            context.SetMode(self.CssParserMode());
            context
        } else {
            crate::parser::css_parser_context::CSSParserContext::<D::Parser>::FromMode(
                self.CssParserMode(),
                secure,
                None,
            )
        };
        crate::parser::css_parser::CSSParser::<D::Parser>::ParseDeclarationList(
            &context, self, text,
        );
    }
}

#[cfg(test)]
mod property_store_tests {
    use super::*;
    use crate::css_value::*;
    use foundation::{Length, LengthType};
    use std::cell::RefCell;
    // Fixtures verify only the base dispatcher and expose which required child
    // operation was called. They are not implementations of CSS derived values.
    struct Fixture {
        label: &'static str,
        text: String,
        number: f64,
        hash: u32,
        failed: bool,
        url: bool,
        random: bool,
        list_base: Option<Box<Fixture>>,
        pair_base: Option<Box<Fixture>>,
        scoped: Option<Rc<CSSValue<Dispatch>>>,
        scope_arguments: RefCell<Vec<Option<u32>>>,
        factory_length_type: Option<LengthType>,
    }
    impl Fixture {
        fn new(label: &'static str) -> Self {
            Self {
                label,
                text: String::FromUtf8(label.as_bytes()),
                number: 1.0,
                hash: 0x12345678,
                failed: false,
                url: false,
                random: false,
                list_base: None,
                pair_base: None,
                scoped: None,
                scope_arguments: RefCell::new(Vec::new()),
                factory_length_type: None,
            }
        }
    }
    impl CSSValueSubclass for Fixture {
        fn CustomCSSText(&self) -> String {
            self.text.clone()
        }
        fn Equals(&self, other: &Self) -> bool {
            self.text == other.text && self.number == other.number
        }
    }
    impl CSSValueCustomHash for Fixture {
        fn CustomHash(&self) -> u32 {
            self.hash
        }
    }
    impl CSSValueSubresources for Fixture {
        fn HasFailedOrCanceledSubresources(&self) -> bool {
            self.failed
        }
    }
    impl CSSValueRandom for Fixture {
        fn HasRandomFunctions(&self) -> bool {
            self.random
        }
    }
    impl CSSValueUrl<RefCell<Vec<&'static str>>> for Fixture {
        fn ReResolveUrl(&self, document: &RefCell<Vec<&'static str>>) {
            document.borrow_mut().push(self.label);
        }
    }
    impl CSSValueListUrls for Fixture {
        fn MayContainUrl(&self) -> bool {
            self.url
        }
    }
    impl CSSValueListSubclass<Dispatch> for Fixture {
        fn AsValueList(&self) -> &Fixture {
            self.list_base
                .as_deref()
                .expect("fixture needs the real base-list projection")
        }
    }
    impl CSSValuePairSubclass<Dispatch> for Fixture {
        fn AsValuePair(&self) -> &Fixture {
            self.pair_base
                .as_deref()
                .expect("fixture needs the real base-pair projection")
        }
    }
    impl CSSValueTreeScope<Dispatch> for Fixture {
        fn PopulateWithTreeScope<'a>(&'a self, scope: Option<&'a u32>) -> &'a CSSValue<Dispatch> {
            self.scope_arguments.borrow_mut().push(scope.copied());
            self.scoped
                .as_deref()
                .expect("fixture must supply the actual scoped value")
        }
    }
    struct Dispatch;
    macro_rules! fixture_payloads { ($($name:ident),+ $(,)?) => { $(type $name = Fixture;)+ }; }
    impl CSSValueDispatch for Dispatch {
        type Document = RefCell<Vec<&'static str>>;
        type TreeScope = u32;
        fixture_payloads!(
            CSSNumericLiteralValue,
            CSSMathFunctionValue,
            CSSIdentifierValue,
            CSSScopedKeywordValue,
            CSSColor,
            CSSUnresolvedColorValue,
            CSSColorMixValue,
            CSSAlphaColorValue,
            CSSContrastColorValue,
            CSSCounterValue,
            CSSCounterContentValue,
            CSSQuadValue,
            CSSCustomIdentValue,
            CSSStringValue,
            CSSURIValue,
            CSSURLPatternValue,
            CSSValuePair,
            CSSLightDarkValuePair,
            CSSParamValuePair,
            CSSScrollValue,
            CSSViewValue,
            CSSRatioValue,
            CSSRelativeColorValue,
            CSSBasicShapeCircleValue,
            CSSBasicShapeEllipseValue,
            CSSBasicShapePolygonValue,
            CSSBasicShapeInsetValue,
            CSSBasicShapeRectValue,
            CSSBasicShapeXYWHValue,
            CSSPathValue,
            CSSShapeValue,
            CSSImageValue,
            CSSCursorImageValue,
            CSSCrossfadeValue,
            CSSPaintValue,
            CSSLinearGradientValue,
            CSSRadialGradientValue,
            CSSConicGradientValue,
            CSSConstantGradientValue,
            CSSColorImageValue,
            CSSLinearTimingFunctionValue,
            CSSCubicBezierTimingFunctionValue,
            CSSStepsTimingFunctionValue,
            CSSProgressValue,
            CSSBorderImageSliceValue,
            CSSDynamicRangeLimitMixValue,
            CSSFontFeatureValue,
            CSSFontFaceSrcValue,
            CSSFontFamilyValue,
            CSSFontStyleRangeValue,
            CSSFontVariationValue,
            CSSAlternateValue,
            CSSInheritedValue,
            CSSInitialValue,
            CSSUnsetValue,
            CSSRevertValue,
            CSSRevertLayerValue,
            CSSRevertRuleValue,
            CSSReflectValue,
            CSSShadowValue,
            CSSUnicodeRangeValue,
            CSSGridTemplateAreasValue,
            CSSPaletteMixValue,
            CSSRayValue,
            CSSUnparsedDeclarationValue,
            CSSPendingSubstitutionValue,
            CSSPendingSystemFontValue,
            CSSInvalidVariableValue,
            CSSCyclicVariableValue,
            CSSFlipRevertValue,
            CSSLayoutFunctionValue,
            CSSContentDistributionValue,
            CSSKeyframeShorthandValue,
            CSSInitialColorValue,
            CSSImageSetOptionValue,
            CSSImageSetTypeValue,
            CSSRepeatStyleValue,
            CSSSuperellipseValue,
            CSSSymbolsValue,
            CSSTriggerAttachmentValue,
            CSSRepeatValue,
            CSSValueList,
            CSSFunctionValue,
            CSSImageSetValue,
            CSSBracketedValueList,
            CSSGridAutoRepeatValue,
            CSSGridIntegerRepeatValue,
            CSSAxisValue
        );
        fn CreateIdentifierFromLength(value: &Length) -> Rc<CSSValue<Self>> {
            let mut payload = Fixture::new("identifier factory");
            payload.factory_length_type = Some(value.GetType());
            Rc::new(CSSValue::new(CSSValuePayload::kIdentifierClass(payload)))
        }
        fn CreatePrimitiveFromLength(value: &Length, zoom: f32) -> Rc<CSSValue<Self>> {
            let mut payload = Fixture::new("primitive factory");
            payload.factory_length_type = Some(value.GetType());
            payload.number = f64::from(zoom);
            Rc::new(CSSValue::new(CSSValuePayload::kNumericLiteralClass(
                payload,
            )))
        }
    }

    impl CSSPropertyValueBackend for Dispatch {
        fn MatchingShorthandsForLonghand(id: CSSPropertyID) -> Vec<CSSPropertyID> {
            assert_eq!(id, CSSPropertyID::kMarginTop);
            vec![
                CSSPropertyID::kMargin,
                CSSPropertyID::kMarginBlock,
                CSSPropertyID::kMarginInline,
                CSSPropertyID::kPadding,
            ]
        }
        fn IsAffectedByAll(id: CSSPropertyID) -> bool {
            match id {
                CSSPropertyID::kWidth
                | CSSPropertyID::kHeight
                | CSSPropertyID::kInlineSize
                | CSSPropertyID::kColor
                | CSSPropertyID::kMarginTop
                | CSSPropertyID::kMarginRight
                | CSSPropertyID::kMarginBottom
                | CSSPropertyID::kMarginLeft => true,
                CSSPropertyID::kVariable | CSSPropertyID::kDirection | CSSPropertyID::kAll => false,
                _ => panic!("fixture metadata must explicitly cover the tested property"),
            }
        }
    }
    struct Declaration {
        set: Option<Rc<CSSPropertyValueSet<Dispatch>>>,
        property_match: bool,
    }
    impl CSSPropertyValueSetBackend for Dispatch {
        type CSSStyleDeclaration = Declaration;
        type ExecutionContext = u32;
        fn ShorthandForProperty(id: CSSPropertyID) -> Vec<CSSPropertyID> {
            match id {
                CSSPropertyID::kMargin => vec![
                    CSSPropertyID::kMarginTop,
                    CSSPropertyID::kMarginRight,
                    CSSPropertyID::kMarginBottom,
                    CSSPropertyID::kMarginLeft,
                ],
                CSSPropertyID::kWidth
                | CSSPropertyID::kHeight
                | CSSPropertyID::kInlineSize
                | CSSPropertyID::kColor
                | CSSPropertyID::kVariable
                | CSSPropertyID::kDirection
                | CSSPropertyID::kAll
                | CSSPropertyID::kMarginTop
                | CSSPropertyID::kMarginRight
                | CSSPropertyID::kMarginBottom
                | CSSPropertyID::kMarginLeft => vec![],
                _ => panic!("fixture shorthand metadata must explicitly cover the tested property"),
            }
        }
        fn IsInSameLogicalPropertyGroupWithDifferentMappingLogic(
            a: CSSPropertyID,
            b: CSSPropertyID,
        ) -> bool {
            assert!(a == CSSPropertyID::kWidth || a == CSSPropertyID::kInlineSize);
            matches!(
                (a, b),
                (CSSPropertyID::kWidth, CSSPropertyID::kInlineSize)
                    | (CSSPropertyID::kInlineSize, CSSPropertyID::kWidth)
            )
        }
        fn SerializeShorthand(_: &CSSPropertyValueSet<Self>, id: CSSPropertyID) -> String {
            assert_eq!(id, CSSPropertyID::kMargin);
            String::from("fixture shorthand")
        }
        fn AsText(_: &CSSPropertyValueSet<Self>) -> String {
            String::from("fixture full serialization")
        }
        fn CreateIdentifier(id: CSSValueID) -> Rc<CSSValue<Self>> {
            assert_eq!(id, CSSValueID::kRevert);
            revert()
        }
        fn DeclarationPropertyValueSet(style: &Declaration) -> Option<&CSSPropertyValueSet<Self>> {
            style.set.as_deref()
        }
        fn DeclarationPropertyMatches(
            style: &Declaration,
            _: CSSPropertyID,
            _: &CSSValue<Self>,
        ) -> bool {
            style.property_match
        }
        fn NewCSSStyleDeclaration(
            context: Option<&u32>,
            _: &MutableCSSPropertyValueSet<Self>,
        ) -> Rc<Declaration> {
            assert_eq!(context, Some(&7));
            Rc::new(Declaration {
                set: None,
                property_match: false,
            })
        }
    }
    fn text(value: &'static str) -> Rc<CSSValue<Dispatch>> {
        Rc::new(CSSValue::new(CSSValuePayload::kStringClass(Fixture::new(
            value,
        ))))
    }
    fn revert() -> Rc<CSSValue<Dispatch>> {
        Rc::new(CSSValue::new(CSSValuePayload::kRevertClass(Fixture::new(
            "revert",
        ))))
    }
    fn property(
        id: CSSPropertyID,
        value: Rc<CSSValue<Dispatch>>,
        important: bool,
    ) -> CSSPropertyValue<Dispatch> {
        CSSPropertyValue::new(&CSSPropertyName::new(id), value, important, false, 0, false)
    }
    fn custom(
        name: &AtomicString,
        value: Rc<CSSValue<Dispatch>>,
        important: bool,
    ) -> CSSPropertyValue<Dispatch> {
        CSSPropertyValue::new(
            &CSSPropertyName::custom(name.clone()),
            value,
            important,
            false,
            0,
            false,
        )
    }
    fn ids(set: &CSSPropertyValueSet<Dispatch>) -> Vec<CSSPropertyID> {
        set.Properties()
            .iter()
            .map(CSSPropertyValue::PropertyID)
            .collect()
    }

    #[test]
    fn property_bits_name_shorthand_and_equality_keep_source_metadata_rules() {
        let mut a = CSSPropertyValue::<Dispatch>::new(
            &CSSPropertyName::new(CSSPropertyID::kMarginTop),
            text("a"),
            false,
            true,
            5,
            true,
        );
        assert!(a.IsSetFromShorthand() && a.IsImplicit());
        assert_eq!(a.ShorthandID(), CSSPropertyID::kMarginBlock);
        let negative = CSSPropertyValue::<Dispatch>::new(
            &CSSPropertyName::new(CSSPropertyID::kMarginTop),
            a.ValueRef(),
            false,
            true,
            -1,
            false,
        );
        assert_eq!(negative.ShorthandID(), CSSPropertyID::kPadding);
        assert!(a == negative);
        let other = property(CSSPropertyID::kHeight, a.ValueRef(), false);
        assert!(a == other);
        a.SetImportant();
        assert!(a != other);
        assert_eq!(a.Name().Id(), CSSPropertyID::kMarginTop);
        let name = AtomicString::from_str("--x");
        let p = custom(&name, text("x"), true);
        assert_eq!(p.CustomPropertyName(), &name);
        assert_eq!(p.Name().ToAtomicString(), &name);
        assert!(!p.IsAffectedByAll());
        assert!(property(CSSPropertyID::kWidth, text("w"), false).IsAffectedByAll());
    }
    #[test]
    fn lookup_direction_duplicate_names_and_order_are_preserved() {
        let name = AtomicString::from_str("--x");
        let props = [
            property(CSSPropertyID::kWidth, text("first"), false),
            custom(&name, text("custom first"), false),
            property(CSSPropertyID::kWidth, text("last"), true),
            custom(&name, text("custom last"), true),
        ];
        let mutable = MutableCSSPropertyValueSet::FromProperties(&props);
        let immutable = ImmutableCSSPropertyValueSet::<Dispatch>::Create(
            &props,
            CSSParserMode::kSVGAttributeMode,
            true,
        );
        assert_eq!(mutable.FindPropertyIndex(CSSPropertyID::kWidth), 0);
        assert_eq!(immutable.FindPropertyIndex(CSSPropertyID::kWidth), 2);
        assert_eq!(mutable.FindPropertyIndex(&name), 1);
        assert_eq!(immutable.FindPropertyIndex(&name), 3);
        assert_eq!(
            mutable.GetPropertyValue(CSSPropertyID::kWidth).Utf8(),
            "first"
        );
        assert_eq!(
            immutable.GetPropertyValue(CSSPropertyID::kWidth).Utf8(),
            "last"
        );
        assert!(!mutable.PropertyIsImportant(&name));
        assert!(immutable.PropertyIsImportant(&name));
        assert!(immutable.ContainsCursorHand());
        assert_eq!(immutable.CssParserMode(), CSSParserMode::kSVGAttributeMode);
        assert!(Rc::ptr_eq(&immutable, &immutable.ImmutableCopyIfNeeded()));
        assert_eq!(
            immutable
                .MutableCopy()
                .GetPropertyValue(CSSPropertyID::kWidth)
                .Utf8(),
            "first"
        );
    }
    #[test]
    fn all_removal_custom_reads_and_streamlined_overload_preserve_order() {
        let name = AtomicString::from_str("--x");
        let mut set = MutableCSSPropertyValueSet::<Dispatch>::new(CSSParserMode::kHTMLStandardMode);
        set.SetLonghandProperty(property(CSSPropertyID::kWidth, text("50px"), false));
        set.SetLonghandProperty(custom(&name, text("custom"), false));
        set.SetLonghandProperty(property(CSSPropertyID::kDirection, text("rtl"), false));
        set.SetLonghandProperty(property(CSSPropertyID::kAll, revert(), false));
        assert_eq!(
            ids(&set),
            [
                CSSPropertyID::kVariable,
                CSSPropertyID::kDirection,
                CSSPropertyID::kAll
            ]
        );
        assert_eq!(set.GetPropertyValue(CSSPropertyID::kWidth).Utf8(), "revert");
        assert_eq!(set.GetPropertyValue(&name).Utf8(), "revert"); // source custom overload returns true
        assert_eq!(
            set.GetPropertyCSSValue(&name).unwrap().CssText().Utf8(),
            "custom"
        );
        set.SetLonghandProperty(property(CSSPropertyID::kWidth, text("100px"), false));
        assert_eq!(set.GetPropertyValue(CSSPropertyID::kAll).Utf8(), "");
        assert_eq!(set.GetPropertyValue(CSSPropertyID::kWidth).Utf8(), "100px");
        assert!(set.RemoveProperty(CSSPropertyID::kAll, None));
        assert_eq!(
            ids(&set),
            [CSSPropertyID::kVariable, CSSPropertyID::kDirection]
        );
        assert!(!set.HasAllProperty());
        set.SetLonghandPropertyValue(CSSPropertyID::kWidth, text("before"));
        set.SetLonghandPropertyValue(CSSPropertyID::kAll, revert());
        assert!(set.HasProperty(CSSPropertyID::kWidth));
        assert_eq!(set.GetPropertyValue(CSSPropertyID::kWidth).Utf8(), "revert");
        set.Clear();
        assert!(!set.HasAllProperty());
        assert!(set.GetPropertyValue(CSSPropertyID::kAll).empty());
        assert!(!set.GetPropertyValue(CSSPropertyID::kAll).IsNull());
    }
    #[test]
    fn hash_mutation_stays_invalid_and_copy_computes_fresh_content_hash() {
        let mut set = MutableCSSPropertyValueSet::<Dispatch>::new(CSSParserMode::kSVGAttributeMode);
        let p = property(CSSPropertyID::kWidth, text("50px"), true);
        set.SetLonghandProperty(p.clone());
        let h = set.GetHash();
        let mut expected = 3141592653;
        AddIntToHash(&mut expected, CSSPropertyID::kWidth as u32);
        AddIntToHash(&mut expected, 1);
        AddIntToHash(&mut expected, p.Value().Hash());
        assert_eq!(h, expected);
        assert_eq!(set.SetLonghandProperty(p), SetResult::kUnchanged);
        assert_eq!(set.GetHash(), h);
        set.SetLonghandProperty(property(CSSPropertyID::kWidth, text("100px"), true));
        assert!(set.ModifiedSinceHashing());
        assert_eq!(set.GetHash(), u32::MAX);
        let copy = set.MutableCopy();
        assert!(!copy.ModifiedSinceHashing());
        assert_eq!(copy.GetHash(), set.ComputeHash());
        let base = Rc::new(set.IntoBase());
        let immutable = base.ImmutableCopyIfNeeded();
        assert!(!immutable.IsMutable());
        assert_eq!(immutable.GetHash(), base.ComputeHash());
        assert_eq!(immutable.CssParserMode(), CSSParserMode::kSVGAttributeMode);
        let name = AtomicString::from_str("--x");
        let p = custom(&name, text("x"), false);
        let custom_set = MutableCSSPropertyValueSet::FromProperties(&[p.clone()]);
        let mut hash = 3141592653;
        AddIntToHash(&mut hash, name.Hash());
        AddIntToHash(&mut hash, 0);
        AddIntToHash(&mut hash, p.Value().Hash());
        assert_eq!(custom_set.GetHash(), hash);
        assert_ne!(
            copy.GetHash(),
            MutableCSSPropertyValueSet::FromProperties(&[property(
                CSSPropertyID::kWidth,
                text("100px"),
                false
            )])
            .GetHash()
        );
    }
    #[test]
    fn logical_replacement_cascade_and_bulk_removal_keep_original_order() {
        let mut set = MutableCSSPropertyValueSet::<Dispatch>::new(CSSParserMode::kHTMLStandardMode);
        set.SetLonghandProperty(property(CSSPropertyID::kWidth, text("w1"), false));
        set.SetLonghandProperty(property(CSSPropertyID::kInlineSize, text("logical"), false));
        assert_eq!(
            set.SetLonghandProperty(property(CSSPropertyID::kWidth, text("w2"), false)),
            SetResult::kChangedPropertySet
        );
        assert_eq!(
            ids(&set),
            [CSSPropertyID::kInlineSize, CSSPropertyID::kWidth]
        );
        let p = property(CSSPropertyID::kHeight, text("important"), true);
        assert!(set.AddRespectingCascade(&p));
        assert!(set.AddRespectingCascade(&p));
        assert!(!set.AddRespectingCascade(&property(
            CSSPropertyID::kHeight,
            text("normal"),
            false
        )));
        let name = AtomicString::from_str("--keep");
        set.SetLonghandProperty(custom(&name, text("custom"), false));
        let style = MutableCSSPropertyValueSet::FromProperties(set.Properties());
        set.RemoveEquivalentProperties(&style);
        assert_eq!(ids(&set), [CSSPropertyID::kVariable]);
        let mut returned = String::from("old");
        assert!(set.RemoveProperty(&name, Some(&mut returned)));
        assert_eq!(returned.Utf8(), "custom");
        assert!(!set.RemoveProperty(&name, Some(&mut returned)));
        assert!(returned.empty());
        assert!(!returned.IsNull());
    }
    #[test]
    fn shorthand_expansion_copy_subresources_and_cssom_use_required_dependencies() {
        let mut set = MutableCSSPropertyValueSet::<Dispatch>::new(CSSParserMode::kSVGAttributeMode);
        set.SetProperty(CSSPropertyID::kMargin, text("m"), true);
        assert!(set.ShorthandIsImportant(CSSPropertyID::kMargin));
        assert_eq!(
            set.GetPropertyValue(CSSPropertyID::kMargin).Utf8(),
            "fixture shorthand"
        );
        assert_eq!(set.AsText().Utf8(), "fixture full serialization");
        let copy = set.CopyPropertiesInSet(&[
            CSSPropertyID::kMarginLeft,
            CSSPropertyID::kWidth,
            CSSPropertyID::kMarginTop,
        ]);
        assert_eq!(
            ids(&copy),
            [CSSPropertyID::kMarginLeft, CSSPropertyID::kMarginTop]
        );
        assert_eq!(copy.CssParserMode(), CSSParserMode::kHTMLStandardMode);
        assert!(!copy.PropertyIsImportant(CSSPropertyID::kMarginTop));
        assert!(set.RemoveProperty(CSSPropertyID::kMargin, None));
        assert!(set.IsEmpty());
        let mut image = Fixture::new("failed image");
        image.failed = true;
        let image = Rc::new(CSSValue::new(CSSValuePayload::kImageClass(image)));
        set.SetLonghandProperty(property(CSSPropertyID::kWidth, image, false));
        assert!(set.HasFailedOrCanceledSubresources());
        let first = set.EnsureCSSStyleDeclaration(Some(&7));
        let second = set.EnsureCSSStyleDeclaration(None);
        assert!(Rc::ptr_eq(&first, &second));
        let style = Declaration {
            set: None,
            property_match: true,
        };
        set.RemoveEquivalentPropertiesPreservingShorthands(&style);
        assert!(set.IsEmpty());
    }
    #[test]
    fn rule_adapter_retains_mutable_subclass_and_existing_value_identity() {
        let original = text("original");
        let immutable = ImmutableCSSPropertyValueSet::<Dispatch>::Create(
            &[property(CSSPropertyID::kWidth, original.clone(), false)],
            CSSParserMode::kHTMLStandardMode,
            false,
        );
        let base = CSSPropertyValueSetRuleHandle::FromImmutable(immutable);
        assert!(!base.IsMutable());
        assert!(base.MutablePropertySet().is_none());
        let copy = base.MutableCopy();
        let mutable = CSSPropertyValueSetRuleHandle::FromMutable(copy.clone());
        assert!(mutable.IsMutable());
        assert!(Rc::ptr_eq(
            &mutable.GetPropertyCSSValue(CSSPropertyID::kWidth).unwrap(),
            &original
        ));
        assert!(Rc::ptr_eq(&mutable.MutablePropertySet().unwrap(), &copy));
        copy.borrow_mut().SetLonghandProperty(property(
            CSSPropertyID::kWidth,
            text("edited"),
            false,
        ));
        assert_eq!(
            mutable
                .GetPropertyCSSValue(CSSPropertyID::kWidth)
                .unwrap()
                .CssText()
                .Utf8(),
            "edited"
        );
        assert_eq!(
            base.GetPropertyCSSValue(CSSPropertyID::kWidth)
                .unwrap()
                .CssText()
                .Utf8(),
            "original"
        );
    }
}

// Explicit rule-layer ownership interface. Chromium's base Member can point
// to a mutable subclass; a mutable copy must retain that editable subclass.
// GetPropertyCSSValue clones only the existing GC-equivalent Rc reference.
pub trait CSSPropertyValueSetRuleAdapter: Sized {
    type MutablePropertySet;
    type CSSValue;
    fn IsMutable(&self) -> bool;
    fn MutableCopy(&self) -> Rc<std::cell::RefCell<Self::MutablePropertySet>>;
    fn FromMutable(set: Rc<std::cell::RefCell<Self::MutablePropertySet>>) -> Rc<Self>;
    fn MutablePropertySet(&self) -> Option<Rc<std::cell::RefCell<Self::MutablePropertySet>>>;
    fn GetPropertyCSSValue(&self, id: CSSPropertyID) -> Option<Rc<Self::CSSValue>>;
    fn HasFailedOrCanceledSubresources(&self) -> bool;
}

pub enum CSSPropertyValueSetRuleHandle<D: CSSPropertyValueSetBackend> {
    Immutable(Rc<CSSPropertyValueSet<D>>),
    Mutable(Rc<std::cell::RefCell<MutableCSSPropertyValueSet<D>>>),
}
#[allow(non_snake_case)]
impl<D: CSSPropertyValueSetBackend> CSSPropertyValueSetRuleHandle<D> {
    pub fn FromImmutable(set: Rc<CSSPropertyValueSet<D>>) -> Rc<Self> {
        assert!(!set.IsMutable());
        Rc::new(Self::Immutable(set))
    }

    // cpp: css_property_value_set.cc:61-73. The rule-layer handle retains
    // identity for immutable stores and snapshots mutable stores.
    pub fn ImmutableCopyIfNeeded(self: &Rc<Self>) -> Rc<Self> {
        match self.as_ref() {
            Self::Immutable(_) => self.clone(),
            Self::Mutable(set) => {
                let set = set.borrow();
                Self::FromImmutable(ImmutableCSSPropertyValueSet::Create(
                    set.Properties(),
                    set.CssParserMode(),
                    false,
                ))
            }
        }
    }
}
#[allow(non_snake_case)]
impl<D: CSSPropertyValueSetBackend> CSSPropertyValueSetRuleAdapter
    for CSSPropertyValueSetRuleHandle<D>
{
    type MutablePropertySet = MutableCSSPropertyValueSet<D>;
    type CSSValue = CSSValue<D>;
    fn IsMutable(&self) -> bool {
        matches!(self, Self::Mutable(_))
    }
    fn MutableCopy(&self) -> Rc<std::cell::RefCell<Self::MutablePropertySet>> {
        let copy = match self {
            Self::Immutable(set) => set.MutableCopy(),
            Self::Mutable(set) => set.borrow().MutableCopy(),
        };
        Rc::new(std::cell::RefCell::new(copy))
    }
    fn FromMutable(set: Rc<std::cell::RefCell<Self::MutablePropertySet>>) -> Rc<Self> {
        Rc::new(Self::Mutable(set))
    }
    fn MutablePropertySet(&self) -> Option<Rc<std::cell::RefCell<Self::MutablePropertySet>>> {
        match self {
            Self::Immutable(_) => None,
            Self::Mutable(set) => Some(set.clone()),
        }
    }
    fn GetPropertyCSSValue(&self, id: CSSPropertyID) -> Option<Rc<Self::CSSValue>> {
        match self {
            Self::Immutable(set) => set.GetPropertyValueRef(id),
            Self::Mutable(set) => set.borrow().GetPropertyValueRef(id),
        }
    }
    fn HasFailedOrCanceledSubresources(&self) -> bool {
        match self {
            Self::Immutable(set) => set.HasFailedOrCanceledSubresources(),
            Self::Mutable(set) => set.borrow().HasFailedOrCanceledSubresources(),
        }
    }
}
