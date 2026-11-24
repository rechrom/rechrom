# S4 design: classify the compositor packages and layerize's dependencies

Status: implemented (S4-1..S4-5, 2026-11-24, on top of `5245ff02`) with the
recommended decisions S4-D1 (b) `compositor_values`, S4-D2
`compositor_tree_sync`, S4-D3 keep `compositor_layerize`, S4-D4 move
`gfx::Transform` (no SkRegion was dragged along, so no fallback), S4-D5 roots
under `crates/compositor/`. Changes against the plan below:

- `compositor_input` also dropped `compositor_scheduler` in S4-2 (all its
  uses were base time); the facade is the only `compositor_tree_sync`
  importer, so `compositor_tree` keeps no stubs for the moved modules.
- S4-4a kept layerize's `compositor_input` edge until S4-4c, because the
  ScrollbarTransform facet impl moved with `gfx::Transform`.
  `TilingRasterSource` moved in S4-4b with the other values.
- Extra cuts the plan did not list: `TraceValue` (and its `From<TimeDelta>`),
  base `SourceLocation` (carried by `BeginFrameArgs`), `MemoryAllocation`,
  `TileMemoryLimitPolicy` (named by `ManagedMemoryPolicy`), the full
  `surface_id` module, and `FloatClipRectTransform`'s impl for
  `gfx::Transform`. The `SharedImageFormat` `RasterFormat` impl moved to
  `compositor_raster` (coherence).
- The moved tree-sync spans use category `compositor.tree_sync` and names
  `Compositor.TreeSync.*`; the facade's `Compositor.Tree.Commit` /
  `Activate` spans are unchanged.
- Test G (`layerize_names_only_shared_modules`) existed S4-1..S4-4b and was
  deleted in S4-4c; both allowance constants are empty and removed in S4-5.

Written 2026-11-23 against main `7f978395`. Stage S4 of the live-property-tree plan. `MAIN_IMPL_SPLIT.md`
("Compositor sub-crate items used by layerize (decide in S4)") lists the
items that need a decision. This document turns that list into a
classification of every package, a move plan, the AGENTS.md wording, and a
boundary-test extension that enforces the classification.

Data sources: every `Cargo.toml` under `crates/` and `apps/` (dependencies and
reverse dependencies below), and `grep` of `crates/layerize/src` for each
package path. Chromium terms refer to revision
`6c1d401fcca5e1b0030563a90c2f2bba168e0c15`.

## 0. Classes

Chromium compiles cc as one component. Which side owns a piece depends on
the class that uses it, not on a build target. The classification follows
the Chromium **source file** of each module:

| Class | Meaning | Chromium examples | May `layerize` depend on it? |
|---|---|---|---|
| **S**: shared cc/gfx/viz value root | Values and algorithms in files that both `LayerTreeHost` and `LayerTreeHostImpl` use, or plain gfx/viz/base values. Owns no lifecycle, thread or engine state. | `cc/trees/property_tree.cc`, `{transform,effect,clip,scroll}_node.h`, `synced_property.h`, `cc/animation/*` (one AnimationHost class, main and impl instances), `cc/paint/*`, `cc/base/{region,math_util}`, `cc/input/{main_thread_scrolling_reason,overscroll_behavior,scroll_snap_data,layer_selection_bound,touch_action_region}.h`, `cc/trees/{compositor_commit_data,browser_controls_params,layer_tree_settings}.h`, `viz/common/surfaces/local_surface_id.h`, `viz/common/frame_sinks/begin_frame_args.h`, `ui/gfx/*` | yes |
| **I**: cc impl package | Code that only `LayerTreeHostImpl`, `LayerTreeImpl`, `ProxyImpl`, the Scheduler, tiles, raster or draw run. | `layer_tree_impl.cc`, `layer_tree_host_impl.cc`, `tree_synchronizer.cc`, `cc/scheduler/*`, `cc/tiles/*`, `cc/raster/*`, `cc/resources/*`, `damage_tracker.cc`, `occlusion_tracker.cc`, `cc/input/{input_handler,scrollbar_controller,...}.cc` | no |
| **M**: cc main / Blink layerization | Code that only the main thread runs: `LayerTreeHost`, `ProxyMain`, cc `Layer`, Blink `PaintArtifactCompositor` and helpers. Lives in or under `crates/layerize/`. | `layer_tree_host.cc`, `proxy_main.cc`, `cc/layers/layer.cc`, `third_party/blink/renderer/platform/graphics/compositing/*` | it is layerize |
| **F**: facade | The Page-facing engine package of its side. | `compositor` (impl), `layerize` (main) | `compositor`: never |

