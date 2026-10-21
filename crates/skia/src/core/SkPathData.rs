// Native immutable storage: one allocation contains the ref-counted metadata and aligned
// point/conic/verb tail. Only the private uninitialized owner can fill and publish it.
use crate::include::core::SkPathTypes::*;
use crate::include::private::SkPathRef::*;
use crate::src::core::SkIDChangeListener::{List, SkIDChangeListener};
use crate::src::core::SkPathEnums::*;
use crate::src::core::SkPathPriv::SkPathPriv;
use crate::src::core::SkPathRaw::SkPathRaw;
use crate::src::core::SkPoint::SkPoint;
use crate::src::core::SkRect::SkRect;
use std::sync::atomic::{AtomicI32, AtomicU8, Ordering};
use std::sync::{Arc, OnceLock};

// Rust ownership implementation for native SkSpan/sk_sp; no borrowed view outlives its owner.
#[repr(C)]
pub(crate) struct SkPathSpan<T> {
    ptr: *mut T,
    len: usize,
}
impl<T> SkPathSpan<T> {
    fn empty() -> Self {
        Self {
            ptr: core::ptr::null_mut(),
            len: 0,
        }
    }
}
impl<T> core::ops::Deref for SkPathSpan<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        if self.len == 0 {
            &[]
        } else {
            unsafe { core::slice::from_raw_parts(self.ptr, self.len) }
        }
    }
}
impl<T> core::ops::DerefMut for SkPathSpan<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        if self.len == 0 {
            &mut []
        } else {
            unsafe { core::slice::from_raw_parts_mut(self.ptr, self.len) }
        }
    }
}

#[repr(transparent)]
pub struct SkPathDataRef {
    ptr: core::ptr::NonNull<SkPathData>,
}
// Geometry is immutable after publication; only the atomic convexity cache and locked listener list mutate.
unsafe impl Send for SkPathData {}
unsafe impl Sync for SkPathData {}
unsafe impl Send for SkPathDataRef {}
unsafe impl Sync for SkPathDataRef {}
impl core::ops::Deref for SkPathDataRef {
    type Target = SkPathData;
    fn deref(&self) -> &SkPathData {
        unsafe { self.ptr.as_ref() }
    }
}
// Source: include/core/SkRefCnt.h:176-176
impl Clone for SkPathDataRef {
    #[inline]
    fn clone(&self) -> Self {
        self.fRefCnt.fetch_add(1, Ordering::Relaxed);
        Self { ptr: self.ptr }
    }
}
impl PartialEq for SkPathDataRef {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}
impl SkPathDataRef {
    #[inline]
    pub fn as_ptr(this: &Self) -> *const SkPathData {
        this.ptr.as_ptr()
    }
    #[inline]
    pub fn ptr_eq(a: &Self, b: &Self) -> bool {
        a.ptr == b.ptr
    }
    // Source: include/core/SkRefCnt.h:175-175
    #[inline]
    pub(crate) fn get_mut(this: &mut Self) -> Option<&mut SkPathData> {
        if this.fRefCnt.load(Ordering::Acquire) == 1 {
            Some(unsafe { this.ptr.as_mut() })
        } else {
            None
        }
    }
}

// Source: include/core/SkRefCnt.h:177-184
impl Drop for SkPathDataRef {
    #[inline]
    fn drop(&mut self) {
        if self.fRefCnt.fetch_add(-1, Ordering::AcqRel) == 1 {
            // Native SkNVRefCnt restores 1 for its destructor assertion in debug builds.
            #[cfg(sk_debug)]
            self.fRefCnt.store(1, Ordering::Relaxed);
            unsafe {
                destroy_data(self.ptr);
            }
        }
    }
}
#[inline]
unsafe fn destroy_data(ptr: core::ptr::NonNull<SkPathData>) {
    let data = unsafe { ptr.as_ref() };
    let size = core::mem::size_of::<SkPathData>()
        + data.fPoints.len * core::mem::size_of::<SkPoint>()
        + data.fConics.len * core::mem::size_of::<f32>()
        + data.fVerbs.len * core::mem::size_of::<SkPathVerb>();
    let layout = unsafe {
        std::alloc::Layout::from_size_align_unchecked(size, core::mem::align_of::<SkPathData>())
    };
    // Drop notifies listeners before the backing bytes are released. The tail contains only POD values.
    unsafe {
        core::ptr::drop_in_place(ptr.as_ptr());
        // Source: src/core/SkPathData.cpp:190-192
        std::alloc::dealloc(ptr.as_ptr().cast::<u8>(), layout);
    }
}

