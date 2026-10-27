// Copyright 2012 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/tiles/tile_manager.cc:213-262,1633-1700
//
//! Raster task-category selection and dependency-graph construction. Tile
//! policy selects the work; this module owns the graph used to execute it.

use crate::{
    task::{Edge, Node, TaskGraph, TaskRef},
    tile_task::{TileTaskImpl, TileTaskRef},
    TaskCategory,
};

pub const TASK_CATEGORY_NONCONCURRENT_FOREGROUND: u16 =
    TaskCategory::TASK_CATEGORY_NONCONCURRENT_FOREGROUND as u16;
pub const TASK_CATEGORY_FOREGROUND: u16 = TaskCategory::TASK_CATEGORY_FOREGROUND as u16;
pub const TASK_CATEGORY_BACKGROUND: u16 = TaskCategory::TASK_CATEGORY_BACKGROUND as u16;
pub const TASK_CATEGORY_BACKGROUND_WITH_NORMAL_THREAD_PRIORITY: u16 =
    TaskCategory::TASK_CATEGORY_BACKGROUND_WITH_NORMAL_THREAD_PRIORITY as u16;

pub const REQUIRED_FOR_ACTIVATION_DONE_TASK_PRIORITY: u16 = 1;
pub const REQUIRED_FOR_DRAW_DONE_TASK_PRIORITY: u16 = 2;
pub const ALL_DONE_TASK_PRIORITY: u16 = 3;
pub const TILE_TASK_PRIORITY_BASE: u16 = 10;

pub trait TileTaskGraphTraceScope {}

/// Diagnostics are injected without giving raster graph construction access to
/// a TileManager, layer tree, resource pool, or compositor facade.
pub trait TileTaskGraphTrace {
    fn InsertNodeForTask(
        &self,
        category: u16,
        dependencies: usize,
    ) -> Box<dyn TileTaskGraphTraceScope>;
}

// Source: cc/tiles/tile_manager.cc:213-225
fn TaskCategoryForTileTask<C>(task: &dyn TileTaskImpl<C>, use_foreground_category: bool) -> u16 {
    if !task.tile_task().supports_concurrent_execution() {
        return TASK_CATEGORY_NONCONCURRENT_FOREGROUND;
    }
    if use_foreground_category {
        return TASK_CATEGORY_FOREGROUND;
    }
    if !task.tile_task().supports_background_thread_priority() {
        return TASK_CATEGORY_BACKGROUND_WITH_NORMAL_THREAD_PRIORITY;
    }
    TASK_CATEGORY_BACKGROUND
}

// Source: cc/tiles/tile_manager.cc:227-239
fn IsForegroundCategory(category: u16) -> bool {
    match category {
        TASK_CATEGORY_NONCONCURRENT_FOREGROUND | TASK_CATEGORY_FOREGROUND => true,
        TASK_CATEGORY_BACKGROUND | TASK_CATEGORY_BACKGROUND_WITH_NORMAL_THREAD_PRIORITY => false,
        _ => unreachable!("native NOTREACHED: invalid TaskCategory"),
    }
}

// Source: cc/tiles/tile_manager.cc:251-262
pub fn InsertNodeForTask<C>(
    graph: &mut TaskGraph<C>,
    task: TileTaskRef<C>,
    category: u16,
    priority: u16,
    dependencies: usize,
    has_external_dependency: bool,
    trace: &dyn TileTaskGraphTrace,
) {
    let _trace = trace.InsertNodeForTask(category, dependencies);
    debug_assert!(!graph
        .nodes
        .iter()
        .any(|node| std::ptr::eq(node.task.task(), task.task())));
    let task: TaskRef<C> = task;
    graph.nodes.push(Node::new(
        task,
        category,
        priority,
        dependencies as u32,
        has_external_dependency,
    ));
}

