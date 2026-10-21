// Canonical mask-cache records use the native non-derived cached-data factory result.
// All pinned CPU callers originate in BlurMaskFilterImpl::copy_mask_to_cacheddata,
// which calls ResourceCache::{newCachedData,NewCachedData} and constructs base data.
// Arbitrary derived cached-data admission requires a real erased base interface first;
// it must not change these native key namespaces or downcast by a caller-chosen state type.
use crate::include::core::SkBlurTypes::SkBlurStyle;

use crate::include::core::SkSize::SkSize;

use crate::src::core::SkCachedData::SkCachedData;

use crate::src::core::SkMask::SkMask;

use crate::src::core::SkRRect::SkRRect;

use crate::src::core::SkRect::SkRect;

use crate::src::core::SkResourceCache::{Key, Rec, RecLinks};

// Source: src/core/SkMaskCache.cpp:28-31
pub struct MaskValue {
    pub fMask: SkMask,
    pub fData: SkCachedData<()>,
}

// Source: src/core/SkMaskCache.cpp:34-34
static RRECT_NAMESPACE: u32 = 0;

// Source: src/core/SkMaskCache.cpp:36-37
impl RRectBlurKey {
    // Source: src/core/SkMaskCache.cpp:38-45
    pub fn new(sigma: f32, rrect: &SkRRect, style: SkBlurStyle) -> Self {
        // Explicit native words avoid hashing Rust struct padding or container pointers.
        // The native RRect payload is rect4,radii8,type1 words, in that order.
        let mut words = Vec::with_capacity(15);
        words.push(sigma.to_bits());
        words.push(style as i32 as u32);
        let rect = rrect.rect();
        words.extend([
            rect.fLeft.to_bits(),
            rect.fTop.to_bits(),
            rect.fRight.to_bits(),
            rect.fBottom.to_bits(),
        ]);
        for p in rrect.all_radii() {
            words.extend([p.fX.to_bits(), p.fY.to_bits()])
        }
        words.push(rrect.getType() as i32 as u32);
        Self {
            key: Key::new((&RRECT_NAMESPACE as *const u32).cast(), 0, &words),
            fSigma: sigma,
            fStyle: style as i32,
            fRRect: *rrect,
        }
    }
}

// Source: src/core/SkMaskCache.cpp:47-50
pub struct RRectBlurKey {
    key: Key,
    pub fSigma: f32,
    pub fStyle: i32,
    pub fRRect: SkRRect,
}

// Source: src/core/SkMaskCache.cpp:52-52
impl RRectBlurRec {
    // Source: src/core/SkMaskCache.cpp:53-57
    pub fn new(key: RRectBlurKey, mask: &SkMask, data: &SkCachedData<()>) -> Self {
        let value = MaskValue {
            fMask: SkMask::new(std::ptr::null(), mask.fBounds, mask.fRowBytes, mask.fFormat),
            fData: data.attachToCacheAndRef(),
        };
        Self {
            fKey: key,
            links: RecLinks::new(),
            fValue: Some(value),
        }
    }
}

// Source: src/core/SkMaskCache.cpp:58-60
impl Drop for RRectBlurRec {
    fn drop(&mut self) {
        let value = self.fValue.take().unwrap();
        value.fData.detachFromCacheAndUnref();
    }
}

// Source: src/core/SkMaskCache.cpp:62-63
pub struct RRectBlurRec {
    pub fKey: RRectBlurKey,
    links: RecLinks,
    fValue: Option<MaskValue>,
}

impl RRectBlurRec {
    pub fn getKey(&self) -> &Key {
        &self.fKey.key
    }
}

impl RRectBlurRec {
    pub fn getCategory(&self) -> &'static str {
        "rrect-blur"
    }
}

