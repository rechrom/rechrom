//! ImageDecoder::Create/SniffMimeTypeInternal admission, fixed Chromium
//! 6c1d401fcca5e1b0030563a90c2f2bba168e0c15, image_decoder.cc:153-246,294-382.
//! Header recognition selects the actual enabled codec; it never decodes all
//! pixels merely to decide whether a DataObject may carry a JS File as image.
use crate::{
    image_decoder::{DecodedImage, ImageDecodeInput, ImageDecoder},
    skia_image_decoder::SkiaImageDecoder,
};
use std::{io, sync::Arc};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageCodec {
    Jpeg,
    Png,
    Gif,
    Webp,
    Icon,
    Bmp,
    #[cfg(feature = "avif-backend")]
    Avif,
}
impl ImageCodec {
    pub fn FilenameExtension(self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::Gif => "gif",
            Self::Webp => "webp",
            Self::Icon => "ico",
            Self::Bmp => "bmp",
            #[cfg(feature = "avif-backend")]
            Self::Avif => "avif",
        }
    }
    pub fn MimeType(self) -> &'static str {
        match self {
            Self::Jpeg => "image/jpeg",
            Self::Png => "image/png",
            Self::Gif => "image/gif",
            Self::Webp => "image/webp",
            Self::Icon => "image/x-icon",
            Self::Bmp => "image/bmp",
            #[cfg(feature = "avif-backend")]
            Self::Avif => "image/avif",
        }
    }
}
pub struct AdmittedImageDecoder {
    data: Arc<[u8]>,
    codec: ImageCodec,
    decoder: SkiaImageDecoder,
}
impl AdmittedImageDecoder {
    pub fn Codec(&self) -> ImageCodec {
        self.codec
    }
    pub fn Decode(&mut self) -> io::Result<DecodedImage> {
        self.decoder.Decode(&ImageDecodeInput {
            bytes: &self.data,
            mime_type: self.codec.MimeType(),
        })
    }
}
pub fn SniffImageCodec(data: &[u8], data_complete: bool) -> Option<ImageCodec> {
    if data.len() < 14 && !data_complete {
        return None;
    }
    // Complete short input is zero-padded exactly like FastSharedBufferReader
    // in source. Only this fixed signature prefix is copied, never the Blob.
    let mut prefix = [0u8; 14];
    let count = data.len().min(prefix.len());
    prefix[..count].copy_from_slice(&data[..count]);
    let p = &prefix;
    if p.starts_with(b"\xff\xd8\xff") {
        return Some(ImageCodec::Jpeg);
    }
    if p.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Some(ImageCodec::Png);
    }
    if p.starts_with(b"GIF87a") || p.starts_with(b"GIF89a") {
        return Some(ImageCodec::Gif);
    }
    if &p[..4] == b"RIFF" && &p[8..14] == b"WEBPVP" {
        return Some(ImageCodec::Webp);
    }
    if p.starts_with(b"\0\0\x01\0") || p.starts_with(b"\0\0\x02\0") {
        return Some(ImageCodec::Icon);
    }
    if p.starts_with(b"BM") || p.starts_with(b"BA") {
        return Some(ImageCodec::Bmp);
    }
    #[cfg(feature = "avif-backend")]
    if crabby_avif::decoder::Decoder::peek_compatible_file_type(&data[..data.len().min(144)]) {
        return Some(ImageCodec::Avif);
    }
    // This product has no ENABLE_JXL_DECODER backend. The conditional source
    // branch does not admit a codec which its decoder owner cannot construct.
    None
}
pub fn CreateImageDecoderForData(
    data: Arc<[u8]>,
    data_complete: bool,
) -> Option<AdmittedImageDecoder> {
    let codec = SniffImageCodec(&data, data_complete)?;
    Some(AdmittedImageDecoder {
        data,
        codec,
        decoder: SkiaImageDecoder,
    })
}
