//! Independent native owners used while assembling a canonical HostImpl.
//!
//! Chromium correspondence:
//! - cc/trees/layer_tree_host_impl.cc:173-264
//! - components/viz/common/surfaces/child_local_surface_id_allocator.cc:14-45
//! - base/containers/lru_cache.h
//!
//! This module supplies allocation, sequence and delivery capabilities. It
//! contains no Page, Host, tree, layer, tile or frame state.

use super::{
    execution::{EngineSequencedTaskRunner, EngineTask, EngineTaskQueue},
    host_metrics::{FrameSequenceTrackers, ReportingController},
    host_owners::{
        AverageLagTrackingManager, ChildLocalSurfaceIdAllocator, ColorSpaceCache,
        EventsMetricsManager, FrameSorter, MutatorEvents, SequenceChecker,
    },
    native_animation::NativeAnimationOwner,
    native_metrics::NativeMetricsServices,
    native_services::NativeTaskRunnerServices,
    resources::CpuColorSpace,
    types::LocalSurfaceId,
};
use crate::{
    adapter::{
        image_animation::{ImageAnimationTask, ImageAnimationTaskRunner},
        layer_tree_host_impl::{
            HostChildLocalSurfaceIdAllocator, HostColorSpaceCache, HostImplTask,
            HostImplTaskRunnerId,
        },
        layer_tree_impl::{LayerTreeHostId, LayerTreeId},
        presentation::SequenceChecker as SequenceCheckerCapability,
        property_tree::PropertyTreesOwnerId,
        task_runner::{RepeatingClosure, SequencedTaskRunner},
        time::TimeDelta,
    },
    engine::CompositorExecutor,
    NativeCompositorTypes,
};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

/// Native entropy belongs to the embedding. The allocator consumes a fresh,
/// non-zero token only when Chromium's source algorithm generates a child id.
pub(crate) trait NativeSurfaceTokenSource: Send + Sync {
    fn NextEmbedToken(&self) -> [u64; 2];
}

pub(crate) struct OsSurfaceTokenSource;
impl NativeSurfaceTokenSource for OsSurfaceTokenSource {
    fn NextEmbedToken(&self) -> [u64; 2] {
        loop {
            let mut bytes = [0u8; 16];
            getrandom::fill(&mut bytes).expect("OS entropy for LocalSurfaceId token");
            let token = [
                u64::from_ne_bytes(bytes[..8].try_into().expect("token half")),
                u64::from_ne_bytes(bytes[8..].try_into().expect("token half")),
            ];
            if token != [0, 0] {
                return token;
            }
        }
    }
}

/// Posted image-animation tasks route back to the resident controller owner;
/// the task itself contains only that owner's weak identity.
pub(crate) trait NativeImageAnimationDispatch: Send + Sync {
    fn Dispatch(&self, task: ImageAnimationTask);
}

pub(crate) struct NativeHostCoreOwners {
    metrics: NativeMetricsServices,
    animation: NativeAnimationOwner,
    task_runners: NativeTaskRunnerServices,
    commands: EngineTaskQueue,
    image_animation_dispatch: Arc<dyn NativeImageAnimationDispatch>,
    surface_tokens: Arc<dyn NativeSurfaceTokenSource>,
    next_tree_identity: AtomicU64,
}

impl NativeHostCoreOwners {
    pub(crate) fn new(
        metrics: NativeMetricsServices,
        animation: NativeAnimationOwner,
        task_runners: NativeTaskRunnerServices,
        commands: EngineTaskQueue,
        image_animation_dispatch: Arc<dyn NativeImageAnimationDispatch>,
        surface_tokens: Arc<dyn NativeSurfaceTokenSource>,
    ) -> Self {
        Self {
            metrics,
            animation,
            task_runners,
            commands,
            image_animation_dispatch,
            surface_tokens,
            next_tree_identity: AtomicU64::new(1),
        }
    }

    pub(crate) fn CreateChildLocalSurfaceIdAllocator(&self) -> ChildLocalSurfaceIdAllocator {
        ChildLocalSurfaceIdAllocator(Box::new(CpuChildLocalSurfaceIdAllocator::new(
            self.surface_tokens.clone(),
        )))
    }

