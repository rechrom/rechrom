//! Native cc-selected raster execution and immutable resource leases.
//! No layerization, tile-priority, cancellation, graph or compositor state lives
//! here. The only inputs are the canonical RasterPlayback and output allocation.
#![allow(non_snake_case)]
mod backend;
mod translated;
pub use translated::{BuildTranslatedFilterOperations, ReplayDisplayItemListIntoTranslatedTarget};
#[cfg(any(test, feature = "artifact_test_support"))]
pub use translated::{
    RasterizePaintArtifact, RasterizePaintArtifactTransparent, RasterizePaintArtifactWithScale,
};
mod ui_translated;
use backend::{RasterBackend, RasterBackendTarget};
use compositor_core::{
    adapter::{
        geometry::{AxisTransform2d, Size},
        layer_tree_impl::ResourceId,
        prioritized_tile::ScrollOffsetMap,
        resource_pool::{ResourcePoolMemoryDump, SharedImageTracingImportance},
        resource_types::{SharedImageFormat, SinglePlaneFormat},
    },
    runtime::{
        paint::{NativeRasterSource, RasterImageProviderRef, RasterTransform},
        raster_execution::{
            RasterBuffer, RasterBufferProviderBackend, RasterPlayback, RasterResourceLease,
        },
        resources::{
            CommandBufferNamespace, NativeSharedImage, RasterDumpGuid, RasterSharedImage,
            RasterSyncToken, RasterWriteMapping,
        },
        skia::SkRegion,
    },
};
use std::{
    collections::HashMap,
    io,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex, Weak,
    },
};

/// Current backend's immutable host-memory resource. This representation is
/// private to `raster`; compositor and Viz retain only `RasterResourceLease`.
struct HostMemoryResource {
    resource: ResourceId,
    size: Size,
    format: SharedImageFormat,
    row_bytes: usize,
    pixels: Vec<u8>,
    opaque: bool,
    content_id: u64,
}
impl RasterResourceLease for HostMemoryResource {
    fn resource_id(&self) -> ResourceId {
        self.resource
    }
    fn size(&self) -> Size {
        self.size
    }
    fn format(&self) -> SharedImageFormat {
        self.format
    }
    fn is_opaque(&self) -> bool {
        self.opaque
    }
    fn backend_resource(&self) -> &(dyn std::any::Any + Send + Sync) {
        self
    }
}

/// Call-scoped mapping returned to a renderer that needs host-readable bytes.
/// The compositor and Viz never see this value. A future backend may provide a
/// different mapping implementation without changing their contracts.
pub struct RasterReadView<'a> {
    size: Size,
    format: SharedImageFormat,
    row_bytes: usize,
    bytes: &'a [u8],
}
impl RasterReadView<'_> {
    pub fn size(&self) -> Size {
        self.size
    }
    pub fn format(&self) -> SharedImageFormat {
        self.format
    }
    pub fn row_bytes(&self) -> usize {
        self.row_bytes
    }
    pub fn bytes(&self) -> &[u8] {
        self.bytes
    }
}

