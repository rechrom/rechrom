//! Concrete external resource values used by the native compositor family.
//!
//! The cc resource pool owns reuse, accounting and release policy. The actual
//! shared image remains an injected platform object behind `SharedImageBackend`;
//! this module does not allocate pixels or implement a second resource pool.

use super::types::{CpuCompositorTypes, CpuSkiaBackend};
use crate::adapter::{
    geometry::Size,
    layer_tree_host_impl::{ContentColorUsage, HostDisplayColorSpaces},
    layer_tree_impl::ResourceId,
    resource_pool::{
        PoolSharedImage, PoolTransferableResource, ResourcePoolMemoryDump, ResourcePoolTypes,
        SharedImageTracingImportance,
    },
    resource_types::SharedImageFormat,
};
use std::{cmp::Ordering, io, sync::Arc};

// Source vocabulary: ui/gfx/color_space.h:50-153,413-429.
pub use layerize::commit::color_space::{
    ColorMatrixId, ColorPrimaryId, ColorRangeId, ColorTransferId, RasterColorSpace,
};

/// Immutable native gfx::DisplayColorSpaces value. The six entries retain the
/// source usage/alpha indexing; callers can inject an already-resolved display
/// value without moving display policy into cc.
// Source: ui/gfx/display_color_spaces.h:49-159.
#[derive(Clone, Debug, PartialEq)]
pub struct RasterDisplayColorSpaces {
    color_spaces: [RasterColorSpace; 6],
    formats: [SharedImageFormat; 6],
    pub primaries: [f32; 8],
    pub sdr_max_luminance_nits: f32,
    pub hdr_max_luminance_relative: f32,
}
impl Default for RasterDisplayColorSpaces {
    // Source: ui/gfx/display_color_spaces.cc:17-56.
    fn default() -> Self {
        #[cfg(any(target_os = "macos", feature = "chromeos"))]
        let format = crate::adapter::resource_types::SinglePlaneFormat::BGRA_8888;
        #[cfg(not(any(target_os = "macos", feature = "chromeos")))]
        let format = crate::adapter::resource_types::SinglePlaneFormat::RGBA_8888;
        Self {
            color_spaces: std::array::from_fn(|_| RasterColorSpace::CreateSRGB()),
            formats: [format; 6],
            // SkNamedPrimariesExt::kSRGB / Rec709.
            primaries: [0.64, 0.33, 0.30, 0.60, 0.15, 0.06, 0.3127, 0.3290],
            sdr_max_luminance_nits: 203.,
            hdr_max_luminance_relative: 1.,
        }
    }
}
impl RasterDisplayColorSpaces {
    fn index(usage: ContentColorUsage, needs_alpha: bool) -> usize {
        let base = match usage {
            ContentColorUsage::kSRGB => 0,
            ContentColorUsage::kWideColorGamut => 2,
            ContentColorUsage::kHDR => 4,
        };
        base + usize::from(needs_alpha)
    }
    pub fn SetOutputColorSpaceAndFormat(
        &mut self,
        usage: ContentColorUsage,
        needs_alpha: bool,
        color_space: RasterColorSpace,
        format: SharedImageFormat,
    ) {
        let index = Self::index(usage, needs_alpha);
        self.color_spaces[index] = color_space;
        self.formats[index] = format;
    }
    pub fn GetOutputColorSpace(
        &self,
        usage: ContentColorUsage,
        needs_alpha: bool,
    ) -> RasterColorSpace {
        self.color_spaces[Self::index(usage, needs_alpha)].clone()
    }
    pub fn GetOutputFormat(
        &self,
        usage: ContentColorUsage,
        needs_alpha: bool,
    ) -> SharedImageFormat {
        self.formats[Self::index(usage, needs_alpha)]
    }
    // Source: ui/gfx/display_color_spaces.cc:157-161.
    pub fn SupportsHDR(&self) -> bool {
        self.GetOutputColorSpace(ContentColorUsage::kHDR, false)
            .IsHDR()
            || self
                .GetOutputColorSpace(ContentColorUsage::kHDR, true)
                .IsHDR()
            || self.hdr_max_luminance_relative > 1.
    }
    // Source: ui/gfx/display_color_spaces.cc:163-165.
    pub fn GetHdrHeadroom(&self) -> f32 {
        self.hdr_max_luminance_relative.log2()
    }
    // Source: ui/gfx/display_color_spaces.cc:116-155.
    pub fn GetRasterAndCompositeColorSpace(
        &self,
        mut usage: ContentColorUsage,
    ) -> RasterColorSpace {
        if !self.SupportsHDR() && usage == ContentColorUsage::kHDR {
            usage = ContentColorUsage::kWideColorGamut;
        }
        let result = self.GetOutputColorSpace(usage, true);
        if result.IsSuitableForBlending() {
            return result;
        }
        match usage {
            ContentColorUsage::kSRGB => RasterColorSpace::CreateSRGB(),
            ContentColorUsage::kHDR => RasterColorSpace::CreateDisplayP3Hdr(),
            ContentColorUsage::kWideColorGamut => RasterColorSpace::CreateDisplayP3(),
        }
    }
}
impl HostDisplayColorSpaces<RasterColorSpace> for RasterDisplayColorSpaces {
    fn GetRasterAndCompositeColorSpace(&self, usage: ContentColorUsage) -> RasterColorSpace {
        RasterDisplayColorSpaces::GetRasterAndCompositeColorSpace(self, usage)
    }
    fn GetHdrHeadroom(&self) -> f32 {
        RasterDisplayColorSpaces::GetHdrHeadroom(self)
    }
}

