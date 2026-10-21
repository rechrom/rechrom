// Drawing safety: unsafe render entrypoints require the SkBlitter primitive
// contract: live exclusive destination/context/arena leases, valid bounds and
// initialized terminating sparse runs. Clip wrappers preserve this caller lease;
// clipping alone does not establish ownership or backing-storage lifetime.
// Copyright 2006 The Android Open Source Project. BSD-style license.
use crate::include::private::SkFixed::*;
use crate::src::core::SkBlitter::SkBlitter;
use crate::src::core::SkRect::{SkIRect, SkRect};
use crate::src::core::SkRegion::{Cliperator, SkRegion};
// Source: src/core/SkScan.h:25-25
pub type SkXRect = SkIRect;
pub struct SkScan;
// Source: src/core/SkScan.h:35-35
pub type HairRgnProc = unsafe fn(
    &[crate::src::core::SkPoint::SkPoint],
    Option<&SkRegion>,
    core::pin::Pin<&mut dyn SkBlitter>,
);
// Source: src/core/SkScan.h:36-36
pub type HairRCProc = unsafe fn(
    &[crate::src::core::SkPoint::SkPoint],
    &crate::src::core::SkRasterClip::SkRasterClip,
    core::pin::Pin<&mut dyn SkBlitter>,
);

// Source: src/core/SkScan.h:98-103
pub fn XRect_set_i(xr: &mut SkXRect, src: &SkIRect) {
    xr.fLeft = SkIntToFixed(src.fLeft);
    xr.fTop = SkIntToFixed(src.fTop);
    xr.fRight = SkIntToFixed(src.fRight);
    xr.fBottom = SkIntToFixed(src.fBottom);
}
// Source: src/core/SkScan.h:109-114
pub fn XRect_set(xr: &mut SkXRect, src: &SkRect) {
    xr.fLeft = SkFloatToFixed(src.fLeft);
    xr.fTop = SkFloatToFixed(src.fTop);
    xr.fRight = SkFloatToFixed(src.fRight);
    xr.fBottom = SkFloatToFixed(src.fBottom);
}
// Source: src/core/SkScan.h:118-123
pub fn XRect_round(xr: &SkXRect, dst: &mut SkIRect) {
    dst.fLeft = SkFixedRoundToInt(xr.fLeft);
    dst.fTop = SkFixedRoundToInt(xr.fTop);
    dst.fRight = SkFixedRoundToInt(xr.fRight);
    dst.fBottom = SkFixedRoundToInt(xr.fBottom);
}
// Source: src/core/SkScan.h:128-133
pub fn XRect_roundOut(xr: &SkXRect, dst: &mut SkIRect) {
    dst.fLeft = SkFixedFloorToInt(xr.fLeft);
    dst.fTop = SkFixedFloorToInt(xr.fTop);
    dst.fRight = SkFixedCeilToInt(xr.fRight);
    dst.fBottom = SkFixedCeilToInt(xr.fBottom);
}
// Source: src/core/SkScan.cpp:13-15
unsafe fn blitrect(mut blitter: core::pin::Pin<&mut dyn SkBlitter>, r: &SkIRect) {
    blitter
        .as_mut()
        .blitRect(r.fLeft, r.fTop, r.width(), r.height());
}
impl SkScan {
    // Source: src/core/SkScan.cpp:17-45
    pub unsafe fn FillIRect(
        r: &SkIRect,
        clip: Option<&SkRegion>,
        mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    ) {
        if !r.is_empty() {
            if let Some(clip) = clip {
                if clip.is_rect() {
                    let clip_bounds = clip.get_bounds();
                    if clip_bounds.contains_rect(r) {
                        blitrect(blitter.as_mut(), r);
                    } else {
                        let mut rr = *r;
                        if rr.intersect(clip_bounds) {
                            blitrect(blitter.as_mut(), &rr);
                        }
                    }
                } else {
                    let mut cliper = Cliperator::new(clip, *r);
                    while !cliper.done() {
                        blitrect(blitter.as_mut(), cliper.rect());
                        cliper.next();
                    }
                }
            } else {
                blitrect(blitter.as_mut(), r);
            }
        }
    }
    // Source: src/core/SkScan.cpp:47-53
    pub unsafe fn FillXRect(
        xr: &SkXRect,
        clip: Option<&SkRegion>,
        mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    ) {
        let mut r = SkIRect::make_empty();
        XRect_round(xr, &mut r);
        Self::FillIRect(&r, clip, blitter.as_mut());
    }
    // Source: src/core/SkScan.cpp:55-61
    pub unsafe fn FillRect(
        r: &SkRect,
        clip: Option<&SkRegion>,
        mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    ) {
        let ir = r.round();
        Self::FillIRect(&ir, clip, blitter.as_mut());
    }
}
impl SkScan {
    // Source: src/core/SkScan.cpp:65-78
    pub unsafe fn FillIRectRasterClip(
        r: &SkIRect,
        clip: &crate::src::core::SkRasterClip::SkRasterClip,
        mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    ) {
        if clip.is_empty() || r.is_empty() {
            return;
        }
        if clip.is_bw() {
            Self::FillIRect(r, Some(clip.bw_rgn()), blitter.as_mut());
            return;
        }
        let mut wrapper = Box::pin(
            crate::src::core::SkRasterClip::SkAAClipBlitterWrapper::from_clip(
                clip,
                blitter.as_mut(),
            ),
        );
        let (rgn, mut blitter) = wrapper.as_mut().parts();
        Self::FillIRect(r, Some(rgn), blitter.as_mut());
    }
    // Source: src/core/SkScan.cpp:80-93
    pub unsafe fn FillXRectRasterClip(
        r: &SkXRect,
        clip: &crate::src::core::SkRasterClip::SkRasterClip,
        mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    ) {
        if clip.is_empty() || r.is_empty() {
            return;
        }
        if clip.is_bw() {
            Self::FillXRect(r, Some(clip.bw_rgn()), blitter.as_mut());
            return;
        }
        let mut wrapper = Box::pin(
            crate::src::core::SkRasterClip::SkAAClipBlitterWrapper::from_clip(
                clip,
                blitter.as_mut(),
            ),
        );
        let (rgn, mut blitter) = wrapper.as_mut().parts();
        Self::FillXRect(r, Some(rgn), blitter.as_mut());
    }
    // Source: src/core/SkScan.cpp:95-108
    pub unsafe fn FillRectRasterClip(
        r: &SkRect,
        clip: &crate::src::core::SkRasterClip::SkRasterClip,
        mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    ) {
        if clip.is_empty() || r.isEmpty() {
            return;
        }
        if clip.is_bw() {
            Self::FillRect(r, Some(clip.bw_rgn()), blitter.as_mut());
            return;
        }
        let mut wrapper = Box::pin(
            crate::src::core::SkRasterClip::SkAAClipBlitterWrapper::from_clip(
                clip,
                blitter.as_mut(),
            ),
        );
        let (rgn, mut blitter) = wrapper.as_mut().parts();
        Self::FillRect(r, Some(rgn), blitter.as_mut());
    }
}

impl SkScan {
    // Source: src/core/SkScan.h:61-61
    /// Safety: preserve the actual destination/blitter/arena lease for this draw.
    pub unsafe fn FillTriangle(
        pts: &[crate::src::core::SkPoint::SkPoint; 3],
        clip: &crate::src::core::SkRasterClip::SkRasterClip,
        blitter: core::pin::Pin<&mut dyn SkBlitter>,
    ) {
        unsafe {
            crate::src::core::SkScan_Path::FillTriangleRasterClip(pts, clip, blitter);
        }
    }
}
