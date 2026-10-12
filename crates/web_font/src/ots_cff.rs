// Copyright (c) 2009-2017 The OTS Authors. All rights reserved.
// Use of this source code is governed by the BSD-style OTS license.
//! CFF1 validation at the web-font sanitation boundary.
//!
//! Chromium OTS preserves a valid CFF1 table byte-for-byte when serializing
//! it. Its parser validates the header and INDEX structure, checks the CFF and
//! `maxp` glyph counts agree, then executes every Type2 CharString. Rechrom
//! keeps those same stages. The final Type2 execution uses the repository's
//! pure-Rust `ttf-parser` interpreter instead of maintaining a second drawing
//! interpreter beside the font engine.

use crate::invalid_font;
use std::io;
use ttf_parser::{cff, CFFError, GlyphId, OutlineBuilder};

#[derive(Debug)]
struct Index {
    count: u16,
    offsets: Vec<usize>,
    next: usize,
}

fn read_offset(bytes: &[u8], cursor: &mut usize, size: usize) -> io::Result<usize> {
    if !(1..=4).contains(&size) {
        return Err(invalid_font());
    }
    let mut value = 0usize;
    for _ in 0..size {
        value = value
            .checked_shl(8)
            .and_then(|value| {
                bytes
                    .get(*cursor)
                    .copied()
                    .map(|byte| value + usize::from(byte))
            })
            .ok_or_else(invalid_font)?;
        *cursor += 1;
    }
    Ok(value)
}

// cpp: third_party/ots/src/src/cff.cc ParseIndex
fn parse_index(bytes: &[u8], start: usize) -> io::Result<Index> {
    let count_bytes = bytes.get(start..start + 2).ok_or_else(invalid_font)?;
    let count = u16::from_be_bytes([count_bytes[0], count_bytes[1]]);
    let mut cursor = start + 2;
    if count == 0 {
        return Ok(Index {
            count,
            offsets: Vec::new(),
            next: cursor,
        });
    }

    let size = usize::from(*bytes.get(cursor).ok_or_else(invalid_font)?);
    cursor += 1;
    if !(1..=4).contains(&size) {
        return Err(invalid_font());
    }
    let array_size = (usize::from(count) + 1)
        .checked_mul(size)
        .ok_or_else(invalid_font)?;
    let object_data = cursor.checked_add(array_size).ok_or_else(invalid_font)?;
    if object_data >= bytes.len() {
        return Err(invalid_font());
    }

    let mut offsets = Vec::with_capacity(usize::from(count) + 1);
    for index in 0..=count {
        let relative = read_offset(bytes, &mut cursor, size)?;
        if relative < 1 || (index == 0 && relative != 1) {
            return Err(invalid_font());
        }
        let absolute = object_data
            .checked_add(relative - 1)
            .filter(|offset| *offset <= bytes.len())
            .ok_or_else(invalid_font)?;
        if offsets.last().is_some_and(|previous| *previous > absolute) {
            return Err(invalid_font());
        }
        offsets.push(absolute);
    }
    let next = *offsets.last().ok_or_else(invalid_font)?;
    Ok(Index {
        count,
        offsets,
        next,
    })
}

// cpp: third_party/ots/src/src/cff.cc ParseNameData
fn validate_name(bytes: &[u8], index: &Index) -> io::Result<()> {
    if index.count != 1 || index.offsets.len() != 2 {
        return Err(invalid_font());
    }
    let name = bytes
        .get(index.offsets[0]..index.offsets[1])
        .ok_or_else(invalid_font)?;
    if name.len() > 127 {
        return Err(invalid_font());
    }
    for (position, character) in name.iter().copied().enumerate() {
        if position == 0 && character == 0 {
            continue;
        }
        if !(33..=126).contains(&character) || b"[](){}<>/% ".contains(&character) {
            return Err(invalid_font());
        }
    }
    Ok(())
}

#[derive(Default)]
struct ValidationOutline;

impl OutlineBuilder for ValidationOutline {
    fn move_to(&mut self, _x: f32, _y: f32) {}
    fn line_to(&mut self, _x: f32, _y: f32) {}
    fn quad_to(&mut self, _x1: f32, _y1: f32, _x: f32, _y: f32) {}
    fn curve_to(&mut self, _x1: f32, _y1: f32, _x2: f32, _y2: f32, _x: f32, _y: f32) {}
    fn close(&mut self) {}
}

// cpp: third_party/ots/src/src/cff.cc OpenTypeCFF::Parse/Serialize
pub(crate) fn Cff(data: &[u8], glyphs: u16) -> io::Result<Vec<u8>> {
    let header = data.get(..4).ok_or_else(invalid_font)?;
    if header[0] != 1 || header[1] != 0 || header[2] != 4 || !(1..=4).contains(&header[3]) {
        return Err(invalid_font());
    }

    let name = parse_index(data, usize::from(header[2]))?;
    validate_name(data, &name)?;
    let top_dict = parse_index(data, name.next)?;
    if top_dict.count != 1 {
        return Err(invalid_font());
    }
    let strings = parse_index(data, top_dict.next)?;
    if usize::from(strings.count) >= 65_000 - 390 {
        return Err(invalid_font());
    }
    // Parsing the global subroutines here preserves OTS's top-level INDEX
    // validation. The Type2 interpreter below validates subroutine use.
    let _global_subroutines = parse_index(data, strings.next)?;

    let table = cff::Table::parse(data).ok_or_else(invalid_font)?;
    if table.number_of_glyphs() != glyphs {
        return Err(invalid_font());
    }
    for glyph in 0..glyphs {
        let mut outline = ValidationOutline;
        match table.outline(GlyphId(glyph), &mut outline) {
            Ok(_) | Err(CFFError::ZeroBBox) => {}
            Err(_) => return Err(invalid_font()),
        }
    }

    // OTS CFF1 serialization writes the original validated table unchanged.
    Ok(data.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_truncated_cff_before_charstring_execution() {
        assert!(Cff(&[1, 0, 4, 4], 1).is_err());
    }
}
