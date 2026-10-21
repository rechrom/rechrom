use crate::src::core::SkRect::SkIRect;
use crate::src::core::SkRegionPriv::{
    value_is_sentinel, RunHead, RunHeadHandle, RUN_TYPE_SENTINEL as SENTINEL,
};

// Source: include/core/SkRegion.h:148-167
impl SkRegion {
    pub fn is_empty(&self) -> bool {
        matches!(self.runs, RegionRuns::Empty)
    }
    pub fn is_rect(&self) -> bool {
        matches!(self.runs, RegionRuns::Rect)
    }
    pub fn is_complex(&self) -> bool {
        !self.is_empty() && !self.is_rect()
    }
    pub fn get_bounds(&self) -> &SkIRect {
        &self.bounds
    }
    // Rust ownership accessor; called only after upstream complex-state checks.
    fn head(&self) -> &RunHeadHandle {
        match &self.runs {
            RegionRuns::Complex(h) => h,
            _ => unreachable!("complex region required"),
        }
    }
}

// Source: include/core/SkRegion.h:322-330
impl SkRegion {
    pub fn quick_contains(&self, r: &SkIRect) -> bool {
        debug_assert_eq!(self.is_empty(), self.bounds.is_empty());
        r.fLeft < r.fRight
            && r.fTop < r.fBottom
            && self.is_rect()
            && self.bounds.fLeft <= r.fLeft
            && self.bounds.fTop <= r.fTop
            && self.bounds.fRight >= r.fRight
            && self.bounds.fBottom >= r.fBottom
    }
}

// Source: include/core/SkRegion.h:339-342
impl SkRegion {
    pub fn quick_reject_rect(&self, r: &SkIRect) -> bool {
        self.is_empty() || r.is_empty() || !SkIRect::intersects(&self.bounds, r)
    }
}

// Source: include/core/SkRegion.h:351-354
impl SkRegion {
    pub fn quick_reject_region(&self, r: &Self) -> bool {
        self.is_empty() || r.is_empty() || !SkIRect::intersects(&self.bounds, &r.bounds)
    }
}

// Source: include/core/SkRegion.h:378-388
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Op {
    Difference,
    Intersect,
    Union,
    Xor,
    ReverseDifference,
    Replace,
}

// Source: include/core/SkRegion.h:396-404
impl SkRegion {
    pub fn op_rect(&mut self, rect: SkIRect, op: Op) -> bool {
        if self.is_rect() && op == Op::Intersect {
            if !self.bounds.intersect(&rect) {
                return self.set_empty();
            }
            return true;
        }
        // The original supports aliasing this as input/output; retain a cheap
        // shared snapshot so Rust can borrow output exclusively.
        let input = self.clone();
        self.op_region_rect(&input, rect, op)
    }
}

// Source: include/core/SkRegion.h:412-412
impl SkRegion {
    pub fn op_region(&mut self, r: &Self, op: Op) -> bool {
        let input = self.clone();
        self.op(&input, r, op)
    }
}

// Source: include/core/SkRegion.h:643-656
// Source: src/core/SkRegion.cpp:54-55
// Empty/Rect encode the two native pointer sentinels; Complex retains the native
// atomic shared run allocation through a Rust owning pointer handle.
#[derive(Clone, Debug)]
pub(crate) enum RegionRuns {
    Empty,
    Rect,
    Complex(RunHeadHandle),
}
#[derive(Clone, Debug)]
pub struct SkRegion {
    pub(crate) bounds: SkIRect,
    pub(crate) runs: RegionRuns,
}
impl Default for SkRegion {
    fn default() -> Self {
        Self::new()
    }
}

// Source: src/core/SkRegion.cpp:58-93
// The native 256-element inline buffer remains uninitialized. Heap growth uses
// realloc and copies inline bytes only on the first transition. Access helpers
// form typed references only to runs already written by the sweep algorithm.
struct RunArray {
    stack: [core::mem::MaybeUninit<i32>; 256],
    heap: *mut core::mem::MaybeUninit<i32>,
    count: usize,
}
unsafe extern "C" {
    #[link_name = "realloc"]
    fn region_realloc(pointer: *mut core::ffi::c_void, size: usize) -> *mut core::ffi::c_void;
    #[link_name = "free"]
    fn region_free(pointer: *mut core::ffi::c_void);
}
impl Drop for RunArray {
    fn drop(&mut self) {
        unsafe {
            region_free(self.heap.cast());
        }
    }
}
impl RunArray {
    fn new() -> Self {
        Self {
            stack: [core::mem::MaybeUninit::uninit(); 256],
            heap: core::ptr::null_mut(),
            count: 256,
        }
    }
    fn pointer(&self) -> *const core::mem::MaybeUninit<i32> {
        if self.heap.is_null() {
            self.stack.as_ptr()
        } else {
            self.heap
        }
    }
    fn pointer_mut(&mut self) -> *mut core::mem::MaybeUninit<i32> {
        if self.heap.is_null() {
            self.stack.as_mut_ptr()
        } else {
            self.heap
        }
    }
    fn read(&self, index: usize) -> i32 {
        crate::SkASSERT!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 73; @condition b"(unsigned)i < (unsigned)fCount\0".as_ptr(); index < self.count);
        unsafe { (*self.pointer().add(index)).assume_init() }
    }
    fn write(&mut self, index: usize, value: i32) {
        crate::SkASSERT!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 73; @condition b"(unsigned)i < (unsigned)fCount\0".as_ptr(); index < self.count);
        unsafe {
            self.pointer_mut()
                .add(index)
                .write(core::mem::MaybeUninit::new(value));
        }
    }
    fn written_range(&self, range: core::ops::Range<usize>) -> &[i32] {
        assert!(range.end <= self.count);
        unsafe { core::slice::from_raw_parts(self.pointer().add(range.start).cast(), range.len()) }
    }
    fn written_prefix_mut(&mut self, count: usize) -> &mut [i32] {
        assert!(count <= self.count);
        unsafe { core::slice::from_raw_parts_mut(self.pointer_mut().cast(), count) }
    }
    fn resize_to_at_least(&mut self, count: usize) {
        if count > self.count {
            let new_count = count
                .checked_add(count >> 1)
                .filter(|&x| x <= i32::MAX as usize)
                .unwrap_or(i32::MAX as usize);
            let bytes = new_count
                .checked_mul(core::mem::size_of::<i32>())
                .expect("Region allocation overflow");
            let allocation: *mut core::mem::MaybeUninit<i32> =
                unsafe { region_realloc(self.heap.cast(), bytes) }.cast();
            if allocation.is_null() {
                std::alloc::handle_alloc_error(
                    std::alloc::Layout::from_size_align(bytes, core::mem::align_of::<i32>())
                        .unwrap(),
                );
            }
            if self.heap.is_null() {
                unsafe {
                    core::ptr::copy_nonoverlapping(self.stack.as_ptr(), allocation, self.count);
                }
            }
            self.heap = allocation;
            self.count = new_count;
        }
    }
}

