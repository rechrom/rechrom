// Source: src/sksl/SkSLModule.h:29-37
impl ModuleType {
    pub const sksl_shared: Self = Self(2);
    pub const sksl_compute: Self = Self(3);
    pub const sksl_frag: Self = Self(4);
    pub const sksl_gpu: Self = Self(5);
    pub const sksl_public: Self = Self(6);
    pub const sksl_rt_shader: Self = Self(7);
    pub const sksl_vert: Self = Self(8);
    pub const sksl_graphite_frag: Self = Self(9);
    pub const sksl_graphite_vert: Self = Self(10);
}
// Source: src/sksl/SkSLModule.h:39-47
// Adapted enum: transparent signed8 retains native cast identities including
// unknown values handled by the actual ModuleTypeToString default branch.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleType(pub i8);
impl ModuleType {
    pub const program: Self = Self(0);
    pub const unknown: Self = Self(1);
}
// Source: src/sksl/SkSLModule.h:49-54
#[repr(C)]
pub struct Module {
    pub fParent: Option<core::ptr::NonNull<Module>>,
    pub fSymbols: core::mem::ManuallyDrop<Option<Box<super::ir::SkSLSymbolTable::SymbolTable>>>,
    pub fElements: core::mem::ManuallyDrop<
        Vec<
            Option<
                super::ir::SkSLIRNode::IRNodeBox<dyn super::ir::SkSLProgramElement::ProgramElement>,
            >,
        >,
    >,
    pub fModuleType: ModuleType,
}
impl Default for Module {
    fn default() -> Self {
        Self {
            fParent: None,
            fSymbols: core::mem::ManuallyDrop::new(None),
            fElements: core::mem::ManuallyDrop::new(Vec::new()),
            fModuleType: ModuleType::unknown,
        }
    }
}
impl Drop for Module {
    fn drop(&mut self) {
        unsafe {
            while let Some(e) = self.fElements.pop() {
                drop(e);
            }
            core::mem::ManuallyDrop::drop(&mut self.fElements);
            core::mem::ManuallyDrop::drop(&mut self.fSymbols);
        }
    }
}

// Source: src/sksl/SkSLModule.cpp:12-19
pub fn ModuleTypeToString(module_type: ModuleType) -> &'static str {
    match module_type {
        ModuleType::sksl_shared => "sksl_shared",
        ModuleType::sksl_compute => "sksl_compute",
        ModuleType::sksl_frag => "sksl_frag",
        ModuleType::sksl_gpu => "sksl_gpu",
        ModuleType::sksl_public => "sksl_public",
        ModuleType::sksl_rt_shader => "sksl_rt_shader",
        ModuleType::sksl_vert => "sksl_vert",
        ModuleType::sksl_graphite_frag => "sksl_graphite_frag",
        ModuleType::sksl_graphite_vert => "sksl_graphite_vert",
        _ => "unknown",
    }
}

// Source: src/sksl/SkSLModule.h:59-59
// Preserve the native SkSL namespace declaration; the real definition remains
// in its existing implementation-file module, with no duplicate data owner.
pub use super::SkSLModuleDataDefault::GetModuleData;
