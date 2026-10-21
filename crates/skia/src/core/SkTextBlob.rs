use crate::include::core::SkRefCnt::SkNVRefCnt;

use crate::src::core::SkRect::SkRect;

use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

use crate::src::core::SkSafeMath::SkSafeMath;

use crate::src::core::SkTextBlobPriv::RunRecord;
use core::mem::size_of;

// Rust unwind guard: the native purge-before-runs order is retained, while Rust callback
// unwinding must still release every run's font. The guard itself carries no source credit.
struct RunDestructionGuard(*mut SkTextBlob);

// No-storage owning-protocol glue, not an upstream algorithm region. The same
// malloc block remains owned by the guard until both header and trailing runs are dropped.
struct BlobStorageGuard(*mut SkTextBlob);

impl Drop for BlobStorageGuard {
    fn drop(&mut self) {
        unsafe { SkTextBlob::free_storage(self.0) }
    }
}

impl SkTextBlob {
    /// Safety: ptr is the original malloc-compatible allocation, completely initialized,
    /// with the sole remaining native reference already consumed. No Box is reconstructed.
    /// Run font destructors must not unwind; a purge callback may unwind, in which case
    /// the runs and matching storage are still destroyed exactly once.
    pub(crate) unsafe fn dispose_raw(ptr: std::ptr::NonNull<Self>) {
        let _storage = BlobStorageGuard(ptr.as_ptr());
        std::ptr::drop_in_place(ptr.as_ptr());
    }
}

use crate::src::core::SkTextBlobPriv::SkTextBlobRunIterator;
use crate::src::core::{
    SkFontPriv::SkFontPriv,
    SkReadBuffer::SkReadBuffer,
    SkTextBlobPriv::SkTextBlobPriv,
    SkWriteBuffer::{SkBinaryWriteBuffer, SkWriteBuffer},
};

// Canonical ownership protocol glue, adapted/uncredited: reuse the original sole embedded
// NV counter and matching raw disposer. The shared sk_sp performs all handle operations.
unsafe impl crate::include::core::SkRefCnt::SkNVRefCntObject for SkTextBlob {
    fn nv_ref_cnt(&self) -> &SkNVRefCnt<Self> {
        &self.fRefCnt
    }
    unsafe fn dispose_raw(object: std::ptr::NonNull<Self>) {
        Self::dispose_raw(object);
    }
}

unsafe impl crate::include::core::SkRefCnt::SkIntrusiveRefCounted for SkTextBlob {
    unsafe fn intrusive_retain(object: std::ptr::NonNull<Self>) {
        object.as_ref().fRefCnt.ref_();
    }
    unsafe fn intrusive_release(object: std::ptr::NonNull<Self>) {
        SkNVRefCnt::unref(object);
    }
}

impl SkTextBlob {
    // Source: include/core/SkTextBlob.h:54-54
    pub fn bounds(&self) -> &SkRect {
        &self.fBounds
    }
}

impl SkTextBlob {
    // Source: include/core/SkTextBlob.h:60-60
    pub fn uniqueID(&self) -> u32 {
        self.fUniqueID
    }
}

// Source: include/core/SkTextBlob.h:233-236
// Source: include/core/SkTextBlob.h:242-242
pub struct Run {
    pub fTypeface: *const crate::src::core::SkTypeface::SkTypeface,
    pub fGlyphCount: i32,
    pub fGlyphIndices: *const u16,
    // Source: include/core/SkTextBlob.h:237-241
    #[cfg(SK_UNTIL_CRBUG_1187654_IS_FIXED)]
    pub fClusterIndex_forTest: *const u32,
    #[cfg(SK_UNTIL_CRBUG_1187654_IS_FIXED)]
    pub fUtf8Size_forTest: i32,
    #[cfg(SK_UNTIL_CRBUG_1187654_IS_FIXED)]
    pub fUtf8_forTest: *const core::ffi::c_char,
}

// Source: include/core/SkTextBlob.h:253-258
pub struct ExperimentalRun {
    pub font: crate::src::core::SkFont::SkFont,
    pub count: i32,
    pub glyphs: *const u16,
    pub positions: *const crate::src::core::SkPoint::SkPoint,
}

// Adapted source: include/core/SkTextBlob.h:231-232: flat Rust Iter storage declaration is separate from native nested class scope
// Source: include/core/SkTextBlob.h:262-263
pub struct Iter {
    fRunRecord: *const RunRecord,
}

// Source: include/core/SkTextBlob.h:282-282
pub type PurgeDelegate = fn(u32, u32);

impl SkTextBlob {
    // Source: include/core/SkTextBlob.h:286-289
    pub fn notifyAddedToCache(&self, cache_id: u32, purge_delegate: PurgeDelegate) {
        self.fCacheID.store(cache_id, Ordering::SeqCst);
        self.fPurgeDelegate
            .store(purge_delegate as usize, Ordering::SeqCst);
    }
}

// Canonical contiguous object/run representation. Public Blob factory/owning handle
// awaits shared NV disposal support; no Box/Arc owner is substituted for the trailing block.
// Structural source: include/core/SkTextBlob.h:43-43: native private access label is language scaffolding, not the Blob type/body
// Source: include/core/SkTextBlob.h:296-299
#[repr(C)]
pub struct SkTextBlob {
    pub(crate) fRefCnt: core::mem::ManuallyDrop<SkNVRefCnt<SkTextBlob>>,
    pub(crate) fBounds: SkRect,
    pub(crate) fUniqueID: u32,
    pub(crate) fCacheID: AtomicU32,
    pub(crate) fPurgeDelegate: AtomicUsize,
    #[cfg(sk_debug)]
    pub(crate) fStorageSize: std::mem::MaybeUninit<usize>,
}

// Source: include/core/SkTextBlob.h:350-354
/// Native nonowning writable views supplied by SkTextBlobBuilder. Backing storage
/// expires at the builder's next allocation or publication; accessors only cast pointers.
#[derive(Clone, Copy)]
pub struct RunBuffer {
    pub glyphs: *mut u16,
    pub pos: *mut f32,
    pub utf8text: *mut u8,
    pub clusters: *mut u32,
}

// Source: include/core/SkTextBlob.h:357-358
impl RunBuffer {
    pub fn points(&self) -> *mut crate::src::core::SkPoint::SkPoint {
        self.pos.cast()
    }
    pub fn xforms(&self) -> *mut crate::src::core::SkRSXform::SkRSXform {
        self.pos.cast()
    }
}

impl RunRecord {
    // Source: src/core/SkTextBlob.cpp:49-52
    // Source: src/core/SkTextBlob.cpp:54-55
    // Source: src/core/SkTextBlob.cpp:58-60
    // Source: src/core/SkTextBlob.cpp:62-66
    // Source: src/core/SkTextBlob.cpp:68-69
    pub fn StorageSize(
        glyph_count: u32,
        text_size: u32,
        positioning: GlyphPositioning,
        safe: &mut SkSafeMath,
    ) -> usize {
        let glyph_size = safe.mul(glyph_count as usize, std::mem::size_of::<u16>());
        let pos_count = Self::PosCount(glyph_count, positioning, safe);
        let pos_size = safe.mul(pos_count, std::mem::size_of::<f32>());
        let mut size = std::mem::size_of::<Self>();
        let aligned = safe.alignUp(glyph_size, 4);
        size = safe.add(size, aligned);
        size = safe.add(size, pos_size);
        if text_size != 0 {
            size = safe.add(size, std::mem::size_of::<u32>());
            let clusters = safe.mul(glyph_count as usize, std::mem::size_of::<u32>());
            size = safe.add(size, clusters);
            size = safe.add(size, text_size as usize)
        }
        safe.alignUp(size, std::mem::size_of::<usize>())
    }
}

impl RunRecord {
    // Source: src/core/SkTextBlob.cpp:71-71
    // Source: src/core/SkTextBlob.cpp:74-75
    pub unsafe fn First(blob: *const SkTextBlob) -> *const Self {
        let p = blob.add(1).cast::<u8>();
        p.add(p.align_offset(std::mem::size_of::<usize>())).cast()
    }
}

impl RunRecord {
    // Source: src/core/SkTextBlob.cpp:77-79
    pub unsafe fn Next(run: *const Self) -> *const Self {
        if (*run).fFlags & Self::LAST_FLAG != 0 {
            std::ptr::null()
        } else {
            Self::NextUnchecked(run)
        }
    }
}