// Source: src/core/SkRegion.cpp:102-115
// A slice index replaces the native pointer while preserving the preceding count word.
fn skip_intervals(runs: &[i32], mut first_interval: usize) -> usize {
    let intervals = runs[first_interval - 1];
    #[cfg(sk_debug)]
    {
        if intervals > 0 {
            crate::SkASSERT!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 106; @condition b"runs[0] < runs[1]\0".as_ptr(); runs[first_interval] < runs[first_interval + 1]);
            crate::SkASSERT!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 107; @condition b"runs[1] < SkRegion_kRunTypeSentinel\0".as_ptr(); runs[first_interval + 1] < SENTINEL);
        } else {
            crate::SkASSERT!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 109; @condition b"0 == intervals\0".as_ptr(); (intervals) == (0));
            crate::SkASSERT!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 110; @condition b"SkRegion_kRunTypeSentinel == runs[0]\0".as_ptr(); (runs[first_interval]) == ( SENTINEL));
        }
    }
    first_interval += (intervals * 2 + 1) as usize;
    first_interval
}

// Source: src/core/SkRegion.cpp:117-119
fn assert_sentinel(value: i32, sentinel: bool) {
    crate::SkASSERT!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 118; @condition b"SkRegionValueIsSentinel(value) == isSentinel\0".as_ptr(); (value_is_sentinel(value)) == ( sentinel));
}

// Source: src/core/SkRegion.cpp:121-141
impl SkRegion {
    pub fn runs_are_a_rect(runs: &[i32], bounds: &mut SkIRect) -> bool {
        assert_sentinel(runs[0], false);
        crate::SkASSERT!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 124; @condition b"count >= kRectRegionRuns\0".as_ptr(); runs.len() >= 7);
        if runs.len() == 7 {
            assert_sentinel(runs[1], false);
            crate::SkASSERT!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 128; @condition b"1 == runs[2]\0".as_ptr(); (runs[2]) == (1));
            assert_sentinel(runs[3], false);
            assert_sentinel(runs[4], false);
            assert_sentinel(runs[5], true);
            assert_sentinel(runs[6], true);
            crate::SkASSERT!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 134; @condition b"runs[0] < runs[1]\0".as_ptr(); runs[0] < runs[1]);
            crate::SkASSERT!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 135; @condition b"runs[3] < runs[4]\0".as_ptr(); runs[3] < runs[4]);
            bounds.set_ltrb(runs[3], runs[0], runs[4], runs[1]);
            return true;
        }
        false
    }
}

// Source: src/core/SkRegion.cpp:145-158
impl SkRegion {
    pub fn new() -> Self {
        Self {
            bounds: SkIRect::make_empty(),
            runs: RegionRuns::Empty,
        }
    }
    pub fn from_region(src: &Self) -> Self {
        src.clone()
    }
    pub fn from_rect(rect: SkIRect) -> Self {
        let mut r = Self::new();
        r.set_rect(rect);
        r
    }
}

// Source: src/core/SkRegion.cpp:160-190
impl SkRegion {
    // Assignment/drop releases the intrusive run reference; copying increments
    // the same native atomic count. Allocation retains nullable results.
    fn allocate_runs(&mut self, count: i32) {
        self.runs = RunHead::alloc(count)
            .map(RegionRuns::Complex)
            .unwrap_or(RegionRuns::Rect);
    }
    fn allocate_runs_with_counts(&mut self, count: i32, y: i32, intervals: i32) {
        self.runs = RunHead::alloc_with_counts(count, y, intervals)
            .map(RegionRuns::Complex)
            .unwrap_or(RegionRuns::Rect);
    }
    fn allocate_runs_like(&mut self, head: &RunHeadHandle) {
        self.allocate_runs_with_counts(head.run_count, head.y_span_count(), head.interval_count());
    }
    pub fn assign(&mut self, src: &Self) {
        self.set_region(src);
    }
}

// Source: src/core/SkRegion.cpp:192-196
impl SkRegion {
    pub fn swap(&mut self, other: &mut Self) {
        std::mem::swap(self, other);
    }
}

// Source: src/core/SkRegion.cpp:198-205
impl SkRegion {
    pub fn compute_region_complexity(&self) -> i32 {
        if self.is_empty() {
            0
        } else if self.is_rect() {
            1
        } else {
            self.head().interval_count()
        }
    }
}

// Source: src/core/SkRegion.cpp:207-224
impl SkRegion {
    pub fn set_empty(&mut self) -> bool {
        self.runs = RegionRuns::Empty;
        self.bounds.set_empty();
        false
    }
    pub fn set_rect(&mut self, rect: SkIRect) -> bool {
        if rect.is_empty() || rect.fRight == SENTINEL || rect.fBottom == SENTINEL {
            return self.set_empty();
        }
        self.runs = RegionRuns::Empty;
        self.bounds = rect;
        self.runs = RegionRuns::Rect;
        true
    }
}

// Source: src/core/SkRegion.cpp:226-237
impl SkRegion {
    pub fn set_region(&mut self, src: &Self) -> bool {
        self.runs = RegionRuns::Empty;
        self.bounds = src.bounds;
        self.runs = src.runs.clone();
        !self.is_empty()
    }
}

// Source: src/core/SkRegion.cpp:239-249
impl SkRegion {
    pub fn op_rect_region(&mut self, rect: SkIRect, region: &Self, op: Op) -> bool {
        let tmp = Self::from_rect(rect);
        self.op(&tmp, region, op)
    }
    pub fn op_region_rect(&mut self, region: &Self, rect: SkIRect, op: Op) -> bool {
        let tmp = Self::from_rect(rect);
        self.op(region, &tmp, op)
    }
}

// Source: src/core/SkRegion.cpp:255-278
// Native Android-framework diagnostic. The C allocation and snprintf formatting
// retain the native returned-buffer contract; the caller frees the result with free.
// Exposing the translated method does not enable Android-framework integration.
impl SkRegion {
    pub fn toString(&self) -> *mut core::ffi::c_char {
        unsafe extern "C" {
            #[link_name = "malloc"]
            fn region_string_malloc(size: usize) -> *mut core::ffi::c_void;
            #[link_name = "snprintf"]
            fn region_string_snprintf(
                dst: *mut core::ffi::c_char,
                size: usize,
                format: *const core::ffi::c_char,
                ...
            ) -> core::ffi::c_int;
        }
        let mut iter = RegionIterator::new(self);
        let mut count: i32 = 0;
        while !iter.done() {
            count += 1;
            iter.next();
        }
        let max = (count * ((11 * 4) + 5)) + 11 + 1;
        // sk_malloc_throw terminates on allocation failure before the native null guard.
        let allocation = unsafe { region_string_malloc(max as usize) };
        if allocation.is_null() {
            std::alloc::handle_alloc_error(
                std::alloc::Layout::from_size_align(max as usize, 1).unwrap(),
            );
        }
        let result = allocation.cast::<core::ffi::c_char>();
        if result.is_null() {
            return core::ptr::null_mut();
        }
        count = unsafe { region_string_snprintf(result, max as usize, c"SkRegion(".as_ptr()) };
        iter.reset(self);
        while !iter.done() {
            let r = iter.rect();
            count += unsafe {
                region_string_snprintf(
                    result.offset(count as isize),
                    (max - count) as usize,
                    c"(%d,%d,%d,%d)".as_ptr(),
                    r.fLeft,
                    r.fTop,
                    r.fRight,
                    r.fBottom,
                )
            };
            iter.next();
        }
        count += unsafe {
            region_string_snprintf(
                result.offset(count as isize),
                (max - count) as usize,
                c")".as_ptr(),
            )
        };
        let _ = count;
        result
    }
}

