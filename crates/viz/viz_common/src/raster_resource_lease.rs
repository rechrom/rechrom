//! Read access to one exported raster resource, as Viz's resource provider
//! maps an imported TransferableResource for the display renderer
//! (components/viz/service/display/display_resource_provider.h ScopedReadLock
//! over the gpu::ClientSharedImage the resource names).
use crate::ResourceId;
use compositor_tree::geometry::Size;
use compositor_values::color_space::RasterColorSpace;
use compositor_values::shared_image_format::SharedImageFormat;
use std::any::Any;

/// Opaque read lease to one immutable raster resource. The compositor and Viz
/// retain identity and lifetime only; a concrete renderer asks the raster
/// package to map the backend payload it understands.
pub trait RasterResourceLease: Send + Sync {
    fn resource_id(&self) -> ResourceId;
    fn size(&self) -> Size;
    fn format(&self) -> SharedImageFormat;
    fn is_opaque(&self) -> bool;
    /// Resource color identity used by Viz/Renderer. Tile resources default to
    /// sRGB; multiplanar video overrides this with the decoded gfx::ColorSpace.
    fn color_space(&self) -> RasterColorSpace {
        RasterColorSpace::CreateSRGB()
    }
    fn backend_resource(&self) -> &(dyn Any + Send + Sync);
}
