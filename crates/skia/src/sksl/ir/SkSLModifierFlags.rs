// Source: src/sksl/ir/SkSLModifierFlags.h:18-18
pub use crate::src::sksl::SkSLContext::Context;
// Source: src/sksl/ir/SkSLModifierFlags.h:19-19
pub use crate::src::sksl::SkSLPosition::Position;
// Source: src/sksl/ir/SkSLModifierFlags.h:21-45
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ModifierFlag(pub i32);
impl ModifierFlag {
    pub const kNone: Self = Self(0);
    pub const kFlat: Self = Self(1 << 0);
    pub const kNoPerspective: Self = Self(1 << 1);
    pub const kConst: Self = Self(1 << 2);
    pub const kUniform: Self = Self(1 << 3);
    pub const kIn: Self = Self(1 << 4);
    pub const kOut: Self = Self(1 << 5);
    pub const kHighp: Self = Self(1 << 6);
    pub const kMediump: Self = Self(1 << 7);
    pub const kLowp: Self = Self(1 << 8);
    pub const kReadOnly: Self = Self(1 << 9);
    pub const kWriteOnly: Self = Self(1 << 10);
    pub const kBuffer: Self = Self(1 << 11);
    pub const kPixelLocal: Self = Self(1 << 12);
    pub const kWorkgroup: Self = Self(1 << 13);
    pub const kExport: Self = Self(1 << 14);
    pub const kES3: Self = Self(1 << 15);
    pub const kPure: Self = Self(1 << 16);
    pub const kInline: Self = Self(1 << 17);
    pub const kNoInline: Self = Self(1 << 18);
}
// Source: src/sksl/ir/SkSLModifierFlags.h:53-57
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ModifierFlags(pub i32);
impl ModifierFlags {
    // Source: src/sksl/ir/SkSLModifierFlags.h:70-70
    pub fn isConst(self) -> bool {
        self.0 & ModifierFlag::kConst.0 != 0
    }
    // Source: src/sksl/ir/SkSLModifierFlags.h:71-71
    pub fn isUniform(self) -> bool {
        self.0 & ModifierFlag::kUniform.0 != 0
    }
    // Source: src/sksl/ir/SkSLModifierFlags.h:72-72
    pub fn isReadOnly(self) -> bool {
        self.0 & ModifierFlag::kReadOnly.0 != 0
    }
    // Source: src/sksl/ir/SkSLModifierFlags.h:73-73
    pub fn isWriteOnly(self) -> bool {
        self.0 & ModifierFlag::kWriteOnly.0 != 0
    }
    // Source: src/sksl/ir/SkSLModifierFlags.h:74-74
    pub fn isBuffer(self) -> bool {
        self.0 & ModifierFlag::kBuffer.0 != 0
    }
    // Source: src/sksl/ir/SkSLModifierFlags.h:75-75
    pub fn isPixelLocal(self) -> bool {
        self.0 & ModifierFlag::kPixelLocal.0 != 0
    }
    // Source: src/sksl/ir/SkSLModifierFlags.h:76-76
    pub fn isWorkgroup(self) -> bool {
        self.0 & ModifierFlag::kWorkgroup.0 != 0
    }
    // Source: src/sksl/ir/SkSLModifierFlags.h:77-77
    pub fn isExport(self) -> bool {
        self.0 & ModifierFlag::kExport.0 != 0
    }
    // Source: src/sksl/ir/SkSLModifierFlags.h:78-78
    pub fn isES3(self) -> bool {
        self.0 & ModifierFlag::kES3.0 != 0
    }
    // Source: src/sksl/ir/SkSLModifierFlags.h:79-79
    pub fn isPure(self) -> bool {
        self.0 & ModifierFlag::kPure.0 != 0
    }
    // Source: src/sksl/ir/SkSLModifierFlags.h:80-80
    pub fn isInline(self) -> bool {
        self.0 & ModifierFlag::kInline.0 != 0
    }
    // Source: src/sksl/ir/SkSLModifierFlags.h:81-81
    pub fn isNoInline(self) -> bool {
        self.0 & ModifierFlag::kNoInline.0 != 0
    }
    // Source: src/sksl/ir/SkSLModifierFlags.h:82-82
    pub fn isFlat(self) -> bool {
        self.0 & ModifierFlag::kFlat.0 != 0
    }
    // Source: src/sksl/ir/SkSLModifierFlags.h:83-83
    pub fn isNoPerspective(self) -> bool {
        self.0 & ModifierFlag::kNoPerspective.0 != 0
    }
}
// Rust bitmask operator bridge for native SkEnumBitMask inheritance.
impl From<ModifierFlag> for ModifierFlags {
    fn from(v: ModifierFlag) -> Self {
        Self(v.0)
    }
}
impl core::ops::BitOr for ModifierFlag {
    type Output = ModifierFlags;
    fn bitor(self, r: Self) -> ModifierFlags {
        ModifierFlags(self.0 | r.0)
    }
}
impl core::ops::BitOr<ModifierFlag> for ModifierFlags {
    type Output = Self;
    fn bitor(self, r: ModifierFlag) -> Self {
        Self(self.0 | r.0)
    }
}
impl core::ops::BitAnd<ModifierFlag> for ModifierFlags {
    type Output = Self;
    fn bitand(self, r: ModifierFlag) -> Self {
        Self(self.0 & r.0)
    }
}
impl core::ops::BitOr for ModifierFlags {
    type Output = Self;
    fn bitor(self, r: Self) -> Self {
        Self(self.0 | r.0)
    }
}
impl core::ops::BitAnd for ModifierFlags {
    type Output = Self;
    fn bitand(self, r: Self) -> Self {
        Self(self.0 & r.0)
    }
}
impl core::ops::Not for ModifierFlags {
    type Output = Self;
    fn not(self) -> Self {
        Self(!self.0)
    }
}

