# Rechrom scroll-stop white-screen handoff

Updated: 2026-10-24, Asia/Singapore

## User-visible symptom

On a long MDN page, native trackpad scrolling can work for several replays and
then the window returns to white after scrolling stops. This is process death,
not merely a missing final repaint. Do not report the issue fixed until the
release browser survives repeated native fixture playback.

Test URL:

```text
https://developer.mozilla.org/en-US/docs/Web/CSS/Guides/Grid_layout/Basic_concepts
```

Canonical input fixture:

```text
apps/rechrom/fixtures/native_trackpad_scroll.jsonl
```

## Confirmed failures and fixes so far

### 1. Exact document-image container mismatch

The first failure was a Rust panic from paint validation:

```text
DocumentColorSpaceNotPublished(...)
```

The display item requested an exact 16x16 explicit-light document-image
container, while the resource only contained a 24x24 implicit-default record.
This was a top-level `kDrawMask` use in the main paint artifact, not a nested SVG
artifact.

The bypass was `Page::ApplyPendingScrollUpdates`: when retained scroll-property
reuse failed, it painted and committed internally, bypassing Page's normal
`DocumentImageContainerRequirements -> ConfigurePaintDocumentImageContainers ->
Paint` sequence. The next retained lifecycle could publish that artifact.

Current fix in `crates/browser/src/page.rs` checks every candidate frame's
document-image requirements after pending scroll updates. If a use-site lacks
an exact producer-owned variant, it excludes the retained fast path and lets
the normal full Paint pass configure and re-record it. This code is still
uncommitted.

### 2. SkSL ProgramOp / RasterPipelineOp numeric mismatch

After fixing the paint-resource panic, the process instead crashed with:

```text
EXC_BAD_ACCESS / SIGSEGV / KERN_INVALID_ADDRESS at 0
thread: page-raster-3
[null PC]
SkRasterPipeline_opts::neon_highp_stages::start_pipeline
SkRasterPipeline::run
SkRuntimeColorFilter::onFilterColor4f
SkRuntimeColorFilter::affectsTransparentBlack
SkCanvas::begin_layer_record
```

The latest confirmed browser crash report after the first alignment fix is:

```text
~/Library/Logs/DiagnosticReports/rechrom-2026-10-24-013849.ips
```

A focused test showed that a known Luma runtime effect produced this impossible
stage list before the fix:

```text
uniform_color
set_base_pointer
store_src
callback
bilinear_nx
mipmap_linear_update
bicubic_p3y
load_condition_mask
cast_to_uint_from_float
invsqrt_float
load_src
store_f32
```

The root cause is that Rust's `ProgramOp` and `BuilderOp` declarations used the
order `LOWP, SKSL, HIGHP_ONLY`, while the translated
`SkRasterPipelineOp` uses upstream's current `LOWP, HIGHP_ONLY, SKSL` order.
`Program::appendStages` follows upstream and directly casts a native ProgramOp
to `SkRasterPipelineOp`; therefore every SkSL operation after index 108 was
executing an unrelated native stage with the wrong context type. One of those
was `callback`, which jumped through a null function pointer.

Current uncommitted fix in
`crates/skia/src/sksl/codegen/SkSLRasterPipelineBuilder.rs` mechanically aligns
all native discriminants in both enums with
`crates/skia/src/core/SkRasterPipelineOpList.rs`. This is a large numeric-only
diff because hundreds of discriminants were stale. It restores the invariant
required by upstream's direct cast.

Upstream reference:

```text
https://skia.googlesource.com/skia/+/main/src/sksl/codegen/SkSLRasterPipelineBuilder.h
https://skia.googlesource.com/skia/+/main/src/sksl/codegen/SkSLRasterPipelineBuilder.cpp
```

Upstream builds both enums from `SK_RASTER_PIPELINE_OPS_ALL(M)` and directly
casts `stage.op` in `Program::appendStages`.

## Verification status

Added an uncommitted regression test:

```text
crates/skia/tests/runtime_luma.rs
```

It shares one Luma runtime filter across 8 threads and performs 10,000
single-pixel evaluations per thread. Before enum alignment, even one evaluation
SIGSEGVed. After alignment, all 80,000 evaluations pass:

