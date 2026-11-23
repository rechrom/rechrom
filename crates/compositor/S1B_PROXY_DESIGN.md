# S1b design: ProxyMain / ProxyImpl across the document and compositor threads

Status: S1b-1..S1b-3 implemented (see "S1b status" at the end); designed
2026-11-23 at b084f58d. Originally nothing here was
implemented. Prerequisite: S1a (`layerize::LayerizeEngine<F>` holds the main
half, see `MAIN_IMPL_SPLIT.md`). The S1a leftovers (Send + Sync endpoint, real
protected-sequence synchronizer) are steps S1b-1/S1b-2 below.

Chromium revision `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`. Line numbers
for Rechrom refer to this worktree at b084f58d.

## 0. Target in one paragraph

Page owns `LayerizeEngine` on the `page-document-*` thread and runs it right
after Paint, inside a dispatched BeginMainFrame, exactly where Blink calls
`LocalFrameView::PushPaintArtifactToCompositor`
(`third_party/blink/renderer/core/frame/local_frame_view.cc:2965-2975`,
`PaintArtifactCompositor::Update` at `:3348`). The `page-compositor-*` thread
owns only the impl half (`CompositorEngine`: LayerTreeHostImpl, Scheduler,
input, tiles, raster, draw, frame sink). The only cross-thread data path for
the main tree is Chromium's commit:

- impl -> main: `BeginMainFrame(BeginMainFrameAndCommitState)` with the impl
  deltas and mutator events as **values**;
- main -> impl: `NotifyReadyToCommit(CommitState moved, protected lease of
  ThreadUnsafeCommitState, CompletionEvent)` or `BeginMainFrameAborted(reason)`;
- impl runs `FinishCommit` while main is parked on the protected sequence;
- display lists cross as `Arc` (already true: `CpuPaintRecord` leases,
  `layerize::commit::paint`).

`PaintArtifact` never crosses to the compositor thread any more.

## 1. Mapping table

Types to add. Main side lives in `layerize` (`layerize::runtime::proxy_main`),
impl side in `compositor` (`compositor::runtime::proxy_impl`), and every value
that crosses threads in `layerize::commit::proxy` (plus
`layerize::commit::completion_event`). `F` is the single `LayerizeFamily`
(`CpuCompositorTypes<SkRegion>`, exposed to `browser` as a facade alias
`compositor::MainFamily`).

### 1.1 Messages (`layerize::commit::proxy`)

| Chromium | Rust value | Notes |
|---|---|---|
| `BeginMainFrameAndCommitState` (`cc/trees/begin_main_frame_and_commit_state.h`; filled at `proxy_impl.cc:808-830`) | `BeginMainFrameAndCommitState { begin_frame_args, trace_id, commit_data: Box<CompositorCommitData>, mutator_events: Option<MutatorEvents>, completed_image_decode_requests, active_sequence_trackers, evicted_ui_resources, reason, consecutive_no_damage_main_frames }` | `CompositorCommitData` is already `layerize::commit::compositor_commit_data`. `MutatorEvents` (`compositor/src/runtime/host_owners.rs`) moves to commit as a plain value. |
| `CommitEarlyOutReason` (`cc/scheduler/commit_earlyout_reason.h`) | `commit::proxy::CommitEarlyOutReason` | git mv of `compositor/src/cc/scheduler/commit_earlyout_reason.rs`; compositor re-exports. |
| `ProxyMain::CommitPipelineStage` (`proxy_main.h`) | `commit::proxy::CommitPipelineStage { No, Animate, UpdateLayers, Commit }` | Replaces `EngineCore::prepared_update_layers` (`engine_core.rs:63`). |
| `CommitTimestamps` | existing `commit::commit_state::CommitTimestamps` | |
| `CompletionEvent` (`cc/base/completion_event.h:15-89`) | `commit::completion_event::CompletionEvent` = `Arc<(Mutex<State>, Condvar)>`; `Wait`, `TimedWait`, `IsSignaled`, `Signal` | Manual-reset (ProxyMain creates it with `ResetPolicy::MANUAL`, `proxy_main.cc:471`). Debug asserts for wait-once / signal-once as in the header. |
| `&unsafe_state` raw pointer + `commit_completion_event_` (`proxy_main.cc:474-477,555-560`) | `commit::proxy::ProtectedCommitLease<F>` | `unsafe impl Send`. Holds a `NonNull<CpuMainLayerHost<F>>`, `Arc<CpuMainPaintServices<F::Skia>>`, `Arc<dyn MainMutatorHost>`, the `CompletionEvent`, and `blocking: bool`. Only method: `WithCommitContext(f)` (same signature as `LayerizeEngine::WithCommitContext`, `layerize_engine.rs:315-324`). `Drop` signals the event (covers unwind and channel teardown). |
| `ProxyMain -> ProxyImpl` posted tasks | `enum MainToImpl<F>` | `Initialize { completion, init: ProxyImplInit }`, `SetNeedsCommit { reason, urgent, unthrottle }`, `SetNeedsRedraw(Rect)`, `SetDeferBeginMainFrame(bool)`, `SetPauseRendering { pause, delay_until_visibility_change }`, `SetVisible(bool)`, `SetTargetLocalSurfaceId(LocalSurfaceId)`, `NotifyNewLocalSurfaceIdExpectedWhilePaused`, `BeginMainFrameAborted { reason, main_thread_start_time, swap_promises, scroll_and_viewport_changes_synced }`, `NotifyReadyToCommit { lease: ProtectedCommitLease<F>, commit_state: Box<CommitState<F>>, main_thread_start_time, scroll_and_viewport_changes_synced, commit_timeout }`, `SetMutator(LayerTreeMutator)`, `QueueImageDecode {..}`, `Stop { completion }`. |
| `ProxyImpl -> ProxyMain` posted tasks | `enum ImplToMain` | `BeginMainFrame(Box<BeginMainFrameAndCommitState>)`, `DidCompleteCommit { source_frame_number, timestamps }`, `DidCommitAndDrawFrame(i32)`, `DidPresentCompositorFrame { frame_token, details }`, `NotifyImageDecodeRequestFinished`, `NotifyTransitionRequestFinished`, `DidCompletePageScaleAnimation`, `BeginMainFrameNotExpectedSoon`, `BeginMainFrameNotExpectedUntil(TimeTicks)`, `DidLoseLayerTreeFrameSink`. |

Input-routing values (`CompositorMainThreadEffect::WheelRequires*`, gesture
acks, `ExternalWheel`) are InputHandlerProxy / WidgetInputHandlerManager
traffic, not Proxy traffic, and keep their existing `Command`/`ImplCommand`
routes.

### 1.2 Methods