impl RunRecord {
    // Source: src/core/SkTextBlob.cpp:91-93
    // Source: src/core/SkTextBlob.cpp:95-103
    pub unsafe fn validate(&self, storage_top: *const u8) {
        #[cfg(sk_debug)]
        {
            debug_assert_eq!(
                self.fMagic,
                crate::src::core::SkTextBlobPriv::kRunRecordMagic
            );
            let end = Self::NextUnchecked(self);
            debug_assert!(end.cast::<u8>() <= storage_top);
            debug_assert!(
                self.glyphBuffer().add(self.fCount as usize).cast::<u8>()
                    <= self.posBuffer().cast::<u8>()
            );
            debug_assert!(
                self.posBuffer()
                    .add(
                        self.fCount as usize
                            * SkTextBlob::ScalarsPerGlyph(self.positioning()) as usize
                    )
                    .cast::<u8>()
                    <= end.cast_mut().cast()
            );
            if self.isExtended() {
                crate::SkASSERT!(@source b"src/core/SkTextBlob.cpp\0".as_ptr(),99; @condition b"textSize() > 0\0".as_ptr(); self.textSize() > 0);
                debug_assert!(self.textSizePtr().cast::<u8>() < end.cast_mut().cast());
                debug_assert!(self.clusterBuffer().cast::<u8>() < end.cast_mut().cast());
                debug_assert!(
                    self.textBuffer().add(self.textSize() as usize) <= end.cast_mut().cast()
                );
            }
        }
    }
}

impl RunRecord {
    // Source: src/core/SkTextBlob.cpp:108-115
    pub(crate) unsafe fn NextUnchecked(run: *const Self) -> *const Self {
        let mut safe = SkSafeMath::new();
        let size = Self::StorageSize(
            (*run).glyphCount(),
            (*run).textSize(),
            (*run).positioning(),
            &mut safe,
        );
        let res = run.cast::<u8>().add(size).cast();
        debug_assert!(safe.ok());
        res
    }
}

impl RunRecord {
    // Source: src/core/SkTextBlob.cpp:117-121
    pub(crate) fn PosCount(
        glyph_count: u32,
        positioning: GlyphPositioning,
        safe: &mut SkSafeMath,
    ) -> usize {
        safe.mul(
            glyph_count as usize,
            SkTextBlob::ScalarsPerGlyph(positioning) as usize,
        )
    }
}

impl RunRecord {
    // Source: src/core/SkTextBlob.cpp:123-123
    // Source: src/core/SkTextBlob.cpp:125-130
    pub(crate) unsafe fn textSizePtr(&self) -> *mut u32 {
        crate::SkASSERT!(@source b"src/core/SkTextBlob.cpp\0".as_ptr(),125; @condition b"isExtended()\0".as_ptr(); self.isExtended());
        let mut safe = SkSafeMath::new();
        let res = self
            .posBuffer()
            .add(Self::PosCount(self.fCount, self.positioning(), &mut safe))
            .cast();
        debug_assert!(safe.ok());
        res
    }
}

impl RunRecord {
    // Source: src/core/SkTextBlob.cpp:132-135
    // Source: src/core/SkTextBlob.cpp:138-139
    // Source: src/core/SkTextBlob.cpp:142-143
    pub(crate) unsafe fn grow(&mut self, count: u32) {
        let initial_pos = self.posBuffer();
        let initial_count = self.fCount;
        self.fCount = self.fCount.wrapping_add(count);
        let copy_size = initial_count as usize
            * std::mem::size_of::<f32>()
            * SkTextBlob::ScalarsPerGlyph(self.positioning()) as usize;
        debug_assert!(
            self.posBuffer().cast::<u8>().add(copy_size)
                <= Self::NextUnchecked(self).cast_mut().cast()
        );
        std::ptr::copy(
            initial_pos.cast::<u8>(),
            self.posBuffer().cast::<u8>(),
            copy_size,
        )
    }
}

// Source: src/core/SkTextBlob.cpp:145-152
pub(crate) fn next_id() -> u32 {
    static NEXT_ID: AtomicU32 = AtomicU32::new(1);
    loop {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        if id != 0 {
            return id;
        }
    }
}

impl SkTextBlob {
    // Source: src/core/SkTextBlob.cpp:154-158
    /// Safety: storage names aligned uninitialized header bytes in an allocation including
    /// at least one initialized trailing run and a valid final last tag. In debug builds
    /// fStorageSize must be written with the allocation extent before any iteration or Drop.
    /// Initialization creates no owning handle; final release must use native NV disposal.
    pub(crate) unsafe fn init_at(storage: *mut Self, bounds: SkRect) {
        storage.write(Self {
            fRefCnt: core::mem::ManuallyDrop::new(SkNVRefCnt::new()),
            fBounds: bounds,
            fUniqueID: next_id(),
            fCacheID: AtomicU32::new(0),
            fPurgeDelegate: AtomicUsize::new(0),
            #[cfg(sk_debug)]
            fStorageSize: std::mem::MaybeUninit::uninit(),
        })
    }
}

impl Drop for SkTextBlob {
    // Source: src/core/SkTextBlob.cpp:160-165
    fn drop(&mut self) {
        // Source: include/core/SkTextBlob.h:42,296-305
        // Native destructor body destroys trailing runs before the actual NV
        // base. Guard order also retains that sequence if purge unwinds.
        struct BaseDestructionGuard(*mut core::mem::ManuallyDrop<SkNVRefCnt<SkTextBlob>>);
        impl Drop for BaseDestructionGuard {
            fn drop(&mut self) {
                unsafe {
                    core::mem::ManuallyDrop::drop(&mut *self.0);
                }
            }
        }
        let object = self as *mut SkTextBlob;
        let _destruction = (
            RunDestructionGuard(object),
            BaseDestructionGuard(core::ptr::addr_of_mut!(self.fRefCnt)),
        );
        unsafe {
            if self.fCacheID.load(Ordering::SeqCst) != 0 {
                let raw = self.fPurgeDelegate.load(Ordering::SeqCst);
                assert!(raw != 0);
                let f: PurgeDelegate = std::mem::transmute(raw);
                f(self.fUniqueID, self.fCacheID.load(Ordering::SeqCst))
            }
        }
    }
}

impl Drop for RunDestructionGuard {
    // Source: src/core/SkTextBlob.cpp:167-174
    fn drop(&mut self) {
        unsafe {
            let mut run = RunRecord::First(self.0);
            loop {
                let next = RunRecord::Next(run);
                #[cfg(sk_debug)]
                (*run).validate(
                    self.0
                        .cast::<u8>()
                        .add((*self.0).fStorageSize.assume_init()),
                );
                std::ptr::drop_in_place(run.cast_mut());
                run = next;
                if run.is_null() {
                    break;
                }
            }
        }
    }
}

// Source: src/core/SkTextBlob.cpp:178-185
// Adapted union storage: preserve all four native-endian bytes, including
// padding and the complete extended byte; no assumed little-endian bit shifts.
struct PositioningAndExtended {
    bytes: [u8; 4],
}
impl PositioningAndExtended {
    fn from_int(value: i32) -> Self {
        Self {
            bytes: value.to_ne_bytes(),
        }
    }
    fn int_value(&self) -> i32 {
        i32::from_ne_bytes(self.bytes)
    }
    fn positioning(&self) -> u8 {
        self.bytes[0]
    }
    fn set_positioning(&mut self, value: u8) {
        self.bytes[0] = value;
    }
    fn extended(&self) -> bool {
        self.bytes[1] != 0
    }
    fn set_extended(&mut self, value: bool) {
        self.bytes[1] = value as u8;
    }
}

// Source: src/core/SkTextBlob.cpp:191-196
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum GlyphPositioning {
    kDefault_Positioning = 0,
    kHorizontal_Positioning = 1,
    kFull_Positioning = 2,
    kRSXform_Positioning = 3,
}

impl SkTextBlob {
    // Source: src/core/SkTextBlob.cpp:198-207
    pub fn ScalarsPerGlyph(pos: GlyphPositioning) -> u32 {
        const SCALARS: [u8; 4] = [0, 1, 2, 4];
        debug_assert!((pos as usize) <= 3);
        SCALARS[pos as usize] as u32
    }
}

impl SkTextBlob {
    // Source: src/core/SkTextBlob.cpp:209-211
    /// Safety: ptr names a malloc-compatible block whose objects were already destroyed.
    pub(crate) unsafe fn free_storage(ptr: *mut Self) {
        unsafe extern "C" {
            fn free(p: *mut std::ffi::c_void);
        }
        free(ptr.cast());
    }
}

