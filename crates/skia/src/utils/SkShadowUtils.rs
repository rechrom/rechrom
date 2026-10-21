// Copyright 2017 Google LLC. BSD-style license.
use crate::src::core::{
    SkColor::*, SkDrawShadowInfo::SkDrawShadowRec, SkMatrix::SkMatrix, SkPath::SkPath,
    SkPoint::SkPoint, SkPoint3::SkPoint3, SkRect::SkRect,
};
// Source: include/utils/SkShadowUtils.h:23-36
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkShadowFlags {
    kNone_ShadowFlag = 0x00,
    kTransparentOccluder_ShadowFlag = 0x01,
    kGeometricOnly_ShadowFlag = 0x02,
    kDirectionalLight_ShadowFlag = 0x04,
    kConcaveBlurOnly_ShadowFlag = 0x08,
    kAll_ShadowFlag = 0x0F,
}
// Source: include/utils/SkShadowUtils.h:38-39
pub struct SkShadowUtils;
// Source: src/utils/SkShadowUtils.cpp:64-66
fn resource_cache_shared_id() -> u64 {
    0x2020776f64616873
}
// Source: src/utils/SkShadowUtils.cpp:69-72
#[derive(Clone, Copy)]
struct AmbientVerticesFactory {
    fOccluderHeight: f32,
    fTransparent: bool,
    fOffset: SkPoint,
}
impl AmbientVerticesFactory {
    // Source: src/utils/SkShadowUtils.cpp:74-80
    fn isCompatible(&self, that: &Self, translate: &mut SkPoint) -> bool {
        if self.fOccluderHeight != that.fOccluderHeight || self.fTransparent != that.fTransparent {
            return false;
        }
        *translate = that.fOffset;
        true
    }
}
impl AmbientVerticesFactory {
    // Source: src/utils/SkShadowUtils.cpp:82-93
    fn makeVertices(
        &self,
        path: &SkPath,
        ctm: &SkMatrix,
        translate: &mut SkPoint,
    ) -> crate::include::core::SkRefCnt::sk_sp<crate::src::core::SkVertices::SkVertices> {
        let zParams = SkPoint3::Make(0.0, 0.0, self.fOccluderHeight);
        let mut noTrans = ctm.clone();
        if !ctm.hasPerspective() {
            noTrans[SkMatrix::kMTransX] = 0.0;
            noTrans[SkMatrix::kMTransY] = 0.0;
        }
        *translate = self.fOffset;
        crate::src::utils::SkShadowTessellator::MakeAmbient(
            path,
            &noTrans,
            &zParams,
            self.fTransparent,
        )
    }
}
// Source: src/utils/SkShadowUtils.cpp:98-110
#[derive(Clone, Copy, PartialEq, Eq)]
enum OccluderType {
    kPointTransparent,
    kPointOpaquePartialUmbra,
    kPointOpaqueNoUmbra,
    kDirectional,
    kDirectionalTransparent,
}
// Source: src/utils/SkShadowUtils.cpp:97-97
// Declaration/overload maps to the existing concrete method body below.
// Source: src/utils/SkShadowUtils.cpp:112-117
#[derive(Clone, Copy)]
struct SpotVerticesFactory {
    fOffset: SkPoint,
    fLocalCenter: SkPoint,
    fOccluderHeight: f32,
    fDevLightPos: SkPoint3,
    fLightRadius: f32,
    fOccluderType: OccluderType,
}
impl SpotVerticesFactory {
    // Source: src/utils/SkShadowUtils.cpp:119-145
    fn isCompatible(&self, that: &Self, translate: &mut SkPoint) -> bool {
        if self.fOccluderHeight != that.fOccluderHeight
            || self.fDevLightPos.fZ != that.fDevLightPos.fZ
            || self.fLightRadius != that.fLightRadius
            || self.fOccluderType != that.fOccluderType
        {
            return false;
        }
        match self.fOccluderType {
            OccluderType::kPointTransparent | OccluderType::kPointOpaqueNoUmbra => {
                *translate = that.fOffset;
                true
            }
            OccluderType::kPointOpaquePartialUmbra => {
                if self.fOffset == that.fOffset {
                    *translate = SkPoint::make(0.0, 0.0);
                    true
                } else {
                    false
                }
            }
            OccluderType::kDirectional | OccluderType::kDirectionalTransparent => {
                *translate = SkPoint::make(
                    that.fOffset.fX - self.fOffset.fX,
                    that.fOffset.fY - self.fOffset.fY,
                );
                true
            }
        }
    }
}
impl SpotVerticesFactory {
    // Source: src/utils/SkShadowUtils.cpp:147-174
    fn makeVertices(
        &self,
        path: &SkPath,
        ctm: &SkMatrix,
        translate: &mut SkPoint,
    ) -> crate::include::core::SkRefCnt::sk_sp<crate::src::core::SkVertices::SkVertices> {
        let transparent = self.fOccluderType == OccluderType::kPointTransparent
            || self.fOccluderType == OccluderType::kDirectionalTransparent;
        let directional = self.fOccluderType == OccluderType::kDirectional
            || self.fOccluderType == OccluderType::kDirectionalTransparent;
        let zParams = SkPoint3::Make(0.0, 0.0, self.fOccluderHeight);
        if directional {
            *translate = SkPoint::make(0.0, 0.0);
            crate::src::utils::SkShadowTessellator::MakeSpot(
                path,
                ctm,
                &zParams,
                &self.fDevLightPos,
                self.fLightRadius,
                transparent,
                true,
            )
        } else if ctm.hasPerspective()
            || self.fOccluderType == OccluderType::kPointOpaquePartialUmbra
        {
            *translate = SkPoint::make(0.0, 0.0);
            crate::src::utils::SkShadowTessellator::MakeSpot(
                path,
                ctm,
                &zParams,
                &self.fDevLightPos,
                self.fLightRadius,
                transparent,
                false,
            )
        } else {
            let mut noTrans = ctm.clone();
            noTrans[SkMatrix::kMTransX] = 0.0;
            noTrans[SkMatrix::kMTransY] = 0.0;
            let devCenter = noTrans.mapPoint(self.fLocalCenter);
            let centerLightPos = SkPoint3::Make(devCenter.fX, devCenter.fY, self.fDevLightPos.fZ);
            *translate = self.fOffset;
            crate::src::utils::SkShadowTessellator::MakeSpot(
                path,
                &noTrans,
                &zParams,
                &centerLightPos,
                self.fLightRadius,
                transparent,
                false,
            )
        }
    }
}
// Source: src/utils/SkShadowUtils.cpp:182-270
// Option entries represent only native slots whose factory was initialized.
// Every unused native slot has NaN height and isCompatible exits before reading
// its other indeterminate fields; skipping None preserves that exact branch.
struct TessellationEntry<F> {
    fFactory: F,
    fVertices: crate::include::core::SkRefCnt::sk_sp<crate::src::core::SkVertices::SkVertices>,
    fMatrix: SkMatrix,
}
struct TessellationSet<F> {
    fEntries: [Option<TessellationEntry<F>>; 4],
    fCount: usize,
    fSize: usize,
    fRandom: crate::src::core::SkRandom::SkRandom,
}
// Source: src/utils/SkShadowUtils.cpp:255-263
// C++ fixed arrays destroy entries from the last element to the first. Rust's
// array drop glue runs forward; release the same existing slots in native order.
// Option is the pre-existing unoccupied-entry representation, not another owner.
impl<F> Drop for TessellationSet<F> {
    fn drop(&mut self) {
        for entry in self.fEntries.iter_mut().rev() {
            drop(entry.take());
        }
    }
}
impl<F: VerticesFactory + Copy> TessellationSet<F> {
    fn new() -> Self {
        Self {
            fEntries: core::array::from_fn(|_| None),
            fCount: 0,
            fSize: 0,
            fRandom: Default::default(),
        }
    }
    fn size(&self) -> usize {
        self.fSize
    }
    fn find(
        &self,
        factory: &F,
        matrix: &SkMatrix,
        translate: &mut SkPoint,
    ) -> crate::include::core::SkRefCnt::sk_sp<crate::src::core::SkVertices::SkVertices> {
        for i in 0..4 {
            if let Some(entry) = &self.fEntries[i] {
                if entry.fFactory.compatible(factory, translate) {
                    let m = &entry.fMatrix;
                    if matrix.hasPerspective() || m.hasPerspective() {
                        if matrix != m {
                            continue;
                        }
                    } else if matrix.getScaleX() != m.getScaleX()
                        || matrix.getSkewX() != m.getSkewX()
                        || matrix.getScaleY() != m.getScaleY()
                        || matrix.getSkewY() != m.getSkewY()
                    {
                        continue;
                    }
                    return entry.fVertices.clone();
                }
            }
        }
        Default::default()
    }
    fn add(
        &mut self,
        path: &SkPath,
        factory: &F,
        matrix: &SkMatrix,
        translate: &mut SkPoint,
    ) -> crate::include::core::SkRefCnt::sk_sp<crate::src::core::SkVertices::SkVertices> {
        let vertices = factory.make(path, matrix, translate);
        if !vertices.is_some() {
            return Default::default();
        }
        let i = if self.fCount < 4 {
            let i = self.fCount;
            self.fCount += 1;
            i
        } else {
            let i = self.fRandom.nextULessThan(4) as usize;
            self.fSize -= self.fEntries[i]
                .as_ref()
                .unwrap()
                .fVertices
                .approximateSize();
            i
        };
        self.fEntries[i] = Some(TessellationEntry {
            fFactory: *factory,
            fVertices: vertices.clone(),
            fMatrix: matrix.clone(),
        });
        self.fSize += vertices.approximateSize();
        vertices
    }
}
// Source: src/utils/SkShadowUtils.cpp:182-270
// The native cache lock covers shared find/size. Mutation requires the actual
// removed-record owner operation, as in native add after Find returns false.
struct CachedTessellations {
    fAmbientSet:
        core::mem::ManuallyDrop<core::cell::UnsafeCell<TessellationSet<AmbientVerticesFactory>>>,
    fSpotSet: core::mem::ManuallyDrop<core::cell::UnsafeCell<TessellationSet<SpotVerticesFactory>>>,
    refcnt: crate::include::core::SkRefCnt::SkRefCnt,
}
impl CachedTessellations {
    fn new() -> Self {
        Self {
            refcnt: crate::include::core::SkRefCnt::SkRefCnt::new(),
            fAmbientSet: core::mem::ManuallyDrop::new(core::cell::UnsafeCell::new(
                TessellationSet::new(),
            )),
            fSpotSet: core::mem::ManuallyDrop::new(core::cell::UnsafeCell::new(
                TessellationSet::new(),
            )),
        }
    }
    unsafe fn size(&self) -> usize {
        unsafe { (&*self.fAmbientSet.get()).size() + (&*self.fSpotSet.get()).size() }
    }
}
unsafe impl Send for CachedTessellations {}
unsafe impl Sync for CachedTessellations {}
unsafe impl crate::include::core::SkRefCnt::SkRefCntObject for CachedTessellations {
    // Source: src/utils/SkShadowUtils.cpp:183-183
    fn internal_dispose_raw_proc(&self) -> crate::include::core::SkRefCnt::SkRawDisposeProc {
        crate::include::core::SkRefCnt::SkRefCntDispose::<Self>
    }
    fn ref_cnt_base(&self) -> &crate::include::core::SkRefCnt::SkRefCntBase {
        self.refcnt.ref_cnt_base()
    }
    fn ref_(&self) {
        self.refcnt.ref_();
    }
}
impl Drop for CachedTessellations {
    fn drop(&mut self) {
        unsafe {
            core::mem::ManuallyDrop::drop(&mut self.fSpotSet);
            core::mem::ManuallyDrop::drop(&mut self.fAmbientSet);
        }
    }
}
// Native overloaded set selection projected as static trait dispatch.
trait CachedFactory: VerticesFactory + Copy {
    unsafe fn set<'a>(owner: &'a CachedTessellations) -> &'a TessellationSet<Self>;
    unsafe fn set_mut<'a>(owner: &'a CachedTessellations) -> &'a mut TessellationSet<Self>;
}
impl CachedFactory for AmbientVerticesFactory {
    unsafe fn set<'a>(o: &'a CachedTessellations) -> &'a TessellationSet<Self> {
        unsafe { &*o.fAmbientSet.get() }
    }
    unsafe fn set_mut<'a>(o: &'a CachedTessellations) -> &'a mut TessellationSet<Self> {
        unsafe { &mut *o.fAmbientSet.get() }
    }
}
impl CachedFactory for SpotVerticesFactory {
    unsafe fn set<'a>(o: &'a CachedTessellations) -> &'a TessellationSet<Self> {
        unsafe { &*o.fSpotSet.get() }
    }
    unsafe fn set_mut<'a>(o: &'a CachedTessellations) -> &'a mut TessellationSet<Self> {
        unsafe { &mut *o.fSpotSet.get() }
    }
}
// Source: src/utils/SkShadowUtils.cpp:278-302
struct CachedTessellationsRec {
    links: crate::src::core::SkResourceCache::RecLinks,
    fKey: core::mem::ManuallyDrop<crate::src::core::SkResourceCache::Key>,
    fTessellations:
        core::mem::ManuallyDrop<crate::include::core::SkRefCnt::sk_sp<CachedTessellations>>,
}
impl CachedTessellationsRec {
    fn new(
        key: &crate::src::core::SkResourceCache::Key,
        t: crate::include::core::SkRefCnt::sk_sp<CachedTessellations>,
    ) -> Self {
        Self {
            links: crate::src::core::SkResourceCache::RecLinks::new(),
            fKey: core::mem::ManuallyDrop::new(key.clone()),
            fTessellations: core::mem::ManuallyDrop::new(t),
        }
    }
    fn refTessellations(&self) -> crate::include::core::SkRefCnt::sk_sp<CachedTessellations> {
        (*self.fTessellations).clone()
    }
    unsafe fn find<F: CachedFactory>(
        &self,
        factory: &F,
        matrix: &SkMatrix,
        translate: &mut SkPoint,
    ) -> crate::include::core::SkRefCnt::sk_sp<crate::src::core::SkVertices::SkVertices> {
        unsafe { F::set(&self.fTessellations).find(factory, matrix, translate) }
    }
}
unsafe impl crate::src::core::SkResourceCache::Rec for CachedTessellationsRec {
    fn links(&self) -> &crate::src::core::SkResourceCache::RecLinks {
        &self.links
    }
    fn links_mut(&mut self) -> &mut crate::src::core::SkResourceCache::RecLinks {
        &mut self.links
    }
    fn getKey(&self) -> &crate::src::core::SkResourceCache::Key {
        &self.fKey
    }
    fn bytesUsed(&self) -> usize {
        unsafe { self.fTessellations.size() }
    }
    fn getCategory(&self) -> *const core::ffi::c_char {
        b"tessellated shadow masks\0".as_ptr().cast()
    }
}
impl Drop for CachedTessellationsRec {
    fn drop(&mut self) {
        unsafe {
            core::mem::ManuallyDrop::drop(&mut self.fTessellations);
            core::mem::ManuallyDrop::drop(&mut self.fKey);
        }
    }
}
// Source: src/utils/SkShadowUtils.cpp:311-325
struct FindContext<'a, F> {
    fViewMatrix: &'a SkMatrix,
    fVertices: crate::include::core::SkRefCnt::sk_sp<crate::src::core::SkVertices::SkVertices>,
    fTranslate: SkPoint,
    fTessellationsOnFailure: crate::include::core::SkRefCnt::sk_sp<CachedTessellations>,
    fFactory: &'a F,
}
impl<'a, F> FindContext<'a, F> {
    fn new(viewMatrix: &'a SkMatrix, factory: &'a F) -> Self {
        Self {
            fViewMatrix: viewMatrix,
            fVertices: Default::default(),
            fTranslate: SkPoint::make(0.0, 0.0),
            fTessellationsOnFailure: Default::default(),
            fFactory: factory,
        }
    }
}
// Source: src/utils/SkShadowUtils.cpp:333-345
unsafe fn FindVisitor<F: CachedFactory>(
    baseRec: &dyn crate::src::core::SkResourceCache::Rec,
    ctx: *mut core::ffi::c_void,
) -> bool {
    unsafe {
        let context = &mut *ctx.cast::<FindContext<'_, F>>();
        let rec = &*(baseRec as *const dyn crate::src::core::SkResourceCache::Rec
            as *const CachedTessellationsRec);
        context.fVertices = rec.find(
            context.fFactory,
            context.fViewMatrix,
            &mut context.fTranslate,
        );
        if context.fVertices.is_some() {
            return true;
        }
        context.fTessellationsOnFailure = rec.refTessellations();
        false
    }
}
// Source: src/utils/SkShadowUtils.cpp:347-347
// Declaration/overload maps to the existing concrete method body below.
// Source: src/utils/SkShadowUtils.cpp:376-377
struct ShadowedPath<'a> {
    fPath: &'a SkPath,
    fViewMatrix: &'a SkMatrix,
}
impl<'a> ShadowedPath<'a> {
    // Source: src/utils/SkShadowUtils.cpp:349-355
    fn new(path: &'a SkPath, viewMatrix: &'a SkMatrix) -> Self {
        Self {
            fPath: path,
            fViewMatrix: viewMatrix,
        }
    }
    // Source: src/utils/SkShadowUtils.cpp:357-358
    fn path(&self) -> &SkPath {
        self.fPath
    }
    fn viewMatrix(&self) -> &SkMatrix {
        self.fViewMatrix
    }
    // Source: src/utils/SkShadowUtils.cpp:370-372
    fn keyBytes(&self) -> i32 {
        -1
    }
    fn writeKey(&self, _key: *mut core::ffi::c_void) {
        crate::SK_ABORT!(@source b"src/utils/SkShadowUtils.cpp\0".as_ptr(), 371; "Should never be called");
    }
    fn isRRect(&self, _rrect: &mut crate::src::core::SkRRect::SkRRect) -> bool {
        false
    }
}
// Source: src/utils/SkShadowUtils.cpp:384-384
static kNamespace: u8 = 0;
// Source: src/utils/SkShadowUtils.cpp:387-409
// Key::clone copies the same immutable packed words as native owned key bytes.
struct ShadowInvalidator {
    fKey: crate::src::core::SkResourceCache::Key,
    state: crate::src::core::SkIDChangeListener::SkIDChangeListenerState,
}
impl ShadowInvalidator {
    // Source: src/utils/SkShadowUtils.cpp:389-392
    // Source: src/utils/SkShadowUtils.cpp:389-392
    fn new(key: &crate::src::core::SkResourceCache::Key) -> Self {
        Self {
            fKey: key.clone(),
            state: Default::default(),
        }
    }
    // Source: src/utils/SkShadowUtils.cpp:395-397
    // Source: src/utils/SkShadowUtils.cpp:395-397
    fn getKey(&self) -> &crate::src::core::SkResourceCache::Key {
        &self.fKey
    }
    // Source: src/utils/SkShadowUtils.cpp:400-402
    // Source: src/utils/SkShadowUtils.cpp:400-402
    unsafe fn FindVisitor(
        _rec: &dyn crate::src::core::SkResourceCache::Rec,
        _ctx: *mut core::ffi::c_void,
    ) -> bool {
        false
    }
}
impl crate::src::core::SkIDChangeListener::SkIDChangeListener for ShadowInvalidator {
    fn state(&self) -> &crate::src::core::SkIDChangeListener::SkIDChangeListenerState {
        &self.state
    }
    // Source: src/utils/SkShadowUtils.cpp:404-406
    // Source: src/utils/SkShadowUtils.cpp:404-406
    fn changed(&self) {
        unsafe {
            crate::src::core::SkResourceCache::Find(
                self.getKey(),
                Self::FindVisitor,
                core::ptr::null_mut(),
            );
        }
    }
}
// Generic dispatch is the language projection of native draw_shadow<FACTORY>.
trait VerticesFactory {
    fn compatible(&self, that: &Self, translate: &mut SkPoint) -> bool;
    fn make(
        &self,
        path: &SkPath,
        ctm: &SkMatrix,
        translate: &mut SkPoint,
    ) -> crate::include::core::SkRefCnt::sk_sp<crate::src::core::SkVertices::SkVertices>;
}
impl VerticesFactory for AmbientVerticesFactory {
    fn compatible(&self, t: &Self, v: &mut SkPoint) -> bool {
        self.isCompatible(t, v)
    }
    fn make(
        &self,
        p: &SkPath,
        m: &SkMatrix,
        t: &mut SkPoint,
    ) -> crate::include::core::SkRefCnt::sk_sp<crate::src::core::SkVertices::SkVertices> {
        self.makeVertices(p, m, t)
    }
}
impl VerticesFactory for SpotVerticesFactory {
    fn compatible(&self, t: &Self, v: &mut SkPoint) -> bool {
        self.isCompatible(t, v)
    }
    fn make(
        &self,
        p: &SkPath,
        m: &SkMatrix,
        t: &mut SkPoint,
    ) -> crate::include::core::SkRefCnt::sk_sp<crate::src::core::SkVertices::SkVertices> {
        self.makeVertices(p, m, t)
    }
}
// Source: src/utils/SkShadowUtils.cpp:415-479
// Key owns the native immutable words. CPU ShadowedPath::keyBytes remains -1;
// no key, listener, cache allocation or cache hit is manufactured for that path.
fn draw_shadow<
    F: CachedFactory,
    D: FnMut(
        &crate::src::core::SkVertices::SkVertices,
        crate::src::core::SkBlendMode::SkBlendMode,
        &crate::src::core::SkPaint::SkPaint,
        f32,
        f32,
        bool,
    ) -> Result<(), crate::RasterError>,
