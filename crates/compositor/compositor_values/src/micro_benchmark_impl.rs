// Copyright 2013 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// Source: cc/benchmarks/micro_benchmark_impl.h:22-35.
//! cc::MicroBenchmarkImpl, the polymorphic impl-side benchmark the main
//! LayerTreeHost hands over in CommitState::benchmarks
//! (cc/trees/commit_state.h:163) and LayerTreeHostImpl's
//! MicroBenchmarkControllerImpl runs after each commit.

/// The `LayerTreeHostImpl*` a benchmark receives in DidCompleteCommit
/// (micro_benchmark_impl.h:19,32). The impl host implements it for its
/// per-call borrow; no source benchmark class reads it yet, so it names no
/// accessor.
pub trait MicroBenchmarkHost {}

/// Source: cc/benchmarks/micro_benchmark_impl.h:22-35. The per-layer
/// `RunOnLayer` overloads are the impl layer's own benchmark facets
/// (`LayerImplBenchmark`, `PictureLayerImplBenchmark`).
pub trait MicroBenchmarkImpl: Send {
    /// micro_benchmark_impl.h:31.
    fn IsDone(&self) -> bool;
    /// micro_benchmark_impl.h:32.
    fn DidCompleteCommit(&mut self, host: &mut dyn MicroBenchmarkHost);
}