| Chromium (thread) | New Rust | Replaces today |
|---|---|---|
| `ProxyImpl::ScheduledActionSendBeginMainFrame` (`proxy_impl.cc:801-848`, impl) | `ProxyImpl::ScheduledActionSendBeginMainFrame`: `ProcessCompositorDeltasForPipeline(data_for_commit.is_some())`, `TakeMutatorEvents`, push `ImplToMain::BeginMainFrame` | `EngineCore::ScheduledActionSendBeginMainFrame` (`engine_core.rs:711-740`), which applies the deltas to `main` in place (`ApplyCompositorDeltas`, `:145-177`) and the events to the main AnimationHost (`ApplySourceAnimationEvents`, `:628-648`); the `Tick` -> `MainThreadEvent::BeginFrame` post (`page_runtime.rs:1873-1882`) |
| `ProxyMain::BeginMainFrame` (`proxy_main.cc:174-580`, main) | `ProxyMain::BeginMainFrame(state, &mut dyn MainFrameClient)`; the client is Page (`UpdateRendering` + Paint) | the run_document BMF arm (`page_runtime.rs:2842-2930`), the no-update heuristic (`:2899-2929`), the publication tail (`:3365-3405`) |
| `LayerTreeHost::ApplyCompositorChanges` (`layer_tree_host.cc:1207`, main) | `LayerizeEngine::ApplyCompositorChanges` (exists, `layerize_engine.rs:182`) called from `ProxyMain::BeginMainFrame` | call in `engine_core.rs:165` |
| `LayerTreeHost::ApplyMutatorEvents` (`:1254`, main) | `LayerizeEngine::ApplyMutatorEvents(MutatorEvents)` | `EngineCore::ApplySourceAnimationEvents` (`engine_core.rs:628-648`) |
| `LayerTreeHost::UpdateLayers` + `WillCommit(completion, has_updates)` (`layer_tree_host.cc:424-503`, main) | `LayerizeEngine::WillCommit(completion, has_updates, features, owner) -> Option<CommitState<F>>` | `LayerizeEngine::ActivateCommitState` (`layerize_engine.rs:296-312`), which always produces a state |
| `LayerTreeHost::WaitForProtectedSequenceCompletion` / `WaitForCommitCompletion` (`:520-538`, main) | `EngineMainEndpoint::WaitForProtectedSequenceCompletion` waits on the stored `CompletionEvent` and clears it | assert at `main_endpoint.rs:67-77` |
| `LayerTreeHost::IsOwnerThread` / `InProtectedSequence` (`:329-336`) | owner `ThreadId` captured at construction; `InProtectedSequence` = completion event present and unsignaled | `RunsTasksInCurrentSequence` checks at `main_endpoint.rs:49,61-66` |
| `LayerTreeHost::CommitComplete` (`:552-575`, main) | `LayerizeEngine::CommitComplete(source_frame_number, timestamps)` -> Page `EventTimingDidCommit` | `CompositorMainThreadEffect::MainFrameCommitCompleted` (`engine_core.rs:451-455`) relayed as `Command::EventTimingDidCommit` (`page_runtime.rs:1147-1153`) |
| `ProxyImpl::NotifyReadyToCommitOnImpl` (`proxy_impl.cc:425-493`, impl) | `ProxyImpl::NotifyReadyToCommit`: `NotifyBeginMainFrameStarted`, `host.ReadyToCommit(synced, ..)`, store `DataForCommit`, `scheduler.NotifyReadyToCommit`, and `SetNeedsBeginMainFrame` when `!synced` | `EngineCore::NotifyReadyToCommit` (`engine_core.rs:902-924`), `EngineAssembly::ScheduleFinalizedMainCommit` (`engine_assembly.rs:251-266`) |
| `ProxyImpl::DataForCommit` (`proxy_impl.h:243-263`) | `ProxyImpl::data_for_commit: Option<DataForCommit<F>> { completion: ScopedCommitCompletionEvent, commit_state, lease }` | `EngineCore::{ready_commit, ready_commit_notified}` (`engine_core.rs:51,69`) |
| `ScopedCommitCompletionEvent` (`proxy_impl.cc:42-81`) | `compositor::runtime::proxy_impl::ScopedCommitCompletionEvent`; `Drop` signals and, when main was not blocking, pushes `ImplToMain::DidCompleteCommit` | none |
| `ProxyImpl::ScheduledActionCommit` (`proxy_impl.cc:868-899`, impl) | `ProxyImpl::ScheduledActionCommit`: `BeginCommit`, `lease.WithCommitContext(FinishCommit)`, finish time, hold the completion for activation when `commit_waits_for_activation` | `EngineCore::Commit` (`engine_core.rs:424-458`) |
| `ProxyImpl::BeginMainFrameAbortedOnImpl` (`proxy_impl.cc:306-323`, impl) | `ProxyImpl::BeginMainFrameAborted(reason, start, swap_promises, synced)` with `next_bmf = data_for_commit.is_some()` | `EngineCore::FinishMainFrameWithoutUpdates` (`engine_core.rs:929-949`), `EngineCore::AbortDeferredMainFrame` (`:971-1003`), `ImplCommand::FinishMainFrameWithoutUpdates` (`page_runtime.rs:285-287`), `ImplPresentation::finish_main_frame_without_updates` (`:1088-1108`) |
| `ProxyMain::SetNeedsAnimate / SetNeedsUpdateLayers / SetNeedsCommit` + `SendCommitRequestToImplThreadIfNeeded` (`proxy_main.cc:715-756,1112-1144`, main) | `ProxyMain::{SetNeedsAnimate, SetNeedsUpdateLayers, SetNeedsCommit}`; `CpuMainHostEndpoint::SetNeedsCommit` calls them directly | `MainHostEffect::{NeedsCommit, NeedsUpdateLayers}` through `EngineMainHostEffects::PostEffect` (`execution.rs:281-283`); `FrameDemand::request_begin_frame` -> `ImplCommand::RequestMainFrame` (`page_runtime.rs:380-386`) |
| `ProxyImpl::SetNeedsCommitOnImpl` (`proxy_impl.cc:294-298`) | `ProxyImpl::SetNeedsCommit` -> `scheduler.SetNeedsBeginMainFrame` | `EngineAssembly::RequestMainFrame` (`engine_assembly.rs:281-285`) |
| `ProxyMain::SetDeferMainFrameUpdate` -> `ProxyImpl::SetDeferBeginMainFrameFromMain` (`proxy_main.cc:792-815`, `proxy_impl.cc:232-243`) | `ProxyMain::SetDeferMainFrameUpdate` posts `MainToImpl::SetDeferBeginMainFrame` | `FrameLifecycleMutation::SetDeferMainFrameUpdates` arm (`engine_assembly.rs:392-405`) |
| `ProxyMain::StartDeferringCommits / StopDeferringCommits / IsDeferringCommits` (`proxy_main.cc:848-890`, main only) | `ProxyMain` over the existing `MainFrameLifecycle` (`layerize/src/runtime/main_frame_lifecycle.rs`); no message | `engine_assembly.rs:406-438`, `EngineCore::ExpireCommitDeferral` (`engine_core.rs:958-969`) |
| `ProxyMain::SetVisible` -> `SetVisibleOnImpl` (`proxy_main.cc:701-706`, `proxy_impl.cc:325`) | `MainToImpl::SetVisible` | direct impl `SetVisible` in `EngineCore::SetViewportState` (`engine_core.rs:131-136`) |
| `ProxyMain::SetInputResponsePending` (`proxy_main.cc:840-846`) | `ProxyMain::SetInputResponsePending` (sets `block_on_next_commit`) | none |
| `ProxyMain::Start` / `InitializeOnImplThread` (`proxy_main.cc:898-914,80-89`) | `ProxyMain::Start` posts `MainToImpl::Initialize { completion, init }` and waits | `engine_factory::Create` building both halves on one thread (`engine_factory.rs:126-200`) |
| `ProxyMain::Stop` (`proxy_main.cc:916-946`) | `ProxyMain::Stop` posts `MainToImpl::Stop { completion }` and waits | fire-and-forget `ImplCommand::Stop` (`page_runtime.rs:292,731`) |
| `ProxyMain::SetMutator` -> `InitializeMutatorOnImpl` (`proxy_main.cc:959-965`, `proxy_impl.cc:185-190`) | `MainToImpl::SetMutator(LayerTreeMutator)` | `MutatorSetLayerTreeMutator` (`host_services.rs:2134`) |
| `AnimationHost::CreateMainInstance` / `CreateImplInstance`, `MutatorHost::PushPropertiesTo` (`cc/animation/animation_host.cc`) | `compositor::CreateMainAnimationHost() -> Arc<dyn layerize::commit::main_mutator::MainMutatorHost>` held by `LayerizeEngine`; ProxyImpl creates the impl instance during `Initialize`; `PushMutatorPropertiesTo` reads the main instance only through the lease | `AnimationHostOwner::default()` twice in `engine_factory.rs:162-163`; `PushMutatorPropertiesTo` (`animation_host.rs:940`) |
| `ProxyImpl::DidPresentCompositorFrameOnImplThread` -> `ProxyMain::DidPresentCompositorFrame` (`proxy_impl.cc:720`, `proxy_main.cc:606`) | `ImplToMain::DidPresentCompositorFrame` | `ImplCommand::PresentationFeedback` -> `Command::EventTimingDidPresent` (`page_runtime.rs:1823-1840`); route kept, value renamed |
| `SingleThreadProxy::DoBeginMainFrame` / `DoCommit` (`single_thread_proxy.cc:1172-1223,226-268`) | contrast only; see decision D1 | `EngineAssembly::CommitWithScrollAnimations` (`engine_assembly.rs:171-228`) emulates both shapes today |

