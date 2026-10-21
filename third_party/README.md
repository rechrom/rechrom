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
in `crates/layoutng/layoutng_foundation/base/README.md`.

The multi-column algorithm in `crates/layoutng/layoutng_multicol` and anonymous-column
owner accessor corrections in `crates/layoutng/layoutng_fragment_tree` are translated
from Chromium revision `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`:

- https://github.com/chromium/chromium/blob/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/layout/column_layout_algorithm.cc
- https://github.com/chromium/chromium/blob/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/layout/physical_box_fragment.cc
- https://github.com/chromium/chromium/blob/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/layout/physical_box_fragment.h

The admitted branches and explicit remaining translation gaps are recorded in
`crates/layoutng/layoutng_multicol/README.md`. The Chromium BSD license above applies to
these translated source portions as well.

## Nested SVG viewports

The nested SVG viewport producer, SVG-container traversal, viewport length
resolution, viewBox transforms and paint geometry integration in
`crates/layoutng/layoutng_svg`, `crates/dom`, `crates/layoutng/layoutng`,
`crates/layoutng/layoutng_assembly`, `crates/layoutng/layoutng_fragment_tree` and `crates/paint`
reference Chromium revision `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`.
`NOTICE.chromium-svg` retains the original copyright and license notices,
with a pinned official URL for each referenced source file.

These Blink SVG sources contain both Chromium BSD-style and GNU Library
General Public License notices; the root BSD license is not a replacement
for the source-file-specific terms. The complete, unchanged license texts
below were fetched from that same official Chromium revision:

- `LICENSE.chromium`: the existing file is byte-identical to the pinned root
  [LICENSE](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/LICENSE)
- [LICENSE-LGPL-2](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/LICENSE-LGPL-2):
  SHA-256 `7555fa34bc131a75ca56d65c40cc1ea8f9515d23e353d4c15d58573a042f7805`
- [LICENSE-LGPL-2.1](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/LICENSE-LGPL-2.1):
  SHA-256 `f2b3bd09663381deb99721109d22b47af1213bb43007a8b56a06c6375c8050ce`
- [LICENSE-APPLE](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/LICENSE-APPLE):
  SHA-256 `0b5d3a7cc325942567373b0ecd757d07c132e0ebd7c97bfc63f7e1a76094edb4`

This attribution preserves identified upstream notices and texts. It does
not relicense the rest of Rechrom or assert completion of every possible
redistribution requirement.

## SVG textPath identity and admission

The textPath DOM classification and native-parent admission adapter follows the
same pinned Chromium revision. See `docs/svg-text-path-identity.md` for source
links, ownership and supported boundaries. `NOTICE.chromium-svg-text-path`
preserves original source-specific copyright and license notices; the existing
Chromium and Blink license texts above remain applicable.

## Fetched mask-image resource lifecycle

The source-aligned terminal resource-state publication, typed mask projection,
Paint effect suppression, and native-slot compositing rules follow Chromium
revision `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`. See
`docs/mask-image-lifecycle.md` for ownership and scope and
`NOTICE.chromium-mask-lifecycle` for original source-specific notices and URLs.
The Chromium/Blink license texts already preserved above remain applicable.

## SVG consecutive moves and empty paths

The SVG geometry producer and Paint validity repair follow Chromium revision
`6c1d401fcca5e1b0030563a90c2f2bba168e0c15` and its DEPS-pinned Skia revision
`cacf77bdba7ba7df8ea7236d7e14b08c658ff368`. See
`docs/svg-empty-contour-semantics.md` and
`NOTICE.chromium-svg-empty-contours`. Source-specific notices are preserved;
the Skia BSD license is retained unchanged in `crates/skia/LICENSE`.

## DOM template primitives

The native attribute-list bindings and destination-owned import/clone connection
follow Chromium revision `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`.
See `docs/dom-template-primitives.md` for semantic boundaries and
`NOTICE.chromium-dom-template-primitives` for pinned source-specific notices.
The existing Chromium and Blink license texts remain applicable.

## Native HTML slot bindings

