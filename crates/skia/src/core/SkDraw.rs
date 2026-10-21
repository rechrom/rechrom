use crate::src::core::SkFloatingPoint::sk_float_midpoint;
use crate::src::core::SkMatrix::SkMatrix;
use crate::src::core::SkPoint::SkPoint;

// Source: src/core/SkDraw.h:116-118
impl Draw<'_> {
    /// Safety: the original Draw destination, clip, CTM and arena/chooser pixel
    /// leases remain valid throughout the full native drawing operation.
    pub unsafe fn drawRectSimple(
        &self,
        rect: &crate::src::core::SkRect::SkRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { self.drawRect(rect, paint, None, None) }
    }
}

// Source: src/core/SkDraw.h:129-133
impl Draw<'_> {
    /// # Safety
    /// Uphold Draw's destination pixel lease, exclusive access, and chooser/arena contract.
    pub unsafe fn drawPath(
        &self,
        path: &crate::src::core::SkPath::SkPath,
        paint: &crate::src::core::SkPaint::SkPaint,
        pre: Option<&SkMatrix>,
    ) -> Result<(), crate::RasterError> {
        self.drawPathWithCoverage(
            path,
            paint,
            pre,
            crate::src::core::SkDrawTypes::SkDrawCoverage::kNo,
            None,
        )?;
        Ok(())
    }
}

// Source: src/core/SkDraw.h:141-150
impl Draw<'_> {
    /// # Safety
    /// Uphold Draw's destination pixel lease, exclusive access, and chooser/arena contract.
    pub unsafe fn drawPathCoverage(
        &self,
        path: &crate::src::core::SkPath::SkPath,
        paint: &crate::src::core::SkPaint::SkPaint,
        custom: Option<core::pin::Pin<&mut dyn crate::src::core::SkBlitter::SkBlitter>>,
    ) -> Result<(), crate::RasterError> {
        let is_hairline = paint.getStyle() == crate::src::core::SkPaint::Style::kStroke_Style
            && paint.getStrokeWidth() == 0.0;
        self.drawPathWithCoverage(
            path,
            paint,
            None,
            if is_hairline {
                crate::src::core::SkDrawTypes::SkDrawCoverage::kNo
            } else {
                crate::src::core::SkDrawTypes::SkDrawCoverage::kYes
            },
            custom,
        )?;
        Ok(())
    }
}

// Source: src/core/SkDraw.h:157-162
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RectType {
    kHair,
    kFill,
    kStroke,
    kPath,
}

// Source: src/core/SkDraw.h:177-184
// Arena owns the returned raw blitter until arena destruction, matching canonical
// SkAutoBlitterChoose::get (NonNull, not a second owning Box). Caller must not use
// the pointer after arena reset/drop, alias mutable blitter access, or delete it.
// Factory may return null/None; clipShader ownership transfers into the factory.
// Uncompiled native factory dependency: SkBlitter module has no Choose yet.
pub type BlitterChooser = for<'arena> unsafe fn(
    &crate::src::core::SkPixmap::SkPixmap,
    &SkMatrix,
    &crate::src::core::SkPaint::SkPaint,
    core::pin::Pin<&'arena mut crate::src::core::SkArenaAlloc::SkArenaAlloc>,
    crate::src::core::SkDrawTypes::SkDrawCoverage,
    Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>>,
    &crate::include::core::SkSurfaceProps::SkSurfaceProps,
    &crate::src::core::SkRect::SkRect,
) -> Result<
    Option<core::ptr::NonNull<dyn crate::src::core::SkBlitter::SkBlitter + 'arena>>,
    crate::RasterError,
>;

// Source: src/core/SkDraw.h:76-89
// Native abstract BitmapDevicePainter is a borrow of the actual complete Draw,
// never a second painter allocation. Glyph/image leases stay with its caller.
pub trait BitmapDevicePainter {
    /// Safety: actual glyph pixels, destination and clip leases cover this call.
    unsafe fn paintMasks(
        &self,
        accepted: &[(&crate::src::core::SkGlyph::SkGlyph, SkPoint)],
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError>;
    /// Safety: preserve source image and destination pixel leases for the draw.
    unsafe fn drawBitmap(
        &self,
        bitmap: &crate::src::core::SkBitmap::SkBitmap,
        matrix: &SkMatrix,
        dst: Option<&crate::src::core::SkRect::SkRect>,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: &crate::src::core::SkPaint::SkPaint,
        mips: Option<crate::src::core::SkMipmap::SkMipmap>,
    ) -> Result<(), crate::RasterError>;
}
// Source: src/core/SkDraw.h:107-107
impl BitmapDevicePainter for Draw<'_> {
    unsafe fn paintMasks(
        &self,
        accepted: &[(&crate::src::core::SkGlyph::SkGlyph, SkPoint)],
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { Draw::paintMasks(self, accepted, paint) }
    }
    unsafe fn drawBitmap(
        &self,
        bitmap: &crate::src::core::SkBitmap::SkBitmap,
        matrix: &SkMatrix,
        dst: Option<&crate::src::core::SkRect::SkRect>,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: &crate::src::core::SkPaint::SkPaint,
        mips: Option<crate::src::core::SkMipmap::SkMipmap>,
    ) -> Result<(), crate::RasterError> {
        unsafe { Draw::drawBitmap(self, bitmap, matrix, dst, sampling, paint, mips) }
    }
}
// Source: src/core/SkDraw.h:248-254
// Required CTM/clip pointers remain absent until the caller configures the Draw;
// the operation-scoped references preserve native non-owning relationships.
/// CPU drawing state retains native raw destination access. Every drawing entry
/// is unsafe: for the entire call, fDst must cover its declared dimensions/stride
/// and color layout with live writable storage, initialized whenever blending
/// reads it. The caller must exclude concurrent/aliased pixel reads and writes,
/// including other Bitmap/Draw clones sharing the backing store. Required CTM/RC
/// must be configured and clip bounds must be valid for the destination. Chooser,
/// shaders, filters and custom blitter must obey their native contracts; returned
/// blitters live only in their arena and cannot retain operation borrows beyond
/// it. Custom blitter backing and its owner must remain live through the draw.
pub struct Draw<'operation> {
    // Source: src/core/SkDraw.h:248-248
    pub fDst: crate::src::core::SkPixmap::SkPixmap,
    // Source: src/core/SkDraw.h:249-249
    pub fBlitterChooser: BlitterChooser,
    // Source: src/core/SkDraw.h:250-250
    pub fCTM: Option<&'operation SkMatrix>,
    // Source: src/core/SkDraw.h:251-251
    pub fRC: Option<&'operation crate::src::core::SkRasterClip::SkRasterClip>,
    // Source: src/core/SkDraw.h:252-252
    pub fProps: Option<&'operation crate::include::core::SkSurfaceProps::SkSurfaceProps>,
    // Source: src/core/SkDraw.h:254-254
    pub fCtx: Option<&'operation crate::src::core::SkCPUContextImpl::ContextImpl>,
}

// Source: src/core/SkDraw.h:248-254
// Implicit native copy operations copy the actual descriptor before its
// non-owning function/context/clip pointers; no destination pixel owner is added.
impl Clone for Draw<'_> {
    fn clone(&self) -> Self {
        Self {
            fDst: self.fDst.clone(),
            fBlitterChooser: self.fBlitterChooser,
            fCTM: self.fCTM,
            fRC: self.fRC,
            fProps: self.fProps,
            fCtx: self.fCtx,
        }
    }
    fn clone_from(&mut self, source: &Self) {
        self.fDst.clone_from(&source.fDst);
        self.fBlitterChooser = source.fBlitterChooser;
        self.fCTM = source.fCTM;
        self.fRC = source.fRC;
        self.fProps = source.fProps;
        self.fCtx = source.fCtx;
    }
}

// Source: src/core/SkDraw.cpp:72-82
/// Safety: bitmap pixels remain live and immutable/exclusively leased for every
/// shader use; this native kNever path intentionally retains rather than copies.
unsafe fn make_paint_with_image_and_mips(
    orig_paint: &crate::src::core::SkPaint::SkPaint,
    bitmap: &crate::src::core::SkBitmap::SkBitmap,
    sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
    matrix: Option<&SkMatrix>,
    mips: Option<crate::src::core::SkMipmap::SkMipmap>,
) -> crate::src::core::SkPaint::SkPaint {
    let mut paint = orig_paint.clone();
    let image = unsafe {
        crate::src::image::SkImage_Raster::SkImage_Raster::MakeFromBitmap(
            bitmap,
            crate::src::image::SkImage_Raster::SkCopyPixelsMode::kNever,
            mips,
        )
    }
    .expect("native draw bitmap has valid retained pixel backing");
    paint.setShader(image.makeShaderForPaint(
        orig_paint,
        crate::include::core::SkTileMode::SkTileMode::kClamp,
        crate::include::core::SkTileMode::SkTileMode::kClamp,
        sampling,
        matrix,
    ));
    paint
}
// Source: src/core/SkDraw.cpp:84-89
unsafe fn make_paint_with_image(
    orig_paint: &crate::src::core::SkPaint::SkPaint,
    bitmap: &crate::src::core::SkBitmap::SkBitmap,
    sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
    matrix: Option<&SkMatrix>,
) -> crate::src::core::SkPaint::SkPaint {
    unsafe { make_paint_with_image_and_mips(orig_paint, bitmap, sampling, matrix, None) }
}

// Source: src/core/SkDraw.cpp:91-91
impl Draw<'_> {
    pub fn new() -> Self {
        Self {
            fDst: crate::src::core::SkPixmap::SkPixmap::default(),
            fBlitterChooser: crate::src::core::SkBlitter::Choose,
            fCTM: None,
            fRC: None,
            fProps: None,
            fCtx: None,
        }
    }
}

