// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! markup_formatter.h/.cc: entity masks, HTML attribute and text formatting.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ResolveUrls {
    #[default]
    None,
    All,
    NonLocal,
}
pub const ENTITY_AMP: u16 = 1;
pub const ENTITY_LT: u16 = 2;
pub const ENTITY_GT: u16 = 4;
pub const ENTITY_QUOT: u16 = 8;
pub const ENTITY_NBSP: u16 = 16;
pub const ENTITY_TAB: u16 = 32;
pub const ENTITY_LINE_FEED: u16 = 64;
pub const ENTITY_CARRIAGE_RETURN: u16 = 128;
pub const PCDATA: u16 = ENTITY_AMP | ENTITY_LT | ENTITY_GT;
pub const HTML_PCDATA: u16 = ENTITY_AMP | ENTITY_LT | ENTITY_GT | ENTITY_NBSP;
pub const HTML_ATTRIBUTE: u16 = HTML_PCDATA | ENTITY_QUOT;
pub const XML_ATTRIBUTE: u16 =
    PCDATA | ENTITY_QUOT | ENTITY_TAB | ENTITY_LINE_FEED | ENTITY_CARRIAGE_RETURN;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SerializationType {
    #[default]
    Html,
    Xml,
}
pub fn Append(out: &mut Vec<u16>, value: &str) {
    out.extend(value.encode_utf16());
}
pub fn AppendCharactersReplacingEntities(out: &mut Vec<u16>, text: &[u16], mask: u16) {
    for &unit in text {
        let entity = match unit {
            38 if mask & ENTITY_AMP != 0 => Some("&amp;"),
            60 if mask & ENTITY_LT != 0 => Some("&lt;"),
            62 if mask & ENTITY_GT != 0 => Some("&gt;"),
            34 if mask & ENTITY_QUOT != 0 => Some("&quot;"),
            160 if mask & ENTITY_NBSP != 0 => Some("&nbsp;"),
            9 if mask & ENTITY_TAB != 0 => Some("&#9;"),
            10 if mask & ENTITY_LINE_FEED != 0 => Some("&#10;"),
            13 if mask & ENTITY_CARRIAGE_RETURN != 0 => Some("&#13;"),
            _ => None,
        };
        if let Some(entity) = entity {
            Append(out, entity);
        } else {
            out.push(unit);
        }
    }
}
pub fn AppendAttribute(out: &mut Vec<u16>, prefix: &[u16], name: &[u16], value: &[u16]) {
    AppendAttributeWithType(out, prefix, name, value, SerializationType::Html);
}
pub fn AppendAttributeValue(out: &mut Vec<u16>, value: &[u16], serialization: SerializationType) {
    AppendCharactersReplacingEntities(
        out,
        value,
        match serialization {
            SerializationType::Html => HTML_ATTRIBUTE,
            SerializationType::Xml => XML_ATTRIBUTE,
        },
    );
}
pub fn AppendAttributeWithType(
    out: &mut Vec<u16>,
    prefix: &[u16],
    name: &[u16],
    value: &[u16],
    serialization: SerializationType,
) {
    out.push(32);
    if !prefix.is_empty() {
        out.extend_from_slice(prefix);
        out.push(58);
    }
    out.extend_from_slice(name);
    Append(out, "=\"");
    AppendAttributeValue(out, value, serialization);
    out.push(34);
}
