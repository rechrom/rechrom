# Skia CPU in Rust

This crate owns the browser CPU raster implementation. It depends on Rust utility
crates and has no C/C++ build step. macOS glyph rasterization continues to use
system CoreText/CoreGraphics. This is a CPU subset, not complete Skia.

## Source mapping

Official responsibility owners use the checked-out Skia directories and basenames,
with `.rs` replacing `.h`/`.cpp`. Canonical types include `SkPath`, `SkPathBuilder`,
`SkMatrix`, `SkBitmap`, `SkPixmap`, `SkCanvas`, `SkShader`, `SkRasterPipeline`,
and the gradient/edge types. Legacy Rust names remain compatibility aliases.
Migration-only `raster_*`/`path_*` wrapper namespaces have been removed.

Implementation ownership follows local official source: contour measurement is in
`SkContourMeasure`; cap/join factories in `SkStrokerPriv`; 26.6 arithmetic in
`SkFDot6`; analytic edges in `SkAnalyticEdge`; rectangle strokes in
`SkScan_Hairline`/`SkScan_Antihair`; Gaussian planning in `SkMaskBlurFilter`;
two-point conical gradients in `SkConicalGradient`; pipeline op declarations and
contexts in `SkRasterPipelineOpList`/`SkRasterPipelineOpContexts`.

`compat/` explicitly owns local replay commands, frame storage/readback, F16
accumulation, normalized color representations, pixmap convenience drawing,
image/glyph adapters and the custom PNG encoder. These do not correspond to
complete official classes. `extensions/` contains local shader compatibility,
font HVGL and the specialized gradient replay driver.

`translation_map.json` inventories every source file and the internal declarations,
with official source evidence or explicit local-adapter classification. A canonical
name occurrence proves the source name exists; it does not prove identical APIs,
representation or complete algorithms. Rust signatures, borrow rules and helper
functions still differ. In particular, matrices are affine six-component values,
paths lack native conic verbs/COW, and bitmaps/views use fixed packed RGBA storage.
There is no complete GPU backend or C++ `SkSurface` implementation here.

The directly migrated tiny-skia 0.12.0 and tiny-skia-path 0.12.0 algorithms retain
their copyright headers and licenses (`LICENSE.tiny-skia`,
`LICENSE.tiny-skia-path`). They are source code in this crate, not Cargo dependencies.

## Browser boundary

Renderer converts `paint::paint_engine::DisplayItemList` to
`compat::commands::DrawCommand` one command at a time and replays it through
`SkCanvas`. Borrowed resource bytes avoid a second full display list.
`compat::surface::RasterSurface` is the CPU frame/readback interface consumed by
rechrom_app, which owns winit and window presentation. The optional native C++
reference backend stays in renderer for pixel comparisons.

This restructuring preserves full redraw and normal DEBUG optimization settings.
Existing unit tests, native differential fixtures and a full frame comparison
verify the migration; they do not establish complete upstream feature coverage.

## Click redraw translations (2026-10-02)

Profiling the existing click fixture at 2560×1542, scale 2, in normal
`opt-level=0` DEBUG identified an opaque readback conversion, gradient span
iterator overhead, vertical analytic-edge per-pixel coverage, and scaled glyph
outline fallback. These paths now use an opacity reduction with ownership
transfer, a pointer span driver, full-coverage row spans, and the positive
uniform `SkScalerContextRec::computeMatrices(kVertical)` branch. macOS font
resizing preserves logical optical size through the restricted
`SkCTFontCreateExactCopy` algorithm. Invisible glyph bounds reject bitmap
generation before CoreText rasterization; this is an explicit adapter to
upstream `paintMasks` clipping, not a full strike cache implementation.

Source hashes, restricted branches, before/after timings and native differential
validation are recorded in `artifacts/click-render-20261002` and referenced by
`translation_map.json`. The first three changes preserve all 14,807,040 bytes
of the captured click frame. Scaled glyphs deliberately change the preceding
outline fallback, verified against native glyph replay. Full-page native pixel
parity still has pre-existing differences and is not claimed. Font variations and CTFont selection are prepared once per glyph run, matching
the upstream run-level scaler/strike responsibility; the Rust prepared view is
lifetime bound to the existing font owner. Full redraw and DEBUG optimization
settings are retained.

## Click redraw round 2

`SkRasterClip` now records nonzero bounds and opaque-rectangle classification.
The existing dense coverage mask remains; `from_mask` is an explicit Rust
adapter, while known integer rectangle intersections maintain bounds directly.
Canvas save/restore and offscreen clips retain matching metadata. Span blitters
and gradients use rectangular clip bounds without rescanning their coverage.
General AA clips keep their exact coverage bytes and existing rounding rules.

