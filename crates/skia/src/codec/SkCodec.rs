// Copyright 2015 Google Inc. BSD-style license. Native codec core candidate.
use crate::include::{
    codec::{
        SkCodecAnimation::{Blend, DisposalMethod},
        SkEncodedOrigin::{SkEncodedOrigin, SkEncodedOriginSwapsWidthHeight},
    },
    core::{SkAlphaType::SkAlphaType, SkColorType::SkColorType, SkSize::SkISize},
};
use crate::modules::skcms::skcms::{
    skcms_ApproximatelyEqualProfiles, skcms_Transform, skcms_sRGB_profile,
};
use crate::modules::skcms::src::skcms_public::{
    skcms_AlphaFormat, skcms_ICCProfile, skcms_PixelFormat,
};
use crate::src::codec::{
    SkEncodedInfo::{Alpha, Color, SkEncodedInfo},
    SkFrameHolder::{SkFrameHolderVirtual, SkFrameVirtual},
    SkSampler::{SkSampler, SkSamplerVirtual},
};
use crate::src::core::{
    SkData::SkData,
    SkImageInfo::SkImageInfo,
    SkPixmap::SkPixmap,
    SkRect::{SkIRect, SkRect},
    SkStream::{SkMemoryStream, SkStream},
    SkYUVAPixmaps::{SkYUVAPixmapInfo, SkYUVAPixmaps, SupportedDataTypes},
};
use core::{
    ffi::{c_char, c_void},
    mem::{ManuallyDrop, MaybeUninit},
    pin::Pin,
    ptr::{self, NonNull},
};
use std::sync::Arc;
// Source: include/codec/SkCodec.h:34-41
pub use crate::include::codec::SkEncodedImageFormat::SkEncodedImageFormat;
pub use crate::include::codec::SkPngChunkReader::SkPngChunkReader;
pub use crate::include::private::SkGainmapInfo::SkGainmapInfo;
// Source: include/codec/SkCodec.h:72-72
pub const fn MinBufferedBytesNeeded() -> usize {
    32
}
// Source: include/codec/SkCodec.h:77-129
// Open native enums preserve invalid integer values reaching real default arms.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Result(pub i32);
impl Result {
    pub const kSuccess: Self = Self(0);
    pub const kIncompleteInput: Self = Self(1);
    pub const kErrorInInput: Self = Self(2);
    pub const kInvalidConversion: Self = Self(3);
    pub const kInvalidScale: Self = Self(4);
    pub const kInvalidParameters: Self = Self(5);
    pub const kInvalidInput: Self = Self(6);
    pub const kCouldNotRewind: Self = Self(7);
    pub const kInternalError: Self = Self(8);
    pub const kUnimplemented: Self = Self(9);
    pub const kOutOfMemory: Self = Self(10);
}
// Source: include/codec/SkCodec.h:141-154
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SelectionPolicy(pub i32);
impl SelectionPolicy {
    pub const kPreferStillImage: Self = Self(0);
    pub const kPreferAnimation: Self = Self(1);
}
// Source: include/codec/SkCodec.h:318-331
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ZeroInitialized(pub i32);
impl ZeroInitialized {
    pub const kYes_ZeroInitialized: Self = Self(0);
    pub const kNo_ZeroInitialized: Self = Self(1);
}
// Source: include/codec/SkCodec.h:336-390
#[derive(Clone, Copy)]
pub struct Options {
    pub fZeroInitialized: ZeroInitialized,
    pub fSubset: *const SkIRect,
    pub fFrameIndex: i32,
    pub fPriorFrame: i32,
    pub fMaxDecodeMemory: usize,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            fZeroInitialized: ZeroInitialized::kNo_ZeroInitialized,
            fSubset: ptr::null(),
            fFrameIndex: 0,
            fPriorFrame: kNoFrame,
            fMaxDecodeMemory: 0,
        }
    }
}
// Source: include/codec/SkCodec.h:598-628
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkScanlineOrder(pub i32);
impl SkScanlineOrder {
    pub const kTopDown_SkScanlineOrder: Self = Self(0);
    pub const kBottomUp_SkScanlineOrder: Self = Self(1);
}
// Source: include/codec/SkCodec.h:679-679
pub const kNoFrame: i32 = -1;
// Source: include/codec/SkCodec.h:684-742
pub struct FrameInfo {
    pub fRequiredFrame: i32,
    pub fDuration: i32,
    pub fFullyReceived: bool,
    pub fAlphaType: SkAlphaType,
    pub fHasAlphaWithinBounds: bool,
    pub fDisposalMethod: DisposalMethod,
    pub fBlend: Blend,
    pub fFrameRect: SkIRect,
}
// Native vector<FrameInfo>(n) value-initializes every scalar/enum/rectangle.
impl Default for FrameInfo {
    fn default() -> Self {
        Self {
            fRequiredFrame: 0,
            fDuration: 0,
            fFullyReceived: false,
            fAlphaType: SkAlphaType::kUnknown_SkAlphaType,
            fHasAlphaWithinBounds: false,
            fDisposalMethod: DisposalMethod(0),
            fBlend: Blend(0),
            fFrameRect: SkIRect::MakeEmpty(),
        }
    }
}
// Source: include/codec/SkCodec.h:783-783
pub const kRepetitionCountInfinite: i32 = -1;
// Source: include/codec/SkCodec.h:830-834
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct IsAnimated(pub i32);
impl IsAnimated {
    pub const kYes: Self = Self(0);
    pub const kNo: Self = Self(1);
    pub const kUnknown: Self = Self(2);
}
// Source: include/codec/SkCodec.h:840-840
pub type XformFormat = skcms_PixelFormat;
// Source: include/codec/SkCodec.h:996-1000
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum XformTime {
    kNo_XformTime,
    kPalette_XformTime,
    kDecodeRow_XformTime,
}
// Source: include/codec/SkCodec.h:987-1021
// Native base state stays contiguous, including the actual pointer to its own
// ICC storage. initialize_at must run at the final complete pinned allocation.
// Uninitialized native conversion fields stay MaybeUninit until their producer.
#[repr(C)]
pub struct SkCodecState {
    fEncodedInfo: ManuallyDrop<SkEncodedInfo>,
    fSrcXformFormat: skcms_PixelFormat,
    fStream: ManuallyDrop<Option<Pin<Box<dyn SkStream>>>>,
    fNeedsRewind: bool,
    fOrigin: SkEncodedOrigin,
    fDstInfo: ManuallyDrop<SkImageInfo>,
    fOptions: Options,
    fXformTime: MaybeUninit<XformTime>,
    fDstXformFormat: MaybeUninit<skcms_PixelFormat>,
    fDstProfileStorage: MaybeUninit<skcms_ICCProfile>,
    fDstProfile: *const skcms_ICCProfile,
    fDstXformAlphaFormat: MaybeUninit<skcms_AlphaFormat>,
    fCurrScanline: i32,
    fDecodeBudget: usize,
    fStartedIncrementalDecode: bool,
    fUsingCallbackForHandleFrameIndex: bool,
}
impl SkCodecState {
    // Source: src/codec/SkCodec.cpp:264-273
    /// Safety: dst is the actual uninitialized base subobject in a final complete
    /// allocation; the complete codec must remain pinned through destruction.
    pub unsafe fn initialize_at(
        dst: *mut Self,
        info: SkEncodedInfo,
        srcFormat: skcms_PixelFormat,
        stream: Option<Pin<Box<dyn SkStream>>>,
        origin: SkEncodedOrigin,
    ) {
        unsafe {
            dst.write(Self {
                fEncodedInfo: ManuallyDrop::new(info),
                fSrcXformFormat: srcFormat,
                fStream: ManuallyDrop::new(stream),
                fNeedsRewind: false,
                fOrigin: origin,
                fDstInfo: ManuallyDrop::new(SkImageInfo::default()),
                fOptions: Options::default(),
                fXformTime: MaybeUninit::uninit(),
                fDstXformFormat: MaybeUninit::uninit(),
                fDstProfileStorage: MaybeUninit::uninit(),
                fDstProfile: ptr::null(),
                fDstXformAlphaFormat: MaybeUninit::uninit(),
                fCurrScanline: -1,
                fDecodeBudget: 0,
                fStartedIncrementalDecode: false,
                fUsingCallbackForHandleFrameIndex: false,
            });
            (*dst).fDstProfile = ptr::addr_of!((*dst).fDstProfileStorage).cast();
        }
    }
}
// Source: src/codec/SkCodec.cpp:275-275
// Empty native destructor body, then actual members reverse and encoded info last.
impl Drop for SkCodecState {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.fDstInfo);
            ManuallyDrop::drop(&mut self.fStream);
            ManuallyDrop::drop(&mut self.fEncodedInfo);
        }
    }
}
// Source: include/codec/SkCodec.h:59-59,225-225
// Only real virtuals are override slots. Two projections expose the actual base
// subobject; they introduce no provider/backend, ownership, or separate state.
// Pin preserves the native complete-object ICC self pointer and original stream.
pub unsafe trait SkCodec {
    fn codec_state(&self) -> &SkCodecState;
    fn codec_state_mut(self: Pin<&mut Self>) -> Pin<&mut SkCodecState>;
    // Source: include/codec/SkCodec.h:849-854
    unsafe fn onGetGainmapCodec(
        self: Pin<&mut Self>,
        _: *mut SkGainmapInfo,
        _: *mut Option<Pin<Box<dyn SkCodec>>>,
    ) -> bool {
        false
    }
    unsafe fn onGetGainmapInfo(self: Pin<&mut Self>, _: *mut SkGainmapInfo) -> bool {
        false
    }
    unsafe fn onGetGainmapInfo_stream(
        self: Pin<&mut Self>,
        _: *mut SkGainmapInfo,
        _: *mut Option<Pin<Box<dyn SkStream>>>,
    ) -> bool {
        false
    }
    // Source: include/codec/SkCodec.h:856-870
    fn onGetScaledDimensions(&self, _: f32) -> SkISize {
        self.dimensions()
    }
    fn onDimensionsSupported(self: Pin<&mut Self>, _: &SkISize) -> bool {
        false
    }
    fn onGetEncodedFormat(&self) -> SkEncodedImageFormat;
    // Source: include/codec/SkCodec.h:876-888
    unsafe fn onGetPixels(
        self: Pin<&mut Self>,
        info: &SkImageInfo,
        pixels: *mut u8,
        rowBytes: usize,
        options: &Options,
        rowsDecoded: *mut i32,
    ) -> Result;
    fn onQueryYUVAInfo(&self, _: &SupportedDataTypes, _: Option<&mut SkYUVAPixmapInfo>) -> bool {
        false
    }
    unsafe fn onGetYUVAPlanes(self: Pin<&mut Self>, _: &SkYUVAPixmaps) -> Result {
        Result::kUnimplemented
    }
    fn onGetValidSubset(&self, _: Option<&mut SkIRect>) -> bool {
        false
    }
    // Source: include/codec/SkCodec.h:913-918
    fn onRewind(self: Pin<&mut Self>) -> bool {
        self.rewindStream()
    }
    // Source: include/codec/SkCodec.h:934-981
    fn onGetScanlineOrder(&self) -> SkScanlineOrder {
        SkScanlineOrder::kTopDown_SkScanlineOrder
    }
    fn onOutputScanline(&self, input: i32) -> i32 {
        on_output_scanline(self, input)
    }
    fn conversionSupported(
        self: Pin<&mut Self>,
        dst: &SkImageInfo,
        srcIsOpaque: bool,
        needsColorXform: bool,
    ) -> bool {
        conversion_supported(self, dst, srcIsOpaque, needsColorXform)
    }
    fn usesColorXform(&self) -> bool {
        true
    }
    fn onGetFrameCount(self: Pin<&mut Self>) -> i32 {
        1
    }
    fn onGetFrameInfo(&self, _: i32, _: Option<&mut FrameInfo>) -> bool {
        false
    }
    fn onGetRepetitionCount(self: Pin<&mut Self>) -> i32 {
        0
    }
    fn onIsAnimated(self: Pin<&mut Self>) -> IsAnimated {
        IsAnimated::kNo
    }
    // Source: include/codec/SkCodec.h:1046-1048
    fn getFrameHolder(&self) -> Option<&dyn SkFrameHolderVirtual> {
        None
    }
    // Source: include/codec/SkCodec.h:1069-1092
    fn onStartScanlineDecode(self: Pin<&mut Self>, _: &SkImageInfo, _: &Options) -> Result {
        Result::kUnimplemented
    }
    fn onSupportsIncrementalDecode(self: Pin<&mut Self>, _: &SkImageInfo) -> bool {
        false
    }
    unsafe fn onStartIncrementalDecode(
        self: Pin<&mut Self>,
        _: &SkImageInfo,
        _: *mut u8,
        _: usize,
        _: &Options,
    ) -> Result {
        Result::kUnimplemented
    }
    unsafe fn onIncrementalDecode(self: Pin<&mut Self>, _: *mut i32) -> Result {
        Result::kUnimplemented
    }
    fn onSkipScanlines(self: Pin<&mut Self>, _: i32) -> bool {
        false
    }
    unsafe fn onGetScanlines(self: Pin<&mut Self>, _: *mut u8, _: i32, _: usize) -> i32 {
        0
    }
    // Source: include/codec/SkCodec.h:1120-1126
    fn getSampler(self: Pin<&mut Self>, _: bool) -> Option<NonNull<dyn SkSamplerVirtual>> {
        None
    }
    fn getEncodedData(&self) -> Option<Arc<SkData>> {
        get_encoded_data(self)
    }
}
// The blanket extension is native nonvirtual dispatch: implementations cannot
// substitute wrapper behavior. It carries no owner/vtable state of its own.
pub trait SkCodecExt: SkCodec {
    // Source: include/codec/SkCodec.h:233-267
    fn getInfo(&self) -> SkImageInfo {
        self.codec_state().fEncodedInfo.makeImageInfo()
    }
    fn dimensions(&self) -> SkISize {
        let s = self.codec_state();
        SkISize::Make(s.fEncodedInfo.width(), s.fEncodedInfo.height())
    }
    fn bounds(&self) -> SkIRect {
        let s = self.codec_state();
        SkIRect::MakeWH(s.fEncodedInfo.width(), s.fEncodedInfo.height())
    }
    fn getICCProfile(&self) -> *const skcms_ICCProfile {
        self.codec_state().fEncodedInfo.profile()
    }
    fn getHdrMetadata(&self) -> &crate::include::private::SkHdrMetadata::Metadata {
        self.codec_state().fEncodedInfo.getHdrMetadata()
    }
    fn hasHighBitDepthEncodedData(&self) -> bool {
        self.codec_state().fEncodedInfo.bitsPerComponent() >= 16
    }
    fn getOrigin(&self) -> SkEncodedOrigin {
        self.codec_state().fOrigin
    }
    // Source: include/codec/SkCodec.h:276-289
    fn getScaledDimensions(&self, scale: f32) -> SkISize {
        crate::SkASSERT!(@source b"include/codec/SkCodec.h\0".as_ptr(),278; @condition b"desiredScale > 0.0f\0".as_ptr(); scale>0.0);
        if scale <= 0.0 {
            return SkISize::Make(0, 0);
        }
        if scale >= 1.0 {
            return self.dimensions();
        }
        self.onGetScaledDimensions(scale)
    }
    // Source: include/codec/SkCodec.h:306-313
    fn getValidSubset(&self, subset: Option<&mut SkIRect>) -> bool {
        self.onGetValidSubset(subset)
    }
    fn getEncodedFormat(&self) -> SkEncodedImageFormat {
        self.onGetEncodedFormat()
    }
    // Source: include/codec/SkCodec.h:433-442
    unsafe fn getPixels(
        self: Pin<&mut Self>,
        info: &SkImageInfo,
        pixels: *mut u8,
        rowBytes: usize,
        options: Option<&Options>,
    ) -> Result {
        unsafe { get_pixels(self, info, pixels, rowBytes, options) }
    }
    unsafe fn getPixels_default(
        self: Pin<&mut Self>,
        info: &SkImageInfo,
        pixels: *mut u8,
        rowBytes: usize,
    ) -> Result {
        unsafe { self.getPixels(info, pixels, rowBytes, None) }
    }
    unsafe fn getPixels_pixmap(
        self: Pin<&mut Self>,
        pm: &SkPixmap,
        options: Option<&Options>,
    ) -> Result {
        unsafe { self.getPixels(pm.info(), pm.writable_addr(), pm.rowBytes(), options) }
    }
    // Source: include/codec/SkCodec.h:449-451
    unsafe fn getImage(
        self: Pin<&mut Self>,
        info: &SkImageInfo,
        options: Option<&Options>,
    ) -> core::result::Result<
        (Option<Arc<dyn crate::src::image::SkImage::SkImage>>, Result),
        crate::RasterError,
    > {
        unsafe { get_image(self, info, options) }
    }
    unsafe fn getImage_default(
        self: Pin<&mut Self>,
    ) -> core::result::Result<
        (Option<Arc<dyn crate::src::image::SkImage::SkImage>>, Result),
        crate::RasterError,
    > {
        let mut info = self.getInfo();
        if SkEncodedOriginSwapsWidthHeight(self.getOrigin()) {
            info = crate::src::codec::SkPixmapUtils::SwapWidthHeight(&info);
        }
        unsafe { self.getImage(&info, None) }
    }
    // Source: src/codec/SkCodec.cpp:277-284
    fn queryYUVAInfo(
        &self,
        supported: &SupportedDataTypes,
        mut info: Option<&mut SkYUVAPixmapInfo>,
    ) -> bool {
        if info.is_none() {
            return false;
        }
        self.onQueryYUVAInfo(supported, info.as_deref_mut())
            && info.as_deref().unwrap().isSupported(supported)
    }
    // Source: src/codec/SkCodec.cpp:286-294
    unsafe fn getYUVAPlanes(mut self: Pin<&mut Self>, pixmaps: &SkYUVAPixmaps) -> Result {
        if !pixmaps.isValid() {
            return Result::kInvalidInput;
        }
        if !self.as_mut().rewindIfNeeded() {
            return Result::kCouldNotRewind;
        }
        unsafe { self.onGetYUVAPlanes(pixmaps) }
    }
    // Source: include/codec/SkCodec.h:492-497
    unsafe fn startIncrementalDecode(
        self: Pin<&mut Self>,
        info: &SkImageInfo,
        pixels: *mut u8,
        rowBytes: usize,
        options: Option<&Options>,
    ) -> Result {
        unsafe { start_incremental_decode(self, info, pixels, rowBytes, options) }
    }
    // Source: include/codec/SkCodec.h:496-498
    unsafe fn startIncrementalDecode_default(
        self: Pin<&mut Self>,
        info: &SkImageInfo,
        pixels: *mut u8,
        rowBytes: usize,
    ) -> Result {
        unsafe { self.startIncrementalDecode(info, pixels, rowBytes, None) }
    }
    // Source: include/codec/SkCodec.h:522-527
    unsafe fn incrementalDecode(self: Pin<&mut Self>, rows: *mut i32) -> Result {
        if !self.codec_state().fStartedIncrementalDecode {
            return Result::kInvalidParameters;
        }
        unsafe { self.onIncrementalDecode(rows) }
    }
    // Source: include/codec/SkCodec.h:552-559
    unsafe fn startScanlineDecode(
        self: Pin<&mut Self>,
        info: &SkImageInfo,
        options: Option<&Options>,
    ) -> Result {
        unsafe { start_scanline_decode(self, info, options) }
    }
    // Source: include/codec/SkCodec.h:557-559
    unsafe fn startScanlineDecode_default(self: Pin<&mut Self>, info: &SkImageInfo) -> Result {
        unsafe { self.startScanlineDecode(info, None) }
    }
    // Source: src/codec/SkCodec.cpp:726-762
    unsafe fn getScanlines(self: Pin<&mut Self>, dst: *mut u8, count: i32, rowBytes: usize) -> i32 {
        unsafe { get_scanlines(self, dst, count, rowBytes) }
    }
    fn skipScanlines(mut self: Pin<&mut Self>, count: i32) -> bool {
        let current = self.codec_state().fCurrScanline;
        if current < 0 {
            return false;
        }
        crate::SkASSERT!(@source b"src/codec/SkCodec.cpp\0".as_ptr(),750; @condition b"!fDstInfo.isEmpty()\0".as_ptr(); !self.codec_state().fDstInfo.isEmpty());
        if count < 0 || current + count > self.codec_state().fDstInfo.height() {
            return false;
        }
        let result = self.as_mut().onSkipScanlines(count);
        unsafe { state_mut(self) }.fCurrScanline += count;
        result
    }
    // Source: include/codec/SkCodec.h:636-647
    fn getScanlineOrder(&self) -> SkScanlineOrder {
        self.onGetScanlineOrder()
    }
    fn nextScanline(&self) -> i32 {
        self.outputScanline(self.codec_state().fCurrScanline)
    }
    // Source: src/codec/SkCodec.cpp:764-767
    fn outputScanline(&self, input: i32) -> i32 {
        crate::SkASSERT!(@source b"src/codec/SkCodec.cpp\0".as_ptr(),764; @condition b"0 <= inputScanline && inputScanline < fEncodedInfo.height()\0".as_ptr(); 0<=input&&input<self.codec_state().fEncodedInfo.height());
        self.onOutputScanline(input)
    }
    // Source: include/codec/SkCodec.h:670-672,763-768,803-805,835
    fn getFrameCount(self: Pin<&mut Self>) -> i32 {
        self.onGetFrameCount()
    }
    fn getFrameInfo_index(&self, index: i32, info: Option<&mut FrameInfo>) -> bool {
        if index < 0 {
            return false;
        }
        self.onGetFrameInfo(index, info)
    }
    fn getFrameInfo(self: Pin<&mut Self>) -> Vec<FrameInfo> {
        get_frame_info(self)
    }
    fn getRepetitionCount(self: Pin<&mut Self>) -> i32 {
        self.onGetRepetitionCount()
    }
    fn isAnimated(self: Pin<&mut Self>) -> IsAnimated {
        self.onIsAnimated()
    }
    // Source: include/codec/SkCodec.h:838-840
    fn getEncodedInfo(&self) -> &SkEncodedInfo {
        &self.codec_state().fEncodedInfo
    }
    // Source: src/codec/SkCodec.cpp:321-345
    fn rewindStream(self: Pin<&mut Self>) -> bool {
        if let Some(stream) = unsafe { state_mut(self) }.fStream.as_mut() {
            if !stream.as_mut().rewind() {
                return false;
            }
        }
        true
    }
    #[must_use]
    fn rewindIfNeeded(mut self: Pin<&mut Self>) -> bool {
        let s = unsafe { state_mut(self.as_mut()) };
        let needs = s.fNeedsRewind;
        s.fNeedsRewind = true;
        if !needs {
            return true;
        }
        s.fCurrScanline = -1;
        s.fStartedIncrementalDecode = false;
        self.onRewind()
    }
    // Source: include/codec/SkCodec.h:923-946
    fn stream(self: Pin<&mut Self>) -> Option<Pin<&mut (dyn SkStream + 'static)>> {
        unsafe { state_mut(self) }
            .fStream
            .as_mut()
            .map(|s| s.as_mut())
    }
    fn dstInfo(&self) -> &SkImageInfo {
        &self.codec_state().fDstInfo
    }
    fn options(&self) -> &Options {
        &self.codec_state().fOptions
    }
    fn currScanline(&self) -> i32 {
        self.codec_state().fCurrScanline
    }
    // Source: include/codec/SkCodec.h:962-965
    unsafe fn applyColorXform(&self, dst: *mut u8, src: *const u8, count: i32) {
        unsafe { apply_color_xform(self, dst, src, count) }
    }
    unsafe fn colorXform(&self) -> bool {
        unsafe { self.codec_state().fXformTime.assume_init() != XformTime::kNo_XformTime }
    }
    unsafe fn xformOnDecode(&self) -> bool {
        unsafe { self.codec_state().fXformTime.assume_init() == XformTime::kDecodeRow_XformTime }
    }
    // Source: src/codec/SkCodec.cpp:483-489
    fn allocateFromBudget(self: Pin<&mut Self>, bytes: usize) -> bool {
        let s = unsafe { state_mut(self) };
        if bytes > s.fDecodeBudget {
            return false;
        }
        s.fDecodeBudget -= bytes;
        true
    }
    // Source: include/codec/SkCodec.h:1034-1036
    fn dimensionsSupported(mut self: Pin<&mut Self>, dim: &SkISize) -> bool {
        *dim == self.dimensions() || self.as_mut().onDimensionsSupported(dim)
    }
    // Source: include/codec/SkCodec.h:1023-1023
    unsafe fn initializeColorXform(
        self: Pin<&mut Self>,
        info: &SkImageInfo,
        alpha: Alpha,
        opaque: bool,
    ) -> bool {
        unsafe { initialize_color_xform(self, info, alpha, opaque) }
    }
    // Source: include/codec/SkCodec.h:1038-1041
    unsafe fn getPixelsBudgeted(
        self: Pin<&mut Self>,
        info: &SkImageInfo,
        pixels: *mut u8,
        rowBytes: usize,
        options: &Options,
    ) -> Result {
        unsafe { get_pixels_budgeted(self, info, pixels, rowBytes, options) }
    }
    // Source: include/codec/SkCodec.h:1108-1109
    unsafe fn fillIncompleteImage(
        self: Pin<&mut Self>,
        info: &SkImageInfo,
        dst: *mut u8,
        rowBytes: usize,
        zero: ZeroInitialized,
        requested: i32,
        decoded: i32,
    ) {
        unsafe { fill_incomplete_image(self, info, dst, rowBytes, zero, requested, decoded) }
    }
    // Source: src/codec/SkCodec.cpp:386-481
    unsafe fn handleFrameIndex(
        self: Pin<&mut Self>,
        info: &SkImageInfo,
        pixels: *mut u8,
        rowBytes: usize,
        options: &Options,
        callback: Option<&mut GetPixelsCallback<'_>>,
    ) -> Result {
        unsafe { handle_frame_index(self, info, pixels, rowBytes, options, callback) }
    }
}
impl<T: SkCodec + ?Sized> SkCodecExt for T {}
// Private nonmoving state projection used by the exact native mutation bodies.
unsafe fn state_mut<T: SkCodec + ?Sized>(codec: Pin<&mut T>) -> &mut SkCodecState {
    unsafe { codec.codec_state_mut().get_unchecked_mut() }
}
pub type GetPixelsCallback<'a> =
    dyn FnMut(&SkImageInfo, *mut u8, usize, &Options, i32) -> Result + 'a;
