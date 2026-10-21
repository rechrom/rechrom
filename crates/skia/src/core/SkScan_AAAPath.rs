use crate::src::core::SkBlitter::SkBlitter;
// Pin adaptation: concrete blitters inherit !Unpin from their actual embedded
// SkBlitterStorage. Scan entrypoints receive the original pinned destination;
// local polymorphic owners are pinned before dispatch and never moved afterward.
// get_unchecked_mut only projects existing fields; it never extracts/replaces
// pinned storage or transfers an installed arena/inline allocation.
// Drawing safety: unsafe render entrypoints require the SkBlitter primitive
// contract: live exclusive destination/context/arena leases, valid bounds and
// initialized terminating sparse runs. Clip wrappers preserve this caller lease;
// clipping alone does not establish ownership or backing-storage lifetime.
// Copyright 2016 The Android Open Source Project. BSD-style license.
use crate::include::private::SkFixed::*;
// Source: src/core/SkScan_AAAPath.cpp:91-98
fn add_alpha(alpha: &mut u8, delta: u8) {
    crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 92; @condition b"*alpha + delta <= 256\0".as_ptr(); *alpha as u32 + delta as u32 <= 256);
    *alpha =
        crate::src::core::SkAlphaRuns::SkAlphaRuns::CatchOverflow(*alpha as u32 + delta as u32);
}
fn safely_add_alpha(alpha: &mut u8, delta: u8) {
    *alpha = 255u32.min(*alpha as u32 + delta as u32) as u8;
}
// Source: src/core/SkScan_AAAPath.cpp:100-136
trait AdditiveBlitter: crate::src::core::SkBlitter::SkBlitter {
    fn getRealBlitter(
        self: core::pin::Pin<&mut Self>,
        forceRealBlitter: bool,
    ) -> core::pin::Pin<&mut dyn crate::src::core::SkBlitter::SkBlitter>;
    unsafe fn blitAntiHArray(self: core::pin::Pin<&mut Self>, x: i32, y: i32, antialias: &[u8]);
    unsafe fn blitAntiHPixel(self: core::pin::Pin<&mut Self>, x: i32, y: i32, alpha: u8);
    unsafe fn blitAntiHSpan(self: core::pin::Pin<&mut Self>, x: i32, y: i32, width: i32, alpha: u8);
    fn getWidth(&self) -> i32;
    unsafe fn flush_if_y_changed(self: core::pin::Pin<&mut Self>, y: i32, nextY: i32);
}
// Source: src/core/SkScan_AAAPath.cpp:139-206
// Adapted: regenerate the native non-owning mask descriptor at the flush call
// instead of retaining a pointer into a movable inline storage field.
struct MaskAdditiveBlitter<'a> {
    fRealBlitter: core::pin::Pin<&'a mut dyn crate::src::core::SkBlitter::SkBlitter>,
    maskBounds: crate::src::core::SkRect::SkIRect,
    rowBytes: usize,
    fClipRect: crate::src::core::SkRect::SkIRect,
    fStorage: [core::mem::MaybeUninit<u32>; 258],
    fRow: isize,
    fY: i32,
    storage: crate::src::core::SkBlitter::SkBlitterStorage,
}
impl MaskAdditiveBlitter<'_> {
    const kMAX_WIDTH: i32 = 32;
    const kMAX_STORAGE: i64 = 1024;
    fn CanHandleRect(bounds: &crate::src::core::SkRect::SkIRect) -> bool {
        let width = bounds.width();
        if width > Self::kMAX_WIDTH {
            return false;
        }
        let rb = (width as i64 + 3) & !3;
        let storage = rb * bounds.height() as i64;
        width <= Self::kMAX_WIDTH && storage <= Self::kMAX_STORAGE
    }
    fn row_offset(&mut self, y: i32) -> isize {
        if y != self.fY {
            self.fY = y;
            self.fRow = 1 + (y - self.maskBounds.fTop) as isize * self.rowBytes as isize
                - self.maskBounds.fLeft as isize;
        }
        self.fRow
    }
    fn bytes(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self.fStorage.as_mut_ptr().cast(),
                self.maskBounds.height() as usize * self.rowBytes + 2,
            )
        }
    }
    /// Safety: eventual accesses must return inside this mask storage, including
    /// its original one-byte guard on either side of the active pixel range.
    unsafe fn getRow(&mut self, y: i32) -> *mut u8 {
        let offset = self.row_offset(y);
        self.fStorage
            .as_mut_ptr()
            .cast::<u8>()
            .wrapping_offset(offset)
    }
}
impl<'a> MaskAdditiveBlitter<'a> {
    // Source: src/core/SkScan_AAAPath.cpp:208-227
    // SAFETY: caller retains the exclusive target and its backing leases until this
    // additive owner is dropped; the native final flush runs during Drop.
    unsafe fn new(
        mut realBlitter: core::pin::Pin<&'a mut dyn crate::src::core::SkBlitter::SkBlitter>,
        ir: &crate::src::core::SkRect::SkIRect,
        clipBounds: &crate::src::core::SkRect::SkIRect,
        isInverse: bool,
    ) -> Self {
        assert!(Self::CanHandleRect(ir));
        assert!(ir.width() >= 0 && ir.height() >= 0);
        crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 218; @condition b"!isInverse\0".as_ptr(); !isInverse);
        let mut fClipRect = *ir;
        if !fClipRect.intersect(clipBounds) {
            crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 222; @condition b"0\0".as_ptr(); false);
            fClipRect = crate::src::core::SkRect::SkIRect::default();
        }
        let mut value = Self {
            fRealBlitter: realBlitter,
            maskBounds: *ir,
            rowBytes: ir.width() as usize,
            fClipRect,
            fStorage: [core::mem::MaybeUninit::uninit(); 258],
            fRow: 0,
            fY: ir.fTop - 1,
            storage: crate::src::core::SkBlitter::SkBlitterStorage::new(),
        };
        unsafe {
            value
                .fStorage
                .as_mut_ptr()
                .cast::<u8>()
                .write_bytes(0, ir.height() as usize * value.rowBytes + 2);
        }
        value
    }
}
impl Drop for MaskAdditiveBlitter<'_> {
    fn drop(&mut self) {
        let mask = crate::src::core::SkMask::SkMask::new(
            self.fStorage.as_ptr().cast::<u8>().wrapping_add(1),
            self.maskBounds,
            self.rowBytes as u32,
            crate::src::core::SkMask::Format::kA8_Format,
        );
        // SAFETY: unsafe construction retains the target lease through this final flush;
        // the mask borrows this still-live owner storage for the duration of the call.
        unsafe {
            self.fRealBlitter.as_mut().blitMask(&mask, &self.fClipRect);
        }
    }
}
impl AdditiveBlitter for MaskAdditiveBlitter<'_> {
    fn getRealBlitter(
        self: core::pin::Pin<&mut Self>,
        forceRealBlitter: bool,
    ) -> core::pin::Pin<&mut dyn crate::src::core::SkBlitter::SkBlitter> {
        let this = unsafe { self.get_unchecked_mut() };
        if forceRealBlitter {
            this.fRealBlitter.as_mut()
        } else {
            unsafe { core::pin::Pin::new_unchecked(this) }
        }
    }
    // Source: src/core/SkScan_AAAPath.cpp:229-244
    unsafe fn blitAntiHArray(self: core::pin::Pin<&mut Self>, _: i32, _: i32, _: &[u8]) {
        let this = unsafe { self.get_unchecked_mut() };
        crate::SK_ABORT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 230; "Don't use this; directly add alphas to the mask.");
    }
    unsafe fn blitAntiHPixel(self: core::pin::Pin<&mut Self>, x: i32, y: i32, alpha: u8) {
        let this = unsafe { self.get_unchecked_mut() };
        crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 234; @condition b"x >= fMask.fBounds.fLeft - 1\0".as_ptr(); x >= this.maskBounds.fLeft - 1);
        let row = this.row_offset(y);
        add_alpha(&mut this.bytes()[(row + x as isize) as usize], alpha);
    }
    unsafe fn blitAntiHSpan(
        self: core::pin::Pin<&mut Self>,
        x: i32,
        y: i32,
        width: i32,
        alpha: u8,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 239; @condition b"x >= fMask.fBounds.fLeft - 1\0".as_ptr(); x >= this.maskBounds.fLeft - 1);
        let row = this.row_offset(y);
        for i in 0..width {
            add_alpha(&mut this.bytes()[(row + (x + i) as isize) as usize], alpha);
        }
    }
    fn getWidth(&self) -> i32 {
        self.fClipRect.width()
    }
    unsafe fn flush_if_y_changed(self: core::pin::Pin<&mut Self>, _: i32, _: i32) {
        let this = unsafe { self.get_unchecked_mut() };
    }
}
unsafe impl crate::src::core::SkBlitter::SkBlitter for MaskAdditiveBlitter<'_> {
    fn allocBlitMemory(self: core::pin::Pin<&mut Self>, sz: usize) -> *mut u8 {
        let this = unsafe { self.get_unchecked_mut() };
        this.storage.allocBlitMemory(sz)
    }
    unsafe fn blitH(self: core::pin::Pin<&mut Self>, _: i32, _: i32, _: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        crate::SkDEBUGFAIL!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 119; b"Please call real blitter's blitH instead.\0".as_ptr());
    }
    unsafe fn blitAntiH(
        self: core::pin::Pin<&mut Self>,
        _: i32,
        _: i32,
        _: &mut [u8],
        _: &mut [i16],
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        crate::SkDEBUGFAIL!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 111; b"Please call real blitter's blitAntiH instead.\0".as_ptr());
    }
    // Source: src/core/SkScan_AAAPath.cpp:246-280
    unsafe fn blitV(self: core::pin::Pin<&mut Self>, x: i32, y: i32, height: i32, alpha: u8) {
        let this = unsafe { self.get_unchecked_mut() };
        if alpha == 0 {
            return;
        }
        crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 250; @condition b"x >= fMask.fBounds.fLeft - 1\0".as_ptr(); x >= this.maskBounds.fLeft - 1);
        let mut row = this.row_offset(y);
        for _ in 0..height {
            this.bytes()[(row + x as isize) as usize] = alpha;
            row += this.rowBytes as isize;
        }
    }
    unsafe fn blitRect(self: core::pin::Pin<&mut Self>, x: i32, y: i32, width: i32, height: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 261; @condition b"x >= fMask.fBounds.fLeft - 1\0".as_ptr(); x >= this.maskBounds.fLeft - 1);
        let mut row = this.row_offset(y);
        for _ in 0..height {
            let start = (row + x as isize) as usize;
            this.bytes()[start..start + width as usize].fill(255);
            row += this.rowBytes as isize;
        }
    }
    unsafe fn blitAntiRect(
        self: core::pin::Pin<&mut Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        leftAlpha: u8,
        rightAlpha: u8,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { core::pin::Pin::new_unchecked(&mut *this) }.blitV(x, y, height, leftAlpha);
        unsafe { core::pin::Pin::new_unchecked(&mut *this) }.blitV(
            x + 1 + width,
            y,
            height,
            rightAlpha,
        );
        unsafe { core::pin::Pin::new_unchecked(&mut *this) }.blitRect(x + 1, y, width, height);
    }
}
// Source: src/core/SkScan_AAAPath.cpp:282-367
struct RunBasedAdditiveBlitter<'a> {
    fRealBlitter: core::pin::Pin<&'a mut dyn crate::src::core::SkBlitter::SkBlitter>,
    fCurrY: i32,
    fWidth: i32,
    fLeft: i32,
    fTop: i32,
    fRunsToBuffer: i32,
    fRunsBuffer: *mut u8,
    fCurrentRun: i32,
    fOffsetX: usize,
    storage: crate::src::core::SkBlitter::SkBlitterStorage,
}
impl RunBasedAdditiveBlitter<'_> {
    fn check(&self, x: i32, width: i32) -> bool {
        x >= 0 && x.wrapping_add(width) <= self.fWidth
    }
    fn getRunsSz(&self) -> usize {
        (self.fWidth + 1 + (self.fWidth + 2) / 2) as usize * core::mem::size_of::<i16>()
    }
    fn pointers(&self) -> (*mut i16, *mut u8) {
        let runs = self
            .fRunsBuffer
            .wrapping_add(self.fCurrentRun as usize * self.getRunsSz())
            .cast::<i16>();
        (runs, runs.wrapping_add(self.fWidth as usize + 1).cast())
    }
    unsafe fn view<'r>(
        runs: *mut i16,
        alpha: *mut u8,
        width: usize,
    ) -> crate::src::core::SkAlphaRuns::SkAlphaRuns<'r> {
        crate::src::core::SkAlphaRuns::SkAlphaRuns::from_initialized(
            core::slice::from_raw_parts_mut(runs, width + 1),
            core::slice::from_raw_parts_mut(alpha, width + 1),
            width,
        )
    }
    fn advanceRuns(&mut self) {
        self.fCurrentRun = (self.fCurrentRun + 1) % self.fRunsToBuffer;
        let (r, a) = self.pointers();
        unsafe {
            Self::view(r, a, self.fWidth as usize).reset(self.fWidth as usize);
        }
    }
    fn snapAlpha(alpha: u8) -> u8 {
        if alpha > 247 {
            255
        } else if alpha < 8 {
            0
        } else {
            alpha
        }
    }
    unsafe fn flush(&mut self) {
        if self.fCurrY >= self.fTop {
            crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 346; @condition b"fCurrentRun < fRunsToBuffer\0".as_ptr(); self.fCurrentRun < self.fRunsToBuffer);
            let (r, a) = self.pointers();
            let runs = unsafe { Self::view(r, a, self.fWidth as usize) };
            let mut x = 0;
            while runs.fRuns[x] != 0 {
                runs.fAlpha[x] = Self::snapAlpha(runs.fAlpha[x]);
                x += runs.fRuns[x] as usize;
            }
            if !runs.empty() {
                self.fRealBlitter.as_mut().blitAntiH(
                    self.fLeft,
                    self.fCurrY,
                    runs.fAlpha,
                    runs.fRuns,
                );
                self.advanceRuns();
                self.fOffsetX = 0;
            }
            self.fCurrY = self.fTop - 1;
        }
    }
    unsafe fn checkY(&mut self, y: i32) {
        if y != self.fCurrY {
            self.flush();
            self.fCurrY = y;
        }
    }
}
impl<'a> RunBasedAdditiveBlitter<'a> {
    // Source: src/core/SkScan_AAAPath.cpp:369-401
    // SAFETY: caller retains the exclusive target and its backing leases until this
    // additive owner is dropped; the native final flush runs during Drop.
    unsafe fn new(
        mut realBlitter: core::pin::Pin<&'a mut dyn crate::src::core::SkBlitter::SkBlitter>,
        ir: &crate::src::core::SkRect::SkIRect,
        clipBounds: &crate::src::core::SkRect::SkIRect,
        isInverse: bool,
    ) -> Self {
        let mut sectBounds = crate::src::core::SkRect::SkIRect::default();
        if isInverse {
            sectBounds = *clipBounds;
        } else if !sectBounds.intersect_ab(*ir, *clipBounds) {
            sectBounds = crate::src::core::SkRect::SkIRect::default();
        }
        let left = sectBounds.fLeft;
        let right = sectBounds.fRight;
        let mut result = Self {
            fRealBlitter: realBlitter,
            fCurrY: sectBounds.fTop - 1,
            fWidth: right - left,
            fLeft: left,
            fTop: sectBounds.fTop,
            fRunsToBuffer: 0,
            fRunsBuffer: core::ptr::null_mut(),
            fCurrentRun: -1,
            fOffsetX: 0,
            storage: crate::src::core::SkBlitter::SkBlitterStorage::new(),
        };
        result.fRunsToBuffer = result.fRealBlitter.requestRowsPreserved();
        assert!(result.fRunsToBuffer > 0);
        let bytes = (result.fRunsToBuffer as usize)
            .checked_mul(result.getRunsSz())
            .expect("run allocation overflow");
        result.fRunsBuffer = result.fRealBlitter.as_mut().allocBlitMemory(bytes);
        // Adapted initialization: forming Rust integer slices requires every
        // slot initialized; native sparse traversal only observes written slots.
        unsafe {
            result.fRunsBuffer.write_bytes(0, bytes);
        }
        result.advanceRuns();
        result
    }
    // Source: src/core/SkScan_AAAPath.cpp:403-430
    // Source: src/core/SkScan_AAAPath.cpp:473-500
    unsafe fn blitArray<const SAFE: bool>(&mut self, mut x: i32, y: i32, antialias: &[u8]) {
        self.checkY(y);
        x -= self.fLeft;
        let mut len = antialias.len() as i32;
        let mut skip = 0;
        if x < 0 {
            len += x;
            skip = (-x) as usize;
            x = 0;
        }
        len = len.min(self.fWidth - x);
        crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 483; @condition b"check(x, len)\0".as_ptr(); self.check(x, len));
        if x < self.fOffsetX as i32 {
            self.fOffsetX = 0;
        }
        let (r, a) = self.pointers();
        let mut runs = unsafe { Self::view(r, a, self.fWidth as usize) };
        self.fOffsetX = runs.add(x as usize, 0, len as usize, 0, 0, self.fOffsetX);
        let x = x as usize;
        let len = len as usize;
        let mut i = 0;
        while i < len {
            for j in 1..runs.fRuns[x + i] as usize {
                runs.fRuns[x + i + j] = 1;
                runs.fAlpha[x + i + j] = runs.fAlpha[x + i];
            }
            runs.fRuns[x + i] = 1;
            i += runs.fRuns[x + i] as usize;
        }
        for i in 0..len {
            if SAFE {
                safely_add_alpha(&mut runs.fAlpha[x + i], antialias[skip + i]);
            } else {
                add_alpha(&mut runs.fAlpha[x + i], antialias[skip + i]);
            }
        }
    }
    // Source: src/core/SkScan_AAAPath.cpp:432-443
    // Source: src/core/SkScan_AAAPath.cpp:502-515
    unsafe fn blitPixel<const SAFE: bool>(&mut self, mut x: i32, y: i32, alpha: u8) {
        self.checkY(y);
        x -= self.fLeft;
        if x < self.fOffsetX as i32 {
            self.fOffsetX = 0;
        }
        if self.check(x, 1) {
            let (r, a) = self.pointers();
            let mut runs = unsafe { Self::view(r, a, self.fWidth as usize) };
            self.fOffsetX = runs.add(
                x as usize,
                0,
                1,
                0,
                if SAFE { 0 } else { alpha as u32 },
                self.fOffsetX,
            );
            if SAFE {
                safely_add_alpha(&mut runs.fAlpha[x as usize], alpha);
            }
        }
    }
    // Source: src/core/SkScan_AAAPath.cpp:445-457
    // Source: src/core/SkScan_AAAPath.cpp:517-532
    unsafe fn blitSpan<const SAFE: bool>(&mut self, mut x: i32, y: i32, width: i32, alpha: u8) {
        self.checkY(y);
        x -= self.fLeft;
        if x < self.fOffsetX as i32 {
            self.fOffsetX = 0;
        }
        if self.check(x, width) {
            let (r, a) = self.pointers();
            let mut runs = unsafe { Self::view(r, a, self.fWidth as usize) };
            self.fOffsetX = runs.add(
                x as usize,
                0,
                width as usize,
                0,
                if SAFE { 0 } else { alpha as u32 },
                self.fOffsetX,
            );
            if SAFE {
                let mut i = x as usize;
                while i < (x + width) as usize {
                    safely_add_alpha(&mut runs.fAlpha[i], alpha);
                    i += runs.fRuns[i] as usize;
                }
            }
        }
    }
}
impl Drop for RunBasedAdditiveBlitter<'_> {
    fn drop(&mut self) {
        // SAFETY: unsafe construction retains the exclusive target lease through this final flush.
        unsafe { self.flush() };
    }
}
impl AdditiveBlitter for RunBasedAdditiveBlitter<'_> {
    fn getRealBlitter(
        self: core::pin::Pin<&mut Self>,
        _: bool,
    ) -> core::pin::Pin<&mut dyn crate::src::core::SkBlitter::SkBlitter> {
        let this = unsafe { self.get_unchecked_mut() };
        this.fRealBlitter.as_mut()
    }
    unsafe fn blitAntiHArray(self: core::pin::Pin<&mut Self>, x: i32, y: i32, aa: &[u8]) {
        let this = unsafe { self.get_unchecked_mut() };
        this.blitArray::<false>(x, y, aa)
    }
    unsafe fn blitAntiHPixel(self: core::pin::Pin<&mut Self>, x: i32, y: i32, alpha: u8) {
        let this = unsafe { self.get_unchecked_mut() };
        this.blitPixel::<false>(x, y, alpha)
    }
    unsafe fn blitAntiHSpan(
        self: core::pin::Pin<&mut Self>,
        x: i32,
        y: i32,
        width: i32,
        alpha: u8,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        this.blitSpan::<false>(x, y, width, alpha)
    }
    fn getWidth(&self) -> i32 {
        self.fWidth
    }
    unsafe fn flush_if_y_changed(self: core::pin::Pin<&mut Self>, y: i32, nextY: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        if SkFixedFloorToInt(y) != SkFixedFloorToInt(nextY) {
            this.flush();
        }
    }
}
macro_rules! impl_rle_base_blitter {
    ($ty:ident,$storage:expr) => {
        unsafe impl crate::src::core::SkBlitter::SkBlitter for $ty<'_> {
            fn allocBlitMemory(self: core::pin::Pin<&mut Self>, sz: usize) -> *mut u8 {
        let this = unsafe { self.get_unchecked_mut() };
                ($storage)(this, sz)
            }
            unsafe fn blitH(self: core::pin::Pin<&mut Self>, _: i32, _: i32, _: i32) {
        let this = unsafe { self.get_unchecked_mut() };
                crate::SkDEBUGFAIL!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 119; b"Please call real blitter's blitH instead.\0".as_ptr());
            }
            unsafe fn blitAntiH(self: core::pin::Pin<&mut Self>, _: i32, _: i32, _: &mut [u8], _: &mut [i16]) {
        let this = unsafe { self.get_unchecked_mut() };
                crate::SkDEBUGFAIL!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 111; b"Please call real blitter's blitAntiH instead.\0".as_ptr());
            }
            unsafe fn blitV(self: core::pin::Pin<&mut Self>, _: i32, _: i32, _: i32, _: u8) {
        let this = unsafe { self.get_unchecked_mut() };
                crate::SkDEBUGFAIL!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 115; b"Please call real blitter's blitV instead.\0".as_ptr());
            }
            unsafe fn blitRect(self: core::pin::Pin<&mut Self>, _: i32, _: i32, _: i32, _: i32) {
        let this = unsafe { self.get_unchecked_mut() };
                crate::SkDEBUGFAIL!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 123; b"Please call real blitter's blitRect instead.\0".as_ptr());
            }
            unsafe fn blitAntiRect(self: core::pin::Pin<&mut Self>, _: i32, _: i32, _: i32, _: i32, _: u8, _: u8) {
        let this = unsafe { self.get_unchecked_mut() };
                crate::SkDEBUGFAIL!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 128; b"Please call real blitter's blitAntiRect instead.\0".as_ptr());
            }
        }
    };
}
impl_rle_base_blitter!(
    RunBasedAdditiveBlitter,
    |s: &mut RunBasedAdditiveBlitter<'_>, sz| s.storage.allocBlitMemory(sz)
);
// Source: src/core/SkScan_AAAPath.cpp:460-471
struct SafeRLEAdditiveBlitter<'a> {
    base: RunBasedAdditiveBlitter<'a>,
}
impl<'a> SafeRLEAdditiveBlitter<'a> {
    // SAFETY: caller retains the exclusive target and its backing leases until this
    // additive owner is dropped; the native final flush runs during Drop.
    unsafe fn new(
        mut real: core::pin::Pin<&'a mut dyn crate::src::core::SkBlitter::SkBlitter>,
        ir: &crate::src::core::SkRect::SkIRect,
        clip: &crate::src::core::SkRect::SkIRect,
        inverse: bool,
    ) -> Self {
        Self {
            base: RunBasedAdditiveBlitter::new(real, ir, clip, inverse),
        }
    }
}
impl AdditiveBlitter for SafeRLEAdditiveBlitter<'_> {
    fn getRealBlitter(
        self: core::pin::Pin<&mut Self>,
        force: bool,
    ) -> core::pin::Pin<&mut dyn crate::src::core::SkBlitter::SkBlitter> {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { core::pin::Pin::new_unchecked(&mut this.base) }.getRealBlitter(force)
    }
    unsafe fn blitAntiHArray(self: core::pin::Pin<&mut Self>, x: i32, y: i32, aa: &[u8]) {
        let this = unsafe { self.get_unchecked_mut() };
        this.base.blitArray::<true>(x, y, aa)
    }
    unsafe fn blitAntiHPixel(self: core::pin::Pin<&mut Self>, x: i32, y: i32, alpha: u8) {
        let this = unsafe { self.get_unchecked_mut() };
        this.base.blitPixel::<true>(x, y, alpha)
    }
    unsafe fn blitAntiHSpan(
        self: core::pin::Pin<&mut Self>,
        x: i32,
        y: i32,
        width: i32,
        alpha: u8,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        this.base.blitSpan::<true>(x, y, width, alpha)
    }
    fn getWidth(&self) -> i32 {
        self.base.getWidth()
    }
    unsafe fn flush_if_y_changed(self: core::pin::Pin<&mut Self>, y: i32, nextY: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { core::pin::Pin::new_unchecked(&mut this.base) }.flush_if_y_changed(y, nextY)
    }
}
impl_rle_base_blitter!(
    SafeRLEAdditiveBlitter,
    |s: &mut SafeRLEAdditiveBlitter<'_>, sz| s.base.storage.allocBlitMemory(sz)
);
// Source: src/core/SkScan_AAAPath.cpp:535-559
fn trapezoid_to_alpha(l1: i32, l2: i32) -> u8 {
    crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 536; @condition b"l1 >= 0 && l2 >= 0\0".as_ptr(); l1 >= 0 && l2 >= 0);
    let area = l1.wrapping_add(l2) / 2;
    (area >> 8) as u8
}
fn partial_triangle_to_alpha(a: i32, b: i32) -> u8 {
    crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 543; @condition b"a <= SK_Fixed1\0".as_ptr(); a <= SK_Fixed1);
    let area = (a >> 11).wrapping_mul(a >> 11).wrapping_mul(b >> 11);
    ((area >> 8) & 255) as u8
}
// Source: src/core/SkScan_AAAPath.cpp:561-567
fn get_partial_alpha_height(alpha: u8, partialHeight: i32) -> u8 {
    SkFixedRoundToInt((alpha as i32).wrapping_mul(partialHeight)) as u8
}
fn get_partial_alpha(alpha: u8, fullAlpha: u8) -> u8 {
    ((alpha as u32 * fullAlpha as u32) >> 8) as u8
}
// Source: src/core/SkScan_AAAPath.cpp:572-575
fn fixed_to_alpha(f: i32) -> u8 {
    crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 573; @condition b"f <= SK_Fixed1\0".as_ptr(); f <= SK_Fixed1);
    get_partial_alpha_height(255, f)
}
// Source: src/core/SkScan_AAAPath.cpp:579-587
fn approximate_intersection(mut l1: i32, mut r1: i32, mut l2: i32, mut r2: i32) -> i32 {
    if l1 > r1 {
        core::mem::swap(&mut l1, &mut r1);
    }
    if l2 > r2 {
        core::mem::swap(&mut l2, &mut r2);
    }
    l1.max(l2).wrapping_add(r1.min(r2)) / 2
}
// Source: src/core/SkScan_AAAPath.cpp:590-614
fn compute_alpha_above_line(alphas: &mut [u8], l: i32, r: i32, dY: i32, fullAlpha: u8) {
    crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 595; @condition b"l <= r\0".as_ptr(); l <= r);
    crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 596; @condition b"l >> 16 == 0\0".as_ptr(); l >> 16 == 0);
    let r_ceil = SkFixedCeilToInt(r);
    if r_ceil == 0 {
        return;
    } else if r_ceil == 1 {
        alphas[0] = get_partial_alpha(
            (((r_ceil << 17).wrapping_sub(l).wrapping_sub(r)) >> 9) as u8,
            fullAlpha,
        );
    } else {
        let first = SK_Fixed1 - l;
        let last = r - ((r_ceil - 1) << 16);
        let firstH = SkFixedMul(first, dY);
        alphas[0] = (SkFixedMul(first, firstH) >> 9) as u8;
        let mut alpha16 = firstH.saturating_add(dY >> 1);
        for i in 1..r_ceil - 1 {
            alphas[i as usize] = (alpha16 >> 8) as u8;
            alpha16 = alpha16.saturating_add(dY);
        }
        alphas[(r_ceil - 1) as usize] = fullAlpha.wrapping_sub(partial_triangle_to_alpha(last, dY));
    }
}
// Source: src/core/SkScan_AAAPath.cpp:617-641
fn compute_alpha_below_line(alphas: &mut [u8], l: i32, r: i32, dY: i32, fullAlpha: u8) {
    crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 622; @condition b"l <= r\0".as_ptr(); l <= r);
    crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 623; @condition b"l >> 16 == 0\0".as_ptr(); l >> 16 == 0);
    let r_ceil = SkFixedCeilToInt(r);
    if r_ceil == 0 {
        return;
    } else if r_ceil == 1 {
        alphas[0] = get_partial_alpha(trapezoid_to_alpha(l, r), fullAlpha);
    } else {
        let first = SK_Fixed1 - l;
        let last = r - ((r_ceil - 1) << 16);
        let lastH = SkFixedMul(last, dY);
        alphas[(r_ceil - 1) as usize] = (SkFixedMul(last, lastH) >> 9) as u8;
        let mut alpha16 = lastH.saturating_add(dY >> 1);
        for i in (1..r_ceil - 1).rev() {
            alphas[i as usize] = ((alpha16 >> 8) & 255) as u8;
            alpha16 = alpha16.saturating_add(dY);
        }
        alphas[0] = fullAlpha.wrapping_sub(partial_triangle_to_alpha(first, dY));
    }
}
// Source: src/core/SkScan_AAAPath.cpp:644-664
unsafe fn blit_single_alpha(
    blitter: *mut (dyn AdditiveBlitter + '_),
    y: i32,
    x: i32,
    alpha: u8,
    fullAlpha: u8,
    maskRow: *mut u8,
    noRealBlitter: bool,
) {
    if !maskRow.is_null() {
        let pixel = &mut *maskRow.wrapping_offset(x as isize);
        if fullAlpha == 255 && !noRealBlitter {
            *pixel = alpha;
        } else {
            safely_add_alpha(pixel, get_partial_alpha(alpha, fullAlpha));
        }
    } else if fullAlpha == 255 && !noRealBlitter {
        core::pin::Pin::new_unchecked(&mut *blitter)
            .getRealBlitter(false)
            .blitV(x, y, 1, alpha);
    } else {
        core::pin::Pin::new_unchecked(&mut *blitter).blitAntiHPixel(
            x,
            y,
            get_partial_alpha(alpha, fullAlpha),
        );
    }
}
// Source: src/core/SkScan_AAAPath.cpp:666-685
unsafe fn blit_two_alphas(
    blitter: *mut (dyn AdditiveBlitter + '_),
    y: i32,
    x: i32,
    a1: u8,
    a2: u8,
    fullAlpha: u8,
    maskRow: *mut u8,
    noRealBlitter: bool,
) {
    if !maskRow.is_null() {
        safely_add_alpha(&mut *maskRow.wrapping_offset(x as isize), a1);
        safely_add_alpha(&mut *maskRow.wrapping_offset((x + 1) as isize), a2);
    } else if fullAlpha == 255 && !noRealBlitter {
        core::pin::Pin::new_unchecked(&mut *blitter)
            .getRealBlitter(false)
            .blitAntiH2(x, y, a1 as u32, a2 as u32);
    } else {
        core::pin::Pin::new_unchecked(&mut *blitter).blitAntiHPixel(x, y, a1);
        core::pin::Pin::new_unchecked(&mut *blitter).blitAntiHPixel(x + 1, y, a2);
    }
}
// Source: src/core/SkScan_AAAPath.cpp:687-705
unsafe fn blit_full_alpha(
    blitter: *mut (dyn AdditiveBlitter + '_),
    y: i32,
    x: i32,
    len: i32,
    fullAlpha: u8,
    maskRow: *mut u8,
    noRealBlitter: bool,
) {
    if !maskRow.is_null() {
        for i in 0..len {
            safely_add_alpha(&mut *maskRow.wrapping_offset((x + i) as isize), fullAlpha);
        }
    } else if fullAlpha == 255 && !noRealBlitter {
        core::pin::Pin::new_unchecked(&mut *blitter)
            .getRealBlitter(false)
            .blitH(x, y, len);
    } else {
        core::pin::Pin::new_unchecked(&mut *blitter).blitAntiHSpan(x, y, len, fullAlpha);
    }
}
// Source: src/core/SkScan_AAAPath.cpp:707-804
unsafe fn blit_aaa_trapezoid_row(
    blitter: *mut (dyn AdditiveBlitter + '_),
    y: i32,
    ul: i32,
    ur: i32,
    ll: i32,
    lr: i32,
    lDY: i32,
    rDY: i32,
    fullAlpha: u8,
    maskRow: *mut u8,
    noRealBlitter: bool,
) {
    let l = SkFixedFloorToInt(ul);
    let r = SkFixedCeilToInt(lr);
    let len = (r - l) as usize;
    if len == 1 {
        blit_single_alpha(
            blitter,
            y,
            l,
            trapezoid_to_alpha(ur - ul, lr - ll),
            fullAlpha,
            maskRow,
            noRealBlitter,
        );
        return;
    }
    const QUICK_LEN: usize = 31;
    // Adapted validity/alignment: the same contiguous 4*(len+1) bytes and stack
    // threshold; initialize unused slots before exposing Rust integer slices.
    let mut quickMemory = [0u16; 2 * (QUICK_LEN + 1)];
    let mut heapMemory: Vec<u16> = Vec::new();
    let memory = if len <= QUICK_LEN {
        quickMemory.as_mut_ptr().cast::<u8>()
    } else {
        heapMemory.resize(2 * (len + 1), 0);
        heapMemory.as_mut_ptr().cast::<u8>()
    };
    let alphas = core::slice::from_raw_parts_mut(memory, len + 1);
    let tempAlphas = core::slice::from_raw_parts_mut(memory.add(len + 1), len + 1);
    let runs = core::slice::from_raw_parts_mut(memory.add((len + 1) * 2).cast::<i16>(), len + 1);
    for i in 0..len {
        runs[i] = 1;
        alphas[i] = fullAlpha;
    }
    runs[len] = 0;
    let uL = SkFixedFloorToInt(ul);
    let lL = SkFixedCeilToInt(ll);
    if uL + 2 == lL {
        let first = SkIntToFixed(uL) + SK_Fixed1 - ul;
        let second = ll - ul - first;
        let a1 = fullAlpha.wrapping_sub(partial_triangle_to_alpha(first, lDY));
        let a2 = partial_triangle_to_alpha(second, lDY);
        alphas[0] = if alphas[0] > a1 { alphas[0] - a1 } else { 0 };
        alphas[1] = if alphas[1] > a2 { alphas[1] - a2 } else { 0 };
    } else {
        compute_alpha_below_line(
            &mut tempAlphas[(uL - l) as usize..],
            ul - SkIntToFixed(uL),
            ll - SkIntToFixed(uL),
            lDY,
            fullAlpha,
        );
        for i in uL..lL {
            let j = (i - l) as usize;
            alphas[j] = if alphas[j] > tempAlphas[j] {
                alphas[j] - tempAlphas[j]
            } else {
                0
            };
        }
    }
    let uR = SkFixedFloorToInt(ur);
    let lR = SkFixedCeilToInt(lr);
    if uR + 2 == lR {
        let first = SkIntToFixed(uR) + SK_Fixed1 - ur;
        let second = lr - ur - first;
        let a1 = partial_triangle_to_alpha(first, rDY);
        let a2 = fullAlpha.wrapping_sub(partial_triangle_to_alpha(second, rDY));
        alphas[len - 2] = if alphas[len - 2] > a1 {
            alphas[len - 2] - a1
        } else {
            0
        };
        alphas[len - 1] = if alphas[len - 1] > a2 {
            alphas[len - 1] - a2
        } else {
            0
        };
    } else {
        compute_alpha_above_line(
            &mut tempAlphas[(uR - l) as usize..],
            ur - SkIntToFixed(uR),
            lr - SkIntToFixed(uR),
            rDY,
            fullAlpha,
        );
        for i in uR..lR {
            let j = (i - l) as usize;
            alphas[j] = if alphas[j] > tempAlphas[j] {
                alphas[j] - tempAlphas[j]
            } else {
                0
            };
        }
    }
    if !maskRow.is_null() {
        for i in 0..len {
            safely_add_alpha(
                &mut *maskRow.wrapping_offset(l as isize + i as isize),
                alphas[i],
            );
        }
    } else if fullAlpha == 255 && !noRealBlitter {
        core::pin::Pin::new_unchecked(&mut *blitter)
            .getRealBlitter(false)
            .blitAntiH(l, y, &mut alphas[..len], runs);
    } else {
        core::pin::Pin::new_unchecked(&mut *blitter).blitAntiHArray(l, y, &alphas[..len]);
    }
}
// Source: src/core/SkScan_AAAPath.cpp:806-947
unsafe fn blit_trapezoid_row(
    blitter: *mut (dyn AdditiveBlitter + '_),
    y: i32,
    mut ul: i32,
    mut ur: i32,
    mut ll: i32,
    mut lr: i32,
    lDY: i32,
    rDY: i32,
    fullAlpha: u8,
    maskRow: *mut u8,
    noRealBlitter: bool,
) {
    crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 817; @condition b"lDY >= 0 && rDY >= 0\0".as_ptr(); lDY >= 0 && rDY >= 0);
    if ul > ur {
        return;
    }
    if ll > lr {
        ll = approximate_intersection(ul, ll, ur, lr);
        lr = ll;
    }
    if ul == ur && ll == lr {
        return;
    }
    if ul > ll {
        core::mem::swap(&mut ul, &mut ll);
    }
    if ur > lr {
        core::mem::swap(&mut ur, &mut lr);
    }
    let joinLeft = SkFixedCeilToFixed(ll);
    let joinRite = SkFixedFloorToFixed(ur);
    if joinLeft <= joinRite {
        if ul < joinLeft {
            let len = SkFixedCeilToInt(joinLeft - ul);
            if len == 1 {
                let alpha = trapezoid_to_alpha(joinLeft - ul, joinLeft - ll);
                blit_single_alpha(
                    blitter,
                    y,
                    ul >> 16,
                    alpha,
                    fullAlpha,
                    maskRow,
                    noRealBlitter,
                );
            } else if len == 2 {
                let first = joinLeft - SK_Fixed1 - ul;
                let second = ll - ul - first;
                let a1 = partial_triangle_to_alpha(first, lDY);
                let a2 = fullAlpha.wrapping_sub(partial_triangle_to_alpha(second, lDY));
                blit_two_alphas(
                    blitter,
                    y,
                    ul >> 16,
                    a1,
                    a2,
                    fullAlpha,
                    maskRow,
                    noRealBlitter,
                );
            } else {
                blit_aaa_trapezoid_row(
                    blitter,
                    y,
                    ul,
                    joinLeft,
                    ll,
                    joinLeft,
                    lDY,
                    i32::MAX,
                    fullAlpha,
                    maskRow,
                    noRealBlitter,
                );
            }
        }
        if joinLeft < joinRite {
            blit_full_alpha(
                blitter,
                y,
                SkFixedFloorToInt(joinLeft),
                SkFixedFloorToInt(joinRite - joinLeft),
                fullAlpha,
                maskRow,
                noRealBlitter,
            );
        }
        if lr > joinRite {
            let len = SkFixedCeilToInt(lr - joinRite);
            if len == 1 {
                let alpha = trapezoid_to_alpha(ur - joinRite, lr - joinRite);
                blit_single_alpha(
                    blitter,
                    y,
                    joinRite >> 16,
                    alpha,
                    fullAlpha,
                    maskRow,
                    noRealBlitter,
                );
            } else if len == 2 {
                let first = joinRite + SK_Fixed1 - ur;
                let second = lr - ur - first;
                let a1 = fullAlpha.wrapping_sub(partial_triangle_to_alpha(first, rDY));
                let a2 = partial_triangle_to_alpha(second, rDY);
                blit_two_alphas(
                    blitter,
                    y,
                    joinRite >> 16,
                    a1,
                    a2,
                    fullAlpha,
                    maskRow,
                    noRealBlitter,
                );
            } else {
                blit_aaa_trapezoid_row(
                    blitter,
                    y,
                    joinRite,
                    ur,
                    joinRite,
                    lr,
                    i32::MAX,
                    rDY,
                    fullAlpha,
                    maskRow,
                    noRealBlitter,
                );
            }
        }
    } else {
        blit_aaa_trapezoid_row(
            blitter,
            y,
            ul,
            ur,
            ll,
            lr,
            lDY,
            rDY,
            fullAlpha,
            maskRow,
            noRealBlitter,
        );
    }
}
use crate::src::core::SkAnalyticEdge::{
    SkAnalyticCubicEdge, SkAnalyticEdge, SkAnalyticQuadraticEdge,
};
// Source: src/core/SkScan_AAAPath.cpp:949-959
unsafe fn compare_edges(a: *const SkAnalyticEdge, b: *const SkAnalyticEdge) -> bool {
    if (*a).fUpperY != (*b).fUpperY {
        return (*a).fUpperY < (*b).fUpperY;
    }
    if (*a).fX != (*b).fX {
        return (*a).fX < (*b).fX;
    }
    (*a).fDX < (*b).fDX
}
// Source: src/core/SkScan_AAAPath.cpp:961-972
unsafe fn sort_edges(
    list: &mut [*mut SkAnalyticEdge],
    last: &mut *mut SkAnalyticEdge,
) -> *mut SkAnalyticEdge {
    crate::src::core::SkTSort::SkTQSort(list, &|a, b| compare_edges(*a, *b));
    for i in 1..list.len() {
        (*list[i - 1]).fNext = list[i];
        (*list[i]).fPrev = list[i - 1];
    }
    *last = list[list.len() - 1];
    list[0]
}
// Source: src/core/SkScan_AAAPath.cpp:974-986
unsafe fn validate_sort(mut edge: *const SkAnalyticEdge) {
    #[cfg(sk_debug)]
    {
        let mut y = SkIntToFixed(-32768);
        while (*edge).fUpperY != i32::MAX {
            (*edge).validate();
            crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 980; @condition b"y <= edge->fUpperY\0".as_ptr(); y <= (*edge).fUpperY);
            y = (*edge).fUpperY;
            edge = (*edge).fNext;
        }
    }
}
// Source: src/core/SkScan_AAAPath.cpp:991-1009
unsafe fn is_smooth_enough_edge(
    thisEdge: *mut SkAnalyticEdge,
    nextEdge: *mut SkAnalyticEdge,
    _stop_y: i32,
) -> bool {
    if (*thisEdge).fCurveCount < 0 {
        let cEdge = &*thisEdge.cast::<SkAnalyticCubicEdge>();
        let ddshift = cEdge.fCurveShift;
        return cEdge.fCDx.wrapping_abs() >> 1 >= cEdge.fCDDx.wrapping_abs() >> ddshift
            && cEdge.fCDy.wrapping_abs() >> 1 >= cEdge.fCDDy.wrapping_abs() >> ddshift
            && cEdge.fCDy.wrapping_sub(cEdge.fCDDy >> ddshift) >> cEdge.fToFixedShift >= SK_Fixed1;
    } else if (*thisEdge).fCurveCount > 0 {
        let qEdge = &*thisEdge.cast::<SkAnalyticQuadraticEdge>();
        return qEdge.fQDx.wrapping_abs() >> 1 >= qEdge.fQDDx.wrapping_abs()
            && qEdge.fQDy.wrapping_abs() >> 1 >= qEdge.fQDDy.wrapping_abs()
            && qEdge.fQDy.wrapping_sub(qEdge.fQDDy) >> qEdge.fCurveShift >= SK_Fixed1;
    }
    (*nextEdge)
        .fDX
        .saturating_sub((*thisEdge).fDX)
        .wrapping_abs()
        <= SK_Fixed1
        && (*nextEdge).fLowerY.wrapping_sub((*nextEdge).fUpperY) >= SK_Fixed1
}
// Source: src/core/SkScan_AAAPath.cpp:1013-1036
unsafe fn is_smooth_enough(
    leftE: *mut SkAnalyticEdge,
    riteE: *mut SkAnalyticEdge,
    mut currE: *mut SkAnalyticEdge,
    stop_y: i32,
) -> bool {
    if (*currE).fUpperY >= stop_y.wrapping_shl(16) {
        return false;
    }
    if (*leftE).fLowerY.wrapping_add(SK_Fixed1) < (*riteE).fLowerY {
        return is_smooth_enough_edge(leftE, currE, stop_y);
    } else if (*leftE).fLowerY > (*riteE).fLowerY.wrapping_add(SK_Fixed1) {
        return is_smooth_enough_edge(riteE, currE, stop_y);
    }
    let mut nextCurrE = (*currE).fNext;
    if (*nextCurrE).fUpperY >= stop_y.wrapping_shl(16) {
        return false;
    }
    if (*nextCurrE).fUpperX < (*currE).fUpperX {
        core::mem::swap(&mut currE, &mut nextCurrE);
    }
    is_smooth_enough_edge(leftE, currE, stop_y) && is_smooth_enough_edge(riteE, nextCurrE, stop_y)
}
// The native caller only sets isUsingMask for this concrete additive subtype.
unsafe fn mask_row(blitter: *mut (dyn AdditiveBlitter + '_), y: i32) -> *mut u8 {
    (*(blitter as *mut MaskAdditiveBlitter<'_>)).getRow(y)
}
// Source: src/core/SkScan_AAAPath.cpp:1038-1305
unsafe fn aaa_walk_convex_edges(
    prevHead: *mut SkAnalyticEdge,
    blitter: *mut (dyn AdditiveBlitter + '_),
    _start_y: i32,
    stop_y: i32,
    leftBound: i32,
    riteBound: i32,
    isUsingMask: bool,
) {
    validate_sort((*prevHead).fNext);
    let mut leftE = (*prevHead).fNext;
    let mut riteE = (*leftE).fNext;
    let mut currE = (*riteE).fNext;
    let mut y = (*leftE).fUpperY.max((*riteE).fUpperY);
    'walk: loop {
        while (*leftE).fLowerY <= y {
            if !(*leftE).update(y) {
                if SkFixedFloorToInt((*currE).fUpperY) >= stop_y {
                    break 'walk;
                }
                leftE = currE;
                currE = (*currE).fNext;
            }
        }
        while (*riteE).fLowerY <= y {
            if !(*riteE).update(y) {
                if SkFixedFloorToInt((*currE).fUpperY) >= stop_y {
                    break 'walk;
                }
                riteE = currE;
                currE = (*currE).fNext;
            }
        }
        {
            crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 1075; @condition b"leftE\0".as_ptr(); !leftE.is_null());
            crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 1076; @condition b"riteE\0".as_ptr(); !riteE.is_null());
        };
        if SkFixedFloorToInt(y) >= stop_y {
            break;
        }
        {
            crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 1083; @condition b"SkFixedFloorToInt(leftE->fUpperY) <= stop_y\0".as_ptr(); SkFixedFloorToInt((*leftE).fUpperY)<=stop_y);
            crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 1084; @condition b"SkFixedFloorToInt(riteE->fUpperY) <= stop_y\0".as_ptr(); SkFixedFloorToInt((*riteE).fUpperY)<=stop_y);
        };
        (*leftE).goY(y);
        (*riteE).goY(y);
        if (*leftE).fX > (*riteE).fX || ((*leftE).fX == (*riteE).fX && (*leftE).fDX > (*riteE).fDX)
        {
            core::mem::swap(&mut leftE, &mut riteE);
        }
        let mut local_bot_fixed = (*leftE).fLowerY.min((*riteE).fLowerY);
        if is_smooth_enough(leftE, riteE, currE, stop_y) {
            local_bot_fixed = SkFixedCeilToFixed(local_bot_fixed);
        }
        local_bot_fixed = local_bot_fixed.min(SkIntToFixed(stop_y));
        let mut left = leftBound.max((*leftE).fX);
        let dLeft = (*leftE).fDX;
        let mut rite = riteBound.min((*riteE).fX);
        let dRite = (*riteE).fDX;
        if dLeft | dRite == 0 {
            let fullLeft = SkFixedCeilToInt(left);
            let fullRite = SkFixedFloorToInt(rite);
            let partialLeft = SkIntToFixed(fullLeft) - left;
            let partialRite = rite - SkIntToFixed(fullRite);
            let fullTop = SkFixedCeilToInt(y);
            let fullBot = SkFixedFloorToInt(local_bot_fixed);
            let mut partialTop = SkIntToFixed(fullTop) - y;
            let mut partialBot = local_bot_fixed - SkIntToFixed(fullBot);
            if fullTop > fullBot {
                partialTop -= SK_Fixed1 - partialBot;
                partialBot = 0;
            }
            if fullRite >= fullLeft {
                if partialTop > 0 {
                    if partialLeft > 0 {
                        core::pin::Pin::new_unchecked(&mut *blitter).blitAntiHPixel(
                            fullLeft - 1,
                            fullTop - 1,
                            fixed_to_alpha(SkFixedMul(partialTop, partialLeft)),
                        );
                    }
                    core::pin::Pin::new_unchecked(&mut *blitter).blitAntiHSpan(
                        fullLeft,
                        fullTop - 1,
                        fullRite - fullLeft,
                        fixed_to_alpha(partialTop),
                    );
                    if partialRite > 0 {
                        core::pin::Pin::new_unchecked(&mut *blitter).blitAntiHPixel(
                            fullRite,
                            fullTop - 1,
                            fixed_to_alpha(SkFixedMul(partialTop, partialRite)),
                        );
                    }
                    core::pin::Pin::new_unchecked(&mut *blitter)
                        .flush_if_y_changed(y, y + partialTop);
                }
                if fullBot > fullTop
                    && (fullRite > fullLeft
                        || fixed_to_alpha(partialLeft) > 0
                        || fixed_to_alpha(partialRite) > 0)
                {
                    core::pin::Pin::new_unchecked(&mut *blitter)
                        .getRealBlitter(false)
                        .blitAntiRect(
                            fullLeft - 1,
                            fullTop,
                            fullRite - fullLeft,
                            fullBot - fullTop,
                            fixed_to_alpha(partialLeft),
                            fixed_to_alpha(partialRite),
                        );
                }
                if partialBot > 0 {
                    if partialLeft > 0 {
                        core::pin::Pin::new_unchecked(&mut *blitter).blitAntiHPixel(
                            fullLeft - 1,
                            fullBot,
                            fixed_to_alpha(SkFixedMul(partialBot, partialLeft)),
                        );
                    }
                    core::pin::Pin::new_unchecked(&mut *blitter).blitAntiHSpan(
                        fullLeft,
                        fullBot,
                        fullRite - fullLeft,
                        fixed_to_alpha(partialBot),
                    );
                    if partialRite > 0 {
                        core::pin::Pin::new_unchecked(&mut *blitter).blitAntiHPixel(
                            fullRite,
                            fullBot,
                            fixed_to_alpha(SkFixedMul(partialBot, partialRite)),
                        );
                    }
                }
            } else {
                let width = rite - left;
                if width > 0 {
                    if partialTop > 0 {
                        core::pin::Pin::new_unchecked(&mut *blitter).blitAntiHSpan(
                            fullLeft - 1,
                            fullTop - 1,
                            1,
                            fixed_to_alpha(SkFixedMul(partialTop, width)),
                        );
                        core::pin::Pin::new_unchecked(&mut *blitter)
                            .flush_if_y_changed(y, y + partialTop);
                    }
                    if fullBot > fullTop {
                        core::pin::Pin::new_unchecked(&mut *blitter)
                            .getRealBlitter(false)
                            .blitV(
                                fullLeft - 1,
                                fullTop,
                                fullBot - fullTop,
                                fixed_to_alpha(width),
                            );
                    }
                    if partialBot > 0 {
                        core::pin::Pin::new_unchecked(&mut *blitter).blitAntiHSpan(
                            fullLeft - 1,
                            fullBot,
                            1,
                            fixed_to_alpha(SkFixedMul(partialBot, width)),
                        );
                    }
                }
            }
            y = local_bot_fixed;
        } else {
            const SNAP_DIGIT: i32 = SK_Fixed1 >> 4;
            const SNAP_HALF: i32 = SNAP_DIGIT >> 1;
            const SNAP_MASK: i32 = -1 ^ (SNAP_DIGIT - 1);
            left = left.wrapping_add(SNAP_HALF);
            rite = rite.wrapping_add(SNAP_HALF);
            let mut count = SkFixedCeilToInt(local_bot_fixed) - SkFixedFloorToInt(y);
            let mut maskRow = if isUsingMask {
                mask_row(blitter, y >> 16)
            } else {
                core::ptr::null_mut()
            };
            if count > 1 {
                if y & -65536 != y {
                    count -= 1;
                    let nextY = SkFixedCeilToFixed(y + 1);
                    let dY = nextY - y;
                    let nextLeft = left.wrapping_add(SkFixedMul(dLeft, dY));
                    let nextRite = rite.wrapping_add(SkFixedMul(dRite, dY));
                    crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 1220; @condition b"(left & kSnapMask) >= leftBound && (rite & kSnapMask) <= riteBound && (nextLeft & kSnapMask) >= leftBound && (nextRite & kSnapMask) <= riteBound\0".as_ptr(); (left & SNAP_MASK) >= leftBound
                            && (rite & SNAP_MASK) <= riteBound
                            && (nextLeft & SNAP_MASK) >= leftBound
                            && (nextRite & SNAP_MASK) <= riteBound);
                    blit_trapezoid_row(
                        blitter,
                        y >> 16,
                        left & SNAP_MASK,
                        rite & SNAP_MASK,
                        nextLeft & SNAP_MASK,
                        nextRite & SNAP_MASK,
                        (*leftE).fDY,
                        (*riteE).fDY,
                        get_partial_alpha_height(255, dY),
                        maskRow,
                        false,
                    );
                    core::pin::Pin::new_unchecked(&mut *blitter).flush_if_y_changed(y, nextY);
                    left = nextLeft;
                    rite = nextRite;
                    y = nextY;
                }
                while count > 1 {
                    count -= 1;
                    if isUsingMask {
                        maskRow = mask_row(blitter, y >> 16);
                    }
                    let nextY = y.wrapping_add(SK_Fixed1);
                    let nextLeft = left.wrapping_add(dLeft);
                    let nextRite = rite.wrapping_add(dRite);
                    crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 1246; @condition b"(left & kSnapMask) >= leftBound && (rite & kSnapMask) <= riteBound && (nextLeft & kSnapMask) >= leftBound && (nextRite & kSnapMask) <= riteBound\0".as_ptr(); (left & SNAP_MASK) >= leftBound
                            && (rite & SNAP_MASK) <= riteBound
                            && (nextLeft & SNAP_MASK) >= leftBound
                            && (nextRite & SNAP_MASK) <= riteBound);
                    blit_trapezoid_row(
                        blitter,
                        y >> 16,
                        left & SNAP_MASK,
                        rite & SNAP_MASK,
                        nextLeft & SNAP_MASK,
                        nextRite & SNAP_MASK,
                        (*leftE).fDY,
                        (*riteE).fDY,
                        255,
                        maskRow,
                        false,
                    );
                    core::pin::Pin::new_unchecked(&mut *blitter).flush_if_y_changed(y, nextY);
                    left = nextLeft;
                    rite = nextRite;
                    y = nextY;
                }
            }
            if isUsingMask {
                maskRow = mask_row(blitter, y >> 16);
            }
            let dY = local_bot_fixed - y;
            crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 1272; @condition b"dY <= SK_Fixed1\0".as_ptr(); dY <= SK_Fixed1);
            let nextLeft = left
                .wrapping_add(SkFixedMul(dLeft, dY))
                .max(leftBound.wrapping_add(SNAP_HALF));
            let nextRite = rite
                .wrapping_add(SkFixedMul(dRite, dY))
                .min(riteBound.wrapping_add(SNAP_HALF));
            crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 1278; @condition b"(left & kSnapMask) >= leftBound && (rite & kSnapMask) <= riteBound && (nextLeft & kSnapMask) >= leftBound && (nextRite & kSnapMask) <= riteBound\0".as_ptr(); (left & SNAP_MASK) >= leftBound
                    && (rite & SNAP_MASK) <= riteBound
                    && (nextLeft & SNAP_MASK) >= leftBound
                    && (nextRite & SNAP_MASK) <= riteBound);
            blit_trapezoid_row(
                blitter,
                y >> 16,
                left & SNAP_MASK,
                rite & SNAP_MASK,
                nextLeft & SNAP_MASK,
                nextRite & SNAP_MASK,
                (*leftE).fDY,
                (*riteE).fDY,
                get_partial_alpha_height(255, dY),
                maskRow,
                false,
            );
            core::pin::Pin::new_unchecked(&mut *blitter).flush_if_y_changed(y, local_bot_fixed);
            left = nextLeft;
            rite = nextRite;
            y = local_bot_fixed;
            left = left.wrapping_sub(SNAP_HALF);
            rite = rite.wrapping_sub(SNAP_HALF);
        }
        (*leftE).fX = left;
        (*riteE).fX = rite;
        (*leftE).fY = y;
        (*riteE).fY = y;
    }
}
use crate::src::core::SkScanPriv::{
    backward_insert_edge_based_on_x, backward_insert_start, insert_edge_after, remove_edge,
};
// Source: src/core/SkScan_AAAPath.cpp:1307-1309
fn update_next_next_y(y: i32, nextY: i32, nextNextY: &mut i32) {
    if y > nextY && y < *nextNextY {
        *nextNextY = y;
    }
}
// Source: src/core/SkScan_AAAPath.cpp:1311-1315
unsafe fn check_intersection(edge: *const SkAnalyticEdge, nextY: i32, nextNextY: &mut i32) {
    let prev = (*edge).fPrev;
    if !(*prev).fPrev.is_null()
        && (*prev).fX.wrapping_add((*prev).fDX) > (*edge).fX.wrapping_add((*edge).fDX)
    {
        *nextNextY = nextY.wrapping_add(SK_Fixed1 >> SkAnalyticEdge::kDefaultAccuracy);
    }
}
// Source: src/core/SkScan_AAAPath.cpp:1317-1321
unsafe fn check_intersection_fwd(edge: *const SkAnalyticEdge, nextY: i32, nextNextY: &mut i32) {
    let next = (*edge).fNext;
    if !(*next).fNext.is_null()
        && (*edge).fX.wrapping_add((*edge).fDX) > (*next).fX.wrapping_add((*next).fDX)
    {
        *nextNextY = nextY.wrapping_add(SK_Fixed1 >> SkAnalyticEdge::kDefaultAccuracy);
    }
}
// Source: src/core/SkScan_AAAPath.cpp:1323-1364
unsafe fn insert_new_edges(mut newEdge: *mut SkAnalyticEdge, y: i32, nextNextY: &mut i32) {
    if (*newEdge).fUpperY > y {
        update_next_next_y((*newEdge).fUpperY, y, nextNextY);
        return;
    }
    let prev = (*newEdge).fPrev;
    if (*prev).fX <= (*newEdge).fX {
        while (*newEdge).fUpperY <= y {
            check_intersection(newEdge, y, nextNextY);
            update_next_next_y((*newEdge).fLowerY, y, nextNextY);
            newEdge = (*newEdge).fNext;
        }
        update_next_next_y((*newEdge).fUpperY, y, nextNextY);
        return;
    }
    let mut start = backward_insert_start(prev, (*newEdge).fX);
    loop {
        let next = (*newEdge).fNext;
        let mut already_linked = false;
        loop {
            if (*start).fNext == newEdge {
                already_linked = true;
                break;
            }
            let after = (*start).fNext;
            if (*after).fX >= (*newEdge).fX {
                break;
            }
            crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 1351; @condition b"start != after\0".as_ptr(); start != after);
            start = after;
        }
        if !already_linked {
            remove_edge(newEdge);
            insert_edge_after(newEdge, start);
        }
        check_intersection(newEdge, y, nextNextY);
        check_intersection_fwd(newEdge, y, nextNextY);
        update_next_next_y((*newEdge).fLowerY, y, nextNextY);
        start = newEdge;
        newEdge = next;
        if (*newEdge).fUpperY > y {
            break;
        }
    }
    update_next_next_y((*newEdge).fUpperY, y, nextNextY);
}
// Source: src/core/SkScan_AAAPath.cpp:1366-1377
unsafe fn validate_edges_for_y(mut edge: *const SkAnalyticEdge, y: i32) {
    #[cfg(sk_debug)]
    {
        while (*edge).fUpperY <= y {
            crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 1369; @condition b"edge->fPrev && edge->fNext\0".as_ptr(); !(*edge).fPrev.is_null() && !(*edge).fNext.is_null());
            {
                crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 1370; @condition b"edge->fPrev->fNext == edge\0".as_ptr(); (*(*edge).fPrev).fNext == edge as *mut _);
                crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 1371; @condition b"edge->fNext->fPrev == edge\0".as_ptr(); (*(*edge).fNext).fPrev == edge as *mut _);
            };
            crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 1372; @condition b"edge->fUpperY <= edge->fLowerY\0".as_ptr(); (*edge).fUpperY <= (*edge).fLowerY);
            crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 1373; @condition b"edge->fPrev->fPrev == nullptr || edge->fPrev->fX <= edge->fX\0".as_ptr(); (*(*edge).fPrev).fPrev.is_null() || (*(*edge).fPrev).fX <= (*edge).fX);
            edge = (*edge).fNext;
        }
    }
}
// Source: src/core/SkScan_AAAPath.cpp:1380-1397
unsafe fn edges_too_close(
    prev: *const SkAnalyticEdge,
    next: *const SkAnalyticEdge,
    lowerY: i32,
) -> bool {
    const SLACK: i32 = SK_Fixed1;
    !next.is_null()
        && !prev.is_null()
        && (*next).fUpperY < lowerY
        && (*prev).fX.wrapping_add(SLACK) >= (*next).fX.wrapping_sub((*next).fDX.wrapping_abs())
}
// Source: src/core/SkScan_AAAPath.cpp:1401-1403
fn edges_too_close_removed(prevRite: i32, ul: i32, ll: i32) -> bool {
    prevRite > SkFixedFloorToInt(ul) || prevRite > SkFixedFloorToInt(ll)
}
// Source: src/core/SkScan_AAAPath.cpp:1405-1602
unsafe fn aaa_walk_edges(
    prevHead: *mut SkAnalyticEdge,
    nextTail: *mut SkAnalyticEdge,
    fillType: crate::include::core::SkPathTypes::SkPathFillType,
    blitter: *mut (dyn AdditiveBlitter + '_),
    mut start_y: i32,
    stop_y: i32,
    leftClip: i32,
    rightClip: i32,
    isUsingMask: bool,
    forceRLE: bool,
    skipIntersect: bool,
) {
    use crate::include::core::SkPathTypes::{SkPathFillType_IsEvenOdd, SkPathFillType_IsInverse};
    (*prevHead).fX = leftClip;
    (*prevHead).fUpperX = leftClip;
    (*nextTail).fX = rightClip;
    (*nextTail).fUpperX = rightClip;
    let mut y = (*(*prevHead).fNext).fUpperY.max(SkIntToFixed(start_y));
    let mut nextNextY = i32::MAX;
    let mut edge = (*prevHead).fNext;
    while (*edge).fUpperY <= y {
        (*edge).goY(y);
        update_next_next_y((*edge).fLowerY, y, &mut nextNextY);
        edge = (*edge).fNext;
    }
    update_next_next_y((*edge).fUpperY, y, &mut nextNextY);
    let windingMask = if SkPathFillType_IsEvenOdd(fillType) {
        1
    } else {
        -1
    };
    let isInverse = SkPathFillType_IsInverse(fillType);
    if isInverse && SkIntToFixed(start_y) != y {
        let width = SkFixedFloorToInt(rightClip - leftClip);
        if SkFixedFloorToInt(y) != start_y {
            core::pin::Pin::new_unchecked(&mut *blitter)
                .getRealBlitter(false)
                .blitRect(
                    SkFixedFloorToInt(leftClip),
                    start_y,
                    width,
                    SkFixedFloorToInt(y) - start_y,
                );
            start_y = SkFixedFloorToInt(y);
        }
        let row = if isUsingMask {
            mask_row(blitter, start_y)
        } else {
            core::ptr::null_mut()
        };
        blit_full_alpha(
            blitter,
            start_y,
            SkFixedFloorToInt(leftClip),
            width,
            fixed_to_alpha(y - SkIntToFixed(start_y)),
            row,
            false,
        );
    }
    loop {
        let mut w = 0i32;
        let mut in_interval = isInverse;
        let mut prevX = (*prevHead).fX;
        let mut nextY = nextNextY.min(SkFixedCeilToFixed(y + 1));
        let mut currE = (*prevHead).fNext;
        let mut leftE = prevHead;
        let mut left = leftClip;
        let mut leftDY = 0;
        let mut prevRite = SkFixedFloorToInt(leftClip);
        nextNextY = i32::MAX;
        crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 1464; @condition b"(nextY & ((SK_Fixed1 >> 2) - 1)) == 0\0".as_ptr(); (nextY & ((SK_Fixed1 >> 2) - 1)) == 0);
        let mut yShift = 0;
        if (nextY - y) & (SK_Fixed1 >> 2) != 0 {
            yShift = 2;
            nextY = y + (SK_Fixed1 >> 2);
        } else if (nextY - y) & (SK_Fixed1 >> 1) != 0 {
            yShift = 1;
            crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 1471; @condition b"nextY == y + (SK_Fixed1 >> 1)\0".as_ptr(); nextY == y + (SK_Fixed1 >> 1));
        }
        let fullAlpha = fixed_to_alpha(nextY - y);
        let row = if isUsingMask {
            mask_row(blitter, SkFixedFloorToInt(y))
        } else {
            core::ptr::null_mut()
        };
        crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 1483; @condition b"currE->fPrev == prevHead\0".as_ptr(); (*currE).fPrev == prevHead);
        validate_edges_for_y(currE, y);
        let noRealBlitter = forceRLE;
        while (*currE).fUpperY <= y {
            {
                crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 1491; @condition b"currE->fLowerY >= nextY\0".as_ptr(); (*currE).fLowerY>=nextY);
                crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 1492; @condition b"currE->fY == y\0".as_ptr(); (*currE).fY==y);
            };
            w += (*currE).fWinding as i32;
            let prev_in_interval = in_interval;
            in_interval = ((w & windingMask) == 0) == isInverse;
            let isLeft = in_interval && !prev_in_interval;
            let isRite = !in_interval && prev_in_interval;
            if isRite {
                let mut rite = (*currE).fX;
                (*currE).goYShift(nextY, yShift);
                let nextLeft = leftClip.max((*leftE).fX);
                rite = rightClip.min(rite);
                let nextRite = rightClip.min((*currE).fX);
                blit_trapezoid_row(
                    blitter,
                    y >> 16,
                    left,
                    rite,
                    nextLeft,
                    nextRite,
                    leftDY,
                    (*currE).fDY,
                    fullAlpha,
                    row,
                    noRealBlitter
                        || (fullAlpha == 255
                            && (edges_too_close_removed(prevRite, left, (*leftE).fX)
                                || edges_too_close(currE, (*currE).fNext, nextY))),
                );
                prevRite = SkFixedCeilToInt(rite.max((*currE).fX));
            } else {
                if isLeft {
                    left = (*currE).fX.max(leftClip);
                    leftDY = (*currE).fDY;
                    leftE = currE;
                }
                (*currE).goYShift(nextY, yShift);
            }
            let next = (*currE).fNext;
            while (*currE).fLowerY <= nextY {
                if (*currE).fCurveCount < 0 {
                    let cubic = &mut *currE.cast::<SkAnalyticCubicEdge>();
                    cubic.keepContinuous();
                    if !cubic.updateCubic() {
                        break;
                    }
                } else if (*currE).fCurveCount > 0 {
                    let quad = &mut *currE.cast::<SkAnalyticQuadraticEdge>();
                    quad.keepContinuous();
                    if !quad.updateQuadratic() {
                        break;
                    }
                } else {
                    break;
                }
            }
            crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 1551; @condition b"currE->fY == nextY\0".as_ptr(); (*currE).fY == nextY);
            if (*currE).fLowerY <= nextY {
                remove_edge(currE);
            } else {
                update_next_next_y((*currE).fLowerY, nextY, &mut nextNextY);
                let newX = (*currE).fX;
                crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 1558; @condition b"currE->fLowerY > nextY\0".as_ptr(); (*currE).fLowerY > nextY);
                if newX < prevX {
                    backward_insert_edge_based_on_x(currE);
                } else {
                    prevX = newX;
                }
                if !skipIntersect {
                    check_intersection(currE, nextY, &mut nextNextY);
                }
            }
            currE = next;
            crate::SkASSERT!(@source b"src/core/SkScan_AAAPath.cpp\0".as_ptr(), 1571; @condition b"currE\0".as_ptr(); !currE.is_null());
        }
        if in_interval {
            blit_trapezoid_row(
                blitter,
                y >> 16,
                left,
                rightClip,
                leftClip.max((*leftE).fX),
                rightClip,
                leftDY,
                0,
                fullAlpha,
                row,
                noRealBlitter
                    || (fullAlpha == 255 && edges_too_close((*leftE).fPrev, leftE, nextY)),
            );
        }
        if forceRLE {
            core::pin::Pin::new_unchecked(&mut *blitter).flush_if_y_changed(y, nextY);
        }
        y = nextY;
        if y >= SkIntToFixed(stop_y) {
            break;
        }
        insert_new_edges(currE, y, &mut nextNextY);
    }
}
// Source: src/core/SkScan_AAAPath.cpp:1604-1703
unsafe fn aaa_fill_path<const EVEN: bool>(
    path: &crate::src::core::SkPathRaw::SkPathRaw<'_>,
    clipRect: &crate::src::core::SkRect::SkIRect,
    blitter: *mut (dyn AdditiveBlitter + '_),
    mut start_y: i32,
    mut stop_y: i32,
    pathContainedInClip: bool,
    isUsingMask: bool,
    forceRLE: bool,
) {
    let mut builder = crate::src::core::SkEdgeBuilder::SkAnalyticEdgeBuilder::new();
    let count = builder.buildEdges::<EVEN>(
        path,
        if pathContainedInClip {
            None
        } else {
            Some(clipRect)
        },
    );
    let mut rect = *clipRect;
    if count == 0 {
        if path.isInverseFillType() {
            if rect.fTop < start_y {
                rect.fTop = start_y;
            }
            if rect.fBottom > stop_y {
                rect.fBottom = stop_y;
            }
            if !rect.is_empty() {
                core::pin::Pin::new_unchecked(&mut *blitter)
                    .getRealBlitter(false)
                    .blitRect(rect.fLeft, rect.fTop, rect.width(), rect.height());
            }
        }
        return;
    }
    let mut headEdge = SkAnalyticEdge::new();
    let mut tailEdge = SkAnalyticEdge::new();
    let mut last = core::ptr::null_mut();
    let edge = sort_edges(builder.analyticEdgeList(), &mut last);
    headEdge.fPrev = core::ptr::null_mut();
    headEdge.fNext = edge;
    headEdge.fUpperY = i32::MIN;
    headEdge.fLowerY = i32::MIN;
    headEdge.fX = i32::MIN;
    headEdge.fDX = 0;
    headEdge.fDY = i32::MAX;
    headEdge.fUpperX = i32::MIN;
    (*edge).fPrev = &mut headEdge;
    tailEdge.fPrev = last;
    tailEdge.fNext = core::ptr::null_mut();
    tailEdge.fUpperY = i32::MAX;
    tailEdge.fLowerY = i32::MAX;
    tailEdge.fX = i32::MAX;
    tailEdge.fDX = 0;
    tailEdge.fDY = i32::MAX;
    tailEdge.fUpperX = i32::MAX;
    (*last).fNext = &mut tailEdge;
    if !pathContainedInClip && start_y < clipRect.fTop {
        start_y = clipRect.fTop;
    }
    if !pathContainedInClip && stop_y > clipRect.fBottom {
        stop_y = clipRect.fBottom;
    }
    let mut leftBound = SkIntToFixed(rect.fLeft);
    let mut rightBound = SkIntToFixed(rect.fRight);
    if isUsingMask {
        let ir = path.bounds().roundOut();
        leftBound = leftBound.max(SkIntToFixed(ir.fLeft));
        rightBound = rightBound.min(SkIntToFixed(ir.fRight));
    }
    if !path.isInverseFillType() && path.isKnownToBeConvex() && count >= 2 {
        aaa_walk_convex_edges(
            &mut headEdge,
            blitter,
            start_y,
            stop_y,
            leftBound,
            rightBound,
            isUsingMask,
        );
    } else {
        let skipIntersect = path.points().len() > ((stop_y - start_y) * 2) as usize;
        aaa_walk_edges(
            &mut headEdge,
            &mut tailEdge,
            path.fillType(),
            blitter,
            start_y,
            stop_y,
            leftBound,
            rightBound,
            isUsingMask,
            forceRLE,
            skipIntersect,
        );
    }
}
// Source: src/core/SkScan_AAAPath.cpp:1706-1722
unsafe fn try_blit_fat_anti_rect(
    mut blitter: core::pin::Pin<&mut dyn crate::src::core::SkBlitter::SkBlitter>,
    raw: &crate::src::core::SkPathRaw::SkPathRaw<'_>,
    clip: &crate::src::core::SkRect::SkIRect,
) -> bool {
    let Some(mut rect) = raw.isRect() else {
        return false;
    };
    if !rect.intersect(&crate::src::core::SkRect::SkRect::Make(clip)) {
        return true;
    }
    let bounds = rect.roundOut();
    if bounds.width() < 3 {
        return false;
    }
    blitter.as_mut().blitFatAntiRect(&rect);
    true
}
impl crate::src::core::SkScan::SkScan {
    pub unsafe fn AAAFillPath(
        path: &crate::src::core::SkPathRaw::SkPathRaw<'_>,
        mut blitter: core::pin::Pin<&mut dyn crate::src::core::SkBlitter::SkBlitter>,
        ir: &crate::src::core::SkRect::SkIRect,
        clipBounds: &crate::src::core::SkRect::SkIRect,
        forceRLE: bool,
    ) {
        Self::AAAFillPathWithRounding::<false>(path, blitter.as_mut(), ir, clipBounds, forceRLE);
    }
    // Source: src/core/SkScan_AAAPath.cpp:1724-1781
    pub unsafe fn AAAFillPathWithRounding<const EVEN: bool>(
        path: &crate::src::core::SkPathRaw::SkPathRaw<'_>,
        mut blitter: core::pin::Pin<&mut dyn crate::src::core::SkBlitter::SkBlitter>,
        ir: &crate::src::core::SkRect::SkIRect,
        clipBounds: &crate::src::core::SkRect::SkIRect,
        forceRLE: bool,
    ) {
        let containedInClip = clipBounds.contains_rect(ir);
        let isInverse = path.isInverseFillType();
        unsafe {
            if MaskAdditiveBlitter::CanHandleRect(ir) && !isInverse && !forceRLE {
                if !try_blit_fat_anti_rect(blitter.as_mut(), path, clipBounds) {
                    let mut additive = Box::pin(MaskAdditiveBlitter::new(
                        blitter.as_mut(),
                        ir,
                        clipBounds,
                        isInverse,
                    ));
                    aaa_fill_path::<EVEN>(
                        path,
                        clipBounds,
                        additive.as_mut().get_unchecked_mut() as *mut dyn AdditiveBlitter,
                        ir.fTop,
                        ir.fBottom,
                        containedInClip,
                        true,
                        forceRLE,
                    );
                }
            } else if !isInverse && path.isKnownToBeConvex() {
                let mut additive = Box::pin(RunBasedAdditiveBlitter::new(
                    blitter.as_mut(),
                    ir,
                    clipBounds,
                    isInverse,
                ));
                aaa_fill_path::<EVEN>(
                    path,
                    clipBounds,
                    additive.as_mut().get_unchecked_mut() as *mut dyn AdditiveBlitter,
                    ir.fTop,
                    ir.fBottom,
                    containedInClip,
                    false,
                    forceRLE,
                );
            } else {
                let mut additive = Box::pin(SafeRLEAdditiveBlitter::new(
                    blitter.as_mut(),
                    ir,
                    clipBounds,
                    isInverse,
                ));
                aaa_fill_path::<EVEN>(
                    path,
                    clipBounds,
                    additive.as_mut().get_unchecked_mut() as *mut dyn AdditiveBlitter,
                    ir.fTop,
                    ir.fBottom,
                    containedInClip,
                    false,
                    forceRLE,
                );
            }
        }
    }
}
