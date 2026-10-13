# Integer grid repeat production integration

Pinned source: Chromium `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`.

## Cause and ownership

The production parser explicitly rejected `repeat(...)`, and its CSSValue dispatch
left the integer-repeat subclass uninhabited. The generic translated converter
already represented integer repeat, but the production `grid_application`
adapter admitted only plain tracks and line names. LayoutNG correctly used the
canonical native ComputedStyle; adding an alternative layout algorithm or
reparsing projected CSS text there would repair the wrong owner.

The production path is now:

- `ConsumeTracks` consumes a standalone integer repeat with a typed positive
  integer/count expression, track sizes and optional line names
- `CSSGridIntegerRepeatValue` retains that expression and the source parser's
  `kGridMaxTracks / repeated_track_count` clamp
- `TemplateTracks` resolves the count and track sizes with existing Style
  conversion context, then uses `AppendIntegerGridRepeat`
- Both the generic converter and production adapter share that native operation:
  repeated named-line indices/ordered flags, one compact integer repeater, and
  one copy of each repeated track size
- Existing canonical ComputedStyle, DOM native-style retention, LayoutObject,
  grid placement and track-sizing owners receive the result unchanged

Line-height dependency inspection traverses the new typed repeat (including its
count), and nested typed track functions, preserving native dependency flags.
Grid line names reject reserved `auto` and `span`, as Chromium's shared consumer
requires. Existing diagnostic tests use unsupported auto-repeat instead of the
newly supported integer form; six separately paused test files are untouched.

## Closed support slice

Standalone positive-integer repeat in template rows and columns, ordinary grid
shorthands, CSS variables and supported integer CSSMath, fixed/fractional/
intrinsic/minmax/fit-content track sizes, named-line expansion, inherited values,
media changes and native style/fragment snapshot ownership use existing engines.
Nested repeat and repeat in `grid-auto-rows/columns` are invalid. `auto-fill`,
`auto-fit` and subgrid remain explicit unsupported collaborators. This does not
claim all CSS Grid, general CSSOM serialization, or unrelated image rendering.

No LayoutNG, Paint, Raster, Renderer, public cross-crate interface, site selector,
site-sized offset, or hardcoded Bilibili dimension was added to production code.

## Evidence

The campaign evidence is outside the repository at
`rechrom-evidence/bilibili/grid-layout/`. It contains the source manifest, full
live CSS response, task-start files, fail-before logs, focused production tests,
unchanged paused-test hashes, Release page-protocol fixture images, real Bilibili
and Wikipedia captures, source/binary hashes and comparison limits.

Integer-repeat and expanded-track fixtures are compared without a tolerance.
Real-page cross-browser geometry is recorded separately from personalized
recommendations and animation state. A Chromium JPEG and a differently sized
Rechrom PNG do not qualify for a zero-pixel cross-browser claim.
