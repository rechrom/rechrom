// Translation status: complete
// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Source: cc/raster/task_graph_work_queue.h:5-31
// Source: cc/raster/task_graph_work_queue.cc:5-21
use super::task::{TaskGraph, TaskRef, TaskVector};
use super::task_graph_runner::NamespaceToken;
use crate::raster_task::TaskGraphTrace;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

// Source: cc/raster/task_graph_work_queue.h:33-51
// Source: cc/raster/task_graph_work_queue.cc:132-144
// NamespaceToken is the non-owning stable identity of the resident map value;
// it replaces TaskNamespace* without copying or owning namespace state.
pub struct PrioritizedTask<C = ()> {
    pub task: TaskRef<C>,
    pub task_namespace: NamespaceToken,
    pub category: u16,
    pub priority: u16,
}
impl<C> PrioritizedTask<C> {
    pub fn new(
        task: TaskRef<C>,
        task_namespace: NamespaceToken,
        category: u16,
        priority: u16,
    ) -> Self {
        Self {
            task,
            task_namespace,
            category,
            priority,
        }
    }
}
// Source: cc/raster/task_graph_work_queue.h:53-82
// Source: cc/raster/task_graph_work_queue.cc:122-127
pub type CategorizedTask<C = ()> = (u16, TaskRef<C>);
pub struct TaskNamespace<C = ()> {
    // Native member destruction is reverse declaration order; Rust drops in
    // declaration order, so running/completed owners precede the graph here.
    pub running_tasks: Vec<CategorizedTask<C>>,
    pub completed_tasks: TaskVector<C>,
    pub ready_to_run_tasks: BTreeMap<u16, Vec<PrioritizedTask<C>>>,
    pub graph: TaskGraph<C>,
}
impl<C> Default for TaskNamespace<C> {
    fn default() -> Self {
        Self {
            graph: TaskGraph::new(),
            ready_to_run_tasks: BTreeMap::new(),
            completed_tasks: Vec::new(),
            running_tasks: Vec::new(),
        }
    }
}
pub type ReadyNamespaces = BTreeMap<u16, Vec<NamespaceToken>>;

// Source: cc/raster/task_graph_work_queue.h:212-219
// The source CompareToken orders the opaque native id, not allocation addresses.
impl PartialOrd for NamespaceToken {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for NamespaceToken {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id().cmp(&other.id())
    }
}

// Source: cc/raster/task_graph_work_queue.cc:23-27
fn CompareTaskPriority<C>(a: &PrioritizedTask<C>, b: &PrioritizedTask<C>) -> bool {
    a.priority > b.priority
}
// Source: cc/raster/task_graph_work_queue.cc:29-48
fn CompareTaskNamespacePriority<C>(
    namespaces: &BTreeMap<NamespaceToken, TaskNamespace<C>>,
    category: u16,
    a: &NamespaceToken,
    b: &NamespaceToken,
) -> bool {
    let a = &namespaces[a].ready_to_run_tasks[&category];
    let b = &namespaces[b].ready_to_run_tasks[&category];
    debug_assert!(!a.is_empty());
    debug_assert!(!b.is_empty());
    CompareTaskPriority(&a[0], &b[0])
}

// Rust spelling of the source's std::{make,push,pop}_heap primitives. These
// operate in place and add no task/category policy. Equal-priority order is
// unspecified by the native standard-library contract as well.
fn SiftDown<T>(values: &mut [T], mut parent: usize, compare: &impl Fn(&T, &T) -> bool) {
    while let Some(mut child) = parent
        .checked_mul(2)
        .and_then(|n| n.checked_add(1))
        .filter(|n| *n < values.len())
    {
        if child + 1 < values.len() && compare(&values[child], &values[child + 1]) {
            child += 1;
        }
        if !compare(&values[parent], &values[child]) {
            break;
        }
        values.swap(parent, child);
        parent = child;
    }
}
fn MakeHeap<T>(values: &mut [T], compare: impl Fn(&T, &T) -> bool) {
    for parent in (0..values.len() / 2).rev() {
        SiftDown(values, parent, &compare);
    }
}
fn PushHeap<T>(values: &mut [T], compare: impl Fn(&T, &T) -> bool) {
    let Some(mut child) = values.len().checked_sub(1) else {
        return;
    };
    while child > 0 {
        let parent = (child - 1) / 2;
        if !compare(&values[parent], &values[child]) {
            break;
        }
        values.swap(parent, child);
        child = parent;
    }
}
fn PopHeap<T>(values: &mut [T], compare: impl Fn(&T, &T) -> bool) {
    if values.len() > 1 {
        let last = values.len() - 1;
        values.swap(0, last);
        SiftDown(&mut values[..last], 0, &compare);
    }
}

