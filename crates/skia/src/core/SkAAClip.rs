use crate::src::core::SkBlitter::SkBlitter;
// Pin adaptation: concrete blitters inherit !Unpin from their actual embedded
// SkBlitterStorage. Scan entrypoints receive the original pinned destination;
// local polymorphic owners are pinned before dispatch and never moved afterward.
// get_unchecked_mut only projects existing fields; it never extracts/replaces
// pinned storage or transfers an installed arena/inline allocation.
use crate::src::core::SkRect::SkIRect;
use crate::src::core::SkRegion::{RegionIterator, SkRegion};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicI32, Ordering};

// Source: src/core/SkAAClip.h:33-34
impl SkAAClip {
    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }
    pub fn get_bounds(&self) -> &SkIRect {
        &self.bounds
    }
}

// Source: src/core/SkAAClip.h:57-59
impl SkAAClip {
    pub fn quick_contains_rect(&self, rect: &SkIRect) -> bool {
        self.quick_contains(rect.fLeft, rect.fTop, rect.fRight, rect.fBottom)
    }
}

// Source: src/core/SkAAClip.h:74-75
// The Rust owning pointer retains native single-tail allocation and atomic reference ownership.
#[derive(Debug)]
pub struct SkAAClip {
    bounds: SkIRect,
    head: Option<RunHead>,
}

// Source: src/core/SkAAClip.h:92-102
// Optional borrows and the init scratch-capacity guard are Rust lifetime adaptations.
impl<'a> SkAAClipBlitter<'a> {
    pub fn new() -> Self {
        Self {
            storage: crate::src::core::SkBlitter::SkBlitterStorage::new(),
            blitter: None,
            clip: None,
            bounds: SkIRect::make_empty(),
            runs: core::ptr::null_mut(),
            aa: core::ptr::null_mut(),
            scanline_scratch: core::ptr::null_mut(),
            gray_mask_scratch: crate::src::core::SkAutoMalloc::SkAutoSMalloc::new(),
        }
    }
    pub fn init(
        self: core::pin::Pin<&mut Self>,
        mut blitter: core::pin::Pin<&'a mut dyn crate::src::core::SkBlitter::SkBlitter>,
        clip: &'a SkAAClip,
    ) {
        unsafe { self.get_unchecked_mut() }.init_fields(blitter, clip);
    }
    // Called before wrapper publication or through its exclusive pinned lease.
    pub(crate) fn init_fields(
        &mut self,
        mut blitter: core::pin::Pin<&'a mut dyn crate::src::core::SkBlitter::SkBlitter>,
        clip: &'a SkAAClip,
    ) {
        crate::SkASSERT!(@source b"src/core/SkAAClip.h\0".as_ptr(), 98; @condition b"aaclip && !aaclip->isEmpty()\0".as_ptr(); !clip.is_empty());
        if !self.scanline_scratch.is_null() {
            // Rust safety guard: native init assumes the existing scratch remains large
            // enough. Reject a larger clip before publishing inconsistent cached views.
            let count = unsafe { self.aa.offset_from(self.scanline_scratch) as usize / 2 };
            assert!((clip.get_bounds().width() as usize) < count);
        }
        self.bounds = *clip.get_bounds();
        self.blitter = Some(blitter);
        self.clip = Some(clip);
    }
}

// Source: src/core/SkAAClip.h:111-123
// One native scanline block, runs/alpha pointer views and native inline gray scratch.
// Optional borrowed targets remain a Rust representation adaptation.
pub struct SkAAClipBlitter<'a> {
    // Rust composition retains the original SkBlitter base allocation lifetime.
    storage: crate::src::core::SkBlitter::SkBlitterStorage,
    blitter: Option<core::pin::Pin<&'a mut dyn crate::src::core::SkBlitter::SkBlitter>>,
    clip: Option<&'a SkAAClip>,
    bounds: SkIRect,
    runs: *mut i16,
    aa: *mut u8,
    scanline_scratch: *mut u8,
    gray_mask_scratch: crate::src::core::SkAutoMalloc::SkAutoSMalloc<1024>,
}
// Source: src/core/SkAAClip.cpp:32-49
struct AutoAAClipValidate(*const SkAAClip);
impl AutoAAClipValidate {
    // Safety: caller keeps the pointed clip at the same address until guard
    // drop. Clip methods hold &mut self for that entire interval.
    unsafe fn new(clip: *const SkAAClip) -> Self {
        unsafe {
            (*clip).validate();
        }
        Self(clip)
    }
}
impl Drop for AutoAAClipValidate {
    fn drop(&mut self) {
        unsafe {
            (*self.0).validate();
        }
    }
}

// Source: src/core/SkAAClip.cpp:53-69
const MAX_I32: i32 = i32::MAX;
#[cfg(sk_debug)]
fn compute_row_length(row: &[u8], mut width: i32) -> usize {
    let mut p = 0;
    while width > 0 {
        let n = i32::from(row[p]);
        {
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 61; @condition b"n > 0\0".as_ptr(); n > 0);
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 62; @condition b"n <= width\0".as_ptr(); n <= width);
        };
        p += 2;
        width -= n;
    }
    crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 66; @condition b"0 == width\0".as_ptr(); (width) == (0));
    p
}

// Source: src/core/SkAAClip.cpp:71-77
#[derive(Clone, Copy, Debug)]
struct YOffset {
    y: i32,
    offset: u32,
}

// Source: src/core/SkAAClip.cpp:79-124
struct RowIter<'a> {
    row: Option<&'a [u8]>,
    left: i32,
    right: i32,
    bounds_right: i32,
    done: bool,
    alpha: u8,
}
impl<'a> RowIter<'a> {
    fn new(row: Option<&'a [u8]>, bounds: &SkIRect) -> Self {
        if let Some(data) = row {
            let right = bounds.fLeft + i32::from(data[0]);
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 87; @condition b"fRight <= fBoundsRight\0".as_ptr(); right <= bounds.fRight);
            Self {
                row,
                left: bounds.fLeft,
                right,
                bounds_right: bounds.fRight,
                done: false,
                alpha: data[1],
            }
        } else {
            Self {
                row,
                left: bounds.fLeft,
                right: MAX_I32,
                bounds_right: bounds.fRight,
                done: true,
                alpha: 0,
            }
        }
    }
    fn next(&mut self) {
        if !self.done {
            self.left = self.right;
            if self.right == self.bounds_right {
                self.done = true;
                self.right = MAX_I32;
                self.alpha = 0;
            } else {
                let row = &self.row.unwrap()[2..];
                self.row = Some(row);
                self.right += i32::from(row[0]);
                self.alpha = row[1];
                crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 112; @condition b"fRight <= fBoundsRight\0".as_ptr(); self.right <= self.bounds_right);
            }
        }
    }
}

// Source: src/core/SkAAClip.cpp:126-170
struct Iter<'a> {
    offsets: &'a [YOffset],
    data: &'a [u8],
    index: usize,
    top: i32,
    bottom: i32,
    done: bool,
}
impl<'a> Iter<'a> {
    fn empty() -> Self {
        Self {
            offsets: &[],
            data: &[],
            index: 0,
            top: MAX_I32,
            bottom: MAX_I32,
            done: true,
        }
    }
    fn new(y: i32, data: &'a [u8], offsets: &'a [YOffset]) -> Self {
        Self {
            offsets,
            data,
            index: 0,
            top: y,
            bottom: y + offsets[0].y + 1,
            done: false,
        }
    }
    fn data(&self) -> Option<&'a [u8]> {
        if self.done {
            None
        } else {
            Some(&self.data[self.offsets[self.index].offset as usize..])
        }
    }
    fn next(&mut self) {
        if !self.done {
            let next = self.index + 1;
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 147; @condition b"curr <= fStopYOff\0".as_ptr(); next <= self.offsets.len());
            self.top = self.bottom;
            if next >= self.offsets.len() {
                self.done = true;
                self.bottom = MAX_I32;
            } else {
                self.bottom += self.offsets[next].y - self.offsets[self.index].y;
                self.index = next;
            }
        }
    }
}

