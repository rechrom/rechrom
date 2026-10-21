// Source: include/private/SkTPin.h:18-21
// Numeric Copy values replace the native template's reference return at its scalar call sites.
#[inline]
pub const fn SkTPin<T: Copy + [const] PartialOrd>(x: T, lo: T, hi: T) -> T {
    let upper = if hi < x { hi } else { x };
    if lo < upper {
        upper
    } else {
        lo
    }
}
// Source: include/private/SkTPin.h:18-21
// Native reference-return form preserves identity and supports non-Copy values.
// The value-return projection above remains for existing scalar consumers.
pub const fn SkTPin_ref<'a, T: [const] PartialOrd>(x: &'a T, lo: &'a T, hi: &'a T) -> &'a T {
    let upper = if hi < x { hi } else { x };
    if lo < upper {
        upper
    } else {
        lo
    }
}
