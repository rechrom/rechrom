// cpp: layoutng/internal/text_item_type.h:10-30
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum TextItemType {
    kNormal = 0,
    kForcedLineBreak = 1,
    kFlowControl = 2,
    kSymbolMarker = 3,
    kLayoutGenerated = 4,
}

impl TextItemType {
    // C++ permits duplicate enum discriminants; Rust uses an associated alias.
    // cpp: layoutng/internal/text_item_type.h:29-29
    #[allow(non_upper_case_globals)]
    pub const kMaxValue: Self = Self::kLayoutGenerated;
}