// Source: src/core/SkAAClip.cpp:176-201
// Native header, YOffset array and encoded bytes occupy one malloc allocation.
// RunHead is the Rust owning pointer; its initialization bit prevents creating
// typed references before Builder/AllocRect have written every payload byte.
#[repr(C)]
struct NativeRunHead {
    ref_count: AtomicI32,
    row_count: i32,
    data_size: usize,
}
#[derive(Debug)]
struct RunHead {
    pointer: NonNull<NativeRunHead>,
    initialized: bool,
}
unsafe impl Send for RunHead {}
unsafe impl Sync for RunHead {}
unsafe extern "C" {
    #[link_name = "malloc"]
    fn aa_malloc(size: usize) -> *mut core::ffi::c_void;
    #[link_name = "free"]
    fn aa_free(pointer: *mut core::ffi::c_void);
}
impl Clone for RunHead {
    fn clone(&self) -> Self {
        assert!(self.initialized);
        let previous = self.ref_count().fetch_add(1, Ordering::SeqCst);
        debug_assert!(previous > 0);
        Self {
            pointer: self.pointer,
            initialized: true,
        }
    }
}
impl RunHead {
    fn native(&self) -> &NativeRunHead {
        unsafe { self.pointer.as_ref() }
    }
    fn ref_count(&self) -> &AtomicI32 {
        &self.native().ref_count
    }
    fn row_count(&self) -> usize {
        self.native().row_count as usize
    }
    fn data_size(&self) -> usize {
        self.native().data_size
    }
    fn offsets_ptr(&self) -> *mut YOffset {
        unsafe { self.pointer.as_ptr().add(1).cast() }
    }
    fn data_ptr(&self) -> *mut u8 {
        unsafe { self.offsets_ptr().add(self.row_count()).cast() }
    }
    fn get_mut(head: &mut Self) -> Option<&mut Self> {
        if head.ref_count().load(Ordering::SeqCst) == 1 {
            Some(head)
        } else {
            None
        }
    }
    fn offsets(&self) -> &[YOffset] {
        assert!(self.initialized);
        unsafe { core::slice::from_raw_parts(self.offsets_ptr(), self.row_count()) }
    }
    fn offsets_mut(&mut self) -> &mut [YOffset] {
        assert!(self.initialized);
        assert_eq!(self.ref_count().load(Ordering::SeqCst), 1);
        unsafe { core::slice::from_raw_parts_mut(self.offsets_ptr(), self.row_count()) }
    }
    fn data(&self) -> &[u8] {
        assert!(self.initialized);
        unsafe { core::slice::from_raw_parts(self.data_ptr(), self.data_size()) }
    }
    fn parts_mut(&mut self) -> (&mut [YOffset], &mut [u8]) {
        assert!(self.initialized);
        assert_eq!(self.ref_count().load(Ordering::SeqCst), 1);
        unsafe {
            (
                core::slice::from_raw_parts_mut(self.offsets_ptr(), self.row_count()),
                core::slice::from_raw_parts_mut(self.data_ptr(), self.data_size()),
            )
        }
    }
    fn write_offset(&mut self, index: usize, value: YOffset) {
        assert!(index < self.row_count());
        assert_eq!(self.ref_count().load(Ordering::SeqCst), 1);
        unsafe {
            self.offsets_ptr().add(index).write(value);
        }
    }
    fn write_data(&mut self, offset: usize, bytes: &[u8]) {
        assert!(offset
            .checked_add(bytes.len())
            .is_some_and(|n| n <= self.data_size()));
        assert_eq!(self.ref_count().load(Ordering::SeqCst), 1);
        unsafe {
            core::ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                self.data_ptr().add(offset),
                bytes.len(),
            );
        }
    }
    // Safety: caller has initialized exactly the returned byte subrange.
    unsafe fn written_data(&self, offset: usize, len: usize) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.data_ptr().add(offset), len) }
    }
    // Safety: caller has written every offset record and byte of native data.
    unsafe fn assume_initialized(&mut self) {
        self.initialized = true;
    }
    fn set_row_count(&mut self, count: usize) {
        assert_eq!(self.ref_count().load(Ordering::SeqCst), 1);
        assert!(count > 0 && count <= self.row_count());
        unsafe {
            self.pointer.as_mut().row_count = count as i32;
        }
    }
    fn move_offset_payload_left(&mut self, skip: usize) {
        assert_eq!(self.ref_count().load(Ordering::SeqCst), 1);
        let offset = skip * core::mem::size_of::<YOffset>();
        let size = self.row_count() * core::mem::size_of::<YOffset>() + self.data_size();
        let base = self.offsets_ptr().cast::<u8>();
        unsafe {
            core::ptr::copy(base.add(offset), base, size - offset);
        }
    }
    fn move_data_left(&mut self, skip: usize) {
        assert_eq!(self.ref_count().load(Ordering::SeqCst), 1);
        let offset = skip * core::mem::size_of::<YOffset>();
        let source = self.data_ptr();
        unsafe {
            core::ptr::copy(source, source.sub(offset), self.data_size());
        }
    }
    fn alloc(rows: usize, bytes: usize) -> Self {
        assert!(i32::try_from(rows).is_ok());
        let size = core::mem::size_of::<NativeRunHead>()
            .checked_add(
                rows.checked_mul(core::mem::size_of::<YOffset>())
                    .expect("AAClip allocation overflow"),
            )
            .and_then(|n| n.checked_add(bytes))
            .expect("AAClip allocation overflow");
        let raw = unsafe { aa_malloc(size) }.cast::<NativeRunHead>();
        let Some(pointer) = NonNull::new(raw) else {
            std::alloc::handle_alloc_error(
                std::alloc::Layout::from_size_align(size, core::mem::align_of::<NativeRunHead>())
                    .unwrap(),
            )
        };
        unsafe {
            pointer.as_ptr().write(NativeRunHead {
                ref_count: AtomicI32::new(1),
                row_count: rows as i32,
                data_size: bytes,
            });
        }
        Self {
            pointer,
            initialized: false,
        }
    }
    fn from_slices(offsets: &[YOffset], data: &[u8]) -> Self {
        let mut h = Self::alloc(offsets.len(), data.len());
        unsafe {
            core::ptr::copy_nonoverlapping(offsets.as_ptr(), h.offsets_ptr(), offsets.len());
        }
        h.write_data(0, data);
        unsafe {
            h.assume_initialized();
        }
        h
    }
}
// Source: src/core/SkAAClip.cpp:203-243
impl RunHead {
    fn compute_row_size_for_width(mut width: i32) -> usize {
        let mut segments = 0;
        while width > 0 {
            segments += 1;
            width -= width.min(255);
        }
        segments * 2
    }
    fn alloc_rect(bounds: &SkIRect) -> Self {
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 215; @condition b"!bounds.isEmpty()\0".as_ptr(); !bounds.is_empty());
        let mut width = bounds.width();
        let mut head = Self::alloc(1, Self::compute_row_size_for_width(width));
        head.write_offset(
            0,
            YOffset {
                y: bounds.height() - 1,
                offset: 0,
            },
        );
        let mut p = 0;
        while width > 0 {
            let n = width.min(255);
            head.write_data(p, &[n as u8, 255]);
            width -= n;
            p += 2;
        }
        unsafe {
            head.assume_initialized();
        }
        head
    }
    fn iterate(clip: &SkAAClip) -> Iter<'_> {
        match &clip.head {
            None => Iter::empty(),
            Some(h) => Iter::new(clip.bounds.fTop, h.data(), h.offsets()),
        }
    }
}

// Source: src/core/SkAAClip.cpp:247-278
// Vec<Row> owns each row byte buffer; indices replace the upstream pointers
// that could be invalidated by array growth.
struct Builder {
    bounds: SkIRect,
    rows: Vec<Row>,
    curr: Option<usize>,
    prev_y: i32,
    width: i32,
    min_y: i32,
}
struct Row {
    y: i32,
    width: i32,
    data: Vec<u8>,
}
impl Builder {
    fn new(bounds: SkIRect) -> Self {
        Self {
            bounds,
            rows: Vec::new(),
            curr: None,
            prev_y: -1,
            width: bounds.width(),
            min_y: bounds.fTop,
        }
    }
}

// Source: src/core/SkAAClip.cpp:287-322
impl Builder {
    fn add_run(&mut self, mut x: i32, mut y: i32, alpha: u8, count: i32) {
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 288; @condition b"count > 0\0".as_ptr(); count > 0);
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 289; @condition b"fBounds.contains(x, y)\0".as_ptr(); self.bounds.contains_xy(x, y));
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 290; @condition b"fBounds.contains(x + count - 1, y)\0".as_ptr(); self.bounds.contains_xy(x + count - 1, y));
        x -= self.bounds.fLeft;
        y -= self.bounds.fTop;
        if y != self.prev_y {
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 297; @condition b"y > fPrevY\0".as_ptr(); y > self.prev_y);
            self.prev_y = y;
            let index = self.flush_row(true).unwrap();
            let row = &mut self.rows[index];
            row.y = y;
            row.width = 0;
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 303; @condition b"row->fData->empty()\0".as_ptr(); row.data.is_empty());
            self.curr = Some(index);
        }
        let row = &mut self.rows[self.curr.unwrap()];
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 307; @condition b"row->fWidth <= x\0".as_ptr(); row.width <= x);
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 308; @condition b"row->fWidth < fBounds.width()\0".as_ptr(); row.width < self.bounds.width());
        let gap = x - row.width;
        if gap != 0 {
            Self::append_run(&mut row.data, 0, gap);
            row.width += gap;
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 316; @condition b"row->fWidth < fBounds.width()\0".as_ptr(); row.width < self.bounds.width());
        }
        Self::append_run(&mut row.data, alpha, count);
        row.width += count;
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 321; @condition b"row->fWidth <= fBounds.width()\0".as_ptr(); row.width <= self.bounds.width());
    }
}

// Source: src/core/SkAAClip.cpp:324-345
impl Builder {
    fn add_column(&mut self, x: i32, mut y: i32, alpha: u8, height: i32) {
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 325; @condition b"fBounds.contains(x, y + height - 1)\0".as_ptr(); self.bounds.contains_xy(x, y + height - 1));
        self.add_run(x, y, alpha, 1);
        let i = self.curr.unwrap();
        self.flush_row_h(i);
        y -= self.bounds.fTop;
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 330; @condition b"y == fCurrRow->fY\0".as_ptr(); (y) == ( self.rows[i].y));
        self.rows[i].y = y + height - 1;
    }
    fn add_rect_run(&mut self, x: i32, mut y: i32, width: i32, height: i32) {
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 335; @condition b"fBounds.contains(x + width - 1, y + height - 1)\0".as_ptr(); self.bounds.contains_xy(x + width - 1, y + height - 1));
        self.add_run(x, y, 255, width);
        let i = self.curr.unwrap();
        self.flush_row_h(i);
        y -= self.bounds.fTop;
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 343; @condition b"y == fCurrRow->fY\0".as_ptr(); (y) == ( self.rows[i].y));
        self.rows[i].y = y + height - 1;
    }
}

// Source: src/core/SkAAClip.cpp:347-386
impl Builder {
    fn add_anti_rect_run(
        &mut self,
        mut x: i32,
        mut y: i32,
        mut width: i32,
        height: i32,
        left: u8,
        right: u8,
    ) {
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 352; @condition b"fBounds.contains(x + width + (rightAlpha > 0 ? 1 : 0), y + height - 1)\0".as_ptr(); self
            .bounds
            .contains_xy(x + width + i32::from(right > 0), y + height - 1));
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 354; @condition b"width >= 0\0".as_ptr(); width >= 0);
        if left == 255 {
            width += 1;
        } else if left > 0 {
            self.add_run(x, y, left, 1);
            x += 1;
        } else {
            x += 1;
        }
        if right == 255 {
            width += 1;
        }
        if width > 0 {
            self.add_run(x, y, 255, width);
        }
        if right > 0 && right < 255 {
            self.add_run(x + width, y, right, 1);
        }
        if let Some(i) = self.curr {
            self.flush_row_h(i);
            y -= self.bounds.fTop;
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 383; @condition b"y == fCurrRow->fY\0".as_ptr(); (y) == ( self.rows[i].y));
            self.rows[i].y = y + height - 1;
        }
    }
}

// Source: src/core/SkAAClip.cpp:388-436
impl Builder {
    fn finish(&mut self, target: &mut SkAAClip) -> bool {
        self.flush_row(false);
        let size = self.rows.iter().map(|r| r.data.len()).sum();
        if size == 0 {
            return target.set_empty();
        }
        {
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 404; @condition b"fMinY >= fBounds.fTop\0".as_ptr(); self.min_y >= self.bounds.fTop);
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 405; @condition b"fMinY < fBounds.fBottom\0".as_ptr(); self.min_y < self.bounds.fBottom);
        };
        let adjust = self.min_y - self.bounds.fTop;
        self.bounds.fTop = self.min_y;
        let mut head = RunHead::alloc(self.rows.len(), size);
        let mut p = 0;
        #[cfg(sk_debug)]
        let mut prev_y = self.rows[0].y - 1;
        for (i, row) in self.rows.iter().enumerate() {
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 417; @condition b"prevY < row->fY\0".as_ptr(); prev_y < row.y);
            #[cfg(sk_debug)]
            {
                prev_y = row.y;
            }
            head.write_offset(
                i,
                YOffset {
                    y: row.y - adjust,
                    offset: p as u32,
                },
            );
            let n = row.data.len();
            head.write_data(p, &row.data);
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 426; @condition b"compute_row_length(data, fBounds.width()) == n\0".as_ptr(); (
                compute_row_length(unsafe { head.written_data(p, n) }, self.bounds.width())) == (
                n
            ));
            p += n;
        }
        unsafe {
            head.assume_initialized();
        }
        target.head = None;
        target.bounds = self.bounds;
        target.head = Some(head);
        target.trim_bounds()
    }
}

