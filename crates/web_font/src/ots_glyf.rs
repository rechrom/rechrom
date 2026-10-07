// Copyright (c) 2009-2017 The OTS Authors. All rights reserved.
// Use of this source code is governed by the BSD-style OTS license.
//! Source OTS head/maxp/loca/glyf parsing and serialization.
use std::io;
fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "invalid web font")
}
struct Buffer<'a> {
    data: &'a [u8],
    position: usize,
}
impl<'a> Buffer<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, position: 0 }
    }
    fn skip(&mut self, n: usize) -> io::Result<()> {
        if n > self.data.len() - self.position {
            return Err(invalid());
        }
        self.position += n;
        Ok(())
    }
    fn u8(&mut self) -> io::Result<u8> {
        let value = *self.data.get(self.position).ok_or_else(invalid)?;
        self.position += 1;
        Ok(value)
    }
    fn u16(&mut self) -> io::Result<u16> {
        let high = self.u8()?;
        let low = self.u8()?;
        Ok(u16::from_be_bytes([high, low]))
    }
    fn i16(&mut self) -> io::Result<i16> {
        Ok(self.u16()? as i16)
    }
    fn u32(&mut self) -> io::Result<u32> {
        Ok((u32::from(self.u16()?) << 16) | u32::from(self.u16()?))
    }
}
fn get16(data: &[u8], i: usize) -> io::Result<u16> {
    let bytes = data.get(i..i + 2).ok_or_else(invalid)?;
    Ok(u16::from_be_bytes(bytes.try_into().unwrap()))
}
fn set16(data: &mut [u8], i: usize, n: u16) {
    data[i..i + 2].copy_from_slice(&n.to_be_bytes());
}

// cpp: web_font/third_party/ots/src/head.cc:14-130
pub(crate) fn Head(data: &[u8]) -> io::Result<Vec<u8>> {
    if data.len() < 54 {
        return Err(invalid());
    }
    let version = u32::from_be_bytes(data[0..4].try_into().unwrap());
    let magic = u32::from_be_bytes(data[12..16].try_into().unwrap());
    let upem = get16(data, 18)?;
    if version >> 16 != 1
        || magic != 0x5f0f3cf5
        || !(16..=16384).contains(&upem)
        || get16(data, 36)? as i16 > get16(data, 40)? as i16
        || get16(data, 38)? as i16 > get16(data, 42)? as i16
        || get16(data, 50)? > 1
        || get16(data, 52)? != 0
    {
        return Err(invalid());
    }
    let mut out = data[..54].to_vec();
    out[..4].copy_from_slice(&0x10000u32.to_be_bytes());
    out[8..12].fill(0);
    set16(&mut out, 16, get16(data, 16)? & 0x381f);
    set16(&mut out, 44, get16(data, 44)? & 0x7f);
    set16(&mut out, 48, 2);
    Ok(out)
}

// cpp: web_font/third_party/ots/src/maxp.cc:13-101
pub(crate) fn Maxp(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut read = Buffer::new(data);
    let version = read.u32()?;
    if version >> 16 > 1 {
        return Err(invalid());
    }
    if read.u16()? == 0 {
        return Err(invalid());
    }
    if version >> 16 == 1 {
        read.skip(26)?;
        let mut out = data[..32].to_vec();
        out[..4].copy_from_slice(&0x10000u32.to_be_bytes());
        let zones = match get16(data, 14)? {
            0 => 1,
            3 => 2,
            x => x,
        };
        if !matches!(zones, 1 | 2) {
            return Err(invalid());
        }
        set16(&mut out, 14, zones);
        Ok(out)
    } else {
        let mut out = data[..6].to_vec();
        out[..4].copy_from_slice(&0x5000u32.to_be_bytes());
        Ok(out)
    }
}

// cpp: web_font/third_party/ots/src/loca.cc:17-92
pub(crate) fn loca(data: &[u8], head: &[u8], maxp: &[u8]) -> io::Result<Vec<u32>> {
    let mut read = Buffer::new(data);
    let count = usize::from(get16(maxp, 4)?);
    let short = get16(head, 50)? == 0;
    let mut offsets = Vec::with_capacity(count + 1);
    let mut previous = 0;
    for _ in 0..=count {
        let offset = if short {
            u32::from(read.u16()?) * 2
        } else {
            read.u32()?
        };
        if offset < previous {
            return Err(invalid());
        }
        offsets.push(offset);
        previous = offset;
    }
    Ok(offsets)
}

