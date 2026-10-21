// Copyright 2017 Google LLC. BSD-style license.
// Native portable 16-byte store width; AVX 32-byte dispatch remains pending.
// Associated native Vec storage selection and usize caller counts are Rust adapters.
use crate::src::core::SkVx::WideLane;
// Source: src/opts/SkMemset_opts.h:16-23
pub fn memsetT<T: WideLane>(buffer: &mut [T], value: T, count: usize) {
    assert!(count <= buffer.len());
    unsafe {
        memsetT_raw(buffer.as_mut_ptr(), value, count);
    }
}
// Source: src/opts/SkMemset_opts.h:38-46
pub fn memset16(buffer: &mut [u16], value: u16, count: usize) {
    memsetT(buffer, value, count);
}
pub fn memset32(buffer: &mut [u32], value: u32, count: usize) {
    memsetT(buffer, value, count);
}
pub fn memset64(buffer: &mut [u64], value: u64, count: usize) {
    memsetT(buffer, value, count);
}
// Source: src/opts/SkMemset_opts.h:48-55
pub fn rect_memsetT<T: WideLane>(
    buffer: &mut [T],
    value: T,
    count: usize,
    row_bytes: usize,
    mut height: usize,
) {
    assert_eq!(row_bytes % std::mem::size_of::<T>(), 0);
    let stride = row_bytes / std::mem::size_of::<T>();
    let mut offset = 0;
    while height > 0 {
        memsetT(&mut buffer[offset..], value, count);
        offset += stride;
        height -= 1;
    }
}
// Source: src/opts/SkMemset_opts.h:57-68
pub fn rect_memset16(
    buffer: &mut [u16],
    value: u16,
    count: usize,
    row_bytes: usize,
    height: usize,
) {
    rect_memsetT(buffer, value, count, row_bytes, height);
}
pub fn rect_memset32(
    buffer: &mut [u32],
    value: u32,
    count: usize,
    row_bytes: usize,
    height: usize,
) {
    rect_memsetT(buffer, value, count, row_bytes, height);
}
pub fn rect_memset64(
    buffer: &mut [u64],
    value: u64,
    count: usize,
    row_bytes: usize,
    height: usize,
) {
    rect_memsetT(buffer, value, count, row_bytes, height);
}
// Native VecSize=16/sizeof(T) is selected by the actual typed lane provider.
// Providers cover all current callers: u8/u16/u32/u64 and SkColor4f.
/// Safety: count initialized writable T elements are exclusively leased; count
/// fits the original nonnegative native int domain, no wide-value alias exists.
pub unsafe fn memsetT_raw<T: WideLane>(buffer: *mut T, value: T, count: usize) {
    memset_wide(buffer, value, count);
}
// Source: src/opts/SkMemset_opts.h:25-36
unsafe fn memset_wide<T: WideLane>(mut buffer: *mut T, value: T, mut count: usize) {
    let wide_value = T::splat_wide(value);
    while count >= T::WIDTH {
        T::store_wide(&wide_value, buffer);
        buffer = buffer.add(T::WIDTH);
        count -= T::WIDTH;
    }
    while count > 0 {
        buffer.write_unaligned(value);
        buffer = buffer.add(1);
        count -= 1;
    }
}
// Source: src/opts/SkMemset_opts.h:48-55
/// Safety: each row must cover count writable T values for height rows at the
/// supplied byte pitch. Rows may be padded; no slice over gaps is constructed.
pub unsafe fn rect_memsetT_raw<T: WideLane>(
    mut buffer: *mut T,
    value: T,
    count: usize,
    row_bytes: usize,
    mut height: usize,
) {
    while height > 0 {
        memsetT_raw(buffer, value, count);
        buffer = buffer.cast::<u8>().wrapping_add(row_bytes).cast();
        height -= 1;
    }
}
// Source: src/opts/SkMemset_opts.h:57-68
/// Safety: same byte-pitched writable-storage contract as rect_memsetT_raw.
pub unsafe fn rect_memset32_raw(
    buffer: *mut u32,
    value: u32,
    count: usize,
    row_bytes: usize,
    height: usize,
) {
    rect_memsetT_raw(buffer, value, count, row_bytes, height);
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod avx {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::*;
    // Source: src/opts/SkMemset_opts.h:16-46
    #[target_feature(enable = "avx")]
    pub unsafe fn memset16(mut p: *mut u16, v: u16, mut n: i32) {
        let wide = _mm256_set1_ps(f32::from_bits((v as u32) | ((v as u32) << 16)));
        while n >= 16 {
            _mm256_storeu_ps(p.cast(), wide);
            p = p.add(16);
            n -= 16;
        }
        while n > 0 {
            p.write_unaligned(v);
            p = p.add(1);
            n -= 1;
        }
    }
    // Source: src/opts/SkMemset_opts.h:48-68
    #[target_feature(enable = "avx")]
    pub unsafe fn rect_memset16(
        mut p: *mut u16,
        v: u16,
        n: i32,
        row_bytes: usize,
        mut height: i32,
    ) {
        while height > 0 {
            memset16(p, v, n);
            p = p.cast::<u8>().wrapping_add(row_bytes).cast();
            height -= 1;
        }
    }
    // Source: src/opts/SkMemset_opts.h:16-46
    #[target_feature(enable = "avx")]
    pub unsafe fn memset32(mut p: *mut u32, v: u32, mut n: i32) {
        let wide = _mm256_set1_ps(f32::from_bits(v));
        while n >= 8 {
            _mm256_storeu_ps(p.cast(), wide);
            p = p.add(8);
            n -= 8;
        }
        while n > 0 {
            p.write_unaligned(v);
            p = p.add(1);
            n -= 1;
        }
    }
    // Source: src/opts/SkMemset_opts.h:48-68
    #[target_feature(enable = "avx")]
    pub unsafe fn rect_memset32(
        mut p: *mut u32,
        v: u32,
        n: i32,
        row_bytes: usize,
        mut height: i32,
    ) {
        while height > 0 {
            memset32(p, v, n);
            p = p.cast::<u8>().wrapping_add(row_bytes).cast();
            height -= 1;
        }
    }
    // Source: src/opts/SkMemset_opts.h:16-46
    #[target_feature(enable = "avx")]
    pub unsafe fn memset64(mut p: *mut u64, v: u64, mut n: i32) {
        let wide = _mm256_broadcast_sd(&f64::from_bits(v));
        while n >= 4 {
            _mm256_storeu_pd(p.cast(), wide);
            p = p.add(4);
            n -= 4;
        }
        while n > 0 {
            p.write_unaligned(v);
            p = p.add(1);
            n -= 1;
        }
    }
    // Source: src/opts/SkMemset_opts.h:48-68
    #[target_feature(enable = "avx")]
    pub unsafe fn rect_memset64(
        mut p: *mut u64,
        v: u64,
        n: i32,
        row_bytes: usize,
        mut height: i32,
    ) {
        while height > 0 {
            memset64(p, v, n);
            p = p.cast::<u8>().wrapping_add(row_bytes).cast();
            height -= 1;
        }
    }
}
