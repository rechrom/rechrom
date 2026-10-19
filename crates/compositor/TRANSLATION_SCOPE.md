# Compositor translation scope

The compositor accepts PaintEngine's immutable `PaintArtifact` directly and
owns the complete platform-independent path from artifact layerization through
cc layer/tree/tile management and `CompositorFrame` construction. The former
`LayerTileEngine` and `FramePlan` state machines have been removed from the
production and workspace dependency graphs.

Every selected Chromium `.h`/`.cc` pair maps to one same-stem Rust file under
`src/`, preserving the complete source-relative directory. Source
annotations record exact physical line intervals. A target is complete only
when every selected source line is represented and its source behavior is
reachable through the production wiring.

## Exact source baseline

| Source area | Files added to baseline | Physical lines added |
| --- | ---: | ---: |
| Existing impl compositor baseline | 153 | 46,986 |
| Impl-frame submission value used directly by `DrawLayers` | 2 | 103 |
| Layer/tile debug color value algorithms used directly by append-quads paths | 2 | 518 |
| Tiling geometry primitives and protected-sequence state from `cc/base` | 14 | 1,912 |
| Core `Layer`/`PictureLayer`/recording, hit-test opaqueness, and solid-scrollbar path | 39 | 12,648 |
| Tiling, tile ownership, priorities, and queues | 39 | 9,593 |
| Compositor-owned tile resource pool | 2 | 1,346 |
| Raster task boundary types | 6 | 572 |
| Default CPU native task graph dependency queue and synchronous runner | 4 | 832 |
| Blink PaintArtifact compositor/layerization | 14 | 7,919 |
| Converted display-list value owner and direct RTree/geometry/invalidation closure | 7 | 2,803 |
| Node geometry caches, FloatClipRect, per-item invalidation and tracking | 9 | 1,449 |
| Client-only default pending-tree commit lifecycle owner | 2 | 666 |
| Internal LayersAsJSON diagnostic owner (no external Layer references) | 2 | 259 |
| **Core baseline** | **295 total** | **87,606 total** |

Header and implementation files with the same stem merge into one Rust file,
so the core baseline contains **166 Rust targets**. `SOURCE_FILES.txt` is
the authoritative file-level manifest; `translation_progress.py` calculates
line coverage from that manifest and the pinned Chromium checkout.

This is a responsibility closure, not an include closure. A Chromium include
does not automatically pull another subsystem into this crate. The existing
`CompositorEngine` command/effect boundary substitutes for Chromium's outer
main-thread host/proxy shell, while image-decode policy, platform raster
buffers, instrumentation, and specialized layer types enter through typed
adapters. A source file is added only when this compositor must own its live
state or algorithm to replace `LayerTileEngine`.

`cc/base/list_container*` stays with the external Viz quad container,
`cc/base/rolling_time_delta_history*` stays with the unselected compositor
timing metrics subsystem. `cc/base/rtree.h` is included solely as the direct
dependency of the converted `cc::DisplayItemList` value owner: Paint still
produces the immutable artifact, while the canonical converter fills this
distinct list. Its bounds/query index, GeometryMapper, and RasterInvalidator
are required for the selected Picture/Recording path; raster playback and
pixel execution stay external.

## Owned responsibilities

- Consume `Arc<PaintArtifact>` and perform Blink `PaintArtifactCompositor`,
  `PendingLayer`, property-tree mapping, and paint-chunk-to-layer logic.
- Own main-thread `cc::Layer`/`PictureLayer` state and synchronization into
  pending/active impl trees.
- Own `PictureLayerImpl`, tilings, tiles, priorities, damage, activation,
  resource accounting, raster task selection, and completion handling.
- Emit typed immutable raster work to the injected raster executor and consume
  typed completions without giving the executor ownership of layer/tile state.
- Produce `CompositorFrame` through the existing outer compositor
  command/effect contract.

## External boundaries

- PaintEngine continues to produce `PaintArtifact`; compositor never mutates
  DOM, style, layout, or Paint's resident state.
- Raster playback, pixels, concrete GPU/one-copy/zero-copy buffer providers,
  concrete GPU/software image-decode caches, and worker-thread placement stay
  outside the core behind typed live interfaces.
- OS synchronization primitives, feature/switch lookup, histogram/devtools
  sinks, clocks, and task-runner implementations are injected services rather
  than translated compositor state.
- Image decode caches/controllers/checker-image policy, UI resources, HUD,
  mirror/video/texture/surface/view-transition layers, JSON diagnostics, and
  Chromium proxy/thread-placement classes are outside this core manifest.
- Viz aggregation, renderer execution, platform surfaces, swap/present, clocks,
  and OS threads remain external.