// Stored masks always have null fImage; pixels belong to the synchronized cached-data
// owner. RecLinks is touched only by the externally synchronized owning cache.
unsafe impl Send for RRectBlurRec {}
// The unique embedded links/key stay fixed while the owning cache pins this boxed record.
#[cfg(target_pointer_width = "64")]
unsafe impl Rec for RRectBlurRec {
    fn links(&self) -> &RecLinks {
        &self.links
    }
    fn links_mut(&mut self) -> &mut RecLinks {
        &mut self.links
    }
    // Source: src/core/SkMaskCache.cpp:65-65
    fn getKey(&self) -> &Key {
        &self.fKey.key
    }
    // Source: src/core/SkMaskCache.cpp:66-66
    // Adapted budget: native layout overhead is retained; Rust host allocation layout differs.
    fn bytesUsed(&self) -> usize {
        native_record_overhead(60) + self.fValue.as_ref().unwrap().fData.size()
    }
    // Source: src/core/SkMaskCache.cpp:67-67
    fn getCategory(&self) -> *const std::ffi::c_char {
        b"rrect-blur\0".as_ptr().cast()
    }
    // Source: src/core/SkMaskCache.cpp:68-70
    fn diagnostic_only_getDiscardable(
        &self,
    ) -> Option<
        std::ptr::NonNull<
            dyn crate::include::private::chromium::SkDiscardableMemory::SkDiscardableMemory,
        >,
    > {
        self.fValue
            .as_ref()
            .unwrap()
            .fData
            .diagnostic_only_getDiscardable()
    }
}
#[cfg(target_pointer_width = "64")]
impl RRectBlurRec {
    pub fn visitor(&self) -> Option<MaskValue> {
        Self::Visitor(self)
    }
    // Source: src/core/SkMaskCache.cpp:72-74
    // Source: src/core/SkMaskCache.cpp:76-84
    pub fn Visitor(base_rec: &dyn Rec) -> Option<MaskValue> {
        let rec = (base_rec as &dyn std::any::Any)
            .downcast_ref::<Self>()
            .expect("matching native mask record type");
        let value = rec.fValue.as_ref().unwrap();
        let tmp_data = value.fData.clone();
        if tmp_data.data().is_null() {
            drop(tmp_data);
            return None;
        }
        Some(MaskValue {
            fMask: value.fMask,
            fData: tmp_data,
        })
    }
}

// Bridge the native SkTLazy result/context without allocating or changing visitor ownership.
#[cfg(target_pointer_width = "64")]
unsafe fn rrect_find_visitor(rec: &dyn Rec, context: *mut std::ffi::c_void) -> bool {
    if let Some(value) = RRectBlurRec::Visitor(rec) {
        *context.cast::<Option<MaskValue>>() = Some(value);
        true
    } else {
        false
    }
}
// Source: src/core/SkMaskCache.cpp:88-95
// Source: src/core/SkMaskCache.cpp:97-100
#[cfg(target_pointer_width = "64")]
/// Safety: callers obey the native cache callback/no-reentry contract. The returned
/// cached-data client keeps mask pixels locked; the nonowning mask must not outlive it.
pub unsafe fn FindAndRefRRect(
    sigma: f32,
    style: SkBlurStyle,
    rrect: &SkRRect,
    mask: &mut Option<SkMask>,
    local_cache: Option<&mut crate::src::core::SkResourceCache::SkResourceCache>,
) -> Option<SkCachedData<()>> {
    let mut result: Option<MaskValue> = None;
    let key = RRectBlurKey::new(sigma, rrect, style);
    let context = (&mut result as *mut Option<MaskValue>).cast();
    let found = match local_cache {
        Some(cache) => cache.find(&key.key, rrect_find_visitor, context),
        None => crate::src::core::SkResourceCache::Find(&key.key, rrect_find_visitor, context),
    };
    if !found {
        return None;
    }
    let result = result.expect("successful native visitor initializes result");
    *mask = Some(SkMask::new(
        result.fData.data(),
        result.fMask.fBounds,
        result.fMask.fRowBytes,
        result.fMask.fFormat,
    ));
    Some(result.fData)
}
// Source: src/core/SkMaskCache.cpp:102-107
#[cfg(target_pointer_width = "64")]
pub unsafe fn AddRRect(
    sigma: f32,
    style: SkBlurStyle,
    rrect: &SkRRect,
    mask: &SkMask,
    data: &SkCachedData<()>,
    local_cache: Option<&mut crate::src::core::SkResourceCache::SkResourceCache>,
) {
    let key = RRectBlurKey::new(sigma, rrect, style);
    let rec = Box::new(RRectBlurRec::new(key, mask, data));
    match local_cache {
        Some(cache) => cache.add(rec, std::ptr::null_mut()),
        None => crate::src::core::SkResourceCache::Add(rec, std::ptr::null_mut()),
    }
}

// Source: src/core/SkMaskCache.cpp:112-112
static RECTS_NAMESPACE: u32 = 0;