// Source: cc/tiles/tile_manager.cc:1633-1654
pub fn InsertNodeForDecodeTask<C>(
    graph: &mut TaskGraph<C>,
    task: TileTaskRef<C>,
    priority: u16,
    use_foreground_category: bool,
    trace: &dyn TileTaskGraphTrace,
) {
    let mut dependency_count = 0u32;
    let mut has_external_dependency = false;
    task.tile_task().WithDependencies(|dependencies| {
        if !dependencies.is_empty() {
            debug_assert_eq!(dependencies.len(), 1);
            let dependency = &dependencies[0];
            if !dependency.tile_task().HasCompleted() {
                if dependency.IsRasterTask() {
                    InsertNodeForDecodeTask(
                        graph,
                        dependency.clone(),
                        priority,
                        use_foreground_category,
                        trace,
                    );
                    let dependency_base: TaskRef<C> = dependency.clone();
                    let task_base: TaskRef<C> = task.clone();
                    graph.edges.push(Edge::new(&dependency_base, &task_base));
                } else {
                    has_external_dependency = true;
                }
                dependency_count = 1;
            }
        }
    });
    let category = TaskCategoryForTileTask(task.as_ref(), use_foreground_category);
    InsertNodeForTask(
        graph,
        task,
        category,
        priority,
        dependency_count as usize,
        has_external_dependency,
        trace,
    );
}

// Source: cc/tiles/tile_manager.cc:1656-1700
pub fn InsertNodesForRasterTask<C>(
    graph: &mut TaskGraph<C>,
    raster_task: TileTaskRef<C>,
    priority: u16,
    use_foreground_category: bool,
    trace: &dyn TileTaskGraphTrace,
) {
    let mut dependencies = 0usize;
    raster_task.tile_task().WithDependencies(|tasks| {
        for decode_task in tasks {
            if decode_task.tile_task().HasCompleted() {
                continue;
            }
            dependencies += 1;
            let decode_it = graph
                .nodes
                .iter()
                .position(|node| std::ptr::eq(node.task.task(), decode_task.task()));
            if let Some(index) = decode_it {
                if use_foreground_category && !IsForegroundCategory(graph.nodes[index].category) {
                    graph.nodes[index].category = TASK_CATEGORY_FOREGROUND;
                }
            }
            if decode_it.is_none() {
                InsertNodeForDecodeTask(
                    graph,
                    decode_task.clone(),
                    priority,
                    use_foreground_category,
                    trace,
                );
            }
            let decode_base: TaskRef<C> = decode_task.clone();
            let raster_base: TaskRef<C> = raster_task.clone();
            graph.edges.push(Edge::new(&decode_base, &raster_base));
        }
    });
    let category = TaskCategoryForTileTask(raster_task.as_ref(), use_foreground_category);
    InsertNodeForTask(
        graph,
        raster_task,
        category,
        priority,
        dependencies,
        false,
        trace,
    );
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RasterTaskGraphMembership {
    pub required_for_activation: bool,
    pub required_for_draw: bool,
    pub needed_now: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CompletionDependencyCounts {
    pub required_for_activation: usize,
    pub required_for_draw: usize,
    pub all: usize,
}

pub struct CompletionTasks<'a, C> {
    pub required_for_activation: &'a TileTaskRef<C>,
    pub required_for_draw: &'a TileTaskRef<C>,
    pub all: &'a TileTaskRef<C>,
}

impl<C> Copy for CompletionTasks<'_, C> {}
impl<C> Clone for CompletionTasks<'_, C> {
    fn clone(&self) -> Self {
        *self
    }
}

