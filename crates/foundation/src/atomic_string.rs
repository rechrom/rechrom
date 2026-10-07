// C++: foundation/blink_base/wtf/text/atomic_string.h/.cc,
// atomic_string_table.h/.cc, and atomic_string_hash.h. This is the shared
// storage and lifetime boundary; the full String and StringImpl API remains
// unconnected.
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, LazyLock, Mutex, Weak};

use crate::rapidhash::ComputeHashAndMaskTop8Bits;

// HashTrieNode selects slots from StringImpl pointer bits and requires the
// source's 16-byte alignment for an 8-byte AtomicString handle.
#[repr(align(16))]
#[derive(Debug)]
enum Atom {
    Null,
    Deleted,
    Text {
        units: Vec<u16>,
        latin1: Option<Vec<u8>>,
        hash: u32,
    },
}

impl Drop for Atom {
    fn drop(&mut self) {
        let self_ptr = self as *const Atom;
        if let Atom::Text { units, .. } = self {
            if units.is_empty() {
                return;
            }
            let mut table = ATOM_TABLE.lock().expect("atomic string table poisoned");
            if table
                .get(units.as_slice())
                .is_some_and(|entry| std::ptr::eq(entry.as_ptr(), self_ptr))
            {
                table.remove(units.as_slice());
            }
        }
    }
}

static NULL_ATOM: LazyLock<Arc<Atom>> = LazyLock::new(|| Arc::new(Atom::Null));
static DELETED_ATOM: LazyLock<Arc<Atom>> = LazyLock::new(|| Arc::new(Atom::Deleted));
static EMPTY_ATOM: LazyLock<Arc<Atom>> = LazyLock::new(|| {
    Arc::new(Atom::Text {
        units: Vec::new(),
        latin1: Some(Vec::new()),
        hash: ComputeHashAndMaskTop8Bits(&[]),
    })
});
static ATOM_TABLE: LazyLock<Mutex<HashMap<Vec<u16>, Weak<Atom>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// cpp: foundation/blink_base/wtf/text/atomic_string.h:61-64,108-125,280-291
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct AtomicString(Arc<Atom>);

const _: () = assert!(std::mem::size_of::<AtomicString>() == 8);

impl Default for AtomicString {
    fn default() -> Self {
        Self(Arc::clone(&NULL_ATOM))
    }
}

impl AtomicString {
    // cpp: foundation/blink_base/wtf/text/atomic_string.cc:38-47
    pub fn from_utf16(units: &[u16]) -> Self {
        if units.is_empty() {
            return Self(Arc::clone(&EMPTY_ATOM));
        }
        let mut table = ATOM_TABLE.lock().expect("atomic string table poisoned");
        if let Some(atom) = table.get(units).and_then(Weak::upgrade) {
            return Self(atom);
        }
        let latin1: Option<Vec<u8>> = units
            .iter()
            .all(|unit| *unit <= 0xff)
            .then(|| units.iter().map(|unit| *unit as u8).collect());
        let hash = if let Some(bytes) = latin1.as_ref() {
            ComputeHashAndMaskTop8Bits(bytes)
        } else {
            let mut bytes = Vec::with_capacity(units.len() * 2);
            for unit in units {
                bytes.extend_from_slice(&unit.to_le_bytes());
            }
            ComputeHashAndMaskTop8Bits(&bytes)
        };
        let atom = Arc::new(Atom::Text {
            units: units.to_vec(),
            latin1,
            hash,
        });
        table.insert(units.to_vec(), Arc::downgrade(&atom));
        Self(atom)
    }

    // cpp: foundation/blink_base/wtf/text/atomic_string.h:91-96
    // cpp: foundation/blink_base/wtf/text/atomic_string.cc:78-86
    pub fn FromUtf8(bytes: &[u8]) -> Self {
        match std::str::from_utf8(bytes) {
            Ok(value) => Self::from_utf16(&value.encode_utf16().collect::<Vec<_>>()),
            Err(_) => Self::default(),
        }
    }

    pub fn from_str(value: &str) -> Self {
        Self::from_utf16(&value.encode_utf16().collect::<Vec<_>>())
    }

    // cpp: foundation/blink_base/wtf/text/atomic_string.cc:38-39
    pub fn from_latin1(bytes: &[u8]) -> Self {
        Self::from_utf16(&bytes.iter().map(|byte| *byte as u16).collect::<Vec<_>>())
    }

