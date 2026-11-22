//! Shared integration-test support. Cargo does not build `tests/support/` as
//! its own test target; each integration test includes it with `mod support;`.
pub mod test_thread;