pub(crate) struct SkPathDataUninit {
    ptr: core::ptr::NonNull<SkPathData>,
}
impl SkPathDataUninit {
    fn copy_points(&mut self, points: &[SkPoint]) {
        let data = unsafe { self.ptr.as_ref() };
        assert_eq!(data.fPoints.len, points.len());
        if !points.is_empty() {
            unsafe {
                core::ptr::copy_nonoverlapping(points.as_ptr(), data.fPoints.ptr, points.len());
            }
        }
    }
    fn copy_conics(&mut self, conics: &[f32]) {
        let data = unsafe { self.ptr.as_ref() };
        assert_eq!(data.fConics.len, conics.len());
        if !conics.is_empty() {
            unsafe {
                core::ptr::copy_nonoverlapping(conics.as_ptr(), data.fConics.ptr, conics.len());
            }
        }
    }
    fn copy_verbs(&mut self, verbs: &[SkPathVerb]) {
        let data = unsafe { self.ptr.as_ref() };
        assert_eq!(data.fVerbs.len, verbs.len());
        if !verbs.is_empty() {
            unsafe {
                core::ptr::copy_nonoverlapping(verbs.as_ptr(), data.fVerbs.ptr, verbs.len());
            }
        }
    }
    fn write_verb(&mut self, index: usize, verb: SkPathVerb) {
        let data = unsafe { self.ptr.as_ref() };
        assert!(index < data.fVerbs.len);
        unsafe {
            data.fVerbs.ptr.add(index).write(verb);
        }
    }
    // Safety: all points, conics and verbs have been initialized by the owning native factory.
    unsafe fn publish(self) -> SkPathDataRef {
        let this = core::mem::ManuallyDrop::new(self);
        SkPathDataRef { ptr: this.ptr }
    }
}
impl Drop for SkPathDataUninit {
    fn drop(&mut self) {
        unsafe {
            destroy_data(self.ptr);
        }
    }
}

// Source: src/core/SkPathData.cpp:27-30
impl SkPathData {
    fn PeekEmptySingleton() -> &'static SkPathDataRef {
        static EMPTY: OnceLock<SkPathDataRef> = OnceLock::new();
        EMPTY.get_or_init(|| {
            Self::MakeNoCheck(&[], &[], &[], None, None).expect("empty pathdata is valid")
        })
    }
}

// Source: src/core/SkPathData.cpp:32-45
fn next_pathdata_unique_id() -> u32 {
    const HIGH_BITS: u32 = 2;
    static NEXT_ID: AtomicI32 = AtomicI32::new(1);
    loop {
        let mut id = NEXT_ID.fetch_add(1, Ordering::Relaxed) as u32;
        id <<= HIGH_BITS;
        id >>= HIGH_BITS;
        if id != 0 {
            return id;
        }
    }
}

// Source: src/core/SkPathData.cpp:47-73
struct SkSafeAccumulator {
    total: Option<usize>,
}
impl SkSafeAccumulator {
    fn new(n: usize) -> Self {
        Self { total: Some(n) }
    }
    fn ok(&self) -> bool {
        self.total.is_some()
    }
    fn add(&mut self, n: usize) -> &mut Self {
        self.total = self.total.and_then(|v| v.checked_add(n));
        self
    }
    fn addMul(&mut self, a: usize, b: usize) -> &mut Self {
        self.total = self
            .total
            .and_then(|v| a.checked_mul(b).and_then(|n| v.checked_add(n)));
        self
    }
    fn total(&self) -> Option<usize> {
        self.total
    }
}

