//! Cc-selected raster execution and immutable resource leases.
//! No layerization, tile-priority, cancellation, graph or compositor state lives
//! here. The only inputs are the canonical RasterPlayback and output allocation.
#![allow(non_snake_case)]
mod backend;
mod image_operations;
mod translated;
#[cfg(target_vendor = "apple")]
pub use translated::SkiaFontBackendFactory;
pub use translated::{
    BuildTranslatedFilterOperations, Canvas2DRasterBackendFactory,
    ImageBitmapRespectingOrientation, RasterizeDocumentDragImage, RasterizeDocumentImageBitmap,
    ReplayDisplayItemListIntoTranslatedTarget, ResizeAndOrientDragImage, ResizeDragImageBitmap,
    ScaleCachedDragImage, ScaleDocumentDragImage,
};
#[cfg(any(test, feature = "artifact_test_support"))]
pub use translated::{
    RasterizePaintArtifact, RasterizePaintArtifactTransparent, RasterizePaintArtifactWithScale,
};
mod ui_translated;
use backend::{RasterBackend, RasterBackendTarget};
use compositor_core::{
    adapter::{
        geometry::{AxisTransform2d, Rect, Size},
        layer_tree_impl::ResourceId,
        picture_layer_tiling::TilingRasterSource,
        prioritized_tile::ScrollOffsetMap,
        resource_pool::{ResourcePoolMemoryDump, SharedImageTracingImportance},
        resource_types::{
            ChannelFormat, PlaneConfig, SharedImageFormat, SinglePlaneFormat, Subsampling,
        },
    },
    runtime::{
        paint::{RasterImageProviderRef, RasterSourceStorage, RasterTransform},
        raster_execution::{
            RasterBuffer, RasterBufferProviderBackend, RasterPlayback, RasterResourceLease,
        },
        resources::{
            ColorMatrixId, ColorPrimaryId, ColorRangeId, ColorTransferId, CommandBufferNamespace,
            RasterColorSpace, RasterDumpGuid, RasterResourceSource, RasterSharedImage,
            RasterSyncToken, RasterSynchronizationType, RasterTransferableResource,
            RasterWriteMapping, SharedImageBackend,
        },
        skia::SkRegion,
    },
};
use std::{
    collections::{HashMap, VecDeque},
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
    color_space: RasterColorSpace,
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
    fn color_space(&self) -> RasterColorSpace {
        self.color_space.clone()
    }
    fn backend_resource(&self) -> &(dyn std::any::Any + Send + Sync) {
        self
    }
}

/// Immutable decoder-owned multiplanar lease. Unlike tile allocations this is
/// never acquired for raster writes: VideoFrameSubmitter replaces the whole
/// resource when a new frame is published.
struct HostVideoResource {
    resource: ResourceId,
    size: Size,
    format: SharedImageFormat,
    color_space: RasterColorSpace,
    y_stride: usize,
    uv_stride: usize,
    y_plane: Arc<Vec<u8>>,
    uv_plane: Arc<Vec<u8>>,
}
impl RasterResourceLease for HostVideoResource {
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
        true
    }
    fn color_space(&self) -> RasterColorSpace {
        self.color_space.clone()
    }
    fn backend_resource(&self) -> &(dyn std::any::Any + Send + Sync) {
        self
    }
}

pub struct Nv12ReadView<'a> {
    pub size: Size,
    pub y_stride: usize,
    pub uv_stride: usize,
    pub y_plane: &'a [u8],
    pub uv_plane: &'a [u8],
    pub color_space: RasterColorSpace,
}

pub fn MapVideoResourceForRead(resource: &dyn RasterResourceLease) -> io::Result<Nv12ReadView<'_>> {
    let resource = resource
        .backend_resource()
        .downcast_ref::<HostVideoResource>()
        .ok_or_else(|| io::Error::other("raster resource is not a host-readable NV12 frame"))?;
    Ok(Nv12ReadView {
        size: resource.size,
        y_stride: resource.y_stride,
        uv_stride: resource.uv_stride,
        y_plane: resource.y_plane.as_slice(),
        uv_plane: resource.uv_plane.as_slice(),
        color_space: resource.color_space.clone(),
    })
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
    color_space: RasterColorSpace,
    creation: RasterSyncToken,
    completed: AtomicU64,
    release_counter: Arc<AtomicU64>,
    state: Mutex<AllocationState>,
}

struct VideoAllocation {
    resource: ResourceId,
    creation: RasterSyncToken,
    destruction_token: Mutex<RasterSyncToken>,
    lease: Arc<HostVideoResource>,
}