// Source: src/core/SkDraw.cpp:93-111
// Native point-proc record; only the actual clip wrapper is default-constructed.
// Remaining native fields stay uninitialized until init() succeeds. UnsafeCell
// allows the owned AA wrapper's blitter to mutate during a const-record proc;
// a later chooser must lend its region/blitter fields disjointly, not freeze the
// complete wrapper behind a shared borrow. The record itself cannot move pinned.
type PtProc = unsafe fn(
    &PtProcRec<'_>,
    &[SkPoint],
    core::pin::Pin<&mut dyn crate::src::core::SkBlitter::SkBlitter>,
);
struct PtProcRec<'operation> {
    fMode: core::mem::MaybeUninit<crate::src::core::SkCanvas::PointMode>,
    fPaint: core::mem::MaybeUninit<&'operation crate::src::core::SkPaint::SkPaint>,
    fClip: core::mem::MaybeUninit<Option<core::ptr::NonNull<crate::src::core::SkRegion::SkRegion>>>,
    fRC: core::mem::MaybeUninit<&'operation crate::src::core::SkRasterClip::SkRasterClip>,
    fClipBounds: core::mem::MaybeUninit<crate::src::core::SkRect::SkRect>,
    fRadius: core::mem::MaybeUninit<f32>,
    fWrapper:
        core::cell::UnsafeCell<crate::src::core::SkRasterClip::SkAAClipBlitterWrapper<'operation>>,
    pinned: core::marker::PhantomPinned,
}
impl PtProcRec<'_> {
    fn new() -> Self {
        Self {
            fMode: core::mem::MaybeUninit::uninit(),
            fPaint: core::mem::MaybeUninit::uninit(),
            fClip: core::mem::MaybeUninit::uninit(),
            fRC: core::mem::MaybeUninit::uninit(),
            fClipBounds: core::mem::MaybeUninit::uninit(),
            fRadius: core::mem::MaybeUninit::uninit(),
            fWrapper: core::cell::UnsafeCell::new(
                crate::src::core::SkRasterClip::SkAAClipBlitterWrapper::new(),
            ),
            pinned: core::marker::PhantomPinned,
        }
    }
}

// Source: src/core/SkDraw.cpp:113-148
// Safety: initialized rec.fClip, and live exclusive destination/clip/blitter lease.
unsafe fn bw_pt_hair_proc(
    rec: &PtProcRec<'_>,
    dev_pts: &[SkPoint],
    mut blitter: core::pin::Pin<&mut dyn crate::src::core::SkBlitter::SkBlitter>,
) {
    let direct = blitter.as_mut().canDirectBlit();
    let clip = unsafe { rec.fClip.assume_init_ref().unwrap().as_ref() };
    if direct.is_some() && clip.is_rect() {
        let cr = *clip.get_bounds();
        let direct = direct.unwrap();
        let pm = direct.pm;
        let v = direct.value;
        macro_rules! direct_blit_loop {
            ($method:ident,$value:expr) => {{
                for p in dev_pts {
                    let x = crate::src::core::SkScalar::SkScalarFloorToInt(p.fX);
                    let y = crate::src::core::SkScalar::SkScalarFloorToInt(p.fY);
                    if cr.contains_xy(x, y) {
                        unsafe {
                            pm.$method(x, y).write($value);
                        }
                    }
                }
            }};
        }
        match pm.info().bytesPerPixel() {
            1 => direct_blit_loop!(writable_addr8, v as u8),
            2 => direct_blit_loop!(writable_addr16, v as u16),
            4 => direct_blit_loop!(writable_addr32, v as u32),
            8 => direct_blit_loop!(writable_addr64, v),
            _ => {
                crate::SkASSERT!(@source b"src/core/SkDraw.cpp\0".as_ptr(), 137; @condition b"false\0".as_ptr(); false)
            }
        }
    } else {
        for p in dev_pts {
            let x = crate::src::core::SkScalar::SkScalarFloorToInt(p.fX);
            let y = crate::src::core::SkScalar::SkScalarFloorToInt(p.fY);
            if clip.contains_xy(x, y) {
                unsafe {
                    blitter.as_mut().blitH(x, y, 1);
                }
            }
        }
    }
}

// Source: src/core/SkDraw.cpp:150-155
// Safety: rec.init succeeded; the blitter and bounded clip retain their pixel lease.
unsafe fn bw_line_hair_proc(
    rec: &PtProcRec<'_>,
    dev_pts: &[SkPoint],
    mut blitter: core::pin::Pin<&mut dyn crate::src::core::SkBlitter::SkBlitter>,
) {
    let clip = unsafe { rec.fRC.assume_init_ref() };
    let mut i = 0;
    while i + 1 < dev_pts.len() {
        unsafe {
            crate::src::core::SkScan::SkScan::HairLine(&dev_pts[i..i + 2], clip, blitter.as_mut());
        }
        i += 2;
    }
}

// Source: src/core/SkDraw.cpp:157-160
// Safety: rec.init succeeded; the blitter and bounded clip retain their pixel lease.
unsafe fn bw_poly_hair_proc(
    rec: &PtProcRec<'_>,
    dev_pts: &[SkPoint],
    mut blitter: core::pin::Pin<&mut dyn crate::src::core::SkBlitter::SkBlitter>,
) {
    let clip = unsafe { rec.fRC.assume_init_ref() };
    unsafe {
        crate::src::core::SkScan::SkScan::HairLine(dev_pts, clip, blitter.as_mut());
    }
}

// Source: src/core/SkDraw.cpp:164-169
// Safety: rec.init succeeded; the blitter and bounded clip retain their pixel lease.
unsafe fn aa_line_hair_proc(
    rec: &PtProcRec<'_>,
    dev_pts: &[SkPoint],
    mut blitter: core::pin::Pin<&mut dyn crate::src::core::SkBlitter::SkBlitter>,
) {
    let clip = unsafe { rec.fRC.assume_init_ref() };
    let mut i = 0;
    while i + 1 < dev_pts.len() {
        unsafe {
            crate::src::core::SkScan::SkScan::AntiHairLine(
                &dev_pts[i..i + 2],
                clip,
                blitter.as_mut(),
            );
        }
        i += 2;
    }
}

// Source: src/core/SkDraw.cpp:171-174
// Safety: rec.init succeeded; the blitter and bounded clip retain their pixel lease.
unsafe fn aa_poly_hair_proc(
    rec: &PtProcRec<'_>,
    dev_pts: &[SkPoint],
    mut blitter: core::pin::Pin<&mut dyn crate::src::core::SkBlitter::SkBlitter>,
) {
    let clip = unsafe { rec.fRC.assume_init_ref() };
    unsafe {
        crate::src::core::SkScan::SkScan::AntiHairLine(dev_pts, clip, blitter.as_mut());
    }
}

// Source: src/core/SkDraw.cpp:178-183
fn make_square_rad(center: SkPoint, radius: f32) -> crate::src::core::SkRect::SkRect {
    crate::src::core::SkRect::SkRect {
        fLeft: center.fX - radius,
        fTop: center.fY - radius,
        fRight: center.fX + radius,
        fBottom: center.fY + radius,
    }
}

// Source: src/core/SkDraw.cpp:185-191
fn make_xrect(r: &crate::src::core::SkRect::SkRect) -> crate::src::core::SkScan::SkXRect {
    use crate::include::private::SkFixed::SkScalarToFixed;
    crate::SkASSERT!(@source b"src/core/SkDraw.cpp\0".as_ptr(), 186; @condition b"SkRectPriv::FitsInFixed(r)\0".as_ptr(); crate::src::core::SkRectPriv::SkRectPriv::FitsInFixed(r));
    crate::src::core::SkScan::SkXRect {
        fLeft: SkScalarToFixed(r.fLeft),
        fTop: SkScalarToFixed(r.fTop),
        fRight: SkScalarToFixed(r.fRight),
        fBottom: SkScalarToFixed(r.fBottom),
    }
}

// Source: src/core/SkDraw.cpp:193-201
// Safety: initialized native proc record and exclusive live bounded blitter lease.
unsafe fn bw_square_proc(
    rec: &PtProcRec<'_>,
    dev_pts: &[SkPoint],
    mut blitter: core::pin::Pin<&mut dyn crate::src::core::SkBlitter::SkBlitter>,
) {
    for p in dev_pts {
        let mut r = make_square_rad(*p, unsafe { rec.fRadius.assume_init() });
        if r.intersect(unsafe { rec.fClipBounds.assume_init_ref() }) {
            unsafe {
                crate::src::core::SkScan::SkScan::FillXRectRasterClip(
                    &make_xrect(&r),
                    rec.fRC.assume_init_ref(),
                    blitter.as_mut(),
                );
            }
        }
    }
}

// Source: src/core/SkDraw.cpp:203-211
// Safety: initialized native proc record and exclusive live bounded blitter lease.
unsafe fn aa_square_proc(
    rec: &PtProcRec<'_>,
    dev_pts: &[SkPoint],
    mut blitter: core::pin::Pin<&mut dyn crate::src::core::SkBlitter::SkBlitter>,
) {
    for p in dev_pts {
        let mut r = make_square_rad(*p, unsafe { rec.fRadius.assume_init() });
        if r.intersect(unsafe { rec.fClipBounds.assume_init_ref() }) {
            unsafe {
                crate::src::core::SkScan::SkScan::AntiFillXRectRasterClip(
                    &make_xrect(&r),
                    rec.fRC.assume_init_ref(),
                    blitter.as_mut(),
                );
            }
        }
    }
}

// Source: src/core/SkDraw.cpp:213-251
impl<'operation> PtProcRec<'operation> {
    fn init(
        self: core::pin::Pin<&mut Self>,
        mode: crate::src::core::SkCanvas::PointMode,
        paint: &'operation crate::src::core::SkPaint::SkPaint,
        matrix: &SkMatrix,
        rc: &'operation crate::src::core::SkRasterClip::SkRasterClip,
    ) -> bool {
        use crate::src::core::SkCanvas::PointMode;
        if (mode as u32) > (PointMode::kPolygon_PointMode as u32) {
            return false;
        }
        if paint.getPathEffect().is_some() || paint.getMaskFilter().is_some() {
            return false;
        }
        let width = paint.getStrokeWidth();
        let mut radius = -1.0;
        if width == 0.0 {
            radius = 0.5;
        } else if paint.getStrokeCap() != crate::src::core::SkPaint::Cap::kRound_Cap
            && matrix.isScaleTranslate()
            && mode == PointMode::kPoints_PointMode
        {
            let sx = matrix.get(SkMatrix::kMScaleX);
            let sy = matrix.get(SkMatrix::kMScaleY);
            if crate::src::core::SkScalar::SkScalarNearlyZero(
                sx - sy,
                crate::src::core::SkScalar::SK_ScalarNearlyZero,
            ) {
                radius = (width * sx.abs()) / 2.0;
            }
        }
        if radius > 0.0 {
            let clip_bounds = crate::src::core::SkRect::SkRect::Make(rc.get_bounds());
            if !crate::src::core::SkRectPriv::SkRectPriv::FitsInFixed(&clip_bounds) {
                return false;
            }
            let this = unsafe { self.get_unchecked_mut() };
            this.fMode.write(mode);
            this.fPaint.write(paint);
            this.fClip.write(None);
            this.fRC.write(rc);
            this.fClipBounds.write(clip_bounds);
            this.fRadius.write(radius);
            return true;
        }
        false
    }
}