## 2. Commit protocol and blocking semantics

### 2.1 Sequence

```text
impl  Scheduler: SEND_BEGIN_MAIN_FRAME
impl    ProxyImpl::ScheduledActionSendBeginMainFrame
          commit_data = ProcessCompositorDeltasForPipeline(next_bmf = data_for_commit.is_some())
          mutator_events = TakeMutatorEvents()
          post ImplToMain::BeginMainFrame(state) -------------------> doc queue (rendering priority)
impl  continues: input, impl animations, draws of the active tree, raster

doc   ProxyMain::BeginMainFrame(state)
        not visible           -> post BeginMainFrameAborted(kAbortedNotVisible, synced=false)
        defer / paused        -> post BeginMainFrameAborted(kAbortedDeferredMainFrameUpdate, false)
        final_stage = max_requested_stage; current_stage = Animate
        if !IsDeferringCommits: layerize.ApplyCompositorChanges(commit_data); synced = true
              (PAC scroll callbacks -> ScrollOffsetChanged / ViewportScaleChanged, applied
               to Page synchronously, same task)
        layerize.ApplyMutatorEvents(mutator_events)
        Page::UpdateRendering(args)            (Style, Layout, PrePaint, Paint)
          Paint -> layerize.UpdatePaintArtifact(artifact)   [PushPaintArtifactToCompositor]
                   (composited-animation direct updates also land here)
        skip_commit (deferring commits or animate_only)
                              -> post BeginMainFrameAborted(kAbortedDeferredCommit, synced)
        current_stage = UpdateLayers; updated = UpdateLayers()
        completion = CompletionEvent::new()
        state = layerize.WillCommit(completion.clone(), has_updates)
        None                  -> completion.Signal();
                                 post BeginMainFrameAborted(kFinishedNoUpdates, synced=true);
                                 layerize.CommitComplete(..)
        Some(state)           -> endpoint stores completion (in_commit = true)
                                 post MainToImpl::NotifyReadyToCommit { lease, state, .. } --> impl
                                 if block_on_next_commit: WaitForProtectedSequenceCompletion()
        DidBeginMainFrame; return to the document event loop

impl  ProxyImpl::NotifyReadyToCommit -> DataForCommit; scheduler.NotifyReadyToCommit
impl  Scheduler: COMMIT
        BeginCommit; lease.WithCommitContext(|unsafe, layers, id| FinishCommit(..))
        drop ScopedCommitCompletionEvent  -> Signal  (doc thread may touch the main tree again)
                                           -> ImplToMain::DidCompleteCommit if main did not block
impl  Scheduler: POST_COMMIT, ACTIVATE, DRAW (unchanged)

doc   ProxyMain::DidCompleteCommit -> layerize.CommitComplete -> Page::EventTimingDidCommit
```

### 2.2 What blocks, and for how long

- **The impl thread never waits for the document thread.** Every impl -> main
  message is a post. This matches Chromium: the only `CompletionEvent::Wait`
  calls in `proxy_main.cc` are on the main thread (`:785,910,932,940,987,1003`)
  and in the protected sequence (`layer_tree_host.cc:533`).
- **The document thread waits** in exactly these places:
  1. `WaitForProtectedSequenceCompletion`, the first time it touches protected
     main state after `NotifyReadyToCommit` (any `LayerizeEngine` method that
     borrows the main host, the PAC or main Layers: `UpdatePaintArtifact`,
     `ApplyCompositorChanges`, `ApplyCompositedAnimationProperties`,
     `ApplyScrollbarPaintMutation`, `CreateVideoSurfaceForeignLayer`,
     `SetViewportRectAndScale`, `WillCommit`, the next `BeginMainFrame`).
     Rust aliasing rules forbid even reads of the leased host while impl holds
     `&mut`, so S1b waits on every such entry, not only on writes (Chromium's
     `ProtectedSequenceReadable` reads are not available to us; see risk R2).
     Work that does not touch the main tree (JavaScript, input dispatch,
     Style, Layout, PrePaint, Paint into a new `PaintArtifact`, the
     `ConvertPaintArtifactSubsetToRecord` services path) runs concurrently
     with FinishCommit, as in Chromium.
  2. Right after posting ReadyToCommit when `block_on_next_commit`
     (`ProxyMain::SetInputResponsePending`, set by the input path when the
     frame carries an input response; `proxy_main.cc:552-566`).
  3. `ProxyMain::Start` and `ProxyMain::Stop` (one-off).
