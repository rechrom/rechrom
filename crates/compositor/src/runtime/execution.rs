//! Private command transport and execution placement for the canonical owners.
//! A mailbox entry can name an owner but cannot recover or call one. Commands
//! are consumed by the Engine with its current resident owner and live context.
use crate::{
    adapter::{
        layer_tree_host_impl::HostImplTask,
        raster_task::TaskGraphTrace,
        resource_pool::WeakResourcePoolId,
        scheduler_host::SchedulerTask,
        task_runner::{OnceClosure, SequencedTaskRunner, TaskLocation},
        tile::TileManagerId,
        time::{TimeDelta, TimeTicks},
    },
    cc::{
        raster::{
            synchronous_task_graph_runner::SynchronousTaskGraphRunner,
            task::{TaskGraph, TaskRef, TaskVector},
            task_graph_runner::{NamespaceToken, TaskGraphRunner},
        },
        tiles::tile_manager::TileManagerCommand,
    },
    engine::CompositorExecutor,
};
use compositor_raster::worker_dispatch::{
    DispatchRasterWorkerLanes, RunRasterWorkerUntilIdle, ScheduleRasterTaskGraph,
};
use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, Weak,
    },
};

pub(crate) enum EngineTask {
    RenewTreePriority,
    PaintWorkletState {
        delegate: crate::adapter::layer_tree_host_impl::HostImplDelegateId,
        state: crate::cc::scheduler::scheduler_state_machine::PaintWorkletState,
    },

    AnimationWorkletState {
        delegate: crate::adapter::layer_tree_host_impl::HostImplDelegateId,
        state: crate::adapter::layer_tree_host_impl::AnimationWorkletMutationState,
        list: crate::adapter::layer_tree_impl::ElementListType,
    },

    RemoveLatencyMonitor {
        owner: crate::adapter::layer_tree_host_impl::HostImplWeak,
        id: crate::adapter::layer_tree_host_impl::HostImplLatencyMonitorId,
    },
    ScrollbarAnimation {
        delegate: crate::adapter::layer_tree_host_impl::HostImplDelegateId,
        task: crate::adapter::scrollbar_animation::ScrollbarAnimationTask,
    },
    DidNotProduceFrame {
        ack: crate::adapter::scheduler_host::BeginFrameAck,
        reason: crate::cc::scheduler::scheduler::FrameSkippedReason,
    },
    BeginFrameSourcePaused {
        paused: bool,
    },
    CheckerNeedsInvalidation {
        delegate: crate::adapter::layer_tree_host_impl::HostImplDelegateId,
    },
    Autoscroll {
        owner: crate::adapter::layer_tree_host_impl::HostImplWeak,
        task: crate::adapter::scrollbar::AutoscrollTask,
    },
    ImageAnimation(crate::adapter::image_animation::ImageAnimationTask),
    MainHost(super::main_endpoint::MainHostEffect),
    ImageDecoded {
        callback: crate::adapter::layer_tree_host_impl::HostImageDecodeCallback,
        succeeded: bool,
    },
    Host(HostImplTask),
    Metrics {
        target: crate::adapter::layer_tree_host_impl::HostImplWeak,
        results: super::host_metrics::CustomTrackerResults,
    },
    PostedHost(
        crate::adapter::layer_tree_host_impl::HostImplPostedCompositorTask<
            super::metadata::FrameTimingDetails,
        >,
    ),
    Scheduler(SchedulerTask),
    PoolEviction(WeakResourcePoolId),
    Tile {
        owner: TileManagerId,
        alive: Weak<()>,
        command: TileManagerCommand,
    },
    TileTasksFinished {
        owner: TileManagerId,
        alive: Weak<()>,
        start: TimeTicks,
        all_tiles_done: bool,
    },
    PostedClosure(OnceClosure),
    RasterWorkCompleted,
    RasterFailure(std::io::Error),
}

/// Independent transport state only. Mutex never protects a tree, tile manager
/// or callback execution. Taking a command releases the lock before dispatch.
#[derive(Clone, Default)]
pub(crate) struct EngineTaskQueue(Arc<Mutex<VecDeque<EngineTask>>>);
impl EngineTaskQueue {
    pub(crate) fn Push(&self, task: EngineTask) {
        self.0
            .lock()
            .expect("compositor command transport")
            .push_back(task);
    }
    pub(crate) fn Pop(&self) -> Option<EngineTask> {
        self.0
            .lock()
            .expect("compositor command transport")
            .pop_front()
    }
    pub(crate) fn IsEmpty(&self) -> bool {
        self.0
            .lock()
            .expect("compositor command transport")
            .is_empty()
    }
}

