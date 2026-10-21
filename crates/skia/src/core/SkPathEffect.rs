//! Native retained path-effect dispatch. Native serialization retains the actual intrusive path-effect owner.
use crate::src::core::SkMatrix::SkMatrix;
use crate::src::core::SkPath::SkPath;
use crate::src::core::SkPathBuilder::SkPathBuilder;
use crate::src::core::SkPathEffectBase::SkPathEffectBase;
use crate::src::core::SkRect::SkRect;
use crate::src::core::SkStrokeRec::SkStrokeRec;
// Source: include/core/SkPathEffect.h:34-34
// Rust trait declarations adapt the non-owning virtual-call view only; no owning base is claimed.
pub trait SkPathEffect:
    crate::src::core::SkFlattenable::SkFlattenable
    + crate::src::core::SkPathEffectBase::SkPathEffectProjection
{
    // Source: include/core/SkPathEffect.h:53-55
    fn GetFlattenableType() -> crate::src::core::SkFlattenable::Type
    where
        Self: Sized,
    {
        crate::src::core::SkFlattenable::Type::kSkPathEffect_Type
    }
    // Source: include/core/SkPathEffect.h:72-77
    fn filterPath(
        &self,
        dst: &mut SkPathBuilder,
        src: &SkPath,
        rec: &mut SkStrokeRec,
        bounds: Option<&SkRect>,
        ctm: &SkMatrix,
    ) -> bool;
    fn filterPathWithoutCTM(
        &self,
        dst: &mut SkPathBuilder,
        src: &SkPath,
        rec: &mut SkStrokeRec,
    ) -> bool;
    fn needsCTM(&self) -> bool;
    // Rust projection of the native private as_PEB dispatch; no extra native operation.
    fn computeFastBounds(&self, bounds: Option<&mut SkRect>) -> bool;
}
// Source: src/core/SkPathEffect.cpp:27-43
impl<T: SkPathEffectBase + 'static> SkPathEffect for T {
    fn filterPathWithoutCTM(
        &self,
        dst: &mut SkPathBuilder,
        src: &SkPath,
        rec: &mut SkStrokeRec,
    ) -> bool {
        self.filterPath(dst, src, rec, None, &SkMatrix::default())
    }
    fn filterPath(
        &self,
        dst: &mut SkPathBuilder,
        src: &SkPath,
        rec: &mut SkStrokeRec,
        bounds: Option<&SkRect>,
        ctm: &SkMatrix,
    ) -> bool {
        self.onFilterPath(dst, src, rec, bounds, ctm)
    }
    fn needsCTM(&self) -> bool {
        self.onNeedsCTM()
    }
    fn computeFastBounds(&self, bounds: Option<&mut SkRect>) -> bool {
        SkPathEffectBase::computeFastBounds(self, bounds)
    }
}

// The native Base::asPoints definition shares this original cpp with filter/needsCTM.
pub(crate) fn asPoints<T: SkPathEffectBase + ?Sized>(
    effect: &T,
    results: Option<&mut crate::src::core::SkPathEffectBase::PointData>,
    src: &SkPath,
    rec: &SkStrokeRec,
    mx: &SkMatrix,
    rect: Option<&SkRect>,
) -> bool {
    effect.onAsPoints(results, src, rec, mx, rect)
}
use crate::include::core::SkRefCnt::{sk_sp, SkRefCnt, SkRefCntBase, SkRefCntObject};
use crate::src::core::SkFlattenable::{SkFlattenable, Type};
// Source: src/core/SkPathEffect.cpp:53-53
// Source: src/core/SkPathEffect.cpp:68-69
struct SkPairPathEffect {
    // Rust drops fields in declaration order: preserve native reverse member destruction.
    fPE1: sk_sp<dyn SkPathEffect>,
    fPE0: sk_sp<dyn SkPathEffect>,
    fRefCnt: SkRefCnt,
}
impl SkPairPathEffect {
    // Source: src/core/SkPathEffect.cpp:55-60
    fn new(pe0: sk_sp<dyn SkPathEffect>, pe1: sk_sp<dyn SkPathEffect>) -> Self {
        debug_assert!(pe0.is_some());
        debug_assert!(pe1.is_some());
        Self {
            fPE0: pe0,
            fPE1: pe1,
            fRefCnt: SkRefCnt::new(),
        }
    }
}
// Source: src/core/SkPathEffect.cpp:62-65
impl SkPairPathEffect {
    fn flatten(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        buffer.writeFlattenable(
            self.fPE0
                .get()
                .map(|p| p as &dyn crate::src::core::SkFlattenable::SkFlattenableView),
        )?;
        buffer.writeFlattenable(
            self.fPE1
                .get()
                .map(|p| p as &dyn crate::src::core::SkFlattenable::SkFlattenableView),
        )?;

        Ok(())
    }
}
// Source: src/core/SkPathEffect.cpp:77-77
struct SkComposePathEffect {
    inherited: SkPairPathEffect,
}
impl SkComposePathEffect {
    // Source: src/core/SkPathEffect.cpp:84-95
    fn Make(
        outer: sk_sp<dyn SkPathEffect>,
        inner: sk_sp<dyn SkPathEffect>,
    ) -> sk_sp<dyn SkPathEffect> {
        if !outer.is_some() {
            return inner;
        }
        if !inner.is_some() {
            return outer;
        }
        // Native lvalue arguments copy both handles before moving them into the pair.
        let object: Box<dyn SkPathEffect> = Box::new(Self {
            inherited: SkPairPathEffect::new(outer.clone(), inner.clone()),
        });
        unsafe { sk_sp::from_raw(Box::into_raw(object)) }
    }
}
unsafe impl SkRefCntObject for SkComposePathEffect {
    // Native complete-object virtual disposal dispatch; this actual concrete
    // Sized type pairs the original raw allocation with its typed adapter.
    fn internal_dispose_raw_proc(&self) -> crate::include::core::SkRefCnt::SkRawDisposeProc {
        crate::include::core::SkRefCnt::SkRefCntDispose::<Self>
    }

