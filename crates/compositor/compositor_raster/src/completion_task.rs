//! Raster completion tasks and their injected worker/origin capabilities.

use crate::{
    raster_task::TraceTaskIdProvider,
    task::{Task, TaskImpl},
    tile_task::{
        SupportsBackgroundThreadPriority, SupportsConcurrentExecution, TileTask, TileTaskImpl,
    },
};
use std::sync::{Arc, Mutex};

pub type OnceClosure = Box<dyn FnOnce() + Send + 'static>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RasterTaskLocation {
    pub function: &'static str,
    pub file: &'static str,
    pub line: u32,
}

pub trait RasterOriginTaskRunner: Send + Sync {
    fn PostRasterTask(&self, location: RasterTaskLocation, closure: OnceClosure);
}

pub trait RasterWorkerTaskTraceScope {}

pub trait RasterWorkerTaskDiagnostics: TraceTaskIdProvider + Send + Sync {
    fn TraceWorkerTask(&self, event: &'static str) -> Box<dyn RasterWorkerTaskTraceScope>;
}

pub trait RasterQueryQueue: Send + Sync {
    fn CheckRasterFinishedQueries(&self) -> bool;
}

pub trait RasterCommandFlush: Send + Sync {
    fn FlushTileRasterGraphiteCommands(&self);
}

// Source: cc/tiles/tile_manager.cc:264-298.
pub struct TaskSetFinishedTask<C, R: ?Sized, D: ?Sized> {
    on_task_set_finished_callback_: Mutex<Option<OnceClosure>>,
    task_runner_: Arc<R>,
    base_: TileTask<C>,
    diagnostics_: Arc<D>,
}

impl<C, R, D> TaskSetFinishedTask<C, R, D>
where
    R: RasterOriginTaskRunner + ?Sized,
    D: RasterWorkerTaskDiagnostics + ?Sized,
{
    pub fn new(
        task_runner: Arc<R>,
        on_task_set_finished_callback: OnceClosure,
        diagnostics: Arc<D>,
    ) -> Self {
        let base = TileTask::new(
            diagnostics.as_ref(),
            SupportsConcurrentExecution::kYes,
            SupportsBackgroundThreadPriority::kYes,
            None,
        );
        Self {
            base_: base,
            task_runner_: task_runner,
            on_task_set_finished_callback_: Mutex::new(Some(on_task_set_finished_callback)),
            diagnostics_: diagnostics,
        }
    }

    fn TaskSetFinished(&self) {
        let callback = self
            .on_task_set_finished_callback_
            .lock()
            .unwrap()
            .take()
            .expect("source CHECK(on_task_set_finished_callback_)");
        self.task_runner_.PostRasterTask(
            RasterTaskLocation {
                function: "TaskSetFinishedTaskImpl::TaskSetFinished",
                file: "cc/tiles/tile_manager.cc",
                line: 290,
            },
            callback,
        );
    }
}

impl<C, R, D> TaskImpl<C> for TaskSetFinishedTask<C, R, D>
where
    R: RasterOriginTaskRunner + ?Sized,
    D: RasterWorkerTaskDiagnostics + ?Sized,
{
    fn task(&self) -> &Task {
        self.base_.task()
    }

    fn RunOnWorkerThread(&self) {
        let _trace = self
            .diagnostics_
            .TraceWorkerTask("TaskSetFinishedTaskImpl::RunOnWorkerThread");
        self.TaskSetFinished();
    }

    fn AsTileTask(&self) -> Option<&dyn TileTaskImpl<C>> {
        Some(self)
    }
}

impl<C, R, D> TileTaskImpl<C> for TaskSetFinishedTask<C, R, D>
where
    R: RasterOriginTaskRunner + ?Sized,
    D: RasterWorkerTaskDiagnostics + ?Sized,
{
    fn tile_task(&self) -> &TileTask<C> {
        &self.base_
    }

    fn OnTaskCompleted(&self) -> Option<C> {
        None
    }
}

// Source: cc/tiles/tile_manager.cc:300-343.
pub struct DidFinishRunningAllTilesTask<C, R: ?Sized, D: ?Sized> {
    completion_cb_: Mutex<Option<Box<dyn FnOnce(bool) + Send>>>,
    raster_buffer_provider_for_flush_: Option<Arc<dyn RasterCommandFlush>>,
    pending_raster_queries_: Option<Arc<dyn RasterQueryQueue>>,
    task_runner_: Arc<R>,
    base_: TileTask<C>,
    diagnostics_: Arc<D>,
}

impl<C, R, D> DidFinishRunningAllTilesTask<C, R, D>
where
    R: RasterOriginTaskRunner + ?Sized,
    D: RasterWorkerTaskDiagnostics + ?Sized,
{
    pub fn new(
        task_runner: Arc<R>,
        pending_raster_queries: Option<Arc<dyn RasterQueryQueue>>,
        raster_buffer_provider: Option<Arc<dyn RasterCommandFlush>>,
        num_tile_tasks: usize,
        completion_cb: Box<dyn FnOnce(bool) + Send>,
        diagnostics: Arc<D>,
    ) -> Self {
        let base = TileTask::new(
            diagnostics.as_ref(),
            SupportsConcurrentExecution::kNo,
            SupportsBackgroundThreadPriority::kYes,
            None,
        );
        Self {
            base_: base,
            task_runner_: task_runner,
            pending_raster_queries_: pending_raster_queries,
            raster_buffer_provider_for_flush_: if num_tile_tasks > 0 {
                raster_buffer_provider
            } else {
                None
            },
            completion_cb_: Mutex::new(Some(completion_cb)),
            diagnostics_: diagnostics,
        }
    }
}

impl<C, R, D> TaskImpl<C> for DidFinishRunningAllTilesTask<C, R, D>
where
    R: RasterOriginTaskRunner + ?Sized,
    D: RasterWorkerTaskDiagnostics + ?Sized,
{
    fn task(&self) -> &Task {
        self.base_.task()
    }

    fn RunOnWorkerThread(&self) {
        let _trace = self
            .diagnostics_
            .TraceWorkerTask("DidFinishRunningAllTilesTask::RunOnWorkerThread");
        if let Some(provider) = self.raster_buffer_provider_for_flush_.as_ref() {
            provider.FlushTileRasterGraphiteCommands();
        }
        let has_pending_queries = self
            .pending_raster_queries_
            .as_ref()
            .is_some_and(|queries| queries.CheckRasterFinishedQueries());
        let callback = self
            .completion_cb_
            .lock()
            .unwrap()
            .take()
            .expect("source once completion_cb_");
        self.task_runner_.PostRasterTask(
            RasterTaskLocation {
                function: "DidFinishRunningAllTilesTask::RunOnWorkerThread",
                file: "cc/tiles/tile_manager.cc",
                line: 327,
            },
            Box::new(move || callback(has_pending_queries)),
        );
    }

    fn AsTileTask(&self) -> Option<&dyn TileTaskImpl<C>> {
        Some(self)
    }
}

impl<C, R, D> TileTaskImpl<C> for DidFinishRunningAllTilesTask<C, R, D>
where
    R: RasterOriginTaskRunner + ?Sized,
    D: RasterWorkerTaskDiagnostics + ?Sized,
{
    fn tile_task(&self) -> &TileTask<C> {
        &self.base_
    }

    fn OnTaskCompleted(&self) -> Option<C> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

    #[derive(Default)]
    struct TestRunner {
        tasks: Mutex<Vec<OnceClosure>>,
    }
    impl RasterOriginTaskRunner for TestRunner {
        fn PostRasterTask(&self, _: RasterTaskLocation, closure: OnceClosure) {
            self.tasks.lock().unwrap().push(closure);
        }
    }

    struct TestScope;
    impl RasterWorkerTaskTraceScope for TestScope {}

    #[derive(Default)]
    struct TestDiagnostics {
        next_id: AtomicU64,
    }
    impl TraceTaskIdProvider for TestDiagnostics {
        fn GetNextGlobalTraceId(&self) -> u64 {
            self.next_id.fetch_add(1, Ordering::Relaxed) + 1
        }
    }
    impl RasterWorkerTaskDiagnostics for TestDiagnostics {
        fn TraceWorkerTask(&self, _: &'static str) -> Box<dyn RasterWorkerTaskTraceScope> {
            Box::new(TestScope)
        }
    }

    struct TestQueryQueue;
    impl RasterQueryQueue for TestQueryQueue {
        fn CheckRasterFinishedQueries(&self) -> bool {
            true
        }
    }

    #[derive(Default)]
    struct TestFlush {
        calls: AtomicUsize,
    }
    impl RasterCommandFlush for TestFlush {
        fn FlushTileRasterGraphiteCommands(&self) {
            self.calls.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn finish_task(task: &dyn TileTaskImpl<()>) {
        task.task().WithState(|state| state.DidFinish());
        task.tile_task().DidComplete();
    }

    #[test]
    fn all_tiles_completion_flushes_checks_queries_then_posts_result() {
        let runner = Arc::new(TestRunner::default());
        let diagnostics = Arc::new(TestDiagnostics::default());
        let flush = Arc::new(TestFlush::default());
        let result = Arc::new(Mutex::new(None));
        let result_for_callback = Arc::clone(&result);
        let task = DidFinishRunningAllTilesTask::<(), _, _>::new(
            Arc::clone(&runner),
            Some(Arc::new(TestQueryQueue)),
            Some(flush.clone()),
            1,
            Box::new(move |pending| *result_for_callback.lock().unwrap() = Some(pending)),
            diagnostics,
        );
        task.task().WithState(|state| {
            state.DidSchedule();
            state.DidStart();
        });

        task.RunOnWorkerThread();

        assert_eq!(flush.calls.load(Ordering::Relaxed), 1);
        assert_eq!(*result.lock().unwrap(), None);
        runner.tasks.lock().unwrap().pop().unwrap()();
        assert_eq!(*result.lock().unwrap(), Some(true));
        finish_task(&task);
    }

    #[test]
    fn all_tiles_completion_does_not_flush_without_tile_tasks() {
        let runner = Arc::new(TestRunner::default());
        let diagnostics = Arc::new(TestDiagnostics::default());
        let flush = Arc::new(TestFlush::default());
        let task = DidFinishRunningAllTilesTask::<(), _, _>::new(
            Arc::clone(&runner),
            None,
            Some(flush.clone()),
            0,
            Box::new(|_| {}),
            diagnostics,
        );
        task.task().WithState(|state| {
            state.DidSchedule();
            state.DidStart();
        });

        task.RunOnWorkerThread();

        assert_eq!(flush.calls.load(Ordering::Relaxed), 0);
        runner.tasks.lock().unwrap().pop().unwrap()();
        finish_task(&task);
    }
}
