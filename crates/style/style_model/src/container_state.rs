// Copyright 2023 The Chromium Authors. BSD-style license; see Chromium LICENSE.
// cpp: third_party/blink/renderer/core/css/container_state.h:13-28
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContainerStuckLogical {
    kNo,
    kStart,
    kEnd,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContainerStuckPhysical {
    kNo,
    kLeft,
    kRight,
    kTop,
    kBottom,
}
// cpp: third_party/blink/renderer/core/css/container_state.h:43-60,81-85
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContainerSnapped {
    kNone = 0,
    kX = 1 << 0,
    kY = 1 << 1,
}
pub type ContainerSnappedFlags = u32;
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContainerScrollable {
    kNone = 0,
    kStart = 1 << 0,
    kEnd = 1 << 1,
}
pub type ContainerScrollableFlags = u32;
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContainerScrolled {
    kNone = 0,
    kStart = 1 << 0,
    kEnd = 1 << 1,
}

pub trait Flippable: Sized {
    fn Flip(self) -> Self;
}
// One generic function represents the three C++ overloads.
pub fn Flip<T: Flippable>(value: T) -> T {
    value.Flip()
}
impl Flippable for ContainerStuckLogical {
    // cpp: third_party/blink/renderer/core/css/container_state.h:30-39
    fn Flip(self) -> Self {
        match self {
            Self::kNo => Self::kNo,
            Self::kStart => Self::kEnd,
            Self::kEnd => Self::kStart,
        }
    }
}
impl Flippable for ContainerScrolled {
    // cpp: third_party/blink/renderer/core/css/container_state.h:87-96
    fn Flip(self) -> Self {
        match self {
            Self::kNone => Self::kNone,
            Self::kStart => Self::kEnd,
            Self::kEnd => Self::kStart,
        }
    }
}
impl Flippable for ContainerScrollableFlags {
    // cpp: third_party/blink/renderer/core/css/container_state.h:62-79
    fn Flip(self) -> Self {
        if self == ContainerScrollable::kNone as u32 {
            return self;
        }
        let mut flipped = ContainerScrollable::kNone as u32;
        if self & ContainerScrollable::kStart as u32 != 0 {
            flipped |= ContainerScrollable::kEnd as u32;
        }
        if self & ContainerScrollable::kEnd as u32 != 0 {
            flipped |= ContainerScrollable::kStart as u32;
        }
        flipped
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn flip_matches_axis_and_mask_semantics() {
        assert_eq!(Flip(ContainerStuckLogical::kNo), ContainerStuckLogical::kNo);
        assert_eq!(
            Flip(ContainerStuckLogical::kStart),
            ContainerStuckLogical::kEnd
        );
        assert_eq!(Flip(ContainerScrolled::kEnd), ContainerScrolled::kStart);
        for (input, expected) in [
            (0_u32, 0),
            (1, 2),
            (2, 1),
            (3, 3),
            (4, 0),
            (5, 2),
            (6, 1),
            (7, 3),
        ] {
            assert_eq!(Flip(input), expected);
        }
    }
}
