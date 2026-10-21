// RasterError interface/propagation adaptation only; UNVERIFIED; no added native-line credit.
// Copyright 2023 Google LLC. BSD-style license.
// Real source-aligned CPU shader uses the retained compiled effect and RP Program.
use super::{
    SkShader::SkShader,
    SkShaderBase::{MatrixRec, ShaderType, SkShaderBase},
};
use crate::src::core::SkRuntimeEffect::{ChildPtr, SkRuntimeEffect, TracedShader};
use crate::src::core::SkRuntimeEffectPriv::{
    self, RuntimeEffectRPCallbacks, UniformsCallbackContext,
};
use crate::src::core::{
    SkColorSpace::SkColorSpace, SkData::SkData, SkEffectPriv::SkStageRec, SkMatrix::SkMatrix,
    SkPoint::SkIPoint, SkRasterPipeline::MissingStage,
};
use crate::src::sksl::tracing::SkSLDebugTracePriv::DebugTracePriv;
use std::sync::Arc;
// Source: src/shaders/SkRuntimeShader.h:28-28
pub type UniformsCallback = SkRuntimeEffectPriv::UniformsCallback;
// Source: src/shaders/SkRuntimeShader.h:44-44
impl SkShader for SkRuntimeShader {
    fn isOpaque(&self) -> bool {
        self.fEffect.alwaysOpaque()
    }
}
// Source: src/shaders/SkRuntimeShader.h:46-46
impl SkRuntimeShader {
    fn header_type(&self) -> ShaderType {
        ShaderType::kRuntime
    }
}
// Source: src/shaders/SkRuntimeShader.h:52-52
impl SkRuntimeShader {
    // Concrete method is present. Native SkShaderBase.h365 virtual-query slot is still
    // a shared-interface dependency; no fake dynamic-query fallback is added here.
    pub fn asRuntimeEffect(&self) -> &SkRuntimeEffect {
        self.fEffect.as_ref()
    }
}
// Source: src/shaders/SkRuntimeShader.h:54-55
impl SkRuntimeShader {
    pub fn effect(&self) -> Arc<SkRuntimeEffect> {
        Arc::clone(&self.fEffect)
    }
    pub fn children(&self) -> &[ChildPtr] {
        &self.fChildren
    }
}
// Source: src/shaders/SkRuntimeShader.h:66-70
// Native fields stay in source order. Arc/Option/Vec retain the actual effect, trace,
// uniform bytes, callback and child owners; they do not replace their implementations.
pub struct SkRuntimeShader {
    fEffect: Arc<SkRuntimeEffect>,
    fDebugTrace: Option<Arc<DebugTracePriv>>,
    fUniformData: Option<Arc<SkData>>,
    fUniformsCallback: Option<UniformsCallback>,
    fChildren: Vec<ChildPtr>,
}
// Native std::vector<ChildPtr> destroys from end to begin (selected Apple
// libc++ __vector/vector.h:746-750). UNVERIFIED Rust ownership adaptation.
// Tuple drop glue cleans all later owners during unwind before the automatic
// fEffect Arc field drop; effect retains its original public/internal Arc type.
impl Drop for SkRuntimeShader {
    fn drop(&mut self) {
        let owners = (
            crate::src::core::SkRuntimeEffect::RuntimeChildren(core::mem::take(
                &mut self.fChildren,
            )),
            self.fUniformsCallback.take(),
            self.fUniformData.take(),
            self.fDebugTrace.take(),
        );
        drop(owners);
    }
}
// Source: src/shaders/SkRuntimeShader.cpp:45-52
impl SkRuntimeShader {
    pub fn new(
        effect: Arc<SkRuntimeEffect>,
        debugTrace: Option<Arc<DebugTracePriv>>,
        uniforms: Option<Arc<SkData>>,
        children: &[ChildPtr],
    ) -> Self {
        Self {
            fEffect: effect,
            fDebugTrace: debugTrace,
            fUniformData: uniforms,
            fUniformsCallback: None,
            fChildren: children.to_vec(),
        }
    }
}
// Source: src/shaders/SkRuntimeShader.cpp:54-61
impl SkRuntimeShader {
    pub fn new_with_callback(
        effect: Arc<SkRuntimeEffect>,
        debugTrace: Option<Arc<DebugTracePriv>>,
        uniformsCallback: UniformsCallback,
        children: &[ChildPtr],
    ) -> Self {
        Self {
            fEffect: effect,
            fDebugTrace: debugTrace,
            fUniformData: None,
            fUniformsCallback: Some(uniformsCallback),
            fChildren: children.to_vec(),
        }
    }
}
// Source: src/shaders/SkRuntimeShader.cpp:63-69
fn make_debug_trace(effect: &SkRuntimeEffect, coord: &SkIPoint) -> Arc<DebugTracePriv> {
    // Build the real mutable trace before publishing its sole Arc allocation.
    let mut debugTrace = DebugTracePriv::new();
    debugTrace.setSource(effect.source());
    debugTrace.setTraceCoord(coord);
    Arc::new(debugTrace)
}
// Source: src/shaders/SkRuntimeShader.cpp:71-78
impl SkRuntimeShader {
    pub fn makeTracedClone(&self, coord: &SkIPoint) -> Result<TracedShader, MissingStage> {
        let unoptimized = unsafe { self.fEffect.makeUnoptimizedClone() };
        let debugTrace = make_debug_trace(&unoptimized, coord);
        let debugShader: Arc<dyn SkShader> = Arc::new(Self::new(
            unoptimized,
            Some(Arc::clone(&debugTrace)),
            self.uniformData(None),
            &self.fChildren,
        ));
        Ok(TracedShader {
            shader: Some(debugShader),
            debugTrace: Some(debugTrace),
        })
    }
}
// Source: src/shaders/SkRuntimeShader.cpp:80-102
impl SkShaderBase for SkRuntimeShader {
    fn asRuntimeEffect(&self) -> Option<&SkRuntimeEffect> {
        Some(&self.fEffect)
    }
    fn asRuntimeShader(&self) -> Option<&Self> {
        Some(self)
    }
    // Shared virtual dispatch delegates the source header's exact type method.
    fn r#type(&self) -> ShaderType {
        self.header_type()
    }
    /// Caller keeps rec's actual pipeline/arena and borrowed shader children alive
    /// throughout construction and execution of their source-aligned stage contexts.
    unsafe fn appendStages(
        &self,
        rec: &SkStageRec<'_>,
        mRec: &MatrixRec,
    ) -> Result<bool, crate::RasterError> {
        if !SkRuntimeEffectPriv::CanDraw(
            crate::src::core::SkCapabilities::RasterBackend().as_ref(),
            self.fEffect.as_ref(),
        ) {
            // This is the native version/capability rejection, never an unimplemented fallback.
            return Ok(false);
        }
        if let Some(program) = self.fEffect.getRPProgram(self.fDebugTrace.as_ref())? {
            let Some(newMRec) = (unsafe { mRec.apply(rec, &SkMatrix::default()) }) else {
                return Ok(false);
            };
            let uniforms = unsafe {
                SkRuntimeEffectPriv::UniformsAsSpan(
                    self.fEffect.uniforms(),
                    self.uniformData(rec.fDstCS)
                        .expect("native UniformsAsSpan requires nonnull data at its dereference"),
                    self.fUniformData.is_none(),
                    rec.fDstCS.map(Arc::as_ref),
                    &mut *rec.fAlloc,
                )
            };
            let uniforms = if uniforms.1 == 0 {
                &[]
            } else {
                unsafe { core::slice::from_raw_parts(uniforms.0, uniforms.1) }
            };
            let mut callbacks = RuntimeEffectRPCallbacks::new(
                rec,
                &newMRec,
                &self.fChildren,
                &self.fEffect.fSampleUsages,
            );
            let success = unsafe {
                program.appendStages(rec.fPipeline, rec.fAlloc, Some(&mut callbacks), uniforms)
            }?;
            return Ok(success);
        }
        // Native absence of a genuinely compiled RP program; producer MissingStage propagates above.
        Ok(false)
    }
}
// Source: src/shaders/SkRuntimeShader.cpp:104-114
impl SkRuntimeShader {
    pub fn uniformData(&self, dstCS: Option<&Arc<SkColorSpace>>) -> Option<Arc<SkData>> {
        if let Some(uniforms) = &self.fUniformData {
            return Some(Arc::clone(uniforms));
        }
        // Invoke the real late-binding callback for each paint; do not memoize it.
        crate::SkASSERT!(@source b"src/shaders/SkRuntimeShader.cpp\0".as_ptr(),110; @condition b"fUniformsCallback\0".as_ptr(); self.fUniformsCallback.is_some());
        let callback = self
            .fUniformsCallback
            .as_ref()
            .expect("native uniforms callback is required");
        let uniforms = callback(&UniformsCallbackContext {
            fDstColorSpace: dstCS,
        });
        crate::SkASSERT!(@source b"src/shaders/SkRuntimeShader.cpp\0".as_ptr(),112; @condition b"uniforms && uniforms->size() == fEffect->uniformSize()\0".as_ptr(); uniforms.as_ref().is_some_and(|data|data.size()==self.fEffect.uniformSize()));
        uniforms
    }
}
// Native cpp116-186 flatten/CreateProc, header50/59/62-64 remain explicit serialization/debugger
// dependencies outside this CPU shader slice. No empty read/write implementation is supplied.

