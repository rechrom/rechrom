use crate::src::core::SkImageInfo::SkImageInfo;
use crate::src::core::SkMatrix::SkMatrix;
use crate::src::core::SkRect::SkIRect;
use core::ffi::c_void;
use core::mem::MaybeUninit;

// Source: include/core/SkRasterHandleAllocator.h:37-52
// Rust virtual destruction uses the implementation's normal Drop. The descriptor
// itself does not own/drop pixels: installPixels receives its release obligation.
pub type Handle = *mut c_void;
#[repr(C)]
pub struct Rec {
    pub fReleaseProc: Option<unsafe extern "C" fn(*mut c_void, *mut c_void)>,
    pub fReleaseCtx: *mut c_void,
    pub fPixels: *mut c_void,
    pub fRowBytes: usize,
    pub fHandle: Handle,
}

// Source: include/core/SkRasterHandleAllocator.h:54-70
/// Native abstract raster allocation/handle contract; no platform allocator is supplied.
///
/// # Safety
/// Returning true from allocHandle must initialize every Rec field. Its backing
/// must cover the requested image/stride, satisfy native raster alignment and
/// initialized-read requirements, and remain live and exclusively writable by
/// the installing owner until its release callback runs. The release proc/context
/// must support exactly the ownership/thread behavior of Bitmap::installPixels.
/// updateHandle receives only a handle produced by this implementation whose
/// backing has not been released; it must not invalidate that pixel lease.
/// Returning false leaves the output unobserved and transfers no release obligation.
pub unsafe trait SkRasterHandleAllocator {
    fn allocHandle(&mut self, info: &SkImageInfo, rec: &mut MaybeUninit<Rec>) -> bool;
    /// # Safety
    /// handle must be a live handle returned by this allocator, with its pixel
    /// allocation still retained by the installing device.
    unsafe fn updateHandle(&mut self, handle: Handle, matrix: &SkMatrix, clip: &SkIRect);
}
