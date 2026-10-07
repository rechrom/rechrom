# QuickJS in Rust

This crate contains Rechrom's Rust translation of Bellard QuickJS 2026-06-04. Runtime code is Rust and does not use QuickJS through C FFI. The unchanged upstream source is retained under `vendor/quickjs-2026-06-04` for differential tests and licensing.

The low-level public interface follows `quickjs.h`: create a runtime with `JS_NewRuntime`, create a context with `JS_NewContext`, evaluate with `JS_Eval`, and release returned values and owners. Rechrom's safe browser-facing adapter lives in `crates/javascript`.

Default features preserve compact opcodes and SharedArrayBuffer/Atomics support. Optional features expose upstream allocator and diagnostic configurations used for compatibility work.

The translation retains upstream MIT licensing and original symbol names. Rechrom-specific module embedding and exception metadata hooks are opt-in; ordinary evaluation retains upstream behavior.
