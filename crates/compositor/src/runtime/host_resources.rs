//! Native UI/resource owners for HostImpl. Pixel storage and Skia surface
//! operations are provided by the external raster backend, never reimplemented
//! in the compositor. Each wrapper retains exactly the supplied native owner.
use super::{
    resources::{RasterColorSpace, RasterSharedImage, RasterSyncToken, RasterTransferableResource},
    types::{CpuCompositorTypes, CpuSkiaBackend},
};
use crate::adapter::{
    geometry::Size,
    layer_tree_host_impl::{
        HostImplResourceTypes, HostUIBitmapFormat, HostUIResourceBitmap, HostUIResourceImageInfo,
        HostUIResourceTypes, HostUISharedMapping,
    },
};
use std::sync::Arc;

/// A default native WritableSharedMemoryMapping is invalid. A valid mapping is
/// the actual external allocation and stays exclusively owned until dropped.
pub trait SharedMemoryMappingBackend {
    fn memory(&self) -> &[u8];
    fn memory_mut(&mut self) -> &mut [u8];
}
#[derive(Default)]
pub struct SharedMemoryMapping(pub Option<Box<dyn SharedMemoryMappingBackend>>);
#[derive(Clone)]
pub struct UIBitmap(pub Arc<dyn HostUIResourceBitmap + Send + Sync>);
impl HostUIResourceBitmap for UIBitmap {
    fn GetFormat(&self) -> HostUIBitmapFormat {
        self.0.GetFormat()
    }
    fn GetSize(&self) -> Size {
        self.0.GetSize()
    }
    fn GetOpaque(&self) -> bool {
        self.0.GetOpaque()
    }
    fn GetPixels(&self) -> &[u8] {
        self.0.GetPixels()
    }
    fn row_bytes(&self) -> usize {
        self.0.row_bytes()
    }
}

pub struct RawUIBitmap {
    size: Size,
    pixels: Vec<u8>,
    row_bytes: usize,
    opaque: bool,
}
impl RawUIBitmap {
    pub fn new(size: Size, pixels: Vec<u8>, row_bytes: usize, opaque: bool) -> Self {
        debug_assert_eq!(pixels.len(), row_bytes * size.height().max(0) as usize);
        Self {
            size,
            pixels,
            row_bytes,
            opaque,
        }
    }
}
impl HostUIResourceBitmap for RawUIBitmap {
    fn GetFormat(&self) -> HostUIBitmapFormat {
        HostUIBitmapFormat::RGBA8
    }
    fn GetSize(&self) -> Size {
        self.size
    }
    fn GetOpaque(&self) -> bool {
        self.opaque
    }
    fn GetPixels(&self) -> &[u8] {
        &self.pixels
    }
    fn row_bytes(&self) -> usize {
        self.row_bytes
    }
}
// The UI-resource raster values the injected raster service produces
// (cc_raster::ui_resource); the image-info facet is LayerTreeHostImpl's.
pub use cc_raster::ui_resource::{SourceBitmap, UICanvas, UIImage, UIImageInfo, UISurface, UiImage};
impl HostUIResourceImageInfo for UIImageInfo {
    fn minRowBytes(&self) -> usize {
        self.min_row_bytes
    }
}
/// Native UI plane projection of the same exclusively mapped raster backing.
/// This adapter owns only the external write lease; dropping it publishes the
/// pixels through that lease's allocation owner, without a compositor cache.
struct RasterSharedWriteMapping(Box<dyn super::resources::RasterWriteMapping>);
impl HostUISharedMapping for RasterSharedWriteMapping {
    fn GetMemoryForPlane(&mut self, plane: usize) -> &mut [u8] {
        assert_eq!(
            plane, 0,
            "native N32 UI shared images have exactly one plane"
        );
        self.0.bytes_mut()
    }
}
pub struct SharedMapping(pub Box<dyn HostUISharedMapping>);
impl SharedMapping {
    pub fn FromRasterWriteMapping(mapping: Box<dyn super::resources::RasterWriteMapping>) -> Self {
        Self(Box::new(RasterSharedWriteMapping(mapping)))
    }
}
impl HostUISharedMapping for SharedMapping {
    fn GetMemoryForPlane(&mut self, plane: usize) -> &mut [u8] {
        self.0.GetMemoryForPlane(plane)
    }
}
impl<R: CpuSkiaBackend> HostImplResourceTypes for CpuCompositorTypes<R> {
    type SharedMemoryMapping = SharedMemoryMapping;
    type SharedImage = RasterSharedImage;
    type ResourceId = crate::adapter::layer_tree_impl::ResourceId;
}
impl<R: CpuSkiaBackend> HostUIResourceTypes for CpuCompositorTypes<R> {
    type UIBitmap = UIBitmap;
    type UISyncToken = RasterSyncToken;
    type UIColorSpace = RasterColorSpace;
    type UIImageInfo = UIImageInfo;
    type UIImage = UIImage;
    type UISourceBitmap<'a> = SourceBitmap<'a>;
    type UIScopedMapping = SharedMapping;
    type UISurface<'a> = UISurface<'a>;
    type UITransferableResource = RasterTransferableResource;
}