- **Duration.** From the ReadyToCommit post until impl drops the
  `ScopedCommitCompletionEvent` after FinishCommit: impl queue latency (impl
  input outranks ordinary tasks, `page_runtime.rs:1503-1508`) plus
  FinishCommit. With `main_frame_before_activation_enabled = false`
  (`compositor_scheduler/src/scheduler_settings.rs:75`,
  `scheduler_state_machine.rs:1138`), SEND_BEGIN_MAIN_FRAME is not issued
  while a pending tree exists, so ReadyToCommit never waits for raster of a
  previous pending tree. When `commit_waits_for_activation` is set the
  completion is held until activation (`proxy_impl.cc:886-893`); that is the
  only raster-length wait and it is Chromium's.
- **Translation of `CompletionEvent`.** `Mutex<State> + Condvar`, manual
  reset, `Wait` returns immediately if signaled, `Signal` from any thread. It
  is owned by `Arc` (C++ stores a raw pointer owned by
  `LayerTreeHost::commit_completion_event_`; `Arc` replaces the lifetime
  argument). `ProtectedCommitLease::drop` signals, so a panicking impl
  FinishCommit, or an impl thread that exits with ReadyToCommit still queued,
  releases the document thread instead of deadlocking it. The lease also sets
  a `poisoned` flag in that case; `WaitForProtectedSequenceCompletion` then
  returns `Err(ImplTerminated)` and the document turns it into the existing
  PageError path instead of touching a half-pushed host.

### 2.3 Impl input and draw while main is busy

- Between BeginMainFrame and ReadyToCommit the Scheduler is in
  `BEGIN_MAIN_FRAME_SENT`; impl still handles wheel/gesture scrolling
  (`InputHandlerProxy` path in `run_impl`, `page_runtime.rs:1955-2000`),
  ticks impl animations and draws the active tree on every BeginFrame. New
  deltas accumulate in the impl tree; the pipelined BMF pulls them with
  `next_bmf = data_for_commit.is_some()` (`proxy_impl.cc:812-815`).
- Between ReadyToCommit and COMMIT the impl thread only runs FinishCommit when
  the Scheduler selects it; draws of the old active tree continue until
  activation, as today.
- The main thread's long Paint does not delay impl frames: the Scheduler's
  BMF deadline handling (`ShouldDraw` without the main frame) is unchanged.
  Today the same holds only because the main half runs on impl synchronously
  inside `ImplCommand::Commit`, which blocks impl input for the PAC Update
  (`page_runtime.rs:1527-1533` notes this). S1b removes that block: PAC
  Update and UpdateLayers move off the impl thread.

## 3. The blockers

### B1. Layerize only inside a dispatched BMF

Today `EngineAssembly::CommitWithScrollAnimations` pulls impl deltas
synchronously when an artifact arrives without a BMF
(`engine_assembly.rs:186-199`), and `ImplPresentation` requests a BMF for an
unsolicited artifact (`page_runtime.rs:1925-1948`, `needs_main_frame_for` /
`is_unsolicited` at `:1159-1167`).

Resolution:
- `LayerizeEngine::UpdatePaintArtifact` is called only from
  `ProxyMain::BeginMainFrame` (asserted: `current_pipeline_stage ==
  Animate`/`UpdateLayers`). Page already keeps a newer resident artifact until
  the next BMF on the external path (`page_runtime.rs:3367-3372`); that rule
  becomes the only rule.
- Paint produced outside a BMF (input or resource tasks) marks
  `ProxyMain::SetNeedsUpdateLayers` instead of committing; the artifact is
  pushed by the next `BeginMainFrame`.
- Delete the `!awaiting_main` delta pull (`engine_assembly.rs:186-199`) and
  the unsolicited handling (`page_runtime.rs:1920-1955`, `:1159-1167`) with
  the facade `Commit` / `CommitWithScrollAnimations`
  (`compositor/src/engine.rs:558-575`) once no caller remains (S1b-7).
- The composited-animation direct path outside a BMF
  (`EngineAssembly::PrepareCompositedAnimationProperties`, `!awaiting_main`
  branch at `engine_assembly.rs:375-380`) disappears with B5: the direct
  update is applied to the main tree and calls `SetNeedsCommit`; the commit
  happens in the next BMF.

### B2. Commit admission on the document thread

Today `ImplPresentation` admits an artifact by `main_frame_epoch` and
`required_scroll_sequence` (`page_runtime.rs:835-866` fields, `:1197-1208`
check, sequence taken from PAC `ScrollOffsetChanged` at `:1118-1127`), and
`PageRenderPipeline` keeps `pending` / `presentable`
(`browser/src/page.rs:331-336`) keyed on `main_commit_pending`
(`page.rs:928-950`, `:1930-2047`).

Resolution: the epoch/sequence machinery is a substitute for "the main half
runs in the same task as the BMF it answers". Once
`ApplyCompositorChanges`, Paint and WillCommit run in one
`ProxyMain::BeginMainFrame` call, the pairing is structural:
- `main_frame_epoch` -> the BMF itself. `Command::BeginMainFrame(state)`
  replaces `MainThreadEvent::BeginFrame { main_frame_epoch }`
  (`page_runtime.rs:358-367`) and every `main_frame_epoch` field of
  `Command::{ExternalWheel, ExternalCanonicalWheel, CompositorEffects}`
  (`:192-213`). `ImplPresentation::{required,responded}_main_frame_epoch`,
  `require_main_frame`, `main_frame_pending`, `needs_main_frame_for`,
  `is_unsolicited` and `commit` (`:860-861,908-910,1085-1087,1159-1236`) are
  deleted. `run_document`'s `completed_main_frame_epoch` / `published` tuple
  (`:2709,2723,3376-3405`) is deleted.