Rules that follow:

1. `layerize` (and every **M** package) depends only on **S** packages,
   other **M** packages and non-compositor crates.
2. **S** packages depend only on other **S** packages and non-compositor value
   crates (`paint_model` for gfx geometry, `browser_tracing`, `skia`,
   `foundation`, `image_resource`). They never depend on an **I** package.
3. **I** packages may depend on **S**, other **I** packages and
   `layerize::commit` (through the facade only, as today), never on an **M**
   package.
4. Crates outside the compositor family (Blink-side `animation`, `layout*`,
   `interaction_scroll`, `paint_model`, `browser`, `raster`, `renderer*`,
   `viz`) depend only on **S** packages or on a facade.

## 1. Dependency facts (7f978395)

Forward dependencies, compositor family and layerize only (non-compositor
dependencies omitted except where relevant):

| Package | Depends on |
|---|---|
| `scroll_snap` | (none) |
| `scroll_animation` | scroll_snap |
| `compositor_tree` | scroll_snap, browser_tracing, paint_model |
| `compositor_layerize` | compositor_tree, browser_tracing |
| `compositor_raster` | browser_tracing |
| `compositor_draw` | compositor_tree |
| `compositor_animation` | scroll_animation, compositor_scheduler, compositor_tree |
| `compositor_input` | scroll_snap, compositor_animation, compositor_scheduler, compositor_tree |
| `compositor_tile` | compositor_draw, compositor_raster, compositor_tree |
| `compositor_scheduler` | scroll_animation, compositor_draw, compositor_tile, compositor_tree |
| `compositor_resource` | compositor_raster, compositor_scheduler, compositor_tile, compositor_tree |
| `compositor_frame` | compositor_draw, compositor_resource, compositor_scheduler, compositor_tile, compositor_tree, image_resource |
| `layerize` | compositor_tree, compositor_input, compositor_draw, compositor_animation, scroll_animation, compositor_scheduler, compositor_tile, compositor_frame, compositor_resource, compositor_layerize, paint, paint_model, paint_skia, skia, canvas, foundation, browser_tracing, serde_json, getrandom |
| `compositor` | all of the above packages, plus layerize, paint, paint_model, paint_skia, canvas, skia, foundation |

Reverse dependencies (who names the package in a manifest):

| Package | Used by |
|---|---|
| `scroll_snap` | compositor_input, compositor_tree, scroll_animation, interaction_scroll, layout, layout_base, paint_model |
| `scroll_animation` | browser, compositor, compositor_animation, compositor_scheduler, interaction_scroll, layerize |
| `compositor_tree` | compositor, compositor_animation, compositor_draw, compositor_frame, compositor_input, compositor_layerize, compositor_resource, compositor_scheduler, compositor_tile, layerize |
| `compositor_input` | compositor, layerize |
| `compositor_draw` | compositor, compositor_frame, compositor_scheduler, compositor_tile, layerize |
| `compositor_animation` | animation (Blink), compositor, compositor_input, layerize |
| `compositor_scheduler` | compositor, compositor_animation, compositor_frame, compositor_input, compositor_resource, layerize |
| `compositor_tile` | compositor, compositor_frame, compositor_resource, compositor_scheduler, layerize |
| `compositor_frame` | compositor, layerize |
| `compositor_resource` | compositor, compositor_frame, layerize |
| `compositor_raster` | compositor, compositor_resource, compositor_tile |
| `compositor_layerize` | compositor, layerize |
| `layerize` | compositor |
| `compositor` | browser, raster, renderer, renderer_skia, viz, rechrom_app |