// Source: src/core/SkAAClip.cpp:438-454
impl Builder {
    fn dump(&self) {
        self.validate();
        for row in &self.rows {
            unsafe {
                crate::SkDebugf!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 443; b"Y:%3d W:%3d\0".as_ptr(),row.y as core::ffi::c_int,row.width as core::ffi::c_int);
            }
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 446; @condition b"!(count & 1)\0".as_ptr(); (row.data.len() & 1) == ( 0));
            for pair in row.data.chunks_exact(2) {
                unsafe {
                    crate::SkDebugf!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 449; b" [%3d:%02X]\0".as_ptr(),pair[0] as core::ffi::c_int,pair[1] as core::ffi::c_int);
                }
            }
            unsafe {
                crate::SkDebugf!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 452; b"\n\0".as_ptr());
            }
        }
    }
}

// Source: src/core/SkAAClip.cpp:456-478
impl Builder {
    fn validate(&self) {
        if !cfg!(sk_debug) {
            return;
        }
        let mut prev = -1;
        for row in &self.rows {
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 461; @condition b"prevY < row.fY\0".as_ptr(); prev < row.y);
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 462; @condition b"fWidth == row.fWidth\0".as_ptr(); (self.width) == (row.width));
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 465; @condition b"!(count & 1)\0".as_ptr(); (row.data.len() & 1) == ( 0));
            let mut w = 0;
            for pair in row.data.chunks_exact(2) {
                let n = i32::from(pair[0]);
                crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 469; @condition b"n > 0\0".as_ptr(); n > 0);
                w += n;
                crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 471; @condition b"w <= fWidth\0".as_ptr(); w <= self.width);
            }
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 474; @condition b"w == fWidth\0".as_ptr(); (w) == (self.width));
            prev = row.y;
        }
    }
}

// Source: src/core/SkAAClip.cpp:480-522
impl Builder {
    fn flush_row_h(&mut self, i: usize) {
        let row = &mut self.rows[i];
        if row.width < self.width {
            Self::append_run(&mut row.data, 0, self.width - row.width);
            row.width = self.width;
        }
    }
    fn flush_row(&mut self, ready: bool) -> Option<usize> {
        let mut next = None;
        let count = self.rows.len();
        if count > 0 {
            self.flush_row_h(count - 1);
        }
        if count > 1 {
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 498; @condition b"prev->fWidth == fWidth\0".as_ptr(); (self.rows[count - 2].width) == ( self.width));
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 499; @condition b"curr->fWidth == fWidth\0".as_ptr(); (self.rows[count - 1].width) == ( self.width));
            if self.rows[count - 2].data == self.rows[count - 1].data {
                self.rows[count - 2].y = self.rows[count - 1].y;
                if ready {
                    self.rows[count - 1].data.clear();
                    next = Some(count - 1);
                } else {
                    self.rows.pop();
                }
            } else if ready {
                next = Some(self.rows.len());
                self.rows.push(Row {
                    y: 0,
                    width: 0,
                    data: Vec::new(),
                });
            }
        } else if ready {
            next = Some(self.rows.len());
            self.rows.push(Row {
                y: 0,
                width: 0,
                data: Vec::new(),
            });
        }
        next
    }
}

// Source: src/core/SkAAClip.cpp:524-535
impl Builder {
    fn append_run(data: &mut Vec<u8>, alpha: u8, mut count: i32) {
        loop {
            let n = count.min(255);
            data.extend_from_slice(&[n as u8, alpha]);
            count -= n;
            if count <= 0 {
                break;
            }
        }
    }
}

// Source: src/core/SkAAClip.cpp:538-602
impl Builder {
    fn operate_x(
        &mut self,
        last_y: i32,
        a: &mut RowIter<'_>,
        b: &mut RowIter<'_>,
        proc: fn(u8, u8) -> u8,
    ) {
        fn advance(iter: &mut RowIter<'_>, left: &mut i32, right: &mut i32, end: i32) {
            if end == *right {
                iter.next();
                *left = iter.left;
                *right = iter.right;
            }
        }
        let (mut al, mut ar, mut bl, mut br) = (a.left, a.right, b.left, b.right);
        let mut prev_right = self.bounds.fLeft;
        loop {
            let (mut aa, mut ba) = (0, 0);
            let (left, mut right);
            if al < bl {
                left = al;
                aa = a.alpha;
                if ar <= bl {
                    right = ar;
                } else {
                    al = bl;
                    right = al;
                }
            } else if bl < al {
                left = bl;
                ba = b.alpha;
                if br <= al {
                    right = br;
                } else {
                    bl = al;
                    right = bl;
                }
            } else {
                left = al;
                right = ar.min(br);
                al = right;
                bl = right;
                aa = a.alpha;
                ba = b.alpha;
            }
            if left >= self.bounds.fRight {
                break;
            }
            if right > self.bounds.fRight {
                right = self.bounds.fRight;
            }
            if left >= self.bounds.fLeft {
                crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 590; @condition b"rite > left\0".as_ptr(); right > left);
                self.add_run(left, last_y, proc(aa, ba), right - left);
                prev_right = right;
            }
            advance(a, &mut al, &mut ar, right);
            advance(b, &mut bl, &mut br, right);
            if a.done && b.done {
                break;
            }
        }
        if prev_right < self.bounds.fRight {
            self.add_run(prev_right, last_y, 0, self.bounds.fRight - prev_right);
        }
    }
}

// Source: src/core/SkAAClip.cpp:604-627
impl Builder {
    fn operate_y(
        &mut self,
        a: &SkAAClip,
        b: &SkAAClip,
        op: crate::include::core::SkClipOp::SkClipOp,
    ) {
        use crate::include::core::SkClipOp::SkClipOp;
        use crate::include::private::SkMath::SkMulDiv255Round;
        let proc: fn(u8, u8) -> u8 = if op == SkClipOp::kDifference {
            |a, b| SkMulDiv255Round(u32::from(a), u32::from(255 - b)) as u8
        } else {
            |a, b| SkMulDiv255Round(u32::from(a), u32::from(b)) as u8
        };
        let (mut ia, mut ib) = (RunHead::iterate(a), RunHead::iterate(b));
        {
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 612; @condition b"!iterA.done()\0".as_ptr(); !ia.done);
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 615; @condition b"!iterB.done()\0".as_ptr(); !ib.done);
        };
        let (mut at, mut ab, mut bt, mut bb) = (ia.top, ia.bottom, ib.top, ib.bottom);
        fn advance(i: &mut Iter<'_>, top: &mut i32, bottom: &mut i32, end: i32) {
            if end == *bottom {
                i.next();
                *top = *bottom;
                crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 623; @condition b"iterBot == iter.top()\0".as_ptr(); (*bottom) == ( i.top));
                *bottom = i.bottom;
            }
        }
        // Source: src/core/SkAAClip.cpp:634-684
        loop {
            let (mut ra, mut rb) = (None, None);
            let (top, mut bottom);
            if at < bt {
                top = at;
                ra = ia.data();
                if ab <= bt {
                    bottom = ab;
                } else {
                    at = bt;
                    bottom = at;
                }
            } else if bt < at {
                top = bt;
                rb = ib.data();
                if bb <= at {
                    bottom = bb;
                } else {
                    bt = at;
                    bottom = bt;
                }
            } else {
                top = at;
                bottom = ab.min(bb);
                at = bottom;
                bt = bottom;
                ra = ia.data();
                rb = ib.data();
            }
            if top >= self.bounds.fBottom {
                break;
            }
            if bottom > self.bounds.fBottom {
                bottom = self.bounds.fBottom;
            }
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 670; @condition b"top < bot\0".as_ptr(); top < bottom);
            if ra.is_none() && rb.is_none() {
                self.add_run(self.bounds.fLeft, bottom - 1, 0, self.bounds.width());
            } else if top >= self.bounds.fTop {
                crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 675; @condition b"bot <= fBounds.fBottom\0".as_ptr(); bottom <= self.bounds.fBottom);
                let mut ria = RowIter::new(
                    ra,
                    if ra.is_some() {
                        a.get_bounds()
                    } else {
                        &self.bounds
                    },
                );
                let mut rib = RowIter::new(
                    rb,
                    if rb.is_some() {
                        b.get_bounds()
                    } else {
                        &self.bounds
                    },
                );
                self.operate_x(bottom - 1, &mut ria, &mut rib, proc);
            }
            advance(&mut ia, &mut at, &mut ab, bottom);
            advance(&mut ib, &mut bt, &mut bb, bottom);
            if ia.done && ib.done {
                break;
            }
        }
    }
}

// Source: src/core/SkAAClip.cpp:686-719
struct BuilderBlitter<'a> {
    // Rust composition retains the original SkBlitter base allocation lifetime.
    storage: crate::src::core::SkBlitter::SkBlitterStorage,
    builder: &'a mut Builder,
    last_y: i32,
    left: i32,
    right: i32,
    min_y: i32,
}
impl<'a> BuilderBlitter<'a> {
    fn new(builder: &'a mut Builder) -> Self {
        let (left, right) = (builder.bounds.fLeft, builder.bounds.fRight);
        Self {
            storage: crate::src::core::SkBlitter::SkBlitterStorage::new(),
            builder,
            last_y: -i32::MAX,
            left,
            right,
            min_y: i32::MAX,
        }
    }
    fn check_for_y_gap(&mut self, y: i32) {
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 696; @condition b"y >= fLastY\0".as_ptr(); y >= self.last_y);
        if self.last_y > -i32::MAX {
            let gap = y - self.last_y;
            if gap > 1 {
                self.builder
                    .add_run(self.left, y - 1, 0, self.right - self.left);
            }
        }
        self.last_y = y;
    }
    fn finish(&mut self) {
        if self.min_y < i32::MAX {
            self.builder.min_y = self.min_y;
        }
    }
}