// Source: src/core/SkMaskCache.cpp:114-115
impl RectsBlurKey {
    // Source: src/core/SkMaskCache.cpp:116-129
    // Source: src/core/SkMaskCache.cpp:131-133
    pub fn new(sigma: f32, style: SkBlurStyle, rects: &[SkRect]) -> Self {
        debug_assert!(rects.len() == 1 || rects.len() == 2);
        let ir = rects[0].roundOut();
        let mut sizes = [SkSize::Make(0., 0.); 4];
        sizes[0] = SkSize::Make(rects[0].width(), rects[0].height());
        if rects.len() == 2 {
            sizes[1] = SkSize::Make(rects[1].width(), rects[1].height());
            sizes[2] = SkSize::Make(
                rects[0].fLeft - rects[1].fLeft,
                rects[0].fTop - rects[1].fTop,
            )
        } else {
            sizes[1] = SkSize::Make(0., 0.);
            sizes[2] = SkSize::Make(0., 0.)
        }
        sizes[3] = SkSize::Make(
            rects[0].fLeft - ir.fLeft as f32,
            rects[0].fTop - ir.fTop as f32,
        );
        let mut words = Vec::with_capacity(10);
        words.push(sigma.to_bits());
        words.push(style as i32 as u32);
        for s in sizes {
            words.extend([s.fWidth.to_bits(), s.fHeight.to_bits()])
        }
        Self {
            key: Key::new((&RECTS_NAMESPACE as *const u32).cast(), 0, &words),
            fSigma: sigma,
            fStyle: style as i32,
            fSizes: sizes,
        }
    }
}

// Source: src/core/SkMaskCache.cpp:135-138
pub struct RectsBlurKey {
    key: Key,
    pub fSigma: f32,
    pub fStyle: i32,
    pub fSizes: [SkSize; 4],
}

// Source: src/core/SkMaskCache.cpp:140-140
impl RectsBlurRec {
    // Source: src/core/SkMaskCache.cpp:141-145
    pub fn new(key: RectsBlurKey, mask: &SkMask, data: &SkCachedData<()>) -> Self {
        let value = MaskValue {
            fMask: SkMask::new(std::ptr::null(), mask.fBounds, mask.fRowBytes, mask.fFormat),
            fData: data.attachToCacheAndRef(),
        };
        Self {
            fKey: key,
            links: RecLinks::new(),
            fValue: Some(value),
        }
    }
}

// Source: src/core/SkMaskCache.cpp:146-148
impl Drop for RectsBlurRec {
    fn drop(&mut self) {
        let value = self.fValue.take().unwrap();
        value.fData.detachFromCacheAndUnref();
    }
}

// Source: src/core/SkMaskCache.cpp:150-151
pub struct RectsBlurRec {
    pub fKey: RectsBlurKey,
    links: RecLinks,
    fValue: Option<MaskValue>,
}

impl RectsBlurRec {
    pub fn getKey(&self) -> &Key {
        &self.fKey.key
    }
}

impl RectsBlurRec {
    pub fn getCategory(&self) -> &'static str {
        "rects-blur"
    }
}

