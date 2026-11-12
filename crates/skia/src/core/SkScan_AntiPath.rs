// Drawing safety: unsafe render entrypoints require the SkBlitter primitive
// contract: live exclusive destination/context/arena leases, valid bounds and
// initialized terminating sparse runs. Clip wrappers preserve this caller lease;
// clipping alone does not establish ownership or backing-storage lifetime.
// Copyright 2006 The Android Open Source Project. BSD-style license.
use crate::src::core::SkBlitter::SkBlitter;
use crate::src::core::SkPathRaw::SkPathRaw;
use crate::src::core::SkPoint::SkIPoint;
use crate::src::core::SkRect::{SkIRect, SkRect};
use crate::src::core::SkRegion::{Op, SkRegion};
use crate::src::core::SkScan::SkScan;
use crate::src::core::SkScanPriv::{SkScanClipper, SK_SUPERSAMPLE_SHIFT};

/// Present picture-device coordinates to the AA scan converter while the
/// concrete destination blitter continues to address tile-local pixels.
struct RasterPhaseBlitter<'a> {
    inner: core::pin::Pin<&'a mut dyn SkBlitter>,
    origin: SkIPoint,
    _pin: core::marker::PhantomPinned,
}
impl<'a> RasterPhaseBlitter<'a> {
    fn new(inner: core::pin::Pin<&'a mut dyn SkBlitter>, origin: SkIPoint) -> Self {
        Self {
            inner,
            origin,
            _pin: core::marker::PhantomPinned,
        }
    }
}
unsafe impl SkBlitter for RasterPhaseBlitter<'_> {
    fn allocBlitMemory(self: core::pin::Pin<&mut Self>, size: usize) -> *mut u8 {
        unsafe { self.get_unchecked_mut() }
            .inner
            .as_mut()
            .allocBlitMemory(size)
    }

    unsafe fn blitH(self: core::pin::Pin<&mut Self>, x: i32, y: i32, width: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.inner
                .as_mut()
                .blitH(x - this.origin.fX, y - this.origin.fY, width)
        };
    }

    unsafe fn blitAntiH(
        self: core::pin::Pin<&mut Self>,
        x: i32,
        y: i32,
        antialias: &mut [u8],
        runs: &mut [i16],
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.inner
                .as_mut()
                .blitAntiH(x - this.origin.fX, y - this.origin.fY, antialias, runs)
        };
    }

    unsafe fn blitAntiH2(self: core::pin::Pin<&mut Self>, x: i32, y: i32, a0: u32, a1: u32) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.inner
                .as_mut()
                .blitAntiH2(x - this.origin.fX, y - this.origin.fY, a0, a1)
        };
    }

    unsafe fn blitAntiV2(self: core::pin::Pin<&mut Self>, x: i32, y: i32, a0: u32, a1: u32) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.inner
                .as_mut()
                .blitAntiV2(x - this.origin.fX, y - this.origin.fY, a0, a1)
        };
    }

    unsafe fn blitV(self: core::pin::Pin<&mut Self>, x: i32, y: i32, height: i32, alpha: u8) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.inner
                .as_mut()
                .blitV(x - this.origin.fX, y - this.origin.fY, height, alpha)
        };
    }

    fn requestRowsPreserved(&self) -> i32 {
        self.inner.requestRowsPreserved()
    }
}

