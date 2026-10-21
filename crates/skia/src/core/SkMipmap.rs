// Copyright 2013 Google Inc. BSD-style license.
use super::SkPixmap::SkPixmap;
// Source: src/core/SkMipmap.h:24-24
// Nullable factory and nullable allocation result retain their distinct native
// meanings; an actual allocation owns the discardable-memory implementation.
pub type SkDiscardableFactoryProc =
    Option<unsafe fn(usize) -> Option<Box<dyn SkDiscardableMemory>>>;
// Source: src/core/SkMipmap.h:26-27
// Source: src/core/SkMipmap.h:29-30
// Box<dyn SkMipmapDownSampler> preserves the virtual destructor's unique ownership.
pub trait SkMipmapDownSampler {
    /// Source and destination descriptors must point to readable/writable pixel allocations of their declared dimensions.
    // Rust drawing-stage failures propagate through Build and its real callers;
    // native allocation/unsupported-format rejection remains a separate None.
    unsafe fn buildLevel(&self, dst: &SkPixmap, src: &SkPixmap) -> Result<(), crate::RasterError>;
}
use super::SkCachedData::{DataChange, SkCachedData};
use super::SkColorSpace::SkColorSpace;
use super::SkImageInfo::SkImageInfo;
use crate::include::core::SkSize::{SkISize, SkSize};
use crate::include::private::chromium::SkDiscardableMemory::SkDiscardableMemory;
use std::sync::Arc;
impl SkMipmap {
    // Source: src/core/SkMipmap.h:44-45
    /// Safety: source pixels stay readable and the factory returns locked memory.
    pub unsafe fn BuildWithContents(
        src: &SkPixmap,
        fact: SkDiscardableFactoryProc,
    ) -> Result<Option<Self>, crate::RasterError> {
        unsafe { Self::Build(src, fact, true) }
    }
    // Source: src/core/SkMipmap.h:53-53
    pub fn ComputeLevelCountForSize(size: SkISize) -> i32 {
        Self::ComputeLevelCount(size.width(), size.height())
    }
    // Source: src/core/SkMipmap.h:59-61
    pub fn ComputeLevelSizeForSize(size: SkISize, level: i32) -> SkISize {
        Self::ComputeLevelSize(size.width(), size.height(), level)
    }
}
// Source: src/core/SkMipmap.h:70-73
#[repr(C, align(8))]
/// Non-owning level descriptor. Its pixels remain in the same cached-data
/// allocation; copying this value never locks, retains, or reallocates pixels.
/// A caller must preserve the original mipmap's locked-data lifetime while using
/// addresses obtained from the descriptor.
pub struct Level {
    pub fPixmap: SkPixmap,
    pub fScale: SkSize,
}
// Source: src/core/SkMipmap.h:70-73
// Implicit native member copy; the level view never owns its pixel region.
impl Clone for Level {
    fn clone(&self) -> Self {
        Self {
            fPixmap: self.fPixmap.clone(),
            fScale: self.fScale,
        }
    }
    fn clone_from(&mut self, source: &Self) {
        self.fPixmap.clone_from(&source.fPixmap);
        self.fScale = source.fScale;
    }
}
impl SkMipmap {
    // Source: src/core/SkMipmap.h:87-87
    pub fn MakeDownSampler(src: &SkPixmap) -> Option<Box<dyn SkMipmapDownSampler>> {
        #[cfg(SK_USE_DRAWING_MIPMAP_DOWNSAMPLER)]
        {
            crate::src::core::SkMipmapDrawDownSampler::MakeDownSampler(src)
        }
        #[cfg(not(SK_USE_DRAWING_MIPMAP_DOWNSAMPLER))]
        {
            crate::src::core::SkMipmapHQDownSampler::MakeDownSampler(src)
        }
    }
}
// Source: src/core/SkMipmap.h:90-92
impl DataChange for MipmapState {
    fn onDataChange(&mut self, _old: *mut u8, new: *mut u8) {
        self.fLevels = new.cast();
    }
}
// Native sk_sp<SkMipmap> is this custom ref handle, not Arc: Clone/Drop must run cache lock transitions.
#[derive(Clone)]
pub struct SkMipmap {
    data: SkCachedData<MipmapState>,
}
// Inherited SkCachedData methods forward to the actual cached-data owner; no new source credit.
impl SkMipmap {
    pub fn size(&self) -> usize {
        self.data.size()
    }
    pub fn data(&self) -> *const u8 {
        self.data.data()
    }
    pub fn diagnostic_only_getDiscardable(
        &self,
    ) -> Option<std::ptr::NonNull<dyn SkDiscardableMemory>> {
        self.data.diagnostic_only_getDiscardable()
    }
    pub(crate) fn attachToCacheAndRef(&self) -> Self {
        Self {
            data: self.data.attachToCacheAndRef(),
        }
    }
    pub(crate) fn detachFromCacheAndUnref(self) {
        self.data.detachFromCacheAndUnref();
    }
}
// Source: src/core/SkMipmap.h:95-97
// Fields live in the actual cached-data subclass state; cached-data owns the contiguous level/pixel storage.
// Source: src/core/SkMipmap.cpp:32-32
// The default derived destructor releases fCS before the cached-data parent's
// backing storage. Level pixmaps in that raw block are intentionally not dropped:
// native placement construction stores them without color-space ownership
// (cpp116-118). Returned descriptors acquire fCS only via get/extractLevel.
struct MipmapState {
    // Source: src/core/SkMipmap.h:95-95
    fCS: Option<Arc<SkColorSpace>>,
    // Source: src/core/SkMipmap.h:96-96
    fLevels: *mut Level,
    // Source: src/core/SkMipmap.h:97-97
    fCount: i32,
}
// No blanket Send promise: the cached-data parent must qualify its real
// discardable allocation/operation lease before crossing threads.
impl SkMipmap {
    // Source: src/core/SkMipmap.cpp:34-43
    fn AllocLevelsSize(levelCount: i32, pixelSize: usize) -> usize {
        if levelCount < 0 {
            return 0;
        }
        let size =
            (levelCount as i64 + 1) * (std::mem::size_of::<Level>() as i64) + pixelSize as i64;
        if size < i32::MIN as i64 || size > i32::MAX as i64 {
            return 0;
        }
        size as i32 as usize
    }
}
impl SkMipmap {
    // Source: src/core/SkMipmap.cpp:45-49
    // Source: src/core/SkMipmap.cpp:51-52
    // Source: src/core/SkMipmap.cpp:55-60
    // Source: src/core/SkMipmap.cpp:62-65
    // Source: src/core/SkMipmap.cpp:67-78
    // Source: src/core/SkMipmap.cpp:80-81
    // Source: src/core/SkMipmap.cpp:84-87
    // Source: src/core/SkMipmap.cpp:89-95
    // Source: src/core/SkMipmap.cpp:99-99
    // Source: src/core/SkMipmap.cpp:101-108
    // Source: src/core/SkMipmap.cpp:110-113
    // Source: src/core/SkMipmap.cpp:118-120
    // Source: src/core/SkMipmap.cpp:122-129
    // Source: src/core/SkMipmap.cpp:131-133
    /// Source must describe readable pixels when computeContents is true. The factory returns locked native discardable memory.
    pub unsafe fn Build(
        src: &SkPixmap,
        fact: SkDiscardableFactoryProc,
        computeContents: bool,
    ) -> Result<Option<Self>, crate::RasterError> {
        if src.width() <= 1 && src.height() <= 1 {
            return Ok(None);
        }
        let ct = src.colorType();
        let at = src.alphaType();
        let mut size = 0usize;
        let count = Self::ComputeLevelCount(src.width(), src.height());
        for current in (0..=count).rev() {
            let dims = Self::ComputeLevelSize(src.width(), src.height(), current);
            size += super::SkImageInfoPriv::SkColorTypeMinRowBytes(ct, dims.fWidth)
                * dims.fHeight as usize;
        }
        let storage = Self::AllocLevelsSize(count, size);
        if storage == 0 {
            return Ok(None);
        }
        let state = MipmapState {
            fCS: None,
            fLevels: std::ptr::null_mut(),
            fCount: 0,
        };
        let data = if let Some(fact) = fact {
            let Some(dm) = fact(storage) else {
                return Ok(None);
            };
            SkCachedData::from_discardable(storage, dm, state)
        } else {
            unsafe extern "C" {
                fn malloc(size: usize) -> *mut std::ffi::c_void;
            }
            let p = malloc(storage) as *mut u8;
            if p.is_null() {
                return Ok(None);
            }
            SkCachedData::from_malloc(p, storage, state)
        };
        let mip = Self { data };
        let levels = mip.data.writable_data().cast::<Level>();
        crate::SkASSERT!(@source b"src/core/SkMipmap.cpp\0".as_ptr(),87; @condition b"mipmap->fLevels\0".as_ptr(); !levels.is_null());
        mip.data.with_state_mut(|s| {
            s.fCS = src.info().refColorSpace();
            s.fCount = count;
            s.fLevels = levels;
        });
        let base = levels.add(count as usize).cast::<u8>();
        let mut addr = base;
        let mut width = src.width();
        let mut height = src.height();
        let mut srcPM = src.clone();
        crate::SkASSERT!(@source b"src/core/SkMipmap.cpp\0".as_ptr(),99; @condition b"SkIsAlign8((uintptr_t)addr)\0".as_ptr(); addr as usize&7==0);
        let down = if computeContents {
            let Some(down) = Self::MakeDownSampler(src) else {
                // Source: src/core/SkMipmap.cpp:104-105
                // Explicit native delete precedes destruction of srcPM.
                drop(mip);
                return Ok(None);
            };
            Some(down)
        } else {
            None
        };
        for i in 0..count {
            width = 1.max(width >> 1);
            height = 1.max(height >> 1);
            let rb = super::SkImageInfoPriv::SkColorTypeMinRowBytes(ct, width) as u32;
            let info = SkImageInfo::Make(width, height, ct, at);
            let dst = SkPixmap::new(&info, addr, rb as usize);
            let scale = SkSize::Make(
                width as f32 / src.width() as f32,
                height as f32 / src.height() as f32,
            );
            std::ptr::write(
                levels.add(i as usize),
                Level {
                    fPixmap: dst,
                    fScale: scale,
                },
            );
            let dst = &(*levels.add(i as usize)).fPixmap;
            if let Some(down) = &down {
                down.buildLevel(dst, &srcPM)?;
            }
            srcPM.clone_from(dst);
            addr = addr.add(height as usize * rb as usize);
        }
        crate::SkASSERT!(@source b"src/core/SkMipmap.cpp\0".as_ptr(),129; @condition b"addr == baseAddr + size\0".as_ptr(); addr==base.add(size));
        crate::SkASSERT!(@source b"src/core/SkMipmap.cpp\0".as_ptr(),131; @condition b"mipmap->fLevels\0".as_ptr(); mip.data.with_state(|s|!s.fLevels.is_null()));
        Ok(Some(mip))
    }
}
impl SkMipmap {
    // Source: src/core/SkMipmap.cpp:135-138
    // Source: src/core/SkMipmap.cpp:145-146
    // Source: src/core/SkMipmap.cpp:148-150
    // Source: src/core/SkMipmap.cpp:153-153
    // Source: src/core/SkMipmap.cpp:156-156
    // Source: src/core/SkMipmap.cpp:162-164
    // Source: src/core/SkMipmap.cpp:166-167
    pub fn ComputeLevelCount(baseWidth: i32, baseHeight: i32) -> i32 {
        if baseWidth < 1 || baseHeight < 1 {
            return 0;
        }
        let largest = baseWidth.max(baseHeight);
        if largest < 2 {
            return 0;
        }
        let leading = (largest as u32).leading_zeros() as i32;
        let significant = 32 - leading;
        let mut count = significant;
        if count > 0 {
            count -= 1;
        }
        count
    }
}
impl SkMipmap {
    // Source: src/core/SkMipmap.cpp:169-172
    // Source: src/core/SkMipmap.cpp:174-177
    // Source: src/core/SkMipmap.cpp:186-187
    // Source: src/core/SkMipmap.cpp:189-190
    pub fn ComputeLevelSize(baseWidth: i32, baseHeight: i32, level: i32) -> SkISize {
        if baseWidth < 1 || baseHeight < 1 {
            return SkISize::Make(0, 0);
        }
        let max = Self::ComputeLevelCount(baseWidth, baseHeight);
        if level >= max || level < 0 {
            return SkISize::Make(0, 0);
        }
        SkISize::Make(
            1.max(baseWidth >> (level + 1)),
            1.max(baseHeight >> (level + 1)),
        )
    }
}
impl SkMipmap {
    // Source: src/core/SkMipmap.cpp:196-197
    // Source: src/core/SkMipmap.cpp:201-201
    // Source: src/core/SkMipmap.cpp:209-211
    // Source: src/core/SkMipmap.cpp:214-219
    pub fn ComputeLevel(scale: SkSize) -> f32 {
        crate::SkASSERT!(@source b"src/core/SkMipmap.cpp\0".as_ptr(),197; @condition b"scaleSize.width() >= 0 && scaleSize.height() >= 0\0".as_ptr(); scale.width()>=0.0&&scale.height()>=0.0);
        // SK_SUPPORT_LEGACY_ANISOTROPIC_MIPMAP_SCALE is not selected.
        let scale = if scale.height() < scale.width() {
            scale.height()
        } else {
            scale.width()
        };
        if scale >= 1.0 || scale <= 0.0 || !scale.is_finite() {
            return -1.0;
        }
        let raw = -scale.log2() - 0.5;
        let L = if raw < 0.0 { 0.0 } else { raw };
        if !L.is_finite() {
            return -1.0;
        }
        L
    }
}
impl SkMipmap {
    // Source: src/core/SkMipmap.cpp:221-224
    // Source: src/core/SkMipmap.cpp:226-230
    // Source: src/core/SkMipmap.cpp:232-236
    // Source: src/core/SkMipmap.cpp:238-241
    pub fn extractLevel(&self, scale: SkSize, level: Option<&mut Level>) -> bool {
        self.data.with_state(|s| {
            if s.fLevels.is_null() {
                return false;
            }
            let L = Self::ComputeLevel(scale);
            let mut index = crate::src::core::SkFloatingPoint::sk_float_round2int(L);
            if index <= 0 {
                return false;
            }
            if index > s.fCount {
                index = s.fCount;
            }
            if let Some(level) = level {
                unsafe {
                    level.clone_from(&*s.fLevels.add((index - 1) as usize));
                }
                level.fPixmap.setColorSpace(s.fCS.clone());
            }
            true
        })
    }
}
impl SkMipmap {
    // Source: src/core/SkMipmap.cpp:243-246
    // Source: src/core/SkMipmap.cpp:248-251
    // Source: src/core/SkMipmap.cpp:253-256
    // Source: src/core/SkMipmap.cpp:258-265
    pub fn validForRootLevel(&self, root: &SkImageInfo) -> bool {
        self.data.with_state(|s| {
            if s.fLevels.is_null() {
                return false;
            }
            let d = root.dimensions();
            if d.width() <= 1 && d.height() <= 1 {
                return false;
            }
            unsafe {
                let first = &(*s.fLevels).fPixmap;
                if first.width() != 1.max(d.width() >> 1)
                    || first.height() != 1.max(d.height() >> 1)
                {
                    return false;
                }
                for i in 0..s.fCount {
                    let p = &(*s.fLevels.add(i as usize)).fPixmap;
                    if p.colorType() != root.colorType() || p.alphaType() != root.alphaType() {
                        return false;
                    }
                }
            }
            true
        })
    }
}
impl SkMipmap {
    // Source: src/core/SkMipmap.cpp:269-275
    pub unsafe fn BuildBitmap(
        src: &crate::src::core::SkBitmap::SkBitmap,
        fact: SkDiscardableFactoryProc,
    ) -> Result<Option<Self>, crate::RasterError> {
        let mut pm = SkPixmap::default();
        if !src.peekPixels(Some(&mut pm)) {
            return Ok(None);
        }
        Self::Build(&pm, fact, true)
    }
}
impl SkMipmap {
    // Source: src/core/SkMipmap.cpp:277-279
    pub fn countLevels(&self) -> i32 {
        self.data.with_state(|s| s.fCount)
    }
}
impl SkMipmap {
    // Source: src/core/SkMipmap.cpp:281-292
    // Source: src/core/SkMipmap.cpp:294-297
    pub fn getLevel(&self, index: i32, level: Option<&mut Level>) -> bool {
        self.data.with_state(|s| {
            if s.fLevels.is_null() {
                return false;
            }
            if index < 0 {
                return false;
            }
            if index > s.fCount - 1 {
                return false;
            }
            if let Some(level) = level {
                unsafe {
                    level.clone_from(&*s.fLevels.add(index as usize));
                }
                level.fPixmap.setColorSpace(s.fCS.clone());
            }
            true
        })
    }
}
