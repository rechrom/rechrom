# SVG text visual bounds

SVG text is shaped and positioned before Paint. A textPath/rotate transform can
move a glyph from its baseline box outside an overflow clip to a visible location
inside it. The pre-transform inline rectangle is therefore not a paint cull or
recording bound.

The source-aligned ownership is:

- LayoutSVGText and LayoutSVGInline union native FragmentItem::ObjectBoundingBox
  values, which retain floating-point transformed glyph geometry and remove the
  font scale exactly once
- SVG Layout extends that parent bound for native stroke and text shadows. The
  existing ShadowData/ShadowList owners compute their previously missing pure
  outsets methods; no parsing, filesystem, platform or Paint work is added
- The existing SvgLayoutSupport assembly record exposes a pure visual-rectangle
  query. The Layout boundary invokes it on the SVGInlineText parent and carries
  one immutable fragment-relative rectangle in SvgTextPaintData
- Paint consumes the same rectangle for DrawingRecorder, foreground visibility
  and shadow visibility. Ordinary text retains its ink-overflow path; SVG glyph
  transforms remain record-local, and the clip/culling mechanisms remain active

The rectangle is computed from the completed native fragment snapshot, after
inline associations are finalized. No new retained cache or invalidation owner
is introduced. This can perform repeated parent unions for long text; it is a
correctness repair, not a performance claim. Any future caching must preserve
native bounds invalidation instead of caching at the Paint boundary.

## Verification

The production Page regression starts with a generic circle whose baseline lies
outside a clip while dy moves its glyphs inward. Before the repair, clipping
removes 147 pixels that are inside the clip. Afterward, all in-clip pixels are
identical to the non-clipped control. The control is required to contain ink.

`browser/tests/svg_text_path_geometry.rs` also covers direct rotated SVG text,
tspan and textPath parent scopes, fractional viewBox/font scale, spacing and
spacingAndGlyphs, strokes, shadows, percentage stroke visual bounds, retained dy
mutation, old immutable artifact pixels, and native shadow extent rounding,
negative spread, original-rectangle inclusion and inset exclusion.

The source is Chromium revision
`6c1d401fcca5e1b0030563a90c2f2bba168e0c15`: core/layout/svg/layout_svg_inline.cc,
layout_svg_text.cc, svg_layout_support.cc; core/layout/inline/fragment_item.cc;
core/paint/text_fragment_painter.cc; core/style/shadow_data.cc and shadow_list.cc.
Original notices and source URLs: `third_party/NOTICE.chromium-svg-text-bounds`.

The real MDN homepage additionally verifies integration. Its mandala is static,
uses Inter 300 and a 560px SVG/viewBox575, and retains its original SVG content.
No site-specific offset, glyph substitution, image replacement or renderer
algorithm was introduced. The normal Chromium capture is scaled JPEG, so it
cannot establish strict cross-browser zero-pixel parity. SVG DOM bounding-rect
projection, fonts beyond the tested assets, unrelated ad geometry and below-fold
coverage remain separate from this paint-bounds repair.