// Source: src/core/SkScan_AntiPath.cpp:23-34
fn safeRoundOut(src: &SkRect) -> SkIRect {
    let mut dst = src.roundOut();
    let limit = i32::MAX >> SK_SUPERSAMPLE_SHIFT;
    dst.intersect(&SkIRect {
        fLeft: -limit,
        fTop: -limit,
        fRight: limit,
        fBottom: limit,
    });
    dst
}
// Source: src/core/SkScan_AntiPath.cpp:36-44
const fn overflows_short_shift(value: i32, shift: i32) -> i32 {
    let s = 16 + shift;
    (value.wrapping_shl(s as u32) >> s).wrapping_sub(value)
}
const _: () = assert!(overflows_short_shift(8191, SK_SUPERSAMPLE_SHIFT) == 0);
const _: () = assert!(overflows_short_shift(8192, SK_SUPERSAMPLE_SHIFT) != 0);
const _: () = assert!(overflows_short_shift(32767, 0) == 0);
const _: () = assert!(overflows_short_shift(32768, 0) != 0);
// Source: src/core/SkScan_AntiPath.cpp:50-57
fn rect_overflows_short_shift(rect: SkIRect, shift: i32) -> i32 {
    overflows_short_shift(rect.fLeft, shift)
        | overflows_short_shift(rect.fRight, shift)
        | overflows_short_shift(rect.fTop, shift)
        | overflows_short_shift(rect.fBottom, shift)
}
impl SkScan {
    pub unsafe fn AntiFillPath(
        path: &SkPathRaw<'_>,
        origClip: &SkRegion,
        mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
        forceRLE: bool,
    ) {
        Self::AntiFillPathWithRounding::<false>(path, origClip, blitter.as_mut(), forceRLE);
    }
    pub unsafe fn AntiFillPathWithPhase(
        path: &SkPathRaw<'_>,
        origClip: &SkRegion,
        mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
        forceRLE: bool,
        phase: SkIPoint,
        even_rounding: bool,
    ) {
        if phase.fX == 0 && phase.fY == 0 {
            if even_rounding {
                Self::AntiFillPathWithRounding::<true>(path, origClip, blitter.as_mut(), forceRLE);
            } else {
                Self::AntiFillPath(path, origClip, blitter.as_mut(), forceRLE);
            }
            return;
        }
        let mut scan_clip = origClip.clone();
        scan_clip.translate(phase.fX, phase.fY);
        let mut offset_blitter = core::pin::pin!(RasterPhaseBlitter::new(blitter.as_mut(), phase));
        if even_rounding {
            Self::AntiFillPathWithRounding::<true>(
                path,
                &scan_clip,
                offset_blitter.as_mut(),
                forceRLE,
            );
        } else {
            Self::AntiFillPathWithRounding::<false>(
                path,
                &scan_clip,
                offset_blitter.as_mut(),
                forceRLE,
            );
        }
    }
    // Source: src/core/SkScan_AntiPath.cpp:59-135
    pub unsafe fn AntiFillPathWithRounding<const EVEN: bool>(
        path: &SkPathRaw<'_>,
        origClip: &SkRegion,
        mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
        forceRLE: bool,
    ) {
        if origClip.is_empty() {
            return;
        }
        let isInverse = path.isInverseFillType();
        let ir = safeRoundOut(&path.bounds());
        if ir.is_empty() {
            if isInverse {
                blitter.as_mut().blitRegion(origClip);
            }
            return;
        }
        let mut clippedIR = SkIRect::default();
        if isInverse {
            clippedIR = *origClip.get_bounds();
        } else if !clippedIR.intersect_ab(ir, *origClip.get_bounds()) {
            return;
        }
        if rect_overflows_short_shift(clippedIR, SK_SUPERSAMPLE_SHIFT) != 0 {
            Self::FillPathWithRounding::<EVEN>(path, origClip, blitter.as_mut());
            return;
        }
        let mut tmpClipStorage = SkRegion::new();
        let mut clipRgn = origClip;
        const MAX_CLIP_COORD: i32 = 32767;
        let bounds = origClip.get_bounds();
        if bounds.fRight > MAX_CLIP_COORD || bounds.fBottom > MAX_CLIP_COORD {
            let limit = SkIRect {
                fLeft: 0,
                fTop: 0,
                fRight: MAX_CLIP_COORD,
                fBottom: MAX_CLIP_COORD,
            };
            tmpClipStorage.op_region_rect(origClip, limit, Op::Intersect);
            clipRgn = &tmpClipStorage;
        }
        let mut clipper = Box::pin(SkScanClipper::new(
            blitter.as_mut(),
            Some(clipRgn),
            &ir,
            false,
            false,
        ));
        if clipper.as_mut().getBlitter().is_none() {
            drop(clipper);
            if isInverse {
                blitter.as_mut().blitRegion(clipRgn);
            }
            return;
        }
        crate::SkASSERT!(@source b"src/core/SkScan_AntiPath.cpp\0".as_ptr(), 120; @condition b"clipper.getClipRect() == nullptr || *clipper.getClipRect() == clipRgn->getBounds()\0".as_ptr(); clipper.getClipRect().is_none() || clipper.getClipRect() == Some(clipRgn.get_bounds()));
        let mut blitter = clipper.as_mut().getBlitter().unwrap();
        if isInverse {
            crate::src::core::SkScan_Path::sk_blit_above(blitter.as_mut(), &ir, clipRgn);
        }
        Self::AAAFillPathWithRounding::<EVEN>(
            path,
            blitter.as_mut(),
            &ir,
            clipRgn.get_bounds(),
            forceRLE,
        );
        if isInverse {
            crate::src::core::SkScan_Path::sk_blit_below(blitter.as_mut(), &ir, clipRgn);
        }
    }
}
impl SkScan {
    // Source: src/core/SkScan_AntiPath.cpp:139-155
    // Rust names distinguish the original Region/RasterClip overloads.
    pub unsafe fn AntiFillPathRasterClip(
        raw: &SkPathRaw,
        clip: &crate::src::core::SkRasterClip::SkRasterClip,
        mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    ) {
        Self::AntiFillPathRasterClipWithPhase(
            raw,
            clip,
            blitter.as_mut(),
            SkIPoint::make(0, 0),
            false,
        );
    }

