//! Typed dependency boundary for the first impl-input translations.
//!
//! This is Rechrom adapter code, not counted as translated Chromium coverage.
//! It preserves opaque ElementId sentinels, bounded hit-test reason sets, and
//! scroll granularity values until their source-owning modules are connected.

use compositor_tree::LayerTreeId;
pub use compositor_tree::{ElementId, RemapElementIdToCcNamespace};

/// Non-owning identity of a node in the current resident active ScrollTree.
/// The facade validates the tree allocation identity when resolving it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InputScrollNodeId {
    pub tree: LayerTreeId,
    pub index: i32,
}

/// Non-owning identity of a layer in the current resident active tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InputLayerId {
    pub tree: LayerTreeId,
    pub id: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerResultType {
    kUnhandled = 0,
    kScrollbarScroll,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InputHandlerPointerResult {
    pub type_: PointerResultType,
    pub scroll_units: crate::ScrollGranularity,
    pub scroll_delta: compositor_tree::input_geometry::Vector2dF,
    pub target_scroller: ElementId,
}

impl Default for InputHandlerPointerResult {
    fn default() -> Self {
        Self {
            type_: PointerResultType::kUnhandled,
            scroll_units: crate::ScrollGranularity::kScrollByPrecisePixel,
            scroll_delta: compositor_tree::input_geometry::Vector2dF::default(),
            target_scroller: ElementId::default(),
        }
    }
}

// Canonical reason declarations now live in the translated source owner.
pub use crate::main_thread_scrolling_reason::{MainThreadHitTestReason, MainThreadHitTestReasons};

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

pub use crate::ScrollGranularity;
