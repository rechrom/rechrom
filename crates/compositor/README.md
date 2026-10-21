# Chromium impl-compositor translation

This crate is the source-aligned, platform-independent compositor core. Its
stable outer boundary remains Rechrom's `CompositorEngine` command/effect
interface. The source-aligned core is the production implementation; the
previous compositor and separate layer/tile package have been removed.

Each same-stem Chromium `.h`/`.cc` pair maps to one Rust file with the same
relative directory and stem. Every translated declaration and implementation
block records the original relative path and line interval.

This crate owns compositor state, Blink PaintArtifact layerization, cc layers,
picture-layer tilings, tile priority and lifecycle, raster-task selection,
scheduling, tree activation, damage, draw-property calculation, and impl
scrolling. Its public input accepts PaintEngine's immutable `PaintArtifact`
directly. It does not own PaintArtifact production, raster playback or pixels,
Viz aggregation, render targets, presentation, platform graphics APIs, or OS
threads.

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