// Source: src/core/SkRegion.cpp:283-295
impl SkRegion {
    pub fn count_runtype_values(&self, top: &mut i32, bottom: &mut i32) -> i32 {
        let count = if self.is_rect() {
            2
        } else {
            crate::SkASSERT!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 289; @condition b"this->isComplex()\0".as_ptr(); self.is_complex());
            self.head().interval_count() * 2
        };
        *top = self.bounds.fTop;
        *bottom = self.bounds.fBottom;
        count
    }
}

// Source: src/core/SkRegion.cpp:297-373
impl SkRegion {
    pub fn set_runs(&mut self, runs: &mut [i32]) -> bool {
        #[cfg(sk_debug)]
        self.validate();
        crate::SkASSERT!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 303; @condition b"count > 0\0".as_ptr(); !runs.is_empty());
        let mut start = 0;
        let mut stop = runs.len();
        if stop <= 2 {
            assert_sentinel(runs[stop - 1], true);
            return self.set_empty();
        }
        if stop > 7 {
            assert_sentinel(runs[0], false);
            assert_sentinel(runs[1], false);
            if runs[3] == SENTINEL {
                start = 3;
                runs[start] = runs[start - 2];
                for offset in 1..=4 {
                    assert_sentinel(runs[start + offset], false);
                }
            }
            assert_sentinel(runs[stop - 1], true);
            assert_sentinel(runs[stop - 2], true);
            if runs[stop - 5] == SENTINEL {
                runs[stop - 4] = SENTINEL;
                stop -= 3;
                assert_sentinel(runs[stop - 1], true);
                assert_sentinel(runs[stop - 2], true);
                for offset in 3..=6 {
                    assert_sentinel(runs[stop - offset], false);
                }
            }
        }
        let count = stop - start;
        crate::SkASSERT!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 345; @condition b"count >= kRectRegionRuns\0".as_ptr(); count >= 7);
        if Self::runs_are_a_rect(&runs[start..stop], &mut self.bounds) {
            return self.set_rect(self.bounds);
        }
        if !self.is_complex() || self.head().run_count as usize != count {
            self.runs = RegionRuns::Empty;
            self.allocate_runs(count as i32);
        }
        let RegionRuns::Complex(head) = &mut self.runs else {
            unreachable!("complex runs allocated");
        };
        let head = RunHead::ensure_writable(head);
        head.copy_runs_from(&runs[start..stop]);
        head.compute_run_bounds(&mut self.bounds);
        if self.bounds.is_empty() {
            return self.set_empty();
        }
        true
    }
}

// Source: src/core/SkRegion.cpp:375-384
impl SkRegion {
    pub fn build_rect_runs(bounds: &SkIRect) -> [i32; 7] {
        [
            bounds.fTop,
            bounds.fBottom,
            1,
            bounds.fLeft,
            bounds.fRight,
            SENTINEL,
            SENTINEL,
        ]
    }
}

// Source: src/core/SkRegion.cpp:386-418
impl SkRegion {
    pub fn contains_xy(&self, x: i32, y: i32) -> bool {
        #[cfg(sk_debug)]
        self.validate();
        if !self.bounds.contains_xy(x, y) {
            return false;
        }
        if self.is_rect() {
            return true;
        }
        crate::SkASSERT!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 395; @condition b"this->isComplex()\0".as_ptr(); self.is_complex());
        let mut runs = &self.head().find_scanline(y)[2..];
        loop {
            if x < runs[0] {
                break;
            }
            if x < runs[1] {
                return true;
            }
            runs = &runs[2..];
        }
        false
    }
}

// Source: src/core/SkRegion.cpp:420-442
fn scanline_bottom(runs: &[i32]) -> i32 {
    runs[0]
}
fn scanline_next(runs: &[i32]) -> &[i32] {
    &runs[2 + runs[1] as usize * 2 + 1..]
}
fn scanline_contains(mut runs: &[i32], left: i32, right: i32) -> bool {
    runs = &runs[2..];
    loop {
        if left < runs[0] {
            break;
        }
        if right <= runs[1] {
            return true;
        }
        runs = &runs[2..];
    }
    false
}

// Source: src/core/SkRegion.cpp:444-466
impl SkRegion {
    pub fn contains_rect(&self, rect: &SkIRect) -> bool {
        #[cfg(sk_debug)]
        self.validate();
        if !self.bounds.contains_rect(rect) {
            return false;
        }
        if self.is_rect() {
            return true;
        }
        crate::SkASSERT!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 453; @condition b"this->isComplex()\0".as_ptr(); self.is_complex());
        let mut scanline = self.head().find_scanline(rect.fTop);
        loop {
            if !scanline_contains(scanline, rect.fLeft, rect.fRight) {
                return false;
            }
            if rect.fBottom <= scanline_bottom(scanline) {
                break;
            }
            scanline = scanline_next(scanline);
        }
        true
    }
}

// Source: src/core/SkRegion.cpp:468-506
impl SkRegion {
    pub fn contains_region(&self, other: &Self) -> bool {
        if self.is_empty() || other.is_empty() || !self.bounds.contains_rect(&other.bounds) {
            return false;
        }
        if self.is_rect() {
            return true;
        }
        if other.is_rect() {
            return self.contains_rect(other.get_bounds());
        }
        !Self::oper(other, self, Op::Difference, None)
    }
    pub fn get_runs<'a>(&'a self, storage: &'a mut [i32; 7]) -> (&'a [i32], i32) {
        if self.is_empty() {
            storage[0] = SENTINEL;
            (&storage[..1], 0)
        } else if self.is_rect() {
            *storage = Self::build_rect_runs(&self.bounds);
            (storage, 1)
        } else {
            (self.head().readonly_runs(), self.head().interval_count())
        }
    }
}

// Source: src/core/SkRegion.cpp:509-524
fn scanline_intersects(mut runs: &[i32], left: i32, right: i32) -> bool {
    runs = &runs[2..];
    loop {
        if right <= runs[0] {
            break;
        }
        if left < runs[1] {
            return true;
        }
        runs = &runs[2..];
    }
    false
}

