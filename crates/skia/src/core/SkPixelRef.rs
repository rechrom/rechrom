// Copyright 2011 Google Inc. BSD-style license.
use std::sync::atomic::{AtomicU32, Ordering};
// Source: include/core/SkPixelRef.h:29-29
// Native public class name aliases its sole real storage, without a second owner.
pub type SkPixelRef = SkPixelRefStorage;
// The native external-synchronization contract does not prove arbitrary release
// contexts Send/Sync. This complete owner supplies no blanket thread qualification.
impl SkPixelRef {
    // Source: include/core/SkPixelRef.h:34-38
    pub fn dimensions(&self) -> crate::include::core::SkSize::SkISize {
        crate::include::core::SkSize::SkISize::Make(
            self.dimensionsFields.fWidth,
            self.dimensionsFields.fHeight,
        )
    }
}
impl SkPixelRef {
    pub fn width(&self) -> i32 {
        self.dimensionsFields.fWidth
    }
}
impl SkPixelRef {
    pub fn height(&self) -> i32 {
        self.dimensionsFields.fHeight
    }
}
impl SkPixelRef {
    pub fn pixels(&self) -> *mut c_void {
        self.dimensionsFields.fPixels
    }
}
impl SkPixelRef {
    pub fn rowBytes(&self) -> usize {
        self.dimensionsFields.fRowBytes
    }
}
impl SkPixelRef {
    // Source: include/core/SkPixelRef.h:56-56
    pub fn isImmutable(&self) -> bool {
        self.fMutability.get() != 0
    }
}
impl SkPixelRef {
    // Source: include/core/SkPixelRef.h:77-79
    pub fn notifyAddedToCache(&self) {
        self.fAddedToCache.store(true, Ordering::SeqCst);
    }
}
// Source: include/core/SkPixelRef.h:81-81
// The actual diagnostic virtual default is null. This is a genuine virtual
// contract alongside the original base owner, not a discardable-memory producer.
pub trait SkPixelRefVirtuals {
    fn diagnostic_only_getDiscardable(
        &self,
    ) -> Option<
        core::ptr::NonNull<
            dyn crate::include::private::chromium::SkDiscardableMemory::SkDiscardableMemory,
        >,
    > {
        None
    }
}
// The actual non-derived PixelRef inherits the native base default unchanged.
impl SkPixelRefVirtuals for SkPixelRef {}
// Source: include/core/SkPixelRef.h:87-90
// Actual native dimensions/pixel fields stored exactly once in this owned group;
// the grouping preserves source order around native inline genIDIsUnique().
struct PixelRefDimensions {
    fWidth: i32,
    fHeight: i32,
    fPixels: *mut c_void,
    fRowBytes: usize,
}
impl SkPixelRef {
    // Source: include/core/SkPixelRef.h:93-93
    fn genIDIsUnique(&self) -> bool {
        self.fTaggedGenID.load(Ordering::SeqCst) & 1 != 0
    }
}
// Source: include/core/SkPixelRef.h:94-94
// Source: include/core/SkPixelRef.h:96-96
// Source: include/core/SkPixelRef.h:99-99
// Source: include/core/SkPixelRef.h:101-105
// Arc supplies native ref ownership. The same release state remains in this owner.
// Cell maps the original non-atomic byte, without adding a synchronization policy.
// Its mutation is confined to the original owner/COW operation; no Sync promise is made.
#[doc(hidden)]
pub struct SkPixelRefStorage {
    // Source: src/core/SkPixelRef.cpp:145-150
    // Derived release state is consumed by Drop before the original PixelRef
    // destructor body. Native nontrivial members then die in reverse order,
    // and the actual PixelStorage parent is last. These are the same fields,
    // reordered for Rust's declaration-order field teardown, without a proxy.
    release: Option<(unsafe extern "C" fn(*mut c_void, *mut c_void), *mut c_void)>,
    // Source: include/core/SkPixelRef.h:101-105
    fMutability: core::cell::Cell<u8>,
    // Source: include/core/SkPixelRef.h:99-99
    fAddedToCache: AtomicBool,
    // Source: include/core/SkPixelRef.h:96-96
    fGenIDChangeListeners: List,
    // Source: include/core/SkPixelRef.h:94-94
    fTaggedGenID: AtomicU32,
    // Source: include/core/SkPixelRef.h:87-90
    dimensionsFields: PixelRefDimensions,
    // Source: include/core/SkPixelRef.h:29-29
    pub storage: SkPixelStorage,
}
// Source: src/core/SkPixelRef.cpp:20-20
// Source: src/core/SkPixelRef.cpp:22-22
// Source: src/core/SkPixelRef.cpp:24-29
impl crate::src::core::SkNextID::SkNextID {
    pub fn ImageID() -> u32 {
        static NEXT_ID: AtomicU32 = AtomicU32::new(2);
        loop {
            let id = NEXT_ID.fetch_add(2, Ordering::Relaxed);
            if id != 0 {
                return id;
            }
        }
    }
}
use crate::src::core::SkIDChangeListener::{List, SkIDChangeListener};
use crate::src::core::SkPixelStorage::{SkPixelStorage, Type};
use std::ffi::c_void;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
// Source: src/core/SkPixelRef.cpp:33-43
impl SkPixelRef {
    pub fn new(width: i32, height: i32, pixels: *mut c_void, rowBytes: usize) -> Self {
        let mut result = Self {
            storage: SkPixelStorage::new(Type::kPixelRef),
            dimensionsFields: PixelRefDimensions {
                fWidth: width,
                fHeight: height,
                fPixels: pixels,
                fRowBytes: rowBytes,
            },
            fTaggedGenID: AtomicU32::new(0),
            fGenIDChangeListeners: List::default(),
            fAddedToCache: AtomicBool::new(false),
            fMutability: core::cell::Cell::new(0),
            release: None,
        };
        result.needsNewGenID();
        result.fMutability.set(0);
        result
    }
}
// Source: src/core/SkPixelRef.cpp:45-47
impl Drop for SkPixelRef {
    fn drop(&mut self) {
        if let Some((release, ctx)) = self.release.take() {
            unsafe {
                release(self.dimensionsFields.fPixels, ctx);
            }
        }
        self.callGenIDChangeListeners();
    }
}
impl SkPixelRef {
    // Source: src/core/SkPixelRef.cpp:50-53
    // Source: src/core/SkPixelRef.cpp:57-58
    pub fn android_only_reset(&mut self, width: i32, height: i32, rowBytes: usize) {
        self.dimensionsFields.fWidth = width;
        self.dimensionsFields.fHeight = height;
        self.dimensionsFields.fRowBytes = rowBytes;
        self.notifyPixelsChanged();
    }
}
impl SkPixelRef {
    // Source: src/core/SkPixelRef.cpp:60-63
    fn needsNewGenID(&self) {
        self.fTaggedGenID.store(0, Ordering::SeqCst);
        crate::SkASSERT!(@source b"src/core/SkPixelRef.cpp\0".as_ptr(),62; @condition b"!this->genIDIsUnique()\0".as_ptr(); !self.genIDIsUnique());
    }
}
impl SkPixelRef {
    // Source: src/core/SkPixelRef.cpp:65-71
    // Source: src/core/SkPixelRef.cpp:73-73
    // Source: src/core/SkPixelRef.cpp:76-78
    pub fn getGenerationID(&self) -> u32 {
        let mut id = self.fTaggedGenID.load(Ordering::SeqCst);
        if id == 0 {
            let next = crate::src::core::SkNextID::SkNextID::ImageID() | 1;
            id = match self.fTaggedGenID.compare_exchange(
                id,
                next,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => next,
                Err(winner) => winner,
            };
        }
        id & !1
    }
}
impl SkPixelRef {
    // Source: src/core/SkPixelRef.cpp:80-81
    // Source: src/core/SkPixelRef.cpp:83-87
    pub fn addGenIDChangeListener(&self, listener: Option<Arc<dyn SkIDChangeListener>>) {
        let Some(listener) = listener else {
            return;
        };
        if !self.genIDIsUnique() {
            return;
        }
        crate::SkASSERT!(@source b"src/core/SkPixelRef.cpp\0".as_ptr(),85; @condition b"!listener->shouldDeregister()\0".as_ptr(); !listener.shouldDeregister());
        self.fGenIDChangeListeners.add(Some(listener));
    }
}
impl SkPixelRef {
    // Source: src/core/SkPixelRef.cpp:90-90
    // Source: src/core/SkPixelRef.cpp:92-97
    // Source: src/core/SkPixelRef.cpp:100-102
    fn callGenIDChangeListeners(&self) {
        if self.genIDIsUnique() {
            self.fGenIDChangeListeners.changed();
            if self.fAddedToCache.swap(false, Ordering::SeqCst) {
                crate::src::core::SkBitmapCache::SkNotifyBitmapGenIDIsStale(self.getGenerationID());
            }
        } else {
            self.fGenIDChangeListeners.reset();
        }
    }
}
impl SkPixelRef {
    // Source: src/core/SkPixelRef.cpp:104-104
    // Source: src/core/SkPixelRef.cpp:106-108
    // Source: src/core/SkPixelRef.cpp:110-112
    pub fn notifyPixelsChanged(&self) {
        #[cfg(sk_debug)]
        if self.isImmutable() {
            unsafe {
                crate::SkDebugf!(@source b"src/core/SkPixelRef.cpp\0".as_ptr(),107;b"========== notifyPixelsChanged called on immutable pixelref\0".as_ptr());
            }
        }
        self.callGenIDChangeListeners();
        self.needsNewGenID();
    }
}
impl SkPixelRef {
    // Source: src/core/SkPixelRef.cpp:114-116
    pub fn setImmutable(&self) {
        self.fMutability.set(2);
    }
}
impl SkPixelRef {
    // Source: src/core/SkPixelRef.cpp:118-118
    // Source: src/core/SkPixelRef.cpp:125-127
    pub fn setImmutableWithID(&self, genID: u32) {
        self.fMutability.set(2);
        self.fTaggedGenID.store(genID, Ordering::SeqCst);
    }
}
impl SkPixelRef {
    // Source: src/core/SkPixelRef.cpp:129-132
    pub fn setTemporarilyImmutable(&self) {
        crate::SkASSERT!(@source b"src/core/SkPixelRef.cpp\0".as_ptr(), 130; @condition b"fMutability != kImmutable\0".as_ptr(); self.fMutability.get() != 2);
        self.fMutability.set(1);
    }
}
impl SkPixelRef {
    // Source: src/core/SkPixelRef.cpp:134-137
    pub fn restoreMutability(&self) {
        crate::SkASSERT!(@source b"src/core/SkPixelRef.cpp\0".as_ptr(), 135; @condition b"fMutability != kImmutable\0".as_ptr(); self.fMutability.get() != 2);
        self.fMutability.set(0);
    }
}
// Source: src/core/SkPixelRef.cpp:139-153
/// Safety: a supplied release callback/context must remain valid and own exactly the
/// release responsibility passed here until the final shared PixelRef reference drops.
/// The callback must be safe to run on that final owner's thread.
pub unsafe fn SkMakePixelRefWithProc(
    width: i32,
    height: i32,
    rowBytes: usize,
    addr: *mut c_void,
    releaseProc: Option<unsafe extern "C" fn(*mut c_void, *mut c_void)>,
    ctx: *mut c_void,
) -> Arc<SkPixelRef> {
    crate::SkASSERT!(@source b"src/core/SkPixelRef.cpp\0".as_ptr(), 141; @condition b"width >= 0 && height >= 0\0".as_ptr(); width >= 0 && height >= 0);
    let mut pixelref = SkPixelRef::new(width, height, addr, rowBytes);
    if let Some(release) = releaseProc {
        pixelref.release = Some((release, ctx));
    }
    Arc::new(pixelref)
}
impl crate::src::core::SkPixelStorage::SkPixelStorageObject for SkPixelRef {
    fn pixel_storage(&self) -> &crate::src::core::SkPixelStorage::SkPixelStorage {
        &self.storage
    }
}
