# Repository instructions

Rechrom is a source-aligned Rust translation of Chromium (Blink, cc, Viz,
Skia) organised into small engines with typed boundaries. This file is the
contract for working in it: how the pipeline is structured, the rules every
change must follow, and how to validate work.

Contents:

1. [Architecture](#1-architecture): the pipeline, each boundary's interface,
   each stage's internal workflow, side engines, data-flow rules.
2. [Engineering rules](#2-engineering-rules): Chromium alignment, boundaries,
   integration gaps.
3. [Validation](#3-validation): check tiers, browser verification, pixels,
   performance, scrolling.
4. [Build](#4-build) and [commits](#5-commit-messages).

The type and method names below are examples of the current design, not a
frozen ABI. Small refinements to names, fields, ownership or where a stage is
split are allowed when evidence requires them, provided the main ideas hold:
clear responsibility, one-way dependencies, minimal typed interfaces,
platform-independent engines, and no backward reach through the pipeline.
Update this file when a justified refinement changes the durable contract
instead of preserving obsolete wording.

---

## 1. Architecture

### 1.1 Pipeline

```text
URL / input
  -> Loader      resource, url_loader, document_loader, open, html
  -> DOM         dom, cssom
  -> Style       style_*
  -> Layout      layoutng, layout_*
  -> Paint       prepaint, paint_*
  -> Compositor  compositor_*        (layerization, layers, tiles, raster scheduling)
  -> Viz         viz                 (surface aggregation, display scheduling)
  -> Renderer    renderer, renderer_skia
  -> host render target / display
```

`browser::Page` is the assembly root. It owns the document-side engines,
routes their mutations and effects in this order, and passes each engine's
typed resident output to the next boundary. Page contains lifecycle
orchestration only, never copied Style/Layout/Paint/tile/raster algorithms.
The application embedding (`apps/rechrom`) supplies platform services,
BeginFrame sources, thread placement, event pumping, render targets and
presentation.

### 1.2 Boundary interfaces

Boundaries 1–4 run on one Page document sequence and pass **borrowed live
owners**: no copied trees. Boundaries 5–8 can cross threads and pass
**immutable shared values**.

| # | Boundary | Interface | How data crosses |
|---|---|---|---|
| 1 | Loader → DOM | `url_loader` request/response bytes feed `open`'s `HTMLDocumentParser`; edits are `dom::DOMMutation`; navigation lifecycle is `open::OpenMutation` → `OpenEffect`. | The parser builds into the Page's shared `Rc<RefCell<DOM>>` owner. |
| 2 | DOM → Style | `DocumentStyleEngine::Update(owner: &mut impl StyleDocument, &MediaValuesCachedData, UserAgentStyleSheetInput) -> StyleUpdateImpact` | Style reads the live DOM through `StyleDocument` and stores each `ComputedStyle` owner on its element. |
| 3 | Style → Layout | `LayoutEngine::ApplyMutation(LayoutMutation<'_>)` then `Layout()`; `TreeUpdate(&mut dyn FnMut(&mut LayoutTreeUpdate))` lends Layout's tree builder for one synchronous attach transaction. | DOM's `LayoutTreeBuilder` attaches layout objects in place, sharing the same `ComputedStyle` owner. |
| 4 | Layout → Paint | `PaintEngine::Paint(&LayoutPaintOutput, EditingPaintState)`; `LayoutPaintOutput` wraps a `LayoutResultHandle`, walked with `PaintFragmentCursor`; resources arrive as `PaintMutation` / `PaintResourceUpdate`. | Paint walks the resident fragments and their attached Paint sidecars in place. |
| 5 | Paint → Compositor | `CompositorEngine::Commit(Arc<PaintArtifact>, CompositorViewportState) -> CompositorOutput`; other changes arrive as `CompositorMutation`. | Immutable, revisioned `PaintArtifact` behind an `Arc`. |
| 6 | Compositor → Viz | `CompositorFrame { metadata, render_pass_list, resource_list }`, plus frame-scoped raster read leases; Viz returns `FrameSubmission` with returned resources. | Immutable value per frame. |
| 7 | Viz → Renderer | `Aggregate(DeviceRect, &[SurfacePlacement]) -> VizOutput` (an `AggregatedFrame` of `AggregatedRenderPass`es). | Immutable aggregate per frame. |
| 8 | Renderer → display | `HostOutput::Render(&mut VizOutput, B::Target) -> (RenderUpdate, B::Target)`; backend injected through `DirectRendererBackend`. | Borrowed host render target; the host presents. |

Borrowing across boundaries 1–4 is deliberate: it is how Chromium's in-process
data flow keeps its performance. A boundary violation is code that goes
**around** an interface (reaching into another engine's private state, OS
calls inside an engine, copying an engine's tree), not code that borrows
through it.

### 1.3 Stage internals

Each stage is split into packages. The order below is execution order;
dependencies point toward the stage's shared model package and never back
into the stage facade. Each package boundary is also a performance-attribution
boundary: the package owns its hot algorithms, its typed per-frame metrics and
a trace category named after it, so profiles identify the stage before
lower-level investigation. A facade may aggregate those values but must not
collapse them into one opaque timer.

**Loader**: `document_loader` (navigation, commit) → `fetch_policy`
(CSP/CORS/mixed content) → `url_loader` / `http_transport` (bytes) →
`resource` (identity, cache) → `decode` / `image_decoder` / `document_image`
(subresource decode) → `open` (parser feeding, scripts) → `html` parser → DOM.
These packages own fetching, resource identity, decode dispatch, cancellation
and parser feeding. They never mutate Layout, Paint, tiles, Renderer or native
windows.

**DOM** (`dom`, `cssom`): input is typed DOM/CSSOM mutation; output is
retained document state plus the minimal invalidation/mutation facts consumed
by Page, Style, Layout and script bindings. DOM supplies immutable event-path
snapshots from its tree/slot owner (per-scope retargeting, closed-root
visibility); Interaction owns dispatch lifetime and Window routing; bindings
only project those values. DOM owns mutation-observer registration and
script-free interest selection; WebAPI owns JavaScript records, callback
delivery and its microtask queue. DOM hosts Chromium's `LayoutTreeBuilder`
through the boundary-3 transaction. It does not otherwise call downstream
engines and does not own computed layout or pixels.

**Style**: `style_parser` (CSS syntax) → `style_rules` (RuleSet indexes) →
`style_invalidation` → `style_walk` (flat-tree recalc traversal) →
`style_match` (selector matching) → `style_resolve` (cascade →
`ComputedStyle`) → `style_diff` → `style_commit` (store on element, deferred
effects). Shared model: `style_model`.

- Input borrows the live DOM/CSSOM facts, pending changes, page environment,
  stylesheet owners and the small font or container inputs it needs. Do not
  pass `&mut DOM`, a Layout engine or general callbacks.
- During element recalc: compare the old `ComputedStyle`, build the new one,
  store that same owner on the element, compute Chromium's diff and reattach
  result, then give Layout the same owner in the same order. Keep unchanged
  style identity.
- No Style-owned result store, no delayed document-wide publish step, no
  second layout-style model. Extra boundary values are only for facts that are
  not in `ComputedStyle`.

**Layout**: `LayoutMutation` applied by `layoutng_assembly` (attach, style
binding) → `layout_invalidation` → `layout_reuse` (cache hits) → algorithms
(`layout_algo_block`, `_inline`, `_flex`, `_grid`, `_table`, `_multicol`,
`_float`, `_out_of_flow`, `_replaced`, `_svg`, `_forms`, `_list`) →
`layout_fragment` (resident fragment tree) → `layout_commit` (typed
publication). Shared roots: `layoutng_geometry`, `layoutng_style` (the single
`ComputedStyle` owner), `layoutng_foundation` (shared base types).

- Output is a retained `LayoutResultHandle` and borrowed `LayoutOutput` access
  to resident geometry. Paint and Interaction traverse native
  `PhysicalFragment` / `FragmentItems` pointers through cursors; fragment-local
  Paint data stays attached to those owners.
- Synchronous geometry queries read the same resident fragments and must not
  materialize or cache a parallel document-wide tree.
- Layout owns the resident LayoutObject tree and the translated LayoutNG
  algorithms. It does not parse CSS, fetch resources, record final paint
  commands, raster or present.

**Paint**: `prepaint` (property trees, coordinates, culling, invalidation) →
`paint_walk` (paint order, phases, culling) → per object `paint_reuse` (cache
and retained-state proofs) → `paint_record` (backend-neutral PaintOps) →
`paint_chunk` (DisplayItem grouping) → `paint_commit` (freeze into the
published `PaintArtifact`). `paint` is the facade and lifecycle orchestrator.
Shared: `paint_model` (property-tree, coordinate, path, low-level paint values,
layer classification), `paint_skia` (Skia path values for geometry services).
Trace categories: `prepaint`, `paint_walk`, `paint_reuse`, `paint_record`,
`paint_chunk`, `paint_commit`.

- Output is an immutable, revisioned `PaintArtifact`: display items, paint
  chunks, property state, resource-dependency indices. Paint owns no network,
  DOM/Layout mutation, tiles, raster resources or render target.
- PrePaint reads and updates the canonical fragment-local Paint sidecars owned
  by the Layout result. Keep their addresses and client identities stable
  across targeted updates; pass references plus an opaque lifetime owner. Do
  not clone `FragmentPaintData`, publish a second Paint-input snapshot, or run
  an extra whole-fragment validation walk.
- Retained PrePaint data is limited to Chromium's derived FragmentData,
  ObjectPaintProperties, PaintLayer, invalidation and cache state. It must
  never become another copy of Layout's geometry or fragment payload.

**Compositor**: two loops driven by `compositor_scheduler` (BeginFrame state
machine; commit, activation and draw action selection).

- Main-frame commit, when a `PaintArtifact` arrives: `compositor_layerize`
  (Blink `PaintArtifactCompositor` / `PendingLayer`, Paint → cc properties) →
  `compositor_tree` (commit into the pending tree).
- Impl frame, each BeginFrame: `compositor_input` (impl scroll/pinch) and
  `compositor_animation` tick → `compositor_tile` (tilings, priority,
  PrepareTiles) → `compositor_raster` (raster task graph and completion
  accounting) → `compositor_resource` (pooling, budget, export) → activation
  (`compositor_tree`) → `compositor_draw` (draw properties, damage, occlusion,
  render surfaces, quads) → `compositor_frame` (immutable `CompositorFrame`).
- `compositor` is the Page-facing facade. Shared roots `scroll_snap` and
  `scroll_animation` live under `crates/compositor/`, carry only
  source-aligned values and algorithms, and never own Page or facade
  lifecycle. Trace categories are `compositor.<package>`; packages report
  their own work counts.
- Each instance belongs to exactly one Page and consumes only that Page's
  `Arc<PaintArtifact>`, viewport, input, BeginFrame and resource completions.
  It has no toolbar/content roles, tab collection or other Page recordings.
  There is no separate `LayerTileEngine` or `FramePlan` lifecycle between
  Paint and Compositor.
- Active/pending layer and tile state stay private. Raster execution and pixel
  caches are injected services; the compositor owns their task selection,
  completion accounting and teardown policy. Viz/Renderer never take its layer
  or tile owners. It owns frame scheduling and compositor scrolling, not raster
  playback, surface aggregation or presentation.
- Typed main-thread effects carry wheel disposition and acknowledged
  impl-scroll deltas; Page returns a completed frame's acknowledgement.
  Main-frame completion and pending-tree activation gate commits without
  blocking impl input/display or impl-only tile coverage.

**Viz** (`viz`): surface manager → aggregator (`Aggregate`) → display
scheduling. Input is `CompositorFrame`, surface lifecycle, BeginFrame/deadline
and swap acknowledgements; output is a validated aggregate and swap decision.
Pure state and logic: no Skia, window, IOSurface, GPU or OS dependency.

**Renderer** (`renderer`, `renderer_skia`): `HostOutput::Render` → occlusion
culling → `DirectRenderer::DrawFrame` (damage, allocation requirements,
clipping, BSP ordering) → injected backend. `renderer` never reads backend
pixel maps, never depends on Skia, Raster or a native window, and never drives
Page, DOM, Style, Layout or Paint backward. `renderer_skia` implements the
backend with the translated SoftwareRenderer: CPU pass backings, Canvas
execution, sampling/blending/filters, Picture playback, writing into a
borrowed CPU target. Its dependency on `renderer` is one-way; Page constructs
and injects it. No GPU backend exists yet. Native surface allocation and
swap/present are host responsibilities.

### 1.4 Side engines

- **`event_loop`**: input is opaque `EventLoopMutation<Task, BeginFrame>` plus
  an injected `EventLoopExecutor`; output is `EventLoopEffect` and
  `WakeRequest`. Selects turns and rendering opportunities. No Page, message
  pump, OS wait, clock source, DOM or script semantics.
- **`interaction`**: input is host `InputEvent`, immutable document
  interaction state and the current fragment snapshot; output is
  `InteractionOutput` (`DOMMutation`, `InteractionStateMutation` or an
  interaction effect). Order: `interaction_input` (host input values,
  frame-aligned coalescing) → `interaction_hit_test` (read-only target,
  cursor, caret, text-position queries over the borrowed fragment cursor) →
  `interaction_dispatch` (DOM Event values, paths, listener propagation) →
  `interaction_default` (activation, focus, controls, editing, selection) /
  `interaction_scroll` (blocking wheel regions, main-thread scroll update).
  `interaction` is the Page-facing facade and synchronous transaction owner;
  Page keeps lifecycle and compositor-route decisions. These packages never
  invoke Style, Layout, Paint, Compositor or Renderer, never retain a copied
  interaction tree, never buffer synchronous DOM effects past listener
  reentry, and never depend back on the facade.
- **`javascript` / `webapi`**: input is an explicit task, event or host
  binding call; output is script-visible values and typed DOM/CSSOM/resource
  effects. The runtime owns no rendering lifecycle, Layout, Paint or
  presentation. File API values, native Intersection/Resize Observer
  mechanisms and WebGL API state are dependency-root packages under
  `crates/webapi/`; DOM, Layout, Interaction and the facade may depend on their
  narrow values without depending on the `webapi` facade. Platform GPU
  implementations are separate backend packages.
- **`font_engine`**: input is immutable decoded faces plus typed
  font-rule/loading and versioned system-query mutations; output is retained
  ShapeResult/metrics or explicit host requests. Owns selection, lazy face
  admission, fallback and shaping caches (HarfRust, read-fonts). Platform
  discovery, file I/O and OS facts are supplied by the embedding through
  `SystemFontProvider` and completed outside compute; `font_engine` makes no
  OS calls. Style may announce the final FontDescription as a pure cache miss;
  Page completes it before Layout. The synchronous Layout adapter may collect
  glyph-specific fallback misses; Page completes them and retries privately
  through existing invalidation gates before publishing fragments. Inline
  layout still owns item analysis, bidi, line breaking and boundary
  reshaping.
- **`image_decoder`**: encoded bitmap bytes → immutable bitmap frames. Owns no
  DOM, Style, Layout, Paint, timeline, tile or Renderer.
- **`document_image`**: input is `DocumentImageMutation` (container state,
  BeginFrame time); output is `DocumentImageEffect` with immutable, revisioned
  document-paint frames. Owns the isolated SVG Style/Layout/Paint lifecycle and
  converts its own engine failures at its boundary. Never owns Renderer, Page,
  a timer or an OS thread.
- **`raster`**: input is immutable raster work selected by Compositor; output
  is immutable raster resources and typed completions. Owns the
  backend-neutral raster-command vocabulary, Skia replay (typefaces through
  CoreText on Apple, Fontations elsewhere) and tile pixel resources. Never
  owns layerization, layers, tilings, tile priority, activation, DOM, Style,
  Layout, Paint, Page, Viz or the native window.

### 1.5 Page domains and the embedding

- Each `browser::Page` is an independent domain. Pages never reference or
  invoke each other or share mutable document/rendering state. The embedding
  owns their execution placement, lifetime, input routing and surface
  embedding.
- Everything before `CompositorFrame` stays in that Page's runtime: its
  document task scheduler and executor (DOM/JS/Style/Layout/Paint), compositor
  sequence and raster state. The embedding may inject shared finite worker
  pools (like Chromium's process worker pools), but Page state, sequencing,
  cancellation and outputs stay independent.
- The browser host owns only typed handles and policy values. It never calls
  a Page synchronously or waits for its JavaScript. Page outputs (frames,
  title, errors) reach the host as values; the host does not walk a Page's
  DOM.
- Shared physical BeginFrame delivery is an enqueue-only clock fanout. Each
  visible Page schedules frames independently; a blocked Page cannot stall
  another Page or the window's Viz/Renderer deadlines.
- Returned errors and unwind panics from a Page's document, compositor or
  raster executor terminate only that Page's producer. Emit a typed PageError
  (view/document identity, URL, stage, kind, terminal state); native and
  headless hosts both write one-line structured `rechrom-page-error` JSON to
  stderr. Window/display failures stay on the separate global Fatal boundary.
- Browser chrome and popups use the same Page presentation contract;
  window-level Viz/Renderer consume their frames without reaching back into
  Pages.
- Platform code (fonts, MIME, clipboard, input, vsync, presentation) lives in
  the embedding layer (`browser` platform modules, `apps/rechrom`) as small
  per-OS modules with equivalent behaviour on every supported platform.
  Engines receive platform facts as typed values. Platform interfaces are
  declared directly against the OS API (as the Fontconfig, CoreText and
  DirectWrite hosts do) rather than through extra binding crates.

### 1.6 Data-flow rules

These apply to every boundary above.

- Keep engines independent. An engine may depend on shared input and output
  types, never on another engine's internal objects, storage, caches or crate
  internals. Once input has crossed an interface the receiver works from its
  own state. No broad `&mut` access to the sender, no large trait mirroring the
  sender's object model, no callbacks or getters that reach back into it. Each
  engine stays testable and replaceable on its own.
- An interface only separates packages; it must not replace Chromium's data
  flow. Keep the same owner, object identity, update order, lifetime,
  invalidation, caches and commit points. If an interface cannot carry that
  flow, change the interface, not the algorithm. Page may route values but
  never makes an engine's decisions.
- Borrow live data and pass the existing shared owner where identity matters.
  Never copy whole trees or engine objects, build a second general model,
  reparse values, delay results, repeat work or add another full walk.
- Element and LayoutObject retain and read the same `ComputedStyle` owner. No
  Rechrom-only flattened layout style, no full style copy at a boundary. Keep
  Chromium's copy-on-write, anonymous-style adjustment, used-value calculation
  and derived layout data at the corresponding lifecycle points. Small
  sidecars are allowed only for document resource facts Chromium does not
  store in `ComputedStyle`.
- Keep Chromium's element-by-element work element-by-element unless there is
  evidence that a batch has the same order, result and cost. A small interface
  must preserve both Chromium's behaviour and its hot-path performance.
- Preserve each package's established input/output contract when replacing
  its internals. Never expose RuleSet, parser, cache, scheduler or renderer
  internals across a public boundary just to connect translated code. Small
  interface refinements are fine when they express Chromium's behaviour more
  directly; when a boundary lacks information, extend it with the smallest
  platform-neutral value that carries Chromium's semantics.

---

## 2. Engineering rules

### 2.1 Translate Chromium, do not invent

- Critical engines are source-aligned translations of Chromium, not
  greenfield code. For every fix, trace the corresponding Chromium path and
  align lifecycle, invalidation, ownership, scheduling and rendering
  semantics.
- Implement the reusable class of behaviour Chromium represents, not a
  plausible behaviour that passes one page. Treat a site-specific symptom as
  evidence of a missing general mechanism: fix the whole class through the
  Chromium implementation, then verify representative cases.
- No second algorithm, page-specific compensation or parallel compatibility
  path unless Chromium has the same architectural branch and the existing
  package interface requires it. No application-specific branches.
- Platform branches follow Chromium's own platform split (for example
  `layout_theme_font_provider_win.cc`, `font_platform_data_win.cc`). Every
  supported platform (Windows, macOS; x86-64, arm64) gets a real
  implementation, not a stub, when Chromium has one.

### 2.2 When behaviour differs, assume an integration gap first

When a translated mechanism exists but Rechrom still differs from Chromium:

1. Inspect the complete production call path before writing replacement
   logic. Verify that the application actually reaches the translated code
   with its original typed ownership and lifecycle. Translated source files do
   not prove the pipeline uses them.
2. Look for the usual gaps: a missing host input; a wrong value carried by an
   existing interface; a lifecycle or coordinate conversion at a boundary; an
   incomplete translated branch; string or CSSOM projections that discard
   compiled state; adapter loops that reimplement source behaviour;
   placeholder or `Unavailable` backends; compatibility paths that bypass the
   translated RuleSet, engine, cache or invalidation owner.
3. Repair the owning module or its existing minimal typed connection. Connect
   or complete the translated mechanism, remove the bypass where practical and
   fix the whole affected class. If the translation itself is wrong or
   incomplete, finish it inside its owning package while keeping that
   package's input/output contract.

### 2.3 Iteration speed

- Be aggressive about completing coherent changes once the owning Chromium
  mechanism and production call path are understood. Do not split work into
  tiny edits or keep seeking confirmation. Speed never justifies bypasses or
  parallel algorithms.
- Do not add tests during active implementation. Reuse existing regression
  tests and fixtures; add a test only when an uncovered, concrete failure needs
  a durable guard.
- Finish a coherent slice before validating it in one focused batch. Run only
  the checks that resolve a specific uncertainty; defer broad suites until a
  stable integration point.
- Do not over-validate: choose the smallest check that catches the real risk,
  and stop once it passes. Do not rerun the same coverage through several
  commands, run broad suites for a narrow change, or add tests that repeat what
  the compiler or an existing test already proves. Documentation-only changes
  need only a text or diff check unless they change executable examples or
  generated files.
- When disk usage is high, stale `target/` caches may be evicted, but never
  artifacts used by an active build.

---

## 3. Validation

### 3.1 Check tiers

1. While editing: `cargo check` the owning crate. Some crates only type-check
   correctly through the app because of feature unification (for example
   `raster` via `-p rechrom_app`).
2. After a coherent slice: build `rechrom_app` once with
   `--profile iteration` and reuse the same `target/iteration/rechrom`
   headless process for the full DevTools correctness matrix.
3. Only after that passes: fat-LTO Release.

Iteration artifacts are not evidence for performance, final experience or
release delivery. Keep input-pipeline diagnostics behind a runtime trace switch
and extend the shared trace before the slice build instead of relinking for
each new log.

### 3.2 Browser verification

- Use a headless Rechrom process for routine automated and regression checks.
  Drive it through its DevTools endpoint (navigation, input dispatch, traces,
  inspection). Do not open or automate a visible window for these checks; it
  costs resources and disturbs repeatable measurements.
- Rechrom's DevTools HTTP transport is not full CDP merely because a method
  has a CDP-compatible name. When reporting protocol compatibility,
  distinguish matching method semantics from matching target discovery,
  WebSocket JSON-RPC transport, parameters and response shape.
- Open a visible Release window only for final user-facing verification, or
  when the behaviour specifically requires native window interaction.
- For a visible URL check, stop stale Rechrom processes, then launch the exact
  executable built in this workspace (the binary inside the macOS bundle, the
  Linux binary or the Windows `.exe`) through the command-line contract:

  ```text
  <rechrom-executable> --url https://example.com --devtools-port 0
  ```

  Never start it through an OS app-name lookup: that can pick a different
  installed build, drop the URL and silently fall back to `about:home`. After
  launch, confirm through DevTools that the Page target URL is the requested
  URL before handing the window over.

### 3.3 Pixel comparison

- Whenever visual correctness is involved, capture Rechrom and Chromium
  separately with `Page.captureScreenshot`, under the same CSS viewport,
  device-pixel ratio, URL, scroll position, loading state and animation state,
  then compare with `tools/pixel_diff.py`.
- `tools/pixel_diff.py` forms a union mask over every RGBA channel and reports
  `changed_pixels`. Never call `getbbox()` on an RGBA difference image: with
  opaque screenshots the alpha difference is zero and hides real RGB changes.
  Only `changed_pixels: 0` is a zero-pixel result, and it counts only when both
  browsers rendered the intended real page (not blank pages, error pages or
  mismatched states).
- An operating-system screenshot is never the primary visual-regression
  input (window chrome, shadows, occlusion, display scaling and WindowServer
  timing make it a different measurement). Use one only as a final check of
  native window and compositor integration.

### 3.4 Performance

- Record the exact interaction through the headless DevTools protocol,
  retrieve the Performance report or trace, and capture page screenshots when
  visual state matters. Base every optimization on recorded evidence.
- If the report cannot expose the relevant scheduling, input, lifecycle,
  raster, compositing or presentation cause, improve tracing and report
  generation first, and iterate until the tool can diagnose and verify the
  optimization.

### 3.5 Scrolling

- Reproduce scroll problems by replaying the recorded macOS trackpad stream in
  `apps/rechrom/fixtures/trackpad_scroll.jsonl`.
- Match the native page's CSS viewport and device-pixel ratio. Headless
  defaults to DPR 1; use `--force-device-scale-factor 2` for a DPR 2 display.
- Reaching the final scroll position verifies the result, not smoothness.
  Inspect frame publication/commit intervals and input latency during
  continuous motion. Headless publication does not establish physical display
  FPS.
- For strict input-to-first-visible-frame measurements: replay the fixture as
  native macOS scroll events through the normal AppKit/winit path, drive a
  high-contrast page marker directly from scroll offset, and timestamp input
  and ScreenCaptureKit frame PTS in the same monotonic clock. Randomize the
  event's VSync phase and collect enough runs to report min, p50, p95 and
  missed frames for Rechrom and Chromium under identical rendering settings.
  - SCK PTS measures WindowServer-visible presentation, not physical panel
    emission.
  - Keep the SCK callback under the frame budget: for the full-width color-row
    fixture, hash one dense vertical scanline instead of the whole page, record
    the callback's own processing time, and distinguish missing SCK samples
    from missing browser submissions.
  - A synchronous HTTP input driver can create false input gaps. Use native
    replay for visible-window measurements; treat background protocol
    injection as a downstream pipeline check only.

---

## 4. Build

- Builds set `RUSTC_BOOTSTRAP=1`: the translated Skia uses nightly language
  features on the pinned stable toolchain (`rust-toolchain.toml`).
- Requirements are the Rust toolchain plus the platform C/C++ build tools
  (Visual Studio Build Tools on Windows, Xcode command-line tools on macOS).
  Text, fonts and Unicode run on pure-Rust crates (HarfRust, read-fonts/skrifa,
  ICU4X); do not add system-library dependencies.
- Chromium and Skia sources are pinned: Chromium
  `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`, Skia
  `cacf77bdba7ba7df8ea7236d7e14b08c658ff368`. Translate from these revisions
  and cite the source file and lines next to translated code.
- On memory-constrained machines build with `-j 1` and never run two cargo
  jobs at once.

---

## 5. Commit messages

All commits follow [Conventional Commits](https://www.conventionalcommits.org/)
so they stay compatible with conventional-changelog:

```text
<type>[optional scope][!]: <description>
```

- Types: `feat`, `fix`, `perf`, `refactor`, `test`, `docs`, `build`, `ci`,
  `chore`, `revert`.
- Lowercase imperative description, no trailing period, subject of 72
  characters or fewer.
- Use a scope when it names the affected subsystem, such as `paint`,
  `layout`, `renderer` or `app`.
- Mark breaking changes with `!` and explain them in a `BREAKING CHANGE:`
  footer.

```text
perf(renderer): retain raster tiles across scroll frames
fix(layout): invalidate fragments after style mutation
feat!: rename the public crate to rechrom
```
