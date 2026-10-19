//! Native cc-selected CPU raster execution and immutable resource leases.
//! No layerization, tile-priority, cancellation, graph or compositor state lives
//! here. The only inputs are the canonical RasterPlayback and output allocation.
#![allow(non_snake_case)]
mod canvas;
mod ui;
use compositor_core::{
    adapter::{
        geometry::{AxisTransform2d, Rect, Size},
        layer_tree_impl::ResourceId,
        prioritized_tile::ScrollOffsetMap,
        resource_pool::{ResourcePoolMemoryDump, SharedImageTracingImportance},
        resource_types::{SharedImageFormat, SinglePlaneFormat},
    },
    runtime::{
        cpu_raster::{
            CpuRasterBuffer, CpuRasterBufferProvider, CpuRasterResourceLease,
            CpuRasterResourceProvider, RasterPlayback,
        },
        paint::{CpuImageProviderRef, CpuRasterSource, RasterTransform},
        resources::{
            CommandBufferNamespace, CpuDumpGuid, CpuSharedImage, CpuSyncToken, CpuWriteMapping,
            NativeSharedImage,
        },
        skia::SkRegion,
    },
};
use skia::{src::core::SkCanvas::RasterImageCache, PixelFormat};
use std::{
    collections::HashMap,
    io,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex, Weak,
    },
};

