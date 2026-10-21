use crate::include::core::SkFontMetrics::SkFontMetrics;
use crate::src::core::{
    SkArenaAlloc::SkArenaAlloc,
    SkDescriptor::SkDescriptor,
    SkGlyph::{SkGlyph, SkGlyphDigest, SkGlyphPositionRoundingSpec, SkPackedGlyphID},
    SkScalerContext::SkScalerContextVirtual,
    SkStrikeCache::SkStrikeCache,
    SkStrikeSpec::SkStrikeSpec,
};
use std::{
    cell::UnsafeCell,
    ptr::NonNull,
    sync::{Arc, Weak},
};
// Source: src/core/SkStrike.h:36-42
pub trait SkStrikePinner {
    fn canDelete(&mut self) -> bool;
    fn assertValid(&mut self) {}
}
// Rust Traits instantiation for src/core/SkStrike.h:188-189.
// UNVERIFIED: the original digest-specific load factors apply to the actual
// THashTable. GAT key view preserves native GetKey's by-value packed key.
struct GlyphDigestTraits;
impl crate::src::core::SkTHash::HashTraits<SkGlyphDigest, SkPackedGlyphID> for GlyphDigestTraits {
    type Key<'a>
        = SkPackedGlyphID
    where
        SkGlyphDigest: 'a,
        SkPackedGlyphID: 'a;
    fn GetKey<'a>(value: &'a SkGlyphDigest) -> Self::Key<'a>
    where
        SkPackedGlyphID: 'a,
    {
        SkGlyphDigest::GetKey(*value)
    }
    fn Hash(key: &SkPackedGlyphID) -> u32 {
        SkGlyphDigest::Hash(*key)
    }
    fn loadPolicy() -> Option<(fn(i32, i32) -> bool, fn(i32, i32) -> bool)> {
        Some((SkGlyphDigest::ShouldGrow, SkGlyphDigest::ShouldShrink))
    }
}

// Source: src/core/SkStrike.h:163-166
// UNVERIFIED: native enum replaces the former boolean adaptation at the
// actual internalPrepare boundary; it does not add a second preparation algorithm.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
enum PathDetail {
    kMetricsOnly,
    kMetricsAndPath,
}

// Source: src/core/SkStrike.h:174-212
// Rust owning-field order retains native destruction: pinner, arena, scaler,
// glyph vector, digest table, descriptor/typeface. Shared strike protocol below
// dispatches through the same CPU allocation and native lock.
// Source: src/core/SkStrike.h:44-44
pub struct SkStrike {
    pub(crate) links: core::mem::ManuallyDrop<UnsafeCell<StrikeLinks>>,
    state: core::mem::ManuallyDrop<UnsafeCell<StrikeState>>,
    // Drop protected scaler/arena state before destroying its native lock.
    fStrikeLock: core::mem::ManuallyDrop<crate::include::private::SkMutex::SkMutex>,
    // Weak projection of the same Arc allocation; no auxiliary strong owner/counter.
    self_owner: Weak<SkStrike>,
    fStrikeSpec: core::mem::ManuallyDrop<SkStrikeSpec>,
    pub(crate) fStrikeCache: NonNull<SkStrikeCache>,
    fRoundingSpec: SkGlyphPositionRoundingSpec,
    fFontMetrics: SkFontMetrics,
}
// Source: src/core/SkStrike.h:175-213 (implicit derived destruction)
// Native reverse member order: pinner, arena, scaler, vector, hash, mutex,
// descriptor/spec, then the virtual parent. The existing sole Arc supplies the
// complete-object allocation/refcount lifetime; no parent counter is added.
impl Drop for SkStrike {
    fn drop(&mut self) {
        struct Member<T>(*mut core::mem::ManuallyDrop<T>);
        impl<T> Drop for Member<T> {
            fn drop(&mut self) {
                unsafe {
                    core::mem::ManuallyDrop::drop(&mut *self.0);
                }
            }
        }
        // Tuple fields drop left to right, including cleanup after Rust unwind.
        let members = (
            Member(core::ptr::addr_of_mut!(self.links)),
            Member(core::ptr::addr_of_mut!(self.state)),
            Member(core::ptr::addr_of_mut!(self.fStrikeLock)),
            Member(core::ptr::addr_of_mut!(self.fStrikeSpec)),
        );
        drop(members);
    }
}
pub(crate) struct StrikeLinks {
    pub(crate) fPinner: Option<Box<dyn SkStrikePinner>>,
    pub(crate) fNext: *const SkStrike,
    pub(crate) fPrev: *const SkStrike,
    pub(crate) fMemoryUsed: usize,
    pub(crate) fRemoved: bool,
}
struct StrikeState {
    fAlloc: SkArenaAlloc,
    fScalerContext: Box<dyn SkScalerContextVirtual>,
    fGlyphForIndex: Vec<NonNull<SkGlyph>>,
    fDigestForPackedGlyphID:
        crate::src::core::SkTHash::THashTable<SkGlyphDigest, SkPackedGlyphID, GlyphDigestTraits>,
    fMemoryIncrease: usize,
}
// This is a narrow owner boundary, not a Send/Sync assertion for ScalerContext,
// Glyph, Arena, effect traits, or arbitrary backend values. The sole constructor
// is unsafe: it requires the *entire retained graph* to be transferable between
// threads, and all backend access stays behind state. Metrics fields are immutable.
// links is accessed only under its one originating cache mutex. The unsafe
// creation/operation lease keeps that exact cache object alive and stationary
// for each cache-pointer access. Native default Strike destruction does not
// dereference fStrikeCache; dropping a detached owner alone needs no cache read.
unsafe impl Send for SkStrike {}
unsafe impl Sync for SkStrike {}

