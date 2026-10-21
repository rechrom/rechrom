use super::SkData::SkData;
use super::SkImageInfo::SkImageInfo;
use super::SkPixelRef::{SkMakePixelRefWithProc, SkPixelRef};
use std::ffi::c_void;
use std::sync::Arc;
// Source: src/core/SkMallocPixelRef.cpp:19-27
fn is_valid(info: &SkImageInfo) -> bool {
    if info.width() < 0
        || info.height() < 0
        || info.colorType() as u32 > crate::include::core::SkColorType::kLastEnum_SkColorType as u32
        || info.alphaType() as u32 > crate::include::core::SkAlphaType::kLastEnum_SkAlphaType as u32
    {
        return false;
    }
    true
}
// Source: src/core/SkMallocPixelRef.cpp:29-31
// Source: src/core/SkMallocPixelRef.cpp:34-41
// Source: src/core/SkMallocPixelRef.cpp:47-50
// Source: src/core/SkMallocPixelRef.cpp:52-57
pub fn MakeAllocate(info: &SkImageInfo, mut rowBytes: usize) -> Option<Arc<SkPixelRef>> {
    if rowBytes == 0 {
        rowBytes = info.minRowBytes();
    }
    if !is_valid(info) || !info.validRowBytes(rowBytes) {
        return None;
    }
    let size = info.computeByteSize(rowBytes);
    if SkImageInfo::ByteSizeOverflowed(size) {
        return None;
    }
    unsafe extern "C" {
        fn calloc(count: usize, size: usize) -> *mut c_void;
    }
    let addr = unsafe { calloc(1, size) };
    if addr.is_null() {
        return None;
    }
    // Existing native release-holder representation substitutes the final subclass destructor.
    unsafe extern "C" fn release(addr: *mut c_void, _ctx: *mut c_void) {
        unsafe extern "C" {
            fn free(ptr: *mut c_void);
        }
        free(addr);
    }
    Some(unsafe {
        SkMakePixelRefWithProc(
            info.width(),
            info.height(),
            rowBytes,
            addr,
            Some(release),
            std::ptr::null_mut(),
        )
    })
}
// Source: src/core/SkMallocPixelRef.cpp:59-65
// Source: src/core/SkMallocPixelRef.cpp:69-82
pub fn MakeWithData(
    info: &SkImageInfo,
    rowBytes: usize,
    data: Arc<SkData>,
) -> Option<Arc<SkPixelRef>> {
    // Arc is the native non-null owned data parameter; its pointer cannot be null.
    if !is_valid(info) {
        return None;
    }
    if rowBytes < info.minRowBytes() || data.size() < info.computeByteSize(rowBytes) {
        return None;
    }
    // The derived fData member must release before the base PixelRef listener destructor.
    unsafe extern "C" fn release(_addr: *mut c_void, ctx: *mut c_void) {
        drop(Arc::from_raw(ctx as *const SkData));
    }
    let pixels = data.data() as *mut c_void;
    let ctx = Arc::into_raw(data) as *mut c_void;
    let pr = unsafe {
        SkMakePixelRefWithProc(
            info.width(),
            info.height(),
            rowBytes,
            pixels,
            Some(release),
            ctx,
        )
    };
    pr.setImmutable();
    Some(pr)
}
