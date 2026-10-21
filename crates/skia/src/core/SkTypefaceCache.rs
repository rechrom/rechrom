use crate::src::core::SkTypeface::{SkTypeface, SkTypefaceRef};
// Source: src/core/SkTypefaceCache.h:71-72
pub struct SkTypefaceCache {
    fTypefaces: crate::include::private::SkTArray::TArray<SkTypefaceRef, true>,
}
// Source: src/core/SkTypefaceCache.cpp:21-21
impl Default for SkTypefaceCache {
    fn default() -> Self {
        Self {
            fTypefaces: crate::include::private::SkTArray::TArray::new(),
        }
    }
}
// Native TArray owns the actual allocation, grow policy, forward element
// destruction and removeShuffle move/drop interval. Every stored SkTypefaceRef
// is the actual native STRONG sk_sp owner, including the cache-only purge test.
// Safety: an exclusively owned cache may move between threads. Every contained
// actual typeface is Send+Sync; global access remains under the native mutex.
unsafe impl Send for SkTypefaceCache {}
impl SkTypefaceCache {
    // Source: src/core/SkTypefaceCache.cpp:23-25
    // Source: src/core/SkTypefaceCache.cpp:27-33
    pub fn add(&mut self, face: SkTypefaceRef) {
        // Source: src/core/SkTypefaceCache.cpp:24-24
        crate::SkASSERT_RELEASE!(@source b"src/core/SkTypefaceCache.cpp\0".as_ptr(),24; @condition b"face\0".as_ptr(); face.is_some());
        let limit = crate::src::core::SkGraphics::GetTypefaceCacheCountLimit();
        if self.fTypefaces.size() >= limit {
            self.purge(limit >> 2);
        }
        if limit > 0 {
            self.fTypefaces.emplace_back_with(|| face);
        }
    }
    // Source: src/core/SkTypefaceCache.cpp:35-42
    // A Rust closure carries the native callback's context without an untyped void pointer.
    pub fn findByProcAndRef(
        &self,
        mut proc: impl FnMut(&SkTypeface) -> bool,
    ) -> Option<SkTypefaceRef> {
        for typeface in self.fTypefaces.as_slice() {
            if proc(typeface.as_ref()) {
                return Some(typeface.clone());
            }
        }
        None
    }
    // Source: src/core/SkTypefaceCache.cpp:44-58
    fn purge(&mut self, mut num_to_purge: i32) {
        let mut count = self.fTypefaces.size();
        let mut i = 0;
        while i < count {
            if self.fTypefaces[i as usize].unique() {
                self.fTypefaces.removeShuffle(i);
                count -= 1;
                num_to_purge -= 1;
                if num_to_purge == 0 {
                    return;
                }
            } else {
                i += 1;
            }
        }
    }
    // Source: src/core/SkTypefaceCache.cpp:60-62
    pub fn purgeAll(&mut self) {
        self.purge(self.fTypefaces.size());
    }
}
// Source: src/core/SkTypefaceCache.cpp:66-69
// C++ constructs an ordinary static cache and registers its destructor. Rust statics do not
// run Drop, so standard-C exit registration retains the native cache lifetime explicitly.
struct GlobalCacheStorage {
    cache: core::cell::UnsafeCell<Option<SkTypefaceCache>>,
}
// The native typeface_cache_mutex alone guards normal shared cache access.
// Native diagnostic Dump and process teardown require external quiescence.
unsafe impl Sync for GlobalCacheStorage {}
static GLOBAL_CACHE: std::sync::OnceLock<GlobalCacheStorage> = std::sync::OnceLock::new();
unsafe extern "C" {
    fn atexit(callback: extern "C" fn()) -> i32;
}
extern "C" fn release_global_cache() {
    // Native ordinary static destruction is not enclosed in typeface_cache_mutex.
    // Process teardown must exclude further cache accesses, just as native Get.
    if let Some(cache) = GLOBAL_CACHE.get() {
        let owned = unsafe { (&mut *cache.cache.get()).take() };
        drop(owned);
    }
}
fn Get() -> &'static GlobalCacheStorage {
    GLOBAL_CACHE.get_or_init(|| {
        let cache = GlobalCacheStorage {
            cache: core::cell::UnsafeCell::new(Some(SkTypefaceCache::default())),
        };
        unsafe {
            atexit(release_global_cache);
        }
        cache
    })
}
// Source: src/core/SkTypefaceCache.cpp:71-74
pub fn NewTypefaceID() -> u32 {
    static NEXT_ID: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(1);
    NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed) as u32
}

