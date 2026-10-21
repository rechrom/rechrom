use super::{
    SkImage::SkImage,
    SkImage_Raster::{SkCopyPixelsMode, SkImage_Raster},
};
use crate::src::core::SkBitmap::SkBitmap;
use std::sync::Arc;
// Source: src/image/SkImage_RasterFactories.cpp:32-70
fn valid_args(
    info: &crate::src::core::SkImageInfo::SkImageInfo,
    row_bytes: usize,
    min_size: Option<&mut usize>,
) -> bool {
    let max_dimension = crate::include::private::SkMath::SK_MaxS32 >> 2;
    let mut bitmap = SkBitmap::default();
    if !bitmap.setInfo(info, row_bytes) {
        return false;
    }
    if info.width() <= 0 || info.height() <= 0 {
        return false;
    }
    if info.width() > max_dimension || info.height() > max_dimension {
        return false;
    }
    if info.colorType() as u32 > crate::include::core::SkColorType::kLastEnum_SkColorType as u32 {
        return false;
    }
    if info.alphaType() as u32 > crate::include::core::SkAlphaType::kLastEnum_SkAlphaType as u32 {
        return false;
    }
    if info.colorType() == crate::include::core::SkColorType::SkColorType::kUnknown_SkColorType {
        return false;
    }
    if !info.validRowBytes(row_bytes) {
        return false;
    }
    let size = info.computeByteSize(row_bytes);
    if crate::src::core::SkImageInfo::SkImageInfo::ByteSizeOverflowed(size) {
        return false;
    }
    if let Some(min_size) = min_size {
        *min_size = size;
    }
    true
}

// Source: src/image/SkImage_RasterFactories.cpp:73-79
/// Safety: backing pixels must satisfy MakeFromBitmap's complete retained-image
/// immutable lifetime/alias contract; mutable sources are copied by the native factory.
pub unsafe fn RasterFromBitmap(bitmap: &SkBitmap) -> Option<Arc<dyn SkImage>> {
    bitmap.pixelRef()?;
    unsafe { SkImage_Raster::MakeFromBitmap(bitmap, SkCopyPixelsMode::kIfMutable, None) }
        .map(|image| image as Arc<dyn SkImage>)
}

// Source: src/image/SkImage_RasterFactories.cpp:81-92
/// Safety: pixmap covers its native complete readable layout for the copy. The
/// returned image owns the actual copied allocation, independent of this borrow.
pub unsafe fn RasterFromPixmapCopy(
    pixmap: &crate::src::core::SkPixmap::SkPixmap,
) -> Option<Arc<dyn SkImage>> {
    let mut size = 0;
    if !valid_args(pixmap.info(), pixmap.rowBytes(), Some(&mut size)) || pixmap.addr().is_null() {
        return None;
    }
    let data =
        unsafe { crate::src::core::SkData::SkData::MakeWithCopy(pixmap.addr().cast(), size) };
    Some(Arc::new(unsafe {
        SkImage_Raster::from_data(pixmap.info(), data, pixmap.rowBytes(), None, 0)
    }))
}
// Source: src/image/SkImage_RasterFactories.cpp:94-107
/// Safety: data contains initialized native pixels with immutable retained-image
/// semantics; external writable aliases may not mutate while the image is live.
pub unsafe fn RasterFromData(
    info: &crate::src::core::SkImageInfo::SkImageInfo,
    data: Option<Arc<crate::src::core::SkData::SkData>>,
    row_bytes: usize,
) -> Option<Arc<dyn SkImage>> {
    let mut size = 0;
    if !valid_args(info, row_bytes, Some(&mut size)) {
        return None;
    }
    let data = data?;
    if data.size() < size {
        return None;
    }
    Some(Arc::new(unsafe {
        SkImage_Raster::from_data(info, data, row_bytes, None, 0)
    }))
}
// Source: src/image/SkImage_RasterFactories.cpp:108-125
/// Safety: src retains initialized immutable CPU pixels through the complete
/// filter graph and returned image; actual backend offscreen/draw leases hold.
/// out_subset and offset are exclusive output slots, as in the native contract.
pub unsafe fn MakeWithFilter(
    src: Option<Arc<dyn SkImage>>,
    filter: Option<&dyn crate::src::core::SkImageFilter::SkImageFilter>,
    subset: &crate::src::core::SkRect::SkIRect,
    clip_bounds: &crate::src::core::SkRect::SkIRect,
    out_subset: Option<&mut crate::src::core::SkRect::SkIRect>,
    offset: Option<&mut crate::src::core::SkPoint::SkIPoint>,
) -> Result<Option<Arc<dyn SkImage>>, crate::RasterError> {
    let (Some(src), Some(filter)) = (src, filter) else {
        return Ok(None);
    };
    let backend = crate::src::core::SkImageFilterTypes::MakeRasterBackend(
        &crate::include::core::SkSurfaceProps::SkSurfaceProps::default(),
        src.colorType(),
    )?;
    unsafe { filter.makeImageWithFilter(backend, src, subset, clip_bounds, out_subset, offset) }
}