// Stored masks always have null fImage; pixels belong to the synchronized cached-data
// owner. RecLinks is touched only by the externally synchronized owning cache.
unsafe impl Send for RectsBlurRec {}
// The unique embedded links/key stay fixed while the owning cache pins this boxed record.
#[cfg(target_pointer_width = "64")]
unsafe impl Rec for RectsBlurRec {
    fn links(&self) -> &RecLinks {
        &self.links
    }
    fn links_mut(&mut self) -> &mut RecLinks {
        &mut self.links
    }
    // Source: src/core/SkMaskCache.cpp:153-153
    fn getKey(&self) -> &Key {
        &self.fKey.key
    }
    // Source: src/core/SkMaskCache.cpp:154-154
    // Adapted budget: native layout overhead is retained; Rust host allocation layout differs.
    fn bytesUsed(&self) -> usize {
        native_record_overhead(40) + self.fValue.as_ref().unwrap().fData.size()
    }
    // Source: src/core/SkMaskCache.cpp:155-155
    fn getCategory(&self) -> *const std::ffi::c_char {
        b"rects-blur\0".as_ptr().cast()
    }
    // Source: src/core/SkMaskCache.cpp:156-158
    fn diagnostic_only_getDiscardable(
        &self,
    ) -> Option<
        std::ptr::NonNull<
            dyn crate::include::private::chromium::SkDiscardableMemory::SkDiscardableMemory,
        >,
    > {
        self.fValue
            .as_ref()
            .unwrap()
            .fData
            .diagnostic_only_getDiscardable()
    }
}
#[cfg(target_pointer_width = "64")]
impl RectsBlurRec {
    pub fn visitor(&self) -> Option<MaskValue> {
        Self::Visitor(self)
    }
    // Source: src/core/SkMaskCache.cpp:160-162
    // Source: src/core/SkMaskCache.cpp:164-172
    pub fn Visitor(base_rec: &dyn Rec) -> Option<MaskValue> {
        let rec = (base_rec as &dyn std::any::Any)
            .downcast_ref::<Self>()
            .expect("matching native mask record type");
        let value = rec.fValue.as_ref().unwrap();
        let tmp_data = value.fData.clone();
        if tmp_data.data().is_null() {
            drop(tmp_data);
            return None;
        }
        Some(MaskValue {
            fMask: value.fMask,
            fData: tmp_data,
        })
    }
}

#[cfg(target_pointer_width = "64")]
unsafe fn rects_find_visitor(rec: &dyn Rec, context: *mut std::ffi::c_void) -> bool {
    if let Some(value) = RectsBlurRec::Visitor(rec) {
        *context.cast::<Option<MaskValue>>() = Some(value);
        true
    } else {
        false
    }
}
// Source: src/core/SkMaskCache.cpp:176-185
// Source: src/core/SkMaskCache.cpp:187-190
#[cfg(target_pointer_width = "64")]
/// Safety: same native callback/no-reentry and returned mask/client lifetime contract
/// as FindAndRefRRect. Rect count must be one or two, as in the native key constructor.
pub unsafe fn FindAndRefRects(
    sigma: f32,
    style: SkBlurStyle,
    rects: &[SkRect],
    mask: &mut Option<SkMask>,
    local_cache: Option<&mut crate::src::core::SkResourceCache::SkResourceCache>,
) -> Option<SkCachedData<()>> {
    let mut result: Option<MaskValue> = None;
    let key = RectsBlurKey::new(sigma, style, rects);
    let context = (&mut result as *mut Option<MaskValue>).cast();
    let found = match local_cache {
        Some(cache) => cache.find(&key.key, rects_find_visitor, context),
        None => crate::src::core::SkResourceCache::Find(&key.key, rects_find_visitor, context),
    };
    if !found {
        return None;
    }
    let result = result.expect("successful native visitor initializes result");
    *mask = Some(SkMask::new(
        result.fData.data(),
        result.fMask.fBounds,
        result.fMask.fRowBytes,
        result.fMask.fFormat,
    ));
    Some(result.fData)
}
// Source: src/core/SkMaskCache.cpp:192-200
#[cfg(target_pointer_width = "64")]
pub unsafe fn AddRects(
    sigma: f32,
    style: SkBlurStyle,
    rects: &[SkRect],
    mask: &SkMask,
    data: &SkCachedData<()>,
    local_cache: Option<&mut crate::src::core::SkResourceCache::SkResourceCache>,
) {
    let key = RectsBlurKey::new(sigma, style, rects);
    let rec = Box::new(RectsBlurRec::new(key, mask, data));
    match local_cache {
        Some(cache) => cache.add(rec, std::ptr::null_mut()),
        None => crate::src::core::SkResourceCache::Add(rec, std::ptr::null_mut()),
    }
}

// Logical native eviction accounting, independent of Rust Box/Arc/trait-object allocations.
// 64-bit values152/128 verified against pinned C++ sizeof. The32-bit C++ ABI
// (including key/member alignment) remains pending; pointer width alone is insufficient.
// This does not claim host heap bytes or ABI equivalence.
#[cfg(target_pointer_width = "64")]
fn native_record_overhead(key_payload: usize) -> usize {
    let ptr = std::mem::size_of::<usize>();
    let align = |n: usize| (n + ptr - 1) & !(ptr - 1);
    let base_rec = 3 * ptr;
    let key = align(16 + ptr + key_payload);
    let mask_value = align(ptr + 16 + 4 + 1) + ptr;
    base_rec + key + mask_value
}
