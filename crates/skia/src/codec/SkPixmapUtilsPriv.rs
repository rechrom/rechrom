use crate::include::codec::SkEncodedOrigin::*;
use crate::src::core::{SkAutoPixmapStorage::SkAutoPixmapStorage, SkPixmap::SkPixmap};
// Source: src/codec/SkPixmapUtilsPriv.h:30-51
/// Safety: decode initializes every pixel subsequently read by Orient. The original
/// destination remains exclusively writable and all descriptors obey native leases.
pub unsafe fn OrientWithDecode<F>(
    dst: &SkPixmap,
    origin: SkEncodedOrigin,
    decode: F,
) -> Result<bool, crate::RasterError>
where
    F: FnOnce(&SkPixmap) -> bool,
{
    let mut storage = SkAutoPixmapStorage::new();
    let mut tmp = dst;
    if origin != kTopLeft_SkEncodedOrigin {
        let mut info = dst.info().clone();
        if SkEncodedOriginSwapsWidthHeight(origin) {
            info = super::SkPixmapUtils::SwapWidthHeight(&info);
        }
        if !storage.tryAlloc(&info) {
            return Ok(false);
        }
        tmp = unsafe { storage.pixmap() };
    }
    if !decode(tmp) {
        return Ok(false);
    }
    if !core::ptr::eq(tmp, dst) {
        return unsafe { super::SkPixmapUtils::Orient(dst, tmp, origin) };
    }
    Ok(true)
}