- Chromium proxy/thread-placement classes are replaced by Rechrom's existing
  `CompositorEngine` command/effect boundary. Their algorithms are not silently
  reimplemented in adapters.

## `layer_tile` replacement map

| Current custom file | Source-aligned owner |
| --- | --- |
| `layer_tile/engine.rs` | Existing `CompositorEngine` host boundary plus `LayerTreeHostImpl`, `LayerTreeImpl`, `TreeSynchronizer`, and `TileManager`; the extra engine lifecycle is deleted |
| `layer_tile/manager.rs` | Blink `PaintArtifactCompositor`/`PendingLayer`, `PictureLayerImpl`, `PictureLayerTilingSet`, and `TileManager` |
| `layer_tile/recording.rs` | `PaintChunksToCcLayer`, `ContentLayerClientImpl`, `RecordingSource`, and the immutable raster-task boundary |
| `layer_tile/geometry.rs` | `ChunkToLayerMapper`, `PropertyTreeManager`, cc property trees, and draw-property utilities |
| `layer_tile/plan.rs` | Canonical layer trees, `FrameData`, tiles, raster tasks, and `CompositorFrame`; `FramePlan` is deleted rather than moved |
| `layer_tile/scrollbar.rs` | Chromium scrollbar layer and impl-layer classes |

No implementation is migrated from `layer_tile`. Every new source-relative
Rust file is translated directly from the pinned Chromium checkout, including
areas where the old package contains source annotations. The old package is
read only when the final production wiring is compared or removed; none of its
code counts toward one-to-one coverage.

The browser submits the immutable artifact to its sole Page compositor.
Synchronous image output drives that compositor and its injected raster
executor through the ordinary command and completion path; no second
layer/tile or compositor owner remains in Page or the application host.

## Completed migration sequence

1. Finished the original impl-compositor baseline.
2. Translated the added Blink layerization and cc layer/tile files into this
   crate with the same directory/file/line mapping.
3. Wired `PaintArtifact` directly into the translated compositor and routed its
   raster effects to the existing raster executor.
4. Migrated renderer/raster/browser consumers from the former layer/tile value types to
   compositor-owned task/resource identities.
5. Deleted `LayerTileEngine`, `FramePlan`, the separate layer/tile crate, and
   the old compositor after switching the production dependency graph.

The default CPU path is a required production acceptance gate: PaintEngine's
resident `Arc<PaintArtifact>` must drive layerization, scheduling, CPU raster
playback through the injected executor, completion, and visible
`CompositorFrame` submission for first paint, incremental mutations, scrolling,
and resource-loss recovery. Source coverage and successful compilation alone do
not complete the migration.

The default CPU path additionally owns `cc/raster/task_graph_work_queue` and
`cc/raster/synchronous_task_graph_runner`: these are dependency/cancellation/
completion algorithms needed to run the selected tile tasks. The synchronous
runner creates no worker or clock. A threaded implementation may inject task
execution through the same external runner contract without replacing this
canonical dependency state. OS workers/executors remain external.

`cc/trees/client_layer_tree_host_impl.{h,cc}` is the pinned revision's
client-side owner for BeginCommit/CreatePendingTree/FinishCommit/CommitComplete.
It is part of the normal pending-tree path; commit-to-active remains a source
configuration branch. Its typed executor/worklet effects cross the existing
Page boundary without including Proxy, GPU, or platform implementations.

## CPU raster external dependency evidence

The injected `raster::native::CpuRasterProvider` executes the selected immutable
`RasterPlayback` through existing Skia CPU playback. ResourcePool/TileManager
retain allocation policy and completion decisions; the external provider owns
only actual pixel storage, decode-cache execution and immutable read leases.
`cc/paint/clear_for_opaque_raster.{h,cc}` (35 + 83 physical source lines) is
translated in Paint as a pure raster helper consumed by
`RasterSource::PlaybackToCanvas` (`cc/raster/raster_source.cc:43–139`). It is an
external Paint dependency and does not change this compositor core manifest.
The new native execution modules do not consume `LayerTileEngine` or `FramePlan`.


## CPU Paint value-analysis dependency closure

These Paint-owned sources remain external dependencies and are excluded from
this compositor core manifest and its source-line total:

