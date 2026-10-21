use crate::src::core::{
    SkDescriptor::SkDescriptor,
    SkStrike::{SkStrike, SkStrikePinner},
    SkStrikeSpec::SkStrikeSpec,
};
use std::{
    cell::UnsafeCell,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, OnceLock,
    },
};
// Source: src/core/SkStrikeCache.h:29-36
pub const DEFAULT_FONT_CACHE_COUNT_LIMIT: i32 = 2048;
pub const DEFAULT_FONT_CACHE_LIMIT: usize = 2 * 1024 * 1024;
// Source: src/core/SkStrikeCache.h:101-114
// Source: src/core/SkStrikeCache.h:41-41
pub struct SkStrikeCache {
    // Source: src/core/SkStrikeCache.h:100-100
    // Existing mutex guards the entire actual native cache state and LRU mutation.
    state: UnsafeCell<CacheState>,
    // Native fLock outlives destruction of the protected cache members.
    fLock: crate::include::private::SkMutex::SkMutex,
}
// Private state representation supplies native reverse member destruction;
// its same outer cache mutex still outlives this entire protected state.
pub(crate) struct CacheState {
    // Source: src/core/SkStrikeCache.h:114
    fPinnerCount: i32,
    // Source: src/core/SkStrikeCache.h:113
    fCacheCount: i32,
    // Source: src/core/SkStrikeCache.h:112
    fCacheCountLimit: i32,
    // Source: src/core/SkStrikeCache.h:111
    pub(crate) fTotalMemoryUsed: usize,
    // Source: src/core/SkStrikeCache.h:110
    fCacheSizeLimit: usize,
    // Source: src/core/SkStrikeCache.h:107-108
    fStrikeLookup: crate::src::core::SkTHash::THashTable<StrikeHandle, SkDescriptor, StrikeTraits>,
    // Source: src/core/SkStrikeCache.h:102
    fTail: *const SkStrike,
    // Source: src/core/SkStrikeCache.h:101
    fHead: *const SkStrike,
}
// Every member owner was admitted through unsafe find/create's native transferable
// resource contract. The raw LRU links are stable Arc allocations; all are accessed
// exclusively under this one mutex. No backend/effect blanket implementation.
unsafe impl Send for CacheState {}
// The one native fLock protects every access to the same CacheState.
unsafe impl Send for SkStrikeCache {}
unsafe impl Sync for SkStrikeCache {}
// Source: src/core/SkStrikeCache.h:104-109
// UNVERIFIED: native nullable sk_sp moved-from state, not a missing producer.
// Every live table entry owns Some(actual strike); table slot moves leave None
// exactly as sk_sp::release. No descriptor copy is stored alongside the owner.
struct StrikeHandle(Option<Arc<SkStrike>>);
impl crate::src::core::SkTHash::NativeMove for StrikeHandle {
    fn move_construct(source: &mut Self) -> Self {
        Self(source.0.take())
    }
    fn move_assign(destination: &mut Self, source: &mut Self) {
        destination.0 = source.0.take();
    }
}
struct StrikeTraits;

impl SkStrikeCache {
    // Source: src/core/SkStrikeCache.h:43
    pub fn new() -> Self {
        Self {
            fLock: crate::include::private::SkMutex::SkMutex::new(),
            state: UnsafeCell::new(CacheState {
                fHead: std::ptr::null(),
                fTail: std::ptr::null(),
                fStrikeLookup: crate::src::core::SkTHash::THashTable::default(),
                fCacheSizeLimit: DEFAULT_FONT_CACHE_LIMIT,
                fTotalMemoryUsed: 0,
                fCacheCountLimit: DEFAULT_FONT_CACHE_COUNT_LIMIT,
                fCacheCount: 0,
                fPinnerCount: 0,
            }),
        }
    }
}
impl SkStrikeCache {
    pub(crate) fn with_state<R>(&self, f: impl FnOnce(&mut CacheState) -> R) -> R {
        let mut lock_storage =
            core::pin::pin!(crate::include::private::SkMutex::SkAutoMutexExclusiveStorage::new());
        let _lock = unsafe {
            crate::include::private::SkMutex::SkAutoMutexExclusive::init(
                lock_storage.as_mut(),
                core::pin::Pin::new_unchecked(&self.fLock),
            )
        };
        f(unsafe { &mut *self.state.get() })
    }
}
// Source: src/core/SkStrikeCache.cpp:29-33
// AtomicBool adapts the externally mutable native bool without a Rust data race.
// Its value is read only in the native non-forced-thread-local configuration.
pub static gSkUseThreadLocalStrikeCaches_IAcknowledgeThisIsIncrediblyExperimental: AtomicBool =
    AtomicBool::new(cfg!(SK_ENABLE_THREADLOCAL_STRIKECACHE));