// Source: src/core/SkPathData.cpp:75-77
pub const gPtsPerVerb: [u8; 6] = [1, 1, 2, 2, 3, 0];

// Source: src/core/SkPathData.cpp:79-81
fn valid_conic_weight(w: f32) -> bool {
    w >= 0.0 && w.is_finite()
}

// Source: src/core/SkPathData.cpp:83-136
fn valid_path_data(pts: &[SkPoint], vbs: &[SkPathVerb], conics: &[f32]) -> bool {
    if vbs.is_empty() {
        return pts.is_empty() && conics.is_empty();
    }
    if vbs[0] != SkPathVerb::kMove {
        return false;
    }
    let mut prev = SkPathVerb::kMove;
    let mut point_count = 1;
    let mut conic_count = 0;
    for &curr in &vbs[1..] {
        if curr as u32 > SkPathVerb::kLast_Verb as u32 {
            return false;
        }
        if prev == SkPathVerb::kMove && curr == SkPathVerb::kMove {
            return false;
        }
        if prev == SkPathVerb::kClose && curr != SkPathVerb::kMove {
            return false;
        }
        point_count += gPtsPerVerb[curr as usize] as usize;
        conic_count += (curr == SkPathVerb::kConic) as usize;
        prev = curr;
    }
    if pts.len() != point_count || conics.len() != conic_count {
        return false;
    }
    for &w in conics {
        if !valid_conic_weight(w) {
            return false;
        }
    }
    true
}

// Source: src/core/SkPathData.cpp:141-143
fn report_pathdata_make_failure(_reason: &str) {}

// Source: src/core/SkPathData.cpp:147-182
impl SkPathData {
    unsafe fn initialize_at(ptr: *mut Self, npts: usize, nvbs: usize, ncns: usize) {
        // The allocation is stable before any spans are formed. The tail is still uninitialized.
        let data = unsafe { ptr.add(1).cast::<u8>() };
        let (points, conics, verbs) = if nvbs == 0 {
            debug_assert!(npts == 0 && ncns == 0);
            (
                SkPathSpan::empty(),
                SkPathSpan::empty(),
                SkPathSpan::empty(),
            )
        } else {
            let point_ptr = data.cast::<SkPoint>();
            let conic_ptr =
                unsafe { data.add(npts * core::mem::size_of::<SkPoint>()) }.cast::<f32>();
            let verb_ptr = unsafe { conic_ptr.add(ncns) }.cast::<SkPathVerb>();
            debug_assert_eq!(point_ptr as usize % core::mem::align_of::<SkPoint>(), 0);
            debug_assert_eq!(conic_ptr as usize % core::mem::align_of::<f32>(), 0);
            (
                SkPathSpan {
                    ptr: point_ptr,
                    len: npts,
                },
                SkPathSpan {
                    ptr: conic_ptr,
                    len: ncns,
                },
                SkPathSpan {
                    ptr: verb_ptr,
                    len: nvbs,
                },
            )
        };
        unsafe {
            ptr.write(Self {
                fRefCnt: AtomicI32::new(1),
                fGenIDChangeListeners: List::default(),
                fPoints: points,
                fConics: conics,
                fVerbs: verbs,
                // Bounds and segment mask are private and finished before publication to a caller.
                fBounds: SkRect::MakeEmpty(),
                fUniqueID: next_pathdata_unique_id(),
                fConvexity: AtomicU8::new(SkPathConvexity::kUnknown as u8),
                fSegmentMask: 0,
                fType: SkPathIsAType::kGeneral,
                fIsA: SkPathIsAData {
                    fStartIndex: 0,
                    fDirection: SkPathDirection::kCW,
                },
            });
        }
    }
}

// Source: src/core/SkPathData.cpp:184-188
impl Drop for SkPathData {
    fn drop(&mut self) {
        #[cfg(sk_debug)]
        {
            self.fUniqueID = 0xeeeeeeee;
        }
        // List field destruction performs the original listener notifications while buffers still live.
    }
}

