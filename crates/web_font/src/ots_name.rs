// Copyright (c) 2011-2017 The OTS Authors. All rights reserved.
// Use of this source code is governed by the BSD-style OTS license.
//! Native translation of the OTS name table, including rejection, synthesis,
//! PostScript normalization and serialization. Other OTS tables remain owned
//! by the unfinished sanitizer; this module does not bypass their checks.
use std::{collections::HashSet, io};

// cpp: web_font/third_party/ots/src/name.h:19-52
#[derive(Clone)]
struct NameRecord {
    key: [u16; 4],
    text: Vec<u8>,
}
pub(crate) struct OpenTypeName {
    names: Vec<NameRecord>,
    lang_tags: Vec<Vec<u8>>,
    name_ids: HashSet<u16>,
}
fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "invalid web font")
}
fn u16_at(bytes: &[u8], offset: usize) -> io::Result<u16> {
    let data = bytes.get(offset..offset + 2).ok_or_else(invalid)?;
    Ok(u16::from_be_bytes([data[0], data[1]]))
}
fn read_u16(bytes: &[u8], offset: &mut usize) -> io::Result<u16> {
    let value = u16_at(bytes, *offset)?;
    *offset += 2;
    Ok(value)
}
fn write_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

// cpp: web_font/third_party/ots/src/name.cc:16-20
fn allowed_ps(byte: u8) -> bool {
    // strchr accepts the string terminator too, as in the original helper.
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~' | 0)
}
// cpp: web_font/third_party/ots/src/name.cc:22-34
fn sanitize_ascii(text: &mut [u8]) -> bool {
    if text.len() > 63 {
        return false;
    }
    for byte in text {
        if !allowed_ps(*byte) {
            *byte = b'_';
        }
    }
    true
}
// cpp: web_font/third_party/ots/src/name.cc:36-53
fn sanitize_utf16(text: &mut [u8]) -> bool {
    if text.len() % 2 != 0 || text.len() > 126 {
        return false;
    }
    for pair in text.chunks_exact_mut(2) {
        if pair[0] != 0 {
            return false;
        }
        // Preserve source's assignment to the first byte of the pair.
        if !allowed_ps(pair[1]) {
            pair[0] = b'_';
        }
    }
    true
}
// cpp: web_font/third_party/ots/src/name.cc:55-63
fn ascii_to_utf16(text: &[u8]) -> Vec<u8> {
    text.iter().flat_map(|b| [0, *b]).collect()
}

impl OpenTypeName {
    // cpp: web_font/third_party/ots/src/name.cc:71-251
    pub(crate) fn Parse(data: &[u8]) -> io::Result<Self> {
        let mut position = 0;
        let format = read_u16(data, &mut position)?;
        if format > 1 {
            return Err(invalid());
        }
        let count = read_u16(data, &mut position)?;
        let strings = read_u16(data, &mut position)? as usize;
        if strings > data.len() {
            return Err(invalid());
        }
        let mut out = Self {
            names: Vec::new(),
            lang_tags: Vec::new(),
            name_ids: HashSet::new(),
        };
        let mut sort_required = false;
        for _ in 0..count {
            let mut key = [0; 4];
            for field in &mut key {
                *field = read_u16(data, &mut position)?;
            }
            let length = read_u16(data, &mut position)? as usize;
            let offset = read_u16(data, &mut position)? as usize;
            let known = match key[0] {
                0 => key[1] <= 6,
                1 => key[1] <= 32,
                2 => key[1] <= 2,
                3 => key[1] <= 6 || key[1] == 10,
                4 => key[1] <= 255,
                _ => false,
            };
            if !known || strings + offset + length > data.len() {
                continue;
            }
            let mut text = data[strings + offset..strings + offset + length].to_vec();
            if key[3] == 6 {
                let valid = match key[0] {
                    1 => sanitize_ascii(&mut text),
                    0 | 3 => sanitize_utf16(&mut text),
                    _ => true,
                };
                if !valid {
                    continue;
                }
            }
            if out.names.last().is_some_and(|last| last.key >= key) {
                sort_required = true;
            }
            out.name_ids.insert(key[3]);
            out.names.push(NameRecord { key, text });
        }
        if format == 1 {
            let tags = read_u16(data, &mut position)?;
            for _ in 0..tags {
                let length = read_u16(data, &mut position)? as usize;
                let offset = read_u16(data, &mut position)? as usize;
                if strings + offset + length > data.len() || length > 200 {
                    return Err(invalid());
                }
                out.lang_tags
                    .push(data[strings + offset..strings + offset + length].to_vec());
            }
        }
        if position > strings {
            return Err(invalid());
        }
        let standards: [Option<&[u8]>; 7] = [
            None,
            Some(b"OTS derived font"),
            Some(b"Unspecified"),
            None,
            Some(b"OTS derived font"),
            Some(b"1.000"),
            Some(b"OTS-derived-font"),
        ];
        let mut mac = [false; 7];
        let mut win = [false; 7];
        for record in &out.names {
            let id = record.key[3] as usize;
            if id >= 7 || standards[id].is_none() {
                continue;
            }
            if record.key[0] == 1 {
                mac[id] = true;
            }
            if record.key[0] == 3 {
                win[id] = true;
            }
        }
        for (id, text) in standards.into_iter().enumerate() {
            if let Some(text) = text {
                if !mac[id] && !win[id] {
                    out.names.push(NameRecord {
                        key: [1, 0, 0, id as u16],
                        text: text.to_vec(),
                    });
                    out.names.push(NameRecord {
                        key: [3, 1, 1033, id as u16],
                        text: ascii_to_utf16(text),
                    });
                    sort_required = true;
                }
            }
        }
        if sort_required {
            out.names.sort_unstable_by_key(|n| n.key);
        }
        Ok(out)
    }

