// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: base/time/time.h:395-400,1175-1178
// Narrow value-only translation of TimeTicks. The embedding supplies the clock;
// input events retain the source's signed microsecond value and null origin.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct TimeTicks(i64);

impl TimeTicks {
    // Source: base/time/time.h:1190
    pub const fn new() -> Self {
        Self(0)
    }

    // Source: base/time/time.h:1294-1305,1316-1318
    pub const fn FromInternalValue(us: i64) -> Self {
        Self(us)
    }

    // Source: base/time/time.h:426-433
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    // Source: base/time/time.h:450-454
    pub const fn ToInternalValue(self) -> i64 {
        self.0
    }
}