impl SkTextBlobRunIterator {
    // Source: src/core/SkTextBlob.cpp:221-224
    /// Safety: blob points to a live initialized contiguous blob, kept alive and immutable
    /// throughout this iterator's use. Debug storage size must already be initialized.
    pub unsafe fn new(blob: *const SkTextBlob) -> Self {
        Self {
            fCurrentRun: RunRecord::First(blob),
            #[cfg(sk_debug)]
            fStorageTop: blob.cast::<u8>().add((*blob).fStorageSize.assume_init()),
        }
    }
}

impl SkTextBlobRunIterator {
    // Source: src/core/SkTextBlob.cpp:226-227
    // Source: src/core/SkTextBlob.cpp:229-233
    pub unsafe fn next(&mut self) {
        crate::SkASSERT!(@source b"src/core/SkTextBlob.cpp\0".as_ptr(),227; @condition b"!this->done()\0".as_ptr(); !self.done());
        if !self.done() {
            #[cfg(sk_debug)]
            (*self.fCurrentRun).validate(self.fStorageTop);
            self.fCurrentRun = RunRecord::Next(self.fCurrentRun);
        }
    }
}

impl SkTextBlobRunIterator {
    // Source: src/core/SkTextBlob.cpp:235-244
    // Source: src/core/SkTextBlob.cpp:246-247
    pub unsafe fn positioning(&self) -> GlyphPositioning {
        crate::SkASSERT!(@source b"src/core/SkTextBlob.cpp\0".as_ptr(),236; @condition b"!this->done()\0".as_ptr(); !self.done());
        // The iterator uses the same canonical enum, enforcing the native static assertions.
        (*self.fCurrentRun).positioning()
    }
}

impl SkTextBlobRunIterator {
    // Source: src/core/SkTextBlob.cpp:249-251
    pub unsafe fn scalarsPerGlyph(&self) -> u32 {
        SkTextBlob::ScalarsPerGlyph((*self.fCurrentRun).positioning())
    }
}

impl SkTextBlobRunIterator {
    // Source: src/core/SkTextBlob.cpp:253-255
    pub unsafe fn isLCD(&self) -> bool {
        (*self.fCurrentRun).font().getEdging()
            == crate::src::core::SkFont::Edging::kSubpixelAntiAlias
    }
}

