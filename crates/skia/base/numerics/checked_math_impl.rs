#![allow(non_snake_case)]
pub trait CheckedModulo<R, V> {
    fn modulo(self, r: R, out: &mut V) -> bool;
}
// Source: base/numerics/checked_math_impl.h:72-100,143-164,213-236 and primitive selected fast operations.
use super::safe_conversions::*;
pub trait CheckedBinary<R, V> {
    fn add(self, r: R, out: &mut V) -> bool;
    fn sub(self, r: R, out: &mut V) -> bool;
    fn mul(self, r: R, out: &mut V) -> bool;
    fn div(self, r: R, out: &mut V) -> bool;
}
pub trait IntegralOps: Integer {
    fn negate(self) -> Self;
    fn invert(self) -> Self::Unsigned;
    fn checked_lsh(self, shift: u64, out: &mut Self) -> bool;
    fn checked_rsh(self, shift: u64, out: &mut Self) -> bool;
    fn clamped_lsh(self, shift: u64) -> Self;
    fn clamped_rsh(self, shift: u64) -> Self;
    fn saturated_neg(self) -> Self;
    fn saturated_abs(self) -> Self;
}
pub trait State<T: Numeric>: Copy {
    fn from(value: T, valid: bool) -> Self;
    fn value(self) -> T;
    fn valid(self) -> bool;
}
// Source: checked_math_impl.h:604-641. The validity flag precedes the native value.
#[derive(Clone, Copy)]
pub struct IntegerState<T: Numeric> {
    is_valid_: bool,
    value_: T,
}
impl<T: Numeric> State<T> for IntegerState<T> {
    fn from(value: T, valid: bool) -> Self {
        Self {
            is_valid_: valid,
            value_: value,
        }
    }
    fn value(self) -> T {
        self.value_
    }
    fn valid(self) -> bool {
        self.is_valid_
    }
}
// Source: checked_math_impl.h:645-683. Floating state has no separate flag.
#[derive(Clone, Copy)]
pub struct FloatState<T: Numeric> {
    value_: T,
}
impl<T: Numeric> State<T> for FloatState<T> {
    fn from(value: T, valid: bool) -> Self {
        Self {
            value_: if valid { value } else { T::nan() },
        }
    }
    fn value(self) -> T {
        self.value_
    }
    fn valid(self) -> bool {
        self.value_.finite()
    }
}
pub trait StateValue: Numeric {
    type Storage: State<Self>;
    fn state_from<S: Numeric + Cast<Self>>(v: S, valid: bool) -> Self::Storage;
}
// Source: checked_math_impl.h primitive specialization for i8.
pub mod i8_ops {
    pub const fn CheckedAddImpl(x: i8, y: i8, result: &mut i8) -> bool {
        let ux = x as u8;
        let uy = y as u8;
        let z = ux.wrapping_add(uy);
        if (((z ^ ux) & (z ^ uy)) as i8) < 0 {
            return false;
        }
        *result = z as i8;
        true
    }
    pub const fn CheckedSubImpl(x: i8, y: i8, result: &mut i8) -> bool {
        let ux = x as u8;
        let uy = y as u8;
        let z = ux.wrapping_sub(uy);
        if (((z ^ ux) & (ux ^ uy)) as i8) < 0 {
            return false;
        }
        *result = z as i8;
        true
    }
    pub const fn CheckedMulImpl(x: i8, y: i8, result: &mut i8) -> bool {
        let ux = if x < 0 {
            (0 as u8).wrapping_sub(x as u8)
        } else {
            x as u8
        };
        let uy = if y < 0 {
            (0 as u8).wrapping_sub(y as u8)
        } else {
            y as u8
        };
        let z = ux.wrapping_mul(uy);
        let negative = x ^ y < 0;
        if uy > (negative as u8) && ux > ((i8::MAX as u8).wrapping_add(negative as u8)) / uy {
            return false;
        }
        *result = if negative {
            (0 as u8).wrapping_sub(z)
        } else {
            z
        } as i8;
        true
    }
}
impl StateValue for i8 {
    type Storage = IntegerState<Self>;
    fn state_from<S: Numeric + Cast<Self>>(v: S, valid: bool) -> Self::Storage {
        let ok = valid && v.range().IsValid();
        let value = if !S::FLOAT || ok { v.cast() } else { 0 };
        IntegerState::from(value, ok)
    }
}
impl IntegralOps for i8 {
    fn negate(self) -> Self {
        (0 as u8).wrapping_sub(self as u8) as Self
    }
    fn invert(self) -> u8 {
        !(self as u8)
    }
    fn checked_lsh(self, shift: u64, out: &mut Self) -> bool {
        let digits = Self::DIGITS as u64;
        if !self.negative() && shift < digits {
            *out = ((self as u8) << (shift as u32)) as Self;
            return (*out >> (shift as u32)) == self;
        }
        if false || self != 0 || shift != digits {
            return false;
        }
        *out = 0;
        true
    }
    fn checked_rsh(self, shift: u64, out: &mut Self) -> bool {
        if shift >= Self::BITS as u64 {
            return false;
        }
        *out = self >> (shift as u32);
        true
    }
    fn clamped_lsh(self, shift: u64) -> Self {
        if shift < Self::DIGITS as u64 {
            let result = ((self as u8) << (shift as u32)) as Self;
            if result >> (shift as u32) == self {
                return result;
            }
        }
        if self == 0 {
            0
        } else if self.negative() {
            Self::MIN
        } else {
            Self::MAX
        }
    }
    fn clamped_rsh(self, shift: u64) -> Self {
        if shift < Self::BITS as u64 {
            self >> (shift as u32)
        } else if self.negative() {
            -1
        } else {
            0
        }
    }
    fn saturated_neg(self) -> Self {
        if super::safe_math_shared_impl::ClampedNegFastOp::<Self>::is_supported {
            super::safe_math_shared_impl::ClampedNegFastOp::<Self>::Do(self)
        } else {
            if self == Self::MIN {
                Self::MAX
            } else {
                -self
            }
        }
    }
    fn saturated_abs(self) -> Self {
        let abs = self.unsigned_abs();
        abs.wrapping_sub(((abs as Self) < 0) as u8) as Self
    }
}
// Source: checked_math_impl.h primitive specialization for u8.
pub mod u8_ops {
    pub const fn CheckedAddImpl(x: u8, y: u8, result: &mut u8) -> bool {
        let ux = x as u8;
        let uy = y as u8;
        let z = ux.wrapping_add(uy);
        if z < uy {
            return false;
        }
        *result = z as u8;
        true
    }
    pub const fn CheckedSubImpl(x: u8, y: u8, result: &mut u8) -> bool {
        let ux = x as u8;
        let uy = y as u8;
        let z = ux.wrapping_sub(uy);
        if x < y {
            return false;
        }
        *result = z as u8;
        true
    }
    pub const fn CheckedMulImpl(x: u8, y: u8, result: &mut u8) -> bool {
        let ux = x;
        let uy = y;
        let z = ux.wrapping_mul(uy);
        let negative = false;
        if uy > 1 && ux > ((u8::MAX as u8).wrapping_add(negative as u8)) / uy {
            return false;
        }
        *result = if negative {
            (0 as u8).wrapping_sub(z)
        } else {
            z
        } as u8;
        true
    }
}
impl StateValue for u8 {
    type Storage = IntegerState<Self>;
    fn state_from<S: Numeric + Cast<Self>>(v: S, valid: bool) -> Self::Storage {
        let ok = valid && v.range().IsValid();
        let value = if !S::FLOAT || ok { v.cast() } else { 0 };
        IntegerState::from(value, ok)
    }
}
impl IntegralOps for u8 {
    fn negate(self) -> Self {
        (0 as u8).wrapping_sub(self as u8) as Self
    }
    fn invert(self) -> u8 {
        !(self as u8)
    }
    fn checked_lsh(self, shift: u64, out: &mut Self) -> bool {
        let digits = Self::DIGITS as u64;
        if !self.negative() && shift < digits {
            *out = ((self as u8) << (shift as u32)) as Self;
            return (*out >> (shift as u32)) == self;
        }
        if true || self != 0 || shift != digits {
            return false;
        }
        *out = 0;
        true
    }
    fn checked_rsh(self, shift: u64, out: &mut Self) -> bool {
        if shift >= Self::BITS as u64 {
            return false;
        }
        *out = self >> (shift as u32);
        true
    }
    fn clamped_lsh(self, shift: u64) -> Self {
        if shift < Self::DIGITS as u64 {
            let result = ((self as u8) << (shift as u32)) as Self;
            if result >> (shift as u32) == self {
                return result;
            }
        }
        if self == 0 {
            0
        } else if self.negative() {
            Self::MIN
        } else {
            Self::MAX
        }
    }
    fn clamped_rsh(self, shift: u64) -> Self {
        if shift < Self::BITS as u64 {
            self >> (shift as u32)
        } else if self.negative() {
            0
        } else {
            0
        }
    }
    fn saturated_neg(self) -> Self {
        0
    }
    fn saturated_abs(self) -> Self {
        self
    }
}
// Source: checked_math_impl.h primitive specialization for i16.
pub mod i16_ops {
    pub const fn CheckedAddImpl(x: i16, y: i16, result: &mut i16) -> bool {
        let ux = x as u16;
        let uy = y as u16;
        let z = ux.wrapping_add(uy);
        if (((z ^ ux) & (z ^ uy)) as i16) < 0 {
            return false;
        }
        *result = z as i16;
        true
    }
    pub const fn CheckedSubImpl(x: i16, y: i16, result: &mut i16) -> bool {
        let ux = x as u16;
        let uy = y as u16;
        let z = ux.wrapping_sub(uy);
        if (((z ^ ux) & (ux ^ uy)) as i16) < 0 {
            return false;
        }
        *result = z as i16;
        true
    }
    pub const fn CheckedMulImpl(x: i16, y: i16, result: &mut i16) -> bool {
        let ux = if x < 0 {
            (0 as u16).wrapping_sub(x as u16)
        } else {
            x as u16
        };
        let uy = if y < 0 {
            (0 as u16).wrapping_sub(y as u16)
        } else {
            y as u16
        };
        let z = ux.wrapping_mul(uy);
        let negative = x ^ y < 0;
        if uy > (negative as u16) && ux > ((i16::MAX as u16).wrapping_add(negative as u16)) / uy {
            return false;
        }
        *result = if negative {
            (0 as u16).wrapping_sub(z)
        } else {
            z
        } as i16;
        true
    }
}
impl StateValue for i16 {
    type Storage = IntegerState<Self>;
    fn state_from<S: Numeric + Cast<Self>>(v: S, valid: bool) -> Self::Storage {
        let ok = valid && v.range().IsValid();
        let value = if !S::FLOAT || ok { v.cast() } else { 0 };
        IntegerState::from(value, ok)
    }
}
impl IntegralOps for i16 {
    fn negate(self) -> Self {
        (0 as u16).wrapping_sub(self as u16) as Self
    }
    fn invert(self) -> u16 {
        !(self as u16)
    }
    fn checked_lsh(self, shift: u64, out: &mut Self) -> bool {
        let digits = Self::DIGITS as u64;
        if !self.negative() && shift < digits {
            *out = ((self as u16) << (shift as u32)) as Self;
            return (*out >> (shift as u32)) == self;
        }
        if false || self != 0 || shift != digits {
            return false;
        }
        *out = 0;
        true
    }
    fn checked_rsh(self, shift: u64, out: &mut Self) -> bool {
        if shift >= Self::BITS as u64 {
            return false;
        }
        *out = self >> (shift as u32);
        true
    }
    fn clamped_lsh(self, shift: u64) -> Self {
        if shift < Self::DIGITS as u64 {
            let result = ((self as u16) << (shift as u32)) as Self;
            if result >> (shift as u32) == self {
                return result;
            }
        }
        if self == 0 {
            0
        } else if self.negative() {
            Self::MIN
        } else {
            Self::MAX
        }
    }
    fn clamped_rsh(self, shift: u64) -> Self {
        if shift < Self::BITS as u64 {
            self >> (shift as u32)
        } else if self.negative() {
            -1
        } else {
            0
        }
    }
    fn saturated_neg(self) -> Self {
        if super::safe_math_shared_impl::ClampedNegFastOp::<Self>::is_supported {
            super::safe_math_shared_impl::ClampedNegFastOp::<Self>::Do(self)
        } else {
            if self == Self::MIN {
                Self::MAX
            } else {
                -self
            }
        }
    }
    fn saturated_abs(self) -> Self {
        let abs = self.unsigned_abs();
        abs.wrapping_sub(((abs as Self) < 0) as u16) as Self
    }
}
// Source: checked_math_impl.h primitive specialization for u16.
pub mod u16_ops {
    pub const fn CheckedAddImpl(x: u16, y: u16, result: &mut u16) -> bool {
        let ux = x as u16;
        let uy = y as u16;
        let z = ux.wrapping_add(uy);
        if z < uy {
            return false;
        }
        *result = z as u16;
        true
    }
    pub const fn CheckedSubImpl(x: u16, y: u16, result: &mut u16) -> bool {
        let ux = x as u16;
        let uy = y as u16;
        let z = ux.wrapping_sub(uy);
        if x < y {
            return false;
        }
        *result = z as u16;
        true
    }
    pub const fn CheckedMulImpl(x: u16, y: u16, result: &mut u16) -> bool {
        let ux = x;
        let uy = y;
        let z = ux.wrapping_mul(uy);
        let negative = false;
        if uy > 1 && ux > ((u16::MAX as u16).wrapping_add(negative as u16)) / uy {
            return false;
        }
        *result = if negative {
            (0 as u16).wrapping_sub(z)
        } else {
            z
        } as u16;
        true
    }
}
impl StateValue for u16 {
    type Storage = IntegerState<Self>;
    fn state_from<S: Numeric + Cast<Self>>(v: S, valid: bool) -> Self::Storage {
        let ok = valid && v.range().IsValid();
        let value = if !S::FLOAT || ok { v.cast() } else { 0 };
        IntegerState::from(value, ok)
    }
}
impl IntegralOps for u16 {
    fn negate(self) -> Self {
        (0 as u16).wrapping_sub(self as u16) as Self
    }
    fn invert(self) -> u16 {
        !(self as u16)
    }
    fn checked_lsh(self, shift: u64, out: &mut Self) -> bool {
        let digits = Self::DIGITS as u64;
        if !self.negative() && shift < digits {
            *out = ((self as u16) << (shift as u32)) as Self;
            return (*out >> (shift as u32)) == self;
        }
        if true || self != 0 || shift != digits {
            return false;
        }
        *out = 0;
        true
    }
    fn checked_rsh(self, shift: u64, out: &mut Self) -> bool {
        if shift >= Self::BITS as u64 {
            return false;
        }
        *out = self >> (shift as u32);
        true
    }
    fn clamped_lsh(self, shift: u64) -> Self {
        if shift < Self::DIGITS as u64 {
            let result = ((self as u16) << (shift as u32)) as Self;
            if result >> (shift as u32) == self {
                return result;
            }
        }
        if self == 0 {
            0
        } else if self.negative() {
            Self::MIN
        } else {
            Self::MAX
        }
    }
    fn clamped_rsh(self, shift: u64) -> Self {
        if shift < Self::BITS as u64 {
            self >> (shift as u32)
        } else if self.negative() {
            0
        } else {
            0
        }
    }
    fn saturated_neg(self) -> Self {
        0
    }
    fn saturated_abs(self) -> Self {
        self
    }
}
// Source: checked_math_impl.h primitive specialization for i32.
pub mod i32_ops {
    pub const fn CheckedAddImpl(x: i32, y: i32, result: &mut i32) -> bool {
        let ux = x as u32;
        let uy = y as u32;
        let z = ux.wrapping_add(uy);
        if (((z ^ ux) & (z ^ uy)) as i32) < 0 {
            return false;
        }
        *result = z as i32;
        true
    }
    pub const fn CheckedSubImpl(x: i32, y: i32, result: &mut i32) -> bool {
        let ux = x as u32;
        let uy = y as u32;
        let z = ux.wrapping_sub(uy);
        if (((z ^ ux) & (ux ^ uy)) as i32) < 0 {
            return false;
        }
        *result = z as i32;
        true
    }
    pub const fn CheckedMulImpl(x: i32, y: i32, result: &mut i32) -> bool {
        let ux = if x < 0 {
            (0 as u32).wrapping_sub(x as u32)
        } else {
            x as u32
        };
        let uy = if y < 0 {
            (0 as u32).wrapping_sub(y as u32)
        } else {
            y as u32
        };
        let z = ux.wrapping_mul(uy);
        let negative = x ^ y < 0;
        if uy > (negative as u32) && ux > ((i32::MAX as u32).wrapping_add(negative as u32)) / uy {
            return false;
        }
        *result = if negative {
            (0 as u32).wrapping_sub(z)
        } else {
            z
        } as i32;
        true
    }
}
impl StateValue for i32 {
    type Storage = IntegerState<Self>;
    fn state_from<S: Numeric + Cast<Self>>(v: S, valid: bool) -> Self::Storage {
        let ok = valid && v.range().IsValid();
        let value = if !S::FLOAT || ok { v.cast() } else { 0 };
        IntegerState::from(value, ok)
    }
}
impl IntegralOps for i32 {
    fn negate(self) -> Self {
        (0 as u32).wrapping_sub(self as u32) as Self
    }
    fn invert(self) -> u32 {
        !(self as u32)
    }
    fn checked_lsh(self, shift: u64, out: &mut Self) -> bool {
        let digits = Self::DIGITS as u64;
        if !self.negative() && shift < digits {
            *out = ((self as u32) << (shift as u32)) as Self;
            return (*out >> (shift as u32)) == self;
        }
        if false || self != 0 || shift != digits {
            return false;
        }
        *out = 0;
        true
    }
    fn checked_rsh(self, shift: u64, out: &mut Self) -> bool {
        if shift >= Self::BITS as u64 {
            return false;
        }
        *out = self >> (shift as u32);
        true
    }
    fn clamped_lsh(self, shift: u64) -> Self {
        if shift < Self::DIGITS as u64 {
            let result = ((self as u32) << (shift as u32)) as Self;
            if result >> (shift as u32) == self {
                return result;
            }
        }
        if self == 0 {
            0
        } else if self.negative() {
            Self::MIN
        } else {
            Self::MAX
        }
    }
    fn clamped_rsh(self, shift: u64) -> Self {
        if shift < Self::BITS as u64 {
            self >> (shift as u32)
        } else if self.negative() {
            -1
        } else {
            0
        }
    }
    fn saturated_neg(self) -> Self {
        if super::safe_math_shared_impl::ClampedNegFastOp::<Self>::is_supported {
            super::safe_math_shared_impl::ClampedNegFastOp::<Self>::Do(self)
        } else {
            if self == Self::MIN {
                Self::MAX
            } else {
                -self
            }
        }
    }
    fn saturated_abs(self) -> Self {
        let abs = self.unsigned_abs();
        abs.wrapping_sub(((abs as Self) < 0) as u32) as Self
    }
}
// Source: checked_math_impl.h primitive specialization for u32.
pub mod u32_ops {
    pub const fn CheckedAddImpl(x: u32, y: u32, result: &mut u32) -> bool {
        let ux = x as u32;
        let uy = y as u32;
        let z = ux.wrapping_add(uy);
        if z < uy {
            return false;
        }
        *result = z as u32;
        true
    }
    pub const fn CheckedSubImpl(x: u32, y: u32, result: &mut u32) -> bool {
        let ux = x as u32;
        let uy = y as u32;
        let z = ux.wrapping_sub(uy);
        if x < y {
            return false;
        }
        *result = z as u32;
        true
    }
    pub const fn CheckedMulImpl(x: u32, y: u32, result: &mut u32) -> bool {
        let ux = x;
        let uy = y;
        let z = ux.wrapping_mul(uy);
        let negative = false;
        if uy > 1 && ux > ((u32::MAX as u32).wrapping_add(negative as u32)) / uy {
            return false;
        }
        *result = if negative {
            (0 as u32).wrapping_sub(z)
        } else {
            z
        } as u32;
        true
    }
}
impl StateValue for u32 {
    type Storage = IntegerState<Self>;
    fn state_from<S: Numeric + Cast<Self>>(v: S, valid: bool) -> Self::Storage {
        let ok = valid && v.range().IsValid();
        let value = if !S::FLOAT || ok { v.cast() } else { 0 };
        IntegerState::from(value, ok)
    }
}
impl IntegralOps for u32 {
    fn negate(self) -> Self {
        (0 as u32).wrapping_sub(self as u32) as Self
    }
    fn invert(self) -> u32 {
        !(self as u32)
    }
    fn checked_lsh(self, shift: u64, out: &mut Self) -> bool {
        let digits = Self::DIGITS as u64;
        if !self.negative() && shift < digits {
            *out = ((self as u32) << (shift as u32)) as Self;
            return (*out >> (shift as u32)) == self;
        }
        if true || self != 0 || shift != digits {
            return false;
        }
        *out = 0;
        true
    }
    fn checked_rsh(self, shift: u64, out: &mut Self) -> bool {
        if shift >= Self::BITS as u64 {
            return false;
        }
        *out = self >> (shift as u32);
        true
    }
    fn clamped_lsh(self, shift: u64) -> Self {
        if shift < Self::DIGITS as u64 {
            let result = ((self as u32) << (shift as u32)) as Self;
            if result >> (shift as u32) == self {
                return result;
            }
        }
        if self == 0 {
            0
        } else if self.negative() {
            Self::MIN
        } else {
            Self::MAX
        }
    }
    fn clamped_rsh(self, shift: u64) -> Self {
        if shift < Self::BITS as u64 {
            self >> (shift as u32)
        } else if self.negative() {
            0
        } else {
            0
        }
    }
    fn saturated_neg(self) -> Self {
        0
    }
    fn saturated_abs(self) -> Self {
        self
    }
}
// Source: checked_math_impl.h primitive specialization for i64.
pub mod i64_ops {
    pub const fn CheckedAddImpl(x: i64, y: i64, result: &mut i64) -> bool {
        let ux = x as u64;
        let uy = y as u64;
        let z = ux.wrapping_add(uy);
        if (((z ^ ux) & (z ^ uy)) as i64) < 0 {
            return false;
        }
        *result = z as i64;
        true
    }
    pub const fn CheckedSubImpl(x: i64, y: i64, result: &mut i64) -> bool {
        let ux = x as u64;
        let uy = y as u64;
        let z = ux.wrapping_sub(uy);
        if (((z ^ ux) & (ux ^ uy)) as i64) < 0 {
            return false;
        }
        *result = z as i64;
        true
    }
    pub const fn CheckedMulImpl(x: i64, y: i64, result: &mut i64) -> bool {
        let ux = if x < 0 {
            (0 as u64).wrapping_sub(x as u64)
        } else {
            x as u64
        };
        let uy = if y < 0 {
            (0 as u64).wrapping_sub(y as u64)
        } else {
            y as u64
        };
        let z = ux.wrapping_mul(uy);
        let negative = x ^ y < 0;
        if uy > (negative as u64) && ux > ((i64::MAX as u64).wrapping_add(negative as u64)) / uy {
            return false;
        }
        *result = if negative {
            (0 as u64).wrapping_sub(z)
        } else {
            z
        } as i64;
        true
    }
}
impl StateValue for i64 {
    type Storage = IntegerState<Self>;
    fn state_from<S: Numeric + Cast<Self>>(v: S, valid: bool) -> Self::Storage {
        let ok = valid && v.range().IsValid();
        let value = if !S::FLOAT || ok { v.cast() } else { 0 };
        IntegerState::from(value, ok)
    }
}
impl IntegralOps for i64 {
    fn negate(self) -> Self {
        (0 as u64).wrapping_sub(self as u64) as Self
    }
    fn invert(self) -> u64 {
        !(self as u64)
    }
    fn checked_lsh(self, shift: u64, out: &mut Self) -> bool {
        let digits = Self::DIGITS as u64;
        if !self.negative() && shift < digits {
            *out = ((self as u64) << (shift as u32)) as Self;
            return (*out >> (shift as u32)) == self;
        }
        if false || self != 0 || shift != digits {
            return false;
        }
        *out = 0;
        true
    }
    fn checked_rsh(self, shift: u64, out: &mut Self) -> bool {
        if shift >= Self::BITS as u64 {
            return false;
        }
        *out = self >> (shift as u32);
        true
    }
    fn clamped_lsh(self, shift: u64) -> Self {
        if shift < Self::DIGITS as u64 {
            let result = ((self as u64) << (shift as u32)) as Self;
            if result >> (shift as u32) == self {
                return result;
            }
        }
        if self == 0 {
            0
        } else if self.negative() {
            Self::MIN
        } else {
            Self::MAX
        }
    }
    fn clamped_rsh(self, shift: u64) -> Self {
        if shift < Self::BITS as u64 {
            self >> (shift as u32)
        } else if self.negative() {
            -1
        } else {
            0
        }
    }
    fn saturated_neg(self) -> Self {
        if super::safe_math_shared_impl::ClampedNegFastOp::<Self>::is_supported {
            super::safe_math_shared_impl::ClampedNegFastOp::<Self>::Do(self)
        } else {
            if self == Self::MIN {
                Self::MAX
            } else {
                -self
            }
        }
    }
    fn saturated_abs(self) -> Self {
        let abs = self.unsigned_abs();
        abs.wrapping_sub(((abs as Self) < 0) as u64) as Self
    }
}
// Source: checked_math_impl.h primitive specialization for u64.
pub mod u64_ops {
    pub const fn CheckedAddImpl(x: u64, y: u64, result: &mut u64) -> bool {
        let ux = x as u64;
        let uy = y as u64;
        let z = ux.wrapping_add(uy);
        if z < uy {
            return false;
        }
        *result = z as u64;
        true
    }
    pub const fn CheckedSubImpl(x: u64, y: u64, result: &mut u64) -> bool {
        let ux = x as u64;
        let uy = y as u64;
        let z = ux.wrapping_sub(uy);
        if x < y {
            return false;
        }
        *result = z as u64;
        true
    }
    pub const fn CheckedMulImpl(x: u64, y: u64, result: &mut u64) -> bool {
        let ux = x;
        let uy = y;
        let z = ux.wrapping_mul(uy);
        let negative = false;
        if uy > 1 && ux > ((u64::MAX as u64).wrapping_add(negative as u64)) / uy {
            return false;
        }
        *result = if negative {
            (0 as u64).wrapping_sub(z)
        } else {
            z
        } as u64;
        true
    }
}
impl StateValue for u64 {
    type Storage = IntegerState<Self>;
    fn state_from<S: Numeric + Cast<Self>>(v: S, valid: bool) -> Self::Storage {
        let ok = valid && v.range().IsValid();
        let value = if !S::FLOAT || ok { v.cast() } else { 0 };
        IntegerState::from(value, ok)
    }
}
impl IntegralOps for u64 {
    fn negate(self) -> Self {
        (0 as u64).wrapping_sub(self as u64) as Self
    }
    fn invert(self) -> u64 {
        !(self as u64)
    }
    fn checked_lsh(self, shift: u64, out: &mut Self) -> bool {
        let digits = Self::DIGITS as u64;
        if !self.negative() && shift < digits {
            *out = ((self as u64) << (shift as u32)) as Self;
            return (*out >> (shift as u32)) == self;
        }
        if true || self != 0 || shift != digits {
            return false;
        }
        *out = 0;
        true
    }
    fn checked_rsh(self, shift: u64, out: &mut Self) -> bool {
        if shift >= Self::BITS as u64 {
            return false;
        }
        *out = self >> (shift as u32);
        true
    }
    fn clamped_lsh(self, shift: u64) -> Self {
        if shift < Self::DIGITS as u64 {
            let result = ((self as u64) << (shift as u32)) as Self;
            if result >> (shift as u32) == self {
                return result;
            }
        }
        if self == 0 {
            0
        } else if self.negative() {
            Self::MIN
        } else {
            Self::MAX
        }
    }
    fn clamped_rsh(self, shift: u64) -> Self {
        if shift < Self::BITS as u64 {
            self >> (shift as u32)
        } else if self.negative() {
            0
        } else {
            0
        }
    }
    fn saturated_neg(self) -> Self {
        0
    }
    fn saturated_abs(self) -> Self {
        self
    }
}
// Source: checked_math_impl.h primitive specialization for isize.
pub mod isize_ops {
    pub const fn CheckedAddImpl(x: isize, y: isize, result: &mut isize) -> bool {
        let ux = x as usize;
        let uy = y as usize;
        let z = ux.wrapping_add(uy);
        if (((z ^ ux) & (z ^ uy)) as isize) < 0 {
            return false;
        }
        *result = z as isize;
        true
    }
    pub const fn CheckedSubImpl(x: isize, y: isize, result: &mut isize) -> bool {
        let ux = x as usize;
        let uy = y as usize;
        let z = ux.wrapping_sub(uy);
        if (((z ^ ux) & (ux ^ uy)) as isize) < 0 {
            return false;
        }
        *result = z as isize;
        true
    }
    pub const fn CheckedMulImpl(x: isize, y: isize, result: &mut isize) -> bool {
        let ux = if x < 0 {
            (0 as usize).wrapping_sub(x as usize)
        } else {
            x as usize
        };
        let uy = if y < 0 {
            (0 as usize).wrapping_sub(y as usize)
        } else {
            y as usize
        };
        let z = ux.wrapping_mul(uy);
        let negative = x ^ y < 0;
        if uy > (negative as usize)
            && ux > ((isize::MAX as usize).wrapping_add(negative as usize)) / uy
        {
            return false;
        }
        *result = if negative {
            (0 as usize).wrapping_sub(z)
        } else {
            z
        } as isize;
        true
    }
}
impl StateValue for isize {
    type Storage = IntegerState<Self>;
    fn state_from<S: Numeric + Cast<Self>>(v: S, valid: bool) -> Self::Storage {
        let ok = valid && v.range().IsValid();
        let value = if !S::FLOAT || ok { v.cast() } else { 0 };
        IntegerState::from(value, ok)
    }
}
impl IntegralOps for isize {
    fn negate(self) -> Self {
        (0 as usize).wrapping_sub(self as usize) as Self
    }
    fn invert(self) -> usize {
        !(self as usize)
    }
    fn checked_lsh(self, shift: u64, out: &mut Self) -> bool {
        let digits = Self::DIGITS as u64;
        if !self.negative() && shift < digits {
            *out = ((self as usize) << (shift as u32)) as Self;
            return (*out >> (shift as u32)) == self;
        }
        if false || self != 0 || shift != digits {
            return false;
        }
        *out = 0;
        true
    }
    fn checked_rsh(self, shift: u64, out: &mut Self) -> bool {
        if shift >= Self::BITS as u64 {
            return false;
        }
        *out = self >> (shift as u32);
        true
    }
    fn clamped_lsh(self, shift: u64) -> Self {
        if shift < Self::DIGITS as u64 {
            let result = ((self as usize) << (shift as u32)) as Self;
            if result >> (shift as u32) == self {
                return result;
            }
        }
        if self == 0 {
            0
        } else if self.negative() {
            Self::MIN
        } else {
            Self::MAX
        }
    }
    fn clamped_rsh(self, shift: u64) -> Self {
        if shift < Self::BITS as u64 {
            self >> (shift as u32)
        } else if self.negative() {
            -1
        } else {
            0
        }
    }
    fn saturated_neg(self) -> Self {
        if super::safe_math_shared_impl::ClampedNegFastOp::<Self>::is_supported {
            super::safe_math_shared_impl::ClampedNegFastOp::<Self>::Do(self)
        } else {
            if self == Self::MIN {
                Self::MAX
            } else {
                -self
            }
        }
    }
    fn saturated_abs(self) -> Self {
        let abs = self.unsigned_abs();
        abs.wrapping_sub(((abs as Self) < 0) as usize) as Self
    }
}
// Source: checked_math_impl.h primitive specialization for usize.
pub mod usize_ops {
    pub const fn CheckedAddImpl(x: usize, y: usize, result: &mut usize) -> bool {
        let ux = x as usize;
        let uy = y as usize;
        let z = ux.wrapping_add(uy);
        if z < uy {
            return false;
        }
        *result = z as usize;
        true
    }
    pub const fn CheckedSubImpl(x: usize, y: usize, result: &mut usize) -> bool {
        let ux = x as usize;
        let uy = y as usize;
        let z = ux.wrapping_sub(uy);
        if x < y {
            return false;
        }
        *result = z as usize;
        true
    }
    pub const fn CheckedMulImpl(x: usize, y: usize, result: &mut usize) -> bool {
        let ux = x;
        let uy = y;
        let z = ux.wrapping_mul(uy);
        let negative = false;
        if uy > 1 && ux > ((usize::MAX as usize).wrapping_add(negative as usize)) / uy {
            return false;
        }
        *result = if negative {
            (0 as usize).wrapping_sub(z)
        } else {
            z
        } as usize;
        true
    }
}
impl StateValue for usize {
    type Storage = IntegerState<Self>;
    fn state_from<S: Numeric + Cast<Self>>(v: S, valid: bool) -> Self::Storage {
        let ok = valid && v.range().IsValid();
        let value = if !S::FLOAT || ok { v.cast() } else { 0 };
        IntegerState::from(value, ok)
    }
}
impl IntegralOps for usize {
    fn negate(self) -> Self {
        (0 as usize).wrapping_sub(self as usize) as Self
    }
    fn invert(self) -> usize {
        !(self as usize)
    }
    fn checked_lsh(self, shift: u64, out: &mut Self) -> bool {
        let digits = Self::DIGITS as u64;
        if !self.negative() && shift < digits {
            *out = ((self as usize) << (shift as u32)) as Self;
            return (*out >> (shift as u32)) == self;
        }
        if true || self != 0 || shift != digits {
            return false;
        }
        *out = 0;
        true
    }
    fn checked_rsh(self, shift: u64, out: &mut Self) -> bool {
        if shift >= Self::BITS as u64 {
            return false;
        }
        *out = self >> (shift as u32);
        true
    }
    fn clamped_lsh(self, shift: u64) -> Self {
        if shift < Self::DIGITS as u64 {
            let result = ((self as usize) << (shift as u32)) as Self;
            if result >> (shift as u32) == self {
                return result;
            }
        }
        if self == 0 {
            0
        } else if self.negative() {
            Self::MIN
        } else {
            Self::MAX
        }
    }
    fn clamped_rsh(self, shift: u64) -> Self {
        if shift < Self::BITS as u64 {
            self >> (shift as u32)
        } else if self.negative() {
            0
        } else {
            0
        }
    }
    fn saturated_neg(self) -> Self {
        0
    }
    fn saturated_abs(self) -> Self {
        self
    }
}
impl StateValue for f32 {
    type Storage = FloatState<Self>;
    fn state_from<S: Numeric + Cast<Self>>(v: S, valid: bool) -> Self::Storage {
        FloatState::from(
            if valid && (S::CONTAINED || v.range().IsValid()) {
                v.cast()
            } else {
                Self::NAN
            },
            true,
        )
    }
}
impl StateValue for f64 {
    type Storage = FloatState<Self>;
    fn state_from<S: Numeric + Cast<Self>>(v: S, valid: bool) -> Self::Storage {
        FloatState::from(
            if valid && (S::CONTAINED || v.range().IsValid()) {
                v.cast()
            } else {
                Self::NAN
            },
            true,
        )
    }
}

pub trait BitPair<R: Numeric>: Numeric {
    type Output: Numeric;
    fn bit_and(self, r: R) -> Self::Output;
    fn bit_or(self, r: R) -> Self::Output;
    fn bit_xor(self, r: R) -> Self::Output;
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i8> for i8 {
    type Output = u8;
    fn bit_and(self, r: i8) -> u8 {
        (self as u8) & (r as u8)
    }
    fn bit_or(self, r: i8) -> u8 {
        (self as u8) | (r as u8)
    }
    fn bit_xor(self, r: i8) -> u8 {
        (self as u8) ^ (r as u8)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u8> for i8 {
    type Output = u8;
    fn bit_and(self, r: u8) -> u8 {
        (self as u8) & (r as u8)
    }
    fn bit_or(self, r: u8) -> u8 {
        (self as u8) | (r as u8)
    }
    fn bit_xor(self, r: u8) -> u8 {
        (self as u8) ^ (r as u8)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i16> for i8 {
    type Output = u16;
    fn bit_and(self, r: i16) -> u16 {
        (self as u16) & (r as u16)
    }
    fn bit_or(self, r: i16) -> u16 {
        (self as u16) | (r as u16)
    }
    fn bit_xor(self, r: i16) -> u16 {
        (self as u16) ^ (r as u16)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u16> for i8 {
    type Output = u16;
    fn bit_and(self, r: u16) -> u16 {
        (self as u16) & (r as u16)
    }
    fn bit_or(self, r: u16) -> u16 {
        (self as u16) | (r as u16)
    }
    fn bit_xor(self, r: u16) -> u16 {
        (self as u16) ^ (r as u16)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i32> for i8 {
    type Output = u32;
    fn bit_and(self, r: i32) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: i32) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: i32) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u32> for i8 {
    type Output = u32;
    fn bit_and(self, r: u32) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: u32) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: u32) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i64> for i8 {
    type Output = u64;
    fn bit_and(self, r: i64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u64> for i8 {
    type Output = u64;
    fn bit_and(self, r: u64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<isize> for i8 {
    type Output = u32;
    fn bit_and(self, r: isize) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: isize) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: isize) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<usize> for i8 {
    type Output = u32;
    fn bit_and(self, r: usize) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: usize) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: usize) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i8> for u8 {
    type Output = u8;
    fn bit_and(self, r: i8) -> u8 {
        (self as u8) & (r as u8)
    }
    fn bit_or(self, r: i8) -> u8 {
        (self as u8) | (r as u8)
    }
    fn bit_xor(self, r: i8) -> u8 {
        (self as u8) ^ (r as u8)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u8> for u8 {
    type Output = u8;
    fn bit_and(self, r: u8) -> u8 {
        (self as u8) & (r as u8)
    }
    fn bit_or(self, r: u8) -> u8 {
        (self as u8) | (r as u8)
    }
    fn bit_xor(self, r: u8) -> u8 {
        (self as u8) ^ (r as u8)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i16> for u8 {
    type Output = u16;
    fn bit_and(self, r: i16) -> u16 {
        (self as u16) & (r as u16)
    }
    fn bit_or(self, r: i16) -> u16 {
        (self as u16) | (r as u16)
    }
    fn bit_xor(self, r: i16) -> u16 {
        (self as u16) ^ (r as u16)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u16> for u8 {
    type Output = u16;
    fn bit_and(self, r: u16) -> u16 {
        (self as u16) & (r as u16)
    }
    fn bit_or(self, r: u16) -> u16 {
        (self as u16) | (r as u16)
    }
    fn bit_xor(self, r: u16) -> u16 {
        (self as u16) ^ (r as u16)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i32> for u8 {
    type Output = u32;
    fn bit_and(self, r: i32) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: i32) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: i32) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u32> for u8 {
    type Output = u32;
    fn bit_and(self, r: u32) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: u32) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: u32) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i64> for u8 {
    type Output = u64;
    fn bit_and(self, r: i64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u64> for u8 {
    type Output = u64;
    fn bit_and(self, r: u64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<isize> for u8 {
    type Output = u32;
    fn bit_and(self, r: isize) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: isize) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: isize) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<usize> for u8 {
    type Output = u32;
    fn bit_and(self, r: usize) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: usize) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: usize) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i8> for i16 {
    type Output = u16;
    fn bit_and(self, r: i8) -> u16 {
        (self as u16) & (r as u16)
    }
    fn bit_or(self, r: i8) -> u16 {
        (self as u16) | (r as u16)
    }
    fn bit_xor(self, r: i8) -> u16 {
        (self as u16) ^ (r as u16)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u8> for i16 {
    type Output = u16;
    fn bit_and(self, r: u8) -> u16 {
        (self as u16) & (r as u16)
    }
    fn bit_or(self, r: u8) -> u16 {
        (self as u16) | (r as u16)
    }
    fn bit_xor(self, r: u8) -> u16 {
        (self as u16) ^ (r as u16)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i16> for i16 {
    type Output = u16;
    fn bit_and(self, r: i16) -> u16 {
        (self as u16) & (r as u16)
    }
    fn bit_or(self, r: i16) -> u16 {
        (self as u16) | (r as u16)
    }
    fn bit_xor(self, r: i16) -> u16 {
        (self as u16) ^ (r as u16)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u16> for i16 {
    type Output = u16;
    fn bit_and(self, r: u16) -> u16 {
        (self as u16) & (r as u16)
    }
    fn bit_or(self, r: u16) -> u16 {
        (self as u16) | (r as u16)
    }
    fn bit_xor(self, r: u16) -> u16 {
        (self as u16) ^ (r as u16)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i32> for i16 {
    type Output = u32;
    fn bit_and(self, r: i32) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: i32) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: i32) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u32> for i16 {
    type Output = u32;
    fn bit_and(self, r: u32) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: u32) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: u32) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i64> for i16 {
    type Output = u64;
    fn bit_and(self, r: i64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u64> for i16 {
    type Output = u64;
    fn bit_and(self, r: u64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<isize> for i16 {
    type Output = u32;
    fn bit_and(self, r: isize) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: isize) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: isize) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<usize> for i16 {
    type Output = u32;
    fn bit_and(self, r: usize) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: usize) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: usize) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i8> for u16 {
    type Output = u16;
    fn bit_and(self, r: i8) -> u16 {
        (self as u16) & (r as u16)
    }
    fn bit_or(self, r: i8) -> u16 {
        (self as u16) | (r as u16)
    }
    fn bit_xor(self, r: i8) -> u16 {
        (self as u16) ^ (r as u16)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u8> for u16 {
    type Output = u16;
    fn bit_and(self, r: u8) -> u16 {
        (self as u16) & (r as u16)
    }
    fn bit_or(self, r: u8) -> u16 {
        (self as u16) | (r as u16)
    }
    fn bit_xor(self, r: u8) -> u16 {
        (self as u16) ^ (r as u16)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i16> for u16 {
    type Output = u16;
    fn bit_and(self, r: i16) -> u16 {
        (self as u16) & (r as u16)
    }
    fn bit_or(self, r: i16) -> u16 {
        (self as u16) | (r as u16)
    }
    fn bit_xor(self, r: i16) -> u16 {
        (self as u16) ^ (r as u16)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u16> for u16 {
    type Output = u16;
    fn bit_and(self, r: u16) -> u16 {
        (self as u16) & (r as u16)
    }
    fn bit_or(self, r: u16) -> u16 {
        (self as u16) | (r as u16)
    }
    fn bit_xor(self, r: u16) -> u16 {
        (self as u16) ^ (r as u16)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i32> for u16 {
    type Output = u32;
    fn bit_and(self, r: i32) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: i32) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: i32) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u32> for u16 {
    type Output = u32;
    fn bit_and(self, r: u32) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: u32) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: u32) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i64> for u16 {
    type Output = u64;
    fn bit_and(self, r: i64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u64> for u16 {
    type Output = u64;
    fn bit_and(self, r: u64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<isize> for u16 {
    type Output = u32;
    fn bit_and(self, r: isize) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: isize) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: isize) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<usize> for u16 {
    type Output = u32;
    fn bit_and(self, r: usize) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: usize) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: usize) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i8> for i32 {
    type Output = u32;
    fn bit_and(self, r: i8) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: i8) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: i8) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u8> for i32 {
    type Output = u32;
    fn bit_and(self, r: u8) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: u8) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: u8) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i16> for i32 {
    type Output = u32;
    fn bit_and(self, r: i16) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: i16) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: i16) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u16> for i32 {
    type Output = u32;
    fn bit_and(self, r: u16) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: u16) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: u16) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i32> for i32 {
    type Output = u32;
    fn bit_and(self, r: i32) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: i32) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: i32) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u32> for i32 {
    type Output = u32;
    fn bit_and(self, r: u32) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: u32) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: u32) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i64> for i32 {
    type Output = u64;
    fn bit_and(self, r: i64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u64> for i32 {
    type Output = u64;
    fn bit_and(self, r: u64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<isize> for i32 {
    type Output = u32;
    fn bit_and(self, r: isize) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: isize) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: isize) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<usize> for i32 {
    type Output = u32;
    fn bit_and(self, r: usize) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: usize) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: usize) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i8> for u32 {
    type Output = u32;
    fn bit_and(self, r: i8) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: i8) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: i8) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u8> for u32 {
    type Output = u32;
    fn bit_and(self, r: u8) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: u8) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: u8) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i16> for u32 {
    type Output = u32;
    fn bit_and(self, r: i16) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: i16) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: i16) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u16> for u32 {
    type Output = u32;
    fn bit_and(self, r: u16) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: u16) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: u16) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i32> for u32 {
    type Output = u32;
    fn bit_and(self, r: i32) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: i32) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: i32) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u32> for u32 {
    type Output = u32;
    fn bit_and(self, r: u32) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: u32) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: u32) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i64> for u32 {
    type Output = u64;
    fn bit_and(self, r: i64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u64> for u32 {
    type Output = u64;
    fn bit_and(self, r: u64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<isize> for u32 {
    type Output = u32;
    fn bit_and(self, r: isize) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: isize) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: isize) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<usize> for u32 {
    type Output = u32;
    fn bit_and(self, r: usize) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: usize) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: usize) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i8> for i64 {
    type Output = u64;
    fn bit_and(self, r: i8) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i8) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i8) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u8> for i64 {
    type Output = u64;
    fn bit_and(self, r: u8) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u8) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u8) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i16> for i64 {
    type Output = u64;
    fn bit_and(self, r: i16) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i16) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i16) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u16> for i64 {
    type Output = u64;
    fn bit_and(self, r: u16) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u16) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u16) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i32> for i64 {
    type Output = u64;
    fn bit_and(self, r: i32) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i32) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i32) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u32> for i64 {
    type Output = u64;
    fn bit_and(self, r: u32) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u32) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u32) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i64> for i64 {
    type Output = u64;
    fn bit_and(self, r: i64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u64> for i64 {
    type Output = u64;
    fn bit_and(self, r: u64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<isize> for i64 {
    type Output = u64;
    fn bit_and(self, r: isize) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: isize) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: isize) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<usize> for i64 {
    type Output = u64;
    fn bit_and(self, r: usize) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: usize) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: usize) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i8> for u64 {
    type Output = u64;
    fn bit_and(self, r: i8) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i8) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i8) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u8> for u64 {
    type Output = u64;
    fn bit_and(self, r: u8) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u8) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u8) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i16> for u64 {
    type Output = u64;
    fn bit_and(self, r: i16) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i16) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i16) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u16> for u64 {
    type Output = u64;
    fn bit_and(self, r: u16) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u16) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u16) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i32> for u64 {
    type Output = u64;
    fn bit_and(self, r: i32) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i32) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i32) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u32> for u64 {
    type Output = u64;
    fn bit_and(self, r: u32) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u32) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u32) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i64> for u64 {
    type Output = u64;
    fn bit_and(self, r: i64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u64> for u64 {
    type Output = u64;
    fn bit_and(self, r: u64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<isize> for u64 {
    type Output = u64;
    fn bit_and(self, r: isize) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: isize) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: isize) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<usize> for u64 {
    type Output = u64;
    fn bit_and(self, r: usize) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: usize) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: usize) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i8> for isize {
    type Output = u32;
    fn bit_and(self, r: i8) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: i8) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: i8) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u8> for isize {
    type Output = u32;
    fn bit_and(self, r: u8) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: u8) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: u8) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i16> for isize {
    type Output = u32;
    fn bit_and(self, r: i16) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: i16) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: i16) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u16> for isize {
    type Output = u32;
    fn bit_and(self, r: u16) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: u16) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: u16) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i32> for isize {
    type Output = u32;
    fn bit_and(self, r: i32) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: i32) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: i32) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u32> for isize {
    type Output = u32;
    fn bit_and(self, r: u32) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: u32) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: u32) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i64> for isize {
    type Output = u64;
    fn bit_and(self, r: i64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u64> for isize {
    type Output = u64;
    fn bit_and(self, r: u64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<isize> for isize {
    type Output = u32;
    fn bit_and(self, r: isize) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: isize) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: isize) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<usize> for isize {
    type Output = u32;
    fn bit_and(self, r: usize) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: usize) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: usize) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i8> for usize {
    type Output = u32;
    fn bit_and(self, r: i8) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: i8) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: i8) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u8> for usize {
    type Output = u32;
    fn bit_and(self, r: u8) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: u8) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: u8) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i16> for usize {
    type Output = u32;
    fn bit_and(self, r: i16) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: i16) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: i16) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u16> for usize {
    type Output = u32;
    fn bit_and(self, r: u16) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: u16) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: u16) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i32> for usize {
    type Output = u32;
    fn bit_and(self, r: i32) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: i32) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: i32) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u32> for usize {
    type Output = u32;
    fn bit_and(self, r: u32) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: u32) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: u32) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<i64> for usize {
    type Output = u64;
    fn bit_and(self, r: i64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<u64> for usize {
    type Output = u64;
    fn bit_and(self, r: u64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<isize> for usize {
    type Output = u32;
    fn bit_and(self, r: isize) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: isize) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: isize) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "32")]
impl BitPair<usize> for usize {
    type Output = u32;
    fn bit_and(self, r: usize) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: usize) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: usize) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i8> for i8 {
    type Output = u8;
    fn bit_and(self, r: i8) -> u8 {
        (self as u8) & (r as u8)
    }
    fn bit_or(self, r: i8) -> u8 {
        (self as u8) | (r as u8)
    }
    fn bit_xor(self, r: i8) -> u8 {
        (self as u8) ^ (r as u8)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u8> for i8 {
    type Output = u8;
    fn bit_and(self, r: u8) -> u8 {
        (self as u8) & (r as u8)
    }
    fn bit_or(self, r: u8) -> u8 {
        (self as u8) | (r as u8)
    }
    fn bit_xor(self, r: u8) -> u8 {
        (self as u8) ^ (r as u8)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i16> for i8 {
    type Output = u16;
    fn bit_and(self, r: i16) -> u16 {
        (self as u16) & (r as u16)
    }
    fn bit_or(self, r: i16) -> u16 {
        (self as u16) | (r as u16)
    }
    fn bit_xor(self, r: i16) -> u16 {
        (self as u16) ^ (r as u16)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u16> for i8 {
    type Output = u16;
    fn bit_and(self, r: u16) -> u16 {
        (self as u16) & (r as u16)
    }
    fn bit_or(self, r: u16) -> u16 {
        (self as u16) | (r as u16)
    }
    fn bit_xor(self, r: u16) -> u16 {
        (self as u16) ^ (r as u16)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i32> for i8 {
    type Output = u32;
    fn bit_and(self, r: i32) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: i32) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: i32) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u32> for i8 {
    type Output = u32;
    fn bit_and(self, r: u32) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: u32) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: u32) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i64> for i8 {
    type Output = u64;
    fn bit_and(self, r: i64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u64> for i8 {
    type Output = u64;
    fn bit_and(self, r: u64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<isize> for i8 {
    type Output = u64;
    fn bit_and(self, r: isize) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: isize) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: isize) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<usize> for i8 {
    type Output = u64;
    fn bit_and(self, r: usize) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: usize) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: usize) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i8> for u8 {
    type Output = u8;
    fn bit_and(self, r: i8) -> u8 {
        (self as u8) & (r as u8)
    }
    fn bit_or(self, r: i8) -> u8 {
        (self as u8) | (r as u8)
    }
    fn bit_xor(self, r: i8) -> u8 {
        (self as u8) ^ (r as u8)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u8> for u8 {
    type Output = u8;
    fn bit_and(self, r: u8) -> u8 {
        (self as u8) & (r as u8)
    }
    fn bit_or(self, r: u8) -> u8 {
        (self as u8) | (r as u8)
    }
    fn bit_xor(self, r: u8) -> u8 {
        (self as u8) ^ (r as u8)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i16> for u8 {
    type Output = u16;
    fn bit_and(self, r: i16) -> u16 {
        (self as u16) & (r as u16)
    }
    fn bit_or(self, r: i16) -> u16 {
        (self as u16) | (r as u16)
    }
    fn bit_xor(self, r: i16) -> u16 {
        (self as u16) ^ (r as u16)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u16> for u8 {
    type Output = u16;
    fn bit_and(self, r: u16) -> u16 {
        (self as u16) & (r as u16)
    }
    fn bit_or(self, r: u16) -> u16 {
        (self as u16) | (r as u16)
    }
    fn bit_xor(self, r: u16) -> u16 {
        (self as u16) ^ (r as u16)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i32> for u8 {
    type Output = u32;
    fn bit_and(self, r: i32) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: i32) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: i32) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u32> for u8 {
    type Output = u32;
    fn bit_and(self, r: u32) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: u32) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: u32) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i64> for u8 {
    type Output = u64;
    fn bit_and(self, r: i64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u64> for u8 {
    type Output = u64;
    fn bit_and(self, r: u64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<isize> for u8 {
    type Output = u64;
    fn bit_and(self, r: isize) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: isize) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: isize) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<usize> for u8 {
    type Output = u64;
    fn bit_and(self, r: usize) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: usize) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: usize) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i8> for i16 {
    type Output = u16;
    fn bit_and(self, r: i8) -> u16 {
        (self as u16) & (r as u16)
    }
    fn bit_or(self, r: i8) -> u16 {
        (self as u16) | (r as u16)
    }
    fn bit_xor(self, r: i8) -> u16 {
        (self as u16) ^ (r as u16)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u8> for i16 {
    type Output = u16;
    fn bit_and(self, r: u8) -> u16 {
        (self as u16) & (r as u16)
    }
    fn bit_or(self, r: u8) -> u16 {
        (self as u16) | (r as u16)
    }
    fn bit_xor(self, r: u8) -> u16 {
        (self as u16) ^ (r as u16)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i16> for i16 {
    type Output = u16;
    fn bit_and(self, r: i16) -> u16 {
        (self as u16) & (r as u16)
    }
    fn bit_or(self, r: i16) -> u16 {
        (self as u16) | (r as u16)
    }
    fn bit_xor(self, r: i16) -> u16 {
        (self as u16) ^ (r as u16)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u16> for i16 {
    type Output = u16;
    fn bit_and(self, r: u16) -> u16 {
        (self as u16) & (r as u16)
    }
    fn bit_or(self, r: u16) -> u16 {
        (self as u16) | (r as u16)
    }
    fn bit_xor(self, r: u16) -> u16 {
        (self as u16) ^ (r as u16)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i32> for i16 {
    type Output = u32;
    fn bit_and(self, r: i32) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: i32) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: i32) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u32> for i16 {
    type Output = u32;
    fn bit_and(self, r: u32) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: u32) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: u32) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i64> for i16 {
    type Output = u64;
    fn bit_and(self, r: i64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u64> for i16 {
    type Output = u64;
    fn bit_and(self, r: u64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<isize> for i16 {
    type Output = u64;
    fn bit_and(self, r: isize) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: isize) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: isize) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<usize> for i16 {
    type Output = u64;
    fn bit_and(self, r: usize) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: usize) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: usize) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i8> for u16 {
    type Output = u16;
    fn bit_and(self, r: i8) -> u16 {
        (self as u16) & (r as u16)
    }
    fn bit_or(self, r: i8) -> u16 {
        (self as u16) | (r as u16)
    }
    fn bit_xor(self, r: i8) -> u16 {
        (self as u16) ^ (r as u16)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u8> for u16 {
    type Output = u16;
    fn bit_and(self, r: u8) -> u16 {
        (self as u16) & (r as u16)
    }
    fn bit_or(self, r: u8) -> u16 {
        (self as u16) | (r as u16)
    }
    fn bit_xor(self, r: u8) -> u16 {
        (self as u16) ^ (r as u16)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i16> for u16 {
    type Output = u16;
    fn bit_and(self, r: i16) -> u16 {
        (self as u16) & (r as u16)
    }
    fn bit_or(self, r: i16) -> u16 {
        (self as u16) | (r as u16)
    }
    fn bit_xor(self, r: i16) -> u16 {
        (self as u16) ^ (r as u16)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u16> for u16 {
    type Output = u16;
    fn bit_and(self, r: u16) -> u16 {
        (self as u16) & (r as u16)
    }
    fn bit_or(self, r: u16) -> u16 {
        (self as u16) | (r as u16)
    }
    fn bit_xor(self, r: u16) -> u16 {
        (self as u16) ^ (r as u16)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i32> for u16 {
    type Output = u32;
    fn bit_and(self, r: i32) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: i32) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: i32) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u32> for u16 {
    type Output = u32;
    fn bit_and(self, r: u32) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: u32) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: u32) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i64> for u16 {
    type Output = u64;
    fn bit_and(self, r: i64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u64> for u16 {
    type Output = u64;
    fn bit_and(self, r: u64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<isize> for u16 {
    type Output = u64;
    fn bit_and(self, r: isize) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: isize) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: isize) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<usize> for u16 {
    type Output = u64;
    fn bit_and(self, r: usize) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: usize) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: usize) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i8> for i32 {
    type Output = u32;
    fn bit_and(self, r: i8) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: i8) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: i8) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u8> for i32 {
    type Output = u32;
    fn bit_and(self, r: u8) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: u8) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: u8) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i16> for i32 {
    type Output = u32;
    fn bit_and(self, r: i16) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: i16) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: i16) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u16> for i32 {
    type Output = u32;
    fn bit_and(self, r: u16) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: u16) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: u16) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i32> for i32 {
    type Output = u32;
    fn bit_and(self, r: i32) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: i32) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: i32) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u32> for i32 {
    type Output = u32;
    fn bit_and(self, r: u32) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: u32) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: u32) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i64> for i32 {
    type Output = u64;
    fn bit_and(self, r: i64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u64> for i32 {
    type Output = u64;
    fn bit_and(self, r: u64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<isize> for i32 {
    type Output = u64;
    fn bit_and(self, r: isize) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: isize) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: isize) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<usize> for i32 {
    type Output = u64;
    fn bit_and(self, r: usize) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: usize) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: usize) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i8> for u32 {
    type Output = u32;
    fn bit_and(self, r: i8) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: i8) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: i8) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u8> for u32 {
    type Output = u32;
    fn bit_and(self, r: u8) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: u8) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: u8) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i16> for u32 {
    type Output = u32;
    fn bit_and(self, r: i16) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: i16) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: i16) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u16> for u32 {
    type Output = u32;
    fn bit_and(self, r: u16) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: u16) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: u16) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i32> for u32 {
    type Output = u32;
    fn bit_and(self, r: i32) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: i32) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: i32) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u32> for u32 {
    type Output = u32;
    fn bit_and(self, r: u32) -> u32 {
        (self as u32) & (r as u32)
    }
    fn bit_or(self, r: u32) -> u32 {
        (self as u32) | (r as u32)
    }
    fn bit_xor(self, r: u32) -> u32 {
        (self as u32) ^ (r as u32)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i64> for u32 {
    type Output = u64;
    fn bit_and(self, r: i64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u64> for u32 {
    type Output = u64;
    fn bit_and(self, r: u64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<isize> for u32 {
    type Output = u64;
    fn bit_and(self, r: isize) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: isize) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: isize) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<usize> for u32 {
    type Output = u64;
    fn bit_and(self, r: usize) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: usize) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: usize) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i8> for i64 {
    type Output = u64;
    fn bit_and(self, r: i8) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i8) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i8) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u8> for i64 {
    type Output = u64;
    fn bit_and(self, r: u8) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u8) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u8) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i16> for i64 {
    type Output = u64;
    fn bit_and(self, r: i16) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i16) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i16) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u16> for i64 {
    type Output = u64;
    fn bit_and(self, r: u16) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u16) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u16) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i32> for i64 {
    type Output = u64;
    fn bit_and(self, r: i32) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i32) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i32) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u32> for i64 {
    type Output = u64;
    fn bit_and(self, r: u32) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u32) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u32) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i64> for i64 {
    type Output = u64;
    fn bit_and(self, r: i64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u64> for i64 {
    type Output = u64;
    fn bit_and(self, r: u64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<isize> for i64 {
    type Output = u64;
    fn bit_and(self, r: isize) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: isize) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: isize) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<usize> for i64 {
    type Output = u64;
    fn bit_and(self, r: usize) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: usize) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: usize) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i8> for u64 {
    type Output = u64;
    fn bit_and(self, r: i8) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i8) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i8) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u8> for u64 {
    type Output = u64;
    fn bit_and(self, r: u8) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u8) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u8) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i16> for u64 {
    type Output = u64;
    fn bit_and(self, r: i16) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i16) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i16) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u16> for u64 {
    type Output = u64;
    fn bit_and(self, r: u16) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u16) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u16) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i32> for u64 {
    type Output = u64;
    fn bit_and(self, r: i32) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i32) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i32) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u32> for u64 {
    type Output = u64;
    fn bit_and(self, r: u32) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u32) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u32) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i64> for u64 {
    type Output = u64;
    fn bit_and(self, r: i64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u64> for u64 {
    type Output = u64;
    fn bit_and(self, r: u64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<isize> for u64 {
    type Output = u64;
    fn bit_and(self, r: isize) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: isize) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: isize) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<usize> for u64 {
    type Output = u64;
    fn bit_and(self, r: usize) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: usize) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: usize) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i8> for isize {
    type Output = u64;
    fn bit_and(self, r: i8) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i8) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i8) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u8> for isize {
    type Output = u64;
    fn bit_and(self, r: u8) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u8) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u8) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i16> for isize {
    type Output = u64;
    fn bit_and(self, r: i16) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i16) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i16) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u16> for isize {
    type Output = u64;
    fn bit_and(self, r: u16) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u16) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u16) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i32> for isize {
    type Output = u64;
    fn bit_and(self, r: i32) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i32) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i32) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u32> for isize {
    type Output = u64;
    fn bit_and(self, r: u32) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u32) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u32) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i64> for isize {
    type Output = u64;
    fn bit_and(self, r: i64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u64> for isize {
    type Output = u64;
    fn bit_and(self, r: u64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<isize> for isize {
    type Output = u64;
    fn bit_and(self, r: isize) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: isize) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: isize) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<usize> for isize {
    type Output = u64;
    fn bit_and(self, r: usize) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: usize) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: usize) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i8> for usize {
    type Output = u64;
    fn bit_and(self, r: i8) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i8) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i8) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u8> for usize {
    type Output = u64;
    fn bit_and(self, r: u8) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u8) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u8) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i16> for usize {
    type Output = u64;
    fn bit_and(self, r: i16) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i16) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i16) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u16> for usize {
    type Output = u64;
    fn bit_and(self, r: u16) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u16) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u16) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i32> for usize {
    type Output = u64;
    fn bit_and(self, r: i32) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i32) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i32) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u32> for usize {
    type Output = u64;
    fn bit_and(self, r: u32) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u32) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u32) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<i64> for usize {
    type Output = u64;
    fn bit_and(self, r: i64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: i64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: i64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<u64> for usize {
    type Output = u64;
    fn bit_and(self, r: u64) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: u64) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: u64) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<isize> for usize {
    type Output = u64;
    fn bit_and(self, r: isize) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: isize) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: isize) -> u64 {
        (self as u64) ^ (r as u64)
    }
}
#[cfg(target_pointer_width = "64")]
impl BitPair<usize> for usize {
    type Output = u64;
    fn bit_and(self, r: usize) -> u64 {
        (self as u64) & (r as u64)
    }
    fn bit_or(self, r: usize) -> u64 {
        (self as u64) | (r as u64)
    }
    fn bit_xor(self, r: usize) -> u64 {
        (self as u64) ^ (r as u64)
    }
}

// Source: checked_math_impl.h:557-580, i8/f32 float specialization.
impl<V: Numeric> CheckedBinary<f32, V> for i8
where
    f32: Cast<V>,
{
    fn add(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) + (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) - (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) * (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) / (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, i8/f64 float specialization.
impl<V: Numeric> CheckedBinary<f64, V> for i8
where
    f64: Cast<V>,
{
    fn add(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) + (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) - (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) * (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) / (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, u8/f32 float specialization.
impl<V: Numeric> CheckedBinary<f32, V> for u8
where
    f32: Cast<V>,
{
    fn add(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) + (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) - (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) * (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) / (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, u8/f64 float specialization.
impl<V: Numeric> CheckedBinary<f64, V> for u8
where
    f64: Cast<V>,
{
    fn add(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) + (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) - (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) * (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) / (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, i16/f32 float specialization.
impl<V: Numeric> CheckedBinary<f32, V> for i16
where
    f32: Cast<V>,
{
    fn add(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) + (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) - (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) * (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) / (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, i16/f64 float specialization.
impl<V: Numeric> CheckedBinary<f64, V> for i16
where
    f64: Cast<V>,
{
    fn add(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) + (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) - (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) * (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) / (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, u16/f32 float specialization.
impl<V: Numeric> CheckedBinary<f32, V> for u16
where
    f32: Cast<V>,
{
    fn add(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) + (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) - (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) * (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) / (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, u16/f64 float specialization.
impl<V: Numeric> CheckedBinary<f64, V> for u16
where
    f64: Cast<V>,
{
    fn add(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) + (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) - (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) * (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) / (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, i32/f32 float specialization.
impl<V: Numeric> CheckedBinary<f32, V> for i32
where
    f32: Cast<V>,
{
    fn add(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) + (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) - (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) * (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) / (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, i32/f64 float specialization.
impl<V: Numeric> CheckedBinary<f64, V> for i32
where
    f64: Cast<V>,
{
    fn add(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) + (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) - (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) * (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) / (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, u32/f32 float specialization.
impl<V: Numeric> CheckedBinary<f32, V> for u32
where
    f32: Cast<V>,
{
    fn add(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) + (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) - (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) * (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) / (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, u32/f64 float specialization.
impl<V: Numeric> CheckedBinary<f64, V> for u32
where
    f64: Cast<V>,
{
    fn add(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) + (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) - (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) * (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) / (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, i64/f32 float specialization.
impl<V: Numeric> CheckedBinary<f32, V> for i64
where
    f32: Cast<V>,
{
    fn add(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) + (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) - (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) * (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) / (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, i64/f64 float specialization.
impl<V: Numeric> CheckedBinary<f64, V> for i64
where
    f64: Cast<V>,
{
    fn add(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) + (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) - (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) * (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) / (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, u64/f32 float specialization.
impl<V: Numeric> CheckedBinary<f32, V> for u64
where
    f32: Cast<V>,
{
    fn add(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) + (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) - (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) * (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) / (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, u64/f64 float specialization.
impl<V: Numeric> CheckedBinary<f64, V> for u64
where
    f64: Cast<V>,
{
    fn add(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) + (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) - (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) * (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) / (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, isize/f32 float specialization.
impl<V: Numeric> CheckedBinary<f32, V> for isize
where
    f32: Cast<V>,
{
    fn add(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) + (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) - (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) * (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) / (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, isize/f64 float specialization.
impl<V: Numeric> CheckedBinary<f64, V> for isize
where
    f64: Cast<V>,
{
    fn add(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) + (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) - (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) * (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) / (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, usize/f32 float specialization.
impl<V: Numeric> CheckedBinary<f32, V> for usize
where
    f32: Cast<V>,
{
    fn add(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) + (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) - (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) * (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) / (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, usize/f64 float specialization.
impl<V: Numeric> CheckedBinary<f64, V> for usize
where
    f64: Cast<V>,
{
    fn add(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) + (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) - (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) * (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) / (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, f32/i8 float specialization.
impl<V: Numeric> CheckedBinary<i8, V> for f32
where
    f32: Cast<V>,
{
    fn add(self, r: i8, out: &mut V) -> bool {
        let result = (self as f32) + (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i8, out: &mut V) -> bool {
        let result = (self as f32) - (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i8, out: &mut V) -> bool {
        let result = (self as f32) * (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i8, out: &mut V) -> bool {
        let result = (self as f32) / (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, f32/u8 float specialization.
impl<V: Numeric> CheckedBinary<u8, V> for f32
where
    f32: Cast<V>,
{
    fn add(self, r: u8, out: &mut V) -> bool {
        let result = (self as f32) + (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u8, out: &mut V) -> bool {
        let result = (self as f32) - (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u8, out: &mut V) -> bool {
        let result = (self as f32) * (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u8, out: &mut V) -> bool {
        let result = (self as f32) / (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, f32/i16 float specialization.
impl<V: Numeric> CheckedBinary<i16, V> for f32
where
    f32: Cast<V>,
{
    fn add(self, r: i16, out: &mut V) -> bool {
        let result = (self as f32) + (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i16, out: &mut V) -> bool {
        let result = (self as f32) - (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i16, out: &mut V) -> bool {
        let result = (self as f32) * (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i16, out: &mut V) -> bool {
        let result = (self as f32) / (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, f32/u16 float specialization.
impl<V: Numeric> CheckedBinary<u16, V> for f32
where
    f32: Cast<V>,
{
    fn add(self, r: u16, out: &mut V) -> bool {
        let result = (self as f32) + (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u16, out: &mut V) -> bool {
        let result = (self as f32) - (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u16, out: &mut V) -> bool {
        let result = (self as f32) * (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u16, out: &mut V) -> bool {
        let result = (self as f32) / (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, f32/i32 float specialization.
impl<V: Numeric> CheckedBinary<i32, V> for f32
where
    f32: Cast<V>,
{
    fn add(self, r: i32, out: &mut V) -> bool {
        let result = (self as f32) + (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i32, out: &mut V) -> bool {
        let result = (self as f32) - (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i32, out: &mut V) -> bool {
        let result = (self as f32) * (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i32, out: &mut V) -> bool {
        let result = (self as f32) / (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, f32/u32 float specialization.
impl<V: Numeric> CheckedBinary<u32, V> for f32
where
    f32: Cast<V>,
{
    fn add(self, r: u32, out: &mut V) -> bool {
        let result = (self as f32) + (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u32, out: &mut V) -> bool {
        let result = (self as f32) - (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u32, out: &mut V) -> bool {
        let result = (self as f32) * (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u32, out: &mut V) -> bool {
        let result = (self as f32) / (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, f32/i64 float specialization.
impl<V: Numeric> CheckedBinary<i64, V> for f32
where
    f32: Cast<V>,
{
    fn add(self, r: i64, out: &mut V) -> bool {
        let result = (self as f32) + (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i64, out: &mut V) -> bool {
        let result = (self as f32) - (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i64, out: &mut V) -> bool {
        let result = (self as f32) * (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i64, out: &mut V) -> bool {
        let result = (self as f32) / (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, f32/u64 float specialization.
impl<V: Numeric> CheckedBinary<u64, V> for f32
where
    f32: Cast<V>,
{
    fn add(self, r: u64, out: &mut V) -> bool {
        let result = (self as f32) + (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u64, out: &mut V) -> bool {
        let result = (self as f32) - (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u64, out: &mut V) -> bool {
        let result = (self as f32) * (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u64, out: &mut V) -> bool {
        let result = (self as f32) / (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, f32/isize float specialization.
impl<V: Numeric> CheckedBinary<isize, V> for f32
where
    f32: Cast<V>,
{
    fn add(self, r: isize, out: &mut V) -> bool {
        let result = (self as f32) + (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: isize, out: &mut V) -> bool {
        let result = (self as f32) - (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: isize, out: &mut V) -> bool {
        let result = (self as f32) * (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: isize, out: &mut V) -> bool {
        let result = (self as f32) / (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, f32/usize float specialization.
impl<V: Numeric> CheckedBinary<usize, V> for f32
where
    f32: Cast<V>,
{
    fn add(self, r: usize, out: &mut V) -> bool {
        let result = (self as f32) + (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: usize, out: &mut V) -> bool {
        let result = (self as f32) - (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: usize, out: &mut V) -> bool {
        let result = (self as f32) * (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: usize, out: &mut V) -> bool {
        let result = (self as f32) / (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, f32/f32 float specialization.
impl<V: Numeric> CheckedBinary<f32, V> for f32
where
    f32: Cast<V>,
{
    fn add(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) + (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) - (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) * (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: f32, out: &mut V) -> bool {
        let result = (self as f32) / (r as f32);
        if !<f32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f32 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, f32/f64 float specialization.
impl<V: Numeric> CheckedBinary<f64, V> for f32
where
    f64: Cast<V>,
{
    fn add(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) + (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) - (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) * (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) / (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, f64/i8 float specialization.
impl<V: Numeric> CheckedBinary<i8, V> for f64
where
    f64: Cast<V>,
{
    fn add(self, r: i8, out: &mut V) -> bool {
        let result = (self as f64) + (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i8, out: &mut V) -> bool {
        let result = (self as f64) - (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i8, out: &mut V) -> bool {
        let result = (self as f64) * (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i8, out: &mut V) -> bool {
        let result = (self as f64) / (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, f64/u8 float specialization.
impl<V: Numeric> CheckedBinary<u8, V> for f64
where
    f64: Cast<V>,
{
    fn add(self, r: u8, out: &mut V) -> bool {
        let result = (self as f64) + (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u8, out: &mut V) -> bool {
        let result = (self as f64) - (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u8, out: &mut V) -> bool {
        let result = (self as f64) * (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u8, out: &mut V) -> bool {
        let result = (self as f64) / (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, f64/i16 float specialization.
impl<V: Numeric> CheckedBinary<i16, V> for f64
where
    f64: Cast<V>,
{
    fn add(self, r: i16, out: &mut V) -> bool {
        let result = (self as f64) + (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i16, out: &mut V) -> bool {
        let result = (self as f64) - (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i16, out: &mut V) -> bool {
        let result = (self as f64) * (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i16, out: &mut V) -> bool {
        let result = (self as f64) / (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, f64/u16 float specialization.
impl<V: Numeric> CheckedBinary<u16, V> for f64
where
    f64: Cast<V>,
{
    fn add(self, r: u16, out: &mut V) -> bool {
        let result = (self as f64) + (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u16, out: &mut V) -> bool {
        let result = (self as f64) - (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u16, out: &mut V) -> bool {
        let result = (self as f64) * (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u16, out: &mut V) -> bool {
        let result = (self as f64) / (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, f64/i32 float specialization.
impl<V: Numeric> CheckedBinary<i32, V> for f64
where
    f64: Cast<V>,
{
    fn add(self, r: i32, out: &mut V) -> bool {
        let result = (self as f64) + (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i32, out: &mut V) -> bool {
        let result = (self as f64) - (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i32, out: &mut V) -> bool {
        let result = (self as f64) * (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i32, out: &mut V) -> bool {
        let result = (self as f64) / (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, f64/u32 float specialization.
impl<V: Numeric> CheckedBinary<u32, V> for f64
where
    f64: Cast<V>,
{
    fn add(self, r: u32, out: &mut V) -> bool {
        let result = (self as f64) + (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u32, out: &mut V) -> bool {
        let result = (self as f64) - (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u32, out: &mut V) -> bool {
        let result = (self as f64) * (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u32, out: &mut V) -> bool {
        let result = (self as f64) / (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, f64/i64 float specialization.
impl<V: Numeric> CheckedBinary<i64, V> for f64
where
    f64: Cast<V>,
{
    fn add(self, r: i64, out: &mut V) -> bool {
        let result = (self as f64) + (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i64, out: &mut V) -> bool {
        let result = (self as f64) - (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i64, out: &mut V) -> bool {
        let result = (self as f64) * (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i64, out: &mut V) -> bool {
        let result = (self as f64) / (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, f64/u64 float specialization.
impl<V: Numeric> CheckedBinary<u64, V> for f64
where
    f64: Cast<V>,
{
    fn add(self, r: u64, out: &mut V) -> bool {
        let result = (self as f64) + (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u64, out: &mut V) -> bool {
        let result = (self as f64) - (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u64, out: &mut V) -> bool {
        let result = (self as f64) * (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u64, out: &mut V) -> bool {
        let result = (self as f64) / (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, f64/isize float specialization.
impl<V: Numeric> CheckedBinary<isize, V> for f64
where
    f64: Cast<V>,
{
    fn add(self, r: isize, out: &mut V) -> bool {
        let result = (self as f64) + (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: isize, out: &mut V) -> bool {
        let result = (self as f64) - (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: isize, out: &mut V) -> bool {
        let result = (self as f64) * (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: isize, out: &mut V) -> bool {
        let result = (self as f64) / (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, f64/usize float specialization.
impl<V: Numeric> CheckedBinary<usize, V> for f64
where
    f64: Cast<V>,
{
    fn add(self, r: usize, out: &mut V) -> bool {
        let result = (self as f64) + (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: usize, out: &mut V) -> bool {
        let result = (self as f64) - (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: usize, out: &mut V) -> bool {
        let result = (self as f64) * (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: usize, out: &mut V) -> bool {
        let result = (self as f64) / (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, f64/f32 float specialization.
impl<V: Numeric> CheckedBinary<f32, V> for f64
where
    f64: Cast<V>,
{
    fn add(self, r: f32, out: &mut V) -> bool {
        let result = (self as f64) + (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: f32, out: &mut V) -> bool {
        let result = (self as f64) - (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: f32, out: &mut V) -> bool {
        let result = (self as f64) * (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: f32, out: &mut V) -> bool {
        let result = (self as f64) / (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
}

// Source: checked_math_impl.h:557-580, f64/f64 float specialization.
impl<V: Numeric> CheckedBinary<f64, V> for f64
where
    f64: Cast<V>,
{
    fn add(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) + (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) - (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) * (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: f64, out: &mut V) -> bool {
        let result = (self as f64) / (r as f64);
        if !<f64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <f64 as Cast<V>>::cast(result);
        true
    }
}

impl StateValue for bool {
    type Storage = IntegerState<Self>;
    fn state_from<S: Numeric + Cast<Self>>(v: S, valid: bool) -> Self::Storage {
        let ok = valid && v.range().IsValid();
        let value = if !S::FLOAT || ok { v.cast() } else { false };
        IntegerState::from(value, ok)
    }
}

// Source: checked_math_impl.h:99-354. Native per-pair template instantiations.
use super::native_overflow_builtins::BuiltinTriple;
use super::safe_math_shared_impl::{CheckedAddFastOp, CheckedMulFastOp, CheckedSubFastOp};
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i8, V> for i8
where
    i8: BuiltinTriple<i8, V>,
    i16: Cast<V>,
    i16: Cast<V>,
    i8: Cast<V>,
{
    fn add(self, r: i8, out: &mut V) -> bool {
        if CheckedAddFastOp::<i8, i8>::is_supported {
            return unsafe { CheckedAddFastOp::<i8, i8>::Do(self, r, out) };
        }
        if !<i8 as Cast<i16>>::range(self).IsValid() || !<i8 as Cast<i16>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i16;
        let valid = i16_ops::CheckedAddImpl(self as i16, r as i16, &mut result);
        if !valid || !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i8, out: &mut V) -> bool {
        if CheckedSubFastOp::<i8, i8>::is_supported {
            return unsafe { CheckedSubFastOp::<i8, i8>::Do(self, r, out) };
        }
        if !<i8 as Cast<i16>>::range(self).IsValid() || !<i8 as Cast<i16>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i16;
        let valid = i16_ops::CheckedSubImpl(self as i16, r as i16, &mut result);
        if !valid || !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i8, out: &mut V) -> bool {
        if CheckedMulFastOp::<i8, i8>::is_supported {
            return unsafe { CheckedMulFastOp::<i8, i8>::Do(self, r, out) };
        }
        if (!<i8 as Cast<i16>>::range(self).IsValid() || !<i8 as Cast<i16>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i16;
        let valid = if CheckedMulFastOp::<i16, i16>::is_supported {
            unsafe { CheckedMulFastOp::<i16, i16>::Do(self as i16, r as i16, &mut result) }
        } else {
            i16_ops::CheckedMulImpl(self as i16, r as i16, &mut result)
        };
        if !valid || !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i8 as Cast<i8>>::CONTAINED && (self as i8) == i8::MIN && r == (-1i64 as i8) {
            return false;
        }
        if (!<i8 as Cast<i8>>::range(self).IsValid() || !<i8 as Cast<i8>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i8) / (r as i8);
        if !<i8 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i8 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i8, V> for i8
where
    i8: Cast<V>,
{
    fn modulo(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i8 as Cast<i8>>::CONTAINED && (self as i8) == i8::MIN && r == (-1i64 as i8) {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i8) % (r as i8);
        if !<i8 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i8 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u8, V> for i8
where
    i8: BuiltinTriple<u8, V>,
    i16: Cast<V>,
    i16: Cast<V>,
    i16: Cast<V>,
{
    fn add(self, r: u8, out: &mut V) -> bool {
        if CheckedAddFastOp::<i8, u8>::is_supported {
            return unsafe { CheckedAddFastOp::<i8, u8>::Do(self, r, out) };
        }
        if !<i8 as Cast<i16>>::range(self).IsValid() || !<u8 as Cast<i16>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i16;
        let valid = i16_ops::CheckedAddImpl(self as i16, r as i16, &mut result);
        if !valid || !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u8, out: &mut V) -> bool {
        if CheckedSubFastOp::<i8, u8>::is_supported {
            return unsafe { CheckedSubFastOp::<i8, u8>::Do(self, r, out) };
        }
        if !<i8 as Cast<i16>>::range(self).IsValid() || !<u8 as Cast<i16>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i16;
        let valid = i16_ops::CheckedSubImpl(self as i16, r as i16, &mut result);
        if !valid || !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u8, out: &mut V) -> bool {
        if CheckedMulFastOp::<i8, u8>::is_supported {
            return unsafe { CheckedMulFastOp::<i8, u8>::Do(self, r, out) };
        }
        if (!<i8 as Cast<i16>>::range(self).IsValid() || !<u8 as Cast<i16>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i16;
        let valid = if CheckedMulFastOp::<i16, i16>::is_supported {
            unsafe { CheckedMulFastOp::<i16, i16>::Do(self as i16, r as i16, &mut result) }
        } else {
            i16_ops::CheckedMulImpl(self as i16, r as i16, &mut result)
        };
        if !valid || !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i16 as Cast<i8>>::CONTAINED && (self as i16) == i16::MIN && r == (-1i64 as u8)
        {
            return false;
        }
        if (!<i8 as Cast<i16>>::range(self).IsValid() || !<u8 as Cast<i16>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i16) / (r as i16);
        if !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u8, V> for i8
where
    i16: Cast<V>,
{
    fn modulo(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i16 as Cast<i8>>::CONTAINED && (self as i16) == i16::MIN && r == (-1i64 as u8)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i16) % (r as i16);
        if !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i16, V> for i8
where
    i8: BuiltinTriple<i16, V>,
    i32: Cast<V>,
    i32: Cast<V>,
    i16: Cast<V>,
{
    fn add(self, r: i16, out: &mut V) -> bool {
        if CheckedAddFastOp::<i8, i16>::is_supported {
            return unsafe { CheckedAddFastOp::<i8, i16>::Do(self, r, out) };
        }
        if !<i8 as Cast<i32>>::range(self).IsValid() || !<i16 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedAddImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i16, out: &mut V) -> bool {
        if CheckedSubFastOp::<i8, i16>::is_supported {
            return unsafe { CheckedSubFastOp::<i8, i16>::Do(self, r, out) };
        }
        if !<i8 as Cast<i32>>::range(self).IsValid() || !<i16 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedSubImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i16, out: &mut V) -> bool {
        if CheckedMulFastOp::<i8, i16>::is_supported {
            return unsafe { CheckedMulFastOp::<i8, i16>::Do(self, r, out) };
        }
        if (!<i8 as Cast<i32>>::range(self).IsValid() || !<i16 as Cast<i32>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i32;
        let valid = if CheckedMulFastOp::<i32, i32>::is_supported {
            unsafe { CheckedMulFastOp::<i32, i32>::Do(self as i32, r as i32, &mut result) }
        } else {
            i32_ops::CheckedMulImpl(self as i32, r as i32, &mut result)
        };
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i16 as Cast<i8>>::CONTAINED && (self as i16) == i16::MIN && r == (-1i64 as i16)
        {
            return false;
        }
        if (!<i8 as Cast<i16>>::range(self).IsValid() || !<i16 as Cast<i16>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i16) / (r as i16);
        if !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i16, V> for i8
where
    i16: Cast<V>,
{
    fn modulo(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i16 as Cast<i8>>::CONTAINED && (self as i16) == i16::MIN && r == (-1i64 as i16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i16) % (r as i16);
        if !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u16, V> for i8
where
    i8: BuiltinTriple<u16, V>,
    i32: Cast<V>,
    i32: Cast<V>,
    i32: Cast<V>,
{
    fn add(self, r: u16, out: &mut V) -> bool {
        if CheckedAddFastOp::<i8, u16>::is_supported {
            return unsafe { CheckedAddFastOp::<i8, u16>::Do(self, r, out) };
        }
        if !<i8 as Cast<i32>>::range(self).IsValid() || !<u16 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedAddImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u16, out: &mut V) -> bool {
        if CheckedSubFastOp::<i8, u16>::is_supported {
            return unsafe { CheckedSubFastOp::<i8, u16>::Do(self, r, out) };
        }
        if !<i8 as Cast<i32>>::range(self).IsValid() || !<u16 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedSubImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u16, out: &mut V) -> bool {
        if CheckedMulFastOp::<i8, u16>::is_supported {
            return unsafe { CheckedMulFastOp::<i8, u16>::Do(self, r, out) };
        }
        if (!<i8 as Cast<i32>>::range(self).IsValid() || !<u16 as Cast<i32>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i32;
        let valid = if CheckedMulFastOp::<i32, i32>::is_supported {
            unsafe { CheckedMulFastOp::<i32, i32>::Do(self as i32, r as i32, &mut result) }
        } else {
            i32_ops::CheckedMulImpl(self as i32, r as i32, &mut result)
        };
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i32 as Cast<i8>>::CONTAINED && (self as i32) == i32::MIN && r == (-1i64 as u16)
        {
            return false;
        }
        if (!<i8 as Cast<i32>>::range(self).IsValid() || !<u16 as Cast<i32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i32) / (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u16, V> for i8
where
    i32: Cast<V>,
{
    fn modulo(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i32 as Cast<i8>>::CONTAINED && (self as i32) == i32::MIN && r == (-1i64 as u16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i32) % (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i32, V> for i8
where
    i8: BuiltinTriple<i32, V>,
    i32: Cast<V>,
    i64: Cast<V>,
    i32: Cast<V>,
{
    fn add(self, r: i32, out: &mut V) -> bool {
        if CheckedAddFastOp::<i8, i32>::is_supported {
            return unsafe { CheckedAddFastOp::<i8, i32>::Do(self, r, out) };
        }
        if !<i8 as Cast<i32>>::range(self).IsValid() || !<i32 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedAddImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i32, out: &mut V) -> bool {
        if CheckedSubFastOp::<i8, i32>::is_supported {
            return unsafe { CheckedSubFastOp::<i8, i32>::Do(self, r, out) };
        }
        if !<i8 as Cast<i32>>::range(self).IsValid() || !<i32 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedSubImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i32, out: &mut V) -> bool {
        if CheckedMulFastOp::<i8, i32>::is_supported {
            return unsafe { CheckedMulFastOp::<i8, i32>::Do(self, r, out) };
        }
        if (!<i8 as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i32 as Cast<i8>>::CONTAINED && (self as i32) == i32::MIN && r == (-1i64 as i32)
        {
            return false;
        }
        if (!<i8 as Cast<i32>>::range(self).IsValid() || !<i32 as Cast<i32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i32) / (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i32, V> for i8
where
    i32: Cast<V>,
{
    fn modulo(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i32 as Cast<i8>>::CONTAINED && (self as i32) == i32::MIN && r == (-1i64 as i32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i32) % (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u32, V> for i8
where
    i8: BuiltinTriple<u32, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: u32, out: &mut V) -> bool {
        if CheckedAddFastOp::<i8, u32>::is_supported {
            return unsafe { CheckedAddFastOp::<i8, u32>::Do(self, r, out) };
        }
        if !<i8 as Cast<i64>>::range(self).IsValid() || !<u32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u32, out: &mut V) -> bool {
        if CheckedSubFastOp::<i8, u32>::is_supported {
            return unsafe { CheckedSubFastOp::<i8, u32>::Do(self, r, out) };
        }
        if !<i8 as Cast<i64>>::range(self).IsValid() || !<u32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u32, out: &mut V) -> bool {
        if CheckedMulFastOp::<i8, u32>::is_supported {
            return unsafe { CheckedMulFastOp::<i8, u32>::Do(self, r, out) };
        }
        if (!<i8 as Cast<i64>>::range(self).IsValid() || !<u32 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i64 as Cast<i8>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as u32)
        {
            return false;
        }
        if (!<i8 as Cast<i64>>::range(self).IsValid() || !<u32 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u32, V> for i8
where
    i64: Cast<V>,
{
    fn modulo(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i64 as Cast<i8>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as u32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i64, V> for i8
where
    i8: BuiltinTriple<i64, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: i64, out: &mut V) -> bool {
        if CheckedAddFastOp::<i8, i64>::is_supported {
            return unsafe { CheckedAddFastOp::<i8, i64>::Do(self, r, out) };
        }
        if !<i8 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i64, out: &mut V) -> bool {
        if CheckedSubFastOp::<i8, i64>::is_supported {
            return unsafe { CheckedSubFastOp::<i8, i64>::Do(self, r, out) };
        }
        if !<i8 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i64, out: &mut V) -> bool {
        if CheckedMulFastOp::<i8, i64>::is_supported {
            return unsafe { CheckedMulFastOp::<i8, i64>::Do(self, r, out) };
        }
        if (!<i8 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i64 as Cast<i8>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as i64)
        {
            return false;
        }
        if (!<i8 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i64, V> for i8
where
    i64: Cast<V>,
{
    fn modulo(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i64 as Cast<i8>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as i64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u64, V> for i8
where
    i8: BuiltinTriple<u64, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u64: Cast<V>,
{
    fn add(self, r: u64, out: &mut V) -> bool {
        if CheckedAddFastOp::<i8, u64>::is_supported {
            return unsafe { CheckedAddFastOp::<i8, u64>::Do(self, r, out) };
        }
        if !<i8 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u64, out: &mut V) -> bool {
        if CheckedSubFastOp::<i8, u64>::is_supported {
            return unsafe { CheckedSubFastOp::<i8, u64>::Do(self, r, out) };
        }
        if !<i8 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u64, out: &mut V) -> bool {
        if CheckedMulFastOp::<i8, u64>::is_supported {
            return unsafe { CheckedMulFastOp::<i8, u64>::Do(self, r, out) };
        }
        if (!<i8 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <u64 as Cast<i8>>::CONTAINED && (self as u64) == u64::MIN && r == (-1i64 as u64)
        {
            return false;
        }
        if (!<i8 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u64) / (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u64, V> for i8
where
    u64: Cast<V>,
{
    fn modulo(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <u64 as Cast<i8>>::CONTAINED && (self as u64) == u64::MIN && r == (-1i64 as u64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u64) % (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<isize, V> for i8
where
    i8: BuiltinTriple<isize, V>,
    isize: Cast<V>,
    i64: Cast<V>,
    isize: Cast<V>,
{
    fn add(self, r: isize, out: &mut V) -> bool {
        if CheckedAddFastOp::<i8, isize>::is_supported {
            return unsafe { CheckedAddFastOp::<i8, isize>::Do(self, r, out) };
        }
        if !<i8 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedAddImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: isize, out: &mut V) -> bool {
        if CheckedSubFastOp::<i8, isize>::is_supported {
            return unsafe { CheckedSubFastOp::<i8, isize>::Do(self, r, out) };
        }
        if !<i8 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedSubImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: isize, out: &mut V) -> bool {
        if CheckedMulFastOp::<i8, isize>::is_supported {
            return unsafe { CheckedMulFastOp::<i8, isize>::Do(self, r, out) };
        }
        if (!<i8 as Cast<i64>>::range(self).IsValid() || !<isize as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true
            && <isize as Cast<i8>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as isize)
        {
            return false;
        }
        if (!<i8 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as isize) / (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<isize, V> for i8
where
    isize: Cast<V>,
{
    fn modulo(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true
            && <isize as Cast<i8>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as isize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as isize) % (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<usize, V> for i8
where
    i8: BuiltinTriple<usize, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: usize, out: &mut V) -> bool {
        if CheckedAddFastOp::<i8, usize>::is_supported {
            return unsafe { CheckedAddFastOp::<i8, usize>::Do(self, r, out) };
        }
        if !<i8 as Cast<i64>>::range(self).IsValid() || !<usize as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: usize, out: &mut V) -> bool {
        if CheckedSubFastOp::<i8, usize>::is_supported {
            return unsafe { CheckedSubFastOp::<i8, usize>::Do(self, r, out) };
        }
        if !<i8 as Cast<i64>>::range(self).IsValid() || !<usize as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: usize, out: &mut V) -> bool {
        if CheckedMulFastOp::<i8, usize>::is_supported {
            return unsafe { CheckedMulFastOp::<i8, usize>::Do(self, r, out) };
        }
        if (!<i8 as Cast<i64>>::range(self).IsValid() || !<usize as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<i8>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as usize)
        {
            return false;
        }
        if (!<i8 as Cast<i64>>::range(self).IsValid() || !<usize as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<usize, V> for i8
where
    i64: Cast<V>,
{
    fn modulo(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<i8>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as usize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i8, V> for u8
where
    u8: BuiltinTriple<i8, V>,
    i16: Cast<V>,
    i16: Cast<V>,
    i16: Cast<V>,
{
    fn add(self, r: i8, out: &mut V) -> bool {
        if CheckedAddFastOp::<u8, i8>::is_supported {
            return unsafe { CheckedAddFastOp::<u8, i8>::Do(self, r, out) };
        }
        if !<u8 as Cast<i16>>::range(self).IsValid() || !<i8 as Cast<i16>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i16;
        let valid = i16_ops::CheckedAddImpl(self as i16, r as i16, &mut result);
        if !valid || !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i8, out: &mut V) -> bool {
        if CheckedSubFastOp::<u8, i8>::is_supported {
            return unsafe { CheckedSubFastOp::<u8, i8>::Do(self, r, out) };
        }
        if !<u8 as Cast<i16>>::range(self).IsValid() || !<i8 as Cast<i16>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i16;
        let valid = i16_ops::CheckedSubImpl(self as i16, r as i16, &mut result);
        if !valid || !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i8, out: &mut V) -> bool {
        if CheckedMulFastOp::<u8, i8>::is_supported {
            return unsafe { CheckedMulFastOp::<u8, i8>::Do(self, r, out) };
        }
        if (!<u8 as Cast<i16>>::range(self).IsValid() || !<i8 as Cast<i16>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i16;
        let valid = if CheckedMulFastOp::<i16, i16>::is_supported {
            unsafe { CheckedMulFastOp::<i16, i16>::Do(self as i16, r as i16, &mut result) }
        } else {
            i16_ops::CheckedMulImpl(self as i16, r as i16, &mut result)
        };
        if !valid || !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i16 as Cast<u8>>::CONTAINED && (self as i16) == i16::MIN && r == (-1i64 as i8)
        {
            return false;
        }
        if (!<u8 as Cast<i16>>::range(self).IsValid() || !<i8 as Cast<i16>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i16) / (r as i16);
        if !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i8, V> for u8
where
    i16: Cast<V>,
{
    fn modulo(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i16 as Cast<u8>>::CONTAINED && (self as i16) == i16::MIN && r == (-1i64 as i8)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i16) % (r as i16);
        if !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u8, V> for u8
where
    u8: BuiltinTriple<u8, V>,
    u16: Cast<V>,
    u16: Cast<V>,
    u8: Cast<V>,
{
    fn add(self, r: u8, out: &mut V) -> bool {
        if CheckedAddFastOp::<u8, u8>::is_supported {
            return unsafe { CheckedAddFastOp::<u8, u8>::Do(self, r, out) };
        }
        if !<u8 as Cast<u16>>::range(self).IsValid() || !<u8 as Cast<u16>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u16;
        let valid = u16_ops::CheckedAddImpl(self as u16, r as u16, &mut result);
        if !valid || !<u16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u16 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u8, out: &mut V) -> bool {
        if CheckedSubFastOp::<u8, u8>::is_supported {
            return unsafe { CheckedSubFastOp::<u8, u8>::Do(self, r, out) };
        }
        if !<u8 as Cast<u16>>::range(self).IsValid() || !<u8 as Cast<u16>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u16;
        let valid = u16_ops::CheckedSubImpl(self as u16, r as u16, &mut result);
        if !valid || !<u16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u16 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u8, out: &mut V) -> bool {
        if CheckedMulFastOp::<u8, u8>::is_supported {
            return unsafe { CheckedMulFastOp::<u8, u8>::Do(self, r, out) };
        }
        if (!<u8 as Cast<u16>>::range(self).IsValid() || !<u8 as Cast<u16>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u16;
        let valid = if CheckedMulFastOp::<u16, u16>::is_supported {
            unsafe { CheckedMulFastOp::<u16, u16>::Do(self as u16, r as u16, &mut result) }
        } else {
            u16_ops::CheckedMulImpl(self as u16, r as u16, &mut result)
        };
        if !valid || !<u16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u16 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <u8 as Cast<u8>>::CONTAINED && (self as u8) == u8::MIN && r == (-1i64 as u8) {
            return false;
        }
        if (!<u8 as Cast<u8>>::range(self).IsValid() || !<u8 as Cast<u8>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u8) / (r as u8);
        if !<u8 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u8 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u8, V> for u8
where
    u8: Cast<V>,
{
    fn modulo(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <u8 as Cast<u8>>::CONTAINED && (self as u8) == u8::MIN && r == (-1i64 as u8) {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u8) % (r as u8);
        if !<u8 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u8 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i16, V> for u8
where
    u8: BuiltinTriple<i16, V>,
    i32: Cast<V>,
    i32: Cast<V>,
    i16: Cast<V>,
{
    fn add(self, r: i16, out: &mut V) -> bool {
        if CheckedAddFastOp::<u8, i16>::is_supported {
            return unsafe { CheckedAddFastOp::<u8, i16>::Do(self, r, out) };
        }
        if !<u8 as Cast<i32>>::range(self).IsValid() || !<i16 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedAddImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i16, out: &mut V) -> bool {
        if CheckedSubFastOp::<u8, i16>::is_supported {
            return unsafe { CheckedSubFastOp::<u8, i16>::Do(self, r, out) };
        }
        if !<u8 as Cast<i32>>::range(self).IsValid() || !<i16 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedSubImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i16, out: &mut V) -> bool {
        if CheckedMulFastOp::<u8, i16>::is_supported {
            return unsafe { CheckedMulFastOp::<u8, i16>::Do(self, r, out) };
        }
        if (!<u8 as Cast<i32>>::range(self).IsValid() || !<i16 as Cast<i32>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i32;
        let valid = if CheckedMulFastOp::<i32, i32>::is_supported {
            unsafe { CheckedMulFastOp::<i32, i32>::Do(self as i32, r as i32, &mut result) }
        } else {
            i32_ops::CheckedMulImpl(self as i32, r as i32, &mut result)
        };
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i16 as Cast<u8>>::CONTAINED && (self as i16) == i16::MIN && r == (-1i64 as i16)
        {
            return false;
        }
        if (!<u8 as Cast<i16>>::range(self).IsValid() || !<i16 as Cast<i16>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i16) / (r as i16);
        if !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i16, V> for u8
where
    i16: Cast<V>,
{
    fn modulo(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i16 as Cast<u8>>::CONTAINED && (self as i16) == i16::MIN && r == (-1i64 as i16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i16) % (r as i16);
        if !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u16, V> for u8
where
    u8: BuiltinTriple<u16, V>,
    u32: Cast<V>,
    u32: Cast<V>,
    u16: Cast<V>,
{
    fn add(self, r: u16, out: &mut V) -> bool {
        if CheckedAddFastOp::<u8, u16>::is_supported {
            return unsafe { CheckedAddFastOp::<u8, u16>::Do(self, r, out) };
        }
        if !<u8 as Cast<u32>>::range(self).IsValid() || !<u16 as Cast<u32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u32;
        let valid = u32_ops::CheckedAddImpl(self as u32, r as u32, &mut result);
        if !valid || !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u16, out: &mut V) -> bool {
        if CheckedSubFastOp::<u8, u16>::is_supported {
            return unsafe { CheckedSubFastOp::<u8, u16>::Do(self, r, out) };
        }
        if !<u8 as Cast<u32>>::range(self).IsValid() || !<u16 as Cast<u32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u32;
        let valid = u32_ops::CheckedSubImpl(self as u32, r as u32, &mut result);
        if !valid || !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u16, out: &mut V) -> bool {
        if CheckedMulFastOp::<u8, u16>::is_supported {
            return unsafe { CheckedMulFastOp::<u8, u16>::Do(self, r, out) };
        }
        if (!<u8 as Cast<u32>>::range(self).IsValid() || !<u16 as Cast<u32>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u32;
        let valid = if CheckedMulFastOp::<u32, u32>::is_supported {
            unsafe { CheckedMulFastOp::<u32, u32>::Do(self as u32, r as u32, &mut result) }
        } else {
            u32_ops::CheckedMulImpl(self as u32, r as u32, &mut result)
        };
        if !valid || !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <u16 as Cast<u8>>::CONTAINED && (self as u16) == u16::MIN && r == (-1i64 as u16)
        {
            return false;
        }
        if (!<u8 as Cast<u16>>::range(self).IsValid() || !<u16 as Cast<u16>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u16) / (r as u16);
        if !<u16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u16, V> for u8
where
    u16: Cast<V>,
{
    fn modulo(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <u16 as Cast<u8>>::CONTAINED && (self as u16) == u16::MIN && r == (-1i64 as u16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u16) % (r as u16);
        if !<u16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i32, V> for u8
where
    u8: BuiltinTriple<i32, V>,
    i32: Cast<V>,
    i64: Cast<V>,
    i32: Cast<V>,
{
    fn add(self, r: i32, out: &mut V) -> bool {
        if CheckedAddFastOp::<u8, i32>::is_supported {
            return unsafe { CheckedAddFastOp::<u8, i32>::Do(self, r, out) };
        }
        if !<u8 as Cast<i32>>::range(self).IsValid() || !<i32 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedAddImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i32, out: &mut V) -> bool {
        if CheckedSubFastOp::<u8, i32>::is_supported {
            return unsafe { CheckedSubFastOp::<u8, i32>::Do(self, r, out) };
        }
        if !<u8 as Cast<i32>>::range(self).IsValid() || !<i32 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedSubImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i32, out: &mut V) -> bool {
        if CheckedMulFastOp::<u8, i32>::is_supported {
            return unsafe { CheckedMulFastOp::<u8, i32>::Do(self, r, out) };
        }
        if (!<u8 as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i32 as Cast<u8>>::CONTAINED && (self as i32) == i32::MIN && r == (-1i64 as i32)
        {
            return false;
        }
        if (!<u8 as Cast<i32>>::range(self).IsValid() || !<i32 as Cast<i32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i32) / (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i32, V> for u8
where
    i32: Cast<V>,
{
    fn modulo(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i32 as Cast<u8>>::CONTAINED && (self as i32) == i32::MIN && r == (-1i64 as i32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i32) % (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u32, V> for u8
where
    u8: BuiltinTriple<u32, V>,
    u32: Cast<V>,
    u64: Cast<V>,
    u32: Cast<V>,
{
    fn add(self, r: u32, out: &mut V) -> bool {
        if CheckedAddFastOp::<u8, u32>::is_supported {
            return unsafe { CheckedAddFastOp::<u8, u32>::Do(self, r, out) };
        }
        if !<u8 as Cast<u32>>::range(self).IsValid() || !<u32 as Cast<u32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u32;
        let valid = u32_ops::CheckedAddImpl(self as u32, r as u32, &mut result);
        if !valid || !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u32, out: &mut V) -> bool {
        if CheckedSubFastOp::<u8, u32>::is_supported {
            return unsafe { CheckedSubFastOp::<u8, u32>::Do(self, r, out) };
        }
        if !<u8 as Cast<u32>>::range(self).IsValid() || !<u32 as Cast<u32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u32;
        let valid = u32_ops::CheckedSubImpl(self as u32, r as u32, &mut result);
        if !valid || !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u32, out: &mut V) -> bool {
        if CheckedMulFastOp::<u8, u32>::is_supported {
            return unsafe { CheckedMulFastOp::<u8, u32>::Do(self, r, out) };
        }
        if (!<u8 as Cast<u64>>::range(self).IsValid() || !<u32 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <u32 as Cast<u8>>::CONTAINED && (self as u32) == u32::MIN && r == (-1i64 as u32)
        {
            return false;
        }
        if (!<u8 as Cast<u32>>::range(self).IsValid() || !<u32 as Cast<u32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u32) / (r as u32);
        if !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u32, V> for u8
where
    u32: Cast<V>,
{
    fn modulo(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <u32 as Cast<u8>>::CONTAINED && (self as u32) == u32::MIN && r == (-1i64 as u32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u32) % (r as u32);
        if !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i64, V> for u8
where
    u8: BuiltinTriple<i64, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: i64, out: &mut V) -> bool {
        if CheckedAddFastOp::<u8, i64>::is_supported {
            return unsafe { CheckedAddFastOp::<u8, i64>::Do(self, r, out) };
        }
        if !<u8 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i64, out: &mut V) -> bool {
        if CheckedSubFastOp::<u8, i64>::is_supported {
            return unsafe { CheckedSubFastOp::<u8, i64>::Do(self, r, out) };
        }
        if !<u8 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i64, out: &mut V) -> bool {
        if CheckedMulFastOp::<u8, i64>::is_supported {
            return unsafe { CheckedMulFastOp::<u8, i64>::Do(self, r, out) };
        }
        if (!<u8 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i64 as Cast<u8>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as i64)
        {
            return false;
        }
        if (!<u8 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i64, V> for u8
where
    i64: Cast<V>,
{
    fn modulo(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i64 as Cast<u8>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as i64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u64, V> for u8
where
    u8: BuiltinTriple<u64, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u64: Cast<V>,
{
    fn add(self, r: u64, out: &mut V) -> bool {
        if CheckedAddFastOp::<u8, u64>::is_supported {
            return unsafe { CheckedAddFastOp::<u8, u64>::Do(self, r, out) };
        }
        if !<u8 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u64, out: &mut V) -> bool {
        if CheckedSubFastOp::<u8, u64>::is_supported {
            return unsafe { CheckedSubFastOp::<u8, u64>::Do(self, r, out) };
        }
        if !<u8 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u64, out: &mut V) -> bool {
        if CheckedMulFastOp::<u8, u64>::is_supported {
            return unsafe { CheckedMulFastOp::<u8, u64>::Do(self, r, out) };
        }
        if (!<u8 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <u64 as Cast<u8>>::CONTAINED && (self as u64) == u64::MIN && r == (-1i64 as u64)
        {
            return false;
        }
        if (!<u8 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u64) / (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u64, V> for u8
where
    u64: Cast<V>,
{
    fn modulo(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <u64 as Cast<u8>>::CONTAINED && (self as u64) == u64::MIN && r == (-1i64 as u64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u64) % (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<isize, V> for u8
where
    u8: BuiltinTriple<isize, V>,
    isize: Cast<V>,
    i64: Cast<V>,
    isize: Cast<V>,
{
    fn add(self, r: isize, out: &mut V) -> bool {
        if CheckedAddFastOp::<u8, isize>::is_supported {
            return unsafe { CheckedAddFastOp::<u8, isize>::Do(self, r, out) };
        }
        if !<u8 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedAddImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: isize, out: &mut V) -> bool {
        if CheckedSubFastOp::<u8, isize>::is_supported {
            return unsafe { CheckedSubFastOp::<u8, isize>::Do(self, r, out) };
        }
        if !<u8 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedSubImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: isize, out: &mut V) -> bool {
        if CheckedMulFastOp::<u8, isize>::is_supported {
            return unsafe { CheckedMulFastOp::<u8, isize>::Do(self, r, out) };
        }
        if (!<u8 as Cast<i64>>::range(self).IsValid() || !<isize as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <isize as Cast<u8>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as isize)
        {
            return false;
        }
        if (!<u8 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as isize) / (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<isize, V> for u8
where
    isize: Cast<V>,
{
    fn modulo(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <isize as Cast<u8>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as isize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as isize) % (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<usize, V> for u8
where
    u8: BuiltinTriple<usize, V>,
    usize: Cast<V>,
    u64: Cast<V>,
    usize: Cast<V>,
{
    fn add(self, r: usize, out: &mut V) -> bool {
        if CheckedAddFastOp::<u8, usize>::is_supported {
            return unsafe { CheckedAddFastOp::<u8, usize>::Do(self, r, out) };
        }
        if !<u8 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedAddImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: usize, out: &mut V) -> bool {
        if CheckedSubFastOp::<u8, usize>::is_supported {
            return unsafe { CheckedSubFastOp::<u8, usize>::Do(self, r, out) };
        }
        if !<u8 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedSubImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: usize, out: &mut V) -> bool {
        if CheckedMulFastOp::<u8, usize>::is_supported {
            return unsafe { CheckedMulFastOp::<u8, usize>::Do(self, r, out) };
        }
        if (!<u8 as Cast<u64>>::range(self).IsValid() || !<usize as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<u8>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as usize)
        {
            return false;
        }
        if (!<u8 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as usize) / (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<usize, V> for u8
where
    usize: Cast<V>,
{
    fn modulo(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<u8>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as usize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as usize) % (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i8, V> for i16
where
    i16: BuiltinTriple<i8, V>,
    i32: Cast<V>,
    i32: Cast<V>,
    i16: Cast<V>,
{
    fn add(self, r: i8, out: &mut V) -> bool {
        if CheckedAddFastOp::<i16, i8>::is_supported {
            return unsafe { CheckedAddFastOp::<i16, i8>::Do(self, r, out) };
        }
        if !<i16 as Cast<i32>>::range(self).IsValid() || !<i8 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedAddImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i8, out: &mut V) -> bool {
        if CheckedSubFastOp::<i16, i8>::is_supported {
            return unsafe { CheckedSubFastOp::<i16, i8>::Do(self, r, out) };
        }
        if !<i16 as Cast<i32>>::range(self).IsValid() || !<i8 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedSubImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i8, out: &mut V) -> bool {
        if CheckedMulFastOp::<i16, i8>::is_supported {
            return unsafe { CheckedMulFastOp::<i16, i8>::Do(self, r, out) };
        }
        if (!<i16 as Cast<i32>>::range(self).IsValid() || !<i8 as Cast<i32>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i32;
        let valid = if CheckedMulFastOp::<i32, i32>::is_supported {
            unsafe { CheckedMulFastOp::<i32, i32>::Do(self as i32, r as i32, &mut result) }
        } else {
            i32_ops::CheckedMulImpl(self as i32, r as i32, &mut result)
        };
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i16 as Cast<i16>>::CONTAINED && (self as i16) == i16::MIN && r == (-1i64 as i8)
        {
            return false;
        }
        if (!<i16 as Cast<i16>>::range(self).IsValid() || !<i8 as Cast<i16>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i16) / (r as i16);
        if !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i8, V> for i16
where
    i16: Cast<V>,
{
    fn modulo(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i16 as Cast<i16>>::CONTAINED && (self as i16) == i16::MIN && r == (-1i64 as i8)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i16) % (r as i16);
        if !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u8, V> for i16
where
    i16: BuiltinTriple<u8, V>,
    i32: Cast<V>,
    i32: Cast<V>,
    i16: Cast<V>,
{
    fn add(self, r: u8, out: &mut V) -> bool {
        if CheckedAddFastOp::<i16, u8>::is_supported {
            return unsafe { CheckedAddFastOp::<i16, u8>::Do(self, r, out) };
        }
        if !<i16 as Cast<i32>>::range(self).IsValid() || !<u8 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedAddImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u8, out: &mut V) -> bool {
        if CheckedSubFastOp::<i16, u8>::is_supported {
            return unsafe { CheckedSubFastOp::<i16, u8>::Do(self, r, out) };
        }
        if !<i16 as Cast<i32>>::range(self).IsValid() || !<u8 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedSubImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u8, out: &mut V) -> bool {
        if CheckedMulFastOp::<i16, u8>::is_supported {
            return unsafe { CheckedMulFastOp::<i16, u8>::Do(self, r, out) };
        }
        if (!<i16 as Cast<i32>>::range(self).IsValid() || !<u8 as Cast<i32>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i32;
        let valid = if CheckedMulFastOp::<i32, i32>::is_supported {
            unsafe { CheckedMulFastOp::<i32, i32>::Do(self as i32, r as i32, &mut result) }
        } else {
            i32_ops::CheckedMulImpl(self as i32, r as i32, &mut result)
        };
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i16 as Cast<i16>>::CONTAINED && (self as i16) == i16::MIN && r == (-1i64 as u8)
        {
            return false;
        }
        if (!<i16 as Cast<i16>>::range(self).IsValid() || !<u8 as Cast<i16>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i16) / (r as i16);
        if !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u8, V> for i16
where
    i16: Cast<V>,
{
    fn modulo(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i16 as Cast<i16>>::CONTAINED && (self as i16) == i16::MIN && r == (-1i64 as u8)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i16) % (r as i16);
        if !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i16, V> for i16
where
    i16: BuiltinTriple<i16, V>,
    i32: Cast<V>,
    i32: Cast<V>,
    i16: Cast<V>,
{
    fn add(self, r: i16, out: &mut V) -> bool {
        if CheckedAddFastOp::<i16, i16>::is_supported {
            return unsafe { CheckedAddFastOp::<i16, i16>::Do(self, r, out) };
        }
        if !<i16 as Cast<i32>>::range(self).IsValid() || !<i16 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedAddImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i16, out: &mut V) -> bool {
        if CheckedSubFastOp::<i16, i16>::is_supported {
            return unsafe { CheckedSubFastOp::<i16, i16>::Do(self, r, out) };
        }
        if !<i16 as Cast<i32>>::range(self).IsValid() || !<i16 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedSubImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i16, out: &mut V) -> bool {
        if CheckedMulFastOp::<i16, i16>::is_supported {
            return unsafe { CheckedMulFastOp::<i16, i16>::Do(self, r, out) };
        }
        if (!<i16 as Cast<i32>>::range(self).IsValid() || !<i16 as Cast<i32>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i32;
        let valid = if CheckedMulFastOp::<i32, i32>::is_supported {
            unsafe { CheckedMulFastOp::<i32, i32>::Do(self as i32, r as i32, &mut result) }
        } else {
            i32_ops::CheckedMulImpl(self as i32, r as i32, &mut result)
        };
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i16 as Cast<i16>>::CONTAINED && (self as i16) == i16::MIN && r == (-1i64 as i16)
        {
            return false;
        }
        if (!<i16 as Cast<i16>>::range(self).IsValid() || !<i16 as Cast<i16>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i16) / (r as i16);
        if !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i16, V> for i16
where
    i16: Cast<V>,
{
    fn modulo(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i16 as Cast<i16>>::CONTAINED && (self as i16) == i16::MIN && r == (-1i64 as i16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i16) % (r as i16);
        if !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u16, V> for i16
where
    i16: BuiltinTriple<u16, V>,
    i32: Cast<V>,
    i32: Cast<V>,
    i32: Cast<V>,
{
    fn add(self, r: u16, out: &mut V) -> bool {
        if CheckedAddFastOp::<i16, u16>::is_supported {
            return unsafe { CheckedAddFastOp::<i16, u16>::Do(self, r, out) };
        }
        if !<i16 as Cast<i32>>::range(self).IsValid() || !<u16 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedAddImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u16, out: &mut V) -> bool {
        if CheckedSubFastOp::<i16, u16>::is_supported {
            return unsafe { CheckedSubFastOp::<i16, u16>::Do(self, r, out) };
        }
        if !<i16 as Cast<i32>>::range(self).IsValid() || !<u16 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedSubImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u16, out: &mut V) -> bool {
        if CheckedMulFastOp::<i16, u16>::is_supported {
            return unsafe { CheckedMulFastOp::<i16, u16>::Do(self, r, out) };
        }
        if (!<i16 as Cast<i32>>::range(self).IsValid() || !<u16 as Cast<i32>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i32;
        let valid = if CheckedMulFastOp::<i32, i32>::is_supported {
            unsafe { CheckedMulFastOp::<i32, i32>::Do(self as i32, r as i32, &mut result) }
        } else {
            i32_ops::CheckedMulImpl(self as i32, r as i32, &mut result)
        };
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i32 as Cast<i16>>::CONTAINED
            && (self as i32) == i32::MIN
            && r == (-1i64 as u16)
        {
            return false;
        }
        if (!<i16 as Cast<i32>>::range(self).IsValid() || !<u16 as Cast<i32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i32) / (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u16, V> for i16
where
    i32: Cast<V>,
{
    fn modulo(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i32 as Cast<i16>>::CONTAINED
            && (self as i32) == i32::MIN
            && r == (-1i64 as u16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i32) % (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i32, V> for i16
where
    i16: BuiltinTriple<i32, V>,
    i32: Cast<V>,
    i64: Cast<V>,
    i32: Cast<V>,
{
    fn add(self, r: i32, out: &mut V) -> bool {
        if CheckedAddFastOp::<i16, i32>::is_supported {
            return unsafe { CheckedAddFastOp::<i16, i32>::Do(self, r, out) };
        }
        if !<i16 as Cast<i32>>::range(self).IsValid() || !<i32 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedAddImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i32, out: &mut V) -> bool {
        if CheckedSubFastOp::<i16, i32>::is_supported {
            return unsafe { CheckedSubFastOp::<i16, i32>::Do(self, r, out) };
        }
        if !<i16 as Cast<i32>>::range(self).IsValid() || !<i32 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedSubImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i32, out: &mut V) -> bool {
        if CheckedMulFastOp::<i16, i32>::is_supported {
            return unsafe { CheckedMulFastOp::<i16, i32>::Do(self, r, out) };
        }
        if (!<i16 as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i32 as Cast<i16>>::CONTAINED && (self as i32) == i32::MIN && r == (-1i64 as i32)
        {
            return false;
        }
        if (!<i16 as Cast<i32>>::range(self).IsValid() || !<i32 as Cast<i32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i32) / (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i32, V> for i16
where
    i32: Cast<V>,
{
    fn modulo(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i32 as Cast<i16>>::CONTAINED && (self as i32) == i32::MIN && r == (-1i64 as i32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i32) % (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u32, V> for i16
where
    i16: BuiltinTriple<u32, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: u32, out: &mut V) -> bool {
        if CheckedAddFastOp::<i16, u32>::is_supported {
            return unsafe { CheckedAddFastOp::<i16, u32>::Do(self, r, out) };
        }
        if !<i16 as Cast<i64>>::range(self).IsValid() || !<u32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u32, out: &mut V) -> bool {
        if CheckedSubFastOp::<i16, u32>::is_supported {
            return unsafe { CheckedSubFastOp::<i16, u32>::Do(self, r, out) };
        }
        if !<i16 as Cast<i64>>::range(self).IsValid() || !<u32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u32, out: &mut V) -> bool {
        if CheckedMulFastOp::<i16, u32>::is_supported {
            return unsafe { CheckedMulFastOp::<i16, u32>::Do(self, r, out) };
        }
        if (!<i16 as Cast<i64>>::range(self).IsValid() || !<u32 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<i16>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as u32)
        {
            return false;
        }
        if (!<i16 as Cast<i64>>::range(self).IsValid() || !<u32 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u32, V> for i16
where
    i64: Cast<V>,
{
    fn modulo(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<i16>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as u32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i64, V> for i16
where
    i16: BuiltinTriple<i64, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: i64, out: &mut V) -> bool {
        if CheckedAddFastOp::<i16, i64>::is_supported {
            return unsafe { CheckedAddFastOp::<i16, i64>::Do(self, r, out) };
        }
        if !<i16 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i64, out: &mut V) -> bool {
        if CheckedSubFastOp::<i16, i64>::is_supported {
            return unsafe { CheckedSubFastOp::<i16, i64>::Do(self, r, out) };
        }
        if !<i16 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i64, out: &mut V) -> bool {
        if CheckedMulFastOp::<i16, i64>::is_supported {
            return unsafe { CheckedMulFastOp::<i16, i64>::Do(self, r, out) };
        }
        if (!<i16 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i64 as Cast<i16>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as i64)
        {
            return false;
        }
        if (!<i16 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i64, V> for i16
where
    i64: Cast<V>,
{
    fn modulo(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i64 as Cast<i16>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as i64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u64, V> for i16
where
    i16: BuiltinTriple<u64, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u64: Cast<V>,
{
    fn add(self, r: u64, out: &mut V) -> bool {
        if CheckedAddFastOp::<i16, u64>::is_supported {
            return unsafe { CheckedAddFastOp::<i16, u64>::Do(self, r, out) };
        }
        if !<i16 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u64, out: &mut V) -> bool {
        if CheckedSubFastOp::<i16, u64>::is_supported {
            return unsafe { CheckedSubFastOp::<i16, u64>::Do(self, r, out) };
        }
        if !<i16 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u64, out: &mut V) -> bool {
        if CheckedMulFastOp::<i16, u64>::is_supported {
            return unsafe { CheckedMulFastOp::<i16, u64>::Do(self, r, out) };
        }
        if (!<i16 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<i16>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u64)
        {
            return false;
        }
        if (!<i16 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u64) / (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u64, V> for i16
where
    u64: Cast<V>,
{
    fn modulo(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<i16>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u64) % (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<isize, V> for i16
where
    i16: BuiltinTriple<isize, V>,
    isize: Cast<V>,
    i64: Cast<V>,
    isize: Cast<V>,
{
    fn add(self, r: isize, out: &mut V) -> bool {
        if CheckedAddFastOp::<i16, isize>::is_supported {
            return unsafe { CheckedAddFastOp::<i16, isize>::Do(self, r, out) };
        }
        if !<i16 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedAddImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: isize, out: &mut V) -> bool {
        if CheckedSubFastOp::<i16, isize>::is_supported {
            return unsafe { CheckedSubFastOp::<i16, isize>::Do(self, r, out) };
        }
        if !<i16 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedSubImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: isize, out: &mut V) -> bool {
        if CheckedMulFastOp::<i16, isize>::is_supported {
            return unsafe { CheckedMulFastOp::<i16, isize>::Do(self, r, out) };
        }
        if (!<i16 as Cast<i64>>::range(self).IsValid() || !<isize as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true
            && <isize as Cast<i16>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as isize)
        {
            return false;
        }
        if (!<i16 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as isize) / (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<isize, V> for i16
where
    isize: Cast<V>,
{
    fn modulo(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true
            && <isize as Cast<i16>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as isize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as isize) % (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<usize, V> for i16
where
    i16: BuiltinTriple<usize, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: usize, out: &mut V) -> bool {
        if CheckedAddFastOp::<i16, usize>::is_supported {
            return unsafe { CheckedAddFastOp::<i16, usize>::Do(self, r, out) };
        }
        if !<i16 as Cast<i64>>::range(self).IsValid() || !<usize as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: usize, out: &mut V) -> bool {
        if CheckedSubFastOp::<i16, usize>::is_supported {
            return unsafe { CheckedSubFastOp::<i16, usize>::Do(self, r, out) };
        }
        if !<i16 as Cast<i64>>::range(self).IsValid() || !<usize as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: usize, out: &mut V) -> bool {
        if CheckedMulFastOp::<i16, usize>::is_supported {
            return unsafe { CheckedMulFastOp::<i16, usize>::Do(self, r, out) };
        }
        if (!<i16 as Cast<i64>>::range(self).IsValid() || !<usize as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<i16>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as usize)
        {
            return false;
        }
        if (!<i16 as Cast<i64>>::range(self).IsValid() || !<usize as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<usize, V> for i16
where
    i64: Cast<V>,
{
    fn modulo(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<i16>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as usize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i8, V> for u16
where
    u16: BuiltinTriple<i8, V>,
    i32: Cast<V>,
    i32: Cast<V>,
    i32: Cast<V>,
{
    fn add(self, r: i8, out: &mut V) -> bool {
        if CheckedAddFastOp::<u16, i8>::is_supported {
            return unsafe { CheckedAddFastOp::<u16, i8>::Do(self, r, out) };
        }
        if !<u16 as Cast<i32>>::range(self).IsValid() || !<i8 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedAddImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i8, out: &mut V) -> bool {
        if CheckedSubFastOp::<u16, i8>::is_supported {
            return unsafe { CheckedSubFastOp::<u16, i8>::Do(self, r, out) };
        }
        if !<u16 as Cast<i32>>::range(self).IsValid() || !<i8 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedSubImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i8, out: &mut V) -> bool {
        if CheckedMulFastOp::<u16, i8>::is_supported {
            return unsafe { CheckedMulFastOp::<u16, i8>::Do(self, r, out) };
        }
        if (!<u16 as Cast<i32>>::range(self).IsValid() || !<i8 as Cast<i32>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i32;
        let valid = if CheckedMulFastOp::<i32, i32>::is_supported {
            unsafe { CheckedMulFastOp::<i32, i32>::Do(self as i32, r as i32, &mut result) }
        } else {
            i32_ops::CheckedMulImpl(self as i32, r as i32, &mut result)
        };
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i32 as Cast<u16>>::CONTAINED && (self as i32) == i32::MIN && r == (-1i64 as i8)
        {
            return false;
        }
        if (!<u16 as Cast<i32>>::range(self).IsValid() || !<i8 as Cast<i32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i32) / (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i8, V> for u16
where
    i32: Cast<V>,
{
    fn modulo(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i32 as Cast<u16>>::CONTAINED && (self as i32) == i32::MIN && r == (-1i64 as i8)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i32) % (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u8, V> for u16
where
    u16: BuiltinTriple<u8, V>,
    u32: Cast<V>,
    u32: Cast<V>,
    u16: Cast<V>,
{
    fn add(self, r: u8, out: &mut V) -> bool {
        if CheckedAddFastOp::<u16, u8>::is_supported {
            return unsafe { CheckedAddFastOp::<u16, u8>::Do(self, r, out) };
        }
        if !<u16 as Cast<u32>>::range(self).IsValid() || !<u8 as Cast<u32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u32;
        let valid = u32_ops::CheckedAddImpl(self as u32, r as u32, &mut result);
        if !valid || !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u8, out: &mut V) -> bool {
        if CheckedSubFastOp::<u16, u8>::is_supported {
            return unsafe { CheckedSubFastOp::<u16, u8>::Do(self, r, out) };
        }
        if !<u16 as Cast<u32>>::range(self).IsValid() || !<u8 as Cast<u32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u32;
        let valid = u32_ops::CheckedSubImpl(self as u32, r as u32, &mut result);
        if !valid || !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u8, out: &mut V) -> bool {
        if CheckedMulFastOp::<u16, u8>::is_supported {
            return unsafe { CheckedMulFastOp::<u16, u8>::Do(self, r, out) };
        }
        if (!<u16 as Cast<u32>>::range(self).IsValid() || !<u8 as Cast<u32>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u32;
        let valid = if CheckedMulFastOp::<u32, u32>::is_supported {
            unsafe { CheckedMulFastOp::<u32, u32>::Do(self as u32, r as u32, &mut result) }
        } else {
            u32_ops::CheckedMulImpl(self as u32, r as u32, &mut result)
        };
        if !valid || !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <u16 as Cast<u16>>::CONTAINED && (self as u16) == u16::MIN && r == (-1i64 as u8)
        {
            return false;
        }
        if (!<u16 as Cast<u16>>::range(self).IsValid() || !<u8 as Cast<u16>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u16) / (r as u16);
        if !<u16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u8, V> for u16
where
    u16: Cast<V>,
{
    fn modulo(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <u16 as Cast<u16>>::CONTAINED && (self as u16) == u16::MIN && r == (-1i64 as u8)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u16) % (r as u16);
        if !<u16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i16, V> for u16
where
    u16: BuiltinTriple<i16, V>,
    i32: Cast<V>,
    i32: Cast<V>,
    i32: Cast<V>,
{
    fn add(self, r: i16, out: &mut V) -> bool {
        if CheckedAddFastOp::<u16, i16>::is_supported {
            return unsafe { CheckedAddFastOp::<u16, i16>::Do(self, r, out) };
        }
        if !<u16 as Cast<i32>>::range(self).IsValid() || !<i16 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedAddImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i16, out: &mut V) -> bool {
        if CheckedSubFastOp::<u16, i16>::is_supported {
            return unsafe { CheckedSubFastOp::<u16, i16>::Do(self, r, out) };
        }
        if !<u16 as Cast<i32>>::range(self).IsValid() || !<i16 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedSubImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i16, out: &mut V) -> bool {
        if CheckedMulFastOp::<u16, i16>::is_supported {
            return unsafe { CheckedMulFastOp::<u16, i16>::Do(self, r, out) };
        }
        if (!<u16 as Cast<i32>>::range(self).IsValid() || !<i16 as Cast<i32>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i32;
        let valid = if CheckedMulFastOp::<i32, i32>::is_supported {
            unsafe { CheckedMulFastOp::<i32, i32>::Do(self as i32, r as i32, &mut result) }
        } else {
            i32_ops::CheckedMulImpl(self as i32, r as i32, &mut result)
        };
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i32 as Cast<u16>>::CONTAINED
            && (self as i32) == i32::MIN
            && r == (-1i64 as i16)
        {
            return false;
        }
        if (!<u16 as Cast<i32>>::range(self).IsValid() || !<i16 as Cast<i32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i32) / (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i16, V> for u16
where
    i32: Cast<V>,
{
    fn modulo(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i32 as Cast<u16>>::CONTAINED
            && (self as i32) == i32::MIN
            && r == (-1i64 as i16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i32) % (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u16, V> for u16
where
    u16: BuiltinTriple<u16, V>,
    u32: Cast<V>,
    u32: Cast<V>,
    u16: Cast<V>,
{
    fn add(self, r: u16, out: &mut V) -> bool {
        if CheckedAddFastOp::<u16, u16>::is_supported {
            return unsafe { CheckedAddFastOp::<u16, u16>::Do(self, r, out) };
        }
        if !<u16 as Cast<u32>>::range(self).IsValid() || !<u16 as Cast<u32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u32;
        let valid = u32_ops::CheckedAddImpl(self as u32, r as u32, &mut result);
        if !valid || !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u16, out: &mut V) -> bool {
        if CheckedSubFastOp::<u16, u16>::is_supported {
            return unsafe { CheckedSubFastOp::<u16, u16>::Do(self, r, out) };
        }
        if !<u16 as Cast<u32>>::range(self).IsValid() || !<u16 as Cast<u32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u32;
        let valid = u32_ops::CheckedSubImpl(self as u32, r as u32, &mut result);
        if !valid || !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u16, out: &mut V) -> bool {
        if CheckedMulFastOp::<u16, u16>::is_supported {
            return unsafe { CheckedMulFastOp::<u16, u16>::Do(self, r, out) };
        }
        if (!<u16 as Cast<u32>>::range(self).IsValid() || !<u16 as Cast<u32>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u32;
        let valid = if CheckedMulFastOp::<u32, u32>::is_supported {
            unsafe { CheckedMulFastOp::<u32, u32>::Do(self as u32, r as u32, &mut result) }
        } else {
            u32_ops::CheckedMulImpl(self as u32, r as u32, &mut result)
        };
        if !valid || !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u16 as Cast<u16>>::CONTAINED
            && (self as u16) == u16::MIN
            && r == (-1i64 as u16)
        {
            return false;
        }
        if (!<u16 as Cast<u16>>::range(self).IsValid() || !<u16 as Cast<u16>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u16) / (r as u16);
        if !<u16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u16, V> for u16
where
    u16: Cast<V>,
{
    fn modulo(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u16 as Cast<u16>>::CONTAINED
            && (self as u16) == u16::MIN
            && r == (-1i64 as u16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u16) % (r as u16);
        if !<u16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i32, V> for u16
where
    u16: BuiltinTriple<i32, V>,
    i32: Cast<V>,
    i64: Cast<V>,
    i32: Cast<V>,
{
    fn add(self, r: i32, out: &mut V) -> bool {
        if CheckedAddFastOp::<u16, i32>::is_supported {
            return unsafe { CheckedAddFastOp::<u16, i32>::Do(self, r, out) };
        }
        if !<u16 as Cast<i32>>::range(self).IsValid() || !<i32 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedAddImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i32, out: &mut V) -> bool {
        if CheckedSubFastOp::<u16, i32>::is_supported {
            return unsafe { CheckedSubFastOp::<u16, i32>::Do(self, r, out) };
        }
        if !<u16 as Cast<i32>>::range(self).IsValid() || !<i32 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedSubImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i32, out: &mut V) -> bool {
        if CheckedMulFastOp::<u16, i32>::is_supported {
            return unsafe { CheckedMulFastOp::<u16, i32>::Do(self, r, out) };
        }
        if (!<u16 as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i32 as Cast<u16>>::CONTAINED
            && (self as i32) == i32::MIN
            && r == (-1i64 as i32)
        {
            return false;
        }
        if (!<u16 as Cast<i32>>::range(self).IsValid() || !<i32 as Cast<i32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i32) / (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i32, V> for u16
where
    i32: Cast<V>,
{
    fn modulo(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i32 as Cast<u16>>::CONTAINED
            && (self as i32) == i32::MIN
            && r == (-1i64 as i32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i32) % (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u32, V> for u16
where
    u16: BuiltinTriple<u32, V>,
    u32: Cast<V>,
    u64: Cast<V>,
    u32: Cast<V>,
{
    fn add(self, r: u32, out: &mut V) -> bool {
        if CheckedAddFastOp::<u16, u32>::is_supported {
            return unsafe { CheckedAddFastOp::<u16, u32>::Do(self, r, out) };
        }
        if !<u16 as Cast<u32>>::range(self).IsValid() || !<u32 as Cast<u32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u32;
        let valid = u32_ops::CheckedAddImpl(self as u32, r as u32, &mut result);
        if !valid || !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u32, out: &mut V) -> bool {
        if CheckedSubFastOp::<u16, u32>::is_supported {
            return unsafe { CheckedSubFastOp::<u16, u32>::Do(self, r, out) };
        }
        if !<u16 as Cast<u32>>::range(self).IsValid() || !<u32 as Cast<u32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u32;
        let valid = u32_ops::CheckedSubImpl(self as u32, r as u32, &mut result);
        if !valid || !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u32, out: &mut V) -> bool {
        if CheckedMulFastOp::<u16, u32>::is_supported {
            return unsafe { CheckedMulFastOp::<u16, u32>::Do(self, r, out) };
        }
        if (!<u16 as Cast<u64>>::range(self).IsValid() || !<u32 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u32 as Cast<u16>>::CONTAINED
            && (self as u32) == u32::MIN
            && r == (-1i64 as u32)
        {
            return false;
        }
        if (!<u16 as Cast<u32>>::range(self).IsValid() || !<u32 as Cast<u32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u32) / (r as u32);
        if !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u32, V> for u16
where
    u32: Cast<V>,
{
    fn modulo(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u32 as Cast<u16>>::CONTAINED
            && (self as u32) == u32::MIN
            && r == (-1i64 as u32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u32) % (r as u32);
        if !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i64, V> for u16
where
    u16: BuiltinTriple<i64, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: i64, out: &mut V) -> bool {
        if CheckedAddFastOp::<u16, i64>::is_supported {
            return unsafe { CheckedAddFastOp::<u16, i64>::Do(self, r, out) };
        }
        if !<u16 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i64, out: &mut V) -> bool {
        if CheckedSubFastOp::<u16, i64>::is_supported {
            return unsafe { CheckedSubFastOp::<u16, i64>::Do(self, r, out) };
        }
        if !<u16 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i64, out: &mut V) -> bool {
        if CheckedMulFastOp::<u16, i64>::is_supported {
            return unsafe { CheckedMulFastOp::<u16, i64>::Do(self, r, out) };
        }
        if (!<u16 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<u16>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as i64)
        {
            return false;
        }
        if (!<u16 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i64, V> for u16
where
    i64: Cast<V>,
{
    fn modulo(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<u16>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as i64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u64, V> for u16
where
    u16: BuiltinTriple<u64, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u64: Cast<V>,
{
    fn add(self, r: u64, out: &mut V) -> bool {
        if CheckedAddFastOp::<u16, u64>::is_supported {
            return unsafe { CheckedAddFastOp::<u16, u64>::Do(self, r, out) };
        }
        if !<u16 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u64, out: &mut V) -> bool {
        if CheckedSubFastOp::<u16, u64>::is_supported {
            return unsafe { CheckedSubFastOp::<u16, u64>::Do(self, r, out) };
        }
        if !<u16 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u64, out: &mut V) -> bool {
        if CheckedMulFastOp::<u16, u64>::is_supported {
            return unsafe { CheckedMulFastOp::<u16, u64>::Do(self, r, out) };
        }
        if (!<u16 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u16>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u64)
        {
            return false;
        }
        if (!<u16 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u64) / (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u64, V> for u16
where
    u64: Cast<V>,
{
    fn modulo(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u16>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u64) % (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<isize, V> for u16
where
    u16: BuiltinTriple<isize, V>,
    isize: Cast<V>,
    i64: Cast<V>,
    isize: Cast<V>,
{
    fn add(self, r: isize, out: &mut V) -> bool {
        if CheckedAddFastOp::<u16, isize>::is_supported {
            return unsafe { CheckedAddFastOp::<u16, isize>::Do(self, r, out) };
        }
        if !<u16 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedAddImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: isize, out: &mut V) -> bool {
        if CheckedSubFastOp::<u16, isize>::is_supported {
            return unsafe { CheckedSubFastOp::<u16, isize>::Do(self, r, out) };
        }
        if !<u16 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedSubImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: isize, out: &mut V) -> bool {
        if CheckedMulFastOp::<u16, isize>::is_supported {
            return unsafe { CheckedMulFastOp::<u16, isize>::Do(self, r, out) };
        }
        if (!<u16 as Cast<i64>>::range(self).IsValid() || !<isize as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <isize as Cast<u16>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as isize)
        {
            return false;
        }
        if (!<u16 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as isize) / (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<isize, V> for u16
where
    isize: Cast<V>,
{
    fn modulo(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <isize as Cast<u16>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as isize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as isize) % (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<usize, V> for u16
where
    u16: BuiltinTriple<usize, V>,
    usize: Cast<V>,
    u64: Cast<V>,
    usize: Cast<V>,
{
    fn add(self, r: usize, out: &mut V) -> bool {
        if CheckedAddFastOp::<u16, usize>::is_supported {
            return unsafe { CheckedAddFastOp::<u16, usize>::Do(self, r, out) };
        }
        if !<u16 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedAddImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: usize, out: &mut V) -> bool {
        if CheckedSubFastOp::<u16, usize>::is_supported {
            return unsafe { CheckedSubFastOp::<u16, usize>::Do(self, r, out) };
        }
        if !<u16 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedSubImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: usize, out: &mut V) -> bool {
        if CheckedMulFastOp::<u16, usize>::is_supported {
            return unsafe { CheckedMulFastOp::<u16, usize>::Do(self, r, out) };
        }
        if (!<u16 as Cast<u64>>::range(self).IsValid() || !<usize as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<u16>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as usize)
        {
            return false;
        }
        if (!<u16 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as usize) / (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<usize, V> for u16
where
    usize: Cast<V>,
{
    fn modulo(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<u16>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as usize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as usize) % (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i8, V> for i32
where
    i32: BuiltinTriple<i8, V>,
    i32: Cast<V>,
    i64: Cast<V>,
    i32: Cast<V>,
{
    fn add(self, r: i8, out: &mut V) -> bool {
        if CheckedAddFastOp::<i32, i8>::is_supported {
            return unsafe { CheckedAddFastOp::<i32, i8>::Do(self, r, out) };
        }
        if !<i32 as Cast<i32>>::range(self).IsValid() || !<i8 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedAddImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i8, out: &mut V) -> bool {
        if CheckedSubFastOp::<i32, i8>::is_supported {
            return unsafe { CheckedSubFastOp::<i32, i8>::Do(self, r, out) };
        }
        if !<i32 as Cast<i32>>::range(self).IsValid() || !<i8 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedSubImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i8, out: &mut V) -> bool {
        if CheckedMulFastOp::<i32, i8>::is_supported {
            return unsafe { CheckedMulFastOp::<i32, i8>::Do(self, r, out) };
        }
        if (!<i32 as Cast<i64>>::range(self).IsValid() || !<i8 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i32 as Cast<i32>>::CONTAINED && (self as i32) == i32::MIN && r == (-1i64 as i8)
        {
            return false;
        }
        if (!<i32 as Cast<i32>>::range(self).IsValid() || !<i8 as Cast<i32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i32) / (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i8, V> for i32
where
    i32: Cast<V>,
{
    fn modulo(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i32 as Cast<i32>>::CONTAINED && (self as i32) == i32::MIN && r == (-1i64 as i8)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i32) % (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u8, V> for i32
where
    i32: BuiltinTriple<u8, V>,
    i32: Cast<V>,
    i64: Cast<V>,
    i32: Cast<V>,
{
    fn add(self, r: u8, out: &mut V) -> bool {
        if CheckedAddFastOp::<i32, u8>::is_supported {
            return unsafe { CheckedAddFastOp::<i32, u8>::Do(self, r, out) };
        }
        if !<i32 as Cast<i32>>::range(self).IsValid() || !<u8 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedAddImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u8, out: &mut V) -> bool {
        if CheckedSubFastOp::<i32, u8>::is_supported {
            return unsafe { CheckedSubFastOp::<i32, u8>::Do(self, r, out) };
        }
        if !<i32 as Cast<i32>>::range(self).IsValid() || !<u8 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedSubImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u8, out: &mut V) -> bool {
        if CheckedMulFastOp::<i32, u8>::is_supported {
            return unsafe { CheckedMulFastOp::<i32, u8>::Do(self, r, out) };
        }
        if (!<i32 as Cast<i64>>::range(self).IsValid() || !<u8 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i32 as Cast<i32>>::CONTAINED && (self as i32) == i32::MIN && r == (-1i64 as u8)
        {
            return false;
        }
        if (!<i32 as Cast<i32>>::range(self).IsValid() || !<u8 as Cast<i32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i32) / (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u8, V> for i32
where
    i32: Cast<V>,
{
    fn modulo(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i32 as Cast<i32>>::CONTAINED && (self as i32) == i32::MIN && r == (-1i64 as u8)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i32) % (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i16, V> for i32
where
    i32: BuiltinTriple<i16, V>,
    i32: Cast<V>,
    i64: Cast<V>,
    i32: Cast<V>,
{
    fn add(self, r: i16, out: &mut V) -> bool {
        if CheckedAddFastOp::<i32, i16>::is_supported {
            return unsafe { CheckedAddFastOp::<i32, i16>::Do(self, r, out) };
        }
        if !<i32 as Cast<i32>>::range(self).IsValid() || !<i16 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedAddImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i16, out: &mut V) -> bool {
        if CheckedSubFastOp::<i32, i16>::is_supported {
            return unsafe { CheckedSubFastOp::<i32, i16>::Do(self, r, out) };
        }
        if !<i32 as Cast<i32>>::range(self).IsValid() || !<i16 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedSubImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i16, out: &mut V) -> bool {
        if CheckedMulFastOp::<i32, i16>::is_supported {
            return unsafe { CheckedMulFastOp::<i32, i16>::Do(self, r, out) };
        }
        if (!<i32 as Cast<i64>>::range(self).IsValid() || !<i16 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i32 as Cast<i32>>::CONTAINED && (self as i32) == i32::MIN && r == (-1i64 as i16)
        {
            return false;
        }
        if (!<i32 as Cast<i32>>::range(self).IsValid() || !<i16 as Cast<i32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i32) / (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i16, V> for i32
where
    i32: Cast<V>,
{
    fn modulo(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i32 as Cast<i32>>::CONTAINED && (self as i32) == i32::MIN && r == (-1i64 as i16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i32) % (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u16, V> for i32
where
    i32: BuiltinTriple<u16, V>,
    i32: Cast<V>,
    i64: Cast<V>,
    i32: Cast<V>,
{
    fn add(self, r: u16, out: &mut V) -> bool {
        if CheckedAddFastOp::<i32, u16>::is_supported {
            return unsafe { CheckedAddFastOp::<i32, u16>::Do(self, r, out) };
        }
        if !<i32 as Cast<i32>>::range(self).IsValid() || !<u16 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedAddImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u16, out: &mut V) -> bool {
        if CheckedSubFastOp::<i32, u16>::is_supported {
            return unsafe { CheckedSubFastOp::<i32, u16>::Do(self, r, out) };
        }
        if !<i32 as Cast<i32>>::range(self).IsValid() || !<u16 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedSubImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u16, out: &mut V) -> bool {
        if CheckedMulFastOp::<i32, u16>::is_supported {
            return unsafe { CheckedMulFastOp::<i32, u16>::Do(self, r, out) };
        }
        if (!<i32 as Cast<i64>>::range(self).IsValid() || !<u16 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i32 as Cast<i32>>::CONTAINED
            && (self as i32) == i32::MIN
            && r == (-1i64 as u16)
        {
            return false;
        }
        if (!<i32 as Cast<i32>>::range(self).IsValid() || !<u16 as Cast<i32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i32) / (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u16, V> for i32
where
    i32: Cast<V>,
{
    fn modulo(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i32 as Cast<i32>>::CONTAINED
            && (self as i32) == i32::MIN
            && r == (-1i64 as u16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i32) % (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i32, V> for i32
where
    i32: BuiltinTriple<i32, V>,
    i32: Cast<V>,
    i64: Cast<V>,
    i32: Cast<V>,
{
    fn add(self, r: i32, out: &mut V) -> bool {
        if CheckedAddFastOp::<i32, i32>::is_supported {
            return unsafe { CheckedAddFastOp::<i32, i32>::Do(self, r, out) };
        }
        if !<i32 as Cast<i32>>::range(self).IsValid() || !<i32 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedAddImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i32, out: &mut V) -> bool {
        if CheckedSubFastOp::<i32, i32>::is_supported {
            return unsafe { CheckedSubFastOp::<i32, i32>::Do(self, r, out) };
        }
        if !<i32 as Cast<i32>>::range(self).IsValid() || !<i32 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedSubImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i32, out: &mut V) -> bool {
        if CheckedMulFastOp::<i32, i32>::is_supported {
            return unsafe { CheckedMulFastOp::<i32, i32>::Do(self, r, out) };
        }
        if (!<i32 as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i32 as Cast<i32>>::CONTAINED && (self as i32) == i32::MIN && r == (-1i64 as i32)
        {
            return false;
        }
        if (!<i32 as Cast<i32>>::range(self).IsValid() || !<i32 as Cast<i32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i32) / (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i32, V> for i32
where
    i32: Cast<V>,
{
    fn modulo(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i32 as Cast<i32>>::CONTAINED && (self as i32) == i32::MIN && r == (-1i64 as i32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i32) % (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u32, V> for i32
where
    i32: BuiltinTriple<u32, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: u32, out: &mut V) -> bool {
        if CheckedAddFastOp::<i32, u32>::is_supported {
            return unsafe { CheckedAddFastOp::<i32, u32>::Do(self, r, out) };
        }
        if !<i32 as Cast<i64>>::range(self).IsValid() || !<u32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u32, out: &mut V) -> bool {
        if CheckedSubFastOp::<i32, u32>::is_supported {
            return unsafe { CheckedSubFastOp::<i32, u32>::Do(self, r, out) };
        }
        if !<i32 as Cast<i64>>::range(self).IsValid() || !<u32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u32, out: &mut V) -> bool {
        if CheckedMulFastOp::<i32, u32>::is_supported {
            return unsafe { CheckedMulFastOp::<i32, u32>::Do(self, r, out) };
        }
        if (!<i32 as Cast<i64>>::range(self).IsValid() || !<u32 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<i32>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as u32)
        {
            return false;
        }
        if (!<i32 as Cast<i64>>::range(self).IsValid() || !<u32 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u32, V> for i32
where
    i64: Cast<V>,
{
    fn modulo(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<i32>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as u32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i64, V> for i32
where
    i32: BuiltinTriple<i64, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: i64, out: &mut V) -> bool {
        if CheckedAddFastOp::<i32, i64>::is_supported {
            return unsafe { CheckedAddFastOp::<i32, i64>::Do(self, r, out) };
        }
        if !<i32 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i64, out: &mut V) -> bool {
        if CheckedSubFastOp::<i32, i64>::is_supported {
            return unsafe { CheckedSubFastOp::<i32, i64>::Do(self, r, out) };
        }
        if !<i32 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i64, out: &mut V) -> bool {
        if CheckedMulFastOp::<i32, i64>::is_supported {
            return unsafe { CheckedMulFastOp::<i32, i64>::Do(self, r, out) };
        }
        if (!<i32 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i64 as Cast<i32>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as i64)
        {
            return false;
        }
        if (!<i32 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i64, V> for i32
where
    i64: Cast<V>,
{
    fn modulo(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i64 as Cast<i32>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as i64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u64, V> for i32
where
    i32: BuiltinTriple<u64, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u64: Cast<V>,
{
    fn add(self, r: u64, out: &mut V) -> bool {
        if CheckedAddFastOp::<i32, u64>::is_supported {
            return unsafe { CheckedAddFastOp::<i32, u64>::Do(self, r, out) };
        }
        if !<i32 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u64, out: &mut V) -> bool {
        if CheckedSubFastOp::<i32, u64>::is_supported {
            return unsafe { CheckedSubFastOp::<i32, u64>::Do(self, r, out) };
        }
        if !<i32 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u64, out: &mut V) -> bool {
        if CheckedMulFastOp::<i32, u64>::is_supported {
            return unsafe { CheckedMulFastOp::<i32, u64>::Do(self, r, out) };
        }
        if (!<i32 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<i32>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u64)
        {
            return false;
        }
        if (!<i32 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u64) / (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u64, V> for i32
where
    u64: Cast<V>,
{
    fn modulo(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<i32>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u64) % (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<isize, V> for i32
where
    i32: BuiltinTriple<isize, V>,
    isize: Cast<V>,
    i64: Cast<V>,
    isize: Cast<V>,
{
    fn add(self, r: isize, out: &mut V) -> bool {
        if CheckedAddFastOp::<i32, isize>::is_supported {
            return unsafe { CheckedAddFastOp::<i32, isize>::Do(self, r, out) };
        }
        if !<i32 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedAddImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: isize, out: &mut V) -> bool {
        if CheckedSubFastOp::<i32, isize>::is_supported {
            return unsafe { CheckedSubFastOp::<i32, isize>::Do(self, r, out) };
        }
        if !<i32 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedSubImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: isize, out: &mut V) -> bool {
        if CheckedMulFastOp::<i32, isize>::is_supported {
            return unsafe { CheckedMulFastOp::<i32, isize>::Do(self, r, out) };
        }
        if (!<i32 as Cast<i64>>::range(self).IsValid() || !<isize as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true
            && <isize as Cast<i32>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as isize)
        {
            return false;
        }
        if (!<i32 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as isize) / (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<isize, V> for i32
where
    isize: Cast<V>,
{
    fn modulo(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true
            && <isize as Cast<i32>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as isize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as isize) % (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<usize, V> for i32
where
    i32: BuiltinTriple<usize, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: usize, out: &mut V) -> bool {
        if CheckedAddFastOp::<i32, usize>::is_supported {
            return unsafe { CheckedAddFastOp::<i32, usize>::Do(self, r, out) };
        }
        if !<i32 as Cast<i64>>::range(self).IsValid() || !<usize as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: usize, out: &mut V) -> bool {
        if CheckedSubFastOp::<i32, usize>::is_supported {
            return unsafe { CheckedSubFastOp::<i32, usize>::Do(self, r, out) };
        }
        if !<i32 as Cast<i64>>::range(self).IsValid() || !<usize as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: usize, out: &mut V) -> bool {
        if CheckedMulFastOp::<i32, usize>::is_supported {
            return unsafe { CheckedMulFastOp::<i32, usize>::Do(self, r, out) };
        }
        if (!<i32 as Cast<i64>>::range(self).IsValid() || !<usize as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<i32>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as usize)
        {
            return false;
        }
        if (!<i32 as Cast<i64>>::range(self).IsValid() || !<usize as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<usize, V> for i32
where
    i64: Cast<V>,
{
    fn modulo(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<i32>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as usize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i8, V> for u32
where
    u32: BuiltinTriple<i8, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: i8, out: &mut V) -> bool {
        if CheckedAddFastOp::<u32, i8>::is_supported {
            return unsafe { CheckedAddFastOp::<u32, i8>::Do(self, r, out) };
        }
        if !<u32 as Cast<i64>>::range(self).IsValid() || !<i8 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i8, out: &mut V) -> bool {
        if CheckedSubFastOp::<u32, i8>::is_supported {
            return unsafe { CheckedSubFastOp::<u32, i8>::Do(self, r, out) };
        }
        if !<u32 as Cast<i64>>::range(self).IsValid() || !<i8 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i8, out: &mut V) -> bool {
        if CheckedMulFastOp::<u32, i8>::is_supported {
            return unsafe { CheckedMulFastOp::<u32, i8>::Do(self, r, out) };
        }
        if (!<u32 as Cast<i64>>::range(self).IsValid() || !<i8 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i64 as Cast<u32>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as i8)
        {
            return false;
        }
        if (!<u32 as Cast<i64>>::range(self).IsValid() || !<i8 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i8, V> for u32
where
    i64: Cast<V>,
{
    fn modulo(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i64 as Cast<u32>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as i8)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u8, V> for u32
where
    u32: BuiltinTriple<u8, V>,
    u32: Cast<V>,
    u64: Cast<V>,
    u32: Cast<V>,
{
    fn add(self, r: u8, out: &mut V) -> bool {
        if CheckedAddFastOp::<u32, u8>::is_supported {
            return unsafe { CheckedAddFastOp::<u32, u8>::Do(self, r, out) };
        }
        if !<u32 as Cast<u32>>::range(self).IsValid() || !<u8 as Cast<u32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u32;
        let valid = u32_ops::CheckedAddImpl(self as u32, r as u32, &mut result);
        if !valid || !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u8, out: &mut V) -> bool {
        if CheckedSubFastOp::<u32, u8>::is_supported {
            return unsafe { CheckedSubFastOp::<u32, u8>::Do(self, r, out) };
        }
        if !<u32 as Cast<u32>>::range(self).IsValid() || !<u8 as Cast<u32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u32;
        let valid = u32_ops::CheckedSubImpl(self as u32, r as u32, &mut result);
        if !valid || !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u8, out: &mut V) -> bool {
        if CheckedMulFastOp::<u32, u8>::is_supported {
            return unsafe { CheckedMulFastOp::<u32, u8>::Do(self, r, out) };
        }
        if (!<u32 as Cast<u64>>::range(self).IsValid() || !<u8 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <u32 as Cast<u32>>::CONTAINED && (self as u32) == u32::MIN && r == (-1i64 as u8)
        {
            return false;
        }
        if (!<u32 as Cast<u32>>::range(self).IsValid() || !<u8 as Cast<u32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u32) / (r as u32);
        if !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u8, V> for u32
where
    u32: Cast<V>,
{
    fn modulo(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <u32 as Cast<u32>>::CONTAINED && (self as u32) == u32::MIN && r == (-1i64 as u8)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u32) % (r as u32);
        if !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i16, V> for u32
where
    u32: BuiltinTriple<i16, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: i16, out: &mut V) -> bool {
        if CheckedAddFastOp::<u32, i16>::is_supported {
            return unsafe { CheckedAddFastOp::<u32, i16>::Do(self, r, out) };
        }
        if !<u32 as Cast<i64>>::range(self).IsValid() || !<i16 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i16, out: &mut V) -> bool {
        if CheckedSubFastOp::<u32, i16>::is_supported {
            return unsafe { CheckedSubFastOp::<u32, i16>::Do(self, r, out) };
        }
        if !<u32 as Cast<i64>>::range(self).IsValid() || !<i16 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i16, out: &mut V) -> bool {
        if CheckedMulFastOp::<u32, i16>::is_supported {
            return unsafe { CheckedMulFastOp::<u32, i16>::Do(self, r, out) };
        }
        if (!<u32 as Cast<i64>>::range(self).IsValid() || !<i16 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<u32>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as i16)
        {
            return false;
        }
        if (!<u32 as Cast<i64>>::range(self).IsValid() || !<i16 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i16, V> for u32
where
    i64: Cast<V>,
{
    fn modulo(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<u32>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as i16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u16, V> for u32
where
    u32: BuiltinTriple<u16, V>,
    u32: Cast<V>,
    u64: Cast<V>,
    u32: Cast<V>,
{
    fn add(self, r: u16, out: &mut V) -> bool {
        if CheckedAddFastOp::<u32, u16>::is_supported {
            return unsafe { CheckedAddFastOp::<u32, u16>::Do(self, r, out) };
        }
        if !<u32 as Cast<u32>>::range(self).IsValid() || !<u16 as Cast<u32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u32;
        let valid = u32_ops::CheckedAddImpl(self as u32, r as u32, &mut result);
        if !valid || !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u16, out: &mut V) -> bool {
        if CheckedSubFastOp::<u32, u16>::is_supported {
            return unsafe { CheckedSubFastOp::<u32, u16>::Do(self, r, out) };
        }
        if !<u32 as Cast<u32>>::range(self).IsValid() || !<u16 as Cast<u32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u32;
        let valid = u32_ops::CheckedSubImpl(self as u32, r as u32, &mut result);
        if !valid || !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u16, out: &mut V) -> bool {
        if CheckedMulFastOp::<u32, u16>::is_supported {
            return unsafe { CheckedMulFastOp::<u32, u16>::Do(self, r, out) };
        }
        if (!<u32 as Cast<u64>>::range(self).IsValid() || !<u16 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u32 as Cast<u32>>::CONTAINED
            && (self as u32) == u32::MIN
            && r == (-1i64 as u16)
        {
            return false;
        }
        if (!<u32 as Cast<u32>>::range(self).IsValid() || !<u16 as Cast<u32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u32) / (r as u32);
        if !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u16, V> for u32
where
    u32: Cast<V>,
{
    fn modulo(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u32 as Cast<u32>>::CONTAINED
            && (self as u32) == u32::MIN
            && r == (-1i64 as u16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u32) % (r as u32);
        if !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i32, V> for u32
where
    u32: BuiltinTriple<i32, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: i32, out: &mut V) -> bool {
        if CheckedAddFastOp::<u32, i32>::is_supported {
            return unsafe { CheckedAddFastOp::<u32, i32>::Do(self, r, out) };
        }
        if !<u32 as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i32, out: &mut V) -> bool {
        if CheckedSubFastOp::<u32, i32>::is_supported {
            return unsafe { CheckedSubFastOp::<u32, i32>::Do(self, r, out) };
        }
        if !<u32 as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i32, out: &mut V) -> bool {
        if CheckedMulFastOp::<u32, i32>::is_supported {
            return unsafe { CheckedMulFastOp::<u32, i32>::Do(self, r, out) };
        }
        if (!<u32 as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<u32>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as i32)
        {
            return false;
        }
        if (!<u32 as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i32, V> for u32
where
    i64: Cast<V>,
{
    fn modulo(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<u32>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as i32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u32, V> for u32
where
    u32: BuiltinTriple<u32, V>,
    u32: Cast<V>,
    u64: Cast<V>,
    u32: Cast<V>,
{
    fn add(self, r: u32, out: &mut V) -> bool {
        if CheckedAddFastOp::<u32, u32>::is_supported {
            return unsafe { CheckedAddFastOp::<u32, u32>::Do(self, r, out) };
        }
        if !<u32 as Cast<u32>>::range(self).IsValid() || !<u32 as Cast<u32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u32;
        let valid = u32_ops::CheckedAddImpl(self as u32, r as u32, &mut result);
        if !valid || !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u32, out: &mut V) -> bool {
        if CheckedSubFastOp::<u32, u32>::is_supported {
            return unsafe { CheckedSubFastOp::<u32, u32>::Do(self, r, out) };
        }
        if !<u32 as Cast<u32>>::range(self).IsValid() || !<u32 as Cast<u32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u32;
        let valid = u32_ops::CheckedSubImpl(self as u32, r as u32, &mut result);
        if !valid || !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u32, out: &mut V) -> bool {
        if CheckedMulFastOp::<u32, u32>::is_supported {
            return unsafe { CheckedMulFastOp::<u32, u32>::Do(self, r, out) };
        }
        if (!<u32 as Cast<u64>>::range(self).IsValid() || !<u32 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u32 as Cast<u32>>::CONTAINED
            && (self as u32) == u32::MIN
            && r == (-1i64 as u32)
        {
            return false;
        }
        if (!<u32 as Cast<u32>>::range(self).IsValid() || !<u32 as Cast<u32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u32) / (r as u32);
        if !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u32, V> for u32
where
    u32: Cast<V>,
{
    fn modulo(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u32 as Cast<u32>>::CONTAINED
            && (self as u32) == u32::MIN
            && r == (-1i64 as u32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u32) % (r as u32);
        if !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i64, V> for u32
where
    u32: BuiltinTriple<i64, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: i64, out: &mut V) -> bool {
        if CheckedAddFastOp::<u32, i64>::is_supported {
            return unsafe { CheckedAddFastOp::<u32, i64>::Do(self, r, out) };
        }
        if !<u32 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i64, out: &mut V) -> bool {
        if CheckedSubFastOp::<u32, i64>::is_supported {
            return unsafe { CheckedSubFastOp::<u32, i64>::Do(self, r, out) };
        }
        if !<u32 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i64, out: &mut V) -> bool {
        if CheckedMulFastOp::<u32, i64>::is_supported {
            return unsafe { CheckedMulFastOp::<u32, i64>::Do(self, r, out) };
        }
        if (!<u32 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<u32>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as i64)
        {
            return false;
        }
        if (!<u32 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i64, V> for u32
where
    i64: Cast<V>,
{
    fn modulo(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<u32>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as i64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u64, V> for u32
where
    u32: BuiltinTriple<u64, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u64: Cast<V>,
{
    fn add(self, r: u64, out: &mut V) -> bool {
        if CheckedAddFastOp::<u32, u64>::is_supported {
            return unsafe { CheckedAddFastOp::<u32, u64>::Do(self, r, out) };
        }
        if !<u32 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u64, out: &mut V) -> bool {
        if CheckedSubFastOp::<u32, u64>::is_supported {
            return unsafe { CheckedSubFastOp::<u32, u64>::Do(self, r, out) };
        }
        if !<u32 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u64, out: &mut V) -> bool {
        if CheckedMulFastOp::<u32, u64>::is_supported {
            return unsafe { CheckedMulFastOp::<u32, u64>::Do(self, r, out) };
        }
        if (!<u32 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u32>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u64)
        {
            return false;
        }
        if (!<u32 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u64) / (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u64, V> for u32
where
    u64: Cast<V>,
{
    fn modulo(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u32>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u64) % (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<isize, V> for u32
where
    u32: BuiltinTriple<isize, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: isize, out: &mut V) -> bool {
        if CheckedAddFastOp::<u32, isize>::is_supported {
            return unsafe { CheckedAddFastOp::<u32, isize>::Do(self, r, out) };
        }
        if !<u32 as Cast<i64>>::range(self).IsValid() || !<isize as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: isize, out: &mut V) -> bool {
        if CheckedSubFastOp::<u32, isize>::is_supported {
            return unsafe { CheckedSubFastOp::<u32, isize>::Do(self, r, out) };
        }
        if !<u32 as Cast<i64>>::range(self).IsValid() || !<isize as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: isize, out: &mut V) -> bool {
        if CheckedMulFastOp::<u32, isize>::is_supported {
            return unsafe { CheckedMulFastOp::<u32, isize>::Do(self, r, out) };
        }
        if (!<u32 as Cast<i64>>::range(self).IsValid() || !<isize as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<u32>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as isize)
        {
            return false;
        }
        if (!<u32 as Cast<i64>>::range(self).IsValid() || !<isize as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<isize, V> for u32
where
    i64: Cast<V>,
{
    fn modulo(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<u32>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as isize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<usize, V> for u32
where
    u32: BuiltinTriple<usize, V>,
    usize: Cast<V>,
    u64: Cast<V>,
    usize: Cast<V>,
{
    fn add(self, r: usize, out: &mut V) -> bool {
        if CheckedAddFastOp::<u32, usize>::is_supported {
            return unsafe { CheckedAddFastOp::<u32, usize>::Do(self, r, out) };
        }
        if !<u32 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedAddImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: usize, out: &mut V) -> bool {
        if CheckedSubFastOp::<u32, usize>::is_supported {
            return unsafe { CheckedSubFastOp::<u32, usize>::Do(self, r, out) };
        }
        if !<u32 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedSubImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: usize, out: &mut V) -> bool {
        if CheckedMulFastOp::<u32, usize>::is_supported {
            return unsafe { CheckedMulFastOp::<u32, usize>::Do(self, r, out) };
        }
        if (!<u32 as Cast<u64>>::range(self).IsValid() || !<usize as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<u32>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as usize)
        {
            return false;
        }
        if (!<u32 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as usize) / (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<usize, V> for u32
where
    usize: Cast<V>,
{
    fn modulo(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<u32>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as usize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as usize) % (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i8, V> for i64
where
    i64: BuiltinTriple<i8, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: i8, out: &mut V) -> bool {
        if CheckedAddFastOp::<i64, i8>::is_supported {
            return unsafe { CheckedAddFastOp::<i64, i8>::Do(self, r, out) };
        }
        if !<i64 as Cast<i64>>::range(self).IsValid() || !<i8 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i8, out: &mut V) -> bool {
        if CheckedSubFastOp::<i64, i8>::is_supported {
            return unsafe { CheckedSubFastOp::<i64, i8>::Do(self, r, out) };
        }
        if !<i64 as Cast<i64>>::range(self).IsValid() || !<i8 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i8, out: &mut V) -> bool {
        if CheckedMulFastOp::<i64, i8>::is_supported {
            return unsafe { CheckedMulFastOp::<i64, i8>::Do(self, r, out) };
        }
        if (!<i64 as Cast<i64>>::range(self).IsValid() || !<i8 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i64 as Cast<i64>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as i8)
        {
            return false;
        }
        if (!<i64 as Cast<i64>>::range(self).IsValid() || !<i8 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i8, V> for i64
where
    i64: Cast<V>,
{
    fn modulo(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i64 as Cast<i64>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as i8)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u8, V> for i64
where
    i64: BuiltinTriple<u8, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: u8, out: &mut V) -> bool {
        if CheckedAddFastOp::<i64, u8>::is_supported {
            return unsafe { CheckedAddFastOp::<i64, u8>::Do(self, r, out) };
        }
        if !<i64 as Cast<i64>>::range(self).IsValid() || !<u8 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u8, out: &mut V) -> bool {
        if CheckedSubFastOp::<i64, u8>::is_supported {
            return unsafe { CheckedSubFastOp::<i64, u8>::Do(self, r, out) };
        }
        if !<i64 as Cast<i64>>::range(self).IsValid() || !<u8 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u8, out: &mut V) -> bool {
        if CheckedMulFastOp::<i64, u8>::is_supported {
            return unsafe { CheckedMulFastOp::<i64, u8>::Do(self, r, out) };
        }
        if (!<i64 as Cast<i64>>::range(self).IsValid() || !<u8 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i64 as Cast<i64>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as u8)
        {
            return false;
        }
        if (!<i64 as Cast<i64>>::range(self).IsValid() || !<u8 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u8, V> for i64
where
    i64: Cast<V>,
{
    fn modulo(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i64 as Cast<i64>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as u8)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i16, V> for i64
where
    i64: BuiltinTriple<i16, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: i16, out: &mut V) -> bool {
        if CheckedAddFastOp::<i64, i16>::is_supported {
            return unsafe { CheckedAddFastOp::<i64, i16>::Do(self, r, out) };
        }
        if !<i64 as Cast<i64>>::range(self).IsValid() || !<i16 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i16, out: &mut V) -> bool {
        if CheckedSubFastOp::<i64, i16>::is_supported {
            return unsafe { CheckedSubFastOp::<i64, i16>::Do(self, r, out) };
        }
        if !<i64 as Cast<i64>>::range(self).IsValid() || !<i16 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i16, out: &mut V) -> bool {
        if CheckedMulFastOp::<i64, i16>::is_supported {
            return unsafe { CheckedMulFastOp::<i64, i16>::Do(self, r, out) };
        }
        if (!<i64 as Cast<i64>>::range(self).IsValid() || !<i16 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i64 as Cast<i64>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as i16)
        {
            return false;
        }
        if (!<i64 as Cast<i64>>::range(self).IsValid() || !<i16 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i16, V> for i64
where
    i64: Cast<V>,
{
    fn modulo(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i64 as Cast<i64>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as i16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u16, V> for i64
where
    i64: BuiltinTriple<u16, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: u16, out: &mut V) -> bool {
        if CheckedAddFastOp::<i64, u16>::is_supported {
            return unsafe { CheckedAddFastOp::<i64, u16>::Do(self, r, out) };
        }
        if !<i64 as Cast<i64>>::range(self).IsValid() || !<u16 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u16, out: &mut V) -> bool {
        if CheckedSubFastOp::<i64, u16>::is_supported {
            return unsafe { CheckedSubFastOp::<i64, u16>::Do(self, r, out) };
        }
        if !<i64 as Cast<i64>>::range(self).IsValid() || !<u16 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u16, out: &mut V) -> bool {
        if CheckedMulFastOp::<i64, u16>::is_supported {
            return unsafe { CheckedMulFastOp::<i64, u16>::Do(self, r, out) };
        }
        if (!<i64 as Cast<i64>>::range(self).IsValid() || !<u16 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<i64>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as u16)
        {
            return false;
        }
        if (!<i64 as Cast<i64>>::range(self).IsValid() || !<u16 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u16, V> for i64
where
    i64: Cast<V>,
{
    fn modulo(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<i64>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as u16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i32, V> for i64
where
    i64: BuiltinTriple<i32, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: i32, out: &mut V) -> bool {
        if CheckedAddFastOp::<i64, i32>::is_supported {
            return unsafe { CheckedAddFastOp::<i64, i32>::Do(self, r, out) };
        }
        if !<i64 as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i32, out: &mut V) -> bool {
        if CheckedSubFastOp::<i64, i32>::is_supported {
            return unsafe { CheckedSubFastOp::<i64, i32>::Do(self, r, out) };
        }
        if !<i64 as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i32, out: &mut V) -> bool {
        if CheckedMulFastOp::<i64, i32>::is_supported {
            return unsafe { CheckedMulFastOp::<i64, i32>::Do(self, r, out) };
        }
        if (!<i64 as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i64 as Cast<i64>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as i32)
        {
            return false;
        }
        if (!<i64 as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i32, V> for i64
where
    i64: Cast<V>,
{
    fn modulo(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i64 as Cast<i64>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as i32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u32, V> for i64
where
    i64: BuiltinTriple<u32, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: u32, out: &mut V) -> bool {
        if CheckedAddFastOp::<i64, u32>::is_supported {
            return unsafe { CheckedAddFastOp::<i64, u32>::Do(self, r, out) };
        }
        if !<i64 as Cast<i64>>::range(self).IsValid() || !<u32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u32, out: &mut V) -> bool {
        if CheckedSubFastOp::<i64, u32>::is_supported {
            return unsafe { CheckedSubFastOp::<i64, u32>::Do(self, r, out) };
        }
        if !<i64 as Cast<i64>>::range(self).IsValid() || !<u32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u32, out: &mut V) -> bool {
        if CheckedMulFastOp::<i64, u32>::is_supported {
            return unsafe { CheckedMulFastOp::<i64, u32>::Do(self, r, out) };
        }
        if (!<i64 as Cast<i64>>::range(self).IsValid() || !<u32 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<i64>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as u32)
        {
            return false;
        }
        if (!<i64 as Cast<i64>>::range(self).IsValid() || !<u32 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u32, V> for i64
where
    i64: Cast<V>,
{
    fn modulo(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<i64>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as u32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i64, V> for i64
where
    i64: BuiltinTriple<i64, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: i64, out: &mut V) -> bool {
        if CheckedAddFastOp::<i64, i64>::is_supported {
            return unsafe { CheckedAddFastOp::<i64, i64>::Do(self, r, out) };
        }
        if !<i64 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i64, out: &mut V) -> bool {
        if CheckedSubFastOp::<i64, i64>::is_supported {
            return unsafe { CheckedSubFastOp::<i64, i64>::Do(self, r, out) };
        }
        if !<i64 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i64, out: &mut V) -> bool {
        if CheckedMulFastOp::<i64, i64>::is_supported {
            return unsafe { CheckedMulFastOp::<i64, i64>::Do(self, r, out) };
        }
        if (!<i64 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i64 as Cast<i64>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as i64)
        {
            return false;
        }
        if (!<i64 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i64, V> for i64
where
    i64: Cast<V>,
{
    fn modulo(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i64 as Cast<i64>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as i64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u64, V> for i64
where
    i64: BuiltinTriple<u64, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u64: Cast<V>,
{
    fn add(self, r: u64, out: &mut V) -> bool {
        if CheckedAddFastOp::<i64, u64>::is_supported {
            return unsafe { CheckedAddFastOp::<i64, u64>::Do(self, r, out) };
        }
        if !<i64 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u64, out: &mut V) -> bool {
        if CheckedSubFastOp::<i64, u64>::is_supported {
            return unsafe { CheckedSubFastOp::<i64, u64>::Do(self, r, out) };
        }
        if !<i64 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u64, out: &mut V) -> bool {
        if CheckedMulFastOp::<i64, u64>::is_supported {
            return unsafe { CheckedMulFastOp::<i64, u64>::Do(self, r, out) };
        }
        if (!<i64 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<i64>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u64)
        {
            return false;
        }
        if (!<i64 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u64) / (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u64, V> for i64
where
    u64: Cast<V>,
{
    fn modulo(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<i64>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u64) % (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<isize, V> for i64
where
    i64: BuiltinTriple<isize, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: isize, out: &mut V) -> bool {
        if CheckedAddFastOp::<i64, isize>::is_supported {
            return unsafe { CheckedAddFastOp::<i64, isize>::Do(self, r, out) };
        }
        if !<i64 as Cast<i64>>::range(self).IsValid() || !<isize as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: isize, out: &mut V) -> bool {
        if CheckedSubFastOp::<i64, isize>::is_supported {
            return unsafe { CheckedSubFastOp::<i64, isize>::Do(self, r, out) };
        }
        if !<i64 as Cast<i64>>::range(self).IsValid() || !<isize as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: isize, out: &mut V) -> bool {
        if CheckedMulFastOp::<i64, isize>::is_supported {
            return unsafe { CheckedMulFastOp::<i64, isize>::Do(self, r, out) };
        }
        if (!<i64 as Cast<i64>>::range(self).IsValid() || !<isize as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true
            && <i64 as Cast<i64>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as isize)
        {
            return false;
        }
        if (!<i64 as Cast<i64>>::range(self).IsValid() || !<isize as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<isize, V> for i64
where
    i64: Cast<V>,
{
    fn modulo(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true
            && <i64 as Cast<i64>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as isize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<usize, V> for i64
where
    i64: BuiltinTriple<usize, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: usize, out: &mut V) -> bool {
        if CheckedAddFastOp::<i64, usize>::is_supported {
            return unsafe { CheckedAddFastOp::<i64, usize>::Do(self, r, out) };
        }
        if !<i64 as Cast<i64>>::range(self).IsValid() || !<usize as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: usize, out: &mut V) -> bool {
        if CheckedSubFastOp::<i64, usize>::is_supported {
            return unsafe { CheckedSubFastOp::<i64, usize>::Do(self, r, out) };
        }
        if !<i64 as Cast<i64>>::range(self).IsValid() || !<usize as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: usize, out: &mut V) -> bool {
        if CheckedMulFastOp::<i64, usize>::is_supported {
            return unsafe { CheckedMulFastOp::<i64, usize>::Do(self, r, out) };
        }
        if (!<i64 as Cast<i64>>::range(self).IsValid() || !<usize as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<i64>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as usize)
        {
            return false;
        }
        if (!<i64 as Cast<i64>>::range(self).IsValid() || !<usize as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<usize, V> for i64
where
    i64: Cast<V>,
{
    fn modulo(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<i64>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as usize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i8, V> for u64
where
    u64: BuiltinTriple<i8, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u64: Cast<V>,
{
    fn add(self, r: i8, out: &mut V) -> bool {
        if CheckedAddFastOp::<u64, i8>::is_supported {
            return unsafe { CheckedAddFastOp::<u64, i8>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<i8 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i8, out: &mut V) -> bool {
        if CheckedSubFastOp::<u64, i8>::is_supported {
            return unsafe { CheckedSubFastOp::<u64, i8>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<i8 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i8, out: &mut V) -> bool {
        if CheckedMulFastOp::<u64, i8>::is_supported {
            return unsafe { CheckedMulFastOp::<u64, i8>::Do(self, r, out) };
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<i8 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <u64 as Cast<u64>>::CONTAINED && (self as u64) == u64::MIN && r == (-1i64 as i8)
        {
            return false;
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<i8 as Cast<u64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u64) / (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i8, V> for u64
where
    u64: Cast<V>,
{
    fn modulo(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <u64 as Cast<u64>>::CONTAINED && (self as u64) == u64::MIN && r == (-1i64 as i8)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u64) % (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u8, V> for u64
where
    u64: BuiltinTriple<u8, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u64: Cast<V>,
{
    fn add(self, r: u8, out: &mut V) -> bool {
        if CheckedAddFastOp::<u64, u8>::is_supported {
            return unsafe { CheckedAddFastOp::<u64, u8>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<u8 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u8, out: &mut V) -> bool {
        if CheckedSubFastOp::<u64, u8>::is_supported {
            return unsafe { CheckedSubFastOp::<u64, u8>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<u8 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u8, out: &mut V) -> bool {
        if CheckedMulFastOp::<u64, u8>::is_supported {
            return unsafe { CheckedMulFastOp::<u64, u8>::Do(self, r, out) };
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<u8 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <u64 as Cast<u64>>::CONTAINED && (self as u64) == u64::MIN && r == (-1i64 as u8)
        {
            return false;
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<u8 as Cast<u64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u64) / (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u8, V> for u64
where
    u64: Cast<V>,
{
    fn modulo(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <u64 as Cast<u64>>::CONTAINED && (self as u64) == u64::MIN && r == (-1i64 as u8)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u64) % (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i16, V> for u64
where
    u64: BuiltinTriple<i16, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u64: Cast<V>,
{
    fn add(self, r: i16, out: &mut V) -> bool {
        if CheckedAddFastOp::<u64, i16>::is_supported {
            return unsafe { CheckedAddFastOp::<u64, i16>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<i16 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i16, out: &mut V) -> bool {
        if CheckedSubFastOp::<u64, i16>::is_supported {
            return unsafe { CheckedSubFastOp::<u64, i16>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<i16 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i16, out: &mut V) -> bool {
        if CheckedMulFastOp::<u64, i16>::is_supported {
            return unsafe { CheckedMulFastOp::<u64, i16>::Do(self, r, out) };
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<i16 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u64>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as i16)
        {
            return false;
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<i16 as Cast<u64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u64) / (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i16, V> for u64
where
    u64: Cast<V>,
{
    fn modulo(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u64>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as i16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u64) % (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u16, V> for u64
where
    u64: BuiltinTriple<u16, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u64: Cast<V>,
{
    fn add(self, r: u16, out: &mut V) -> bool {
        if CheckedAddFastOp::<u64, u16>::is_supported {
            return unsafe { CheckedAddFastOp::<u64, u16>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<u16 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u16, out: &mut V) -> bool {
        if CheckedSubFastOp::<u64, u16>::is_supported {
            return unsafe { CheckedSubFastOp::<u64, u16>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<u16 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u16, out: &mut V) -> bool {
        if CheckedMulFastOp::<u64, u16>::is_supported {
            return unsafe { CheckedMulFastOp::<u64, u16>::Do(self, r, out) };
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<u16 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u64>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u16)
        {
            return false;
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<u16 as Cast<u64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u64) / (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u16, V> for u64
where
    u64: Cast<V>,
{
    fn modulo(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u64>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u64) % (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i32, V> for u64
where
    u64: BuiltinTriple<i32, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u64: Cast<V>,
{
    fn add(self, r: i32, out: &mut V) -> bool {
        if CheckedAddFastOp::<u64, i32>::is_supported {
            return unsafe { CheckedAddFastOp::<u64, i32>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<i32 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i32, out: &mut V) -> bool {
        if CheckedSubFastOp::<u64, i32>::is_supported {
            return unsafe { CheckedSubFastOp::<u64, i32>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<i32 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i32, out: &mut V) -> bool {
        if CheckedMulFastOp::<u64, i32>::is_supported {
            return unsafe { CheckedMulFastOp::<u64, i32>::Do(self, r, out) };
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<i32 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u64>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as i32)
        {
            return false;
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<i32 as Cast<u64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u64) / (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i32, V> for u64
where
    u64: Cast<V>,
{
    fn modulo(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u64>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as i32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u64) % (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u32, V> for u64
where
    u64: BuiltinTriple<u32, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u64: Cast<V>,
{
    fn add(self, r: u32, out: &mut V) -> bool {
        if CheckedAddFastOp::<u64, u32>::is_supported {
            return unsafe { CheckedAddFastOp::<u64, u32>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<u32 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u32, out: &mut V) -> bool {
        if CheckedSubFastOp::<u64, u32>::is_supported {
            return unsafe { CheckedSubFastOp::<u64, u32>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<u32 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u32, out: &mut V) -> bool {
        if CheckedMulFastOp::<u64, u32>::is_supported {
            return unsafe { CheckedMulFastOp::<u64, u32>::Do(self, r, out) };
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<u32 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u64>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u32)
        {
            return false;
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<u32 as Cast<u64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u64) / (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u32, V> for u64
where
    u64: Cast<V>,
{
    fn modulo(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u64>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u64) % (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i64, V> for u64
where
    u64: BuiltinTriple<i64, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u64: Cast<V>,
{
    fn add(self, r: i64, out: &mut V) -> bool {
        if CheckedAddFastOp::<u64, i64>::is_supported {
            return unsafe { CheckedAddFastOp::<u64, i64>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<i64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i64, out: &mut V) -> bool {
        if CheckedSubFastOp::<u64, i64>::is_supported {
            return unsafe { CheckedSubFastOp::<u64, i64>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<i64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i64, out: &mut V) -> bool {
        if CheckedMulFastOp::<u64, i64>::is_supported {
            return unsafe { CheckedMulFastOp::<u64, i64>::Do(self, r, out) };
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<i64 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u64>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as i64)
        {
            return false;
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<i64 as Cast<u64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u64) / (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i64, V> for u64
where
    u64: Cast<V>,
{
    fn modulo(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u64>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as i64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u64) % (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u64, V> for u64
where
    u64: BuiltinTriple<u64, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u64: Cast<V>,
{
    fn add(self, r: u64, out: &mut V) -> bool {
        if CheckedAddFastOp::<u64, u64>::is_supported {
            return unsafe { CheckedAddFastOp::<u64, u64>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u64, out: &mut V) -> bool {
        if CheckedSubFastOp::<u64, u64>::is_supported {
            return unsafe { CheckedSubFastOp::<u64, u64>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u64, out: &mut V) -> bool {
        if CheckedMulFastOp::<u64, u64>::is_supported {
            return unsafe { CheckedMulFastOp::<u64, u64>::Do(self, r, out) };
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u64>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u64)
        {
            return false;
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u64) / (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u64, V> for u64
where
    u64: Cast<V>,
{
    fn modulo(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u64>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u64) % (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<isize, V> for u64
where
    u64: BuiltinTriple<isize, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u64: Cast<V>,
{
    fn add(self, r: isize, out: &mut V) -> bool {
        if CheckedAddFastOp::<u64, isize>::is_supported {
            return unsafe { CheckedAddFastOp::<u64, isize>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<isize as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: isize, out: &mut V) -> bool {
        if CheckedSubFastOp::<u64, isize>::is_supported {
            return unsafe { CheckedSubFastOp::<u64, isize>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<isize as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: isize, out: &mut V) -> bool {
        if CheckedMulFastOp::<u64, isize>::is_supported {
            return unsafe { CheckedMulFastOp::<u64, isize>::Do(self, r, out) };
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<isize as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u64>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as isize)
        {
            return false;
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<isize as Cast<u64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u64) / (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<isize, V> for u64
where
    u64: Cast<V>,
{
    fn modulo(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u64>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as isize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u64) % (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<usize, V> for u64
where
    u64: BuiltinTriple<usize, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u64: Cast<V>,
{
    fn add(self, r: usize, out: &mut V) -> bool {
        if CheckedAddFastOp::<u64, usize>::is_supported {
            return unsafe { CheckedAddFastOp::<u64, usize>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<usize as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: usize, out: &mut V) -> bool {
        if CheckedSubFastOp::<u64, usize>::is_supported {
            return unsafe { CheckedSubFastOp::<u64, usize>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<usize as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: usize, out: &mut V) -> bool {
        if CheckedMulFastOp::<u64, usize>::is_supported {
            return unsafe { CheckedMulFastOp::<u64, usize>::Do(self, r, out) };
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<usize as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u64>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as usize)
        {
            return false;
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<usize as Cast<u64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u64) / (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<usize, V> for u64
where
    u64: Cast<V>,
{
    fn modulo(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u64>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as usize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u64) % (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i8, V> for isize
where
    isize: BuiltinTriple<i8, V>,
    isize: Cast<V>,
    i64: Cast<V>,
    isize: Cast<V>,
{
    fn add(self, r: i8, out: &mut V) -> bool {
        if CheckedAddFastOp::<isize, i8>::is_supported {
            return unsafe { CheckedAddFastOp::<isize, i8>::Do(self, r, out) };
        }
        if !<isize as Cast<isize>>::range(self).IsValid()
            || !<i8 as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedAddImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i8, out: &mut V) -> bool {
        if CheckedSubFastOp::<isize, i8>::is_supported {
            return unsafe { CheckedSubFastOp::<isize, i8>::Do(self, r, out) };
        }
        if !<isize as Cast<isize>>::range(self).IsValid()
            || !<i8 as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedSubImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i8, out: &mut V) -> bool {
        if CheckedMulFastOp::<isize, i8>::is_supported {
            return unsafe { CheckedMulFastOp::<isize, i8>::Do(self, r, out) };
        }
        if (!<isize as Cast<i64>>::range(self).IsValid() || !<i8 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true
            && <isize as Cast<isize>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as i8)
        {
            return false;
        }
        if (!<isize as Cast<isize>>::range(self).IsValid()
            || !<i8 as Cast<isize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as isize) / (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i8, V> for isize
where
    isize: Cast<V>,
{
    fn modulo(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true
            && <isize as Cast<isize>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as i8)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as isize) % (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u8, V> for isize
where
    isize: BuiltinTriple<u8, V>,
    isize: Cast<V>,
    i64: Cast<V>,
    isize: Cast<V>,
{
    fn add(self, r: u8, out: &mut V) -> bool {
        if CheckedAddFastOp::<isize, u8>::is_supported {
            return unsafe { CheckedAddFastOp::<isize, u8>::Do(self, r, out) };
        }
        if !<isize as Cast<isize>>::range(self).IsValid()
            || !<u8 as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedAddImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u8, out: &mut V) -> bool {
        if CheckedSubFastOp::<isize, u8>::is_supported {
            return unsafe { CheckedSubFastOp::<isize, u8>::Do(self, r, out) };
        }
        if !<isize as Cast<isize>>::range(self).IsValid()
            || !<u8 as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedSubImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u8, out: &mut V) -> bool {
        if CheckedMulFastOp::<isize, u8>::is_supported {
            return unsafe { CheckedMulFastOp::<isize, u8>::Do(self, r, out) };
        }
        if (!<isize as Cast<i64>>::range(self).IsValid() || !<u8 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <isize as Cast<isize>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as u8)
        {
            return false;
        }
        if (!<isize as Cast<isize>>::range(self).IsValid()
            || !<u8 as Cast<isize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as isize) / (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u8, V> for isize
where
    isize: Cast<V>,
{
    fn modulo(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <isize as Cast<isize>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as u8)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as isize) % (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i16, V> for isize
where
    isize: BuiltinTriple<i16, V>,
    isize: Cast<V>,
    i64: Cast<V>,
    isize: Cast<V>,
{
    fn add(self, r: i16, out: &mut V) -> bool {
        if CheckedAddFastOp::<isize, i16>::is_supported {
            return unsafe { CheckedAddFastOp::<isize, i16>::Do(self, r, out) };
        }
        if !<isize as Cast<isize>>::range(self).IsValid()
            || !<i16 as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedAddImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i16, out: &mut V) -> bool {
        if CheckedSubFastOp::<isize, i16>::is_supported {
            return unsafe { CheckedSubFastOp::<isize, i16>::Do(self, r, out) };
        }
        if !<isize as Cast<isize>>::range(self).IsValid()
            || !<i16 as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedSubImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i16, out: &mut V) -> bool {
        if CheckedMulFastOp::<isize, i16>::is_supported {
            return unsafe { CheckedMulFastOp::<isize, i16>::Do(self, r, out) };
        }
        if (!<isize as Cast<i64>>::range(self).IsValid() || !<i16 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true
            && <isize as Cast<isize>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as i16)
        {
            return false;
        }
        if (!<isize as Cast<isize>>::range(self).IsValid()
            || !<i16 as Cast<isize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as isize) / (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i16, V> for isize
where
    isize: Cast<V>,
{
    fn modulo(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true
            && <isize as Cast<isize>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as i16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as isize) % (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u16, V> for isize
where
    isize: BuiltinTriple<u16, V>,
    isize: Cast<V>,
    i64: Cast<V>,
    isize: Cast<V>,
{
    fn add(self, r: u16, out: &mut V) -> bool {
        if CheckedAddFastOp::<isize, u16>::is_supported {
            return unsafe { CheckedAddFastOp::<isize, u16>::Do(self, r, out) };
        }
        if !<isize as Cast<isize>>::range(self).IsValid()
            || !<u16 as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedAddImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u16, out: &mut V) -> bool {
        if CheckedSubFastOp::<isize, u16>::is_supported {
            return unsafe { CheckedSubFastOp::<isize, u16>::Do(self, r, out) };
        }
        if !<isize as Cast<isize>>::range(self).IsValid()
            || !<u16 as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedSubImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u16, out: &mut V) -> bool {
        if CheckedMulFastOp::<isize, u16>::is_supported {
            return unsafe { CheckedMulFastOp::<isize, u16>::Do(self, r, out) };
        }
        if (!<isize as Cast<i64>>::range(self).IsValid() || !<u16 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <isize as Cast<isize>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as u16)
        {
            return false;
        }
        if (!<isize as Cast<isize>>::range(self).IsValid()
            || !<u16 as Cast<isize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as isize) / (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u16, V> for isize
where
    isize: Cast<V>,
{
    fn modulo(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <isize as Cast<isize>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as u16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as isize) % (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i32, V> for isize
where
    isize: BuiltinTriple<i32, V>,
    i32: Cast<V>,
    i64: Cast<V>,
    i32: Cast<V>,
{
    fn add(self, r: i32, out: &mut V) -> bool {
        if CheckedAddFastOp::<isize, i32>::is_supported {
            return unsafe { CheckedAddFastOp::<isize, i32>::Do(self, r, out) };
        }
        if !<isize as Cast<i32>>::range(self).IsValid() || !<i32 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedAddImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i32, out: &mut V) -> bool {
        if CheckedSubFastOp::<isize, i32>::is_supported {
            return unsafe { CheckedSubFastOp::<isize, i32>::Do(self, r, out) };
        }
        if !<isize as Cast<i32>>::range(self).IsValid() || !<i32 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedSubImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i32, out: &mut V) -> bool {
        if CheckedMulFastOp::<isize, i32>::is_supported {
            return unsafe { CheckedMulFastOp::<isize, i32>::Do(self, r, out) };
        }
        if (!<isize as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true
            && <i32 as Cast<isize>>::CONTAINED
            && (self as i32) == i32::MIN
            && r == (-1i64 as i32)
        {
            return false;
        }
        if (!<isize as Cast<i32>>::range(self).IsValid() || !<i32 as Cast<i32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i32) / (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i32, V> for isize
where
    i32: Cast<V>,
{
    fn modulo(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true
            && <i32 as Cast<isize>>::CONTAINED
            && (self as i32) == i32::MIN
            && r == (-1i64 as i32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i32) % (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u32, V> for isize
where
    isize: BuiltinTriple<u32, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: u32, out: &mut V) -> bool {
        if CheckedAddFastOp::<isize, u32>::is_supported {
            return unsafe { CheckedAddFastOp::<isize, u32>::Do(self, r, out) };
        }
        if !<isize as Cast<i64>>::range(self).IsValid() || !<u32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u32, out: &mut V) -> bool {
        if CheckedSubFastOp::<isize, u32>::is_supported {
            return unsafe { CheckedSubFastOp::<isize, u32>::Do(self, r, out) };
        }
        if !<isize as Cast<i64>>::range(self).IsValid() || !<u32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u32, out: &mut V) -> bool {
        if CheckedMulFastOp::<isize, u32>::is_supported {
            return unsafe { CheckedMulFastOp::<isize, u32>::Do(self, r, out) };
        }
        if (!<isize as Cast<i64>>::range(self).IsValid() || !<u32 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<isize>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as u32)
        {
            return false;
        }
        if (!<isize as Cast<i64>>::range(self).IsValid() || !<u32 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u32, V> for isize
where
    i64: Cast<V>,
{
    fn modulo(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<isize>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as u32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i64, V> for isize
where
    isize: BuiltinTriple<i64, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: i64, out: &mut V) -> bool {
        if CheckedAddFastOp::<isize, i64>::is_supported {
            return unsafe { CheckedAddFastOp::<isize, i64>::Do(self, r, out) };
        }
        if !<isize as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i64, out: &mut V) -> bool {
        if CheckedSubFastOp::<isize, i64>::is_supported {
            return unsafe { CheckedSubFastOp::<isize, i64>::Do(self, r, out) };
        }
        if !<isize as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i64, out: &mut V) -> bool {
        if CheckedMulFastOp::<isize, i64>::is_supported {
            return unsafe { CheckedMulFastOp::<isize, i64>::Do(self, r, out) };
        }
        if (!<isize as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true
            && <i64 as Cast<isize>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as i64)
        {
            return false;
        }
        if (!<isize as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i64, V> for isize
where
    i64: Cast<V>,
{
    fn modulo(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true
            && <i64 as Cast<isize>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as i64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u64, V> for isize
where
    isize: BuiltinTriple<u64, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u64: Cast<V>,
{
    fn add(self, r: u64, out: &mut V) -> bool {
        if CheckedAddFastOp::<isize, u64>::is_supported {
            return unsafe { CheckedAddFastOp::<isize, u64>::Do(self, r, out) };
        }
        if !<isize as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u64, out: &mut V) -> bool {
        if CheckedSubFastOp::<isize, u64>::is_supported {
            return unsafe { CheckedSubFastOp::<isize, u64>::Do(self, r, out) };
        }
        if !<isize as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u64, out: &mut V) -> bool {
        if CheckedMulFastOp::<isize, u64>::is_supported {
            return unsafe { CheckedMulFastOp::<isize, u64>::Do(self, r, out) };
        }
        if (!<isize as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<isize>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u64)
        {
            return false;
        }
        if (!<isize as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u64) / (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u64, V> for isize
where
    u64: Cast<V>,
{
    fn modulo(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<isize>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u64) % (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<isize, V> for isize
where
    isize: BuiltinTriple<isize, V>,
    isize: Cast<V>,
    i64: Cast<V>,
    isize: Cast<V>,
{
    fn add(self, r: isize, out: &mut V) -> bool {
        if CheckedAddFastOp::<isize, isize>::is_supported {
            return unsafe { CheckedAddFastOp::<isize, isize>::Do(self, r, out) };
        }
        if !<isize as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedAddImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: isize, out: &mut V) -> bool {
        if CheckedSubFastOp::<isize, isize>::is_supported {
            return unsafe { CheckedSubFastOp::<isize, isize>::Do(self, r, out) };
        }
        if !<isize as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedSubImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: isize, out: &mut V) -> bool {
        if CheckedMulFastOp::<isize, isize>::is_supported {
            return unsafe { CheckedMulFastOp::<isize, isize>::Do(self, r, out) };
        }
        if (!<isize as Cast<i64>>::range(self).IsValid()
            || !<isize as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true
            && <isize as Cast<isize>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as isize)
        {
            return false;
        }
        if (!<isize as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as isize) / (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<isize, V> for isize
where
    isize: Cast<V>,
{
    fn modulo(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true
            && <isize as Cast<isize>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as isize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as isize) % (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<usize, V> for isize
where
    isize: BuiltinTriple<usize, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: usize, out: &mut V) -> bool {
        if CheckedAddFastOp::<isize, usize>::is_supported {
            return unsafe { CheckedAddFastOp::<isize, usize>::Do(self, r, out) };
        }
        if !<isize as Cast<i64>>::range(self).IsValid() || !<usize as Cast<i64>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: usize, out: &mut V) -> bool {
        if CheckedSubFastOp::<isize, usize>::is_supported {
            return unsafe { CheckedSubFastOp::<isize, usize>::Do(self, r, out) };
        }
        if !<isize as Cast<i64>>::range(self).IsValid() || !<usize as Cast<i64>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: usize, out: &mut V) -> bool {
        if CheckedMulFastOp::<isize, usize>::is_supported {
            return unsafe { CheckedMulFastOp::<isize, usize>::Do(self, r, out) };
        }
        if (!<isize as Cast<i64>>::range(self).IsValid()
            || !<usize as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<isize>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as usize)
        {
            return false;
        }
        if (!<isize as Cast<i64>>::range(self).IsValid()
            || !<usize as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<usize, V> for isize
where
    i64: Cast<V>,
{
    fn modulo(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<isize>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as usize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i8, V> for usize
where
    usize: BuiltinTriple<i8, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: i8, out: &mut V) -> bool {
        if CheckedAddFastOp::<usize, i8>::is_supported {
            return unsafe { CheckedAddFastOp::<usize, i8>::Do(self, r, out) };
        }
        if !<usize as Cast<i64>>::range(self).IsValid() || !<i8 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i8, out: &mut V) -> bool {
        if CheckedSubFastOp::<usize, i8>::is_supported {
            return unsafe { CheckedSubFastOp::<usize, i8>::Do(self, r, out) };
        }
        if !<usize as Cast<i64>>::range(self).IsValid() || !<i8 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i8, out: &mut V) -> bool {
        if CheckedMulFastOp::<usize, i8>::is_supported {
            return unsafe { CheckedMulFastOp::<usize, i8>::Do(self, r, out) };
        }
        if (!<usize as Cast<i64>>::range(self).IsValid() || !<i8 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<usize>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as i8)
        {
            return false;
        }
        if (!<usize as Cast<i64>>::range(self).IsValid() || !<i8 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i8, V> for usize
where
    i64: Cast<V>,
{
    fn modulo(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<usize>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as i8)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u8, V> for usize
where
    usize: BuiltinTriple<u8, V>,
    usize: Cast<V>,
    u64: Cast<V>,
    usize: Cast<V>,
{
    fn add(self, r: u8, out: &mut V) -> bool {
        if CheckedAddFastOp::<usize, u8>::is_supported {
            return unsafe { CheckedAddFastOp::<usize, u8>::Do(self, r, out) };
        }
        if !<usize as Cast<usize>>::range(self).IsValid()
            || !<u8 as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedAddImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u8, out: &mut V) -> bool {
        if CheckedSubFastOp::<usize, u8>::is_supported {
            return unsafe { CheckedSubFastOp::<usize, u8>::Do(self, r, out) };
        }
        if !<usize as Cast<usize>>::range(self).IsValid()
            || !<u8 as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedSubImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u8, out: &mut V) -> bool {
        if CheckedMulFastOp::<usize, u8>::is_supported {
            return unsafe { CheckedMulFastOp::<usize, u8>::Do(self, r, out) };
        }
        if (!<usize as Cast<u64>>::range(self).IsValid() || !<u8 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<usize>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as u8)
        {
            return false;
        }
        if (!<usize as Cast<usize>>::range(self).IsValid()
            || !<u8 as Cast<usize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as usize) / (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u8, V> for usize
where
    usize: Cast<V>,
{
    fn modulo(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<usize>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as u8)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as usize) % (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i16, V> for usize
where
    usize: BuiltinTriple<i16, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: i16, out: &mut V) -> bool {
        if CheckedAddFastOp::<usize, i16>::is_supported {
            return unsafe { CheckedAddFastOp::<usize, i16>::Do(self, r, out) };
        }
        if !<usize as Cast<i64>>::range(self).IsValid() || !<i16 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i16, out: &mut V) -> bool {
        if CheckedSubFastOp::<usize, i16>::is_supported {
            return unsafe { CheckedSubFastOp::<usize, i16>::Do(self, r, out) };
        }
        if !<usize as Cast<i64>>::range(self).IsValid() || !<i16 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i16, out: &mut V) -> bool {
        if CheckedMulFastOp::<usize, i16>::is_supported {
            return unsafe { CheckedMulFastOp::<usize, i16>::Do(self, r, out) };
        }
        if (!<usize as Cast<i64>>::range(self).IsValid() || !<i16 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<usize>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as i16)
        {
            return false;
        }
        if (!<usize as Cast<i64>>::range(self).IsValid() || !<i16 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i16, V> for usize
where
    i64: Cast<V>,
{
    fn modulo(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<usize>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as i16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u16, V> for usize
where
    usize: BuiltinTriple<u16, V>,
    usize: Cast<V>,
    u64: Cast<V>,
    usize: Cast<V>,
{
    fn add(self, r: u16, out: &mut V) -> bool {
        if CheckedAddFastOp::<usize, u16>::is_supported {
            return unsafe { CheckedAddFastOp::<usize, u16>::Do(self, r, out) };
        }
        if !<usize as Cast<usize>>::range(self).IsValid()
            || !<u16 as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedAddImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u16, out: &mut V) -> bool {
        if CheckedSubFastOp::<usize, u16>::is_supported {
            return unsafe { CheckedSubFastOp::<usize, u16>::Do(self, r, out) };
        }
        if !<usize as Cast<usize>>::range(self).IsValid()
            || !<u16 as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedSubImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u16, out: &mut V) -> bool {
        if CheckedMulFastOp::<usize, u16>::is_supported {
            return unsafe { CheckedMulFastOp::<usize, u16>::Do(self, r, out) };
        }
        if (!<usize as Cast<u64>>::range(self).IsValid() || !<u16 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<usize>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as u16)
        {
            return false;
        }
        if (!<usize as Cast<usize>>::range(self).IsValid()
            || !<u16 as Cast<usize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as usize) / (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u16, V> for usize
where
    usize: Cast<V>,
{
    fn modulo(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<usize>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as u16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as usize) % (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i32, V> for usize
where
    usize: BuiltinTriple<i32, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: i32, out: &mut V) -> bool {
        if CheckedAddFastOp::<usize, i32>::is_supported {
            return unsafe { CheckedAddFastOp::<usize, i32>::Do(self, r, out) };
        }
        if !<usize as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i32, out: &mut V) -> bool {
        if CheckedSubFastOp::<usize, i32>::is_supported {
            return unsafe { CheckedSubFastOp::<usize, i32>::Do(self, r, out) };
        }
        if !<usize as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i32, out: &mut V) -> bool {
        if CheckedMulFastOp::<usize, i32>::is_supported {
            return unsafe { CheckedMulFastOp::<usize, i32>::Do(self, r, out) };
        }
        if (!<usize as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<usize>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as i32)
        {
            return false;
        }
        if (!<usize as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i32, V> for usize
where
    i64: Cast<V>,
{
    fn modulo(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<usize>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as i32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u32, V> for usize
where
    usize: BuiltinTriple<u32, V>,
    u32: Cast<V>,
    u64: Cast<V>,
    u32: Cast<V>,
{
    fn add(self, r: u32, out: &mut V) -> bool {
        if CheckedAddFastOp::<usize, u32>::is_supported {
            return unsafe { CheckedAddFastOp::<usize, u32>::Do(self, r, out) };
        }
        if !<usize as Cast<u32>>::range(self).IsValid() || !<u32 as Cast<u32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u32;
        let valid = u32_ops::CheckedAddImpl(self as u32, r as u32, &mut result);
        if !valid || !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u32, out: &mut V) -> bool {
        if CheckedSubFastOp::<usize, u32>::is_supported {
            return unsafe { CheckedSubFastOp::<usize, u32>::Do(self, r, out) };
        }
        if !<usize as Cast<u32>>::range(self).IsValid() || !<u32 as Cast<u32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u32;
        let valid = u32_ops::CheckedSubImpl(self as u32, r as u32, &mut result);
        if !valid || !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u32, out: &mut V) -> bool {
        if CheckedMulFastOp::<usize, u32>::is_supported {
            return unsafe { CheckedMulFastOp::<usize, u32>::Do(self, r, out) };
        }
        if (!<usize as Cast<u64>>::range(self).IsValid() || !<u32 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u32 as Cast<usize>>::CONTAINED
            && (self as u32) == u32::MIN
            && r == (-1i64 as u32)
        {
            return false;
        }
        if (!<usize as Cast<u32>>::range(self).IsValid() || !<u32 as Cast<u32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u32) / (r as u32);
        if !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u32, V> for usize
where
    u32: Cast<V>,
{
    fn modulo(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u32 as Cast<usize>>::CONTAINED
            && (self as u32) == u32::MIN
            && r == (-1i64 as u32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u32) % (r as u32);
        if !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<i64, V> for usize
where
    usize: BuiltinTriple<i64, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: i64, out: &mut V) -> bool {
        if CheckedAddFastOp::<usize, i64>::is_supported {
            return unsafe { CheckedAddFastOp::<usize, i64>::Do(self, r, out) };
        }
        if !<usize as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i64, out: &mut V) -> bool {
        if CheckedSubFastOp::<usize, i64>::is_supported {
            return unsafe { CheckedSubFastOp::<usize, i64>::Do(self, r, out) };
        }
        if !<usize as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i64, out: &mut V) -> bool {
        if CheckedMulFastOp::<usize, i64>::is_supported {
            return unsafe { CheckedMulFastOp::<usize, i64>::Do(self, r, out) };
        }
        if (!<usize as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<usize>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as i64)
        {
            return false;
        }
        if (!<usize as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<i64, V> for usize
where
    i64: Cast<V>,
{
    fn modulo(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<usize>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as i64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<u64, V> for usize
where
    usize: BuiltinTriple<u64, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u64: Cast<V>,
{
    fn add(self, r: u64, out: &mut V) -> bool {
        if CheckedAddFastOp::<usize, u64>::is_supported {
            return unsafe { CheckedAddFastOp::<usize, u64>::Do(self, r, out) };
        }
        if !<usize as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u64, out: &mut V) -> bool {
        if CheckedSubFastOp::<usize, u64>::is_supported {
            return unsafe { CheckedSubFastOp::<usize, u64>::Do(self, r, out) };
        }
        if !<usize as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u64, out: &mut V) -> bool {
        if CheckedMulFastOp::<usize, u64>::is_supported {
            return unsafe { CheckedMulFastOp::<usize, u64>::Do(self, r, out) };
        }
        if (!<usize as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<usize>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u64)
        {
            return false;
        }
        if (!<usize as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u64) / (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<u64, V> for usize
where
    u64: Cast<V>,
{
    fn modulo(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<usize>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u64) % (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<isize, V> for usize
where
    usize: BuiltinTriple<isize, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: isize, out: &mut V) -> bool {
        if CheckedAddFastOp::<usize, isize>::is_supported {
            return unsafe { CheckedAddFastOp::<usize, isize>::Do(self, r, out) };
        }
        if !<usize as Cast<i64>>::range(self).IsValid() || !<isize as Cast<i64>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: isize, out: &mut V) -> bool {
        if CheckedSubFastOp::<usize, isize>::is_supported {
            return unsafe { CheckedSubFastOp::<usize, isize>::Do(self, r, out) };
        }
        if !<usize as Cast<i64>>::range(self).IsValid() || !<isize as Cast<i64>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: isize, out: &mut V) -> bool {
        if CheckedMulFastOp::<usize, isize>::is_supported {
            return unsafe { CheckedMulFastOp::<usize, isize>::Do(self, r, out) };
        }
        if (!<usize as Cast<i64>>::range(self).IsValid()
            || !<isize as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<usize>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as isize)
        {
            return false;
        }
        if (!<usize as Cast<i64>>::range(self).IsValid()
            || !<isize as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<isize, V> for usize
where
    i64: Cast<V>,
{
    fn modulo(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<usize>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as isize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedBinary<usize, V> for usize
where
    usize: BuiltinTriple<usize, V>,
    usize: Cast<V>,
    u64: Cast<V>,
    usize: Cast<V>,
{
    fn add(self, r: usize, out: &mut V) -> bool {
        if CheckedAddFastOp::<usize, usize>::is_supported {
            return unsafe { CheckedAddFastOp::<usize, usize>::Do(self, r, out) };
        }
        if !<usize as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedAddImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: usize, out: &mut V) -> bool {
        if CheckedSubFastOp::<usize, usize>::is_supported {
            return unsafe { CheckedSubFastOp::<usize, usize>::Do(self, r, out) };
        }
        if !<usize as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedSubImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: usize, out: &mut V) -> bool {
        if CheckedMulFastOp::<usize, usize>::is_supported {
            return unsafe { CheckedMulFastOp::<usize, usize>::Do(self, r, out) };
        }
        if (!<usize as Cast<u64>>::range(self).IsValid()
            || !<usize as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<usize>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as usize)
        {
            return false;
        }
        if (!<usize as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as usize) / (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "32")]
impl<V: Numeric> CheckedModulo<usize, V> for usize
where
    usize: Cast<V>,
{
    fn modulo(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<usize>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as usize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as usize) % (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i8, V> for i8
where
    i8: BuiltinTriple<i8, V>,
    i16: Cast<V>,
    i16: Cast<V>,
    i8: Cast<V>,
{
    fn add(self, r: i8, out: &mut V) -> bool {
        if CheckedAddFastOp::<i8, i8>::is_supported {
            return unsafe { CheckedAddFastOp::<i8, i8>::Do(self, r, out) };
        }
        if !<i8 as Cast<i16>>::range(self).IsValid() || !<i8 as Cast<i16>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i16;
        let valid = i16_ops::CheckedAddImpl(self as i16, r as i16, &mut result);
        if !valid || !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i8, out: &mut V) -> bool {
        if CheckedSubFastOp::<i8, i8>::is_supported {
            return unsafe { CheckedSubFastOp::<i8, i8>::Do(self, r, out) };
        }
        if !<i8 as Cast<i16>>::range(self).IsValid() || !<i8 as Cast<i16>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i16;
        let valid = i16_ops::CheckedSubImpl(self as i16, r as i16, &mut result);
        if !valid || !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i8, out: &mut V) -> bool {
        if CheckedMulFastOp::<i8, i8>::is_supported {
            return unsafe { CheckedMulFastOp::<i8, i8>::Do(self, r, out) };
        }
        if (!<i8 as Cast<i16>>::range(self).IsValid() || !<i8 as Cast<i16>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i16;
        let valid = if CheckedMulFastOp::<i16, i16>::is_supported {
            unsafe { CheckedMulFastOp::<i16, i16>::Do(self as i16, r as i16, &mut result) }
        } else {
            i16_ops::CheckedMulImpl(self as i16, r as i16, &mut result)
        };
        if !valid || !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i8 as Cast<i8>>::CONTAINED && (self as i8) == i8::MIN && r == (-1i64 as i8) {
            return false;
        }
        if (!<i8 as Cast<i8>>::range(self).IsValid() || !<i8 as Cast<i8>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i8) / (r as i8);
        if !<i8 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i8 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i8, V> for i8
where
    i8: Cast<V>,
{
    fn modulo(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i8 as Cast<i8>>::CONTAINED && (self as i8) == i8::MIN && r == (-1i64 as i8) {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i8) % (r as i8);
        if !<i8 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i8 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u8, V> for i8
where
    i8: BuiltinTriple<u8, V>,
    i16: Cast<V>,
    i16: Cast<V>,
    i16: Cast<V>,
{
    fn add(self, r: u8, out: &mut V) -> bool {
        if CheckedAddFastOp::<i8, u8>::is_supported {
            return unsafe { CheckedAddFastOp::<i8, u8>::Do(self, r, out) };
        }
        if !<i8 as Cast<i16>>::range(self).IsValid() || !<u8 as Cast<i16>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i16;
        let valid = i16_ops::CheckedAddImpl(self as i16, r as i16, &mut result);
        if !valid || !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u8, out: &mut V) -> bool {
        if CheckedSubFastOp::<i8, u8>::is_supported {
            return unsafe { CheckedSubFastOp::<i8, u8>::Do(self, r, out) };
        }
        if !<i8 as Cast<i16>>::range(self).IsValid() || !<u8 as Cast<i16>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i16;
        let valid = i16_ops::CheckedSubImpl(self as i16, r as i16, &mut result);
        if !valid || !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u8, out: &mut V) -> bool {
        if CheckedMulFastOp::<i8, u8>::is_supported {
            return unsafe { CheckedMulFastOp::<i8, u8>::Do(self, r, out) };
        }
        if (!<i8 as Cast<i16>>::range(self).IsValid() || !<u8 as Cast<i16>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i16;
        let valid = if CheckedMulFastOp::<i16, i16>::is_supported {
            unsafe { CheckedMulFastOp::<i16, i16>::Do(self as i16, r as i16, &mut result) }
        } else {
            i16_ops::CheckedMulImpl(self as i16, r as i16, &mut result)
        };
        if !valid || !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i16 as Cast<i8>>::CONTAINED && (self as i16) == i16::MIN && r == (-1i64 as u8)
        {
            return false;
        }
        if (!<i8 as Cast<i16>>::range(self).IsValid() || !<u8 as Cast<i16>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i16) / (r as i16);
        if !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u8, V> for i8
where
    i16: Cast<V>,
{
    fn modulo(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i16 as Cast<i8>>::CONTAINED && (self as i16) == i16::MIN && r == (-1i64 as u8)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i16) % (r as i16);
        if !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i16, V> for i8
where
    i8: BuiltinTriple<i16, V>,
    i32: Cast<V>,
    i32: Cast<V>,
    i16: Cast<V>,
{
    fn add(self, r: i16, out: &mut V) -> bool {
        if CheckedAddFastOp::<i8, i16>::is_supported {
            return unsafe { CheckedAddFastOp::<i8, i16>::Do(self, r, out) };
        }
        if !<i8 as Cast<i32>>::range(self).IsValid() || !<i16 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedAddImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i16, out: &mut V) -> bool {
        if CheckedSubFastOp::<i8, i16>::is_supported {
            return unsafe { CheckedSubFastOp::<i8, i16>::Do(self, r, out) };
        }
        if !<i8 as Cast<i32>>::range(self).IsValid() || !<i16 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedSubImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i16, out: &mut V) -> bool {
        if CheckedMulFastOp::<i8, i16>::is_supported {
            return unsafe { CheckedMulFastOp::<i8, i16>::Do(self, r, out) };
        }
        if (!<i8 as Cast<i32>>::range(self).IsValid() || !<i16 as Cast<i32>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i32;
        let valid = if CheckedMulFastOp::<i32, i32>::is_supported {
            unsafe { CheckedMulFastOp::<i32, i32>::Do(self as i32, r as i32, &mut result) }
        } else {
            i32_ops::CheckedMulImpl(self as i32, r as i32, &mut result)
        };
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i16 as Cast<i8>>::CONTAINED && (self as i16) == i16::MIN && r == (-1i64 as i16)
        {
            return false;
        }
        if (!<i8 as Cast<i16>>::range(self).IsValid() || !<i16 as Cast<i16>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i16) / (r as i16);
        if !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i16, V> for i8
where
    i16: Cast<V>,
{
    fn modulo(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i16 as Cast<i8>>::CONTAINED && (self as i16) == i16::MIN && r == (-1i64 as i16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i16) % (r as i16);
        if !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u16, V> for i8
where
    i8: BuiltinTriple<u16, V>,
    i32: Cast<V>,
    i32: Cast<V>,
    i32: Cast<V>,
{
    fn add(self, r: u16, out: &mut V) -> bool {
        if CheckedAddFastOp::<i8, u16>::is_supported {
            return unsafe { CheckedAddFastOp::<i8, u16>::Do(self, r, out) };
        }
        if !<i8 as Cast<i32>>::range(self).IsValid() || !<u16 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedAddImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u16, out: &mut V) -> bool {
        if CheckedSubFastOp::<i8, u16>::is_supported {
            return unsafe { CheckedSubFastOp::<i8, u16>::Do(self, r, out) };
        }
        if !<i8 as Cast<i32>>::range(self).IsValid() || !<u16 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedSubImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u16, out: &mut V) -> bool {
        if CheckedMulFastOp::<i8, u16>::is_supported {
            return unsafe { CheckedMulFastOp::<i8, u16>::Do(self, r, out) };
        }
        if (!<i8 as Cast<i32>>::range(self).IsValid() || !<u16 as Cast<i32>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i32;
        let valid = if CheckedMulFastOp::<i32, i32>::is_supported {
            unsafe { CheckedMulFastOp::<i32, i32>::Do(self as i32, r as i32, &mut result) }
        } else {
            i32_ops::CheckedMulImpl(self as i32, r as i32, &mut result)
        };
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i32 as Cast<i8>>::CONTAINED && (self as i32) == i32::MIN && r == (-1i64 as u16)
        {
            return false;
        }
        if (!<i8 as Cast<i32>>::range(self).IsValid() || !<u16 as Cast<i32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i32) / (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u16, V> for i8
where
    i32: Cast<V>,
{
    fn modulo(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i32 as Cast<i8>>::CONTAINED && (self as i32) == i32::MIN && r == (-1i64 as u16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i32) % (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i32, V> for i8
where
    i8: BuiltinTriple<i32, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i32: Cast<V>,
{
    fn add(self, r: i32, out: &mut V) -> bool {
        if CheckedAddFastOp::<i8, i32>::is_supported {
            return unsafe { CheckedAddFastOp::<i8, i32>::Do(self, r, out) };
        }
        if !<i8 as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i32, out: &mut V) -> bool {
        if CheckedSubFastOp::<i8, i32>::is_supported {
            return unsafe { CheckedSubFastOp::<i8, i32>::Do(self, r, out) };
        }
        if !<i8 as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i32, out: &mut V) -> bool {
        if CheckedMulFastOp::<i8, i32>::is_supported {
            return unsafe { CheckedMulFastOp::<i8, i32>::Do(self, r, out) };
        }
        if (!<i8 as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i32 as Cast<i8>>::CONTAINED && (self as i32) == i32::MIN && r == (-1i64 as i32)
        {
            return false;
        }
        if (!<i8 as Cast<i32>>::range(self).IsValid() || !<i32 as Cast<i32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i32) / (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i32, V> for i8
where
    i32: Cast<V>,
{
    fn modulo(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i32 as Cast<i8>>::CONTAINED && (self as i32) == i32::MIN && r == (-1i64 as i32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i32) % (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u32, V> for i8
where
    i8: BuiltinTriple<u32, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: u32, out: &mut V) -> bool {
        if CheckedAddFastOp::<i8, u32>::is_supported {
            return unsafe { CheckedAddFastOp::<i8, u32>::Do(self, r, out) };
        }
        if !<i8 as Cast<i64>>::range(self).IsValid() || !<u32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u32, out: &mut V) -> bool {
        if CheckedSubFastOp::<i8, u32>::is_supported {
            return unsafe { CheckedSubFastOp::<i8, u32>::Do(self, r, out) };
        }
        if !<i8 as Cast<i64>>::range(self).IsValid() || !<u32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u32, out: &mut V) -> bool {
        if CheckedMulFastOp::<i8, u32>::is_supported {
            return unsafe { CheckedMulFastOp::<i8, u32>::Do(self, r, out) };
        }
        if (!<i8 as Cast<i64>>::range(self).IsValid() || !<u32 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i64 as Cast<i8>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as u32)
        {
            return false;
        }
        if (!<i8 as Cast<i64>>::range(self).IsValid() || !<u32 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u32, V> for i8
where
    i64: Cast<V>,
{
    fn modulo(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i64 as Cast<i8>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as u32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i64, V> for i8
where
    i8: BuiltinTriple<i64, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: i64, out: &mut V) -> bool {
        if CheckedAddFastOp::<i8, i64>::is_supported {
            return unsafe { CheckedAddFastOp::<i8, i64>::Do(self, r, out) };
        }
        if !<i8 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i64, out: &mut V) -> bool {
        if CheckedSubFastOp::<i8, i64>::is_supported {
            return unsafe { CheckedSubFastOp::<i8, i64>::Do(self, r, out) };
        }
        if !<i8 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i64, out: &mut V) -> bool {
        if CheckedMulFastOp::<i8, i64>::is_supported {
            return unsafe { CheckedMulFastOp::<i8, i64>::Do(self, r, out) };
        }
        if (!<i8 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i64 as Cast<i8>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as i64)
        {
            return false;
        }
        if (!<i8 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i64, V> for i8
where
    i64: Cast<V>,
{
    fn modulo(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i64 as Cast<i8>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as i64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u64, V> for i8
where
    i8: BuiltinTriple<u64, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u64: Cast<V>,
{
    fn add(self, r: u64, out: &mut V) -> bool {
        if CheckedAddFastOp::<i8, u64>::is_supported {
            return unsafe { CheckedAddFastOp::<i8, u64>::Do(self, r, out) };
        }
        if !<i8 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u64, out: &mut V) -> bool {
        if CheckedSubFastOp::<i8, u64>::is_supported {
            return unsafe { CheckedSubFastOp::<i8, u64>::Do(self, r, out) };
        }
        if !<i8 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u64, out: &mut V) -> bool {
        if CheckedMulFastOp::<i8, u64>::is_supported {
            return unsafe { CheckedMulFastOp::<i8, u64>::Do(self, r, out) };
        }
        if (!<i8 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <u64 as Cast<i8>>::CONTAINED && (self as u64) == u64::MIN && r == (-1i64 as u64)
        {
            return false;
        }
        if (!<i8 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u64) / (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u64, V> for i8
where
    u64: Cast<V>,
{
    fn modulo(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <u64 as Cast<i8>>::CONTAINED && (self as u64) == u64::MIN && r == (-1i64 as u64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u64) % (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<isize, V> for i8
where
    i8: BuiltinTriple<isize, V>,
    isize: Cast<V>,
    isize: Cast<V>,
    isize: Cast<V>,
{
    fn add(self, r: isize, out: &mut V) -> bool {
        if CheckedAddFastOp::<i8, isize>::is_supported {
            return unsafe { CheckedAddFastOp::<i8, isize>::Do(self, r, out) };
        }
        if !<i8 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedAddImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: isize, out: &mut V) -> bool {
        if CheckedSubFastOp::<i8, isize>::is_supported {
            return unsafe { CheckedSubFastOp::<i8, isize>::Do(self, r, out) };
        }
        if !<i8 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedSubImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: isize, out: &mut V) -> bool {
        if CheckedMulFastOp::<i8, isize>::is_supported {
            return unsafe { CheckedMulFastOp::<i8, isize>::Do(self, r, out) };
        }
        if (!<i8 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = if CheckedMulFastOp::<isize, isize>::is_supported {
            unsafe { CheckedMulFastOp::<isize, isize>::Do(self as isize, r as isize, &mut result) }
        } else {
            isize_ops::CheckedMulImpl(self as isize, r as isize, &mut result)
        };
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true
            && <isize as Cast<i8>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as isize)
        {
            return false;
        }
        if (!<i8 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as isize) / (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<isize, V> for i8
where
    isize: Cast<V>,
{
    fn modulo(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true
            && <isize as Cast<i8>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as isize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as isize) % (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<usize, V> for i8
where
    i8: BuiltinTriple<usize, V>,
    usize: Cast<V>,
    usize: Cast<V>,
    usize: Cast<V>,
{
    fn add(self, r: usize, out: &mut V) -> bool {
        if CheckedAddFastOp::<i8, usize>::is_supported {
            return unsafe { CheckedAddFastOp::<i8, usize>::Do(self, r, out) };
        }
        if !<i8 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedAddImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: usize, out: &mut V) -> bool {
        if CheckedSubFastOp::<i8, usize>::is_supported {
            return unsafe { CheckedSubFastOp::<i8, usize>::Do(self, r, out) };
        }
        if !<i8 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedSubImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: usize, out: &mut V) -> bool {
        if CheckedMulFastOp::<i8, usize>::is_supported {
            return unsafe { CheckedMulFastOp::<i8, usize>::Do(self, r, out) };
        }
        if (!<i8 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = if CheckedMulFastOp::<usize, usize>::is_supported {
            unsafe { CheckedMulFastOp::<usize, usize>::Do(self as usize, r as usize, &mut result) }
        } else {
            usize_ops::CheckedMulImpl(self as usize, r as usize, &mut result)
        };
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<i8>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as usize)
        {
            return false;
        }
        if (!<i8 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as usize) / (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<usize, V> for i8
where
    usize: Cast<V>,
{
    fn modulo(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<i8>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as usize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as usize) % (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i8, V> for u8
where
    u8: BuiltinTriple<i8, V>,
    i16: Cast<V>,
    i16: Cast<V>,
    i16: Cast<V>,
{
    fn add(self, r: i8, out: &mut V) -> bool {
        if CheckedAddFastOp::<u8, i8>::is_supported {
            return unsafe { CheckedAddFastOp::<u8, i8>::Do(self, r, out) };
        }
        if !<u8 as Cast<i16>>::range(self).IsValid() || !<i8 as Cast<i16>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i16;
        let valid = i16_ops::CheckedAddImpl(self as i16, r as i16, &mut result);
        if !valid || !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i8, out: &mut V) -> bool {
        if CheckedSubFastOp::<u8, i8>::is_supported {
            return unsafe { CheckedSubFastOp::<u8, i8>::Do(self, r, out) };
        }
        if !<u8 as Cast<i16>>::range(self).IsValid() || !<i8 as Cast<i16>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i16;
        let valid = i16_ops::CheckedSubImpl(self as i16, r as i16, &mut result);
        if !valid || !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i8, out: &mut V) -> bool {
        if CheckedMulFastOp::<u8, i8>::is_supported {
            return unsafe { CheckedMulFastOp::<u8, i8>::Do(self, r, out) };
        }
        if (!<u8 as Cast<i16>>::range(self).IsValid() || !<i8 as Cast<i16>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i16;
        let valid = if CheckedMulFastOp::<i16, i16>::is_supported {
            unsafe { CheckedMulFastOp::<i16, i16>::Do(self as i16, r as i16, &mut result) }
        } else {
            i16_ops::CheckedMulImpl(self as i16, r as i16, &mut result)
        };
        if !valid || !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i16 as Cast<u8>>::CONTAINED && (self as i16) == i16::MIN && r == (-1i64 as i8)
        {
            return false;
        }
        if (!<u8 as Cast<i16>>::range(self).IsValid() || !<i8 as Cast<i16>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i16) / (r as i16);
        if !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i8, V> for u8
where
    i16: Cast<V>,
{
    fn modulo(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i16 as Cast<u8>>::CONTAINED && (self as i16) == i16::MIN && r == (-1i64 as i8)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i16) % (r as i16);
        if !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u8, V> for u8
where
    u8: BuiltinTriple<u8, V>,
    u16: Cast<V>,
    u16: Cast<V>,
    u8: Cast<V>,
{
    fn add(self, r: u8, out: &mut V) -> bool {
        if CheckedAddFastOp::<u8, u8>::is_supported {
            return unsafe { CheckedAddFastOp::<u8, u8>::Do(self, r, out) };
        }
        if !<u8 as Cast<u16>>::range(self).IsValid() || !<u8 as Cast<u16>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u16;
        let valid = u16_ops::CheckedAddImpl(self as u16, r as u16, &mut result);
        if !valid || !<u16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u16 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u8, out: &mut V) -> bool {
        if CheckedSubFastOp::<u8, u8>::is_supported {
            return unsafe { CheckedSubFastOp::<u8, u8>::Do(self, r, out) };
        }
        if !<u8 as Cast<u16>>::range(self).IsValid() || !<u8 as Cast<u16>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u16;
        let valid = u16_ops::CheckedSubImpl(self as u16, r as u16, &mut result);
        if !valid || !<u16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u16 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u8, out: &mut V) -> bool {
        if CheckedMulFastOp::<u8, u8>::is_supported {
            return unsafe { CheckedMulFastOp::<u8, u8>::Do(self, r, out) };
        }
        if (!<u8 as Cast<u16>>::range(self).IsValid() || !<u8 as Cast<u16>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u16;
        let valid = if CheckedMulFastOp::<u16, u16>::is_supported {
            unsafe { CheckedMulFastOp::<u16, u16>::Do(self as u16, r as u16, &mut result) }
        } else {
            u16_ops::CheckedMulImpl(self as u16, r as u16, &mut result)
        };
        if !valid || !<u16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u16 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <u8 as Cast<u8>>::CONTAINED && (self as u8) == u8::MIN && r == (-1i64 as u8) {
            return false;
        }
        if (!<u8 as Cast<u8>>::range(self).IsValid() || !<u8 as Cast<u8>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u8) / (r as u8);
        if !<u8 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u8 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u8, V> for u8
where
    u8: Cast<V>,
{
    fn modulo(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <u8 as Cast<u8>>::CONTAINED && (self as u8) == u8::MIN && r == (-1i64 as u8) {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u8) % (r as u8);
        if !<u8 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u8 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i16, V> for u8
where
    u8: BuiltinTriple<i16, V>,
    i32: Cast<V>,
    i32: Cast<V>,
    i16: Cast<V>,
{
    fn add(self, r: i16, out: &mut V) -> bool {
        if CheckedAddFastOp::<u8, i16>::is_supported {
            return unsafe { CheckedAddFastOp::<u8, i16>::Do(self, r, out) };
        }
        if !<u8 as Cast<i32>>::range(self).IsValid() || !<i16 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedAddImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i16, out: &mut V) -> bool {
        if CheckedSubFastOp::<u8, i16>::is_supported {
            return unsafe { CheckedSubFastOp::<u8, i16>::Do(self, r, out) };
        }
        if !<u8 as Cast<i32>>::range(self).IsValid() || !<i16 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedSubImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i16, out: &mut V) -> bool {
        if CheckedMulFastOp::<u8, i16>::is_supported {
            return unsafe { CheckedMulFastOp::<u8, i16>::Do(self, r, out) };
        }
        if (!<u8 as Cast<i32>>::range(self).IsValid() || !<i16 as Cast<i32>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i32;
        let valid = if CheckedMulFastOp::<i32, i32>::is_supported {
            unsafe { CheckedMulFastOp::<i32, i32>::Do(self as i32, r as i32, &mut result) }
        } else {
            i32_ops::CheckedMulImpl(self as i32, r as i32, &mut result)
        };
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i16 as Cast<u8>>::CONTAINED && (self as i16) == i16::MIN && r == (-1i64 as i16)
        {
            return false;
        }
        if (!<u8 as Cast<i16>>::range(self).IsValid() || !<i16 as Cast<i16>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i16) / (r as i16);
        if !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i16, V> for u8
where
    i16: Cast<V>,
{
    fn modulo(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i16 as Cast<u8>>::CONTAINED && (self as i16) == i16::MIN && r == (-1i64 as i16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i16) % (r as i16);
        if !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u16, V> for u8
where
    u8: BuiltinTriple<u16, V>,
    u32: Cast<V>,
    u32: Cast<V>,
    u16: Cast<V>,
{
    fn add(self, r: u16, out: &mut V) -> bool {
        if CheckedAddFastOp::<u8, u16>::is_supported {
            return unsafe { CheckedAddFastOp::<u8, u16>::Do(self, r, out) };
        }
        if !<u8 as Cast<u32>>::range(self).IsValid() || !<u16 as Cast<u32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u32;
        let valid = u32_ops::CheckedAddImpl(self as u32, r as u32, &mut result);
        if !valid || !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u16, out: &mut V) -> bool {
        if CheckedSubFastOp::<u8, u16>::is_supported {
            return unsafe { CheckedSubFastOp::<u8, u16>::Do(self, r, out) };
        }
        if !<u8 as Cast<u32>>::range(self).IsValid() || !<u16 as Cast<u32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u32;
        let valid = u32_ops::CheckedSubImpl(self as u32, r as u32, &mut result);
        if !valid || !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u16, out: &mut V) -> bool {
        if CheckedMulFastOp::<u8, u16>::is_supported {
            return unsafe { CheckedMulFastOp::<u8, u16>::Do(self, r, out) };
        }
        if (!<u8 as Cast<u32>>::range(self).IsValid() || !<u16 as Cast<u32>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u32;
        let valid = if CheckedMulFastOp::<u32, u32>::is_supported {
            unsafe { CheckedMulFastOp::<u32, u32>::Do(self as u32, r as u32, &mut result) }
        } else {
            u32_ops::CheckedMulImpl(self as u32, r as u32, &mut result)
        };
        if !valid || !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <u16 as Cast<u8>>::CONTAINED && (self as u16) == u16::MIN && r == (-1i64 as u16)
        {
            return false;
        }
        if (!<u8 as Cast<u16>>::range(self).IsValid() || !<u16 as Cast<u16>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u16) / (r as u16);
        if !<u16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u16, V> for u8
where
    u16: Cast<V>,
{
    fn modulo(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <u16 as Cast<u8>>::CONTAINED && (self as u16) == u16::MIN && r == (-1i64 as u16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u16) % (r as u16);
        if !<u16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i32, V> for u8
where
    u8: BuiltinTriple<i32, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i32: Cast<V>,
{
    fn add(self, r: i32, out: &mut V) -> bool {
        if CheckedAddFastOp::<u8, i32>::is_supported {
            return unsafe { CheckedAddFastOp::<u8, i32>::Do(self, r, out) };
        }
        if !<u8 as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i32, out: &mut V) -> bool {
        if CheckedSubFastOp::<u8, i32>::is_supported {
            return unsafe { CheckedSubFastOp::<u8, i32>::Do(self, r, out) };
        }
        if !<u8 as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i32, out: &mut V) -> bool {
        if CheckedMulFastOp::<u8, i32>::is_supported {
            return unsafe { CheckedMulFastOp::<u8, i32>::Do(self, r, out) };
        }
        if (!<u8 as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i32 as Cast<u8>>::CONTAINED && (self as i32) == i32::MIN && r == (-1i64 as i32)
        {
            return false;
        }
        if (!<u8 as Cast<i32>>::range(self).IsValid() || !<i32 as Cast<i32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i32) / (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i32, V> for u8
where
    i32: Cast<V>,
{
    fn modulo(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i32 as Cast<u8>>::CONTAINED && (self as i32) == i32::MIN && r == (-1i64 as i32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i32) % (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u32, V> for u8
where
    u8: BuiltinTriple<u32, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u32: Cast<V>,
{
    fn add(self, r: u32, out: &mut V) -> bool {
        if CheckedAddFastOp::<u8, u32>::is_supported {
            return unsafe { CheckedAddFastOp::<u8, u32>::Do(self, r, out) };
        }
        if !<u8 as Cast<u64>>::range(self).IsValid() || !<u32 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u32, out: &mut V) -> bool {
        if CheckedSubFastOp::<u8, u32>::is_supported {
            return unsafe { CheckedSubFastOp::<u8, u32>::Do(self, r, out) };
        }
        if !<u8 as Cast<u64>>::range(self).IsValid() || !<u32 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u32, out: &mut V) -> bool {
        if CheckedMulFastOp::<u8, u32>::is_supported {
            return unsafe { CheckedMulFastOp::<u8, u32>::Do(self, r, out) };
        }
        if (!<u8 as Cast<u64>>::range(self).IsValid() || !<u32 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <u32 as Cast<u8>>::CONTAINED && (self as u32) == u32::MIN && r == (-1i64 as u32)
        {
            return false;
        }
        if (!<u8 as Cast<u32>>::range(self).IsValid() || !<u32 as Cast<u32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u32) / (r as u32);
        if !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u32, V> for u8
where
    u32: Cast<V>,
{
    fn modulo(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <u32 as Cast<u8>>::CONTAINED && (self as u32) == u32::MIN && r == (-1i64 as u32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u32) % (r as u32);
        if !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i64, V> for u8
where
    u8: BuiltinTriple<i64, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: i64, out: &mut V) -> bool {
        if CheckedAddFastOp::<u8, i64>::is_supported {
            return unsafe { CheckedAddFastOp::<u8, i64>::Do(self, r, out) };
        }
        if !<u8 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i64, out: &mut V) -> bool {
        if CheckedSubFastOp::<u8, i64>::is_supported {
            return unsafe { CheckedSubFastOp::<u8, i64>::Do(self, r, out) };
        }
        if !<u8 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i64, out: &mut V) -> bool {
        if CheckedMulFastOp::<u8, i64>::is_supported {
            return unsafe { CheckedMulFastOp::<u8, i64>::Do(self, r, out) };
        }
        if (!<u8 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i64 as Cast<u8>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as i64)
        {
            return false;
        }
        if (!<u8 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i64, V> for u8
where
    i64: Cast<V>,
{
    fn modulo(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i64 as Cast<u8>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as i64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u64, V> for u8
where
    u8: BuiltinTriple<u64, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u64: Cast<V>,
{
    fn add(self, r: u64, out: &mut V) -> bool {
        if CheckedAddFastOp::<u8, u64>::is_supported {
            return unsafe { CheckedAddFastOp::<u8, u64>::Do(self, r, out) };
        }
        if !<u8 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u64, out: &mut V) -> bool {
        if CheckedSubFastOp::<u8, u64>::is_supported {
            return unsafe { CheckedSubFastOp::<u8, u64>::Do(self, r, out) };
        }
        if !<u8 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u64, out: &mut V) -> bool {
        if CheckedMulFastOp::<u8, u64>::is_supported {
            return unsafe { CheckedMulFastOp::<u8, u64>::Do(self, r, out) };
        }
        if (!<u8 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <u64 as Cast<u8>>::CONTAINED && (self as u64) == u64::MIN && r == (-1i64 as u64)
        {
            return false;
        }
        if (!<u8 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u64) / (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u64, V> for u8
where
    u64: Cast<V>,
{
    fn modulo(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <u64 as Cast<u8>>::CONTAINED && (self as u64) == u64::MIN && r == (-1i64 as u64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u64) % (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<isize, V> for u8
where
    u8: BuiltinTriple<isize, V>,
    isize: Cast<V>,
    isize: Cast<V>,
    isize: Cast<V>,
{
    fn add(self, r: isize, out: &mut V) -> bool {
        if CheckedAddFastOp::<u8, isize>::is_supported {
            return unsafe { CheckedAddFastOp::<u8, isize>::Do(self, r, out) };
        }
        if !<u8 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedAddImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: isize, out: &mut V) -> bool {
        if CheckedSubFastOp::<u8, isize>::is_supported {
            return unsafe { CheckedSubFastOp::<u8, isize>::Do(self, r, out) };
        }
        if !<u8 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedSubImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: isize, out: &mut V) -> bool {
        if CheckedMulFastOp::<u8, isize>::is_supported {
            return unsafe { CheckedMulFastOp::<u8, isize>::Do(self, r, out) };
        }
        if (!<u8 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = if CheckedMulFastOp::<isize, isize>::is_supported {
            unsafe { CheckedMulFastOp::<isize, isize>::Do(self as isize, r as isize, &mut result) }
        } else {
            isize_ops::CheckedMulImpl(self as isize, r as isize, &mut result)
        };
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <isize as Cast<u8>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as isize)
        {
            return false;
        }
        if (!<u8 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as isize) / (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<isize, V> for u8
where
    isize: Cast<V>,
{
    fn modulo(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <isize as Cast<u8>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as isize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as isize) % (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<usize, V> for u8
where
    u8: BuiltinTriple<usize, V>,
    usize: Cast<V>,
    usize: Cast<V>,
    usize: Cast<V>,
{
    fn add(self, r: usize, out: &mut V) -> bool {
        if CheckedAddFastOp::<u8, usize>::is_supported {
            return unsafe { CheckedAddFastOp::<u8, usize>::Do(self, r, out) };
        }
        if !<u8 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedAddImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: usize, out: &mut V) -> bool {
        if CheckedSubFastOp::<u8, usize>::is_supported {
            return unsafe { CheckedSubFastOp::<u8, usize>::Do(self, r, out) };
        }
        if !<u8 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedSubImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: usize, out: &mut V) -> bool {
        if CheckedMulFastOp::<u8, usize>::is_supported {
            return unsafe { CheckedMulFastOp::<u8, usize>::Do(self, r, out) };
        }
        if (!<u8 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = if CheckedMulFastOp::<usize, usize>::is_supported {
            unsafe { CheckedMulFastOp::<usize, usize>::Do(self as usize, r as usize, &mut result) }
        } else {
            usize_ops::CheckedMulImpl(self as usize, r as usize, &mut result)
        };
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<u8>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as usize)
        {
            return false;
        }
        if (!<u8 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as usize) / (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<usize, V> for u8
where
    usize: Cast<V>,
{
    fn modulo(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<u8>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as usize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as usize) % (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i8, V> for i16
where
    i16: BuiltinTriple<i8, V>,
    i32: Cast<V>,
    i32: Cast<V>,
    i16: Cast<V>,
{
    fn add(self, r: i8, out: &mut V) -> bool {
        if CheckedAddFastOp::<i16, i8>::is_supported {
            return unsafe { CheckedAddFastOp::<i16, i8>::Do(self, r, out) };
        }
        if !<i16 as Cast<i32>>::range(self).IsValid() || !<i8 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedAddImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i8, out: &mut V) -> bool {
        if CheckedSubFastOp::<i16, i8>::is_supported {
            return unsafe { CheckedSubFastOp::<i16, i8>::Do(self, r, out) };
        }
        if !<i16 as Cast<i32>>::range(self).IsValid() || !<i8 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedSubImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i8, out: &mut V) -> bool {
        if CheckedMulFastOp::<i16, i8>::is_supported {
            return unsafe { CheckedMulFastOp::<i16, i8>::Do(self, r, out) };
        }
        if (!<i16 as Cast<i32>>::range(self).IsValid() || !<i8 as Cast<i32>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i32;
        let valid = if CheckedMulFastOp::<i32, i32>::is_supported {
            unsafe { CheckedMulFastOp::<i32, i32>::Do(self as i32, r as i32, &mut result) }
        } else {
            i32_ops::CheckedMulImpl(self as i32, r as i32, &mut result)
        };
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i16 as Cast<i16>>::CONTAINED && (self as i16) == i16::MIN && r == (-1i64 as i8)
        {
            return false;
        }
        if (!<i16 as Cast<i16>>::range(self).IsValid() || !<i8 as Cast<i16>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i16) / (r as i16);
        if !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i8, V> for i16
where
    i16: Cast<V>,
{
    fn modulo(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i16 as Cast<i16>>::CONTAINED && (self as i16) == i16::MIN && r == (-1i64 as i8)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i16) % (r as i16);
        if !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u8, V> for i16
where
    i16: BuiltinTriple<u8, V>,
    i32: Cast<V>,
    i32: Cast<V>,
    i16: Cast<V>,
{
    fn add(self, r: u8, out: &mut V) -> bool {
        if CheckedAddFastOp::<i16, u8>::is_supported {
            return unsafe { CheckedAddFastOp::<i16, u8>::Do(self, r, out) };
        }
        if !<i16 as Cast<i32>>::range(self).IsValid() || !<u8 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedAddImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u8, out: &mut V) -> bool {
        if CheckedSubFastOp::<i16, u8>::is_supported {
            return unsafe { CheckedSubFastOp::<i16, u8>::Do(self, r, out) };
        }
        if !<i16 as Cast<i32>>::range(self).IsValid() || !<u8 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedSubImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u8, out: &mut V) -> bool {
        if CheckedMulFastOp::<i16, u8>::is_supported {
            return unsafe { CheckedMulFastOp::<i16, u8>::Do(self, r, out) };
        }
        if (!<i16 as Cast<i32>>::range(self).IsValid() || !<u8 as Cast<i32>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i32;
        let valid = if CheckedMulFastOp::<i32, i32>::is_supported {
            unsafe { CheckedMulFastOp::<i32, i32>::Do(self as i32, r as i32, &mut result) }
        } else {
            i32_ops::CheckedMulImpl(self as i32, r as i32, &mut result)
        };
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i16 as Cast<i16>>::CONTAINED && (self as i16) == i16::MIN && r == (-1i64 as u8)
        {
            return false;
        }
        if (!<i16 as Cast<i16>>::range(self).IsValid() || !<u8 as Cast<i16>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i16) / (r as i16);
        if !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u8, V> for i16
where
    i16: Cast<V>,
{
    fn modulo(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i16 as Cast<i16>>::CONTAINED && (self as i16) == i16::MIN && r == (-1i64 as u8)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i16) % (r as i16);
        if !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i16, V> for i16
where
    i16: BuiltinTriple<i16, V>,
    i32: Cast<V>,
    i32: Cast<V>,
    i16: Cast<V>,
{
    fn add(self, r: i16, out: &mut V) -> bool {
        if CheckedAddFastOp::<i16, i16>::is_supported {
            return unsafe { CheckedAddFastOp::<i16, i16>::Do(self, r, out) };
        }
        if !<i16 as Cast<i32>>::range(self).IsValid() || !<i16 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedAddImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i16, out: &mut V) -> bool {
        if CheckedSubFastOp::<i16, i16>::is_supported {
            return unsafe { CheckedSubFastOp::<i16, i16>::Do(self, r, out) };
        }
        if !<i16 as Cast<i32>>::range(self).IsValid() || !<i16 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedSubImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i16, out: &mut V) -> bool {
        if CheckedMulFastOp::<i16, i16>::is_supported {
            return unsafe { CheckedMulFastOp::<i16, i16>::Do(self, r, out) };
        }
        if (!<i16 as Cast<i32>>::range(self).IsValid() || !<i16 as Cast<i32>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i32;
        let valid = if CheckedMulFastOp::<i32, i32>::is_supported {
            unsafe { CheckedMulFastOp::<i32, i32>::Do(self as i32, r as i32, &mut result) }
        } else {
            i32_ops::CheckedMulImpl(self as i32, r as i32, &mut result)
        };
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i16 as Cast<i16>>::CONTAINED && (self as i16) == i16::MIN && r == (-1i64 as i16)
        {
            return false;
        }
        if (!<i16 as Cast<i16>>::range(self).IsValid() || !<i16 as Cast<i16>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i16) / (r as i16);
        if !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i16, V> for i16
where
    i16: Cast<V>,
{
    fn modulo(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i16 as Cast<i16>>::CONTAINED && (self as i16) == i16::MIN && r == (-1i64 as i16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i16) % (r as i16);
        if !<i16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u16, V> for i16
where
    i16: BuiltinTriple<u16, V>,
    i32: Cast<V>,
    i32: Cast<V>,
    i32: Cast<V>,
{
    fn add(self, r: u16, out: &mut V) -> bool {
        if CheckedAddFastOp::<i16, u16>::is_supported {
            return unsafe { CheckedAddFastOp::<i16, u16>::Do(self, r, out) };
        }
        if !<i16 as Cast<i32>>::range(self).IsValid() || !<u16 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedAddImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u16, out: &mut V) -> bool {
        if CheckedSubFastOp::<i16, u16>::is_supported {
            return unsafe { CheckedSubFastOp::<i16, u16>::Do(self, r, out) };
        }
        if !<i16 as Cast<i32>>::range(self).IsValid() || !<u16 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedSubImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u16, out: &mut V) -> bool {
        if CheckedMulFastOp::<i16, u16>::is_supported {
            return unsafe { CheckedMulFastOp::<i16, u16>::Do(self, r, out) };
        }
        if (!<i16 as Cast<i32>>::range(self).IsValid() || !<u16 as Cast<i32>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i32;
        let valid = if CheckedMulFastOp::<i32, i32>::is_supported {
            unsafe { CheckedMulFastOp::<i32, i32>::Do(self as i32, r as i32, &mut result) }
        } else {
            i32_ops::CheckedMulImpl(self as i32, r as i32, &mut result)
        };
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i32 as Cast<i16>>::CONTAINED
            && (self as i32) == i32::MIN
            && r == (-1i64 as u16)
        {
            return false;
        }
        if (!<i16 as Cast<i32>>::range(self).IsValid() || !<u16 as Cast<i32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i32) / (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u16, V> for i16
where
    i32: Cast<V>,
{
    fn modulo(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i32 as Cast<i16>>::CONTAINED
            && (self as i32) == i32::MIN
            && r == (-1i64 as u16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i32) % (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i32, V> for i16
where
    i16: BuiltinTriple<i32, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i32: Cast<V>,
{
    fn add(self, r: i32, out: &mut V) -> bool {
        if CheckedAddFastOp::<i16, i32>::is_supported {
            return unsafe { CheckedAddFastOp::<i16, i32>::Do(self, r, out) };
        }
        if !<i16 as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i32, out: &mut V) -> bool {
        if CheckedSubFastOp::<i16, i32>::is_supported {
            return unsafe { CheckedSubFastOp::<i16, i32>::Do(self, r, out) };
        }
        if !<i16 as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i32, out: &mut V) -> bool {
        if CheckedMulFastOp::<i16, i32>::is_supported {
            return unsafe { CheckedMulFastOp::<i16, i32>::Do(self, r, out) };
        }
        if (!<i16 as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i32 as Cast<i16>>::CONTAINED && (self as i32) == i32::MIN && r == (-1i64 as i32)
        {
            return false;
        }
        if (!<i16 as Cast<i32>>::range(self).IsValid() || !<i32 as Cast<i32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i32) / (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i32, V> for i16
where
    i32: Cast<V>,
{
    fn modulo(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i32 as Cast<i16>>::CONTAINED && (self as i32) == i32::MIN && r == (-1i64 as i32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i32) % (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u32, V> for i16
where
    i16: BuiltinTriple<u32, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: u32, out: &mut V) -> bool {
        if CheckedAddFastOp::<i16, u32>::is_supported {
            return unsafe { CheckedAddFastOp::<i16, u32>::Do(self, r, out) };
        }
        if !<i16 as Cast<i64>>::range(self).IsValid() || !<u32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u32, out: &mut V) -> bool {
        if CheckedSubFastOp::<i16, u32>::is_supported {
            return unsafe { CheckedSubFastOp::<i16, u32>::Do(self, r, out) };
        }
        if !<i16 as Cast<i64>>::range(self).IsValid() || !<u32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u32, out: &mut V) -> bool {
        if CheckedMulFastOp::<i16, u32>::is_supported {
            return unsafe { CheckedMulFastOp::<i16, u32>::Do(self, r, out) };
        }
        if (!<i16 as Cast<i64>>::range(self).IsValid() || !<u32 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<i16>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as u32)
        {
            return false;
        }
        if (!<i16 as Cast<i64>>::range(self).IsValid() || !<u32 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u32, V> for i16
where
    i64: Cast<V>,
{
    fn modulo(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<i16>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as u32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i64, V> for i16
where
    i16: BuiltinTriple<i64, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: i64, out: &mut V) -> bool {
        if CheckedAddFastOp::<i16, i64>::is_supported {
            return unsafe { CheckedAddFastOp::<i16, i64>::Do(self, r, out) };
        }
        if !<i16 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i64, out: &mut V) -> bool {
        if CheckedSubFastOp::<i16, i64>::is_supported {
            return unsafe { CheckedSubFastOp::<i16, i64>::Do(self, r, out) };
        }
        if !<i16 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i64, out: &mut V) -> bool {
        if CheckedMulFastOp::<i16, i64>::is_supported {
            return unsafe { CheckedMulFastOp::<i16, i64>::Do(self, r, out) };
        }
        if (!<i16 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i64 as Cast<i16>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as i64)
        {
            return false;
        }
        if (!<i16 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i64, V> for i16
where
    i64: Cast<V>,
{
    fn modulo(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i64 as Cast<i16>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as i64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u64, V> for i16
where
    i16: BuiltinTriple<u64, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u64: Cast<V>,
{
    fn add(self, r: u64, out: &mut V) -> bool {
        if CheckedAddFastOp::<i16, u64>::is_supported {
            return unsafe { CheckedAddFastOp::<i16, u64>::Do(self, r, out) };
        }
        if !<i16 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u64, out: &mut V) -> bool {
        if CheckedSubFastOp::<i16, u64>::is_supported {
            return unsafe { CheckedSubFastOp::<i16, u64>::Do(self, r, out) };
        }
        if !<i16 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u64, out: &mut V) -> bool {
        if CheckedMulFastOp::<i16, u64>::is_supported {
            return unsafe { CheckedMulFastOp::<i16, u64>::Do(self, r, out) };
        }
        if (!<i16 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<i16>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u64)
        {
            return false;
        }
        if (!<i16 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u64) / (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u64, V> for i16
where
    u64: Cast<V>,
{
    fn modulo(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<i16>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u64) % (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<isize, V> for i16
where
    i16: BuiltinTriple<isize, V>,
    isize: Cast<V>,
    isize: Cast<V>,
    isize: Cast<V>,
{
    fn add(self, r: isize, out: &mut V) -> bool {
        if CheckedAddFastOp::<i16, isize>::is_supported {
            return unsafe { CheckedAddFastOp::<i16, isize>::Do(self, r, out) };
        }
        if !<i16 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedAddImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: isize, out: &mut V) -> bool {
        if CheckedSubFastOp::<i16, isize>::is_supported {
            return unsafe { CheckedSubFastOp::<i16, isize>::Do(self, r, out) };
        }
        if !<i16 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedSubImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: isize, out: &mut V) -> bool {
        if CheckedMulFastOp::<i16, isize>::is_supported {
            return unsafe { CheckedMulFastOp::<i16, isize>::Do(self, r, out) };
        }
        if (!<i16 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = if CheckedMulFastOp::<isize, isize>::is_supported {
            unsafe { CheckedMulFastOp::<isize, isize>::Do(self as isize, r as isize, &mut result) }
        } else {
            isize_ops::CheckedMulImpl(self as isize, r as isize, &mut result)
        };
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true
            && <isize as Cast<i16>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as isize)
        {
            return false;
        }
        if (!<i16 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as isize) / (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<isize, V> for i16
where
    isize: Cast<V>,
{
    fn modulo(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true
            && <isize as Cast<i16>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as isize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as isize) % (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<usize, V> for i16
where
    i16: BuiltinTriple<usize, V>,
    usize: Cast<V>,
    usize: Cast<V>,
    usize: Cast<V>,
{
    fn add(self, r: usize, out: &mut V) -> bool {
        if CheckedAddFastOp::<i16, usize>::is_supported {
            return unsafe { CheckedAddFastOp::<i16, usize>::Do(self, r, out) };
        }
        if !<i16 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedAddImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: usize, out: &mut V) -> bool {
        if CheckedSubFastOp::<i16, usize>::is_supported {
            return unsafe { CheckedSubFastOp::<i16, usize>::Do(self, r, out) };
        }
        if !<i16 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedSubImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: usize, out: &mut V) -> bool {
        if CheckedMulFastOp::<i16, usize>::is_supported {
            return unsafe { CheckedMulFastOp::<i16, usize>::Do(self, r, out) };
        }
        if (!<i16 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = if CheckedMulFastOp::<usize, usize>::is_supported {
            unsafe { CheckedMulFastOp::<usize, usize>::Do(self as usize, r as usize, &mut result) }
        } else {
            usize_ops::CheckedMulImpl(self as usize, r as usize, &mut result)
        };
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<i16>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as usize)
        {
            return false;
        }
        if (!<i16 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as usize) / (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<usize, V> for i16
where
    usize: Cast<V>,
{
    fn modulo(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<i16>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as usize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as usize) % (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i8, V> for u16
where
    u16: BuiltinTriple<i8, V>,
    i32: Cast<V>,
    i32: Cast<V>,
    i32: Cast<V>,
{
    fn add(self, r: i8, out: &mut V) -> bool {
        if CheckedAddFastOp::<u16, i8>::is_supported {
            return unsafe { CheckedAddFastOp::<u16, i8>::Do(self, r, out) };
        }
        if !<u16 as Cast<i32>>::range(self).IsValid() || !<i8 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedAddImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i8, out: &mut V) -> bool {
        if CheckedSubFastOp::<u16, i8>::is_supported {
            return unsafe { CheckedSubFastOp::<u16, i8>::Do(self, r, out) };
        }
        if !<u16 as Cast<i32>>::range(self).IsValid() || !<i8 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedSubImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i8, out: &mut V) -> bool {
        if CheckedMulFastOp::<u16, i8>::is_supported {
            return unsafe { CheckedMulFastOp::<u16, i8>::Do(self, r, out) };
        }
        if (!<u16 as Cast<i32>>::range(self).IsValid() || !<i8 as Cast<i32>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i32;
        let valid = if CheckedMulFastOp::<i32, i32>::is_supported {
            unsafe { CheckedMulFastOp::<i32, i32>::Do(self as i32, r as i32, &mut result) }
        } else {
            i32_ops::CheckedMulImpl(self as i32, r as i32, &mut result)
        };
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i32 as Cast<u16>>::CONTAINED && (self as i32) == i32::MIN && r == (-1i64 as i8)
        {
            return false;
        }
        if (!<u16 as Cast<i32>>::range(self).IsValid() || !<i8 as Cast<i32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i32) / (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i8, V> for u16
where
    i32: Cast<V>,
{
    fn modulo(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i32 as Cast<u16>>::CONTAINED && (self as i32) == i32::MIN && r == (-1i64 as i8)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i32) % (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u8, V> for u16
where
    u16: BuiltinTriple<u8, V>,
    u32: Cast<V>,
    u32: Cast<V>,
    u16: Cast<V>,
{
    fn add(self, r: u8, out: &mut V) -> bool {
        if CheckedAddFastOp::<u16, u8>::is_supported {
            return unsafe { CheckedAddFastOp::<u16, u8>::Do(self, r, out) };
        }
        if !<u16 as Cast<u32>>::range(self).IsValid() || !<u8 as Cast<u32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u32;
        let valid = u32_ops::CheckedAddImpl(self as u32, r as u32, &mut result);
        if !valid || !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u8, out: &mut V) -> bool {
        if CheckedSubFastOp::<u16, u8>::is_supported {
            return unsafe { CheckedSubFastOp::<u16, u8>::Do(self, r, out) };
        }
        if !<u16 as Cast<u32>>::range(self).IsValid() || !<u8 as Cast<u32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u32;
        let valid = u32_ops::CheckedSubImpl(self as u32, r as u32, &mut result);
        if !valid || !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u8, out: &mut V) -> bool {
        if CheckedMulFastOp::<u16, u8>::is_supported {
            return unsafe { CheckedMulFastOp::<u16, u8>::Do(self, r, out) };
        }
        if (!<u16 as Cast<u32>>::range(self).IsValid() || !<u8 as Cast<u32>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u32;
        let valid = if CheckedMulFastOp::<u32, u32>::is_supported {
            unsafe { CheckedMulFastOp::<u32, u32>::Do(self as u32, r as u32, &mut result) }
        } else {
            u32_ops::CheckedMulImpl(self as u32, r as u32, &mut result)
        };
        if !valid || !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <u16 as Cast<u16>>::CONTAINED && (self as u16) == u16::MIN && r == (-1i64 as u8)
        {
            return false;
        }
        if (!<u16 as Cast<u16>>::range(self).IsValid() || !<u8 as Cast<u16>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u16) / (r as u16);
        if !<u16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u8, V> for u16
where
    u16: Cast<V>,
{
    fn modulo(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <u16 as Cast<u16>>::CONTAINED && (self as u16) == u16::MIN && r == (-1i64 as u8)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u16) % (r as u16);
        if !<u16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i16, V> for u16
where
    u16: BuiltinTriple<i16, V>,
    i32: Cast<V>,
    i32: Cast<V>,
    i32: Cast<V>,
{
    fn add(self, r: i16, out: &mut V) -> bool {
        if CheckedAddFastOp::<u16, i16>::is_supported {
            return unsafe { CheckedAddFastOp::<u16, i16>::Do(self, r, out) };
        }
        if !<u16 as Cast<i32>>::range(self).IsValid() || !<i16 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedAddImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i16, out: &mut V) -> bool {
        if CheckedSubFastOp::<u16, i16>::is_supported {
            return unsafe { CheckedSubFastOp::<u16, i16>::Do(self, r, out) };
        }
        if !<u16 as Cast<i32>>::range(self).IsValid() || !<i16 as Cast<i32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i32;
        let valid = i32_ops::CheckedSubImpl(self as i32, r as i32, &mut result);
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i16, out: &mut V) -> bool {
        if CheckedMulFastOp::<u16, i16>::is_supported {
            return unsafe { CheckedMulFastOp::<u16, i16>::Do(self, r, out) };
        }
        if (!<u16 as Cast<i32>>::range(self).IsValid() || !<i16 as Cast<i32>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i32;
        let valid = if CheckedMulFastOp::<i32, i32>::is_supported {
            unsafe { CheckedMulFastOp::<i32, i32>::Do(self as i32, r as i32, &mut result) }
        } else {
            i32_ops::CheckedMulImpl(self as i32, r as i32, &mut result)
        };
        if !valid || !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i32 as Cast<u16>>::CONTAINED
            && (self as i32) == i32::MIN
            && r == (-1i64 as i16)
        {
            return false;
        }
        if (!<u16 as Cast<i32>>::range(self).IsValid() || !<i16 as Cast<i32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i32) / (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i16, V> for u16
where
    i32: Cast<V>,
{
    fn modulo(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i32 as Cast<u16>>::CONTAINED
            && (self as i32) == i32::MIN
            && r == (-1i64 as i16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i32) % (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u16, V> for u16
where
    u16: BuiltinTriple<u16, V>,
    u32: Cast<V>,
    u32: Cast<V>,
    u16: Cast<V>,
{
    fn add(self, r: u16, out: &mut V) -> bool {
        if CheckedAddFastOp::<u16, u16>::is_supported {
            return unsafe { CheckedAddFastOp::<u16, u16>::Do(self, r, out) };
        }
        if !<u16 as Cast<u32>>::range(self).IsValid() || !<u16 as Cast<u32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u32;
        let valid = u32_ops::CheckedAddImpl(self as u32, r as u32, &mut result);
        if !valid || !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u16, out: &mut V) -> bool {
        if CheckedSubFastOp::<u16, u16>::is_supported {
            return unsafe { CheckedSubFastOp::<u16, u16>::Do(self, r, out) };
        }
        if !<u16 as Cast<u32>>::range(self).IsValid() || !<u16 as Cast<u32>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u32;
        let valid = u32_ops::CheckedSubImpl(self as u32, r as u32, &mut result);
        if !valid || !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u16, out: &mut V) -> bool {
        if CheckedMulFastOp::<u16, u16>::is_supported {
            return unsafe { CheckedMulFastOp::<u16, u16>::Do(self, r, out) };
        }
        if (!<u16 as Cast<u32>>::range(self).IsValid() || !<u16 as Cast<u32>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u32;
        let valid = if CheckedMulFastOp::<u32, u32>::is_supported {
            unsafe { CheckedMulFastOp::<u32, u32>::Do(self as u32, r as u32, &mut result) }
        } else {
            u32_ops::CheckedMulImpl(self as u32, r as u32, &mut result)
        };
        if !valid || !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u16 as Cast<u16>>::CONTAINED
            && (self as u16) == u16::MIN
            && r == (-1i64 as u16)
        {
            return false;
        }
        if (!<u16 as Cast<u16>>::range(self).IsValid() || !<u16 as Cast<u16>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u16) / (r as u16);
        if !<u16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u16, V> for u16
where
    u16: Cast<V>,
{
    fn modulo(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u16 as Cast<u16>>::CONTAINED
            && (self as u16) == u16::MIN
            && r == (-1i64 as u16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u16) % (r as u16);
        if !<u16 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u16 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i32, V> for u16
where
    u16: BuiltinTriple<i32, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i32: Cast<V>,
{
    fn add(self, r: i32, out: &mut V) -> bool {
        if CheckedAddFastOp::<u16, i32>::is_supported {
            return unsafe { CheckedAddFastOp::<u16, i32>::Do(self, r, out) };
        }
        if !<u16 as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i32, out: &mut V) -> bool {
        if CheckedSubFastOp::<u16, i32>::is_supported {
            return unsafe { CheckedSubFastOp::<u16, i32>::Do(self, r, out) };
        }
        if !<u16 as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i32, out: &mut V) -> bool {
        if CheckedMulFastOp::<u16, i32>::is_supported {
            return unsafe { CheckedMulFastOp::<u16, i32>::Do(self, r, out) };
        }
        if (!<u16 as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i32 as Cast<u16>>::CONTAINED
            && (self as i32) == i32::MIN
            && r == (-1i64 as i32)
        {
            return false;
        }
        if (!<u16 as Cast<i32>>::range(self).IsValid() || !<i32 as Cast<i32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i32) / (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i32, V> for u16
where
    i32: Cast<V>,
{
    fn modulo(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i32 as Cast<u16>>::CONTAINED
            && (self as i32) == i32::MIN
            && r == (-1i64 as i32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i32) % (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u32, V> for u16
where
    u16: BuiltinTriple<u32, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u32: Cast<V>,
{
    fn add(self, r: u32, out: &mut V) -> bool {
        if CheckedAddFastOp::<u16, u32>::is_supported {
            return unsafe { CheckedAddFastOp::<u16, u32>::Do(self, r, out) };
        }
        if !<u16 as Cast<u64>>::range(self).IsValid() || !<u32 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u32, out: &mut V) -> bool {
        if CheckedSubFastOp::<u16, u32>::is_supported {
            return unsafe { CheckedSubFastOp::<u16, u32>::Do(self, r, out) };
        }
        if !<u16 as Cast<u64>>::range(self).IsValid() || !<u32 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u32, out: &mut V) -> bool {
        if CheckedMulFastOp::<u16, u32>::is_supported {
            return unsafe { CheckedMulFastOp::<u16, u32>::Do(self, r, out) };
        }
        if (!<u16 as Cast<u64>>::range(self).IsValid() || !<u32 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u32 as Cast<u16>>::CONTAINED
            && (self as u32) == u32::MIN
            && r == (-1i64 as u32)
        {
            return false;
        }
        if (!<u16 as Cast<u32>>::range(self).IsValid() || !<u32 as Cast<u32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u32) / (r as u32);
        if !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u32, V> for u16
where
    u32: Cast<V>,
{
    fn modulo(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u32 as Cast<u16>>::CONTAINED
            && (self as u32) == u32::MIN
            && r == (-1i64 as u32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u32) % (r as u32);
        if !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i64, V> for u16
where
    u16: BuiltinTriple<i64, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: i64, out: &mut V) -> bool {
        if CheckedAddFastOp::<u16, i64>::is_supported {
            return unsafe { CheckedAddFastOp::<u16, i64>::Do(self, r, out) };
        }
        if !<u16 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i64, out: &mut V) -> bool {
        if CheckedSubFastOp::<u16, i64>::is_supported {
            return unsafe { CheckedSubFastOp::<u16, i64>::Do(self, r, out) };
        }
        if !<u16 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i64, out: &mut V) -> bool {
        if CheckedMulFastOp::<u16, i64>::is_supported {
            return unsafe { CheckedMulFastOp::<u16, i64>::Do(self, r, out) };
        }
        if (!<u16 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<u16>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as i64)
        {
            return false;
        }
        if (!<u16 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i64, V> for u16
where
    i64: Cast<V>,
{
    fn modulo(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<u16>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as i64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u64, V> for u16
where
    u16: BuiltinTriple<u64, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u64: Cast<V>,
{
    fn add(self, r: u64, out: &mut V) -> bool {
        if CheckedAddFastOp::<u16, u64>::is_supported {
            return unsafe { CheckedAddFastOp::<u16, u64>::Do(self, r, out) };
        }
        if !<u16 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u64, out: &mut V) -> bool {
        if CheckedSubFastOp::<u16, u64>::is_supported {
            return unsafe { CheckedSubFastOp::<u16, u64>::Do(self, r, out) };
        }
        if !<u16 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u64, out: &mut V) -> bool {
        if CheckedMulFastOp::<u16, u64>::is_supported {
            return unsafe { CheckedMulFastOp::<u16, u64>::Do(self, r, out) };
        }
        if (!<u16 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u16>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u64)
        {
            return false;
        }
        if (!<u16 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u64) / (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u64, V> for u16
where
    u64: Cast<V>,
{
    fn modulo(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u16>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u64) % (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<isize, V> for u16
where
    u16: BuiltinTriple<isize, V>,
    isize: Cast<V>,
    isize: Cast<V>,
    isize: Cast<V>,
{
    fn add(self, r: isize, out: &mut V) -> bool {
        if CheckedAddFastOp::<u16, isize>::is_supported {
            return unsafe { CheckedAddFastOp::<u16, isize>::Do(self, r, out) };
        }
        if !<u16 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedAddImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: isize, out: &mut V) -> bool {
        if CheckedSubFastOp::<u16, isize>::is_supported {
            return unsafe { CheckedSubFastOp::<u16, isize>::Do(self, r, out) };
        }
        if !<u16 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedSubImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: isize, out: &mut V) -> bool {
        if CheckedMulFastOp::<u16, isize>::is_supported {
            return unsafe { CheckedMulFastOp::<u16, isize>::Do(self, r, out) };
        }
        if (!<u16 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = if CheckedMulFastOp::<isize, isize>::is_supported {
            unsafe { CheckedMulFastOp::<isize, isize>::Do(self as isize, r as isize, &mut result) }
        } else {
            isize_ops::CheckedMulImpl(self as isize, r as isize, &mut result)
        };
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <isize as Cast<u16>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as isize)
        {
            return false;
        }
        if (!<u16 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as isize) / (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<isize, V> for u16
where
    isize: Cast<V>,
{
    fn modulo(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <isize as Cast<u16>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as isize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as isize) % (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<usize, V> for u16
where
    u16: BuiltinTriple<usize, V>,
    usize: Cast<V>,
    usize: Cast<V>,
    usize: Cast<V>,
{
    fn add(self, r: usize, out: &mut V) -> bool {
        if CheckedAddFastOp::<u16, usize>::is_supported {
            return unsafe { CheckedAddFastOp::<u16, usize>::Do(self, r, out) };
        }
        if !<u16 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedAddImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: usize, out: &mut V) -> bool {
        if CheckedSubFastOp::<u16, usize>::is_supported {
            return unsafe { CheckedSubFastOp::<u16, usize>::Do(self, r, out) };
        }
        if !<u16 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedSubImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: usize, out: &mut V) -> bool {
        if CheckedMulFastOp::<u16, usize>::is_supported {
            return unsafe { CheckedMulFastOp::<u16, usize>::Do(self, r, out) };
        }
        if (!<u16 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = if CheckedMulFastOp::<usize, usize>::is_supported {
            unsafe { CheckedMulFastOp::<usize, usize>::Do(self as usize, r as usize, &mut result) }
        } else {
            usize_ops::CheckedMulImpl(self as usize, r as usize, &mut result)
        };
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<u16>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as usize)
        {
            return false;
        }
        if (!<u16 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as usize) / (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<usize, V> for u16
where
    usize: Cast<V>,
{
    fn modulo(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<u16>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as usize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as usize) % (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i8, V> for i32
where
    i32: BuiltinTriple<i8, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i32: Cast<V>,
{
    fn add(self, r: i8, out: &mut V) -> bool {
        if CheckedAddFastOp::<i32, i8>::is_supported {
            return unsafe { CheckedAddFastOp::<i32, i8>::Do(self, r, out) };
        }
        if !<i32 as Cast<i64>>::range(self).IsValid() || !<i8 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i8, out: &mut V) -> bool {
        if CheckedSubFastOp::<i32, i8>::is_supported {
            return unsafe { CheckedSubFastOp::<i32, i8>::Do(self, r, out) };
        }
        if !<i32 as Cast<i64>>::range(self).IsValid() || !<i8 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i8, out: &mut V) -> bool {
        if CheckedMulFastOp::<i32, i8>::is_supported {
            return unsafe { CheckedMulFastOp::<i32, i8>::Do(self, r, out) };
        }
        if (!<i32 as Cast<i64>>::range(self).IsValid() || !<i8 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i32 as Cast<i32>>::CONTAINED && (self as i32) == i32::MIN && r == (-1i64 as i8)
        {
            return false;
        }
        if (!<i32 as Cast<i32>>::range(self).IsValid() || !<i8 as Cast<i32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i32) / (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i8, V> for i32
where
    i32: Cast<V>,
{
    fn modulo(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i32 as Cast<i32>>::CONTAINED && (self as i32) == i32::MIN && r == (-1i64 as i8)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i32) % (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u8, V> for i32
where
    i32: BuiltinTriple<u8, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i32: Cast<V>,
{
    fn add(self, r: u8, out: &mut V) -> bool {
        if CheckedAddFastOp::<i32, u8>::is_supported {
            return unsafe { CheckedAddFastOp::<i32, u8>::Do(self, r, out) };
        }
        if !<i32 as Cast<i64>>::range(self).IsValid() || !<u8 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u8, out: &mut V) -> bool {
        if CheckedSubFastOp::<i32, u8>::is_supported {
            return unsafe { CheckedSubFastOp::<i32, u8>::Do(self, r, out) };
        }
        if !<i32 as Cast<i64>>::range(self).IsValid() || !<u8 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u8, out: &mut V) -> bool {
        if CheckedMulFastOp::<i32, u8>::is_supported {
            return unsafe { CheckedMulFastOp::<i32, u8>::Do(self, r, out) };
        }
        if (!<i32 as Cast<i64>>::range(self).IsValid() || !<u8 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i32 as Cast<i32>>::CONTAINED && (self as i32) == i32::MIN && r == (-1i64 as u8)
        {
            return false;
        }
        if (!<i32 as Cast<i32>>::range(self).IsValid() || !<u8 as Cast<i32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i32) / (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u8, V> for i32
where
    i32: Cast<V>,
{
    fn modulo(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i32 as Cast<i32>>::CONTAINED && (self as i32) == i32::MIN && r == (-1i64 as u8)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i32) % (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i16, V> for i32
where
    i32: BuiltinTriple<i16, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i32: Cast<V>,
{
    fn add(self, r: i16, out: &mut V) -> bool {
        if CheckedAddFastOp::<i32, i16>::is_supported {
            return unsafe { CheckedAddFastOp::<i32, i16>::Do(self, r, out) };
        }
        if !<i32 as Cast<i64>>::range(self).IsValid() || !<i16 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i16, out: &mut V) -> bool {
        if CheckedSubFastOp::<i32, i16>::is_supported {
            return unsafe { CheckedSubFastOp::<i32, i16>::Do(self, r, out) };
        }
        if !<i32 as Cast<i64>>::range(self).IsValid() || !<i16 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i16, out: &mut V) -> bool {
        if CheckedMulFastOp::<i32, i16>::is_supported {
            return unsafe { CheckedMulFastOp::<i32, i16>::Do(self, r, out) };
        }
        if (!<i32 as Cast<i64>>::range(self).IsValid() || !<i16 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i32 as Cast<i32>>::CONTAINED && (self as i32) == i32::MIN && r == (-1i64 as i16)
        {
            return false;
        }
        if (!<i32 as Cast<i32>>::range(self).IsValid() || !<i16 as Cast<i32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i32) / (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i16, V> for i32
where
    i32: Cast<V>,
{
    fn modulo(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i32 as Cast<i32>>::CONTAINED && (self as i32) == i32::MIN && r == (-1i64 as i16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i32) % (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u16, V> for i32
where
    i32: BuiltinTriple<u16, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i32: Cast<V>,
{
    fn add(self, r: u16, out: &mut V) -> bool {
        if CheckedAddFastOp::<i32, u16>::is_supported {
            return unsafe { CheckedAddFastOp::<i32, u16>::Do(self, r, out) };
        }
        if !<i32 as Cast<i64>>::range(self).IsValid() || !<u16 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u16, out: &mut V) -> bool {
        if CheckedSubFastOp::<i32, u16>::is_supported {
            return unsafe { CheckedSubFastOp::<i32, u16>::Do(self, r, out) };
        }
        if !<i32 as Cast<i64>>::range(self).IsValid() || !<u16 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u16, out: &mut V) -> bool {
        if CheckedMulFastOp::<i32, u16>::is_supported {
            return unsafe { CheckedMulFastOp::<i32, u16>::Do(self, r, out) };
        }
        if (!<i32 as Cast<i64>>::range(self).IsValid() || !<u16 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i32 as Cast<i32>>::CONTAINED
            && (self as i32) == i32::MIN
            && r == (-1i64 as u16)
        {
            return false;
        }
        if (!<i32 as Cast<i32>>::range(self).IsValid() || !<u16 as Cast<i32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i32) / (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u16, V> for i32
where
    i32: Cast<V>,
{
    fn modulo(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i32 as Cast<i32>>::CONTAINED
            && (self as i32) == i32::MIN
            && r == (-1i64 as u16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i32) % (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i32, V> for i32
where
    i32: BuiltinTriple<i32, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i32: Cast<V>,
{
    fn add(self, r: i32, out: &mut V) -> bool {
        if CheckedAddFastOp::<i32, i32>::is_supported {
            return unsafe { CheckedAddFastOp::<i32, i32>::Do(self, r, out) };
        }
        if !<i32 as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i32, out: &mut V) -> bool {
        if CheckedSubFastOp::<i32, i32>::is_supported {
            return unsafe { CheckedSubFastOp::<i32, i32>::Do(self, r, out) };
        }
        if !<i32 as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i32, out: &mut V) -> bool {
        if CheckedMulFastOp::<i32, i32>::is_supported {
            return unsafe { CheckedMulFastOp::<i32, i32>::Do(self, r, out) };
        }
        if (!<i32 as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i32 as Cast<i32>>::CONTAINED && (self as i32) == i32::MIN && r == (-1i64 as i32)
        {
            return false;
        }
        if (!<i32 as Cast<i32>>::range(self).IsValid() || !<i32 as Cast<i32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i32) / (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i32, V> for i32
where
    i32: Cast<V>,
{
    fn modulo(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i32 as Cast<i32>>::CONTAINED && (self as i32) == i32::MIN && r == (-1i64 as i32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i32) % (r as i32);
        if !<i32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u32, V> for i32
where
    i32: BuiltinTriple<u32, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: u32, out: &mut V) -> bool {
        if CheckedAddFastOp::<i32, u32>::is_supported {
            return unsafe { CheckedAddFastOp::<i32, u32>::Do(self, r, out) };
        }
        if !<i32 as Cast<i64>>::range(self).IsValid() || !<u32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u32, out: &mut V) -> bool {
        if CheckedSubFastOp::<i32, u32>::is_supported {
            return unsafe { CheckedSubFastOp::<i32, u32>::Do(self, r, out) };
        }
        if !<i32 as Cast<i64>>::range(self).IsValid() || !<u32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u32, out: &mut V) -> bool {
        if CheckedMulFastOp::<i32, u32>::is_supported {
            return unsafe { CheckedMulFastOp::<i32, u32>::Do(self, r, out) };
        }
        if (!<i32 as Cast<i64>>::range(self).IsValid() || !<u32 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<i32>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as u32)
        {
            return false;
        }
        if (!<i32 as Cast<i64>>::range(self).IsValid() || !<u32 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u32, V> for i32
where
    i64: Cast<V>,
{
    fn modulo(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<i32>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as u32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i64, V> for i32
where
    i32: BuiltinTriple<i64, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: i64, out: &mut V) -> bool {
        if CheckedAddFastOp::<i32, i64>::is_supported {
            return unsafe { CheckedAddFastOp::<i32, i64>::Do(self, r, out) };
        }
        if !<i32 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i64, out: &mut V) -> bool {
        if CheckedSubFastOp::<i32, i64>::is_supported {
            return unsafe { CheckedSubFastOp::<i32, i64>::Do(self, r, out) };
        }
        if !<i32 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i64, out: &mut V) -> bool {
        if CheckedMulFastOp::<i32, i64>::is_supported {
            return unsafe { CheckedMulFastOp::<i32, i64>::Do(self, r, out) };
        }
        if (!<i32 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i64 as Cast<i32>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as i64)
        {
            return false;
        }
        if (!<i32 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i64, V> for i32
where
    i64: Cast<V>,
{
    fn modulo(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i64 as Cast<i32>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as i64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u64, V> for i32
where
    i32: BuiltinTriple<u64, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u64: Cast<V>,
{
    fn add(self, r: u64, out: &mut V) -> bool {
        if CheckedAddFastOp::<i32, u64>::is_supported {
            return unsafe { CheckedAddFastOp::<i32, u64>::Do(self, r, out) };
        }
        if !<i32 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u64, out: &mut V) -> bool {
        if CheckedSubFastOp::<i32, u64>::is_supported {
            return unsafe { CheckedSubFastOp::<i32, u64>::Do(self, r, out) };
        }
        if !<i32 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u64, out: &mut V) -> bool {
        if CheckedMulFastOp::<i32, u64>::is_supported {
            return unsafe { CheckedMulFastOp::<i32, u64>::Do(self, r, out) };
        }
        if (!<i32 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<i32>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u64)
        {
            return false;
        }
        if (!<i32 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u64) / (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u64, V> for i32
where
    u64: Cast<V>,
{
    fn modulo(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<i32>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u64) % (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<isize, V> for i32
where
    i32: BuiltinTriple<isize, V>,
    isize: Cast<V>,
    isize: Cast<V>,
    isize: Cast<V>,
{
    fn add(self, r: isize, out: &mut V) -> bool {
        if CheckedAddFastOp::<i32, isize>::is_supported {
            return unsafe { CheckedAddFastOp::<i32, isize>::Do(self, r, out) };
        }
        if !<i32 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedAddImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: isize, out: &mut V) -> bool {
        if CheckedSubFastOp::<i32, isize>::is_supported {
            return unsafe { CheckedSubFastOp::<i32, isize>::Do(self, r, out) };
        }
        if !<i32 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedSubImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: isize, out: &mut V) -> bool {
        if CheckedMulFastOp::<i32, isize>::is_supported {
            return unsafe { CheckedMulFastOp::<i32, isize>::Do(self, r, out) };
        }
        if (!<i32 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = if CheckedMulFastOp::<isize, isize>::is_supported {
            unsafe { CheckedMulFastOp::<isize, isize>::Do(self as isize, r as isize, &mut result) }
        } else {
            isize_ops::CheckedMulImpl(self as isize, r as isize, &mut result)
        };
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true
            && <isize as Cast<i32>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as isize)
        {
            return false;
        }
        if (!<i32 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as isize) / (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<isize, V> for i32
where
    isize: Cast<V>,
{
    fn modulo(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true
            && <isize as Cast<i32>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as isize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as isize) % (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<usize, V> for i32
where
    i32: BuiltinTriple<usize, V>,
    usize: Cast<V>,
    usize: Cast<V>,
    usize: Cast<V>,
{
    fn add(self, r: usize, out: &mut V) -> bool {
        if CheckedAddFastOp::<i32, usize>::is_supported {
            return unsafe { CheckedAddFastOp::<i32, usize>::Do(self, r, out) };
        }
        if !<i32 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedAddImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: usize, out: &mut V) -> bool {
        if CheckedSubFastOp::<i32, usize>::is_supported {
            return unsafe { CheckedSubFastOp::<i32, usize>::Do(self, r, out) };
        }
        if !<i32 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedSubImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: usize, out: &mut V) -> bool {
        if CheckedMulFastOp::<i32, usize>::is_supported {
            return unsafe { CheckedMulFastOp::<i32, usize>::Do(self, r, out) };
        }
        if (!<i32 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = if CheckedMulFastOp::<usize, usize>::is_supported {
            unsafe { CheckedMulFastOp::<usize, usize>::Do(self as usize, r as usize, &mut result) }
        } else {
            usize_ops::CheckedMulImpl(self as usize, r as usize, &mut result)
        };
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<i32>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as usize)
        {
            return false;
        }
        if (!<i32 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as usize) / (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<usize, V> for i32
where
    usize: Cast<V>,
{
    fn modulo(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<i32>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as usize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as usize) % (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i8, V> for u32
where
    u32: BuiltinTriple<i8, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: i8, out: &mut V) -> bool {
        if CheckedAddFastOp::<u32, i8>::is_supported {
            return unsafe { CheckedAddFastOp::<u32, i8>::Do(self, r, out) };
        }
        if !<u32 as Cast<i64>>::range(self).IsValid() || !<i8 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i8, out: &mut V) -> bool {
        if CheckedSubFastOp::<u32, i8>::is_supported {
            return unsafe { CheckedSubFastOp::<u32, i8>::Do(self, r, out) };
        }
        if !<u32 as Cast<i64>>::range(self).IsValid() || !<i8 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i8, out: &mut V) -> bool {
        if CheckedMulFastOp::<u32, i8>::is_supported {
            return unsafe { CheckedMulFastOp::<u32, i8>::Do(self, r, out) };
        }
        if (!<u32 as Cast<i64>>::range(self).IsValid() || !<i8 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i64 as Cast<u32>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as i8)
        {
            return false;
        }
        if (!<u32 as Cast<i64>>::range(self).IsValid() || !<i8 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i8, V> for u32
where
    i64: Cast<V>,
{
    fn modulo(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i64 as Cast<u32>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as i8)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u8, V> for u32
where
    u32: BuiltinTriple<u8, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u32: Cast<V>,
{
    fn add(self, r: u8, out: &mut V) -> bool {
        if CheckedAddFastOp::<u32, u8>::is_supported {
            return unsafe { CheckedAddFastOp::<u32, u8>::Do(self, r, out) };
        }
        if !<u32 as Cast<u64>>::range(self).IsValid() || !<u8 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u8, out: &mut V) -> bool {
        if CheckedSubFastOp::<u32, u8>::is_supported {
            return unsafe { CheckedSubFastOp::<u32, u8>::Do(self, r, out) };
        }
        if !<u32 as Cast<u64>>::range(self).IsValid() || !<u8 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u8, out: &mut V) -> bool {
        if CheckedMulFastOp::<u32, u8>::is_supported {
            return unsafe { CheckedMulFastOp::<u32, u8>::Do(self, r, out) };
        }
        if (!<u32 as Cast<u64>>::range(self).IsValid() || !<u8 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <u32 as Cast<u32>>::CONTAINED && (self as u32) == u32::MIN && r == (-1i64 as u8)
        {
            return false;
        }
        if (!<u32 as Cast<u32>>::range(self).IsValid() || !<u8 as Cast<u32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u32) / (r as u32);
        if !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u8, V> for u32
where
    u32: Cast<V>,
{
    fn modulo(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <u32 as Cast<u32>>::CONTAINED && (self as u32) == u32::MIN && r == (-1i64 as u8)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u32) % (r as u32);
        if !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i16, V> for u32
where
    u32: BuiltinTriple<i16, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: i16, out: &mut V) -> bool {
        if CheckedAddFastOp::<u32, i16>::is_supported {
            return unsafe { CheckedAddFastOp::<u32, i16>::Do(self, r, out) };
        }
        if !<u32 as Cast<i64>>::range(self).IsValid() || !<i16 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i16, out: &mut V) -> bool {
        if CheckedSubFastOp::<u32, i16>::is_supported {
            return unsafe { CheckedSubFastOp::<u32, i16>::Do(self, r, out) };
        }
        if !<u32 as Cast<i64>>::range(self).IsValid() || !<i16 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i16, out: &mut V) -> bool {
        if CheckedMulFastOp::<u32, i16>::is_supported {
            return unsafe { CheckedMulFastOp::<u32, i16>::Do(self, r, out) };
        }
        if (!<u32 as Cast<i64>>::range(self).IsValid() || !<i16 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<u32>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as i16)
        {
            return false;
        }
        if (!<u32 as Cast<i64>>::range(self).IsValid() || !<i16 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i16, V> for u32
where
    i64: Cast<V>,
{
    fn modulo(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<u32>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as i16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u16, V> for u32
where
    u32: BuiltinTriple<u16, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u32: Cast<V>,
{
    fn add(self, r: u16, out: &mut V) -> bool {
        if CheckedAddFastOp::<u32, u16>::is_supported {
            return unsafe { CheckedAddFastOp::<u32, u16>::Do(self, r, out) };
        }
        if !<u32 as Cast<u64>>::range(self).IsValid() || !<u16 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u16, out: &mut V) -> bool {
        if CheckedSubFastOp::<u32, u16>::is_supported {
            return unsafe { CheckedSubFastOp::<u32, u16>::Do(self, r, out) };
        }
        if !<u32 as Cast<u64>>::range(self).IsValid() || !<u16 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u16, out: &mut V) -> bool {
        if CheckedMulFastOp::<u32, u16>::is_supported {
            return unsafe { CheckedMulFastOp::<u32, u16>::Do(self, r, out) };
        }
        if (!<u32 as Cast<u64>>::range(self).IsValid() || !<u16 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u32 as Cast<u32>>::CONTAINED
            && (self as u32) == u32::MIN
            && r == (-1i64 as u16)
        {
            return false;
        }
        if (!<u32 as Cast<u32>>::range(self).IsValid() || !<u16 as Cast<u32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u32) / (r as u32);
        if !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u16, V> for u32
where
    u32: Cast<V>,
{
    fn modulo(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u32 as Cast<u32>>::CONTAINED
            && (self as u32) == u32::MIN
            && r == (-1i64 as u16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u32) % (r as u32);
        if !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i32, V> for u32
where
    u32: BuiltinTriple<i32, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: i32, out: &mut V) -> bool {
        if CheckedAddFastOp::<u32, i32>::is_supported {
            return unsafe { CheckedAddFastOp::<u32, i32>::Do(self, r, out) };
        }
        if !<u32 as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i32, out: &mut V) -> bool {
        if CheckedSubFastOp::<u32, i32>::is_supported {
            return unsafe { CheckedSubFastOp::<u32, i32>::Do(self, r, out) };
        }
        if !<u32 as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i32, out: &mut V) -> bool {
        if CheckedMulFastOp::<u32, i32>::is_supported {
            return unsafe { CheckedMulFastOp::<u32, i32>::Do(self, r, out) };
        }
        if (!<u32 as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<u32>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as i32)
        {
            return false;
        }
        if (!<u32 as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i32, V> for u32
where
    i64: Cast<V>,
{
    fn modulo(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<u32>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as i32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u32, V> for u32
where
    u32: BuiltinTriple<u32, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u32: Cast<V>,
{
    fn add(self, r: u32, out: &mut V) -> bool {
        if CheckedAddFastOp::<u32, u32>::is_supported {
            return unsafe { CheckedAddFastOp::<u32, u32>::Do(self, r, out) };
        }
        if !<u32 as Cast<u64>>::range(self).IsValid() || !<u32 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u32, out: &mut V) -> bool {
        if CheckedSubFastOp::<u32, u32>::is_supported {
            return unsafe { CheckedSubFastOp::<u32, u32>::Do(self, r, out) };
        }
        if !<u32 as Cast<u64>>::range(self).IsValid() || !<u32 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u32, out: &mut V) -> bool {
        if CheckedMulFastOp::<u32, u32>::is_supported {
            return unsafe { CheckedMulFastOp::<u32, u32>::Do(self, r, out) };
        }
        if (!<u32 as Cast<u64>>::range(self).IsValid() || !<u32 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u32 as Cast<u32>>::CONTAINED
            && (self as u32) == u32::MIN
            && r == (-1i64 as u32)
        {
            return false;
        }
        if (!<u32 as Cast<u32>>::range(self).IsValid() || !<u32 as Cast<u32>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u32) / (r as u32);
        if !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u32, V> for u32
where
    u32: Cast<V>,
{
    fn modulo(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u32 as Cast<u32>>::CONTAINED
            && (self as u32) == u32::MIN
            && r == (-1i64 as u32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u32) % (r as u32);
        if !<u32 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u32 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i64, V> for u32
where
    u32: BuiltinTriple<i64, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: i64, out: &mut V) -> bool {
        if CheckedAddFastOp::<u32, i64>::is_supported {
            return unsafe { CheckedAddFastOp::<u32, i64>::Do(self, r, out) };
        }
        if !<u32 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i64, out: &mut V) -> bool {
        if CheckedSubFastOp::<u32, i64>::is_supported {
            return unsafe { CheckedSubFastOp::<u32, i64>::Do(self, r, out) };
        }
        if !<u32 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i64, out: &mut V) -> bool {
        if CheckedMulFastOp::<u32, i64>::is_supported {
            return unsafe { CheckedMulFastOp::<u32, i64>::Do(self, r, out) };
        }
        if (!<u32 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<u32>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as i64)
        {
            return false;
        }
        if (!<u32 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i64, V> for u32
where
    i64: Cast<V>,
{
    fn modulo(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<u32>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as i64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u64, V> for u32
where
    u32: BuiltinTriple<u64, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u64: Cast<V>,
{
    fn add(self, r: u64, out: &mut V) -> bool {
        if CheckedAddFastOp::<u32, u64>::is_supported {
            return unsafe { CheckedAddFastOp::<u32, u64>::Do(self, r, out) };
        }
        if !<u32 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u64, out: &mut V) -> bool {
        if CheckedSubFastOp::<u32, u64>::is_supported {
            return unsafe { CheckedSubFastOp::<u32, u64>::Do(self, r, out) };
        }
        if !<u32 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u64, out: &mut V) -> bool {
        if CheckedMulFastOp::<u32, u64>::is_supported {
            return unsafe { CheckedMulFastOp::<u32, u64>::Do(self, r, out) };
        }
        if (!<u32 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u32>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u64)
        {
            return false;
        }
        if (!<u32 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u64) / (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u64, V> for u32
where
    u64: Cast<V>,
{
    fn modulo(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u32>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u64) % (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<isize, V> for u32
where
    u32: BuiltinTriple<isize, V>,
    isize: Cast<V>,
    isize: Cast<V>,
    isize: Cast<V>,
{
    fn add(self, r: isize, out: &mut V) -> bool {
        if CheckedAddFastOp::<u32, isize>::is_supported {
            return unsafe { CheckedAddFastOp::<u32, isize>::Do(self, r, out) };
        }
        if !<u32 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedAddImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: isize, out: &mut V) -> bool {
        if CheckedSubFastOp::<u32, isize>::is_supported {
            return unsafe { CheckedSubFastOp::<u32, isize>::Do(self, r, out) };
        }
        if !<u32 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedSubImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: isize, out: &mut V) -> bool {
        if CheckedMulFastOp::<u32, isize>::is_supported {
            return unsafe { CheckedMulFastOp::<u32, isize>::Do(self, r, out) };
        }
        if (!<u32 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = if CheckedMulFastOp::<isize, isize>::is_supported {
            unsafe { CheckedMulFastOp::<isize, isize>::Do(self as isize, r as isize, &mut result) }
        } else {
            isize_ops::CheckedMulImpl(self as isize, r as isize, &mut result)
        };
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <isize as Cast<u32>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as isize)
        {
            return false;
        }
        if (!<u32 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as isize) / (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<isize, V> for u32
where
    isize: Cast<V>,
{
    fn modulo(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <isize as Cast<u32>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as isize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as isize) % (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<usize, V> for u32
where
    u32: BuiltinTriple<usize, V>,
    usize: Cast<V>,
    usize: Cast<V>,
    usize: Cast<V>,
{
    fn add(self, r: usize, out: &mut V) -> bool {
        if CheckedAddFastOp::<u32, usize>::is_supported {
            return unsafe { CheckedAddFastOp::<u32, usize>::Do(self, r, out) };
        }
        if !<u32 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedAddImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: usize, out: &mut V) -> bool {
        if CheckedSubFastOp::<u32, usize>::is_supported {
            return unsafe { CheckedSubFastOp::<u32, usize>::Do(self, r, out) };
        }
        if !<u32 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedSubImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: usize, out: &mut V) -> bool {
        if CheckedMulFastOp::<u32, usize>::is_supported {
            return unsafe { CheckedMulFastOp::<u32, usize>::Do(self, r, out) };
        }
        if (!<u32 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = if CheckedMulFastOp::<usize, usize>::is_supported {
            unsafe { CheckedMulFastOp::<usize, usize>::Do(self as usize, r as usize, &mut result) }
        } else {
            usize_ops::CheckedMulImpl(self as usize, r as usize, &mut result)
        };
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<u32>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as usize)
        {
            return false;
        }
        if (!<u32 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as usize) / (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<usize, V> for u32
where
    usize: Cast<V>,
{
    fn modulo(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<u32>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as usize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as usize) % (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i8, V> for i64
where
    i64: BuiltinTriple<i8, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: i8, out: &mut V) -> bool {
        if CheckedAddFastOp::<i64, i8>::is_supported {
            return unsafe { CheckedAddFastOp::<i64, i8>::Do(self, r, out) };
        }
        if !<i64 as Cast<i64>>::range(self).IsValid() || !<i8 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i8, out: &mut V) -> bool {
        if CheckedSubFastOp::<i64, i8>::is_supported {
            return unsafe { CheckedSubFastOp::<i64, i8>::Do(self, r, out) };
        }
        if !<i64 as Cast<i64>>::range(self).IsValid() || !<i8 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i8, out: &mut V) -> bool {
        if CheckedMulFastOp::<i64, i8>::is_supported {
            return unsafe { CheckedMulFastOp::<i64, i8>::Do(self, r, out) };
        }
        if (!<i64 as Cast<i64>>::range(self).IsValid() || !<i8 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i64 as Cast<i64>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as i8)
        {
            return false;
        }
        if (!<i64 as Cast<i64>>::range(self).IsValid() || !<i8 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i8, V> for i64
where
    i64: Cast<V>,
{
    fn modulo(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i64 as Cast<i64>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as i8)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u8, V> for i64
where
    i64: BuiltinTriple<u8, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: u8, out: &mut V) -> bool {
        if CheckedAddFastOp::<i64, u8>::is_supported {
            return unsafe { CheckedAddFastOp::<i64, u8>::Do(self, r, out) };
        }
        if !<i64 as Cast<i64>>::range(self).IsValid() || !<u8 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u8, out: &mut V) -> bool {
        if CheckedSubFastOp::<i64, u8>::is_supported {
            return unsafe { CheckedSubFastOp::<i64, u8>::Do(self, r, out) };
        }
        if !<i64 as Cast<i64>>::range(self).IsValid() || !<u8 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u8, out: &mut V) -> bool {
        if CheckedMulFastOp::<i64, u8>::is_supported {
            return unsafe { CheckedMulFastOp::<i64, u8>::Do(self, r, out) };
        }
        if (!<i64 as Cast<i64>>::range(self).IsValid() || !<u8 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i64 as Cast<i64>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as u8)
        {
            return false;
        }
        if (!<i64 as Cast<i64>>::range(self).IsValid() || !<u8 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u8, V> for i64
where
    i64: Cast<V>,
{
    fn modulo(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <i64 as Cast<i64>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as u8)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i16, V> for i64
where
    i64: BuiltinTriple<i16, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: i16, out: &mut V) -> bool {
        if CheckedAddFastOp::<i64, i16>::is_supported {
            return unsafe { CheckedAddFastOp::<i64, i16>::Do(self, r, out) };
        }
        if !<i64 as Cast<i64>>::range(self).IsValid() || !<i16 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i16, out: &mut V) -> bool {
        if CheckedSubFastOp::<i64, i16>::is_supported {
            return unsafe { CheckedSubFastOp::<i64, i16>::Do(self, r, out) };
        }
        if !<i64 as Cast<i64>>::range(self).IsValid() || !<i16 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i16, out: &mut V) -> bool {
        if CheckedMulFastOp::<i64, i16>::is_supported {
            return unsafe { CheckedMulFastOp::<i64, i16>::Do(self, r, out) };
        }
        if (!<i64 as Cast<i64>>::range(self).IsValid() || !<i16 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i64 as Cast<i64>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as i16)
        {
            return false;
        }
        if (!<i64 as Cast<i64>>::range(self).IsValid() || !<i16 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i16, V> for i64
where
    i64: Cast<V>,
{
    fn modulo(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i64 as Cast<i64>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as i16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u16, V> for i64
where
    i64: BuiltinTriple<u16, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: u16, out: &mut V) -> bool {
        if CheckedAddFastOp::<i64, u16>::is_supported {
            return unsafe { CheckedAddFastOp::<i64, u16>::Do(self, r, out) };
        }
        if !<i64 as Cast<i64>>::range(self).IsValid() || !<u16 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u16, out: &mut V) -> bool {
        if CheckedSubFastOp::<i64, u16>::is_supported {
            return unsafe { CheckedSubFastOp::<i64, u16>::Do(self, r, out) };
        }
        if !<i64 as Cast<i64>>::range(self).IsValid() || !<u16 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u16, out: &mut V) -> bool {
        if CheckedMulFastOp::<i64, u16>::is_supported {
            return unsafe { CheckedMulFastOp::<i64, u16>::Do(self, r, out) };
        }
        if (!<i64 as Cast<i64>>::range(self).IsValid() || !<u16 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<i64>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as u16)
        {
            return false;
        }
        if (!<i64 as Cast<i64>>::range(self).IsValid() || !<u16 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u16, V> for i64
where
    i64: Cast<V>,
{
    fn modulo(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<i64>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as u16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i32, V> for i64
where
    i64: BuiltinTriple<i32, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: i32, out: &mut V) -> bool {
        if CheckedAddFastOp::<i64, i32>::is_supported {
            return unsafe { CheckedAddFastOp::<i64, i32>::Do(self, r, out) };
        }
        if !<i64 as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i32, out: &mut V) -> bool {
        if CheckedSubFastOp::<i64, i32>::is_supported {
            return unsafe { CheckedSubFastOp::<i64, i32>::Do(self, r, out) };
        }
        if !<i64 as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i32, out: &mut V) -> bool {
        if CheckedMulFastOp::<i64, i32>::is_supported {
            return unsafe { CheckedMulFastOp::<i64, i32>::Do(self, r, out) };
        }
        if (!<i64 as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i64 as Cast<i64>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as i32)
        {
            return false;
        }
        if (!<i64 as Cast<i64>>::range(self).IsValid() || !<i32 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i32, V> for i64
where
    i64: Cast<V>,
{
    fn modulo(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i64 as Cast<i64>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as i32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u32, V> for i64
where
    i64: BuiltinTriple<u32, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: u32, out: &mut V) -> bool {
        if CheckedAddFastOp::<i64, u32>::is_supported {
            return unsafe { CheckedAddFastOp::<i64, u32>::Do(self, r, out) };
        }
        if !<i64 as Cast<i64>>::range(self).IsValid() || !<u32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u32, out: &mut V) -> bool {
        if CheckedSubFastOp::<i64, u32>::is_supported {
            return unsafe { CheckedSubFastOp::<i64, u32>::Do(self, r, out) };
        }
        if !<i64 as Cast<i64>>::range(self).IsValid() || !<u32 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u32, out: &mut V) -> bool {
        if CheckedMulFastOp::<i64, u32>::is_supported {
            return unsafe { CheckedMulFastOp::<i64, u32>::Do(self, r, out) };
        }
        if (!<i64 as Cast<i64>>::range(self).IsValid() || !<u32 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<i64>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as u32)
        {
            return false;
        }
        if (!<i64 as Cast<i64>>::range(self).IsValid() || !<u32 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u32, V> for i64
where
    i64: Cast<V>,
{
    fn modulo(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <i64 as Cast<i64>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as u32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i64, V> for i64
where
    i64: BuiltinTriple<i64, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: i64, out: &mut V) -> bool {
        if CheckedAddFastOp::<i64, i64>::is_supported {
            return unsafe { CheckedAddFastOp::<i64, i64>::Do(self, r, out) };
        }
        if !<i64 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i64, out: &mut V) -> bool {
        if CheckedSubFastOp::<i64, i64>::is_supported {
            return unsafe { CheckedSubFastOp::<i64, i64>::Do(self, r, out) };
        }
        if !<i64 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i64, out: &mut V) -> bool {
        if CheckedMulFastOp::<i64, i64>::is_supported {
            return unsafe { CheckedMulFastOp::<i64, i64>::Do(self, r, out) };
        }
        if (!<i64 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i64 as Cast<i64>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as i64)
        {
            return false;
        }
        if (!<i64 as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i64, V> for i64
where
    i64: Cast<V>,
{
    fn modulo(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true && <i64 as Cast<i64>>::CONTAINED && (self as i64) == i64::MIN && r == (-1i64 as i64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u64, V> for i64
where
    i64: BuiltinTriple<u64, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u64: Cast<V>,
{
    fn add(self, r: u64, out: &mut V) -> bool {
        if CheckedAddFastOp::<i64, u64>::is_supported {
            return unsafe { CheckedAddFastOp::<i64, u64>::Do(self, r, out) };
        }
        if !<i64 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u64, out: &mut V) -> bool {
        if CheckedSubFastOp::<i64, u64>::is_supported {
            return unsafe { CheckedSubFastOp::<i64, u64>::Do(self, r, out) };
        }
        if !<i64 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u64, out: &mut V) -> bool {
        if CheckedMulFastOp::<i64, u64>::is_supported {
            return unsafe { CheckedMulFastOp::<i64, u64>::Do(self, r, out) };
        }
        if (!<i64 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<i64>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u64)
        {
            return false;
        }
        if (!<i64 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u64) / (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u64, V> for i64
where
    u64: Cast<V>,
{
    fn modulo(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<i64>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u64) % (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<isize, V> for i64
where
    i64: BuiltinTriple<isize, V>,
    isize: Cast<V>,
    isize: Cast<V>,
    isize: Cast<V>,
{
    fn add(self, r: isize, out: &mut V) -> bool {
        if CheckedAddFastOp::<i64, isize>::is_supported {
            return unsafe { CheckedAddFastOp::<i64, isize>::Do(self, r, out) };
        }
        if !<i64 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedAddImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: isize, out: &mut V) -> bool {
        if CheckedSubFastOp::<i64, isize>::is_supported {
            return unsafe { CheckedSubFastOp::<i64, isize>::Do(self, r, out) };
        }
        if !<i64 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedSubImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: isize, out: &mut V) -> bool {
        if CheckedMulFastOp::<i64, isize>::is_supported {
            return unsafe { CheckedMulFastOp::<i64, isize>::Do(self, r, out) };
        }
        if (!<i64 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = if CheckedMulFastOp::<isize, isize>::is_supported {
            unsafe { CheckedMulFastOp::<isize, isize>::Do(self as isize, r as isize, &mut result) }
        } else {
            isize_ops::CheckedMulImpl(self as isize, r as isize, &mut result)
        };
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true
            && <isize as Cast<i64>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as isize)
        {
            return false;
        }
        if (!<i64 as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as isize) / (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<isize, V> for i64
where
    isize: Cast<V>,
{
    fn modulo(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true
            && <isize as Cast<i64>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as isize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as isize) % (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<usize, V> for i64
where
    i64: BuiltinTriple<usize, V>,
    usize: Cast<V>,
    usize: Cast<V>,
    usize: Cast<V>,
{
    fn add(self, r: usize, out: &mut V) -> bool {
        if CheckedAddFastOp::<i64, usize>::is_supported {
            return unsafe { CheckedAddFastOp::<i64, usize>::Do(self, r, out) };
        }
        if !<i64 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedAddImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: usize, out: &mut V) -> bool {
        if CheckedSubFastOp::<i64, usize>::is_supported {
            return unsafe { CheckedSubFastOp::<i64, usize>::Do(self, r, out) };
        }
        if !<i64 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedSubImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: usize, out: &mut V) -> bool {
        if CheckedMulFastOp::<i64, usize>::is_supported {
            return unsafe { CheckedMulFastOp::<i64, usize>::Do(self, r, out) };
        }
        if (!<i64 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = if CheckedMulFastOp::<usize, usize>::is_supported {
            unsafe { CheckedMulFastOp::<usize, usize>::Do(self as usize, r as usize, &mut result) }
        } else {
            usize_ops::CheckedMulImpl(self as usize, r as usize, &mut result)
        };
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<i64>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as usize)
        {
            return false;
        }
        if (!<i64 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as usize) / (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<usize, V> for i64
where
    usize: Cast<V>,
{
    fn modulo(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<i64>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as usize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as usize) % (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i8, V> for u64
where
    u64: BuiltinTriple<i8, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u64: Cast<V>,
{
    fn add(self, r: i8, out: &mut V) -> bool {
        if CheckedAddFastOp::<u64, i8>::is_supported {
            return unsafe { CheckedAddFastOp::<u64, i8>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<i8 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i8, out: &mut V) -> bool {
        if CheckedSubFastOp::<u64, i8>::is_supported {
            return unsafe { CheckedSubFastOp::<u64, i8>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<i8 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i8, out: &mut V) -> bool {
        if CheckedMulFastOp::<u64, i8>::is_supported {
            return unsafe { CheckedMulFastOp::<u64, i8>::Do(self, r, out) };
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<i8 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <u64 as Cast<u64>>::CONTAINED && (self as u64) == u64::MIN && r == (-1i64 as i8)
        {
            return false;
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<i8 as Cast<u64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u64) / (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i8, V> for u64
where
    u64: Cast<V>,
{
    fn modulo(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <u64 as Cast<u64>>::CONTAINED && (self as u64) == u64::MIN && r == (-1i64 as i8)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u64) % (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u8, V> for u64
where
    u64: BuiltinTriple<u8, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u64: Cast<V>,
{
    fn add(self, r: u8, out: &mut V) -> bool {
        if CheckedAddFastOp::<u64, u8>::is_supported {
            return unsafe { CheckedAddFastOp::<u64, u8>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<u8 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u8, out: &mut V) -> bool {
        if CheckedSubFastOp::<u64, u8>::is_supported {
            return unsafe { CheckedSubFastOp::<u64, u8>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<u8 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u8, out: &mut V) -> bool {
        if CheckedMulFastOp::<u64, u8>::is_supported {
            return unsafe { CheckedMulFastOp::<u64, u8>::Do(self, r, out) };
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<u8 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <u64 as Cast<u64>>::CONTAINED && (self as u64) == u64::MIN && r == (-1i64 as u8)
        {
            return false;
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<u8 as Cast<u64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u64) / (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u8, V> for u64
where
    u64: Cast<V>,
{
    fn modulo(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false && <u64 as Cast<u64>>::CONTAINED && (self as u64) == u64::MIN && r == (-1i64 as u8)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u64) % (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i16, V> for u64
where
    u64: BuiltinTriple<i16, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u64: Cast<V>,
{
    fn add(self, r: i16, out: &mut V) -> bool {
        if CheckedAddFastOp::<u64, i16>::is_supported {
            return unsafe { CheckedAddFastOp::<u64, i16>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<i16 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i16, out: &mut V) -> bool {
        if CheckedSubFastOp::<u64, i16>::is_supported {
            return unsafe { CheckedSubFastOp::<u64, i16>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<i16 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i16, out: &mut V) -> bool {
        if CheckedMulFastOp::<u64, i16>::is_supported {
            return unsafe { CheckedMulFastOp::<u64, i16>::Do(self, r, out) };
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<i16 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u64>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as i16)
        {
            return false;
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<i16 as Cast<u64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u64) / (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i16, V> for u64
where
    u64: Cast<V>,
{
    fn modulo(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u64>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as i16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u64) % (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u16, V> for u64
where
    u64: BuiltinTriple<u16, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u64: Cast<V>,
{
    fn add(self, r: u16, out: &mut V) -> bool {
        if CheckedAddFastOp::<u64, u16>::is_supported {
            return unsafe { CheckedAddFastOp::<u64, u16>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<u16 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u16, out: &mut V) -> bool {
        if CheckedSubFastOp::<u64, u16>::is_supported {
            return unsafe { CheckedSubFastOp::<u64, u16>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<u16 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u16, out: &mut V) -> bool {
        if CheckedMulFastOp::<u64, u16>::is_supported {
            return unsafe { CheckedMulFastOp::<u64, u16>::Do(self, r, out) };
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<u16 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u64>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u16)
        {
            return false;
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<u16 as Cast<u64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u64) / (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u16, V> for u64
where
    u64: Cast<V>,
{
    fn modulo(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u64>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u64) % (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i32, V> for u64
where
    u64: BuiltinTriple<i32, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u64: Cast<V>,
{
    fn add(self, r: i32, out: &mut V) -> bool {
        if CheckedAddFastOp::<u64, i32>::is_supported {
            return unsafe { CheckedAddFastOp::<u64, i32>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<i32 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i32, out: &mut V) -> bool {
        if CheckedSubFastOp::<u64, i32>::is_supported {
            return unsafe { CheckedSubFastOp::<u64, i32>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<i32 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i32, out: &mut V) -> bool {
        if CheckedMulFastOp::<u64, i32>::is_supported {
            return unsafe { CheckedMulFastOp::<u64, i32>::Do(self, r, out) };
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<i32 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u64>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as i32)
        {
            return false;
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<i32 as Cast<u64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u64) / (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i32, V> for u64
where
    u64: Cast<V>,
{
    fn modulo(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u64>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as i32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u64) % (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u32, V> for u64
where
    u64: BuiltinTriple<u32, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u64: Cast<V>,
{
    fn add(self, r: u32, out: &mut V) -> bool {
        if CheckedAddFastOp::<u64, u32>::is_supported {
            return unsafe { CheckedAddFastOp::<u64, u32>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<u32 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u32, out: &mut V) -> bool {
        if CheckedSubFastOp::<u64, u32>::is_supported {
            return unsafe { CheckedSubFastOp::<u64, u32>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<u32 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u32, out: &mut V) -> bool {
        if CheckedMulFastOp::<u64, u32>::is_supported {
            return unsafe { CheckedMulFastOp::<u64, u32>::Do(self, r, out) };
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<u32 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u64>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u32)
        {
            return false;
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<u32 as Cast<u64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u64) / (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u32, V> for u64
where
    u64: Cast<V>,
{
    fn modulo(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u64>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u64) % (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i64, V> for u64
where
    u64: BuiltinTriple<i64, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u64: Cast<V>,
{
    fn add(self, r: i64, out: &mut V) -> bool {
        if CheckedAddFastOp::<u64, i64>::is_supported {
            return unsafe { CheckedAddFastOp::<u64, i64>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<i64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i64, out: &mut V) -> bool {
        if CheckedSubFastOp::<u64, i64>::is_supported {
            return unsafe { CheckedSubFastOp::<u64, i64>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<i64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i64, out: &mut V) -> bool {
        if CheckedMulFastOp::<u64, i64>::is_supported {
            return unsafe { CheckedMulFastOp::<u64, i64>::Do(self, r, out) };
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<i64 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u64>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as i64)
        {
            return false;
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<i64 as Cast<u64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u64) / (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i64, V> for u64
where
    u64: Cast<V>,
{
    fn modulo(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u64>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as i64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u64) % (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u64, V> for u64
where
    u64: BuiltinTriple<u64, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u64: Cast<V>,
{
    fn add(self, r: u64, out: &mut V) -> bool {
        if CheckedAddFastOp::<u64, u64>::is_supported {
            return unsafe { CheckedAddFastOp::<u64, u64>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u64, out: &mut V) -> bool {
        if CheckedSubFastOp::<u64, u64>::is_supported {
            return unsafe { CheckedSubFastOp::<u64, u64>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u64, out: &mut V) -> bool {
        if CheckedMulFastOp::<u64, u64>::is_supported {
            return unsafe { CheckedMulFastOp::<u64, u64>::Do(self, r, out) };
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u64>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u64)
        {
            return false;
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u64) / (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u64, V> for u64
where
    u64: Cast<V>,
{
    fn modulo(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u64>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u64) % (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<isize, V> for u64
where
    u64: BuiltinTriple<isize, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u64: Cast<V>,
{
    fn add(self, r: isize, out: &mut V) -> bool {
        if CheckedAddFastOp::<u64, isize>::is_supported {
            return unsafe { CheckedAddFastOp::<u64, isize>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<isize as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: isize, out: &mut V) -> bool {
        if CheckedSubFastOp::<u64, isize>::is_supported {
            return unsafe { CheckedSubFastOp::<u64, isize>::Do(self, r, out) };
        }
        if !<u64 as Cast<u64>>::range(self).IsValid() || !<isize as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: isize, out: &mut V) -> bool {
        if CheckedMulFastOp::<u64, isize>::is_supported {
            return unsafe { CheckedMulFastOp::<u64, isize>::Do(self, r, out) };
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<isize as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u64>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as isize)
        {
            return false;
        }
        if (!<u64 as Cast<u64>>::range(self).IsValid() || !<isize as Cast<u64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u64) / (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<isize, V> for u64
where
    u64: Cast<V>,
{
    fn modulo(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<u64>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as isize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u64) % (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<usize, V> for u64
where
    u64: BuiltinTriple<usize, V>,
    usize: Cast<V>,
    usize: Cast<V>,
    usize: Cast<V>,
{
    fn add(self, r: usize, out: &mut V) -> bool {
        if CheckedAddFastOp::<u64, usize>::is_supported {
            return unsafe { CheckedAddFastOp::<u64, usize>::Do(self, r, out) };
        }
        if !<u64 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedAddImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: usize, out: &mut V) -> bool {
        if CheckedSubFastOp::<u64, usize>::is_supported {
            return unsafe { CheckedSubFastOp::<u64, usize>::Do(self, r, out) };
        }
        if !<u64 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedSubImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: usize, out: &mut V) -> bool {
        if CheckedMulFastOp::<u64, usize>::is_supported {
            return unsafe { CheckedMulFastOp::<u64, usize>::Do(self, r, out) };
        }
        if (!<u64 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = if CheckedMulFastOp::<usize, usize>::is_supported {
            unsafe { CheckedMulFastOp::<usize, usize>::Do(self as usize, r as usize, &mut result) }
        } else {
            usize_ops::CheckedMulImpl(self as usize, r as usize, &mut result)
        };
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<u64>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as usize)
        {
            return false;
        }
        if (!<u64 as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as usize) / (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<usize, V> for u64
where
    usize: Cast<V>,
{
    fn modulo(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<u64>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as usize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as usize) % (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i8, V> for isize
where
    isize: BuiltinTriple<i8, V>,
    isize: Cast<V>,
    isize: Cast<V>,
    isize: Cast<V>,
{
    fn add(self, r: i8, out: &mut V) -> bool {
        if CheckedAddFastOp::<isize, i8>::is_supported {
            return unsafe { CheckedAddFastOp::<isize, i8>::Do(self, r, out) };
        }
        if !<isize as Cast<isize>>::range(self).IsValid()
            || !<i8 as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedAddImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i8, out: &mut V) -> bool {
        if CheckedSubFastOp::<isize, i8>::is_supported {
            return unsafe { CheckedSubFastOp::<isize, i8>::Do(self, r, out) };
        }
        if !<isize as Cast<isize>>::range(self).IsValid()
            || !<i8 as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedSubImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i8, out: &mut V) -> bool {
        if CheckedMulFastOp::<isize, i8>::is_supported {
            return unsafe { CheckedMulFastOp::<isize, i8>::Do(self, r, out) };
        }
        if (!<isize as Cast<isize>>::range(self).IsValid()
            || !<i8 as Cast<isize>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = if CheckedMulFastOp::<isize, isize>::is_supported {
            unsafe { CheckedMulFastOp::<isize, isize>::Do(self as isize, r as isize, &mut result) }
        } else {
            isize_ops::CheckedMulImpl(self as isize, r as isize, &mut result)
        };
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true
            && <isize as Cast<isize>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as i8)
        {
            return false;
        }
        if (!<isize as Cast<isize>>::range(self).IsValid()
            || !<i8 as Cast<isize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as isize) / (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i8, V> for isize
where
    isize: Cast<V>,
{
    fn modulo(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true
            && <isize as Cast<isize>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as i8)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as isize) % (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u8, V> for isize
where
    isize: BuiltinTriple<u8, V>,
    isize: Cast<V>,
    isize: Cast<V>,
    isize: Cast<V>,
{
    fn add(self, r: u8, out: &mut V) -> bool {
        if CheckedAddFastOp::<isize, u8>::is_supported {
            return unsafe { CheckedAddFastOp::<isize, u8>::Do(self, r, out) };
        }
        if !<isize as Cast<isize>>::range(self).IsValid()
            || !<u8 as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedAddImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u8, out: &mut V) -> bool {
        if CheckedSubFastOp::<isize, u8>::is_supported {
            return unsafe { CheckedSubFastOp::<isize, u8>::Do(self, r, out) };
        }
        if !<isize as Cast<isize>>::range(self).IsValid()
            || !<u8 as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedSubImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u8, out: &mut V) -> bool {
        if CheckedMulFastOp::<isize, u8>::is_supported {
            return unsafe { CheckedMulFastOp::<isize, u8>::Do(self, r, out) };
        }
        if (!<isize as Cast<isize>>::range(self).IsValid()
            || !<u8 as Cast<isize>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = if CheckedMulFastOp::<isize, isize>::is_supported {
            unsafe { CheckedMulFastOp::<isize, isize>::Do(self as isize, r as isize, &mut result) }
        } else {
            isize_ops::CheckedMulImpl(self as isize, r as isize, &mut result)
        };
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <isize as Cast<isize>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as u8)
        {
            return false;
        }
        if (!<isize as Cast<isize>>::range(self).IsValid()
            || !<u8 as Cast<isize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as isize) / (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u8, V> for isize
where
    isize: Cast<V>,
{
    fn modulo(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <isize as Cast<isize>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as u8)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as isize) % (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i16, V> for isize
where
    isize: BuiltinTriple<i16, V>,
    isize: Cast<V>,
    isize: Cast<V>,
    isize: Cast<V>,
{
    fn add(self, r: i16, out: &mut V) -> bool {
        if CheckedAddFastOp::<isize, i16>::is_supported {
            return unsafe { CheckedAddFastOp::<isize, i16>::Do(self, r, out) };
        }
        if !<isize as Cast<isize>>::range(self).IsValid()
            || !<i16 as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedAddImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i16, out: &mut V) -> bool {
        if CheckedSubFastOp::<isize, i16>::is_supported {
            return unsafe { CheckedSubFastOp::<isize, i16>::Do(self, r, out) };
        }
        if !<isize as Cast<isize>>::range(self).IsValid()
            || !<i16 as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedSubImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i16, out: &mut V) -> bool {
        if CheckedMulFastOp::<isize, i16>::is_supported {
            return unsafe { CheckedMulFastOp::<isize, i16>::Do(self, r, out) };
        }
        if (!<isize as Cast<isize>>::range(self).IsValid()
            || !<i16 as Cast<isize>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = if CheckedMulFastOp::<isize, isize>::is_supported {
            unsafe { CheckedMulFastOp::<isize, isize>::Do(self as isize, r as isize, &mut result) }
        } else {
            isize_ops::CheckedMulImpl(self as isize, r as isize, &mut result)
        };
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true
            && <isize as Cast<isize>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as i16)
        {
            return false;
        }
        if (!<isize as Cast<isize>>::range(self).IsValid()
            || !<i16 as Cast<isize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as isize) / (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i16, V> for isize
where
    isize: Cast<V>,
{
    fn modulo(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true
            && <isize as Cast<isize>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as i16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as isize) % (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u16, V> for isize
where
    isize: BuiltinTriple<u16, V>,
    isize: Cast<V>,
    isize: Cast<V>,
    isize: Cast<V>,
{
    fn add(self, r: u16, out: &mut V) -> bool {
        if CheckedAddFastOp::<isize, u16>::is_supported {
            return unsafe { CheckedAddFastOp::<isize, u16>::Do(self, r, out) };
        }
        if !<isize as Cast<isize>>::range(self).IsValid()
            || !<u16 as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedAddImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u16, out: &mut V) -> bool {
        if CheckedSubFastOp::<isize, u16>::is_supported {
            return unsafe { CheckedSubFastOp::<isize, u16>::Do(self, r, out) };
        }
        if !<isize as Cast<isize>>::range(self).IsValid()
            || !<u16 as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedSubImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u16, out: &mut V) -> bool {
        if CheckedMulFastOp::<isize, u16>::is_supported {
            return unsafe { CheckedMulFastOp::<isize, u16>::Do(self, r, out) };
        }
        if (!<isize as Cast<isize>>::range(self).IsValid()
            || !<u16 as Cast<isize>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = if CheckedMulFastOp::<isize, isize>::is_supported {
            unsafe { CheckedMulFastOp::<isize, isize>::Do(self as isize, r as isize, &mut result) }
        } else {
            isize_ops::CheckedMulImpl(self as isize, r as isize, &mut result)
        };
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <isize as Cast<isize>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as u16)
        {
            return false;
        }
        if (!<isize as Cast<isize>>::range(self).IsValid()
            || !<u16 as Cast<isize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as isize) / (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u16, V> for isize
where
    isize: Cast<V>,
{
    fn modulo(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <isize as Cast<isize>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as u16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as isize) % (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i32, V> for isize
where
    isize: BuiltinTriple<i32, V>,
    isize: Cast<V>,
    isize: Cast<V>,
    isize: Cast<V>,
{
    fn add(self, r: i32, out: &mut V) -> bool {
        if CheckedAddFastOp::<isize, i32>::is_supported {
            return unsafe { CheckedAddFastOp::<isize, i32>::Do(self, r, out) };
        }
        if !<isize as Cast<isize>>::range(self).IsValid()
            || !<i32 as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedAddImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i32, out: &mut V) -> bool {
        if CheckedSubFastOp::<isize, i32>::is_supported {
            return unsafe { CheckedSubFastOp::<isize, i32>::Do(self, r, out) };
        }
        if !<isize as Cast<isize>>::range(self).IsValid()
            || !<i32 as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedSubImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i32, out: &mut V) -> bool {
        if CheckedMulFastOp::<isize, i32>::is_supported {
            return unsafe { CheckedMulFastOp::<isize, i32>::Do(self, r, out) };
        }
        if (!<isize as Cast<isize>>::range(self).IsValid()
            || !<i32 as Cast<isize>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = if CheckedMulFastOp::<isize, isize>::is_supported {
            unsafe { CheckedMulFastOp::<isize, isize>::Do(self as isize, r as isize, &mut result) }
        } else {
            isize_ops::CheckedMulImpl(self as isize, r as isize, &mut result)
        };
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true
            && <isize as Cast<isize>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as i32)
        {
            return false;
        }
        if (!<isize as Cast<isize>>::range(self).IsValid()
            || !<i32 as Cast<isize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as isize) / (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i32, V> for isize
where
    isize: Cast<V>,
{
    fn modulo(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true
            && <isize as Cast<isize>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as i32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as isize) % (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u32, V> for isize
where
    isize: BuiltinTriple<u32, V>,
    isize: Cast<V>,
    isize: Cast<V>,
    isize: Cast<V>,
{
    fn add(self, r: u32, out: &mut V) -> bool {
        if CheckedAddFastOp::<isize, u32>::is_supported {
            return unsafe { CheckedAddFastOp::<isize, u32>::Do(self, r, out) };
        }
        if !<isize as Cast<isize>>::range(self).IsValid()
            || !<u32 as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedAddImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u32, out: &mut V) -> bool {
        if CheckedSubFastOp::<isize, u32>::is_supported {
            return unsafe { CheckedSubFastOp::<isize, u32>::Do(self, r, out) };
        }
        if !<isize as Cast<isize>>::range(self).IsValid()
            || !<u32 as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedSubImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u32, out: &mut V) -> bool {
        if CheckedMulFastOp::<isize, u32>::is_supported {
            return unsafe { CheckedMulFastOp::<isize, u32>::Do(self, r, out) };
        }
        if (!<isize as Cast<isize>>::range(self).IsValid()
            || !<u32 as Cast<isize>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = if CheckedMulFastOp::<isize, isize>::is_supported {
            unsafe { CheckedMulFastOp::<isize, isize>::Do(self as isize, r as isize, &mut result) }
        } else {
            isize_ops::CheckedMulImpl(self as isize, r as isize, &mut result)
        };
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <isize as Cast<isize>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as u32)
        {
            return false;
        }
        if (!<isize as Cast<isize>>::range(self).IsValid()
            || !<u32 as Cast<isize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as isize) / (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u32, V> for isize
where
    isize: Cast<V>,
{
    fn modulo(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <isize as Cast<isize>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as u32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as isize) % (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i64, V> for isize
where
    isize: BuiltinTriple<i64, V>,
    i64: Cast<V>,
    i64: Cast<V>,
    i64: Cast<V>,
{
    fn add(self, r: i64, out: &mut V) -> bool {
        if CheckedAddFastOp::<isize, i64>::is_supported {
            return unsafe { CheckedAddFastOp::<isize, i64>::Do(self, r, out) };
        }
        if !<isize as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedAddImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i64, out: &mut V) -> bool {
        if CheckedSubFastOp::<isize, i64>::is_supported {
            return unsafe { CheckedSubFastOp::<isize, i64>::Do(self, r, out) };
        }
        if !<isize as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as i64;
        let valid = i64_ops::CheckedSubImpl(self as i64, r as i64, &mut result);
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i64, out: &mut V) -> bool {
        if CheckedMulFastOp::<isize, i64>::is_supported {
            return unsafe { CheckedMulFastOp::<isize, i64>::Do(self, r, out) };
        }
        if (!<isize as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as i64;
        let valid = if CheckedMulFastOp::<i64, i64>::is_supported {
            unsafe { CheckedMulFastOp::<i64, i64>::Do(self as i64, r as i64, &mut result) }
        } else {
            i64_ops::CheckedMulImpl(self as i64, r as i64, &mut result)
        };
        if !valid || !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true
            && <i64 as Cast<isize>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as i64)
        {
            return false;
        }
        if (!<isize as Cast<i64>>::range(self).IsValid() || !<i64 as Cast<i64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as i64) / (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i64, V> for isize
where
    i64: Cast<V>,
{
    fn modulo(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true
            && <i64 as Cast<isize>>::CONTAINED
            && (self as i64) == i64::MIN
            && r == (-1i64 as i64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as i64) % (r as i64);
        if !<i64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <i64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u64, V> for isize
where
    isize: BuiltinTriple<u64, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u64: Cast<V>,
{
    fn add(self, r: u64, out: &mut V) -> bool {
        if CheckedAddFastOp::<isize, u64>::is_supported {
            return unsafe { CheckedAddFastOp::<isize, u64>::Do(self, r, out) };
        }
        if !<isize as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u64, out: &mut V) -> bool {
        if CheckedSubFastOp::<isize, u64>::is_supported {
            return unsafe { CheckedSubFastOp::<isize, u64>::Do(self, r, out) };
        }
        if !<isize as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u64, out: &mut V) -> bool {
        if CheckedMulFastOp::<isize, u64>::is_supported {
            return unsafe { CheckedMulFastOp::<isize, u64>::Do(self, r, out) };
        }
        if (!<isize as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<isize>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u64)
        {
            return false;
        }
        if (!<isize as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u64) / (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u64, V> for isize
where
    u64: Cast<V>,
{
    fn modulo(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<isize>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u64) % (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<isize, V> for isize
where
    isize: BuiltinTriple<isize, V>,
    isize: Cast<V>,
    isize: Cast<V>,
    isize: Cast<V>,
{
    fn add(self, r: isize, out: &mut V) -> bool {
        if CheckedAddFastOp::<isize, isize>::is_supported {
            return unsafe { CheckedAddFastOp::<isize, isize>::Do(self, r, out) };
        }
        if !<isize as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedAddImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: isize, out: &mut V) -> bool {
        if CheckedSubFastOp::<isize, isize>::is_supported {
            return unsafe { CheckedSubFastOp::<isize, isize>::Do(self, r, out) };
        }
        if !<isize as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = isize_ops::CheckedSubImpl(self as isize, r as isize, &mut result);
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: isize, out: &mut V) -> bool {
        if CheckedMulFastOp::<isize, isize>::is_supported {
            return unsafe { CheckedMulFastOp::<isize, isize>::Do(self, r, out) };
        }
        if (!<isize as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as isize;
        let valid = if CheckedMulFastOp::<isize, isize>::is_supported {
            unsafe { CheckedMulFastOp::<isize, isize>::Do(self as isize, r as isize, &mut result) }
        } else {
            isize_ops::CheckedMulImpl(self as isize, r as isize, &mut result)
        };
        if !valid || !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true
            && <isize as Cast<isize>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as isize)
        {
            return false;
        }
        if (!<isize as Cast<isize>>::range(self).IsValid()
            || !<isize as Cast<isize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as isize) / (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<isize, V> for isize
where
    isize: Cast<V>,
{
    fn modulo(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if true
            && <isize as Cast<isize>>::CONTAINED
            && (self as isize) == isize::MIN
            && r == (-1i64 as isize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as isize) % (r as isize);
        if !<isize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <isize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<usize, V> for isize
where
    isize: BuiltinTriple<usize, V>,
    usize: Cast<V>,
    usize: Cast<V>,
    usize: Cast<V>,
{
    fn add(self, r: usize, out: &mut V) -> bool {
        if CheckedAddFastOp::<isize, usize>::is_supported {
            return unsafe { CheckedAddFastOp::<isize, usize>::Do(self, r, out) };
        }
        if !<isize as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedAddImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: usize, out: &mut V) -> bool {
        if CheckedSubFastOp::<isize, usize>::is_supported {
            return unsafe { CheckedSubFastOp::<isize, usize>::Do(self, r, out) };
        }
        if !<isize as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedSubImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: usize, out: &mut V) -> bool {
        if CheckedMulFastOp::<isize, usize>::is_supported {
            return unsafe { CheckedMulFastOp::<isize, usize>::Do(self, r, out) };
        }
        if (!<isize as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = if CheckedMulFastOp::<usize, usize>::is_supported {
            unsafe { CheckedMulFastOp::<usize, usize>::Do(self as usize, r as usize, &mut result) }
        } else {
            usize_ops::CheckedMulImpl(self as usize, r as usize, &mut result)
        };
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<isize>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as usize)
        {
            return false;
        }
        if (!<isize as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as usize) / (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<usize, V> for isize
where
    usize: Cast<V>,
{
    fn modulo(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<isize>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as usize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as usize) % (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i8, V> for usize
where
    usize: BuiltinTriple<i8, V>,
    usize: Cast<V>,
    usize: Cast<V>,
    usize: Cast<V>,
{
    fn add(self, r: i8, out: &mut V) -> bool {
        if CheckedAddFastOp::<usize, i8>::is_supported {
            return unsafe { CheckedAddFastOp::<usize, i8>::Do(self, r, out) };
        }
        if !<usize as Cast<usize>>::range(self).IsValid()
            || !<i8 as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedAddImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i8, out: &mut V) -> bool {
        if CheckedSubFastOp::<usize, i8>::is_supported {
            return unsafe { CheckedSubFastOp::<usize, i8>::Do(self, r, out) };
        }
        if !<usize as Cast<usize>>::range(self).IsValid()
            || !<i8 as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedSubImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i8, out: &mut V) -> bool {
        if CheckedMulFastOp::<usize, i8>::is_supported {
            return unsafe { CheckedMulFastOp::<usize, i8>::Do(self, r, out) };
        }
        if (!<usize as Cast<usize>>::range(self).IsValid()
            || !<i8 as Cast<usize>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = if CheckedMulFastOp::<usize, usize>::is_supported {
            unsafe { CheckedMulFastOp::<usize, usize>::Do(self as usize, r as usize, &mut result) }
        } else {
            usize_ops::CheckedMulImpl(self as usize, r as usize, &mut result)
        };
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<usize>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as i8)
        {
            return false;
        }
        if (!<usize as Cast<usize>>::range(self).IsValid()
            || !<i8 as Cast<usize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as usize) / (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i8, V> for usize
where
    usize: Cast<V>,
{
    fn modulo(self, r: i8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<usize>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as i8)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as usize) % (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u8, V> for usize
where
    usize: BuiltinTriple<u8, V>,
    usize: Cast<V>,
    usize: Cast<V>,
    usize: Cast<V>,
{
    fn add(self, r: u8, out: &mut V) -> bool {
        if CheckedAddFastOp::<usize, u8>::is_supported {
            return unsafe { CheckedAddFastOp::<usize, u8>::Do(self, r, out) };
        }
        if !<usize as Cast<usize>>::range(self).IsValid()
            || !<u8 as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedAddImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u8, out: &mut V) -> bool {
        if CheckedSubFastOp::<usize, u8>::is_supported {
            return unsafe { CheckedSubFastOp::<usize, u8>::Do(self, r, out) };
        }
        if !<usize as Cast<usize>>::range(self).IsValid()
            || !<u8 as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedSubImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u8, out: &mut V) -> bool {
        if CheckedMulFastOp::<usize, u8>::is_supported {
            return unsafe { CheckedMulFastOp::<usize, u8>::Do(self, r, out) };
        }
        if (!<usize as Cast<usize>>::range(self).IsValid()
            || !<u8 as Cast<usize>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = if CheckedMulFastOp::<usize, usize>::is_supported {
            unsafe { CheckedMulFastOp::<usize, usize>::Do(self as usize, r as usize, &mut result) }
        } else {
            usize_ops::CheckedMulImpl(self as usize, r as usize, &mut result)
        };
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<usize>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as u8)
        {
            return false;
        }
        if (!<usize as Cast<usize>>::range(self).IsValid()
            || !<u8 as Cast<usize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as usize) / (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u8, V> for usize
where
    usize: Cast<V>,
{
    fn modulo(self, r: u8, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<usize>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as u8)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as usize) % (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i16, V> for usize
where
    usize: BuiltinTriple<i16, V>,
    usize: Cast<V>,
    usize: Cast<V>,
    usize: Cast<V>,
{
    fn add(self, r: i16, out: &mut V) -> bool {
        if CheckedAddFastOp::<usize, i16>::is_supported {
            return unsafe { CheckedAddFastOp::<usize, i16>::Do(self, r, out) };
        }
        if !<usize as Cast<usize>>::range(self).IsValid()
            || !<i16 as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedAddImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i16, out: &mut V) -> bool {
        if CheckedSubFastOp::<usize, i16>::is_supported {
            return unsafe { CheckedSubFastOp::<usize, i16>::Do(self, r, out) };
        }
        if !<usize as Cast<usize>>::range(self).IsValid()
            || !<i16 as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedSubImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i16, out: &mut V) -> bool {
        if CheckedMulFastOp::<usize, i16>::is_supported {
            return unsafe { CheckedMulFastOp::<usize, i16>::Do(self, r, out) };
        }
        if (!<usize as Cast<usize>>::range(self).IsValid()
            || !<i16 as Cast<usize>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = if CheckedMulFastOp::<usize, usize>::is_supported {
            unsafe { CheckedMulFastOp::<usize, usize>::Do(self as usize, r as usize, &mut result) }
        } else {
            usize_ops::CheckedMulImpl(self as usize, r as usize, &mut result)
        };
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<usize>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as i16)
        {
            return false;
        }
        if (!<usize as Cast<usize>>::range(self).IsValid()
            || !<i16 as Cast<usize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as usize) / (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i16, V> for usize
where
    usize: Cast<V>,
{
    fn modulo(self, r: i16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<usize>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as i16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as usize) % (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u16, V> for usize
where
    usize: BuiltinTriple<u16, V>,
    usize: Cast<V>,
    usize: Cast<V>,
    usize: Cast<V>,
{
    fn add(self, r: u16, out: &mut V) -> bool {
        if CheckedAddFastOp::<usize, u16>::is_supported {
            return unsafe { CheckedAddFastOp::<usize, u16>::Do(self, r, out) };
        }
        if !<usize as Cast<usize>>::range(self).IsValid()
            || !<u16 as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedAddImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u16, out: &mut V) -> bool {
        if CheckedSubFastOp::<usize, u16>::is_supported {
            return unsafe { CheckedSubFastOp::<usize, u16>::Do(self, r, out) };
        }
        if !<usize as Cast<usize>>::range(self).IsValid()
            || !<u16 as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedSubImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u16, out: &mut V) -> bool {
        if CheckedMulFastOp::<usize, u16>::is_supported {
            return unsafe { CheckedMulFastOp::<usize, u16>::Do(self, r, out) };
        }
        if (!<usize as Cast<usize>>::range(self).IsValid()
            || !<u16 as Cast<usize>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = if CheckedMulFastOp::<usize, usize>::is_supported {
            unsafe { CheckedMulFastOp::<usize, usize>::Do(self as usize, r as usize, &mut result) }
        } else {
            usize_ops::CheckedMulImpl(self as usize, r as usize, &mut result)
        };
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<usize>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as u16)
        {
            return false;
        }
        if (!<usize as Cast<usize>>::range(self).IsValid()
            || !<u16 as Cast<usize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as usize) / (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u16, V> for usize
where
    usize: Cast<V>,
{
    fn modulo(self, r: u16, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<usize>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as u16)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as usize) % (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i32, V> for usize
where
    usize: BuiltinTriple<i32, V>,
    usize: Cast<V>,
    usize: Cast<V>,
    usize: Cast<V>,
{
    fn add(self, r: i32, out: &mut V) -> bool {
        if CheckedAddFastOp::<usize, i32>::is_supported {
            return unsafe { CheckedAddFastOp::<usize, i32>::Do(self, r, out) };
        }
        if !<usize as Cast<usize>>::range(self).IsValid()
            || !<i32 as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedAddImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i32, out: &mut V) -> bool {
        if CheckedSubFastOp::<usize, i32>::is_supported {
            return unsafe { CheckedSubFastOp::<usize, i32>::Do(self, r, out) };
        }
        if !<usize as Cast<usize>>::range(self).IsValid()
            || !<i32 as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedSubImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i32, out: &mut V) -> bool {
        if CheckedMulFastOp::<usize, i32>::is_supported {
            return unsafe { CheckedMulFastOp::<usize, i32>::Do(self, r, out) };
        }
        if (!<usize as Cast<usize>>::range(self).IsValid()
            || !<i32 as Cast<usize>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = if CheckedMulFastOp::<usize, usize>::is_supported {
            unsafe { CheckedMulFastOp::<usize, usize>::Do(self as usize, r as usize, &mut result) }
        } else {
            usize_ops::CheckedMulImpl(self as usize, r as usize, &mut result)
        };
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<usize>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as i32)
        {
            return false;
        }
        if (!<usize as Cast<usize>>::range(self).IsValid()
            || !<i32 as Cast<usize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as usize) / (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i32, V> for usize
where
    usize: Cast<V>,
{
    fn modulo(self, r: i32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<usize>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as i32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as usize) % (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u32, V> for usize
where
    usize: BuiltinTriple<u32, V>,
    usize: Cast<V>,
    usize: Cast<V>,
    usize: Cast<V>,
{
    fn add(self, r: u32, out: &mut V) -> bool {
        if CheckedAddFastOp::<usize, u32>::is_supported {
            return unsafe { CheckedAddFastOp::<usize, u32>::Do(self, r, out) };
        }
        if !<usize as Cast<usize>>::range(self).IsValid()
            || !<u32 as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedAddImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u32, out: &mut V) -> bool {
        if CheckedSubFastOp::<usize, u32>::is_supported {
            return unsafe { CheckedSubFastOp::<usize, u32>::Do(self, r, out) };
        }
        if !<usize as Cast<usize>>::range(self).IsValid()
            || !<u32 as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedSubImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u32, out: &mut V) -> bool {
        if CheckedMulFastOp::<usize, u32>::is_supported {
            return unsafe { CheckedMulFastOp::<usize, u32>::Do(self, r, out) };
        }
        if (!<usize as Cast<usize>>::range(self).IsValid()
            || !<u32 as Cast<usize>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = if CheckedMulFastOp::<usize, usize>::is_supported {
            unsafe { CheckedMulFastOp::<usize, usize>::Do(self as usize, r as usize, &mut result) }
        } else {
            usize_ops::CheckedMulImpl(self as usize, r as usize, &mut result)
        };
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<usize>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as u32)
        {
            return false;
        }
        if (!<usize as Cast<usize>>::range(self).IsValid()
            || !<u32 as Cast<usize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as usize) / (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u32, V> for usize
where
    usize: Cast<V>,
{
    fn modulo(self, r: u32, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<usize>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as u32)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as usize) % (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<i64, V> for usize
where
    usize: BuiltinTriple<i64, V>,
    usize: Cast<V>,
    usize: Cast<V>,
    usize: Cast<V>,
{
    fn add(self, r: i64, out: &mut V) -> bool {
        if CheckedAddFastOp::<usize, i64>::is_supported {
            return unsafe { CheckedAddFastOp::<usize, i64>::Do(self, r, out) };
        }
        if !<usize as Cast<usize>>::range(self).IsValid()
            || !<i64 as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedAddImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: i64, out: &mut V) -> bool {
        if CheckedSubFastOp::<usize, i64>::is_supported {
            return unsafe { CheckedSubFastOp::<usize, i64>::Do(self, r, out) };
        }
        if !<usize as Cast<usize>>::range(self).IsValid()
            || !<i64 as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedSubImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: i64, out: &mut V) -> bool {
        if CheckedMulFastOp::<usize, i64>::is_supported {
            return unsafe { CheckedMulFastOp::<usize, i64>::Do(self, r, out) };
        }
        if (!<usize as Cast<usize>>::range(self).IsValid()
            || !<i64 as Cast<usize>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = if CheckedMulFastOp::<usize, usize>::is_supported {
            unsafe { CheckedMulFastOp::<usize, usize>::Do(self as usize, r as usize, &mut result) }
        } else {
            usize_ops::CheckedMulImpl(self as usize, r as usize, &mut result)
        };
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<usize>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as i64)
        {
            return false;
        }
        if (!<usize as Cast<usize>>::range(self).IsValid()
            || !<i64 as Cast<usize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as usize) / (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<i64, V> for usize
where
    usize: Cast<V>,
{
    fn modulo(self, r: i64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<usize>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as i64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as usize) % (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<u64, V> for usize
where
    usize: BuiltinTriple<u64, V>,
    u64: Cast<V>,
    u64: Cast<V>,
    u64: Cast<V>,
{
    fn add(self, r: u64, out: &mut V) -> bool {
        if CheckedAddFastOp::<usize, u64>::is_supported {
            return unsafe { CheckedAddFastOp::<usize, u64>::Do(self, r, out) };
        }
        if !<usize as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedAddImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: u64, out: &mut V) -> bool {
        if CheckedSubFastOp::<usize, u64>::is_supported {
            return unsafe { CheckedSubFastOp::<usize, u64>::Do(self, r, out) };
        }
        if !<usize as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid() {
            return false;
        }
        let mut result = 0 as u64;
        let valid = u64_ops::CheckedSubImpl(self as u64, r as u64, &mut result);
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: u64, out: &mut V) -> bool {
        if CheckedMulFastOp::<usize, u64>::is_supported {
            return unsafe { CheckedMulFastOp::<usize, u64>::Do(self, r, out) };
        }
        if (!<usize as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as u64;
        let valid = if CheckedMulFastOp::<u64, u64>::is_supported {
            unsafe { CheckedMulFastOp::<u64, u64>::Do(self as u64, r as u64, &mut result) }
        } else {
            u64_ops::CheckedMulImpl(self as u64, r as u64, &mut result)
        };
        if !valid || !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<usize>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u64)
        {
            return false;
        }
        if (!<usize as Cast<u64>>::range(self).IsValid() || !<u64 as Cast<u64>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as u64) / (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<u64, V> for usize
where
    u64: Cast<V>,
{
    fn modulo(self, r: u64, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <u64 as Cast<usize>>::CONTAINED
            && (self as u64) == u64::MIN
            && r == (-1i64 as u64)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as u64) % (r as u64);
        if !<u64 as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <u64 as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<isize, V> for usize
where
    usize: BuiltinTriple<isize, V>,
    usize: Cast<V>,
    usize: Cast<V>,
    usize: Cast<V>,
{
    fn add(self, r: isize, out: &mut V) -> bool {
        if CheckedAddFastOp::<usize, isize>::is_supported {
            return unsafe { CheckedAddFastOp::<usize, isize>::Do(self, r, out) };
        }
        if !<usize as Cast<usize>>::range(self).IsValid()
            || !<isize as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedAddImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: isize, out: &mut V) -> bool {
        if CheckedSubFastOp::<usize, isize>::is_supported {
            return unsafe { CheckedSubFastOp::<usize, isize>::Do(self, r, out) };
        }
        if !<usize as Cast<usize>>::range(self).IsValid()
            || !<isize as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedSubImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: isize, out: &mut V) -> bool {
        if CheckedMulFastOp::<usize, isize>::is_supported {
            return unsafe { CheckedMulFastOp::<usize, isize>::Do(self, r, out) };
        }
        if (!<usize as Cast<usize>>::range(self).IsValid()
            || !<isize as Cast<usize>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = if CheckedMulFastOp::<usize, usize>::is_supported {
            unsafe { CheckedMulFastOp::<usize, usize>::Do(self as usize, r as usize, &mut result) }
        } else {
            usize_ops::CheckedMulImpl(self as usize, r as usize, &mut result)
        };
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<usize>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as isize)
        {
            return false;
        }
        if (!<usize as Cast<usize>>::range(self).IsValid()
            || !<isize as Cast<usize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as usize) / (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<isize, V> for usize
where
    usize: Cast<V>,
{
    fn modulo(self, r: isize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<usize>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as isize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as usize) % (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedBinary<usize, V> for usize
where
    usize: BuiltinTriple<usize, V>,
    usize: Cast<V>,
    usize: Cast<V>,
    usize: Cast<V>,
{
    fn add(self, r: usize, out: &mut V) -> bool {
        if CheckedAddFastOp::<usize, usize>::is_supported {
            return unsafe { CheckedAddFastOp::<usize, usize>::Do(self, r, out) };
        }
        if !<usize as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedAddImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn sub(self, r: usize, out: &mut V) -> bool {
        if CheckedSubFastOp::<usize, usize>::is_supported {
            return unsafe { CheckedSubFastOp::<usize, usize>::Do(self, r, out) };
        }
        if !<usize as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid()
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = usize_ops::CheckedSubImpl(self as usize, r as usize, &mut result);
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn mul(self, r: usize, out: &mut V) -> bool {
        if CheckedMulFastOp::<usize, usize>::is_supported {
            return unsafe { CheckedMulFastOp::<usize, usize>::Do(self, r, out) };
        }
        if (!<usize as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid())
            && self != 0
            && r != 0
        {
            return false;
        }
        let mut result = 0 as usize;
        let valid = if CheckedMulFastOp::<usize, usize>::is_supported {
            unsafe { CheckedMulFastOp::<usize, usize>::Do(self as usize, r as usize, &mut result) }
        } else {
            usize_ops::CheckedMulImpl(self as usize, r as usize, &mut result)
        };
        if !valid || !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
    fn div(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<usize>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as usize)
        {
            return false;
        }
        if (!<usize as Cast<usize>>::range(self).IsValid()
            || !<usize as Cast<usize>>::range(r).IsValid())
            && self != 0
        {
            return false;
        }
        let result = (self as usize) / (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
#[cfg(target_pointer_width = "64")]
impl<V: Numeric> CheckedModulo<usize, V> for usize
where
    usize: Cast<V>,
{
    fn modulo(self, r: usize, out: &mut V) -> bool {
        if r == 0 {
            return false;
        }
        if false
            && <usize as Cast<usize>>::CONTAINED
            && (self as usize) == usize::MIN
            && r == (-1i64 as usize)
        {
            *out = V::ZERO;
            return true;
        }
        let result = (self as usize) % (r as usize);
        if !<usize as Cast<V>>::range(result).IsValid() {
            return false;
        }
        *out = <usize as Cast<V>>::cast(result);
        true
    }
}
// Source: original shift template constraints; normalization preserves every
// in-range count, and negative/oversized counts remain out of bounds.
pub trait ShiftAmount: Numeric {
    fn shift_amount(self) -> u64;
}
pub trait UnsignedShiftAmount: ShiftAmount {}
macro_rules! shift_signed{($($t:ty),*)=>{$(impl ShiftAmount for $t{fn shift_amount(self)->u64{self as u64}})*}}
shift_signed!(i8, i16, i32, i64, isize);
macro_rules! shift_unsigned{($($t:ty),*)=>{$(impl ShiftAmount for $t{fn shift_amount(self)->u64{self as u64}}impl UnsignedShiftAmount for $t{})*}}
shift_unsigned!(u8, u16, u32, u64, usize);

// Source: checked_math_impl.h integer state constructors/accessors, concrete const domains.
impl IntegerState<i8> {
    pub(crate) const fn const_from(value: i8, valid: bool) -> Self {
        Self {
            is_valid_: valid,
            value_: value,
        }
    }
    pub(crate) const fn const_value(self) -> i8 {
        self.value_
    }
    pub(crate) const fn const_valid(self) -> bool {
        self.is_valid_
    }
}
impl IntegerState<u8> {
    pub(crate) const fn const_from(value: u8, valid: bool) -> Self {
        Self {
            is_valid_: valid,
            value_: value,
        }
    }
    pub(crate) const fn const_value(self) -> u8 {
        self.value_
    }
    pub(crate) const fn const_valid(self) -> bool {
        self.is_valid_
    }
}
impl IntegerState<i16> {
    pub(crate) const fn const_from(value: i16, valid: bool) -> Self {
        Self {
            is_valid_: valid,
            value_: value,
        }
    }
    pub(crate) const fn const_value(self) -> i16 {
        self.value_
    }
    pub(crate) const fn const_valid(self) -> bool {
        self.is_valid_
    }
}
impl IntegerState<u16> {
    pub(crate) const fn const_from(value: u16, valid: bool) -> Self {
        Self {
            is_valid_: valid,
            value_: value,
        }
    }
    pub(crate) const fn const_value(self) -> u16 {
        self.value_
    }
    pub(crate) const fn const_valid(self) -> bool {
        self.is_valid_
    }
}
impl IntegerState<i32> {
    pub(crate) const fn const_from(value: i32, valid: bool) -> Self {
        Self {
            is_valid_: valid,
            value_: value,
        }
    }
    pub(crate) const fn const_value(self) -> i32 {
        self.value_
    }
    pub(crate) const fn const_valid(self) -> bool {
        self.is_valid_
    }
}
impl IntegerState<u32> {
    pub(crate) const fn const_from(value: u32, valid: bool) -> Self {
        Self {
            is_valid_: valid,
            value_: value,
        }
    }
    pub(crate) const fn const_value(self) -> u32 {
        self.value_
    }
    pub(crate) const fn const_valid(self) -> bool {
        self.is_valid_
    }
}
impl IntegerState<i64> {
    pub(crate) const fn const_from(value: i64, valid: bool) -> Self {
        Self {
            is_valid_: valid,
            value_: value,
        }
    }
    pub(crate) const fn const_value(self) -> i64 {
        self.value_
    }
    pub(crate) const fn const_valid(self) -> bool {
        self.is_valid_
    }
}
impl IntegerState<u64> {
    pub(crate) const fn const_from(value: u64, valid: bool) -> Self {
        Self {
            is_valid_: valid,
            value_: value,
        }
    }
    pub(crate) const fn const_value(self) -> u64 {
        self.value_
    }
    pub(crate) const fn const_valid(self) -> bool {
        self.is_valid_
    }
}
impl IntegerState<isize> {
    pub(crate) const fn const_from(value: isize, valid: bool) -> Self {
        Self {
            is_valid_: valid,
            value_: value,
        }
    }
    pub(crate) const fn const_value(self) -> isize {
        self.value_
    }
    pub(crate) const fn const_valid(self) -> bool {
        self.is_valid_
    }
}
impl IntegerState<usize> {
    pub(crate) const fn const_from(value: usize, valid: bool) -> Self {
        Self {
            is_valid_: valid,
            value_: value,
        }
    }
    pub(crate) const fn const_value(self) -> usize {
        self.value_
    }
    pub(crate) const fn const_valid(self) -> bool {
        self.is_valid_
    }
}