// Source: src/core/SkDraw.cpp:253-293
impl<'operation> PtProcRec<'operation> {
    // Safety: init succeeded. The supplied pointer denotes an exclusive blitter
    // valid for 'operation; the selected pointer/proc may be used only while this
    // pinned record and its initialized wrapper live, without reinitialization.
    unsafe fn chooseProc(
        self: core::pin::Pin<&mut Self>,
        blitter_ptr: &mut core::ptr::NonNull<
            dyn crate::src::core::SkBlitter::SkBlitter + 'operation,
        >,
    ) -> Option<PtProc> {
        let this = unsafe { self.get_unchecked_mut() };
        let mut proc: Option<PtProc> = None;
        let rc = *unsafe { this.fRC.assume_init_ref() };
        if rc.is_bw() {
            this.fClip
                .write(Some(core::ptr::NonNull::from(rc.bw_rgn())));
        } else {
            let mut wrapper = core::pin::Pin::new_unchecked(this.fWrapper.get_mut());
            unsafe {
                wrapper
                    .as_mut()
                    .init(rc, core::pin::Pin::new_unchecked(blitter_ptr.as_mut()));
            }
            let (region, blitter) = wrapper.parts();
            this.fClip.write(Some(core::ptr::NonNull::from(region)));
            *blitter_ptr = core::ptr::NonNull::from(core::pin::Pin::get_unchecked_mut(blitter));
        }
        use crate::src::core::SkCanvas::PointMode;
        crate::SkASSERT!(@source b"src/core/SkDraw.cpp\0".as_ptr(), 267; @condition b"0 == SkCanvas::kPoints_PointMode\0".as_ptr(); (PointMode::kPoints_PointMode as usize) == (0));
        crate::SkASSERT!(@source b"src/core/SkDraw.cpp\0".as_ptr(), 268; @condition b"1 == SkCanvas::kLines_PointMode\0".as_ptr(); (PointMode::kLines_PointMode as usize) == (1));
        crate::SkASSERT!(@source b"src/core/SkDraw.cpp\0".as_ptr(), 269; @condition b"2 == SkCanvas::kPolygon_PointMode\0".as_ptr(); (PointMode::kPolygon_PointMode as usize) == (2));
        let mode = unsafe { this.fMode.assume_init() } as usize;
        crate::SkASSERT!(@source b"src/core/SkDraw.cpp\0".as_ptr(), 270; @condition b"(unsigned)fMode <= (unsigned)SkCanvas::kPolygon_PointMode\0".as_ptr(); mode<=PointMode::kPolygon_PointMode as usize);
        let paint = *unsafe { this.fPaint.assume_init_ref() };
        if paint.isAntiAlias() {
            if paint.getStrokeWidth() == 0.0 {
                let procs: [PtProc; 3] = [aa_square_proc, aa_line_hair_proc, aa_poly_hair_proc];
                proc = Some(procs[mode]);
            } else if paint.getStrokeCap() != crate::src::core::SkPaint::Cap::kRound_Cap {
                crate::SkASSERT!(@source b"src/core/SkDraw.cpp\0".as_ptr(), 279; @condition b"SkCanvas::kPoints_PointMode == fMode\0".as_ptr(); (mode) == (PointMode::kPoints_PointMode as usize));
                proc = Some(aa_square_proc);
            }
        } else {
            if unsafe { this.fRadius.assume_init() } <= 0.5 {
                let procs: [PtProc; 3] = [bw_pt_hair_proc, bw_line_hair_proc, bw_poly_hair_proc];
                proc = Some(procs[mode]);
            } else {
                proc = Some(bw_square_proc);
            }
        }
        proc
    }
}

// Source: src/core/SkDraw.cpp:297-348
impl Draw<'_> {
    /// # Safety
    /// Preserve Draw's exclusive pixel/clip/context/chooser lease. Any optional
    /// device must carry that same exclusive native dispatch lease for this call.
    pub unsafe fn drawPoints(
        &self,
        mode: crate::src::core::SkCanvas::PointMode,
        mut points: &[SkPoint],
        paint: &crate::src::core::SkPaint::SkPaint,
        device: Option<core::ptr::NonNull<dyn crate::src::core::SkDevice::SkDeviceVirtual + '_>>,
    ) -> Result<(), crate::RasterError> {
        use crate::src::core::SkCanvas::PointMode;
        if mode == PointMode::kLines_PointMode {
            points = &points[..points.len() & !1];
        }
        #[cfg(sk_debug)]
        {
            self.validate();
        };
        let rc = self.fRC.expect("native Draw requires clip");
        if points.is_empty() || rc.is_empty() {
            return Ok(());
        }
        let mut chooser =
            std::pin::pin!(crate::src::core::SkAutoBlitterChoose::SkAutoBlitterChoose::new());
        let mut rec = Box::pin(PtProcRec::new());
        if device.is_none()
            && rec.as_mut().init(
                mode,
                paint,
                self.fCTM.expect("native Draw requires CTM"),
                rc,
            )
        {
            let mut blitter = unsafe {
                chooser.as_mut().choose(
                    self,
                    None,
                    paint,
                    &crate::src::core::SkRect::SkRect::MakeEmpty(),
                    crate::src::core::SkDrawTypes::SkDrawCoverage::kNo,
                )
            }?
            .expect("native Choose supplies a blitter");
            // Move only the Pin<Box> owner, never the pinned record allocation. Its
            // wrapper now drops before the arena on success, nonfinite exit or Err.
            let mut rec = rec;
            let mut dev_pts: [core::mem::MaybeUninit<SkPoint>; 32] =
                [core::mem::MaybeUninit::uninit(); 32];
            let proc = unsafe { rec.as_mut().chooseProc(&mut blitter) }
                .expect("native initialized point proc");
            let backup = usize::from(mode == PointMode::kPolygon_PointMode);
            let mut count = points.len();
            let mut offset = 0;
            loop {
                let n = count.min(32);
                let matrix = self.fCTM.unwrap();
                unsafe {
                    (matrix.getMapPtsProc())(
                        matrix,
                        dev_pts.as_mut_ptr().cast::<SkPoint>(),
                        points.as_ptr().add(offset),
                        n,
                    );
                }
                let mapped =
                    unsafe { core::slice::from_raw_parts(dev_pts.as_ptr().cast::<SkPoint>(), n) };
                let scalars =
                    unsafe { core::slice::from_raw_parts(mapped.as_ptr().cast::<f32>(), n * 2) };
                if !crate::src::core::SkFloatingPoint::SkIsFinite(scalars) {
                    return Ok(());
                }
                unsafe {
                    proc(
                        rec.as_ref().get_ref(),
                        mapped,
                        core::pin::Pin::new_unchecked(blitter.as_mut()),
                    );
                }
                offset += n - backup;
                crate::SkASSERT!(@source b"src/core/SkDraw.cpp\0".as_ptr(), 339; @condition b"count >= n\0".as_ptr(); count>=n);
                count -= n;
                if count > 0 {
                    count += backup;
                } else {
                    break;
                }
            }
        } else {
            unsafe { self.drawDevicePoints(mode, points, paint, device) }?;
        }
        Ok(())
    }
}

// Source: src/core/SkDraw.cpp:350-355
fn clipped_out(
    matrix: &SkMatrix,
    clip: &crate::src::core::SkRasterClip::SkRasterClip,
    src: &crate::src::core::SkRect::SkRect,
) -> bool {
    let mut dst = crate::src::core::SkRect::SkRect::default();
    matrix.mapRect(&mut dst, src);
    clip.quick_reject(&dst.roundOut())
}

// Source: src/core/SkDraw.cpp:357-362
fn clipped_out_dimensions(
    matrix: &SkMatrix,
    clip: &crate::src::core::SkRasterClip::SkRasterClip,
    width: i32,
    height: i32,
) -> bool {
    let mut rect = crate::src::core::SkRect::SkRect::default();
    rect.setIWH(width, height);
    clipped_out(matrix, clip, &rect)
}

// Source: src/core/SkDraw.cpp:364-366
fn clipHandlesSprite(
    clip: &crate::src::core::SkRasterClip::SkRasterClip,
    x: i32,
    y: i32,
    pixmap: &crate::src::core::SkPixmap::SkPixmap,
) -> bool {
    clip.is_bw()
        || clip.quick_contains(&crate::src::core::SkRect::SkIRect::make_xywh(
            x,
            y,
            pixmap.width(),
            pixmap.height(),
        ))
}