// Source: src/codec/SkCodec.cpp:296-319
fn conversion_supported<T: SkCodec + ?Sized>(
    this: Pin<&mut T>,
    dst: &SkImageInfo,
    opaque: bool,
    _needs: bool,
) -> bool {
    if !crate::src::codec::SkCodecPriv::SkCodecPriv::ValidAlpha(dst.alphaType(), opaque) {
        return false;
    }
    use SkColorType::*;
    match dst.colorType() {
        kRGBA_8888_SkColorType
        | kBGRA_8888_SkColorType
        | kRGBA_F16_SkColorType
        | kBGRA_10101010_XR_SkColorType => true,
        kBGR_101010x_XR_SkColorType | kRGB_565_SkColorType => opaque,
        kGray_8_SkColorType => {
            this.codec_state().fEncodedInfo.color() == Color::kGray_Color && opaque
        }
        kAlpha_8_SkColorType => this.codec_state().fEncodedInfo.color() == Color::kXAlpha_Color,
        _ => false,
    }
}
// Source: src/codec/SkCodec.cpp:347-354
fn frame_rect_on_screen(mut rect: SkIRect, screen: &SkIRect) -> SkIRect {
    if !rect.intersect(screen) {
        return SkIRect::make_empty();
    }
    rect
}

// Source: src/codec/SkCodec.cpp:356-384
pub unsafe fn zero_rect(
    info: &SkImageInfo,
    pixels: *mut u8,
    rowBytes: usize,
    srcDimensions: SkISize,
    mut prevRect: SkIRect,
) -> bool {
    unsafe {
        let dimensions = info.dimensions();
        if dimensions != srcDimensions {
            let src = SkRect::MakeISize(srcDimensions);
            let dst = SkRect::MakeISize(dimensions);
            let Some(map) = crate::src::core::SkMatrix::SkMatrix::Rect2Rect(
                &src,
                &dst,
                crate::src::core::SkMatrix::ScaleToFit::kFill_ScaleToFit,
            ) else {
                return false;
            };
            let mut rect = SkRect::Make(&prevRect);
            if !map.mapRect_in_place(&mut rect) {
                return false;
            }
            prevRect = rect.roundOut();
        }
        if !prevRect.intersect(&SkIRect::MakeSize(&dimensions)) {
            return true;
        }
        let fillInfo = info.makeDimensions(prevRect.size());
        let bpp = info.bytesPerPixel() as usize;
        let offset = prevRect.x() as usize * bpp + prevRect.y() as usize * rowBytes;
        SkSampler::Fill(
            &fillInfo,
            pixels.add(offset),
            rowBytes,
            ZeroInitialized::kNo_ZeroInitialized,
        );
        true
    }
}
// Source: src/codec/SkCodec.cpp:386-481
unsafe fn handle_frame_index<T: SkCodec + ?Sized>(
    mut this: Pin<&mut T>,
    info: &SkImageInfo,
    pixels: *mut u8,
    rowBytes: usize,
    options: &Options,
    mut callback: Option<&mut GetPixelsCallback<'_>>,
) -> Result {
    unsafe {
        if callback.is_some() {
            state_mut(this.as_mut()).fUsingCallbackForHandleFrameIndex = true;
        } else if this.codec_state().fUsingCallbackForHandleFrameIndex {
            return Result::kSuccess;
        }
        if !this.as_mut().rewindIfNeeded() {
            return Result::kCouldNotRewind;
        }
        let index = options.fFrameIndex;
        if index == 0 {
            let alpha = this.codec_state().fEncodedInfo.alpha();
            let opaque = this.codec_state().fEncodedInfo.opaque();
            return if initialize_color_xform(this, info, alpha, opaque) {
                Result::kSuccess
            } else {
                Result::kInvalidConversion
            };
        }
        if index < 0 {
            return Result::kInvalidParameters;
        }
        if !options.fSubset.is_null() {
            return Result::kInvalidParameters;
        }
        if index >= this.as_mut().onGetFrameCount() {
            return Result::kIncompleteInput;
        }
        let this_ptr = unsafe { this.as_mut().get_unchecked_mut() } as *mut T;
        let holder = unsafe { (&*this_ptr).getFrameHolder() };
        crate::SkASSERT!(@source b"src/codec/SkCodec.cpp\0".as_ptr(),421; @condition b"frameHolder\0".as_ptr(); holder.is_some());
        let holder = holder.expect("native animated codec has frame holder")
            as *const dyn SkFrameHolderVirtual;
        let frame = (*holder).getFrame(index);
        crate::SkASSERT!(@source b"src/codec/SkCodec.cpp\0".as_ptr(),424; @condition b"frame\0".as_ptr(); frame.is_some());
        let frame = frame.expect("native frame exists") as *const dyn SkFrameVirtual;
        let required = (*frame).frame().getRequiredFrame();
        if required != kNoFrame {
            let prepped: *const dyn SkFrameVirtual;
            if options.fPriorFrame == kNoFrame {
                let result = if let Some(cb) = callback.as_deref_mut() {
                    cb(info, pixels, rowBytes, options, required)
                } else {
                    let mut prev = *options;
                    prev.fFrameIndex = required;
                    unsafe { Pin::new_unchecked(&mut *this_ptr) }.getPixels(
                        info,
                        pixels,
                        rowBytes,
                        Some(&prev),
                    )
                };
                if result != Result::kSuccess {
                    return result;
                }
                prepped = (*holder)
                    .getFrame(required)
                    .expect("native required frame exists");
            } else {
                if options.fPriorFrame < required || options.fPriorFrame >= index {
                    return Result::kInvalidParameters;
                }
                prepped = (*holder)
                    .getFrame(options.fPriorFrame)
                    .expect("native prior frame exists");
            }
            crate::SkASSERT!(@source b"src/codec/SkCodec.cpp\0".as_ptr(),456; @condition b"preppedFrame\0".as_ptr(); !prepped.is_null());
            match (*prepped).frame().getDisposalMethod() {
                DisposalMethod::kRestorePrevious => {
                    crate::SkASSERT!(@source b"src/codec/SkCodec.cpp\0".as_ptr(),459; @condition b"options.fPriorFrame != kNoFrame\0".as_ptr(); options.fPriorFrame!=kNoFrame);
                    return Result::kInvalidParameters;
                }
                DisposalMethod::kRestoreBGColor => {
                    if (*prepped).frame().frameId() == required {
                        let rect = (*prepped).frame().frameRect();
                        if !zero_rect(info, pixels, rowBytes, this.dimensions(), rect) {
                            return Result::kInternalError;
                        }
                    }
                }
                _ => {}
            }
        }
        let alpha = (*frame).reportedAlpha();
        let opaque = !(*frame).frame().hasAlpha();
        if initialize_color_xform(this, info, alpha, opaque) {
            Result::kSuccess
        } else {
            Result::kInvalidConversion
        }
    }
}
// Source: src/codec/SkCodec.cpp:491-564
unsafe fn get_pixels<T: SkCodec + ?Sized>(
    mut this: Pin<&mut T>,
    info: &SkImageInfo,
    pixels: *mut u8,
    rowBytes: usize,
    options: Option<&Options>,
) -> Result {
    unsafe {
        let defaults = Options::default();
        let opts = options.unwrap_or(&defaults);
        state_mut(this.as_mut()).fDecodeBudget = if opts.fMaxDecodeMemory != 0 {
            opts.fMaxDecodeMemory
        } else {
            usize::MAX
        };
        get_pixels_budgeted(this, info, pixels, rowBytes, opts)
    }
}
unsafe fn get_pixels_budgeted<T: SkCodec + ?Sized>(
    mut this: Pin<&mut T>,
    info: &SkImageInfo,
    pixels: *mut u8,
    rowBytes: usize,
    options: &Options,
) -> Result {
    unsafe {
        if info.colorType() == SkColorType::kUnknown_SkColorType {
            return Result::kInvalidConversion;
        }
        if pixels.is_null() {
            return Result::kInvalidParameters;
        }
        if rowBytes < info.minRowBytes() {
            return Result::kInvalidParameters;
        }
        crate::SkASSERT!(@source b"src/codec/SkCodec.cpp\0".as_ptr(),515; @condition b"options\0".as_ptr(); !core::ptr::from_ref(options).is_null());
        crate::SkASSERT!(@source b"src/codec/SkCodec.cpp\0".as_ptr(),516; @condition b"fDecodeBudget > 0\0".as_ptr(); this.codec_state().fDecodeBudget>0);
        if !options.fSubset.is_null() {
            let mut subset = *options.fSubset;
            if !this.onGetValidSubset(Some(&mut subset)) || subset != *options.fSubset {
                return Result::kUnimplemented;
            }
        }
        let frameResult = this
            .as_mut()
            .handleFrameIndex(info, pixels, rowBytes, options, None);
        if frameResult != Result::kSuccess {
            return frameResult;
        }
        if !this.as_mut().dimensionsSupported(&info.dimensions()) {
            return Result::kInvalidScale;
        }
        *state_mut(this.as_mut()).fDstInfo = info.clone();
        state_mut(this.as_mut()).fOptions = *options;
        let mut rows = 0;
        let result = this
            .as_mut()
            .onGetPixels(info, pixels, rowBytes, options, &mut rows);
        if (result == Result::kIncompleteInput || result == Result::kErrorInInput)
            && rows != info.height()
        {
            state_mut(this.as_mut()).fOptions.fSubset = ptr::null();
            fill_incomplete_image(
                this,
                info,
                pixels,
                rowBytes,
                options.fZeroInitialized,
                info.height(),
                rows,
            );
        }
        result
    }
}
// Source: src/codec/SkCodec.cpp:566-616
unsafe fn get_image<T: SkCodec + ?Sized>(
    mut this: Pin<&mut T>,
    info: &SkImageInfo,
    options: Option<&Options>,
) -> core::result::Result<
    (Option<Arc<dyn crate::src::image::SkImage::SkImage>>, Result),
    crate::RasterError,