The opaque two-stop gradient driver specializes the official highp stages and
precomputes the unchanged dither stage's exact period-eight values per span.
It retains FMA, tile origin and packing; the translucent driver is unchanged.

Ordinary integer RRect drawing collects horizontal intervals from the same
analytic tiles, preserving direct-event order, gutter exclusion and pair
rounding. `RunBasedAdditiveBlitter::snapAlpha` is applied once per accumulated
interval; direct and small-mask values retain their old semantics. This removes
the full-frame coverage allocation/copy/reconstruction. Internal tile storage
is still dense, and F16/clipping consumers retain mask output: it is an adapter
to official blitter responsibilities, not a complete upstream RLE implementation.

Source evidence, byte oracles, native comparisons and DEBUG click measurements
are recorded in `artifacts/click-render-round2-20261002/results.json`.

DEBUG click optimization, round 3 (2026-10-02):

- `SkBlitter_ARGB32::blend_anti_h_prepared` and `SkBlitRow_D32::color32`
  translate the constant-color horizontal run arithmetic; `SkBlitRow_opts.rs`
  owns the matching four-pixel kernel and scalar tails.
- `SkRasterPipelineBlitter` is still a compatibility adapter. Its solid N32
  SrcOver horizontal-run/rectangle branch now selects the owned blitter, with
  clip coverage combined before the blitter as in `SkAAClipBlitter`. Other
  blends, shaders, gamma spaces, forced highp, vertical/pair/mask methods retain
  the previous pipeline. This is not a complete port of `SkBlitter::Choose`.
- Uniform bitmap erase uses the existing contiguous fill helper and keeps its
  color conversion. Gradient broadcasts are prepared once per draw in local
  fused driver state; nearest-even packing remains unchanged.
- `artifacts/click-render-round3-20261002/results.json` records CPU full-redraw
  measurements, native comparisons, and the changed pixels from replacing the
  old constant-color pipeline's rounding. No frame cache or dirty redraw.

DEBUG click optimization, round 4 (2026-10-02):

- The local fused gradient driver specializes opaque vertical two-stop gradients:
  its interpolation is constant across x, and upstream ordered dithering has an
  exact eight-pixel period. Each covered tile-row span computes those eight
  pixels through the existing stage arithmetic and fills the same span from
  their bytes. Translucent/partial/nonvertical cases retain the previous path.
  This equivalent specialization is local; upstream has no matching dispatch.
- The dense-tile interval adapter skips equal coverage/flag groups by word loads
  and handles boundaries/tails with the preceding scalar predicate. It transfers
  owned coverage into Mask. Tiles are still dense; this is not a full RLE port.
- `artifacts/click-render-round4-20261002/results.json` contains normal DEBUG
  full-frame measurements and byte/native regressions. No state persists between
  spans or frames, and this round preserves complete frame bytes.

DEBUG click optimization, round 5 (2026-10-02):

- `SkBlitMask_opts.rs` translates the ARM NEON general, opaque and black A8
  mask-to-N32 row arithmetic, including eight-pixel loads/stores and scalar tails.
  The premultiplied RGBA single-row interface adapts upstream color/layout/stride
  inputs. The portable fallback deliberately uses the ARM alpha+1/256 rules;
  it does not claim to implement upstream non-NEON Sk4px rounding.
- The Mac glyph adapter prepares color and smoothing preblend once per run,
  following `SkScalerContext::fPreBlend` ownership. It clips each glyph once,
  converts the visible BGRX rows to A8, and invokes the row blitter. Gamma and
  clip multiplication retain the preceding arithmetic; F16 remains per pixel.
  This is a limited local adapter, with no SkStrike or glyph image cache.
- `artifacts/click-render-round5-20261002/results.json` records ordinary DEBUG
  full-redraw measurements, unchanged complete frame bytes and direct upstream
  A8/glyph comparisons. No frame cache or dirty redraw is introduced.

DEBUG click optimization, round 6 (2026-10-02):

- `SkStrike.rs` and `SkStrikeCache.rs` translate lazy once-only glyph images,
  run-held strikes, and whole-strike LRU purge. Descriptors are hashed once per
  run; integer position and clip are applied after exact glyph/phase lookup.
  This is an image subset, with generic HashMap/Arc storage and exact f32
  phases, rather than the complete upstream descriptor/packed-ID/arena API.
- The Mac port retains native font owners on the rendering thread, with exact
  identity comparisons. Native-family fonts are identified by the metadata
  actually used to create CTFont, without retaining unused font-file bytes.
  Local owner limits are 32 identities/64 fonts; file-backed identities have
  a separate 2 MiB target (one oversized font can exceed it). Image accounting
  defaults to upstream's 2 MiB/2048 strikes and coarse LRU purge. A run may
  temporarily exceed the budget; allocator/key/native-font overhead and live
  external Arc handles are not a hard process-memory bound.
