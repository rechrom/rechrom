use std::{ffi::c_char, io};

#[cfg(rechrom_libjpeg)]
unsafe extern "C" {
    fn rechrom_jpeg_decode_scaled_n32(
        encoded: *const u8,
        encoded_length: usize,
        target_width: i32,
        target_height: i32,
        destination: *mut u8,
        destination_length: usize,
        error_message: *mut c_char,
        error_message_length: usize,
    ) -> i32;
}

/// Decode directly into the exact reduced-resolution N32 allocation libjpeg
/// advertises for this JPEG. `None` means the requested size is not one of the
/// codec's supported 1/8 scale numerators and the caller should scale its full
/// decode instead.
pub fn decode_to_n32(encoded: &[u8], width: i32, height: i32) -> io::Result<Option<Vec<u8>>> {
    if width <= 0 || height <= 0 || !encoded.starts_with(&[0xff, 0xd8]) {
        return Ok(None);
    }
    #[cfg(rechrom_libjpeg)]
    {
        let length = (width as usize)
            .checked_mul(height as usize)
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or_else(|| io::Error::other("scaled JPEG allocation overflow"))?;
        let mut pixels = vec![0; length];
        let mut message = [0 as c_char; 256];
        let result = unsafe {
            rechrom_jpeg_decode_scaled_n32(
                encoded.as_ptr(),
                encoded.len(),
                width,
                height,
                pixels.as_mut_ptr(),
                pixels.len(),
                message.as_mut_ptr(),
                message.len(),
            )
        };
        return match result {
            0 => Ok(Some(pixels)),
            1 => Ok(None),
            _ => {
                let end = message
                    .iter()
                    .position(|byte| *byte == 0)
                    .unwrap_or(message.len());
                let bytes = message[..end]
                    .iter()
                    .map(|byte| *byte as u8)
                    .collect::<Vec<_>>();
                Err(io::Error::other(format!(
                    "scaled JPEG decode failed: {}",
                    String::from_utf8_lossy(&bytes)
                )))
            }
        };
    }
    #[cfg(not(rechrom_libjpeg))]
    {
        let _ = (encoded, width, height);
        Ok(None)
    }
}

#[cfg(all(test, rechrom_libjpeg))]
mod tests {
    use super::decode_to_n32;
    use image::{codecs::jpeg::JpegEncoder, ExtendedColorType};

    fn jpeg(width: u32, height: u32) -> Vec<u8> {
        let pixels = vec![127; width as usize * height as usize * 3];
        let mut encoded = Vec::new();
        JpegEncoder::new_with_quality(&mut encoded, 90)
            .encode(&pixels, width, height, ExtendedColorType::Rgb8)
            .unwrap();
        encoded
    }

    #[test]
    fn reduced_decode_requires_whole_mcus_like_blink() {
        // 16 is a whole number of MCUs for both 4:4:4 (8x8) and 4:2:0
        // (16x16); 12 is not. Blink exposes the former half-size decode but
        // only the original size for the latter.
        assert!(decode_to_n32(&jpeg(16, 16), 8, 8).unwrap().is_some());
        assert!(decode_to_n32(&jpeg(12, 12), 6, 6).unwrap().is_none());
    }
}
