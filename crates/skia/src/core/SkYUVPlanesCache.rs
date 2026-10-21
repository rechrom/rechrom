use super::SkBitmapCache::SkMakeResourceCacheSharedIDForBitmap;
use super::SkCachedData::SkCachedData;
use super::SkResourceCache as ResourceCache;
use super::SkResourceCache::{Key, Rec, RecLinks, SkResourceCache};
use super::SkYUVAPixmaps::SkYUVAPixmaps;
use crate::include::private::chromium::SkDiscardableMemory::SkDiscardableMemory;
use std::ffi::{c_char, c_void};
use std::mem::ManuallyDrop;
use std::ptr::NonNull;

// Source: src/core/SkYUVPlanesCache.h:19-20
pub struct SkYUVPlanesCache;
// Source: src/core/SkYUVPlanesCache.cpp:23-23
static gYUVPlanesKeyNamespaceLabel: u32 = 0;
// Source: src/core/SkYUVPlanesCache.cpp:25-28
// Option is the transfer marker for a native reference, never a substitute backing.
struct YUVValue {
    fPixmaps: SkYUVAPixmaps,
    fData: Option<SkCachedData<()>>,
}
// Source: src/core/SkYUVPlanesCache.cpp:30-39
// Key owns the native key words, including fGenID as its sole payload word.
struct YUVPlanesKey {
    base: Key,
}
impl YUVPlanesKey {
    fn new(genID: u32) -> Self {
        Self {
            base: Key::new(
                std::ptr::addr_of!(gYUVPlanesKeyNamespaceLabel).cast(),
                SkMakeResourceCacheSharedIDForBitmap(genID),
                &[genID],
            ),
        }
    }
}
// Source: src/core/SkYUVPlanesCache.cpp:41-48
// Source: src/core/SkYUVPlanesCache.cpp:53-54
// Inherited intrusive links and contiguous fields are required by Rust. Manual
// drops preserve destructor-body detach, then reverse native member destruction.
struct YUVPlanesRec {
    links: RecLinks,
    fKey: ManuallyDrop<YUVPlanesKey>,
    fValue: ManuallyDrop<YUVValue>,
}
impl YUVPlanesRec {
    fn new(key: YUVPlanesKey, data: &SkCachedData<()>, pixmaps: &SkYUVAPixmaps) -> Self {
        let fPixmaps = pixmaps.clone();
        let fData = data.attachToCacheAndRef();
        Self {
            links: RecLinks::new(),
            fKey: ManuallyDrop::new(key),
            fValue: ManuallyDrop::new(YUVValue {
                fPixmaps,
                fData: Some(fData),
            }),
        }
    }
}
// Cache ownership serializes record access; plane addresses are nonowning views
// into the actual cached allocation. No external plane owner gains Send here.
unsafe impl Send for YUVPlanesRec {}
// Source: src/core/SkYUVPlanesCache.cpp:49-51
impl Drop for YUVPlanesRec {
    fn drop(&mut self) {
        self.fValue.fData.take().unwrap().detachFromCacheAndUnref();
        unsafe {
            ManuallyDrop::drop(&mut self.fValue);
            ManuallyDrop::drop(&mut self.fKey);
        }
    }
}
unsafe impl Rec for YUVPlanesRec {
    fn links(&self) -> &RecLinks {
        &self.links
    }
    fn links_mut(&mut self) -> &mut RecLinks {
        &mut self.links
    }
    // Source: src/core/SkYUVPlanesCache.cpp:56-56
    fn getKey(&self) -> &Key {
        &self.fKey.base
    }
    // Source: src/core/SkYUVPlanesCache.cpp:57-57
    // Adapted: actual Rust owner size, not a claim of native ABI/budget equality.
    fn bytesUsed(&self) -> usize {
        std::mem::size_of::<Self>() + self.fValue.fData.as_ref().unwrap().size()
    }
    // Source: src/core/SkYUVPlanesCache.cpp:58-58
    fn getCategory(&self) -> *const c_char {
        c"yuv-planes".as_ptr()
    }
    // Source: src/core/SkYUVPlanesCache.cpp:59-61
    fn diagnostic_only_getDiscardable(&self) -> Option<NonNull<dyn SkDiscardableMemory>> {
        self.fValue
            .fData
            .as_ref()
            .unwrap()
            .diagnostic_only_getDiscardable()
    }
}
impl YUVPlanesRec {
    // Source: src/core/SkYUVPlanesCache.cpp:63-76
    unsafe fn Visitor(baseRec: &dyn Rec, contextData: *mut c_void) -> bool {
        // Namespace/key identity guarantees the same concrete type as native static_cast.
        let rec = &*(baseRec as *const dyn Rec as *const Self);
        let result = &mut *contextData.cast::<YUVValue>();
        let tmpData = rec.fValue.fData.as_ref().unwrap().clone();
        if tmpData.data().is_null() {
            drop(tmpData);
            return false;
        }
        result.fData = Some(tmpData);
        result.fPixmaps = rec.fValue.fPixmaps.clone();
        true
    }
}
impl SkYUVPlanesCache {
    // Source: src/core/SkYUVPlanesCache.h:28-30
    // Source: src/core/SkYUVPlanesCache.cpp:80-91
    /// Safety: on success output is nonnull; its plane addresses remain valid
    /// only while the returned cached-data client reference is retained.
    pub unsafe fn FindAndRef(
        genID: u32,
        pixmaps: Option<&mut SkYUVAPixmaps>,
        localCache: Option<&mut SkResourceCache>,
    ) -> Option<SkCachedData<()>> {
        let mut result = YUVValue {
            fPixmaps: SkYUVAPixmaps::default(),
            fData: None,
        };
        let key = YUVPlanesKey::new(genID);
        // Source: src/core/SkYUVPlanesCache.cpp:19-20
        let found = match localCache {
            Some(cache) => cache.find(
                &key.base,
                YUVPlanesRec::Visitor,
                std::ptr::from_mut(&mut result).cast(),
            ),
            None => ResourceCache::Find(
                &key.base,
                YUVPlanesRec::Visitor,
                std::ptr::from_mut(&mut result).cast(),
            ),
        };
        if !found {
            return None;
        }
        *pixmaps.expect("native successful lookup requires nonnull output") = result.fPixmaps;
        result.fData
    }
    // Source: src/core/SkYUVPlanesCache.h:36-37
    // Source: src/core/SkYUVPlanesCache.cpp:93-97
    /// Safety: every pixmap address must refer into data's actual allocation;
    /// callbacks and local cache lifetime obey the ResourceCache contract.
    pub unsafe fn Add(
        genID: u32,
        data: &SkCachedData<()>,
        pixmaps: &SkYUVAPixmaps,
        localCache: Option<&mut SkResourceCache>,
    ) {
        let key = YUVPlanesKey::new(genID);
        let rec = Box::new(YUVPlanesRec::new(key, data, pixmaps));
        match localCache {
            Some(cache) => cache.add(rec, std::ptr::null_mut()),
            None => ResourceCache::Add(rec, std::ptr::null_mut()),
        }
    }
}
