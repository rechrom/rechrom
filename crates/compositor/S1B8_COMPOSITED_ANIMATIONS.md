# S1b-8 design: composited animations local on the document thread

Status: design only, nothing implemented. Written 2026-11-23 against main
`7f978395`; Rechrom line numbers refer to that revision. Resolves blocker B5
of `S1B_PROXY_DESIGN.md` (section 3, row S1b-8 of the step table).

Prerequisite: S1b-7 as agreed with the coordinator. Page owns
`layerize::LayerizeEngine` (which now contains ProxyMain) on the
`page-document-*` thread; `run_impl` owns `compositor::CompositorEngine`; the
two talk only through `ImplToMain` / `MainToImpl` (`CommitState` +
`ProtectedCommitLease`). The interfaces this design relies on:

- `LayerizeEngine::{new, Start, Stop, BeginMainFrame(state, &mut dyn
  MainFrameClient) -> MainFrameResult, RunImplTask, UpdatePaintArtifact,
  SetNeeds*, ApplyCompositedAnimationProperties, ...}`
- `CompositorEngine::{new(config, CompositorServices, ProxyImplInit, to_main),
  ReadyToCommit(CommitState, lease), BeginMainFrameAborted, SetNeedsCommit,
  SetDeferBeginMainFrame, SetVisible, BeginFrame, Input, Mutate, Poll, ...}`

Chromium revision `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`. There is no
Chromium checkout on this machine. Chromium line numbers below come from the
`// Source:` citations already in the Rechrom translation; where none exists
the function is named and marked "(lines: confirm)". Confirm those against
the pinned checkout before implementation.

## 0. Target in one paragraph

Blink's direct transform/opacity update is a synchronous main-thread call:
PrePaint finds a property node whose only change is a composited animated
value, `PaintArtifactCompositor::DirectlyUpdate*` writes it into the main
`LayerTreeHost` property trees, and `LayerTreeHost::SetNeedsCommit` asks
ProxyMain for a commit. Inside a BeginMainFrame that request only raises
`final_pipeline_stage_` to COMMIT, so the frame's own commit carries the
value: one property-only commit per animated frame, with no UpdateLayers.
After S1b-8, Rechrom does exactly this. Page calls
`layerize.ApplyCompositedAnimationProperties(update)` right after
`PaintEngine::PrePaintCompositedAnimationUpdate`, in the same task. It
installs or removes cc KeyframeModels on the main AnimationHost in that same
task (`layerize.ApplySourcePropertyAnimation`). If PAC rejects the direct
update, Page falls back to the full Paint path in the same lifecycle. The
compositor thread no longer receives composited-animation values, so there is
no commit slot to coalesce against and no publication-sequence gate.

## 1. Current path (main 7f978395)

### 1.1 Page: collect, gate and prepare

`Page::UpdateFrameIfNeededInternal` (`crates/browser/src/page.rs:9494`), reached
from `UpdateRendering` (`page/begin_frame.rs:330`) through
`UpdateFrameIfNeeded` (`page.rs:9309-9313`):

1. `animation_paint_nodes` = `PaintEngine::AnimationTargetedNodes()` when
   only paint-affecting style changed (`page.rs:9818-9825`).
2. Admissibility: no dirty paint resources and no pending paint mutations
   (`page.rs:10014-10027`), no child-frame paint (`:10028-10030`).
3. **Gate** (`page.rs:10032-10041`):
   - external runtime: `external_compositor_published_frame_sequence ==
     Some(frame.sequence)`. The field is declared at `page.rs:6826`,
     initialized at `:7412`, cleared at `:7770` (SetExternalCompositor(false))
     and `:7863` (DidInstallNewDocument), and set by
     `DidPublishFrameToExternalCompositor` (`page.rs:7777-7784`), which
     `run_document` calls after it sends `ImplCommand::Commit`
     (`apps/rechrom/src/page_runtime.rs:3404`).
   - internal presentation: `PageRenderPipeline::CanApplyCompositedAnimationProperties`
     (`page.rs:1243-1250`) -> facade `CompositorEngine::CanApplyCompositedAnimationProperties`
     (`compositor/src/engine.rs:683-685`) -> `LayerizeEngine::HasResidentPaintArtifact`.