// Source: src/core/SkTextBlob.cpp:257-264
// Source: include/core/SkTextBlob.h:529-538
// malloc/realloc keeps the original relocatable run representation; no Vec/Box
// reconstructs ownership of trailing records. All writable views expire at reserve/make.
pub struct SkTextBlobBuilder {
    fStorage: *mut u8,
    fStorageSize: usize,
    fStorageUsed: usize,
    fBounds: SkRect,
    fRunCount: i32,
    fDeferredBounds: bool,
    fLastRun: usize,
    fCurrentRunBuffer: RunBuffer,
}
impl SkTextBlobBuilder {
    pub fn new() -> Self {
        Self {
            fStorage: std::ptr::null_mut(),
            fStorageSize: 0,
            fStorageUsed: 0,
            fBounds: SkRect::MakeEmpty(),
            fRunCount: 0,
            fDeferredBounds: false,
            fLastRun: 0,
            fCurrentRunBuffer: RunBuffer {
                glyphs: std::ptr::null_mut(),
                pos: std::ptr::null_mut(),
                utf8text: std::ptr::null_mut(),
                clusters: std::ptr::null_mut(),
            },
        }
    }
}
// Source: src/core/SkTextBlob.cpp:266-272
impl Drop for SkTextBlobBuilder {
    fn drop(&mut self) {
        if !self.fStorage.is_null() {
            // Native make consumes and destroys abandoned runs through the blob owner.
            unsafe {
                drop(self.make());
            }
        }
    }
}
// Source: src/core/SkTextBlob.cpp:274-276
fn map_quad_to_rect(xform: &crate::src::core::SkRSXform::SkRSXform, rect: &SkRect) -> SkRect {
    let mut matrix = crate::src::core::SkMatrix::SkMatrix::default();
    matrix.setRSXform(xform);
    let mut bounds = SkRect::MakeEmpty();
    matrix.mapRect(&mut bounds, rect);
    bounds
}
impl SkTextBlobBuilder {
    // Source: src/core/SkTextBlob.cpp:278-322
    unsafe fn TightRunBounds(run: &RunRecord) -> SkRect {
        let font = run.font();
        let mut bounds = SkRect::MakeEmpty();
        if run.positioning() == GlyphPositioning::kDefault_Positioning {
            font.measureText(
                run.glyphBuffer().cast(),
                run.glyphCount() as usize * 2,
                crate::include::core::SkFontTypes::SkTextEncoding::kGlyphID,
                Some(&mut bounds),
                None,
            )
            .expect("native no-effects font measurement has no fallible color-filter producer");
            return bounds.makeOffset(run.offset().fX, run.offset().fY);
        }
        let mut glyph_bounds = vec![SkRect::MakeEmpty(); run.glyphCount() as usize];
        font.getBounds(
            std::slice::from_raw_parts(run.glyphBuffer(), run.glyphCount() as usize),
            &mut glyph_bounds,
            None,
        )
        .expect("native no-effects font measurement has no fallible color-filter producer");
        if run.positioning() == GlyphPositioning::kRSXform_Positioning {
            let xform = run.xformBuffer();
            debug_assert!(
                xform.add(run.glyphCount() as usize).cast::<u8>()
                    <= RunRecord::Next(run).cast_mut().cast()
            );
            for i in 0..run.glyphCount() as usize {
                bounds.join(&map_quad_to_rect(&*xform.add(i), &glyph_bounds[i]));
            }
        } else {
            debug_assert!(matches!(
                run.positioning(),
                GlyphPositioning::kFull_Positioning | GlyphPositioning::kHorizontal_Positioning
            ));
            let horizontal_y: f32 = 0.0;
            let mut px = run.posBuffer();
            let full = run.positioning() == GlyphPositioning::kFull_Positioning;
            let mut py = if full {
                px.add(1)
            } else {
                &horizontal_y as *const f32
            };
            let inc_x = SkTextBlob::ScalarsPerGlyph(run.positioning()) as usize;
            let inc_y = if full { inc_x } else { 0 };
            for glyph in &glyph_bounds {
                bounds.join(&glyph.makeOffset(*px, *py));
                px = px.add(inc_x);
                py = py.add(inc_y);
            }
            debug_assert!(px.cast::<u8>() <= RunRecord::Next(run).cast_mut().cast());
        }
        bounds.makeOffset(run.offset().fX, run.offset().fY)
    }
    // Source: src/core/SkTextBlob.cpp:324-382
    unsafe fn ConservativeRunBounds(run: &RunRecord) -> SkRect {
        crate::SkASSERT!(@source b"src/core/SkTextBlob.cpp\0".as_ptr(),325; @condition b"run.glyphCount() > 0\0".as_ptr(); run.glyphCount() > 0);
        debug_assert!(run.positioning() != GlyphPositioning::kDefault_Positioning);
        let font_bounds = crate::src::core::SkFontPriv::SkFontPriv::GetFontBounds(run.font());
        if font_bounds.isEmpty() {
            return Self::TightRunBounds(run);
        }
        let mut bounds = SkRect::MakeEmpty();
        match run.positioning() {
            GlyphPositioning::kHorizontal_Positioning => {
                let p = run.posBuffer();
                debug_assert!(
                    p.add(run.glyphCount() as usize).cast::<u8>()
                        <= RunRecord::Next(run).cast_mut().cast()
                );
                let mut min_x = *p;
                let mut max_x = *p;
                for i in 1..run.glyphCount() as usize {
                    let x = *p.add(i);
                    // Preserve std::min/max operand order, including NaN and signed zero.
                    min_x = if min_x < x { min_x } else { x };
                    max_x = if x < max_x { max_x } else { x };
                }
                bounds = SkRect::MakeLTRB(min_x, 0.0, max_x, 0.0);
            }
            GlyphPositioning::kFull_Positioning => {
                let p = run.pointBuffer();
                debug_assert!(
                    p.add(run.glyphCount() as usize).cast::<u8>()
                        <= RunRecord::Next(run).cast_mut().cast()
                );
                bounds =
                    SkRect::BoundsOrEmpty(std::slice::from_raw_parts(p, run.glyphCount() as usize));
            }
            GlyphPositioning::kRSXform_Positioning => {
                let p = run.xformBuffer();
                debug_assert!(
                    p.add(run.glyphCount() as usize).cast::<u8>()
                        <= RunRecord::Next(run).cast_mut().cast()
                );
                for i in 0..run.glyphCount() as usize {
                    bounds.join(&map_quad_to_rect(&*p.add(i), &font_bounds));
                }
            }
            _ => unreachable!("unsupported positioning mode"),
        }
        if run.positioning() != GlyphPositioning::kRSXform_Positioning {
            bounds.fLeft += font_bounds.left();
            bounds.fTop += font_bounds.top();
            bounds.fRight += font_bounds.right();
            bounds.fBottom += font_bounds.bottom();
        }
        bounds.makeOffset(run.offset().fX, run.offset().fY)
    }
    // Source: src/core/SkTextBlob.cpp:384-400
    unsafe fn updateDeferredBounds(&mut self) {
        crate::SkASSERT!(@source b"src/core/SkTextBlob.cpp\0".as_ptr(),385; @condition b"!fDeferredBounds || fRunCount > 0\0".as_ptr(); !self.fDeferredBounds || self.fRunCount > 0);
        if !self.fDeferredBounds {
            return;
        }
        debug_assert!(self.fLastRun >= blob_header_size());
        let run = &*self.fStorage.add(self.fLastRun).cast::<RunRecord>();
        let bounds = if run.positioning() == GlyphPositioning::kDefault_Positioning {
            Self::TightRunBounds(run)
        } else {
            Self::ConservativeRunBounds(run)
        };
        self.fBounds.join(&bounds);
        self.fDeferredBounds = false;
    }
    // Source: src/core/SkTextBlob.cpp:402-425
    unsafe fn reserve(&mut self, size: usize) {
        let mut safe = SkSafeMath::new();
        if safe.add(self.fStorageUsed, size) <= self.fStorageSize && safe.ok() {
            return;
        }
        if self.fRunCount == 0 {
            debug_assert!(self.fStorage.is_null());
            crate::SkASSERT!(@source b"src/core/SkTextBlob.cpp\0".as_ptr(),412; @condition b"0 == fStorageSize\0".as_ptr(); (self.fStorageSize)==(0));
            crate::SkASSERT!(@source b"src/core/SkTextBlob.cpp\0".as_ptr(),413; @condition b"0 == fStorageUsed\0".as_ptr(); (self.fStorageUsed)==(0));
            self.fStorageUsed = blob_header_size();
        }
        self.fStorageSize = safe.add(self.fStorageUsed, size);
        unsafe extern "C" {
            fn realloc(p: *mut std::ffi::c_void, size: usize) -> *mut std::ffi::c_void;
        }
        let requested = if safe.ok() {
            self.fStorageSize
        } else {
            usize::MAX
        };
        let storage = realloc(self.fStorage.cast(), requested).cast::<u8>();
        if storage.is_null() {
            std::process::abort();
        }
        self.fStorage = storage;
    }
    // Source: src/core/SkTextBlob.cpp:427-485
    unsafe fn mergeRun(
        &mut self,
        font: &crate::src::core::SkFont::SkFont,
        positioning: GlyphPositioning,
        count: u32,
        offset: crate::src::core::SkPoint::SkPoint,
    ) -> bool {
        if self.fLastRun == 0 {
            crate::SkASSERT!(@source b"src/core/SkTextBlob.cpp\0".as_ptr(),430; @condition b"0 == fRunCount\0".as_ptr(); (self.fRunCount)==(0));
            return false;
        }
        debug_assert!(self.fLastRun >= blob_header_size());
        let run = &*self.fStorage.add(self.fLastRun).cast::<RunRecord>();
        crate::SkASSERT!(@source b"src/core/SkTextBlob.cpp\0".as_ptr(),437; @condition b"run->glyphCount() > 0\0".as_ptr(); run.glyphCount() > 0);
        if run.textSize() != 0 {
            return false;
        }
        if run.positioning() != positioning
            || run.font() != font
            || run.glyphCount().wrapping_add(count) < run.glyphCount()
        {
            return false;
        }
        if positioning != GlyphPositioning::kFull_Positioning
            && (positioning != GlyphPositioning::kHorizontal_Positioning
                || run.offset().fY != offset.fY)
        {
            return false;
        }
        let mut safe = SkSafeMath::new();
        let size_delta = RunRecord::StorageSize(
            run.glyphCount().wrapping_add(count),
            0,
            positioning,
            &mut safe,
        )
        .wrapping_sub(RunRecord::StorageSize(
            run.glyphCount(),
            0,
            positioning,
            &mut safe,
        ));
        if !safe.ok() {
            return false;
        }
        self.reserve(size_delta);
        let run = self.fStorage.add(self.fLastRun).cast::<RunRecord>();
        let previous_count = (*run).glyphCount();
        (*run).grow(count);
        self.fCurrentRunBuffer.glyphs = (*run).glyphBuffer().add(previous_count as usize);
        self.fCurrentRunBuffer.pos = (*run)
            .posBuffer()
            .add(previous_count as usize * SkTextBlob::ScalarsPerGlyph(positioning) as usize);
        self.fStorageUsed += size_delta;
        crate::SkASSERT!(@source b"src/core/SkTextBlob.cpp\0".as_ptr(),481; @condition b"fStorageUsed <= fStorageSize\0".as_ptr(); self.fStorageUsed <= self.fStorageSize);
        (*run).validate(self.fStorage.add(self.fStorageUsed));
        true
    }
    // Source: src/core/SkTextBlob.cpp:487-534
    unsafe fn allocInternal(
        &mut self,
        font: &crate::src::core::SkFont::SkFont,
        positioning: GlyphPositioning,
        count: i32,
        text_size: i32,
        offset: crate::src::core::SkPoint::SkPoint,
        bounds: Option<&SkRect>,
    ) {
        if count <= 0 || text_size < 0 {
            self.fCurrentRunBuffer = RunBuffer {
                glyphs: std::ptr::null_mut(),
                pos: std::ptr::null_mut(),
                utf8text: std::ptr::null_mut(),
                clusters: std::ptr::null_mut(),
            };
            return;
        }
        if text_size != 0 || !self.mergeRun(font, positioning, count as u32, offset) {
            self.updateDeferredBounds();
            let mut safe = SkSafeMath::new();
            let run_size =
                RunRecord::StorageSize(count as u32, text_size as u32, positioning, &mut safe);
            if !safe.ok() {
                self.fCurrentRunBuffer = RunBuffer {
                    glyphs: std::ptr::null_mut(),
                    pos: std::ptr::null_mut(),
                    utf8text: std::ptr::null_mut(),
                    clusters: std::ptr::null_mut(),
                };
                return;
            }
            self.reserve(run_size);
            debug_assert!(self.fStorageUsed >= blob_header_size());
            crate::SkASSERT!(@source b"src/core/SkTextBlob.cpp\0".as_ptr(),509; @condition b"fStorageUsed + runSize <= fStorageSize\0".as_ptr(); self.fStorageUsed + run_size <= self.fStorageSize);
            let run = self.fStorage.add(self.fStorageUsed).cast::<RunRecord>();
            RunRecord::init_at(
                run,
                count as u32,
                text_size as u32,
                offset,
                font,
                positioning,
            );
            self.fCurrentRunBuffer = RunBuffer {
                glyphs: (*run).glyphBuffer(),
                pos: (*run).posBuffer(),
                utf8text: (*run).textBuffer(),
                clusters: (*run).clusterBuffer(),
            };
            self.fLastRun = self.fStorageUsed;
            self.fStorageUsed += run_size;
            self.fRunCount += 1;
            crate::SkASSERT!(@source b"src/core/SkTextBlob.cpp\0".as_ptr(),522; @condition b"fStorageUsed <= fStorageSize\0".as_ptr(); self.fStorageUsed <= self.fStorageSize);
            (*run).validate(self.fStorage.add(self.fStorageUsed));
        }
        debug_assert!(text_size > 0 || self.fCurrentRunBuffer.utf8text.is_null());
        debug_assert!(text_size > 0 || self.fCurrentRunBuffer.clusters.is_null());
        if !self.fDeferredBounds {
            if let Some(bounds) = bounds {
                self.fBounds.join(bounds);
            } else {
                self.fDeferredBounds = true;
            }
        }
    }
    // Source: src/core/SkTextBlob.cpp:538-543
    /// Safety: initialize all returned glyph/position/text views before another allocation, make, destruction or unwinding; font and bounds do not alias builder storage.
    pub unsafe fn allocRun(
        &mut self,
        font: &crate::src::core::SkFont::SkFont,
        count: i32,
        x: f32,
        y: f32,
        bounds: Option<&SkRect>,
    ) -> &RunBuffer {
        self.allocInternal(
            font,
            GlyphPositioning::kDefault_Positioning,
            count,
            0,
            crate::src::core::SkPoint::SkPoint::make(x, y),
            bounds,
        );
        &self.fCurrentRunBuffer
    }
    // Source: src/core/SkTextBlob.cpp:545-550
    /// Safety: initialize all returned glyph/position/text views before another allocation, make, destruction or unwinding; font and bounds do not alias builder storage.
    pub unsafe fn allocRunPosH(
        &mut self,
        font: &crate::src::core::SkFont::SkFont,
        count: i32,
        y: f32,
        bounds: Option<&SkRect>,
    ) -> &RunBuffer {
        self.allocInternal(
            font,
            GlyphPositioning::kHorizontal_Positioning,
            count,
            0,
            crate::src::core::SkPoint::SkPoint::make(0.0, y),
            bounds,
        );
        &self.fCurrentRunBuffer
    }
    // Source: src/core/SkTextBlob.cpp:552-556
    /// Safety: initialize all returned glyph/position/text views before another allocation, make, destruction or unwinding; font and bounds do not alias builder storage.
    pub unsafe fn allocRunPos(
        &mut self,
        font: &crate::src::core::SkFont::SkFont,
        count: i32,
        bounds: Option<&SkRect>,
    ) -> &RunBuffer {
        self.allocInternal(
            font,
            GlyphPositioning::kFull_Positioning,
            count,
            0,
            crate::src::core::SkPoint::SkPoint::make(0.0, 0.0),
            bounds,
        );
        &self.fCurrentRunBuffer
    }
    // Source: src/core/SkTextBlob.cpp:558-562
    /// Safety: initialize all returned glyph/position/text views before another allocation, make, destruction or unwinding; font and bounds do not alias builder storage.
    pub unsafe fn allocRunRSXform(
        &mut self,
        font: &crate::src::core::SkFont::SkFont,
        count: i32,
    ) -> &RunBuffer {
        self.allocInternal(
            font,
            GlyphPositioning::kRSXform_Positioning,
            count,
            0,
            crate::src::core::SkPoint::SkPoint::make(0.0, 0.0),
            None,
        );
        &self.fCurrentRunBuffer
    }
    // Source: src/core/SkTextBlob.cpp:564-575
    /// Safety: initialize all returned glyph/position/text views before another allocation, make, destruction or unwinding; font and bounds do not alias builder storage.
    pub unsafe fn allocRunText(
        &mut self,
        font: &crate::src::core::SkFont::SkFont,
        count: i32,
        x: f32,
        y: f32,
        text_size: i32,
        bounds: Option<&SkRect>,
    ) -> &RunBuffer {
        self.allocInternal(
            font,
            GlyphPositioning::kDefault_Positioning,
            count,
            text_size,
            crate::src::core::SkPoint::SkPoint::make(x, y),
            bounds,
        );
        &self.fCurrentRunBuffer
    }
    // Source: src/core/SkTextBlob.cpp:577-589
    /// Safety: initialize all returned glyph/position/text views before another allocation, make, destruction or unwinding; font and bounds do not alias builder storage.
    pub unsafe fn allocRunTextPosH(
        &mut self,
        font: &crate::src::core::SkFont::SkFont,
        count: i32,
        y: f32,
        text_size: i32,
        bounds: Option<&SkRect>,
    ) -> &RunBuffer {
        self.allocInternal(
            font,
            GlyphPositioning::kHorizontal_Positioning,
            count,
            text_size,
            crate::src::core::SkPoint::SkPoint::make(0.0, y),
            bounds,
        );
        &self.fCurrentRunBuffer
    }
    // Source: src/core/SkTextBlob.cpp:591-601
    /// Safety: initialize all returned glyph/position/text views before another allocation, make, destruction or unwinding; font and bounds do not alias builder storage.
    pub unsafe fn allocRunTextPos(
        &mut self,
        font: &crate::src::core::SkFont::SkFont,
        count: i32,
        text_size: i32,
        bounds: Option<&SkRect>,
    ) -> &RunBuffer {
        self.allocInternal(
            font,
            GlyphPositioning::kFull_Positioning,
            count,
            text_size,
            crate::src::core::SkPoint::SkPoint::make(0.0, 0.0),
            bounds,
        );
        &self.fCurrentRunBuffer
    }
    // Source: src/core/SkTextBlob.cpp:603-614
    /// Safety: initialize all returned glyph/position/text views before another allocation, make, destruction or unwinding; font and bounds do not alias builder storage.
    pub unsafe fn allocRunTextRSXform(
        &mut self,
        font: &crate::src::core::SkFont::SkFont,
        count: i32,
        text_size: i32,
        bounds: Option<&SkRect>,
    ) -> &RunBuffer {
        self.allocInternal(
            font,
            GlyphPositioning::kRSXform_Positioning,
            count,
            text_size,
            crate::src::core::SkPoint::SkPoint::make(0.0, 0.0),
            bounds,
        );
        &self.fCurrentRunBuffer
    }
    // Source: src/core/SkTextBlob.cpp:616-658
    /// Safety: every allocated run has initialized glyph/position/text data. Returned owner
    /// consumes the malloc block; all earlier RunBuffer pointers must stop being used.
    pub unsafe fn make(&mut self) -> crate::include::core::SkRefCnt::sk_sp<SkTextBlob> {
        if self.fRunCount == 0 {
            debug_assert!(self.fStorage.is_null());
            crate::SkASSERT!(@source b"src/core/SkTextBlob.cpp\0".as_ptr(),620; @condition b"fStorageUsed == 0\0".as_ptr(); (self.fStorageUsed)==(0));
            crate::SkASSERT!(@source b"src/core/SkTextBlob.cpp\0".as_ptr(),621; @condition b"fStorageSize == 0\0".as_ptr(); (self.fStorageSize)==(0));
            crate::SkASSERT!(@source b"src/core/SkTextBlob.cpp\0".as_ptr(),622; @condition b"fLastRun == 0\0".as_ptr(); (self.fLastRun)==(0));
            crate::SkASSERT!(@source b"src/core/SkTextBlob.cpp\0".as_ptr(),623; @condition b"fBounds.isEmpty()\0".as_ptr(); self.fBounds.isEmpty());
            return crate::include::core::SkRefCnt::sk_sp::default();
        }
        self.updateDeferredBounds();
        let last = self.fStorage.add(self.fLastRun).cast::<RunRecord>();
        (*last).fFlags |= RunRecord::LAST_FLAG;
        let blob = self.fStorage.cast::<SkTextBlob>();
        self.fStorage = std::ptr::null_mut();
        SkTextBlob::init_at(blob, self.fBounds);
        #[cfg(sk_debug)]
        {
            (*blob).fStorageSize.write(self.fStorageSize);
            let mut safe = SkSafeMath::new();
            let mut size = blob_header_size();
            let mut run = RunRecord::First(blob);
            while !run.is_null() {
                size += RunRecord::StorageSize(
                    (*run).fCount,
                    (*run).textSize(),
                    (*run).positioning(),
                    &mut safe,
                );
                (*run).validate(blob.cast::<u8>().add(self.fStorageUsed));
                self.fRunCount -= 1;
                run = RunRecord::Next(run);
            }
            debug_assert_eq!(size, self.fStorageUsed);
            crate::SkASSERT!(@source b"src/core/SkTextBlob.cpp\0".as_ptr(),647; @condition b"fRunCount == 0\0".as_ptr(); (self.fRunCount)==(0));
            debug_assert!(safe.ok());
        }
        self.fStorageUsed = 0;
        self.fStorageSize = 0;
        self.fRunCount = 0;
        self.fLastRun = 0;
        self.fBounds.setEmpty();
        crate::include::core::SkRefCnt::sk_sp::from_raw(blob)
    }
}
// Pointer-alignment adapter for the source's repeated SkAlignPtr(sizeof(SkTextBlob)).
fn blob_header_size() -> usize {
    let align = std::mem::size_of::<usize>();
    (std::mem::size_of::<SkTextBlob>() + align - 1) & !(align - 1)
}