- `required_scroll_sequence` (`:862,1126,1206`) -> nothing. The rollback it
  guards (`page_runtime.rs:1199-1203`: an older artifact answering a newer BMF
  advances `SyncedScrollOffset`'s reflected slot without the delta) cannot
  happen because the artifact committed by a BMF is painted after that BMF's
  `ApplyCompositorChanges`. Keep `compositor_scroll_sequence` in
  `ContentFrameProvenance` for traces only.
- `PageRenderPipeline::pending` (`page.rs:331`) -> `ProxyMain` in-commit state
  (`commit_completion` in the endpoint). The rollback guarded at
  `page.rs:2019-2025` (a newer artifact replacing a ready commit before
  FinishCommit) is prevented by `WaitForProtectedSequenceCompletion` before the
  next PAC Update, which is exactly the Chromium mechanism.
- `PageRenderPipeline::presentable` and the run_document no-update heuristic
  (`page_runtime.rs:2899-2929`) -> `WillCommit(has_updates)` returning `None`
  (`layer_tree_host.cc:470-483`) plus `kFinishedNoUpdates`. See decision D3 on
  whether S1b translates `has_updates` fully or moves the heuristic as is.
- The `WouldBlock` retry protocol of `CommitArtifactInternal`
  (`page.rs:1943-1966,2030-2034`) and `ImplPresentation::publish`
  (`page_runtime.rs:1238-1272`) is deleted: a commit can no longer be offered
  while the previous one is in flight.
- What stays on impl in `ImplPresentation`: input routing, gesture queue,
  fling, child-frame queue, resize provenance, VisualProperties ACK and frame
  flushing (`flush_frames`). Child frames for video and resize provenance
  ride in `MainToImpl::NotifyReadyToCommit` (they belong to the main frame;
  `queue_child_frames` keeps its latest-per-surface rule).
- Viewport (`CommitArtifactInternal` computing `CompositorViewportState`,
  `page.rs:1895-1941`, and `EngineCore::SetViewportState`,
  `engine_core.rs:103-138`) moves to the document thread: LayerTreeHost owns
  `SetViewportRectAndScale` and the parent `LocalSurfaceId` (Blink sets them
  from VisualProperties on main). Only `SetVisible` and
  `SetTargetLocalSurfaceId` cross as messages.

### B3. Scheduler actions split main / impl

`SchedulerClient for EngineCore` (`engine_core.rs:677-900`) keeps only impl
work. Concretely:
- `ScheduledActionSendBeginMainFrame` (`:711-740`): keep
  `lifecycle.DidBeginMainFrame`, `NotifyBeginMainFrameStarted` moves to
  `NotifyReadyToCommit`/`BeginMainFrameAborted` as in
  `proxy_impl.cc:318,470`; remove `ExpireCommitDeferral`,
  `MainFrameLifecycle().IsDeferringCommits()`, `ApplyCompositorDeltas`,
  `ApplySourceAnimationEvents` and the `prepared_paint_commit` /
  `had_unnotified_commit` branches; build and post the BMF state.
- `awaiting_main` (`:70`) -> `scheduler.CommitPending()` (the source state;
  `proxy_impl.cc:318` DCHECKs it). `ready_commit` / `ready_commit_notified`
  (`:51,69`) -> `data_for_commit`. `prepared_paint_commit`,
  `prepared_update_layers`, `main_frame_applied_deltas` (`:58,63,71`) ->
  `ProxyMain::{current_pipeline_stage, final_pipeline_stage}` and the
  `scroll_and_viewport_changes_synced` argument.
- `NotifyReadyToCommit` (`:902-924`) becomes the `MainToImpl` handler;
  `FinishMainFrameWithoutUpdates` (`:929-949`) and `AbortDeferredMainFrame`
  (`:971-1003`) become one `BeginMainFrameAborted(reason, synced)` handler.
  `CommitDeferralChanged` / `MainFrameCommitDeferred` effects are produced on
  main by ProxyMain.
- `EngineCore::layerize` (`:42`) and `main_effects` (`:46`) leave EngineCore.
  The impl paint value services lease (`CpuImplPaintServices::new`) arrives in
  `ProxyImplInit`.

### B4. Cross-thread protected sequence

`EngineMainEndpoint` (`layerize/src/runtime/main_endpoint.rs`):
- Replace `protected_sequence: AtomicBool` (`:24`) with
  `owner: std::thread::ThreadId` and `commit_completion: Mutex<Option<CompletionEvent>>`.
- `EnterProtectedSequence` (`:48-52`) -> `BeginProtectedSequence(&self,
  completion)` called by `WillCommit` on the owner thread (asserts owner and
  no outstanding completion, `layer_tree_host.cc:428`). The end of the
  sequence is the impl-side `Signal`, not a guard drop on the owner thread
  (`protected_sequence_synchronizer.h`: started on owner, concluded on
  non-owner).
- `IsOwnerThread` (`:61-63`) = `thread::current().id() == owner`;
  `InProtectedSequence` (`:64-66`) = completion present and unsignaled (from
  any thread); `WaitForProtectedSequenceCompletion` (`:67-77`) = wait, then
  clear (`layer_tree_host.cc:526-537`).
- `CpuMainHostEndpoint` (`main_host.rs:54`) and `MainHostEffectSink`
  (`commit/main_effects.rs:61`) gain `Send + Sync`. The sink no longer posts
  into the compositor `EngineTaskQueue` (`execution.rs:281-283`): ProxyMain
  implements it (`PostEffect` -> `SetNeedsCommit` /
  `SetNeedsUpdateLayers` / `SetTargetLocalSurfaceId` / ...).
- `LayerizeEngine::main` becomes `Box<CpuMainLayerHost<F>>` so the lease
  pointer survives moves of the engine; `LayerizeEngine::drop` waits for the
  protected sequence first (Chromium: `ProxyMain::Stop` waits).
- Static assertions (compile-time, no test): `CpuMainLayerHost<Root>: Send`,
  `ThreadUnsafeCommitState<Root>: Sync`, `CommitState<Root>: Send`,
  `CpuMainPaintServices<SkRegion>: Send + Sync`, `EngineMainEndpoint: Send +
  Sync`. Layers are already `Send + Sync` (`layerize/src/cc/layers/layer.rs:184`).

### B5. Composited animations local on the document thread

Today Page gates the direct transform/opacity update on
`external_compositor_published_frame_sequence == frame.sequence`
(`browser/src/page.rs:9983-9996`) and routes the update to the compositor
thread as `CompositorMutation::CompositedAnimationProperties`
(`page.rs:10247-10249`, `page/scroll.rs:790-814` pushes it to
`pending_compositor_mutations` for `ImplCommand::InputContinuation`,
`page_runtime.rs:3509-3526`). Impl coalesces updates while the commit slot is
busy (`engine_assembly.rs:331-356`, `:1762,1817,1859`).

Resolution (Blink `PaintArtifactCompositor::DirectlyUpdate*` runs on main and
calls `LayerTreeHost::SetNeedsCommit`):
- The gate becomes `layerize.HasResidentPaintArtifact()` for both presentation
  modes; `external_compositor_published_frame_sequence`,
  `DidPublishFrameToExternalCompositor` (`page.rs:6778,7729-7736`) are
  deleted.
- `CompositedAnimationProperties` calls
  `layerize.ApplyCompositedAnimationProperties(update)` (`layerize_engine.rs:216`)
  directly; that waits for the protected sequence if one is open, then the
  endpoint's `SetNeedsCommit` reaches ProxyMain (no-op while
  `current_pipeline_stage == Animate`, `proxy_main.cc:741-751`).
- Delete `CoalesceCompositedAnimationProperties`,
  `TakePendingCompositedAnimation`, `HasPendingCompositedAnimation`
  (`layerize_engine.rs:233-268`) and their impl callers: the main tree has its
  own pending CommitState (`ActivateCommitState` copies it,
  `layer_tree_host.cc:505-518`), so there is no occupied slot to coalesce
  against.
