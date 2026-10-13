# Typed minmax tracks in the production Style pipeline

The real MDN homepage uses an extended named-column grid with
`minmax(0,48rem)` between flexible side tracks. The production parser formerly
reported this function as unsupported. After custom-property substitution,
the source StyleCascade consequently treated the entire track declaration as
unset. Layout received no explicit named track list, even though its named-line
resolver and native min/max sizing algorithm already existed.

## Owning path

1. The existing production grid parser consumes each `minmax()` argument with
   the same breadth consumer used by plain tracks
2. It retains a typed `CSSFunctionValue(kMinmax)` with two typed arguments
3. The production grid application converts those arguments through its
   existing typed length/math/font/viewport conversion
4. A small internal `GridTrackSizeInput` joins the production and generic
   StyleBuilderConverter paths at one native constructor dispatch
5. The existing `ComputedGridTrackList` retains native min/max sizes and named
   lines. The canonical ComputedStyle reaches Layout without string projection

There is no new placement, sizing, cascade, custom-property, or layout algorithm.
No package contract changes are needed. The helper and enum stay inside Style.

## Source semantics

Chromium revision: `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`.

- `css_parsing_utils.cc:6903-6923`: ConsumeGridBreadth
- `css_parsing_utils.cc:6997-7035`: ConsumeGridTrackSize
- `style_builder_converter.cc:123-143`: ConvertGridTrackBreadth
- `style_builder_converter.cc:1618-1635`: ConvertGridTrackSize

The parser uses RestoringBlockGuard, requires a comma and two complete
breadths, rejects a flex minimum including `0fr`, and accepts a flex maximum.
Nested track functions and negative literal breadths are invalid. CSS math
continues through the existing nonnegative range conversion, so a syntactically
valid negative computed calc is clamped by its established owner. The parser
does not normalize a maximum smaller than its minimum.

Existing native track-count limits and named-line indices are unchanged.
`repeat()`, `fit-content()` and `subgrid` parsing remain outside this slice.
They are still reported as unsupported; this is not complete CSS Grid support.

## Regression evidence

`browser/tests/named_grid_minmax.rs` exercises the real Page pipeline:
canonical typed tracks/names, centered and extended fragments, custom-property
math, narrow-container shrinkage, and repeated custom-property mutation while
retaining prior style and fragment snapshots.

`style/tests/production_grid_minmax.rs` covers all four grid track longhands,
shorthands, typed function arguments, intrinsic/percentage/flex/math breadths,
invalid grammar and the explicitly unsupported boundaries above.

At container width 1180, the generic five-track fixture resolves to
`[16,190,768,190,16]`; content-start is x206. At width 600 the middle track
shrinks to 568 and content-start is x16. These are native layout results.

The fresh live MDN capture places its header at x206 with width974; the previous
capture placed it at x6.390625 with width1173.609375. Chromium's normal cloud
reference has the same CSS viewport1180×757, DPR1, but a 15px scrollbar consumes
root content width, so its corresponding coordinates are x198.5 and width966.5.
That scrollbar difference is independent. Remaining row sizing, icons,
advertisement rendering, and other page differences are not fixed here.

The independent source review and full-source hashes are retained with the
task evidence. Original notices are in `third_party/NOTICE.chromium-grid-minmax`.
