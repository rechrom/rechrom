//! Shared platform-neutral referrer policy; request transport preserves its
//! original public path while DOM/CSSOM consume the same typed value.
pub use foundation::referrer_policy::{IsSameOrigin, ReferrerPolicy};
