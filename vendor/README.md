# vendor/

Third-party crates Rechrom carries with local changes. Every other dependency
comes unmodified from crates.io, pinned with checksums in `Cargo.lock`; there
are no git dependencies.

Rules (see `DECISIONS.md` for the per-crate evidence):

- A crate is vendored only when its patch is necessary: a real fix with no
  released alternative, Chromium-parity behaviour, or a replacement that would
  be a large rewrite for no gain. Otherwise it comes from crates.io.
- Operating-system functionality belongs in `apps/rechrom` platform modules,
  not in a patched library (AGENTS.md §1.5); patches there shrink to hooks.
- Each crate's exact difference from its pristine upstream package lives in
  `patches/<crate>.patch`, the way Chromium keeps
  `third_party/rust/chromium_crates_io/patches`. Regenerate the patches with
  `tools/vendor_patches.sh <pristine-root>` after any change here; the diff
  must stay reviewable.
- When upstream releases the fix, switch back to crates.io and delete the
  crate here.

| Crate | Upstream | SHA-256 | Patch |
|---|---|---|---|
| `crabby-avif` | <https://github.com/webmproject/CrabbyAvif/archive/8f2b74b4d8be956395f527020c2190762e086f98.tar.gz> | `0ba0db75f208edfbfeed855f1a8fb667b615122f9f5d786ceaeaaa081a4d450c` | +755 / -163 |
| `hyper-util` | <https://static.crates.io/crates/hyper-util/hyper-util-0.1.21.crate> | `ddc03d96684f9226b8a787cdb71488417b53ab5ea8fdb1dac946cb9431cc8bff` | +61 / -66 |
| `rav1d` | <https://static.crates.io/crates/rav1d/rav1d-1.1.0.crate> | `1932f060d5e7bd49dc9f8b272c1dc5e9ce0ffe141c28be900265d3989b36c9ed` | +3 / -3 |
| `sfv` | <https://static.crates.io/crates/sfv/sfv-0.15.0.crate> | `890ef0e5532daf21fdaa3dbaf991417187f3a87ab69231fe69be942495746480` | +90 / -20 |
| `smithay-clipboard` | <https://static.crates.io/crates/smithay-clipboard/smithay-clipboard-0.7.2.crate> | `cc8216eec463674a0e90f29e0ae41a4db573ec5b56b1c6c1c71615d249b6d846` | +356 / -139 |
| `softbuffer` | <https://static.crates.io/crates/softbuffer/softbuffer-0.4.8.crate> | `aac18da81ebbf05109ab275b157c22a653bb3c12cf884450179942f81bcbf6c3` | +1152 / -41 |
| `winit` | <https://static.crates.io/crates/winit/winit-0.30.13.crate> | `a6755fa58a9f8350bd1e472d4c3fcc25f824ec358933bba33306d0b63df5978d` | +3245 / -784 |
| `xml` | <https://static.crates.io/crates/xml/xml-1.4.0.crate> | `2f45bb2c13fec6a6cb4c0f76a7e94839e110a14ec803ec2940777a94c347bc52` | +212 / -4 |
| `zune-jpeg` | <https://static.crates.io/crates/zune-jpeg/zune-jpeg-0.5.15.crate> | `27bc9d5b815bc103f142aa054f561d9187d191692ec7c2d1e2b4737f8dbd7296` | +845 / -0 |
