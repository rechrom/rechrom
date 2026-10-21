use crate::include::core::SkRefCnt::{SkRefCnt, SkRefCntBase, SkRefCntObject};
use crate::include::private::SkTemplates::AutoTArray;
// Source: src/effects/SkDashImpl.h:17-19
// Rust inheritance dispatch/counter projection is adapted; interval ownership is native AutoTArray.
unsafe impl SkRefCntObject for SkDashImpl {
    // Native complete-object virtual disposal dispatch; this actual concrete
    // Sized type pairs the original raw allocation with its typed adapter.
    fn internal_dispose_raw_proc(&self) -> crate::include::core::SkRefCnt::SkRawDisposeProc {
        crate::include::core::SkRefCnt::SkRefCntDispose::<Self>
    }
    fn ref_cnt_base(&self) -> &SkRefCntBase {
        self.fRefCnt.ref_cnt_base()
    }
    fn ref_(&self) {
        self.fRefCnt.ref_();
    }
}
impl crate::src::core::SkFlattenable::SkFlattenable for SkDashImpl {
    unsafe fn project_path_effect_owner(
        &self,
        allocation: std::ptr::NonNull<()>,
    ) -> Option<std::ptr::NonNull<dyn crate::src::core::SkPathEffect::SkPathEffect>> {
        let concrete: *mut Self = allocation.as_ptr().cast();
        Some(unsafe {
            std::ptr::NonNull::new_unchecked(
                concrete as *mut dyn crate::src::core::SkPathEffect::SkPathEffect,
            )
        })
    }
}
impl crate::src::core::SkFlattenable::SkFlattenableView for SkDashImpl {
    // Source: src/effects/SkDashImpl.h:22-22
    fn flatten(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.flatten_impl(buffer)?;
        Ok(())
    }
    // Source: src/effects/SkDashImpl.h:32-32
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkDashImpl")
    }
    fn getFlattenableType(&self) -> crate::src::core::SkFlattenable::Type {
        crate::src::core::SkPathEffectBase::SkPathEffectBase::getFlattenableType(self)
    }
}
// Source: src/effects/SkDashImpl.h:34-38
impl SkDashImpl {
    pub(crate) fn computeFastBounds_impl(
        &self,
        _bounds: Option<&mut crate::src::core::SkRect::SkRect>,
    ) -> bool {
        true
    }
}
// Source: src/effects/SkDashImpl.h:40-46
// Only the array has a nontrivial member destructor. It drops before native base counter.
pub struct SkDashImpl {
    pub(crate) fIntervals: AutoTArray<f32>,
    pub(crate) fPhase: f32,
    pub(crate) fInitialDashLength: f32,
    pub(crate) fIntervalLength: f32,
    pub(crate) fInitialDashIndex: usize,
    pub(crate) fRefCnt: SkRefCnt,
}
