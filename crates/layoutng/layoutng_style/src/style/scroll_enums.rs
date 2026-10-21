// cpp: layoutng_style/style/scroll_enums.h:5-21
pub mod mojom {
    /// Scroll origin. A transparent i32 allows the C++ kMinValue/kMaxValue aliases.
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct ScrollType(i32);

    #[allow(non_upper_case_globals)]
    impl ScrollType {
        pub const kUser: Self = Self(0);
        pub const kProgrammatic: Self = Self(1);
        pub const kClamping: Self = Self(2);
        pub const kCompositor: Self = Self(3);
        pub const kAnchoring: Self = Self(4);
        pub const kScrollStart: Self = Self(5);
        pub const kMinValue: Self = Self(0);
        pub const kMaxValue: Self = Self(5);
        pub const fn from_i32(value: i32) -> Self {
            Self(value)
        }
        pub const fn value(self) -> i32 {
            self.0
        }
    }

    // cpp: layoutng_style/style/scroll_enums.h:22-31
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct ScrollBehavior(i32);

    #[allow(non_upper_case_globals)]
    impl ScrollBehavior {
        pub const kAuto: Self = Self(0);
        pub const kInstant: Self = Self(1);
        pub const kSmooth: Self = Self(2);
        pub const kMinValue: Self = Self(0);
        pub const kMaxValue: Self = Self(2);
        pub const fn from_i32(value: i32) -> Self {
            Self(value)
        }
        pub const fn value(self) -> i32 {
            self.0
        }
    }

    // cpp: layoutng_style/style/scroll_enums.h:33-36
    pub mod blink {
        pub use super::{ScrollBehavior, ScrollType};
    }
}