>(
    factory: &F,
    mut drawProc: D,
    path: &ShadowedPath<'_>,
    color: SkColor,
) -> Result<bool, crate::RasterError> {
    let mut context = FindContext::new(path.viewMatrix(), factory);
    let mut key = None;
    let keyDataBytes = path.keyBytes();
    if keyDataBytes >= 0 {
        let mut words = vec![0u32; (keyDataBytes as usize + 3) / 4];
        path.writeKey(words.as_mut_ptr().cast());
        key = Some(crate::src::core::SkResourceCache::Key::new(
            core::ptr::addr_of!(kNamespace).cast(),
            resource_cache_shared_id(),
            &words,
        ));
        unsafe {
            crate::src::core::SkResourceCache::Find(
                key.as_ref().unwrap(),
                FindVisitor::<F>,
                core::ptr::addr_of_mut!(context).cast(),
            );
        }
    }
    let vertices = if context.fVertices.is_some() {
        core::mem::take(&mut context.fVertices)
    } else {
        if let Some(key) = key.as_ref() {
            let tessellations = if context.fTessellationsOnFailure.is_some() {
                core::mem::take(&mut context.fTessellationsOnFailure)
            } else {
                unsafe {
                    crate::include::core::SkRefCnt::sk_sp::from_raw(Box::into_raw(Box::new(
                        CachedTessellations::new(),
                    )))
                }
            };
            let vertices = unsafe {
                F::set_mut(&tessellations).add(
                    path.path(),
                    factory,
                    path.viewMatrix(),
                    &mut context.fTranslate,
                )
            };
            if !vertices.is_some() {
                return Ok(false);
            }
            let rec = Box::new(CachedTessellationsRec::new(key, tessellations));
            let listener: std::sync::Arc<
                dyn crate::src::core::SkIDChangeListener::SkIDChangeListener,
            > = std::sync::Arc::new(ShadowInvalidator::new(key));
            crate::src::core::SkPathPriv::SkPathPriv::AddGenIDChangeListener(
                path.path(),
                Some(listener),
            );
            unsafe { crate::src::core::SkResourceCache::Add(rec, core::ptr::null_mut()) };
            vertices
        } else {
            let vertices = factory.make(path.path(), path.viewMatrix(), &mut context.fTranslate);
            if !vertices.is_some() {
                return Ok(false);
            }
            vertices
        }
    };
    let mut paint = crate::src::core::SkPaint::SkPaint::default();
    paint.setColorFilter(Some(
        crate::src::effects::colorfilters::SkBlendModeColorFilter::BlendColor(
            color,
            crate::src::core::SkBlendMode::SkBlendMode::kModulate,
        )
        .unwrap()
        .makeComposed(Some(crate::src::core::SkColorFilterPriv::MakeGaussian())),
    ));
    drawProc(
        &vertices,
        crate::src::core::SkBlendMode::SkBlendMode::kDst,
        &paint,
        context.fTranslate.fX,
        context.fTranslate.fY,
        path.viewMatrix().hasPerspective(),
    )?;
    Ok(true)
}
// Source: src/utils/SkShadowUtils.cpp:482-484
fn tilted(zPlaneParams: &SkPoint3) -> bool {
    !crate::src::core::SkScalar::SkScalarNearlyZero(
        zPlaneParams.fX,
        crate::src::core::SkScalar::SK_ScalarNearlyZero,
    ) || !crate::src::core::SkScalar::SkScalarNearlyZero(
        zPlaneParams.fY,
        crate::src::core::SkScalar::SK_ScalarNearlyZero,
    )
}
impl SkShadowUtils {
    // Source: include/utils/SkShadowUtils.h:98-99
    // Declaration/overload maps to the existing concrete method body below.
    // Source: src/utils/SkShadowUtils.cpp:487-540
    pub fn ComputeTonalColors(
        inAmbientColor: SkColor,
        inSpotColor: SkColor,
        outAmbientColor: &mut SkColor,
        outSpotColor: &mut SkColor,
    ) {
        *outAmbientColor = SkColorSetARGB(SkColorGetA(inAmbientColor), 0, 0, 0);
        let spotR = SkColorGetR(inSpotColor);
        let spotG = SkColorGetG(inSpotColor);
        let spotB = SkColorGetB(inSpotColor);
        let max = spotR.max(spotG).max(spotB);
        let min = spotR.min(spotG).min(spotB);
        let luminance = 0.5 * (max + min) as f32 / 255.0;
        let origA = SkColorGetA(inSpotColor) as f32 / 255.0;
        let alphaAdjust = (2.6 + (-2.66667 + 1.06667 * origA) * origA) * origA;
        let colorAlpha = (3.544762 + (-4.891428 + 2.3466 * luminance) * luminance) * luminance;
        let colorAlpha =
            crate::include::private::SkTPin::SkTPin(alphaAdjust * colorAlpha, 0.0, 1.0);
        let greyscaleAlpha =
            crate::include::private::SkTPin::SkTPin(origA * (1.0 - 0.4 * luminance), 0.0, 1.0);
        let colorScale = colorAlpha * (1.0 - greyscaleAlpha);
        let tonalAlpha = colorScale + greyscaleAlpha;
        let unPremulScale = colorScale / tonalAlpha;
        *outSpotColor = SkColorSetARGB(
            (tonalAlpha * 255.999) as u32,
            (unPremulScale * spotR as f32) as u32,
            (unPremulScale * spotG as f32) as u32,
            (unPremulScale * spotB as f32) as u32,
        );
    }
}
// Source: src/utils/SkShadowUtils.cpp:542-565
// Option replaces the native bool plus uninitialized POD output. It is not an
// alternate producer; inverse failure is the same native failure branch.
fn fill_shadow_rec(
    _path: &SkPath,
    zPlaneParams: &SkPoint3,
    lightPos: &SkPoint3,
    lightRadius: f32,
    ambientColor: SkColor,
    spotColor: SkColor,
    flags: u32,
    ctm: &SkMatrix,
) -> Option<SkDrawShadowRec> {
    let mut pt = SkPoint::make(lightPos.fX, lightPos.fY);
    if flags & SkShadowFlags::kDirectionalLight_ShadowFlag as u32 == 0 {
        let inverse = ctm.invert()?;
        pt = inverse.mapPoint(pt);
    }
    Some(SkDrawShadowRec {
        fZPlaneParams: *zPlaneParams,
        fLightPos: SkPoint3::Make(pt.fX, pt.fY, lightPos.fZ),
        fLightRadius: lightRadius,
        fAmbientColor: ambientColor,
        fSpotColor: spotColor,
        fFlags: flags,
    })
}
impl SkShadowUtils {
    // Source: include/utils/SkShadowUtils.h:61-63
    // Declaration/overload maps to the existing concrete method body below.
    // Source: src/utils/SkShadowUtils.cpp:568-579
    pub unsafe fn DrawShadow(
        canvas: core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'_>>,
        path: &SkPath,
        zPlaneParams: &SkPoint3,
        lightPos: &SkPoint3,
        lightRadius: f32,
        ambientColor: SkColor,
        spotColor: SkColor,
        flags: u32,
    ) -> Result<(), crate::RasterError> {
        let Some(rec) = fill_shadow_rec(
            path,
            zPlaneParams,
            lightPos,
            lightRadius,
            ambientColor,
            spotColor,
            flags,
            &unsafe { canvas.as_ref() }.getTotalMatrix(),
        ) else {
            return Ok(());
        };
        unsafe {
            crate::src::core::SkCanvas::SkCanvas::private_draw_shadow_rec_raw(canvas, path, &rec)
        }
    }
    // Source: include/utils/SkShadowUtils.h:64-64
    // Rust's named overload binds the actual native default argument. Drawing is
    // still performed by the existing real original-Canvas operation above.
    pub unsafe fn DrawShadowDefault(
        canvas: core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'_>>,
        path: &SkPath,
        zPlaneParams: &SkPoint3,
        lightPos: &SkPoint3,
        lightRadius: f32,
        ambientColor: SkColor,
        spotColor: SkColor,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            Self::DrawShadow(
                canvas,
                path,
                zPlaneParams,
                lightPos,
                lightRadius,
                ambientColor,
                spotColor,
                SkShadowFlags::kNone_ShadowFlag as u32,
            )
        }
    }
    // Source: include/utils/SkShadowUtils.h:86-88
    // Declaration/overload maps to the existing concrete method body below.
    // Source: src/utils/SkShadowUtils.cpp:581-593
    pub fn GetLocalBounds(
        ctm: &SkMatrix,
        path: &SkPath,
        zPlaneParams: &SkPoint3,
        lightPos: &SkPoint3,
        lightRadius: f32,
        flags: u32,
        bounds: &mut SkRect,
    ) -> bool {
        let Some(rec) = fill_shadow_rec(
            path,
            zPlaneParams,
            lightPos,
            lightRadius,
            SK_ColorBLACK,
            SK_ColorBLACK,
            flags,
            ctm,
        ) else {
            return false;
        };
        crate::src::core::SkDrawShadowInfo::SkDrawShadowMetrics::GetLocalBounds(
            path, &rec, ctm, bounds,
        );
        true
    }
}
// Source: src/utils/SkShadowUtils.cpp:597-600
fn validate_rec(rec: &SkDrawShadowRec) -> bool {
    rec.fLightPos.isFinite() && rec.fZPlaneParams.isFinite() && rec.fLightRadius.is_finite()
}
// Operation-scoped projection of SkAutoDeviceTransformRestore for a polymorphic
// device: the raw base pointer has no independently retained mutable reference.
// The caller keeps the original concrete device pinned/exclusive through scope.
struct ShadowDeviceTransform {
    base: core::ptr::NonNull<crate::src::core::SkDevice::SkDevice>,
    previous: crate::src::core::SkM44::SkM44,
}
impl ShadowDeviceTransform {
    unsafe fn new<T: crate::src::core::SkDevice::SkDeviceVirtual + ?Sized>(
        device: &mut T,
        m: &crate::src::core::SkM44::SkM44,
    ) -> Self {
        let previous = crate::src::core::SkM44::SkM44::from(device.base().local_to_device());
        let mut base = core::ptr::NonNull::from(unsafe { device.base_mut() });
        unsafe { base.as_mut() }.set_local_to_device(m);
        Self { base, previous }
    }
}
impl Drop for ShadowDeviceTransform {
    fn drop(&mut self) {
        unsafe { self.base.as_mut() }.set_local_to_device(&self.previous);
    }
}
// Source: src/utils/SkShadowUtils.cpp:602-861
// Real base-virtual body, kept here with its native implementation source. Root
// connects SkDeviceVirtual::drawShadow to this function; no Device target edit.
// Safety: Canvas and this original device have the native exclusive operation
// lease; Canvas stays pinned and does not separately alias a mutable device view.
pub unsafe fn draw_device_shadow<T: crate::src::core::SkDevice::SkDeviceVirtual + ?Sized>(
    device: &mut T,
    canvas: core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'_>>,
    path: &SkPath,
    rec: &SkDrawShadowRec,
) -> Result<(), crate::RasterError> {
    use crate::include::core::{SkBlender::SkBlender, SkBlurTypes::SkBlurStyle};
    use crate::src::core::SkBlendMode::SkBlendMode;
    use crate::src::core::{
        SkM44::SkM44,
        SkPaint::{SkPaint, Style},
    };
    if !validate_rec(rec) {
        return Ok(());
    }
    let viewMatrix = device.base().local_to_device().clone();
    let drawVertsProc = |device: &mut T,
                         vertices: &crate::src::core::SkVertices::SkVertices,
                         mode: SkBlendMode,
                         paint: &SkPaint,
                         tx: f32,
                         ty: f32,
                         hasPerspective: bool|
     -> Result<(), crate::RasterError> {
        if vertices.priv_().vertexCount() != 0 {
            let matrix = if hasPerspective {
                SkM44::default()
            } else {
                *device.base().local_to_device44() * SkM44::Translate(tx, ty, 0.0)
            };
            let _adr = unsafe { ShadowDeviceTransform::new(device, &matrix) };
            unsafe { device.drawVertices(vertices, <dyn SkBlender>::Mode(mode), paint, true) }?;
        }
        Ok(())
    };
    let shadowedPath = ShadowedPath::new(path, &viewMatrix);
    let tiltZPlane = tilted(&rec.fZPlaneParams);
    let transparent = rec.fFlags & SkShadowFlags::kTransparentOccluder_ShadowFlag as u32 != 0;
    let useBlur =
        rec.fFlags & SkShadowFlags::kConcaveBlurOnly_ShadowFlag as u32 != 0 && !path.isConvex();
    let uncached = tiltZPlane || path.isVolatile();
    let directional = rec.fFlags & SkShadowFlags::kDirectionalLight_ShadowFlag as u32 != 0;
    let zPlaneParams = rec.fZPlaneParams;
    let mut devLightPos = rec.fLightPos;
    if !directional {
        let p = viewMatrix.mapPoint(SkPoint::make(devLightPos.fX, devLightPos.fY));
        devLightPos.fX = p.fX;
        devLightPos.fY = p.fY;
    }
    let lightRadius = rec.fLightRadius;
    if SkColorGetA(rec.fAmbientColor) > 0 {
        let _adr = unsafe { ShadowDeviceTransform::new(device, &SkM44::default()) };
        let mut success = false;
        if uncached && !useBlur {
            let vertices = crate::src::utils::SkShadowTessellator::MakeAmbient(
                path,
                &viewMatrix,
                &zPlaneParams,
                transparent,
            );
            if vertices.is_some() {
                let mut paint = SkPaint::default();
                paint.setColorFilter(Some(
                    crate::src::effects::colorfilters::SkBlendModeColorFilter::BlendColor(
                        rec.fAmbientColor,
                        SkBlendMode::kModulate,
                    )
                    .unwrap()
                    .makeComposed(Some(crate::src::core::SkColorFilterPriv::MakeGaussian())),
                ));
                unsafe {
                    device.drawVertices(
                        &vertices,
                        <dyn SkBlender>::Mode(SkBlendMode::kDst),
                        &paint,
                        true,
                    )
                }?;
                success = true;
            }
        }
        if !success && !useBlur {
            let factory = AmbientVerticesFactory {
                fOccluderHeight: zPlaneParams.fZ,
                fTransparent: transparent,
                fOffset: if viewMatrix.hasPerspective() {
                    SkPoint::make(0.0, 0.0)
                } else {
                    SkPoint::make(viewMatrix.getTranslateX(), viewMatrix.getTranslateY())
                },
            };
            success = draw_shadow(
                &factory,
                |v, m, p, x, y, h| drawVertsProc(device, v, m, p, x, y, h),
                &shadowedPath,
                rec.fAmbientColor,
            )?;
        }
        if !success {
            let mut devSpacePath =
                path.makeTransform(&unsafe { canvas.as_ref() }.getLocalToDeviceAs3x3());
            devSpacePath.setIsVolatile(true);
            let devSpaceOutset =
                crate::src::core::SkDrawShadowInfo::AmbientBlurRadius(zPlaneParams.fZ);
            let oneOverA = crate::src::core::SkDrawShadowInfo::AmbientRecipAlpha(zPlaneParams.fZ);
            let blurRadius = 0.5 * devSpaceOutset * oneOverA;
            let strokeWidth = 0.5 * (devSpaceOutset - blurRadius);
            let autoRestore =
                unsafe { crate::src::core::SkCanvas::SkAutoCanvasRestore::new(Some(canvas), true) };
            unsafe {
                core::pin::Pin::new_unchecked(&mut *canvas.as_ptr()).setMatrix44(&SkM44::default())
            };
            let mut paint = SkPaint::default();
            paint.setColor(rec.fAmbientColor);
            paint.setStrokeWidth(strokeWidth);
            paint.setStyle(Style::kStrokeAndFill_Style);
            let sigma = crate::src::core::SkBlurMask::ConvertRadiusToSigma(blurRadius);
            paint.setMaskFilter(crate::src::core::SkMaskFilter::MakeBlur(
                SkBlurStyle::kNormal_SkBlurStyle,
                sigma,
                false,
            ));
            unsafe {
                crate::src::core::SkCanvas::SkCanvas::drawPath_raw(canvas, &devSpacePath, &paint)
            }?;
            unsafe { autoRestore.finish() }?;
        }
    }
    if SkColorGetA(rec.fSpotColor) > 0 {
        let _adr = unsafe { ShadowDeviceTransform::new(device, &SkM44::default()) };
        let mut success = false;
        if uncached && !useBlur {
            let vertices = crate::src::utils::SkShadowTessellator::MakeSpot(
                path,
                &viewMatrix,
                &zPlaneParams,
                &devLightPos,
                lightRadius,
                transparent,
                directional,
            );
            if vertices.is_some() {
                let mut paint = SkPaint::default();
                paint.setColorFilter(Some(
                    crate::src::effects::colorfilters::SkBlendModeColorFilter::BlendColor(
                        rec.fSpotColor,
                        SkBlendMode::kModulate,
                    )
                    .unwrap()
                    .makeComposed(Some(crate::src::core::SkColorFilterPriv::MakeGaussian())),
                ));
                unsafe {
                    device.drawVertices(
                        &vertices,
                        <dyn SkBlender>::Mode(SkBlendMode::kDst),
                        &paint,
                        true,
                    )
                }?;
                success = true;
            }
        }
        if !success && !useBlur {
            let localCenter = SkPoint::make(path.getBounds().centerX(), path.getBounds().centerY());
            let center = viewMatrix.mapPoint(localCenter);
            let (mut radius, mut scale) = (0.0, 0.0);
            let mut offset = SkPoint::default();
            if directional {
                crate::src::core::SkDrawShadowInfo::GetDirectionalParams(
                    zPlaneParams.fZ,
                    devLightPos.fX,
                    devLightPos.fY,
                    devLightPos.fZ,
                    lightRadius,
                    &mut radius,
                    &mut scale,
                    &mut offset,
                );
            } else {
                crate::src::core::SkDrawShadowInfo::GetSpotParams(
                    zPlaneParams.fZ,
                    devLightPos.fX - center.fX,
                    devLightPos.fY - center.fY,
                    devLightPos.fZ,
                    lightRadius,
                    &mut radius,
                    &mut scale,
                    &mut offset,
                );
            }
            let devBounds = viewMatrix.mapRect_value(path.getBounds());
            let occluderType = if transparent
                || offset.fX.abs() > 0.5 * devBounds.width()
                || offset.fY.abs() > 0.5 * devBounds.height()
            {
                if directional {
                    OccluderType::kDirectionalTransparent
                } else {
                    OccluderType::kPointTransparent
                }
            } else if directional {
                OccluderType::kDirectional
            } else if offset.length() * scale + scale < radius {
                OccluderType::kPointOpaqueNoUmbra
            } else if path.isConvex() {
                OccluderType::kPointOpaquePartialUmbra
            } else {
                OccluderType::kPointTransparent
            };
            offset.fX += viewMatrix.getTranslateX();
            offset.fY += viewMatrix.getTranslateY();
            let factory = SpotVerticesFactory {
                fOffset: offset,
                fLocalCenter: localCenter,
                fOccluderHeight: zPlaneParams.fZ,
                fDevLightPos: devLightPos,
                fLightRadius: lightRadius,
                fOccluderType: occluderType,
            };
            success = draw_shadow(
                &factory,
                |v, m, p, x, y, h| drawVertsProc(device, v, m, p, x, y, h),
                &shadowedPath,
                rec.fSpotColor,
            )?;
        }
        if !success {
            let mut shadowMatrix = SkMatrix::default();
            let mut radius = 0.0;
            if !crate::src::core::SkDrawShadowInfo::GetSpotShadowTransform(
                &devLightPos,
                lightRadius,
                &unsafe { canvas.as_ref() }.getLocalToDeviceAs3x3(),
                &zPlaneParams,
                path.getBounds(),
                directional,
                &mut shadowMatrix,
                &mut radius,
            ) {
                return Ok(());
            }
            let autoRestore =
                unsafe { crate::src::core::SkCanvas::SkAutoCanvasRestore::new(Some(canvas), true) };
            unsafe {
                core::pin::Pin::new_unchecked(&mut *canvas.as_ptr()).setMatrix(&shadowMatrix)
            };
            let mut paint = SkPaint::default();
            paint.setColor(rec.fSpotColor);
            let sigma = crate::src::core::SkBlurMask::ConvertRadiusToSigma(radius);
            paint.setMaskFilter(crate::src::core::SkMaskFilter::MakeBlur(
                SkBlurStyle::kNormal_SkBlurStyle,
                sigma,
                false,
            ));
            unsafe { crate::src::core::SkCanvas::SkCanvas::drawPath_raw(canvas, path, &paint) }?;
            unsafe { autoRestore.finish() }?;
        }
    }
    Ok(())
}
