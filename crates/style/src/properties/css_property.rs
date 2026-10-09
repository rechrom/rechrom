// Copyright 2019 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// cpp: third_party/blink/renderer/core/css/properties/css_property.h:28-36,
//      74-130, 214-333

use crate::css_property_name::CSSPropertyName;
use crate::css_property_names::{GetJSPropertyName, GetPropertyName, GetPropertyNameAtomicString};
use crate::properties::css_property_instances::{
    kPropertyUnvisitedIDs, kPropertyVisitedIDs, GetPropertyInternal,
};
use foundation::{kLastCSSProperty, AtomicString, CSSPropertyID, ConvertToCSSPropertyID};

// Determines how far to process a value requested from a computed style.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CSSValuePhase {
    kComputedValue,
    kResolvedValue,
}

pub type Flags = u64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CSSProperty {
    // Chromium stores these in bit fields with the same widths.
    property_id_: u16,
    repetition_separator_: u8,
    flags_: Flags,
}

impl CSSProperty {
    // cpp: css_property.h:49-64
    pub fn Get(id: CSSPropertyID) -> &'static CSSProperty {
        assert!(id > CSSPropertyID::kInvalid && id <= kLastCSSProperty);
        GetPropertyInternal(id)
            .AsResolvedProperty()
            .expect("resolved CSSPropertyID must address a resolved property")
    }

    // cpp: css_property.h:216-302
    pub const kInterpolable: Flags = 1 << 0;
    pub const kCompositableProperty: Flags = 1 << 1;
    pub const kDescriptor: Flags = 1 << 2;
    pub const kProperty: Flags = 1 << 3;
    pub const kShorthand: Flags = 1 << 4;
    pub const kLonghand: Flags = 1 << 5;
    pub const kInherited: Flags = 1 << 6;
    pub const kVisited: Flags = 1 << 7;
    pub const kNotVisited: Flags = 1u64 << 33;
    pub const kInternal: Flags = 1 << 8;
    pub const kAnimation: Flags = 1 << 9;
    pub const kNotAnimation: Flags = 1u64 << 34;
    pub const kValidForFirstLetter: Flags = 1 << 10;
    pub const kValidForCue: Flags = 1 << 11;
    pub const kValidForMarker: Flags = 1 << 12;
    pub const kSurrogate: Flags = 1 << 13;
    pub const kAffectsFont: Flags = 1 << 14;
    pub const kBackground: Flags = 1 << 15;
    pub const kBorder: Flags = 1 << 16;
    pub const kBorderRadius: Flags = 1 << 17;
    pub const kValidForHighlightLegacy: Flags = 1 << 18;
    pub const kInLogicalPropertyGroup: Flags = 1 << 19;
    pub const kValidForFirstLine: Flags = 1 << 20;
    pub const kHighlightColors: Flags = 1 << 21;
    pub const kVisitedHighlightColors: Flags = 1 << 22;
    pub const kSupportsIncrementalStyle: Flags = 1 << 23;
    pub const kIdempotent: Flags = 1 << 24;
    pub const kOverlapping: Flags = 1 << 25;
    pub const kLegacyOverlapping: Flags = 1 << 26;
    pub const kNotLegacyOverlapping: Flags = 1u64 << 35;
    pub const kValidForKeyframe: Flags = 1 << 27;
    pub const kValidForPositionTry: Flags = 1 << 28;
    pub const kValidForHighlight: Flags = 1u64 << 29;
    pub const kAcceptsNumericLiteral: Flags = 1u64 << 30;
    pub const kValidForPermissionElement: Flags = 1u64 << 31;
    pub const kValidForPageContext: Flags = 1u64 << 32;
    pub const kValidForVisited: Flags = 1u64 << 36;
    pub const kValidForPermissionIcon: Flags = 1u64 << 37;
    pub const kPercentagesDependOnUsedValue: Flags = 1u64 << 38;
    pub const kPercentagesDoNotDependOnUsedValue: Flags = 1u64 << 39;

    // cpp: css_property.h:304-316
    pub fn new(property_id: CSSPropertyID, flags: Flags, repetition_separator: char) -> Self {
        debug_assert_ne!(flags & Self::kVisited, flags & Self::kNotVisited);
        debug_assert_ne!(flags & Self::kAnimation, flags & Self::kNotAnimation);
        debug_assert_ne!(
            flags & Self::kLegacyOverlapping,
            flags & Self::kNotLegacyOverlapping
        );
        debug_assert!(repetition_separator.is_ascii());
        Self {
            property_id_: property_id as u16,
            repetition_separator_: repetition_separator as u8,
            flags_: flags,
        }
    }

    // cpp: css_property.h:74-130
    pub fn PropertyID(&self) -> CSSPropertyID {
        // SAFETY: the only constructor accepts a CSSPropertyID and stores its
        // generated, contiguous repr value losslessly in Chromium's 16 bits.
        unsafe { std::mem::transmute::<i32, CSSPropertyID>(self.property_id_ as i32) }
    }
    pub fn GetCSSPropertyName(&self) -> CSSPropertyName {
        CSSPropertyName::new(self.PropertyID())
    }
    pub fn HasEqualCSSPropertyName(&self, other: &CSSProperty) -> bool {
        self.GetCSSPropertyName() == other.GetCSSPropertyName()
    }
    pub fn GetPropertyName(&self) -> &'static str {
        GetPropertyName(self.PropertyID())
    }
    pub fn GetPropertyNameAtomicString(&self) -> &'static AtomicString {
        GetPropertyNameAtomicString(self.PropertyID())
    }
    pub fn GetJSPropertyName(&self) -> &'static str {
        GetJSPropertyName(self.PropertyID())
    }
    pub fn IDEquals(&self, id: CSSPropertyID) -> bool {
        self.property_id_ == id as u16
    }
    pub fn IsResolvedProperty(&self) -> bool {
        true
    }
    pub fn GetFlags(&self) -> Flags {
        self.flags_
    }
    pub fn IsInterpolable(&self) -> bool {
        self.has(Self::kInterpolable)
    }
    pub fn IsCompositableProperty(&self) -> bool {
        self.has(Self::kCompositableProperty)
    }
    pub fn IsDescriptor(&self) -> bool {
        self.has(Self::kDescriptor)
    }
    pub fn IsProperty(&self) -> bool {
        self.has(Self::kProperty)
    }
    pub fn IsShorthand(&self) -> bool {
        self.has(Self::kShorthand)
    }
    pub fn IsLonghand(&self) -> bool {
        self.has(Self::kLonghand)
    }
    pub fn IsInherited(&self) -> bool {
        self.has(Self::kInherited)
    }
    pub fn IsVisited(&self) -> bool {
        self.has(Self::kVisited)
    }
    pub fn IsInternal(&self) -> bool {
        self.has(Self::kInternal)
    }
    pub fn IsAnimationProperty(&self) -> bool {
        self.has(Self::kAnimation)
    }
    pub fn SupportsIncrementalStyle(&self) -> bool {
        self.has(Self::kSupportsIncrementalStyle)
    }
    pub fn IsIdempotent(&self) -> bool {
        self.has(Self::kIdempotent)
    }
    pub fn AcceptsNumericLiteral(&self) -> bool {
        self.has(Self::kAcceptsNumericLiteral)
    }
    pub fn IsValidForFirstLetter(&self) -> bool {
        self.has(Self::kValidForFirstLetter)
    }
    pub fn IsValidForFirstLine(&self) -> bool {
        self.has(Self::kValidForFirstLine)
    }
    pub fn IsValidForCue(&self) -> bool {
        self.has(Self::kValidForCue)
    }
    pub fn IsValidForMarker(&self) -> bool {
        self.has(Self::kValidForMarker)
    }
    pub fn IsValidForKeyframe(&self) -> bool {
        self.has(Self::kValidForKeyframe)
    }
    pub fn IsValidForPositionTry(&self) -> bool {
        self.has(Self::kValidForPositionTry)
    }
    pub fn IsSurrogate(&self) -> bool {
        self.has(Self::kSurrogate)
    }
    pub fn AffectsFont(&self) -> bool {
        self.has(Self::kAffectsFont)
    }
    pub fn IsBackground(&self) -> bool {
        self.has(Self::kBackground)
    }
    pub fn IsBorder(&self) -> bool {
        self.has(Self::kBorder)
    }
    pub fn IsBorderRadius(&self) -> bool {
        self.has(Self::kBorderRadius)
    }
    pub fn IsInLogicalPropertyGroup(&self) -> bool {
        self.has(Self::kInLogicalPropertyGroup)
    }
    pub fn IsRepeated(&self) -> bool {
        self.repetition_separator_ != 0
    }
    pub fn RepetitionSeparator(&self) -> char {
        self.repetition_separator_ as char
    }
    pub fn IsLayoutDependentProperty(&self) -> bool {
        false
    }
    pub fn PercentagesDependOnUsedValue(&self) -> bool {
        self.has(Self::kPercentagesDependOnUsedValue)
    }
    pub fn PercentagesDoNotDependOnUsedValue(&self) -> bool {
        self.has(Self::kPercentagesDoNotDependOnUsedValue)
    }

    // cpp: css_property.h:182-202
    pub fn GetVisitedProperty(&self) -> Option<&'static CSSProperty> {
        let visited_id =
            ConvertToCSSPropertyID(kPropertyVisitedIDs[self.property_id_ as usize] as i32);
        (visited_id != CSSPropertyID::kInvalid).then(|| Self::Get(visited_id))
    }

    pub fn UnvisitedID(id: usize) -> CSSPropertyID {
        ConvertToCSSPropertyID(kPropertyUnvisitedIDs[id] as i32)
    }

    pub fn GetUnvisitedProperty(&self) -> Option<&'static CSSProperty> {
        let unvisited_id = Self::UnvisitedID(self.property_id_ as usize);
        (unvisited_id != CSSPropertyID::kInvalid).then(|| Self::Get(unvisited_id))
    }

    fn has(&self, flag: Flags) -> bool {
        self.flags_ & flag != 0
    }
}

// cpp: css_property.h:318-325
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ValueMode {
    kNormal = 0,
    kAnimated = 1 << 0,
    kAttrTainted = 1 << 1,
}

pub type ValueModeFlags = u8;
