// Copyright (c) 2009-2017 The OTS Authors. All rights reserved.
// Use of this source code is governed by the BSD-style OTS license.
use std::io;
fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "invalid web font")
}
fn u16_at(d: &[u8], i: usize) -> io::Result<u16> {
    Ok(u16::from_be_bytes(
        d.get(i..i + 2).ok_or_else(invalid)?.try_into().unwrap(),
    ))
}
fn u32_at(d: &[u8], i: usize) -> io::Result<u32> {
    Ok(u32::from_be_bytes(
        d.get(i..i + 4).ok_or_else(invalid)?.try_into().unwrap(),
    ))
}
fn set16(d: &mut [u8], i: usize, x: u16) {
    d[i..i + 2].copy_from_slice(&x.to_be_bytes());
}

// C++ uses offsetof(OS2Data, ...) on this native struct, not wire offsets.
#[repr(C)]
struct OS2Layout {
    basic: [u16; 16],
    panose: [u8; 10],
    ranges_vendor: [u32; 5],
    selection_through_win: [u16; 8],
    code_page_range_1: u32,
    code_page_range_2: u32,
    x_height: i16,
    cap_height: i16,
    default_char: u16,
    break_char: u16,
    max_context: u16,
    lower: u16,
    upper: u16,
}
// cpp: web_font/third_party/ots/src/os2.cc:15-318
pub(crate) fn OS2(data: &[u8], head: &mut [u8]) -> io::Result<Vec<u8>> {
    let mut version = u16_at(data, 0)?;
    if version > 5 || data.len() < 78 {
        return Err(invalid());
    }
    let mut out = data[..78].to_vec();
    set16(&mut out, 4, u16_at(data, 4)?.clamp(1, 1000));
    set16(&mut out, 6, u16_at(data, 6)?.clamp(1, 9));
    let mut kind = u16_at(data, 8)?;
    if kind & 2 != 0 {
        kind &= 0xfff3;
    } else if kind & 4 != 0 {
        kind &= 0xfff4;
    } else if kind & 8 != 0 {
        kind &= 0xfff9;
    }
    set16(&mut out, 8, kind & 0x30f);
    for offset in [10, 12, 18, 20, 26] {
        if (u16_at(data, offset)? as i16) < 0 {
            set16(&mut out, offset, 0);
        }
    }
    let mut selection = u16_at(data, 62)?;
    if selection & 0x40 != 0 {
        selection &= 0xffde;
    }
    let mut mac = u16_at(head, 44)?;
    if selection & 1 != 0 {
        mac |= 2;
    }
    if selection & 2 != 0 {
        mac |= 4;
    }
    if selection & 0x40 != 0 {
        mac &= 0xfffc;
    }
    set16(head, 44, mac);
    set16(&mut out, 62, selection & 0x3ff);
    if u16_at(data, 64)? > u16_at(data, 66)? {
        set16(&mut out, 64, u16_at(data, 66)?);
    }
    if (u16_at(data, 72)? as i16) < 0 {
        set16(&mut out, 72, 0);
    }
    if version >= 1 {
        if data.len() < std::mem::offset_of!(OS2Layout, code_page_range_2) {
            version = 0;
        } else {
            out.extend_from_slice(data.get(78..86).ok_or_else(invalid)?);
            if version >= 2 {
                if data.len() < std::mem::offset_of!(OS2Layout, max_context) {
                    version = 1;
                } else {
                    out.extend_from_slice(data.get(86..96).ok_or_else(invalid)?);
                    for offset in [86, 88] {
                        if (u16_at(data, offset)? as i16) < 0 {
                            set16(&mut out, offset, 0);
                        }
                    }
                    if version >= 5 {
                        out.extend_from_slice(data.get(96..100).ok_or_else(invalid)?);
                        set16(&mut out, 96, u16_at(data, 96)?.min(0xfffe));
                        set16(&mut out, 98, u16_at(data, 98)?.max(2));
                    }
                }
            }
        }
    }
    set16(&mut out, 0, version);
    Ok(out)
}

// cpp: web_font/third_party/ots/src/hhea.cc:15-26
// cpp: web_font/third_party/ots/src/vhea.cc:15-29
// cpp: web_font/third_party/ots/src/metrics.cc:16-104
pub(crate) fn MetricsHeader(
    data: &[u8],
    head: &[u8],
    glyphs: u16,
    vertical: bool,
) -> io::Result<Vec<u8>> {
    let version = u32_at(data, 0)?;
    if (vertical && !matches!(version, 0x10000 | 0x11000))
        || (!vertical && version >> 16 != 1)
        || data.len() < 36
        || u16_at(data, 32)? != 0
        || u16_at(data, 34)? > glyphs
    {
        return Err(invalid());
    }
    let mut out = data[..36].to_vec();
    out[24..34].fill(0);
    for offset in [4, 8] {
        if (u16_at(data, offset)? as i16) < 0 {
            set16(&mut out, offset, 0);
        }
    }
    if u16_at(head, 44)? & 2 == 0 {
        set16(&mut out, 22, 0);
    }
    Ok(out)
}
// cpp: web_font/third_party/ots/src/metrics.cc:106-175
pub(crate) fn Metrics(data: &[u8], header: &[u8], glyphs: u16) -> io::Result<Vec<u8>> {
    let count = u16_at(header, 34)?;
    if count == 0 || count > glyphs {
        return Err(invalid());
    }
    let size = usize::from(count) * 4 + usize::from(glyphs - count) * 2;
    Ok(data.get(..size).ok_or_else(invalid)?.to_vec())
}