Because of `compositor_tile -> compositor_raster` and `compositor_scheduler ->
{compositor_draw, compositor_tile}`, layerize today links the whole impl
package set except the facade. layerize cannot be built or tested without
the impl algorithms.

What layerize names (all paths, including re-exports in
`layerize/src/{cc,adapter,commit}.rs`):

| Package | Items layerize uses |
|---|---|
| `compositor_tree` | `geometry::*`, `property_types` (BlendMode as SkBlendMode, FilterOperations, PropertyPath, PropertyTransform, ...), `property_tree` (storage base, PropertyTreesOwnerId, ...), `transform::PropertyTreeTransform`, `effect_node` (EffectTreeFilters), `effect_tree_update::EffectNodeUpdateResult`, `render_surface_lifecycle::PlanRenderSurfaceLifecycle`, `scroll_delta_collection::{CollectScrollDeltas, PullAllScrollDeltasForTesting}`, `region`, `region_backend`, `clip_node`, `transform_node`, `sticky_position_constraint`, `target_property`, `property_ids`, `viewport_property_ids`, `synced_property`, `math_util`, `commit_types`, `damage_reason` |
| `compositor_input` | `browser_controls_params`, `compositor_commit_data`, `event_listener_properties`, `input_types` (ElementId), `layer_selection_bound`, `main_thread_scrolling_reason`, `overscroll_behavior`, `page_scale_animation`, `scroll_snap_data`, `scroll_source_type`, `hit_test_opaqueness`, `touch_action`, `touch_action_region`, `scroll_hit_test_rect`, `scrollbar` (cc::Scrollbar, ScrollbarOrientation), `scroll_decision::DecideScrollBy`, `scrollbar_controller_boundary::ScrollbarTransform` (facet implemented in `commit/transform.rs:253`) |
| `compositor_draw` | `Color4f`, `damage_reason` (a compositor_tree re-export), `occlusion_context::SkBlendMode` (a compositor_tree re-export), `occlusion::SkMatrix`, `render_surface_policy::RenderSurfacePropertyChangedFlags`, the facets `occlusion::OcclusionTransform`, `occlusion_context::OcclusionTransform`, `damage_context::DamageTransform` (implemented in `commit/transform.rs:216-290`) |
| `compositor_animation` | `compositor_mutation::PropertyAnimationMutation`, `property_animation_state`, `paint_worklet::{PaintWorkletInput, PropertyValue}`, `ElementListType` |
| `compositor_scheduler` | `TimeTicks`, `TimeDelta` (re-exports of `scroll_animation`), `commit_earlyout_reason::CommitEarlyOutReason`, `begin_frame::BeginFrameArgs`, `scheduler_settings::SchedulerSettings`, `settings::LayerTreeSettingsFeatures` |
| `compositor_tile` | `tile_manager_settings::TileManagerSettings` (LayerTreeSettings), `prioritized_tile_types::ScrollOffsetMap`, the facet `picture_layer_tiling_types::TilingRasterSource` (implemented by `commit/paint/picture.rs:138`) |
| `compositor_frame` | `surface_id::{LocalSurfaceId, ContentColorUsage}`, the trait `unbounded::LocalSurfaceId` |
| `compositor_resource` | `PriorityCutoff`, `managed_memory_policy::ManagedMemoryPolicy` (LayerTreeSettings), `resource_types::SharedImageFormat` |
| `compositor_layerize` | the whole package (PAC layerization policy, `content_layer_client`) |
| `scroll_animation` | `compositor_mutation::{ScrollAnimationMutation, ScrollAnimationCompletion}` |

## 2. Classification

