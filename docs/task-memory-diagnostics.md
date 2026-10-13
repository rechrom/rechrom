# Task execution and allocation-birth diagnostics

The normal build uses Rust's standard allocator. `RECHROM_TASK_DIAGNOSTICS=1`
adds durable entry/exit lines for selected existing task/lifecycle trace spans,
independently of in-memory Tracing.start and its event cap. Actual host task
selection, rendering dispatch, final BeginFrame, Page loading/style/layout/paint,
script callbacks/checkpoints, raster batches and renderer phases are separate.
A compositor main-frame request is not evidence that Page has begun execution.
Frame receipt can be coalesced; correlate the actual BeginFrame source/sequence.
A scope exit includes errors/unwind and alone does not prove successful output.

For bounded investigations only, build:

```
cargo build -p rechrom_app --release --features allocation_diagnostics
RECHROM_TASK_DIAGNOSTICS=1 RECHROM_MEMORY_DIAGNOSTICS=1 target/release/rechrom --headless ...
```

The compile-time feature installs a System allocator wrapper with a fixed header
on every Rust allocation for the entire process lifetime. The runtime flags
control logging, never allocation pointer layout. All feature-build allocations
are counted, even before main or with logging disabled. Never treat this binary
as the ordinary Release baseline or a zero-overhead profiler.

The header retains the nearest structural phase at allocation birth. Deallocation
on another thread charges that original phase. Reallocation preserves it and
adjusts live requested bytes; `allocated_requested` counts initial requests plus
positive realloc growth. It is not the cumulative size of every realloc call.
The one-second sampler can run while the Page owner remains inside a long task.
It reads only fixed atomic counters and optional process allocator observations;
it does not access thread-confined Page, JS, or native heap owners.

These are requested Rust storage bytes, not semantic ownership, reachability,
malloc usable size, RSS or stack traces. QuickJS's translated default allocator
and Foundation nested payloads use std::alloc and therefore overlap these totals.
Native C allocations, including external HarfBuzz allocation, are not covered.
Header/padding bytes are separately reported; allocator rounding, tcache and
size-class effects remain outside exact attribution. Independently sampled rows
are not an atomic whole-heap census.

Trace recording/serialization inside browser_tracing is separately tagged so
channel/archive allocations do not masquerade as Page storage. Higher-level
DevTools report serialization can remain unscoped. Memory-census temporary
allocations are tagged separately. Durable log output uses the existing stderr
file, without a new queue or retention buffer.

The phase map follows existing package/task boundaries rather than adding a
parallel lifecycle. Pinned Chromium `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`
LayerTreeHost WillBeginMainFrame/DidBeginMainFrame and LocalFrameView lifecycle
execution scopes distinguish requests from actual work. No scheduler priorities,
collection thresholds, cache budgets, content or allocator purge policy change.