- `PropertyAnimation`, `ScrollAnimation` and `FinalizeScrollAnimations`
  mutations (`compositor/src/engine.rs:275-280`) are main AnimationHost
  mutations: Page applies them to `layerize.MutatorHost()` locally
  (`EngineCore::ApplySource{Scroll,Property}Animation`,
  `engine_core.rs:605-627`, move to LayerizeEngine). The
  `ScrollAnimationCommitPrepared` / `Finalized` round trip
  (`engine_assembly.rs:205-249`) becomes a synchronous call within the BMF.
  `ScrollbarPaint` likewise (`engine_core.rs:304-320`).

### B6. Non-Send AnimationHostOwner / mutator backend

`AnimationHostOwner` (`compositor/src/runtime/animation_host.rs:103-109`) is
`Arc<Mutex<AnimationState>>`; it is `Send + Sync` only if `AnimationState:
Send`. `AnimationState::layer_tree_mutator: Option<Box<LayerTreeMutator>>`
(`:87`) wraps `Box<dyn LayerTreeMutatorBackend>` without `Send`
(`host_owners.rs:528-531`).

Resolution, following Chromium:
- `pub trait LayerTreeMutatorBackend: Send` (`host_owners.rs:528`). Chromium
  hands the mutator from main to impl by PostTask (`proxy_main.cc:959-965`),
  so a sendable bound is the source contract.
- The mutator belongs to the impl AnimationHost only
  (`ProxyImpl::InitializeMutatorOnImpl` -> `LayerTreeHostImpl::SetLayerTreeMutator`):
  assert in `SetLayerTreeMutator` (`animation_host.rs:1995`) that the owner is
  an impl instance.
- Main and impl are already distinct owners (`engine_factory.rs:162-163`;
  asserted in `PushMutatorPropertiesTo`, `animation_host.rs:953-956`). Add
  `layerize::commit::main_mutator::MainMutatorHost: Send + Sync` (the main
  AnimationHost API the main half calls: `ApplySourceScrollAnimation`,
  `ApplySourcePropertyAnimation`, `ApplyMutatorEvents`,
  `TakeSourceScrollCompletions`, `TakeNeedsCommit`, `IsElementAnimating`,
  `RemoveStaleTimelines`), implemented by `AnimationHostOwner`;
  `compositor::CreateMainAnimationHost()` creates it for Page
  (`AnimationHost::CreateMainInstance`), and ProxyImpl creates the impl
  instance during `Initialize` (`CreateImplInstance`).
- `MainHostEffectSink::IsElementAnimating` (`commit/main_effects.rs:65`)
  reads the main instance locally on the document thread.
- `EngineMainHostEffects` (`execution.rs:242-296`) is deleted once ProxyMain
  implements the sink.
- If the static `Send` assertion reports other members (candidates:
  `ScrollOffsetAnimationsImpl`, `ScrollTimelineRef`, `AnimationRef<Curve>`),
  fix them the same way: the translated cc objects are thread-confined by
  protocol, not by type, so their Rust types must be `Send`.

## 4. Thread and ownership changes, ordered into commits

Each step must pass `cargo check -p rechrom_app` (feature unification) when
the step is complete. Verification column: what the step needs before the
next one starts. "Matrix" = AGENTS.md 3.1 tier 2 on one
`target/iteration/rechrom` headless process. "Scroll set" = the
`apps/rechrom/fixtures/trackpad_scroll.jsonl` replay headless with
`--force-device-scale-factor 2` (final offset plus commit/publication
intervals), the two rollback cases (`page_runtime.rs:1199-1203`: impl scroll
during a long main frame, then a new artifact; `page.rs:2019-2025`: two
artifacts while a commit is pending), `resize_fps.ps1` / `scroll_fps.ps1` and
input latency from the DevTools Performance report
(`C:\Users\Administrator\rechrom-handoff\tools`, run `logic_check.ps1` first).

| Step | Change | Verification |
|---|---|---|
| **S1b-1** Send groundwork | B6 (`LayerTreeMutatorBackend: Send`, impl-only mutator assert, any further `Send` fixes), `Send + Sync` on `CpuMainHostEndpoint` / `MainHostEffectSink`, `LayerizeEngine::main` boxed, static assertions of B4. No behaviour change. | `cargo check -p rechrom_app`; `cargo test -p layerize --lib` |
| **S1b-2** Real synchronizer | `commit::completion_event::CompletionEvent`; endpoint of B4 (owner ThreadId, stored completion, real wait, poisoned flag); `ProtectedCommitLease<F>`; `EngineCore::Commit` uses the lease. Still one thread, so the wait never blocks. One small cross-thread unit test for the synchronizer (a new uncovered concurrency guard; AGENTS.md 2.3 allows it). | check; layerize unit tests; Matrix smoke |
| **S1b-3** Messages | `layerize::commit::proxy` (table 1.1), `CommitEarlyOutReason` git mv, `MutatorEvents` value move. Types only. | check |
| **S1b-4** In-process ProxyMain / ProxyImpl | `layerize::runtime::proxy_main::ProxyMain` (pipeline stages, defer/pause, `block_on_next_commit`, commit request bits) and `compositor::runtime::proxy_impl::ProxyImpl` (`data_for_commit`, `ScopedCommitCompletionEvent`). B3 applied: SchedulerClient touches only impl state; EngineAssembly connects them with an in-process `VecDeque` transport drained in the same `Poll`, preserving today's call order exactly (this is the old S1b of `MAIN_IMPL_SPLIT.md`). `LayerizeEngine::WillCommit` returns `Option` but the caller still forces `has_updates = true` (D3). | Matrix; Scroll set (must be identical to baseline: single thread) |
| **S1b-5** Main AnimationHost on the main side | `MainMutatorHost`, `CreateMainAnimationHost`, animation mutations and mutator events move into `LayerizeEngine` (B5 last bullet, B6); `EngineMainHostEffects` deleted; ProxyImpl reads main animations only through the lease. | Matrix (CSS transitions, smooth scroll, scroll-linked animations); Scroll set |
| **S1b-6** Page owns the main half (single thread) | Split `PageRenderPipeline` (`page.rs:292-337`) into `PageMainCompositor` (document: `LayerizeEngine`, `ProxyMain`, viewport/LocalSurfaceId, video foreign layers, drag-image records) and `PageImplCompositor` (impl: `CompositorEngine`, raster engine, Viz/Renderer for internal presentation, outbound frames). The document-thread second compositor (`state.rendering`, used by `PublishVideoFrame` `page.rs:613-656` and `drag_image.rs:160-170`) becomes the real `PageMainCompositor`. Internal presentation keeps both halves in Page with the in-process transport. External runtime: still one `PageMainCompositor` + `PageImplCompositor` pair on the compositor thread (temporary), Page's copy only serves video/drag as before. Page calls layerize from its Paint lifecycle inside `ProxyMain::BeginMainFrame`. | Matrix; Scroll set; video and drag-image pages |
| **S1b-7** Thread move (external) | `page_runtime.rs`: `PageMainCompositor` moves into Page on `page-document-*` (`run_document`, `:2659-2705`: create it in `SetExternalCompositor(true)` with an mpsc transport over `impl_sender`; `ProxyMain::Start` posts `Initialize` and waits). `run_impl` (`:1411-1467`) builds `PageImplCompositor` from `ProxyImplInit`. `ImplCommand` (`:250-293`): add `Proxy(MainToImpl<MainFamily>)`, delete `Commit`, `FinishMainFrameWithoutUpdates`, `RequestMainFrame`. `Command` (`:159-249`): add `BeginMainFrame(Box<..>)` and `ProxyMain(ImplToMain)`, delete `MainThreadEvent::BeginFrame`'s epoch and the `main_frame_epoch` fields; `FrameDemand` (`:368-395`) calls `ProxyMain::SetNeedsAnimate`. B1 and B2 applied (delete the `ImplPresentation` admission fields and methods, the Commit arm `:1884-1955`, the publication tail `:3365-3405`, `CommitArtifactInternal`'s retry, the delta pull `engine_assembly.rs:186-199`). The impl `ScrollApplied` trace (`page_runtime.rs:1111-1145`) is re-sourced from impl `DidCompositorScroll`. `DocumentQueue` treats `BeginMainFrame` as the rendering opportunity. | Full Matrix; Scroll set at DPR 2 with commit/publication intervals compared against the S1b-6 baseline; both rollback cases; `resize_fps`; input latency p50/p95; pixel diff of the matrix pages (AGENTS.md 3.3) |
| **S1b-8** Composited animations local | B5 (gate, direct apply, delete coalescing and `external_compositor_published_frame_sequence`). | Matrix animation pages; `scroll_fps`; trace shows one commit per animated frame |
| **S1b-9** Cleanup and contract | Remove `LAYERIZE_ENGINE_ALLOWED` (`layerize/tests/layerize_boundary.rs:25`); EngineCore no longer names `layerize::LayerizeEngine`; delete facade `Commit` / `CommitWithScrollAnimations` / `FinishMainFrameWithoutUpdates` / `RequestMainFrame` / `CanApplyCompositedAnimationProperties` (`engine.rs:558-605`); update `MAIN_IMPL_SPLIT.md` and AGENTS.md 1.2 boundary 5 / 1.3 Compositor (D4). | `cargo test --test layerize_boundary`; Matrix; then fat-LTO Release and a visible-window trackpad check (AGENTS.md 3.5) |

