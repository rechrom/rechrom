// UNVERIFIED Compiler owner/context/diagnostic translation. All validation prohibited.
// Program/module owners and selected full optimizer preserve native lifetime/order; unverified.
use super::{
    ir::SkSLSymbolTable::SymbolTable,
    SkSLContext::Context,
    SkSLErrorReporter::{ErrorReporter, ErrorReporterData, ErrorReporterExt},
    SkSLModule::{Module, ModuleType},
    SkSLPool::Pool,
    SkSLPosition::Position,
    SkSLProgramKind::ProgramKind,
    SkSLProgramSettings::{ProgramConfig, ProgramSettings},
};
use core::{
    marker::PhantomPinned,
    mem::{ManuallyDrop, MaybeUninit},
    pin::Pin,
    ptr::NonNull,
};
use std::sync::Arc;
// UNVERIFIED: exact native builtin identifiers, not a compiler implementation.
// Source: src/sksl/SkSLCompiler.h:25-25
pub const SK_FRAGCOLOR_BUILTIN: i32 = 10001;
// Source: src/sksl/SkSLCompiler.h:26-26
pub const SK_LASTFRAGCOLOR_BUILTIN: i32 = 10008;
// Source: src/sksl/SkSLCompiler.h:27-27
pub const SK_SECONDARYFRAGCOLOR_BUILTIN: i32 = 10012;
// Source: src/sksl/SkSLCompiler.h:28-28
pub const SK_FRAGCOORD_BUILTIN: i32 = 15;
// Source: src/sksl/SkSLCompiler.h:29-29
pub const SK_CLOCKWISE_BUILTIN: i32 = 17;
// Source: src/sksl/SkSLCompiler.h:30-30
pub const SK_SAMPLEMASKIN_BUILTIN: i32 = 20;
// Source: src/sksl/SkSLCompiler.h:31-31
pub const SK_SAMPLEMASK_BUILTIN: i32 = 10020;
// Source: src/sksl/SkSLCompiler.h:33-33
pub const SK_VERTEXID_BUILTIN: i32 = 42;
// Source: src/sksl/SkSLCompiler.h:34-34
pub const SK_INSTANCEID_BUILTIN: i32 = 43;
// Source: src/sksl/SkSLCompiler.h:35-35
pub const SK_POSITION_BUILTIN: i32 = 0;
// Source: src/sksl/SkSLCompiler.h:36-36
pub const SK_POINTSIZE_BUILTIN: i32 = 1;
// Source: src/sksl/SkSLCompiler.h:38-38
pub const SK_NUMWORKGROUPS_BUILTIN: i32 = 24;
// Source: src/sksl/SkSLCompiler.h:39-39
pub const SK_WORKGROUPID_BUILTIN: i32 = 26;
// Source: src/sksl/SkSLCompiler.h:40-40
pub const SK_LOCALINVOCATIONID_BUILTIN: i32 = 27;
// Source: src/sksl/SkSLCompiler.h:41-41
pub const SK_GLOBALINVOCATIONID_BUILTIN: i32 = 28;
// Source: src/sksl/SkSLCompiler.h:42-42
pub const SK_LOCALINVOCATIONINDEX_BUILTIN: i32 = 29;
// Source: src/sksl/SkSLCompiler.h:66-66
// Source: src/sksl/SkSLCompiler.h:216-216
// Source: src/sksl/SkSLCompiler.h:217-217
// Source: src/sksl/SkSLCompiler.h:218-218
// Source: src/sksl/SkSLCompiler.h:219-219
// Source: src/sksl/SkSLCompiler.h:220-220
// Source: src/sksl/SkSLCompiler.h:222-222
// Source: src/sksl/SkSLCompiler.h:106-107
// No Clone/Copy implementation; private fields and pinned-only constructor retain
// the native deleted-copy and stable CompilerErrorReporter backpointer protocol.
// Actual field order, self-reference and reverse native owner destruction. Pinning is a
// language-only address lease; Program shares this same actual Context through Arc.
#[repr(C)]
pub struct Compiler {
    fErrorReporter: CompilerErrorReporter,
    fContext: ManuallyDrop<Arc<Context>>,
    fGlobalSymbols: ManuallyDrop<Option<Box<SymbolTable>>>,
    fConfig: ManuallyDrop<Option<Box<ProgramConfig>>>,
    fPool: ManuallyDrop<Option<Box<Pool>>>,
    fErrorText: ManuallyDrop<Vec<u8>>,
    pin: PhantomPinned,
}
// Source: src/sksl/SkSLCompiler.h:68-68
pub static FRAGCOLOR_NAME: [u8; 13] = *b"sk_FragColor\0";
// Source: src/sksl/SkSLCompiler.h:69-69
pub static RTADJUST_NAME: [u8; 12] = *b"sk_RTAdjust\0";
// Source: src/sksl/SkSLCompiler.h:70-70
pub static POSITION_NAME: [u8; 12] = *b"sk_Position\0";
// Source: src/sksl/SkSLCompiler.h:71-71
pub static POISON_TAG: [u8; 9] = *b"<POISON>\0";
// Unverified new translation; no compilation or numerical verification performed.
// Source: src/sksl/SkSLCompiler.h:79-90
pub fn GetRTAdjustVector(rtDims: crate::include::core::SkSize::SkISize, flipY: bool) -> [f32; 4] {
    let mut result = core::mem::MaybeUninit::<[f32; 4]>::uninit();
    unsafe {
        let p = result.as_mut_ptr().cast::<f32>();
        p.add(0).write(2.0f32 / rtDims.width() as f32);
        p.add(2).write(2.0f32 / rtDims.height() as f32);
        p.add(1).write(-1.0f32);
        p.add(3).write(-1.0f32);
        if flipY {
            p.add(2).write(-*p.add(2));
            p.add(3).write(-*p.add(3));
        }
        result.assume_init()
    }
}
// Source: src/sksl/SkSLCompiler.h:96-101
pub fn GetRTFlipVector(rtHeight: i32, flipY: bool) -> [f32; 2] {
    let mut result = core::mem::MaybeUninit::<[f32; 2]>::uninit();
    unsafe {
        let p = result.as_mut_ptr().cast::<f32>();
        p.add(0).write(if flipY { rtHeight as f32 } else { 0.0 });
        p.add(1).write(if flipY { -1.0 } else { 1.0 });
        result.assume_init()
    }
}
// Source: src/sksl/SkSLCompiler.h:113-117
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverrideFlag {
    kDefault,
    kOff,
    kOn,
}
// Source: src/sksl/SkSLCompiler.h:118-118
pub fn EnableOptimizer(flag: OverrideFlag) {
    sOptimizer.store(flag as i32, core::sync::atomic::Ordering::Relaxed);
}
// Source: src/sksl/SkSLCompiler.h:119-119
pub fn EnableInliner(flag: OverrideFlag) {
    sInliner.store(flag as i32, core::sync::atomic::Ordering::Relaxed);
}
impl Compiler {
    // Source: src/sksl/SkSLCompiler.h:129-129
    pub unsafe fn errorReporter(&mut self) -> &mut dyn ErrorReporter {
        (&mut *self.context_ptr()).fErrors.as_mut()
    }
    // Source: src/sksl/SkSLCompiler.h:131-131
    pub unsafe fn errorCount(&self) -> i32 {
        (*self.context_ptr()).fErrors.as_ref().errorCount()
    }
    // Source: src/sksl/SkSLCompiler.h:135-138
    pub unsafe fn resetErrors(&mut self) {
        self.fErrorText.clear();
        self.errorReporter().resetErrorCount();
    }
    // Source: src/sksl/SkSLCompiler.h:140-142
    pub fn context(&self) -> &Context {
        &self.fContext
    }
    // Source: src/sksl/SkSLCompiler.h:144-146
    pub fn globalSymbols(&self) -> Option<NonNull<SymbolTable>> {
        self.fGlobalSymbols.as_deref().map(NonNull::from)
    }
    // Source: src/sksl/SkSLCompiler.h:148-150
    pub fn symbolTable(&self) -> Option<NonNull<SymbolTable>> {
        self.fContext.fSymbolTable
    }
    // Language projection of native shared_ptr mutable state. Caller preserves the native
    // exclusive compilation/mutation lease across every Program sharing this Context.
    pub(crate) fn context_ptr(&self) -> *mut Context {
        Arc::as_ptr(&self.fContext) as *mut Context
    }
}
// Source: src/sksl/SkSLCompiler.h:168-168
// Source: src/sksl/SkSLCompiler.h:177-177
#[repr(C)]
struct CompilerErrorReporter {
    base: ErrorReporterData,
    fCompiler: NonNull<Compiler>,
}
// Source: src/sksl/SkSLCompiler.h:170-170
impl CompilerErrorReporter {
    fn new(compiler: NonNull<Compiler>) -> Self {
        Self {
            base: ErrorReporterData::new(),
            fCompiler: compiler,
        }
    }
}
impl ErrorReporter for CompilerErrorReporter {
    fn state(&self) -> &ErrorReporterData {
        &self.base
    }
    fn state_mut(&mut self) -> &mut ErrorReporterData {
        &mut self.base
    }
    // Source: src/sksl/SkSLCompiler.h:172-174
    fn handleError(&mut self, msg: &[u8], pos: Position) {
        unsafe {
            // Native re-entry reads current Context reporter, then writes only the disjoint text field.
            // Reborrow `self` when that reporter is this very callback, avoiding an overlapping &mut Compiler.
            let compiler = self.fCompiler.as_ptr();
            let context_owner = core::ptr::addr_of!((*compiler).fContext).cast::<Arc<Context>>();
            let context = Arc::as_ptr(&*context_owner);
            let current = core::ptr::addr_of!((*context).fErrors).read();
            let source = if current.as_ptr() as *mut () == self as *mut Self as *mut () {
                self.source().unwrap_or(&[])
            } else {
                current.as_ref().source().unwrap_or(&[])
            };
            handle_error_text(
                &mut *core::ptr::addr_of_mut!((*compiler).fErrorText),
                source,
                msg,
                pos,
            );
        }
    }
}
// Source: src/sksl/SkSLCompiler.h:224-224
// Source: src/sksl/SkSLCompiler.cpp:40-40
// Adapted atomic storage prevents a Rust data race; caller synchronization still determines native override observation.
static sOptimizer: core::sync::atomic::AtomicI32 =
    core::sync::atomic::AtomicI32::new(OverrideFlag::kDefault as i32);
