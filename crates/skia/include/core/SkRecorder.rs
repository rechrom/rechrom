// Source: include/core/SkRecorder.h:21-21
// Native abstract public name reexports its sole trait; no extra object or vtable layer.
pub use RecorderInterface as SkRecorder;
// Source: include/core/SkRecorder.h:29-33
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    kCPU,
    kGanesh,
    kGraphite,
}
// Rust trait dispatch substitutes native abstract inheritance. Capture-only state and virtuals are outside CPU lifecycle scope.
pub trait RecorderInterface {
    // Source: include/core/SkRecorder.h:35-35
    fn type_(&self) -> Type;
    // Source: include/core/SkRecorder.h:37-37
    fn cpuRecorder(&self) -> Option<&crate::src::core::SkCPURecorder::Recorder<'_>>;
    // Source: include/core/SkRecorder.h:51-51
    fn makeCaptureCanvas<'device>(
        &self,
        canvas: Option<core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'device>>>,
    ) -> Option<core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'device>>>;
    // Source: include/core/SkRecorder.h:52-52
    fn createCaptureBreakpoint<'device>(
        &self,
        surface: core::ptr::NonNull<crate::src::image::SkSurface_Base::SkSurface_Base<'device>>,
    );
}
