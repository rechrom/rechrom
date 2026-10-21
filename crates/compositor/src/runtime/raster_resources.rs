//! Injected resource bridge for the canonical ResourcePool/RasterBufferProvider.
//! Only native allocation/replay capabilities live here. The source pool owns
//! lease state and budgets; TileManager owns scheduling and completion order.
use crate::{
    adapter::{
        layer_tree_impl::ResourceId,
        resource_pool::{
            PoolSharedImageInterface, SharedImageDescriptor, SharedImageTestDescriptor,
        },
        tile_manager::RasterBufferProvider,
    },
    cc::resources::resource_pool::{InUsePoolResource, ResourcePool},
    engine::CompositorRasterProvider,
    runtime::types::{CpuCompositorTypes, CpuSkiaBackend},
    runtime::{
        execution::{EngineTask, EngineTaskQueue},
        paint::{RasterFailureHandler, TileRasterBuffer},
        resources::{RasterBufferUsage, RasterColorSpace, RasterSharedImage},
        skia::SkRegion,
    },
};
use std::{
    io,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};

/// Native allocation identities are independent of Viz's imported-resource
/// identities. This allocator owns no pool, backing, pixel or exported frame.
pub(crate) struct RasterAllocationIds(AtomicU32);
impl RasterAllocationIds {
    pub(crate) fn new() -> Self {
        Self(AtomicU32::new(1))
    }
    fn Next(&self) -> ResourceId {
        let id = self
            .0
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .expect("raster allocation identity exhausted");
        ResourceId(id)
    }
}

pub(crate) struct EngineRasterFailure(pub(crate) EngineTaskQueue);
impl RasterFailureHandler for EngineRasterFailure {
    fn OnRasterFailure(&self, error: io::Error) {
        self.0.Push(EngineTask::RasterFailure(error));
    }
}

/// Source shared-image calls retain the actual native image returned by the
/// pixel backend. Neither service nor allocator stores a pool/owner backpointer.
pub(crate) struct RasterPoolImages {
    provider: Arc<dyn CompositorRasterProvider>,
    ids: Arc<RasterAllocationIds>,
    failure: Arc<dyn RasterFailureHandler>,
}
impl RasterPoolImages {
    pub(crate) fn new(
        provider: Arc<dyn CompositorRasterProvider>,
        ids: Arc<RasterAllocationIds>,
        failure: Arc<dyn RasterFailureHandler>,
    ) -> Self {
        Self {
            provider,
            ids,
            failure,
        }
    }
    fn Allocate(
        &self,
        size: crate::adapter::geometry::Size,
        format: &crate::adapter::resource_types::SharedImageFormat,
    ) -> Option<RasterSharedImage> {
        match self.provider.CreateSharedImage(self.ids.Next(), size, format) {
            Ok(image) => Some(image),
            Err(error) => {
                self.failure.OnRasterFailure(error);
                None
            }
        }
    }
}
impl<R: CpuSkiaBackend> PoolSharedImageInterface<CpuCompositorTypes<R>> for RasterPoolImages {
    fn CreateSharedImage(
        &self,
        descriptor: SharedImageDescriptor<'_, RasterColorSpace>,
        _buffer_usage: Option<RasterBufferUsage>,
    ) -> Option<RasterSharedImage> {
        self.Allocate(*descriptor.size, descriptor.format)
    }
    fn CreateSharedImageForSoftwareCompositor(
        &self,
        descriptor: SharedImageDescriptor<'_, RasterColorSpace>,
    ) -> Option<RasterSharedImage> {
        self.Allocate(*descriptor.size, descriptor.format)
    }
    fn CreateSharedImageForTesting(
        &self,
        descriptor: SharedImageTestDescriptor<'_, RasterColorSpace>,
        _texture_target: u32,
    ) -> Option<RasterSharedImage> {
        self.Allocate(*descriptor.size, descriptor.format)
    }
}

pub(crate) struct ExternalRasterBufferProvider {
    provider: Arc<dyn CompositorRasterProvider>,
    images: RasterPoolImages,
    failure: Arc<dyn RasterFailureHandler>,
}
impl ExternalRasterBufferProvider {
    pub(crate) fn new(
        provider: Arc<dyn CompositorRasterProvider>,
        ids: Arc<RasterAllocationIds>,
        failure: Arc<dyn RasterFailureHandler>,
    ) -> Self {
        Self {
            images: RasterPoolImages::new(provider.clone(), ids, failure.clone()),
            provider,
            failure,
        }
    }
}
impl RasterBufferProvider<CpuCompositorTypes<SkRegion>> for ExternalRasterBufferProvider
{
    fn Shutdown(&mut self) {
        if let Err(error) = self.provider.Shutdown() {
            self.failure.OnRasterFailure(error);
        }
    }
    fn NotifyWorkSubmitted(&mut self) {
        self.provider.NotifyWorkSubmitted();
    }
    // Source: cc/raster/zero_copy_raster_buffer_provider.cc:197-201.
    // Playback publishes the raster resource before the worker task completes;
    // completed zero-copy resources have no deferred GPU work to wait for.
    fn IsResourceReadyToDraw(
        &self,
        _resource: &InUsePoolResource<CpuCompositorTypes<SkRegion>>,
    ) -> bool {
        true
    }

    fn CanPartialRasterIntoProvidedResource(&self) -> bool {
        self.provider.CanPartialRasterIntoProvidedResource()
    }
    fn AcquireBufferForRaster(
        &mut self,
        resource: &InUsePoolResource<CpuCompositorTypes<SkRegion>>,
        pool: &mut ResourcePool<CpuCompositorTypes<SkRegion>>,
        resource_content_id: u64,
        previous_content_id: u64,
    ) -> Option<TileRasterBuffer<SkRegion>> {
        // Source boundary: cc/raster/zero_copy_raster_buffer_provider.cc:38-55.
        // The current host-readable branch installs the real backing before playback,
        // then records that same image's creation sync token. Installation
        // calls canonical ResourcePool::OnBackingAllocated on the live pool.
        if resource.WithBacking(|backing| backing.is_none()) {
            resource.InstallSoftwareBacking(
                &self.images,
                "ZeroCopyRasterBufferProviderSoftware",
                pool,
            );
            resource.WithBacking(|backing| {
                let backing = backing.expect("source installed backing");
                let image = backing
                    .shared_image()
                    .expect("source software shared image");
                backing.set_mailbox_sync_token(image.creation_sync_token());
            });
        }
        let id = resource.WithBacking(|backing| {
            backing
                .expect("source backing")
                .shared_image()
                .expect("source software shared image")
                .RasterAllocationId()
        });
        match self.provider.AcquireBufferForRaster(
            id,
            *resource.size(),
            resource.format(),
            resource_content_id,
            previous_content_id,
        ) {
            Ok(buffer) => Some(TileRasterBuffer::FromBoxed(buffer, self.failure.clone())),
            Err(error) => {
                self.failure.OnRasterFailure(error);
                None
            }
        }
    }
}

impl ExternalRasterBufferProvider {
    pub(crate) fn shared_images(&self) -> &RasterPoolImages {
        &self.images
    }
    // Source: cc/raster/zero_copy_raster_buffer_provider.cc:203-209.
    pub(crate) fn SetReadyToDrawCallback(
        &self,
        _resources: &[&InUsePoolResource<CpuCompositorTypes<SkRegion>>],
        _callback: crate::adapter::task_runner::OnceClosure,
        _previous: u64,
    ) -> u64 {
        0
    }
}