impl SkStrike {
    // Source: src/core/SkStrike.h:86-88
    pub fn getFontMetrics(&self) -> &SkFontMetrics {
        &self.fFontMetrics
    }
}
impl SkStrike {
    // Source: src/core/SkStrike.h:110-112
    pub fn getDescriptor(&self) -> &SkDescriptor {
        self.fStrikeSpec.descriptor()
    }
}
impl SkStrike {
    // Source: src/core/SkStrike.h:114-116
    pub fn roundingSpec(&self) -> &SkGlyphPositionRoundingSpec {
        &self.fRoundingSpec
    }
}
impl SkStrike {
    // Source: src/core/SkStrike.h:128-130
    pub fn strikeSpec(&self) -> &SkStrikeSpec {
        &self.fStrikeSpec
    }
}
impl SkStrike {
    // Source: src/core/SkStrike.h:132-136
    pub unsafe fn verifyPinnedStrike(&self) {
        unsafe { self.fStrikeCache.as_ref() }.with_state(|_| unsafe {
            if let Some(pinner) = &mut (*self.links.get()).fPinner {
                pinner.assertValid();
            }
        });
    }
}
impl SkStrike {
    // Source: src/core/SkStrike.h:201-203
    const kMinGlyphCount: usize = 8;
    const kMinGlyphImageSize: usize = 16 * 8;
    const kMinAllocAmount: usize = Self::kMinGlyphImageSize * Self::kMinGlyphCount;
}
// Source: src/core/SkStrike.cpp:33-43
unsafe fn use_or_generate_metrics(
    metrics: Option<&SkFontMetrics>,
    context: &mut dyn SkScalerContextVirtual,
) -> SkFontMetrics {
    if let Some(metrics) = metrics {
        *metrics
    } else {
        // Native backend initializes all fields. Rust references require initialized storage.
        let mut answer: SkFontMetrics = unsafe { std::mem::zeroed() };
        unsafe { context.getFontMetrics(&mut answer) };
        answer
    }
}

