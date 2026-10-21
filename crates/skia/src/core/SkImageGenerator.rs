// Fixed upstream cacf77bdba7ba7df8ea7236d7e14b08c658ff368; unverified.
use super::{SkData::SkData, SkImageInfo::SkImageInfo, SkPixmap::SkPixmap};
use crate::include::core::{SkColorType::SkColorType, SkRecorder::SkRecorder};
use std::sync::Arc;
// Source: include/core/SkImageGenerator.h:129-132
pub struct SkImageGeneratorState {
    pub fInfo: SkImageInfo,
    pub fUniqueID: u32,
}
// Source: src/core/SkImageGenerator.cpp:14-17
impl SkImageGeneratorState {
    pub fn new(info: &SkImageInfo, uniqueID: u32) -> Self {
        Self {
            fInfo: info.clone(),
            fUniqueID: if uniqueID == 0 {
                super::SkNextID::SkNextID::ImageID()
            } else {
                uniqueID
            },
        }
    }
}
// Source: include/core/SkImageGenerator.h:118-119
pub const kNeedNewImageUniqueID: u32 = 0;
pub struct Options;
// The native unique generator stays the sole mutable decoder. A shared lazy
// image serializes all nonconst virtual calls using its actual generator mutex.
// Source: include/core/SkImageGenerator.h:23-23
// Native abstract interface adds no global thread-transfer marker. Callers
// crossing threads must retain their actual unique transfer/access contract.
// Source nonconst virtuals are represented by shared complete-object receivers:
// only each concrete owner's actual mutable pointee/cache fields use UnsafeCell.
// Base State remains outside that mutation domain, so its metadata reference
// can coexist with the original locked decoder operation without an &mut Self.
pub trait SkImageGenerator {
    fn state(&self) -> &SkImageGeneratorState;
    // Source: include/core/SkImageGenerator.h:31-31
    fn uniqueID(&self) -> u32 {
        self.state().fUniqueID
    }
    // Source: include/core/SkImageGenerator.h:40-40
    /// Safety: hold the original unique-generator or SharedGenerator mutex lease
    /// for all mutable payload/cache operations, excluding reentrant operations.
    unsafe fn refEncodedData(&self) -> Option<Arc<SkData>> {
        unsafe { self.onRefEncodedData() }
    }
    // Source: include/core/SkImageGenerator.h:45-45
    fn getInfo(&self) -> &SkImageInfo {
        &self.state().fInfo
    }
    // Source: include/core/SkImageGenerator.h:51-51
    fn isValid(&self, recorder: Option<&dyn SkRecorder>) -> bool {
        self.onIsValid(recorder)
    }
    // Source: include/core/SkImageGenerator.h:56-58
    fn isProtected(&self) -> bool {
        self.onIsProtected()
    }
    // Source: src/core/SkImageGenerator.cpp:19-32
    // Caller additionally holds the original exclusive mutable-payload operation
    // lease. No mutable reference to the base metadata or complete generator forms.
    unsafe fn getPixels(&self, info: &SkImageInfo, pixels: *mut u8, rowBytes: usize) -> bool {
        if info.colorType() == SkColorType::kUnknown_SkColorType {
            return false;
        }
        if pixels.is_null() {
            return false;
        }
        if rowBytes < info.minRowBytes() {
            return false;
        }
        unsafe { self.onGetPixels(info, pixels, rowBytes, &Options) }
    }
    // Source: include/core/SkImageGenerator.h:84-86
    unsafe fn getPixels_pixmap(&self, pm: &SkPixmap) -> bool {
        unsafe { self.getPixels(pm.info(), pm.writable_addr(), pm.rowBytes()) }
    }
    // Source: include/core/SkImageGenerator.h:99-100
    // Source: src/core/SkImageGenerator.cpp:34-40
    // Native nullable pointer remains nullable for the virtual callback. If a
    // callback claims success, its output must be nonnull as in native code.
    fn queryYUVAInfo(
        &self,
        supported: &super::SkYUVAPixmaps::SupportedDataTypes,
        mut info: Option<&mut super::SkYUVAPixmaps::SkYUVAPixmapInfo>,
    ) -> bool {
        crate::SkASSERT!(@source b"src/core/SkImageGenerator.cpp\0".as_ptr(),36; @condition b"yuvaPixmapInfo\0".as_ptr(); info.is_some());
        self.onQueryYUVAInfo(supported, info.as_deref_mut())
            && info
                .as_deref()
                .expect("native successful YUVA query requires nonnull output")
                .isSupported(supported)
    }
    // Source: include/core/SkImageGenerator.h:110-110
    // Source: src/core/SkImageGenerator.cpp:42-44
    /// Safety: caller provides the actual preallocated writable YUVA planes under
    /// the original decoder's exclusive mutation/initialization lease.
    unsafe fn getYUVAPlanes(&self, pixmaps: &super::SkYUVAPixmaps::SkYUVAPixmaps) -> bool {
        unsafe { self.onGetYUVAPlanes(pixmaps) }
    }
    // Source: include/core/SkImageGenerator.h:112-112
    fn isTextureGenerator(&self) -> bool {
        false
    }
    // Source: include/core/SkImageGenerator.h:120-126
    // Same source mutable-payload lease as refEncodedData; native default is null.
    unsafe fn onRefEncodedData(&self) -> Option<Arc<SkData>> {
        None
    }
    unsafe fn onGetPixels(
        &self,
        _info: &SkImageInfo,
        _pixels: *mut u8,
        _rowBytes: usize,
        _options: &Options,
    ) -> bool {
        false
    }
    fn onIsValid(&self, _recorder: Option<&dyn SkRecorder>) -> bool {
        true
    }
    fn onIsProtected(&self) -> bool {
        false
    }
    // Source: include/core/SkImageGenerator.h:125-126
    fn onQueryYUVAInfo(
        &self,
        _supported: &super::SkYUVAPixmaps::SupportedDataTypes,
        _info: Option<&mut super::SkYUVAPixmaps::SkYUVAPixmapInfo>,
    ) -> bool {
        false
    }
    // Source: include/core/SkImageGenerator.h:127-127
    /// Safety: same actual writable-plane/decoder lease as getYUVAPlanes.
    unsafe fn onGetYUVAPlanes(&self, _pixmaps: &super::SkYUVAPixmaps::SkYUVAPixmaps) -> bool {
        false
    }
}
// Native default YUVA virtuals return false exactly as upstream; actual decoder
// subclasses supply query/decode behavior, never a fabricated planar producer.
