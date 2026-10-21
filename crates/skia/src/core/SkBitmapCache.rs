// Copyright 2014 Google Inc. BSD-style license.
use crate::src::core::SkRect::SkIRect;
// Source: src/core/SkBitmapCache.h:29-31
#[derive(Clone, Copy, Debug)]
pub struct SkBitmapCacheDesc {
    pub fImageID: u32,
    pub fSubset: SkIRect,
}
// Source: src/core/SkBitmapCache.h:33-37
impl SkBitmapCacheDesc {
    pub fn validate(&self) {
        crate::SkASSERT!(@source b"src/core/SkBitmapCache.h\0".as_ptr(),34; @condition b"fImageID\0".as_ptr(); self.fImageID!=0);
        crate::SkASSERT!(@source b"src/core/SkBitmapCache.h\0".as_ptr(),35; @condition b"fSubset.fLeft >= 0 && fSubset.fTop >= 0\0".as_ptr(); self.fSubset.fLeft>=0&&self.fSubset.fTop>=0);
        crate::SkASSERT!(@source b"src/core/SkBitmapCache.h\0".as_ptr(),36; @condition b"fSubset.width() > 0 && fSubset.height() > 0\0".as_ptr(); self.fSubset.width()>0&&self.fSubset.height()>0);
    }
}
// Source: src/core/SkBitmapCache.h:62-62
// Namespace-only class. Lookup exists on the verified native ARM64/64-bit budget configuration.
pub struct SkMipmapCache;
// Source: src/core/SkBitmapCache.cpp:33-36
pub fn SkMakeResourceCacheSharedIDForBitmap(bitmapGenID: u32) -> u64 {
    // Scalar expansion of SkSetFourByteTag('b','m','a','p'), retaining big-endian tag encoding.
    let sharedID = u32::from_be_bytes(*b"bmap") as u64;
    (sharedID << 32) | bitmapGenID as u64
}
// Source: src/core/SkBitmapCache.cpp:38-40
pub fn SkNotifyBitmapGenIDIsStale(bitmapGenID: u32) {
    super::SkResourceCache::PostPurgeSharedID(SkMakeResourceCacheSharedIDForBitmap(bitmapGenID));
}
// Source: src/core/SkBitmapCache.cpp:44-48
impl SkBitmapCacheDesc {
    pub fn Make(imageID: u32, subset: &SkIRect) -> Self {
        crate::SkASSERT!(@source b"src/core/SkBitmapCache.cpp\0".as_ptr(),45; @condition b"imageID\0".as_ptr(); imageID!=0);
        crate::SkASSERT!(@source b"src/core/SkBitmapCache.cpp\0".as_ptr(),46; @condition b"subset.width() > 0 && subset.height() > 0\0".as_ptr(); subset.width()>0&&subset.height()>0);
        Self {
            fImageID: imageID,
            fSubset: *subset,
        }
    }
}

// Source: src/core/SkBitmapCache.cpp:50-53
impl SkBitmapCacheDesc {
    pub fn MakeFromImage(image: &dyn crate::src::image::SkImage::SkImage) -> Self {
        let bounds = SkIRect::make_wh(image.width(), image.height());
        Self::Make(image.uniqueID(), &bounds)
    }
}

// Source: src/core/SkBitmapCache.cpp:71-73
pub fn SkBitmapCache_setImmutableWithID(pr: &crate::src::core::SkPixelRef::SkPixelRef, id: u32) {
    pr.setImmutableWithID(id);
}

// Source: src/core/SkBitmapCache.h:42-42
pub struct SkBitmapCache;

