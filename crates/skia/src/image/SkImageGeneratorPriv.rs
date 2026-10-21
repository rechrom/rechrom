// Source: src/image/SkImageGeneratorPriv.h:27-27
// The native namespace is represented by this original header module.
// Source: src/image/SkImageGeneratorPriv.h:33-39
pub use super::SkPictureImageGenerator::MakeFromPicture;
// Source: src/image/SkImageGeneratorPriv.h:41-46
pub use super::SkPictureImageGenerator::MakeFromPicture_default;
// Source: src/image/SkImageGeneratorPriv.h:56-57
// Same actual encoded factory and unique generator result. The borrowed-data
// refcount/virtual decoder graph is not given an invented Send/Sync bound.
// Safety remains the real global factory initialization/synchronization contract;
// default std::nullopt alpha remains explicit None.
pub use crate::src::codec::SkImageGenerator_FromEncoded::MakeFromEncoded;
// Native omitted optional argument, retaining the real factory/global/fallback.
pub unsafe fn MakeFromEncoded_default(
    data: Option<std::sync::Arc<crate::src::core::SkData::SkData>>,
) -> Option<Box<dyn crate::src::core::SkImageGenerator::SkImageGenerator>> {
    unsafe { MakeFromEncoded(data, None) }
}