`NOTICE.chromium-html-slot-bindings` records pinned Blink IDL, constructor,
assignment and flattening source attribution for the native slot binding slice.
See `docs/html-slot-bindings.md` for the owner contract and explicit limits.

## Native custom-element definitions and reactions

The native registry, definition metadata, element state and scoped reaction
queues, together with their host constructor/callback bindings, follow Chromium
revision `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`. See
`docs/custom-element-definitions.md` for ownership, source links, verification
status and explicit unsupported boundaries.

`NOTICE.chromium-custom-elements` preserves the original source-specific
notices and pinned official URLs. The existing `LICENSE.chromium`,
`LICENSE-LGPL-2` and `LICENSE-LGPL-2.1` texts remain applicable; the Chromium
BSD license does not replace the LGPL notices on the DOM integration sources.

## Ordinary DOM property descriptors

The ordinary DOM-node descriptor and declaring-interface native receiver bridge
follows Chromium revision `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`.
See `docs/dom-property-descriptors.md` for ownership, source links and explicit
remaining boundaries. `NOTICE.chromium-dom-property-descriptors` preserves the
original source-specific notices, pinned URLs and source SHA-256 hashes.

The existing `LICENSE.chromium`, `LICENSE-LGPL-2` and `LICENSE-LGPL-2.1` texts
remain applicable. Chromium's root BSD license does not replace the LGPL terms
on the referenced DOM/HTML IDL files. Existing QuickJS license and attribution
remain unchanged; this binding slice reuses its existing language semantics.

## HTML hidden presentation hint

The native production style resolver's HTML hidden-attribute hint follows
Chromium revision `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`. See
`docs/html-hidden-presentation-hint.md` and `NOTICE.chromium-html-hidden`
for the source owner, pinned attribution, tests and explicit boundaries.
Existing Chromium/Blink license texts remain applicable; source-specific
LGPL notices are preserved and not replaced by Chromium's root BSD license.

## Composed event paths and dispatch lifetime

The DOM-owned slot-aware event-path snapshot, tree-scope target retargeting,
closed-root path filtering and interaction/WebAPI dispatch lifetime follow
Chromium revision `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`. See
`docs/event-composed-path.md` for ownership and explicit supported boundaries.
`NOTICE.chromium-event-path` preserves the original source-specific copyright
and license headers, pinned official URLs and source SHA-256 hashes.

The existing Chromium and Blink license texts remain applicable. The Chromium
root BSD license does not replace GNU Library GPL notices on the referenced
Event, EventDispatcher and DOM integration sources.

## Root scrollbars and viewport geometry

The root scrollbar ownership, overflow propagation, native client geometry and
viewport invalidation integration follows Chromium revision
`6c1d401fcca5e1b0030563a90c2f2bba168e0c15`. See
`docs/root-scrollbar-viewport.md` and `NOTICE.chromium-root-scrollbars` for the
existing typed host boundary, source owners, original notices and limitations.
Source-specific LGPL and Apple notices remain applicable; the Chromium root
BSD license does not replace those terms.

## Connected shadow image resources

The resource admission and completion-event adapter includes shadow consumers
following Chromium revision `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`.
See `docs/shadow-image-resources.md` for the typed ownership boundary, verified
behavior and explicit limits. `NOTICE.chromium-shadow-images` preserves the
pinned official URLs, hashes and original source-specific copyright/license
notices. Existing Chromium, LGPL and Apple terms remain applicable.

## SVG text visual bounds

The SVG parent bounds, native shadow extents and SVG paint-bounds projection follow Chromium revision `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`. See `NOTICE.chromium-svg-text-bounds` for original source headers, URLs and license attribution, and `../docs/svg-text-visual-bounds.md` for ownership and regression coverage.

## Named font alternates

Native feature-value parsing, the shared scoped alias registry, immutable
per-requested-family font resolution, active stylesheet ordering and reshape
invalidation follow Chromium revision
`6c1d401fcca5e1b0030563a90c2f2bba168e0c15`.
See `docs/named-font-alternates.md` and `NOTICE.chromium-font-alternates` for
ownership, verified behavior, explicit limits and source-specific notices.
The existing BSD, LGPL and Apple license texts remain applicable.
