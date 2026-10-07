// Copyright (c) 2009-2017 The OTS Authors. All rights reserved.
// Use of this source code is governed by the BSD-style OTS license.
//! Source SFNT directory, translated table dispatch and output stream.
//! Untranslated OTS table handlers report their capability; they are never
//! treated as sanitized data or substituted with fixture font bytes.
use crate::{
    be_u16, be_u32, invalid_font, ots_cmap, ots_device_tables, ots_glyf, ots_metrics,
    ots_name::OpenTypeName,
};
use std::{collections::BTreeMap, io};
type Tag = [u8; 4];
fn tag(name: &str) -> Tag {
    name.as_bytes().try_into().unwrap()
}
fn capability(name: &[u8]) -> io::Error {
    io::Error::new(
        io::ErrorKind::Unsupported,
        format!(
            "native OTS {} table sanitizer is not translated",
            String::from_utf8_lossy(name)
        ),
    )
}
fn get<'a>(tables: &'a BTreeMap<Tag, Vec<u8>>, name: &str) -> io::Result<&'a [u8]> {
    tables
        .get(&tag(name))
        .map(Vec::as_slice)
        .ok_or_else(invalid_font)
}
fn set(tables: &mut BTreeMap<Tag, Vec<u8>>, name: &str, data: Vec<u8>) {
    tables.insert(tag(name), data);
}
fn optional(tables: &mut BTreeMap<Tag, Vec<u8>>, name: &str, data: Option<Vec<u8>>) {
    if let Some(data) = data {
        set(tables, name, data);
    } else {
        tables.remove(&tag(name));
    }
}
// cpp: web_font/web_font_decoder.cc:15-38
fn passthru(name: &Tag) -> bool {
    matches!(
        name,
        b"CBDT"
            | b"CBLC"
            | b"COLR"
            | b"CPAL"
            | b"CFF2"
            | b"sbix"
            | b"STAT"
            | b"avar"
            | b"BASE"
            | b"cvar"
            | b"fvar"
            | b"gvar"
            | b"HVAR"
            | b"MVAR"
            | b"VVAR"
            | b"GDEF"
            | b"GPOS"
            | b"GSUB"
    )
}
// cpp: web_font/third_party/ots/src/ots.cc:113-173,548-569
fn known(name: &Tag) -> bool {
    matches!(
        name,
        b"maxp"
            | b"head"
            | b"OS/2"
            | b"cmap"
            | b"hhea"
            | b"hmtx"
            | b"name"
            | b"post"
            | b"loca"
            | b"glyf"
            | b"CFF "
            | b"VDMX"
            | b"hdmx"
            | b"gasp"
            | b"cvt "
            | b"fpgm"
            | b"prep"
            | b"LTSH"
            | b"VORG"
            | b"kern"
            | b"fvar"
            | b"avar"
            | b"cvar"
            | b"gvar"
            | b"HVAR"
            | b"MVAR"
            | b"STAT"
            | b"VVAR"
            | b"CFF2"
            | b"GDEF"
            | b"GPOS"
            | b"GSUB"
            | b"vhea"
            | b"vmtx"
            | b"MATH"
    )
}

