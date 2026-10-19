// Translation status: complete
// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/trees/target_property.h:1-46.
use crate::adapter::input_types::ElementId;
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
// Source: cc/trees/target_property.cc:1-25.
const _: () = assert!(
    (LAST_TARGET_PROPERTY as usize) < 32,
    "The number of cc target properties has exceeded the capacity of TargetProperties"
);
const _: () = assert!(
    32 % (8 * std::mem::size_of::<u32>()) == 0,
    "The maximum number of target properties should be a multiple of sizeof(uint32_t)"
);
