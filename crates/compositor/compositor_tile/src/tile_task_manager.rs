// Translation status: complete
// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Source: cc/tiles/tile_task_manager.h:5-16
// Source: cc/tiles/tile_task_manager.cc:5-13
use compositor_raster::{
    raster_task::{TileTaskManagerServices, TileTaskManagerTraceEvent},
    task::{TaskGraph, TaskVector},
    task_graph_runner::{NamespaceToken, TaskGraphRunner},
    tile_task::TileTaskRef,
};

// Source: cc/tiles/tile_task_manager.h:17-40
// Source: cc/tiles/tile_task_manager.cc:15-17
// The native default construction/virtual destruction map to a Rust trait.
// Trace/allowance macros receive a per-call live service; all scheduling and
// completion decisions remain in the canonical implementation below.
// Rust boundary: C explicitly carries the result of the source origin-thread
// completion callback through the existing Page command/effect contract. It
// does not alter worker execution or completion -> DidComplete -> dependency
// ordering. Tasks without a completion value use the default C = ().
pub trait TileTaskManager<C = ()> {
    // Source: cc/tiles/tile_task_manager.h:26-30
    fn ScheduleTasks(&self, graph: &mut TaskGraph<C>, services: &dyn TileTaskManagerServices);

    // Source: cc/tiles/tile_task_manager.h:32-33
    fn ExternalDependencyCompletedForTask(&self, task: TileTaskRef<C>);

    // Source: cc/tiles/tile_task_manager.h:35-36
    fn CheckForCompletedTasks(
        &self,
        services: &dyn TileTaskManagerServices,
        consumer: &mut dyn TileTaskCompletionConsumer<C>,
    );

    // Source: cc/tiles/tile_task_manager.h:38-39
    fn Shutdown(&self, services: &dyn TileTaskManagerServices);
}

// Source: cc/tiles/tile_task_manager.h:49-52,61-68
// The source bound owner callback becomes an explicit call-scoped consumer.
// Neither worker nor manager retains a callable owner or captures a Page.
// The original dependent owning reference is transferred at callback.Run.
pub trait TileTaskCompletionConsumer<C> {
    fn OnRasterCompletion(&mut self, completion: C);
    fn ExternalDependencyCompleted(&mut self, dependent: TileTaskRef<C>);
}

// Source: cc/tiles/tile_task_manager.h:42-70
// Source: cc/tiles/tile_task_manager.cc:36
// The independent runner is an Arc-owned capability lease corresponding to
// source raw_ptr<TaskGraphRunner> plus the embedding's longer-lived runner.
// This avoids a fabricated static borrow: it owns no manager/host/Page, and
// cloning the capability does not clone runner state or create a thread.
// Reverse member declaration order preserves native member destruction.
// No Clone is implemented: source copy construction/assignment are deleted.
pub struct TileTaskManagerImpl<C = ()> {
    namespace_token_: NamespaceToken,
    task_graph_runner_: std::sync::Arc<dyn TaskGraphRunner<C>>,
}

impl<C> TileTaskManagerImpl<C> {
    // Source: cc/tiles/tile_task_manager.h:49-52
    // Source: cc/tiles/tile_task_manager.cc:19-26
    pub fn Create(task_graph_runner: std::sync::Arc<dyn TaskGraphRunner<C>>) -> Box<Self> {
        Box::new(Self::new(task_graph_runner))
    }

    // Source: cc/tiles/tile_task_manager.h:61-69
    // Source: cc/tiles/tile_task_manager.cc:28-34
    fn new(task_graph_runner: std::sync::Arc<dyn TaskGraphRunner<C>>) -> Self {
        Self {
            namespace_token_: task_graph_runner.GenerateNamespaceToken(),
            task_graph_runner_: task_graph_runner,
        }
    }
}

impl<C> TileTaskManager<C> for TileTaskManagerImpl<C> {
    // Source: cc/tiles/tile_task_manager.h:54-55
    // Source: cc/tiles/tile_task_manager.cc:38-41
    fn ScheduleTasks(&self, graph: &mut TaskGraph<C>, services: &dyn TileTaskManagerServices) {
        let _trace = services.TraceTileTaskManager(TileTaskManagerTraceEvent::ScheduleTasks);
        self.task_graph_runner_
            .ScheduleTasks(self.namespace_token_, graph);
    }

    // Source: cc/tiles/tile_task_manager.h:56
    // Source: cc/tiles/tile_task_manager.cc:43-47
    fn ExternalDependencyCompletedForTask(&self, task: TileTaskRef<C>) {
        self.task_graph_runner_
            .ExternalDependencyCompletedForTask(self.namespace_token_, task);
    }

    // Source: cc/tiles/tile_task_manager.h:57
    // Source: cc/tiles/tile_task_manager.cc:49-67
    fn CheckForCompletedTasks(
        &self,
        services: &dyn TileTaskManagerServices,
        consumer: &mut dyn TileTaskCompletionConsumer<C>,
    ) {
        let _trace =
            services.TraceTileTaskManager(TileTaskManagerTraceEvent::CheckForCompletedTasks);
        let mut completed_tasks = TaskVector::new();
        self.task_graph_runner_
            .CollectCompletedTasks(self.namespace_token_, &mut completed_tasks);

        for task in &completed_tasks {
            debug_assert!(task
                .task()
                .WithState(|state| state.IsFinished() || state.IsCanceled()));
            // Source: cc/tiles/tile_task_manager.cc:56
            // Safe Rust mapping of the native static_cast<TileTask*>: this
            // namespace's TaskRef owner must actually be TileTask-derived.
            // The original TaskRef remains alive for the entire projection.
            let tile_task = task
                .AsTileTask()
                .expect("TileTaskManager namespace requires TileTask-derived tasks");
            // Source cc57: consume the typed return at this exact native
            // synchronous callback point, before DidComplete/dependencies.
            if let Some(completion) = tile_task.OnTaskCompleted() {
                consumer.OnRasterCompletion(completion);
            }
            tile_task.tile_task().DidComplete();
            // Source: cc/tiles/tile_task_manager.cc:59-61
            // Native TODO: notify directly when the dependency runs rather
            // than waiting for collection; doing so requires a shared lock.
            // Preserve current collection-time notification, not that TODO.
            let dependent = tile_task.tile_task().WithExternalDependent(|dependent| {
                if let Some(dependent) = dependent.as_ref() {
                    dependent.tile_task().ExternalDependencyCompleted();
                }
                dependent.take()
            });
            // The access-cell borrow ends before invoking external code. The
            // original strong reference has already moved out of the resident
            // slot, exactly as std::move(dependent) does before native Run().
            if let Some(dependent) = dependent {
                consumer.ExternalDependencyCompleted(dependent);
            }
        }
    }

    // Source: cc/tiles/tile_task_manager.h:58
    // Source: cc/tiles/tile_task_manager.cc:69-79
    fn Shutdown(&self, services: &dyn TileTaskManagerServices) {
        let _trace = services.TraceTileTaskManager(TileTaskManagerTraceEvent::Shutdown);
        // Cancel non-scheduled tasks, then wait for running tasks to finish.
        let mut empty = TaskGraph::new();
        self.task_graph_runner_
            .ScheduleTasks(self.namespace_token_, &mut empty);
        {
            let _allow_wait = services.AllowSyncPrimitivesOutsideBlockingScope();
            self.task_graph_runner_
                .WaitForTasksToFinishRunning(self.namespace_token_);
        }
    }
}

// Source: cc/tiles/tile_task_manager.h:72-74
// Source: cc/tiles/tile_task_manager.cc:81
// Namespace/include guards and default virtual destruction map to Rust.
