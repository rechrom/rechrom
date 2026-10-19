// Translation status: complete
// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Source: cc/raster/task_graph_runner.h:5-15
use super::task::{TaskGraph, TaskRef, TaskVector};

// Source: cc/raster/task_graph_runner.h:16-30
// Native implicit copy construction preserves this opaque identity. Only the
// canonical TaskGraphWorkQueue can generate a nonzero identity; the public
// default constructor returns the native invalid token.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
pub struct NamespaceToken {
    id_: i32,
}

impl NamespaceToken {
    // Source: cc/raster/task_graph_runner.h:19-20
    pub fn new() -> Self {
        Self { id_: 0 }
    }

    // Source: cc/raster/task_graph_runner.h:22
    pub fn IsValid(&self) -> bool {
        self.id_ != 0
    }

    // Source: cc/raster/task_graph_runner.h:24-29
    // Raster-module visibility represents the TaskGraphWorkQueue friend. This
    // does not allocate a token or provide a second namespace-id generator.
    pub(super) fn from_id(id: i32) -> Self {
        Self { id_: id }
    }

    pub(super) fn id(&self) -> i32 {
        self.id_
    }
}

// Source: cc/raster/task_graph_runner.h:32-65
// Implementations must be safe to call from any thread. ScheduleTasks never
// executes a task synchronously. Tasks run in dependency order, and every
// scheduled task must eventually be returned by CollectCompletedTasks, even
// when a later ScheduleTasks cancels it. Priority is only a hint; this interface
// provides no synchronization for data shared by task implementations.
// This is a thread-independent interface, not a pool or wait implementation.
// Rust boundary: C explicitly carries the result of the source origin-thread
// completion callback through the existing Page command/effect contract. It
// does not alter worker execution or completion -> DidComplete -> dependency
// ordering. Tasks without a completion value use the default C = ().
pub trait TaskGraphRunner<C = ()>: Send + Sync {
    // Source: cc/raster/task_graph_runner.h:67-69
    /// Return a unique, nonzero namespace token.
    fn GenerateNamespaceToken(&self) -> NamespaceToken;

    // Source: cc/raster/task_graph_runner.h:71-76
    /// Schedule the graph, canceling no-longer-needed tasks unless running.
    /// Cancellation still enqueues the task for completion collection.
    /// The graph object remains caller-owned; implementations may mutate or
    /// swap its contents, matching the native TaskGraph* parameter.
    fn ScheduleTasks(&self, token: NamespaceToken, graph: &mut TaskGraph<C>);

    // Source: cc/raster/task_graph_runner.h:78-80
    /// Signal completion of the task's external dependency. The parameter is
    /// an owning task reference, matching the native scoped_refptr value.
    fn ExternalDependencyCompletedForTask(&self, token: NamespaceToken, task: TaskRef<C>);

    // Source: cc/raster/task_graph_runner.h:82-83
    /// Wait for every scheduled task to finish running.
    fn WaitForTasksToFinishRunning(&self, token: NamespaceToken);

    // Source: cc/raster/task_graph_runner.h:85-87
    /// Collect completed owning task references into the caller's vector.
    fn CollectCompletedTasks(&self, token: NamespaceToken, completed_tasks: &mut TaskVector<C>);

    // Source: cc/raster/task_graph_runner.h:89
    fn RunTasksUntilIdleForTest(&self);
}

// Source: cc/raster/task_graph_runner.h:90-94
// Trait-object destruction and the Rust module boundary replace the virtual
// destructor, namespace closure and include guard; no wait mechanism is added.
