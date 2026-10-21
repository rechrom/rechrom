// Source: src/codec/SkImageGenerator_FromEncoded.cpp:8-21
// SkCodec/SkCodecImageGenerator are the actual
// native dependencies below, not substituted decoder traits or providers.
use crate::include::core::SkAlphaType::SkAlphaType;
use crate::src::codec::SkCodecImageGenerator::SkCodecImageGenerator;
use crate::src::core::{
    SkData::SkData, SkGraphics::ImageGeneratorFromEncodedDataFactory,
    SkImageGenerator::SkImageGenerator,
};
use crate::src::image::{SkImage::SkImage, SkImage_Lazy::DeferredFromGenerator};
use std::sync::Arc;
// Source: src/codec/SkImageGenerator_FromEncoded.cpp:23-23
static mut gFactory: Option<ImageGeneratorFromEncodedDataFactory> = None;
// Source: src/codec/SkImageGenerator_FromEncoded.cpp:25-31
/// Safety: native global factory reads/writes must be externally synchronized.
/// Install before concurrent calls, or exclude all concurrent readers/writers.
pub unsafe fn SetImageGeneratorFromEncodedDataFactory(
    factory: Option<ImageGeneratorFromEncodedDataFactory>,
) -> Option<ImageGeneratorFromEncodedDataFactory> {
    let prev = unsafe { gFactory };
    unsafe {
        gFactory = factory;
    }
    prev
}
// Source: src/codec/SkImageGenerator_FromEncoded.cpp:35-46
/// Safety: synchronize gFactory as for its setter; registered callback obeys its
/// true owned immutable-data/unique-generator transfer contract.
pub unsafe fn MakeFromEncoded(
    data: Option<Arc<SkData>>,
    at: Option<SkAlphaType>,
) -> Option<Box<dyn SkImageGenerator>> {
    if data.is_none() || at == Some(SkAlphaType::kOpaque_SkAlphaType) {
        return None;
    }
    if let Some(factory) = unsafe { gFactory } {
        // Native gFactory(data) copies its sk_sp argument, retaining the original
        // input for the real codec fallback when the callback returns null.
        if let Some(generator) = unsafe { factory(data.clone()) } {
            return Some(generator);
        }
    }
    unsafe { SkCodecImageGenerator::MakeFromEncodedCodec(data, at) }
}
// Source: src/codec/SkImageGenerator_FromEncoded.cpp:52-58
/// Safety: same native factory synchronization and callback contract.
pub unsafe fn DeferredFromEncodedData(
    encoded: Option<Arc<SkData>>,
    alpha_type: Option<SkAlphaType>,
) -> Option<Arc<dyn SkImage>> {
    let encoded = encoded?;
    if encoded.empty() {
        return None;
    }
    DeferredFromGenerator(unsafe { MakeFromEncoded(Some(encoded), alpha_type) })
}
// Source: src/codec/SkImageGenerator_FromEncoded.cpp:64-67
pub fn DeferredImage(
    codec: Option<core::pin::Pin<Box<dyn super::SkCodec::SkCodec>>>,
    alpha_type: Option<SkAlphaType>,
) -> Option<Arc<dyn SkImage>> {
    DeferredFromGenerator(SkCodecImageGenerator::MakeFromCodec(codec, alpha_type))
}

// Source: include/codec/SkCodec.h:1174-1175.
// Native optional alpha default; transfer the same existing unique Codec owner.
pub fn DeferredImage_default(
    codec: Option<core::pin::Pin<Box<dyn super::SkCodec::SkCodec>>>,
) -> Option<Arc<dyn SkImage>> {
    DeferredImage(codec, None)
}
