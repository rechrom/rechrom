// Copyright 2014/2017/2023 Google Inc. BSD-style license.
use super::SkImageInfo::SkImageInfo;
use super::SkPixmap::SkPixmap;
use crate::src::core::SkRect::SkIRect;
// Source: src/core/SkReadPixelsRec.h:25-31
impl SkReadPixelsRec {
    pub fn new(info: &SkImageInfo, pixels: *mut u8, rowBytes: usize, x: i32, y: i32) -> Self {
        Self {
            fPixels: pixels,
            fRowBytes: rowBytes,
            fInfo: info.clone(),
            fX: x,
            fY: y,
        }
    }
}
// Source: src/core/SkReadPixelsRec.h:33-39
impl SkReadPixelsRec {
    pub fn from_pixmap(pm: &SkPixmap, x: i32, y: i32) -> Self {
        Self {
            fPixels: pm.writable_addr(),
            fRowBytes: pm.rowBytes(),
            fInfo: pm.info().clone(),
            fX: x,
            fY: y,
        }
    }
}
// Source: src/core/SkReadPixelsRec.h:41-45
pub struct SkReadPixelsRec {
    pub fPixels: *mut u8,
    pub fRowBytes: usize,
    pub fInfo: SkImageInfo,
    pub fX: i32,
    pub fY: i32,
}
// Source: src/core/SkReadPixelsRec.cpp:12-12
// Source: src/core/SkReadPixelsRec.cpp:14-20
// Source: src/core/SkReadPixelsRec.cpp:22-24
// Source: src/core/SkReadPixelsRec.cpp:26-31
// Source: src/core/SkReadPixelsRec.cpp:34-39
// Source: src/core/SkReadPixelsRec.cpp:42-48
// Source: src/core/SkReadPixelsRec.cpp:50-50
// Source: src/core/SkReadPixelsRec.cpp:52-54
// Source: src/core/SkReadPixelsRec.cpp:56-57
impl SkReadPixelsRec {
    pub fn trim(&mut self, width: i32, height: i32) -> bool {
        let minRowBytes = self.fInfo.minRowBytes();
        if self.fPixels.is_null() || self.fRowBytes < minRowBytes || minRowBytes == 0 {
            return false;
        }
        if self.fInfo.width() <= 0 || self.fInfo.height() <= 0 {
            return false;
        }
        if self.fX == i32::MIN || self.fY == i32::MIN {
            return false;
        }
        let mut x = self.fX;
        let mut y = self.fY;
        let mut r = SkIRect::make_xywh(x, y, self.fInfo.width(), self.fInfo.height());
        if !r.intersect(&SkIRect::make_ltrb(0, 0, width, height)) {
            return false;
        }
        if x > 0 {
            x = 0;
        }
        if y > 0 {
            y = 0;
        }
        // Checked arithmetic carries the native SkSafeMath overflow flag through the three operations.
        let Some(y_offset) = ((-y) as usize).checked_mul(self.fRowBytes) else {
            return false;
        };
        let Some(x_offset) = ((-x) as usize).checked_mul(self.fInfo.bytesPerPixel() as usize)
        else {
            return false;
        };
        let Some(total) = y_offset.checked_add(x_offset) else {
            return false;
        };
        self.fPixels = self.fPixels.wrapping_add(total);
        self.fInfo = self.fInfo.makeDimensions(r.size());
        self.fX = r.fLeft;
        self.fY = r.fTop;
        true
    }
}
