// Copyright 2015 Google Inc. BSD-style license.
use core::ffi::c_void;
use crate::include::core::{SkAlphaType::SkAlphaType, SkSize::SkISize};
use crate::src::core::{SkImageInfo::SkImageInfo, SkRect::SkIRect,
    SkYUVAPixmaps::{SkYUVAPixmapInfo, SkYUVAPixmaps, SupportedDataTypes},
    SkStream::SkStream, SkData::SkData};
use std::sync::Arc;

// Source: include/codec/SkCodec.h:34-46
// Actual forward declarations: no object layout or decoder implementation is
// inferred. Only pointers to these incomplete classes occur in this interface.
unsafe extern "C" {
    pub type SkFrameHolder;
    pub type SkSampler;
    pub type SkGainmapInfo;
}
// Native opaque enum declarations fix the underlying type as int. These carry
// no invented enumerators; the defining headers remain separate dependencies.
#[repr(transparent)] #[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkEncodedImageFormat(pub i32);
pub mod SkCodecAnimation {
    #[repr(transparent)] #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct Blend(pub i32);
    #[repr(transparent)] #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct DisposalMethod(pub i32);
}

// Source: include/codec/SkCodec.h:72-72
pub const fn MinBufferedBytesNeeded() -> usize { 32 }
// Source: include/codec/SkCodec.h:77-129
#[repr(i32)] #[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Result {
    kSuccess, kIncompleteInput, kErrorInInput, kInvalidConversion, kInvalidScale,
    kInvalidParameters, kInvalidInput, kCouldNotRewind, kInternalError,
    kUnimplemented, kOutOfMemory,
}
// Source: include/codec/SkCodec.h:141-154
#[repr(i32)] #[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionPolicy { kPreferStillImage, kPreferAnimation }
// Source: include/codec/SkCodec.h:318-331
#[repr(i32)] #[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZeroInitialized { kYes_ZeroInitialized, kNo_ZeroInitialized }
// Source: include/codec/SkCodec.h:336-390
// The nullable native subset pointer stays a pointer: copying options neither
// owns nor extends the caller's rectangle lifetime. Dereferencing remains unsafe.
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
        Self { fZeroInitialized: ZeroInitialized::kNo_ZeroInitialized,
            fSubset: core::ptr::null(), fFrameIndex: 0, fPriorFrame: kNoFrame,
            fMaxDecodeMemory: 0 }
    }
}
// Source: include/codec/SkCodec.h:592-628
#[repr(i32)] #[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkScanlineOrder { kTopDown_SkScanlineOrder, kBottomUp_SkScanlineOrder }
// Source: include/codec/SkCodec.h:679-679
pub const kNoFrame: i32 = -1;
// Source: include/codec/SkCodec.h:684-742
pub struct FrameInfo {
    pub fRequiredFrame: i32,
    pub fDuration: i32,
    pub fFullyReceived: bool,
    pub fAlphaType: SkAlphaType,
    pub fHasAlphaWithinBounds: bool,
    pub fDisposalMethod: SkCodecAnimation::DisposalMethod,
    pub fBlend: SkCodecAnimation::Blend,
    pub fFrameRect: SkIRect,
}
// Source: include/codec/SkCodec.h:783-783
pub const kRepetitionCountInfinite: i32 = -1;
// Source: include/codec/SkCodec.h:830-834
#[repr(i32)] #[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IsAnimated { kYes, kNo, kUnknown }