// The cache owns the original stable boxed Rec. Native callbacks carry its raw
// identity, not a retained state owner. Exact contended final-unlock backend
// lifetime remains an unsafe native precondition, recorded separately as pending.
#[cfg(all(target_arch = "aarch64", target_pointer_width = "64"))]
mod bitmap_cache {
    use super::*;
    use crate::include::private::chromium::SkDiscardableMemory::SkDiscardableMemory;
    use crate::include::private::SkMutex::SkMutex;
    use crate::src::core::SkResourceCache::{self, Key, Rec, RecLinks};
    use crate::src::core::{SkBitmap::SkBitmap, SkImageInfo::SkImageInfo, SkPixmap::SkPixmap};
    use core::{cell::UnsafeCell, ptr::NonNull};
    use std::ffi::{c_char, c_void};
    unsafe extern "C" {
        fn malloc(size: usize) -> *mut c_void;
        fn free(pointer: *mut c_void);
    }
    // Source: src/core/SkBitmapCache.cpp:55-67
    static gBitmapKeyNamespaceLabel: u32 = 0;
    struct BitmapKey {
        key: Key,
        fDesc: SkBitmapCacheDesc,
    }
    impl BitmapKey {
        fn new(desc: &SkBitmapCacheDesc) -> Self {
            Self {
                fDesc: *desc,
                key: Key::new(
                    core::ptr::addr_of!(gBitmapKeyNamespaceLabel).cast(),
                    SkMakeResourceCacheSharedIDForBitmap(desc.fImageID),
                    &[
                        desc.fImageID,
                        desc.fSubset.fLeft as u32,
                        desc.fSubset.fTop as u32,
                        desc.fSubset.fRight as u32,
                        desc.fSubset.fBottom as u32,
                    ],
                ),
            }
        }
    }
    // Source: src/core/SkBitmapCache.cpp:75-90
    // Native fields remain in this one Rec allocation; UnsafeCell grants only
    // the mutex-protected state projection, never a whole mutable Rec alias.
    pub struct BitmapCacheRec {
        state: NativeState,
        fMutex: SkMutex,
        fKey: BitmapKey,
        fRowBytes: usize,
        fPrUniqueID: u32,
        links: RecLinks,
    }
    impl BitmapCacheRec {
        fn new(
            desc: &SkBitmapCacheDesc,
            info: &SkImageInfo,
            row_bytes: usize,
            memory: Option<Box<dyn SkDiscardableMemory>>,
            block: *mut c_void,
        ) -> Self {
            crate::SkASSERT!(@source b"src/core/SkBitmapCache.cpp\0".as_ptr(),85; @condition b"!(fDM && fMalloc)\0".as_ptr(); memory.is_none()||block.is_null());
            // Preserve native member-constructor evaluation: key, mutex,
            // discardable/malloc state, then image info. Private field order
            // below separately expresses destructor-body/reverse-member order.
            Self {
                fKey: BitmapKey::new(desc),
                fMutex: SkMutex::new(),
                state: NativeState {
                    mutable: UnsafeCell::new(MutableState {
                        fDM: memory,
                        fMalloc: block,
                        fExternalCounter: 0,
                        fDiscardableIsLocked: true,
                    }),
                    fInfo: info.clone(),
                },
                fRowBytes: row_bytes,
                fPrUniqueID: crate::src::core::SkNextID::SkNextID::ImageID(),
                links: RecLinks::new(),
            }
        }
    }
    // Source: src/core/SkBitmapCache.cpp:92-99
    impl Drop for NativeState {
        fn drop(&mut self) {
            let this = self.mutable.get_mut();
            crate::SkASSERT!(@source b"src/core/SkBitmapCache.cpp\0".as_ptr(),93; @condition b"0 == fExternalCounter\0".as_ptr(); 0==this.fExternalCounter);
            if this.fDiscardableIsLocked {
                if let Some(memory) = this.fDM.as_mut() {
                    crate::SkASSERT!(@source b"src/core/SkBitmapCache.cpp\0".as_ptr(),95; @condition b"fDM->data()\0".as_ptr(); !memory.data().is_null());
                    memory.unlock();
                }
            }
            unsafe {
                free(this.fMalloc);
            }
        }
    }
    impl Drop for BitmapCacheRec {
        fn drop(&mut self) {
            debug_assert_eq!(self.state.mutable.get_mut().fExternalCounter, 0);
        }
    }
    // Cache owns links; external callbacks touch only mutex-protected state.
    unsafe impl Rec for BitmapCacheRec {
        fn links(&self) -> &RecLinks {
            &self.links
        }
        fn links_mut(&mut self) -> &mut RecLinks {
            &mut self.links
        }
        // Source: src/core/SkBitmapCache.cpp:101-101
        fn getKey(&self) -> &Key {
            &self.fKey.key
        }
        // Source: src/core/SkBitmapCache.cpp:102-104
        fn bytesUsed(&self) -> usize {
            // Original BitmapKey==48 bytes verified against original .cpp with
            // clang++ static_assert on this native ARM64/64-bit ABI. Rust owner
            // allocation overhead is explicitly not the native budget footprint.
            48 + self.state.fInfo.computeByteSize(self.fRowBytes)
        }
        // Source: src/core/SkBitmapCache.cpp:105-108
        fn canBePurged(&self) -> bool {
            unsafe {
                let mutex = NonNull::new_unchecked(core::ptr::addr_of!(self.fMutex).cast_mut());
                SkMutex::acquire_raw(mutex);
                let result = (*self.state.mutable.get()).fExternalCounter == 0;
                SkMutex::release_raw(mutex);
                result
            }
        }
        // Source: src/core/SkBitmapCache.cpp:109-111
        unsafe fn postAddInstall(&self, bitmap: *mut c_void) {
            crate::SkAssertResult!(@source b"src/core/SkBitmapCache.cpp\0".as_ptr(),110;
                @condition b"this->install(static_cast<SkBitmap*>(payload))\0".as_ptr();
                unsafe{self.install(&mut *bitmap.cast::<SkBitmap>())});
        }
        // Source: src/core/SkBitmapCache.cpp:113-113
        fn getCategory(&self) -> *const c_char {
            c"bitmap".as_ptr()
        }
        // Source: src/core/SkBitmapCache.cpp:114-116
        fn diagnostic_only_getDiscardable(
            &self,
        ) -> Option<core::ptr::NonNull<dyn SkDiscardableMemory>> {
            // Native diagnostics inspect only the owned DM pointer under the
            // cache-operation lease; no whole mutable state borrow is created.
            let memory = unsafe { &*core::ptr::addr_of!((*self.state.mutable.get()).fDM) };
            memory
                .as_ref()
                .map(|memory| core::ptr::NonNull::from(memory.as_ref()))
        }
    }
    impl BitmapCacheRec {
        // Source: src/core/SkBitmapCache.cpp:118-133
        unsafe extern "C" fn ReleaseProc(_address: *mut c_void, context: *mut c_void) {
            // Native unchecked ctx: cache owns this stable Rec and its external
            // count prevents deletion until the final mutex-release publication.
            let rec = context.cast::<Self>();
            let mutex =
                unsafe { NonNull::new_unchecked(core::ptr::addr_of!((*rec).fMutex).cast_mut()) };
            unsafe {
                SkMutex::acquire_raw(mutex);
            }
            {
                let state = unsafe { &mut *(*rec).state.mutable.get() };
                crate::SkASSERT!(@source b"src/core/SkBitmapCache.cpp\0".as_ptr(),122; @condition b"rec->fExternalCounter > 0\0".as_ptr(); state.fExternalCounter>0);
                state.fExternalCounter -= 1;
                if state.fDM.is_some() {
                    crate::SkASSERT!(@source b"src/core/SkBitmapCache.cpp\0".as_ptr(),125; @condition b"rec->fMalloc == nullptr\0".as_ptr(); state.fMalloc.is_null());
                    if state.fExternalCounter == 0 {
                        state.fDM.as_mut().unwrap().unlock();
                        state.fDiscardableIsLocked = false;
                    }
                } else {
                    crate::SkASSERT!(@source b"src/core/SkBitmapCache.cpp\0".as_ptr(),131; @condition b"rec->fMalloc != nullptr\0".as_ptr(); !state.fMalloc.is_null());
                }
            }
            // No Rec/state/mutex receiver borrow survives publication and no
            // subsequent Rec access occurs. Native contended OS backend lifetime
            // through signal return is still an unchecked pending precondition.
            unsafe {
                SkMutex::release_raw(mutex);
            }
        }
        // Source: src/core/SkBitmapCache.cpp:135-159
        /// Safety: owning cache lock keeps this original stable Rec alive through
        /// the complete call. Replaced PixelRef release must not reenter this Rec
        /// mutex. All native allocation/release callbacks are nonunwinding.
        unsafe fn install(&self, bitmap: &mut SkBitmap) -> bool {
            let mutex =
                unsafe { NonNull::new_unchecked(core::ptr::addr_of!(self.fMutex).cast_mut()) };
            unsafe {
                SkMutex::acquire_raw(mutex);
            }
            let state = unsafe { &mut *self.state.mutable.get() };
            if state.fDM.is_none() && state.fMalloc.is_null() {
                unsafe {
                    SkMutex::release_raw(mutex);
                }
                return false;
            }
            if let Some(memory) = state.fDM.as_mut() {
                if !state.fDiscardableIsLocked {
                    crate::SkASSERT!(@source b"src/core/SkBitmapCache.cpp\0".as_ptr(),144; @condition b"fExternalCounter == 0\0".as_ptr(); state.fExternalCounter==0);
                    if !memory.lock() {
                        state.fDM = None;
                        unsafe {
                            SkMutex::release_raw(mutex);
                        }
                        return false;
                    }
                    state.fDiscardableIsLocked = true;
                }
                crate::SkASSERT!(@source b"src/core/SkBitmapCache.cpp\0".as_ptr(),151; @condition b"fDM->data()\0".as_ptr(); !memory.data().is_null());
            }
            let pixels = if let Some(memory) = state.fDM.as_mut() {
                memory.data().cast()
            } else {
                state.fMalloc
            };
            // Original order: install with raw Rec context while holding fMutex,
            // set immutable ID, then increment the native external counter.
            let _ = unsafe {
                bitmap.installPixels(
                    &self.state.fInfo,
                    pixels,
                    self.fRowBytes,
                    Some(Self::ReleaseProc),
                    core::ptr::from_ref(self).cast_mut().cast(),
                )
            };
            let pixel_ref = unsafe { bitmap.pixelRef().unwrap_unchecked() };
            SkBitmapCache_setImmutableWithID(pixel_ref, self.fPrUniqueID);
            state.fExternalCounter += 1;
            unsafe {
                SkMutex::release_raw(mutex);
            }
            true
        }
        // Source: src/core/SkBitmapCache.cpp:161-165
        unsafe fn Finder(base: &dyn Rec, bitmap: *mut c_void) -> bool {
            let rec = (base as &dyn std::any::Any)
                .downcast_ref::<Self>()
                .unwrap_or_else(|| std::process::abort());
            unsafe { rec.install(&mut *bitmap.cast::<SkBitmap>()) }
        }
    }
    // Source: src/core/SkBitmapCache.cpp:167-184
    // Source: src/core/SkBitmapCache.cpp:167-184
    // Adapted private grouping: Drop runs the original destructor body, then
    // fInfo dies before fDM. Mutable fields remain a separate UnsafeCell so an
    // immutable info query never overlaps a whole mutable state lease.
    struct NativeState {
        // Source: src/core/SkBitmapCache.cpp:176-176
        fInfo: SkImageInfo,
        mutable: UnsafeCell<MutableState>,
    }
    struct MutableState {
        fDM: Option<Box<dyn SkDiscardableMemory>>,
        fMalloc: *mut c_void,
        fExternalCounter: i32,
        fDiscardableIsLocked: bool,
    }
    // The native mutex synchronizes the sole Rec-owned storage and counter.
    // Callbacks carry unchecked Rec identity; there is no extra retained owner.
    unsafe impl Send for NativeState {}
    // Source: src/core/SkBitmapCache.h:51-53
    pub type BitmapRecPtr = Box<BitmapCacheRec>;
    impl SkBitmapCache {
        // Source: src/core/SkBitmapCache.cpp:186-186
        pub fn PrivateDeleteRec(rec: BitmapRecPtr) {
            drop(rec);
        }
        // Source: src/core/SkBitmapCache.cpp:188-214
        /// Safety: desc/info obey the native source/subset contract. Alloc's raw
        /// writable pixmap borrows this Rec; initialize pixels before Add and end
        /// all writes before installing immutable PixelRefs. Factory returns real
        /// locked discardable storage, with nonunwinding native memory methods.
        pub unsafe fn Alloc(
            desc: &SkBitmapCacheDesc,
            info: &SkImageInfo,
            pixmap: &mut SkPixmap,
        ) -> Option<BitmapRecPtr> {
            crate::SkASSERT!(@source b"src/core/SkBitmapCache.cpp\0".as_ptr(),191; @condition b"info.width() == desc.fSubset.width()\0".as_ptr(); info.width()==desc.fSubset.width());
            crate::SkASSERT!(@source b"src/core/SkBitmapCache.cpp\0".as_ptr(),192; @condition b"info.height() == desc.fSubset.height()\0".as_ptr(); info.height()==desc.fSubset.height());
            let row_bytes = info.minRowBytes();
            let size = info.computeByteSize(row_bytes);
            if SkImageInfo::ByteSizeOverflowed(size) {
                return None;
            }
            let mut memory = None;
            let mut block = core::ptr::null_mut();
            if let Some(factory) = SkResourceCache::GetDiscardableFactory() {
                memory = unsafe { factory(size) };
            } else {
                block = unsafe { malloc(size) };
            }
            if memory.is_none() && block.is_null() {
                return None;
            }
            let pixels = if let Some(memory) = memory.as_mut() {
                memory.data().cast()
            } else {
                block
            };
            *pixmap = SkPixmap::new(info, pixels.cast(), row_bytes);
            Some(Box::new(BitmapCacheRec::new(
                desc, info, row_bytes, memory, block,
            )))
        }
        // Source: src/core/SkBitmapCache.cpp:216-218
        /// Safety: rec's pixels are initialized and all Alloc raw writers ended;
        /// bitmap is a live uniquely mutable output, with native release callbacks
        /// that cannot reenter the global cache. No pending Rec borrow escapes.
        pub unsafe fn Add(rec: BitmapRecPtr, bitmap: &mut SkBitmap) {
            unsafe {
                SkResourceCache::Add(rec, core::ptr::from_mut(bitmap).cast());
            }
        }
        // Source: src/core/SkBitmapCache.cpp:220-223
        /// Safety: bitmap is an exclusive output; old release callbacks obey the
        /// global cache nonreentry contract, and all installed pixels stay readonly.
        pub unsafe fn Find(desc: &SkBitmapCacheDesc, bitmap: &mut SkBitmap) -> bool {
            desc.validate();
            let key = BitmapKey::new(desc);
            unsafe {
                SkResourceCache::Find(
                    &key.key,
                    BitmapCacheRec::Finder,
                    core::ptr::from_mut(bitmap).cast(),
                )
            }
        }
    }
}
#[cfg(all(target_arch = "aarch64", target_pointer_width = "64"))]
pub use bitmap_cache::{BitmapCacheRec, BitmapRecPtr};