// cpp: web_font/third_party/ots/src/post.cc:14-175
pub(crate) fn Post(data: &[u8], glyphs: u16, cff: bool) -> io::Result<Vec<u8>> {
    let mut version = u32_at(data, 0)?;
    if !matches!(version, 0x10000 | 0x20000 | 0x30000) || data.len() < 32 {
        return Err(invalid());
    }
    let mut out = data[..32].to_vec();
    out[16..32].fill(0);
    if (u16_at(data, 10)? as i16) < 0 {
        set16(&mut out, 10, 1);
    }
    let mut tail = Vec::new();
    if version == 0x20000 {
        let count = u16_at(data, 32)?;
        if count == 0 {
            if glyphs > 258 {
                return Err(invalid());
            }
            version = 0x10000;
        } else {
            if count != glyphs {
                return Err(invalid());
            }
            let string_start = 34 + usize::from(count) * 2;
            let names = data.get(34..string_start).ok_or_else(invalid)?;
            let mut position = string_start;
            let mut strings = 0usize;
            while position < data.len() {
                let length = usize::from(data[position]);
                position += 1;
                let text = data.get(position..position + length).ok_or_else(invalid)?;
                if text.contains(&0) {
                    return Err(invalid());
                }
                strings += 1;
                position += length;
            }
            for index in names.chunks_exact(2) {
                let id = u16::from_be_bytes(index.try_into().unwrap());
                if id >= 258 && usize::from(id - 258) >= strings {
                    return Err(invalid());
                }
            }
            tail.extend_from_slice(&data[32..]);
        }
    }
    if cff {
        version = 0x30000;
    }
    out[..4].copy_from_slice(&version.to_be_bytes());
    if version == 0x20000 {
        out.extend_from_slice(&tail);
    }
    Ok(out)
}

// cpp: web_font/third_party/ots/src/hdmx.cc:16-118
pub(crate) fn Hdmx(data: &[u8], head: &[u8], glyphs: u16) -> io::Result<Option<Vec<u8>>> {
    if u16_at(head, 16)? & 0x14 == 0 {
        return Ok(None);
    }
    let version = u16_at(data, 0)?;
    let count = u16_at(data, 2)? as i16;
    let size = u32_at(data, 4)? as i32;
    let actual = i32::from(glyphs) + 2;
    if version != 0 || count <= 0 || size < actual {
        return Ok(None);
    }
    let pad = size - actual;
    if pad > 3 {
        return Err(invalid());
    }
    let mut out = data.get(..8).ok_or_else(invalid)?.to_vec();
    let mut previous = 0;
    for i in 0..count {
        let p = 8 + i as usize * size as usize;
        let record = data.get(p..p + size as usize).ok_or_else(invalid)?;
        if i != 0 && record[0] <= previous {
            return Ok(None);
        }
        previous = record[0];
        out.extend_from_slice(&record[..actual as usize]);
        out.resize(out.len() + pad as usize, 0);
    }
    Ok(Some(out))
}

// cpp: web_font/third_party/ots/src/gasp.cc:14-82
pub(crate) fn Gasp(data: &[u8]) -> io::Result<Option<Vec<u8>>> {
    let mut version = u16_at(data, 0)?;
    let count = u16_at(data, 2)?;
    if version > 1 || count == 0 {
        return Ok(None);
    }
    let mut out = data.get(..4).ok_or_else(invalid)?.to_vec();
    let mut previous = 0;
    for i in 0..count {
        let p = 4 + usize::from(i) * 4;
        let ppem = u16_at(data, p)?;
        let mut behavior = u16_at(data, p + 2)?;
        if (i != 0 && previous >= ppem) || (i == count - 1 && ppem != 65535) {
            return Ok(None);
        }
        previous = ppem;
        if behavior >> 8 != 0 {
            behavior &= 15;
        }
        if version == 0 && behavior >> 2 != 0 {
            version = 1;
        }
        out.extend_from_slice(&ppem.to_be_bytes());
        out.extend_from_slice(&behavior.to_be_bytes());
    }
    set16(&mut out, 0, version);
    Ok(Some(out))
}

// cpp: web_font/third_party/ots/src/cvt.cc:14-36
// cpp: web_font/third_party/ots/src/fpgm.cc:14-29
// cpp: web_font/third_party/ots/src/prep.cc:14-30
pub(crate) fn Program(data: &[u8], control_values: bool) -> io::Result<Vec<u8>> {
    if data.len() >= 128 * 1024 || (control_values && data.len() % 2 != 0) {
        return Err(invalid());
    }
    Ok(data.to_vec())
}
