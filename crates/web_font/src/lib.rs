#![allow(non_snake_case)]

use std::io;
mod ots_cmap;
mod ots_device_tables;
mod ots_glyf;
mod ots_metrics;
mod ots_name;
mod ots_sfnt;

const MAXIMUM_DECODED_FONT_SIZE: usize = 128 * 1024 * 1024;

fn invalid_font() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "invalid web font")
}

fn be_u16(bytes: &[u8], offset: usize) -> io::Result<u16> {
    let slice = bytes.get(offset..offset + 2).ok_or_else(invalid_font)?;
    Ok(u16::from_be_bytes([slice[0], slice[1]]))
}

fn be_u32(bytes: &[u8], offset: usize) -> io::Result<u32> {
    let slice = bytes.get(offset..offset + 4).ok_or_else(invalid_font)?;
    Ok(u32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

fn validate_sfnt_directory(bytes: &[u8], directory: usize) -> io::Result<()> {
    let signature = bytes
        .get(directory..directory + 4)
        .ok_or_else(invalid_font)?;
    if signature != [0, 1, 0, 0]
        && signature != b"OTTO"
        && signature != b"true"
        && signature != b"typ1"
    {
        return Err(invalid_font());
    }
    let count = be_u16(bytes, directory + 4)? as usize;
    if count == 0 || count > 4096 {
        return Err(invalid_font());
    }
    let record_end = directory
        .checked_add(12)
        .and_then(|x| x.checked_add(count.checked_mul(16)?))
        .ok_or_else(invalid_font)?;
    if record_end > bytes.len() {
        return Err(invalid_font());
    }
    for index in 0..count {
        let record = directory + 12 + index * 16;
        let offset = be_u32(bytes, record + 8)? as usize;
        let length = be_u32(bytes, record + 12)? as usize;
        if length == 0
            || offset
                .checked_add(length)
                .is_none_or(|end| end > bytes.len())
        {
            return Err(invalid_font());
        }
    }
    Ok(())
}

fn validate_sfnt(bytes: &[u8]) -> io::Result<()> {
    if bytes.len() < 12 || bytes.len() > MAXIMUM_DECODED_FONT_SIZE {
        return Err(invalid_font());
    }
    if bytes.starts_with(b"ttcf") {
        let count = be_u32(bytes, 8)? as usize;
        if count == 0
            || count > 4096
            || 12_usize
                .checked_add(count * 4)
                .is_none_or(|end| end > bytes.len())
        {
            return Err(invalid_font());
        }
        for index in 0..count {
            validate_sfnt_directory(bytes, be_u32(bytes, 12 + index * 4)? as usize)?;
        }
    } else {
        validate_sfnt_directory(bytes, 0)?;
    }
    Ok(())
}

// cpp: web_font/web_font_decoder.h:10-13
// cpp: web_font/web_font_decoder.cc:43-55
pub fn DecodeWebFont(bytes: Vec<u8>) -> io::Result<Vec<u8>> {
    if bytes.is_empty() || bytes.len() > MAXIMUM_DECODED_FONT_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid web font size",
        ));
    }
    let decoded = if bytes.starts_with(b"wOFF") {
        wuff::decompress_woff1(&bytes).map_err(|_| invalid_font())?
    } else if bytes.starts_with(b"wOF2") {
        wuff::decompress_woff2(&bytes).map_err(|_| invalid_font())?
    } else {
        bytes
    };
    validate_sfnt(&decoded)?;
    ots_sfnt::Sanitize(&decoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_opentype_and_rejects_invalid_input() {
        assert!(DecodeWebFont(vec![]).is_err());
        assert!(DecodeWebFont(b"not a font".to_vec()).is_err());
        let path = "/System/Library/Fonts/Supplemental/Arial.ttf";
        if let Ok(bytes) = std::fs::read(path) {
            let decoded = DecodeWebFont(bytes).unwrap();
            assert_eq!(&decoded[..4], &[0, 1, 0, 0]);
        }
    }

    #[test]
    fn rejects_out_of_bounds_sfnt_table() {
        let mut bytes = vec![0; 28];
        bytes[..4].copy_from_slice(&[0, 1, 0, 0]);
        bytes[4..6].copy_from_slice(&1_u16.to_be_bytes());
        bytes[12..16].copy_from_slice(b"head");
        bytes[20..24].copy_from_slice(&24_u32.to_be_bytes());
        bytes[24..28].copy_from_slice(&100_u32.to_be_bytes());
        assert!(DecodeWebFont(bytes).is_err());
    }
}
