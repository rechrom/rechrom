# Chromium Viz core translation

This crate translates the platform-independent management and algorithmic core
of Chromium Viz while preserving Rechrom's existing `VizEngine` command/value
boundary. This is the sole Viz implementation in the workspace.

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

`VizEngine::Aggregate` returns one opaque `VizOutput`, the sole Viz-to-Renderer
interface. It owns the source-shaped `AggregatedFrame`, globally remapped CPU
resource leases, root presentation data and draw receipt. Renderer cannot
address producer Surfaces, submissions, metadata or resource lists.
