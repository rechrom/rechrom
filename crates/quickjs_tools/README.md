# QuickJS host tools in Rust

This crate contains Rust translations of the host and tool programs distributed with QuickJS 2026-06-04, including shell services, the bytecode compiler, the Test262 runner, and Unicode table generation.

The engine stays in the dependency-free `quickjs` crate. POSIX bindings and command-line services live here so the browser engine does not expose host shell capabilities.

macOS is the currently exercised host platform. Windows and other host configurations remain in development.
