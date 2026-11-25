//! The cc resource facets of the viz sync-token and returned-resource values
//! (`viz_common::returned_resource`).
pub use viz_common::returned_resource::{
    CommandBufferNamespace, RasterReturnedResource, RasterSyncToken,
};

use viz_common::shared_image::{RasterDumpGuid, RasterSharedImage};
use viz_common::transferable_resource::RasterTransferableResource;

use crate::client_resource_lifecycle::ReturnedClientResource;
use crate::resource_pool_boundary::ResourceSyncToken;
use crate::ResourceId;

impl ResourceSyncToken for RasterSyncToken {
    // Source: gpu/command_buffer/common/sync_token.h:66-68.
    fn HasData(&self) -> bool {
        RasterSyncToken::HasData(self)
    }
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

// The resource-pool facets of the viz shared-image and transferable values.
impl crate::resource_pool_boundary::PoolSharedImage<RasterSyncToken, RasterDumpGuid>
    for RasterSharedImage
{
    fn UpdateDestructionSyncToken(&self, token: &RasterSyncToken) {
        RasterSharedImage::UpdateDestructionSyncToken(self, token);
    }
    fn OnMemoryDump(
        &self,
        dump: &mut dyn crate::resource_pool_boundary::ResourcePoolMemoryDump<RasterDumpGuid>,
        guid: &RasterDumpGuid,
        importance: crate::resource_pool_boundary::SharedImageTracingImportance,
    ) {
        RasterSharedImage::OnMemoryDump(self, dump, guid, importance);
    }
}
impl crate::resource_pool_boundary::PoolTransferableResource for RasterTransferableResource {
    // Source: components/viz/common/resources/transferable_resource.h:45-62.
    fn SetGpuCommandsCompletedSynchronization(&mut self) {
        RasterTransferableResource::SetGpuCommandsCompletedSynchronization(self);
    }
}
