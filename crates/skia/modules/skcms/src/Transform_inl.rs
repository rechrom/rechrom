// Copyright 2018 Google LLC. BSD-style license.
// Native textual template inclusion becomes independent Rust namespace instantiation.
#[cfg(all(target_arch = "aarch64", not(skcms_portable)))]
mod neon {
    // Copyright 2018 Google Inc. BSD-style license.
    // Selected CPU configuration: ARM64 Clang baseline N=4 with actual NEON primitives.
    // Array register lanes preserve values; Rust layout/ABI and explicit spill/load are adaptations.

    use std::arch::aarch64::*;
    // Source: modules/skcms/src/Transform_inl.h:14-19
    pub type F = crate::modules::skcms::src::skcms_TransformBaseline::V<f32>;
    pub type I32 = crate::modules::skcms::src::skcms_TransformBaseline::V<i32>;
    pub type U64 = crate::modules::skcms::src::skcms_TransformBaseline::V<u64>;
    pub type U32 = crate::modules::skcms::src::skcms_TransformBaseline::V<u32>;
    pub type U16 = crate::modules::skcms::src::skcms_TransformBaseline::V<u16>;
    pub type U8 = crate::modules::skcms::src::skcms_TransformBaseline::V<u8>;
    // Source: modules/skcms/src/Transform_inl.h:27-29
    pub const F0: F = [0.0; 4];
    pub const F1: F = [1.0; 4];
    pub const FInfBits: F = [0x7f800000u32 as f32; 4];
    // Source: modules/skcms/src/Transform_inl.h:97-102
    /// Caller supplies readable storage for all bytes of T; no ownership is transferred.
    pub unsafe fn load<T: Copy, P>(ptr: *const P) -> T {
        unsafe { ptr.cast::<T>().read_unaligned() }
    }
    // Source: modules/skcms/src/Transform_inl.h:103-106
    /// Caller supplies exclusive writable storage for all bytes of T.
    pub unsafe fn store<T: Copy, P>(ptr: *mut P, val: &T) {
        unsafe { ptr.cast::<T>().write_unaligned(*val) }
    }
    // Source: modules/skcms/src/Transform_inl.h:110-118
    // Rust trait dispatch replaces the native compile-time convertvector intrinsic.
    // Primitive casts match all native-defined lane conversions; Rust defines saturation for float overflow.
    pub trait CastLane<D>: Copy {
        fn cast_lane(self) -> D;
    }
    macro_rules! cast_lanes {($src:ty;$($dst:ty),*)=>{$(impl CastLane<$dst> for $src {fn cast_lane(self)->$dst{self as $dst}})*};}
    cast_lanes!(f32;f32,i32,u64,u32,u16,u8);
    cast_lanes!(i32;f32,i32,u64,u32,u16,u8);
    cast_lanes!(u64;f32,i32,u64,u32,u16,u8);
    cast_lanes!(u32;f32,i32,u64,u32,u16,u8);
    cast_lanes!(u16;f32,i32,u64,u32,u16,u8);
    cast_lanes!(u8;f32,i32,u64,u32,u16,u8);
    pub fn cast<D: Copy, S: CastLane<D>>(v: &[S; 4]) -> [D; 4] {
        v.map(CastLane::cast_lane)
    }
    // Source: modules/skcms/src/Transform_inl.h:120-124
    /// Caller guarantees the copied bytes form a valid D value.
    pub unsafe fn bit_pun<D: Copy, S: Copy>(v: &S) -> D {
        assert_eq!(std::mem::size_of::<D>(), std::mem::size_of::<S>());
        unsafe { load(v) }
    }
    // Source: modules/skcms/src/Transform_inl.h:129-129
    pub unsafe fn to_fixed(f: F) -> U32 {
        std::array::from_fn(|i| unsafe { (f[i] + 0.5).to_int_unchecked::<i32>() as u32 })
    }
    // Source: modules/skcms/src/Transform_inl.h:140-144
    pub fn if_then_else(cond: I32, t: F, e: F) -> F {
        std::array::from_fn(|i| {
            f32::from_bits((cond[i] as u32 & t[i].to_bits()) | (!cond[i] as u32 & e[i].to_bits()))
        })
    }
    // Source: modules/skcms/src/Transform_inl.h:148-150
    pub fn F_from_Half(half: U16) -> F {
        unsafe {
            let mut out = F0;
            vst1q_f32(
                out.as_mut_ptr(),
                vcvt_f32_f16(vreinterpret_f16_u16(vld1_u16(half.as_ptr()))),
            );
            out
        }
    }
    // Source: modules/skcms/src/Transform_inl.h:180-182
    // LLVM can fold a half→float→half pair without preserving signaling-NaN quieting.
    // The opaque value barrier keeps both native conversions observable.
    pub fn Half_from_F(f: F) -> U16 {
        let f = std::hint::black_box(f);
        unsafe {
            let mut out = [0; 4];
            vst1_u16(
                out.as_mut_ptr(),
                vreinterpret_u16_f16(vcvt_f16_f32(vld1q_f32(f.as_ptr()))),
            );
            out
        }
    }
    // Source: modules/skcms/src/Transform_inl.h:200-203
    pub fn swap_endian_16(v: U16) -> U16 {
        unsafe {
            let mut out = [0; 4];
            vst1_u16(
                out.as_mut_ptr(),
                vreinterpret_u16_u8(vrev16_u8(vreinterpret_u8_u16(vld1_u16(v.as_ptr())))),
            );
            out
        }
    }
    // Source: modules/skcms/src/Transform_inl.h:206-209
    pub fn swap_endian_16x4(rgba: &U64) -> U64 {
        rgba.map(|x| ((x & 0x00ff00ff00ff00ff) << 8) | ((x & 0xff00ff00ff00ff00) >> 8))
    }
    // Source: modules/skcms/src/Transform_inl.h:212-213
    pub fn min_(x: F, y: F) -> F {
        unsafe {
            let mut out = F0;
            vst1q_f32(
                out.as_mut_ptr(),
                vminq_f32(vld1q_f32(x.as_ptr()), vld1q_f32(y.as_ptr())),
            );
            out
        }
    }
    pub fn max_(x: F, y: F) -> F {
        unsafe {
            let mut out = F0;
            vst1q_f32(
                out.as_mut_ptr(),
                vmaxq_f32(vld1q_f32(x.as_ptr()), vld1q_f32(y.as_ptr())),
            );
            out
        }
    }
    // Source: modules/skcms/src/Transform_inl.h:222-226
    pub fn floor_(x: F) -> F {
        unsafe {
            let mut out = F0;
            vst1q_f32(out.as_mut_ptr(), vrndmq_f32(vld1q_f32(x.as_ptr())));
            out
        }
    }
    // Source: modules/skcms/src/Transform_inl.h:250-262
    pub fn approx_log2(x: F) -> F {
        std::array::from_fn(|i| {
            let bits = x[i].to_bits() as i32;
            let e = bits as f32 * (1.0 / (1 << 23) as f32);
            let m = f32::from_bits(((bits & 0x007fffff) | 0x3f000000) as u32);
            e - 124.225514990_f32 - 1.498030302_f32 * m - 1.725879990_f32 / (0.3520887068_f32 + m)
        })
    }
    // Source: modules/skcms/src/Transform_inl.h:264-267
    pub fn approx_log(x: F) -> F {
        approx_log2(x).map(|x| 0.69314718_f32 * x)
    }
    // Source: modules/skcms/src/Transform_inl.h:269-279
    pub fn approx_exp2(x: F) -> F {
        let floor = floor_(x);
        let fbits = std::array::from_fn(|i| {
            let fract = x[i] - floor[i];
            (1.0 * (1 << 23) as f32)
                * (x[i] + 121.274057500_f32 - 1.490129070_f32 * fract
                    + 27.728023300_f32 / (4.84252568_f32 - fract))
        });
        let v = min_(max_(fbits, F0), FInfBits);
        std::array::from_fn(|i| f32::from_bits((v[i] as i32) as u32))
    }
    // Source: modules/skcms/src/Transform_inl.h:280-283
    pub fn approx_pow(x: F, y: f32) -> F {
        let calculated = approx_exp2(approx_log2(x).map(|x| x * y));
        std::array::from_fn(|i| {
            if x[i] == 0.0 || x[i] == 1.0 {
                x[i]
            } else {
                calculated[i]
            }
        })
    }
    // Source: modules/skcms/src/Transform_inl.h:285-288
    pub fn approx_exp(x: F) -> F {
        approx_exp2(x.map(|x| 1.4426950408889634074_f32 * x))
    }
    // Source: modules/skcms/src/Transform_inl.h:290-294
    pub fn strip_sign(x: F, sign: &mut U32) -> F {
        std::array::from_fn(|i| {
            let bits = x[i].to_bits();
            sign[i] = bits & 0x80000000;
            f32::from_bits(bits ^ sign[i])
        })
    }
    // Source: modules/skcms/src/Transform_inl.h:296-298
    pub fn apply_sign(x: F, sign: U32) -> F {
        std::array::from_fn(|i| f32::from_bits(sign[i] | x[i].to_bits()))
    }

    use crate::modules::skcms::src::skcms_public::skcms_TransferFunction;
    // Source: modules/skcms/src/Transform_inl.h:301-312
    pub fn apply_tf(tf: &skcms_TransferFunction, mut x: F) -> F {
        let mut sign = [0; 4];
        x = strip_sign(x, &mut sign);
        let exponential = approx_pow(x.map(|x| tf.a * x + tf.b), tf.g);
        let v = std::array::from_fn(|i| {
            if x[i] < tf.d {
                tf.c * x[i] + tf.f
            } else {
                exponential[i] + tf.e
            }
        });
        apply_sign(v, sign)
    }
    // Source: modules/skcms/src/Transform_inl.h:315-319
    pub fn apply_gamma(tf: &skcms_TransferFunction, mut x: F) -> F {
        let mut sign = [0; 4];
        x = strip_sign(x, &mut sign);
        apply_sign(approx_pow(x, tf.g), sign)
    }
    // Source: modules/skcms/src/Transform_inl.h:321-331
    pub fn apply_pq(tf: &skcms_TransferFunction, mut x: F) -> F {
        let mut sign = [0; 4];
        x = strip_sign(x, &mut sign);
        let powered = approx_pow(x, tf.c);
        let numerator = max_(powered.map(|v| tf.a + tf.b * v), F0);
        let v = approx_pow(
            std::array::from_fn(|i| numerator[i] / (tf.d + tf.e * powered[i])),
            tf.f,
        );
        apply_sign(v, sign)
    }
    // Source: modules/skcms/src/Transform_inl.h:333-345
    pub fn apply_hlg(tf: &skcms_TransferFunction, mut x: F) -> F {
        let (r, g, a, b, c, k) = (tf.a, tf.b, tf.c, tf.d, tf.e, tf.f + 1.0);
        let mut sign = [0; 4];
        x = strip_sign(x, &mut sign);
        let power = approx_pow(x.map(|x| x * r), g);
        let exponential = approx_exp(x.map(|x| (x - c) * a));
        let v = std::array::from_fn(|i| {
            if x[i] * r <= 1.0 {
                power[i]
            } else {
                exponential[i] + b
            }
        });
        apply_sign(v, sign).map(|v| k * v)
    }
    // Source: modules/skcms/src/Transform_inl.h:347-360
    pub fn apply_hlginv(tf: &skcms_TransferFunction, mut x: F) -> F {
        let (r, g, a, b, c, k) = (tf.a, tf.b, tf.c, tf.d, tf.e, tf.f + 1.0);
        let mut sign = [0; 4];
        x = strip_sign(x, &mut sign);
        x = x.map(|x| x / k);
        let power = approx_pow(x, g);
        let logarithm = approx_log(x.map(|x| x - b));
        let v = std::array::from_fn(|i| {
            if x[i] <= 1.0 {
                r * power[i]
            } else {
                a * logarithm[i] + c
            }
        });
        apply_sign(v, sign)
    }
    // Source: modules/skcms/src/Transform_inl.h:365-369
    pub fn compute_Y_in_xyzd50(x: F, y: F, z: F) -> F {
        std::array::from_fn(|i| {
            -0.02831655_f32 * x[i] + 1.00995452_f32 * y[i] + 0.02102382_f32 * z[i]
        })
    }
    // Source: modules/skcms/src/Transform_inl.h:372-384
    /// Caller supplies readable strided storage through p[9].
    pub unsafe fn load_3<D: Copy, P: CastLane<D>>(p: *const P) -> [D; 4] {
        unsafe {
            [
                (*p).cast_lane(),
                (*p.add(3)).cast_lane(),
                (*p.add(6)).cast_lane(),
                (*p.add(9)).cast_lane(),
            ]
        }
    }
    // Source: modules/skcms/src/Transform_inl.h:386-398
    /// Caller supplies readable strided storage through p[12].
    pub unsafe fn load_4<D: Copy, P: CastLane<D>>(p: *const P) -> [D; 4] {
        unsafe {
            [
                (*p).cast_lane(),
                (*p.add(4)).cast_lane(),
                (*p.add(8)).cast_lane(),
                (*p.add(12)).cast_lane(),
            ]
        }
    }
    // Source: modules/skcms/src/Transform_inl.h:400-415
    /// Caller supplies exclusive writable strided storage through p[9].
    pub unsafe fn store_3<T: CastLane<P>, P: Copy>(p: *mut P, v: &[T; 4]) {
        unsafe {
            p.write(v[0].cast_lane());
            p.add(3).write(v[1].cast_lane());
            p.add(6).write(v[2].cast_lane());
            p.add(9).write(v[3].cast_lane());
        }
    }
    // Source: modules/skcms/src/Transform_inl.h:417-432
    /// Caller supplies exclusive writable strided storage through p[12].
    pub unsafe fn store_4<T: CastLane<P>, P: Copy>(p: *mut P, v: &[T; 4]) {
        unsafe {
            p.write(v[0].cast_lane());
            p.add(4).write(v[1].cast_lane());
            p.add(8).write(v[2].cast_lane());
            p.add(12).write(v[3].cast_lane());
        }
    }
    // Source: modules/skcms/src/Transform_inl.h:435-450
    /// Caller supplies readable storage for all four signed byte indices.
    pub unsafe fn gather_8(p: *const u8, ix: I32) -> U8 {
        std::array::from_fn(|i| unsafe { *p.offset(ix[i] as isize) })
    }
    // Source: modules/skcms/src/Transform_inl.h:452-471
    /// Caller supplies readable storage for the indexed 16-bit values.
    pub unsafe fn gather_16(p: *const u8, ix: I32) -> U16 {
        std::array::from_fn(|i| unsafe { load(p.offset(2 * ix[i] as isize)) })
    }
    // Source: modules/skcms/src/Transform_inl.h:473-493
    /// Caller supplies readable storage for the indexed 32-bit values.
    pub unsafe fn gather_32(p: *const u8, ix: I32) -> U32 {
        std::array::from_fn(|i| unsafe { load(p.offset(4 * ix[i] as isize)) })
    }
    // Source: modules/skcms/src/Transform_inl.h:495-531
    /// Caller supplies readable storage including the native fourth overread byte for each index.
    pub unsafe fn gather_24(p: *const u8, ix: I32) -> U32 {
        std::array::from_fn(
            |i| unsafe { load::<u32, _>(p.offset(3 * ix[i] as isize)) } & 0x00ffffff,
        )
    }

    // Source: modules/skcms/src/Transform_inl.h:533-587
    /// Caller supplies eight readable bytes per six-byte indexed RGB entry (the native two-byte overread).
    pub unsafe fn gather_48(p: *const u8, ix: I32, v: &mut U64) {
        *v = std::array::from_fn(
            |i| unsafe { load::<u64, _>(p.offset(6 * ix[i] as isize)) } & 0x0000ffffffffffff,
        );
    }
    // Source: modules/skcms/src/Transform_inl.h:589-591
    pub fn F_from_U8(v: U8) -> F {
        cast::<f32, _>(&v).map(|x| x * (1.0 / 255.0))
    }
    // Source: modules/skcms/src/Transform_inl.h:593-599
    pub fn F_from_U16_BE(v: U16) -> F {
        let lo = v.map(|v| v >> 8);
        let hi = v.map(|v| v.wrapping_shl(8) & 0xffff);
        cast::<f32, _>(&std::array::from_fn(|i| lo[i] | hi[i])).map(|x| x * (1.0 / 65535.0))
    }
    // Source: modules/skcms/src/Transform_inl.h:601-604
    pub fn U16_from_F(v: F) -> U16 {
        cast(&v.map(|x| x * 65535.0 + 0.5))
    }
    // Source: modules/skcms/src/Transform_inl.h:606-608
    pub fn minus_1_ulp(v: F) -> F {
        v.map(|x| f32::from_bits(x.to_bits().wrapping_sub(1)))
    }
    // Source: modules/skcms/src/Transform_inl.h:610-632
    /// Caller supplies a nonempty valid borrowed curve table and native-defined index conversion inputs.
    pub unsafe fn table(curve: &crate::modules::skcms::src::skcms_public::skcms_Curve, v: F) -> F {
        let curve = unsafe { curve.table };
        let ix = max_(F0, min_(v, F1)).map(|x| x * (curve.table_entries - 1) as f32);
        let lo: I32 = cast(&ix);
        let hi: I32 = cast(&minus_1_ulp(ix.map(|x| x + 1.0)));
        let t = std::array::from_fn::<_, 4, _>(|i| ix[i] - lo[i] as f32);
        let (l, h) = if !curve.table_8.is_null() {
            unsafe {
                (
                    F_from_U8(gather_8(curve.table_8, lo)),
                    F_from_U8(gather_8(curve.table_8, hi)),
                )
            }
        } else {
            unsafe {
                (
                    F_from_U16_BE(gather_16(curve.table_16, lo)),
                    F_from_U16_BE(gather_16(curve.table_16, hi)),
                )
            }
        };
        std::array::from_fn(|i| l[i] + (h[i] - l[i]) * t[i])
    }
    // Source: modules/skcms/src/Transform_inl.h:634-640
    /// Caller supplies valid RGB grid storage including native gather24 overread bytes.
    pub unsafe fn sample_clut_8_rgb(grid: *const u8, ix: I32, r: &mut F, g: &mut F, b: &mut F) {
        let rgb = unsafe { gather_24(grid, ix) };
        *r = rgb.map(|x| ((x >> 0) & 0xff) as f32 * (1.0 / 255.0));
        *g = rgb.map(|x| ((x >> 8) & 0xff) as f32 * (1.0 / 255.0));
        *b = rgb.map(|x| ((x >> 16) & 0xff) as f32 * (1.0 / 255.0));
    }
    // Source: modules/skcms/src/Transform_inl.h:642-650
    /// Caller supplies valid four-channel grid storage.
    pub unsafe fn sample_clut_8_rgba(
        grid: *const u8,
        ix: I32,
        r: &mut F,
        g: &mut F,
        b: &mut F,
        a: &mut F,
    ) {
        let rgba = unsafe { gather_32(grid, ix) };
        *r = rgba.map(|x| ((x >> 0) & 0xff) as f32 * (1.0 / 255.0));
        *g = rgba.map(|x| ((x >> 8) & 0xff) as f32 * (1.0 / 255.0));
        *b = rgba.map(|x| ((x >> 16) & 0xff) as f32 * (1.0 / 255.0));
        *a = rgba.map(|x| ((x >> 24) & 0xff) as f32 * (1.0 / 255.0));
    }
    // Source: modules/skcms/src/Transform_inl.h:652-668
    /// Caller supplies valid RGB16 grid storage including native gather48 overread bytes.
    pub unsafe fn sample_clut_16_rgb(grid: *const u8, ix: I32, r: &mut F, g: &mut F, b: &mut F) {
        let mut rgb = [0; 4];
        unsafe { gather_48(grid, ix, &mut rgb) };
        rgb = swap_endian_16x4(&rgb);
        *r = rgb.map(|x| ((x >> 0) & 0xffff) as f32 * (1.0 / 65535.0));
        *g = rgb.map(|x| ((x >> 16) & 0xffff) as f32 * (1.0 / 65535.0));
        *b = rgb.map(|x| ((x >> 32) & 0xffff) as f32 * (1.0 / 65535.0));
    }
    // Source: modules/skcms/src/Transform_inl.h:670-676
    /// Caller supplies valid RGBA16 grid storage.
    pub unsafe fn sample_clut_16_rgba(
        grid: *const u8,
        ix: I32,
        r: &mut F,
        g: &mut F,
        b: &mut F,
        a: &mut F,
    ) {
        unsafe {
            *r = F_from_U16_BE(gather_16(grid, ix.map(|i| 4 * i)));
            *g = F_from_U16_BE(gather_16(grid, ix.map(|i| 4 * i + 1)));
            *b = F_from_U16_BE(gather_16(grid, ix.map(|i| 4 * i + 2)));
            *a = F_from_U16_BE(gather_16(grid, ix.map(|i| 4 * i + 3)));
        }
    }
    // Source: modules/skcms/src/Transform_inl.h:678-760
    /// Caller supplies a valid borrowed ICC CLUT with native readable gather-overrun bytes.
    /// The four mutable channel references must identify distinct registers.
    pub unsafe fn clut(
        input_channels: u32,
        output_channels: u32,
        grid_points: &[u8; 4],
        grid_8: *const u8,
        grid_16: *const u8,
        r: &mut F,
        g: &mut F,
        b: &mut F,
        a: &mut F,
    ) {
        let dim = input_channels as i32;
        if dim <= 0 || dim > 4 {
            return;
        }
        assert!(output_channels == 3 || output_channels == 4);
        let mut index = [[0i32; 4]; 8];
        let mut weight = [F0; 8];
        let inputs = [*r, *g, *b, *a];
        let mut stride = 1;
        for i in (0..dim as usize).rev() {
            let x = max_(F0, min_(inputs[i], F1)).map(|x| x * (grid_points[i] as i32 - 1) as f32);
            let lo: I32 = cast(&x);
            let hi: I32 = cast(&minus_1_ulp(x.map(|x| x + 1.0)));
            index[i] = lo.map(|x| x * stride);
            index[i + 4] = hi.map(|x| x * stride);
            stride *= grid_points[i] as i32;
            let t = std::array::from_fn::<_, 4, _>(|j| x[j] - lo[j] as f32);
            weight[i] = t.map(|x| 1.0 - x);
            weight[i + 4] = t;
        }
        *r = F0;
        *g = F0;
        *b = F0;
        if output_channels == 4 {
            *a = F0;
        }
        for combo in 0..(1 << dim) {
            let mut ix = index[(combo & 1) * 4];
            let mut w = weight[(combo & 1) * 4];
            // Native switch falls through in the order dimension3, dimension2, dimension1.
            if (dim - 1) & 3 >= 3 {
                let k = 3 + (combo & 8) / 2;
                for j in 0..4 {
                    ix[j] += index[k][j];
                    w[j] *= weight[k][j];
                }
            }
            if (dim - 1) & 3 >= 2 {
                let k = 2 + (combo & 4);
                for j in 0..4 {
                    ix[j] += index[k][j];
                    w[j] *= weight[k][j];
                }
            }
            if (dim - 1) & 3 >= 1 {
                let k = 1 + (combo & 2) * 2;
                for j in 0..4 {
                    ix[j] += index[k][j];
                    w[j] *= weight[k][j];
                }
            }
            let (mut rr, mut gg, mut bb, mut aa) = (F0, F0, F0, F0);
            if output_channels == 3 {
                if !grid_8.is_null() {
                    unsafe { sample_clut_8_rgb(grid_8, ix, &mut rr, &mut gg, &mut bb) }
                } else {
                    unsafe { sample_clut_16_rgb(grid_16, ix, &mut rr, &mut gg, &mut bb) }
                }
            } else {
                if !grid_8.is_null() {
                    unsafe { sample_clut_8_rgba(grid_8, ix, &mut rr, &mut gg, &mut bb, &mut aa) }
                } else {
                    unsafe { sample_clut_16_rgba(grid_16, ix, &mut rr, &mut gg, &mut bb, &mut aa) }
                }
            }
            for j in 0..4 {
                r[j] += w[j] * rr[j];
                g[j] += w[j] * gg[j];
                b[j] += w[j] * bb[j];
                a[j] += w[j] * aa[j];
            }
        }
    }
    // Source: modules/skcms/src/Transform_inl.h:762-766
    pub unsafe fn clut_A2B(
        a2b: &crate::modules::skcms::src::skcms_public::skcms_A2B,
        r: &mut F,
        g: &mut F,
        b: &mut F,
        mut a: F,
    ) {
        unsafe {
            clut(
                a2b.input_channels,
                a2b.output_channels,
                &a2b.grid_points,
                a2b.grid_8,
                a2b.grid_16,
                r,
                g,
                b,
                &mut a,
            )
        }
    }
    // Source: modules/skcms/src/Transform_inl.h:767-771
    pub unsafe fn clut_B2A(
        b2a: &crate::modules::skcms::src::skcms_public::skcms_B2A,
        r: &mut F,
        g: &mut F,
        b: &mut F,
        a: &mut F,
    ) {
        unsafe {
            clut(
                b2a.input_channels,
                b2a.output_channels,
                &b2a.grid_points,
                b2a.grid_8,
                b2a.grid_16,
                r,
                g,
                b,
                a,
            )
        }
    }