impl SharedImageBackend for VideoAllocation {
    fn RasterAllocationId(&self) -> ResourceId {
        self.resource
    }
    fn creation_sync_token(&self) -> RasterSyncToken {
        self.creation
    }
    fn EndImport(&self, _token: &RasterSyncToken) -> RasterSyncToken {
        self.creation
    }
    fn UpdateDestructionSyncToken(&self, token: &RasterSyncToken) {
        *self
            .destruction_token
            .lock()
            .expect("video allocation lock poisoned") = *token;
    }
    fn OnMemoryDump(
        &self,
        dump: &mut dyn ResourcePoolMemoryDump<RasterDumpGuid>,
        _guid: &RasterDumpGuid,
        _importance: SharedImageTracingImportance,
    ) {
        let guid = dump.CreateAllocatorDump(&format!("cpu_video_image/{}", self.resource.0));
        dump.AddScalar(
            &guid,
            "size",
            "bytes",
            (self.lease.y_plane.len() + self.lease.uv_plane.len()) as u64,
        );
    }
    fn RasterReadLease(&self) -> Option<Arc<dyn RasterResourceLease>> {
        Some(self.lease.clone())
    }
    fn MapRasterWrite(self: Arc<Self>) -> io::Result<Box<dyn RasterWriteMapping>> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "immutable video resources cannot be mapped for raster writes",
        ))
    }
}
/// Exclusive write ownership of the very same N32 allocation. The
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
impl SharedImageBackend for Allocation {
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
                        color_space: self.color_space.clone(),
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
struct CachedContentSnapshot {
    sequence: u64,
    pixels: Arc<HostMemoryResource>,
}

struct ContentSnapshotCache {
    entries: HashMap<u64, CachedContentSnapshot>,
    insertion_order: VecDeque<(u64, u64)>,
    bytes: usize,
    byte_limit: usize,
    next_sequence: u64,
}
impl ContentSnapshotCache {
    fn new(byte_limit: usize) -> Self {
        Self {
            entries: HashMap::new(),
            insertion_order: VecDeque::new(),
            bytes: 0,
            // Production supplies 64 MiB. Even a zero limit retains the newest
            // single snapshot so one invalidation can still consume it.
            byte_limit,
            next_sequence: 1,
        }
    }

    fn Insert(&mut self, content_id: u64, pixels: Arc<HostMemoryResource>) {
        if let Some(previous) = self.entries.remove(&content_id) {
            self.bytes = self.bytes.saturating_sub(previous.pixels.pixels.len());
        }
        let sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.wrapping_add(1).max(1);
        self.bytes = self.bytes.saturating_add(pixels.pixels.len());
        self.entries
            .insert(content_id, CachedContentSnapshot { sequence, pixels });
        self.insertion_order.push_back((content_id, sequence));
        while self.bytes > self.byte_limit && self.entries.len() > 1 {
            let Some((old_id, old_sequence)) = self.insertion_order.pop_front() else {
                break;
            };
            if self
                .entries
                .get(&old_id)
                .is_some_and(|entry| entry.sequence == old_sequence)
            {
                let old = self.entries.remove(&old_id).expect("entry checked");
                self.bytes = self.bytes.saturating_sub(old.pixels.pixels.len());
            }
        }
    }

    fn Take(&mut self, content_id: u64) -> Option<Arc<HostMemoryResource>> {
        let snapshot = self.entries.remove(&content_id)?;
        self.bytes = self.bytes.saturating_sub(snapshot.pixels.pixels.len());
        self.insertion_order.retain(|(queued_id, sequence)| {
            *queued_id != content_id || *sequence != snapshot.sequence
        });
        Some(snapshot.pixels)
    }

