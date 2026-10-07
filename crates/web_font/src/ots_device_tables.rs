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
fn read16(d: &[u8], position: &mut usize) -> io::Result<u16> {
    let result = u16_at(d, *position)?;
    *position += 2;
    Ok(result)
}
fn put16(d: &mut Vec<u8>, n: u16) {
    d.extend_from_slice(&n.to_be_bytes());
}

// cpp: web_font/third_party/ots/src/kern.cc:14-175
pub(crate) fn Kern(data: &[u8]) -> io::Result<Option<Vec<u8>>> {
    let mut position = 0;
    let version = read16(data, &mut position)?;
    let count = read16(data, &mut position)?;
    if version > 0 || count == 0 {
        return Ok(None);
    }
    let mut tables = Vec::new();
    for _ in 0..count {
        let version = read16(data, &mut position)?;
        let length = read16(data, &mut position)?;
        if version > 0 {
            continue;
        }
        if position - 4 + usize::from(length) > data.len() {
            return Err(invalid());
        }
        let coverage = read16(data, &mut position)?;
        if coverage & 1 == 0 {
            continue;
        }
        if coverage & 0xf0 != 0 {
            return Ok(None);
        }
        if coverage >> 8 != 0 {
            continue;
        }
        let count = read16(data, &mut position)?;
        let _search = read16(data, &mut position)?;
        let selector = read16(data, &mut position)?;
        let _shift = read16(data, &mut position)?;
        if count == 0 || usize::from(count) > 65536 / 6 {
            return Ok(None);
        }
        let log = count.ilog2() as u16;
        let search = 6u16 << log;
        if selector != log {
            return Err(invalid());
        }
        let size = 14 + usize::from(count) * 6;
        if size > 65535 {
            return Err(invalid());
        }
        let mut out = Vec::new();
        for n in [
            version,
            size as u16,
            coverage,
            count,
            search,
            log,
            count * 6 - search,
        ] {
            put16(&mut out, n);
        }
        let mut previous = 0;
        for i in 0..count {
            let left = read16(data, &mut position)?;
            let right = read16(data, &mut position)?;
            let value = read16(data, &mut position)?;
            let key = (u32::from(left) << 16) | u32::from(right);
            if i != 0 && key <= previous {
                return Ok(None);
            }
            previous = key;
            for n in [left, right, value] {
                put16(&mut out, n);
            }
        }
        tables.push(out);
    }
    if tables.is_empty() {
        return Ok(None);
    }
    let mut out = Vec::new();
    put16(&mut out, version);
    put16(&mut out, tables.len() as u16);
    for table in tables {
        out.extend_from_slice(&table);
    }
    Ok(Some(out))
}

// cpp: web_font/third_party/ots/src/ltsh.cc:15-69
pub(crate) fn Ltsh(data: &[u8], glyphs: u16) -> io::Result<Option<Vec<u8>>> {
    let version = u16_at(data, 0)?;
    let count = u16_at(data, 2)?;
    if version != 0 || count != glyphs {
        return Ok(None);
    }
    Ok(Some(
        data.get(..4 + usize::from(count))
            .ok_or_else(invalid)?
            .to_vec(),
    ))
}

// cpp: web_font/third_party/ots/src/vdmx.cc:14-153
pub(crate) fn Vdmx(data: &[u8]) -> io::Result<Option<Vec<u8>>> {
    let version = u16_at(data, 0)?;
    let records = u16_at(data, 2)?;
    let ratios = u16_at(data, 4)?;
    if version > 1 {
        return Ok(None);
    }
    let mut position = 6;
    for i in 0..ratios {
        let r = data.get(position..position + 4).ok_or_else(invalid)?;
        if r[0] > 1 || r[2] > r[3] || (i < ratios - 1 && r[1] == 0 && r[2] == 0 && r[3] == 0) {
            return Ok(None);
        }
        position += 4;
    }
    let base = position;
    for _ in 0..ratios {
        let offset = read16(data, &mut position)? as usize;
        if base + offset >= data.len() {
            return Err(invalid());
        }
    }
    for _ in 0..records {
        let count = read16(data, &mut position)?;
        data.get(position..position + 2).ok_or_else(invalid)?;
        position += 2;
        let mut previous = 0;
        for i in 0..count {
            let pel = read16(data, &mut position)?;
            let max = read16(data, &mut position)? as i16;
            let min = read16(data, &mut position)? as i16;
            if max < min || (i != 0 && previous >= pel) {
                return Ok(None);
            }
            previous = pel;
        }
    }
    Ok(Some(data[..position].to_vec()))
}
