# SVG textPath identity and unresolved resources

The DOM-to-Layout boundary classifies structurally eligible SVG `textPath`
elements as `kSvgTextPath` regardless of whether referenced path geometry is
currently available. `ElementData::svg_text_path` is optional resource state,
not an object-class discriminator. Native metadata validates supplied geometry
but permits its absence. The existing LayoutSVGTextPath mapper returns `None`
for unavailable geometry; SVG text layout hides those characters.

Both DOM adapters perform textPath-specific admission before construction:
its immediate DOM parent must be SVG `text` or `a`. The actual native parent
must be SVGText, or an SVG inline anchor whose effective parent admits the
child. TSpan, TextPath, group, root, foreignObject and HTML parents reject it.
This preserves the source eligibility and child-admission rules without relaxing
SVG container assertions. Flattened `display:contents` nodes are handled using
the actual native parent. This does not replace the general layout admission
mechanism or repair all SVG anchor classification rules.

Reference changes and target-geometry changes are existing metadata mutations.
They retain the compatible native textPath object, invalidate its geometry,
and publish new immutable fragments/paint artifacts. The owning engines keep
their pure typed inputs and outputs. No new I/O, DOM reachback, Page geometry,
CSS projection, raster workaround, resource cache or site branch is introduced.

## Chromium provenance

Revision: `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`

- [SVGTextPathElement](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/svg/svg_text_path_element.cc#129): construction, DOM eligibility, independent resource invalidation
- [LayoutSVGTextPath](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/layout/svg/layout_svg_text_path.cc#69): child policy and nullable path mapping
- [LayoutSVGInline](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/layout/svg/layout_svg_inline.cc#43): anchor delegates child admission to its parent
- [SVG text positioning](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/layout/svg/svg_text_layout_algorithm.cc#626): a null mapper hides glyphs

## Regression coverage

`browser/tests/svg_text_path_identity.rs` exercises production Page lifecycle:
missing/empty/move-only/unavailable-multicontour/supported-line resources;
hidden pixels versus an empty text control; valid and invalid structural
parents; href and referenced-path mutation; retained native identity; immutable
old-frame pixels. Reference shapes are non-rendering to isolate textPath
lifecycle from shape painting.

## Bounds

The existing SVG path adapter accepts only a flattened single contour. An
explicit initial move followed by a relative move (as used by MDN's mandala)
currently becomes unsupported even when the first contour is empty. Removing
the single-subpath guard would wrongly join the two points. This repair keeps
textPath identity but does not claim to display that valid path correctly.
Native contour, pathLength, transform and broader reference semantics remain
separate work. Empty SVG inline line-breaking and move-only shape painting also
remain separate observed failure paths.