// Source: src/core/SkAAClip.cpp:721-755
impl BuilderBlitter<'_> {
    fn blit_v(&mut self, x: i32, y: i32, height: i32, alpha: u8) {
        if height == 1 {
            let (mut aa, mut runs) = ([alpha, 0], [1, 0]);
            self.blit_anti_h(x, y, &mut aa, &mut runs);
        } else {
            self.record_min_y(y);
            self.builder.add_column(x, y, alpha, height);
            self.last_y = y + height - 1;
        }
    }
    fn blit_rect(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.record_min_y(y);
        self.check_for_y_gap(y);
        self.builder.add_rect_run(x, y, width, height);
        self.last_y = y + height - 1;
    }
    fn blit_anti_rect(&mut self, x: i32, y: i32, width: i32, height: i32, left: u8, right: u8) {
        self.record_min_y(y);
        self.check_for_y_gap(y);
        self.builder
            .add_anti_rect_run(x, y, width, height, left, right);
        self.last_y = y + height - 1;
    }
}

// Source: src/core/SkAAClip.cpp:760-806
impl BuilderBlitter<'_> {
    fn blit_h(&mut self, x: i32, y: i32, width: i32) {
        self.record_min_y(y);
        self.check_for_y_gap(y);
        self.builder.add_run(x, y, 255, width);
    }
    fn blit_anti_h(&mut self, mut x: i32, y: i32, alpha: &mut [u8], runs: &mut [i16]) {
        self.record_min_y(y);
        self.check_for_y_gap(y);
        let mut p = 0;
        loop {
            let count = i32::from(runs[p]);
            if count <= 0 {
                return;
            }
            let (mut local_x, mut local_count) = (x, count);
            if x < self.left {
                crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 785; @condition b"0x10 > *alpha\0".as_ptr(); alpha[p] < 16);
                let gap = self.left - x;
                crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 787; @condition b"gap <= count\0".as_ptr(); gap <= count);
                local_x += gap;
                local_count -= gap;
            }
            let right = x + count;
            if right > self.right {
                crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 793; @condition b"0x10 > *alpha\0".as_ptr(); alpha[p] < 16);
                local_count -= right - self.right;
                crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 795; @condition b"localCount >= 0\0".as_ptr(); local_count >= 0);
            }
            if local_count != 0 {
                self.builder.add_run(local_x, y, alpha[p], local_count);
            }
            p += count as usize;
            x += count;
        }
    }
}

// Source: src/core/SkAAClip.cpp:808-824
impl BuilderBlitter<'_> {
    fn record_min_y(&mut self, y: i32) {
        if y < self.min_y {
            self.min_y = y;
        }
    }
}
// Dynamic dispatch wiring; algorithm bodies above retain original source order.
// Safety: each blitter owns SkBlitterStorage for its full lifetime; allocation
// pointers remain valid through the next reset or the owner's drop.
unsafe impl crate::src::core::SkBlitter::SkBlitter for BuilderBlitter<'_> {
    fn allocBlitMemory(self: core::pin::Pin<&mut Self>, sz: usize) -> *mut u8 {
        let this = unsafe { self.get_unchecked_mut() };
        this.storage.allocBlitMemory(sz)
    }
    // Source: src/core/SkAAClip.cpp:757-758
    unsafe fn blitMask(
        self: core::pin::Pin<&mut Self>,
        _mask: &crate::src::core::SkMask::SkMask,
        _clip: &SkIRect,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        this.unexpected();
    }
    unsafe fn blitH(self: core::pin::Pin<&mut Self>, x: i32, y: i32, width: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { this.blit_h(x, y, width) }
    }
    unsafe fn blitAntiH(
        self: core::pin::Pin<&mut Self>,
        x: i32,
        y: i32,
        aa: &mut [u8],
        runs: &mut [i16],
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { this.blit_anti_h(x, y, aa, runs) }
    }
    unsafe fn blitV(self: core::pin::Pin<&mut Self>, x: i32, y: i32, height: i32, alpha: u8) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { this.blit_v(x, y, height, alpha) }
    }
    unsafe fn blitRect(self: core::pin::Pin<&mut Self>, x: i32, y: i32, width: i32, height: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { this.blit_rect(x, y, width, height) }
    }
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
        unsafe { this.blit_anti_rect(x, y, width, height, left, right) }
    }
}

// Source: src/core/SkAAClip.cpp:826-829
impl BuilderBlitter<'_> {
    fn unexpected(&self) -> ! {
        crate::SK_ABORT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 827; "---- did not expect to get called here");
    }
}

// Source: src/core/SkAAClip.cpp:831-834
impl Builder {
    fn apply_clip_op(
        &mut self,
        target: &mut SkAAClip,
        other: &SkAAClip,
        op: crate::include::core::SkClipOp::SkClipOp,
    ) -> bool {
        self.operate_y(target, other, op);
        self.finish(target)
    }
}

// Source: src/core/SkAAClip.cpp:836-849
impl Builder {
    fn blit_path(
        &mut self,
        target: &mut SkAAClip,
        path: &crate::src::core::SkPath::SkPath,
        do_aa: bool,
    ) -> bool {
        use crate::src::core::SkPathEnums::SkResolveConvexity;
        use crate::src::core::SkPathPriv::SkPathPriv;
        use crate::src::core::SkPathRaw::SkPathRaw;
        use crate::src::core::SkScan::SkScan;
        let mut blitter = Box::pin(BuilderBlitter::new(self));
        let clip = SkRegion::from_rect(blitter.builder.bounds);
        let raw = SkPathPriv::Raw(path, SkResolveConvexity::kYes)
            .unwrap_or(SkPathRaw::Empty(path.getFillType()));
        if do_aa {
            // The exclusive builder owns all output; clip bounds constrain coordinates.
            unsafe {
                SkScan::AntiFillPath(&raw, &clip, blitter.as_mut(), true);
            }
        } else {
            // The exclusive builder owns all output; clip bounds constrain coordinates.
            unsafe {
                SkScan::FillPath(&raw, &clip, blitter.as_mut());
            }
        }
        unsafe { blitter.as_mut().get_unchecked_mut() }.finish();
        // Keep the blitter's native base allocation alive until after builder
        // completion, exactly as the source local object's return-time drop.
        unsafe { blitter.as_mut().get_unchecked_mut() }
            .builder
            .finish(target)
    }
}

// Source: src/core/SkAAClip.cpp:853-891
impl SkAAClip {
    pub fn copy_to_mask(&self, mask: &mut crate::src::core::SkMask::SkMaskBuilder) {
        use crate::src::core::SkMask::{AllocType, Format, SkMaskBuilder};
        fn expand(dst: &mut [u8], mut row: &[u8], mut width: i32) {
            let mut p = 0;
            while width > 0 {
                let n = i32::from(row[0]);
                crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 857; @condition b"width >= n\0".as_ptr(); width >= n);
                dst[p..p + n as usize].fill(row[1]);
                p += n as usize;
                row = &row[2..];
                width -= n;
            }
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 863; @condition b"0 == width\0".as_ptr(); (width) == (0));
        }
        *mask.format() = Format::kA8_Format;
        if self.is_empty() {
            *mask.bounds() = SkIRect::make_empty();
            *mask.image() = core::ptr::null_mut();
            *mask.rowBytes() = 0;
            return;
        }
        *mask.bounds() = self.bounds;
        *mask.rowBytes() = self.bounds.width() as u32;
        let size = mask.computeImageSize();
        let image = SkMaskBuilder::AllocImage(size, AllocType::kUninit_Alloc);
        *mask.image() = image;
        // AllocImage reserves exactly the descriptor size; each row below is
        // expanded from validated RLE, and destination ownership stays caller's.
        let dst = unsafe { core::slice::from_raw_parts_mut(image, size) };
        let mut iter = RunHead::iterate(self);
        let width = self.bounds.width();
        let row_bytes = mask.fRowBytes as usize;
        let mut y = self.bounds.fTop;
        let mut p = 0;
        while !iter.done {
            loop {
                expand(&mut dst[p..p + row_bytes], iter.data().unwrap(), width);
                p += row_bytes;
                y += 1;
                if y >= iter.bottom {
                    break;
                }
            }
            iter.next();
        }
    }
}

// Source: src/core/SkAAClip.cpp:895-927
impl SkAAClip {
    // Source: src/core/SkAAClip.h:65-65
    // Source: src/core/SkAAClip.cpp:895-927
    #[cfg(sk_debug)]
    pub fn validate(&self) {
        let Some(h) = &self.head else {
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 897; @condition b"fBounds.isEmpty()\0".as_ptr(); self.bounds.is_empty());
            return;
        };
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 900; @condition b"!fBounds.isEmpty()\0".as_ptr(); !self.bounds.is_empty());
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 903; @condition b"head->fRefCnt.load() > 0\0".as_ptr(); h.ref_count().load(Ordering::SeqCst)>0);
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 904; @condition b"head->fRowCount > 0\0".as_ptr(); h.native().row_count>0);
        let last_y = self.bounds.height() - 1;
        let mut prev_y = -1;
        let mut prev_offset = -1i32;
        for yoff in h.offsets() {
            {
                crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 914; @condition b"prevY < yoff->fY\0".as_ptr(); prev_y<yoff.y);
                crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 915; @condition b"yoff->fY <= lastY\0".as_ptr(); yoff.y<=last_y);
            };
            prev_y = yoff.y;
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 917; @condition b"prevOffset < (int32_t)yoff->fOffset\0".as_ptr(); prev_offset < (yoff.offset as i32));
            prev_offset = yoff.offset as i32;
            let length = compute_row_length(&h.data()[yoff.offset as usize..], self.bounds.width());
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 921; @condition b"yoff->fOffset + rowLength <= head->fDataSize\0".as_ptr(); yoff.offset as usize + length <= h.data().len());
        }
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 926; @condition b"yoff->fY == lastY\0".as_ptr(); (h.offsets().last().unwrap().y) == ( last_y));
    }
    #[cfg(not(sk_debug))]
    pub fn validate(&self) {}
}

// Source: src/core/SkAAClip.cpp:929-968
#[cfg(sk_debug)]
fn dump_one_row(mut row: &[u8], mut width: i32, leading: i32) {
    if leading != 0 {
        unsafe {
            crate::SkDebugf!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 932; b"%03d \0".as_ptr(),leading as core::ffi::c_int);
        }
    }
    while width > 0 {
        let n = i32::from(row[0]);
        let value = row[1];
        let out = if value == 255 {
            '*'
        } else if value > 0 {
            '+'
        } else {
            '.'
        };
        for _ in 0..n {
            unsafe {
                crate::SkDebugf!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 944; b"%c\0".as_ptr(),out as core::ffi::c_int);
            }
        }
        row = &row[2..];
        width -= n;
    }
    unsafe {
        crate::SkDebugf!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 949; b"\n\0".as_ptr());
    }
}
impl SkAAClip {
    // Source: src/core/SkAAClip.h:66-66
    #[cfg(sk_debug)]
    pub fn debug(&self, compress_y: bool) {
        let mut iter = RunHead::iterate(self);
        let width = self.bounds.width();
        let mut y = self.bounds.fTop;
        while !iter.done {
            if compress_y {
                dump_one_row(iter.data().unwrap(), width, iter.bottom - iter.top + 1);
            } else {
                loop {
                    dump_one_row(iter.data().unwrap(), width, 0);
                    y += 1;
                    if y >= iter.bottom {
                        break;
                    }
                }
            }
            iter.next();
        }
    }
    #[cfg(not(sk_debug))]
    pub fn debug(&self, compress_y: bool) {}
}