// Sync-token and returned-resource values live in compositor_resource.
pub use compositor_resource::returned_resource::{
    CommandBufferNamespace, RasterReturnedResource, RasterSyncToken,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RasterDumpGuid(pub [u64; 2]);

/// Exclusive software-write lease for the actual shared-image allocation.
/// Dropping the lease publishes the written pixels back to the same allocation;
/// callers never receive a provider, resource table, or copied pixel snapshot.
pub trait RasterWriteMapping: Send {
    fn size(&self) -> Size;
    fn format(&self) -> SharedImageFormat;
    fn row_bytes(&self) -> usize;
    fn bytes_mut(&mut self) -> &mut [u8];
}

/// Platform shared-image owner injected by the browser/Viz boundary.
pub trait SharedImageBackend: Send + Sync {
    /// Identity of the actual raster allocation, independent of the Viz resource
    /// ID assigned later by ClientResourceProvider::ImportResource. Raster
    /// acquisition precedes export and must address the backing allocation.
    fn RasterAllocationId(&self) -> ResourceId;
    fn creation_sync_token(&self) -> RasterSyncToken;
    /// Completes the current import and returns the parent-visible release
    /// point, matching ClientSharedImage::EndImport.
    fn EndImport(&self, token: &RasterSyncToken) -> RasterSyncToken;
    fn UpdateDestructionSyncToken(&self, token: &RasterSyncToken);
    fn OnMemoryDump(
        &self,
        dump: &mut dyn ResourcePoolMemoryDump<RasterDumpGuid>,
        guid: &RasterDumpGuid,
        importance: SharedImageTracingImportance,
    );
    /// Immutable read lease exported by the raster owner. Non-host-readable
    /// images may return `None`; the current software Renderer requires `Some` and never
    /// obtains the provider, cache, tile or mutable allocation behind it.
    fn RasterReadLease(&self) -> Option<Arc<dyn super::raster_execution::RasterResourceLease>>;
    /// Maps the same allocation for exclusive host writes. Implementations must
    /// reject concurrent raster/read ownership rather than fabricate storage.
    fn MapRasterWrite(self: Arc<Self>) -> io::Result<Box<dyn RasterWriteMapping>>;
}

#[derive(Clone)]
pub struct RasterSharedImage(Arc<dyn SharedImageBackend>);
impl RasterSharedImage {
    pub fn new(image: Arc<dyn SharedImageBackend>) -> Self {
        Self(image)
    }
    pub fn creation_sync_token(&self) -> RasterSyncToken {
        self.0.creation_sync_token()
    }
    pub fn RasterAllocationId(&self) -> ResourceId {
        self.0.RasterAllocationId()
    }
    pub fn RasterReadLease(&self) -> Option<Arc<dyn super::raster_execution::RasterResourceLease>> {
        self.0.RasterReadLease()
    }
    pub fn MapRasterWrite(&self) -> io::Result<Box<dyn RasterWriteMapping>> {
        Arc::clone(&self.0).MapRasterWrite()
    }
}
impl PoolSharedImage<RasterSyncToken, RasterDumpGuid> for RasterSharedImage {
    fn UpdateDestructionSyncToken(&self, token: &RasterSyncToken) {
        self.0.UpdateDestructionSyncToken(token);
    }
    fn OnMemoryDump(
        &self,
        dump: &mut dyn ResourcePoolMemoryDump<RasterDumpGuid>,
        guid: &RasterDumpGuid,
        importance: SharedImageTracingImportance,
    ) {
        self.0.OnMemoryDump(dump, guid, importance);
    }
}
impl crate::adapter::layer_tree_host_impl::HostUISharedImage<RasterSyncToken>
    for RasterSharedImage
{
    fn creation_sync_token(&self) -> RasterSyncToken {
        self.0.creation_sync_token()
    }
    fn UpdateDestructionSyncToken(&mut self, token: &RasterSyncToken) {
        self.0.UpdateDestructionSyncToken(token);
    }
}

// Source: ui/gfx/buffer_types.h:24-39.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RasterBufferUsage {
    GpuRead,
    Scanout,
    ScanoutCameraReadWrite,
    CameraAndCpuReadWrite,
    ScanoutCpuReadWrite,
    ScanoutVdaWrite,
    ProtectedScanout,
    ProtectedScanoutVdaWrite,
    GpuReadCpuReadWrite,
    ScanoutVeaCpuRead,
    ScanoutFrontRendering,
    VeaReadCameraAndCpuReadWrite,
}