impl SkTextBlobPriv {
    // Source: src/core/SkTextBlob.cpp:662-702
    /// Safety: blob runs must be fully initialized and remain immutable during
    /// synchronous writer callbacks; callbacks obey their original leases.
    pub unsafe fn Flatten(blob: &SkTextBlob, buffer: &mut dyn SkWriteBuffer) {
        buffer.writeRect(blob.bounds());
        let mut it = SkTextBlobRunIterator::new(blob);
        while !it.done() {
            crate::SkASSERT!(@source b"src/core/SkTextBlob.cpp\0".as_ptr(),669; @condition b"it.glyphCount() > 0\0".as_ptr(); it.glyphCount() > 0);
            buffer.write32(it.glyphCount() as i32);
            let mut pe = PositioningAndExtended::from_int(0);
            pe.set_positioning(it.positioning() as u8);
            debug_assert_eq!(it.positioning() as i32, pe.int_value());
            let textSize = it.textSize();
            pe.set_extended(textSize > 0);
            buffer.write32(pe.int_value());
            if pe.extended() {
                buffer.write32(textSize as i32);
            }
            buffer.writePoint(it.offset());
            SkFontPriv::Flatten(it.font(), buffer);
            buffer.writeByteArray(
                it.glyphs().cast(),
                it.glyphCount() as usize * size_of::<u16>(),
            );
            buffer.writeByteArray(
                it.pos().cast(),
                it.glyphCount() as usize
                    * size_of::<f32>()
                    * SkTextBlob::ScalarsPerGlyph(it.positioning()) as usize,
            );
            if pe.extended() {
                buffer.writeByteArray(
                    it.clusters().cast(),
                    size_of::<u32>() * it.glyphCount() as usize,
                );
                buffer.writeByteArray(it.text(), it.textSize() as usize);
            }
            it.next();
        }
        buffer.write32(0);
    }

