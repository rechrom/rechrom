use super::SkCPUContextImpl::ContextImpl;
use super::SkCPURecorderImpl::RecorderImpl;
use crate::include::core::SkRecorder::{SkRecorder, Type};
use std::sync::OnceLock;
// Source: include/core/SkCPURecorder.h:25-26
// Native public CPU Recorder base is flattened into its sole in-scope actual RecorderImpl subclass.
pub type Recorder<'ctx> = RecorderImpl<'ctx>;
// Source: include/core/SkCPURecorder.h:32-33
impl SkRecorder for RecorderImpl<'_> {
    fn type_(&self) -> Type {
        Type::kCPU
    }
    fn cpuRecorder(&self) -> Option<&Recorder<'_>> {
        Some(self)
    }
    // Source: include/core/SkCPURecorder.h:60-60
    fn makeCaptureCanvas<'device>(
        &self,
        _canvas: Option<core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'device>>>,
    ) -> Option<core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'device>>> {
        None
    }
    // Source: src/core/SkCPURecorder.cpp:22-23
    fn createCaptureBreakpoint<'device>(
        &self,
        _surface: core::ptr::NonNull<crate::src::image::SkSurface_Base::SkSurface_Base<'device>>,
    ) {
    }
}
// Immutable native context pointer and shared TODO singleton are safe to share; current CPU lifecycle methods mutate no state.
unsafe impl Send for RecorderImpl<'_> {}
unsafe impl Sync for RecorderImpl<'_> {}
// Source: include/core/SkCPURecorder.h:64-72
pub fn AsRecorder(recorder: Option<&dyn SkRecorder>) -> Option<&Recorder<'_>> {
    let recorder = recorder?;
    if recorder.type_() != Type::kCPU {
        return None;
    }
    recorder.cpuRecorder()
}
// Source: src/core/SkCPURecorder.cpp:17-20
impl RecorderImpl<'static> {
    pub fn TODO() -> &'static Self {
        static RECORDER: OnceLock<&'static RecorderImpl<'static>> = OnceLock::new();
        RECORDER.get_or_init(|| Box::leak(ContextImpl::TODO().makeRecorder()))
    }
}
