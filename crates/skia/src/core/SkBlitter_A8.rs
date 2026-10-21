// Source: src/core/SkBlitter_A8.h:27-55
// Concrete native virtual declarations below are expressed by the SkBlitter
// implementation; native factories return explicit Result at Rust's boundary.
// Copyright 2006 The Android Open Source Project. BSD-style license.
use crate::src::core::SkBlendMode::SkBlendMode;
use crate::src::core::SkBlitter::{SkBlitter, SkBlitterStorage};
use crate::src::core::SkPixmap::SkPixmap;

// Source: src/core/SkBlitter_A8.h:27-38
// Composition retains the base-owned scratch allocation. Constructors borrow the canonical SkPaint and its optional trait getters.
pub struct SkA8_Coverage_Blitter {
    fDevice: SkPixmap,
    base: SkBlitterStorage,
}
impl SkA8_Coverage_Blitter {
    // Source: src/core/SkBlitter_A8.cpp:24-29
    /// Safety: device's pixel storage must remain writable and live for every
    /// call made through this blitter, as with the original non-owning Pixmap.
    pub unsafe fn new(device: &SkPixmap, paint: &crate::src::core::SkPaint::SkPaint) -> Self {
        crate::SkASSERT!(@source b"src/core/SkBlitter_A8.cpp\0".as_ptr(), 27; @condition b"nullptr == paint.getShader()\0".as_ptr(); paint.getShader().is_none());
        crate::SkASSERT!(@source b"src/core/SkBlitter_A8.cpp\0".as_ptr(), 28; @condition b"nullptr == paint.getColorFilter()\0".as_ptr(); paint.getColorFilter().is_none());
        Self {
            fDevice: device.clone(),
            base: SkBlitterStorage::new(),
        }
    }
}
// Safety: scratch storage delegates to the canonical aligned owner; primitive
// methods consume sparse runs synchronously and retain no borrowed row buffers.
unsafe impl SkBlitter for SkA8_Coverage_Blitter {
    fn allocBlitMemory(self: core::pin::Pin<&mut Self>, sz: usize) -> *mut u8 {
        let this = unsafe { self.get_unchecked_mut() };
        this.base.allocBlitMemory(sz)
    }
    // Source: src/core/SkBlitter_A8.cpp:31-52
    unsafe fn blitAntiH(
        self: core::pin::Pin<&mut Self>,
        x: i32,
        y: i32,
        antialias: &mut [u8],
        runs: &mut [i16],
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        let mut device = this.fDevice.writable_addr8(x, y);
        #[cfg(sk_debug)]
        let mut total_count: i32 = 0;
        let mut offset = 0;
        loop {
            let count = runs[offset];
            crate::SkASSERT!(@source b"src/core/SkBlitter_A8.cpp\0".as_ptr(), 38; @condition b"count >= 0\0".as_ptr(); count >= 0);
            if count == 0 {
                return;
            }
            let count = count as usize;
            unsafe {
                if antialias[offset] != 0 {
                    device.write_bytes(antialias[offset], count);
                }
                device = device.add(count);
            }
            offset += count;
            #[cfg(sk_debug)]
            {
                total_count += count as i32;
            }
        }
        crate::SkASSERT!(@source b"src/core/SkBlitter_A8.cpp\0".as_ptr(), 51; @condition b"fDevice.width() == totalCount\0".as_ptr(); this.fDevice.width() == total_count);
    }
    // Source: src/core/SkBlitter_A8.cpp:54-56
    unsafe fn blitH(self: core::pin::Pin<&mut Self>, x: i32, y: i32, width: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.fDevice
                .writable_addr8(x, y)
                .write_bytes(255, width as usize);
        }
    }
    // Source: src/core/SkBlitter_A8.cpp:58-69
    unsafe fn blitV(self: core::pin::Pin<&mut Self>, x: i32, y: i32, mut height: i32, alpha: u8) {
        let this = unsafe { self.get_unchecked_mut() };
        if alpha == 0 {
            return;
        }
        let mut dst = this.fDevice.writable_addr8(x, y);
        let dstRB = this.fDevice.rowBytes();
        loop {
            height -= 1;
            if height < 0 {
                break;
            }
            unsafe {
                *dst = alpha;
                dst = dst.wrapping_add(dstRB);
            }
        }
    }
    // Source: src/core/SkBlitter_A8.cpp:71-78
    unsafe fn blitRect(
        self: core::pin::Pin<&mut Self>,
        x: i32,
        y: i32,
        width: i32,
        mut height: i32,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        let mut dst = this.fDevice.writable_addr8(x, y);
        let dstRB = this.fDevice.rowBytes();
        loop {
            height -= 1;
            if height < 0 {
                break;
            }
            unsafe {
                dst.write_bytes(255, width as usize);
                dst = dst.wrapping_add(dstRB);
            }
        }
    }
    // Source: src/core/SkBlitter_A8.cpp:80-101
    unsafe fn blitMask(
        self: core::pin::Pin<&mut Self>,
        mask: &crate::src::core::SkMask::SkMask,
        clip: &crate::src::core::SkRect::SkIRect,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        if mask.fFormat != crate::src::core::SkMask::Format::kA8_Format {
            crate::src::core::SkBlitter::blitMask(
                core::pin::Pin::new_unchecked(&mut *this),
                mask,
                clip,
            );
            return;
        }
        let x = clip.fLeft;
        let y = clip.fTop;
        let width = clip.width();
        let mut height = clip.height();
        let mut dst = this.fDevice.writable_addr8(x, y);
        let mut src = mask.getAddr8(x, y);
        let srcRB = mask.fRowBytes as usize;
        let dstRB = this.fDevice.rowBytes();
        while height > 0 {
            core::ptr::copy_nonoverlapping(src, dst, width as usize);
            dst = dst.wrapping_add(dstRB);
            src = src.wrapping_add(srcRB);
            height -= 1;
        }
    }
}

