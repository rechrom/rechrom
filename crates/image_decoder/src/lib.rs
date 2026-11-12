pub mod decoder_admission;
mod ico_image_decoder;
pub mod image_decoder;
pub mod jpeg_scaled_decode;
pub mod skia_image_decoder;
pub mod video_frame_decoder;

// Private codec implementation; DecodeEngine still injects ImageDecoder.
#[cfg(feature = "avif-backend")]
mod avif_backend;

#[cfg(feature = "avif-backend")]
mod avif_rgb8;
