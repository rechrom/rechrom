# SVG viewport translation

The nested viewport path is translated from Chromium
`6c1d401fcca5e1b0030563a90c2f2bba168e0c15`:

- `core/svg/svg_element.cc` and `svg_svg_element.cc`: direct-parent outermost
  identity, including the `foreignObject` boundary
- `core/layout/svg/layout_svg_viewport_container.{h,cc}`: separate viewport
  identity; attribute lengths; local transform × translation × viewBox
- `core/layout/svg/svg_content_container.cc`: independent SVG child updates
- `core/svg/svg_length_functions.cc`: nearest ancestor viewport resolution
- `core/svg/svg_preserve_aspect_ratio.cc`: viewBox transform
- `core/layout/svg/transform_helper.cc`: canonical ComputedStyle transforms
- `core/paint/paint_property_tree_builder.cc`: inverse-viewBox viewport clip

## Ownership and interface adaptation

DOM carries typed authored viewport lengths, with no computed geometry.
`layoutng_svg` owns resolution, transforms and SVG child traversal. The existing
LayoutBlockFlow-compatible storage is retained to fit the resident LayoutObject
and immutable-fragment ABI; groups and nested viewports do **not** invoke the
CSS block layout algorithm. Their children retain their own physical fragments,
like the SVG root. The boundary exports these once into the immutable tree.

Paint receives resolved affine matrices, viewport clips and viewport reference
sizes. Its existing property-tree and raster replay path handles them. Paint no
longer computes the root viewBox algorithm independently. ForeignObject retains
its distinct CSS child-layout boundary. Nested SVG clips do not create CSS
scroll containers or CSS box decoration. Transform style changes invalidate SVG
layout; metadata changes and published revisions retain the existing owners.
No I/O, parser, renderer or browser ownership moved into an engine.

## Implemented range

- Outermost/nested identity, including groups and foreignObject
- x/y/width/height defaults, finite user units, percentages and CSS absolute units
- Nearest viewport/viewBox percentage references, including no-viewBox viewports
- none/meet/slice viewBox mapping and alignment
- Ordinary canonical CSS transforms, independent transforms and transform origin
  with the view-box reference (motion-path excluded)
- Overflow hidden/clip/scroll clipping; auto and visible do not clip; no scrolling
- Zero viewport and valid-empty viewBox paint suppression
- Retained geometry, transform, clipping and parent-identity mutations
- Root zoom/reference integration and native SVG text transform query wiring

## Explicit unsupported branches

Typed UnsupportedLayout errors report contextual/calculated/invalid viewport
lengths, lengths outside the admitted finite LayoutUnit range, CSS motion-path
transforms and transformed SVG container fill-box/stroke-box bounds. These
branches are not silently treated as ordinary CSS boxes or generic groups.

This is not a full SVG engine translation. Existing source gaps remain in
container object/stroke/decorated bounds and client-box queries, generic shape
percentage geometry, marker/resource-bound dependencies, SMIL motion and
use/symbol/switch-specific semantics. Incremental viewport dependence is
conservative within the existing root's forced SVG update cycle. The existing
root aspect-ratio sizing also quantizes some dimensions; that is independent of
nested viewport resolution. Do not infer full SVG or pixel parity from this
bounded implementation.

Tests: `cargo test -p browser --test nested_svg_viewport -- --test-threads=1`.
Source-specific license notices are retained under `third_party/`.
