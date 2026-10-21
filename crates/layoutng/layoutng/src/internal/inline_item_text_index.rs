// cpp: layoutng/internal/inline_item_text_index.h:15-39
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InlineItemTextIndex {
    pub item_index: u32,
    pub text_offset: u32,
}

#[allow(non_snake_case)]
impl InlineItemTextIndex {
    // cpp: layoutng/internal/inline_item_text_index.h:16-17
    pub fn IsNotZero(&self) -> bool {
        self.text_offset != 0 || self.item_index != 0
    }

    pub fn IsZero(&self) -> bool {
        self.text_offset == 0 && self.item_index == 0
    }

    // Both coordinates are compared with OR in the source. This is not a
    // consistent PartialOrd, so the four operators have named methods.
    // cpp: layoutng/internal/inline_item_text_index.h:22-27
    pub fn GreaterThan(&self, other: &Self) -> bool {
        self.text_offset > other.text_offset || self.item_index > other.item_index
    }

    pub fn LessThan(&self, other: &Self) -> bool {
        self.text_offset < other.text_offset || self.item_index < other.item_index
    }

    // cpp: layoutng/internal/inline_item_text_index.h:28-33
    pub fn GreaterOrEqual(&self, other: &Self) -> bool {
        !self.LessThan(other)
    }

    pub fn LessOrEqual(&self, other: &Self) -> bool {
        !self.GreaterThan(other)
    }
}

// cpp: layoutng/internal/inline_item_text_index.h:41-44
impl std::fmt::Display for InlineItemTextIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{{},{}}}", self.item_index, self.text_offset)
    }
}