/// Maps an opaque raster resource for the current host-readable renderer.
/// Backend representation stays inside this crate; failure means the selected
/// renderer and resource backend are incompatible.
pub fn MapResourceForRead(resource: &dyn RasterResourceLease) -> io::Result<RasterReadView<'_>> {
    let resource = resource
        .backend_resource()
        .downcast_ref::<HostMemoryResource>()
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Unsupported,
                "raster resource is not host-readable",
            )
        })?;
    Ok(RasterReadView {
        size: resource.size,
        format: resource.format,
        row_bytes: resource.row_bytes,
        bytes: &resource.pixels,
    })
}
struct AllocationState {
    pixels: Option<Arc<HostMemoryResource>>,
    rasterizing: bool,
    destruction_token: RasterSyncToken,
}
struct Allocation {
    resource: ResourceId,
    size: Size,
    format: SharedImageFormat,
    creation: RasterSyncToken,
    completed: AtomicU64,
    release_counter: Arc<AtomicU64>,
    state: Mutex<AllocationState>,
}
/// Exclusive write ownership of the very same native N32 allocation. The
/// Vec is moved out of its sole published host-memory owner, never copied. The
/// independent command-buffer counter is an execution capability, not a
/// retained provider/host/tree reference.
struct HostMemoryWrite {
    allocation: Arc<Allocation>,
    pixels: Option<HostMemoryResource>,
}
impl RasterWriteMapping for HostMemoryWrite {
    fn size(&self) -> Size {
        self.allocation.size
    }
    fn format(&self) -> SharedImageFormat {
        self.allocation.format
    }
    fn row_bytes(&self) -> usize {
        self.pixels
            .as_ref()
            .expect("live raster write lease")
            .row_bytes
    }
    fn bytes_mut(&mut self) -> &mut [u8] {
        &mut self
            .pixels
            .as_mut()
            .expect("live raster write lease")
            .pixels
    }
}
impl Drop for HostMemoryWrite {
    fn drop(&mut self) {
        let mut pixels = self
            .pixels
            .take()
            .expect("raster write lease published exactly once");
        pixels.opaque = pixels_opaque(&pixels.pixels);
        let mut state = self
            .allocation
            .state
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        debug_assert!(state.rasterizing && state.pixels.is_none());
        state.pixels = Some(Arc::new(pixels));
        let point = self
            .allocation
            .release_counter
            .fetch_add(1, Ordering::Relaxed);
        self.allocation.completed.store(point, Ordering::Release);
        state.rasterizing = false;
    }
}
impl NativeSharedImage for Allocation {
    fn MapRasterWrite(self: Arc<Self>) -> io::Result<Box<dyn RasterWriteMapping>> {
        let pixels = {
            let mut state = self
                .state
                .lock()
                .map_err(|_| io::Error::other("raster allocation lock poisoned"))?;
            if state.rasterizing {
                return Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "raster allocation already has an exclusive raster/write lease",
                ));
            }
            let mut pixels = match state.pixels.take() {
                Some(pixels) => match Arc::try_unwrap(pixels) {
                    Ok(pixels) => pixels,
                    Err(pixels) => {
                        state.pixels = Some(pixels);
                        return Err(io::Error::new(
                            io::ErrorKind::WouldBlock,
                            "raster allocation still has a live immutable read lease",
                        ));
                    }
                },
                None => {
                    let len = pixel_len(self.size)?;
                    let mut bytes = Vec::new();
                    bytes.try_reserve_exact(len).map_err(io::Error::other)?;
                    bytes.resize(len, 0);
                    HostMemoryResource {
                        resource: self.resource,
                        size: self.size,
                        format: self.format,
                        row_bytes: self.size.width() as usize * 4,
                        pixels: bytes,
                        opaque: false,
                        content_id: 0,
                    }
                }
            };
            // External writes do not correspond to a Tile's previous raster
            // content identity. A later raster cannot claim partial reuse of it.
            pixels.content_id = 0;
            state.rasterizing = true;
            pixels
        };
        Ok(Box::new(HostMemoryWrite {
            allocation: self,
            pixels: Some(pixels),
        }))
    }
    fn RasterAllocationId(&self) -> ResourceId {
        self.resource
    }
    fn creation_sync_token(&self) -> RasterSyncToken {
        self.creation
    }
    fn EndImport(&self, _token: &RasterSyncToken) -> RasterSyncToken {
        // raster writes are complete before publication. The acquired release point
        // belongs to this real in-process allocation and execution sequence.
        let mut token = self.creation;
        token.release_count = self.completed.load(Ordering::Acquire);
        token.verified_flush = true;
        token
    }
    fn UpdateDestructionSyncToken(&self, token: &RasterSyncToken) {
        self.state
            .lock()
            .expect("raster allocation lock poisoned")
            .destruction_token = *token;
    }
    fn OnMemoryDump(
        &self,
        dump: &mut dyn ResourcePoolMemoryDump<RasterDumpGuid>,
        _guid: &RasterDumpGuid,
        _importance: SharedImageTracingImportance,
    ) {
        let guid = dump.CreateAllocatorDump(&format!("cpu_shared_image/{}", self.resource.0));
        let state = self.state.lock().expect("raster allocation lock poisoned");
        let bytes = state
            .pixels
            .as_ref()
            .map_or(0, |pixels| pixels.pixels.len());
        dump.AddScalar(&guid, "size", "bytes", bytes as u64);
    }
    fn RasterReadLease(&self) -> Option<Arc<dyn RasterResourceLease>> {
        self.state
            .lock()
            .expect("raster allocation lock poisoned")
            .pixels
            .as_ref()
            .map(|pixels| Arc::clone(pixels) as Arc<dyn RasterResourceLease>)
    }
}
struct ProviderState {
    allocations: Mutex<HashMap<ResourceId, Weak<Allocation>>>,
    backend: Arc<dyn RasterBackend>,
    next_release: Arc<AtomicU64>,
    next_allocation: AtomicU64,
    command_buffer: u64,
}
/// Raster package service boundary. It owns raster allocation identity, replay
/// admission, backend caches and immutable publication. Compositor owns tile
/// selection and scheduling; the selected Skia implementation remains private.
#[derive(Clone)]
pub struct RasterEngine(Arc<ProviderState>);