// Source: src/core/SkPathData.cpp:194-200
impl SkPathData {
    pub fn addGenIDChangeListener(&self, listener: Option<Arc<dyn SkIDChangeListener>>) {
        if !core::ptr::eq(self, SkPathDataRef::as_ptr(Self::PeekEmptySingleton())) {
            self.fGenIDChangeListeners.add(listener);
        }
    }
}

// Source: src/core/SkPathData.cpp:206-223
impl SkPathData {
    pub(crate) fn Alloc(npts: usize, nvbs: usize, ncns: usize) -> Option<SkPathDataUninit> {
        // Enforce the native constructor precondition at this safe Rust allocation boundary.
        assert!((npts == 0 && nvbs == 0 && ncns == 0) || (npts != 0 && nvbs != 0));
        let mut accum = SkSafeAccumulator::new(core::mem::size_of::<Self>());
        accum
            .addMul(npts, core::mem::size_of::<SkPoint>())
            .addMul(ncns, core::mem::size_of::<f32>())
            .addMul(nvbs, core::mem::size_of::<SkPathVerb>());
        if let Some(size) = accum.total() {
            let layout = std::alloc::Layout::from_size_align(size, core::mem::align_of::<Self>())
                .unwrap_or_else(|_| {
                    std::alloc::handle_alloc_error(std::alloc::Layout::new::<Self>())
                });
            let allocation = unsafe { std::alloc::alloc(layout) };
            let Some(ptr) = core::ptr::NonNull::new(allocation.cast::<Self>()) else {
                std::alloc::handle_alloc_error(layout);
            };
            unsafe {
                Self::initialize_at(ptr.as_ptr(), npts, nvbs, ncns);
            }
            return Some(SkPathDataUninit { ptr });
        }
        None
    }
}

// Source: src/core/SkPathData.cpp:225-254
impl SkPathData {
    fn finishInit(&mut self, bounds: Option<SkRect>, segment_mask: Option<u8>) -> bool {
        debug_assert!(valid_path_data(&self.fPoints, &self.fVerbs, &self.fConics));
        if self.fPoints.is_empty() {
            self.fBounds = SkRect::MakeEmpty();
            self.fSegmentMask = 0;
            return true;
        }
        self.fSegmentMask = if let Some(mask) = segment_mask {
            mask
        } else {
            SkPathPriv::ComputeSegmentMask(&self.fVerbs)
        };
        if let Some(bounds) = bounds {
            self.fBounds = bounds.makeSorted();
            debug_assert!(self.fPoints.iter().all(SkPoint::is_finite));
        } else if let Some(r) = SkPathPriv::TrimmedBounds(&self.fPoints, &self.fVerbs) {
            self.fBounds = r;
        } else {
            report_pathdata_make_failure("non-finite bounds");
            return false;
        }
        crate::SkASSERT!(@source b"src/core/SkPathData.cpp\0".as_ptr(),252; @condition b"fBounds.isSorted()\0".as_ptr(); self.fBounds.isSorted());
        true
    }
}

// Source: src/core/SkPathData.cpp:256-291
impl SkPathData {
    pub fn MakeTransform(
        src: &SkPathRaw<'_>,
        mx: &crate::src::core::SkMatrix::SkMatrix,
    ) -> Option<SkPathDataRef> {
        if src.empty() {
            return Some(SkPathData::Empty());
        }
        if mx.hasPerspective() {
            return crate::src::core::SkPathBuilder::SkPathBuilder::new()
                .addRaw(src, crate::src::core::SkPathBuilder::Reserve::kExact)
                .transform(mx)
                .detachData();
        }
        let mut result = Self::Alloc(src.points().len(), src.verbs().len(), src.conics().len())?;
        // Raw destination writes preserve the original uninitialized tail-to-publication lifecycle.
        let dest = unsafe { result.ptr.as_ref().fPoints.ptr };
        unsafe {
            (mx.getMapPtsProc())(mx, dest, src.points().as_ptr(), src.points().len());
        }
        result.copy_conics(src.conics());
        result.copy_verbs(src.verbs());
        let mut transformed = None;
        if mx.rectStaysRect() {
            let mut bounds = SkRect::MakeEmpty();
            mx.mapRect(&mut bounds, &src.bounds());
            if !bounds.isFinite() {
                report_pathdata_make_failure("transform created non-finite bounds");
                return None;
            }
            transformed = Some(bounds);
        }
        let mut result = unsafe { result.publish() };
        if !SkPathDataRef::get_mut(&mut result)
            .unwrap()
            .finishInit(transformed, Some(src.fSegmentMask))
        {
            return None;
        }
        result.setConvexity(SkPathPriv::TransformConvexity(
            mx,
            src.fPoints,
            src.fConvexity,
        ));
        Some(result)
    }
}

