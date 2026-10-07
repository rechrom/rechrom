// Copyright (c) 2009-2017 The OTS Authors. All rights reserved.
// Use of this source code is governed by the BSD-style OTS license.
//! Source OTS character maps, including BMP, UCS-4 and variation selectors.
use std::{collections::BTreeSet, io};
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
fn u24_at(d: &[u8], i: usize) -> io::Result<u32> {
    let b = d.get(i..i + 3).ok_or_else(invalid)?;
    Ok((u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]))
}
fn set16(d: &mut [u8], i: usize, x: u16) {
    d[i..i + 2].copy_from_slice(&x.to_be_bytes());
}
fn put16(d: &mut Vec<u8>, x: u16) {
    d.extend_from_slice(&x.to_be_bytes());
}
fn put32(d: &mut Vec<u8>, x: u32) {
    d.extend_from_slice(&x.to_be_bytes());
}
fn put_at(d: &mut Vec<u8>, i: usize, data: &[u8]) {
    if d.len() < i + data.len() {
        d.resize(i + data.len(), 0);
    }
    d[i..i + data.len()].copy_from_slice(data);
}

// cpp: web_font/third_party/ots/src/cmap.cc:63-266
fn format4(d: &[u8], glyphs: u16, os2: &mut [u8]) -> io::Result<Vec<u8>> {
    if u16_at(d, 4)? != 0 {
        return Err(invalid());
    }
    let twice = u16_at(d, 6)?;
    let search = u16_at(d, 8)?;
    let selector = u16_at(d, 10)?;
    let shift = u16_at(d, 12)?;
    if twice & 1 != 0 || search & 1 != 0 || twice == 0 {
        return Err(invalid());
    }
    let count = usize::from(twice / 2);
    let log = count.ilog2() as u16;
    if search != 2u16 << log || selector != log || shift != twice - search {
        return Err(invalid());
    }
    let ends = 14;
    let starts = ends + count * 2 + 2;
    let deltas = starts + count * 2;
    let offsets = deltas + count * 2;
    if u16_at(d, ends + count * 2)? != 0 {
        return Err(invalid());
    }
    let mut ranges = Vec::with_capacity(count);
    for i in 0..count {
        let start = u16_at(d, starts + i * 2)?;
        let end = u16_at(d, ends + i * 2)?;
        let delta = u16_at(d, deltas + i * 2)?;
        let mut offset = u16_at(d, offsets + i * 2)?;
        if offset & 1 != 0 {
            if i == count - 1 {
                offset = 0;
            } else {
                return Err(invalid());
            }
        }
        ranges.push((start, end, delta, offset, offsets + i * 2));
    }
    for i in 1..count {
        let (start, end, ..) = ranges[i];
        let (previous_start, previous_end, ..) = ranges[i - 1];
        if i == count - 1
            && previous_start == 65535
            && previous_end == 65535
            && start == 65535
            && end == 65535
        {
            continue;
        }
        if end <= previous_end || start <= previous_end {
            return Err(invalid());
        }
        let first = u16_at(os2, 64)?;
        let last = u16_at(os2, 66)?;
        if first != 65535 && start != 65535 && first > start {
            set16(os2, 64, start);
        }
        if last != 65535 && end != 65535 && last < end {
            set16(os2, 66, end);
        }
    }
    if ranges[count - 1].0 != 65535 || ranges[count - 1].1 != 65535 {
        return Err(invalid());
    }
    for (start, end, delta, offset, position) in ranges {
        for cp in u32::from(start)..=u32::from(end) {
            let glyph = if offset == 0 {
                (cp as u16).wrapping_add(delta)
            } else {
                u16_at(
                    d,
                    position + usize::from(offset) + (cp as usize - usize::from(start)) * 2,
                )?
            };
            if glyph >= glyphs {
                return Err(invalid());
            }
        }
    }
    Ok(d.to_vec())
}

// cpp: web_font/third_party/ots/src/cmap.cc:268-398
fn format_groups(d: &[u8], format: u16, glyphs: u16) -> io::Result<Vec<u8>> {
    if u32_at(d, 8)? != 0 {
        return Err(invalid());
    }
    let count = u32_at(d, 12)? as usize;
    if count == 0 || d.len().saturating_sub(16) / 12 < count {
        return Err(invalid());
    }
    let mut previous = None;
    let mut out = Vec::new();
    put16(&mut out, format);
    put16(&mut out, 0);
    put32(&mut out, (count * 12 + 16) as u32);
    put32(&mut out, 0);
    put32(&mut out, count as u32);
    for i in 0..count {
        let p = 16 + i * 12;
        let (start, end, glyph) = (u32_at(d, p)?, u32_at(d, p + 4)?, u32_at(d, p + 8)?);
        if start > 0x10ffff || end > 0x10ffff || glyph > 65535 {
            return Err(invalid());
        }
        if format == 12 {
            if end < start || end - start + glyph > u32::from(glyphs) {
                return Err(invalid());
            }
        } else if glyph >= u32::from(glyphs) {
            return Err(invalid());
        }
        if previous.is_some_and(|(s, e)| start <= s || start <= e) {
            return Err(invalid());
        }
        previous = Some((start, end));
        put32(&mut out, start);
        put32(&mut out, end);
        put32(&mut out, glyph);
    }
    Ok(out)
}