    fn Owns(&self, content_id: u64, pixels: &Arc<HostMemoryResource>) -> bool {
        self.entries
            .get(&content_id)
            .is_some_and(|snapshot| Arc::ptr_eq(&snapshot.pixels, pixels))
    }
}

struct ProviderState {
    allocations: Mutex<HashMap<ResourceId, Weak<Allocation>>>,
    // A content id names the immutable pixels produced for one cc Tile. A
    // bounded FIFO keeps enough recent content for copy-on-write partial
    // raster when Viz is still reading the old allocation. Taking a snapshot
    // consumes it, and the byte limit prevents offscreen tile history leaks.
    content_snapshots: Mutex<ContentSnapshotCache>,
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
    /// Synchronous source drag-image replay uses this Page's actual backend,
    /// without exposing its Canvas, cache or allocation owner to Browser.
    pub fn RasterizeLinkDragImage(
        &self,
        size: (i32, i32),
        scale: f32,
        text: &[((f32, f32), font_engine::plain_text_painter::PlainTextPaint)],
    ) -> io::Result<Option<image_resource::ImmutableImageBitmap>> {
        self.0.backend.RasterLinkDragImage(size, scale, text)
    }
    pub fn RasterizeDragImageRecord(
        &self,
        record: &compositor_core::engine::ImmutablePaintRecord,
        layout_size: (f64, f64),
        paint_offset: (f64, f64),
        scale: f32,
        opacity: f32,
        snapshots: &std::collections::BTreeMap<
            u32,
            canvas::child_paint_record::CanvasSnapshotReplacement,
        >,
    ) -> io::Result<Option<image_resource::ImmutableImageBitmap>> {
        self.0
            .backend
            .RasterDragImage(record, layout_size, paint_offset, scale, opacity, snapshots)
    }

