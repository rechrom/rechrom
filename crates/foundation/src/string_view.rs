#![allow(non_snake_case)]

use crate::String;
use std::fmt;
use std::ops::Index;
use std::sync::Arc;

// cpp: foundation/blink_base/wtf/text/string_view.h:258-260
#[unsafe(no_mangle)]
pub extern "Rust" fn StringViewIs8Bit(view: &StringView) -> bool {
    view.Is8Bit()
}

// Keep the source storage width alongside a UTF-16 code-unit projection for
// Rust consumers that have not yet been mapped to separate Span8/Span16 paths.
// Arc keeps returned views valid across fragment/font boundaries. Subviews
// share that backing storage and move a range, as Chromium StringView does.
// cpp: foundation/blink_base/wtf/text/string_view.h:113-145,219-226
#[derive(Clone, Debug)]
pub struct StringView {
    units: Arc<[u16]>,
    latin1: Option<Arc<[u8]>>,
    null: bool,
    offset: usize,
    length: usize,
}

impl PartialEq for StringView {
    fn eq(&self, other: &Self) -> bool {
        self.null == other.null && self.Span16() == other.Span16()
    }
}

impl Eq for StringView {}

impl Default for StringView {
    fn default() -> Self {
        Self {
            units: Arc::from([]),
            latin1: None,
            null: true,
            offset: 0,
            length: 0,
        }
    }
}

impl StringView {
    pub fn new(source: &str, offset: u32, length: u32) -> Self {
        let units: Vec<u16> = source.encode_utf16().collect();
        let start = offset as usize;
        let end = start
            .checked_add(length as usize)
            .expect("StringView range overflow");
        assert!(end <= units.len(), "StringView range exceeds string length");
        Self {
            units: Arc::from(&units[start..end]),
            latin1: None,
            null: false,
            offset: 0,
            length: length as usize,
        }
    }

    // cpp: foundation/blink_base/wtf/text/string_view.h:134-145,258-260
    pub fn from_blink_string_range(source: &String, offset: u32, length: u32) -> Self {
        let start = offset as usize;
        let end = start
            .checked_add(length as usize)
            .expect("StringView range overflow");
        let units = source.Span16().unwrap_or_default();
        assert!(end <= units.len(), "StringView range exceeds string length");
        Self {
            units: Arc::from(&units[start..end]),
            latin1: source.Span8().map(|bytes| Arc::from(&bytes[start..end])),
            null: source.IsNull(),
            offset: 0,
            length: length as usize,
        }
    }

    pub fn length(&self) -> u32 {
        self.length as u32
    }
    pub fn IsEmpty(&self) -> bool {
        self.length == 0
    }
    pub fn IsNull(&self) -> bool {
        self.null
    }
    pub fn Is8Bit(&self) -> bool {
        self.latin1.is_some()
    }
    pub fn Span8(&self) -> Option<&[u8]> {
        self.latin1
            .as_ref()
            .map(|bytes| &bytes[self.offset..self.offset + self.length])
    }
    // Compatibility projection for consumers still expecting UTF-16.
    pub fn Span16(&self) -> &[u16] {
        &self.units[self.offset..self.offset + self.length]
    }

    // cpp: foundation/blink_base/wtf/text/string_view.cc:347-357
    pub fn NextCodePointOffset(&self, index: u32) -> u32 {
        assert!(index < self.length());
        let next = index + 1;
        if self.Is8Bit() {
            return next;
        }
        let units = self.Span16();
        let first = units[index as usize];
        if (0xd800..=0xdbff).contains(&first)
            && next < self.length()
            && (0xdc00..=0xdfff).contains(&units[next as usize])
        {
            next + 1
        } else {
            next
        }
    }

    // cpp: foundation/blink_base/wtf/text/string_view.cc:339-345
    // cpp: foundation/blink_base/wtf/text/utf16.h:24-28
    pub fn CodePointAt(&self, offset: u32) -> crate::UChar32 {
        let index = offset as usize;
        let units = self.Span16();
        let unit = units[index];
        let pair = if (0xd800..=0xdbff).contains(&unit) {
            units
                .get(index + 1)
                .copied()
                .filter(|next| (0xdc00..=0xdfff).contains(next))
                .map(|next| (unit, next))
        } else if (0xdc00..=0xdfff).contains(&unit) && index > 0 {
            units
                .get(index - 1)
                .copied()
                .filter(|previous| (0xd800..=0xdbff).contains(previous))
                .map(|previous| (previous, unit))
        } else {
            None
        };
        pair.map_or(i32::from(unit), |(high, low)| {
            0x10000 + ((i32::from(high) - 0xd800) << 10) + i32::from(low) - 0xdc00
        })
    }

    // cpp: foundation/blink_base/wtf/text/string_view.h:476-493
    pub fn Substring(&self, offset: u32, length: u32) -> Self {
        let start = offset as usize;
        let end = start
            .checked_add(length as usize)
            .expect("StringView range overflow");
        assert!(start <= self.length && end <= self.length);
        Self {
            units: self.units.clone(),
            latin1: self.latin1.clone(),
            null: self.null,
            offset: self.offset + start,
            length: end - start,
        }
    }

