use super::SkCPUContextImpl::ContextImpl;
use std::marker::PhantomData;
// Source: src/core/SkCPURecorderImpl.h:16-18
// Source: src/core/SkCPURecorderImpl.h:20-20
// Source: src/core/SkCPURecorderImpl.h:22-24
// Flattened native public Recorder base owns no in-scope fields. fCtx remains borrowed, never retained or cloned.
pub struct RecorderImpl<'ctx> {
    fCtx: *const ContextImpl,
    lifetime: PhantomData<&'ctx ContextImpl>,
}
impl<'ctx> RecorderImpl<'ctx> {
    pub fn new(ctx: &'ctx ContextImpl) -> Self {
        Self {
            fCtx: ctx,
            lifetime: PhantomData,
        }
    }
}
impl<'ctx> RecorderImpl<'ctx> {
    pub fn ctx(&self) -> *const ContextImpl {
        self.fCtx
    }
}
// Source: src/core/SkCPURecorderImpl.h:28-30
pub fn asRRI<'a, 'ctx>(rr: &'a super::SkCPURecorder::Recorder<'ctx>) -> &'a RecorderImpl<'ctx> {
    rr
}
