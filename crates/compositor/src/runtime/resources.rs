//! Concrete external resource values used by the native compositor family.
//!
//! The cc resource pool owns reuse, accounting and release policy. The actual
//! shared image remains an injected platform object behind `NativeSharedImage`;
//! this module does not allocate pixels or implement a second resource pool.

use super::types::{CpuCompositorTypes, CpuSkiaBackend};
use crate::adapter::{
    geometry::Size,
    layer_tree_host_impl::{ContentColorUsage, HostDisplayColorSpaces},
    layer_tree_impl::ResourceId,
    resource_pool::{
        PoolSharedImage, PoolTransferableResource, ResourcePoolMemoryDump, ResourcePoolTypes,
        ResourceSyncToken, SharedImageTracingImportance,
    },
    resource_types::SharedImageFormat,
};
use std::{cmp::Ordering, io, sync::Arc};

// Source vocabulary: ui/gfx/color_space.h:50-153,413-429.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ColorPrimaryId {
    Invalid = 0,
    Bt709,
    Bt470M,
    Bt470Bg,
    Smpte170M,
    Smpte240M,
    Film,
    Bt2020,
    SmpteSt428_1,
    SmpteSt431_2,
    P3,
    XyzD50,
    AdobeRgb,
    AppleGenericRgb,
    WideGamutColorSpin,
    Custom,
    Ebu3213E,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ColorTransferId {
    Invalid = 0,
    Bt709,
    Bt709Apple,
    Gamma18,
    Gamma22,
    Gamma24,
    Gamma28,
    Smpte170M,
    Smpte240M,
    Linear,
    Log,
    LogSqrt,
    Iec61966_2_4,
    Bt1361Ecg,
    Srgb,
    Bt2020_10,
    Bt2020_12,
    Pq,
    SmpteSt428_1,
    Hlg,
    SrgbHdr,
    LinearHdr,
    Custom,
    CustomHdr,
    ScrgbLinear80Nits,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ColorMatrixId {
    Invalid = 0,
    Rgb,
    Bt709,
    Fcc,
    Bt470Bg,
    Smpte170M,
    Smpte240M,
    Ycocg,
    Bt2020Ncl,
    Ydzdx,
    Gbr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ColorRangeId {
    Invalid = 0,
    Limited,
    Full,
    Derived,
}

/// IPC-safe gfx::ColorSpace value. Floating values compare by their native bit
/// representation, matching Chromium's field-wise serialized identity.
#[derive(Clone, Debug)]
pub struct RasterColorSpace {
    pub primaries: ColorPrimaryId,
    pub transfer: ColorTransferId,
    pub matrix: ColorMatrixId,
    pub range: ColorRangeId,
    pub custom_primary_matrix: [f32; 9],
    pub transfer_params: [f32; 7],
}
impl Default for RasterColorSpace {
    fn default() -> Self {
        Self {
            primaries: ColorPrimaryId::Invalid,
            transfer: ColorTransferId::Invalid,
            matrix: ColorMatrixId::Invalid,
            range: ColorRangeId::Invalid,
            custom_primary_matrix: [0.; 9],
            transfer_params: [0.; 7],
        }
    }
}
impl RasterColorSpace {
    // Source: ui/gfx/color_space.h:185-189.
    pub fn CreateSRGB() -> Self {
        Self {
            primaries: ColorPrimaryId::Bt709,
            transfer: ColorTransferId::Srgb,
            matrix: ColorMatrixId::Rgb,
            range: ColorRangeId::Full,
            ..Self::default()
        }
    }
    // Source: ui/gfx/color_space.h:221-225.
    pub fn CreateHDR10() -> Self {
        Self {
            primaries: ColorPrimaryId::Bt2020,
            transfer: ColorTransferId::Pq,
            matrix: ColorMatrixId::Rgb,
            range: ColorRangeId::Full,
            ..Self::default()
        }
    }
    pub fn CreateDisplayP3() -> Self {
        Self {
            primaries: ColorPrimaryId::P3,
            transfer: ColorTransferId::Srgb,
            matrix: ColorMatrixId::Rgb,
            range: ColorRangeId::Full,
            ..Self::default()
        }
    }
    pub fn CreateDisplayP3Hdr() -> Self {
        Self {
            primaries: ColorPrimaryId::P3,
            transfer: ColorTransferId::SrgbHdr,
            matrix: ColorMatrixId::Rgb,
            range: ColorRangeId::Full,
            ..Self::default()
        }
    }
    // Source: ui/gfx/color_space.cc:205-207.
    pub fn IsValid(&self) -> bool {
        self.primaries != ColorPrimaryId::Invalid
            && self.transfer != ColorTransferId::Invalid
            && self.matrix != ColorMatrixId::Invalid
            && self.range != ColorRangeId::Invalid
    }
    // Source: ui/gfx/color_space.cc:388-395.
    pub fn IsHDR(&self) -> bool {
        matches!(
            self.transfer,
            ColorTransferId::Pq
                | ColorTransferId::Hlg
                | ColorTransferId::LinearHdr
                | ColorTransferId::SrgbHdr
                | ColorTransferId::CustomHdr
                | ColorTransferId::ScrgbLinear80Nits
        )
    }
    // Source: ui/gfx/color_space.cc:640-667.
    pub fn IsSuitableForBlending(&self) -> bool {
        match self.transfer {
            ColorTransferId::Pq
            | ColorTransferId::Hlg
            | ColorTransferId::LinearHdr
            | ColorTransferId::ScrgbLinear80Nits => false,
            ColorTransferId::CustomHdr => self.transfer_params[6] >= 1.25,
            _ => true,
        }
    }
    /// Exact known-primary fast path used by the two cc call sites whose
    /// comparison operand is always `ColorSpace::CreateSRGB()`.
    // Source: ui/gfx/color_space.cc:94-113,830-840.
    pub fn ContainsSRGB(&self) -> bool {
        matches!(
            self.primaries,
            ColorPrimaryId::Bt709
                | ColorPrimaryId::Bt2020
                | ColorPrimaryId::SmpteSt428_1
                | ColorPrimaryId::SmpteSt431_2
                | ColorPrimaryId::P3
                | ColorPrimaryId::XyzD50
                | ColorPrimaryId::AdobeRgb
                | ColorPrimaryId::WideGamutColorSpin
        )
    }
    fn transfer_param_count(&self) -> usize {
        // Source: ui/gfx/color_space.cc:320-333.
        match self.transfer {
            ColorTransferId::Custom | ColorTransferId::CustomHdr => 7,
            ColorTransferId::Pq => 1,
            ColorTransferId::Hlg => 3,
            _ => 0,
        }
    }
    fn cmp_float_prefix(lhs: &[f32], rhs: &[f32], count: usize) -> Ordering {
        lhs[..count]
            .iter()
            .flat_map(|value| value.to_ne_bytes())
            .cmp(rhs[..count].iter().flat_map(|value| value.to_ne_bytes()))
    }
}
impl PartialEq for RasterColorSpace {
    fn eq(&self, rhs: &Self) -> bool {
        self.primaries == rhs.primaries
            && self.transfer == rhs.transfer
            && self.matrix == rhs.matrix
            && self.range == rhs.range
            && (self.primaries != ColorPrimaryId::Custom
                || Self::cmp_float_prefix(
                    &self.custom_primary_matrix,
                    &rhs.custom_primary_matrix,
                    9,
                ) == Ordering::Equal)
            && Self::cmp_float_prefix(
                &self.transfer_params,
                &rhs.transfer_params,
                self.transfer_param_count(),
            ) == Ordering::Equal
    }
}
impl Eq for RasterColorSpace {}
impl PartialOrd for RasterColorSpace {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}
impl Ord for RasterColorSpace {
    // Source: ui/gfx/color_space.cc:421-462.
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.primaries
            .cmp(&rhs.primaries)
            .then_with(|| self.transfer.cmp(&rhs.transfer))
            .then_with(|| self.matrix.cmp(&rhs.matrix))
            .then_with(|| self.range.cmp(&rhs.range))
            .then_with(|| {
                (self.primaries == ColorPrimaryId::Custom)
                    .then(|| {
                        Self::cmp_float_prefix(
                            &self.custom_primary_matrix,
                            &rhs.custom_primary_matrix,
                            9,
                        )
                    })
                    .unwrap_or(Ordering::Equal)
            })
            .then_with(|| {
                Self::cmp_float_prefix(
                    &self.transfer_params,
                    &rhs.transfer_params,
                    self.transfer_param_count(),
                )
            })
    }
}

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
    pub fn GetRasterAndCompositeColorSpace(&self, mut usage: ContentColorUsage) -> RasterColorSpace {
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

// Source vocabulary: gpu/command_buffer/common/constants.h and sync_token.h.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(i8)]
pub enum CommandBufferNamespace {
    #[default]
    Invalid = -1,
    GpuIo = 0,
    InProcess = 1,
    VizSkiaOutputSurface = 2,
    VizSkiaOutputSurfaceNonDdl = 3,
    GpuChannelSharedImageInterface = 4,
    WebNnContextInterface = 5,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RasterSyncToken {
    pub verified_flush: bool,
    pub namespace_id: CommandBufferNamespace,
    pub command_buffer_id: u64,
    pub release_count: u64,
}
impl RasterSyncToken {
    pub fn new(
        namespace_id: CommandBufferNamespace,
        command_buffer_id: u64,
        release_count: u64,
    ) -> Self {
        Self {
            verified_flush: false,
            namespace_id,
            command_buffer_id,
            release_count,
        }
    }
}
impl ResourceSyncToken for RasterSyncToken {
    // Source: gpu/command_buffer/common/sync_token.h:66-68.
    fn HasData(&self) -> bool {
        self.namespace_id != CommandBufferNamespace::Invalid
    }
}

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
pub trait NativeSharedImage: Send + Sync {
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
pub struct RasterSharedImage(Arc<dyn NativeSharedImage>);
impl RasterSharedImage {
    pub fn new(image: Arc<dyn NativeSharedImage>) -> Self {
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
impl crate::adapter::layer_tree_host_impl::HostUISharedImage<RasterSyncToken> for RasterSharedImage {
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

// Source value: components/viz/common/resources/returned_resource.h:25-60.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RasterReturnedResource {
    pub id: ResourceId,
    pub sync_token: RasterSyncToken,
    pub count: i32,
    pub lost: bool,
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
