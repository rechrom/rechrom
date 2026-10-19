// Translation status: complete
// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Source: cc/raster/tile_task.h:5-13
// Source: cc/raster/tile_task.cc:5-14
use super::task::{Task, TaskImpl};
use crate::adapter::raster_task::{TileTaskFeatures, TraceTaskIdProvider};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

// Source: cc/raster/tile_task.h:57-59
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SupportsConcurrentExecution {
    kYes,
    kNo,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SupportsBackgroundThreadPriority {
    kYes,
    kNo,
}

// Source: cc/raster/tile_task.h:14-16,44-52
// A derived tile task owns one native TileTask base, which owns one Task base.
// The two virtual predicates retain their native default bodies, while worker
// execution and origin-thread completion remain distinct typed callbacks.
// Rust boundary: C explicitly carries the result of the source origin-thread
// completion callback through the existing Page command/effect contract. It
// does not alter worker execution or completion -> DidComplete -> dependency
// ordering. Tasks without a completion value use the default C = ().
pub trait TileTaskImpl<C = ()>: TaskImpl<C> {
    fn tile_task(&self) -> &TileTask<C>;

    // Source: cc/raster/tile_task.h:44
    // Source: cc/raster/tile_task.cc:37-39
    fn IsRasterTask(&self) -> bool {
        self.tile_task().IsRasterTask()
    }

    // Source: cc/raster/tile_task.h:46-48
    /// Called by the owner on the origin thread to process task completion.
    fn OnTaskCompleted(&self) -> Option<C>;

    // Source: cc/raster/tile_task.h:50-52
    // Source: cc/raster/tile_task.cc:41-49
    fn TaskContainsLCPCandidateImages(&self) -> bool {
        self.tile_task().TaskContainsLCPCandidateImages()
    }
}

pub type TileTaskRef<C = ()> = Arc<dyn TileTaskImpl<C>>;
pub type TileTaskVector<C = ()> = Vec<TileTaskRef<C>>;

// Source: cc/raster/tile_task.h:61-72
// Reverse owning-member order preserves C++ destruction: external dependent,
// dependency vector, then Task base. Local access cells only make these source
// fields safely borrowable through their refcounted owner, without exposing
// locks to scheduling algorithms. Neither cell contains scheduling policy.
pub struct TileTask<C = ()> {
    external_dependent_: Mutex<Option<TileTaskRef<C>>>,
    dependencies_: Mutex<TileTaskVector<C>>,
    supports_concurrent_execution_: SupportsConcurrentExecution,
    supports_background_thread_priority_: SupportsBackgroundThreadPriority,
    did_complete_: AtomicBool,
    task_: Task,
}

impl<C> TileTask<C> {
    // Source: cc/raster/tile_task.h:61-68,71
    // Source: cc/raster/tile_task.cc:15-22
    pub fn new(
        trace_ids: &dyn TraceTaskIdProvider,
        supports_concurrent_execution: SupportsConcurrentExecution,
        supports_background_thread_priority: SupportsBackgroundThreadPriority,
        dependencies: Option<&mut TileTaskVector<C>>,
    ) -> Self {
        // Base construction precedes native member initialization.
        let task = Task::new(trace_ids);
        let dependencies = match dependencies {
            Some(dependencies) => std::mem::take(dependencies),
            None => TileTaskVector::new(),
        };
        Self {
            external_dependent_: Mutex::new(None),
            dependencies_: Mutex::new(dependencies),
            supports_concurrent_execution_: supports_concurrent_execution,
            supports_background_thread_priority_: supports_background_thread_priority,
            did_complete_: AtomicBool::new(false),
            task_: task,
        }
    }

    // Source: cc/raster/tile_task.h:14
    // Rust base-class projection, retaining the same resident Task.
    pub fn task(&self) -> &Task {
        &self.task_
    }

    // Source: cc/raster/tile_task.h:18
    // Scoped readonly borrow of the resident vector, not a dependency snapshot.
    pub fn WithDependencies<R>(&self, callback: impl FnOnce(&TileTaskVector<C>) -> R) -> R {
        let dependencies = self.dependencies_.lock().unwrap();
        callback(&dependencies)
    }

    // Source: cc/raster/tile_task.h:23
    // Scoped mutable borrow replaces scoped_refptr<TileTask>&. The native
    // completion owner can release this strong reference at the same point.
    pub fn WithExternalDependent<R>(
        &self,
        callback: impl FnOnce(&mut Option<TileTaskRef<C>>) -> R,
    ) -> R {
        let mut dependent = self.external_dependent_.lock().unwrap();
        callback(&mut dependent)
    }

    // Source: cc/raster/tile_task.h:30-35
    pub fn supports_concurrent_execution(&self) -> bool {
        self.supports_concurrent_execution_ == SupportsConcurrentExecution::kYes
    }

    // Source: cc/raster/tile_task.h:37-42
    pub fn supports_background_thread_priority(&self) -> bool {
        self.supports_background_thread_priority_ == SupportsBackgroundThreadPriority::kYes
    }

    // Source: cc/raster/tile_task.h:44
    // Source: cc/raster/tile_task.cc:37-39
    pub fn IsRasterTask(&self) -> bool {
        true
    }

    // Source: cc/raster/tile_task.h:50-52
    // Source: cc/raster/tile_task.cc:41-49
    pub fn TaskContainsLCPCandidateImages(&self) -> bool {
        let dependencies = self.dependencies_.lock().unwrap();
        for dependent in dependencies.iter().cloned() {
            if !dependent.tile_task().HasCompleted() && dependent.TaskContainsLCPCandidateImages() {
                return true;
            }
        }
        false
    }

    // Source: cc/raster/tile_task.h:54
    // Source: cc/raster/tile_task.cc:28-31
    pub fn DidComplete(&self) {
        debug_assert!(!self.did_complete_.load(Ordering::Relaxed));
        self.did_complete_.store(true, Ordering::Relaxed);
    }

    // Source: cc/raster/tile_task.h:55
    // Source: cc/raster/tile_task.cc:33-35
    pub fn HasCompleted(&self) -> bool {
        self.did_complete_.load(Ordering::Relaxed)
    }

    // Source: cc/raster/tile_task.h:20-22
    // Source: cc/raster/tile_task.cc:51-62
    // `this` is the source's refcounted derived owner, not a second Task state.
    // Keeping it explicit preserves virtual IsRasterTask dispatch and the
    // native push_back(this) ownership increment. The temporary strong cycle
    // is intentional: ExternalDependencyCompleted drops the dependency side;
    // the completion owner releases external_dependent through its accessor.
    pub fn SetExternalDependent(
        this: &TileTaskRef,
        dependent: TileTaskRef,
        features: &dyn TileTaskFeatures,
    ) {
        if features.PreventDuplicateImageDecodesEnabled() {
            assert!(this.IsRasterTask() != dependent.IsRasterTask());
            {
                let external_dependent = this.tile_task().external_dependent_.lock().unwrap();
                assert!(external_dependent.as_ref().is_none_or(|external| {
                    external.task().WithState(|state| state.IsCanceled())
                }));
            }
            {
                let mut dependencies = dependent.tile_task().dependencies_.lock().unwrap();
                assert_eq!(dependencies.len(), 0);
                dependencies.push(Arc::clone(this));
            }
            *this.tile_task().external_dependent_.lock().unwrap() = Some(dependent);
        }
    }

    // Source: cc/raster/tile_task.h:25-28
    // Source: cc/raster/tile_task.cc:64-67
    pub fn ExternalDependencyCompleted(&self) {
        let mut dependencies = self.dependencies_.lock().unwrap();
        assert_eq!(dependencies.len(), 1);
        dependencies.clear();
    }
}

impl<C> Drop for TileTask<C> {
    // Source: cc/raster/tile_task.h:64
    // Source: cc/raster/tile_task.cc:24-26
    fn drop(&mut self) {
        debug_assert!(self.did_complete_.load(Ordering::Relaxed));
    }
}

// Source: cc/raster/tile_task.h:74-76
// Source: cc/raster/tile_task.cc:69
// Namespaces/include guards become the Rust module boundary. No worker,
// origin-thread executor or feature implementation is introduced here.
