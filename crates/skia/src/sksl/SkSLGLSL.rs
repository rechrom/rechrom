// UNVERIFIED; native enum represented by its actual int value to retain aliases in source order.
// Source: src/sksl/SkSLGLSL.h:15-15
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GLSLGeneration(pub i32);
impl GLSLGeneration {
    // Source: src/sksl/SkSLGLSL.h:19-19
    pub const k110: Self = Self(0);
    // Source: src/sksl/SkSLGLSL.h:20-20
    pub const k100es: Self = Self::k110;
    // Source: src/sksl/SkSLGLSL.h:24-24
    pub const k130: Self = Self(1);
    // Source: src/sksl/SkSLGLSL.h:28-28
    pub const k140: Self = Self(2);
    // Source: src/sksl/SkSLGLSL.h:32-32
    pub const k150: Self = Self(3);
    // Source: src/sksl/SkSLGLSL.h:36-36
    pub const k330: Self = Self(4);
    // Source: src/sksl/SkSLGLSL.h:37-37
    pub const k300es: Self = Self::k330;
    // Source: src/sksl/SkSLGLSL.h:41-41
    pub const k400: Self = Self(5);
    // Source: src/sksl/SkSLGLSL.h:45-45
    pub const k420: Self = Self(6);
    // Source: src/sksl/SkSLGLSL.h:49-49
    pub const k310es: Self = Self(7);
    // Source: src/sksl/SkSLGLSL.h:53-53
    pub const k320es: Self = Self(8);
}
