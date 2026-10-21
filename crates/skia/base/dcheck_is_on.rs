// Source: base/dcheck_is_on.h:10-14
// Inputs are the original preprocessor presences, independent of Rust profiles.
pub const DCHECK_IS_ON: bool = cfg!(any(not(sk_native_ndebug), DCHECK_ALWAYS_ON));
// Source: base/dcheck_is_on.h:16-20
pub const EXPENSIVE_DCHECKS_ARE_ON: bool = cfg!(EXPENSIVE_DCHECKS_ARE_ON);
// Source: base/dcheck_is_on.h:22-26
pub const ENABLE_CHECK_ELISION_WARNING: bool = cfg!(ENABLE_CHECK_ELISION_WARNING);
