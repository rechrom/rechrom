use super::{
    ir::SkSLSymbolTable::SymbolTable, SkSLBuiltinTypes::BuiltinTypes,
    SkSLErrorReporter::ErrorReporter, SkSLModule::Module, SkSLPool::Pool,
    SkSLProgramSettings::ProgramConfig,
};
use core::ptr::NonNull;
// Source: src/sksl/SkSLContext.h:24-24
// Source: src/sksl/SkSLContext.h:30-30
// Source: src/sksl/SkSLContext.h:33-33
// Source: src/sksl/SkSLContext.h:36-36
// Source: src/sksl/SkSLContext.h:44-44
// Source: src/sksl/SkSLContext.h:48-48
// Borrowed compiler state. Raw Type/Module/table/error leases follow the native
// compiler lifetime; Context owns none of these references.
#[repr(C)]
pub struct Context {
    pub fTypes: NonNull<BuiltinTypes>,
    pub fConfig: Option<NonNull<ProgramConfig>>,
    pub fErrors: NonNull<dyn ErrorReporter>,
    pub fModule: Option<NonNull<Module>>,
    pub fSymbolTable: Option<NonNull<SymbolTable>>,
}
impl Context {
    // Source: src/sksl/SkSLContext.h:38-41
    pub unsafe fn setErrorReporter(&mut self, e: NonNull<dyn ErrorReporter>) {
        self.fErrors = e;
    }
    // Source: src/sksl/SkSLContext.cpp:17-21
    /// Safety: types/errors remain live and retain exclusive mutation protocol for this context.
    pub unsafe fn new(types: &BuiltinTypes, errors: NonNull<dyn ErrorReporter>) -> Self {
        let value = Self {
            fTypes: NonNull::from(types),
            fConfig: None,
            fErrors: errors,
            fModule: None,
            fSymbolTable: None,
        };
        crate::SkASSERT!(@source b"src/sksl/SkSLContext.cpp\0".as_ptr(),20; @condition b"!Pool::IsAttached()\0".as_ptr(); !Pool::IsAttached());
        value
    }
}
// Source: src/sksl/SkSLContext.cpp:23-25
impl Drop for Context {
    fn drop(&mut self) {
        crate::SkASSERT!(@source b"src/sksl/SkSLContext.cpp\0".as_ptr(),24; @condition b"!Pool::IsAttached()\0".as_ptr(); !Pool::IsAttached());
    }
}