impl SkStrike {
    // Source: src/core/SkStrike.cpp:45-59
    /// Safety: every retained backend/effect/pinner resource can transfer between
    /// threads and be invoked serially on any thread; its external backing stays
    /// live until its owner drops. The non-owning cache pointer must identify a live stationary cache during
    /// every later cache-using mutation/unlock and pinner verification. The
    /// native default destructor does not access that cache pointer; retaining
    /// or dropping a detached Arc alone does not extend the cache lease.
    pub(crate) unsafe fn new(
        cache: NonNull<SkStrikeCache>,
        spec: &SkStrikeSpec,
        mut scaler: Box<dyn SkScalerContextVirtual>,
        metrics: Option<&SkFontMetrics>,
        pinner: Option<Box<dyn SkStrikePinner>>,
    ) -> Arc<Self> {
        let font_metrics = unsafe { use_or_generate_metrics(metrics, scaler.as_mut()) };
        let rounding = SkGlyphPositionRoundingSpec::new(
            scaler.base().isSubpixel(),
            scaler.base().getRec().compute_axis_alignment_for_h_text(),
        );
        let strike_spec = spec.clone();
        Arc::new_cyclic(|self_owner| {
            // Keep construction values in ordinary RAII locals until every
            // potentially fallible member construction has completed. Only the
            // complete object's commit transfers them to manual destruction.
            let links = StrikeLinks {
                fPinner: pinner,
                fNext: std::ptr::null(),
                fPrev: std::ptr::null(),
                fMemoryUsed: std::mem::size_of::<Self>(),
                fRemoved: false,
            };
            let state = StrikeState {
                // Reverse native owning-member destruction order is retained
                // inside this existing state group: arena, scaler, vector, hash.
                fAlloc: SkArenaAlloc::new(Self::kMinAllocAmount),
                fScalerContext: scaler,
                fGlyphForIndex: Vec::new(),
                fDigestForPackedGlyphID: crate::src::core::SkTHash::THashTable::default(),
                fMemoryIncrease: 0,
            };
            let lock = crate::include::private::SkMutex::SkMutex::new();
            let weak = self_owner.clone();
            Self {
                fStrikeLock: core::mem::ManuallyDrop::new(lock),
                self_owner: weak,
                links: core::mem::ManuallyDrop::new(UnsafeCell::new(links)),
                state: core::mem::ManuallyDrop::new(UnsafeCell::new(state)),
                fStrikeSpec: core::mem::ManuallyDrop::new(strike_spec),
                fStrikeCache: cache,
                fRoundingSpec: rounding,
                fFontMetrics: font_metrics,
            }
        })
    }
}
// Source: src/core/SkStrike.cpp:61-86
// Native object lock is separate from its protected state. The borrow keeps the
// same complete owner and its semaphore backend alive through release returning.
struct Monitor<'a> {
    strike: &'a SkStrike,
    state: Option<&'a mut StrikeState>,
}
impl Drop for Monitor<'_> {
    fn drop(&mut self) {
        // End the exclusive state reference before releasing the native lock.
        self.state.take();
        unsafe {
            self.strike.unlock();
        }
    }
}
impl SkStrike {
    fn monitor(&self) -> Monitor<'_> {
        self.lock();
        Monitor {
            strike: self,
            state: Some(unsafe { &mut *self.state.get() }),
        }
    }
    // Source: src/core/SkStrike.cpp:77-80
    fn lock(&self) {
        unsafe {
            crate::include::private::SkMutex::SkMutex::acquire_raw(NonNull::from(
                &*self.fStrikeLock,
            ));
            (*self.state.get()).fMemoryIncrease = 0;
        }
    }
    // Source: src/core/SkStrike.cpp:82-86
    /// Safety: this thread owns fStrikeLock and all state references have ended.
    /// The original cache identity remains live for positive-delta accounting.
    unsafe fn unlock(&self) {
        // A positive delta performs the native non-owning cache callback after
        // releasing fStrikeLock; its caller keeps the originating cache live.
        let increase = unsafe { (*self.state.get()).fMemoryIncrease };
        unsafe {
            crate::include::private::SkMutex::SkMutex::release_raw(NonNull::from(
                &*self.fStrikeLock,
            ));
        }
        self.updateMemoryUsage(increase);
    }
}
impl SkStrike {
    // Source: src/core/SkStrike.cpp:164-185
    /// Safety: source glyph and optional pixels retain their backing lease and
    /// do not alias the destination glyph mutated under this strike's monitor.
    /// Returned pointers require this strike to remain alive and no conflicting
    /// mutation while the caller reads their arena-owned contents.
    // Source: src/core/SkStrike.h:69-70
    pub unsafe fn mergeGlyphAndImage(
        &self,
        to_id: SkPackedGlyphID,
        from_glyph: &SkGlyph,
    ) -> NonNull<SkGlyph> {
        let mut monitor = self.monitor();
        let state = monitor.state.as_deref_mut().unwrap();
        if let Some(digest) = state.fDigestForPackedGlyphID.find(&to_id).copied() {
            let glyph = state.fGlyphForIndex[digest.index() as usize];
            if from_glyph.setImageHasBeenCalled() {
                let target = unsafe { &mut *glyph.as_ptr() };
                if target.setImageHasBeenCalled() {
                    debug_assert!(
                        false,
                        "Re-adding image to existing glyph. This should not happen."
                    );
                }
                state.fMemoryIncrease +=
                    unsafe { target.setMetricsAndImage(&mut state.fAlloc, from_glyph) };
            }
            glyph
        } else {
            let glyph = unsafe { NonNull::new_unchecked(state.fAlloc.make(SkGlyph::new(to_id))) };
            state.fMemoryIncrease +=
                unsafe { (*glyph.as_ptr()).setMetricsAndImage(&mut state.fAlloc, from_glyph) }
                    + std::mem::size_of::<SkGlyph>();
            unsafe { state.addGlyphAndDigest(glyph) };
            glyph
        }
    }
    // Source: src/core/SkStrike.cpp:187-197
    /// Safety: glyph belongs to this strike; the input path does not alias its
    /// sidecar. Returned arena pointer retains the same strike/operation lease.
    // Source: src/core/SkStrike.h:73-74
    pub unsafe fn mergePath(
        &self,
        glyph: NonNull<SkGlyph>,
        path: Option<&crate::src::core::SkPath::SkPath>,
        hairline: bool,
        modified: bool,
    ) -> *const crate::src::core::SkPath::SkPath {
        let mut monitor = self.monitor();
        let state = monitor.state.as_deref_mut().unwrap();
        let glyph = unsafe { &mut *glyph.as_ptr() };
        if glyph.setPathHasBeenCalled() {
            debug_assert!(
                false,
                "Re-adding path to existing glyph. This should not happen."
            );
        }
        if unsafe { glyph.setPath(&mut state.fAlloc, path, hairline, modified) } {
            state.fMemoryIncrease += unsafe { glyph.path().unwrap().approximateBytesUsed() };
        }
        unsafe { glyph.path() }.map_or(std::ptr::null(), |path| path as *const _)
    }
    // Source: src/core/SkStrike.cpp:199-210
    /// Safety: glyph belongs to this strike and has an exclusive mutation lease.
    /// The entire retained drawable graph satisfies this strike's original
    /// thread-transfer contract. Returned pointer borrows the strike's arena.
    // Source: src/core/SkStrike.h:77-78
    pub unsafe fn mergeDrawable(
        &self,
        glyph: NonNull<SkGlyph>,
        drawable: crate::include::core::SkRefCnt::sk_sp<
            dyn crate::src::core::SkDrawable::SkDrawable,
        >,
    ) -> Option<NonNull<dyn crate::src::core::SkDrawable::SkDrawable + '_>> {
        let mut monitor = self.monitor();
        let state = monitor.state.as_deref_mut().unwrap();
        let glyph = unsafe { &mut *glyph.as_ptr() };
        if glyph.setDrawableHasBeenCalled() {
            debug_assert!(
                false,
                "Re-adding drawable to existing glyph. This should not happen."
            );
        }
        if unsafe { glyph.setDrawable(&mut state.fAlloc, drawable) } {
            state.fMemoryIncrease += unsafe { glyph.drawable().unwrap().approximateBytesUsed() };
            debug_assert!(state.fMemoryIncrease > 0);
        }
        unsafe { glyph.drawable() }.map(NonNull::from)
    }
}
impl SkStrike {
    // Source: src/core/SkStrike.cpp:212-216
    /// Safety: glyph is a path-prepared member of this strike; native intercept/output leases hold.
    // Source: src/core/SkStrike.h:83-84
    pub unsafe fn findIntercepts(
        &self,
        bounds: &[f32; 2],
        scale: f32,
        x_pos: f32,
        mut glyph: NonNull<SkGlyph>,
        array: Option<&mut [f32]>,
        count: &mut i32,
    ) {
        let mut lock_storage =
            core::pin::pin!(crate::include::private::SkMutex::SkAutoMutexExclusiveStorage::new());
        let _lock = unsafe {
            crate::include::private::SkMutex::SkAutoMutexExclusive::init(
                lock_storage.as_mut(),
                core::pin::Pin::new_unchecked(&*self.fStrikeLock),
            )
        };
        let state = unsafe { &mut *self.state.get() };
        unsafe {
            glyph
                .as_mut()
                .ensureIntercepts(bounds, scale, x_pos, array, count, &mut state.fAlloc);
        }
    }
}
impl SkStrike {
    // Source: src/core/SkStrike.cpp:218-222
    /// Safety: all returned glyph-pointer reads retain this strike and the native
    /// operation lease; no incompatible path/image mutation aliases those reads.
    // Source: src/core/SkStrike.h:90-91
    pub unsafe fn metrics(&self, ids: &[u16], results: &mut [NonNull<SkGlyph>]) {
        assert!(results.len() >= ids.len());
        let mut monitor = self.monitor();
        unsafe {
            monitor.state.as_deref_mut().unwrap().internal_prepare(
                ids,
                PathDetail::kMetricsOnly,
                results,
            );
        }
    }
}
impl SkStrike {
    // Source: src/core/SkStrike.cpp:224-240
    /// Safety: each addressed glyph is an aligned readable u32 and each advance
    /// is an aligned writable f32. Both strided spans stay live through the call.
    /// Output writes must not invalidate future glyph reads or alias strike state.
    /// Strides are in 32-bit elements, as in the native interface.
    // Source: src/core/SkStrike.h:93-98
    pub unsafe fn getWidthsStrided(
        &self,
        count: u32,
        first_glyph: *const u32,
        glyph_stride_32: u32,
        first_advance: *mut f32,
        advance_stride_32: u32,
        scale: f32,
    ) {
        let mut monitor = self.monitor();
        let state = monitor.state.as_deref_mut().unwrap();
        let mut g_ptr = first_glyph;
        let mut a_ptr = first_advance;
        for _ in 0..count {
            let glyph =
                unsafe { state.glyph_metrics(SkPackedGlyphID::from_glyph(g_ptr.read() as u16)) };
            unsafe { a_ptr.write(glyph.as_ref().advanceX() * scale) };
            g_ptr = g_ptr.wrapping_add(glyph_stride_32 as usize);
            a_ptr = a_ptr.wrapping_add(advance_stride_32 as usize);
        }
    }
}
impl SkStrike {
    // Source: src/core/SkStrike.cpp:242-246
    /// Safety: real strike/backing leases remain live and no incompatible glyph mutation aliases reads.
    // Source: src/core/SkStrike.h:100-101
    pub unsafe fn preparePaths(&self, ids: &[u16], results: &mut [NonNull<SkGlyph>]) {
        assert!(results.len() >= ids.len());
        let mut monitor = self.monitor();
        unsafe {
            monitor.state.as_deref_mut().unwrap().internal_prepare(
                ids,
                PathDetail::kMetricsAndPath,
                results,
            );
        }
    }
}
impl SkStrike {
    // Source: src/core/SkStrike.cpp:248-259
    /// Safety: native strike/pixel leases are exclusive. Failed images are not returned as initialized pixels.
    // Source: src/core/SkStrike.h:103-104
    pub unsafe fn prepareImages(
        &self,
        ids: &[SkPackedGlyphID],
        results: &mut [NonNull<SkGlyph>],
    ) -> Result<(), crate::RasterError> {
        assert!(results.len() >= ids.len());
        let mut monitor = self.monitor();
        let state = monitor.state.as_deref_mut().unwrap();
        for (id, result) in ids.iter().zip(results.iter_mut()) {
            let mut glyph = unsafe { state.glyph_metrics(*id) };
            unsafe {
                state.prepare_image(glyph.as_mut())?;
            }
            *result = glyph;
        }
        Ok(())
    }
}
impl SkStrike {
    // Source: src/core/SkStrike.cpp:261-274
    /// Safety: same complete operation leases as metrics; concrete drawables retain their real owners.
    // Source: src/core/SkStrike.h:106-107
    pub unsafe fn prepareDrawables(&self, ids: &[u16], results: &mut [NonNull<SkGlyph>]) {
        assert!(results.len() >= ids.len());
        {
            let mut monitor = self.monitor();
            let state = monitor.state.as_deref_mut().unwrap();
            for (id, result) in ids.iter().zip(results.iter_mut()) {
                let mut glyph = unsafe { state.glyph_metrics(SkPackedGlyphID::from_glyph(*id)) };
                unsafe {
                    state.prepare_drawable(glyph.as_mut());
                }
                *result = glyph;
            }
        }
    }
}
// Source: src/core/SkStrike.h:138-138
// Source: src/core/SkStrike.cpp:295-308
#[cfg(all(target_vendor = "apple", target_arch = "aarch64"))]
impl SkStrike {
    pub fn dump(&self) {
        let mut lock_storage =
            core::pin::pin!(crate::include::private::SkMutex::SkAutoMutexExclusiveStorage::new());
        let _lock = unsafe {
            crate::include::private::SkMutex::SkAutoMutexExclusive::init(
                lock_storage.as_mut(),
                core::pin::Pin::new_unchecked(&*self.fStrikeLock),
            )
        };
        let state = unsafe { &*self.state.get() };
        let face = unsafe { &*state.fScalerContext.base().getTypeface() };
        let rec = state.fScalerContext.base().getRec();
        // Existing Typeface name API uses UTF-8 String; materialize its native
        // NUL-terminated view for the unchanged printf protocol, no owner replacement.
        let mut name = String::new();
        face.getFamilyName(&mut name);
        let name = crate::src::core::SkString::SkString::from_view(name.as_bytes());
        let mut msg = crate::src::core::SkString::SkString::new();
        let style = face.fontStyle();
        let rec_text = rec.dump();
        unsafe {
            msg.printf(
                b"cache typeface:%x %25s:(%d,%d,%d)\n %s glyphs:%3d\0".as_ptr(),
                face.uniqueID(),
                name.c_str(),
                style.weight(),
                style.width(),
                style.slant().0 as i32,
                rec_text.c_str(),
                state.fDigestForPackedGlyphID.count(),
            );
        }
        unsafe {
            crate::SkDebugf!(@source b"src/core/SkStrike.cpp\0".as_ptr(),307; b"%s\n\0".as_ptr(),msg.c_str());
        }
    }
}

