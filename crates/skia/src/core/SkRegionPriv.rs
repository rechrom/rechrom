use crate::src::core::SkRect::SkIRect;
use std::alloc::{handle_alloc_error, Layout};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicI32, Ordering};

// Source: src/core/SkRegionPriv.h:18-33
pub struct SkRegionPriv;
impl SkRegionPriv {
    pub const kRunTypeSentinel: i32 = RUN_TYPE_SENTINEL;
    pub fn VisitSpans(
        region: &crate::src::core::SkRegion::SkRegion,
        visitor: impl FnMut(&SkIRect),
    ) {
        region.visit_spans(visitor);
    }
    #[cfg(sk_debug)]
    pub fn Validate(region: &crate::src::core::SkRegion::SkRegion) {
        region.validate();
    }
}
pub type RunType = i32;

// Source: src/core/SkRegionPriv.h:35-39
pub const RUN_TYPE_SENTINEL: i32 = 0x7fff_ffff;
pub fn value_is_sentinel(value: i32) -> bool {
    value == RUN_TYPE_SENTINEL
}

// Source: src/core/SkRegionPriv.h:41-54
#[cfg(sk_debug)]
fn compute_interval_count(runs: &[i32]) -> usize {
    let mut cursor = 0;
    while runs[cursor] < RUN_TYPE_SENTINEL {
        crate::SkASSERT!(@source b"src/core/SkRegionPriv.h\0".as_ptr(), 48; @condition b"curr[0] < curr[1]\0".as_ptr(); runs[cursor] < runs[cursor + 1]);
        crate::SkASSERT!(@source b"src/core/SkRegionPriv.h\0".as_ptr(), 49; @condition b"curr[1] < SkRegion_kRunTypeSentinel\0".as_ptr(); runs[cursor + 1] < RUN_TYPE_SENTINEL);
        cursor += 2;
    }
    cursor / 2
}