// Source: include/codec/SkCodec.h:59-59
// Source: include/codec/SkCodec.h:225-225
// Native virtual methods map only to virtual trait slots; none of the native
// nonvirtual wrappers are replaced by provider methods. Unique Box ownership
// supplies concrete dynamic destruction. No decoder implementation is supplied.
// Native defaults requiring the unported SkEncodedInfo/base state or .cpp body
// remain required slots here; their default bodies are explicitly pending.
pub trait SkCodec {
    // Source: include/codec/SkCodec.h:849-850
    unsafe fn onGetGainmapCodec(&mut self, _info: *mut SkGainmapInfo,
        _codec: *mut Option<Box<dyn SkCodec>>) -> bool { false }
    unsafe fn onGetGainmapInfo(&mut self, _info: *mut SkGainmapInfo) -> bool { false }
    // Source: include/codec/SkCodec.h:854-854
    unsafe fn onGetGainmapInfo_stream(&mut self, _info: *mut SkGainmapInfo,
        _stream: *mut Option<Box<dyn SkStream>>) -> bool { false }
    // Source: include/codec/SkCodec.h:856-859
    fn onGetScaledDimensions(&self, desiredScale: f32) -> SkISize;
    // Source: include/codec/SkCodec.h:866-868
    fn onDimensionsSupported(&mut self, _dimensions: &SkISize) -> bool { false }
    // Source: include/codec/SkCodec.h:870-870
    fn onGetEncodedFormat(&self) -> SkEncodedImageFormat;
    // Source: include/codec/SkCodec.h:876-878
    /// Safety: pixels and nullable rowsDecoded satisfy the original writable
    /// allocation, row-stride, subset and exclusive decode contracts.
    unsafe fn onGetPixels(&mut self, info: &SkImageInfo, pixels: *mut c_void,
        rowBytes: usize, options: &Options, rowsDecoded: *mut i32) -> Result;
    // Source: include/codec/SkCodec.h:880-881
    fn onQueryYUVAInfo(&self, _supported: &SupportedDataTypes,
        _info: Option<&mut SkYUVAPixmapInfo>) -> bool { false }
    // Source: include/codec/SkCodec.h:883-883
    unsafe fn onGetYUVAPlanes(&mut self, _pixmaps: &SkYUVAPixmaps) -> Result {
        Result::kUnimplemented
    }
    // Source: include/codec/SkCodec.h:885-888
    fn onGetValidSubset(&self, _desiredSubset: Option<&mut SkIRect>) -> bool { false }
    // Source: include/codec/SkCodec.h:913-918
    fn onRewind(&mut self) -> bool;
    // Source: include/codec/SkCodec.h:934-934
    fn onGetScanlineOrder(&self) -> SkScanlineOrder { SkScanlineOrder::kTopDown_SkScanlineOrder }
    // Source: include/codec/SkCodec.h:948-948
    fn onOutputScanline(&self, inputScanline: i32) -> i32;
    // Source: include/codec/SkCodec.h:955-956
    fn conversionSupported(&mut self, dst: &SkImageInfo, srcIsOpaque: bool,
        needsColorXform: bool) -> bool;
    // Source: include/codec/SkCodec.h:961-961
    fn usesColorXform(&self) -> bool { true }
    // Source: include/codec/SkCodec.h:967-969
    fn onGetFrameCount(&mut self) -> i32 { 1 }
    // Source: include/codec/SkCodec.h:971-973
    fn onGetFrameInfo(&self, _index: i32, _info: Option<&mut FrameInfo>) -> bool { false }
    // Source: include/codec/SkCodec.h:975-977
    fn onGetRepetitionCount(&mut self) -> i32 { 0 }
    // Source: include/codec/SkCodec.h:979-981
    fn onIsAnimated(&mut self) -> IsAnimated { IsAnimated::kNo }
    // Source: include/codec/SkCodec.h:1046-1048
    fn getFrameHolder(&self) -> *const SkFrameHolder { core::ptr::null() }
    // Source: include/codec/SkCodec.h:1069-1072
    fn onStartScanlineDecode(&mut self, _dstInfo: &SkImageInfo,
        _options: &Options) -> Result { Result::kUnimplemented }
    // Source: include/codec/SkCodec.h:1078-1078
    fn onSupportsIncrementalDecode(&mut self, _info: &SkImageInfo) -> bool { false }
    // Source: include/codec/SkCodec.h:1080-1083
    unsafe fn onStartIncrementalDecode(&mut self, _info: &SkImageInfo, _dst: *mut c_void,
        _rowBytes: usize, _options: &Options) -> Result { Result::kUnimplemented }
    // Source: include/codec/SkCodec.h:1085-1087
    unsafe fn onIncrementalDecode(&mut self, _rowsDecoded: *mut i32) -> Result { Result::kUnimplemented }
    // Source: include/codec/SkCodec.h:1090-1090
    fn onSkipScanlines(&mut self, _countLines: i32) -> bool { false }
    // Source: include/codec/SkCodec.h:1092-1092
    unsafe fn onGetScanlines(&mut self, _dst: *mut c_void, _countLines: i32,
        _rowBytes: usize) -> i32 { 0 }
    // Source: include/codec/SkCodec.h:1120-1120
    fn getSampler(&mut self, _createIfNecessary: bool) -> *mut SkSampler { core::ptr::null_mut() }
    // Source: include/codec/SkCodec.h:1126-1126
    fn getEncodedData(&self) -> Option<Arc<SkData>>;
}

// Source: include/codec/SkCodec.h:636-636
impl dyn SkCodec + '_ {
    pub fn getScanlineOrder(&self) -> SkScanlineOrder { self.onGetScanlineOrder() }
    // Source: include/codec/SkCodec.h:670-672
    pub fn getFrameCount(&mut self) -> i32 { self.onGetFrameCount() }
    // Source: include/codec/SkCodec.h:763-768
    pub fn getFrameInfo_index(&self, index: i32, info: Option<&mut FrameInfo>) -> bool {
        if index < 0 { return false; }
        self.onGetFrameInfo(index, info)
    }
    // Source: include/codec/SkCodec.h:803-805
    pub fn getRepetitionCount(&mut self) -> i32 { self.onGetRepetitionCount() }
    // Source: include/codec/SkCodec.h:835-835
    pub fn isAnimated(&mut self) -> IsAnimated { self.onIsAnimated() }
    // Source: include/codec/SkCodec.h:313-313
    pub fn getEncodedFormat(&self) -> SkEncodedImageFormat { self.onGetEncodedFormat() }
    // Source: include/codec/SkCodec.h:306-308
    pub fn getValidSubset(&self, desiredSubset: Option<&mut SkIRect>) -> bool {
        self.onGetValidSubset(desiredSubset)
    }
}
// Source: include/codec/SkCodec.h:1140-1152
pub mod SkCodecs {
    use super::*;
    pub type DecodeContext = *mut c_void;
    pub type IsFormatCallback = unsafe fn(data: *const c_void, len: usize) -> bool;
    pub type MakeFromStreamCallback = unsafe fn(stream: Option<Box<dyn SkStream>>,
        result: *mut Result, context: DecodeContext) -> Option<Box<dyn SkCodec>>;
    // std::string_view is nonowning and may contain NUL; it is not a Rust str.
    pub struct Decoder<'a> {
        pub id: &'a [u8],
        pub isFormat: Option<IsFormatCallback>,
        pub makeFromStream: Option<MakeFromStreamCallback>,
    }
}
