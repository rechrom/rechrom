//! UI resource drawing through the translated SkCanvas backend. HostImpl owns
//! sizing and lifecycle policy; this module owns concrete image/surface values.
use super::*;
use compositor_core::{
    adapter::layer_tree_host_impl::{
        HostUICanvas, HostUIPixmap, HostUISharedImageUsage, HostUISourceBitmap, HostUISurface,
    },
    runtime::{
        host_resources::{NativeUiImage, SourceBitmap, UICanvas, UIImage, UIImageInfo, UISurface},
        native_ui::NativeUiRasterServices,
        resources::RasterColorSpace,
    },
};
use skia::{
    include::core::SkSamplingOptions::SkSamplingOptions,
    raster_target::{PixelFormat, RasterTarget},
    src::{
        core::{
            SkCanvas::SrcRectConstraint, SkImageInfo::SkImageInfo, SkPixmap::SkPixmap,
            SkRect::SkRect,
        },
        image::{SkImage::SkImage, SkImage_RasterFactories::RasterFromPixmapCopy},
    },
    RasterError,
};
use std::sync::Arc;

struct TranslatedImage(Arc<dyn SkImage>);
impl NativeUiImage for TranslatedImage {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
fn translated_image(image: &UIImage) -> &Arc<dyn SkImage> {
    &image
        .as_any()
        .downcast_ref::<TranslatedImage>()
        .expect("translated UI canvas received its own image type")
        .0
}
fn image_info(size: Size) -> SkImageInfo {
    SkImageInfo::MakeN32Premul(size.width(), size.height())
}

struct Source<'a> {
    info: Option<UIImageInfo>,
    pixels: Option<&'a [u8]>,
    row_bytes: usize,
}
impl<'a> HostUISourceBitmap<'a, UIImageInfo, UIImage> for Source<'a> {
    fn setInfo(&mut self, info: UIImageInfo, row_bytes: usize) {
        self.info = Some(info);
        self.row_bytes = row_bytes;
        self.pixels = None;
    }
    fn setPixels(&mut self, pixels: &'a [u8]) {
        self.pixels = Some(pixels);
    }
    fn asImage(&self) -> UIImage {
        let info = self
            .info
            .as_ref()
            .expect("translated source bitmap setInfo precedes asImage");
        let pixels = self
            .pixels
            .expect("translated source bitmap setPixels precedes asImage");
        let native_info = image_info(info.size);
        let pixmap = SkPixmap::new(&native_info, pixels.as_ptr(), self.row_bytes);
        let image = unsafe { RasterFromPixmapCopy(&pixmap) }
            .expect("validated translated N32 source bitmap metadata");
        Arc::new(TranslatedImage(image))
    }
}

enum Storage<'a> {
    Borrowed(&'a mut [u8]),
    Owned(Vec<u8>),
}
impl Storage<'_> {
    fn bytes(&self) -> &[u8] {
        match self {
            Self::Borrowed(bytes) => bytes,
            Self::Owned(bytes) => bytes,
        }
    }
    fn bytes_mut(&mut self) -> &mut [u8] {
        match self {
            Self::Borrowed(bytes) => bytes,
            Self::Owned(bytes) => bytes,
        }
    }
}

struct Canvas<'a> {
    storage: Storage<'a>,
    size: Size,
    row_bytes: usize,
    scale_x: f32,
    scale_y: f32,
}
impl Canvas<'_> {
    fn with_target<T>(
        &mut self,
        operation: impl FnOnce(&mut RasterTarget<'_>) -> Result<T, RasterError>,
    ) -> T {
        let mut target = RasterTarget::new(
            self.storage.bytes_mut(),
            self.size.width(),
            self.size.height(),
            self.row_bytes,
            PixelFormat::Bgra8888,
            None,
        )
        .expect("validated translated UI surface layout");
        target.scale(self.scale_x, self.scale_y);
        let value = operation(&mut target).expect("translated UI raster operation must succeed");
        target
            .finish()
            .expect("translated UI surface finalization must succeed");
        value
    }
}
impl HostUICanvas<UIImageInfo, UIImage> for Canvas<'_> {
    fn writePixels(&mut self, info: &UIImageInfo, pixels: &[u8], row_bytes: usize, x: i32, y: i32) {
        let source_info = image_info(info.size);
        self.with_target(|target| unsafe {
            target.write_pixels(&source_info, pixels, row_bytes, x, y)
        });
    }
    fn scale(&mut self, x: f32, y: f32) {
        self.scale_x *= x;
        self.scale_y *= y;
    }
    fn clearTransparent(&mut self) {
        self.with_target(|target| target.clear(0));
    }
    fn drawImage(&mut self, image: &UIImage, x: f32, y: f32) {
        let image = translated_image(image);
        let destination = SkRect::MakeXYWH(x, y, image.width() as f32, image.height() as f32);
        self.with_target(|target| unsafe {
            target.draw_image_rect(
                image,
                None,
                &destination,
                &SkSamplingOptions::default(),
                None,
                SrcRectConstraint::kFast_SrcRectConstraint,
            )
        });
    }
    fn peekPixels(&self) -> HostUIPixmap<'_> {
        HostUIPixmap {
            pixels: self.storage.bytes(),
        }
    }
}