/// Actual immutable pixel allocation, shared without pixel readback/copy.
pub struct CpuPixels {
    resource: ResourceId,
    size: Size,
    format: PixelFormat,
    row_bytes: usize,
    pixels: Vec<u8>,
    content_id: u64,
}
impl CpuRasterResourceLease for CpuPixels {
    type PixelFormat = PixelFormat;
    fn resource_id(&self) -> ResourceId {
        self.resource
    }
    fn size(&self) -> Size {
        self.size
    }
    fn format(&self) -> PixelFormat {
        self.format
    }
    fn row_bytes(&self) -> usize {
        self.row_bytes
    }
    fn pixels(&self) -> &[u8] {
        &self.pixels
    }
}
#[derive(Clone)]
pub struct CpuPixelLease(Arc<CpuPixels>);
impl CpuRasterResourceLease for CpuPixelLease {
    type PixelFormat = PixelFormat;
    fn resource_id(&self) -> ResourceId {
        self.0.resource_id()
    }
    fn size(&self) -> Size {
        self.0.size()
    }
    fn format(&self) -> PixelFormat {
        self.0.format()
    }
    fn row_bytes(&self) -> usize {
        self.0.row_bytes()
    }
    fn pixels(&self) -> &[u8] {
        self.0.pixels()
    }
}
struct AllocationState {
    pixels: Option<Arc<CpuPixels>>,
    rasterizing: bool,
    destruction_token: CpuSyncToken,
}
struct Allocation {
    resource: ResourceId,
    size: Size,
    format: PixelFormat,
    creation: CpuSyncToken,
    completed: AtomicU64,
    release_counter: Arc<AtomicU64>,
    state: Mutex<AllocationState>,
}
/// Exclusive write ownership of the very same native N32 allocation. The
/// Vec is moved out of its sole published CpuPixels owner, never copied. The
/// independent command-buffer counter is an execution capability, not a
/// retained provider/host/tree reference.
struct CpuMappedWrite {
    allocation: Arc<Allocation>,
    pixels: Option<CpuPixels>,
}
impl CpuWriteMapping for CpuMappedWrite {
    fn size(&self) -> Size {
        self.allocation.size
    }
    fn format(&self) -> PixelFormat {
        self.allocation.format
    }
    fn row_bytes(&self) -> usize {
        self.pixels
            .as_ref()
            .expect("live CPU write lease")
            .row_bytes
    }
    fn bytes_mut(&mut self) -> &mut [u8] {
        &mut self.pixels.as_mut().expect("live CPU write lease").pixels
    }
}
impl Drop for CpuMappedWrite {
    fn drop(&mut self) {
        let pixels = self
            .pixels
            .take()
            .expect("CPU write lease published exactly once");
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
    fn MapCpuWrite(self: Arc<Self>) -> io::Result<Box<dyn CpuWriteMapping>> {
        let pixels = {
            let mut state = self
                .state
                .lock()
                .map_err(|_| io::Error::other("CPU allocation lock poisoned"))?;
            if state.rasterizing {
                return Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "CPU allocation already has an exclusive raster/write lease",
                ));
            }
            let mut pixels = match state.pixels.take() {
                Some(pixels) => match Arc::try_unwrap(pixels) {
                    Ok(pixels) => pixels,
                    Err(pixels) => {
                        state.pixels = Some(pixels);
                        return Err(io::Error::new(
                            io::ErrorKind::WouldBlock,
                            "CPU allocation still has a live immutable read lease",
                        ));
                    }
                },
                None => {
                    let len = pixel_len(self.size)?;
                    let mut bytes = Vec::new();
                    bytes.try_reserve_exact(len).map_err(io::Error::other)?;
                    bytes.resize(len, 0);
                    CpuPixels {
                        resource: self.resource,
                        size: self.size,
                        format: self.format,
                        row_bytes: self.size.width() as usize * 4,
                        pixels: bytes,
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
        Ok(Box::new(CpuMappedWrite {
            allocation: self,
            pixels: Some(pixels),
        }))
    }
    fn CpuRasterAllocationId(&self) -> ResourceId {
        self.resource
    }
    fn creation_sync_token(&self) -> CpuSyncToken {
        self.creation
    }
    fn EndImport(&self, _token: &CpuSyncToken) -> CpuSyncToken {
        // CPU writes are complete before publication. The acquired release point
        // belongs to this real in-process allocation and execution sequence.
        let mut token = self.creation;
        token.release_count = self.completed.load(Ordering::Acquire);
        token.verified_flush = true;
        token
    }
    fn UpdateDestructionSyncToken(&self, token: &CpuSyncToken) {
        self.state
            .lock()
            .expect("CPU allocation lock poisoned")
            .destruction_token = *token;
    }
    fn OnMemoryDump(
        &self,
        dump: &mut dyn ResourcePoolMemoryDump<CpuDumpGuid>,
        _guid: &CpuDumpGuid,
        _importance: SharedImageTracingImportance,
    ) {
        let guid = dump.CreateAllocatorDump(&format!("cpu_shared_image/{}", self.resource.0));
        let state = self.state.lock().expect("CPU allocation lock poisoned");
        let bytes = state
            .pixels
            .as_ref()
            .map_or(0, |pixels| pixels.pixels.len());
        dump.AddScalar(&guid, "size", "bytes", bytes as u64);
    }
    fn CpuReadLease(&self) -> Option<Arc<dyn CpuRasterResourceLease<PixelFormat = PixelFormat>>> {
        self.state
            .lock()
            .expect("CPU allocation lock poisoned")
            .pixels
            .as_ref()
            .map(|pixels| {
                Arc::clone(pixels) as Arc<dyn CpuRasterResourceLease<PixelFormat = PixelFormat>>
            })
    }
}
struct ProviderState {
    allocations: Mutex<HashMap<ResourceId, Weak<Allocation>>>,
    image_caches: Mutex<Vec<RasterImageCache>>,
    image_cache_byte_limit: usize,
    next_release: Arc<AtomicU64>,
    next_allocation: AtomicU64,
    command_buffer: u64,
}
struct CpuCacheLease {
    provider: Arc<ProviderState>,
    cache: Option<RasterImageCache>,
}
impl std::ops::Deref for CpuCacheLease {
    type Target = RasterImageCache;
    fn deref(&self) -> &Self::Target {
        self.cache.as_ref().expect("actual cache lease")
    }
}
impl std::ops::DerefMut for CpuCacheLease {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.cache.as_mut().expect("actual cache lease")
    }
}
impl Drop for CpuCacheLease {
    fn drop(&mut self) {
        self.provider
            .image_caches
            .lock()
            .expect("CPU cache pool poisoned")
            .push(self.cache.take().expect("actual cache lease returned once"));
    }
}
#[derive(Clone)]
pub struct CpuRasterProvider(Arc<ProviderState>);
impl CpuRasterProvider {
    /// The caller supplies the identity of this actual CPU execution sequence.
    pub fn new(command_buffer: u64, image_cache_byte_limit: usize) -> Self {
        Self(Arc::new(ProviderState {
            allocations: Mutex::new(HashMap::new()),
            image_caches: Mutex::new(Vec::new()),
            image_cache_byte_limit,
            next_release: Arc::new(AtomicU64::new(1)),
            next_allocation: AtomicU64::new(1),
            command_buffer,
        }))
    }
    fn token(&self) -> CpuSyncToken {
        let mut token = CpuSyncToken::new(
            CommandBufferNamespace::InProcess,
            self.0.command_buffer,
            self.0.next_release.fetch_add(1, Ordering::Relaxed),
        );
        token.verified_flush = true;
        token
    }
    // The external executor may run independent tile buffers concurrently.
    // A cache lease owns its actual Skia cache for playback; the pool lock only
    // transfers that ownership and is never held during CPU rasterization.
    fn CacheForPlayback(&self) -> CpuCacheLease {
        let cache = self
            .0
            .image_caches
            .lock()
            .expect("CPU cache pool poisoned")
            .pop()
            .unwrap_or_else(|| {
                RasterImageCache::with_pixel_byte_limit(self.0.image_cache_byte_limit)
            });
        CpuCacheLease {
            provider: Arc::clone(&self.0),
            cache: Some(cache),
        }
    }
    /// Called by the actual resource allocator, before cc acquires a raster buffer.
    /// The provider retains only a weak index; ResourcePool/shared-image ownership
    /// remains authoritative and dropping the native image frees the allocation.
    pub fn CreateSharedImage(
        &self,
        id: ResourceId,
        size: Size,
        format: &SharedImageFormat,
    ) -> io::Result<CpuSharedImage> {
        if id == ResourceId::default() || size.width() <= 0 || size.height() <= 0 {
            return Err(io::Error::other("invalid native CPU image allocation"));
        }
        let format = pixel_format(format)?;
        let mut allocations = self
            .0
            .allocations
            .lock()
            .expect("CPU allocation index poisoned");
        // Caller-selected tile IDs and internally allocated UI IDs share the
        // same monotonic native namespace and the very same allocation table.
        self.0
            .next_allocation
            .fetch_max(id.0 as u64 + 1, Ordering::Relaxed);
        // Source SharedImage creation returns its backend allocation identity;
        // cc's logical ResourceId is only a preferred number at this native
        // boundary. UI allocations share this table, so an occupied preference
        // obtains a new actual identity. All acquire/export paths consume the
        // returned NativeSharedImage::CpuRasterAllocationId(), never the hint.
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
        format: PixelFormat,
    ) -> io::Result<CpuSharedImage> {
        if allocations.get(&id).and_then(Weak::upgrade).is_some() {
            return Err(io::Error::other(
                "native CPU resource id already owns an allocation",
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
        Ok(CpuSharedImage::new(image))
    }
    fn CreateUiSharedImage(
        &self,
        size: Size,
        format: &SharedImageFormat,
    ) -> io::Result<CpuSharedImage> {
        if size.width() <= 0 || size.height() <= 0 {
            return Err(io::Error::other("invalid native CPU UI image size"));
        }
        let format = pixel_format(format)?;
        let mut allocations = self
            .0
            .allocations
            .lock()
            .map_err(|_| io::Error::other("CPU allocation index poisoned"))?;
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
                .map_err(|_| io::Error::other("native CPU allocation identities exhausted"))?;
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
            .expect("CPU allocation index poisoned")
            .get(&id)
            .and_then(Weak::upgrade)
            .ok_or_else(|| {
                io::Error::other("cc-selected CPU resource has no live native shared image")
            })
    }
    pub fn CompletionToken(&self, id: ResourceId) -> io::Result<CpuSyncToken> {
        let allocation = self.allocation(id)?;
        Ok(allocation.EndImport(&allocation.creation))
    }
    pub fn RasterizePicture(
        &self,
        list: &compositor_core::runtime::paint::ContentDisplayItemList<SkRegion>,
        content: Rect,
        scale: f32,
        nearest: bool,
        animation: &std::collections::BTreeMap<
            compositor_core::adapter::image_animation::PaintImageId,
            usize,
        >,
        scrolls: &ScrollOffsetMap,
    ) -> io::Result<Arc<dyn CpuRasterResourceLease<PixelFormat = PixelFormat>>> {
        let size = content.size();
        let len = pixel_len(size)?;
        let mut cache = self.CacheForPlayback();
        let pixels = canvas::RasterPicture(
            list,
            content,
            scale,
            nearest,
            animation,
            scrolls,
            vec![0; len],
            &mut cache,
        )?;
        Ok(Arc::new(CpuPixels {
            resource: ResourceId::default(),
            size,
            format: PixelFormat::Rgba8888,
            row_bytes: size.width() as usize * 4,
            pixels,
            content_id: 0,
        }))
    }
}
fn pixel_len(size: Size) -> io::Result<usize> {
    if size.width() <= 0 || size.height() <= 0 {
        return Err(io::Error::other("empty CPU raster allocation"));
    }
    (size.width() as usize)
        .checked_mul(size.height() as usize)
        .and_then(|length| length.checked_mul(4))
        .filter(|length| *length <= isize::MAX as usize)
        .ok_or_else(|| io::Error::other("CPU raster allocation overflow"))
}
fn pixel_format(format: &SharedImageFormat) -> io::Result<PixelFormat> {
    if *format == SinglePlaneFormat::RGBA_8888 {
        return Ok(PixelFormat::Rgba8888);
    }
    if *format == SinglePlaneFormat::BGRA_8888 {
        return Ok(PixelFormat::Bgra8888);
    }
    if *format == SinglePlaneFormat::BGRX_8888 {
        return Ok(PixelFormat::Bgrx8888);
    }
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        format!("CPU raster format {:?} is not implemented", format),
    ))
}

pub struct CpuBuffer {
    provider: CpuRasterProvider,
    allocation: Arc<Allocation>,
    previous: Option<CpuPixels>,
    resource_content_id: u64,
    previous_content_id: u64,
    published: bool,
}
impl Drop for CpuBuffer {
    fn drop(&mut self) {
        if !self.published {
            let mut state = self
                .allocation
                .state
                .lock()
                .expect("CPU allocation lock poisoned");
            state.rasterizing = false;
            if let Some(previous) = self.previous.take() {
                state.pixels = Some(Arc::new(previous));
            }
        }
    }
}
impl<'image>
    CpuRasterBuffer<
        CpuRasterSource<SkRegion>,
        RasterTransform,
        CpuImageProviderRef<'image, SkRegion>,
        ScrollOffsetMap,
    > for CpuBuffer
{
    fn SupportsBackgroundThreadPriority(&self) -> bool {
        false
    }
    fn Playback(
        &mut self,
        request: RasterPlayback<
            '_,
            CpuRasterSource<SkRegion>,
            RasterTransform,
            CpuImageProviderRef<'image, SkRegion>,
            ScrollOffsetMap,
        >,
    ) -> io::Result<()> {
        if self.published {
            return Err(io::Error::other(
                "native CPU raster buffer cannot publish twice",
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
        let mut cache = self.provider.CacheForPlayback();
        let new_content_id = request.new_content_id;
        let pixels = canvas::RasterSource(
            RasterPlayback {
                invalid_content_rect: &playback_rect,
                ..request
            },
            self.allocation.size,
            self.allocation.format,
            storage,
            &mut cache,
        )?;
        let pixels = Arc::new(CpuPixels {
            resource: self.allocation.resource,
            size: self.allocation.size,
            format: self.allocation.format,
            row_bytes: self.allocation.size.width() as usize * 4,
            pixels,
            content_id: new_content_id,
        });
        let mut state = self
            .allocation
            .state
            .lock()
            .expect("CPU allocation lock poisoned");
        state.pixels = Some(pixels);
        state.rasterizing = false;
        let point = self.provider.0.next_release.fetch_add(1, Ordering::Relaxed);
        self.allocation.completed.store(point, Ordering::Release);
        self.published = true;
        Ok(())
    }
}
impl<'image>
    CpuRasterBufferProvider<
        CpuRasterSource<SkRegion>,
        RasterTransform,
        CpuImageProviderRef<'image, SkRegion>,
        ScrollOffsetMap,
    > for CpuRasterProvider
{
    type Buffer = CpuBuffer;
    fn AcquireBufferForRaster(
        &mut self,
        id: ResourceId,
        size: Size,
        format: &SharedImageFormat,
        resource_content_id: u64,
        previous_content_id: u64,
    ) -> io::Result<CpuBuffer> {
        let allocation = self.allocation(id)?;
        if allocation.size != size || allocation.format != pixel_format(format)? {
            return Err(io::Error::other(
                "native raster metadata does not match shared image allocation",
            ));
        }
        let previous = {
            let mut state = allocation
                .state
                .lock()
                .expect("CPU allocation lock poisoned");
            if state.rasterizing {
                return Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "CPU resource is already being rastered",
                ));
            }
            let previous = match state.pixels.take() {
                Some(pixels) => match Arc::try_unwrap(pixels) {
                    Ok(pixels) => Some(pixels),
                    Err(pixels) => {
                        state.pixels = Some(pixels);
                        return Err(io::Error::new(
                            io::ErrorKind::WouldBlock,
                            "CPU resource still has a live immutable read lease",
                        ));
                    }
                },
                None => None,
            };
            state.rasterizing = true;
            previous
        };
        Ok(CpuBuffer {
            provider: self.clone(),
            allocation,
            previous,
            resource_content_id,
            previous_content_id,
            published: false,
        })
    }
    fn IsResourceReadyToDraw(&self, id: ResourceId) -> bool {
        self.allocation(id).is_ok_and(|allocation| {
            let state = allocation
                .state
                .lock()
                .expect("CPU allocation lock poisoned");
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
            .expect("CPU allocation index poisoned");
        if allocations
            .values()
            .filter_map(Weak::upgrade)
            .any(|allocation| {
                allocation
                    .state
                    .lock()
                    .expect("CPU allocation lock poisoned")
                    .rasterizing
            })
        {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "CPU raster buffers must finish before shutdown",
            ));
        }
        allocations.retain(|_, allocation| allocation.strong_count() != 0);
        Ok(())
    }
}
impl CpuRasterResourceProvider for CpuRasterProvider {
    type Lease = CpuPixelLease;
    fn ExportResource(&self, id: ResourceId) -> Option<CpuPixelLease> {
        let allocation = self.allocation(id).ok()?;
        let state = allocation
            .state
            .lock()
            .expect("CPU allocation lock poisoned");
        state
            .pixels
            .as_ref()
            .map(|pixels| CpuPixelLease(Arc::clone(pixels)))
    }
}

/// The compositor's only injected pixel capability. Cloning this handle shares
/// the same native allocation/cache owners; it does not clone pixel storage.
/// Canonical cc code continues to choose tasks, resources and completion order.
impl compositor_core::CompositorPixelProvider for CpuRasterProvider {
    fn CreateSharedImage(
        &self,
        resource: ResourceId,
        size: Size,
        format: &SharedImageFormat,
    ) -> io::Result<CpuSharedImage> {
        CpuRasterProvider::CreateSharedImage(self, resource, size, format)
    }
    fn AcquireBufferForRaster(
        &self,
        resource: ResourceId,
        size: Size,
        format: &SharedImageFormat,
        resource_content_id: u64,
        previous_content_id: u64,
    ) -> io::Result<Box<compositor_core::runtime::paint::CpuTileRasterBufferValue<SkRegion>>> {
        let buffer = <Self as CpuRasterBufferProvider<
            CpuRasterSource<SkRegion>,
            RasterTransform,
            CpuImageProviderRef<'_, SkRegion>,
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
    fn IsResourceReadyToDraw(&self, resource: ResourceId) -> bool {
        <Self as CpuRasterBufferProvider<
            CpuRasterSource<SkRegion>,
            RasterTransform,
            CpuImageProviderRef<'_, SkRegion>,
            ScrollOffsetMap,
        >>::IsResourceReadyToDraw(self, resource)
    }
    fn CanPartialRasterIntoProvidedResource(&self) -> bool {
        <Self as CpuRasterBufferProvider<
            CpuRasterSource<SkRegion>,
            RasterTransform,
            CpuImageProviderRef<'_, SkRegion>,
            ScrollOffsetMap,
        >>::CanPartialRasterIntoProvidedResource(self)
    }
    fn CompletionToken(&self, resource: ResourceId) -> io::Result<CpuSyncToken> {
        CpuRasterProvider::CompletionToken(self, resource)
    }
    fn NotifyWorkSubmitted(&self) {
        <Self as CpuRasterBufferProvider<
            CpuRasterSource<SkRegion>,
            RasterTransform,
            CpuImageProviderRef<'_, SkRegion>,
            ScrollOffsetMap,
        >>::NotifyWorkSubmitted(&mut self.clone());
    }
    fn Shutdown(&self) -> io::Result<()> {
        <Self as CpuRasterBufferProvider<
            CpuRasterSource<SkRegion>,
            RasterTransform,
            CpuImageProviderRef<'_, SkRegion>,
            ScrollOffsetMap,
        >>::Shutdown(&mut self.clone())
    }
}

mod solid;
pub use solid::{CpuPaintAnalysisTrace, CpuSolidColorBackend};
