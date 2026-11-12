// Copyright 2008 The Chromium Authors
// Use of this source code is governed by a BSD-style license.
//! Source: platform/image-decoders/ico/ico_image_decoder.h:43-65,115-117;
//! ico_image_decoder.cc:88-113,244-349. The resident decoder publishes frame 0.
#![allow(non_snake_case)]

use crate::image_decoder::DecodedImage;
use image_resource::{ImageHotSpot, ImageOrientation, PaintImageAlphaType};
use std::io::{self, BufReader, Cursor, Read, Seek, SeekFrom};

const DIRECTORY_SIZE: usize = 6;
const ENTRY_SIZE: usize = 16;
const ICON: u16 = 1;
const CURSOR: u16 = 2;

// Source ico_image_decoder.h:51-57; .cc:306-349.
struct IconDirectoryEntry {
    bytes: [u8; ENTRY_SIZE],
    width: u32,
    height: u32,
    bit_count: u16,
    hot_spot: Option<ImageHotSpot>,
    image_offset: usize,
}
impl IconDirectoryEntry {
    fn Read(bytes: &[u8], file_type: u16) -> Self {
        let width = if bytes[0] == 0 {
            256
        } else {
            u32::from(bytes[0])
        };
        let height = if bytes[1] == 0 {
            256
        } else {
            u32::from(bytes[1])
        };
        let mut bit_count = if file_type == CURSOR {
            0
        } else {
            U16(bytes, 6)
        };
        let hot_spot = (file_type == CURSOR).then(|| ImageHotSpot {
            x: i32::from(U16(bytes, 4)),
            y: i32::from(U16(bytes, 6)),
        });
        if bit_count == 0 {
            let mut color_count = if bytes[2] == 0 {
                256u16
            } else {
                u16::from(bytes[2])
            };
            color_count -= 1;
            while color_count != 0 {
                bit_count += 1;
                color_count >>= 1;
            }
        }
        Self {
            bytes: bytes.try_into().expect("complete directory entry"),
            width,
            height,
            bit_count,
            hot_spot,
            image_offset: u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize,
        }
    }
    // Source ico_image_decoder.cc:105-113. CUR hotspot words are not bpp.
    fn BetterThan(&self, other: &Self) -> bool {
        (self.width * self.height, self.bit_count) > (other.width * other.height, other.bit_count)
    }
}
fn U16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}
fn Invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

// Source ProcessDirectory/ProcessDirectoryEntries, cc:244-304. The pixel
// backend has no frame-index API, so a 22-byte header view admits exactly the
// selected source entry while all original payload bytes remain borrowed at
// their original offsets. No image data or whole container is copied.
fn FrameZero(bytes: &[u8]) -> io::Result<IconDirectoryEntry> {
    if bytes.len() < DIRECTORY_SIZE {
        return Err(Invalid("incomplete ICO directory"));
    }
    let file_type = U16(bytes, 2);
    let count = usize::from(U16(bytes, 4));
    if !matches!(file_type, ICON | CURSOR) || count == 0 {
        return Err(Invalid("invalid ICO directory type or count"));
    }
    let directory_end = DIRECTORY_SIZE + count * ENTRY_SIZE;
    if directory_end > bytes.len() {
        return Err(Invalid("incomplete ICO directory entries"));
    }
    let mut best: Option<IconDirectoryEntry> = None;
    for entry in bytes[DIRECTORY_SIZE..directory_end].chunks_exact(ENTRY_SIZE) {
        let entry = IconDirectoryEntry::Read(entry, file_type);
        if entry.image_offset < directory_end {
            return Err(Invalid("ICO image overlaps directory"));
        }
        if best.as_ref().is_none_or(|best| entry.BetterThan(best)) {
            best = Some(entry);
        }
    }
    Ok(best.expect("nonempty checked directory"))
}

struct FrameZeroReader<'a> {
    bytes: Cursor<&'a [u8]>,
    header: [u8; DIRECTORY_SIZE + ENTRY_SIZE],
}
impl<'a> FrameZeroReader<'a> {
    fn new(bytes: &'a [u8], entry: &IconDirectoryEntry) -> Self {
        let mut header = [0; DIRECTORY_SIZE + ENTRY_SIZE];
        header[2..4].copy_from_slice(&ICON.to_le_bytes());
        header[4..6].copy_from_slice(&1u16.to_le_bytes());
        header[DIRECTORY_SIZE..].copy_from_slice(&entry.bytes);
        // These words are backend header hints only. The original CUR hotspot
        // is already retained above and is never reinterpreted as color depth.
        // Pixel depth is read by the existing PNG/BMP pixel codec itself.
        header[DIRECTORY_SIZE + 4..DIRECTORY_SIZE + 8].fill(0);
        Self {
            bytes: Cursor::new(bytes),
            header,
        }
    }
}
impl Read for FrameZeroReader<'_> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        let start = self.bytes.position();
        let count = self.bytes.read(out)?;
        if let Ok(start) = usize::try_from(start) {
            if start < self.header.len() {
                let overlap = count.min(self.header.len() - start);
                out[..overlap].copy_from_slice(&self.header[start..start + overlap]);
            }
        }
        Ok(count)
    }
}
impl Seek for FrameZeroReader<'_> {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        self.bytes.seek(position)
    }
}

pub(crate) fn Decode(bytes: &[u8]) -> io::Result<DecodedImage> {
    let frame = FrameZero(bytes)?;
    let decoder =
        image::codecs::ico::IcoDecoder::new(BufReader::new(FrameZeroReader::new(bytes, &frame)))
            .map_err(io::Error::other)?;
    let decoded = image::DynamicImage::from_decoder(decoder).map_err(io::Error::other)?;
    // ICOImageDecoder::SetSize validates the embedded PNG/BMP dimensions
    // against the chosen directory entry (cc:65-71,203-238).
    if decoded.width() != frame.width || decoded.height() != frame.height {
        return Err(Invalid(
            "ICO decoded frame dimensions differ from directory",
        ));
    }
    Ok(DecodedImage {
        filename_extension: Some("ico"),
        width: frame.width,
        height: frame.height,
        orientation: ImageOrientation::kDefault,
        hot_spot: frame.hot_spot,
        alpha_type: PaintImageAlphaType::Unpremultiplied,
        animation: image_resource::PaintImageAnimationMetadata::Known(
            image_resource::PaintImageAnimationInfo::confirmed_static(),
        ),
        animation_frames: Vec::new(),
        rgba8: decoded.into_rgba8().into_raw(),
    })
}
