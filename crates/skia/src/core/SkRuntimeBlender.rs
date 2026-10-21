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
// Source: src/core/SkRuntimeBlender.h:26-31
impl SkRuntimeBlender {
    pub fn new(effect: Arc<SkRuntimeEffect>, uniforms: Arc<SkData>, children: &[ChildPtr]) -> Self {
        Self {
            fEffect: core::mem::ManuallyDrop::new(effect),
            fUniforms: core::mem::ManuallyDrop::new(uniforms),
            fChildren: children.to_vec(),
            fRefCnt: crate::include::core::SkRefCnt::SkRefCnt::new(),
        }
    }
    pub fn Make(
        effect: Arc<SkRuntimeEffect>,
        uniforms: Arc<SkData>,
        children: &[ChildPtr],
    ) -> crate::include::core::SkRefCnt::sk_sp<dyn crate::include::core::SkBlender::SkBlender> {
        let owner: Box<dyn crate::include::core::SkBlender::SkBlender> =
            Box::new(Self::new(effect, uniforms, children));
        unsafe { crate::include::core::SkRefCnt::sk_sp::from_raw(Box::into_raw(owner)) }
    }
}
// Source: src/core/SkRuntimeBlender.h:43-45
impl SkRuntimeBlender {
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
// Source: src/core/SkRuntimeBlender.h:48-50
pub struct SkRuntimeBlender {
    fEffect: core::mem::ManuallyDrop<Arc<SkRuntimeEffect>>,
    fUniforms: core::mem::ManuallyDrop<Arc<SkData>>,
    fChildren: Vec<ChildPtr>,
    fRefCnt: crate::include::core::SkRefCnt::SkRefCnt,
}
// Native SkRuntimeBlender is an immutable SkRefCnt object after construction.
// Its child effects are retained native effect owners and every mutation during
// raster happens in the call-local pipeline arena, never in these fields.
unsafe impl Send for SkRuntimeBlender {}
unsafe impl Sync for SkRuntimeBlender {}
// Preserve h48-50 declaration order exactly. The local tuple enforces native
// reverse owner destruction, including cleanup of later owners on Rust unwind.
// UNVERIFIED language adaptation; no default/replacement Arc is manufactured.
impl Drop for SkRuntimeBlender {
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
impl SkFlattenableView for SkRuntimeBlender {
    // Source: src/core/SkRuntimeBlender.h:41-41
    fn getFactory(&self) -> Option<Factory> {
        Some(Self::CreateProc)
    }
    // Source: src/core/SkRuntimeBlender.h:39-39
    fn flatten(&self, buffer: &mut dyn SkWriteBuffer) -> Result<(), crate::RasterError> {
        self.flatten_impl(buffer)?;
        Ok(())
    }
    // Source: src/core/SkRuntimeBlender.h:41-41
    fn getTypeName(&self) -> Option<&str> {
        Some("SkRuntimeBlender")
    }
    // Source: src/core/SkRuntimeBlender.h:24-24
    fn getFlattenableType(&self) -> Type {
        Type::kSkBlender_Type
    }
}
unsafe impl crate::include::core::SkRefCnt::SkRefCntObject for SkRuntimeBlender {
    fn internal_dispose_raw_proc(&self) -> crate::include::core::SkRefCnt::SkRawDisposeProc {
        crate::include::core::SkRefCnt::SkRefCntDispose::<Self>
    }
    fn ref_cnt_base(&self) -> &crate::include::core::SkRefCnt::SkRefCntBase {
        &self.fRefCnt
    }
    fn ref_(&self) {
        crate::include::core::SkRefCnt::SkRefCnt::ref_(&self.fRefCnt);
    }
}
impl crate::src::core::SkFlattenable::SkFlattenable for SkRuntimeBlender {
    unsafe fn project_path_effect_owner(
        &self,
        _allocation: core::ptr::NonNull<()>,
    ) -> Option<core::ptr::NonNull<dyn crate::src::core::SkPathEffect::SkPathEffect>> {
        None
    }
}
impl crate::include::core::SkBlender::SkBlender for SkRuntimeBlender {}
impl crate::src::core::SkBlenderBase::SkBlenderBase for SkRuntimeBlender {
    // Source: src/core/SkRuntimeBlender.h:37-37
    unsafe fn onAppendStages(&self, rec: &SkStageRec<'_>) -> Result<bool, crate::RasterError> {
        self.append_runtime_stages(rec)
    }
    // Source: src/core/SkRuntimeBlender.h:33-33
    fn asRuntimeEffect(&self) -> Option<&SkRuntimeEffect> {
        Some(&self.fEffect)
    }
    // Source: src/core/SkRuntimeBlender.h:35-35
    fn r#type(&self) -> crate::src::core::SkBlenderBase::BlenderType {
        crate::src::core::SkBlenderBase::BlenderType::kRuntime
    }
}

// Source: src/core/SkRuntimeBlender.cpp:32-32
#[cfg(SK_BUILD_FOR_DEBUGGER)]
const kLenientSkSLDeserialization: bool = true;
// Source: src/core/SkRuntimeBlender.cpp:34-34
#[cfg(not(SK_BUILD_FOR_DEBUGGER))]
const kLenientSkSLDeserialization: bool = false;
// UNVERIFIED: native constexpr selected branch controls actual CreateProc below.
// Source: src/core/SkRuntimeBlender.cpp:37-47
impl SkRuntimeBlender {
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
// Source: src/core/SkRuntimeBlender.cpp:49-89
impl SkRuntimeBlender {
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
            effect = SkMakeCachedRuntimeEffect(SkRuntimeEffect::MakeForBlender, sksl);
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
                crate::SkDebugf!(@source b"src/core/SkRuntimeBlender.cpp\0".as_ptr(),83;b"Serialized SkSL failed to compile. Ignoring/dropping SkSL blender.\n\0".as_ptr())
            };
            return Ok(None);
        }
        let Some(effect) = effect else {
            return Ok(None);
        };
        let blender = effect.makeBlender(uniforms, &children);
        Ok(if blender.get().is_some() {
            Some(FlattenableOwner::Blender(blender))
        } else {
            None
        })
    }
}
// Source: src/core/SkRuntimeBlender.cpp:91-111
impl SkRuntimeBlender {
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
