# Local SVG mask resources

The native mask-image cascade now creates `StyleMaskSourceImage`. Locality is
fragment-only or a parsed URL equal to the committed document URL without its
fragment, matching Chromium `ElementStyleResources::LoadMaskSource`. Local
references keep their originating document/TreeScope and normalized fragment;
external sources retain a genuine fetched-image owner. The fetched subtype
flag is never set on a mask wrapper.

`<mask>` maps to the hidden `LayoutSVGResourceMasker`, retaining child layout
without ordinary paint/hit participation. The existing post-cascade SVG
resource recorder uses `native_paint_style::Export` for complete canonical
paint values and the existing native SVG geometry conversion. Its immutable
command records use the existing pattern recorder/replayer. The document's
first-connected-ID catalog publishes mask and clip graphs together; unchanged
consumers are invalidated by its existing revision and immutable snapshots.
There is no Paint callback into DOM or another network/cache owner.

Layout validates native resource attachment and supplies the resource viewport.
Paint resolves SVG region/content units before the bitmap mask path, and feeds
an immutable non-repeating resource paint record into the existing mask
compositor. CSS mask-position/size/repeat do not reinterpret SVG resource units.
Alpha, luminance, match-source and native mask-layer ordering are retained.
Paint-only updates preserve a viewport only for an unchanged scoped target.
Fetched SVG document replay shares LayerTile's validated formal mask-record
boundary, so its property effect owns the outer DstIn exactly once; ordinary
nested DstIn paint operations remain intact.

## Bounded support

- Local, scoped masks with solid authored paths or canonical fixed-length
  rectangles/circles/ellipses, plus ordinary groups
- Canonical fill/currentColor, fill/stroke opacity, fill-rule and ordinary fixed
  stroke metrics; group opacity and ordinary visible content
- User-space region/content units; object-bounding-box units for consumers with
  an established fill box; resource-region percentages use its native viewport
- Canonical CSS mask override/inheritance, duplicate-ID first-target behavior,
  missing/wrong-kind/empty targets, target/child mutations and immutable old frames
- Existing bitmap/fetched SVG-image mask lifecycle and compositing

Explicit unsupported collaborators include transformed or zoomed resource
content, percentage/expression child geometry or stroke metrics, CSS `d`,
non-scaling strokes, nondefault paint order, `pathLength`, hidden groups with
potential visibility overrides, no-own-box resource content, linearRGB mask
color interpolation, context paint,
text/images/use/foreignObject, paint servers and nested mask/clip/filter effects.
SVG group object-bounding-box masks remain unsupported until their native fill
bounds cross this boundary. External SVG fragment target selection is also
unsupported; ordinary fetched SVG images without a fragment retain their
working image path. The optional C++ raster oracle lacks pattern shader replay;
the production Rust replay is the verified path on the cloud platform.

Unsupported resources retain an explicit diagnostic and reject before culling;
they are never treated as successful empty masks. Missing/wrong-kind targets
instead retain valid transparent mask failure semantics. This is not complete
SVG mask or whole-page Bilibili fidelity support. The pre-existing compositor
rejection of pattern-shaded SVG shape draws also remains unchanged; canonical
pattern-content mask override is tested at its resource-command boundary.

## Source and checks

Chromium revision `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`:
`element_style_resources.cc`, `style_mask_source_image.{h,cc}`,
`svg_mask_element.cc`, `layout_svg_resource_masker.{h,cc}` and
`svg_mask_painter.cc`. See the accompanying third-party notice.

Production Page tests live in `crates/browser/tests/local_svg_masks.rs`, with
existing mask lifecycle, SVG mask graphics, hidden resource, clip lifecycle,
paint-layer ownership and SVG edge suites serving as regressions. Test results,
exact source/executable hashes, fail-before logs and page captures are recorded
in the task evidence rather than represented as a whole-workspace green claim.