4. Gate open: `PaintEngine::PrePaintCompositedAnimationUpdate`
   (`paint/src/paint_engine.rs:1225-1263`), which first tries
   `PrePaintTreeWalk::DirectlyUpdateCompositedAnimationProperties`
   (`paint/prepaint/src/pre_paint_tree_walk.rs:8208`) and otherwise runs a full
   PrePaint walk and collects the update from it. Gate closed: a plain
   `PrePaintWithLifecycle`, so the frame goes through full Paint.
5. With an update (`page.rs:10073-10338`):
   - It builds `PropertyAnimationMutation::Install` values for each binding
     inline (`:10079-10287`). This is a second copy of
     `InstallCompositorPropertyAnimations` (`page.rs:9341-9492`, used by the
     full-paint path at `:10549`). The start time is converted through the
     **impl half's** executor clock: `state.rendering.borrow().impl_half.executor.TimeTicks`
     (`:10263`; the same at `:9447`).
   - It routes `CompositorMutation::CompositedAnimationProperties(update)` and
     then each `CompositorMutation::PropertyAnimation(Install)` through
     `RouteCompositorMutation` (`:10290-10301`). A comment there explains the
     ordering: the element mapping must reach the compositor before the
     KeyframeModel can tick, because both travel separately.
   - It marks the animations compositor-driven (`:10302-10311`), commits the
     retained paint lifecycle without Paint (`:10312-10328`) and returns
     before the full-paint path, so `PageScripts::DidPaint` is skipped
     (`:10329-10338`).
6. Removals and replacements (`RoutePendingPropertyAnimationRemovals` /
   `Replacements`, `page.rs:9315-9339`, called at `:9624-9625`) and
   `ScrollbarPaint` (`page/pointer_scroll_controls.rs:209`) take the same route.

### 1.2 Routing to the compositor

`RouteCompositorMutation` (`page/scroll.rs:790-815`):

- external: push onto `pending_compositor_mutations`. `run_document` drains
  it after `UpdateRendering` with `send_page_compositor_continuations`
  (`page_runtime.rs:3494-3527`, called at `:2877`), **before** the no-update
  check that may send `ImplCommand::FinishMainFrameWithoutUpdates`
  (`:2902-2925`). On the impl thread, `ImplCommand::InputContinuation`
  (`page_runtime.rs:296`, handled at `:1985-2005`) calls
  `ContinueCompositorMutation`. If the result carries
  `CompositorMainThreadEffect::BeginMainFrame`, it bumps `main_frame_epoch`
  and requires a main frame.
- internal: `PageRenderPipeline::ContinueCompositorMutation` synchronously,
  on the document thread.

Both end in `CompositorEngine::Mutate` (`engine.rs:699-725`), arm
`CompositedAnimationProperties` -> `EngineAssembly::ApplyCompositedAnimationProperties`.

### 1.3 Compositor: coalesce or prepare a property-only commit

`EngineAssembly` (`compositor/src/runtime/engine_assembly.rs`):

- `ApplyCompositedAnimationProperties` (`:325-350`): returns `NotReady`
  without a resident artifact. When `!core.CanPrepareCompositedAnimationCommit()`
  (`engine_core.rs:285-289`: a ready commit or a prepared commit already
  exists), it **coalesces** the update into
  `LayerizeEngine::pending_composited_animation` (latest value per property
  node; `layerize_engine.rs:115,296-331`) under the
  `Compositor.Animation.CoalescePropertyUpdate` span. Otherwise it continues
  to:
- `PrepareCompositedAnimationProperties` (`:360-376`):
  `EngineCore::PrepareCompositedAnimationCommit` (`engine_core.rs:317-347`)
  -> `LayerizeEngine::ApplyCompositedAnimationProperties`
  (`layerize_engine.rs:279-290`) -> `CpuMainPaintServices::ApplyCompositedAnimationProperties`
  (`layerize/src/runtime/paint_services.rs:776-817`, span
  `Compositor.Layerize.DirectAnimationPropertyUpdate`), which calls
  `PaintArtifactCompositor::DirectlyUpdateTransform` /
  `DirectlyUpdateCompositedOpacityValue` (`paint_artifact_compositor.rs:2066-2125`)
  and, when all succeed, `CompositorSimpleValuesUpdated` on each node. If
  one fails, `PrepareCompositedAnimationCommit` returns
  `NotReady("resident compositor property mapping is unavailable")`. Then
  `ProxyMain::PrepareCommit(false)` (property-only, no UpdateLayers), and
  `DidBeginCommit` when a BMF is open (suppresses the redundant
  SetNeedsCommit).
