//! Rust implementation of the upstream `skvx` vector namespace.
//! Width-specific structs preserve migrated tiny-skia storage and arithmetic.
//! They correspond to `skvx::Vec<N,T>` aliases by responsibility, not ABI.
pub mod skvx {
    #![allow(non_camel_case_types)]
    #[cfg(all(not(feature = "std"), feature = "no-std-float"))]
    use crate::path::NoStdFloat;
    use bytemuck::cast;

    // Copyright 2020 Yevhenii Reizner
    //
    // Use of this source code is governed by a BSD-style license that can be
    // found in the LICENSE file.

    // This module was written from scratch, therefore there is no Google copyright.

    // f32x16, i32x16 and u32x16 are implemented as [Tx8; 2] and not as [T; 16].
    // This way we still can use some SIMD.
    //
    // We doesn't use #[inline] that much in this module.
    // The compiler will inline most of the methods automatically.
    // The only exception is U16x16, were we have to force inlining,
    // otherwise the performance will be horrible.

    #[allow(dead_code)]
    #[inline]
    pub fn generic_bit_blend<T>(mask: T, y: T, n: T) -> T
    where
        T: Copy + core::ops::BitXor<Output = T> + core::ops::BitAnd<Output = T>,
    {
        n ^ ((n ^ y) & mask)
    }

    /// A faster and more forgiving f32 min/max implementation.
    ///
    /// Unlike std one, we do not care about NaN.
    #[allow(dead_code)]
    pub trait FasterMinMax {
        fn faster_min(self, rhs: f32) -> f32;
        fn faster_max(self, rhs: f32) -> f32;
    }

    #[allow(dead_code)]
    impl FasterMinMax for f32 {
        fn faster_min(self, rhs: f32) -> f32 {
            if rhs < self {
                rhs
            } else {
                self
            }
        }

        fn faster_max(self, rhs: f32) -> f32 {
            if self < rhs {
                rhs
            } else {
                self
            }
        }
    }

    // Migrated unchanged in behavior from tiny-skia-0.12.0/src/wide/f32x4_t.rs.

    // Copyright 2020 Yevhenii Reizner
    //
    // Use of this source code is governed by a BSD-style license that can be
    // found in the LICENSE file.

    // Based on https://github.com/Lokathor/wide (Zlib)

    cfg_if::cfg_if! {
        if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
            #[cfg(target_arch = "x86")]
            use core::arch::x86::*;
            #[cfg(target_arch = "x86_64")]
            use core::arch::x86_64::*;

            #[derive(Clone, Copy, Debug)]
            #[repr(C, align(16))]
            pub struct float4(__m128);
        } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
            use core::arch::wasm32::*;