impl RasterEngine {
    /// The caller supplies the identity of this actual raster execution sequence.
    pub fn new(command_buffer: u64, image_cache_byte_limit: usize) -> Self {
        let backend: Arc<dyn RasterBackend> = Arc::new(translated::TranslatedSkiaBackend::new(
            image_cache_byte_limit,
        ));
        Self(Arc::new(ProviderState {
            allocations: Mutex::new(HashMap::new()),
            backend,
            next_release: Arc::new(AtomicU64::new(1)),
            next_allocation: AtomicU64::new(1),
            command_buffer,
        }))
    }
    fn token(&self) -> RasterSyncToken {
        let mut token = RasterSyncToken::new(
            CommandBufferNamespace::InProcess,
            self.0.command_buffer,
            self.0.next_release.fetch_add(1, Ordering::Relaxed),
        );
        token.verified_flush = true;
        token
    }
    /// Called by the actual resource allocator, before cc acquires a raster buffer.
    /// The provider retains only a weak index; ResourcePool/shared-image ownership
    /// remains authoritative and dropping the native image frees the allocation.
    pub fn CreateSharedImage(
        &self,
        id: ResourceId,
        size: Size,
        format: &SharedImageFormat,
    ) -> io::Result<RasterSharedImage> {
        if id == ResourceId::default() || size.width() <= 0 || size.height() <= 0 {
            return Err(io::Error::other("invalid native raster image allocation"));
        }
        validate_pixel_format(format)?;
        let format = *format;
        let mut allocations = self
            .0
            .allocations
            .lock()
            .expect("raster allocation index poisoned");
        // Caller-selected tile IDs and internally allocated UI IDs share the
        // same monotonic native namespace and the very same allocation table.
        self.0
            .next_allocation
            .fetch_max(id.0 as u64 + 1, Ordering::Relaxed);
        // Source SharedImage creation returns its backend allocation identity;
        // cc's logical ResourceId is only a preferred number at this native
        // boundary. UI allocations share this table, so an occupied preference
        // obtains a new actual identity. All acquire/export paths consume the
        // returned NativeSharedImage::RasterAllocationId(), never the hint.
        let id = if allocations
            .get(&id)
            .is_some_and(|value| value.strong_count() != 0)
        {
            self.NextAllocationLocked(&allocations)?
        } else {
            id
        };
        self.CreateSharedImageLocked(&mut allocations, id, size, format)
    }
    fn CreateSharedImageLocked(
        &self,
        allocations: &mut HashMap<ResourceId, Weak<Allocation>>,
        id: ResourceId,
        size: Size,
        format: SharedImageFormat,
    ) -> io::Result<RasterSharedImage> {
        if allocations.get(&id).and_then(Weak::upgrade).is_some() {
            return Err(io::Error::other(
                "native raster resource id already owns an allocation",
            ));
        }
        allocations.retain(|_, allocation| allocation.strong_count() != 0);
        let creation = self.token();
        let image = Arc::new(Allocation {
            resource: id,
            size,
            format,
            creation,
            completed: AtomicU64::new(creation.release_count),
            release_counter: Arc::clone(&self.0.next_release),
            state: Mutex::new(AllocationState {
                pixels: None,
                rasterizing: false,
                destruction_token: creation,
            }),
        });
        allocations.insert(id, Arc::downgrade(&image));
        Ok(RasterSharedImage::new(image))
    }
    fn CreateUiSharedImage(
        &self,
        size: Size,
        format: &SharedImageFormat,
    ) -> io::Result<RasterSharedImage> {
        if size.width() <= 0 || size.height() <= 0 {
            return Err(io::Error::other("invalid native raster UI image size"));
        }
        validate_pixel_format(format)?;
        let format = *format;
        let mut allocations = self
            .0
            .allocations
            .lock()
            .map_err(|_| io::Error::other("raster allocation index poisoned"))?;
        let id = self.NextAllocationLocked(&allocations)?;
        self.CreateSharedImageLocked(&mut allocations, id, size, format)
    }
    fn NextAllocationLocked(
        &self,
        allocations: &HashMap<ResourceId, Weak<Allocation>>,
    ) -> io::Result<ResourceId> {
        loop {
            let next = self.0.next_allocation.fetch_add(1, Ordering::Relaxed);
            let next = u32::try_from(next)
                .map_err(|_| io::Error::other("native raster allocation identities exhausted"))?;
            let id = ResourceId(next);
            if !allocations
                .get(&id)
                .is_some_and(|value| value.strong_count() != 0)
            {
                return Ok(id);
            }
        }
    }
    fn allocation(&self, id: ResourceId) -> io::Result<Arc<Allocation>> {
        self.0
            .allocations
            .lock()
            .expect("raster allocation index poisoned")
            .get(&id)
            .and_then(Weak::upgrade)
            .ok_or_else(|| {
                io::Error::other("cc-selected raster resource has no live native shared image")
            })
    }
}
fn pixels_opaque(pixels: &[u8]) -> bool {
    pixels.chunks_exact(4).all(|pixel| pixel[3] == u8::MAX)
}
fn pixel_len(size: Size) -> io::Result<usize> {
    if size.width() <= 0 || size.height() <= 0 {
        return Err(io::Error::other("empty raster allocation"));
    }
    (size.width() as usize)
        .checked_mul(size.height() as usize)
        .and_then(|length| length.checked_mul(4))
        .filter(|length| *length <= isize::MAX as usize)
        .ok_or_else(|| io::Error::other("raster allocation overflow"))
}
fn validate_pixel_format(format: &SharedImageFormat) -> io::Result<()> {
    if *format == SinglePlaneFormat::RGBA_8888
        || *format == SinglePlaneFormat::BGRA_8888
        || *format == SinglePlaneFormat::BGRX_8888
    {
        return Ok(());
    }
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        format!("raster format {:?} is not implemented", format),
    ))
}