struct Surface<'a> {
    canvas: UICanvas<'a>,
}
impl<'a> HostUISurface<UIImageInfo, UIImage> for Surface<'a> {
    type Canvas = UICanvas<'a>;
    fn getCanvas(&mut self) -> &mut Self::Canvas {
        &mut self.canvas
    }
    fn peekPixels(&self) -> HostUIPixmap<'_> {
        self.canvas.peekPixels()
    }
}
fn surface<'a>(storage: Storage<'a>, size: Size, row_bytes: usize) -> UISurface<'a> {
    UISurface(Box::new(Surface {
        canvas: UICanvas(Box::new(Canvas {
            storage,
            size,
            row_bytes,
            scale_x: 1.0,
            scale_y: 1.0,
        })),
    }))
}

impl NativeUiRasterServices for RasterEngine {
    fn MakeN32Premul(&self, size: Size) -> UIImageInfo {
        let info = image_info(size);
        UIImageInfo {
            size,
            min_row_bytes: info.minRowBytes(),
        }
    }
    fn NewSourceBitmap<'a>(&self) -> SourceBitmap<'a> {
        SourceBitmap(Box::new(Source {
            info: None,
            pixels: None,
            row_bytes: 0,
        }))
    }
    fn WrapPixels<'a>(
        &self,
        info: UIImageInfo,
        pixels: &'a mut [u8],
        row_bytes: usize,
    ) -> Option<UISurface<'a>> {
        let target = RasterTarget::new(
            pixels,
            info.size.width(),
            info.size.height(),
            row_bytes,
            PixelFormat::Bgra8888,
            None,
        )
        .ok()?;
        target.finish().ok()?;
        Some(surface(Storage::Borrowed(pixels), info.size, row_bytes))
    }
    fn RasterSurface<'a>(&self, info: UIImageInfo) -> Option<UISurface<'a>> {
        if info.size.width() <= 0 || info.size.height() <= 0 {
            return None;
        }
        let row_bytes = image_info(info.size).minRowBytes();
        let len = row_bytes.checked_mul(info.size.height() as usize)?;
        let pixels = vec![0; len];
        Some(surface(Storage::Owned(pixels), info.size, row_bytes))
    }
    fn CreateSoftwareSharedImage(
        &self,
        format: SharedImageFormat,
        size: Size,
        _color: &RasterColorSpace,
        _usage: HostUISharedImageUsage,
        _label: &'static str,
    ) -> Option<RasterSharedImage> {
        self.CreateUiSharedImage(size, &format).ok()
    }
    fn CreateSharedImageWithPixels(
        &self,
        format: SharedImageFormat,
        size: Size,
        _color: &RasterColorSpace,
        _usage: HostUISharedImageUsage,
        _label: &'static str,
        pixels: &[u8],
    ) -> Option<RasterSharedImage> {
        if super::pixel_len(size).ok()? != pixels.len() {
            return None;
        }
        let image = self.CreateUiSharedImage(size, &format).ok()?;
        let mut mapping = image.MapRasterWrite().ok()?;
        let target = mapping.bytes_mut();
        debug_assert_eq!(target.len(), pixels.len());
        target.copy_from_slice(pixels);
        drop(mapping);
        Some(image)
    }
}
