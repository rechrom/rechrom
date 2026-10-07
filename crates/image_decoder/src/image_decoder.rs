#![allow(non_snake_case)]

use std::io;

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
    pub rgba8: Vec<u8>,
}

// cpp: image_decoder/image_decoder.h:25-29
pub trait ImageDecoder {
    fn Decode(&mut self, input: &ImageDecodeInput<'_>) -> io::Result<DecodedImage>;
}
