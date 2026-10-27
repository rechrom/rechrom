//! Platform-neutral trace identity allocation for the canonical raster Task.
//! The embedding supplies the existing process-wide trace-id source. This
//! interface creates no thread, worker, scheduler, or thread pool.
pub trait TraceTaskIdProvider {
    fn GetNextGlobalTraceId(&self) -> u64;
}

/// Live feature value consumed by cc/raster/tile_task.cc:51-62. The core owns
/// all dependency mutation; the embedding only supplies the feature query.
pub trait TileTaskFeatures {
    fn PreventDuplicateImageDecodesEnabled(&self) -> bool;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileTaskManagerTraceEvent {
    ScheduleTasks,
    CheckForCompletedTasks,
    Shutdown,
}

/// External RAII tracing scope; the actual trace implementation lives outside
/// compositor and its concrete Drop closes the trace scope.
pub trait TileTaskManagerTraceScope {}

/// External RAII allowance for the source's explicitly permitted blocking
/// primitive use. This capability does not implement a wait or a thread pool.
pub trait SyncPrimitivesAllowance {}

pub trait TileTaskManagerServices {
    fn TraceTileTaskManager(
        &self,
        event: TileTaskManagerTraceEvent,
    ) -> Box<dyn TileTaskManagerTraceScope + '_>;
    fn AllowSyncPrimitivesOutsideBlockingScope(&self) -> Box<dyn SyncPrimitivesAllowance + '_>;
}

/// The process tracing side effects used by the native task-graph queue and
/// synchronous runner. This capability must not retain a task, graph, Page or
/// compositor owner. The returned guards only delimit the corresponding trace.
pub trait TaskGraphTrace: Send + Sync {
    fn TraceWorkQueueSchedule(&self) -> Box<dyn TaskGraphScheduleTrace + '_>;
    fn TraceSynchronousRunner(
        &self,
        event: SynchronousRunnerTraceEvent,
    ) -> Box<dyn TaskGraphTraceScope + '_>;
    fn HeapProfilerTaskExecution(&self) -> Box<dyn TaskGraphTraceScope + '_>;
    fn TerminateSynchronousTaskFlow(&self, trace_task_id: u64);
}

pub trait TaskGraphTraceScope {}
pub trait TaskGraphScheduleTrace {
    fn AddFlowId(&mut self, trace_task_id: u64);
}
#[derive(Clone, Copy, Debug)]
pub enum SynchronousRunnerTraceEvent {
    ScheduleTasks { num_nodes: usize, num_edges: usize },
    WaitForTasksToFinishRunning,
    CollectCompletedTasks,
    RunTask,
}