| Package | Chromium side | Used by today | layerize may depend? | Move? |
|---|---|---|---|---|
| `scroll_snap` | **S**: cc/input snap values and algorithms (`scroll_snap_data`, `snap_selection_strategy`, `scroll_utils`), ElementId identity | 4 compositor packages, layout, layout_base, paint_model, interaction_scroll | yes | stays (already a declared shared root) |
| `scroll_animation` | **S**: `cc/animation/scroll_offset_animation_curve*`, gfx `tween`/`cubic_bezier`, base `TimeTicks`/`TimeDelta`, Rechrom mutation values | browser, compositor, compositor_animation, compositor_scheduler, interaction_scroll, layerize | yes | stays (declared shared root) |
| `compositor_tree` | **mixed**. **S**: property-tree storage and nodes, ids, synced property, damage reason, region, math_util, property types, render-surface lifecycle and effect/scroll update helpers of `property_tree.cc`. **I**: `activation`, `commit_sync`, `commit_properties`, `property_push`, `property_tree_update`, `clip_tree_update`, `render_target_update`, `scrollbar_opacity_sync`, `layer_collections` (pending/active tree sync, activation and impl-tree updates; about 2,400 lines, self-contained, used only by the `compositor` facade) | all impl packages, compositor_layerize, layerize | yes, S modules only | becomes **S**; the nine I modules move out (S4-3) |
| `compositor_input` | **mixed**. **S**: the 16 value modules layerize names (table 1, without the `scrollbar_controller_boundary` facet) plus the inline modules they need (`browser_controls_state`, `actively_scrolling_type`, and `snap_selection_strategy`, a `scroll_snap` re-export). They depend only on compositor_tree, scroll_snap and scroll_animation (through the `compositor_animation::cubic_bezier` and `compositor_scheduler::TimeDelta` re-exports). **I**: InputHandler pieces (`scroll_begin/update/end`, `scroll_state*`, `elastic_overscroll_controller`, `browser_controls_offset_manager*`, `scrollbar_controller`, `scrollbar_animation*`, `viewport`, `hit_test_list`, ...) | compositor, layerize | not as a whole; the S part yes | values move to an S root (S4-4); the package stays **I** |
| `compositor_draw` | **I**: `draw_property_utils` impl half, damage, occlusion, render surfaces, quads. Values layerize names: Skia/gfx values (`Color4f`, `SkMatrix`), property-tree render-surface flags, and three impl facets | compositor, compositor_frame, compositor_scheduler, compositor_tile, layerize | no | stays **I**; values and facets are cut (S4-4) |
| `compositor_animation` | **S**: `cc/animation/*` serves both AnimationHost instances (`AnimationHost::CreateMainInstance` / `CreateImplInstance`, animation_host.cc:55-57,74-79) | animation (Blink keyframed curves), compositor, compositor_input, layerize | yes | stays under crates/compositor as an **S** root. Its `compositor_scheduler` dependency is only `TimeTicks`/`TimeDelta` and switches to `scroll_animation` (S4-2) |
| `compositor_scheduler` | **I**: `cc/scheduler/*`, timing history. Values layerize names: base time (really `scroll_animation`), `BeginFrameArgs` (viz value carried in `BeginMainFrameAndCommitState`), `CommitEarlyOutReason` (cc/scheduler/commit_earlyout_reason.h, named by both proxies), `SchedulerSettings` / `LayerTreeSettingsFeatures` (fields of `LayerTreeSettings`, cc/trees/layer_tree_settings.h) | compositor, compositor_animation, compositor_frame, compositor_input, compositor_resource, layerize | no | stays **I**; values cut (S4-4) |
| `compositor_tile` | **I**: `cc/tiles/*`. Values: `TileManagerSettings` (LayerTreeSettings field), `ScrollOffsetMap`, the facet `TilingRasterSource` (cc::RasterSource is created on main by `RecordingSource::CreateRasterSource`) | compositor, compositor_frame, compositor_resource, compositor_scheduler, layerize | no | stays **I**; values and facet cut (S4-4) |
| `compositor_frame` | **I**: frame data, render-frame metadata, submission. Values: viz `LocalSurfaceId`, gfx `ContentColorUsage` | compositor, layerize | no | stays **I**; values cut (S4-4) |
| `compositor_resource` | **I**: `cc/resources/*`. Values: `ManagedMemoryPolicy` (LayerTreeSettings), `PriorityCutoff`, viz `SharedImageFormat` | compositor, compositor_frame, layerize | no | stays **I**; values cut (S4-4) |
| `compositor_raster` | **I**: `cc/raster` task graph and completion accounting | compositor, compositor_resource, compositor_tile | no (not used) | stays |
| `compositor_layerize` | **M**: Blink `PaintArtifactCompositor` layerization policy (`layerize_walk`, `chunk_policy`, `pending_layer_policy`, `property_tree_policy`, `chunk_to_layer_mapper`, ...) plus cc main `content_layer_client.h` | layerize; compositor only through the dead stub `compositor/src/cc/layers/content_layer_client.rs` (declared at `cc.rs:138`, no user) | it is layerize-internal | **moves** to `crates/layerize/compositor_layerize` (S4-1); compositor drops the dependency |
| `layerize` | **F/M**: cc main side, ProxyMain, Blink compositing | compositor (commit vocabulary only); browser from S1b-7 on | n/a | stays |
| `compositor` | **F/I**: LayerTreeHostImpl, ProxyImpl, Rechrom assembly | browser, raster, renderer, renderer_skia, viz, rechrom_app | never | stays |