// Source: src/core/SkBitmapCache.cpp:228-229
// Rust Option borrows select the original local cache or the real global cache;
// neither branch creates a cache or retains a replacement owner.
macro_rules! CHECK_LOCAL{
    ($cache:expr,$local:ident,$global:ident $(,$arg:expr)* $(,)?)=>{
        match $cache{
            Some(cache)=>cache.$local($($arg),*),
            None=>self::SkResourceCache::$global($($arg),*),
        }
    };
}

// The adapted Key allocation has no C++ ABI identity. Native sizeof(MipMapKey)==48
// was measured by including the original SkBitmapCache.cpp on ARM64/64-bit.
// Keep that exact native budget; other ABI footprints are pending verification.
#[cfg(all(target_arch = "aarch64", target_pointer_width = "64"))]
mod mipmap_cache {
    use super::*;
    use crate::src::core::SkMipmap::SkMipmap;
    use crate::src::core::SkResourceCache::SkResourceCache as ResourceCache;
    use crate::src::core::SkResourceCache::{self, Key, Rec, RecLinks};
    use std::ffi::{c_char, c_void};
    use std::mem::ManuallyDrop;

    // Source: src/core/SkBitmapCache.cpp:232-232
    static gMipMapKeyNamespaceLabel: u32 = 0;
    // Source: src/core/SkBitmapCache.cpp:234-239
    // Source: src/core/SkBitmapCache.cpp:241-242
    struct MipMapKey {
        key: Key,
        fDesc: SkBitmapCacheDesc,
    }
    impl MipMapKey {
        fn new(desc: &SkBitmapCacheDesc) -> Self {
            let fDesc = *desc;
            let key = Key::new(
                std::ptr::addr_of!(gMipMapKeyNamespaceLabel).cast(),
                SkMakeResourceCacheSharedIDForBitmap(fDesc.fImageID),
                &[
                    fDesc.fImageID,
                    fDesc.fSubset.fLeft as u32,
                    fDesc.fSubset.fTop as u32,
                    fDesc.fSubset.fRight as u32,
                    fDesc.fSubset.fBottom as u32,
                ],
            );
            Self { key, fDesc }
        }
    }
    // Source: src/core/SkBitmapCache.cpp:244-250
    impl MipMapRec {
        fn new(desc: &SkBitmapCacheDesc, result: &SkMipmap) -> Self {
            let fKey = MipMapKey::new(desc);
            let fMipMap = ManuallyDrop::new(result.attachToCacheAndRef());
            Self {
                fKey,
                fMipMap,
                links: RecLinks::new(),
            }
        }
    }
    // Source: src/core/SkBitmapCache.cpp:252-254
    impl Drop for MipMapRec {
        fn drop(&mut self) {
            unsafe { ManuallyDrop::take(&mut self.fMipMap) }.detachFromCacheAndUnref();
        }
    }
    // No external callback points to this record. CachedData's independently owned
    // allocation serializes client/cache ref transitions; shared access never mutates
    // these stable fields. Cache links remain exclusively cache-operation controlled.
    unsafe impl Rec for MipMapRec {
        fn links(&self) -> &RecLinks {
            &self.links
        }
        fn links_mut(&mut self) -> &mut RecLinks {
            &mut self.links
        }
        // Source: src/core/SkBitmapCache.cpp:256-256
        fn getKey(&self) -> &Key {
            &self.fKey.key
        }
        // Adapted/uncredited native cpp257: selected ARM64/64-bit native key footprint.
        fn bytesUsed(&self) -> usize {
            48 + self.fMipMap.size()
        }
        // Source: src/core/SkBitmapCache.cpp:258-261
        fn getCategory(&self) -> *const c_char {
            c"mipmap".as_ptr()
        }
        fn diagnostic_only_getDiscardable(
            &self,
        ) -> Option<
            std::ptr::NonNull<
                dyn crate::include::private::chromium::SkDiscardableMemory::SkDiscardableMemory,
            >,
        > {
            self.fMipMap.diagnostic_only_getDiscardable()
        }
    }
    impl MipMapRec {
        // Source: src/core/SkBitmapCache.cpp:263-265
        // Source: src/core/SkBitmapCache.cpp:268-271
        // Source: src/core/SkBitmapCache.cpp:273-275
        unsafe fn Finder(baseRec: &dyn Rec, contextMip: *mut c_void) -> bool {
            // This visitor is installed only for our private namespace key.
            let rec = &*(baseRec as *const dyn Rec as *const MipMapRec);
            let mm = SkMipmap::clone(&rec.fMipMap);
            if mm.data().is_null() {
                drop(mm);
                return false;
            }
            *contextMip.cast::<Option<SkMipmap>>() = Some(mm);
            true
        }
    }
    // Source: src/core/SkBitmapCache.cpp:278-279
    struct MipMapRec {
        fKey: MipMapKey,
        fMipMap: ManuallyDrop<SkMipmap>,
        links: RecLinks,
    }
    // fMipMap's raw level pointer targets its own reference-counted cached-data
    // allocation. Moving the record transfers the handle without relocating or
    // concurrently mutating that allocation; cache link mutation stays locked.
    unsafe impl Send for MipMapRec {}
    impl SkMipmapCache {
        // Source: src/core/SkBitmapCache.cpp:283-286
        // Source: src/core/SkBitmapCache.cpp:288-292
        pub fn FindAndRef(
            desc: &SkBitmapCacheDesc,
            localCache: Option<&mut ResourceCache>,
        ) -> Option<SkMipmap> {
            let key = MipMapKey::new(desc);
            let mut result = None;
            let context = std::ptr::addr_of_mut!(result).cast();
            let found = unsafe {
                CHECK_LOCAL!(localCache, find, Find, &key.key, MipMapRec::Finder, context)
            };
            if !found {
                result = None;
            }
            result
        }
    }
    // Source: src/core/SkBitmapCache.cpp:294-297
    fn get_fact(
        localCache: Option<&ResourceCache>,
    ) -> Option<self::SkResourceCache::DiscardableFactory> {
        CHECK_LOCAL!(localCache, discardableFactory, GetDiscardableFactory)
    }
    impl SkMipmapCache {
        // Source: src/core/SkBitmapCache.cpp:299-303
        // Source: src/core/SkBitmapCache.cpp:305-312
        pub fn AddAndRef(
            image: &dyn crate::src::image::SkImage_Base::SkImage_Base,
            localCache: Option<&mut ResourceCache>,
        ) -> Result<Option<SkMipmap>, crate::RasterError> {
            let mut src = crate::src::core::SkBitmap::SkBitmap::default();
            if !image.getROPixelsCPU(
                &mut src,
                crate::src::image::SkImage::CachingHint::kAllow_CachingHint,
            ) {
                return Ok(None);
            }
            // The actual image contract supplies retained readable CPU pixels;
            // the cache's native factory contract supplies locked discardable storage.
            let mipmap = unsafe { SkMipmap::BuildBitmap(&src, get_fact(localCache.as_deref())) }?;
            if let Some(mm) = &mipmap {
                let rec = Box::new(MipMapRec::new(&SkBitmapCacheDesc::MakeFromImage(image), mm));
                unsafe { CHECK_LOCAL!(localCache, add, Add, rec, std::ptr::null_mut()) }
                image.notifyAddedToRasterCache();
            }
            Ok(mipmap)
        }
    }
}