- Outside a BMF (`!awaiting_main`, `:369-374`) it finalizes immediately
  (`FinalizePreparedCommit` + `ScheduleFinalizedMainCommit`): a commit
  finished outside a BeginMainFrame (blocker B1).
- `Poll` re-tries the coalesced value once the slot is free
  (`:1742-1752`), and `CompositorOutput::main_commit_pending` includes
  `HasPendingCompositedAnimation()` (`:1804`, `:1857`). Page's
  `PageRenderPipeline` admission keys on that.
- Within a BMF, `run_document` answers with
  `ImplCommand::FinishMainFrameWithoutUpdates`, and
  `EngineAssembly::FinishMainFrameWithoutUpdates` (`:281-296`) finalizes the
  prepared property-only commit instead of aborting (`HasPreparedPaintCommit`).

### 1.4 Main AnimationHost and events

- KeyframeModel install/remove: `CompositorMutation::PropertyAnimation` ->
  `EngineAssembly::ApplySourcePropertyAnimation` (`engine_assembly.rs:317-323`)
  -> `LayerizeEngine::ApplySourcePropertyAnimation` (`layerize_engine.rs:372-383`).
  Since S1b-5 this already mutates the **main** AnimationHost that
  LayerizeEngine owns, and calls `SetNeedsCommitFromMutatorHost`.
- Impl -> main events: `LayerTreeHostImpl::TakeMutatorEvents`
  (`cc/trees/layer_tree_host_impl.rs:6899`) goes into the BMF state;
  `ProxyMain::BeginMainFrame` calls `LayerizeEngine::ApplyMutatorEvents`
  (`layerize_engine.rs:389-398`; Chromium layer_tree_host.cc:1254-1266), which
  returns only source **scroll** completions.
  `AnimationQueries::DelegateAnimationStarted` is empty and
  `DelegateAnimationFinished` handles only `SCROLL_OFFSET`
  (`compositor/src/runtime/animation_host.rs:1691-1720`). Blink transform/opacity
  animations never receive started/finished notifications. Page avoids
  needing them by computing the start time itself (1.1 step 5).
- Push: `AnimationHostOwner::PushMutatorPropertiesTo`
  (`animation_host.rs:955-1000`; Chromium animation_host.cc:485-518,538-570,602-640)
  reads the main instance through the leased `ThreadUnsafeCommitState`
  during FinishCommit (layer_tree_impl.cc:814-817).

### 1.5 Why the machinery exists

Every piece of section 1 makes up for the main half not being where Blink
runs:

| Mechanism | Compensates for |
|---|---|
| `external_compositor_published_frame_sequence` gate | Page cannot ask the compositor-thread PAC whether the artifact it would update is resident, so it waits until the host has *sent* the artifact. |
| `pending_composited_animation` coalescing | The single-thread engine has one prepared/ready commit slot; a new value arriving while it is occupied has nowhere to go. In Chromium the value is written into the main tree, which already owns the next commit's state. |
| Out-of-BMF finalize (`!awaiting_main`) | The update arrives as a message, outside the BMF that owns it (B1). |
| Element mapping before KeyframeModel ordering | Two separately routed messages. In Chromium both are main-tree mutations taken by one commit. |
| `NotReady` on PAC rejection | Page has already skipped Paint when the compositor finds out the direct update is not possible. |

## 2. Chromium path

### 2.1 Direct update during PrePaint (Blink, main thread)

- `PaintPropertyTreeBuilder` updates a transform/effect node; when the change
  type is `kChangedOnlySimpleValues` and the node has direct compositing
  reasons, it calls `PaintArtifactCompositor::DirectlyUpdateTransform` /
  `DirectlyUpdateCompositedOpacityValue`. On success the change becomes
  `kChangedOnlyCompositedValues` and `CompositorSimpleValuesUpdated()` is
  called (`core/paint/paint_property_tree_builder.cc`,
  `FragmentPaintPropertyTreeBuilder::UpdateTransform` / `UpdateEffect`; lines:
  confirm; `platform/graphics/paint/paint_property_node.h:82-85`). The
  deferred variant for animations without a full property update is
  `PaintPropertyTreeBuilder::DirectlyUpdateTransformMatrix` /
  `DirectlyUpdateOpacityValue`, run from `LocalFrameView::ExecuteAllPendingUpdates`
  (lines: confirm). Rechrom's `DirectlyUpdateCompositedAnimationProperties`
  cites both.
