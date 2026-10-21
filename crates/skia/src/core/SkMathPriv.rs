// Source-aligned Skia translation. Rust methods use snake_case; fields retain upstream names.
#![allow(non_snake_case)]

macro_rules! native_const_assert {
 ($condition:expr,$line:expr,$text:expr)=>{{
  #[cfg(sk_debug)] {
   const fn at_const(value:bool){assert!(value);}
   fn at_runtime(value:bool){crate::SkASSERT!(@source b"src/core/SkMathPriv.h\0".as_ptr(),$line; @condition $text; value);}
   core::intrinsics::const_eval_select(($condition,),at_const,at_runtime)
  }
 }};
}
// Source: src/core/SkMathPriv.h:22-24
#[inline]
pub const fn SkCLZ(x: u32) -> i32 {
    x.leading_zeros() as i32
}

// Source: src/core/SkMathPriv.h:29-31
#[inline]
pub const fn SkCTZ(x: u32) -> i32 {
    x.trailing_zeros() as i32
}

// Source: src/core/SkMathPriv.h:36-38
#[inline]
pub const fn SkPopCount(x: u32) -> i32 {
    x.count_ones() as i32
}

// Source: src/core/SkMathPriv.h:47-47
#[inline]
pub fn SkSqrt32(n: i32) -> i32 {
    SkSqrtBits(n, 15)
}

// Source: src/core/SkMathPriv.h:52-54
#[inline]
pub const fn SkClampPos(value: i32) -> i32 {
    value & !(value >> 31)
}

// Source: src/core/SkMathPriv.h:59-63
// Source: src/core/SkMathPriv.h:59-63. Built-in promotion and genuine class
// overloads share the same operator protocol; each full expression ends before
// the following output assignment begins.
#[inline]
pub fn SkTDivMod<In, Out>(numer: In, denom: In, div: &mut Out, modulo: &mut Out)
where
    In: SkCppDiv<In> + SkCppRem<In>,
    <In as SkCppDiv<In>>::Output: SkCppConvertTo<Out>,
    <In as SkCppRem<In>>::Output: SkCppConvertTo<Out>,
    Out: SkCppAssignFrom<Out>,
{
    {
        let quotient = numer.cpp_div(&denom);
        div.cpp_assign_from(quotient.cpp_convert_to());
    }
    {
        let remainder = numer.cpp_rem(&denom);
        modulo.cpp_assign_from(remainder.cpp_convert_to());
    }
}
// Native pointer outputs may alias; only one short assignment borrow exists at
// each original statement. Division/remainder native defined-domain preconditions apply.
pub unsafe fn SkTDivMod_raw<In, Out>(numer: In, denom: In, div: *mut Out, modulo: *mut Out)
where
    In: SkCppDiv<In> + SkCppRem<In>,
    <In as SkCppDiv<In>>::Output: SkCppConvertTo<Out>,
    <In as SkCppRem<In>>::Output: SkCppConvertTo<Out>,
    Out: SkCppAssignFrom<Out>,
{
    {
        let quotient = numer.cpp_div(&denom);
        unsafe {
            (&mut *div).cpp_assign_from(quotient.cpp_convert_to());
        }
    }
    {
        let remainder = numer.cpp_rem(&denom);
        unsafe {
            (&mut *modulo).cpp_assign_from(remainder.cpp_convert_to());
        }
    }
}

// Source: src/core/SkMathPriv.h:67-67
#[inline]
pub const fn SkExtractSign(n: i32) -> i32 {
    n >> 31
}

// Source: src/core/SkMathPriv.h:72-75
#[inline]
pub fn SkApplySign(n: i32, sign: i32) -> i32 {
    crate::SkASSERT!(@source b"src/core/SkMathPriv.h\0".as_ptr(),73; @condition b"sign == 0 || sign == -1\0".as_ptr(); sign == 0 || sign == -1);
    (n ^ sign).wrapping_sub(sign)
}

// Source: src/core/SkMathPriv.h:78-80
#[inline]
pub fn SkCopySign32(x: i32, y: i32) -> i32 {
    SkApplySign(x, SkExtractSign(x ^ y))
}

// Source: src/core/SkMathPriv.h:87-92
#[inline]
pub fn SkClampUMax(mut value: u32, max: u32) -> u32 {
    if value > max {
        value = max;
    }
    value
}

// Source: src/core/SkMathPriv.h:98-107
#[inline]
pub fn sk_negate_to_size_t(value: i32) -> usize {
    (value as usize).wrapping_neg()
}

// Source: src/core/SkMathPriv.h:114-119
#[inline]
pub fn SkMulDiv255Trunc(a: u32, b: u32) -> u32 {
    crate::SkASSERT!(@source b"src/core/SkMathPriv.h\0".as_ptr(),115; @condition b"(uint8_t)a == a\0".as_ptr();a as u8 as u32==a);
    crate::SkASSERT!(@source b"src/core/SkMathPriv.h\0".as_ptr(),116; @condition b"(uint8_t)b == b\0".as_ptr();b as u8 as u32==b);
    let prod = a.wrapping_mul(b).wrapping_add(1);
    prod.wrapping_add(prod >> 8) >> 8
}

// Source: src/core/SkMathPriv.h:124-129
#[inline]
pub fn SkMulDiv255Ceiling(a: u32, b: u32) -> u32 {
    crate::SkASSERT!(@source b"src/core/SkMathPriv.h\0".as_ptr(),125; @condition b"(uint8_t)a == a\0".as_ptr();a as u8 as u32==a);
    crate::SkASSERT!(@source b"src/core/SkMathPriv.h\0".as_ptr(),126; @condition b"(uint8_t)b == b\0".as_ptr();b as u8 as u32==b);
    let prod = a.wrapping_mul(b).wrapping_add(255);
    prod.wrapping_add(prod >> 8) >> 8
}