// Source: components/viz/common/resources/transferable_resource.h:66-87.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum RasterResourceSource {
    #[default]
    Unknown = 0,
    Ar = 1,
    Canvas = 2,
    DrawingBuffer = 3,
    ExoBuffer = 4,
    HeadsUpDisplay = 5,
    ImageLayerBridge = 6,
    PpbGraphics3d = 7,
    PepperGraphics2d = 8,
    ViewTransition = 9,
    StaleContent = 10,
    Test = 11,
    TileRasterTask = 12,
    Ui = 13,
    Video = 14,
    WebGpuSwapBuffer = 15,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum RasterSynchronizationType {
    #[default]
    SyncToken,
    GpuCommandsCompleted,
    ReleaseFence,
}

/// The transferable value keeps the injected shared-image owner alive while
/// Viz holds the resource. It contains no copied pixels.
// Source value copy: components/viz/common/resources/transferable_resource.h.
// Only the native shared-image lease is shared; no owner metadata is copied.
#[derive(Clone)]
pub struct RasterTransferableResource {
    pub id: ResourceId,
    pub shared_image: RasterSharedImage,
    pub source: RasterResourceSource,
    pub sync_token: RasterSyncToken,
    pub synchronization_type: RasterSynchronizationType,
}
impl RasterTransferableResource {
    // Source: components/viz/common/resources/transferable_resource.cc:44-50.
    pub fn ToReturnedResource(&self) -> RasterReturnedResource {
        RasterReturnedResource {
            id: self.id,
            sync_token: self.shared_image.0.EndImport(&self.sync_token),
            count: 1,
            lost: false,
        }
    }
    pub fn RasterReadLease(&self) -> Option<Arc<dyn super::raster_execution::RasterResourceLease>> {
        self.shared_image.RasterReadLease()
    }
}
impl PoolTransferableResource for RasterTransferableResource {
    // Source: components/viz/common/resources/transferable_resource.h:45-62.
    fn SetGpuCommandsCompletedSynchronization(&mut self) {
        self.synchronization_type = RasterSynchronizationType::GpuCommandsCompleted;
    }
}


impl<R: CpuSkiaBackend> ResourcePoolTypes for CpuCompositorTypes<R> {
    type ColorSpace = RasterColorSpace;
    type SyncToken = RasterSyncToken;
    type SharedImage = RasterSharedImage;
    type DumpGuid = RasterDumpGuid;
    type BufferUsage = RasterBufferUsage;
    type ResourceSource = RasterResourceSource;
    type TransferableResource = RasterTransferableResource;

    fn EstimatedSizeInBytes(format: &SharedImageFormat, size: &Size) -> usize {
        format.EstimatedSizeInBytes(size)
    }
    fn VerifySizeInBytes(format: &SharedImageFormat, size: &Size) -> bool {
        format.VerifySizeInBytes(size)
    }
}

// Production compile assertion: the canonical native family, with the public
// SkRegion backend, supplies the complete resource-pool value contract.
const _: fn() = || {
    fn assert_resource_pool_types<T: ResourcePoolTypes>() {}
    assert_resource_pool_types::<CpuCompositorTypes<crate::runtime::skia::SkRegion>>();
};