- `PaintArtifactCompositor::CanDirectlyUpdateProperties`
  (`paint_artifact_compositor.cc:1447-1457`): false when PAC needs a full
  update or the host is inside `WillCommit`. `DirectlyUpdateCompositedOpacityValue`
  (`:1458-1469`) and `DirectlyUpdateTransform` (`:1479-1495`) delegate to
  `PropertyTreeManager::DirectlyUpdateCompositedOpacityValue` (lines: confirm,
  near `property_tree_manager.cc:120-136`) and `DirectlyUpdateTransform`
  (`property_tree_manager.cc:179-206`). These write the cc node, mark the
  tree `needs_update` and call `LayerTreeHost::SetNeedsCommit`.
- If `DirectlyUpdate*` returns false, the property change stays
  `kChangedOnlySimpleValues`, PAC is marked for update and the same
  lifecycle's Paint runs `PaintArtifactCompositor::Update`
  (`local_frame_view.cc:3348`, through `PushPaintArtifactToCompositor`,
  `:2965-2975`). There is no error, and no second frame is needed.

### 2.2 The commit request inside BeginMainFrame (cc, main thread)

- `LayerTreeHost::SetNeedsCommit` -> `ProxyMain::SetNeedsCommit`
  (`cc/trees/proxy_main.cc:741-756`). While `current_pipeline_stage_ ==
  ANIMATE_PIPELINE_STAGE` it only raises `final_pipeline_stage_` to
  `COMMIT_PIPELINE_STAGE` and returns. Blink's whole lifecycle (Style, Layout,
  PrePaint, Paint) runs inside `LayerTreeHost::RequestMainFrameUpdate` in that
  stage (`proxy_main.cc:357-360`).
- After the lifecycle, `final_pipeline_stage_ < UPDATE_LAYERS_PIPELINE_STAGE`
  skips `LayerTreeHost::UpdateLayers`, but the commit still proceeds
  (`proxy_main.cc:439-580`; `WillCommit` at `layer_tree_host.cc:421-441`,
  `ActivateCommitState` at `:504-517`). The property trees are pushed in
  FinishCommit; the PictureLayer recordings stay as they are. **One
  property-only commit per animated frame.**
- Outside a BeginMainFrame (a forced lifecycle), `SetNeedsCommit` goes to
  `SendCommitRequestToImplThreadIfNeeded` (`proxy_main.cc:1112-1144`) ->
  `ProxyImpl::SetNeedsCommitOnImpl`. The written value waits in the main tree
  for the next BeginMainFrame's commit, and later writes overwrite it there.
- During the protected sequence (between `NotifyReadyToCommit` and the
  completion signal), Blink cannot write the tree: `CanDirectlyUpdateProperties`
  sees `in_commit`, and any main-tree access first calls
  `WaitForProtectedSequenceCompletion` (`layer_tree_host.cc:520-538`). The
  main tree is the only buffer; nothing is coalesced.

### 2.3 Starting compositor animations

- `DocumentAnimations::UpdateAnimations` -> `PendingAnimations::Update` ->
  `Animation::StartAnimationOnCompositor` ->
  `CompositorAnimations::StartAnimationOnCompositor` builds
  `CompositorKeyframeModel`s (cc `KeyframeModel` + gfx keyframed curves) and
  calls `cc::Animation::AddKeyframeModel` on the **main** AnimationHost
  instance (`core/animation/{document_animations,pending_animations,animation,
  compositor_animations}.cc`; lines: confirm). Blink runs this after Paint, from
  `LocalFrameView::RunPaintLifecyclePhase`, with the PAC so element ids are
  resolved (lines: confirm). The main AnimationHost asks for a commit through
  `MutatorHostClient::SetMutatorsNeedCommit` -> `LayerTreeHost::SetNeedsCommit`,
  which again only raises the final stage inside the BMF.
- In FinishCommit, `AnimationHost::PushPropertiesTo` pushes timelines,
  animations and KeyframeModels to the impl instance
  (`cc/animation/animation_host.cc:485-518,538-570,602-640`). The same commit
  carries the property-tree element ids, so the order is structural.

### 2.4 Events back to Blink

- Impl ticks produce `AnimationEvents` (STARTED with the monotonic start time,
  FINISHED, ABORTED, TAKEOVER). `ProxyImpl::ScheduledActionSendBeginMainFrame`
  puts them in `BeginMainFrameAndCommitState::mutator_events`
  (`proxy_impl.cc:801-848`).