Observations:

- `compositor_tree -> paint_model` exists because `geometry`, `transform`,
  `enum_set` and `input_geometry` re-export the gfx geometry owner in
  `paint_model`. In Chromium, `ui/gfx` sits below both Blink and cc. Rule 2
  admits it as a value dependency; it is not a Blink engine dependency.
- `raster` names `compositor` for `runtime::canvas_child_record`, which is a
  re-export of `layerize::commit::paint::canvas_child_record`. After S4 it
  could name `layerize::commit` directly. This is out of scope and not needed
  for the rules above.
- The three facets in `compositor_draw` / `compositor_input` and
  `TilingRasterSource` in `compositor_tile` are impl-algorithm traits that
  layerize implements for its own values (`commit::transform::Transform<R>`,
  `commit::paint::picture::RasterSourceStorage<R>`). The orphan rule forces
  the impl to sit next to either the trait or the type. Section 3 covers the
  options.

## 3. Where the cut values go

Decision S4-D1 (section 7) is the home of the values cut from I packages.
Options:

- **(a) Fold into `compositor_tree`**, which becomes the single cc shared
  root ("cc trees and commit values"). It adds no new package. All candidate
  modules already depend only on compositor_tree, scroll_snap and
  scroll_animation, so this creates no cycle.
- **(b) A new S root `compositor_values`** (name open) under
  `crates/compositor/`, depending on compositor_tree, scroll_snap and
  scroll_animation. It keeps compositor_tree about trees and gives the
  settings, viz and input values one clear owner. The package and trace
  category exist for attribution only; it has no hot algorithm.
- **(c) Into `layerize::commit`**: the existing S1a pattern (cc/paint and
  property trees moved there). Impl packages cannot depend on layerize
  (rule 3 allows only the facade), so this works only for values that
  sub-crates do not name. It does not fit `compositor_commit_data`,
  `ElementId`, `BeginFrameArgs` and others that impl sub-crates use.

Recommendation: **(b)** for settings, viz and input values, keeping
compositor_tree's scope intact. The facets go with the value they bind:

| Facet | Today | Recommended |
|---|---|---|
| `OcclusionTransform` x2, `DamageTransform` (compositor_draw), `ScrollbarTransform` (compositor_input) | traits in I packages, impls for `Transform<R>` in `layerize/src/commit/transform.rs:216-290` | Move `Transform<R>` (gfx::Transform) and its backend trait `RoundedRectTransformBackend` from `layerize::commit::transform` into the S root. Then each I package implements its own facet for that foreign type (local trait, foreign type: allowed), and layerize no longer names impl traits. If the backend pulls `SkRegion` along (`commit::skia_region`), use the fallback: declare the four facet traits in the S root and keep the impls in layerize. |
| `TilingRasterSource` (compositor_tile) | trait in I, impl for `RasterSourceStorage<R>` in `commit/paint/picture.rs:138` | Declare the trait in the S root (cc::RasterSource is a shared cc/raster value created on main); compositor_tile re-exports it. |

## 4. Steps

Each step is one commit plus one `cargo check -p rechrom_app` and the boundary
test (`cargo test -p layerize --test layerize_boundary`). No behaviour
changes; every move is `git mv` plus re-export stubs where needed. The
boundary test arrives in S4-1 with temporary allowances, and each later step
deletes its allowance (same pattern as `LAYERIZE_ENGINE_ALLOWED`).