// Source: src/core/SkDraw.cpp:368-442
impl Draw<'_> {
    /// Safety: source backing and destination pixel leases remain valid and do
    /// not conflict throughout sprite blitting, shader creation and rectangle draw.
    pub unsafe fn drawBitmap(
        &self,
        bitmap: &crate::src::core::SkBitmap::SkBitmap,
        prematrix: &SkMatrix,
        dst_bounds: Option<&crate::src::core::SkRect::SkRect>,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        orig_paint: &crate::src::core::SkPaint::SkPaint,
        mips: Option<crate::src::core::SkMipmap::SkMipmap>,
    ) -> Result<(), crate::RasterError> {
        use crate::src::core::SkRect::{SkIRect, SkRect};
        #[cfg(sk_debug)]
        self.validate();
        let clip = self.fRC.expect("native configured raster clip");
        if clip.is_empty()
            || bitmap.width() == 0
            || bitmap.height() == 0
            || bitmap.colorType()
                == crate::include::core::SkColorType::SkColorType::kUnknown_SkColorType
        {
            return Ok(());
        }
        // Eager owned copy preserves the native copy-on-first-write paint state.
        let mut paint = orig_paint.clone();
        if paint.getStyle() != crate::src::core::SkPaint::Style::kFill_Style {
            paint.setStyle(crate::src::core::SkPaint::Style::kFill_Style);
        }
        let matrix = SkMatrix::Concat(self.fCTM.expect("native configured CTM"), prematrix);
        if clipped_out_dimensions(&matrix, clip, bitmap.width(), bitmap.height()) {
            return Ok(());
        }
        if !crate::src::core::SkImageInfoPriv::SkColorTypeIsAlphaOnly(bitmap.colorType())
            && crate::src::core::SkMatrix::SkTreatAsSprite(
                &matrix,
                &bitmap.dimensions(),
                sampling,
                paint.isAntiAlias(),
            )
        {
            let mut pixmap = crate::src::core::SkPixmap::SkPixmap::default();
            if !bitmap.peekPixels(Some(&mut pixmap)) {
                return Ok(());
            }
            let x = crate::include::core::SkScalar::SkScalarRoundToInt(matrix.getTranslateX());
            let y = crate::include::core::SkScalar::SkScalarRoundToInt(matrix.getTranslateY());
            if clipHandlesSprite(clip, x, y, &pixmap) {
                let mut allocator =
                    core::pin::pin!(crate::src::core::SkArenaAlloc::SkSTArenaAlloc::<
                        { crate::src::core::SkDrawTypes::kSkBlitterContextSize },
                    >::new(0));
                let arena = allocator.as_mut().pinned_arena();
                if let Some(mut blitter) = unsafe {
                    crate::src::core::SkBlitter_Sprite::ChooseSprite(
                        &self.fDst,
                        &paint,
                        &pixmap,
                        x,
                        y,
                        arena,
                        clip.clip_shader(),
                    )
                }? {
                    unsafe {
                        crate::src::core::SkScan::SkScan::FillIRectRasterClip(
                            &SkIRect::make_xywh(x, y, pixmap.width(), pixmap.height()),
                            clip,
                            core::pin::Pin::new_unchecked(blitter.as_mut()),
                        )
                    };
                    return Ok(());
                }
            }
        }
        let draw = Draw {
            fDst: self.fDst.clone(),
            fBlitterChooser: self.fBlitterChooser,
            fCTM: Some(&matrix),
            fRC: self.fRC,
            fProps: self.fProps,
            fCtx: self.fCtx,
        };
        // Selected upstream configuration leaves SK_SUPPORT_LEGACY_ALPHA_BITMAP_AS_COVERAGE undefined.
        let paint_with_shader =
            unsafe { make_paint_with_image_and_mips(&paint, bitmap, sampling, None, mips) };
        let src_bounds = SkRect::MakeIWH(bitmap.width(), bitmap.height());
        if let Some(dst_bounds) = dst_bounds {
            unsafe {
                self.drawRect(
                    &src_bounds,
                    &paint_with_shader,
                    Some(prematrix),
                    Some(dst_bounds),
                )
            }?;
        } else {
            unsafe { draw.drawRectSimple(&src_bounds, &paint_with_shader) }?;
        }
        Ok(())
    }
}

// Source: src/core/SkDraw.cpp:444-492
impl Draw<'_> {
    /// Safety: destination and source backing obey the Draw pixel lease throughout
    /// real sprite blitting and any fallback shader/rectangle execution.
    pub unsafe fn drawSprite(
        &self,
        bitmap: &crate::src::core::SkBitmap::SkBitmap,
        x: i32,
        y: i32,
        orig_paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        use crate::src::core::SkRect::{SkIRect, SkRect};
        #[cfg(sk_debug)]
        self.validate();
        let clip = self.fRC.expect("native configured raster clip");
        if clip.is_empty()
            || bitmap.width() == 0
            || bitmap.height() == 0
            || bitmap.colorType()
                == crate::include::core::SkColorType::SkColorType::kUnknown_SkColorType
        {
            return Ok(());
        }
        let bounds = SkIRect::make_xywh(x, y, bitmap.width(), bitmap.height());
        if clip.quick_reject(&bounds) {
            return Ok(());
        }
        let mut paint = orig_paint.clone();
        paint.setStyle(crate::src::core::SkPaint::Style::kFill_Style);
        let mut pixmap = crate::src::core::SkPixmap::SkPixmap::default();
        if !bitmap.peekPixels(Some(&mut pixmap)) {
            return Ok(());
        }
        if paint.getColorFilter().is_none() && clipHandlesSprite(clip, x, y, &pixmap) {
            let mut allocator = core::pin::pin!(crate::src::core::SkArenaAlloc::SkSTArenaAlloc::<
                { crate::src::core::SkDrawTypes::kSkBlitterContextSize },
            >::new(0));
            if let Some(mut blitter) = unsafe {
                crate::src::core::SkBlitter_Sprite::ChooseSprite(
                    &self.fDst,
                    &paint,
                    &pixmap,
                    x,
                    y,
                    allocator.as_mut().pinned_arena(),
                    clip.clip_shader(),
                )
            }? {
                unsafe {
                    crate::src::core::SkScan::SkScan::FillIRectRasterClip(
                        &bounds,
                        clip,
                        core::pin::Pin::new_unchecked(blitter.as_mut()),
                    )
                };
                return Ok(());
            }
        }
        let rect = SkRect::Make(&bounds);
        let matrix = SkMatrix::Translate(rect.fLeft, rect.fTop);
        let paint_with_shader = unsafe {
            make_paint_with_image(
                &paint,
                bitmap,
                &crate::include::core::SkSamplingOptions::SkSamplingOptions::new(),
                Some(&matrix),
            )
        };
        let draw = Draw {
            fDst: self.fDst.clone(),
            fBlitterChooser: self.fBlitterChooser,
            fCTM: Some(SkMatrix::I()),
            fRC: self.fRC,
            fProps: self.fProps,
            fCtx: self.fCtx,
        };
        unsafe { draw.drawRect(&rect, &paint_with_shader, None, None) }?;
        Ok(())
    }
}

// Source: src/core/SkDraw.cpp:494-524
impl Draw<'_> {
    /// Safety: source mask storage is live with the native format/layout; the
    /// destination lease and chooser/arena contracts hold through all blitting.
    pub unsafe fn drawDevMask(
        &self,
        source: &crate::src::core::SkMask::SkMask,
        paint: &crate::src::core::SkPaint::SkPaint,
        paint_matrix: Option<&SkMatrix>,
    ) -> Result<(), crate::RasterError> {
        if source.fBounds.is_empty() {
            return Ok(());
        }
        let mut destination = crate::src::core::SkMask::SkMaskBuilder::new();
        let filtered = if let Some(filter) = paint.getMaskFilter() {
            unsafe {
                filter.filterMask(
                    &mut destination,
                    source,
                    self.fCTM.expect("native configured CTM"),
                    None,
                )
            }
        } else {
            Ok(false)
        };
        let _free = unsafe {
            crate::src::core::SkMask::SkAutoMaskFreeImage::from_raw(destination.fImage as *mut u8)
        };
        let filtered = filtered?;
        let mask = if filtered { &destination.mask } else { source };
        let mut chooser =
            core::pin::pin!(crate::src::core::SkAutoBlitterChoose::SkAutoBlitterChoose::new());
        let mut blitter = unsafe {
            chooser.as_mut().choose(
                self,
                paint_matrix,
                paint,
                &crate::src::core::SkRect::SkRect::Make(&destination.fBounds),
                crate::src::core::SkDrawTypes::SkDrawCoverage::kNo,
            )
        }?
        .expect("native Choose supplies a blitter");
        let clip = self.fRC.expect("native configured raster clip");
        if clip.is_bw() {
            unsafe {
                core::pin::Pin::new_unchecked(blitter.as_mut()).blitMaskRegion(mask, clip.bw_rgn());
            }
        } else {
            let mut wrapper =
                crate::src::core::SkRasterClip::SkAAClipBlitterWrapper::from_clip(clip, unsafe {
                    core::pin::Pin::new_unchecked(blitter.as_mut())
                });
            let mut wrapper = core::pin::pin!(wrapper);
            let (region, mut blitter) = wrapper.as_mut().parts();
            unsafe {
                blitter.as_mut().blitMaskRegion(mask, region);
            }
        }
        Ok(())
    }
}

// Source: src/core/SkDraw.cpp:526-609
impl Draw<'_> {
    /// Safety: live source A8 backing, exclusive destination pixels and all native
    /// shader/filter leases hold until the temporary Canvas and its device drop.
    pub unsafe fn drawBitmapAsMask(
        &self,
        bitmap: &crate::src::core::SkBitmap::SkBitmap,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: &crate::src::core::SkPaint::SkPaint,
        paint_matrix: Option<&SkMatrix>,
    ) -> Result<(), crate::RasterError> {
        use crate::src::core::SkMask::{Format, SkMask, SkMaskBuilder};
        use crate::src::core::SkRect::{SkIRect, SkRect};
        crate::SkASSERT!(@source b"src/core/SkDraw.cpp\0".as_ptr(), 530; @condition b"bitmap.colorType() == kAlpha_8_SkColorType\0".as_ptr(); (bitmap.colorType()) == (crate::include::core::SkColorType::SkColorType::kAlpha_8_SkColorType));
        let clip = self.fRC.expect("native configured raster clip");
        if clip.is_empty() {
            return Ok(());
        }
        let matrix = self.fCTM.expect("native configured CTM");
        if crate::src::core::SkMatrix::SkTreatAsSprite(
            matrix,
            &bitmap.dimensions(),
            sampling,
            paint.isAntiAlias(),
        ) {
            let x = crate::include::core::SkScalar::SkScalarRoundToInt(matrix.getTranslateX());
            let y = crate::include::core::SkScalar::SkScalarRoundToInt(matrix.getTranslateY());
            let mut pixmap = crate::src::core::SkPixmap::SkPixmap::default();
            if !bitmap.peekPixels(Some(&mut pixmap)) {
                return Ok(());
            }
            let mask = SkMask::new(
                pixmap.addr8_xy(0, 0),
                SkIRect::make_xywh(x, y, pixmap.width(), pixmap.height()),
                pixmap.rowBytes() as u32,
                Format::kA8_Format,
            );
            unsafe { self.drawDevMask(&mask, paint, paint_matrix) }?;
        } else {
            let mut rect = SkRect::MakeIWH(bitmap.width(), bitmap.height());
            let input = rect;
            matrix.mapRect(&mut rect, &input);
            let mut mask = SkMaskBuilder::new();
            mask.mask.fBounds = rect.round();
            crate::SkASSERT!(@source b"src/core/SkDraw.cpp\0".as_ptr(), 563; @condition b"fDst.bounds().contains(fRC->getBounds())\0".as_ptr(); self.fDst.bounds().contains_rect(clip.get_bounds()));
            let mut device_bounds = self.fDst.bounds();
            device_bounds.intersect(&clip.get_bounds().make_outset(1, 1));
            if !mask.mask.fBounds.intersect(&device_bounds) {
                return Ok(());
            }
            mask.mask.fFormat = Format::kA8_Format;
            mask.mask.fRowBytes = (mask.fBounds.width() as u32).wrapping_add(3) & !3;
            let size = mask.computeImageSize();
            if size == 0 {
                return Ok(());
            }
            let mut storage =
                crate::include::private::SkTemplates::AutoTMalloc::<u8>::with_count(size);
            mask.mask.fImage = storage.data_mut();
            unsafe {
                core::ptr::write_bytes(storage.data_mut(), 0, size);
            }
            {
                let mut device = crate::src::core::SkBitmap::SkBitmap::default();
                unsafe {
                    device.installPixels(
                        &crate::src::core::SkImageInfo::SkImageInfo::MakeA8(
                            mask.fBounds.width(),
                            mask.fBounds.height(),
                        ),
                        storage.data_mut().cast(),
                        mask.fRowBytes as usize,
                        None,
                        core::ptr::null_mut(),
                    );
                }
                let mut canvas =
                    unsafe { crate::src::core::SkCanvas::SkCanvas::with_bitmap(&device) };
                unsafe {
                    canvas
                        .as_mut()
                        .translate(-(mask.fBounds.fLeft as f32), -(mask.fBounds.fTop as f32));
                    canvas.as_mut().concat(matrix);
                }
                let mut tmp_paint = crate::src::core::SkPaint::SkPaint::default();
                tmp_paint.setAntiAlias(paint.isAntiAlias());
                tmp_paint.setDither(paint.isDither());
                let paint_with_shader =
                    unsafe { make_paint_with_image(&tmp_paint, bitmap, sampling, None) };
                let bounds = SkRect::MakeIWH(bitmap.width(), bitmap.height());
                let canvas_raw =
                    unsafe { core::ptr::NonNull::from(canvas.as_mut().get_unchecked_mut()) };
                unsafe {
                    crate::src::core::SkCanvas::SkCanvas::drawRect_raw(
                        canvas_raw,
                        &bounds,
                        &paint_with_shader,
                    )
                }?;
            }
            unsafe { self.drawDevMask(&mask.mask, paint, paint_matrix) }?;
        }
        Ok(())
    }
}