// Source: src/core/SkRegionPriv.h:56-80
// Native four-i32 header followed immediately by count i32 run values, in one
// allocation. The owning handle only supplies Rust drop/borrow discipline.
#[repr(C)]
#[derive(Debug)]
pub struct RunHead {
    ref_count: AtomicI32,
    pub run_count: i32,
    // Source: src/core/SkRegionPriv.h:254-255
    y_span_count: i32,
    interval_count: i32,
}
pub struct RunHeadHandle {
    pointer: NonNull<RunHead>,
    initialized: bool,
}
// Immutable aliases are shared; mutation below requires the actual native
// atomic count to be one, or creates an independent copy first.
unsafe impl Send for RunHeadHandle {}
unsafe impl Sync for RunHeadHandle {}
impl core::ops::Deref for RunHeadHandle {
    type Target = RunHead;
    fn deref(&self) -> &RunHead {
        unsafe { self.pointer.as_ref() }
    }
}
impl core::fmt::Debug for RunHeadHandle {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RunHeadHandle")
            .field("header", &**self)
            .field("initialized", &self.initialized)
            .finish()
    }
}
impl Clone for RunHeadHandle {
    fn clone(&self) -> Self {
        assert!(self.initialized, "only completed regions may be shared");
        let previous = self.ref_count.fetch_add(1, Ordering::SeqCst);
        debug_assert!(previous >= 1);
        Self {
            pointer: self.pointer,
            initialized: true,
        }
    }
}
impl Drop for RunHeadHandle {
    fn drop(&mut self) {
        debug_assert!(self.ref_count.load(Ordering::SeqCst) >= 1);
        if self.ref_count.fetch_sub(1, Ordering::SeqCst) == 1 {
            unsafe {
                region_head_free(self.pointer.as_ptr().cast());
            }
        }
    }
}
impl RunHeadHandle {
    fn unique_header(&mut self) -> &mut RunHead {
        assert_eq!(self.ref_count.load(Ordering::SeqCst), 1);
        unsafe { self.pointer.as_mut() }
    }
    fn runs_ptr(&self) -> *mut i32 {
        unsafe { self.pointer.as_ptr().add(1).cast() }
    }
    pub fn y_span_count(&self) -> i32 {
        self.y_span_count
    }
    pub fn interval_count(&self) -> i32 {
        self.interval_count
    }
}
impl RunHead {
    fn layout(count: i32) -> Layout {
        Layout::from_size_align(
            core::mem::size_of::<Self>() + count as usize * 4,
            core::mem::align_of::<Self>(),
        )
        .expect("Invalid Size")
    }
    // Source: src/core/SkRegionPriv.h:82-97
    pub fn alloc(count: i32) -> Option<RunHeadHandle> {
        if count < 7 {
            return None;
        }
        let size = i64::from(count) * 4 + core::mem::size_of::<Self>() as i64;
        if count < 0 || i32::try_from(size).is_err() {
            crate::SK_ABORT!(@source b"src/core/SkRegionPriv.h\0".as_ptr(), 88; "Invalid Size");
        };
        debug_assert_eq!(core::mem::size_of::<Self>(), 16);
        let layout = Self::layout(count);
        let raw = unsafe { region_head_malloc(layout.size()) };
        let Some(pointer) = NonNull::new(raw.cast::<Self>()) else {
            handle_alloc_error(layout)
        };
        unsafe {
            pointer.as_ptr().write(Self {
                ref_count: AtomicI32::new(1),
                run_count: count,
                y_span_count: 0,
                interval_count: 0,
            });
        }
        // Unlike forming an i32 slice over uninitialized memory, this handle
        // can safely exist until the caller writes the full native run array.
        Some(RunHeadHandle {
            pointer,
            initialized: false,
        })
    }
    // Source: src/core/SkRegionPriv.h:99-111
    pub fn alloc_with_counts(
        count: i32,
        y_span_count: i32,
        interval_count: i32,
    ) -> Option<RunHeadHandle> {
        if y_span_count <= 0 || interval_count <= 1 {
            return None;
        }
        let mut head = Self::alloc(count)?;
        let header = head.unique_header();
        header.y_span_count = y_span_count;
        header.interval_count = interval_count;
        Some(head)
    }
    // Source: src/core/SkRegionPriv.h:113-140
    pub fn ensure_writable(head: &mut RunHeadHandle) -> &mut RunHeadHandle {
        if head.ref_count.load(Ordering::SeqCst) > 1 {
            let mut writable =
                Self::alloc_with_counts(head.run_count, head.y_span_count(), head.interval_count())
                    .expect("valid complex region counts");
            writable.copy_runs_from(head.readonly_runs());
            // Replacing the handle decrements and possibly frees the old head
            // only after the new allocation has copied all source run values.
            *head = writable;
        }
        head
    }
}
impl RunHeadHandle {
    pub fn writable_runs(&mut self) -> &mut [i32] {
        assert!(
            self.initialized,
            "write all runs with copy_runs_from before taking an i32 slice"
        );
        assert_eq!(self.ref_count.load(Ordering::SeqCst), 1);
        unsafe { core::slice::from_raw_parts_mut(self.runs_ptr(), self.run_count as usize) }
    }
    pub fn readonly_runs(&self) -> &[i32] {
        assert!(self.initialized, "uninitialized region run array");
        unsafe { core::slice::from_raw_parts(self.runs_ptr(), self.run_count as usize) }
    }
    pub fn copy_runs_from(&mut self, source: &[i32]) {
        assert_eq!(self.ref_count.load(Ordering::SeqCst), 1);
        assert_eq!(source.len(), self.run_count as usize);
        unsafe {
            core::ptr::copy_nonoverlapping(source.as_ptr(), self.runs_ptr(), source.len());
        }
        self.initialized = true;
    }
    pub(crate) fn write_run(&mut self, index: usize, value: i32) {
        assert_eq!(self.ref_count.load(Ordering::SeqCst), 1);
        assert!(index < self.run_count as usize);
        unsafe {
            self.runs_ptr().add(index).write(value);
        }
    }
    // Safety: caller has written all run_count values in the native tail.
    pub(crate) unsafe fn assume_initialized(&mut self) {
        self.initialized = true;
    }
    pub fn ptr_eq(a: &Self, b: &Self) -> bool {
        a.pointer == b.pointer
    }
    // Source: src/core/SkRegionPriv.h:141-161
    pub fn skip_entire_scanline(runs: &[i32]) -> usize {
        crate::SkASSERT!(@source b"src/core/SkRegionPriv.h\0".as_ptr(), 147; @condition b"runs[0] < SkRegion_kRunTypeSentinel\0".as_ptr(); runs[0] < RUN_TYPE_SENTINEL);
        let intervals = runs[1] as usize;
        crate::SkASSERT!(@source b"src/core/SkRegionPriv.h\0".as_ptr(), 150; @condition b"runs[2 + intervals * 2] == SkRegion_kRunTypeSentinel\0".as_ptr(); (runs[2 + intervals * 2]) == ( RUN_TYPE_SENTINEL));
        #[cfg(sk_debug)]
        crate::SkASSERT!(@source b"src/core/SkRegionPriv.h\0".as_ptr(), 154; @condition b"n == intervals\0".as_ptr(); (compute_interval_count(&runs[2..])) == ( intervals));
        1 + 1 + intervals * 2 + 1
    }

