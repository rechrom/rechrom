//! base::Location value carried by viz::BeginFrameArgs::created_from.
//! Moved from compositor_scheduler::trace_sink, which re-exports it.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceLocation {
    pub function_name: String,
    pub file_name: String,
    pub line_number: u32,
}

impl SourceLocation {
    pub fn ToString(&self) -> String {
        format!(
            "{}@{}:{}",
            self.function_name, self.file_name, self.line_number
        )
    }
}