// Source: src/core/SkDraw.cpp:613-626
// Adaptation: project the original Draw's two borrowed state inputs explicitly until
// its full canonical chooser/context contract is available; no second state owner.
fn compute_conservative_local_clip_bounds(
    clip: &crate::src::core::SkRasterClip::SkRasterClip,
    ctm: &SkMatrix,
    local_bounds: &mut crate::src::core::SkRect::SkRect,
) -> bool {
    if clip.is_empty() {
        return false;
    }
    if let Some(inverse) = ctm.invert() {
        let mut dev_bounds = *clip.get_bounds();
        dev_bounds.outset(1, 1);
        inverse.mapRect(
            local_bounds,
            &crate::src::core::SkRect::SkRect::Make(&dev_bounds),
        );
        return true;
    }
    false
}

// Source: src/core/SkDraw.cpp:630-642
impl Draw<'_> {
    /// # Safety
    /// Uphold Draw's destination pixel lease, exclusive access, and chooser/arena contract.
    pub unsafe fn drawPaint(
        &self,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        #[cfg(sk_debug)]
        self.validate();
        let clip = self.fRC.expect("native Draw requires fRC");
        if clip.is_empty() {
            return Ok(());
        }
        let mut dev_rect = crate::src::core::SkRect::SkIRect::default();
        dev_rect.set_wh(self.fDst.width(), self.fDst.height());
        // Inline arena is pinned before native choose can allocate from it.
        let mut blitter =
            std::pin::pin!(crate::src::core::SkAutoBlitterChoose::SkAutoBlitterChoose::new());
        let mut pointer = blitter
            .as_mut()
            .choose(
                self,
                None,
                paint,
                &crate::src::core::SkRect::SkRect::Make(&dev_rect),
                crate::src::core::SkDrawTypes::SkDrawCoverage::kNo,
            )?
            .expect("native Choose supplies a blitter");
        unsafe {
            crate::src::core::SkScan::SkScan::FillIRectRasterClip(
                &dev_rect,
                clip,
                core::pin::Pin::new_unchecked(pointer.as_mut()),
            );
        }
        Ok(())
    }
}

// Source: src/core/SkDraw.cpp:646-652
// Canonical Paint and Matrix single-vector APIs are present.
fn compute_stroke_size(paint: &crate::src::core::SkPaint::SkPaint, matrix: &SkMatrix) -> SkPoint {
    crate::SkASSERT!(@source b"src/core/SkDraw.cpp\0".as_ptr(), 647; @condition b"matrix.rectStaysRect()\0".as_ptr(); matrix.rectStaysRect());
    crate::SkASSERT!(@source b"src/core/SkDraw.cpp\0".as_ptr(), 648; @condition b"SkPaint::kFill_Style != paint.getStyle()\0".as_ptr(); paint.getStyle() != crate::src::core::SkPaint::Style::kFill_Style);
    let size = matrix.mapVector(SkPoint::make(
        paint.getStrokeWidth(),
        paint.getStrokeWidth(),
    ));
    SkPoint::make(size.fX.abs(), size.fY.abs())
}

// Source: src/core/SkDraw.cpp:654-665
fn easy_rect_join(
    rect: &crate::src::core::SkRect::SkRect,
    paint: &crate::src::core::SkPaint::SkPaint,
    matrix: &SkMatrix,
    strokeSize: &mut SkPoint,
) -> bool {
    if rect.isEmpty()
        || paint.getStrokeJoin() != crate::src::core::SkPaint::Join::kMiter_Join
        || paint.getStrokeMiter() < crate::include::core::SkScalar::SK_ScalarSqrt2
    {
        return false;
    }
    *strokeSize = compute_stroke_size(paint, matrix);
    true
}

// Source: src/core/SkDraw.cpp:667-693
// Static native classification: no alternate Paint or backend is introduced.
pub fn ComputeRectType(
    rect: &crate::src::core::SkRect::SkRect,
    paint: &crate::src::core::SkPaint::SkPaint,
    matrix: &SkMatrix,
    strokeSize: &mut SkPoint,
) -> RectType {
    use crate::src::core::SkPaint::Style;
    let width = paint.getStrokeWidth();
    let zeroWidth = width == 0.0;
    let mut style = paint.getStyle();
    if style == Style::kStrokeAndFill_Style && zeroWidth {
        style = Style::kFill_Style;
    }
    if paint.getPathEffect().is_some()
        || paint.getMaskFilter().is_some()
        || !matrix.rectStaysRect()
        || style == Style::kStrokeAndFill_Style
    {
        return RectType::kPath;
    }
    if style == Style::kFill_Style {
        return RectType::kFill;
    }
    if zeroWidth {
        return RectType::kHair;
    }
    if easy_rect_join(rect, paint, matrix, strokeSize) {
        return RectType::kStroke;
    }
    RectType::kPath
}

// Source: src/core/SkDraw.cpp:695-697
fn rect_points(r: &crate::src::core::SkRect::SkRect) -> &[SkPoint; 2] {
    // Both source-aligned structs are repr(C), with four/two consecutive f32 fields.
    // The view retains the original rectangle borrow and does not allocate or copy.
    unsafe { &*(core::ptr::from_ref(r).cast::<[SkPoint; 2]>()) }
}

// Source: src/core/SkDraw.cpp:700-700
fn rect_points_mut(r: &mut crate::src::core::SkRect::SkRect) -> &mut [SkPoint; 2] {
    // The exclusive view covers exactly the original rectangle's four floats.
    unsafe { &mut *(core::ptr::from_mut(r).cast::<[SkPoint; 2]>()) }
}