// Source: src/image/SkImage_RasterFactories.cpp:130-159
/// Safety: data's retained bytes must satisfy SkData's immutable readable-storage
/// contract through this call; the returned image owns its decoded pixel allocation.
pub unsafe fn RasterFromCompressedTextureData(
    data: Option<Arc<crate::src::core::SkData::SkData>>,
    width: i32,
    height: i32,
    compression: crate::include::core::SkTextureCompressionType::SkTextureCompressionType,
) -> Option<Arc<dyn SkImage>> {
    use crate::include::core::{SkAlphaType::SkAlphaType, SkSize::SkISize};
    use crate::src::core::SkCompressedDataUtils::{
        SkCompressedFormatDataSize, SkDecompress, SkTextureCompressionTypeIsOpaque,
    };
    let expectedSize = SkCompressedFormatDataSize(compression, SkISize::Make(width, height), false);
    let data = data?;
    if data.size() < expectedSize {
        return None;
    }
    let at = if SkTextureCompressionTypeIsOpaque(compression) {
        SkAlphaType::kOpaque_SkAlphaType
    } else {
        SkAlphaType::kPremul_SkAlphaType
    };
    let ii = crate::src::core::SkImageInfo::SkImageInfo::MakeN32(width, height, at);
    if !valid_args(&ii, ii.minRowBytes(), None) {
        return None;
    }
    let mut bitmap = SkBitmap::default();
    if !bitmap.tryAllocPixels_info(&ii) {
        return None;
    }
    if !unsafe { SkDecompress(data, SkISize::Make(width, height), compression, &mut bitmap) } {
        return None;
    }
    bitmap.setImmutable();
    unsafe { RasterFromBitmap(&bitmap) }
}

// Source: src/image/SkImage_RasterFactories.cpp:161-170
/// Safety: pixels and release context remain valid/immutable through final image
/// release, and the procedure is valid for exactly one final allocation release.
pub unsafe fn RasterFromPixmap(
    pixmap: &crate::src::core::SkPixmap::SkPixmap,
    release: Option<crate::src::core::SkData::ReleaseProc>,
    context: *mut core::ffi::c_void,
) -> Option<Arc<dyn SkImage>> {
    let mut size = 0;
    if !valid_args(pixmap.info(), pixmap.rowBytes(), Some(&mut size)) || pixmap.addr().is_null() {
        return None;
    }
    let data = unsafe {
        crate::src::core::SkData::SkData::MakeWithProc(pixmap.addr().cast(), size, release, context)
    };
    Some(Arc::new(unsafe {
        SkImage_Raster::from_data(pixmap.info(), data, pixmap.rowBytes(), None, 0)
    }))
}