    // Source: src/core/SkRegionPriv.h:164-189
    pub fn find_scanline(&self, y: i32) -> &[i32] {
        let mut runs = self.readonly_runs();
        crate::SkASSERT!(@source b"src/core/SkRegionPriv.h\0".as_ptr(), 175; @condition b"y >= runs[0]\0".as_ptr(); y >= runs[0]);
        runs = &runs[1..];
        loop {
            let bottom = runs[0];
            crate::SkASSERT!(@source b"src/core/SkRegionPriv.h\0".as_ptr(), 182; @condition b"bottom < SkRegion_kRunTypeSentinel\0".as_ptr(); bottom < RUN_TYPE_SENTINEL);
            if y < bottom {
                break;
            }
            runs = &runs[Self::skip_entire_scanline(runs)..];
        }
        runs
    }

    // Source: src/core/SkRegionPriv.h:191-251
    pub fn compute_run_bounds(&mut self, bounds: &mut SkIRect) {
        let runs = self.readonly_runs();
        bounds.fTop = runs[0];
        let mut cursor = 1;
        let mut y_span_count = 0;
        let mut interval_count = 0;
        let mut left = i32::MAX;
        let mut right = i32::MIN;
        let bottom = loop {
            let bottom = runs[cursor];
            cursor += 1;
            crate::SkASSERT!(@source b"src/core/SkRegionPriv.h\0".as_ptr(), 204; @condition b"bot < SkRegion_kRunTypeSentinel\0".as_ptr(); bottom < RUN_TYPE_SENTINEL);
            y_span_count += 1;
            let intervals = runs[cursor];
            cursor += 1;
            {
                crate::SkASSERT!(@source b"src/core/SkRegionPriv.h\0".as_ptr(), 208; @condition b"intervals >= 0\0".as_ptr(); intervals>=0);
                crate::SkASSERT!(@source b"src/core/SkRegionPriv.h\0".as_ptr(), 209; @condition b"intervals < SkRegion_kRunTypeSentinel\0".as_ptr(); intervals<RUN_TYPE_SENTINEL);
            };
            if intervals > 0 {
                #[cfg(sk_debug)]
                crate::SkASSERT!(@source b"src/core/SkRegionPriv.h\0".as_ptr(), 215; @condition b"n == intervals\0".as_ptr(); (compute_interval_count(&runs[cursor..])) == ( intervals as usize));
                let l = runs[cursor];
                crate::SkASSERT!(@source b"src/core/SkRegionPriv.h\0".as_ptr(), 219; @condition b"L < SkRegion_kRunTypeSentinel\0".as_ptr(); l < RUN_TYPE_SENTINEL);
                if left > l {
                    left = l;
                }
                cursor += intervals as usize * 2;
                let r = runs[cursor - 1];
                crate::SkASSERT!(@source b"src/core/SkRegionPriv.h\0".as_ptr(), 226; @condition b"R < SkRegion_kRunTypeSentinel\0".as_ptr(); r < RUN_TYPE_SENTINEL);
                if right < r {
                    right = r;
                }
                interval_count += intervals;
            }
            crate::SkASSERT!(@source b"src/core/SkRegionPriv.h\0".as_ptr(), 233; @condition b"SkRegion_kRunTypeSentinel == *runs\0".as_ptr(); (RUN_TYPE_SENTINEL) == ( runs[cursor]));
            cursor += 1;
            if runs[cursor] >= RUN_TYPE_SENTINEL {
                break bottom;
            }
        };
        #[cfg(sk_debug)]
        crate::SkASSERT!(@source b"src/core/SkRegionPriv.h\0".as_ptr(), 242; @condition b"runCount == fRunCount\0".as_ptr(); (cursor + 1) == ( self.run_count as usize));
        let header = self.unique_header();
        header.y_span_count = y_span_count;
        header.interval_count = interval_count;
        bounds.fLeft = left;
        bounds.fRight = right;
        bounds.fBottom = bottom;
    }
}

unsafe extern "C" {
    #[link_name = "malloc"]
    fn region_head_malloc(size: usize) -> *mut core::ffi::c_void;
    #[link_name = "free"]
    fn region_head_free(pointer: *mut core::ffi::c_void);
}