// Source: cc/raster/task_graph_work_queue.cc:50-107
// The iterator keeps only its source task borrow and edge cursor. Each step
// resolves the current node in the live graph, permitting the caller to mutate
// that node's dependency count between increments without a retained alias.
struct DependentIterator<'a, C> {
    task: &'a TaskRef<C>,
    current_index: usize,
}
impl<'a, C> DependentIterator<'a, C> {
    fn new(task: &'a TaskRef<C>) -> Self {
        Self {
            task,
            current_index: usize::MAX,
        }
    }
    fn Next(&mut self, graph: &TaskGraph<C>) -> Option<usize> {
        loop {
            self.current_index = self.current_index.wrapping_add(1);
            if self.current_index == graph.edges.len() {
                return None;
            }
            let edge = &graph.edges[self.current_index];
            if std::ptr::addr_eq(edge.task.as_ptr(), Arc::as_ptr(self.task)) {
                let index = graph.nodes.iter().position(|node| {
                    std::ptr::addr_eq(Arc::as_ptr(&node.task), edge.dependent.as_ptr())
                });
                return Some(index.expect("CHECK dependent node belongs to graph"));
            }
        }
    }
}
// Source: cc/raster/task_graph_work_queue.cc:109-120
fn RebuildNamespaceHeaps<C>(
    ready: &mut ReadyNamespaces,
    namespaces: &BTreeMap<NamespaceToken, TaskNamespace<C>>,
) {
    for (&category, task_namespaces) in ready {
        MakeHeap(task_namespaces, |a, b| {
            CompareTaskNamespacePriority(namespaces, category, a, b)
        });
    }
}