    pub unsafe fn AntiFillPathRasterClipWithPhase(
        raw: &SkPathRaw,
        clip: &crate::src::core::SkRasterClip::SkRasterClip,
        mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
        phase: SkIPoint,
        even_rounding: bool,
    ) {
        crate::SkASSERT!(@source b"src/core/SkScan_AntiPath.cpp\0".as_ptr(), 140; @condition b"raw.bounds().isFinite()\0".as_ptr(); raw.bounds().isFinite());
        if clip.is_empty() {
            return;
        }
        if clip.is_bw() {
            Self::AntiFillPathWithPhase(
                raw,
                clip.bw_rgn(),
                blitter.as_mut(),
                false,
                phase,
                even_rounding,
            );
        } else {
            let mut tmp = SkRegion::new();
            let mut aa_blitter =
                core::pin::pin!(crate::src::core::SkAAClip::SkAAClipBlitter::new());
            tmp.set_rect(*clip.get_bounds());
            aa_blitter.as_mut().init(blitter.as_mut(), clip.aa_rgn());
            Self::AntiFillPathWithPhase(raw, &tmp, aa_blitter.as_mut(), true, phase, even_rounding);
        }
    }

    /// Scan an antialiased hairline in picture-device coordinates while
    /// preserving the tile-local destination and clip coordinate system.
    pub unsafe fn AntiHairPathRasterClipWithPhase(
        raw: &SkPathRaw,
        clip: &crate::src::core::SkRasterClip::SkRasterClip,
        mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
        phase: SkIPoint,
        cap: crate::src::core::SkPaint::Cap,
    ) {
        use crate::src::core::SkPaint::Cap;

        let mut scan_clip = crate::src::core::SkRasterClip::SkRasterClip::new();
        clip.translate_to(phase.fX, phase.fY, Some(&mut scan_clip));
        let mut offset_blitter = core::pin::pin!(RasterPhaseBlitter::new(blitter.as_mut(), phase));
        match cap {
            Cap::kButt_Cap => Self::AntiHairPath(raw, &scan_clip, offset_blitter.as_mut()),
            Cap::kSquare_Cap => Self::AntiHairSquarePath(raw, &scan_clip, offset_blitter.as_mut()),
            Cap::kRound_Cap => Self::AntiHairRoundPath(raw, &scan_clip, offset_blitter.as_mut()),
        }
    }
}
