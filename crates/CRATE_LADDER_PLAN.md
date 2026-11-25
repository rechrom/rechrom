# Crate ladder plan: apply the crate boundary rules (AGENTS.md 1.7)

Status: plan, not started. Written 2026-11-25 against main `4ee114c1`.
Goal: every crate dependency obeys AGENTS.md 1.7. Dependencies form a ladder
down the pipeline. No reverse, skip or root-to-engine edges. Skip-step
requests travel on the Proxy or input queue. Data crosses by move, borrow or
retained shared ownership, never by copy. No behaviour change except B5's
input routing.

## 1. Tiers

| Tier | Crates (examples) | May depend on |
|---|---|---|
| **R0 base roots** | `foundation`, `browser_tracing`, `layout_geometry` | R0 |
| **R1 platform services** (Chromium `blink/renderer/platform`, `ui/gfx`, `cc/paint`, `components/viz/common`) | `font_engine`, `image_resource`, `canvas`, `skia`, new `cc_paint`, new `viz_common`, `compositor_tree`, `compositor_values`, `compositor_animation`, `scroll_snap`, `scroll_animation` | R0, R1 |
| **R2 document value roots** | `layout_style`, `paint_input_types`, `layout_style_paint`, `paint_model` (after B4) | R0, R1, R2 |
| **Pipeline engines** (facade plus internal packages) | DOM (`dom`, `cssom`) → Style → Layout → Paint → Layerize ⇒ Compositor → Viz → Renderer | roots, own packages, **direct upstream facade only** |
| **Side engines** (AGENTS.md 1.4) | `interaction`, `javascript`/`webapi`, `document_image`, `event_loop`, `raster`, `animation` | roots and the facades their 1.4 contract names (listed per engine in the ladder test) |
| **Assembly** | `browser`, `apps/rechrom`, `document`, `document_loader`, `open`, `page_mutation` | anything |

## 2. Violation inventory (from Cargo.toml at 4ee114c1)

