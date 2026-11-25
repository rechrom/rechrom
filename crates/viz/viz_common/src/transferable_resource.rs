//! components/viz/common/resources/transferable_resource.h values.
use crate::raster_resource_lease::RasterResourceLease;
use crate::returned_resource::{RasterReturnedResource, RasterSyncToken};
use crate::shared_image::RasterSharedImage;
use crate::ResourceId;
use std::sync::Arc;

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
            sync_token: self.shared_image.EndImport(&self.sync_token),
            count: 1,
            lost: false,
        }
    }
    pub fn RasterReadLease(&self) -> Option<Arc<dyn RasterResourceLease>> {
        self.shared_image.RasterReadLease()
    }
    // Source: components/viz/common/resources/transferable_resource.h:45-62.
    pub fn SetGpuCommandsCompletedSynchronization(&mut self) {
        self.synchronization_type = RasterSynchronizationType::GpuCommandsCompleted;
    }
}