    // cpp: web_font/third_party/ots/src/name.cc:254-322
    pub(crate) fn Serialize(&self) -> io::Result<Vec<u8>> {
        let count = self.names.len() as u16;
        let tags = self.lang_tags.len() as u16;
        let format = u16::from(!self.lang_tags.is_empty());
        let offset = 6
            + usize::from(count) * 12
            + if format != 0 {
                2 + usize::from(tags) * 4
            } else {
                0
            };
        if offset > 65535 {
            return Err(invalid());
        }
        let mut out = Vec::new();
        let mut strings = Vec::new();
        write_u16(&mut out, format);
        write_u16(&mut out, count);
        write_u16(&mut out, offset as u16);
        for record in &self.names {
            if strings.len() + record.text.len() > 65535 {
                return Err(invalid());
            }
            for field in record.key {
                write_u16(&mut out, field);
            }
            write_u16(&mut out, record.text.len() as u16);
            write_u16(&mut out, strings.len() as u16);
            strings.extend_from_slice(&record.text);
        }
        if format == 1 {
            write_u16(&mut out, tags);
            for tag in &self.lang_tags {
                if strings.len() + tag.len() > 65535 {
                    return Err(invalid());
                }
                write_u16(&mut out, tag.len() as u16);
                write_u16(&mut out, strings.len() as u16);
                strings.extend_from_slice(tag);
            }
        }
        out.extend_from_slice(&strings);
        Ok(out)
    }

    // cpp: web_font/third_party/ots/src/name.cc:324-367
    pub(crate) fn IsValidNameId(&mut self, id: u16, add: bool) -> bool {
        if add && !self.name_ids.contains(&id) {
            let mut platforms = HashSet::new();
            let count = self.names.len();
            for i in 0..count {
                let platform = self.names[i].key[0];
                if !matches!(platform, 0 | 1 | 3) || !platforms.insert(platform) {
                    continue;
                }
                self.names.push(NameRecord {
                    key: match platform {
                        0 => [0, 0, 0, id],
                        1 => [1, 0, 0, id],
                        _ => [3, 1, 1033, id],
                    },
                    text: b"NoName".to_vec(),
                });
            }
            if !platforms.is_empty() {
                self.names.sort_unstable_by_key(|n| n.key);
                self.name_ids.insert(id);
            }
        }
        self.name_ids.contains(&id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn table<'a>(font: &'a [u8], tag: &[u8; 4]) -> &'a [u8] {
        let count = u16_at(font, 4).unwrap() as usize;
        for record in font[12..12 + count * 16].chunks_exact(16) {
            if &record[..4] == tag {
                let offset = u32::from_be_bytes(record[8..12].try_into().unwrap()) as usize;
                let length = u32::from_be_bytes(record[12..16].try_into().unwrap()) as usize;
                return &font[offset..offset + length];
            }
        }
        panic!("table absent");
    }
    #[test]
    fn name_table_bytes_match_independent_cpp_ots() {
        let original =
            include_bytes!("../../../../src/third_party/skia/resources/fonts/Roboto-Regular.ttf");
        let sanitized =
            include_bytes!("../../../artifacts/cpp-reference/page-resources/font-sanitized.ttf");
        let name = OpenTypeName::Parse(table(original, b"name")).unwrap();
        assert_eq!(name.Serialize().unwrap(), table(sanitized, b"name"));
    }
}
