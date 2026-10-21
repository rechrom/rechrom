// Source: base/numerics/safe_conversions_impl.h; pointer-width domains follow native intptr_t/uintptr_t.
#[cfg(target_pointer_width = "32")]
mod domain {
    // Source: base/numerics/safe_conversions_impl.h. Primitive-domain specialization of native templates.
    #![allow(non_snake_case, non_upper_case_globals)]
    use core::cmp::Ordering;
    #[derive(Clone, Copy, Default, PartialEq, Eq)]
    pub struct RangeCheck {
        is_underflow_: bool,
        is_overflow_: bool,
    }
    impl RangeCheck {
        pub const fn new(lower: bool, upper: bool) -> Self {
            Self {
                is_underflow_: !lower,
                is_overflow_: !upper,
            }
        }
        pub const fn IsValid(self) -> bool {
            !self.is_overflow_ && !self.is_underflow_
        }
        pub const fn IsInvalid(self) -> bool {
            self.is_overflow_ && self.is_underflow_
        }
        pub const fn IsOverflow(self) -> bool {
            self.is_overflow_ && !self.is_underflow_
        }
        pub const fn IsUnderflow(self) -> bool {
            !self.is_overflow_ && self.is_underflow_
        }
        pub const fn IsOverflowFlagSet(self) -> bool {
            self.is_overflow_
        }
        pub const fn IsUnderflowFlagSet(self) -> bool {
            self.is_underflow_
        }
    }
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub enum IntegerRepresentation {
        kUnsigned,
        kSigned,
    }
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub enum NumericRangeRepresentation {
        kNotContained,
        kContained,
    }
    pub(super) mod sealed {
        pub trait Primitive {}
    }
    pub trait Numeric: sealed::Primitive + Copy + Default + PartialOrd + PartialEq {
        const ZERO: Self;
        const MAX: Self;
        const LOWEST: Self;
        const SIGNED: bool;
        const FLOAT: bool;
        const DIGITS: u32;
        const MAX_EXPONENT: u32;
        fn negative(self) -> bool;
        fn finite(self) -> bool;
        fn nan() -> Self;
        fn overflow() -> Self;
        fn underflow() -> Self;
    }
    pub trait Cast<D: Numeric>: Numeric {
        fn cast(self) -> D;
        fn range(self) -> RangeCheck;
        const CONTAINED: bool;
        fn default_saturate(self) -> D {
            let c = self.range();
            if !c.IsOverflowFlagSet() {
                if !c.IsUnderflowFlagSet() {
                    self.cast()
                } else {
                    D::underflow()
                }
            } else if !Self::FLOAT || !c.IsUnderflowFlagSet() {
                D::overflow()
            } else {
                D::nan()
            }
        }
    }
    pub trait StrictCast<D: Numeric>: Cast<D> {}
    pub trait Input: Copy {
        type Value: Numeric;
        fn value(self) -> Self::Value;
        fn valid(self) -> bool {
            true
        }
    }
    impl<T: Numeric> Input for T {
        type Value = T;
        fn value(self) -> T {
            self
        }
    }
    pub trait Promote<R: Numeric>: Numeric {
        type Max: Numeric;
        type Lowest: Numeric;
        type Big: Numeric;
        type Fast: Numeric;
        type Sum: Numeric;
        const BIG_CONTAINED: bool;
        const FAST_CONTAINED: bool;
        fn sum_pair(self, r: R) -> (Self::Sum, Self::Sum);
    }
    pub fn IsValueNegative<T: Numeric>(v: T) -> bool {
        v.negative()
    }
    pub fn DstRangeRelationToSrcRange<D: Numeric, S: Cast<D>>(v: S) -> RangeCheck {
        v.range()
    }
    pub const fn kIsTypeInRangeForNumericType<D: Numeric, S: Cast<D>>() -> bool {
        S::CONTAINED
    }
    pub fn IsLess<L: Numeric + Cast<R> + Promote<R>, R: Numeric + Cast<L>>(l: L, r: R) -> bool {
        let a = l.range();
        let b = r.range();
        let (x, y) = l.sum_pair(r);
        a.IsUnderflow() || b.IsOverflow() || (a == b && x < y)
    }
    pub fn IsLessOrEqual<L: Numeric + Cast<R> + Promote<R>, R: Numeric + Cast<L>>(
        l: L,
        r: R,
    ) -> bool {
        let a = l.range();
        let b = r.range();
        let (x, y) = l.sum_pair(r);
        a.IsUnderflow() || b.IsOverflow() || (a == b && x <= y)
    }
    pub fn IsGreater<L: Numeric + Cast<R> + Promote<R>, R: Numeric + Cast<L>>(l: L, r: R) -> bool {
        let a = l.range();
        let b = r.range();
        let (x, y) = l.sum_pair(r);
        a.IsOverflow() || b.IsUnderflow() || (a == b && x > y)
    }
    pub fn IsGreaterOrEqual<L: Numeric + Cast<R> + Promote<R>, R: Numeric + Cast<L>>(
        l: L,
        r: R,
    ) -> bool {
        let a = l.range();
        let b = r.range();
        let (x, y) = l.sum_pair(r);
        a.IsOverflow() || b.IsUnderflow() || (a == b && x >= y)
    }
    pub fn IsEqual<L: Numeric + Cast<R> + Promote<R>, R: Numeric + Cast<L>>(l: L, r: R) -> bool {
        let (x, y) = l.sum_pair(r);
        l.range() == r.range() && x == y
    }
    pub fn IsNotEqual<L: Numeric + Cast<R> + Promote<R>, R: Numeric + Cast<L>>(l: L, r: R) -> bool {
        let (x, y) = l.sum_pair(r);
        l.range() != r.range() || x != y
    }
    pub struct CheckOnFailure;
    impl CheckOnFailure {
        #[cfg(all(
            not(base_numeric_compiler_msvc),
            any(base_numeric_compiler_clang, base_numeric_compiler_gcc)
        ))]
        pub fn HandleFailure<T: Default>() -> T {
            core::intrinsics::abort()
        }
    }
    pub trait FailureHandler {
        fn HandleFailure<T: Default>() -> T;
    }
    #[cfg(all(
        not(base_numeric_compiler_msvc),
        any(base_numeric_compiler_clang, base_numeric_compiler_gcc)
    ))]
    impl FailureHandler for CheckOnFailure {
        fn HandleFailure<T: Default>() -> T {
            Self::HandleFailure()
        }
    }
    pub const kEnableAsmCode: bool = cfg!(base_numeric_ndebug);
    impl sealed::Primitive for i8 {}
    impl Numeric for i8 {
        const ZERO: Self = 0 as i8;
        const MAX: Self = Self::MAX;
        const LOWEST: Self = Self::MIN;
        const SIGNED: bool = true;
        const FLOAT: bool = false;
        const DIGITS: u32 = 7;
        const MAX_EXPONENT: u32 = 8;
        fn negative(self) -> bool {
            self < 0 as i8
        }
        fn finite(self) -> bool {
            true
        }
        fn nan() -> Self {
            0
        }
        fn overflow() -> Self {
            Self::MAX
        }
        fn underflow() -> Self {
            Self::MIN
        }
    }
    impl sealed::Primitive for u8 {}
    impl Numeric for u8 {
        const ZERO: Self = 0 as u8;
        const MAX: Self = Self::MAX;
        const LOWEST: Self = Self::MIN;
        const SIGNED: bool = false;
        const FLOAT: bool = false;
        const DIGITS: u32 = 8;
        const MAX_EXPONENT: u32 = 9;
        fn negative(self) -> bool {
            self < 0 as u8
        }
        fn finite(self) -> bool {
            true
        }
        fn nan() -> Self {
            0
        }
        fn overflow() -> Self {
            Self::MAX
        }
        fn underflow() -> Self {
            Self::MIN
        }
    }
    impl sealed::Primitive for i16 {}
    impl Numeric for i16 {
        const ZERO: Self = 0 as i16;
        const MAX: Self = Self::MAX;
        const LOWEST: Self = Self::MIN;
        const SIGNED: bool = true;
        const FLOAT: bool = false;
        const DIGITS: u32 = 15;
        const MAX_EXPONENT: u32 = 16;
        fn negative(self) -> bool {
            self < 0 as i16
        }
        fn finite(self) -> bool {
            true
        }
        fn nan() -> Self {
            0
        }
        fn overflow() -> Self {
            Self::MAX
        }
        fn underflow() -> Self {
            Self::MIN
        }
    }
    impl sealed::Primitive for u16 {}
    impl Numeric for u16 {
        const ZERO: Self = 0 as u16;
        const MAX: Self = Self::MAX;
        const LOWEST: Self = Self::MIN;
        const SIGNED: bool = false;
        const FLOAT: bool = false;
        const DIGITS: u32 = 16;
        const MAX_EXPONENT: u32 = 17;
        fn negative(self) -> bool {
            self < 0 as u16
        }
        fn finite(self) -> bool {
            true
        }
        fn nan() -> Self {
            0
        }
        fn overflow() -> Self {
            Self::MAX
        }
        fn underflow() -> Self {
            Self::MIN
        }
    }
    impl sealed::Primitive for i32 {}
    impl Numeric for i32 {
        const ZERO: Self = 0 as i32;
        const MAX: Self = Self::MAX;
        const LOWEST: Self = Self::MIN;
        const SIGNED: bool = true;
        const FLOAT: bool = false;
        const DIGITS: u32 = 31;
        const MAX_EXPONENT: u32 = 32;
        fn negative(self) -> bool {
            self < 0 as i32
        }
        fn finite(self) -> bool {
            true
        }
        fn nan() -> Self {
            0
        }
        fn overflow() -> Self {
            Self::MAX
        }
        fn underflow() -> Self {
            Self::MIN
        }
    }
    impl sealed::Primitive for u32 {}
    impl Numeric for u32 {
        const ZERO: Self = 0 as u32;
        const MAX: Self = Self::MAX;
        const LOWEST: Self = Self::MIN;
        const SIGNED: bool = false;
        const FLOAT: bool = false;
        const DIGITS: u32 = 32;
        const MAX_EXPONENT: u32 = 33;
        fn negative(self) -> bool {
            self < 0 as u32
        }
        fn finite(self) -> bool {
            true
        }
        fn nan() -> Self {
            0
        }
        fn overflow() -> Self {
            Self::MAX
        }
        fn underflow() -> Self {
            Self::MIN
        }
    }
    impl sealed::Primitive for i64 {}
    impl Numeric for i64 {
        const ZERO: Self = 0 as i64;
        const MAX: Self = Self::MAX;
        const LOWEST: Self = Self::MIN;
        const SIGNED: bool = true;
        const FLOAT: bool = false;
        const DIGITS: u32 = 63;
        const MAX_EXPONENT: u32 = 64;
        fn negative(self) -> bool {
            self < 0 as i64
        }
        fn finite(self) -> bool {
            true
        }
        fn nan() -> Self {
            0
        }
        fn overflow() -> Self {
            Self::MAX
        }
        fn underflow() -> Self {
            Self::MIN
        }
    }
    impl sealed::Primitive for u64 {}
    impl Numeric for u64 {
        const ZERO: Self = 0 as u64;
        const MAX: Self = Self::MAX;
        const LOWEST: Self = Self::MIN;
        const SIGNED: bool = false;
        const FLOAT: bool = false;
        const DIGITS: u32 = 64;
        const MAX_EXPONENT: u32 = 65;
        fn negative(self) -> bool {
            self < 0 as u64
        }
        fn finite(self) -> bool {
            true
        }
        fn nan() -> Self {
            0
        }
        fn overflow() -> Self {
            Self::MAX
        }
        fn underflow() -> Self {
            Self::MIN
        }
    }
    impl sealed::Primitive for f32 {}
    impl Numeric for f32 {
        const ZERO: Self = 0 as f32;
        const MAX: Self = Self::MAX;
        const LOWEST: Self = Self::MIN;
        const SIGNED: bool = true;
        const FLOAT: bool = true;
        const DIGITS: u32 = 24;
        const MAX_EXPONENT: u32 = 128;
        fn negative(self) -> bool {
            self < 0 as f32
        }
        fn finite(self) -> bool {
            self.is_finite()
        }
        fn nan() -> Self {
            Self::NAN
        }
        fn overflow() -> Self {
            Self::INFINITY
        }
        fn underflow() -> Self {
            Self::NEG_INFINITY
        }
    }
    impl sealed::Primitive for f64 {}
    impl Numeric for f64 {
        const ZERO: Self = 0 as f64;
        const MAX: Self = Self::MAX;
        const LOWEST: Self = Self::MIN;
        const SIGNED: bool = true;
        const FLOAT: bool = true;
        const DIGITS: u32 = 53;
        const MAX_EXPONENT: u32 = 1024;
        fn negative(self) -> bool {
            self < 0 as f64
        }
        fn finite(self) -> bool {
            self.is_finite()
        }
        fn nan() -> Self {
            Self::NAN
        }
        fn overflow() -> Self {
            Self::INFINITY
        }
        fn underflow() -> Self {
            Self::NEG_INFINITY
        }
    }
    impl sealed::Primitive for isize {}
    impl Numeric for isize {
        const ZERO: Self = 0 as isize;
        const MAX: Self = Self::MAX;
        const LOWEST: Self = Self::MIN;
        const SIGNED: bool = true;
        const FLOAT: bool = false;
        const DIGITS: u32 = 31;
        const MAX_EXPONENT: u32 = 32;
        fn negative(self) -> bool {
            self < 0 as isize
        }
        fn finite(self) -> bool {
            true
        }
        fn nan() -> Self {
            0
        }
        fn overflow() -> Self {
            Self::MAX
        }
        fn underflow() -> Self {
            Self::MIN
        }
    }
    impl sealed::Primitive for usize {}
    impl Numeric for usize {
        const ZERO: Self = 0 as usize;
        const MAX: Self = Self::MAX;
        const LOWEST: Self = Self::MIN;
        const SIGNED: bool = false;
        const FLOAT: bool = false;
        const DIGITS: u32 = 32;
        const MAX_EXPONENT: u32 = 33;
        fn negative(self) -> bool {
            self < 0 as usize
        }
        fn finite(self) -> bool {
            true
        }
        fn nan() -> Self {
            0
        }
        fn overflow() -> Self {
            Self::MAX
        }
        fn underflow() -> Self {
            Self::MIN
        }
    }
    impl Cast<i8> for i8 {
        const CONTAINED: bool = true;
        fn cast(self) -> i8 {
            self as i8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> i8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,i8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,i8>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { -128 as i8 } else { 127 as i8 };
            if <i8 as Cast<i8>>::range(self).IsValid() {
                self as i8
            } else {
                saturated
            }
        }
    }
    impl StrictCast<i8> for i8 {}
    impl Promote<i8> for i8 {
        type Max = i8;
        type Lowest = i8;
        type Big = i8;
        type Fast = i16;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i8) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<u8> for i8 {
        const CONTAINED: bool = false;
        fn cast(self) -> u8 {
            self as u8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, true)
        }
        fn default_saturate(self) -> u8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,i8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,i8>::Do(self)}
            }
            let is_min = true || (!false && self.negative());
            let saturated = if is_min { 0 as u8 } else { 127 as u8 };
            if <i8 as Cast<u8>>::range(self).IsValid() {
                self as u8
            } else {
                saturated
            }
        }
    }
    impl Promote<u8> for i8 {
        type Max = u8;
        type Lowest = i8;
        type Big = i16;
        type Fast = i16;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u8) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<i16> for i8 {
        const CONTAINED: bool = true;
        fn cast(self) -> i16 {
            self as i16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> i16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,i8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,i8>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { -128 as i16 } else { 127 as i16 };
            if <i8 as Cast<i16>>::range(self).IsValid() {
                self as i16
            } else {
                saturated
            }
        }
    }
    impl StrictCast<i16> for i8 {}
    impl Promote<i16> for i8 {
        type Max = i16;
        type Lowest = i16;
        type Big = i16;
        type Fast = i32;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i16) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<u16> for i8 {
        const CONTAINED: bool = false;
        fn cast(self) -> u16 {
            self as u16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, true)
        }
        fn default_saturate(self) -> u16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,i8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,i8>::Do(self)}
            }
            let is_min = true || (!false && self.negative());
            let saturated = if is_min { 0 as u16 } else { 127 as u16 };
            if <i8 as Cast<u16>>::range(self).IsValid() {
                self as u16
            } else {
                saturated
            }
        }
    }
    impl Promote<u16> for i8 {
        type Max = u16;
        type Lowest = i8;
        type Big = i32;
        type Fast = i32;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u16) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<i32> for i8 {
        const CONTAINED: bool = true;
        fn cast(self) -> i32 {
            self as i32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> i32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,i8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,i8>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { -128 as i32 } else { 127 as i32 };
            if <i8 as Cast<i32>>::range(self).IsValid() {
                self as i32
            } else {
                saturated
            }
        }
    }
    impl StrictCast<i32> for i8 {}
    impl Promote<i32> for i8 {
        type Max = i32;
        type Lowest = i32;
        type Big = i32;
        type Fast = i64;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i32) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<u32> for i8 {
        const CONTAINED: bool = false;
        fn cast(self) -> u32 {
            self as u32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, true)
        }
        fn default_saturate(self) -> u32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,i8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,i8>::Do(self)}
            }
            let is_min = true || (!false && self.negative());
            let saturated = if is_min { 0 as u32 } else { 127 as u32 };
            if <i8 as Cast<u32>>::range(self).IsValid() {
                self as u32
            } else {
                saturated
            }
        }
    }
    impl Promote<u32> for i8 {
        type Max = u32;
        type Lowest = i8;
        type Big = i64;
        type Fast = i64;
        type Sum = u32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u32) -> (u32, u32) {
            (self as u32, r as u32)
        }
    }
    impl Cast<i64> for i8 {
        const CONTAINED: bool = true;
        fn cast(self) -> i64 {
            self as i64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> i64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,i8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,i8>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { -128 as i64 } else { 127 as i64 };
            if <i8 as Cast<i64>>::range(self).IsValid() {
                self as i64
            } else {
                saturated
            }
        }
    }
    impl StrictCast<i64> for i8 {}
    impl Promote<i64> for i8 {
        type Max = i64;
        type Lowest = i64;
        type Big = i64;
        type Fast = i64;
        type Sum = i64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i64) -> (i64, i64) {
            (self as i64, r as i64)
        }
    }
    impl Cast<u64> for i8 {
        const CONTAINED: bool = false;
        fn cast(self) -> u64 {
            self as u64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, true)
        }
        fn default_saturate(self) -> u64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,i8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,i8>::Do(self)}
            }
            let is_min = true || (!false && self.negative());
            let saturated = if is_min { 0 as u64 } else { 127 as u64 };
            if <i8 as Cast<u64>>::range(self).IsValid() {
                self as u64
            } else {
                saturated
            }
        }
    }
    impl Promote<u64> for i8 {
        type Max = u64;
        type Lowest = i8;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = false;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u64) -> (u64, u64) {
            (self as u64, r as u64)
        }
    }
    impl Cast<f32> for i8 {
        const CONTAINED: bool = true;
        fn cast(self) -> f32 {
            self as f32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f32> for i8 {}
    impl Promote<f32> for i8 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f32) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<f64> for i8 {
        const CONTAINED: bool = true;
        fn cast(self) -> f64 {
            self as f64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f64> for i8 {}
    impl Promote<f64> for i8 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f64) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<isize> for i8 {
        const CONTAINED: bool = true;
        fn cast(self) -> isize {
            self as isize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> isize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,i8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,i8>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { -128 as isize } else { 127 as isize };
            if <i8 as Cast<isize>>::range(self).IsValid() {
                self as isize
            } else {
                saturated
            }
        }
    }
    impl StrictCast<isize> for i8 {}
    impl Promote<isize> for i8 {
        type Max = isize;
        type Lowest = isize;
        type Big = isize;
        type Fast = i64;
        type Sum = isize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: isize) -> (isize, isize) {
            (self as isize, r as isize)
        }
    }
    impl Cast<usize> for i8 {
        const CONTAINED: bool = false;
        fn cast(self) -> usize {
            self as usize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, true)
        }
        fn default_saturate(self) -> usize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,i8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,i8>::Do(self)}
            }
            let is_min = true || (!false && self.negative());
            let saturated = if is_min { 0 as usize } else { 127 as usize };
            if <i8 as Cast<usize>>::range(self).IsValid() {
                self as usize
            } else {
                saturated
            }
        }
    }
    impl Promote<usize> for i8 {
        type Max = usize;
        type Lowest = i8;
        type Big = i64;
        type Fast = i64;
        type Sum = usize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: usize) -> (usize, usize) {
            (self as usize, r as usize)
        }
    }
    impl Cast<i8> for u8 {
        const CONTAINED: bool = false;
        fn cast(self) -> i8 {
            self as i8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (i8::MAX as u8))
        }
        fn default_saturate(self) -> i8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,u8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,u8>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as i8 } else { 127 as i8 };
            if <u8 as Cast<i8>>::range(self).IsValid() {
                self as i8
            } else {
                saturated
            }
        }
    }
    impl Promote<i8> for u8 {
        type Max = u8;
        type Lowest = i8;
        type Big = i16;
        type Fast = i16;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i8) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<u8> for u8 {
        const CONTAINED: bool = true;
        fn cast(self) -> u8 {
            self as u8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> u8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,u8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,u8>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as u8 } else { 255 as u8 };
            if <u8 as Cast<u8>>::range(self).IsValid() {
                self as u8
            } else {
                saturated
            }
        }
    }
    impl StrictCast<u8> for u8 {}
    impl Promote<u8> for u8 {
        type Max = u8;
        type Lowest = u8;
        type Big = u8;
        type Fast = u16;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u8) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<i16> for u8 {
        const CONTAINED: bool = true;
        fn cast(self) -> i16 {
            self as i16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> i16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,u8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,u8>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as i16 } else { 255 as i16 };
            if <u8 as Cast<i16>>::range(self).IsValid() {
                self as i16
            } else {
                saturated
            }
        }
    }
    impl StrictCast<i16> for u8 {}
    impl Promote<i16> for u8 {
        type Max = i16;
        type Lowest = i16;
        type Big = i16;
        type Fast = i32;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i16) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<u16> for u8 {
        const CONTAINED: bool = true;
        fn cast(self) -> u16 {
            self as u16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> u16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,u8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,u8>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as u16 } else { 255 as u16 };
            if <u8 as Cast<u16>>::range(self).IsValid() {
                self as u16
            } else {
                saturated
            }
        }
    }
    impl StrictCast<u16> for u8 {}
    impl Promote<u16> for u8 {
        type Max = u16;
        type Lowest = u8;
        type Big = u16;
        type Fast = u32;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u16) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<i32> for u8 {
        const CONTAINED: bool = true;
        fn cast(self) -> i32 {
            self as i32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> i32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,u8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,u8>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as i32 } else { 255 as i32 };
            if <u8 as Cast<i32>>::range(self).IsValid() {
                self as i32
            } else {
                saturated
            }
        }
    }
    impl StrictCast<i32> for u8 {}
    impl Promote<i32> for u8 {
        type Max = i32;
        type Lowest = i32;
        type Big = i32;
        type Fast = i64;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i32) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<u32> for u8 {
        const CONTAINED: bool = true;
        fn cast(self) -> u32 {
            self as u32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> u32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,u8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,u8>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as u32 } else { 255 as u32 };
            if <u8 as Cast<u32>>::range(self).IsValid() {
                self as u32
            } else {
                saturated
            }
        }
    }
    impl StrictCast<u32> for u8 {}
    impl Promote<u32> for u8 {
        type Max = u32;
        type Lowest = u8;
        type Big = u32;
        type Fast = u64;
        type Sum = u32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u32) -> (u32, u32) {
            (self as u32, r as u32)
        }
    }
    impl Cast<i64> for u8 {
        const CONTAINED: bool = true;
        fn cast(self) -> i64 {
            self as i64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> i64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,u8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,u8>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as i64 } else { 255 as i64 };
            if <u8 as Cast<i64>>::range(self).IsValid() {
                self as i64
            } else {
                saturated
            }
        }
    }
    impl StrictCast<i64> for u8 {}
    impl Promote<i64> for u8 {
        type Max = i64;
        type Lowest = i64;
        type Big = i64;
        type Fast = i64;
        type Sum = i64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i64) -> (i64, i64) {
            (self as i64, r as i64)
        }
    }
    impl Cast<u64> for u8 {
        const CONTAINED: bool = true;
        fn cast(self) -> u64 {
            self as u64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> u64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,u8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,u8>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as u64 } else { 255 as u64 };
            if <u8 as Cast<u64>>::range(self).IsValid() {
                self as u64
            } else {
                saturated
            }
        }
    }
    impl StrictCast<u64> for u8 {}
    impl Promote<u64> for u8 {
        type Max = u64;
        type Lowest = u8;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u64) -> (u64, u64) {
            (self as u64, r as u64)
        }
    }
    impl Cast<f32> for u8 {
        const CONTAINED: bool = true;
        fn cast(self) -> f32 {
            self as f32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f32> for u8 {}
    impl Promote<f32> for u8 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f32) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<f64> for u8 {
        const CONTAINED: bool = true;
        fn cast(self) -> f64 {
            self as f64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f64> for u8 {}
    impl Promote<f64> for u8 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f64) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<isize> for u8 {
        const CONTAINED: bool = true;
        fn cast(self) -> isize {
            self as isize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> isize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,u8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,u8>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as isize } else { 255 as isize };
            if <u8 as Cast<isize>>::range(self).IsValid() {
                self as isize
            } else {
                saturated
            }
        }
    }
    impl StrictCast<isize> for u8 {}
    impl Promote<isize> for u8 {
        type Max = isize;
        type Lowest = isize;
        type Big = isize;
        type Fast = i64;
        type Sum = isize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: isize) -> (isize, isize) {
            (self as isize, r as isize)
        }
    }
    impl Cast<usize> for u8 {
        const CONTAINED: bool = true;
        fn cast(self) -> usize {
            self as usize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> usize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,u8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,u8>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as usize } else { 255 as usize };
            if <u8 as Cast<usize>>::range(self).IsValid() {
                self as usize
            } else {
                saturated
            }
        }
    }
    impl StrictCast<usize> for u8 {}
    impl Promote<usize> for u8 {
        type Max = usize;
        type Lowest = u8;
        type Big = usize;
        type Fast = u64;
        type Sum = usize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: usize) -> (usize, usize) {
            (self as usize, r as usize)
        }
    }
    impl Cast<i8> for i16 {
        const CONTAINED: bool = false;
        fn cast(self) -> i8 {
            self as i8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= (i8::MIN as i16), self <= (i8::MAX as i16))
        }
        fn default_saturate(self) -> i8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,i16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,i16>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min { -128 as i8 } else { 127 as i8 };
            if <i16 as Cast<i8>>::range(self).IsValid() {
                self as i8
            } else {
                saturated
            }
        }
    }
    impl Promote<i8> for i16 {
        type Max = i16;
        type Lowest = i16;
        type Big = i16;
        type Fast = i32;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i8) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<u8> for i16 {
        const CONTAINED: bool = false;
        fn cast(self) -> u8 {
            self as u8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, self <= (u8::MAX as i16))
        }
        fn default_saturate(self) -> u8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,i16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,i16>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min { 0 as u8 } else { 255 as u8 };
            if <i16 as Cast<u8>>::range(self).IsValid() {
                self as u8
            } else {
                saturated
            }
        }
    }
    impl Promote<u8> for i16 {
        type Max = i16;
        type Lowest = i16;
        type Big = i16;
        type Fast = i32;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u8) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<i16> for i16 {
        const CONTAINED: bool = true;
        fn cast(self) -> i16 {
            self as i16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> i16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,i16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,i16>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { -32768 as i16 } else { 32767 as i16 };
            if <i16 as Cast<i16>>::range(self).IsValid() {
                self as i16
            } else {
                saturated
            }
        }
    }
    impl StrictCast<i16> for i16 {}
    impl Promote<i16> for i16 {
        type Max = i16;
        type Lowest = i16;
        type Big = i16;
        type Fast = i32;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i16) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<u16> for i16 {
        const CONTAINED: bool = false;
        fn cast(self) -> u16 {
            self as u16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, true)
        }
        fn default_saturate(self) -> u16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,i16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,i16>::Do(self)}
            }
            let is_min = true || (!false && self.negative());
            let saturated = if is_min { 0 as u16 } else { 32767 as u16 };
            if <i16 as Cast<u16>>::range(self).IsValid() {
                self as u16
            } else {
                saturated
            }
        }
    }
    impl Promote<u16> for i16 {
        type Max = u16;
        type Lowest = i16;
        type Big = i32;
        type Fast = i32;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u16) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<i32> for i16 {
        const CONTAINED: bool = true;
        fn cast(self) -> i32 {
            self as i32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> i32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,i16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,i16>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { -32768 as i32 } else { 32767 as i32 };
            if <i16 as Cast<i32>>::range(self).IsValid() {
                self as i32
            } else {
                saturated
            }
        }
    }
    impl StrictCast<i32> for i16 {}
    impl Promote<i32> for i16 {
        type Max = i32;
        type Lowest = i32;
        type Big = i32;
        type Fast = i64;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i32) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<u32> for i16 {
        const CONTAINED: bool = false;
        fn cast(self) -> u32 {
            self as u32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, true)
        }
        fn default_saturate(self) -> u32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,i16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,i16>::Do(self)}
            }
            let is_min = true || (!false && self.negative());
            let saturated = if is_min { 0 as u32 } else { 32767 as u32 };
            if <i16 as Cast<u32>>::range(self).IsValid() {
                self as u32
            } else {
                saturated
            }
        }
    }
    impl Promote<u32> for i16 {
        type Max = u32;
        type Lowest = i16;
        type Big = i64;
        type Fast = i64;
        type Sum = u32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u32) -> (u32, u32) {
            (self as u32, r as u32)
        }
    }
    impl Cast<i64> for i16 {
        const CONTAINED: bool = true;
        fn cast(self) -> i64 {
            self as i64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> i64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,i16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,i16>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { -32768 as i64 } else { 32767 as i64 };
            if <i16 as Cast<i64>>::range(self).IsValid() {
                self as i64
            } else {
                saturated
            }
        }
    }
    impl StrictCast<i64> for i16 {}
    impl Promote<i64> for i16 {
        type Max = i64;
        type Lowest = i64;
        type Big = i64;
        type Fast = i64;
        type Sum = i64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i64) -> (i64, i64) {
            (self as i64, r as i64)
        }
    }
    impl Cast<u64> for i16 {
        const CONTAINED: bool = false;
        fn cast(self) -> u64 {
            self as u64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, true)
        }
        fn default_saturate(self) -> u64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,i16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,i16>::Do(self)}
            }
            let is_min = true || (!false && self.negative());
            let saturated = if is_min { 0 as u64 } else { 32767 as u64 };
            if <i16 as Cast<u64>>::range(self).IsValid() {
                self as u64
            } else {
                saturated
            }
        }
    }
    impl Promote<u64> for i16 {
        type Max = u64;
        type Lowest = i16;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = false;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u64) -> (u64, u64) {
            (self as u64, r as u64)
        }
    }
    impl Cast<f32> for i16 {
        const CONTAINED: bool = true;
        fn cast(self) -> f32 {
            self as f32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f32> for i16 {}
    impl Promote<f32> for i16 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f32) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<f64> for i16 {
        const CONTAINED: bool = true;
        fn cast(self) -> f64 {
            self as f64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f64> for i16 {}
    impl Promote<f64> for i16 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f64) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<isize> for i16 {
        const CONTAINED: bool = true;
        fn cast(self) -> isize {
            self as isize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> isize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,i16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,i16>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min {
                -32768 as isize
            } else {
                32767 as isize
            };
            if <i16 as Cast<isize>>::range(self).IsValid() {
                self as isize
            } else {
                saturated
            }
        }
    }
    impl StrictCast<isize> for i16 {}
    impl Promote<isize> for i16 {
        type Max = isize;
        type Lowest = isize;
        type Big = isize;
        type Fast = i64;
        type Sum = isize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: isize) -> (isize, isize) {
            (self as isize, r as isize)
        }
    }
    impl Cast<usize> for i16 {
        const CONTAINED: bool = false;
        fn cast(self) -> usize {
            self as usize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, true)
        }
        fn default_saturate(self) -> usize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,i16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,i16>::Do(self)}
            }
            let is_min = true || (!false && self.negative());
            let saturated = if is_min { 0 as usize } else { 32767 as usize };
            if <i16 as Cast<usize>>::range(self).IsValid() {
                self as usize
            } else {
                saturated
            }
        }
    }
    impl Promote<usize> for i16 {
        type Max = usize;
        type Lowest = i16;
        type Big = i64;
        type Fast = i64;
        type Sum = usize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: usize) -> (usize, usize) {
            (self as usize, r as usize)
        }
    }
    impl Cast<i8> for u16 {
        const CONTAINED: bool = false;
        fn cast(self) -> i8 {
            self as i8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (i8::MAX as u16))
        }
        fn default_saturate(self) -> i8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,u16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,u16>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as i8 } else { 127 as i8 };
            if <u16 as Cast<i8>>::range(self).IsValid() {
                self as i8
            } else {
                saturated
            }
        }
    }
    impl Promote<i8> for u16 {
        type Max = u16;
        type Lowest = i8;
        type Big = i32;
        type Fast = i32;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i8) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<u8> for u16 {
        const CONTAINED: bool = false;
        fn cast(self) -> u8 {
            self as u8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (u8::MAX as u16))
        }
        fn default_saturate(self) -> u8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,u16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,u16>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as u8 } else { 255 as u8 };
            if <u16 as Cast<u8>>::range(self).IsValid() {
                self as u8
            } else {
                saturated
            }
        }
    }
    impl Promote<u8> for u16 {
        type Max = u16;
        type Lowest = u8;
        type Big = u16;
        type Fast = u32;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u8) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<i16> for u16 {
        const CONTAINED: bool = false;
        fn cast(self) -> i16 {
            self as i16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (i16::MAX as u16))
        }
        fn default_saturate(self) -> i16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,u16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,u16>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as i16 } else { 32767 as i16 };
            if <u16 as Cast<i16>>::range(self).IsValid() {
                self as i16
            } else {
                saturated
            }
        }
    }
    impl Promote<i16> for u16 {
        type Max = u16;
        type Lowest = i16;
        type Big = i32;
        type Fast = i32;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i16) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<u16> for u16 {
        const CONTAINED: bool = true;
        fn cast(self) -> u16 {
            self as u16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> u16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,u16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,u16>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as u16 } else { 65535 as u16 };
            if <u16 as Cast<u16>>::range(self).IsValid() {
                self as u16
            } else {
                saturated
            }
        }
    }
    impl StrictCast<u16> for u16 {}
    impl Promote<u16> for u16 {
        type Max = u16;
        type Lowest = u16;
        type Big = u16;
        type Fast = u32;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u16) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<i32> for u16 {
        const CONTAINED: bool = true;
        fn cast(self) -> i32 {
            self as i32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> i32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,u16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,u16>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as i32 } else { 65535 as i32 };
            if <u16 as Cast<i32>>::range(self).IsValid() {
                self as i32
            } else {
                saturated
            }
        }
    }
    impl StrictCast<i32> for u16 {}
    impl Promote<i32> for u16 {
        type Max = i32;
        type Lowest = i32;
        type Big = i32;
        type Fast = i64;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i32) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<u32> for u16 {
        const CONTAINED: bool = true;
        fn cast(self) -> u32 {
            self as u32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> u32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,u16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,u16>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as u32 } else { 65535 as u32 };
            if <u16 as Cast<u32>>::range(self).IsValid() {
                self as u32
            } else {
                saturated
            }
        }
    }
    impl StrictCast<u32> for u16 {}
    impl Promote<u32> for u16 {
        type Max = u32;
        type Lowest = u16;
        type Big = u32;
        type Fast = u64;
        type Sum = u32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u32) -> (u32, u32) {
            (self as u32, r as u32)
        }
    }
    impl Cast<i64> for u16 {
        const CONTAINED: bool = true;
        fn cast(self) -> i64 {
            self as i64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> i64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,u16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,u16>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as i64 } else { 65535 as i64 };
            if <u16 as Cast<i64>>::range(self).IsValid() {
                self as i64
            } else {
                saturated
            }
        }
    }
    impl StrictCast<i64> for u16 {}
    impl Promote<i64> for u16 {
        type Max = i64;
        type Lowest = i64;
        type Big = i64;
        type Fast = i64;
        type Sum = i64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i64) -> (i64, i64) {
            (self as i64, r as i64)
        }
    }
    impl Cast<u64> for u16 {
        const CONTAINED: bool = true;
        fn cast(self) -> u64 {
            self as u64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> u64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,u16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,u16>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as u64 } else { 65535 as u64 };
            if <u16 as Cast<u64>>::range(self).IsValid() {
                self as u64
            } else {
                saturated
            }
        }
    }
    impl StrictCast<u64> for u16 {}
    impl Promote<u64> for u16 {
        type Max = u64;
        type Lowest = u16;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u64) -> (u64, u64) {
            (self as u64, r as u64)
        }
    }
    impl Cast<f32> for u16 {
        const CONTAINED: bool = true;
        fn cast(self) -> f32 {
            self as f32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f32> for u16 {}
    impl Promote<f32> for u16 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f32) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<f64> for u16 {
        const CONTAINED: bool = true;
        fn cast(self) -> f64 {
            self as f64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f64> for u16 {}
    impl Promote<f64> for u16 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f64) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<isize> for u16 {
        const CONTAINED: bool = true;
        fn cast(self) -> isize {
            self as isize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> isize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,u16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,u16>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as isize } else { 65535 as isize };
            if <u16 as Cast<isize>>::range(self).IsValid() {
                self as isize
            } else {
                saturated
            }
        }
    }
    impl StrictCast<isize> for u16 {}
    impl Promote<isize> for u16 {
        type Max = isize;
        type Lowest = isize;
        type Big = isize;
        type Fast = i64;
        type Sum = isize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: isize) -> (isize, isize) {
            (self as isize, r as isize)
        }
    }
    impl Cast<usize> for u16 {
        const CONTAINED: bool = true;
        fn cast(self) -> usize {
            self as usize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> usize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,u16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,u16>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as usize } else { 65535 as usize };
            if <u16 as Cast<usize>>::range(self).IsValid() {
                self as usize
            } else {
                saturated
            }
        }
    }
    impl StrictCast<usize> for u16 {}
    impl Promote<usize> for u16 {
        type Max = usize;
        type Lowest = u16;
        type Big = usize;
        type Fast = u64;
        type Sum = usize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: usize) -> (usize, usize) {
            (self as usize, r as usize)
        }
    }
    impl Cast<i8> for i32 {
        const CONTAINED: bool = false;
        fn cast(self) -> i8 {
            self as i8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= (i8::MIN as i32), self <= (i8::MAX as i32))
        }
        fn default_saturate(self) -> i8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,i32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,i32>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min { -128 as i8 } else { 127 as i8 };
            if <i32 as Cast<i8>>::range(self).IsValid() {
                self as i8
            } else {
                saturated
            }
        }
    }
    impl Promote<i8> for i32 {
        type Max = i32;
        type Lowest = i32;
        type Big = i32;
        type Fast = i64;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i8) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<u8> for i32 {
        const CONTAINED: bool = false;
        fn cast(self) -> u8 {
            self as u8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, self <= (u8::MAX as i32))
        }
        fn default_saturate(self) -> u8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,i32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,i32>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min { 0 as u8 } else { 255 as u8 };
            if <i32 as Cast<u8>>::range(self).IsValid() {
                self as u8
            } else {
                saturated
            }
        }
    }
    impl Promote<u8> for i32 {
        type Max = i32;
        type Lowest = i32;
        type Big = i32;
        type Fast = i64;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u8) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<i16> for i32 {
        const CONTAINED: bool = false;
        fn cast(self) -> i16 {
            self as i16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= (i16::MIN as i32), self <= (i16::MAX as i32))
        }
        fn default_saturate(self) -> i16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,i32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,i32>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min { -32768 as i16 } else { 32767 as i16 };
            if <i32 as Cast<i16>>::range(self).IsValid() {
                self as i16
            } else {
                saturated
            }
        }
    }
    impl Promote<i16> for i32 {
        type Max = i32;
        type Lowest = i32;
        type Big = i32;
        type Fast = i64;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i16) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<u16> for i32 {
        const CONTAINED: bool = false;
        fn cast(self) -> u16 {
            self as u16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, self <= (u16::MAX as i32))
        }
        fn default_saturate(self) -> u16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,i32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,i32>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min { 0 as u16 } else { 65535 as u16 };
            if <i32 as Cast<u16>>::range(self).IsValid() {
                self as u16
            } else {
                saturated
            }
        }
    }
    impl Promote<u16> for i32 {
        type Max = i32;
        type Lowest = i32;
        type Big = i32;
        type Fast = i64;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u16) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<i32> for i32 {
        const CONTAINED: bool = true;
        fn cast(self) -> i32 {
            self as i32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> i32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,i32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,i32>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min {
                -2147483648 as i32
            } else {
                2147483647 as i32
            };
            if <i32 as Cast<i32>>::range(self).IsValid() {
                self as i32
            } else {
                saturated
            }
        }
    }
    impl StrictCast<i32> for i32 {}
    impl Promote<i32> for i32 {
        type Max = i32;
        type Lowest = i32;
        type Big = i32;
        type Fast = i64;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i32) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<u32> for i32 {
        const CONTAINED: bool = false;
        fn cast(self) -> u32 {
            self as u32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, true)
        }
        fn default_saturate(self) -> u32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,i32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,i32>::Do(self)}
            }
            let is_min = true || (!false && self.negative());
            let saturated = if is_min { 0 as u32 } else { 2147483647 as u32 };
            if <i32 as Cast<u32>>::range(self).IsValid() {
                self as u32
            } else {
                saturated
            }
        }
    }
    impl Promote<u32> for i32 {
        type Max = u32;
        type Lowest = i32;
        type Big = i64;
        type Fast = i64;
        type Sum = u32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u32) -> (u32, u32) {
            (self as u32, r as u32)
        }
    }
    impl Cast<i64> for i32 {
        const CONTAINED: bool = true;
        fn cast(self) -> i64 {
            self as i64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> i64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,i32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,i32>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min {
                -2147483648 as i64
            } else {
                2147483647 as i64
            };
            if <i32 as Cast<i64>>::range(self).IsValid() {
                self as i64
            } else {
                saturated
            }
        }
    }
    impl StrictCast<i64> for i32 {}
    impl Promote<i64> for i32 {
        type Max = i64;
        type Lowest = i64;
        type Big = i64;
        type Fast = i64;
        type Sum = i64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i64) -> (i64, i64) {
            (self as i64, r as i64)
        }
    }
    impl Cast<u64> for i32 {
        const CONTAINED: bool = false;
        fn cast(self) -> u64 {
            self as u64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, true)
        }
        fn default_saturate(self) -> u64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,i32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,i32>::Do(self)}
            }
            let is_min = true || (!false && self.negative());
            let saturated = if is_min { 0 as u64 } else { 2147483647 as u64 };
            if <i32 as Cast<u64>>::range(self).IsValid() {
                self as u64
            } else {
                saturated
            }
        }
    }
    impl Promote<u64> for i32 {
        type Max = u64;
        type Lowest = i32;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = false;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u64) -> (u64, u64) {
            (self as u64, r as u64)
        }
    }
    impl Cast<f32> for i32 {
        const CONTAINED: bool = true;
        fn cast(self) -> f32 {
            self as f32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f32> for i32 {}
    impl Promote<f32> for i32 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f32) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<f64> for i32 {
        const CONTAINED: bool = true;
        fn cast(self) -> f64 {
            self as f64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f64> for i32 {}
    impl Promote<f64> for i32 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f64) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<isize> for i32 {
        const CONTAINED: bool = true;
        fn cast(self) -> isize {
            self as isize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> isize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,i32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,i32>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min {
                -2147483648 as isize
            } else {
                2147483647 as isize
            };
            if <i32 as Cast<isize>>::range(self).IsValid() {
                self as isize
            } else {
                saturated
            }
        }
    }
    impl StrictCast<isize> for i32 {}
    impl Promote<isize> for i32 {
        type Max = isize;
        type Lowest = isize;
        type Big = isize;
        type Fast = i64;
        type Sum = isize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: isize) -> (isize, isize) {
            (self as isize, r as isize)
        }
    }
    impl Cast<usize> for i32 {
        const CONTAINED: bool = false;
        fn cast(self) -> usize {
            self as usize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, true)
        }
        fn default_saturate(self) -> usize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,i32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,i32>::Do(self)}
            }
            let is_min = true || (!false && self.negative());
            let saturated = if is_min {
                0 as usize
            } else {
                2147483647 as usize
            };
            if <i32 as Cast<usize>>::range(self).IsValid() {
                self as usize
            } else {
                saturated
            }
        }
    }
    impl Promote<usize> for i32 {
        type Max = usize;
        type Lowest = i32;
        type Big = i64;
        type Fast = i64;
        type Sum = usize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: usize) -> (usize, usize) {
            (self as usize, r as usize)
        }
    }
    impl Cast<i8> for u32 {
        const CONTAINED: bool = false;
        fn cast(self) -> i8 {
            self as i8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (i8::MAX as u32))
        }
        fn default_saturate(self) -> i8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,u32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,u32>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as i8 } else { 127 as i8 };
            if <u32 as Cast<i8>>::range(self).IsValid() {
                self as i8
            } else {
                saturated
            }
        }
    }
    impl Promote<i8> for u32 {
        type Max = u32;
        type Lowest = i8;
        type Big = i64;
        type Fast = i64;
        type Sum = u32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i8) -> (u32, u32) {
            (self as u32, r as u32)
        }
    }
    impl Cast<u8> for u32 {
        const CONTAINED: bool = false;
        fn cast(self) -> u8 {
            self as u8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (u8::MAX as u32))
        }
        fn default_saturate(self) -> u8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,u32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,u32>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as u8 } else { 255 as u8 };
            if <u32 as Cast<u8>>::range(self).IsValid() {
                self as u8
            } else {
                saturated
            }
        }
    }
    impl Promote<u8> for u32 {
        type Max = u32;
        type Lowest = u8;
        type Big = u32;
        type Fast = u64;
        type Sum = u32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u8) -> (u32, u32) {
            (self as u32, r as u32)
        }
    }
    impl Cast<i16> for u32 {
        const CONTAINED: bool = false;
        fn cast(self) -> i16 {
            self as i16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (i16::MAX as u32))
        }
        fn default_saturate(self) -> i16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,u32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,u32>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as i16 } else { 32767 as i16 };
            if <u32 as Cast<i16>>::range(self).IsValid() {
                self as i16
            } else {
                saturated
            }
        }
    }
    impl Promote<i16> for u32 {
        type Max = u32;
        type Lowest = i16;
        type Big = i64;
        type Fast = i64;
        type Sum = u32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i16) -> (u32, u32) {
            (self as u32, r as u32)
        }
    }
    impl Cast<u16> for u32 {
        const CONTAINED: bool = false;
        fn cast(self) -> u16 {
            self as u16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (u16::MAX as u32))
        }
        fn default_saturate(self) -> u16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,u32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,u32>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as u16 } else { 65535 as u16 };
            if <u32 as Cast<u16>>::range(self).IsValid() {
                self as u16
            } else {
                saturated
            }
        }
    }
    impl Promote<u16> for u32 {
        type Max = u32;
        type Lowest = u16;
        type Big = u32;
        type Fast = u64;
        type Sum = u32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u16) -> (u32, u32) {
            (self as u32, r as u32)
        }
    }
    impl Cast<i32> for u32 {
        const CONTAINED: bool = false;
        fn cast(self) -> i32 {
            self as i32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (i32::MAX as u32))
        }
        fn default_saturate(self) -> i32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,u32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,u32>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as i32 } else { 2147483647 as i32 };
            if <u32 as Cast<i32>>::range(self).IsValid() {
                self as i32
            } else {
                saturated
            }
        }
    }
    impl Promote<i32> for u32 {
        type Max = u32;
        type Lowest = i32;
        type Big = i64;
        type Fast = i64;
        type Sum = u32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i32) -> (u32, u32) {
            (self as u32, r as u32)
        }
    }
    impl Cast<u32> for u32 {
        const CONTAINED: bool = true;
        fn cast(self) -> u32 {
            self as u32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> u32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,u32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,u32>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as u32 } else { 4294967295 as u32 };
            if <u32 as Cast<u32>>::range(self).IsValid() {
                self as u32
            } else {
                saturated
            }
        }
    }
    impl StrictCast<u32> for u32 {}
    impl Promote<u32> for u32 {
        type Max = u32;
        type Lowest = u32;
        type Big = u32;
        type Fast = u64;
        type Sum = u32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u32) -> (u32, u32) {
            (self as u32, r as u32)
        }
    }
    impl Cast<i64> for u32 {
        const CONTAINED: bool = true;
        fn cast(self) -> i64 {
            self as i64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> i64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,u32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,u32>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as i64 } else { 4294967295 as i64 };
            if <u32 as Cast<i64>>::range(self).IsValid() {
                self as i64
            } else {
                saturated
            }
        }
    }
    impl StrictCast<i64> for u32 {}
    impl Promote<i64> for u32 {
        type Max = i64;
        type Lowest = i64;
        type Big = i64;
        type Fast = i64;
        type Sum = i64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i64) -> (i64, i64) {
            (self as i64, r as i64)
        }
    }
    impl Cast<u64> for u32 {
        const CONTAINED: bool = true;
        fn cast(self) -> u64 {
            self as u64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> u64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,u32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,u32>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as u64 } else { 4294967295 as u64 };
            if <u32 as Cast<u64>>::range(self).IsValid() {
                self as u64
            } else {
                saturated
            }
        }
    }
    impl StrictCast<u64> for u32 {}
    impl Promote<u64> for u32 {
        type Max = u64;
        type Lowest = u32;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u64) -> (u64, u64) {
            (self as u64, r as u64)
        }
    }
    impl Cast<f32> for u32 {
        const CONTAINED: bool = true;
        fn cast(self) -> f32 {
            self as f32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f32> for u32 {}
    impl Promote<f32> for u32 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f32) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<f64> for u32 {
        const CONTAINED: bool = true;
        fn cast(self) -> f64 {
            self as f64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f64> for u32 {}
    impl Promote<f64> for u32 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f64) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<isize> for u32 {
        const CONTAINED: bool = false;
        fn cast(self) -> isize {
            self as isize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (isize::MAX as u32))
        }
        fn default_saturate(self) -> isize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,u32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,u32>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min {
                0 as isize
            } else {
                2147483647 as isize
            };
            if <u32 as Cast<isize>>::range(self).IsValid() {
                self as isize
            } else {
                saturated
            }
        }
    }
    impl Promote<isize> for u32 {
        type Max = u32;
        type Lowest = isize;
        type Big = i64;
        type Fast = i64;
        type Sum = u32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: isize) -> (u32, u32) {
            (self as u32, r as u32)
        }
    }
    impl Cast<usize> for u32 {
        const CONTAINED: bool = true;
        fn cast(self) -> usize {
            self as usize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> usize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,u32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,u32>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min {
                0 as usize
            } else {
                4294967295 as usize
            };
            if <u32 as Cast<usize>>::range(self).IsValid() {
                self as usize
            } else {
                saturated
            }
        }
    }
    impl StrictCast<usize> for u32 {}
    impl Promote<usize> for u32 {
        type Max = usize;
        type Lowest = usize;
        type Big = usize;
        type Fast = u64;
        type Sum = usize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: usize) -> (usize, usize) {
            (self as usize, r as usize)
        }
    }
    impl Cast<i8> for i64 {
        const CONTAINED: bool = false;
        fn cast(self) -> i8 {
            self as i8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= (i8::MIN as i64), self <= (i8::MAX as i64))
        }
        fn default_saturate(self) -> i8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,i64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,i64>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min { -128 as i8 } else { 127 as i8 };
            if <i64 as Cast<i8>>::range(self).IsValid() {
                self as i8
            } else {
                saturated
            }
        }
    }
    impl Promote<i8> for i64 {
        type Max = i64;
        type Lowest = i64;
        type Big = i64;
        type Fast = i64;
        type Sum = i64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i8) -> (i64, i64) {
            (self as i64, r as i64)
        }
    }
    impl Cast<u8> for i64 {
        const CONTAINED: bool = false;
        fn cast(self) -> u8 {
            self as u8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, self <= (u8::MAX as i64))
        }
        fn default_saturate(self) -> u8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,i64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,i64>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min { 0 as u8 } else { 255 as u8 };
            if <i64 as Cast<u8>>::range(self).IsValid() {
                self as u8
            } else {
                saturated
            }
        }
    }
    impl Promote<u8> for i64 {
        type Max = i64;
        type Lowest = i64;
        type Big = i64;
        type Fast = i64;
        type Sum = i64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u8) -> (i64, i64) {
            (self as i64, r as i64)
        }
    }
    impl Cast<i16> for i64 {
        const CONTAINED: bool = false;
        fn cast(self) -> i16 {
            self as i16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= (i16::MIN as i64), self <= (i16::MAX as i64))
        }
        fn default_saturate(self) -> i16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,i64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,i64>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min { -32768 as i16 } else { 32767 as i16 };
            if <i64 as Cast<i16>>::range(self).IsValid() {
                self as i16
            } else {
                saturated
            }
        }
    }
    impl Promote<i16> for i64 {
        type Max = i64;
        type Lowest = i64;
        type Big = i64;
        type Fast = i64;
        type Sum = i64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i16) -> (i64, i64) {
            (self as i64, r as i64)
        }
    }
    impl Cast<u16> for i64 {
        const CONTAINED: bool = false;
        fn cast(self) -> u16 {
            self as u16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, self <= (u16::MAX as i64))
        }
        fn default_saturate(self) -> u16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,i64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,i64>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min { 0 as u16 } else { 65535 as u16 };
            if <i64 as Cast<u16>>::range(self).IsValid() {
                self as u16
            } else {
                saturated
            }
        }
    }
    impl Promote<u16> for i64 {
        type Max = i64;
        type Lowest = i64;
        type Big = i64;
        type Fast = i64;
        type Sum = i64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u16) -> (i64, i64) {
            (self as i64, r as i64)
        }
    }
    impl Cast<i32> for i64 {
        const CONTAINED: bool = false;
        fn cast(self) -> i32 {
            self as i32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= (i32::MIN as i64), self <= (i32::MAX as i64))
        }
        fn default_saturate(self) -> i32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,i64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,i64>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min {
                -2147483648 as i32
            } else {
                2147483647 as i32
            };
            if <i64 as Cast<i32>>::range(self).IsValid() {
                self as i32
            } else {
                saturated
            }
        }
    }
    impl Promote<i32> for i64 {
        type Max = i64;
        type Lowest = i64;
        type Big = i64;
        type Fast = i64;
        type Sum = i64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i32) -> (i64, i64) {
            (self as i64, r as i64)
        }
    }
    impl Cast<u32> for i64 {
        const CONTAINED: bool = false;
        fn cast(self) -> u32 {
            self as u32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, self <= (u32::MAX as i64))
        }
        fn default_saturate(self) -> u32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,i64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,i64>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min { 0 as u32 } else { 4294967295 as u32 };
            if <i64 as Cast<u32>>::range(self).IsValid() {
                self as u32
            } else {
                saturated
            }
        }
    }
    impl Promote<u32> for i64 {
        type Max = i64;
        type Lowest = i64;
        type Big = i64;
        type Fast = i64;
        type Sum = i64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u32) -> (i64, i64) {
            (self as i64, r as i64)
        }
    }
    impl Cast<i64> for i64 {
        const CONTAINED: bool = true;
        fn cast(self) -> i64 {
            self as i64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> i64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,i64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,i64>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min {
                -9223372036854775808 as i64
            } else {
                9223372036854775807 as i64
            };
            if <i64 as Cast<i64>>::range(self).IsValid() {
                self as i64
            } else {
                saturated
            }
        }
    }
    impl StrictCast<i64> for i64 {}
    impl Promote<i64> for i64 {
        type Max = i64;
        type Lowest = i64;
        type Big = i64;
        type Fast = i64;
        type Sum = i64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i64) -> (i64, i64) {
            (self as i64, r as i64)
        }
    }
    impl Cast<u64> for i64 {
        const CONTAINED: bool = false;
        fn cast(self) -> u64 {
            self as u64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, true)
        }
        fn default_saturate(self) -> u64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,i64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,i64>::Do(self)}
            }
            let is_min = true || (!false && self.negative());
            let saturated = if is_min {
                0 as u64
            } else {
                9223372036854775807 as u64
            };
            if <i64 as Cast<u64>>::range(self).IsValid() {
                self as u64
            } else {
                saturated
            }
        }
    }
    impl Promote<u64> for i64 {
        type Max = u64;
        type Lowest = i64;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = false;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u64) -> (u64, u64) {
            (self as u64, r as u64)
        }
    }
    impl Cast<f32> for i64 {
        const CONTAINED: bool = true;
        fn cast(self) -> f32 {
            self as f32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f32> for i64 {}
    impl Promote<f32> for i64 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f32) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<f64> for i64 {
        const CONTAINED: bool = true;
        fn cast(self) -> f64 {
            self as f64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f64> for i64 {}
    impl Promote<f64> for i64 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f64) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<isize> for i64 {
        const CONTAINED: bool = false;
        fn cast(self) -> isize {
            self as isize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= (isize::MIN as i64), self <= (isize::MAX as i64))
        }
        fn default_saturate(self) -> isize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,i64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,i64>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min {
                -2147483648 as isize
            } else {
                2147483647 as isize
            };
            if <i64 as Cast<isize>>::range(self).IsValid() {
                self as isize
            } else {
                saturated
            }
        }
    }
    impl Promote<isize> for i64 {
        type Max = i64;
        type Lowest = i64;
        type Big = i64;
        type Fast = i64;
        type Sum = i64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: isize) -> (i64, i64) {
            (self as i64, r as i64)
        }
    }
    impl Cast<usize> for i64 {
        const CONTAINED: bool = false;
        fn cast(self) -> usize {
            self as usize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, self <= (usize::MAX as i64))
        }
        fn default_saturate(self) -> usize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,i64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,i64>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min {
                0 as usize
            } else {
                4294967295 as usize
            };
            if <i64 as Cast<usize>>::range(self).IsValid() {
                self as usize
            } else {
                saturated
            }
        }
    }
    impl Promote<usize> for i64 {
        type Max = i64;
        type Lowest = i64;
        type Big = i64;
        type Fast = i64;
        type Sum = i64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: usize) -> (i64, i64) {
            (self as i64, r as i64)
        }
    }
    impl Cast<i8> for u64 {
        const CONTAINED: bool = false;
        fn cast(self) -> i8 {
            self as i8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (i8::MAX as u64))
        }
        fn default_saturate(self) -> i8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,u64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,u64>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as i8 } else { 127 as i8 };
            if <u64 as Cast<i8>>::range(self).IsValid() {
                self as i8
            } else {
                saturated
            }
        }
    }
    impl Promote<i8> for u64 {
        type Max = u64;
        type Lowest = i8;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = false;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i8) -> (u64, u64) {
            (self as u64, r as u64)
        }
    }
    impl Cast<u8> for u64 {
        const CONTAINED: bool = false;
        fn cast(self) -> u8 {
            self as u8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (u8::MAX as u64))
        }
        fn default_saturate(self) -> u8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,u64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,u64>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as u8 } else { 255 as u8 };
            if <u64 as Cast<u8>>::range(self).IsValid() {
                self as u8
            } else {
                saturated
            }
        }
    }
    impl Promote<u8> for u64 {
        type Max = u64;
        type Lowest = u8;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u8) -> (u64, u64) {
            (self as u64, r as u64)
        }
    }
    impl Cast<i16> for u64 {
        const CONTAINED: bool = false;
        fn cast(self) -> i16 {
            self as i16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (i16::MAX as u64))
        }
        fn default_saturate(self) -> i16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,u64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,u64>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as i16 } else { 32767 as i16 };
            if <u64 as Cast<i16>>::range(self).IsValid() {
                self as i16
            } else {
                saturated
            }
        }
    }
    impl Promote<i16> for u64 {
        type Max = u64;
        type Lowest = i16;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = false;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i16) -> (u64, u64) {
            (self as u64, r as u64)
        }
    }
    impl Cast<u16> for u64 {
        const CONTAINED: bool = false;
        fn cast(self) -> u16 {
            self as u16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (u16::MAX as u64))
        }
        fn default_saturate(self) -> u16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,u64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,u64>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as u16 } else { 65535 as u16 };
            if <u64 as Cast<u16>>::range(self).IsValid() {
                self as u16
            } else {
                saturated
            }
        }
    }
    impl Promote<u16> for u64 {
        type Max = u64;
        type Lowest = u16;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u16) -> (u64, u64) {
            (self as u64, r as u64)
        }
    }
    impl Cast<i32> for u64 {
        const CONTAINED: bool = false;
        fn cast(self) -> i32 {
            self as i32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (i32::MAX as u64))
        }
        fn default_saturate(self) -> i32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,u64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,u64>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as i32 } else { 2147483647 as i32 };
            if <u64 as Cast<i32>>::range(self).IsValid() {
                self as i32
            } else {
                saturated
            }
        }
    }
    impl Promote<i32> for u64 {
        type Max = u64;
        type Lowest = i32;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = false;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i32) -> (u64, u64) {
            (self as u64, r as u64)
        }
    }
    impl Cast<u32> for u64 {
        const CONTAINED: bool = false;
        fn cast(self) -> u32 {
            self as u32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (u32::MAX as u64))
        }
        fn default_saturate(self) -> u32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,u64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,u64>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as u32 } else { 4294967295 as u32 };
            if <u64 as Cast<u32>>::range(self).IsValid() {
                self as u32
            } else {
                saturated
            }
        }
    }
    impl Promote<u32> for u64 {
        type Max = u64;
        type Lowest = u32;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u32) -> (u64, u64) {
            (self as u64, r as u64)
        }
    }
    impl Cast<i64> for u64 {
        const CONTAINED: bool = false;
        fn cast(self) -> i64 {
            self as i64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (i64::MAX as u64))
        }
        fn default_saturate(self) -> i64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,u64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,u64>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min {
                0 as i64
            } else {
                9223372036854775807 as i64
            };
            if <u64 as Cast<i64>>::range(self).IsValid() {
                self as i64
            } else {
                saturated
            }
        }
    }
    impl Promote<i64> for u64 {
        type Max = u64;
        type Lowest = i64;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = false;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i64) -> (u64, u64) {
            (self as u64, r as u64)
        }
    }
    impl Cast<u64> for u64 {
        const CONTAINED: bool = true;
        fn cast(self) -> u64 {
            self as u64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> u64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,u64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,u64>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min {
                0 as u64
            } else {
                18446744073709551615 as u64
            };
            if <u64 as Cast<u64>>::range(self).IsValid() {
                self as u64
            } else {
                saturated
            }
        }
    }
    impl StrictCast<u64> for u64 {}
    impl Promote<u64> for u64 {
        type Max = u64;
        type Lowest = u64;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u64) -> (u64, u64) {
            (self as u64, r as u64)
        }
    }
    impl Cast<f32> for u64 {
        const CONTAINED: bool = true;
        fn cast(self) -> f32 {
            self as f32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f32> for u64 {}
    impl Promote<f32> for u64 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f32) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<f64> for u64 {
        const CONTAINED: bool = true;
        fn cast(self) -> f64 {
            self as f64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f64> for u64 {}
    impl Promote<f64> for u64 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f64) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<isize> for u64 {
        const CONTAINED: bool = false;
        fn cast(self) -> isize {
            self as isize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (isize::MAX as u64))
        }
        fn default_saturate(self) -> isize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,u64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,u64>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min {
                0 as isize
            } else {
                2147483647 as isize
            };
            if <u64 as Cast<isize>>::range(self).IsValid() {
                self as isize
            } else {
                saturated
            }
        }
    }
    impl Promote<isize> for u64 {
        type Max = u64;
        type Lowest = isize;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = false;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: isize) -> (u64, u64) {
            (self as u64, r as u64)
        }
    }
    impl Cast<usize> for u64 {
        const CONTAINED: bool = false;
        fn cast(self) -> usize {
            self as usize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (usize::MAX as u64))
        }
        fn default_saturate(self) -> usize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,u64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,u64>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min {
                0 as usize
            } else {
                4294967295 as usize
            };
            if <u64 as Cast<usize>>::range(self).IsValid() {
                self as usize
            } else {
                saturated
            }
        }
    }
    impl Promote<usize> for u64 {
        type Max = u64;
        type Lowest = usize;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: usize) -> (u64, u64) {
            (self as u64, r as u64)
        }
    }
    impl Cast<i8> for f32 {
        const CONTAINED: bool = false;
        fn cast(self) -> i8 {
            self as i8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= (i8::MIN as f32), self <= (i8::MAX as f32))
        }
    }
    impl Promote<i8> for f32 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i8) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<u8> for f32 {
        const CONTAINED: bool = false;
        fn cast(self) -> u8 {
            self as u8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self > -1.0, self <= (u8::MAX as f32))
        }
    }
    impl Promote<u8> for f32 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u8) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<i16> for f32 {
        const CONTAINED: bool = false;
        fn cast(self) -> i16 {
            self as i16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= (i16::MIN as f32), self <= (i16::MAX as f32))
        }
    }
    impl Promote<i16> for f32 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i16) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<u16> for f32 {
        const CONTAINED: bool = false;
        fn cast(self) -> u16 {
            self as u16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self > -1.0, self <= (u16::MAX as f32))
        }
    }
    impl Promote<u16> for f32 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u16) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<i32> for f32 {
        const CONTAINED: bool = false;
        fn cast(self) -> i32 {
            self as i32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(
                self >= (i32::MIN as f32),
                self <= ((i32::MAX & !(((1 as i32) << 7) - 1)) as f32),
            )
        }
    }
    impl Promote<i32> for f32 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i32) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<u32> for f32 {
        const CONTAINED: bool = false;
        fn cast(self) -> u32 {
            self as u32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(
                self > -1.0,
                self <= ((u32::MAX & !(((1 as u32) << 8) - 1)) as f32),
            )
        }
    }
    impl Promote<u32> for f32 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u32) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<i64> for f32 {
        const CONTAINED: bool = false;
        fn cast(self) -> i64 {
            self as i64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(
                self >= (i64::MIN as f32),
                self <= ((i64::MAX & !(((1 as i64) << 39) - 1)) as f32),
            )
        }
    }
    impl Promote<i64> for f32 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i64) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<u64> for f32 {
        const CONTAINED: bool = false;
        fn cast(self) -> u64 {
            self as u64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(
                self > -1.0,
                self <= ((u64::MAX & !(((1 as u64) << 40) - 1)) as f32),
            )
        }
    }
    impl Promote<u64> for f32 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u64) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<f32> for f32 {
        const CONTAINED: bool = true;
        fn cast(self) -> f32 {
            self as f32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f32> for f32 {}
    impl Promote<f32> for f32 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f32) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<f64> for f32 {
        const CONTAINED: bool = true;
        fn cast(self) -> f64 {
            self as f64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f64> for f32 {}
    impl Promote<f64> for f32 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f64) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<isize> for f32 {
        const CONTAINED: bool = false;
        fn cast(self) -> isize {
            self as isize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(
                self >= (isize::MIN as f32),
                self <= ((isize::MAX & !(((1 as isize) << 7) - 1)) as f32),
            )
        }
    }
    impl Promote<isize> for f32 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: isize) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<usize> for f32 {
        const CONTAINED: bool = false;
        fn cast(self) -> usize {
            self as usize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(
                self > -1.0,
                self <= ((usize::MAX & !(((1 as usize) << 8) - 1)) as f32),
            )
        }
    }
    impl Promote<usize> for f32 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: usize) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<i8> for f64 {
        const CONTAINED: bool = false;
        fn cast(self) -> i8 {
            self as i8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= (i8::MIN as f64), self <= (i8::MAX as f64))
        }
    }
    impl Promote<i8> for f64 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i8) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<u8> for f64 {
        const CONTAINED: bool = false;
        fn cast(self) -> u8 {
            self as u8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self > -1.0, self <= (u8::MAX as f64))
        }
    }
    impl Promote<u8> for f64 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u8) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<i16> for f64 {
        const CONTAINED: bool = false;
        fn cast(self) -> i16 {
            self as i16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= (i16::MIN as f64), self <= (i16::MAX as f64))
        }
    }
    impl Promote<i16> for f64 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i16) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<u16> for f64 {
        const CONTAINED: bool = false;
        fn cast(self) -> u16 {
            self as u16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self > -1.0, self <= (u16::MAX as f64))
        }
    }
    impl Promote<u16> for f64 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u16) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<i32> for f64 {
        const CONTAINED: bool = false;
        fn cast(self) -> i32 {
            self as i32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= (i32::MIN as f64), self <= (i32::MAX as f64))
        }
    }
    impl Promote<i32> for f64 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i32) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<u32> for f64 {
        const CONTAINED: bool = false;
        fn cast(self) -> u32 {
            self as u32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self > -1.0, self <= (u32::MAX as f64))
        }
    }
    impl Promote<u32> for f64 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u32) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<i64> for f64 {
        const CONTAINED: bool = false;
        fn cast(self) -> i64 {
            self as i64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(
                self >= (i64::MIN as f64),
                self <= ((i64::MAX & !(((1 as i64) << 10) - 1)) as f64),
            )
        }
    }
    impl Promote<i64> for f64 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i64) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<u64> for f64 {
        const CONTAINED: bool = false;
        fn cast(self) -> u64 {
            self as u64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(
                self > -1.0,
                self <= ((u64::MAX & !(((1 as u64) << 11) - 1)) as f64),
            )
        }
    }
    impl Promote<u64> for f64 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u64) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<f32> for f64 {
        const CONTAINED: bool = false;
        fn cast(self) -> f32 {
            self as f32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= (f32::MIN as f64), self <= (f32::MAX as f64))
        }
    }
    impl Promote<f32> for f64 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f32) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<f64> for f64 {
        const CONTAINED: bool = true;
        fn cast(self) -> f64 {
            self as f64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f64> for f64 {}
    impl Promote<f64> for f64 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f64) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<isize> for f64 {
        const CONTAINED: bool = false;
        fn cast(self) -> isize {
            self as isize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= (isize::MIN as f64), self <= (isize::MAX as f64))
        }
    }
    impl Promote<isize> for f64 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: isize) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<usize> for f64 {
        const CONTAINED: bool = false;
        fn cast(self) -> usize {
            self as usize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self > -1.0, self <= (usize::MAX as f64))
        }
    }
    impl Promote<usize> for f64 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: usize) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<i8> for isize {
        const CONTAINED: bool = false;
        fn cast(self) -> i8 {
            self as i8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= (i8::MIN as isize), self <= (i8::MAX as isize))
        }
        fn default_saturate(self) -> i8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,isize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,isize>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min { -128 as i8 } else { 127 as i8 };
            if <isize as Cast<i8>>::range(self).IsValid() {
                self as i8
            } else {
                saturated
            }
        }
    }
    impl Promote<i8> for isize {
        type Max = isize;
        type Lowest = isize;
        type Big = isize;
        type Fast = i64;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i8) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<u8> for isize {
        const CONTAINED: bool = false;
        fn cast(self) -> u8 {
            self as u8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, self <= (u8::MAX as isize))
        }
        fn default_saturate(self) -> u8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,isize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,isize>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min { 0 as u8 } else { 255 as u8 };
            if <isize as Cast<u8>>::range(self).IsValid() {
                self as u8
            } else {
                saturated
            }
        }
    }
    impl Promote<u8> for isize {
        type Max = isize;
        type Lowest = isize;
        type Big = isize;
        type Fast = i64;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u8) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<i16> for isize {
        const CONTAINED: bool = false;
        fn cast(self) -> i16 {
            self as i16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= (i16::MIN as isize), self <= (i16::MAX as isize))
        }
        fn default_saturate(self) -> i16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,isize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,isize>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min { -32768 as i16 } else { 32767 as i16 };
            if <isize as Cast<i16>>::range(self).IsValid() {
                self as i16
            } else {
                saturated
            }
        }
    }
    impl Promote<i16> for isize {
        type Max = isize;
        type Lowest = isize;
        type Big = isize;
        type Fast = i64;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i16) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<u16> for isize {
        const CONTAINED: bool = false;
        fn cast(self) -> u16 {
            self as u16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, self <= (u16::MAX as isize))
        }
        fn default_saturate(self) -> u16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,isize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,isize>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min { 0 as u16 } else { 65535 as u16 };
            if <isize as Cast<u16>>::range(self).IsValid() {
                self as u16
            } else {
                saturated
            }
        }
    }
    impl Promote<u16> for isize {
        type Max = isize;
        type Lowest = isize;
        type Big = isize;
        type Fast = i64;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u16) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<i32> for isize {
        const CONTAINED: bool = true;
        fn cast(self) -> i32 {
            self as i32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> i32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,isize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,isize>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min {
                -2147483648 as i32
            } else {
                2147483647 as i32
            };
            if <isize as Cast<i32>>::range(self).IsValid() {
                self as i32
            } else {
                saturated
            }
        }
    }
    impl StrictCast<i32> for isize {}
    impl Promote<i32> for isize {
        type Max = i32;
        type Lowest = i32;
        type Big = i32;
        type Fast = i64;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i32) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<u32> for isize {
        const CONTAINED: bool = false;
        fn cast(self) -> u32 {
            self as u32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, true)
        }
        fn default_saturate(self) -> u32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,isize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,isize>::Do(self)}
            }
            let is_min = true || (!false && self.negative());
            let saturated = if is_min { 0 as u32 } else { 2147483647 as u32 };
            if <isize as Cast<u32>>::range(self).IsValid() {
                self as u32
            } else {
                saturated
            }
        }
    }
    impl Promote<u32> for isize {
        type Max = u32;
        type Lowest = isize;
        type Big = i64;
        type Fast = i64;
        type Sum = u32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u32) -> (u32, u32) {
            (self as u32, r as u32)
        }
    }
    impl Cast<i64> for isize {
        const CONTAINED: bool = true;
        fn cast(self) -> i64 {
            self as i64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> i64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,isize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,isize>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min {
                -2147483648 as i64
            } else {
                2147483647 as i64
            };
            if <isize as Cast<i64>>::range(self).IsValid() {
                self as i64
            } else {
                saturated
            }
        }
    }
    impl StrictCast<i64> for isize {}
    impl Promote<i64> for isize {
        type Max = i64;
        type Lowest = i64;
        type Big = i64;
        type Fast = i64;
        type Sum = i64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i64) -> (i64, i64) {
            (self as i64, r as i64)
        }
    }
    impl Cast<u64> for isize {
        const CONTAINED: bool = false;
        fn cast(self) -> u64 {
            self as u64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, true)
        }
        fn default_saturate(self) -> u64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,isize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,isize>::Do(self)}
            }
            let is_min = true || (!false && self.negative());
            let saturated = if is_min { 0 as u64 } else { 2147483647 as u64 };
            if <isize as Cast<u64>>::range(self).IsValid() {
                self as u64
            } else {
                saturated
            }
        }
    }
    impl Promote<u64> for isize {
        type Max = u64;
        type Lowest = isize;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = false;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u64) -> (u64, u64) {
            (self as u64, r as u64)
        }
    }
    impl Cast<f32> for isize {
        const CONTAINED: bool = true;
        fn cast(self) -> f32 {
            self as f32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f32> for isize {}
    impl Promote<f32> for isize {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f32) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<f64> for isize {
        const CONTAINED: bool = true;
        fn cast(self) -> f64 {
            self as f64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f64> for isize {}
    impl Promote<f64> for isize {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f64) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<isize> for isize {
        const CONTAINED: bool = true;
        fn cast(self) -> isize {
            self as isize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> isize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,isize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,isize>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min {
                -2147483648 as isize
            } else {
                2147483647 as isize
            };
            if <isize as Cast<isize>>::range(self).IsValid() {
                self as isize
            } else {
                saturated
            }
        }
    }
    impl StrictCast<isize> for isize {}
    impl Promote<isize> for isize {
        type Max = isize;
        type Lowest = isize;
        type Big = isize;
        type Fast = i64;
        type Sum = isize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: isize) -> (isize, isize) {
            (self as isize, r as isize)
        }
    }
    impl Cast<usize> for isize {
        const CONTAINED: bool = false;
        fn cast(self) -> usize {
            self as usize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, true)
        }
        fn default_saturate(self) -> usize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,isize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,isize>::Do(self)}
            }
            let is_min = true || (!false && self.negative());
            let saturated = if is_min {
                0 as usize
            } else {
                2147483647 as usize
            };
            if <isize as Cast<usize>>::range(self).IsValid() {
                self as usize
            } else {
                saturated
            }
        }
    }
    impl Promote<usize> for isize {
        type Max = usize;
        type Lowest = isize;
        type Big = i64;
        type Fast = i64;
        type Sum = usize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: usize) -> (usize, usize) {
            (self as usize, r as usize)
        }
    }
    impl Cast<i8> for usize {
        const CONTAINED: bool = false;
        fn cast(self) -> i8 {
            self as i8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (i8::MAX as usize))
        }
        fn default_saturate(self) -> i8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,usize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,usize>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as i8 } else { 127 as i8 };
            if <usize as Cast<i8>>::range(self).IsValid() {
                self as i8
            } else {
                saturated
            }
        }
    }
    impl Promote<i8> for usize {
        type Max = usize;
        type Lowest = i8;
        type Big = i64;
        type Fast = i64;
        type Sum = usize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i8) -> (usize, usize) {
            (self as usize, r as usize)
        }
    }
    impl Cast<u8> for usize {
        const CONTAINED: bool = false;
        fn cast(self) -> u8 {
            self as u8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (u8::MAX as usize))
        }
        fn default_saturate(self) -> u8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,usize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,usize>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as u8 } else { 255 as u8 };
            if <usize as Cast<u8>>::range(self).IsValid() {
                self as u8
            } else {
                saturated
            }
        }
    }
    impl Promote<u8> for usize {
        type Max = usize;
        type Lowest = u8;
        type Big = usize;
        type Fast = u64;
        type Sum = usize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u8) -> (usize, usize) {
            (self as usize, r as usize)
        }
    }
    impl Cast<i16> for usize {
        const CONTAINED: bool = false;
        fn cast(self) -> i16 {
            self as i16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (i16::MAX as usize))
        }
        fn default_saturate(self) -> i16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,usize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,usize>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as i16 } else { 32767 as i16 };
            if <usize as Cast<i16>>::range(self).IsValid() {
                self as i16
            } else {
                saturated
            }
        }
    }
    impl Promote<i16> for usize {
        type Max = usize;
        type Lowest = i16;
        type Big = i64;
        type Fast = i64;
        type Sum = usize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i16) -> (usize, usize) {
            (self as usize, r as usize)
        }
    }
    impl Cast<u16> for usize {
        const CONTAINED: bool = false;
        fn cast(self) -> u16 {
            self as u16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (u16::MAX as usize))
        }
        fn default_saturate(self) -> u16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,usize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,usize>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as u16 } else { 65535 as u16 };
            if <usize as Cast<u16>>::range(self).IsValid() {
                self as u16
            } else {
                saturated
            }
        }
    }
    impl Promote<u16> for usize {
        type Max = usize;
        type Lowest = u16;
        type Big = usize;
        type Fast = u64;
        type Sum = usize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u16) -> (usize, usize) {
            (self as usize, r as usize)
        }
    }
    impl Cast<i32> for usize {
        const CONTAINED: bool = false;
        fn cast(self) -> i32 {
            self as i32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (i32::MAX as usize))
        }
        fn default_saturate(self) -> i32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,usize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,usize>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as i32 } else { 2147483647 as i32 };
            if <usize as Cast<i32>>::range(self).IsValid() {
                self as i32
            } else {
                saturated
            }
        }
    }
    impl Promote<i32> for usize {
        type Max = usize;
        type Lowest = i32;
        type Big = i64;
        type Fast = i64;
        type Sum = usize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i32) -> (usize, usize) {
            (self as usize, r as usize)
        }
    }
    impl Cast<u32> for usize {
        const CONTAINED: bool = true;
        fn cast(self) -> u32 {
            self as u32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> u32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,usize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,usize>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as u32 } else { 4294967295 as u32 };
            if <usize as Cast<u32>>::range(self).IsValid() {
                self as u32
            } else {
                saturated
            }
        }
    }
    impl StrictCast<u32> for usize {}
    impl Promote<u32> for usize {
        type Max = u32;
        type Lowest = u32;
        type Big = u32;
        type Fast = u64;
        type Sum = u32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u32) -> (u32, u32) {
            (self as u32, r as u32)
        }
    }
    impl Cast<i64> for usize {
        const CONTAINED: bool = true;
        fn cast(self) -> i64 {
            self as i64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> i64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,usize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,usize>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as i64 } else { 4294967295 as i64 };
            if <usize as Cast<i64>>::range(self).IsValid() {
                self as i64
            } else {
                saturated
            }
        }
    }
    impl StrictCast<i64> for usize {}
    impl Promote<i64> for usize {
        type Max = i64;
        type Lowest = i64;
        type Big = i64;
        type Fast = i64;
        type Sum = i64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i64) -> (i64, i64) {
            (self as i64, r as i64)
        }
    }
    impl Cast<u64> for usize {
        const CONTAINED: bool = true;
        fn cast(self) -> u64 {
            self as u64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> u64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,usize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,usize>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as u64 } else { 4294967295 as u64 };
            if <usize as Cast<u64>>::range(self).IsValid() {
                self as u64
            } else {
                saturated
            }
        }
    }
    impl StrictCast<u64> for usize {}
    impl Promote<u64> for usize {
        type Max = u64;
        type Lowest = usize;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u64) -> (u64, u64) {
            (self as u64, r as u64)
        }
    }
    impl Cast<f32> for usize {
        const CONTAINED: bool = true;
        fn cast(self) -> f32 {
            self as f32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f32> for usize {}
    impl Promote<f32> for usize {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f32) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<f64> for usize {
        const CONTAINED: bool = true;
        fn cast(self) -> f64 {
            self as f64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f64> for usize {}
    impl Promote<f64> for usize {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f64) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<isize> for usize {
        const CONTAINED: bool = false;
        fn cast(self) -> isize {
            self as isize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (isize::MAX as usize))
        }
        fn default_saturate(self) -> isize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,usize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,usize>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min {
                0 as isize
            } else {
                2147483647 as isize
            };
            if <usize as Cast<isize>>::range(self).IsValid() {
                self as isize
            } else {
                saturated
            }
        }
    }
    impl Promote<isize> for usize {
        type Max = usize;
        type Lowest = isize;
        type Big = i64;
        type Fast = i64;
        type Sum = usize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: isize) -> (usize, usize) {
            (self as usize, r as usize)
        }
    }
    impl Cast<usize> for usize {
        const CONTAINED: bool = true;
        fn cast(self) -> usize {
            self as usize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> usize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,usize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,usize>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min {
                0 as usize
            } else {
                4294967295 as usize
            };
            if <usize as Cast<usize>>::range(self).IsValid() {
                self as usize
            } else {
                saturated
            }
        }
    }
    impl StrictCast<usize> for usize {}
    impl Promote<usize> for usize {
        type Max = usize;
        type Lowest = usize;
        type Big = usize;
        type Fast = u64;
        type Sum = usize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: usize) -> (usize, usize) {
            (self as usize, r as usize)
        }
    }
    // Native make_signed/make_unsigned domains exclude bool. No f64 coercion of integer comparisons.
    pub trait Integer: Numeric {
        type Signed: Numeric;
        type Unsigned: Numeric;
        fn ConditionalNegate(self, negative: bool) -> Self::Signed;
        fn unsigned_abs(self) -> Self::Unsigned;
        fn as_signed(self) -> Self::Signed;
        fn as_unsigned(self) -> Self::Unsigned;
    }
    impl Integer for i8 {
        type Signed = i8;
        type Unsigned = u8;
        fn ConditionalNegate(self, n: bool) -> i8 {
            ((self as u8 ^ (0 as u8).wrapping_sub(n as u8)).wrapping_add(n as u8)) as i8
        }
        fn unsigned_abs(self) -> u8 {
            if self.negative() {
                (0 as u8).wrapping_sub(self as u8)
            } else {
                self as u8
            }
        }
        fn as_signed(self) -> i8 {
            self as i8
        }
        fn as_unsigned(self) -> u8 {
            self as u8
        }
    }
    impl Integer for u8 {
        type Signed = i8;
        type Unsigned = u8;
        fn ConditionalNegate(self, n: bool) -> i8 {
            ((self as u8 ^ (0 as u8).wrapping_sub(n as u8)).wrapping_add(n as u8)) as i8
        }
        fn unsigned_abs(self) -> u8 {
            if self.negative() {
                (0 as u8).wrapping_sub(self as u8)
            } else {
                self as u8
            }
        }
        fn as_signed(self) -> i8 {
            self as i8
        }
        fn as_unsigned(self) -> u8 {
            self as u8
        }
    }
    impl Integer for i16 {
        type Signed = i16;
        type Unsigned = u16;
        fn ConditionalNegate(self, n: bool) -> i16 {
            ((self as u16 ^ (0 as u16).wrapping_sub(n as u16)).wrapping_add(n as u16)) as i16
        }
        fn unsigned_abs(self) -> u16 {
            if self.negative() {
                (0 as u16).wrapping_sub(self as u16)
            } else {
                self as u16
            }
        }
        fn as_signed(self) -> i16 {
            self as i16
        }
        fn as_unsigned(self) -> u16 {
            self as u16
        }
    }
    impl Integer for u16 {
        type Signed = i16;
        type Unsigned = u16;
        fn ConditionalNegate(self, n: bool) -> i16 {
            ((self as u16 ^ (0 as u16).wrapping_sub(n as u16)).wrapping_add(n as u16)) as i16
        }
        fn unsigned_abs(self) -> u16 {
            if self.negative() {
                (0 as u16).wrapping_sub(self as u16)
            } else {
                self as u16
            }
        }
        fn as_signed(self) -> i16 {
            self as i16
        }
        fn as_unsigned(self) -> u16 {
            self as u16
        }
    }
    impl Integer for i32 {
        type Signed = i32;
        type Unsigned = u32;
        fn ConditionalNegate(self, n: bool) -> i32 {
            ((self as u32 ^ (0 as u32).wrapping_sub(n as u32)).wrapping_add(n as u32)) as i32
        }
        fn unsigned_abs(self) -> u32 {
            if self.negative() {
                (0 as u32).wrapping_sub(self as u32)
            } else {
                self as u32
            }
        }
        fn as_signed(self) -> i32 {
            self as i32
        }
        fn as_unsigned(self) -> u32 {
            self as u32
        }
    }
    impl Integer for u32 {
        type Signed = i32;
        type Unsigned = u32;
        fn ConditionalNegate(self, n: bool) -> i32 {
            ((self as u32 ^ (0 as u32).wrapping_sub(n as u32)).wrapping_add(n as u32)) as i32
        }
        fn unsigned_abs(self) -> u32 {
            if self.negative() {
                (0 as u32).wrapping_sub(self as u32)
            } else {
                self as u32
            }
        }
        fn as_signed(self) -> i32 {
            self as i32
        }
        fn as_unsigned(self) -> u32 {
            self as u32
        }
    }
    impl Integer for i64 {
        type Signed = i64;
        type Unsigned = u64;
        fn ConditionalNegate(self, n: bool) -> i64 {
            ((self as u64 ^ (0 as u64).wrapping_sub(n as u64)).wrapping_add(n as u64)) as i64
        }
        fn unsigned_abs(self) -> u64 {
            if self.negative() {
                (0 as u64).wrapping_sub(self as u64)
            } else {
                self as u64
            }
        }
        fn as_signed(self) -> i64 {
            self as i64
        }
        fn as_unsigned(self) -> u64 {
            self as u64
        }
    }
    impl Integer for u64 {
        type Signed = i64;
        type Unsigned = u64;
        fn ConditionalNegate(self, n: bool) -> i64 {
            ((self as u64 ^ (0 as u64).wrapping_sub(n as u64)).wrapping_add(n as u64)) as i64
        }
        fn unsigned_abs(self) -> u64 {
            if self.negative() {
                (0 as u64).wrapping_sub(self as u64)
            } else {
                self as u64
            }
        }
        fn as_signed(self) -> i64 {
            self as i64
        }
        fn as_unsigned(self) -> u64 {
            self as u64
        }
    }
    impl Integer for isize {
        type Signed = i32;
        type Unsigned = u32;
        fn ConditionalNegate(self, n: bool) -> i32 {
            ((self as u32 ^ (0 as u32).wrapping_sub(n as u32)).wrapping_add(n as u32)) as i32
        }
        fn unsigned_abs(self) -> u32 {
            if self.negative() {
                (0 as u32).wrapping_sub(self as u32)
            } else {
                self as u32
            }
        }
        fn as_signed(self) -> i32 {
            self as i32
        }
        fn as_unsigned(self) -> u32 {
            self as u32
        }
    }
    impl Integer for usize {
        type Signed = i32;
        type Unsigned = u32;
        fn ConditionalNegate(self, n: bool) -> i32 {
            ((self as u32 ^ (0 as u32).wrapping_sub(n as u32)).wrapping_add(n as u32)) as i32
        }
        fn unsigned_abs(self) -> u32 {
            if self.negative() {
                (0 as u32).wrapping_sub(self as u32)
            } else {
                self as u32
            }
        }
        fn as_signed(self) -> i32 {
            self as i32
        }
        fn as_unsigned(self) -> u32 {
            self as u32
        }
    }
    pub fn ConditionalNegate<T: Integer>(v: T, n: bool) -> T::Signed {
        v.ConditionalNegate(n)
    }
    pub fn SafeUnsignedAbs<T: Integer>(v: T) -> T::Unsigned {
        v.unsigned_abs()
    }
    pub fn as_signed<T: Integer>(v: T) -> T::Signed {
        v.as_signed()
    }
    pub fn as_unsigned<T: Integer>(v: T) -> T::Unsigned {
        v.as_unsigned()
    }
}
#[cfg(target_pointer_width = "64")]
mod domain {
    // Source: base/numerics/safe_conversions_impl.h. Primitive-domain specialization of native templates.
    #![allow(non_snake_case, non_upper_case_globals)]
    use core::cmp::Ordering;
    #[derive(Clone, Copy, Default, PartialEq, Eq)]
    pub struct RangeCheck {
        is_underflow_: bool,
        is_overflow_: bool,
    }
    impl RangeCheck {
        pub const fn new(lower: bool, upper: bool) -> Self {
            Self {
                is_underflow_: !lower,
                is_overflow_: !upper,
            }
        }
        pub const fn IsValid(self) -> bool {
            !self.is_overflow_ && !self.is_underflow_
        }
        pub const fn IsInvalid(self) -> bool {
            self.is_overflow_ && self.is_underflow_
        }
        pub const fn IsOverflow(self) -> bool {
            self.is_overflow_ && !self.is_underflow_
        }
        pub const fn IsUnderflow(self) -> bool {
            !self.is_overflow_ && self.is_underflow_
        }
        pub const fn IsOverflowFlagSet(self) -> bool {
            self.is_overflow_
        }
        pub const fn IsUnderflowFlagSet(self) -> bool {
            self.is_underflow_
        }
    }
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub enum IntegerRepresentation {
        kUnsigned,
        kSigned,
    }
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub enum NumericRangeRepresentation {
        kNotContained,
        kContained,
    }
    pub(super) mod sealed {
        pub trait Primitive {}
    }
    pub trait Numeric: sealed::Primitive + Copy + Default + PartialOrd + PartialEq {
        const ZERO: Self;
        const MAX: Self;
        const LOWEST: Self;
        const SIGNED: bool;
        const FLOAT: bool;
        const DIGITS: u32;
        const MAX_EXPONENT: u32;
        fn negative(self) -> bool;
        fn finite(self) -> bool;
        fn nan() -> Self;
        fn overflow() -> Self;
        fn underflow() -> Self;
    }
    pub trait Cast<D: Numeric>: Numeric {
        fn cast(self) -> D;
        fn range(self) -> RangeCheck;
        const CONTAINED: bool;
        fn default_saturate(self) -> D {
            let c = self.range();
            if !c.IsOverflowFlagSet() {
                if !c.IsUnderflowFlagSet() {
                    self.cast()
                } else {
                    D::underflow()
                }
            } else if !Self::FLOAT || !c.IsUnderflowFlagSet() {
                D::overflow()
            } else {
                D::nan()
            }
        }
    }
    pub trait StrictCast<D: Numeric>: Cast<D> {}
    pub trait Input: Copy {
        type Value: Numeric;
        fn value(self) -> Self::Value;
        fn valid(self) -> bool {
            true
        }
    }
    impl<T: Numeric> Input for T {
        type Value = T;
        fn value(self) -> T {
            self
        }
    }
    pub trait Promote<R: Numeric>: Numeric {
        type Max: Numeric;
        type Lowest: Numeric;
        type Big: Numeric;
        type Fast: Numeric;
        type Sum: Numeric;
        const BIG_CONTAINED: bool;
        const FAST_CONTAINED: bool;
        fn sum_pair(self, r: R) -> (Self::Sum, Self::Sum);
    }
    pub fn IsValueNegative<T: Numeric>(v: T) -> bool {
        v.negative()
    }
    pub fn DstRangeRelationToSrcRange<D: Numeric, S: Cast<D>>(v: S) -> RangeCheck {
        v.range()
    }
    pub const fn kIsTypeInRangeForNumericType<D: Numeric, S: Cast<D>>() -> bool {
        S::CONTAINED
    }
    pub fn IsLess<L: Numeric + Cast<R> + Promote<R>, R: Numeric + Cast<L>>(l: L, r: R) -> bool {
        let a = l.range();
        let b = r.range();
        let (x, y) = l.sum_pair(r);
        a.IsUnderflow() || b.IsOverflow() || (a == b && x < y)
    }
    pub fn IsLessOrEqual<L: Numeric + Cast<R> + Promote<R>, R: Numeric + Cast<L>>(
        l: L,
        r: R,
    ) -> bool {
        let a = l.range();
        let b = r.range();
        let (x, y) = l.sum_pair(r);
        a.IsUnderflow() || b.IsOverflow() || (a == b && x <= y)
    }
    pub fn IsGreater<L: Numeric + Cast<R> + Promote<R>, R: Numeric + Cast<L>>(l: L, r: R) -> bool {
        let a = l.range();
        let b = r.range();
        let (x, y) = l.sum_pair(r);
        a.IsOverflow() || b.IsUnderflow() || (a == b && x > y)
    }
    pub fn IsGreaterOrEqual<L: Numeric + Cast<R> + Promote<R>, R: Numeric + Cast<L>>(
        l: L,
        r: R,
    ) -> bool {
        let a = l.range();
        let b = r.range();
        let (x, y) = l.sum_pair(r);
        a.IsOverflow() || b.IsUnderflow() || (a == b && x >= y)
    }
    pub fn IsEqual<L: Numeric + Cast<R> + Promote<R>, R: Numeric + Cast<L>>(l: L, r: R) -> bool {
        let (x, y) = l.sum_pair(r);
        l.range() == r.range() && x == y
    }
    pub fn IsNotEqual<L: Numeric + Cast<R> + Promote<R>, R: Numeric + Cast<L>>(l: L, r: R) -> bool {
        let (x, y) = l.sum_pair(r);
        l.range() != r.range() || x != y
    }
    pub struct CheckOnFailure;
    impl CheckOnFailure {
        #[cfg(all(
            not(base_numeric_compiler_msvc),
            any(base_numeric_compiler_clang, base_numeric_compiler_gcc)
        ))]
        pub fn HandleFailure<T: Default>() -> T {
            core::intrinsics::abort()
        }
    }
    pub trait FailureHandler {
        fn HandleFailure<T: Default>() -> T;
    }
    #[cfg(all(
        not(base_numeric_compiler_msvc),
        any(base_numeric_compiler_clang, base_numeric_compiler_gcc)
    ))]
    impl FailureHandler for CheckOnFailure {
        fn HandleFailure<T: Default>() -> T {
            Self::HandleFailure()
        }
    }
    pub const kEnableAsmCode: bool = cfg!(base_numeric_ndebug);
    impl sealed::Primitive for i8 {}
    impl Numeric for i8 {
        const ZERO: Self = 0 as i8;
        const MAX: Self = Self::MAX;
        const LOWEST: Self = Self::MIN;
        const SIGNED: bool = true;
        const FLOAT: bool = false;
        const DIGITS: u32 = 7;
        const MAX_EXPONENT: u32 = 8;
        fn negative(self) -> bool {
            self < 0 as i8
        }
        fn finite(self) -> bool {
            true
        }
        fn nan() -> Self {
            0
        }
        fn overflow() -> Self {
            Self::MAX
        }
        fn underflow() -> Self {
            Self::MIN
        }
    }
    impl sealed::Primitive for u8 {}
    impl Numeric for u8 {
        const ZERO: Self = 0 as u8;
        const MAX: Self = Self::MAX;
        const LOWEST: Self = Self::MIN;
        const SIGNED: bool = false;
        const FLOAT: bool = false;
        const DIGITS: u32 = 8;
        const MAX_EXPONENT: u32 = 9;
        fn negative(self) -> bool {
            self < 0 as u8
        }
        fn finite(self) -> bool {
            true
        }
        fn nan() -> Self {
            0
        }
        fn overflow() -> Self {
            Self::MAX
        }
        fn underflow() -> Self {
            Self::MIN
        }
    }
    impl sealed::Primitive for i16 {}
    impl Numeric for i16 {
        const ZERO: Self = 0 as i16;
        const MAX: Self = Self::MAX;
        const LOWEST: Self = Self::MIN;
        const SIGNED: bool = true;
        const FLOAT: bool = false;
        const DIGITS: u32 = 15;
        const MAX_EXPONENT: u32 = 16;
        fn negative(self) -> bool {
            self < 0 as i16
        }
        fn finite(self) -> bool {
            true
        }
        fn nan() -> Self {
            0
        }
        fn overflow() -> Self {
            Self::MAX
        }
        fn underflow() -> Self {
            Self::MIN
        }
    }
    impl sealed::Primitive for u16 {}
    impl Numeric for u16 {
        const ZERO: Self = 0 as u16;
        const MAX: Self = Self::MAX;
        const LOWEST: Self = Self::MIN;
        const SIGNED: bool = false;
        const FLOAT: bool = false;
        const DIGITS: u32 = 16;
        const MAX_EXPONENT: u32 = 17;
        fn negative(self) -> bool {
            self < 0 as u16
        }
        fn finite(self) -> bool {
            true
        }
        fn nan() -> Self {
            0
        }
        fn overflow() -> Self {
            Self::MAX
        }
        fn underflow() -> Self {
            Self::MIN
        }
    }
    impl sealed::Primitive for i32 {}
    impl Numeric for i32 {
        const ZERO: Self = 0 as i32;
        const MAX: Self = Self::MAX;
        const LOWEST: Self = Self::MIN;
        const SIGNED: bool = true;
        const FLOAT: bool = false;
        const DIGITS: u32 = 31;
        const MAX_EXPONENT: u32 = 32;
        fn negative(self) -> bool {
            self < 0 as i32
        }
        fn finite(self) -> bool {
            true
        }
        fn nan() -> Self {
            0
        }
        fn overflow() -> Self {
            Self::MAX
        }
        fn underflow() -> Self {
            Self::MIN
        }
    }
    impl sealed::Primitive for u32 {}
    impl Numeric for u32 {
        const ZERO: Self = 0 as u32;
        const MAX: Self = Self::MAX;
        const LOWEST: Self = Self::MIN;
        const SIGNED: bool = false;
        const FLOAT: bool = false;
        const DIGITS: u32 = 32;
        const MAX_EXPONENT: u32 = 33;
        fn negative(self) -> bool {
            self < 0 as u32
        }
        fn finite(self) -> bool {
            true
        }
        fn nan() -> Self {
            0
        }
        fn overflow() -> Self {
            Self::MAX
        }
        fn underflow() -> Self {
            Self::MIN
        }
    }
    impl sealed::Primitive for i64 {}
    impl Numeric for i64 {
        const ZERO: Self = 0 as i64;
        const MAX: Self = Self::MAX;
        const LOWEST: Self = Self::MIN;
        const SIGNED: bool = true;
        const FLOAT: bool = false;
        const DIGITS: u32 = 63;
        const MAX_EXPONENT: u32 = 64;
        fn negative(self) -> bool {
            self < 0 as i64
        }
        fn finite(self) -> bool {
            true
        }
        fn nan() -> Self {
            0
        }
        fn overflow() -> Self {
            Self::MAX
        }
        fn underflow() -> Self {
            Self::MIN
        }
    }
    impl sealed::Primitive for u64 {}
    impl Numeric for u64 {
        const ZERO: Self = 0 as u64;
        const MAX: Self = Self::MAX;
        const LOWEST: Self = Self::MIN;
        const SIGNED: bool = false;
        const FLOAT: bool = false;
        const DIGITS: u32 = 64;
        const MAX_EXPONENT: u32 = 65;
        fn negative(self) -> bool {
            self < 0 as u64
        }
        fn finite(self) -> bool {
            true
        }
        fn nan() -> Self {
            0
        }
        fn overflow() -> Self {
            Self::MAX
        }
        fn underflow() -> Self {
            Self::MIN
        }
    }
    impl sealed::Primitive for f32 {}
    impl Numeric for f32 {
        const ZERO: Self = 0 as f32;
        const MAX: Self = Self::MAX;
        const LOWEST: Self = Self::MIN;
        const SIGNED: bool = true;
        const FLOAT: bool = true;
        const DIGITS: u32 = 24;
        const MAX_EXPONENT: u32 = 128;
        fn negative(self) -> bool {
            self < 0 as f32
        }
        fn finite(self) -> bool {
            self.is_finite()
        }
        fn nan() -> Self {
            Self::NAN
        }
        fn overflow() -> Self {
            Self::INFINITY
        }
        fn underflow() -> Self {
            Self::NEG_INFINITY
        }
    }
    impl sealed::Primitive for f64 {}
    impl Numeric for f64 {
        const ZERO: Self = 0 as f64;
        const MAX: Self = Self::MAX;
        const LOWEST: Self = Self::MIN;
        const SIGNED: bool = true;
        const FLOAT: bool = true;
        const DIGITS: u32 = 53;
        const MAX_EXPONENT: u32 = 1024;
        fn negative(self) -> bool {
            self < 0 as f64
        }
        fn finite(self) -> bool {
            self.is_finite()
        }
        fn nan() -> Self {
            Self::NAN
        }
        fn overflow() -> Self {
            Self::INFINITY
        }
        fn underflow() -> Self {
            Self::NEG_INFINITY
        }
    }
    impl sealed::Primitive for isize {}
    impl Numeric for isize {
        const ZERO: Self = 0 as isize;
        const MAX: Self = Self::MAX;
        const LOWEST: Self = Self::MIN;
        const SIGNED: bool = true;
        const FLOAT: bool = false;
        const DIGITS: u32 = 63;
        const MAX_EXPONENT: u32 = 64;
        fn negative(self) -> bool {
            self < 0 as isize
        }
        fn finite(self) -> bool {
            true
        }
        fn nan() -> Self {
            0
        }
        fn overflow() -> Self {
            Self::MAX
        }
        fn underflow() -> Self {
            Self::MIN
        }
    }
    impl sealed::Primitive for usize {}
    impl Numeric for usize {
        const ZERO: Self = 0 as usize;
        const MAX: Self = Self::MAX;
        const LOWEST: Self = Self::MIN;
        const SIGNED: bool = false;
        const FLOAT: bool = false;
        const DIGITS: u32 = 64;
        const MAX_EXPONENT: u32 = 65;
        fn negative(self) -> bool {
            self < 0 as usize
        }
        fn finite(self) -> bool {
            true
        }
        fn nan() -> Self {
            0
        }
        fn overflow() -> Self {
            Self::MAX
        }
        fn underflow() -> Self {
            Self::MIN
        }
    }
    impl Cast<i8> for i8 {
        const CONTAINED: bool = true;
        fn cast(self) -> i8 {
            self as i8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> i8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,i8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,i8>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { -128 as i8 } else { 127 as i8 };
            if <i8 as Cast<i8>>::range(self).IsValid() {
                self as i8
            } else {
                saturated
            }
        }
    }
    impl StrictCast<i8> for i8 {}
    impl Promote<i8> for i8 {
        type Max = i8;
        type Lowest = i8;
        type Big = i8;
        type Fast = i16;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i8) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<u8> for i8 {
        const CONTAINED: bool = false;
        fn cast(self) -> u8 {
            self as u8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, true)
        }
        fn default_saturate(self) -> u8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,i8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,i8>::Do(self)}
            }
            let is_min = true || (!false && self.negative());
            let saturated = if is_min { 0 as u8 } else { 127 as u8 };
            if <i8 as Cast<u8>>::range(self).IsValid() {
                self as u8
            } else {
                saturated
            }
        }
    }
    impl Promote<u8> for i8 {
        type Max = u8;
        type Lowest = i8;
        type Big = i16;
        type Fast = i16;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u8) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<i16> for i8 {
        const CONTAINED: bool = true;
        fn cast(self) -> i16 {
            self as i16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> i16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,i8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,i8>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { -128 as i16 } else { 127 as i16 };
            if <i8 as Cast<i16>>::range(self).IsValid() {
                self as i16
            } else {
                saturated
            }
        }
    }
    impl StrictCast<i16> for i8 {}
    impl Promote<i16> for i8 {
        type Max = i16;
        type Lowest = i16;
        type Big = i16;
        type Fast = i32;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i16) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<u16> for i8 {
        const CONTAINED: bool = false;
        fn cast(self) -> u16 {
            self as u16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, true)
        }
        fn default_saturate(self) -> u16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,i8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,i8>::Do(self)}
            }
            let is_min = true || (!false && self.negative());
            let saturated = if is_min { 0 as u16 } else { 127 as u16 };
            if <i8 as Cast<u16>>::range(self).IsValid() {
                self as u16
            } else {
                saturated
            }
        }
    }
    impl Promote<u16> for i8 {
        type Max = u16;
        type Lowest = i8;
        type Big = i32;
        type Fast = i32;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u16) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<i32> for i8 {
        const CONTAINED: bool = true;
        fn cast(self) -> i32 {
            self as i32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> i32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,i8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,i8>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { -128 as i32 } else { 127 as i32 };
            if <i8 as Cast<i32>>::range(self).IsValid() {
                self as i32
            } else {
                saturated
            }
        }
    }
    impl StrictCast<i32> for i8 {}
    impl Promote<i32> for i8 {
        type Max = i32;
        type Lowest = i32;
        type Big = i32;
        type Fast = i64;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i32) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<u32> for i8 {
        const CONTAINED: bool = false;
        fn cast(self) -> u32 {
            self as u32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, true)
        }
        fn default_saturate(self) -> u32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,i8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,i8>::Do(self)}
            }
            let is_min = true || (!false && self.negative());
            let saturated = if is_min { 0 as u32 } else { 127 as u32 };
            if <i8 as Cast<u32>>::range(self).IsValid() {
                self as u32
            } else {
                saturated
            }
        }
    }
    impl Promote<u32> for i8 {
        type Max = u32;
        type Lowest = i8;
        type Big = i64;
        type Fast = i64;
        type Sum = u32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u32) -> (u32, u32) {
            (self as u32, r as u32)
        }
    }
    impl Cast<i64> for i8 {
        const CONTAINED: bool = true;
        fn cast(self) -> i64 {
            self as i64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> i64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,i8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,i8>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { -128 as i64 } else { 127 as i64 };
            if <i8 as Cast<i64>>::range(self).IsValid() {
                self as i64
            } else {
                saturated
            }
        }
    }
    impl StrictCast<i64> for i8 {}
    impl Promote<i64> for i8 {
        type Max = i64;
        type Lowest = i64;
        type Big = i64;
        type Fast = i64;
        type Sum = i64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i64) -> (i64, i64) {
            (self as i64, r as i64)
        }
    }
    impl Cast<u64> for i8 {
        const CONTAINED: bool = false;
        fn cast(self) -> u64 {
            self as u64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, true)
        }
        fn default_saturate(self) -> u64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,i8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,i8>::Do(self)}
            }
            let is_min = true || (!false && self.negative());
            let saturated = if is_min { 0 as u64 } else { 127 as u64 };
            if <i8 as Cast<u64>>::range(self).IsValid() {
                self as u64
            } else {
                saturated
            }
        }
    }
    impl Promote<u64> for i8 {
        type Max = u64;
        type Lowest = i8;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = false;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u64) -> (u64, u64) {
            (self as u64, r as u64)
        }
    }
    impl Cast<f32> for i8 {
        const CONTAINED: bool = true;
        fn cast(self) -> f32 {
            self as f32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f32> for i8 {}
    impl Promote<f32> for i8 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f32) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<f64> for i8 {
        const CONTAINED: bool = true;
        fn cast(self) -> f64 {
            self as f64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f64> for i8 {}
    impl Promote<f64> for i8 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f64) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<isize> for i8 {
        const CONTAINED: bool = true;
        fn cast(self) -> isize {
            self as isize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> isize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,i8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,i8>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { -128 as isize } else { 127 as isize };
            if <i8 as Cast<isize>>::range(self).IsValid() {
                self as isize
            } else {
                saturated
            }
        }
    }
    impl StrictCast<isize> for i8 {}
    impl Promote<isize> for i8 {
        type Max = isize;
        type Lowest = isize;
        type Big = isize;
        type Fast = isize;
        type Sum = isize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: isize) -> (isize, isize) {
            (self as isize, r as isize)
        }
    }
    impl Cast<usize> for i8 {
        const CONTAINED: bool = false;
        fn cast(self) -> usize {
            self as usize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, true)
        }
        fn default_saturate(self) -> usize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,i8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,i8>::Do(self)}
            }
            let is_min = true || (!false && self.negative());
            let saturated = if is_min { 0 as usize } else { 127 as usize };
            if <i8 as Cast<usize>>::range(self).IsValid() {
                self as usize
            } else {
                saturated
            }
        }
    }
    impl Promote<usize> for i8 {
        type Max = usize;
        type Lowest = i8;
        type Big = usize;
        type Fast = usize;
        type Sum = usize;
        const BIG_CONTAINED: bool = false;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: usize) -> (usize, usize) {
            (self as usize, r as usize)
        }
    }
    impl Cast<i8> for u8 {
        const CONTAINED: bool = false;
        fn cast(self) -> i8 {
            self as i8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (i8::MAX as u8))
        }
        fn default_saturate(self) -> i8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,u8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,u8>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as i8 } else { 127 as i8 };
            if <u8 as Cast<i8>>::range(self).IsValid() {
                self as i8
            } else {
                saturated
            }
        }
    }
    impl Promote<i8> for u8 {
        type Max = u8;
        type Lowest = i8;
        type Big = i16;
        type Fast = i16;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i8) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<u8> for u8 {
        const CONTAINED: bool = true;
        fn cast(self) -> u8 {
            self as u8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> u8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,u8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,u8>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as u8 } else { 255 as u8 };
            if <u8 as Cast<u8>>::range(self).IsValid() {
                self as u8
            } else {
                saturated
            }
        }
    }
    impl StrictCast<u8> for u8 {}
    impl Promote<u8> for u8 {
        type Max = u8;
        type Lowest = u8;
        type Big = u8;
        type Fast = u16;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u8) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<i16> for u8 {
        const CONTAINED: bool = true;
        fn cast(self) -> i16 {
            self as i16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> i16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,u8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,u8>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as i16 } else { 255 as i16 };
            if <u8 as Cast<i16>>::range(self).IsValid() {
                self as i16
            } else {
                saturated
            }
        }
    }
    impl StrictCast<i16> for u8 {}
    impl Promote<i16> for u8 {
        type Max = i16;
        type Lowest = i16;
        type Big = i16;
        type Fast = i32;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i16) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<u16> for u8 {
        const CONTAINED: bool = true;
        fn cast(self) -> u16 {
            self as u16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> u16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,u8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,u8>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as u16 } else { 255 as u16 };
            if <u8 as Cast<u16>>::range(self).IsValid() {
                self as u16
            } else {
                saturated
            }
        }
    }
    impl StrictCast<u16> for u8 {}
    impl Promote<u16> for u8 {
        type Max = u16;
        type Lowest = u8;
        type Big = u16;
        type Fast = u32;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u16) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<i32> for u8 {
        const CONTAINED: bool = true;
        fn cast(self) -> i32 {
            self as i32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> i32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,u8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,u8>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as i32 } else { 255 as i32 };
            if <u8 as Cast<i32>>::range(self).IsValid() {
                self as i32
            } else {
                saturated
            }
        }
    }
    impl StrictCast<i32> for u8 {}
    impl Promote<i32> for u8 {
        type Max = i32;
        type Lowest = i32;
        type Big = i32;
        type Fast = i64;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i32) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<u32> for u8 {
        const CONTAINED: bool = true;
        fn cast(self) -> u32 {
            self as u32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> u32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,u8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,u8>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as u32 } else { 255 as u32 };
            if <u8 as Cast<u32>>::range(self).IsValid() {
                self as u32
            } else {
                saturated
            }
        }
    }
    impl StrictCast<u32> for u8 {}
    impl Promote<u32> for u8 {
        type Max = u32;
        type Lowest = u8;
        type Big = u32;
        type Fast = u64;
        type Sum = u32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u32) -> (u32, u32) {
            (self as u32, r as u32)
        }
    }
    impl Cast<i64> for u8 {
        const CONTAINED: bool = true;
        fn cast(self) -> i64 {
            self as i64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> i64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,u8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,u8>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as i64 } else { 255 as i64 };
            if <u8 as Cast<i64>>::range(self).IsValid() {
                self as i64
            } else {
                saturated
            }
        }
    }
    impl StrictCast<i64> for u8 {}
    impl Promote<i64> for u8 {
        type Max = i64;
        type Lowest = i64;
        type Big = i64;
        type Fast = i64;
        type Sum = i64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i64) -> (i64, i64) {
            (self as i64, r as i64)
        }
    }
    impl Cast<u64> for u8 {
        const CONTAINED: bool = true;
        fn cast(self) -> u64 {
            self as u64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> u64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,u8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,u8>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as u64 } else { 255 as u64 };
            if <u8 as Cast<u64>>::range(self).IsValid() {
                self as u64
            } else {
                saturated
            }
        }
    }
    impl StrictCast<u64> for u8 {}
    impl Promote<u64> for u8 {
        type Max = u64;
        type Lowest = u8;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u64) -> (u64, u64) {
            (self as u64, r as u64)
        }
    }
    impl Cast<f32> for u8 {
        const CONTAINED: bool = true;
        fn cast(self) -> f32 {
            self as f32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f32> for u8 {}
    impl Promote<f32> for u8 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f32) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<f64> for u8 {
        const CONTAINED: bool = true;
        fn cast(self) -> f64 {
            self as f64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f64> for u8 {}
    impl Promote<f64> for u8 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f64) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<isize> for u8 {
        const CONTAINED: bool = true;
        fn cast(self) -> isize {
            self as isize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> isize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,u8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,u8>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as isize } else { 255 as isize };
            if <u8 as Cast<isize>>::range(self).IsValid() {
                self as isize
            } else {
                saturated
            }
        }
    }
    impl StrictCast<isize> for u8 {}
    impl Promote<isize> for u8 {
        type Max = isize;
        type Lowest = isize;
        type Big = isize;
        type Fast = isize;
        type Sum = isize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: isize) -> (isize, isize) {
            (self as isize, r as isize)
        }
    }
    impl Cast<usize> for u8 {
        const CONTAINED: bool = true;
        fn cast(self) -> usize {
            self as usize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> usize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,u8>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,u8>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as usize } else { 255 as usize };
            if <u8 as Cast<usize>>::range(self).IsValid() {
                self as usize
            } else {
                saturated
            }
        }
    }
    impl StrictCast<usize> for u8 {}
    impl Promote<usize> for u8 {
        type Max = usize;
        type Lowest = u8;
        type Big = usize;
        type Fast = usize;
        type Sum = usize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: usize) -> (usize, usize) {
            (self as usize, r as usize)
        }
    }
    impl Cast<i8> for i16 {
        const CONTAINED: bool = false;
        fn cast(self) -> i8 {
            self as i8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= (i8::MIN as i16), self <= (i8::MAX as i16))
        }
        fn default_saturate(self) -> i8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,i16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,i16>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min { -128 as i8 } else { 127 as i8 };
            if <i16 as Cast<i8>>::range(self).IsValid() {
                self as i8
            } else {
                saturated
            }
        }
    }
    impl Promote<i8> for i16 {
        type Max = i16;
        type Lowest = i16;
        type Big = i16;
        type Fast = i32;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i8) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<u8> for i16 {
        const CONTAINED: bool = false;
        fn cast(self) -> u8 {
            self as u8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, self <= (u8::MAX as i16))
        }
        fn default_saturate(self) -> u8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,i16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,i16>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min { 0 as u8 } else { 255 as u8 };
            if <i16 as Cast<u8>>::range(self).IsValid() {
                self as u8
            } else {
                saturated
            }
        }
    }
    impl Promote<u8> for i16 {
        type Max = i16;
        type Lowest = i16;
        type Big = i16;
        type Fast = i32;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u8) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<i16> for i16 {
        const CONTAINED: bool = true;
        fn cast(self) -> i16 {
            self as i16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> i16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,i16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,i16>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { -32768 as i16 } else { 32767 as i16 };
            if <i16 as Cast<i16>>::range(self).IsValid() {
                self as i16
            } else {
                saturated
            }
        }
    }
    impl StrictCast<i16> for i16 {}
    impl Promote<i16> for i16 {
        type Max = i16;
        type Lowest = i16;
        type Big = i16;
        type Fast = i32;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i16) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<u16> for i16 {
        const CONTAINED: bool = false;
        fn cast(self) -> u16 {
            self as u16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, true)
        }
        fn default_saturate(self) -> u16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,i16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,i16>::Do(self)}
            }
            let is_min = true || (!false && self.negative());
            let saturated = if is_min { 0 as u16 } else { 32767 as u16 };
            if <i16 as Cast<u16>>::range(self).IsValid() {
                self as u16
            } else {
                saturated
            }
        }
    }
    impl Promote<u16> for i16 {
        type Max = u16;
        type Lowest = i16;
        type Big = i32;
        type Fast = i32;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u16) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<i32> for i16 {
        const CONTAINED: bool = true;
        fn cast(self) -> i32 {
            self as i32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> i32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,i16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,i16>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { -32768 as i32 } else { 32767 as i32 };
            if <i16 as Cast<i32>>::range(self).IsValid() {
                self as i32
            } else {
                saturated
            }
        }
    }
    impl StrictCast<i32> for i16 {}
    impl Promote<i32> for i16 {
        type Max = i32;
        type Lowest = i32;
        type Big = i32;
        type Fast = i64;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i32) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<u32> for i16 {
        const CONTAINED: bool = false;
        fn cast(self) -> u32 {
            self as u32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, true)
        }
        fn default_saturate(self) -> u32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,i16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,i16>::Do(self)}
            }
            let is_min = true || (!false && self.negative());
            let saturated = if is_min { 0 as u32 } else { 32767 as u32 };
            if <i16 as Cast<u32>>::range(self).IsValid() {
                self as u32
            } else {
                saturated
            }
        }
    }
    impl Promote<u32> for i16 {
        type Max = u32;
        type Lowest = i16;
        type Big = i64;
        type Fast = i64;
        type Sum = u32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u32) -> (u32, u32) {
            (self as u32, r as u32)
        }
    }
    impl Cast<i64> for i16 {
        const CONTAINED: bool = true;
        fn cast(self) -> i64 {
            self as i64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> i64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,i16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,i16>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { -32768 as i64 } else { 32767 as i64 };
            if <i16 as Cast<i64>>::range(self).IsValid() {
                self as i64
            } else {
                saturated
            }
        }
    }
    impl StrictCast<i64> for i16 {}
    impl Promote<i64> for i16 {
        type Max = i64;
        type Lowest = i64;
        type Big = i64;
        type Fast = i64;
        type Sum = i64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i64) -> (i64, i64) {
            (self as i64, r as i64)
        }
    }
    impl Cast<u64> for i16 {
        const CONTAINED: bool = false;
        fn cast(self) -> u64 {
            self as u64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, true)
        }
        fn default_saturate(self) -> u64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,i16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,i16>::Do(self)}
            }
            let is_min = true || (!false && self.negative());
            let saturated = if is_min { 0 as u64 } else { 32767 as u64 };
            if <i16 as Cast<u64>>::range(self).IsValid() {
                self as u64
            } else {
                saturated
            }
        }
    }
    impl Promote<u64> for i16 {
        type Max = u64;
        type Lowest = i16;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = false;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u64) -> (u64, u64) {
            (self as u64, r as u64)
        }
    }
    impl Cast<f32> for i16 {
        const CONTAINED: bool = true;
        fn cast(self) -> f32 {
            self as f32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f32> for i16 {}
    impl Promote<f32> for i16 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f32) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<f64> for i16 {
        const CONTAINED: bool = true;
        fn cast(self) -> f64 {
            self as f64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f64> for i16 {}
    impl Promote<f64> for i16 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f64) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<isize> for i16 {
        const CONTAINED: bool = true;
        fn cast(self) -> isize {
            self as isize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> isize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,i16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,i16>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min {
                -32768 as isize
            } else {
                32767 as isize
            };
            if <i16 as Cast<isize>>::range(self).IsValid() {
                self as isize
            } else {
                saturated
            }
        }
    }
    impl StrictCast<isize> for i16 {}
    impl Promote<isize> for i16 {
        type Max = isize;
        type Lowest = isize;
        type Big = isize;
        type Fast = isize;
        type Sum = isize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: isize) -> (isize, isize) {
            (self as isize, r as isize)
        }
    }
    impl Cast<usize> for i16 {
        const CONTAINED: bool = false;
        fn cast(self) -> usize {
            self as usize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, true)
        }
        fn default_saturate(self) -> usize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,i16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,i16>::Do(self)}
            }
            let is_min = true || (!false && self.negative());
            let saturated = if is_min { 0 as usize } else { 32767 as usize };
            if <i16 as Cast<usize>>::range(self).IsValid() {
                self as usize
            } else {
                saturated
            }
        }
    }
    impl Promote<usize> for i16 {
        type Max = usize;
        type Lowest = i16;
        type Big = usize;
        type Fast = usize;
        type Sum = usize;
        const BIG_CONTAINED: bool = false;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: usize) -> (usize, usize) {
            (self as usize, r as usize)
        }
    }
    impl Cast<i8> for u16 {
        const CONTAINED: bool = false;
        fn cast(self) -> i8 {
            self as i8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (i8::MAX as u16))
        }
        fn default_saturate(self) -> i8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,u16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,u16>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as i8 } else { 127 as i8 };
            if <u16 as Cast<i8>>::range(self).IsValid() {
                self as i8
            } else {
                saturated
            }
        }
    }
    impl Promote<i8> for u16 {
        type Max = u16;
        type Lowest = i8;
        type Big = i32;
        type Fast = i32;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i8) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<u8> for u16 {
        const CONTAINED: bool = false;
        fn cast(self) -> u8 {
            self as u8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (u8::MAX as u16))
        }
        fn default_saturate(self) -> u8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,u16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,u16>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as u8 } else { 255 as u8 };
            if <u16 as Cast<u8>>::range(self).IsValid() {
                self as u8
            } else {
                saturated
            }
        }
    }
    impl Promote<u8> for u16 {
        type Max = u16;
        type Lowest = u8;
        type Big = u16;
        type Fast = u32;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u8) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<i16> for u16 {
        const CONTAINED: bool = false;
        fn cast(self) -> i16 {
            self as i16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (i16::MAX as u16))
        }
        fn default_saturate(self) -> i16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,u16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,u16>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as i16 } else { 32767 as i16 };
            if <u16 as Cast<i16>>::range(self).IsValid() {
                self as i16
            } else {
                saturated
            }
        }
    }
    impl Promote<i16> for u16 {
        type Max = u16;
        type Lowest = i16;
        type Big = i32;
        type Fast = i32;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i16) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<u16> for u16 {
        const CONTAINED: bool = true;
        fn cast(self) -> u16 {
            self as u16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> u16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,u16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,u16>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as u16 } else { 65535 as u16 };
            if <u16 as Cast<u16>>::range(self).IsValid() {
                self as u16
            } else {
                saturated
            }
        }
    }
    impl StrictCast<u16> for u16 {}
    impl Promote<u16> for u16 {
        type Max = u16;
        type Lowest = u16;
        type Big = u16;
        type Fast = u32;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u16) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<i32> for u16 {
        const CONTAINED: bool = true;
        fn cast(self) -> i32 {
            self as i32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> i32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,u16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,u16>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as i32 } else { 65535 as i32 };
            if <u16 as Cast<i32>>::range(self).IsValid() {
                self as i32
            } else {
                saturated
            }
        }
    }
    impl StrictCast<i32> for u16 {}
    impl Promote<i32> for u16 {
        type Max = i32;
        type Lowest = i32;
        type Big = i32;
        type Fast = i64;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i32) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<u32> for u16 {
        const CONTAINED: bool = true;
        fn cast(self) -> u32 {
            self as u32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> u32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,u16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,u16>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as u32 } else { 65535 as u32 };
            if <u16 as Cast<u32>>::range(self).IsValid() {
                self as u32
            } else {
                saturated
            }
        }
    }
    impl StrictCast<u32> for u16 {}
    impl Promote<u32> for u16 {
        type Max = u32;
        type Lowest = u16;
        type Big = u32;
        type Fast = u64;
        type Sum = u32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u32) -> (u32, u32) {
            (self as u32, r as u32)
        }
    }
    impl Cast<i64> for u16 {
        const CONTAINED: bool = true;
        fn cast(self) -> i64 {
            self as i64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> i64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,u16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,u16>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as i64 } else { 65535 as i64 };
            if <u16 as Cast<i64>>::range(self).IsValid() {
                self as i64
            } else {
                saturated
            }
        }
    }
    impl StrictCast<i64> for u16 {}
    impl Promote<i64> for u16 {
        type Max = i64;
        type Lowest = i64;
        type Big = i64;
        type Fast = i64;
        type Sum = i64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i64) -> (i64, i64) {
            (self as i64, r as i64)
        }
    }
    impl Cast<u64> for u16 {
        const CONTAINED: bool = true;
        fn cast(self) -> u64 {
            self as u64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> u64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,u16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,u16>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as u64 } else { 65535 as u64 };
            if <u16 as Cast<u64>>::range(self).IsValid() {
                self as u64
            } else {
                saturated
            }
        }
    }
    impl StrictCast<u64> for u16 {}
    impl Promote<u64> for u16 {
        type Max = u64;
        type Lowest = u16;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u64) -> (u64, u64) {
            (self as u64, r as u64)
        }
    }
    impl Cast<f32> for u16 {
        const CONTAINED: bool = true;
        fn cast(self) -> f32 {
            self as f32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f32> for u16 {}
    impl Promote<f32> for u16 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f32) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<f64> for u16 {
        const CONTAINED: bool = true;
        fn cast(self) -> f64 {
            self as f64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f64> for u16 {}
    impl Promote<f64> for u16 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f64) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<isize> for u16 {
        const CONTAINED: bool = true;
        fn cast(self) -> isize {
            self as isize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> isize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,u16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,u16>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as isize } else { 65535 as isize };
            if <u16 as Cast<isize>>::range(self).IsValid() {
                self as isize
            } else {
                saturated
            }
        }
    }
    impl StrictCast<isize> for u16 {}
    impl Promote<isize> for u16 {
        type Max = isize;
        type Lowest = isize;
        type Big = isize;
        type Fast = isize;
        type Sum = isize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: isize) -> (isize, isize) {
            (self as isize, r as isize)
        }
    }
    impl Cast<usize> for u16 {
        const CONTAINED: bool = true;
        fn cast(self) -> usize {
            self as usize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> usize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,u16>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,u16>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min { 0 as usize } else { 65535 as usize };
            if <u16 as Cast<usize>>::range(self).IsValid() {
                self as usize
            } else {
                saturated
            }
        }
    }
    impl StrictCast<usize> for u16 {}
    impl Promote<usize> for u16 {
        type Max = usize;
        type Lowest = u16;
        type Big = usize;
        type Fast = usize;
        type Sum = usize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: usize) -> (usize, usize) {
            (self as usize, r as usize)
        }
    }
    impl Cast<i8> for i32 {
        const CONTAINED: bool = false;
        fn cast(self) -> i8 {
            self as i8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= (i8::MIN as i32), self <= (i8::MAX as i32))
        }
        fn default_saturate(self) -> i8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,i32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,i32>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min { -128 as i8 } else { 127 as i8 };
            if <i32 as Cast<i8>>::range(self).IsValid() {
                self as i8
            } else {
                saturated
            }
        }
    }
    impl Promote<i8> for i32 {
        type Max = i32;
        type Lowest = i32;
        type Big = i32;
        type Fast = i64;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i8) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<u8> for i32 {
        const CONTAINED: bool = false;
        fn cast(self) -> u8 {
            self as u8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, self <= (u8::MAX as i32))
        }
        fn default_saturate(self) -> u8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,i32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,i32>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min { 0 as u8 } else { 255 as u8 };
            if <i32 as Cast<u8>>::range(self).IsValid() {
                self as u8
            } else {
                saturated
            }
        }
    }
    impl Promote<u8> for i32 {
        type Max = i32;
        type Lowest = i32;
        type Big = i32;
        type Fast = i64;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u8) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<i16> for i32 {
        const CONTAINED: bool = false;
        fn cast(self) -> i16 {
            self as i16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= (i16::MIN as i32), self <= (i16::MAX as i32))
        }
        fn default_saturate(self) -> i16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,i32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,i32>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min { -32768 as i16 } else { 32767 as i16 };
            if <i32 as Cast<i16>>::range(self).IsValid() {
                self as i16
            } else {
                saturated
            }
        }
    }
    impl Promote<i16> for i32 {
        type Max = i32;
        type Lowest = i32;
        type Big = i32;
        type Fast = i64;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i16) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<u16> for i32 {
        const CONTAINED: bool = false;
        fn cast(self) -> u16 {
            self as u16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, self <= (u16::MAX as i32))
        }
        fn default_saturate(self) -> u16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,i32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,i32>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min { 0 as u16 } else { 65535 as u16 };
            if <i32 as Cast<u16>>::range(self).IsValid() {
                self as u16
            } else {
                saturated
            }
        }
    }
    impl Promote<u16> for i32 {
        type Max = i32;
        type Lowest = i32;
        type Big = i32;
        type Fast = i64;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u16) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<i32> for i32 {
        const CONTAINED: bool = true;
        fn cast(self) -> i32 {
            self as i32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> i32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,i32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,i32>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min {
                -2147483648 as i32
            } else {
                2147483647 as i32
            };
            if <i32 as Cast<i32>>::range(self).IsValid() {
                self as i32
            } else {
                saturated
            }
        }
    }
    impl StrictCast<i32> for i32 {}
    impl Promote<i32> for i32 {
        type Max = i32;
        type Lowest = i32;
        type Big = i32;
        type Fast = i64;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i32) -> (i32, i32) {
            (self as i32, r as i32)
        }
    }
    impl Cast<u32> for i32 {
        const CONTAINED: bool = false;
        fn cast(self) -> u32 {
            self as u32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, true)
        }
        fn default_saturate(self) -> u32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,i32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,i32>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min { 0 as u32 } else { 4294967295 as u32 };
            if <i32 as Cast<u32>>::range(self).IsValid() {
                self as u32
            } else {
                saturated
            }
        }
    }
    impl Promote<u32> for i32 {
        type Max = u32;
        type Lowest = i32;
        type Big = i64;
        type Fast = i64;
        type Sum = u32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u32) -> (u32, u32) {
            (self as u32, r as u32)
        }
    }
    impl Cast<i64> for i32 {
        const CONTAINED: bool = true;
        fn cast(self) -> i64 {
            self as i64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> i64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,i32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,i32>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min {
                -9223372036854775808 as i64
            } else {
                9223372036854775807 as i64
            };
            if <i32 as Cast<i64>>::range(self).IsValid() {
                self as i64
            } else {
                saturated
            }
        }
    }
    impl StrictCast<i64> for i32 {}
    impl Promote<i64> for i32 {
        type Max = i64;
        type Lowest = i64;
        type Big = i64;
        type Fast = i64;
        type Sum = i64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i64) -> (i64, i64) {
            (self as i64, r as i64)
        }
    }
    impl Cast<u64> for i32 {
        const CONTAINED: bool = false;
        fn cast(self) -> u64 {
            self as u64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, true)
        }
        fn default_saturate(self) -> u64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,i32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,i32>::Do(self)}
            }
            let is_min = true || (!false && self.negative());
            let saturated = if is_min {
                0 as u64
            } else {
                9223372036854775807 as u64
            };
            if <i32 as Cast<u64>>::range(self).IsValid() {
                self as u64
            } else {
                saturated
            }
        }
    }
    impl Promote<u64> for i32 {
        type Max = u64;
        type Lowest = i32;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = false;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u64) -> (u64, u64) {
            (self as u64, r as u64)
        }
    }
    impl Cast<f32> for i32 {
        const CONTAINED: bool = true;
        fn cast(self) -> f32 {
            self as f32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f32> for i32 {}
    impl Promote<f32> for i32 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f32) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<f64> for i32 {
        const CONTAINED: bool = true;
        fn cast(self) -> f64 {
            self as f64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f64> for i32 {}
    impl Promote<f64> for i32 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f64) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<isize> for i32 {
        const CONTAINED: bool = true;
        fn cast(self) -> isize {
            self as isize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> isize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,i32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,i32>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min {
                -9223372036854775808 as isize
            } else {
                9223372036854775807 as isize
            };
            if <i32 as Cast<isize>>::range(self).IsValid() {
                self as isize
            } else {
                saturated
            }
        }
    }
    impl StrictCast<isize> for i32 {}
    impl Promote<isize> for i32 {
        type Max = isize;
        type Lowest = isize;
        type Big = isize;
        type Fast = isize;
        type Sum = isize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: isize) -> (isize, isize) {
            (self as isize, r as isize)
        }
    }
    impl Cast<usize> for i32 {
        const CONTAINED: bool = false;
        fn cast(self) -> usize {
            self as usize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, true)
        }
        fn default_saturate(self) -> usize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,i32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,i32>::Do(self)}
            }
            let is_min = true || (!false && self.negative());
            let saturated = if is_min {
                0 as usize
            } else {
                9223372036854775807 as usize
            };
            if <i32 as Cast<usize>>::range(self).IsValid() {
                self as usize
            } else {
                saturated
            }
        }
    }
    impl Promote<usize> for i32 {
        type Max = usize;
        type Lowest = i32;
        type Big = usize;
        type Fast = usize;
        type Sum = usize;
        const BIG_CONTAINED: bool = false;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: usize) -> (usize, usize) {
            (self as usize, r as usize)
        }
    }
    impl Cast<i8> for u32 {
        const CONTAINED: bool = false;
        fn cast(self) -> i8 {
            self as i8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (i8::MAX as u32))
        }
        fn default_saturate(self) -> i8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,u32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,u32>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as i8 } else { 127 as i8 };
            if <u32 as Cast<i8>>::range(self).IsValid() {
                self as i8
            } else {
                saturated
            }
        }
    }
    impl Promote<i8> for u32 {
        type Max = u32;
        type Lowest = i8;
        type Big = i64;
        type Fast = i64;
        type Sum = u32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i8) -> (u32, u32) {
            (self as u32, r as u32)
        }
    }
    impl Cast<u8> for u32 {
        const CONTAINED: bool = false;
        fn cast(self) -> u8 {
            self as u8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (u8::MAX as u32))
        }
        fn default_saturate(self) -> u8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,u32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,u32>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as u8 } else { 255 as u8 };
            if <u32 as Cast<u8>>::range(self).IsValid() {
                self as u8
            } else {
                saturated
            }
        }
    }
    impl Promote<u8> for u32 {
        type Max = u32;
        type Lowest = u8;
        type Big = u32;
        type Fast = u64;
        type Sum = u32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u8) -> (u32, u32) {
            (self as u32, r as u32)
        }
    }
    impl Cast<i16> for u32 {
        const CONTAINED: bool = false;
        fn cast(self) -> i16 {
            self as i16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (i16::MAX as u32))
        }
        fn default_saturate(self) -> i16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,u32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,u32>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as i16 } else { 32767 as i16 };
            if <u32 as Cast<i16>>::range(self).IsValid() {
                self as i16
            } else {
                saturated
            }
        }
    }
    impl Promote<i16> for u32 {
        type Max = u32;
        type Lowest = i16;
        type Big = i64;
        type Fast = i64;
        type Sum = u32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i16) -> (u32, u32) {
            (self as u32, r as u32)
        }
    }
    impl Cast<u16> for u32 {
        const CONTAINED: bool = false;
        fn cast(self) -> u16 {
            self as u16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (u16::MAX as u32))
        }
        fn default_saturate(self) -> u16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,u32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,u32>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as u16 } else { 65535 as u16 };
            if <u32 as Cast<u16>>::range(self).IsValid() {
                self as u16
            } else {
                saturated
            }
        }
    }
    impl Promote<u16> for u32 {
        type Max = u32;
        type Lowest = u16;
        type Big = u32;
        type Fast = u64;
        type Sum = u32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u16) -> (u32, u32) {
            (self as u32, r as u32)
        }
    }
    impl Cast<i32> for u32 {
        const CONTAINED: bool = false;
        fn cast(self) -> i32 {
            self as i32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (i32::MAX as u32))
        }
        fn default_saturate(self) -> i32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,u32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,u32>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as i32 } else { 2147483647 as i32 };
            if <u32 as Cast<i32>>::range(self).IsValid() {
                self as i32
            } else {
                saturated
            }
        }
    }
    impl Promote<i32> for u32 {
        type Max = u32;
        type Lowest = i32;
        type Big = i64;
        type Fast = i64;
        type Sum = u32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i32) -> (u32, u32) {
            (self as u32, r as u32)
        }
    }
    impl Cast<u32> for u32 {
        const CONTAINED: bool = true;
        fn cast(self) -> u32 {
            self as u32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> u32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,u32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,u32>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as u32 } else { 4294967295 as u32 };
            if <u32 as Cast<u32>>::range(self).IsValid() {
                self as u32
            } else {
                saturated
            }
        }
    }
    impl StrictCast<u32> for u32 {}
    impl Promote<u32> for u32 {
        type Max = u32;
        type Lowest = u32;
        type Big = u32;
        type Fast = u64;
        type Sum = u32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u32) -> (u32, u32) {
            (self as u32, r as u32)
        }
    }
    impl Cast<i64> for u32 {
        const CONTAINED: bool = true;
        fn cast(self) -> i64 {
            self as i64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> i64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,u32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,u32>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min {
                0 as i64
            } else {
                9223372036854775807 as i64
            };
            if <u32 as Cast<i64>>::range(self).IsValid() {
                self as i64
            } else {
                saturated
            }
        }
    }
    impl StrictCast<i64> for u32 {}
    impl Promote<i64> for u32 {
        type Max = i64;
        type Lowest = i64;
        type Big = i64;
        type Fast = i64;
        type Sum = i64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i64) -> (i64, i64) {
            (self as i64, r as i64)
        }
    }
    impl Cast<u64> for u32 {
        const CONTAINED: bool = true;
        fn cast(self) -> u64 {
            self as u64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> u64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,u32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,u32>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min {
                0 as u64
            } else {
                18446744073709551615 as u64
            };
            if <u32 as Cast<u64>>::range(self).IsValid() {
                self as u64
            } else {
                saturated
            }
        }
    }
    impl StrictCast<u64> for u32 {}
    impl Promote<u64> for u32 {
        type Max = u64;
        type Lowest = u32;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u64) -> (u64, u64) {
            (self as u64, r as u64)
        }
    }
    impl Cast<f32> for u32 {
        const CONTAINED: bool = true;
        fn cast(self) -> f32 {
            self as f32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f32> for u32 {}
    impl Promote<f32> for u32 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f32) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<f64> for u32 {
        const CONTAINED: bool = true;
        fn cast(self) -> f64 {
            self as f64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f64> for u32 {}
    impl Promote<f64> for u32 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f64) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<isize> for u32 {
        const CONTAINED: bool = true;
        fn cast(self) -> isize {
            self as isize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> isize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,u32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,u32>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min {
                0 as isize
            } else {
                4294967295 as isize
            };
            if <u32 as Cast<isize>>::range(self).IsValid() {
                self as isize
            } else {
                saturated
            }
        }
    }
    impl StrictCast<isize> for u32 {}
    impl Promote<isize> for u32 {
        type Max = isize;
        type Lowest = isize;
        type Big = isize;
        type Fast = isize;
        type Sum = isize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: isize) -> (isize, isize) {
            (self as isize, r as isize)
        }
    }
    impl Cast<usize> for u32 {
        const CONTAINED: bool = true;
        fn cast(self) -> usize {
            self as usize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> usize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,u32>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,u32>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min {
                0 as usize
            } else {
                18446744073709551615 as usize
            };
            if <u32 as Cast<usize>>::range(self).IsValid() {
                self as usize
            } else {
                saturated
            }
        }
    }
    impl StrictCast<usize> for u32 {}
    impl Promote<usize> for u32 {
        type Max = usize;
        type Lowest = u32;
        type Big = usize;
        type Fast = usize;
        type Sum = usize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: usize) -> (usize, usize) {
            (self as usize, r as usize)
        }
    }
    impl Cast<i8> for i64 {
        const CONTAINED: bool = false;
        fn cast(self) -> i8 {
            self as i8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= (i8::MIN as i64), self <= (i8::MAX as i64))
        }
        fn default_saturate(self) -> i8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,i64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,i64>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min { -128 as i8 } else { 127 as i8 };
            if <i64 as Cast<i8>>::range(self).IsValid() {
                self as i8
            } else {
                saturated
            }
        }
    }
    impl Promote<i8> for i64 {
        type Max = i64;
        type Lowest = i64;
        type Big = i64;
        type Fast = i64;
        type Sum = i64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i8) -> (i64, i64) {
            (self as i64, r as i64)
        }
    }
    impl Cast<u8> for i64 {
        const CONTAINED: bool = false;
        fn cast(self) -> u8 {
            self as u8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, self <= (u8::MAX as i64))
        }
        fn default_saturate(self) -> u8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,i64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,i64>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min { 0 as u8 } else { 255 as u8 };
            if <i64 as Cast<u8>>::range(self).IsValid() {
                self as u8
            } else {
                saturated
            }
        }
    }
    impl Promote<u8> for i64 {
        type Max = i64;
        type Lowest = i64;
        type Big = i64;
        type Fast = i64;
        type Sum = i64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u8) -> (i64, i64) {
            (self as i64, r as i64)
        }
    }
    impl Cast<i16> for i64 {
        const CONTAINED: bool = false;
        fn cast(self) -> i16 {
            self as i16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= (i16::MIN as i64), self <= (i16::MAX as i64))
        }
        fn default_saturate(self) -> i16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,i64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,i64>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min { -32768 as i16 } else { 32767 as i16 };
            if <i64 as Cast<i16>>::range(self).IsValid() {
                self as i16
            } else {
                saturated
            }
        }
    }
    impl Promote<i16> for i64 {
        type Max = i64;
        type Lowest = i64;
        type Big = i64;
        type Fast = i64;
        type Sum = i64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i16) -> (i64, i64) {
            (self as i64, r as i64)
        }
    }
    impl Cast<u16> for i64 {
        const CONTAINED: bool = false;
        fn cast(self) -> u16 {
            self as u16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, self <= (u16::MAX as i64))
        }
        fn default_saturate(self) -> u16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,i64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,i64>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min { 0 as u16 } else { 65535 as u16 };
            if <i64 as Cast<u16>>::range(self).IsValid() {
                self as u16
            } else {
                saturated
            }
        }
    }
    impl Promote<u16> for i64 {
        type Max = i64;
        type Lowest = i64;
        type Big = i64;
        type Fast = i64;
        type Sum = i64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u16) -> (i64, i64) {
            (self as i64, r as i64)
        }
    }
    impl Cast<i32> for i64 {
        const CONTAINED: bool = false;
        fn cast(self) -> i32 {
            self as i32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= (i32::MIN as i64), self <= (i32::MAX as i64))
        }
        fn default_saturate(self) -> i32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,i64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,i64>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min {
                -2147483648 as i32
            } else {
                2147483647 as i32
            };
            if <i64 as Cast<i32>>::range(self).IsValid() {
                self as i32
            } else {
                saturated
            }
        }
    }
    impl Promote<i32> for i64 {
        type Max = i64;
        type Lowest = i64;
        type Big = i64;
        type Fast = i64;
        type Sum = i64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i32) -> (i64, i64) {
            (self as i64, r as i64)
        }
    }
    impl Cast<u32> for i64 {
        const CONTAINED: bool = false;
        fn cast(self) -> u32 {
            self as u32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, self <= (u32::MAX as i64))
        }
        fn default_saturate(self) -> u32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,i64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,i64>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min { 0 as u32 } else { 4294967295 as u32 };
            if <i64 as Cast<u32>>::range(self).IsValid() {
                self as u32
            } else {
                saturated
            }
        }
    }
    impl Promote<u32> for i64 {
        type Max = i64;
        type Lowest = i64;
        type Big = i64;
        type Fast = i64;
        type Sum = i64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u32) -> (i64, i64) {
            (self as i64, r as i64)
        }
    }
    impl Cast<i64> for i64 {
        const CONTAINED: bool = true;
        fn cast(self) -> i64 {
            self as i64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> i64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,i64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,i64>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min {
                -9223372036854775808 as i64
            } else {
                9223372036854775807 as i64
            };
            if <i64 as Cast<i64>>::range(self).IsValid() {
                self as i64
            } else {
                saturated
            }
        }
    }
    impl StrictCast<i64> for i64 {}
    impl Promote<i64> for i64 {
        type Max = i64;
        type Lowest = i64;
        type Big = i64;
        type Fast = i64;
        type Sum = i64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i64) -> (i64, i64) {
            (self as i64, r as i64)
        }
    }
    impl Cast<u64> for i64 {
        const CONTAINED: bool = false;
        fn cast(self) -> u64 {
            self as u64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, true)
        }
        fn default_saturate(self) -> u64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,i64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,i64>::Do(self)}
            }
            let is_min = true || (!false && self.negative());
            let saturated = if is_min {
                0 as u64
            } else {
                9223372036854775807 as u64
            };
            if <i64 as Cast<u64>>::range(self).IsValid() {
                self as u64
            } else {
                saturated
            }
        }
    }
    impl Promote<u64> for i64 {
        type Max = u64;
        type Lowest = i64;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = false;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u64) -> (u64, u64) {
            (self as u64, r as u64)
        }
    }
    impl Cast<f32> for i64 {
        const CONTAINED: bool = true;
        fn cast(self) -> f32 {
            self as f32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f32> for i64 {}
    impl Promote<f32> for i64 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f32) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<f64> for i64 {
        const CONTAINED: bool = true;
        fn cast(self) -> f64 {
            self as f64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f64> for i64 {}
    impl Promote<f64> for i64 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f64) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<isize> for i64 {
        const CONTAINED: bool = true;
        fn cast(self) -> isize {
            self as isize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> isize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,i64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,i64>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min {
                -9223372036854775808 as isize
            } else {
                9223372036854775807 as isize
            };
            if <i64 as Cast<isize>>::range(self).IsValid() {
                self as isize
            } else {
                saturated
            }
        }
    }
    impl StrictCast<isize> for i64 {}
    impl Promote<isize> for i64 {
        type Max = isize;
        type Lowest = isize;
        type Big = isize;
        type Fast = isize;
        type Sum = isize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: isize) -> (isize, isize) {
            (self as isize, r as isize)
        }
    }
    impl Cast<usize> for i64 {
        const CONTAINED: bool = false;
        fn cast(self) -> usize {
            self as usize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, true)
        }
        fn default_saturate(self) -> usize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,i64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,i64>::Do(self)}
            }
            let is_min = true || (!false && self.negative());
            let saturated = if is_min {
                0 as usize
            } else {
                9223372036854775807 as usize
            };
            if <i64 as Cast<usize>>::range(self).IsValid() {
                self as usize
            } else {
                saturated
            }
        }
    }
    impl Promote<usize> for i64 {
        type Max = usize;
        type Lowest = i64;
        type Big = usize;
        type Fast = usize;
        type Sum = usize;
        const BIG_CONTAINED: bool = false;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: usize) -> (usize, usize) {
            (self as usize, r as usize)
        }
    }
    impl Cast<i8> for u64 {
        const CONTAINED: bool = false;
        fn cast(self) -> i8 {
            self as i8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (i8::MAX as u64))
        }
        fn default_saturate(self) -> i8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,u64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,u64>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as i8 } else { 127 as i8 };
            if <u64 as Cast<i8>>::range(self).IsValid() {
                self as i8
            } else {
                saturated
            }
        }
    }
    impl Promote<i8> for u64 {
        type Max = u64;
        type Lowest = i8;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = false;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i8) -> (u64, u64) {
            (self as u64, r as u64)
        }
    }
    impl Cast<u8> for u64 {
        const CONTAINED: bool = false;
        fn cast(self) -> u8 {
            self as u8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (u8::MAX as u64))
        }
        fn default_saturate(self) -> u8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,u64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,u64>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as u8 } else { 255 as u8 };
            if <u64 as Cast<u8>>::range(self).IsValid() {
                self as u8
            } else {
                saturated
            }
        }
    }
    impl Promote<u8> for u64 {
        type Max = u64;
        type Lowest = u8;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u8) -> (u64, u64) {
            (self as u64, r as u64)
        }
    }
    impl Cast<i16> for u64 {
        const CONTAINED: bool = false;
        fn cast(self) -> i16 {
            self as i16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (i16::MAX as u64))
        }
        fn default_saturate(self) -> i16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,u64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,u64>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as i16 } else { 32767 as i16 };
            if <u64 as Cast<i16>>::range(self).IsValid() {
                self as i16
            } else {
                saturated
            }
        }
    }
    impl Promote<i16> for u64 {
        type Max = u64;
        type Lowest = i16;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = false;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i16) -> (u64, u64) {
            (self as u64, r as u64)
        }
    }
    impl Cast<u16> for u64 {
        const CONTAINED: bool = false;
        fn cast(self) -> u16 {
            self as u16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (u16::MAX as u64))
        }
        fn default_saturate(self) -> u16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,u64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,u64>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as u16 } else { 65535 as u16 };
            if <u64 as Cast<u16>>::range(self).IsValid() {
                self as u16
            } else {
                saturated
            }
        }
    }
    impl Promote<u16> for u64 {
        type Max = u64;
        type Lowest = u16;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u16) -> (u64, u64) {
            (self as u64, r as u64)
        }
    }
    impl Cast<i32> for u64 {
        const CONTAINED: bool = false;
        fn cast(self) -> i32 {
            self as i32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (i32::MAX as u64))
        }
        fn default_saturate(self) -> i32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,u64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,u64>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as i32 } else { 2147483647 as i32 };
            if <u64 as Cast<i32>>::range(self).IsValid() {
                self as i32
            } else {
                saturated
            }
        }
    }
    impl Promote<i32> for u64 {
        type Max = u64;
        type Lowest = i32;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = false;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i32) -> (u64, u64) {
            (self as u64, r as u64)
        }
    }
    impl Cast<u32> for u64 {
        const CONTAINED: bool = false;
        fn cast(self) -> u32 {
            self as u32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (u32::MAX as u64))
        }
        fn default_saturate(self) -> u32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,u64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,u64>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as u32 } else { 4294967295 as u32 };
            if <u64 as Cast<u32>>::range(self).IsValid() {
                self as u32
            } else {
                saturated
            }
        }
    }
    impl Promote<u32> for u64 {
        type Max = u64;
        type Lowest = u32;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u32) -> (u64, u64) {
            (self as u64, r as u64)
        }
    }
    impl Cast<i64> for u64 {
        const CONTAINED: bool = false;
        fn cast(self) -> i64 {
            self as i64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (i64::MAX as u64))
        }
        fn default_saturate(self) -> i64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,u64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,u64>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min {
                0 as i64
            } else {
                9223372036854775807 as i64
            };
            if <u64 as Cast<i64>>::range(self).IsValid() {
                self as i64
            } else {
                saturated
            }
        }
    }
    impl Promote<i64> for u64 {
        type Max = u64;
        type Lowest = i64;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = false;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i64) -> (u64, u64) {
            (self as u64, r as u64)
        }
    }
    impl Cast<u64> for u64 {
        const CONTAINED: bool = true;
        fn cast(self) -> u64 {
            self as u64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> u64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,u64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,u64>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min {
                0 as u64
            } else {
                18446744073709551615 as u64
            };
            if <u64 as Cast<u64>>::range(self).IsValid() {
                self as u64
            } else {
                saturated
            }
        }
    }
    impl StrictCast<u64> for u64 {}
    impl Promote<u64> for u64 {
        type Max = u64;
        type Lowest = u64;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u64) -> (u64, u64) {
            (self as u64, r as u64)
        }
    }
    impl Cast<f32> for u64 {
        const CONTAINED: bool = true;
        fn cast(self) -> f32 {
            self as f32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f32> for u64 {}
    impl Promote<f32> for u64 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f32) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<f64> for u64 {
        const CONTAINED: bool = true;
        fn cast(self) -> f64 {
            self as f64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f64> for u64 {}
    impl Promote<f64> for u64 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f64) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<isize> for u64 {
        const CONTAINED: bool = false;
        fn cast(self) -> isize {
            self as isize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (isize::MAX as u64))
        }
        fn default_saturate(self) -> isize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,u64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,u64>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min {
                0 as isize
            } else {
                4294967295 as isize
            };
            if <u64 as Cast<isize>>::range(self).IsValid() {
                self as isize
            } else {
                saturated
            }
        }
    }
    impl Promote<isize> for u64 {
        type Max = u64;
        type Lowest = isize;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = false;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: isize) -> (u64, u64) {
            (self as u64, r as u64)
        }
    }
    impl Cast<usize> for u64 {
        const CONTAINED: bool = true;
        fn cast(self) -> usize {
            self as usize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> usize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,u64>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,u64>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min {
                0 as usize
            } else {
                18446744073709551615 as usize
            };
            if <u64 as Cast<usize>>::range(self).IsValid() {
                self as usize
            } else {
                saturated
            }
        }
    }
    impl StrictCast<usize> for u64 {}
    impl Promote<usize> for u64 {
        type Max = usize;
        type Lowest = usize;
        type Big = usize;
        type Fast = usize;
        type Sum = usize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: usize) -> (usize, usize) {
            (self as usize, r as usize)
        }
    }
    impl Cast<i8> for f32 {
        const CONTAINED: bool = false;
        fn cast(self) -> i8 {
            self as i8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= (i8::MIN as f32), self <= (i8::MAX as f32))
        }
    }
    impl Promote<i8> for f32 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i8) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<u8> for f32 {
        const CONTAINED: bool = false;
        fn cast(self) -> u8 {
            self as u8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self > -1.0, self <= (u8::MAX as f32))
        }
    }
    impl Promote<u8> for f32 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u8) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<i16> for f32 {
        const CONTAINED: bool = false;
        fn cast(self) -> i16 {
            self as i16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= (i16::MIN as f32), self <= (i16::MAX as f32))
        }
    }
    impl Promote<i16> for f32 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i16) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<u16> for f32 {
        const CONTAINED: bool = false;
        fn cast(self) -> u16 {
            self as u16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self > -1.0, self <= (u16::MAX as f32))
        }
    }
    impl Promote<u16> for f32 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u16) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<i32> for f32 {
        const CONTAINED: bool = false;
        fn cast(self) -> i32 {
            self as i32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(
                self >= (i32::MIN as f32),
                self <= ((i32::MAX & !(((1 as i32) << 7) - 1)) as f32),
            )
        }
    }
    impl Promote<i32> for f32 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i32) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<u32> for f32 {
        const CONTAINED: bool = false;
        fn cast(self) -> u32 {
            self as u32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(
                self > -1.0,
                self <= ((u32::MAX & !(((1 as u32) << 8) - 1)) as f32),
            )
        }
    }
    impl Promote<u32> for f32 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u32) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<i64> for f32 {
        const CONTAINED: bool = false;
        fn cast(self) -> i64 {
            self as i64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(
                self >= (i64::MIN as f32),
                self <= ((i64::MAX & !(((1 as i64) << 39) - 1)) as f32),
            )
        }
    }
    impl Promote<i64> for f32 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i64) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<u64> for f32 {
        const CONTAINED: bool = false;
        fn cast(self) -> u64 {
            self as u64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(
                self > -1.0,
                self <= ((u64::MAX & !(((1 as u64) << 40) - 1)) as f32),
            )
        }
    }
    impl Promote<u64> for f32 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u64) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<f32> for f32 {
        const CONTAINED: bool = true;
        fn cast(self) -> f32 {
            self as f32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f32> for f32 {}
    impl Promote<f32> for f32 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f32) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<f64> for f32 {
        const CONTAINED: bool = true;
        fn cast(self) -> f64 {
            self as f64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f64> for f32 {}
    impl Promote<f64> for f32 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f64) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<isize> for f32 {
        const CONTAINED: bool = false;
        fn cast(self) -> isize {
            self as isize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(
                self >= (isize::MIN as f32),
                self <= ((isize::MAX & !(((1 as isize) << 39) - 1)) as f32),
            )
        }
    }
    impl Promote<isize> for f32 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: isize) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<usize> for f32 {
        const CONTAINED: bool = false;
        fn cast(self) -> usize {
            self as usize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(
                self > -1.0,
                self <= ((usize::MAX & !(((1 as usize) << 40) - 1)) as f32),
            )
        }
    }
    impl Promote<usize> for f32 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: usize) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<i8> for f64 {
        const CONTAINED: bool = false;
        fn cast(self) -> i8 {
            self as i8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= (i8::MIN as f64), self <= (i8::MAX as f64))
        }
    }
    impl Promote<i8> for f64 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i8) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<u8> for f64 {
        const CONTAINED: bool = false;
        fn cast(self) -> u8 {
            self as u8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self > -1.0, self <= (u8::MAX as f64))
        }
    }
    impl Promote<u8> for f64 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u8) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<i16> for f64 {
        const CONTAINED: bool = false;
        fn cast(self) -> i16 {
            self as i16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= (i16::MIN as f64), self <= (i16::MAX as f64))
        }
    }
    impl Promote<i16> for f64 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i16) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<u16> for f64 {
        const CONTAINED: bool = false;
        fn cast(self) -> u16 {
            self as u16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self > -1.0, self <= (u16::MAX as f64))
        }
    }
    impl Promote<u16> for f64 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u16) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<i32> for f64 {
        const CONTAINED: bool = false;
        fn cast(self) -> i32 {
            self as i32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= (i32::MIN as f64), self <= (i32::MAX as f64))
        }
    }
    impl Promote<i32> for f64 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i32) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<u32> for f64 {
        const CONTAINED: bool = false;
        fn cast(self) -> u32 {
            self as u32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self > -1.0, self <= (u32::MAX as f64))
        }
    }
    impl Promote<u32> for f64 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u32) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<i64> for f64 {
        const CONTAINED: bool = false;
        fn cast(self) -> i64 {
            self as i64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(
                self >= (i64::MIN as f64),
                self <= ((i64::MAX & !(((1 as i64) << 10) - 1)) as f64),
            )
        }
    }
    impl Promote<i64> for f64 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i64) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<u64> for f64 {
        const CONTAINED: bool = false;
        fn cast(self) -> u64 {
            self as u64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(
                self > -1.0,
                self <= ((u64::MAX & !(((1 as u64) << 11) - 1)) as f64),
            )
        }
    }
    impl Promote<u64> for f64 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u64) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<f32> for f64 {
        const CONTAINED: bool = false;
        fn cast(self) -> f32 {
            self as f32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= (f32::MIN as f64), self <= (f32::MAX as f64))
        }
    }
    impl Promote<f32> for f64 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f32) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<f64> for f64 {
        const CONTAINED: bool = true;
        fn cast(self) -> f64 {
            self as f64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f64> for f64 {}
    impl Promote<f64> for f64 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f64) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<isize> for f64 {
        const CONTAINED: bool = false;
        fn cast(self) -> isize {
            self as isize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(
                self >= (isize::MIN as f64),
                self <= ((isize::MAX & !(((1 as isize) << 10) - 1)) as f64),
            )
        }
    }
    impl Promote<isize> for f64 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: isize) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<usize> for f64 {
        const CONTAINED: bool = false;
        fn cast(self) -> usize {
            self as usize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(
                self > -1.0,
                self <= ((usize::MAX & !(((1 as usize) << 11) - 1)) as f64),
            )
        }
    }
    impl Promote<usize> for f64 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: usize) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<i8> for isize {
        const CONTAINED: bool = false;
        fn cast(self) -> i8 {
            self as i8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= (i8::MIN as isize), self <= (i8::MAX as isize))
        }
        fn default_saturate(self) -> i8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,isize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,isize>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min { -128 as i8 } else { 127 as i8 };
            if <isize as Cast<i8>>::range(self).IsValid() {
                self as i8
            } else {
                saturated
            }
        }
    }
    impl Promote<i8> for isize {
        type Max = isize;
        type Lowest = isize;
        type Big = isize;
        type Fast = isize;
        type Sum = isize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i8) -> (isize, isize) {
            (self as isize, r as isize)
        }
    }
    impl Cast<u8> for isize {
        const CONTAINED: bool = false;
        fn cast(self) -> u8 {
            self as u8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, self <= (u8::MAX as isize))
        }
        fn default_saturate(self) -> u8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,isize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,isize>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min { 0 as u8 } else { 255 as u8 };
            if <isize as Cast<u8>>::range(self).IsValid() {
                self as u8
            } else {
                saturated
            }
        }
    }
    impl Promote<u8> for isize {
        type Max = isize;
        type Lowest = isize;
        type Big = isize;
        type Fast = isize;
        type Sum = isize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u8) -> (isize, isize) {
            (self as isize, r as isize)
        }
    }
    impl Cast<i16> for isize {
        const CONTAINED: bool = false;
        fn cast(self) -> i16 {
            self as i16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= (i16::MIN as isize), self <= (i16::MAX as isize))
        }
        fn default_saturate(self) -> i16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,isize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,isize>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min { -32768 as i16 } else { 32767 as i16 };
            if <isize as Cast<i16>>::range(self).IsValid() {
                self as i16
            } else {
                saturated
            }
        }
    }
    impl Promote<i16> for isize {
        type Max = isize;
        type Lowest = isize;
        type Big = isize;
        type Fast = isize;
        type Sum = isize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i16) -> (isize, isize) {
            (self as isize, r as isize)
        }
    }
    impl Cast<u16> for isize {
        const CONTAINED: bool = false;
        fn cast(self) -> u16 {
            self as u16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, self <= (u16::MAX as isize))
        }
        fn default_saturate(self) -> u16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,isize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,isize>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min { 0 as u16 } else { 65535 as u16 };
            if <isize as Cast<u16>>::range(self).IsValid() {
                self as u16
            } else {
                saturated
            }
        }
    }
    impl Promote<u16> for isize {
        type Max = isize;
        type Lowest = isize;
        type Big = isize;
        type Fast = isize;
        type Sum = isize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u16) -> (isize, isize) {
            (self as isize, r as isize)
        }
    }
    impl Cast<i32> for isize {
        const CONTAINED: bool = false;
        fn cast(self) -> i32 {
            self as i32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= (i32::MIN as isize), self <= (i32::MAX as isize))
        }
        fn default_saturate(self) -> i32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,isize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,isize>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min {
                -2147483648 as i32
            } else {
                2147483647 as i32
            };
            if <isize as Cast<i32>>::range(self).IsValid() {
                self as i32
            } else {
                saturated
            }
        }
    }
    impl Promote<i32> for isize {
        type Max = isize;
        type Lowest = isize;
        type Big = isize;
        type Fast = isize;
        type Sum = isize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i32) -> (isize, isize) {
            (self as isize, r as isize)
        }
    }
    impl Cast<u32> for isize {
        const CONTAINED: bool = false;
        fn cast(self) -> u32 {
            self as u32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, self <= (u32::MAX as isize))
        }
        fn default_saturate(self) -> u32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,isize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,isize>::Do(self)}
            }
            let is_min = false || (!false && self.negative());
            let saturated = if is_min { 0 as u32 } else { 4294967295 as u32 };
            if <isize as Cast<u32>>::range(self).IsValid() {
                self as u32
            } else {
                saturated
            }
        }
    }
    impl Promote<u32> for isize {
        type Max = isize;
        type Lowest = isize;
        type Big = isize;
        type Fast = isize;
        type Sum = isize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u32) -> (isize, isize) {
            (self as isize, r as isize)
        }
    }
    impl Cast<i64> for isize {
        const CONTAINED: bool = true;
        fn cast(self) -> i64 {
            self as i64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> i64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,isize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,isize>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min {
                -9223372036854775808 as i64
            } else {
                9223372036854775807 as i64
            };
            if <isize as Cast<i64>>::range(self).IsValid() {
                self as i64
            } else {
                saturated
            }
        }
    }
    impl StrictCast<i64> for isize {}
    impl Promote<i64> for isize {
        type Max = i64;
        type Lowest = i64;
        type Big = i64;
        type Fast = i64;
        type Sum = i64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i64) -> (i64, i64) {
            (self as i64, r as i64)
        }
    }
    impl Cast<u64> for isize {
        const CONTAINED: bool = false;
        fn cast(self) -> u64 {
            self as u64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, true)
        }
        fn default_saturate(self) -> u64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,isize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,isize>::Do(self)}
            }
            let is_min = true || (!false && self.negative());
            let saturated = if is_min {
                0 as u64
            } else {
                9223372036854775807 as u64
            };
            if <isize as Cast<u64>>::range(self).IsValid() {
                self as u64
            } else {
                saturated
            }
        }
    }
    impl Promote<u64> for isize {
        type Max = u64;
        type Lowest = isize;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = false;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u64) -> (u64, u64) {
            (self as u64, r as u64)
        }
    }
    impl Cast<f32> for isize {
        const CONTAINED: bool = true;
        fn cast(self) -> f32 {
            self as f32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f32> for isize {}
    impl Promote<f32> for isize {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f32) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<f64> for isize {
        const CONTAINED: bool = true;
        fn cast(self) -> f64 {
            self as f64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f64> for isize {}
    impl Promote<f64> for isize {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f64) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<isize> for isize {
        const CONTAINED: bool = true;
        fn cast(self) -> isize {
            self as isize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> isize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,isize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,isize>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min {
                -9223372036854775808 as isize
            } else {
                9223372036854775807 as isize
            };
            if <isize as Cast<isize>>::range(self).IsValid() {
                self as isize
            } else {
                saturated
            }
        }
    }
    impl StrictCast<isize> for isize {}
    impl Promote<isize> for isize {
        type Max = isize;
        type Lowest = isize;
        type Big = isize;
        type Fast = isize;
        type Sum = isize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: isize) -> (isize, isize) {
            (self as isize, r as isize)
        }
    }
    impl Cast<usize> for isize {
        const CONTAINED: bool = false;
        fn cast(self) -> usize {
            self as usize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, true)
        }
        fn default_saturate(self) -> usize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,isize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,isize>::Do(self)}
            }
            let is_min = true || (!false && self.negative());
            let saturated = if is_min {
                0 as usize
            } else {
                9223372036854775807 as usize
            };
            if <isize as Cast<usize>>::range(self).IsValid() {
                self as usize
            } else {
                saturated
            }
        }
    }
    impl Promote<usize> for isize {
        type Max = usize;
        type Lowest = isize;
        type Big = usize;
        type Fast = usize;
        type Sum = usize;
        const BIG_CONTAINED: bool = false;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: usize) -> (usize, usize) {
            (self as usize, r as usize)
        }
    }
    impl Cast<i8> for usize {
        const CONTAINED: bool = false;
        fn cast(self) -> i8 {
            self as i8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (i8::MAX as usize))
        }
        fn default_saturate(self) -> i8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,usize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i8,usize>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as i8 } else { 127 as i8 };
            if <usize as Cast<i8>>::range(self).IsValid() {
                self as i8
            } else {
                saturated
            }
        }
    }
    impl Promote<i8> for usize {
        type Max = usize;
        type Lowest = i8;
        type Big = usize;
        type Fast = usize;
        type Sum = usize;
        const BIG_CONTAINED: bool = false;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i8) -> (usize, usize) {
            (self as usize, r as usize)
        }
    }
    impl Cast<u8> for usize {
        const CONTAINED: bool = false;
        fn cast(self) -> u8 {
            self as u8
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (u8::MAX as usize))
        }
        fn default_saturate(self) -> u8 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,usize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u8,usize>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as u8 } else { 255 as u8 };
            if <usize as Cast<u8>>::range(self).IsValid() {
                self as u8
            } else {
                saturated
            }
        }
    }
    impl Promote<u8> for usize {
        type Max = usize;
        type Lowest = u8;
        type Big = usize;
        type Fast = usize;
        type Sum = usize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u8) -> (usize, usize) {
            (self as usize, r as usize)
        }
    }
    impl Cast<i16> for usize {
        const CONTAINED: bool = false;
        fn cast(self) -> i16 {
            self as i16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (i16::MAX as usize))
        }
        fn default_saturate(self) -> i16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,usize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i16,usize>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as i16 } else { 32767 as i16 };
            if <usize as Cast<i16>>::range(self).IsValid() {
                self as i16
            } else {
                saturated
            }
        }
    }
    impl Promote<i16> for usize {
        type Max = usize;
        type Lowest = i16;
        type Big = usize;
        type Fast = usize;
        type Sum = usize;
        const BIG_CONTAINED: bool = false;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i16) -> (usize, usize) {
            (self as usize, r as usize)
        }
    }
    impl Cast<u16> for usize {
        const CONTAINED: bool = false;
        fn cast(self) -> u16 {
            self as u16
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (u16::MAX as usize))
        }
        fn default_saturate(self) -> u16 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,usize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u16,usize>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as u16 } else { 65535 as u16 };
            if <usize as Cast<u16>>::range(self).IsValid() {
                self as u16
            } else {
                saturated
            }
        }
    }
    impl Promote<u16> for usize {
        type Max = usize;
        type Lowest = u16;
        type Big = usize;
        type Fast = usize;
        type Sum = usize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u16) -> (usize, usize) {
            (self as usize, r as usize)
        }
    }
    impl Cast<i32> for usize {
        const CONTAINED: bool = false;
        fn cast(self) -> i32 {
            self as i32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (i32::MAX as usize))
        }
        fn default_saturate(self) -> i32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,usize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i32,usize>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as i32 } else { 2147483647 as i32 };
            if <usize as Cast<i32>>::range(self).IsValid() {
                self as i32
            } else {
                saturated
            }
        }
    }
    impl Promote<i32> for usize {
        type Max = usize;
        type Lowest = i32;
        type Big = usize;
        type Fast = usize;
        type Sum = usize;
        const BIG_CONTAINED: bool = false;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i32) -> (usize, usize) {
            (self as usize, r as usize)
        }
    }
    impl Cast<u32> for usize {
        const CONTAINED: bool = false;
        fn cast(self) -> u32 {
            self as u32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (u32::MAX as usize))
        }
        fn default_saturate(self) -> u32 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,usize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u32,usize>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min { 0 as u32 } else { 4294967295 as u32 };
            if <usize as Cast<u32>>::range(self).IsValid() {
                self as u32
            } else {
                saturated
            }
        }
    }
    impl Promote<u32> for usize {
        type Max = usize;
        type Lowest = u32;
        type Big = usize;
        type Fast = usize;
        type Sum = usize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u32) -> (usize, usize) {
            (self as usize, r as usize)
        }
    }
    impl Cast<i64> for usize {
        const CONTAINED: bool = false;
        fn cast(self) -> i64 {
            self as i64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (i64::MAX as usize))
        }
        fn default_saturate(self) -> i64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,usize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<i64,usize>::Do(self)}
            }
            let is_min = false || (!true && self.negative());
            let saturated = if is_min {
                0 as i64
            } else {
                9223372036854775807 as i64
            };
            if <usize as Cast<i64>>::range(self).IsValid() {
                self as i64
            } else {
                saturated
            }
        }
    }
    impl Promote<i64> for usize {
        type Max = usize;
        type Lowest = i64;
        type Big = usize;
        type Fast = usize;
        type Sum = usize;
        const BIG_CONTAINED: bool = false;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i64) -> (usize, usize) {
            (self as usize, r as usize)
        }
    }
    impl Cast<u64> for usize {
        const CONTAINED: bool = true;
        fn cast(self) -> u64 {
            self as u64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> u64 {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,usize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<u64,usize>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min {
                0 as u64
            } else {
                18446744073709551615 as u64
            };
            if <usize as Cast<u64>>::range(self).IsValid() {
                self as u64
            } else {
                saturated
            }
        }
    }
    impl StrictCast<u64> for usize {}
    impl Promote<u64> for usize {
        type Max = u64;
        type Lowest = u64;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u64) -> (u64, u64) {
            (self as u64, r as u64)
        }
    }
    impl Cast<f32> for usize {
        const CONTAINED: bool = true;
        fn cast(self) -> f32 {
            self as f32
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f32> for usize {}
    impl Promote<f32> for usize {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f32) -> (f32, f32) {
            (self as f32, r as f32)
        }
    }
    impl Cast<f64> for usize {
        const CONTAINED: bool = true;
        fn cast(self) -> f64 {
            self as f64
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f64> for usize {}
    impl Promote<f64> for usize {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f64) -> (f64, f64) {
            (self as f64, r as f64)
        }
    }
    impl Cast<isize> for usize {
        const CONTAINED: bool = false;
        fn cast(self) -> isize {
            self as isize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= (isize::MAX as usize))
        }
        fn default_saturate(self) -> isize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,usize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<isize,usize>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min {
                0 as isize
            } else {
                4294967295 as isize
            };
            if <usize as Cast<isize>>::range(self).IsValid() {
                self as isize
            } else {
                saturated
            }
        }
    }
    impl Promote<isize> for usize {
        type Max = usize;
        type Lowest = isize;
        type Big = usize;
        type Fast = usize;
        type Sum = usize;
        const BIG_CONTAINED: bool = false;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: isize) -> (usize, usize) {
            (self as usize, r as usize)
        }
    }
    impl Cast<usize> for usize {
        const CONTAINED: bool = true;
        fn cast(self) -> usize {
            self as usize
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
        fn default_saturate(self) -> usize {
            #[cfg(all(base_numeric_armel, target_arch = "arm"))]
            {
                if super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,usize>::is_supported{return super::super::safe_conversions_arm_impl::SaturateFastAsmOp::<usize,usize>::Do(self)}
            }
            let is_min = true || (!true && self.negative());
            let saturated = if is_min {
                0 as usize
            } else {
                18446744073709551615 as usize
            };
            if <usize as Cast<usize>>::range(self).IsValid() {
                self as usize
            } else {
                saturated
            }
        }
    }
    impl StrictCast<usize> for usize {}
    impl Promote<usize> for usize {
        type Max = usize;
        type Lowest = usize;
        type Big = usize;
        type Fast = usize;
        type Sum = usize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: usize) -> (usize, usize) {
            (self as usize, r as usize)
        }
    }
    // Native make_signed/make_unsigned domains exclude bool. No f64 coercion of integer comparisons.
    pub trait Integer: Numeric {
        type Signed: Numeric;
        type Unsigned: Numeric;
        fn ConditionalNegate(self, negative: bool) -> Self::Signed;
        fn unsigned_abs(self) -> Self::Unsigned;
        fn as_signed(self) -> Self::Signed;
        fn as_unsigned(self) -> Self::Unsigned;
    }
    impl Integer for i8 {
        type Signed = i8;
        type Unsigned = u8;
        fn ConditionalNegate(self, n: bool) -> i8 {
            ((self as u8 ^ (0 as u8).wrapping_sub(n as u8)).wrapping_add(n as u8)) as i8
        }
        fn unsigned_abs(self) -> u8 {
            if self.negative() {
                (0 as u8).wrapping_sub(self as u8)
            } else {
                self as u8
            }
        }
        fn as_signed(self) -> i8 {
            self as i8
        }
        fn as_unsigned(self) -> u8 {
            self as u8
        }
    }
    impl Integer for u8 {
        type Signed = i8;
        type Unsigned = u8;
        fn ConditionalNegate(self, n: bool) -> i8 {
            ((self as u8 ^ (0 as u8).wrapping_sub(n as u8)).wrapping_add(n as u8)) as i8
        }
        fn unsigned_abs(self) -> u8 {
            if self.negative() {
                (0 as u8).wrapping_sub(self as u8)
            } else {
                self as u8
            }
        }
        fn as_signed(self) -> i8 {
            self as i8
        }
        fn as_unsigned(self) -> u8 {
            self as u8
        }
    }
    impl Integer for i16 {
        type Signed = i16;
        type Unsigned = u16;
        fn ConditionalNegate(self, n: bool) -> i16 {
            ((self as u16 ^ (0 as u16).wrapping_sub(n as u16)).wrapping_add(n as u16)) as i16
        }
        fn unsigned_abs(self) -> u16 {
            if self.negative() {
                (0 as u16).wrapping_sub(self as u16)
            } else {
                self as u16
            }
        }
        fn as_signed(self) -> i16 {
            self as i16
        }
        fn as_unsigned(self) -> u16 {
            self as u16
        }
    }
    impl Integer for u16 {
        type Signed = i16;
        type Unsigned = u16;
        fn ConditionalNegate(self, n: bool) -> i16 {
            ((self as u16 ^ (0 as u16).wrapping_sub(n as u16)).wrapping_add(n as u16)) as i16
        }
        fn unsigned_abs(self) -> u16 {
            if self.negative() {
                (0 as u16).wrapping_sub(self as u16)
            } else {
                self as u16
            }
        }
        fn as_signed(self) -> i16 {
            self as i16
        }
        fn as_unsigned(self) -> u16 {
            self as u16
        }
    }
    impl Integer for i32 {
        type Signed = i32;
        type Unsigned = u32;
        fn ConditionalNegate(self, n: bool) -> i32 {
            ((self as u32 ^ (0 as u32).wrapping_sub(n as u32)).wrapping_add(n as u32)) as i32
        }
        fn unsigned_abs(self) -> u32 {
            if self.negative() {
                (0 as u32).wrapping_sub(self as u32)
            } else {
                self as u32
            }
        }
        fn as_signed(self) -> i32 {
            self as i32
        }
        fn as_unsigned(self) -> u32 {
            self as u32
        }
    }
    impl Integer for u32 {
        type Signed = i32;
        type Unsigned = u32;
        fn ConditionalNegate(self, n: bool) -> i32 {
            ((self as u32 ^ (0 as u32).wrapping_sub(n as u32)).wrapping_add(n as u32)) as i32
        }
        fn unsigned_abs(self) -> u32 {
            if self.negative() {
                (0 as u32).wrapping_sub(self as u32)
            } else {
                self as u32
            }
        }
        fn as_signed(self) -> i32 {
            self as i32
        }
        fn as_unsigned(self) -> u32 {
            self as u32
        }
    }
    impl Integer for i64 {
        type Signed = i64;
        type Unsigned = u64;
        fn ConditionalNegate(self, n: bool) -> i64 {
            ((self as u64 ^ (0 as u64).wrapping_sub(n as u64)).wrapping_add(n as u64)) as i64
        }
        fn unsigned_abs(self) -> u64 {
            if self.negative() {
                (0 as u64).wrapping_sub(self as u64)
            } else {
                self as u64
            }
        }
        fn as_signed(self) -> i64 {
            self as i64
        }
        fn as_unsigned(self) -> u64 {
            self as u64
        }
    }
    impl Integer for u64 {
        type Signed = i64;
        type Unsigned = u64;
        fn ConditionalNegate(self, n: bool) -> i64 {
            ((self as u64 ^ (0 as u64).wrapping_sub(n as u64)).wrapping_add(n as u64)) as i64
        }
        fn unsigned_abs(self) -> u64 {
            if self.negative() {
                (0 as u64).wrapping_sub(self as u64)
            } else {
                self as u64
            }
        }
        fn as_signed(self) -> i64 {
            self as i64
        }
        fn as_unsigned(self) -> u64 {
            self as u64
        }
    }
    impl Integer for isize {
        type Signed = i64;
        type Unsigned = u64;
        fn ConditionalNegate(self, n: bool) -> i64 {
            ((self as u64 ^ (0 as u64).wrapping_sub(n as u64)).wrapping_add(n as u64)) as i64
        }
        fn unsigned_abs(self) -> u64 {
            if self.negative() {
                (0 as u64).wrapping_sub(self as u64)
            } else {
                self as u64
            }
        }
        fn as_signed(self) -> i64 {
            self as i64
        }
        fn as_unsigned(self) -> u64 {
            self as u64
        }
    }
    impl Integer for usize {
        type Signed = i64;
        type Unsigned = u64;
        fn ConditionalNegate(self, n: bool) -> i64 {
            ((self as u64 ^ (0 as u64).wrapping_sub(n as u64)).wrapping_add(n as u64)) as i64
        }
        fn unsigned_abs(self) -> u64 {
            if self.negative() {
                (0 as u64).wrapping_sub(self as u64)
            } else {
                self as u64
            }
        }
        fn as_signed(self) -> i64 {
            self as i64
        }
        fn as_unsigned(self) -> u64 {
            self as u64
        }
    }
    pub fn ConditionalNegate<T: Integer>(v: T, n: bool) -> T::Signed {
        v.ConditionalNegate(n)
    }
    pub fn SafeUnsignedAbs<T: Integer>(v: T) -> T::Unsigned {
        v.unsigned_abs()
    }
    pub fn as_signed<T: Integer>(v: T) -> T::Signed {
        v.as_signed()
    }
    pub fn as_unsigned<T: Integer>(v: T) -> T::Unsigned {
        v.as_unsigned()
    }
}
pub use domain::*;

pub trait SafeComparison<R: Numeric>: Numeric {
    fn safe_lt(self, r: R) -> bool;
    fn safe_le(self, r: R) -> bool;
    fn safe_gt(self, r: R) -> bool;
    fn safe_ge(self, r: R) -> bool;
    fn safe_eq(self, r: R) -> bool;
    fn safe_ne(self, r: R) -> bool;
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i8> for i8 {
    fn safe_lt(self, r: i8) -> bool {
        IsLess(self as i8, r as i8)
    }
    fn safe_le(self, r: i8) -> bool {
        IsLessOrEqual(self as i8, r as i8)
    }
    fn safe_gt(self, r: i8) -> bool {
        IsGreater(self as i8, r as i8)
    }
    fn safe_ge(self, r: i8) -> bool {
        IsGreaterOrEqual(self as i8, r as i8)
    }
    fn safe_eq(self, r: i8) -> bool {
        IsEqual(self as i8, r as i8)
    }
    fn safe_ne(self, r: i8) -> bool {
        IsNotEqual(self as i8, r as i8)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u8> for i8 {
    fn safe_lt(self, r: u8) -> bool {
        IsLess(self as i16, r as i16)
    }
    fn safe_le(self, r: u8) -> bool {
        IsLessOrEqual(self as i16, r as i16)
    }
    fn safe_gt(self, r: u8) -> bool {
        IsGreater(self as i16, r as i16)
    }
    fn safe_ge(self, r: u8) -> bool {
        IsGreaterOrEqual(self as i16, r as i16)
    }
    fn safe_eq(self, r: u8) -> bool {
        IsEqual(self as i16, r as i16)
    }
    fn safe_ne(self, r: u8) -> bool {
        IsNotEqual(self as i16, r as i16)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i16> for i8 {
    fn safe_lt(self, r: i16) -> bool {
        IsLess(self as i16, r as i16)
    }
    fn safe_le(self, r: i16) -> bool {
        IsLessOrEqual(self as i16, r as i16)
    }
    fn safe_gt(self, r: i16) -> bool {
        IsGreater(self as i16, r as i16)
    }
    fn safe_ge(self, r: i16) -> bool {
        IsGreaterOrEqual(self as i16, r as i16)
    }
    fn safe_eq(self, r: i16) -> bool {
        IsEqual(self as i16, r as i16)
    }
    fn safe_ne(self, r: i16) -> bool {
        IsNotEqual(self as i16, r as i16)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u16> for i8 {
    fn safe_lt(self, r: u16) -> bool {
        IsLess(self as i32, r as i32)
    }
    fn safe_le(self, r: u16) -> bool {
        IsLessOrEqual(self as i32, r as i32)
    }
    fn safe_gt(self, r: u16) -> bool {
        IsGreater(self as i32, r as i32)
    }
    fn safe_ge(self, r: u16) -> bool {
        IsGreaterOrEqual(self as i32, r as i32)
    }
    fn safe_eq(self, r: u16) -> bool {
        IsEqual(self as i32, r as i32)
    }
    fn safe_ne(self, r: u16) -> bool {
        IsNotEqual(self as i32, r as i32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i32> for i8 {
    fn safe_lt(self, r: i32) -> bool {
        IsLess(self as i32, r as i32)
    }
    fn safe_le(self, r: i32) -> bool {
        IsLessOrEqual(self as i32, r as i32)
    }
    fn safe_gt(self, r: i32) -> bool {
        IsGreater(self as i32, r as i32)
    }
    fn safe_ge(self, r: i32) -> bool {
        IsGreaterOrEqual(self as i32, r as i32)
    }
    fn safe_eq(self, r: i32) -> bool {
        IsEqual(self as i32, r as i32)
    }
    fn safe_ne(self, r: i32) -> bool {
        IsNotEqual(self as i32, r as i32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u32> for i8 {
    fn safe_lt(self, r: u32) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: u32) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: u32) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: u32) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: u32) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: u32) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i64> for i8 {
    fn safe_lt(self, r: i64) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: i64) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: i64) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: i64) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: i64) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: i64) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u64> for i8 {
    fn safe_lt(self, r: u64) -> bool {
        IsLess(self, r)
    }
    fn safe_le(self, r: u64) -> bool {
        IsLessOrEqual(self, r)
    }
    fn safe_gt(self, r: u64) -> bool {
        IsGreater(self, r)
    }
    fn safe_ge(self, r: u64) -> bool {
        IsGreaterOrEqual(self, r)
    }
    fn safe_eq(self, r: u64) -> bool {
        IsEqual(self, r)
    }
    fn safe_ne(self, r: u64) -> bool {
        IsNotEqual(self, r)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<f32> for i8 {
    fn safe_lt(self, r: f32) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: f32) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: f32) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: f32) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: f32) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: f32) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<f64> for i8 {
    fn safe_lt(self, r: f64) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: f64) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: f64) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: f64) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: f64) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: f64) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<isize> for i8 {
    fn safe_lt(self, r: isize) -> bool {
        IsLess(self as isize, r as isize)
    }
    fn safe_le(self, r: isize) -> bool {
        IsLessOrEqual(self as isize, r as isize)
    }
    fn safe_gt(self, r: isize) -> bool {
        IsGreater(self as isize, r as isize)
    }
    fn safe_ge(self, r: isize) -> bool {
        IsGreaterOrEqual(self as isize, r as isize)
    }
    fn safe_eq(self, r: isize) -> bool {
        IsEqual(self as isize, r as isize)
    }
    fn safe_ne(self, r: isize) -> bool {
        IsNotEqual(self as isize, r as isize)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<usize> for i8 {
    fn safe_lt(self, r: usize) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: usize) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: usize) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: usize) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: usize) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: usize) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i8> for u8 {
    fn safe_lt(self, r: i8) -> bool {
        IsLess(self as i16, r as i16)
    }
    fn safe_le(self, r: i8) -> bool {
        IsLessOrEqual(self as i16, r as i16)
    }
    fn safe_gt(self, r: i8) -> bool {
        IsGreater(self as i16, r as i16)
    }
    fn safe_ge(self, r: i8) -> bool {
        IsGreaterOrEqual(self as i16, r as i16)
    }
    fn safe_eq(self, r: i8) -> bool {
        IsEqual(self as i16, r as i16)
    }
    fn safe_ne(self, r: i8) -> bool {
        IsNotEqual(self as i16, r as i16)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u8> for u8 {
    fn safe_lt(self, r: u8) -> bool {
        IsLess(self as u8, r as u8)
    }
    fn safe_le(self, r: u8) -> bool {
        IsLessOrEqual(self as u8, r as u8)
    }
    fn safe_gt(self, r: u8) -> bool {
        IsGreater(self as u8, r as u8)
    }
    fn safe_ge(self, r: u8) -> bool {
        IsGreaterOrEqual(self as u8, r as u8)
    }
    fn safe_eq(self, r: u8) -> bool {
        IsEqual(self as u8, r as u8)
    }
    fn safe_ne(self, r: u8) -> bool {
        IsNotEqual(self as u8, r as u8)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i16> for u8 {
    fn safe_lt(self, r: i16) -> bool {
        IsLess(self as i16, r as i16)
    }
    fn safe_le(self, r: i16) -> bool {
        IsLessOrEqual(self as i16, r as i16)
    }
    fn safe_gt(self, r: i16) -> bool {
        IsGreater(self as i16, r as i16)
    }
    fn safe_ge(self, r: i16) -> bool {
        IsGreaterOrEqual(self as i16, r as i16)
    }
    fn safe_eq(self, r: i16) -> bool {
        IsEqual(self as i16, r as i16)
    }
    fn safe_ne(self, r: i16) -> bool {
        IsNotEqual(self as i16, r as i16)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u16> for u8 {
    fn safe_lt(self, r: u16) -> bool {
        IsLess(self as u16, r as u16)
    }
    fn safe_le(self, r: u16) -> bool {
        IsLessOrEqual(self as u16, r as u16)
    }
    fn safe_gt(self, r: u16) -> bool {
        IsGreater(self as u16, r as u16)
    }
    fn safe_ge(self, r: u16) -> bool {
        IsGreaterOrEqual(self as u16, r as u16)
    }
    fn safe_eq(self, r: u16) -> bool {
        IsEqual(self as u16, r as u16)
    }
    fn safe_ne(self, r: u16) -> bool {
        IsNotEqual(self as u16, r as u16)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i32> for u8 {
    fn safe_lt(self, r: i32) -> bool {
        IsLess(self as i32, r as i32)
    }
    fn safe_le(self, r: i32) -> bool {
        IsLessOrEqual(self as i32, r as i32)
    }
    fn safe_gt(self, r: i32) -> bool {
        IsGreater(self as i32, r as i32)
    }
    fn safe_ge(self, r: i32) -> bool {
        IsGreaterOrEqual(self as i32, r as i32)
    }
    fn safe_eq(self, r: i32) -> bool {
        IsEqual(self as i32, r as i32)
    }
    fn safe_ne(self, r: i32) -> bool {
        IsNotEqual(self as i32, r as i32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u32> for u8 {
    fn safe_lt(self, r: u32) -> bool {
        IsLess(self as u32, r as u32)
    }
    fn safe_le(self, r: u32) -> bool {
        IsLessOrEqual(self as u32, r as u32)
    }
    fn safe_gt(self, r: u32) -> bool {
        IsGreater(self as u32, r as u32)
    }
    fn safe_ge(self, r: u32) -> bool {
        IsGreaterOrEqual(self as u32, r as u32)
    }
    fn safe_eq(self, r: u32) -> bool {
        IsEqual(self as u32, r as u32)
    }
    fn safe_ne(self, r: u32) -> bool {
        IsNotEqual(self as u32, r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i64> for u8 {
    fn safe_lt(self, r: i64) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: i64) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: i64) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: i64) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: i64) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: i64) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u64> for u8 {
    fn safe_lt(self, r: u64) -> bool {
        IsLess(self as u64, r as u64)
    }
    fn safe_le(self, r: u64) -> bool {
        IsLessOrEqual(self as u64, r as u64)
    }
    fn safe_gt(self, r: u64) -> bool {
        IsGreater(self as u64, r as u64)
    }
    fn safe_ge(self, r: u64) -> bool {
        IsGreaterOrEqual(self as u64, r as u64)
    }
    fn safe_eq(self, r: u64) -> bool {
        IsEqual(self as u64, r as u64)
    }
    fn safe_ne(self, r: u64) -> bool {
        IsNotEqual(self as u64, r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<f32> for u8 {
    fn safe_lt(self, r: f32) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: f32) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: f32) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: f32) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: f32) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: f32) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<f64> for u8 {
    fn safe_lt(self, r: f64) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: f64) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: f64) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: f64) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: f64) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: f64) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<isize> for u8 {
    fn safe_lt(self, r: isize) -> bool {
        IsLess(self as isize, r as isize)
    }
    fn safe_le(self, r: isize) -> bool {
        IsLessOrEqual(self as isize, r as isize)
    }
    fn safe_gt(self, r: isize) -> bool {
        IsGreater(self as isize, r as isize)
    }
    fn safe_ge(self, r: isize) -> bool {
        IsGreaterOrEqual(self as isize, r as isize)
    }
    fn safe_eq(self, r: isize) -> bool {
        IsEqual(self as isize, r as isize)
    }
    fn safe_ne(self, r: isize) -> bool {
        IsNotEqual(self as isize, r as isize)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<usize> for u8 {
    fn safe_lt(self, r: usize) -> bool {
        IsLess(self as usize, r as usize)
    }
    fn safe_le(self, r: usize) -> bool {
        IsLessOrEqual(self as usize, r as usize)
    }
    fn safe_gt(self, r: usize) -> bool {
        IsGreater(self as usize, r as usize)
    }
    fn safe_ge(self, r: usize) -> bool {
        IsGreaterOrEqual(self as usize, r as usize)
    }
    fn safe_eq(self, r: usize) -> bool {
        IsEqual(self as usize, r as usize)
    }
    fn safe_ne(self, r: usize) -> bool {
        IsNotEqual(self as usize, r as usize)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i8> for i16 {
    fn safe_lt(self, r: i8) -> bool {
        IsLess(self as i16, r as i16)
    }
    fn safe_le(self, r: i8) -> bool {
        IsLessOrEqual(self as i16, r as i16)
    }
    fn safe_gt(self, r: i8) -> bool {
        IsGreater(self as i16, r as i16)
    }
    fn safe_ge(self, r: i8) -> bool {
        IsGreaterOrEqual(self as i16, r as i16)
    }
    fn safe_eq(self, r: i8) -> bool {
        IsEqual(self as i16, r as i16)
    }
    fn safe_ne(self, r: i8) -> bool {
        IsNotEqual(self as i16, r as i16)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u8> for i16 {
    fn safe_lt(self, r: u8) -> bool {
        IsLess(self as i16, r as i16)
    }
    fn safe_le(self, r: u8) -> bool {
        IsLessOrEqual(self as i16, r as i16)
    }
    fn safe_gt(self, r: u8) -> bool {
        IsGreater(self as i16, r as i16)
    }
    fn safe_ge(self, r: u8) -> bool {
        IsGreaterOrEqual(self as i16, r as i16)
    }
    fn safe_eq(self, r: u8) -> bool {
        IsEqual(self as i16, r as i16)
    }
    fn safe_ne(self, r: u8) -> bool {
        IsNotEqual(self as i16, r as i16)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i16> for i16 {
    fn safe_lt(self, r: i16) -> bool {
        IsLess(self as i16, r as i16)
    }
    fn safe_le(self, r: i16) -> bool {
        IsLessOrEqual(self as i16, r as i16)
    }
    fn safe_gt(self, r: i16) -> bool {
        IsGreater(self as i16, r as i16)
    }
    fn safe_ge(self, r: i16) -> bool {
        IsGreaterOrEqual(self as i16, r as i16)
    }
    fn safe_eq(self, r: i16) -> bool {
        IsEqual(self as i16, r as i16)
    }
    fn safe_ne(self, r: i16) -> bool {
        IsNotEqual(self as i16, r as i16)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u16> for i16 {
    fn safe_lt(self, r: u16) -> bool {
        IsLess(self as i32, r as i32)
    }
    fn safe_le(self, r: u16) -> bool {
        IsLessOrEqual(self as i32, r as i32)
    }
    fn safe_gt(self, r: u16) -> bool {
        IsGreater(self as i32, r as i32)
    }
    fn safe_ge(self, r: u16) -> bool {
        IsGreaterOrEqual(self as i32, r as i32)
    }
    fn safe_eq(self, r: u16) -> bool {
        IsEqual(self as i32, r as i32)
    }
    fn safe_ne(self, r: u16) -> bool {
        IsNotEqual(self as i32, r as i32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i32> for i16 {
    fn safe_lt(self, r: i32) -> bool {
        IsLess(self as i32, r as i32)
    }
    fn safe_le(self, r: i32) -> bool {
        IsLessOrEqual(self as i32, r as i32)
    }
    fn safe_gt(self, r: i32) -> bool {
        IsGreater(self as i32, r as i32)
    }
    fn safe_ge(self, r: i32) -> bool {
        IsGreaterOrEqual(self as i32, r as i32)
    }
    fn safe_eq(self, r: i32) -> bool {
        IsEqual(self as i32, r as i32)
    }
    fn safe_ne(self, r: i32) -> bool {
        IsNotEqual(self as i32, r as i32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u32> for i16 {
    fn safe_lt(self, r: u32) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: u32) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: u32) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: u32) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: u32) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: u32) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i64> for i16 {
    fn safe_lt(self, r: i64) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: i64) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: i64) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: i64) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: i64) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: i64) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u64> for i16 {
    fn safe_lt(self, r: u64) -> bool {
        IsLess(self, r)
    }
    fn safe_le(self, r: u64) -> bool {
        IsLessOrEqual(self, r)
    }
    fn safe_gt(self, r: u64) -> bool {
        IsGreater(self, r)
    }
    fn safe_ge(self, r: u64) -> bool {
        IsGreaterOrEqual(self, r)
    }
    fn safe_eq(self, r: u64) -> bool {
        IsEqual(self, r)
    }
    fn safe_ne(self, r: u64) -> bool {
        IsNotEqual(self, r)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<f32> for i16 {
    fn safe_lt(self, r: f32) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: f32) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: f32) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: f32) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: f32) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: f32) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<f64> for i16 {
    fn safe_lt(self, r: f64) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: f64) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: f64) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: f64) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: f64) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: f64) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<isize> for i16 {
    fn safe_lt(self, r: isize) -> bool {
        IsLess(self as isize, r as isize)
    }
    fn safe_le(self, r: isize) -> bool {
        IsLessOrEqual(self as isize, r as isize)
    }
    fn safe_gt(self, r: isize) -> bool {
        IsGreater(self as isize, r as isize)
    }
    fn safe_ge(self, r: isize) -> bool {
        IsGreaterOrEqual(self as isize, r as isize)
    }
    fn safe_eq(self, r: isize) -> bool {
        IsEqual(self as isize, r as isize)
    }
    fn safe_ne(self, r: isize) -> bool {
        IsNotEqual(self as isize, r as isize)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<usize> for i16 {
    fn safe_lt(self, r: usize) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: usize) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: usize) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: usize) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: usize) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: usize) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i8> for u16 {
    fn safe_lt(self, r: i8) -> bool {
        IsLess(self as i32, r as i32)
    }
    fn safe_le(self, r: i8) -> bool {
        IsLessOrEqual(self as i32, r as i32)
    }
    fn safe_gt(self, r: i8) -> bool {
        IsGreater(self as i32, r as i32)
    }
    fn safe_ge(self, r: i8) -> bool {
        IsGreaterOrEqual(self as i32, r as i32)
    }
    fn safe_eq(self, r: i8) -> bool {
        IsEqual(self as i32, r as i32)
    }
    fn safe_ne(self, r: i8) -> bool {
        IsNotEqual(self as i32, r as i32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u8> for u16 {
    fn safe_lt(self, r: u8) -> bool {
        IsLess(self as u16, r as u16)
    }
    fn safe_le(self, r: u8) -> bool {
        IsLessOrEqual(self as u16, r as u16)
    }
    fn safe_gt(self, r: u8) -> bool {
        IsGreater(self as u16, r as u16)
    }
    fn safe_ge(self, r: u8) -> bool {
        IsGreaterOrEqual(self as u16, r as u16)
    }
    fn safe_eq(self, r: u8) -> bool {
        IsEqual(self as u16, r as u16)
    }
    fn safe_ne(self, r: u8) -> bool {
        IsNotEqual(self as u16, r as u16)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i16> for u16 {
    fn safe_lt(self, r: i16) -> bool {
        IsLess(self as i32, r as i32)
    }
    fn safe_le(self, r: i16) -> bool {
        IsLessOrEqual(self as i32, r as i32)
    }
    fn safe_gt(self, r: i16) -> bool {
        IsGreater(self as i32, r as i32)
    }
    fn safe_ge(self, r: i16) -> bool {
        IsGreaterOrEqual(self as i32, r as i32)
    }
    fn safe_eq(self, r: i16) -> bool {
        IsEqual(self as i32, r as i32)
    }
    fn safe_ne(self, r: i16) -> bool {
        IsNotEqual(self as i32, r as i32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u16> for u16 {
    fn safe_lt(self, r: u16) -> bool {
        IsLess(self as u16, r as u16)
    }
    fn safe_le(self, r: u16) -> bool {
        IsLessOrEqual(self as u16, r as u16)
    }
    fn safe_gt(self, r: u16) -> bool {
        IsGreater(self as u16, r as u16)
    }
    fn safe_ge(self, r: u16) -> bool {
        IsGreaterOrEqual(self as u16, r as u16)
    }
    fn safe_eq(self, r: u16) -> bool {
        IsEqual(self as u16, r as u16)
    }
    fn safe_ne(self, r: u16) -> bool {
        IsNotEqual(self as u16, r as u16)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i32> for u16 {
    fn safe_lt(self, r: i32) -> bool {
        IsLess(self as i32, r as i32)
    }
    fn safe_le(self, r: i32) -> bool {
        IsLessOrEqual(self as i32, r as i32)
    }
    fn safe_gt(self, r: i32) -> bool {
        IsGreater(self as i32, r as i32)
    }
    fn safe_ge(self, r: i32) -> bool {
        IsGreaterOrEqual(self as i32, r as i32)
    }
    fn safe_eq(self, r: i32) -> bool {
        IsEqual(self as i32, r as i32)
    }
    fn safe_ne(self, r: i32) -> bool {
        IsNotEqual(self as i32, r as i32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u32> for u16 {
    fn safe_lt(self, r: u32) -> bool {
        IsLess(self as u32, r as u32)
    }
    fn safe_le(self, r: u32) -> bool {
        IsLessOrEqual(self as u32, r as u32)
    }
    fn safe_gt(self, r: u32) -> bool {
        IsGreater(self as u32, r as u32)
    }
    fn safe_ge(self, r: u32) -> bool {
        IsGreaterOrEqual(self as u32, r as u32)
    }
    fn safe_eq(self, r: u32) -> bool {
        IsEqual(self as u32, r as u32)
    }
    fn safe_ne(self, r: u32) -> bool {
        IsNotEqual(self as u32, r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i64> for u16 {
    fn safe_lt(self, r: i64) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: i64) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: i64) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: i64) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: i64) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: i64) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u64> for u16 {
    fn safe_lt(self, r: u64) -> bool {
        IsLess(self as u64, r as u64)
    }
    fn safe_le(self, r: u64) -> bool {
        IsLessOrEqual(self as u64, r as u64)
    }
    fn safe_gt(self, r: u64) -> bool {
        IsGreater(self as u64, r as u64)
    }
    fn safe_ge(self, r: u64) -> bool {
        IsGreaterOrEqual(self as u64, r as u64)
    }
    fn safe_eq(self, r: u64) -> bool {
        IsEqual(self as u64, r as u64)
    }
    fn safe_ne(self, r: u64) -> bool {
        IsNotEqual(self as u64, r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<f32> for u16 {
    fn safe_lt(self, r: f32) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: f32) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: f32) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: f32) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: f32) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: f32) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<f64> for u16 {
    fn safe_lt(self, r: f64) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: f64) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: f64) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: f64) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: f64) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: f64) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<isize> for u16 {
    fn safe_lt(self, r: isize) -> bool {
        IsLess(self as isize, r as isize)
    }
    fn safe_le(self, r: isize) -> bool {
        IsLessOrEqual(self as isize, r as isize)
    }
    fn safe_gt(self, r: isize) -> bool {
        IsGreater(self as isize, r as isize)
    }
    fn safe_ge(self, r: isize) -> bool {
        IsGreaterOrEqual(self as isize, r as isize)
    }
    fn safe_eq(self, r: isize) -> bool {
        IsEqual(self as isize, r as isize)
    }
    fn safe_ne(self, r: isize) -> bool {
        IsNotEqual(self as isize, r as isize)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<usize> for u16 {
    fn safe_lt(self, r: usize) -> bool {
        IsLess(self as usize, r as usize)
    }
    fn safe_le(self, r: usize) -> bool {
        IsLessOrEqual(self as usize, r as usize)
    }
    fn safe_gt(self, r: usize) -> bool {
        IsGreater(self as usize, r as usize)
    }
    fn safe_ge(self, r: usize) -> bool {
        IsGreaterOrEqual(self as usize, r as usize)
    }
    fn safe_eq(self, r: usize) -> bool {
        IsEqual(self as usize, r as usize)
    }
    fn safe_ne(self, r: usize) -> bool {
        IsNotEqual(self as usize, r as usize)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i8> for i32 {
    fn safe_lt(self, r: i8) -> bool {
        IsLess(self as i32, r as i32)
    }
    fn safe_le(self, r: i8) -> bool {
        IsLessOrEqual(self as i32, r as i32)
    }
    fn safe_gt(self, r: i8) -> bool {
        IsGreater(self as i32, r as i32)
    }
    fn safe_ge(self, r: i8) -> bool {
        IsGreaterOrEqual(self as i32, r as i32)
    }
    fn safe_eq(self, r: i8) -> bool {
        IsEqual(self as i32, r as i32)
    }
    fn safe_ne(self, r: i8) -> bool {
        IsNotEqual(self as i32, r as i32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u8> for i32 {
    fn safe_lt(self, r: u8) -> bool {
        IsLess(self as i32, r as i32)
    }
    fn safe_le(self, r: u8) -> bool {
        IsLessOrEqual(self as i32, r as i32)
    }
    fn safe_gt(self, r: u8) -> bool {
        IsGreater(self as i32, r as i32)
    }
    fn safe_ge(self, r: u8) -> bool {
        IsGreaterOrEqual(self as i32, r as i32)
    }
    fn safe_eq(self, r: u8) -> bool {
        IsEqual(self as i32, r as i32)
    }
    fn safe_ne(self, r: u8) -> bool {
        IsNotEqual(self as i32, r as i32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i16> for i32 {
    fn safe_lt(self, r: i16) -> bool {
        IsLess(self as i32, r as i32)
    }
    fn safe_le(self, r: i16) -> bool {
        IsLessOrEqual(self as i32, r as i32)
    }
    fn safe_gt(self, r: i16) -> bool {
        IsGreater(self as i32, r as i32)
    }
    fn safe_ge(self, r: i16) -> bool {
        IsGreaterOrEqual(self as i32, r as i32)
    }
    fn safe_eq(self, r: i16) -> bool {
        IsEqual(self as i32, r as i32)
    }
    fn safe_ne(self, r: i16) -> bool {
        IsNotEqual(self as i32, r as i32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u16> for i32 {
    fn safe_lt(self, r: u16) -> bool {
        IsLess(self as i32, r as i32)
    }
    fn safe_le(self, r: u16) -> bool {
        IsLessOrEqual(self as i32, r as i32)
    }
    fn safe_gt(self, r: u16) -> bool {
        IsGreater(self as i32, r as i32)
    }
    fn safe_ge(self, r: u16) -> bool {
        IsGreaterOrEqual(self as i32, r as i32)
    }
    fn safe_eq(self, r: u16) -> bool {
        IsEqual(self as i32, r as i32)
    }
    fn safe_ne(self, r: u16) -> bool {
        IsNotEqual(self as i32, r as i32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i32> for i32 {
    fn safe_lt(self, r: i32) -> bool {
        IsLess(self as i32, r as i32)
    }
    fn safe_le(self, r: i32) -> bool {
        IsLessOrEqual(self as i32, r as i32)
    }
    fn safe_gt(self, r: i32) -> bool {
        IsGreater(self as i32, r as i32)
    }
    fn safe_ge(self, r: i32) -> bool {
        IsGreaterOrEqual(self as i32, r as i32)
    }
    fn safe_eq(self, r: i32) -> bool {
        IsEqual(self as i32, r as i32)
    }
    fn safe_ne(self, r: i32) -> bool {
        IsNotEqual(self as i32, r as i32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u32> for i32 {
    fn safe_lt(self, r: u32) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: u32) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: u32) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: u32) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: u32) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: u32) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i64> for i32 {
    fn safe_lt(self, r: i64) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: i64) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: i64) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: i64) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: i64) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: i64) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u64> for i32 {
    fn safe_lt(self, r: u64) -> bool {
        IsLess(self, r)
    }
    fn safe_le(self, r: u64) -> bool {
        IsLessOrEqual(self, r)
    }
    fn safe_gt(self, r: u64) -> bool {
        IsGreater(self, r)
    }
    fn safe_ge(self, r: u64) -> bool {
        IsGreaterOrEqual(self, r)
    }
    fn safe_eq(self, r: u64) -> bool {
        IsEqual(self, r)
    }
    fn safe_ne(self, r: u64) -> bool {
        IsNotEqual(self, r)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<f32> for i32 {
    fn safe_lt(self, r: f32) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: f32) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: f32) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: f32) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: f32) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: f32) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<f64> for i32 {
    fn safe_lt(self, r: f64) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: f64) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: f64) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: f64) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: f64) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: f64) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<isize> for i32 {
    fn safe_lt(self, r: isize) -> bool {
        IsLess(self as isize, r as isize)
    }
    fn safe_le(self, r: isize) -> bool {
        IsLessOrEqual(self as isize, r as isize)
    }
    fn safe_gt(self, r: isize) -> bool {
        IsGreater(self as isize, r as isize)
    }
    fn safe_ge(self, r: isize) -> bool {
        IsGreaterOrEqual(self as isize, r as isize)
    }
    fn safe_eq(self, r: isize) -> bool {
        IsEqual(self as isize, r as isize)
    }
    fn safe_ne(self, r: isize) -> bool {
        IsNotEqual(self as isize, r as isize)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<usize> for i32 {
    fn safe_lt(self, r: usize) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: usize) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: usize) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: usize) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: usize) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: usize) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i8> for u32 {
    fn safe_lt(self, r: i8) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: i8) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: i8) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: i8) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: i8) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: i8) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u8> for u32 {
    fn safe_lt(self, r: u8) -> bool {
        IsLess(self as u32, r as u32)
    }
    fn safe_le(self, r: u8) -> bool {
        IsLessOrEqual(self as u32, r as u32)
    }
    fn safe_gt(self, r: u8) -> bool {
        IsGreater(self as u32, r as u32)
    }
    fn safe_ge(self, r: u8) -> bool {
        IsGreaterOrEqual(self as u32, r as u32)
    }
    fn safe_eq(self, r: u8) -> bool {
        IsEqual(self as u32, r as u32)
    }
    fn safe_ne(self, r: u8) -> bool {
        IsNotEqual(self as u32, r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i16> for u32 {
    fn safe_lt(self, r: i16) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: i16) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: i16) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: i16) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: i16) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: i16) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u16> for u32 {
    fn safe_lt(self, r: u16) -> bool {
        IsLess(self as u32, r as u32)
    }
    fn safe_le(self, r: u16) -> bool {
        IsLessOrEqual(self as u32, r as u32)
    }
    fn safe_gt(self, r: u16) -> bool {
        IsGreater(self as u32, r as u32)
    }
    fn safe_ge(self, r: u16) -> bool {
        IsGreaterOrEqual(self as u32, r as u32)
    }
    fn safe_eq(self, r: u16) -> bool {
        IsEqual(self as u32, r as u32)
    }
    fn safe_ne(self, r: u16) -> bool {
        IsNotEqual(self as u32, r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i32> for u32 {
    fn safe_lt(self, r: i32) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: i32) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: i32) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: i32) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: i32) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: i32) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u32> for u32 {
    fn safe_lt(self, r: u32) -> bool {
        IsLess(self as u32, r as u32)
    }
    fn safe_le(self, r: u32) -> bool {
        IsLessOrEqual(self as u32, r as u32)
    }
    fn safe_gt(self, r: u32) -> bool {
        IsGreater(self as u32, r as u32)
    }
    fn safe_ge(self, r: u32) -> bool {
        IsGreaterOrEqual(self as u32, r as u32)
    }
    fn safe_eq(self, r: u32) -> bool {
        IsEqual(self as u32, r as u32)
    }
    fn safe_ne(self, r: u32) -> bool {
        IsNotEqual(self as u32, r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i64> for u32 {
    fn safe_lt(self, r: i64) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: i64) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: i64) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: i64) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: i64) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: i64) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u64> for u32 {
    fn safe_lt(self, r: u64) -> bool {
        IsLess(self as u64, r as u64)
    }
    fn safe_le(self, r: u64) -> bool {
        IsLessOrEqual(self as u64, r as u64)
    }
    fn safe_gt(self, r: u64) -> bool {
        IsGreater(self as u64, r as u64)
    }
    fn safe_ge(self, r: u64) -> bool {
        IsGreaterOrEqual(self as u64, r as u64)
    }
    fn safe_eq(self, r: u64) -> bool {
        IsEqual(self as u64, r as u64)
    }
    fn safe_ne(self, r: u64) -> bool {
        IsNotEqual(self as u64, r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<f32> for u32 {
    fn safe_lt(self, r: f32) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: f32) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: f32) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: f32) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: f32) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: f32) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<f64> for u32 {
    fn safe_lt(self, r: f64) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: f64) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: f64) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: f64) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: f64) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: f64) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<isize> for u32 {
    fn safe_lt(self, r: isize) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: isize) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: isize) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: isize) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: isize) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: isize) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<usize> for u32 {
    fn safe_lt(self, r: usize) -> bool {
        IsLess(self as usize, r as usize)
    }
    fn safe_le(self, r: usize) -> bool {
        IsLessOrEqual(self as usize, r as usize)
    }
    fn safe_gt(self, r: usize) -> bool {
        IsGreater(self as usize, r as usize)
    }
    fn safe_ge(self, r: usize) -> bool {
        IsGreaterOrEqual(self as usize, r as usize)
    }
    fn safe_eq(self, r: usize) -> bool {
        IsEqual(self as usize, r as usize)
    }
    fn safe_ne(self, r: usize) -> bool {
        IsNotEqual(self as usize, r as usize)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i8> for i64 {
    fn safe_lt(self, r: i8) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: i8) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: i8) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: i8) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: i8) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: i8) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u8> for i64 {
    fn safe_lt(self, r: u8) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: u8) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: u8) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: u8) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: u8) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: u8) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i16> for i64 {
    fn safe_lt(self, r: i16) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: i16) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: i16) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: i16) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: i16) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: i16) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u16> for i64 {
    fn safe_lt(self, r: u16) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: u16) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: u16) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: u16) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: u16) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: u16) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i32> for i64 {
    fn safe_lt(self, r: i32) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: i32) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: i32) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: i32) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: i32) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: i32) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u32> for i64 {
    fn safe_lt(self, r: u32) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: u32) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: u32) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: u32) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: u32) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: u32) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i64> for i64 {
    fn safe_lt(self, r: i64) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: i64) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: i64) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: i64) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: i64) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: i64) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u64> for i64 {
    fn safe_lt(self, r: u64) -> bool {
        IsLess(self, r)
    }
    fn safe_le(self, r: u64) -> bool {
        IsLessOrEqual(self, r)
    }
    fn safe_gt(self, r: u64) -> bool {
        IsGreater(self, r)
    }
    fn safe_ge(self, r: u64) -> bool {
        IsGreaterOrEqual(self, r)
    }
    fn safe_eq(self, r: u64) -> bool {
        IsEqual(self, r)
    }
    fn safe_ne(self, r: u64) -> bool {
        IsNotEqual(self, r)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<f32> for i64 {
    fn safe_lt(self, r: f32) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: f32) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: f32) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: f32) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: f32) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: f32) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<f64> for i64 {
    fn safe_lt(self, r: f64) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: f64) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: f64) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: f64) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: f64) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: f64) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<isize> for i64 {
    fn safe_lt(self, r: isize) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: isize) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: isize) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: isize) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: isize) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: isize) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<usize> for i64 {
    fn safe_lt(self, r: usize) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: usize) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: usize) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: usize) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: usize) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: usize) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i8> for u64 {
    fn safe_lt(self, r: i8) -> bool {
        IsLess(self, r)
    }
    fn safe_le(self, r: i8) -> bool {
        IsLessOrEqual(self, r)
    }
    fn safe_gt(self, r: i8) -> bool {
        IsGreater(self, r)
    }
    fn safe_ge(self, r: i8) -> bool {
        IsGreaterOrEqual(self, r)
    }
    fn safe_eq(self, r: i8) -> bool {
        IsEqual(self, r)
    }
    fn safe_ne(self, r: i8) -> bool {
        IsNotEqual(self, r)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u8> for u64 {
    fn safe_lt(self, r: u8) -> bool {
        IsLess(self as u64, r as u64)
    }
    fn safe_le(self, r: u8) -> bool {
        IsLessOrEqual(self as u64, r as u64)
    }
    fn safe_gt(self, r: u8) -> bool {
        IsGreater(self as u64, r as u64)
    }
    fn safe_ge(self, r: u8) -> bool {
        IsGreaterOrEqual(self as u64, r as u64)
    }
    fn safe_eq(self, r: u8) -> bool {
        IsEqual(self as u64, r as u64)
    }
    fn safe_ne(self, r: u8) -> bool {
        IsNotEqual(self as u64, r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i16> for u64 {
    fn safe_lt(self, r: i16) -> bool {
        IsLess(self, r)
    }
    fn safe_le(self, r: i16) -> bool {
        IsLessOrEqual(self, r)
    }
    fn safe_gt(self, r: i16) -> bool {
        IsGreater(self, r)
    }
    fn safe_ge(self, r: i16) -> bool {
        IsGreaterOrEqual(self, r)
    }
    fn safe_eq(self, r: i16) -> bool {
        IsEqual(self, r)
    }
    fn safe_ne(self, r: i16) -> bool {
        IsNotEqual(self, r)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u16> for u64 {
    fn safe_lt(self, r: u16) -> bool {
        IsLess(self as u64, r as u64)
    }
    fn safe_le(self, r: u16) -> bool {
        IsLessOrEqual(self as u64, r as u64)
    }
    fn safe_gt(self, r: u16) -> bool {
        IsGreater(self as u64, r as u64)
    }
    fn safe_ge(self, r: u16) -> bool {
        IsGreaterOrEqual(self as u64, r as u64)
    }
    fn safe_eq(self, r: u16) -> bool {
        IsEqual(self as u64, r as u64)
    }
    fn safe_ne(self, r: u16) -> bool {
        IsNotEqual(self as u64, r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i32> for u64 {
    fn safe_lt(self, r: i32) -> bool {
        IsLess(self, r)
    }
    fn safe_le(self, r: i32) -> bool {
        IsLessOrEqual(self, r)
    }
    fn safe_gt(self, r: i32) -> bool {
        IsGreater(self, r)
    }
    fn safe_ge(self, r: i32) -> bool {
        IsGreaterOrEqual(self, r)
    }
    fn safe_eq(self, r: i32) -> bool {
        IsEqual(self, r)
    }
    fn safe_ne(self, r: i32) -> bool {
        IsNotEqual(self, r)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u32> for u64 {
    fn safe_lt(self, r: u32) -> bool {
        IsLess(self as u64, r as u64)
    }
    fn safe_le(self, r: u32) -> bool {
        IsLessOrEqual(self as u64, r as u64)
    }
    fn safe_gt(self, r: u32) -> bool {
        IsGreater(self as u64, r as u64)
    }
    fn safe_ge(self, r: u32) -> bool {
        IsGreaterOrEqual(self as u64, r as u64)
    }
    fn safe_eq(self, r: u32) -> bool {
        IsEqual(self as u64, r as u64)
    }
    fn safe_ne(self, r: u32) -> bool {
        IsNotEqual(self as u64, r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i64> for u64 {
    fn safe_lt(self, r: i64) -> bool {
        IsLess(self, r)
    }
    fn safe_le(self, r: i64) -> bool {
        IsLessOrEqual(self, r)
    }
    fn safe_gt(self, r: i64) -> bool {
        IsGreater(self, r)
    }
    fn safe_ge(self, r: i64) -> bool {
        IsGreaterOrEqual(self, r)
    }
    fn safe_eq(self, r: i64) -> bool {
        IsEqual(self, r)
    }
    fn safe_ne(self, r: i64) -> bool {
        IsNotEqual(self, r)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u64> for u64 {
    fn safe_lt(self, r: u64) -> bool {
        IsLess(self as u64, r as u64)
    }
    fn safe_le(self, r: u64) -> bool {
        IsLessOrEqual(self as u64, r as u64)
    }
    fn safe_gt(self, r: u64) -> bool {
        IsGreater(self as u64, r as u64)
    }
    fn safe_ge(self, r: u64) -> bool {
        IsGreaterOrEqual(self as u64, r as u64)
    }
    fn safe_eq(self, r: u64) -> bool {
        IsEqual(self as u64, r as u64)
    }
    fn safe_ne(self, r: u64) -> bool {
        IsNotEqual(self as u64, r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<f32> for u64 {
    fn safe_lt(self, r: f32) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: f32) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: f32) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: f32) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: f32) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: f32) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<f64> for u64 {
    fn safe_lt(self, r: f64) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: f64) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: f64) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: f64) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: f64) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: f64) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<isize> for u64 {
    fn safe_lt(self, r: isize) -> bool {
        IsLess(self, r)
    }
    fn safe_le(self, r: isize) -> bool {
        IsLessOrEqual(self, r)
    }
    fn safe_gt(self, r: isize) -> bool {
        IsGreater(self, r)
    }
    fn safe_ge(self, r: isize) -> bool {
        IsGreaterOrEqual(self, r)
    }
    fn safe_eq(self, r: isize) -> bool {
        IsEqual(self, r)
    }
    fn safe_ne(self, r: isize) -> bool {
        IsNotEqual(self, r)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<usize> for u64 {
    fn safe_lt(self, r: usize) -> bool {
        IsLess(self as u64, r as u64)
    }
    fn safe_le(self, r: usize) -> bool {
        IsLessOrEqual(self as u64, r as u64)
    }
    fn safe_gt(self, r: usize) -> bool {
        IsGreater(self as u64, r as u64)
    }
    fn safe_ge(self, r: usize) -> bool {
        IsGreaterOrEqual(self as u64, r as u64)
    }
    fn safe_eq(self, r: usize) -> bool {
        IsEqual(self as u64, r as u64)
    }
    fn safe_ne(self, r: usize) -> bool {
        IsNotEqual(self as u64, r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i8> for f32 {
    fn safe_lt(self, r: i8) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: i8) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: i8) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: i8) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: i8) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: i8) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u8> for f32 {
    fn safe_lt(self, r: u8) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: u8) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: u8) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: u8) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: u8) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: u8) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i16> for f32 {
    fn safe_lt(self, r: i16) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: i16) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: i16) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: i16) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: i16) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: i16) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u16> for f32 {
    fn safe_lt(self, r: u16) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: u16) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: u16) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: u16) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: u16) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: u16) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i32> for f32 {
    fn safe_lt(self, r: i32) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: i32) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: i32) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: i32) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: i32) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: i32) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u32> for f32 {
    fn safe_lt(self, r: u32) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: u32) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: u32) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: u32) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: u32) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: u32) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i64> for f32 {
    fn safe_lt(self, r: i64) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: i64) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: i64) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: i64) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: i64) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: i64) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u64> for f32 {
    fn safe_lt(self, r: u64) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: u64) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: u64) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: u64) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: u64) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: u64) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<f32> for f32 {
    fn safe_lt(self, r: f32) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: f32) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: f32) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: f32) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: f32) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: f32) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<f64> for f32 {
    fn safe_lt(self, r: f64) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: f64) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: f64) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: f64) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: f64) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: f64) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<isize> for f32 {
    fn safe_lt(self, r: isize) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: isize) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: isize) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: isize) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: isize) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: isize) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<usize> for f32 {
    fn safe_lt(self, r: usize) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: usize) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: usize) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: usize) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: usize) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: usize) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i8> for f64 {
    fn safe_lt(self, r: i8) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: i8) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: i8) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: i8) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: i8) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: i8) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u8> for f64 {
    fn safe_lt(self, r: u8) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: u8) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: u8) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: u8) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: u8) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: u8) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i16> for f64 {
    fn safe_lt(self, r: i16) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: i16) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: i16) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: i16) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: i16) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: i16) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u16> for f64 {
    fn safe_lt(self, r: u16) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: u16) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: u16) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: u16) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: u16) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: u16) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i32> for f64 {
    fn safe_lt(self, r: i32) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: i32) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: i32) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: i32) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: i32) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: i32) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u32> for f64 {
    fn safe_lt(self, r: u32) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: u32) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: u32) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: u32) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: u32) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: u32) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i64> for f64 {
    fn safe_lt(self, r: i64) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: i64) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: i64) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: i64) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: i64) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: i64) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u64> for f64 {
    fn safe_lt(self, r: u64) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: u64) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: u64) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: u64) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: u64) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: u64) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<f32> for f64 {
    fn safe_lt(self, r: f32) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: f32) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: f32) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: f32) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: f32) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: f32) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<f64> for f64 {
    fn safe_lt(self, r: f64) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: f64) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: f64) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: f64) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: f64) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: f64) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<isize> for f64 {
    fn safe_lt(self, r: isize) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: isize) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: isize) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: isize) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: isize) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: isize) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<usize> for f64 {
    fn safe_lt(self, r: usize) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: usize) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: usize) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: usize) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: usize) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: usize) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i8> for isize {
    fn safe_lt(self, r: i8) -> bool {
        IsLess(self as isize, r as isize)
    }
    fn safe_le(self, r: i8) -> bool {
        IsLessOrEqual(self as isize, r as isize)
    }
    fn safe_gt(self, r: i8) -> bool {
        IsGreater(self as isize, r as isize)
    }
    fn safe_ge(self, r: i8) -> bool {
        IsGreaterOrEqual(self as isize, r as isize)
    }
    fn safe_eq(self, r: i8) -> bool {
        IsEqual(self as isize, r as isize)
    }
    fn safe_ne(self, r: i8) -> bool {
        IsNotEqual(self as isize, r as isize)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u8> for isize {
    fn safe_lt(self, r: u8) -> bool {
        IsLess(self as isize, r as isize)
    }
    fn safe_le(self, r: u8) -> bool {
        IsLessOrEqual(self as isize, r as isize)
    }
    fn safe_gt(self, r: u8) -> bool {
        IsGreater(self as isize, r as isize)
    }
    fn safe_ge(self, r: u8) -> bool {
        IsGreaterOrEqual(self as isize, r as isize)
    }
    fn safe_eq(self, r: u8) -> bool {
        IsEqual(self as isize, r as isize)
    }
    fn safe_ne(self, r: u8) -> bool {
        IsNotEqual(self as isize, r as isize)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i16> for isize {
    fn safe_lt(self, r: i16) -> bool {
        IsLess(self as isize, r as isize)
    }
    fn safe_le(self, r: i16) -> bool {
        IsLessOrEqual(self as isize, r as isize)
    }
    fn safe_gt(self, r: i16) -> bool {
        IsGreater(self as isize, r as isize)
    }
    fn safe_ge(self, r: i16) -> bool {
        IsGreaterOrEqual(self as isize, r as isize)
    }
    fn safe_eq(self, r: i16) -> bool {
        IsEqual(self as isize, r as isize)
    }
    fn safe_ne(self, r: i16) -> bool {
        IsNotEqual(self as isize, r as isize)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u16> for isize {
    fn safe_lt(self, r: u16) -> bool {
        IsLess(self as isize, r as isize)
    }
    fn safe_le(self, r: u16) -> bool {
        IsLessOrEqual(self as isize, r as isize)
    }
    fn safe_gt(self, r: u16) -> bool {
        IsGreater(self as isize, r as isize)
    }
    fn safe_ge(self, r: u16) -> bool {
        IsGreaterOrEqual(self as isize, r as isize)
    }
    fn safe_eq(self, r: u16) -> bool {
        IsEqual(self as isize, r as isize)
    }
    fn safe_ne(self, r: u16) -> bool {
        IsNotEqual(self as isize, r as isize)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i32> for isize {
    fn safe_lt(self, r: i32) -> bool {
        IsLess(self as i32, r as i32)
    }
    fn safe_le(self, r: i32) -> bool {
        IsLessOrEqual(self as i32, r as i32)
    }
    fn safe_gt(self, r: i32) -> bool {
        IsGreater(self as i32, r as i32)
    }
    fn safe_ge(self, r: i32) -> bool {
        IsGreaterOrEqual(self as i32, r as i32)
    }
    fn safe_eq(self, r: i32) -> bool {
        IsEqual(self as i32, r as i32)
    }
    fn safe_ne(self, r: i32) -> bool {
        IsNotEqual(self as i32, r as i32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u32> for isize {
    fn safe_lt(self, r: u32) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: u32) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: u32) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: u32) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: u32) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: u32) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i64> for isize {
    fn safe_lt(self, r: i64) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: i64) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: i64) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: i64) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: i64) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: i64) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u64> for isize {
    fn safe_lt(self, r: u64) -> bool {
        IsLess(self, r)
    }
    fn safe_le(self, r: u64) -> bool {
        IsLessOrEqual(self, r)
    }
    fn safe_gt(self, r: u64) -> bool {
        IsGreater(self, r)
    }
    fn safe_ge(self, r: u64) -> bool {
        IsGreaterOrEqual(self, r)
    }
    fn safe_eq(self, r: u64) -> bool {
        IsEqual(self, r)
    }
    fn safe_ne(self, r: u64) -> bool {
        IsNotEqual(self, r)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<f32> for isize {
    fn safe_lt(self, r: f32) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: f32) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: f32) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: f32) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: f32) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: f32) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<f64> for isize {
    fn safe_lt(self, r: f64) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: f64) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: f64) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: f64) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: f64) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: f64) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<isize> for isize {
    fn safe_lt(self, r: isize) -> bool {
        IsLess(self as isize, r as isize)
    }
    fn safe_le(self, r: isize) -> bool {
        IsLessOrEqual(self as isize, r as isize)
    }
    fn safe_gt(self, r: isize) -> bool {
        IsGreater(self as isize, r as isize)
    }
    fn safe_ge(self, r: isize) -> bool {
        IsGreaterOrEqual(self as isize, r as isize)
    }
    fn safe_eq(self, r: isize) -> bool {
        IsEqual(self as isize, r as isize)
    }
    fn safe_ne(self, r: isize) -> bool {
        IsNotEqual(self as isize, r as isize)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<usize> for isize {
    fn safe_lt(self, r: usize) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: usize) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: usize) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: usize) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: usize) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: usize) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i8> for usize {
    fn safe_lt(self, r: i8) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: i8) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: i8) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: i8) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: i8) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: i8) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u8> for usize {
    fn safe_lt(self, r: u8) -> bool {
        IsLess(self as usize, r as usize)
    }
    fn safe_le(self, r: u8) -> bool {
        IsLessOrEqual(self as usize, r as usize)
    }
    fn safe_gt(self, r: u8) -> bool {
        IsGreater(self as usize, r as usize)
    }
    fn safe_ge(self, r: u8) -> bool {
        IsGreaterOrEqual(self as usize, r as usize)
    }
    fn safe_eq(self, r: u8) -> bool {
        IsEqual(self as usize, r as usize)
    }
    fn safe_ne(self, r: u8) -> bool {
        IsNotEqual(self as usize, r as usize)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i16> for usize {
    fn safe_lt(self, r: i16) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: i16) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: i16) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: i16) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: i16) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: i16) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u16> for usize {
    fn safe_lt(self, r: u16) -> bool {
        IsLess(self as usize, r as usize)
    }
    fn safe_le(self, r: u16) -> bool {
        IsLessOrEqual(self as usize, r as usize)
    }
    fn safe_gt(self, r: u16) -> bool {
        IsGreater(self as usize, r as usize)
    }
    fn safe_ge(self, r: u16) -> bool {
        IsGreaterOrEqual(self as usize, r as usize)
    }
    fn safe_eq(self, r: u16) -> bool {
        IsEqual(self as usize, r as usize)
    }
    fn safe_ne(self, r: u16) -> bool {
        IsNotEqual(self as usize, r as usize)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i32> for usize {
    fn safe_lt(self, r: i32) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: i32) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: i32) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: i32) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: i32) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: i32) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u32> for usize {
    fn safe_lt(self, r: u32) -> bool {
        IsLess(self as u32, r as u32)
    }
    fn safe_le(self, r: u32) -> bool {
        IsLessOrEqual(self as u32, r as u32)
    }
    fn safe_gt(self, r: u32) -> bool {
        IsGreater(self as u32, r as u32)
    }
    fn safe_ge(self, r: u32) -> bool {
        IsGreaterOrEqual(self as u32, r as u32)
    }
    fn safe_eq(self, r: u32) -> bool {
        IsEqual(self as u32, r as u32)
    }
    fn safe_ne(self, r: u32) -> bool {
        IsNotEqual(self as u32, r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<i64> for usize {
    fn safe_lt(self, r: i64) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: i64) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: i64) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: i64) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: i64) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: i64) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<u64> for usize {
    fn safe_lt(self, r: u64) -> bool {
        IsLess(self as u64, r as u64)
    }
    fn safe_le(self, r: u64) -> bool {
        IsLessOrEqual(self as u64, r as u64)
    }
    fn safe_gt(self, r: u64) -> bool {
        IsGreater(self as u64, r as u64)
    }
    fn safe_ge(self, r: u64) -> bool {
        IsGreaterOrEqual(self as u64, r as u64)
    }
    fn safe_eq(self, r: u64) -> bool {
        IsEqual(self as u64, r as u64)
    }
    fn safe_ne(self, r: u64) -> bool {
        IsNotEqual(self as u64, r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<f32> for usize {
    fn safe_lt(self, r: f32) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: f32) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: f32) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: f32) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: f32) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: f32) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<f64> for usize {
    fn safe_lt(self, r: f64) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: f64) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: f64) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: f64) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: f64) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: f64) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<isize> for usize {
    fn safe_lt(self, r: isize) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: isize) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: isize) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: isize) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: isize) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: isize) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "32")]
impl SafeComparison<usize> for usize {
    fn safe_lt(self, r: usize) -> bool {
        IsLess(self as usize, r as usize)
    }
    fn safe_le(self, r: usize) -> bool {
        IsLessOrEqual(self as usize, r as usize)
    }
    fn safe_gt(self, r: usize) -> bool {
        IsGreater(self as usize, r as usize)
    }
    fn safe_ge(self, r: usize) -> bool {
        IsGreaterOrEqual(self as usize, r as usize)
    }
    fn safe_eq(self, r: usize) -> bool {
        IsEqual(self as usize, r as usize)
    }
    fn safe_ne(self, r: usize) -> bool {
        IsNotEqual(self as usize, r as usize)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i8> for i8 {
    fn safe_lt(self, r: i8) -> bool {
        IsLess(self as i8, r as i8)
    }
    fn safe_le(self, r: i8) -> bool {
        IsLessOrEqual(self as i8, r as i8)
    }
    fn safe_gt(self, r: i8) -> bool {
        IsGreater(self as i8, r as i8)
    }
    fn safe_ge(self, r: i8) -> bool {
        IsGreaterOrEqual(self as i8, r as i8)
    }
    fn safe_eq(self, r: i8) -> bool {
        IsEqual(self as i8, r as i8)
    }
    fn safe_ne(self, r: i8) -> bool {
        IsNotEqual(self as i8, r as i8)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u8> for i8 {
    fn safe_lt(self, r: u8) -> bool {
        IsLess(self as i16, r as i16)
    }
    fn safe_le(self, r: u8) -> bool {
        IsLessOrEqual(self as i16, r as i16)
    }
    fn safe_gt(self, r: u8) -> bool {
        IsGreater(self as i16, r as i16)
    }
    fn safe_ge(self, r: u8) -> bool {
        IsGreaterOrEqual(self as i16, r as i16)
    }
    fn safe_eq(self, r: u8) -> bool {
        IsEqual(self as i16, r as i16)
    }
    fn safe_ne(self, r: u8) -> bool {
        IsNotEqual(self as i16, r as i16)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i16> for i8 {
    fn safe_lt(self, r: i16) -> bool {
        IsLess(self as i16, r as i16)
    }
    fn safe_le(self, r: i16) -> bool {
        IsLessOrEqual(self as i16, r as i16)
    }
    fn safe_gt(self, r: i16) -> bool {
        IsGreater(self as i16, r as i16)
    }
    fn safe_ge(self, r: i16) -> bool {
        IsGreaterOrEqual(self as i16, r as i16)
    }
    fn safe_eq(self, r: i16) -> bool {
        IsEqual(self as i16, r as i16)
    }
    fn safe_ne(self, r: i16) -> bool {
        IsNotEqual(self as i16, r as i16)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u16> for i8 {
    fn safe_lt(self, r: u16) -> bool {
        IsLess(self as i32, r as i32)
    }
    fn safe_le(self, r: u16) -> bool {
        IsLessOrEqual(self as i32, r as i32)
    }
    fn safe_gt(self, r: u16) -> bool {
        IsGreater(self as i32, r as i32)
    }
    fn safe_ge(self, r: u16) -> bool {
        IsGreaterOrEqual(self as i32, r as i32)
    }
    fn safe_eq(self, r: u16) -> bool {
        IsEqual(self as i32, r as i32)
    }
    fn safe_ne(self, r: u16) -> bool {
        IsNotEqual(self as i32, r as i32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i32> for i8 {
    fn safe_lt(self, r: i32) -> bool {
        IsLess(self as i32, r as i32)
    }
    fn safe_le(self, r: i32) -> bool {
        IsLessOrEqual(self as i32, r as i32)
    }
    fn safe_gt(self, r: i32) -> bool {
        IsGreater(self as i32, r as i32)
    }
    fn safe_ge(self, r: i32) -> bool {
        IsGreaterOrEqual(self as i32, r as i32)
    }
    fn safe_eq(self, r: i32) -> bool {
        IsEqual(self as i32, r as i32)
    }
    fn safe_ne(self, r: i32) -> bool {
        IsNotEqual(self as i32, r as i32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u32> for i8 {
    fn safe_lt(self, r: u32) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: u32) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: u32) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: u32) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: u32) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: u32) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i64> for i8 {
    fn safe_lt(self, r: i64) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: i64) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: i64) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: i64) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: i64) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: i64) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u64> for i8 {
    fn safe_lt(self, r: u64) -> bool {
        IsLess(self, r)
    }
    fn safe_le(self, r: u64) -> bool {
        IsLessOrEqual(self, r)
    }
    fn safe_gt(self, r: u64) -> bool {
        IsGreater(self, r)
    }
    fn safe_ge(self, r: u64) -> bool {
        IsGreaterOrEqual(self, r)
    }
    fn safe_eq(self, r: u64) -> bool {
        IsEqual(self, r)
    }
    fn safe_ne(self, r: u64) -> bool {
        IsNotEqual(self, r)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<f32> for i8 {
    fn safe_lt(self, r: f32) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: f32) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: f32) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: f32) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: f32) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: f32) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<f64> for i8 {
    fn safe_lt(self, r: f64) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: f64) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: f64) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: f64) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: f64) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: f64) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<isize> for i8 {
    fn safe_lt(self, r: isize) -> bool {
        IsLess(self as isize, r as isize)
    }
    fn safe_le(self, r: isize) -> bool {
        IsLessOrEqual(self as isize, r as isize)
    }
    fn safe_gt(self, r: isize) -> bool {
        IsGreater(self as isize, r as isize)
    }
    fn safe_ge(self, r: isize) -> bool {
        IsGreaterOrEqual(self as isize, r as isize)
    }
    fn safe_eq(self, r: isize) -> bool {
        IsEqual(self as isize, r as isize)
    }
    fn safe_ne(self, r: isize) -> bool {
        IsNotEqual(self as isize, r as isize)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<usize> for i8 {
    fn safe_lt(self, r: usize) -> bool {
        IsLess(self, r)
    }
    fn safe_le(self, r: usize) -> bool {
        IsLessOrEqual(self, r)
    }
    fn safe_gt(self, r: usize) -> bool {
        IsGreater(self, r)
    }
    fn safe_ge(self, r: usize) -> bool {
        IsGreaterOrEqual(self, r)
    }
    fn safe_eq(self, r: usize) -> bool {
        IsEqual(self, r)
    }
    fn safe_ne(self, r: usize) -> bool {
        IsNotEqual(self, r)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i8> for u8 {
    fn safe_lt(self, r: i8) -> bool {
        IsLess(self as i16, r as i16)
    }
    fn safe_le(self, r: i8) -> bool {
        IsLessOrEqual(self as i16, r as i16)
    }
    fn safe_gt(self, r: i8) -> bool {
        IsGreater(self as i16, r as i16)
    }
    fn safe_ge(self, r: i8) -> bool {
        IsGreaterOrEqual(self as i16, r as i16)
    }
    fn safe_eq(self, r: i8) -> bool {
        IsEqual(self as i16, r as i16)
    }
    fn safe_ne(self, r: i8) -> bool {
        IsNotEqual(self as i16, r as i16)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u8> for u8 {
    fn safe_lt(self, r: u8) -> bool {
        IsLess(self as u8, r as u8)
    }
    fn safe_le(self, r: u8) -> bool {
        IsLessOrEqual(self as u8, r as u8)
    }
    fn safe_gt(self, r: u8) -> bool {
        IsGreater(self as u8, r as u8)
    }
    fn safe_ge(self, r: u8) -> bool {
        IsGreaterOrEqual(self as u8, r as u8)
    }
    fn safe_eq(self, r: u8) -> bool {
        IsEqual(self as u8, r as u8)
    }
    fn safe_ne(self, r: u8) -> bool {
        IsNotEqual(self as u8, r as u8)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i16> for u8 {
    fn safe_lt(self, r: i16) -> bool {
        IsLess(self as i16, r as i16)
    }
    fn safe_le(self, r: i16) -> bool {
        IsLessOrEqual(self as i16, r as i16)
    }
    fn safe_gt(self, r: i16) -> bool {
        IsGreater(self as i16, r as i16)
    }
    fn safe_ge(self, r: i16) -> bool {
        IsGreaterOrEqual(self as i16, r as i16)
    }
    fn safe_eq(self, r: i16) -> bool {
        IsEqual(self as i16, r as i16)
    }
    fn safe_ne(self, r: i16) -> bool {
        IsNotEqual(self as i16, r as i16)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u16> for u8 {
    fn safe_lt(self, r: u16) -> bool {
        IsLess(self as u16, r as u16)
    }
    fn safe_le(self, r: u16) -> bool {
        IsLessOrEqual(self as u16, r as u16)
    }
    fn safe_gt(self, r: u16) -> bool {
        IsGreater(self as u16, r as u16)
    }
    fn safe_ge(self, r: u16) -> bool {
        IsGreaterOrEqual(self as u16, r as u16)
    }
    fn safe_eq(self, r: u16) -> bool {
        IsEqual(self as u16, r as u16)
    }
    fn safe_ne(self, r: u16) -> bool {
        IsNotEqual(self as u16, r as u16)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i32> for u8 {
    fn safe_lt(self, r: i32) -> bool {
        IsLess(self as i32, r as i32)
    }
    fn safe_le(self, r: i32) -> bool {
        IsLessOrEqual(self as i32, r as i32)
    }
    fn safe_gt(self, r: i32) -> bool {
        IsGreater(self as i32, r as i32)
    }
    fn safe_ge(self, r: i32) -> bool {
        IsGreaterOrEqual(self as i32, r as i32)
    }
    fn safe_eq(self, r: i32) -> bool {
        IsEqual(self as i32, r as i32)
    }
    fn safe_ne(self, r: i32) -> bool {
        IsNotEqual(self as i32, r as i32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u32> for u8 {
    fn safe_lt(self, r: u32) -> bool {
        IsLess(self as u32, r as u32)
    }
    fn safe_le(self, r: u32) -> bool {
        IsLessOrEqual(self as u32, r as u32)
    }
    fn safe_gt(self, r: u32) -> bool {
        IsGreater(self as u32, r as u32)
    }
    fn safe_ge(self, r: u32) -> bool {
        IsGreaterOrEqual(self as u32, r as u32)
    }
    fn safe_eq(self, r: u32) -> bool {
        IsEqual(self as u32, r as u32)
    }
    fn safe_ne(self, r: u32) -> bool {
        IsNotEqual(self as u32, r as u32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i64> for u8 {
    fn safe_lt(self, r: i64) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: i64) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: i64) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: i64) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: i64) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: i64) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u64> for u8 {
    fn safe_lt(self, r: u64) -> bool {
        IsLess(self as u64, r as u64)
    }
    fn safe_le(self, r: u64) -> bool {
        IsLessOrEqual(self as u64, r as u64)
    }
    fn safe_gt(self, r: u64) -> bool {
        IsGreater(self as u64, r as u64)
    }
    fn safe_ge(self, r: u64) -> bool {
        IsGreaterOrEqual(self as u64, r as u64)
    }
    fn safe_eq(self, r: u64) -> bool {
        IsEqual(self as u64, r as u64)
    }
    fn safe_ne(self, r: u64) -> bool {
        IsNotEqual(self as u64, r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<f32> for u8 {
    fn safe_lt(self, r: f32) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: f32) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: f32) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: f32) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: f32) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: f32) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<f64> for u8 {
    fn safe_lt(self, r: f64) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: f64) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: f64) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: f64) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: f64) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: f64) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<isize> for u8 {
    fn safe_lt(self, r: isize) -> bool {
        IsLess(self as isize, r as isize)
    }
    fn safe_le(self, r: isize) -> bool {
        IsLessOrEqual(self as isize, r as isize)
    }
    fn safe_gt(self, r: isize) -> bool {
        IsGreater(self as isize, r as isize)
    }
    fn safe_ge(self, r: isize) -> bool {
        IsGreaterOrEqual(self as isize, r as isize)
    }
    fn safe_eq(self, r: isize) -> bool {
        IsEqual(self as isize, r as isize)
    }
    fn safe_ne(self, r: isize) -> bool {
        IsNotEqual(self as isize, r as isize)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<usize> for u8 {
    fn safe_lt(self, r: usize) -> bool {
        IsLess(self as usize, r as usize)
    }
    fn safe_le(self, r: usize) -> bool {
        IsLessOrEqual(self as usize, r as usize)
    }
    fn safe_gt(self, r: usize) -> bool {
        IsGreater(self as usize, r as usize)
    }
    fn safe_ge(self, r: usize) -> bool {
        IsGreaterOrEqual(self as usize, r as usize)
    }
    fn safe_eq(self, r: usize) -> bool {
        IsEqual(self as usize, r as usize)
    }
    fn safe_ne(self, r: usize) -> bool {
        IsNotEqual(self as usize, r as usize)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i8> for i16 {
    fn safe_lt(self, r: i8) -> bool {
        IsLess(self as i16, r as i16)
    }
    fn safe_le(self, r: i8) -> bool {
        IsLessOrEqual(self as i16, r as i16)
    }
    fn safe_gt(self, r: i8) -> bool {
        IsGreater(self as i16, r as i16)
    }
    fn safe_ge(self, r: i8) -> bool {
        IsGreaterOrEqual(self as i16, r as i16)
    }
    fn safe_eq(self, r: i8) -> bool {
        IsEqual(self as i16, r as i16)
    }
    fn safe_ne(self, r: i8) -> bool {
        IsNotEqual(self as i16, r as i16)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u8> for i16 {
    fn safe_lt(self, r: u8) -> bool {
        IsLess(self as i16, r as i16)
    }
    fn safe_le(self, r: u8) -> bool {
        IsLessOrEqual(self as i16, r as i16)
    }
    fn safe_gt(self, r: u8) -> bool {
        IsGreater(self as i16, r as i16)
    }
    fn safe_ge(self, r: u8) -> bool {
        IsGreaterOrEqual(self as i16, r as i16)
    }
    fn safe_eq(self, r: u8) -> bool {
        IsEqual(self as i16, r as i16)
    }
    fn safe_ne(self, r: u8) -> bool {
        IsNotEqual(self as i16, r as i16)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i16> for i16 {
    fn safe_lt(self, r: i16) -> bool {
        IsLess(self as i16, r as i16)
    }
    fn safe_le(self, r: i16) -> bool {
        IsLessOrEqual(self as i16, r as i16)
    }
    fn safe_gt(self, r: i16) -> bool {
        IsGreater(self as i16, r as i16)
    }
    fn safe_ge(self, r: i16) -> bool {
        IsGreaterOrEqual(self as i16, r as i16)
    }
    fn safe_eq(self, r: i16) -> bool {
        IsEqual(self as i16, r as i16)
    }
    fn safe_ne(self, r: i16) -> bool {
        IsNotEqual(self as i16, r as i16)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u16> for i16 {
    fn safe_lt(self, r: u16) -> bool {
        IsLess(self as i32, r as i32)
    }
    fn safe_le(self, r: u16) -> bool {
        IsLessOrEqual(self as i32, r as i32)
    }
    fn safe_gt(self, r: u16) -> bool {
        IsGreater(self as i32, r as i32)
    }
    fn safe_ge(self, r: u16) -> bool {
        IsGreaterOrEqual(self as i32, r as i32)
    }
    fn safe_eq(self, r: u16) -> bool {
        IsEqual(self as i32, r as i32)
    }
    fn safe_ne(self, r: u16) -> bool {
        IsNotEqual(self as i32, r as i32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i32> for i16 {
    fn safe_lt(self, r: i32) -> bool {
        IsLess(self as i32, r as i32)
    }
    fn safe_le(self, r: i32) -> bool {
        IsLessOrEqual(self as i32, r as i32)
    }
    fn safe_gt(self, r: i32) -> bool {
        IsGreater(self as i32, r as i32)
    }
    fn safe_ge(self, r: i32) -> bool {
        IsGreaterOrEqual(self as i32, r as i32)
    }
    fn safe_eq(self, r: i32) -> bool {
        IsEqual(self as i32, r as i32)
    }
    fn safe_ne(self, r: i32) -> bool {
        IsNotEqual(self as i32, r as i32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u32> for i16 {
    fn safe_lt(self, r: u32) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: u32) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: u32) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: u32) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: u32) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: u32) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i64> for i16 {
    fn safe_lt(self, r: i64) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: i64) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: i64) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: i64) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: i64) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: i64) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u64> for i16 {
    fn safe_lt(self, r: u64) -> bool {
        IsLess(self, r)
    }
    fn safe_le(self, r: u64) -> bool {
        IsLessOrEqual(self, r)
    }
    fn safe_gt(self, r: u64) -> bool {
        IsGreater(self, r)
    }
    fn safe_ge(self, r: u64) -> bool {
        IsGreaterOrEqual(self, r)
    }
    fn safe_eq(self, r: u64) -> bool {
        IsEqual(self, r)
    }
    fn safe_ne(self, r: u64) -> bool {
        IsNotEqual(self, r)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<f32> for i16 {
    fn safe_lt(self, r: f32) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: f32) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: f32) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: f32) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: f32) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: f32) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<f64> for i16 {
    fn safe_lt(self, r: f64) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: f64) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: f64) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: f64) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: f64) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: f64) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<isize> for i16 {
    fn safe_lt(self, r: isize) -> bool {
        IsLess(self as isize, r as isize)
    }
    fn safe_le(self, r: isize) -> bool {
        IsLessOrEqual(self as isize, r as isize)
    }
    fn safe_gt(self, r: isize) -> bool {
        IsGreater(self as isize, r as isize)
    }
    fn safe_ge(self, r: isize) -> bool {
        IsGreaterOrEqual(self as isize, r as isize)
    }
    fn safe_eq(self, r: isize) -> bool {
        IsEqual(self as isize, r as isize)
    }
    fn safe_ne(self, r: isize) -> bool {
        IsNotEqual(self as isize, r as isize)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<usize> for i16 {
    fn safe_lt(self, r: usize) -> bool {
        IsLess(self, r)
    }
    fn safe_le(self, r: usize) -> bool {
        IsLessOrEqual(self, r)
    }
    fn safe_gt(self, r: usize) -> bool {
        IsGreater(self, r)
    }
    fn safe_ge(self, r: usize) -> bool {
        IsGreaterOrEqual(self, r)
    }
    fn safe_eq(self, r: usize) -> bool {
        IsEqual(self, r)
    }
    fn safe_ne(self, r: usize) -> bool {
        IsNotEqual(self, r)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i8> for u16 {
    fn safe_lt(self, r: i8) -> bool {
        IsLess(self as i32, r as i32)
    }
    fn safe_le(self, r: i8) -> bool {
        IsLessOrEqual(self as i32, r as i32)
    }
    fn safe_gt(self, r: i8) -> bool {
        IsGreater(self as i32, r as i32)
    }
    fn safe_ge(self, r: i8) -> bool {
        IsGreaterOrEqual(self as i32, r as i32)
    }
    fn safe_eq(self, r: i8) -> bool {
        IsEqual(self as i32, r as i32)
    }
    fn safe_ne(self, r: i8) -> bool {
        IsNotEqual(self as i32, r as i32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u8> for u16 {
    fn safe_lt(self, r: u8) -> bool {
        IsLess(self as u16, r as u16)
    }
    fn safe_le(self, r: u8) -> bool {
        IsLessOrEqual(self as u16, r as u16)
    }
    fn safe_gt(self, r: u8) -> bool {
        IsGreater(self as u16, r as u16)
    }
    fn safe_ge(self, r: u8) -> bool {
        IsGreaterOrEqual(self as u16, r as u16)
    }
    fn safe_eq(self, r: u8) -> bool {
        IsEqual(self as u16, r as u16)
    }
    fn safe_ne(self, r: u8) -> bool {
        IsNotEqual(self as u16, r as u16)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i16> for u16 {
    fn safe_lt(self, r: i16) -> bool {
        IsLess(self as i32, r as i32)
    }
    fn safe_le(self, r: i16) -> bool {
        IsLessOrEqual(self as i32, r as i32)
    }
    fn safe_gt(self, r: i16) -> bool {
        IsGreater(self as i32, r as i32)
    }
    fn safe_ge(self, r: i16) -> bool {
        IsGreaterOrEqual(self as i32, r as i32)
    }
    fn safe_eq(self, r: i16) -> bool {
        IsEqual(self as i32, r as i32)
    }
    fn safe_ne(self, r: i16) -> bool {
        IsNotEqual(self as i32, r as i32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u16> for u16 {
    fn safe_lt(self, r: u16) -> bool {
        IsLess(self as u16, r as u16)
    }
    fn safe_le(self, r: u16) -> bool {
        IsLessOrEqual(self as u16, r as u16)
    }
    fn safe_gt(self, r: u16) -> bool {
        IsGreater(self as u16, r as u16)
    }
    fn safe_ge(self, r: u16) -> bool {
        IsGreaterOrEqual(self as u16, r as u16)
    }
    fn safe_eq(self, r: u16) -> bool {
        IsEqual(self as u16, r as u16)
    }
    fn safe_ne(self, r: u16) -> bool {
        IsNotEqual(self as u16, r as u16)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i32> for u16 {
    fn safe_lt(self, r: i32) -> bool {
        IsLess(self as i32, r as i32)
    }
    fn safe_le(self, r: i32) -> bool {
        IsLessOrEqual(self as i32, r as i32)
    }
    fn safe_gt(self, r: i32) -> bool {
        IsGreater(self as i32, r as i32)
    }
    fn safe_ge(self, r: i32) -> bool {
        IsGreaterOrEqual(self as i32, r as i32)
    }
    fn safe_eq(self, r: i32) -> bool {
        IsEqual(self as i32, r as i32)
    }
    fn safe_ne(self, r: i32) -> bool {
        IsNotEqual(self as i32, r as i32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u32> for u16 {
    fn safe_lt(self, r: u32) -> bool {
        IsLess(self as u32, r as u32)
    }
    fn safe_le(self, r: u32) -> bool {
        IsLessOrEqual(self as u32, r as u32)
    }
    fn safe_gt(self, r: u32) -> bool {
        IsGreater(self as u32, r as u32)
    }
    fn safe_ge(self, r: u32) -> bool {
        IsGreaterOrEqual(self as u32, r as u32)
    }
    fn safe_eq(self, r: u32) -> bool {
        IsEqual(self as u32, r as u32)
    }
    fn safe_ne(self, r: u32) -> bool {
        IsNotEqual(self as u32, r as u32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i64> for u16 {
    fn safe_lt(self, r: i64) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: i64) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: i64) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: i64) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: i64) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: i64) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u64> for u16 {
    fn safe_lt(self, r: u64) -> bool {
        IsLess(self as u64, r as u64)
    }
    fn safe_le(self, r: u64) -> bool {
        IsLessOrEqual(self as u64, r as u64)
    }
    fn safe_gt(self, r: u64) -> bool {
        IsGreater(self as u64, r as u64)
    }
    fn safe_ge(self, r: u64) -> bool {
        IsGreaterOrEqual(self as u64, r as u64)
    }
    fn safe_eq(self, r: u64) -> bool {
        IsEqual(self as u64, r as u64)
    }
    fn safe_ne(self, r: u64) -> bool {
        IsNotEqual(self as u64, r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<f32> for u16 {
    fn safe_lt(self, r: f32) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: f32) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: f32) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: f32) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: f32) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: f32) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<f64> for u16 {
    fn safe_lt(self, r: f64) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: f64) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: f64) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: f64) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: f64) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: f64) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<isize> for u16 {
    fn safe_lt(self, r: isize) -> bool {
        IsLess(self as isize, r as isize)
    }
    fn safe_le(self, r: isize) -> bool {
        IsLessOrEqual(self as isize, r as isize)
    }
    fn safe_gt(self, r: isize) -> bool {
        IsGreater(self as isize, r as isize)
    }
    fn safe_ge(self, r: isize) -> bool {
        IsGreaterOrEqual(self as isize, r as isize)
    }
    fn safe_eq(self, r: isize) -> bool {
        IsEqual(self as isize, r as isize)
    }
    fn safe_ne(self, r: isize) -> bool {
        IsNotEqual(self as isize, r as isize)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<usize> for u16 {
    fn safe_lt(self, r: usize) -> bool {
        IsLess(self as usize, r as usize)
    }
    fn safe_le(self, r: usize) -> bool {
        IsLessOrEqual(self as usize, r as usize)
    }
    fn safe_gt(self, r: usize) -> bool {
        IsGreater(self as usize, r as usize)
    }
    fn safe_ge(self, r: usize) -> bool {
        IsGreaterOrEqual(self as usize, r as usize)
    }
    fn safe_eq(self, r: usize) -> bool {
        IsEqual(self as usize, r as usize)
    }
    fn safe_ne(self, r: usize) -> bool {
        IsNotEqual(self as usize, r as usize)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i8> for i32 {
    fn safe_lt(self, r: i8) -> bool {
        IsLess(self as i32, r as i32)
    }
    fn safe_le(self, r: i8) -> bool {
        IsLessOrEqual(self as i32, r as i32)
    }
    fn safe_gt(self, r: i8) -> bool {
        IsGreater(self as i32, r as i32)
    }
    fn safe_ge(self, r: i8) -> bool {
        IsGreaterOrEqual(self as i32, r as i32)
    }
    fn safe_eq(self, r: i8) -> bool {
        IsEqual(self as i32, r as i32)
    }
    fn safe_ne(self, r: i8) -> bool {
        IsNotEqual(self as i32, r as i32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u8> for i32 {
    fn safe_lt(self, r: u8) -> bool {
        IsLess(self as i32, r as i32)
    }
    fn safe_le(self, r: u8) -> bool {
        IsLessOrEqual(self as i32, r as i32)
    }
    fn safe_gt(self, r: u8) -> bool {
        IsGreater(self as i32, r as i32)
    }
    fn safe_ge(self, r: u8) -> bool {
        IsGreaterOrEqual(self as i32, r as i32)
    }
    fn safe_eq(self, r: u8) -> bool {
        IsEqual(self as i32, r as i32)
    }
    fn safe_ne(self, r: u8) -> bool {
        IsNotEqual(self as i32, r as i32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i16> for i32 {
    fn safe_lt(self, r: i16) -> bool {
        IsLess(self as i32, r as i32)
    }
    fn safe_le(self, r: i16) -> bool {
        IsLessOrEqual(self as i32, r as i32)
    }
    fn safe_gt(self, r: i16) -> bool {
        IsGreater(self as i32, r as i32)
    }
    fn safe_ge(self, r: i16) -> bool {
        IsGreaterOrEqual(self as i32, r as i32)
    }
    fn safe_eq(self, r: i16) -> bool {
        IsEqual(self as i32, r as i32)
    }
    fn safe_ne(self, r: i16) -> bool {
        IsNotEqual(self as i32, r as i32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u16> for i32 {
    fn safe_lt(self, r: u16) -> bool {
        IsLess(self as i32, r as i32)
    }
    fn safe_le(self, r: u16) -> bool {
        IsLessOrEqual(self as i32, r as i32)
    }
    fn safe_gt(self, r: u16) -> bool {
        IsGreater(self as i32, r as i32)
    }
    fn safe_ge(self, r: u16) -> bool {
        IsGreaterOrEqual(self as i32, r as i32)
    }
    fn safe_eq(self, r: u16) -> bool {
        IsEqual(self as i32, r as i32)
    }
    fn safe_ne(self, r: u16) -> bool {
        IsNotEqual(self as i32, r as i32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i32> for i32 {
    fn safe_lt(self, r: i32) -> bool {
        IsLess(self as i32, r as i32)
    }
    fn safe_le(self, r: i32) -> bool {
        IsLessOrEqual(self as i32, r as i32)
    }
    fn safe_gt(self, r: i32) -> bool {
        IsGreater(self as i32, r as i32)
    }
    fn safe_ge(self, r: i32) -> bool {
        IsGreaterOrEqual(self as i32, r as i32)
    }
    fn safe_eq(self, r: i32) -> bool {
        IsEqual(self as i32, r as i32)
    }
    fn safe_ne(self, r: i32) -> bool {
        IsNotEqual(self as i32, r as i32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u32> for i32 {
    fn safe_lt(self, r: u32) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: u32) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: u32) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: u32) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: u32) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: u32) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i64> for i32 {
    fn safe_lt(self, r: i64) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: i64) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: i64) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: i64) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: i64) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: i64) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u64> for i32 {
    fn safe_lt(self, r: u64) -> bool {
        IsLess(self, r)
    }
    fn safe_le(self, r: u64) -> bool {
        IsLessOrEqual(self, r)
    }
    fn safe_gt(self, r: u64) -> bool {
        IsGreater(self, r)
    }
    fn safe_ge(self, r: u64) -> bool {
        IsGreaterOrEqual(self, r)
    }
    fn safe_eq(self, r: u64) -> bool {
        IsEqual(self, r)
    }
    fn safe_ne(self, r: u64) -> bool {
        IsNotEqual(self, r)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<f32> for i32 {
    fn safe_lt(self, r: f32) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: f32) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: f32) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: f32) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: f32) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: f32) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<f64> for i32 {
    fn safe_lt(self, r: f64) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: f64) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: f64) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: f64) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: f64) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: f64) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<isize> for i32 {
    fn safe_lt(self, r: isize) -> bool {
        IsLess(self as isize, r as isize)
    }
    fn safe_le(self, r: isize) -> bool {
        IsLessOrEqual(self as isize, r as isize)
    }
    fn safe_gt(self, r: isize) -> bool {
        IsGreater(self as isize, r as isize)
    }
    fn safe_ge(self, r: isize) -> bool {
        IsGreaterOrEqual(self as isize, r as isize)
    }
    fn safe_eq(self, r: isize) -> bool {
        IsEqual(self as isize, r as isize)
    }
    fn safe_ne(self, r: isize) -> bool {
        IsNotEqual(self as isize, r as isize)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<usize> for i32 {
    fn safe_lt(self, r: usize) -> bool {
        IsLess(self, r)
    }
    fn safe_le(self, r: usize) -> bool {
        IsLessOrEqual(self, r)
    }
    fn safe_gt(self, r: usize) -> bool {
        IsGreater(self, r)
    }
    fn safe_ge(self, r: usize) -> bool {
        IsGreaterOrEqual(self, r)
    }
    fn safe_eq(self, r: usize) -> bool {
        IsEqual(self, r)
    }
    fn safe_ne(self, r: usize) -> bool {
        IsNotEqual(self, r)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i8> for u32 {
    fn safe_lt(self, r: i8) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: i8) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: i8) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: i8) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: i8) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: i8) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u8> for u32 {
    fn safe_lt(self, r: u8) -> bool {
        IsLess(self as u32, r as u32)
    }
    fn safe_le(self, r: u8) -> bool {
        IsLessOrEqual(self as u32, r as u32)
    }
    fn safe_gt(self, r: u8) -> bool {
        IsGreater(self as u32, r as u32)
    }
    fn safe_ge(self, r: u8) -> bool {
        IsGreaterOrEqual(self as u32, r as u32)
    }
    fn safe_eq(self, r: u8) -> bool {
        IsEqual(self as u32, r as u32)
    }
    fn safe_ne(self, r: u8) -> bool {
        IsNotEqual(self as u32, r as u32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i16> for u32 {
    fn safe_lt(self, r: i16) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: i16) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: i16) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: i16) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: i16) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: i16) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u16> for u32 {
    fn safe_lt(self, r: u16) -> bool {
        IsLess(self as u32, r as u32)
    }
    fn safe_le(self, r: u16) -> bool {
        IsLessOrEqual(self as u32, r as u32)
    }
    fn safe_gt(self, r: u16) -> bool {
        IsGreater(self as u32, r as u32)
    }
    fn safe_ge(self, r: u16) -> bool {
        IsGreaterOrEqual(self as u32, r as u32)
    }
    fn safe_eq(self, r: u16) -> bool {
        IsEqual(self as u32, r as u32)
    }
    fn safe_ne(self, r: u16) -> bool {
        IsNotEqual(self as u32, r as u32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i32> for u32 {
    fn safe_lt(self, r: i32) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: i32) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: i32) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: i32) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: i32) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: i32) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u32> for u32 {
    fn safe_lt(self, r: u32) -> bool {
        IsLess(self as u32, r as u32)
    }
    fn safe_le(self, r: u32) -> bool {
        IsLessOrEqual(self as u32, r as u32)
    }
    fn safe_gt(self, r: u32) -> bool {
        IsGreater(self as u32, r as u32)
    }
    fn safe_ge(self, r: u32) -> bool {
        IsGreaterOrEqual(self as u32, r as u32)
    }
    fn safe_eq(self, r: u32) -> bool {
        IsEqual(self as u32, r as u32)
    }
    fn safe_ne(self, r: u32) -> bool {
        IsNotEqual(self as u32, r as u32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i64> for u32 {
    fn safe_lt(self, r: i64) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: i64) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: i64) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: i64) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: i64) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: i64) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u64> for u32 {
    fn safe_lt(self, r: u64) -> bool {
        IsLess(self as u64, r as u64)
    }
    fn safe_le(self, r: u64) -> bool {
        IsLessOrEqual(self as u64, r as u64)
    }
    fn safe_gt(self, r: u64) -> bool {
        IsGreater(self as u64, r as u64)
    }
    fn safe_ge(self, r: u64) -> bool {
        IsGreaterOrEqual(self as u64, r as u64)
    }
    fn safe_eq(self, r: u64) -> bool {
        IsEqual(self as u64, r as u64)
    }
    fn safe_ne(self, r: u64) -> bool {
        IsNotEqual(self as u64, r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<f32> for u32 {
    fn safe_lt(self, r: f32) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: f32) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: f32) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: f32) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: f32) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: f32) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<f64> for u32 {
    fn safe_lt(self, r: f64) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: f64) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: f64) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: f64) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: f64) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: f64) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<isize> for u32 {
    fn safe_lt(self, r: isize) -> bool {
        IsLess(self as isize, r as isize)
    }
    fn safe_le(self, r: isize) -> bool {
        IsLessOrEqual(self as isize, r as isize)
    }
    fn safe_gt(self, r: isize) -> bool {
        IsGreater(self as isize, r as isize)
    }
    fn safe_ge(self, r: isize) -> bool {
        IsGreaterOrEqual(self as isize, r as isize)
    }
    fn safe_eq(self, r: isize) -> bool {
        IsEqual(self as isize, r as isize)
    }
    fn safe_ne(self, r: isize) -> bool {
        IsNotEqual(self as isize, r as isize)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<usize> for u32 {
    fn safe_lt(self, r: usize) -> bool {
        IsLess(self as usize, r as usize)
    }
    fn safe_le(self, r: usize) -> bool {
        IsLessOrEqual(self as usize, r as usize)
    }
    fn safe_gt(self, r: usize) -> bool {
        IsGreater(self as usize, r as usize)
    }
    fn safe_ge(self, r: usize) -> bool {
        IsGreaterOrEqual(self as usize, r as usize)
    }
    fn safe_eq(self, r: usize) -> bool {
        IsEqual(self as usize, r as usize)
    }
    fn safe_ne(self, r: usize) -> bool {
        IsNotEqual(self as usize, r as usize)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i8> for i64 {
    fn safe_lt(self, r: i8) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: i8) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: i8) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: i8) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: i8) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: i8) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u8> for i64 {
    fn safe_lt(self, r: u8) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: u8) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: u8) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: u8) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: u8) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: u8) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i16> for i64 {
    fn safe_lt(self, r: i16) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: i16) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: i16) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: i16) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: i16) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: i16) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u16> for i64 {
    fn safe_lt(self, r: u16) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: u16) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: u16) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: u16) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: u16) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: u16) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i32> for i64 {
    fn safe_lt(self, r: i32) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: i32) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: i32) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: i32) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: i32) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: i32) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u32> for i64 {
    fn safe_lt(self, r: u32) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: u32) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: u32) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: u32) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: u32) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: u32) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i64> for i64 {
    fn safe_lt(self, r: i64) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: i64) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: i64) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: i64) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: i64) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: i64) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u64> for i64 {
    fn safe_lt(self, r: u64) -> bool {
        IsLess(self, r)
    }
    fn safe_le(self, r: u64) -> bool {
        IsLessOrEqual(self, r)
    }
    fn safe_gt(self, r: u64) -> bool {
        IsGreater(self, r)
    }
    fn safe_ge(self, r: u64) -> bool {
        IsGreaterOrEqual(self, r)
    }
    fn safe_eq(self, r: u64) -> bool {
        IsEqual(self, r)
    }
    fn safe_ne(self, r: u64) -> bool {
        IsNotEqual(self, r)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<f32> for i64 {
    fn safe_lt(self, r: f32) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: f32) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: f32) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: f32) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: f32) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: f32) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<f64> for i64 {
    fn safe_lt(self, r: f64) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: f64) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: f64) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: f64) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: f64) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: f64) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<isize> for i64 {
    fn safe_lt(self, r: isize) -> bool {
        IsLess(self as isize, r as isize)
    }
    fn safe_le(self, r: isize) -> bool {
        IsLessOrEqual(self as isize, r as isize)
    }
    fn safe_gt(self, r: isize) -> bool {
        IsGreater(self as isize, r as isize)
    }
    fn safe_ge(self, r: isize) -> bool {
        IsGreaterOrEqual(self as isize, r as isize)
    }
    fn safe_eq(self, r: isize) -> bool {
        IsEqual(self as isize, r as isize)
    }
    fn safe_ne(self, r: isize) -> bool {
        IsNotEqual(self as isize, r as isize)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<usize> for i64 {
    fn safe_lt(self, r: usize) -> bool {
        IsLess(self, r)
    }
    fn safe_le(self, r: usize) -> bool {
        IsLessOrEqual(self, r)
    }
    fn safe_gt(self, r: usize) -> bool {
        IsGreater(self, r)
    }
    fn safe_ge(self, r: usize) -> bool {
        IsGreaterOrEqual(self, r)
    }
    fn safe_eq(self, r: usize) -> bool {
        IsEqual(self, r)
    }
    fn safe_ne(self, r: usize) -> bool {
        IsNotEqual(self, r)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i8> for u64 {
    fn safe_lt(self, r: i8) -> bool {
        IsLess(self, r)
    }
    fn safe_le(self, r: i8) -> bool {
        IsLessOrEqual(self, r)
    }
    fn safe_gt(self, r: i8) -> bool {
        IsGreater(self, r)
    }
    fn safe_ge(self, r: i8) -> bool {
        IsGreaterOrEqual(self, r)
    }
    fn safe_eq(self, r: i8) -> bool {
        IsEqual(self, r)
    }
    fn safe_ne(self, r: i8) -> bool {
        IsNotEqual(self, r)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u8> for u64 {
    fn safe_lt(self, r: u8) -> bool {
        IsLess(self as u64, r as u64)
    }
    fn safe_le(self, r: u8) -> bool {
        IsLessOrEqual(self as u64, r as u64)
    }
    fn safe_gt(self, r: u8) -> bool {
        IsGreater(self as u64, r as u64)
    }
    fn safe_ge(self, r: u8) -> bool {
        IsGreaterOrEqual(self as u64, r as u64)
    }
    fn safe_eq(self, r: u8) -> bool {
        IsEqual(self as u64, r as u64)
    }
    fn safe_ne(self, r: u8) -> bool {
        IsNotEqual(self as u64, r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i16> for u64 {
    fn safe_lt(self, r: i16) -> bool {
        IsLess(self, r)
    }
    fn safe_le(self, r: i16) -> bool {
        IsLessOrEqual(self, r)
    }
    fn safe_gt(self, r: i16) -> bool {
        IsGreater(self, r)
    }
    fn safe_ge(self, r: i16) -> bool {
        IsGreaterOrEqual(self, r)
    }
    fn safe_eq(self, r: i16) -> bool {
        IsEqual(self, r)
    }
    fn safe_ne(self, r: i16) -> bool {
        IsNotEqual(self, r)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u16> for u64 {
    fn safe_lt(self, r: u16) -> bool {
        IsLess(self as u64, r as u64)
    }
    fn safe_le(self, r: u16) -> bool {
        IsLessOrEqual(self as u64, r as u64)
    }
    fn safe_gt(self, r: u16) -> bool {
        IsGreater(self as u64, r as u64)
    }
    fn safe_ge(self, r: u16) -> bool {
        IsGreaterOrEqual(self as u64, r as u64)
    }
    fn safe_eq(self, r: u16) -> bool {
        IsEqual(self as u64, r as u64)
    }
    fn safe_ne(self, r: u16) -> bool {
        IsNotEqual(self as u64, r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i32> for u64 {
    fn safe_lt(self, r: i32) -> bool {
        IsLess(self, r)
    }
    fn safe_le(self, r: i32) -> bool {
        IsLessOrEqual(self, r)
    }
    fn safe_gt(self, r: i32) -> bool {
        IsGreater(self, r)
    }
    fn safe_ge(self, r: i32) -> bool {
        IsGreaterOrEqual(self, r)
    }
    fn safe_eq(self, r: i32) -> bool {
        IsEqual(self, r)
    }
    fn safe_ne(self, r: i32) -> bool {
        IsNotEqual(self, r)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u32> for u64 {
    fn safe_lt(self, r: u32) -> bool {
        IsLess(self as u64, r as u64)
    }
    fn safe_le(self, r: u32) -> bool {
        IsLessOrEqual(self as u64, r as u64)
    }
    fn safe_gt(self, r: u32) -> bool {
        IsGreater(self as u64, r as u64)
    }
    fn safe_ge(self, r: u32) -> bool {
        IsGreaterOrEqual(self as u64, r as u64)
    }
    fn safe_eq(self, r: u32) -> bool {
        IsEqual(self as u64, r as u64)
    }
    fn safe_ne(self, r: u32) -> bool {
        IsNotEqual(self as u64, r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i64> for u64 {
    fn safe_lt(self, r: i64) -> bool {
        IsLess(self, r)
    }
    fn safe_le(self, r: i64) -> bool {
        IsLessOrEqual(self, r)
    }
    fn safe_gt(self, r: i64) -> bool {
        IsGreater(self, r)
    }
    fn safe_ge(self, r: i64) -> bool {
        IsGreaterOrEqual(self, r)
    }
    fn safe_eq(self, r: i64) -> bool {
        IsEqual(self, r)
    }
    fn safe_ne(self, r: i64) -> bool {
        IsNotEqual(self, r)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u64> for u64 {
    fn safe_lt(self, r: u64) -> bool {
        IsLess(self as u64, r as u64)
    }
    fn safe_le(self, r: u64) -> bool {
        IsLessOrEqual(self as u64, r as u64)
    }
    fn safe_gt(self, r: u64) -> bool {
        IsGreater(self as u64, r as u64)
    }
    fn safe_ge(self, r: u64) -> bool {
        IsGreaterOrEqual(self as u64, r as u64)
    }
    fn safe_eq(self, r: u64) -> bool {
        IsEqual(self as u64, r as u64)
    }
    fn safe_ne(self, r: u64) -> bool {
        IsNotEqual(self as u64, r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<f32> for u64 {
    fn safe_lt(self, r: f32) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: f32) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: f32) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: f32) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: f32) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: f32) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<f64> for u64 {
    fn safe_lt(self, r: f64) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: f64) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: f64) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: f64) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: f64) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: f64) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<isize> for u64 {
    fn safe_lt(self, r: isize) -> bool {
        IsLess(self, r)
    }
    fn safe_le(self, r: isize) -> bool {
        IsLessOrEqual(self, r)
    }
    fn safe_gt(self, r: isize) -> bool {
        IsGreater(self, r)
    }
    fn safe_ge(self, r: isize) -> bool {
        IsGreaterOrEqual(self, r)
    }
    fn safe_eq(self, r: isize) -> bool {
        IsEqual(self, r)
    }
    fn safe_ne(self, r: isize) -> bool {
        IsNotEqual(self, r)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<usize> for u64 {
    fn safe_lt(self, r: usize) -> bool {
        IsLess(self as usize, r as usize)
    }
    fn safe_le(self, r: usize) -> bool {
        IsLessOrEqual(self as usize, r as usize)
    }
    fn safe_gt(self, r: usize) -> bool {
        IsGreater(self as usize, r as usize)
    }
    fn safe_ge(self, r: usize) -> bool {
        IsGreaterOrEqual(self as usize, r as usize)
    }
    fn safe_eq(self, r: usize) -> bool {
        IsEqual(self as usize, r as usize)
    }
    fn safe_ne(self, r: usize) -> bool {
        IsNotEqual(self as usize, r as usize)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i8> for f32 {
    fn safe_lt(self, r: i8) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: i8) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: i8) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: i8) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: i8) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: i8) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u8> for f32 {
    fn safe_lt(self, r: u8) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: u8) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: u8) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: u8) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: u8) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: u8) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i16> for f32 {
    fn safe_lt(self, r: i16) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: i16) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: i16) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: i16) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: i16) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: i16) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u16> for f32 {
    fn safe_lt(self, r: u16) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: u16) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: u16) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: u16) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: u16) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: u16) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i32> for f32 {
    fn safe_lt(self, r: i32) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: i32) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: i32) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: i32) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: i32) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: i32) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u32> for f32 {
    fn safe_lt(self, r: u32) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: u32) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: u32) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: u32) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: u32) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: u32) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i64> for f32 {
    fn safe_lt(self, r: i64) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: i64) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: i64) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: i64) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: i64) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: i64) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u64> for f32 {
    fn safe_lt(self, r: u64) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: u64) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: u64) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: u64) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: u64) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: u64) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<f32> for f32 {
    fn safe_lt(self, r: f32) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: f32) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: f32) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: f32) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: f32) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: f32) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<f64> for f32 {
    fn safe_lt(self, r: f64) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: f64) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: f64) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: f64) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: f64) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: f64) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<isize> for f32 {
    fn safe_lt(self, r: isize) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: isize) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: isize) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: isize) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: isize) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: isize) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<usize> for f32 {
    fn safe_lt(self, r: usize) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: usize) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: usize) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: usize) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: usize) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: usize) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i8> for f64 {
    fn safe_lt(self, r: i8) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: i8) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: i8) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: i8) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: i8) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: i8) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u8> for f64 {
    fn safe_lt(self, r: u8) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: u8) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: u8) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: u8) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: u8) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: u8) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i16> for f64 {
    fn safe_lt(self, r: i16) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: i16) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: i16) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: i16) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: i16) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: i16) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u16> for f64 {
    fn safe_lt(self, r: u16) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: u16) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: u16) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: u16) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: u16) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: u16) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i32> for f64 {
    fn safe_lt(self, r: i32) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: i32) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: i32) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: i32) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: i32) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: i32) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u32> for f64 {
    fn safe_lt(self, r: u32) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: u32) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: u32) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: u32) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: u32) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: u32) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i64> for f64 {
    fn safe_lt(self, r: i64) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: i64) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: i64) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: i64) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: i64) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: i64) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u64> for f64 {
    fn safe_lt(self, r: u64) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: u64) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: u64) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: u64) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: u64) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: u64) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<f32> for f64 {
    fn safe_lt(self, r: f32) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: f32) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: f32) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: f32) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: f32) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: f32) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<f64> for f64 {
    fn safe_lt(self, r: f64) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: f64) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: f64) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: f64) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: f64) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: f64) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<isize> for f64 {
    fn safe_lt(self, r: isize) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: isize) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: isize) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: isize) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: isize) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: isize) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<usize> for f64 {
    fn safe_lt(self, r: usize) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: usize) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: usize) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: usize) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: usize) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: usize) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i8> for isize {
    fn safe_lt(self, r: i8) -> bool {
        IsLess(self as isize, r as isize)
    }
    fn safe_le(self, r: i8) -> bool {
        IsLessOrEqual(self as isize, r as isize)
    }
    fn safe_gt(self, r: i8) -> bool {
        IsGreater(self as isize, r as isize)
    }
    fn safe_ge(self, r: i8) -> bool {
        IsGreaterOrEqual(self as isize, r as isize)
    }
    fn safe_eq(self, r: i8) -> bool {
        IsEqual(self as isize, r as isize)
    }
    fn safe_ne(self, r: i8) -> bool {
        IsNotEqual(self as isize, r as isize)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u8> for isize {
    fn safe_lt(self, r: u8) -> bool {
        IsLess(self as isize, r as isize)
    }
    fn safe_le(self, r: u8) -> bool {
        IsLessOrEqual(self as isize, r as isize)
    }
    fn safe_gt(self, r: u8) -> bool {
        IsGreater(self as isize, r as isize)
    }
    fn safe_ge(self, r: u8) -> bool {
        IsGreaterOrEqual(self as isize, r as isize)
    }
    fn safe_eq(self, r: u8) -> bool {
        IsEqual(self as isize, r as isize)
    }
    fn safe_ne(self, r: u8) -> bool {
        IsNotEqual(self as isize, r as isize)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i16> for isize {
    fn safe_lt(self, r: i16) -> bool {
        IsLess(self as isize, r as isize)
    }
    fn safe_le(self, r: i16) -> bool {
        IsLessOrEqual(self as isize, r as isize)
    }
    fn safe_gt(self, r: i16) -> bool {
        IsGreater(self as isize, r as isize)
    }
    fn safe_ge(self, r: i16) -> bool {
        IsGreaterOrEqual(self as isize, r as isize)
    }
    fn safe_eq(self, r: i16) -> bool {
        IsEqual(self as isize, r as isize)
    }
    fn safe_ne(self, r: i16) -> bool {
        IsNotEqual(self as isize, r as isize)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u16> for isize {
    fn safe_lt(self, r: u16) -> bool {
        IsLess(self as isize, r as isize)
    }
    fn safe_le(self, r: u16) -> bool {
        IsLessOrEqual(self as isize, r as isize)
    }
    fn safe_gt(self, r: u16) -> bool {
        IsGreater(self as isize, r as isize)
    }
    fn safe_ge(self, r: u16) -> bool {
        IsGreaterOrEqual(self as isize, r as isize)
    }
    fn safe_eq(self, r: u16) -> bool {
        IsEqual(self as isize, r as isize)
    }
    fn safe_ne(self, r: u16) -> bool {
        IsNotEqual(self as isize, r as isize)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i32> for isize {
    fn safe_lt(self, r: i32) -> bool {
        IsLess(self as isize, r as isize)
    }
    fn safe_le(self, r: i32) -> bool {
        IsLessOrEqual(self as isize, r as isize)
    }
    fn safe_gt(self, r: i32) -> bool {
        IsGreater(self as isize, r as isize)
    }
    fn safe_ge(self, r: i32) -> bool {
        IsGreaterOrEqual(self as isize, r as isize)
    }
    fn safe_eq(self, r: i32) -> bool {
        IsEqual(self as isize, r as isize)
    }
    fn safe_ne(self, r: i32) -> bool {
        IsNotEqual(self as isize, r as isize)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u32> for isize {
    fn safe_lt(self, r: u32) -> bool {
        IsLess(self as isize, r as isize)
    }
    fn safe_le(self, r: u32) -> bool {
        IsLessOrEqual(self as isize, r as isize)
    }
    fn safe_gt(self, r: u32) -> bool {
        IsGreater(self as isize, r as isize)
    }
    fn safe_ge(self, r: u32) -> bool {
        IsGreaterOrEqual(self as isize, r as isize)
    }
    fn safe_eq(self, r: u32) -> bool {
        IsEqual(self as isize, r as isize)
    }
    fn safe_ne(self, r: u32) -> bool {
        IsNotEqual(self as isize, r as isize)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i64> for isize {
    fn safe_lt(self, r: i64) -> bool {
        IsLess(self as i64, r as i64)
    }
    fn safe_le(self, r: i64) -> bool {
        IsLessOrEqual(self as i64, r as i64)
    }
    fn safe_gt(self, r: i64) -> bool {
        IsGreater(self as i64, r as i64)
    }
    fn safe_ge(self, r: i64) -> bool {
        IsGreaterOrEqual(self as i64, r as i64)
    }
    fn safe_eq(self, r: i64) -> bool {
        IsEqual(self as i64, r as i64)
    }
    fn safe_ne(self, r: i64) -> bool {
        IsNotEqual(self as i64, r as i64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u64> for isize {
    fn safe_lt(self, r: u64) -> bool {
        IsLess(self, r)
    }
    fn safe_le(self, r: u64) -> bool {
        IsLessOrEqual(self, r)
    }
    fn safe_gt(self, r: u64) -> bool {
        IsGreater(self, r)
    }
    fn safe_ge(self, r: u64) -> bool {
        IsGreaterOrEqual(self, r)
    }
    fn safe_eq(self, r: u64) -> bool {
        IsEqual(self, r)
    }
    fn safe_ne(self, r: u64) -> bool {
        IsNotEqual(self, r)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<f32> for isize {
    fn safe_lt(self, r: f32) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: f32) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: f32) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: f32) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: f32) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: f32) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<f64> for isize {
    fn safe_lt(self, r: f64) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: f64) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: f64) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: f64) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: f64) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: f64) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<isize> for isize {
    fn safe_lt(self, r: isize) -> bool {
        IsLess(self as isize, r as isize)
    }
    fn safe_le(self, r: isize) -> bool {
        IsLessOrEqual(self as isize, r as isize)
    }
    fn safe_gt(self, r: isize) -> bool {
        IsGreater(self as isize, r as isize)
    }
    fn safe_ge(self, r: isize) -> bool {
        IsGreaterOrEqual(self as isize, r as isize)
    }
    fn safe_eq(self, r: isize) -> bool {
        IsEqual(self as isize, r as isize)
    }
    fn safe_ne(self, r: isize) -> bool {
        IsNotEqual(self as isize, r as isize)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<usize> for isize {
    fn safe_lt(self, r: usize) -> bool {
        IsLess(self, r)
    }
    fn safe_le(self, r: usize) -> bool {
        IsLessOrEqual(self, r)
    }
    fn safe_gt(self, r: usize) -> bool {
        IsGreater(self, r)
    }
    fn safe_ge(self, r: usize) -> bool {
        IsGreaterOrEqual(self, r)
    }
    fn safe_eq(self, r: usize) -> bool {
        IsEqual(self, r)
    }
    fn safe_ne(self, r: usize) -> bool {
        IsNotEqual(self, r)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i8> for usize {
    fn safe_lt(self, r: i8) -> bool {
        IsLess(self, r)
    }
    fn safe_le(self, r: i8) -> bool {
        IsLessOrEqual(self, r)
    }
    fn safe_gt(self, r: i8) -> bool {
        IsGreater(self, r)
    }
    fn safe_ge(self, r: i8) -> bool {
        IsGreaterOrEqual(self, r)
    }
    fn safe_eq(self, r: i8) -> bool {
        IsEqual(self, r)
    }
    fn safe_ne(self, r: i8) -> bool {
        IsNotEqual(self, r)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u8> for usize {
    fn safe_lt(self, r: u8) -> bool {
        IsLess(self as usize, r as usize)
    }
    fn safe_le(self, r: u8) -> bool {
        IsLessOrEqual(self as usize, r as usize)
    }
    fn safe_gt(self, r: u8) -> bool {
        IsGreater(self as usize, r as usize)
    }
    fn safe_ge(self, r: u8) -> bool {
        IsGreaterOrEqual(self as usize, r as usize)
    }
    fn safe_eq(self, r: u8) -> bool {
        IsEqual(self as usize, r as usize)
    }
    fn safe_ne(self, r: u8) -> bool {
        IsNotEqual(self as usize, r as usize)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i16> for usize {
    fn safe_lt(self, r: i16) -> bool {
        IsLess(self, r)
    }
    fn safe_le(self, r: i16) -> bool {
        IsLessOrEqual(self, r)
    }
    fn safe_gt(self, r: i16) -> bool {
        IsGreater(self, r)
    }
    fn safe_ge(self, r: i16) -> bool {
        IsGreaterOrEqual(self, r)
    }
    fn safe_eq(self, r: i16) -> bool {
        IsEqual(self, r)
    }
    fn safe_ne(self, r: i16) -> bool {
        IsNotEqual(self, r)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u16> for usize {
    fn safe_lt(self, r: u16) -> bool {
        IsLess(self as usize, r as usize)
    }
    fn safe_le(self, r: u16) -> bool {
        IsLessOrEqual(self as usize, r as usize)
    }
    fn safe_gt(self, r: u16) -> bool {
        IsGreater(self as usize, r as usize)
    }
    fn safe_ge(self, r: u16) -> bool {
        IsGreaterOrEqual(self as usize, r as usize)
    }
    fn safe_eq(self, r: u16) -> bool {
        IsEqual(self as usize, r as usize)
    }
    fn safe_ne(self, r: u16) -> bool {
        IsNotEqual(self as usize, r as usize)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i32> for usize {
    fn safe_lt(self, r: i32) -> bool {
        IsLess(self, r)
    }
    fn safe_le(self, r: i32) -> bool {
        IsLessOrEqual(self, r)
    }
    fn safe_gt(self, r: i32) -> bool {
        IsGreater(self, r)
    }
    fn safe_ge(self, r: i32) -> bool {
        IsGreaterOrEqual(self, r)
    }
    fn safe_eq(self, r: i32) -> bool {
        IsEqual(self, r)
    }
    fn safe_ne(self, r: i32) -> bool {
        IsNotEqual(self, r)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u32> for usize {
    fn safe_lt(self, r: u32) -> bool {
        IsLess(self as usize, r as usize)
    }
    fn safe_le(self, r: u32) -> bool {
        IsLessOrEqual(self as usize, r as usize)
    }
    fn safe_gt(self, r: u32) -> bool {
        IsGreater(self as usize, r as usize)
    }
    fn safe_ge(self, r: u32) -> bool {
        IsGreaterOrEqual(self as usize, r as usize)
    }
    fn safe_eq(self, r: u32) -> bool {
        IsEqual(self as usize, r as usize)
    }
    fn safe_ne(self, r: u32) -> bool {
        IsNotEqual(self as usize, r as usize)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<i64> for usize {
    fn safe_lt(self, r: i64) -> bool {
        IsLess(self, r)
    }
    fn safe_le(self, r: i64) -> bool {
        IsLessOrEqual(self, r)
    }
    fn safe_gt(self, r: i64) -> bool {
        IsGreater(self, r)
    }
    fn safe_ge(self, r: i64) -> bool {
        IsGreaterOrEqual(self, r)
    }
    fn safe_eq(self, r: i64) -> bool {
        IsEqual(self, r)
    }
    fn safe_ne(self, r: i64) -> bool {
        IsNotEqual(self, r)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<u64> for usize {
    fn safe_lt(self, r: u64) -> bool {
        IsLess(self as u64, r as u64)
    }
    fn safe_le(self, r: u64) -> bool {
        IsLessOrEqual(self as u64, r as u64)
    }
    fn safe_gt(self, r: u64) -> bool {
        IsGreater(self as u64, r as u64)
    }
    fn safe_ge(self, r: u64) -> bool {
        IsGreaterOrEqual(self as u64, r as u64)
    }
    fn safe_eq(self, r: u64) -> bool {
        IsEqual(self as u64, r as u64)
    }
    fn safe_ne(self, r: u64) -> bool {
        IsNotEqual(self as u64, r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<f32> for usize {
    fn safe_lt(self, r: f32) -> bool {
        IsLess(self as f32, r as f32)
    }
    fn safe_le(self, r: f32) -> bool {
        IsLessOrEqual(self as f32, r as f32)
    }
    fn safe_gt(self, r: f32) -> bool {
        IsGreater(self as f32, r as f32)
    }
    fn safe_ge(self, r: f32) -> bool {
        IsGreaterOrEqual(self as f32, r as f32)
    }
    fn safe_eq(self, r: f32) -> bool {
        IsEqual(self as f32, r as f32)
    }
    fn safe_ne(self, r: f32) -> bool {
        IsNotEqual(self as f32, r as f32)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<f64> for usize {
    fn safe_lt(self, r: f64) -> bool {
        IsLess(self as f64, r as f64)
    }
    fn safe_le(self, r: f64) -> bool {
        IsLessOrEqual(self as f64, r as f64)
    }
    fn safe_gt(self, r: f64) -> bool {
        IsGreater(self as f64, r as f64)
    }
    fn safe_ge(self, r: f64) -> bool {
        IsGreaterOrEqual(self as f64, r as f64)
    }
    fn safe_eq(self, r: f64) -> bool {
        IsEqual(self as f64, r as f64)
    }
    fn safe_ne(self, r: f64) -> bool {
        IsNotEqual(self as f64, r as f64)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<isize> for usize {
    fn safe_lt(self, r: isize) -> bool {
        IsLess(self, r)
    }
    fn safe_le(self, r: isize) -> bool {
        IsLessOrEqual(self, r)
    }
    fn safe_gt(self, r: isize) -> bool {
        IsGreater(self, r)
    }
    fn safe_ge(self, r: isize) -> bool {
        IsGreaterOrEqual(self, r)
    }
    fn safe_eq(self, r: isize) -> bool {
        IsEqual(self, r)
    }
    fn safe_ne(self, r: isize) -> bool {
        IsNotEqual(self, r)
    }
}
#[cfg(target_pointer_width = "64")]
impl SafeComparison<usize> for usize {
    fn safe_lt(self, r: usize) -> bool {
        IsLess(self as usize, r as usize)
    }
    fn safe_le(self, r: usize) -> bool {
        IsLessOrEqual(self as usize, r as usize)
    }
    fn safe_gt(self, r: usize) -> bool {
        IsGreater(self as usize, r as usize)
    }
    fn safe_ge(self, r: usize) -> bool {
        IsGreaterOrEqual(self as usize, r as usize)
    }
    fn safe_eq(self, r: usize) -> bool {
        IsEqual(self as usize, r as usize)
    }
    fn safe_ne(self, r: usize) -> bool {
        IsNotEqual(self as usize, r as usize)
    }
}

// Source: safe_conversions_impl.h:199-354. Actual customizable Bounds range specializations.
pub trait RangeWith<D: Numeric>: Cast<D> {
    fn range_with(self, low: D, high: D) -> RangeCheck;
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i8> for i8 {
    fn range_with(self, low: i8, high: i8) -> RangeCheck {
        RangeCheck::new(
            (i8::MIN as i8) >= low || (self as i8) >= low,
            (i8::MAX as i8) <= high || (self as i8) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u8> for i8 {
    fn range_with(self, low: u8, high: u8) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u8 || (self as u8) >= low),
            (i8::MAX as i32) <= (high as i32) || (self as i32) <= (high as i32),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i16> for i8 {
    fn range_with(self, low: i16, high: i16) -> RangeCheck {
        RangeCheck::new(
            (i8::MIN as i16) >= low || (self as i16) >= low,
            (i8::MAX as i16) <= high || (self as i16) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u16> for i8 {
    fn range_with(self, low: u16, high: u16) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u16 || (self as u16) >= low),
            (i8::MAX as i32) <= (high as i32) || (self as i32) <= (high as i32),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i32> for i8 {
    fn range_with(self, low: i32, high: i32) -> RangeCheck {
        RangeCheck::new(
            (i8::MIN as i32) >= low || (self as i32) >= low,
            (i8::MAX as i32) <= high || (self as i32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u32> for i8 {
    fn range_with(self, low: u32, high: u32) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u32 || (self as u32) >= low),
            (i8::MAX as u32) <= (high as u32) || (self as u32) <= (high as u32),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i64> for i8 {
    fn range_with(self, low: i64, high: i64) -> RangeCheck {
        RangeCheck::new(
            (i8::MIN as i64) >= low || (self as i64) >= low,
            (i8::MAX as i64) <= high || (self as i64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u64> for i8 {
    fn range_with(self, low: u64, high: u64) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u64 || (self as u64) >= low),
            (i8::MAX as u64) <= (high as u64) || (self as u64) <= (high as u64),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<f32> for i8 {
    fn range_with(self, low: f32, high: f32) -> RangeCheck {
        RangeCheck::new(
            (i8::MIN as f32) >= low || (self as f32) >= low,
            (i8::MAX as f32) <= high || (self as f32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<f64> for i8 {
    fn range_with(self, low: f64, high: f64) -> RangeCheck {
        RangeCheck::new(
            (i8::MIN as f64) >= low || (self as f64) >= low,
            (i8::MAX as f64) <= high || (self as f64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<isize> for i8 {
    fn range_with(self, low: isize, high: isize) -> RangeCheck {
        RangeCheck::new(
            (i8::MIN as isize) >= low || (self as isize) >= low,
            (i8::MAX as isize) <= high || (self as isize) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<usize> for i8 {
    fn range_with(self, low: usize, high: usize) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as usize || (self as usize) >= low),
            (i8::MAX as usize) <= (high as usize) || (self as usize) <= (high as usize),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i8> for u8 {
    fn range_with(self, low: i8, high: i8) -> RangeCheck {
        RangeCheck::new(
            low <= 0 as i8 || (self as i32) >= (low as i32),
            (self as i32) <= (high as i32),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u8> for u8 {
    fn range_with(self, low: u8, high: u8) -> RangeCheck {
        RangeCheck::new(
            (u8::MIN as u8) >= low || (self as u8) >= low,
            (u8::MAX as u8) <= high || (self as u8) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i16> for u8 {
    fn range_with(self, low: i16, high: i16) -> RangeCheck {
        RangeCheck::new(
            (u8::MIN as i16) >= low || (self as i16) >= low,
            (u8::MAX as i16) <= high || (self as i16) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u16> for u8 {
    fn range_with(self, low: u16, high: u16) -> RangeCheck {
        RangeCheck::new(
            (u8::MIN as u16) >= low || (self as u16) >= low,
            (u8::MAX as u16) <= high || (self as u16) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i32> for u8 {
    fn range_with(self, low: i32, high: i32) -> RangeCheck {
        RangeCheck::new(
            (u8::MIN as i32) >= low || (self as i32) >= low,
            (u8::MAX as i32) <= high || (self as i32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u32> for u8 {
    fn range_with(self, low: u32, high: u32) -> RangeCheck {
        RangeCheck::new(
            (u8::MIN as u32) >= low || (self as u32) >= low,
            (u8::MAX as u32) <= high || (self as u32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i64> for u8 {
    fn range_with(self, low: i64, high: i64) -> RangeCheck {
        RangeCheck::new(
            (u8::MIN as i64) >= low || (self as i64) >= low,
            (u8::MAX as i64) <= high || (self as i64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u64> for u8 {
    fn range_with(self, low: u64, high: u64) -> RangeCheck {
        RangeCheck::new(
            (u8::MIN as u64) >= low || (self as u64) >= low,
            (u8::MAX as u64) <= high || (self as u64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<f32> for u8 {
    fn range_with(self, low: f32, high: f32) -> RangeCheck {
        RangeCheck::new(
            (u8::MIN as f32) >= low || (self as f32) >= low,
            (u8::MAX as f32) <= high || (self as f32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<f64> for u8 {
    fn range_with(self, low: f64, high: f64) -> RangeCheck {
        RangeCheck::new(
            (u8::MIN as f64) >= low || (self as f64) >= low,
            (u8::MAX as f64) <= high || (self as f64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<isize> for u8 {
    fn range_with(self, low: isize, high: isize) -> RangeCheck {
        RangeCheck::new(
            (u8::MIN as isize) >= low || (self as isize) >= low,
            (u8::MAX as isize) <= high || (self as isize) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<usize> for u8 {
    fn range_with(self, low: usize, high: usize) -> RangeCheck {
        RangeCheck::new(
            (u8::MIN as usize) >= low || (self as usize) >= low,
            (u8::MAX as usize) <= high || (self as usize) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i8> for i16 {
    fn range_with(self, low: i8, high: i8) -> RangeCheck {
        RangeCheck::new(self >= (low as i16), self <= (high as i16))
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u8> for i16 {
    fn range_with(self, low: u8, high: u8) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u8 || (self as u8) >= low),
            (i16::MAX as i32) <= (high as i32) || (self as i32) <= (high as i32),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i16> for i16 {
    fn range_with(self, low: i16, high: i16) -> RangeCheck {
        RangeCheck::new(
            (i16::MIN as i16) >= low || (self as i16) >= low,
            (i16::MAX as i16) <= high || (self as i16) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u16> for i16 {
    fn range_with(self, low: u16, high: u16) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u16 || (self as u16) >= low),
            (i16::MAX as i32) <= (high as i32) || (self as i32) <= (high as i32),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i32> for i16 {
    fn range_with(self, low: i32, high: i32) -> RangeCheck {
        RangeCheck::new(
            (i16::MIN as i32) >= low || (self as i32) >= low,
            (i16::MAX as i32) <= high || (self as i32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u32> for i16 {
    fn range_with(self, low: u32, high: u32) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u32 || (self as u32) >= low),
            (i16::MAX as u32) <= (high as u32) || (self as u32) <= (high as u32),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i64> for i16 {
    fn range_with(self, low: i64, high: i64) -> RangeCheck {
        RangeCheck::new(
            (i16::MIN as i64) >= low || (self as i64) >= low,
            (i16::MAX as i64) <= high || (self as i64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u64> for i16 {
    fn range_with(self, low: u64, high: u64) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u64 || (self as u64) >= low),
            (i16::MAX as u64) <= (high as u64) || (self as u64) <= (high as u64),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<f32> for i16 {
    fn range_with(self, low: f32, high: f32) -> RangeCheck {
        RangeCheck::new(
            (i16::MIN as f32) >= low || (self as f32) >= low,
            (i16::MAX as f32) <= high || (self as f32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<f64> for i16 {
    fn range_with(self, low: f64, high: f64) -> RangeCheck {
        RangeCheck::new(
            (i16::MIN as f64) >= low || (self as f64) >= low,
            (i16::MAX as f64) <= high || (self as f64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<isize> for i16 {
    fn range_with(self, low: isize, high: isize) -> RangeCheck {
        RangeCheck::new(
            (i16::MIN as isize) >= low || (self as isize) >= low,
            (i16::MAX as isize) <= high || (self as isize) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<usize> for i16 {
    fn range_with(self, low: usize, high: usize) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as usize || (self as usize) >= low),
            (i16::MAX as usize) <= (high as usize) || (self as usize) <= (high as usize),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i8> for u16 {
    fn range_with(self, low: i8, high: i8) -> RangeCheck {
        RangeCheck::new(
            low <= 0 as i8 || (self as i32) >= (low as i32),
            (self as i32) <= (high as i32),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u8> for u16 {
    fn range_with(self, low: u8, high: u8) -> RangeCheck {
        RangeCheck::new(
            low == 0 as u8 || self >= (low as u16),
            self <= (high as u16),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i16> for u16 {
    fn range_with(self, low: i16, high: i16) -> RangeCheck {
        RangeCheck::new(
            low <= 0 as i16 || (self as i32) >= (low as i32),
            (self as i32) <= (high as i32),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u16> for u16 {
    fn range_with(self, low: u16, high: u16) -> RangeCheck {
        RangeCheck::new(
            (u16::MIN as u16) >= low || (self as u16) >= low,
            (u16::MAX as u16) <= high || (self as u16) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i32> for u16 {
    fn range_with(self, low: i32, high: i32) -> RangeCheck {
        RangeCheck::new(
            (u16::MIN as i32) >= low || (self as i32) >= low,
            (u16::MAX as i32) <= high || (self as i32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u32> for u16 {
    fn range_with(self, low: u32, high: u32) -> RangeCheck {
        RangeCheck::new(
            (u16::MIN as u32) >= low || (self as u32) >= low,
            (u16::MAX as u32) <= high || (self as u32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i64> for u16 {
    fn range_with(self, low: i64, high: i64) -> RangeCheck {
        RangeCheck::new(
            (u16::MIN as i64) >= low || (self as i64) >= low,
            (u16::MAX as i64) <= high || (self as i64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u64> for u16 {
    fn range_with(self, low: u64, high: u64) -> RangeCheck {
        RangeCheck::new(
            (u16::MIN as u64) >= low || (self as u64) >= low,
            (u16::MAX as u64) <= high || (self as u64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<f32> for u16 {
    fn range_with(self, low: f32, high: f32) -> RangeCheck {
        RangeCheck::new(
            (u16::MIN as f32) >= low || (self as f32) >= low,
            (u16::MAX as f32) <= high || (self as f32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<f64> for u16 {
    fn range_with(self, low: f64, high: f64) -> RangeCheck {
        RangeCheck::new(
            (u16::MIN as f64) >= low || (self as f64) >= low,
            (u16::MAX as f64) <= high || (self as f64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<isize> for u16 {
    fn range_with(self, low: isize, high: isize) -> RangeCheck {
        RangeCheck::new(
            (u16::MIN as isize) >= low || (self as isize) >= low,
            (u16::MAX as isize) <= high || (self as isize) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<usize> for u16 {
    fn range_with(self, low: usize, high: usize) -> RangeCheck {
        RangeCheck::new(
            (u16::MIN as usize) >= low || (self as usize) >= low,
            (u16::MAX as usize) <= high || (self as usize) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i8> for i32 {
    fn range_with(self, low: i8, high: i8) -> RangeCheck {
        RangeCheck::new(self >= (low as i32), self <= (high as i32))
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u8> for i32 {
    fn range_with(self, low: u8, high: u8) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u8 || (self as u8) >= low),
            (i32::MAX as i32) <= (high as i32) || (self as i32) <= (high as i32),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i16> for i32 {
    fn range_with(self, low: i16, high: i16) -> RangeCheck {
        RangeCheck::new(self >= (low as i32), self <= (high as i32))
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u16> for i32 {
    fn range_with(self, low: u16, high: u16) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u16 || (self as u16) >= low),
            (i32::MAX as i32) <= (high as i32) || (self as i32) <= (high as i32),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i32> for i32 {
    fn range_with(self, low: i32, high: i32) -> RangeCheck {
        RangeCheck::new(
            (i32::MIN as i32) >= low || (self as i32) >= low,
            (i32::MAX as i32) <= high || (self as i32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u32> for i32 {
    fn range_with(self, low: u32, high: u32) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u32 || (self as u32) >= low),
            (i32::MAX as u32) <= (high as u32) || (self as u32) <= (high as u32),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i64> for i32 {
    fn range_with(self, low: i64, high: i64) -> RangeCheck {
        RangeCheck::new(
            (i32::MIN as i64) >= low || (self as i64) >= low,
            (i32::MAX as i64) <= high || (self as i64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u64> for i32 {
    fn range_with(self, low: u64, high: u64) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u64 || (self as u64) >= low),
            (i32::MAX as u64) <= (high as u64) || (self as u64) <= (high as u64),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<f32> for i32 {
    fn range_with(self, low: f32, high: f32) -> RangeCheck {
        RangeCheck::new(
            (i32::MIN as f32) >= low || (self as f32) >= low,
            (i32::MAX as f32) <= high || (self as f32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<f64> for i32 {
    fn range_with(self, low: f64, high: f64) -> RangeCheck {
        RangeCheck::new(
            (i32::MIN as f64) >= low || (self as f64) >= low,
            (i32::MAX as f64) <= high || (self as f64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<isize> for i32 {
    fn range_with(self, low: isize, high: isize) -> RangeCheck {
        RangeCheck::new(
            (i32::MIN as isize) >= low || (self as isize) >= low,
            (i32::MAX as isize) <= high || (self as isize) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<usize> for i32 {
    fn range_with(self, low: usize, high: usize) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as usize || (self as usize) >= low),
            (i32::MAX as usize) <= (high as usize) || (self as usize) <= (high as usize),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i8> for u32 {
    fn range_with(self, low: i8, high: i8) -> RangeCheck {
        RangeCheck::new(
            low <= 0 as i8 || (self as u32) >= (low as u32),
            (self as u32) <= (high as u32),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u8> for u32 {
    fn range_with(self, low: u8, high: u8) -> RangeCheck {
        RangeCheck::new(
            low == 0 as u8 || self >= (low as u32),
            self <= (high as u32),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i16> for u32 {
    fn range_with(self, low: i16, high: i16) -> RangeCheck {
        RangeCheck::new(
            low <= 0 as i16 || (self as u32) >= (low as u32),
            (self as u32) <= (high as u32),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u16> for u32 {
    fn range_with(self, low: u16, high: u16) -> RangeCheck {
        RangeCheck::new(
            low == 0 as u16 || self >= (low as u32),
            self <= (high as u32),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i32> for u32 {
    fn range_with(self, low: i32, high: i32) -> RangeCheck {
        RangeCheck::new(
            low <= 0 as i32 || (self as u32) >= (low as u32),
            (self as u32) <= (high as u32),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u32> for u32 {
    fn range_with(self, low: u32, high: u32) -> RangeCheck {
        RangeCheck::new(
            (u32::MIN as u32) >= low || (self as u32) >= low,
            (u32::MAX as u32) <= high || (self as u32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i64> for u32 {
    fn range_with(self, low: i64, high: i64) -> RangeCheck {
        RangeCheck::new(
            (u32::MIN as i64) >= low || (self as i64) >= low,
            (u32::MAX as i64) <= high || (self as i64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u64> for u32 {
    fn range_with(self, low: u64, high: u64) -> RangeCheck {
        RangeCheck::new(
            (u32::MIN as u64) >= low || (self as u64) >= low,
            (u32::MAX as u64) <= high || (self as u64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<f32> for u32 {
    fn range_with(self, low: f32, high: f32) -> RangeCheck {
        RangeCheck::new(
            (u32::MIN as f32) >= low || (self as f32) >= low,
            (u32::MAX as f32) <= high || (self as f32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<f64> for u32 {
    fn range_with(self, low: f64, high: f64) -> RangeCheck {
        RangeCheck::new(
            (u32::MIN as f64) >= low || (self as f64) >= low,
            (u32::MAX as f64) <= high || (self as f64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<isize> for u32 {
    fn range_with(self, low: isize, high: isize) -> RangeCheck {
        RangeCheck::new(
            low <= 0 as isize || (self as u32) >= (low as u32),
            (self as u32) <= (high as u32),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<usize> for u32 {
    fn range_with(self, low: usize, high: usize) -> RangeCheck {
        RangeCheck::new(
            (u32::MIN as usize) >= low || (self as usize) >= low,
            (u32::MAX as usize) <= high || (self as usize) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i8> for i64 {
    fn range_with(self, low: i8, high: i8) -> RangeCheck {
        RangeCheck::new(self >= (low as i64), self <= (high as i64))
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u8> for i64 {
    fn range_with(self, low: u8, high: u8) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u8 || (self as u8) >= low),
            (i64::MAX as i64) <= (high as i64) || (self as i64) <= (high as i64),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i16> for i64 {
    fn range_with(self, low: i16, high: i16) -> RangeCheck {
        RangeCheck::new(self >= (low as i64), self <= (high as i64))
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u16> for i64 {
    fn range_with(self, low: u16, high: u16) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u16 || (self as u16) >= low),
            (i64::MAX as i64) <= (high as i64) || (self as i64) <= (high as i64),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i32> for i64 {
    fn range_with(self, low: i32, high: i32) -> RangeCheck {
        RangeCheck::new(self >= (low as i64), self <= (high as i64))
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u32> for i64 {
    fn range_with(self, low: u32, high: u32) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u32 || (self as u32) >= low),
            (i64::MAX as i64) <= (high as i64) || (self as i64) <= (high as i64),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i64> for i64 {
    fn range_with(self, low: i64, high: i64) -> RangeCheck {
        RangeCheck::new(
            (i64::MIN as i64) >= low || (self as i64) >= low,
            (i64::MAX as i64) <= high || (self as i64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u64> for i64 {
    fn range_with(self, low: u64, high: u64) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u64 || (self as u64) >= low),
            (i64::MAX as u64) <= (high as u64) || (self as u64) <= (high as u64),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<f32> for i64 {
    fn range_with(self, low: f32, high: f32) -> RangeCheck {
        RangeCheck::new(
            (i64::MIN as f32) >= low || (self as f32) >= low,
            (i64::MAX as f32) <= high || (self as f32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<f64> for i64 {
    fn range_with(self, low: f64, high: f64) -> RangeCheck {
        RangeCheck::new(
            (i64::MIN as f64) >= low || (self as f64) >= low,
            (i64::MAX as f64) <= high || (self as f64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<isize> for i64 {
    fn range_with(self, low: isize, high: isize) -> RangeCheck {
        RangeCheck::new(self >= (low as i64), self <= (high as i64))
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<usize> for i64 {
    fn range_with(self, low: usize, high: usize) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as usize || (self as usize) >= low),
            (i64::MAX as i64) <= (high as i64) || (self as i64) <= (high as i64),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i8> for u64 {
    fn range_with(self, low: i8, high: i8) -> RangeCheck {
        RangeCheck::new(
            low <= 0 as i8 || (self as u64) >= (low as u64),
            (self as u64) <= (high as u64),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u8> for u64 {
    fn range_with(self, low: u8, high: u8) -> RangeCheck {
        RangeCheck::new(
            low == 0 as u8 || self >= (low as u64),
            self <= (high as u64),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i16> for u64 {
    fn range_with(self, low: i16, high: i16) -> RangeCheck {
        RangeCheck::new(
            low <= 0 as i16 || (self as u64) >= (low as u64),
            (self as u64) <= (high as u64),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u16> for u64 {
    fn range_with(self, low: u16, high: u16) -> RangeCheck {
        RangeCheck::new(
            low == 0 as u16 || self >= (low as u64),
            self <= (high as u64),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i32> for u64 {
    fn range_with(self, low: i32, high: i32) -> RangeCheck {
        RangeCheck::new(
            low <= 0 as i32 || (self as u64) >= (low as u64),
            (self as u64) <= (high as u64),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u32> for u64 {
    fn range_with(self, low: u32, high: u32) -> RangeCheck {
        RangeCheck::new(
            low == 0 as u32 || self >= (low as u64),
            self <= (high as u64),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i64> for u64 {
    fn range_with(self, low: i64, high: i64) -> RangeCheck {
        RangeCheck::new(
            low <= 0 as i64 || (self as u64) >= (low as u64),
            (self as u64) <= (high as u64),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u64> for u64 {
    fn range_with(self, low: u64, high: u64) -> RangeCheck {
        RangeCheck::new(
            (u64::MIN as u64) >= low || (self as u64) >= low,
            (u64::MAX as u64) <= high || (self as u64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<f32> for u64 {
    fn range_with(self, low: f32, high: f32) -> RangeCheck {
        RangeCheck::new(
            (u64::MIN as f32) >= low || (self as f32) >= low,
            (u64::MAX as f32) <= high || (self as f32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<f64> for u64 {
    fn range_with(self, low: f64, high: f64) -> RangeCheck {
        RangeCheck::new(
            (u64::MIN as f64) >= low || (self as f64) >= low,
            (u64::MAX as f64) <= high || (self as f64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<isize> for u64 {
    fn range_with(self, low: isize, high: isize) -> RangeCheck {
        RangeCheck::new(
            low <= 0 as isize || (self as u64) >= (low as u64),
            (self as u64) <= (high as u64),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<usize> for u64 {
    fn range_with(self, low: usize, high: usize) -> RangeCheck {
        RangeCheck::new(
            low == 0 as usize || self >= (low as u64),
            self <= (high as u64),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i8> for f32 {
    fn range_with(self, low: i8, high: i8) -> RangeCheck {
        RangeCheck::new(self >= (low as f32), self <= (high as f32))
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u8> for f32 {
    fn range_with(self, low: u8, high: u8) -> RangeCheck {
        RangeCheck::new(
            (self > -1.0) && (low == 0 as u8 || (self as u8) >= low),
            (f32::MAX as f32) <= (high as f32) || (self as f32) <= (high as f32),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i16> for f32 {
    fn range_with(self, low: i16, high: i16) -> RangeCheck {
        RangeCheck::new(self >= (low as f32), self <= (high as f32))
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u16> for f32 {
    fn range_with(self, low: u16, high: u16) -> RangeCheck {
        RangeCheck::new(
            (self > -1.0) && (low == 0 as u16 || (self as u16) >= low),
            (f32::MAX as f32) <= (high as f32) || (self as f32) <= (high as f32),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i32> for f32 {
    fn range_with(self, low: i32, high: i32) -> RangeCheck {
        let low = {
            let neg = low < 0;
            let abs = if low < 0 {
                (0 as u32).wrapping_sub(low as u32)
            } else {
                low as u32
            };
            let masked = abs & !(((1 as u32) << 7) - 1);
            if neg {
                (0 as u32).wrapping_sub(masked) as i32
            } else {
                masked as i32
            }
        };
        let high = {
            let neg = high < 0;
            let abs = if high < 0 {
                (0 as u32).wrapping_sub(high as u32)
            } else {
                high as u32
            };
            let masked = abs & !(((1 as u32) << 7) - 1);
            if neg {
                (0 as u32).wrapping_sub(masked) as i32
            } else {
                masked as i32
            }
        };
        RangeCheck::new(self >= (low as f32), self <= (high as f32))
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u32> for f32 {
    fn range_with(self, low: u32, high: u32) -> RangeCheck {
        let low = {
            let neg = low < 0;
            let abs = low as u32;
            let masked = abs & !(((1 as u32) << 8) - 1);
            if neg {
                (0 as u32).wrapping_sub(masked) as u32
            } else {
                masked as u32
            }
        };
        let high = {
            let neg = high < 0;
            let abs = high as u32;
            let masked = abs & !(((1 as u32) << 8) - 1);
            if neg {
                (0 as u32).wrapping_sub(masked) as u32
            } else {
                masked as u32
            }
        };
        RangeCheck::new(
            (self > -1.0) && (low == 0 as u32 || (self as u32) >= low),
            (f32::MAX as f32) <= (high as f32) || (self as f32) <= (high as f32),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i64> for f32 {
    fn range_with(self, low: i64, high: i64) -> RangeCheck {
        let low = {
            let neg = low < 0;
            let abs = if low < 0 {
                (0 as u64).wrapping_sub(low as u64)
            } else {
                low as u64
            };
            let masked = abs & !(((1 as u64) << 39) - 1);
            if neg {
                (0 as u64).wrapping_sub(masked) as i64
            } else {
                masked as i64
            }
        };
        let high = {
            let neg = high < 0;
            let abs = if high < 0 {
                (0 as u64).wrapping_sub(high as u64)
            } else {
                high as u64
            };
            let masked = abs & !(((1 as u64) << 39) - 1);
            if neg {
                (0 as u64).wrapping_sub(masked) as i64
            } else {
                masked as i64
            }
        };
        RangeCheck::new(self >= (low as f32), self <= (high as f32))
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u64> for f32 {
    fn range_with(self, low: u64, high: u64) -> RangeCheck {
        let low = {
            let neg = low < 0;
            let abs = low as u64;
            let masked = abs & !(((1 as u64) << 40) - 1);
            if neg {
                (0 as u64).wrapping_sub(masked) as u64
            } else {
                masked as u64
            }
        };
        let high = {
            let neg = high < 0;
            let abs = high as u64;
            let masked = abs & !(((1 as u64) << 40) - 1);
            if neg {
                (0 as u64).wrapping_sub(masked) as u64
            } else {
                masked as u64
            }
        };
        RangeCheck::new(
            (self > -1.0) && (low == 0 as u64 || (self as u64) >= low),
            (f32::MAX as f32) <= (high as f32) || (self as f32) <= (high as f32),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<f32> for f32 {
    fn range_with(self, low: f32, high: f32) -> RangeCheck {
        RangeCheck::new(
            (f32::MIN as f32) >= low || (self as f32) >= low,
            (f32::MAX as f32) <= high || (self as f32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<f64> for f32 {
    fn range_with(self, low: f64, high: f64) -> RangeCheck {
        RangeCheck::new(
            (f32::MIN as f64) >= low || (self as f64) >= low,
            (f32::MAX as f64) <= high || (self as f64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<isize> for f32 {
    fn range_with(self, low: isize, high: isize) -> RangeCheck {
        let low = {
            let neg = low < 0;
            let abs = if low < 0 {
                (0 as u32).wrapping_sub(low as u32)
            } else {
                low as u32
            };
            let masked = abs & !(((1 as u32) << 7) - 1);
            if neg {
                (0 as u32).wrapping_sub(masked) as isize
            } else {
                masked as isize
            }
        };
        let high = {
            let neg = high < 0;
            let abs = if high < 0 {
                (0 as u32).wrapping_sub(high as u32)
            } else {
                high as u32
            };
            let masked = abs & !(((1 as u32) << 7) - 1);
            if neg {
                (0 as u32).wrapping_sub(masked) as isize
            } else {
                masked as isize
            }
        };
        RangeCheck::new(self >= (low as f32), self <= (high as f32))
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<usize> for f32 {
    fn range_with(self, low: usize, high: usize) -> RangeCheck {
        let low = {
            let neg = low < 0;
            let abs = low as u32;
            let masked = abs & !(((1 as u32) << 8) - 1);
            if neg {
                (0 as u32).wrapping_sub(masked) as usize
            } else {
                masked as usize
            }
        };
        let high = {
            let neg = high < 0;
            let abs = high as u32;
            let masked = abs & !(((1 as u32) << 8) - 1);
            if neg {
                (0 as u32).wrapping_sub(masked) as usize
            } else {
                masked as usize
            }
        };
        RangeCheck::new(
            (self > -1.0) && (low == 0 as usize || (self as usize) >= low),
            (f32::MAX as f32) <= (high as f32) || (self as f32) <= (high as f32),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i8> for f64 {
    fn range_with(self, low: i8, high: i8) -> RangeCheck {
        RangeCheck::new(self >= (low as f64), self <= (high as f64))
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u8> for f64 {
    fn range_with(self, low: u8, high: u8) -> RangeCheck {
        RangeCheck::new(
            (self > -1.0) && (low == 0 as u8 || (self as u8) >= low),
            (f64::MAX as f64) <= (high as f64) || (self as f64) <= (high as f64),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i16> for f64 {
    fn range_with(self, low: i16, high: i16) -> RangeCheck {
        RangeCheck::new(self >= (low as f64), self <= (high as f64))
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u16> for f64 {
    fn range_with(self, low: u16, high: u16) -> RangeCheck {
        RangeCheck::new(
            (self > -1.0) && (low == 0 as u16 || (self as u16) >= low),
            (f64::MAX as f64) <= (high as f64) || (self as f64) <= (high as f64),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i32> for f64 {
    fn range_with(self, low: i32, high: i32) -> RangeCheck {
        RangeCheck::new(self >= (low as f64), self <= (high as f64))
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u32> for f64 {
    fn range_with(self, low: u32, high: u32) -> RangeCheck {
        RangeCheck::new(
            (self > -1.0) && (low == 0 as u32 || (self as u32) >= low),
            (f64::MAX as f64) <= (high as f64) || (self as f64) <= (high as f64),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i64> for f64 {
    fn range_with(self, low: i64, high: i64) -> RangeCheck {
        let low = {
            let neg = low < 0;
            let abs = if low < 0 {
                (0 as u64).wrapping_sub(low as u64)
            } else {
                low as u64
            };
            let masked = abs & !(((1 as u64) << 10) - 1);
            if neg {
                (0 as u64).wrapping_sub(masked) as i64
            } else {
                masked as i64
            }
        };
        let high = {
            let neg = high < 0;
            let abs = if high < 0 {
                (0 as u64).wrapping_sub(high as u64)
            } else {
                high as u64
            };
            let masked = abs & !(((1 as u64) << 10) - 1);
            if neg {
                (0 as u64).wrapping_sub(masked) as i64
            } else {
                masked as i64
            }
        };
        RangeCheck::new(self >= (low as f64), self <= (high as f64))
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u64> for f64 {
    fn range_with(self, low: u64, high: u64) -> RangeCheck {
        let low = {
            let neg = low < 0;
            let abs = low as u64;
            let masked = abs & !(((1 as u64) << 11) - 1);
            if neg {
                (0 as u64).wrapping_sub(masked) as u64
            } else {
                masked as u64
            }
        };
        let high = {
            let neg = high < 0;
            let abs = high as u64;
            let masked = abs & !(((1 as u64) << 11) - 1);
            if neg {
                (0 as u64).wrapping_sub(masked) as u64
            } else {
                masked as u64
            }
        };
        RangeCheck::new(
            (self > -1.0) && (low == 0 as u64 || (self as u64) >= low),
            (f64::MAX as f64) <= (high as f64) || (self as f64) <= (high as f64),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<f32> for f64 {
    fn range_with(self, low: f32, high: f32) -> RangeCheck {
        RangeCheck::new(self >= (low as f64), self <= (high as f64))
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<f64> for f64 {
    fn range_with(self, low: f64, high: f64) -> RangeCheck {
        RangeCheck::new(
            (f64::MIN as f64) >= low || (self as f64) >= low,
            (f64::MAX as f64) <= high || (self as f64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<isize> for f64 {
    fn range_with(self, low: isize, high: isize) -> RangeCheck {
        RangeCheck::new(self >= (low as f64), self <= (high as f64))
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<usize> for f64 {
    fn range_with(self, low: usize, high: usize) -> RangeCheck {
        RangeCheck::new(
            (self > -1.0) && (low == 0 as usize || (self as usize) >= low),
            (f64::MAX as f64) <= (high as f64) || (self as f64) <= (high as f64),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i8> for isize {
    fn range_with(self, low: i8, high: i8) -> RangeCheck {
        RangeCheck::new(self >= (low as isize), self <= (high as isize))
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u8> for isize {
    fn range_with(self, low: u8, high: u8) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u8 || (self as u8) >= low),
            (isize::MAX as i32) <= (high as i32) || (self as i32) <= (high as i32),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i16> for isize {
    fn range_with(self, low: i16, high: i16) -> RangeCheck {
        RangeCheck::new(self >= (low as isize), self <= (high as isize))
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u16> for isize {
    fn range_with(self, low: u16, high: u16) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u16 || (self as u16) >= low),
            (isize::MAX as i32) <= (high as i32) || (self as i32) <= (high as i32),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i32> for isize {
    fn range_with(self, low: i32, high: i32) -> RangeCheck {
        RangeCheck::new(
            (isize::MIN as i32) >= low || (self as i32) >= low,
            (isize::MAX as i32) <= high || (self as i32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u32> for isize {
    fn range_with(self, low: u32, high: u32) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u32 || (self as u32) >= low),
            (isize::MAX as u32) <= (high as u32) || (self as u32) <= (high as u32),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i64> for isize {
    fn range_with(self, low: i64, high: i64) -> RangeCheck {
        RangeCheck::new(
            (isize::MIN as i64) >= low || (self as i64) >= low,
            (isize::MAX as i64) <= high || (self as i64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u64> for isize {
    fn range_with(self, low: u64, high: u64) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u64 || (self as u64) >= low),
            (isize::MAX as u64) <= (high as u64) || (self as u64) <= (high as u64),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<f32> for isize {
    fn range_with(self, low: f32, high: f32) -> RangeCheck {
        RangeCheck::new(
            (isize::MIN as f32) >= low || (self as f32) >= low,
            (isize::MAX as f32) <= high || (self as f32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<f64> for isize {
    fn range_with(self, low: f64, high: f64) -> RangeCheck {
        RangeCheck::new(
            (isize::MIN as f64) >= low || (self as f64) >= low,
            (isize::MAX as f64) <= high || (self as f64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<isize> for isize {
    fn range_with(self, low: isize, high: isize) -> RangeCheck {
        RangeCheck::new(
            (isize::MIN as isize) >= low || (self as isize) >= low,
            (isize::MAX as isize) <= high || (self as isize) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<usize> for isize {
    fn range_with(self, low: usize, high: usize) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as usize || (self as usize) >= low),
            (isize::MAX as usize) <= (high as usize) || (self as usize) <= (high as usize),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i8> for usize {
    fn range_with(self, low: i8, high: i8) -> RangeCheck {
        RangeCheck::new(
            low <= 0 as i8 || (self as usize) >= (low as usize),
            (self as usize) <= (high as usize),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u8> for usize {
    fn range_with(self, low: u8, high: u8) -> RangeCheck {
        RangeCheck::new(
            low == 0 as u8 || self >= (low as usize),
            self <= (high as usize),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i16> for usize {
    fn range_with(self, low: i16, high: i16) -> RangeCheck {
        RangeCheck::new(
            low <= 0 as i16 || (self as usize) >= (low as usize),
            (self as usize) <= (high as usize),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u16> for usize {
    fn range_with(self, low: u16, high: u16) -> RangeCheck {
        RangeCheck::new(
            low == 0 as u16 || self >= (low as usize),
            self <= (high as usize),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i32> for usize {
    fn range_with(self, low: i32, high: i32) -> RangeCheck {
        RangeCheck::new(
            low <= 0 as i32 || (self as usize) >= (low as usize),
            (self as usize) <= (high as usize),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u32> for usize {
    fn range_with(self, low: u32, high: u32) -> RangeCheck {
        RangeCheck::new(
            (usize::MIN as u32) >= low || (self as u32) >= low,
            (usize::MAX as u32) <= high || (self as u32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<i64> for usize {
    fn range_with(self, low: i64, high: i64) -> RangeCheck {
        RangeCheck::new(
            (usize::MIN as i64) >= low || (self as i64) >= low,
            (usize::MAX as i64) <= high || (self as i64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<u64> for usize {
    fn range_with(self, low: u64, high: u64) -> RangeCheck {
        RangeCheck::new(
            (usize::MIN as u64) >= low || (self as u64) >= low,
            (usize::MAX as u64) <= high || (self as u64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<f32> for usize {
    fn range_with(self, low: f32, high: f32) -> RangeCheck {
        RangeCheck::new(
            (usize::MIN as f32) >= low || (self as f32) >= low,
            (usize::MAX as f32) <= high || (self as f32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<f64> for usize {
    fn range_with(self, low: f64, high: f64) -> RangeCheck {
        RangeCheck::new(
            (usize::MIN as f64) >= low || (self as f64) >= low,
            (usize::MAX as f64) <= high || (self as f64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<isize> for usize {
    fn range_with(self, low: isize, high: isize) -> RangeCheck {
        RangeCheck::new(
            low <= 0 as isize || (self as usize) >= (low as usize),
            (self as usize) <= (high as usize),
        )
    }
}
#[cfg(target_pointer_width = "32")]
impl RangeWith<usize> for usize {
    fn range_with(self, low: usize, high: usize) -> RangeCheck {
        RangeCheck::new(
            (usize::MIN as usize) >= low || (self as usize) >= low,
            (usize::MAX as usize) <= high || (self as usize) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i8> for i8 {
    fn range_with(self, low: i8, high: i8) -> RangeCheck {
        RangeCheck::new(
            (i8::MIN as i8) >= low || (self as i8) >= low,
            (i8::MAX as i8) <= high || (self as i8) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u8> for i8 {
    fn range_with(self, low: u8, high: u8) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u8 || (self as u8) >= low),
            (i8::MAX as i32) <= (high as i32) || (self as i32) <= (high as i32),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i16> for i8 {
    fn range_with(self, low: i16, high: i16) -> RangeCheck {
        RangeCheck::new(
            (i8::MIN as i16) >= low || (self as i16) >= low,
            (i8::MAX as i16) <= high || (self as i16) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u16> for i8 {
    fn range_with(self, low: u16, high: u16) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u16 || (self as u16) >= low),
            (i8::MAX as i32) <= (high as i32) || (self as i32) <= (high as i32),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i32> for i8 {
    fn range_with(self, low: i32, high: i32) -> RangeCheck {
        RangeCheck::new(
            (i8::MIN as i32) >= low || (self as i32) >= low,
            (i8::MAX as i32) <= high || (self as i32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u32> for i8 {
    fn range_with(self, low: u32, high: u32) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u32 || (self as u32) >= low),
            (i8::MAX as u32) <= (high as u32) || (self as u32) <= (high as u32),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i64> for i8 {
    fn range_with(self, low: i64, high: i64) -> RangeCheck {
        RangeCheck::new(
            (i8::MIN as i64) >= low || (self as i64) >= low,
            (i8::MAX as i64) <= high || (self as i64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u64> for i8 {
    fn range_with(self, low: u64, high: u64) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u64 || (self as u64) >= low),
            (i8::MAX as u64) <= (high as u64) || (self as u64) <= (high as u64),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<f32> for i8 {
    fn range_with(self, low: f32, high: f32) -> RangeCheck {
        RangeCheck::new(
            (i8::MIN as f32) >= low || (self as f32) >= low,
            (i8::MAX as f32) <= high || (self as f32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<f64> for i8 {
    fn range_with(self, low: f64, high: f64) -> RangeCheck {
        RangeCheck::new(
            (i8::MIN as f64) >= low || (self as f64) >= low,
            (i8::MAX as f64) <= high || (self as f64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<isize> for i8 {
    fn range_with(self, low: isize, high: isize) -> RangeCheck {
        RangeCheck::new(
            (i8::MIN as isize) >= low || (self as isize) >= low,
            (i8::MAX as isize) <= high || (self as isize) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<usize> for i8 {
    fn range_with(self, low: usize, high: usize) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as usize || (self as usize) >= low),
            (i8::MAX as usize) <= (high as usize) || (self as usize) <= (high as usize),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i8> for u8 {
    fn range_with(self, low: i8, high: i8) -> RangeCheck {
        RangeCheck::new(
            low <= 0 as i8 || (self as i32) >= (low as i32),
            (self as i32) <= (high as i32),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u8> for u8 {
    fn range_with(self, low: u8, high: u8) -> RangeCheck {
        RangeCheck::new(
            (u8::MIN as u8) >= low || (self as u8) >= low,
            (u8::MAX as u8) <= high || (self as u8) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i16> for u8 {
    fn range_with(self, low: i16, high: i16) -> RangeCheck {
        RangeCheck::new(
            (u8::MIN as i16) >= low || (self as i16) >= low,
            (u8::MAX as i16) <= high || (self as i16) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u16> for u8 {
    fn range_with(self, low: u16, high: u16) -> RangeCheck {
        RangeCheck::new(
            (u8::MIN as u16) >= low || (self as u16) >= low,
            (u8::MAX as u16) <= high || (self as u16) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i32> for u8 {
    fn range_with(self, low: i32, high: i32) -> RangeCheck {
        RangeCheck::new(
            (u8::MIN as i32) >= low || (self as i32) >= low,
            (u8::MAX as i32) <= high || (self as i32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u32> for u8 {
    fn range_with(self, low: u32, high: u32) -> RangeCheck {
        RangeCheck::new(
            (u8::MIN as u32) >= low || (self as u32) >= low,
            (u8::MAX as u32) <= high || (self as u32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i64> for u8 {
    fn range_with(self, low: i64, high: i64) -> RangeCheck {
        RangeCheck::new(
            (u8::MIN as i64) >= low || (self as i64) >= low,
            (u8::MAX as i64) <= high || (self as i64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u64> for u8 {
    fn range_with(self, low: u64, high: u64) -> RangeCheck {
        RangeCheck::new(
            (u8::MIN as u64) >= low || (self as u64) >= low,
            (u8::MAX as u64) <= high || (self as u64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<f32> for u8 {
    fn range_with(self, low: f32, high: f32) -> RangeCheck {
        RangeCheck::new(
            (u8::MIN as f32) >= low || (self as f32) >= low,
            (u8::MAX as f32) <= high || (self as f32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<f64> for u8 {
    fn range_with(self, low: f64, high: f64) -> RangeCheck {
        RangeCheck::new(
            (u8::MIN as f64) >= low || (self as f64) >= low,
            (u8::MAX as f64) <= high || (self as f64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<isize> for u8 {
    fn range_with(self, low: isize, high: isize) -> RangeCheck {
        RangeCheck::new(
            (u8::MIN as isize) >= low || (self as isize) >= low,
            (u8::MAX as isize) <= high || (self as isize) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<usize> for u8 {
    fn range_with(self, low: usize, high: usize) -> RangeCheck {
        RangeCheck::new(
            (u8::MIN as usize) >= low || (self as usize) >= low,
            (u8::MAX as usize) <= high || (self as usize) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i8> for i16 {
    fn range_with(self, low: i8, high: i8) -> RangeCheck {
        RangeCheck::new(self >= (low as i16), self <= (high as i16))
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u8> for i16 {
    fn range_with(self, low: u8, high: u8) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u8 || (self as u8) >= low),
            (i16::MAX as i32) <= (high as i32) || (self as i32) <= (high as i32),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i16> for i16 {
    fn range_with(self, low: i16, high: i16) -> RangeCheck {
        RangeCheck::new(
            (i16::MIN as i16) >= low || (self as i16) >= low,
            (i16::MAX as i16) <= high || (self as i16) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u16> for i16 {
    fn range_with(self, low: u16, high: u16) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u16 || (self as u16) >= low),
            (i16::MAX as i32) <= (high as i32) || (self as i32) <= (high as i32),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i32> for i16 {
    fn range_with(self, low: i32, high: i32) -> RangeCheck {
        RangeCheck::new(
            (i16::MIN as i32) >= low || (self as i32) >= low,
            (i16::MAX as i32) <= high || (self as i32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u32> for i16 {
    fn range_with(self, low: u32, high: u32) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u32 || (self as u32) >= low),
            (i16::MAX as u32) <= (high as u32) || (self as u32) <= (high as u32),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i64> for i16 {
    fn range_with(self, low: i64, high: i64) -> RangeCheck {
        RangeCheck::new(
            (i16::MIN as i64) >= low || (self as i64) >= low,
            (i16::MAX as i64) <= high || (self as i64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u64> for i16 {
    fn range_with(self, low: u64, high: u64) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u64 || (self as u64) >= low),
            (i16::MAX as u64) <= (high as u64) || (self as u64) <= (high as u64),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<f32> for i16 {
    fn range_with(self, low: f32, high: f32) -> RangeCheck {
        RangeCheck::new(
            (i16::MIN as f32) >= low || (self as f32) >= low,
            (i16::MAX as f32) <= high || (self as f32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<f64> for i16 {
    fn range_with(self, low: f64, high: f64) -> RangeCheck {
        RangeCheck::new(
            (i16::MIN as f64) >= low || (self as f64) >= low,
            (i16::MAX as f64) <= high || (self as f64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<isize> for i16 {
    fn range_with(self, low: isize, high: isize) -> RangeCheck {
        RangeCheck::new(
            (i16::MIN as isize) >= low || (self as isize) >= low,
            (i16::MAX as isize) <= high || (self as isize) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<usize> for i16 {
    fn range_with(self, low: usize, high: usize) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as usize || (self as usize) >= low),
            (i16::MAX as usize) <= (high as usize) || (self as usize) <= (high as usize),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i8> for u16 {
    fn range_with(self, low: i8, high: i8) -> RangeCheck {
        RangeCheck::new(
            low <= 0 as i8 || (self as i32) >= (low as i32),
            (self as i32) <= (high as i32),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u8> for u16 {
    fn range_with(self, low: u8, high: u8) -> RangeCheck {
        RangeCheck::new(
            low == 0 as u8 || self >= (low as u16),
            self <= (high as u16),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i16> for u16 {
    fn range_with(self, low: i16, high: i16) -> RangeCheck {
        RangeCheck::new(
            low <= 0 as i16 || (self as i32) >= (low as i32),
            (self as i32) <= (high as i32),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u16> for u16 {
    fn range_with(self, low: u16, high: u16) -> RangeCheck {
        RangeCheck::new(
            (u16::MIN as u16) >= low || (self as u16) >= low,
            (u16::MAX as u16) <= high || (self as u16) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i32> for u16 {
    fn range_with(self, low: i32, high: i32) -> RangeCheck {
        RangeCheck::new(
            (u16::MIN as i32) >= low || (self as i32) >= low,
            (u16::MAX as i32) <= high || (self as i32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u32> for u16 {
    fn range_with(self, low: u32, high: u32) -> RangeCheck {
        RangeCheck::new(
            (u16::MIN as u32) >= low || (self as u32) >= low,
            (u16::MAX as u32) <= high || (self as u32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i64> for u16 {
    fn range_with(self, low: i64, high: i64) -> RangeCheck {
        RangeCheck::new(
            (u16::MIN as i64) >= low || (self as i64) >= low,
            (u16::MAX as i64) <= high || (self as i64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u64> for u16 {
    fn range_with(self, low: u64, high: u64) -> RangeCheck {
        RangeCheck::new(
            (u16::MIN as u64) >= low || (self as u64) >= low,
            (u16::MAX as u64) <= high || (self as u64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<f32> for u16 {
    fn range_with(self, low: f32, high: f32) -> RangeCheck {
        RangeCheck::new(
            (u16::MIN as f32) >= low || (self as f32) >= low,
            (u16::MAX as f32) <= high || (self as f32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<f64> for u16 {
    fn range_with(self, low: f64, high: f64) -> RangeCheck {
        RangeCheck::new(
            (u16::MIN as f64) >= low || (self as f64) >= low,
            (u16::MAX as f64) <= high || (self as f64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<isize> for u16 {
    fn range_with(self, low: isize, high: isize) -> RangeCheck {
        RangeCheck::new(
            (u16::MIN as isize) >= low || (self as isize) >= low,
            (u16::MAX as isize) <= high || (self as isize) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<usize> for u16 {
    fn range_with(self, low: usize, high: usize) -> RangeCheck {
        RangeCheck::new(
            (u16::MIN as usize) >= low || (self as usize) >= low,
            (u16::MAX as usize) <= high || (self as usize) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i8> for i32 {
    fn range_with(self, low: i8, high: i8) -> RangeCheck {
        RangeCheck::new(self >= (low as i32), self <= (high as i32))
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u8> for i32 {
    fn range_with(self, low: u8, high: u8) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u8 || (self as u8) >= low),
            (i32::MAX as i32) <= (high as i32) || (self as i32) <= (high as i32),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i16> for i32 {
    fn range_with(self, low: i16, high: i16) -> RangeCheck {
        RangeCheck::new(self >= (low as i32), self <= (high as i32))
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u16> for i32 {
    fn range_with(self, low: u16, high: u16) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u16 || (self as u16) >= low),
            (i32::MAX as i32) <= (high as i32) || (self as i32) <= (high as i32),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i32> for i32 {
    fn range_with(self, low: i32, high: i32) -> RangeCheck {
        RangeCheck::new(
            (i32::MIN as i32) >= low || (self as i32) >= low,
            (i32::MAX as i32) <= high || (self as i32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u32> for i32 {
    fn range_with(self, low: u32, high: u32) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u32 || (self as u32) >= low),
            (i32::MAX as u32) <= (high as u32) || (self as u32) <= (high as u32),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i64> for i32 {
    fn range_with(self, low: i64, high: i64) -> RangeCheck {
        RangeCheck::new(
            (i32::MIN as i64) >= low || (self as i64) >= low,
            (i32::MAX as i64) <= high || (self as i64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u64> for i32 {
    fn range_with(self, low: u64, high: u64) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u64 || (self as u64) >= low),
            (i32::MAX as u64) <= (high as u64) || (self as u64) <= (high as u64),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<f32> for i32 {
    fn range_with(self, low: f32, high: f32) -> RangeCheck {
        RangeCheck::new(
            (i32::MIN as f32) >= low || (self as f32) >= low,
            (i32::MAX as f32) <= high || (self as f32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<f64> for i32 {
    fn range_with(self, low: f64, high: f64) -> RangeCheck {
        RangeCheck::new(
            (i32::MIN as f64) >= low || (self as f64) >= low,
            (i32::MAX as f64) <= high || (self as f64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<isize> for i32 {
    fn range_with(self, low: isize, high: isize) -> RangeCheck {
        RangeCheck::new(
            (i32::MIN as isize) >= low || (self as isize) >= low,
            (i32::MAX as isize) <= high || (self as isize) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<usize> for i32 {
    fn range_with(self, low: usize, high: usize) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as usize || (self as usize) >= low),
            (i32::MAX as usize) <= (high as usize) || (self as usize) <= (high as usize),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i8> for u32 {
    fn range_with(self, low: i8, high: i8) -> RangeCheck {
        RangeCheck::new(
            low <= 0 as i8 || (self as u32) >= (low as u32),
            (self as u32) <= (high as u32),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u8> for u32 {
    fn range_with(self, low: u8, high: u8) -> RangeCheck {
        RangeCheck::new(
            low == 0 as u8 || self >= (low as u32),
            self <= (high as u32),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i16> for u32 {
    fn range_with(self, low: i16, high: i16) -> RangeCheck {
        RangeCheck::new(
            low <= 0 as i16 || (self as u32) >= (low as u32),
            (self as u32) <= (high as u32),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u16> for u32 {
    fn range_with(self, low: u16, high: u16) -> RangeCheck {
        RangeCheck::new(
            low == 0 as u16 || self >= (low as u32),
            self <= (high as u32),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i32> for u32 {
    fn range_with(self, low: i32, high: i32) -> RangeCheck {
        RangeCheck::new(
            low <= 0 as i32 || (self as u32) >= (low as u32),
            (self as u32) <= (high as u32),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u32> for u32 {
    fn range_with(self, low: u32, high: u32) -> RangeCheck {
        RangeCheck::new(
            (u32::MIN as u32) >= low || (self as u32) >= low,
            (u32::MAX as u32) <= high || (self as u32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i64> for u32 {
    fn range_with(self, low: i64, high: i64) -> RangeCheck {
        RangeCheck::new(
            (u32::MIN as i64) >= low || (self as i64) >= low,
            (u32::MAX as i64) <= high || (self as i64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u64> for u32 {
    fn range_with(self, low: u64, high: u64) -> RangeCheck {
        RangeCheck::new(
            (u32::MIN as u64) >= low || (self as u64) >= low,
            (u32::MAX as u64) <= high || (self as u64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<f32> for u32 {
    fn range_with(self, low: f32, high: f32) -> RangeCheck {
        RangeCheck::new(
            (u32::MIN as f32) >= low || (self as f32) >= low,
            (u32::MAX as f32) <= high || (self as f32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<f64> for u32 {
    fn range_with(self, low: f64, high: f64) -> RangeCheck {
        RangeCheck::new(
            (u32::MIN as f64) >= low || (self as f64) >= low,
            (u32::MAX as f64) <= high || (self as f64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<isize> for u32 {
    fn range_with(self, low: isize, high: isize) -> RangeCheck {
        RangeCheck::new(
            (u32::MIN as isize) >= low || (self as isize) >= low,
            (u32::MAX as isize) <= high || (self as isize) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<usize> for u32 {
    fn range_with(self, low: usize, high: usize) -> RangeCheck {
        RangeCheck::new(
            (u32::MIN as usize) >= low || (self as usize) >= low,
            (u32::MAX as usize) <= high || (self as usize) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i8> for i64 {
    fn range_with(self, low: i8, high: i8) -> RangeCheck {
        RangeCheck::new(self >= (low as i64), self <= (high as i64))
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u8> for i64 {
    fn range_with(self, low: u8, high: u8) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u8 || (self as u8) >= low),
            (i64::MAX as i64) <= (high as i64) || (self as i64) <= (high as i64),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i16> for i64 {
    fn range_with(self, low: i16, high: i16) -> RangeCheck {
        RangeCheck::new(self >= (low as i64), self <= (high as i64))
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u16> for i64 {
    fn range_with(self, low: u16, high: u16) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u16 || (self as u16) >= low),
            (i64::MAX as i64) <= (high as i64) || (self as i64) <= (high as i64),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i32> for i64 {
    fn range_with(self, low: i32, high: i32) -> RangeCheck {
        RangeCheck::new(self >= (low as i64), self <= (high as i64))
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u32> for i64 {
    fn range_with(self, low: u32, high: u32) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u32 || (self as u32) >= low),
            (i64::MAX as i64) <= (high as i64) || (self as i64) <= (high as i64),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i64> for i64 {
    fn range_with(self, low: i64, high: i64) -> RangeCheck {
        RangeCheck::new(
            (i64::MIN as i64) >= low || (self as i64) >= low,
            (i64::MAX as i64) <= high || (self as i64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u64> for i64 {
    fn range_with(self, low: u64, high: u64) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u64 || (self as u64) >= low),
            (i64::MAX as u64) <= (high as u64) || (self as u64) <= (high as u64),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<f32> for i64 {
    fn range_with(self, low: f32, high: f32) -> RangeCheck {
        RangeCheck::new(
            (i64::MIN as f32) >= low || (self as f32) >= low,
            (i64::MAX as f32) <= high || (self as f32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<f64> for i64 {
    fn range_with(self, low: f64, high: f64) -> RangeCheck {
        RangeCheck::new(
            (i64::MIN as f64) >= low || (self as f64) >= low,
            (i64::MAX as f64) <= high || (self as f64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<isize> for i64 {
    fn range_with(self, low: isize, high: isize) -> RangeCheck {
        RangeCheck::new(
            (i64::MIN as isize) >= low || (self as isize) >= low,
            (i64::MAX as isize) <= high || (self as isize) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<usize> for i64 {
    fn range_with(self, low: usize, high: usize) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as usize || (self as usize) >= low),
            (i64::MAX as usize) <= (high as usize) || (self as usize) <= (high as usize),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i8> for u64 {
    fn range_with(self, low: i8, high: i8) -> RangeCheck {
        RangeCheck::new(
            low <= 0 as i8 || (self as u64) >= (low as u64),
            (self as u64) <= (high as u64),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u8> for u64 {
    fn range_with(self, low: u8, high: u8) -> RangeCheck {
        RangeCheck::new(
            low == 0 as u8 || self >= (low as u64),
            self <= (high as u64),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i16> for u64 {
    fn range_with(self, low: i16, high: i16) -> RangeCheck {
        RangeCheck::new(
            low <= 0 as i16 || (self as u64) >= (low as u64),
            (self as u64) <= (high as u64),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u16> for u64 {
    fn range_with(self, low: u16, high: u16) -> RangeCheck {
        RangeCheck::new(
            low == 0 as u16 || self >= (low as u64),
            self <= (high as u64),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i32> for u64 {
    fn range_with(self, low: i32, high: i32) -> RangeCheck {
        RangeCheck::new(
            low <= 0 as i32 || (self as u64) >= (low as u64),
            (self as u64) <= (high as u64),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u32> for u64 {
    fn range_with(self, low: u32, high: u32) -> RangeCheck {
        RangeCheck::new(
            low == 0 as u32 || self >= (low as u64),
            self <= (high as u64),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i64> for u64 {
    fn range_with(self, low: i64, high: i64) -> RangeCheck {
        RangeCheck::new(
            low <= 0 as i64 || (self as u64) >= (low as u64),
            (self as u64) <= (high as u64),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u64> for u64 {
    fn range_with(self, low: u64, high: u64) -> RangeCheck {
        RangeCheck::new(
            (u64::MIN as u64) >= low || (self as u64) >= low,
            (u64::MAX as u64) <= high || (self as u64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<f32> for u64 {
    fn range_with(self, low: f32, high: f32) -> RangeCheck {
        RangeCheck::new(
            (u64::MIN as f32) >= low || (self as f32) >= low,
            (u64::MAX as f32) <= high || (self as f32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<f64> for u64 {
    fn range_with(self, low: f64, high: f64) -> RangeCheck {
        RangeCheck::new(
            (u64::MIN as f64) >= low || (self as f64) >= low,
            (u64::MAX as f64) <= high || (self as f64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<isize> for u64 {
    fn range_with(self, low: isize, high: isize) -> RangeCheck {
        RangeCheck::new(
            low <= 0 as isize || (self as u64) >= (low as u64),
            (self as u64) <= (high as u64),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<usize> for u64 {
    fn range_with(self, low: usize, high: usize) -> RangeCheck {
        RangeCheck::new(
            (u64::MIN as usize) >= low || (self as usize) >= low,
            (u64::MAX as usize) <= high || (self as usize) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i8> for f32 {
    fn range_with(self, low: i8, high: i8) -> RangeCheck {
        RangeCheck::new(self >= (low as f32), self <= (high as f32))
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u8> for f32 {
    fn range_with(self, low: u8, high: u8) -> RangeCheck {
        RangeCheck::new(
            (self > -1.0) && (low == 0 as u8 || (self as u8) >= low),
            (f32::MAX as f32) <= (high as f32) || (self as f32) <= (high as f32),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i16> for f32 {
    fn range_with(self, low: i16, high: i16) -> RangeCheck {
        RangeCheck::new(self >= (low as f32), self <= (high as f32))
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u16> for f32 {
    fn range_with(self, low: u16, high: u16) -> RangeCheck {
        RangeCheck::new(
            (self > -1.0) && (low == 0 as u16 || (self as u16) >= low),
            (f32::MAX as f32) <= (high as f32) || (self as f32) <= (high as f32),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i32> for f32 {
    fn range_with(self, low: i32, high: i32) -> RangeCheck {
        let low = {
            let neg = low < 0;
            let abs = if low < 0 {
                (0 as u32).wrapping_sub(low as u32)
            } else {
                low as u32
            };
            let masked = abs & !(((1 as u32) << 7) - 1);
            if neg {
                (0 as u32).wrapping_sub(masked) as i32
            } else {
                masked as i32
            }
        };
        let high = {
            let neg = high < 0;
            let abs = if high < 0 {
                (0 as u32).wrapping_sub(high as u32)
            } else {
                high as u32
            };
            let masked = abs & !(((1 as u32) << 7) - 1);
            if neg {
                (0 as u32).wrapping_sub(masked) as i32
            } else {
                masked as i32
            }
        };
        RangeCheck::new(self >= (low as f32), self <= (high as f32))
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u32> for f32 {
    fn range_with(self, low: u32, high: u32) -> RangeCheck {
        let low = {
            let neg = low < 0;
            let abs = low as u32;
            let masked = abs & !(((1 as u32) << 8) - 1);
            if neg {
                (0 as u32).wrapping_sub(masked) as u32
            } else {
                masked as u32
            }
        };
        let high = {
            let neg = high < 0;
            let abs = high as u32;
            let masked = abs & !(((1 as u32) << 8) - 1);
            if neg {
                (0 as u32).wrapping_sub(masked) as u32
            } else {
                masked as u32
            }
        };
        RangeCheck::new(
            (self > -1.0) && (low == 0 as u32 || (self as u32) >= low),
            (f32::MAX as f32) <= (high as f32) || (self as f32) <= (high as f32),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i64> for f32 {
    fn range_with(self, low: i64, high: i64) -> RangeCheck {
        let low = {
            let neg = low < 0;
            let abs = if low < 0 {
                (0 as u64).wrapping_sub(low as u64)
            } else {
                low as u64
            };
            let masked = abs & !(((1 as u64) << 39) - 1);
            if neg {
                (0 as u64).wrapping_sub(masked) as i64
            } else {
                masked as i64
            }
        };
        let high = {
            let neg = high < 0;
            let abs = if high < 0 {
                (0 as u64).wrapping_sub(high as u64)
            } else {
                high as u64
            };
            let masked = abs & !(((1 as u64) << 39) - 1);
            if neg {
                (0 as u64).wrapping_sub(masked) as i64
            } else {
                masked as i64
            }
        };
        RangeCheck::new(self >= (low as f32), self <= (high as f32))
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u64> for f32 {
    fn range_with(self, low: u64, high: u64) -> RangeCheck {
        let low = {
            let neg = low < 0;
            let abs = low as u64;
            let masked = abs & !(((1 as u64) << 40) - 1);
            if neg {
                (0 as u64).wrapping_sub(masked) as u64
            } else {
                masked as u64
            }
        };
        let high = {
            let neg = high < 0;
            let abs = high as u64;
            let masked = abs & !(((1 as u64) << 40) - 1);
            if neg {
                (0 as u64).wrapping_sub(masked) as u64
            } else {
                masked as u64
            }
        };
        RangeCheck::new(
            (self > -1.0) && (low == 0 as u64 || (self as u64) >= low),
            (f32::MAX as f32) <= (high as f32) || (self as f32) <= (high as f32),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<f32> for f32 {
    fn range_with(self, low: f32, high: f32) -> RangeCheck {
        RangeCheck::new(
            (f32::MIN as f32) >= low || (self as f32) >= low,
            (f32::MAX as f32) <= high || (self as f32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<f64> for f32 {
    fn range_with(self, low: f64, high: f64) -> RangeCheck {
        RangeCheck::new(
            (f32::MIN as f64) >= low || (self as f64) >= low,
            (f32::MAX as f64) <= high || (self as f64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<isize> for f32 {
    fn range_with(self, low: isize, high: isize) -> RangeCheck {
        let low = {
            let neg = low < 0;
            let abs = if low < 0 {
                (0 as u64).wrapping_sub(low as u64)
            } else {
                low as u64
            };
            let masked = abs & !(((1 as u64) << 39) - 1);
            if neg {
                (0 as u64).wrapping_sub(masked) as isize
            } else {
                masked as isize
            }
        };
        let high = {
            let neg = high < 0;
            let abs = if high < 0 {
                (0 as u64).wrapping_sub(high as u64)
            } else {
                high as u64
            };
            let masked = abs & !(((1 as u64) << 39) - 1);
            if neg {
                (0 as u64).wrapping_sub(masked) as isize
            } else {
                masked as isize
            }
        };
        RangeCheck::new(self >= (low as f32), self <= (high as f32))
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<usize> for f32 {
    fn range_with(self, low: usize, high: usize) -> RangeCheck {
        let low = {
            let neg = low < 0;
            let abs = low as u64;
            let masked = abs & !(((1 as u64) << 40) - 1);
            if neg {
                (0 as u64).wrapping_sub(masked) as usize
            } else {
                masked as usize
            }
        };
        let high = {
            let neg = high < 0;
            let abs = high as u64;
            let masked = abs & !(((1 as u64) << 40) - 1);
            if neg {
                (0 as u64).wrapping_sub(masked) as usize
            } else {
                masked as usize
            }
        };
        RangeCheck::new(
            (self > -1.0) && (low == 0 as usize || (self as usize) >= low),
            (f32::MAX as f32) <= (high as f32) || (self as f32) <= (high as f32),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i8> for f64 {
    fn range_with(self, low: i8, high: i8) -> RangeCheck {
        RangeCheck::new(self >= (low as f64), self <= (high as f64))
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u8> for f64 {
    fn range_with(self, low: u8, high: u8) -> RangeCheck {
        RangeCheck::new(
            (self > -1.0) && (low == 0 as u8 || (self as u8) >= low),
            (f64::MAX as f64) <= (high as f64) || (self as f64) <= (high as f64),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i16> for f64 {
    fn range_with(self, low: i16, high: i16) -> RangeCheck {
        RangeCheck::new(self >= (low as f64), self <= (high as f64))
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u16> for f64 {
    fn range_with(self, low: u16, high: u16) -> RangeCheck {
        RangeCheck::new(
            (self > -1.0) && (low == 0 as u16 || (self as u16) >= low),
            (f64::MAX as f64) <= (high as f64) || (self as f64) <= (high as f64),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i32> for f64 {
    fn range_with(self, low: i32, high: i32) -> RangeCheck {
        RangeCheck::new(self >= (low as f64), self <= (high as f64))
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u32> for f64 {
    fn range_with(self, low: u32, high: u32) -> RangeCheck {
        RangeCheck::new(
            (self > -1.0) && (low == 0 as u32 || (self as u32) >= low),
            (f64::MAX as f64) <= (high as f64) || (self as f64) <= (high as f64),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i64> for f64 {
    fn range_with(self, low: i64, high: i64) -> RangeCheck {
        let low = {
            let neg = low < 0;
            let abs = if low < 0 {
                (0 as u64).wrapping_sub(low as u64)
            } else {
                low as u64
            };
            let masked = abs & !(((1 as u64) << 10) - 1);
            if neg {
                (0 as u64).wrapping_sub(masked) as i64
            } else {
                masked as i64
            }
        };
        let high = {
            let neg = high < 0;
            let abs = if high < 0 {
                (0 as u64).wrapping_sub(high as u64)
            } else {
                high as u64
            };
            let masked = abs & !(((1 as u64) << 10) - 1);
            if neg {
                (0 as u64).wrapping_sub(masked) as i64
            } else {
                masked as i64
            }
        };
        RangeCheck::new(self >= (low as f64), self <= (high as f64))
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u64> for f64 {
    fn range_with(self, low: u64, high: u64) -> RangeCheck {
        let low = {
            let neg = low < 0;
            let abs = low as u64;
            let masked = abs & !(((1 as u64) << 11) - 1);
            if neg {
                (0 as u64).wrapping_sub(masked) as u64
            } else {
                masked as u64
            }
        };
        let high = {
            let neg = high < 0;
            let abs = high as u64;
            let masked = abs & !(((1 as u64) << 11) - 1);
            if neg {
                (0 as u64).wrapping_sub(masked) as u64
            } else {
                masked as u64
            }
        };
        RangeCheck::new(
            (self > -1.0) && (low == 0 as u64 || (self as u64) >= low),
            (f64::MAX as f64) <= (high as f64) || (self as f64) <= (high as f64),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<f32> for f64 {
    fn range_with(self, low: f32, high: f32) -> RangeCheck {
        RangeCheck::new(self >= (low as f64), self <= (high as f64))
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<f64> for f64 {
    fn range_with(self, low: f64, high: f64) -> RangeCheck {
        RangeCheck::new(
            (f64::MIN as f64) >= low || (self as f64) >= low,
            (f64::MAX as f64) <= high || (self as f64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<isize> for f64 {
    fn range_with(self, low: isize, high: isize) -> RangeCheck {
        let low = {
            let neg = low < 0;
            let abs = if low < 0 {
                (0 as u64).wrapping_sub(low as u64)
            } else {
                low as u64
            };
            let masked = abs & !(((1 as u64) << 10) - 1);
            if neg {
                (0 as u64).wrapping_sub(masked) as isize
            } else {
                masked as isize
            }
        };
        let high = {
            let neg = high < 0;
            let abs = if high < 0 {
                (0 as u64).wrapping_sub(high as u64)
            } else {
                high as u64
            };
            let masked = abs & !(((1 as u64) << 10) - 1);
            if neg {
                (0 as u64).wrapping_sub(masked) as isize
            } else {
                masked as isize
            }
        };
        RangeCheck::new(self >= (low as f64), self <= (high as f64))
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<usize> for f64 {
    fn range_with(self, low: usize, high: usize) -> RangeCheck {
        let low = {
            let neg = low < 0;
            let abs = low as u64;
            let masked = abs & !(((1 as u64) << 11) - 1);
            if neg {
                (0 as u64).wrapping_sub(masked) as usize
            } else {
                masked as usize
            }
        };
        let high = {
            let neg = high < 0;
            let abs = high as u64;
            let masked = abs & !(((1 as u64) << 11) - 1);
            if neg {
                (0 as u64).wrapping_sub(masked) as usize
            } else {
                masked as usize
            }
        };
        RangeCheck::new(
            (self > -1.0) && (low == 0 as usize || (self as usize) >= low),
            (f64::MAX as f64) <= (high as f64) || (self as f64) <= (high as f64),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i8> for isize {
    fn range_with(self, low: i8, high: i8) -> RangeCheck {
        RangeCheck::new(self >= (low as isize), self <= (high as isize))
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u8> for isize {
    fn range_with(self, low: u8, high: u8) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u8 || (self as u8) >= low),
            (isize::MAX as isize) <= (high as isize) || (self as isize) <= (high as isize),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i16> for isize {
    fn range_with(self, low: i16, high: i16) -> RangeCheck {
        RangeCheck::new(self >= (low as isize), self <= (high as isize))
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u16> for isize {
    fn range_with(self, low: u16, high: u16) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u16 || (self as u16) >= low),
            (isize::MAX as isize) <= (high as isize) || (self as isize) <= (high as isize),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i32> for isize {
    fn range_with(self, low: i32, high: i32) -> RangeCheck {
        RangeCheck::new(self >= (low as isize), self <= (high as isize))
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u32> for isize {
    fn range_with(self, low: u32, high: u32) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u32 || (self as u32) >= low),
            (isize::MAX as isize) <= (high as isize) || (self as isize) <= (high as isize),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i64> for isize {
    fn range_with(self, low: i64, high: i64) -> RangeCheck {
        RangeCheck::new(
            (isize::MIN as i64) >= low || (self as i64) >= low,
            (isize::MAX as i64) <= high || (self as i64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u64> for isize {
    fn range_with(self, low: u64, high: u64) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as u64 || (self as u64) >= low),
            (isize::MAX as u64) <= (high as u64) || (self as u64) <= (high as u64),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<f32> for isize {
    fn range_with(self, low: f32, high: f32) -> RangeCheck {
        RangeCheck::new(
            (isize::MIN as f32) >= low || (self as f32) >= low,
            (isize::MAX as f32) <= high || (self as f32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<f64> for isize {
    fn range_with(self, low: f64, high: f64) -> RangeCheck {
        RangeCheck::new(
            (isize::MIN as f64) >= low || (self as f64) >= low,
            (isize::MAX as f64) <= high || (self as f64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<isize> for isize {
    fn range_with(self, low: isize, high: isize) -> RangeCheck {
        RangeCheck::new(
            (isize::MIN as isize) >= low || (self as isize) >= low,
            (isize::MAX as isize) <= high || (self as isize) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<usize> for isize {
    fn range_with(self, low: usize, high: usize) -> RangeCheck {
        RangeCheck::new(
            (self >= 0) && (low == 0 as usize || (self as usize) >= low),
            (isize::MAX as usize) <= (high as usize) || (self as usize) <= (high as usize),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i8> for usize {
    fn range_with(self, low: i8, high: i8) -> RangeCheck {
        RangeCheck::new(
            low <= 0 as i8 || (self as usize) >= (low as usize),
            (self as usize) <= (high as usize),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u8> for usize {
    fn range_with(self, low: u8, high: u8) -> RangeCheck {
        RangeCheck::new(
            low == 0 as u8 || self >= (low as usize),
            self <= (high as usize),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i16> for usize {
    fn range_with(self, low: i16, high: i16) -> RangeCheck {
        RangeCheck::new(
            low <= 0 as i16 || (self as usize) >= (low as usize),
            (self as usize) <= (high as usize),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u16> for usize {
    fn range_with(self, low: u16, high: u16) -> RangeCheck {
        RangeCheck::new(
            low == 0 as u16 || self >= (low as usize),
            self <= (high as usize),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i32> for usize {
    fn range_with(self, low: i32, high: i32) -> RangeCheck {
        RangeCheck::new(
            low <= 0 as i32 || (self as usize) >= (low as usize),
            (self as usize) <= (high as usize),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u32> for usize {
    fn range_with(self, low: u32, high: u32) -> RangeCheck {
        RangeCheck::new(
            low == 0 as u32 || self >= (low as usize),
            self <= (high as usize),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<i64> for usize {
    fn range_with(self, low: i64, high: i64) -> RangeCheck {
        RangeCheck::new(
            low <= 0 as i64 || (self as usize) >= (low as usize),
            (self as usize) <= (high as usize),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<u64> for usize {
    fn range_with(self, low: u64, high: u64) -> RangeCheck {
        RangeCheck::new(
            (usize::MIN as u64) >= low || (self as u64) >= low,
            (usize::MAX as u64) <= high || (self as u64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<f32> for usize {
    fn range_with(self, low: f32, high: f32) -> RangeCheck {
        RangeCheck::new(
            (usize::MIN as f32) >= low || (self as f32) >= low,
            (usize::MAX as f32) <= high || (self as f32) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<f64> for usize {
    fn range_with(self, low: f64, high: f64) -> RangeCheck {
        RangeCheck::new(
            (usize::MIN as f64) >= low || (self as f64) >= low,
            (usize::MAX as f64) <= high || (self as f64) <= high,
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<isize> for usize {
    fn range_with(self, low: isize, high: isize) -> RangeCheck {
        RangeCheck::new(
            low <= 0 as isize || (self as usize) >= (low as usize),
            (self as usize) <= (high as usize),
        )
    }
}
#[cfg(target_pointer_width = "64")]
impl RangeWith<usize> for usize {
    fn range_with(self, low: usize, high: usize) -> RangeCheck {
        RangeCheck::new(
            (usize::MIN as usize) >= low || (self as usize) >= low,
            (usize::MAX as usize) <= high || (self as usize) <= high,
        )
    }
}
// Source: safe_conversions_impl.h:711-740. Common bounds use safe source-aware comparisons.
pub fn kCommonMax<D, S>() -> D
where
    D: Numeric + Cast<S> + Promote<S>,
    S: Numeric + Cast<D>,
{
    if IsGreaterOrEqual(D::MAX, S::MAX) {
        S::MAX.cast()
    } else {
        D::MAX
    }
}
pub fn kCommonMin<D, S>() -> D
where
    D: Numeric + Cast<S> + Promote<S>,
    S: Numeric + Cast<D>,
{
    if IsLessOrEqual(D::LOWEST, S::LOWEST) {
        S::LOWEST.cast()
    } else {
        D::LOWEST
    }
}
pub fn CommonMaxOrMin<D, S>(is_min: bool) -> D
where
    D: Numeric + Cast<S> + Promote<S>,
    S: Numeric + Cast<D>,
{
    if is_min {
        kCommonMin::<D, S>()
    } else {
        kCommonMax::<D, S>()
    }
}

// Source: arithmetic bool is a native conversion/StrictNumeric domain;
// make_unsigned<bool>, integer inversion and overflow builtin bool outputs are not instantiated.
#[cfg(target_pointer_width = "32")]
mod bool_domain {
    use super::*;
    impl super::domain::sealed::Primitive for bool {}
    impl Numeric for bool {
        const ZERO: Self = false;
        const MAX: Self = true;
        const LOWEST: Self = false;
        const SIGNED: bool = false;
        const FLOAT: bool = false;
        const DIGITS: u32 = 1;
        const MAX_EXPONENT: u32 = 2;
        fn negative(self) -> bool {
            false
        }
        fn finite(self) -> bool {
            true
        }
        fn nan() -> Self {
            false
        }
        fn overflow() -> Self {
            true
        }
        fn underflow() -> Self {
            false
        }
    }
    impl Cast<bool> for i8 {
        const CONTAINED: bool = false;
        fn cast(self) -> bool {
            (self != 0 as i8)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, self <= 1 as i8)
        }
    }
    impl RangeWith<bool> for i8 {
        fn range_with(self, low: bool, high: bool) -> RangeCheck {
            RangeCheck::new(
                (self >= 0) && (!low || (self != 0 as i8) >= low),
                self <= (high as u8 as i8),
            )
        }
    }
    impl Promote<bool> for i8 {
        type Max = i8;
        type Lowest = i8;
        type Big = i8;
        type Fast = i16;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: bool) -> (i32, i32) {
            ((self as i32), (r as u8 as i32))
        }
    }
    impl SafeComparison<bool> for i8 {
        fn safe_lt(self, r: bool) -> bool {
            IsLess(self, (r as u8 as i8))
        }
        fn safe_le(self, r: bool) -> bool {
            IsLessOrEqual(self, (r as u8 as i8))
        }
        fn safe_gt(self, r: bool) -> bool {
            IsGreater(self, (r as u8 as i8))
        }
        fn safe_ge(self, r: bool) -> bool {
            IsGreaterOrEqual(self, (r as u8 as i8))
        }
        fn safe_eq(self, r: bool) -> bool {
            IsEqual(self, (r as u8 as i8))
        }
        fn safe_ne(self, r: bool) -> bool {
            IsNotEqual(self, (r as u8 as i8))
        }
    }
    impl Cast<bool> for u8 {
        const CONTAINED: bool = false;
        fn cast(self) -> bool {
            (self != 0 as u8)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= 1 as u8)
        }
    }
    impl RangeWith<bool> for u8 {
        fn range_with(self, low: bool, high: bool) -> RangeCheck {
            RangeCheck::new(
                (true) && (!low || (self != 0 as u8) >= low),
                self <= (high as u8 as u8),
            )
        }
    }
    impl Promote<bool> for u8 {
        type Max = u8;
        type Lowest = bool;
        type Big = u8;
        type Fast = u16;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: bool) -> (i32, i32) {
            ((self as i32), (r as u8 as i32))
        }
    }
    impl SafeComparison<bool> for u8 {
        fn safe_lt(self, r: bool) -> bool {
            IsLess(self, (r as u8 as u8))
        }
        fn safe_le(self, r: bool) -> bool {
            IsLessOrEqual(self, (r as u8 as u8))
        }
        fn safe_gt(self, r: bool) -> bool {
            IsGreater(self, (r as u8 as u8))
        }
        fn safe_ge(self, r: bool) -> bool {
            IsGreaterOrEqual(self, (r as u8 as u8))
        }
        fn safe_eq(self, r: bool) -> bool {
            IsEqual(self, (r as u8 as u8))
        }
        fn safe_ne(self, r: bool) -> bool {
            IsNotEqual(self, (r as u8 as u8))
        }
    }
    impl Cast<bool> for i16 {
        const CONTAINED: bool = false;
        fn cast(self) -> bool {
            (self != 0 as i16)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, self <= 1 as i16)
        }
    }
    impl RangeWith<bool> for i16 {
        fn range_with(self, low: bool, high: bool) -> RangeCheck {
            RangeCheck::new(
                (self >= 0) && (!low || (self != 0 as i16) >= low),
                self <= (high as u8 as i16),
            )
        }
    }
    impl Promote<bool> for i16 {
        type Max = i16;
        type Lowest = i16;
        type Big = i16;
        type Fast = i32;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: bool) -> (i32, i32) {
            ((self as i32), (r as u8 as i32))
        }
    }
    impl SafeComparison<bool> for i16 {
        fn safe_lt(self, r: bool) -> bool {
            IsLess(self, (r as u8 as i16))
        }
        fn safe_le(self, r: bool) -> bool {
            IsLessOrEqual(self, (r as u8 as i16))
        }
        fn safe_gt(self, r: bool) -> bool {
            IsGreater(self, (r as u8 as i16))
        }
        fn safe_ge(self, r: bool) -> bool {
            IsGreaterOrEqual(self, (r as u8 as i16))
        }
        fn safe_eq(self, r: bool) -> bool {
            IsEqual(self, (r as u8 as i16))
        }
        fn safe_ne(self, r: bool) -> bool {
            IsNotEqual(self, (r as u8 as i16))
        }
    }
    impl Cast<bool> for u16 {
        const CONTAINED: bool = false;
        fn cast(self) -> bool {
            (self != 0 as u16)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= 1 as u16)
        }
    }
    impl RangeWith<bool> for u16 {
        fn range_with(self, low: bool, high: bool) -> RangeCheck {
            RangeCheck::new(
                (true) && (!low || (self != 0 as u16) >= low),
                self <= (high as u8 as u16),
            )
        }
    }
    impl Promote<bool> for u16 {
        type Max = u16;
        type Lowest = bool;
        type Big = u16;
        type Fast = u32;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: bool) -> (i32, i32) {
            ((self as i32), (r as u8 as i32))
        }
    }
    impl SafeComparison<bool> for u16 {
        fn safe_lt(self, r: bool) -> bool {
            IsLess(self, (r as u8 as u16))
        }
        fn safe_le(self, r: bool) -> bool {
            IsLessOrEqual(self, (r as u8 as u16))
        }
        fn safe_gt(self, r: bool) -> bool {
            IsGreater(self, (r as u8 as u16))
        }
        fn safe_ge(self, r: bool) -> bool {
            IsGreaterOrEqual(self, (r as u8 as u16))
        }
        fn safe_eq(self, r: bool) -> bool {
            IsEqual(self, (r as u8 as u16))
        }
        fn safe_ne(self, r: bool) -> bool {
            IsNotEqual(self, (r as u8 as u16))
        }
    }
    impl Cast<bool> for i32 {
        const CONTAINED: bool = false;
        fn cast(self) -> bool {
            (self != 0 as i32)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, self <= 1 as i32)
        }
    }
    impl RangeWith<bool> for i32 {
        fn range_with(self, low: bool, high: bool) -> RangeCheck {
            RangeCheck::new(
                (self >= 0) && (!low || (self != 0 as i32) >= low),
                self <= (high as u8 as i32),
            )
        }
    }
    impl Promote<bool> for i32 {
        type Max = i32;
        type Lowest = i32;
        type Big = i32;
        type Fast = i64;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: bool) -> (i32, i32) {
            (self, (r as u8 as i32))
        }
    }
    impl SafeComparison<bool> for i32 {
        fn safe_lt(self, r: bool) -> bool {
            IsLess(self, (r as u8 as i32))
        }
        fn safe_le(self, r: bool) -> bool {
            IsLessOrEqual(self, (r as u8 as i32))
        }
        fn safe_gt(self, r: bool) -> bool {
            IsGreater(self, (r as u8 as i32))
        }
        fn safe_ge(self, r: bool) -> bool {
            IsGreaterOrEqual(self, (r as u8 as i32))
        }
        fn safe_eq(self, r: bool) -> bool {
            IsEqual(self, (r as u8 as i32))
        }
        fn safe_ne(self, r: bool) -> bool {
            IsNotEqual(self, (r as u8 as i32))
        }
    }
    impl Cast<bool> for u32 {
        const CONTAINED: bool = false;
        fn cast(self) -> bool {
            (self != 0 as u32)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= 1 as u32)
        }
    }
    impl RangeWith<bool> for u32 {
        fn range_with(self, low: bool, high: bool) -> RangeCheck {
            RangeCheck::new(
                (true) && (!low || (self != 0 as u32) >= low),
                self <= (high as u8 as u32),
            )
        }
    }
    impl Promote<bool> for u32 {
        type Max = u32;
        type Lowest = bool;
        type Big = u32;
        type Fast = u64;
        type Sum = u32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: bool) -> (u32, u32) {
            (self, (r as u8 as u32))
        }
    }
    impl SafeComparison<bool> for u32 {
        fn safe_lt(self, r: bool) -> bool {
            IsLess(self, (r as u8 as u32))
        }
        fn safe_le(self, r: bool) -> bool {
            IsLessOrEqual(self, (r as u8 as u32))
        }
        fn safe_gt(self, r: bool) -> bool {
            IsGreater(self, (r as u8 as u32))
        }
        fn safe_ge(self, r: bool) -> bool {
            IsGreaterOrEqual(self, (r as u8 as u32))
        }
        fn safe_eq(self, r: bool) -> bool {
            IsEqual(self, (r as u8 as u32))
        }
        fn safe_ne(self, r: bool) -> bool {
            IsNotEqual(self, (r as u8 as u32))
        }
    }
    impl Cast<bool> for i64 {
        const CONTAINED: bool = false;
        fn cast(self) -> bool {
            (self != 0 as i64)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, self <= 1 as i64)
        }
    }
    impl RangeWith<bool> for i64 {
        fn range_with(self, low: bool, high: bool) -> RangeCheck {
            RangeCheck::new(
                (self >= 0) && (!low || (self != 0 as i64) >= low),
                self <= (high as u8 as i64),
            )
        }
    }
    impl Promote<bool> for i64 {
        type Max = i64;
        type Lowest = i64;
        type Big = i64;
        type Fast = i64;
        type Sum = i64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: bool) -> (i64, i64) {
            (self, (r as u8 as i64))
        }
    }
    impl SafeComparison<bool> for i64 {
        fn safe_lt(self, r: bool) -> bool {
            IsLess(self, (r as u8 as i64))
        }
        fn safe_le(self, r: bool) -> bool {
            IsLessOrEqual(self, (r as u8 as i64))
        }
        fn safe_gt(self, r: bool) -> bool {
            IsGreater(self, (r as u8 as i64))
        }
        fn safe_ge(self, r: bool) -> bool {
            IsGreaterOrEqual(self, (r as u8 as i64))
        }
        fn safe_eq(self, r: bool) -> bool {
            IsEqual(self, (r as u8 as i64))
        }
        fn safe_ne(self, r: bool) -> bool {
            IsNotEqual(self, (r as u8 as i64))
        }
    }
    impl Cast<bool> for u64 {
        const CONTAINED: bool = false;
        fn cast(self) -> bool {
            (self != 0 as u64)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= 1 as u64)
        }
    }
    impl RangeWith<bool> for u64 {
        fn range_with(self, low: bool, high: bool) -> RangeCheck {
            RangeCheck::new(
                (true) && (!low || (self != 0 as u64) >= low),
                self <= (high as u8 as u64),
            )
        }
    }
    impl Promote<bool> for u64 {
        type Max = u64;
        type Lowest = bool;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: bool) -> (u64, u64) {
            (self, (r as u8 as u64))
        }
    }
    impl SafeComparison<bool> for u64 {
        fn safe_lt(self, r: bool) -> bool {
            IsLess(self, (r as u8 as u64))
        }
        fn safe_le(self, r: bool) -> bool {
            IsLessOrEqual(self, (r as u8 as u64))
        }
        fn safe_gt(self, r: bool) -> bool {
            IsGreater(self, (r as u8 as u64))
        }
        fn safe_ge(self, r: bool) -> bool {
            IsGreaterOrEqual(self, (r as u8 as u64))
        }
        fn safe_eq(self, r: bool) -> bool {
            IsEqual(self, (r as u8 as u64))
        }
        fn safe_ne(self, r: bool) -> bool {
            IsNotEqual(self, (r as u8 as u64))
        }
    }
    impl Cast<bool> for f32 {
        const CONTAINED: bool = false;
        fn cast(self) -> bool {
            (self != 0 as f32)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self > -1.0, self <= 1 as f32)
        }
    }
    impl RangeWith<bool> for f32 {
        fn range_with(self, low: bool, high: bool) -> RangeCheck {
            RangeCheck::new(
                (self > -1.0) && (!low || (self != 0 as f32) >= low),
                self <= (high as u8 as f32),
            )
        }
    }
    impl Promote<bool> for f32 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: bool) -> (f32, f32) {
            (self, (r as u8 as f32))
        }
    }
    impl SafeComparison<bool> for f32 {
        fn safe_lt(self, r: bool) -> bool {
            IsLess(self, (r as u8 as f32))
        }
        fn safe_le(self, r: bool) -> bool {
            IsLessOrEqual(self, (r as u8 as f32))
        }
        fn safe_gt(self, r: bool) -> bool {
            IsGreater(self, (r as u8 as f32))
        }
        fn safe_ge(self, r: bool) -> bool {
            IsGreaterOrEqual(self, (r as u8 as f32))
        }
        fn safe_eq(self, r: bool) -> bool {
            IsEqual(self, (r as u8 as f32))
        }
        fn safe_ne(self, r: bool) -> bool {
            IsNotEqual(self, (r as u8 as f32))
        }
    }
    impl Cast<bool> for f64 {
        const CONTAINED: bool = false;
        fn cast(self) -> bool {
            (self != 0 as f64)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self > -1.0, self <= 1 as f64)
        }
    }
    impl RangeWith<bool> for f64 {
        fn range_with(self, low: bool, high: bool) -> RangeCheck {
            RangeCheck::new(
                (self > -1.0) && (!low || (self != 0 as f64) >= low),
                self <= (high as u8 as f64),
            )
        }
    }
    impl Promote<bool> for f64 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: bool) -> (f64, f64) {
            (self, (r as u8 as f64))
        }
    }
    impl SafeComparison<bool> for f64 {
        fn safe_lt(self, r: bool) -> bool {
            IsLess(self, (r as u8 as f64))
        }
        fn safe_le(self, r: bool) -> bool {
            IsLessOrEqual(self, (r as u8 as f64))
        }
        fn safe_gt(self, r: bool) -> bool {
            IsGreater(self, (r as u8 as f64))
        }
        fn safe_ge(self, r: bool) -> bool {
            IsGreaterOrEqual(self, (r as u8 as f64))
        }
        fn safe_eq(self, r: bool) -> bool {
            IsEqual(self, (r as u8 as f64))
        }
        fn safe_ne(self, r: bool) -> bool {
            IsNotEqual(self, (r as u8 as f64))
        }
    }
    impl Cast<bool> for isize {
        const CONTAINED: bool = false;
        fn cast(self) -> bool {
            (self != 0 as isize)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, self <= 1 as isize)
        }
    }
    impl RangeWith<bool> for isize {
        fn range_with(self, low: bool, high: bool) -> RangeCheck {
            RangeCheck::new(
                (self >= 0) && (!low || (self != 0 as isize) >= low),
                self <= (high as u8 as isize),
            )
        }
    }
    impl Promote<bool> for isize {
        type Max = isize;
        type Lowest = isize;
        type Big = isize;
        type Fast = i64;
        type Sum = isize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: bool) -> (isize, isize) {
            (self, (r as u8 as isize))
        }
    }
    impl SafeComparison<bool> for isize {
        fn safe_lt(self, r: bool) -> bool {
            IsLess(self, (r as u8 as isize))
        }
        fn safe_le(self, r: bool) -> bool {
            IsLessOrEqual(self, (r as u8 as isize))
        }
        fn safe_gt(self, r: bool) -> bool {
            IsGreater(self, (r as u8 as isize))
        }
        fn safe_ge(self, r: bool) -> bool {
            IsGreaterOrEqual(self, (r as u8 as isize))
        }
        fn safe_eq(self, r: bool) -> bool {
            IsEqual(self, (r as u8 as isize))
        }
        fn safe_ne(self, r: bool) -> bool {
            IsNotEqual(self, (r as u8 as isize))
        }
    }
    impl Cast<bool> for usize {
        const CONTAINED: bool = false;
        fn cast(self) -> bool {
            (self != 0 as usize)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= 1 as usize)
        }
    }
    impl RangeWith<bool> for usize {
        fn range_with(self, low: bool, high: bool) -> RangeCheck {
            RangeCheck::new(
                (true) && (!low || (self != 0 as usize) >= low),
                self <= (high as u8 as usize),
            )
        }
    }
    impl Promote<bool> for usize {
        type Max = usize;
        type Lowest = bool;
        type Big = usize;
        type Fast = u64;
        type Sum = usize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: bool) -> (usize, usize) {
            (self, (r as u8 as usize))
        }
    }
    impl SafeComparison<bool> for usize {
        fn safe_lt(self, r: bool) -> bool {
            IsLess(self, (r as u8 as usize))
        }
        fn safe_le(self, r: bool) -> bool {
            IsLessOrEqual(self, (r as u8 as usize))
        }
        fn safe_gt(self, r: bool) -> bool {
            IsGreater(self, (r as u8 as usize))
        }
        fn safe_ge(self, r: bool) -> bool {
            IsGreaterOrEqual(self, (r as u8 as usize))
        }
        fn safe_eq(self, r: bool) -> bool {
            IsEqual(self, (r as u8 as usize))
        }
        fn safe_ne(self, r: bool) -> bool {
            IsNotEqual(self, (r as u8 as usize))
        }
    }
    impl Cast<i8> for bool {
        const CONTAINED: bool = true;
        fn cast(self) -> i8 {
            (self as u8 as i8)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<i8> for bool {}
    impl RangeWith<i8> for bool {
        fn range_with(self, low: i8, high: i8) -> RangeCheck {
            RangeCheck::new(
                0 as i8 >= low || (self as u8 as i8) >= low,
                1 as i8 <= high || (self as u8 as i8) <= high,
            )
        }
    }
    impl Promote<i8> for bool {
        type Max = i8;
        type Lowest = i8;
        type Big = i8;
        type Fast = i16;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i8) -> (i32, i32) {
            ((self as u8 as i32), (r as i32))
        }
    }
    impl SafeComparison<i8> for bool {
        fn safe_lt(self, r: i8) -> bool {
            IsLess((self as u8 as i8), r)
        }
        fn safe_le(self, r: i8) -> bool {
            IsLessOrEqual((self as u8 as i8), r)
        }
        fn safe_gt(self, r: i8) -> bool {
            IsGreater((self as u8 as i8), r)
        }
        fn safe_ge(self, r: i8) -> bool {
            IsGreaterOrEqual((self as u8 as i8), r)
        }
        fn safe_eq(self, r: i8) -> bool {
            IsEqual((self as u8 as i8), r)
        }
        fn safe_ne(self, r: i8) -> bool {
            IsNotEqual((self as u8 as i8), r)
        }
    }
    impl Cast<u8> for bool {
        const CONTAINED: bool = true;
        fn cast(self) -> u8 {
            (self as u8 as u8)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<u8> for bool {}
    impl RangeWith<u8> for bool {
        fn range_with(self, low: u8, high: u8) -> RangeCheck {
            RangeCheck::new(
                0 as u8 >= low || (self as u8 as u8) >= low,
                1 as u8 <= high || (self as u8 as u8) <= high,
            )
        }
    }
    impl Promote<u8> for bool {
        type Max = u8;
        type Lowest = bool;
        type Big = u8;
        type Fast = u16;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u8) -> (i32, i32) {
            ((self as u8 as i32), (r as i32))
        }
    }
    impl SafeComparison<u8> for bool {
        fn safe_lt(self, r: u8) -> bool {
            IsLess((self as u8 as u8), r)
        }
        fn safe_le(self, r: u8) -> bool {
            IsLessOrEqual((self as u8 as u8), r)
        }
        fn safe_gt(self, r: u8) -> bool {
            IsGreater((self as u8 as u8), r)
        }
        fn safe_ge(self, r: u8) -> bool {
            IsGreaterOrEqual((self as u8 as u8), r)
        }
        fn safe_eq(self, r: u8) -> bool {
            IsEqual((self as u8 as u8), r)
        }
        fn safe_ne(self, r: u8) -> bool {
            IsNotEqual((self as u8 as u8), r)
        }
    }
    impl Cast<i16> for bool {
        const CONTAINED: bool = true;
        fn cast(self) -> i16 {
            (self as u8 as i16)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<i16> for bool {}
    impl RangeWith<i16> for bool {
        fn range_with(self, low: i16, high: i16) -> RangeCheck {
            RangeCheck::new(
                0 as i16 >= low || (self as u8 as i16) >= low,
                1 as i16 <= high || (self as u8 as i16) <= high,
            )
        }
    }
    impl Promote<i16> for bool {
        type Max = i16;
        type Lowest = i16;
        type Big = i16;
        type Fast = i32;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i16) -> (i32, i32) {
            ((self as u8 as i32), (r as i32))
        }
    }
    impl SafeComparison<i16> for bool {
        fn safe_lt(self, r: i16) -> bool {
            IsLess((self as u8 as i16), r)
        }
        fn safe_le(self, r: i16) -> bool {
            IsLessOrEqual((self as u8 as i16), r)
        }
        fn safe_gt(self, r: i16) -> bool {
            IsGreater((self as u8 as i16), r)
        }
        fn safe_ge(self, r: i16) -> bool {
            IsGreaterOrEqual((self as u8 as i16), r)
        }
        fn safe_eq(self, r: i16) -> bool {
            IsEqual((self as u8 as i16), r)
        }
        fn safe_ne(self, r: i16) -> bool {
            IsNotEqual((self as u8 as i16), r)
        }
    }
    impl Cast<u16> for bool {
        const CONTAINED: bool = true;
        fn cast(self) -> u16 {
            (self as u8 as u16)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<u16> for bool {}
    impl RangeWith<u16> for bool {
        fn range_with(self, low: u16, high: u16) -> RangeCheck {
            RangeCheck::new(
                0 as u16 >= low || (self as u8 as u16) >= low,
                1 as u16 <= high || (self as u8 as u16) <= high,
            )
        }
    }
    impl Promote<u16> for bool {
        type Max = u16;
        type Lowest = bool;
        type Big = u16;
        type Fast = u32;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u16) -> (i32, i32) {
            ((self as u8 as i32), (r as i32))
        }
    }
    impl SafeComparison<u16> for bool {
        fn safe_lt(self, r: u16) -> bool {
            IsLess((self as u8 as u16), r)
        }
        fn safe_le(self, r: u16) -> bool {
            IsLessOrEqual((self as u8 as u16), r)
        }
        fn safe_gt(self, r: u16) -> bool {
            IsGreater((self as u8 as u16), r)
        }
        fn safe_ge(self, r: u16) -> bool {
            IsGreaterOrEqual((self as u8 as u16), r)
        }
        fn safe_eq(self, r: u16) -> bool {
            IsEqual((self as u8 as u16), r)
        }
        fn safe_ne(self, r: u16) -> bool {
            IsNotEqual((self as u8 as u16), r)
        }
    }
    impl Cast<i32> for bool {
        const CONTAINED: bool = true;
        fn cast(self) -> i32 {
            (self as u8 as i32)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<i32> for bool {}
    impl RangeWith<i32> for bool {
        fn range_with(self, low: i32, high: i32) -> RangeCheck {
            RangeCheck::new(
                0 as i32 >= low || (self as u8 as i32) >= low,
                1 as i32 <= high || (self as u8 as i32) <= high,
            )
        }
    }
    impl Promote<i32> for bool {
        type Max = i32;
        type Lowest = i32;
        type Big = i32;
        type Fast = i64;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i32) -> (i32, i32) {
            ((self as u8 as i32), r)
        }
    }
    impl SafeComparison<i32> for bool {
        fn safe_lt(self, r: i32) -> bool {
            IsLess((self as u8 as i32), r)
        }
        fn safe_le(self, r: i32) -> bool {
            IsLessOrEqual((self as u8 as i32), r)
        }
        fn safe_gt(self, r: i32) -> bool {
            IsGreater((self as u8 as i32), r)
        }
        fn safe_ge(self, r: i32) -> bool {
            IsGreaterOrEqual((self as u8 as i32), r)
        }
        fn safe_eq(self, r: i32) -> bool {
            IsEqual((self as u8 as i32), r)
        }
        fn safe_ne(self, r: i32) -> bool {
            IsNotEqual((self as u8 as i32), r)
        }
    }
    impl Cast<u32> for bool {
        const CONTAINED: bool = true;
        fn cast(self) -> u32 {
            (self as u8 as u32)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<u32> for bool {}
    impl RangeWith<u32> for bool {
        fn range_with(self, low: u32, high: u32) -> RangeCheck {
            RangeCheck::new(
                0 as u32 >= low || (self as u8 as u32) >= low,
                1 as u32 <= high || (self as u8 as u32) <= high,
            )
        }
    }
    impl Promote<u32> for bool {
        type Max = u32;
        type Lowest = bool;
        type Big = u32;
        type Fast = u64;
        type Sum = u32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u32) -> (u32, u32) {
            ((self as u8 as u32), r)
        }
    }
    impl SafeComparison<u32> for bool {
        fn safe_lt(self, r: u32) -> bool {
            IsLess((self as u8 as u32), r)
        }
        fn safe_le(self, r: u32) -> bool {
            IsLessOrEqual((self as u8 as u32), r)
        }
        fn safe_gt(self, r: u32) -> bool {
            IsGreater((self as u8 as u32), r)
        }
        fn safe_ge(self, r: u32) -> bool {
            IsGreaterOrEqual((self as u8 as u32), r)
        }
        fn safe_eq(self, r: u32) -> bool {
            IsEqual((self as u8 as u32), r)
        }
        fn safe_ne(self, r: u32) -> bool {
            IsNotEqual((self as u8 as u32), r)
        }
    }
    impl Cast<i64> for bool {
        const CONTAINED: bool = true;
        fn cast(self) -> i64 {
            (self as u8 as i64)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<i64> for bool {}
    impl RangeWith<i64> for bool {
        fn range_with(self, low: i64, high: i64) -> RangeCheck {
            RangeCheck::new(
                0 as i64 >= low || (self as u8 as i64) >= low,
                1 as i64 <= high || (self as u8 as i64) <= high,
            )
        }
    }
    impl Promote<i64> for bool {
        type Max = i64;
        type Lowest = i64;
        type Big = i64;
        type Fast = i64;
        type Sum = i64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i64) -> (i64, i64) {
            ((self as u8 as i64), r)
        }
    }
    impl SafeComparison<i64> for bool {
        fn safe_lt(self, r: i64) -> bool {
            IsLess((self as u8 as i64), r)
        }
        fn safe_le(self, r: i64) -> bool {
            IsLessOrEqual((self as u8 as i64), r)
        }
        fn safe_gt(self, r: i64) -> bool {
            IsGreater((self as u8 as i64), r)
        }
        fn safe_ge(self, r: i64) -> bool {
            IsGreaterOrEqual((self as u8 as i64), r)
        }
        fn safe_eq(self, r: i64) -> bool {
            IsEqual((self as u8 as i64), r)
        }
        fn safe_ne(self, r: i64) -> bool {
            IsNotEqual((self as u8 as i64), r)
        }
    }
    impl Cast<u64> for bool {
        const CONTAINED: bool = true;
        fn cast(self) -> u64 {
            (self as u8 as u64)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<u64> for bool {}
    impl RangeWith<u64> for bool {
        fn range_with(self, low: u64, high: u64) -> RangeCheck {
            RangeCheck::new(
                0 as u64 >= low || (self as u8 as u64) >= low,
                1 as u64 <= high || (self as u8 as u64) <= high,
            )
        }
    }
    impl Promote<u64> for bool {
        type Max = u64;
        type Lowest = bool;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u64) -> (u64, u64) {
            ((self as u8 as u64), r)
        }
    }
    impl SafeComparison<u64> for bool {
        fn safe_lt(self, r: u64) -> bool {
            IsLess((self as u8 as u64), r)
        }
        fn safe_le(self, r: u64) -> bool {
            IsLessOrEqual((self as u8 as u64), r)
        }
        fn safe_gt(self, r: u64) -> bool {
            IsGreater((self as u8 as u64), r)
        }
        fn safe_ge(self, r: u64) -> bool {
            IsGreaterOrEqual((self as u8 as u64), r)
        }
        fn safe_eq(self, r: u64) -> bool {
            IsEqual((self as u8 as u64), r)
        }
        fn safe_ne(self, r: u64) -> bool {
            IsNotEqual((self as u8 as u64), r)
        }
    }
    impl Cast<f32> for bool {
        const CONTAINED: bool = true;
        fn cast(self) -> f32 {
            (self as u8 as f32)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f32> for bool {}
    impl RangeWith<f32> for bool {
        fn range_with(self, low: f32, high: f32) -> RangeCheck {
            RangeCheck::new(
                0 as f32 >= low || (self as u8 as f32) >= low,
                1 as f32 <= high || (self as u8 as f32) <= high,
            )
        }
    }
    impl Promote<f32> for bool {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f32) -> (f32, f32) {
            ((self as u8 as f32), r)
        }
    }
    impl SafeComparison<f32> for bool {
        fn safe_lt(self, r: f32) -> bool {
            IsLess((self as u8 as f32), r)
        }
        fn safe_le(self, r: f32) -> bool {
            IsLessOrEqual((self as u8 as f32), r)
        }
        fn safe_gt(self, r: f32) -> bool {
            IsGreater((self as u8 as f32), r)
        }
        fn safe_ge(self, r: f32) -> bool {
            IsGreaterOrEqual((self as u8 as f32), r)
        }
        fn safe_eq(self, r: f32) -> bool {
            IsEqual((self as u8 as f32), r)
        }
        fn safe_ne(self, r: f32) -> bool {
            IsNotEqual((self as u8 as f32), r)
        }
    }
    impl Cast<f64> for bool {
        const CONTAINED: bool = true;
        fn cast(self) -> f64 {
            (self as u8 as f64)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f64> for bool {}
    impl RangeWith<f64> for bool {
        fn range_with(self, low: f64, high: f64) -> RangeCheck {
            RangeCheck::new(
                0 as f64 >= low || (self as u8 as f64) >= low,
                1 as f64 <= high || (self as u8 as f64) <= high,
            )
        }
    }
    impl Promote<f64> for bool {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f64) -> (f64, f64) {
            ((self as u8 as f64), r)
        }
    }
    impl SafeComparison<f64> for bool {
        fn safe_lt(self, r: f64) -> bool {
            IsLess((self as u8 as f64), r)
        }
        fn safe_le(self, r: f64) -> bool {
            IsLessOrEqual((self as u8 as f64), r)
        }
        fn safe_gt(self, r: f64) -> bool {
            IsGreater((self as u8 as f64), r)
        }
        fn safe_ge(self, r: f64) -> bool {
            IsGreaterOrEqual((self as u8 as f64), r)
        }
        fn safe_eq(self, r: f64) -> bool {
            IsEqual((self as u8 as f64), r)
        }
        fn safe_ne(self, r: f64) -> bool {
            IsNotEqual((self as u8 as f64), r)
        }
    }
    impl Cast<isize> for bool {
        const CONTAINED: bool = true;
        fn cast(self) -> isize {
            (self as u8 as isize)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<isize> for bool {}
    impl RangeWith<isize> for bool {
        fn range_with(self, low: isize, high: isize) -> RangeCheck {
            RangeCheck::new(
                0 as isize >= low || (self as u8 as isize) >= low,
                1 as isize <= high || (self as u8 as isize) <= high,
            )
        }
    }
    impl Promote<isize> for bool {
        type Max = isize;
        type Lowest = isize;
        type Big = isize;
        type Fast = i64;
        type Sum = isize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: isize) -> (isize, isize) {
            ((self as u8 as isize), r)
        }
    }
    impl SafeComparison<isize> for bool {
        fn safe_lt(self, r: isize) -> bool {
            IsLess((self as u8 as isize), r)
        }
        fn safe_le(self, r: isize) -> bool {
            IsLessOrEqual((self as u8 as isize), r)
        }
        fn safe_gt(self, r: isize) -> bool {
            IsGreater((self as u8 as isize), r)
        }
        fn safe_ge(self, r: isize) -> bool {
            IsGreaterOrEqual((self as u8 as isize), r)
        }
        fn safe_eq(self, r: isize) -> bool {
            IsEqual((self as u8 as isize), r)
        }
        fn safe_ne(self, r: isize) -> bool {
            IsNotEqual((self as u8 as isize), r)
        }
    }
    impl Cast<usize> for bool {
        const CONTAINED: bool = true;
        fn cast(self) -> usize {
            (self as u8 as usize)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<usize> for bool {}
    impl RangeWith<usize> for bool {
        fn range_with(self, low: usize, high: usize) -> RangeCheck {
            RangeCheck::new(
                0 as usize >= low || (self as u8 as usize) >= low,
                1 as usize <= high || (self as u8 as usize) <= high,
            )
        }
    }
    impl Promote<usize> for bool {
        type Max = usize;
        type Lowest = bool;
        type Big = usize;
        type Fast = u64;
        type Sum = usize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: usize) -> (usize, usize) {
            ((self as u8 as usize), r)
        }
    }
    impl SafeComparison<usize> for bool {
        fn safe_lt(self, r: usize) -> bool {
            IsLess((self as u8 as usize), r)
        }
        fn safe_le(self, r: usize) -> bool {
            IsLessOrEqual((self as u8 as usize), r)
        }
        fn safe_gt(self, r: usize) -> bool {
            IsGreater((self as u8 as usize), r)
        }
        fn safe_ge(self, r: usize) -> bool {
            IsGreaterOrEqual((self as u8 as usize), r)
        }
        fn safe_eq(self, r: usize) -> bool {
            IsEqual((self as u8 as usize), r)
        }
        fn safe_ne(self, r: usize) -> bool {
            IsNotEqual((self as u8 as usize), r)
        }
    }
    impl Cast<bool> for bool {
        const CONTAINED: bool = true;
        fn cast(self) -> bool {
            self
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<bool> for bool {}
    impl RangeWith<bool> for bool {
        fn range_with(self, low: bool, high: bool) -> RangeCheck {
            RangeCheck::new(false >= low || self >= low, true <= high || self <= high)
        }
    }
    impl SafeComparison<bool> for bool {
        fn safe_lt(self, r: bool) -> bool {
            (self as i32) < (r as i32)
        }
        fn safe_le(self, r: bool) -> bool {
            (self as i32) <= (r as i32)
        }
        fn safe_gt(self, r: bool) -> bool {
            (self as i32) > (r as i32)
        }
        fn safe_ge(self, r: bool) -> bool {
            (self as i32) >= (r as i32)
        }
        fn safe_eq(self, r: bool) -> bool {
            (self as i32) == (r as i32)
        }
        fn safe_ne(self, r: bool) -> bool {
            (self as i32) != (r as i32)
        }
    }
}
#[cfg(target_pointer_width = "64")]
mod bool_domain {
    use super::*;
    impl super::domain::sealed::Primitive for bool {}
    impl Numeric for bool {
        const ZERO: Self = false;
        const MAX: Self = true;
        const LOWEST: Self = false;
        const SIGNED: bool = false;
        const FLOAT: bool = false;
        const DIGITS: u32 = 1;
        const MAX_EXPONENT: u32 = 2;
        fn negative(self) -> bool {
            false
        }
        fn finite(self) -> bool {
            true
        }
        fn nan() -> Self {
            false
        }
        fn overflow() -> Self {
            true
        }
        fn underflow() -> Self {
            false
        }
    }
    impl Cast<bool> for i8 {
        const CONTAINED: bool = false;
        fn cast(self) -> bool {
            (self != 0 as i8)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, self <= 1 as i8)
        }
    }
    impl RangeWith<bool> for i8 {
        fn range_with(self, low: bool, high: bool) -> RangeCheck {
            RangeCheck::new(
                (self >= 0) && (!low || (self != 0 as i8) >= low),
                self <= (high as u8 as i8),
            )
        }
    }
    impl Promote<bool> for i8 {
        type Max = i8;
        type Lowest = i8;
        type Big = i8;
        type Fast = i16;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: bool) -> (i32, i32) {
            ((self as i32), (r as u8 as i32))
        }
    }
    impl SafeComparison<bool> for i8 {
        fn safe_lt(self, r: bool) -> bool {
            IsLess(self, (r as u8 as i8))
        }
        fn safe_le(self, r: bool) -> bool {
            IsLessOrEqual(self, (r as u8 as i8))
        }
        fn safe_gt(self, r: bool) -> bool {
            IsGreater(self, (r as u8 as i8))
        }
        fn safe_ge(self, r: bool) -> bool {
            IsGreaterOrEqual(self, (r as u8 as i8))
        }
        fn safe_eq(self, r: bool) -> bool {
            IsEqual(self, (r as u8 as i8))
        }
        fn safe_ne(self, r: bool) -> bool {
            IsNotEqual(self, (r as u8 as i8))
        }
    }
    impl Cast<bool> for u8 {
        const CONTAINED: bool = false;
        fn cast(self) -> bool {
            (self != 0 as u8)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= 1 as u8)
        }
    }
    impl RangeWith<bool> for u8 {
        fn range_with(self, low: bool, high: bool) -> RangeCheck {
            RangeCheck::new(
                (true) && (!low || (self != 0 as u8) >= low),
                self <= (high as u8 as u8),
            )
        }
    }
    impl Promote<bool> for u8 {
        type Max = u8;
        type Lowest = bool;
        type Big = u8;
        type Fast = u16;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: bool) -> (i32, i32) {
            ((self as i32), (r as u8 as i32))
        }
    }
    impl SafeComparison<bool> for u8 {
        fn safe_lt(self, r: bool) -> bool {
            IsLess(self, (r as u8 as u8))
        }
        fn safe_le(self, r: bool) -> bool {
            IsLessOrEqual(self, (r as u8 as u8))
        }
        fn safe_gt(self, r: bool) -> bool {
            IsGreater(self, (r as u8 as u8))
        }
        fn safe_ge(self, r: bool) -> bool {
            IsGreaterOrEqual(self, (r as u8 as u8))
        }
        fn safe_eq(self, r: bool) -> bool {
            IsEqual(self, (r as u8 as u8))
        }
        fn safe_ne(self, r: bool) -> bool {
            IsNotEqual(self, (r as u8 as u8))
        }
    }
    impl Cast<bool> for i16 {
        const CONTAINED: bool = false;
        fn cast(self) -> bool {
            (self != 0 as i16)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, self <= 1 as i16)
        }
    }
    impl RangeWith<bool> for i16 {
        fn range_with(self, low: bool, high: bool) -> RangeCheck {
            RangeCheck::new(
                (self >= 0) && (!low || (self != 0 as i16) >= low),
                self <= (high as u8 as i16),
            )
        }
    }
    impl Promote<bool> for i16 {
        type Max = i16;
        type Lowest = i16;
        type Big = i16;
        type Fast = i32;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: bool) -> (i32, i32) {
            ((self as i32), (r as u8 as i32))
        }
    }
    impl SafeComparison<bool> for i16 {
        fn safe_lt(self, r: bool) -> bool {
            IsLess(self, (r as u8 as i16))
        }
        fn safe_le(self, r: bool) -> bool {
            IsLessOrEqual(self, (r as u8 as i16))
        }
        fn safe_gt(self, r: bool) -> bool {
            IsGreater(self, (r as u8 as i16))
        }
        fn safe_ge(self, r: bool) -> bool {
            IsGreaterOrEqual(self, (r as u8 as i16))
        }
        fn safe_eq(self, r: bool) -> bool {
            IsEqual(self, (r as u8 as i16))
        }
        fn safe_ne(self, r: bool) -> bool {
            IsNotEqual(self, (r as u8 as i16))
        }
    }
    impl Cast<bool> for u16 {
        const CONTAINED: bool = false;
        fn cast(self) -> bool {
            (self != 0 as u16)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= 1 as u16)
        }
    }
    impl RangeWith<bool> for u16 {
        fn range_with(self, low: bool, high: bool) -> RangeCheck {
            RangeCheck::new(
                (true) && (!low || (self != 0 as u16) >= low),
                self <= (high as u8 as u16),
            )
        }
    }
    impl Promote<bool> for u16 {
        type Max = u16;
        type Lowest = bool;
        type Big = u16;
        type Fast = u32;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: bool) -> (i32, i32) {
            ((self as i32), (r as u8 as i32))
        }
    }
    impl SafeComparison<bool> for u16 {
        fn safe_lt(self, r: bool) -> bool {
            IsLess(self, (r as u8 as u16))
        }
        fn safe_le(self, r: bool) -> bool {
            IsLessOrEqual(self, (r as u8 as u16))
        }
        fn safe_gt(self, r: bool) -> bool {
            IsGreater(self, (r as u8 as u16))
        }
        fn safe_ge(self, r: bool) -> bool {
            IsGreaterOrEqual(self, (r as u8 as u16))
        }
        fn safe_eq(self, r: bool) -> bool {
            IsEqual(self, (r as u8 as u16))
        }
        fn safe_ne(self, r: bool) -> bool {
            IsNotEqual(self, (r as u8 as u16))
        }
    }
    impl Cast<bool> for i32 {
        const CONTAINED: bool = false;
        fn cast(self) -> bool {
            (self != 0 as i32)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, self <= 1 as i32)
        }
    }
    impl RangeWith<bool> for i32 {
        fn range_with(self, low: bool, high: bool) -> RangeCheck {
            RangeCheck::new(
                (self >= 0) && (!low || (self != 0 as i32) >= low),
                self <= (high as u8 as i32),
            )
        }
    }
    impl Promote<bool> for i32 {
        type Max = i32;
        type Lowest = i32;
        type Big = i32;
        type Fast = i64;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: bool) -> (i32, i32) {
            (self, (r as u8 as i32))
        }
    }
    impl SafeComparison<bool> for i32 {
        fn safe_lt(self, r: bool) -> bool {
            IsLess(self, (r as u8 as i32))
        }
        fn safe_le(self, r: bool) -> bool {
            IsLessOrEqual(self, (r as u8 as i32))
        }
        fn safe_gt(self, r: bool) -> bool {
            IsGreater(self, (r as u8 as i32))
        }
        fn safe_ge(self, r: bool) -> bool {
            IsGreaterOrEqual(self, (r as u8 as i32))
        }
        fn safe_eq(self, r: bool) -> bool {
            IsEqual(self, (r as u8 as i32))
        }
        fn safe_ne(self, r: bool) -> bool {
            IsNotEqual(self, (r as u8 as i32))
        }
    }
    impl Cast<bool> for u32 {
        const CONTAINED: bool = false;
        fn cast(self) -> bool {
            (self != 0 as u32)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= 1 as u32)
        }
    }
    impl RangeWith<bool> for u32 {
        fn range_with(self, low: bool, high: bool) -> RangeCheck {
            RangeCheck::new(
                (true) && (!low || (self != 0 as u32) >= low),
                self <= (high as u8 as u32),
            )
        }
    }
    impl Promote<bool> for u32 {
        type Max = u32;
        type Lowest = bool;
        type Big = u32;
        type Fast = u64;
        type Sum = u32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: bool) -> (u32, u32) {
            (self, (r as u8 as u32))
        }
    }
    impl SafeComparison<bool> for u32 {
        fn safe_lt(self, r: bool) -> bool {
            IsLess(self, (r as u8 as u32))
        }
        fn safe_le(self, r: bool) -> bool {
            IsLessOrEqual(self, (r as u8 as u32))
        }
        fn safe_gt(self, r: bool) -> bool {
            IsGreater(self, (r as u8 as u32))
        }
        fn safe_ge(self, r: bool) -> bool {
            IsGreaterOrEqual(self, (r as u8 as u32))
        }
        fn safe_eq(self, r: bool) -> bool {
            IsEqual(self, (r as u8 as u32))
        }
        fn safe_ne(self, r: bool) -> bool {
            IsNotEqual(self, (r as u8 as u32))
        }
    }
    impl Cast<bool> for i64 {
        const CONTAINED: bool = false;
        fn cast(self) -> bool {
            (self != 0 as i64)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, self <= 1 as i64)
        }
    }
    impl RangeWith<bool> for i64 {
        fn range_with(self, low: bool, high: bool) -> RangeCheck {
            RangeCheck::new(
                (self >= 0) && (!low || (self != 0 as i64) >= low),
                self <= (high as u8 as i64),
            )
        }
    }
    impl Promote<bool> for i64 {
        type Max = i64;
        type Lowest = i64;
        type Big = i64;
        type Fast = i64;
        type Sum = i64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: bool) -> (i64, i64) {
            (self, (r as u8 as i64))
        }
    }
    impl SafeComparison<bool> for i64 {
        fn safe_lt(self, r: bool) -> bool {
            IsLess(self, (r as u8 as i64))
        }
        fn safe_le(self, r: bool) -> bool {
            IsLessOrEqual(self, (r as u8 as i64))
        }
        fn safe_gt(self, r: bool) -> bool {
            IsGreater(self, (r as u8 as i64))
        }
        fn safe_ge(self, r: bool) -> bool {
            IsGreaterOrEqual(self, (r as u8 as i64))
        }
        fn safe_eq(self, r: bool) -> bool {
            IsEqual(self, (r as u8 as i64))
        }
        fn safe_ne(self, r: bool) -> bool {
            IsNotEqual(self, (r as u8 as i64))
        }
    }
    impl Cast<bool> for u64 {
        const CONTAINED: bool = false;
        fn cast(self) -> bool {
            (self != 0 as u64)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= 1 as u64)
        }
    }
    impl RangeWith<bool> for u64 {
        fn range_with(self, low: bool, high: bool) -> RangeCheck {
            RangeCheck::new(
                (true) && (!low || (self != 0 as u64) >= low),
                self <= (high as u8 as u64),
            )
        }
    }
    impl Promote<bool> for u64 {
        type Max = u64;
        type Lowest = bool;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: bool) -> (u64, u64) {
            (self, (r as u8 as u64))
        }
    }
    impl SafeComparison<bool> for u64 {
        fn safe_lt(self, r: bool) -> bool {
            IsLess(self, (r as u8 as u64))
        }
        fn safe_le(self, r: bool) -> bool {
            IsLessOrEqual(self, (r as u8 as u64))
        }
        fn safe_gt(self, r: bool) -> bool {
            IsGreater(self, (r as u8 as u64))
        }
        fn safe_ge(self, r: bool) -> bool {
            IsGreaterOrEqual(self, (r as u8 as u64))
        }
        fn safe_eq(self, r: bool) -> bool {
            IsEqual(self, (r as u8 as u64))
        }
        fn safe_ne(self, r: bool) -> bool {
            IsNotEqual(self, (r as u8 as u64))
        }
    }
    impl Cast<bool> for f32 {
        const CONTAINED: bool = false;
        fn cast(self) -> bool {
            (self != 0 as f32)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self > -1.0, self <= 1 as f32)
        }
    }
    impl RangeWith<bool> for f32 {
        fn range_with(self, low: bool, high: bool) -> RangeCheck {
            RangeCheck::new(
                (self > -1.0) && (!low || (self != 0 as f32) >= low),
                self <= (high as u8 as f32),
            )
        }
    }
    impl Promote<bool> for f32 {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: bool) -> (f32, f32) {
            (self, (r as u8 as f32))
        }
    }
    impl SafeComparison<bool> for f32 {
        fn safe_lt(self, r: bool) -> bool {
            IsLess(self, (r as u8 as f32))
        }
        fn safe_le(self, r: bool) -> bool {
            IsLessOrEqual(self, (r as u8 as f32))
        }
        fn safe_gt(self, r: bool) -> bool {
            IsGreater(self, (r as u8 as f32))
        }
        fn safe_ge(self, r: bool) -> bool {
            IsGreaterOrEqual(self, (r as u8 as f32))
        }
        fn safe_eq(self, r: bool) -> bool {
            IsEqual(self, (r as u8 as f32))
        }
        fn safe_ne(self, r: bool) -> bool {
            IsNotEqual(self, (r as u8 as f32))
        }
    }
    impl Cast<bool> for f64 {
        const CONTAINED: bool = false;
        fn cast(self) -> bool {
            (self != 0 as f64)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self > -1.0, self <= 1 as f64)
        }
    }
    impl RangeWith<bool> for f64 {
        fn range_with(self, low: bool, high: bool) -> RangeCheck {
            RangeCheck::new(
                (self > -1.0) && (!low || (self != 0 as f64) >= low),
                self <= (high as u8 as f64),
            )
        }
    }
    impl Promote<bool> for f64 {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: bool) -> (f64, f64) {
            (self, (r as u8 as f64))
        }
    }
    impl SafeComparison<bool> for f64 {
        fn safe_lt(self, r: bool) -> bool {
            IsLess(self, (r as u8 as f64))
        }
        fn safe_le(self, r: bool) -> bool {
            IsLessOrEqual(self, (r as u8 as f64))
        }
        fn safe_gt(self, r: bool) -> bool {
            IsGreater(self, (r as u8 as f64))
        }
        fn safe_ge(self, r: bool) -> bool {
            IsGreaterOrEqual(self, (r as u8 as f64))
        }
        fn safe_eq(self, r: bool) -> bool {
            IsEqual(self, (r as u8 as f64))
        }
        fn safe_ne(self, r: bool) -> bool {
            IsNotEqual(self, (r as u8 as f64))
        }
    }
    impl Cast<bool> for isize {
        const CONTAINED: bool = false;
        fn cast(self) -> bool {
            (self != 0 as isize)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(self >= 0, self <= 1 as isize)
        }
    }
    impl RangeWith<bool> for isize {
        fn range_with(self, low: bool, high: bool) -> RangeCheck {
            RangeCheck::new(
                (self >= 0) && (!low || (self != 0 as isize) >= low),
                self <= (high as u8 as isize),
            )
        }
    }
    impl Promote<bool> for isize {
        type Max = isize;
        type Lowest = isize;
        type Big = isize;
        type Fast = isize;
        type Sum = isize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: bool) -> (isize, isize) {
            (self, (r as u8 as isize))
        }
    }
    impl SafeComparison<bool> for isize {
        fn safe_lt(self, r: bool) -> bool {
            IsLess(self, (r as u8 as isize))
        }
        fn safe_le(self, r: bool) -> bool {
            IsLessOrEqual(self, (r as u8 as isize))
        }
        fn safe_gt(self, r: bool) -> bool {
            IsGreater(self, (r as u8 as isize))
        }
        fn safe_ge(self, r: bool) -> bool {
            IsGreaterOrEqual(self, (r as u8 as isize))
        }
        fn safe_eq(self, r: bool) -> bool {
            IsEqual(self, (r as u8 as isize))
        }
        fn safe_ne(self, r: bool) -> bool {
            IsNotEqual(self, (r as u8 as isize))
        }
    }
    impl Cast<bool> for usize {
        const CONTAINED: bool = false;
        fn cast(self) -> bool {
            (self != 0 as usize)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, self <= 1 as usize)
        }
    }
    impl RangeWith<bool> for usize {
        fn range_with(self, low: bool, high: bool) -> RangeCheck {
            RangeCheck::new(
                (true) && (!low || (self != 0 as usize) >= low),
                self <= (high as u8 as usize),
            )
        }
    }
    impl Promote<bool> for usize {
        type Max = usize;
        type Lowest = bool;
        type Big = usize;
        type Fast = usize;
        type Sum = usize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: bool) -> (usize, usize) {
            (self, (r as u8 as usize))
        }
    }
    impl SafeComparison<bool> for usize {
        fn safe_lt(self, r: bool) -> bool {
            IsLess(self, (r as u8 as usize))
        }
        fn safe_le(self, r: bool) -> bool {
            IsLessOrEqual(self, (r as u8 as usize))
        }
        fn safe_gt(self, r: bool) -> bool {
            IsGreater(self, (r as u8 as usize))
        }
        fn safe_ge(self, r: bool) -> bool {
            IsGreaterOrEqual(self, (r as u8 as usize))
        }
        fn safe_eq(self, r: bool) -> bool {
            IsEqual(self, (r as u8 as usize))
        }
        fn safe_ne(self, r: bool) -> bool {
            IsNotEqual(self, (r as u8 as usize))
        }
    }
    impl Cast<i8> for bool {
        const CONTAINED: bool = true;
        fn cast(self) -> i8 {
            (self as u8 as i8)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<i8> for bool {}
    impl RangeWith<i8> for bool {
        fn range_with(self, low: i8, high: i8) -> RangeCheck {
            RangeCheck::new(
                0 as i8 >= low || (self as u8 as i8) >= low,
                1 as i8 <= high || (self as u8 as i8) <= high,
            )
        }
    }
    impl Promote<i8> for bool {
        type Max = i8;
        type Lowest = i8;
        type Big = i8;
        type Fast = i16;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i8) -> (i32, i32) {
            ((self as u8 as i32), (r as i32))
        }
    }
    impl SafeComparison<i8> for bool {
        fn safe_lt(self, r: i8) -> bool {
            IsLess((self as u8 as i8), r)
        }
        fn safe_le(self, r: i8) -> bool {
            IsLessOrEqual((self as u8 as i8), r)
        }
        fn safe_gt(self, r: i8) -> bool {
            IsGreater((self as u8 as i8), r)
        }
        fn safe_ge(self, r: i8) -> bool {
            IsGreaterOrEqual((self as u8 as i8), r)
        }
        fn safe_eq(self, r: i8) -> bool {
            IsEqual((self as u8 as i8), r)
        }
        fn safe_ne(self, r: i8) -> bool {
            IsNotEqual((self as u8 as i8), r)
        }
    }
    impl Cast<u8> for bool {
        const CONTAINED: bool = true;
        fn cast(self) -> u8 {
            (self as u8 as u8)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<u8> for bool {}
    impl RangeWith<u8> for bool {
        fn range_with(self, low: u8, high: u8) -> RangeCheck {
            RangeCheck::new(
                0 as u8 >= low || (self as u8 as u8) >= low,
                1 as u8 <= high || (self as u8 as u8) <= high,
            )
        }
    }
    impl Promote<u8> for bool {
        type Max = u8;
        type Lowest = bool;
        type Big = u8;
        type Fast = u16;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u8) -> (i32, i32) {
            ((self as u8 as i32), (r as i32))
        }
    }
    impl SafeComparison<u8> for bool {
        fn safe_lt(self, r: u8) -> bool {
            IsLess((self as u8 as u8), r)
        }
        fn safe_le(self, r: u8) -> bool {
            IsLessOrEqual((self as u8 as u8), r)
        }
        fn safe_gt(self, r: u8) -> bool {
            IsGreater((self as u8 as u8), r)
        }
        fn safe_ge(self, r: u8) -> bool {
            IsGreaterOrEqual((self as u8 as u8), r)
        }
        fn safe_eq(self, r: u8) -> bool {
            IsEqual((self as u8 as u8), r)
        }
        fn safe_ne(self, r: u8) -> bool {
            IsNotEqual((self as u8 as u8), r)
        }
    }
    impl Cast<i16> for bool {
        const CONTAINED: bool = true;
        fn cast(self) -> i16 {
            (self as u8 as i16)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<i16> for bool {}
    impl RangeWith<i16> for bool {
        fn range_with(self, low: i16, high: i16) -> RangeCheck {
            RangeCheck::new(
                0 as i16 >= low || (self as u8 as i16) >= low,
                1 as i16 <= high || (self as u8 as i16) <= high,
            )
        }
    }
    impl Promote<i16> for bool {
        type Max = i16;
        type Lowest = i16;
        type Big = i16;
        type Fast = i32;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i16) -> (i32, i32) {
            ((self as u8 as i32), (r as i32))
        }
    }
    impl SafeComparison<i16> for bool {
        fn safe_lt(self, r: i16) -> bool {
            IsLess((self as u8 as i16), r)
        }
        fn safe_le(self, r: i16) -> bool {
            IsLessOrEqual((self as u8 as i16), r)
        }
        fn safe_gt(self, r: i16) -> bool {
            IsGreater((self as u8 as i16), r)
        }
        fn safe_ge(self, r: i16) -> bool {
            IsGreaterOrEqual((self as u8 as i16), r)
        }
        fn safe_eq(self, r: i16) -> bool {
            IsEqual((self as u8 as i16), r)
        }
        fn safe_ne(self, r: i16) -> bool {
            IsNotEqual((self as u8 as i16), r)
        }
    }
    impl Cast<u16> for bool {
        const CONTAINED: bool = true;
        fn cast(self) -> u16 {
            (self as u8 as u16)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<u16> for bool {}
    impl RangeWith<u16> for bool {
        fn range_with(self, low: u16, high: u16) -> RangeCheck {
            RangeCheck::new(
                0 as u16 >= low || (self as u8 as u16) >= low,
                1 as u16 <= high || (self as u8 as u16) <= high,
            )
        }
    }
    impl Promote<u16> for bool {
        type Max = u16;
        type Lowest = bool;
        type Big = u16;
        type Fast = u32;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u16) -> (i32, i32) {
            ((self as u8 as i32), (r as i32))
        }
    }
    impl SafeComparison<u16> for bool {
        fn safe_lt(self, r: u16) -> bool {
            IsLess((self as u8 as u16), r)
        }
        fn safe_le(self, r: u16) -> bool {
            IsLessOrEqual((self as u8 as u16), r)
        }
        fn safe_gt(self, r: u16) -> bool {
            IsGreater((self as u8 as u16), r)
        }
        fn safe_ge(self, r: u16) -> bool {
            IsGreaterOrEqual((self as u8 as u16), r)
        }
        fn safe_eq(self, r: u16) -> bool {
            IsEqual((self as u8 as u16), r)
        }
        fn safe_ne(self, r: u16) -> bool {
            IsNotEqual((self as u8 as u16), r)
        }
    }
    impl Cast<i32> for bool {
        const CONTAINED: bool = true;
        fn cast(self) -> i32 {
            (self as u8 as i32)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<i32> for bool {}
    impl RangeWith<i32> for bool {
        fn range_with(self, low: i32, high: i32) -> RangeCheck {
            RangeCheck::new(
                0 as i32 >= low || (self as u8 as i32) >= low,
                1 as i32 <= high || (self as u8 as i32) <= high,
            )
        }
    }
    impl Promote<i32> for bool {
        type Max = i32;
        type Lowest = i32;
        type Big = i32;
        type Fast = i64;
        type Sum = i32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: i32) -> (i32, i32) {
            ((self as u8 as i32), r)
        }
    }
    impl SafeComparison<i32> for bool {
        fn safe_lt(self, r: i32) -> bool {
            IsLess((self as u8 as i32), r)
        }
        fn safe_le(self, r: i32) -> bool {
            IsLessOrEqual((self as u8 as i32), r)
        }
        fn safe_gt(self, r: i32) -> bool {
            IsGreater((self as u8 as i32), r)
        }
        fn safe_ge(self, r: i32) -> bool {
            IsGreaterOrEqual((self as u8 as i32), r)
        }
        fn safe_eq(self, r: i32) -> bool {
            IsEqual((self as u8 as i32), r)
        }
        fn safe_ne(self, r: i32) -> bool {
            IsNotEqual((self as u8 as i32), r)
        }
    }
    impl Cast<u32> for bool {
        const CONTAINED: bool = true;
        fn cast(self) -> u32 {
            (self as u8 as u32)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<u32> for bool {}
    impl RangeWith<u32> for bool {
        fn range_with(self, low: u32, high: u32) -> RangeCheck {
            RangeCheck::new(
                0 as u32 >= low || (self as u8 as u32) >= low,
                1 as u32 <= high || (self as u8 as u32) <= high,
            )
        }
    }
    impl Promote<u32> for bool {
        type Max = u32;
        type Lowest = bool;
        type Big = u32;
        type Fast = u64;
        type Sum = u32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = true;
        fn sum_pair(self, r: u32) -> (u32, u32) {
            ((self as u8 as u32), r)
        }
    }
    impl SafeComparison<u32> for bool {
        fn safe_lt(self, r: u32) -> bool {
            IsLess((self as u8 as u32), r)
        }
        fn safe_le(self, r: u32) -> bool {
            IsLessOrEqual((self as u8 as u32), r)
        }
        fn safe_gt(self, r: u32) -> bool {
            IsGreater((self as u8 as u32), r)
        }
        fn safe_ge(self, r: u32) -> bool {
            IsGreaterOrEqual((self as u8 as u32), r)
        }
        fn safe_eq(self, r: u32) -> bool {
            IsEqual((self as u8 as u32), r)
        }
        fn safe_ne(self, r: u32) -> bool {
            IsNotEqual((self as u8 as u32), r)
        }
    }
    impl Cast<i64> for bool {
        const CONTAINED: bool = true;
        fn cast(self) -> i64 {
            (self as u8 as i64)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<i64> for bool {}
    impl RangeWith<i64> for bool {
        fn range_with(self, low: i64, high: i64) -> RangeCheck {
            RangeCheck::new(
                0 as i64 >= low || (self as u8 as i64) >= low,
                1 as i64 <= high || (self as u8 as i64) <= high,
            )
        }
    }
    impl Promote<i64> for bool {
        type Max = i64;
        type Lowest = i64;
        type Big = i64;
        type Fast = i64;
        type Sum = i64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: i64) -> (i64, i64) {
            ((self as u8 as i64), r)
        }
    }
    impl SafeComparison<i64> for bool {
        fn safe_lt(self, r: i64) -> bool {
            IsLess((self as u8 as i64), r)
        }
        fn safe_le(self, r: i64) -> bool {
            IsLessOrEqual((self as u8 as i64), r)
        }
        fn safe_gt(self, r: i64) -> bool {
            IsGreater((self as u8 as i64), r)
        }
        fn safe_ge(self, r: i64) -> bool {
            IsGreaterOrEqual((self as u8 as i64), r)
        }
        fn safe_eq(self, r: i64) -> bool {
            IsEqual((self as u8 as i64), r)
        }
        fn safe_ne(self, r: i64) -> bool {
            IsNotEqual((self as u8 as i64), r)
        }
    }
    impl Cast<u64> for bool {
        const CONTAINED: bool = true;
        fn cast(self) -> u64 {
            (self as u8 as u64)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<u64> for bool {}
    impl RangeWith<u64> for bool {
        fn range_with(self, low: u64, high: u64) -> RangeCheck {
            RangeCheck::new(
                0 as u64 >= low || (self as u8 as u64) >= low,
                1 as u64 <= high || (self as u8 as u64) <= high,
            )
        }
    }
    impl Promote<u64> for bool {
        type Max = u64;
        type Lowest = bool;
        type Big = u64;
        type Fast = u64;
        type Sum = u64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: u64) -> (u64, u64) {
            ((self as u8 as u64), r)
        }
    }
    impl SafeComparison<u64> for bool {
        fn safe_lt(self, r: u64) -> bool {
            IsLess((self as u8 as u64), r)
        }
        fn safe_le(self, r: u64) -> bool {
            IsLessOrEqual((self as u8 as u64), r)
        }
        fn safe_gt(self, r: u64) -> bool {
            IsGreater((self as u8 as u64), r)
        }
        fn safe_ge(self, r: u64) -> bool {
            IsGreaterOrEqual((self as u8 as u64), r)
        }
        fn safe_eq(self, r: u64) -> bool {
            IsEqual((self as u8 as u64), r)
        }
        fn safe_ne(self, r: u64) -> bool {
            IsNotEqual((self as u8 as u64), r)
        }
    }
    impl Cast<f32> for bool {
        const CONTAINED: bool = true;
        fn cast(self) -> f32 {
            (self as u8 as f32)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f32> for bool {}
    impl RangeWith<f32> for bool {
        fn range_with(self, low: f32, high: f32) -> RangeCheck {
            RangeCheck::new(
                0 as f32 >= low || (self as u8 as f32) >= low,
                1 as f32 <= high || (self as u8 as f32) <= high,
            )
        }
    }
    impl Promote<f32> for bool {
        type Max = f32;
        type Lowest = f32;
        type Big = f32;
        type Fast = f32;
        type Sum = f32;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f32) -> (f32, f32) {
            ((self as u8 as f32), r)
        }
    }
    impl SafeComparison<f32> for bool {
        fn safe_lt(self, r: f32) -> bool {
            IsLess((self as u8 as f32), r)
        }
        fn safe_le(self, r: f32) -> bool {
            IsLessOrEqual((self as u8 as f32), r)
        }
        fn safe_gt(self, r: f32) -> bool {
            IsGreater((self as u8 as f32), r)
        }
        fn safe_ge(self, r: f32) -> bool {
            IsGreaterOrEqual((self as u8 as f32), r)
        }
        fn safe_eq(self, r: f32) -> bool {
            IsEqual((self as u8 as f32), r)
        }
        fn safe_ne(self, r: f32) -> bool {
            IsNotEqual((self as u8 as f32), r)
        }
    }
    impl Cast<f64> for bool {
        const CONTAINED: bool = true;
        fn cast(self) -> f64 {
            (self as u8 as f64)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<f64> for bool {}
    impl RangeWith<f64> for bool {
        fn range_with(self, low: f64, high: f64) -> RangeCheck {
            RangeCheck::new(
                0 as f64 >= low || (self as u8 as f64) >= low,
                1 as f64 <= high || (self as u8 as f64) <= high,
            )
        }
    }
    impl Promote<f64> for bool {
        type Max = f64;
        type Lowest = f64;
        type Big = f64;
        type Fast = f64;
        type Sum = f64;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: f64) -> (f64, f64) {
            ((self as u8 as f64), r)
        }
    }
    impl SafeComparison<f64> for bool {
        fn safe_lt(self, r: f64) -> bool {
            IsLess((self as u8 as f64), r)
        }
        fn safe_le(self, r: f64) -> bool {
            IsLessOrEqual((self as u8 as f64), r)
        }
        fn safe_gt(self, r: f64) -> bool {
            IsGreater((self as u8 as f64), r)
        }
        fn safe_ge(self, r: f64) -> bool {
            IsGreaterOrEqual((self as u8 as f64), r)
        }
        fn safe_eq(self, r: f64) -> bool {
            IsEqual((self as u8 as f64), r)
        }
        fn safe_ne(self, r: f64) -> bool {
            IsNotEqual((self as u8 as f64), r)
        }
    }
    impl Cast<isize> for bool {
        const CONTAINED: bool = true;
        fn cast(self) -> isize {
            (self as u8 as isize)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<isize> for bool {}
    impl RangeWith<isize> for bool {
        fn range_with(self, low: isize, high: isize) -> RangeCheck {
            RangeCheck::new(
                0 as isize >= low || (self as u8 as isize) >= low,
                1 as isize <= high || (self as u8 as isize) <= high,
            )
        }
    }
    impl Promote<isize> for bool {
        type Max = isize;
        type Lowest = isize;
        type Big = isize;
        type Fast = isize;
        type Sum = isize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: isize) -> (isize, isize) {
            ((self as u8 as isize), r)
        }
    }
    impl SafeComparison<isize> for bool {
        fn safe_lt(self, r: isize) -> bool {
            IsLess((self as u8 as isize), r)
        }
        fn safe_le(self, r: isize) -> bool {
            IsLessOrEqual((self as u8 as isize), r)
        }
        fn safe_gt(self, r: isize) -> bool {
            IsGreater((self as u8 as isize), r)
        }
        fn safe_ge(self, r: isize) -> bool {
            IsGreaterOrEqual((self as u8 as isize), r)
        }
        fn safe_eq(self, r: isize) -> bool {
            IsEqual((self as u8 as isize), r)
        }
        fn safe_ne(self, r: isize) -> bool {
            IsNotEqual((self as u8 as isize), r)
        }
    }
    impl Cast<usize> for bool {
        const CONTAINED: bool = true;
        fn cast(self) -> usize {
            (self as u8 as usize)
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<usize> for bool {}
    impl RangeWith<usize> for bool {
        fn range_with(self, low: usize, high: usize) -> RangeCheck {
            RangeCheck::new(
                0 as usize >= low || (self as u8 as usize) >= low,
                1 as usize <= high || (self as u8 as usize) <= high,
            )
        }
    }
    impl Promote<usize> for bool {
        type Max = usize;
        type Lowest = bool;
        type Big = usize;
        type Fast = usize;
        type Sum = usize;
        const BIG_CONTAINED: bool = true;
        const FAST_CONTAINED: bool = false;
        fn sum_pair(self, r: usize) -> (usize, usize) {
            ((self as u8 as usize), r)
        }
    }
    impl SafeComparison<usize> for bool {
        fn safe_lt(self, r: usize) -> bool {
            IsLess((self as u8 as usize), r)
        }
        fn safe_le(self, r: usize) -> bool {
            IsLessOrEqual((self as u8 as usize), r)
        }
        fn safe_gt(self, r: usize) -> bool {
            IsGreater((self as u8 as usize), r)
        }
        fn safe_ge(self, r: usize) -> bool {
            IsGreaterOrEqual((self as u8 as usize), r)
        }
        fn safe_eq(self, r: usize) -> bool {
            IsEqual((self as u8 as usize), r)
        }
        fn safe_ne(self, r: usize) -> bool {
            IsNotEqual((self as u8 as usize), r)
        }
    }
    impl Cast<bool> for bool {
        const CONTAINED: bool = true;
        fn cast(self) -> bool {
            self
        }
        fn range(self) -> RangeCheck {
            RangeCheck::new(true, true)
        }
    }
    impl StrictCast<bool> for bool {}
    impl RangeWith<bool> for bool {
        fn range_with(self, low: bool, high: bool) -> RangeCheck {
            RangeCheck::new(false >= low || self >= low, true <= high || self <= high)
        }
    }
    impl SafeComparison<bool> for bool {
        fn safe_lt(self, r: bool) -> bool {
            (self as i32) < (r as i32)
        }
        fn safe_le(self, r: bool) -> bool {
            (self as i32) <= (r as i32)
        }
        fn safe_gt(self, r: bool) -> bool {
            (self as i32) > (r as i32)
        }
        fn safe_ge(self, r: bool) -> bool {
            (self as i32) >= (r as i32)
        }
        fn safe_eq(self, r: bool) -> bool {
            (self as i32) == (r as i32)
        }
        fn safe_ne(self, r: bool) -> bool {
            (self as i32) != (r as i32)
        }
    }
}
// Source: integral_constant_like.h:14-21 and native enum/constant extraction.
// Explicit adapters carry the actual source enum's underlying arithmetic value;
// no blanket trait guesses the representation of arbitrary Rust objects.
#[macro_export]
macro_rules! BASE_NUMERIC_ENUM_INPUT {
    ($enum:ty,$underlying:ty,$convert:expr) => {
        impl $crate::base::numerics::safe_conversions_impl::Input for $enum {
            type Value = $underlying;
            fn value(self) -> Self::Value {
                ($convert)(self)
            }
        }
        impl $crate::base::numerics::safe_conversions::StrictInput for $enum {}
        impl $crate::base::numerics::safe_conversions::UncheckedInput for $enum {}
    };
}
// IntegralConstantLike's constexpr equality/conversion proof is not replaced by
// a permissive Rust blanket implementation; concrete native adapters remain a
// language-level dependency until those exact const constraints are available.
#[cfg(not(base_numeric_native_inputs_configured))]
compile_error!("actual native numerics compiler/macro inputs must be supplied");
// Source: safe_conversions_impl.h:360-381. No 128-bit or two-bit specialization.
pub struct IntegerSelector<const BITS: u32, const SIGNED: bool>;
pub trait IntegerForDigitsAndSign {
    type Type: Integer;
}
macro_rules! integer_selector {
    ($bits:literal,$s:literal,$t:ty) => {
        impl IntegerForDigitsAndSign for IntegerSelector<$bits, $s> {
            type Type = $t;
        }
    };
}
integer_selector!(8, true, i8);
integer_selector!(8, false, u8);
integer_selector!(16, true, i16);
integer_selector!(16, false, u16);
integer_selector!(32, true, i32);
integer_selector!(32, false, u32);
integer_selector!(64, true, i64);
integer_selector!(64, false, u64);
const _: () = assert!(
    core::mem::size_of::<libc::intmax_t>() * 8 == 64,
    "Max integer size not supported for this toolchain."
);

// Source: safe_conversions_impl.h:83-94. _MSC_VER has priority even with __clang__.
#[cfg(base_numeric_compiler_msvc)]
unsafe extern "C" {
    fn rechrom_numeric_debugbreak();
}
#[cfg(base_numeric_compiler_msvc)]
impl CheckOnFailure {
    pub fn HandleFailure<T: Default>() -> T {
        unsafe { rechrom_numeric_debugbreak() };
        T::default()
    }
}
#[cfg(base_numeric_compiler_msvc)]
impl FailureHandler for CheckOnFailure {
    fn HandleFailure<T: Default>() -> T {
        Self::HandleFailure()
    }
}
