//! CPU virtual interface retaining the canonical intrusive Flattenable base.
use crate::src::core::SkMatrix::SkMatrix;
use crate::src::core::SkPath::SkPath;
use crate::src::core::SkPathBuilder::SkPathBuilder;
use crate::src::core::SkPathEffect::SkPathEffect;
use crate::src::core::SkPoint::SkPoint;
use crate::src::core::SkRect::SkRect;
use crate::src::core::SkStrokeRec::SkStrokeRec;
// Source: src/core/SkPathEffectBase.h:31-66
// Box<[Point]> expresses the native delete[] ownership. The array is dropped before
// the native reverse member order (last, first, path); clip remains uninitialized until set.
pub struct PointData {
    pub fPoints: Option<Box<[SkPoint]>>,
    pub fLast: SkPath,
    pub fFirst: SkPath,
    pub fPath: SkPath,
    pub fClipRect: std::mem::MaybeUninit<SkRect>,
    pub fSize: SkPoint,
    pub fNumPoints: i32,
    pub fFlags: u32,
}
impl PointData {
    pub fn new() -> Self {
        Self {
            fFlags: 0,
            fPoints: None,
            fNumPoints: 0,
            fSize: SkPoint::make(1.0, 1.0),
            fClipRect: std::mem::MaybeUninit::uninit(),
            fPath: SkPath::default(),
            fFirst: SkPath::default(),
            fLast: SkPath::default(),
        }
    }
    pub fn points(&mut self) -> &mut [SkPoint] {
        // Slice bounds preserve Rust validity when public count/owner fields disagree.
        let points = self.fPoints.as_deref_mut().unwrap_or(&mut []);
        &mut points[..self.fNumPoints as usize]
    }
}
pub const kCircles_PointFlag: u32 = 0x01;
pub const kUsePath_PointFlag: u32 = 0x02;
pub const kUseClip_PointFlag: u32 = 0x04;
pub trait SkPathEffectBase: SkPathEffect {
    // Source: src/core/SkPathEffectBase.h:73-75
    fn asPoints(
        &self,
        results: Option<&mut PointData>,
        src: &SkPath,
        rec: &SkStrokeRec,
        mx: &SkMatrix,
        rect: Option<&SkRect>,
    ) -> bool {
        crate::src::core::SkPathEffect::asPoints(self, results, src, rec, mx, rect)
    }
    // Source: src/core/SkPathEffectBase.h:87-89
    fn getFlattenableType(&self) -> crate::src::core::SkFlattenable::Type {
        crate::src::core::SkFlattenable::Type::kSkPathEffect_Type
    }
    // Source: src/core/SkPathEffectBase.h:105-131
    // Nested PointData/DashInfo become module types; virtual receivers become trait methods.
    fn onFilterPath(
        &self,
        dst: &mut SkPathBuilder,
        src: &SkPath,
        rec: &mut SkStrokeRec,
        bounds: Option<&SkRect>,
        ctm: &SkMatrix,
    ) -> bool;
    fn onNeedsCTM(&self) -> bool {
        false
    }
    fn onAsPoints(
        &self,
        _results: Option<&mut PointData>,
        _src: &SkPath,
        _rec: &SkStrokeRec,
        _mx: &SkMatrix,
        _rect: Option<&SkRect>,
    ) -> bool {
        false
    }
    fn asADash(&self) -> Option<DashInfo<'_>> {
        None
    }
    fn computeFastBounds(&self, bounds: Option<&mut SkRect>) -> bool;
}
pub struct DashInfo<'a> {
    pub fIntervals: &'a [f32],
    pub fPhase: f32,
}

// Source: src/core/SkPathEffectBase.h:141-155
// Required Rust projection glue replaces native's known-base static_cast. It does not
// invert the hierarchy: SkPathEffectBase still extends SkPathEffect above.
/// # Safety
/// Both projections must return the SAME complete concrete self object, with its sole
/// embedded reference counter and its concrete dynamic Box-disposal implementation.
/// Projecting a member, wrapper, or unrelated object is forbidden. The concrete payload
/// must satisfy 'static, as required by the canonical owning path-effect object handles.
/// The returned reference is only borrowed for self's borrow; 'static describes payload
/// metadata, never the lifetime of the returned reference or the allocation.
pub unsafe trait SkPathEffectProjection {
    fn project_base(&self) -> &(dyn SkPathEffectBase + 'static);
    fn project_base_mut(&mut self) -> &mut (dyn SkPathEffectBase + 'static);
    /// # Safety
    /// allocation is the ORIGINAL owning raw pointer's allocation data provenance for
    /// this exact complete object. It is not reconstructed from a shared borrow.
    /// The caller retains its existing reference throughout dispatch; no ref is consumed.
    unsafe fn project_base_owner(
        &self,
        allocation: std::ptr::NonNull<()>,
    ) -> std::ptr::NonNull<dyn SkPathEffectBase>;
}
// The compiler constructs metadata from the actual complete T; no vtable reinterpretation.
unsafe impl<T: SkPathEffectBase + 'static> SkPathEffectProjection for T {
    fn project_base(&self) -> &(dyn SkPathEffectBase + 'static) {
        self
    }
    fn project_base_mut(&mut self) -> &mut (dyn SkPathEffectBase + 'static) {
        self
    }
    unsafe fn project_base_owner(
        &self,
        allocation: std::ptr::NonNull<()>,
    ) -> std::ptr::NonNull<dyn SkPathEffectBase> {
        // Recover T from the original complete allocation, then normal raw-pointer unsizing
        // supplies its Base metadata. &self chooses T but does not supply owner provenance.
        let concrete: *mut T = allocation.as_ptr().cast();
        let projected: *mut dyn SkPathEffectBase = concrete;
        unsafe { std::ptr::NonNull::new_unchecked(projected) }
    }
}
pub fn as_PEB_mut(
    effect: Option<&mut dyn SkPathEffect>,
) -> Option<&mut (dyn SkPathEffectBase + 'static)> {
    effect.map(|effect| effect.project_base_mut())
}
pub fn as_PEB(effect: Option<&dyn SkPathEffect>) -> Option<&(dyn SkPathEffectBase + 'static)> {
    effect.map(|effect| effect.project_base())
}
pub fn as_PEB_ref(
    effect: &crate::include::core::SkRefCnt::sk_sp<dyn SkPathEffect>,
) -> Option<&(dyn SkPathEffectBase + 'static)> {
    as_PEB(effect.get())
}
pub fn as_PEB_sp(
    mut effect: crate::include::core::SkRefCnt::sk_sp<dyn SkPathEffect>,
) -> crate::include::core::SkRefCnt::sk_sp<dyn SkPathEffectBase> {
    // Keep the original allocation data pointer's provenance from the owned handle. The
    // short borrow selects concrete dispatch; the projected owner is NOT derived from it.
    let projected = effect
        .as_ptr()
        .map(|owner| unsafe { owner.as_ref().project_base_owner(owner.cast()) });
    // End dispatch's borrow before taking the original reference. No retain or release/free.
    let _ = effect.release();
    match projected {
        Some(ptr) => unsafe { crate::include::core::SkRefCnt::sk_sp::from_raw(ptr.as_ptr()) },
        None => crate::include::core::SkRefCnt::sk_sp::default(),
    }
}

// Native header declarations project the existing canonical implementations.
// These exports add no object, owner, counter, fallback or native body.
// Source: src/core/SkPathEffectBase.h:91-96
pub use super::SkPathEffect::Deserialize;
// Source: src/core/SkPathEffectBase.h:133-133
pub use super::SkPathEffect::RegisterFlattenables;
