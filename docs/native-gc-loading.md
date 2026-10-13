# Requested native collection during loading

The application services Foundation allocation-pressure requests at a
coalesced normal-priority browser-page owner task, after foreground/background Page tasks and
native scopes have returned. Active document loading is not a no-GC scope. Input
and priority-frame checks still precede collection. General background work keeps
its old priority. The existing opaque EventLoopEngine is unchanged. No collector, heap
threshold, timer, cache budget or engine dependency changes.

Chromium pin `6c1d401fcca5e1b0030563a90c2f2bba168e0c15` selects V8
`96550751177dbdb761b1708431616374148bc0eb`. The relevant unified heap production
path is Blink ThreadState → CppHeap::ReportBufferedAllocationSizeIfPossible →
V8 allocation limits/incremental tasks. `Heap::ShouldOptimizeForLoadTime` is
`IsLoading() && !AllocationLimitOvershotByLargeMargin()`, and the loading hint is
time-bounded. Rechrom's removed absolute Page::IsLoading veto had no pressure
override. Its former Background-only service could also starve behind a continuous
loading stream; the private NativeGC task shares the normal FIFO instead. The standalone cppgc GCInvoker safe non-nestable foreground dispatch
corroborates the task boundary, but is not Blink's exact heap-growth policy.
Rechrom retains its synchronous requested collector, not V8 incremental behavior.

Sources:
- https://github.com/chromium/chromium/blob/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/platform/heap/thread_state.cc
- https://github.com/v8/v8/blob/96550751177dbdb761b1708431616374148bc0eb/src/heap/cppgc-js/cpp-heap.cc
- https://github.com/v8/v8/blob/96550751177dbdb761b1708431616374148bc0eb/src/heap/heap.cc
- https://github.com/v8/v8/blob/96550751177dbdb761b1708431616374148bc0eb/src/heap/heap.h
- https://github.com/v8/v8/blob/96550751177dbdb761b1708431616374148bc0eb/src/heap/cppgc-internal/gc-invoker.cc

`RECHROM_MEMORY_DIAGNOSTICS` remains default off. Foundation classifies registered
root cells without tracing, marking or pruning and reports work-vector requested
capacity. Skia reports calling-thread identity-vector capacity and strike charges,
which can overlap. Host GNU/Linux optionally resolves mallinfo2 for coarse arena
in-use/free and mmap observations. Collection before/after snapshots include
elapsed collection time; sampling/logging time outside that interval is excluded.
These counters are neither a complete allocation census nor additive RSS totals.
No diagnostic calls allocator trim, forces GC, clears caches or changes content.

Foreground dispatch/coalescing follows the pinned
https://github.com/v8/v8/blob/96550751177dbdb761b1708431616374148bc0eb/src/heap/incremental-marking-job.cc#L46
user-visible task path. `native-gc-task queue_ms` is the successful task attempt's
enqueue-to-service time: priority-yield retries start a new attempt timestamp.
It is neither the exact allocation-request age nor a hard bound on total delay.
Normal FIFO prevents later normal work overtaking GC; indefinitely ready input
or rendering may still defer service under the preserved priority rules.