> {
    unsafe {
        let defaults = Options::default();
        let options = options.unwrap_or(&defaults);
        state_mut(this.as_mut()).fDecodeBudget = if options.fMaxDecodeMemory != 0 {
            options.fMaxDecodeMemory
        } else {
            usize::MAX
        };
        let size = info.computeByteSize(info.minRowBytes());
        if !this.as_mut().allocateFromBudget(size) {
            return Ok((None, Result::kOutOfMemory));
        }
        let mut bitmap = crate::src::core::SkBitmap::SkBitmap::default();
        if !bitmap.tryAllocPixels_info(info) {
            return Ok((None, Result::kInternalError));
        }
        // Checked initialization adapter: native reads result only after decode sets
        // it. If Orient fails before invoking decode, its uninitialized result is not
        // manufactured as a valid Rust enum or silently turned into success.
        let mut result = None;
        let origin = this.getOrigin();
        let oriented = crate::src::codec::SkPixmapUtilsPriv::OrientWithDecode(
            bitmap.pixmap(),
            origin,
            |pm| {
                let r = get_pixels_budgeted(
                    this.as_mut(),
                    pm.info(),
                    pm.writable_addr(),
                    pm.rowBytes(),
                    options,
                );
                result = Some(r);
                r == Result::kSuccess || r == Result::kIncompleteInput || r == Result::kErrorInInput
            },
        )?;
        if !oriented {
            return Ok((None,result.expect("native getImage requires decode to initialize result before unsuccessful Orient")));
        }
        bitmap.setImmutable();
        Ok((
            crate::src::image::SkImage_RasterFactories::RasterFromBitmap(&bitmap),
            Result::kSuccess,
        ))
    }
}
// Source: src/codec/SkCodec.cpp:618-670
unsafe fn start_incremental_decode<T: SkCodec + ?Sized>(
    mut this: Pin<&mut T>,
    info: &SkImageInfo,
    pixels: *mut u8,
    rowBytes: usize,
    options: Option<&Options>,
) -> Result {
    unsafe {
        state_mut(this.as_mut()).fStartedIncrementalDecode = false;
        if !this.as_mut().onSupportsIncrementalDecode(info) {
            return Result::kUnimplemented;
        }
        if info.colorType() == SkColorType::kUnknown_SkColorType {
            return Result::kInvalidConversion;
        }
        if pixels.is_null() {
            return Result::kInvalidParameters;
        }
        let defaults = Options::default();
        let opts = if let Some(opts) = options {
            if !opts.fSubset.is_null() {
                let subset = &*opts.fSubset;
                let size = SkIRect::MakeSize(&info.dimensions());
                if !size.contains_rect(subset) {
                    return Result::kInvalidParameters;
                }
                let top = subset.top();
                let bottom = subset.bottom();
                if top < 0 || top >= info.height() || top >= bottom || bottom > info.height() {
                    return Result::kInvalidParameters;
                }
            }
            opts
        } else {
            &defaults
        };
        let frameResult = this
            .as_mut()
            .handleFrameIndex(info, pixels, rowBytes, opts, None);
        if frameResult != Result::kSuccess {
            return frameResult;
        }
        if !this.as_mut().dimensionsSupported(&info.dimensions()) {
            return Result::kInvalidScale;
        }
        *state_mut(this.as_mut()).fDstInfo = info.clone();
        state_mut(this.as_mut()).fOptions = *opts;
        let copiedOptions = this.codec_state().fOptions;
        let result = this
            .as_mut()
            .onStartIncrementalDecode(info, pixels, rowBytes, &copiedOptions);
        if result == Result::kSuccess {
            state_mut(this).fStartedIncrementalDecode = true;
        }
        crate::SkASSERT!(@source b"src/codec/SkCodec.cpp\0".as_ptr(),668; @condition b"result != kUnimplemented\0".as_ptr(); result!=Result::kUnimplemented);
        result
    }
}
// Source: src/codec/SkCodec.cpp:673-724
unsafe fn start_scanline_decode<T: SkCodec + ?Sized>(
    mut this: Pin<&mut T>,
    info: &SkImageInfo,
    options: Option<&Options>,
) -> Result {
    unsafe {
        state_mut(this.as_mut()).fCurrScanline = -1;
        let defaults = Options::default();
        let options = options.unwrap_or(&defaults);
        state_mut(this.as_mut()).fDecodeBudget = if options.fMaxDecodeMemory != 0 {
            options.fMaxDecodeMemory
        } else {
            usize::MAX
        };
        if !options.fSubset.is_null() {
            let subset = &*options.fSubset;
            let size = SkIRect::MakeSize(&info.dimensions());
            if !size.contains_rect(subset) {
                return Result::kInvalidInput;
            }
            if subset.top() != 0 || subset.height() != info.height() {
                return Result::kInvalidInput;
            }
        }
        if options.fFrameIndex != 0 {
            return Result::kUnimplemented;
        }
        let frameResult = this
            .as_mut()
            .handleFrameIndex(info, ptr::null_mut(), 0, options, None);
        if frameResult != Result::kSuccess {
            return frameResult;
        }
        if !this.as_mut().dimensionsSupported(&info.dimensions()) {
            return Result::kInvalidScale;
        }
        let result = this.as_mut().onStartScanlineDecode(info, options);
        if result != Result::kSuccess {
            return result;
        }
        let state = state_mut(this);
        state.fCurrScanline = 0;
        *state.fDstInfo = info.clone();
        state.fOptions = *options;
        Result::kSuccess
    }
}
// Source: src/codec/SkCodec.cpp:726-743
unsafe fn get_scanlines<T: SkCodec + ?Sized>(
    mut this: Pin<&mut T>,
    dst: *mut u8,
    count: i32,
    rowBytes: usize,
) -> i32 {
    unsafe {
        let current = this.codec_state().fCurrScanline;
        if current < 0 {
            return 0;
        }
        crate::SkASSERT!(@source b"src/codec/SkCodec.cpp\0".as_ptr(),731; @condition b"!fDstInfo.isEmpty()\0".as_ptr(); !this.codec_state().fDstInfo.isEmpty());
        if count <= 0 || current + count > this.codec_state().fDstInfo.height() {
            return 0;
        }
        let decoded = this.as_mut().onGetScanlines(dst, count, rowBytes);
        if decoded < count {
            let info = (*this.codec_state().fDstInfo).clone();
            let zero = this.codec_state().fOptions.fZeroInitialized;
            fill_incomplete_image(this.as_mut(), &info, dst, rowBytes, zero, count, decoded);
        }
        state_mut(this).fCurrScanline += count;
        decoded
    }
}
// Source: src/codec/SkCodec.cpp:769-783
fn on_output_scanline<T: SkCodec + ?Sized>(this: &T, input: i32) -> i32 {
    match this.getScanlineOrder() {
        SkScanlineOrder::kTopDown_SkScanlineOrder => input,
        SkScanlineOrder::kBottomUp_SkScanlineOrder => {
            this.codec_state().fEncodedInfo.height() - input - 1
        }
        _ => {
            crate::SkASSERT!(@source b"src/codec/SkCodec.cpp\0".as_ptr(),776; @condition b"false\0".as_ptr(); false);
            0
        }
    }
}
// Source: src/codec/SkCodec.cpp:785-797
unsafe fn fill_incomplete_image<T: SkCodec + ?Sized>(
    mut this: Pin<&mut T>,
    info: &SkImageInfo,
    dst: *mut u8,
    rowBytes: usize,
    zero: ZeroInitialized,
    requested: i32,
    decoded: i32,
) {
    unsafe {
        if zero == ZeroInitialized::kYes_ZeroInitialized {
            return;
        }
        let remaining = requested - decoded;
        let sampler = this.as_mut().getSampler(false);
        let width = if let Some(s) = sampler {
            s.as_ref().fillWidth()
        } else if !this.codec_state().fOptions.fSubset.is_null() {
            (*this.codec_state().fOptions.fSubset).width()
        } else {
            info.width()
        };
        let fillDst = if this.getScanlineOrder() == SkScanlineOrder::kBottomUp_SkScanlineOrder {
            dst
        } else {
            dst.add(decoded as usize * rowBytes)
        };
        let fillInfo = info.makeWH(width, remaining);
        SkSampler::Fill(
            &fillInfo,
            fillDst,
            rowBytes,
            ZeroInitialized::kNo_ZeroInitialized,
        );
    }
}
// Source: src/codec/SkCodec.cpp:799-839
// Root merges this actual definition into canonical SkCodec.rs, not this helper header.
pub fn SkCodecPriv_SelectXformFormat(
    colorType: crate::include::core::SkColorType::SkColorType,
    forColorTable: bool,
    outFormat: &mut crate::modules::skcms::src::skcms_public::skcms_PixelFormat,
) -> bool {
    use crate::include::core::SkColorType::SkColorType;
    use crate::modules::skcms::src::skcms_public::skcms_PixelFormat::*;
    *outFormat = match colorType {
        SkColorType::kRGBA_8888_SkColorType => skcms_PixelFormat_RGBA_8888,
        SkColorType::kBGRA_8888_SkColorType => skcms_PixelFormat_BGRA_8888,
        SkColorType::kRGB_565_SkColorType => {
            if forColorTable {
                #[cfg(SK_PMCOLOR_IS_RGBA)]
                {
                    skcms_PixelFormat_RGBA_8888
                }
                #[cfg(not(SK_PMCOLOR_IS_RGBA))]
                {
                    skcms_PixelFormat_BGRA_8888
                }
            } else {
                skcms_PixelFormat_BGR_565
            }
        }
        SkColorType::kRGBA_F16_SkColorType => skcms_PixelFormat_RGBA_hhhh,
        SkColorType::kRGBA_1010102_SkColorType => skcms_PixelFormat_RGBA_1010102,
        SkColorType::kBGR_101010x_XR_SkColorType => skcms_PixelFormat_BGR_101010x_XR,
        SkColorType::kGray_8_SkColorType => skcms_PixelFormat_G_8,
        _ => return false,
    };
    true
}