// Source: src/core/SkBlitter_A8.cpp:105-108
fn div255(prod: u32) -> u8 {
    crate::SkASSERT!(@source b"src/core/SkBlitter_A8.cpp\0".as_ptr(), 106; @condition b"prod <= 255*255\0".as_ptr(); prod <= 255 * 255);
    (((prod + 128) * 257) >> 16) as u8
}
// Source: src/core/SkBlitter_A8.cpp:110-112
fn u8_lerp(a: u8, b: u8, t: u8) -> u8 {
    div255((255 - t as u32) * a as u32 + t as u32 * b as u32)
}
// Source: src/core/SkBlitter_A8.cpp:114-114
type AlphaProc = fn(u8, u8) -> u8;
// Source: src/core/SkBlitter_A8.cpp:116-117
fn srcover_p(src: u8, dst: u8) -> u8 {
    src + div255((255 - src as u32) * dst as u32)
}
fn src_p(src: u8, _dst: u8) -> u8 {
    src
}
// Source: src/core/SkBlitter_A8.cpp:119-124
fn A8_row_bw(dst: &mut [u8], src: u8, n: i32, proc: AlphaProc) {
    for i in 0..n {
        dst[i as usize] = proc(src, dst[i as usize]);
    }
}
type A8_RowBlitBW = fn(&mut [u8], u8, i32);
// Source: src/core/SkBlitter_A8.cpp:126-139
fn A8_row_aa(dst: &mut [u8], mut src: u8, n: i32, aa: u8, proc: AlphaProc, canFoldAA: bool) {
    if canFoldAA {
        src = div255(src as u32 * aa as u32);
        for i in 0..n {
            dst[i as usize] = proc(src, dst[i as usize]);
        }
    } else {
        for i in 0..n {
            dst[i as usize] = u8_lerp(dst[i as usize], proc(src, dst[i as usize]), aa);
        }
    }
}
type A8_RowBlitAA = fn(&mut [u8], u8, i32, u8);
// Source: src/core/SkBlitter_A8.cpp:141-158
// Rust noncapturing closures replace the C++ WRAP_BLIT macro.
struct A8_RowBlitBWPair {
    mode: SkBlendMode,
    oneProc: AlphaProc,
    bwProc: A8_RowBlitBW,
    aaProc: A8_RowBlitAA,
}
static gA8_RowBlitPairs: [A8_RowBlitBWPair; 2] = [
    A8_RowBlitBWPair {
        mode: SkBlendMode::kSrcOver,
        oneProc: srcover_p,
        bwProc: |d, s, n| A8_row_bw(d, s, n, srcover_p),
        aaProc: |d, s, n, a| A8_row_aa(d, s, n, a, srcover_p, true),
    },
    A8_RowBlitBWPair {
        mode: SkBlendMode::kSrc,
        oneProc: src_p,
        bwProc: |d, s, n| A8_row_bw(d, s, n, src_p),
        aaProc: |d, s, n, a| A8_row_aa(d, s, n, a, src_p, false),
    },
];
// Source: src/core/SkBlitter_A8.cpp:160-167
fn find_a8_rowproc_pair(bm: SkBlendMode) -> Option<&'static A8_RowBlitBWPair> {
    for pair in &gA8_RowBlitPairs {
        if pair.mode == bm {
            return Some(pair);
        }
    }
    None
}

