# Compositor main/impl thread split (live property tree, stage S1)

Status: S1c done. S1a-1 done (see "S1a-1 status" at the end); S1a-2/3
and the thread split (S1b/S1d) are not implemented.

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