    // cpp: foundation/blink_base/wtf/text/atomic_string.h:121-125
    pub fn length(&self) -> usize {
        match self.0.as_ref() {
            Atom::Text { units, .. } => units.len(),
            Atom::Null => 0,
            Atom::Deleted => panic!("deleted AtomicString has no length"),
        }
    }

    pub fn empty(&self) -> bool {
        self.length() == 0
    }

    pub fn IsNull(&self) -> bool {
        matches!(self.0.as_ref(), Atom::Null)
    }

    // cpp: foundation/blink_base/wtf/text/atomic_string.h:129
    // The interning table makes this address stable and unique for each atom.
    pub fn Impl(&self) -> *const () {
        match self.0.as_ref() {
            Atom::Null => std::ptr::null(),
            Atom::Deleted => 2usize as *const (),
            Atom::Text { .. } => Arc::as_ptr(&self.0).cast(),
        }
    }

    // cpp: foundation/blink_base/wtf/text/atomic_string.h:133-135
    pub fn at(&self, index: usize) -> u16 {
        match self.0.as_ref() {
            Atom::Text { units, .. } => units.get(index).copied().unwrap_or(0),
            Atom::Null => 0,
            Atom::Deleted => panic!("deleted AtomicString cannot be indexed"),
        }
    }

    // cpp: foundation/blink_base/wtf/text/atomic_string.h:139-141
    pub fn Is8Bit(&self) -> bool {
        match self.0.as_ref() {
            Atom::Text { latin1, .. } => latin1.is_some(),
            Atom::Null | Atom::Deleted => panic!("AtomicString has no character storage"),
        }
    }

    pub fn utf16_units(&self) -> Option<&[u16]> {
        match self.0.as_ref() {
            Atom::Text { units, .. } => Some(units),
            Atom::Null => None,
            Atom::Deleted => panic!("deleted AtomicString has no character storage"),
        }
    }

    // cpp: foundation/blink_base/wtf/text/atomic_string.h:127-129
    pub fn CharactersSizeInBytes(&self) -> usize {
        match self.0.as_ref() {
            Atom::Text { units, latin1, .. } => units.len() * if latin1.is_some() { 1 } else { 2 },
            Atom::Null => 0,
            Atom::Deleted => panic!("deleted AtomicString has no characters"),
        }
    }

    // cpp: foundation/blink_base/wtf/text/atomic_string.h:146-150
    pub fn Span8(&self) -> &[u8] {
        match self.0.as_ref() {
            Atom::Text {
                latin1: Some(bytes),
                ..
            } => bytes,
            _ => panic!("AtomicString is not 8-bit"),
        }
    }

    pub fn Span16(&self) -> &[u16] {
        match self.0.as_ref() {
            Atom::Text {
                units,
                latin1: None,
                ..
            } => units,
            _ => panic!("AtomicString is not 16-bit"),
        }
    }

    // cpp: foundation/blink_base/wtf/text/atomic_string.h:153-156
    pub fn Ascii(&self) -> String {
        self.utf16_units()
            .unwrap_or_default()
            .iter()
            .map(|unit| {
                if *unit == 0 || (0x20..=0x7f).contains(unit) {
                    char::from_u32(*unit as u32).unwrap()
                } else {
                    '?'
                }
            })
            .collect()
    }

    // cpp: foundation/blink_base/wtf/text/atomic_string.h:157-160
    pub fn Latin1(&self) -> Vec<u8> {
        self.utf16_units()
            .unwrap_or_default()
            .iter()
            .map(|unit| if *unit <= 0xff { *unit as u8 } else { b'?' })
            .collect()
    }

    pub fn Utf8(&self) -> String {
        String::from_utf16_lossy(self.utf16_units().unwrap_or_default())
    }

    // cpp: foundation/blink_base/wtf/text/atomic_string.h:143
    pub fn Hash(&self) -> u32 {
        match self.0.as_ref() {
            Atom::Text { hash, .. } => *hash,
            Atom::Null | Atom::Deleted => panic!("AtomicString::Hash requires text storage"),
        }
    }

    // cpp: foundation/blink_base/wtf/text/atomic_string.h:274-275
    pub fn ContainsNoAsciiUpper(&self) -> bool {
        self.utf16_units()
            .unwrap_or_default()
            .iter()
            .all(|unit| !(*unit >= b'A' as u16 && *unit <= b'Z' as u16))
    }