// Source: src/core/SkBlitter_A8.cpp:169-186
pub struct SkA8_Blitter {
    fDevice: SkPixmap,
    fOneProc: AlphaProc,
    fBWProc: A8_RowBlitBW,
    fAAProc: A8_RowBlitAA,
    fSrc: u8,
    base: SkBlitterStorage,
}
impl SkA8_Blitter {
    // Source: src/core/SkBlitter_A8.cpp:188-201
    /// Safety: device's backing storage must remain writable and live while
    /// this non-owning blitter is used.
    pub unsafe fn new(device: &SkPixmap, paint: &crate::src::core::SkPaint::SkPaint) -> Self {
        crate::SkASSERT!(@source b"src/core/SkBlitter_A8.cpp\0".as_ptr(), 190; @condition b"nullptr == paint.getShader()\0".as_ptr(); paint.getShader().is_none());
        crate::SkASSERT!(@source b"src/core/SkBlitter_A8.cpp\0".as_ptr(), 191; @condition b"nullptr == paint.getColorFilter()\0".as_ptr(); paint.getColorFilter().is_none());
        let mode = paint.asBlendMode();
        crate::SkASSERT!(@source b"src/core/SkBlitter_A8.cpp\0".as_ptr(), 193; @condition b"mode\0".as_ptr(); mode.is_some());
        let pair = find_a8_rowproc_pair(mode.unwrap());
        crate::SkASSERT!(@source b"src/core/SkBlitter_A8.cpp\0".as_ptr(), 195; @condition b"pair\0".as_ptr(); pair.is_some());
        let pair = pair.unwrap();
        Self {
            fDevice: device.clone(),
            fOneProc: pair.oneProc,
            fBWProc: pair.bwProc,
            fAAProc: pair.aaProc,
            fSrc: paint.getAlpha(),
            base: SkBlitterStorage::new(),
        }
    }
}
// Safety: scratch storage delegates to the canonical aligned owner; primitive
// methods consume sparse runs synchronously and retain no borrowed row buffers.
unsafe impl SkBlitter for SkA8_Blitter {
    fn allocBlitMemory(self: core::pin::Pin<&mut Self>, sz: usize) -> *mut u8 {
        let this = unsafe { self.get_unchecked_mut() };
        this.base.allocBlitMemory(sz)
    }
    // Source: src/core/SkBlitter_A8.cpp:203-227
    unsafe fn blitAntiH(
        self: core::pin::Pin<&mut Self>,
        x: i32,
        y: i32,
        antialias: &mut [u8],
        runs: &mut [i16],
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        let mut device = this.fDevice.writable_addr8(x, y);
        #[cfg(sk_debug)]
        let mut total_count: i32 = 0;
        let mut offset = 0;
        loop {
            let count = runs[offset];
            crate::SkASSERT!(@source b"src/core/SkBlitter_A8.cpp\0".as_ptr(), 209; @condition b"count >= 0\0".as_ptr(); count >= 0);
            if count == 0 {
                return;
            }
            let count = count as usize;
            unsafe {
                if antialias[offset] == 255 {
                    (this.fBWProc)(
                        core::slice::from_raw_parts_mut(device, count),
                        this.fSrc,
                        count as i32,
                    );
                } else if antialias[offset] != 0 {
                    (this.fAAProc)(
                        core::slice::from_raw_parts_mut(device, count),
                        this.fSrc,
                        count as i32,
                        antialias[offset],
                    );
                }
                device = device.add(count);
            }
            offset += count;
            #[cfg(sk_debug)]
            {
                total_count += count as i32;
            }
        }
        crate::SkASSERT!(@source b"src/core/SkBlitter_A8.cpp\0".as_ptr(), 226; @condition b"fDevice.width() == totalCount\0".as_ptr(); this.fDevice.width() == total_count);
    }
    // Source: src/core/SkBlitter_A8.cpp:229-231
    unsafe fn blitH(self: core::pin::Pin<&mut Self>, x: i32, y: i32, width: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            (this.fBWProc)(
                core::slice::from_raw_parts_mut(this.fDevice.writable_addr8(x, y), width as usize),
                this.fSrc,
                width,
            );
        }
    }
    // Source: src/core/SkBlitter_A8.cpp:233-248
    unsafe fn blitV(self: core::pin::Pin<&mut Self>, x: i32, y: i32, mut height: i32, aa: u8) {
        let this = unsafe { self.get_unchecked_mut() };
        let mut device = this.fDevice.writable_addr8(x, y);
        let dstRB = this.fDevice.rowBytes();
        if aa == 255 {
            loop {
                height -= 1;
                if height < 0 {
                    break;
                }
                unsafe {
                    *device = (this.fOneProc)(this.fSrc, *device);
                    device = device.wrapping_add(dstRB);
                }
            }
        } else if aa != 0 {
            loop {
                height -= 1;
                if height < 0 {
                    break;
                }
                unsafe {
                    (this.fAAProc)(core::slice::from_raw_parts_mut(device, 1), this.fSrc, 1, aa);
                    device = device.wrapping_add(dstRB);
                }
            }
        }
    }
    // Source: src/core/SkBlitter_A8.cpp:250-258
    unsafe fn blitRect(
        self: core::pin::Pin<&mut Self>,
        x: i32,
        y: i32,
        width: i32,
        mut height: i32,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        let mut device = this.fDevice.writable_addr8(x, y);
        let dstRB = this.fDevice.rowBytes();
        loop {
            height -= 1;
            if height < 0 {
                break;
            }
            unsafe {
                (this.fBWProc)(
                    core::slice::from_raw_parts_mut(device, width as usize),
                    this.fSrc,
                    width,
                );
                device = device.wrapping_add(dstRB);
            }
        }
    }
    // Source: src/core/SkBlitter_A8.cpp:260-283
    unsafe fn blitMask(
        self: core::pin::Pin<&mut Self>,
        mask: &crate::src::core::SkMask::SkMask,
        clip: &crate::src::core::SkRect::SkIRect,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        if mask.fFormat != crate::src::core::SkMask::Format::kA8_Format {
            crate::src::core::SkBlitter::blitMask(
                core::pin::Pin::new_unchecked(&mut *this),
                mask,
                clip,
            );
            return;
        }
        let x = clip.fLeft;
        let y = clip.fTop;
        let width = clip.width();
        let mut height = clip.height();
        let mut dst = this.fDevice.writable_addr8(x, y);
        let mut src = mask.getAddr8(x, y);
        let srcRB = mask.fRowBytes as usize;
        let dstRB = this.fDevice.rowBytes();
        while height > 0 {
            for i in 0..width as usize {
                *dst.add(i) = u8_lerp(
                    *dst.add(i),
                    (this.fOneProc)(this.fSrc, *dst.add(i)),
                    *src.add(i),
                );
            }
            dst = dst.wrapping_add(dstRB);
            src = src.wrapping_add(srcRB);
            height -= 1;
        }
    }
}

