// C++: foundation/blink_base/wtf/text/wtf_string.h/.cc.
// This is the owning UTF-16 and null/empty value boundary. Call surfaces that
// depend on StringImpl, CString, and the rest of WTF String remain separate.
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::Arc;

// cpp: foundation/blink_base/wtf/text/wtf_string.h:54-55,211-215
#[derive(Clone, Debug)]
pub struct BlinkString {
    // A null C++ String has no StringImpl; an empty String owns a zero-length
    // StringImpl. Option preserves that distinction while Arc preserves cheap
    // value copies and stable code-unit storage.
    units: Option<Arc<[u16]>>,
    // StringImpl can store Latin-1 directly. Keep the UTF-16 code-unit
    // projection above for Rust callers that have not yet been width-mapped.
    latin1: Option<Arc<[u8]>>,
    // Rust host APIs consume UTF-8. This projection is never used for source
    // equality, hashing, length, or offsets; unpaired surrogates remain in
    // `units` even when the projection needs a replacement character.
    projection: Arc<str>,
}

impl Default for BlinkString {
    fn default() -> Self {
        Self {
            units: None,
            latin1: None,
            projection: Arc::from(""),
        }
    }
}

impl PartialEq for BlinkString {
    fn eq(&self, other: &Self) -> bool {
        self.units == other.units
    }
}

impl Eq for BlinkString {}

impl Hash for BlinkString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.units.hash(state);
    }
}

#[allow(non_snake_case)]
impl BlinkString {
    pub fn new() -> Self {
        Self::default()
    }

    // cpp: foundation/blink_base/wtf/text/wtf_string.h:151-158
    pub fn from_utf16(units: &[u16]) -> Self {
        Self {
            units: Some(Arc::from(units)),
            latin1: None,
            projection: Arc::from(std::string::String::from_utf16_lossy(units)),
        }
    }

    // cpp: foundation/blink_base/wtf/text/wtf_string.h:168-170
    // cpp: foundation/blink_base/wtf/text/wtf_string.cc:70-71
    pub fn from_latin1(bytes: &[u8]) -> Self {
        let units: Vec<u16> = bytes.iter().map(|&byte| u16::from(byte)).collect();
        let projection: std::string::String = bytes.iter().map(|&byte| char::from(byte)).collect();
        Self {
            units: Some(Arc::from(units)),
            latin1: Some(Arc::from(bytes)),
            projection: Arc::from(projection),
        }
    }

    // cpp: foundation/blink_base/wtf/text/wtf_string.cc:321-333
    pub fn Make8BitFrom16BitSource(source: &[u16]) -> Self {
        Self::from_latin1(&source.iter().map(|&unit| unit as u8).collect::<Vec<_>>())
    }

    // cpp: foundation/blink_base/wtf/text/string_impl.cc:350-370
    pub fn Create8BitIfPossible(source: &[u16]) -> Self {
        if source.iter().all(|&unit| unit <= 0xff) {
            Self::Make8BitFrom16BitSource(source)
        } else {
            Self::from_utf16(source)
        }
    }

    // cpp: foundation/blink_base/wtf/text/wtf_string.h:208-215
    pub fn length(&self) -> u32 {
        self.units.as_ref().map_or(0, |units| units.len() as u32)
    }

    // cpp: foundation/blink_base/wtf/text/wtf_string.h:225-231
    // Source String::operator[] returns zero for null/out-of-range indices,
    // including indices produced by unsigned wraparound before offset zero.
    pub fn CodeUnitAt(&self, index: u32) -> u16 {
        self.units
            .as_ref()
            .and_then(|units| units.get(index as usize))
            .copied()
            .unwrap_or(0)
    }

    pub fn empty(&self) -> bool {
        self.length() == 0
    }

    pub fn IsNull(&self) -> bool {
        self.units.is_none()
    }

    // cpp: foundation/blink_base/wtf/text/wtf_string.h:321-321,329-334
    pub fn Is8Bit(&self) -> bool {
        self.latin1.is_some()
    }

    pub fn Span8(&self) -> Option<&[u8]> {
        self.latin1.as_deref()
    }

    // cpp: foundation/blink_base/wtf/text/wtf_string.h:84-90
    // cpp: foundation/blink_base/wtf/text/wtf_string.cc:348-371
    pub fn FromUtf8(bytes: &[u8]) -> Self {
        match std::str::from_utf8(bytes) {
            Ok(text) if text.is_ascii() => Self::from_latin1(bytes),
            Ok(text) => Self::from_utf16(&text.encode_utf16().collect::<Vec<_>>()),
            Err(_) => Self::default(),
        }
    }

    // cpp: foundation/blink_base/wtf/text/wtf_string.h:338-342
    // This also exposes the code-unit projection of an 8-bit string until
    // every Rust consumer can select Span8/Span16 like the C++ source.
    pub fn Span16(&self) -> Option<&[u16]> {
        self.units.as_deref()
    }

    pub fn Utf8(&self) -> std::string::String {
        self.projection.to_string()
    }

    pub fn as_str(&self) -> &str {
        &self.projection
    }

    pub fn push_str(&mut self, value: &str) {
        let mut units = self.units.as_deref().unwrap_or_default().to_vec();
        units.extend(value.encode_utf16());
        *self = Self::from_utf16(&units);
    }

    pub fn push_string(&mut self, value: &Self) {
        let mut units = self.units.as_deref().unwrap_or_default().to_vec();
        units.extend(value.units.as_deref().unwrap_or_default());
        *self = Self::from_utf16(&units);
    }

    pub fn Number(value: impl fmt::Display) -> Self {
        Self::FromUtf8(value.to_string().as_bytes())
    }

    // cpp: foundation/blink_base/wtf/text/wtf_string.cc:380-382
    pub fn EncodeForDebugging(&self) -> Self {
        crate::StringView::from(self).EncodeForDebugging()
    }
}

// cpp: foundation/blink_base/wtf/text/wtf_string.h:166-180
// Inline shaping selects the same storage-width branch as StringImpl.
// cpp: foundation/blink_base/wtf/text/wtf_string.h:321-321
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "Rust" fn StringIs8BitForInline(text: &BlinkString) -> bool {
    text.Is8Bit()
}

impl From<&str> for BlinkString {
    fn from(value: &str) -> Self {
        // The C++ char* constructor interprets bytes as Latin-1.
        Self::from_latin1(value.as_bytes())
    }
}

impl Deref for BlinkString {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.projection
    }
}

impl AsRef<str> for BlinkString {
    fn as_ref(&self) -> &str {
        &self.projection
    }
}

impl fmt::Display for BlinkString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.projection)
    }
}

impl fmt::Write for BlinkString {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_str(s);
        Ok(())
    }
}

impl From<std::string::String> for BlinkString {
    fn from(value: std::string::String) -> Self {
        Self::from(value.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_empty_and_utf16_lengths_remain_distinct() {
        let null = BlinkString::default();
        let empty = BlinkString::from("");
        let text = BlinkString::FromUtf8("a😀b".as_bytes());
        assert!(null.IsNull() && null.empty());
        assert!(!empty.IsNull() && empty.empty());
        assert_ne!(null, empty);
        assert_eq!(text.length(), 4);
        assert_eq!(text.Span16(), Some(&[0x61, 0xd83d, 0xde00, 0x62][..]));
    }

    #[test]
    fn invalid_utf8_is_null_but_empty_utf8_is_not() {
        assert!(BlinkString::FromUtf8(&[0xff]).IsNull());
        assert!(!BlinkString::FromUtf8(b"").IsNull());
    }
}