// Native reverse field destruction; only the actual Effect remains for automatic drop.

// Source: src/shaders/SkRuntimeShader.h:62-64
const kHasLegacyLocalMatrix_Flag: u32 = 1 << 1;
// Source: src/shaders/SkRuntimeShader.cpp:116-128
impl SkRuntimeShader {
    fn flatten_impl(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        if crate::src::core::SkKnownRuntimeEffects::IsSkiaKnownRuntimeEffect(
            self.fEffect.fStableKey as i32,
        ) {
            buffer.write32(self.fEffect.fStableKey as i32);
        } else {
            buffer.write32(0);
            let text = self.fEffect.source();
            let length = text.iter().position(|b| *b == 0).unwrap_or(text.len());
            buffer.writeStringBytes(&text[..length]);
        }
        let uniforms = self.uniformData(None);
        buffer.writeDataAsByteArray(uniforms.as_deref());
        crate::src::core::SkRuntimeEffect::WriteChildEffects(buffer, &self.fChildren)?;

        Ok(())
    }
    // Source: src/shaders/SkRuntimeShader.cpp:130-186
    pub fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        use crate::src::core::{
            SkFlattenable::FlattenableOwner,
            SkRuntimeEffect::{ReadChildEffects, SkMakeCachedRuntimeEffect},
        };
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
            let mut source = Vec::new();
            buffer.readString(&mut source);
            effect = SkMakeCachedRuntimeEffect(SkRuntimeEffect::MakeForShader, source);
        }
        #[cfg(not(SK_BUILD_FOR_DEBUGGER))]
        if !buffer.validate(effect.is_some()) {
            return Ok(None);
        }
        let uniforms = buffer.readByteArrayAsData();
        let mut localMatrix = None;
        if buffer.isVersionLT(crate::src::core::SkPicturePriv::kNoShaderLocalMatrix) {
            let flags = buffer.read32() as u32;
            if flags & kHasLegacyLocalMatrix_Flag != 0 {
                let mut matrix = SkMatrix::default();
                buffer.readMatrix(&mut matrix);
                localMatrix = Some(matrix);
            }
        }
        let mut children = Vec::new();
        if !ReadChildEffects(buffer, effect.as_ref().map(|e| e.children()), &mut children)? {
            return Ok(None);
        }
        #[cfg(SK_BUILD_FOR_DEBUGGER)]
        if effect.is_none() {
            for (i, child) in children.iter().enumerate() {
                if let ChildPtr::Shader(shader) = child {
                    unsafe {
                        crate::SkDebugf!(@source b"src/shaders/SkRuntimeShader.cpp\0".as_ptr(),173;b"Serialized SkSL failed to compile. Replacing shader with child %d.\n\0".as_ptr(),i as core::ffi::c_int)
                    };
                    return Ok(Some(FlattenableOwner::Shader(shader.clone())));
                }
            }
            unsafe {
                crate::SkDebugf!(@source b"src/shaders/SkRuntimeShader.cpp\0".as_ptr(),180;b"Serialized SkSL failed to compile. Ignoring/dropping SkSL shader.\n\0".as_ptr())
            };
            return Ok(None);
        }
        Ok(effect
            .and_then(|effect| effect.makeShader(uniforms, &children, localMatrix.as_ref()))
            .map(FlattenableOwner::Shader))
    }
}
impl crate::src::core::SkFlattenable::SkFlattenableView for SkRuntimeShader {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn flatten(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.flatten_impl(buffer)?;
        Ok(())
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkRuntimeShader")
    }
    fn getFlattenableType(&self) -> crate::src::core::SkFlattenable::Type {
        crate::src::core::SkFlattenable::Type::kSkShader_Type
    }
}
