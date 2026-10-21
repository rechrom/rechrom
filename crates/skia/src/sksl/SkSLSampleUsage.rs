// Copyright 2020 Google LLC. BSD-style license.
// Source: include/private/SkSLSampleUsage.h:20-31
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    kNone,
    kPassThrough,
    kUniformMatrix,
    kFragCoord,
    kExplicit,
}
// Source: include/private/SkSLSampleUsage.h:34-34
impl Default for SampleUsage {
    fn default() -> Self {
        Self {
            fKind: Kind::kNone,
            fHasPerspective: false,
        }
    }
}
impl SampleUsage {
    // Source: include/private/SkSLSampleUsage.h:36-40
    pub fn new(kind: Kind, hasPerspective: bool) -> Self {
        let value = Self {
            fKind: kind,
            fHasPerspective: hasPerspective,
        };
        if kind != Kind::kUniformMatrix {
            debug_assert!(!value.fHasPerspective);
        }
        value
    }
    // Source: include/private/SkSLSampleUsage.h:43-45
    pub fn UniformMatrix(hasPerspective: bool) -> Self {
        Self::new(Kind::kUniformMatrix, hasPerspective)
    }
    // Source: include/private/SkSLSampleUsage.h:47-49
    pub fn Explicit() -> Self {
        Self::new(Kind::kExplicit, false)
    }
    // Source: include/private/SkSLSampleUsage.h:51-53
    pub fn PassThrough() -> Self {
        Self::new(Kind::kPassThrough, false)
    }
    // Source: include/private/SkSLSampleUsage.h:55-55
    pub fn FragCoord() -> Self {
        Self::new(Kind::kFragCoord, false)
    }
}
// Source: include/private/SkSLSampleUsage.h:57-61
impl PartialEq for SampleUsage {
    fn eq(&self, that: &Self) -> bool {
        self.fKind == that.fKind && self.fHasPerspective == that.fHasPerspective
    }
    fn ne(&self, that: &Self) -> bool {
        !self.eq(that)
    }
}
impl Eq for SampleUsage {}
impl SampleUsage {
    // Source: include/private/SkSLSampleUsage.h:64-64
    pub fn MatrixUniformName() -> &'static str {
        "matrix"
    }
    // Source: include/private/SkSLSampleUsage.h:68-76
    pub fn kind(&self) -> Kind {
        self.fKind
    }
    pub fn hasPerspective(&self) -> bool {
        self.fHasPerspective
    }
    pub fn isSampled(&self) -> bool {
        self.fKind != Kind::kNone
    }
    pub fn isPassThrough(&self) -> bool {
        self.fKind == Kind::kPassThrough
    }
    pub fn isExplicit(&self) -> bool {
        self.fKind == Kind::kExplicit
    }
    pub fn isUniformMatrix(&self) -> bool {
        self.fKind == Kind::kUniformMatrix
    }
    pub fn isFragCoord(&self) -> bool {
        self.fKind == Kind::kFragCoord
    }
}
// Source: include/private/SkSLSampleUsage.h:79-81
// Adapted flat Rust storage retains native field order; defaults occur above.
#[derive(Clone, Copy, Debug)]
pub struct SampleUsage {
    fKind: Kind,
    fHasPerspective: bool,
}
impl SampleUsage {
    // Source: src/sksl/SkSLSampleUsage.cpp:14-24
    pub fn merge(&mut self, other: &Self) -> Self {
        crate::SkASSERT!(@source b"src/sksl/SkSLSampleUsage.cpp\0".as_ptr(),17; @condition b"fKind != Kind::kUniformMatrix && other.fKind != Kind::kUniformMatrix\0".as_ptr(); self.fKind != Kind::kUniformMatrix && other.fKind != Kind::kUniformMatrix);
        const _: () = {
            assert!((Kind::kExplicit as i32) > Kind::kPassThrough as i32);
            assert!((Kind::kPassThrough as i32) > Kind::kNone as i32);
        };
        self.fKind = self.fKind.max(other.fKind);
        *self
    }
}
