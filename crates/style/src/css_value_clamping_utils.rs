#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]
// Copyright 2021 The Chromium Authors. BSD-style license; see Chromium LICENSE.
mod sealed {
    pub trait Floating {}
    impl Floating for f32 {}
    impl Floating for f64 {}
}

// The C++ floating-point template and float/double overloads use one sealed
// Rust trait, preserving the input and output precision.
pub trait CSSFloatingPoint: sealed::Floating + Copy {
    fn censor_nan(self) -> Self;
    fn clamp_finite(self) -> Self;
}
macro_rules! floating_impl {
    ($t:ty) => {
        impl CSSFloatingPoint for $t {
            // cpp: third_party/blink/renderer/core/css/css_value_clamping_utils.h:20-24
            fn censor_nan(self) -> Self {
                if self.is_nan() {
                    0.0
                } else {
                    self
                }
            }
            // cpp: third_party/blink/renderer/platform/wtf/math_extras.h:149-156,303-322
            fn clamp_finite(self) -> Self {
                let value = self.censor_nan();
                if value >= <$t>::MAX {
                    <$t>::MAX
                } else if value <= -<$t>::MAX {
                    -<$t>::MAX
                } else {
                    value
                }
            }
        }
    };
}
floating_impl!(f32);
floating_impl!(f64);

// cpp: third_party/blink/renderer/core/css/css_value_clamping_utils.h:15-30
pub struct CSSValueClampingUtils;
impl CSSValueClampingUtils {
    // cpp: third_party/blink/renderer/core/css/css_value_clamping_utils.h:20-24
    pub fn CensorNaNToZero<T: CSSFloatingPoint>(value: T) -> T {
        value.censor_nan()
    }
    // cpp: third_party/blink/renderer/core/css/css_value_clamping_utils.cc:10-12
    pub fn ClampDouble(value: f64) -> f64 {
        value.clamp_finite()
    }
    // cpp: third_party/blink/renderer/core/css/css_value_clamping_utils.cc:14-20
    pub fn ClampLength<T: CSSFloatingPoint>(value: T) -> T {
        value.clamp_finite()
    }
    // cpp: third_party/blink/renderer/core/css/css_value_clamping_utils.cc:22-24
    pub fn ClampTime(value: f64) -> f64 {
        Self::ClampDouble(value)
    }
    // cpp: third_party/blink/renderer/core/css/css_value_clamping_utils.cc:26-38
    pub fn ClampAngle(value: f64) -> f64 {
        const kApproxDoubleInfinityAngle: f64 = 2_867_080_569_122_160.0;
        let value = Self::CensorNaNToZero(value);
        if value >= kApproxDoubleInfinityAngle {
            kApproxDoubleInfinityAngle
        } else if value <= -kApproxDoubleInfinityAngle {
            -kApproxDoubleInfinityAngle
        } else {
            value
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::css_value_clamping_utils;
    #[test]
    fn clamping_special_values() {
        use css_value_clamping_utils::CSSValueClampingUtils as C;
        assert_eq!(C::ClampDouble(f64::NAN), 0.0);
        assert_eq!(C::ClampDouble(f64::INFINITY), f64::MAX);
        assert_eq!(C::ClampDouble(f64::NEG_INFINITY), -f64::MAX);
        assert_eq!(C::ClampLength(f32::INFINITY), f32::MAX);
        assert_eq!(C::ClampLength(f32::NAN), 0.0);
        assert_eq!(C::ClampTime(-17.5), -17.5);
        assert_eq!(C::ClampAngle(f64::INFINITY), 2867080569122160.0);
        assert_eq!(C::ClampAngle(f64::NEG_INFINITY), -2867080569122160.0);
        assert_eq!(C::ClampAngle(f64::NAN), 0.0);
        assert_eq!(C::ClampAngle(-0.0).to_bits(), (-0.0_f64).to_bits());
    }
}
