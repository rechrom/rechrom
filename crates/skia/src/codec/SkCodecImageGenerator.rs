// Copyright 2015 Google Inc. BSD-style license.
use super::SkCodec::{FrameInfo, Options, Result as CodecResult, SkCodec, SkCodecExt};
use crate::include::{
    codec::SkEncodedOrigin::SkEncodedOriginSwapsWidthHeight,
    core::{SkAlphaType::SkAlphaType, SkSize::SkISize},
};
use crate::src::core::{
    SkData::SkData,
    SkImageGenerator::{Options as GeneratorOptions, SkImageGenerator, SkImageGeneratorState},
    SkImageInfo::SkImageInfo,
    SkPixmap::SkPixmap,
    SkYUVAPixmaps::{SkYUVAPixmapInfo, SkYUVAPixmaps, SupportedDataTypes},
};
use core::{cell::UnsafeCell, pin::Pin};
use std::sync::Arc;
// Source: src/codec/SkCodecImageGenerator.h:24-24
// Source: src/codec/SkCodecImageGenerator.h:124-125
// Private reverse representation: cached Data, actual pinned unique Codec,
// then original Generator state. Pin moves only its original owner handle.
// Native SkImageGenerator has no global Send constraint. This same unique
// Each original mutable pointee/cache field has its own UnsafeCell; the base
// metadata is never included in a complete-object mutable borrow. The same
// Codec stays local unless an actual outer transfer/access contract proves the
// complete stream/decoder graph; no unsafe Send is fabricated.
pub struct SkCodecImageGenerator {
    // Source: src/codec/SkCodecImageGenerator.h:125-125
    fCachedData: UnsafeCell<Option<Arc<SkData>>>,
    // Source: src/codec/SkCodecImageGenerator.h:124-124
    fCodec: UnsafeCell<Pin<Box<dyn SkCodec>>>,
    // Source: src/codec/SkCodecImageGenerator.h:24-24
    base: SkImageGeneratorState,
}
impl SkCodecImageGenerator {
    // Source: src/codec/SkCodecImageGenerator.h:30-31
    // Source: src/codec/SkCodecImageGenerator.cpp:23-31
    pub unsafe fn MakeFromEncodedCodec(
        data: Option<Arc<SkData>>,
        at: Option<SkAlphaType>,
    ) -> Option<Box<dyn SkImageGenerator>> {
        let codec = unsafe { super::SkCodec::MakeFromData(data.clone(), None) }?;
        // Native original Data parameter remains alive until this return; the
        // factory's by-value argument above is its actual sk_sp retain, not a new owner.
        Some(Box::new(Self::new(codec, at)))
    }
    // Source: src/codec/SkCodecImageGenerator.h:30-31
    pub unsafe fn MakeFromEncodedCodec_default(
        data: Option<Arc<SkData>>,
    ) -> Option<Box<dyn SkImageGenerator>> {
        unsafe { Self::MakeFromEncodedCodec(data, None) }
    }
    // Source: src/codec/SkCodecImageGenerator.h:33-34
    // Source: src/codec/SkCodecImageGenerator.cpp:33-38
    pub fn MakeFromCodec(
        codec: Option<Pin<Box<dyn SkCodec>>>,
        at: Option<SkAlphaType>,
    ) -> Option<Box<dyn SkImageGenerator>> {
        codec.map(|codec| Box::new(Self::new(codec, at)) as Box<dyn SkImageGenerator>)
    }
    // Source: src/codec/SkCodecImageGenerator.h:33-34
    pub fn MakeFromCodec_default(
        codec: Option<Pin<Box<dyn SkCodec>>>,
    ) -> Option<Box<dyn SkImageGenerator>> {
        Self::MakeFromCodec(codec, None)
    }
    // Source: src/codec/SkCodecImageGenerator.h:122-122
    // Source: src/codec/SkCodecImageGenerator.cpp:56-58
    fn new(codec: Pin<Box<dyn SkCodec>>, at: Option<SkAlphaType>) -> Self {
        let base = SkImageGeneratorState::new(&adjust_info(codec.as_ref().get_ref(), at), 0);
        Self {
            base,
            fCodec: UnsafeCell::new(codec),
            fCachedData: UnsafeCell::new(None),
        }
    }
    // Source: src/codec/SkCodecImageGenerator.cpp:60-67
    /// Safety: the original exclusive generator operation lease is held.
    unsafe fn onRefEncodedData(&self) -> Option<Arc<SkData>> {
        let cached = self.fCachedData.get();
        // Do not hold a cache reference across the actual virtual encoded-data call.
        if unsafe { (&*cached).is_none() } {
            let data = super::SkCodecPriv::SkCodecPriv::GetEncodedData(unsafe {
                (&*self.fCodec.get()).as_ref().get_ref()
            });
            unsafe {
                *cached = data;
            }
        }
        unsafe { (&*cached).clone() }
    }
    // Source: src/codec/SkCodecImageGenerator.h:69-69
    // Source: src/codec/SkCodecImageGenerator.cpp:69-86
    /// Safety: destination retains the native exclusive writable pixel lease;
    /// options and all Codec input/backing remain live through decoding/orienting.
    pub unsafe fn getPixels(
        &self,
        info: &SkImageInfo,
        pixels: *mut u8,
        row_bytes: usize,
        options: Option<&Options>,
    ) -> bool {
        let dst = SkPixmap::new(info, pixels, row_bytes);
        let origin = unsafe { (&*self.fCodec.get()).as_ref().get_ref().getOrigin() };
        let mut decode = |pm: &SkPixmap| {
            let result = unsafe {
                (&mut *self.fCodec.get())
                    .as_mut()
                    .getPixels_pixmap(pm, options)
            };
            match result {
                CodecResult::kSuccess
                | CodecResult::kIncompleteInput
                | CodecResult::kErrorInInput => true,
                _ => false,
            }
        };
        // Exact native temporary/orientation producer; no alternate image decoder.
        match unsafe { super::SkPixmapUtilsPriv::OrientWithDecode(&dst, origin, &mut decode) } {
            Ok(decoded) => decoded,
            // Adapted/uncredited Rust integration: the actual orientation draw can
            // fail with RasterError. Native Generator returns bool; report that real
            // operation failure as generator failure, never successful decoded pixels.
            Err(_) => false,
        }
    }
    // Source: src/codec/SkCodecImageGenerator.h:69-69
    pub unsafe fn getPixels_default(
        &self,
        info: &SkImageInfo,
        pixels: *mut u8,
        row_bytes: usize,
    ) -> bool {
        unsafe { self.getPixels(info, pixels, row_bytes, None) }
    }
    // Source: src/codec/SkCodecImageGenerator.h:76-76
    /// Safety: hold the original exclusive Codec payload operation lease.
    pub unsafe fn getFrameCount(&self) -> i32 {
        unsafe { (&mut *self.fCodec.get()).as_mut().getFrameCount() }
    }
    // Source: src/codec/SkCodecImageGenerator.h:85-87
    pub fn getFrameInfo(&self, index: i32, info: Option<&mut FrameInfo>) -> bool {
        unsafe { (&*self.fCodec.get()).as_ref().get_ref() }.getFrameInfo_index(index, info)
    }
    // Source: src/codec/SkCodecImageGenerator.h:103-103
    /// Safety: hold the original exclusive Codec payload operation lease.
    pub unsafe fn getRepetitionCount(&self) -> i32 {
        unsafe { (&mut *self.fCodec.get()).as_mut().getRepetitionCount() }
    }
    // Source: src/codec/SkCodecImageGenerator.h:45-45
    // Source: src/codec/SkCodecImageGenerator.cpp:107-114
    pub fn getScaledDimensions(&self, desired_scale: f32) -> SkISize {
        let mut size =
            unsafe { (&*self.fCodec.get()).as_ref().get_ref() }.getScaledDimensions(desired_scale);
        if SkEncodedOriginSwapsWidthHeight(
            unsafe { (&*self.fCodec.get()).as_ref().get_ref() }.getOrigin(),
        ) {
            core::mem::swap(&mut size.fWidth, &mut size.fHeight);
        }
        size
    }
}
// Source: src/codec/SkCodecImageGenerator.cpp:40-54
fn adjust_info(codec: &dyn SkCodec, at: Option<SkAlphaType>) -> SkImageInfo {
    crate::SkASSERT!(@source b"src/codec/SkCodecImageGenerator.cpp\0".as_ptr(),41; @condition b"at != kOpaque_SkAlphaType\0".as_ptr(); at!=Some(SkAlphaType::kOpaque_SkAlphaType));
    let mut info = codec.getInfo();
    if let Some(at) = at {
        info = info.makeAlphaType(at);
    } else if info.alphaType() == SkAlphaType::kUnpremul_SkAlphaType {
        info = info.makeAlphaType(SkAlphaType::kPremul_SkAlphaType);
    }
    if SkEncodedOriginSwapsWidthHeight(codec.getOrigin()) {
        info = super::SkPixmapUtils::SwapWidthHeight(&info);
    }
    info
}
// Native virtual forwarding to this actual complete concrete owner. The only
// interface has no fabricated global Send requirement.
impl SkImageGenerator for SkCodecImageGenerator {
    fn state(&self) -> &SkImageGeneratorState {
        &self.base
    }
    unsafe fn onRefEncodedData(&self) -> Option<Arc<SkData>> {
        unsafe { Self::onRefEncodedData(self) }
    }
    // Source: src/codec/SkCodecImageGenerator.h:108-111
    // Source: src/codec/SkCodecImageGenerator.cpp:88-91
    unsafe fn onGetPixels(
        &self,
        info: &SkImageInfo,
        pixels: *mut u8,
        row_bytes: usize,
        _options: &GeneratorOptions,
    ) -> bool {
        unsafe { self.getPixels(info, pixels, row_bytes, None) }
    }
    // Source: src/codec/SkCodecImageGenerator.h:113-114
    // Source: src/codec/SkCodecImageGenerator.cpp:93-97
    fn onQueryYUVAInfo(
        &self,
        supported: &SupportedDataTypes,
        info: Option<&mut SkYUVAPixmapInfo>,
    ) -> bool {
        unsafe { (&*self.fCodec.get()).as_ref().get_ref() }.queryYUVAInfo(supported, info)
    }
    // Source: src/codec/SkCodecImageGenerator.h:116-116
    // Source: src/codec/SkCodecImageGenerator.cpp:99-105
    unsafe fn onGetYUVAPlanes(&self, pixmaps: &SkYUVAPixmaps) -> bool {
        match unsafe { unsafe { (&mut *self.fCodec.get()).as_mut().getYUVAPlanes(pixmaps) } } {
            CodecResult::kSuccess | CodecResult::kIncompleteInput | CodecResult::kErrorInInput => {
                true
            }
            _ => false,
        }
    }
}
