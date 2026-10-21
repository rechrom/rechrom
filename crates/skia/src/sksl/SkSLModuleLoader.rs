// Pinned source; unverified. Global loader lifetime and lock ownership are explicit.
use core::{mem::ManuallyDrop, pin::Pin, ptr::NonNull};
use std::sync::{Mutex, MutexGuard, OnceLock};
// Source: src/sksl/SkSLModuleLoader.h:16-18
use super::{
    ir::{
        SkSLIRNode::ProgramElementKind, SkSLSymbol::Symbol, SkSLSymbolTable::SymbolTable,
        SkSLType::Type,
    },
    SkSLBuiltinTypes::BuiltinTypes,
    SkSLCompiler::Compiler,
    SkSLModule::{Module, ModuleType},
    SkSLPosition::Position,
    SkSLProgramKind::ProgramKind,
};
// Source: src/sksl/SkSLModuleLoader.h:20-20
pub type BuiltinTypePtr = for<'a> fn(&'a BuiltinTypes) -> &'a dyn Type;
// Source: src/sksl/SkSLModuleLoader.cpp:32-36
// Source: src/sksl/SkSLModuleLoader.cpp:38-74
// Computational TYPE/MODULE_DATA macros become genuine typed field projections
// and direct GetModuleData calls in the corresponding native load branches.
const kRootTypes: &[BuiltinTypePtr] = &[
    |types| &**types.fVoid,
    |types| &**types.fFloat,
    |types| &**types.fFloat2,
    |types| &**types.fFloat3,
    |types| &**types.fFloat4,
    |types| &**types.fHalf,
    |types| &**types.fHalf2,
    |types| &**types.fHalf3,
    |types| &**types.fHalf4,
    |types| &**types.fInt,
    |types| &**types.fInt2,
    |types| &**types.fInt3,
    |types| &**types.fInt4,
    |types| &**types.fUInt,
    |types| &**types.fUInt2,
    |types| &**types.fUInt3,
    |types| &**types.fUInt4,
    |types| &**types.fShort,
    |types| &**types.fShort2,
    |types| &**types.fShort3,
    |types| &**types.fShort4,
    |types| &**types.fUShort,
    |types| &**types.fUShort2,
    |types| &**types.fUShort3,
    |types| &**types.fUShort4,
    |types| &**types.fBool,
    |types| &**types.fBool2,
    |types| &**types.fBool3,
    |types| &**types.fBool4,
    |types| &**types.fFloat2x2,
    |types| &**types.fFloat2x3,
    |types| &**types.fFloat2x4,
    |types| &**types.fFloat3x2,
    |types| &**types.fFloat3x3,
    |types| &**types.fFloat3x4,
    |types| &**types.fFloat4x2,
    |types| &**types.fFloat4x3,
    |types| &**types.fFloat4x4,
    |types| &**types.fHalf2x2,
    |types| &**types.fHalf2x3,
    |types| &**types.fHalf2x4,
    |types| &**types.fHalf3x2,
    |types| &**types.fHalf3x3,
    |types| &**types.fHalf3x4,
    |types| &**types.fHalf4x2,
    |types| &**types.fHalf4x3,
    |types| &**types.fHalf4x4,
    |types| &**types.fSquareMat,
    |types| &**types.fSquareHMat,
    |types| &**types.fMat,
    |types| &**types.fHMat,
    |types| &**types.fGenType,
    |types| &**types.fGenIType,
    |types| &**types.fGenUType,
    |types| &**types.fGenHType,
    |types| &**types.fGenBType,
    |types| &**types.fIntLiteral,
    |types| &**types.fFloatLiteral,
    |types| &**types.fVec,
    |types| &**types.fIVec,
    |types| &**types.fUVec,
    |types| &**types.fHVec,
    |types| &**types.fSVec,
    |types| &**types.fUSVec,
    |types| &**types.fBVec,
    |types| &**types.fColorFilter,
    |types| &**types.fShader,
    |types| &**types.fBlender,
];
// Source: src/sksl/SkSLModuleLoader.cpp:76-87
const kPrivateTypes: &[BuiltinTypePtr] = &[
    |types| &**types.fSampler2D,
    |types| &**types.fSamplerExternalOES,
    |types| &**types.fSampler2DRect,
    |types| &**types.fSubpassInput,
    |types| &**types.fSubpassInputMS,
    |types| &**types.fSampler,
    |types| &**types.fTexture2D_sample,
    |types| &**types.fTexture2D,
    |types| &**types.fReadOnlyTexture2D,
    |types| &**types.fWriteOnlyTexture2D,
    |types| &**types.fGenTexture2D,
    |types| &**types.fReadableTexture2D,
    |types| &**types.fWritableTexture2D,
    |types| &**types.fAtomicUInt,
    |types| &**types.fAtomic_uint,
];
// Source: src/sksl/SkSLModuleLoader.cpp:91-114
// Adapted private Rust field order maps native implicit member destruction.
// Descendant modules are destroyed before their borrowed parent modules.
struct Impl {
    // Source: src/sksl/SkSLModuleLoader.cpp:113-113
    fRuntimeShaderModule: Option<Box<Module>>,
    // Source: src/sksl/SkSLModuleLoader.cpp:111-112
    fPublicModule: Option<Box<Module>>,
    // Source: src/sksl/SkSLModuleLoader.cpp:109-109
    fGraphiteFragmentModule: Option<Box<Module>>,
    // Source: src/sksl/SkSLModuleLoader.cpp:108-108
    fGraphiteVertexModule: Option<Box<Module>>,
    // Source: src/sksl/SkSLModuleLoader.cpp:107-107
    fComputeModule: Option<Box<Module>>,
    // Source: src/sksl/SkSLModuleLoader.cpp:106-106
    fFragmentModule: Option<Box<Module>>,
    // Source: src/sksl/SkSLModuleLoader.cpp:105-105
    fVertexModule: Option<Box<Module>>,
    // Source: src/sksl/SkSLModuleLoader.cpp:103-104
    fGPUModule: Option<Box<Module>>,
    // Source: src/sksl/SkSLModuleLoader.cpp:102-102
    fSharedModule: Option<Box<Module>>,
    // Source: src/sksl/SkSLModuleLoader.cpp:100-100
    fRootModule: Option<Box<Module>>,
}
// Actual native cache can cross threads only behind its original loader mutex.
// Published pointers are immutable borrowed module graphs, not independently mutable owners.
unsafe impl Send for Impl {}
// Source: src/sksl/SkSLModuleLoader.h:26-30
// guard must be released before the locally-owned mutex/cache is destroyed.
pub struct ModuleLoader {
    guard: ManuallyDrop<MutexGuard<'static, Impl>>,
    fLocalImpl: Option<Box<Mutex<Impl>>>,
}
impl ModuleLoader {
    // Source: src/sksl/SkSLModuleLoader.cpp:116-119
    // Source: src/sksl/SkSLModuleLoader.h:39-39
    /// Safety: initial root IR allocation must have the native process-lifetime
    /// allocation lease; later operations retain the returned guard until done.
    pub unsafe fn Get() -> Self {
        static INSTANCE: OnceLock<&'static Mutex<Impl>> = OnceLock::new();
        let mutex =
            *INSTANCE.get_or_init(|| Box::leak(Box::new(Mutex::new(unsafe { Impl::new() }))));
        Self {
            guard: ManuallyDrop::new(mutex.lock().unwrap_or_else(|e| e.into_inner())),
            fLocalImpl: None,
        }
    }
    // Source: src/sksl/SkSLModuleLoader.cpp:121-125
    // Source: src/sksl/SkSLModuleLoader.h:33-33
    /// Safety: root IR allocation and returned module borrows retain their native
    /// allocation-pool leases through this local loader's destruction.
    pub unsafe fn new() -> Self {
        let owner = Box::new(Mutex::new(unsafe { Impl::new() }));
        // Box storage is stable. Private ownership plus Drop below bounds this
        // erased guard lifetime to the original mutex allocation, even after moves.
        let mutex: &'static Mutex<Impl> = unsafe { &*(&*owner as *const Mutex<Impl>) };
        Self {
            guard: ManuallyDrop::new(mutex.lock().unwrap_or_else(|e| e.into_inner())),
            fLocalImpl: Some(owner),
        }
    }
    // Source: src/sksl/SkSLModuleLoader.cpp:127-131
    // Source: src/sksl/SkSLModuleLoader.h:34-34
    // Singleton construction's same mutex-acquire operation is folded into Get;
    // private Impl cannot be passed by an unrelated owner.
}
// Source: src/sksl/SkSLModuleLoader.cpp:133-135
// Source: src/sksl/SkSLModuleLoader.h:35-35
impl Drop for ModuleLoader {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.guard);
        }
        drop(self.fLocalImpl.take());
    }
}
// Source: src/sksl/SkSLModuleLoader.cpp:137-139
impl Impl {
    unsafe fn new() -> Self {
        let mut this = Self {
            fRootModule: None,
            fSharedModule: None,
            fGPUModule: None,
            fVertexModule: None,
            fFragmentModule: None,
            fComputeModule: None,
            fGraphiteVertexModule: None,
            fGraphiteFragmentModule: None,
            fPublicModule: None,
            fRuntimeShaderModule: None,
        };
        this.makeRootSymbolTable();
        this
    }
}
// Source: src/sksl/SkSLModuleLoader.cpp:141-183
unsafe fn compile_and_shrink(
    mut compiler: NonNull<Compiler>,
    kind: ProgramKind,
    moduleType: ModuleType,
    source: Vec<u8>,
    parent: NonNull<Module>,
) -> Box<Module> {
    let Some(mut module) =
        Pin::new_unchecked(compiler.as_mut()).compileModule(kind, moduleType, source, parent, true)
    else {
        eprintln!(
            "Unable to load module {}",
            super::SkSLModule::ModuleTypeToString(moduleType)
        );
        std::process::abort();
    };
    module.fElements.retain(|element| {
        match element
            .as_ref()
            .expect("native nonnull module element")
            .kind()
        {
            ProgramElementKind::kFunction
            | ProgramElementKind::kGlobalVar
            | ProgramElementKind::kInterfaceBlock
            | ProgramElementKind::kStructDefinition => true,
            ProgramElementKind::kFunctionPrototype => false,
            _ => {
                debug_assert!(
                    false,
                    "Unsupported element: {}",
                    String::from_utf8_lossy(&element.as_ref().unwrap().description())
                );
                true
            }
        }
    });
    module.fElements.shrink_to_fit();
    module
}
impl ModuleLoader {
    // Source: src/sksl/SkSLModuleLoader.cpp:185-187
    /// Returned pointer is borrowed from this actual cache. Local loader pointers
    /// cannot outlive the loader; Get's process-lifetime cache remains live after unlock.
    // Source: src/sksl/SkSLModuleLoader.h:42-42
    pub fn rootModule(&self) -> NonNull<Module> {
        NonNull::from(
            self.guard
                .fRootModule
                .as_deref()
                .expect("actual root module"),
        )
    }
    // Source: src/sksl/SkSLModuleLoader.cpp:189-229
    // Source: src/sksl/SkSLModuleLoader.h:59-59
    pub unsafe fn addPublicTypeAliases(&mut self, module: NonNull<Module>) {
        let types = BuiltinTypes::Get();
        let mut symbols = NonNull::from(module.as_ref().fSymbols.as_deref().unwrap_unchecked());
        symbols
            .as_mut()
            .addWithoutOwnershipOrDie(NonNull::from(&**types.fVec2 as &dyn Symbol));
        symbols
            .as_mut()
            .addWithoutOwnershipOrDie(NonNull::from(&**types.fVec3 as &dyn Symbol));
        symbols
            .as_mut()
            .addWithoutOwnershipOrDie(NonNull::from(&**types.fVec4 as &dyn Symbol));
        symbols
            .as_mut()
            .addWithoutOwnershipOrDie(NonNull::from(&**types.fIVec2 as &dyn Symbol));
        symbols
            .as_mut()
            .addWithoutOwnershipOrDie(NonNull::from(&**types.fIVec3 as &dyn Symbol));
        symbols
            .as_mut()
            .addWithoutOwnershipOrDie(NonNull::from(&**types.fIVec4 as &dyn Symbol));
        symbols
            .as_mut()
            .addWithoutOwnershipOrDie(NonNull::from(&**types.fUVec2 as &dyn Symbol));
        symbols
            .as_mut()
            .addWithoutOwnershipOrDie(NonNull::from(&**types.fUVec3 as &dyn Symbol));
        symbols
            .as_mut()
            .addWithoutOwnershipOrDie(NonNull::from(&**types.fUVec4 as &dyn Symbol));
        symbols
            .as_mut()
            .addWithoutOwnershipOrDie(NonNull::from(&**types.fBVec2 as &dyn Symbol));
        symbols
            .as_mut()
            .addWithoutOwnershipOrDie(NonNull::from(&**types.fBVec3 as &dyn Symbol));
        symbols
            .as_mut()
            .addWithoutOwnershipOrDie(NonNull::from(&**types.fBVec4 as &dyn Symbol));
        symbols
            .as_mut()
            .addWithoutOwnershipOrDie(NonNull::from(&**types.fMat2 as &dyn Symbol));
        symbols
            .as_mut()
            .addWithoutOwnershipOrDie(NonNull::from(&**types.fMat3 as &dyn Symbol));
        symbols
            .as_mut()
            .addWithoutOwnershipOrDie(NonNull::from(&**types.fMat4 as &dyn Symbol));
        symbols
            .as_mut()
            .addWithoutOwnershipOrDie(NonNull::from(&**types.fMat2x2 as &dyn Symbol));
        symbols
            .as_mut()
            .addWithoutOwnershipOrDie(NonNull::from(&**types.fMat2x3 as &dyn Symbol));
        symbols
            .as_mut()
            .addWithoutOwnershipOrDie(NonNull::from(&**types.fMat2x4 as &dyn Symbol));
        symbols
            .as_mut()
            .addWithoutOwnershipOrDie(NonNull::from(&**types.fMat3x2 as &dyn Symbol));
        symbols
            .as_mut()
            .addWithoutOwnershipOrDie(NonNull::from(&**types.fMat3x3 as &dyn Symbol));
        symbols
            .as_mut()
            .addWithoutOwnershipOrDie(NonNull::from(&**types.fMat3x4 as &dyn Symbol));
        symbols
            .as_mut()
            .addWithoutOwnershipOrDie(NonNull::from(&**types.fMat4x2 as &dyn Symbol));
        symbols
            .as_mut()
            .addWithoutOwnershipOrDie(NonNull::from(&**types.fMat4x3 as &dyn Symbol));
        symbols
            .as_mut()
            .addWithoutOwnershipOrDie(NonNull::from(&**types.fMat4x4 as &dyn Symbol));
        for private in kPrivateTypes {
            let alias = <dyn Type>::MakeAliasType(private(types).name(), &**types.fInvalid);
            symbols
                .as_mut()
                .inject(alias.cast(|p| p as *mut dyn Symbol));
        }
    }
    // Source: src/sksl/SkSLModuleLoader.cpp:231-241
    // Source: src/sksl/SkSLModuleLoader.h:54-54
    pub unsafe fn loadPublicModule(&mut self, compiler: NonNull<Compiler>) -> NonNull<Module> {
        if self.guard.fPublicModule.is_none() {
            let parent = self.loadSharedModule(compiler);
            self.guard.fPublicModule = Some(compile_and_shrink(
                compiler,
                ProgramKind::kFragment,
                ModuleType::sksl_public,
                super::SkSLModuleDataDefault::GetModuleData(
                    ModuleType::sksl_public,
                    "sksl_public.sksl",
                ),
                parent,
            ));
            let module = NonNull::from(self.guard.fPublicModule.as_deref().unwrap_unchecked());
            self.addPublicTypeAliases(module);
        }
        NonNull::from(self.guard.fPublicModule.as_deref().unwrap_unchecked())
    }
    // Source: src/sksl/SkSLModuleLoader.cpp:243-252
    // Source: src/sksl/SkSLModuleLoader.h:55-55
    pub unsafe fn loadPrivateRTShaderModule(
        &mut self,
        compiler: NonNull<Compiler>,
    ) -> NonNull<Module> {
        if self.guard.fRuntimeShaderModule.is_none() {
            let parent = self.loadPublicModule(compiler);
            self.guard.fRuntimeShaderModule = Some(compile_and_shrink(
                compiler,
                ProgramKind::kFragment,
                ModuleType::sksl_rt_shader,
                super::SkSLModuleDataDefault::GetModuleData(
                    ModuleType::sksl_rt_shader,
                    "sksl_rt_shader.sksl",
                ),
                parent,
            ));
        }
        NonNull::from(
            self.guard
                .fRuntimeShaderModule
                .as_deref()
                .unwrap_unchecked(),
        )
    }
    // Source: src/sksl/SkSLModuleLoader.cpp:254-263
    // Source: src/sksl/SkSLModuleLoader.h:46-46
    pub unsafe fn loadSharedModule(&mut self, compiler: NonNull<Compiler>) -> NonNull<Module> {
        if self.guard.fSharedModule.is_none() {
            let parent = self.rootModule();
            self.guard.fSharedModule = Some(compile_and_shrink(
                compiler,
                ProgramKind::kFragment,
                ModuleType::sksl_shared,
                super::SkSLModuleDataDefault::GetModuleData(
                    ModuleType::sksl_shared,
                    "sksl_shared.sksl",
                ),
                parent,
            ));
        }
        NonNull::from(self.guard.fSharedModule.as_deref().unwrap_unchecked())
    }
    // Source: src/sksl/SkSLModuleLoader.cpp:265-274
    // Source: src/sksl/SkSLModuleLoader.h:47-47
    pub unsafe fn loadGPUModule(&mut self, compiler: NonNull<Compiler>) -> NonNull<Module> {
        if self.guard.fGPUModule.is_none() {
            let parent = self.loadSharedModule(compiler);
            self.guard.fGPUModule = Some(compile_and_shrink(
                compiler,
                ProgramKind::kFragment,
                ModuleType::sksl_gpu,
                super::SkSLModuleDataDefault::GetModuleData(ModuleType::sksl_gpu, "sksl_gpu.sksl"),
                parent,
            ));
        }
        NonNull::from(self.guard.fGPUModule.as_deref().unwrap_unchecked())
    }
    // Source: src/sksl/SkSLModuleLoader.cpp:276-285
    // Source: src/sksl/SkSLModuleLoader.h:49-49
    pub unsafe fn loadFragmentModule(&mut self, compiler: NonNull<Compiler>) -> NonNull<Module> {
        if self.guard.fFragmentModule.is_none() {
            let parent = self.loadGPUModule(compiler);
            self.guard.fFragmentModule = Some(compile_and_shrink(
                compiler,
                ProgramKind::kFragment,
                ModuleType::sksl_frag,
                super::SkSLModuleDataDefault::GetModuleData(
                    ModuleType::sksl_frag,
                    "sksl_frag.sksl",
                ),
                parent,
            ));
        }
        NonNull::from(self.guard.fFragmentModule.as_deref().unwrap_unchecked())
    }
    // Source: src/sksl/SkSLModuleLoader.cpp:287-296
    // Source: src/sksl/SkSLModuleLoader.h:48-48
    pub unsafe fn loadVertexModule(&mut self, compiler: NonNull<Compiler>) -> NonNull<Module> {
        if self.guard.fVertexModule.is_none() {
            let parent = self.loadGPUModule(compiler);
            self.guard.fVertexModule = Some(compile_and_shrink(
                compiler,
                ProgramKind::kVertex,
                ModuleType::sksl_vert,
                super::SkSLModuleDataDefault::GetModuleData(
                    ModuleType::sksl_vert,
                    "sksl_vert.sksl",
                ),
                parent,
            ));
        }
        NonNull::from(self.guard.fVertexModule.as_deref().unwrap_unchecked())
    }
    // Source: src/sksl/SkSLModuleLoader.cpp:298-307
    // Source: src/sksl/SkSLModuleLoader.h:50-50
    pub unsafe fn loadComputeModule(&mut self, compiler: NonNull<Compiler>) -> NonNull<Module> {
        if self.guard.fComputeModule.is_none() {
            let parent = self.loadGPUModule(compiler);
            self.guard.fComputeModule = Some(compile_and_shrink(
                compiler,
                ProgramKind::kCompute,
                ModuleType::sksl_compute,
                super::SkSLModuleDataDefault::GetModuleData(
                    ModuleType::sksl_compute,
                    "sksl_compute.sksl",
                ),
                parent,
            ));
        }
        NonNull::from(self.guard.fComputeModule.as_deref().unwrap_unchecked())
    }
    // Source: src/sksl/SkSLModuleLoader.cpp:309-318
    // Source: src/sksl/SkSLModuleLoader.h:52-52
    pub unsafe fn loadGraphiteFragmentModule(
        &mut self,
        compiler: NonNull<Compiler>,
    ) -> NonNull<Module> {
        if self.guard.fGraphiteFragmentModule.is_none() {
            let parent = self.loadFragmentModule(compiler);
            self.guard.fGraphiteFragmentModule = Some(compile_and_shrink(
                compiler,
                ProgramKind::kGraphiteFragment,
                ModuleType::sksl_graphite_frag,
                super::SkSLModuleDataDefault::GetModuleData(
                    ModuleType::sksl_graphite_frag,
                    "sksl_graphite_frag.sksl",
                ),
                parent,
            ));
        }
        NonNull::from(
            self.guard
                .fGraphiteFragmentModule
                .as_deref()
                .unwrap_unchecked(),
        )
    }
    // Source: src/sksl/SkSLModuleLoader.cpp:320-329
    // Source: src/sksl/SkSLModuleLoader.h:51-51
    pub unsafe fn loadGraphiteVertexModule(
        &mut self,
        compiler: NonNull<Compiler>,
    ) -> NonNull<Module> {
        if self.guard.fGraphiteVertexModule.is_none() {
            let parent = self.loadVertexModule(compiler);
            self.guard.fGraphiteVertexModule = Some(compile_and_shrink(
                compiler,
                ProgramKind::kGraphiteVertex,
                ModuleType::sksl_graphite_vert,
                super::SkSLModuleDataDefault::GetModuleData(
                    ModuleType::sksl_graphite_vert,
                    "sksl_graphite_vert.sksl",
                ),
                parent,
            ));
        }
        NonNull::from(
            self.guard
                .fGraphiteVertexModule
                .as_deref()
                .unwrap_unchecked(),
        )
    }
}
// Source: src/sksl/SkSLModuleLoader.cpp:331-356
impl Impl {
    unsafe fn makeRootSymbolTable(&mut self) {
        let mut root = Box::new(Module::default());
        *root.fSymbols = Some(Box::new(SymbolTable::new(true)));
        let types = BuiltinTypes::Get();
        let symbols = root.fSymbols.as_mut().unwrap_unchecked();
        for field in kRootTypes {
            symbols.addWithoutOwnershipOrDie(NonNull::from(field(types) as &dyn Symbol));
        }
        for field in kPrivateTypes {
            symbols.addWithoutOwnershipOrDie(NonNull::from(field(types) as &dyn Symbol));
        }
        let caps = <dyn super::ir::SkSLVariable::Variable>::Make(
            Position::default(),
            Position::default(),
            &super::ir::SkSLLayout::Layout::default(),
            super::ir::SkSLModifierFlags::ModifierFlags::default(),
            NonNull::from(&**types.fSkCaps),
            b"sk_Caps",
            Vec::new(),
            false,
            super::ir::SkSLVariable::Storage::kGlobal,
        );
        symbols.addOrDie(caps.cast(|p| p as *mut dyn Symbol));
        self.fRootModule = Some(root);
    }
}
// Native Impl member teardown retains parent modules until every child is gone.
// Adapted ownership of src/sksl/SkSLModuleLoader.cpp:91-114; no added algorithm credit.
impl Drop for Impl {
    fn drop(&mut self) {
        drop(self.fRuntimeShaderModule.take());
        drop(self.fPublicModule.take());
        drop(self.fGraphiteFragmentModule.take());
        drop(self.fGraphiteVertexModule.take());
        drop(self.fComputeModule.take());
        drop(self.fFragmentModule.take());
        drop(self.fVertexModule.take());
        drop(self.fGPUModule.take());
        drop(self.fSharedModule.take());
        drop(self.fRootModule.take());
    }
}
