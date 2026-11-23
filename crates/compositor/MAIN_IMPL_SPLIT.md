# Compositor main/impl thread split (live property tree, stage S1)

Status: S1c done. S1a-1, S1a-2, S1a-3 B1-B2, the blanket family impls, B3a-B3d done (see "S1a-3 status" and "Plan
correction" at the end); S1a-3 B3-B4 and the thread split (S1b/S1d) are not
implemented.

## Chromium reference

Pinned revision `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`.

- `LocalFrameView::RunPaintLifecyclePhase` -> `PaintTree` ->
  `PushPaintArtifactToCompositor`
  (`third_party/blink/renderer/core/frame/local_frame_view.cc:2965-2975,3253`)
  calls `PaintArtifactCompositor::Update(artifact, viewport_properties,
  scroll_translation_nodes, ...)` (`:3348`) on the **main thread**. Blink paint
  property nodes and PaintChunk states are read only there.
- `ProxyMain::BeginMainFrame` (`cc/trees/proxy_main.cc`,
  `DCHECK(IsMainThread())`) applies the impl deltas carried by
  `BeginMainFrameAndCommitState::commit_data`
  (`LayerTreeHost::ApplyCompositorChanges`), runs the Blink lifecycle
  (`UpdateLayers` is where PAC output reaches `LayerTreeHost`), then
  `LayerTreeHost::WillCommit` returns `std::unique_ptr<CommitState>`
  (`cc/trees/layer_tree_host.cc:424`).
- ProxyMain posts `ProxyImpl::NotifyReadyToCommitOnImpl(completion,
  std::move(commit_state), &unsafe_state, ...)`. Main then waits on the
  protected sequence (`LayerTreeHost::WaitForProtectedSequenceCompletion`,
  `cc/base/protected_sequence_synchronizer.h`) the next time it touches
  protected Layer state.
- On impl, the Scheduler selects `ScheduledActionCommit`
  (`cc/trees/proxy_impl.cc`) -> `LayerTreeHostImpl::BeginCommit` /
  `FinishCommit(*commit_state, *unsafe_state)` pushes main Layers into the
  pending tree. Then the completion event is signalled and main may resume
  mutating Layers.
- `ProxyImpl::ScheduledActionSendBeginMainFrame` calls
  `LayerTreeHostImpl::ProcessCompositorDeltas` and posts
  `ProxyMain::BeginMainFrame(std::move(begin_main_frame_state))`. Impl->main
  deltas are therefore **values** produced on impl and applied on main.
- `ProxyMain::BeginMainFrame` early-outs with
  `ProxyImpl::BeginMainFrameAbortedOnImpl(reason, ...)` when deferring or when
  there are no updates.

## Target ownership

### Main half (Page-owned, `page-document-*` thread)

`CompositorMainEngine` (new facade; name may change), created by Page next to
the PaintEngine:

- `CpuMainLayerHost` (LayerTreeHost) including `pending` CommitState and
  `ThreadUnsafeCommitState` (`runtime/main_host.rs:69-90`).
- `PaintArtifactCompositor`, `previous_artifact`, `PaintChunksToCcLayer`
  conversion and `ConvertPaintArtifactSubsetToRecord`
  (`runtime/engine_core.rs:53-57,103-113`).
- `Rc<PacScrollCallbacks>` / `PacScrollEffects`
  (`runtime/pac_scroll.rs`). All `Rc`/`rc::Weak` stay inside this half.
- `UpdateLayers`, `WillCommit`, `ActivateCommitState`, source-frame numbering
  (`engine_core.rs:488-525`).
- Main half of `ApplyCompositorDeltas` (`main.ApplyCompositorChanges(data)`
  plus `ViewportScaleChanged` / `UpdateCompositorScrollState` effects,
  `engine_core.rs:165-200`).
- Composited-animation property updates and their coalescing
  (`engine_assembly.rs:333-400`, `engine_core.rs:420-470`).
- `CanvasChildPaintRecords` publication (`engine_core.rs:320-371`) and
  `ApplyScrollbarPaintMutation` (`engine_core.rs:389-410`).
- `SetHaveScrollEventHandlers` / `SetEventListenerProperties`
  (`engine_assembly.rs:544-549`; LayerTreeHost calls from ChromeClient in
  Chromium, not impl input).
- PAC half of `ApplySourceScrollAnimation`
  (`DropCompositorScrollDeltaNextCommit`, `engine_core.rs:707-720`).
- Main-frame lifecycle deferral state (`main_frame_lifecycle.rs`): Chromium's
  `LayerTreeHost::defer_main_frame_update_` / `StartDeferringCommits` live on
  main; impl only receives `SetDeferBeginMainFrame`.
- The document-thread video/drag engine (`browser/src/page.rs:622,639`,
  `page/drag_image.rs:166`) folds into this half: it only uses
  `CreateVideoSurfaceForeignLayer` and `ConvertPaintArtifactSubsetToRecord`,
  both main-side.

### Impl half (`page-compositor-*` thread)

`CompositorEngine` keeps its current facade for impl work:
`ClientLayerTreeHostImpl`, `CpuHostServices`, Scheduler, input handler and
wheel/gesture state, tiles/raster/resources/draw/frame sink, animation host
ticks, presentation feedback. It never receives `PaintArtifact`, PAC or Blink
property nodes; it receives `CommitState` values and leases of the main
`ThreadUnsafeCommitState` only inside the protected sequence.

## Commit protocol (ProxyMain / ProxyImpl)

```text
impl Scheduler: SEND_BEGIN_MAIN_FRAME
  ProcessCompositorDeltasForPipeline(next_bmf)          (impl)
  ApplySourceAnimationEvents -> MutatorEvents value     (impl)
  post BeginMainFrame { args, commit_data, events, epoch } ----> document

document: ProxyMain::BeginMainFrame
  if deferring: post BeginMainFrameAborted(reason, applied_deltas) -> impl
  main.ApplyCompositorChanges(commit_data); animation events
  Blink lifecycle (Style/Layout/PrePaint/Paint)
  PAC::Update(artifact) + UpdateLayers + WillCommit
  state = ActivateCommitState(owner)                    (CommitState by value)
  post ReadyToCommit { state, unsafe: ProtectedLease, synced } ---> impl
  [main continues non-compositor work; first access to main Layers or
   LayerTreeHost blocks in WaitForProtectedSequenceCompletion]

impl Scheduler: NotifyReadyToCommit -> COMMIT
  BeginCommit; FinishCommit(&mut state, lease.unsafe_state, lease.main_context)
  lease.Complete()  ---------------------------------> wakes document
  MainFrameCommitCompleted effect; POST_COMMIT; ACTIVATE; DRAW
```

- `CommitState` crosses by value (it already is move-only:
  `main_host.rs:396 ActivateCommitState`).
- `ProtectedLease` is the translation of `ProtectedSequenceSynchronizer`
  for `ThreadUnsafeCommitState` and the main `LayerContext` that
  `FinishCommit` needs (`engine_core.rs:543-550`). It is a narrowly scoped
  `unsafe impl Send` wrapper over a raw pointer to the main host plus a
  `Mutex<bool> + Condvar` completion. Soundness argument: the document thread
  may not touch the main host between `ReadyToCommit` and `Complete()`;
  `EngineMainEndpoint::WaitForProtectedSequenceCompletion`
  (`main_endpoint.rs:135`) becomes a real wait instead of an assertion, and
  `IsOwnerThread`/`InProtectedSequence` become per-thread checks. Layers are
  already `Send + Sync` (`cc/layers/layer.rs:211`). `CpuMainHostEndpoint` and
  the paint value services lent to `FinishCommit` must be `Send + Sync`.
- Property-only commits (composited animation) follow the same path with
  `update_layers = false`.
- No-update main frames post `BeginMainFrameAborted(kFinishedNoUpdates)`.

## Impl -> main effects

They already cross as `CompositorMainThreadEffect` values (wheel
disposition, hit-test requests, gesture acks, presentation feedback, scroll
offset). After the split the effects generated by the main half
(`ScrollOffsetChanged` from PAC callbacks, `UpdateCompositorScrollState`,
`ViewportScaleChanged`, `CanvasChildPaintRecords`, `ScrollbarPaintFallback`,
`ScrollAnimationCommitPrepared/Finalized`) are produced directly on the
document thread and never round-trip through the compositor thread.

## BeginMainFrame scheduling

`SEND_BEGIN_MAIN_FRAME` posts `Command::Main(BeginMainFrame{..})` carrying
the commit data. `MainHostEffect::NeedsCommit/NeedsUpdateLayers`
(`main_endpoint.rs:22-30`) become a posted `SetNeedsBeginMainFrame` to impl
(ProxyMain::SetNeedsCommit -> ProxyImpl::SetNeedsCommitOnImpl). The document
thread runs Paint only inside a dispatched BeginMainFrame; artifacts produced
outside one are held and committed at the next BMF, exactly as Blink.

## Blockers for a one-pass implementation

1. **Synchronous delta pull outside a BMF.**
   `EngineAssembly::CommitWithScrollAnimations` pulls impl deltas into the
   main host when an artifact arrives without a dispatched BMF
   (`runtime/engine_assembly.rs:188-201`). With main on another thread this is
   a synchronous cross-thread round trip that Chromium never does. The fix is
   to restrict PAC Update to dispatched BMFs, which changes Page lifecycle
   scheduling for unsolicited animation artifacts.
2. **Commit admission lives on the impl thread.** `ImplPresentation`
   (`apps/rechrom/src/page_runtime.rs:1110-1215`) decides whether an artifact
   answers a BMF from `main_frame_epoch` and `required_scroll_sequence`, and
   takes that sequence from PAC `ScrollOffsetChanged` effects (`:1118-1126`),
   which are main-half output. `PageRenderPipeline`
   (`crates/browser/src/page.rs:1940-2070`) holds the `pending`/`presentable`
   state keyed to the synchronous `main_commit_pending` output. All of this
   has to move to the document thread as ProxyMain state, for both the
   internal (single-thread) and external presentation paths.
3. **Scheduler callbacks read main state.**
   `SchedulerClient::ScheduledActionSendBeginMainFrame`
   (`runtime/engine_core.rs:818-850`) calls `ApplyCompositorDeltas` (which
   mutates `main`) and reads `prepared_paint_commit` / `ready_commit`
   synchronously. `NotifyReadyToCommit`, `FinishMainFrameWithoutUpdates` and
   `AbortDeferredMainFrame` (`engine_core.rs:1009-1111`) mix main flags
   (`awaiting_main`, `ready_commit_notified`) with impl calls. Each needs a
   main-side mirror updated by posted messages.
4. **The protected sequence assumes one sequence.**
   `EngineMainEndpoint::EnterProtectedSequence` asserts
   `RunsTasksInCurrentSequence` (`runtime/main_endpoint.rs:114`) and
   `WaitForProtectedSequenceCompletion` asserts instead of waiting (`:135`).
   `CpuMainHostEndpoint` (`runtime/main_host.rs:54`) has no `Send + Sync`
   bound.
5. **Direct composited-animation readiness.** Page gates
   `CompositedAnimationProperties` on
   `external_compositor_published_frame_sequence` (`browser/src/page.rs:9984`)
   and routes it to the compositor thread (`:10248`). It becomes a local call
   into the main half.

Points 1 and 2 change when the document thread runs Paint relative to
BeginMainFrame. Scroll-offset rollback regressions are documented in exactly
that code (`page_runtime.rs:1199-1203`, `page.rs:2019-2025`), so the change
needs headless scroll-fixture verification (AGENTS.md 3.5) that one blind pass
cannot give.

## Proposed sequence

- S1a: make `CpuMainHostEndpoint` and the paint services lent to
  `FinishCommit` `Send + Sync`; replace the single-sequence protected-sequence
  asserts with a real synchronizer (no behaviour change while both halves
  share a thread).
- S1b: split `EngineCore` into `MainCore` / `ImplCore` behind the existing
  facade, with the BMF/ReadyToCommit/Aborted messages as in-process values
  (still one thread). The SchedulerClient only touches `ImplCore`.
- S1c: done (PaintRecord leases hold only `items`, `resources` and
  `image_content_revisions`).
- S1d: move `MainCore` into Page on the document thread, move
  `ImplPresentation` admission into Page as ProxyMain state, run Paint only
  in dispatched BMFs, verify with the trackpad fixture and the DevTools
  matrix.

## S1a progress: the `layerize` crate

Agreed shape: `crates/layerize` (package `layerize`, engine `LayerizeEngine`)
is the main half; `compositor` depends on it and names only
`layerize::commit`; `layerize` never depends on `compositor`. The guard is
`crates/layerize/tests/layerize_boundary.rs`. Its property-node allowance
lists the compositor files that still hold main-half code; remove each entry
when its file moves.

Done:

- The crate, with `layerize::commit` containing
  `protected_sequence_synchronizer` (moved from `cc/base`, the compositor path
  re-exports it) and re-exports of the `synced_property`, `property_ids` and
  `viewport_property_ids` values from `compositor_tree`.

What still blocks moving the main-half files:

- **The type family spans both halves.** `MainLayerTypes` has the full impl
  family `LayerTreeTypes` as a supertrait (`src/adapter/main_layer.rs:19-32`).
  `LayerTreeTypes` (`src/adapter/layer_tree_impl.rs:131-260`) names
  `LayerTreeImpl`, draw/occlusion/damage contexts, `TileManager`, frame sink
  and so on. Every main generic (`Layer<B>`, `PaintArtifactCompositor<B>`,
  `CommitState<B>`, `PropertyTreesFor<B>`) is bounded through it.
- **One concrete family serves both halves.** Concrete main code names
  `CpuCompositorTypes<R>` (`src/runtime/types.rs`, the `Root<R>` alias in
  `runtime/paint_services.rs` and `runtime/paint/*`), which implements the
  impl traits too.
- **The push target creates impl objects.** `MainLayerImpl::Create` takes
  `&mut LayerTreeImpl<B>` (`src/adapter/main_layer.rs:304-310`). The other
  `MainLayerImpl` setters, which write through `B::ImplContext`, already form
  the push-target trait. `Create` has to become a factory method on
  `MainLayerImplTree` (the `ImplContext` trait).

Resolution (one pass of its own):

1. Add `layerize::commit::CommitTypes` with the associated types the main half
   uses: `Transform`, `Filters`, `Path`, `RegionBackend`, `CopyOutputRequest`,
   `SurfaceRange`, `ViewTransitionRequest`, `SelectionBound`,
   `UIResourceRequest`, `Layer: MainLayerImpl`, `ImplContext`. Move
   `MainLayerImpl` / `MainLayerImplTree` (without `Create`), `CommitState`,
   `ThreadUnsafeCommitState`, `PropertyTreesFor` and `cc/trees/property_tree.rs`
   with it.
2. Make `MainLayerTypes: CommitTypes` and `LayerTreeTypes: CommitTypes`, with
   the impl-only bounds restated in `LayerTreeTypes`.
3. Split `CpuCompositorTypes<R>` into a main family in `layerize` (which
   implements `CommitTypes` and the main traits) and the impl family, which
   names the main family for commit values.
4. Move the files listed in "Main half" with `git mv`.

### S1a-1 status (commits bde07277..HEAD)

Done:

- `layerize::commit` holds the plain commit values, `LayerTreeDebugState`, the
  cc property trees and their service traits, `CommitState` /
  `ThreadUnsafeCommitState` / `CommitTimestamps`, `CommitTypes` and
  `CommitStateTypes`.
- `CommitTypes` declares the property-tree parameters (`Transform`, `Filters`,
  `Path`, `RenderSurface`, `CopyOutputRequest`), `RegionBackend`, `Layer` (the
  push target), `UIResourceBitmap` and the CommitState payload types.
  `LayerTreeTypes` (impl) builds on it and restates the impl-only bounds there.
  `ImplTreeTransform` names the impl transform bound for facets that bind
  `LayerTreeTypes<Transform = T>`.
- Main layers create LayerImpls only through `main_layer::LayerImplFactory`
  (one method per Layer::CreateLayerImpl override). The tree synchronizer
  lends it over the pending tree with `SyncTypes::WithLayerImplFactory`.
- `MainLayerTypes` builds on `CommitTypes`, not `LayerTreeTypes`. Its
  remaining impl-side bounds (`DrawFilters`, `LayerTreeTransform`,
  `SkRegionBackend`, `MainLayerImpl`, `MainLayerCopyOutputRequest`) are compositor
  traits that have to move with the main half (S1a-2/3) or into
  `layerize::commit`.
- `LayerImplTypes` names `LayerTreeTypes` explicitly. It used to reach it only
  through `MainLayerTypes`.

### Compositor sub-crate items used by layerize (decide in S4)

These are compositor-engine packages. In S4, each used item is either
classified as a shared cc value root (like Layout's shared roots) or moved into
`layerize::commit`. Nothing is changed yet.

- `compositor_tree`: `geometry::*` (gfx value types); `property_types` (trace
  vocabulary, `FilterOperations`, `MaskFilterInfo`, `BlendMode`,
  `SubtreeCaptureId`, `ViewTransitionElementResourceId`, `InlinedVector`, the
  `PropertyPath` base trait); `property_tree` (`PropertyTree`, `PropertyNode`,
  `ClipTree`, `PropertyTreesState`, `PropertyTreesCachedData`,
  `PropertyTreesOwnerId`, draw-transform/sticky/anchor data, and the transform
  update policy functions); `transform::PropertyTreeTransform`; `effect_node`
  (`EffectTreeFilters` and the effect policies); `effect_tree_update::
  EffectNodeUpdateResult`; `render_surface_lifecycle::PlanRenderSurfaceLifecycle`;
  `scroll_delta_collection::{CollectScrollDeltas, PullAllScrollDeltasForTesting}`;
  the modules `clip_node`, `transform_node`, `target_property`, `property_ids`,
  `viewport_property_ids`, `synced_property`, `math_util`, `commit_types`.
- `compositor_input`: `scroll_decision::DecideScrollBy`; the modules
  `browser_controls_params`, `compositor_commit_data`,
  `event_listener_properties`, `input_types` (`ElementId`),
  `layer_selection_bound`, `main_thread_scrolling_reason`,
  `overscroll_behavior`, `page_scale_animation`, `scroll_snap_data`,
  `scroll_source_type`.
- `compositor_draw`: `Color4f`; the module `damage_reason`;
  `render_surface_policy::RenderSurfacePropertyChangedFlags`.
- `compositor_animation`: the module `property_animation_state`.
- `compositor_scheduler`: `TimeDelta`, `TimeTicks`, which it re-exports from
  `scroll_animation`.
- Also `skia::SkPath`, for the `geometry::Path` impl in
  `layerize/src/skia_path.rs`.

### S1a-2 status

Done:

- `LayerImplFactory` lives in `layerize::commit::layer_impl_factory`.
- `MainLayerTypes` no longer requires `LayerTreeTransform`, because
  `Are2dAxisAligned` needs only `PropertyTreeTransform`. `region_backend` is
  re-exported from `layerize::commit`.
- **Push target (S1a-2b part 1).** `layerize::commit::push_properties` holds
  `MainLayerImpl`, `MainLayerImplTree`, `LayerValue`, `LayerDebugInfo` /
  `Invalidation` and the new `PushPropertiesTypes: CommitTypes<Layer:
  MainLayerImpl<Self>, RegionBackend: SkRegionBackend>` with the pushed types
  `NodeId`, `RegionCaptureBounds`, `TrackedElementRects`, `FilterQuality`,
  `DynamicRangeLimit` and `ImplContext<'a>` (moved out of `MainLayerTypes`,
  which now has `PushPropertiesTypes` as a supertrait). The compositor paths
  `adapter::main_layer::{MainLayerImpl, ...}` and
  `cc::layers::layer::LayerDebugInfo` re-export them. `CompositorLayer` and the
  impl contexts implement the foreign traits for local types.

- **Property-tree value facets (S1a-2b part 2).**
  `layerize::commit::layer_value_backend` holds `DrawFilters`,
  `MainLayerPath` and `MainLayerCopyOutputRequest` (re-exported from
  `adapter::draw_property` / `adapter::main_layer`). Their impls for foreign
  types moved with them: `DrawFilters for FilterOperations` and the
  `FilterMapTransform` matrix binding to `layerize::commit::skia_filters`
  (the compositor's damage/occlusion/render-surface filter facets import the
  binding from there), `MainLayerPath for SkPath` to `layerize/src/skia_path.rs`.
  `CopyOutputRequest` is a compositor type, so its impl stays in
  `runtime/types.rs`. layerize now depends on `paint` and `foundation`.
- **Main Skia value backend (S1a-2b part 3).**
  `layerize::commit::skia_value_backend::MainSkiaBackend` holds the backend's
  `Filters: DrawFilters` and `Path: MainLayerPath` value types and the path
  operations (`FromPaintPath`, `StrokeBounds`, `StrokePath`,
  `StrokeBackendPath`). The compositor's `SkiaValueBackend`
  (`runtime/types.rs`) is now a blanket-implemented extension adding only the
  impl filter facets (`RenderSurfaceFilters`, `OcclusionFilters`,
  `DamageFilters`); `CpuSkiaBackend` is unchanged. `SkRegion` (local to the
  compositor) implements `MainSkiaBackend` in `runtime/skia.rs`.
- **Family binding (S1a-2b part 4, option h approved).**
  `layerize::commit::layerize_family::LayerizeFamily: PushPropertiesTypes +
  CommitStateTypes` with `type Skia: MainSkiaBackend + SkRegionBackend`;
  `runtime/owner_types.rs` implements it for `CpuCompositorTypes<R>` with
  `Skia = R`. It has no users yet. Add impl-provided hooks to it only when a
  main trait moves and its impl needs one (e.g.
  `ScrollbarLayerTypes::RasterizeScrollbarThumb`, which builds the impl
  `UIBitmap`).

- **Shared Skia/gfx values (S1a-2b part 5).** `layerize::commit::skia_region`
  (git mv of `runtime/skia.rs`) holds `SkRegion`, its `SkRegionBackend`,
  `RoundedRectTransformBackend` and `MainSkiaBackend` impls, the SkRRect
  helpers and `stroke_backend_path`. `layerize::commit::transform` (git mv of
  `runtime/geometry.rs`) holds `Transform<R>` and `RoundedRectTransformBackend`
  with the impls of the shared facets (gfx `Transform`, `PropertyTransform`,
  `PropertyTreeTransform`, both `OcclusionTransform`s, `ScrollbarTransform`,
  `DamageTransform`, `GeometryMapperMatrix`); `FloatClipRectTransform` moved
  with it (`commit/float_clip_rect_transform.rs`, was
  `runtime/paint/geometry_values.rs`). The compositor files of the old names
  re-export them and keep only impls of compositor-local traits
  (`runtime/skia.rs`: Damage/Occlusion/RenderSurface filter facets;
  `runtime/geometry.rs`: `DrawTransform`, `LayerTreeTransform`,
  `LayerImplTransform`, `ConversionTransform`, through the new public
  `Transform::{FromSource, MapPoint, MapRect, FlattenedSkMatrix, GetInverse}`).
  layerize now depends on `paint_model` and `paint_skia`.
- **Paint service split (S1a-2c part 1).** The `CpuPaintServices` struct
  (`runtime/paint_services.rs`) is now `CpuMainPaintServices` (PAC, chunk
  conversion, GeometryMapper, PaintRecord production, main Layer services;
  the unrelated `paint::CpuPaintServices` trait is unchanged).
  `CpuImplPaintServices` holds only the shared `CpuPaintValueServices` lease
  (`CpuMainPaintServices::ImplServices()`). `engine_factory::Create` takes
  both; the impl image-decode factory uses `impl_paint`. Nothing else in the
  old struct was impl-side. `EngineCore` still holds the main services until
  S1a-3.

- **cc/paint classification (S1a-2c part 2, approved).**
  1. `layerize::commit::skia_value_backend::PaintSkiaBackend` =
     `SkRegionBackend + RoundedRectTransformBackend +
     MainSkiaBackend<Filters = FilterOperations> + Send + Sync + 'static`
     (blanket). The compositor's `CpuSkiaBackend` is now `PaintSkiaBackend +
     SkiaValueBackend` (blanket), and `LayerizeFamily::Skia: PaintSkiaBackend`.
  2. `layerize::commit::paint` is cc/paint: `PaintRecord` (the lease struct,
     from `runtime/paint.rs`), `display_item_list` (git mv of
     `cc/paint/display_item_list.rs`), `display_item_list_types` (git mv of
     `adapter/display_item_list.rs`, plus `DirectlyCompositedImageInfo` from
     `adapter/picture_layer_impl.rs`), `node_info` (git mv of
     `runtime/display_item_list.rs`), `record`, `lowering`, `worklet`,
     `image_values`, `metadata`, `discardable_image_map`,
     `solid_color_analyzer` (git mv from `runtime/paint/`), `image_provider`
     (the ImageProvider/DecodedDrawImage values cut from
     `runtime/paint/raster.rs`) and `image_metadata` (PaintImageId,
     FrameMetadata, AnimatedImageMetadata, cut from `adapter/image_animation.rs`).
     `commit::rtree` is a git mv of `cc/base/rtree.rs`. The record code bounds
     on `PaintSkiaBackend`. The compositor paths (`cc::paint::display_item_list`,
     `adapter::display_item_list`, `runtime::display_item_list`, `cc::base::rtree`,
     `runtime::paint::*`, `adapter::image_animation::*`) re-export them.
     Impls of compositor-local traits for these values (HostPaintImage,
     HostDecodePaintImage, HostDrawImage, TileManagerPaintImage,
     TileManagerTargetColorParams, TileManagerDrawImage,
     PicturePaintWorkletInput, PictureLayerNodeInfo) moved to
     `runtime/paint/record_bindings.rs`; the PictureDisplayItemList /
     HostDisplayItemList facets of `DisplayItemList` to the end of
     `adapter/picture_layer_impl.rs`. `CpuPaintOp::FromConversion` became the
     free `CpuPaintOpFromConversion` in `runtime/paint/conversion.rs` (inherent
     impls of a foreign type are not allowed). `raster.rs` and `picture.rs`
     stay in the compositor: they are impl raster/tiling bindings plus main
     recorder/services parts that move with S1a-3.
  3. `commit::{ScrollOffsetMap, ContentColorUsage}` re-export
     `compositor_tile::prioritized_tile_types` / `compositor_frame::surface_id`
     (layerize now depends on `compositor_tile`, `compositor_frame`,
     `compositor_resource`).
  4. `commit::color_space` holds `RasterColorSpace` and its id enums,
     `HostColorSpace` (+ its impl for `RasterColorSpace`, moved from
     `runtime/host.rs`), `HostColorSpacePrimaryID`, `HostTargetColorParams`,
     `HostImageDecodingMode`; `runtime/resources.rs` and
     `adapter/layer_tree_host_impl.rs` re-export them.

  Known gap: the `#[cfg(test)]` modules of the moved `lowering.rs`
  (`CpuMainPaintServices`, a compositor type until S1a-3) and `metadata.rs`
  (needs `canvas` / `paint_input_types` dev-dependencies) do not build as
  layerize unit tests yet; fix them when `paint_services.rs` moves.

### S1a-3 status

Approved for S1a-3: option (h) as above; during S1a only,
`runtime/engine_core.rs` and `runtime/engine_assembly.rs` may name
`layerize::LayerizeEngine` (a named temporary exception in
`layerize_boundary.rs`, removed in S1b when Page owns it).

- **B1 done (cc main layers).** git mv into layerize:
  `cc/layers/{layer, layer_list_iterator, picture_layer, recording_source,
  painted_scrollbar_layer, solid_color_layer, solid_color_scrollbar_layer,
  surface_layer, scrollbar_layer_base, touch_action_region}`,
  `cc/input/scrollbar.rs` (the cc::Scrollbar trait, plus gfx `Insets` from
  `adapter/scrollbar.rs`) and the families `adapter/{main_layer,
  picture_layer, recording_source, scrollbar_layer_base, surface_layer,
  solid_color_layer, painted_scrollbar_layer, solid_color_scrollbar_layer}`.
  They live in `layerize/src/{cc,adapter}`; `layerize::commit::layers` and
  `layerize::commit::layer_types` re-export them for the impl half (the
  TreeSynchronizer push reads main Layers inside the protected sequence).
  Private alias modules in `layerize/src/{cc,adapter}.rs` keep the moved
  files' value-root paths. Shared values moved to `commit`:
  `layer_tree_settings` (cc/trees), `invalidation_region` (cc/base),
  `draw_property_utils::ScreenSpaceTransformForLayer`, and the new
  `surface_range::SurfaceRangeTypes` (viz::SurfaceRange default/==/IsValid,
  formerly on `SurfaceLayerImplTypes`, now its supertrait). The recording
  values of `runtime/paint/picture.rs` (Picture, CpuRecordingTypes,
  RasterSourceStorage + TilingRasterSource, CpuPictureRecorder and its
  recorder impls, the RecordingDisplayItemList / PictureDisplayItemList
  impls) are `commit::paint::picture`; the impl facets (PictureRasterSource,
  TileManagerRasterSource, discardable-image facets), MicroBenchmark,
  CpuPaintServices and RasterTransform stay in `runtime/paint/picture.rs`.
  The `PictureLayerNodeInfo` impl for `NodeInfo` moved to layerize.
  `PaintedScrollbarLayerImplTarget` no longer returns the impl
  `PaintedScrollbarLayerImpl`: it has the seven setters
  PaintedScrollbarLayer::PushPropertiesTo calls (implemented in
  `runtime/owner_types.rs`), and `PaintedScrollbarLayerTypes` no longer
  requires `PaintedScrollbarLayerImplTypes`.
- **Deviation (blanket impls deferred to B2).** The family impls of the moved
  traits (`MainLayerTypes`, `ScrollbarLayerTypes`, `PictureLayerTypes`,
  `SolidColorLayerTypes`, ...) stay in the compositor as foreign-trait impls
  for the local `CpuCompositorTypes<R>` (legal). They cannot become blanket
  impls over `F: LayerizeFamily` yet, because their associated types are
  still compositor types: `DerivedServices = dyn CpuPaintServices<Self>`
  names PAC/ContentLayerClient/LayersAsJson service traits (B2),
  `MicroBenchmark`, `DidScrollCallback`, `TaskRunner`, `ObjectTracker`,
  `PaintTraceScope`, `ScrollbarCanvas`/`Scrollbar` (`runtime/paint/
  scrollbar.rs`). Convert them in B2 once those move; add hooks to
  `LayerizeFamily` for impl-provided items (`RasterizeScrollbarThumb` builds
  the impl `UIBitmap`; `SurfaceRangeTypes` is already a commit trait).

- **B2 done (Blink compositing + Paint-value bindings).** Copied into
  layerize with one-line compositor re-export stubs (as in B1):
  `third_party/blink/renderer/platform/graphics/{compositing,paint}/*` and
  the adapters `paint_artifact_compositor`, `paint_chunks_to_cc_layer`,
  `content_layer_client_impl`, `pending_layer`, `property_tree_manager`,
  `paint_geometry`, `raster_invalidator` (exported for EngineCore through the
  temporary `layerize::main_types`). git mv into `layerize/src/runtime/`:
  `runtime/paint.rs` (PaintInput, the property-node handles,
  PropertyTreeState, the PaintChunk/PaintArtifact facets) and
  `runtime/paint/{artifact_nodes, conversion, conversion_nodes,
  conversion_values, geometry_mapper, geometry_nodes, invalidation,
  pending_nodes, pending_values, property_state, reasons, scroll_nodes,
  scrollbar, subset, validation}.rs`; `CpuPaintServices` and `MicroBenchmark`
  moved from `runtime/paint/picture.rs` to `layerize/src/runtime/paint/
  layer_services.rs`. The compositor's `runtime/paint.rs` re-exports
  `layerize::runtime::paint::*` and keeps `picture.rs` (impl facets,
  RasterTransform), `raster.rs`, `record_bindings.rs`, `services.rs`
  (Skia platform services) and the new `artifact_record.rs`. Shared values
  moved to commit: `viz_metadata` (TrackedElementFeature/Rect/Rects,
  FrameToken, RegionCaptureCropId/Bounds and their `LayerValue` impls) and
  `paint::NodeId`; their impls of compositor traits stay in
  `runtime/{metadata,types}.rs`. The `ConversionTransform for Transform<R>`
  impl moved to layerize `adapter/paint_chunks_to_cc_layer.rs`.
  Genericization: the bindings that named `CpuCompositorTypes<R>` are now
  `impl<R, B> Trait<B> for Value<R> where B: FamilyTrait<Node = Node<R>, ..>`
  (ArtifactEffectNode, ArtifactDisplayItem, PendingDisplayItem); the family
  owners are generic over `B` (`CpuForeignLayer<B: MainLayerTypes>`,
  `CpuLayersAsJson<B>`); `PaintInput::ViewportProperties::<R, B>()`.
  Deviation: `ArtifactPaintRecord for CpuPaintRecord<R>` stays in the
  compositor (`runtime/paint/artifact_record.rs`, with its unit test). A
  family-generic impl needs `B: PaintArtifactCompositorTypes<PaintRecord =
  Self>`, and that where clause supplies a param-env candidate for the same
  impl, so the trait's opaque `ops` iterator cannot be normalized (E0053);
  boxing would add an allocation per record walk. It moves once the family
  impls are blanket impls (then no where clause is needed).
  Boundary test: the moved files left the property-node allowance (only
  `runtime/paint_services.rs` remains); a temporary
  `MAIN_HALF_REEXPORT_ALLOWED` lists the re-export stubs until B3.
- **Still in the compositor after B2.** `runtime/paint_services.rs`
  (CpuMainPaintServices, the PAC/PendingLayer/PaintChunks/SynthesizedClip
  service impls, `Root<R> = CpuCompositorTypes<R>`; it also names impl traits
  `TileManagerTypes`, `PictureLayerImplTypes`, `PictureLayerTilingTypes`,
  `DrawTransform` and `engine::ScrollbarPaintMutation`, which must be split
  off first), `pac_scroll.rs` (`engine::CompositorScrollState`) and
  `canvas_child_record.rs`; they move with B3 (main_host). The main family
  impls (`picture_types.rs`: PAC, PaintChunks, SynthesizedClip,
  PropertyTreeManager, PendingLayer, PictureLayer, SolidColorLayer;
  `owner_types.rs`: MainLayerTypes, ...) are still compositor impls for
  `CpuCompositorTypes<R>`; their associated types now name layerize values,
  so the blanket conversion is unblocked except for the impl-provided items
  (`TaskRunner`, `DidScrollCallback`, `RasterizeScrollbarThumb` and the
  other compositor-defined associated types of the main traits) need
  `LayerizeFamily` hooks. The layerize unit tests are not
  built yet: `lowering.rs` and `metadata.rs` still need compositor types or
  dev-dependencies; the moved `invalidation`, `scroll_nodes`, `scrollbar` and
  `subset` tests name only layerize paths but were not run (B4).

- **Blanket family impls done.** `layerize/src/runtime/main_family.rs` holds
  `impl<F: LayerizeFamily>` blanket impls of PAC, PaintChunks,
  SynthesizedClip, PropertyTreeManager, PendingLayer, PictureLayer,
  SolidColorLayer, MainLayer, ScrollbarLayer, SolidColorScrollbarLayer and
  PaintedScrollbarLayer types (cut from `runtime/{picture_types,
  owner_types}.rs`). `LayerizeFamily` now fixes, through its
  `PushPropertiesTypes<..>` supertrait, the commit values the main half names
  (`Transform = Transform<Skia>`, `Filters`/`Path` of `Skia`,
  `RegionBackend = Skia`, `NodeId`, `RegionCaptureBounds`,
  `TrackedElementRects`, `FilterQuality`, `DynamicRangeLimit`,
  `CopyOutputRequest: MainLayerCopyOutputRequest<TaskRunner = TaskRunner>`)
  and requires the push targets (`Layer: PictureLayerImplTarget<Self> +
  SolidColorLayerImplTarget + PaintedScrollbarLayerImplTarget +
  SolidColorScrollbarLayerImplTarget + SurfaceLayerImplTarget`) plus
  `SurfaceRangeTypes`, so `F: LayerizeFamily` alone proves every main trait
  and their associated types normalize. Its only hook is
  `RasterizeScrollbarThumb` (builds the impl `UIBitmap`). `DidScrollCallback`,
  `ResultTaskRunner` and `TaskRunner` moved to `commit::main_callbacks`
  (re-exported from `runtime/types.rs`). Rule for main code from now on:
  bound generics on `F: LayerizeFamily` only, never on a main trait, or the
  param-env candidate shadows the blanket impl and projections stop
  normalizing. `ArtifactPaintRecord` is
  `impl<F: LayerizeFamily> for CpuPaintRecord<F::Skia>` in
  `layerize/src/runtime/paint/artifact_record.rs` (no boxing needed); its unit
  test stays in the compositor file of the same name (it needs the concrete
  family).

- **B3a done (main paint services).** `runtime/paint_services.rs` is a git mv
  to `layerize/src/runtime/paint_services.rs` (`CpuMainPaintServices`,
  `CpuPaintFeatures`, `CpuPaintPlatform` and all PAC / PaintChunks /
  PendingLayer / SynthesizedClip / ContentLayerClient service impls). The
  family impls are `impl<R: PaintSkiaBackend, F: LayerizeFamily<Skia = R>>
  Trait<F> for CpuMainPaintServices<R>`; the PAC entry points
  (`CreateCompositor`, `UpdateCompositor`, `ApplyCompositedAnimationProperties`,
  `ApplyScrollbarPaintMutation`, now taking the mutation's four fields instead
  of `engine::ScrollbarPaintMutation`) are generic methods over
  `F: LayerizeFamily<Skia = R>`. `SynthesizedClipServices::MapRect` uses
  `ToEnclosingRect(Transform::MapRect)` (gfx::Transform::MapRect(Rect)) instead
  of the impl `DrawTransform::MapRectInt`. The compositor file of the same name
  keeps `CpuImplPaintServices` (now `CpuImplPaintServices::new(&main)`), the
  impl `SynchronizationTrace` binding (through the new
  `CpuMainPaintServices::Platform()`) and the family compile assertion, and
  re-exports the main items (temporarily listed in
  `MAIN_HALF_REEXPORT_ALLOWED`). `CpuMainPaintServices::new_cpu` became the
  free `runtime::paint::NewCpuMainPaintServices` (inherent impls of a foreign
  type are not allowed). `PROPERTY_NODE_ALLOWED` is empty.

- **B3b done (LayerizeEngine, first owners).** `layerize::LayerizeEngine<F:
  LayerizeFamily>` (`layerize/src/layerize_engine.rs`) owns the PAC, the
  previous artifact, the `PacScrollCallbacks` strong owner and its
  `PacScrollEffects` FIFO, the `Arc<CpuMainPaintServices>` and the
  `MainFrameLifecycle`. `EngineCore` holds `layerize: LayerizeEngine<Root>`
  in place of those six fields and calls it where it ran them before
  (`new`, `UpdateCompositor` + `SetPreviousArtifact`,
  `ApplyScrollbarPaintMutation`, `ApplyCompositedAnimationProperties`,
  `DropCompositorScrollDeltaNextCommit`, `PushScrollEffect` /
  `PopScrollEffect`, `Shutdown`, `MainFrameLifecycle[Mut]()`; EngineAssembly
  reaches the lifecycle through `core.layerize`). The main LayerTreeHost is
  still a compositor owner, so each call borrows it as `&mut dyn
  MainLayerHosts<F>`; the canvas-child publication still runs in EngineCore
  over `layerize.Pac()` (its errors are `CompositorError`). git mv:
  `runtime/pac_scroll.rs` and `runtime/main_frame_lifecycle.rs` to
  `layerize/src/runtime/`, `runtime/canvas_child_record.rs` to
  `layerize::commit::paint::canvas_child_record` (raster input; layerize now
  depends on `canvas`; the compositor path `runtime::canvas_child_record`
  re-exports it for `raster`). Values cut to commit: `commit::main_effects`
  (`CompositorScrollState` from `engine.rs`, `PacScrollEffect`) and
  `commit::paint_holding` (`PaintHoldingReason`, `FrameLifecycleMutation`,
  `CommitDeferralReason`); `engine.rs` re-exports them.

- **Boundary test.** `layerize_boundary.rs` has the named temporary
  `LAYERIZE_ENGINE_ALLOWED` exception (`layerize::LayerizeEngine` in
  `runtime/engine_core.rs` and `runtime/engine_assembly.rs`, removed in S1b).
  `PROPERTY_NODE_ALLOWED` is empty; `MAIN_HALF_REEXPORT_ALLOWED` still lists
  the re-export stubs (plus `runtime/paint_services.rs`), which go when
  EngineCore stops naming the main types through them. The test passes.

- **B3c done (main LayerTreeHost in LayerizeEngine).** git mv
  `runtime/main_host.rs` and `runtime/main_endpoint.rs` to
  `layerize/src/runtime/`. `CpuMainLayerHost<F: LayerizeFamily>` is generic;
  `LayerizeEngine` owns it (declared after PAC) and its
  `EngineMainEndpoint`, and builds both from
  `commit::main_effects::MainHostInit` (id, owner, mutator, settings,
  threaded, embedder background color, sink). The impl owners it named were
  split off as follows:
  - `LayerizeFamily` fixes `LocalSurfaceId` (viz value, re-exported as
    `commit::main_effects::LocalSurfaceId`), `MainRootLayer = LayerRef<Self>`,
    `MainLayerRef = LayerBorrow<Self>` and `ViewTransitionRequest:
    commit::main_callbacks::MainViewTransitionRequest` (implemented for
    `host_owners::ViewTransitionRequest`), and gains the hooks
    `CreateUIResourceRequest`, `UIResourceBitmapSize`, `EmptyUIResourceBitmap`
    (UIResourceRequest/UIBitmap) and `UpdateMainPropertyTrees` (the
    draw_property_utils.cc:1590-1605 step of UpdateLayers; its body, which
    needs the impl `DrawContext` type, is in `runtime/owner_types.rs`).
    `UIResourceRequestType` moved to `commit::values`.
  - `WithCommitContext` lends `(&ThreadUnsafeCommitState, &mut LayerContext,
    MainLayerHostId)`; EngineCore builds `sync::MainSynchronizationContext`
    from it. `LayerizeEngine::WithCommitContext` enters the protected
    sequence.
  - The endpoint's Proxy side is `commit::main_effects::{MainHostEffect,
    MainHostEffectSink}`; the compositor implements the sink with
    `execution::EngineMainHostEffects` (executor, command queue, events, main
    `AnimationHostOwner`, main task runner). The AnimationHost mutations
    (`ApplySource{Scroll,Property}Animation`, animation events, scroll
    completions) are EngineCore calls on that owner, followed by
    `LayerizeEngine::SetNeedsCommitFromMutatorHost`. The sink and
    `CpuMainHostEndpoint` are not `Send + Sync` yet (the AnimationHost owner
    holds a non-Send mutator backend).
  Moved into `LayerizeEngine` from EngineCore/EngineAssembly, same call order:
  `ApplyCompositorChanges` (main half of ApplyCompositorDeltas; returns the
  page scale for the ViewportScaleChanged effect), `UpdatePaintArtifact`
  (PAC::Update + canvas-child publication + resident artifact; in an
  `F: LayerizeFamily<Skia = SkRegion>` block because raster downcasts the
  SkRegion `CanvasChildRecordReplay`), `ActivateCommitState` (UpdateLayers,
  WillCommit, ActivateCommitState, source frame advance, DidBeginCommit),
  `ConvertPaintArtifactSubsetToRecord`, `CreateVideoSurfaceForeignLayer`,
  `NotifyTransitionRequestsFinished` and the composited-animation coalescing
  (`CoalesceCompositedAnimationProperties`, `TakePendingCompositedAnimation`,
  `HasPendingCompositedAnimation`). Errors cross as
  `commit::layerize_error::LayerizeError { kind, detail }` (PaintInputError
  names paint property nodes; `engine.rs` maps the kind to
  `CompositorPaintInputKind` as before). Plain LayerTreeHost setters are
  reached through `LayerizeEngine::{Main, MainMut}`.

- **B3d done (no main-half re-export stubs).** Deleted the compositor stubs
  `adapter/{content_layer_client_impl, paint_artifact_compositor,
  paint_chunks_to_cc_layer, paint_geometry, pending_layer,
  property_tree_manager, raster_invalidator}.rs` and the whole
  `src/third_party` tree; `layerize::main_types` is gone. The compositor's
  `runtime/paint.rs` re-exports only `layerize::commit::paint` (plus the
  Paint engine values). `runtime/paint/services.rs` (Skia CPU paint platform,
  `NewCpuMainPaintServices`) is a git mv to `layerize/src/runtime/paint/`
  (layerize depends on `getrandom`); `CpuPaintFeatures` and
  `SkiaPaintDiagnostics` moved to `commit::main_paint_config`.
  `LayerizeEngine::new(features, diagnostics, init)` (an
  `F: LayerizeFamily<Skia = SkRegion>` constructor; the generic one is
  `WithPaintServices`) builds the main paint services itself; the factory
  creates it through `EngineCore::CreateLayerize` before the impl services
  and leases `LayerizeEngine::PaintValues()` to `CpuImplPaintServices::new`.
  `SynchronizationTrace` moved to `commit::main_callbacks` (implemented by
  `CpuMainPaintServices` in layerize). `LayerizeFamily::RasterizeScrollbarThumb`
  became `CreateUIResourceBitmap(pixel_size, pixels, row_bytes, opaque)`: the
  main half paints the thumb (`main_family.rs`), the family only allocates
  the bitmap (`EmptyUIResourceBitmap` is now a provided method).
  `layerize_boundary.rs` has no re-export allowance any more; only
  `LAYERIZE_ENGINE_ALLOWED` remains. The compositor unit test of
  `ArtifactPaintRecord` (`runtime/paint/artifact_record.rs`) was deleted with
  the stubs (it named PAC traits); B4 re-adds it in layerize.

Next: B4 (layerize unit tests: lowering.rs/metadata.rs imports and
dev-dependencies, the ArtifactPaintRecord test).

### Plan correction: option (a) does not compile

The agreed option (a) (`CommitFamily<R, I>` defined in `layerize::commit`,
`CpuCompositorTypes<R>` an alias of it) breaks the orphan rule for the impl
traits that live in compositor sub-crates. The family implements these
foreign traits, and an alias of a layerize type is not a local type in
`compositor`:

- `compositor_tile`: `TileTypes`, `PictureLayerTilingTypes`
  (`runtime/picture_types.rs:143,154`)
- `compositor_animation`: `PaintWorkletTypes` (`picture_types.rs:166`)
- `compositor_frame`: `FrameDataTypes` (`runtime/types.rs:184`),
  `UnboundedTypes` (`runtime/host_types.rs:51`)
- `compositor_resource`: `ResourcePoolTypes` (`runtime/resources.rs:550`)

Making them work under (a) would need layerize to depend on those impl
packages and forward every item through mirror traits.

**Replacement (option h): keep the family local to the compositor, bind the
main half through one commit trait.**

1. `CpuCompositorTypes<R>` stays a compositor struct. All impl traits
   (compositor and sub-crate) stay implemented there, unchanged.
2. `layerize::commit` declares `LayerizeFamily` (name open):
   `PushPropertiesTypes + CommitStateTypes` plus `type Skia:
   MainSkiaBackend` (the main part of `SkiaValueBackend`/`CpuSkiaBackend`,
   see step 3 of the old plan) and the few impl-provided hooks main traits
   need today (e.g. `ScrollbarLayerTypes::RasterizeScrollbarThumb`, which
   builds the impl `UIBitmap`; `PaintedScrollbarConstructor`; the `MainLayerTypes`
   `DerivedServices`/`MicroBenchmark` items). The compositor implements it for
   `CpuCompositorTypes<R>` (foreign trait, local type: allowed).
3. When a main trait moves to layerize (S1a-3), its impl becomes a blanket
   `impl<F: LayerizeFamily> MainTrait for F { type Recording =
   CpuRecordingTypes<F::Skia>; ... }` in layerize. There is exactly one family,
   so the blanket impls do not conflict. Compositor code that names
   `<CpuCompositorTypes<R> as PictureLayerTypes>::Recording` still normalizes
   through the blanket impl.
4. Main concrete code that names `CpuCompositorTypes<R>` (`Root<R>` in
   `runtime/paint_services.rs`, `runtime/paint/*`) becomes generic over
   `F: LayerizeFamily`, using `F::Skia` where it used `R`. This is the same
   amount of genericization option (a) needed (it added `I` everywhere).

Remaining S1a-2b work under (h): `MainSkiaBackend` in commit, then
`LayerizeFamily` and its compositor impl. S1a-2c and S1a-3 are unchanged
except that main-trait impls become blanket impls as in point 3.