```bash
RUSTC_BOOTSTRAP=1 cargo test -p skia --test runtime_luma --release -- --nocapture
```

However, the full browser still crashed around native replay 4 with the same
top-level RasterPipeline stack. Therefore the task is not complete. The focused
Luma failure is fixed, but the page reaches another runtime effect or another
context-lifetime defect.

Release build succeeds:

```bash
RUSTC_BOOTSTRAP=1 cargo build --release -p rechrom_app
```

## Active diagnostics

`crates/skia/src/effects/colorfilters/SkColorFilterBase.rs` currently contains
temporary diagnostics immediately before scalar runtime-filter execution. It
prints:

- runtime-effect stable key;
- runtime-effect source;
- final RasterPipeline stage list.

Remove these diagnostics after the remaining crash is identified.

The current diagnostic release browser is running as PID 8698 on DevTools port
9363. It was started in a unified exec session and may disappear when that
session is closed. Do not kill unrelated PID 98417, which is an older headless
red-page process.

Diagnostic browser command:

```bash
./target/release/rechrom \
  --url 'https://developer.mozilla.org/en-US/docs/Web/CSS/Guides/Grid_layout/Basic_concepts' \
  --width 1280 --height 791 --devtools-port 9363
```

Native replay command:

```bash
/private/tmp/scroll_capture rechrom 8698 \
  apps/rechrom/fixtures/native_trackpad_scroll.jsonl \
  /private/tmp/rechrom-runtime-diag-1 0
```

If PID 8698 is gone, start Rechrom in a long-lived `exec_command` session rather
than a detached short command; two detached launches exited cleanly as their
shell invocation ended and produced no crash report.

Capture the app stderr in `/private/tmp/rechrom-runtime-diag.log`. On the next
crash, inspect the last `runtime-filter-eval` block and stage dump. Also inspect
the newest crash report:

```bash
ls -lt ~/Library/Logs/DiagnosticReports/rechrom-*.ips | head
```

## Next steps

1. Replay the native fixture against PID 8698 until the process crashes or at
   least 8 complete runs survive.
2. If it crashes, correlate the final `runtime-filter-eval` stable key/source
   and stage list with the crash timestamp.
3. Verify every dumped stage is semantically expected for that SkSL source.
4. If stages are now correct, inspect that stage's context allocation and
   lifetime in `SkSLRasterPipelineBuilder::Program::makeStages` and
   `appendStages`; the remaining null call is then likely a context/function
   pointer carried by a legitimate stage rather than an opcode mismatch.
5. Keep the fix source-aligned. Do not special-case Luma, return `false` from
   `affectsTransparentBlack`, disable NEON, or swallow the crash.
6. After the second fix, rebuild Release and replay the native fixture 8 times.
   Confirm the PID survives, no new `.ips` appears, and stderr has neither
   `DocumentColorSpaceNotPublished` nor a page-raster failure.
7. Remove temporary stderr diagnostics, rerun the focused test, run
   `git diff --check`, and run formatting only on changed files. Workspace-wide
   `cargo fmt --check` currently reports an unrelated existing formatting diff
   in `crates/webapi/src/window_bindings.rs`; do not modify it as part of this
   task.

## Working tree

Expected uncommitted files at handoff:

```text
 M crates/browser/src/page.rs
 M crates/skia/src/effects/colorfilters/SkColorFilterBase.rs
 M crates/skia/src/sksl/codegen/SkSLRasterPipelineBuilder.rs
?? crates/skia/tests/runtime_luma.rs
?? HANDOFF_SCROLL_WHITE_SCREEN.md
```

No commit or push was made in this side conversation.


## Continuation: 2026-10-24, about 01:55 Asia/Singapore

### Changes

- `ProgramOp` and `BuilderOp` now reference `SkRasterPipelineOp` discriminants
  directly instead of copying numeric literals. Program extended ops begin at
  `kNumRasterPipelineHighpOps`; Builder extended ops reference Program's values.
  This mirrors the shared `SK_RASTER_PIPELINE_OPS_ALL` and extended-op lists in
  the local Chromium Skia header, and prevents the original drift recurring.
- The existing concurrent Luma regression now cycles transparent, opaque red,
  opaque green, and half-alpha blue inputs and checks the actual luminance alpha
  and zero RGB, rather than validating only transparent black.