// cpp: web_font/third_party/ots/src/cmap.cc:400-538,874-926
fn format14(d: &[u8]) -> io::Result<Vec<u8>> {
    let count = u32_at(d, 6)? as usize;
    if count == 0 || count > 259 {
        return Err(invalid());
    }
    let mut records = Vec::with_capacity(count);
    let mut previous = 0;
    for i in 0..count {
        let p = 10 + i * 11;
        let selector = u24_at(d, p)?;
        let default = u32_at(d, p + 3)? as usize;
        let nondefault = u32_at(d, p + 7)? as usize;
        if !((0x180b..=0x180d).contains(&selector)
            || (0xfe00..=0xfe0f).contains(&selector)
            || (0xe0100..=0xe01ef).contains(&selector))
            || (i != 0 && previous >= selector)
            || (default == 0 && nondefault == 0)
            || (default != 0 && default >= d.len())
            || (nondefault != 0 && nondefault >= d.len())
        {
            return Err(invalid());
        }
        previous = selector;
        records.push((p, default, nondefault));
    }
    let mut position = 10 + count * 11;
    let mut ranges = Vec::new();
    for (_, default, nondefault) in &records {
        if *default != 0 {
            let number = u32_at(d, *default)? as usize;
            position = *default + 4;
            if number == 0 || d.len().saturating_sub(position) / 4 < number {
                return Err(invalid());
            }
            let mut last = 0;
            for _ in 0..number {
                let unicode = u24_at(d, position)?;
                let extra = *d.get(position + 3).ok_or_else(invalid)?;
                let check = unicode + u32::from(extra);
                if unicode == 0
                    || unicode > 0x10ffff
                    || check > 0xffffff
                    || (last != 0 && unicode <= last)
                {
                    return Err(invalid());
                }
                last = check;
                position += 4;
            }
            ranges.push((*default, position));
        }
        if *nondefault != 0 {
            let number = u32_at(d, *nondefault)? as usize;
            position = *nondefault + 4;
            if number == 0 || d.len().saturating_sub(position) / 5 < number {
                return Err(invalid());
            }
            let mut last = 0;
            for _ in 0..number {
                let unicode = u24_at(d, position)?;
                let glyph = u16_at(d, position + 3)?;
                if glyph == 0
                    || unicode == 0
                    || unicode > 0x10ffff
                    || (last != 0 && unicode <= last)
                {
                    return Err(invalid());
                }
                last = unicode;
                position += 5;
            }
            ranges.push((*nondefault, position));
        }
    }
    if position != d.len() {
        return Err(invalid());
    }
    let mut out = Vec::new();
    put16(&mut out, 14);
    put32(&mut out, d.len() as u32);
    put32(&mut out, count as u32);
    for (p, _, _) in records {
        out.extend_from_slice(&d[p..p + 11]);
    }
    for (start, end) in ranges {
        put_at(&mut out, start, &d[start..end]);
    }
    Ok(out)
}