// Source: src/core/SkRegion.cpp:525-577
impl SkRegion {
    pub fn intersects_rect(&self, rect: &SkIRect) -> bool {
        if self.is_empty() || rect.is_empty() {
            return false;
        }
        let mut sect = SkIRect::make_empty();
        if !sect.intersect_ab(self.bounds, *rect) {
            return false;
        }
        if self.is_rect() {
            return true;
        }
        crate::SkASSERT!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 538; @condition b"this->isComplex()\0".as_ptr(); self.is_complex());
        let mut scanline = self.head().find_scanline(sect.fTop);
        loop {
            if scanline_intersects(scanline, sect.fLeft, sect.fRight) {
                return true;
            }
            if sect.fBottom <= scanline_bottom(scanline) {
                break;
            }
            scanline = scanline_next(scanline);
        }
        false
    }
    pub fn intersects_region(&self, other: &Self) -> bool {
        if self.is_empty() || other.is_empty() {
            return false;
        }
        if !SkIRect::intersects(&self.bounds, &other.bounds) {
            return false;
        }
        let (ar, br) = (self.is_rect(), other.is_rect());
        if ar && br {
            return true;
        }
        if ar {
            return other.intersects_rect(self.get_bounds());
        }
        if br {
            return self.intersects_rect(other.get_bounds());
        }
        Self::oper(self, other, Op::Intersect, None)
    }
}

// Source: src/core/SkRegion.cpp:581-606
impl PartialEq for SkRegion {
    fn eq(&self, other: &Self) -> bool {
        if std::ptr::eq(self, other) {
            return true;
        }
        if self.bounds != other.bounds {
            return false;
        }
        match (&self.runs, &other.runs) {
            (RegionRuns::Empty, RegionRuns::Empty) | (RegionRuns::Rect, RegionRuns::Rect) => true,
            (RegionRuns::Complex(a), RegionRuns::Complex(b)) => {
                RunHeadHandle::ptr_eq(a, b)
                    || (a.run_count == b.run_count && a.readonly_runs() == b.readonly_runs())
            }
            _ => false,
        }
    }
}

// Source: src/core/SkRegion.cpp:608-616
fn pin_offset_s32(min: i32, max: i32, mut offset: i32) -> i32 {
    crate::SkASSERT!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 610; @condition b"min <= max\0".as_ptr(); min <= max);
    if i64::from(min) + i64::from(offset) < i64::from(i32::MIN) {
        offset = i32::MIN.wrapping_sub(min);
    }
    if i64::from(max) + i64::from(offset) > i64::from(i32::MAX) {
        offset = i32::MAX.wrapping_sub(max);
    }
    offset
}

// Source: src/core/SkRegion.cpp:618-675
impl SkRegion {
    // Rust separates the aliasing and nonaliasing entry points; both share the
    // original run cursor traversal, and the aliasing path retains the native run copy-on-write transition.
    pub fn translate_to(&self, dx: i32, dy: i32, dst: Option<&mut Self>) {
        let Some(dst) = dst else {
            return;
        };
        if self.is_empty() {
            dst.set_empty();
            return;
        }
        let dx = pin_offset_s32(self.bounds.fLeft, self.bounds.fRight, dx);
        let dy = pin_offset_s32(self.bounds.fTop, self.bounds.fBottom, dy);
        if self.is_rect() {
            dst.set_rect(self.bounds.make_offset(dx, dy));
            return;
        }
        let mut tmp = Self::new();
        tmp.allocate_runs_like(self.head());
        tmp.bounds = self.bounds;
        dst.swap(&mut tmp);
        drop(tmp);
        dst.bounds.offset(dx, dy);
        let RegionRuns::Complex(head) = &mut dst.runs else {
            unreachable!()
        };
        let head = RunHead::ensure_writable(head);
        translate_copy_runs(self.head().readonly_runs(), head, dx, dy);
    }
    pub fn translate(&mut self, dx: i32, dy: i32) {
        if self.is_empty() {
            return;
        }
        let dx = pin_offset_s32(self.bounds.fLeft, self.bounds.fRight, dx);
        let dy = pin_offset_s32(self.bounds.fTop, self.bounds.fBottom, dy);
        if self.is_rect() {
            self.set_rect(self.bounds.make_offset(dx, dy));
            return;
        }
        self.bounds.offset(dx, dy);
        let RegionRuns::Complex(head) = &mut self.runs else {
            unreachable!()
        };
        translate_run_values(RunHead::ensure_writable(head).writable_runs(), dx, dy);
    }
}
fn translate_copy_runs(source: &[i32], head: &mut RunHeadHandle, dx: i32, dy: i32) {
    let (mut src, mut dst) = (0, 0);
    head.write_run(dst, source[src] + dy);
    src += 1;
    dst += 1;
    loop {
        let bottom = source[src];
        src += 1;
        if bottom == SENTINEL {
            break;
        }
        head.write_run(dst, bottom + dy);
        dst += 1;
        head.write_run(dst, source[src]);
        dst += 1;
        src += 1;
        loop {
            let x = source[src];
            src += 1;
            if x == SENTINEL {
                break;
            }
            head.write_run(dst, x + dx);
            dst += 1;
            head.write_run(dst, source[src] + dx);
            dst += 1;
            src += 1;
        }
        head.write_run(dst, SENTINEL);
        dst += 1;
    }
    head.write_run(dst, SENTINEL);
    dst += 1;
    crate::SkASSERT!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 670; @condition b"sruns - fRunHead->readonly_runs() == fRunHead->fRunCount\0".as_ptr(); (src) == ( source.len()));
    crate::SkASSERT!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 671; @condition b"druns - dst->fRunHead->readonly_runs() == dst->fRunHead->fRunCount\0".as_ptr(); (dst) == ( head.run_count as usize));
    unsafe {
        head.assume_initialized();
    }
}
fn translate_run_values(runs: &mut [i32], dx: i32, dy: i32) {
    runs[0] += dy;
    let mut cursor = 1;
    loop {
        if runs[cursor] == SENTINEL {
            cursor += 1;
            break;
        }
        runs[cursor] += dy;
        cursor += 2;
        loop {
            if runs[cursor] == SENTINEL {
                cursor += 1;
                break;
            }
            runs[cursor] += dx;
            runs[cursor + 1] += dx;
            cursor += 2;
        }
    }
    debug_assert_eq!(cursor, runs.len());
}

// Source: src/core/SkRegion.cpp:679-692
impl SkRegion {
    pub fn set_rects(&mut self, rects: &[SkIRect]) -> bool {
        if rects.is_empty() {
            return self.set_empty();
        }
        if rects.len() == 1 {
            return self.set_rect(rects[0]);
        }
        let mid = rects.len() / 2;
        let (mut left, mut right) = (Self::new(), Self::new());
        left.set_rects(&rects[..mid]);
        right.set_rects(&rects[mid..]);
        self.op(&left, &right, Op::Union)
    }
}