// Source: cc/raster/task_graph_work_queue.h:84-112
// Source: cc/raster/task_graph_work_queue.h:203-211,221-235
// Source: cc/raster/task_graph_work_queue.cc:129-130
// The embedding supplies synchronization; this owner creates no thread/clock.
// Rust moves implement deleted-copy/default-move values. Member order matches
// native reverse destruction: scalar, non-owning ready identities, namespaces.
pub struct TaskGraphWorkQueue<C = ()> {
    next_namespace_id_: i32,
    ready_to_run_namespaces_: ReadyNamespaces,
    namespaces_: BTreeMap<NamespaceToken, TaskNamespace<C>>,
}
impl<C> Default for TaskGraphWorkQueue<C> {
    fn default() -> Self {
        Self::new()
    }
}
impl<C> TaskGraphWorkQueue<C> {
    pub fn new() -> Self {
        Self {
            next_namespace_id_: 1,
            ready_to_run_namespaces_: BTreeMap::new(),
            namespaces_: BTreeMap::new(),
        }
    }
    // Source: cc/raster/task_graph_work_queue.cc:146-150
    pub fn GenerateNamespaceToken(&mut self) -> NamespaceToken {
        let token = NamespaceToken::from_id(self.next_namespace_id_);
        self.next_namespace_id_ += 1;
        debug_assert!(!self.namespaces_.contains_key(&token));
        token
    }
    // Source: cc/raster/task_graph_work_queue.cc:152-177
    pub fn ScheduleTasks(
        &mut self,
        token: NamespaceToken,
        graph: &mut TaskGraph<C>,
        trace: &dyn TaskGraphTrace,
    ) {
        let task_namespace = self.namespaces_.entry(token).or_default();
        for task in &task_namespace.completed_tasks {
            let mut node_it = DependentIterator::new(task);
            while let Some(index) = node_it.Next(graph) {
                let node = &mut graph.nodes[index];
                debug_assert!(node.dependencies > 0);
                node.dependencies -= 1;
            }
        }
        for tasks in task_namespace.ready_to_run_tasks.values_mut() {
            tasks.clear();
        }
        {
            let mut event = trace.TraceWorkQueueSchedule();
            for node in &graph.nodes {
                let id = node.task.task().trace_task_id();
                debug_assert_ne!(
                    id, 0,
                    "Every raster task should be associated with a task id."
                );
                event.AddFlowId(id);
            }
        }
        // Source: cc/raster/task_graph_work_queue.cc:179-223
        for node in &graph.nodes {
            if let Some(index) = task_namespace
                .graph
                .nodes
                .iter()
                .position(|old| Arc::ptr_eq(&old.task, &node.task))
            {
                let last = task_namespace.graph.nodes.len() - 1;
                task_namespace.graph.nodes.swap(index, last);
                node.task.task().WithState(|s| {
                    if s.IsScheduled() {
                        s.Reset();
                    }
                });
                task_namespace.graph.nodes.pop();
            }
            if node.dependencies != 0 {
                continue;
            }
            if node.task.task().WithState(|s| s.IsFinished()) {
                continue;
            }
            if task_namespace
                .running_tasks
                .iter()
                .any(|(_, task)| Arc::ptr_eq(task, &node.task))
            {
                continue;
            }
            node.task.task().WithState(|s| s.DidSchedule());
            task_namespace
                .ready_to_run_tasks
                .entry(node.category)
                .or_default()
                .push(PrioritizedTask::new(
                    node.task.clone(),
                    token,
                    node.category,
                    node.priority,
                ));
        }
        for tasks in task_namespace.ready_to_run_tasks.values_mut() {
            MakeHeap(tasks, CompareTaskPriority);
        }
        task_namespace.graph.Swap(graph);
        // Source: cc/raster/task_graph_work_queue.cc:225-243
        for node in &graph.nodes {
            if node.task.task().WithState(|s| s.IsFinished()) {
                continue;
            }
            if task_namespace
                .running_tasks
                .iter()
                .any(|(_, task)| Arc::ptr_eq(task, &node.task))
            {
                continue;
            }
            debug_assert!(!task_namespace
                .completed_tasks
                .iter()
                .any(|task| Arc::ptr_eq(task, &node.task)));
            node.task.task().WithState(|s| s.DidCancel());
            task_namespace.completed_tasks.push(node.task.clone());
        }
        // Source: cc/raster/task_graph_work_queue.cc:245-262
        for namespaces in self.ready_to_run_namespaces_.values_mut() {
            namespaces.clear();
        }
        for (&id, namespace) in &self.namespaces_ {
            for (&category, tasks) in &namespace.ready_to_run_tasks {
                if !tasks.is_empty() {
                    self.ready_to_run_namespaces_
                        .entry(category)
                        .or_default()
                        .push(id);
                }
            }
        }
        RebuildNamespaceHeaps(&mut self.ready_to_run_namespaces_, &self.namespaces_);
    }
    // Source: cc/raster/task_graph_work_queue.cc:264-301
    pub fn GetNextTaskToRun(&mut self, category: u16) -> PrioritizedTask<C> {
        let ready = self.ready_to_run_namespaces_.entry(category).or_default();
        debug_assert!(!ready.is_empty());
        PopHeap(ready, |a, b| {
            CompareTaskNamespacePriority(&self.namespaces_, category, a, b)
        });
        let token = ready.pop().expect("ready namespace");
        let task_namespace = self
            .namespaces_
            .get_mut(&token)
            .expect("resident namespace");
        let tasks = task_namespace
            .ready_to_run_tasks
            .entry(category)
            .or_default();
        debug_assert!(!tasks.is_empty());
        PopHeap(tasks, CompareTaskPriority);
        let task = tasks.pop().expect("ready task");
        if !tasks.is_empty() {
            ready.push(token);
            PushHeap(ready, |a, b| {
                CompareTaskNamespacePriority(&self.namespaces_, category, a, b)
            });
        }
        task.task.task().WithState(|s| s.DidStart());
        self.namespaces_
            .get_mut(&token)
            .unwrap()
            .running_tasks
            .push((task.category, task.task.clone()));
        task
    }
    // Source: cc/raster/task_graph_work_queue.cc:303-319
    pub fn ExternalDependencyCompletedForTask(
        &mut self,
        token: NamespaceToken,
        task: TaskRef<C>,
    ) -> bool {
        let namespace = self.namespaces_.get_mut(&token);
        assert!(namespace.is_some() || task.task().WithState(|s| s.IsCanceled()));
        if let Some(namespace) = namespace {
            let Some(index) = namespace
                .graph
                .nodes
                .iter()
                .position(|node| Arc::ptr_eq(&node.task, &task))
            else {
                return false;
            };
            namespace.graph.nodes[index].has_external_dependency = false;
            return self.DecrementNodeDependencies(index, token, true);
        }
        false
    }
    // Source: cc/raster/task_graph_work_queue.cc:321-368
    fn DecrementNodeDependencies(
        &mut self,
        index: usize,
        token: NamespaceToken,
        rebuild_heap: bool,
    ) -> bool {
        let namespace = self.namespaces_.get_mut(&token).unwrap();
        let node = &mut namespace.graph.nodes[index];
        debug_assert!(node.dependencies > 0);
        node.dependencies -= 1;
        if node.dependencies == 0 && node.task.task().WithState(|s| s.IsNew()) {
            let tasks = namespace
                .ready_to_run_tasks
                .entry(node.category)
                .or_default();
            let was_empty = tasks.is_empty();
            node.task.task().WithState(|s| s.DidSchedule());
            tasks.push(PrioritizedTask::new(
                node.task.clone(),
                token,
                node.category,
                node.priority,
            ));
            PushHeap(tasks, CompareTaskPriority);
            let category = node.category;
            if was_empty {
                let ready = self.ready_to_run_namespaces_.entry(category).or_default();
                debug_assert!(!ready.contains(&token));
                // Native TODO: push_heap could replace the subsequent rebuild
                // when this is the only changed namespace priority.
                ready.push(token);
            }
            if rebuild_heap {
                let ready = self.ready_to_run_namespaces_.entry(category).or_default();
                MakeHeap(ready, |a, b| {
                    CompareTaskNamespacePriority(&self.namespaces_, category, a, b)
                });
            }
            return true;
        }
        false
    }
    // Source: cc/raster/task_graph_work_queue.cc:370-401
    pub fn CompleteTask(&mut self, completed_task: PrioritizedTask<C>) {
        let token = completed_task.task_namespace;
        let task = completed_task.task;
        let namespace = self
            .namespaces_
            .get_mut(&token)
            .expect("resident namespace");
        let index = namespace
            .running_tasks
            .iter()
            .position(|(_, t)| Arc::ptr_eq(t, &task))
            .expect("CHECK completed task was running");
        let last = namespace.running_tasks.len() - 1;
        namespace.running_tasks.swap(index, last);
        namespace.running_tasks.pop();
        let mut heap_properties = true;
        let mut dependent_it = DependentIterator::new(&task);
        while let Some(index) = dependent_it.Next(&self.namespaces_[&token].graph) {
            heap_properties &= !self.DecrementNodeDependencies(index, token, false);
        }
        if !heap_properties {
            RebuildNamespaceHeaps(&mut self.ready_to_run_namespaces_, &self.namespaces_);
        }
        task.task().WithState(|s| s.DidFinish());
        self.namespaces_
            .get_mut(&token)
            .unwrap()
            .completed_tasks
            .push(task);
    }
    // Source: cc/raster/task_graph_work_queue.cc:403-421
    pub fn CollectCompletedTasks(&mut self, token: NamespaceToken, completed: &mut TaskVector<C>) {
        let Some(namespace) = self.namespaces_.get_mut(&token) else {
            return;
        };
        debug_assert!(completed.is_empty());
        std::mem::swap(completed, &mut namespace.completed_tasks);
        if !Self::HasFinishedRunningTasksInNamespace(namespace) {
            return;
        }
        debug_assert!(namespace.completed_tasks.is_empty());
        debug_assert!(!Self::HasReadyToRunTasksInNamespace(namespace));
        debug_assert!(namespace.running_tasks.is_empty());
        self.namespaces_.remove(&token);
    }
    // Source: cc/raster/task_graph_work_queue.cc:423-442
    pub fn DependencyMismatch(graph: &TaskGraph<C>) -> bool {
        let mut dependencies = HashMap::new();
        for edge in &graph.edges {
            *dependencies
                .entry(edge.dependent.as_ptr().cast::<()>())
                .or_insert(0usize) += 1;
        }
        for node in &graph.nodes {
            let mut count = *dependencies
                .entry(Arc::as_ptr(&node.task).cast::<()>())
                .or_insert(0);
            if node.has_external_dependency {
                count += 1;
            }
            if count != node.dependencies as usize {
                return true;
            }
        }
        false
    }
    // Source: cc/raster/task_graph_work_queue.h:114-122
    pub fn GetNamespaceForToken(&self, token: NamespaceToken) -> Option<&TaskNamespace<C>> {
        self.namespaces_.get(&token)
    }
    // Source: cc/raster/task_graph_work_queue.h:124-130
    pub fn HasReadyToRunTasksInNamespace(namespace: &TaskNamespace<C>) -> bool {
        !namespace.ready_to_run_tasks.values().all(Vec::is_empty)
    }
    // Source: cc/raster/task_graph_work_queue.h:132-136
    pub fn HasTasksBlockedOnExternalDependencyInNamespace(namespace: &TaskNamespace<C>) -> bool {
        namespace
            .graph
            .nodes
            .iter()
            .any(|node| node.has_external_dependency)
    }
    // Source: cc/raster/task_graph_work_queue.h:138-143
    pub fn HasFinishedRunningTasksInNamespace(namespace: &TaskNamespace<C>) -> bool {
        namespace.running_tasks.is_empty()
            && !Self::HasReadyToRunTasksInNamespace(namespace)
            && !Self::HasTasksBlockedOnExternalDependencyInNamespace(namespace)
    }
    // Source: cc/raster/task_graph_work_queue.h:145-149
    pub fn HasReadyToRunTasks(&self) -> bool {
        !self.ready_to_run_namespaces_.values().all(Vec::is_empty)
    }
    // Source: cc/raster/task_graph_work_queue.h:151-154
    pub fn HasReadyToRunTasksForCategory(&self, category: u16) -> bool {
        self.ready_to_run_namespaces_
            .get(&category)
            .is_some_and(|tasks| !tasks.is_empty())
    }
    // Source: cc/raster/task_graph_work_queue.h:156
    pub fn HasAnyNamespaces(&self) -> bool {
        !self.namespaces_.is_empty()
    }
    // Source: cc/raster/task_graph_work_queue.h:158-163
    pub fn HasFinishedRunningTasksInAllNamespaces(&self) -> bool {
        self.namespaces_
            .values()
            .all(Self::HasFinishedRunningTasksInNamespace)
    }
    // Source: cc/raster/task_graph_work_queue.h:165-167
    pub fn ready_to_run_namespaces(&self) -> &ReadyNamespaces {
        &self.ready_to_run_namespaces_
    }
    // Source: cc/raster/task_graph_work_queue.h:169-175
    pub fn NumRunningTasks(&self) -> usize {
        self.namespaces_
            .values()
            .map(|ns| ns.running_tasks.len())
            .sum()
    }
    // Source: cc/raster/task_graph_work_queue.h:177-188
    pub fn NumRunningTasksForCategory(&self, category: u16) -> usize {
        let mut count = 0;
        for namespace in self.namespaces_.values() {
            for (task_category, _) in &namespace.running_tasks {
                if *task_category == category {
                    count += 1;
                }
            }
        }
        count
    }
    // Source: cc/raster/task_graph_work_queue.h:190-201
    pub fn NumReadyTasksForCategory(&self, category: u16) -> usize {
        let Some(ready) = self.ready_to_run_namespaces_.get(&category) else {
            return 0;
        };
        let mut count = 0;
        for token in ready {
            let namespace = &self.namespaces_[token];
            debug_assert!(namespace.ready_to_run_tasks.contains_key(&category));
            count += namespace.ready_to_run_tasks[&category].len();
        }
        count
    }
}