// Source: src/codec/SkCodec.cpp:840-890
unsafe fn initialize_color_xform<T: SkCodec + ?Sized>(
    mut this: Pin<&mut T>,
    dst: &SkImageInfo,
    alpha: Alpha,
    opaque: bool,
) -> bool {
    unsafe {
        state_mut(this.as_mut())
            .fXformTime
            .write(XformTime::kNo_XformTime);
        let mut needs = false;
        if this.usesColorXform() {
            let high = matches!(
                dst.colorType(),
                SkColorType::kRGBA_F16_SkColorType
                    | SkColorType::kRGBA_1010102_SkColorType
                    | SkColorType::kBGR_101010x_XR_SkColorType
            );
            if high {
                needs = true;
                if let Some(cs) = dst.colorSpace() {
                    let state = state_mut(this.as_mut());
                    let profile = state.fDstProfileStorage.write(core::mem::zeroed());
                    cs.toProfile(profile);
                    state.fDstProfile = profile;
                } else {
                    let src = this.codec_state().fEncodedInfo.profile();
                    state_mut(this.as_mut()).fDstProfile = if src.is_null() {
                        skcms_sRGB_profile() as *const skcms_ICCProfile
                    } else {
                        src
                    };
                }
            } else if let Some(cs) = dst.colorSpace() {
                let state = state_mut(this.as_mut());
                let profile = state.fDstProfileStorage.write(core::mem::zeroed());
                cs.toProfile(profile);
                state.fDstProfile = profile;
                let src = this.codec_state().fEncodedInfo.profile();
                let src = if src.is_null() {
                    skcms_sRGB_profile() as *const skcms_ICCProfile
                } else {
                    src
                };
                if !skcms_ApproximatelyEqualProfiles(&*src, &*this.codec_state().fDstProfile) {
                    needs = true;
                }
            }
        }
        if !this.as_mut().conversionSupported(dst, opaque, needs) {
            return false;
        }
        if needs {
            let time = if this.codec_state().fEncodedInfo.color() != Color::kPalette_Color
                || dst.colorType() == SkColorType::kRGBA_F16_SkColorType
            {
                XformTime::kDecodeRow_XformTime
            } else {
                XformTime::kPalette_XformTime
            };
            state_mut(this.as_mut()).fXformTime.write(time);
            let mut format = skcms_PixelFormat::skcms_PixelFormat_A_8;
            if !crate::src::codec::SkCodecPriv::SkCodecPriv::SelectXformFormat(
                dst.colorType(),
                time == XformTime::kPalette_XformTime,
                &mut format,
            ) {
                return false;
            }
            state_mut(this.as_mut()).fDstXformFormat.write(format);
            let alpha = if alpha == Alpha::kUnpremul_Alpha
                && dst.alphaType() == SkAlphaType::kPremul_SkAlphaType
            {
                skcms_AlphaFormat::skcms_AlphaFormat_PremulAsEncoded
            } else {
                skcms_AlphaFormat::skcms_AlphaFormat_Unpremul
            };
            state_mut(this).fDstXformAlphaFormat.write(alpha);
        }
        true
    }
}
// Source: src/codec/SkCodec.cpp:892-898
unsafe fn apply_color_xform<T: SkCodec + ?Sized>(
    this: &T,
    dst: *mut u8,
    src: *const u8,
    count: i32,
) {
    unsafe {
        let s = this.codec_state();
        let srcProfile = s.fEncodedInfo.profile();
        crate::SkAssertResult!(@source b"src/codec/SkCodec.cpp\0".as_ptr(),895; @condition b"skcms_Transform(src, fSrcXformFormat, skcms_AlphaFormat_Unpremul, srcProfile, dst, fDstXformFormat, fDstXformAlphaFormat, fDstProfile, count)\0".as_ptr(); skcms_Transform(src.cast(),s.fSrcXformFormat,skcms_AlphaFormat::skcms_AlphaFormat_Unpremul,srcProfile.as_ref(),dst.cast(),s.fDstXformFormat.assume_init(),s.fDstXformAlphaFormat.assume_init(),s.fDstProfile.as_ref(),count as usize));
    }
}
// Source: src/codec/SkCodec.cpp:900-917
fn get_frame_info<T: SkCodec + ?Sized>(mut this: Pin<&mut T>) -> Vec<FrameInfo> {
    let count = this.as_mut().getFrameCount();
    crate::SkASSERT!(@source b"src/codec/SkCodec.cpp\0".as_ptr(),902; @condition b"frameCount >= 0\0".as_ptr(); count>=0);
    if count <= 0 {
        return Vec::new();
    }
    if count == 1 && !this.onGetFrameInfo(0, None) {
        return Vec::new();
    }
    let mut result: Vec<FrameInfo> = (0..count).map(|_| FrameInfo::default()).collect();
    for i in 0..count {
        crate::SkAssertResult!(@source b"src/codec/SkCodec.cpp\0".as_ptr(),914; @condition b"this->onGetFrameInfo(i, &result[i])\0".as_ptr(); this.onGetFrameInfo(i,Some(&mut result[i as usize])));
    }
    result
}
// Source: src/codec/SkCodec.cpp:919-947
pub fn ResultToString(result: Result) -> *const c_char {
    let bytes: &[u8] = match result {
        Result::kSuccess => b"success\0",
        Result::kIncompleteInput => b"incomplete input\0",
        Result::kErrorInInput => b"error in input\0",
        Result::kInvalidConversion => b"invalid conversion\0",
        Result::kInvalidScale => b"invalid scale\0",
        Result::kInvalidParameters => b"invalid parameters\0",
        Result::kInvalidInput => b"invalid input\0",
        Result::kCouldNotRewind => b"could not rewind\0",
        Result::kInternalError => b"internal error\0",
        Result::kUnimplemented => b"unimplemented\0",
        Result::kOutOfMemory => b"out of memory\0",
        _ => {
            crate::SkASSERT!(@source b"src/codec/SkCodec.cpp\0".as_ptr(),944; @condition b"false\0".as_ptr(); false);
            b"bogus result value\0"
        }
    };
    bytes.as_ptr().cast()
}
// Source: src/codec/SkCodec.cpp:949-961
pub fn SkFrame_fillIn<F: SkFrameVirtual + ?Sized>(
    this: &F,
    info: &mut FrameInfo,
    fullyReceived: bool,
) {
    crate::SkASSERT!(@source b"src/codec/SkCodec.cpp\0".as_ptr(),950; @condition b"frameInfo\0".as_ptr(); !core::ptr::from_mut(info).is_null());
    let f = this.frame();
    info.fRequiredFrame = f.fRequiredFrame;
    info.fDuration = f.fDuration;
    info.fFullyReceived = fullyReceived;
    info.fAlphaType = if f.fHasAlpha {
        SkAlphaType::kUnpremul_SkAlphaType
    } else {
        SkAlphaType::kOpaque_SkAlphaType
    };
    info.fHasAlphaWithinBounds = this.reportedAlpha() != Alpha::kOpaque_Alpha;
    info.fDisposalMethod = f.fDisposalMethod;
    info.fBlend = f.fBlend;
    info.fFrameRect = f.fRect;
}