    pub(crate) fn CreateColorSpaceCache(&self, capacity: usize) -> ColorSpaceCache {
        ColorSpaceCache(Box::new(CpuColorSpaceCache::new(capacity)))
    }

    pub(crate) fn ResolveSequencedTaskRunner(
        &self,
        runner: HostImplTaskRunnerId,
    ) -> Arc<dyn SequencedTaskRunner> {
        let executor = self
            .task_runners
            .Resolve(runner)
            .expect("Host task-runner identity must resolve")
            .clone();
        Arc::new(EngineSequencedTaskRunner::new(
            executor,
            self.commands.clone(),
        ))
    }

    pub(crate) fn NewSequenceChecker(&self) -> SequenceChecker {
        let runner = self.task_runners.CompositorRunnerId();
        let executor = self
            .task_runners
            .Resolve(runner)
            .expect("Host sequence-checker runner identity must resolve")
            .clone();
        SequenceChecker(Box::new(ExecutorSequenceChecker(executor)))
    }

    pub(crate) fn MainRunnerId(&self) -> HostImplTaskRunnerId {
        self.task_runners.MainRunnerId()
    }

    pub(crate) fn CompositorRunnerId(&self) -> HostImplTaskRunnerId {
        self.task_runners.CompositorRunnerId()
    }

    pub(crate) fn RouteHostRepeatingClosure(&self, task: HostImplTask) -> RepeatingClosure {
        let commands = self.commands.clone();
        Arc::new(move || commands.Push(EngineTask::Host(task.clone())))
    }

    pub(crate) fn CreateImageAnimationTaskRunner(
        &self,
        runner: HostImplTaskRunnerId,
    ) -> Box<dyn ImageAnimationTaskRunner> {
        let executor = self
            .task_runners
            .Resolve(runner)
            .expect("image-animation runner identity must resolve")
            .clone();
        Box::new(CpuImageAnimationTaskRunner {
            executor,
            dispatch: self.image_animation_dispatch.clone(),
        })
    }

    pub(crate) fn AllocateLayerTreeIdentity(&self) -> (LayerTreeId, PropertyTreesOwnerId) {
        let value = self.next_tree_identity.fetch_add(1, Ordering::Relaxed);
        assert_ne!(value, 0, "layer-tree identity overflow");
        (
            LayerTreeId(value),
            PropertyTreesOwnerId::FromOwnerValue(value),
        )
    }

    pub(crate) fn animation(&self) -> &NativeAnimationOwner {
        &self.animation
    }

    pub(crate) fn MutatorSetDelegate(&self, owner: Option<LayerTreeHostId>) {
        self.animation.SetDelegate(owner)
    }

    pub(crate) fn MutatorCreateEvents(&self) -> MutatorEvents {
        self.animation.CreateEvents()
    }

    pub(crate) fn MutatorClearMutators(&self) {
        self.animation.Clear()
    }

    pub(crate) fn CreateFrameSorter(&self) -> FrameSorter {
        self.metrics.CreateFrameSorter()
    }

    pub(crate) fn CreateFrameTrackers(
        &self,
        single_threaded: bool,
    ) -> FrameSequenceTrackers<NativeCompositorTypes> {
        self.metrics.CreateFrameTrackers(single_threaded)
    }

    pub(crate) fn CreateReportingController(
        &self,
        should_report_histograms: bool,
    ) -> ReportingController<NativeCompositorTypes> {
        self.metrics
            .CreateReportingController(!should_report_histograms)
    }

    pub(crate) fn CreateAverageLagTrackingManager(&self) -> AverageLagTrackingManager {
        self.metrics.CreateAverageLagTrackingManager()
    }
    pub(crate) fn CreateReportingControllerForHost(
        &self,
        should_report_histograms: bool,
        id: i32,
        is_trees_in_viz_client: bool,
    ) -> ReportingController<NativeCompositorTypes> {
        self.metrics.CreateReportingControllerForHost(
            should_report_histograms,
            id,
            is_trees_in_viz_client,
        )
    }