// Source: src/core/SkAAClip.cpp:972-1009
fn count_left_right_zeros(mut row: &[u8], mut width: i32) -> (i32, i32) {
    let mut zeros = 0;
    loop {
        if row[1] != 0 {
            break;
        }
        let n = i32::from(row[0]);
        {
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 982; @condition b"n > 0\0".as_ptr(); n > 0);
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 983; @condition b"n <= width\0".as_ptr(); n <= width);
        };
        zeros += n;
        row = &row[2..];
        width -= n;
        if width <= 0 {
            break;
        }
    }
    let left = zeros;
    if width == 0 {
        return (left, left);
    }
    zeros = 0;
    while width > 0 {
        let n = i32::from(row[0]);
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 982; @condition b"n > 0\0".as_ptr(); n > 0);
        if row[1] == 0 {
            zeros += n;
        } else {
            zeros = 0;
        }
        row = &row[2..];
        width -= n;
    }
    (left, zeros)
}

// Source: src/core/SkAAClip.cpp:1011-1055
fn trim_row_left_right(row: &mut [u8], mut width: i32, mut left: i32, mut right: i32) -> usize {
    let (mut trim, mut p) = (0, 0);
    while left > 0 {
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1016; @condition b"0 == row[1]\0".as_ptr(); (row[p + 1]) == ( 0));
        let n = i32::from(row[p]);
        {
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1018; @condition b"n > 0\0".as_ptr(); n > 0);
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1019; @condition b"n <= width\0".as_ptr(); n <= width);
        };
        width -= n;
        p += 2;
        if n > left {
            row[p - 2] = (n - left) as u8;
            break;
        }
        trim += 2;
        left -= n;
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1028; @condition b"leftZ >= 0\0".as_ptr(); left >= 0);
    }
    if right != 0 {
        while width > 0 {
            let n = i32::from(row[p]);
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1019; @condition b"n <= width\0".as_ptr(); n <= width);
            width -= n;
            p += 2;
        }
        loop {
            p -= 2;
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1042; @condition b"0 == row[1]\0".as_ptr(); (row[p + 1]) == ( 0));
            let n = i32::from(row[p]);
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1018; @condition b"n > 0\0".as_ptr(); n > 0);
            if n > right {
                row[p] = (n - right) as u8;
                break;
            }
            right -= n;
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1050; @condition b"riteZ >= 0\0".as_ptr(); right >= 0);
            if right <= 0 {
                break;
            }
        }
    }
    trim
}

// Source: src/core/SkAAClip.cpp:1057-1116
impl SkAAClip {
    fn trim_left_right(&mut self) -> bool {
        if self.is_empty() {
            return false;
        }
        #[cfg(sk_debug)]
        let _validate = unsafe { AutoAAClipValidate::new(self as *const _) };
        let width = self.bounds.width();
        let head = self.head.as_ref().unwrap();
        let (mut left, mut right) = (width, width);
        for yoff in head.offsets() {
            let (l, r) = count_left_right_zeros(&head.data()[yoff.offset as usize..], width);
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1078; @condition b"L + R < width || (L == width && R == width)\0".as_ptr(); l + r < width || (l == width && r == width));
            left = left.min(l);
            right = right.min(r);
            if left | right == 0 {
                return true;
            }
        }
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1092; @condition b"leftZeros || riteZeros\0".as_ptr(); left != 0 || right != 0);
        if width == left {
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1094; @condition b"width == riteZeros\0".as_ptr(); (width) == ( right));
            return self.set_empty();
        }
        self.validate();
        self.bounds.fLeft += left;
        self.bounds.fRight -= right;
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1102; @condition b"!fBounds.isEmpty()\0".as_ptr(); !self.bounds.is_empty());
        let head =
            RunHead::get_mut(self.head.as_mut().unwrap()).expect("Builder owns unique AAClip");
        let (offsets, data) = head.parts_mut();
        for yoff in offsets {
            let row = &mut data[yoff.offset as usize..];
            #[cfg(sk_debug)]
            {
                let _ = compute_row_length(row, width);
            };
            yoff.offset += trim_row_left_right(row, width, left, right) as u32;
            #[cfg(sk_debug)]
            {
                let _ = compute_row_length(&data[yoff.offset as usize..], width - left - right);
            };
        }
        true
    }
}

// Source: src/core/SkAAClip.cpp:1118-1131
fn row_is_all_zeros(mut row: &[u8], mut width: i32) -> bool {
    crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1119; @condition b"width > 0\0".as_ptr(); width > 0);
    loop {
        if row[1] != 0 {
            return false;
        }
        let n = i32::from(row[0]);
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1125; @condition b"n <= width\0".as_ptr(); n <= width);
        width -= n;
        row = &row[2..];
        if width <= 0 {
            break;
        }
    }
    crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1129; @condition b"0 == width\0".as_ptr(); (width) == (0));
    true
}

// Source: src/core/SkAAClip.cpp:1133-1207
impl SkAAClip {
    fn trim_top_bottom(&mut self) -> bool {
        if self.is_empty() {
            return false;
        }
        self.validate();
        let width = self.bounds.width();
        let head = self.head.as_ref().unwrap();
        let mut skip = 0;
        while skip < head.offsets().len() {
            if !row_is_all_zeros(&head.data()[head.offsets()[skip].offset as usize..], width) {
                break;
            }
            skip += 1;
        }
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1157; @condition b"skip <= head->fRowCount\0".as_ptr(); skip <= head.offsets().len());
        if skip == head.offsets().len() {
            return self.set_empty();
        }
        if skip > 0 {
            let head =
                RunHead::get_mut(self.head.as_mut().unwrap()).expect("Builder owns unique AAClip");
            let dy = head.offsets()[skip - 1].y + 1;
            for yoff in &mut head.offsets_mut()[skip..] {
                crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1167; @condition b"yoff[i].fY >= dy\0".as_ptr(); yoff.y >= dy);
                yoff.y -= dy;
            }
            // Move offsets and byte payload together, retaining the source
            // single-allocation layout and subsequent data-base adjustment.
            head.move_offset_payload_left(skip);
            self.bounds.fTop += dy;
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1175; @condition b"!fBounds.isEmpty()\0".as_ptr(); !self.bounds.is_empty());
            head.set_row_count(head.row_count() - skip);
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1177; @condition b"head->fRowCount > 0\0".as_ptr(); !head.offsets().is_empty());
            self.validate();
        }
        let head =
            RunHead::get_mut(self.head.as_mut().unwrap()).expect("Builder owns unique AAClip");
        let mut last = head.offsets().len() - 1;
        while row_is_all_zeros(&head.data()[head.offsets()[last].offset as usize..], width) {
            last -= 1;
        }
        let skip = head.offsets().len() - last - 1;
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1193; @condition b"skip >= 0 && skip < head->fRowCount\0".as_ptr(); skip < head.offsets().len());
        if skip > 0 {
            head.move_data_left(skip);
            self.bounds.fBottom = self.bounds.fTop + head.offsets()[last].y + 1;
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1200; @condition b"!fBounds.isEmpty()\0".as_ptr(); !self.bounds.is_empty());
            head.set_row_count(head.row_count() - skip);
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1202; @condition b"head->fRowCount > 0\0".as_ptr(); !head.offsets().is_empty());
        }
        self.validate();
        true
    }
}

// Source: src/core/SkAAClip.cpp:1209-1232
impl SkAAClip {
    fn trim_bounds(&mut self) -> bool {
        if self.is_empty() {
            return false;
        }
        let head = self.head.as_ref().unwrap();
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1224; @condition b"head->fRowCount > 0\0".as_ptr(); !head.offsets().is_empty());
        let last = head.offsets().last().unwrap();
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1226; @condition b"lastY.fY + 1 <= fBounds.height()\0".as_ptr(); last.y + 1 <= self.bounds.height());
        self.bounds.fBottom = self.bounds.fTop + last.y + 1;
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1228; @condition b"lastY.fY + 1 == fBounds.height()\0".as_ptr(); (last.y + 1) == ( self.bounds.height()));
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1229; @condition b"!fBounds.isEmpty()\0".as_ptr(); !self.bounds.is_empty());
        self.trim_top_bottom() && self.trim_left_right()
    }
}

// Source: src/core/SkAAClip.cpp:1236-1245
impl SkAAClip {
    pub fn new() -> Self {
        Self {
            bounds: SkIRect::make_empty(),
            head: None,
        }
    }
    // Source: src/core/SkAAClip.cpp:1241-1245
    pub fn from_clip(src: &Self) -> Self {
        // The release copy constructor has no bounds value until operator=.
        // Raw field writes keep that construction phase out of Rust references.
        let mut storage = core::mem::MaybeUninit::<Self>::uninit();
        let dst = storage.as_mut_ptr();
        unsafe {
            #[cfg(sk_debug)]
            core::ptr::addr_of_mut!((*dst).bounds).write(SkIRect::make_empty());
            core::ptr::addr_of_mut!((*dst).head).write(None);
            {
                #[cfg(sk_debug)]
                let _validate = AutoAAClipValidate::new(dst);
                src.validate();
                // freeRuns sees the null head installed above. No full-object
                // reference is formed in the release indeterminate-bounds phase.
                core::ptr::addr_of_mut!((*dst).bounds).write(src.bounds);
                core::ptr::addr_of_mut!((*dst).head).write(src.head.clone());
            }
            storage.assume_init()
        }
    }
}

// Source: src/core/SkAAClip.cpp:1247-1264
impl SkAAClip {
    // Drop and clone perform the source's actual atomic decrement/free and shared
    // assignment. Rust borrows prohibit aliased mutable self assignment.
    pub fn assign(&mut self, src: &Self) {
        #[cfg(sk_debug)]
        let _validate = unsafe { AutoAAClipValidate::new(self as *const _) };
        src.validate();
        self.head = None;
        self.bounds = src.bounds;
        self.head = src.head.clone();
    }
}

// Source: src/core/SkAAClip.cpp:1266-1285
impl SkAAClip {
    pub fn set_empty(&mut self) -> bool {
        self.head = None;
        self.bounds.set_empty();
        false
    }
    pub fn set_rect(&mut self, bounds: SkIRect) -> bool {
        if bounds.is_empty() {
            return self.set_empty();
        }
        #[cfg(sk_debug)]
        let _validate = unsafe { AutoAAClipValidate::new(self as *const _) };
        self.head = None;
        self.bounds = bounds;
        self.head = Some(RunHead::alloc_rect(&bounds));
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1283; @condition b"!this->isEmpty()\0".as_ptr(); !self.is_empty());
        true
    }
}