// Source: src/codec/SkCodec.cpp:963-965
fn independent(frame: &dyn SkFrameVirtual) -> bool {
    frame.frame().getRequiredFrame() == kNoFrame
}
// Source: src/codec/SkCodec.cpp:967-969
fn restore_bg(frame: &dyn SkFrameVirtual) -> bool {
    frame.frame().getDisposalMethod() == DisposalMethod::kRestoreBGColor
}

// Source: src/codec/SkCodec.cpp:1013-1085
pub fn SkFrameHolder_setAlphaAndRequiredFrame<H: SkFrameHolderVirtual + ?Sized>(
    this: &H,
    frame: &mut dyn SkFrameVirtual,
) {
    let reportsAlpha = frame.reportedAlpha() != Alpha::kOpaque_Alpha;
    let h = this.frame_holder();
    let screenRect = SkIRect::make_wh(h.fScreenWidth, h.fScreenHeight);
    let frameRect = frame_rect_on_screen(frame.frame().frameRect(), &screenRect);
    let i = frame.frame().frameId();
    if i == 0 {
        frame
            .frame_mut()
            .setHasAlpha(reportsAlpha || frameRect != screenRect);
        frame.frame_mut().setRequiredFrame(kNoFrame);
        return;
    }
    let blendWithPrevFrame = frame.frame().getBlend() == Blend::kSrcOver;
    if (!reportsAlpha || !blendWithPrevFrame) && frameRect == screenRect {
        frame.frame_mut().setHasAlpha(reportsAlpha);
        frame.frame_mut().setRequiredFrame(kNoFrame);
        return;
    }
    let mut prev = this
        .getFrame(i - 1)
        .expect("native previous frame is nonnull");
    while prev.frame().getDisposalMethod() == DisposalMethod::kRestorePrevious {
        let prevId = prev.frame().frameId();
        if prevId == 0 {
            frame.frame_mut().setHasAlpha(true);
            frame.frame_mut().setRequiredFrame(kNoFrame);
            return;
        }
        prev = this
            .getFrame(prevId - 1)
            .expect("native previous frame is nonnull");
    }
    let clearPrevFrame = restore_bg(prev);
    let mut prevFrameRect = frame_rect_on_screen(prev.frame().frameRect(), &screenRect);
    if clearPrevFrame {
        if prevFrameRect == screenRect || independent(prev) {
            frame.frame_mut().setHasAlpha(true);
            frame.frame_mut().setRequiredFrame(kNoFrame);
            return;
        }
    }
    if reportsAlpha && blendWithPrevFrame {
        frame.frame_mut().setRequiredFrame(prev.frame().frameId());
        frame
            .frame_mut()
            .setHasAlpha(prev.frame().hasAlpha() || clearPrevFrame);
        return;
    }
    while frameRect.contains_rect(&prevFrameRect) {
        let prevRequiredFrame = prev.frame().getRequiredFrame();
        if prevRequiredFrame == kNoFrame {
            frame.frame_mut().setRequiredFrame(kNoFrame);
            frame.frame_mut().setHasAlpha(true);
            return;
        }
        prev = this
            .getFrame(prevRequiredFrame)
            .expect("native required frame is nonnull");
        prevFrameRect = frame_rect_on_screen(prev.frame().frameRect(), &screenRect);
    }
    frame.frame_mut().setRequiredFrame(prev.frame().frameId());
    if restore_bg(prev) {
        frame.frame_mut().setHasAlpha(true);
        return;
    }
    crate::SkASSERT!(@source b"src/codec/SkCodec.cpp\0".as_ptr(),1083; @condition b"prevFrame->getDisposalMethod() == SkCodecAnimation::DisposalMethod::kKeep\0".as_ptr(); prev.frame().getDisposalMethod()==DisposalMethod::kKeep);
    frame
        .frame_mut()
        .setHasAlpha(prev.frame().hasAlpha() || (reportsAlpha && !blendWithPrevFrame));
}

