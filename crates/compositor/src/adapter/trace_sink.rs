//! Typed tracing boundary. The embedding may serialize these events into its
//! existing tracing collector. There is no clock or thread ownership here.

use super::begin_frame::BeginFrameArgs;

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TraceEvent {
    BeginFrameFlow {
        frame_time_us: i64,
        location: String,
    },
    BeginFrameAsyncBegin {
        location: String,
        frame_time_us: i64,
        new_args: BeginFrameArgs,
        current_args: BeginFrameArgs,
    },
    BeginFrameAsyncEnd {
        location: String,
        frame_time_us: i64,
    },
}

pub trait TraceSink {
    fn enabled(&self) -> bool {
        true
    }
    fn emit(&mut self, event: TraceEvent);
}

#[derive(Default)]
pub struct NullTraceSink;
impl TraceSink for NullTraceSink {
    fn enabled(&self) -> bool {
        false
    }
    fn emit(&mut self, _event: TraceEvent) {}
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BeginImplFrameState {
    #[default]
    BEGIN_FRAME_FINISHED,
    BEGIN_FRAME_USING,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BeginImplFrameTimestamps {
    pub interval_delta: i64,
    pub now_to_deadline_delta: i64,
    pub frame_time_to_now_delta: i64,
    pub frame_time_to_deadline_delta: i64,
    pub now: i64,
    pub frame_time: i64,
    pub deadline: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BeginImplFrameArgsTrace {
    pub updated_at_us: i64,
    pub finished_at_us: i64,
    pub state: BeginImplFrameState,
    pub current_args: Option<BeginFrameArgs>,
    pub last_args: Option<BeginFrameArgs>,
    pub timestamps_in_us: BeginImplFrameTimestamps,
}