// Source: src/core/SkAAClip.cpp:1287-1313
impl SkAAClip {
    pub fn is_rect(&self) -> bool {
        let Some(head) = &self.head else {
            return false;
        };
        if head.offsets().len() != 1 {
            return false;
        }
        let yoff = head.offsets()[0];
        if yoff.y != self.bounds.fBottom - 1 {
            return false;
        }
        let row = &head.data()[yoff.offset as usize..];
        let mut width = self.bounds.width();
        let mut p = 0;
        loop {
            if row[p + 1] != 255 {
                return false;
            }
            let n = i32::from(row[p]);
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1308; @condition b"n <= width\0".as_ptr(); n <= width);
            width -= n;
            p += 2;
            if width <= 0 {
                break;
            }
        }
        true
    }
}

// Source: src/core/SkAAClip.cpp:1315-1401
impl SkAAClip {
    pub fn set_region(&mut self, region: &SkRegion) -> bool {
        if region.is_empty() {
            return self.set_empty();
        }
        if region.is_rect() {
            return self.set_rect(*region.get_bounds());
        }
        let bounds = *region.get_bounds();
        let (ox, oy) = (bounds.fLeft, bounds.fTop);
        let mut ys = Vec::with_capacity(bounds.height().min(1024) as usize);
        let mut xs = Vec::with_capacity(bounds.width().min(512) as usize * 128);
        fn append(xs: &mut Vec<u8>, value: u8, mut count: i32) {
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1335; @condition b"count >= 0\0".as_ptr(); count >= 0);
            while count > 0 {
                let n = count.min(255);
                xs.extend_from_slice(&[n as u8, value]);
                count -= n;
            }
        }
        let mut iter = RegionIterator::new(region);
        let (mut prev_right, mut prev_bot) = (0, 0);
        let mut has_curr_y = false;
        while !iter.done() {
            let r = iter.rect();
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1355; @condition b"bounds.contains(r)\0".as_ptr(); bounds.contains_rect(r));
            let bot = r.fBottom - oy;
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1358; @condition b"bot >= prevBot\0".as_ptr(); bot >= prev_bot);
            if bot > prev_bot {
                if has_curr_y {
                    append(&mut xs, 0, bounds.width() - prev_right);
                }
                let top = r.fTop - oy;
                if top > prev_bot {
                    ys.push(YOffset {
                        y: top - 1,
                        offset: xs.len() as u32,
                    });
                    append(&mut xs, 0, bounds.width());
                }
                ys.push(YOffset {
                    y: bot - 1,
                    offset: xs.len() as u32,
                });
                has_curr_y = true;
                prev_right = 0;
                prev_bot = bot;
            }
            let x = r.fLeft - ox;
            append(&mut xs, 0, x - prev_right);
            let w = r.fRight - r.fLeft;
            append(&mut xs, 255, w);
            prev_right = x + w;
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1386; @condition b"prevRight <= bounds.width()\0".as_ptr(); prev_right <= bounds.width());
            iter.next();
        }
        append(&mut xs, 0, bounds.width() - prev_right);
        let head = RunHead::from_slices(&ys, &xs);
        self.set_empty();
        self.bounds = bounds;
        self.head = Some(head);
        self.validate();
        true
    }
}

// Source: src/core/SkAAClip.cpp:1403-1430
impl SkAAClip {
    pub fn set_path(
        &mut self,
        path: &crate::src::core::SkPath::SkPath,
        clip: SkIRect,
        do_aa: bool,
    ) -> bool {
        #[cfg(sk_debug)]
        let _validate = unsafe { AutoAAClipValidate::new(self as *const _) };
        if clip.is_empty() {
            return self.set_empty();
        }
        let bounds = if path.isInverseFillType() {
            clip
        } else {
            let mut bounds = path.getBounds().roundOut();
            if bounds.is_empty64() || !bounds.intersect(&clip) {
                return self.set_empty();
            }
            bounds
        };
        let mut builder = Builder::new(bounds);
        builder.blit_path(self, path, do_aa)
    }
}

// Source: src/core/SkAAClip.cpp:1432-1463
impl SkAAClip {
    pub fn op(&mut self, other: &Self, op: crate::include::core::SkClipOp::SkClipOp) -> bool {
        #[cfg(sk_debug)]
        let _validate = unsafe { AutoAAClipValidate::new(self as *const _) };
        use crate::include::core::SkClipOp::SkClipOp;
        if self.is_empty() {
            return false;
        }
        let mut bounds = self.bounds;
        match op {
            SkClipOp::kDifference => {
                if other.is_empty() || !SkIRect::intersects(&self.bounds, &other.bounds) {
                    return true;
                }
            }
            SkClipOp::kIntersect => {
                if other.is_empty() || !bounds.intersect(&other.bounds) {
                    return self.set_empty();
                }
            }
        }
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1458; @condition b"SkIRect::Intersects(bounds, fBounds)\0".as_ptr(); SkIRect::intersects(&bounds, &self.bounds));
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1459; @condition b"SkIRect::Intersects(bounds, other.fBounds)\0".as_ptr(); SkIRect::intersects(&bounds, &other.bounds));
        let mut builder = Builder::new(bounds);
        let result = builder.apply_clip_op(self, other, op);
        result
    }
}

// Source: src/core/SkAAClip.cpp:1465-1492
impl SkAAClip {
    pub fn op_rect(&mut self, rect: SkIRect, op: crate::include::core::SkClipOp::SkClipOp) -> bool {
        use crate::include::core::SkClipOp::SkClipOp;
        let mut bounds = self.bounds;
        if !bounds.intersect(&rect) {
            match op {
                SkClipOp::kDifference => !self.is_empty(),
                SkClipOp::kIntersect => self.set_empty(),
            }
        } else if bounds == self.bounds {
            match op {
                SkClipOp::kDifference => self.set_empty(),
                SkClipOp::kIntersect => !self.is_empty(),
            }
        } else if op == SkClipOp::kIntersect
            && self.quick_contains(bounds.fLeft, bounds.fTop, bounds.fRight, bounds.fBottom)
        {
            self.set_rect(bounds)
        } else {
            let mut clip = Self::new();
            clip.set_rect(rect);
            self.op(&clip, op)
        }
    }
}

// Source: src/core/SkAAClip.cpp:1494-1526
impl SkAAClip {
    pub fn op_float_rect(
        &mut self,
        rect: &crate::src::core::SkRect::SkRect,
        op: crate::include::core::SkClipOp::SkClipOp,
        do_aa: bool,
    ) -> bool {
        use crate::include::core::SkClipOp::SkClipOp;
        use crate::include::core::SkPathTypes::{SkPathDirection, SkPathFillType};
        use crate::src::core::SkPath::SkPath;
        use crate::src::core::SkRect::SkRect;
        if !do_aa {
            return self.op_rect(rect.round(), op);
        }
        let mut pixel_bounds = self.bounds;
        if !pixel_bounds.intersect(&rect.roundOut()) {
            return match op {
                SkClipOp::kDifference => !self.is_empty(),
                SkClipOp::kIntersect => self.set_empty(),
            };
        } else if rect.contains(&SkRect::Make(&self.bounds)) {
            return match op {
                SkClipOp::kDifference => self.set_empty(),
                SkClipOp::kIntersect => !self.is_empty(),
            };
        } else if op == SkClipOp::kIntersect
            && self.quick_contains(
                pixel_bounds.fLeft,
                pixel_bounds.fTop,
                pixel_bounds.fRight,
                pixel_bounds.fBottom,
            )
        {
            return self.set_path(
                &SkPath::Rect(rect, SkPathFillType::kDefault, SkPathDirection::kCW, 0),
                pixel_bounds,
                true,
            );
        } else {
            let mut rect_clip = SkAAClip::new();
            rect_clip.set_path(
                &SkPath::Rect(rect, SkPathFillType::kDefault, SkPathDirection::kCW, 0),
                if op == SkClipOp::kDifference {
                    self.bounds
                } else {
                    pixel_bounds
                },
                true,
            );
            self.op(&rect_clip, op)
        }
    }
}

// Source: src/core/SkAAClip.cpp:1530-1547
impl SkAAClip {
    pub fn translate_to(&self, dx: i32, dy: i32, dst: Option<&mut Self>) -> bool {
        let Some(dst) = dst else {
            return !self.is_empty();
        };
        if self.is_empty() {
            return dst.set_empty();
        }
        let head = self.head.clone();
        dst.head = None;
        dst.head = head;
        dst.bounds = self.bounds;
        dst.bounds.offset(dx, dy);
        true
    }
    pub fn translate(&mut self, dx: i32, dy: i32) -> bool {
        if self.is_empty() {
            return self.set_empty();
        }
        self.bounds.offset(dx, dy);
        true
    }
}

// Source: src/core/SkAAClip.cpp:1549-1556
// The null-owner guard is supplied by Option<RunHead> drop; the native decrement
// and last-owner free are literal. Distributed owner/drop representation is adapted.
impl Drop for RunHead {
    fn drop(&mut self) {
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1551; @condition b"fRunHead->fRefCnt.load() >= 1\0".as_ptr(); self.ref_count().load(Ordering::SeqCst) > 0);
        if self.ref_count().fetch_sub(1, Ordering::SeqCst) == 1 {
            unsafe {
                aa_free(self.pointer.as_ptr().cast());
            }
        }
    }
}
// Source: src/core/SkAAClip.cpp:1558-1576
impl SkAAClip {
    pub fn find_row(&self, mut y: i32, last_y: Option<&mut i32>) -> Option<&[u8]> {
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1559; @condition b"fRunHead\0".as_ptr(); self.head.is_some());
        if y < self.bounds.fTop || y >= self.bounds.fBottom {
            return None;
        }
        y -= self.bounds.fTop;
        let head = self.head.as_ref().unwrap();
        let mut i = 0;
        while head.offsets()[i].y < y {
            i += 1;
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1569; @condition b"yoff - fRunHead->yoffsets() < fRunHead->fRowCount\0".as_ptr(); i < head.offsets().len());
        }
        let yoff = head.offsets()[i];
        if let Some(last) = last_y {
            *last = self.bounds.fTop + yoff.y;
        }
        Some(&head.data()[yoff.offset as usize..])
    }
}

