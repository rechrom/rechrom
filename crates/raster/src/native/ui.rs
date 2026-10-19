//! Thin native UI resource binding. HostImpl retains UI sizing/lifecycle
//! policy; Skia owns N32 copy, scale, clear and image sampling primitives.
use super::*;
use compositor_core::{
    adapter::layer_tree_host_impl::{
        HostUICanvas, HostUIPixmap, HostUISharedImageUsage, HostUISourceBitmap, HostUISurface,
    },
    runtime::{
        host_resources::{SourceBitmap, UICanvas, UIImage, UIImageInfo, UISurface},
        native_ui::NativeUiRasterServices,
        resources::CpuColorSpace,
    },
};
use skia::src::core::SkSurface_Raster::{snapshot_n32, N32Surface};

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
            .expect("native source bitmap setInfo precedes asImage");
        Arc::new(
            snapshot_n32(
                info.size.width() as u32,
                info.size.height() as u32,
                self.pixels
                    .expect("native source bitmap setPixels precedes asImage"),
                self.row_bytes,
            )
            .expect("validated N32 source bitmap metadata"),
        )
    }
}
struct Canvas<'a>(N32Surface<'a>);
impl HostUICanvas<UIImageInfo, UIImage> for Canvas<'_> {
    fn writePixels(&mut self, info: &UIImageInfo, pixels: &[u8], row_bytes: usize, x: i32, y: i32) {
        self.0.write_pixels(
            info.size.width() as u32,
            info.size.height() as u32,
            pixels,
            row_bytes,
            x,
            y,
        );
    }
    fn scale(&mut self, x: f32, y: f32) {
        self.0.scale(x, y);
    }
    fn clearTransparent(&mut self) {
        self.0.clear_transparent();
    }
    fn drawImage(&mut self, image: &UIImage, x: f32, y: f32) {
        self.0.draw_image(image.as_ref().as_ref(), x, y);
    }
    fn peekPixels(&self) -> HostUIPixmap<'_> {
        HostUIPixmap {
            pixels: self.0.pixels(),
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
fn surface<'a>(value: N32Surface<'a>) -> UISurface<'a> {
    UISurface(Box::new(Surface {
        canvas: UICanvas(Box::new(Canvas(value))),
    }))
}
impl NativeUiRasterServices for CpuRasterProvider {
    fn MakeN32Premul(&self, size: Size) -> UIImageInfo {
        // SkImageInfo::MakeN32Premul / minRowBytes (valid source dimensions).
        UIImageInfo {
            size,
            min_row_bytes: (size.width().max(0) as usize).checked_mul(4).unwrap_or(0),
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
        if info.size.width() <= 0 || info.size.height() <= 0 {
            return None;
        }
        N32Surface::wrap(
            info.size.width() as u32,
            info.size.height() as u32,
            pixels,
            row_bytes,
        )
        .map(surface)
    }
    fn RasterSurface<'a>(&self, info: UIImageInfo) -> Option<UISurface<'a>> {
        if info.size.width() <= 0 || info.size.height() <= 0 {
            return None;
        }
        N32Surface::raster(info.size.width() as u32, info.size.height() as u32).map(surface)
    }
    fn CreateSoftwareSharedImage(
        &self,
        format: SharedImageFormat,
        size: Size,
        _color: &CpuColorSpace,
        _usage: HostUISharedImageUsage,
        _label: &'static str,
    ) -> Option<CpuSharedImage> {
        self.CreateUiSharedImage(size, &format).ok()
    }
    fn CreateSharedImageWithPixels(
        &self,
        format: SharedImageFormat,
        size: Size,
        _color: &CpuColorSpace,
        _usage: HostUISharedImageUsage,
        _label: &'static str,
        pixels: &[u8],
    ) -> Option<CpuSharedImage> {
        let image = self.CreateUiSharedImage(size, &format).ok()?;
        let mut mapping = image.MapCpuWrite().ok()?;
        let target = mapping.bytes_mut();
        if target.len() != pixels.len() {
            return None;
        }
        // The source SharedImage-with-initial-data API supplies bytes already in
        // the declared native format. This is an actual upload, not a snapshot
        // retained alongside the shared image or a compositor-side pixel path.
        target.copy_from_slice(pixels);
        drop(mapping);
        Some(image)
    }
}