// Source: src/sksl/SkSLCompiler.h:225-225
// Source: src/sksl/SkSLCompiler.cpp:41-41
static sInliner: core::sync::atomic::AtomicI32 =
    core::sync::atomic::AtomicI32::new(OverrideFlag::kDefault as i32);
// Source: src/sksl/SkSLCompiler.cpp:43-57
// Actual native RAII configuration swap; raw borrow retains the enclosing Context lease.
pub struct AutoProgramConfig {
    fContext: NonNull<Context>,
    fOldConfig: Option<NonNull<ProgramConfig>>,
}
impl AutoProgramConfig {
    pub unsafe fn new(
        mut context: NonNull<Context>,
        config: Option<NonNull<ProgramConfig>>,
    ) -> Self {
        let value = Self {
            fContext: context,
            fOldConfig: context.as_ref().fConfig,
        };
        context.as_mut().fConfig = config;
        value
    }
}
impl Drop for AutoProgramConfig {
    fn drop(&mut self) {
        unsafe {
            self.fContext.as_mut().fConfig = self.fOldConfig;
        }
    }
}
// Source: src/sksl/SkSLCompiler.cpp:59-61
impl Compiler {
    /// Safety: all shared Context/Type/module and pool-thread leases follow native Compiler lifetime.
    pub unsafe fn new() -> Pin<Box<Self>> {
        let p = Box::into_raw(Box::<Self>::new_uninit()).cast::<Self>();
        core::ptr::addr_of_mut!((*p).fErrorReporter)
            .write(CompilerErrorReporter::new(NonNull::new_unchecked(p)));
        let reporter = NonNull::from(
            &mut *core::ptr::addr_of_mut!((*p).fErrorReporter) as &mut dyn ErrorReporter
        );
        core::ptr::addr_of_mut!((*p).fContext).write(ManuallyDrop::new(Arc::new(Context::new(
            super::SkSLBuiltinTypes::BuiltinTypes::Get(),
            reporter,
        ))));
        core::ptr::addr_of_mut!((*p).fGlobalSymbols).write(ManuallyDrop::new(None));
        core::ptr::addr_of_mut!((*p).fConfig).write(ManuallyDrop::new(None));
        core::ptr::addr_of_mut!((*p).fPool).write(ManuallyDrop::new(None));
        core::ptr::addr_of_mut!((*p).fErrorText).write(ManuallyDrop::new(Vec::new()));
        core::ptr::addr_of_mut!((*p).pin).write(PhantomPinned);
        Pin::new_unchecked(Box::from_raw(p))
    }
}
// Source: src/sksl/SkSLCompiler.cpp:63-63
impl Drop for Compiler {
    fn drop(&mut self) {
        // Original field order remains h216-222. Native reverse teardown is also
        // honored if a Rust owner destructor unwinds, instead of leaking later owners.
        struct Owner<'a, T>(&'a mut ManuallyDrop<T>);
        impl<T> Drop for Owner<'_, T> {
            fn drop(&mut self) {
                unsafe {
                    ManuallyDrop::drop(self.0);
                }
            }
        }
        let owners = (
            Owner(&mut self.fErrorText),
            Owner(&mut self.fPool),
            Owner(&mut self.fConfig),
            Owner(&mut self.fGlobalSymbols),
            Owner(&mut self.fContext),
        );
        drop(owners);
    }
}
// Source: src/sksl/SkSLCompiler.cpp:65-87
impl Compiler {
    /// Safety: this is the original pinned Compiler under its native compilation lease.
    pub unsafe fn moduleForProgramKind(&mut self, kind: ProgramKind) -> NonNull<Module> {
        let mut loader = super::SkSLModuleLoader::ModuleLoader::Get();
        self.moduleForProgramKindWithLoader(kind, &mut loader)
    }
    pub unsafe fn moduleForProgramKindWithLoader(
        &mut self,
        kind: ProgramKind,
        loader: &mut super::SkSLModuleLoader::ModuleLoader,
    ) -> NonNull<Module> {
        let compiler = NonNull::from(self);
        match kind {
            ProgramKind::kFragment => loader.loadFragmentModule(compiler),
            ProgramKind::kVertex => loader.loadVertexModule(compiler),
            ProgramKind::kCompute => loader.loadComputeModule(compiler),
            ProgramKind::kGraphiteFragment => loader.loadGraphiteFragmentModule(compiler),
            ProgramKind::kGraphiteVertex => loader.loadGraphiteVertexModule(compiler),
            ProgramKind::kPrivateRuntimeBlender
            | ProgramKind::kPrivateRuntimeColorFilter
            | ProgramKind::kPrivateRuntimeShader => loader.loadPrivateRTShaderModule(compiler),
            ProgramKind::kRuntimeColorFilter
            | ProgramKind::kRuntimeShader
            | ProgramKind::kRuntimeBlender
            | ProgramKind::kMeshVertex
            | ProgramKind::kMeshFragment => loader.loadPublicModule(compiler),
        }
    }
}
// Source: src/sksl/SkSLCompiler.cpp:89-124
pub fn FinalizeSettings(
    settings: &mut super::SkSLProgramSettings::ProgramSettings,
    kind: super::SkSLProgramKind::ProgramKind,
) {
    use core::sync::atomic::Ordering;
    match sOptimizer.load(Ordering::Relaxed) {
        0 => {}
        1 => settings.fOptimize = false,
        2 => settings.fOptimize = true,
        _ => unreachable!(),
    }
    match sInliner.load(Ordering::Relaxed) {
        0 => {}
        1 => settings.fInlineThreshold = 0,
        2 => {
            if settings.fInlineThreshold == 0 {
                settings.fInlineThreshold = super::SkSLDefines::kDefaultInlineThreshold;
            }
        }
        _ => unreachable!(),
    }
    settings.fInlineThreshold *= settings.fOptimize as i32;
    settings.fRemoveDeadFunctions &= settings.fOptimize;
    settings.fRemoveDeadVariables &= settings.fOptimize;
    if super::SkSLProgramSettings::ProgramConfig::IsRuntimeEffect(kind) {
        settings.fAllowNarrowingConversions = true;
    }
}
impl Compiler {
    // Source: src/sksl/SkSLCompiler.cpp:126-162
    pub unsafe fn initializeContext(
        self: Pin<&mut Self>,
        module: NonNull<Module>,
        kind: ProgramKind,
        settings: ProgramSettings,
        source: Option<&[u8]>,
        moduleType: ModuleType,
    ) {
        let this = self.get_unchecked_mut();
        let ctx = this.context_ptr();
        debug_assert!(this.fPool.is_none());
        debug_assert!(this.fConfig.is_none());
        debug_assert!((*ctx).fSymbolTable.is_none());
        debug_assert!((*ctx).fConfig.is_none());
        debug_assert!((*ctx).fModule.is_none());
        this.resetErrors();
        *this.fConfig = Some(Box::new(ProgramConfig::new(moduleType, kind, settings)));
        FinalizeSettings(
            &mut this.fConfig.as_mut().unwrap_unchecked().fSettings,
            kind,
        );
        if settings.fUseMemoryPool {
            *this.fPool = Some(Pool::Create());
            this.fPool.as_ref().unwrap_unchecked().attachToThread();
        }
        (*ctx).fConfig = Some(NonNull::from(
            this.fConfig.as_mut().unwrap_unchecked().as_mut(),
        ));
        (*ctx).fModule = Some(module);
        (*ctx).fErrors.as_mut().setSource(source);
        let parent = NonNull::from(
            module
                .as_ref()
                .fSymbols
                .as_ref()
                .unwrap_unchecked()
                .as_ref(),
        );
        *this.fGlobalSymbols = Some(Box::new(SymbolTable::with_parent(
            Some(parent),
            moduleType != ModuleType::program,
        )));
        this.fGlobalSymbols
            .as_mut()
            .unwrap_unchecked()
            .markModuleBoundary();
        (*ctx).fSymbolTable = Some(NonNull::from(
            this.fGlobalSymbols.as_mut().unwrap_unchecked().as_mut(),
        ));
    }
    // Source: src/sksl/SkSLCompiler.cpp:164-178
    pub unsafe fn cleanupContext(self: Pin<&mut Self>) {
        let this = self.get_unchecked_mut();
        let ctx = this.context_ptr();
        (*ctx).fConfig = None;
        (*ctx).fModule = None;
        (*ctx).fErrors.as_mut().setSource(None);
        (*ctx).fSymbolTable = None;
        *this.fConfig = None;
        *this.fGlobalSymbols = None;
        if let Some(pool) = this.fPool.as_ref() {
            pool.detachFromThread();
            *this.fPool = None;
        }
    }
}
impl Compiler {
    // Parser's original friend field transfer; no second global symbol-table owner.
    pub(crate) fn takeGlobalSymbolsForModule(&mut self) -> Option<Box<SymbolTable>> {
        self.fGlobalSymbols.take()
    }
    // Source: src/sksl/SkSLCompiler.cpp:180-211
    // Source: src/sksl/SkSLCompiler.h:152-156
    pub unsafe fn compileModule(
        self: Pin<&mut Self>,
        kind: ProgramKind,
        moduleType: ModuleType,
        moduleSource: Vec<u8>,
        parent: NonNull<Module>,
        shouldInline: bool,
    ) -> Option<Box<Module>> {
        let mut compiler = NonNull::from(self.get_unchecked_mut());
        debug_assert!(compiler.as_ref().errorCount() == 0);
        let source = Box::new(moduleSource);
        let mut settings = ProgramSettings::default();
        settings.fUseMemoryPool = false;
        Pin::new_unchecked(compiler.as_mut()).initializeContext(
            parent,
            kind,
            settings,
            Some(&source),
            moduleType,
        );
        let mut parser = super::SkSLParser::Parser::new(compiler, &settings, kind, source);
        let mut module = parser.moduleInheritingFrom(parent);
        drop(parser);
        Pin::new_unchecked(compiler.as_mut()).cleanupContext();
        if compiler.as_ref().errorCount() != 0 {
            // Source: src/sksl/SkSLCompiler.cpp:202-204
            let mut module_name = super::SkSLModule::ModuleTypeToString(moduleType)
                .as_bytes()
                .to_vec();
            module_name.push(0);
            let mut errors = compiler.as_mut().errorText(true);
            errors.push(0);
            crate::SkDebugf!(@source b"src/sksl/SkSLCompiler.cpp\0".as_ptr(),202; b"Unexpected errors compiling %s:\n\n%s\n\0".as_ptr(),module_name.as_ptr(),errors.as_ptr());
            return None;
        }
        if shouldInline {
            compiler
                .as_mut()
                .optimizeModuleAfterLoading(kind, &mut module);
        }
        Some(module)
    }
    // Source: src/sksl/SkSLCompiler.cpp:213-231
    // Source: src/sksl/SkSLCompiler.h:121-123
    pub unsafe fn convertProgram(
        self: Pin<&mut Self>,
        kind: ProgramKind,
        programSource: Vec<u8>,
        settings: &ProgramSettings,
    ) -> Option<Box<super::ir::SkSLProgram::Program>> {
        let mut compiler = NonNull::from(self.get_unchecked_mut());
        let source = Box::new(programSource);
        let module = compiler.as_mut().moduleForProgramKind(kind);
        Pin::new_unchecked(compiler.as_mut()).initializeContext(
            module,
            kind,
            *settings,
            Some(&source),
            ModuleType::program,
        );
        let mut parser = super::SkSLParser::Parser::new(compiler, settings, kind, source);
        let program = parser.programInheritingFrom(module);
        drop(parser);
        Pin::new_unchecked(compiler.as_mut()).cleanupContext();
        program
    }
    // Source: src/sksl/SkSLCompiler.cpp:233-251
    // Source: src/sksl/SkSLCompiler.h:197-199
    pub unsafe fn releaseProgram(
        &mut self,
        source: Option<Box<Vec<u8>>>,
        elements: Vec<
            super::ir::SkSLIRNode::IRNodeBox<dyn super::ir::SkSLProgramElement::ProgramElement>,
        >,
    ) -> Option<Box<super::ir::SkSLProgram::Program>> {
        let pool = self.fPool.as_deref().map(NonNull::from);
        let mut result = Box::new(super::ir::SkSLProgram::Program::new(
            source,
            self.fConfig.take(),
            Arc::clone(&self.fContext),
            elements.into_iter().map(Some).collect(),
            self.fGlobalSymbols.take(),
            self.fPool.take(),
        ));
        (*self.context_ptr()).fSymbolTable = None;
        // The actual pool now belongs to result. Detach before its Program Drop
        // reattaches to destroy IR, on both ordinary failure and Rust unwind.
        // Declared after result so this guard runs before result during unwind.
        struct DetachPool(Option<NonNull<Pool>>);
        impl Drop for DetachPool {
            fn drop(&mut self) {
                if let Some(pool) = self.0 {
                    unsafe {
                        pool.as_ref().detachFromThread();
                    }
                }
            }
        }
        let detach = DetachPool(pool);
        let success = self.finalize(&mut result) && self.optimize(&mut result);
        drop(detach);
        if success {
            Some(result)
        } else {
            None
        }
    }
    // Source: src/sksl/SkSLCompiler.cpp:253-313
    // Source: src/sksl/SkSLCompiler.h:159-159
    pub unsafe fn optimizeModuleBeforeMinifying(
        &mut self,
        kind: ProgramKind,
        module: &mut Module,
        shrinkSymbols: bool,
    ) -> bool {
        crate::SkASSERT!(@source b"src/sksl/SkSLCompiler.cpp\0".as_ptr(),254; @condition b"this->errorCount() == 0\0".as_ptr(); self.errorCount()==0);
        let _m = super::SkSLModuleLoader::ModuleLoader::Get();
        let mut config = ProgramConfig::new(module.fModuleType, kind, ProgramSettings::default());
        let context = self.context_ptr();
        let _config = AutoProgramConfig::new(
            NonNull::new_unchecked(context),
            Some(NonNull::from(&mut config)),
        );
        let mut usage = super::analysis::SkSLProgramUsage::GetUsageForModule(module);
        if shrinkSymbols {
            super::transform::SkSLRenamePrivateSymbols::RenamePrivateSymbols(
                &*context, module, &mut usage, kind,
            );
            super::transform::SkSLReplaceConstVarsWithLiterals::ReplaceConstVarsWithLiterals(
                module, &mut usage,
            );
        }
        super::transform::SkSLEliminateUnreachableCode::EliminateUnreachableCodeForModule(
            module, &mut usage,
        );
        if kind == ProgramKind::kRuntimeShader {
            while super::transform::SkSLEliminateDeadFunctions::EliminateDeadFunctionsForModule(
                &*context, module, &mut usage,
            ) {}
        }
        while
            super::transform::SkSLEliminateDeadLocalVariables::EliminateDeadLocalVariablesForModule(
                &*context, module, &mut usage,
            )
        {}
        let onlyPrivateGlobals = !ProgramConfig::IsRuntimeEffect(kind);
        while super::transform::SkSLEliminateDeadGlobalVariables::EliminateDeadGlobalVariablesForModule(&*context,module,&mut usage,onlyPrivateGlobals) {}
        super::transform::SkSLEliminateEmptyStatements::EliminateEmptyStatements(module);
        super::transform::SkSLEliminateUnnecessaryBraces::EliminateUnnecessaryBraces(
            &*context, module,
        );
        super::transform::SkSLReplaceSplatCastsWithSwizzles::ReplaceSplatCastsWithSwizzles(
            &*context, module,
        );
        debug_assert!(*usage == *super::analysis::SkSLProgramUsage::GetUsageForModule(module));
        self.errorCount() == 0
    }
    // Source: src/sksl/SkSLCompiler.cpp:315-339
    // Source: src/sksl/SkSLCompiler.h:208-208
    pub unsafe fn optimizeModuleAfterLoading(
        &mut self,
        kind: ProgramKind,
        module: &mut Module,
    ) -> bool {
        crate::SkASSERT!(@source b"src/sksl/SkSLCompiler.cpp\0".as_ptr(),316; @condition b"this->errorCount() == 0\0".as_ptr(); self.errorCount()==0);
        #[cfg(not(SK_ENABLE_OPTIMIZE_SIZE))]
        {
            let mut config =
                ProgramConfig::new(module.fModuleType, kind, ProgramSettings::default());
            let _config = AutoProgramConfig::new(
                NonNull::new_unchecked(self.context_ptr()),
                Some(NonNull::from(&mut config)),
            );
            let mut usage = super::analysis::SkSLProgramUsage::GetUsageForModule(module);
            let mut inliner =
                super::SkSLInliner::Inliner::new(NonNull::new_unchecked(self.context_ptr()));
            while self.errorCount() == 0 {
                let symbols = NonNull::from(module.fSymbols.as_deref().unwrap_unchecked());
                if !self.runInlinerForElements(
                    &mut inliner,
                    &mut module.fElements,
                    symbols,
                    &mut usage,
                ) {
                    break;
                }
            }
            debug_assert!(*usage == *super::analysis::SkSLProgramUsage::GetUsageForModule(module));
        }
        self.errorCount() == 0
    }
    // Source: src/sksl/SkSLCompiler.cpp:341-377
    // Source: src/sksl/SkSLCompiler.h:202-202
    pub unsafe fn optimize(&mut self, program: &mut super::ir::SkSLProgram::Program) -> bool {
        if !program
            .fConfig
            .as_ref()
            .unwrap_unchecked()
            .fSettings
            .fOptimize
        {
            return true;
        }
        debug_assert!(self.errorCount() == 0);
        if self.errorCount() == 0 {
            #[cfg(not(SK_ENABLE_OPTIMIZE_SIZE))]
            {
                let mut inliner =
                    super::SkSLInliner::Inliner::new(NonNull::new_unchecked(self.context_ptr()));
                let symbols = NonNull::from(program.fSymbols.as_deref().unwrap_unchecked());
                self.runInlinerForElements(
                    &mut inliner,
                    &mut program.fOwnedElements,
                    symbols,
                    program.fUsage.as_mut().unwrap_unchecked(),
                );
            }
            super::transform::SkSLEliminateUnreachableCode::EliminateUnreachableCode(program);
            while super::transform::SkSLEliminateDeadFunctions::EliminateDeadFunctions(program) {}
            while super::transform::SkSLEliminateDeadLocalVariables::EliminateDeadLocalVariables(
                program,
            ) {}
            while super::transform::SkSLEliminateDeadGlobalVariables::EliminateDeadGlobalVariables(
                program,
            ) {}
            debug_assert!(
                program.usage().unwrap_unchecked()
                    == &*super::analysis::SkSLProgramUsage::GetUsage(program)
            );
            #[cfg(sk_debug)]
            super::analysis::SkSLCheckSymbolTableCorrectness::CheckSymbolTableCorrectness(program);
        }
        self.errorCount() == 0
    }
    // Source: src/sksl/SkSLCompiler.cpp:379-386
    // Source: src/sksl/SkSLCompiler.h:165-165
    pub unsafe fn runInliner(&mut self, program: &mut super::ir::SkSLProgram::Program) {
        #[cfg(not(SK_ENABLE_OPTIMIZE_SIZE))]
        {
            let config = NonNull::from(program.fConfig.as_deref().unwrap_unchecked());
            let _config =
                AutoProgramConfig::new(NonNull::new_unchecked(self.context_ptr()), Some(config));
            let mut inliner =
                super::SkSLInliner::Inliner::new(NonNull::new_unchecked(self.context_ptr()));
            let symbols = NonNull::from(program.fSymbols.as_deref().unwrap_unchecked());
            self.runInlinerForElements(
                &mut inliner,
                &mut program.fOwnedElements,
                symbols,
                program.fUsage.as_mut().unwrap_unchecked(),
            );
        }
    }
    // Source: src/sksl/SkSLCompiler.cpp:388-405
    // Source: src/sksl/SkSLCompiler.h:211-214
    pub unsafe fn runInlinerForElements(
        &mut self,
        inliner: &mut super::SkSLInliner::Inliner,
        elements: &mut Vec<
            Option<
                super::ir::SkSLIRNode::IRNodeBox<dyn super::ir::SkSLProgramElement::ProgramElement>,
            >,
        >,
        symbols: NonNull<SymbolTable>,
        usage: &mut super::analysis::SkSLProgramUsage::ProgramUsage,
    ) -> bool {
        // Source: src/sksl/SkSLCompiler.cpp:392-394
        #[cfg(SK_ENABLE_OPTIMIZE_SIZE)]
        {
            true
        }
        #[cfg(not(SK_ENABLE_OPTIMIZE_SIZE))]
        {
            let context = self.context_ptr();
            debug_assert!((*context).fSymbolTable.is_none());
            (*context).fSymbolTable = Some(symbols);
            // Keep the real table installed only for the native inliner interval.
            struct ClearSymbols(*mut Context);
            impl Drop for ClearSymbols {
                fn drop(&mut self) {
                    unsafe {
                        (*self.0).fSymbolTable = None;
                    }
                }
            }
            let clear = ClearSymbols(context);
            let result = inliner.analyze(elements, symbols, usage);
            drop(clear);
            result
        }
    }
    // Source: src/sksl/SkSLCompiler.cpp:407-439
    // Source: src/sksl/SkSLCompiler.h:205-205
    pub unsafe fn finalize(&mut self, program: &mut super::ir::SkSLProgram::Program) -> bool {
        super::transform::SkSLFindAndDeclareBuiltinFunctions::FindAndDeclareBuiltinFunctions(
            program,
        );
        super::transform::SkSLFindAndDeclareBuiltinVariables::FindAndDeclareBuiltinVariables(
            program,
        );
        super::transform::SkSLFindAndDeclareBuiltinStructs::FindAndDeclareBuiltinStructs(program);
        super::analysis::SkSLFinalizationChecks::DoFinalizationChecks(program);
        if (*self.context_ptr())
            .fConfig
            .unwrap_unchecked()
            .as_ref()
            .strictES2Mode()
            && self.errorCount() == 0
        {
            for pe in program.fOwnedElements.iter() {
                super::SkSLAnalysis::ValidateIndexingForES2(
                    &**pe.as_ref().unwrap_unchecked(),
                    self.errorReporter(),
                );
            }
        }
        if self.errorCount() == 0 {
            super::analysis::SkSLCheckProgramStructure::CheckProgramStructure(program);
            #[cfg(sk_debug)]
            super::analysis::SkSLCheckSymbolTableCorrectness::CheckSymbolTableCorrectness(program);
        }
        debug_assert!(
            program.usage().unwrap_unchecked()
                == &*super::analysis::SkSLProgramUsage::GetUsage(program)
        );
        self.errorCount() == 0
    }
}
// Source: src/sksl/SkSLCompiler.cpp:441-518
fn handle_error_text(text: &mut Vec<u8>, src: &[u8], msg: &[u8], pos: Position) {
    text.extend_from_slice(b"error: ");
    let mut printLocation = false;
    let mut line = -1;
    if pos.valid() {
        line = pos.line(Some(src));
        printLocation = pos.startOffset() < src.len() as i32;
        text.extend_from_slice(line.to_string().as_bytes());
        text.extend_from_slice(b": ");
    }
    text.extend_from_slice(msg);
    text.push(b'\n');
    if printLocation {
        let kMaxSurroundingChars = 100;
        let mut lineStart = pos.startOffset();
        while lineStart > 0 {
            if src[(lineStart - 1) as usize] == b'\n' {
                break;
            }
            lineStart -= 1;
        }
        let mut lineText = Vec::new();
        let mut caretText = Vec::new();
        if pos.startOffset() - lineStart > kMaxSurroundingChars {
            lineStart = pos.startOffset() - kMaxSurroundingChars;
            lineText.extend_from_slice(b"...");
            caretText.extend_from_slice(b"   ");
        }
        let mut lineSuffix: &[u8] = b"...\n";
        let mut lineStop = pos.endOffset() + kMaxSurroundingChars;
        if lineStop >= src.len() as i32 {
            lineStop = src.len() as i32 - 1;
            lineSuffix = b"\n";
        }
        for i in lineStart..lineStop {
            let c = src[i as usize];
            if c == b'\n' {
                lineSuffix = b"\n";
                break;
            }
            match c {
                b'\t' => lineText.extend_from_slice(b"    "),
                0 => lineText.push(b' '),
                _ => lineText.push(c),
            }
        }
        text.extend_from_slice(&lineText);
        text.extend_from_slice(lineSuffix);
        let mut i = lineStart;
        while i < src.len() as i32 {
            if i >= pos.endOffset() {
                break;
            }
            match src[i as usize] {
                b'\t' => caretText.extend_from_slice(if i >= pos.startOffset() {
                    b"^^^^"
                } else {
                    b"    "
                }),
                b'\n' => {
                    crate::SkASSERT!(@source b"src/sksl/SkSLCompiler.cpp\0".as_ptr(),506; @condition b"i >= pos.startOffset()\0".as_ptr(); i>=pos.startOffset());
                    caretText.extend_from_slice(if pos.endOffset() > i + 1 {
                        b"..."
                    } else {
                        b"^"
                    });
                    i = src.len() as i32;
                }
                _ => caretText.push(if i >= pos.startOffset() { b'^' } else { b' ' }),
            }
            i += 1;
        }
        text.extend_from_slice(&caretText);
        text.push(b'\n');
    }
}
impl Compiler {
    // Language delegation to the above native field-disjoint diagnostic body.
    pub unsafe fn handleError(&mut self, msg: &[u8], pos: Position) {
        let reporter = (*self.context_ptr()).fErrors;
        let source = reporter.as_ref().source().unwrap_or(&[]);
        handle_error_text(&mut self.fErrorText, source, msg, pos);
    }
    // Source: src/sksl/SkSLCompiler.cpp:520-527
    pub unsafe fn errorText(&mut self, showCount: bool) -> Vec<u8> {
        if showCount {
            self.writeErrorCount();
        }
        let result = self.fErrorText.to_vec();
        self.resetErrors();
        result
    }
    // Source: src/sksl/SkSLCompiler.cpp:529-535
    pub unsafe fn writeErrorCount(&mut self) {
        let count = self.errorCount();
        if count != 0 {
            self.fErrorText
                .extend_from_slice(count.to_string().as_bytes());
            self.fErrorText.extend_from_slice(if count == 1 {
                b" error\n"
            } else {
                b" errors\n"
            });
        }
    }
}
