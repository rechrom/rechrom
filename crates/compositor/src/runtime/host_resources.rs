//! Native UI/resource owners for HostImpl. Pixel storage and Skia surface
//! operations are provided by the external raster backend, never reimplemented
//! in the compositor. Each wrapper retains exactly the supplied native owner.
use super::{
    resources::{CpuColorSpace, CpuSharedImage, CpuSyncToken, CpuTransferableResource},
    types::{CpuCompositorTypes, CpuSkiaBackend},
};
use crate::adapter::{
    geometry::Size,
    layer_tree_host_impl::{
        HostImplResourceTypes, HostUIBitmapFormat, HostUICanvas, HostUIPixmap,
        HostUIResourceBitmap, HostUIResourceImageInfo, HostUIResourceTypes, HostUISharedMapping,
        HostUISourceBitmap, HostUISurface,
    },
};
use std::sync::Arc;

/// A default native WritableSharedMemoryMapping is invalid. A valid mapping is
/// the actual external allocation and stays exclusively owned until dropped.
pub trait NativeSharedMemoryMapping {
    fn memory(&self) -> &[u8];
    fn memory_mut(&mut self) -> &mut [u8];
}
#[derive(Default)]
pub struct SharedMemoryMapping(pub Option<Box<dyn NativeSharedMemoryMapping>>);
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
/// This file only uses native SkImageInfo::MakeN32Premul. The external Skia
/// factory supplies its actual dimensions and minRowBytes value, without a
/// compositor-side bytes-per-pixel or allocation calculation.
#[derive(Clone, Debug)]
pub struct UIImageInfo {
    pub size: Size,
    pub min_row_bytes: usize,
}
impl HostUIResourceImageInfo for UIImageInfo {
    fn minRowBytes(&self) -> usize {
        self.min_row_bytes
    }
}
/// The concrete CPU image value is the actual external Skia allocation.
pub type UIImage = Arc<skia::raster::Pixmap>;
pub struct SourceBitmap<'a>(pub Box<dyn HostUISourceBitmap<'a, UIImageInfo, UIImage> + 'a>);
impl<'a> HostUISourceBitmap<'a, UIImageInfo, UIImage> for SourceBitmap<'a> {
    fn setInfo(&mut self, info: UIImageInfo, row_bytes: usize) {
        self.0.setInfo(info, row_bytes)
    }
    fn setPixels(&mut self, pixels: &'a [u8]) {
        self.0.setPixels(pixels)
    }
    fn asImage(&self) -> UIImage {
        self.0.asImage()
    }
}
pub struct UICanvas<'a>(pub Box<dyn HostUICanvas<UIImageInfo, UIImage> + 'a>);
impl HostUICanvas<UIImageInfo, UIImage> for UICanvas<'_> {
    fn peekPixels(&self) -> HostUIPixmap<'_> {
        self.0.peekPixels()
    }
    fn writePixels(&mut self, info: &UIImageInfo, pixels: &[u8], row_bytes: usize, x: i32, y: i32) {
        self.0.writePixels(info, pixels, row_bytes, x, y)
    }
    fn scale(&mut self, x: f32, y: f32) {
        self.0.scale(x, y)
    }
    fn clearTransparent(&mut self) {
        self.0.clearTransparent()
    }
    fn drawImage(&mut self, image: &UIImage, x: f32, y: f32) {
        self.0.drawImage(image, x, y)
    }
}
pub struct UISurface<'a>(
    pub Box<dyn HostUISurface<UIImageInfo, UIImage, Canvas = UICanvas<'a>> + 'a>,
);
impl<'a> HostUISurface<UIImageInfo, UIImage> for UISurface<'a> {
    type Canvas = UICanvas<'a>;
    fn getCanvas(&mut self) -> &mut Self::Canvas {
        self.0.getCanvas()
    }
    fn peekPixels(&self) -> HostUIPixmap<'_> {
        self.0.peekPixels()
    }
}
/// Native UI plane projection of the same exclusively mapped CPU backing.
/// This adapter owns only the external write lease; dropping it publishes the
/// pixels through that lease's allocation owner, without a compositor cache.
struct CpuSharedWriteMapping(Box<dyn super::resources::CpuWriteMapping>);
impl HostUISharedMapping for CpuSharedWriteMapping {
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
    pub fn FromCpuWriteMapping(mapping: Box<dyn super::resources::CpuWriteMapping>) -> Self {
        Self(Box::new(CpuSharedWriteMapping(mapping)))
    }
}
impl HostUISharedMapping for SharedMapping {
    fn GetMemoryForPlane(&mut self, plane: usize) -> &mut [u8] {
        self.0.GetMemoryForPlane(plane)
    }
}
impl<R: CpuSkiaBackend> HostImplResourceTypes for CpuCompositorTypes<R> {
    type SharedMemoryMapping = SharedMemoryMapping;
    type SharedImage = CpuSharedImage;
    type ResourceId = crate::adapter::layer_tree_impl::ResourceId;
}
impl<R: CpuSkiaBackend> HostUIResourceTypes for CpuCompositorTypes<R> {
    type UIBitmap = UIBitmap;
    type UISyncToken = CpuSyncToken;
    type UIColorSpace = CpuColorSpace;
    type UIImageInfo = UIImageInfo;
    type UIImage = UIImage;
    type UISourceBitmap<'a> = SourceBitmap<'a>;
    type UIScopedMapping = SharedMapping;
    type UISurface<'a> = UISurface<'a>;
    type UITransferableResource = CpuTransferableResource;
}