    // Source: src/core/SkTextBlob.cpp:704-791
    // Actual SkFontPriv::Unflatten -> readTypeface -> MemoryStream; unverified.
    /// Safety: all native typeface/scaler and callback leases remain live; no
    /// callbacks may unwind while a run has not been fully initialized.
    pub unsafe fn MakeFromBuffer(
        reader: &mut SkReadBuffer<'_>,
    ) -> crate::include::core::SkRefCnt::sk_sp<SkTextBlob> {
        use crate::include::core::SkRefCnt::sk_sp;
        let bounds = reader.readRect();
        let mut blobBuilder = SkTextBlobBuilder::new();
        let mut safe = SkSafeMath::new();
        loop {
            let glyphCount = reader.read32();
            if glyphCount == 0 {
                break;
            }
            let pe = PositioningAndExtended::from_int(reader.read32());
            let position = pe.positioning();
            if glyphCount <= 0 || position > GlyphPositioning::kRSXform_Positioning as u8 {
                return sk_sp::default();
            }
            let pos = match position {
                0 => GlyphPositioning::kDefault_Positioning,
                1 => GlyphPositioning::kHorizontal_Positioning,
                2 => GlyphPositioning::kFull_Positioning,
                3 => GlyphPositioning::kRSXform_Positioning,
                _ => unreachable!("validated native positioning"),
            };
            let textSize = if pe.extended() { reader.read32() } else { 0 };
            if textSize < 0 {
                return sk_sp::default();
            }
            let offset = reader.readPoint();
            let mut font = crate::src::core::SkFont::SkFont::default();
            SkFontPriv::Unflatten(&mut font, reader);
            let glyphSize = safe.mul(glyphCount as usize, size_of::<u16>());
            let scalarsSize = safe.mul(size_of::<f32>(), SkTextBlob::ScalarsPerGlyph(pos) as usize);
            let posSize = safe.mul(glyphCount as usize, scalarsSize);
            let clusterSize = if pe.extended() {
                safe.mul(glyphCount as usize, size_of::<u32>())
            } else {
                0
            };
            let glyphAndPosSize = safe.add(glyphSize, posSize);
            let clusterAndTextSize = safe.add(clusterSize, textSize as usize);
            let totalSize = safe.add(glyphAndPosSize, clusterAndTextSize);
            if !reader.isValid() || !safe.ok() || totalSize > reader.available() {
                return sk_sp::default();
            }
            let buf = *match pos {
                GlyphPositioning::kDefault_Positioning => blobBuilder.allocRunText(
                    &font,
                    glyphCount,
                    offset.fX,
                    offset.fY,
                    textSize,
                    Some(&bounds),
                ),
                GlyphPositioning::kHorizontal_Positioning => blobBuilder.allocRunTextPosH(
                    &font,
                    glyphCount,
                    offset.fY,
                    textSize,
                    Some(&bounds),
                ),
                GlyphPositioning::kFull_Positioning => {
                    blobBuilder.allocRunTextPos(&font, glyphCount, textSize, Some(&bounds))
                }
                GlyphPositioning::kRSXform_Positioning => {
                    blobBuilder.allocRunTextRSXform(&font, glyphCount, textSize, Some(&bounds))
                }
            };
            if buf.glyphs.is_null()
                || buf.pos.is_null()
                || (pe.extended() && (buf.clusters.is_null() || buf.utf8text.is_null()))
            {
                return sk_sp::default();
            }
            // Rust's &mut [u8] input requires initialized storage. Initialize the
            // actual run buffers before borrowing them; successful reads overwrite
            // every byte. Rejected runs are destroyed by the original builder.
            core::ptr::write_bytes(buf.glyphs.cast::<u8>(), 0, glyphSize);
            core::ptr::write_bytes(buf.pos.cast::<u8>(), 0, posSize);
            if pe.extended() {
                core::ptr::write_bytes(buf.clusters.cast::<u8>(), 0, clusterSize);
                core::ptr::write_bytes(buf.utf8text, 0, textSize as usize);
            }
            if !reader.readByteArray(core::slice::from_raw_parts_mut(
                buf.glyphs.cast(),
                glyphSize,
            )) || !reader.readByteArray(core::slice::from_raw_parts_mut(buf.pos.cast(), posSize))
            {
                return sk_sp::default();
            }
            if pe.extended() {
                if !reader.readByteArray(core::slice::from_raw_parts_mut(
                    buf.clusters.cast(),
                    clusterSize,
                )) || !reader.readByteArray(core::slice::from_raw_parts_mut(
                    buf.utf8text,
                    textSize as usize,
                )) {
                    return sk_sp::default();
                }
                for i in 0..glyphCount as usize {
                    if *buf.clusters.add(i) >= textSize as u32 {
                        return sk_sp::default();
                    }
                }
            }
        }
        blobBuilder.make()
    }
}

impl SkTextBlob {
    // Source: include/core/SkTextBlob.h:116-122
    /// string is null or NUL-terminated readable bytes for the synchronous
    /// font conversion. Native strlen counts bytes for every encoding option.
    pub unsafe fn MakeFromString(
        string: *const core::ffi::c_char,
        font: &crate::src::core::SkFont::SkFont,
        encoding: crate::include::core::SkFontTypes::SkTextEncoding,
    ) -> Result<crate::include::core::SkRefCnt::sk_sp<Self>, crate::RasterError> {
        if string.is_null() {
            return Ok(crate::include::core::SkRefCnt::sk_sp::default());
        }
        let len = unsafe { core::ffi::CStr::from_ptr(string) }
            .to_bytes()
            .len();
        unsafe { Self::MakeFromText(string.cast(), len, font, encoding) }
    }
    /// Native default encoding argument; same input lease as MakeFromString.
    pub unsafe fn MakeFromString_default(
        string: *const core::ffi::c_char,
        font: &crate::src::core::SkFont::SkFont,
    ) -> Result<crate::include::core::SkRefCnt::sk_sp<Self>, crate::RasterError> {
        unsafe {
            Self::MakeFromString(
                string,
                font,
                crate::include::core::SkFontTypes::SkTextEncoding::kUTF8,
            )
        }
    }
    // Source: include/core/SkTextBlob.h:162-167
    /// All input slices remain readable during native run conversion/copy.
    pub unsafe fn MakeFromPosHGlyphs(
        glyphs: &[u16],
        xpos: &[f32],
        const_y: f32,
        font: &crate::src::core::SkFont::SkFont,
    ) -> crate::include::core::SkRefCnt::sk_sp<Self> {
        unsafe {
            Self::MakeFromPosTextH(
                glyphs.as_ptr().cast(),
                core::mem::size_of_val(glyphs),
                xpos,
                const_y,
                font,
                crate::include::core::SkFontTypes::SkTextEncoding::kGlyphID,
            )
        }
    }
    // Source: include/core/SkTextBlob.h:168-172
    /// All input slices remain readable during native run conversion/copy.
    pub unsafe fn MakeFromPosGlyphs(
        glyphs: &[u16],
        pos: &[crate::src::core::SkPoint::SkPoint],
        font: &crate::src::core::SkFont::SkFont,
    ) -> crate::include::core::SkRefCnt::sk_sp<Self> {
        unsafe {
            Self::MakeFromPosText(
                glyphs.as_ptr().cast(),
                core::mem::size_of_val(glyphs),
                pos,
                font,
                crate::include::core::SkFontTypes::SkTextEncoding::kGlyphID,
            )
        }
    }
    // Source: include/core/SkTextBlob.h:173-178
    /// All input slices remain readable during native run conversion/copy.
    pub unsafe fn MakeFromRSXformGlyphs(
        glyphs: &[u16],
        xform: &[crate::src::core::SkRSXform::SkRSXform],
        font: &crate::src::core::SkFont::SkFont,
    ) -> crate::include::core::SkRefCnt::sk_sp<Self> {
        unsafe {
            Self::MakeFromRSXform(
                glyphs.as_ptr().cast(),
                core::mem::size_of_val(glyphs),
                xform,
                font,
                crate::include::core::SkFontTypes::SkTextEncoding::kGlyphID,
            )
        }
    }
}

