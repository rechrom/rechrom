// cpp: layoutng/internal/text_offset_range.h:14-35
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextOffsetRange {
    pub start: u32,
    pub end: u32,
}

#[allow(non_snake_case)]
impl TextOffsetRange {
    // cpp: layoutng/internal/text_offset_range.h:17-19
    pub fn new(start: u32, end: u32) -> Self {
        let range = Self { start, end };
        range.AssertValid();
        range
    }

    // cpp: layoutng/internal/text_offset_range.h:21-24
    pub fn Length(&self) -> u32 {
        self.AssertValid();
        self.end - self.start
    }

    // cpp: layoutng/internal/text_offset_range.h:26-27
    pub fn AssertValid(&self) {
        debug_assert!(self.end >= self.start);
    }

    pub fn AssertNotEmpty(&self) {
        debug_assert!(self.end > self.start);
    }
}
