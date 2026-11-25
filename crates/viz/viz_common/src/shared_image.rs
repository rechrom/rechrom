//! Shared-image backend values carried by a TransferableResource
//! (gpu/command_buffer/client/client_shared_image.h). The actual shared image
//! remains an injected platform object behind `SharedImageBackend`; this
//! module does not allocate pixels or implement a resource pool.
use crate::raster_resource_lease::RasterResourceLease;
use crate::returned_resource::RasterSyncToken;
use crate::ResourceId;
use compositor_tree::geometry::Size;
use compositor_values::shared_image_format::SharedImageFormat;
use std::{io, sync::Arc};

/// base::trace_event::ProcessMemoryDump facet ClientSharedImage::OnMemoryDump
/// writes into.
pub trait ResourcePoolMemoryDump<G> {
    fn CreateAllocatorDump(&mut self, name: &str) -> G;
    fn AddScalar(&mut self, guid: &G, name: &str, units: &str, value: u64);
}
// Source vocabulary: gpu/command_buffer/common/shared_image_trace_utils.h
// (TracingImportance::kClientOwner).
#[derive(Clone, Copy)]
pub enum SharedImageTracingImportance {
    ClientOwner,
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
    fn RasterReadLease(&self) -> Option<Arc<dyn RasterResourceLease>>;
    /// Maps the same allocation for exclusive host writes. Implementations must
    /// reject concurrent raster/read ownership rather than fabricate storage.
    fn MapRasterWrite(self: Arc<Self>) -> io::Result<Box<dyn RasterWriteMapping>>;
}

/// gpu::ClientSharedImage: one shared reference to the injected backend.
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
    pub fn RasterReadLease(&self) -> Option<Arc<dyn RasterResourceLease>> {
        self.0.RasterReadLease()
    }
    pub fn MapRasterWrite(&self) -> io::Result<Box<dyn RasterWriteMapping>> {
        Arc::clone(&self.0).MapRasterWrite()
    }
    pub fn EndImport(&self, token: &RasterSyncToken) -> RasterSyncToken {
        self.0.EndImport(token)
    }
    pub fn UpdateDestructionSyncToken(&self, token: &RasterSyncToken) {
        self.0.UpdateDestructionSyncToken(token);
    }
    pub fn OnMemoryDump(
        &self,
        dump: &mut dyn ResourcePoolMemoryDump<RasterDumpGuid>,
        guid: &RasterDumpGuid,
        importance: SharedImageTracingImportance,
    ) {
        self.0.OnMemoryDump(dump, guid, importance);
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
