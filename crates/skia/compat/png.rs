//! Local uncompressed PNG byte stream encoder; official SkPngEncoderImpl uses
//! libpng and is not implemented by this module.
//! RGBA PNG byte stream encoder used by screenshot adapters.
fn be32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffff_u32;
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb8_8320 & (0_u32.wrapping_sub(crc & 1)));
        }
    }
    !crc
}

fn adler32(bytes: &[u8]) -> u32 {
    let (mut a, mut b) = (1_u32, 0_u32);
    for &byte in bytes {
        a = (a + u32::from(byte)) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

fn chunk(png: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    be32(png, data.len().try_into().expect("PNG chunk too large"));
    let start = png.len();
    png.extend_from_slice(kind);
    png.extend_from_slice(data);
    be32(png, crc32(&png[start..]));
}

#[allow(non_snake_case)]
pub fn EncodeRgbaPng(rgba: &[u8], width: u32, height: u32) -> Vec<u8> {
    assert_eq!(rgba.len(), width as usize * height as usize * 4);
    let mut scanlines = Vec::with_capacity(rgba.len() + height as usize);
    for row in rgba.chunks_exact(width as usize * 4) {
        scanlines.push(0);
        scanlines.extend_from_slice(row);
    }
    let mut compressed = vec![0x78, 0x01];
    for (index, block) in scanlines.chunks(65535).enumerate() {
        let final_block = (index + 1) * 65535 >= scanlines.len();
        compressed.push(if final_block { 1 } else { 0 });
        let length = block.len() as u16;
        compressed.extend_from_slice(&length.to_le_bytes());
        compressed.extend_from_slice(&(!length).to_le_bytes());
        compressed.extend_from_slice(block);
    }
    be32(&mut compressed, adler32(&scanlines));
    let mut png = vec![137, 80, 78, 71, 13, 10, 26, 10];
    let mut header = Vec::with_capacity(13);
    be32(&mut header, width);
    be32(&mut header, height);
    header.extend_from_slice(&[8, 6, 0, 0, 0]);
    chunk(&mut png, b"IHDR", &header);
    chunk(&mut png, b"IDAT", &compressed);
    chunk(&mut png, b"IEND", &[]);
    png
}