impl SkStrikeCache {
    // Source: src/core/SkStrikeCache.cpp:35-47
    // Native caches are allocated once and deliberately not deleted, including
    // when a thread exits. Leaking the owner preserves pointers held by strikes.
    // Source: src/core/SkStrikeCache.h:45
    pub fn GlobalStrikeCache() -> &'static Self {
        std::thread_local! {
            static LOCAL_CACHE: &'static SkStrikeCache =
                Box::leak(Box::new(SkStrikeCache::new()));
        }
        #[cfg(SK_ENABLE_THREADLOCAL_STRIKECACHE)]
        {
            LOCAL_CACHE.with(|cache| *cache)
        }
        #[cfg(not(SK_ENABLE_THREADLOCAL_STRIKECACHE))]
        {
            if gSkUseThreadLocalStrikeCaches_IAcknowledgeThisIsIncrediblyExperimental
                .load(Ordering::Relaxed)
            {
                return LOCAL_CACHE.with(|cache| *cache);
            }
            static CACHE: OnceLock<SkStrikeCache> = OnceLock::new();
            CACHE.get_or_init(Self::new)
        }
    }
}
impl SkStrikeCache {
    // Source: src/core/SkStrikeCache.cpp:49-57
    /// Safety: complete retained typeface/effect/backend graphs can transfer between
    /// threads and execute under a strike lock on any thread for their full owner lifetime.
    /// The returned strike stores this exact non-owning cache identity. Keep it
    /// alive and stationary during every later cache-using strike operation,
    /// including unlock accounting and pinner verification. Native default
    /// Strike destruction alone does not dereference its cache pointer. Cached
    /// entries/links must not be used after this cache starts destruction.
    /// Its cache/strike locks provide serialization, not an external owner lease.
    // Source: src/core/SkStrikeCache.h:54
    pub unsafe fn findOrCreateStrike(&self, spec: &SkStrikeSpec) -> Arc<SkStrike> {
        self.with_state(|state| {
            let result = if let Some(strike) = unsafe { state.internal_find(spec.descriptor()) } {
                strike
            } else {
                unsafe { state.internal_create(core::ptr::NonNull::from(self), spec, None, None) }
            };
            unsafe { state.internal_purge(0, false) };
            result
        })
    }
}
impl SkStrikeCache {
    // Source: src/core/SkStrikeCache.cpp:63-65
    // Source: src/core/SkStrikeCache.h:59
    pub fn PurgeAll() {
        Self::GlobalStrikeCache().purgeAll();
    }
}
// Source: src/core/SkStrikeCache.h:60-60
// Source: src/core/SkStrikeCache.cpp:67-79
impl SkStrikeCache {
    pub fn Dump() {
        use crate::src::core::SkGraphics;
        unsafe {
            crate::SkDebugf!(@source b"src/core/SkStrikeCache.cpp\0".as_ptr(), 68; b"GlyphCache [     used    budget ]\n\0".as_ptr());
            crate::SkDebugf!(@source b"src/core/SkStrikeCache.cpp\0".as_ptr(), 69; b"    bytes  [ %8zu  %8zu ]\n\0".as_ptr(),SkGraphics::GetFontCacheUsed(),SkGraphics::GetFontCacheLimit());
            crate::SkDebugf!(@source b"src/core/SkStrikeCache.cpp\0".as_ptr(), 71; b"    count  [ %8d  %8d ]\n\0".as_ptr(),SkGraphics::GetFontCacheCountUsed(),SkGraphics::GetFontCacheCountLimit());
        }
        let visitor = |strike: &SkStrike| strike.dump();
        Self::GlobalStrikeCache().forEachStrike(visitor);
    }
}