// Source: src/core/SkBlitter_A8.cpp:287-313
/// Safety: pixel storage must remain writable/live for all later blitter calls;
/// the returned pointer belongs to alloc and must not outlive/reset that arena.
pub unsafe fn SkChooseA8Blitter<'arena>(
    dst: &SkPixmap,
    _ctm: &crate::src::core::SkMatrix::SkMatrix,
    paint: &crate::src::core::SkPaint::SkPaint,
    alloc: core::pin::Pin<&'arena mut crate::src::core::SkArenaAlloc::SkArenaAlloc>,
    drawCoverage: crate::src::core::SkDrawTypes::SkDrawCoverage,
    clipShader: Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>>,
) -> Result<
    Option<core::ptr::NonNull<dyn SkBlitter + 'arena>>,
    crate::src::core::SkRasterPipeline::MissingStage,
> {
    if dst.colorType() != crate::include::core::SkColorType::SkColorType::kAlpha_8_SkColorType {
        return Ok(None);
    }
    if paint.getShader().is_some() || paint.getColorFilter().is_some() {
        return Ok(None);
    }
    if clipShader.is_some() {
        return Ok(None);
    }
    if drawCoverage == crate::src::core::SkDrawTypes::SkDrawCoverage::kYes {
        let ptr = core::pin::Pin::get_unchecked_mut(
            alloc.make_pinned(SkA8_Coverage_Blitter::new(dst, paint)),
        ) as *mut SkA8_Coverage_Blitter;
        return Ok(core::ptr::NonNull::new(ptr as *mut dyn SkBlitter));
    } else {
        if let Some(mode) = paint.asBlendMode() {
            if find_a8_rowproc_pair(mode).is_some() {
                let ptr = core::pin::Pin::get_unchecked_mut(
                    alloc.make_pinned(SkA8_Blitter::new(dst, paint)),
                ) as *mut SkA8_Blitter;
                return Ok(core::ptr::NonNull::new(ptr as *mut dyn SkBlitter));
            }
        }
    }
    Ok(None)
}
// Source: src/core/SkBlitter_A8.cpp:315-324
/// Safety: same backing-store and arena lifetime contract as SkChooseA8Blitter.
pub unsafe fn SkA8Blitter_Choose<'arena>(
    dst: &SkPixmap,
    ctm: &crate::src::core::SkMatrix::SkMatrix,
    paint: &crate::src::core::SkPaint::SkPaint,
    alloc: core::pin::Pin<&'arena mut crate::src::core::SkArenaAlloc::SkArenaAlloc>,
    coverage: crate::src::core::SkDrawTypes::SkDrawCoverage,
    clipShader: Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>>,
    _props: &crate::include::core::SkSurfaceProps::SkSurfaceProps,
    _bounds: &crate::src::core::SkRect::SkRect,
) -> Result<
    Option<core::ptr::NonNull<dyn SkBlitter + 'arena>>,
    crate::src::core::SkRasterPipeline::MissingStage,
> {
    SkChooseA8Blitter(dst, ctm, paint, alloc, coverage, clipShader)
}