| Source stem | Sources | Physical source lines | Target |
| --- | ---: | ---: | --- |
| `cc/paint/paint_op_buffer_iterator.{h,cc}` | 2 | 416 | `paint/src/cc/paint/paint_op_buffer_iterator.rs` |
| `cc/paint/solid_color_analyzer.{h,cc}` | 2 | 413 | `paint/src/cc/paint/solid_color_analyzer.rs` |
| `cc/paint/discardable_image_map.{h,cc}` | 2 | 541 | `paint/src/cc/paint/discardable_image_map.rs` |
| `cc/paint/paint_flags.{h,cc}` | 2 | 617 | `paint/src/cc/paint/paint_flags.rs` |
| `cc/paint/draw_looper.{h,cc}` | 2 | 192 | `paint/src/cc/paint/draw_looper.rs` |
| `cc/paint/path_effect.{h,cc}` | 2 | 207 | `paint/src/cc/paint/path_effect.rs` |
| `cc/paint/color_filter.{h,cc}` | 2 | 320 | `paint/src/cc/paint/color_filter.rs` |

The seven full stems total **14 source files / 7 Rust files / 2,706 physical
source lines**. Runtime scalar PaintFlags now stores canonical CorePaintFlags
rather than a reduced alpha/AA/blend projection. Native modifier constructors
remain typed Skia value boundaries; buffer traversal and Paint decisions stay
in these canonical Paint modules.

`paint_op_buffer.rs` is explicitly a selected-source translation, not coverage
of the whole 1,048-line stem. Its selected physical ranges are
`cc/paint/paint_op_buffer.h:111,213–245,285–310,370–371,379–400`
(84 lines) and `.cc:76–80,283–304,324–339,588–601` (57 lines): **141 selected
source lines**. Allocation, serialization and the rest of that stem are not
counted as translated by this module.

`paint/src/cc/paint/paint_op.rs` contains only the explicitly annotated native
metadata methods (149 selected source lines) plus source ComputePaintRect
`.cc:2327–2368` (42 lines): **191 selected source lines**. These include
slow-path/non-AA classification, nested record metadata, text/layer/LCD,
discardable-image predicates and conservative paint bounds. It does not claim
whole-stem coverage;
the production `CpuNativePaintValueServices` now binds these projections to the
actual immutable/native and converted op owners. It is assembled only through
`CpuPaintServices::<SkRegion>::new_native(features, diagnostics)`.

`paint/src/wavy_geometry.rs` separately translates the pure path-value ranges
`third_party/blink/renderer/core/paint/decoration_line_painter.h:24–31` and
`.cc:130–146,173–194,197–213` (**64 selected source lines**). The HTML
solid-color decoration path is now wired: actual `DrawPath` in a tile record,
then `DrawRect` holding that immutable `PaintRecordShader`, with Repeat-X and
Decal-Y. The adapter follows `.cc:225–238,281–289,306–324,497–526`; it does not
claim the SVG paint-server ribbon branches. The retained record owns its real
ops and resource leases, with no PaintArtifact copy. Native CPU tiling uses
`SkPictureShader.cpp:180–241`, affine scale extraction from
`SkMatrix.cpp:1479–1495`, and native independent-axis image sampling from
`SkImageShader.cpp:455–468,547–580,655–660` /
`SkRasterPipeline_opts.h:2247–2255,3612–3646`. A Decal sample is masked after
gather; no transparent padding or second-axis Repeat is used. Native stroke
bounds use the existing Skia stroker and tight-bounds primitive with Blink's
`platform/geometry/path.cc:126–165` precision. These external Paint/Skia
dependency ranges do not change the compositor core manifest or claim the
remainder of their source stems.

The actual no-pixels backend uses selected native Skia value/device primitives:
`include/utils/SkNoDrawCanvas.h`, `src/core/SkCanvas.cpp:1608–1644,2963–2976`,
`src/core/SkDevice.cpp:510–645`, `src/core/SkM44.cpp:48–255`,
`src/core/SkMatrixInvert.cpp:35–150`, and `src/core/SkRRect.cpp:298–344,406–425`.
Those primitives own only matrix/clip/shape values; they never allocate a raster
surface for classification. Native rich Paint commands must be lowered at the
Paint-to-ContentDisplayItemList boundary into real replay/analysis ops sharing
the original resource owner, rather than presented as a different cc op type.

Path classification reads the public native SkPath value's `isVolatile` and
source `isConvex` algorithm (`SkPathPriv.cpp:420–657`, **238 physical source
lines**), including implicit closure, multiple contours, degenerate backtracking
and nonfinite arithmetic. The dependency includes native `SkPathIter` span
iteration, `SkPathEnums` direction values, and the original immutable
`SkPathData` relaxed atomic convexity cache; metadata does not substitute a
constant convex/volatile result.

The Paint producer also publishes source scroll facts from the actual native
Layout owners. The additional selected owner algorithms are
`core/paint/paint_layer_scrollable_area.cc:588–618,2776–2796`,
`core/html/forms/text_control_element.cc:1271–1279`, and
`core/frame/local_frame_view.cc:1504–1527` (**85 physical source lines**).
The scrolling preference runtime flag is passed through Layout's constraint
and node input owners, with its official disabled initial setting from
`platform/runtime_enabled_features.json5:4756–4759`. These external Blink
producer ranges do not change the cc core manifest. Complete compositing
preference and nonempty snap-container inputs remain explicit input blockers
until their original native producer algorithms and data owners are present;
an absent optional payload is never inferred from compositor defaults.