// Source: src/core/SkRegion.cpp:701-799
struct SpanRec<'a> {
    a_runs: &'a [i32],
    b_runs: &'a [i32],
    a_left: i32,
    a_right: i32,
    b_left: i32,
    b_right: i32,
    left: i32,
    right: i32,
    inside: i32,
}
impl<'a> SpanRec<'a> {
    fn new(a: &'a [i32], b: &'a [i32]) -> Self {
        Self {
            a_left: a[0],
            a_right: a[1],
            b_left: b[0],
            b_right: b[1],
            a_runs: &a[2..],
            b_runs: &b[2..],
            left: 0,
            right: 0,
            inside: 0,
        }
    }
    fn done(&self) -> bool {
        self.a_left == SENTINEL && self.b_left == SENTINEL
    }
    fn next(&mut self) {
        debug_assert!(self.a_left == SENTINEL || self.a_left < self.a_right);
        debug_assert!(self.b_left == SENTINEL || self.b_left < self.b_right);
        let (mut al, ar, mut bl, br) = (self.a_left, self.a_right, self.b_left, self.b_right);
        let (inside, left, mut right);
        let (mut a_flush, mut b_flush) = (false, false);
        if al < bl {
            inside = 1;
            left = al;
            if ar <= bl {
                right = ar;
                a_flush = true;
            } else {
                al = bl;
                right = al;
            }
        } else if bl < al {
            inside = 2;
            left = bl;
            if br <= al {
                right = br;
                b_flush = true;
            } else {
                bl = al;
                right = bl;
            }
        } else {
            inside = 3;
            left = al;
            // At least one comparison must hold, so initialize from the
            // first branch's value and keep the original two-branch order.
            right = ar;
            if ar <= br {
                bl = ar;
                right = bl;
                a_flush = true;
            }
            if br <= ar {
                al = br;
                right = al;
                b_flush = true;
            }
        }
        let mut ar = ar;
        let mut br = br;
        if a_flush {
            al = self.a_runs[0];
            ar = self.a_runs[1];
            self.a_runs = &self.a_runs[2..];
        }
        if b_flush {
            bl = self.b_runs[0];
            br = self.b_runs[1];
            self.b_runs = &self.b_runs[2..];
        }
        crate::SkASSERT!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 787; @condition b"left <= rite\0".as_ptr(); left <= right);
        (self.a_left, self.a_right, self.b_left, self.b_right) = (al, ar, bl, br);
        (self.left, self.right, self.inside) = (left, right, inside);
    }
}

// Source: src/core/SkRegion.cpp:801-853
// Slice starts at intervalCount, permitting safe access to the native runs[-1].
fn distance_to_sentinel(runs: &[i32]) -> usize {
    let intervals = runs[0] as usize;
    crate::SkASSERT_RELEASE!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 813; @condition b"runs[intervals * 2] == SkRegion_kRunTypeSentinel\0".as_ptr(); (runs[1 + intervals * 2]) == ( SENTINEL));
    intervals * 2
}
fn operate_on_span(
    a: &[i32],
    b: &[i32],
    array: &mut RunArray,
    dst_offset: usize,
    min: i32,
    max: i32,
) -> usize {
    array.resize_to_at_least(dst_offset + distance_to_sentinel(a) + distance_to_sentinel(b) + 2);
    let mut dst = dst_offset;
    let mut first_interval = true;
    let mut rec = SpanRec::new(&a[1..], &b[1..]);
    while !rec.done() {
        rec.next();
        let (left, right) = (rec.left, rec.right);
        if (rec.inside.wrapping_sub(min) as u32) <= (max - min) as u32 && left < right {
            if first_interval || array.read(dst - 1) < left {
                array.write(dst, left);
                array.write(dst + 1, right);
                dst += 2;
                first_interval = false;
            } else {
                array.write(dst - 1, right);
            }
        }
    }
    crate::SkASSERT!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 850; @condition b"dst < &(*array)[array->count() - 1]\0".as_ptr(); dst < array.count - 1);
    array.write(dst, SENTINEL);
    dst + 1
}

// Source: src/core/SkRegion.cpp:859-872
const OP_MIN_MAX: [(i32, i32); 4] = [(1, 1), (3, 3), (1, 3), (1, 2)];

// Source: src/core/SkRegion.cpp:874-931
struct RgnOper<'a> {
    array: &'a mut RunArray,
    min: i32,
    max: i32,
    top: i32,
    prev_dst: usize,
    prev_len: usize,
}
impl<'a> RgnOper<'a> {
    fn new(top: i32, array: &'a mut RunArray, op: Op) -> Self {
        let (min, max) = OP_MIN_MAX[op as usize];
        Self {
            array,
            min,
            max,
            top,
            prev_dst: 1,
            prev_len: 0,
        }
    }
    fn add_span(&mut self, bottom: i32, a: &[i32], b: &[i32]) {
        let start = self.prev_dst + self.prev_len + 2;
        let stop = operate_on_span(a, b, self.array, start, self.min, self.max);
        let len = stop - start;
        crate::SkASSERT!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 890; @condition b"len >= 1 && (len & 1) == 1\0".as_ptr(); len >= 1 && len & 1 == 1);
        crate::SkASSERT!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 891; @condition b"SkRegion_kRunTypeSentinel == (*fArray)[stop - 1]\0".as_ptr(); (self.array.read(stop - 1)) == ( SENTINEL));
        if self.prev_len == len
            && (len == 1
                || self
                    .array
                    .written_range(self.prev_dst..self.prev_dst + len - 1)
                    == self.array.written_range(start..start + len - 1))
        {
            self.array.write(self.prev_dst - 2, bottom);
        } else if len == 1 && self.prev_len == 0 {
            self.top = bottom;
        } else {
            self.array.write(start - 2, bottom);
            self.array.write(start - 1, (len >> 1) as i32);
            self.prev_dst = start;
            self.prev_len = len;
        }
    }
    fn flush(&mut self) -> usize {
        self.array.write(0, self.top);
        crate::SkASSERT!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 916; @condition b"fArray->count() > SkToInt(fPrevDst + fPrevLen)\0".as_ptr(); self.array.count > self.prev_dst + self.prev_len);
        self.array.write(self.prev_dst + self.prev_len, SENTINEL);
        self.prev_dst + self.prev_len + 1
    }
    fn is_empty(&self) -> bool {
        self.prev_len == 0
    }
}