impl StrikeState {
    // Source: src/core/SkStrike.cpp:337-344
    // Source: src/core/SkStrike.cpp:346-364
    // CPU measurement's only requested action is native kDirectMask. Factoring
    // this exact case does not publish the still-pending GPU/CPU-image dispatch.
    unsafe fn glyph_metrics(&mut self, id: SkPackedGlyphID) -> NonNull<SkGlyph> {
        if self.fDigestForPackedGlyphID.find(&id).is_none() {
            let glyph = unsafe { self.fScalerContext.makeGlyph(id, &mut self.fAlloc) };
            let ptr = unsafe { NonNull::new_unchecked(self.fAlloc.make(glyph)) };
            self.fMemoryIncrease += std::mem::size_of::<SkGlyph>();
            unsafe { self.addGlyphAndDigest(ptr) };
        }
        let digest = self
            .fDigestForPackedGlyphID
            .find_mut(&id)
            .unwrap_or_else(|| unreachable!());
        digest.setDirectMaskAction();
        self.fGlyphForIndex[digest.index() as usize]
    }
}
impl StrikeState {
    // Source: src/core/SkStrike.cpp:366-372
    /// Safety: glyph is the actual live arena allocation owned by this strike.
    unsafe fn addGlyphAndDigest(&mut self, glyph: NonNull<SkGlyph>) -> &mut SkGlyphDigest {
        let index = self.fGlyphForIndex.len();
        let digest = SkGlyphDigest::new(index, unsafe { glyph.as_ref() });
        let newDigest = self
            .fDigestForPackedGlyphID
            .set(digest)
            .expect("native digest table insertion has capacity");
        self.fGlyphForIndex.push(glyph);
        newDigest
    }
}
impl StrikeState {
    // Source: src/core/SkStrike.cpp:374-379
    unsafe fn prepare_image(&mut self, glyph: &mut SkGlyph) -> Result<bool, crate::RasterError> {
        // Account once at the exact native allocation checkpoint. The arena owns
        // reserved bytes on success, RasterError and backend panic alike. Monitor
        // publishes this delta after releasing the strike mutex on every exit.
        let memory_increase = &mut self.fMemoryIncrease;
        glyph.setImageFromScalerWithAllocation(
            &mut self.fAlloc,
            self.fScalerContext.as_mut(),
            |bytes| {
                *memory_increase = memory_increase.wrapping_add(bytes);
            },
        )?;
        Ok(!glyph.image().is_null())
    }
}
impl StrikeState {
    // Source: src/core/SkStrike.cpp:381-386
    unsafe fn prepare_path(&mut self, glyph: &mut SkGlyph) -> bool {
        if unsafe { glyph.setPathFromScaler(&mut self.fAlloc, self.fScalerContext.as_mut()) } {
            self.fMemoryIncrease += unsafe { glyph.path().unwrap().approximateBytesUsed() };
        }
        unsafe { glyph.path().is_some() }
    }
}
impl StrikeState {
    // Source: src/core/SkStrike.cpp:388-395
    unsafe fn prepare_drawable(&mut self, glyph: &mut SkGlyph) -> bool {
        if unsafe { glyph.setDrawableFromScaler(&mut self.fAlloc, self.fScalerContext.as_mut()) } {
            let increase = unsafe { glyph.drawable().unwrap().approximateBytesUsed() };
            crate::SkASSERT!(@source b"src/core/SkStrike.cpp\0".as_ptr(),391; @condition b"increase > 0\0".as_ptr(); increase > 0);
            self.fMemoryIncrease += increase;
        }
        unsafe { glyph.drawable().is_some() }
    }
}
impl StrikeState {
    // Source: src/core/SkStrike.cpp:397-415
    /// Native caller holds the strike monitor for the whole import operation.
    unsafe fn mergeGlyphFromBuffer(
        &mut self,
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> Option<NonNull<SkGlyph>> {
        crate::SkASSERT!(@source b"src/core/SkStrike.cpp\0".as_ptr(),398; @condition b"buffer.isValid()\0".as_ptr(); buffer.isValid());
        let prototype = SkGlyph::MakeFromBuffer(buffer);
        if !buffer.validate(prototype.is_some()) {
            return None;
        }
        let prototype = prototype.unwrap();
        if let Some(digest) = self.fDigestForPackedGlyphID.find(&prototype.getPackedID()) {
            return Some(self.fGlyphForIndex[digest.index() as usize]);
        }
        let glyph = unsafe { NonNull::new_unchecked(self.fAlloc.make(prototype)) };
        self.fMemoryIncrease += std::mem::size_of::<SkGlyph>();
        unsafe { self.addGlyphAndDigest(glyph) };
        Some(glyph)
    }
    // Source: src/core/SkStrike.cpp:417-425
    unsafe fn mergeGlyphAndImageFromBuffer(
        &mut self,
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> bool {
        crate::SkASSERT!(@source b"src/core/SkStrike.cpp\0".as_ptr(),418; @condition b"buffer.isValid()\0".as_ptr(); buffer.isValid());
        let glyph = unsafe { self.mergeGlyphFromBuffer(buffer) };
        if !buffer.validate(glyph.is_some()) {
            return false;
        }
        self.fMemoryIncrease +=
            unsafe { (*glyph.unwrap().as_ptr()).addImageFromBuffer(buffer, &mut self.fAlloc) };
        buffer.isValid()
    }
    // Source: src/core/SkStrike.cpp:427-435
    unsafe fn mergeGlyphAndPathFromBuffer(
        &mut self,
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> bool {
        crate::SkASSERT!(@source b"src/core/SkStrike.cpp\0".as_ptr(),428; @condition b"buffer.isValid()\0".as_ptr(); buffer.isValid());
        let glyph = unsafe { self.mergeGlyphFromBuffer(buffer) };
        if !buffer.validate(glyph.is_some()) {
            return false;
        }
        self.fMemoryIncrease +=
            unsafe { (*glyph.unwrap().as_ptr()).addPathFromBuffer(buffer, &mut self.fAlloc) };
        buffer.isValid()
    }
}
impl StrikeState {
    // Source: src/core/SkStrike.cpp:447-459
    unsafe fn internal_prepare(
        &mut self,
        ids: &[u16],
        path_detail: PathDetail,
        results: &mut [NonNull<SkGlyph>],
    ) {
        for (id, result) in ids.iter().zip(results.iter_mut()) {
            let mut glyph = unsafe { self.glyph_metrics(SkPackedGlyphID::from_glyph(*id)) };
            if path_detail == PathDetail::kMetricsAndPath {
                unsafe {
                    self.prepare_path(glyph.as_mut());
                }
            }
            *result = glyph;
        }
    }
}
impl SkStrike {
    // Source: src/core/SkStrike.cpp:461-471
    fn updateMemoryUsage(&self, increase: usize) {
        if increase > 0 {
            unsafe { self.fStrikeCache.as_ref() }.with_state(|cache| unsafe {
                let links = &mut *self.links.get();
                links.fMemoryUsed += increase;
                if !links.fRemoved {
                    cache.fTotalMemoryUsed += increase;
                }
            });
        }
    }
}

impl SkStrike {
    // Rust lifetime/alias adaptation, not an additional native algorithm region:
    // keep the native strike monitor while a CPU consumer reads immutable metrics.
    // This avoids overlapping Rust &SkGlyph with native sidecar mutation under another monitor.
    pub(crate) unsafe fn withMetrics<R>(
        &self,
        ids: &[u16],
        results: &mut [NonNull<SkGlyph>],
        consume: impl FnOnce(&[NonNull<SkGlyph>]) -> R,
    ) -> R {
        assert!(results.len() >= ids.len());
        let mut monitor = self.monitor();
        unsafe {
            monitor.state.as_deref_mut().unwrap().internal_prepare(
                ids,
                PathDetail::kMetricsOnly,
                results,
            );
        }
        consume(&results[..ids.len()])
    }
}

// Source: src/core/SkStrike.cpp:61-86
// A single borrowed monitor owns the native lock/unlock pair. Its existing Drop
// unlocks before updating the originating cache, including error exits.
pub struct SkStrikeGlyphRunGuard<'a> {
    monitor: Monitor<'a>,
}
impl SkStrike {
    // Source: src/core/SkStrike.h:52-54
    /// Safety: the retained strike graph and glyph backing leases remain valid;
    /// Preparation and digest lookup require this guard. The native painter
    /// retains prepared glyph pointers after unlock: keep the actual strike
    /// alive, access only the prepared payload, and exclude concurrent mutation
    /// of any fields referenced through those pointers.
    pub unsafe fn lockForGlyphRun(&self) -> SkStrikeGlyphRunGuard<'_> {
        SkStrikeGlyphRunGuard {
            monitor: self.monitor(),
        }
    }
}
impl SkStrikeGlyphRunGuard<'_> {
    // Source: src/core/SkStrike.cpp:337-339
    /// Safety: digest was obtained from this same guard/strike; its pointer must
    /// not outlive its owning strike or alias a later mutating preparation
    /// operation. Releasing this guard does not release the strike's arena.
    pub unsafe fn glyph(&self, digest: SkGlyphDigest) -> NonNull<SkGlyph> {
        self.monitor.state.as_deref().unwrap().fGlyphForIndex[digest.index() as usize]
    }
    // Source: src/core/SkStrike.cpp:346-371
    pub unsafe fn digestFor(
        &mut self,
        action: crate::src::core::SkGlyph::ActionType,
        id: SkPackedGlyphID,
    ) -> Result<SkGlyphDigest, crate::RasterError> {
        unsafe {
            self.monitor
                .state
                .as_deref_mut()
                .unwrap()
                .digest_for(action, id)
        }
    }
}