    // cpp: foundation/blink_base/wtf/text/atomic_string.cc:88-104
    pub fn ToAsciiLower(&self) -> Self {
        if self.ContainsNoAsciiUpper() {
            return self.clone();
        }
        Self::from_utf16(
            &self
                .utf16_units()
                .unwrap_or_default()
                .iter()
                .map(|unit| {
                    if (*unit >= b'A' as u16) && (*unit <= b'Z' as u16) {
                        *unit + 32
                    } else {
                        *unit
                    }
                })
                .collect::<Vec<_>>(),
        )
    }

    // cpp: foundation/blink_base/wtf/text/atomic_string.cc:110-116
    pub fn ToAsciiUpper(&self) -> Self {
        let Some(units) = self.utf16_units() else {
            return self.clone();
        };
        Self::from_utf16(
            &units
                .iter()
                .map(|unit| {
                    if (*unit >= b'a' as u16) && (*unit <= b'z' as u16) {
                        *unit - 32
                    } else {
                        *unit
                    }
                })
                .collect::<Vec<_>>(),
        )
    }

    fn deleted() -> Self {
        Self(Arc::clone(&DELETED_ATOM))
    }
}

// cpp: foundation/blink_base/wtf/text/atomic_string.h:294-296
impl PartialEq for AtomicString {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for AtomicString {}

// cpp: foundation/blink_base/wtf/text/atomic_string.cc:110-112
// cpp: foundation/blink_base/wtf/text/wtf_string.cc:403-405
// cpp: foundation/blink_base/wtf/text/string_view.cc:414-453
impl fmt::Display for AtomicString {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Some(units) = self.utf16_units() else {
            return out.write_str("<null>");
        };
        out.write_str("\"")?;
        for unit in units {
            match *unit {
                0x09 => out.write_str("\\t")?,
                0x0A => out.write_str("\\n")?,
                0x0D => out.write_str("\\r")?,
                0x22 => out.write_str("\\\"")?,
                0x5C => out.write_str("\\\\")?,
                0x20..=0x7E => write!(out, "{}", char::from(*unit as u8))?,
                _ => write!(out, "\\u{unit:04X}")?,
            }
        }
        out.write_str("\"")
    }
}

// cpp: foundation/blink_base/wtf/text/atomic_string.h:305-309
// A C string compares by code units; the null atom is distinct from "".
impl PartialEq<str> for AtomicString {
    fn eq(&self, other: &str) -> bool {
        self.utf16_units()
            .is_some_and(|units| units.iter().copied().eq(other.encode_utf16()))
    }
}

impl PartialEq<&str> for AtomicString {
    fn eq(&self, other: &&str) -> bool {
        <AtomicString as PartialEq<str>>::eq(self, other)
    }
}

impl Hash for AtomicString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self.0.as_ref() {
            Atom::Text { .. } => state.write_u32(self.Hash()),
            Atom::Null => state.write_u8(0),
            Atom::Deleted => state.write_u8(1),
        }
    }
}

#[allow(non_upper_case_globals)]
pub static g_null_atom: LazyLock<AtomicString> = LazyLock::new(AtomicString::default);
#[allow(non_upper_case_globals)]
pub static g_empty_atom: LazyLock<AtomicString> =
    LazyLock::new(|| AtomicString(Arc::clone(&EMPTY_ATOM)));

// cpp: foundation/blink_base/wtf/text/atomic_string_hash.h:36-57
pub struct AtomicStringHashTraits;

impl AtomicStringHashTraits {
    pub fn GetHash(key: &AtomicString) -> u32 {
        key.Hash()
    }

    pub const kSafeToCompareToEmptyOrDeleted: bool = false;

    pub fn EmptyValue() -> &'static AtomicString {
        &g_null_atom
    }

    pub fn Peek(value: &AtomicString) -> &AtomicString {
        value
    }

    pub fn IsEmptyValue(value: &AtomicString) -> bool {
        value.IsNull()
    }

    pub fn IsDeletedValue(value: &AtomicString) -> bool {
        matches!(value.0.as_ref(), Atom::Deleted)
    }

    pub fn ConstructDeletedValue(slot: &mut AtomicString) {
        *slot = AtomicString::deleted();
    }
}
