// UNVERIFIED: validation prohibited by user.
use core::ffi::c_char;
// Source: src/sksl/SkSLGraphiteModules.h:12-15
// Borrowed native C strings. No additional owner or reference count is created.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GraphiteModules {
    pub fFragmentShader: *const c_char,
    pub fVertexShader: *const c_char,
}
// Source: src/sksl/SkSLGraphiteModules.h:21-21
// PENDING: LoadGraphiteModules is implemented by a separate Graphite-specific
// native source outside this CPU module-data batch; no empty implementation.
// Source: src/sksl/SkSLGraphiteModules.h:22-22
pub use super::SkSLModuleDataDefault::SetGraphiteModuleData;
