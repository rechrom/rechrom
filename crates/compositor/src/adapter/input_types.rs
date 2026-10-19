//! Typed dependency boundary for the first impl-input translations.
//!
//! This is Rechrom adapter code, not counted as translated Chromium coverage.
//! It preserves opaque ElementId sentinels, bounded hit-test reason sets, and
//! scroll granularity values until their source-owning modules are connected.

/// Opaque scrolling/animation identity from cc/paint/element_id.h.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ElementId {
    id_: u64,
}

impl ElementId {
    pub const kInvalidElementId: u64 = 0;
    pub const kDeletedElementId: u64 = u64::MAX;

    pub const fn new(id: u64) -> Self {
        debug_assert!(id != Self::kInvalidElementId);
        debug_assert!(id != Self::kDeletedElementId);
        Self { id_: id }
    }

    pub const fn DeletedValue() -> Self {
        Self {
            id_: Self::kDeletedElementId,
        }
    }

    pub const fn GetInternalValue(self) -> u64 {
        self.id_
    }

    pub const fn IsValidInternalValue(value: u64) -> bool {
        value != Self::kInvalidElementId && value != Self::kDeletedElementId
    }

    pub const fn IsValid(self) -> bool {
        Self::IsValidInternalValue(self.id_)
    }

    pub fn ToString(self) -> String {
        format!("({})", self.id_)
    }
}

pub fn RemapElementIdToCcNamespace(element_id: ElementId) -> ElementId {
    assert!(element_id.GetInternalValue() & 1 == 0);
    ElementId::new(element_id.GetInternalValue() | 1)
}

// Canonical reason declarations now live in the translated source owner.
pub use crate::cc::input::main_thread_scrolling_reason::{
    MainThreadHitTestReason, MainThreadHitTestReasons,
};

/// Typed bounded-enum-set dependency. This adapter supplies the value semantics
/// required by cc's source-level base::EnumSet dependency, without public bits.
pub trait EnumSetValue: Copy {
    fn ordinal(self) -> u32;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnumSet<T> {
    bits: u64,
    domain: std::marker::PhantomData<T>,
}

impl<T> Default for EnumSet<T> {
    fn default() -> Self {
        Self {
            bits: 0,
            domain: std::marker::PhantomData,
        }
    }
}

impl<T: EnumSetValue> EnumSet<T> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn Put(&mut self, value: T) {
        self.bits |= 1u64 << value.ordinal();
    }
    pub fn PutAll(&mut self, other: &Self) {
        self.bits |= other.bits;
    }
    pub fn Remove(&mut self, value: T) {
        self.bits &= !(1u64 << value.ordinal());
    }
    pub fn Has(&self, value: T) -> bool {
        self.bits & (1u64 << value.ordinal()) != 0
    }
    pub fn HasAny(&self, other: &Self) -> bool {
        self.bits & other.bits != 0
    }
    pub fn empty(&self) -> bool {
        self.bits == 0
    }
    pub fn size(&self) -> usize {
        self.bits.count_ones() as usize
    }
    pub fn Clear(&mut self) {
        self.bits = 0;
    }
}

/// Structured tracing dependency used by MainThreadScrollingReason. Array
/// ownership and string escaping remain in the adapter, not in source logic.
pub trait TraceArraySink {
    fn BeginArray(&mut self, name: &str);
    fn AppendString(&mut self, value: &str);
    fn EndArray(&mut self);
}

#[derive(Default)]
pub struct TracedValueJSON {
    arrays: Vec<(String, Vec<String>)>,
    open: Option<usize>,
}

impl TraceArraySink for TracedValueJSON {
    fn BeginArray(&mut self, name: &str) {
        assert!(self.open.is_none());
        self.open = Some(self.arrays.len());
        self.arrays.push((name.to_owned(), Vec::new()));
    }
    fn AppendString(&mut self, value: &str) {
        let index = self.open.expect("AppendString requires an open array");
        self.arrays[index].1.push(value.to_owned());
    }
    fn EndArray(&mut self) {
        assert!(self.open.take().is_some());
    }
}

impl TracedValueJSON {
    pub fn ToJSON(&self) -> String {
        assert!(self.open.is_none());
        fn quote(value: &str) -> String {
            let mut out = String::from("\"");
            for c in value.chars() {
                match c {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    c if c < '\u{20}' => out.push_str(&format!("\\u{:04x}", c as u32)),
                    c => out.push(c),
                }
            }
            out.push('"');
            out
        }
        let fields: Vec<String> = self
            .arrays
            .iter()
            .map(|(name, values)| {
                let items = values
                    .iter()
                    .map(|s| quote(s))
                    .collect::<Vec<_>>()
                    .join(",");
                format!("{}:[{}]", quote(name), items)
            })
            .collect();
        format!("{{{}}}", fields.join(","))
    }
}

/// Values from ui/events/types/scroll_types.h; this is a host-boundary enum.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ScrollGranularity {
    kScrollByPrecisePixel = 0,
    kScrollByPixel,
    kScrollByLine,
    kScrollByPage,
    kScrollByDocument,
}

impl ScrollGranularity {
    pub const kFirstScrollGranularity: Self = Self::kScrollByPrecisePixel;
    pub const kMaxValue: Self = Self::kScrollByDocument;
}