fn raise(maxp: &mut [u8], field: usize, n: u16) {
    if maxp.len() == 32 && n > u16::from_be_bytes(maxp[field..field + 2].try_into().unwrap()) {
        set16(maxp, field, n);
    }
}

// cpp: web_font/third_party/ots/src/glyf.cc:19-147
fn simple(glyph: &mut Buffer<'_>, contours: i16, maxp: &mut [u8]) -> io::Result<()> {
    let mut count = 0u16;
    for i in 0..contours {
        let endpoint = glyph.u16()?;
        if endpoint == 65535 || (i != 0 && u32::from(endpoint) + 1 <= u32::from(count)) {
            return Err(invalid());
        }
        count = endpoint + 1;
    }
    raise(maxp, 6, count);
    let instructions = glyph.u16()?;
    raise(maxp, 26, instructions);
    glyph.skip(usize::from(instructions))?;
    let mut coordinates = 0usize;
    let mut index = 0u32;
    while index < u32::from(count) {
        let flag = glyph.u8()?;
        let mut delta = if flag & 2 != 0 {
            1
        } else if flag & 16 == 0 {
            2
        } else {
            0
        };
        delta += if flag & 4 != 0 {
            1
        } else if flag & 32 == 0 {
            2
        } else {
            0
        };
        if flag & 64 != 0 && index != 0 {
            return Err(invalid());
        }
        if flag & 8 != 0 {
            if index + 1 >= u32::from(count) {
                return Err(invalid());
            }
            let repeat = glyph.u8()?;
            if repeat == 0 {
                return Err(invalid());
            }
            delta *= 1 + usize::from(repeat);
            index += u32::from(repeat);
            if index >= u32::from(count) {
                return Err(invalid());
            }
        }
        if flag & 128 != 0 {
            return Err(invalid());
        }
        coordinates += delta;
        if coordinates > glyph.data.len() {
            return Err(invalid());
        }
        index += 1;
    }
    glyph.skip(coordinates)
}

// cpp: web_font/third_party/ots/src/glyf.cc:149-229,421-465
fn components(
    glyph: &mut Buffer<'_>,
    count: u16,
    level: u32,
    stack: &mut Vec<(u16, u32)>,
) -> io::Result<u16> {
    loop {
        let flags = glyph.u16()?;
        let id = glyph.u16()?;
        if id >= count {
            return Err(invalid());
        }
        glyph.skip(if flags & 1 != 0 { 4 } else { 2 })?;
        glyph.skip(if flags & 8 != 0 {
            2
        } else if flags & 64 != 0 {
            4
        } else if flags & 128 != 0 {
            8
        } else {
            0
        })?;
        stack.push((id, level));
        if flags & 32 == 0 {
            return Ok(flags);
        }
    }
}

// cpp: web_font/third_party/ots/src/glyf.cc:467-508
fn section<'a>(data: &'a [u8], offsets: &[u32], id: usize) -> io::Result<&'a [u8]> {
    let start = offsets[id] as usize;
    let end = offsets[id + 1] as usize;
    if start == end {
        return Ok(&[]);
    }
    data.get(start..end).ok_or_else(invalid)
}

