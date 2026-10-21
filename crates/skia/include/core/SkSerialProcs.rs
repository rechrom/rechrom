use crate::include::core::SkAlphaType::SkAlphaType;
use crate::src::core::{
    SkData::SkData,
    SkTypeface::{SkTypeface, SkTypefaceRef},
};
use crate::src::image::SkImage::SkImage;
use core::ffi::c_void;
use std::sync::Arc;
// Source: include/core/SkSerialProcs.h:29-29
pub type SkSerialReturnType = Option<Arc<SkData>>;
// Source: include/core/SkSerialProcs.h:39-39
pub type SkSerialPictureProc =
    unsafe fn(&dyn crate::src::core::SkPicture::SkPicture, *mut c_void) -> SkSerialReturnType;
// Source: include/core/SkSerialProcs.h:47-47
pub type SkDeserialPictureProc = unsafe fn(
    *const u8,
    usize,
    *mut c_void,
) -> Option<
    crate::include::core::SkRefCnt::sk_sp<dyn crate::src::core::SkPicture::SkPicture>,
>;
// Source: include/core/SkSerialProcs.h:40-41
pub type SkSerialImageProc = unsafe fn(&dyn SkImage, *mut c_void) -> SkSerialReturnType;
pub type SkSerialTypefaceProc = unsafe fn(&SkTypeface, *mut c_void) -> SkSerialReturnType;
// Source: include/core/SkSerialProcs.h:62-65
pub type SkDeserialImageProc =
    unsafe fn(*const u8, usize, Option<SkAlphaType>, *mut c_void) -> Option<Arc<dyn SkImage>>;
// Source: include/core/SkSerialProcs.h:67-69
pub type SkDeserialImageFromDataProc =
    unsafe fn(Arc<SkData>, Option<SkAlphaType>, *mut c_void) -> Option<Arc<dyn SkImage>>;
// Source: include/core/SkSerialProcs.h:76-76
pub type SkSlugProc = unsafe fn(
    &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    *mut c_void,
) -> Option<
    crate::include::core::SkRefCnt::sk_sp<dyn crate::include::private::chromium::Slug::Slug>,
>;
// Source: include/core/SkSerialProcs.h:83-83
// Safety: callback must not fork/duplicate the borrowed memory stream and retain
// it past return; its complete readable-byte lease ends with the native callback.
pub type SkDeserialTypefaceStreamProc = unsafe fn(
    core::pin::Pin<&mut dyn crate::src::core::SkStream::SkStream>,
    *mut c_void,
) -> Option<SkTypefaceRef>;
// Source: include/core/SkSerialProcs.h:84-84
pub type SkDeserialTypefaceProc = unsafe fn(*const u8, usize, *mut c_void) -> Option<SkTypefaceRef>;
// Source: include/core/SkSerialProcs.h:86-86
pub type SkDeserialAllowTagsProc = unsafe fn(u32, *mut c_void) -> bool;
// Source: include/core/SkSerialProcs.h:92-97
#[derive(Clone, Copy, Default)]
pub struct SkSerialProcs {
    // Source: include/core/SkSerialProcs.h:89-90
    pub fPictureProc: Option<SkSerialPictureProc>,
    pub fPictureCtx: *mut c_void,
    pub fImageProc: Option<SkSerialImageProc>,
    pub fImageCtx: *mut c_void,
    pub fTypefaceProc: Option<SkSerialTypefaceProc>,
    pub fTypefaceCtx: *mut c_void,
}
// Source: include/core/SkSerialProcs.h:103-105
// Source: include/core/SkSerialProcs.h:110-111
// Source: include/core/SkSerialProcs.h:116-120
#[derive(Clone, Copy)]
pub struct SkDeserialProcs {
    // Source: include/core/SkSerialProcs.h:100-101
    pub fPictureProc: Option<SkDeserialPictureProc>,
    pub fPictureCtx: *mut c_void,
    pub fImageProc: Option<SkDeserialImageProc>,
    pub fImageDataProc: Option<SkDeserialImageFromDataProc>,
    pub fImageCtx: *mut c_void,
    // Source: include/core/SkSerialProcs.h:107-108
    pub fSlugProc: Option<SkSlugProc>,
    pub fSlugCtx: *mut c_void,
    pub fTypefaceStreamProc: Option<SkDeserialTypefaceStreamProc>,
    pub fTypefaceCtx: *mut c_void,
    pub fAllowSkSL: bool,
    pub fAllowTagsProc: Option<SkDeserialAllowTagsProc>,
    pub fAllowTagsCtx: *mut c_void,
}
impl Default for SkDeserialProcs {
    fn default() -> Self {
        Self {
            fPictureProc: None,
            fPictureCtx: core::ptr::null_mut(),
            fImageProc: None,
            fImageDataProc: None,
            fImageCtx: core::ptr::null_mut(),
            fSlugProc: None,
            fSlugCtx: core::ptr::null_mut(),
            fTypefaceStreamProc: None,
            fTypefaceCtx: core::ptr::null_mut(),
            fAllowSkSL: true,
            fAllowTagsProc: None,
            fAllowTagsCtx: core::ptr::null_mut(),
        }
    }
}
