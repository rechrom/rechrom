// Copyright 2023 Google LLC. BSD-style license.
// Source: src/sksl/tracing/SkSLTraceHook.h:19-27
// Rust trait destruction follows the concrete owner; no C++ object ABI is asserted.
pub trait TraceHook: Send + Sync {
    fn line(&mut self, lineNum: i32);
    fn var(&mut self, slot: i32, val: i32);
    fn enter(&mut self, fnIdx: i32);
    fn exit(&mut self, fnIdx: i32);
    fn scope(&mut self, delta: i32);
}

// Source: src/sksl/tracing/SkSLTraceHook.h:29-41
// This is the native borrowed vector pointer, not a second trace owner.
// Moving the vector object or accessing it concurrently while this hook runs is invalid.
pub struct Tracer {
    fTraceInfo: std::ptr::NonNull<Vec<super::SkSLDebugTracePriv::TraceInfo>>,
}
// The only operations on the borrowed vector require &mut Tracer; no shared
// method reads, writes or exports its pointer. This has the same transfer/share
// requirements as a borrowed Vec<TraceInfo>. Make's unsafe lease must hold on
// every thread, including the raw stage callbacks that recover &mut TraceHook.
unsafe impl Send for Tracer {}
unsafe impl Sync for Tracer {}
impl Tracer {
    // Source: src/sksl/tracing/SkSLTraceHook.cpp:13-17
    /// The pointed-to Vec object must stay alive and at the same address until
    /// this hook is destroyed, and each callback requires exclusive access to it
    /// on every thread. The owner cannot read the trace concurrently with a
    /// callback. A traced pipeline must not run concurrently against this vector.
    pub unsafe fn Make(
        traceInfo: std::ptr::NonNull<Vec<super::SkSLDebugTracePriv::TraceInfo>>,
    ) -> Box<Self> {
        Box::new(Self {
            fTraceInfo: traceInfo,
        })
    }
}
impl TraceHook for Tracer {
    // Source: src/sksl/tracing/SkSLTraceHook.cpp:19-21
    fn line(&mut self, lineNum: i32) {
        use super::SkSLDebugTracePriv::{Op, TraceInfo};
        unsafe {
            self.fTraceInfo.as_mut().push(TraceInfo {
                op: Op::kLine,
                data: [lineNum, 0],
            });
        }
    }
    // Source: src/sksl/tracing/SkSLTraceHook.cpp:22-24
    fn var(&mut self, slot: i32, val: i32) {
        use super::SkSLDebugTracePriv::{Op, TraceInfo};
        unsafe {
            self.fTraceInfo.as_mut().push(TraceInfo {
                op: Op::kVar,
                data: [slot, val],
            });
        }
    }
    // Source: src/sksl/tracing/SkSLTraceHook.cpp:25-27
    fn enter(&mut self, fnIdx: i32) {
        use super::SkSLDebugTracePriv::{Op, TraceInfo};
        unsafe {
            self.fTraceInfo.as_mut().push(TraceInfo {
                op: Op::kEnter,
                data: [fnIdx, 0],
            });
        }
    }
    // Source: src/sksl/tracing/SkSLTraceHook.cpp:28-30
    fn exit(&mut self, fnIdx: i32) {
        use super::SkSLDebugTracePriv::{Op, TraceInfo};
        unsafe {
            self.fTraceInfo.as_mut().push(TraceInfo {
                op: Op::kExit,
                data: [fnIdx, 0],
            });
        }
    }
    // Source: src/sksl/tracing/SkSLTraceHook.cpp:31-33
    fn scope(&mut self, delta: i32) {
        use super::SkSLDebugTracePriv::{Op, TraceInfo};
        unsafe {
            self.fTraceInfo.as_mut().push(TraceInfo {
                op: Op::kScope,
                data: [delta, 0],
            });
        }
    }
}