    fn ref_cnt_base(&self) -> &SkRefCntBase {
        self.inherited.fRefCnt.ref_cnt_base()
    }
    fn ref_(&self) {
        self.inherited.fRefCnt.ref_();
    }
}
impl SkPathEffectBase for SkComposePathEffect {
    // Source: src/core/SkPathEffect.cpp:97-107
    fn onFilterPath(
        &self,
        builder: &mut SkPathBuilder,
        src: &SkPath,
        rec: &mut SkStrokeRec,
        cullRect: Option<&SkRect>,
        ctm: &SkMatrix,
    ) -> bool {
        let mut tmp = SkPath::default();
        let mut ptr = src;
        if self
            .inherited
            .fPE1
            .get()
            .unwrap()
            .filterPath(builder, src, rec, cullRect, ctm)
        {
            tmp = builder.detach(None);
            ptr = &tmp;
        }
        self.inherited
            .fPE0
            .get()
            .unwrap()
            .filterPath(builder, ptr, rec, cullRect, ctm)
    }
    fn computeFastBounds(&self, bounds: Option<&mut SkRect>) -> bool {
        self.computeFastBounds_impl(bounds)
    }
}
// Source: src/core/SkPathEffect.cpp:109-109
impl SkFlattenable for SkComposePathEffect {
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
impl crate::src::core::SkFlattenable::SkFlattenableView for SkComposePathEffect {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }

    // Actual native SK_FLATTENABLE_HOOKS dispatch and inherited pair serialization.
    fn flatten(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.inherited.flatten(buffer)?;
        Ok(())
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkComposePathEffect")
    }
    fn getFlattenableType(&self) -> Type {
        SkPathEffectBase::getFlattenableType(self)
    }
}
// Source: src/core/SkPathEffect.cpp:111-115
impl SkComposePathEffect {
    fn computeFastBounds_impl(&self, mut bounds: Option<&mut SkRect>) -> bool {
        SkPathEffectBase::computeFastBounds(
            crate::src::core::SkPathEffectBase::as_PEB_ref(&self.inherited.fPE1).unwrap(),
            bounds.as_deref_mut(),
        ) && SkPathEffectBase::computeFastBounds(
            crate::src::core::SkPathEffectBase::as_PEB_ref(&self.inherited.fPE0).unwrap(),
            bounds,
        )
    }
}
// Source: src/core/SkPathEffect.cpp:127-131
impl SkComposePathEffect {
    pub(crate) fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        let pe0 = buffer.readPathEffect()?;
        let pe1 = buffer.readPathEffect()?;
        Ok(into_flattenable(Self::Make(pe0, pe1)))
    }
}
// Source: src/core/SkPathEffect.cpp:139-139
struct SkSumPathEffect {
    inherited: SkPairPathEffect,
}
impl SkSumPathEffect {
    // Source: src/core/SkPathEffect.cpp:146-157
    fn Make(
        first: sk_sp<dyn SkPathEffect>,
        second: sk_sp<dyn SkPathEffect>,
    ) -> sk_sp<dyn SkPathEffect> {
        if !first.is_some() {
            return second;
        }
        if !second.is_some() {
            return first;
        }
        let object: Box<dyn SkPathEffect> = Box::new(Self {
            inherited: SkPairPathEffect::new(first.clone(), second.clone()),
        });
        unsafe { sk_sp::from_raw(Box::into_raw(object)) }
    }
}
unsafe impl SkRefCntObject for SkSumPathEffect {
    // Native complete-object virtual disposal dispatch; this actual concrete
    // Sized type pairs the original raw allocation with its typed adapter.
    fn internal_dispose_raw_proc(&self) -> crate::include::core::SkRefCnt::SkRawDisposeProc {
        crate::include::core::SkRefCnt::SkRefCntDispose::<Self>
    }

