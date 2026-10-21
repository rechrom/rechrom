// Source: src/core/SkSafeRange.h:46-46
pub struct SkSafeRange {
    fOK: bool,
}
impl SkSafeRange {
    // Source: src/core/SkSafeRange.h:46-46
    pub fn new() -> Self {
        Self { fOK: true }
    }
    // Source: src/core/SkSafeRange.h:21-21
    pub fn as_bool(&self) -> bool {
        self.fOK
    }
    // Source: src/core/SkSafeRange.h:23-23
    pub fn ok(&self) -> bool {
        self.fOK
    }
    // Source: src/core/SkSafeRange.h:28-35
    // Primitive integer templates use the existing sealed cast representation;
    // callers pass checked enum maxima as their actual unsigned wire view.
    pub fn checkLE(&mut self, mut value: u64, max: u32) -> u32 {
        debug_assert!((max as i64) >= 0);
        if value > max as u64 {
            self.fOK = false;
            value = 0;
        }
        value as u32
    }
    // Source: src/core/SkSafeRange.h:37-43
    pub fn checkGE(&mut self, mut value: i32, min: i32) -> i32 {
        if value < min {
            self.fOK = false;
            value = min;
        }
        value
    }
}
