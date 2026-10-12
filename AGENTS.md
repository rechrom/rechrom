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

During active implementation, prioritize making coherent code changes over
repeatedly running tests. Run only the focused checks needed to resolve a
specific uncertainty, and defer broad or repeated test suites until the change
has reached a stable integration point. Tests should validate a completed
slice of work rather than interrupt each small edit, because excessive test
runs reduce iteration speed.

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
  -> compositor (owns layer_tile + raster scheduling)
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
  Style, Layout, and Script bindings. DOM does not directly call downstream
  engines and does not own computed layout or pixels.
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
  result; projections must not replace or reinterpret it.
- `layoutng`: input is `LayoutMutation` (constraints, tree/style/text/editing or
  scroll changes) containing Style's canonical result; output is a retained,
  immutable `FragmentNode` snapshot and related geometry indices. It owns the
  resident LayoutObject tree and translated LayoutNG algorithms. It does not
  parse CSS, fetch resources, record final paint commands, raster, or present.
- `paint`: input is the immutable fragment snapshot plus `PaintMutation` or
  immutable resource snapshots; output is an immutable, revisioned
  `PaintArtifact` containing display items, paint chunks, property state, and
  resource-dependency indices. It does not own network, DOM/Layout mutation,
  tiles, raster resources, or a render target.
- `layer_tile`: input is `Arc<PaintArtifact>` plus `FrameConfig`; output is a
  pending/active `FramePlan`, `RasterBatch`, and typed tile resource releases.
  It owns retained layer identity, tilings, tile metadata, visibility/damage,
  and activation state. It does not execute raster, own platform surfaces, or
  mutate Paint.
- `raster`: input is immutable `FramePlan`/`RasterBatch`; output is immutable
  raster resources and `RasterCompletion` values. It owns Skia replay and tile
  pixel resources. It never calls DOM, Style, Layout, Paint, Page, Viz, or the
  native window.
- `compositor`: input is immutable artifact/viewport snapshots, BeginFrame,
  input-scroll, resize, and raster-completion commands; it internally
  coordinates `layer_tile` and `raster`; output is typed effects culminating in
  a `CompositorFrame`. It owns active/pending frame scheduling and compositor
  scrolling, but not surface aggregation or platform presentation.
- `viz`: input is `CompositorFrame`, surface lifecycle, BeginFrame/deadline, and
  swap acknowledgements; output is a validated aggregate render pass and swap
  decision for Renderer. It owns surface aggregation and display scheduling as
  pure state/logic, with no Skia, window, IOSurface, GPU, or OS dependency.
- `renderer`: input is Viz's immutable aggregate/frame data plus raster
  resources and an abstract render target; output is rendered damage/resource
  usage and presentation-ready pixels. Platform surface allocation and actual
  swap/present remain host responsibilities; Renderer never drives Page, DOM,
  Style, Layout, or Paint backward.
- `browser::Page`: owns and assembles the document-side engines, routes their
  mutations/effects in the order above, and publishes immutable snapshots to
  the compositor boundary. Page contains lifecycle orchestration, not copied
  Style/Layout/Paint/tile/raster algorithms. The application embedding supplies
  platform services, BeginFrame sources, thread placement, event pumping,
  render targets, and presentation.