- `ProxyMain::BeginMainFrame` -> `LayerTreeHost::ApplyMutatorEvents`
  (`proxy_main.cc:351-352`, `layer_tree_host.cc:1254-1266`) ->
  `AnimationHost::SetAnimationEvents` -> `cc::Animation::DispatchAndDelegateAnimationEvent`
  -> `AnimationDelegate::NotifyAnimationStarted` / `NotifyAnimationFinished`
  -> Blink `CompositorAnimation` -> `blink::Animation::NotifyAnimationStarted`
  (resolves a pending start time; `PendingAnimations::NotifyCompositorAnimationStarted`)
  (`cc/animation/animation.cc`, `core/animation/animation.cc`,
  `pending_animations.cc`; lines: confirm).

## 3. Target on the S1b-7 interfaces

### 3.1 Reentrant access during BeginMainFrame (interface requirement)

`LayerizeEngine::BeginMainFrame(state, &mut dyn MainFrameClient)` holds
`&mut LayerizeEngine` while it calls Page's lifecycle. In Chromium, Blink
calls back into PAC, LayerTreeHost and AnimationHost from inside
`ProxyMain::BeginMainFrame`. Rust needs the client to receive that access as
a parameter:

```text
trait MainFrameClient<F> {
    /// LayerTreeHostClient::UpdateLayerTreeHost / BeginMainFrame lifecycle.
    fn UpdateRendering(&mut self, args: &BeginFrameArgs, main: &mut MainFrameLayerize<'_, F>);
    ...
}

/// The LayerizeEngine minus its ProxyMain stage bookkeeping, lent for one
/// lifecycle: exactly the entry points Blink reaches during the BMF.
struct MainFrameLayerize<'a, F> {
    UpdatePaintArtifact(..)                       // PushPaintArtifactToCompositor
    ApplyCompositedAnimationProperties(&update) -> bool   // PAC::DirectlyUpdate*
    ApplySourcePropertyAnimation(mutation)        // cc::Animation::AddKeyframeModel/Remove
    ApplySourceScrollAnimation(mutation)
    ApplyScrollbarPaintMutation(..) -> bool
    HasResidentPaintArtifact() -> bool
}
```

Calls that reach `LayerTreeHost::SetNeedsCommit` go to the endpoint, which
ProxyMain implements (S1B_PROXY_DESIGN B4). Inside the lent scope ProxyMain
reads `current_pipeline_stage == Animate` and only raises the final stage;
no message is posted. Outside a BMF, Page calls the same methods on its owned
`LayerizeEngine` (each one waits for the protected sequence first, decision
D2), and `SetNeedsCommit` posts `MainToImpl::SetNeedsCommit`.

If S1b-7 lands a different shape (for example the client receives
`&mut LayerizeEngine` minus a split-off `ProxyMain`), S1b-8 uses that shape.
What matters is that these calls are synchronous and happen inside the BMF
task.

### 3.2 Page flow after S1b-8

```text
doc  LayerizeEngine::BeginMainFrame(state, page)       current_stage = Animate
       ApplyCompositorChanges, ApplyMutatorEvents
       page.UpdateRendering(args, main)
         ... Style, Layout ...
         if direct-animation admissible (page.rs:10014-10030 rules kept)
            && main.HasResidentPaintArtifact()                    [gate, both modes]
           (prepared, update) = paint.PrePaintCompositedAnimationUpdate(..)
           if let Some(update):
             if main.ApplyCompositedAnimationProperties(&update):  [PAC::DirectlyUpdate*]
               for m in InstallCompositorPropertyAnimations(bindings): [one helper]
                 main.ApplySourcePropertyAnimation(m)
               commit retained paint lifecycle; return (no Paint, no DidPaint)
             else: fall through with `prepared` (None -> full PrePaint)   [2.1 fallback]
         full Paint -> main.UpdatePaintArtifact(artifact)
         main.ApplySourcePropertyAnimation(installs for full_paint_animation_bindings)
       final_stage: COMMIT (direct update) or UPDATE_LAYERS (new artifact)
       UpdateLayers only if final_stage >= UPDATE_LAYERS
       WillCommit -> post MainToImpl::NotifyReadyToCommit      [exactly one commit]
impl ProxyImpl: ReadyToCommit -> COMMIT -> FinishCommit (property trees +
     AnimationHost::PushPropertiesTo in the same commit) -> activate -> draw
```

