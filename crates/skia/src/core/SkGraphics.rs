use super::SkStrikeCache::SkStrikeCache;
use std::sync::atomic::{AtomicI32, Ordering};

// Source: include/core/SkGraphics.h:30-30
// Source: src/core/SkGraphics.cpp:22-31
pub fn Init() {
    super::SkCpu::CacheRuntimeFeatures();
    super::SkOpts::Init();
    super::SkOpts::Init_BitmapProcState();
    super::SkOpts::Init_BlitMask();
    super::SkOpts::Init_BlitRow();
    super::SkOpts::Init_Memset();
    super::SkOpts::Init_Swizzler();
}

// Source: src/core/SkGraphics.cpp:40-44
// Source: include/core/SkGraphics.h:141-141
pub fn PurgeAllCaches() {
    PurgeFontCache();
    PurgeResourceCache();
    super::SkImageFilter_Base::SkImageFilter_Base::PurgeCache();
}

// Source: src/core/SkGraphics.cpp:48-50
// Source: include/core/SkGraphics.h:37-37
pub fn GetFontCacheLimit() -> usize {
    SkStrikeCache::GlobalStrikeCache().getCacheSizeLimit()
}

// Source: src/core/SkGraphics.cpp:52-54
// Source: include/core/SkGraphics.h:46-46
pub fn SetFontCacheLimit(bytes: usize) -> usize {
    SkStrikeCache::GlobalStrikeCache().setCacheSizeLimit(bytes)
}

// Source: src/core/SkGraphics.cpp:56-58
// Source: include/core/SkGraphics.h:51-51
pub fn GetFontCacheUsed() -> usize {
    SkStrikeCache::GlobalStrikeCache().getTotalMemoryUsed()
}

// Source: src/core/SkGraphics.cpp:60-62
// Source: include/core/SkGraphics.h:63-63
pub fn GetFontCacheCountLimit() -> i32 {
    SkStrikeCache::GlobalStrikeCache().getCacheCountLimit()
}

// Source: src/core/SkGraphics.cpp:64-66
// Source: include/core/SkGraphics.h:70-70
pub fn SetFontCacheCountLimit(count: i32) -> i32 {
    SkStrikeCache::GlobalStrikeCache().setCacheCountLimit(count)
}

// Source: src/core/SkGraphics.cpp:68-70
// Source: include/core/SkGraphics.h:57-57
pub fn GetFontCacheCountUsed() -> i32 {
    SkStrikeCache::GlobalStrikeCache().getCacheCountUsed()
}

// Source: src/core/SkGraphics.cpp:72-75
// Source: include/core/SkGraphics.h:90-90
pub fn PurgeFontCache() {
    SkStrikeCache::GlobalStrikeCache().purgeAll();
    super::SkTypefaceCache::PurgeAll();
}

// Source: src/core/SkGraphics.cpp:77-79
// Source: include/core/SkGraphics.h:97-97
pub fn PurgePinnedFontCache() {
    // The default argument in SkStrikeCache.h:67 is zero.
    SkStrikeCache::GlobalStrikeCache().purgePinned(0);
}

// Source: src/core/SkGraphics.cpp:81-81
// Source: include/core/SkGraphics.h:102-102
pub fn GetResourceCacheTotalBytesUsed() -> usize {
    super::SkResourceCache::GetTotalBytesUsed()
}

// Source: src/core/SkGraphics.cpp:83-83
// Source: include/core/SkGraphics.h:109-109
pub fn GetResourceCacheTotalByteLimit() -> usize {
    super::SkResourceCache::GetTotalByteLimit()
}

// Source: src/core/SkGraphics.cpp:85-87
// Source: include/core/SkGraphics.h:110-110
pub fn SetResourceCacheTotalByteLimit(newLimit: usize) -> usize {
    super::SkResourceCache::SetTotalByteLimit(newLimit)
}

// Source: src/core/SkGraphics.cpp:89-91
// Source: include/core/SkGraphics.h:126-126
pub fn GetResourceCacheSingleAllocationByteLimit() -> usize {
    super::SkResourceCache::GetSingleAllocationByteLimit()
}

// Source: src/core/SkGraphics.cpp:93-95
// Source: include/core/SkGraphics.h:127-127
pub fn SetResourceCacheSingleAllocationByteLimit(newLimit: usize) -> usize {
    super::SkResourceCache::SetSingleAllocationByteLimit(newLimit)
}

// Source: src/core/SkGraphics.cpp:97-100
// Source: include/core/SkGraphics.h:116-116
pub fn PurgeResourceCache() {
    super::SkImageFilter_Base::SkImageFilter_Base::PurgeCache();
    super::SkResourceCache::PurgeAll();
}

