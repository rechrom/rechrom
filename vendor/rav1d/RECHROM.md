# Rechrom rav1d pin and official robustness backports

- Official package: https://crates.io/crates/rav1d/1.1.0
- Source revision: https://github.com/memorysafety/rav1d/tree/782dab2135ea64a057c097088a13eb8ed3cc3320
- Official `.crate` SHA-256: `1932f060d5e7bd49dc9f8b272c1dc5e9ce0ffe141c28be900265d3989b36c9ed`
- License: BSD-2-Clause; the complete official COPYING and copyright notices are retained

`UPSTREAM_MANIFEST.json` records every original packaged file. All package files
are preserved. Only these two source files are changed, with exact pre/post
hashes in `RECHROM-backport-hashes.json` and a reviewable
`RECHROM-backports.patch`:

1. `src/decode.rs`: https://github.com/memorysafety/rav1d/commit/6c6f463785a546dea18fd9ca0f14634c2ee0557c
   (PR #1503; issue #1497). Error cleanup must not unwrap a frame header already
   taken by frame-exit cleanup in low-latency mode. This matters with Crabby's
   `max_frame_delay=1`; an outer panic catcher cannot contain an extern-C abort
2. `src/obu.rs`: https://github.com/memorysafety/rav1d/commit/9d78516f3232fe8b9c2bfe9d9a7506163033aae6
   (PR #1476). Signed ITU-T T.35 payload-size arithmetic prevents malformed
   trailing-byte underflow. Context is adapted to 1.1.0's existing `<= 0` check

These narrow upstream corrections do not imply a complete security audit or
full conformance to newer dav1d. The initial transpilation was rav1d commit
`c048d92e2679182f13ddf15fffb62ff1d039c54d`, with VideoLAN dav1d parent
`3b7b09613489bd45922ae76ff4b1dbbdf04ae7a6` (2023-02-03). Later selective
backports include dav1d 1.5.1 changes; this is not an exact translation of one
later dav1d SHA. The embedded version string `966d63c1` is a rav1d revision,
not proof of a VideoLAN dav1d source revision.

The consuming manifests explicitly disable default features and select only
`bitdepth_8` and `bitdepth_16`. The original assembly source and build script are
retained for provenance, but the `asm` feature is absent and build.rs performs
no assembly compilation or native-library linking in this configuration. Its
`cc` and `nasm-rs` build dependencies remain Rust tools in the resolved graph;
their presence is not evidence of a C/C++ codec being compiled. Feature-tree,
build-output and artifact-link audits must verify this configuration.