impl SkStrikeGlyphRunGuard<'_> {
    // Source: src/core/SkStrike.h:55
    /// Safety: glyph belongs to this strike's arena and has no other active
    /// reference during preparation. The guard retains the native strike lock.
    pub unsafe fn prepareForImage(
        &mut self,
        mut glyph: NonNull<SkGlyph>,
    ) -> Result<bool, crate::RasterError> {
        unsafe {
            self.monitor
                .state
                .as_deref_mut()
                .unwrap()
                .prepare_image(glyph.as_mut())
        }
    }
    // Source: src/core/SkStrike.h:56
    /// Safety: same arena ownership and exclusive glyph access as prepareForImage.
    pub unsafe fn prepareForPath(&mut self, mut glyph: NonNull<SkGlyph>) -> bool {
        unsafe {
            self.monitor
                .state
                .as_deref_mut()
                .unwrap()
                .prepare_path(glyph.as_mut())
        }
    }
    // Source: src/core/SkStrike.h:57
    /// Safety: same arena ownership and exclusive glyph access as prepareForImage.
    pub unsafe fn prepareForDrawable(&mut self, mut glyph: NonNull<SkGlyph>) -> bool {
        unsafe {
            self.monitor
                .state
                .as_deref_mut()
                .unwrap()
                .prepare_drawable(glyph.as_mut())
        }
    }
}
// Source: src/core/SkStrike.cpp:310-335
#[cfg(all(target_vendor = "apple", target_arch = "aarch64"))]
impl SkStrike {
    pub fn dumpMemoryStatistics(
        &self,
        dump: &mut dyn crate::include::core::SkTraceMemoryDump::SkTraceMemoryDump,
    ) {
        let mut lock_storage =
            core::pin::pin!(crate::include::private::SkMutex::SkAutoMutexExclusiveStorage::new());
        let _lock = unsafe {
            crate::include::private::SkMutex::SkAutoMutexExclusive::init(
                lock_storage.as_mut(),
                core::pin::Pin::new_unchecked(&*self.fStrikeLock),
            )
        };
        let state = unsafe { &*self.state.get() };
        let face = unsafe { &*state.fScalerContext.base().getTypeface() };
        let rec = state.fScalerContext.base().getRec();
        let mut name = String::new();
        face.getFamilyName(&mut name);
        let mut name = crate::src::core::SkString::SkString::from_view(name.as_bytes());
        unsafe extern "C" {
            fn isalnum(c: core::ffi::c_int) -> core::ffi::c_int;
        }
        for i in 0..name.size() {
            unsafe {
                let p = name.data_mut().add(i);
                if isalnum(*p as core::ffi::c_char as i32) == 0 {
                    *p = b'_';
                }
            }
        }
        let dumpName = unsafe {
            crate::src::core::SkString::SkStringPrintf(
                b"%s/%s_%u/%p\0".as_ptr(),
                (
                    crate::src::core::SkStrikeCache::kGlyphCacheDumpName
                        .as_ptr()
                        .cast::<u8>(),
                    (name.c_str(), (rec.fTypefaceID, (self as *const Self, ()))),
                ),
            )
        };
        let dumpName = unsafe { core::ffi::CStr::from_ptr(dumpName.c_str().cast()) };
        dump.dumpNumericValue(dumpName, c"size", c"bytes", unsafe {
            (*self.links.get()).fMemoryUsed
        } as u64);
        dump.dumpNumericValue(
            dumpName,
            c"glyph_count",
            c"objects",
            state.fDigestForPackedGlyphID.count() as u64,
        );
        dump.setMemoryBacking(dumpName, c"malloc", None);
    }
}
impl StrikeState {
    // Source: src/core/SkStrike.cpp:437-445
    unsafe fn mergeGlyphAndDrawableFromBuffer(
        &mut self,
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> Result<bool, crate::RasterError> {
        crate::SkASSERT!(@source b"src/core/SkStrike.cpp\0".as_ptr(),438; @condition b"buffer.isValid()\0".as_ptr(); buffer.isValid());
        let glyph = unsafe { self.mergeGlyphFromBuffer(buffer) };
        if !buffer.validate(glyph.is_some()) {
            return Ok(false);
        }
        self.fMemoryIncrease += unsafe {
            (*glyph.expect("native validated glyph").as_ptr())
                .addDrawableFromBuffer(buffer, &mut self.fAlloc)
        }?;
        Ok(buffer.isValid())
    }
}
impl SkStrike {
    // Source: src/core/SkStrike.cpp:87-117
    /// Safety: source glyph payloads retain the same initialized backing/arena lease
    /// through the synchronous writer, including any drawable snapshot callbacks.
    pub unsafe fn FlattenGlyphsByType(
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
        images: &mut [SkGlyph],
        paths: &mut [SkGlyph],
        drawables: &mut [SkGlyph],
    ) -> Result<(), crate::RasterError> {
        if !(images.len() <= i32::MAX as usize
            && paths.len() <= i32::MAX as usize
            && drawables.len() <= i32::MAX as usize)
        {
            std::process::abort();
        }
        buffer.writeInt(images.len() as i32);
        for glyph in images {
            debug_assert!(crate::src::core::SkMask::SkMask::IsValidFormat(
                glyph.maskFormat() as u8
            ));
            glyph.flattenMetrics(buffer);
            unsafe { glyph.flattenImage(buffer) };
        }
        buffer.writeInt(paths.len() as i32);
        for glyph in paths {
            debug_assert!(crate::src::core::SkMask::SkMask::IsValidFormat(
                glyph.maskFormat() as u8
            ));
            glyph.flattenMetrics(buffer);
            unsafe { glyph.flattenPath(buffer) };
        }
        buffer.writeInt(drawables.len() as i32);
        for glyph in drawables {
            debug_assert!(crate::src::core::SkMask::SkMask::IsValidFormat(
                glyph.maskFormat() as u8
            ));
            glyph.flattenMetrics(buffer);
            unsafe { glyph.flattenDrawable(buffer) }?;
        }
        Ok(())
    }
    // Source: src/core/SkStrike.cpp:119-162
    /// Safety: imported backend/arena payloads retain the native strike resource
    /// lease and callbacks do not unwind while partially publishing a glyph.
    pub unsafe fn mergeFromBuffer(
        &self,
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> Result<bool, crate::RasterError> {
        let imagesCount = buffer.readInt();
        if imagesCount == 0 && !buffer.isValid() {
            return Ok(false);
        }
        {
            let mut monitor = self.monitor();
            let state = monitor.state.as_deref_mut().unwrap();
            for _ in 0..imagesCount {
                if !unsafe { state.mergeGlyphAndImageFromBuffer(buffer) } {
                    return Ok(false);
                }
            }
        }
        let pathsCount = buffer.readInt();
        if pathsCount == 0 && !buffer.isValid() {
            return Ok(false);
        }
        {
            let mut monitor = self.monitor();
            let state = monitor.state.as_deref_mut().unwrap();
            for _ in 0..pathsCount {
                if !unsafe { state.mergeGlyphAndPathFromBuffer(buffer) } {
                    return Ok(false);
                }
            }
        }
        let drawablesCount = buffer.readInt();
        if drawablesCount == 0 && !buffer.isValid() {
            return Ok(false);
        }
        {
            let mut monitor = self.monitor();
            let state = monitor.state.as_deref_mut().unwrap();
            for _ in 0..drawablesCount {
                if !unsafe { state.mergeGlyphAndDrawableFromBuffer(buffer) }? {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }
}
impl SkStrike {
    // Source: src/core/SkStrike.cpp:276-283
    /// Safety: every input union has an active glyph-ID member. The caller takes
    /// responsibility for destroying the resulting path members exactly once, as
    /// native PathOpSubmitter does. The actual strike/scaler resource lease holds.
    // Source: src/core/SkStrike.h:123-123
    pub unsafe fn glyphIDsToPaths(
        &self,
        idsOrPaths: &mut [crate::src::text::StrikeForGPU::IDOrPath],
    ) {
        let mut monitor = self.monitor();
        let state = monitor.state.as_deref_mut().unwrap();
        for idOrPath in idsOrPaths {
            let id = unsafe { idOrPath.fGlyphID };
            // This is the existing native glyph(packedID) direct-mask digest path; it
            // does not substitute an image/path producer or copy the arena glyph.
            let mut glyph = unsafe { state.glyph_metrics(SkPackedGlyphID::from_glyph(id)) };
            unsafe { state.prepare_path(glyph.as_mut()) };
            let path = unsafe { glyph.as_ref().path() }
                .expect("native path conversion requires a prepared outline");
            unsafe {
                core::ptr::addr_of_mut!(idOrPath.fPath)
                    .write(core::mem::ManuallyDrop::new(path.clone()));
            }
        }
    }
    // Source: src/core/SkStrike.cpp:285-293
    /// Safety: every input union has an active glyph-ID member. The returned raw
    /// drawable member borrows the actual strike/scaler backing; retain the native
    /// resource lease and exclude conflicting mutation for the consumer's use.
    // Source: src/core/SkStrike.h:126-126
    pub unsafe fn glyphIDsToDrawables(
        &self,
        idsOrDrawables: &mut [crate::src::text::StrikeForGPU::IDOrDrawable],
    ) {
        let mut monitor = self.monitor();
        let state = monitor.state.as_deref_mut().unwrap();
        for idOrDrawable in idsOrDrawables {
            let id = unsafe { idOrDrawable.fGlyphID };
            let mut glyph = unsafe { state.glyph_metrics(SkPackedGlyphID::from_glyph(id)) };
            unsafe { state.prepare_drawable(glyph.as_mut()) };
            let drawable = unsafe { glyph.as_ref().drawable() };
            debug_assert!(drawable.is_some());
            idOrDrawable.fDrawable = drawable.map(NonNull::from);
        }
    }
}

impl StrikeState {
    // Source: src/core/SkStrike.cpp:346-371
    // Same existing digest algorithm; shared by guarded CPU and native virtual dispatch.
    unsafe fn digest_for(
        &mut self,
        action: crate::src::core::SkGlyph::ActionType,
        id: SkPackedGlyphID,
    ) -> Result<SkGlyphDigest, crate::RasterError> {
        use crate::src::core::SkGlyph::{ActionType, GlyphAction};
        let state = self;
        if let Some(digest) = state.fDigestForPackedGlyphID.find(&id) {
            if digest.actionFor(action) != GlyphAction::kUnset {
                return Ok(*digest);
            }
        }
        let mut glyph = if let Some(digest) = state.fDigestForPackedGlyphID.find(&id) {
            state.fGlyphForIndex[digest.index() as usize]
        } else {
            let value = unsafe { state.fScalerContext.makeGlyph(id, &mut state.fAlloc) };
            let glyph = unsafe { NonNull::new_unchecked(state.fAlloc.make(value)) };
            state.fMemoryIncrease += std::mem::size_of::<SkGlyph>();
            let digest = SkGlyphDigest::new(state.fGlyphForIndex.len(), unsafe { glyph.as_ref() });
            state
                .fDigestForPackedGlyphID
                .set(digest)
                .expect("native digest table insertion has capacity");
            state.fGlyphForIndex.push(glyph);
            glyph
        };
        // Copy packed metadata out while the native producer mutates its glyph;
        // write back only after success. Errors leave the action unset for retry.
        let mut digest = *state.fDigestForPackedGlyphID.find(&id).unwrap();
        unsafe {
            digest.setActionForCPU(action, glyph.as_mut(), |kind, value| match kind {
                ActionType::kDirectMaskCPU => state.prepare_image(value),
                ActionType::kPath => Ok(state.prepare_path(value)),
                ActionType::kDrawable => Ok(state.prepare_drawable(value)),
                _ => unreachable!("non-producing native action dispatch"),
            })?;
        }
        // Native updates the existing slot in place. Do not call set here:
        // set performs the native growth check even when replacing an entry.
        *state
            .fDigestForPackedGlyphID
            .find_mut(&id)
            .expect("native previously inserted digest remains present") = digest;
        Ok(digest)
    }
}

// Source: src/core/SkStrike.h:53-57,110-120
// Shared virtual calls preserve the same native lock lease and allocation.
impl crate::src::text::StrikeForGPU::StrikeForGPU for SkStrike {
    fn lock(&self) {
        SkStrike::lock(self);
    }
    unsafe fn unlock(&self) {
        unsafe {
            SkStrike::unlock(self);
        }
    }
    unsafe fn digestFor(
        &self,
        action: crate::src::core::SkGlyph::ActionType,
        id: SkPackedGlyphID,
    ) -> Result<SkGlyphDigest, crate::RasterError> {
        unsafe { (&mut *self.state.get()).digest_for(action, id) }
    }
    unsafe fn prepareForImage(&self, glyph: *mut SkGlyph) -> Result<bool, crate::RasterError> {
        unsafe { (&mut *self.state.get()).prepare_image(&mut *glyph) }
    }
    unsafe fn prepareForPath(&self, glyph: *mut SkGlyph) -> Result<bool, crate::RasterError> {
        Ok(unsafe { (&mut *self.state.get()).prepare_path(&mut *glyph) })
    }
    unsafe fn prepareForDrawable(&self, glyph: *mut SkGlyph) -> Result<bool, crate::RasterError> {
        Ok(unsafe { (&mut *self.state.get()).prepare_drawable(&mut *glyph) })
    }
    fn getDescriptor(&self) -> &SkDescriptor {
        SkStrike::getDescriptor(self)
    }
    fn roundingSpec(&self) -> &SkGlyphPositionRoundingSpec {
        SkStrike::roundingSpec(self)
    }
    fn strikePromise(&self) -> crate::src::text::StrikeForGPU::SkStrikePromise {
        // Upgrade the original allocation, precisely one native sk_ref_sp retain.
        // A value extracted with Arc::try_unwrap no longer has a shared owner;
        // reject that state rather than reconstructing an Arc from a borrowed ptr.
        let owner = self
            .self_owner
            .upgrade()
            .expect("strike promise requires its original shared owner");
        crate::src::text::StrikeForGPU::SkStrikePromise::from_strike(Some(owner))
    }
}
