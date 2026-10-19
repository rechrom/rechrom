// Translation status: complete
// Copyright 2013 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/benchmarks/micro_benchmark_controller_impl.h:1-40;
//         cc/benchmarks/micro_benchmark_controller_impl.cc:1-15.
use crate::adapter::micro_benchmark::{HostHandle, MicroBenchmarkImpl};
// Source: cc/benchmarks/micro_benchmark_controller_impl.h:18-35.
// Option<Box<M>> preserves the unique_ptr's moved-from/null representation.
pub struct MicroBenchmarkControllerImpl<M> {
    benchmarks_: Vec<Option<Box<M>>>,
}
impl<M> MicroBenchmarkControllerImpl<M> {
    // Source: cc/benchmarks/micro_benchmark_controller_impl.cc:17-23.
    // Only the source non-null precondition is checked here. The resident host
    // is borrowed afresh when a benchmark receives DidCompleteCommit.
    pub fn new<H: HostHandle + ?Sized>(host: &H) -> Self {
        debug_assert!(!host.IsNull());
        Self {
            benchmarks_: Vec::new(),
        }
    }
    // Source: cc/benchmarks/micro_benchmark_controller_impl.cc:25-28.
    pub fn ScheduleRun(&mut self, benchmark: Box<M>) {
        self.benchmarks_.push(Some(benchmark));
    }
    // Source: cc/benchmarks/micro_benchmark_controller_impl.cc:30-37.
    pub fn DidCompleteCommit<H: HostHandle + ?Sized>(&mut self, host: &mut H)
    where
        M: MicroBenchmarkImpl<H>,
    {
        for benchmark in &mut self.benchmarks_ {
            let benchmark = benchmark.as_mut().expect("source nonnull benchmark");
            debug_assert!(!benchmark.IsDone());
            benchmark.DidCompleteCommit(host);
        }
        self.CleanUpFinishedBenchmarks::<H>();
    }
    // Source: cc/benchmarks/micro_benchmark_controller_impl.cc:39-44.
    // std::erase_if(vector<unique_ptr>) compacts by move assignment, destroying a
    // removed pointee when that slot is overwritten, then erases the trailing range.
    fn CleanUpFinishedBenchmarks<H: HostHandle + ?Sized>(&mut self)
    where
        M: MicroBenchmarkImpl<H>,
    {
        let Some(mut write) = self
            .benchmarks_
            .iter()
            .position(|b| b.as_ref().unwrap().IsDone())
        else {
            return;
        };
        for read in write + 1..self.benchmarks_.len() {
            if !self.benchmarks_[read].as_ref().unwrap().IsDone() {
                self.benchmarks_[write] = self.benchmarks_[read].take();
                write += 1;
            }
        }
        self.benchmarks_.truncate(write);
    }
}
