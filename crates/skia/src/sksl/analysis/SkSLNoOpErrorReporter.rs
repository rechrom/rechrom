// UNVERIFIED: user prohibited all validation.
use crate::src::sksl::{
    SkSLErrorReporter::{ErrorReporter, ErrorReporterData},
    SkSLPosition::Position,
};
// Source: src/sksl/analysis/SkSLNoOpErrorReporter.h:17-20
// Adapted inherited reporter state and virtual callback; empty callback is native.
pub struct NoOpErrorReporter {
    base: ErrorReporterData,
}
impl NoOpErrorReporter {
    pub fn new() -> Self {
        Self {
            base: ErrorReporterData::new(),
        }
    }
}
impl ErrorReporter for NoOpErrorReporter {
    fn state(&self) -> &ErrorReporterData {
        &self.base
    }
    fn state_mut(&mut self) -> &mut ErrorReporterData {
        &mut self.base
    }
    fn handleError(&mut self, _: &[u8], _: Position) {}
}