/// Posting always returns before the original callback can run, even when an
/// injected executor invokes its delivery closure immediately. Engine consumes
/// PostedClosure only after leaving the current source callback's borrow scope.
pub(crate) struct EngineSequencedTaskRunner {
    executor: Arc<dyn CompositorExecutor>,
    commands: EngineTaskQueue,
}
impl EngineSequencedTaskRunner {
    pub(crate) fn new(executor: Arc<dyn CompositorExecutor>, commands: EngineTaskQueue) -> Self {
        Self { executor, commands }
    }
}
impl SequencedTaskRunner for EngineSequencedTaskRunner {
    fn PostDelayedTask(&self, _location: TaskLocation, closure: OnceClosure, delay: TimeDelta) {
        let commands = self.commands.clone();
        self.executor.ExecuteAfter(
            delay,
            Box::new(move || {
                commands.Push(EngineTask::PostedClosure(closure));
            }),
        );
    }
    fn Now(&self) -> TimeTicks {
        self.executor.Now()
    }
    fn CalledOnValidThread(&self) -> bool {
        self.executor.RunsTasksInCurrentSequence()
    }
}

/// Placement adapter for the already translated queue/runner, not a worker
/// pool. The canonical runner alone selects tasks and changes task state.
/// ScheduleTasks is enqueue-only; Kick is called after the owner's source turn
/// ends. This preserves that contract even with a direct synchronous executor.
pub(crate) struct ExecutorTaskGraphRunner<C: 'static> {
    native: Arc<SynchronousTaskGraphRunner<C>>,
    executor: Arc<dyn CompositorExecutor>,
    commands: EngineTaskQueue,
    needs_drain: AtomicBool,
    worker_limit: usize,
}
impl<C: 'static> ExecutorTaskGraphRunner<C> {
    pub(crate) fn new(
        executor: Arc<dyn CompositorExecutor>,
        commands: EngineTaskQueue,
        trace: Arc<dyn TaskGraphTrace>,
        worker_limit: usize,
    ) -> Self {
        Self {
            native: Arc::new(SynchronousTaskGraphRunner::new(trace)),
            executor,
            commands,
            needs_drain: AtomicBool::new(false),
            worker_limit: worker_limit.max(1),
        }
    }
    pub(crate) fn Kick(&self) {
        if !self.needs_drain.swap(false, Ordering::AcqRel) {
            return;
        }
        let trace_context = browser_tracing::context();
        DispatchRasterWorkerLanes(self.worker_limit, |_| {
            let native = self.native.clone();
            let commands = self.commands.clone();
            self.executor.Execute(Box::new(move || {
                let _context = browser_tracing::scope(trace_context);
                // TaskGraphWorkQueue marks a task running before releasing its
                // queue mutex, so independent executor lanes can drain the
                // same graph just like Chromium's raster worker pool. Owner
                // callbacks still return through the typed command queue.
                RunRasterWorkerUntilIdle(|| native.RunUntilIdle());
                commands.Push(EngineTask::RasterWorkCompleted);
            }));
        });
    }
}
impl<C: 'static> TaskGraphRunner<C> for ExecutorTaskGraphRunner<C> {
    fn GenerateNamespaceToken(&self) -> NamespaceToken {
        self.native.GenerateNamespaceToken()
    }
    fn ScheduleTasks(&self, token: NamespaceToken, graph: &mut TaskGraph<C>) {
        // TaskGraphWorkQueue's mutex is the synchronization boundary between
        // the cc origin sequence and the injected raster executor. Holding a
        // second lock through RunOnWorkerThread would turn asynchronous raster
        // into a synchronous input-thread stall.
        ScheduleRasterTaskGraph(graph.nodes.len(), graph.edges.len(), || {
            self.native.ScheduleTasks(token, graph)
        });
        self.needs_drain.store(true, Ordering::Release);
    }
    fn ExternalDependencyCompletedForTask(&self, token: NamespaceToken, task: TaskRef<C>) {
        self.native.ExternalDependencyCompletedForTask(token, task);
        self.needs_drain.store(true, Ordering::Release);
    }
    fn WaitForTasksToFinishRunning(&self, token: NamespaceToken) {
        // Native shutdown explicitly permits waiting. The runner's queue lock
        // arbitrates with any already posted drain one task at a time.
        self.native.WaitForTasksToFinishRunning(token);
    }
    fn CollectCompletedTasks(&self, token: NamespaceToken, completed: &mut TaskVector<C>) {
        self.native.CollectCompletedTasks(token, completed);
    }
    fn RunTasksUntilIdleForTest(&self) {
        self.native.RunUntilIdle();
    }
}
