// RasterError interface/propagation adaptation only; UNVERIFIED; no added native-line credit.
// Copyright 2006 The Android Open Source Project. BSD-style license.
// Rust dynamic dispatch replaces the upstream virtual base. Sparse alpha/run
// slices are mutable because clipping subclasses mutate them in upstream code.
// Primitive, mask, allocation and clip-wrapper behavior follows the native source.
// Concrete factories are owned by the raster_blit translation group.
// Source: src/core/SkBlitter.cpp:45-45
// Adapted shared configuration: atomic access replaces the native mutable bool.
// Relaxed ordering carries only this independent testing selection flag.
pub static gSkForceRasterPipelineBlitter: core::sync::atomic::AtomicBool =
    core::sync::atomic::AtomicBool::new(false);
// Source: src/core/SkBlitter.cpp:49-52
pub fn scalar_to_alpha(a: f32) -> u8 {
    let alpha = (a * 255.0) as u8;
    if alpha > 247 {
        255
    } else if alpha < 8 {
        0
    } else {
        alpha
    }
}
/// Safety: allocBlitMemory(sz) must return uniquely writable storage covering
/// sz bytes, aligned for i16, alive until this blitter's next allocation or drop.
/// blitAntiH must not free/invalidate its borrowed alpha/run buffers during the
/// call. requestRowsPreserved describes rows retained after blitAntiH, and the
/// returned allocation must remain live for those rows; the row count is positive.
/// Implementations retain
/// the native sparse-run termination and bounds contracts of all primitive calls.
///
/// # Primitive caller safety
/// Every unsafe drawing method requires a live, exclusively writable destination
/// and all backing pixel/context/arena leases through the call and any deferred
/// flush/drop. Coordinates and lengths must address that destination (or the
/// wrapper's documented clipping domain). Sparse runs must be initialized,
/// terminating, positive when traversed, and span the corresponding alpha data.
/// Implementations may write raw pixels without performing bounds checks.
pub unsafe trait SkBlitter {
    // Source: src/core/SkBlitter.h:129-129
    // Implementors advertising direct access must retain a writable pixmap
    // lease for all returned bounds through the drawing call.
    fn canDirectBlit(mut self: core::pin::Pin<&mut Self>) -> Option<DirectBlit> {
        None
    }

    fn allocBlitMemory(self: core::pin::Pin<&mut Self>, sz: usize) -> *mut u8;

    // Source: src/core/SkBlitter.h:50-50
    unsafe fn blitH(self: core::pin::Pin<&mut Self>, x: i32, y: i32, width: i32);
    // Source: src/core/SkBlitter.h:64-64
    unsafe fn blitAntiH(
        self: core::pin::Pin<&mut Self>,
        x: i32,
        y: i32,
        antialias: &mut [u8],
        runs: &mut [i16],
    );
    // Source: src/core/SkBlitter.h:88-98
    unsafe fn blitAntiH2(mut self: core::pin::Pin<&mut Self>, x: i32, y: i32, a0: u32, a1: u32) {
        let mut runs = [1i16, 1, 0];
        let mut aa = [a0 as u8, a1 as u8];
        self.as_mut().blitAntiH(x, y, &mut aa, &mut runs);
    }
    // Source: src/core/SkBlitter.h:101-114
    unsafe fn blitAntiV2(mut self: core::pin::Pin<&mut Self>, x: i32, y: i32, a0: u32, a1: u32) {
        let mut runs = [1i16, 0];
        let mut aa = [a0 as u8];
        self.as_mut().blitAntiH(x, y, &mut aa, &mut runs);
        runs[0] = 1;
        runs[1] = 0;
        aa[0] = a1 as u8;
        self.as_mut().blitAntiH(x, y + 1, &mut aa, &mut runs);
    }
    // Source: src/core/SkBlitter.h:122-122
    fn requestRowsPreserved(&self) -> i32 {
        1
    }
    unsafe fn blitFatAntiRect(
        mut self: core::pin::Pin<&mut Self>,
        rect: &crate::src::core::SkRect::SkRect,
    ) {
        blitFatAntiRect(self, rect)
    }
    unsafe fn blitV(
        mut self: core::pin::Pin<&mut Self>,
        x: i32,
        mut y: i32,
        mut height: i32,
        alpha: u8,
    ) {
        blitV(self, x, y, height, alpha)
    }
    unsafe fn blitRect(
        mut self: core::pin::Pin<&mut Self>,
        x: i32,
        mut y: i32,
        width: i32,
        mut height: i32,
    ) {
        blitRect(self, x, y, width, height)
    }
    unsafe fn blitAntiRect(
        mut self: core::pin::Pin<&mut Self>,
        mut x: i32,
        y: i32,
        width: i32,
        height: i32,
        left_alpha: u8,
        right_alpha: u8,
    ) {
        blitAntiRect(self, x, y, width, height, left_alpha, right_alpha)
    }
    // Source: src/core/SkBlitter.h:142-142
    /// Safety: uphold blitMask's initialized mask and exclusive destination
    /// lease for every rectangle produced by the actual region intersection.
    unsafe fn blitMaskRegion(
        mut self: core::pin::Pin<&mut Self>,
        mask: &crate::src::core::SkMask::SkMask,
        clip: &crate::src::core::SkRegion::SkRegion,
    ) {
        blitMaskRegion(self, mask, clip)
    }
    unsafe fn blitRectRegion(
        mut self: core::pin::Pin<&mut Self>,
        rect: &crate::src::core::SkRect::SkIRect,
        clip: &crate::src::core::SkRegion::SkRegion,
    ) {
        blitRectRegion(self, rect, clip)
    }
    unsafe fn blitRegion(
        mut self: core::pin::Pin<&mut Self>,
        clip: &crate::src::core::SkRegion::SkRegion,
    ) {
        blitRegion(self, clip)
    }
    unsafe fn blitMask(
        mut self: core::pin::Pin<&mut Self>,
        mask: &crate::src::core::SkMask::SkMask,
        clip: &crate::src::core::SkRect::SkIRect,
    ) {
        unsafe { blitMask(self, mask, clip) }
    }
}
// Source: src/core/SkBlitter.h:137-139
// Source: src/core/SkBlitter.h:169-169
// Rust composition retains one allocation owner per original base-class instance.
pub struct SkBlitterStorage {
    _pin: core::marker::PhantomPinned,
    fBlitMemory: crate::src::core::SkAutoMalloc::SkAutoMalloc,
}
impl SkBlitterStorage {
    pub fn new() -> Self {
        Self {
            _pin: core::marker::PhantomPinned,
            fBlitMemory: crate::src::core::SkAutoMalloc::SkAutoMalloc::new(0),
        }
    }
    pub fn allocBlitMemory(&mut self, sz: usize) -> *mut u8 {
        self.fBlitMemory.reset(
            sz,
            crate::src::core::SkAutoMalloc::OnShrink::kReuse_OnShrink,
        )
    }
}

