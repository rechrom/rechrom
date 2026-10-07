//! src/opts/SkMemset_opts.h: memsetT<uint32_t> / rect_memset32.
//! ARM uses the original 16-byte vector width, with a scalar pixel tail.

pub fn rect_memset32(
    buffer: &mut [u32],
    value: u32,
    count: usize,
    row_stride: usize,
    height: usize,
) {
    if count == 0 || height == 0 {
        return;
    }
    let required = (height - 1)
        .checked_mul(row_stride)
        .and_then(|last| last.checked_add(count))
        .expect("valid rectangle extent");
    assert!(count <= row_stride && required <= buffer.len());
    #[cfg(target_arch = "aarch64")]
    unsafe {
        use core::arch::aarch64::*;
        // SAFETY: the complete rectangle was validated above. All vector
        // stores cover four initialized pixels within the current real row.
        let wide_value = vdupq_n_u32(value);
        let mut row = buffer.as_mut_ptr();
        for y in 0..height {
            let mut at = row;
            let mut remaining = count;
            while remaining >= 4 {
                vst1q_u32(at, wide_value);
                at = at.add(4);
                remaining -= 4;
            }
            while remaining > 0 {
                *at = value;
                at = at.add(1);
                remaining -= 1;
            }
            // A cropped final row need not include its trailing padding.
            if y + 1 < height {
                row = row.add(row_stride);
            }
        }
    }
    #[cfg(not(target_arch = "aarch64"))]
    for y in 0..height {
        buffer[y * row_stride..y * row_stride + count].fill(value);
    }
}