// Source: src/core/SkDraw.cpp:702-709
// Safety: caller preserves the destination lease across the cloned borrowed Draw.
unsafe fn draw_rect_as_path(
    orig: &Draw<'_>,
    rect: &crate::src::core::SkRect::SkRect,
    paint: &crate::src::core::SkPaint::SkPaint,
    ctm: &SkMatrix,
) -> Result<(), crate::RasterError> {
    let mut draw = orig.clone();
    draw.fCTM = Some(ctm);
    let path = crate::src::core::SkPath::SkPath::Rect(
        rect,
        crate::include::core::SkPathTypes::SkPathFillType::kWinding,
        crate::include::core::SkPathTypes::SkPathDirection::kCW,
        0,
    );
    draw.drawPath(&path, paint, None)?;
    Ok(())
}
// Source: src/core/SkDraw.cpp:711-804
impl Draw<'_> {
    /// # Safety
    /// Uphold Draw's destination pixel lease, exclusive access, and chooser/arena contract.
    pub unsafe fn drawRect(
        &self,
        pre_paint_rect: &crate::src::core::SkRect::SkRect,
        paint: &crate::src::core::SkPaint::SkPaint,
        paint_matrix: Option<&SkMatrix>,
        post_paint_rect: Option<&crate::src::core::SkRect::SkRect>,
    ) -> Result<(), crate::RasterError> {
        use crate::src::core::SkPaint::Style;
        use crate::src::core::SkRect::SkRect;
        use crate::src::core::SkScan::SkScan;
        #[cfg(sk_debug)]
        self.validate();
        let clip = self.fRC.expect("native Draw requires fRC");
        if clip.is_empty() {
            return Ok(());
        }
        let ctm = self.fCTM.expect("native Draw requires fCTM");
        // Local conditional storage realizes the source copy-on-first-write: clone
        // precisely when paintMatrix requires writable preConcat, otherwise borrow.
        let matrix_storage=paint_matrix.map(|pm|{
        crate::SkASSERT!(@source b"src/core/SkDraw.cpp\0".as_ptr(), 724; @condition b"postPaintRect\0".as_ptr(); post_paint_rect.is_some());let mut m=ctm.clone();m.preConcat(pm);m
    });
        if paint_matrix.is_none() {
            crate::SkASSERT!(@source b"src/core/SkDraw.cpp\0".as_ptr(), 727; @condition b"!postPaintRect\0".as_ptr(); post_paint_rect.is_none());
        }
        let matrix = matrix_storage.as_ref().unwrap_or(ctm);
        let mut stroke_size = SkPoint::default();
        let rtype = ComputeRectType(pre_paint_rect, paint, ctm, &mut stroke_size);
        if rtype == RectType::kPath {
            draw_rect_as_path(self, pre_paint_rect, paint, matrix)?;
            return Ok(());
        }
        let mut dev_rect = SkRect::default();
        let paint_rect = if paint_matrix.is_some() {
            post_paint_rect.expect("native paintMatrix requires postPaintRect")
        } else {
            pre_paint_rect
        };
        ctm.mapPoints(rect_points_mut(&mut dev_rect), rect_points(paint_rect));
        dev_rect.sort();
        let mut bbox = dev_rect;
        if paint.getStyle() != Style::kFill_Style {
            if paint.getStrokeWidth() == 0.0 {
                bbox.outset(1.0, 1.0);
            } else {
                let ssize = if rtype == RectType::kStroke {
                    stroke_size
                } else {
                    compute_stroke_size(paint, ctm)
                };
                bbox.outset(ssize.fX / 2.0, ssize.fY / 2.0);
            }
        }
        if crate::src::core::SkPathPriv::SkPathPriv::TooBigForMath(&bbox) {
            return Ok(());
        }
        if !crate::src::core::SkRectPriv::SkRectPriv::FitsInFixed(&bbox) && rtype != RectType::kHair
        {
            draw_rect_as_path(self, pre_paint_rect, paint, matrix)?;
            return Ok(());
        }
        let ir = bbox.roundOut();
        if clip.quick_reject(&ir) {
            return Ok(());
        }
        let mut storage =
            std::pin::pin!(crate::src::core::SkAutoBlitterChoose::SkAutoBlitterChoose::new());
        let mut pointer = storage
            .as_mut()
            .choose(
                self,
                Some(matrix),
                paint,
                &dev_rect,
                crate::src::core::SkDrawTypes::SkDrawCoverage::kNo,
            )?
            .expect("native Choose supplies a blitter");
        let mut blitter = unsafe { core::pin::Pin::new_unchecked(pointer.as_mut()) };
        match rtype {
            RectType::kFill => {
                if paint.isAntiAlias() {
                    SkScan::AntiFillRectRasterClip(&dev_rect, clip, blitter.as_mut());
                } else {
                    SkScan::FillRectRasterClip(&dev_rect, clip, blitter.as_mut());
                }
            }
            RectType::kStroke => {
                if paint.isAntiAlias() {
                    SkScan::AntiFrameRectRasterClip(
                        &dev_rect,
                        &stroke_size,
                        clip,
                        blitter.as_mut(),
                    );
                } else {
                    SkScan::FrameRect(&dev_rect, &stroke_size, clip, blitter.as_mut());
                }
            }
            RectType::kHair => {
                if paint.isAntiAlias() {
                    SkScan::AntiHairRect(&dev_rect, clip, blitter.as_mut());
                } else {
                    SkScan::HairRect(&dev_rect, clip, blitter.as_mut());
                }
            }
            _ => {
                crate::SkDEBUGFAIL!(@source b"src/core/SkDraw.cpp\0".as_ptr(), 802; b"bad rtype\0".as_ptr())
            }
        }
        Ok(())
    }
}

// Source: src/core/SkDraw.cpp:806-814
fn fast_len(vec: SkPoint) -> f32 {
    let mut x = vec.fX.abs();
    let mut y = vec.fY.abs();
    if x < y {
        core::mem::swap(&mut x, &mut y);
    }
    x + (y / 2.0)
}

// Source: src/core/SkDraw.cpp:816-837
// Option<&mut f32> retains the original nullable output and writes only on success.
pub fn DrawTreatAAStrokeAsHairline(
    strokeWidth: f32,
    matrix: &SkMatrix,
    coverage: Option<&mut f32>,
) -> bool {
    crate::SkASSERT!(@source b"src/core/SkDraw.cpp\0".as_ptr(), 817; @condition b"strokeWidth > 0\0".as_ptr(); strokeWidth > 0.0);
    if matrix.hasPerspective() {
        return false;
    }
    let src = [
        SkPoint::make(strokeWidth, 0.0),
        SkPoint::make(0.0, strokeWidth),
    ];
    let mut dst = [SkPoint::make(0.0, 0.0); 2];
    matrix.mapVectors(&mut dst, &src);
    let len0 = fast_len(dst[0]);
    let len1 = fast_len(dst[1]);
    if len0 <= 1.0 && len1 <= 1.0 {
        if let Some(coverage) = coverage {
            *coverage = sk_float_midpoint(len0, len1);
        }
        return true;
    }
    false
}

