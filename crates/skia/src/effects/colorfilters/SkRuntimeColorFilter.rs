// Real native runtime effect owner and actual RP execution; unverified.
use crate::src::core::{
    SkData::SkData,
    SkEffectPriv::SkStageRec,
    SkFlattenable::{Factory, FactoryResult, FlattenableOwner, SkFlattenableView, Type},
    SkMatrix::SkMatrix,
    SkReadBuffer::SkReadBuffer,
    SkRuntimeEffect::{
        ChildPtr, ReadChildEffects, SkMakeCachedRuntimeEffect, SkRuntimeEffect, WriteChildEffects,
    },
    SkRuntimeEffectPriv::{self, RuntimeEffectRPCallbacks},
    SkWriteBuffer::SkWriteBuffer,
};
use crate::src::shaders::SkShaderBase::MatrixRec;
use std::sync::Arc;
// Source: src/effects/colorfilters/SkRuntimeColorFilter.cpp:37-37
#[cfg(SK_BUILD_FOR_DEBUGGER)]
const kLenientSkSLDeserialization: bool = true;
// Source: src/effects/colorfilters/SkRuntimeColorFilter.cpp:39-39
#[cfg(not(SK_BUILD_FOR_DEBUGGER))]
const kLenientSkSLDeserialization: bool = false;
// UNVERIFIED: native constexpr selected branch controls actual CreateProc below.
// Source: src/effects/colorfilters/SkRuntimeColorFilter.h:43-45
impl SkRuntimeColorFilter {
    pub fn effect(&self) -> Arc<SkRuntimeEffect> {
        Arc::clone(&self.fEffect)
    }
    pub fn uniforms(&self) -> Arc<SkData> {
        Arc::clone(&self.fUniforms)
    }
    pub fn children(&self) -> &[ChildPtr] {
        &self.fChildren
    }
}
// Source: src/effects/colorfilters/SkRuntimeColorFilter.h:48-50
pub struct SkRuntimeColorFilter {
    fEffect: core::mem::ManuallyDrop<Arc<SkRuntimeEffect>>,
    fUniforms: core::mem::ManuallyDrop<Arc<SkData>>,
    fChildren: Vec<ChildPtr>,
}
// Native runtime color filters are immutable retained effects. Raster-pipeline
// state is allocated per invocation; the effect, uniform data and child list
// are only read after this object is published.
unsafe impl Send for SkRuntimeColorFilter {}
unsafe impl Sync for SkRuntimeColorFilter {}
// Preserve h48-50 declaration order exactly. The local tuple enforces native
// reverse owner destruction, including cleanup of later owners on Rust unwind.
// UNVERIFIED language adaptation; no default/replacement Arc is manufactured.
impl Drop for SkRuntimeColorFilter {
    fn drop(&mut self) {
        struct Owner<'a, T>(&'a mut core::mem::ManuallyDrop<T>);
        impl<T> Drop for Owner<'_, T> {
            fn drop(&mut self) {
                unsafe {
                    core::mem::ManuallyDrop::drop(self.0);
                }
            }
        }
        let owners = (
            crate::src::core::SkRuntimeEffect::RuntimeChildren(core::mem::take(
                &mut self.fChildren,
            )),
            Owner(&mut self.fUniforms),
            Owner(&mut self.fEffect),
        );
        drop(owners);
    }
}
impl SkFlattenableView for SkRuntimeColorFilter {
    // Source: src/effects/colorfilters/SkRuntimeColorFilter.h:41-41
    fn getFactory(&self) -> Option<Factory> {
        Some(Self::CreateProc)
    }
    // Source: src/effects/colorfilters/SkRuntimeColorFilter.h:35-35
    fn flatten(&self, buffer: &mut dyn SkWriteBuffer) -> Result<(), crate::RasterError> {
        self.flatten_impl(buffer)?;
        Ok(())
    }
    // Source: src/effects/colorfilters/SkRuntimeColorFilter.h:41-41
    fn getTypeName(&self) -> Option<&str> {
        Some("SkRuntimeColorFilter")
    }
    // Source: src/effects/colorfilters/SkRuntimeColorFilter.h:25-25
    fn getFlattenableType(&self) -> Type {
        Type::kSkColorFilter_Type
    }
}
impl crate::src::core::SkColorFilter::SkColorFilter for SkRuntimeColorFilter {}
impl crate::src::effects::colorfilters::SkColorFilterBase::SkColorFilterBase
    for SkRuntimeColorFilter
{
    // Source: src/effects/colorfilters/SkRuntimeColorFilter.h:31-31
    unsafe fn appendStages(
        &self,
        rec: &SkStageRec<'_>,
        _opaque: bool,
    ) -> Result<bool, crate::RasterError> {
        self.append_runtime_stages(rec)
    }
    // Source: src/effects/colorfilters/SkRuntimeColorFilter.cpp:71-73
    fn onIsAlphaUnchanged(&self) -> bool {
        self.fEffect.isAlphaUnchanged()
    }
    // Source: src/effects/colorfilters/SkRuntimeColorFilter.cpp:87-87
    fn asRuntimeEffect(&self) -> Option<&SkRuntimeEffect> {
        Some(&self.fEffect)
    }
    // Source: src/effects/colorfilters/SkRuntimeColorFilter.h:39-39
    fn r#type(&self) -> crate::src::effects::colorfilters::SkColorFilterBase::Type {
        crate::src::effects::colorfilters::SkColorFilterBase::Type::kRuntime
    }
}