// Source: src/core/SkPathData.cpp:293-314
impl SkPathData {
    pub fn makeTransform(
        &self,
        mx: &crate::src::core::SkMatrix::SkMatrix,
    ) -> Option<SkPathDataRef> {
        if mx.isIdentity() {
            self.fRefCnt.fetch_add(1, Ordering::Relaxed);
            return Some(SkPathDataRef {
                ptr: core::ptr::NonNull::from(self),
            });
        }
        let ft = SkPathFillType::kWinding;
        if let Some(mut result) = Self::MakeTransform(&self.raw(ft, SkResolveConvexity::kNo), mx) {
            if (self.fType == SkPathIsAType::kOval || self.fType == SkPathIsAType::kRRect)
                && mx.rectStaysRect()
                && SkPathPriv::IsAxisAligned(&self.fPoints)
            {
                let (dir, start) = SkPathPriv::TransformDirAndStart(
                    mx,
                    self.fType == SkPathIsAType::kRRect,
                    self.fIsA.fDirection,
                    self.fIsA.fStartIndex as u32,
                );
                SkPathDataRef::get_mut(&mut result)
                    .unwrap()
                    .setupIsA(self.fType, dir, start);
            }
            return Some(result);
        }
        None
    }
}

// Source: src/core/SkPathData.cpp:316-318
impl SkPathData {
    pub fn makeOffset(&self, v: SkPoint) -> Option<SkPathDataRef> {
        self.makeTransform(&crate::src::core::SkMatrix::SkMatrix::Translate(v.fX, v.fY))
    }
}

// Source: src/core/SkPathData.cpp:320-328
impl PartialEq for SkPathData {
    fn eq(&self, b: &Self) -> bool {
        if core::ptr::eq(self, b) {
            return true;
        }
        self.points() == b.points() && self.conics() == b.conics() && self.verbs() == b.verbs()
    }

    // Source: src/core/SkPathData.h:112-114
    #[inline]
    fn ne(&self, b: &Self) -> bool {
        !self.eq(b)
    }
}

// Source: src/core/SkPathData.cpp:332-346
impl SkPathData {
    pub(crate) fn MakeNoCheck(
        pts: &[SkPoint],
        vbs: &[SkPathVerb],
        conics: &[f32],
        bounds: Option<SkRect>,
        segment_mask: Option<u8>,
    ) -> Option<SkPathDataRef> {
        crate::SkASSERT!(@source b"src/core/SkPathData.cpp\0".as_ptr(),337; @condition b"valid_path_data(pts, vbs, conics)\0".as_ptr(); valid_path_data(pts, vbs, conics));
        let mut allocation = Self::Alloc(pts.len(), vbs.len(), conics.len())?;
        allocation.copy_points(pts);
        allocation.copy_conics(conics);
        allocation.copy_verbs(vbs);
        // Every tail element is initialized before this creates a safe SkPathData reference.
        let mut path = unsafe { allocation.publish() };
        if SkPathDataRef::get_mut(&mut path)
            .expect("new pathdata is exclusive")
            .finishInit(bounds, segment_mask)
        {
            Some(path)
        } else {
            None
        }
    }
}

// Source: src/core/SkPathData.cpp:348-350
impl SkPathData {
    pub(crate) fn MakeNoCheckRaw(raw: &SkPathRaw<'_>) -> Option<SkPathDataRef> {
        Self::MakeNoCheck(
            raw.points(),
            raw.verbs(),
            raw.conics(),
            Some(raw.fBounds),
            Some(raw.fSegmentMask),
        )
    }
}

