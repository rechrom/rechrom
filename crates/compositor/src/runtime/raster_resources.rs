//! CPU storage bridge for the canonical ResourcePool/RasterBufferProvider.
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
    engine::CompositorPixelProvider,
    runtime::types::{CpuCompositorTypes, CpuSkiaBackend},
    runtime::{
        execution::{EngineTask, EngineTaskQueue},
        paint::{CpuRasterFailureHandler, CpuTileRasterBuffer},
        resources::{CpuBufferUsage, CpuColorSpace, CpuSharedImage},
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
pub(crate) struct CpuAllocationIds(AtomicU32);
impl CpuAllocationIds {
    pub(crate) fn new() -> Self {
        Self(AtomicU32::new(1))
    }
    fn Next(&self) -> ResourceId {
        let id = self
            .0
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .expect("CPU allocation identity exhausted");
        ResourceId(id)
    }
}

pub(crate) struct EngineRasterFailure(pub(crate) EngineTaskQueue);
impl CpuRasterFailureHandler for EngineRasterFailure {
    fn OnRasterFailure(&self, error: io::Error) {
        self.0.Push(EngineTask::RasterFailure(error));
    }
}

/// Source shared-image calls retain the actual native image returned by the
/// pixel backend. Neither service nor allocator stores a pool/owner backpointer.
pub(crate) struct CpuPoolImages<R: CpuSkiaBackend> {
    pixels: Arc<dyn CompositorPixelProvider<R>>,
    ids: Arc<CpuAllocationIds>,
    failure: Arc<dyn CpuRasterFailureHandler>,
}
impl<R: CpuSkiaBackend> CpuPoolImages<R> {
    pub(crate) fn new(
        pixels: Arc<dyn CompositorPixelProvider<R>>,
        ids: Arc<CpuAllocationIds>,
        failure: Arc<dyn CpuRasterFailureHandler>,
    ) -> Self {
        Self {
            pixels,
            ids,
            failure,
        }
    }
    fn Allocate(
        &self,
        size: crate::adapter::geometry::Size,
        format: &crate::adapter::resource_types::SharedImageFormat,
    ) -> Option<CpuSharedImage> {
        match self.pixels.CreateSharedImage(self.ids.Next(), size, format) {
            Ok(image) => Some(image),
            Err(error) => {
                self.failure.OnRasterFailure(error);
                None
            }
        }
    }
}
impl<R: CpuSkiaBackend> PoolSharedImageInterface<CpuCompositorTypes<R>> for CpuPoolImages<R> {
    fn CreateSharedImage(
        &self,
        descriptor: SharedImageDescriptor<'_, CpuColorSpace>,
        _buffer_usage: Option<CpuBufferUsage>,
    ) -> Option<CpuSharedImage> {
        self.Allocate(*descriptor.size, descriptor.format)
    }
    fn CreateSharedImageForSoftwareCompositor(
        &self,
        descriptor: SharedImageDescriptor<'_, CpuColorSpace>,
    ) -> Option<CpuSharedImage> {
        self.Allocate(*descriptor.size, descriptor.format)
    }
    fn CreateSharedImageForTesting(
        &self,
        descriptor: SharedImageTestDescriptor<'_, CpuColorSpace>,
        _texture_target: u32,
    ) -> Option<CpuSharedImage> {
        self.Allocate(*descriptor.size, descriptor.format)
    }
}

pub(crate) struct CpuSoftwareRasterBufferProvider<R: CpuSkiaBackend> {
    pixels: Arc<dyn CompositorPixelProvider<R>>,
    images: CpuPoolImages<R>,
    failure: Arc<dyn CpuRasterFailureHandler>,
}
impl<R: CpuSkiaBackend> CpuSoftwareRasterBufferProvider<R> {
    pub(crate) fn new(
        pixels: Arc<dyn CompositorPixelProvider<R>>,
        ids: Arc<CpuAllocationIds>,
        failure: Arc<dyn CpuRasterFailureHandler>,
    ) -> Self {
        Self {
            images: CpuPoolImages::new(pixels.clone(), ids, failure.clone()),
            pixels,
            failure,
        }
    }
}
impl<R: CpuSkiaBackend> RasterBufferProvider<CpuCompositorTypes<R>>
    for CpuSoftwareRasterBufferProvider<R>
{
    fn Shutdown(&mut self) {
        if let Err(error) = self.pixels.Shutdown() {
            self.failure.OnRasterFailure(error);
        }
    }
    fn NotifyWorkSubmitted(&mut self) {
        self.pixels.NotifyWorkSubmitted();
    }
    // Source: cc/raster/zero_copy_raster_buffer_provider.cc:197-201.
    // Playback publishes the CPU allocation before the worker task completes;
    // completed zero-copy resources have no deferred GPU work to wait for.
    fn IsResourceReadyToDraw(&self, _resource: &InUsePoolResource<CpuCompositorTypes<R>>) -> bool {
        true
    }

    fn CanPartialRasterIntoProvidedResource(&self) -> bool {
        self.pixels.CanPartialRasterIntoProvidedResource()
    }
    fn AcquireBufferForRaster(
        &mut self,
        resource: &InUsePoolResource<CpuCompositorTypes<R>>,
        pool: &mut ResourcePool<CpuCompositorTypes<R>>,
        resource_content_id: u64,
        previous_content_id: u64,
    ) -> Option<CpuTileRasterBuffer<R>> {
        // Source boundary: cc/raster/zero_copy_raster_buffer_provider.cc:38-55.
        // The CPU software branch installs the real backing before playback,
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
                backing.set_mailbox_sync_token(image.native().creation_sync_token());
            });
        }
        let id = resource.WithBacking(|backing| {
            backing
                .expect("source backing")
                .shared_image()
                .expect("source software shared image")
                .CpuRasterAllocationId()
        });
        match self.pixels.AcquireBufferForRaster(
            id,
            *resource.size(),
            resource.format(),
            resource_content_id,
            previous_content_id,
        ) {
            Ok(buffer) => Some(CpuTileRasterBuffer::FromBoxed(buffer, self.failure.clone())),
            Err(error) => {
                self.failure.OnRasterFailure(error);
                None
            }
        }
    }
}

impl<R: CpuSkiaBackend> CpuSoftwareRasterBufferProvider<R> {
    pub(crate) fn shared_images(&self) -> &CpuPoolImages<R> {
        &self.images
    }
    // Source: cc/raster/zero_copy_raster_buffer_provider.cc:203-209.
    pub(crate) fn SetReadyToDrawCallback(
        &self,
        _resources: &[&InUsePoolResource<CpuCompositorTypes<R>>],
        _callback: crate::adapter::task_runner::OnceClosure,
        _previous: u64,
    ) -> u64 {
        0
    }
}