impl ModifierFlags {
    // Source: src/sksl/ir/SkSLModifierFlags.cpp:17-86
    pub fn paddedDescription(&self) -> Vec<u8> {
        let mut result = Vec::new();
        if self.0 & ModifierFlag::kExport.0 != 0 {
            result.extend_from_slice(b"$export ");
        }
        if self.0 & ModifierFlag::kES3.0 != 0 {
            result.extend_from_slice(b"$es3 ");
        }
        if self.0 & ModifierFlag::kPure.0 != 0 {
            result.extend_from_slice(b"$pure ");
        }
        if self.0 & ModifierFlag::kInline.0 != 0 {
            result.extend_from_slice(b"inline ");
        }
        if self.0 & ModifierFlag::kNoInline.0 != 0 {
            result.extend_from_slice(b"noinline ");
        }
        if self.0 & ModifierFlag::kFlat.0 != 0 {
            result.extend_from_slice(b"flat ");
        }
        if self.0 & ModifierFlag::kNoPerspective.0 != 0 {
            result.extend_from_slice(b"noperspective ");
        }
        if self.0 & ModifierFlag::kConst.0 != 0 {
            result.extend_from_slice(b"const ");
        }
        if self.0 & ModifierFlag::kUniform.0 != 0 {
            result.extend_from_slice(b"uniform ");
        }
        if (self.0 & ModifierFlag::kIn.0 != 0) && (self.0 & ModifierFlag::kOut.0 != 0) {
            result.extend_from_slice(b"inout ");
        } else if self.0 & ModifierFlag::kIn.0 != 0 {
            result.extend_from_slice(b"in ");
        } else if self.0 & ModifierFlag::kOut.0 != 0 {
            result.extend_from_slice(b"out ");
        }
        if self.0 & ModifierFlag::kHighp.0 != 0 {
            result.extend_from_slice(b"highp ");
        }
        if self.0 & ModifierFlag::kMediump.0 != 0 {
            result.extend_from_slice(b"mediump ");
        }
        if self.0 & ModifierFlag::kLowp.0 != 0 {
            result.extend_from_slice(b"lowp ");
        }
        if self.0 & ModifierFlag::kReadOnly.0 != 0 {
            result.extend_from_slice(b"readonly ");
        }
        if self.0 & ModifierFlag::kWriteOnly.0 != 0 {
            result.extend_from_slice(b"writeonly ");
        }
        if self.0 & ModifierFlag::kBuffer.0 != 0 {
            result.extend_from_slice(b"buffer ");
        }
        if self.0 & ModifierFlag::kPixelLocal.0 != 0 {
            result.extend_from_slice(b"pixel_local ");
        }
        if self.0 & ModifierFlag::kWorkgroup.0 != 0 {
            result.extend_from_slice(b"workgroup ");
        }
        result
    }
    // Source: src/sksl/ir/SkSLModifierFlags.cpp:88-94
    pub fn description(&self) -> Vec<u8> {
        let mut s = self.paddedDescription();
        if !s.is_empty() {
            s.pop();
        }
        s
    }
    // Source: src/sksl/ir/SkSLModifierFlags.cpp:96-135
    pub unsafe fn checkPermittedFlags(
        &self,
        context: &crate::src::sksl::SkSLContext::Context,
        pos: crate::src::sksl::SkSLPosition::Position,
        permitted: ModifierFlags,
    ) -> bool {
        use crate::src::sksl::SkSLErrorReporter::ErrorReporterExt;
        const FLAGS: &[(ModifierFlag, &[u8])] = &[
            (ModifierFlag::kConst, b"const"),
            (ModifierFlag::kIn, b"in"),
            (ModifierFlag::kOut, b"out"),
            (ModifierFlag::kUniform, b"uniform"),
            (ModifierFlag::kFlat, b"flat"),
            (ModifierFlag::kNoPerspective, b"noperspective"),
            (ModifierFlag::kPure, b"$pure"),
            (ModifierFlag::kInline, b"inline"),
            (ModifierFlag::kNoInline, b"noinline"),
            (ModifierFlag::kHighp, b"highp"),
            (ModifierFlag::kMediump, b"mediump"),
            (ModifierFlag::kLowp, b"lowp"),
            (ModifierFlag::kExport, b"$export"),
            (ModifierFlag::kES3, b"$es3"),
            (ModifierFlag::kWorkgroup, b"workgroup"),
            (ModifierFlag::kReadOnly, b"readonly"),
            (ModifierFlag::kWriteOnly, b"writeonly"),
            (ModifierFlag::kBuffer, b"buffer"),
            (ModifierFlag::kPixelLocal, b"pixel_local"),
        ];
        let mut success = true;
        let mut flags = self.0;
        for (flag, name) in FLAGS {
            if flags & flag.0 != 0 {
                if permitted.0 & flag.0 == 0 {
                    let mut message = b"'".to_vec();
                    message.extend_from_slice(name);
                    message.extend_from_slice(b"' is not permitted here");
                    let mut errors = context.fErrors;
                    errors.as_mut().error(pos, &message);
                    success = false;
                }
                flags &= !flag.0;
            }
        }
        debug_assert_eq!(flags, 0);
        success
    }
}