- Kept temporary runtime-filter diagnostics: the remaining native crash has not
  been identified, so removing those diagnostics would discard the next clue.

### Completed checks

- Release build succeeds after changing discriminants to canonical references.
- Updated Luma test passes all 80,000 concurrent evaluations (6.25 seconds).
- Changed-file rustfmt check and `git diff --check` pass.
- The numeric-alignment diagnostic Release build survived 8 full fixture replays
  through the headless DevTools input route, at CSS 1280x743 and DPR 2. MDN's
  real article rendered successfully, with scrollHeight 23528 and final scrollY
  15845. This is a downstream regression check, not native acceptance.
- That headless log contains 5 Luma scalar evaluations with the expected stage
  list (`init_lane_masks`, constants/copies, `dot_3_floats`, clamp, `load_src`).
  None contains `callback`; no DocumentColorSpaceNotPublished, terminal raster
  error, or new Rechrom .ips was observed.
- A second headless 8-replay run on the newly rebuilt canonical-reference binary
  is in progress on DevTools port 9365; log `/private/tmp/rechrom-headless-final.log`.

### Native verification blocker

The old PID 8698 was stopped and replaced during diagnosis. Current native
browser is PID 11912 on port 9364, with log
`/private/tmp/rechrom-native-cont.log`. PID 98417 was not touched.

This Codex app's shell environment lacks both native event-posting and screen
capture permission: `CGPreflightPostEventAccess()` returns false, and the
canonical `/private/tmp/scroll_capture` exits with ScreenCaptureKit error -3801
before replay. A no-capture replay helper was tried, but its events cannot be
accepted as valid native playback because event-posting permission is absent.
The user has been asked to enable Accessibility and Screen & System Audio
Recording for the active Codex app. Do not claim native success until permission
is available and 8 real native fixture replays complete.

Scratch drivers are in this chat's generated workspace `work/` directory, not
in the repository. Continue to use `/private/tmp/scroll_capture` once permission
is available. The latest crash report remains rechrom-2026-10-24-013849.ips.

## Continuation: 2026-10-24, about 03:50 Asia/Singapore

### White-frame root cause and fix

The final white frame was produced by cc itself: it contained one root
background quad, while all 88 picture layers were skipped. Instrumenting the
effect tree showed the contents-root effect opacity being animated through
`1.0, 0.917, ... 0.0`; at zero, `EffectTree::UpdateIsDrawn` correctly marked the
entire root subtree undrawn.

This was the overlay scrollbar fade animation. The translated Paint property
builder did not create Blink's `HorizontalScrollbarEffect` and
`VerticalScrollbarEffect` nodes, so both scrollbar layers inherited the
contents-root effect. `ScrollbarLayerImplBase` then applied its fade directly
to effect node 1, hiding the document.

The fix adds persistent per-axis overlay-scrollbar effect roles and states in
`crates/paint/src/paint_property_tree.rs` and
`crates/paint/src/pre_paint_tree_walk.rs`, following Chromium
`paint_property_tree_builder.cc:3666-3741`. The states use each scrollbar's
real compositor element id, the pre-overflow output clip, and
`ActiveOpacityAnimation`. They participate in retained-node caches, logical
effect reparenting, snapshot canonicalization, and change-marker refresh.
`crates/paint/src/scrollable_area_painter.rs` now records each scrollbar under
its matching effect. During the fade, the contents-root opacity remains 1 and
the final frame keeps the document quads.

Also retained the exact Chromium ordering correction in
`ComputeLayerClipAndVisibleRect`: install the current clip on the layer before
calling `LayerVisibleRect`, because that calculation reads the layer's clip
fields.

### Final checks completed

- Removed temporary compositor layer/effect diagnostics and the Skia
  runtime-filter stage dump.
- Clean Release build succeeds.
- Concurrent Luma regression passes all 80,000 evaluations.
- Eight headless fixture replays on the fixed Release binary completed; every
  post-scroll screenshot remained nonblank with RGB extrema `0..255` in all
  channels.
- Changed-file `rustfmt --check` and `git diff --check` pass.

### Native acceptance blocker at that point