Removals and replacements (`page.rs:9315-9339`) and `ScrollbarPaint`
(`pointer_scroll_controls.rs:209`) become calls on the same lent or owned
`LayerizeEngine`. `CompositorMutation::{CompositedAnimationProperties,
PropertyAnimation, ScrollbarPaint}` are no longer used by Page.

### 3.3 Gate

`external_compositor_published_frame_sequence` is replaced by
`LayerizeEngine::HasResidentPaintArtifact()` for both presentation modes.
This is the same predicate the internal mode uses today, and the one Blink
uses implicitly: PAC exists and has run `Update`. `PAC::CanDirectlyUpdateProperties`
stays the final authority, inside `ApplyCompositedAnimationProperties`. The
remaining admissibility rules (`page.rs:10014-10030`: dirty paint resources,
pending paint mutations, child-frame paint) are Page lifecycle rules and stay.

### 3.4 Fallback

A `false` from `ApplyCompositedAnimationProperties` is no longer an error.
Page continues into the full-paint path with `prepared_animation_prepaint`
(`page.rs:10340`, consumed at `:10382-10398`). The nodes were not downgraded
by `CompositorSimpleValuesUpdated`, so `PaintArtifactCompositor::Update`
sees them as changed (Blink's behaviour in 2.1). Partial application needs
care: today `CpuMainPaintServices::ApplyCompositedAnimationProperties`
(`paint_services.rs:795-808`) may have written some transforms before one
fails. Blink updates node by node and falls back per node. The Rust
translation should either check `CanDirectlyUpdateProperties` once before
writing (all nodes share the PAC and host, so it holds for all or none) and
treat a per-node `kInvalidPropertyNodeId` miss as "this node goes through
the full update", or keep the all-or-nothing bool and rely on the full
`Update` rewriting every node. Recommendation: the first, which is Blink's
per-node shape. The full Update is correct either way, because it rewrites
the cc nodes from the artifact.

### 3.5 KeyframeModel start time

`PropertyAnimationMutation::Install::start_time` is converted with the impl
half's executor clock (`page.rs:9447,10263`). After S1b-7 the external Page has
no `impl_half`. The conversion must use the document-side mapping of the same
cc `TimeTicks` domain. `PageCompositorClockMapping`
(`page.rs:2589,7787`; `TimeTicks` at `page.rs:244`) already exists for this and
is bound by the embedding. With internal presentation the executor clock and
the mapping are the same domain. Section 7, decision D-B, covers translating
the started event instead.

### 3.6 Outside a BeginMainFrame

Page's direct path runs only inside `UpdateFrameIfNeededInternal`, which after
S1b-7 is reached from `MainFrameClient::UpdateRendering` and from forced
lifecycles (screenshots, `UpdateAllLifecyclePhases`-style callers). In a
forced lifecycle the update is written into the main tree and `SetNeedsCommit`
posts a request; the next BMF commits it (2.2). No out-of-BMF finalize exists
any more (B1 is gone with S1b-7).

## 4. What is deleted

| Item | Location (7f978395) |
|---|---|
| `external_compositor_published_frame_sequence` field, init and resets | `page.rs:6826,7412,7770,7863` |
| `Page::DidPublishFrameToExternalCompositor` and its call | `page.rs:7777-7784`; `page_runtime.rs:3404` (may already be gone with the S1b-7 publication tail) |
| `PageRenderPipeline::CanApplyCompositedAnimationProperties` | `page.rs:1243-1250` |
| Inline KeyframeModel builder in the direct path (use `InstallCompositorPropertyAnimations`, returning mutations) | `page.rs:10079-10287` |
| The "publish mapping before KeyframeModel" ordering workaround | `page.rs:10290-10301` |
| `CompositorMutation::CompositedAnimationProperties` (and `PropertyAnimation`, `ScrollbarPaint` once Page no longer routes them) | `compositor/src/engine.rs:270-275`, arms `:713-721` |
| `CompositorEngine::CanApplyCompositedAnimationProperties` | `engine.rs:683-685` |
| `EngineAssembly::{ApplyCompositedAnimationProperties, PrepareCompositedAnimationProperties, CanApplyCompositedAnimationProperties}` | `engine_assembly.rs:325-380` |
| Coalesced follow-up in `Poll` | `engine_assembly.rs:1742-1752` |
| `HasPendingCompositedAnimation()` terms of `main_commit_pending` | `engine_assembly.rs:1804,1857` |
| `EngineCore::{CanPrepareCompositedAnimationCommit, PrepareCompositedAnimationCommit}` | `engine_core.rs:285-289,312-347` |
| `LayerizeEngine::{pending_composited_animation, CoalesceCompositedAnimationProperties, TakePendingCompositedAnimation, HasPendingCompositedAnimation}` | `layerize_engine.rs:113-115,292-331` |
| `ProxyMain::PrepareCommit(false)` property-only flag | `proxy_main.rs:71-79,189-195`, if S1b-7 did not already replace it with pipeline stages (S1b-4 deviation list) |
| Span `Compositor.Animation.CoalescePropertyUpdate` | `engine_assembly.rs:335-338` |
| `ImplCommand::InputContinuation` payloads for animation (the variant stays for input continuations) | `page_runtime.rs:296,1985-2005,3514-3516` |

Kept: `LayerizeEngine::ApplyCompositedAnimationProperties` (now returns the
per-node result, 3.4), `CpuMainPaintServices::ApplyCompositedAnimationProperties`,
the PAC/PropertyTreeManager translations, `PaintEngine::PrePaintCompositedAnimationUpdate`,
the `Compositor.Layerize.DirectAnimationPropertyUpdate` span (renamed into the
`layerize` trace category if S4 renames categories).

## 5. Steps

Each step ends with `cargo check -p rechrom_app` (one check per step, AGENTS.md
2.3). The order keeps each step bisectable: no step depends on a later deletion.

| Step | Change | Verification |
|---|---|---|
| **S1b-8a** Local apply | Page calls `ApplyCompositedAnimationProperties` on the lent or owned LayerizeEngine (3.1) in both modes instead of `RouteCompositorMutation(CompositedAnimationProperties)`. The gate becomes `HasResidentPaintArtifact()` (3.3). On `false`, fall through to full Paint (3.4). The KeyframeModel installs still use the existing route for this step. Delete the field and the `DidPublishFrameToExternalCompositor` call. | check; matrix animation pages (6.1) with trace |
| **S1b-8b** KeyframeModels local | `InstallCompositorPropertyAnimations` returns the mutations; both paths (direct and full) apply them through `ApplySourcePropertyAnimation`; removals/replacements likewise. Start time via `PageCompositorClockMapping` (3.5). Delete the inline duplicate and the ordering workaround. | check; animation pages; one-commit-per-frame trace check (6.2) |
| **S1b-8c** Delete coalescing | Remove the compositor-side entry points and the layerize coalescing fields (section 4 rows from `engine.rs`, `engine_assembly.rs`, `engine_core.rs`, `layerize_engine.rs`). The compiler finds any missed caller. | check; `cargo test -p layerize --test layerize_boundary` |
| **S1b-8d** ScrollbarPaint local (B5 remainder) | `pointer_scroll_controls.rs:209` calls `ApplyScrollbarPaintMutation` locally; the `ScrollbarPaintFallback` effect becomes the returned bool, handled in Page. | check; overlay-scrollbar fade page, scroll set |
| **S1b-8e** (optional, decision D-B) Started/finished events | Fill `DelegateAnimationStarted` / non-scroll `DelegateAnimationFinished` (`animation_host.rs:1691-1720`) so `ApplyMutatorEvents` returns property-animation events too; Page forwards them to `animation_engine` as Blink's `NotifyAnimationStarted`. Only then may Install omit `start_time`. | animation pages with `animation-delay` and a paused/resumed animation; pixel diff against Chromium |

The S1b-7 owner may already have moved `ScrollAnimation` /
`FinalizeScrollAnimations` and the `ScrollAnimationCommitPrepared` round trip
(`engine_assembly.rs:205-249`; observed at `page/scroll.rs:380`), because
they are tied to the deleted `ImplCommand::Commit`. If not, they belong with
S1b-8d (B5's last bullet in S1B_PROXY_DESIGN.md).

## 6. Verification

### 6.1 Pages

`rechrom-handoff/tools/matrix.ps1` has no animation pages. Add these local
fixture pages to the S1b-8 run, each with a Chromium screenshot at the same
animation time (use a paused animation or `animation-delay: -Ns;
animation-play-state: paused` for deterministic pixels, AGENTS.md 3.3):

1. Infinite `@keyframes` transform translate on a composited box (the
   direct-update hot path).
2. Opacity keyframes on a composited box.
3. Transform and opacity on the same element (two bindings, one node each).
4. A CSS transition started by a class toggle from script (install from the
   direct path on its first frame).
5. An animation removed mid-flight (`animation: none`) and one replaced by
   name (removal/replacement routes).
6. A full-paint change on the same frame as an animated sample (for example a
   text change next to the animated box): the fallback and mixed-commit case.
7. The first frame after navigation with an animation already running (gate
   before any resident artifact).
8. A forced lifecycle outside a BMF (DevTools `Page.captureScreenshot` while
   animating): the value must reach the next BMF's commit.

### 6.2 Trace criterion: one commit per animated frame

Record a headless DevTools trace for page 1 over at least 120 frames
(`--devtools-port`, Performance report, AGENTS.md 3.4). For every
`MainFrameCycle` interval that contains a
`Compositor.Layerize.DirectAnimationPropertyUpdate` span, require:

- exactly one `Compositor.Tree.Commit` span (one ReadyToCommit/FinishCommit)
  attributed to that BMF, and no `Compositor.Tree.Commit` outside a BMF;
- no UpdateLayers in that BMF (property-only commit). No UpdateLayers span
  exists today. Add a `LayerTreeHost.UpdateLayers` span in the
  `compositor.layerize` category before the slice build (AGENTS.md 3.1
  extends the trace before building, not after). The
  `PrePaint.DirectAnimationPropertyUpdate` span must be present with
  `rejection = 0`;
- no `Compositor.Animation.CoalescePropertyUpdate` span (deleted);
- no `BeginMainFrameAborted(kFinishedNoUpdates)` for these frames.

Also compare commit-to-activation latency and main-frame duration against the
S1b-7 baseline: the PAC direct update now runs on the document thread inside
the BMF and may wait on the protected sequence (R5 of S1B_PROXY_DESIGN.md).

### 6.3 Regression set

`scroll_fps.ps1` and the trackpad replay at DPR 2 (scrolling pages with
composited animations next to a scroller), because the BMF/commit cadence is
shared with scrolling. `logic_check.ps1` first, as the memory notes require.

## 7. Risks and decisions

Risks:

- **R1. Partial writes on rejection** (3.4). Without the per-node shape, a
  mid-update failure leaves some cc nodes updated and the others waiting for
  the full Update of the same lifecycle. The result is correct because Update
  rewrites them, but it differs from Blink's per-node path.
- **R2. Wait inside the lifecycle.** `ApplyCompositedAnimationProperties`
  outside a BMF may wait on the protected sequence (D2 waits on every
  main-tree access). Inside the BMF there is no open sequence (the previous
  commit completed before the BMF began, `main_frame_before_activation_enabled = false`).
  Forced lifecycles during a pending commit will block; measure with the
  `ProxyMain.WaitForCommitCompletion` span.
- **R3. Start time domain** (3.5). A wrong clock mapping shows as a
  phase-shifted animation, not as a failure. Page 4 (transition from script)
  and a pixel diff at a paused time catch it.
- **R4. Fewer frames but behaviour change.** Today a value that arrives while
  the slot is busy produces a second, "frame-late" commit (the coalesced
  follow-up). After S1b-8 the value lands in the next BMF's commit. Frame
  pacing of animations changes: compare animation frame publication
  intervals against Chromium, not only against the old Rechrom.
- **R5. Element-id mapping race.** The ordering workaround (`page.rs:10290`)
  existed for startup. With both mutations in one commit this race cannot
  happen, but only if Page applies the property update before the install in
  the same lifecycle. Keep that order in code even though it is no longer
  load-bearing.

Decisions for the user:

- **D-A. Per-node fallback.** Translate Blink's per-node
  `DirectlyUpdate*` with per-node fallback (recommended), or keep the
  all-or-nothing bool?
- **D-B. Started/finished events (S1b-8e).** Translate
  `AnimationDelegate::NotifyAnimationStarted/Finished` into the Blink
  animation engine now, or keep Page computing the start time? Recommended:
  defer to after S1b-9. It is a separate Blink animation-timing gap
  (`DelegateAnimationStarted` is empty today) and not needed for the thread
  move.
- **D-C. Scroll-animation round trip ownership.** S1b-7 or S1b-8d?
- **D-D. Animation fixture pages.** Add them to the repo
  (`apps/rechrom/fixtures/`), or keep them in the handoff tools next to
  `matrix.ps1`?