// Source: src/core/SkPathData.cpp:352-354
impl SkPathData {
    pub fn Empty() -> SkPathDataRef {
        Self::PeekEmptySingleton().clone()
    }
}

// Source: src/core/SkPathData.cpp:356-367
impl SkPathData {
    pub(crate) fn setupIsA(&mut self, typ: SkPathIsAType, dir: SkPathDirection, index: u32) {
        self.setConvexity(SkPathDirection_ToConvexity(dir));
        debug_assert!(typ == SkPathIsAType::kOval || typ == SkPathIsAType::kRRect);
        self.fType = typ;
        debug_assert!(
            (typ == SkPathIsAType::kOval && index < 4)
                || (typ == SkPathIsAType::kRRect && index < 8)
        );
        self.fIsA.fDirection = dir;
        self.fIsA.fStartIndex = index as u8;
    }
}

// Source: src/core/SkPathData.cpp:369-375
impl SkPathData {
    pub fn Rect(r: &SkRect, dir: SkPathDirection, index: u32) -> Option<SkPathDataRef> {
        if !r.isFinite() {
            return None;
        }
        let storage = crate::src::core::SkPathRawShapes::Rect::new(r, dir, index);
        let raw = storage.raw();
        Self::MakeNoCheck(
            raw.points(),
            raw.verbs(),
            raw.conics(),
            Some(raw.fBounds),
            Some(raw.fSegmentMask),
        )
    }
}

// Source: src/core/SkPathData.cpp:377-386
impl SkPathData {
    pub fn Oval(r: &SkRect, dir: SkPathDirection, index: u32) -> Option<SkPathDataRef> {
        if !r.isFinite() {
            return None;
        }
        let storage = crate::src::core::SkPathRawShapes::Oval::new(r, dir, index);
        let raw = storage.raw();
        let mut path = Self::MakeNoCheck(
            raw.points(),
            raw.verbs(),
            raw.conics(),
            Some(raw.fBounds),
            Some(raw.fSegmentMask),
        )?;
        SkPathDataRef::get_mut(&mut path)
            .expect("factory ownership")
            .setupIsA(SkPathIsAType::kOval, dir, index);
        Some(path)
    }
}

// Source: src/core/SkPathData.cpp:388-399
impl SkPathData {
    pub fn RRect(
        r: &crate::src::core::SkRRect::SkRRect,
        dir: SkPathDirection,
        index: u32,
    ) -> Option<SkPathDataRef> {
        if !r.isValid() {
            return None;
        }
        let storage = crate::src::core::SkPathRawShapes::RRect::new(r, dir, index);
        let raw = storage.raw();
        if let Some(mut path) = Self::Make(raw.points(), raw.verbs(), raw.conics()) {
            SkPathDataRef::get_mut(&mut path)
                .expect("factory ownership")
                .setupIsA(SkPathIsAType::kRRect, dir, index);
            return Some(path);
        }
        None
    }
}

// Source: src/core/SkPathData.cpp:401-421
impl SkPathData {
    pub fn Polygon(pts: &[SkPoint], is_closed: bool) -> Option<SkPathDataRef> {
        if pts.is_empty() || (pts.len() == 1 && !is_closed) {
            return Some(Self::Empty());
        }
        let nverbs = pts.len() + is_closed as usize;
        let mut allocation = Self::Alloc(pts.len(), nverbs, 0)?;
        allocation.copy_points(pts);
        allocation.write_verb(0, SkPathVerb::kMove);
        for i in 1..pts.len() {
            allocation.write_verb(i, SkPathVerb::kLine);
        }
        if is_closed {
            allocation.write_verb(nverbs - 1, SkPathVerb::kClose);
        }
        let mut path = unsafe { allocation.publish() };
        if SkPathDataRef::get_mut(&mut path)
            .expect("factory ownership")
            .finishInit(None, Some(kLine_SkPathSegmentMask as u8))
        {
            Some(path)
        } else {
            None
        }
    }
}

