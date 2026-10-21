//! Independent raster capabilities retained by the canonical ResourcePool.
//! This object contains no pool/tree/host reference. The provider clone names
//! the same native import/export allocation and release always has a live client.
use super::{
    client_resources::RasterClientResources,
    execution::{EngineTask, EngineTaskQueue},
    native_services::NativeEventSink,
    resources::{
        RasterResourceSource, RasterSharedImage, RasterSyncToken, RasterSynchronizationType,
        RasterTransferableResource,
    },
};
use crate::{
    adapter::{
        layer_tree_impl::ResourceId,
        resource_pool::*,
        time::{TimeDelta, TimeTicks},
    },
    engine::CompositorExecutor,
    NativeCompositorTypes,
};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

#[derive(Clone, Copy)]
pub(crate) struct RasterPoolConfig {
    pub prefer_exact_size_reuse: bool,
    pub invalidate_resources_on_size_change: bool,
}
pub(crate) struct RasterPoolServices {
    identity: ResourcePoolId,
    resources: RasterClientResources,
    executor: Arc<dyn CompositorExecutor>,
    commands: EngineTaskQueue,
    config: RasterPoolConfig,
    events: Arc<dyn NativeEventSink>,
}
impl RasterPoolServices {
    pub(crate) fn new(
        identity: ResourcePoolId,
        resources: RasterClientResources,
        executor: Arc<dyn CompositorExecutor>,
        commands: EngineTaskQueue,
        config: RasterPoolConfig,
        events: Arc<dyn NativeEventSink>,
    ) -> Self {
        Self {
            identity,
            resources,
            executor,
            commands,
            config,
            events,
        }
    }
}
impl ResourcePoolPlatform<NativeCompositorTypes> for RasterPoolServices {
    fn MakeTransferableResource(
        &self,
        image: RasterSharedImage,
        source: RasterResourceSource,
        sync_token: RasterSyncToken,
    ) -> RasterTransferableResource {
        // Source value constructor: components/viz/common/resources/transferable_resource.cc:28-41.
        RasterTransferableResource {
            id: ResourceId::default(),
            shared_image: image,
            source,
            sync_token,
            synchronization_type: RasterSynchronizationType::SyncToken,
        }
    }
    fn ImportResource(
        &self,
        resource: RasterTransferableResource,
        callback: ResourcePoolReleaseCallback,
    ) -> ResourceId {
        assert_eq!(callback.owner.id, self.identity, "source pool import owner");
        self.resources.ImportPoolResource(resource, callback)
    }
    fn RemoveImportedResource(
        &self,
        id: ResourceId,
        client: &mut dyn PoolReleaseClient<NativeCompositorTypes>,
    ) {
        self.resources.RemovePoolResource(self.identity, id, client);
    }
    fn FlushPendingWork(&self) {
        // The current host-readable storage has no deferred command stream to flush.
    }
}
impl ResourcePoolTiming for RasterPoolServices {
    fn DefaultTickClock(&self) -> PoolClockId {
        PoolClockId(1)
    }
    fn NowTicks(&self, clock: PoolClockId) -> TimeTicks {
        assert_eq!(
            clock,
            self.DefaultTickClock(),
            "source pool clock capability"
        );
        self.executor.Now()
    }
    fn PostDelayedEviction(&self, owner: WeakResourcePoolId, delay: TimeDelta) {
        assert_eq!(owner.id, self.identity, "source eviction target");
        let commands = self.commands.clone();
        self.executor.ExecuteAfter(
            delay,
            Box::new(move || {
                commands.Push(EngineTask::PoolEviction(owner));
            }),
        );
    }
}
struct RasterPoolTrack;
impl PoolTracingTrack for RasterPoolTrack {}
struct RasterPoolScope {
    events: Arc<dyn NativeEventSink>,
    name: &'static str,
}
impl PoolTraceScope for RasterPoolScope {}
impl Drop for RasterPoolScope {
    fn drop(&mut self) {
        self.events
            .Emit("compositor.resource_pool", format!("{} end", self.name));
    }
}
static NEXT_POOL_TRACE_ID: AtomicU64 = AtomicU64::new(1);
impl ResourcePoolDiagnostics for RasterPoolServices {
    fn GetNextGlobalTraceId(&self) -> u64 {
        NEXT_POOL_TRACE_ID
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .expect("native trace ID exhausted")
    }
    fn RegisterTracingTrack(
        &self,
        name: &str,
        tracing_id: i32,
        disable_sibling_merge: bool,
    ) -> Box<dyn PoolTracingTrack> {
        self.events.Emit(
            "compositor.resource_pool",
            format!("track {name} id={tracing_id} disable_sibling_merge={disable_sibling_merge}"),
        );
        Box::new(RasterPoolTrack)
    }
    fn RegisterDumpProvider(&self, owner: WeakResourcePoolId, name: &str) {
        self.events.Emit(
            "compositor.resource_pool",
            format!("register_dump {name} owner={}", owner.id.0),
        );
    }
    fn UnregisterDumpProvider(&self, owner: ResourcePoolId) {
        self.events.Emit(
            "compositor.resource_pool",
            format!("unregister_dump owner={}", owner.0),
        );
    }
    fn HistogramMemoryMediumMb(&self, name: &str, value: usize) {
        self.events.HistogramCount(name, value as f32);
    }
    fn HistogramCounts10000(&self, name: &str, value: usize) {
        self.events.HistogramCount(name, value as f32);
    }
    fn TraceCounter(&self, tracing_id: i32, name: &str, value: usize) {
        self.events.Emit(
            "compositor.resource_pool",
            format!("counter id={tracing_id} {name}={value}"),
        );
    }
    fn Trace(&self, event: ResourcePoolTraceEvent) -> Box<dyn PoolTraceScope + '_> {
        let name = match event {
            ResourcePoolTraceEvent::OnResourceReleased => "OnResourceReleased",
            ResourcePoolTraceEvent::NotifyOfViewportSizeChange { old_size, new_size } => {
                self.events.Emit(
                    "compositor.resource_pool",
                    format!("viewport {old_size:?} -> {new_size:?}"),
                );
                "NotifyOfViewportSizeChange"
            }
            ResourcePoolTraceEvent::InvalidateResources => "InvalidateResources",
        };
        self.events
            .Emit("compositor.resource_pool", format!("{name} begin"));
        Box::new(RasterPoolScope {
            events: self.events.clone(),
            name,
        })
    }
    fn ResourcePoolPreferExactSizeReuse(&self) -> bool {
        self.config.prefer_exact_size_reuse
    }
    fn InvalidateResourcesOnSizeChange(&self) -> bool {
        self.config.invalidate_resources_on_size_change
    }
}