S1b-6 and S1b-7 are the large ones. S1b-7 cannot be split further without a
temporary synchronous cross-thread bridge, which is exactly what this design
avoids; it is the step that needs the headless scroll evidence.

## 5. Risks and open questions

Decisions needed:

- **D1. Internal (synchronous) presentation.** Chromium uses
  `SingleThreadProxy` for single-threaded embeddings
  (`single_thread_proxy.cc:1068-1259`): one object, BMF and commit in the same
  call. Options: (a) internal presentation also runs `ProxyMain`/`ProxyImpl`
  over the in-process transport (one Rechrom code path, ordering equal to
  SingleThreadProxy because both halves run back to back); (b) translate
  `SingleThreadProxy` as a second proxy (Chromium's own branch, more code).
  Recommendation: (a) in S1b, revisit (b) only if a test embedding needs
  SingleThreadProxy-specific behaviour.
- **D2. Wait granularity.** Rust cannot express `ProtectedSequenceReadable`
  reads safely while impl holds `&mut` of the host. S1b waits on every
  `LayerizeEngine` entry that borrows the main tree (coarser than Chromium).
  The finer split (readable/writable fields behind cell types) is a later
  optimisation. Accept?
- **D3. `has_updates`.** S1b-4 keeps `has_updates = true` and S1b-7 moves the
  run_document artifact-identity heuristic (`page_runtime.rs:2899-2929`) into
  `ProxyMain` unchanged, or S1b-7 translates `UpdateLayers()`'s return value
  and `WillCommit`'s `has_updates` (`proxy_main.cc:455-505`,
  `layer_tree_host.cc:470-483`) and deletes the heuristic. The second is the
  Chromium path and removes a known source of rollback bugs but changes
  no-update behaviour in the same step as the thread move. Recommendation:
  translate it in S1b-7, because the heuristic depends on the `published`
  tuple that S1b-7 deletes anyway.
- **D4. Boundary 5 contract.** AGENTS.md 1.2 boundary 5 is
  `CompositorEngine::Commit(Arc<PaintArtifact>, ..)`. After S1b the Paint ->
  Compositor boundary is Paint -> `LayerizeEngine` (document thread, borrowed
  `Arc<PaintArtifact>`) and Layerize -> Compositor is `CommitState` +
  protected lease. This changes the durable contract text; approve updating
  AGENTS.md in S1b-9.
- **D5. `block_on_next_commit`.** Chromium blocks main after ReadyToCommit
  only for input-response frames. Should S1b-7 wire
  `SetInputResponsePending` from the frame-aligned input path (closer to
  Chromium latency behaviour) or leave it off initially (simpler, measured
  separately)?

Risks:

- **R1. Soundness of the lease.** `unsafe impl Send` for a raw pointer into the
  document thread's host. It is sound only if every document-side access to
  the host goes through a waiting entry point. Mitigation: the host is
  reachable only through `LayerizeEngine` methods that call
  `WaitForProtectedSequenceCompletion` first (one private accessor
  `fn main_mut(&mut self)` that waits; `Main()`/`MainMut()` at
  `layerize_engine.rs:118-123` become waiting accessors), plus debug
  assertions `!InProtectedSequence()` in the accessor.
- **R2. Hidden shared state.** Main Layers are `Send + Sync`, but anything in
  them using `Rc`/`RefCell` or a `Cell` touched by both FinishCommit and main
  would race. The S1b-1 static assertions find type-level problems; logical
  sharing through `Arc<Mutex<..>>` (e.g. `AnimationHostOwner`) stays correct
  but may contend.
- **R3. Deadlock.** The document thread waiting on the protected sequence must
  not hold a lock the impl thread needs (routes `Mutex`, `VisualPropertiesThrottle`,
  trace buffers). ProxyMain waits only at the entry of LayerizeEngine calls,
  outside those locks; review `run_document` call sites in S1b-7.
- **R4. Scroll regressions.** The epoch and scroll-sequence guards are
  removed in one step (S1b-7). The structural argument (B2) says they become
  unnecessary, but the rollback cases at `page_runtime.rs:1199-1203` and
  `page.rs:2019-2025` must be replayed before and after; keep the old
  `ContentFrameProvenance.compositor_scroll_sequence` in traces to compare.
- **R5. Latency shape changes.** PAC Update and UpdateLayers leave the impl
  thread (good for impl input), but the document thread now pays them and may
  wait on FinishCommit. Measure main-frame duration, commit wait time (new
  trace span `ProxyMain.WaitForCommitCompletion`, category `compositor_layerize`)
  and input-to-frame latency against the S1b-6 baseline.
