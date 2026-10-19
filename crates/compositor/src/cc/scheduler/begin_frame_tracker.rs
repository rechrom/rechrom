// Translation status: complete
// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::adapter::{
    begin_frame::BeginFrameArgs,
    time::{TimeDelta, TimeTicks},
    trace_sink::{
        BeginImplFrameArgsTrace, BeginImplFrameState, SourceLocation, TraceEvent, TraceSink,
    },
};

// Source: cc/scheduler/begin_frame_tracker.h:25-43,95-100
// Checks valid, monotonically increasing BeginFrameArgs and their usage period.
// TimeTicks::Now() is supplied as the `now` input to Start/Finish, keeping clock
// ownership at the host boundary. TRACE_EVENT macros use the typed TraceSink.
#[derive(Debug)]
pub struct BeginFrameTracker {
    location_: SourceLocation,
    location_string_: String,
    current_updated_at_: TimeTicks,
    current_args_: BeginFrameArgs,
    current_finished_at_: TimeTicks,
}

impl BeginFrameTracker {
    // Source: cc/scheduler/begin_frame_tracker.cc:13-18
    pub fn new(location: SourceLocation) -> Self {
        let location_string = location.ToString();
        Self {
            location_: location,
            location_string_: location_string,
            current_updated_at_: TimeTicks::default(),
            current_args_: BeginFrameArgs::default(),
            current_finished_at_: TimeTicks::default() + TimeDelta::Microseconds(-1),
        }
    }

    // Source: cc/scheduler/begin_frame_tracker.h:45-52
    // Source: cc/scheduler/begin_frame_tracker.cc:20-49
    pub fn Start(&mut self, new_args: &BeginFrameArgs, now: TimeTicks, trace: &mut dyn TraceSink) {
        // Source: cc/scheduler/begin_frame_tracker.cc:21-34
        // TRACE_EVENT performs lazy argument evaluation while tracing is off.
        if trace.enabled() {
            trace.emit(TraceEvent::BeginFrameFlow {
                frame_time_us: new_args.frame_time.since_origin().InMicroseconds(),
                location: self.location_string_.clone(),
            });
            trace.emit(TraceEvent::BeginFrameAsyncBegin {
                location: self.location_string_.clone(),
                frame_time_us: new_args.frame_time.since_origin().InMicroseconds(),
                new_args: new_args.clone(),
                current_args: self.current_args_.clone(),
            });
        }

        // Source: cc/scheduler/begin_frame_tracker.cc:36-44
        debug_assert!(new_args.IsValid());
        debug_assert!(self.current_args_.frame_time <= new_args.frame_time);
        debug_assert!(
            self.HasFinished(),
            "Tried to start a new frame before finishing an existing frame."
        );
        self.current_updated_at_ = now;
        self.current_args_ = new_args.clone();
        self.current_finished_at_ = TimeTicks::default();

        // Source: cc/scheduler/begin_frame_tracker.cc:46-48
        // TODO(mithro): Track the delta from frame_time to current_updated_at_.
    }

    // Source: cc/scheduler/begin_frame_tracker.h:62-64
    // Source: cc/scheduler/begin_frame_tracker.cc:51-57
    pub fn Current(&self) -> &BeginFrameArgs {
        debug_assert!(
            !self.HasFinished(),
            "Tried to use viz::BeginFrameArgs after marking the frame finished."
        );
        debug_assert!(
            self.current_args_.IsValid(),
            "Tried to use viz::BeginFrameArgs before starting a frame!"
        );
        &self.current_args_
    }

    // Source: cc/scheduler/begin_frame_tracker.h:53-55
    // Source: cc/scheduler/begin_frame_tracker.cc:59-68
    pub fn Finish(&mut self, now: TimeTicks, trace: &mut dyn TraceSink) {
        debug_assert!(
            !self.HasFinished(),
            "Tried to finish an already finished frame"
        );
        self.current_finished_at_ = now;
        if trace.enabled() {
            trace.emit(TraceEvent::BeginFrameAsyncEnd {
                location: self.location_string_.clone(),
                frame_time_us: self
                    .current_args_
                    .frame_time
                    .since_origin()
                    .InMicroseconds(),
            });
        }
    }

