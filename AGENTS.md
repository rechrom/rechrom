# Repository instructions

## Commit messages

All commits must follow [Conventional Commits](https://www.conventionalcommits.org/) so they remain compatible with conventional-changelog.

Use this form:

```text
<type>[optional scope][!]: <description>
```

Allowed types are `feat`, `fix`, `perf`, `refactor`, `test`, `docs`, `build`, `ci`, `chore`, and `revert`.

- Write the description in lowercase imperative form without a trailing period.
- Keep the subject concise; aim for 72 characters or fewer.
- Use a scope when it makes the affected subsystem clearer, such as `paint`, `layout`, `renderer`, or `app`.
- Mark breaking changes with `!` and explain them in a `BREAKING CHANGE:` footer.

Examples:

```text
perf(renderer): retain raster tiles across scroll frames
fix(layout): invalidate fragments after style mutation
feat!: rename the public crate to rechrom
```

## Browser verification

Use a headless Rechrom process for routine automated testing and regression
verification. Connect to its DevTools endpoint to navigate pages, dispatch
input events, collect traces, and inspect results. Do not open or automate a
visible browser window for these checks, because it consumes extra resources
and can interfere with repeatable measurements.

For screenshot and pixel-fidelity comparisons, capture the page through the
browser's page screenshot protocol. Use `Page.captureScreenshot` for Rechrom
and the same CDP method for Chromium. Capture both browsers separately with the
same CSS viewport, device-pixel ratio, URL, scroll position, loading state, and
animation state, then compare the returned page pixels directly. Do not use an
operating-system screenshot as the primary visual-regression input: window
chrome, shadows, occlusion, display scaling, and WindowServer timing make it a
different measurement. A system screenshot may be used only for a final check
of native window and compositor integration.

Rechrom's current DevTools HTTP transport is not full CDP merely because a
method has a CDP-compatible name. When reporting protocol compatibility,
distinguish matching method semantics from matching target discovery,
WebSocket JSON-RPC transport, parameters, and response shape.

For performance work, use the headless DevTools protocol to record the exact
interaction, retrieve the Performance report or trace, and capture page
screenshots when visual state matters. Base optimizations on the recorded
evidence. If the report cannot expose the relevant scheduling, input,
lifecycle, raster, compositing, or presentation cause, improve the tracing and
report generation first and iterate on it as needed so that the performance
tool can diagnose and verify the optimization.

Use `apps/rechrom/fixtures/native_trackpad_scroll.jsonl` to replay a recorded real macOS trackpad event stream when reproducing and diagnosing scroll-experience problems.

For strict input-to-first-visible-frame measurements, replay that fixture as
native macOS scroll events through the normal AppKit/winit entry path, drive a
high-contrast page marker directly from scroll offset, and timestamp input plus
ScreenCaptureKit frame PTS in the same monotonic clock domain. Randomize the
event's VSync phase and collect enough runs to report min, p50, p95, and missed
frame counts for Rechrom and Chromium under identical rendering settings. SCK
PTS measures WindowServer-visible presentation, not physical panel emission.
Keep the SCK callback below the frame budget: for the full-width color-row
fixture, hash one dense vertical scanline instead of the full page, record the
callback's own processing time, and distinguish missing SCK samples from
missing browser submissions. A synchronous HTTP input driver can also create
false input gaps; use native replay for visible-window measurements and treat
background protocol injection as a downstream pipeline check only.

Match the native page's CSS viewport and device-pixel ratio when reproducing
scroll performance; headless defaults to DPR 1, so use
`--force-device-scale-factor 2` for a DPR 2 display. Reaching the final scroll
position after settling verifies the result, not smoothness. Inspect frame
publication/commit intervals and input latency during continuous motion;
headless publication does not establish physical display FPS.

Open a visible Release window only for final user-facing experiential
verification, or when the behavior being tested specifically requires native
window interaction.

For a visible URL verification, launch the exact Rechrom executable for the
current platform and pass the requested address through the cross-platform
command-line contract, for example:

```text
<rechrom-executable> --url https://example.com --devtools-port 0
```

Resolve `<rechrom-executable>` to the artifact built in this workspace (for
example the executable inside a macOS bundle, the Linux binary, or the Windows
`.exe`). Stop stale Rechrom processes first. Do not use an operating-system app
name lookup to start the browser: it can select a different installed build
without forwarding the URL and silently fall back to `about:home`. After launch,
use the cross-platform DevTools endpoint to confirm that the Page target URL is
the requested URL before handing the window to the user.

## Core engineering principles

### Iteration speed

Do not frequently add new tests during active implementation. Prefer reusing
existing regression tests and reproduction fixtures. Finish a coherent round of
fixes before validating it in one focused batch; do not add a new test for each
small edit. Add a regression test only when an uncovered, concrete failure needs
a durable guard.

During active implementation, prioritize making coherent code changes over
repeatedly running tests. Run only the focused checks needed to resolve a
specific uncertainty, and defer broad or repeated test suites until the change
has reached a stable integration point. Tests should validate a completed
slice of work rather than interrupt each small edit, because excessive test
runs reduce iteration speed.

When disk usage is high, stale `target/` build caches may be evicted, but do not remove artifacts used by active builds.

1. Use the repository's browser pixel-comparison workflow whenever visual
   correctness is involved. Capture Rechrom and Chromium through
   `Page.captureScreenshot` under identical viewport, DPR, URL, scroll,
   loading, and animation conditions, then run the pixel-diff tool. A strict
   zero-pixel result counts only when both browsers rendered the intended real
   page successfully; blank pages, error pages, and mismatched page states do
   not count.
   Use `tools/pixel_diff.py` for the strict comparison. Do not call
   `getbbox()` directly on an RGBA difference image: when both screenshots are
   opaque, the alpha-difference channel is zero and can hide real RGB changes.
   The comparison must form a union mask from every RGBA channel and report the
   changed-pixel count; only `changed_pixels: 0` is a zero-pixel result.
2. Preserve the existing architectural and crate boundaries. Keep modules
   independent of their thread placement, communicate through typed
   interfaces, maintain one-way dependencies, and expose the smallest interface
   that expresses the required behavior. Do not solve a local problem with
   cross-layer calls, application-specific branches, or unnecessary public API.
3. Match Chromium's logical behavior and design intent for each fix. Trace the
   corresponding Chromium path and align lifecycle, invalidation, ownership,
   scheduling, and rendering semantics. Do not introduce a plausible custom
   behavior merely because it passes one page or test; implement the reusable
   class of behavior represented by Chromium. Treat a site-specific symptom as
   evidence of a missing general mechanism: use the corresponding Chromium
   implementation to identify and fix the whole class of problems, then verify
   representative cases. Preserve Rechrom's existing crate boundaries,
   one-way dependency graph, minimal typed interfaces, and application-level
   assembly while doing so; alignment does not justify cross-layer shortcuts,
   duplicated ownership, or site-specific branches.
4. When a mature Chromium mechanism has already been translated but Rechrom
   still shows a correctness or performance failure that Chromium does not,
   inspect the complete production call path before writing replacement logic.
   Verify that the application actually reaches the translated implementation
   with its original typed ownership and lifecycle. Look specifically for
   string or CSSOM projections that discard compiled state, adapter loops that
   reimplement source behavior, placeholder or `Unavailable` backend methods,
   and compatibility paths that bypass the translated RuleSet, engine, cache,
   or invalidation owner. Treat these as evidence of a systemic integration
   gap: connect or complete the translated mechanism, remove the bypass where
   practical, and fix the whole affected class while preserving the existing
   crate boundaries and minimal interfaces. Preserve each package's established
   input/output contract while replacing its internals; do not expose RuleSet,
   parser, cache, scheduler, or renderer implementation details across a public
   boundary merely to connect translated code. Preserve the semantic contract,
   dependency direction, and ownership boundary rather than freezing the exact
   Rust type shape: small interface refinements are allowed when they express
   Chromium's behavior more directly. Do not assume that translated
   source files imply that the production pipeline is using them.
5. Treat Rechrom's critical engines and browser subsystems as source-aligned
   translations of Chromium, not as greenfield implementations, while keeping
   Rechrom's refactored package boundaries and established input/output
   interfaces as the default architecture. Align each package's internal logic
   and the semantics carried across a boundary; do not recreate Chromium's
   in-process object graph by exposing internal objects or adding reverse and
   cross-layer dependencies. When behavior differs, first assume an integration
   defect: a missing host input, an incorrect value carried by an existing
   interface, a lifecycle or coordinate conversion at a boundary, a translated
   branch that is incomplete, or production code that bypasses the translated
   path. Compare the corresponding Chromium call chain and repair the owning
   module or its existing minimal typed connection. Do not invent a second
   algorithm, page-specific compensation, or parallel compatibility path unless
   Chromium itself has the same architectural branch and the repository's
   existing package interface requires it. If the translated implementation is
   wrong or incomplete, finish the translation inside its owning package while
   preserving its public input/output contract; if a boundary lacks required
   information, extend that contract only with the smallest platform-neutral
   value needed to express Chromium's semantics.

## Core package contracts

These contracts describe the architectural direction, not a frozen ABI or an
exhaustive list of types. Chromium source logic is translated inside the owning
package; `browser::Page` is the assembly root that routes typed values between
them. Small changes to names, fields, ownership, or where a stage is split are
allowed when implementation evidence requires them, provided the result keeps
the same main ideas: clear responsibility, one-way dependencies, minimal typed
interfaces, platform-independent engines, and no backward reach through the
pipeline. Update this section when a justified refinement changes the durable
contract instead of preserving obsolete wording in code. The concrete type and
method names below are examples of the current design, not requirements to keep
an awkward interface; follow the responsibility and data-flow intent when a
small refinement makes the implementation clearer or closer to Chromium.

```text
URL/input
  -> resource + document_loader/open
  -> DOM/CSSOM
  -> style
  -> layoutng
  -> paint
  -> compositor (owns layerization, layers, tiles, and raster scheduling)
  -> CompositorFrame
  -> viz
  -> renderer
  -> host render target
```

- `event_loop`: input is opaque `EventLoopMutation<Task, BeginFrame>` plus an
  injected `EventLoopExecutor`; output is `EventLoopEffect` and `WakeRequest`.
  It selects turns and rendering opportunities. It contains no Page, message
  pump, OS wait, clock source, DOM, or script semantics.
- `resource`, `url_loader`, `document_loader`, and `open`: input is navigation
  or resource requests and immutable bytes/stream events; output is typed
  document/resource/open effects. They own fetching, resource identity,
  decoding dispatch, cancellation, and parser feeding. They do not mutate
  Layout, Paint, tiles, Renderer, or native windows.
- `image_decoder`: input is encoded bitmap bytes; output is immutable bitmap
  frames. It owns no DOM, Style, Layout, Paint, timeline, tile, or Renderer.
- `document_image`: input is explicit `DocumentImageMutation` including
  container state and BeginFrame time; output is `DocumentImageEffect` carrying
  immutable, revisioned document-paint frames. It owns the isolated SVG
  Style/Layout/Paint lifecycle, but never Renderer, Page, a timer, or an OS
  thread.
- `dom` and `cssom`: input is typed DOM/CSSOM mutation; output is retained
  document state and the minimal invalidation/mutation facts consumed by Page,
  Style, Layout, and Script bindings. DOM also supplies immutable event-path
  snapshots from its existing tree/slot owner, including per-scope retargeting
  and closed-root visibility; Interaction owns dispatch lifetime and Window
  routing, and Script bindings only project those values. DOM owns mutation-observer
  registration and script-free interest selection; WebAPI owns JavaScript records,
  callback delivery and its microtask queue. DOM does not directly
  call downstream engines and does not own computed layout or pixels.
- `javascript`/`webapi`: input is an explicit task, event, or host binding call;
  output is script-visible values and typed DOM/CSSOM/resource effects. The
  runtime does not own rendering lifecycle, Layout, Paint, or presentation.
- `interaction`: input is host `InputEvent`, immutable document interaction
  state, and the current fragment snapshot; output is `InteractionOutput`
  (`DOMMutation`, `InteractionStateMutation`, or an interaction effect). It
  performs hit testing and default actions without directly invoking Style,
  Layout, Paint, Compositor, or Renderer.
- `style`: input is DOM/CSSOM state, stylesheet changes, and an immutable host
  environment such as viewport/media/DPR/preferences; output is Chromium
  `ComputedStyle` ownership plus the projected typed style mutation/impact used
  at the Layout boundary. It does not perform geometry, paint recording,
  raster, or presentation. The native `ComputedStyle` remains the canonical
  result; projections must not replace or reinterpret it. Shared immutable UA
  stylesheet owners cross the input boundary intact and are retained directly;
  borrowed stylesheet values remain observable by value. Thread placement must
  not erase this ownership or require recalc to deep-compare shared UA rules.
- `font_engine`: input is immutable decoded faces, typed font-rule/loading and
  versioned system-query mutations; output is retained ShapeResult/metrics or
  explicit host requests. It owns selection, lazy face admission, fallback and
  shaping caches. Platform discovery and file I/O are supplied by the embedding
  through SystemFontProvider and completed outside compute. Style may announce
  the final FontDescription as a pure cache miss; Page completes that request
  before Layout. The synchronous Layout adapter may still collect glyph-specific
  fallback misses; Page completes them and retries privately through existing
  invalidation gates before publishing fragments.
  Inline still owns item analysis, bidi, line breaking and boundary reshaping.
- `layoutng`: input is `LayoutMutation` (constraints, tree/style/text/editing or
  scroll changes) containing Style's canonical result; output is a retained
  `LayoutResultHandle` and borrowed `LayoutOutput` access to resident geometry.
  Paint and Interaction traverse native `PhysicalFragment`/`FragmentItems`
  pointers through cursors; fragment-local Paint data stays attached to those
  native owners. Synchronous geometry queries read the same resident fragments
  and must not materialize or cache a parallel document-wide tree. Native
  LayoutObject ownership stays private. It owns the resident LayoutObject tree
  and translated LayoutNG algorithms. It does not parse CSS, fetch resources,
  record final paint commands, raster, or present.
- `paint`: input is the retained native layout result plus `PaintMutation` or
  immutable resource snapshots; output is an immutable, revisioned
  `PaintArtifact` containing display items, paint chunks, property state, and
  resource-dependency indices. It does not own network, DOM/Layout mutation,
  tiles, raster resources, or a render target.
  Paint's frame execution flows from `prepaint` into a `paint_walk` that invokes
  `paint_reuse`, `paint_record`, and `paint_chunk`, then into `paint_commit`,
  with `paint` as the compatibility facade and lifecycle orchestrator.
  This execution order is not a dependency chain: `paint_model` is the shared
  dependency root; `prepaint`, `paint_reuse`, and `paint_chunk` depend toward
  that root; `paint_record` consumes their typed outputs; `paint_walk` drives
  `paint_record`; and `paint_commit` consumes completed record/chunk values.
  `paint_model` owns shared property-tree, coordinate, path, low-level paint
  values, and the layer
  classification needed while PrePaint builds its retained tree;
  `prepaint` owns property/coordinate/invalidation preparation; `paint_walk`
  owns traversal-order execution plus phase and culling decisions;
  `paint_reuse` owns cache and retained-state proofs; `paint_record` produces
  concrete backend-neutral PaintOps; `paint_chunk` groups semantic DisplayItems;
  and `paint_commit` freezes completed values into the published artifact.
  Dependencies point
  toward model packages and never back into the `paint` facade.
  Each stage owns a typed per-frame metrics value and a same-named trace
  category (`prepaint`, `paint_walk`, `paint_reuse`, `paint_record`,
  `paint_chunk`, `paint_commit`). Keep hot algorithms and their counters in
  the owning package so CPU profiles, trace duration, and work-volume ratios
  identify the stage before lower-level investigation. The facade may aggregate
  those values, but must not collapse them into one opaque Paint timer.
- `raster`: input is immutable raster work selected by Compositor from its
  resident picture layers and tiles; output is immutable raster resources and
  typed completion values. It owns Skia replay and tile pixel resources. It
  never owns layerization, layers, tilings, tile priority, activation, DOM,
  Style, Layout, Paint, Page, Viz, or the native window.
- `compositor`: each instance belongs to exactly one Page domain and consumes
  that Page's immutable `Arc<PaintArtifact>` directly, together with its local
  viewport, input, BeginFrame, and resource completions. It translates Blink's
  `PaintArtifactCompositor`/`PendingLayer` path and Chromium's `cc::Layer`,
  `PictureLayer`, tiling, tile-priority, damage, pending/active-tree, and raster
  scheduling logic. There is no separate `LayerTileEngine` or `FramePlan`
  lifecycle between Paint and Compositor.
  It consumes only that Page's local state. It has no toolbar/content roles,
  tab collection, or other Page recordings.
  Its active/pending layer and tile state remains private to that instance;
  the host gives each Page an independent raster executor and routes immutable
  `CompositorFrame` output plus immutable frame-scoped raster read leases to
  the window. Raster execution and pixel caches are injected services; the
  compositor owns their task selection, completion accounting, and teardown
  policy. Viz/Renderer never take the compositor's layer or tile owners. Output
  is typed raster work, lifecycle effects, and a `CompositorFrame`. It owns
  active/pending frame scheduling and compositor scrolling, but not raster
  playback, surface aggregation, or platform presentation. Typed
  implementation is partitioned into `compositor_layerize`, `compositor_tree`,
  `compositor_tile`, `compositor_raster`, `compositor_draw`, `compositor_frame`,
  `compositor_resource`, `compositor_scheduler`, `compositor_input`, and
  `compositor_animation`, with `compositor` as the Page-facing facade. Each
  package boundary is also a performance-attribution boundary: traces use the
  matching `compositor.<package>` category and package owners report their own
  work counts instead of the facade reconstructing private state. Typed
  main-thread effects carry wheel disposition and acknowledged impl-scroll
  deltas; Page returns a completed frame's acknowledgement. Main-frame
  completion and pending-tree activation gate commits without blocking impl
  input/display or impl-only tile coverage.
- `viz`: input is `CompositorFrame`, surface lifecycle, BeginFrame/deadline, and
  swap acknowledgements; output is a validated aggregate render pass and swap
  decision for Renderer. It owns surface aggregation and display scheduling as
  pure state/logic, with no Skia, window, IOSurface, GPU, or OS dependency.
- `renderer`: input is Viz's immutable aggregate/frame data and a backend-owned
  render target; output is rendered damage/resource usage and completed output.
  It owns translated DirectRenderer traversal, damage, allocation requirements,
  clipping decisions and BSP ordering. The embedding injects `RendererBackend`
  at construction. It never reads backend pixel maps or depends directly on
  Skia, Raster, or a native window, and never drives Page, DOM, Style, Layout,
  or Paint backward.
- `renderer_skia`: implements `RendererBackend` using the translated
  SoftwareRenderer and renderer utilities. It owns CPU pass backings, Canvas
  execution, sampling/blending/filter execution, Picture playback capability
  and writes into a borrowed CPU target. Its dependency on `renderer` is
  one-way; Page constructs and injects `RendererSkiaBackend`. No GPU backend is
  currently implemented. Native surface allocation and swap/present remain
  host responsibilities.
- `browser::Page`: is an independent domain. Pages never reference or invoke
  each other or share mutable document/rendering state. The embedding owns
  their execution placement, lifetime, input routing, and surface embedding.
  Everything before `CompositorFrame` stays in that Page's runtime: its own
  document task scheduler and executor (DOM/JS/Style/Layout/Paint), compositor
  executor, and raster executor/pools. The browser host owns only typed handles
  and policy values; it must not synchronously call a Page or wait for its JS.
  Shared physical BeginFrame delivery is an enqueue-only clock fanout. Each
  visible Page receives and schedules frames independently; a blocked Page
  cannot stall another Page or the window's Viz/Renderer deadlines.
  Returned errors and unwind panics from a Page's document/compositor/raster
  executor terminate only that Page's producer. Emit a typed PageError event
  with view/document identity, URL, stage, kind, and terminal state; native and
  headless hosts both write one-line structured `rechrom-page-error` JSON to
  stderr. Keep window/display failures on the separate global Fatal boundary.
  Browser chrome and popups use the same independent Page presentation
  contract; window-level Viz/Renderer consume their frames without reaching
  back into Pages. Page owns and assembles the document-side engines, routes their
  mutations/effects in the order above, and passes each engine's typed resident
  output object to the next boundary. Page contains lifecycle orchestration, not copied
  Style/Layout/Paint/tile/raster algorithms. The application embedding supplies
  platform services, BeginFrame sources, thread placement, event pumping,
  render targets, and presentation.
