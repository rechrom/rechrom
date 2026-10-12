# FontEngine boundary

`font_engine::FontEngine` owns platform-neutral font selection and retained
resolution. `Shape` takes complete UTF-16 context, an output range, the full
computed `FontDescription`, direction, optional analyzed segment ranges, and a
font-context version. `GetMetrics` uses the same resolver. Both return
`FontCompute::{Ready, Pending, Error}`. Returned shape results retain their
actual glyph font data through cache invalidation.

The font description carries locale, orientation, features, variation axes,
size adjustment, and spacing. Inline remains responsible for building text
items, bidi analysis, line breaking and reshaping at line boundaries. Existing
translated inline code keeps its Font/ShapeResult adapters; it does not adopt
an asynchronous layout algorithm.

## Injected platform capability

The embedding constructs `SystemFontProvider`. Its typed protocol distinguishes
family, generic and unique-name lookup, contextual fallback with locale and
presentation, and opening an exact opaque face ID. Metadata lookup does not
parse a system font catalog. Opening a selected face returns immutable bytes,
collection index and explicit variation axes. FontEngine never calls the
provider or performs OS/network I/O from shaping.

The host returns `SystemFontResponse` through `ApplyMutation`. Responses must
match a pending request and its context version. Positive and negative results
are retained; stale or unsolicited responses are rejected. Provider failures
remain distinct from negative answers and retain the pending request for an
explicit retry. Contextual system lookup starts when the iterator exhausts
explicit families, using the complete unresolved shaping slice. Its admitted
candidates are restricted to that exact text/locale/style/presentation/base-face
query and retain platform order. An already-open first candidate is shaped
before another is opened; only a genuine miss advances the candidate prefix.
CSS aliases are kept separately from the exact opened face identity.

Web-font downloads stay in ResourceEngine, and decoding/sanitization stays in
DecodeEngine. FontEngine's typed rule/state input contains no DOM or CSSOM.
The embedding remains responsible for mapping document font-display policy to
ready/loading/failed rules; this change does not implement a new font-display
scheduler.

## Browser assembly

Browser constraints initially carry one embedded emergency face and a pure
font-query outbox, rather than a parsed installed-font snapshot. Missing family
lookups and contextual system-fallback stages append queries. Grapheme
boundaries keep joiners and combining sequences with their unresolved bases;
primary-font invisible glyphs cannot prematurely split a fallback unit. Page performs a
private Layout pass, drains and answers requests outside compute, admits new
immutable faces, invalidates the existing font-sensitive layout cache, and
retries. Only the settled pass reaches container-query publication, measurement,
Paint or PageClient. Negative/previously answered requests do not repeat I/O or
trigger an unchanged retry. The browser's platform service is retained across
passes. Embeddings can inject their own service with
`Page::SetSystemFontProvider`; `Page::SystemFontsChanged` publishes a fresh
provider/query context and invalidates font-dependent layout without mutating
old frame resources. Exact query associations carry a semantic revision,
so a changed answer invalidates selection even when it reuses already-open bytes.

The resolver initializes only selected primary/fallback faces. Fonts retain
lazy fallback sources and preserve the original snapshot index for paint.
Paint and RustSkia use exact bytes, collection index and canonicalized variation
coordinates. The macOS path no longer rematches a CSS/native family string in
place of the shaped face. No C++Skia backend is enabled by this boundary.
Embedded bitmap-only glyphs
(PNG, premultiplied BGRA and grayscale strikes) use the RustSkia glyph cache and
image composition path instead of being incorrectly required to supply vector
outlines. This does not add COLR or SVG font rendering.

## Chromium reference and limits

Fallback iterator and missing-cluster shaping queue were compared with Blink
revision `78ef49f74feba3d0dde852881feef92bbf86aad7`, especially
`platform/fonts/font_fallback_iterator.cc`,
`platform/fonts/shaping/harfbuzz_shaper.cc`, and
`platform/text/character_variation_sequences.cc`.

This is not a claim of complete Chromium parity. The shaping backend remains
rustybuzz, with the repository's existing native HarfBuzz/ICU support. Features
not already translated, font-display timing, platform-specific locale mapping,
and all platform edge cases need separate source-aligned work. Chromium oracle
execution was not available, so pixel equality with Chromium is unverified.
Linux tests do not establish macOS runtime correctness.
