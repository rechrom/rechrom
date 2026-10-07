#![allow(non_snake_case)]

use std::io;

use layoutng_assembly::internal::layout_input::ConstraintSpace;

use crate::image_decoder::{DecodedImage, ImageDecodeInput};

// cpp: image_decoder/document_image_decoder.h:7-18
pub trait DocumentImageDecoder {
    fn CanDecode(&self, input: &ImageDecodeInput<'_>) -> bool;
    fn Decode(
        &mut self,
        input: &ImageDecodeInput<'_>,
        host_constraints: &ConstraintSpace,
    ) -> io::Result<DecodedImage>;
}