// Source: src/core/SkRegion.cpp:933-1044
fn operate(a: &[i32], b: &[i32], dst: &mut RunArray, op: Op, quick_exit: bool) -> i32 {
    // Count followed by sentinel and dummy right coordinate.
    let empty = [0, SENTINEL, 0];
    let (mut at, mut ab, mut bt, mut bb) = (a[0], a[1], b[0], b[1]);
    // Index addresses intervalCount instead of the first interval.
    let (mut ai, mut bi) = (2, 2);
    for v in [at, ab, bt, bb] {
        assert_sentinel(v, false);
    }
    let mut oper = RgnOper::new(at.min(bt), dst, op);
    let mut prev_bottom = SENTINEL;
    while ab < SENTINEL || bb < SENTINEL {
        let (top, mut bottom);
        let (mut run0, mut run1) = (&empty[..], &empty[..]);
        let (mut af, mut bf) = (false, false);
        if at < bt {
            top = at;
            run0 = &a[ai..];
            if ab <= bt {
                bottom = ab;
                af = true;
            } else {
                at = bt;
                bottom = at;
            }
        } else if bt < at {
            top = bt;
            run1 = &b[bi..];
            if bb <= at {
                bottom = bb;
                bf = true;
            } else {
                bt = at;
                bottom = bt;
            }
        } else {
            top = at;
            run0 = &a[ai..];
            run1 = &b[bi..];
            bottom = ab;
            if ab <= bb {
                bt = ab;
                bottom = bt;
                af = true;
            }
            if bb <= ab {
                at = bb;
                bottom = at;
                bf = true;
            }
        }
        if top > prev_bottom {
            oper.add_span(top, &empty, &empty);
        }
        oper.add_span(bottom, run0, run1);
        if quick_exit && !oper.is_empty() {
            return -1;
        }
        if af {
            ai = skip_intervals(a, ai + 1);
            at = ab;
            ab = a[ai];
            ai += 1;
            if ab == SENTINEL {
                at = ab;
            }
        }
        if bf {
            bi = skip_intervals(b, bi + 1);
            bt = bb;
            bb = b[bi];
            bi += 1;
            if bb == SENTINEL {
                bt = bb;
            }
        }
        prev_bottom = bottom;
    }
    oper.flush() as i32
}

// Source: src/core/SkRegion.cpp:1063-1073
fn set_empty_check(result: Option<&mut SkRegion>) -> bool {
    result.is_some_and(SkRegion::set_empty)
}
fn set_rect_check(result: Option<&mut SkRegion>, rect: SkIRect) -> bool {
    match result {
        Some(r) => r.set_rect(rect),
        None => !rect.is_empty(),
    }
}
fn set_region_check(result: Option<&mut SkRegion>, region: &SkRegion) -> bool {
    match result {
        Some(r) => r.set_region(region),
        None => !region.is_empty(),
    }
}

// Source: src/core/SkRegion.cpp:1075-1178
impl SkRegion {
    pub fn oper<'a>(
        mut a: &'a Self,
        mut b: &'a Self,
        mut op: Op,
        result: Option<&mut Self>,
    ) -> bool {
        if op == Op::Replace {
            return set_region_check(result, b);
        }
        if op == Op::ReverseDifference {
            std::mem::swap(&mut a, &mut b);
            op = Op::Difference;
        }
        let mut bounds = SkIRect::make_empty();
        let (ae, be, ar, br) = (a.is_empty(), b.is_empty(), a.is_rect(), b.is_rect());
        match op {
            Op::Difference => {
                if ae {
                    return set_empty_check(result);
                }
                if be || !SkIRect::intersects(&a.bounds, &b.bounds) {
                    return set_region_check(result, a);
                }
                if br && b.bounds.contains_no_empty_check(&a.bounds) {
                    return set_empty_check(result);
                }
            }
            Op::Intersect => {
                if (ae | be) || !bounds.intersect_ab(a.bounds, b.bounds) {
                    return set_empty_check(result);
                }
                if ar & br {
                    return set_rect_check(result, bounds);
                }
                if ar && a.bounds.contains_rect(&b.bounds) {
                    return set_region_check(result, b);
                }
                if br && b.bounds.contains_rect(&a.bounds) {
                    return set_region_check(result, a);
                }
            }
            Op::Union => {
                if ae {
                    return set_region_check(result, b);
                }
                if be {
                    return set_region_check(result, a);
                }
                if ar && a.bounds.contains_rect(&b.bounds) {
                    return set_region_check(result, a);
                }
                if br && b.bounds.contains_rect(&a.bounds) {
                    return set_region_check(result, b);
                }
            }
            Op::Xor => {
                if ae {
                    return set_region_check(result, b);
                }
                if be {
                    return set_region_check(result, a);
                }
            }
            _ => {
                crate::SkDEBUGFAIL!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 1152; b"unknown region op\0".as_ptr());
                return false;
            }
        }
        let (mut ta, mut tb) = ([0; 7], [0; 7]);
        let (a_runs, _) = a.get_runs(&mut ta);
        let (b_runs, _) = b.get_runs(&mut tb);
        let mut array = RunArray::new();
        let count = operate(a_runs, b_runs, &mut array, op, result.is_none());
        crate::SkASSERT!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 1165; @condition b"count <= array.count()\0".as_ptr(); count <= array.count as i32);
        if let Some(result) = result {
            crate::SkASSERT!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 1168; @condition b"count >= 0\0".as_ptr(); count >= 0);
            result.set_runs(array.written_prefix_mut(count as usize))
        } else {
            count == -1 || count > 2
        }
    }
    pub fn op(&mut self, a: &Self, b: &Self, op: Op) -> bool {
        Self::oper(a, b, op, Some(self))
    }
}

// Source: src/core/SkRegion.cpp:1182-1213
impl SkRegion {
    /// Safety: storage is null for sizing, or writable for the returned byte count.
    pub unsafe fn writeToMemory(&self, storage: *mut u8) -> usize {
        if storage.is_null() {
            let mut size = 4;
            if !self.is_empty() {
                size += core::mem::size_of_val(&self.bounds);
                if self.is_complex() {
                    size += 8;
                    size += self.head().run_count as usize * 4;
                }
            }
            return size;
        }
        let mut buffer = unsafe { crate::src::core::SkBuffer::SkWBuffer::new(storage) };
        if self.is_empty() {
            unsafe { buffer.write32(-1) }
        } else {
            let rect = self.is_rect();
            unsafe {
                buffer.write32(if rect { 0 } else { self.head().run_count });
                buffer.write(
                    (&self.bounds as *const SkIRect).cast(),
                    core::mem::size_of::<SkIRect>(),
                );
            }
            if !rect {
                let head = self.head();
                unsafe {
                    buffer.write32(head.y_span_count());
                    buffer.write32(head.interval_count());
                    buffer.write(
                        head.readonly_runs().as_ptr().cast(),
                        head.run_count as usize * 4,
                    );
                }
            }
        }
        buffer.pos()
    }
}

// Source: src/core/SkRegion.cpp:1215-1230
fn validate_run_count(y: i32, intervals: i32, count: i32) -> bool {
    if y < 1 || intervals < 2 {
        return false;
    }
    2i32.checked_add(y)
        .and_then(|s| s.checked_add(y))
        .and_then(|s| s.checked_add(y))
        .and_then(|s| s.checked_add(intervals))
        .and_then(|s| s.checked_add(intervals))
        == Some(count)
}