pub struct RasterTaskGraphCandidate<C> {
    pub task: TileTaskRef<C>,
    pub membership: RasterTaskGraphMembership,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RasterTaskBatchResult {
    pub completion_counts: CompletionDependencyCounts,
    pub next_priority: usize,
    pub inserted_tasks: usize,
}

/// Inserts the tile-selected raster work in priority order and owns aggregate
/// completion-fence dependency accounting.
pub fn InsertRasterTaskBatchWithCompletionEdges<C>(
    graph: &mut TaskGraph<C>,
    candidates: impl IntoIterator<Item = RasterTaskGraphCandidate<C>>,
    first_priority: usize,
    completion_tasks: CompletionTasks<'_, C>,
    graph_trace: &dyn TileTaskGraphTrace,
) -> RasterTaskBatchResult {
    let mut trace =
        browser_tracing::span("compositor.raster", "Compositor.Raster.InsertRasterTasks");
    let mut result = RasterTaskBatchResult {
        next_priority: first_priority,
        ..Default::default()
    };
    for candidate in candidates {
        let counts = InsertRasterTaskWithCompletionEdges(
            graph,
            candidate.task,
            result.next_priority as u16,
            candidate.membership,
            completion_tasks,
            graph_trace,
        );
        result.completion_counts.required_for_activation += counts.required_for_activation;
        result.completion_counts.required_for_draw += counts.required_for_draw;
        result.completion_counts.all += counts.all;
        result.next_priority += 1;
        result.inserted_tasks += 1;
    }
    trace.set("inserted_tasks", result.inserted_tasks as f64);
    trace.set(
        "activation_dependencies",
        result.completion_counts.required_for_activation as f64,
    );
    trace.set(
        "draw_dependencies",
        result.completion_counts.required_for_draw as f64,
    );
    trace.set("all_dependencies", result.completion_counts.all as f64);
    result
}

/// Adds one tile-selected raster task and all completion-fence edges. Tile
/// selection supplies membership flags; raster owns graph topology/category.
pub fn InsertRasterTaskWithCompletionEdges<C>(
    graph: &mut TaskGraph<C>,
    task: TileTaskRef<C>,
    priority: u16,
    membership: RasterTaskGraphMembership,
    completion_tasks: CompletionTasks<'_, C>,
    trace: &dyn TileTaskGraphTrace,
) -> CompletionDependencyCounts {
    let base: TaskRef<C> = task.clone();
    let mut counts = CompletionDependencyCounts {
        all: 1,
        ..CompletionDependencyCounts::default()
    };
    if membership.required_for_activation {
        counts.required_for_activation = 1;
        let dependent: TaskRef<C> = completion_tasks.required_for_activation.clone();
        graph.edges.push(Edge::new(&base, &dependent));
    }
    if membership.required_for_draw {
        counts.required_for_draw = 1;
        let dependent: TaskRef<C> = completion_tasks.required_for_draw.clone();
        graph.edges.push(Edge::new(&base, &dependent));
    }
    let all_done: TaskRef<C> = completion_tasks.all.clone();
    graph.edges.push(Edge::new(&base, &all_done));
    InsertNodesForRasterTask(
        graph,
        task,
        priority,
        membership.required_for_draw || membership.required_for_activation || membership.needed_now,
        trace,
    );
    counts
}

/// Inserts a newly locked standalone decode task once and connects it to the
/// all-work completion fence. Returns true only when graph topology changed.
pub fn InsertStandaloneDecodeTaskWithCompletionEdge<C>(
    graph: &mut TaskGraph<C>,
    task: TileTaskRef<C>,
    priority: u16,
    all_done_task: &TileTaskRef<C>,
    trace: &dyn TileTaskGraphTrace,
) -> bool {
    if graph
        .nodes
        .iter()
        .any(|node| std::ptr::eq(node.task.task(), task.task()))
    {
        return false;
    }
    InsertNodeForDecodeTask(graph, task.clone(), priority, false, trace);
    let base: TaskRef<C> = task;
    let dependent: TaskRef<C> = all_done_task.clone();
    graph.edges.push(Edge::new(&base, &dependent));
    true
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StandaloneDecodeBatchResult {
    pub next_priority: usize,
    pub inserted_tasks: usize,
}

pub fn InsertStandaloneDecodeTaskBatchWithCompletionEdge<C>(
    graph: &mut TaskGraph<C>,
    tasks: impl IntoIterator<Item = TileTaskRef<C>>,
    first_priority: usize,
    all_done_task: &TileTaskRef<C>,
    graph_trace: &dyn TileTaskGraphTrace,
) -> StandaloneDecodeBatchResult {
    let mut trace =
        browser_tracing::span("compositor.raster", "Compositor.Raster.InsertDecodeTasks");
    let mut result = StandaloneDecodeBatchResult {
        next_priority: first_priority,
        ..Default::default()
    };
    for task in tasks {
        if !InsertStandaloneDecodeTaskWithCompletionEdge(
            graph,
            task,
            result.next_priority as u16,
            all_done_task,
            graph_trace,
        ) {
            continue;
        }
        result.next_priority += 1;
        result.inserted_tasks += 1;
    }
    trace.set("inserted_tasks", result.inserted_tasks as f64);
    result
}

/// Adds the three completion fences and runs them inline when no worker task
/// exists, matching the native completion-only graph semantics.
pub fn FinalizeCompletionGraph<C>(
    graph: &mut TaskGraph<C>,
    tasks: CompletionTasks<'_, C>,
    counts: CompletionDependencyCounts,
    trace: &dyn TileTaskGraphTrace,
) -> bool {
    let mut package_trace =
        browser_tracing::span("compositor.raster", "Compositor.Raster.FinalizeTaskGraph");
    let only_completion_tasks = graph.nodes.is_empty();
    let worker_nodes = graph.nodes.len();
    let worker_edges = graph.edges.len();
    InsertNodeForTask(
        graph,
        tasks.required_for_activation.clone(),
        TASK_CATEGORY_NONCONCURRENT_FOREGROUND,
        REQUIRED_FOR_ACTIVATION_DONE_TASK_PRIORITY,
        counts.required_for_activation,
        false,
        trace,
    );
    InsertNodeForTask(
        graph,
        tasks.required_for_draw.clone(),
        TASK_CATEGORY_NONCONCURRENT_FOREGROUND,
        REQUIRED_FOR_DRAW_DONE_TASK_PRIORITY,
        counts.required_for_draw,
        false,
        trace,
    );
    InsertNodeForTask(
        graph,
        tasks.all.clone(),
        TASK_CATEGORY_NONCONCURRENT_FOREGROUND,
        ALL_DONE_TASK_PRIORITY,
        counts.all,
        false,
        trace,
    );
    package_trace.set("worker_nodes", worker_nodes as f64);
    package_trace.set("worker_edges", worker_edges as f64);
    package_trace.set(
        "activation_dependencies",
        counts.required_for_activation as f64,
    );
    package_trace.set("draw_dependencies", counts.required_for_draw as f64);
    package_trace.set("all_dependencies", counts.all as f64);
    package_trace.set("completion_only", only_completion_tasks as u8 as f64);
    if !only_completion_tasks {
        return false;
    }

    debug_assert_eq!(counts, CompletionDependencyCounts::default());
    for node in &graph.nodes {
        debug_assert_eq!(node.dependencies, 0);
        debug_assert!(!node.has_external_dependency);
        debug_assert!(
            std::ptr::eq(node.task.task(), tasks.required_for_activation.task())
                || std::ptr::eq(node.task.task(), tasks.required_for_draw.task())
                || std::ptr::eq(node.task.task(), tasks.all.task())
        );
        let tile_task = node
            .task
            .AsTileTask()
            .expect("completion-only graph must contain the three concrete completion tile tasks");
        tile_task.task().WithState(|state| state.DidSchedule());
        tile_task.task().WithState(|state| state.DidStart());
        tile_task.RunOnWorkerThread();
        tile_task.task().WithState(|state| state.DidFinish());
        debug_assert!(tile_task.OnTaskCompleted().is_none());
        tile_task.tile_task().DidComplete();
    }
    graph.Reset();
    true
}
