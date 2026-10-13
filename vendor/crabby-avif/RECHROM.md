# Rechrom decode-only CrabbyAvif packaging

Upstream: https://github.com/webmproject/CrabbyAvif/tree/8f2b74b4d8be956395f527020c2190762e086f98

This is the CrabbyAvif revision pinned by Chromium
`6c1d401fcca5e1b0030563a90c2f2bba168e0c15` in DEPS. All original `src/`
files and the Apache-2.0 LICENSE are retained. `UPSTREAM_MANIFEST.json` records
original SHA-256 hashes. `Cargo.toml.upstream` and `build.rs.upstream` retain the
original build configuration without executing it.

## Deliberate packaging and integration differences

- The decode-only Cargo manifest builds an rlib. No build script, C API, CLI,
  encoder, native dav1d/aom, libyuv, sharpyuv, PNG/JPEG/GIF helper, HEIC, AV2,
  JPEG XL or experimental sample-transform feature can be selected here
- `src/codecs/rav1d.rs` is adapted directly from the retained upstream
  `src/codecs/dav1d.rs`. Its changes are typed Rust API imports, Option/NonNull
  pointer adaptation, Default initialized structs, Dav1dResult.0, the opaque
  RawArc context, and version labeling. The two-thread configuration, low-latency
  frame delay, operating point/all-layers, frame-size limit, send/get EAGAIN
  schedule, drain, category handling, grid/final-frame paths and retained-picture
  lifetime follow the pinned adapter. Rechrom additionally rejects empty codec
  payloads, unconditionally unreferences initialized data wrappers, and checks
  positive dimensions/valid depth/non-null plane pointers/positive aligned
  sufficient strides before creating borrowed plane views
- `codecs/mod.rs`, `decoder/mod.rs`, `image.rs` and `lib.rs` register the Rust
  codec and preserve the same grid/final-frame eligibility branches. The native
  adapter source is retained for audit but never compiled
- The exact rav1d crate version is 1.1.0, with default features and all assembly
  disabled, and both Rust 8-bit and 16-bit (10/12-bit AV1) paths enabled. Rechrom
  patches it to a vendored source with two separately identified official
  upstream robustness backports; see `../rav1d/RECHROM.md`

The AVIF candidate is private to `image_decoder`, opt-in with `avif-backend`.
Production format dispatch remains unchanged. Crabby's Rust RGB fallback is
source-aligned but not verified byte-equivalent to Chromium's enabled libyuv
route. rav1d 1.1.0 is not the dav1d source revision pinned by Chromium. Neither
fact may be described as exact Blink output parity. See
`../../docs/avif-rust-backend.md` for the admission and verification boundaries.
