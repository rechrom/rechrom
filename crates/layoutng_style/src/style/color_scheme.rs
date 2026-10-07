// cpp: layoutng_style/style/color_scheme.h:5-13
pub mod mojom {
    // cpp: layoutng_style/style/computed_style.h:36-37
    // The source declares the ABI width, but does not supply enum values.
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct PreferredColorScheme(i32);

    impl PreferredColorScheme {
        pub const fn from_i32(value: i32) -> Self {
            Self(value)
        }

        pub const fn value(self) -> i32 {
            self.0
        }
    }

    // The C++ min/max aliases share values with Light and Dark.
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct ColorScheme(i32);
    #[allow(non_upper_case_globals)]
    impl ColorScheme {
        pub const kLight: Self = Self(0);
        pub const kDark: Self = Self(1);
        pub const kMinValue: Self = Self(0);
        pub const kMaxValue: Self = Self(1);
        pub const fn from_i32(value: i32) -> Self {
            Self(value)
        }
        pub const fn value(self) -> i32 {
            self.0
        }
    }

    // cpp: layoutng_style/style/color_scheme.h:15-17
    pub mod blink {
        pub use super::{ColorScheme, PreferredColorScheme};
    }
}
