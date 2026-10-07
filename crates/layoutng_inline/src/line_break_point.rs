// C++: layoutng_inline/line_break_point.h.
// The source's InlineItemTextIndex is owned by //src/layoutng. Its Cargo
// connection is pending the mutual LayoutNG/inline assembly, not copied here.
use foundation::LayoutUnit;
use layoutng::internal::inline_item_text_index::InlineItemTextIndex;

// cpp: layoutng_inline/line_break_point.h:20-56
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct LineBreakPoint {
    pub offset: InlineItemTextIndex,
    pub end: InlineItemTextIndex,
    pub is_hyphenated: bool,
    // C++ EXPENSIVE_DCHECKS_ARE_ON() adds this field only in that build.
    #[cfg(feature = "expensive_dchecks")]
    pub line_width: LayoutUnit,
}

#[allow(non_snake_case)]
impl LineBreakPoint {
    // cpp: layoutng_inline/line_break_point.h:25-33
    pub fn new(offset: InlineItemTextIndex, end: InlineItemTextIndex, is_hyphenated: bool) -> Self {
        Self {
            offset,
            end,
            is_hyphenated,
            #[cfg(feature = "expensive_dchecks")]
            line_width: LayoutUnit::default(),
        }
    }

    pub fn from_offset(offset: InlineItemTextIndex, is_hyphenated: bool) -> Self {
        Self::new(offset, offset, is_hyphenated)
    }

    // cpp: layoutng_inline/line_break_point.h:35-35
    pub fn IsNotZero(&self) -> bool {
        self.offset.text_offset != 0
    }
}

// cpp: layoutng_inline/line_break_point.h:37-40
impl PartialEq for LineBreakPoint {
    fn eq(&self, other: &Self) -> bool {
        self.offset == other.offset
            && self.end == other.end
            && self.is_hyphenated == other.is_hyphenated
    }
}
impl Eq for LineBreakPoint {}
