# Scroll publication and render-pass backing lifecycle

Source pin: Chromium `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`.
This is an adaptation to Rechrom's typed engine boundaries, not a claim that
Rechrom implements Chromium's entire scheduler or compositor input system.

## Committed and impl-side scroll state

`cc/base/synced_property.h` separates a main-thread base, active impl delta,
and the delta reflected in a pending tree. A new main scroll without impl input
must replace the base. Keeping the previous absolute visual offset cancels
programmatic scrolls when the new tree activates.

The compositor now owns this additive state privately. An immutable
`MainThreadScrollDelta` carries a monotonic batch sequence, native owner client
identity and actual delta. Page applies it to the current DOM value through the
existing document mutation path and publishes its acknowledged sequence only
in a completed clean, retained or full `PageFrame`. Deferred lifecycles retain
the preceding frame's acknowledgement. Activation subtracts reflected deltas
exactly once and preserves input received while raster was running.

Only one delta batch can be outstanding. The embedding uses the supported
non-overlapping main/pending branch from
`cc/scheduler/scheduler_state_machine.cc`: a main opportunity has explicit
completion, and a distinct pending main commit waits for activation. Impl-only
tile coverage may run concurrently with main synchronization. Input and display
continue during the main gate. Recoverable raster allocation failure preserves
the active tree, releases the failed pending gate, and allows later work to
retry; startup errors still propagate.

The host forwards typed main-thread effects and gives compositor-delta delivery
user-blocking priority before the rendering opportunity. Wheel DOM dispatch is
preserved. A disposition bit prevents a compositor-handled wheel from executing
a second main-thread default scroll; coalescers cannot combine differing
dispositions. Fallback wheel default handling is unchanged.

Property IDs can be rebuilt in one document; continuity uses native owner client
identity. Root replacement/document replacement discards old-owner deltas and
latches. A callback for a missing owner is consumed without mutating a replacement.

Relevant source chain: `cc/trees/property_tree_layer_list_delegate.cc`,
`cc/trees/layer_tree_host_impl.cc`, Blink `scrollable_area.cc`,
`paint_layer_scrollable_area.cc`, `scrolling_coordinator.cc`, and
`paint_artifact_compositor.cc`.

## Root sticky and post-layout clamping

The merged root LayoutView accepts its native viewport scroll ownership even
when authored overflow remains visible. Root-sticky constraints use that owner;
a viewport-fixed ancestor is carried into the native sticky constraint input.
A retained scroll exports both the current sticky offset and its native recorded
constraint basis together before property transforms are consumed.

`PaintLayerScrollableArea::UpdateAfterLayout` now reaches the already-translated
`DelayScrollOffsetClampScope`. Its GC-rooted queue holds real native areas rather
than unresolved external placeholders. Native offset-space min/max clamping
runs after the complete scrollbar-relayout transaction, before sticky/export.
It refreshes metadata without re-entering the mutable area, marks changed
paint properties, preserves disposed areas' saved offsets, and clears the native
clamp/origin flags.

Page reflects final sourced-element offsets from immutable fragments after
font/container-query convergence. This includes zero-range scrollers. It does
not duplicate native geometry math or invalidate completed layout. Superseded
queued requests are removed; scroll events use the existing semantic target and
request the next BeginFrame. Synchronous viewport and element scroll reads
perform the existing measurement flush before observing the canonical offset.

## Render-pass backing requirements

Pinned `components/viz/service/display/direct_renderer.cc` builds a complete
current render-pass requirements map before drawing; Software/Skia renderers
remove obsolete backings. A missing backing cannot be treated as undamaged.

Renderer now reconciles `(SurfaceId, RenderPassId)` against all passes for each
surface represented in the validated complete aggregate. Only obsolete keys
for represented surfaces are released. Current roots, undamaged child passes,
and omitted retained surfaces remain alive. Omitted surfaces are released only
through the existing explicit `InvalidateSurface` lifecycle. Recreated, resized,
format-changed or failed backing pixels are invalid and force full repaint,
regardless of an empty incoming damage rectangle.

`BackingStats` and optional `RECHROM_RENDERER_BACKING_DIAGNOSTICS` report retained
pass count, composed/scratch allocation capacity, clip-cache accounting and
frame-local temporary outputs. These are component measurements, not total RSS.
They cannot identify the cause of an external SIGKILL by themselves.

## Validation scope

Focused tests cover main-frame serialization/completion, impl-only pending work,
allocation-error recovery, delta acknowledgement/clamping/owner replacement,
wheel-disposition coalescing, native sticky transforms, layout shrink/expansion,
render-pass churn/reuse/recreation and multiple surface lifecycle. Production
protocol captures additionally cover actual programmatic and precise-wheel
scrolls, fixed/sticky/clipped layers, changed layout and return to top.

Existing unrelated whole-target test compilation failures remain explicit;
scoped passes are not a whole-workspace green claim. Actual page PNGs are compared
without resizing, crops or tolerance using the repository RGBA-union pixel tool.