pub struct RasterBufferSession {
    provider: RasterEngine,
    allocation: Arc<Allocation>,
    previous: Option<HostMemoryResource>,
    resource_content_id: u64,
    previous_content_id: u64,
    finished: bool,
}
impl Drop for RasterBufferSession {
    fn drop(&mut self) {
        // Dropping an acquired buffer before playback preserves its previous
        // publication. Once playback starts, success publishes new pixels and
        // failure explicitly invalidates the resource before returning.
        if !self.finished {
            let mut state = self
                .allocation
                .state
                .lock()
                .expect("raster allocation lock poisoned");
            state.rasterizing = false;
            if let Some(previous) = self.previous.take() {
                state.pixels = Some(Arc::new(previous));
            }
        }
    }
}
impl RasterBufferSession {
    /// Chromium's zero-copy provider drops a SharedImage that cannot be mapped
    /// and lets the tile checkerboard. Backend replay errors have the same
    /// publication semantics here: after write access begins, partially
    /// modified storage must never be republished as either the old or new
    /// content identity.
    fn FinishFailedPlayback(&mut self, error: io::Error) -> io::Result<()> {
        let mut state = self
            .allocation
            .state
            .lock()
            .expect("raster allocation lock poisoned");
        debug_assert!(state.rasterizing && state.pixels.is_none());
        state.pixels = None;
        state.rasterizing = false;
        self.finished = true;
        Err(error)
    }
}
impl<'image>
    RasterBuffer<
        NativeRasterSource<SkRegion>,
        RasterTransform,
        RasterImageProviderRef<'image, SkRegion>,
        ScrollOffsetMap,
    > for RasterBufferSession
{
    fn SupportsBackgroundThreadPriority(&self) -> bool {
        false
    }
    fn Playback(
        &mut self,
        request: RasterPlayback<
            '_,
            NativeRasterSource<SkRegion>,
            RasterTransform,
            RasterImageProviderRef<'image, SkRegion>,
            ScrollOffsetMap,
        >,
    ) -> io::Result<()> {
        if self.finished {
            return Err(io::Error::other(
                "native raster buffer cannot finish playback twice",
            ));
        }
        // Source zero-copy raster providers allocate a fixed resource size.
        // Edge tiles may cover a smaller content rect; RasterSource clips and
        // translates that rect into the full allocation below.
        let mut previous = self.previous.take();
        // Source: cc/raster/zero_copy_raster_buffer_provider.cc:108-123;
        // gpu_raster_buffer_provider.cc:253-258. A newly assigned resource
        // always receives full raster even when the tile's dirty rect is small.
        let has_previous =
            self.resource_content_id != 0 && self.resource_content_id == self.previous_content_id;
        if has_previous
            && previous
                .as_ref()
                .is_none_or(|pixels| pixels.content_id != self.previous_content_id)
        {
            self.previous = previous;
            return Err(io::Error::other(
                "native preserved resource content identity does not match its actual pixels",
            ));
        }
        let mut playback_rect = *request.content_rect;
        if has_previous {
            playback_rect.Intersect(*request.invalid_content_rect);
        }
        if playback_rect.IsEmpty() {
            self.previous = previous;
            return Err(io::Error::other("native raster task has no dirty pixels"));
        }
        let storage = previous.take().map_or_else(
            || pixel_len(self.allocation.size).map(|len| vec![0; len]),
            |pixels| Ok(pixels.pixels),
        )?;
        let new_content_id = request.new_content_id;
        let mut target =
            RasterBackendTarget::new(self.allocation.size, self.allocation.format, storage)?;
        if let Err(error) = self.provider.0.backend.Raster(
            RasterPlayback {
                invalid_content_rect: &playback_rect,
                ..request
            },
            &mut target,
        ) {
            return self.FinishFailedPlayback(error);
        }
        let pixels = match target.into_pixels() {
            Ok(pixels) => pixels,
            Err(error) => return self.FinishFailedPlayback(error),
        };
        let opaque = pixels_opaque(&pixels);
        let pixels = Arc::new(HostMemoryResource {
            resource: self.allocation.resource,
            size: self.allocation.size,
            format: self.allocation.format,
            row_bytes: self.allocation.size.width() as usize * 4,
            pixels,
            opaque,
            content_id: new_content_id,
        });
        let mut state = self
            .allocation
            .state
            .lock()
            .expect("raster allocation lock poisoned");
        state.pixels = Some(pixels);
        state.rasterizing = false;
        let point = self.provider.0.next_release.fetch_add(1, Ordering::Relaxed);
        self.allocation.completed.store(point, Ordering::Release);
        self.finished = true;
        Ok(())
    }
}
impl<'image>
    RasterBufferProviderBackend<
        NativeRasterSource<SkRegion>,
        RasterTransform,
        RasterImageProviderRef<'image, SkRegion>,
        ScrollOffsetMap,
    > for RasterEngine
{
    type Buffer = RasterBufferSession;
    fn AcquireBufferForRaster(
        &mut self,
        id: ResourceId,
        size: Size,
        format: &SharedImageFormat,
        resource_content_id: u64,
        previous_content_id: u64,
    ) -> io::Result<RasterBufferSession> {
        let allocation = self.allocation(id)?;
        validate_pixel_format(format)?;
        if allocation.size != size || allocation.format != *format {
            return Err(io::Error::other(
                "native raster metadata does not match shared image allocation",
            ));
        }
        let previous = {
            let mut state = allocation
                .state
                .lock()
                .expect("raster allocation lock poisoned");
            if state.rasterizing {
                return Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "raster resource is already being rastered",
                ));
            }
            let previous = match state.pixels.take() {
                Some(pixels) => match Arc::try_unwrap(pixels) {
                    Ok(pixels) => Some(pixels),
                    Err(pixels) => {
                        state.pixels = Some(pixels);
                        return Err(io::Error::new(
                            io::ErrorKind::WouldBlock,
                            "raster resource still has a live immutable read lease",
                        ));
                    }
                },
                None => None,
            };
            state.rasterizing = true;
            previous
        };
        Ok(RasterBufferSession {
            provider: self.clone(),
            allocation,
            previous,
            resource_content_id,
            previous_content_id,
            finished: false,
        })
    }
    fn IsResourceReadyToDraw(&self, id: ResourceId) -> bool {
        self.allocation(id).is_ok_and(|allocation| {
            let state = allocation
                .state
                .lock()
                .expect("raster allocation lock poisoned");
            !state.rasterizing && state.pixels.is_some()
        })
    }
    fn CanPartialRasterIntoProvidedResource(&self) -> bool {
        true
    }
    fn NotifyWorkSubmitted(&mut self) {}
    fn Shutdown(&mut self) -> io::Result<()> {
        let mut allocations = self
            .0
            .allocations
            .lock()
            .expect("raster allocation index poisoned");
        if allocations
            .values()
            .filter_map(Weak::upgrade)
            .any(|allocation| {
                allocation
                    .state
                    .lock()
                    .expect("raster allocation lock poisoned")
                    .rasterizing
            })
        {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "raster buffers must finish before shutdown",
            ));
        }
        allocations.retain(|_, allocation| allocation.strong_count() != 0);
        Ok(())
    }
}
/// The compositor's only injected pixel capability. Cloning this handle shares
/// the same native allocation/cache owners; it does not clone pixel storage.
/// Canonical cc code continues to choose tasks, resources and completion order.
impl compositor_core::CompositorRasterProvider for RasterEngine {
    fn CreateSharedImage(
        &self,
        resource: ResourceId,
        size: Size,
        format: &SharedImageFormat,
    ) -> io::Result<RasterSharedImage> {
        RasterEngine::CreateSharedImage(self, resource, size, format)
    }
    fn AcquireBufferForRaster(
        &self,
        resource: ResourceId,
        size: Size,
        format: &SharedImageFormat,
        resource_content_id: u64,
        previous_content_id: u64,
    ) -> io::Result<Box<compositor_core::runtime::paint::TileRasterBufferValue>> {
        let buffer = <Self as RasterBufferProviderBackend<
            NativeRasterSource<SkRegion>,
            RasterTransform,
            RasterImageProviderRef<'_, SkRegion>,
            ScrollOffsetMap,
        >>::AcquireBufferForRaster(
            &mut self.clone(),
            resource,
            size,
            format,
            resource_content_id,
            previous_content_id,
        )?;
        Ok(Box::new(buffer))
    }
    fn CanPartialRasterIntoProvidedResource(&self) -> bool {
        <Self as RasterBufferProviderBackend<
            NativeRasterSource<SkRegion>,
            RasterTransform,
            RasterImageProviderRef<'_, SkRegion>,
            ScrollOffsetMap,
        >>::CanPartialRasterIntoProvidedResource(self)
    }
    fn NotifyWorkSubmitted(&self) {
        <Self as RasterBufferProviderBackend<
            NativeRasterSource<SkRegion>,
            RasterTransform,
            RasterImageProviderRef<'_, SkRegion>,
            ScrollOffsetMap,
        >>::NotifyWorkSubmitted(&mut self.clone());
    }
    fn Shutdown(&self) -> io::Result<()> {
        <Self as RasterBufferProviderBackend<
            NativeRasterSource<SkRegion>,
            RasterTransform,
            RasterImageProviderRef<'_, SkRegion>,
            ScrollOffsetMap,
        >>::Shutdown(&mut self.clone())
    }
}
