// Copyright 2008 Google Inc. BSD-style license.
use crate::include::core::SkTileMode::SkTileMode;
use crate::src::core::SkArenaAlloc::SkSTArenaAlloc;
use crate::src::core::SkMatrix::SkMatrix;
use crate::src::core::SkPixmap::SkPixmap;
use crate::src::core::SkScalar::{SkScalarNearlyEqual, SkScalarNearlyZero, SkScalarRoundToScalar};
use crate::src::image::SkImage_Base::SkImage_Base;
use std::mem::MaybeUninit;
impl SkBitmapProcState<'_> {
    // Source: src/core/SkBitmapProcState.h:32-35
    #[cfg(all(target_arch = "aarch64", target_pointer_width = "64"))]
    pub fn setup(
        mut self: std::pin::Pin<&mut Self>,
        inv: &SkMatrix,
        color: u32,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
    ) -> Result<bool, crate::RasterError> {
        Ok(self.as_mut().init(inv, color as u8, sampling)?
            && unsafe { self.get_unchecked_mut().chooseProcs() })
    }
}
// Slice-based proc boundaries require every u32 element in the passed scratch/
// output slice to be initialized before forming the Rust reference, even when
// native C++ would accept uninitialized output storage. Callers must either use
// initialized scratch (explicit uncredited adaptation), or an actual raw boundary.
// Proc callers preserve native count/output bounds, little-endian packed 16-bit
// coordinates, and readable pixel storage; padding never substitutes for samples.
// Source: src/core/SkBitmapProcState.h:37-37
pub type ShaderProc32 = unsafe fn(*const std::ffi::c_void, i32, i32, &mut [u32], i32);
// Source: src/core/SkBitmapProcState.h:39-42
pub type MatrixProc = for<'image> unsafe fn(&SkBitmapProcState<'image>, &mut [u32], i32, i32, i32);
// Source: src/core/SkBitmapProcState.h:44-47
pub type SampleProc32 = for<'image> unsafe fn(&SkBitmapProcState<'image>, &[u32], i32, &mut [u32]);
// Source: src/core/SkBitmapProcState.h:49-49
// Source: src/core/SkBitmapProcState.h:51-56
// Source: src/core/SkBitmapProcState.h:58-59
// Source: src/core/SkBitmapProcState.h:61-62
// Source: src/core/SkBitmapProcState.h:64-64
// One owned field group preserves header source order around inline methods.
// Deref below projects these same fields, never a copied or fabricated ready view.
// Native scalars without constructor initialization remain MaybeUninit until init/choose.
pub struct SkBitmapProcStateFields<'image> {
    pub fImage: Option<&'image dyn SkImage_Base>,
    pub fPixmap: SkPixmap,
    pub fInvMatrix: SkMatrix,
    pub fPaintAlpha: MaybeUninit<u8>,
    pub fTileModeX: SkTileMode,
    pub fTileModeY: SkTileMode,
    pub fBilerp: MaybeUninit<bool>,
    pub fInvSx: MaybeUninit<i64>,
    pub fInvKy: MaybeUninit<i64>,
    pub fFilterOneX: MaybeUninit<i32>,
    pub fFilterOneY: MaybeUninit<i32>,
    pub fAlphaScale: MaybeUninit<u16>,
}
impl SkBitmapProcState<'_> {
    // Source: src/core/SkBitmapProcState.h:79-79
    /// Safety: successful native setup/chooseProcs must have initialized the slot.
    pub unsafe fn getShaderProc32(&self) -> Option<ShaderProc32> {
        self.fShaderProc32.assume_init()
    }
    // Source: src/core/SkBitmapProcState.h:84-84
    #[cfg(not(sk_debug))]
    /// Safety: successful native setup/chooseProcs must have initialized the slot.
    pub unsafe fn getMatrixProc(&self) -> Option<MatrixProc> {
        self.fMatrixProc.assume_init()
    }
    // Source: src/core/SkBitmapProcState.h:86-86
    /// Safety: successful native setup/chooseProcs must have initialized the slot.
    pub unsafe fn getSampleProc32(&self) -> Option<SampleProc32> {
        self.fSampleProc32.assume_init()
    }
}
// Source: src/core/SkBitmapProcState.h:90-91
// Source: src/core/SkBitmapProcState.h:93-93
// Source: src/core/SkBitmapProcState.h:95-96
// Actual inline arena is dropped before the public Pixmap, matching native
// reverse member teardown. Function-pointer slots have no destruction effects.
// The arena's PhantomPinned makes the entire state immovable once pinned.
pub struct SkBitmapProcState<'image> {
    fAlloc: SkSTArenaAlloc<136>,
    fShaderProc32: MaybeUninit<Option<ShaderProc32>>,
    fMatrixProc: MaybeUninit<Option<MatrixProc>>,
    fSampleProc32: MaybeUninit<Option<SampleProc32>>,
    fields: SkBitmapProcStateFields<'image>,
}
// Field grouping/projections are uncredited Rust representation glue, no ABI claim.
impl<'image> std::ops::Deref for SkBitmapProcState<'image> {
    type Target = SkBitmapProcStateFields<'image>;
    fn deref(&self) -> &Self::Target {
        &self.fields
    }
}
impl SkBitmapProcState<'_> {
    pub(crate) fn pixmapIsOpaque(&self) -> bool {
        self.fields.fPixmap.isOpaque()
    }
}
impl std::ops::DerefMut for SkBitmapProcState<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.fields
    }
}
// Source: src/core/SkBitmapProcState.h:121-123
pub fn PACK_TWO_SHORTS(pri: u32, sec: u32) -> u32 {
    pri | (sec << 16)
}
pub fn UNPACK_PRIMARY_SHORT(packed: u32) -> u32 {
    packed & 0xffff
}
pub fn UNPACK_SECONDARY_SHORT(packed: u32) -> u32 {
    packed >> 16
}
// Source: src/core/SkBitmapProcState.h:127-131
#[cfg(sk_debug)]
pub fn pack_two_shorts(pri: u32, sec: u32) -> u32 {
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState.h\0".as_ptr(),128; @condition b"(uint16_t)pri == pri\0".as_ptr(); pri as u16 as u32==pri);
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState.h\0".as_ptr(),129; @condition b"(uint16_t)sec == sec\0".as_ptr(); sec as u16 as u32==sec);
    PACK_TWO_SHORTS(pri, sec)
}
// Source: src/core/SkBitmapProcState.h:133-133
#[cfg(not(sk_debug))]
pub fn pack_two_shorts(pri: u32, sec: u32) -> u32 {
    PACK_TWO_SHORTS(pri, sec)
}
impl SkBitmapProcStateAutoMapper {
    // Source: src/core/SkBitmapProcState.h:154-159
    // Source: src/core/SkBitmapProcState.h:161-165
    // Source: src/core/SkBitmapProcState.h:170-172
    // Source: src/core/SkBitmapProcState.h:175-178
    // Source: src/core/SkBitmapProcState.h:180-184
    /// Safety: fInvMatrix/fBilerp are initialized; filtered paths also require
    /// initialized filterOneX/Y. This is the native precondition inside setup/procs.
    pub unsafe fn new(
        s: &SkBitmapProcState<'_>,
        x: i32,
        y: i32,
        scalarPoint: Option<&mut crate::src::core::SkPoint::SkPoint>,
    ) -> Self {
        use crate::include::private::SkFixed::*;
        let pt = s
            .fInvMatrix
            .mapPoint(crate::src::core::SkPoint::SkPoint::make(
                x as f32 + 0.5,
                y as f32 + 0.5,
            ));
        let mut biasX = 0;
        let mut biasY = 0;
        if s.fBilerp.assume_init() {
            biasX = s.fFilterOneX.assume_init() >> 1;
            biasY = s.fFilterOneY.assume_init() >> 1;
        } else {
            biasX = 1;
            biasY = 1;
        }
        let fX = (SkScalarToFixed3232(pt.fX) as u64).wrapping_sub(SkFixedToFixed3232(biasX) as u64)
            as i64;
        let fY = (SkScalarToFixed3232(pt.fY) as u64).wrapping_sub(SkFixedToFixed3232(biasY) as u64)
            as i64;
        if let Some(point) = scalarPoint {
            point.set(
                pt.fX - SkFixedToScalar(biasX),
                pt.fY - SkFixedToScalar(biasY),
            );
        }
        Self { fX, fY }
    }
    // Source: src/core/SkBitmapProcState.h:186-187
    // Source: src/core/SkBitmapProcState.h:189-190
    // Source: src/core/SkBitmapProcState.h:192-193
    pub fn fixed3232X(&self) -> i64 {
        self.fX
    }
    pub fn fixed3232Y(&self) -> i64 {
        self.fY
    }
    pub fn fixedX(&self) -> i32 {
        crate::include::private::SkFixed::SkFixed3232ToFixed(self.fX)
    }
    pub fn fixedY(&self) -> i32 {
        crate::include::private::SkFixed::SkFixed3232ToFixed(self.fY)
    }
    pub fn intX(&self) -> i32 {
        crate::include::private::SkFixed::SkFixed3232ToInt(self.fX)
    }
    pub fn intY(&self) -> i32 {
        crate::include::private::SkFixed::SkFixed3232ToInt(self.fY)
    }
}
// Source: src/core/SkBitmapProcState.h:196-197
pub struct SkBitmapProcStateAutoMapper {
    fX: i64,
    fY: i64,
}
// Source: src/core/SkBitmapProcState.cpp:34-38
// Source: src/core/SkBitmapProcState.cpp:40-48
// Source: src/core/SkBitmapProcState.cpp:50-51
// Source: src/core/SkBitmapProcState.cpp:55-82
unsafe fn Clamp_S32_opaque_D32_nofilter_DX_shaderproc(
    sIn: *const std::ffi::c_void,
    x: i32,
    y: i32,
    dst: &mut [u32],
    count: i32,
) {
    use crate::include::private::SkFixed::SkFixed3232ToInt;
    use crate::include::private::SkTPin::SkTPin;
    let s = &*sIn.cast::<SkBitmapProcState<'_>>();
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),37; @condition b"s.fInvMatrix.isScaleTranslate()\0".as_ptr(); s.fInvMatrix.isScaleTranslate());
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),38; @condition b"s.fAlphaScale == 256\0".as_ptr(); (s.fAlphaScale.assume_init())==(256));
    let maxX = (s.fPixmap.width() - 1) as u32;
    let mut fx;
    let dstY;
    {
        let mapper = SkBitmapProcStateAutoMapper::new(s, x, y, None);
        let maxY = (s.fPixmap.height() - 1) as u32;
        dstY = SkTPin(mapper.intY(), 0, maxY as i32);
        fx = mapper.fixed3232X();
    }
    let src = s.fPixmap.addr32_xy(0, dstY);
    let dx = s.fInvSx.assume_init();
    if SkFixed3232ToInt(fx) as u64 <= maxX as u64
        && SkFixed3232ToInt(fx.wrapping_add(dx.wrapping_mul((count - 1) as i64))) as u64
            <= maxX as u64
    {
        let count4 = count >> 2;
        let mut out = 0usize;
        for _ in 0..count4 {
            let src0 = *src.add(SkFixed3232ToInt(fx) as usize);
            fx = fx.wrapping_add(dx);
            let src1 = *src.add(SkFixed3232ToInt(fx) as usize);
            fx = fx.wrapping_add(dx);
            let src2 = *src.add(SkFixed3232ToInt(fx) as usize);
            fx = fx.wrapping_add(dx);
            let src3 = *src.add(SkFixed3232ToInt(fx) as usize);
            fx = fx.wrapping_add(dx);
            dst[out] = src0;
            dst[out + 1] = src1;
            dst[out + 2] = src2;
            dst[out + 3] = src3;
            out += 4;
        }
        for _ in (count4 << 2)..count {
            let index = SkFixed3232ToInt(fx) as u32;
            crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),72; @condition b"index <= maxX\0".as_ptr(); index<=maxX);
            dst[out] = *src.add(index as usize);
            out += 1;
            fx = fx.wrapping_add(dx);
        }
    } else {
        for i in 0..count as usize {
            dst[i] = *src.add(SkTPin(SkFixed3232ToInt(fx), 0, maxX as i32) as usize);
            fx = fx.wrapping_add(dx);
        }
    }
}
// Source: src/core/SkBitmapProcState.cpp:84-90
// Source: src/core/SkBitmapProcState.cpp:93-94
// Source: src/core/SkBitmapProcState.cpp:96-96
// Source: src/core/SkBitmapProcState.cpp:98-101
// Source: src/core/SkBitmapProcState.cpp:104-106
// Source: src/core/SkBitmapProcState.cpp:108-111
// Source: src/core/SkBitmapProcState.cpp:113-116
// Source: src/core/SkBitmapProcState.cpp:118-119
// Source: src/core/SkBitmapProcState.cpp:122-126
unsafe fn S32_alpha_D32_nofilter_DX(
    s: &SkBitmapProcState<'_>,
    xy: &[u32],
    mut count: i32,
    colors: &mut [u32],
) {
    use super::SkColorPriv::SkAlphaMulQ;
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),86; @condition b"count > 0 && colors != nullptr\0".as_ptr(); count>0&&!colors.as_ptr().is_null());
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),87; @condition b"s.fInvMatrix.isScaleTranslate()\0".as_ptr(); s.fInvMatrix.isScaleTranslate());
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),88; @condition b"!s.fBilerp\0".as_ptr(); !s.fBilerp.assume_init());
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),89; @condition b"4 == s.fPixmap.info().bytesPerPixel()\0".as_ptr(); (s.fPixmap.info().bytesPerPixel())==(4));
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),90; @condition b"s.fAlphaScale <= 256\0".as_ptr(); s.fAlphaScale.assume_init()<=256);
    let y = xy[0];
    let mut at = 1usize;
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),94; @condition b"y < (unsigned)s.fPixmap.height()\0".as_ptr(); y<(s.fPixmap.height()as u32));
    let row = s
        .fPixmap
        .addr()
        .add(y as usize * s.fPixmap.rowBytes())
        .cast::<u32>();
    if s.fPixmap.width() == 1 {
        crate::src::opts::SkMemset_opts::memset32(
            colors,
            SkAlphaMulQ(*row, s.fAlphaScale.assume_init() as u32),
            count as usize,
        );
        return;
    }
    let mut out = 0usize;
    while count >= 4 {
        let x01 = xy[at];
        let x23 = xy[at + 1];
        at += 2;
        let p0 = *row.add(UNPACK_PRIMARY_SHORT(x01) as usize);
        let p1 = *row.add(UNPACK_SECONDARY_SHORT(x01) as usize);
        let p2 = *row.add(UNPACK_PRIMARY_SHORT(x23) as usize);
        let p3 = *row.add(UNPACK_SECONDARY_SHORT(x23) as usize);
        colors[out] = SkAlphaMulQ(p0, s.fAlphaScale.assume_init() as u32);
        colors[out + 1] = SkAlphaMulQ(p1, s.fAlphaScale.assume_init() as u32);
        colors[out + 2] = SkAlphaMulQ(p2, s.fAlphaScale.assume_init() as u32);
        colors[out + 3] = SkAlphaMulQ(p3, s.fAlphaScale.assume_init() as u32);
        out += 4;
        count -= 4;
    }
    let mut xs = xy.as_ptr().add(at).cast::<u16>();
    while count > 0 {
        count -= 1;
        colors[out] = SkAlphaMulQ(*row.add(*xs as usize), s.fAlphaScale.assume_init() as u32);
        xs = xs.add(1);
        out += 1;
    }
}
// Source: src/core/SkBitmapProcState.cpp:128-133
// Source: src/core/SkBitmapProcState.cpp:135-136
// Source: src/core/SkBitmapProcState.cpp:138-146
unsafe fn S32_alpha_D32_nofilter_DXDY(
    s: &SkBitmapProcState<'_>,
    xy: &[u32],
    count: i32,
    colors: &mut [u32],
) {
    use super::SkColorPriv::SkAlphaMulQ;
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),130; @condition b"count > 0 && colors != nullptr\0".as_ptr(); count>0&&!colors.as_ptr().is_null());
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),131; @condition b"!s.fBilerp\0".as_ptr(); !s.fBilerp.assume_init());
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),132; @condition b"4 == s.fPixmap.info().bytesPerPixel()\0".as_ptr(); (s.fPixmap.info().bytesPerPixel())==(4));
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),133; @condition b"s.fAlphaScale <= 256\0".as_ptr(); s.fAlphaScale.assume_init()<=256);
    let src = s.fPixmap.addr();
    let rb = s.fPixmap.rowBytes();
    for i in 0..count as usize {
        let XY = xy[i];
        let x = XY & 0xffff;
        let y = XY >> 16;
        crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),142; @condition b"x < (unsigned)s.fPixmap.width()\0".as_ptr(); x<s.fPixmap.width()as u32);
        crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),143; @condition b"y < (unsigned)s.fPixmap.height()\0".as_ptr(); y<s.fPixmap.height()as u32);
        colors[i] = SkAlphaMulQ(
            *src.add(y as usize * rb).cast::<u32>().add(x as usize),
            s.fAlphaScale.assume_init() as u32,
        );
    }
}
impl<'image> SkBitmapProcState<'image> {
    // Source: src/core/SkBitmapProcState.cpp:148-152
    pub fn new(image: Option<&'image dyn SkImage_Base>, tmx: SkTileMode, tmy: SkTileMode) -> Self {
        Self {
            fields: SkBitmapProcStateFields {
                fImage: image,
                fPixmap: SkPixmap::default(),
                fInvMatrix: SkMatrix::default(),
                fPaintAlpha: MaybeUninit::uninit(),
                fTileModeX: tmx,
                fTileModeY: tmy,
                fBilerp: MaybeUninit::uninit(),
                fInvSx: MaybeUninit::uninit(),
                fInvKy: MaybeUninit::uninit(),
                fFilterOneX: MaybeUninit::uninit(),
                fFilterOneY: MaybeUninit::uninit(),
                fAlphaScale: MaybeUninit::uninit(),
            },
            fAlloc: SkSTArenaAlloc::new(136),
            fShaderProc32: MaybeUninit::uninit(),
            fMatrixProc: MaybeUninit::uninit(),
            fSampleProc32: MaybeUninit::uninit(),
        }
    }
}
// Source: src/core/SkBitmapProcState.cpp:155-157
pub(crate) fn matrix_only_scale_translate(m: &SkMatrix) -> bool {
    (m.getType() & !SkMatrix::kTranslate_Mask) == SkMatrix::kScale_Mask
}
// Source: src/core/SkBitmapProcState.cpp:163-164
// Source: src/core/SkBitmapProcState.cpp:166-166
// Source: src/core/SkBitmapProcState.cpp:168-170
pub(crate) fn just_trans_general(matrix: &SkMatrix) -> bool {
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),164; @condition b"matrix_only_scale_translate(matrix)\0".as_ptr(); matrix_only_scale_translate(matrix));
    let tol = 1.0 / 32768.0;
    SkScalarNearlyZero(matrix.getScaleX() - 1.0, tol)
        && SkScalarNearlyZero(matrix.getScaleY() - 1.0, tol)
}
// Source: src/core/SkBitmapProcState.cpp:176-177
// Source: src/core/SkBitmapProcState.cpp:179-182
pub(crate) fn just_trans_integral(m: &SkMatrix) -> bool {
    let tol = 1.0 / 256.0;
    m.getType() <= SkMatrix::kTranslate_Mask
        && SkScalarNearlyEqual(
            m.getTranslateX(),
            SkScalarRoundToScalar(m.getTranslateX()),
            tol,
        )
        && SkScalarNearlyEqual(
            m.getTranslateY(),
            SkScalarRoundToScalar(m.getTranslateY()),
            tol,
        )
}
// Source: src/core/SkBitmapProcState.cpp:184-184
// Source: src/core/SkBitmapProcState.cpp:187-188
pub(crate) fn valid_for_filtering(dimension: u32) -> bool {
    dimension & !0x3fff == 0
}
impl SkBitmapProcState<'_> {
    // Source: src/core/SkBitmapProcState.cpp:190-196
    // Source: src/core/SkBitmapProcState.cpp:198-199
    // Source: src/core/SkBitmapProcState.cpp:201-206
    // Source: src/core/SkBitmapProcState.cpp:208-210
    // Source: src/core/SkBitmapProcState.cpp:212-213
    // Source: src/core/SkBitmapProcState.cpp:221-223
    // Source: src/core/SkBitmapProcState.cpp:233-239
    // Source: src/core/SkBitmapProcState.cpp:242-243
    // Source: src/core/SkBitmapProcState.cpp:245-248
    // Source: src/core/SkBitmapProcState.cpp:250-251
    #[cfg(all(target_arch = "aarch64", target_pointer_width = "64"))]
    fn init(
        mut self: std::pin::Pin<&mut Self>,
        inv: &SkMatrix,
        paintAlpha: u8,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
    ) -> Result<bool, crate::RasterError> {
        use crate::include::core::SkSamplingOptions::{SkFilterMode, SkMipmapMode};
        crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),192; @condition b"!inv.hasPerspective()\0".as_ptr(); !inv.hasPerspective());
        crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),193; @condition b"SkOpts::S32_alpha_D32_filter_DXDY || inv.isScaleTranslate()\0".as_ptr(); crate::src::core::SkOpts::S32_alpha_D32_filter_DXDY.is_some() || inv.isScaleTranslate());
        crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),194; @condition b"!sampling.isAniso()\0".as_ptr(); !sampling.isAniso());
        crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),195; @condition b"!sampling.useCubic\0".as_ptr(); !sampling.useCubic());
        crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),196; @condition b"sampling.mipmap != SkMipmapMode::kLinear\0".as_ptr(); sampling.mipmap() != SkMipmapMode::kLinear);
        // Only unpinned fields are mutated; fAlloc is separately projected and
        // pinned below, and the whole state remains at its placement address.
        let this = unsafe { self.as_mut().get_unchecked_mut() };
        this.fields.fPixmap.reset();
        this.fields.fBilerp.write(false);
        let image = this
            .fields
            .fImage
            .expect("native init requires nonnull image");
        let access = unsafe {
            let arena = std::pin::Pin::new_unchecked(&mut this.fAlloc).arena();
            crate::src::core::SkMipmapAccessor::SkMipmapAccessor::Make(
                arena,
                image,
                inv,
                sampling.mipmap(),
            )?
        };
        let Some(access) = access else {
            return Ok(false);
        };
        let (pixmap, matrix) = unsafe { access.as_ref() }.level();
        this.fields.fPixmap = pixmap;
        this.fields.fInvMatrix = matrix;
        this.fields.fInvMatrix.preConcat(inv);
        this.fields.fPaintAlpha.write(paintAlpha);
        this.fields
            .fBilerp
            .write(sampling.filter() == SkFilterMode::kLinear);
        crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),210; @condition b"fPixmap.addr()\0".as_ptr(); !this.fields.fPixmap.addr().is_null());
        let mut integral_translate_only = just_trans_integral(&this.fields.fInvMatrix);
        if !integral_translate_only {
            if this.fields.fTileModeX != SkTileMode::kClamp
                || this.fields.fTileModeY != SkTileMode::kClamp
            {
                let _ = crate::src::core::SkMatrixPriv::SkMatrixPriv::PostIDiv(
                    &mut this.fields.fInvMatrix,
                    this.fields.fPixmap.width(),
                    this.fields.fPixmap.height(),
                );
            }
            if matrix_only_scale_translate(&this.fields.fInvMatrix) {
                if let Some(forward) = this.fields.fInvMatrix.invert() {
                    if just_trans_general(&forward) {
                        this.fields
                            .fInvMatrix
                            .setTranslate(-forward.getTranslateX(), -forward.getTranslateY());
                    }
                }
            }
            integral_translate_only = just_trans_integral(&this.fields.fInvMatrix);
        }
        if unsafe { this.fields.fBilerp.assume_init() }
            && (!valid_for_filtering(
                (this.fields.fPixmap.width() | this.fields.fPixmap.height()) as u32,
            ) || integral_translate_only)
        {
            this.fields.fBilerp.write(false);
        }
        Ok(true)
    }
}
impl SkBitmapProcState<'_> {
    // Source: src/core/SkBitmapProcState.cpp:263-268
    // Source: src/core/SkBitmapProcState.cpp:270-270
    // Source: src/core/SkBitmapProcState.cpp:272-273
    // Source: src/core/SkBitmapProcState.cpp:275-275
    // Source: src/core/SkBitmapProcState.cpp:277-279
    // Source: src/core/SkBitmapProcState.cpp:281-286
    // Source: src/core/SkBitmapProcState.cpp:290-298
    // Source: src/core/SkBitmapProcState.cpp:300-301
    // Crate-only friend visibility supports paired numeric verification using
    // this actual state. Safety: native init invariants (Pixmap/Matrix/alpha/
    // filtering/tile state) must already hold; this does not establish them.
    pub(crate) unsafe fn chooseProcs(&mut self) -> bool {
        use crate::include::core::{
            SkAlphaType::{kOpaque_SkAlphaType, kPremul_SkAlphaType},
            SkColorType::kN32_SkColorType,
        };
        crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),264; @condition b"!fInvMatrix.hasPerspective()\0".as_ptr(); !self.fInvMatrix.hasPerspective());
        crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),265; @condition b"SkOpts::S32_alpha_D32_filter_DXDY || fInvMatrix.isScaleTranslate()\0".as_ptr(); crate::src::core::SkOpts::S32_alpha_D32_filter_DXDY.is_some() || self.fInvMatrix.isScaleTranslate());
        crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),266; @condition b"fPixmap.colorType() == kN32_SkColorType\0".as_ptr(); (self.fPixmap.colorType())==(kN32_SkColorType));
        crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),267; @condition b"fPixmap.alphaType() == kPremul_SkAlphaType || fPixmap.alphaType() == kOpaque_SkAlphaType\0".as_ptr(); self.fPixmap.alphaType()==kPremul_SkAlphaType||self.fPixmap.alphaType()==kOpaque_SkAlphaType);
        crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),270; @condition b"fTileModeX != SkTileMode::kDecal\0".as_ptr(); (self.fTileModeX)!=(SkTileMode::kDecal));
        let invSx =
            crate::include::private::SkFixed::SkScalarToFixed3232(self.fInvMatrix.getScaleX());
        let invKy =
            crate::include::private::SkFixed::SkScalarToFixed3232(self.fInvMatrix.getSkewY());
        self.fields.fInvSx.write(invSx);
        self.fields.fInvKy.write(invKy);
        let alpha =
            super::SkColorPriv::SkAlpha255To256(self.fPaintAlpha.assume_init() as u32) as u16;
        self.fields.fAlphaScale.write(alpha);
        let translate_only = (self.fInvMatrix.getType() & !SkMatrix::kTranslate_Mask) == 0;
        let matrix = self.chooseMatrixProc(translate_only);
        self.fMatrixProc.write(matrix);
        crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),279; @condition b"fMatrixProc\0".as_ptr(); self.fMatrixProc.assume_init().is_some());
        let sample = if self.fInvMatrix.isScaleTranslate() {
            if self.fBilerp.assume_init() {
                crate::src::core::SkOpts::S32_alpha_D32_filter_DX
            } else {
                Some(S32_alpha_D32_nofilter_DX as SampleProc32)
            }
        } else {
            if self.fBilerp.assume_init() {
                crate::src::core::SkOpts::S32_alpha_D32_filter_DXDY
            } else {
                Some(S32_alpha_D32_nofilter_DXDY as SampleProc32)
            }
        };
        self.fSampleProc32.write(sample);
        crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),286; @condition b"fSampleProc32\0".as_ptr(); sample.is_some());
        let shader = if self.fAlphaScale.assume_init() == 256
            && !self.fBilerp.assume_init()
            && self.fTileModeX == SkTileMode::kClamp
            && self.fTileModeY == SkTileMode::kClamp
            && self.fInvMatrix.isScaleTranslate()
        {
            Some(Clamp_S32_opaque_D32_nofilter_DX_shaderproc as ShaderProc32)
        } else {
            self.chooseShaderProc32()
        };
        self.fShaderProc32.write(shader);
        true
    }
}
// Source: src/core/SkBitmapProcState.cpp:303-310
// Source: src/core/SkBitmapProcState.cpp:312-316
// Source: src/core/SkBitmapProcState.cpp:319-329
// Source: src/core/SkBitmapProcState.cpp:331-340
// Source: src/core/SkBitmapProcState.cpp:342-343
unsafe fn Clamp_S32_D32_nofilter_trans_shaderproc(
    sIn: *const std::ffi::c_void,
    x: i32,
    y: i32,
    colors: &mut [u32],
    mut count: i32,
) {
    use crate::include::private::SkTPin::SkTPin;
    let s = &*sIn.cast::<SkBitmapProcState<'_>>();
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),308; @condition b"s.fInvMatrix.isTranslate()\0".as_ptr(); s.fInvMatrix.isTranslate());
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),309; @condition b"count > 0 && colors != nullptr\0".as_ptr(); count>0&&!colors.as_ptr().is_null());
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),310; @condition b"!s.fBilerp\0".as_ptr(); !s.fBilerp.assume_init());
    let maxX = s.fPixmap.width() - 1;
    let maxY = s.fPixmap.height() - 1;
    let mut ix = s.fFilterOneX.assume_init().wrapping_add(x);
    let iy = SkTPin(s.fFilterOneY.assume_init().wrapping_add(y), 0, maxY);
    let row = s.fPixmap.addr32_xy(0, iy);
    let mut out = 0usize;
    if ix < 0 {
        let n = (-ix).min(count);
        crate::src::opts::SkMemset_opts::memset32(&mut colors[out..], *row, n as usize);
        count -= n;
        if count == 0 {
            return;
        }
        out += n as usize;
        crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),327; @condition b"-ix == n\0".as_ptr(); (-ix)==(n));
        ix = 0;
    }
    if ix <= maxX {
        let n = (maxX - ix + 1).min(count);
        std::ptr::copy_nonoverlapping(
            row.add(ix as usize),
            colors.as_mut_ptr().add(out),
            n as usize,
        );
        count -= n;
        if count == 0 {
            return;
        }
        out += n as usize;
    }
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),340; @condition b"count > 0\0".as_ptr(); count>0);
    crate::src::opts::SkMemset_opts::memset32(
        &mut colors[out..],
        *row.add(maxX as usize),
        count as usize,
    );
}
// Source: src/core/SkBitmapProcState.cpp:345-355
pub(crate) fn sk_int_mod(mut x: i32, n: i32) -> i32 {
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),346; @condition b"n > 0\0".as_ptr(); n > 0);
    if x as u32 >= n as u32 {
        if x < 0 {
            x = n + !(!x % n);
        } else {
            x %= n;
        }
    }
    x
}
// Source: src/core/SkBitmapProcState.cpp:357-363
pub(crate) fn sk_int_mirror(mut x: i32, n: i32) -> i32 {
    x = sk_int_mod(x, 2 * n);
    if x >= n {
        x = n + !(x - n);
    }
    x
}
// Source: src/core/SkBitmapProcState.cpp:365-372
// Source: src/core/SkBitmapProcState.cpp:374-378
// Source: src/core/SkBitmapProcState.cpp:380-391
unsafe fn Repeat_S32_D32_nofilter_trans_shaderproc(
    sIn: *const std::ffi::c_void,
    x: i32,
    y: i32,
    colors: &mut [u32],
    mut count: i32,
) {
    let s = &*sIn.cast::<SkBitmapProcState<'_>>();
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),370; @condition b"s.fInvMatrix.isTranslate()\0".as_ptr(); s.fInvMatrix.isTranslate());
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),371; @condition b"count > 0 && colors != nullptr\0".as_ptr(); count>0&&!colors.as_ptr().is_null());
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),372; @condition b"!s.fBilerp\0".as_ptr(); !s.fBilerp.assume_init());
    let stopX = s.fPixmap.width();
    let stopY = s.fPixmap.height();
    let mut ix = s.fFilterOneX.assume_init().wrapping_add(x);
    let iy = sk_int_mod(s.fFilterOneY.assume_init().wrapping_add(y), stopY);
    let row = s.fPixmap.addr32_xy(0, iy);
    ix = sk_int_mod(ix, stopX);
    let mut out = 0usize;
    loop {
        let n = (stopX - ix).min(count);
        std::ptr::copy_nonoverlapping(
            row.add(ix as usize),
            colors.as_mut_ptr().add(out),
            n as usize,
        );
        count -= n;
        if count == 0 {
            return;
        }
        out += n as usize;
        ix = 0;
    }
}
// Source: src/core/SkBitmapProcState.cpp:393-399
// Source: src/core/SkBitmapProcState.cpp:401-401
// Source: src/core/SkBitmapProcState.cpp:403-405
// Source: src/core/SkBitmapProcState.cpp:407-409
// Source: src/core/SkBitmapProcState.cpp:412-413
// Source: src/core/SkBitmapProcState.cpp:415-416
pub(crate) fn filter_32_alpha(
    t: u32,
    color0: u32,
    color1: u32,
    dstColor: &mut u32,
    alphaScale: u32,
) {
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),398; @condition b"(unsigned)t <= 0xF\0".as_ptr(); t <= 15);
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),399; @condition b"alphaScale <= 256\0".as_ptr(); alphaScale <= 256);
    let mask = 0x00ff00ff;
    let mut scale = 256 - 16 * t;
    let mut lo = (color0 & mask).wrapping_mul(scale);
    let mut hi = ((color0 >> 8) & mask).wrapping_mul(scale);
    scale = 16 * t;
    lo = lo.wrapping_add((color1 & mask).wrapping_mul(scale));
    hi = hi.wrapping_add(((color1 >> 8) & mask).wrapping_mul(scale));
    lo = ((lo >> 8) & mask).wrapping_mul(alphaScale);
    hi = ((hi >> 8) & mask).wrapping_mul(alphaScale);
    *dstColor = ((lo >> 8) & mask) | (hi & !mask);
}
// Source: src/core/SkBitmapProcState.cpp:418-425
// Source: src/core/SkBitmapProcState.cpp:427-429
// Source: src/core/SkBitmapProcState.cpp:431-432
// Source: src/core/SkBitmapProcState.cpp:435-435
// Source: src/core/SkBitmapProcState.cpp:437-441
// Source: src/core/SkBitmapProcState.cpp:443-446
// Source: src/core/SkBitmapProcState.cpp:452-457
// Source: src/core/SkBitmapProcState.cpp:459-471
// Source: src/core/SkBitmapProcState.cpp:474-476
// Source: src/core/SkBitmapProcState.cpp:478-483
// Source: src/core/SkBitmapProcState.cpp:485-496
// Source: src/core/SkBitmapProcState.cpp:498-499
// Source: src/core/SkBitmapProcState.cpp:501-501
// Source: src/core/SkBitmapProcState.cpp:503-504
// Source: src/core/SkBitmapProcState.cpp:506-515
// Source: src/core/SkBitmapProcState.cpp:517-518
unsafe fn S32_D32_constX_shaderproc(
    sIn: *const std::ffi::c_void,
    x: i32,
    y: i32,
    colors: &mut [u32],
    count: i32,
) {
    use crate::include::private::{SkFixed::SkFixed3232ToInt, SkTPin::SkTPin};
    let s = &*sIn.cast::<SkBitmapProcState<'_>>();
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),423; @condition b"s.fInvMatrix.isScaleTranslate()\0".as_ptr(); s.fInvMatrix.isScaleTranslate());
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),424; @condition b"count > 0 && colors != nullptr\0".as_ptr(); count>0&&!colors.as_ptr().is_null());
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),425; @condition b"1 == s.fPixmap.width()\0".as_ptr(); (s.fPixmap.width())==(1));
    let iY0: i32;
    let mut iY1 = MaybeUninit::<i32>::uninit();
    let mut iSubY = MaybeUninit::<u32>::uninit();
    if s.fBilerp.assume_init() {
        let mproc = s.getMatrixProc().expect("native initialized matrix proc");
        // Adapted/uncredited native433: initialized words permit the Rust slice
        // boundary; native stack output storage is otherwise uninitialized.
        let mut xy = [0u32; 2];
        mproc(s, &mut xy, 1, x, y);
        iY0 = (xy[0] >> 18) as i32;
        iY1.write((xy[0] & 0x3fff) as i32);
        iSubY.write((xy[0] >> 14) & 0xf);
    } else {
        let yTemp = if s.fInvMatrix.isTranslate() {
            s.fFilterOneY.assume_init().wrapping_add(y)
        } else {
            let mapper = SkBitmapProcStateAutoMapper::new(s, x, y, None);
            if s.fTileModeX != SkTileMode::kClamp || s.fTileModeY != SkTileMode::kClamp {
                SkFixed3232ToInt(mapper.fixed3232Y().wrapping_mul(s.fPixmap.height() as i64))
            } else {
                mapper.intY()
            }
        };
        let stopY = s.fPixmap.height();
        iY0 = match s.fTileModeY {
            SkTileMode::kClamp => SkTPin(yTemp, 0, stopY - 1),
            SkTileMode::kRepeat => sk_int_mod(yTemp, stopY),
            _ => sk_int_mirror(yTemp, stopY),
        };
        #[cfg(sk_debug)]
        {
            let mapper = SkBitmapProcStateAutoMapper::new(s, x, y, None);
            let mut iY2 = if !s.fInvMatrix.isTranslate()
                && (s.fTileModeX != SkTileMode::kClamp || s.fTileModeY != SkTileMode::kClamp)
            {
                SkFixed3232ToInt(mapper.fixed3232Y().wrapping_mul(s.fPixmap.height() as i64))
            } else {
                mapper.intY()
            };
            iY2 = match s.fTileModeY {
                SkTileMode::kClamp => SkTPin(iY2, 0, stopY - 1),
                SkTileMode::kRepeat => sk_int_mod(iY2, stopY),
                _ => sk_int_mirror(iY2, stopY),
            };
            crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),498; @condition b"iY0 == iY2\0".as_ptr(); (iY0)==(iY2));
        }
    }
    let row0 = s.fPixmap.addr32_xy(0, iY0);
    let mut color = 0;
    if s.fBilerp.assume_init() {
        let row1 = s.fPixmap.addr32_xy(0, iY1.assume_init());
        filter_32_alpha(
            iSubY.assume_init(),
            *row0,
            *row1,
            &mut color,
            s.fAlphaScale.assume_init() as u32,
        );
    } else {
        if s.fAlphaScale.assume_init() < 256 {
            color = super::SkColorPriv::SkAlphaMulQ(*row0, s.fAlphaScale.assume_init() as u32);
        } else {
            color = *row0;
        }
    }
    crate::src::opts::SkMemset_opts::memset32(colors, color, count as usize);
}
// Source: src/core/SkBitmapProcState.cpp:520-521
// Source: src/core/SkBitmapProcState.cpp:523-524
unsafe fn DoNothing_shaderproc(
    _sIn: *const std::ffi::c_void,
    _x: i32,
    _y: i32,
    colors: &mut [u32],
    count: i32,
) {
    // This is the actual native too-large-translation shader, not a missing proc fallback.
    crate::src::opts::SkMemset_opts::memset32(colors, 0, count as usize);
}
impl SkBitmapProcState<'_> {
    // Source: src/core/SkBitmapProcState.cpp:526-528
    // Source: src/core/SkBitmapProcState.cpp:535-538
    // Source: src/core/SkBitmapProcState.cpp:543-544
    // Source: src/core/SkBitmapProcState.cpp:546-547
    unsafe fn setupForTranslate(&mut self) -> bool {
        let mut pt = crate::src::core::SkPoint::SkPoint::default();
        let mapper = SkBitmapProcStateAutoMapper::new(self, 0, 0, Some(&mut pt));
        let too_big = (1i32 << 30) as f32;
        if pt.fX.abs() > too_big || pt.fY.abs() > too_big {
            return false;
        }
        self.fields.fFilterOneX.write(mapper.intX());
        self.fields.fFilterOneY.write(mapper.intY());
        true
    }
    // Source: src/core/SkBitmapProcState.cpp:549-549
    // Source: src/core/SkBitmapProcState.cpp:551-553
    // Source: src/core/SkBitmapProcState.cpp:555-560
    // Source: src/core/SkBitmapProcState.cpp:562-570
    // Source: src/core/SkBitmapProcState.cpp:572-573
    // Source: src/core/SkBitmapProcState.cpp:575-588
    unsafe fn chooseShaderProc32(&mut self) -> Option<ShaderProc32> {
        if self.fPixmap.colorType() != crate::include::core::SkColorType::kN32_SkColorType {
            return None;
        }
        if self.fPixmap.width() == 1 && self.fInvMatrix.isScaleTranslate() {
            if !self.fBilerp.assume_init()
                && self.fInvMatrix.isTranslate()
                && !self.setupForTranslate()
            {
                return Some(DoNothing_shaderproc);
            }
            return Some(S32_D32_constX_shaderproc);
        }
        if self.fAlphaScale.assume_init() < 256 {
            return None;
        }
        if !self.fInvMatrix.isTranslate() {
            return None;
        }
        if self.fBilerp.assume_init() {
            return None;
        }
        let tx = self.fTileModeX;
        let ty = self.fTileModeY;
        if tx == SkTileMode::kClamp && ty == SkTileMode::kClamp {
            if self.setupForTranslate() {
                return Some(Clamp_S32_D32_nofilter_trans_shaderproc);
            }
            return Some(DoNothing_shaderproc);
        }
        if tx == SkTileMode::kRepeat && ty == SkTileMode::kRepeat {
            if self.setupForTranslate() {
                return Some(Repeat_S32_D32_nofilter_trans_shaderproc);
            }
            return Some(DoNothing_shaderproc);
        }
        None
    }
}
// Source: src/core/SkBitmapProcState.cpp:592-595
// Source: src/core/SkBitmapProcState.cpp:597-601
#[cfg(sk_debug)]
pub(crate) fn check_scale_nofilter(bitmapXY: &[u32], count: i32, mx: u32, my: u32) {
    let y = bitmapXY[0];
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),595; @condition b"y < my\0".as_ptr(); y < my);
    // The source reinterprets the remaining native words as u16 coordinates.
    for i in 0..count as usize {
        let bytes = bitmapXY[1 + i / 2].to_ne_bytes();
        let offset = 2 * (i % 2);
        let x = u16::from_ne_bytes([bytes[offset], bytes[offset + 1]]);
        crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),599; @condition b"xptr[i] < mx\0".as_ptr(); (x as u32) < mx);
    }
}
// Source: src/core/SkBitmapProcState.cpp:603-609
// Source: src/core/SkBitmapProcState.cpp:611-618
#[cfg(sk_debug)]
pub(crate) fn check_scale_filter(bitmapXY: &[u32], count: i32, mx: u32, my: u32) {
    let yy = bitmapXY[0];
    let y0 = yy >> 18;
    let y1 = yy & 0x3fff;
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),608; @condition b"y0 < my\0".as_ptr(); y0 < my);
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),609; @condition b"y1 < my\0".as_ptr(); y1 < my);
    for i in 0..count as usize {
        let xx = bitmapXY[1 + i];
        let x0 = xx >> 18;
        let x1 = xx & 0x3fff;
        crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),615; @condition b"x0 < mx\0".as_ptr(); x0 < mx);
        crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),616; @condition b"x1 < mx\0".as_ptr(); x1 < mx);
    }
}
// Source: src/core/SkBitmapProcState.cpp:620-628
#[cfg(sk_debug)]
pub(crate) fn check_affine_nofilter(bitmapXY: &[u32], count: i32, mx: u32, my: u32) {
    for i in 0..count as usize {
        let xy = bitmapXY[i];
        let x = xy & 65535;
        let y = xy >> 16;
        crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),625; @condition b"x < mx\0".as_ptr(); x < mx);
        crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),626; @condition b"y < my\0".as_ptr(); y < my);
    }
}
// Source: src/core/SkBitmapProcState.cpp:630-636
// Source: src/core/SkBitmapProcState.cpp:638-644
#[cfg(sk_debug)]
pub(crate) fn check_affine_filter(bitmapXY: &[u32], count: i32, mx: u32, my: u32) {
    let mut offset = 0;
    for _ in 0..count {
        let yy = bitmapXY[offset];
        offset += 1;
        let y0 = yy >> 18;
        let y1 = yy & 0x3fff;
        crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),635; @condition b"y0 < my\0".as_ptr(); y0 < my);
        crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),636; @condition b"y1 < my\0".as_ptr(); y1 < my);
        let xx = bitmapXY[offset];
        offset += 1;
        let x0 = xx >> 18;
        let x1 = xx & 0x3fff;
        crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),641; @condition b"x0 < mx\0".as_ptr(); x0 < mx);
        crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),642; @condition b"x1 < mx\0".as_ptr(); x1 < mx);
    }
}
impl SkBitmapProcState<'_> {
    // Source: src/core/SkBitmapProcState.cpp:646-650
    // Source: src/core/SkBitmapProcState.cpp:652-652
    // Source: src/core/SkBitmapProcState.cpp:654-654
    // Source: src/core/SkBitmapProcState.cpp:656-660
    // Source: src/core/SkBitmapProcState.cpp:662-663
    #[cfg(sk_debug)]
    unsafe fn DebugMatrixProc(
        state: &SkBitmapProcState<'_>,
        bitmapXY: &mut [u32],
        count: i32,
        x: i32,
        y: i32,
    ) {
        crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),649; @condition b"bitmapXY\0".as_ptr(); !bitmapXY.as_ptr().is_null());
        crate::SkASSERT!(@source b"src/core/SkBitmapProcState.cpp\0".as_ptr(),650; @condition b"count > 0\0".as_ptr(); count > 0);
        (state
            .fMatrixProc
            .assume_init()
            .expect("native initialized matrix proc"))(state, bitmapXY, count, x, y);
        let proc: fn(&[u32], i32, u32, u32) = if state.fInvMatrix.isScaleTranslate() {
            if state.fBilerp.assume_init() {
                check_scale_filter
            } else {
                check_scale_nofilter
            }
        } else {
            if state.fBilerp.assume_init() {
                check_affine_filter
            } else {
                check_affine_nofilter
            }
        };
        proc(
            bitmapXY,
            count,
            state.fPixmap.width() as u32,
            state.fPixmap.height() as u32,
        );
    }
    // Source: src/core/SkBitmapProcState.cpp:665-667
    #[cfg(sk_debug)]
    /// Safety: successful native setup/chooseProcs must have initialized matrix state/proc.
    pub unsafe fn getMatrixProc(&self) -> Option<MatrixProc> {
        Some(Self::DebugMatrixProc)
    }
    // Source: src/core/SkBitmapProcState.cpp:680-681
    // Source: src/core/SkBitmapProcState.cpp:683-692
    // Source: src/core/SkBitmapProcState.cpp:694-696
    // Source: src/core/SkBitmapProcState.cpp:698-699
    /// Safety: chooseProcs/setup has initialized fBilerp and fInvMatrix.
    pub unsafe fn maxCountForBufferSize(&self, bufferSize: usize) -> i32 {
        let mut size = bufferSize as i32;
        size &= !3;
        if self.fInvMatrix.isScaleTranslate() {
            size -= 4;
            if size < 0 {
                size = 0;
            }
            size >>= 1;
        } else {
            size >>= 2;
        }
        if self.fBilerp.assume_init() {
            size >>= 1;
        }
        size
    }
}
