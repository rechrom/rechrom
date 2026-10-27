// Translation status: complete
// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Source: cc/raster/task.h:5-19
// Source: cc/raster/task.cc:5-15
// Includes, export declarations, forward declarations and namespaces map to
// Rust modules/imports. The process-global trace-id service is constructor
// injection; this translation does not create or inspect worker threads.
use crate::raster_task::TraceTaskIdProvider;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex, Weak};

// Source: cc/raster/task.h:20-77
// New -> Scheduled -> Running -> Finished, or New/Scheduled -> Canceled.
// Only Task constructs this state. Its mutation methods do not synchronize;
// the holder supplies exclusive access, as required by the source contract.
#[repr(u16)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum Value {
    kNew,
    kScheduled,
    kRunning,
    kFinished,
    kCanceled,
}

pub struct TaskState {
    value_: Value,
}

impl TaskState {
    // Source: cc/raster/task.h:61-66
    // Source: cc/raster/task.cc:16
    fn new() -> Self {
        Self {
            value_: Value::kNew,
        }
    }

    // Source: cc/raster/task.h:43
    // Source: cc/raster/task.cc:26-28
    pub fn IsNew(&self) -> bool {
        self.value_ == Value::kNew
    }

    // Source: cc/raster/task.h:44
    // Source: cc/raster/task.cc:30-32
    pub fn IsScheduled(&self) -> bool {
        self.value_ == Value::kScheduled
    }

    // Source: cc/raster/task.h:45
    // Source: cc/raster/task.cc:34-36
    pub fn IsRunning(&self) -> bool {
        self.value_ == Value::kRunning
    }

    // Source: cc/raster/task.h:46
    // Source: cc/raster/task.cc:38-40
    pub fn IsFinished(&self) -> bool {
        self.value_ == Value::kFinished
    }

    // Source: cc/raster/task.h:47
    // Source: cc/raster/task.cc:42-44
    pub fn IsCanceled(&self) -> bool {
        self.value_ == Value::kCanceled
    }

    // Source: cc/raster/task.h:49-53
    // Source: cc/raster/task.cc:46-48
    pub fn Reset(&mut self) {
        self.value_ = Value::kNew;
    }

    // Source: cc/raster/task.h:59
    // Source: cc/raster/task.cc:50-64
    pub fn ToString(&self) -> String {
        match self.value_ {
            Value::kNew => "NEW",
            Value::kScheduled => "SCHEDULED",
            Value::kRunning => "RUNNING",
            Value::kFinished => "FINISHED",
            Value::kCanceled => "CANCELED",
        }
        .to_owned()
        // The exhaustive enum match encodes the source NOTREACHED().
    }

    // Source: cc/raster/task.h:54
    // Source: cc/raster/task.cc:66-70
    pub fn DidSchedule(&mut self) {
        debug_assert!(
            self.value_ == Value::kNew,
            "Task should be in NEW state to get scheduled."
        );
        self.value_ = Value::kScheduled;
    }

    // Source: cc/raster/task.h:55
    // Source: cc/raster/task.cc:72-77
    pub fn DidStart(&mut self) {
        debug_assert!(
            self.value_ == Value::kScheduled,
            "Task should be only in SCHEDULED state to start, that is it should not be started or finished."
        );
        self.value_ = Value::kRunning;
    }

    // Source: cc/raster/task.h:56
    // Source: cc/raster/task.cc:79-83
    pub fn DidFinish(&mut self) {
        debug_assert!(
            self.value_ == Value::kRunning,
            "Task should be running and not finished earlier."
        );
        self.value_ = Value::kFinished;
    }

    // Source: cc/raster/task.h:57
    // Source: cc/raster/task.cc:85-89
    pub fn DidCancel(&mut self) {
        debug_assert!(
            self.value_ == Value::kNew || self.value_ == Value::kScheduled,
            "Task should be either new or scheduled to get canceled."
        );
        self.value_ = Value::kCanceled;
    }
}

impl Drop for TaskState {
    // Source: cc/raster/task.cc:18-24
    fn drop(&mut self) {
        debug_assert!(
            self.value_ != Value::kRunning,
            "Running task should never get destroyed."
        );
        debug_assert!(
            self.value_ == Value::kFinished || self.value_ == Value::kCanceled,
            "Task, if scheduled, should get concluded either in FINISHED or CANCELED state."
        );
    }
}

// Source: cc/raster/task.h:79-109
// Arc<TaskImpl> implements the source's refcounted polymorphic owner. A derived
// task owns this base exactly once and exposes it through TaskImpl::task().
// The local mutable-data cell only supplies safe exclusive access through a
// shared Arc: no queue, scheduling, worker dispatch, or lifecycle policy lives
// in this cell. TaskState remains the sole state-transition implementation.
// Relaxed frame-number storage only makes the source scalar field safe through
// a shared refcounted owner; it is not a publication or scheduling primitive.
pub struct Task {
    state_: Mutex<TaskState>,
    frame_number_: AtomicI64,
    trace_task_id_: u64,
}

// Source: cc/raster/task.h:82-84,94-103
// A derived implementation remains responsible for its own worker-data
// synchronization. Arc drops the concrete derived object and its base, as the
// native virtual destructor and RefCountedThreadSafe<Task> require.
// Rust boundary: C explicitly carries the result of the source origin-thread
// completion callback through the existing Page command/effect contract. It
// does not alter worker execution or completion -> DidComplete -> dependency
// ordering. Tasks without a completion value use the default C = ().
pub trait TaskImpl<C = ()>: Send + Sync {
    fn task(&self) -> &Task;
    fn RunOnWorkerThread(&self);

