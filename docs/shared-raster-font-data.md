# Retain immutable font backing for raster identity

Raster's PaintResources already retains `font_engine::SharedFontBytes`, which
owns either immutable allocated data or a read-only mapping. The Skia boundary
now accepts an optional opaque retained immutable byte owner alongside its
existing borrowed face view. Raster supplies a cheap SharedFontBytes clone; it
does not create a file-sized intermediate Vec or expose font-engine internals.

The glyph-path identity cache retains that owner only when its complete pointer
and length match the borrowed view. It otherwise retains the old exact-byte
copy fallback. Exact byte equality, collection index, variation coordinates,
size and outline parameters remain unchanged. The old admission limit and
purge-before-release order are unchanged, including oversized fonts. No font
substitution, cache budget increase or image-quality reduction is involved.

Pinned Chromium `6c1d401fcca5e1b0030563a90c2f2bba168e0c15` selects Skia
`cacf77bdba7ba7df8ea7236d7e14b08c658ff368`. The source model is retained immutable
SkData backing, not names or non-owning pointer identity:

- [SkData stable data and external lifetime](https://github.com/google/skia/blob/cacf77bdba7ba7df8ea7236d7e14b08c658ff368/include/core/SkData.h#L21-L32)
- [Retained external data and release](https://github.com/google/skia/blob/cacf77bdba7ba7df8ea7236d7e14b08c658ff368/src/core/SkData.cpp#L36-L94)
- [Typeface retains SkData](https://github.com/google/skia/blob/cacf77bdba7ba7df8ea7236d7e14b08c658ff368/src/ports/SkTypeface_fontations.cpp#L137-L155)
- [Typeface variants retain that same data](https://github.com/google/skia/blob/cacf77bdba7ba7df8ea7236d7e14b08c658ff368/src/ports/SkTypeface_fontations.cpp#L987-L1037)

`font_identity_capacity_bytes` counts privately copied identity payloads;
`font_identity_shared_bytes` counts referenced immutable backing lengths and
overlaps provider/paint owners and other threads. Never add it as new storage.
The measured Linux glyph-path copy issue is distinct from the unchanged macOS
FontDataIdentity path and from other live allocator owners.
