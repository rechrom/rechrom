// cpp: layoutng_style/style/max_lines_data.h:15-29
/// CSS max-lines count, optionally combined with `auto`.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaxLinesData {
    lines_: u16,
    has_auto_: bool,
}

#[allow(non_snake_case)]
impl MaxLinesData {
    pub fn new(lines: u16, has_auto: bool) -> Self {
        let result = Self {
            lines_: lines,
            has_auto_: has_auto,
        };
        if result.lines_ == 0 {
            debug_assert!(
                result.has_auto_,
                "max-lines can't have zero lines without auto"
            );
        }
        result
    }

    // cpp: layoutng_style/style/max_lines_data.h:31-39
    // Equality is derived above.
    pub fn Lines(&self) -> u32 {
        self.lines_ as u32
    }
    pub fn HasAutoKeyword(&self) -> bool {
        self.has_auto_
    }
    pub fn IsAutoValue(&self) -> bool {
        self.lines_ == 0 && self.has_auto_
    }
}