ScreenCaptureKit capture now works, but Accessibility event posting is still
disabled: `CGPreflightPostEventAccess()` returns `false`. The canonical helper
reports all 917 fixture inputs and captures the real window, but the profiled
browser remains at `root_scroll_y=0.000` and all captured frame hashes are
identical, proving the posted wheel events were not delivered. Native 8-run
acceptance was therefore pending until Accessibility could be enabled for the
active Codex app. These capture-only attempts were not counted as native
replays.

## Continuation: 2026-10-24, about 02:53 Asia/Singapore

### Native acceptance completed

Accessibility was enabled for ChatGPT Second and
`CGPreflightPostEventAccess()` returned `true`. The canonical
`/private/tmp/scroll_capture` helper then replayed the 917-event native trackpad
fixture eight consecutive times against the final Release browser (PID 18475).

All eight runs completed with 917 recorded inputs. They captured 730, 909, 874,
826, 778, 781, 792, and 764 frames respectively. Every run had hundreds of
unique frame hashes, different first and final hashes, and a nonwhite final
scanline with RGB extrema `0..255`. The browser accepted 7,169 compositor wheel
callbacks during the session, including 6,881 compositor-owned callbacks, and
executed 6,506 DOM wheel dispatches. The previously static capture behavior is
gone.

The browser remained alive through all eight runs and was stopped normally
after verification. No panic, fatal error, segmentation fault,
`DocumentColorSpaceNotPublished`, terminal page-raster failure, or new Rechrom
crash report appeared. This completes the native acceptance that was previously
blocked by macOS permissions.

Artifacts are `/private/tmp/rechrom-native-accept-permitted-{1..8}.frames.jsonl`,
matching `.inputs.jsonl` files, and `/private/tmp/rechrom-native-accept.log`.
The `root_scroll_y` presenter profiling field stayed at zero despite accepted
wheel callbacks and changing captured content; it is not a valid scroll-offset
oracle for this compositor-owned path.

## Continuation: 2026-10-24, compositor scroll latency audit

The latency investigation kept Chromium's scheduling semantics intact. A
discarded experiment that skipped PaintArtifact synchronization reduced work
but failed the Release `scroll_frame_lifecycle` regression tests (duplicate
async default application and incorrect sticky/fixed pixels); it was fully
reverted.

The first DevTools workload was also discarded as evidence because it sent
only `changed` wheel events. All 111 events reported `compositor_owned=0` and
therefore measured the Blink/main-thread fallback rather than compositor
scrolling.

A corrected visible Release run sent a complete `began`/`changed`/`ended`
gesture. All 102 callbacks reported `compositor_owned=1`. For the first input,
the trace recorded compositor dispatch at 0.38 ms, frame preparation beginning
at 5.11 ms, compositor/window submission at 9.20 ms, and the following VSync at
12.11 ms. This matches Chromium's `InputHandlerProxy` and cc scheduler contract:
late input admitted before the 0.333 scroll deadline joins the current frame;
input after that deadline waits for the following frame.

The canonical native helper then replayed a recorded trackpad gesture against
the same visible build. All 73 traced wheel callbacks were compositor-owned and
the first changed compositor frame was submitted about 13 ms after the gesture
began. The helper's ScreenCaptureKit stream cannot be used as a first-pixel
latency oracle: around that input it had no captured frame for 383 ms before
the gesture and did not deliver its next sample until 71 ms afterward. Earlier
75-95 ms estimates were capture gaps, not browser pipeline latency.

No latency code change was retained. The translated path already connects
`DidStartScroll`/`RenewTreePriority`, `SMOOTHNESS_TAKES_PRIORITY`,
`WAIT_FOR_SCROLL`, `SetWaitingForScrollEvent(false)`, and the Viz/window
deadline. Changing the deadline ratio, synthesizing a frame from the input
callback, or bypassing PaintArtifact synchronization would diverge from the
local Chromium source without evidence of a missing connection.

Trace artifacts:

```text
/private/tmp/rechrom-mdn-visible-phase-trace.json
/private/tmp/rechrom-mdn-visible-phase-smoothness.json
/private/tmp/rechrom-mdn-native-latency-trace.json
/private/tmp/rechrom-mdn-native-latency.frames.jsonl
/private/tmp/rechrom-mdn-native-latency.inputs.jsonl
```