| Step | Change |
|---|---|
| **S4-1** Classify and move M | Add the classification table and tests (section 6) with `LAYERIZE_IMPL_DEPENDENCY_ALLOWED = [compositor_input, compositor_draw, compositor_scheduler, compositor_tile, compositor_frame, compositor_resource]` and `SHARED_ROOT_IMPL_DEPENDENCY_ALLOWED = [compositor_animation -> compositor_scheduler]` (the only Shared -> Impl edge today). `CLASSIFICATION` lists only existing packages; `compositor_values` and `compositor_tree_sync` are added by the steps that create them. Delete `compositor/src/cc/layers/content_layer_client.rs` and its `cc.rs:138` declaration, and drop `compositor_layerize` from `crates/compositor/Cargo.toml`. `git mv crates/compositor/compositor_layerize crates/layerize/compositor_layerize`; update the workspace member (`Cargo.toml:60`) and the layerize path. |
| **S4-2** Shared roots off the scheduler | `compositor_animation` (and `compositor_input`'s value modules) name `scroll_animation::{TimeTicks, TimeDelta}` directly and drop `compositor_scheduler`. Remove the matching allowance. |
| **S4-3** compositor_tree becomes S | `git mv` the nine I modules (`activation`, `commit_sync`, `commit_properties`, `property_push`, `property_tree_update`, `clip_tree_update`, `render_target_update`, `scrollbar_opacity_sync`, `layer_collections`) into a new I package (proposed `compositor_tree_sync`, trace category `compositor.tree_sync`; or into the existing `compositor_draw` / facade, decision S4-D2). The `compositor` facade is their only importer. |
| **S4-4a** Input values | `git mv` the compositor_input value modules (section 2 row) into the S root (decision S4-D1); compositor_input re-exports them under the old paths, so impl importers stay unchanged. layerize names the S root. Remove `compositor_input` from layerize. |
| **S4-4b** Settings and viz values | Cut `BeginFrameArgs`, `CommitEarlyOutReason`, `SchedulerSettings`, `LayerTreeSettingsFeatures`, `TileManagerSettings`, `ScrollOffsetMap`, `ManagedMemoryPolicy`, `PriorityCutoff`, `SharedImageFormat`, `LocalSurfaceId` (+ its `unbounded` trait), `ContentColorUsage`, `Color4f`, `SkMatrix`, `RenderSurfacePropertyChangedFlags` into the S root, with re-exports in the I packages. Remove `compositor_scheduler`, `compositor_tile`, `compositor_frame`, `compositor_resource` from layerize. |
| **S4-4c** Facets | Section 3 facet table. Remove `compositor_draw` from layerize. `LAYERIZE_IMPL_DEPENDENCY_ALLOWED` is now empty. |
| **S4-5** Contract | AGENTS.md text (section 5); delete the allowance constants; update `MAIN_IMPL_SPLIT.md` ("decide in S4" section) and `crates/compositor/README.md` (its package table still lists `compositor_layerize` and calls the family "the compositor"). |

S4-4a/b/c are independent of each other and can land in any order after
S4-2. S4 is independent of S1b-7/S1b-8 except for the files they touch:
avoid running S4-1 and S4-4c while `crates/compositor/src/engine.rs`,
`engine_core.rs`, `engine_assembly.rs` or `layerize/src/layerize_engine.rs`
are under refactor, because their imports change.

Verification: `cargo check -p rechrom_app`, `cargo test -p layerize --lib`,
the boundary tests, and `cargo test -p layerize` building without
`compositor_raster` in its dependency graph (`cargo tree -p layerize -e normal`
must not list any I package). No browser run is needed: no executable
behaviour changes. One Matrix smoke run at the end of S4 catches accidental
re-export shadowing.

## 5. AGENTS.md wording

§1.1 pipeline lines (current text:
"`-> Layerize    layerize            (cc main side: ...)`",
"`-> Compositor  compositor_*        (cc impl side: ...)`"):

```text
  -> Layerize    layerize, compositor_layerize   (cc main side: PaintArtifactCompositor, LayerTreeHost, commit)
  -> Compositor  compositor, compositor_*        (cc impl side: layers, tiles, raster scheduling, draw)
```

§1.3 **Layerize** paragraph: replace "(with `compositor_layerize`)" and add
the dependency rule:

> **Layerize** (`layerize`, with `compositor_layerize` under
> `crates/layerize/`): cc's main side, `LayerizeEngine`. [...unchanged list...]
> Main code is generic over `F: LayerizeFamily` only. Blink paint property
> nodes are read only here. Layerize packages depend only on the cc shared
> roots (below), never on a Compositor impl package or the `compositor`
> facade. Guard: `crates/layerize/tests/layerize_boundary.rs`.

§1.3 **Compositor**, third bullet (current: "Shared roots `scroll_snap` and
`scroll_animation` live under `crates/compositor/` ..."):

> - `compositor` is the Page-facing facade of the impl side. The cc shared
>   roots `scroll_snap`, `scroll_animation`, `compositor_tree` (cc property
>   trees and their nodes), `compositor_animation` (cc/animation, one
>   AnimationHost class for the main and the impl instance) and
>   `compositor_values` (cc settings, viz and input values named by both
>   sides) live under `crates/compositor/`. They carry only source-aligned
>   values and algorithms used by both sides, depend only on each other and
>   on value crates (`paint_model` geometry, `skia`, `foundation`), and never
>   own Page, facade or thread lifecycle. Every other `crates/compositor`
>   package is impl-only: neither Layerize nor any crate outside the
>   compositor family may depend on it. Trace categories are
>   `compositor.<package>`; packages report their own work counts.

§1.3 **Compositor**, impl-frame bullet: `compositor_tree` no longer does
activation. Replace "activation (`compositor_tree`)" with "activation
(`compositor_tree_sync`)" and "`FinishCommit` into the pending tree
(`compositor_tree`)" with "(`compositor_tree_sync` over the shared
`compositor_tree` property trees)". Adjust if S4-D2 picks another home.

§1.3 **Layout** mentions `paint_input_types` as naming values "without
depending on Layout"; no change. §1.2 is unchanged by S4.

## 6. Boundary test extension (prose and pseudocode)

Extend `crates/layerize/tests/layerize_boundary.rs`. Keep the three existing
tests. `layerize_does_not_depend_on_compositor` is subsumed by test B but
can stay as a readable special case.

```text
enum Class { Shared, Impl, Main, ImplFacade, MainFacade }

/// The S4 classification (section 2). Every package whose manifest lives
/// under crates/compositor/ or crates/layerize/ must appear exactly once.
const CLASSIFICATION: &[(&str /*package*/, Class)] = &[
    ("scroll_snap", Shared), ("scroll_animation", Shared),
    ("compositor_tree", Shared), ("compositor_animation", Shared),
    ("compositor_values", Shared),
    ("compositor_tree_sync", Impl), ("compositor_input", Impl),
    ("compositor_draw", Impl), ("compositor_scheduler", Impl),
    ("compositor_tile", Impl), ("compositor_frame", Impl),
    ("compositor_resource", Impl), ("compositor_raster", Impl),
    ("compositor_layerize", Main),
    ("compositor", ImplFacade), ("layerize", MainFacade),
];

/// Temporary, shrinking per S4 step; each entry names its removal step.
const LAYERIZE_IMPL_DEPENDENCY_ALLOWED: &[(&str /*from*/, &str /*to*/, &str /*step*/)] = &[...];
const SHARED_ROOT_IMPL_DEPENDENCY_ALLOWED: &[(&str, &str, &str)] =
    &[("compositor_animation", "compositor_scheduler", "S4-2")];
```

Helpers (same style as `crates/layout/layout/tests/dependency_boundary.rs`):
`manifests(dir)` walks `crates/` and `apps/` (skipping `target`, `.git`,
`.claude`); `package_name(manifest)` reads `[package] name`;
`dependencies(manifest)` returns the keys of `[dependencies]`,
`[dev-dependencies]` and `[build-dependencies]`, resolving
`package = "..."` renames and `path = "..."` entries to package names.

Tests:

- **A. `every_compositor_package_is_classified`.** For each manifest under
  `crates/compositor/` and `crates/layerize/`, its package name is in
  `CLASSIFICATION`. Every `CLASSIFICATION` entry has a manifest (no stale
  entries). Every `Main` package's manifest path starts with
  `crates/layerize/` (Main code lives with layerize).
- **B. `main_side_depends_only_on_shared_roots`.** For every `Main` or
  `MainFacade` package: each dependency that is a classified package must be
  `Shared` or `Main`, unless the triple `(from, to, step)` is in
  `LAYERIZE_IMPL_DEPENDENCY_ALLOWED`. The message names the dependency and
  the classification row.
- **C. `shared_roots_do_not_depend_on_impl`.** For every `Shared` package:
  each classified dependency must be `Shared`, unless the triple is in
  `SHARED_ROOT_IMPL_DEPENDENCY_ALLOWED`. Unclassified (non-compositor)
  dependencies are checked against a small value allowlist (`paint_model`,
  `browser_tracing`, `skia`, `foundation`, `image_resource`); a Shared root
  naming `paint`, `layout`, `browser` or another engine facade fails.
- **D. `impl_side_does_not_depend_on_main`.** For every `Impl` or
  `ImplFacade` package: no dependency is `Main` or `MainFacade`, except the
  single `compositor -> layerize` edge (the commit vocabulary; source-level
  use stays restricted to `layerize::commit` by the existing
  `compositor_names_only_the_commit_vocabulary`).
- **E. `outside_crates_use_only_shared_roots_or_facades`.** For every
  manifest outside `crates/compositor/` and `crates/layerize/`: each
  classified dependency must be `Shared`, `ImplFacade` or `MainFacade`. This
  admits today's `animation -> compositor_animation`, `layout* ->
  scroll_snap`, `browser -> scroll_animation`, and later `browser ->
  layerize`, and rejects, for example, `raster -> compositor_tile`.
- **F. `allowances_have_no_stale_entries`.** Each triple of both allowance
  constants is still present as a manifest dependency (as `property_node_allowance_has_no_stale_entries` does today).
- **G. (S4 transition only) `layerize_names_only_shared_modules`.** Until
  S4-3/S4-4 land, `compositor_tree` and `compositor_input` are mixed. A source
  scan of `crates/layerize/**/*.rs` (code text before `//`, as `code()` does
  today) rejects `compositor_tree::<m>` for `m` in the nine impl modules, and
  `compositor_input::<m>` for `m` outside the value modules of the section 2 row. Delete the
  test when both packages are split. It exists so that no new impl use
  appears while the cuts are in flight.

Like the existing tests, these scan text because Cargo cannot express the
rules. Parse the manifests with a minimal line scanner, as
`dependency_boundary.rs` does; a TOML dependency would be a new crate just
for a test.

## 7. Open decisions

- **S4-D1. Home of the cut values:** (a) fold into `compositor_tree`, (b) new
  shared root `compositor_values` (recommended; name open), or (c)
  `layerize::commit` where no impl sub-crate needs them.
- **S4-D2. Home of the nine impl modules of `compositor_tree`:** a new
  `compositor_tree_sync` package (recommended: keeps a trace category for
  commit/activation work), or the `compositor` facade's `cc/trees` (fewer
  packages, but the README forbids the facade from absorbing algorithms).
- **S4-D3. `compositor_layerize` name after the move:** keep the package
  name and its `compositor.layerize` trace category (recommended for S4,
  since traces and handoff tools use it), or rename to `layerize_policy`
  with a `layerize.*` category.
- **S4-D4. Facets:** move `gfx::Transform<R>` into the shared root so each
  impl package implements its own facet (recommended if
  `RoundedRectTransformBackend` does not drag `SkRegion` along), or declare
  the facet traits in the shared root and keep the impls in layerize.
- **S4-D5. Shared roots' location:** keep them under `crates/compositor/`
  (recommended; matches the current AGENTS.md wording for `scroll_snap` /
  `scroll_animation`), or move all cc shared roots to a neutral
  `crates/cc_shared/` so that "everything under `crates/compositor/` is
  impl-only" becomes a directory rule, as Layout's is.