impl SkStrikeCache {
    // Source: src/core/SkStrikeCache.cpp:102-107
    // Source: src/core/SkStrikeCache.h:47
    pub fn findStrike(&self, desc: &SkDescriptor) -> Option<Arc<SkStrike>> {
        self.with_state(|state| unsafe {
            let result = state.internal_find(desc);
            state.internal_purge(0, false);
            result
        })
    }
}
impl CacheState {
    // Source: src/core/SkStrikeCache.cpp:109-133
    unsafe fn internal_find(&mut self, desc: &SkDescriptor) -> Option<Arc<SkStrike>> {
        if !self.fHead.is_null() && unsafe { (*self.fHead).getDescriptor() == desc } {
            unsafe {
                Arc::increment_strong_count(self.fHead);
                return Some(Arc::from_raw(self.fHead));
            }
        }
        let result = self
            .fStrikeLookup
            .find(desc)?
            .0
            .as_ref()
            .expect("native found strike is nonnull")
            .clone();
        let ptr = Arc::as_ptr(&result);
        if self.fHead != ptr {
            unsafe {
                let links = &mut *(*ptr).links.get();
                (*(*links.fPrev).links.get()).fNext = links.fNext;
                if !links.fNext.is_null() {
                    (*(*links.fNext).links.get()).fPrev = links.fPrev;
                } else {
                    self.fTail = links.fPrev;
                }
                (*(*self.fHead).links.get()).fPrev = ptr;
                links.fNext = self.fHead;
                links.fPrev = std::ptr::null();
                self.fHead = ptr;
            }
        }
        Some(result)
    }
}
impl SkStrikeCache {
    // Source: src/core/SkStrikeCache.cpp:135-141
    /// Safety: same complete retained-resource transfer and stationary cache-owner
    /// operation-lease contract as findOrCreateStrike; no lifetime is extended.
    pub unsafe fn createStrike(
        &self,
        spec: &SkStrikeSpec,
        metrics: Option<&crate::include::core::SkFontMetrics::SkFontMetrics>,
        pinner: Option<Box<dyn SkStrikePinner>>,
    ) -> Arc<SkStrike> {
        self.with_state(|state| unsafe {
            state.internal_create(core::ptr::NonNull::from(self), spec, metrics, pinner)
        })
    }
}
impl CacheState {
    // Source: src/core/SkStrikeCache.cpp:143-152
    unsafe fn internal_create(
        &mut self,
        cache: core::ptr::NonNull<SkStrikeCache>,
        spec: &SkStrikeSpec,
        metrics: Option<&crate::include::core::SkFontMetrics::SkFontMetrics>,
        pinner: Option<Box<dyn SkStrikePinner>>,
    ) -> Arc<SkStrike> {
        let scaler = unsafe { spec.createScalerContext() };
        let strike = unsafe { SkStrike::new(cache, spec, scaler, metrics, pinner) };
        unsafe { self.internal_attach(strike.clone()) };
        strike
    }
}
impl SkStrikeCache {
    // Source: src/core/SkStrikeCache.cpp:154-157
    pub fn purgePinned(&self, min_bytes: usize) {
        self.with_state(|state| unsafe {
            state.internal_purge(min_bytes, true);
        });
    }
}
impl SkStrikeCache {
    // Source: src/core/SkStrikeCache.cpp:159-162
    // Source: src/core/SkStrikeCache.h:66
    pub fn purgeAll(&self) {
        self.with_state(|state| unsafe {
            state.internal_purge(state.fTotalMemoryUsed, true);
        });
    }
}
impl SkStrikeCache {
    // Source: src/core/SkStrikeCache.cpp:164-167
    // Source: src/core/SkStrikeCache.h:75
    pub fn getTotalMemoryUsed(&self) -> usize {
        self.with_state(|s| s.fTotalMemoryUsed)
    }
}
impl SkStrikeCache {
    // Source: src/core/SkStrikeCache.cpp:169-172
    // Source: src/core/SkStrikeCache.h:71
    pub fn getCacheCountUsed(&self) -> i32 {
        self.with_state(|s| s.fCacheCount)
    }
}
impl SkStrikeCache {
    // Source: src/core/SkStrikeCache.cpp:174-177
    // Source: src/core/SkStrikeCache.h:69
    pub fn getCacheCountLimit(&self) -> i32 {
        self.with_state(|s| s.fCacheCountLimit)
    }
}
impl SkStrikeCache {
    // Source: src/core/SkStrikeCache.cpp:179-186
    // Source: src/core/SkStrikeCache.h:74
    pub fn setCacheSizeLimit(&self, new_limit: usize) -> usize {
        self.with_state(|s| unsafe {
            let prev = s.fCacheSizeLimit;
            s.fCacheSizeLimit = new_limit;
            s.internal_purge(0, false);
            prev
        })
    }
}
impl SkStrikeCache {
    // Source: src/core/SkStrikeCache.cpp:188-191
    // Source: src/core/SkStrikeCache.h:73
    pub fn getCacheSizeLimit(&self) -> usize {
        self.with_state(|s| s.fCacheSizeLimit)
    }
}
impl SkStrikeCache {
    // Source: src/core/SkStrikeCache.cpp:193-204
    // Source: src/core/SkStrikeCache.h:70
    pub fn setCacheCountLimit(&self, new_limit: i32) -> i32 {
        let new_limit = new_limit.max(0);
        self.with_state(|s| unsafe {
            let prev = s.fCacheCountLimit;
            s.fCacheCountLimit = new_limit;
            s.internal_purge(0, false);
            prev
        })
    }
}
impl SkStrikeCache {
    // Source: src/core/SkStrikeCache.h:98-98
    // Source: src/core/SkStrikeCache.cpp:206-214
    // The callback runs while the native cache lock is held. No owner snapshot,
    // extra ref increments, or lock release is inserted around visitation.
    fn forEachStrike(&self, mut visitor: impl FnMut(&SkStrike)) {
        self.with_state(|state| unsafe {
            state.validate();
            let mut strike = state.fHead;
            while !strike.is_null() {
                visitor(&*strike);
                strike = (*(*strike).links.get()).fNext;
            }
        });
    }
}
impl CacheState {
    // Source: src/core/SkStrikeCache.cpp:216-275
    unsafe fn internal_purge(&mut self, min_bytes: usize, check_pinners: bool) -> usize {
        // Native default builds force the check for staged rollout; the
        // opt-out build keeps each caller's actual/default bool argument.
        #[cfg(not(SK_STRIKE_CACHE_DOESNT_AUTO_CHECK_PINNERS))]
        let check_pinners = true;
        if self.fPinnerCount == self.fCacheCount && !check_pinners {
            return 0;
        }
        let mut bytes_needed = if self.fTotalMemoryUsed > self.fCacheSizeLimit {
            self.fTotalMemoryUsed - self.fCacheSizeLimit
        } else {
            0
        };
        bytes_needed = bytes_needed.max(min_bytes);
        if bytes_needed != 0 {
            bytes_needed = bytes_needed.max(self.fTotalMemoryUsed >> 2);
        }
        let mut count_needed = if self.fCacheCount > self.fCacheCountLimit {
            self.fCacheCount - self.fCacheCountLimit
        } else {
            0
        };
        if count_needed != 0 {
            count_needed = count_needed.max(self.fCacheCount >> 2);
        }
        if count_needed == 0 && bytes_needed == 0 {
            return 0;
        }
        let (mut bytes_freed, mut count_freed) = (0, 0);
        let mut ptr = self.fTail;
        while !ptr.is_null() && (bytes_freed < bytes_needed || count_freed < count_needed) {
            unsafe {
                let links = &mut *(*ptr).links.get();
                let prev = links.fPrev;
                if links
                    .fPinner
                    .as_mut()
                    .is_none_or(|p| check_pinners && p.canDelete())
                {
                    bytes_freed += links.fMemoryUsed;
                    count_freed += 1;
                    self.internal_remove(ptr);
                }
                ptr = prev;
            }
        }
        unsafe { self.validate() };
        // Source: src/core/SkStrikeCache.cpp:267-272
        #[cfg(SPEW_PURGE_STATUS)]
        if count_freed != 0 {
            unsafe {
                crate::SkDebugf!(@source b"src/core/SkStrikeCache.cpp\0".as_ptr(), 269; b"purging %dK from font cache [%d entries]\n\0".as_ptr(),(bytes_freed>>10) as i32,count_freed);
            }
        }
        bytes_freed
    }
}
impl CacheState {
    // Source: src/core/SkStrikeCache.cpp:277-297
    unsafe fn internal_attach(&mut self, strike: Arc<SkStrike>) {
        unsafe {
            let ptr = Arc::as_ptr(&strike);
            let links = &mut *strike.links.get();
            crate::SkASSERT!(@source b"src/core/SkStrikeCache.cpp\0".as_ptr(),278; @condition b"fStrikeLookup.find(strike->getDescriptor()) == nullptr\0".as_ptr(); self.fStrikeLookup.find(strike.getDescriptor()).is_none());
            // Native set(std::move(strike)) transfers this exact strong owner.
            // Keep only the preexisting raw pointer while updating LRU links.
            self.fStrikeLookup
                .set(StrikeHandle(Some(strike)))
                .expect("native table insertion has capacity");
            crate::SkASSERT!(@source b"src/core/SkStrikeCache.cpp\0".as_ptr(),281; @condition b"nullptr == strikePtr->fPrev && nullptr == strikePtr->fNext\0".as_ptr(); links.fPrev.is_null() && links.fNext.is_null());
            self.fCacheCount += 1;
            self.fPinnerCount += i32::from(links.fPinner.is_some());
            self.fTotalMemoryUsed += links.fMemoryUsed;
            if !self.fHead.is_null() {
                (*(*self.fHead).links.get()).fPrev = ptr;
                links.fNext = self.fHead;
            }
            if self.fTail.is_null() {
                self.fTail = ptr;
            }
            self.fHead = ptr;
        }
    }
}
impl CacheState {
    // Source: src/core/SkStrikeCache.cpp:299-319
    unsafe fn internal_remove(&mut self, ptr: *const SkStrike) {
        unsafe {
            crate::SkASSERT!(@source b"src/core/SkStrikeCache.cpp\0".as_ptr(),300; @condition b"fCacheCount > 0\0".as_ptr(); self.fCacheCount > 0);
            let links = &mut *(*ptr).links.get();
            self.fCacheCount -= 1;
            self.fPinnerCount -= i32::from(links.fPinner.is_some());
            self.fTotalMemoryUsed -= links.fMemoryUsed;
            if !links.fPrev.is_null() {
                (*(*links.fPrev).links.get()).fNext = links.fNext;
            } else {
                self.fHead = links.fNext;
            }
            if !links.fNext.is_null() {
                (*(*links.fNext).links.get()).fPrev = links.fPrev;
            } else {
                self.fTail = links.fPrev;
            }
            links.fPrev = std::ptr::null();
            links.fNext = std::ptr::null();
            links.fRemoved = true;
            self.fStrikeLookup.remove((*ptr).getDescriptor());
        }
    }
}
impl CacheState {
    // Source: src/core/SkStrikeCache.cpp:321-343
    unsafe fn validate(&self) {
        #[cfg(sk_debug)]
        unsafe {
            let (mut bytes, mut count) = (0, 0);
            let mut ptr = self.fHead;
            while !ptr.is_null() {
                let links = &*(*ptr).links.get();
                bytes += links.fMemoryUsed;
                count += 1;
                crate::SkASSERT!(@source b"src/core/SkStrikeCache.cpp\0".as_ptr(),330; @condition b"fStrikeLookup.findOrNull(strike->getDescriptor()) != nullptr\0".as_ptr(); self.fStrikeLookup.find((*ptr).getDescriptor()).is_some());
                ptr = links.fNext;
            }
            // Source: src/core/SkStrikeCache.cpp:334-337
            if self.fCacheCount != count {
                crate::SkDebugf!(@source b"src/core/SkStrikeCache.cpp\0".as_ptr(), 335; b"fCacheCount: %d, computedCount: %d\0".as_ptr(),self.fCacheCount,count);
                crate::SK_ABORT!(@source b"src/core/SkStrikeCache.cpp\0".as_ptr(),336; "fCacheCount != computedCount");
            }
            // Source: src/core/SkStrikeCache.cpp:338-341
            if self.fTotalMemoryUsed != bytes {
                crate::SkDebugf!(@source b"src/core/SkStrikeCache.cpp\0".as_ptr(), 339; b"fTotalMemoryUsed: %zu, computedBytes: %zu\0".as_ptr(),self.fTotalMemoryUsed,bytes);
                crate::SK_ABORT!(@source b"src/core/SkStrikeCache.cpp\0".as_ptr(),340; "fTotalMemoryUsed == computedBytes");
            }
        }
    }
}