    // Source: cc/scheduler/begin_frame_tracker.h:65-68
    // Source: cc/scheduler/begin_frame_tracker.cc:70-77
    pub fn Last(&self) -> &BeginFrameArgs {
        debug_assert!(
            self.current_args_.IsValid(),
            "Tried to use last viz::BeginFrameArgs before starting a frame!"
        );
        debug_assert!(
            self.HasFinished(),
            "Tried to use last viz::BeginFrameArgs before the frame is finished."
        );
        &self.current_args_
    }

    // Source: cc/scheduler/begin_frame_tracker.h:69
    // Source: cc/scheduler/begin_frame_tracker.cc:79-83
    pub fn HasLast(&self) -> bool {
        debug_assert!(
            self.HasFinished(),
            "Tried to use last viz::BeginFrameArgs before the frame is finished."
        );
        self.current_args_.IsValid()
    }

    // Source: cc/scheduler/begin_frame_tracker.h:71-74
    // Source: cc/scheduler/begin_frame_tracker.cc:85-93
    pub fn Interval(&self) -> TimeDelta {
        let mut interval = self.current_args_.interval;
        // Normal intervals are about 16ms; less than 1ms is likely an error.
        if interval < TimeDelta::Milliseconds(1) {
            interval = BeginFrameArgs::DefaultInterval();
        }
        interval
    }

    // Source: cc/scheduler/begin_frame_tracker.h:76-79
    // Source: cc/scheduler/begin_frame_tracker.cc:94-124
    // The typed output substitutes for Perfetto's protobuf writer/context.
    pub fn AsProtozeroInto(&self, now: TimeTicks, state: &mut BeginImplFrameArgsTrace) {
        state.updated_at_us = self.current_updated_at_.since_origin().InMicroseconds();
        state.finished_at_us = self.current_finished_at_.since_origin().InMicroseconds();
        if self.HasFinished() {
            state.state = BeginImplFrameState::BEGIN_FRAME_FINISHED;
            state.current_args = Some(self.current_args_.clone());
        } else {
            state.state = BeginImplFrameState::BEGIN_FRAME_USING;
            state.last_args = Some(self.current_args_.clone());
        }

        let frame_time = self.current_args_.frame_time;
        let deadline = self.current_args_.deadline;
        let interval = self.current_args_.interval;

        let timestamps = &mut state.timestamps_in_us;
        timestamps.interval_delta = interval.InMicroseconds();
        timestamps.now_to_deadline_delta = (deadline - now).InMicroseconds();
        timestamps.frame_time_to_now_delta = (now - frame_time).InMicroseconds();
        timestamps.frame_time_to_deadline_delta = (deadline - frame_time).InMicroseconds();
        timestamps.now = now.since_origin().InMicroseconds();
        timestamps.frame_time = frame_time.since_origin().InMicroseconds();
        timestamps.deadline = deadline.since_origin().InMicroseconds();
    }

    // Source: cc/scheduler/begin_frame_tracker.h:81-85
    pub fn DangerousMethodHasStarted(&self) -> bool {
        !self.current_updated_at_.is_null()
    }

    // Source: cc/scheduler/begin_frame_tracker.h:86
    pub fn DangerousMethodHasFinished(&self) -> bool {
        self.HasFinished()
    }

    // Source: cc/scheduler/begin_frame_tracker.h:87
    // Source: cc/scheduler/begin_frame_tracker.cc:126-133
    pub fn DangerousMethodCurrentOrLast(&self) -> &BeginFrameArgs {
        if !self.HasFinished() {
            self.Current()
        } else {
            self.Last()
        }
    }

    // Source: cc/scheduler/begin_frame_tracker.h:89-93
    fn HasFinished(&self) -> bool {
        !self.current_finished_at_.is_null()
    }
}
