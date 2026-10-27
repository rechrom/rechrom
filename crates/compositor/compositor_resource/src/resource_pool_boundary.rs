//! Independent platform capabilities for the canonical resource-pool owner.
//! These interfaces neither own nor query Page/Host/Layer/Tile/ResourcePool.
//! Callback/task registration carries weak identity and payload only; the root
//! routes them to the current live canonical owner on its sequence.
use crate::{resource_types::SharedImageFormat, ResourceId};
use compositor_scheduler::{TimeDelta, TimeTicks};
use compositor_tree::geometry::Size;
use std::sync::Weak;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourcePoolId(pub u64);
#[derive(Clone)]
pub struct WeakResourcePoolId {
    pub id: ResourcePoolId,
    pub(crate) lifetime: Weak<()>,
}
impl WeakResourcePoolId {
    pub fn IsValid(&self) -> bool {
        self.lifetime.strong_count() != 0
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PoolClockId(pub u64);
#[derive(Clone)]
pub struct ResourcePoolReleaseCallback {
    pub owner: WeakResourcePoolId,
    pub unique_id: usize,
}
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
// Native value vocabulary: gpu/command_buffer/common/shared_image_usage.h:19-72.
pub struct SharedImageUsageSet(pub u32);
impl SharedImageUsageSet {
    pub const DISPLAY_READ: Self = Self(1 << 2);
    pub const RASTER_WRITE: Self = Self(1 << 18);
    pub const SCANOUT: Self = Self(1 << 4);
    pub const CPU_WRITE_ONLY: Self = Self(1 << 11);
}
impl std::ops::BitOr for SharedImageUsageSet {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}
impl std::ops::BitOrAssign for SharedImageUsageSet {
    fn bitor_assign(&mut self, other: Self) {
        self.0 |= other.0;
    }
}
pub struct SharedImageDescriptor<'a, C> {
    pub format: &'a SharedImageFormat,
    pub size: &'a Size,
    pub color_space: &'a C,
    pub usage: SharedImageUsageSet,
    pub debug_label: &'a str,
}
#[derive(Clone, Copy)]
pub enum TestSurfaceOrigin {
    TopLeft,
}
#[derive(Clone, Copy)]
pub enum TestAlphaType {
    Opaque,
}
pub struct SharedImageTestDescriptor<'a, C> {
    pub format: &'a SharedImageFormat,
    pub size: &'a Size,
    pub color_space: &'a C,
    pub surface_origin: TestSurfaceOrigin,
    pub alpha_type: TestAlphaType,
    pub usage: SharedImageUsageSet,
}
pub trait ResourceSyncToken: Clone + Default + Send + Sync {
    fn HasData(&self) -> bool;
}
pub trait ResourcePoolMemoryDump<G> {
    fn CreateAllocatorDump(&mut self, name: &str) -> G;
    fn AddScalar(&mut self, guid: &G, name: &str, units: &str, value: u64);
}
#[derive(Clone, Copy)]
pub enum SharedImageTracingImportance {
    ClientOwner,
}
pub trait PoolSharedImage<T: ResourceSyncToken, G>: Clone + Send + Sync {
    fn UpdateDestructionSyncToken(&self, token: &T);
    fn OnMemoryDump(
        &self,
        dump: &mut dyn ResourcePoolMemoryDump<G>,
        guid: &G,
        importance: SharedImageTracingImportance,
    );
}
pub trait PoolTransferableResource {
    fn SetGpuCommandsCompletedSynchronization(&mut self);
}
pub trait ResourcePoolTypes: Send + Sync + 'static {
    type ColorSpace: Clone + PartialEq + Send + Sync;
    type SyncToken: ResourceSyncToken;
    type SharedImage: PoolSharedImage<Self::SyncToken, Self::DumpGuid>;
    type DumpGuid;
    type BufferUsage;
    type ResourceSource;
    type TransferableResource: PoolTransferableResource;
    // Value operations supplied by the owner of viz::SharedImageFormat.
    fn EstimatedSizeInBytes(format: &SharedImageFormat, size: &Size) -> usize;
    fn VerifySizeInBytes(format: &SharedImageFormat, size: &Size) -> bool;
}
pub trait PoolSharedImageInterface<B: ResourcePoolTypes> {
    fn CreateSharedImage(
        &self,
        descriptor: SharedImageDescriptor<'_, B::ColorSpace>,
        buffer_usage: Option<B::BufferUsage>,
    ) -> Option<B::SharedImage>;
    fn CreateSharedImageForSoftwareCompositor(
        &self,
        descriptor: SharedImageDescriptor<'_, B::ColorSpace>,
    ) -> Option<B::SharedImage>;
    fn CreateSharedImageForTesting(
        &self,
        descriptor: SharedImageTestDescriptor<'_, B::ColorSpace>,
        texture_target: u32,
    ) -> Option<B::SharedImage>;
}
pub trait PoolReleaseClient<B: ResourcePoolTypes> {
    fn OnResourceReleased(&mut self, unique_id: usize, sync_token: &B::SyncToken, lost: bool);
}
pub trait ResourcePoolPlatform<B: ResourcePoolTypes> {
    fn MakeTransferableResource(
        &self,
        image: B::SharedImage,
        source: B::ResourceSource,
        sync_token: B::SyncToken,
    ) -> B::TransferableResource;
    fn ImportResource(
        &self,
        resource: B::TransferableResource,
        callback: ResourcePoolReleaseCallback,
    ) -> ResourceId;
    /// May synchronously invoke client; must not retain this call's client.
    fn RemoveImportedResource(&self, id: ResourceId, client: &mut dyn PoolReleaseClient<B>);
    fn FlushPendingWork(&self);
}
pub trait ResourcePoolTiming {
    fn DefaultTickClock(&self) -> PoolClockId;
    fn NowTicks(&self, clock: PoolClockId) -> TimeTicks;
    /// Enqueue-only; execute canonical EvictExpiredResources on the live owner.
    fn PostDelayedEviction(&self, owner: WeakResourcePoolId, delay: TimeDelta);
}
pub trait PoolTraceScope {}
pub trait PoolTracingTrack {}
#[derive(Clone, Copy)]
pub enum ResourcePoolTraceEvent {
    OnResourceReleased,
    NotifyOfViewportSizeChange { old_size: Size, new_size: Size },
    InvalidateResources,
}
pub trait ResourcePoolDiagnostics {
    fn GetNextGlobalTraceId(&self) -> u64;
    fn RegisterTracingTrack(
        &self,
        name: &str,
        tracing_id: i32,
        disable_sibling_merge: bool,
    ) -> Box<dyn PoolTracingTrack>;
    fn RegisterDumpProvider(&self, owner: WeakResourcePoolId, name: &str);
    fn UnregisterDumpProvider(&self, owner: ResourcePoolId);
    fn HistogramMemoryMediumMb(&self, name: &str, value: usize);
    fn HistogramCounts10000(&self, name: &str, value: usize);
    fn TraceCounter(&self, tracing_id: i32, name: &str, value: usize);
    fn Trace(&self, event: ResourcePoolTraceEvent) -> Box<dyn PoolTraceScope + '_>;
    fn ResourcePoolPreferExactSizeReuse(&self) -> bool;
    fn InvalidateResourcesOnSizeChange(&self) -> bool;
}
pub trait ResourcePoolServices<B: ResourcePoolTypes>:
    ResourcePoolPlatform<B> + ResourcePoolTiming + ResourcePoolDiagnostics
{
}
impl<
        B: ResourcePoolTypes,
        T: ResourcePoolPlatform<B> + ResourcePoolTiming + ResourcePoolDiagnostics,
    > ResourcePoolServices<B> for T
{
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MemoryDumpLevelOfDetail {
    Background,
    Detailed,
}