### Scheduler measured timing dependency (2026-10-18)

The runnable scheduler directly consumes `cc/metrics/compositor_timing_history.{h,cc}` (138 + 429 lines) and `cc/base/rolling_time_delta_history.{h,cc}` (54 + 86 lines): 4 source files, 2 same-stem Rust targets, 707 lines. They retain the bounded multiset/chronological history, percentile cache, measured critical/noncritical queue and commit/activation/draw histories, and original VSync-aligned metrics buckets. Native runtime only injects executor time, schedules typed task tokens and forwards diagnostics. The core manifest now contains **299 source files / 168 Rust targets / 88,313 lines**.

### Native wheel/autoscroll animation owner closure (2026-10-18)

The source normal CPU wheel/autoscroll path directly requires nine complete `cc/animation` stems: `scroll_offset_animations_impl`, `scroll_offset_animation_curve`, `scroll_offset_animation_curve_factory`, `keyframe_model`, `keyframe_effect`, `animation`, `animation_timeline`, `animation_id_provider`, and `animation_events`. Their 18 header/implementation sources contribute 4,429 lines; the core baseline is now **317 sources / 177 same-stem targets / 92,742 lines**. These owners keep source model/effect/phase ordering. Concrete CSS transform/filter/color curves are outside this closure and retain typed curve interfaces.

The direct pure animation phase dependency `ui/gfx/animation/keyframe/keyframe_model.{h,cc}` contributes 622 lines (258 + 364), and `keyframe_effect.{h,cc}` 507 lines (133 + 374). These two same-stem external value/phase owners are recorded separately (4 source files / 1,129 lines), not silently counted as translated cc core. They contain no platform clocks, renderer, GPU, or pixel execution. Runtime supplies the current native clock/diagnostics and call-scoped mutation targets. A stem remains in progress until all of its fields and methods are faithfully covered.


### Production CPU Paint capability assembly

`runtime/paint/native_services.rs` constructs the concrete value/metadata,
raster-tracking and logging/recorder capabilities. Metadata updates remain in
canonical `PaintOpBufferMetadata`; folding and solid analysis call the canonical
Paint iterators/analyzer, and discardable image generation calls the canonical
map/RTree. The generated animated metadata map is consumed once when crossing
the fixed runtime value boundary, rather than retained twice. Scroll offsets
are borrowed from the same Generate call. Recording retains original native
PaintRecord/resource leases; it does not snapshot a PaintArtifact or consult a
layer/tile owner. Optional under-invalidation pixel comparison is explicitly
configuration-gated: requesting it without its native raster comparison backend
fails constructor validation; the ordinary disabled path honors the canonical
recording/converter preconditions. Logging emits actual operation fields.

Remaining producer/native replay capabilities are explicit: independently
filled SVG clip children need genuine SkPathOps union; Document images need a
typed native record/color capability; animated images need the actual frame
resolver/timeline; reference/reflection filters need the actual replay graph
and reference-space geometry. They are not synthesized as empty records,
static images or unrelated Paint op kinds. These blockers are not part of the
completed HTML solid-color wavy record-shader path.


The CPU UI resource capability is implemented by the same injected
`raster::native::CpuRasterProvider`: its single native backing table/counter
serves both tile and UI images. The returned shared image supplies the actual
allocation identity; the caller's tile id is only preferred. CPU write mapping
moves the sole mutable pixel owner, rejects concurrent read/raster leases and
publishes the same backing on Drop. `SharedMapping` is only a plane projection.
The six native UI methods delegate copy, scale, clear and image draw to Skia's
borrowed N32 surface (`skia/src/core/SkSurface_Raster.rs`); WrapPixels preserves
its original target buffer and row stride. The actual UI N32 layout is BGRA8888,
as required by `cc/trees/layer_tree_host_impl.cc:3531–3532`. Source evidence is
`src/image/SkSurface_Raster.cpp:191–208`,
`src/core/SkWritePixelsRec.cpp:12–59`, `SkConvertPixels.cpp:28–45`,
`SkCanvas.cpp:1307–1323,2385–2396`, and
`src/image/SkImage_RasterFactories.cpp:73–79` /
`SkImage_Raster.cpp:182–206` (copy-if-mutable image ownership). The backend's
immutable Pixmap packs the active N32 image rows once at asImage; no caller
PaintArtifact or target WrapPixels storage is copied. These are external native
backend dependencies and do not add compositor core translation coverage.