// Source: src/core/SkGraphics.cpp:102-102
// Relaxed atomics express native integer accesses without Rust static-mut data races;
// the original defined single-thread policy retains its exact values and update order.
static TYPEFACE_CACHE_COUNT_LIMIT: AtomicI32 = AtomicI32::new(1024);
// Source: src/core/SkGraphics.cpp:104-106
// Source: include/core/SkGraphics.h:76-76
pub fn GetTypefaceCacheCountLimit() -> i32 {
    TYPEFACE_CACHE_COUNT_LIMIT.load(Ordering::Relaxed)
}
// Source: src/core/SkGraphics.cpp:108-112
// Source: include/core/SkGraphics.h:83-83
pub fn SetTypefaceCacheCountLimit(count: i32) -> i32 {
    let previous = TYPEFACE_CACHE_COUNT_LIMIT.load(Ordering::Relaxed);
    TYPEFACE_CACHE_COUNT_LIMIT.store(count, Ordering::Relaxed);
    previous
}

// Source: src/core/SkGraphics.cpp:35-38
// Source: include/core/SkGraphics.h:133-133
pub fn DumpMemoryStatistics(
    dump: &mut dyn crate::include::core::SkTraceMemoryDump::SkTraceMemoryDump,
) {
    super::SkResourceCache::DumpMemoryStatistics(dump);
    SkStrikeCache::DumpMemoryStatistics(dump);
}

// Source: include/core/SkGraphics.h:143-144
// Actual nullable retained immutable data and unique decoder owner, rather than
// an opaque callback/provider or an unowned pointer standing in for unique_ptr.
pub type ImageGeneratorFromEncodedDataFactory =
    unsafe fn(
        Option<std::sync::Arc<super::SkData::SkData>>,
    ) -> Option<Box<dyn super::SkImageGenerator::SkImageGenerator>>;

// Source: include/core/SkGraphics.h:20-20
pub use crate::include::core::SkOpenTypeSVGDecoder::SkOpenTypeSVGDecoder;
// Source: include/core/SkGraphics.h:163-164
// The nullable factory transfers its unique concrete decoder owner. Dynamic
// Rust drop dispatches the real implementation's destructor.
pub type OpenTypeSVGDecoderFactory =
    unsafe fn(svg: *const u8, length: usize) -> Option<Box<dyn SkOpenTypeSVGDecoder>>;
// Source: src/core/SkGraphics.cpp:114-114
static mut gSVGDecoderFactory: Option<OpenTypeSVGDecoderFactory> = None;
// Source: include/core/SkGraphics.h:165-165
// Source: src/core/SkGraphics.cpp:116-121
/// Safety: native global access must be externally synchronized. Initialize the
/// factory before concurrent use or exclude every concurrent reader/writer.
/// This stores only the original function pointer, not a retained decoder owner.
pub unsafe fn SetOpenTypeSVGDecoderFactory(
    factory: Option<OpenTypeSVGDecoderFactory>,
) -> Option<OpenTypeSVGDecoderFactory> {
    let old = unsafe { gSVGDecoderFactory };
    unsafe {
        gSVGDecoderFactory = factory;
    }
    old
}
// Source: include/core/SkGraphics.h:166-166
// Source: src/core/SkGraphics.cpp:123-125
/// Safety: same external global-access synchronization as the native setter.
pub unsafe fn GetOpenTypeSVGDecoderFactory() -> Option<OpenTypeSVGDecoderFactory> {
    unsafe { gSVGDecoderFactory }
}

// Source: include/core/SkGraphics.h:153-154
// The original definition/global lives at its codec source target, not here.
pub use crate::src::codec::SkImageGenerator_FromEncoded::SetImageGeneratorFromEncodedDataFactory;

