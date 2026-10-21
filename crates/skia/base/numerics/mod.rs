// Root module wiring for delivered external source translations.
// Unresolved true native dependencies remain visible; zero Skia row credit.
pub mod checked_math;
pub mod checked_math_impl;
pub mod clamped_math;
pub mod clamped_math_impl;
pub mod native_overflow_builtins;
pub mod safe_conversions;
pub mod safe_conversions_arm_impl;
pub mod safe_conversions_impl;
pub mod safe_math;
pub mod safe_math_arm_impl;
pub mod safe_math_clang_gcc_impl;
pub mod safe_math_shared_impl;
