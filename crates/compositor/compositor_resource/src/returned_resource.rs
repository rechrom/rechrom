//! Sync tokens and returned-resource values exchanged with Viz
//! (gpu/command_buffer/common/sync_token.h,
//! components/viz/common/resources/returned_resource.h).
use crate::client_resource_lifecycle::ReturnedClientResource;
use crate::resource_pool_boundary::ResourceSyncToken;
use crate::ResourceId;

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

// Source value: components/viz/common/resources/returned_resource.h:25-60.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RasterReturnedResource {
    pub id: ResourceId,
    pub sync_token: RasterSyncToken,
    pub count: i32,
    pub lost: bool,
}
impl ReturnedClientResource<RasterSyncToken> for RasterReturnedResource {
    fn Id(&self) -> ResourceId {
        self.id
    }
    fn Count(&self) -> i32 {
        self.count
    }
    fn Lost(&self) -> bool {
        self.lost
    }
    fn SyncToken(&self) -> &RasterSyncToken {
        &self.sync_token
    }
}