// cpp: web_font/third_party/ots/src/glyf.cc:232-387,389-419,510-519
pub(crate) fn Glyf(
    data: &[u8],
    loca_data: &[u8],
    head: &mut Vec<u8>,
    maxp: &mut Vec<u8>,
) -> io::Result<(Vec<u8>, Vec<u8>)> {
    let offsets = loca(loca_data, head, maxp)?;
    let count = get16(maxp, 4)?;
    let mut out = Vec::new();
    let mut result_offsets = Vec::with_capacity(offsets.len());
    for id in 0..usize::from(count) {
        result_offsets.push(out.len() as u32);
        let mut glyph = Buffer::new(section(data, &offsets, id)?);
        if glyph.data.is_empty() {
            continue;
        }
        let contours = glyph.i16()?;
        let (mut xmin, mut ymin, mut xmax, mut ymax) =
            (glyph.i16()?, glyph.i16()?, glyph.i16()?, glyph.i16()?);
        if contours <= -2 {
            return Err(invalid());
        }
        if xmin == 32767 && xmax == -32767 && ymin == 32767 && ymax == -32767 {
            xmin = 0;
            xmax = 0;
            ymin = 0;
            ymax = 0;
        }
        if xmin > xmax || ymin > ymax {
            return Err(invalid());
        }
        if contours == 0 {
            glyph.position = 0;
        } else if contours > 0 {
            simple(&mut glyph, contours, maxp)?;
        } else {
            let mut stack = Vec::new();
            let flags = components(&mut glyph, count, 1, &mut stack)?;
            if flags & 256 != 0 {
                let instructions = glyph.u16()?;
                raise(maxp, 26, instructions);
                glyph.skip(usize::from(instructions))?;
            }
            let mut total = 0u32;
            while let Some((child, level)) = stack.pop() {
                let mut child_glyph = Buffer::new(section(data, &offsets, usize::from(child))?);
                if child_glyph.data.is_empty() {
                    continue;
                }
                let contours = child_glyph.i16()?;
                child_glyph.skip(8)?;
                if contours <= -2 {
                    return Err(invalid());
                }
                if contours == 0 {
                    continue;
                }
                if level > 65535 {
                    return Err(invalid());
                }
                raise(maxp, 30, level as u16);
                if contours > 0 {
                    let mut points = 0u16;
                    for _ in 0..contours {
                        points = child_glyph.u16()?.wrapping_add(1);
                    }
                    total = total.wrapping_add(u32::from(points));
                } else {
                    components(&mut child_glyph, count, level + 1, &mut stack)?;
                }
                if total > 65535 {
                    return Err(invalid());
                }
                raise(maxp, 10, total as u16);
            }
        }
        out.extend_from_slice(&glyph.data[..glyph.position]);
        out.resize((out.len() + 3) & !3, 0);
    }
    result_offsets.push(out.len() as u32);
    if result_offsets.iter().copied().max().unwrap_or(0) >= 65535 * 2 && get16(head, 50)? != 1 {
        set16(head, 50, 1);
    }
    let mut result_loca = Vec::new();
    let short = get16(head, 50)? == 0;
    for offset in result_offsets {
        if short {
            let half = offset >> 1;
            if half > 65535 {
                return Err(invalid());
            }
            result_loca.extend_from_slice(&(half as u16).to_be_bytes());
        } else {
            result_loca.extend_from_slice(&offset.to_be_bytes());
        }
    }
    if out.is_empty() {
        out.push(0);
    }
    Ok((out, result_loca))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn table<'a>(font: &'a [u8], tag: &[u8; 4]) -> &'a [u8] {
        for record in font[12..12 + usize::from(get16(font, 4).unwrap()) * 16].chunks_exact(16) {
            if &record[..4] == tag {
                let start = u32::from_be_bytes(record[8..12].try_into().unwrap()) as usize;
                let size = u32::from_be_bytes(record[12..16].try_into().unwrap()) as usize;
                return &font[start..start + size];
            }
        }
        panic!("table absent");
    }
    #[test]
    fn outlines_offsets_and_limits_match_independent_cpp_ots() {
        let original =
            include_bytes!("../../../../src/third_party/skia/resources/fonts/Roboto-Regular.ttf");
        let reference =
            include_bytes!("../../../artifacts/cpp-reference/page-resources/font-sanitized.ttf");
        let mut head = Head(table(original, b"head")).unwrap();
        let mut maxp = Maxp(table(original, b"maxp")).unwrap();
        let (glyf, loca) = Glyf(
            table(original, b"glyf"),
            table(original, b"loca"),
            &mut head,
            &mut maxp,
        )
        .unwrap();
        assert_eq!(glyf, table(reference, b"glyf"));
        assert_eq!(loca, table(reference, b"loca"));
        assert_eq!(maxp, table(reference, b"maxp"));
        let mut expected = table(reference, b"head").to_vec();
        expected[8..12].fill(0);
        assert_eq!(head, expected);
    }
}
