// Source: modules/skcms/src/skcms_internals.h:22-22
// Native array extent, narrowed to native int after sizeof division. No array
// elements are evaluated or copied by this compile-time size projection.
pub const fn ARRAY_COUNT<T, const N: usize>(_: &[T; N]) -> i32 {
    N as i32
}
// Source: modules/skcms/src/skcms_internals.h:99-99
pub const fn SAFE_SIZEOF<T>() -> u64 {
    core::mem::size_of::<T>() as u64
}
// Source: modules/skcms/src/skcms_internals.h:102-102
// Original offsetof(type, variable), retaining the real declared layout.
macro_rules! SAFE_FIXED_SIZE {
    ($ty:ty) => {
        core::mem::offset_of!($ty, variable) as u64
    };
}
pub(crate) use SAFE_FIXED_SIZE;
// Source: modules/skcms/src/skcms_internals.h:126-131
// Borrowed tag data retains the original profile-buffer lifetime, bounds and
// immutability contract. No parser, buffer owner or default tag is synthesized.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct skcms_ICCTag {
    pub signature: u32,
    pub r#type: u32,
    pub size: u32,
    pub buf: *const u8,
}
// Source: modules/skcms/src/skcms_internals.h:133-135
pub use super::skcms_public::{skcms_Curve, skcms_ICCProfile, skcms_TransferFunction};
// Source: modules/skcms/src/skcms_internals.h:140-140
pub use crate::modules::skcms::skcms::skcms_MaxRoundtripError;
// Source: modules/skcms/src/skcms_internals.h:145-145
pub use crate::modules::skcms::skcms::skcms_252_random_bytes;
// Portable math from pinned skcms. Integer conversion has the source finite/in-range precondition.
// Source: modules/skcms/src/skcms_internals.h:148-152
pub fn floorf_(x: f32) -> f32 {
    let roundtrip = (x as i32) as f32;
    if roundtrip > x {
        roundtrip - 1.0
    } else {
        roundtrip
    }
}
pub fn fabsf_(x: f32) -> f32 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

// Source: modules/skcms/src/skcms_internals.h:153-153
pub use crate::modules::skcms::skcms::powf_;

// Source: modules/skcms/src/skcms_internals.h:138-139
pub use crate::modules::skcms::skcms::{skcms_GetTagByIndex, skcms_GetTagBySignature};