// Source: src/core/SkBlitter.cpp:54-104
unsafe fn blitFatAntiRect<B: SkBlitter + ?Sized>(
    mut blitter: core::pin::Pin<&mut B>,
    rect: &crate::src::core::SkRect::SkRect,
) {
    let bounds = rect.roundOut();
    crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 56; @condition b"bounds.width() >= 3\0".as_ptr(); bounds.width() >= 3);
    if bounds.height() == 0 {
        return;
    }
    let run_size = (bounds.width() + 1) as usize;
    let storage = blitter
        .as_mut()
        .allocBlitMemory(run_size * (core::mem::size_of::<i16>() + core::mem::size_of::<u8>()));
    // Initialize unused sparse slots for Rust's slice validity. Their values do
    // not enter upstream run traversal; storage still belongs to this blitter.
    let (runs, alphas) = unsafe {
        storage.write_bytes(0, run_size * 3);
        (
            core::slice::from_raw_parts_mut(storage as *mut i16, run_size),
            core::slice::from_raw_parts_mut(storage.add(run_size * 2), run_size),
        )
    };
    runs[0] = 1;
    runs[1] = (bounds.width() - 2) as i16;
    runs[bounds.width() as usize - 1] = 1;
    runs[bounds.width() as usize] = 0;
    let partial_l = (bounds.fLeft + 1) as f32 - rect.fLeft;
    let partial_r = rect.fRight - (bounds.fRight - 1) as f32;
    let mut partial_t = (bounds.fTop + 1) as f32 - rect.fTop;
    let partial_b = rect.fBottom - (bounds.fBottom - 1) as f32;
    if bounds.height() == 1 {
        partial_t = rect.fBottom - rect.fTop;
    }
    alphas[0] = scalar_to_alpha(partial_l * partial_t);
    alphas[1] = scalar_to_alpha(partial_t);
    alphas[bounds.width() as usize - 1] = scalar_to_alpha(partial_r * partial_t);
    blitter
        .as_mut()
        .blitAntiH(bounds.fLeft, bounds.fTop, alphas, runs);
    if bounds.height() > 2 {
        blitter.as_mut().blitAntiRect(
            bounds.fLeft,
            bounds.fTop + 1,
            bounds.width() - 2,
            bounds.height() - 2,
            scalar_to_alpha(partial_l),
            scalar_to_alpha(partial_r),
        );
    }
    if bounds.height() > 1 {
        alphas[0] = scalar_to_alpha(partial_l * partial_b);
        alphas[1] = scalar_to_alpha(partial_b);
        alphas[bounds.width() as usize - 1] = scalar_to_alpha(partial_r * partial_b);
        blitter
            .as_mut()
            .blitAntiH(bounds.fLeft, bounds.fBottom - 1, alphas, runs);
    }
}
// Source: src/core/SkBlitter.cpp:106-118
pub(crate) unsafe fn blitV<B: SkBlitter + ?Sized>(
    mut blitter: core::pin::Pin<&mut B>,
    x: i32,
    mut y: i32,
    mut height: i32,
    alpha: u8,
) {
    if alpha == 255 {
        blitter.as_mut().blitRect(x, y, 1, height);
    } else {
        let mut runs = [1i16, 0];
        let mut aa = [alpha];
        loop {
            height -= 1;
            if height < 0 {
                break;
            }
            blitter.as_mut().blitAntiH(x, y, &mut aa, &mut runs);
            y += 1;
        }
    }
}
// Source: src/core/SkBlitter.cpp:120-125
unsafe fn blitRect<B: SkBlitter + ?Sized>(
    mut blitter: core::pin::Pin<&mut B>,
    x: i32,
    mut y: i32,
    width: i32,
    mut height: i32,
) {
    crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 121; @condition b"width > 0\0".as_ptr(); width > 0);
    loop {
        height -= 1;
        if height < 0 {
            break;
        }
        blitter.as_mut().blitH(x, y, width);
        y += 1;
    }
}
// Source: src/core/SkBlitter.cpp:130-143
unsafe fn blitAntiRect<B: SkBlitter + ?Sized>(
    mut blitter: core::pin::Pin<&mut B>,
    mut x: i32,
    y: i32,
    width: i32,
    height: i32,
    left_alpha: u8,
    right_alpha: u8,
) {
    if left_alpha > 0 {
        blitter.as_mut().blitV(x, y, height, left_alpha);
    }
    x += 1;
    if width > 0 {
        blitter.as_mut().blitRect(x, y, width, height);
        x += width;
    }
    if right_alpha > 0 {
        blitter.as_mut().blitV(x, y, height, right_alpha);
    }
}
// Source: src/core/SkBlitter.cpp:147-181
pub unsafe fn bits_to_runs<B: SkBlitter + ?Sized>(
    mut blitter: core::pin::Pin<&mut B>,
    mut x: i32,
    y: i32,
    bits: &[u8],
    mut left_mask: u8,
    mut row_bytes: isize,
    right_mask: u8,
) {
    let mut in_fill = false;
    let mut pos = 0;
    let mut offset = 0;
    loop {
        row_bytes -= 1;
        if row_bytes < 0 {
            break;
        }
        let mut b = bits[offset] & left_mask;
        offset += 1;
        if row_bytes == 0 {
            b &= right_mask;
        }
        let mut test = 0x80u8;
        while test != 0 {
            if b & test != 0 {
                if !in_fill {
                    pos = x;
                    in_fill = true;
                }
            } else if in_fill {
                blitter.as_mut().blitH(pos, y, x - pos);
                in_fill = false;
            }
            x += 1;
            test >>= 1;
        }
        left_mask = 255;
    }
    if in_fill {
        blitter.as_mut().blitH(pos, y, x - pos);
    }
}
// Source: src/core/SkBlitter.cpp:184-186
pub fn generate_right_mask(mask_bit_count: u32) -> u8 {
    ((0xff00u32 >> mask_bit_count) & 255) as u8
}
// Source: src/core/SkBlitter.cpp:188-267
/// Safety: mask storage must remain valid for every addressed row. The A8
/// storage must allow the same in-place clipping mutations as upstream's
/// const_cast path when a clipping subclass receives the sparse spans.
/// Safety: the mask backing storage must cover its bounds and stride for the
/// duration of this call. A8 storage must permit mutable blitAntiH clipping.
pub unsafe fn blitMask<B: SkBlitter + ?Sized>(
    mut blitter: core::pin::Pin<&mut B>,
    mask: &crate::src::core::SkMask::SkMask,
    clip: &crate::src::core::SkRect::SkIRect,
) {
    use crate::src::core::SkMask::Format;
    crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 189; @condition b"mask.fBounds.contains(clip)\0".as_ptr(); mask.fBounds.contains_rect(clip));
    if mask.fFormat == Format::kLCD16_Format {
        return;
    }
    if mask.fFormat == Format::kBW_Format {
        let cx = clip.fLeft;
        let mut cy = clip.fTop;
        let mask_left = mask.fBounds.fLeft;
        let mask_row_bytes = mask.fRowBytes as usize;
        let mut height = clip.height();
        let mut bits = mask.getAddr1(cx, cy);
        #[cfg(sk_debug)]
        let end_of_image = mask.fImage.add(
            (mask.fBounds.height() as usize - 1) * mask_row_bytes
                + ((mask.fBounds.width() as usize + 7) >> 3),
        );
        if cx == mask_left && clip.fRight == mask.fBounds.fRight {
            loop {
                height -= 1;
                if height < 0 {
                    break;
                }
                let affected_right_bit = mask.fBounds.width() - 1;
                let row_bytes = (affected_right_bit >> 3) + 1;
                crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 212; @condition b"bits + rowBytes <= endOfImage\0".as_ptr(); bits.add(row_bytes as usize) <= end_of_image);
                let right_mask = generate_right_mask(((affected_right_bit & 7) + 1) as u32);
                bits_to_runs(
                    blitter.as_mut(),
                    cx,
                    cy,
                    core::slice::from_raw_parts(bits, row_bytes as usize),
                    255,
                    row_bytes as isize,
                    right_mask,
                );
                bits = bits.add(mask_row_bytes);
                cy += 1;
            }
        } else {
            let bits_left = cx - ((cx - mask_left) & 7);
            let left_edge = cx - bits_left;
            crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 226; @condition b"leftEdge >= 0\0".as_ptr(); left_edge >= 0);
            let right_edge = clip.fRight - bits_left;
            crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 228; @condition b"rightEdge > leftEdge\0".as_ptr(); right_edge > left_edge);
            let left_mask = 255u8 >> (left_edge & 7);
            let affected_right_bit = right_edge - 1;
            let right_mask = generate_right_mask(((affected_right_bit & 7) + 1) as u32);
            let row_bytes = (affected_right_bit >> 3) + 1;
            loop {
                height -= 1;
                if height < 0 {
                    break;
                }
                crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 244; @condition b"bits + rowBytes <= endOfImage\0".as_ptr(); bits.add(row_bytes as usize) <= end_of_image);
                bits_to_runs(
                    blitter.as_mut(),
                    bits_left,
                    cy,
                    core::slice::from_raw_parts(bits, row_bytes as usize),
                    left_mask,
                    row_bytes as isize,
                    right_mask,
                );
                bits = bits.add(mask_row_bytes);
                cy += 1;
            }
        }
    } else {
        let width = clip.width() as usize;
        let mut runs = vec![1i16; width + 1];
        runs[width] = 0;
        let mut aa = mask.getAddr8(clip.fLeft, clip.fTop);
        let mut height = clip.height();
        let mut y = clip.fTop;
        loop {
            height -= 1;
            if height < 0 {
                break;
            }
            blitter.as_mut().blitAntiH(
                clip.fLeft,
                y,
                core::slice::from_raw_parts_mut(aa.cast_mut(), width),
                &mut runs,
            );
            aa = aa.add(mask.fRowBytes as usize);
            y += 1;
        }
    }
}
// Source: src/core/SkBlitter.h:187-193
pub struct SkRectClipBlitter<'a> {
    _pin: core::marker::PhantomPinned,
    fBlitter: core::pin::Pin<&'a mut dyn SkBlitter>,
    fClipRect: crate::src::core::SkRect::SkIRect,
}
impl<'a> SkRectClipBlitter<'a> {
    pub fn new(
        blitter: core::pin::Pin<&'a mut dyn SkBlitter>,
        clip: crate::src::core::SkRect::SkIRect,
    ) -> Self {
        crate::SkASSERT!(@source b"src/core/SkBlitter.h\0".as_ptr(), 190; @condition b"!clipRect.isEmpty()\0".as_ptr(); !clip.is_empty());
        Self {
            _pin: core::marker::PhantomPinned,
            fBlitter: blitter,
            fClipRect: clip,
        }
    }
}
// Source: src/core/SkBlitter.cpp:270-282
unsafe fn blitMaskRegion<B: SkBlitter + ?Sized>(
    mut blitter: core::pin::Pin<&mut B>,
    mask: &crate::src::core::SkMask::SkMask,
    clip: &crate::src::core::SkRegion::SkRegion,
) {
    if clip.quick_reject_rect(&mask.fBounds) {
        return;
    }
    let mut clipper = crate::src::core::SkRegion::Cliperator::new(clip, mask.fBounds);
    while !clipper.done() {
        let cr = clipper.rect();
        blitter.as_mut().blitMask(mask, cr);
        clipper.next();
    }
}
// Source: src/core/SkBlitter.cpp:284-292
unsafe fn blitRectRegion<B: SkBlitter + ?Sized>(
    mut blitter: core::pin::Pin<&mut B>,
    rect: &crate::src::core::SkRect::SkIRect,
    clip: &crate::src::core::SkRegion::SkRegion,
) {
    let mut clipper = crate::src::core::SkRegion::Cliperator::new(clip, *rect);
    while !clipper.done() {
        let cr = clipper.rect();
        blitter
            .as_mut()
            .blitRect(cr.fLeft, cr.fTop, cr.width(), cr.height());
        clipper.next();
    }
}
// Source: src/core/SkBlitter.cpp:294-298
unsafe fn blitRegion<B: SkBlitter + ?Sized>(
    mut blitter: core::pin::Pin<&mut B>,
    clip: &crate::src::core::SkRegion::SkRegion,
) {
    crate::src::core::SkRegionPriv::SkRegionPriv::VisitSpans(clip, |r| {
        blitter
            .as_mut()
            .blitRect(r.fLeft, r.fTop, r.width(), r.height());
    });
}
// Source: src/core/SkBlitter.cpp:302-316
pub fn compute_anti_width(runs: &[i16]) -> i32 {
    let mut width = 0;
    loop {
        let count = runs[width as usize] as i32;
        crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 308; @condition b"count >= 0\0".as_ptr(); count >= 0);
        if count == 0 {
            break;
        }
        width += count;
    }
    width
}
// Source: src/core/SkBlitter.cpp:318-320
fn y_in_rect(y: i32, rect: &crate::src::core::SkRect::SkIRect) -> bool {
    (y.wrapping_sub(rect.fTop) as u32) < (rect.fBottom.wrapping_sub(rect.fTop) as u32)
}
// Source: src/core/SkBlitter.cpp:322-324
fn x_in_rect(x: i32, rect: &crate::src::core::SkRect::SkIRect) -> bool {
    (x.wrapping_sub(rect.fLeft) as u32) < (rect.fRight.wrapping_sub(rect.fLeft) as u32)
}
unsafe impl SkBlitter for SkRectClipBlitter<'_> {
    // Source: src/core/SkBlitter.cpp:326-346
    unsafe fn blitH(self: core::pin::Pin<&mut Self>, mut left: i32, y: i32, mut width: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 327; @condition b"width > 0\0".as_ptr(); width > 0);
        if !y_in_rect(y, &this.fClipRect) {
            return;
        }
        let mut right = left + width;
        if left < this.fClipRect.fLeft {
            left = this.fClipRect.fLeft;
        }
        if right > this.fClipRect.fRight {
            right = this.fClipRect.fRight;
        }
        width = right - left;
        if width > 0 {
            this.fBlitter.as_mut().blitH(left, y, width);
        }
    }
    // Source: src/core/SkBlitter.cpp:348-384
    unsafe fn blitAntiH(
        self: core::pin::Pin<&mut Self>,
        left: i32,
        y: i32,
        aa: &mut [u8],
        runs: &mut [i16],
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        use crate::src::core::SkAlphaRuns::SkAlphaRuns;
        if !y_in_rect(y, &this.fClipRect) || left >= this.fClipRect.fRight {
            return;
        }
        let mut x0 = left;
        let mut x1 = left + compute_anti_width(runs);
        if x1 <= this.fClipRect.fLeft {
            return;
        }
        crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 364; @condition b"x0 < x1\0".as_ptr(); x0 < x1);
        let mut base = 0;
        if x0 < this.fClipRect.fLeft {
            let dx = (this.fClipRect.fLeft - x0) as usize;
            SkAlphaRuns::BreakAt(runs, aa, dx);
            base += dx;
            x0 = this.fClipRect.fLeft;
        }
        let runs = &mut runs[base..];
        let aa = &mut aa[base..];
        crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 373; @condition b"x0 < x1 && runs[x1 - x0] == 0\0".as_ptr(); x0 < x1 && runs[(x1 - x0) as usize] == 0);
        if x1 > this.fClipRect.fRight {
            x1 = this.fClipRect.fRight;
            SkAlphaRuns::BreakAt(runs, aa, (x1 - x0) as usize);
            runs[(x1 - x0) as usize] = 0;
        }
        crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 380; @condition b"x0 < x1 && runs[x1 - x0] == 0\0".as_ptr(); x0 < x1 && runs[(x1 - x0) as usize] == 0);
        crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 381; @condition b"compute_anti_width(runs) == x1 - x0\0".as_ptr(); (compute_anti_width(runs)) == (x1 - x0));
        this.fBlitter.as_mut().blitAntiH(x0, y, aa, runs);
    }
    // Source: src/core/SkBlitter.cpp:386-406
    unsafe fn blitV(self: core::pin::Pin<&mut Self>, x: i32, y: i32, height: i32, alpha: u8) {
        let this = unsafe { self.get_unchecked_mut() };
        crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 387; @condition b"height > 0\0".as_ptr(); height > 0);
        if !x_in_rect(x, &this.fClipRect) {
            return;
        }
        let mut y0 = y;
        let mut y1 = y + height;
        if y0 < this.fClipRect.fTop {
            y0 = this.fClipRect.fTop;
        }
        if y1 > this.fClipRect.fBottom {
            y1 = this.fClipRect.fBottom;
        }
        if y0 < y1 {
            this.fBlitter.as_mut().blitV(x, y0, y1 - y0, alpha);
        }
    }
    // Source: src/core/SkBlitter.cpp:408-415
    unsafe fn blitRect(
        self: core::pin::Pin<&mut Self>,
        left: i32,
        y: i32,
        width: i32,
        height: i32,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        let r = crate::src::core::SkRect::SkIRect {
            fLeft: left.max(this.fClipRect.fLeft),
            fTop: y.max(this.fClipRect.fTop),
            fRight: (left + width).min(this.fClipRect.fRight),
            fBottom: (y + height).min(this.fClipRect.fBottom),
        };
        if !r.is_empty() {
            this.fBlitter
                .as_mut()
                .blitRect(r.fLeft, r.fTop, r.width(), r.height());
        }
    }
    // Source: src/core/SkBlitter.cpp:417-446
    unsafe fn blitAntiRect(
        self: core::pin::Pin<&mut Self>,
        left: i32,
        y: i32,
        width: i32,
        height: i32,
        mut left_alpha: u8,
        mut right_alpha: u8,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        let r = crate::src::core::SkRect::SkIRect {
            fLeft: left.max(this.fClipRect.fLeft),
            fTop: y.max(this.fClipRect.fTop),
            fRight: (left + width + 2).min(this.fClipRect.fRight),
            fBottom: (y + height).min(this.fClipRect.fBottom),
        };
        if !r.is_empty() {
            if r.fLeft != left {
                crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 425; @condition b"r.fLeft > left\0".as_ptr(); r.fLeft > left);
                left_alpha = 255;
            }
            if r.fRight != left + width + 2 {
                crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 429; @condition b"r.fRight < left + width + 2\0".as_ptr(); r.fRight < left + width + 2);
                right_alpha = 255;
            }
            if left_alpha == 255 && right_alpha == 255 {
                this.fBlitter
                    .as_mut()
                    .blitRect(r.fLeft, r.fTop, r.width(), r.height());
            } else if r.width() == 1 {
                if r.fLeft == left {
                    this.fBlitter
                        .as_mut()
                        .blitV(r.fLeft, r.fTop, r.height(), left_alpha);
                } else {
                    crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 438; @condition b"r.fLeft == left + width + 1\0".as_ptr(); (r.fLeft) == (left + width + 1));
                    this.fBlitter
                        .as_mut()
                        .blitV(r.fLeft, r.fTop, r.height(), right_alpha);
                }
            } else {
                this.fBlitter.as_mut().blitAntiRect(
                    r.fLeft,
                    r.fTop,
                    r.width() - 2,
                    r.height(),
                    left_alpha,
                    right_alpha,
                );
            }
        }
    }
    // Source: src/core/SkBlitter.h:203-205
    fn requestRowsPreserved(&self) -> i32 {
        self.fBlitter.requestRowsPreserved()
    }
    // Source: src/core/SkBlitter.h:207-209
    fn allocBlitMemory(self: core::pin::Pin<&mut Self>, sz: usize) -> *mut u8 {
        let this = unsafe { self.get_unchecked_mut() };
        this.fBlitter.as_mut().allocBlitMemory(sz)
    }

    // Source: src/core/SkBlitter.cpp:448-456
    unsafe fn blitMask(
        self: core::pin::Pin<&mut Self>,
        mask: &crate::src::core::SkMask::SkMask,
        clip: &crate::src::core::SkRect::SkIRect,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 449; @condition b"mask.fBounds.contains(clip)\0".as_ptr(); mask.fBounds.contains_rect(clip));
        let mut r = *clip;
        if r.intersect(&this.fClipRect) {
            this.fBlitter.as_mut().blitMask(mask, &r);
        }
    }
}
// Source: src/core/SkBlitter.h:220-226
pub struct SkRgnClipBlitter<'a> {
    _pin: core::marker::PhantomPinned,
    fBlitter: core::pin::Pin<&'a mut dyn SkBlitter>,
    fRgn: &'a crate::src::core::SkRegion::SkRegion,
}
impl<'a> SkRgnClipBlitter<'a> {
    pub fn new(
        blitter: core::pin::Pin<&'a mut dyn SkBlitter>,
        region: &'a crate::src::core::SkRegion::SkRegion,
    ) -> Self {
        crate::SkASSERT!(@source b"src/core/SkBlitter.h\0".as_ptr(), 223; @condition b"clipRgn && !clipRgn->isEmpty()\0".as_ptr(); !region.is_empty());
        Self {
            _pin: core::marker::PhantomPinned,
            fBlitter: blitter,
            fRgn: region,
        }
    }
}
unsafe impl SkBlitter for SkRgnClipBlitter<'_> {
    // Source: src/core/SkBlitter.h:236-238
    fn requestRowsPreserved(&self) -> i32 {
        self.fBlitter.requestRowsPreserved()
    }
    // Source: src/core/SkBlitter.h:240-242
    fn allocBlitMemory(self: core::pin::Pin<&mut Self>, sz: usize) -> *mut u8 {
        let this = unsafe { self.get_unchecked_mut() };
        this.fBlitter.as_mut().allocBlitMemory(sz)
    }
    // Source: src/core/SkBlitter.cpp:460-468
    unsafe fn blitH(self: core::pin::Pin<&mut Self>, x: i32, y: i32, width: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        let mut span = crate::src::core::SkRegion::Spanerator::new(this.fRgn, y, x, x + width);
        let (mut left, mut right) = (0, 0);
        while span.next(Some(&mut left), Some(&mut right)) {
            crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 465; @condition b"left < right\0".as_ptr(); left < right);
            this.fBlitter.as_mut().blitH(left, y, right - left);
        }
    }
    // Source: src/core/SkBlitter.cpp:470-510
    unsafe fn blitAntiH(
        self: core::pin::Pin<&mut Self>,
        mut x: i32,
        y: i32,
        aa: &mut [u8],
        runs: &mut [i16],
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        use crate::src::core::SkAlphaRuns::SkAlphaRuns;
        let width = compute_anti_width(runs);
        let mut span = crate::src::core::SkRegion::Spanerator::new(this.fRgn, y, x, x + width);
        let (mut left, mut right) = (0, 0);
        #[cfg(sk_debug)]
        let bounds = this.fRgn.get_bounds();
        let mut prev_right = x;
        while span.next(Some(&mut left), Some(&mut right)) {
            {
                crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 482; @condition b"x <= left\0".as_ptr(); x <= left);
                crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 483; @condition b"left < right\0".as_ptr(); left < right);
                crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 484; @condition b"left >= bounds.fLeft && right <= bounds.fRight\0".as_ptr(); left >= bounds.fLeft && right <= bounds.fRight);
            };
            SkAlphaRuns::Break(runs, aa, (left - x) as usize, (right - left) as usize);
            if left > prev_right {
                let index = (prev_right - x) as usize;
                aa[index] = 0;
                runs[index] = (left - prev_right) as i16;
            }
            prev_right = right;
        }
        if prev_right > x {
            runs[(prev_right - x) as usize] = 0;
            let mut base = 0;
            if x < 0 {
                let skip = runs[0] as i32;
                crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 503; @condition b"skip >= -x\0".as_ptr(); skip >= -x);
                base = skip as usize;
                x += skip;
            }
            this.fBlitter
                .as_mut()
                .blitAntiH(x, y, &mut aa[base..], &mut runs[base..]);
        }
    }
    // Source: src/core/SkBlitter.cpp:512-525
    unsafe fn blitV(self: core::pin::Pin<&mut Self>, x: i32, y: i32, height: i32, alpha: u8) {
        let this = unsafe { self.get_unchecked_mut() };
        let bounds = crate::src::core::SkRect::SkIRect::make_xywh(x, y, 1, height);
        let mut iter = crate::src::core::SkRegion::Cliperator::new(this.fRgn, bounds);
        while !iter.done() {
            let r = iter.rect();
            crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 520; @condition b"bounds.contains(r)\0".as_ptr(); bounds.contains_rect(r));
            this.fBlitter.as_mut().blitV(x, r.fTop, r.height(), alpha);
            iter.next();
        }
    }
    // Source: src/core/SkBlitter.cpp:527-540
    unsafe fn blitRect(self: core::pin::Pin<&mut Self>, x: i32, y: i32, width: i32, height: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        let bounds = crate::src::core::SkRect::SkIRect::make_xywh(x, y, width, height);
        let mut iter = crate::src::core::SkRegion::Cliperator::new(this.fRgn, bounds);
        while !iter.done() {
            let r = iter.rect();
            crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 535; @condition b"bounds.contains(r)\0".as_ptr(); bounds.contains_rect(r));
            this.fBlitter
                .as_mut()
                .blitRect(r.fLeft, r.fTop, r.width(), r.height());
            iter.next();
        }
    }
    // Source: src/core/SkBlitter.cpp:542-577
    unsafe fn blitAntiRect(
        self: core::pin::Pin<&mut Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        left_alpha: u8,
        right_alpha: u8,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        let bounds = crate::src::core::SkRect::SkIRect::make_xywh(x, y, width + 2, height);
        let mut iter = crate::src::core::SkRegion::Cliperator::new(this.fRgn, bounds);
        while !iter.done() {
            let r = iter.rect();
            crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 552; @condition b"bounds.contains(r)\0".as_ptr(); bounds.contains_rect(r));
            {
                crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 553; @condition b"r.fLeft >= x\0".as_ptr(); r.fLeft >= x);
                crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 554; @condition b"r.fRight <= x + width + 2\0".as_ptr(); r.fRight <= x + width + 2);
            };
            let effective_left_alpha = if r.fLeft == x { left_alpha } else { 255 };
            let effective_right_alpha = if r.fRight == x + width + 2 {
                right_alpha
            } else {
                255
            };
            if effective_left_alpha == 255 && effective_right_alpha == 255 {
                this.fBlitter
                    .as_mut()
                    .blitRect(r.fLeft, r.fTop, r.width(), r.height());
            } else if r.width() == 1 {
                if r.fLeft == x {
                    this.fBlitter
                        .as_mut()
                        .blitV(r.fLeft, r.fTop, r.height(), effective_left_alpha);
                } else {
                    crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 567; @condition b"r.fLeft == x + width + 1\0".as_ptr(); (r.fLeft) == (x + width + 1));
                    this.fBlitter.as_mut().blitV(
                        r.fLeft,
                        r.fTop,
                        r.height(),
                        effective_right_alpha,
                    );
                }
            } else {
                this.fBlitter.as_mut().blitAntiRect(
                    r.fLeft,
                    r.fTop,
                    r.width() - 2,
                    r.height(),
                    effective_left_alpha,
                    effective_right_alpha,
                );
            }
            iter.next();
        }
    }

    // Source: src/core/SkBlitter.cpp:580-591
    unsafe fn blitMask(
        self: core::pin::Pin<&mut Self>,
        mask: &crate::src::core::SkMask::SkMask,
        clip: &crate::src::core::SkRect::SkIRect,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 581; @condition b"mask.fBounds.contains(clip)\0".as_ptr(); mask.fBounds.contains_rect(clip));
        let mut iter = crate::src::core::SkRegion::Cliperator::new(this.fRgn, *clip);
        while !iter.done() {
            this.fBlitter.as_mut().blitMask(mask, iter.rect());
            iter.next();
        }
    }
}