    pub(crate) fn CreateEventsMetricsManager(&self) -> EventsMetricsManager {
        self.metrics.CreateEventsMetricsManager()
    }
}

struct CpuChildLocalSurfaceIdAllocator {
    current: LocalSurfaceId,
    tokens: Arc<dyn NativeSurfaceTokenSource>,
}

impl CpuChildLocalSurfaceIdAllocator {
    fn new(tokens: Arc<dyn NativeSurfaceTokenSource>) -> Self {
        Self {
            current: LocalSurfaceId {
                parent_sequence_number: 0,
                child_sequence_number: 1,
                embed_token: [0, 0],
            },
            tokens,
        }
    }
}

impl HostChildLocalSurfaceIdAllocator for CpuChildLocalSurfaceIdAllocator {
    fn GenerateId(&mut self) {
        let token = self.tokens.NextEmbedToken();
        assert_ne!(token, [0, 0], "surface token source returned invalid token");
        self.current.child_sequence_number = self
            .current
            .child_sequence_number
            .checked_add(1)
            .expect("child LocalSurfaceId sequence overflow");
        self.current.embed_token = token;
    }
}

impl super::host_owners::NativeChildLocalSurfaceIdAllocator for CpuChildLocalSurfaceIdAllocator {
    fn CurrentLocalSurfaceId(&self) -> &LocalSurfaceId {
        &self.current
    }
    fn GetCurrentLocalSurfaceId(&self) -> LocalSurfaceId {
        self.current
    }

    // Source: child_local_surface_id_allocator.cc:33-45.
    fn UpdateFromParent(&mut self, id: &LocalSurfaceId) {
        if self.current.parent_sequence_number >= id.parent_sequence_number
            && self.current.embed_token == id.embed_token
        {
            return;
        }
        if self.current.embed_token != id.embed_token {
            self.current.child_sequence_number = id.child_sequence_number;
        }
        self.current.parent_sequence_number = id.parent_sequence_number;
        self.current.embed_token = id.embed_token;
    }
}

struct CpuColorSpaceCache {
    capacity: usize,
    entries: Vec<(CpuColorSpace, bool)>,
}

impl CpuColorSpaceCache {
    fn new(capacity: usize) -> Self {
        Self {
            capacity,
            entries: Vec::with_capacity(capacity),
        }
    }
}

impl HostColorSpaceCache<CpuColorSpace> for CpuColorSpaceCache {
    fn Get(&mut self, key: &CpuColorSpace) -> Option<&bool> {
        let index = self.entries.iter().position(|(entry, _)| entry == key)?;
        let entry = self.entries.remove(index);
        self.entries.push(entry);
        self.entries.last().map(|(_, value)| value)
    }

    fn Put(&mut self, key: CpuColorSpace, value: bool) {
        if self.capacity == 0 {
            return;
        }
        if let Some(index) = self.entries.iter().position(|(entry, _)| entry == &key) {
            self.entries.remove(index);
        } else if self.entries.len() == self.capacity {
            self.entries.remove(0);
        }
        self.entries.push((key, value));
    }
}

struct ExecutorSequenceChecker(Arc<dyn CompositorExecutor>);
impl SequenceCheckerCapability for ExecutorSequenceChecker {
    fn Check(&self) {
        assert!(
            self.0.RunsTasksInCurrentSequence(),
            "compositor owner used on the wrong executor sequence"
        );
    }
}

struct CpuImageAnimationTaskRunner {
    executor: Arc<dyn CompositorExecutor>,
    dispatch: Arc<dyn NativeImageAnimationDispatch>,
}
impl ImageAnimationTaskRunner for CpuImageAnimationTaskRunner {
    fn BelongsToCurrentThread(&self) -> bool {
        self.executor.RunsTasksInCurrentSequence()
    }

    fn PostDelayedTask(&mut self, task: ImageAnimationTask, delay: TimeDelta) {
        let dispatch = self.dispatch.clone();
        self.executor.ExecuteAfter(
            delay,
            Box::new(move || {
                if task.IsAlive() {
                    dispatch.Dispatch(task);
                }
            }),
        );
    }
}
