// Translation status: complete
// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/trees/target_property.h:1-46.
use crate::ElementId;
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Type {
    TRANSFORM = 0,
    SCALE,
    ROTATE,
    TRANSLATE,
    OPACITY,
    FILTER,
    SCROLL_OFFSET,
    BACKGROUND_COLOR,
    BOUNDS,
    CSS_CUSTOM_PROPERTY,
    NATIVE_PROPERTY,
    BACKDROP_FILTER,
}
pub const FIRST_TARGET_PROPERTY: Type = Type::TRANSFORM;
pub const LAST_TARGET_PROPERTY: Type = Type::BACKDROP_FILTER;
pub const ALL_TARGET_PROPERTIES: [Type; 12] = [
    Type::TRANSFORM,
    Type::SCALE,
    Type::ROTATE,
    Type::TRANSLATE,
    Type::OPACITY,
    Type::FILTER,
    Type::SCROLL_OFFSET,
    Type::BACKGROUND_COLOR,
    Type::BOUNDS,
    Type::CSS_CUSTOM_PROPERTY,
    Type::NATIVE_PROPERTY,
    Type::BACKDROP_FILTER,
];
pub type PropertyToElementIdMap = std::collections::BTreeMap<Type, ElementId>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TargetProperties(pub u32);

impl TargetProperties {
    pub fn test(self, index: usize) -> bool {
        assert!(index < 32);
        self.0 & (1u32 << index) != 0
    }
    pub fn set(&mut self, index: usize, value: bool) {
        assert!(index < 32);
        if value {
            self.0 |= 1u32 << index;
        } else {
            self.0 &= !(1u32 << index);
        }
    }
    pub fn reset(&mut self) {
        self.0 = 0;
    }
    pub fn all(self) -> bool {
        self.0 == u32::MAX
    }
}

impl std::ops::BitOr for TargetProperties {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
impl std::ops::BitXor for TargetProperties {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }
}
impl std::ops::BitAnd for TargetProperties {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}
impl std::ops::Not for TargetProperties {
    type Output = Self;
    fn not(self) -> Self {
        Self(!self.0)
    }
}
// Source: cc/trees/target_property.cc:1-25.
const _: () = assert!(
    (LAST_TARGET_PROPERTY as usize) < 32,
    "The number of cc target properties has exceeded the capacity of TargetProperties"
);
const _: () = assert!(
    32 % (8 * std::mem::size_of::<u32>()) == 0,
    "The maximum number of target properties should be a multiple of sizeof(uint32_t)"
);
