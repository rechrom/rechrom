# Chromium impl-compositor translation

This crate is the source-aligned, platform-independent compositor core. Its
stable outer boundary remains Rechrom's `CompositorEngine` command/effect
interface. The source-aligned core is the production implementation; the
previous compositor and separate layer/tile package have been removed.

Each same-stem Chromium `.h`/`.cc` pair maps to one Rust file with the same
relative directory and stem. Every translated declaration and implementation
block records the original relative path and line interval.

This directory holds cc's impl side (`compositor` facade and impl packages)
and the cc shared roots both sides name. The main side (Blink PaintArtifact
layerization, cc main layers, `LayerTreeHost`, `ProxyMain`) is `layerize`
with `compositor_layerize` under `crates/layerize/`; the impl side receives
its commits only through `layerize::commit`. The impl side owns impl layer
trees, picture-layer tilings, tile priority and lifecycle, raster-task
selection and playback orchestration, scheduling, tree activation, damage,
draw-property calculation, and impl scrolling. It does not own PaintArtifact
production or layerization, a concrete pixel backend, Viz aggregation, render
targets, presentation, platform graphics APIs, or OS threads.

Package classes (guarded by `crates/layerize/tests/layerize_boundary.rs`):

- cc shared roots: `scroll_snap`, `scroll_animation`, `compositor_tree` (cc
  property trees), `compositor_animation` (cc/animation, main and impl
  AnimationHost instances), `compositor_values` (cc input, settings, viz and
  gfx values, including `gfx::Transform` and `gfx::ColorSpace`, and the
  `input_to_impl` input-queue messages the document thread moves to the impl
  input handler), `cc_paint` (cc/paint: PaintRecord, DisplayItemList, image
  and worklet values, cc/base/rtree and the SkRegion/SkPath value backend;
  `layerize::commit::paint` re-exports it; records are built from the
  PaintArtifact on the producer side), `cc_raster` (cc/raster:
  RasterSource, RasterBuffer / RasterBufferProvider and
  `CompositorRasterProvider`, the UI-resource raster capability; the
  `raster` service implements them). They depend
  only on each other and on value crates (including `viz_common`, the
  components/viz/common frame values under `crates/viz/`); Layerize and
  crates outside this family may use them.
- impl packages: every other package here. Only the `compositor` facade and
  other impl packages may depend on them.

## Responsibility packages and performance attribution

The compositor is partitioned by measurable production responsibility. A
package boundary is valid only when work can be timed and counted at that
boundary without reaching backward into another owner. The `compositor`
package remains the Page-facing facade and assembly root; it does not absorb
algorithmic work from the packages below.

Legacy source-aligned paths in the facade become compatibility re-exports or
concrete type/service adapters after an algorithm moves. They may translate
resident Chromium objects into package trait calls, but do not own the
iteration, selection, phase-order, or fallback policy. This keeps downstream
imports stable while making Cargo ownership and package-level compilation
expose accidental reverse dependencies. The external command/effect protocol and
`CompositorFrameSubmission` remain facade-owned boundary names.

| Package | Owns | Primary performance evidence |
| --- | --- | --- |
| `compositor_tree` (shared) | cc property trees, nodes, ids and their shared update helpers | property-tree update duration |
| `compositor_tree_sync` | pending/active synchronization: commit into the sync tree, activation, property pushes, impl property-tree, clip and render-target updates | nodes synchronized, changed nodes, activation wait, duration |
| `compositor_tile` | tilings, tile size/lifecycle/coverage/priority and raster-work selection | visible/required tiles, selection queue depth, reuse/drop counts, duration |
| `compositor_raster` | raster capabilities, task graphs, paint-op playback execution and completion | graph nodes/edges, queued/running/completed tasks, pixels/bytes, wait and worker duration |
| `compositor_draw` | draw properties, damage, visibility, occlusion, culling, surfaces and quad/render-pass content | visited/culled layers, damage area, surfaces/quads, duration |
| `compositor_frame` | frame token/metadata, render-pass containers, SwapPromise/presentation callbacks and immutable submission payload | passes/quads/resources, frame bytes, build/submit/presentation duration |
| `compositor_resource` | resource IDs, budgets, pools, leases, allocation/reuse, export/return and actual reclamation | live/available bytes, allocation/reuse/reclamation/return counts, duration |
| `compositor_scheduler` | BeginFrame/deadline state machine, action selection and compositor throttling | actions, deadline misses, throttling/defer reasons, duration |
| `compositor_input` | hit testing/routing, scroll latching/snap, browser controls and interaction-driven viewport changes | event route/disposition, scroll/pinch/snap work, main-thread fallback, input-to-mutation duration |
| `compositor_animation` (shared) | timelines/keyframes and tick-time property evaluation | active/ticked animations, property evaluations, promoted/fallback counts, duration |
| `compositor_values` (shared) | cc values named by both sides; no hot algorithm | none (values only) |

Trace categories use the exact package names (`compositor.tree`,
`compositor.tree_sync`, and so on; the moved `compositor_layerize` keeps
`compositor.layerize`). Cross-package calls are nested, so a trace can
separate inclusive facade latency from the exclusive cost of each owner. Work
counts must be recorded by the package that owns the underlying collection;
the facade may aggregate those values but must not reconstruct them by walking
private state. Asynchronous raster spans retain frame/task context and report
queueing separately from worker execution, preventing executor delay from
being misdiagnosed as tile-selection cost.

The translated core is the compositor's decision and control plane. It owns the
resident main/impl layer trees and tile metadata because those are compositor
state, replacing the separate `LayerTileEngine` and its `FramePlan` model.
It emits immutable raster work to an injected executor and consumes typed
completion/resource facts, but never opens a graphics device or presents
pixels. Clocks, task runners, raster execution, frame sinks, and platform
capabilities enter through typed live interfaces; the core does not cache
snapshots of their owner state.

The default raster execution adapter is `raster::RasterEngine`.
Its immutable read leases travel with actual transferable frame resources;
Page/Viz do not consume raster jobs, provider state or a pixel lookup map.
Paint supplies the source-aligned `clear_for_opaque_raster` helper outside this
crate's core source set.

Paint's canonical playback folding, solid-color analysis, discardable-image map,
and selected PaintOpBuffer metadata are external value dependencies. Paint
analysis uses a real no-pixels Skia canvas for analysis and retains per-op alpha
and the actual nested record resource owner.