            // repr(transparent) allows for directly passing the v128 on the WASM stack.
            #[derive(Clone, Copy, Debug)]
            #[repr(transparent)]
            pub struct float4(v128);
        } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
            use core::arch::aarch64::*;

            #[derive(Clone, Copy, Debug)]
            #[repr(C, align(16))]
            pub struct float4(float32x4_t);
        } else {


            #[derive(Clone, Copy, Debug)]
            #[repr(C, align(16))]
            pub struct float4([f32; 4]);
        }
    }

    unsafe impl bytemuck::Zeroable for float4 {}
    unsafe impl bytemuck::Pod for float4 {}

    impl Default for float4 {
        fn default() -> Self {
            Self::splat(0.0)
        }
    }

    impl float4 {
        pub fn splat(n: f32) -> Self {
            Self::from([n, n, n, n])
        }

        pub fn floor(self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(f32x4_floor(self.0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    Self(unsafe { vrndmq_f32(self.0) })
                } else {
                    let roundtrip: float4 = cast(self.trunc_int().to_f32x4());
                    roundtrip - roundtrip.cmp_gt(self).blend(float4::splat(1.0), float4::default())
                }
            }
        }

        pub fn abs(self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(f32x4_abs(self.0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    Self(unsafe { vabsq_f32(self.0) })
                } else {
                    let non_sign_bits = float4::splat(f32::from_bits(i32::MAX as u32));
                    self & non_sign_bits
                }
            }
        }

        pub fn max(self, rhs: Self) -> Self {
            // These technically don't have the same semantics for NaN and 0, but it
            // doesn't seem to matter as Skia does it the same way.
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                    Self(unsafe { _mm_max_ps(self.0, rhs.0) })
                } else if #[cfg(all(feature = "simd", target_feature = "relaxed-simd"))] {
                    Self(f32x4_relaxed_max(self.0, rhs.0))
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(f32x4_pmax(self.0, rhs.0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    Self(unsafe { vmaxq_f32(self.0, rhs.0) })
                } else {
                    Self([
                        self.0[0].faster_max(rhs.0[0]),
                        self.0[1].faster_max(rhs.0[1]),
                        self.0[2].faster_max(rhs.0[2]),
                        self.0[3].faster_max(rhs.0[3]),
                    ])
                }
            }
        }

        pub fn min(self, rhs: Self) -> Self {
            // These technically don't have the same semantics for NaN and 0, but it
            // doesn't seem to matter as Skia does it the same way.
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                    Self(unsafe { _mm_min_ps(self.0, rhs.0) })
                }  else if #[cfg(all(feature = "simd", target_feature = "relaxed-simd"))] {
                    Self(f32x4_relaxed_min(self.0, rhs.0))
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(f32x4_pmin(self.0, rhs.0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    Self(unsafe { vminq_f32(self.0, rhs.0) })
                } else {
                    Self([
                        self.0[0].faster_min(rhs.0[0]),
                        self.0[1].faster_min(rhs.0[1]),
                        self.0[2].faster_min(rhs.0[2]),
                        self.0[3].faster_min(rhs.0[3]),
                    ])
                }
            }
        }

        pub fn cmp_eq(self, rhs: Self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                    Self(unsafe { _mm_cmpeq_ps(self.0, rhs.0) })
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(f32x4_eq(self.0, rhs.0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    Self(cast(unsafe { vceqq_f32(self.0, rhs.0) }))
                } else {
                    Self([
                        if self.0[0] == rhs.0[0] { f32::from_bits(u32::MAX) } else { 0.0 },
                        if self.0[1] == rhs.0[1] { f32::from_bits(u32::MAX) } else { 0.0 },
                        if self.0[2] == rhs.0[2] { f32::from_bits(u32::MAX) } else { 0.0 },
                        if self.0[3] == rhs.0[3] { f32::from_bits(u32::MAX) } else { 0.0 },
                    ])
                }
            }
        }

        pub fn cmp_ne(self, rhs: Self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                    Self(unsafe { _mm_cmpneq_ps(self.0, rhs.0) })
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(f32x4_ne(self.0, rhs.0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    Self(cast(unsafe { vmvnq_u32(vceqq_f32(self.0, rhs.0)) }))
                } else {
                    Self([
                        if self.0[0] != rhs.0[0] { f32::from_bits(u32::MAX) } else { 0.0 },
                        if self.0[1] != rhs.0[1] { f32::from_bits(u32::MAX) } else { 0.0 },
                        if self.0[2] != rhs.0[2] { f32::from_bits(u32::MAX) } else { 0.0 },
                        if self.0[3] != rhs.0[3] { f32::from_bits(u32::MAX) } else { 0.0 },
                    ])
                }
            }
        }

        pub fn cmp_ge(self, rhs: Self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                    Self(unsafe { _mm_cmpge_ps(self.0, rhs.0) })
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(f32x4_ge(self.0, rhs.0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    Self(cast(unsafe { vcgeq_f32(self.0, rhs.0) }))
                } else {
                    Self([
                        if self.0[0] >= rhs.0[0] { f32::from_bits(u32::MAX) } else { 0.0 },
                        if self.0[1] >= rhs.0[1] { f32::from_bits(u32::MAX) } else { 0.0 },
                        if self.0[2] >= rhs.0[2] { f32::from_bits(u32::MAX) } else { 0.0 },
                        if self.0[3] >= rhs.0[3] { f32::from_bits(u32::MAX) } else { 0.0 },
                    ])
                }
            }
        }

        pub fn cmp_gt(self, rhs: Self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                    Self(unsafe { _mm_cmpgt_ps(self.0, rhs.0) })
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(f32x4_gt(self.0, rhs.0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    Self(cast(unsafe { vcgtq_f32(self.0, rhs.0) }))
                } else {
                    Self([
                        if self.0[0] > rhs.0[0] { f32::from_bits(u32::MAX) } else { 0.0 },
                        if self.0[1] > rhs.0[1] { f32::from_bits(u32::MAX) } else { 0.0 },
                        if self.0[2] > rhs.0[2] { f32::from_bits(u32::MAX) } else { 0.0 },
                        if self.0[3] > rhs.0[3] { f32::from_bits(u32::MAX) } else { 0.0 },
                    ])
                }
            }
        }

        pub fn cmp_le(self, rhs: Self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                    Self(unsafe { _mm_cmple_ps(self.0, rhs.0) })
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(f32x4_le(self.0, rhs.0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    Self(cast(unsafe { vcleq_f32(self.0, rhs.0) }))
                } else {
                    Self([
                        if self.0[0] <= rhs.0[0] { f32::from_bits(u32::MAX) } else { 0.0 },
                        if self.0[1] <= rhs.0[1] { f32::from_bits(u32::MAX) } else { 0.0 },
                        if self.0[2] <= rhs.0[2] { f32::from_bits(u32::MAX) } else { 0.0 },
                        if self.0[3] <= rhs.0[3] { f32::from_bits(u32::MAX) } else { 0.0 },
                    ])
                }
            }
        }

        pub fn cmp_lt(self, rhs: Self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                    Self(unsafe { _mm_cmplt_ps(self.0, rhs.0) })
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(f32x4_lt(self.0, rhs.0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    Self(cast(unsafe { vcltq_f32(self.0, rhs.0) }))
                } else {
                    Self([
                        if self.0[0] < rhs.0[0] { f32::from_bits(u32::MAX) } else { 0.0 },
                        if self.0[1] < rhs.0[1] { f32::from_bits(u32::MAX) } else { 0.0 },
                        if self.0[2] < rhs.0[2] { f32::from_bits(u32::MAX) } else { 0.0 },
                        if self.0[3] < rhs.0[3] { f32::from_bits(u32::MAX) } else { 0.0 },
                    ])
                }
            }
        }

        #[inline]
        pub fn blend(self, t: Self, f: Self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse4.1"))] {
                    Self(unsafe { _mm_blendv_ps(f.0, t.0, self.0) })
                } else if #[cfg(all(feature = "simd", target_feature = "relaxed-simd"))] {
                    Self(i32x4_relaxed_laneselect(t.0, f.0, self.0))
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(v128_bitselect(t.0, f.0, self.0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    Self(unsafe { cast(vbslq_u32( cast(self.0), cast(t.0), cast(f.0))) })
                } else {
                    crate::raster::wide::generic_bit_blend(self, t, f)
                }
            }
        }

        pub fn round(self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse4.1"))] {
                    Self(
                        unsafe { _mm_round_ps(self.0, _MM_FROUND_NO_EXC | _MM_FROUND_TO_NEAREST_INT) },
                    )
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(f32x4_nearest(self.0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    Self(unsafe { vrndnq_f32(self.0) })
                } else {


                    let to_int = float4::splat(1.0 / f32::EPSILON);
                    let u: uint4 = cast(self);
                    let e: int4 = cast(u.shr::<23>() & uint4::splat(0xff));
                    let mut y: float4;

                    let no_op_magic = int4::splat(0x7f + 23);
                    let no_op_mask: float4 = cast(e.cmp_gt(no_op_magic) | e.cmp_eq(no_op_magic));
                    let no_op_val: float4 = self;

                    let zero_magic = int4::splat(0x7f - 1);
                    let zero_mask: float4 = cast(e.cmp_lt(zero_magic));
                    let zero_val: float4 = self * float4::splat(0.0);

                    let neg_bit: float4 = cast(cast::<uint4, int4>(u).cmp_lt(int4::default()));
                    let x: float4 = neg_bit.blend(-self, self);
                    y = x + to_int - to_int - x;
                    y = y.cmp_gt(float4::splat(0.5)).blend(
                        y + x - float4::splat(-1.0),
                        y.cmp_lt(float4::splat(-0.5)).blend(y + x + float4::splat(1.0), y + x),
                    );
                    y = neg_bit.blend(-y, y);

                    no_op_mask.blend(no_op_val, zero_mask.blend(zero_val, y))
                }
            }
        }

        pub fn round_int(self) -> int4 {
            // These technically don't have the same semantics for NaN and out of
            // range values, but it doesn't seem to matter as Skia does it the same
            // way.
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                    int4(unsafe { _mm_cvtps_epi32(self.0) })
                } else if #[cfg(all(feature = "simd", target_feature = "relaxed-simd"))] {
                    int4(i32x4_relaxed_trunc_f32x4(self.round().0))
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    int4(i32x4_trunc_sat_f32x4(self.round().0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    int4(unsafe { vcvtnq_s32_f32(self.0) } )
                } else {
                    let rounded: [f32; 4] = cast(self.round());
                    cast([
                        rounded[0] as i32,
                        rounded[1] as i32,
                        rounded[2] as i32,
                        rounded[3] as i32,
                    ])
                }
            }
        }

        pub fn trunc_int(self) -> int4 {
            // These technically don't have the same semantics for NaN and out of
            // range values, but it doesn't seem to matter as Skia does it the same
            // way.
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                    int4(unsafe { _mm_cvttps_epi32(self.0) })
                } else if #[cfg(all(feature = "simd", target_feature = "relaxed-simd"))] {
                    int4(i32x4_relaxed_trunc_f32x4(self.0))
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    int4(i32x4_trunc_sat_f32x4(self.0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    int4(unsafe { vcvtq_s32_f32(self.0) })
                } else {
                    cast([
                        self.0[0] as i32,
                        self.0[1] as i32,
                        self.0[2] as i32,
                        self.0[3] as i32,
                    ])
                }
            }
        }

        pub fn recip_fast(self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                    Self(unsafe { _mm_rcp_ps(self.0) })
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(f32x4_div(f32x4_splat(1.0), self.0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    unsafe {
                        let a = vrecpeq_f32(self.0);
                        let a = vmulq_f32(vrecpsq_f32(self.0, a), a);
                        Self(a)
                    }
                } else {
                    Self::from([
                        1.0 / self.0[0],
                        1.0 / self.0[1],
                        1.0 / self.0[2],
                        1.0 / self.0[3],
                    ])
                }
            }
        }

        pub fn recip_sqrt(self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                    Self(unsafe { _mm_rsqrt_ps(self.0) })
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(f32x4_div(f32x4_splat(1.0), f32x4_sqrt(self.0)))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    unsafe {
                        let a = vrsqrteq_f32(self.0);
                        let a = vmulq_f32(vrsqrtsq_f32(self.0, vmulq_f32(a, a)), a);
                        Self(a)
                    }
                } else {
                    Self::from([
                        1.0 / self.0[0].sqrt(),
                        1.0 / self.0[1].sqrt(),
                        1.0 / self.0[2].sqrt(),
                        1.0 / self.0[3].sqrt(),
                    ])
                }
            }
        }

        pub fn sqrt(self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                    Self(unsafe { _mm_sqrt_ps(self.0) })
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(f32x4_sqrt(self.0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    Self(unsafe { vsqrtq_f32(self.0) })
                } else {
                    Self::from([
                        self.0[0].sqrt(),
                        self.0[1].sqrt(),
                        self.0[2].sqrt(),
                        self.0[3].sqrt(),
                    ])
                }
            }
        }
    }

    impl From<[f32; 4]> for float4 {
        fn from(v: [f32; 4]) -> Self {
            cast(v)
        }
    }

    impl From<float4> for [f32; 4] {
        fn from(v: float4) -> Self {
            cast(v)
        }
    }

    impl core::ops::Add for float4 {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                    Self(unsafe { _mm_add_ps(self.0, rhs.0) })
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(f32x4_add(self.0, rhs.0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    Self(unsafe { vaddq_f32(self.0, rhs.0) })
                } else {
                    Self([
                        self.0[0] + rhs.0[0],
                        self.0[1] + rhs.0[1],
                        self.0[2] + rhs.0[2],
                        self.0[3] + rhs.0[3],
                    ])
                }
            }
        }
    }

    impl core::ops::AddAssign for float4 {
        fn add_assign(&mut self, rhs: float4) {
            *self = *self + rhs;
        }
    }

    impl core::ops::Sub for float4 {
        type Output = Self;

        fn sub(self, rhs: Self) -> Self::Output {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                    Self(unsafe { _mm_sub_ps(self.0, rhs.0) })
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(f32x4_sub(self.0, rhs.0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    Self(unsafe { vsubq_f32(self.0, rhs.0) })
                } else {
                    Self([
                        self.0[0] - rhs.0[0],
                        self.0[1] - rhs.0[1],
                        self.0[2] - rhs.0[2],
                        self.0[3] - rhs.0[3],
                    ])
                }
            }
        }
    }

    impl core::ops::Mul for float4 {
        type Output = Self;

        fn mul(self, rhs: Self) -> Self::Output {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                    Self(unsafe { _mm_mul_ps(self.0, rhs.0) })
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(f32x4_mul(self.0, rhs.0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    Self(unsafe { vmulq_f32(self.0, rhs.0) })
                } else {
                    Self([
                        self.0[0] * rhs.0[0],
                        self.0[1] * rhs.0[1],
                        self.0[2] * rhs.0[2],
                        self.0[3] * rhs.0[3],
                    ])
                }
            }
        }
    }

    impl core::ops::MulAssign for float4 {
        fn mul_assign(&mut self, rhs: float4) {
            *self = *self * rhs;
        }
    }

    impl core::ops::Div for float4 {
        type Output = Self;

        fn div(self, rhs: Self) -> Self::Output {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                    Self(unsafe { _mm_div_ps(self.0, rhs.0) })
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(f32x4_div(self.0, rhs.0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    Self(unsafe { vdivq_f32(self.0, rhs.0) })
                } else {
                    Self([
                        self.0[0] / rhs.0[0],
                        self.0[1] / rhs.0[1],
                        self.0[2] / rhs.0[2],
                        self.0[3] / rhs.0[3],
                    ])
                }
            }
        }
    }

    impl core::ops::BitAnd for float4 {
        type Output = Self;

        #[inline(always)]
        fn bitand(self, rhs: Self) -> Self::Output {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                    Self(unsafe { _mm_and_ps(self.0, rhs.0) })
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(v128_and(self.0, rhs.0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    Self(cast(unsafe { vandq_u32(cast(self.0), cast(rhs.0)) }))
                } else {
                    Self([
                        f32::from_bits(self.0[0].to_bits() & rhs.0[0].to_bits()),
                        f32::from_bits(self.0[1].to_bits() & rhs.0[1].to_bits()),
                        f32::from_bits(self.0[2].to_bits() & rhs.0[2].to_bits()),
                        f32::from_bits(self.0[3].to_bits() & rhs.0[3].to_bits()),
                    ])
                }
            }
        }
    }

    impl core::ops::BitOr for float4 {
        type Output = Self;

        #[inline(always)]
        fn bitor(self, rhs: Self) -> Self::Output {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                    Self(unsafe { _mm_or_ps(self.0, rhs.0) })
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(v128_or(self.0, rhs.0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    Self(cast(unsafe { vorrq_u32(cast(self.0), cast(rhs.0)) }))
                } else {
                    Self([
                        f32::from_bits(self.0[0].to_bits() | rhs.0[0].to_bits()),
                        f32::from_bits(self.0[1].to_bits() | rhs.0[1].to_bits()),
                        f32::from_bits(self.0[2].to_bits() | rhs.0[2].to_bits()),
                        f32::from_bits(self.0[3].to_bits() | rhs.0[3].to_bits()),
                    ])
                }
            }
        }
    }

    impl core::ops::BitXor for float4 {
        type Output = Self;

        #[inline(always)]
        fn bitxor(self, rhs: Self) -> Self::Output {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                    Self(unsafe { _mm_xor_ps(self.0, rhs.0) })
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(v128_xor(self.0, rhs.0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    Self(cast(unsafe { veorq_u32(cast(self.0), cast(rhs.0)) }))
                } else {
                    Self([
                        f32::from_bits(self.0[0].to_bits() ^ rhs.0[0].to_bits()),
                        f32::from_bits(self.0[1].to_bits() ^ rhs.0[1].to_bits()),
                        f32::from_bits(self.0[2].to_bits() ^ rhs.0[2].to_bits()),
                        f32::from_bits(self.0[3].to_bits() ^ rhs.0[3].to_bits()),
                    ])
                }
            }
        }
    }

    impl core::ops::Neg for float4 {
        type Output = Self;

        fn neg(self) -> Self {
            Self::default() - self
        }
    }

    impl core::ops::Not for float4 {
        type Output = Self;

        fn not(self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                    unsafe {
                        let all_bits = _mm_set1_ps(f32::from_bits(u32::MAX));
                        Self(_mm_xor_ps(self.0, all_bits))
                    }
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(v128_not(self.0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    Self(cast(unsafe { vmvnq_u32(cast(self.0)) }))
                } else {
                    self ^ Self::splat(cast(u32::MAX))
                }
            }
        }
    }

    impl core::cmp::PartialEq for float4 {
        fn eq(&self, rhs: &Self) -> bool {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                    unsafe { _mm_movemask_ps(_mm_cmpeq_ps(self.0, rhs.0)) == 0b1111 }
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    unsafe { vminvq_u32(vceqq_f32(self.0, rhs.0)) != 0 }
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    u32x4_all_true(f32x4_eq(self.0, rhs.0))
                } else {
                    self.0 == rhs.0
                }
            }
        }
    }

    // Migrated unchanged in behavior from tiny-skia-0.12.0/src/wide/f32x8_t.rs.

    // Copyright 2020 Yevhenii Reizner
    //
    // Use of this source code is governed by a BSD-style license that can be
    // found in the LICENSE file.

    // Based on https://github.com/Lokathor/wide (Zlib)

    cfg_if::cfg_if! {
        if #[cfg(all(feature = "simd", target_feature = "avx"))] {
            #[cfg(target_arch = "x86")]
            use core::arch::x86::*;
            #[cfg(target_arch = "x86_64")]
            use core::arch::x86_64::*;

            #[derive(Clone, Copy, Debug)]
            #[repr(C, align(32))]
            pub struct float8(__m256);
        } else {


            #[derive(Clone, Copy, Debug)]
            #[repr(C, align(32))]
            pub struct float8(pub float4, pub float4);
        }
    }

    unsafe impl bytemuck::Zeroable for float8 {}
    unsafe impl bytemuck::Pod for float8 {}

    impl Default for float8 {
        fn default() -> Self {
            Self::splat(0.0)
        }
    }

    impl float8 {
        pub fn splat(n: f32) -> Self {
            cast([n, n, n, n, n, n, n, n])
        }

        pub fn floor(self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(self.0.floor(), self.1.floor())
                } else {
                    let roundtrip: float8 = cast(self.trunc_int().to_f32x8());
                    roundtrip
                        - roundtrip
                            .cmp_gt(self)
                            .blend(float8::splat(1.0), float8::default())
                }
            }
        }

        pub fn fract(self) -> Self {
            self - self.floor()
        }

        pub fn normalize(self) -> Self {
            self.max(float8::default()).min(float8::splat(1.0))
        }

        pub fn to_i32x8_bitcast(self) -> int8 {
            bytemuck::cast(self)
        }

        pub fn to_u32x8_bitcast(self) -> uint8 {
            bytemuck::cast(self)
        }

        pub fn cmp_eq(self, rhs: Self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                    Self(unsafe { _mm256_cmp_ps(self.0, rhs.0, _CMP_EQ_OQ) })
                } else {
                    Self(self.0.cmp_eq(rhs.0), self.1.cmp_eq(rhs.1))
                }
            }
        }

        pub fn cmp_ne(self, rhs: Self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                    // Use UQ (unordered quiet) to match SSE _mm_cmpneq_ps behavior,
                    // which returns true when either operand is NaN.
                    Self(unsafe { _mm256_cmp_ps(self.0, rhs.0, _CMP_NEQ_UQ) })
                } else {
                    Self(self.0.cmp_ne(rhs.0), self.1.cmp_ne(rhs.1))
                }
            }
        }

        pub fn cmp_ge(self, rhs: Self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                    Self(unsafe { _mm256_cmp_ps(self.0, rhs.0, _CMP_GE_OQ) })
                } else {
                    Self(self.0.cmp_ge(rhs.0), self.1.cmp_ge(rhs.1))
                }
            }
        }

        pub fn cmp_gt(self, rhs: Self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                    Self(unsafe { _mm256_cmp_ps(self.0, rhs.0, _CMP_GT_OQ) })
                } else {
                    Self(self.0.cmp_gt(rhs.0), self.1.cmp_gt(rhs.1))
                }
            }
        }

        pub fn cmp_le(self, rhs: Self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                    Self(unsafe { _mm256_cmp_ps(self.0, rhs.0, _CMP_LE_OQ) })
                } else {
                    Self(self.0.cmp_le(rhs.0), self.1.cmp_le(rhs.1))
                }
            }
        }

        pub fn cmp_lt(self, rhs: Self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                    Self(unsafe { _mm256_cmp_ps(self.0, rhs.0, _CMP_LT_OQ) })
                } else {
                    Self(self.0.cmp_lt(rhs.0), self.1.cmp_lt(rhs.1))
                }
            }
        }

        #[inline]
        pub fn blend(self, t: Self, f: Self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                    Self(unsafe { _mm256_blendv_ps(f.0, t.0, self.0) })
                } else {
                    Self(self.0.blend(t.0, f.0), self.1.blend(t.1, f.1))
                }
            }
        }

        pub fn abs(self) -> Self {
            let non_sign_bits = float8::splat(f32::from_bits(i32::MAX as u32));
            self & non_sign_bits
        }

        pub fn max(self, rhs: Self) -> Self {
            // These technically don't have the same semantics for NaN and 0, but it
            // doesn't seem to matter as Skia does it the same way.
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                    Self(unsafe { _mm256_max_ps(self.0, rhs.0) })
                } else {
                    Self(self.0.max(rhs.0), self.1.max(rhs.1))
                }
            }
        }

        pub fn min(self, rhs: Self) -> Self {
            // These technically don't have the same semantics for NaN and 0, but it
            // doesn't seem to matter as Skia does it the same way.
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                    Self(unsafe { _mm256_min_ps(self.0, rhs.0) })
                } else {
                    Self(self.0.min(rhs.0), self.1.min(rhs.1))
                }
            }
        }

        pub fn is_finite(self) -> Self {
            let shifted_exp_mask = uint8::splat(0xFF000000);
            let u: uint8 = cast(self);
            let shift_u = u.shl::<1>();
            let out = !(shift_u & shifted_exp_mask).cmp_eq(shifted_exp_mask);
            cast(out)
        }

        pub fn round(self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                    Self(unsafe { _mm256_round_ps(self.0, _MM_FROUND_NO_EXC | _MM_FROUND_TO_NEAREST_INT) })
                } else {
                    Self(self.0.round(), self.1.round())
                }
            }
        }

        pub fn round_int(self) -> int8 {
            // These technically don't have the same semantics for NaN and out of
            // range values, but it doesn't seem to matter as Skia does it the same
            // way.
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                    cast(unsafe { _mm256_cvtps_epi32(self.0) })
                } else {
                    int8(self.0.round_int(), self.1.round_int())
                }
            }
        }

        pub fn trunc_int(self) -> int8 {
            // These technically don't have the same semantics for NaN and out of
            // range values, but it doesn't seem to matter as Skia does it the same
            // way.
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                    cast(unsafe { _mm256_cvttps_epi32(self.0) })
                } else {
                    int8(self.0.trunc_int(), self.1.trunc_int())
                }
            }
        }

        pub fn recip_fast(self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                    Self(unsafe { _mm256_rcp_ps(self.0) })
                } else {
                    Self(self.0.recip_fast(), self.1.recip_fast())
                }
            }
        }

        pub fn recip_sqrt(self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                    Self(unsafe { _mm256_rsqrt_ps(self.0) })
                } else {
                    Self(self.0.recip_sqrt(), self.1.recip_sqrt())
                }
            }
        }

        pub fn sqrt(self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                    Self(unsafe { _mm256_sqrt_ps(self.0) })
                } else {
                    Self(self.0.sqrt(), self.1.sqrt())
                }
            }
        }

        pub fn powf(self, exp: f32) -> Self {
            let x = self;
            // We assume sign(x) is positive so we can use vectorized i32->f32 conversions
            let e = x.to_i32x8_bitcast().to_f32x8() * float8::splat(1.0f32 / ((1 << 23) as f32));
            let m = (x.to_u32x8_bitcast() & uint8::splat(0x007fffff) | uint8::splat(0x3f000000))
                .to_f32x8_bitcast();

            let log2_x = e
                - float8::splat(124.225514990)
                - float8::splat(1.498030302) * m
                - float8::splat(1.725879990) / (float8::splat(0.3520887068) + m);

            let x = log2_x * float8::splat(exp);

            let f = x - x.floor();

            let mut a = x + float8::splat(121.274057500);
            a = a - f * float8::splat(1.490129070);
            a += float8::splat(27.728023300) / (float8::splat(4.84252568) - f);
            a *= float8::splat((1 << 23) as f32);

            let inf_bits = float8::splat(f32::INFINITY.to_bits() as f32);

            let x = a
                .max(float8::splat(0.0))
                .min(inf_bits)
                .round_int()
                .to_f32x8_bitcast();

            let skip = self.cmp_eq(float8::splat(0.0)) | self.cmp_eq(float8::splat(1.0));
            skip.blend(self, x)
        }
    }

    impl From<[f32; 8]> for float8 {
        fn from(v: [f32; 8]) -> Self {
            cast(v)
        }
    }

    impl From<float8> for [f32; 8] {
        fn from(v: float8) -> Self {
            cast(v)
        }
    }

    impl core::ops::Add for float8 {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                    Self(unsafe { _mm256_add_ps(self.0, rhs.0) })
                } else {
                    Self(self.0 + rhs.0, self.1 + rhs.1)
                }
            }
        }
    }

    impl core::ops::AddAssign for float8 {
        fn add_assign(&mut self, rhs: float8) {
            *self = *self + rhs;
        }
    }

    impl core::ops::Sub for float8 {
        type Output = Self;

        fn sub(self, rhs: Self) -> Self::Output {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                    Self(unsafe { _mm256_sub_ps(self.0, rhs.0) })
                } else {
                    Self(self.0 - rhs.0, self.1 - rhs.1)
                }
            }
        }
    }

    impl core::ops::Mul for float8 {
        type Output = Self;

        fn mul(self, rhs: Self) -> Self::Output {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                    Self(unsafe { _mm256_mul_ps(self.0, rhs.0) })
                } else {
                    Self(self.0 * rhs.0, self.1 * rhs.1)
                }
            }
        }
    }

    impl core::ops::MulAssign for float8 {
        fn mul_assign(&mut self, rhs: float8) {
            *self = *self * rhs;
        }
    }

    impl core::ops::Div for float8 {
        type Output = Self;

        fn div(self, rhs: Self) -> Self::Output {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                    Self(unsafe { _mm256_div_ps(self.0, rhs.0) })
                } else {
                    Self(self.0 / rhs.0, self.1 / rhs.1)
                }
            }
        }
    }

    impl core::ops::BitAnd for float8 {
        type Output = Self;

        #[inline(always)]
        fn bitand(self, rhs: Self) -> Self::Output {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                    Self(unsafe { _mm256_and_ps(self.0, rhs.0) })
                } else {
                    Self(self.0 & rhs.0, self.1 & rhs.1)
                }
            }
        }
    }

    impl core::ops::BitOr for float8 {
        type Output = Self;

        #[inline(always)]
        fn bitor(self, rhs: Self) -> Self::Output {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                    Self(unsafe { _mm256_or_ps(self.0, rhs.0) })
                } else {
                    Self(self.0 | rhs.0, self.1 | rhs.1)
                }
            }
        }
    }

    impl core::ops::BitXor for float8 {
        type Output = Self;

        #[inline(always)]
        fn bitxor(self, rhs: Self) -> Self::Output {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                    Self(unsafe { _mm256_xor_ps(self.0, rhs.0) })
                } else {
                    Self(self.0 ^ rhs.0, self.1 ^ rhs.1)
                }
            }
        }
    }

    impl core::ops::Neg for float8 {
        type Output = Self;

        fn neg(self) -> Self {
            Self::default() - self
        }
    }

    impl core::ops::Not for float8 {
        type Output = Self;

        fn not(self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                    let all_bits = unsafe { _mm256_set1_ps(f32::from_bits(u32::MAX)) };
                    Self(unsafe { _mm256_xor_ps(self.0, all_bits) })
                } else {
                    Self(!self.0, !self.1)
                }
            }
        }
    }

    impl core::cmp::PartialEq for float8 {
        fn eq(&self, rhs: &Self) -> bool {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                    let mask = unsafe { _mm256_cmp_ps(self.0, rhs.0, _CMP_EQ_OQ) };
                    unsafe { _mm256_movemask_ps(mask) == 0b1111_1111 }
                } else {
                    self.0 == rhs.0 && self.1 == rhs.1
                }
            }
        }
    }

    // Migrated unchanged in behavior from tiny-skia-0.12.0/src/wide/u32x4_t.rs.

    // Copyright 2020 Yevhenii Reizner
    //
    // Use of this source code is governed by a BSD-style license that can be
    // found in the LICENSE file.

    // Based on https://github.com/Lokathor/wide (Zlib)

    cfg_if::cfg_if! {
        if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
            #[cfg(target_arch = "x86")]
            use core::arch::x86::*;
            #[cfg(target_arch = "x86_64")]
            use core::arch::x86_64::*;

            // unused when AVX is available
            #[cfg(not(all(feature = "simd", target_feature = "avx2")))]


            #[derive(Clone, Copy, Debug)]
            #[repr(C, align(16))]
            pub struct uint4(__m128i);
        } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
            use core::arch::wasm32::*;

            #[derive(Clone, Copy, Debug)]
            #[repr(C, align(16))]
            pub struct uint4(v128);
        } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
            use core::arch::aarch64::*;

            #[derive(Clone, Copy, Debug)]
            #[repr(C, align(16))]
            pub struct uint4(uint32x4_t);
        } else {
            #[derive(Clone, Copy, Debug)]
            #[repr(C, align(16))]
            pub struct uint4([u32; 4]);
        }
    }

    unsafe impl bytemuck::Zeroable for uint4 {}
    unsafe impl bytemuck::Pod for uint4 {}

    impl Default for uint4 {
        fn default() -> Self {
            Self::splat(0)
        }
    }

    impl uint4 {
        pub fn splat(n: u32) -> Self {
            bytemuck::cast([n, n, n, n])
        }

        // unused when AVX is available
        #[cfg(not(all(feature = "simd", target_feature = "avx2")))]
        pub fn cmp_eq(self, rhs: Self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                    Self(unsafe { _mm_cmpeq_epi32(self.0, rhs.0) })
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(u32x4_eq(self.0, rhs.0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    Self(unsafe { vceqq_u32(self.0, rhs.0) })
                } else {
                    Self([
                        if self.0[0] == rhs.0[0] { u32::MAX } else { 0 },
                        if self.0[1] == rhs.0[1] { u32::MAX } else { 0 },
                        if self.0[2] == rhs.0[2] { u32::MAX } else { 0 },
                        if self.0[3] == rhs.0[3] { u32::MAX } else { 0 },
                    ])
                }
            }
        }

        // unused when AVX is available
        #[cfg(not(all(feature = "simd", target_feature = "avx2")))]
        pub fn shl<const RHS: i32>(self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                    let shift = cast([RHS as u64, 0]);
                    Self(unsafe { _mm_sll_epi32(self.0, shift) })
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(u32x4_shl(self.0, RHS as _))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    Self(unsafe { vshlq_n_u32::<RHS>(self.0) })
                } else {
                    let u = RHS as u64;
                    Self([
                        self.0[0] << u,
                        self.0[1] << u,
                        self.0[2] << u,
                        self.0[3] << u,
                    ])
                }
            }
        }

        // unused when AVX is available
        #[cfg(not(all(feature = "simd", target_feature = "avx2")))]
        pub fn shr<const RHS: i32>(self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                    let shift: __m128i = cast([RHS as u64, 0]);
                    Self(unsafe { _mm_srl_epi32(self.0, shift) })
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(u32x4_shr(self.0, RHS as _))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    Self(unsafe { vshrq_n_u32::<RHS>(self.0) })
                } else {
                    let u = RHS as u64;
                    Self([
                        self.0[0] >> u,
                        self.0[1] >> u,
                        self.0[2] >> u,
                        self.0[3] >> u,
                    ])
                }
            }
        }
    }

    impl core::ops::Not for uint4 {
        type Output = Self;

        fn not(self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                    let all_bits = unsafe { _mm_set1_epi32(-1) };
                    Self(unsafe { _mm_xor_si128(self.0, all_bits) })
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(v128_not(self.0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    Self(unsafe { vmvnq_u32(self.0) })
                } else {
                    Self([
                        !self.0[0],
                        !self.0[1],
                        !self.0[2],
                        !self.0[3],
                    ])
                }
            }
        }
    }

    impl core::ops::Add for uint4 {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                    Self(unsafe { _mm_add_epi32(self.0, rhs.0) })
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(u32x4_add(self.0, rhs.0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    Self(unsafe { vaddq_u32(self.0, rhs.0) })
                } else {
                    Self([
                        self.0[0].wrapping_add(rhs.0[0]),
                        self.0[1].wrapping_add(rhs.0[1]),
                        self.0[2].wrapping_add(rhs.0[2]),
                        self.0[3].wrapping_add(rhs.0[3]),
                    ])
                }
            }
        }
    }

    impl core::ops::BitAnd for uint4 {
        type Output = Self;

        fn bitand(self, rhs: Self) -> Self::Output {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                    Self(unsafe { _mm_and_si128(self.0, rhs.0) })
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(v128_and(self.0, rhs.0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    Self(unsafe { vandq_u32(self.0, rhs.0) })
                } else {
                    Self([
                        self.0[0] & rhs.0[0],
                        self.0[1] & rhs.0[1],
                        self.0[2] & rhs.0[2],
                        self.0[3] & rhs.0[3],
                    ])
                }
            }
        }
    }

    impl core::ops::BitOr for uint4 {
        type Output = Self;

        fn bitor(self, rhs: Self) -> Self::Output {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                    Self(unsafe { _mm_or_si128(self.0, rhs.0) })
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(v128_or(self.0, rhs.0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    Self(unsafe { vorrq_u32(self.0, rhs.0) })
                } else {
                    Self([
                        self.0[0] | rhs.0[0],
                        self.0[1] | rhs.0[1],
                        self.0[2] | rhs.0[2],
                        self.0[3] | rhs.0[3],
                    ])
                }
            }
        }
    }

    // Migrated unchanged in behavior from tiny-skia-0.12.0/src/wide/u32x8_t.rs.

    // Copyright 2020 Yevhenii Reizner
    //
    // Use of this source code is governed by a BSD-style license that can be
    // found in the LICENSE file.

    // Based on https://github.com/Lokathor/wide (Zlib)

    cfg_if::cfg_if! {
        if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
            #[cfg(target_arch = "x86")]
            use core::arch::x86::*;
            #[cfg(target_arch = "x86_64")]
            use core::arch::x86_64::*;



            #[derive(Clone, Copy, Debug)]
            #[repr(C, align(32))]
            pub struct uint8(__m256i);
        } else {


            #[derive(Clone, Copy, Debug)]
            #[repr(C, align(32))]
            pub struct uint8(uint4, uint4);
        }
    }

    unsafe impl bytemuck::Zeroable for uint8 {}
    unsafe impl bytemuck::Pod for uint8 {}

    impl Default for uint8 {
        fn default() -> Self {
            Self::splat(0)
        }
    }

    impl uint8 {
        pub fn splat(n: u32) -> Self {
            bytemuck::cast([n, n, n, n, n, n, n, n])
        }

        pub fn to_i32x8_bitcast(self) -> int8 {
            bytemuck::cast(self)
        }

        pub fn to_f32x8_bitcast(self) -> float8 {
            bytemuck::cast(self)
        }

        pub fn cmp_eq(self, rhs: Self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
                    Self(unsafe { _mm256_cmpeq_epi32(self.0, rhs.0) })
                } else {
                    Self(self.0.cmp_eq(rhs.0), self.1.cmp_eq(rhs.1))
                }
            }
        }

        pub fn shl<const RHS: i32>(self) -> Self {
            cfg_if::cfg_if! {
               if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
                    let shift: __m128i = cast([RHS as u64, 0]);
                    Self(unsafe { _mm256_sll_epi32(self.0, shift) })
                } else {
                    Self(self.0.shl::<RHS>(), self.1.shl::<RHS>())
                }
            }
        }

        pub fn shr<const RHS: i32>(self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
                    let shift: __m128i = cast([RHS as u64, 0]);
                    Self(unsafe { _mm256_srl_epi32(self.0, shift) })
                } else {
                    Self(self.0.shr::<RHS>(), self.1.shr::<RHS>())
                }
            }
        }
    }

    impl core::ops::Not for uint8 {
        type Output = Self;

        fn not(self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
                    let all_bits = unsafe { _mm256_set1_epi16(-1) };
                    Self(unsafe { _mm256_xor_si256(self.0, all_bits) })
                } else {
                    Self(!self.0, !self.1)
                }
            }
        }
    }

    impl core::ops::Add for uint8 {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
                    Self(unsafe { _mm256_add_epi32(self.0, rhs.0) })
                } else {
                    Self(self.0 + rhs.0, self.1 + rhs.1)
                }
            }
        }
    }

    impl core::ops::BitAnd for uint8 {
        type Output = Self;

        fn bitand(self, rhs: Self) -> Self::Output {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
                    Self(unsafe { _mm256_and_si256(self.0, rhs.0) })
                } else {
                    Self(self.0 & rhs.0, self.1 & rhs.1)
                }
            }
        }
    }

    impl core::ops::BitOr for uint8 {
        type Output = Self;

        fn bitor(self, rhs: Self) -> Self::Output {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
                    Self(unsafe { _mm256_or_si256(self.0, rhs.0) })
                } else {
                    Self(self.0 | rhs.0, self.1 | rhs.1)
                }
            }
        }
    }

    // Migrated unchanged in behavior from tiny-skia-0.12.0/src/wide/u16x16_t.rs.

    // Copyright 2020 Yevhenii Reizner
    //
    // Use of this source code is governed by a BSD-style license that can be
    // found in the LICENSE file.

    // No need to use explicit 256bit AVX2 SIMD.
    // `-C target-cpu=native` will autovectorize it better than us.
    // Not even sure why explicit instructions are so slow...
    //
    // On ARM AArch64 we can actually get up to 2x performance boost by using SIMD.
    //
    // We also have to inline all the methods. They are pretty large,
    // but without the inlining the performance is plummeting.

    #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))]
    #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))]
    use core::arch::aarch64::uint16x8_t;

    #[allow(non_camel_case_types)]
    #[derive(Copy, Clone, PartialEq, Default, Debug)]
    pub struct UShort16(pub [u16; 16]);

    macro_rules! impl_u16x16_op {
        ($a:expr, $op:ident, $b:expr) => {
            UShort16([
                $a.0[0].$op($b.0[0]),
                $a.0[1].$op($b.0[1]),
                $a.0[2].$op($b.0[2]),
                $a.0[3].$op($b.0[3]),
                $a.0[4].$op($b.0[4]),
                $a.0[5].$op($b.0[5]),
                $a.0[6].$op($b.0[6]),
                $a.0[7].$op($b.0[7]),
                $a.0[8].$op($b.0[8]),
                $a.0[9].$op($b.0[9]),
                $a.0[10].$op($b.0[10]),
                $a.0[11].$op($b.0[11]),
                $a.0[12].$op($b.0[12]),
                $a.0[13].$op($b.0[13]),
                $a.0[14].$op($b.0[14]),
                $a.0[15].$op($b.0[15]),
            ])
        };
    }

    #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))]
    macro_rules! impl_aarch64_call {
        ($f:ident, $a:expr, $b:expr) => {
            let a = $a.split();
            let b = $b.split();
            Self(bytemuck::cast([
                unsafe { core::arch::aarch64::$f(a.0, b.0) },
                unsafe { core::arch::aarch64::$f(a.1, b.1) },
            ]))
        };
    }

    impl UShort16 {
        #[inline]
        pub fn splat(n: u16) -> Self {
            Self([n, n, n, n, n, n, n, n, n, n, n, n, n, n, n, n])
        }

        #[inline]
        pub fn as_slice(&self) -> &[u16; 16] {
            &self.0
        }

        #[inline]
        pub fn min(&self, rhs: &Self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    impl_aarch64_call!(vminq_u16, self, rhs)
                } else {
                    impl_u16x16_op!(self, min, rhs)
                }
            }
        }

        #[inline]
        pub fn max(&self, rhs: &Self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    impl_aarch64_call!(vmaxq_u16, self, rhs)
                } else {
                    impl_u16x16_op!(self, max, rhs)
                }
            }
        }

        #[inline]
        pub fn cmp_le(&self, rhs: &Self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    impl_aarch64_call!(vcleq_u16, self, rhs)
                } else {
                    Self([
                        if self.0[ 0] <= rhs.0[ 0] { !0 } else { 0 },
                        if self.0[ 1] <= rhs.0[ 1] { !0 } else { 0 },
                        if self.0[ 2] <= rhs.0[ 2] { !0 } else { 0 },
                        if self.0[ 3] <= rhs.0[ 3] { !0 } else { 0 },
                        if self.0[ 4] <= rhs.0[ 4] { !0 } else { 0 },
                        if self.0[ 5] <= rhs.0[ 5] { !0 } else { 0 },
                        if self.0[ 6] <= rhs.0[ 6] { !0 } else { 0 },
                        if self.0[ 7] <= rhs.0[ 7] { !0 } else { 0 },
                        if self.0[ 8] <= rhs.0[ 8] { !0 } else { 0 },
                        if self.0[ 9] <= rhs.0[ 9] { !0 } else { 0 },
                        if self.0[10] <= rhs.0[10] { !0 } else { 0 },
                        if self.0[11] <= rhs.0[11] { !0 } else { 0 },
                        if self.0[12] <= rhs.0[12] { !0 } else { 0 },
                        if self.0[13] <= rhs.0[13] { !0 } else { 0 },
                        if self.0[14] <= rhs.0[14] { !0 } else { 0 },
                        if self.0[15] <= rhs.0[15] { !0 } else { 0 },
                    ])
                }
            }
        }

        #[inline]
        pub fn blend(self, t: Self, e: Self) -> Self {
            (t & self) | (e & !self)
        }

        #[inline]
        #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))]
        pub fn split(self) -> (uint16x8_t, uint16x8_t) {
            let pair: [uint16x8_t; 2] = cast(self.0);
            (pair[0], pair[1])
        }
    }

    impl core::ops::Add<UShort16> for UShort16 {
        type Output = Self;

        #[inline]
        fn add(self, rhs: Self) -> Self::Output {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    impl_aarch64_call!(vaddq_u16, self, rhs)
                } else {
                    impl_u16x16_op!(self, add, rhs)
                }
            }
        }
    }

    impl core::ops::Sub<UShort16> for UShort16 {
        type Output = Self;

        #[inline]
        fn sub(self, rhs: Self) -> Self::Output {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    impl_aarch64_call!(vsubq_u16, self, rhs)
                } else {
                    impl_u16x16_op!(self, sub, rhs)
                }
            }
        }
    }

    impl core::ops::Mul<UShort16> for UShort16 {
        type Output = Self;

        #[inline]
        fn mul(self, rhs: Self) -> Self::Output {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    impl_aarch64_call!(vmulq_u16, self, rhs)
                } else {
                    impl_u16x16_op!(self, mul, rhs)
                }
            }
        }
    }

    impl core::ops::Div<UShort16> for UShort16 {
        type Output = Self;

        #[inline]
        fn div(self, rhs: Self) -> Self::Output {
            impl_u16x16_op!(self, div, rhs)
        }
    }

    impl core::ops::BitAnd<UShort16> for UShort16 {
        type Output = Self;

        #[inline]
        fn bitand(self, rhs: Self) -> Self::Output {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    impl_aarch64_call!(vandq_u16, self, rhs)
                } else {
                    impl_u16x16_op!(self, bitand, rhs)
                }
            }
        }
    }

    impl core::ops::BitOr<UShort16> for UShort16 {
        type Output = Self;

        #[inline]
        fn bitor(self, rhs: Self) -> Self::Output {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    impl_aarch64_call!(vorrq_u16, self, rhs)
                } else {
                    impl_u16x16_op!(self, bitor, rhs)
                }
            }
        }
    }

    impl core::ops::Not for UShort16 {
        type Output = Self;

        #[inline]
        fn not(self) -> Self::Output {
            UShort16([
                !self.0[0],
                !self.0[1],
                !self.0[2],
                !self.0[3],
                !self.0[4],
                !self.0[5],
                !self.0[6],
                !self.0[7],
                !self.0[8],
                !self.0[9],
                !self.0[10],
                !self.0[11],
                !self.0[12],
                !self.0[13],
                !self.0[14],
                !self.0[15],
            ])
        }
    }

    impl core::ops::Shr for UShort16 {
        type Output = Self;

        #[inline]
        fn shr(self, rhs: Self) -> Self::Output {
            impl_u16x16_op!(self, shr, rhs)
        }
    }

    // Migrated unchanged in behavior from tiny-skia-0.12.0/src/wide/i32x4_t.rs.

    // Copyright 2020 Yevhenii Reizner
    //
    // Use of this source code is governed by a BSD-style license that can be
    // found in the LICENSE file.

    // Based on https://github.com/Lokathor/wide (Zlib)

    cfg_if::cfg_if! {
        if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
            #[cfg(target_arch = "x86")]
            use core::arch::x86::*;
            #[cfg(target_arch = "x86_64")]
            use core::arch::x86_64::*;

            #[derive(Clone, Copy, Debug)]
            #[repr(C, align(16))]
            pub struct int4(pub __m128i);
        } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
            use core::arch::wasm32::*;

            #[derive(Clone, Copy, Debug)]
            #[repr(C, align(16))]
            pub struct int4(pub v128);
        } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
            use core::arch::aarch64::*;

            #[derive(Clone, Copy, Debug)]
            #[repr(C, align(16))]
            pub struct int4(pub int32x4_t);
        } else {
            #[derive(Clone, Copy, Debug)]
            #[repr(C, align(16))]
            pub struct int4([i32; 4]);
        }
    }

    unsafe impl bytemuck::Zeroable for int4 {}
    unsafe impl bytemuck::Pod for int4 {}

    impl Default for int4 {
        fn default() -> Self {
            Self::splat(0)
        }
    }

    impl int4 {
        pub fn splat(n: i32) -> Self {
            cast([n, n, n, n])
        }

        pub fn blend(self, t: Self, f: Self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse4.1"))] {
                    Self(unsafe { _mm_blendv_epi8(f.0, t.0, self.0) })
                } else if #[cfg(all(feature = "simd", target_feature = "relaxed-simd"))] {
                    Self(i32x4_relaxed_laneselect(t.0, f.0, self.0))
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(v128_bitselect(t.0, f.0, self.0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    Self(unsafe { vbslq_s32(cast(self.0), t.0, f.0) })
                } else {
                    crate::raster::wide::generic_bit_blend(self, t, f)
                }
            }
        }

        pub fn cmp_eq(self, rhs: Self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                    cast(Self(cast(unsafe { _mm_cmpeq_epi32(self.0, rhs.0) })))
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(i32x4_eq(self.0, rhs.0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    Self(unsafe { cast(vceqq_s32(self.0, rhs.0)) })
                } else {
                    Self([
                        if self.0[0] == rhs.0[0] { -1 } else { 0 },
                        if self.0[1] == rhs.0[1] { -1 } else { 0 },
                        if self.0[2] == rhs.0[2] { -1 } else { 0 },
                        if self.0[3] == rhs.0[3] { -1 } else { 0 },
                    ])
                }
            }
        }

        pub fn cmp_gt(self, rhs: Self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                    cast(Self(cast(unsafe { _mm_cmpgt_epi32(self.0, rhs.0) })))
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(i32x4_gt(self.0, rhs.0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    Self(unsafe { cast(vcgtq_s32(self.0, rhs.0)) })
                } else {
                    Self([
                        if self.0[0] > rhs.0[0] { -1 } else { 0 },
                        if self.0[1] > rhs.0[1] { -1 } else { 0 },
                        if self.0[2] > rhs.0[2] { -1 } else { 0 },
                        if self.0[3] > rhs.0[3] { -1 } else { 0 },
                    ])
                }
            }
        }

        pub fn cmp_lt(self, rhs: Self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                    cast(Self(cast(unsafe { _mm_cmplt_epi32(self.0, rhs.0) })))
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(i32x4_lt(self.0, rhs.0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    Self(unsafe { cast(vcltq_s32(self.0, rhs.0)) })
                } else {
                    Self([
                        if self.0[0] < rhs.0[0] { -1 } else { 0 },
                        if self.0[1] < rhs.0[1] { -1 } else { 0 },
                        if self.0[2] < rhs.0[2] { -1 } else { 0 },
                        if self.0[3] < rhs.0[3] { -1 } else { 0 },
                    ])
                }
            }
        }

        pub fn to_f32x4(self) -> float4 {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                    cast(Self(cast(unsafe { _mm_cvtepi32_ps(self.0) })))
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    cast(Self(f32x4_convert_i32x4(self.0)))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    cast(Self(unsafe { cast(vcvtq_f32_s32(self.0)) }))
                } else {
                    let arr: [i32; 4] = cast(self);
                    cast([
                        arr[0] as f32,
                        arr[1] as f32,
                        arr[2] as f32,
                        arr[3] as f32,
                    ])
                }
            }
        }

        pub fn to_f32x4_bitcast(self) -> float4 {
            bytemuck::cast(self)
        }
    }

    impl From<[i32; 4]> for int4 {
        fn from(v: [i32; 4]) -> Self {
            cast(v)
        }
    }

    impl From<int4> for [i32; 4] {
        fn from(v: int4) -> Self {
            cast(v)
        }
    }

    impl core::ops::Add for int4 {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                    Self(unsafe { _mm_add_epi32(self.0, rhs.0) })
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(i32x4_add(self.0, rhs.0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    Self(unsafe { vaddq_s32(self.0, rhs.0) })
                } else {
                    Self([
                        self.0[0].wrapping_add(rhs.0[0]),
                        self.0[1].wrapping_add(rhs.0[1]),
                        self.0[2].wrapping_add(rhs.0[2]),
                        self.0[3].wrapping_add(rhs.0[3]),
                    ])
                }
            }
        }
    }

    impl core::ops::BitAnd for int4 {
        type Output = Self;

        fn bitand(self, rhs: Self) -> Self::Output {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                    Self(unsafe { _mm_and_si128(self.0, rhs.0) })
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(v128_and(self.0, rhs.0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    Self(unsafe { vandq_s32(self.0, rhs.0) })
                } else {
                    Self([
                        self.0[0] & rhs.0[0],
                        self.0[1] & rhs.0[1],
                        self.0[2] & rhs.0[2],
                        self.0[3] & rhs.0[3],
                    ])
                }
            }
        }
    }

    impl core::ops::Mul for int4 {
        type Output = Self;

        fn mul(self, rhs: Self) -> Self::Output {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse4.1"))] {
                    Self(unsafe { _mm_mullo_epi32(self.0, rhs.0) })
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(i32x4_mul(self.0, rhs.0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    Self(unsafe { vmulq_s32(self.0, rhs.0) })
                } else {
                    // Cast is required, since we have to use scalar multiplication on SSE2.
                    let a: [i32; 4] = cast(self);
                    let b: [i32; 4] = cast(rhs);
                    Self(cast([
                        a[0].wrapping_mul(b[0]),
                        a[1].wrapping_mul(b[1]),
                        a[2].wrapping_mul(b[2]),
                        a[3].wrapping_mul(b[3]),
                    ]))
                }
            }
        }
    }

    impl core::ops::BitOr for int4 {
        type Output = Self;

        #[inline]
        fn bitor(self, rhs: Self) -> Self::Output {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                    Self(unsafe { _mm_or_si128(self.0, rhs.0) })
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(v128_or(self.0, rhs.0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    Self(unsafe { vorrq_s32(self.0, rhs.0) })
                } else {
                    Self([
                        self.0[0] | rhs.0[0],
                        self.0[1] | rhs.0[1],
                        self.0[2] | rhs.0[2],
                        self.0[3] | rhs.0[3],
                    ])
                }
            }
        }
    }

    impl core::ops::BitXor for int4 {
        type Output = Self;

        #[inline]
        fn bitxor(self, rhs: Self) -> Self::Output {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                    Self(unsafe { _mm_xor_si128(self.0, rhs.0) })
                } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                    Self(v128_xor(self.0, rhs.0))
                } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                    Self(unsafe { veorq_s32(self.0, rhs.0) })
                } else {
                    Self([
                        self.0[0] ^ rhs.0[0],
                        self.0[1] ^ rhs.0[1],
                        self.0[2] ^ rhs.0[2],
                        self.0[3] ^ rhs.0[3],
                    ])
                }
            }
        }
    }

    // Migrated unchanged in behavior from tiny-skia-0.12.0/src/wide/i32x8_t.rs.

    // Copyright 2020 Yevhenii Reizner
    //
    // Use of this source code is governed by a BSD-style license that can be
    // found in the LICENSE file.

    // Based on https://github.com/Lokathor/wide (Zlib)

    cfg_if::cfg_if! {
        if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
            #[cfg(target_arch = "x86")]
            use core::arch::x86::*;
            #[cfg(target_arch = "x86_64")]
            use core::arch::x86_64::*;

            #[derive(Clone, Copy, Debug)]
            #[repr(C, align(32))]
            pub struct int8(__m256i);
        } else {


            #[derive(Clone, Copy, Debug)]
            #[repr(C, align(32))]
            pub struct int8(pub int4, pub int4);
        }
    }

    unsafe impl bytemuck::Zeroable for int8 {}
    unsafe impl bytemuck::Pod for int8 {}

    impl Default for int8 {
        fn default() -> Self {
            Self::splat(0)
        }
    }

    impl int8 {
        pub fn splat(n: i32) -> Self {
            cast([n, n, n, n, n, n, n, n])
        }

        pub fn blend(self, t: Self, f: Self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
                    Self(unsafe { _mm256_blendv_epi8(f.0, t.0, self.0) })
                } else {
                    Self(self.0.blend(t.0, f.0), self.1.blend(t.1, f.1))
                }
            }
        }

        pub fn cmp_eq(self, rhs: Self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
                    Self(unsafe { _mm256_cmpeq_epi32(self.0, rhs.0) })
                } else {
                    Self(self.0.cmp_eq(rhs.0), self.1.cmp_eq(rhs.1))
                }
            }
        }

        pub fn cmp_gt(self, rhs: Self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
                    Self(unsafe { _mm256_cmpgt_epi32(self.0, rhs.0) })
                } else {
                    Self(self.0.cmp_gt(rhs.0), self.1.cmp_gt(rhs.1))
                }
            }
        }

        pub fn cmp_lt(self, rhs: Self) -> Self {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
                    // There is no `_mm256_cmpLT_epi32`, therefore we have to use
                    // `_mm256_cmpGT_epi32` and then invert the result.
                    let v = unsafe { _mm256_cmpgt_epi32(self.0, rhs.0) };
                    let all_bits = unsafe { _mm256_set1_epi16(-1) };
                    Self(unsafe { _mm256_xor_si256(v, all_bits) })
                } else {
                    Self(self.0.cmp_lt(rhs.0), self.1.cmp_lt(rhs.1))
                }
            }
        }

        pub fn to_f32x8(self) -> float8 {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
                    cast(unsafe { _mm256_cvtepi32_ps(self.0) })
                } else if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                    cast([self.0.to_f32x4(), self.1.to_f32x4()])
                } else {
                    float8(self.0.to_f32x4(), self.1.to_f32x4())
                }
            }
        }

        pub fn to_u32x8_bitcast(self) -> uint8 {
            bytemuck::cast(self)
        }

        pub fn to_f32x8_bitcast(self) -> float8 {
            bytemuck::cast(self)
        }
    }

    impl From<[i32; 8]> for int8 {
        fn from(v: [i32; 8]) -> Self {
            cast(v)
        }
    }

    impl From<int8> for [i32; 8] {
        fn from(v: int8) -> Self {
            cast(v)
        }
    }

    impl core::ops::Add for int8 {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
                    Self(unsafe { _mm256_add_epi32(self.0, rhs.0) })
                } else {
                    Self(self.0 + rhs.0, self.1 + rhs.1)
                }
            }
        }
    }

    impl core::ops::BitAnd for int8 {
        type Output = Self;

        fn bitand(self, rhs: Self) -> Self::Output {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
                    Self(unsafe { _mm256_and_si256(self.0, rhs.0) })
                } else {
                    Self(self.0 & rhs.0, self.1 & rhs.1)
                }
            }
        }
    }

    impl core::ops::Mul for int8 {
        type Output = Self;

        fn mul(self, rhs: Self) -> Self::Output {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
                    Self(unsafe { _mm256_mullo_epi32(self.0, rhs.0) })
                } else {
                    Self(self.0 * rhs.0, self.1 * rhs.1)
                }
            }
        }
    }

    impl core::ops::BitOr for int8 {
        type Output = Self;

        #[inline]
        fn bitor(self, rhs: Self) -> Self::Output {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
                    Self(unsafe { _mm256_or_si256(self.0, rhs.0) })
                } else {
                    Self(self.0 | rhs.0, self.1 | rhs.1)
                }
            }
        }
    }

    impl core::ops::BitXor for int8 {
        type Output = Self;

        #[inline]
        fn bitxor(self, rhs: Self) -> Self::Output {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
                    Self(unsafe { _mm256_xor_si256(self.0, rhs.0) })
                } else {
                    Self(self.0 ^ rhs.0, self.1 ^ rhs.1)
                }
            }
        }
    }

    // Migrated unchanged in behavior from tiny-skia-0.12.0/src/wide/f32x16_t.rs.

    // Copyright 2020 Yevhenii Reizner
    //
    // Use of this source code is governed by a BSD-style license that can be
    // found in the LICENSE file.

    #[derive(Copy, Clone, Debug)]
    #[repr(C, align(32))]
    pub struct Float16(pub float8, pub float8);

    unsafe impl bytemuck::Zeroable for Float16 {}
    unsafe impl bytemuck::Pod for Float16 {}

    impl Default for Float16 {
        fn default() -> Self {
            Self::splat(0.0)
        }
    }

    impl Float16 {
        pub fn splat(n: f32) -> Self {
            Self(float8::splat(n), float8::splat(n))
        }

        #[inline]
        pub fn abs(&self) -> Self {
            // Yes, Skia does it in the same way.
            let abs = |x| bytemuck::cast::<i32, f32>(bytemuck::cast::<f32, i32>(x) & 0x7fffffff);

            let n0: [f32; 8] = self.0.into();
            let n1: [f32; 8] = self.1.into();
            Self(
                float8::from([
                    abs(n0[0]),
                    abs(n0[1]),
                    abs(n0[2]),
                    abs(n0[3]),
                    abs(n0[4]),
                    abs(n0[5]),
                    abs(n0[6]),
                    abs(n0[7]),
                ]),
                float8::from([
                    abs(n1[0]),
                    abs(n1[1]),
                    abs(n1[2]),
                    abs(n1[3]),
                    abs(n1[4]),
                    abs(n1[5]),
                    abs(n1[6]),
                    abs(n1[7]),
                ]),
            )
        }

        pub fn cmp_gt(self, rhs: &Self) -> Self {
            Self(self.0.cmp_gt(rhs.0), self.1.cmp_gt(rhs.1))
        }

        pub fn blend(self, t: Self, f: Self) -> Self {
            Self(self.0.blend(t.0, f.0), self.1.blend(t.1, f.1))
        }

        pub fn normalize(&self) -> Self {
            Self(self.0.normalize(), self.1.normalize())
        }

        pub fn floor(&self) -> Self {
            // Yes, Skia does it in the same way.
            let roundtrip = self.round();
            roundtrip
                - roundtrip
                    .cmp_gt(self)
                    .blend(Float16::splat(1.0), Float16::splat(0.0))
        }

        pub fn sqrt(&self) -> Self {
            Self(self.0.sqrt(), self.1.sqrt())
        }

        pub fn round(&self) -> Self {
            Self(self.0.round(), self.1.round())
        }

        // This method is too heavy and shouldn't be inlined.
        pub fn save_to_u16x16(&self, dst: &mut UShort16) {
            // Do not use to_i32x8, because it involves rounding,
            // and Skia cast's without it.

            let n0: [f32; 8] = self.0.into();
            let n1: [f32; 8] = self.1.into();

            dst.0[0] = n0[0] as u16;
            dst.0[1] = n0[1] as u16;
            dst.0[2] = n0[2] as u16;
            dst.0[3] = n0[3] as u16;

            dst.0[4] = n0[4] as u16;
            dst.0[5] = n0[5] as u16;
            dst.0[6] = n0[6] as u16;
            dst.0[7] = n0[7] as u16;

            dst.0[8] = n1[0] as u16;
            dst.0[9] = n1[1] as u16;
            dst.0[10] = n1[2] as u16;
            dst.0[11] = n1[3] as u16;

            dst.0[12] = n1[4] as u16;
            dst.0[13] = n1[5] as u16;
            dst.0[14] = n1[6] as u16;
            dst.0[15] = n1[7] as u16;
        }
    }

    impl core::ops::Add<Float16> for Float16 {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            Self(self.0 + rhs.0, self.1 + rhs.1)
        }
    }

    impl core::ops::Sub<Float16> for Float16 {
        type Output = Self;

        fn sub(self, rhs: Self) -> Self::Output {
            Self(self.0 - rhs.0, self.1 - rhs.1)
        }
    }

    impl core::ops::Mul<Float16> for Float16 {
        type Output = Self;

        fn mul(self, rhs: Self) -> Self::Output {
            Self(self.0 * rhs.0, self.1 * rhs.1)
        }
    }

    // Migrated unchanged in behavior from tiny-skia-path-0.12.0/src/f32x2_t.rs.

    // Copyright 2020 Yevhenii Reizner
    //
    // Use of this source code is governed by a BSD-style license that can be
    // found in the LICENSE file.

    // Right now, there are no visible benefits of using SIMD for float2. So we don't.
    /// A pair of f32 numbers.
    ///
    /// Mainly for internal use. Do not rely on it!
    #[allow(non_camel_case_types)]
    #[derive(Copy, Clone, Default, PartialEq, Debug)]
    pub struct float2(pub [f32; 2]);

    impl float2 {
        /// Creates a new pair.
        pub fn new(a: f32, b: f32) -> float2 {
            float2([a, b])
        }

        /// Creates a new pair from a single value.
        pub fn splat(x: f32) -> float2 {
            float2([x, x])
        }

        /// Returns an absolute value.
        pub fn abs(self) -> float2 {
            float2([self.x().abs(), self.y().abs()])
        }

        /// Returns a minimum value.
        pub fn min(self, other: float2) -> float2 {
            float2([pmin(self.x(), other.x()), pmin(self.y(), other.y())])
        }

        /// Returns a maximum value.
        pub fn max(self, other: float2) -> float2 {
            float2([pmax(self.x(), other.x()), pmax(self.y(), other.y())])
        }

        /// Returns a maximum of both values.
        pub fn max_component(self) -> f32 {
            pmax(self.x(), self.y())
        }

        /// Returns the first value.
        pub fn x(&self) -> f32 {
            self.0[0]
        }

        /// Returns the second value.
        pub fn y(&self) -> f32 {
            self.0[1]
        }
    }

    impl core::ops::Add<float2> for float2 {
        type Output = float2;

        fn add(self, other: float2) -> float2 {
            float2([self.x() + other.x(), self.y() + other.y()])
        }
    }

    impl core::ops::Sub<float2> for float2 {
        type Output = float2;

        fn sub(self, other: float2) -> float2 {
            float2([self.x() - other.x(), self.y() - other.y()])
        }
    }

    impl core::ops::Mul<float2> for float2 {
        type Output = float2;

        fn mul(self, other: float2) -> float2 {
            float2([self.x() * other.x(), self.y() * other.y()])
        }
    }

    impl core::ops::Div<float2> for float2 {
        type Output = float2;

        fn div(self, other: float2) -> float2 {
            float2([self.x() / other.x(), self.y() / other.y()])
        }
    }

    // A faster and more forgiving f32 min/max implementation.
    //
    // Unlike std one, we do not care about NaN.

    fn pmax(a: f32, b: f32) -> f32 {
        if a < b {
            b
        } else {
            a
        }
    }

    fn pmin(a: f32, b: f32) -> f32 {
        if b < a {
            b
        } else {
            a
        }
    }

    // Migrated unchanged in behavior from tiny-skia-path-0.12.0/src/f32x4_t.rs.

    /// Local scalar geometry vector: retained separately because min/max NaN behavior differs
    /// from the raster float4 type. Responsibility counterpart: skvx::float4.

    // Copyright 2020 Yevhenii Reizner
    //
    // Use of this source code is governed by a BSD-style license that can be
    // found in the LICENSE file.

    // Right now, there are no visible benefits of using SIMD for PathFloat4. So we don't.
    #[derive(Default, Clone, Copy, PartialEq, Debug)]
    #[repr(C, align(16))]
    pub struct PathFloat4(pub [f32; 4]);

    impl PathFloat4 {
        pub fn max(self, rhs: Self) -> Self {
            Self([
                self.0[0].max(rhs.0[0]),
                self.0[1].max(rhs.0[1]),
                self.0[2].max(rhs.0[2]),
                self.0[3].max(rhs.0[3]),
            ])
        }

        pub fn min(self, rhs: Self) -> Self {
            Self([
                self.0[0].min(rhs.0[0]),
                self.0[1].min(rhs.0[1]),
                self.0[2].min(rhs.0[2]),
                self.0[3].min(rhs.0[3]),
            ])
        }
    }

    impl core::ops::Add for PathFloat4 {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            Self([
                self.0[0] + rhs.0[0],
                self.0[1] + rhs.0[1],
                self.0[2] + rhs.0[2],
                self.0[3] + rhs.0[3],
            ])
        }
    }

    impl core::ops::AddAssign for PathFloat4 {
        fn add_assign(&mut self, rhs: PathFloat4) {
            *self = *self + rhs;
        }
    }

    impl core::ops::Sub for PathFloat4 {
        type Output = Self;

        fn sub(self, rhs: Self) -> Self::Output {
            Self([
                self.0[0] - rhs.0[0],
                self.0[1] - rhs.0[1],
                self.0[2] - rhs.0[2],
                self.0[3] - rhs.0[3],
            ])
        }
    }

    impl core::ops::Mul for PathFloat4 {
        type Output = Self;

        fn mul(self, rhs: Self) -> Self::Output {
            Self([
                self.0[0] * rhs.0[0],
                self.0[1] * rhs.0[1],
                self.0[2] * rhs.0[2],
                self.0[3] * rhs.0[3],
            ])
        }
    }

    impl core::ops::MulAssign for PathFloat4 {
        fn mul_assign(&mut self, rhs: PathFloat4) {
            *self = *self * rhs;
        }
    }

    // Migrated caller spellings.
    pub use float2 as f32x2;
    pub use float4 as f32x4;
    pub use float8 as f32x8;
    pub use int4 as i32x4;
    pub use int8 as i32x8;
    pub use uint4 as u32x4;
    pub use uint8 as u32x8;
    pub use Float16 as f32x16;
    pub use UShort16 as u16x16;
}