// Source: src/codec/SkCodec.cpp:1087-1101
fn get_encoded_data<T: SkCodec + ?Sized>(this: &T) -> Option<Arc<SkData>> {
    let stream = &this.codec_state().fStream;
    crate::SkASSERT!(@source b"src/codec/SkCodec.cpp\0".as_ptr(),1088; @condition b"fStream\0".as_ptr(); stream.is_some());
    let stream = stream.as_ref()?;
    if let Some(data) = stream.getData() {
        return Some(data);
    }
    let mut duplicate = stream
        .duplicate()
        .expect("native stream duplicate nonnull precondition");
    if !duplicate.hasLength() {
        return None;
    }
    let size = duplicate.getLength();
    SkData::MakeFromStream(duplicate.as_mut(), size)
}
// Source: include/codec/SkCodec.h:1138-1175
pub mod SkCodecs {
    use super::*;
    // Source: include/codec/SkCodec.h:1140-1152
    // Rust-only interface adaptation of native void*: keep the actual nonowning
    // PNG virtual receiver (both data and metadata), or the same local selection
    // policy borrow, without manufacturing a C++ object representation.
    // No allocation/retention is performed by the context itself.
    pub enum DecodeContext<'context> {
        PngChunkReader(Option<NonNull<dyn SkPngChunkReader + 'context>>),
        SelectionPolicy(&'context mut super::SelectionPolicy),
        Raw(*mut c_void),
    }
    pub type IsFormatCallback = unsafe fn(*const c_void, usize) -> bool;
    // The callback must consume the context synchronously. A returned codec may
    // retain a PNG reader only through its actual intrusive ref/unref owner protocol;
    // it cannot retain this borrowed selectionPolicy or assume a C++ vtable layout.
    pub type MakeFromStreamCallback = for<'context> unsafe fn(
        Option<Pin<Box<dyn SkStream>>>,
        *mut Result,
        DecodeContext<'context>,
    ) -> Option<Pin<Box<dyn SkCodec>>>;
    #[derive(Clone, Copy)]
    pub struct Decoder<'a> {
        pub id: &'a [u8],
        pub isFormat: Option<IsFormatCallback>,
        pub makeFromStream: Option<MakeFromStreamCallback>,
    }
    // Source: src/codec/SkCodec.cpp:91-145
    struct Registry(core::cell::UnsafeCell<Vec<Decoder<'static>>>);
    // Native edits are unsynchronized and must finish before concurrent decoding.
    // This only exposes that original protocol, not a new thread-safe registry.
    unsafe impl Sync for Registry {}
    static DECODERS: Registry = Registry(core::cell::UnsafeCell::new(Vec::new()));
    #[cfg(not(SK_DISABLE_LEGACY_INIT_DECODERS))]
    static ONCE: crate::include::private::SkOnce::SkOnce =
        crate::include::private::SkOnce::SkOnce::new();
    unsafe fn get_decoders_for_editing() -> *mut Vec<Decoder<'static>> {
        unsafe {
            #[cfg(not(SK_DISABLE_LEGACY_INIT_DECODERS))]
            crate::include::private::SkOnce::SkOnce::call_once(&ONCE, || {
                let decoders = &mut *DECODERS.0.get();
                if decoders.is_empty() {
                    #[cfg(SK_CODEC_DECODES_PNG_WITH_LIBPNG)]
                    decoders.push(crate::include::codec::SkPngDecoder::Decoder());
                    #[cfg(all(
                        not(SK_CODEC_DECODES_PNG_WITH_LIBPNG),
                        SK_CODEC_DECODES_PNG_WITH_RUST
                    ))]
                    decoders.push(crate::include::codec::SkPngRustDecoder::Decoder());
                    #[cfg(SK_CODEC_DECODES_JPEG_WITH_RUST)]
                    decoders.push(
                        crate::experimental::rust_jpeg::decoder::SkJpegRustDecoder::Decoder(),
                    );
                    #[cfg(all(not(SK_CODEC_DECODES_JPEG_WITH_RUST), SK_CODEC_DECODES_JPEG))]
                    decoders.push(crate::include::codec::SkJpegDecoder::Decoder());
                    #[cfg(SK_CODEC_DECODES_WEBP)]
                    decoders.push(crate::include::codec::SkWebpDecoder::Decoder());
                    #[cfg(any(SK_CODEC_DECODES_GIF, SK_HAS_WUFFS_LIBRARY))]
                    decoders.push(crate::include::codec::SkGifDecoder::Decoder());
                    #[cfg(SK_CODEC_DECODES_ICO_WITH_RUST)]
                    decoders
                        .push(crate::experimental::rust_ico::decoder::SkIcoRustDecoder::Decoder());
                    #[cfg(all(not(SK_CODEC_DECODES_ICO_WITH_RUST), SK_CODEC_DECODES_ICO))]
                    decoders.push(crate::include::codec::SkIcoDecoder::Decoder());
                    #[cfg(SK_CODEC_DECODES_BMP_WITH_RUST)]
                    decoders.push(crate::include::codec::SkBmpRustDecoder::Decoder());
                    #[cfg(all(not(SK_CODEC_DECODES_BMP_WITH_RUST), SK_CODEC_DECODES_BMP))]
                    decoders.push(crate::include::codec::SkBmpDecoder::Decoder());
                    #[cfg(SK_CODEC_DECODES_WBMP)]
                    decoders.push(crate::include::codec::SkWbmpDecoder::Decoder());
                    #[cfg(all(SK_CODEC_DECODES_AVIF, sk_build_for_android_framework))]
                    decoders.push(crate::include::codec::SkAvifDecoder::CrabbyAvif::Decoder());
                    #[cfg(all(SK_CODEC_DECODES_AVIF, not(sk_build_for_android_framework)))]
                    decoders.push(crate::include::codec::SkAvifDecoder::LibAvif::Decoder());
                    #[cfg(SK_CODEC_DECODES_JPEGXL)]
                    decoders.push(crate::include::codec::SkJpegxlDecoder::Decoder());
                    #[cfg(SK_CODEC_DECODES_RAW)]
                    decoders.push(crate::include::codec::SkRawDecoder::Decoder());
                }
            });
            DECODERS.0.get()
        }
    }
    // Source: src/codec/SkCodec.cpp:147-164
    /// Safety: initialization/registration is externally serialized and completed
    /// before any borrowed decoder slice is used, matching the native global API.
    pub unsafe fn get_decoders() -> &'static [Decoder<'static>] {
        unsafe { &*get_decoders_for_editing() }
    }
    pub unsafe fn Register(d: Decoder<'static>) {
        unsafe {
            let decoders = &mut *get_decoders_for_editing();
            for i in 0..decoders.len() {
                if decoders[i].id == d.id {
                    decoders[i] = d;
                    return;
                }
            }
            decoders.push(d);
        }
    }
    // Source: src/codec/SkCodec.cpp:166-173
    pub fn HasDecoder(id: &[u8]) -> bool {
        unsafe {
            for decoder in get_decoders() {
                if decoder.id == id {
                    return true;
                }
            }
            false
        }
    }
    // Source: include/codec/SkCodec.h:1174-1175
    // Actual body is in the factory worker's SkImageGenerator_FromEncoded.cpp.
    pub use crate::src::codec::SkImageGenerator_FromEncoded::{
        DeferredImage, DeferredImage_default,
    };
}
// Source: src/codec/SkCodec.cpp:177-261
/// Safety: chunkReader/context and each callback obey their original synchronous
/// native lifetime, stream mutation and complete pinned Codec ownership contracts.
pub unsafe fn MakeFromStream<'reader>(
    stream: Option<Pin<Box<dyn SkStream>>>,
    outResult: Option<&mut Result>,
    chunkReader: Option<NonNull<dyn SkPngChunkReader + 'reader>>,
    selectionPolicy: SelectionPolicy,
) -> Option<Pin<Box<dyn SkCodec>>> {
    unsafe {
        MakeFromStream_decoders(
            stream,
            SkCodecs::get_decoders(),
            outResult,
            chunkReader,
            selectionPolicy,
        )
    }
}
pub unsafe fn MakeFromStream_decoders<'reader>(
    stream: Option<Pin<Box<dyn SkStream>>>,
    decoders: &[SkCodecs::Decoder<'_>],
    outResult: Option<&mut Result>,
    chunkReader: Option<NonNull<dyn SkPngChunkReader + 'reader>>,
    mut selectionPolicy: SelectionPolicy,
) -> Option<Pin<Box<dyn SkCodec>>> {
    unsafe {
        let mut storage = MaybeUninit::<Result>::uninit();
        let result = outResult.map_or(storage.as_mut_ptr(), |r| r as *mut Result);
        let Some(mut stream) = stream else {
            result.write(Result::kInvalidInput);
            return None;
        };
        if selectionPolicy != SelectionPolicy::kPreferStillImage
            && selectionPolicy != SelectionPolicy::kPreferAnimation
        {
            result.write(Result::kInvalidParameters);
            return None;
        }
        let mut buffer = [MaybeUninit::<u8>::uninit(); MinBufferedBytesNeeded()];
        let mut bytes = stream
            .as_mut()
            .peek(buffer.as_mut_ptr().cast(), buffer.len());
        if bytes == 0 {
            bytes = stream
                .as_mut()
                .read(buffer.as_mut_ptr().cast(), buffer.len());
            if !stream.as_mut().rewind() {
                crate::SkCodecPrintf!(@source b"src/codec/SkCodec.cpp\0".as_ptr(),218; b"Encoded image data could not peek or rewind to determine format!\n\0".as_ptr());
                result.write(Result::kCouldNotRewind);
                return None;
            }
        }
        let mut rawFallback = None;
        for proc in decoders {
            if proc
                .isFormat
                .expect("native decoder format callback nonnull")(
                buffer.as_ptr().cast(), bytes
            ) {
                let make = proc
                    .makeFromStream
                    .expect("native decoder maker callback nonnull");
                if proc.id == b"png" {
                    return make(
                        Some(stream),
                        result,
                        SkCodecs::DecodeContext::PngChunkReader(chunkReader),
                    );
                } else if proc.id == b"heif" || proc.id == b"gif" {
                    return make(
                        Some(stream),
                        result,
                        SkCodecs::DecodeContext::SelectionPolicy(&mut selectionPolicy),
                    );
                } else if proc.id == b"raw" {
                    rawFallback = Some(make);
                    continue;
                }
                return make(
                    Some(stream),
                    result,
                    SkCodecs::DecodeContext::Raw(ptr::null_mut()),
                );
            }
        }
        if let Some(raw) = rawFallback {
            return raw(
                Some(stream),
                result,
                SkCodecs::DecodeContext::Raw(ptr::null_mut()),
            );
        }
        result.write(if bytes < MinBufferedBytesNeeded() {
            Result::kIncompleteInput
        } else {
            Result::kUnimplemented
        });
        None
    }
}
// Source: src/codec/SkCodec.cpp:252-262
pub unsafe fn MakeFromData<'reader>(
    data: Option<Arc<SkData>>,
    reader: Option<NonNull<dyn SkPngChunkReader + 'reader>>,
) -> Option<Pin<Box<dyn SkCodec>>> {
    unsafe { MakeFromData_decoders(data, SkCodecs::get_decoders(), reader) }
}
pub unsafe fn MakeFromData_decoders<'reader>(
    data: Option<Arc<SkData>>,
    decoders: &[SkCodecs::Decoder<'_>],
    reader: Option<NonNull<dyn SkPngChunkReader + 'reader>>,
) -> Option<Pin<Box<dyn SkCodec>>> {
    unsafe {
        let data = data?;
        let stream = SkMemoryStream::Make(Some(data));
        MakeFromStream_decoders(
            Some(stream),
            decoders,
            None,
            reader,
            SelectionPolicy::kPreferStillImage,
        )
    }
}
// Source: src/codec/SkCodec.cpp:34-85
#[cfg(not(sk_codec_native_inputs_configured))]
compile_error!("SkCodec requires complete explicit native decoder macro inputs; an unknown configured decoder list cannot be replaced with an empty registry");