// Source: src/core/SkMathPriv.h:133-136
#[inline]
pub fn SkDiv255Round(mut prod: u32) -> u32 {
    prod = prod.wrapping_add(128);
    prod.wrapping_add(prod >> 8) >> 8
}

// Source: src/core/SkMathPriv.h:143-145
// Both native compiler intrinsics implement the same 32-bit permutation; Rust lowers this intrinsic for the active target.
#[inline]
pub const fn SkBSwap32(v: u32) -> u32 {
    v.swap_bytes()
}

// Source: src/core/SkMathPriv.h:157-160
#[inline]
pub const fn SkNextLog2(value: u32) -> i32 {
    native_const_assert!(value != 0, 158, b"value != 0\0".as_ptr());
    32 - SkCLZ(value.wrapping_sub(1))
}

// Source: src/core/SkMathPriv.h:171-174
#[inline]
pub const fn SkPrevLog2(value: u32) -> i32 {
    native_const_assert!(value != 0, 172, b"value != 0\0".as_ptr());
    32 - SkCLZ(value >> 1)
}

// Source: src/core/SkMathPriv.h:181-184
#[inline]
pub const fn SkNextPow2(value: i32) -> i32 {
    native_const_assert!(value > 0, 182, b"value > 0\0".as_ptr());
    1i32.wrapping_shl(SkNextLog2(value as u32) as u32)
}

// Source: src/core/SkMathPriv.h:191-194
#[inline]
pub const fn SkPrevPow2(value: i32) -> i32 {
    native_const_assert!(value > 0, 192, b"value > 0\0".as_ptr());
    1i32.wrapping_shl(SkPrevLog2(value as u32) as u32)
}

// Source: src/core/SkMathPriv.h:201-218
#[inline]
pub const fn SkNextSizePow2(mut n: usize) -> usize {
    let num_size_t_bits = usize::BITS;
    let high_bit_set = 1usize << (num_size_t_bits - 1);
    if n == 0 {
        return 1;
    } else if n >= high_bit_set {
        return n;
    }
    n -= 1;
    let mut shift = 1;
    while shift < num_size_t_bits {
        n |= n >> shift;
        shift <<= 1;
    }
    n + 1
}

// Source: src/core/SkMathPriv.h:221-223
pub trait SkFixedCandidate:
    SkCppLt<i32> + SkCppNeg + SkCppLe<f32> + SkCppAssignFrom<<Self as SkCppNeg>::Output>
{
}
impl<T> SkFixedCandidate for T where
    T: SkCppLt<i32> + SkCppNeg + SkCppLe<f32> + SkCppAssignFrom<<T as SkCppNeg>::Output>
{
}
// Source: include/private/SkTemplates.h:43-48; src/core/SkMathPriv.h:221-223.
#[inline]
pub fn SkFitsInFixed<T: SkFixedCandidate>(mut x: T) -> bool {
    if x.cpp_lt(&0) {
        let negated = x.cpp_neg();
        x.cpp_assign_from(negated);
    }
    x.cpp_le(&32767.0f32)
}

// Source: src/core/SkMathPriv.cpp:18-39
pub fn SkSqrtBits(x: i32, mut count: i32) -> i32 {
    debug_assert!(x >= 0 && count > 0 && count as u32 <= 30);
    let mut root = 0u32;
    let mut rem_hi = 0u32;
    let mut rem_lo = x as u32;
    loop {
        root = root.wrapping_shl(1);
        rem_hi = rem_hi.wrapping_shl(2) | (rem_lo >> 30);
        rem_lo = rem_lo.wrapping_shl(2);
        let test_div = root.wrapping_shl(1).wrapping_add(1);
        if rem_hi >= test_div {
            rem_hi -= test_div;
            root += 1;
        }
        count -= 1;
        if count < 0 {
            break;
        }
    }
    root as i32
}