// Source: src/core/SkAAClip.cpp:1578-1595
impl SkAAClip {
    pub fn find_x<'a>(
        &self,
        mut data: &'a [u8],
        mut x: i32,
        initial: Option<&mut i32>,
    ) -> &'a [u8] {
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1579; @condition b"x >= fBounds.fLeft && x < fBounds.fRight\0".as_ptr(); x >= self.bounds.fLeft && x < self.bounds.fRight);
        x -= self.bounds.fLeft;
        loop {
            let n = i32::from(data[0]);
            if x < n {
                if let Some(initial) = initial {
                    *initial = n - x;
                }
                break;
            }
            data = &data[2..];
            x -= n;
        }
        data
    }
}

// Source: src/core/SkAAClip.cpp:1597-1624
impl SkAAClip {
    pub fn quick_contains(&self, left: i32, top: i32, right: i32, bottom: i32) -> bool {
        if self.is_empty()
            || !self
                .bounds
                .contains_rect(&SkIRect::make_ltrb(left, top, right, bottom))
        {
            return false;
        }
        let mut last_y = 0;
        let row = self.find_row(top, Some(&mut last_y)).unwrap();
        if last_y < bottom {
            return false;
        }
        let mut count = 0;
        let mut row = self.find_x(row, left, Some(&mut count));
        let mut width = right - left;
        while row[1] == 255 {
            if count >= width {
                return true;
            }
            width -= count;
            row = &row[2..];
            count = i32::from(row[0]);
        }
        false
    }
}

// Source: src/core/SkAAClip.cpp:1628-1653
fn expand_to_runs(mut data: &[u8], initial: i32, mut width: i32, runs: &mut [i16], aa: &mut [u8]) {
    let mut n = initial;
    let mut p = 0;
    loop {
        if n > width {
            n = width;
        }
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1637; @condition b"n > 0\0".as_ptr(); n > 0);
        runs[p] = n as i16;
        aa[p] = data[1];
        p += n as usize;
        data = &data[2..];
        width -= n;
        if width == 0 {
            break;
        }
        n = i32::from(data[0]);
    }
    runs[p] = 0;
}

// Source: src/core/SkAAClip.cpp:1655-1657
impl Drop for SkAAClipBlitter<'_> {
    fn drop(&mut self) {
        // Derived destructor frees the shared scanline first, then the gray
        // scratch member; the SkBlitter base storage is destroyed afterward.
        unsafe {
            aa_free(self.scanline_scratch.cast());
        }
        drop(core::mem::replace(
            &mut self.gray_mask_scratch,
            crate::src::core::SkAutoMalloc::SkAutoSMalloc::new(),
        ));
    }
}
// Source: src/core/SkAAClip.cpp:1659-1669
// Native single malloc block and pointer offsets. Extra zero initialization is a
// Rust adaptation: the canonical blitter trait borrows typed sparse-run slices.
impl SkAAClipBlitter<'_> {
    fn ensure_runs_and_aa(&mut self) {
        Self::ensure_runs_and_aa_parts(
            self.bounds,
            &mut self.scanline_scratch,
            &mut self.runs,
            &mut self.aa,
        );
    }
    // Split fields preserve an already borrowed inline gray allocation's provenance.
    // Native allocation order is unchanged; only the Rust borrow projection differs.
    fn ensure_runs_and_aa_parts(
        bounds: SkIRect,
        scanline_scratch: &mut *mut u8,
        runs: &mut *mut i16,
        aa: &mut *mut u8,
    ) {
        if scanline_scratch.is_null() {
            let count = (bounds.width() + 1) as usize;
            let bytes = count
                .checked_mul(core::mem::size_of::<u32>())
                .expect("AA scanline allocation overflow");
            let allocation = unsafe { aa_malloc(bytes) }.cast::<u8>();
            if allocation.is_null() {
                std::alloc::handle_alloc_error(
                    std::alloc::Layout::from_size_align(bytes, core::mem::align_of::<u32>())
                        .unwrap(),
                );
            }
            // Adapted: initializing holes avoids references to uninitialized i16/u8.
            unsafe {
                allocation.write_bytes(0, bytes);
            }
            *scanline_scratch = allocation;
            *runs = allocation.cast();
            *aa = unsafe { (*runs).add(count).cast() };
        }
    }
}

// Source: src/core/SkAAClip.cpp:1671-1695
impl SkAAClipBlitter<'_> {
    unsafe fn blit_h_clipped(&mut self, x: i32, y: i32, width: i32) {
        unsafe {
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1672; @condition b"width > 0\0".as_ptr(); width > 0);
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1673; @condition b"fAAClipBounds.contains(x, y)\0".as_ptr(); self.bounds.contains_xy(x, y));
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1674; @condition b"fAAClipBounds.contains(x + width - 1, y)\0".as_ptr(); self.bounds.contains_xy(x + width - 1, y));
            let clip = self.clip.unwrap();
            let mut initial = 0;
            let row = clip.find_x(clip.find_row(y, None).unwrap(), x, Some(&mut initial));
            if initial >= width {
                let alpha = row[1];
                if alpha == 0 {
                    return;
                }
                if alpha == 255 {
                    self.blitter.as_mut().unwrap().as_mut().blitH(x, y, width);
                    return;
                }
            }
            self.ensure_runs_and_aa();
            let (runs, aa) = scanline_runs_and_aa(self.runs, self.aa);
            expand_to_runs(row, initial, width, runs, aa);
            self.blitter
                .as_mut()
                .unwrap()
                .as_mut()
                .blitAntiH(x, y, aa, runs);
        }
    }
}

// Source: src/core/SkAAClip.cpp:1697-1739
fn merge(
    mut row: &[u8],
    mut row_n: i32,
    src_aa: &[u8],
    src_runs: &[i16],
    dst_aa: &mut [u8],
    dst_runs: &mut [i16],
    width: i32,
) {
    #[cfg(sk_debug)]
    let mut accumulated = 0;
    let (mut src, mut dst) = (0, 0);
    let mut src_n = i32::from(src_runs[0]);
    if src_n == 0 {
        return;
    }
    loop {
        {
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1711; @condition b"rowN > 0\0".as_ptr(); row_n > 0);
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1712; @condition b"srcN > 0\0".as_ptr(); src_n > 0);
        };
        let alpha = crate::include::private::SkMath::SkMulDiv255Round(
            u32::from(src_aa[src]),
            u32::from(row[1]),
        );
        let n = src_n.min(row_n);
        dst_runs[dst] = n as i16;
        dst_aa[dst] = alpha as u8;
        dst += n as usize;
        src_n -= n;
        if src_n == 0 {
            let original_n = src_runs[src] as usize;
            src += original_n;
            src_n = i32::from(src_runs[src]);
            if src_n == 0 {
                break;
            }
        }
        row_n -= n;
        if row_n == 0 {
            row = &row[2..];
            row_n = i32::from(row[0]);
        }
        #[cfg(sk_debug)]
        {
            accumulated += n;
        }
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1736; @condition b"accumulated <= width\0".as_ptr(); accumulated <= width);
    }
    dst_runs[dst] = 0;
}

// Source: src/core/SkAAClip.cpp:1741-1752
impl SkAAClipBlitter<'_> {
    unsafe fn blit_anti_h_clipped(&mut self, x: i32, y: i32, aa: &mut [u8], runs: &mut [i16]) {
        unsafe {
            let clip = self.clip.unwrap();
            let mut initial = 0;
            let row = clip.find_x(clip.find_row(y, None).unwrap(), x, Some(&mut initial));
            self.ensure_runs_and_aa();
            let (dst_runs, dst_aa) = scanline_runs_and_aa(self.runs, self.aa);
            merge(
                row,
                initial,
                aa,
                runs,
                dst_aa,
                dst_runs,
                self.bounds.width(),
            );
            self.blitter
                .as_mut()
                .unwrap()
                .as_mut()
                .blitAntiH(x, y, dst_aa, dst_runs);
        }
    }
}

// Source: src/core/SkAAClip.cpp:1754-1780
impl SkAAClipBlitter<'_> {
    unsafe fn blit_v_clipped(&mut self, x: i32, mut y: i32, mut height: i32, alpha: u8) {
        unsafe {
            let clip = self.clip.unwrap();
            if clip.quick_contains(x, y, x + 1, y + height) {
                self.blitter
                    .as_mut()
                    .unwrap()
                    .as_mut()
                    .blitV(x, y, height, alpha);
                return;
            }
            loop {
                let mut last = 0;
                let row = clip.find_row(y, Some(&mut last)).unwrap();
                let mut dy = last - y + 1;
                if dy > height {
                    dy = height;
                }
                height -= dy;
                let row = clip.find_x(row, x, None);
                let new_alpha = crate::include::private::SkMath::SkMulDiv255Round(
                    u32::from(alpha),
                    u32::from(row[1]),
                ) as u8;
                if new_alpha != 0 {
                    self.blitter
                        .as_mut()
                        .unwrap()
                        .as_mut()
                        .blitV(x, y, dy, new_alpha);
                }
                crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1774; @condition b"height >= 0\0".as_ptr(); height >= 0);
                if height <= 0 {
                    break;
                }
                y = last + 1;
            }
        }
    }
}

// Source: src/core/SkAAClip.cpp:1782-1792
impl SkAAClipBlitter<'_> {
    unsafe fn blit_rect_clipped(&mut self, x: i32, mut y: i32, width: i32, mut height: i32) {
        unsafe {
            if self
                .clip
                .unwrap()
                .quick_contains(x, y, x + width, y + height)
            {
                self.blitter
                    .as_mut()
                    .unwrap()
                    .as_mut()
                    .blitRect(x, y, width, height);
                return;
            }
            loop {
                height -= 1;
                if height < 0 {
                    break;
                }
                unsafe { core::pin::Pin::new_unchecked(&mut *self) }.blitH(x, y, width);
                y += 1;
            }
        }
    }
}

// Dynamic dispatch adaptation; algorithm bodies above retain original order.
// Safety: each blitter owns SkBlitterStorage for its full lifetime; allocation
// pointers remain valid through the next reset or the owner's drop.
unsafe impl crate::src::core::SkBlitter::SkBlitter for SkAAClipBlitter<'_> {
    fn allocBlitMemory(self: core::pin::Pin<&mut Self>, sz: usize) -> *mut u8 {
        let this = unsafe { self.get_unchecked_mut() };
        this.storage.allocBlitMemory(sz)
    }
    unsafe fn blitMask(
        self: core::pin::Pin<&mut Self>,
        mask: &crate::src::core::SkMask::SkMask,
        clip: &SkIRect,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.blit_mask(mask, clip);
        }
    }
    unsafe fn blitH(self: core::pin::Pin<&mut Self>, x: i32, y: i32, width: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { SkAAClipBlitter::blit_h_clipped(this, x, y, width) }
    }
    unsafe fn blitAntiH(
        self: core::pin::Pin<&mut Self>,
        x: i32,
        y: i32,
        aa: &mut [u8],
        runs: &mut [i16],
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { SkAAClipBlitter::blit_anti_h_clipped(this, x, y, aa, runs) }
    }
    unsafe fn blitV(self: core::pin::Pin<&mut Self>, x: i32, y: i32, height: i32, alpha: u8) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { SkAAClipBlitter::blit_v_clipped(this, x, y, height, alpha) }
    }
    unsafe fn blitRect(self: core::pin::Pin<&mut Self>, x: i32, y: i32, width: i32, height: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { SkAAClipBlitter::blit_rect_clipped(this, x, y, width, height) }
    }
}

