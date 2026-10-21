// Fixed upstream cacf77bdba7ba7df8ea7236d7e14b08c658ff368; unverified.
use super::SkImage::{
    CachingHint, ImageCore, ImageVirtuals, RequiredProperties, SkImage, SkImageState,
};
use super::SkImage_Base::{SkImageBaseState, SkImage_Base, Type};
use crate::include::core::{SkColorType::SkColorType, SkRecorder::SkRecorder};
use crate::src::core::{
    SkBitmap::SkBitmap,
    SkBitmapCache::{SkBitmapCache, SkBitmapCacheDesc},
    SkColorSpace::SkColorSpace,
    SkData::SkData,
    SkIDChangeListener,
    SkImageGenerator::SkImageGenerator,
    SkImageInfo::SkImageInfo,
    SkPixmap::SkPixmap,
    SkRect::SkIRect,
};
use core::ops::Deref;
use std::sync::{Arc, Mutex, MutexGuard, Weak};
// Source: src/image/SkImage_Lazy.h:106-124.
// Same sole unique generator and original separate exclusion field. Rust fields
// express native reverse teardown: unlock/backend field before generator owner.
// No copied ImageInfo/ColorSpace owner or duplicate unique-ID state is retained.
// Concrete mutable pointee/cache fields carry their own UnsafeCell domains;
// this owner and its base metadata are only shared-borrowed. No Send/Sync claim.
pub struct SharedGenerator {
    // Source: src/image/SkImage_Lazy.h:120.
    fMutex: Mutex<()>,
    // Source: src/image/SkImage_Lazy.h:119.
    fGenerator: Box<dyn SkImageGenerator>,
}
impl SharedGenerator {
    // Source: src/image/SkImage_Lazy.cpp:32-34.
    pub fn Make(gen: Option<Box<dyn SkImageGenerator>>) -> Option<Arc<Self>> {
        gen.map(|gen| Arc::new(Self::new(gen)))
    }
    // Source: src/image/SkImage_Lazy.cpp:36-39.
    fn new(gen: Box<dyn SkImageGenerator>) -> Self {
        let fGenerator = gen;
        let fMutex = Mutex::new(());
        Self { fMutex, fGenerator }
    }
    // Source: src/image/SkImage_Lazy.cpp:41.
    // Native immutable base field, borrowed from the same unique generator. The
    // mutable virtual protocol now borrows only shared Self and its disjoint
    // UnsafeCell payload/cache fields, so this reference remains valid across it.
    pub fn getInfo(&self) -> &SkImageInfo {
        &self.fGenerator.state().fInfo
    }
    // Source: src/image/SkImage_Lazy.cpp:61.
    fn uniqueID(&self) -> u32 {
        self.fGenerator.state().fUniqueID
    }
    // Source: src/image/SkImage_Lazy.cpp:43.
    pub fn isTextureGenerator(&self) -> bool {
        self.fGenerator.isTextureGenerator()
    }
}
// Source: src/image/SkImage_Lazy.h:39-48
// Adapted private field representation: native reverse member destruction.
// Constructor expressions below retain their original source evaluation order.
pub struct Validator {
    // Source: src/image/SkImage_Lazy.h:47-47
    pub fUniqueID: u32,
    // Source: src/image/SkImage_Lazy.h:46-46
    pub fColorSpace: Option<Arc<SkColorSpace>>,
    // Source: src/image/SkImage_Lazy.h:45-45
    pub fInfo: SkImageInfo,
    // Source: src/image/SkImage_Lazy.h:44-44
    pub fSharedGenerator: Option<Arc<SharedGenerator>>,
}
impl Validator {
    // Source: src/image/SkImage_Lazy.cpp:47-76
    pub fn new(
        gen: Option<Arc<SharedGenerator>>,
        mut colorType: Option<SkColorType>,
        colorSpace: Option<Arc<SkColorSpace>>,
    ) -> Self {
        let mut out = Self {
            fSharedGenerator: gen,
            fInfo: SkImageInfo::default(),
            fColorSpace: None,
            fUniqueID: 0,
        };
        let Some(gen) = out.fSharedGenerator.as_ref() else {
            return out;
        };
        out.fInfo = gen.getInfo().clone();
        if out.fInfo.isEmpty() {
            out.fSharedGenerator = None;
            return out;
        }
        out.fUniqueID = gen.uniqueID();
        if colorType == Some(out.fInfo.colorType()) {
            colorType = None;
        }
        if colorType.is_some() || colorSpace.is_some() {
            if let Some(ct) = colorType {
                out.fInfo = out.fInfo.makeColorType(ct);
            }
            if colorSpace.is_some() {
                out.fInfo = out.fInfo.makeColorSpace(colorSpace);
            }
            out.fUniqueID = crate::src::core::SkNextID::SkNextID::ImageID();
        }
        out
    }
}
// Source: src/image/SkImage_Lazy.cpp:81-101.
// Guard borrows the same shared owner; no additional Arc/refcount is made.
// The guard remains alive until after the decoder operation and final unlock.
// Native assertHeld is represented by the unforgeable held MutexGuard.
pub(super) struct ScopedGenerator<'owner> {
    fAutoAcquire: MutexGuard<'owner, ()>,
    fSharedGenerator: &'owner SharedGenerator,
}
impl Deref for ScopedGenerator<'_> {
    type Target = dyn SkImageGenerator;
    fn deref(&self) -> &Self::Target {
        self.fSharedGenerator.fGenerator.as_ref()
    }
}
impl SharedGenerator {
    // Original lock range starts with ScopedGenerator construction and ends at its
    // destruction. No const-metadata getter is wrapped in an extra lock.
    pub(super) fn scoped(&self) -> ScopedGenerator<'_> {
        let fAutoAcquire = self.fMutex.lock().unwrap_or_else(|p| p.into_inner());
        ScopedGenerator {
            fSharedGenerator: self,
            fAutoAcquire,
        }
    }
}
// Source: src/image/SkImage_Lazy.h:96-103
// Rust drops members forward: listener list before cached result, generator and
// base. The weak self handle projects the same Arc owner for native const-this
// calls into public Arc-taking facades; it never creates a second refcount.
pub struct SkImage_Lazy {
    fUniqueIDListeners: SkIDChangeListener::List,
    fOnMakeColorTypeAndSpaceResult: Mutex<Option<Arc<dyn SkImage>>>,
    fSharedGenerator: Arc<SharedGenerator>,
    base: SkImageBaseState,
    self_owner: Weak<Self>,
}
impl SkImage_Lazy {
    // Source: src/image/SkImage_Lazy.cpp:105-110
    pub fn new(validator: Validator) -> Arc<Self> {
        Arc::new_cyclic(|weak| Self {
            fUniqueIDListeners: Default::default(),
            fOnMakeColorTypeAndSpaceResult: Mutex::new(None),
            fSharedGenerator: validator.fSharedGenerator.expect("native valid generator"),
            base: SkImageBaseState::new(&validator.fInfo, validator.fUniqueID, None),
            self_owner: weak.clone(),
        })
    }
    // Source: src/image/SkImage_Lazy.cpp:105-110
    // Derived construction embeds the original lazy subobject in its sole image
    // owner. Its native subset override never enters lazy's self-owner subset path.
    pub(super) fn derived_value(validator: Validator) -> Self {
        Self {
            fUniqueIDListeners: Default::default(),
            fOnMakeColorTypeAndSpaceResult: Mutex::new(None),
            fSharedGenerator: validator.fSharedGenerator.expect("native valid generator"),
            base: SkImageBaseState::new(&validator.fInfo, validator.fUniqueID, None),
            self_owner: Weak::new(),
        }
    }
    // Source: src/image/SkImage_Lazy.cpp:112-156
    pub fn getROPixels(&self, bitmap: &mut SkBitmap, chint: CachingHint) -> bool {
        unsafe {
            let check = |bitmap: &SkBitmap| {
                crate::SkASSERT!(@source b"src/image/SkImage_Lazy.cpp\0".as_ptr(),115; @condition b"bitmap->isImmutable()\0".as_ptr(); bitmap.isImmutable());
                crate::SkASSERT!(@source b"src/image/SkImage_Lazy.cpp\0".as_ptr(),116; @condition b"bitmap->getPixels()\0".as_ptr(); !bitmap.getPixels().is_null());
            };
            let desc = SkBitmapCacheDesc::MakeFromImage(self);
            if SkBitmapCache::Find(&desc, bitmap) {
                check(bitmap);
                return true;
            }
            if chint == CachingHint::kAllow_CachingHint {
                let mut pmap = SkPixmap::default();
                let Some(rec) = SkBitmapCache::Alloc(&desc, self.imageInfo(), &mut pmap) else {
                    return false;
                };
                let success = { self.fSharedGenerator.scoped().getPixels_pixmap(&pmap) };
                if !success && !self.readPixelsProxy(&pmap) {
                    return false;
                }
                SkBitmapCache::Add(rec, bitmap);
                self.notifyAddedToRasterCache();
            } else {
                if !bitmap.tryAllocPixels(self.imageInfo(), 0) {
                    return false;
                }
                let success = {
                    self.fSharedGenerator
                        .scoped()
                        .getPixels_pixmap(bitmap.pixmap())
                };
                if !success && !self.readPixelsProxy(bitmap.pixmap()) {
                    return false;
                }
                bitmap.setImmutable();
            }
            check(bitmap);
            true
        }
    }
    // Source: src/image/SkImage_Lazy.h:89-89
    fn readPixelsProxy(&self, _pixmap: &SkPixmap) -> bool {
        false
    }
    // Source: src/image/SkImage_Lazy.cpp:158-160
    pub fn generator(&self) -> Arc<SharedGenerator> {
        self.fSharedGenerator.clone()
    }
    // Source: src/image/SkImage_Lazy.cpp:162-165
    pub fn onIsProtected(&self) -> bool {
        self.fSharedGenerator.scoped().isProtected()
    }
    // Source: src/image/SkImage_Lazy.cpp:167-179
    unsafe fn onReadPixels(
        &self,
        info: &SkImageInfo,
        pixels: *mut u8,
        rb: usize,
        x: i32,
        y: i32,
        hint: CachingHint,
    ) -> Result<bool, crate::RasterError> {
        let mut bm = SkBitmap::default();
        if self.getROPixels(&mut bm, hint) {
            unsafe { bm.readPixels_info(info, pixels, rb, x, y) }
        } else {
            Ok(false)
        }
    }
    // Source: src/image/SkImage_Lazy.cpp:181-188
    pub fn onRefEncoded(&self) -> Option<Arc<SkData>> {
        if self.fSharedGenerator.uniqueID() == self.uniqueID() {
            return unsafe { self.fSharedGenerator.scoped().refEncodedData() };
        }
        None
    }
    // Source: src/image/SkImage_Lazy.cpp:190-193
    pub fn isValid(&self, recorder: Option<&dyn SkRecorder>) -> bool {
        self.fSharedGenerator.scoped().isValid(recorder)
    }
    // Source: src/image/SkImage_Lazy.cpp:195-205
    fn onMakeSubset(
        &self,
        subset: &SkIRect,
        props: RequiredProperties,
    ) -> Result<Option<Arc<dyn SkImage>>, crate::RasterError> {
        let image: Arc<dyn SkImage> = self.self_owner.upgrade().expect("native live image owner");
        let Some(nonLazyImg) = (unsafe { image.makeRasterImage_default() })? else {
            return Ok(None);
        };
        nonLazyImg.makeSubset(None, subset, props)
    }
    // Source: src/image/SkImage_Lazy.cpp:207-215
    unsafe fn onMakeSurface<'r>(
        &self,
        recorder: Option<&'r dyn SkRecorder>,
        info: &SkImageInfo,
    ) -> Option<crate::include::core::SkRefCnt::sk_sp<dyn super::SkSurface::SkSurface<'r> + 'r>>
    {
        let recorder = match recorder {
            Some(r) => r.cpuRecorder().expect("native CPU recorder"),
            None => crate::src::core::SkCPURecorder::Recorder::TODO(),
        };
        recorder.makeBitmapSurfaceRowBytes(info, 0, None)
    }
    // Source: src/image/SkImage_Lazy.cpp:217-234
    pub(super) fn converted(
        &self,
        targetColorType: SkColorType,
        targetColorSpace: Option<Arc<SkColorSpace>>,
    ) -> Option<Arc<dyn SkImage>> {
        let mut cached = self
            .fOnMakeColorTypeAndSpaceResult
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        if let Some(image) = cached.as_ref() {
            if targetColorType == image.colorType()
                && SkColorSpace::Equals(targetColorSpace.as_deref(), image.colorSpace())
            {
                return Some(image.clone());
            }
        }
        let validator = Validator::new(
            Some(self.fSharedGenerator.clone()),
            Some(targetColorType),
            targetColorSpace,
        );
        let result = if validator.fSharedGenerator.is_some() {
            Some(Self::new(validator) as Arc<dyn SkImage>)
        } else {
            None
        };
        if result.is_some() {
            *cached = result.clone();
        }
        result
    }
    // Source: src/image/SkImage_Lazy.cpp:236-252
    unsafe fn onReinterpretColorSpace(&self, newCS: Arc<SkColorSpace>) -> Option<Arc<dyn SkImage>> {
        let mut bitmap = SkBitmap::default();
        if bitmap.tryAllocPixels(&self.imageInfo().makeColorSpace(Some(newCS)), 0) {
            let mut pixmap = bitmap.pixmap().clone();
            pixmap.setColorSpace(self.refColorSpace());
            if unsafe { self.fSharedGenerator.scoped().getPixels_pixmap(&pixmap) } {
                bitmap.setImmutable();
                return unsafe { super::SkImage_RasterFactories::RasterFromBitmap(&bitmap) };
            }
        }
        None
    }
    // Source: src/image/SkImage_Lazy.h:81-82
    // Source: src/image/SkImage_Lazy.cpp:254-282
    /// Safety: output plane views must be used only while the returned actual
    /// cached-data client handle remains retained/locked. The generator mutex spans
    /// cache lookup, query, allocation, decoding and insertion, exactly as native.
    pub unsafe fn getPlanes(
        &self,
        supported: &crate::src::core::SkYUVAPixmaps::SupportedDataTypes,
        mut pixmaps: Option<&mut crate::src::core::SkYUVAPixmaps::SkYUVAPixmaps>,
    ) -> Option<crate::src::core::SkCachedData::SkCachedData<()>> {
        use crate::src::core::{
            SkResourceCache,
            SkYUVAPixmaps::{SkYUVAPixmapInfo, SkYUVAPixmaps},
            SkYUVPlanesCache::SkYUVPlanesCache,
        };
        let mut generator = self.fSharedGenerator.scoped();
        let data = SkYUVPlanesCache::FindAndRef(generator.uniqueID(), pixmaps.as_deref_mut(), None);
        if let Some(data) = data {
            let out = pixmaps
                .as_deref()
                .expect("native cache success requires output planes");
            crate::SkASSERT!(@source b"src/image/SkImage_Lazy.cpp\0".as_ptr(),262; @condition b"yuvaPixmaps->isValid()\0".as_ptr(); out.isValid());
            crate::SkASSERT!(@source b"src/image/SkImage_Lazy.cpp\0".as_ptr(),263; @condition b"yuvaPixmaps->yuvaInfo().dimensions() == this->dimensions()\0".as_ptr(); out.yuvaInfo().dimensions()==self.dimensions());
            return Some(data);
        }
        let mut info = SkYUVAPixmapInfo::default();
        if !generator.queryYUVAInfo(supported, Some(&mut info))
            || info.yuvaInfo().dimensions() != self.dimensions()
        {
            return None;
        }
        let data = SkResourceCache::NewCachedData(info.computeTotalBytes(None))
            .expect("native cached allocation backing precondition");
        let temp = SkYUVAPixmaps::FromExternalMemory(&info, data.writable_data());
        crate::SkASSERT!(@source b"src/image/SkImage_Lazy.cpp\0".as_ptr(),274; @condition b"tempPixmaps.isValid()\0".as_ptr(); temp.isValid());
        if !unsafe { generator.getYUVAPlanes(&temp) } {
            return None;
        }
        let out = pixmaps
            .as_deref_mut()
            .expect("native decoded planes require nonnull output");
        *out = temp.clone();
        SkYUVPlanesCache::Add(self.uniqueID(), &data, out, None);
        Some(data)
    }
    // Source: src/image/SkImage_Lazy.cpp:284-286
    pub fn addUniqueIDListener(
        &self,
        listener: Option<Arc<dyn SkIDChangeListener::SkIDChangeListener>>,
    ) {
        self.fUniqueIDListeners.add(listener);
    }
}
// Source: src/image/SkImage_Lazy.cpp:291-296
pub fn DeferredFromGenerator(
    generator: Option<Box<dyn SkImageGenerator>>,
) -> Option<Arc<dyn SkImage>> {
    let validator = Validator::new(SharedGenerator::Make(generator), None, None);
    if validator.fSharedGenerator.is_some() {
        Some(SkImage_Lazy::new(validator))
    } else {
        None
    }
}
// Typed native inheritance forwarding: all methods reach the same retained lazy
// owner, generator and native base state. No raster surrogate is constructed.
impl ImageCore for SkImage_Lazy {
    fn state(&self) -> &SkImageState {
        &self.base.image
    }
    fn asImageBase(&self) -> &dyn SkImage_Base {
        self
    }
    fn isTextureBacked(&self) -> bool {
        self.isTextureBackedCPU()
    }
    fn textureSize(&self) -> usize {
        0
    }
    fn isValid(&self, recorder: Option<&dyn SkRecorder>) -> bool {
        Self::isValid(self, recorder)
    }
}
impl ImageVirtuals for SkImage_Lazy {
    fn makeSubset(
        self: Arc<Self>,
        recorder: Option<&dyn SkRecorder>,
        subset: &SkIRect,
        props: RequiredProperties,
    ) -> Result<Option<Arc<dyn SkImage>>, crate::RasterError> {
        super::SkImage_Base::make_subset(&*self, recorder, subset, props)
    }
    fn isLazyGenerated(&self) -> bool {
        self.isLazyGeneratedCPU()
    }
    fn makeColorSpace(
        self: Arc<Self>,
        recorder: Option<&dyn SkRecorder>,
        target: Option<Arc<SkColorSpace>>,
        props: RequiredProperties,
    ) -> Result<Option<Arc<dyn SkImage>>, crate::RasterError> {
        super::SkImage_Base::make_color_space(self, recorder, target, props)
    }
    fn makeColorTypeAndColorSpace(
        self: Arc<Self>,
        _recorder: Option<&dyn SkRecorder>,
        target: SkColorType,
        cs: Option<Arc<SkColorSpace>>,
        _props: RequiredProperties,
    ) -> Result<Option<Arc<dyn SkImage>>, crate::RasterError> {
        Ok(self.converted(target, cs))
    }
}
impl SkImage_Base for SkImage_Lazy {
    fn base_state(&self) -> &SkImageBaseState {
        &self.base
    }
    unsafe fn onReadPixelsCPU(
        &self,
        info: &SkImageInfo,
        pixels: *mut u8,
        rb: usize,
        x: i32,
        y: i32,
        hint: CachingHint,
    ) -> Result<bool, crate::RasterError> {
        unsafe { self.onReadPixels(info, pixels, rb, x, y, hint) }
    }
    // Source: src/image/SkImage_Lazy.h:59-63
    fn onHasMipmaps(&self) -> bool {
        false
    }
    fn onIsProtected(&self) -> bool {
        Self::onIsProtected(self)
    }
    fn getROPixelsCPU(&self, dst: &mut SkBitmap, hint: CachingHint) -> bool {
        self.getROPixels(dst, hint)
    }
    fn onMakeSubsetCPU(
        &self,
        _recorder: Option<&dyn SkRecorder>,
        subset: &SkIRect,
        props: RequiredProperties,
    ) -> Result<Option<Arc<dyn SkImage>>, crate::RasterError> {
        self.onMakeSubset(subset, props)
    }
    fn onRefEncoded(&self) -> Option<Arc<SkData>> {
        Self::onRefEncoded(self)
    }
    unsafe fn onMakeSurfaceCPU<'r>(
        &self,
        recorder: Option<&'r dyn SkRecorder>,
        info: &SkImageInfo,
    ) -> Option<crate::include::core::SkRefCnt::sk_sp<dyn super::SkSurface::SkSurface<'r> + 'r>>
    {
        unsafe { self.onMakeSurface(recorder, info) }
    }
    // Source: src/image/SkImage_Lazy.h:75-75
    fn type_(&self) -> Type {
        Type::kLazy
    }
    unsafe fn onReinterpretColorSpaceCPU(&self, cs: Arc<SkColorSpace>) -> Option<Arc<dyn SkImage>> {
        unsafe { self.onReinterpretColorSpace(cs) }
    }
}