// cpp: web_font/third_party/ots/src/ots.cc:176-190,198-272,605-720
pub(crate) fn Sanitize(data: &[u8]) -> io::Result<Vec<u8>> {
    if data.starts_with(b"ttcf") {
        return Err(capability(b"TTC collection"));
    }
    let mut version = be_u32(data, 0)?;
    if version == u32::from_be_bytes(*b"true") {
        version = 0x10000;
    }
    if version != 0x10000 && version != u32::from_be_bytes(*b"OTTO") {
        return Err(invalid_font());
    }
    let count = usize::from(be_u16(data, 4)?);
    if count == 0 || count >= 4096 {
        return Err(invalid_font());
    }
    let boundary = 12 + count * 16;
    let mut tables = BTreeMap::new();
    let mut edges = Vec::new();
    for i in 0..count {
        let p = 12 + i * 16;
        let name: Tag = data
            .get(p..p + 4)
            .ok_or_else(invalid_font)?
            .try_into()
            .unwrap();
        let offset = be_u32(data, p + 8)? as usize;
        let length = be_u32(data, p + 12)? as usize;
        if offset & 3 != 0
            || offset < boundary
            || offset >= data.len()
            || length == 0
            || length > 1024 * 1024 * 1024
            || offset + length > data.len()
        {
            return Err(invalid_font());
        }
        edges.push((offset, 1i32));
        edges.push((offset + length, 0i32));
        if known(&name) || passthru(&name) {
            tables.insert(name, data[offset..offset + length].to_vec());
        }
    }
    edges.sort_unstable();
    let mut overlap = 0;
    for (_, start) in edges {
        overlap += if start != 0 { 1 } else { -1 };
        if overlap > 1 {
            return Err(invalid_font());
        }
    }
    for name in [
        "maxp", "head", "OS/2", "cmap", "hhea", "hmtx", "name", "post",
    ] {
        get(&tables, name)?;
    }
    let mut maxp = ots_glyf::Maxp(get(&tables, "maxp")?)?;
    let mut head = ots_glyf::Head(get(&tables, "head")?)?;
    let mut os2 = ots_metrics::OS2(get(&tables, "OS/2")?, &mut head)?;
    let glyphs = be_u16(&maxp, 4)?;
    let cmap = ots_cmap::Cmap(get(&tables, "cmap")?, glyphs, &mut os2)?;
    let hhea = ots_metrics::MetricsHeader(get(&tables, "hhea")?, &head, glyphs, false)?;
    let hmtx = ots_metrics::Metrics(get(&tables, "hmtx")?, &hhea, glyphs)?;
    let name = OpenTypeName::Parse(get(&tables, "name")?)?.Serialize()?;
    // Source parses post before the glyph tables, but converts its version
    // for CFF only when serializing, after conflicting outlines are dropped.
    let glyf = tables.contains_key(b"glyf");
    let loca = tables.contains_key(b"loca");
    let cff = tables.contains_key(b"CFF ");
    let cff2 = tables.contains_key(b"CFF2");
    let post = ots_metrics::Post(get(&tables, "post")?, glyphs, cff && !(glyf && loca))?;
    if loca {
        ots_glyf::loca(get(&tables, "loca")?, &head, &maxp)?;
    }
    if glyf && loca {
        let (outline, offsets) = ots_glyf::Glyf(
            get(&tables, "glyf")?,
            get(&tables, "loca")?,
            &mut head,
            &mut maxp,
        )?;
        set(&mut tables, "glyf", outline);
        set(&mut tables, "loca", offsets);
    } else if glyf {
        return Err(invalid_font());
    }
    if let Some(data) = tables.get(b"hdmx") {
        let result = ots_metrics::Hdmx(data, &head, glyphs)?;
        optional(&mut tables, "hdmx", if glyf { result } else { None });
    }
    if let Some(data) = tables.get(b"gasp") {
        let result = ots_metrics::Gasp(data)?;
        optional(&mut tables, "gasp", result);
    }
    for name in ["cvt ", "fpgm", "prep"] {
        if let Some(data) = tables.get(&tag(name)) {
            let result = ots_metrics::Program(data, name == "cvt ")?;
            optional(&mut tables, name, if glyf { Some(result) } else { None });
        }
    }
    if let Some(data) = tables.get(b"vhea") {
        let result = ots_metrics::MetricsHeader(data, &head, glyphs, true)?;
        set(&mut tables, "vhea", result);
    }
    if let Some(data) = tables.get(b"vmtx") {
        let result = ots_metrics::Metrics(data, get(&tables, "vhea")?, glyphs)?;
        set(&mut tables, "vmtx", result);
    }
    for name in ["VDMX", "LTSH", "kern"] {
        if let Some(data) = tables.get(&tag(name)) {
            let result = match name {
                "VDMX" => ots_device_tables::Vdmx(data)?,
                "LTSH" => ots_device_tables::Ltsh(data, glyphs)?,
                _ => ots_device_tables::Kern(data)?,
            };
            optional(&mut tables, name, if glyf { result } else { None });
        }
    }
    for name in ["CFF ", "VORG", "MATH"] {
        if tables.contains_key(&tag(name)) {
            return Err(capability(name.as_bytes()));
        }
    }
    if glyf && loca {
        version = 0x10000;
        tables.remove(b"CFF ");
        tables.remove(b"CFF2");
    } else if cff || cff2 {
        version = u32::from_be_bytes(*b"OTTO");
        tables.remove(b"glyf");
        tables.remove(b"loca");
    } else if !(tables.contains_key(b"CBDT") && tables.contains_key(b"CBLC")) {
        return Err(invalid_font());
    }
    for (key, value) in [
        ("head", head),
        ("maxp", maxp),
        ("OS/2", os2),
        ("cmap", cmap),
        ("hhea", hhea),
        ("hmtx", hmtx),
        ("name", name),
        ("post", post),
    ] {
        set(&mut tables, key, value);
    }
    Serialize(version, &tables)
}

