//! Compatibility adapter for the migrated two-circle radial-gradient API.
//! Upstream SkRadialGradient is a single-center radius specialization. The migrated
//! API accepts two circles and therefore its implementation lives in SkConicalGradient.
//! This alias is a compatibility mapping, not an implementation of SkRadialGradient.
pub use super::SkConicalGradient::SkConicalGradient as RadialGradient;