- **R6. Viewport ordering.** Moving `SetViewportRectAndScale` / LocalSurfaceId
  allocation to the document thread changes when a resize reaches the impl
  tree (now with the commit, as in Chromium). `resize_fps` and the
  VisualProperties ACK path (`page_runtime.rs:398-450`) must be rechecked.
- **R7. Document-thread compositor duplicate.** Until S1b-7 the external
  runtime still has two compositor instances (the Page-side one serving video
  and drag images). S1b-6 must not let them diverge further; S1b-7 removes the
  duplicate.

## S1b status

Decisions adopted by the coordinator: D1 (a) same ProxyMain/ProxyImpl over an
in-process queue for internal presentation; D2 one waiting accessor that
blocks while impl holds the commit lease; D3 `has_updates` translated in
S1b-7 (S1b-4 still forces `true`); D4 AGENTS.md updated in S1b-9; D5
`block_on_next_commit` on the input path deferred.

- **S1b-1 done (Send groundwork).** `LayerTreeMutatorBackend: Send`;
  `AnimationHostOwner` records Chromium's `ThreadInstance`
  (`CreateMainInstance`, `CreateImplInstance`), and `SetLayerTreeMutator`
  asserts the impl instance. `CpuMainHostEndpoint` and `MainHostEffectSink`
  are `Send + Sync`; `LayerizeEngine::main` is boxed. Compile-time
  assertions: `EngineMainEndpoint`, `CpuMainPaintServices<SkRegion>: Send +
  Sync` (layerize_engine.rs); main host `Send`, `ThreadUnsafeCommitState:
  Sync`, `CommitState: Send` for the CPU family (engine_core.rs). The
  assertions found (R2) and S1b-1 fixed: `SyncedScrollOffsetMap` is
  `Arc<RwLock<..>>` (was `Rc<RefCell<..>>`, both property-tree copies);
  `ScrollCallbacks: Send + Sync` with `sync::Weak` and the PAC scroll
  callbacks/effects on `Arc`/`Mutex`/atomic; `Send` on `SwapPromise`,
  `EventMetricsBackend`, `MicroBenchmarkBackend` and the presentation
  callback boxes; the compositor `ResourceEffects` queue is
  `Arc<Mutex<..>>` (ReportTimeSwapPromise pushes into it after moving with
  the commit). `cargo check -p rechrom_app` passes; the layerize unit tests
  were not run (memory limit; check only).
- **S1b-2 done (real synchronizer, one thread).**
  `commit::completion_event::CompletionEvent` translates
  cc/base/completion_event.h (manual reset, `Mutex + Condvar`, `Arc`
  shared; Rust extension `SignalAbandoned` / `WasAbandoned`). The endpoint
  stores the commit completion (`BeginProtectedSequence` from WillCommit,
  `InProtectedSequence` = stored and unsignaled, `WaitForCommitCompletion`
  waits and clears, `CommitComplete` retires a signaled one,
  `WaitForCommitRelease` for teardown); `IsOwnerThread` stays the sink's
  sequence check minus the impl's lease scope. While both halves share the
  sequence (`MainHostInit::impl_on_owner_sequence = true`, set by the
  factory), `WaitForProtectedSequenceCompletion` returns immediately, as
  Chromium's SINGLE_THREADED branch (layer_tree_host.cc:521-522): the
  current single-thread flow still touches the main tree between WillCommit
  and COMMIT (e.g. ApplyCompositorDeltas of a pipelined BMF), so a real wait
  would self-deadlock. `commit::proxy::ProtectedCommitLease<F>` (raw host
  pointer, paint services, endpoint, completion, `blocking`; `unsafe impl
  Send`; `Drop` signals, abandoned if FinishCommit never ran;
  `TakeCompletion` for S1b-4's ScopedCommitCompletionEvent).
  `LayerizeEngine` owns the host as `MainHostOwner` (raw allocation) whose
  `Get`/`GetMut` are the single waiting accessor (D2) for every main-tree
  access, including `Main()`/`MainMut()`; `ActivateCommitState` takes the
  completion and returns `(CommitState, ProtectedCommitLease)`;
  `WithCommitContext` is gone; `Drop` waits for the lease. EngineCore keeps
  `ready_commit = (state, lease)`, commits through the lease, drops it and
  calls `LayerizeEngine::CommitComplete`. Not done: the cross-thread unit
  test for the synchronizer (tests are not built on this machine).
- **S1b-3 done (messages, types only).** `commit::proxy` holds
  `BeginMainFrameAndCommitState<F>` (cc/trees/proxy_common.h),
  `CommitPipelineStage`, `MainToImpl<F>`, `ImplToMain<F>` and re-exports
  `CommitEarlyOutReason`. Deviations from table 1.1: `CommitEarlyOutReason`
  already lives in the shared `compositor_scheduler` crate, so it is
  re-exported instead of git-moved; `MutatorEvents` (and the other
  impl-built payloads: swap promises, LayerTreeMutator, ProxyImpl init
  values, presentation feedback, DrawImage) are named through the new
  `commit::proxy::ProxyTypes: LayerizeFamily` associated types, which the
  compositor implements in S1b-4; the `MutatorEvents` value moves with its
  main consumer in S1b-5. The metrics-only `BeginMainFrameReasons` is not
  translated.
- **S1b-4 part 1 done (payload types).** `ProxyTypes` for
  `CpuCompositorTypes<R>` (`runtime/owner_types.rs`): `MutatorEvents =
  Box<host_owners::MutatorEvents>` (`MutatorEventsBackend: Send` now),
  `ProxySwapPromise = Box<host_owners::SwapPromise<R>>`, `LayerTreeMutator
  = host_owners::LayerTreeMutator`; `ImplInit`, `PresentationFeedback` and
  `ProxyDrawImage` are `()` until S1b-7 posts them. `commit::proxy` gained
  `ProxyTaskQueue<T>` (the in-process posted-task queue of one direction),
  `ProtectedCommitLease::PaintServices` (FinishCommit's trace) and the
  Rechrom-only bridge task `MainToImpl::ReadyCommitAwaitsBeginMainFrame`
  (B1, deleted in S1b-7).
- **S1b-4 part 2 (ProxyMain/ProxyImpl wiring) not started.** Original notes: implement `ProxyTypes` for
  `CpuCompositorTypes<R>` (`MutatorEvents` = `host_owners::MutatorEvents`,
  whose `MutatorEventsBackend` still needs a `Send` bound;
  `ProxySwapPromise` = the boxed `SwapPromise`; `ImplInit` = the new
  ProxyImplInit). ProxyImpl's `DataForCommit` replaces EngineCore's
  `ready_commit: (CommitState, ProtectedCommitLease)` tuple and
  `ready_commit_notified`; its `ScopedCommitCompletionEvent` takes the
  completion with `ProtectedCommitLease::TakeCompletion`. Keep
  `impl_on_owner_sequence = true` until S1b-7: the pipelined BMF applies
  deltas to the main tree while a ready commit is outstanding.
