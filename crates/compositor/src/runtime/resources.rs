//! Concrete external resource values used by the native compositor family.
//!
//! The cc resource pool owns reuse, accounting and release policy. The
//! transferable, returned and shared-image values are components/viz/common
//! (`viz_common`); the actual shared image remains an injected platform object
//! behind `SharedImageBackend`. This module does not allocate pixels or
//! implement a second resource pool.

use super::types::{CpuCompositorTypes, CpuSkiaBackend};
use crate::adapter::{
    geometry::Size,
    layer_tree_host_impl::{ContentColorUsage, HostDisplayColorSpaces},
    resource_pool::ResourcePoolTypes,
    resource_types::SharedImageFormat,
};

// Source vocabulary: ui/gfx/color_space.h:50-153,413-429.
pub use layerize::commit::color_space::{
    ColorMatrixId, ColorPrimaryId, ColorRangeId, ColorTransferId, RasterColorSpace,
};
// ui/gfx/display_color_spaces.h (a cc/viz shared value).
pub use compositor_values::display_color_spaces::RasterDisplayColorSpaces;

impl HostDisplayColorSpaces<RasterColorSpace> for RasterDisplayColorSpaces {
    fn GetRasterAndCompositeColorSpace(&self, usage: ContentColorUsage) -> RasterColorSpace {
        RasterDisplayColorSpaces::GetRasterAndCompositeColorSpace(self, usage)
    }
    fn GetHdrHeadroom(&self) -> f32 {
        RasterDisplayColorSpaces::GetHdrHeadroom(self)
    }
}

// Sync-token, returned-resource, shared-image and transferable values are viz
// values; their resource-pool facets live in compositor_resource.
pub use viz_common::returned_resource::{
    CommandBufferNamespace, RasterReturnedResource, RasterSyncToken,
};
pub use viz_common::shared_image::{
    RasterBufferUsage, RasterDumpGuid, RasterSharedImage, RasterWriteMapping, SharedImageBackend,
};
pub use viz_common::transferable_resource::{
    RasterResourceSource, RasterSynchronizationType, RasterTransferableResource,
};

impl crate::adapter::layer_tree_host_impl::HostUISharedImage<RasterSyncToken>
    for RasterSharedImage
{
    fn creation_sync_token(&self) -> RasterSyncToken {
        RasterSharedImage::creation_sync_token(self)
    }
    fn UpdateDestructionSyncToken(&mut self, token: &RasterSyncToken) {
        RasterSharedImage::UpdateDestructionSyncToken(self, token);
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
