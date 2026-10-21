// Source: src/core/SkCPUContextImpl.h:15-16
// Source: src/core/SkCPUContextImpl.h:17-17
// C++ empty complete objects occupy one byte and have distinct addresses; Rust's byte preserves this native object identity.
pub struct ContextImpl {
    pub(super) native_empty_object: u8,
}
// Source: src/core/SkCPUContextImpl.h:17-17
impl Default for ContextImpl {
    fn default() -> Self {
        Self {
            native_empty_object: 0,
        }
    }
}