// Source: src/core/SkDraw.cpp:839-847
impl Draw<'_> {
    /// # Safety
    /// Uphold Draw's destination pixel lease, exclusive access, and chooser/arena contract.
    pub unsafe fn drawOval(
        &self,
        oval: &crate::src::core::SkRect::SkRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        #[cfg(sk_debug)]
        self.validate();
        if self.fRC.expect("native Draw requires fRC").is_empty() {
            return Ok(());
        }
        self.drawPath(
            &crate::src::core::SkPath::SkPath::Oval(
                oval,
                crate::include::core::SkPathTypes::SkPathDirection::kCW,
                0,
            ),
            paint,
            None,
        )?;
        Ok(())
    }
}
// Source: src/core/SkDraw.cpp:849-879
impl Draw<'_> {
    /// # Safety
    /// Uphold Draw's destination pixel lease, exclusive access, and chooser/arena contract.
    pub unsafe fn drawRRect(
        &self,
        rrect: &crate::src::core::SkRRect::SkRRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        #[cfg(sk_debug)]
        self.validate();
        if self.fRC.expect("native Draw requires fRC").is_empty() {
            return Ok(());
        }
        let mut coverage = 0.0;
        // Native goto DRAW_PATH becomes short-circuit eligibility for the same filter branch.
        let try_filter = !crate::src::core::SkDrawProcs::DrawTreatAsHairline(
            paint,
            self.fCTM.expect("native Draw requires fCTM"),
            &mut coverage,
        ) && paint.getPathEffect().is_none()
            && paint.getStyle() == crate::src::core::SkPaint::Style::kFill_Style;
        if try_filter && paint.getMaskFilter().is_some() && self.drawRRectNinePatch(rrect, paint)? {
            return Ok(());
        }
        self.drawPath(
            &crate::src::core::SkPath::SkPath::RRect(
                rrect,
                crate::include::core::SkPathTypes::SkPathDirection::kCW,
                0,
            ),
            paint,
            None,
        )?;
        Ok(())
    }
}
// Source: src/core/SkDraw.cpp:881-901
impl Draw<'_> {
    /// # Safety
    /// Uphold Draw's destination pixel lease, exclusive access, and chooser/arena contract.
    pub unsafe fn drawRRectNinePatch(
        &self,
        rrect: &crate::src::core::SkRRect::SkRRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<bool, crate::RasterError> {
        crate::SkASSERT!(@source b"src/core/SkDraw.cpp\0".as_ptr(), 882; @condition b"paint.getMaskFilter()\0".as_ptr(); paint.getMaskFilter().is_some());
        let ctm = self.fCTM.expect("native Draw requires fCTM");
        if let Some(rr) = rrect.transform(ctm) {
            let mut storage =
                std::pin::pin!(crate::src::core::SkAutoBlitterChoose::SkAutoBlitterChoose::new());
            let mut pointer = storage
                .as_mut()
                .choose(
                    self,
                    None,
                    paint,
                    rrect.getBounds(),
                    crate::src::core::SkDrawTypes::SkDrawCoverage::kNo,
                )?
                .expect("native Choose supplies a blitter");
            let filter = paint
                .getMaskFilter()
                .expect("native mask-filter precondition");
            let clip = self.fRC.expect("native Draw requires fRC");
            if rrect.getType() == crate::src::core::SkRRect::Type::kRect_Type {
                let dev_rect = *rr.rect();
                if unsafe {
                    filter.filterRects(
                        core::slice::from_ref(&dev_rect),
                        ctm,
                        clip,
                        core::pin::Pin::new_unchecked(pointer.as_mut()),
                        None,
                    )
                }? == crate::src::core::SkMaskFilterBase::FilterReturn::kTrue
                {
                    return Ok(true);
                }
            } else if unsafe {
                filter.filterRRect(
                    &rr,
                    ctm,
                    clip,
                    core::pin::Pin::new_unchecked(pointer.as_mut()),
                    None,
                )
            }? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

// Source: src/core/SkDraw.cpp:903-964
impl Draw<'_> {
    // Safety: caller upholds the Draw pixel lease and custom-blitter arena lifetime.
    unsafe fn drawDevPath(
        &self,
        raw: &crate::src::core::SkPathRaw::SkPathRaw<'_>,
        paint: &crate::src::core::SkPaint::SkPaint,
        coverage: crate::src::core::SkDrawTypes::SkDrawCoverage,
        custom: Option<core::pin::Pin<&mut dyn crate::src::core::SkBlitter::SkBlitter>>,
        do_fill: bool,
    ) -> Result<(), crate::RasterError> {
        use crate::src::core::SkPaint::Cap;
        use crate::src::core::SkScan::SkScan;
        if crate::src::core::SkPathPriv::SkPathPriv::TooBigForMath(&raw.bounds()) {
            return Ok(());
        }
        // Scope each actual mutable blitter lease independently: a caller-owned
        // custom trait-object lifetime must never extend the local arena lifetime.
        let render = |mut blitter: core::pin::Pin<
            &mut dyn crate::src::core::SkBlitter::SkBlitter,
        >|
         -> Result<(), crate::RasterError> {
            let ctm = self.fCTM.expect("native Draw requires fCTM");
            let clip = self.fRC.expect("native Draw requires fRC");
            if let Some(filter) = paint.getMaskFilter() {
                let style = if do_fill {
                    crate::src::core::SkStrokeRec::InitStyle::kFill_InitStyle
                } else {
                    crate::src::core::SkStrokeRec::InitStyle::kHairline_InitStyle
                };
                // Original filterPath dispatch may paint and finish; cache remains null.
                if unsafe { filter.filterPath(raw, ctm, clip, blitter.as_mut(), style, None) }? {
                    return Ok(());
                }
            }
            type PathProc = unsafe fn(
                &crate::src::core::SkPathRaw::SkPathRaw<'_>,
                &crate::src::core::SkRasterClip::SkRasterClip,
                core::pin::Pin<&mut dyn crate::src::core::SkBlitter::SkBlitter>,
            );
            let proc: PathProc = if do_fill {
                if paint.isAntiAlias() {
                    SkScan::AntiFillPathRasterClip
                } else {
                    SkScan::FillPathRasterClip
                }
            } else if paint.isAntiAlias() {
                match paint.getStrokeCap() {
                    Cap::kButt_Cap => SkScan::AntiHairPath,
                    Cap::kSquare_Cap => SkScan::AntiHairSquarePath,
                    Cap::kRound_Cap => SkScan::AntiHairRoundPath,
                }
            } else {
                match paint.getStrokeCap() {
                    Cap::kButt_Cap => SkScan::HairPath,
                    Cap::kSquare_Cap => SkScan::HairSquarePath,
                    Cap::kRound_Cap => SkScan::HairRoundPath,
                }
            };
            unsafe {
                proc(raw, clip, blitter.as_mut());
            }
            Ok(())
        };
        if let Some(custom) = custom {
            render(custom)
        } else {
            let mut storage =
                std::pin::pin!(crate::src::core::SkAutoBlitterChoose::SkAutoBlitterChoose::new());
            let mut selected = storage
                .as_mut()
                .choose(self, None, paint, &raw.bounds(), coverage)?
                .expect("native Choose supplies a blitter");
            render(unsafe { core::pin::Pin::new_unchecked(selected.as_mut()) })
        }
    }
}

// Source: src/core/SkDraw.cpp:979-1006
// Canonical mixed Arc/sk_sp Paint clone preserves each native shared effect.
fn modifyPaintForHairlines(
    origPaint: &crate::src::core::SkPaint::SkPaint,
    matrix: &SkMatrix,
) -> Option<crate::src::core::SkPaint::SkPaint> {
    let mut coverage = 0.0;
    if crate::src::core::SkDrawProcs::DrawTreatAsHairline(origPaint, matrix, &mut coverage) {
        let bm = origPaint.asBlendMode();
        if coverage == 1.0 {
            let mut paint = origPaint.clone();
            paint.setStrokeWidth(0.0);
            return Some(paint);
        } else if bm.is_some_and(crate::src::core::SkBlendMode::SkBlendMode_SupportsCoverageAsAlpha)
        {
            let scale = (coverage * 256.0) as i32;
            let newAlpha = ((origPaint.getAlpha() as i32 * scale) >> 8) as u32;
            let mut paint = origPaint.clone();
            paint.setStrokeWidth(0.0);
            paint.setAlpha(newAlpha);
            return Some(paint);
        }
    }
    None
}

// Source: src/core/SkDraw.cpp:1008-1080
impl Draw<'_> {
    /// # Safety
    /// Uphold Draw's destination pixel lease, exclusive access, and chooser/arena contract.
    pub unsafe fn drawPathWithCoverage(
        &self,
        orig: &crate::src::core::SkPath::SkPath,
        orig_paint: &crate::src::core::SkPaint::SkPaint,
        pre_matrix: Option<&SkMatrix>,
        coverage: crate::src::core::SkDrawTypes::SkDrawCoverage,
        custom: Option<core::pin::Pin<&mut dyn crate::src::core::SkBlitter::SkBlitter>>,
    ) -> Result<(), crate::RasterError> {
        use crate::src::core::SkPathEnums::SkResolveConvexity;
        use crate::src::core::SkPathPriv::SkPathPriv;
        #[cfg(sk_debug)]
        self.validate();
        let clip = self.fRC.expect("native Draw requires fRC");
        if clip.is_empty() {
            return Ok(());
        }
        let ctm = self.fCTM.expect("native Draw requires fCTM");
        let new_paint = modifyPaintForHairlines(orig_paint, ctm);
        let paint = new_paint.as_ref().unwrap_or(orig_paint);
        let needs_fill = paint.getPathEffect().is_some()
            || paint.getStyle() != crate::src::core::SkPaint::Style::kFill_Style;
        let mut builder = crate::src::core::SkPathBuilder::SkPathBuilder::new();
        let pdata;
        let raw;
        let mut do_fill = true;
        if needs_fill {
            let mut cull = crate::src::core::SkRect::SkRect::default();
            let cull_ptr = if compute_conservative_local_clip_bounds(clip, ctm, &mut cull) {
                Some(&cull)
            } else {
                None
            };
            let pre_storage;
            let mut path = orig;
            if let Some(pre) = pre_matrix {
                pre_storage = match path.tryMakeTransform(pre) {
                    Some(p) => p,
                    None => return Ok(()),
                };
                path = &pre_storage;
            }
            do_fill = crate::src::core::SkPathUtils::FillPathWithPaint(
                path,
                paint,
                &mut builder,
                cull_ptr,
                ctm,
            );
            builder.transform(ctm);
            raw = SkPathPriv::RawBuilder(&builder, SkResolveConvexity::kYes);
        } else {
            let mut matrix = ctm.clone();
            if let Some(pre) = pre_matrix {
                matrix.preConcat(pre);
            }
            if matrix.isIdentity() {
                raw = SkPathPriv::Raw(orig, SkResolveConvexity::kYes);
            } else {
                let original = SkPathPriv::Raw(orig, SkResolveConvexity::kNo);
                pdata = match original.and_then(|r| {
                    crate::src::core::SkPathData::SkPathData::MakeTransform(&r, &matrix)
                }) {
                    Some(p) => p,
                    None => return Ok(()),
                };
                raw = Some(pdata.raw(orig.getFillType(), SkResolveConvexity::kYes));
            }
        }
        let Some(raw) = raw else { return Ok(()) };
        // Native SK_BUILD_FOR_FUZZER points-count shortcut is inactive in CPU production.
        self.drawDevPath(&raw, paint, coverage, custom, do_fill)?;
        Ok(())
    }
}

// Source: src/core/SkDraw.cpp:1086-1096
impl Draw<'_> {
    #[cfg(sk_debug)]
    // Source: src/core/SkDraw.h:257-257
    pub(crate) fn validate(&self) {
        crate::SkASSERT!(@source b"src/core/SkDraw.cpp\0".as_ptr(), 1087; @condition b"fCTM != nullptr\0".as_ptr(); self.fCTM.is_some());
        crate::SkASSERT!(@source b"src/core/SkDraw.cpp\0".as_ptr(), 1088; @condition b"fRC != nullptr\0".as_ptr(); self.fRC.is_some());
        let cr = self.fRC.expect("native Draw requires fRC").get_bounds();
        let mut br = crate::src::core::SkRect::SkIRect::default();
        br.set_wh(self.fDst.width(), self.fDst.height());
        debug_assert!(cr.is_empty() || br.contains_rect(cr));
    }
}

// Source: src/core/SkDraw.cpp:1101-1132
fn compute_mask_bounds(
    devPathBounds: &crate::src::core::SkRect::SkRect,
    clipBounds: &crate::src::core::SkRect::SkIRect,
    filter: &dyn crate::src::core::SkMaskFilterBase::SkMaskFilterBase,
    filterMatrix: &SkMatrix,
    bounds: &mut crate::src::core::SkRect::SkIRect,
) -> Result<bool, crate::RasterError> {
    // Rust references enforce the two native non-null preconditions.
    *bounds = devPathBounds.makeOutset(0.5, 0.5).roundOut();
    let mut margin = crate::src::core::SkPoint::SkIPoint::make(0, 0);
    let srcM = crate::src::core::SkMask::SkMask::new(
        core::ptr::null(),
        *bounds,
        0,
        crate::src::core::SkMask::Format::kA8_Format,
    );
    let mut dstM = crate::src::core::SkMask::SkMaskBuilder::new();
    // SAFETY: the null source image explicitly requests bounds-only filtering;
    // the native trait contract forbids allocating an image for this request.
    if !unsafe { filter.filterMask(&mut dstM, &srcM, filterMatrix, Some(&mut margin)) }? {
        return Ok(false);
    }
    const kMaxMargin: i32 = 128;
    if !bounds
        .intersect(&clipBounds.make_outset(margin.fX.min(kMaxMargin), margin.fY.min(kMaxMargin)))
    {
        return Ok(false);
    }
    Ok(true)
}

// Source: src/core/SkDraw.cpp:1134-1172
/// # Safety
/// Every non-null mask image must be live, exclusively writable, and cover the
/// complete bounds/row-byte layout until this draw and its arena blitter finish.
unsafe fn draw_into_mask(
    mask: &crate::src::core::SkMask::SkMask,
    raw: crate::src::core::SkPathRaw::SkPathRaw<'_>,
    style: crate::src::core::SkStrokeRec::InitStyle,
) -> Result<(), crate::RasterError> {
    let mut dst = crate::src::core::SkPixmap::SkPixmap::default();
    if !dst.reset_mask(mask) {
        return Ok(());
    }
    let dx = (-mask.fBounds.fLeft) as f32;
    let dy = (-mask.fBounds.fTop) as f32;
    let translate = SkMatrix::Translate(dx, dy);
    let mut paint = crate::src::core::SkPaint::SkPaint::default();
    paint.setAntiAlias(true);
    let alloc = crate::src::core::SkAutoBlitterChoose::SkBlitterSizedArena::new(0);
    let mut alloc = std::pin::pin!(alloc);
    let mut blitter = unsafe {
        crate::src::core::SkBlitter_A8::SkChooseA8Blitter(
            &dst,
            &translate,
            &paint,
            alloc.as_mut().pinned_arena(),
            crate::src::core::SkDrawTypes::SkDrawCoverage::kNo,
            None,
        )
    }?
    .expect("native default A8 paint supplies a blitter");
    let dev_points = crate::include::private::SkTemplates::AutoSTArray::<
        crate::src::core::SkPoint::SkPoint,
        32,
    >::new();
    let mut dev_points = std::pin::pin!(dev_points);
    // Native AutoSTArray accepts int; retain that representability precondition.
    let count = i32::try_from(raw.fPoints.len()).unwrap_or_else(|_| std::process::abort());
    unsafe {
        dev_points.as_mut().init_uninit(count);
        // Raw mapping initializes each output; no reference to uninitialized
        // SkPoint values is ever formed. This is mapPoints' canonical dispatch.
        (translate.getMapPtsProc())(
            &translate,
            dev_points.as_mut().data_mut(),
            raw.fPoints.as_ptr(),
            raw.fPoints.len(),
        );
        dev_points.as_mut().assume_init();
    }
    // Reborrow the other raw spans for the local scratch allocation's lifetime.
    let mut raw = crate::src::core::SkPathRaw::SkPathRaw {
        fPoints: dev_points
            .as_slice()
            .expect("mapPoints initialized every point"),
        ..raw
    };
    raw.fBounds = raw.fBounds.makeOffset(dx, dy);
    if !raw.fBounds.isFinite() {
        return Ok(());
    }
    let clip = crate::src::core::SkRasterClip::SkRasterClip::from_rect(
        crate::src::core::SkRect::SkIRect::make_wh(mask.fBounds.width(), mask.fBounds.height()),
    );
    match style {
        crate::src::core::SkStrokeRec::InitStyle::kHairline_InitStyle => {
            crate::src::core::SkScan::SkScan::AntiHairPath(&raw, &clip, unsafe {
                core::pin::Pin::new_unchecked(blitter.as_mut())
            })
        }
        crate::src::core::SkStrokeRec::InitStyle::kFill_InitStyle => {
            crate::src::core::SkScan::SkScan::AntiFillPathRasterClip(&raw, &clip, unsafe {
                core::pin::Pin::new_unchecked(blitter.as_mut())
            })
        }
    }
    Ok(())
}

