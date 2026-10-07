//! dtoa.h. Copyright 2024 Fabrice Bellard; MIT, see ../LICENSE.
pub const JS_DTOA_MAX_DIGITS: i32 = 101;
pub const JS_DTOA_FORMAT_FREE: i32 = 0;
pub const JS_DTOA_FORMAT_FIXED: i32 = 1;
pub const JS_DTOA_FORMAT_FRAC: i32 = 2;
pub const JS_DTOA_FORMAT_MASK: i32 = 3;
pub const JS_DTOA_EXP_AUTO: i32 = 0;
pub const JS_DTOA_EXP_ENABLED: i32 = 4;
pub const JS_DTOA_EXP_DISABLED: i32 = 8;
pub const JS_DTOA_EXP_MASK: i32 = 12;
pub const JS_DTOA_MINUS_ZERO: i32 = 16;
pub const JS_ATOD_INT_ONLY: i32 = 1;
pub const JS_ATOD_ACCEPT_BIN_OCT: i32 = 2;
pub const JS_ATOD_ACCEPT_LEGACY_OCTAL: i32 = 4;
pub const JS_ATOD_ACCEPT_UNDERSCORES: i32 = 8;
#[repr(C)]
pub struct JSDTOATempMem {
    pub mem: [u64; 37],
}
#[repr(C)]
pub struct JSATODTempMem {
    pub mem: [u64; 27],
}
impl Default for JSDTOATempMem {
    fn default() -> Self {
        Self { mem: [0; 37] }
    }
}
impl Default for JSATODTempMem {
    fn default() -> Self {
        Self { mem: [0; 27] }
    }
}
