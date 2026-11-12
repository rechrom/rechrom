// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source operations: core/dom/static_range.h:48,65-66;
// core/dom/static_range.cc:73-78; core/events/input_event.cc:140-141,157-158,225-226.
// Borrowed projections of DOM's existing live/static Range owners. This module
// supplies no independent boundary-point storage or DOM traversal algorithm.

use std::rc::Rc;

pub type StaticRangeRef = Rc<dyn StaticRange>;
pub type LiveRangeRef = Rc<dyn LiveRange>;

// DOM owns exception construction and live range offset validation.
#[derive(Clone, Debug)]
pub struct RangeConversionError {
    pub message: String,
}

#[derive(Default)]
pub struct RangeExceptionState {
    exception: Option<RangeConversionError>,
}
impl RangeExceptionState {
    pub fn Throw(&mut self, exception: RangeConversionError) {
        if self.exception.is_none() {
            self.exception = Some(exception);
        }
    }
    pub fn HadException(&self) -> bool {
        self.exception.is_some()
    }
    pub fn TakeException(&mut self) -> Option<RangeConversionError> {
        self.exception.take()
    }
}

// Source: core/dom/static_range.cc:73-78
// toRange returns its newly created live owner even after recording a setStart
// or setEnd exception; InputEvent's source loop continues through every range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StaticRangeSnapshot {
    pub document: u128,
    pub start_container: u64,
    pub start_offset: usize,
    pub end_container: u64,
    pub end_offset: usize,
}

pub trait StaticRange {
    /// Bindings project only values from the existing DOM StaticRange owner.
    /// Non-DOM test/embedder range implementations need not expose a wrapper.
    fn Snapshot(&self) -> Option<StaticRangeSnapshot> {
        None
    }

    fn ToRange(&self, exception_state: &mut RangeExceptionState) -> LiveRangeRef;
}

// Source: core/events/input_event.cc:223-227
pub trait LiveRange {
    fn CreateStaticRange(&self) -> StaticRangeRef;
}
