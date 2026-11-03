#![allow(non_snake_case)]

use std::io;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DecodedImageFrame {
    pub duration: std::time::Duration,
    pub rgba8: Vec<u8>,
}

// cpp: image_decoder/image_decoder.h:14-17
#[derive(Clone, Copy, Debug)]
pub struct ImageDecodeInput<'a> {
    pub bytes: &'a [u8],
    pub mime_type: &'a str,
}

// cpp: image_decoder/image_decoder.h:19-23
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
    pub orientation: image_resource::ImageOrientation,
    /// Representation of `rgba8` and every retained animation frame.
    pub alpha_type: image_resource::PaintImageAlphaType,
    /// Source PaintImageGenerator frame metadata, when the codec's complete
    /// frame owner is available. Unknown animation is explicit, not frame 1.
    pub animation: image_resource::PaintImageAnimationMetadata,
    /// Complete composited frames for an animated bitmap. The first entry is
    /// the same visual frame as `rgba8`; retaining the complete set lets the
    /// compositor select the frame index produced by its image animation
    /// controller, matching cc::PaintImageGenerator ownership.
    pub animation_frames: Vec<DecodedImageFrame>,
    pub rgba8: Vec<u8>,
}

// cpp: image_decoder/image_decoder.h:25-29
pub trait ImageDecoder {
    fn BackgroundFactory(
        &self,
    ) -> Option<std::sync::Arc<dyn Fn() -> Box<dyn ImageDecoder> + Send + Sync>> {
        None
    }
    fn Decode(&mut self, input: &ImageDecodeInput<'_>) -> io::Result<DecodedImage>;
}
