//! Source Chromium double-conversion/WTF numeric primitives shared by DOM
//! and HTML/default owners. The implementation has one owner and no engine
//! dependencies; engine-specific Decimal adapters keep their original paths.
pub mod double_conversion;
pub mod dtoa;