impl SkTextBlob {
    /// Safety: text points to byte_length readable encoded bytes; typeface and glyph metrics
    /// retain their native synchronous leases for conversion, layout and bounds construction.
    // Source: src/core/SkTextBlob.cpp:793-806
    pub unsafe fn MakeFromText(
        text: *const std::ffi::c_void,
        byte_length: usize,
        font: &crate::src::core::SkFont::SkFont,
        encoding: crate::include::core::SkFontTypes::SkTextEncoding,
    ) -> Result<crate::include::core::SkRefCnt::sk_sp<Self>, crate::RasterError> {
        let count = font.countText(text, byte_length, encoding);
        if count == 0 {
            return Ok(crate::include::core::SkRefCnt::sk_sp::default());
        }
        let mut builder = SkTextBlobBuilder::new();
        let buffer = *builder.allocRunPos(font, count as i32, None);
        // Adaptation: native raw output storage is unspecified; Rust mutable
        // slices require initialized elements before entering the font API.
        std::ptr::write_bytes(buffer.glyphs, 0, count);
        std::ptr::write_bytes(buffer.points(), 0, count);
        let glyphs = std::slice::from_raw_parts_mut(buffer.glyphs, count);
        font.textToGlyphs(text, byte_length, encoding, glyphs);
        font.getPos(
            glyphs,
            std::slice::from_raw_parts_mut(buffer.points(), count),
            crate::src::core::SkPoint::SkPoint::make(0.0, 0.0),
        )?;
        Ok(builder.make())
    }
    /// Safety: same encoded-text/typeface leases as MakeFromText; positions remain readable.
    // Source: src/core/SkTextBlob.cpp:808-820
    pub unsafe fn MakeFromPosText(
        text: *const std::ffi::c_void,
        byte_length: usize,
        pos: &[crate::src::core::SkPoint::SkPoint],
        font: &crate::src::core::SkFont::SkFont,
        encoding: crate::include::core::SkFontTypes::SkTextEncoding,
    ) -> crate::include::core::SkRefCnt::sk_sp<Self> {
        let count = font.countText(text, byte_length, encoding);
        if count == 0 || pos.len() < count {
            return crate::include::core::SkRefCnt::sk_sp::default();
        }
        let mut builder = SkTextBlobBuilder::new();
        let buffer = *builder.allocRunPos(font, count as i32, None);
        // Initialize only the storage about to become a typed mutable output slice.
        std::ptr::write_bytes(buffer.glyphs, 0, count);
        font.textToGlyphs(
            text,
            byte_length,
            encoding,
            std::slice::from_raw_parts_mut(buffer.glyphs, count),
        );
        std::ptr::copy_nonoverlapping(pos.as_ptr(), buffer.points(), count);
        builder.make()
    }
    /// Safety: same encoded-text/typeface leases as MakeFromText; positions remain readable.
    // Source: src/core/SkTextBlob.cpp:822-834
    pub unsafe fn MakeFromPosTextH(
        text: *const std::ffi::c_void,
        byte_length: usize,
        xpos: &[f32],
        const_y: f32,
        font: &crate::src::core::SkFont::SkFont,
        encoding: crate::include::core::SkFontTypes::SkTextEncoding,
    ) -> crate::include::core::SkRefCnt::sk_sp<Self> {
        let count = font.countText(text, byte_length, encoding);
        if count == 0 || xpos.len() < count {
            return crate::include::core::SkRefCnt::sk_sp::default();
        }
        let mut builder = SkTextBlobBuilder::new();
        let buffer = *builder.allocRunPosH(font, count as i32, const_y, None);
        std::ptr::write_bytes(buffer.glyphs, 0, count);
        font.textToGlyphs(
            text,
            byte_length,
            encoding,
            std::slice::from_raw_parts_mut(buffer.glyphs, count),
        );
        std::ptr::copy_nonoverlapping(xpos.as_ptr(), buffer.pos, count);
        builder.make()
    }
    /// Safety: same encoded-text/typeface leases as MakeFromText; transforms remain readable.
    // Source: src/core/SkTextBlob.cpp:836-848
    pub unsafe fn MakeFromRSXform(
        text: *const std::ffi::c_void,
        byte_length: usize,
        xform: &[crate::src::core::SkRSXform::SkRSXform],
        font: &crate::src::core::SkFont::SkFont,
        encoding: crate::include::core::SkFontTypes::SkTextEncoding,
    ) -> crate::include::core::SkRefCnt::sk_sp<Self> {
        let count = font.countText(text, byte_length, encoding);
        if count == 0 || xform.len() < count {
            return crate::include::core::SkRefCnt::sk_sp::default();
        }
        let mut builder = SkTextBlobBuilder::new();
        let buffer = *builder.allocRunRSXform(font, count as i32);
        std::ptr::write_bytes(buffer.glyphs, 0, count);
        font.textToGlyphs(
            text,
            byte_length,
            encoding,
            std::slice::from_raw_parts_mut(buffer.glyphs, count),
        );
        std::ptr::copy_nonoverlapping(xform.as_ptr(), buffer.xforms(), count);
        builder.make()
    }
}

impl SkTextBlob {
    // Source: src/core/SkTextBlob.cpp:850-858
    /// Safety: same complete immutable blob and callback leases as Flatten.
    pub unsafe fn serialize(
        &self,
        procs: &crate::include::core::SkSerialProcs::SkSerialProcs,
    ) -> std::sync::Arc<crate::src::core::SkData::SkData> {
        let mut buffer = SkBinaryWriteBuffer::new(*procs);
        SkTextBlobPriv::Flatten(self, &mut buffer);
        let total = buffer.bytesWritten();
        let data = crate::src::core::SkData::SkData::MakeUninitialized(total);
        buffer.writeToMemory(data.writable_data());
        data
    }

    // Source: src/core/SkTextBlob.cpp:860-865
    // Uses the original reader's actual typeface stream producer; unverified.
    pub unsafe fn Deserialize(
        data: &[u8],
        procs: &crate::include::core::SkSerialProcs::SkDeserialProcs,
    ) -> crate::include::core::SkRefCnt::sk_sp<Self> {
        let mut buffer = SkReadBuffer::new(data);
        buffer.setDeserialProcs(*procs);
        SkTextBlobPriv::MakeFromBuffer(&mut buffer)
    }

    // Source: src/core/SkTextBlob.cpp:869-873
    /// Safety: memory names memory_size writable bytes, disjoint from all blob
    /// data and callback state, with the original writer's alignment.
    pub unsafe fn serialize_to_memory(
        &self,
        procs: &crate::include::core::SkSerialProcs::SkSerialProcs,
        memory: *mut u8,
        memory_size: usize,
    ) -> usize {
        let mut buffer = SkBinaryWriteBuffer::with_storage(memory, memory_size, *procs);
        SkTextBlobPriv::Flatten(self, &mut buffer);
        if buffer.usingInitialStorage() {
            buffer.bytesWritten()
        } else {
            0
        }
    }
}

