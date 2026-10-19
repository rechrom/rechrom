// Translation status: complete
// Copyright 2017 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Source: cc/input/overscroll_behavior.h:1-11,53-56.
// Rust module scope replaces header guards/export macros and namespace cc.

// Source: cc/input/overscroll_behavior.h:13-23,25.
// The nested C++ enum lives in this file's module because Rust structs have no
// nested type declarations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Type {
    // Same as contain but also hint that no overscroll affordance should be
    // triggered.
    kNone,
    // Allows the default behavior for the user agent.
    kAuto,
    // Hint to disable scroll chaining. The user agent may show an appropriate
    // overscroll affordance.
    kContain,
    // Disables the overscroll affordance but allows scroll chaining.
    kChain,
}

// Source: cc/input/overscroll_behavior.h:24.
impl Type {
    pub const kMax: Self = Self::kChain;
}

// Source: cc/input/overscroll_behavior.h:12,44-46,52.
#[derive(Clone, Copy, Debug)]
pub struct OverscrollBehavior {
    pub x: Type,
    pub y: Type,
}

impl OverscrollBehavior {
    // Source: cc/input/overscroll_behavior.h:27.
    pub const fn new() -> Self {
        Self {
            x: Type::kAuto,
            y: Type::kAuto,
        }
    }

    // Source: cc/input/overscroll_behavior.h:28.
    // C++ constructor overloading becomes an explicitly named Rust constructor.
    pub const fn FromType(r#type: Type) -> Self {
        Self {
            x: r#type,
            y: r#type,
        }
    }

    // Source: cc/input/overscroll_behavior.h:29-30.
    pub const fn FromAxes(x_type: Type, y_type: Type) -> Self {
        Self {
            x: x_type,
            y: y_type,
        }
    }

    // Source: cc/input/overscroll_behavior.h:31-34.
    // Helpers for common functionality. Prefer these over direct x/y access.
    pub fn HasXLocalBorderEffects(&self) -> bool {
        self.x == Type::kAuto || self.x == Type::kContain
    }

    // Source: cc/input/overscroll_behavior.h:35-37.
    pub fn HasYLocalBorderEffects(&self) -> bool {
        self.y == Type::kAuto || self.y == Type::kContain
    }

    // Source: cc/input/overscroll_behavior.h:38-40.
    pub fn PropagatesXScroll(&self) -> bool {
        self.x == Type::kAuto || self.x == Type::kChain
    }

    // Source: cc/input/overscroll_behavior.h:41-43.
    pub fn PropagatesYScroll(&self) -> bool {
        self.y == Type::kAuto || self.y == Type::kChain
    }
}

// Source: cc/input/overscroll_behavior.h:48-50.
impl PartialEq for OverscrollBehavior {
    fn eq(&self, a: &Self) -> bool {
        (a.x == self.x) && (a.y == self.y)
    }

    // Source: cc/input/overscroll_behavior.h:51.
    fn ne(&self, a: &Self) -> bool {
        !(self == a)
    }
}

// Source: cc/input/overscroll_behavior.h:27.
// Default is the Rust spelling of the no-argument C++ constructor.
impl Default for OverscrollBehavior {
    fn default() -> Self {
        Self::new()
    }
}