// Source: src/core/SkDraw.cpp:1174-1214
/// Safety: in JustRenderImage mode, dst's image must cover its native bounds/stride
/// and remain exclusively writable for the draw. Newly allocated images transfer
/// to the caller, which must release them once with SkMaskBuilder::FreeImage.
pub unsafe fn DrawToMask(
    devRaw: &crate::src::core::SkPathRaw::SkPathRaw<'_>,
    clipBounds: &crate::src::core::SkRect::SkIRect,
    filter: &dyn crate::src::core::SkMaskFilterBase::SkMaskFilterBase,
    filterMatrix: &SkMatrix,
    dst: &mut crate::src::core::SkMask::SkMaskBuilder,
    mode: crate::src::core::SkMask::CreateMode,
    style: crate::src::core::SkStrokeRec::InitStyle,
) -> Result<bool, crate::RasterError> {
    use crate::src::core::SkMask::{AllocType, CreateMode, Format, SkMaskBuilder};
    if devRaw.empty() {
        return Ok(false);
    }
    if mode != CreateMode::kJustRenderImage_CreateMode {
        const kInverseBounds: crate::src::core::SkRect::SkRect = crate::src::core::SkRect::SkRect {
            fLeft: f32::NEG_INFINITY,
            fTop: f32::NEG_INFINITY,
            fRight: f32::INFINITY,
            fBottom: f32::INFINITY,
        };
        let pathBounds = if devRaw.isInverseFillType() {
            kInverseBounds
        } else {
            devRaw.bounds()
        };
        if !compute_mask_bounds(&pathBounds, clipBounds, filter, filterMatrix, dst.bounds())? {
            return Ok(false);
        }
    }
    if mode == CreateMode::kComputeBoundsAndRenderImage_CreateMode {
        *dst.format() = Format::kA8_Format;
        *dst.rowBytes() = dst.mask.fBounds.width() as u32;
        let size = dst.mask.computeImageSize();
        if size == 0 {
            return Ok(false);
        }
        *dst.image() = SkMaskBuilder::AllocImage(size, AllocType::kZeroInit_Alloc);
    }
    if mode != CreateMode::kJustComputeBounds_CreateMode {
        unsafe {
            draw_into_mask(&dst.mask, *devRaw, style)?;
        }
    }
    Ok(true)
}

// Source: src/core/SkDraw.cpp:1216-1373
impl Draw<'_> {
    /// # Safety
    /// Same exclusive Draw/optional device lease as drawPoints().
    pub unsafe fn drawDevicePoints(
        &self,
        mode: crate::src::core::SkCanvas::PointMode,
        mut points: &[SkPoint],
        paint: &crate::src::core::SkPaint::SkPaint,
        mut device: Option<
            core::ptr::NonNull<dyn crate::src::core::SkDevice::SkDeviceVirtual + '_>,
        >,
    ) -> Result<(), crate::RasterError> {
        use crate::src::core::SkCanvas::PointMode;
        use crate::src::core::SkPaint::{Cap, Style};
        use crate::src::core::SkPath::SkPath;
        use crate::src::core::SkRect::SkRect;
        if mode == PointMode::kLines_PointMode {
            points = &points[..points.len() & !1];
        }
        #[cfg(sk_debug)]
        {
            self.validate();
        };
        let rc = self.fRC.expect("native Draw requires clip");
        if points.is_empty() || rc.is_empty() {
            return Ok(());
        }
        let scalars =
            unsafe { core::slice::from_raw_parts(points.as_ptr().cast::<f32>(), points.len() * 2) };
        if !crate::src::core::SkFloatingPoint::SkIsFinite(scalars) {
            return Ok(());
        }
        match mode {
            PointMode::kPoints_PointMode => {
                let mut new_paint = paint.clone();
                new_paint.setStyle(Style::kFill_Style);
                let width = new_paint.getStrokeWidth();
                let radius = width / 2.0;
                if new_paint.getStrokeCap() == Cap::kRound_Cap {
                    if let Some(mut device) = device {
                        for pt in points {
                            let r = SkRect::MakeLTRB(
                                pt.fX - radius,
                                pt.fY - radius,
                                pt.fX + radius,
                                pt.fY + radius,
                            );
                            unsafe { device.as_mut() }.drawOval(&r, &new_paint)?;
                        }
                    } else {
                        let path = SkPath::Circle(
                            0.0,
                            0.0,
                            radius,
                            crate::include::core::SkPathTypes::SkPathDirection::kCW,
                        );
                        let mut pre_matrix = SkMatrix::default();
                        for pt in points {
                            pre_matrix.setTranslate(pt.fX, pt.fY);
                            unsafe { self.drawPath(&path, &new_paint, Some(&pre_matrix)) }?;
                        }
                    }
                } else {
                    let mut r = SkRect::default();
                    for pt in points {
                        r.fLeft = pt.fX - radius;
                        r.fTop = pt.fY - radius;
                        r.fRight = r.fLeft + width;
                        r.fBottom = r.fTop + width;
                        if let Some(mut device) = device {
                            unsafe { device.as_mut() }.drawRect(&r, &new_paint)?;
                        } else {
                            unsafe { self.drawRect(&r, &new_paint, None, None) }?;
                        }
                    }
                }
            }
            PointMode::kLines_PointMode | PointMode::kPolygon_PointMode => {
                if mode == PointMode::kLines_PointMode
                    && points.len() == 2
                    && paint.getPathEffect().is_some()
                {
                    let stroke = crate::src::core::SkStrokeRec::SkStrokeRec::from_paint(paint, 1.0);
                    let mut data = crate::src::core::SkPathEffectBase::PointData::new();
                    let path = SkPath::Line(points[0], points[1]);
                    let cull_rect = SkRect::Make(rc.get_bounds());
                    if crate::src::core::SkPathEffectBase::SkPathEffectBase::asPoints(
                        paint.getPathEffect().unwrap().project_base(),
                        Some(&mut data),
                        &path,
                        &stroke,
                        self.fCTM.unwrap(),
                        Some(&cull_rect),
                    ) {
                        let mut new_paint = paint.clone();
                        new_paint.setPathEffect(crate::include::core::SkRefCnt::sk_sp::default());
                        new_paint.setStyle(Style::kFill_Style);
                        if !data.fFirst.isEmpty() {
                            if let Some(mut device) = device {
                                unsafe { device.as_mut() }.drawPath(&data.fFirst, &new_paint)?;
                            } else {
                                unsafe { self.drawPath(&data.fFirst, &new_paint, None) }?;
                            }
                        }
                        if !data.fLast.isEmpty() {
                            if let Some(mut device) = device {
                                unsafe { device.as_mut() }.drawPath(&data.fLast, &new_paint)?;
                            } else {
                                unsafe { self.drawPath(&data.fLast, &new_paint, None) }?;
                            }
                        }
                        if data.fSize.fX == data.fSize.fY {
                            crate::SkASSERT!(@source b"src/core/SkDraw.cpp\0".as_ptr(), 1316; @condition b"pointData.fSize.fX == newP.getStrokeWidth() / 2.f\0".as_ptr(); data.fSize.fX==new_paint.getStrokeWidth()/2.0);
                            if data.fFlags & crate::src::core::SkPathEffectBase::kCircles_PointFlag
                                != 0
                            {
                                new_paint.setStrokeCap(Cap::kRound_Cap);
                            } else {
                                new_paint.setStrokeCap(Cap::kButt_Cap);
                            }
                            if let Some(mut device) = device {
                                unsafe { device.as_mut() }.drawPoints(
                                    PointMode::kPoints_PointMode,
                                    data.points(),
                                    &new_paint,
                                )?;
                            } else {
                                unsafe {
                                    self.drawDevicePoints(
                                        PointMode::kPoints_PointMode,
                                        data.points(),
                                        &new_paint,
                                        device,
                                    )
                                }?;
                            }
                        } else {
                            crate::SkASSERT!(@source b"src/core/SkDraw.cpp\0".as_ptr(), 1334; @condition b"!(SkPathEffectBase::PointData::kCircles_PointFlag & pointData.fFlags)\0".as_ptr(); data.fFlags&crate::src::core::SkPathEffectBase::kCircles_PointFlag==0);
                            let size = data.fSize;
                            let mut r = SkRect::default();
                            for pt in data.points() {
                                r.setLTRB(
                                    pt.fX - size.fX,
                                    pt.fY - size.fY,
                                    pt.fX + size.fX,
                                    pt.fY + size.fY,
                                );
                                if let Some(mut device) = device {
                                    unsafe { device.as_mut() }.drawRect(&r, &new_paint)?;
                                } else {
                                    unsafe { self.drawRect(&r, &new_paint, None, None) }?;
                                }
                            }
                        }
                        return Ok(());
                    }
                }
                let count = points.len() - 1;
                let mut p = paint.clone();
                p.setStyle(Style::kStroke_Style);
                let inc = if mode == PointMode::kLines_PointMode {
                    2
                } else {
                    1
                };
                for i in (0..count).step_by(inc) {
                    let path = SkPath::Line(points[i], points[i + 1]);
                    if let Some(mut device) = device {
                        unsafe { device.as_mut() }.drawPath(&path, &p)?;
                    } else {
                        unsafe { self.drawPath(&path, &p, None) }?;
                    }
                }
            }
        }
        Ok(())
    }
}

// Source: src/core/SkDraw.h:259-259
#[cfg(not(sk_debug))]
impl Draw<'_> {
    pub(crate) fn validate(&self) {}
}
