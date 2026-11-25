//! The UI-resource raster capability LayerTreeHostImpl::CreateUIResource
//! drives (cc/trees/layer_tree_host_impl.cc CreateUIResource over
//! cc/resources/ui_resource_bitmap.h): the SkBitmap/SkCanvas/SkSurface
//! primitives and the shared-image allocation the injected raster service
//! performs. No UI resource selection, sizing, lifetime or host policy is
//! implemented here; those remain in LayerTreeHostImpl.
use compositor_tree::geometry::Size;
use compositor_values::color_space::RasterColorSpace;
use compositor_values::shared_image_format::SharedImageFormat;
use std::{any::Any, sync::Arc};
use viz_common::shared_image::RasterSharedImage;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HostUISharedImageUsage(u32);
// Source dependency: gpu/command_buffer/common/shared_image_usage.h:25,29,50.
impl HostUISharedImageUsage {
    pub const DISPLAY_READ: Self = Self(1 << 2);
    pub const CPU_WRITE_ONLY: Self = Self(1 << 11);
    pub const SCANOUT: Self = Self(1 << 4);
    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}
impl std::ops::BitOrAssign for HostUISharedImageUsage {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}
pub trait HostUISourceBitmap<'a, Info, Image> {
    fn setInfo(&mut self, info: Info, row_bytes: usize);
    fn setPixels(&mut self, pixels: &'a [u8]);
    fn asImage(&self) -> Image;
}
pub trait HostUICanvas<Info, Image> {
    // Source dependency: third_party/skia/include/core/SkCanvas.h:384-396.
    fn peekPixels(&self) -> HostUIPixmap<'_>;
    fn writePixels(&mut self, info: &Info, pixels: &[u8], row_bytes: usize, x: i32, y: i32);
    fn scale(&mut self, x: f32, y: f32);
    fn clearTransparent(&mut self);
    fn drawImage(&mut self, image: &Image, x: f32, y: f32);
}
/// Borrowed native SkPixmap-to-span result. The surface owns its pixels.
pub struct HostUIPixmap<'a> {
    pub pixels: &'a [u8],
}
pub trait HostUISurface<Info, Image> {
    type Canvas: HostUICanvas<Info, Image>;
    fn getCanvas(&mut self) -> &mut Self::Canvas;
    fn peekPixels(&self) -> HostUIPixmap<'_>;
}

/// This file only uses native SkImageInfo::MakeN32Premul. The external Skia
/// factory supplies its actual dimensions and minRowBytes value, without a
/// compositor-side bytes-per-pixel or allocation calculation.
#[derive(Clone, Debug)]
pub struct UIImageInfo {
    pub size: Size,
    pub min_row_bytes: usize,
}
/// Opaque image owner produced and consumed by the injected raster service.
/// Compositor retains identity and lifetime only; concrete Skia image types stay
/// inside the raster package.
pub trait UiImage: Any {
    fn as_any(&self) -> &dyn Any;
}
pub type UIImage = Arc<dyn UiImage>;
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

/// The raster allocation owner implements this capability. Borrowed
/// surfaces must retain the exact caller pixel slice and its stride; the trait
/// does not permit a compositor-side pixel copy or fabricated image/surface.
pub trait UiRasterServices: Send + Sync {
    fn MakeN32Premul(&self, size: Size) -> UIImageInfo;
    fn NewSourceBitmap<'a>(&self) -> SourceBitmap<'a>;
    fn WrapPixels<'a>(
        &self,
        info: UIImageInfo,
        pixels: &'a mut [u8],
        row_bytes: usize,
    ) -> Option<UISurface<'a>>;
    fn RasterSurface<'a>(&self, info: UIImageInfo) -> Option<UISurface<'a>>;
    fn CreateSoftwareSharedImage(
        &self,
        format: SharedImageFormat,
        size: Size,
        color: &RasterColorSpace,
        usage: HostUISharedImageUsage,
        label: &'static str,
    ) -> Option<RasterSharedImage>;
    fn CreateSharedImageWithPixels(
        &self,
        format: SharedImageFormat,
        size: Size,
        color: &RasterColorSpace,
        usage: HostUISharedImageUsage,
        label: &'static str,
        pixels: &[u8],
    ) -> Option<RasterSharedImage>;
}
