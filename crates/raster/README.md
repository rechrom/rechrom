# Raster engine boundary

`raster` owns raster execution and the resources produced by it. It does not
own Paint artifacts, layerization, tilings, tile priority, task graphs, Viz
surfaces, render passes, or presentation.

The production data flow follows Chromium's `RasterBufferProvider` protocol:

```text
PaintArtifact
  -> compositor/cc selects RasterSource and tile work
  -> RasterEngine acquires an exclusive resource write session
  -> RasterBackend replays one RasterPlayback into unpublished pixels
  -> RasterEngine publishes content identity and a completion token
  -> Viz/Renderer later acquires an immutable resource lease
```

## Input

The compositor supplies resource metadata when it acquires a buffer:

- resource identity, size and pixel format;
- current and previous content identity.

It then supplies one call-scoped `RasterPlayback` containing:

- an immutable `RasterSource` selected by cc;
- content and invalid-content rectangles;
- the resolved raster transform;
- LCD/MSAA/visibility/HDR values;
- a mutable, call-scoped image provider;
- immutable raster-inducing scroll offsets.

Neither input exposes a Page, layer owner, tiling, task graph, scheduler, Viz
surface, renderer, host target, or native window.

## Output

Raster output is deliberately published in two phases. Successful playback
publishes the requested resource and new content identity. The compositor
observes readiness and its completion token. Viz/Renderer later receives an
opaque immutable `RasterResourceLease` with identity, size, format, and
opacity. Concrete storage is not part of the compositor/Viz contract. A
concrete renderer may ask `raster` to map a lease through
`MapResourceForRead`; that mapping either succeeds for the selected backend or
reports that the renderer/backend pair is incompatible. No Canvas, mutable byte
vector, cache, or backend object crosses the package boundary.

This delayed lease is the output value: returning the pixel vector directly
from playback would bypass ResourcePool ownership and permit mutation while a
frame still reads the resource.

## Backend replacement

`RasterEngine` owns allocation identity, exclusive-write admission, content
identity, completion tokens and immutable publication. Its package-private
`RasterBackend` owns the concrete drawing implementation. A backend receives
only one selected playback request and an exclusive view of one unpublished
allocation. It returns success or failure; it cannot publish pixels, replace
resource metadata, or mint content identities and completion tokens.

Playback has explicit transactional publication semantics. Dropping an
acquired buffer before playback restores the previous immutable publication.
Successful playback publishes the new content identity. Once a backend starts
writing, any error immediately ends the write session and invalidates the
resource; partially modified storage is never relabeled as old or new content.
This matches Chromium's zero-copy failure behavior: allocation or mapping
failure drops the unusable SharedImage and the tile checkerboards. Backend
implementations should therefore validate unsupported replay vocabulary before
the first pixel mutation whenever possible; the engine does not clone the full
allocation to manufacture rollback.

The sole concrete receiver implements this contract with `skia`.
`skia` remains a source-aligned Skia mapping and does not depend on Paint,
compositor, Viz, Renderer, Page, or `RasterEngine`. The adapter that implements
`RasterBackend` belongs here in `raster`.

The public allocation/playback/publication boundary is backend independent, but
the current private replay seam carries the selected `skia` region and
retained path values. Producer-authored paths are converted once when
compositor records are lowered. A standalone PaintArtifact replay utility
exists for diagnostics and PNG fixtures; production tile raster still enters
only through `RasterEngine`.

Replacement therefore proceeds behind the existing engine boundary:

1. keep cc's `RasterPlayback`, resource identity and immutable lease unchanged;
2. lower the selected paint record into a backend-neutral, call-scoped replay
   stream inside `raster`;
3. let the private backend execute that stream into the engine's unpublished
   resource;
4. keep the concrete receiver private in `RasterEngine::new`.

The browser must not choose a backend, pass a Canvas, or receive mutable pixels.
Those would leak implementation choice across the Page/compositor boundary.
