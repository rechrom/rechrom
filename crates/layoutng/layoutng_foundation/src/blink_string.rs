// C++: foundation/blink_base/wtf/text/wtf_string.h/.cc.
// This is the owning UTF-16 and null/empty value boundary. Call surfaces that
// depend on StringImpl, CString, and the rest of WTF String remain separate.
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::{Arc, OnceLock};

// WTF::String is one pointer to a shared StringImpl. Keep all Rust projections
// behind the same sized owner so copying a String remains one pointer copy and
// one reference-count update instead of copying three fat Arc slice handles.
// The projections remain an implementation detail until all consumers use the
// source's separate 8-bit/16-bit accessors directly.
#[derive(Debug)]
pub(crate) struct BlinkStringStorage {
    pub(crate) units: Box<[u16]>,
    pub(crate) latin1: Option<Box<[u8]>>,
    projection: Box<str>,
}

fn empty_utf16_storage() -> Arc<BlinkStringStorage> {
    static EMPTY: OnceLock<Arc<BlinkStringStorage>> = OnceLock::new();
    EMPTY
        .get_or_init(|| {
            Arc::new(BlinkStringStorage {
                units: Box::default(),
                latin1: None,
                projection: Box::default(),
            })
        })
        .clone()
}

fn empty_latin1_storage() -> Arc<BlinkStringStorage> {
    static EMPTY: OnceLock<Arc<BlinkStringStorage>> = OnceLock::new();
    EMPTY
        .get_or_init(|| {
            Arc::new(BlinkStringStorage {
                units: Box::default(),
                latin1: Some(Box::default()),
                projection: Box::default(),
            })
        })
        .clone()
}

// cpp: foundation/blink_base/wtf/text/wtf_string.h:54-55,211-215
#[derive(Clone, Debug)]
pub struct BlinkString {
    // A null C++ String has no StringImpl; an empty String owns a zero-length
    // StringImpl. Option<Arc<_>> is pointer-sized and preserves that distinction.
    storage: Option<Arc<BlinkStringStorage>>,
}

impl Default for BlinkString {
    fn default() -> Self {
        Self { storage: None }
    }
}

impl PartialEq for BlinkString {
    fn eq(&self, other: &Self) -> bool {
        match (&self.storage, &other.storage) {
            (None, None) => true,
            (Some(left), Some(right)) => left.units == right.units,
            _ => false,
        }
    }
}

impl Eq for BlinkString {}

impl Hash for BlinkString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.storage.is_some().hash(state);
        if let Some(storage) = &self.storage {
            storage.units.hash(state);
        }
    }
}

#[allow(non_snake_case)]
impl BlinkString {
    pub fn new() -> Self {
        Self::default()
    }

    // cpp: foundation/blink_base/wtf/text/wtf_string.h:151-158
    pub fn from_utf16(units: &[u16]) -> Self {
        let storage = if units.is_empty() {
            empty_utf16_storage()
        } else {
            Arc::new(BlinkStringStorage {
                units: units.into(),
                latin1: None,
                projection: std::string::String::from_utf16_lossy(units).into_boxed_str(),
            })
        };
        Self {
            storage: Some(storage),
        }
    }