    // cpp: foundation/blink_base/wtf/text/string_view.cc:272-282
    pub fn ToString(&self) -> String {
        if self.null {
            String::default()
        } else if let Some(bytes) = self.Span8() {
            String::from_latin1(bytes)
        } else {
            String::from_utf16(self.Span16())
        }
    }

    // cpp: foundation/blink_base/wtf/text/string_view.cc:414-452
    pub fn EncodeForDebugging(&self) -> String {
        if self.null {
            return String::from("<null>");
        }
        let mut encoded = std::string::String::from("\"");
        for &character in self.Span16().iter() {
            match character {
                0x09 => encoded.push_str("\\t"),
                0x0a => encoded.push_str("\\n"),
                0x0d => encoded.push_str("\\r"),
                0x22 => encoded.push_str("\\\""),
                0x5c => encoded.push_str("\\\\"),
                0x20..=0x7e => encoded.push(char::from_u32(character as u32).expect("ASCII")),
                _ => {
                    use std::fmt::Write;
                    write!(encoded, "\\u{character:04X}").expect("String writing cannot fail");
                }
            }
        }
        encoded.push('"');
        String::FromUtf8(encoded.as_bytes())
    }
}

impl From<&str> for StringView {
    fn from(value: &str) -> Self {
        let units: Arc<[u16]> = Arc::from(value.encode_utf16().collect::<Vec<_>>());
        let length = units.len();
        Self {
            units,
            latin1: None,
            null: false,
            offset: 0,
            length,
        }
    }
}

impl From<&String> for StringView {
    fn from(value: &String) -> Self {
        Self::from_blink_string_range(value, 0, value.length())
    }
}

impl fmt::Display for StringView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.EncodeForDebugging().Utf8())
    }
}

impl Index<usize> for StringView {
    type Output = u16;
    fn index(&self, index: usize) -> &Self::Output {
        &self.Span16()[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offsets_count_utf16_units() {
        let view = StringView::new("a😀b", 1, 2);
        assert_eq!(view.length(), 2);
        assert_eq!(view.Span16(), &[0xd83d, 0xde00]);
    }

    #[test]
    fn next_code_point_offset_skips_utf16_surrogate_pair() {
        let utf16 = StringView::from("a😀b");
        assert_eq!(utf16.NextCodePointOffset(0), 1);
        assert_eq!(utf16.NextCodePointOffset(1), 3);
        assert_eq!(utf16.NextCodePointOffset(3), 4);
        let latin1 = String::from_latin1(&[b'a', 0xe9, b'b']);
        let view = StringView::from(&latin1);
        assert!(view.Is8Bit());
        assert_eq!(view.NextCodePointOffset(1), 2);
    }

    #[test]
    fn debug_encoding_preserves_null_and_surrogate_units() {
        assert_eq!(StringView::default().EncodeForDebugging().Utf8(), "<null>");
        let text = String::from_utf16(&[0x41, 0x0a, 0xd83d]);
        let view = StringView::from(&text);
        assert_eq!(view.EncodeForDebugging().Utf8(), "\"A\\n\\uD83D\"");
        assert_eq!(view.ToString(), text);
    }

    #[test]
    fn nested_substrings_share_backing_storage_and_respect_their_bounds() {
        let backing = String::from_latin1(&[b'x', b'a', 0xe9, b'b', b'y']);
        let original = StringView::from(&backing);
        let middle = original.Substring(1, 3);
        let nested = middle.Substring(1, 1);
        assert!(Arc::ptr_eq(&original.units, &nested.units));
        assert!(Arc::ptr_eq(
            original.latin1.as_ref().unwrap(),
            nested.latin1.as_ref().unwrap()
        ));
        assert_eq!(nested.Span8(), Some(&[0xe9][..]));
        assert_eq!(nested.Span16(), &[0xe9]);
        assert_eq!(nested[0], 0xe9);
        assert_eq!(nested.ToString().Utf8(), "é");
        assert_eq!(nested, StringView::from("é"));
        assert_eq!(middle.Substring(3, 0).ToString().Utf8(), "");
        assert!(!middle.Substring(3, 0).IsNull());
        assert!(StringView::default().Substring(0, 0).IsNull());
        drop(original);
        drop(backing);
        assert_eq!(nested.ToString().Utf8(), "é");

        let original = StringView::from("x😀yz");
        let pair = original.Substring(1, 2);
        assert!(Arc::ptr_eq(&original.units, &pair.units));
        assert_eq!(pair.CodePointAt(0), 0x1f600);
        assert_eq!(pair.CodePointAt(1), 0x1f600);
        assert_eq!(pair.NextCodePointOffset(0), 2);
        let low_only = pair.Substring(1, 1);
        assert_eq!(low_only.CodePointAt(0), 0xde00);
        assert_eq!(low_only.EncodeForDebugging().Utf8(), "\"\\uDE00\"");
    }
}
