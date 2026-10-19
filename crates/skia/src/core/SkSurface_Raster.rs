//! Restricted native N32 raster surface used by software UI resources. It
//! borrows WrapPixels storage directly and delegates drawing to the existing
//! SkScan/SkRasterPipeline; no compositor or scene state is present.
use crate::raster::geom::IntSizeExt;
use crate::raster::pipeline::RasterPipelineBlitter;
use crate::raster::pixmap::SubPixmapMut;
use crate::raster::{
    Color, IntSize, Paint, Pattern, Pixmap, PixmapRef, Rect, SpreadMode, Transform,
};

enum Storage<'a> {
    Borrowed(&'a mut [u8]),
    Owned(Vec<u8>),
}
impl Storage<'_> {
    fn bytes(&self) -> &[u8] {
        match self {
            Self::Borrowed(b) => b,
            Self::Owned(b) => b,
        }
    }
    fn bytes_mut(&mut self) -> &mut [u8] {
        match self {
            Self::Borrowed(b) => b,
            Self::Owned(b) => b,
        }
    }
}
pub struct N32Surface<'a> {
    storage: Storage<'a>,
    size: IntSize,
    row_bytes: usize,
    ctm: Transform,
}
fn byte_size(width: u32, height: u32, row_bytes: usize) -> Option<usize> {
    let row = (width as usize).checked_mul(4)?;
    if width == 0
        || height == 0
        || width > i32::MAX as u32
        || height > i32::MAX as u32
        || row_bytes < row
        || row_bytes % 4 != 0
    {
        return None;
    }
    (height as usize - 1)
        .checked_mul(row_bytes)?
        .checked_add(row)
        .filter(|n| *n <= isize::MAX as usize)
}
impl<'a> N32Surface<'a> {
    /// Source: src/image/SkSurface_Raster.cpp:191-208 (WrapPixels);
    /// SkImageInfo::computeByteSize keeps the last row at its active byte count.
    pub fn wrap(width: u32, height: u32, pixels: &'a mut [u8], row_bytes: usize) -> Option<Self> {
        let size = IntSize::from_wh(width, height)?;
        let len = byte_size(width, height, row_bytes)?;
        if pixels.len() < len {
            return None;
        }
        Some(Self {
            storage: Storage::Borrowed(&mut pixels[..len]),
            size,
            row_bytes,
            ctm: Transform::identity(),
        })
    }
    pub fn raster(width: u32, height: u32) -> Option<Self> {
        let size = IntSize::from_wh(width, height)?;
        let row_bytes = (width as usize).checked_mul(4)?;
        let len = byte_size(width, height, row_bytes)?;
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(len).ok()?;
        bytes.resize(len, 0);
        Some(Self {
            storage: Storage::Owned(bytes),
            size,
            row_bytes,
            ctm: Transform::identity(),
        })
    }
    pub fn pixels(&self) -> &[u8] {
        self.storage.bytes()
    }
    // Source: src/core/SkCanvas.cpp:1307-1323, matrix preScale.
    pub fn scale(&mut self, x: f32, y: f32) {
        self.ctm = self.ctm.pre_scale(x, y);
    }
    /// Source: src/core/SkCanvas.cpp:399-427; SkWritePixelsRec.cpp:12-59;
    /// SkConvertPixels.cpp:28-45 matching N32 premultiplied rect_memcpy branch. CTM
    /// does not apply to writePixels. Padding and out-of-intersection rows stay.
    pub fn write_pixels(
        &mut self,
        width: u32,
        height: u32,
        source: &[u8],
        source_row_bytes: usize,
        x: i32,
        y: i32,
    ) -> bool {
        if x == i32::MIN || y == i32::MIN {
            return false;
        }
        let Some(len) = byte_size(width, height, source_row_bytes) else {
            return false;
        };
        if source.len() < len {
            return false;
        }
        let left = (x as i64).max(0);
        let top = (y as i64).max(0);
        let right = (x as i64 + width as i64).min(self.size.width() as i64);
        let bottom = (y as i64 + height as i64).min(self.size.height() as i64);
        if left >= right || top >= bottom {
            return false;
        }
        let count = (right - left) as usize * 4;
        let dst_row_bytes = self.row_bytes;
        let target = self.storage.bytes_mut();
        for row in top..bottom {
            let src = (row - y as i64) as usize * source_row_bytes + (left - x as i64) as usize * 4;
            let dst = row as usize * dst_row_bytes + left as usize * 4;
            crate::src::core::SkConvertPixels::copy_same_info_pixels(
                &mut target[dst..dst + count],
                &source[src..src + count],
            );
        }
        true
    }
    fn fill_device_rect(&mut self, rect: Rect, paint: &Paint) {
        let mut destination = SubPixmapMut {
            data: self.storage.bytes_mut(),
            size: self.size,
            real_width: self.row_bytes / 4,
            format: crate::PixelFormat::Bgra8888,
        };
        let clip = self.size.to_screen_int_rect(0, 0);
        let Some(mut blitter) = RasterPipelineBlitter::new(paint, None, &mut destination) else {
            return;
        };
        crate::raster::scan::fill_rect(&rect, &clip, &mut blitter);
    }
    /// Source: include/core/SkCanvas.h clear -> drawColor(kSrc), in device space.
    pub fn clear_transparent(&mut self) {
        let Some(rect) =
            Rect::from_xywh(0., 0., self.size.width() as f32, self.size.height() as f32)
        else {
            return;
        };
        let mut paint = Paint::default();
        paint.set_color(Color::TRANSPARENT);
        paint.blend_mode = crate::BlendMode::kSrc;
        paint.anti_alias = false;
        self.fill_device_rect(rect, &paint);
    }
    /// Source: src/core/SkCanvas.cpp:2385-2396; SkBitmapDevice::drawImageRect.
    /// Native unpainted drawImage has SrcOver, nearest sampling and no AA.
    /// Source image bounds/local matrix are passed to SkImageShader directly.
    pub fn draw_image(&mut self, image: PixmapRef<'_>, x: f32, y: f32) {
        let Some(rect) = Rect::from_xywh(x, y, image.width() as f32, image.height() as f32)
            .and_then(|r| r.transform(self.ctm))
        else {
            return;
        };
        let paint = Paint {
            shader: Pattern::new_native(
                image,
                SpreadMode::Pad,
                SpreadMode::Pad,
                crate::raster::FilterQuality::Nearest,
                1.,
                Transform::from_translate(x, y).post_concat(self.ctm),
            ),
            blend_mode: crate::BlendMode::kSrcOver,
            anti_alias: false,
            ..Default::default()
        };
        self.fill_device_rect(rect, &paint);
    }
}
/// Source SkImages::RasterFromBitmap's copy-if-mutable branch. A source bitmap
/// borrowed with setPixels is mutable, so asImage obtains a real immutable image
/// allocation; it never retains a pointer into the caller's transient slice.
pub fn snapshot_n32(width: u32, height: u32, pixels: &[u8], row_bytes: usize) -> Option<Pixmap> {
    let len = byte_size(width, height, row_bytes)?;
    if pixels.len() < len {
        return None;
    }
    let packed = (width as usize).checked_mul(4)?;
    let mut result = Pixmap::new(width, height)?;
    // cc/trees/layer_tree_host_impl.cc:3531-3532: software N32 is BGRA8888.
    result.format = crate::PixelFormat::Bgra8888;
    for y in 0..height as usize {
        crate::src::core::SkConvertPixels::copy_same_info_pixels(
            &mut result.data_mut()[y * packed..(y + 1) * packed],
            &pixels[y * row_bytes..y * row_bytes + packed],
        );
    }
    Some(result)
}
