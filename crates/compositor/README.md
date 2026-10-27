# Chromium impl-compositor translation

This crate is the source-aligned, platform-independent compositor core. Its
stable outer boundary remains Rechrom's `CompositorEngine` command/effect
interface. The source-aligned core is the production implementation; the
previous compositor and separate layer/tile package have been removed.

Each same-stem Chromium `.h`/`.cc` pair maps to one Rust file with the same
relative directory and stem. Every translated declaration and implementation
block records the original relative path and line interval.

The compositor package family owns compositor state, Blink PaintArtifact
layerization, cc layers, picture-layer tilings, tile priority and lifecycle,
raster-task selection and playback orchestration, scheduling, tree activation,
damage, draw-property calculation, and impl scrolling. Its public input accepts
PaintEngine's immutable `PaintArtifact` directly. It does not own PaintArtifact
production, a concrete pixel backend, Viz aggregation, render targets,
presentation, platform graphics APIs, or OS threads.

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
`NativeCompositorFrameSubmission` remain facade-owned boundary names.

| Package | Owns | Primary performance evidence |
| --- | --- | --- |
| `compositor_layerize` | PaintArtifact chunk grouping, layer split/merge/promotion and compositing reasons; emits layer descriptions and property references | chunks visited/merged, layers produced/reused, split/squash reasons, invalidations, duration |
| `compositor_tree` | retained layer/property-tree topology, pending/active synchronization, commit and activation | nodes synchronized, changed nodes, activation wait, property-tree update duration |
| `compositor_tile` | tilings, tile size/lifecycle/coverage/priority and raster-work selection | visible/required tiles, selection queue depth, reuse/drop counts, duration |
| `compositor_raster` | raster capabilities, task graphs, paint-op playback execution and completion | graph nodes/edges, queued/running/completed tasks, pixels/bytes, wait and worker duration |
| `compositor_draw` | draw properties, damage, visibility, occlusion, culling, surfaces and quad/render-pass content | visited/culled layers, damage area, surfaces/quads, duration |
| `compositor_frame` | frame token/metadata, render-pass containers, SwapPromise/presentation callbacks and immutable submission payload | passes/quads/resources, frame bytes, build/submit/presentation duration |
| `compositor_resource` | resource IDs, budgets, pools, leases, allocation/reuse, export/return and actual reclamation | live/available bytes, allocation/reuse/reclamation/return counts, duration |
| `compositor_scheduler` | BeginFrame/deadline state machine, action selection and compositor throttling | actions, deadline misses, throttling/defer reasons, duration |
| `compositor_input` | hit testing/routing, scroll latching/snap, browser controls and interaction-driven viewport changes | event route/disposition, scroll/pinch/snap work, main-thread fallback, input-to-mutation duration |
| `compositor_animation` | timelines/keyframes and tick-time property evaluation | active/ticked animations, property evaluations, promoted/fallback counts, duration |

Trace categories use the exact package names (`compositor.layerize`,
`compositor.tree`, and so on). Cross-package calls are nested, so a trace can
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
