# Chromium Viz core translation

This crate translates the platform-independent management and algorithmic core
of Chromium Viz while preserving Rechrom's existing `VizEngine` command/value
boundary. The former implementation lives in `crates/viz_old` during migration
and is not a dependency of this crate.

Each selected Chromium `.h`/`.cc` pair maps to one Rust file with the same
relative directory and stem. Translation blocks record source-relative paths
and physical line intervals. `SOURCE_FILES.txt` is authoritative and
`tools/translation_progress.py` reports physical-line coverage against the
pinned Chromium checkout.

The selected core owns surface identity, surface lifetime and activation,
references and dependency deadlines, frame validation/resolution, damage
tracking, surface aggregation, display deadlines and swap scheduling. GPU,
Skia drawing, overlays, output devices, IPC and OS threads remain injected or
outside this package.

`VizEngine::Aggregate` returns the translated `SurfaceAggregator`'s owned,
globally remapped render passes. Renderer borrows resource leases from the
active Surface frame and does not read producer render-pass storage.