impl SkStrikeCache {
    // Source: src/core/SkStrikeCache.h:49-52
    /// Native omitted metrics and pinner arguments. Same resource-transfer contract
    /// as createStrike; the original cache remains live for subsequent cache-using operations.
    pub unsafe fn createStrikeDefault(&self, spec: &SkStrikeSpec) -> Arc<SkStrike> {
        unsafe { self.createStrike(spec, None, None) }
    }
    // Source: src/core/SkStrikeCache.h:49-52
    /// Safety: same retained-resource transfer contract as createStrike.
    pub unsafe fn createStrikeWithMetrics(
        &self,
        spec: &SkStrikeSpec,
        metrics: Option<&crate::include::core::SkFontMetrics::SkFontMetrics>,
    ) -> Arc<SkStrike> {
        unsafe { self.createStrike(spec, metrics, None) }
    }
    // Source: src/core/SkStrikeCache.h:67
    pub fn purgePinnedDefault(&self) {
        self.purgePinned(0);
    }
}

impl crate::src::core::SkTHash::HashTraits<StrikeHandle, SkDescriptor> for StrikeTraits {
    // Source: src/core/SkStrikeCache.cpp:345-347
    type Key<'a>
        = &'a SkDescriptor
    where
        StrikeHandle: 'a,
        SkDescriptor: 'a;
    fn GetKey<'a>(value: &'a StrikeHandle) -> Self::Key<'a>
    where
        SkDescriptor: 'a,
    {
        value
            .0
            .as_ref()
            .expect("native occupied cache slot has a nonnull strike")
            .getDescriptor()
    }
    // Source: src/core/SkStrikeCache.cpp:349-351
    fn Hash(key: &SkDescriptor) -> u32 {
        key.checksum()
    }
}
// Source: src/core/SkStrikeCache.h:79-79
pub const kGlyphCacheDumpName: &core::ffi::CStr = c"skia/sk_glyph_cache";
// Source: src/core/SkStrikeCache.cpp:81-100
impl SkStrikeCache {
    pub fn DumpMemoryStatistics(
        dump: &mut dyn crate::include::core::SkTraceMemoryDump::SkTraceMemoryDump,
    ) {
        use crate::src::core::SkGraphics;
        dump.dumpNumericValue(
            kGlyphCacheDumpName,
            c"size",
            c"bytes",
            SkGraphics::GetFontCacheUsed() as u64,
        );
        dump.dumpNumericValue(
            kGlyphCacheDumpName,
            c"budget_size",
            c"bytes",
            SkGraphics::GetFontCacheLimit() as u64,
        );
        dump.dumpNumericValue(
            kGlyphCacheDumpName,
            c"glyph_count",
            c"objects",
            SkGraphics::GetFontCacheCountUsed() as u64,
        );
        dump.dumpNumericValue(
            kGlyphCacheDumpName,
            c"budget_glyph_count",
            c"objects",
            SkGraphics::GetFontCacheCountLimit() as u64,
        );
        if dump.getRequestedDetails()
            == crate::include::core::SkTraceMemoryDump::LevelOfDetail::kLight_LevelOfDetail
        {
            dump.setMemoryBacking(kGlyphCacheDumpName, c"malloc", None);
            return;
        }
        Self::GlobalStrikeCache().forEachStrike(|strike| strike.dumpMemoryStatistics(dump));
    }
}

// Source: src/core/SkStrikeCache.h:56-57
// Source: src/core/SkStrikeCache.cpp:59-61
impl crate::src::text::StrikeForGPU::StrikeForGPUCacheInterface for SkStrikeCache {
    unsafe fn findOrCreateScopedStrike(
        &self,
        spec: &SkStrikeSpec,
    ) -> Result<Option<Arc<dyn crate::src::text::StrikeForGPU::StrikeForGPU>>, crate::RasterError>
    {
        let strike = unsafe { self.findOrCreateStrike(spec) };
        let strike: Arc<dyn crate::src::text::StrikeForGPU::StrikeForGPU> = strike;
        Ok(Some(strike))
    }
}
