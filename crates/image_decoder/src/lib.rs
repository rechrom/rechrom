pub mod image_decoder;
pub mod skia_image_decoder;

// Private codec implementation; DecodeEngine still injects ImageDecoder.
#[cfg(feature = "avif-backend")]
mod avif_backend;

#[cfg(feature = "avif-backend")]
mod avif_rgb8;
