// Source: src/core/SkRectMemcpy.h:16-23
// Source: src/core/SkRectMemcpy.h:25-30
/// Source/destination regions must be valid, nonoverlapping initialized/writable byte ranges for all rows.
pub unsafe fn SkRectMemcpy(
    mut dst: *mut u8,
    dstRB: usize,
    mut src: *const u8,
    srcRB: usize,
    trimRowBytes: usize,
    rowCount: i32,
) {
    crate::SkASSERT!(@source b"src/core/SkRectMemcpy.h\0".as_ptr(),18; @condition b"trimRowBytes <= dstRB\0".as_ptr(); trimRowBytes <= dstRB);
    crate::SkASSERT!(@source b"src/core/SkRectMemcpy.h\0".as_ptr(),19; @condition b"trimRowBytes <= srcRB\0".as_ptr(); trimRowBytes <= srcRB);
    if trimRowBytes == dstRB && trimRowBytes == srcRB {
        std::ptr::copy_nonoverlapping(src, dst, trimRowBytes.wrapping_mul(rowCount as usize));
        return;
    }
    for _i in 0..rowCount {
        std::ptr::copy_nonoverlapping(src, dst, trimRowBytes);
        dst = dst.wrapping_add(dstRB);
        src = src.wrapping_add(srcRB);
    }
}
