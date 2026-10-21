use super::{
    SkArenaAlloc::SkArenaAlloc,
    SkBitmap::SkBitmap,
    SkBitmapCache::{SkBitmapCacheDesc, SkMipmapCache},
    SkMatrix::SkMatrix,
    SkMipmap::{Level, SkMipmap},
    SkPixmap::SkPixmap,
};
use crate::include::core::{SkSamplingOptions::SkMipmapMode, SkSize::SkSize};
use crate::src::image::{
    SkImage::{CachingHint, SkImage},
    SkImage_Base::SkImage_Base,
};
impl SkMipmapAccessor {
    // Source: src/core/SkMipmapAccessor.h:31-34
    pub fn level(&self) -> (SkPixmap, SkMatrix) {
        crate::SkASSERT!(@source b"src/core/SkMipmapAccessor.h\0".as_ptr(),32; @condition b"fUpper.addr() != nullptr\0".as_ptr(); !self.fUpper.addr().is_null());
        (self.fUpper.clone(), self.fUpperInv.clone())
    }
    // Source: src/core/SkMipmapAccessor.h:36-39
    pub fn lowerLevel(&self) -> (SkPixmap, SkMatrix) {
        crate::SkASSERT!(@source b"src/core/SkMipmapAccessor.h\0".as_ptr(),37; @condition b"fLower.addr() != nullptr\0".as_ptr(); !self.fLower.addr().is_null());
        (self.fLower.clone(), self.fLowerInv.clone())
    }
    // Source: src/core/SkMipmapAccessor.h:42-42
    pub fn lowerWeight(&self) -> f32 {
        self.fLowerWeight
    }
}
// Source: src/core/SkMipmapAccessor.h:45-49
// Source: src/core/SkMipmapAccessor.h:52-53
// Reversed declaration order preserves native reverse member destruction: Mipmap
// cache client first, Bitmap backing next, matrices, then lower/upper Pixmap colorspaces.
// There is no Clone implementation, matching native SkNoncopyable.
pub struct SkMipmapAccessor {
    fCurrMip: Option<SkMipmap>,
    fBaseStorage: SkBitmap,
    fLowerInv: SkMatrix,
    fUpperInv: SkMatrix,
    fLowerWeight: f32,
    fLower: SkPixmap,
    fUpper: SkPixmap,
}
// Source: src/core/SkMipmapAccessor.cpp:24-33
#[cfg(all(target_arch = "aarch64", target_pointer_width = "64"))]
fn try_load_mips(image: &dyn SkImage_Base) -> Result<Option<SkMipmap>, crate::RasterError> {
    let mut mips = image.refMips();
    if mips.is_none() {
        mips = SkMipmapCache::FindAndRef(&SkBitmapCacheDesc::MakeFromImage(image), None);
    }
    if mips.is_none() {
        mips = SkMipmapCache::AddAndRef(image, None)?;
    }
    Ok(mips)
}
#[cfg(all(target_arch = "aarch64", target_pointer_width = "64"))]
impl SkMipmapAccessor {
    // Source: src/core/SkMipmapAccessor.cpp:35-38
    // Source: src/core/SkMipmapAccessor.cpp:40-40
    // Source: src/core/SkMipmapAccessor.cpp:42-42
    // Source: src/core/SkMipmapAccessor.cpp:44-47
    // Source: src/core/SkMipmapAccessor.cpp:49-61
    // Source: src/core/SkMipmapAccessor.cpp:63-66
    // Source: src/core/SkMipmapAccessor.cpp:70-73
    // Source: src/core/SkMipmapAccessor.cpp:75-77
    // Source: src/core/SkMipmapAccessor.cpp:79-85
    // Source: src/core/SkMipmapAccessor.cpp:87-95
    // Source: src/core/SkMipmapAccessor.cpp:97-109
    pub fn new(
        image: &dyn SkImage_Base,
        inv: &SkMatrix,
        requestedMode: SkMipmapMode,
    ) -> Result<Self, crate::RasterError> {
        // These native members have implicit default constructors. Unused matrix/
        // level values remain inaccessible until their native validity conditions hold.
        let mut out = Self {
            fCurrMip: None,
            fBaseStorage: SkBitmap::default(),
            fLowerInv: SkMatrix::default(),
            fUpperInv: SkMatrix::default(),
            fLowerWeight: 0.0,
            fLower: SkPixmap::default(),
            fUpper: SkPixmap::default(),
        };
        let mut resolvedMode = requestedMode;
        out.fLowerWeight = 0.0;
        let load_upper_from_base = |this: &mut Self| {
            if this.fBaseStorage.getPixels().is_null() {
                // Adapted native43: CPU directContext() is fixed nullptr;
                // no opaque/dereferenceable GPU context object is introduced.
                let _ =
                    image.getROPixelsCPU(&mut this.fBaseStorage, CachingHint::kAllow_CachingHint);
                this.fUpper.reset_with(
                    this.fBaseStorage.info(),
                    this.fBaseStorage.getPixels(),
                    this.fBaseStorage.rowBytes(),
                );
            }
        };
        let mut level = 0.0;
        if requestedMode != SkMipmapMode::kNone {
            let mut scale = SkSize::MakeEmpty();
            if !inv.decomposeScale(Some(&mut scale), None) {
                resolvedMode = SkMipmapMode::kNone;
            } else {
                level =
                    SkMipmap::ComputeLevel(SkSize::Make(1.0 / scale.width(), 1.0 / scale.height()));
                if level <= 0.0 {
                    resolvedMode = SkMipmapMode::kNone;
                    level = 0.0;
                }
            }
        }
        let scale = |pm: &SkPixmap| {
            SkMatrix::Scale(
                pm.width() as f32 / image.width() as f32,
                pm.height() as f32 / image.height() as f32,
            )
        };
        let levelNum = if resolvedMode == SkMipmapMode::kNearest {
            super::SkFloatingPoint::sk_float_round2int(level)
        } else {
            super::SkFloatingPoint::sk_float_floor2int(level)
        };
        let lowerWeight = level - levelNum as f32;
        crate::SkASSERT!(@source b"src/core/SkMipmapAccessor.cpp\0".as_ptr(),73; @condition b"levelNum >= 0\0".as_ptr(); levelNum>=0);
        if levelNum == 0 {
            load_upper_from_base(&mut out);
        }
        if levelNum > 0 || (resolvedMode == SkMipmapMode::kLinear && lowerWeight > 0.0) {
            out.fCurrMip = try_load_mips(image)?;
            if out.fCurrMip.is_none() {
                load_upper_from_base(&mut out);
                resolvedMode = SkMipmapMode::kNone;
            } else {
                let mut levelRec = Level {
                    fPixmap: SkPixmap::default(),
                    fScale: SkSize::MakeEmpty(),
                };
                crate::SkASSERT!(@source b"src/core/SkMipmapAccessor.cpp\0".as_ptr(),87; @condition b"resolvedMode != SkMipmapMode::kNone\0".as_ptr(); resolvedMode!=SkMipmapMode::kNone);
                if levelNum > 0 {
                    if out
                        .fCurrMip
                        .as_ref()
                        .unwrap()
                        .getLevel(levelNum - 1, Some(&mut levelRec))
                    {
                        out.fUpper = levelRec.fPixmap.clone();
                    } else {
                        load_upper_from_base(&mut out);
                        resolvedMode = SkMipmapMode::kNone;
                    }
                }
                if resolvedMode == SkMipmapMode::kLinear {
                    if out
                        .fCurrMip
                        .as_ref()
                        .unwrap()
                        .getLevel(levelNum, Some(&mut levelRec))
                    {
                        out.fLower = levelRec.fPixmap.clone();
                        out.fLowerWeight = lowerWeight;
                        out.fLowerInv = scale(&out.fLower);
                    } else {
                        resolvedMode = SkMipmapMode::kNearest;
                    }
                }
            }
        }
        out.fUpperInv = scale(&out.fUpper);
        Ok(out)
    }
    // Source: src/core/SkMipmapAccessor.cpp:111-112
    // Source: src/core/SkMipmapAccessor.cpp:115-116
    /// The returned pointer borrows the arena allocation; retain the arena while using it.
    pub fn Make(
        alloc: &mut SkArenaAlloc,
        image: &dyn SkImage,
        inv: &SkMatrix,
        mipmap: SkMipmapMode,
    ) -> Result<Option<std::ptr::NonNull<Self>>, crate::RasterError> {
        // Adapted/uncredited native113: safe image virtual calls can panic. Build
        // the complete address-independent owner before installing an arena drop
        // footer, then perform only a pure move into the actual arena allocation.
        // Pixmaps point into retained Bitmap/Mipmap backing, never into this header.
        let value = Self::new(image.asImageBase(), inv, mipmap)?;
        let access = alloc.make(value);
        Ok(if !unsafe { (*access).fUpper.addr().is_null() } {
            std::ptr::NonNull::new(access)
        } else {
            None
        })
    }
}
