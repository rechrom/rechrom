// Translation status: complete
// Copyright 2015 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Source: cc/raster/synchronous_task_graph_runner.h:5-15
// Source: cc/raster/synchronous_task_graph_runner.cc:5-16
use super::task::{TaskGraph, TaskRef, TaskVector};
use super::task_graph_runner::{NamespaceToken, TaskGraphRunner};
use super::task_graph_work_queue::TaskGraphWorkQueue;
use crate::adapter::raster_task::{SynchronousRunnerTraceEvent, TaskGraphTrace};
use std::sync::{Arc, Mutex};

// Source: cc/raster/synchronous_task_graph_runner.h:16-46
// TaskGraphRunner is Send+Sync. The lock protects only its explicitly
// externally-synchronized native work_queue_, never a tree or the task body.
// The trace capability substitutes only the native global tracing sink; it
// cannot retain a task/graph/compositor owner. No thread or clock is created.
pub struct SynchronousTaskGraphRunner<C = ()> {
    work_queue_: Mutex<TaskGraphWorkQueue<C>>,
    trace_: Arc<dyn TaskGraphTrace>,
}
impl<C> SynchronousTaskGraphRunner<C> {
    // Source: cc/raster/synchronous_task_graph_runner.cc:18
    pub fn new(trace: Arc<dyn TaskGraphTrace>) -> Self {
        Self {
            work_queue_: Mutex::new(TaskGraphWorkQueue::new()),
            trace_: trace,
        }
    }
    // Source: cc/raster/synchronous_task_graph_runner.cc:24-26
    pub fn GenerateNamespaceToken(&self) -> NamespaceToken {
        self.work_queue_.lock().unwrap().GenerateNamespaceToken()
    }
    // Source: cc/raster/synchronous_task_graph_runner.cc:28-37
    pub fn ScheduleTasks(&self, token: NamespaceToken, graph: &mut TaskGraph<C>) {
        let _event =
            self.trace_
                .TraceSynchronousRunner(SynchronousRunnerTraceEvent::ScheduleTasks {
                    num_nodes: graph.nodes.len(),
                    num_edges: graph.edges.len(),
                });
        debug_assert!(token.IsValid());
        debug_assert!(!TaskGraphWorkQueue::DependencyMismatch(graph));
        self.work_queue_
            .lock()
            .unwrap()
            .ScheduleTasks(token, graph, self.trace_.as_ref());
    }
    // Source: cc/raster/synchronous_task_graph_runner.cc:39-43
    pub fn ExternalDependencyCompletedForTask(&self, token: NamespaceToken, task: TaskRef<C>) {
        self.work_queue_
            .lock()
            .unwrap()
            .ExternalDependencyCompletedForTask(token, task);
    }
    // Source: cc/raster/synchronous_task_graph_runner.cc:45-59
    pub fn WaitForTasksToFinishRunning(&self, token: NamespaceToken) {
        let _event = self
            .trace_
            .TraceSynchronousRunner(SynchronousRunnerTraceEvent::WaitForTasksToFinishRunning);
        debug_assert!(token.IsValid());
        if self
            .work_queue_
            .lock()
            .unwrap()
            .GetNamespaceForToken(token)
            .is_none()
        {
            return;
        }
        loop {
            // Re-resolve the same resident namespace identity after each
            // synchronous task. A raw pointer across queue mutation is not held.
            let finished = {
                let queue = self.work_queue_.lock().unwrap();
                let namespace = queue
                    .GetNamespaceForToken(token)
                    .expect("namespace lives until collection");
                TaskGraphWorkQueue::HasFinishedRunningTasksInNamespace(namespace)
            };
            if finished {
                break;
            }
            let succeeded = self.RunTask();
            debug_assert!(succeeded);
        }
    }
    // Source: cc/raster/synchronous_task_graph_runner.cc:61-68
    pub fn CollectCompletedTasks(
        &self,
        token: NamespaceToken,
        completed_tasks: &mut TaskVector<C>,
    ) {
        let _event = self
            .trace_
            .TraceSynchronousRunner(SynchronousRunnerTraceEvent::CollectCompletedTasks);
        debug_assert!(token.IsValid());
        self.work_queue_
            .lock()
            .unwrap()
            .CollectCompletedTasks(token, completed_tasks);
    }
    // Source: cc/raster/synchronous_task_graph_runner.cc:70-73
    pub fn RunUntilIdle(&self) {
        while self.RunTask() {}
    }
    // Source: cc/raster/synchronous_task_graph_runner.cc:75-77
    pub fn RunTasksUntilIdleForTest(&self) {
        self.RunUntilIdle();
    }
    // Source: cc/raster/synchronous_task_graph_runner.h:34-35
    pub fn RunSingleTaskForTesting(&self) -> bool {
        self.RunTask()
    }
    // Source: cc/raster/synchronous_task_graph_runner.cc:79-109
    fn RunTask(&self) -> bool {
        let _heap_event = self.trace_.HeapProfilerTaskExecution();
        let _event = self
            .trace_
            .TraceSynchronousRunner(SynchronousRunnerTraceEvent::RunTask);
        let prioritized_task = {
            let mut queue = self.work_queue_.lock().unwrap();
            let found = queue
                .ready_to_run_namespaces()
                .iter()
                .find(|(_, tasks)| !tasks.is_empty());
            let Some((&category, _)) = found else {
                return false;
            };
            queue.GetNextTaskToRun(category)
        };
        prioritized_task.task.RunOnWorkerThread();
        self.trace_
            .TerminateSynchronousTaskFlow(prioritized_task.task.task().trace_task_id());
        self.work_queue_
            .lock()
            .unwrap()
            .CompleteTask(prioritized_task);
        true
    }
}
// Source: cc/raster/synchronous_task_graph_runner.cc:20-22
impl<C> Drop for SynchronousTaskGraphRunner<C> {
    fn drop(&mut self) {
        debug_assert!(!self.work_queue_.get_mut().unwrap().HasReadyToRunTasks());
    }
}
// Source: cc/raster/synchronous_task_graph_runner.h:20-28
// The virtual interface forwards to the exact same native bodies above.
impl<C> TaskGraphRunner<C> for SynchronousTaskGraphRunner<C> {
    fn GenerateNamespaceToken(&self) -> NamespaceToken {
        Self::GenerateNamespaceToken(self)
    }
    fn ScheduleTasks(&self, token: NamespaceToken, graph: &mut TaskGraph<C>) {
        Self::ScheduleTasks(self, token, graph)
    }
    fn ExternalDependencyCompletedForTask(&self, token: NamespaceToken, task: TaskRef<C>) {
        Self::ExternalDependencyCompletedForTask(self, token, task)
    }
    fn WaitForTasksToFinishRunning(&self, token: NamespaceToken) {
        Self::WaitForTasksToFinishRunning(self, token)
    }
    fn CollectCompletedTasks(&self, token: NamespaceToken, tasks: &mut TaskVector<C>) {
        Self::CollectCompletedTasks(self, token, tasks)
    }
    fn RunTasksUntilIdleForTest(&self) {
        Self::RunTasksUntilIdleForTest(self)
    }
}