    fn ref_cnt_base(&self) -> &SkRefCntBase {
        self.inherited.fRefCnt.ref_cnt_base()
    }
    fn ref_(&self) {
        self.inherited.fRefCnt.ref_();
    }
}
impl SkPathEffectBase for SkSumPathEffect {
    // Source: src/core/SkPathEffect.cpp:159-165
    fn onFilterPath(
        &self,
        builder: &mut SkPathBuilder,
        src: &SkPath,
        rec: &mut SkStrokeRec,
        cullRect: Option<&SkRect>,
        ctm: &SkMatrix,
    ) -> bool {
        let filteredFirst = self
            .inherited
            .fPE0
            .get()
            .unwrap()
            .filterPath(builder, src, rec, cullRect, ctm);
        let filteredSecond = self
            .inherited
            .fPE1
            .get()
            .unwrap()
            .filterPath(builder, src, rec, cullRect, ctm);
        filteredFirst || filteredSecond
    }
    fn computeFastBounds(&self, bounds: Option<&mut SkRect>) -> bool {
        self.computeFastBounds_impl(bounds)
    }
}
// Source: src/core/SkPathEffect.cpp:167-167
impl SkFlattenable for SkSumPathEffect {
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
impl crate::src::core::SkFlattenable::SkFlattenableView for SkSumPathEffect {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }

    fn flatten(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.inherited.flatten(buffer)?;
        Ok(())
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkSumPathEffect")
    }
    fn getFlattenableType(&self) -> Type {
        SkPathEffectBase::getFlattenableType(self)
    }
}
// Source: src/core/SkPathEffect.cpp:169-173
impl SkSumPathEffect {
    fn computeFastBounds_impl(&self, mut bounds: Option<&mut SkRect>) -> bool {
        SkPathEffectBase::computeFastBounds(
            crate::src::core::SkPathEffectBase::as_PEB_ref(&self.inherited.fPE0).unwrap(),
            bounds.as_deref_mut(),
        ) && SkPathEffectBase::computeFastBounds(
            crate::src::core::SkPathEffectBase::as_PEB_ref(&self.inherited.fPE1).unwrap(),
            bounds,
        )
    }
}
// Source: src/core/SkPathEffect.cpp:184-188
impl SkSumPathEffect {
    pub(crate) fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        let pe0 = buffer.readPathEffect()?;
        let pe1 = buffer.readPathEffect()?;
        Ok(into_flattenable(Self::Make(pe0.clone(), pe1.clone())))
    }
}
// Source: src/core/SkPathEffect.cpp:192-194
pub fn MakeSum(
    first: sk_sp<dyn SkPathEffect>,
    second: sk_sp<dyn SkPathEffect>,
) -> sk_sp<dyn SkPathEffect> {
    SkSumPathEffect::Make(first, second)
}
// Source: src/core/SkPathEffect.cpp:196-199
pub fn MakeCompose(
    outer: sk_sp<dyn SkPathEffect>,
    inner: sk_sp<dyn SkPathEffect>,
) -> sk_sp<dyn SkPathEffect> {
    SkComposePathEffect::Make(outer, inner)
}

// Ownership adaptation for native sk_sp<Derived> -> sk_sp<SkFlattenable> moves.
pub(crate) fn into_flattenable(
    mut effect: sk_sp<dyn SkPathEffect>,
) -> Option<crate::src::core::SkFlattenable::FlattenableOwner> {
    let raw = effect.release()?;
    Some(
        crate::src::core::SkFlattenable::FlattenableOwner::Intrusive(unsafe {
            sk_sp::from_raw(raw.as_ptr() as *mut dyn SkFlattenable)
        }),
    )
}
// Source: src/core/SkPathEffect.cpp:201-204
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn RegisterFlattenables() {
    unsafe {
        crate::src::core::SkFlattenable::Register(
            b"SkComposePathEffect",
            SkComposePathEffect::CreateProc,
        );
        crate::src::core::SkFlattenable::Register(b"SkSumPathEffect", SkSumPathEffect::CreateProc);
    }
}
// Source: src/core/SkPathEffect.cpp:206-211
pub fn Deserialize(
    data: &[u8],
    procs: Option<&crate::include::core::SkSerialProcs::SkDeserialProcs>,
) -> Result<sk_sp<dyn SkPathEffect>, crate::RasterError> {
    let Some(mut effect) =
        crate::src::core::SkFlattenable::Deserialize(Type::kSkPathEffect_Type, data, procs)?
    else {
        return Ok(sk_sp::default());
    };
    let crate::src::core::SkFlattenable::FlattenableOwner::Intrusive(mut effect) = effect else {
        panic!("checked native path effect must retain its actual intrusive owner");
    };
    let owner = effect
        .as_ptr()
        .expect("native deserializer returned live owner");
    let projected = unsafe { owner.as_ref().project_path_effect_owner(owner.cast()) }
        .expect("checked path-effect type requires its actual owner projection");
    effect.release();
    Ok(unsafe { sk_sp::from_raw(projected.as_ptr()) })
}
