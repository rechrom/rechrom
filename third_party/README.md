# Chromium source attribution

`LICENSE.chromium` is the complete, unchanged Chromium root LICENSE fetched
from the official Chromium mirror at revision
`78ef49f74feba3d0dde852881feef92bbf86aad7`:

https://raw.githubusercontent.com/chromium/chromium/78ef49f74feba3d0dde852881feef92bbf86aad7/LICENSE

License file SHA-256:
`368cca1106be99d39ecd32a38d8305585d802a475effb66380b91ffc9bcf709b`

The standardized sequence table in
`crates/font_engine/src/text/native/variation_sequence_data.rs` is derived
from that revision's
`third_party/blink/renderer/platform/text/character_variation_sequences.cc`:

https://raw.githubusercontent.com/chromium/chromium/78ef49f74feba3d0dde852881feef92bbf86aad7/third_party/blink/renderer/platform/text/character_variation_sequences.cc

Source file SHA-256:
`e54300375f232819506fb1d07a5ca3b6ea1531c03c917cc207a930fa8af7d02d`

The Rust table was checked against all source pairs after packing each pair
as `(base << 21) | selector`, deduplicating and sorting. Its original 2024
Chromium copyright notice is retained. The same pinned Chromium revision is
the source reference for the fallback iterator and shaping alignment described
in `docs/font-engine.md`.

This notice records the identified upstream source and preserves its license;
it does not assign a license to the entire Rechrom repository. The separately
supplied foundation translation has its own provenance limitations documented
in `crates/foundation/base/README.md`.