// Source: src/core/SkPathData.cpp:425-427
impl SkPathData {
    pub(crate) fn getConvexityOrUnknown(&self) -> SkPathConvexity {
        match self.fConvexity.load(Ordering::Relaxed) {
            0 => SkPathConvexity::kConvex_CW,
            1 => SkPathConvexity::kConvex_CCW,
            2 => SkPathConvexity::kConvex_Degenerate,
            3 => SkPathConvexity::kConcave,
            4 => SkPathConvexity::kUnknown,
            _ => unreachable!("invalid convexity cache"),
        }
    }
}

// Source: src/core/SkPathData.cpp:429-436
impl SkPathData {
    pub(crate) fn getResolvedConvexity(&self) -> SkPathConvexity {
        let mut convexity = self.getConvexityOrUnknown();
        if convexity == SkPathConvexity::kUnknown {
            convexity = SkPathPriv::ComputeConvexity(&self.fPoints, &self.fVerbs, &self.fConics);
            self.setConvexity(convexity);
        }
        convexity
    }
}

// Source: src/core/SkPathData.cpp:438-440
impl SkPathData {
    pub(crate) fn setConvexity(&self, cv: SkPathConvexity) {
        self.fConvexity.store(cv as u8, Ordering::Relaxed);
    }
}

// Source: src/core/SkPathData.cpp:442-444
impl SkPathData {
    pub fn isConvex(&self) -> bool {
        SkPathConvexity_IsConvex(self.getResolvedConvexity())
    }
}

// Source: src/core/SkPathData.cpp:446-448
impl SkPathData {
    pub fn computeTightBounds(&self) -> SkRect {
        SkPathPriv::ComputeTightBounds(self.points(), self.verbs(), self.conics())
    }
}

// Source: src/core/SkPathData.cpp:450-461
impl SkPathData {
    pub fn raw(&self, ft: SkPathFillType, rc: SkResolveConvexity) -> SkPathRaw<'_> {
        SkPathRaw {
            fPoints: &self.fPoints,
            fVerbs: &self.fVerbs,
            fConics: &self.fConics,
            fBounds: self.fBounds,
            fFillType: ft,
            fConvexity: if rc == SkResolveConvexity::kYes {
                self.getResolvedConvexity()
            } else {
                self.getConvexityOrUnknown()
            },
            fSegmentMask: self.fSegmentMask,
        }
    }
}

// Source: src/core/SkPathData.cpp:463-471
impl SkPathData {
    pub fn asLine(&self) -> Option<[SkPoint; 2]> {
        if self.fPoints.len() == 2
            && self.fVerbs.len() == 2
            && self.fConics.is_empty()
            && self.fVerbs[1] == SkPathVerb::kLine
        {
            crate::SkASSERT!(@source b"src/core/SkPathData.cpp\0".as_ptr(),467; @condition b"fVerbs[0] == SkPathVerb::kMove\0".as_ptr(); (self.fVerbs[0])==(SkPathVerb::kMove));
            return Some([self.fPoints[0], self.fPoints[1]]);
        }
        None
    }
}

// Source: src/core/SkPathData.cpp:473-483
impl SkPathData {
    pub fn asRect(&self) -> Option<SkPathRectInfo> {
        let rc = SkPathPriv::IsRectContour(
            &self.fPoints,
            &self.fVerbs,
            self.fSegmentMask as u32,
            false,
        )?;
        crate::SkASSERT!(@source b"src/core/SkPathData.cpp\0".as_ptr(),475; @condition b"rc->fRect == fBounds\0".as_ptr(); (rc.fRect)==(self.fBounds));
        Some(SkPathRectInfo {
            fRect: self.fBounds,
            fDirection: rc.fDirection,
            fStartIndex: 0,
        })
    }
}

// Source: src/core/SkPathData.cpp:485-494
impl SkPathData {
    pub fn asOval(&self) -> Option<SkPathOvalInfo> {
        if self.fType == SkPathIsAType::kOval {
            return Some(SkPathOvalInfo {
                fBounds: self.fBounds,
                fDirection: self.fIsA.fDirection,
                fStartIndex: self.fIsA.fStartIndex,
            });
        }
        None
    }
}

