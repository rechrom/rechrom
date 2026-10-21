use super::SkCPUContextImpl::ContextImpl;
use super::SkCPURecorderImpl::RecorderImpl;
use std::sync::OnceLock;
// Source: include/core/SkCPUContext.h:17-19
// Native empty base/sole final subclass share one concrete Rust owner, preserving actual factory and lifetime semantics.
pub type Context = ContextImpl;
pub struct Options;
// Source: src/core/SkCPUContext.cpp:15-17
impl ContextImpl {
    pub fn MakeWithOptions(_opts: &Options) -> Box<Self> {
        Box::new(Self::default())
    }
}
impl ContextImpl {
    // Source: src/core/SkCPUContext.cpp:19-21
    pub fn Make() -> Box<Self> {
        Self::MakeWithOptions(&Options)
    }
}
impl ContextImpl {
    // Source: src/core/SkCPUContext.cpp:23-25
    pub fn makeRecorder(&self) -> Box<RecorderImpl<'_>> {
        Box::new(RecorderImpl::new(self))
    }
}
impl ContextImpl {
    // Source: src/core/SkCPUContext.cpp:27-30
    pub fn TODO() -> &'static Self {
        static CONTEXT: OnceLock<&'static ContextImpl> = OnceLock::new();
        CONTEXT.get_or_init(|| Box::leak(Self::Make()))
    }
}