    /// The caller supplies the identity of this actual raster execution sequence.
    pub fn new(command_buffer: u64, image_cache_byte_limit: usize) -> Self {
        let backend: Arc<dyn RasterBackend> = Arc::new(translated::TranslatedSkiaBackend::new(
            image_cache_byte_limit,
        ));
        Self(Arc::new(ProviderState {
            allocations: Mutex::new(HashMap::new()),
            content_snapshots: Mutex::new(ContentSnapshotCache::new(image_cache_byte_limit)),
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
    /// remains authoritative and dropping the shared image frees the allocation.
    pub fn CreateSharedImage(
        &self,
        id: ResourceId,
        size: Size,
        format: &SharedImageFormat,
        color_space: &RasterColorSpace,
    ) -> io::Result<RasterSharedImage> {
        if id == ResourceId::default() || size.width() <= 0 || size.height() <= 0 {
            return Err(io::Error::other("invalid raster image allocation"));
        }
        validate_pixel_format(format)?;
        let format = *format;
        let mut allocations = self
            .0
            .allocations
            .lock()
            .expect("raster allocation index poisoned");
        // Caller-selected tile IDs and internally allocated UI IDs share the
        // same monotonic allocation namespace and the very same allocation table.
        self.0
            .next_allocation
            .fetch_max(id.0 as u64 + 1, Ordering::Relaxed);
        // Source SharedImage creation returns its backend allocation identity;
        // cc's logical ResourceId is only a preferred number at this allocation
        // boundary. UI allocations share this table, so an occupied preference
        // obtains a new actual identity. All acquire/export paths consume the
        // returned SharedImageBackend::RasterAllocationId(), never the hint.
        let id = if allocations
            .get(&id)
            .is_some_and(|value| value.strong_count() != 0)
        {
            self.NextAllocationLocked(&allocations)?
        } else {
            id
        };
        self.CreateSharedImageLocked(&mut allocations, id, size, format, color_space.clone())
    }
    fn CreateSharedImageLocked(
        &self,
        allocations: &mut HashMap<ResourceId, Weak<Allocation>>,
        id: ResourceId,
        size: Size,
        format: SharedImageFormat,
        color_space: RasterColorSpace,
    ) -> io::Result<RasterSharedImage> {
        if allocations.get(&id).and_then(Weak::upgrade).is_some() {
            return Err(io::Error::other(
                "raster resource id already owns an allocation",
            ));
        }
        allocations.retain(|_, allocation| allocation.strong_count() != 0);
        let creation = self.token();
        let image = Arc::new(Allocation {
            resource: id,
            size,
            format,
            color_space,
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
        color_space: &RasterColorSpace,
    ) -> io::Result<RasterSharedImage> {
        if size.width() <= 0 || size.height() <= 0 {
            return Err(io::Error::other("invalid raster UI image size"));
        }
        validate_pixel_format(format)?;
        let format = *format;
        let mut allocations = self
            .0
            .allocations
            .lock()
            .map_err(|_| io::Error::other("raster allocation index poisoned"))?;
        let id = self.NextAllocationLocked(&allocations)?;
        self.CreateSharedImageLocked(&mut allocations, id, size, format, color_space.clone())
    }

    /// Creates the one multiplanar SharedImage consumed by a video
    /// TextureDrawQuad. Plane storage remains the decoder's immutable Arc
    /// allocations and is never copied into an RGBA tile.
    pub fn CreateImmutableVideoSharedImage(
        &self,
        frame: &image_resource::VideoFrameResource,
    ) -> io::Result<RasterSharedImage> {
        if !frame.Validate() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid immutable NV12 video frame",
            ));
        }
        let size = Size::new(
            i32::try_from(frame.width).map_err(|_| io::Error::other("video width overflow"))?,
            i32::try_from(frame.height).map_err(|_| io::Error::other("video height overflow"))?,
        );
        let format =
            SharedImageFormat::MultiPlane(PlaneConfig::kY_UV, Subsampling::k420, ChannelFormat::k8);
        let color_space = RasterColorSpace {
            primaries: match frame.color_space.primaries {
                image_resource::VideoColorPrimaries::Bt709 => ColorPrimaryId::Bt709,
                image_resource::VideoColorPrimaries::Smpte170M => ColorPrimaryId::Smpte170M,
                image_resource::VideoColorPrimaries::Ebu3213E => ColorPrimaryId::Ebu3213E,
            },
            transfer: match frame.color_space.transfer {
                image_resource::VideoTransferFunction::Bt709Apple => ColorTransferId::Bt709Apple,
            },
            matrix: match frame.color_space.matrix {
                image_resource::VideoColorMatrix::Bt709 => ColorMatrixId::Bt709,
                image_resource::VideoColorMatrix::Smpte170M => ColorMatrixId::Smpte170M,
            },
            range: match frame.color_space.range {
                image_resource::VideoColorRange::Limited => ColorRangeId::Limited,
            },
            ..RasterColorSpace::default()
        };
        let id = {
            let allocations = self
                .0
                .allocations
                .lock()
                .map_err(|_| io::Error::other("raster allocation index poisoned"))?;
            self.NextAllocationLocked(&allocations)?
        };
        let creation = self.token();
        let lease = Arc::new(HostVideoResource {
            resource: id,
            size,
            format,
            color_space,
            y_stride: frame.y_stride,
            uv_stride: frame.uv_stride,
            y_plane: frame.y_plane.clone(),
            uv_plane: frame.uv_plane.clone(),
        });
        Ok(RasterSharedImage::new(Arc::new(VideoAllocation {
            resource: id,
            creation,
            destruction_token: Mutex::new(creation),
            lease,
        })))
    }

    /// VideoResourceUpdater's immutable export boundary. The Viz resource id
    /// names the actual multiplanar SharedImage allocation; no RGBA tile or
    /// second resource table is introduced for video frames.
    pub fn CreateImmutableVideoTransferableResource(
        &self,
        frame: &image_resource::VideoFrameResource,
    ) -> io::Result<RasterTransferableResource> {
        let shared_image = self.CreateImmutableVideoSharedImage(frame)?;
        Ok(RasterTransferableResource {
            id: shared_image.RasterAllocationId(),
            sync_token: shared_image.creation_sync_token(),
            shared_image,
            source: RasterResourceSource::Video,
            synchronization_type: RasterSynchronizationType::SyncToken,
        })
    }
    fn NextAllocationLocked(
        &self,
        allocations: &HashMap<ResourceId, Weak<Allocation>>,
    ) -> io::Result<ResourceId> {
        loop {
            let next = self.0.next_allocation.fetch_add(1, Ordering::Relaxed);
            let next = u32::try_from(next)
                .map_err(|_| io::Error::other("raster allocation identities exhausted"))?;
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
            .ok_or_else(|| io::Error::other("cc-selected raster resource has no live shared image"))
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

fn CopyPlaybackRect(
    destination: &mut [u8],
    source: &[u8],
    allocation_size: Size,
    content_rect: &Rect,
    playback_rect: &Rect,
) -> io::Result<()> {
    if destination.len() != source.len() || destination.len() != pixel_len(allocation_size)? {
        return Err(io::Error::other(
            "partial raster buffers do not match their allocation",
        ));
    }
    let local_x = playback_rect.x - content_rect.x;
    let local_y = playback_rect.y - content_rect.y;
    if local_x < 0
        || local_y < 0
        || playback_rect.width < 0
        || playback_rect.height < 0
        || local_x + playback_rect.width > allocation_size.width()
        || local_y + playback_rect.height > allocation_size.height()
    {
        return Err(io::Error::other(
            "partial raster playback rect exceeds its allocation",
        ));
    }
    let row_bytes = allocation_size.width() as usize * 4;
    let copy_bytes = playback_rect.width as usize * 4;
    let first_byte = local_y as usize * row_bytes + local_x as usize * 4;
    for row in 0..playback_rect.height as usize {
        let begin = first_byte + row * row_bytes;
        let end = begin + copy_bytes;
        destination[begin..end].copy_from_slice(&source[begin..end]);
    }
    Ok(())
}

impl<'image>
    RasterBuffer<
        RasterSourceStorage<SkRegion>,
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
        mut request: RasterPlayback<
            '_,
            RasterSourceStorage<SkRegion>,
            RasterTransform,
            RasterImageProviderRef<'image, SkRegion>,
            ScrollOffsetMap,
        >,
    ) -> io::Result<()> {
        if self.finished {
            return Err(io::Error::other(
                "raster buffer cannot finish playback twice",
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
                "preserved resource content identity does not match its actual pixels",
            ));
        }
        let mut playback_rect = *request.content_rect;
        if has_previous {
            playback_rect.Intersect(*request.invalid_content_rect);
        }
        if playback_rect.IsEmpty() {
            self.previous = previous;
            return Err(io::Error::other("raster task has no dirty pixels"));
        }
        // The translated CPU backend can derive temporary saveLayer bounds
        // from the device clip. For a transparent source, a dirty-rect replay
        // can therefore produce different edge coverage from the original
        // full-tile raster. A one-tile-row layer is safe to repaint in place:
        // every pixel then comes from one playback context. Multi-row layers
        // and rounded backings with explicit replay regions keep Chromium's
        // preserved-pixel partial-raster path.
        if has_previous
            && request.source.requires_clear
            && request.source.FullReplaysForPartialRaster().is_empty()
            && request.source.size().height() <= request.content_rect.height
        {
            playback_rect = *request.content_rect;
        }
        if std::env::var_os("RECHROM_TRACE_PARTIAL_RASTER").is_some() {
            eprintln!(
                "partial-playback resource={} new={} old={} has_previous={} requires_clear={} content={:?} invalid={:?} playback={:?}",
                self.allocation.resource.0,
                request.new_content_id,
                self.resource_content_id,
                has_previous,
                request.source.requires_clear,
                request.content_rect,
                request.invalid_content_rect,
                playback_rect
            );
        }
        let mut storage = previous.take().map_or_else(
            || pixel_len(self.allocation.size).map(|len| vec![0; len]),
            |pixels| Ok(pixels.pixels),
        )?;
        let mut selected_full_replays = Vec::new();
        if has_previous && playback_rect != *request.content_rect {
            for replay in request.source.FullReplaysForPartialRaster() {
                if !replay.replay_rect.Intersects(playback_rect) {
                    continue;
                }
                selected_full_replays.push(*replay);
            }
        }
        let needs_full_replay = !selected_full_replays.is_empty();
        if std::env::var_os("RECHROM_TRACE_PARTIAL_RASTER").is_some() && has_previous {
            eprintln!(
                "partial-full-replay-playback hints={} selected={} playback={:?}",
                request.source.FullReplaysForPartialRaster().len(),
                needs_full_replay,
                playback_rect
            );
        }
        let new_content_id = request.new_content_id;
        let pixels = if needs_full_replay {
            // A rounded opaque backing must be replayed with full-raster clip
            // context instead of blended over its previous coverage. First do
            // ordinary partial playback for the whole dirty area. Then raster
            // the complete tile into fresh storage and publish only the exact
            // backing regions that requested full replay.
            let mut partial_target = match RasterBackendTarget::new(
                self.allocation.size,
                self.allocation.format,
                self.allocation.color_space.clone(),
                storage,
            ) {
                Ok(target) => target,
                Err(error) => return self.FinishFailedPlayback(error),
            };
            for replay in &selected_full_replays {
                let mut partial_rect = replay.outer;
                partial_rect.Intersect(playback_rect);
                if partial_rect.IsEmpty() {
                    continue;
                }
                if let Err(error) = self.provider.0.backend.Raster(
                    RasterPlayback {
                        source: request.source,
                        content_rect: request.content_rect,
                        invalid_content_rect: &partial_rect,
                        new_content_id: request.new_content_id,
                        raster_transform: request.raster_transform,
                        use_lcd_text: request.use_lcd_text,
                        msaa_sample_count: request.msaa_sample_count,
                        visible: request.visible,
                        hdr_headroom: request.hdr_headroom,
                        image_provider: &mut *request.image_provider,
                        raster_inducing_scroll_offsets: request.raster_inducing_scroll_offsets,
                    },
                    &mut partial_target,
                ) {
                    return self.FinishFailedPlayback(error);
                }
            }
            storage = match partial_target.into_pixels() {
                Ok(pixels) => pixels,
                Err(error) => return self.FinishFailedPlayback(error),
            };
            let scratch = match pixel_len(self.allocation.size) {
                Ok(len) => vec![0; len],
                Err(error) => return self.FinishFailedPlayback(error),
            };
            let mut target = match RasterBackendTarget::new(
                self.allocation.size,
                self.allocation.format,
                self.allocation.color_space.clone(),
                scratch,
            ) {
                Ok(target) => target,
                Err(error) => return self.FinishFailedPlayback(error),
            };
            if let Err(error) = self.provider.0.backend.Raster(
                RasterPlayback {
                    source: request.source,
                    content_rect: request.content_rect,
                    invalid_content_rect: request.content_rect,
                    new_content_id: request.new_content_id,
                    raster_transform: request.raster_transform,
                    use_lcd_text: request.use_lcd_text,
                    msaa_sample_count: request.msaa_sample_count,
                    visible: request.visible,
                    hdr_headroom: request.hdr_headroom,
                    image_provider: &mut *request.image_provider,
                    raster_inducing_scroll_offsets: request.raster_inducing_scroll_offsets,
                },
                &mut target,
            ) {
                return self.FinishFailedPlayback(error);
            }
            let scratch = match target.into_pixels() {
                Ok(pixels) => pixels,
                Err(error) => return self.FinishFailedPlayback(error),
            };
            for replay in selected_full_replays {
                let mut replay_rect = replay.replay_rect;
                replay_rect.Intersect(playback_rect);
                if replay_rect.IsEmpty() {
                    continue;
                }
                if let Err(error) = CopyPlaybackRect(
                    &mut storage,
                    &scratch,
                    self.allocation.size,
                    request.content_rect,
                    &replay_rect,
                ) {
                    return self.FinishFailedPlayback(error);
                }
            }
            storage
        } else {
            let mut target = match RasterBackendTarget::new(
                self.allocation.size,
                self.allocation.format,
                self.allocation.color_space.clone(),
                storage,
            ) {
                Ok(target) => target,
                Err(error) => return self.FinishFailedPlayback(error),
            };
            if let Err(error) = self.provider.0.backend.Raster(
                RasterPlayback {
                    invalid_content_rect: &playback_rect,
                    ..request
                },
                &mut target,
            ) {
                return self.FinishFailedPlayback(error);
            }
            match target.into_pixels() {
                Ok(pixels) => pixels,
                Err(error) => return self.FinishFailedPlayback(error),
            }
        };
        let opaque = pixels_opaque(&pixels);
        let pixels = Arc::new(HostMemoryResource {
            resource: self.allocation.resource,
            size: self.allocation.size,
            format: self.allocation.format,
            color_space: self.allocation.color_space.clone(),
            row_bytes: self.allocation.size.width() as usize * 4,
            pixels,
            opaque,
            content_id: new_content_id,
        });
        {
            let mut snapshots = self
                .provider
                .0
                .content_snapshots
                .lock()
                .expect("raster content snapshot index poisoned");
            snapshots.Insert(new_content_id, Arc::clone(&pixels));
        }
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
        RasterSourceStorage<SkRegion>,
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
                "raster metadata does not match shared image allocation",
            ));
        }
        let published = {
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
            state.rasterizing = true;
            state.pixels.take()
        };
        let mut previous = if let Some(pixels) = published {
            let wants_published_pixels = resource_content_id != 0
                && resource_content_id == previous_content_id
                && pixels.content_id == resource_content_id;
            let cache_owns_publication = self
                .0
                .content_snapshots
                .lock()
                .expect("raster content snapshot index poisoned")
                .Owns(pixels.content_id, &pixels);
            let owned_references = 1 + usize::from(cache_owns_publication);
            if Arc::strong_count(&pixels) > owned_references {
                let mut state = allocation
                    .state
                    .lock()
                    .expect("raster allocation lock poisoned");
                debug_assert!(state.rasterizing && state.pixels.is_none());
                state.pixels = Some(pixels);
                state.rasterizing = false;
                return Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "raster resource still has a live immutable read lease",
                ));
            }
            if wants_published_pixels {
                // In-place partial raster consumes the matching cached
                // publication so exclusive ownership can move into the write
                // target. A full raster for different tile content leaves the
                // cache entry alive and starts from a fresh zeroed allocation;
                // that old content may still be the source of a later COW
                // partial raster into another resource.
                if cache_owns_publication {
                    let mut snapshots = self
                        .0
                        .content_snapshots
                        .lock()
                        .expect("raster content snapshot index poisoned");
                    drop(snapshots.Take(pixels.content_id));
                }
                Some(match Arc::try_unwrap(pixels) {
                    Ok(pixels) => pixels,
                    Err(_) => unreachable!("owned raster publication"),
                })
            } else {
                None
            }
        } else {
            None
        };
        let mut effective_resource_content_id = resource_content_id;
        if previous.is_none()
            && resource_content_id != 0
            && resource_content_id == previous_content_id
        {
            let snapshot = {
                let mut snapshots = self
                    .0
                    .content_snapshots
                    .lock()
                    .expect("raster content snapshot index poisoned");
                snapshots.Take(resource_content_id)
            };
            if let Some(snapshot) = snapshot.filter(|snapshot| {
                snapshot.content_id == resource_content_id
                    && snapshot.size == allocation.size
                    && snapshot.format == allocation.format
                    && snapshot.color_space == allocation.color_space
            }) {
                previous = Some(HostMemoryResource {
                    resource: allocation.resource,
                    size: snapshot.size,
                    format: snapshot.format,
                    color_space: snapshot.color_space.clone(),
                    row_bytes: snapshot.row_bytes,
                    pixels: snapshot.pixels.clone(),
                    opaque: snapshot.opaque,
                    content_id: snapshot.content_id,
                });
            } else {
                // The pool may request copy-on-write after the bounded
                // snapshot has already been consumed or evicted. A full
                // replay is still correct for the fresh allocation.
                effective_resource_content_id = 0;
            }
        }
        Ok(RasterBufferSession {
            provider: self.clone(),
            allocation,
            previous,
            resource_content_id: effective_resource_content_id,
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
/// the same allocation/cache owners; it does not clone pixel storage.
/// Canonical cc code continues to choose tasks, resources and completion order.
impl compositor_core::CompositorRasterProvider for RasterEngine {
    fn CreateSharedImage(
        &self,
        resource: ResourceId,
        size: Size,
        format: &SharedImageFormat,
        color_space: &RasterColorSpace,
    ) -> io::Result<RasterSharedImage> {
        RasterEngine::CreateSharedImage(self, resource, size, format, color_space)
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
            RasterSourceStorage<SkRegion>,
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
            RasterSourceStorage<SkRegion>,
            RasterTransform,
            RasterImageProviderRef<'_, SkRegion>,
            ScrollOffsetMap,
        >>::CanPartialRasterIntoProvidedResource(self)
    }
    fn NotifyWorkSubmitted(&self) {
        <Self as RasterBufferProviderBackend<
            RasterSourceStorage<SkRegion>,
            RasterTransform,
            RasterImageProviderRef<'_, SkRegion>,
            ScrollOffsetMap,
        >>::NotifyWorkSubmitted(&mut self.clone());
    }
    fn Shutdown(&self) -> io::Result<()> {
        <Self as RasterBufferProviderBackend<
            RasterSourceStorage<SkRegion>,
            RasterTransform,
            RasterImageProviderRef<'_, SkRegion>,
            ScrollOffsetMap,
        >>::Shutdown(&mut self.clone())
    }
}

#[cfg(test)]
mod video_tests {
    use super::*;
    use image_resource::{
        ImageOrientation, VideoColorMatrix, VideoColorPrimaries, VideoColorRange, VideoColorSpace,
        VideoFrameResource, VideoPixelFormat, VideoTransferFunction,
    };

    fn frame(width: u32, height: u32) -> VideoFrameResource {
        let y_stride = width as usize;
        let uv_stride = (width as usize).div_ceil(2) * 2;
        VideoFrameResource {
            width,
            height,
            orientation: ImageOrientation::kDefault,
            format: VideoPixelFormat::Nv12,
            color_space: VideoColorSpace {
                primaries: VideoColorPrimaries::Bt709,
                transfer: VideoTransferFunction::Bt709Apple,
                matrix: VideoColorMatrix::Bt709,
                range: VideoColorRange::Limited,
            },
            y_stride,
            uv_stride,
            y_plane: Arc::new(vec![16; y_stride * height as usize]),
            uv_plane: Arc::new(vec![128; uv_stride * (height as usize).div_ceil(2)]),
        }
    }

    fn snapshot(content_id: u64, byte: u8) -> Arc<HostMemoryResource> {
        Arc::new(HostMemoryResource {
            resource: ResourceId(content_id as u32),
            size: Size::new(1, 1),
            format: SinglePlaneFormat::RGBA_8888,
            color_space: RasterColorSpace::CreateSRGB(),
            row_bytes: 4,
            pixels: vec![byte; 4],
            opaque: true,
            content_id,
        })
    }

    #[test]
    fn content_snapshot_cache_is_bounded_and_consuming() {
        let mut cache = ContentSnapshotCache::new(8);
        cache.Insert(1, snapshot(1, 1));
        cache.Insert(2, snapshot(2, 2));
        cache.Insert(3, snapshot(3, 3));
        assert!(cache.Take(1).is_none());
        assert_eq!(cache.Take(2).unwrap().pixels, vec![2; 4]);
        assert!(cache.Take(2).is_none());
        assert_eq!(cache.Take(3).unwrap().pixels, vec![3; 4]);
        assert_eq!(cache.bytes, 0);
    }

    #[test]
    fn snapshot_partial_copy_changes_only_playback_pixels() {
        let size = Size::new(4, 3);
        let mut destination = vec![7; pixel_len(size).unwrap()];
        let source = (0..destination.len())
            .map(|byte| byte as u8)
            .collect::<Vec<_>>();
        CopyPlaybackRect(
            &mut destination,
            &source,
            size,
            &Rect::new(10, 20, 4, 3),
            &Rect::new(11, 21, 2, 1),
        )
        .unwrap();

        for pixel in 0..12 {
            let bytes = pixel * 4..pixel * 4 + 4;
            if pixel == 5 || pixel == 6 {
                assert_eq!(destination[bytes.clone()], source[bytes]);
            } else {
                assert_eq!(destination[bytes], [7; 4]);
            }
        }
    }

    #[test]
    fn immutable_video_shared_image_preserves_odd_nv12_planes_and_color() {
        let engine = RasterEngine::new(91, 0);
        let frame = frame(3, 3);
        let image = engine.CreateImmutableVideoSharedImage(&frame).unwrap();
        let lease = image.RasterReadLease().unwrap();
        assert_eq!(lease.format().plane_config(), PlaneConfig::kY_UV);
        assert_eq!(lease.format().subsampling(), Subsampling::k420);
        assert_eq!(lease.color_space().transfer, ColorTransferId::Bt709Apple);
        let mapped = MapVideoResourceForRead(lease.as_ref()).unwrap();
        assert_eq!(mapped.size, Size::new(3, 3));
        assert_eq!(mapped.y_stride, 3);
        assert_eq!(mapped.uv_stride, 4);
        assert_eq!(mapped.y_plane.as_ptr(), frame.y_plane.as_ptr());
        assert_eq!(mapped.uv_plane.as_ptr(), frame.uv_plane.as_ptr());
    }

    #[test]
    fn invalid_video_plane_lengths_are_rejected() {
        let engine = RasterEngine::new(92, 0);
        let mut frame = frame(2, 2);
        frame.uv_plane = Arc::new(vec![128]);
        assert!(engine.CreateImmutableVideoSharedImage(&frame).is_err());
    }

    #[test]
    fn immutable_video_export_is_one_video_transferable_resource() {
        let engine = RasterEngine::new(93, 0);
        let resource = engine
            .CreateImmutableVideoTransferableResource(&frame(4, 2))
            .unwrap();
        assert_ne!(resource.id, ResourceId::default());
        assert_eq!(resource.id, resource.shared_image.RasterAllocationId());
        assert_eq!(resource.source, RasterResourceSource::Video);
        assert_eq!(
            resource.synchronization_type,
            RasterSynchronizationType::SyncToken
        );
        assert!(resource.sync_token.verified_flush);
    }

    #[test]
    fn ordinary_shared_image_preserves_descriptor_color_space_in_read_lease() {
        let engine = RasterEngine::new(94, 0);
        let color_space = RasterColorSpace::CreateDisplayP3();
        let image = engine
            .CreateSharedImage(
                ResourceId(7),
                Size::new(2, 2),
                &SinglePlaneFormat::RGBA_8888,
                &color_space,
            )
            .unwrap();
        let mut mapping = image.clone().MapRasterWrite().unwrap();
        mapping.bytes_mut().fill(255);
        drop(mapping);
        assert_eq!(image.RasterReadLease().unwrap().color_space(), color_space);
    }

    #[test]
    fn ui_shared_image_preserves_requested_color_space() {
        let engine = RasterEngine::new(95, 0);
        let color_space = RasterColorSpace::CreateDisplayP3();
        let image = engine
            .CreateUiSharedImage(Size::new(1, 1), &SinglePlaneFormat::BGRA_8888, &color_space)
            .unwrap();
        let mapping = image.clone().MapRasterWrite().unwrap();
        drop(mapping);
        assert_eq!(image.RasterReadLease().unwrap().color_space(), color_space);
    }
}