// Source: src/core/SkPathData.cpp:496-505
impl SkPathData {
    pub fn asRRect(&self) -> Option<SkPathRRectInfo> {
        if self.fType == SkPathIsAType::kRRect {
            Some(SkPathRRectInfo {
                fRRect: SkPathPriv::DeduceRRectFromContour(
                    &self.fBounds,
                    &self.fPoints,
                    &self.fVerbs,
                ),
                fDirection: self.fIsA.fDirection,
                fStartIndex: self.fIsA.fStartIndex,
            })
        } else {
            None
        }
    }
}

// Source: src/core/SkPathData.cpp:507-509
impl SkPathData {
    pub fn contains(&self, p: SkPoint, ft: SkPathFillType) -> bool {
        SkPathPriv::Contains(&self.raw(ft, SkResolveConvexity::kNo), p)
    }
}

// Source: src/core/SkPathData.cpp:513-523
impl SkPathData {
    pub fn Make(pts: &[SkPoint], vbs: &[SkPathVerb], conics: &[f32]) -> Option<SkPathDataRef> {
        if !valid_path_data(pts, vbs, conics) {
            report_pathdata_make_failure("invalid path data");
            return None;
        }
        Self::MakeNoCheck(pts, vbs, conics, None, None)
    }
}
// Source: src/core/SkPathData.h:102-105
impl SkPathData {
    #[inline]
    pub fn RRectDefault(
        rrect: &crate::src::core::SkRRect::SkRRect,
        dir: SkPathDirection,
    ) -> Option<SkPathDataRef> {
        Self::RRect(rrect, dir, if dir == SkPathDirection::kCW { 6 } else { 7 })
    }
}
// Source: src/core/SkPathData.h:107-109
impl SkPathData {
    #[inline]
    pub fn Line(a: SkPoint, b: SkPoint) -> Option<SkPathDataRef> {
        Self::Polygon(&[a, b], false)
    }
}

// Source: src/core/SkPathData.h:116-116
impl SkPathData {
    #[inline]
    pub fn points(&self) -> &[SkPoint] {
        &self.fPoints
    }
}

// Source: src/core/SkPathData.h:117-117
impl SkPathData {
    #[inline]
    pub fn verbs(&self) -> &[SkPathVerb] {
        &self.fVerbs
    }
}

// Source: src/core/SkPathData.h:118-118
impl SkPathData {
    #[inline]
    pub fn conics(&self) -> &[f32] {
        &self.fConics
    }
}

// Source: src/core/SkPathData.h:119-119
impl SkPathData {
    #[inline]
    pub fn bounds(&self) -> &SkRect {
        &self.fBounds
    }
}

// Source: src/core/SkPathData.h:120-120
impl SkPathData {
    #[inline]
    pub fn segmentMask(&self) -> u8 {
        self.fSegmentMask
    }
}

// Source: src/core/SkPathData.h:123-123
impl SkPathData {
    #[inline]
    pub fn uniqueID(&self) -> u32 {
        self.fUniqueID
    }
}

// Source: src/core/SkPathData.h:130-130
impl SkPathData {
    #[inline]
    pub fn empty(&self) -> bool {
        self.fVerbs.is_empty()
    }
}

// Source: src/core/SkPathData.h:178-178
impl SkPathData {
    #[inline]
    pub fn genIDChangeListenerCount(&self) -> i32 {
        self.fGenIDChangeListeners.count()
    }
}

// Source: src/core/SkPathData.h:187-205
#[repr(C)]
pub struct SkPathData {
    // The original SkNVRefCnt base is an atomic int32 before the path fields.
    fRefCnt: AtomicI32,
    fGenIDChangeListeners: List,
    pub(crate) fPoints: SkPathSpan<SkPoint>,
    pub(crate) fConics: SkPathSpan<f32>,
    pub(crate) fVerbs: SkPathSpan<SkPathVerb>,
    pub(crate) fBounds: SkRect,
    fUniqueID: u32,
    fConvexity: AtomicU8,
    pub(crate) fSegmentMask: u8,
    pub(crate) fType: SkPathIsAType,
    pub(crate) fIsA: SkPathIsAData,
}