// Source: src/core/SkBlitter.h:125-128
pub struct DirectBlit {
    pub pm: crate::src::core::SkPixmap::SkPixmap,
    pub value: u64,
}
// Source: src/core/SkBlitter.h:174-181
pub struct SkNullBlitter {
    storage: SkBlitterStorage,
}
impl SkNullBlitter {
    pub fn new() -> Self {
        Self {
            storage: SkBlitterStorage::new(),
        }
    }
}
// The source class intentionally discards draws. Its inherited base allocation
// is still real owned storage, not an unimplemented execution fallback.
unsafe impl SkBlitter for SkNullBlitter {
    fn allocBlitMemory(self: core::pin::Pin<&mut Self>, sz: usize) -> *mut u8 {
        let this = unsafe { self.get_unchecked_mut() };
        this.storage.allocBlitMemory(sz)
    }
    unsafe fn blitH(self: core::pin::Pin<&mut Self>, _x: i32, _y: i32, _width: i32) {
        let this = unsafe { self.get_unchecked_mut() };
    }
    unsafe fn blitAntiH(
        self: core::pin::Pin<&mut Self>,
        _x: i32,
        _y: i32,
        _aa: &mut [u8],
        _runs: &mut [i16],
    ) {
        let this = unsafe { self.get_unchecked_mut() };
    }
    unsafe fn blitV(self: core::pin::Pin<&mut Self>, _x: i32, _y: i32, _height: i32, _alpha: u8) {
        let this = unsafe { self.get_unchecked_mut() };
    }
    unsafe fn blitRect(
        self: core::pin::Pin<&mut Self>,
        _x: i32,
        _y: i32,
        _width: i32,
        _height: i32,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
    }
    unsafe fn blitMask(
        self: core::pin::Pin<&mut Self>,
        _mask: &crate::src::core::SkMask::SkMask,
        _clip: &crate::src::core::SkRect::SkIRect,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
    }
}
// Source: src/core/SkBlitter.h:287-296
// Adapted owner: a tagged wrapper retains the selected original member and its
// borrows, replacing the native pointer into a self-referential clipper object.
pub enum SkBlitterClipper<'a> {
    Direct(core::pin::Pin<&'a mut dyn SkBlitter>),
    Null(SkNullBlitter),
    Rect(SkRectClipBlitter<'a>),
    Region(SkRgnClipBlitter<'a>),
}
impl<'a> SkBlitterClipper<'a> {
    // Source: src/core/SkBlitter.cpp:595-613
    pub fn apply(
        mut blitter: core::pin::Pin<&'a mut dyn SkBlitter>,
        clip: Option<&'a crate::src::core::SkRegion::SkRegion>,
        ir: Option<&crate::src::core::SkRect::SkIRect>,
    ) -> Self {
        if let Some(clip) = clip {
            let clip_r = clip.get_bounds();
            if clip.is_empty()
                || ir.is_some_and(|ir| !crate::src::core::SkRect::SkIRect::intersects(clip_r, ir))
            {
                return Self::Null(SkNullBlitter::new());
            } else if clip.is_rect() {
                if ir.is_none_or(|ir| !clip_r.contains_rect(ir)) {
                    return Self::Rect(SkRectClipBlitter::new(blitter, *clip_r));
                }
            } else {
                return Self::Region(SkRgnClipBlitter::new(blitter, clip));
            }
        }
        Self::Direct(blitter)
    }
    pub fn blitter(self: core::pin::Pin<&mut Self>) -> core::pin::Pin<&mut (dyn SkBlitter + '_)> {
        // Pin projects the active enum member in place; no variant is replaced.
        unsafe {
            match self.get_unchecked_mut() {
                Self::Direct(b) => b.as_mut(),
                Self::Null(b) => core::pin::Pin::new_unchecked(b),
                Self::Rect(b) => core::pin::Pin::new_unchecked(b),
                Self::Region(b) => core::pin::Pin::new_unchecked(b),
            }
        }
    }
}

impl dyn SkBlitter {
    // Source: src/core/SkBlitter.cpp:617-652
    // Chromium skia_config enables the original legacy shader-context branch;
    // SK_FORCE_RASTER_PIPELINE_BLITTER is not selected by this CPU configuration.
    pub fn UseLegacyBlitter(
        device: &crate::src::core::SkPixmap::SkPixmap,
        paint: &crate::src::core::SkPaint::SkPaint,
        _matrix: &crate::src::core::SkMatrix::SkMatrix,
    ) -> bool {
        if gSkForceRasterPipelineBlitter.load(core::sync::atomic::Ordering::Relaxed) {
            return false;
        }
        if paint.isDither() {
            return false;
        }
        let mf = paint.getMaskFilter();
        if device.alphaType()
            == crate::include::core::SkAlphaType::SkAlphaType::kUnpremul_SkAlphaType
            || !paint.isSrcOver()
            || mf.is_some_and(|mf| mf.getFormat() == crate::src::core::SkMask::Format::k3D_Format)
        {
            return false;
        }
        let cs = device.colorSpace();
        if let Some(cs) = cs.filter(|_| paint.getShader().is_none()) {
            if !paint.getColor4f().fitsInBytes() || !cs.isSRGB() {
                return false;
            }
        }
        device.colorType() == crate::include::core::SkColorType::kN32_SkColorType
    }
}

// Source: src/core/SkBlitter.cpp:654-757
/// Safety: alloc owns every returned blitter/context until its reset/destruction.
/// Destination accesses are live, exclusive and bounded on every primitive call.
/// Keep all retained shader/context/arena backing valid through context teardown;
/// no concurrent context mutation, drawing, or arena reset is permitted.
/// In particular, image-shader source pixels and all coordinate/context storage
/// must be disjoint from destination spans for every shade/blit call. Retaining
/// a shader Arc proves owner lifetime, not source/destination non-aliasing; the
/// caller must establish any source-image COW or independent pixel lease.
pub unsafe fn Choose<'arena>(
    device: &crate::src::core::SkPixmap::SkPixmap,
    ctm: &crate::src::core::SkMatrix::SkMatrix,
    origPaint: &crate::src::core::SkPaint::SkPaint,
    alloc: core::pin::Pin<&'arena mut crate::src::core::SkArenaAlloc::SkArenaAlloc>,
    drawCoverage: crate::src::core::SkDrawTypes::SkDrawCoverage,
    clipShader: Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>>,
    props: &crate::include::core::SkSurfaceProps::SkSurfaceProps,
    devBounds: &crate::src::core::SkRect::SkRect,
) -> Result<Option<core::ptr::NonNull<dyn SkBlitter + 'arena>>, crate::RasterError> {
    use crate::include::core::SkColorType::SkColorType::*;
    use crate::src::core::SkBlendMode::SkBlendMode;
    use crate::src::core::SkBlendModePriv::{CheckFastPath, SkBlendFastPath};
    // Consume the pinned publication lease into a raw native pointer. Every
    // actual allocator call reborrows synchronously; no &mut arena crosses shader
    // callbacks. The returned blitter's arena lifetime preserves the pin lease.
    let ap = alloc.get_unchecked_mut() as *mut crate::src::core::SkArenaAlloc::SkArenaAlloc;
    // Adapted allocation projection shared by the native Null allocation sites.
    // Safety: the caller keeps this exact arena alive, stable and unreset through
    // the returned borrow. Null's owned scratch storage is destroyed by its arena
    // footer; the trait pointer transfers no ownership and must never be freed.
    unsafe fn make_null_blitter<'arena>(
        arena: *mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
    ) -> Option<core::ptr::NonNull<dyn SkBlitter + 'arena>> {
        let pinned = core::pin::Pin::new_unchecked(&mut *arena).make_pinned(SkNullBlitter::new());
        let ptr = core::pin::Pin::get_unchecked_mut(pinned) as *mut SkNullBlitter;
        Some(core::ptr::NonNull::new_unchecked(
            ptr as *mut (dyn SkBlitter + 'arena),
        ))
    }
    if device.colorType() == kUnknown_SkColorType {
        return Ok(make_null_blitter(ap));
    }
    // Native copy-on-first-write paint; clone is performed only by to_mut().
    let mut paint = std::borrow::Cow::Borrowed(origPaint);
    if let Some(mode) = paint.asBlendMode() {
        if mode != SkBlendMode::kSrcOver {
            match CheckFastPath(
                &paint,
                crate::src::core::SkImageInfo::SkColorTypeIsAlwaysOpaque(device.colorType()),
            ) {
                SkBlendFastPath::kSrcOver => paint.to_mut().setBlendMode(SkBlendMode::kSrcOver),
                SkBlendFastPath::kSkipDrawing => return Ok(make_null_blitter(ap)),
                _ => {}
            }
        }
        if mode == SkBlendMode::kClear {
            let p = paint.to_mut();
            p.setShader(None);
            p.setColorFilter(None);
            p.setBlendMode(SkBlendMode::kSrc);
            p.setColor(0);
        }
    }
    if paint.getColorFilter().is_some() {
        let dst_cs_owner = device.refColorSpace();
        crate::src::core::SkPaintPriv::RemoveColorFilter(paint.to_mut(), dst_cs_owner.as_ref())?;
    }
    crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 698; @condition b"!paint->getColorFilter()\0".as_ptr(); paint.getColorFilter().is_none());
    if drawCoverage == crate::src::core::SkDrawTypes::SkDrawCoverage::kYes {
        if device.colorType() == kAlpha_8_SkColorType {
            {
                crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 702; @condition b"!paint->getShader()\0".as_ptr(); paint.getShader().is_none());
                crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 703; @condition b"paint->isSrcOver()\0".as_ptr(); paint.isSrcOver());
            };
            let ptr = (*ap).make(crate::src::core::SkBlitter_A8::SkA8_Coverage_Blitter::new(
                device, &paint,
            ));
            return Ok(Some(core::ptr::NonNull::new_unchecked(
                ptr as *mut (dyn SkBlitter + 'arena),
            )));
        }
        return Ok(make_null_blitter(ap));
    }
    if paint.isDither() && !crate::src::core::SkPaintPriv::ShouldDither(&paint, device.colorType())
    {
        paint.to_mut().setDither(false);
    }
    let create_rp =
        || -> Result<Option<core::ptr::NonNull<dyn SkBlitter + 'arena>>, crate::RasterError> {
            let result = crate::src::core::SkCoreBlitters::SkCreateRasterPipelineBlitter(
                device,
                &paint,
                ctm,
                &mut *ap,
                clipShader.clone(),
                props,
                devBounds,
            )?;
            Ok(if result.is_some() {
                result
            } else {
                make_null_blitter(ap)
            })
        };
    if clipShader.is_some() || !<dyn SkBlitter>::UseLegacyBlitter(device, &paint, ctm) {
        return create_rp();
    }
    crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 726; @condition b"device.colorType() == kN32_SkColorType\0".as_ptr(); device.colorType()==crate::include::core::SkColorType::kN32_SkColorType);
    crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 729; @condition b"paint->asBlendMode() == SkBlendMode::kSrcOver\0".as_ptr(); paint.asBlendMode()==Some(SkBlendMode::kSrcOver));
    let mut shader_context = None;
    if let Some(shader) = paint.refShader() {
        // Adapted Rust lifetime proof: retain this exact Arc in the arena before its
        // borrowing context is allocated. LIFO destruction drops blitter, then
        // context, then this retained Arc, including temporary RemoveColorFilter shaders.
        // This is actual owner storage, not a forged static or an extra context owner.
        let retained = (*ap).make(shader);
        let shader: &'arena std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader> =
            &*retained;
        // Source: src/core/SkBlitter.cpp:734-740
        // Borrow the exact destination ImageInfo owner token for this synchronous
        // context-construction call; no retained clone or extended lifetime is needed.
        let rec = crate::src::shaders::SkShaderBase::ContextRec::new(
            paint.getAlpha(),
            &crate::src::shaders::SkShaderBase::MatrixRec::new(ctm),
            device.colorType(),
            device.info().colorSpaceOwner(),
            props,
        );
        shader_context = shader.makeContext(&rec, ap)?;
        if shader_context.is_none() {
            return create_rp();
        }
    }
    if let Some(context) = shader_context {
        // Construct before installing the arena destructor footer: safe virtual
        // getFlags can unwind, and no footer may observe an uninitialized blitter.
        // This value has no self-pointer, so the subsequent constructor is a pure move.
        let blitter =
            crate::src::core::SkCoreBlitters::SkARGB32_Shader_Blitter::new(device, &paint, context);
        let ptr = (*ap).make_with(|_, _| blitter);
        Ok(Some(core::ptr::NonNull::new_unchecked(
            ptr as *mut (dyn SkBlitter + 'arena),
        )))
    } else if paint.getColor() == crate::src::core::SkColor::SK_ColorBLACK {
        let ptr = (*ap)
            .make(crate::src::core::SkCoreBlitters::SkARGB32_Black_Blitter::new(device, &paint));
        Ok(Some(core::ptr::NonNull::new_unchecked(
            ptr as *mut (dyn SkBlitter + 'arena),
        )))
    } else if paint.getAlpha() == 255 {
        let ptr = (*ap)
            .make(crate::src::core::SkCoreBlitters::SkARGB32_Opaque_Blitter::new(device, &paint));
        Ok(Some(core::ptr::NonNull::new_unchecked(
            ptr as *mut (dyn SkBlitter + 'arena),
        )))
    } else {
        let ptr = (*ap).make(crate::src::core::SkCoreBlitters::SkARGB32_Blitter::new(
            device, &paint,
        ));
        Ok(Some(core::ptr::NonNull::new_unchecked(
            ptr as *mut (dyn SkBlitter + 'arena),
        )))
    }
}

impl<'shader> crate::src::core::SkCoreBlitters::SkShaderBlitter<'shader> {
    // Source: src/core/SkBlitter.cpp:761-769
    /// Safety: shaderContext is a live exclusive arena-owned context, borrowing
    /// the actual retained shader's stable backing. Its storage and shader borrow
    /// must remain valid through context destruction, which is owned by the arena.
    /// Pixel backing is live/exclusive for each primitive call. This base acquires
    /// no ownership of shaderContext and never destroys or extends its lifetime.
    pub unsafe fn new(
        device: &crate::src::core::SkPixmap::SkPixmap,
        paint: &crate::src::core::SkPaint::SkPaint,
        shaderContext: *mut (dyn crate::src::shaders::SkShaderBase::Context + 'shader),
    ) -> Self {
        let raster = crate::src::core::SkCoreBlitters::SkRasterBlitter::new(device);
        let shader = paint.refShader();
        crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 765; @condition b"fShader\0".as_ptr(); shader.is_some());
        crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 766; @condition b"fShaderContext\0".as_ptr(); !shaderContext.is_null());
        let context = unsafe { core::ptr::NonNull::new_unchecked(shaderContext) };
        Self {
            fShader: shader,
            fShaderContext: context,
            raster,
        }
    }
    // Native default destructor: retained shader/raster fields drop normally.
    // Context remains nonowning: no extra drop_in_place, free or lifetime erasure.
}

// Source: src/core/SkBlitter.h:250-280
#[cfg(sk_debug)]
pub struct SkRectClipCheckBlitter<'a> {
    _pin: core::marker::PhantomPinned,
    fBlitter: core::pin::Pin<&'a mut dyn SkBlitter>,
    fClipRect: crate::src::core::SkRect::SkIRect,
}
#[cfg(sk_debug)]
impl<'a> SkRectClipCheckBlitter<'a> {
    pub fn new(
        blitter: core::pin::Pin<&'a mut dyn SkBlitter>,
        clipRect: crate::src::core::SkRect::SkIRect,
    ) -> Self {
        debug_assert!(!clipRect.is_empty());
        Self {
            _pin: core::marker::PhantomPinned,
            fBlitter: blitter,
            fClipRect: clipRect,
        }
    }
}
#[cfg(sk_debug)]
// SAFETY: the borrowed wrapped blitter retains allocation/row/pixel ownership;
// every primitive preserves its caller's lease and forwards the original span.
unsafe impl SkBlitter for SkRectClipCheckBlitter<'_> {
    fn requestRowsPreserved(&self) -> i32 {
        self.fBlitter.requestRowsPreserved()
    }
    fn allocBlitMemory(self: core::pin::Pin<&mut Self>, sz: usize) -> *mut u8 {
        let this = unsafe { self.get_unchecked_mut() };
        this.fBlitter.as_mut().allocBlitMemory(sz)
    }
    // Source: src/core/SkBlitter.cpp:775-778
    unsafe fn blitH(self: core::pin::Pin<&mut Self>, x: i32, y: i32, width: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 776; @condition b"fClipRect.contains(SkIRect::MakeXYWH(x, y, width, 1))\0".as_ptr(); this.fClipRect.contains_rect(&crate::src::core::SkRect::SkIRect::make_xywh(x,y,width,1)));
        this.fBlitter.as_mut().blitH(x, y, width);
    }
    // Source: src/core/SkBlitter.cpp:780-787
    unsafe fn blitAntiH(
        self: core::pin::Pin<&mut Self>,
        x: i32,
        y: i32,
        aa: &mut [u8],
        runs: &mut [i16],
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        let mut width = 0usize;
        while runs[width] != 0 {
            width += runs[width] as usize;
        }
        crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 785; @condition b"fClipRect.contains(SkIRect::MakeXYWH(x, y, width, 1))\0".as_ptr(); this.fClipRect.contains_rect(&crate::src::core::SkRect::SkIRect::make_xywh(x,y,width as i32,1)));
        this.fBlitter.as_mut().blitAntiH(x, y, aa, runs);
    }
    // Source: src/core/SkBlitter.cpp:789-792
    unsafe fn blitV(self: core::pin::Pin<&mut Self>, x: i32, y: i32, height: i32, alpha: u8) {
        let this = unsafe { self.get_unchecked_mut() };
        crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 790; @condition b"fClipRect.contains(SkIRect::MakeXYWH(x, y, 1, height))\0".as_ptr(); this.fClipRect.contains_rect(&crate::src::core::SkRect::SkIRect::make_xywh(x,y,1,height)));
        this.fBlitter.as_mut().blitV(x, y, height, alpha);
    }
    // Source: src/core/SkBlitter.cpp:794-797
    unsafe fn blitRect(self: core::pin::Pin<&mut Self>, x: i32, y: i32, width: i32, height: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 795; @condition b"fClipRect.contains(SkIRect::MakeXYWH(x, y, width, height))\0".as_ptr(); this.fClipRect.contains_rect(&crate::src::core::SkRect::SkIRect::make_xywh(x,y,width,height)));
        this.fBlitter.as_mut().blitRect(x, y, width, height);
    }
    // Source: src/core/SkBlitter.cpp:799-806
    unsafe fn blitAntiRect(
        self: core::pin::Pin<&mut Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        left: u8,
        right: u8,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        let skip_left = (left == 0) as i32;
        let skip_right = (right == 0) as i32;
        let rect = crate::src::core::SkRect::SkIRect::make_xywh(
            x + skip_left,
            y,
            width + 2 - skip_right - skip_left,
            height,
        );
        crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 804; @condition b"r.isEmpty() || fClipRect.contains(r)\0".as_ptr(); rect.is_empty() || this.fClipRect.contains_rect(&rect));
        this.fBlitter
            .as_mut()
            .blitAntiRect(x, y, width, height, left, right);
    }
    // Source: src/core/SkBlitter.cpp:808-812
    unsafe fn blitMask(
        self: core::pin::Pin<&mut Self>,
        mask: &crate::src::core::SkMask::SkMask,
        clip: &crate::src::core::SkRect::SkIRect,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 809; @condition b"mask.fBounds.contains(clip)\0".as_ptr(); mask.fBounds.contains_rect(clip));
        crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 810; @condition b"fClipRect.contains(clip)\0".as_ptr(); this.fClipRect.contains_rect(clip));
        this.fBlitter.as_mut().blitMask(mask, clip);
    }
    // Source: src/core/SkBlitter.cpp:814-817
    unsafe fn blitAntiH2(self: core::pin::Pin<&mut Self>, x: i32, y: i32, a0: u32, a1: u32) {
        let this = unsafe { self.get_unchecked_mut() };
        crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 815; @condition b"fClipRect.contains(SkIRect::MakeXYWH(x, y, 2, 1))\0".as_ptr(); this.fClipRect.contains_rect(&crate::src::core::SkRect::SkIRect::make_xywh(x,y,2,1)));
        this.fBlitter.as_mut().blitAntiH2(x, y, a0, a1);
    }
    // Source: src/core/SkBlitter.cpp:819-822
    unsafe fn blitAntiV2(self: core::pin::Pin<&mut Self>, x: i32, y: i32, a0: u32, a1: u32) {
        let this = unsafe { self.get_unchecked_mut() };
        crate::SkASSERT!(@source b"src/core/SkBlitter.cpp\0".as_ptr(), 820; @condition b"fClipRect.contains(SkIRect::MakeXYWH(x, y, 1, 2))\0".as_ptr(); this.fClipRect.contains_rect(&crate::src::core::SkRect::SkIRect::make_xywh(x,y,1,2)));
        this.fBlitter.as_mut().blitAntiV2(x, y, a0, a1);
    }
}