// Source: src/core/SkTypefaceCache.cpp:76-79
fn typeface_cache_mutex() -> &'static crate::include::private::SkMutex::SkMutex {
    // Same deliberately leaked native heap mutex; its backend remains live
    // through every release and through destruction of the ordinary cache.
    static MUTEX: std::sync::OnceLock<&'static crate::include::private::SkMutex::SkMutex> =
        std::sync::OnceLock::new();
    MUTEX.get_or_init(|| Box::leak(Box::new(crate::include::private::SkMutex::SkMutex::new())))
}
// Source: src/core/SkTypefaceCache.cpp:81-84
pub fn Add(face: SkTypefaceRef) {
    let mut storage =
        core::pin::pin!(crate::include::private::SkMutex::SkAutoMutexExclusiveStorage::new());
    let _guard = unsafe {
        crate::include::private::SkMutex::SkAutoMutexExclusive::init(
            storage.as_mut(),
            core::pin::Pin::new_unchecked(typeface_cache_mutex()),
        )
    };
    unsafe {
        (&mut *Get().cache.get())
            .as_mut()
            .expect("cache exists until process teardown")
            .add(face);
    }
}
// Source: src/core/SkTypefaceCache.cpp:86-89
pub fn FindByProcAndRef(proc: impl FnMut(&SkTypeface) -> bool) -> Option<SkTypefaceRef> {
    let mut storage =
        core::pin::pin!(crate::include::private::SkMutex::SkAutoMutexExclusiveStorage::new());
    let _guard = unsafe {
        crate::include::private::SkMutex::SkAutoMutexExclusive::init(
            storage.as_mut(),
            core::pin::Pin::new_unchecked(typeface_cache_mutex()),
        )
    };
    unsafe {
        (&*Get().cache.get())
            .as_ref()
            .expect("cache exists until process teardown")
            .findByProcAndRef(proc)
    }
}
// Source: src/core/SkTypefaceCache.cpp:91-94
pub fn PurgeAll() {
    let mut storage =
        core::pin::pin!(crate::include::private::SkMutex::SkAutoMutexExclusiveStorage::new());
    let _guard = unsafe {
        crate::include::private::SkMutex::SkAutoMutexExclusive::init(
            storage.as_mut(),
            core::pin::Pin::new_unchecked(typeface_cache_mutex()),
        )
    };
    unsafe {
        (&mut *Get().cache.get())
            .as_mut()
            .expect("cache exists until process teardown")
            .purgeAll();
    }
}

// Source: src/core/SkTypefaceCache.cpp:99-107
#[cfg(sk_debug)]
fn DumpProc(face: &SkTypeface) -> bool {
    let mut n = String::new();
    face.getFamilyName(&mut n);
    let n = crate::src::core::SkString::SkString::from_view(n.as_bytes());
    let s = face.fontStyle();
    let id = face.uniqueID();
    unsafe {
        crate::SkDebugf!(@source b"src/core/SkTypefaceCache.cpp\0".as_ptr(), 104; b"SkTypefaceCache: face %p typefaceID %u weight %d width %d style %d name %s\n\0".as_ptr(),
        face as *const SkTypeface, id, s.weight(), s.width(), s.slant().0 as core::ffi::c_int, n.c_str());
    }
    false
}
// Source: src/core/SkTypefaceCache.h:64-64
// Source: src/core/SkTypefaceCache.cpp:110-114
/// Safety: native diagnostic Dump takes no cache mutex. The caller excludes
/// concurrent cache mutation and process teardown for this read/callback walk.
pub unsafe fn Dump() {
    #[cfg(sk_debug)]
    {
        let cache = unsafe { &*Get().cache.get() };
        let _ = cache
            .as_ref()
            .expect("cache exists until process teardown")
            .findByProcAndRef(DumpProc);
    }
}

// Source: src/core/SkTypefaceCache.h:26-26
pub type FindProc = unsafe fn(*mut SkTypeface, *mut core::ffi::c_void) -> bool;
impl SkTypefaceCache {
    // Source: src/core/SkTypefaceCache.h:19-19
    pub fn new() -> Self {
        Self::default()
    }
    // Source: src/core/SkTypefaceCache.h:39-39
    /// Safety: callback/context preserve the native synchronous typeface lease;
    /// mutation through its pointer must not conflict with retained shared users.
    pub unsafe fn findByProcAndRef_raw(
        &self,
        proc: FindProc,
        ctx: *mut core::ffi::c_void,
    ) -> Option<SkTypefaceRef> {
        self.findByProcAndRef(|face| unsafe {
            proc(face as *const SkTypeface as *mut SkTypeface, ctx)
        })
    }
}
// Source: src/core/SkTypefaceCache.h:58-58
pub unsafe fn FindByProcAndRef_raw(
    proc: FindProc,
    ctx: *mut core::ffi::c_void,
) -> Option<SkTypefaceRef> {
    FindByProcAndRef(|face| unsafe { proc(face as *const SkTypeface as *mut SkTypeface, ctx) })
}