    // cpp: foundation/blink_base/wtf/text/wtf_string.h:168-170
    // cpp: foundation/blink_base/wtf/text/wtf_string.cc:70-71
    pub fn from_latin1(bytes: &[u8]) -> Self {
        let units: Vec<u16> = bytes.iter().map(|&byte| u16::from(byte)).collect();
        let projection: std::string::String = bytes.iter().map(|&byte| char::from(byte)).collect();
        let storage = if bytes.is_empty() {
            empty_latin1_storage()
        } else {
            Arc::new(BlinkStringStorage {
                units: units.into_boxed_slice(),
                latin1: Some(bytes.into()),
                projection: projection.into_boxed_str(),
            })
        };
        Self {
            storage: Some(storage),
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
        self.storage
            .as_ref()
            .map_or(0, |storage| storage.units.len() as u32)
    }

    // cpp: foundation/blink_base/wtf/text/wtf_string.h:225-231
    // Source String::operator[] returns zero for null/out-of-range indices,
    // including indices produced by unsigned wraparound before offset zero.
    pub fn CodeUnitAt(&self, index: u32) -> u16 {
        self.storage
            .as_ref()
            .and_then(|storage| storage.units.get(index as usize))
            .copied()
            .unwrap_or(0)
    }

    pub fn empty(&self) -> bool {
        self.length() == 0
    }

    pub fn IsNull(&self) -> bool {
        self.storage.is_none()
    }

    // cpp: foundation/blink_base/wtf/text/wtf_string.h:321-321,329-334
    pub fn Is8Bit(&self) -> bool {
        self.storage
            .as_ref()
            .is_some_and(|storage| storage.latin1.is_some())
    }

    pub fn Span8(&self) -> Option<&[u8]> {
        self.storage
            .as_ref()
            .and_then(|storage| storage.latin1.as_deref())
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
        self.storage.as_ref().map(|storage| storage.units.as_ref())
    }

    // StringView retains the same StringImpl lifetime in Rust. Creating a
    // range must share storage rather than copy the whole source first.
    pub(crate) fn SharedViewStorage(&self) -> Option<Arc<BlinkStringStorage>> {
        self.storage.clone()
    }

    pub fn Utf8(&self) -> std::string::String {
        self.as_str().to_string()
    }

    /// Unicode default case folding, matching WTF String::FoldCase.
    /// Work on UTF-16 directly so embedded NULs and unpaired surrogates survive.
    pub fn FoldCase(&self) -> Self {
        if self.empty() {
            return self.clone();
        }
        let source = self.Span16().expect("nonempty string has code units");
        let source_length = i32::try_from(source.len()).expect("ICU string length overflow");
        let mut status = 0;
        // SAFETY: preflighting permits a null destination with zero capacity;
        // source is valid for the explicit number of UTF-16 code units.
        let required = unsafe {
            icu_bidi::native::u_strFoldCase(
                std::ptr::null_mut(),
                0,
                source.as_ptr(),
                source_length,
                0, // U_FOLD_CASE_DEFAULT, independent of the current locale.
                &mut status,
            )
        };
        // U_BUFFER_OVERFLOW_ERROR is the expected preflight result.
        assert!(
            status <= 0 || status == 15,
            "ICU case-fold preflight: {status}"
        );
        let mut output = vec![0; usize::try_from(required).expect("ICU case-fold length")];
        status = 0;
        // SAFETY: output has the capacity returned by preflighting the same
        // source/options. ICU writes at most that many code units.
        let written = unsafe {
            icu_bidi::native::u_strFoldCase(
                output.as_mut_ptr(),
                required,
                source.as_ptr(),
                source_length,
                0,
                &mut status,
            )
        };
        assert!(status <= 0, "ICU case folding: {status}");
        assert_eq!(written, required);
        Self::Create8BitIfPossible(&output)
    }

    pub fn as_str(&self) -> &str {
        self.storage
            .as_ref()
            .map_or("", |storage| storage.projection.as_ref())
    }

    pub fn push_str(&mut self, value: &str) {
        let mut units = self.Span16().unwrap_or_default().to_vec();
        units.extend(value.encode_utf16());
        *self = Self::from_utf16(&units);
    }

    pub fn push_string(&mut self, value: &Self) {
        let mut units = self.Span16().unwrap_or_default().to_vec();
        units.extend(value.Span16().unwrap_or_default());
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
        self.as_str()
    }
}

impl AsRef<str> for BlinkString {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for BlinkString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
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

    #[test]
    fn full_case_folding_expands_and_is_locale_independent() {
        let text = BlinkString::FromUtf8("Straße Σςσ İI 𐐀 ﬃ".as_bytes());
        assert_eq!(text.FoldCase().Utf8(), "strasse σσσ i\u{307}i 𐐨 ffi");
    }

    #[test]
    fn case_folding_preserves_null_empty_and_non_scalar_code_units() {
        assert!(BlinkString::default().FoldCase().IsNull());
        assert_eq!(BlinkString::from("").FoldCase(), BlinkString::from(""));
        let units = [0xd800, b'A' as u16, 0, 0xdc00, b'Z' as u16];
        let folded = BlinkString::from_utf16(&units).FoldCase();
        assert_eq!(folded.Span16(), Some(&[0xd800, 0x61, 0, 0xdc00, 0x7a][..]));
    }
}