// Source: src/core/SkRegion.cpp:1232-1320
fn validate_run_view(
    count: usize,
    get: impl Fn(usize) -> Option<i32>,
    given_bounds: SkIRect,
    mut y_count: i32,
    mut interval_count: i32,
) -> bool {
    if !validate_run_count(y_count, interval_count, count as i32) {
        return false;
    }
    if get(count - 1).expect("native checked run bound") != SENTINEL
        || get(count - 2).expect("native checked run bound") != SENTINEL
    {
        return false;
    }
    let mut bounds = SkIRect::make_empty();
    let mut rect = SkIRect::make_empty();
    let mut prev_empty = true;
    rect.fTop = get(0).expect("native checked run bound");
    let mut cursor = 1;
    if rect.fTop == SENTINEL || rect.fTop != given_bounds.fTop {
        return false;
    }
    loop {
        y_count -= 1;
        if y_count < 0 {
            return false;
        }
        let Some(bottom) = get(cursor) else {
            return false;
        };
        rect.fBottom = bottom;
        cursor += 1;
        if bottom == SENTINEL || bottom > given_bounds.fBottom || bottom <= rect.fTop {
            return false;
        }
        let Some(n) = get(cursor) else {
            return false;
        };
        cursor += 1;
        if n < 0 || n > interval_count || cursor + 1 + 2 * n as usize > count {
            return false;
        }
        if n == 0 {
            if prev_empty {
                return false;
            }
            prev_empty = true;
        } else {
            prev_empty = false;
        }
        interval_count -= n;
        let mut first = true;
        let mut last_right = 0;
        for _ in 0..n {
            rect.fLeft = get(cursor).expect("native checked run bound");
            rect.fRight = get(cursor + 1).expect("native checked run bound");
            cursor += 2;
            if rect.fLeft == SENTINEL
                || rect.fRight == SENTINEL
                || rect.fLeft >= rect.fRight
                || (!first && rect.fLeft <= last_right)
            {
                return false;
            }
            last_right = rect.fRight;
            first = false;
            bounds.join(&rect);
        }
        if get(cursor).expect("native checked run bound") != SENTINEL {
            return false;
        }
        cursor += 1;
        rect.fTop = rect.fBottom;
        let Some(next) = get(cursor) else {
            return false;
        };
        if next == SENTINEL {
            break;
        }
    }
    cursor += 1;
    if y_count != 0 || interval_count != 0 || given_bounds != bounds {
        return false;
    }
    crate::SkASSERT!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 1318; @condition b"runs == end\0".as_ptr(); (cursor) == ( count));
    true
}

// Same native run validator, with a borrowed aligned slice for existing in-memory owners.
fn validate_run(runs: &[i32], given_bounds: SkIRect, y_count: i32, interval_count: i32) -> bool {
    validate_run_view(
        runs.len(),
        |index| runs.get(index).copied(),
        given_bounds,
        y_count,
        interval_count,
    )
}

// Source: src/core/SkRegion.cpp:1321-1364
impl SkRegion {
    /// Safety: storage covers length readable bytes; no output view escapes the call.
    pub unsafe fn readFromMemory(&mut self, storage: *const u8, length: usize) -> usize {
        let mut buffer = unsafe { crate::src::core::SkBuffer::SkRBuffer::new(storage, length) };
        let mut tmp = SkRegion::new();
        let mut count = 0;
        if !buffer.readS32(&mut count) || count < -1 {
            return 0;
        }
        if count >= 0 {
            if !unsafe {
                buffer.read(
                    (&mut tmp.bounds as *mut SkIRect).cast(),
                    core::mem::size_of::<SkIRect>(),
                )
            } || tmp.bounds.is_empty()
            {
                return 0;
            }
            if count == 0 {
                tmp.runs = RegionRuns::Rect;
            } else {
                let mut y_count = 0;
                let mut interval_count = 0;
                if !buffer.readS32(&mut y_count)
                    || !buffer.readS32(&mut interval_count)
                    || buffer.available() < (count as usize) * 4
                {
                    return 0;
                }
                let runs = storage.wrapping_add(buffer.pos()).cast::<i32>();
                // Validate directly from wire bytes, before allocating a run owner.
                // Unaligned integer loads replace C++ typed pointer reads.
                if !validate_run_view(
                    count as usize,
                    |i| {
                        if i < count as usize {
                            Some(unsafe { runs.add(i).read_unaligned() })
                        } else {
                            None
                        }
                    },
                    tmp.bounds,
                    y_count,
                    interval_count,
                ) {
                    return 0;
                }
                tmp.allocate_runs_with_counts(count, y_count, interval_count);
                crate::SkASSERT!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 1354; @condition b"tmp.isComplex()\0".as_ptr(); tmp.is_complex());
                let RegionRuns::Complex(head) = &mut tmp.runs else {
                    unreachable!("native validated complex run count");
                };
                let source = buffer.skip(count as usize * 4).cast::<i32>();
                debug_assert!(!source.is_null());
                for i in 0..count as usize {
                    head.write_run(i, unsafe { source.add(i).read_unaligned() });
                }
                unsafe {
                    head.assume_initialized();
                }
            }
        }
        crate::SkASSERT!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 1358; @condition b"tmp.isValid()\0".as_ptr(); tmp.is_valid());
        crate::SkASSERT!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 1359; @condition b"buffer.isValid()\0".as_ptr(); buffer.isValid());
        self.swap(&mut tmp);
        buffer.pos()
    }
}

// Source: src/core/SkRegion.cpp:1366-1379
impl SkRegion {
    pub fn is_valid(&self) -> bool {
        if self.is_empty() {
            return self.bounds == SkIRect::make_empty();
        }
        if self.bounds.is_empty() {
            return false;
        }
        if self.is_rect() {
            return true;
        }
        let h = self.head();
        validate_run(
            h.readonly_runs(),
            self.bounds,
            h.y_span_count(),
            h.interval_count(),
        )
    }
}

// Source: src/core/SkRegion.cpp:1381-1398
impl SkRegion {
    pub fn validate(&self) {
        crate::SkASSERT!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 1382; @condition b"rgn.isValid()\0".as_ptr(); self.is_valid());
    }
    #[cfg(sk_debug)]
    pub fn dump(&self) {
        if self.is_empty() {
            unsafe {
                crate::SkDebugf!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 1386; b"  rgn: empty\n\0".as_ptr());
            }
        } else {
            unsafe {
                crate::SkDebugf!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 1388; b"  rgn: [%d %d %d %d]\0".as_ptr(),
                self.bounds.fLeft as core::ffi::c_int,self.bounds.fTop as core::ffi::c_int,
                self.bounds.fRight as core::ffi::c_int,self.bounds.fBottom as core::ffi::c_int);
            }
            if self.is_complex() {
                for run in self.head().readonly_runs() {
                    unsafe {
                        crate::SkDebugf!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 1392; b" %d\0".as_ptr(),*run as core::ffi::c_int);
                    }
                }
            }
            unsafe {
                crate::SkDebugf!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 1394; b"\n\0".as_ptr());
            }
        }
    }
}