    // Source: cc/tiles/tile_task_manager.cc:54-58
    // Safe typed projection for this source static_cast only. TileTaskManager
    // namespaces require a TileTask-derived owner; other Task implementations
    // explicitly return None. There is no Any/unsafe cast or default stand-in.
    fn AsTileTask(&self) -> Option<&dyn super::tile_task::TileTaskImpl<C>>;
}

pub type TaskRef<C = ()> = Arc<dyn TaskImpl<C>>;
pub type TaskVector<C = ()> = Vec<TaskRef<C>>;

impl Task {
    // Source: cc/raster/task.h:102-108
    // Source: cc/raster/task.cc:91-93
    pub fn new(trace_ids: &(impl TraceTaskIdProvider + ?Sized)) -> Self {
        Self {
            state_: Mutex::new(TaskState::new()),
            frame_number_: AtomicI64::new(-1),
            trace_task_id_: trace_ids.GetNextGlobalTraceId(),
        }
    }

    // Source: cc/raster/task.h:86
    // Rust borrow split: a scoped mutable reference replaces the source's
    // externally synchronized TaskState&. The access cell/guard is private;
    // caller algorithms still operate directly on the one resident TaskState.
    pub fn WithState<R>(&self, callback: impl FnOnce(&mut TaskState) -> R) -> R {
        let mut state = self.state_.lock().unwrap();
        callback(&mut state)
    }

    // Source: cc/raster/task.h:87
    pub fn set_frame_number(&self, frame_number: i64) {
        self.frame_number_.store(frame_number, Ordering::Relaxed);
    }

    // Source: cc/raster/task.h:88
    pub fn frame_number(&self) -> i64 {
        self.frame_number_.load(Ordering::Relaxed)
    }

    // Source: cc/raster/task.h:90-92
    pub fn trace_task_id(&self) -> u64 {
        self.trace_task_id_
    }
}

// Source: cc/raster/task.h:120-140
// Deliberately not Clone: native Node copy construction/assignment are deleted.
// Rust moves provide the native defaulted move construction and assignment.
pub struct Node<C = ()> {
    pub task: TaskRef<C>,
    pub category: u16,
    pub priority: u16,
    pub dependencies: u32,
    pub has_external_dependency: bool,
}

pub type NodeVector<C = ()> = Vec<Node<C>>;

impl<C> Node<C> {
    // Source: cc/raster/task.h:123-133
    // Source: cc/raster/task.cc:101-114
    pub fn new(
        new_task: TaskRef<C>,
        category: u16,
        priority: u16,
        dependencies: u32,
        has_external_dependency: bool,
    ) -> Self {
        Self {
            task: new_task,
            category,
            priority,
            dependencies,
            has_external_dependency,
        }
    }
}

// Source: cc/raster/task.h:142-150
// Weak is the safe non-owning raw_ptr representation: an Edge never keeps a
// Task alive. Both endpoint identities remain stable while nodes own the tasks;
// native consumers resolve them against those live owners. Cloning an Edge
// clones only the non-owning references, matching source raw_ptr copies.
pub struct Edge<C = ()> {
    pub task: Weak<dyn TaskImpl<C>>,
    pub dependent: Weak<dyn TaskImpl<C>>,
}

pub type EdgeVector<C = ()> = Vec<Edge<C>>;

impl<C> Clone for Edge<C> {
    fn clone(&self) -> Self {
        Self {
            task: self.task.clone(),
            dependent: self.dependent.clone(),
        }
    }
}
impl<C> Edge<C> {
    // Source: cc/raster/task.h:145-146
    pub fn new(task: &TaskRef<C>, dependent: &TaskRef<C>) -> Self {
        Self {
            task: Arc::downgrade(task),
            dependent: Arc::downgrade(dependent),
        }
    }
}

// Source: cc/raster/task.h:111-119,152-165
// Source: cc/raster/task.cc:95-99
// Declaration order intentionally reverses the two native members: Rust drops
// members in declaration order, C++ in reverse order. Thus default destruction
// releases edges before the nodes that own their endpoints.
// No Clone is implemented; Rust's moves implement the deleted-copy/default-move
// contract without a second graph or duplicate task state.
pub struct TaskGraph<C = ()> {
    pub edges: EdgeVector<C>,
    pub nodes: NodeVector<C>,
}

impl<C> Default for TaskGraph<C> {
    fn default() -> Self {
        Self {
            edges: Vec::new(),
            nodes: Vec::new(),
        }
    }
}
impl<C> TaskGraph<C> {
    // Source: cc/raster/task.h:152
    // Source: cc/raster/task.cc:95
    pub fn new() -> Self {
        Self::default()
    }

    // Source: cc/raster/task.h:157-158
    // Explicit assignment of an existing graph retains native member assignment
    // order (nodes, then edges), unlike whole-value replacement's drop order.
    pub fn MoveAssign(&mut self, other: Self) {
        let Self { nodes, edges } = other;
        self.nodes = nodes;
        self.edges = edges;
    }

    // Source: cc/raster/task.h:160
    // Source: cc/raster/task.cc:116-119
    pub fn Swap(&mut self, other: &mut Self) {
        std::mem::swap(&mut self.nodes, &mut other.nodes);
        std::mem::swap(&mut self.edges, &mut other.edges);
    }

    // Source: cc/raster/task.h:161
    // Source: cc/raster/task.cc:121-126
    pub fn Reset(&mut self) {
        // Clear non-owning edges before destroying their node-owned tasks.
        self.edges.clear();
        self.nodes.clear();
    }
}

// Source: cc/raster/task.h:167-169
// Source: cc/raster/task.cc:128
// Namespace and include-guard closure map to the Rust module boundary.