// Source: include/core/SkGraphics.h:23-24
// Native has only static members and implicit public trivial special members.
// Rust ZST value representation is adapted, not an assertion of C++ empty-object ABI/size.
#[derive(Clone, Copy, Default)]
pub struct SkGraphics;
impl SkGraphics {
    // Source: include/core/SkGraphics.h:23-24 (implicit default construction).
    pub const fn new() -> Self {
        Self
    }
    // Source: include/core/SkGraphics.h:30
    pub fn Init() {
        crate::src::core::SkGraphics::Init();
    }
    // Source: include/core/SkGraphics.h:37
    pub fn GetFontCacheLimit() -> usize {
        crate::src::core::SkGraphics::GetFontCacheLimit()
    }
    // Source: include/core/SkGraphics.h:46
    pub fn SetFontCacheLimit(bytes: usize) -> usize {
        crate::src::core::SkGraphics::SetFontCacheLimit(bytes)
    }
    // Source: include/core/SkGraphics.h:51
    pub fn GetFontCacheUsed() -> usize {
        crate::src::core::SkGraphics::GetFontCacheUsed()
    }
    // Source: include/core/SkGraphics.h:57
    pub fn GetFontCacheCountUsed() -> i32 {
        crate::src::core::SkGraphics::GetFontCacheCountUsed()
    }
    // Source: include/core/SkGraphics.h:63
    pub fn GetFontCacheCountLimit() -> i32 {
        crate::src::core::SkGraphics::GetFontCacheCountLimit()
    }
    // Source: include/core/SkGraphics.h:70
    pub fn SetFontCacheCountLimit(count: i32) -> i32 {
        crate::src::core::SkGraphics::SetFontCacheCountLimit(count)
    }
    // Source: include/core/SkGraphics.h:76
    pub fn GetTypefaceCacheCountLimit() -> i32 {
        crate::src::core::SkGraphics::GetTypefaceCacheCountLimit()
    }
    // Source: include/core/SkGraphics.h:83
    pub fn SetTypefaceCacheCountLimit(count: i32) -> i32 {
        crate::src::core::SkGraphics::SetTypefaceCacheCountLimit(count)
    }
    // Source: include/core/SkGraphics.h:90
    pub fn PurgeFontCache() {
        crate::src::core::SkGraphics::PurgeFontCache();
    }
    // Source: include/core/SkGraphics.h:97
    pub fn PurgePinnedFontCache() {
        crate::src::core::SkGraphics::PurgePinnedFontCache();
    }
    // Source: include/core/SkGraphics.h:102
    pub fn GetResourceCacheTotalBytesUsed() -> usize {
        crate::src::core::SkGraphics::GetResourceCacheTotalBytesUsed()
    }
    // Source: include/core/SkGraphics.h:109
    pub fn GetResourceCacheTotalByteLimit() -> usize {
        crate::src::core::SkGraphics::GetResourceCacheTotalByteLimit()
    }
    // Source: include/core/SkGraphics.h:110
    pub fn SetResourceCacheTotalByteLimit(limit: usize) -> usize {
        crate::src::core::SkGraphics::SetResourceCacheTotalByteLimit(limit)
    }
    // Source: include/core/SkGraphics.h:116
    pub fn PurgeResourceCache() {
        crate::src::core::SkGraphics::PurgeResourceCache();
    }
    // Source: include/core/SkGraphics.h:126
    pub fn GetResourceCacheSingleAllocationByteLimit() -> usize {
        crate::src::core::SkGraphics::GetResourceCacheSingleAllocationByteLimit()
    }
    // Source: include/core/SkGraphics.h:127
    pub fn SetResourceCacheSingleAllocationByteLimit(limit: usize) -> usize {
        crate::src::core::SkGraphics::SetResourceCacheSingleAllocationByteLimit(limit)
    }
    // Source: include/core/SkGraphics.h:133; src/core/SkGraphics.cpp:35-38
    pub fn DumpMemoryStatistics(
        dump: &mut dyn crate::include::core::SkTraceMemoryDump::SkTraceMemoryDump,
    ) {
        crate::src::core::SkGraphics::DumpMemoryStatistics(dump);
    }
    // Source: include/core/SkGraphics.h:141
    pub fn PurgeAllCaches() {
        crate::src::core::SkGraphics::PurgeAllCaches();
    }
    // Source: include/core/SkGraphics.h:143-144
    // Requires the real nightly inherent_associated_types gate; root owns the crate attribute.
    pub type ImageGeneratorFromEncodedDataFactory =
        crate::src::core::SkGraphics::ImageGeneratorFromEncodedDataFactory;
    // Source: include/core/SkGraphics.h:153-154
    /// Safety: same externally synchronized native global-factory access as the existing free function.
    pub unsafe fn SetImageGeneratorFromEncodedDataFactory(
        factory: Option<Self::ImageGeneratorFromEncodedDataFactory>,
    ) -> Option<Self::ImageGeneratorFromEncodedDataFactory> {
        unsafe { crate::src::core::SkGraphics::SetImageGeneratorFromEncodedDataFactory(factory) }
    }
    // Source: include/core/SkGraphics.h:163-164
    pub type OpenTypeSVGDecoderFactory = crate::src::core::SkGraphics::OpenTypeSVGDecoderFactory;
    // Source: include/core/SkGraphics.h:165
    /// Safety: exclude all concurrent native global-factory readers/writers.
    pub unsafe fn SetOpenTypeSVGDecoderFactory(
        factory: Option<Self::OpenTypeSVGDecoderFactory>,
    ) -> Option<Self::OpenTypeSVGDecoderFactory> {
        unsafe { crate::src::core::SkGraphics::SetOpenTypeSVGDecoderFactory(factory) }
    }
    // Source: include/core/SkGraphics.h:166
    /// Safety: same synchronization contract as the original global getter.
    pub unsafe fn GetOpenTypeSVGDecoderFactory() -> Option<Self::OpenTypeSVGDecoderFactory> {
        unsafe { crate::src::core::SkGraphics::GetOpenTypeSVGDecoderFactory() }
    }
}
