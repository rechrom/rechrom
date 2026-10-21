//! Actual ARM32 saturation assembly. Selection is a native __ARMEL__ input;
//! target_arch alone does not fabricate that native compiler predefined macro.
#![allow(non_snake_case, non_upper_case_globals)]
use super::safe_conversions_impl::{Cast, Integer, Numeric};
use core::marker::PhantomData;

// Source: base/numerics/safe_conversions_arm_impl.h:23-28
pub struct SaturateFastAsmOp<Dst, Src>(PhantomData<(Dst, Src)>);
impl<Dst: Integer, Src: Integer + Cast<Dst>> SaturateFastAsmOp<Dst, Src> {
    pub const is_supported: bool = cfg!(base_numeric_ndebug)
        && Src::SIGNED
        && (Src::DIGITS as u32 + Src::SIGNED as u32) <= 32
        && (Dst::DIGITS as u32 + Dst::SIGNED as u32) <= 32
        && !<Src as Cast<Dst>>::CONTAINED;
}
// Source: base/numerics/safe_conversions_arm_impl.h:30-47
#[cfg(all(base_numeric_armel, target_arch = "arm"))]
impl<Dst: Integer, Src: Integer + Cast<i32>> SaturateFastAsmOp<Dst, Src>
where
    i32: Cast<Dst>,
    u32: Cast<Dst>,
{
    #[inline(always)]
    pub fn Do(value: Src) -> Dst {
        let src = <Src as Cast<i32>>::cast(value);
        if Dst::SIGNED {
            let result: i32;
            unsafe {
                core::arch::asm!("ssat {dst}, #{shift}, {src}",dst=out(reg) result,
                src=in(reg) src,shift=const {
                    let bits=Dst::DIGITS as u32+Dst::SIGNED as u32;
                    if bits<32 {bits} else {32}
                },options(nomem,nostack));
            }
            <i32 as Cast<Dst>>::cast(result)
        } else {
            let result: u32;
            unsafe {
                core::arch::asm!("usat {dst}, #{shift}, {src}",dst=out(reg) result,
                src=in(reg) src,shift=const {
                    let bits=Dst::DIGITS as u32+Dst::SIGNED as u32;
                    if bits<31 {bits} else {31}
                },options(nomem,nostack));
            }
            <u32 as Cast<Dst>>::cast(result)
        }
    }
}
#[cfg(all(base_numeric_armel, not(target_arch = "arm")))]
compile_error!("native __ARMEL__ saturation selected without actual Rust ARM32 codegen target");