// Source: src/core/SkTextBlob.cpp:878-929
// Adapted existing scaler Result and optional output slice; original null output
// counts intersections without writing. No default result hides a scaler failure.
unsafe fn get_glyph_run_intercepts(
    glyphRun: &crate::src::text::GlyphRun::GlyphRun,
    paint: &crate::src::core::SkPaint::SkPaint,
    bounds: &[f32; 2],
    mut intervals: Option<&mut [f32]>,
    intervalCount: &mut i32,
) -> Result<i32, crate::RasterError> {
    use crate::include::core::{SkFontTypes::SkFontHinting, SkRefCnt::sk_sp};
    use crate::src::core::{
        SkPaint::Style,
        SkStrikeSpec::{SkBulkGlyphMetricsAndPaths, SkStrikeSpec},
    };
    let mut scale = 1.0;
    let mut interceptPaint = paint.clone();
    let mut interceptFont = glyphRun.font().clone();
    interceptPaint.setMaskFilter(None);
    if interceptPaint.getPathEffect().is_none() {
        interceptFont.setHinting(SkFontHinting::kNone);
        interceptFont.setSubpixel(true);
        scale = interceptFont.getSize() / SkFontPriv::kCanonicalTextSizeForPaths as f32;
        interceptFont.setSize(SkFontPriv::kCanonicalTextSizeForPaths as f32);
        if interceptPaint.getStrokeWidth() > 0.0 && interceptPaint.getStyle() != Style::kFill_Style
        {
            interceptPaint.setStrokeWidth(interceptPaint.getStrokeWidth() / scale);
        }
    }
    interceptPaint.setStyle(Style::kFill_Style);
    interceptPaint.setPathEffect(sk_sp::default());
    let spec = SkStrikeSpec::MakeWithNoDevice(
        &interceptFont,
        Some(&interceptPaint),
        crate::src::core::SkScalerContext::SkScalerContextFlags::FAKE_GAMMA_AND_BOOST_CONTRAST,
    )?;
    let mut metricsAndPaths = SkBulkGlyphMetricsAndPaths::new(&spec);
    let mut posCursor = glyphRun.positions().as_ptr();
    // Retain the actual scratch span via pointer/length, without holding its
    // mutable method borrow while findIntercepts borrows the same owner.
    let glyphs = metricsAndPaths.glyphs(glyphRun.glyphsIDs());
    let glyph_ptr = glyphs.as_ptr();
    let glyph_count = glyphs.len();
    for index in 0..glyph_count {
        let glyph = glyph_ptr.add(index).read();
        let pos = posCursor.read();
        posCursor = posCursor.add(1);
        if glyph.as_ref().path().is_some() {
            let scaledBounds = [(bounds[0] - pos.fY) / scale, (bounds[1] - pos.fY) / scale];
            metricsAndPaths.findIntercepts(
                &scaledBounds,
                scale,
                pos.fX,
                glyph,
                intervals.as_deref_mut(),
                intervalCount,
            );
        }
    }
    Ok(*intervalCount)
}

impl SkTextBlob {
    // Source: src/core/SkTextBlob.cpp:932-953
    /// Safety: native font, blob and glyph path leases hold. Output, when present,
    /// has room for all intersections; no shared glyph mutation may overlap reads.
    pub unsafe fn getIntercepts(
        &self,
        bounds: &[f32; 2],
        mut intervals: Option<&mut [f32]>,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) -> Result<i32, crate::RasterError> {
        let defaultPaint = if paint.is_none() {
            Some(crate::src::core::SkPaint::SkPaint::default())
        } else {
            None
        };
        let paint = paint.or(defaultPaint.as_ref()).unwrap();
        let mut builder = Box::pin(crate::src::text::GlyphRun::GlyphRunBuilder::new());
        let list = builder
            .as_mut()
            .blobToGlyphRunList(self, crate::src::core::SkPoint::SkPoint::make(0.0, 0.0))?;
        let mut intervalCount = 0;
        for run in list.runs() {
            if run.scaledRotations().is_empty() {
                intervalCount = get_glyph_run_intercepts(
                    run,
                    paint,
                    bounds,
                    intervals.as_deref_mut(),
                    &mut intervalCount,
                )?;
            }
        }
        Ok(intervalCount)
    }
}

impl crate::src::core::SkFont::SkFont {
    // Source: src/core/SkTextBlob.cpp:955-974
    /// Safety: actual font/scaler leases hold through preparation and intersection
    /// calculation, including native path cache mutation.
    pub unsafe fn getIntercepts(
        &self,
        glyphs: &[u16],
        positions: &[crate::src::core::SkPoint::SkPoint],
        top: f32,
        bottom: f32,
        paintPtr: Option<&crate::src::core::SkPaint::SkPaint>,
    ) -> Result<Vec<f32>, crate::RasterError> {
        let count = glyphs.len().min(positions.len());
        if count == 0 {
            return Ok(Vec::new());
        }
        let paint = paintPtr.cloned().unwrap_or_default();
        let bounds = [top, bottom];
        let run = crate::src::text::GlyphRun::GlyphRun::new(self, positions, glyphs, &[], &[], &[]);
        let mut result = vec![0.0; count * 2];
        let mut intervalCount = 0;
        intervalCount =
            get_glyph_run_intercepts(&run, &paint, &bounds, Some(&mut result), &mut intervalCount)?;
        result.resize(intervalCount as usize, 0.0);
        Ok(result)
    }
}

impl Iter {
    // Source: src/core/SkTextBlob.cpp:978-980
    /// Safety: the blob and all its immutable run storage outlive this nonowning iterator.
    pub unsafe fn new(blob: &SkTextBlob) -> Self {
        Self {
            fRunRecord: RunRecord::First(blob),
        }
    }
}

impl Iter {
    // Source: src/core/SkTextBlob.cpp:982-987
    // Source: src/core/SkTextBlob.cpp:993-1002
    /// Safety: original blob remains alive; output pointers remain borrowed from its runs.
    pub unsafe fn next(&mut self, rec: Option<&mut Run>) -> bool {
        if !self.fRunRecord.is_null() {
            let run = &*self.fRunRecord;
            if let Some(rec) = rec {
                rec.fTypeface = run.font().getTypeface();
                rec.fGlyphCount = run.glyphCount() as i32;
                rec.fGlyphIndices = run.glyphBuffer();
                // Source: src/core/SkTextBlob.cpp:988-992
                #[cfg(SK_UNTIL_CRBUG_1187654_IS_FIXED)]
                {
                    rec.fClusterIndex_forTest = run.clusterBuffer();
                    rec.fUtf8Size_forTest = run.textSize() as i32;
                    rec.fUtf8_forTest = run.textBuffer().cast();
                }
            }
            self.fRunRecord = if run.isLastRun() {
                std::ptr::null()
            } else {
                RunRecord::Next(self.fRunRecord)
            };
            return true;
        }
        false
    }
}

impl Iter {
    // Source: src/core/SkTextBlob.cpp:1004-1020
    pub unsafe fn experimentalNext(&mut self, rec: Option<&mut ExperimentalRun>) -> bool {
        if !self.fRunRecord.is_null() {
            let run = &*self.fRunRecord;
            if let Some(rec) = rec {
                drop(std::mem::replace(&mut rec.font, run.font().clone()));
                rec.count = run.glyphCount() as i32;
                rec.glyphs = run.glyphBuffer();
                rec.positions = run.pointBuffer();
            }
            self.fRunRecord = if run.isLastRun() {
                std::ptr::null()
            } else {
                RunRecord::Next(self.fRunRecord)
            };
            return true;
        }
        false
    }
}
// Source: src/core/SkTextBlob.cpp:39-47
#[repr(C)]
struct RunFontStorageEquivalent {
    fSize: f32,
    fScaleX: f32,
    fTypeface: *mut core::ffi::c_void,
    fSkewX: f32,
    fFlags: u32,
}
const _: () = assert!(
    core::mem::size_of::<crate::src::core::SkFont::SkFont>()
        == core::mem::size_of::<RunFontStorageEquivalent>()
);
// Source: src/core/SkTextBlob.cpp:81-89,104-105
#[repr(C)]
struct RunRecordStorageEquivalent {
    fFont: crate::src::core::SkFont::SkFont,
    fOffset: crate::src::core::SkPoint::SkPoint,
    fCount: u32,
    fFlags: u32,
    #[cfg(sk_debug)]
    fMagic: u32,
}
const _: () = assert!(
    core::mem::size_of::<crate::src::core::SkTextBlobPriv::RunRecord>()
        == core::mem::size_of::<RunRecordStorageEquivalent>()
);
// Source: src/core/SkTextBlob.cpp:187-187
const _: () =
    assert!(core::mem::size_of::<PositioningAndExtended>() == core::mem::size_of::<i32>());
impl SkTextBlob {
    // Source: src/core/SkTextBlob.cpp:213-215
    fn operator_new(_size: usize) -> *mut core::ffi::c_void {
        let _ = std::io::Write::write_all(
            &mut std::io::stderr(),
            b"All blobs are created by placement new.\n",
        );
        std::process::abort()
    }
    // Source: src/core/SkTextBlob.cpp:217-219
    unsafe fn operator_new_placement(
        _size: usize,
        p: *mut core::ffi::c_void,
    ) -> *mut core::ffi::c_void {
        p
    }
}