// Source: src/core/SkAAClip.cpp:1794-1803
type MergeAAProc = unsafe fn(*const u8, i32, &[u8], i32, *mut u8);
unsafe fn small_memcpy(dst: *mut u8, src: *const u8, n: usize) {
    unsafe {
        core::ptr::copy_nonoverlapping(src, dst, n);
    }
}
unsafe fn small_bzero(dst: *mut u8, n: usize) {
    unsafe {
        core::ptr::write_bytes(dst, 0, n);
    }
}

// Source: src/core/SkAAClip.cpp:1805-1816
// Trait specialization replaces C++ mergeOne overload resolution.
trait MaskPixel: Copy {
    fn merge_one(self, alpha: u32) -> Self;
}
impl MaskPixel for u8 {
    fn merge_one(self, alpha: u32) -> Self {
        crate::include::private::SkMath::SkMulDiv255Round(u32::from(self), alpha) as u8
    }
}
impl MaskPixel for u16 {
    fn merge_one(self, alpha: u32) -> Self {
        use crate::include::private::SkMath::SkMulDiv255Round;
        use crate::src::core::SkColorData::{
            SkGetPackedB16, SkGetPackedG16, SkGetPackedR16, SkPackRGB16,
        };
        let value = u32::from(self);
        let r = SkGetPackedR16(value);
        let g = SkGetPackedG16(value);
        let b = SkGetPackedB16(value);
        SkPackRGB16(
            SkMulDiv255Round(r, alpha),
            SkMulDiv255Round(g, alpha),
            SkMulDiv255Round(b, alpha),
        )
    }
}

// Source: src/core/SkAAClip.cpp:1818-1849
// Safety: source/destination must be aligned for T and contain src_n pixels;
// their nonoverlap and row encoding are the upstream mask/blitter contract.
unsafe fn merge_t<T: MaskPixel>(
    src: *const u8,
    mut src_n: i32,
    mut row: &[u8],
    mut row_n: i32,
    dst: *mut u8,
) {
    let mut src = src.cast::<T>();
    let mut dst = dst.cast::<T>();
    loop {
        {
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1823; @condition b"rowN > 0\0".as_ptr(); row_n > 0);
            crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1824; @condition b"srcN > 0\0".as_ptr(); src_n > 0);
        };
        let n = row_n.min(src_n);
        let alpha = u32::from(row[1]);
        if alpha == 255 {
            unsafe {
                small_memcpy(
                    dst.cast(),
                    src.cast(),
                    n as usize * core::mem::size_of::<T>(),
                );
            }
        } else if alpha == 0 {
            unsafe {
                small_bzero(dst.cast(), n as usize * core::mem::size_of::<T>());
            }
        } else {
            for i in 0..n as usize {
                unsafe {
                    *dst.add(i) = (*src.add(i)).merge_one(alpha);
                }
            }
        }
        src_n -= n;
        if src_n == 0 {
            break;
        }
        src = unsafe { src.add(n as usize) };
        dst = unsafe { dst.add(n as usize) };
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1845; @condition b"rowN == n\0".as_ptr(); (row_n) == (n));
        row = &row[2..];
        row_n = i32::from(row[0]);
    }
}

// Source: src/core/SkAAClip.cpp:1851-1865
fn find_merge_aa_proc(format: crate::src::core::SkMask::Format) -> Option<MergeAAProc> {
    use crate::src::core::SkMask::Format;
    match format {
        Format::kBW_Format => {
            crate::SkDEBUGFAIL!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1854; b"unsupported\0".as_ptr());
            None
        }
        Format::kA8_Format | Format::k3D_Format => Some(merge_t::<u8>),
        Format::kLCD16_Format => Some(merge_t::<u16>),
        _ => {
            crate::SkDEBUGFAIL!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1862; b"unsupported\0".as_ptr());
            None
        }
    }
}

// Source: src/core/SkAAClip.cpp:1867-1872
fn bit2byte(bit: i32) -> u32 {
    crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1868; @condition b"bitInAByte <= 0xFF\0".as_ptr(); bit <= 255);
    (-bit >> 8) as u32
}

// Source: src/core/SkAAClip.cpp:1874-1913
// Safety: descriptors must address readable BW source and writable A8 target
// images for their rowBytes/height. The output ownership remains external.
unsafe fn upscale_bw_to_a8(
    dst: &crate::src::core::SkMask::SkMask,
    src: &crate::src::core::SkMask::SkMask,
) {
    use crate::src::core::SkMask::Format;
    crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1875; @condition b"SkMask::kBW_Format == srcMask.fFormat\0".as_ptr(); (src.fFormat) == ( Format::kBW_Format));
    crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1876; @condition b"SkMask::kA8_Format == dstMask->fFormat\0".as_ptr(); (dst.fFormat) == ( Format::kA8_Format));
    let (width, height) = (src.fBounds.width(), src.fBounds.height());
    let (mut s, mut d) = (src.fImage, dst.fImage.cast_mut());
    let whole = width >> 3;
    let leftover = width & 7;
    for _ in 0..height {
        let mut out = d;
        for i in 0..whole as usize {
            let byte = i32::from(unsafe { *s.add(i) });
            for bit in 0..8 {
                unsafe {
                    *out.add(bit) = bit2byte(byte & (1 << (7 - bit))) as u8;
                }
            }
            out = unsafe { out.add(8) };
        }
        if leftover != 0 {
            let mut byte = i32::from(unsafe { *s.add(whole as usize) });
            for _ in 0..leftover {
                unsafe {
                    *out = bit2byte(byte & 128) as u8;
                    out = out.add(1);
                }
                byte <<= 1;
            }
        }
        s = unsafe { s.add(src.fRowBytes as usize) };
        d = unsafe { d.add(dst.fRowBytes as usize) };
    }
}

// Source: src/core/SkAAClip.cpp:1915-1975
impl SkAAClipBlitter<'_> {
    // Safety: source mask storage satisfies SkBlitter's readable pixel contract;
    // downstream may use source's specified mutable A8 clipping behavior.
    unsafe fn blit_mask(&mut self, original: &crate::src::core::SkMask::SkMask, clip: &SkIRect) {
        use crate::src::core::SkMask::{Format, SkMaskBuilder};
        let aaclip = self.clip.unwrap();
        crate::SkASSERT!(@source b"src/core/SkAAClip.cpp\0".as_ptr(), 1916; @condition b"fAAClip->getBounds().contains(clip)\0".as_ptr(); aaclip.get_bounds().contains_rect(clip));
        if aaclip.quick_contains(clip.fLeft, clip.fTop, clip.fRight, clip.fBottom) {
            unsafe {
                self.blitter
                    .as_mut()
                    .unwrap()
                    .as_mut()
                    .blitMask(original, clip);
            }
            return;
        }
        let mut gray = SkMaskBuilder::new();
        let mask = if original.fFormat == Format::kBW_Format {
            *gray.format() = Format::kA8_Format;
            *gray.bounds() = original.fBounds;
            *gray.rowBytes() = original.fBounds.width() as u32;
            let size = gray.computeImageSize();
            // Obtain the inline address only after self is borrowed in its final
            // position. It is consumed within this call and never stored in self.
            *gray.image() = self.gray_mask_scratch.reset(
                size,
                crate::src::core::SkAutoMalloc::OnShrink::kAlloc_OnShrink,
                None,
            );
            unsafe {
                upscale_bw_to_a8(&gray.mask, original);
            }
            &gray.mask
        } else {
            original
        };
        // Do not reborrow all of self while mask may point into gray_mask_scratch.
        Self::ensure_runs_and_aa_parts(
            self.bounds,
            &mut self.scanline_scratch,
            &mut self.runs,
            &mut self.aa,
        );
        let mut src = mask.getAddr(clip.fLeft, clip.fTop).cast::<u8>();
        let source_row_bytes = mask.fRowBytes as usize;
        let width = clip.width();
        let merge_proc = find_merge_aa_proc(mask.fFormat).expect("unsupported mask format");
        let mut row_mask = SkMaskBuilder::new();
        *row_mask.format() = if mask.fFormat == Format::k3D_Format {
            Format::kA8_Format
        } else {
            mask.fFormat
        };
        row_mask.bounds().fLeft = clip.fLeft;
        row_mask.bounds().fRight = clip.fRight;
        *row_mask.rowBytes() = mask.fRowBytes;
        *row_mask.image() = self.scanline_scratch;
        let mut y = clip.fTop;
        let stop_y = y + clip.height();
        loop {
            let mut local_stop = 0;
            let row = aaclip.find_row(y, Some(&mut local_stop)).unwrap();
            local_stop = (local_stop + 1).min(stop_y);
            let mut initial = 0;
            let row = aaclip.find_x(row, clip.fLeft, Some(&mut initial));
            loop {
                unsafe {
                    merge_proc(src, width, row, initial, *row_mask.image());
                }
                row_mask.bounds().fTop = y;
                row_mask.bounds().fBottom = y + 1;
                unsafe {
                    self.blitter
                        .as_mut()
                        .unwrap()
                        .as_mut()
                        .blitMask(&row_mask.mask, &row_mask.fBounds);
                    src = src.add(source_row_bytes);
                }
                y += 1;
                if y >= local_stop {
                    break;
                }
            }
            if y >= stop_y {
                break;
            }
        }
    }
}

// Typed view of the source ensureRunsAndAA layout: runs[count], aa[count],
// followed by count unused bytes, all within count 32-bit scanline words.
/// # Safety
/// Both pointers are native views of one live exclusively leased scanline block,
/// aa is exactly count i16 entries after runs, and all count*4 bytes initialized.
unsafe fn scanline_runs_and_aa<'a>(runs: *mut i16, aa: *mut u8) -> (&'a mut [i16], &'a mut [u8]) {
    let count = unsafe { aa.offset_from(runs.cast()) as usize / 2 };
    unsafe {
        (
            core::slice::from_raw_parts_mut(runs, count),
            core::slice::from_raw_parts_mut(aa, count),
        )
    }
}

// Source: src/core/SkAAClip.cpp:1241-1245
impl Clone for SkAAClip {
    fn clone(&self) -> Self {
        Self::from_clip(self)
    }
}