// Source: include/core/SkRegion.h:469-469
// Rust initializes inactive cursor storage as None rather than leaving raw pointers indeterminate.
impl Default for RegionIterator<'_> {
    fn default() -> Self {
        Self {
            region: None,
            runs: None,
            rect: SkIRect::make_empty(),
            done: true,
        }
    }
}
// Source: src/core/SkRegion.cpp:1402-1470
pub struct RegionIterator<'a> {
    region: Option<&'a SkRegion>,
    runs: Option<&'a [i32]>,
    rect: SkIRect,
    done: bool,
}
impl<'a> RegionIterator<'a> {
    pub fn new(region: &'a SkRegion) -> Self {
        let mut result = Self {
            region: None,
            runs: None,
            rect: SkIRect::make_empty(),
            done: true,
        };
        result.reset(region);
        result
    }
    pub fn rewind(&mut self) -> bool {
        if let Some(region) = self.region {
            self.reset(region);
            true
        } else {
            false
        }
    }
    pub fn reset(&mut self, region: &'a SkRegion) {
        self.region = Some(region);
        if region.is_empty() {
            self.done = true;
        } else {
            self.done = false;
            if region.is_rect() {
                self.rect = region.bounds;
                self.runs = None;
            } else {
                let runs = region.head().readonly_runs();
                self.rect.set_ltrb(runs[3], runs[0], runs[4], runs[1]);
                self.runs = Some(&runs[5..]);
            }
        }
    }
    pub fn next(&mut self) {
        if self.done {
            return;
        }
        let Some(mut runs) = self.runs else {
            self.done = true;
            return;
        };
        if runs[0] < SENTINEL {
            self.rect.fLeft = runs[0];
            self.rect.fRight = runs[1];
            runs = &runs[2..];
        } else {
            runs = &runs[1..];
            if runs[0] < SENTINEL {
                if runs[1] == 0 {
                    self.rect.fTop = runs[0];
                    runs = &runs[3..];
                } else {
                    self.rect.fTop = self.rect.fBottom;
                }
                self.rect.fBottom = runs[0];
                assert_sentinel(runs[2], false);
                assert_sentinel(runs[3], false);
                self.rect.fLeft = runs[2];
                self.rect.fRight = runs[3];
                runs = &runs[4..];
            } else {
                self.done = true;
            }
        }
        self.runs = Some(runs);
    }
    // Source: include/core/SkRegion.h:501-501
    pub fn done(&self) -> bool {
        self.done
    }
    // Source: include/core/SkRegion.h:517-517
    pub fn rect(&self) -> &SkIRect {
        &self.rect
    }
    // Source: include/core/SkRegion.h:523-523
    pub fn region(&self) -> Option<&'a SkRegion> {
        self.region
    }
}

// Source: src/core/SkRegion.cpp:1472-1507
pub struct Cliperator<'a> {
    iter: RegionIterator<'a>,
    clip: SkIRect,
    rect: SkIRect,
    done: bool,
}
impl<'a> Cliperator<'a> {
    pub fn new(region: &'a SkRegion, clip: SkIRect) -> Self {
        let mut result = Self {
            iter: RegionIterator::new(region),
            clip,
            rect: SkIRect::make_empty(),
            done: true,
        };
        while !result.iter.done() {
            let r = *result.iter.rect();
            if r.fTop >= clip.fBottom {
                break;
            }
            if result.rect.intersect_ab(clip, r) {
                result.done = false;
                break;
            }
            result.iter.next();
        }
        result
    }
    pub fn next(&mut self) {
        if self.done {
            return;
        }
        self.done = true;
        self.iter.next();
        while !self.iter.done() {
            let r = *self.iter.rect();
            if r.fTop >= self.clip.fBottom {
                break;
            }
            if self.rect.intersect_ab(self.clip, r) {
                self.done = false;
                break;
            }
            self.iter.next();
        }
    }
    // Source: include/core/SkRegion.h:554-554
    pub fn done(&self) -> bool {
        self.done
    }
    // Source: include/core/SkRegion.h:568-568
    pub fn rect(&self) -> &SkIRect {
        &self.rect
    }
}

// Source: src/core/SkRegion.cpp:1511-1588
pub struct Spanerator<'a> {
    runs: Option<&'a [i32]>,
    left: i32,
    right: i32,
    done: bool,
}
impl<'a> Spanerator<'a> {
    pub fn new(region: &'a SkRegion, y: i32, mut left: i32, mut right: i32) -> Self {
        region.validate();
        let r = region.get_bounds();
        let mut result = Self {
            runs: None,
            left: 0,
            right: 0,
            done: true,
        };
        if !region.is_empty() && y >= r.fTop && y < r.fBottom && right > r.fLeft && left < r.fRight
        {
            if region.is_rect() {
                if left < r.fLeft {
                    left = r.fLeft;
                }
                if right > r.fRight {
                    right = r.fRight;
                }
                result.left = left;
                result.right = right;
                result.done = false;
            } else {
                let mut runs = &region.head().find_scanline(y)[2..];
                loop {
                    if runs[0] >= right {
                        break;
                    }
                    if runs[1] <= left {
                        runs = &runs[2..];
                        continue;
                    }
                    result.runs = Some(runs);
                    result.left = left;
                    result.right = right;
                    result.done = false;
                    break;
                }
            }
        }
        result
    }
    pub fn next(&mut self, left: Option<&mut i32>, right: Option<&mut i32>) -> bool {
        if self.done {
            return false;
        }
        let Some(runs) = self.runs else {
            self.done = true;
            if let Some(left) = left {
                *left = self.left;
            }
            if let Some(right) = right {
                *right = self.right;
            }
            return true;
        };
        if runs[0] >= self.right {
            self.done = true;
            return false;
        }
        crate::SkASSERT!(@source b"src/core/SkRegion.cpp\0".as_ptr(), 1578; @condition b"runs[1] > fLeft\0".as_ptr(); runs[1] > self.left);
        if let Some(left) = left {
            *left = self.left.max(runs[0]);
        }
        if let Some(right) = right {
            *right = self.right.min(runs[1]);
        }
        self.runs = Some(&runs[2..]);
        true
    }
}

// Source: src/core/SkRegion.cpp:1592-1631
impl SkRegion {
    pub fn visit_spans(&self, mut visitor: impl FnMut(&SkIRect)) {
        if self.is_empty() {
            return;
        }
        if self.is_rect() {
            visitor(self.get_bounds());
        } else {
            let runs = self.head().readonly_runs();
            let (mut top, mut bottom, mut p) = (runs[0], runs[1], 2);
            loop {
                let pairs = runs[p];
                p += 1;
                if pairs == 1 {
                    visitor(&SkIRect::make_ltrb(runs[p], top, runs[p + 1], bottom));
                    p += 2;
                } else if pairs > 1 {
                    for y in top..bottom {
                        for pair in 0..pairs as usize {
                            visitor(&SkIRect::make_ltrb(
                                runs[p + pair * 2],
                                y,
                                runs[p + pair * 2 + 1],
                                y + 1,
                            ));
                        }
                    }
                    p += pairs as usize * 2;
                }
                assert_sentinel(runs[p], true);
                p += 1;
                top = bottom;
                bottom = runs[p];
                p += 1;
                if value_is_sentinel(bottom) {
                    break;
                }
            }
        }
    }
}