- The convex walker's official vertical-edge `blitAntiRect` branch emits full
  rows without writing dense coverage. Partial top/bottom/narrow branches retain
  the preceding arithmetic. The Rust tile adapter crops gutters and turns these
  events into bounded intervals. Other curved regions still use dense tile
  planes; this is not a complete RunBasedAdditiveBlitter translation.
- BW rectangular glyph clipping uses bounds only. The dense AA bridge can
  forward opaque rows unchanged. PreparedA8Blitter selects its kernel and
  broadcasts invariant color/constants once, while preserving ARM A8 rounding.
- The opaque vertical gradient driver shares its exact eight-pixel period
  between spans of a row. Final lanes use the same stage values, as upstream
  start_pipeline's patched tail. This row-local specialization is derived from
  upstream stages, not a new upstream stage or a retained frame/shader image.
- Packed-AND opacity scanning uses larger bounded NEON blocks. CPU frame storage
  takes ownership of rendered bytes without first allocating a blank copy.
  These are Rust adapters; they do not implement the entire SkSurface API.
- `artifacts/click-render-round6-20261002/results.json` records full DEBUG redraw
  timings and numeric regressions. No frame cache or dirty redraw is added.
  Whole-page equivalence with upstream remains incomplete; unchanged bytes and
  direct upstream parity are reported separately. The 16 ms target is not claimed.

DEBUG click optimization, round 7 (2026-10-02):

- Strike image/path ownership is separate, with shared accounting and lazy path
  preparation. Exact font-byte identity and derived nonhairline stroke paths
  are local adapters. Full replay retains the original fill-then-stroke order.
- The Mac glyph bridge caches actual whole-font bounds and rejects conservatively
  before image-key lookup. Actual color and variable fonts disable this shortcut,
  following the upstream bounds-validity rule. Placement and clipping are tested
  with cold/warm caches, transformations and viewport-edge positions.
- Packed A8 and opacity drivers preserve existing ARM arithmetic while avoiding
  ordinary DEBUG per-intrinsic call overhead. Release retains the NEON A8 driver;
  no-SIMD retains its scalar fallback. Bulk bool-plane zero preserves the prior
  analytic coverage/event partition exactly. These are representation adapters.
- Opaque-alpha bookkeeping starts from a fresh white allocation. Only admitted
  SrcOver operations preserve the proof; layers/masks/other operations invalidate
  it. Proven opaque frames transfer bytes without alpha conversion. Unknown-alpha
  frames retain the scan. This restricted local proof is not full SkImageInfo.
- Constant SrcOver scanline blitters defer unused shader construction, retaining
  the unchanged mask recipe when the mask entry point is actually requested.
- Additional official horizontal real-blitter span interfaces are translated
  and tested, but their exploratory production wiring was withdrawn after a
  broader coverage-partition regression. Final rounded replay keeps its previous
  event representation. They do not contribute to the reported final speedup.
- Ordinary DEBUG full redraw of the button fixture, including toolbar and content
  at 2560x1542, measures 15.70 ms median across 18 final/confirmation samples,
  versus 28.2065 ms in round 6. All samples are retained: three exceed 16 ms,
  with a maximum of 16.493 ms. This reaches the median target for this fixture,
  without guaranteeing every frame or including input dispatch/presentation.
- 180 unique Skia/renderer/browser/manual functions pass; a repeated no-SIMD
  suite also passes. Captured content bytes match round 6 exactly. Final toolbar
  captures agree across both timing runs and the native diagnostic, but no
  round-6 toolbar baseline was captured. Whole-page native comparison still
  differs at 432026 content and 696 toolbar pixels; upstream parity is incomplete.
  Full measurements, exact source hashes, mapping and limitations are recorded
  in `artifacts/click-render-round7-20261002/results.json`. No frame cache,
  dirty redraw, compiler optimization override or new assembly is introduced.

The next CPU scroll round is recorded in
`artifacts/scroll-8ms-20261002/results.json`. It adds resident ordered glyph
scan resources and contained BW-clip bypass, compact analytic tiles and straight-band rectangle coverage,
exact tiled vertical-gradient row replay, full-device BW clip identity, and
constant/full-stride Color32 fills. These adapt official CPU responsibilities;
the ordered event resources and driver folding are local representations,
not literal upstream data structures or additional raster-pipeline stages.
The window target is a CPU-mapped BGRA IOSurface managed by vendored softbuffer.
This is full replay without a framebuffer cache or debug optimization overrides.
The measurements include the ordinary layout entry and native buffer submission;
the 8 ms goal remains unmet for every measured frame. See the report for all
samples, pixel comparisons and the system-compositor timing boundary.
