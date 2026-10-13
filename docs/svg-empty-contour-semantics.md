# SVG consecutive moves and empty-path validity

## Source contract

Chromium is pinned to `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`.
Its DEPS pins Skia to `cacf77bdba7ba7df8ea7236d7e14b08c658ff368`.

- Blink's [SVGPathBuilder](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/svg/svg_path_builder.cc)
  resolves a relative moveto against the current point before forwarding the
  absolute move, and updates the current and subpath-start points
- [SkPathBuilder::moveTo](https://skia.googlesource.com/skia/+/cacf77bdba7ba7df8ea7236d7e14b08c658ff368/src/core/SkPathBuilder.cpp#158)
  replaces the last point when the immediately preceding verb is also a move.
  It appends a new contour after any other verb, including close
- [PathTest](https://skia.googlesource.com/skia/+/cacf77bdba7ba7df8ea7236d7e14b08c658ff368/tests/PathTest.cpp#3052)
  retains move-only paths as valid and verifies consecutive-move replacement
- [SkStroke](https://skia.googlesource.com/skia/+/cacf77bdba7ba7df8ea7236d7e14b08c658ff368/src/core/SkStroke.cpp#1493)
  distinguishes an unadorned move from move+close and zero-length drawing
  segments. The latter retain round/square caps

The concrete failure was a legal `M ... m ... A ...` path. DOM's parser
published both moves into retained commands and its flattened geometry,
incorrectly marked the result as multiple contours, and made textPath
resolution reject it. Removing the single-contour guard would have introduced
a fictitious line between the two move positions.

## Ownership and repair

The DOM SVG geometry producer now performs the same immediately-consecutive
move replacement as Skia. It updates the retained command and the existing
flattened projection together. Relative-coordinate resolution, smooth-control
reset, and subpath-start/close state keep their existing owner. A real second
contour is preserved. No consumer independently reconstructs or normalizes
the path, and no line is inserted.

Paint no longer treats a valid move-only path as an invalid input. Its existing
finite-coordinate, first-move, transform, bounds and Style validation remain.
The existing raster replay produces no pixels for a lone move. Paint does not
drop closed or zero-length paths, since their stroke caps can be visible.

There is no new public interface, core host I/O, renderer, resource cache,
cross-engine callback, element-hiding branch or site-specific condition.
The existing typed shape/text-path data and immutable artifact lifecycle remain
unchanged. The native Skia builder already had the correct move replacement;
the defect was in the upstream geometry producer and Paint validity contract.

## Executable regression coverage

- `dom/tests/svg_path_parser.rs`: absolute/relative repeated moves, line,
  quadratic/cubic/conic curves, the exact MDN circle construction, bounds,
  move-only paths, closed contours and genuine separate contours
- `browser/tests/svg_text_path_identity.rs`: resolved text geometry and actual
  pixels against normalized paths, percentage offsets, visible glyphs,
  filled/stroked curves and close, all three wide-stroke caps, legal move-only
  fill/stroke, and move+close/zero-length-line cap preservation
- Retained referenced-path mutations keep the native textPath object, update
  glyph visibility/position, and preserve previously captured artifacts

The original three new production Page tests fail against the preceding
production sources and pass after the two-file repair. These equal-pixel
comparisons are same-engine regression checks, not Chromium page parity.

## Explicit remaining boundaries

TextPath still uses the existing flattened single-contour mapper. General
multi-contour native measurement, native curve-length accuracy, referenced
transforms and author pathLength are not implemented by this change. Explicit
move+close followed by another contour remains a separate contour, rather than
being collapsed to work around that limitation. The canonical measurement
iterator and hairline-stroke zero-length behavior have separate source-audited
gaps; neither implementation is changed here.

Full MDN visual parity is not established. A matched lossless Chromium protocol
capture is still unavailable, and other live layout/script differences remain.
See `third_party/NOTICE.chromium-svg-empty-contours` for source notices.