    // Source: modules/skcms/src/Transform_inl.h:773-773
    #[derive(Clone, Copy)]
    pub struct NoCtx;
    // Source: modules/skcms/src/Transform_inl.h:775-779
    #[derive(Clone, Copy)]
    pub struct Ctx {
        pub fArg: *const std::ffi::c_void,
    }
    impl Ctx {
        pub fn no_ctx(self) -> NoCtx {
            NoCtx
        }
        pub fn as_ptr<T>(self) -> *const T {
            self.fArg.cast()
        }
    }
    // Source: modules/skcms/src/Transform_inl.h:781-787
    // Rust function references retain the four distinct native mutable register leases.
    pub type StageFn = unsafe fn(Ctx, *const u8, *mut u8, &mut F, &mut F, &mut F, &mut F, i32);
    // Source: modules/skcms/src/Transform_inl.h:822-832
    // Selected SKCMS_HAS_MUSTTAIL=0: each wrapper executes its actual kernel and returns to dispatcher.
    macro_rules! stage_noctx {
        ($name:ident,$kernel:ident,|$src:ident,$dst:ident,$r:ident,$g:ident,$b:ident,$a:ident,$i:ident|$body:block) => {
            pub unsafe fn $name(
                ctx: Ctx,
                src: *const u8,
                dst: *mut u8,
                r: &mut F,
                g: &mut F,
                b: &mut F,
                a: &mut F,
                i: i32,
            ) {
                unsafe { $kernel(ctx.no_ctx(), src, dst, r, g, b, a, i) }
            }
            pub unsafe fn $kernel(
                _ctx: NoCtx,
                $src: *const u8,
                $dst: *mut u8,
                $r: &mut F,
                $g: &mut F,
                $b: &mut F,
                $a: &mut F,
                $i: i32,
            ) {
                unsafe { $body }
            }
        };
    }
    // Native context-taking expansion of the same DECLARE_STAGE macro above.
    macro_rules! stage_ctx {
        ($name:ident,$kernel:ident,$ty:ty,|$ctx:ident,$src:ident,$dst:ident,$r:ident,$g:ident,$b:ident,$a:ident,$i:ident|$body:block) => {
            pub unsafe fn $name(
                ctx: Ctx,
                src: *const u8,
                dst: *mut u8,
                r: &mut F,
                g: &mut F,
                b: &mut F,
                a: &mut F,
                i: i32,
            ) {
                unsafe { $kernel(&*ctx.as_ptr::<$ty>(), src, dst, r, g, b, a, i) }
            }
            pub unsafe fn $kernel(
                $ctx: &$ty,
                $src: *const u8,
                $dst: *mut u8,
                $r: &mut F,
                $g: &mut F,
                $b: &mut F,
                $a: &mut F,
                $i: i32,
            ) {
                unsafe { $body }
            }
        };
    }
    // Source: modules/skcms/src/Transform_inl.h:836-838
    stage_noctx!(Exec_load_a8, Exec_load_a8_k, |src, dst, r, g, b, a, i| {
        *a = F_from_U8(load(src.offset(i as isize)));
    });
    // Source: modules/skcms/src/Transform_inl.h:840-842
    stage_noctx!(Exec_load_g8, Exec_load_g8_k, |src, dst, r, g, b, a, i| {
        *r = F_from_U8(load(src.offset(i as isize)));
        *g = *r;
        *b = *r;
    });
    // Source: modules/skcms/src/Transform_inl.h:844-848
    stage_noctx!(
        Exec_load_ga88,
        Exec_load_ga88_k,
        |src, dst, r, g, b, a, i| {
            let u16: U16 = load(src.offset(2 * i as isize));
            *r = u16.map(|x| (x & 0xff) as f32 * (1.0 / 255.0));
            *g = *r;
            *b = *r;
            *a = u16.map(|x| ((x >> 8) & 0xff) as f32 * (1.0 / 255.0));
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:850-857
    stage_noctx!(
        Exec_load_4444,
        Exec_load_4444_k,
        |src, dst, r, g, b, a, i| {
            let abgr: U16 = load(src.offset(2 * i as isize));
            *r = abgr.map(|x| ((x >> 12) & 0xf) as f32 * (1.0 / 15.0));
            *g = abgr.map(|x| ((x >> 8) & 0xf) as f32 * (1.0 / 15.0));
            *b = abgr.map(|x| ((x >> 4) & 0xf) as f32 * (1.0 / 15.0));
            *a = abgr.map(|x| (x & 0xf) as f32 * (1.0 / 15.0));
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:859-865
    stage_noctx!(Exec_load_565, Exec_load_565_k, |src, dst, r, g, b, a, i| {
        let rgb: U16 = load(src.offset(2 * i as isize));
        *r = rgb.map(|x| (x & 31) as f32 * (1.0 / 31.0));
        *g = rgb.map(|x| (x & (63 << 5)) as f32 * (1.0 / (63 << 5) as f32));
        *b = rgb.map(|x| (x & (31 << 11)) as f32 * (1.0 / (31 << 11) as f32));
    });
    // Source: modules/skcms/src/Transform_inl.h:867-890
    stage_noctx!(Exec_load_888, Exec_load_888_k, |src, dst, r, g, b, a, i| {
        let rgb = src.offset(3 * i as isize);
        let zero = vdup_n_u8(0);
        let mut v = uint8x8x3_t(zero, zero, zero);
        v = vld3_lane_u8::<0>(rgb, v);
        v = vld3_lane_u8::<2>(rgb.add(3), v);
        v = vld3_lane_u8::<4>(rgb.add(6), v);
        v = vld3_lane_u8::<6>(rgb.add(9), v);
        let mut rr = [0; 4];
        let mut gg = [0; 4];
        let mut bb = [0; 4];
        vst1_u16(rr.as_mut_ptr(), vreinterpret_u16_u8(v.0));
        vst1_u16(gg.as_mut_ptr(), vreinterpret_u16_u8(v.1));
        vst1_u16(bb.as_mut_ptr(), vreinterpret_u16_u8(v.2));
        *r = rr.map(|x| x as f32 * (1.0 / 255.0));
        *g = gg.map(|x| x as f32 * (1.0 / 255.0));
        *b = bb.map(|x| x as f32 * (1.0 / 255.0));
    });
    // Source: modules/skcms/src/Transform_inl.h:892-899
    stage_noctx!(
        Exec_load_8888,
        Exec_load_8888_k,
        |src, dst, r, g, b, a, i| {
            let rgba: U32 = load(src.offset(4 * i as isize));
            *r = rgba.map(|x| ((x >> 0) & 0xff) as f32 * (1.0 / 255.0));
            *g = rgba.map(|x| ((x >> 8) & 0xff) as f32 * (1.0 / 255.0));
            *b = rgba.map(|x| ((x >> 16) & 0xff) as f32 * (1.0 / 255.0));
            *a = rgba.map(|x| ((x >> 24) & 0xff) as f32 * (1.0 / 255.0));
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:901-908
    stage_noctx!(
        Exec_load_1010102,
        Exec_load_1010102_k,
        |src, dst, r, g, b, a, i| {
            let rgba: U32 = load(src.offset(4 * i as isize));
            *r = rgba.map(|x| ((x >> 0) & 0x3ff) as f32 * (1.0 / 1023.0));
            *g = rgba.map(|x| ((x >> 10) & 0x3ff) as f32 * (1.0 / 1023.0));
            *b = rgba.map(|x| ((x >> 20) & 0x3ff) as f32 * (1.0 / 1023.0));
            *a = rgba.map(|x| ((x >> 30) & 0x3) as f32 * (1.0 / 3.0));
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:910-915
    stage_noctx!(
        Exec_load_101010x_XR,
        Exec_load_101010x_XR_k,
        |src, dst, r, g, b, a, i| {
            let rgba: U32 = load(src.offset(4 * i as isize));
            *r = rgba.map(|x| ((x >> 0) & 0x3ff).wrapping_sub(384) as f32 / 510.0);
            *g = rgba.map(|x| ((x >> 10) & 0x3ff).wrapping_sub(384) as f32 / 510.0);
            *b = rgba.map(|x| ((x >> 20) & 0x3ff).wrapping_sub(384) as f32 / 510.0);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:917-924
    stage_noctx!(
        Exec_load_10101010_XR,
        Exec_load_10101010_XR_k,
        |src, dst, r, g, b, a, i| {
            let rgba: U64 = load(src.offset(8 * i as isize));
            *r = rgba.map(|x| ((x >> 6) & 0x3ff).wrapping_sub(384) as f32 / 510.0);
            *g = rgba.map(|x| ((x >> 22) & 0x3ff).wrapping_sub(384) as f32 / 510.0);
            *b = rgba.map(|x| ((x >> 38) & 0x3ff).wrapping_sub(384) as f32 / 510.0);
            *a = rgba.map(|x| ((x >> 54) & 0x3ff).wrapping_sub(384) as f32 / 510.0);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:926-940
    stage_noctx!(
        Exec_load_161616LE,
        Exec_load_161616LE_k,
        |src, dst, r, g, b, a, i| {
            let ptr = src.offset(6 * i as isize);
            assert_eq!(ptr as usize & 1, 0);
            let v = vld3_u16(ptr.cast());
            let mut v0 = [0u16; 4];
            vst1_u16(v0.as_mut_ptr(), v.0);
            *r = v0.map(|x| x as f32 * (1.0 / 65535.0));
            let mut v1 = [0u16; 4];
            vst1_u16(v1.as_mut_ptr(), v.1);
            *g = v1.map(|x| x as f32 * (1.0 / 65535.0));
            let mut v2 = [0u16; 4];
            vst1_u16(v2.as_mut_ptr(), v.2);
            *b = v2.map(|x| x as f32 * (1.0 / 65535.0));
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:942-960
    stage_noctx!(
        Exec_load_16161616LE,
        Exec_load_16161616LE_k,
        |src, dst, r, g, b, a, i| {
            let ptr = src.offset(8 * i as isize);
            assert_eq!(ptr as usize & 1, 0);
            let v = vld4_u16(ptr.cast());
            let mut v0 = [0u16; 4];
            vst1_u16(v0.as_mut_ptr(), v.0);
            *r = v0.map(|x| x as f32 * (1.0 / 65535.0));
            let mut v1 = [0u16; 4];
            vst1_u16(v1.as_mut_ptr(), v.1);
            *g = v1.map(|x| x as f32 * (1.0 / 65535.0));
            let mut v2 = [0u16; 4];
            vst1_u16(v2.as_mut_ptr(), v.2);
            *b = v2.map(|x| x as f32 * (1.0 / 65535.0));
            let mut v3 = [0u16; 4];
            vst1_u16(v3.as_mut_ptr(), v.3);
            *a = v3.map(|x| x as f32 * (1.0 / 65535.0));
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:962-980
    stage_noctx!(
        Exec_load_161616BE,
        Exec_load_161616BE_k,
        |src, dst, r, g, b, a, i| {
            let ptr = src.offset(6 * i as isize);
            assert_eq!(ptr as usize & 1, 0);
            let v = vld3_u16(ptr.cast());
            let mut v0 = [0u16; 4];
            vst1_u16(v0.as_mut_ptr(), v.0);
            *r = swap_endian_16(v0).map(|x| x as f32 * (1.0 / 65535.0));
            let mut v1 = [0u16; 4];
            vst1_u16(v1.as_mut_ptr(), v.1);
            *g = swap_endian_16(v1).map(|x| x as f32 * (1.0 / 65535.0));
            let mut v2 = [0u16; 4];
            vst1_u16(v2.as_mut_ptr(), v.2);
            *b = swap_endian_16(v2).map(|x| x as f32 * (1.0 / 65535.0));
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:982-1000
    stage_noctx!(
        Exec_load_16161616BE,
        Exec_load_16161616BE_k,
        |src, dst, r, g, b, a, i| {
            let ptr = src.offset(8 * i as isize);
            assert_eq!(ptr as usize & 1, 0);
            let v = vld4_u16(ptr.cast());
            let mut v0 = [0u16; 4];
            vst1_u16(v0.as_mut_ptr(), v.0);
            *r = swap_endian_16(v0).map(|x| x as f32 * (1.0 / 65535.0));
            let mut v1 = [0u16; 4];
            vst1_u16(v1.as_mut_ptr(), v.1);
            *g = swap_endian_16(v1).map(|x| x as f32 * (1.0 / 65535.0));
            let mut v2 = [0u16; 4];
            vst1_u16(v2.as_mut_ptr(), v.2);
            *b = swap_endian_16(v2).map(|x| x as f32 * (1.0 / 65535.0));
            let mut v3 = [0u16; 4];
            vst1_u16(v3.as_mut_ptr(), v.3);
            *a = swap_endian_16(v3).map(|x| x as f32 * (1.0 / 65535.0));
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1002-1019
    stage_noctx!(Exec_load_hhh, Exec_load_hhh_k, |src, dst, r, g, b, a, i| {
        let ptr = src.offset(6 * i as isize);
        assert_eq!(ptr as usize & 1, 0);
        let v = vld3_u16(ptr.cast());
        let mut v0 = [0u16; 4];
        vst1_u16(v0.as_mut_ptr(), v.0);
        *r = F_from_Half(v0);
        let mut v1 = [0u16; 4];
        vst1_u16(v1.as_mut_ptr(), v.1);
        *g = F_from_Half(v1);
        let mut v2 = [0u16; 4];
        vst1_u16(v2.as_mut_ptr(), v.2);
        *b = F_from_Half(v2);
    });
    // Source: modules/skcms/src/Transform_inl.h:1021-1042
    stage_noctx!(
        Exec_load_hhhh,
        Exec_load_hhhh_k,
        |src, dst, r, g, b, a, i| {
            let ptr = src.offset(8 * i as isize);
            assert_eq!(ptr as usize & 1, 0);
            let v = vld4_u16(ptr.cast());
            let mut v0 = [0u16; 4];
            vst1_u16(v0.as_mut_ptr(), v.0);
            *r = F_from_Half(v0);
            let mut v1 = [0u16; 4];
            vst1_u16(v1.as_mut_ptr(), v.1);
            *g = F_from_Half(v1);
            let mut v2 = [0u16; 4];
            vst1_u16(v2.as_mut_ptr(), v.2);
            *b = F_from_Half(v2);
            let mut v3 = [0u16; 4];
            vst1_u16(v3.as_mut_ptr(), v.3);
            *a = F_from_Half(v3);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1044-1058
    stage_noctx!(Exec_load_fff, Exec_load_fff_k, |src, dst, r, g, b, a, i| {
        let ptr = src.offset(12 * i as isize);
        assert_eq!(ptr as usize & 3, 0);
        let v = vld3q_f32(ptr.cast());
        vst1q_f32(r.as_mut_ptr(), v.0);
        vst1q_f32(g.as_mut_ptr(), v.1);
        vst1q_f32(b.as_mut_ptr(), v.2);
    });
    // Source: modules/skcms/src/Transform_inl.h:1060-1076
    stage_noctx!(
        Exec_load_ffff,
        Exec_load_ffff_k,
        |src, dst, r, g, b, a, i| {
            let ptr = src.offset(16 * i as isize);
            assert_eq!(ptr as usize & 3, 0);
            let v = vld4q_f32(ptr.cast());
            vst1q_f32(r.as_mut_ptr(), v.0);
            vst1q_f32(g.as_mut_ptr(), v.1);
            vst1q_f32(b.as_mut_ptr(), v.2);
            vst1q_f32(a.as_mut_ptr(), v.3);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1078-1082
    stage_noctx!(Exec_swap_rb, Exec_swap_rb_k, |src, dst, r, g, b, a, i| {
        let t = *r;
        *r = *b;
        *b = t;
    });
    // Source: modules/skcms/src/Transform_inl.h:1084-1089
    stage_noctx!(Exec_clamp, Exec_clamp_k, |src, dst, r, g, b, a, i| {
        *r = max_(F0, min_(*r, F1));
        *g = max_(F0, min_(*g, F1));
        *b = max_(F0, min_(*b, F1));
        *a = max_(F0, min_(*a, F1));
    });
    // Source: modules/skcms/src/Transform_inl.h:1091-1096
    stage_noctx!(Exec_invert, Exec_invert_k, |src, dst, r, g, b, a, i| {
        *r = r.map(|x| 1.0 - x);
        *g = g.map(|x| 1.0 - x);
        *b = b.map(|x| 1.0 - x);
        *a = a.map(|x| 1.0 - x);
    });
    // Source: modules/skcms/src/Transform_inl.h:1098-1100
    stage_noctx!(
        Exec_force_opaque,
        Exec_force_opaque_k,
        |src, dst, r, g, b, a, i| {
            *a = F1;
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1102-1106
    stage_noctx!(Exec_premul, Exec_premul_k, |src, dst, r, g, b, a, i| {
        *r = std::array::from_fn(|j| r[j] * a[j]);
        *g = std::array::from_fn(|j| g[j] * a[j]);
        *b = std::array::from_fn(|j| b[j] * a[j]);
    });
    // Source: modules/skcms/src/Transform_inl.h:1108-1113
    stage_noctx!(Exec_unpremul, Exec_unpremul_k, |src, dst, r, g, b, a, i| {
        let scale = std::array::from_fn::<_, 4, _>(|j| {
            if 1.0 / a[j] < f32::INFINITY {
                1.0 / a[j]
            } else {
                0.0
            }
        });
        *r = std::array::from_fn(|j| r[j] * scale[j]);
        *g = std::array::from_fn(|j| g[j] * scale[j]);
        *b = std::array::from_fn(|j| b[j] * scale[j]);
    });
    // Source: modules/skcms/src/Transform_inl.h:1115-1123
    stage_ctx!(
        Exec_matrix_3x3,
        Exec_matrix_3x3_k,
        crate::modules::skcms::src::skcms_public::skcms_Matrix3x3,
        |matrix, src, dst, r, g, b, a, i| {
            let R = std::array::from_fn::<_, 4, _>(|j| {
                matrix.vals[0][0] * r[j] + matrix.vals[0][1] * g[j] + matrix.vals[0][2] * b[j]
            });
            let G = std::array::from_fn::<_, 4, _>(|j| {
                matrix.vals[1][0] * r[j] + matrix.vals[1][1] * g[j] + matrix.vals[1][2] * b[j]
            });
            let B = std::array::from_fn::<_, 4, _>(|j| {
                matrix.vals[2][0] * r[j] + matrix.vals[2][1] * g[j] + matrix.vals[2][2] * b[j]
            });
            *r = R;
            *g = G;
            *b = B;
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1125-1136
    stage_ctx!(
        Exec_matrix_3x4,
        Exec_matrix_3x4_k,
        crate::modules::skcms::src::skcms_public::skcms_Matrix3x4,
        |matrix, src, dst, r, g, b, a, i| {
            let R = std::array::from_fn::<_, 4, _>(|j| {
                matrix.vals[0][0] * r[j]
                    + matrix.vals[0][1] * g[j]
                    + matrix.vals[0][2] * b[j]
                    + matrix.vals[0][3]
            });
            let G = std::array::from_fn::<_, 4, _>(|j| {
                matrix.vals[1][0] * r[j]
                    + matrix.vals[1][1] * g[j]
                    + matrix.vals[1][2] * b[j]
                    + matrix.vals[1][3]
            });
            let B = std::array::from_fn::<_, 4, _>(|j| {
                matrix.vals[2][0] * r[j]
                    + matrix.vals[2][1] * g[j]
                    + matrix.vals[2][2] * b[j]
                    + matrix.vals[2][3]
            });
            *r = R;
            *g = G;
            *b = B;
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1138-1157
    stage_noctx!(
        Exec_lab_to_xyz,
        Exec_lab_to_xyz_k,
        |src, dst, r, g, b, a, i| {
            let l = r.map(|v| v * 100.0);
            let aa = g.map(|v| v * 255.0 - 128.0);
            let bb = b.map(|v| v * 255.0 - 128.0);
            let mut y = l.map(|v| (v + 16.0) * (1.0 / 116.0));
            let mut x = std::array::from_fn::<_, 4, _>(|j| y[j] + aa[j] * (1.0 / 500.0));
            let mut z = std::array::from_fn::<_, 4, _>(|j| y[j] - bb[j] * (1.0 / 200.0));
            x = x.map(|v| {
                if v * v * v > 0.008856 {
                    v * v * v
                } else {
                    (v - (16.0 / 116.0)) * (1.0 / 7.787)
                }
            });
            y = y.map(|v| {
                if v * v * v > 0.008856 {
                    v * v * v
                } else {
                    (v - (16.0 / 116.0)) * (1.0 / 7.787)
                }
            });
            z = z.map(|v| {
                if v * v * v > 0.008856 {
                    v * v * v
                } else {
                    (v - (16.0 / 116.0)) * (1.0 / 7.787)
                }
            });
            *r = x.map(|v| v * 0.9642);
            *g = y;
            *b = z.map(|v| v * 0.8249);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1160-1176
    stage_noctx!(
        Exec_xyz_to_lab,
        Exec_xyz_to_lab_k,
        |src, dst, r, g, b, a, i| {
            let mut x = r.map(|v| v * (1.0 / 0.9642));
            let mut y = *g;
            let mut z = b.map(|v| v * (1.0 / 0.8249));
            let xp = approx_pow(x, 1.0 / 3.0);
            let yp = approx_pow(y, 1.0 / 3.0);
            let zp = approx_pow(z, 1.0 / 3.0);
            x = std::array::from_fn(|j| {
                if x[j] > 0.008856 {
                    xp[j]
                } else {
                    x[j] * 7.787 + (16.0 / 116.0)
                }
            });
            y = std::array::from_fn(|j| {
                if y[j] > 0.008856 {
                    yp[j]
                } else {
                    y[j] * 7.787 + (16.0 / 116.0)
                }
            });
            z = std::array::from_fn(|j| {
                if z[j] > 0.008856 {
                    zp[j]
                } else {
                    z[j] * 7.787 + (16.0 / 116.0)
                }
            });
            let l = y.map(|v| v * 116.0 - 16.0);
            let aa = std::array::from_fn::<_, 4, _>(|j| (x[j] - y[j]) * 500.0);
            let bb = std::array::from_fn::<_, 4, _>(|j| (y[j] - z[j]) * 200.0);
            *r = l.map(|v| v * (1.0 / 100.0));
            *g = aa.map(|v| (v + 128.0) * (1.0 / 255.0));
            *b = bb.map(|v| (v + 128.0) * (1.0 / 255.0));
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1178-1178
    stage_ctx!(
        Exec_gamma_r,
        Exec_gamma_r_k,
        crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
        |tf, src, dst, r, g, b, a, i| {
            *r = apply_gamma(tf, *r);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1179-1179
    stage_ctx!(
        Exec_gamma_g,
        Exec_gamma_g_k,
        crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
        |tf, src, dst, r, g, b, a, i| {
            *g = apply_gamma(tf, *g);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1180-1180
    stage_ctx!(
        Exec_gamma_b,
        Exec_gamma_b_k,
        crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
        |tf, src, dst, r, g, b, a, i| {
            *b = apply_gamma(tf, *b);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1181-1181
    stage_ctx!(
        Exec_gamma_a,
        Exec_gamma_a_k,
        crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
        |tf, src, dst, r, g, b, a, i| {
            *a = apply_gamma(tf, *a);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1183-1187
    stage_ctx!(
        Exec_gamma_rgb,
        Exec_gamma_rgb_k,
        crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
        |tf, src, dst, r, g, b, a, i| {
            *r = apply_gamma(tf, *r);
            *g = apply_gamma(tf, *g);
            *b = apply_gamma(tf, *b);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1189-1189
    stage_ctx!(
        Exec_tf_r,
        Exec_tf_r_k,
        crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
        |tf, src, dst, r, g, b, a, i| {
            *r = apply_tf(tf, *r);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1190-1190
    stage_ctx!(
        Exec_tf_g,
        Exec_tf_g_k,
        crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
        |tf, src, dst, r, g, b, a, i| {
            *g = apply_tf(tf, *g);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1191-1191
    stage_ctx!(
        Exec_tf_b,
        Exec_tf_b_k,
        crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
        |tf, src, dst, r, g, b, a, i| {
            *b = apply_tf(tf, *b);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1192-1192
    stage_ctx!(
        Exec_tf_a,
        Exec_tf_a_k,
        crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
        |tf, src, dst, r, g, b, a, i| {
            *a = apply_tf(tf, *a);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1194-1198
    stage_ctx!(
        Exec_tf_rgb,
        Exec_tf_rgb_k,
        crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
        |tf, src, dst, r, g, b, a, i| {
            *r = apply_tf(tf, *r);
            *g = apply_tf(tf, *g);
            *b = apply_tf(tf, *b);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1200-1200
    stage_ctx!(
        Exec_pq_r,
        Exec_pq_r_k,
        crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
        |tf, src, dst, r, g, b, a, i| {
            *r = apply_pq(tf, *r);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1201-1201
    stage_ctx!(
        Exec_pq_g,
        Exec_pq_g_k,
        crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
        |tf, src, dst, r, g, b, a, i| {
            *g = apply_pq(tf, *g);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1202-1202
    stage_ctx!(
        Exec_pq_b,
        Exec_pq_b_k,
        crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
        |tf, src, dst, r, g, b, a, i| {
            *b = apply_pq(tf, *b);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1203-1203
    stage_ctx!(
        Exec_pq_a,
        Exec_pq_a_k,
        crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
        |tf, src, dst, r, g, b, a, i| {
            *a = apply_pq(tf, *a);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1205-1209
    stage_ctx!(
        Exec_pq_rgb,
        Exec_pq_rgb_k,
        crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
        |tf, src, dst, r, g, b, a, i| {
            *r = apply_pq(tf, *r);
            *g = apply_pq(tf, *g);
            *b = apply_pq(tf, *b);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1211-1211
    stage_ctx!(
        Exec_hlg_r,
        Exec_hlg_r_k,
        crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
        |tf, src, dst, r, g, b, a, i| {
            *r = apply_hlg(tf, *r);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1212-1212
    stage_ctx!(
        Exec_hlg_g,
        Exec_hlg_g_k,
        crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
        |tf, src, dst, r, g, b, a, i| {
            *g = apply_hlg(tf, *g);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1213-1213
    stage_ctx!(
        Exec_hlg_b,
        Exec_hlg_b_k,
        crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
        |tf, src, dst, r, g, b, a, i| {
            *b = apply_hlg(tf, *b);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1214-1214
    stage_ctx!(
        Exec_hlg_a,
        Exec_hlg_a_k,
        crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
        |tf, src, dst, r, g, b, a, i| {
            *a = apply_hlg(tf, *a);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1216-1220
    stage_ctx!(
        Exec_hlg_rgb,
        Exec_hlg_rgb_k,
        crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
        |tf, src, dst, r, g, b, a, i| {
            *r = apply_hlg(tf, *r);
            *g = apply_hlg(tf, *g);
            *b = apply_hlg(tf, *b);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1223-1241
    stage_noctx!(
        Exec_hlg_ootf_scale,
        Exec_hlg_ootf_scale_k,
        |src, dst, r, g, b, a, i| {
            let mut y = compute_Y_in_xyzd50(*r, *g, *b);
            let gamma_minus_1 = 0.2;
            let mut sign = [0; 4];
            y = strip_sign(y, &mut sign);
            let factor = apply_sign(approx_pow(y, gamma_minus_1), sign);
            *r = std::array::from_fn(|j| r[j] * factor[j]);
            *g = std::array::from_fn(|j| g[j] * factor[j]);
            *b = std::array::from_fn(|j| b[j] * factor[j]);
            *r = r.map(|v| v * (1000.0 / 203.0));
            *g = g.map(|v| v * (1000.0 / 203.0));
            *b = b.map(|v| v * (1000.0 / 203.0));
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1243-1243
    stage_ctx!(
        Exec_hlginv_r,
        Exec_hlginv_r_k,
        crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
        |tf, src, dst, r, g, b, a, i| {
            *r = apply_hlginv(tf, *r);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1244-1244
    stage_ctx!(
        Exec_hlginv_g,
        Exec_hlginv_g_k,
        crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
        |tf, src, dst, r, g, b, a, i| {
            *g = apply_hlginv(tf, *g);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1245-1245
    stage_ctx!(
        Exec_hlginv_b,
        Exec_hlginv_b_k,
        crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
        |tf, src, dst, r, g, b, a, i| {
            *b = apply_hlginv(tf, *b);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1246-1246
    stage_ctx!(
        Exec_hlginv_a,
        Exec_hlginv_a_k,
        crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
        |tf, src, dst, r, g, b, a, i| {
            *a = apply_hlginv(tf, *a);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1248-1252
    stage_ctx!(
        Exec_hlginv_rgb,
        Exec_hlginv_rgb_k,
        crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
        |tf, src, dst, r, g, b, a, i| {
            *r = apply_hlginv(tf, *r);
            *g = apply_hlginv(tf, *g);
            *b = apply_hlginv(tf, *b);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1255-1269
    stage_noctx!(
        Exec_hlginv_ootf_scale,
        Exec_hlginv_ootf_scale_k,
        |src, dst, r, g, b, a, i| {
            *r = r.map(|v| v * (203.0 / 1000.0));
            *g = g.map(|v| v * (203.0 / 1000.0));
            *b = b.map(|v| v * (203.0 / 1000.0));
            let gamma_inv_minus_1 = 1.0 / 1.2 - 1.0;
            let mut y = compute_Y_in_xyzd50(*r, *g, *b);
            let mut sign = [0; 4];
            y = strip_sign(y, &mut sign);
            let factor = apply_sign(approx_pow(y, gamma_inv_minus_1), sign);
            *r = std::array::from_fn(|j| r[j] * factor[j]);
            *g = std::array::from_fn(|j| g[j] * factor[j]);
            *b = std::array::from_fn(|j| b[j] * factor[j]);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1271-1271
    stage_ctx!(
        Exec_table_r,
        Exec_table_r_k,
        crate::modules::skcms::src::skcms_public::skcms_Curve,
        |curve, src, dst, r, g, b, a, i| {
            *r = table(curve, *r);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1272-1272
    stage_ctx!(
        Exec_table_g,
        Exec_table_g_k,
        crate::modules::skcms::src::skcms_public::skcms_Curve,
        |curve, src, dst, r, g, b, a, i| {
            *g = table(curve, *g);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1273-1273
    stage_ctx!(
        Exec_table_b,
        Exec_table_b_k,
        crate::modules::skcms::src::skcms_public::skcms_Curve,
        |curve, src, dst, r, g, b, a, i| {
            *b = table(curve, *b);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1274-1274
    stage_ctx!(
        Exec_table_a,
        Exec_table_a_k,
        crate::modules::skcms::src::skcms_public::skcms_Curve,
        |curve, src, dst, r, g, b, a, i| {
            *a = table(curve, *a);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1276-1283
    stage_ctx!(
        Exec_clut_A2B,
        Exec_clut_A2B_k,
        crate::modules::skcms::src::skcms_public::skcms_A2B,
        |a2b, src, dst, r, g, b, a, i| {
            clut_A2B(a2b, r, g, b, *a);
            if a2b.input_channels == 4 {
                *a = F1;
            }
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1285-1287
    stage_ctx!(
        Exec_clut_B2A,
        Exec_clut_B2A_k,
        crate::modules::skcms::src::skcms_public::skcms_B2A,
        |b2a, src, dst, r, g, b, a, i| {
            clut_B2A(b2a, r, g, b, a);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1291-1293
    // FINAL_STAGE uses the same actual wrapper/kernel protocol in non-musttail configuration.
    stage_noctx!(Exec_store_a8, Exec_store_a8_k, |src, dst, r, g, b, a, i| {
        let v: U8 = cast(&to_fixed(a.map(|x| x * 255.0)));
        store(dst.offset(i as isize), &v);
    });
    // Source: modules/skcms/src/Transform_inl.h:1295-1298
    // FINAL_STAGE uses the same actual wrapper/kernel protocol in non-musttail configuration.
    stage_noctx!(Exec_store_g8, Exec_store_g8_k, |src, dst, r, g, b, a, i| {
        let v: U8 = cast(&to_fixed(g.map(|x| x * 255.0)));
        store(dst.offset(i as isize), &v);
    });
    // Source: modules/skcms/src/Transform_inl.h:1300-1304
    // FINAL_STAGE uses the same actual wrapper/kernel protocol in non-musttail configuration.
    stage_noctx!(
        Exec_store_ga88,
        Exec_store_ga88_k,
        |src, dst, r, g, b, a, i| {
            let g: U16 = cast(&to_fixed(g.map(|x| x * 255.0)));
            let a: U16 = cast(&to_fixed(a.map(|x| x * 255.0)).map(|x| x << 8));
            let v: U16 = std::array::from_fn(|j| g[j] | a[j]);
            store(dst.offset(2 * i as isize), &v);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1306-1311
    // FINAL_STAGE uses the same actual wrapper/kernel protocol in non-musttail configuration.
    stage_noctx!(
        Exec_store_4444,
        Exec_store_4444_k,
        |src, dst, r, g, b, a, i| {
            let r: U16 = cast(&to_fixed(r.map(|x| x * 15.0)).map(|x| x.wrapping_shl(12)));
            let g: U16 = cast(&to_fixed(g.map(|x| x * 15.0)).map(|x| x.wrapping_shl(8)));
            let b: U16 = cast(&to_fixed(b.map(|x| x * 15.0)).map(|x| x.wrapping_shl(4)));
            let a: U16 = cast(&to_fixed(a.map(|x| x * 15.0)).map(|x| x.wrapping_shl(0)));
            let v: U16 = std::array::from_fn(|j| r[j] | g[j] | b[j] | a[j]);
            store(dst.offset(2 * i as isize), &v);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1313-1317
    // FINAL_STAGE uses the same actual wrapper/kernel protocol in non-musttail configuration.
    stage_noctx!(
        Exec_store_565,
        Exec_store_565_k,
        |src, dst, r, g, b, a, i| {
            let r: U16 = cast(&to_fixed(r.map(|x| x * 31.0)).map(|x| x.wrapping_shl(0)));
            let g: U16 = cast(&to_fixed(g.map(|x| x * 63.0)).map(|x| x.wrapping_shl(5)));
            let b: U16 = cast(&to_fixed(b.map(|x| x * 31.0)).map(|x| x.wrapping_shl(11)));
            let v: U16 = std::array::from_fn(|j| r[j] | g[j] | b[j]);
            store(dst.offset(2 * i as isize), &v);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1319-1339
    // FINAL_STAGE uses the same actual wrapper/kernel protocol in non-musttail configuration.
    stage_noctx!(
        Exec_store_888,
        Exec_store_888_k,
        |src, dst, r, g, b, a, i| {
            let rgb = dst.offset(3 * i as isize);
            let rr: U16 = cast(&to_fixed(r.map(|x| x * 255.0)));
            let gg: U16 = cast(&to_fixed(g.map(|x| x * 255.0)));
            let bb: U16 = cast(&to_fixed(b.map(|x| x * 255.0)));
            let v = uint8x8x3_t(
                vreinterpret_u8_u16(vld1_u16(rr.as_ptr())),
                vreinterpret_u8_u16(vld1_u16(gg.as_ptr())),
                vreinterpret_u8_u16(vld1_u16(bb.as_ptr())),
            );
            vst3_lane_u8::<0>(rgb, v);
            vst3_lane_u8::<2>(rgb.add(3), v);
            vst3_lane_u8::<4>(rgb.add(6), v);
            vst3_lane_u8::<6>(rgb.add(9), v);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1341-1346
    // FINAL_STAGE uses the same actual wrapper/kernel protocol in non-musttail configuration.
    stage_noctx!(
        Exec_store_8888,
        Exec_store_8888_k,
        |src, dst, r, g, b, a, i| {
            let r: U32 = cast::<u32, _>(&to_fixed(r.map(|x| x * 255.0))).map(|x| x.wrapping_shl(0));
            let g: U32 = cast::<u32, _>(&to_fixed(g.map(|x| x * 255.0))).map(|x| x.wrapping_shl(8));
            let b: U32 =
                cast::<u32, _>(&to_fixed(b.map(|x| x * 255.0))).map(|x| x.wrapping_shl(16));
            let a: U32 =
                cast::<u32, _>(&to_fixed(a.map(|x| x * 255.0))).map(|x| x.wrapping_shl(24));
            let v: U32 = std::array::from_fn(|j| r[j] | g[j] | b[j] | a[j]);
            store(dst.offset(4 * i as isize), &v);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1348-1352
    // FINAL_STAGE uses the same actual wrapper/kernel protocol in non-musttail configuration.
    stage_noctx!(
        Exec_store_101010x_XR,
        Exec_store_101010x_XR_k,
        |src, dst, r, g, b, a, i| {
            let r: U32 =
                cast::<u32, _>(&to_fixed(r.map(|x| x * 510.0 + 384.0))).map(|x| x.wrapping_shl(0));
            let g: U32 =
                cast::<u32, _>(&to_fixed(g.map(|x| x * 510.0 + 384.0))).map(|x| x.wrapping_shl(10));
            let b: U32 =
                cast::<u32, _>(&to_fixed(b.map(|x| x * 510.0 + 384.0))).map(|x| x.wrapping_shl(20));
            let v: U32 = std::array::from_fn(|j| r[j] | g[j] | b[j]);
            store(dst.offset(4 * i as isize), &v);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1354-1360
    // FINAL_STAGE uses the same actual wrapper/kernel protocol in non-musttail configuration.
    stage_noctx!(
        Exec_store_10101010_XR,
        Exec_store_10101010_XR_k,
        |src, dst, r, g, b, a, i| {
            let r: U64 =
                cast::<u64, _>(&to_fixed(r.map(|x| x * 510.0 + 384.0))).map(|x| x.wrapping_shl(6));
            let g: U64 =
                cast::<u64, _>(&to_fixed(g.map(|x| x * 510.0 + 384.0))).map(|x| x.wrapping_shl(22));
            let b: U64 =
                cast::<u64, _>(&to_fixed(b.map(|x| x * 510.0 + 384.0))).map(|x| x.wrapping_shl(38));
            let a: U64 =
                cast::<u64, _>(&to_fixed(a.map(|x| x * 510.0 + 384.0))).map(|x| x.wrapping_shl(54));
            let v: U64 = std::array::from_fn(|j| r[j] | g[j] | b[j] | a[j]);
            store(dst.offset(8 * i as isize), &v);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1362-1367
    // FINAL_STAGE uses the same actual wrapper/kernel protocol in non-musttail configuration.
    stage_noctx!(
        Exec_store_1010102,
        Exec_store_1010102_k,
        |src, dst, r, g, b, a, i| {
            let r: U32 =
                cast::<u32, _>(&to_fixed(r.map(|x| x * 1023.0))).map(|x| x.wrapping_shl(0));
            let g: U32 =
                cast::<u32, _>(&to_fixed(g.map(|x| x * 1023.0))).map(|x| x.wrapping_shl(10));
            let b: U32 =
                cast::<u32, _>(&to_fixed(b.map(|x| x * 1023.0))).map(|x| x.wrapping_shl(20));
            let a: U32 = cast::<u32, _>(&to_fixed(a.map(|x| x * 3.0))).map(|x| x.wrapping_shl(30));
            let v: U32 = std::array::from_fn(|j| r[j] | g[j] | b[j] | a[j]);
            store(dst.offset(4 * i as isize), &v);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1369-1386
    // FINAL_STAGE uses the same actual wrapper/kernel protocol in non-musttail configuration.
    stage_noctx!(
        Exec_store_161616LE,
        Exec_store_161616LE_k,
        |src, dst, r, g, b, a, i| {
            let ptr = dst.offset(6 * i as isize);
            assert_eq!(ptr as usize & 1, 0);
            let r = U16_from_F(*r);
            let g = U16_from_F(*g);
            let b = U16_from_F(*b);
            let v = uint16x4x3_t(
                vld1_u16(r.as_ptr()),
                vld1_u16(g.as_ptr()),
                vld1_u16(b.as_ptr()),
            );
            vst3_u16(ptr.cast(), v);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1388-1407
    // FINAL_STAGE uses the same actual wrapper/kernel protocol in non-musttail configuration.
    stage_noctx!(
        Exec_store_16161616LE,
        Exec_store_16161616LE_k,
        |src, dst, r, g, b, a, i| {
            let ptr = dst.offset(8 * i as isize);
            assert_eq!(ptr as usize & 1, 0);
            let r = U16_from_F(*r);
            let g = U16_from_F(*g);
            let b = U16_from_F(*b);
            let a = U16_from_F(*a);
            let v = uint16x4x4_t(
                vld1_u16(r.as_ptr()),
                vld1_u16(g.as_ptr()),
                vld1_u16(b.as_ptr()),
                vld1_u16(a.as_ptr()),
            );
            vst4_u16(ptr.cast(), v);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1409-1429
    // FINAL_STAGE uses the same actual wrapper/kernel protocol in non-musttail configuration.
    stage_noctx!(
        Exec_store_161616BE,
        Exec_store_161616BE_k,
        |src, dst, r, g, b, a, i| {
            let ptr = dst.offset(6 * i as isize);
            assert_eq!(ptr as usize & 1, 0);
            let r = U16_from_F(*r);
            let r = swap_endian_16(r);
            let g = U16_from_F(*g);
            let g = swap_endian_16(g);
            let b = U16_from_F(*b);
            let b = swap_endian_16(b);
            let v = uint16x4x3_t(
                vld1_u16(r.as_ptr()),
                vld1_u16(g.as_ptr()),
                vld1_u16(b.as_ptr()),
            );
            vst3_u16(ptr.cast(), v);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1431-1450
    // FINAL_STAGE uses the same actual wrapper/kernel protocol in non-musttail configuration.
    stage_noctx!(
        Exec_store_16161616BE,
        Exec_store_16161616BE_k,
        |src, dst, r, g, b, a, i| {
            let ptr = dst.offset(8 * i as isize);
            assert_eq!(ptr as usize & 1, 0);
            let r = U16_from_F(*r);
            let r = swap_endian_16(r);
            let g = U16_from_F(*g);
            let g = swap_endian_16(g);
            let b = U16_from_F(*b);
            let b = swap_endian_16(b);
            let a = U16_from_F(*a);
            let a = swap_endian_16(a);
            let v = uint16x4x4_t(
                vld1_u16(r.as_ptr()),
                vld1_u16(g.as_ptr()),
                vld1_u16(b.as_ptr()),
                vld1_u16(a.as_ptr()),
            );
            vst4_u16(ptr.cast(), v);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1452-1472
    // FINAL_STAGE uses the same actual wrapper/kernel protocol in non-musttail configuration.
    stage_noctx!(
        Exec_store_hhh,
        Exec_store_hhh_k,
        |src, dst, r, g, b, a, i| {
            let ptr = dst.offset(6 * i as isize);
            assert_eq!(ptr as usize & 1, 0);
            let r = Half_from_F(*r);
            let g = Half_from_F(*g);
            let b = Half_from_F(*b);
            let v = uint16x4x3_t(
                vld1_u16(r.as_ptr()),
                vld1_u16(g.as_ptr()),
                vld1_u16(b.as_ptr()),
            );
            vst3_u16(ptr.cast(), v);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1474-1497
    // FINAL_STAGE uses the same actual wrapper/kernel protocol in non-musttail configuration.
    stage_noctx!(
        Exec_store_hhhh,
        Exec_store_hhhh_k,
        |src, dst, r, g, b, a, i| {
            let ptr = dst.offset(8 * i as isize);
            assert_eq!(ptr as usize & 1, 0);
            let r = Half_from_F(*r);
            let g = Half_from_F(*g);
            let b = Half_from_F(*b);
            let a = Half_from_F(*a);
            let v = uint16x4x4_t(
                vld1_u16(r.as_ptr()),
                vld1_u16(g.as_ptr()),
                vld1_u16(b.as_ptr()),
                vld1_u16(a.as_ptr()),
            );
            vst4_u16(ptr.cast(), v);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1499-1515
    // FINAL_STAGE uses the same actual wrapper/kernel protocol in non-musttail configuration.
    stage_noctx!(
        Exec_store_fff,
        Exec_store_fff_k,
        |src, dst, r, g, b, a, i| {
            let ptr = dst.offset(12 * i as isize);
            assert_eq!(ptr as usize & 3, 0);
            let v = float32x4x3_t(
                vld1q_f32(r.as_ptr()),
                vld1q_f32(g.as_ptr()),
                vld1q_f32(b.as_ptr()),
            );
            vst3q_f32(ptr.cast(), v);
        }
    );
    // Source: modules/skcms/src/Transform_inl.h:1517-1535
    // FINAL_STAGE uses the same actual wrapper/kernel protocol in non-musttail configuration.
    stage_noctx!(
        Exec_store_ffff,
        Exec_store_ffff_k,
        |src, dst, r, g, b, a, i| {
            let ptr = dst.offset(16 * i as isize);
            assert_eq!(ptr as usize & 3, 0);
            let v = float32x4x4_t(
                vld1q_f32(r.as_ptr()),
                vld1q_f32(g.as_ptr()),
                vld1q_f32(b.as_ptr()),
                vld1q_f32(a.as_ptr()),
            );
            vst4q_f32(ptr.cast(), v);
        }
    );

    use crate::modules::skcms::src::skcms_Transform::Op;
    // Source: modules/skcms/src/Transform_inl.h:1545-1558
    /// Caller provides a valid op/context stream ending in a final store stage, plus valid pixel leases.
    /// Each typed context must remain readable for the complete execution; registers are independent.
    unsafe fn exec_stages(
        mut ops: *const Op,
        mut contexts: *const *const std::ffi::c_void,
        src: *const u8,
        dst: *mut u8,
        i: i32,
    ) {
        let (mut r, mut g, mut b, mut a) = (F0, F0, F0, F1);
        loop {
            let op = unsafe { ops.read() };
            ops = unsafe { ops.add(1) };
            let ctx = Ctx {
                fArg: unsafe { contexts.read() },
            };
            contexts = unsafe { contexts.add(1) };
            unsafe {
                match op {
                    Op::load_a8 => {
                        Exec_load_a8(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::load_g8 => {
                        Exec_load_g8(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::load_ga88 => {
                        Exec_load_ga88(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::load_4444 => {
                        Exec_load_4444(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::load_565 => {
                        Exec_load_565(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::load_888 => {
                        Exec_load_888(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::load_8888 => {
                        Exec_load_8888(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::load_1010102 => {
                        Exec_load_1010102(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::load_101010x_XR => {
                        Exec_load_101010x_XR(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::load_10101010_XR => {
                        Exec_load_10101010_XR(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::load_161616LE => {
                        Exec_load_161616LE(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::load_16161616LE => {
                        Exec_load_16161616LE(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::load_161616BE => {
                        Exec_load_161616BE(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::load_16161616BE => {
                        Exec_load_16161616BE(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::load_hhh => {
                        Exec_load_hhh(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::load_hhhh => {
                        Exec_load_hhhh(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::load_fff => {
                        Exec_load_fff(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::load_ffff => {
                        Exec_load_ffff(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::swap_rb => {
                        Exec_swap_rb(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::clamp => {
                        Exec_clamp(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::invert => {
                        Exec_invert(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::force_opaque => {
                        Exec_force_opaque(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::premul => {
                        Exec_premul(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::unpremul => {
                        Exec_unpremul(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::matrix_3x3 => {
                        Exec_matrix_3x3(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::matrix_3x4 => {
                        Exec_matrix_3x4(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::lab_to_xyz => {
                        Exec_lab_to_xyz(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::xyz_to_lab => {
                        Exec_xyz_to_lab(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::gamma_r => {
                        Exec_gamma_r(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::gamma_g => {
                        Exec_gamma_g(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::gamma_b => {
                        Exec_gamma_b(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::gamma_a => {
                        Exec_gamma_a(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::gamma_rgb => {
                        Exec_gamma_rgb(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::tf_r => {
                        Exec_tf_r(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::tf_g => {
                        Exec_tf_g(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::tf_b => {
                        Exec_tf_b(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::tf_a => {
                        Exec_tf_a(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::tf_rgb => {
                        Exec_tf_rgb(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::pq_r => {
                        Exec_pq_r(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::pq_g => {
                        Exec_pq_g(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::pq_b => {
                        Exec_pq_b(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::pq_a => {
                        Exec_pq_a(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::pq_rgb => {
                        Exec_pq_rgb(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::hlg_r => {
                        Exec_hlg_r(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::hlg_g => {
                        Exec_hlg_g(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::hlg_b => {
                        Exec_hlg_b(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::hlg_a => {
                        Exec_hlg_a(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::hlg_rgb => {
                        Exec_hlg_rgb(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::hlg_ootf_scale => {
                        Exec_hlg_ootf_scale(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::hlginv_r => {
                        Exec_hlginv_r(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::hlginv_g => {
                        Exec_hlginv_g(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::hlginv_b => {
                        Exec_hlginv_b(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::hlginv_a => {
                        Exec_hlginv_a(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::hlginv_rgb => {
                        Exec_hlginv_rgb(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::hlginv_ootf_scale => {
                        Exec_hlginv_ootf_scale(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::table_r => {
                        Exec_table_r(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::table_g => {
                        Exec_table_g(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::table_b => {
                        Exec_table_b(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::table_a => {
                        Exec_table_a(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::clut_A2B => {
                        Exec_clut_A2B(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::clut_B2A => {
                        Exec_clut_B2A(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                    }
                    Op::store_a8 => {
                        Exec_store_a8(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        return;
                    }
                    Op::store_g8 => {
                        Exec_store_g8(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        return;
                    }
                    Op::store_ga88 => {
                        Exec_store_ga88(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        return;
                    }
                    Op::store_4444 => {
                        Exec_store_4444(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        return;
                    }
                    Op::store_565 => {
                        Exec_store_565(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        return;
                    }
                    Op::store_888 => {
                        Exec_store_888(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        return;
                    }
                    Op::store_8888 => {
                        Exec_store_8888(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        return;
                    }
                    Op::store_1010102 => {
                        Exec_store_1010102(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        return;
                    }
                    Op::store_161616LE => {
                        Exec_store_161616LE(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        return;
                    }
                    Op::store_16161616LE => {
                        Exec_store_16161616LE(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        return;
                    }
                    Op::store_161616BE => {
                        Exec_store_161616BE(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        return;
                    }
                    Op::store_16161616BE => {
                        Exec_store_16161616BE(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        return;
                    }
                    Op::store_101010x_XR => {
                        Exec_store_101010x_XR(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        return;
                    }
                    Op::store_10101010_XR => {
                        Exec_store_10101010_XR(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        return;
                    }
                    Op::store_hhh => {
                        Exec_store_hhh(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        return;
                    }
                    Op::store_hhhh => {
                        Exec_store_hhhh(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        return;
                    }
                    Op::store_fff => {
                        Exec_store_fff(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        return;
                    }
                    Op::store_ffff => {
                        Exec_store_ffff(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        return;
                    }
                }
            }
        }
    }
    // Source: modules/skcms/src/Transform_inl.h:1563-1565
    // Source: modules/skcms/src/Transform_inl.h:1582-1599
    /// Caller supplies a well-formed program terminating in a store and one valid context per op.
    /// Pixel input/output cover n pixels at their declared bpp, including native alignment contracts.
    /// Each bpp is at most16; program stages must agree with these real formats.
    /// Machine-SIMD instances additionally require the caller to establish their
    /// exact native CPU feature set before entry; the N=1 instance uses no SIMD.
    pub unsafe fn run_program(
        program: *const Op,
        contexts: *const *const std::ffi::c_void,
        _programSize: isize,
        src: *const u8,
        dst: *mut u8,
        mut n: i32,
        src_bpp: usize,
        dst_bpp: usize,
    ) {
        let stages = program;
        let mut i = 0;
        // Native baseline N is the width of the selected four-lane register representation.
        let nlanes = F0.len() as i32;
        while n >= nlanes {
            unsafe { exec_stages(stages, contexts, src, dst, i) };
            i += nlanes;
            n -= nlanes;
        }
        if n > 0 {
            // Rust explicitly retains the alignment provided by the native ARM64 stack temporary.
            #[repr(align(16))]
            struct Tail([u8; 4 * 4 * 4]);
            let mut tmp = Tail([0; 4 * 4 * 4]);
            unsafe {
                std::ptr::copy_nonoverlapping(
                    src.add(i as usize * src_bpp),
                    tmp.0.as_mut_ptr(),
                    n as usize * src_bpp,
                );
                exec_stages(stages, contexts, tmp.0.as_ptr(), tmp.0.as_mut_ptr(), 0);
                std::ptr::copy_nonoverlapping(
                    tmp.0.as_ptr(),
                    dst.add(i as usize * dst_bpp),
                    n as usize * dst_bpp,
                );
            }
        }
    }
}
#[cfg(all(target_arch = "aarch64", not(skcms_portable)))]
pub use neon::*;
macro_rules! stage_noctx_template {
    ($name:ident,$kernel:ident,|$src:ident,$dst:ident,$r:ident,$g:ident,$b:ident,$a:ident,$i:ident|$body:block) => {
        pub unsafe fn $name(
            ctx: Ctx,
            src: *const u8,
            dst: *mut u8,
            r: &mut F,
            g: &mut F,
            b: &mut F,
            a: &mut F,
            i: i32,
        ) {
            unsafe { $kernel(ctx.no_ctx(), src, dst, r, g, b, a, i) }
        }
        pub unsafe fn $kernel(
            _ctx: NoCtx,
            $src: *const u8,
            $dst: *mut u8,
            $r: &mut F,
            $g: &mut F,
            $b: &mut F,
            $a: &mut F,
            $i: i32,
        ) {
            unsafe { $body }
        }
    };
}
// Native context-taking expansion of the same DECLARE_STAGE macro above.
macro_rules! stage_ctx_template {
    ($name:ident,$kernel:ident,$ty:ty,|$ctx:ident,$src:ident,$dst:ident,$r:ident,$g:ident,$b:ident,$a:ident,$i:ident|$body:block) => {
        pub unsafe fn $name(
            ctx: Ctx,
            src: *const u8,
            dst: *mut u8,
            r: &mut F,
            g: &mut F,
            b: &mut F,
            a: &mut F,
            i: i32,
        ) {
            unsafe { $kernel(&*ctx.as_ptr::<$ty>(), src, dst, r, g, b, a, i) }
        }
        pub unsafe fn $kernel(
            $ctx: &$ty,
            $src: *const u8,
            $dst: *mut u8,
            $r: &mut F,
            $g: &mut F,
            $b: &mut F,
            $a: &mut F,
            $i: i32,
        ) {
            unsafe { $body }
        }
    };
}

pub(crate) use stage_ctx_template;
pub(crate) use stage_noctx_template;
// Same native template body, width supplied at compile time. Backend helpers
// are actual source ISA branches in the invoking module, never a runtime provider.
macro_rules! instantiate_transform_common {
    ($n:literal) => {
        pub type F = [f32; $n];
        pub type I32 = [i32; $n];
        pub type U64 = [u64; $n];
        pub type U32 = [u32; $n];
        pub type U16 = [u16; $n];
        pub type U8 = [u8; $n];
        pub const F0: F = [0.0; $n];
        pub const F1: F = [1.0; $n];
        pub const FInfBits: F = [0x7f800000u32 as f32; $n];
        // Source: modules/skcms/src/Transform_inl.h:97-102
        /// Caller supplies readable storage for all bytes of T; no ownership is transferred.
        pub unsafe fn load<T: Copy, P>(ptr: *const P) -> T {
            unsafe { unsafe { ptr.cast::<T>().read_unaligned() } }
        }
        // Source: modules/skcms/src/Transform_inl.h:103-106
        /// Caller supplies exclusive writable storage for all bytes of T.
        pub unsafe fn store<T: Copy, P>(ptr: *mut P, val: &T) {
            unsafe { unsafe { ptr.cast::<T>().write_unaligned(*val) } }
        }
        // Source: modules/skcms/src/Transform_inl.h:110-118
        // Rust trait dispatch replaces the native compile-time convertvector intrinsic.
        // Primitive casts match all native-defined lane conversions; Rust defines saturation for float overflow.
        pub trait CastLane<D>: Copy {
            fn cast_lane(self) -> D;
        }
        impl CastLane<f32> for f32 {
            fn cast_lane(self) -> f32 {
                self as f32
            }
        }
        impl CastLane<i32> for f32 {
            fn cast_lane(self) -> i32 {
                self as i32
            }
        }
        impl CastLane<u64> for f32 {
            fn cast_lane(self) -> u64 {
                self as u64
            }
        }
        impl CastLane<u32> for f32 {
            fn cast_lane(self) -> u32 {
                self as u32
            }
        }
        impl CastLane<u16> for f32 {
            fn cast_lane(self) -> u16 {
                self as u16
            }
        }
        impl CastLane<u8> for f32 {
            fn cast_lane(self) -> u8 {
                self as u8
            }
        }
        impl CastLane<f32> for i32 {
            fn cast_lane(self) -> f32 {
                self as f32
            }
        }
        impl CastLane<i32> for i32 {
            fn cast_lane(self) -> i32 {
                self as i32
            }
        }
        impl CastLane<u64> for i32 {
            fn cast_lane(self) -> u64 {
                self as u64
            }
        }
        impl CastLane<u32> for i32 {
            fn cast_lane(self) -> u32 {
                self as u32
            }
        }
        impl CastLane<u16> for i32 {
            fn cast_lane(self) -> u16 {
                self as u16
            }
        }
        impl CastLane<u8> for i32 {
            fn cast_lane(self) -> u8 {
                self as u8
            }
        }
        impl CastLane<f32> for u64 {
            fn cast_lane(self) -> f32 {
                self as f32
            }
        }
        impl CastLane<i32> for u64 {
            fn cast_lane(self) -> i32 {
                self as i32
            }
        }
        impl CastLane<u64> for u64 {
            fn cast_lane(self) -> u64 {
                self as u64
            }
        }
        impl CastLane<u32> for u64 {
            fn cast_lane(self) -> u32 {
                self as u32
            }
        }
        impl CastLane<u16> for u64 {
            fn cast_lane(self) -> u16 {
                self as u16
            }
        }
        impl CastLane<u8> for u64 {
            fn cast_lane(self) -> u8 {
                self as u8
            }
        }
        impl CastLane<f32> for u32 {
            fn cast_lane(self) -> f32 {
                self as f32
            }
        }
        impl CastLane<i32> for u32 {
            fn cast_lane(self) -> i32 {
                self as i32
            }
        }
        impl CastLane<u64> for u32 {
            fn cast_lane(self) -> u64 {
                self as u64
            }
        }
        impl CastLane<u32> for u32 {
            fn cast_lane(self) -> u32 {
                self as u32
            }
        }
        impl CastLane<u16> for u32 {
            fn cast_lane(self) -> u16 {
                self as u16
            }
        }
        impl CastLane<u8> for u32 {
            fn cast_lane(self) -> u8 {
                self as u8
            }
        }
        impl CastLane<f32> for u16 {
            fn cast_lane(self) -> f32 {
                self as f32
            }
        }
        impl CastLane<i32> for u16 {
            fn cast_lane(self) -> i32 {
                self as i32
            }
        }
        impl CastLane<u64> for u16 {
            fn cast_lane(self) -> u64 {
                self as u64
            }
        }
        impl CastLane<u32> for u16 {
            fn cast_lane(self) -> u32 {
                self as u32
            }
        }
        impl CastLane<u16> for u16 {
            fn cast_lane(self) -> u16 {
                self as u16
            }
        }
        impl CastLane<u8> for u16 {
            fn cast_lane(self) -> u8 {
                self as u8
            }
        }
        impl CastLane<f32> for u8 {
            fn cast_lane(self) -> f32 {
                self as f32
            }
        }
        impl CastLane<i32> for u8 {
            fn cast_lane(self) -> i32 {
                self as i32
            }
        }
        impl CastLane<u64> for u8 {
            fn cast_lane(self) -> u64 {
                self as u64
            }
        }
        impl CastLane<u32> for u8 {
            fn cast_lane(self) -> u32 {
                self as u32
            }
        }
        impl CastLane<u16> for u8 {
            fn cast_lane(self) -> u16 {
                self as u16
            }
        }
        impl CastLane<u8> for u8 {
            fn cast_lane(self) -> u8 {
                self as u8
            }
        }
        pub unsafe fn cast<D: Copy, S: CastLane<D>>(v: &[S; $n]) -> [D; $n] {
            unsafe { unsafe { v.map(CastLane::cast_lane) } }
        }
        // Source: modules/skcms/src/Transform_inl.h:120-124
        /// Caller guarantees the copied bytes form a valid D value.
        pub unsafe fn bit_pun<D: Copy, S: Copy>(v: &S) -> D {
            unsafe {
                assert_eq!(std::mem::size_of::<D>(), std::mem::size_of::<S>());
                unsafe { load(v) }
            }
        }
        // Source: modules/skcms/src/Transform_inl.h:206-209
        pub unsafe fn swap_endian_16x4(rgba: &U64) -> U64 {
            unsafe {
                unsafe {
                    rgba.map(|x| ((x & 0x00ff00ff00ff00ff) << 8) | ((x & 0xff00ff00ff00ff00) >> 8))
                }
            }
        }
        // Source: modules/skcms/src/Transform_inl.h:250-262
        pub unsafe fn approx_log2(x: F) -> F {
            unsafe {
                unsafe {
                    std::array::from_fn(|i| {
                        let bits = x[i].to_bits() as i32;
                        let e = bits as f32 * (1.0 / (1 << 23) as f32);
                        let m = f32::from_bits(((bits & 0x007fffff) | 0x3f000000) as u32);
                        e - 124.225514990_f32
                            - 1.498030302_f32 * m
                            - 1.725879990_f32 / (0.3520887068_f32 + m)
                    })
                }
            }
        }
        // Source: modules/skcms/src/Transform_inl.h:264-267
        pub unsafe fn approx_log(x: F) -> F {
            unsafe { unsafe { approx_log2(x).map(|x| 0.69314718_f32 * x) } }
        }
        // Source: modules/skcms/src/Transform_inl.h:269-279
        pub unsafe fn approx_exp2(x: F) -> F {
            unsafe {
                unsafe {
                    let floor = floor_(x);
                    let fbits = std::array::from_fn(|i| {
                        let fract = x[i] - floor[i];
                        (1.0 * (1 << 23) as f32)
                            * (x[i] + 121.274057500_f32 - 1.490129070_f32 * fract
                                + 27.728023300_f32 / (4.84252568_f32 - fract))
                    });
                    let v = min_(max_(fbits, F0), FInfBits);
                    std::array::from_fn(|i| f32::from_bits((v[i] as i32) as u32))
                }
            }
        }
        // Source: modules/skcms/src/Transform_inl.h:280-283
        pub unsafe fn approx_pow(x: F, y: f32) -> F {
            unsafe {
                unsafe {
                    if $n == 1 && (x[0] == 0.0 || x[0] == 1.0) {
                        return x;
                    }
                    let calculated = approx_exp2(approx_log2(x).map(|x| x * y));
                    std::array::from_fn(|i| {
                        if x[i] == 0.0 || x[i] == 1.0 {
                            x[i]
                        } else {
                            calculated[i]
                        }
                    })
                }
            }
        }
        // Source: modules/skcms/src/Transform_inl.h:285-288
        pub unsafe fn approx_exp(x: F) -> F {
            unsafe { unsafe { approx_exp2(x.map(|x| 1.4426950408889634074_f32 * x)) } }
        }
        // Source: modules/skcms/src/Transform_inl.h:290-294
        pub unsafe fn strip_sign(x: F, sign: &mut U32) -> F {
            unsafe {
                unsafe {
                    std::array::from_fn(|i| {
                        let bits = x[i].to_bits();
                        sign[i] = bits & 0x80000000;
                        f32::from_bits(bits ^ sign[i])
                    })
                }
            }
        }
        // Source: modules/skcms/src/Transform_inl.h:296-298
        pub unsafe fn apply_sign(x: F, sign: U32) -> F {
            unsafe { unsafe { std::array::from_fn(|i| f32::from_bits(sign[i] | x[i].to_bits())) } }
        }

        use crate::modules::skcms::src::skcms_public::skcms_TransferFunction;
        // Source: modules/skcms/src/Transform_inl.h:301-312
        pub unsafe fn apply_tf(tf: &skcms_TransferFunction, mut x: F) -> F {
            unsafe {
                unsafe {
                    let mut sign = [0; $n];
                    x = strip_sign(x, &mut sign);
                    if $n == 1 && x[0] < tf.d {
                        return apply_sign(x.map(|v| tf.c * v + tf.f), sign);
                    }
                    let exponential = approx_pow(x.map(|x| tf.a * x + tf.b), tf.g);
                    let v = std::array::from_fn(|i| {
                        if x[i] < tf.d {
                            tf.c * x[i] + tf.f
                        } else {
                            exponential[i] + tf.e
                        }
                    });
                    apply_sign(v, sign)
                }
            }
        }
        // Source: modules/skcms/src/Transform_inl.h:315-319
        pub unsafe fn apply_gamma(tf: &skcms_TransferFunction, mut x: F) -> F {
            unsafe {
                unsafe {
                    let mut sign = [0; $n];
                    x = strip_sign(x, &mut sign);
                    apply_sign(approx_pow(x, tf.g), sign)
                }
            }
        }
        // Source: modules/skcms/src/Transform_inl.h:321-331
        pub unsafe fn apply_pq(tf: &skcms_TransferFunction, mut x: F) -> F {
            unsafe {
                unsafe {
                    let mut sign = [0; $n];
                    x = strip_sign(x, &mut sign);
                    let powered = approx_pow(x, tf.c);
                    let numerator = max_(powered.map(|v| tf.a + tf.b * v), F0);
                    let v = approx_pow(
                        std::array::from_fn(|i| numerator[i] / (tf.d + tf.e * powered[i])),
                        tf.f,
                    );
                    apply_sign(v, sign)
                }
            }
        }
        // Source: modules/skcms/src/Transform_inl.h:333-345
        pub unsafe fn apply_hlg(tf: &skcms_TransferFunction, mut x: F) -> F {
            unsafe {
                unsafe {
                    let (r, g, a, b, c, k) = (tf.a, tf.b, tf.c, tf.d, tf.e, tf.f + 1.0);
                    let mut sign = [0; $n];
                    x = strip_sign(x, &mut sign);
                    if $n == 1 {
                        let v = if x[0] * r <= 1.0 {
                            approx_pow(x.map(|v| v * r), g)
                        } else {
                            approx_exp(x.map(|v| (v - c) * a)).map(|v| v + b)
                        };
                        return apply_sign(v, sign).map(|v| k * v);
                    }
                    let power = approx_pow(x.map(|x| x * r), g);
                    let exponential = approx_exp(x.map(|x| (x - c) * a));
                    let v = std::array::from_fn(|i| {
                        if x[i] * r <= 1.0 {
                            power[i]
                        } else {
                            exponential[i] + b
                        }
                    });
                    apply_sign(v, sign).map(|v| k * v)
                }
            }
        }
        // Source: modules/skcms/src/Transform_inl.h:347-360
        pub unsafe fn apply_hlginv(tf: &skcms_TransferFunction, mut x: F) -> F {
            unsafe {
                unsafe {
                    let (r, g, a, b, c, k) = (tf.a, tf.b, tf.c, tf.d, tf.e, tf.f + 1.0);
                    let mut sign = [0; $n];
                    x = strip_sign(x, &mut sign);
                    x = x.map(|x| x / k);
                    if $n == 1 {
                        let v = if x[0] <= 1.0 {
                            approx_pow(x, g).map(|v| r * v)
                        } else {
                            approx_log(x.map(|v| v - b)).map(|v| a * v + c)
                        };
                        return apply_sign(v, sign);
                    }
                    let power = approx_pow(x, g);
                    let logarithm = approx_log(x.map(|x| x - b));
                    let v = std::array::from_fn(|i| {
                        if x[i] <= 1.0 {
                            r * power[i]
                        } else {
                            a * logarithm[i] + c
                        }
                    });
                    apply_sign(v, sign)
                }
            }
        }
        // Source: modules/skcms/src/Transform_inl.h:365-369
        pub unsafe fn compute_Y_in_xyzd50(x: F, y: F, z: F) -> F {
            unsafe {
                unsafe {
                    std::array::from_fn(|i| {
                        -0.02831655_f32 * x[i] + 1.00995452_f32 * y[i] + 0.02102382_f32 * z[i]
                    })
                }
            }
        }
        // Source: modules/skcms/src/Transform_inl.h:372-432
        pub unsafe fn load_3<D: Copy, P: CastLane<D>>(p: *const P) -> [D; $n] {
            unsafe { std::array::from_fn(|j| unsafe { (*p.add(3 * j)).cast_lane() }) }
        }
        pub unsafe fn load_4<D: Copy, P: CastLane<D>>(p: *const P) -> [D; $n] {
            unsafe { std::array::from_fn(|j| unsafe { (*p.add(4 * j)).cast_lane() }) }
        }
        pub unsafe fn store_3<T: CastLane<P>, P: Copy>(p: *mut P, v: &[T; $n]) {
            unsafe {
                for j in 0..$n {
                    unsafe { p.add(3 * j).write(v[j].cast_lane()) }
                }
            }
        }
        pub unsafe fn store_4<T: CastLane<P>, P: Copy>(p: *mut P, v: &[T; $n]) {
            unsafe {
                for j in 0..$n {
                    unsafe { p.add(4 * j).write(v[j].cast_lane()) }
                }
            }
        }
        // Source: modules/skcms/src/Transform_inl.h:589-591
        pub unsafe fn F_from_U8(v: U8) -> F {
            unsafe { unsafe { cast::<f32, _>(&v).map(|x| x * (1.0 / 255.0)) } }
        }
        // Source: modules/skcms/src/Transform_inl.h:593-599
        pub unsafe fn F_from_U16_BE(v: U16) -> F {
            unsafe {
                unsafe {
                    let lo = v.map(|v| v >> 8);
                    let hi = v.map(|v| v.wrapping_shl(8) & 0xffff);
                    cast::<f32, _>(&std::array::from_fn(|i| lo[i] | hi[i]))
                        .map(|x| x * (1.0 / 65535.0))
                }
            }
        }
        // Source: modules/skcms/src/Transform_inl.h:601-604
        pub unsafe fn U16_from_F(v: F) -> U16 {
            unsafe { unsafe { cast(&v.map(|x| x * 65535.0 + 0.5)) } }
        }
        // Source: modules/skcms/src/Transform_inl.h:606-608
        pub unsafe fn minus_1_ulp(v: F) -> F {
            unsafe { unsafe { v.map(|x| f32::from_bits(x.to_bits().wrapping_sub(1))) } }
        }
        // Source: modules/skcms/src/Transform_inl.h:610-632
        /// Caller supplies a nonempty valid borrowed curve table and native-defined index conversion inputs.
        pub unsafe fn table(
            curve: &crate::modules::skcms::src::skcms_public::skcms_Curve,
            v: F,
        ) -> F {
            unsafe {
                let curve = unsafe { curve.table };
                let ix = max_(F0, min_(v, F1)).map(|x| x * (curve.table_entries - 1) as f32);
                let lo: I32 = cast(&ix);
                let hi: I32 = cast(&minus_1_ulp(ix.map(|x| x + 1.0)));
                let t = std::array::from_fn::<_, $n, _>(|i| ix[i] - lo[i] as f32);
                let (l, h) = if !curve.table_8.is_null() {
                    unsafe {
                        (
                            F_from_U8(gather_8(curve.table_8, lo)),
                            F_from_U8(gather_8(curve.table_8, hi)),
                        )
                    }
                } else {
                    unsafe {
                        (
                            F_from_U16_BE(gather_16(curve.table_16, lo)),
                            F_from_U16_BE(gather_16(curve.table_16, hi)),
                        )
                    }
                };
                std::array::from_fn(|i| l[i] + (h[i] - l[i]) * t[i])
            }
        }
        // Source: modules/skcms/src/Transform_inl.h:634-640
        /// Caller supplies valid RGB grid storage including native gather24 overread bytes.
        pub unsafe fn sample_clut_8_rgb(grid: *const u8, ix: I32, r: &mut F, g: &mut F, b: &mut F) {
            unsafe {
                let rgb = unsafe { gather_24(grid, ix) };
                *r = rgb.map(|x| ((x >> 0) & 0xff) as f32 * (1.0 / 255.0));
                *g = rgb.map(|x| ((x >> 8) & 0xff) as f32 * (1.0 / 255.0));
                *b = rgb.map(|x| ((x >> 16) & 0xff) as f32 * (1.0 / 255.0));
            }
        }
        // Source: modules/skcms/src/Transform_inl.h:642-650
        /// Caller supplies valid four-channel grid storage.
        pub unsafe fn sample_clut_8_rgba(
            grid: *const u8,
            ix: I32,
            r: &mut F,
            g: &mut F,
            b: &mut F,
            a: &mut F,
        ) {
            unsafe {
                let rgba = unsafe { gather_32(grid, ix) };
                *r = rgba.map(|x| ((x >> 0) & 0xff) as f32 * (1.0 / 255.0));
                *g = rgba.map(|x| ((x >> 8) & 0xff) as f32 * (1.0 / 255.0));
                *b = rgba.map(|x| ((x >> 16) & 0xff) as f32 * (1.0 / 255.0));
                *a = rgba.map(|x| ((x >> 24) & 0xff) as f32 * (1.0 / 255.0));
            }
        }
        // Source: modules/skcms/src/Transform_inl.h:652-668
        /// Caller retains the actual grid bytes, including the native gather overread domain.
        pub unsafe fn sample_clut_16_rgb(
            grid: *const u8,
            ix: I32,
            r: &mut F,
            g: &mut F,
            b: &mut F,
        ) {
            unsafe {
                #[cfg(any(
                    target_arch = "arm",
                    all(target_arch = "loongarch64", target_feature = "lsx")
                ))]
                {
                    *r = F_from_U16_BE(gather_16(grid, ix.map(|v| 3 * v)));
                    *g = F_from_U16_BE(gather_16(grid, ix.map(|v| 3 * v + 1)));
                    *b = F_from_U16_BE(gather_16(grid, ix.map(|v| 3 * v + 2)));
                }
                #[cfg(not(any(
                    target_arch = "arm",
                    all(target_arch = "loongarch64", target_feature = "lsx")
                )))]
                {
                    let mut rgb = [0; $n];
                    gather_48(grid, ix, &mut rgb);
                    rgb = swap_endian_16x4(&rgb);
                    *r = rgb.map(|x| ((x >> 0) & 0xffff) as f32 * (1.0 / 65535.0));
                    *g = rgb.map(|x| ((x >> 16) & 0xffff) as f32 * (1.0 / 65535.0));
                    *b = rgb.map(|x| ((x >> 32) & 0xffff) as f32 * (1.0 / 65535.0));
                }
            }
        }
        // Source: modules/skcms/src/Transform_inl.h:670-676
        /// Caller supplies valid RGBA16 grid storage.
        pub unsafe fn sample_clut_16_rgba(
            grid: *const u8,
            ix: I32,
            r: &mut F,
            g: &mut F,
            b: &mut F,
            a: &mut F,
        ) {
            unsafe {
                unsafe {
                    *r = F_from_U16_BE(gather_16(grid, ix.map(|i| 4 * i)));
                    *g = F_from_U16_BE(gather_16(grid, ix.map(|i| 4 * i + 1)));
                    *b = F_from_U16_BE(gather_16(grid, ix.map(|i| 4 * i + 2)));
                    *a = F_from_U16_BE(gather_16(grid, ix.map(|i| 4 * i + 3)));
                }
            }
        }
        // Source: modules/skcms/src/Transform_inl.h:678-760
        /// Caller supplies a valid borrowed ICC CLUT with native readable gather-overrun bytes.
        /// The four mutable channel references must identify distinct registers.
        pub unsafe fn clut(
            input_channels: u32,
            output_channels: u32,
            grid_points: &[u8; 4],
            grid_8: *const u8,
            grid_16: *const u8,
            r: &mut F,
            g: &mut F,
            b: &mut F,
            a: &mut F,
        ) {
            unsafe {
                let dim = input_channels as i32;
                if dim <= 0 || dim > 4 {
                    return;
                }
                debug_assert!(output_channels == 3 || output_channels == 4);
                let mut index = [[0i32; $n]; 8];
                let mut weight = [F0; 8];
                let inputs = [*r, *g, *b, *a];
                let mut stride = 1;
                for i in (0..dim as usize).rev() {
                    let x = max_(F0, min_(inputs[i], F1))
                        .map(|x| x * (grid_points[i] as i32 - 1) as f32);
                    let lo: I32 = cast(&x);
                    let hi: I32 = cast(&minus_1_ulp(x.map(|x| x + 1.0)));
                    index[i] = lo.map(|x| x * stride);
                    index[i + 4] = hi.map(|x| x * stride);
                    stride *= grid_points[i] as i32;
                    let t = std::array::from_fn::<_, $n, _>(|j| x[j] - lo[j] as f32);
                    weight[i] = t.map(|x| 1.0 - x);
                    weight[i + 4] = t;
                }
                *r = F0;
                *g = F0;
                *b = F0;
                if output_channels == 4 {
                    *a = F0;
                }
                for combo in 0..(1 << dim) {
                    let mut ix = index[(combo & 1) * 4];
                    let mut w = weight[(combo & 1) * 4];
                    // Native switch falls through in the order dimension3, dimension2, dimension1.
                    if (dim - 1) & 3 >= 3 {
                        let k = 3 + (combo & 8) / 2;
                        for j in 0..$n {
                            ix[j] += index[k][j];
                            w[j] *= weight[k][j];
                        }
                    }
                    if (dim - 1) & 3 >= 2 {
                        let k = 2 + (combo & 4);
                        for j in 0..$n {
                            ix[j] += index[k][j];
                            w[j] *= weight[k][j];
                        }
                    }
                    if (dim - 1) & 3 >= 1 {
                        let k = 1 + (combo & 2) * 2;
                        for j in 0..$n {
                            ix[j] += index[k][j];
                            w[j] *= weight[k][j];
                        }
                    }
                    let (mut rr, mut gg, mut bb, mut aa) = (F0, F0, F0, F0);
                    if output_channels == 3 {
                        if !grid_8.is_null() {
                            unsafe { sample_clut_8_rgb(grid_8, ix, &mut rr, &mut gg, &mut bb) }
                        } else {
                            unsafe { sample_clut_16_rgb(grid_16, ix, &mut rr, &mut gg, &mut bb) }
                        }
                    } else {
                        if !grid_8.is_null() {
                            unsafe {
                                sample_clut_8_rgba(grid_8, ix, &mut rr, &mut gg, &mut bb, &mut aa)
                            }
                        } else {
                            unsafe {
                                sample_clut_16_rgba(grid_16, ix, &mut rr, &mut gg, &mut bb, &mut aa)
                            }
                        }
                    }
                    for j in 0..$n {
                        r[j] += w[j] * rr[j];
                        g[j] += w[j] * gg[j];
                        b[j] += w[j] * bb[j];
                        a[j] += w[j] * aa[j];
                    }
                }
            }
        }
        // Source: modules/skcms/src/Transform_inl.h:762-766
        pub unsafe fn clut_A2B(
            a2b: &crate::modules::skcms::src::skcms_public::skcms_A2B,
            r: &mut F,
            g: &mut F,
            b: &mut F,
            mut a: F,
        ) {
            unsafe {
                unsafe {
                    clut(
                        a2b.input_channels,
                        a2b.output_channels,
                        &a2b.grid_points,
                        a2b.grid_8,
                        a2b.grid_16,
                        r,
                        g,
                        b,
                        &mut a,
                    )
                }
            }
        }
        // Source: modules/skcms/src/Transform_inl.h:767-771
        pub unsafe fn clut_B2A(
            b2a: &crate::modules::skcms::src::skcms_public::skcms_B2A,
            r: &mut F,
            g: &mut F,
            b: &mut F,
            a: &mut F,
        ) {
            unsafe {
                unsafe {
                    clut(
                        b2a.input_channels,
                        b2a.output_channels,
                        &b2a.grid_points,
                        b2a.grid_8,
                        b2a.grid_16,
                        r,
                        g,
                        b,
                        a,
                    )
                }
            }
        }

        // Source: modules/skcms/src/Transform_inl.h:773-773
        #[derive(Clone, Copy)]
        pub struct NoCtx;
        // Source: modules/skcms/src/Transform_inl.h:775-779
        #[derive(Clone, Copy)]
        pub struct Ctx {
            pub fArg: *const std::ffi::c_void,
        }
        impl Ctx {
            pub fn no_ctx(self) -> NoCtx {
                NoCtx
            }
            pub fn as_ptr<T>(self) -> *const T {
                self.fArg.cast()
            }
        }
        // Source: modules/skcms/src/Transform_inl.h:781-787
        // Rust function references retain the four distinct native mutable register leases.
        pub type StageFn = unsafe fn(Ctx, *const u8, *mut u8, &mut F, &mut F, &mut F, &mut F, i32);
        // Source: modules/skcms/src/Transform_inl.h:822-832
        // Selected SKCMS_HAS_MUSTTAIL=0: each wrapper executes its actual kernel and returns to dispatcher.
        // Source: modules/skcms/src/Transform_inl.h:836-838
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_load_a8,
            Exec_load_a8_k,
            |src, dst, r, g, b, a, i| {
                *a = F_from_U8(load(src.offset(i as isize)));
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:840-842
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_load_g8,
            Exec_load_g8_k,
            |src, dst, r, g, b, a, i| {
                *r = F_from_U8(load(src.offset(i as isize)));
                *g = *r;
                *b = *r;
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:844-848
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_load_ga88,
            Exec_load_ga88_k,
            |src, dst, r, g, b, a, i| {
                let u16: U16 = load(src.offset(2 * i as isize));
                *r = u16.map(|x| (x & 0xff) as f32 * (1.0 / 255.0));
                *g = *r;
                *b = *r;
                *a = u16.map(|x| ((x >> 8) & 0xff) as f32 * (1.0 / 255.0));
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:850-857
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_load_4444,
            Exec_load_4444_k,
            |src, dst, r, g, b, a, i| {
                let abgr: U16 = load(src.offset(2 * i as isize));
                *r = abgr.map(|x| ((x >> 12) & 0xf) as f32 * (1.0 / 15.0));
                *g = abgr.map(|x| ((x >> 8) & 0xf) as f32 * (1.0 / 15.0));
                *b = abgr.map(|x| ((x >> 4) & 0xf) as f32 * (1.0 / 15.0));
                *a = abgr.map(|x| (x & 0xf) as f32 * (1.0 / 15.0));
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:859-865
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_load_565,
            Exec_load_565_k,
            |src, dst, r, g, b, a, i| {
                let rgb: U16 = load(src.offset(2 * i as isize));
                *r = rgb.map(|x| (x & 31) as f32 * (1.0 / 31.0));
                *g = rgb.map(|x| (x & (63 << 5)) as f32 * (1.0 / (63 << 5) as f32));
                *b = rgb.map(|x| (x & (31 << 11)) as f32 * (1.0 / (31 << 11) as f32));
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:867-890
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_load_888,
            Exec_load_888_k,
            |src, dst, r, g, b, a, i| {
                let rgb = src.offset(3 * i as isize);
                *r = cast::<f32, _>(&load_3::<u32, _>(rgb)).map(|v| v * (1.0 / 255.0));
                *g = cast::<f32, _>(&load_3::<u32, _>(rgb.add(1))).map(|v| v * (1.0 / 255.0));
                *b = cast::<f32, _>(&load_3::<u32, _>(rgb.add(2))).map(|v| v * (1.0 / 255.0));
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:892-899
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_load_8888,
            Exec_load_8888_k,
            |src, dst, r, g, b, a, i| {
                let rgba: U32 = load(src.offset(4 * i as isize));
                *r = rgba.map(|x| ((x >> 0) & 0xff) as f32 * (1.0 / 255.0));
                *g = rgba.map(|x| ((x >> 8) & 0xff) as f32 * (1.0 / 255.0));
                *b = rgba.map(|x| ((x >> 16) & 0xff) as f32 * (1.0 / 255.0));
                *a = rgba.map(|x| ((x >> 24) & 0xff) as f32 * (1.0 / 255.0));
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:901-908
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_load_1010102,
            Exec_load_1010102_k,
            |src, dst, r, g, b, a, i| {
                let rgba: U32 = load(src.offset(4 * i as isize));
                *r = rgba.map(|x| ((x >> 0) & 0x3ff) as f32 * (1.0 / 1023.0));
                *g = rgba.map(|x| ((x >> 10) & 0x3ff) as f32 * (1.0 / 1023.0));
                *b = rgba.map(|x| ((x >> 20) & 0x3ff) as f32 * (1.0 / 1023.0));
                *a = rgba.map(|x| ((x >> 30) & 0x3) as f32 * (1.0 / 3.0));
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:910-915
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_load_101010x_XR,
            Exec_load_101010x_XR_k,
            |src, dst, r, g, b, a, i| {
                let rgba: U32 = load(src.offset(4 * i as isize));
                *r = rgba.map(|x| ((x >> 0) & 0x3ff).wrapping_sub(384) as f32 / 510.0);
                *g = rgba.map(|x| ((x >> 10) & 0x3ff).wrapping_sub(384) as f32 / 510.0);
                *b = rgba.map(|x| ((x >> 20) & 0x3ff).wrapping_sub(384) as f32 / 510.0);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:917-924
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_load_10101010_XR,
            Exec_load_10101010_XR_k,
            |src, dst, r, g, b, a, i| {
                let rgba: U64 = load(src.offset(8 * i as isize));
                *r = rgba.map(|x| ((x >> 6) & 0x3ff).wrapping_sub(384) as f32 / 510.0);
                *g = rgba.map(|x| ((x >> 22) & 0x3ff).wrapping_sub(384) as f32 / 510.0);
                *b = rgba.map(|x| ((x >> 38) & 0x3ff).wrapping_sub(384) as f32 / 510.0);
                *a = rgba.map(|x| ((x >> 54) & 0x3ff).wrapping_sub(384) as f32 / 510.0);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:926-940
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_load_161616LE,
            Exec_load_161616LE_k,
            |src, dst, r, g, b, a, i| {
                let ptr = src.offset(6 * i as isize);
                debug_assert_eq!(ptr as usize & 1, 0);
                let rgb = ptr.cast::<u16>();
                *r = cast::<f32, _>(&load_3::<u32, _>(rgb)).map(|v| v * (1.0 / 65535.0));
                *g = cast::<f32, _>(&load_3::<u32, _>(rgb.add(1))).map(|v| v * (1.0 / 65535.0));
                *b = cast::<f32, _>(&load_3::<u32, _>(rgb.add(2))).map(|v| v * (1.0 / 65535.0));
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:942-960
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_load_16161616LE,
            Exec_load_16161616LE_k,
            |src, dst, r, g, b, a, i| {
                let ptr = src.offset(8 * i as isize);
                debug_assert_eq!(ptr as usize & 1, 0);
                let px: U64 = load(ptr);
                *r = px.map(|v| ((v >> 0) & 0xffff) as f32 * (1.0 / 65535.0));
                *g = px.map(|v| ((v >> 16) & 0xffff) as f32 * (1.0 / 65535.0));
                *b = px.map(|v| ((v >> 32) & 0xffff) as f32 * (1.0 / 65535.0));
                *a = px.map(|v| ((v >> 48) & 0xffff) as f32 * (1.0 / 65535.0));
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:962-980
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_load_161616BE,
            Exec_load_161616BE_k,
            |src, dst, r, g, b, a, i| {
                let ptr = src.offset(6 * i as isize);
                debug_assert_eq!(ptr as usize & 1, 0);
                let rgb = ptr.cast::<u16>();
                let rr = load_3::<u32, _>(rgb);
                let gg = load_3::<u32, _>(rgb.add(1));
                let bb = load_3::<u32, _>(rgb.add(2));
                *r = rr.map(|v| (((v & 0xff) << 8) | ((v & 0xff00) >> 8)) as f32 * (1.0 / 65535.0));
                *g = gg.map(|v| (((v & 0xff) << 8) | ((v & 0xff00) >> 8)) as f32 * (1.0 / 65535.0));
                *b = bb.map(|v| (((v & 0xff) << 8) | ((v & 0xff00) >> 8)) as f32 * (1.0 / 65535.0));
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:982-1000
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_load_16161616BE,
            Exec_load_16161616BE_k,
            |src, dst, r, g, b, a, i| {
                let ptr = src.offset(8 * i as isize);
                debug_assert_eq!(ptr as usize & 1, 0);
                let px = swap_endian_16x4(&load::<U64, _>(ptr));
                *r = px.map(|v| ((v >> 0) & 0xffff) as f32 * (1.0 / 65535.0));
                *g = px.map(|v| ((v >> 16) & 0xffff) as f32 * (1.0 / 65535.0));
                *b = px.map(|v| ((v >> 32) & 0xffff) as f32 * (1.0 / 65535.0));
                *a = px.map(|v| ((v >> 48) & 0xffff) as f32 * (1.0 / 65535.0));
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1002-1019
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_load_hhh,
            Exec_load_hhh_k,
            |src, dst, r, g, b, a, i| {
                let ptr = src.offset(6 * i as isize);
                debug_assert_eq!(ptr as usize & 1, 0);
                let rgb = ptr.cast::<u16>();
                *r = F_from_Half(load_3(rgb));
                *g = F_from_Half(load_3(rgb.add(1)));
                *b = F_from_Half(load_3(rgb.add(2)));
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1021-1042
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_load_hhhh,
            Exec_load_hhhh_k,
            |src, dst, r, g, b, a, i| {
                let ptr = src.offset(8 * i as isize);
                debug_assert_eq!(ptr as usize & 1, 0);
                let px: U64 = load(ptr);
                *r = F_from_Half(px.map(|v| ((v >> 0) & 0xffff) as u16));
                *g = F_from_Half(px.map(|v| ((v >> 16) & 0xffff) as u16));
                *b = F_from_Half(px.map(|v| ((v >> 32) & 0xffff) as u16));
                *a = F_from_Half(px.map(|v| ((v >> 48) & 0xffff) as u16));
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1044-1058
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_load_fff,
            Exec_load_fff_k,
            |src, dst, r, g, b, a, i| {
                let ptr = src.offset(12 * i as isize);
                debug_assert_eq!(ptr as usize & 3, 0);
                let rgb = ptr.cast::<f32>();
                *r = load_3(rgb);
                *g = load_3(rgb.add(1));
                *b = load_3(rgb.add(2));
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1060-1076
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_load_ffff,
            Exec_load_ffff_k,
            |src, dst, r, g, b, a, i| {
                let ptr = src.offset(16 * i as isize);
                debug_assert_eq!(ptr as usize & 3, 0);
                let rgba = ptr.cast::<f32>();
                *r = load_4(rgba);
                *g = load_4(rgba.add(1));
                *b = load_4(rgba.add(2));
                *a = load_4(rgba.add(3));
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1078-1082
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_swap_rb,
            Exec_swap_rb_k,
            |src, dst, r, g, b, a, i| {
                let t = *r;
                *r = *b;
                *b = t;
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1084-1089
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_clamp,
            Exec_clamp_k,
            |src, dst, r, g, b, a, i| {
                *r = max_(F0, min_(*r, F1));
                *g = max_(F0, min_(*g, F1));
                *b = max_(F0, min_(*b, F1));
                *a = max_(F0, min_(*a, F1));
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1091-1096
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_invert,
            Exec_invert_k,
            |src, dst, r, g, b, a, i| {
                *r = r.map(|x| 1.0 - x);
                *g = g.map(|x| 1.0 - x);
                *b = b.map(|x| 1.0 - x);
                *a = a.map(|x| 1.0 - x);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1098-1100
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_force_opaque,
            Exec_force_opaque_k,
            |src, dst, r, g, b, a, i| {
                *a = F1;
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1102-1106
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_premul,
            Exec_premul_k,
            |src, dst, r, g, b, a, i| {
                *r = std::array::from_fn(|j| r[j] * a[j]);
                *g = std::array::from_fn(|j| g[j] * a[j]);
                *b = std::array::from_fn(|j| b[j] * a[j]);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1108-1113
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_unpremul,
            Exec_unpremul_k,
            |src, dst, r, g, b, a, i| {
                let scale = std::array::from_fn::<_, $n, _>(|j| {
                    if 1.0 / a[j] < f32::INFINITY {
                        1.0 / a[j]
                    } else {
                        0.0
                    }
                });
                *r = std::array::from_fn(|j| r[j] * scale[j]);
                *g = std::array::from_fn(|j| g[j] * scale[j]);
                *b = std::array::from_fn(|j| b[j] * scale[j]);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1115-1123
        $crate::modules::skcms::src::Transform_inl::stage_ctx_template!(
            Exec_matrix_3x3,
            Exec_matrix_3x3_k,
            crate::modules::skcms::src::skcms_public::skcms_Matrix3x3,
            |matrix, src, dst, r, g, b, a, i| {
                let R = std::array::from_fn::<_, $n, _>(|j| {
                    matrix.vals[0][0] * r[j] + matrix.vals[0][1] * g[j] + matrix.vals[0][2] * b[j]
                });
                let G = std::array::from_fn::<_, $n, _>(|j| {
                    matrix.vals[1][0] * r[j] + matrix.vals[1][1] * g[j] + matrix.vals[1][2] * b[j]
                });
                let B = std::array::from_fn::<_, $n, _>(|j| {
                    matrix.vals[2][0] * r[j] + matrix.vals[2][1] * g[j] + matrix.vals[2][2] * b[j]
                });
                *r = R;
                *g = G;
                *b = B;
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1125-1136
        $crate::modules::skcms::src::Transform_inl::stage_ctx_template!(
            Exec_matrix_3x4,
            Exec_matrix_3x4_k,
            crate::modules::skcms::src::skcms_public::skcms_Matrix3x4,
            |matrix, src, dst, r, g, b, a, i| {
                let R = std::array::from_fn::<_, $n, _>(|j| {
                    matrix.vals[0][0] * r[j]
                        + matrix.vals[0][1] * g[j]
                        + matrix.vals[0][2] * b[j]
                        + matrix.vals[0][3]
                });
                let G = std::array::from_fn::<_, $n, _>(|j| {
                    matrix.vals[1][0] * r[j]
                        + matrix.vals[1][1] * g[j]
                        + matrix.vals[1][2] * b[j]
                        + matrix.vals[1][3]
                });
                let B = std::array::from_fn::<_, $n, _>(|j| {
                    matrix.vals[2][0] * r[j]
                        + matrix.vals[2][1] * g[j]
                        + matrix.vals[2][2] * b[j]
                        + matrix.vals[2][3]
                });
                *r = R;
                *g = G;
                *b = B;
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1138-1157
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_lab_to_xyz,
            Exec_lab_to_xyz_k,
            |src, dst, r, g, b, a, i| {
                let l = r.map(|v| v * 100.0);
                let aa = g.map(|v| v * 255.0 - 128.0);
                let bb = b.map(|v| v * 255.0 - 128.0);
                let mut y = l.map(|v| (v + 16.0) * (1.0 / 116.0));
                let mut x = std::array::from_fn::<_, $n, _>(|j| y[j] + aa[j] * (1.0 / 500.0));
                let mut z = std::array::from_fn::<_, $n, _>(|j| y[j] - bb[j] * (1.0 / 200.0));
                x = x.map(|v| {
                    if v * v * v > 0.008856 {
                        v * v * v
                    } else {
                        (v - (16.0 / 116.0)) * (1.0 / 7.787)
                    }
                });
                y = y.map(|v| {
                    if v * v * v > 0.008856 {
                        v * v * v
                    } else {
                        (v - (16.0 / 116.0)) * (1.0 / 7.787)
                    }
                });
                z = z.map(|v| {
                    if v * v * v > 0.008856 {
                        v * v * v
                    } else {
                        (v - (16.0 / 116.0)) * (1.0 / 7.787)
                    }
                });
                *r = x.map(|v| v * 0.9642);
                *g = y;
                *b = z.map(|v| v * 0.8249);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1160-1176
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_xyz_to_lab,
            Exec_xyz_to_lab_k,
            |src, dst, r, g, b, a, i| {
                let mut x = r.map(|v| v * (1.0 / 0.9642));
                let mut y = *g;
                let mut z = b.map(|v| v * (1.0 / 0.8249));
                if $n == 1 {
                    x = x.map(|v| {
                        if v > 0.008856 {
                            approx_pow([v; $n], 1.0 / 3.0)[0]
                        } else {
                            v * 7.787 + (16.0 / 116.0)
                        }
                    });
                    y = y.map(|v| {
                        if v > 0.008856 {
                            approx_pow([v; $n], 1.0 / 3.0)[0]
                        } else {
                            v * 7.787 + (16.0 / 116.0)
                        }
                    });
                    z = z.map(|v| {
                        if v > 0.008856 {
                            approx_pow([v; $n], 1.0 / 3.0)[0]
                        } else {
                            v * 7.787 + (16.0 / 116.0)
                        }
                    });
                } else {
                    let xp = approx_pow(x, 1.0 / 3.0);
                    let yp = approx_pow(y, 1.0 / 3.0);
                    let zp = approx_pow(z, 1.0 / 3.0);
                    x = std::array::from_fn(|j| {
                        if x[j] > 0.008856 {
                            xp[j]
                        } else {
                            x[j] * 7.787 + (16.0 / 116.0)
                        }
                    });
                    y = std::array::from_fn(|j| {
                        if y[j] > 0.008856 {
                            yp[j]
                        } else {
                            y[j] * 7.787 + (16.0 / 116.0)
                        }
                    });
                    z = std::array::from_fn(|j| {
                        if z[j] > 0.008856 {
                            zp[j]
                        } else {
                            z[j] * 7.787 + (16.0 / 116.0)
                        }
                    });
                }
                let l = y.map(|v| v * 116.0 - 16.0);
                let aa = std::array::from_fn::<_, $n, _>(|j| (x[j] - y[j]) * 500.0);
                let bb = std::array::from_fn::<_, $n, _>(|j| (y[j] - z[j]) * 200.0);
                *r = l.map(|v| v * (1.0 / 100.0));
                *g = aa.map(|v| (v + 128.0) * (1.0 / 255.0));
                *b = bb.map(|v| (v + 128.0) * (1.0 / 255.0));
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1178-1178
        $crate::modules::skcms::src::Transform_inl::stage_ctx_template!(
            Exec_gamma_r,
            Exec_gamma_r_k,
            crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
            |tf, src, dst, r, g, b, a, i| {
                *r = apply_gamma(tf, *r);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1179-1179
        $crate::modules::skcms::src::Transform_inl::stage_ctx_template!(
            Exec_gamma_g,
            Exec_gamma_g_k,
            crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
            |tf, src, dst, r, g, b, a, i| {
                *g = apply_gamma(tf, *g);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1180-1180
        $crate::modules::skcms::src::Transform_inl::stage_ctx_template!(
            Exec_gamma_b,
            Exec_gamma_b_k,
            crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
            |tf, src, dst, r, g, b, a, i| {
                *b = apply_gamma(tf, *b);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1181-1181
        $crate::modules::skcms::src::Transform_inl::stage_ctx_template!(
            Exec_gamma_a,
            Exec_gamma_a_k,
            crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
            |tf, src, dst, r, g, b, a, i| {
                *a = apply_gamma(tf, *a);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1183-1187
        $crate::modules::skcms::src::Transform_inl::stage_ctx_template!(
            Exec_gamma_rgb,
            Exec_gamma_rgb_k,
            crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
            |tf, src, dst, r, g, b, a, i| {
                *r = apply_gamma(tf, *r);
                *g = apply_gamma(tf, *g);
                *b = apply_gamma(tf, *b);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1189-1189
        $crate::modules::skcms::src::Transform_inl::stage_ctx_template!(
            Exec_tf_r,
            Exec_tf_r_k,
            crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
            |tf, src, dst, r, g, b, a, i| {
                *r = apply_tf(tf, *r);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1190-1190
        $crate::modules::skcms::src::Transform_inl::stage_ctx_template!(
            Exec_tf_g,
            Exec_tf_g_k,
            crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
            |tf, src, dst, r, g, b, a, i| {
                *g = apply_tf(tf, *g);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1191-1191
        $crate::modules::skcms::src::Transform_inl::stage_ctx_template!(
            Exec_tf_b,
            Exec_tf_b_k,
            crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
            |tf, src, dst, r, g, b, a, i| {
                *b = apply_tf(tf, *b);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1192-1192
        $crate::modules::skcms::src::Transform_inl::stage_ctx_template!(
            Exec_tf_a,
            Exec_tf_a_k,
            crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
            |tf, src, dst, r, g, b, a, i| {
                *a = apply_tf(tf, *a);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1194-1198
        $crate::modules::skcms::src::Transform_inl::stage_ctx_template!(
            Exec_tf_rgb,
            Exec_tf_rgb_k,
            crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
            |tf, src, dst, r, g, b, a, i| {
                *r = apply_tf(tf, *r);
                *g = apply_tf(tf, *g);
                *b = apply_tf(tf, *b);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1200-1200
        $crate::modules::skcms::src::Transform_inl::stage_ctx_template!(
            Exec_pq_r,
            Exec_pq_r_k,
            crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
            |tf, src, dst, r, g, b, a, i| {
                *r = apply_pq(tf, *r);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1201-1201
        $crate::modules::skcms::src::Transform_inl::stage_ctx_template!(
            Exec_pq_g,
            Exec_pq_g_k,
            crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
            |tf, src, dst, r, g, b, a, i| {
                *g = apply_pq(tf, *g);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1202-1202
        $crate::modules::skcms::src::Transform_inl::stage_ctx_template!(
            Exec_pq_b,
            Exec_pq_b_k,
            crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
            |tf, src, dst, r, g, b, a, i| {
                *b = apply_pq(tf, *b);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1203-1203
        $crate::modules::skcms::src::Transform_inl::stage_ctx_template!(
            Exec_pq_a,
            Exec_pq_a_k,
            crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
            |tf, src, dst, r, g, b, a, i| {
                *a = apply_pq(tf, *a);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1205-1209
        $crate::modules::skcms::src::Transform_inl::stage_ctx_template!(
            Exec_pq_rgb,
            Exec_pq_rgb_k,
            crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
            |tf, src, dst, r, g, b, a, i| {
                *r = apply_pq(tf, *r);
                *g = apply_pq(tf, *g);
                *b = apply_pq(tf, *b);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1211-1211
        $crate::modules::skcms::src::Transform_inl::stage_ctx_template!(
            Exec_hlg_r,
            Exec_hlg_r_k,
            crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
            |tf, src, dst, r, g, b, a, i| {
                *r = apply_hlg(tf, *r);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1212-1212
        $crate::modules::skcms::src::Transform_inl::stage_ctx_template!(
            Exec_hlg_g,
            Exec_hlg_g_k,
            crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
            |tf, src, dst, r, g, b, a, i| {
                *g = apply_hlg(tf, *g);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1213-1213
        $crate::modules::skcms::src::Transform_inl::stage_ctx_template!(
            Exec_hlg_b,
            Exec_hlg_b_k,
            crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
            |tf, src, dst, r, g, b, a, i| {
                *b = apply_hlg(tf, *b);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1214-1214
        $crate::modules::skcms::src::Transform_inl::stage_ctx_template!(
            Exec_hlg_a,
            Exec_hlg_a_k,
            crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
            |tf, src, dst, r, g, b, a, i| {
                *a = apply_hlg(tf, *a);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1216-1220
        $crate::modules::skcms::src::Transform_inl::stage_ctx_template!(
            Exec_hlg_rgb,
            Exec_hlg_rgb_k,
            crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
            |tf, src, dst, r, g, b, a, i| {
                *r = apply_hlg(tf, *r);
                *g = apply_hlg(tf, *g);
                *b = apply_hlg(tf, *b);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1223-1241
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_hlg_ootf_scale,
            Exec_hlg_ootf_scale_k,
            |src, dst, r, g, b, a, i| {
                let mut y = compute_Y_in_xyzd50(*r, *g, *b);
                let gamma_minus_1 = 0.2;
                let mut sign = [0; $n];
                y = strip_sign(y, &mut sign);
                let factor = apply_sign(approx_pow(y, gamma_minus_1), sign);
                *r = std::array::from_fn(|j| r[j] * factor[j]);
                *g = std::array::from_fn(|j| g[j] * factor[j]);
                *b = std::array::from_fn(|j| b[j] * factor[j]);
                *r = r.map(|v| v * (1000.0 / 203.0));
                *g = g.map(|v| v * (1000.0 / 203.0));
                *b = b.map(|v| v * (1000.0 / 203.0));
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1243-1243
        $crate::modules::skcms::src::Transform_inl::stage_ctx_template!(
            Exec_hlginv_r,
            Exec_hlginv_r_k,
            crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
            |tf, src, dst, r, g, b, a, i| {
                *r = apply_hlginv(tf, *r);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1244-1244
        $crate::modules::skcms::src::Transform_inl::stage_ctx_template!(
            Exec_hlginv_g,
            Exec_hlginv_g_k,
            crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
            |tf, src, dst, r, g, b, a, i| {
                *g = apply_hlginv(tf, *g);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1245-1245
        $crate::modules::skcms::src::Transform_inl::stage_ctx_template!(
            Exec_hlginv_b,
            Exec_hlginv_b_k,
            crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
            |tf, src, dst, r, g, b, a, i| {
                *b = apply_hlginv(tf, *b);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1246-1246
        $crate::modules::skcms::src::Transform_inl::stage_ctx_template!(
            Exec_hlginv_a,
            Exec_hlginv_a_k,
            crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
            |tf, src, dst, r, g, b, a, i| {
                *a = apply_hlginv(tf, *a);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1248-1252
        $crate::modules::skcms::src::Transform_inl::stage_ctx_template!(
            Exec_hlginv_rgb,
            Exec_hlginv_rgb_k,
            crate::modules::skcms::src::skcms_public::skcms_TransferFunction,
            |tf, src, dst, r, g, b, a, i| {
                *r = apply_hlginv(tf, *r);
                *g = apply_hlginv(tf, *g);
                *b = apply_hlginv(tf, *b);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1255-1269
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_hlginv_ootf_scale,
            Exec_hlginv_ootf_scale_k,
            |src, dst, r, g, b, a, i| {
                *r = r.map(|v| v * (203.0 / 1000.0));
                *g = g.map(|v| v * (203.0 / 1000.0));
                *b = b.map(|v| v * (203.0 / 1000.0));
                let gamma_inv_minus_1 = 1.0 / 1.2 - 1.0;
                let mut y = compute_Y_in_xyzd50(*r, *g, *b);
                let mut sign = [0; $n];
                y = strip_sign(y, &mut sign);
                let factor = apply_sign(approx_pow(y, gamma_inv_minus_1), sign);
                *r = std::array::from_fn(|j| r[j] * factor[j]);
                *g = std::array::from_fn(|j| g[j] * factor[j]);
                *b = std::array::from_fn(|j| b[j] * factor[j]);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1271-1271
        $crate::modules::skcms::src::Transform_inl::stage_ctx_template!(
            Exec_table_r,
            Exec_table_r_k,
            crate::modules::skcms::src::skcms_public::skcms_Curve,
            |curve, src, dst, r, g, b, a, i| {
                *r = table(curve, *r);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1272-1272
        $crate::modules::skcms::src::Transform_inl::stage_ctx_template!(
            Exec_table_g,
            Exec_table_g_k,
            crate::modules::skcms::src::skcms_public::skcms_Curve,
            |curve, src, dst, r, g, b, a, i| {
                *g = table(curve, *g);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1273-1273
        $crate::modules::skcms::src::Transform_inl::stage_ctx_template!(
            Exec_table_b,
            Exec_table_b_k,
            crate::modules::skcms::src::skcms_public::skcms_Curve,
            |curve, src, dst, r, g, b, a, i| {
                *b = table(curve, *b);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1274-1274
        $crate::modules::skcms::src::Transform_inl::stage_ctx_template!(
            Exec_table_a,
            Exec_table_a_k,
            crate::modules::skcms::src::skcms_public::skcms_Curve,
            |curve, src, dst, r, g, b, a, i| {
                *a = table(curve, *a);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1276-1283
        $crate::modules::skcms::src::Transform_inl::stage_ctx_template!(
            Exec_clut_A2B,
            Exec_clut_A2B_k,
            crate::modules::skcms::src::skcms_public::skcms_A2B,
            |a2b, src, dst, r, g, b, a, i| {
                clut_A2B(a2b, r, g, b, *a);
                if a2b.input_channels == 4 {
                    *a = F1;
                }
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1285-1287
        $crate::modules::skcms::src::Transform_inl::stage_ctx_template!(
            Exec_clut_B2A,
            Exec_clut_B2A_k,
            crate::modules::skcms::src::skcms_public::skcms_B2A,
            |b2a, src, dst, r, g, b, a, i| {
                clut_B2A(b2a, r, g, b, a);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1291-1293
        // FINAL_STAGE uses the same actual wrapper/kernel protocol in non-musttail configuration.
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_store_a8,
            Exec_store_a8_k,
            |src, dst, r, g, b, a, i| {
                let v: U8 = cast(&to_fixed(a.map(|x| x * 255.0)));
                store(dst.offset(i as isize), &v);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1295-1298
        // FINAL_STAGE uses the same actual wrapper/kernel protocol in non-musttail configuration.
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_store_g8,
            Exec_store_g8_k,
            |src, dst, r, g, b, a, i| {
                let v: U8 = cast(&to_fixed(g.map(|x| x * 255.0)));
                store(dst.offset(i as isize), &v);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1300-1304
        // FINAL_STAGE uses the same actual wrapper/kernel protocol in non-musttail configuration.
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_store_ga88,
            Exec_store_ga88_k,
            |src, dst, r, g, b, a, i| {
                let g: U16 = cast(&to_fixed(g.map(|x| x * 255.0)));
                let a: U16 = cast(&to_fixed(a.map(|x| x * 255.0)).map(|x| x << 8));
                let v: U16 = std::array::from_fn(|j| g[j] | a[j]);
                store(dst.offset(2 * i as isize), &v);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1306-1311
        // FINAL_STAGE uses the same actual wrapper/kernel protocol in non-musttail configuration.
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_store_4444,
            Exec_store_4444_k,
            |src, dst, r, g, b, a, i| {
                let r: U16 = cast(&to_fixed(r.map(|x| x * 15.0)).map(|x| x.wrapping_shl(12)));
                let g: U16 = cast(&to_fixed(g.map(|x| x * 15.0)).map(|x| x.wrapping_shl(8)));
                let b: U16 = cast(&to_fixed(b.map(|x| x * 15.0)).map(|x| x.wrapping_shl(4)));
                let a: U16 = cast(&to_fixed(a.map(|x| x * 15.0)).map(|x| x.wrapping_shl(0)));
                let v: U16 = std::array::from_fn(|j| r[j] | g[j] | b[j] | a[j]);
                store(dst.offset(2 * i as isize), &v);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1313-1317
        // FINAL_STAGE uses the same actual wrapper/kernel protocol in non-musttail configuration.
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_store_565,
            Exec_store_565_k,
            |src, dst, r, g, b, a, i| {
                let r: U16 = cast(&to_fixed(r.map(|x| x * 31.0)).map(|x| x.wrapping_shl(0)));
                let g: U16 = cast(&to_fixed(g.map(|x| x * 63.0)).map(|x| x.wrapping_shl(5)));
                let b: U16 = cast(&to_fixed(b.map(|x| x * 31.0)).map(|x| x.wrapping_shl(11)));
                let v: U16 = std::array::from_fn(|j| r[j] | g[j] | b[j]);
                store(dst.offset(2 * i as isize), &v);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1319-1339
        // FINAL_STAGE uses the same actual wrapper/kernel protocol in non-musttail configuration.
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_store_888,
            Exec_store_888_k,
            |src, dst, r, g, b, a, i| {
                let rgb = dst.offset(3 * i as isize);
                store_3(rgb, &cast::<u8, _>(&to_fixed(r.map(|v| v * 255.0))));
                store_3(rgb.add(1), &cast::<u8, _>(&to_fixed(g.map(|v| v * 255.0))));
                store_3(rgb.add(2), &cast::<u8, _>(&to_fixed(b.map(|v| v * 255.0))));
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1341-1346
        // FINAL_STAGE uses the same actual wrapper/kernel protocol in non-musttail configuration.
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_store_8888,
            Exec_store_8888_k,
            |src, dst, r, g, b, a, i| {
                let r: U32 =
                    cast::<u32, _>(&to_fixed(r.map(|x| x * 255.0))).map(|x| x.wrapping_shl(0));
                let g: U32 =
                    cast::<u32, _>(&to_fixed(g.map(|x| x * 255.0))).map(|x| x.wrapping_shl(8));
                let b: U32 =
                    cast::<u32, _>(&to_fixed(b.map(|x| x * 255.0))).map(|x| x.wrapping_shl(16));
                let a: U32 =
                    cast::<u32, _>(&to_fixed(a.map(|x| x * 255.0))).map(|x| x.wrapping_shl(24));
                let v: U32 = std::array::from_fn(|j| r[j] | g[j] | b[j] | a[j]);
                store(dst.offset(4 * i as isize), &v);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1348-1352
        // FINAL_STAGE uses the same actual wrapper/kernel protocol in non-musttail configuration.
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_store_101010x_XR,
            Exec_store_101010x_XR_k,
            |src, dst, r, g, b, a, i| {
                let r: U32 = cast::<u32, _>(&to_fixed(r.map(|x| x * 510.0 + 384.0)))
                    .map(|x| x.wrapping_shl(0));
                let g: U32 = cast::<u32, _>(&to_fixed(g.map(|x| x * 510.0 + 384.0)))
                    .map(|x| x.wrapping_shl(10));
                let b: U32 = cast::<u32, _>(&to_fixed(b.map(|x| x * 510.0 + 384.0)))
                    .map(|x| x.wrapping_shl(20));
                let v: U32 = std::array::from_fn(|j| r[j] | g[j] | b[j]);
                store(dst.offset(4 * i as isize), &v);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1354-1360
        // FINAL_STAGE uses the same actual wrapper/kernel protocol in non-musttail configuration.
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_store_10101010_XR,
            Exec_store_10101010_XR_k,
            |src, dst, r, g, b, a, i| {
                let r: U64 = cast::<u64, _>(&to_fixed(r.map(|x| x * 510.0 + 384.0)))
                    .map(|x| x.wrapping_shl(6));
                let g: U64 = cast::<u64, _>(&to_fixed(g.map(|x| x * 510.0 + 384.0)))
                    .map(|x| x.wrapping_shl(22));
                let b: U64 = cast::<u64, _>(&to_fixed(b.map(|x| x * 510.0 + 384.0)))
                    .map(|x| x.wrapping_shl(38));
                let a: U64 = cast::<u64, _>(&to_fixed(a.map(|x| x * 510.0 + 384.0)))
                    .map(|x| x.wrapping_shl(54));
                let v: U64 = std::array::from_fn(|j| r[j] | g[j] | b[j] | a[j]);
                store(dst.offset(8 * i as isize), &v);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1362-1367
        // FINAL_STAGE uses the same actual wrapper/kernel protocol in non-musttail configuration.
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_store_1010102,
            Exec_store_1010102_k,
            |src, dst, r, g, b, a, i| {
                let r: U32 =
                    cast::<u32, _>(&to_fixed(r.map(|x| x * 1023.0))).map(|x| x.wrapping_shl(0));
                let g: U32 =
                    cast::<u32, _>(&to_fixed(g.map(|x| x * 1023.0))).map(|x| x.wrapping_shl(10));
                let b: U32 =
                    cast::<u32, _>(&to_fixed(b.map(|x| x * 1023.0))).map(|x| x.wrapping_shl(20));
                let a: U32 =
                    cast::<u32, _>(&to_fixed(a.map(|x| x * 3.0))).map(|x| x.wrapping_shl(30));
                let v: U32 = std::array::from_fn(|j| r[j] | g[j] | b[j] | a[j]);
                store(dst.offset(4 * i as isize), &v);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1369-1386
        // FINAL_STAGE uses the same actual wrapper/kernel protocol in non-musttail configuration.
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_store_161616LE,
            Exec_store_161616LE_k,
            |src, dst, r, g, b, a, i| {
                let ptr = dst.offset(6 * i as isize);
                debug_assert_eq!(ptr as usize & 1, 0);
                let rgb = ptr.cast::<u16>();
                store_3(rgb, &U16_from_F(*r));
                store_3(rgb.add(1), &U16_from_F(*g));
                store_3(rgb.add(2), &U16_from_F(*b));
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1388-1407
        // FINAL_STAGE uses the same actual wrapper/kernel protocol in non-musttail configuration.
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_store_16161616LE,
            Exec_store_16161616LE_k,
            |src, dst, r, g, b, a, i| {
                let ptr = dst.offset(8 * i as isize);
                debug_assert_eq!(ptr as usize & 1, 0);
                let rr = to_fixed(r.map(|v| v * 65535.0));
                let gg = to_fixed(g.map(|v| v * 65535.0));
                let bb = to_fixed(b.map(|v| v * 65535.0));
                let aa = to_fixed(a.map(|v| v * 65535.0));
                let px: U64 = std::array::from_fn(|j| {
                    (rr[j] as u64)
                        | ((gg[j] as u64) << 16)
                        | ((bb[j] as u64) << 32)
                        | ((aa[j] as u64) << 48)
                });
                store(ptr, &px);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1409-1429
        // FINAL_STAGE uses the same actual wrapper/kernel protocol in non-musttail configuration.
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_store_161616BE,
            Exec_store_161616BE_k,
            |src, dst, r, g, b, a, i| {
                let ptr = dst.offset(6 * i as isize);
                debug_assert_eq!(ptr as usize & 1, 0);
                let rgb = ptr.cast::<u16>();
                let rr = to_fixed(r.map(|v| v * 65535.0));
                let gg = to_fixed(g.map(|v| v * 65535.0));
                let bb = to_fixed(b.map(|v| v * 65535.0));
                store_3(
                    rgb,
                    &rr.map(|v| (((v & 0xff) << 8) | ((v & 0xff00) >> 8)) as u16),
                );
                store_3(
                    rgb.add(1),
                    &gg.map(|v| (((v & 0xff) << 8) | ((v & 0xff00) >> 8)) as u16),
                );
                store_3(
                    rgb.add(2),
                    &bb.map(|v| (((v & 0xff) << 8) | ((v & 0xff00) >> 8)) as u16),
                );
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1431-1450
        // FINAL_STAGE uses the same actual wrapper/kernel protocol in non-musttail configuration.
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_store_16161616BE,
            Exec_store_16161616BE_k,
            |src, dst, r, g, b, a, i| {
                let ptr = dst.offset(8 * i as isize);
                debug_assert_eq!(ptr as usize & 1, 0);
                let rr = to_fixed(r.map(|v| v * 65535.0));
                let gg = to_fixed(g.map(|v| v * 65535.0));
                let bb = to_fixed(b.map(|v| v * 65535.0));
                let aa = to_fixed(a.map(|v| v * 65535.0));
                let px: U64 = std::array::from_fn(|j| {
                    (rr[j] as u64)
                        | ((gg[j] as u64) << 16)
                        | ((bb[j] as u64) << 32)
                        | ((aa[j] as u64) << 48)
                });
                store(ptr, &swap_endian_16x4(&px));
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1452-1472
        // FINAL_STAGE uses the same actual wrapper/kernel protocol in non-musttail configuration.
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_store_hhh,
            Exec_store_hhh_k,
            |src, dst, r, g, b, a, i| {
                let ptr = dst.offset(6 * i as isize);
                debug_assert_eq!(ptr as usize & 1, 0);
                let rgb = ptr.cast::<u16>();
                let rr = Half_from_F(*r);
                let gg = Half_from_F(*g);
                let bb = Half_from_F(*b);
                store_3(rgb, &rr);
                store_3(rgb.add(1), &gg);
                store_3(rgb.add(2), &bb);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1474-1497
        // FINAL_STAGE uses the same actual wrapper/kernel protocol in non-musttail configuration.
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_store_hhhh,
            Exec_store_hhhh_k,
            |src, dst, r, g, b, a, i| {
                let ptr = dst.offset(8 * i as isize);
                debug_assert_eq!(ptr as usize & 1, 0);
                let rr = Half_from_F(*r);
                let gg = Half_from_F(*g);
                let bb = Half_from_F(*b);
                let aa = Half_from_F(*a);
                let px: U64 = std::array::from_fn(|j| {
                    (rr[j] as u64)
                        | ((gg[j] as u64) << 16)
                        | ((bb[j] as u64) << 32)
                        | ((aa[j] as u64) << 48)
                });
                store(ptr, &px);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1499-1515
        // FINAL_STAGE uses the same actual wrapper/kernel protocol in non-musttail configuration.
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_store_fff,
            Exec_store_fff_k,
            |src, dst, r, g, b, a, i| {
                let ptr = dst.offset(12 * i as isize);
                debug_assert_eq!(ptr as usize & 3, 0);
                let rgb = ptr.cast::<f32>();
                store_3(rgb, r);
                store_3(rgb.add(1), g);
                store_3(rgb.add(2), b);
            }
        );
        // Source: modules/skcms/src/Transform_inl.h:1517-1535
        // FINAL_STAGE uses the same actual wrapper/kernel protocol in non-musttail configuration.
        $crate::modules::skcms::src::Transform_inl::stage_noctx_template!(
            Exec_store_ffff,
            Exec_store_ffff_k,
            |src, dst, r, g, b, a, i| {
                let ptr = dst.offset(16 * i as isize);
                debug_assert_eq!(ptr as usize & 3, 0);
                let rgba = ptr.cast::<f32>();
                store_4(rgba, r);
                store_4(rgba.add(1), g);
                store_4(rgba.add(2), b);
                store_4(rgba.add(3), a);
            }
        );

        use $crate::modules::skcms::src::skcms_Transform::Op;
        // Source: modules/skcms/src/Transform_inl.h:1545-1558
        /// Caller provides a valid op/context stream ending in a final store stage, plus valid pixel leases.
        /// Each typed context must remain readable for the complete execution; registers are independent.
        unsafe fn exec_stages(
            mut ops: *const Op,
            mut contexts: *const *const std::ffi::c_void,
            src: *const u8,
            dst: *mut u8,
            i: i32,
        ) {
            let (mut r, mut g, mut b, mut a) = (F0, F0, F0, F1);
            loop {
                let op = unsafe { ops.read() };
                ops = unsafe { ops.add(1) };
                let ctx = Ctx {
                    fArg: unsafe { contexts.read() },
                };
                contexts = unsafe { contexts.add(1) };
                unsafe {
                    match op {
                        Op::load_a8 => {
                            Exec_load_a8(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::load_g8 => {
                            Exec_load_g8(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::load_ga88 => {
                            Exec_load_ga88(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::load_4444 => {
                            Exec_load_4444(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::load_565 => {
                            Exec_load_565(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::load_888 => {
                            Exec_load_888(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::load_8888 => {
                            Exec_load_8888(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::load_1010102 => {
                            Exec_load_1010102(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::load_101010x_XR => {
                            Exec_load_101010x_XR(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::load_10101010_XR => {
                            Exec_load_10101010_XR(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::load_161616LE => {
                            Exec_load_161616LE(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::load_16161616LE => {
                            Exec_load_16161616LE(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::load_161616BE => {
                            Exec_load_161616BE(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::load_16161616BE => {
                            Exec_load_16161616BE(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::load_hhh => {
                            Exec_load_hhh(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::load_hhhh => {
                            Exec_load_hhhh(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::load_fff => {
                            Exec_load_fff(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::load_ffff => {
                            Exec_load_ffff(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::swap_rb => {
                            Exec_swap_rb(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::clamp => {
                            Exec_clamp(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::invert => {
                            Exec_invert(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::force_opaque => {
                            Exec_force_opaque(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::premul => {
                            Exec_premul(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::unpremul => {
                            Exec_unpremul(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::matrix_3x3 => {
                            Exec_matrix_3x3(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::matrix_3x4 => {
                            Exec_matrix_3x4(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::lab_to_xyz => {
                            Exec_lab_to_xyz(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::xyz_to_lab => {
                            Exec_xyz_to_lab(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::gamma_r => {
                            Exec_gamma_r(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::gamma_g => {
                            Exec_gamma_g(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::gamma_b => {
                            Exec_gamma_b(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::gamma_a => {
                            Exec_gamma_a(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::gamma_rgb => {
                            Exec_gamma_rgb(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::tf_r => {
                            Exec_tf_r(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::tf_g => {
                            Exec_tf_g(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::tf_b => {
                            Exec_tf_b(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::tf_a => {
                            Exec_tf_a(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::tf_rgb => {
                            Exec_tf_rgb(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::pq_r => {
                            Exec_pq_r(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::pq_g => {
                            Exec_pq_g(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::pq_b => {
                            Exec_pq_b(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::pq_a => {
                            Exec_pq_a(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::pq_rgb => {
                            Exec_pq_rgb(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::hlg_r => {
                            Exec_hlg_r(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::hlg_g => {
                            Exec_hlg_g(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::hlg_b => {
                            Exec_hlg_b(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::hlg_a => {
                            Exec_hlg_a(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::hlg_rgb => {
                            Exec_hlg_rgb(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::hlg_ootf_scale => {
                            Exec_hlg_ootf_scale(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::hlginv_r => {
                            Exec_hlginv_r(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::hlginv_g => {
                            Exec_hlginv_g(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::hlginv_b => {
                            Exec_hlginv_b(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::hlginv_a => {
                            Exec_hlginv_a(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::hlginv_rgb => {
                            Exec_hlginv_rgb(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::hlginv_ootf_scale => {
                            Exec_hlginv_ootf_scale(
                                ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i,
                            );
                        }
                        Op::table_r => {
                            Exec_table_r(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::table_g => {
                            Exec_table_g(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::table_b => {
                            Exec_table_b(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::table_a => {
                            Exec_table_a(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::clut_A2B => {
                            Exec_clut_A2B(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::clut_B2A => {
                            Exec_clut_B2A(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                        }
                        Op::store_a8 => {
                            Exec_store_a8(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                            return;
                        }
                        Op::store_g8 => {
                            Exec_store_g8(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                            return;
                        }
                        Op::store_ga88 => {
                            Exec_store_ga88(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                            return;
                        }
                        Op::store_4444 => {
                            Exec_store_4444(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                            return;
                        }
                        Op::store_565 => {
                            Exec_store_565(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                            return;
                        }
                        Op::store_888 => {
                            Exec_store_888(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                            return;
                        }
                        Op::store_8888 => {
                            Exec_store_8888(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                            return;
                        }
                        Op::store_1010102 => {
                            Exec_store_1010102(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                            return;
                        }
                        Op::store_161616LE => {
                            Exec_store_161616LE(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                            return;
                        }
                        Op::store_16161616LE => {
                            Exec_store_16161616LE(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                            return;
                        }
                        Op::store_161616BE => {
                            Exec_store_161616BE(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                            return;
                        }
                        Op::store_16161616BE => {
                            Exec_store_16161616BE(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                            return;
                        }
                        Op::store_101010x_XR => {
                            Exec_store_101010x_XR(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                            return;
                        }
                        Op::store_10101010_XR => {
                            Exec_store_10101010_XR(
                                ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i,
                            );
                            return;
                        }
                        Op::store_hhh => {
                            Exec_store_hhh(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                            return;
                        }
                        Op::store_hhhh => {
                            Exec_store_hhhh(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                            return;
                        }
                        Op::store_fff => {
                            Exec_store_fff(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                            return;
                        }
                        Op::store_ffff => {
                            Exec_store_ffff(ctx, src, dst, &mut r, &mut g, &mut b, &mut a, i);
                            return;
                        }
                    }
                }
            }
        }
        // Source: modules/skcms/src/Transform_inl.h:1563-1565
        // Source: modules/skcms/src/Transform_inl.h:1582-1599
        /// Caller supplies a well-formed program terminating in a store and one valid context per op.
        /// Pixel input/output cover n pixels at their declared bpp, including native alignment contracts.
        /// Each bpp is at most16; program stages must agree with these real formats.
        /// Machine-SIMD instances additionally require the caller to establish their
        /// exact native CPU feature set before entry; the N=1 instance uses no SIMD.
        pub unsafe fn run_program(
            program: *const Op,
            contexts: *const *const std::ffi::c_void,
            _programSize: isize,
            src: *const u8,
            dst: *mut u8,
            mut n: i32,
            src_bpp: usize,
            dst_bpp: usize,
        ) {
            unsafe {
                let stages = program;
                let mut i = 0;
                // Native baseline N is the width of the selected four-lane register representation.
                let nlanes = F0.len() as i32;
                while n >= nlanes {
                    unsafe { exec_stages(stages, contexts, src, dst, i) };
                    i += nlanes;
                    n -= nlanes;
                }
                if n > 0 {
                    // Rust explicitly retains the alignment provided by the native ARM64 stack temporary.
                    #[repr(align(64))]
                    struct Tail([u8; 4 * 4 * $n]);
                    let mut tmp = Tail([0; 4 * 4 * $n]);
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            src.add(i as usize * src_bpp),
                            tmp.0.as_mut_ptr(),
                            n as usize * src_bpp,
                        );
                        exec_stages(stages, contexts, tmp.0.as_ptr(), tmp.0.as_mut_ptr(), 0);
                        std::ptr::copy_nonoverlapping(
                            tmp.0.as_ptr(),
                            dst.add(i as usize * dst_bpp),
                            n as usize * dst_bpp,
                        );
                    }
                }
            }
        }
    };
}
pub(crate) use instantiate_transform_common;

// Native baseline's portable include instantiates these actual scalar branches.
macro_rules! instantiate_transform_portable_backend {
    () => {
        // The common frame contains one scalar per channel, never N=4 registers.
        // Baseline's native public V<T>=T alias remains scalar. These single-element
        // payload arrays are only the Rust template's local register representation.
        $crate::modules::skcms::src::Transform_inl::instantiate_transform_common!(1);

        // Source: modules/skcms/src/Transform_inl.h:129-129
        /// Safety: native float-to-signed-int input range/finite precondition.
        pub unsafe fn to_fixed(f: F) -> U32 {
            f.map(|v| unsafe { (v + 0.5).to_int_unchecked::<i32>() as u32 })
        }
        // Source: modules/skcms/src/Transform_inl.h:162-171
        pub unsafe fn F_from_Half(half: U16) -> F {
            half.map(|half| {
                let wide = half as u32;
                let s = wide & 0x8000;
                let em = wide ^ s;
                let norm = f32::from_bits(
                    (s << 16)
                        .wrapping_add(em << 13)
                        .wrapping_add((127 - 15) << 23),
                );
                // Original fallback flushes all denormal halves to positive zero.
                if em < 0x0400 {
                    0.0
                } else {
                    norm
                }
            })
        }
        // Source: modules/skcms/src/Transform_inl.h:189-196
        pub unsafe fn Half_from_F(f: F) -> U16 {
            f.map(|v| {
                let sem = v.to_bits();
                let s = sem & 0x80000000;
                let em = sem ^ s;
                // N=1 macro does not evaluate the unselected unsigned subtraction.
                if em < 0x38800000 {
                    0
                } else {
                    ((s >> 16)
                        .wrapping_add(em >> 13)
                        .wrapping_sub((127 - 15) << 10)) as u16
                }
            })
        }
        // Source: modules/skcms/src/Transform_inl.h:218-219
        pub unsafe fn min_(x: F, y: F) -> F {
            std::array::from_fn(|j| if x[j] > y[j] { y[j] } else { x[j] })
        }
        pub unsafe fn max_(x: F, y: F) -> F {
            std::array::from_fn(|j| if x[j] < y[j] { y[j] } else { x[j] })
        }
        // Source: modules/skcms/src/Transform_inl.h:222-224
        pub unsafe fn floor_(x: F) -> F {
            x.map(crate::modules::skcms::src::skcms_internals::floorf_)
        }
        // Source: modules/skcms/src/Transform_inl.h:435-438
        pub unsafe fn gather_8(p: *const u8, ix: I32) -> U8 {
            [unsafe { *p.offset(ix[0] as isize) }]
        }
        // Source: modules/skcms/src/Transform_inl.h:452-458
        pub unsafe fn gather_16(p: *const u8, ix: I32) -> U16 {
            [unsafe { load(p.offset(2 * ix[0] as isize)) }]
        }
        // Source: modules/skcms/src/Transform_inl.h:473-479
        pub unsafe fn gather_32(p: *const u8, ix: I32) -> U32 {
            [unsafe { load(p.offset(4 * ix[0] as isize)) }]
        }
        // Source: modules/skcms/src/Transform_inl.h:495-504
        // Source: modules/skcms/src/Transform_inl.h:529-530
        /// Safety: preserve native fourth overread byte for the indexed RGB entry.
        pub unsafe fn gather_24(p: *const u8, ix: I32) -> U32 {
            [unsafe { load::<u32, _>(p.offset(3 * ix[0] as isize)) } & 0x00ffffff]
        }
        // Source: modules/skcms/src/Transform_inl.h:533-541
        // Source: modules/skcms/src/Transform_inl.h:585-585
        /// Safety: preserve native two-byte overread for the indexed RGB16 entry.
        #[cfg(not(target_arch = "arm"))]
        pub unsafe fn gather_48(p: *const u8, ix: I32, v: &mut U64) {
            *v = [unsafe { load::<u64, _>(p.offset(6 * ix[0] as isize)) } & 0x0000ffffffffffff];
        }
    };
}
pub(crate) use instantiate_transform_portable_backend;

// Source: modules/skcms/src/Transform_inl.h:129-586
// Native header inclusion instantiates these real ISA-selected helpers in the
// caller's namespace. Bodies remain here, in the one target for this header.
// Register arrays are language payloads; every selected hardware operation uses
// the same real x86 instruction family, without a narrower program fallback.
macro_rules! instantiate_transform_wide_backend {
    (hsw) => {
        use core::arch::x86_64::*;
        // Source: modules/skcms/src/Transform_inl.h:129
        #[target_feature(enable = "avx,f16c,avx2")]
        pub unsafe fn to_fixed(value: F) -> U32 {
            let v = unsafe { _mm256_loadu_ps(value.as_ptr()) };
            let v = unsafe { _mm256_cvttps_epi32(_mm256_add_ps(v, _mm256_set1_ps(0.5))) };
            let mut out = [0; N];
            unsafe { _mm256_storeu_si256(out.as_mut_ptr().cast(), v) };
            out
        }
        // Source: modules/skcms/src/Transform_inl.h:148,152-159,173
        #[target_feature(enable = "avx,f16c,avx2")]
        pub unsafe fn F_from_Half(half: U16) -> F {
            let bits = unsafe { _mm_loadu_si128(half.as_ptr().cast()) };
            let v = unsafe { _mm256_cvtph_ps(bits) };
            let mut out = [0.0; N];
            unsafe { _mm256_storeu_ps(out.as_mut_ptr(), v) };
            out
        }
        // Source: modules/skcms/src/Transform_inl.h:180,184-186,197
        #[target_feature(enable = "avx,f16c,avx2")]
        pub unsafe fn Half_from_F(value: F) -> U16 {
            let value = core::hint::black_box(value);
            let v = unsafe { _mm256_loadu_ps(value.as_ptr()) };
            let bits = unsafe { _mm256_cvtps_ph::<4>(v) };
            let mut out = [0; N];
            unsafe { _mm_storeu_si128(out.as_mut_ptr().cast(), bits) };
            out
        }
        // Source: modules/skcms/src/Transform_inl.h:218
        #[target_feature(enable = "avx,f16c,avx2")]
        pub unsafe fn min_(x: F, y: F) -> F {
            let x = unsafe { _mm256_loadu_ps(x.as_ptr()) };
            let y = unsafe { _mm256_loadu_ps(y.as_ptr()) };
            let v = unsafe { _mm256_blendv_ps(x, y, _mm256_cmp_ps::<30>(x, y)) };
            let mut out = [0.0; N];
            unsafe { _mm256_storeu_ps(out.as_mut_ptr(), v) };
            out
        }
        // Source: modules/skcms/src/Transform_inl.h:219
        #[target_feature(enable = "avx,f16c,avx2")]
        pub unsafe fn max_(x: F, y: F) -> F {
            let x = unsafe { _mm256_loadu_ps(x.as_ptr()) };
            let y = unsafe { _mm256_loadu_ps(y.as_ptr()) };
            let v = unsafe { _mm256_blendv_ps(x, y, _mm256_cmp_ps::<17>(x, y)) };
            let mut out = [0.0; N];
            unsafe { _mm256_storeu_ps(out.as_mut_ptr(), v) };
            out
        }
        // Source: modules/skcms/src/Transform_inl.h:222-234,248
        #[target_feature(enable = "avx,f16c,avx2")]
        pub unsafe fn floor_(value: F) -> F {
            let x = unsafe { _mm256_loadu_ps(value.as_ptr()) };
            let v = unsafe { _mm256_round_ps::<1>(x) };
            let mut out = [0.0; N];
            unsafe { _mm256_storeu_ps(out.as_mut_ptr(), v) };
            out
        }
        // Source: modules/skcms/src/Transform_inl.h:435-450
        pub unsafe fn gather_8(p: *const u8, ix: I32) -> U8 {
            ix.map(|i| unsafe { p.offset(i as isize).read() })
        }
        // Source: modules/skcms/src/Transform_inl.h:452-471
        pub unsafe fn gather_16(p: *const u8, ix: I32) -> U16 {
            ix.map(|i| unsafe { p.offset(2 * i as isize).cast::<u16>().read_unaligned() })
        }
        // Source: modules/skcms/src/Transform_inl.h:473-493
        pub unsafe fn gather_32(p: *const u8, ix: I32) -> U32 {
            ix.map(|i| unsafe { p.offset(4 * i as isize).cast::<u32>().read_unaligned() })
        }
        // Source: modules/skcms/src/Transform_inl.h:495-531
        #[target_feature(enable = "avx,f16c,avx2")]
        pub unsafe fn gather_24(p: *const u8, ix: I32) -> U32 {
            let ix = unsafe { _mm256_loadu_si256(ix.as_ptr().cast()) };
            let offsets = unsafe { _mm256_mullo_epi32(ix, _mm256_set1_epi32(3)) };
            let v = unsafe {
                _mm256_mask_i32gather_epi32::<1>(
                    _mm256_setzero_si256(),
                    p.cast(),
                    offsets,
                    _mm256_set1_epi32(-1),
                )
            };
            let v = unsafe { _mm256_and_si256(v, _mm256_set1_epi32(0x00ffffff)) };
            let mut out = [0; N];
            unsafe { _mm256_storeu_si256(out.as_mut_ptr().cast(), v) };
            out
        }
        // Source: modules/skcms/src/Transform_inl.h:534-586
        #[target_feature(enable = "avx,f16c,avx2")]
        unsafe fn gather_48_value(p: *const u8, ix: I32) -> U64 {
            let ix = unsafe { _mm256_loadu_si256(ix.as_ptr().cast()) };
            let offsets = unsafe { _mm256_mullo_epi32(ix, _mm256_set1_epi32(6)) };
            let lo = unsafe { _mm256_castsi256_si128(offsets) };
            let hi = unsafe { _mm256_extracti128_si256::<1>(offsets) };
            let zero = unsafe { _mm256_setzero_si256() };
            let mask = unsafe { _mm256_set1_epi64x(-1) };
            let lo = unsafe { _mm256_mask_i32gather_epi64::<1>(zero, p.cast(), lo, mask) };
            let hi = unsafe { _mm256_mask_i32gather_epi64::<1>(zero, p.cast(), hi, mask) };
            let bits = unsafe { _mm256_set1_epi64x(0x0000ffffffffffff) };
            let mut out = [0; N];
            unsafe {
                _mm256_storeu_si256(out.as_mut_ptr().cast(), _mm256_and_si256(lo, bits));
                _mm256_storeu_si256(out.as_mut_ptr().add(4).cast(), _mm256_and_si256(hi, bits));
            }
            out
        }
        // Source: modules/skcms/src/Transform_inl.h:534-586
        #[target_feature(enable = "avx,f16c,avx2")]
        pub unsafe fn gather_48(p: *const u8, ix: I32, out: &mut U64) {
            *out = unsafe { gather_48_value(p, ix) };
        }
        $crate::modules::skcms::src::Transform_inl::instantiate_transform_common!(8);
    };
    (skx) => {
        use core::arch::x86_64::*;
        // Source: modules/skcms/src/Transform_inl.h:129
        #[target_feature(enable = "avx512f,avx512dq")]
        pub unsafe fn to_fixed(value: F) -> U32 {
            let v = unsafe { _mm512_loadu_ps(value.as_ptr()) };
            let v = unsafe { _mm512_cvttps_epi32(_mm512_add_ps(v, _mm512_set1_ps(0.5))) };
            let mut out = [0; N];
            unsafe { _mm512_storeu_si512(out.as_mut_ptr().cast(), v) };
            out
        }
        // Source: modules/skcms/src/Transform_inl.h:148,152-159,173
        #[target_feature(enable = "avx512f,avx512dq")]
        pub unsafe fn F_from_Half(half: U16) -> F {
            let bits = unsafe { _mm256_loadu_si256(half.as_ptr().cast()) };
            let v = unsafe { _mm512_cvtph_ps(bits) };
            let mut out = [0.0; N];
            unsafe { _mm512_storeu_ps(out.as_mut_ptr(), v) };
            out
        }
        // Source: modules/skcms/src/Transform_inl.h:180,184-186,197
        #[target_feature(enable = "avx512f,avx512dq")]
        pub unsafe fn Half_from_F(value: F) -> U16 {
            let value = core::hint::black_box(value);
            let v = unsafe { _mm512_loadu_ps(value.as_ptr()) };
            let bits = unsafe { _mm512_cvtps_ph::<4>(v) };
            let mut out = [0; N];
            unsafe { _mm256_storeu_si256(out.as_mut_ptr().cast(), bits) };
            out
        }
        // Source: modules/skcms/src/Transform_inl.h:218
        #[target_feature(enable = "avx512f,avx512dq")]
        pub unsafe fn min_(x: F, y: F) -> F {
            let x = unsafe { _mm512_loadu_ps(x.as_ptr()) };
            let y = unsafe { _mm512_loadu_ps(y.as_ptr()) };
            let v = unsafe { _mm512_mask_blend_ps(_mm512_cmp_ps_mask::<30>(x, y), x, y) };
            let mut out = [0.0; N];
            unsafe { _mm512_storeu_ps(out.as_mut_ptr(), v) };
            out
        }
        // Source: modules/skcms/src/Transform_inl.h:219
        #[target_feature(enable = "avx512f,avx512dq")]
        pub unsafe fn max_(x: F, y: F) -> F {
            let x = unsafe { _mm512_loadu_ps(x.as_ptr()) };
            let y = unsafe { _mm512_loadu_ps(y.as_ptr()) };
            let v = unsafe { _mm512_mask_blend_ps(_mm512_cmp_ps_mask::<17>(x, y), x, y) };
            let mut out = [0.0; N];
            unsafe { _mm512_storeu_ps(out.as_mut_ptr(), v) };
            out
        }
        // Source: modules/skcms/src/Transform_inl.h:222-234,248
        #[target_feature(enable = "avx512f,avx512dq")]
        pub unsafe fn floor_(value: F) -> F {
            let x = unsafe { _mm512_loadu_ps(value.as_ptr()) };
            let v = unsafe { _mm512_mask_roundscale_ps::<1>(x, 0xffff, x) };
            let mut out = [0.0; N];
            unsafe { _mm512_storeu_ps(out.as_mut_ptr(), v) };
            out
        }
        // Source: modules/skcms/src/Transform_inl.h:435-450
        pub unsafe fn gather_8(p: *const u8, ix: I32) -> U8 {
            ix.map(|i| unsafe { p.offset(i as isize).read() })
        }
        // Source: modules/skcms/src/Transform_inl.h:452-471
        pub unsafe fn gather_16(p: *const u8, ix: I32) -> U16 {
            ix.map(|i| unsafe { p.offset(2 * i as isize).cast::<u16>().read_unaligned() })
        }
        // Source: modules/skcms/src/Transform_inl.h:473-493
        pub unsafe fn gather_32(p: *const u8, ix: I32) -> U32 {
            ix.map(|i| unsafe { p.offset(4 * i as isize).cast::<u32>().read_unaligned() })
        }
        // Source: modules/skcms/src/Transform_inl.h:495-531
        #[target_feature(enable = "avx512f,avx512dq")]
        pub unsafe fn gather_24(p: *const u8, ix: I32) -> U32 {
            let ix = unsafe { _mm512_loadu_si512(ix.as_ptr().cast()) };
            let offsets = unsafe { _mm512_mullo_epi32(ix, _mm512_set1_epi32(3)) };
            let v = unsafe { _mm512_i32gather_epi32::<1>(offsets, p.cast()) };
            let v = unsafe { _mm512_and_si512(v, _mm512_set1_epi32(0x00ffffff)) };
            let mut out = [0; N];
            unsafe { _mm512_storeu_si512(out.as_mut_ptr().cast(), v) };
            out
        }
        // Source: modules/skcms/src/Transform_inl.h:534-586
        #[target_feature(enable = "avx512f,avx512dq")]
        unsafe fn gather_48_value(p: *const u8, ix: I32) -> U64 {
            let ix = unsafe { _mm512_loadu_si512(ix.as_ptr().cast()) };
            let offsets = unsafe { _mm512_mullo_epi32(ix, _mm512_set1_epi32(6)) };
            let lo = unsafe {
                _mm512_i32gather_epi64::<1>(_mm512_extracti32x8_epi32::<0>(offsets), p.cast())
            };
            let hi = unsafe {
                _mm512_i32gather_epi64::<1>(_mm512_extracti32x8_epi32::<1>(offsets), p.cast())
            };
            let bits = unsafe { _mm512_set1_epi64(0x0000ffffffffffff) };
            let mut out = [0; N];
            unsafe {
                _mm512_storeu_si512(out.as_mut_ptr().cast(), _mm512_and_si512(lo, bits));
                _mm512_storeu_si512(out.as_mut_ptr().add(8).cast(), _mm512_and_si512(hi, bits));
            }
            out
        }
        // Source: modules/skcms/src/Transform_inl.h:534-586
        #[target_feature(enable = "avx512f,avx512dq")]
        pub unsafe fn gather_48(p: *const u8, ix: I32, out: &mut U64) {
            *out = unsafe { gather_48_value(p, ix) };
        }
        $crate::modules::skcms::src::Transform_inl::instantiate_transform_common!(16);
    };
}
pub(crate) use instantiate_transform_wide_backend;