| Edge | What it uses | Rule broken | Fixed by |
|---|---|---|---|
| `viz`, `renderer` → `compositor` | `DrawQuad`, `CompositorRenderPass`, `SharedQuadState`, gfx math | reverse dependency | B1 |
| `renderer_skia` → `compositor` | frame types | reverse dependency | B1 |
| `compositor` → `paint` | `CpuPaintRecord`, `ContentDisplayItemList`, `CpuDrawImage`, `TargetColorParams` (cc paint values via Paint's re-export) | skip | B2 |
| `raster` → `paint`, `compositor` | `PaintRect`, `FilterOperations`, `TileMode`, `ImmutablePaintRecord`, gfx values, `PaintArtifact` | skip / reverse | B2, B3 |
| `renderer_skia` → `paint` | `FilterOperations` | skip | B2 |
| `layerize` → `paint` | property nodes, `PaintRect`, `PaintClipPath`, `FilterOperations`, `PaintTraceScope` (and the artifact) | facade used for values | B3 (artifact stays: adjacent) |
| `paint_model` → `layout` | `FragmentPaintData`, `ResidentBoxGeometry`, `StickyPositionPaintData`, svg filter geometry, `OverscrollBehavior` | root on an engine (and `compositor_tree` → `paint_model` makes an R1 root reach Layout) | B4 |
| `layout_base` → `cssom` | to be audited | skip (Layout's upstream is Style) | B6 |
| Page document code (`page.rs`, `page/scroll.rs`, `local_frame_*`, `navigation_api`, `page_scale`, `pointer_scroll_controls`, `programmatic_scroll`, `main_compositor`) → `compositor` | values (`ElementId`, `ScrollSourceType`, `TimeDelta`, `Point`, `LocalSurfaceId`); Proxy construction (`MainFamily`, `ImplToMainTask`, `ProxyImplInit`, `CreateMainHostParams`); `PropertyAnimationMutation`; **`CompositorMutation::Input(ScrollbarGesture)` from the document thread** | skip call | B5 |
| `dom` → `layout` | `LayoutTreeBuilder` hosting | allowed exception (boundary 3) | B6 (document) |

## 3. Steps

Each step is one or more commits: `git mv` plus import changes, re-export
stubs only where they keep an unrelated importer stable for that step, then
`cargo check -p rechrom_app -p layerize --tests` and the ladder test
(`--profile iteration`). Delete each step's allowances in the same commit.

| Step | Change | Done when |
|---|---|---|
| **B0** Ladder test | Add `crates/browser/tests/crate_ladder.rs` (or a workspace-level test crate). It reads every `Cargo.toml`, assigns each crate a tier from section 1, and fails on any edge 1.7 forbids that is not in `LADDER_ALLOWED` (= section 2). It has a second check: Page document-side modules must not name `compositor::` (allowed: `synchronous_presenter.rs`, `compositor_input.rs`, the impl-runner modules). A stale allowance also fails. | test passes with the section 2 allowances |
| **B1** `viz_common` | New R1 root under `crates/viz/viz_common` with `CompositorFrame`, `CompositorRenderPass`, `DrawQuad` and its material types, `SharedQuadState`, quad geometry, `ResourceId`/`TransferableResource` (Chromium `components/viz/common/quads`, `resources`). `compositor` produces, `viz`/`renderer` consume; neither names the other's facade. | `viz`, `renderer`, `renderer_skia` drop `compositor` |
| **B2** `cc_paint` | Move `layerize::commit::paint` into a new R1 root `crates/compositor/cc_paint` (Chromium's separate `cc/paint` component). `layerize::commit` re-exports it so the commit vocabulary is unchanged. `FilterOperations`, `TileMode`, records, display lists and images come from there. | `compositor`, `raster`, `renderer_skia` drop `paint`; `raster` drops `compositor` for paint values |
| **B3** Layerize ← Paint narrowed | Layerize names property nodes, `PaintRect`, `PaintClipPath` and trace scope from `paint_model` / `cc_paint`, and the `paint` facade only for `PaintArtifact` and the update entry (adjacent). Audit `raster`'s `PaintArtifact` / `DocumentPaintArtifactRecord` use: raster replays records only, so the artifact type should not reach it. | `layerize` → `paint` edge carries only the artifact handoff; `raster` has no `paint` edge |
| **B4** Flip the sidecar inversion | Design check first (one page in this doc): Chromium keeps `FragmentData` / `ObjectPaintProperties` in `core/paint` and `core/layout` includes them. Move the sidecar *types* (`FragmentPaintData`, `StickyPositionPaintData`, the paint-data part of `ResidentBoxGeometry`) into an R2 root that Layout holds (candidate: into `paint_model`, which then must not depend on `layout`). Layout stores them; Paint mutates them through the existing borrowed path (no copies; addresses stay stable per 1.3 Paint rules). `OverscrollBehavior` goes to `compositor_values`. | `paint_model` has no `layout` edge; `compositor_tree` reaches no engine |
| **B5** Page document side through channels | (a) values from roots; (b) Proxy construction from `layerize::commit`; (c) `PropertyAnimationMutation` becomes Layerize vocabulary (it is applied to `LayerizeEngine` since S1b-8); (d) a typed input queue `InputToImpl`, owned by the embedding's input router (Chromium `WidgetInputHandlerManager` posting to `InputHandlerProxy`), carries `ScrollbarGesture` and main-thread hit-test replies, moved as messages. The document thread no longer pushes `CompositorMutation::Input`. | B0's second check has no allowances |
| **B6** Remaining edges | Audit `layout_base` → `cssom`: if it is value types, move them to a root, otherwise route through Style. Document the `dom` → `layout` exception with its citation in the ladder test. Optionally audit `webapi` fan-out against its 1.4 contract. | `LADDER_ALLOWED` holds only cited exceptions |
| **B7** Contract and verify | AGENTS.md: tier list and the two channels in 1.7, the new roots in 1.3; update `crates/compositor/README.md`. One `--profile iteration` build of `rechrom_app` in `D:\rechrom`, then the runtime matrix (pixel pages, interaction, screenshots, DPR 2 sticky replay vs reference, scroller round trip, animation page, teardown, resize). | build and matrix pass |

Order: B0, then B1 and B2 (independent), then B3, B5, B4, B6, B7. B4 is the
only step with a design decision; B5 is the only behaviour-relevant step
(input delivery order must match Chromium: replies are posted, not run
inline).

## 4. Data-crossing check per step

Every moved type keeps its ownership mode. Frames and messages move. Paint
records and display lists stay `Arc`-shared. Sidecars stay borrowed in place.
No step may add `Clone` of a tree, a snapshot or a parallel model to satisfy a
new import path. Review each commit for new `.clone()` on non-`Copy` boundary
values.

## 5. Open decisions

- **L-D1** Ladder test location: `crates/browser/tests` (assembly owns it) or
  a new `tools/crate_ladder` test crate. Recommended: a small workspace test
  crate, so it doesn't depend on building `browser`.
- **L-D2** `viz_common` location: `crates/viz/viz_common` (recommended,
  matching Chromium) or under `crates/compositor/`.
- **L-D3** B4 sidecar home: extend `paint_model` (recommended, fewer crates)
  or a new `paint_sidecar` root.
- **L-D4** Input queue owner in the synchronous (in-process) presenter: the
  presenter drains it at the same point Chromium's compositor task runner
  would run it (recommended), or the Page drains it before `UpdateRendering`.