// Source: src/effects/colorfilters/SkRuntimeColorFilter.h:27-29
// Source: src/effects/colorfilters/SkRuntimeColorFilter.cpp:42-47
impl SkRuntimeColorFilter {
    pub fn new(effect: Arc<SkRuntimeEffect>, uniforms: Arc<SkData>, children: &[ChildPtr]) -> Self {
        Self {
            fEffect: core::mem::ManuallyDrop::new(effect),
            fUniforms: core::mem::ManuallyDrop::new(uniforms),
            fChildren: children.to_vec(),
        }
    }
}
// Source: src/effects/colorfilters/SkRuntimeColorFilter.cpp:49-69
impl SkRuntimeColorFilter {
    unsafe fn append_runtime_stages(
        &self,
        rec: &SkStageRec<'_>,
    ) -> Result<bool, crate::RasterError> {
        if !SkRuntimeEffectPriv::CanDraw(
            crate::include::core::SkCapabilities::SkCapabilities::RasterBackend().as_ref(),
            &self.fEffect,
        ) {
            return Ok(false);
        }
        if let Some(program) = self.fEffect.getRPProgram(None)? {
            let span = SkRuntimeEffectPriv::UniformsAsSpan(
                self.fEffect.uniforms(),
                Arc::clone(&self.fUniforms),
                false,
                rec.fDstCS.map(Arc::as_ref),
                &mut *rec.fAlloc,
            );
            let uniforms = if span.1 == 0 {
                &[]
            } else {
                core::slice::from_raw_parts(span.0, span.1)
            };
            let mut matrix = MatrixRec::new(&SkMatrix::default());
            matrix.markCTMApplied();
            let mut callbacks = RuntimeEffectRPCallbacks::new(
                rec,
                &matrix,
                &self.fChildren,
                &self.fEffect.fSampleUsages,
            );
            return program.appendStages(rec.fPipeline, rec.fAlloc, Some(&mut callbacks), uniforms);
        }
        Ok(false)
    }
}
// Source: src/effects/colorfilters/SkRuntimeColorFilter.cpp:75-85
impl SkRuntimeColorFilter {
    fn flatten_impl(&self, buffer: &mut dyn SkWriteBuffer) -> Result<(), crate::RasterError> {
        if crate::src::core::SkKnownRuntimeEffects::IsSkiaKnownRuntimeEffect(
            self.fEffect.fStableKey as i32,
        ) {
            buffer.write32(self.fEffect.fStableKey as i32);
        } else {
            buffer.write32(0);
            let text = self.fEffect.source();
            let len = text.iter().position(|b| *b == 0).unwrap_or(text.len());
            buffer.writeStringBytes(&text[..len]);
        }
        buffer.writeDataAsByteArray(Some(&self.fUniforms));
        WriteChildEffects(buffer, &self.fChildren)?;

        Ok(())
    }
}
// Source: src/effects/colorfilters/SkRuntimeColorFilter.cpp:89-129
impl SkRuntimeColorFilter {
    pub fn CreateProc(buffer: &mut SkReadBuffer<'_>) -> FactoryResult {
        if !buffer.validate(buffer.allowSkSL()) {
            return Ok(None);
        }
        let mut effect = None;
        if !buffer.isVersionLT(crate::src::core::SkPicturePriv::kSerializeStableKeys) {
            let key = buffer.readUInt();
            effect = crate::src::core::SkKnownRuntimeEffects::MaybeGetKnownRuntimeEffect(key);
            if effect.is_none() && !buffer.validate(key == 0) {
                return Ok(None);
            }
        }
        if effect.is_none() {
            let mut sksl = Vec::new();
            buffer.readString(&mut sksl);
            effect = SkMakeCachedRuntimeEffect(SkRuntimeEffect::MakeForColorFilter, sksl);
        }
        if !kLenientSkSLDeserialization && !buffer.validate(effect.is_some()) {
            return Ok(None);
        }
        let uniforms = buffer.readByteArrayAsData();
        let mut children = Vec::new();
        if !ReadChildEffects(buffer, effect.as_ref().map(|e| e.children()), &mut children)? {
            return Ok(None);
        }
        if kLenientSkSLDeserialization && effect.is_none() {
            unsafe {
                crate::SkDebugf!(@source b"src/effects/colorfilters/SkRuntimeColorFilter.cpp\0".as_ptr(),123;b"Serialized SkSL failed to compile. Ignoring/dropping SkSL color filter.\n\0".as_ptr())
            };
            return Ok(None);
        }
        let Some(effect) = effect else {
            return Ok(None);
        };
        Ok(effect
            .makeColorFilter(uniforms, &children)
            .map(FlattenableOwner::ColorFilter))
    }
}
// Source: src/effects/colorfilters/SkRuntimeColorFilter.cpp:133-160
pub fn Lerp(
    weight: f32,
    cf0: Option<Arc<dyn crate::src::core::SkColorFilter::SkColorFilter>>,
    cf1: Option<Arc<dyn crate::src::core::SkColorFilter::SkColorFilter>>,
) -> Option<Arc<dyn crate::src::core::SkColorFilter::SkColorFilter>> {
    if cf0.is_none() && cf1.is_none() {
        return None;
    }
    if weight.is_nan() {
        return None;
    }
    if matches!((&cf0,&cf1),(Some(a),Some(b))if Arc::ptr_eq(a,b)) {
        return cf0;
    }
    if weight <= 0.0 {
        return cf0;
    }
    if weight >= 1.0 {
        return cf1;
    }
    let effect = crate::src::core::SkKnownRuntimeEffects::GetKnownRuntimeEffect(
        crate::src::core::SkKnownRuntimeEffects::StableKey::kLerp,
    )
    .expect("native known lerp effect");
    let data =
        unsafe { SkData::MakeWithCopy(weight.to_ne_bytes().as_ptr(), core::mem::size_of::<f32>()) };
    effect.makeColorFilterFromFilters(Some(data), &[cf0, cf1])
}
// Source: src/effects/colorfilters/SkRuntimeColorFilter.cpp:162-168
pub fn MakeLumaColorFilter() -> Option<Arc<dyn crate::src::core::SkColorFilter::SkColorFilter>> {
    let effect = crate::src::core::SkKnownRuntimeEffects::GetKnownRuntimeEffect(
        crate::src::core::SkKnownRuntimeEffects::StableKey::kLuma,
    )
    .expect("native known luma effect");
    effect.makeColorFilterNoChildren(Some(SkData::MakeEmpty()))
}
// Source: src/effects/colorfilters/SkRuntimeColorFilter.cpp:170-181
// The actual kNumColors declaration is include/effects/SkOverdrawColorFilter.h:28.
// The newly admitted header owns the actual class and count.
pub fn MakeWithSkColors(
    colors: &[crate::src::core::SkColor::SkColor;
         crate::include::effects::SkOverdrawColorFilter::SkOverdrawColorFilter::kNumColors
             as usize],
) -> Option<Arc<dyn crate::src::core::SkColorFilter::SkColorFilter>> {
    use crate::include::effects::SkOverdrawColorFilter::SkOverdrawColorFilter;
    use crate::src::core::{
        SkColor::SkColor4f,
        SkColorData::SkPMColor4f,
        SkKnownRuntimeEffects::{GetKnownRuntimeEffect, StableKey},
    };
    let overdrawEffect =
        GetKnownRuntimeEffect(StableKey::kOverdraw).expect("native known overdraw effect");
    let data = unsafe {
        SkData::MakeUninitialized(
            SkOverdrawColorFilter::kNumColors as usize * core::mem::size_of::<SkPMColor4f>(),
        )
    };
    let premul = unsafe { data.writable_data().cast::<SkPMColor4f>() };
    for i in 0..SkOverdrawColorFilter::kNumColors {
        unsafe {
            premul
                .add(i as usize)
                .write(SkColor4f::FromColor(colors[i as usize]).premul());
        }
    }
    overdrawEffect.makeColorFilterNoChildren(Some(data))
}
