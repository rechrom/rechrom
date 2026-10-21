use crate::src::sksl::SkSLCompiler::POISON_TAG;
// Source: src/sksl/SkSLErrorReporter.h:17-17
pub use crate::src::sksl::SkSLPosition::Position;
use core::ptr::NonNull;
impl ErrorReporterData {
    // Source: src/sksl/SkSLErrorReporter.h:24-24
    pub fn new() -> Self {
        Self {
            fSource: None,
            fErrorCount: 0,
        }
    }
}
impl Default for ErrorReporterData {
    fn default() -> Self {
        Self::new()
    }
}
// Source: src/sksl/SkSLErrorReporter.h:26-26
// Adapted virtual dispatch: Rust trait-object owners execute the concrete Drop;
// actual nonowning source view and scalar counter have trivial native teardown.
// Blanket extension retains native nonvirtual API: callers cannot override
// error/source/count operations separately from the actual virtual callback.
pub trait ErrorReporterExt: ErrorReporter {
    // Source: src/sksl/SkSLErrorReporter.h:28-28
    fn error(&mut self, position: Position, msg: &[u8]) {
        error(self, position, msg)
    }
    // Source: src/sksl/SkSLErrorReporter.h:30-30
    fn source(&self) -> Option<&[u8]> {
        self.state().fSource.map(|p| unsafe { p.as_ref() })
    }
    // Source: src/sksl/SkSLErrorReporter.h:32-32
    /// Safety: source is borrowed, not copied/adopted. Its storage remains live
    /// for every later source()/diagnostic callback until setSource replaces it.
    /// None preserves original null-empty view; Some(empty) preserves nonnull empty.
    unsafe fn setSource(&mut self, source: Option<&[u8]>) {
        self.state_mut().fSource = source.map(NonNull::from);
    }
    // Source: src/sksl/SkSLErrorReporter.h:34-36
    fn errorCount(&self) -> i32 {
        self.state().fErrorCount
    }
    // Source: src/sksl/SkSLErrorReporter.h:38-40
    fn resetErrorCount(&mut self) {
        self.state_mut().fErrorCount = 0;
    }
}
impl<T: ErrorReporter + ?Sized> ErrorReporterExt for T {}
// Source: src/sksl/SkSLErrorReporter.h:46-46
pub trait ErrorReporter {
    // Pure language projections of native inherited state, no second owner.
    fn state(&self) -> &ErrorReporterData;
    fn state_mut(&mut self) -> &mut ErrorReporterData;
    fn handleError(&mut self, msg: &[u8], position: Position);
}
// Native private position(int)49 has no definition or call in pinned sources;
// it is not replaced with a made-up offset/line implementation.
// Source: src/sksl/SkSLErrorReporter.h:51-52
// Adapted view: nullable pointer+slice metadata retains borrowed byte ownership,
// null-versus-nonnull-empty identity and native field order, no copied source.
#[repr(C)]
pub struct ErrorReporterData {
    fSource: Option<NonNull<[u8]>>,
    fErrorCount: i32,
}
// Source: src/sksl/SkSLErrorReporter.h:58-61
// Actual native concrete aborting test reporter, not a generic missing backend.
pub struct TestingOnly_AbortErrorReporter {
    base: ErrorReporterData,
}
impl TestingOnly_AbortErrorReporter {
    pub fn new() -> Self {
        Self {
            base: ErrorReporterData::new(),
        }
    }
}
impl Default for TestingOnly_AbortErrorReporter {
    fn default() -> Self {
        Self::new()
    }
}
// Source: src/sksl/SkSLErrorReporter.cpp:16-23
pub fn error<R: ErrorReporter + ?Sized>(reporter: &mut R, position: Position, msg: &[u8]) {
    // Native array-to-string_view conversion omits the terminal NUL.
    let poison = &POISON_TAG[..POISON_TAG.len() - 1];
    if msg.windows(poison.len()).any(|s| s == poison) {
        return;
    }
    reporter.state_mut().fErrorCount += 1;
    reporter.handleError(msg, position);
}
impl ErrorReporter for TestingOnly_AbortErrorReporter {
    fn state(&self) -> &ErrorReporterData {
        &self.base
    }
    fn state_mut(&mut self) -> &mut ErrorReporterData {
        &mut self.base
    }
    // Source: src/sksl/SkSLErrorReporter.cpp:25-27
    fn handleError(&mut self, msg: &[u8], _pos: Position) {
        crate::SK_ABORT!(@source b"src/sksl/SkSLErrorReporter.cpp\0".as_ptr(),26; "%.*s",msg.len() as i32,msg.as_ptr());
    }
}