// cpp: web_font/third_party/ots/src/ots.cc:764-883
fn Serialize(version: u32, tables: &BTreeMap<Tag, Vec<u8>>) -> io::Result<Vec<u8>> {
    let count = tables.len();
    if count == 0 || count >= 4096 {
        return Err(invalid_font());
    }
    let log = count.ilog2() as u16;
    let search = 16u16 << log;
    let mut out = Vec::new();
    out.extend_from_slice(&version.to_be_bytes());
    out.extend_from_slice(&(count as u16).to_be_bytes());
    out.extend_from_slice(&search.to_be_bytes());
    out.extend_from_slice(&log.to_be_bytes());
    out.extend_from_slice(&((count as u16) * 16 - search).to_be_bytes());
    out.resize(12 + count * 16, 0);
    let mut head_offset = None;
    for (i, (name, data)) in tables.iter().enumerate() {
        if data.is_empty() {
            return Err(invalid_font());
        }
        let offset = out.len();
        let p = 12 + i * 16;
        out[p..p + 4].copy_from_slice(name);
        out[p + 4..p + 8].copy_from_slice(&checksum(data).to_be_bytes());
        out[p + 8..p + 12].copy_from_slice(&(offset as u32).to_be_bytes());
        out[p + 12..p + 16].copy_from_slice(&(data.len() as u32).to_be_bytes());
        out.extend_from_slice(data);
        out.resize((out.len() + 3) & !3, 0);
        if out.len() > crate::MAXIMUM_DECODED_FONT_SIZE {
            return Err(invalid_font());
        }
        if name == b"head" {
            head_offset = Some(offset);
        }
    }
    let adjustment = 0xb1b0afbau32;
    let adjustment = adjustment.wrapping_sub(checksum(&out));
    let offset = head_offset.ok_or_else(invalid_font)? + 8;
    out[offset..offset + 4].copy_from_slice(&adjustment.to_be_bytes());
    Ok(out)
}
fn checksum(data: &[u8]) -> u32 {
    data.chunks(4).fold(0u32, |total, chunk| {
        let mut word = [0; 4];
        word[..chunk.len()].copy_from_slice(chunk);
        total.wrapping_add(u32::from_be_bytes(word))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_font_bytes_match_independent_cpp_ots() {
        let raw =
            include_bytes!("../../../../src/third_party/skia/resources/fonts/Roboto-Regular.ttf");
        let reference =
            include_bytes!("../../../artifacts/cpp-reference/page-resources/font-sanitized.ttf");
        assert_eq!(Sanitize(raw).unwrap(), reference);
    }
}