// cpp: web_font/third_party/ots/src/cmap.cc:540-833,835-1074
pub(crate) fn Cmap(d: &[u8], glyphs: u16, os2: &mut [u8]) -> io::Result<Vec<u8>> {
    let count = usize::from(u16_at(d, 2)?);
    if u16_at(d, 0)? != 0 || count == 0 {
        return Err(invalid());
    }
    let data_offset = 4 + count * 8;
    let mut headers = Vec::new();
    for i in 0..count {
        let p = 4 + i * 8;
        let platform = u16_at(d, p)?;
        let encoding = u16_at(d, p + 2)?;
        let offset = u32_at(d, p + 4)? as usize;
        if offset > 1024 * 1024 * 1024 || offset < data_offset || offset >= d.len() {
            return Err(invalid());
        }
        let format = u16_at(d, offset)?;
        let length = match format {
            0 | 4 => {
                u16_at(d, offset + 4)?;
                usize::from(u16_at(d, offset + 2)?)
            }
            12 | 13 => {
                u32_at(d, offset + 8)?;
                u32_at(d, offset + 4)? as usize
            }
            14 => u32_at(d, offset + 2)? as usize,
            _ => 0,
        };
        if length != 0 && (length > 1024 * 1024 * 1024 || offset + length > d.len()) {
            return Err(invalid());
        }
        headers.push((platform, encoding, offset, format, length));
    }
    let mut unique = BTreeSet::new();
    let mut edges = Vec::new();
    for &(_, _, offset, _, length) in &headers {
        if unique.insert((offset, offset + length)) {
            edges.push((offset, 1i32));
            edges.push((offset + length, 0i32));
        }
    }
    edges.sort_unstable();
    let mut overlap = 0;
    for (_, start) in edges {
        overlap += if start != 0 { 1 } else { -1 };
        if overlap > 1 {
            return Err(invalid());
        }
    }
    let mut bmp_unicode = None;
    let mut selectors = None;
    let mut mac = Vec::new();
    let mut symbol = None;
    let mut bmp_windows = None;
    let mut ucs4 = None;
    let mut fallback = None;
    for (platform, encoding, offset, format, length) in headers {
        let input = &d[offset..offset + length];
        match (platform, encoding, format) {
            (0, 0 | 1, 4) => (bmp_windows = Some(format4(input, glyphs, os2)?)),
            (0, 3, 4) => (bmp_unicode = Some(format4(input, glyphs, os2)?)),
            (0, 3 | 4, 12) => (ucs4 = Some(format_groups(input, 12, glyphs)?)),
            (0, 5, 14) => (selectors = Some(format14(input)?)),
            (1, 0, 0) => {
                u16_at(input, 4)?;
                mac.extend_from_slice(input.get(6..262).ok_or_else(invalid)?);
            }
            (3, 0, 4) => (symbol = Some(format4(input, glyphs, os2)?)),
            (3, 1, 4) => (bmp_windows = Some(format4(input, glyphs, os2)?)),
            (3, 10, 12) => (ucs4 = Some(format_groups(input, 12, glyphs)?)),
            (3, 10, 13) => (fallback = Some(format_groups(input, 13, glyphs)?)),
            _ => {}
        }
    }
    if symbol.is_some() {
        bmp_windows = None;
    }
    if symbol.is_none()
        && bmp_windows.is_none()
        && bmp_unicode.is_none()
        && ucs4.is_none()
        && fallback.is_none()
    {
        return Err(invalid());
    }
    let mut tables = Vec::new();
    for (key, data) in [
        ([0, 3], bmp_unicode),
        ([0, 5], selectors),
        (
            [1, 0],
            if mac.is_empty() {
                None
            } else {
                let mut bytes = Vec::new();
                put16(&mut bytes, 0);
                put16(&mut bytes, 262);
                put16(&mut bytes, 0);
                bytes.extend_from_slice(&mac[..256]);
                Some(bytes)
            },
        ),
        ([3, 0], symbol),
        ([3, 1], bmp_windows),
        ([3, 10], ucs4),
        ([3, 10], fallback),
    ] {
        if let Some(data) = data {
            tables.push((key, data));
        }
    }
    let mut out = Vec::new();
    put16(&mut out, 0);
    put16(&mut out, tables.len() as u16);
    out.resize(4 + tables.len() * 8, 0);
    for (i, (key, data)) in tables.into_iter().enumerate() {
        let p = 4 + i * 8;
        set16(&mut out, p, key[0]);
        set16(&mut out, p + 2, key[1]);
        let offset = out.len() as u32;
        out[p + 4..p + 8].copy_from_slice(&offset.to_be_bytes());
        out.extend_from_slice(&data);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn table<'a>(d: &'a [u8], tag: &[u8; 4]) -> &'a [u8] {
        for record in d[12..12 + usize::from(u16_at(d, 4).unwrap()) * 16].chunks_exact(16) {
            if &record[..4] == tag {
                let offset = u32_at(record, 8).unwrap() as usize;
                let length = u32_at(record, 12).unwrap() as usize;
                return &d[offset..offset + length];
            }
        }
        panic!("table absent");
    }
    #[test]
    fn character_map_and_os2_ranges_match_independent_cpp_ots() {
        let original =
            include_bytes!("../../../../src/third_party/skia/resources/fonts/Roboto-Regular.ttf");
        let reference =
            include_bytes!("../../../artifacts/cpp-reference/page-resources/font-sanitized.ttf");
        let mut os2 = table(original, b"OS/2").to_vec();
        let cmap = Cmap(
            table(original, b"cmap"),
            u16_at(table(original, b"maxp"), 4).unwrap(),
            &mut os2,
        )
        .unwrap();
        assert_eq!(cmap, table(reference, b"cmap"));
        assert_eq!(os2, table(reference, b"OS/2"));
    }
}
