/*
 * (C) 1999-2003 Lars Knoll (knoll@kde.org)
 * Copyright (C) 2004, 2005, 2006 Apple Computer, Inc.
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

// cpp: third_party/blink/renderer/core/css/css_property_value.h:33-85,89-102
// cpp: third_party/blink/renderer/core/css/css_property_value.cc:41-64
#![allow(non_snake_case)]
use crate::css_property_name::CSSPropertyName;
use crate::css_value::{CSSValue, CSSValueDispatch};
use foundation::{kCSSPropertyIDBitLength, AtomicString, CSSPropertyID, ConvertToCSSPropertyID};
use std::rc::Rc;

// Required original generated shorthand metadata, and the unresolved
// CSSProperty::IsAffectedByAll operation. No synthesized property tables.
pub trait CSSPropertyValueBackend: CSSValueDispatch {
    fn MatchingShorthandsForLonghand(property: CSSPropertyID) -> Vec<CSSPropertyID>;
    fn IsAffectedByAll(property: CSSPropertyID) -> bool;
}

pub struct CSSPropertyValue<D: CSSPropertyValueBackend> {
    custom_name_: AtomicString,
    bits_: u32,
    value_: Rc<CSSValue<D>>,
}
impl<D: CSSPropertyValueBackend> Clone for CSSPropertyValue<D> {
    fn clone(&self) -> Self {
        Self {
            custom_name_: self.custom_name_.clone(),
            bits_: self.bits_,
            value_: self.value_.clone(),
        }
    }
}
impl<D: CSSPropertyValueBackend> CSSPropertyValue<D> {
    const PROPERTY_MASK: u32 = (1 << kCSSPropertyIDBitLength) - 1;
    const SHORTHAND_SHIFT: usize = kCSSPropertyIDBitLength;
    const INDEX_SHIFT: usize = kCSSPropertyIDBitLength + 1;
    const IMPORTANT_SHIFT: usize = kCSSPropertyIDBitLength + 3;
    const IMPLICIT_SHIFT: usize = kCSSPropertyIDBitLength + 4;
    // cpp: css_property_value.h:37-53. Assignment into the C++ 2-bit
    // unsigned shorthand index truncates to two bits, including negative ints.
    pub fn new(
        name: &CSSPropertyName,
        value: Rc<CSSValue<D>>,
        important: bool,
        is_set_from_shorthand: bool,
        index_in_shorthands_vector: i32,
        implicit: bool,
    ) -> Self {
        Self {
            custom_name_: if name.IsCustomProperty() {
                name.ToAtomicString().clone()
            } else {
                AtomicString::default()
            },
            bits_: (name.Id() as u32 & Self::PROPERTY_MASK)
                | ((is_set_from_shorthand as u32) << Self::SHORTHAND_SHIFT)
                | ((index_in_shorthands_vector as u32 & 3) << Self::INDEX_SHIFT)
                | ((important as u32) << Self::IMPORTANT_SHIFT)
                | ((implicit as u32) << Self::IMPLICIT_SHIFT),
            value_: value,
        }
    }
    pub fn PropertyID(&self) -> CSSPropertyID {
        ConvertToCSSPropertyID((self.bits_ & Self::PROPERTY_MASK) as i32)
    }
    pub fn CustomPropertyName(&self) -> &AtomicString {
        debug_assert_eq!(self.PropertyID(), CSSPropertyID::kVariable);
        &self.custom_name_
    }
    pub fn IsSetFromShorthand(&self) -> bool {
        self.bits_ & (1 << Self::SHORTHAND_SHIFT) != 0
    }
    pub fn ShorthandID(&self) -> CSSPropertyID {
        if !self.IsSetFromShorthand() {
            return CSSPropertyID::kInvalid;
        }
        let shorthands = D::MatchingShorthandsForLonghand(self.PropertyID());
        shorthands[((self.bits_ >> Self::INDEX_SHIFT) & 3) as usize]
    }
    pub fn IsImportant(&self) -> bool {
        self.bits_ & (1 << Self::IMPORTANT_SHIFT) != 0
    }
    pub fn SetImportant(&mut self) {
        self.bits_ |= 1 << Self::IMPORTANT_SHIFT;
    }
    pub fn IsImplicit(&self) -> bool {
        self.bits_ & (1 << Self::IMPLICIT_SHIFT) != 0
    }
    pub fn IsAffectedByAll(&self) -> bool {
        self.PropertyID() != CSSPropertyID::kVariable && D::IsAffectedByAll(self.PropertyID())
    }
    pub fn Name(&self) -> CSSPropertyName {
        if self.PropertyID() != CSSPropertyID::kVariable {
            CSSPropertyName::new(self.PropertyID())
        } else {
            CSSPropertyName::custom(self.custom_name_.clone())
        }
    }
    pub fn Value(&self) -> &CSSValue<D> {
        &self.value_
    }
    // Rust ownership accessor for retaining the actual source Member pointer.
    pub fn ValueRef(&self) -> Rc<CSSValue<D>> {
        self.value_.clone()
    }
}
// Source equality intentionally ignores name/shorthand/implicit metadata.
impl<D: CSSPropertyValueBackend> PartialEq for CSSPropertyValue<D> {
    fn eq(&self, other: &Self) -> bool {
        (Rc::ptr_eq(&self.value_, &other.value_) || self.value_.as_ref() == other.value_.as_ref())
            && self.IsImportant() == other.IsImportant()
    }
}