// Source: src/core/SkMathPriv.h:59-63,221-223; include/private/SkFixed.h:41-94,134-141.
// Actual C++ expression language protocol. User-defined source classes implement
// their real operator and conversion members here; the math helper never supplies
// a default operator, normalized operand or substitute result for such a class.
pub const trait SkCppAdd<Rhs> {
    type Output;
    fn cpp_add(&self, rhs: &Rhs) -> Self::Output;
}
pub const trait SkCppSub<Rhs> {
    type Output;
    fn cpp_sub(&self, rhs: &Rhs) -> Self::Output;
}
pub const trait SkCppMul<Rhs> {
    type Output;
    fn cpp_mul(&self, rhs: &Rhs) -> Self::Output;
}
pub const trait SkCppDiv<Rhs> {
    type Output;
    fn cpp_div(&self, rhs: &Rhs) -> Self::Output;
}
pub const trait SkCppRem<Rhs> {
    type Output;
    fn cpp_rem(&self, rhs: &Rhs) -> Self::Output;
}
pub const trait SkCppShl<Rhs> {
    type Output;
    fn cpp_shl(&self, rhs: &Rhs) -> Self::Output;
}
pub const trait SkCppShr<Rhs> {
    type Output;
    fn cpp_shr(&self, rhs: &Rhs) -> Self::Output;
}
pub const trait SkCppLt<Rhs> {
    fn cpp_lt(&self, rhs: &Rhs) -> bool;
}
pub const trait SkCppLe<Rhs> {
    fn cpp_le(&self, rhs: &Rhs) -> bool;
}
pub const trait SkCppNeg {
    type Output;
    fn cpp_neg(&self) -> Self::Output;
}
macro_rules! cpp_arithmetic_pair {
 ($l:ty,$r:ty=>$out:ty,$lv:ident,$rv:ident,$left:expr,$right:expr,integer)=>{
  cpp_arithmetic_pair!($l,$r=>$out,$lv,$rv,$left,$right,common);
  const impl SkCppAdd<$r> for $l{type Output=$out;fn cpp_add(&self,rhs:&$r)->$out{let $lv=*self;let $rv=*rhs;($left).wrapping_add($right)}}
  const impl SkCppSub<$r> for $l{type Output=$out;fn cpp_sub(&self,rhs:&$r)->$out{let $lv=*self;let $rv=*rhs;($left).wrapping_sub($right)}}
  const impl SkCppMul<$r> for $l{type Output=$out;fn cpp_mul(&self,rhs:&$r)->$out{let $lv=*self;let $rv=*rhs;($left).wrapping_mul($right)}}
  const impl SkCppRem<$r> for $l{type Output=$out;fn cpp_rem(&self,rhs:&$r)->$out{let $lv=*self;let $rv=*rhs;$left%$right}}
 };
 ($l:ty,$r:ty=>$out:ty,$lv:ident,$rv:ident,$left:expr,$right:expr,float)=>{
  cpp_arithmetic_pair!($l,$r=>$out,$lv,$rv,$left,$right,common);
  const impl SkCppAdd<$r> for $l{type Output=$out;fn cpp_add(&self,rhs:&$r)->$out{let $lv=*self;let $rv=*rhs;$left+$right}}
  const impl SkCppSub<$r> for $l{type Output=$out;fn cpp_sub(&self,rhs:&$r)->$out{let $lv=*self;let $rv=*rhs;$left-$right}}
  const impl SkCppMul<$r> for $l{type Output=$out;fn cpp_mul(&self,rhs:&$r)->$out{let $lv=*self;let $rv=*rhs;$left*$right}}
 };
 ($l:ty,$r:ty=>$out:ty,$lv:ident,$rv:ident,$left:expr,$right:expr,common)=>{
  const impl SkCppDiv<$r> for $l{type Output=$out;fn cpp_div(&self,rhs:&$r)->$out{let $lv=*self;let $rv=*rhs;$left/$right}}
  const impl SkCppLt<$r> for $l{fn cpp_lt(&self,rhs:&$r)->bool{let $lv=*self;let $rv=*rhs;$left<$right}}
  const impl SkCppLe<$r> for $l{fn cpp_le(&self,rhs:&$r)->bool{let $lv=*self;let $rv=*rhs;$left<=$right}}
 };
}
cpp_arithmetic_pair!(bool,bool=>i32,l,r,(l as u8 as i32),(r as u8 as i32),integer);
cpp_arithmetic_pair!(bool,i8=>i32,l,r,(l as u8 as i32),(r as i32),integer);
cpp_arithmetic_pair!(bool,u8=>i32,l,r,(l as u8 as i32),(r as i32),integer);
cpp_arithmetic_pair!(bool,i16=>i32,l,r,(l as u8 as i32),(r as i32),integer);
cpp_arithmetic_pair!(bool,u16=>i32,l,r,(l as u8 as i32),(r as i32),integer);
cpp_arithmetic_pair!(bool,i32=>i32,l,r,(l as u8 as i32),(r as i32),integer);
cpp_arithmetic_pair!(bool,u32=>u32,l,r,(l as u8 as u32),(r as u32),integer);
cpp_arithmetic_pair!(bool,i64=>i64,l,r,(l as u8 as i64),(r as i64),integer);
cpp_arithmetic_pair!(bool,u64=>u64,l,r,(l as u8 as u64),(r as u64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(bool,isize=>i32,l,r,(l as u8 as i32),(r as i32),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(bool,isize=>i64,l,r,(l as u8 as i64),(r as i64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(bool,usize=>u32,l,r,(l as u8 as u32),(r as u32),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(bool,usize=>u64,l,r,(l as u8 as u64),(r as u64),integer);
cpp_arithmetic_pair!(bool,f32=>f32,l,r,(l as u8 as f32),(r as f32),float);
cpp_arithmetic_pair!(bool,f64=>f64,l,r,(l as u8 as f64),(r as f64),float);
cpp_arithmetic_pair!(i8,bool=>i32,l,r,(l as i32),(r as u8 as i32),integer);
cpp_arithmetic_pair!(i8,i8=>i32,l,r,(l as i32),(r as i32),integer);
cpp_arithmetic_pair!(i8,u8=>i32,l,r,(l as i32),(r as i32),integer);
cpp_arithmetic_pair!(i8,i16=>i32,l,r,(l as i32),(r as i32),integer);
cpp_arithmetic_pair!(i8,u16=>i32,l,r,(l as i32),(r as i32),integer);
cpp_arithmetic_pair!(i8,i32=>i32,l,r,(l as i32),(r as i32),integer);
cpp_arithmetic_pair!(i8,u32=>u32,l,r,(l as u32),(r as u32),integer);
cpp_arithmetic_pair!(i8,i64=>i64,l,r,(l as i64),(r as i64),integer);
cpp_arithmetic_pair!(i8,u64=>u64,l,r,(l as u64),(r as u64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(i8,isize=>i32,l,r,(l as i32),(r as i32),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(i8,isize=>i64,l,r,(l as i64),(r as i64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(i8,usize=>u32,l,r,(l as u32),(r as u32),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(i8,usize=>u64,l,r,(l as u64),(r as u64),integer);
cpp_arithmetic_pair!(i8,f32=>f32,l,r,(l as f32),(r as f32),float);
cpp_arithmetic_pair!(i8,f64=>f64,l,r,(l as f64),(r as f64),float);
cpp_arithmetic_pair!(u8,bool=>i32,l,r,(l as i32),(r as u8 as i32),integer);
cpp_arithmetic_pair!(u8,i8=>i32,l,r,(l as i32),(r as i32),integer);
cpp_arithmetic_pair!(u8,u8=>i32,l,r,(l as i32),(r as i32),integer);
cpp_arithmetic_pair!(u8,i16=>i32,l,r,(l as i32),(r as i32),integer);
cpp_arithmetic_pair!(u8,u16=>i32,l,r,(l as i32),(r as i32),integer);
cpp_arithmetic_pair!(u8,i32=>i32,l,r,(l as i32),(r as i32),integer);
cpp_arithmetic_pair!(u8,u32=>u32,l,r,(l as u32),(r as u32),integer);
cpp_arithmetic_pair!(u8,i64=>i64,l,r,(l as i64),(r as i64),integer);
cpp_arithmetic_pair!(u8,u64=>u64,l,r,(l as u64),(r as u64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(u8,isize=>i32,l,r,(l as i32),(r as i32),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(u8,isize=>i64,l,r,(l as i64),(r as i64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(u8,usize=>u32,l,r,(l as u32),(r as u32),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(u8,usize=>u64,l,r,(l as u64),(r as u64),integer);
cpp_arithmetic_pair!(u8,f32=>f32,l,r,(l as f32),(r as f32),float);
cpp_arithmetic_pair!(u8,f64=>f64,l,r,(l as f64),(r as f64),float);
cpp_arithmetic_pair!(i16,bool=>i32,l,r,(l as i32),(r as u8 as i32),integer);
cpp_arithmetic_pair!(i16,i8=>i32,l,r,(l as i32),(r as i32),integer);
cpp_arithmetic_pair!(i16,u8=>i32,l,r,(l as i32),(r as i32),integer);
cpp_arithmetic_pair!(i16,i16=>i32,l,r,(l as i32),(r as i32),integer);
cpp_arithmetic_pair!(i16,u16=>i32,l,r,(l as i32),(r as i32),integer);
cpp_arithmetic_pair!(i16,i32=>i32,l,r,(l as i32),(r as i32),integer);
cpp_arithmetic_pair!(i16,u32=>u32,l,r,(l as u32),(r as u32),integer);
cpp_arithmetic_pair!(i16,i64=>i64,l,r,(l as i64),(r as i64),integer);
cpp_arithmetic_pair!(i16,u64=>u64,l,r,(l as u64),(r as u64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(i16,isize=>i32,l,r,(l as i32),(r as i32),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(i16,isize=>i64,l,r,(l as i64),(r as i64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(i16,usize=>u32,l,r,(l as u32),(r as u32),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(i16,usize=>u64,l,r,(l as u64),(r as u64),integer);
cpp_arithmetic_pair!(i16,f32=>f32,l,r,(l as f32),(r as f32),float);
cpp_arithmetic_pair!(i16,f64=>f64,l,r,(l as f64),(r as f64),float);
cpp_arithmetic_pair!(u16,bool=>i32,l,r,(l as i32),(r as u8 as i32),integer);
cpp_arithmetic_pair!(u16,i8=>i32,l,r,(l as i32),(r as i32),integer);
cpp_arithmetic_pair!(u16,u8=>i32,l,r,(l as i32),(r as i32),integer);
cpp_arithmetic_pair!(u16,i16=>i32,l,r,(l as i32),(r as i32),integer);
cpp_arithmetic_pair!(u16,u16=>i32,l,r,(l as i32),(r as i32),integer);
cpp_arithmetic_pair!(u16,i32=>i32,l,r,(l as i32),(r as i32),integer);
cpp_arithmetic_pair!(u16,u32=>u32,l,r,(l as u32),(r as u32),integer);
cpp_arithmetic_pair!(u16,i64=>i64,l,r,(l as i64),(r as i64),integer);
cpp_arithmetic_pair!(u16,u64=>u64,l,r,(l as u64),(r as u64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(u16,isize=>i32,l,r,(l as i32),(r as i32),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(u16,isize=>i64,l,r,(l as i64),(r as i64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(u16,usize=>u32,l,r,(l as u32),(r as u32),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(u16,usize=>u64,l,r,(l as u64),(r as u64),integer);
cpp_arithmetic_pair!(u16,f32=>f32,l,r,(l as f32),(r as f32),float);
cpp_arithmetic_pair!(u16,f64=>f64,l,r,(l as f64),(r as f64),float);
cpp_arithmetic_pair!(i32,bool=>i32,l,r,(l as i32),(r as u8 as i32),integer);
cpp_arithmetic_pair!(i32,i8=>i32,l,r,(l as i32),(r as i32),integer);
cpp_arithmetic_pair!(i32,u8=>i32,l,r,(l as i32),(r as i32),integer);
cpp_arithmetic_pair!(i32,i16=>i32,l,r,(l as i32),(r as i32),integer);
cpp_arithmetic_pair!(i32,u16=>i32,l,r,(l as i32),(r as i32),integer);
cpp_arithmetic_pair!(i32,i32=>i32,l,r,(l as i32),(r as i32),integer);
cpp_arithmetic_pair!(i32,u32=>u32,l,r,(l as u32),(r as u32),integer);
cpp_arithmetic_pair!(i32,i64=>i64,l,r,(l as i64),(r as i64),integer);
cpp_arithmetic_pair!(i32,u64=>u64,l,r,(l as u64),(r as u64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(i32,isize=>i32,l,r,(l as i32),(r as i32),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(i32,isize=>i64,l,r,(l as i64),(r as i64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(i32,usize=>u32,l,r,(l as u32),(r as u32),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(i32,usize=>u64,l,r,(l as u64),(r as u64),integer);
cpp_arithmetic_pair!(i32,f32=>f32,l,r,(l as f32),(r as f32),float);
cpp_arithmetic_pair!(i32,f64=>f64,l,r,(l as f64),(r as f64),float);
cpp_arithmetic_pair!(u32,bool=>u32,l,r,(l as u32),(r as u8 as u32),integer);
cpp_arithmetic_pair!(u32,i8=>u32,l,r,(l as u32),(r as u32),integer);
cpp_arithmetic_pair!(u32,u8=>u32,l,r,(l as u32),(r as u32),integer);
cpp_arithmetic_pair!(u32,i16=>u32,l,r,(l as u32),(r as u32),integer);
cpp_arithmetic_pair!(u32,u16=>u32,l,r,(l as u32),(r as u32),integer);
cpp_arithmetic_pair!(u32,i32=>u32,l,r,(l as u32),(r as u32),integer);
cpp_arithmetic_pair!(u32,u32=>u32,l,r,(l as u32),(r as u32),integer);
cpp_arithmetic_pair!(u32,i64=>i64,l,r,(l as i64),(r as i64),integer);
cpp_arithmetic_pair!(u32,u64=>u64,l,r,(l as u64),(r as u64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(u32,isize=>u32,l,r,(l as u32),(r as u32),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(u32,isize=>i64,l,r,(l as i64),(r as i64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(u32,usize=>u32,l,r,(l as u32),(r as u32),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(u32,usize=>u64,l,r,(l as u64),(r as u64),integer);
cpp_arithmetic_pair!(u32,f32=>f32,l,r,(l as f32),(r as f32),float);
cpp_arithmetic_pair!(u32,f64=>f64,l,r,(l as f64),(r as f64),float);
cpp_arithmetic_pair!(i64,bool=>i64,l,r,(l as i64),(r as u8 as i64),integer);
cpp_arithmetic_pair!(i64,i8=>i64,l,r,(l as i64),(r as i64),integer);
cpp_arithmetic_pair!(i64,u8=>i64,l,r,(l as i64),(r as i64),integer);
cpp_arithmetic_pair!(i64,i16=>i64,l,r,(l as i64),(r as i64),integer);
cpp_arithmetic_pair!(i64,u16=>i64,l,r,(l as i64),(r as i64),integer);
cpp_arithmetic_pair!(i64,i32=>i64,l,r,(l as i64),(r as i64),integer);
cpp_arithmetic_pair!(i64,u32=>i64,l,r,(l as i64),(r as i64),integer);
cpp_arithmetic_pair!(i64,i64=>i64,l,r,(l as i64),(r as i64),integer);
cpp_arithmetic_pair!(i64,u64=>u64,l,r,(l as u64),(r as u64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(i64,isize=>i64,l,r,(l as i64),(r as i64),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(i64,isize=>i64,l,r,(l as i64),(r as i64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(i64,usize=>i64,l,r,(l as i64),(r as i64),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(i64,usize=>u64,l,r,(l as u64),(r as u64),integer);
cpp_arithmetic_pair!(i64,f32=>f32,l,r,(l as f32),(r as f32),float);
cpp_arithmetic_pair!(i64,f64=>f64,l,r,(l as f64),(r as f64),float);
cpp_arithmetic_pair!(u64,bool=>u64,l,r,(l as u64),(r as u8 as u64),integer);
cpp_arithmetic_pair!(u64,i8=>u64,l,r,(l as u64),(r as u64),integer);
cpp_arithmetic_pair!(u64,u8=>u64,l,r,(l as u64),(r as u64),integer);
cpp_arithmetic_pair!(u64,i16=>u64,l,r,(l as u64),(r as u64),integer);
cpp_arithmetic_pair!(u64,u16=>u64,l,r,(l as u64),(r as u64),integer);
cpp_arithmetic_pair!(u64,i32=>u64,l,r,(l as u64),(r as u64),integer);
cpp_arithmetic_pair!(u64,u32=>u64,l,r,(l as u64),(r as u64),integer);
cpp_arithmetic_pair!(u64,i64=>u64,l,r,(l as u64),(r as u64),integer);
cpp_arithmetic_pair!(u64,u64=>u64,l,r,(l as u64),(r as u64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(u64,isize=>u64,l,r,(l as u64),(r as u64),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(u64,isize=>u64,l,r,(l as u64),(r as u64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(u64,usize=>u64,l,r,(l as u64),(r as u64),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(u64,usize=>u64,l,r,(l as u64),(r as u64),integer);
cpp_arithmetic_pair!(u64,f32=>f32,l,r,(l as f32),(r as f32),float);
cpp_arithmetic_pair!(u64,f64=>f64,l,r,(l as f64),(r as f64),float);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(isize,bool=>i32,l,r,(l as i32),(r as u8 as i32),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(isize,bool=>i64,l,r,(l as i64),(r as u8 as i64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(isize,i8=>i32,l,r,(l as i32),(r as i32),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(isize,i8=>i64,l,r,(l as i64),(r as i64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(isize,u8=>i32,l,r,(l as i32),(r as i32),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(isize,u8=>i64,l,r,(l as i64),(r as i64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(isize,i16=>i32,l,r,(l as i32),(r as i32),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(isize,i16=>i64,l,r,(l as i64),(r as i64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(isize,u16=>i32,l,r,(l as i32),(r as i32),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(isize,u16=>i64,l,r,(l as i64),(r as i64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(isize,i32=>i32,l,r,(l as i32),(r as i32),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(isize,i32=>i64,l,r,(l as i64),(r as i64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(isize,u32=>u32,l,r,(l as u32),(r as u32),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(isize,u32=>i64,l,r,(l as i64),(r as i64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(isize,i64=>i64,l,r,(l as i64),(r as i64),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(isize,i64=>i64,l,r,(l as i64),(r as i64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(isize,u64=>u64,l,r,(l as u64),(r as u64),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(isize,u64=>u64,l,r,(l as u64),(r as u64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(isize,isize=>isize,l,r,(l as isize),(r as isize),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(isize,isize=>isize,l,r,(l as isize),(r as isize),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(isize,usize=>u32,l,r,(l as u32),(r as u32),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(isize,usize=>u64,l,r,(l as u64),(r as u64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(isize,f32=>f32,l,r,(l as f32),(r as f32),float);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(isize,f32=>f32,l,r,(l as f32),(r as f32),float);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(isize,f64=>f64,l,r,(l as f64),(r as f64),float);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(isize,f64=>f64,l,r,(l as f64),(r as f64),float);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(usize,bool=>u32,l,r,(l as u32),(r as u8 as u32),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(usize,bool=>u64,l,r,(l as u64),(r as u8 as u64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(usize,i8=>u32,l,r,(l as u32),(r as u32),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(usize,i8=>u64,l,r,(l as u64),(r as u64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(usize,u8=>u32,l,r,(l as u32),(r as u32),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(usize,u8=>u64,l,r,(l as u64),(r as u64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(usize,i16=>u32,l,r,(l as u32),(r as u32),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(usize,i16=>u64,l,r,(l as u64),(r as u64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(usize,u16=>u32,l,r,(l as u32),(r as u32),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(usize,u16=>u64,l,r,(l as u64),(r as u64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(usize,i32=>u32,l,r,(l as u32),(r as u32),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(usize,i32=>u64,l,r,(l as u64),(r as u64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(usize,u32=>u32,l,r,(l as u32),(r as u32),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(usize,u32=>u64,l,r,(l as u64),(r as u64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(usize,i64=>i64,l,r,(l as i64),(r as i64),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(usize,i64=>u64,l,r,(l as u64),(r as u64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(usize,u64=>u64,l,r,(l as u64),(r as u64),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(usize,u64=>u64,l,r,(l as u64),(r as u64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(usize,isize=>u32,l,r,(l as u32),(r as u32),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(usize,isize=>u64,l,r,(l as u64),(r as u64),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(usize,usize=>usize,l,r,(l as usize),(r as usize),integer);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(usize,usize=>usize,l,r,(l as usize),(r as usize),integer);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(usize,f32=>f32,l,r,(l as f32),(r as f32),float);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(usize,f32=>f32,l,r,(l as f32),(r as f32),float);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(usize,f64=>f64,l,r,(l as f64),(r as f64),float);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(usize,f64=>f64,l,r,(l as f64),(r as f64),float);
cpp_arithmetic_pair!(f32,bool=>f32,l,r,(l as f32),(r as u8 as f32),float);
cpp_arithmetic_pair!(f32,i8=>f32,l,r,(l as f32),(r as f32),float);
cpp_arithmetic_pair!(f32,u8=>f32,l,r,(l as f32),(r as f32),float);
cpp_arithmetic_pair!(f32,i16=>f32,l,r,(l as f32),(r as f32),float);
cpp_arithmetic_pair!(f32,u16=>f32,l,r,(l as f32),(r as f32),float);
cpp_arithmetic_pair!(f32,i32=>f32,l,r,(l as f32),(r as f32),float);
cpp_arithmetic_pair!(f32,u32=>f32,l,r,(l as f32),(r as f32),float);
cpp_arithmetic_pair!(f32,i64=>f32,l,r,(l as f32),(r as f32),float);
cpp_arithmetic_pair!(f32,u64=>f32,l,r,(l as f32),(r as f32),float);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(f32,isize=>f32,l,r,(l as f32),(r as f32),float);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(f32,isize=>f32,l,r,(l as f32),(r as f32),float);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(f32,usize=>f32,l,r,(l as f32),(r as f32),float);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(f32,usize=>f32,l,r,(l as f32),(r as f32),float);
cpp_arithmetic_pair!(f32,f32=>f32,l,r,(l as f32),(r as f32),float);
cpp_arithmetic_pair!(f32,f64=>f64,l,r,(l as f64),(r as f64),float);
cpp_arithmetic_pair!(f64,bool=>f64,l,r,(l as f64),(r as u8 as f64),float);
cpp_arithmetic_pair!(f64,i8=>f64,l,r,(l as f64),(r as f64),float);
cpp_arithmetic_pair!(f64,u8=>f64,l,r,(l as f64),(r as f64),float);
cpp_arithmetic_pair!(f64,i16=>f64,l,r,(l as f64),(r as f64),float);
cpp_arithmetic_pair!(f64,u16=>f64,l,r,(l as f64),(r as f64),float);
cpp_arithmetic_pair!(f64,i32=>f64,l,r,(l as f64),(r as f64),float);
cpp_arithmetic_pair!(f64,u32=>f64,l,r,(l as f64),(r as f64),float);
cpp_arithmetic_pair!(f64,i64=>f64,l,r,(l as f64),(r as f64),float);
cpp_arithmetic_pair!(f64,u64=>f64,l,r,(l as f64),(r as f64),float);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(f64,isize=>f64,l,r,(l as f64),(r as f64),float);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(f64,isize=>f64,l,r,(l as f64),(r as f64),float);
#[cfg(target_pointer_width = "32")]
cpp_arithmetic_pair!(f64,usize=>f64,l,r,(l as f64),(r as f64),float);
#[cfg(target_pointer_width = "64")]
cpp_arithmetic_pair!(f64,usize=>f64,l,r,(l as f64),(r as f64),float);
cpp_arithmetic_pair!(f64,f32=>f64,l,r,(l as f64),(r as f64),float);
cpp_arithmetic_pair!(f64,f64=>f64,l,r,(l as f64),(r as f64),float);
macro_rules! cpp_shift_pair {
    ($l:ty,$r:ty=>$out:ty) => {
        const impl SkCppShl<$r> for $l {
            type Output = $out;
            fn cpp_shl(&self, rhs: &$r) -> $out {
                (*self as $out) << (*rhs as u32)
            }
        }
        const impl SkCppShr<$r> for $l {
            type Output = $out;
            fn cpp_shr(&self, rhs: &$r) -> $out {
                (*self as $out) >> (*rhs as u32)
            }
        }
    };
}
cpp_shift_pair!(bool,bool=>i32);
cpp_shift_pair!(bool,i8=>i32);
cpp_shift_pair!(bool,u8=>i32);
cpp_shift_pair!(bool,i16=>i32);
cpp_shift_pair!(bool,u16=>i32);
cpp_shift_pair!(bool,i32=>i32);
cpp_shift_pair!(bool,u32=>i32);
cpp_shift_pair!(bool,i64=>i32);
cpp_shift_pair!(bool,u64=>i32);
cpp_shift_pair!(bool,isize=>i32);
cpp_shift_pair!(bool,usize=>i32);
cpp_shift_pair!(i8,bool=>i32);
cpp_shift_pair!(i8,i8=>i32);
cpp_shift_pair!(i8,u8=>i32);
cpp_shift_pair!(i8,i16=>i32);
cpp_shift_pair!(i8,u16=>i32);
cpp_shift_pair!(i8,i32=>i32);
cpp_shift_pair!(i8,u32=>i32);
cpp_shift_pair!(i8,i64=>i32);
cpp_shift_pair!(i8,u64=>i32);
cpp_shift_pair!(i8,isize=>i32);
cpp_shift_pair!(i8,usize=>i32);
cpp_shift_pair!(u8,bool=>i32);
cpp_shift_pair!(u8,i8=>i32);
cpp_shift_pair!(u8,u8=>i32);
cpp_shift_pair!(u8,i16=>i32);
cpp_shift_pair!(u8,u16=>i32);
cpp_shift_pair!(u8,i32=>i32);
cpp_shift_pair!(u8,u32=>i32);
cpp_shift_pair!(u8,i64=>i32);
cpp_shift_pair!(u8,u64=>i32);
cpp_shift_pair!(u8,isize=>i32);
cpp_shift_pair!(u8,usize=>i32);
cpp_shift_pair!(i16,bool=>i32);
cpp_shift_pair!(i16,i8=>i32);
cpp_shift_pair!(i16,u8=>i32);
cpp_shift_pair!(i16,i16=>i32);
cpp_shift_pair!(i16,u16=>i32);
cpp_shift_pair!(i16,i32=>i32);
cpp_shift_pair!(i16,u32=>i32);
cpp_shift_pair!(i16,i64=>i32);
cpp_shift_pair!(i16,u64=>i32);
cpp_shift_pair!(i16,isize=>i32);
cpp_shift_pair!(i16,usize=>i32);
cpp_shift_pair!(u16,bool=>i32);
cpp_shift_pair!(u16,i8=>i32);
cpp_shift_pair!(u16,u8=>i32);
cpp_shift_pair!(u16,i16=>i32);
cpp_shift_pair!(u16,u16=>i32);
cpp_shift_pair!(u16,i32=>i32);
cpp_shift_pair!(u16,u32=>i32);
cpp_shift_pair!(u16,i64=>i32);
cpp_shift_pair!(u16,u64=>i32);
cpp_shift_pair!(u16,isize=>i32);
cpp_shift_pair!(u16,usize=>i32);
cpp_shift_pair!(i32,bool=>i32);
cpp_shift_pair!(i32,i8=>i32);
cpp_shift_pair!(i32,u8=>i32);
cpp_shift_pair!(i32,i16=>i32);
cpp_shift_pair!(i32,u16=>i32);
cpp_shift_pair!(i32,i32=>i32);
cpp_shift_pair!(i32,u32=>i32);
cpp_shift_pair!(i32,i64=>i32);
cpp_shift_pair!(i32,u64=>i32);
cpp_shift_pair!(i32,isize=>i32);
cpp_shift_pair!(i32,usize=>i32);
cpp_shift_pair!(u32,bool=>u32);
cpp_shift_pair!(u32,i8=>u32);
cpp_shift_pair!(u32,u8=>u32);
cpp_shift_pair!(u32,i16=>u32);
cpp_shift_pair!(u32,u16=>u32);
cpp_shift_pair!(u32,i32=>u32);
cpp_shift_pair!(u32,u32=>u32);
cpp_shift_pair!(u32,i64=>u32);
cpp_shift_pair!(u32,u64=>u32);
cpp_shift_pair!(u32,isize=>u32);
cpp_shift_pair!(u32,usize=>u32);
cpp_shift_pair!(i64,bool=>i64);
cpp_shift_pair!(i64,i8=>i64);
cpp_shift_pair!(i64,u8=>i64);
cpp_shift_pair!(i64,i16=>i64);
cpp_shift_pair!(i64,u16=>i64);
cpp_shift_pair!(i64,i32=>i64);
cpp_shift_pair!(i64,u32=>i64);
cpp_shift_pair!(i64,i64=>i64);
cpp_shift_pair!(i64,u64=>i64);
cpp_shift_pair!(i64,isize=>i64);
cpp_shift_pair!(i64,usize=>i64);
cpp_shift_pair!(u64,bool=>u64);
cpp_shift_pair!(u64,i8=>u64);
cpp_shift_pair!(u64,u8=>u64);
cpp_shift_pair!(u64,i16=>u64);
cpp_shift_pair!(u64,u16=>u64);
cpp_shift_pair!(u64,i32=>u64);
cpp_shift_pair!(u64,u32=>u64);
cpp_shift_pair!(u64,i64=>u64);
cpp_shift_pair!(u64,u64=>u64);
cpp_shift_pair!(u64,isize=>u64);
cpp_shift_pair!(u64,usize=>u64);
cpp_shift_pair!(isize,bool=>isize);
cpp_shift_pair!(isize,i8=>isize);
cpp_shift_pair!(isize,u8=>isize);
cpp_shift_pair!(isize,i16=>isize);
cpp_shift_pair!(isize,u16=>isize);
cpp_shift_pair!(isize,i32=>isize);
cpp_shift_pair!(isize,u32=>isize);
cpp_shift_pair!(isize,i64=>isize);
cpp_shift_pair!(isize,u64=>isize);
cpp_shift_pair!(isize,isize=>isize);
cpp_shift_pair!(isize,usize=>isize);
cpp_shift_pair!(usize,bool=>usize);
cpp_shift_pair!(usize,i8=>usize);
cpp_shift_pair!(usize,u8=>usize);
cpp_shift_pair!(usize,i16=>usize);
cpp_shift_pair!(usize,u16=>usize);
cpp_shift_pair!(usize,i32=>usize);
cpp_shift_pair!(usize,u32=>usize);
cpp_shift_pair!(usize,i64=>usize);
cpp_shift_pair!(usize,u64=>usize);
cpp_shift_pair!(usize,isize=>usize);
cpp_shift_pair!(usize,usize=>usize);
const impl SkCppNeg for bool {
    type Output = i32;
    fn cpp_neg(&self) -> i32 {
        (*self as u8 as i32).wrapping_neg()
    }
}
const impl SkCppNeg for i8 {
    type Output = i32;
    fn cpp_neg(&self) -> i32 {
        (*self as i32).wrapping_neg()
    }
}
const impl SkCppNeg for u8 {
    type Output = i32;
    fn cpp_neg(&self) -> i32 {
        (*self as i32).wrapping_neg()
    }
}
const impl SkCppNeg for i16 {
    type Output = i32;
    fn cpp_neg(&self) -> i32 {
        (*self as i32).wrapping_neg()
    }
}
const impl SkCppNeg for u16 {
    type Output = i32;
    fn cpp_neg(&self) -> i32 {
        (*self as i32).wrapping_neg()
    }
}
const impl SkCppNeg for i32 {
    type Output = i32;
    fn cpp_neg(&self) -> i32 {
        (*self as i32).wrapping_neg()
    }
}
const impl SkCppNeg for u32 {
    type Output = u32;
    fn cpp_neg(&self) -> u32 {
        (*self as u32).wrapping_neg()
    }
}
const impl SkCppNeg for i64 {
    type Output = i64;
    fn cpp_neg(&self) -> i64 {
        (*self as i64).wrapping_neg()
    }
}
const impl SkCppNeg for u64 {
    type Output = u64;
    fn cpp_neg(&self) -> u64 {
        (*self as u64).wrapping_neg()
    }
}
const impl SkCppNeg for isize {
    type Output = isize;
    fn cpp_neg(&self) -> isize {
        (*self as isize).wrapping_neg()
    }
}
const impl SkCppNeg for usize {
    type Output = usize;
    fn cpp_neg(&self) -> usize {
        (*self as usize).wrapping_neg()
    }
}
const impl SkCppNeg for f32 {
    type Output = f32;
    fn cpp_neg(&self) -> f32 {
        -(*self as f32)
    }
}
const impl SkCppNeg for f64 {
    type Output = f64;
    fn cpp_neg(&self) -> f64 {
        -(*self as f64)
    }
}

// Source: include/private/SkFixed.h:42,56,94,138-141; include/private/SkFloatingPoint.h:38,118-129.
pub const trait SkCppConvertTo<D> {
    fn cpp_convert_to(&self) -> D;
}
const impl<S: Copy, D: [const] crate::include::private::SkTFitsIn::SkCastFrom<S>> SkCppConvertTo<D>
    for S
{
    fn cpp_convert_to(&self) -> D {
        <D as crate::include::private::SkTFitsIn::SkCastFrom<S>>::sk_cast_from(*self)
    }
}

// Source: src/core/SkMathPriv.h:61-62; include/private/SkTemplates.h:45;
// include/private/SkFloatingPoint.h:52-53. A class supplies its actual assignment
// overload; Rust replacement/drop is not assumed to be C++ operator=.
pub trait SkCppAssignFrom<Rhs> {
    fn cpp_assign_from(&mut self, rhs: Rhs);
}
pub trait SkCppAssignBorrowed<Rhs: ?Sized> {
    fn cpp_assign_borrowed(&mut self, rhs: &Rhs);
}
macro_rules! cpp_builtin_assignment {
 ($($t:ty),*)=>{$(
  impl<S> SkCppAssignFrom<S> for $t where $t:crate::include::private::SkTFitsIn::SkCastFrom<S>{
   fn cpp_assign_from(&mut self,rhs:S){*self=<$t as crate::include::private::SkTFitsIn::SkCastFrom<S>>::sk_cast_from(rhs);}
  }
  impl<S:?Sized+SkCppConvertTo<$t>> SkCppAssignBorrowed<S> for $t{
   fn cpp_assign_borrowed(&mut self,rhs:&S){*self=rhs.cpp_convert_to();}
  }
 )*};
}
cpp_builtin_assignment!(bool, i8, u8, i16, u16, i32, u32, i64, u64, isize, usize, f32, f64);
