# Grid placement shorthand omissions

The production Style parser expands grid-area, grid-row and grid-column
through one typed shorthand branch. Bare custom identifiers must survive
omitted components: grid-area copies row-start into omitted column-start and
row-end, then column-start into omitted column-end. Axis shorthands copy start
to omitted end. An explicit component is never changed; a numeric value,
number-plus-name list, or span list implies auto when its end is omitted.
All expanded longhands remain non-implicit, as in Chromium.

## Source ownership

Pinned Chromium revision: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
GridArea::ParseShorthand (shorthands_custom.cc:3613-3683) and
ConsumeGridItemPositionShorthand (css_parsing_utils.cc:7578-7606) are the
source operations. Attribution is in third_party/NOTICE.chromium-grid-placement.

The defect was in the existing Style parser, before layout. The named area map
already stores repeated rectangular areas correctly. grid_application::Position
converts the typed CSSCustomIdentValue to the canonical native GridPosition;
GridPlacement and GridLineResolver resolve its implicit area start/end lines.
The unchanged native track-sizing algorithm then receives the correct span.
There is no new layout solver, public interface, CSSOM projection, site-specific
height/offset, or app-layer compensation. Existing track counts, caps and
placement constraints remain unchanged.

## Validation and boundaries

production_grid_placement covers 39 shorthand combinations plus authoritative
native positions after variable substitution. spanning_grid_rows covers named
versus explicit spans, both axes, automatic/min-content/minmax rows, and four
whole-style-attribute mutations while retaining old style/fragment snapshots.
The original named-row fixture and three Style regressions fail before this
repair and pass after it.

The style.setProperty path has a separately diagnosed pre-existing failure:
reparsing and serializing a pending var shorthand through expanded strings can
discard it as empty longhands. That CSSOM owner requires a separate typed
property-store/serializer repair. Whole-attribute mutation tests do not claim
that setProperty is fixed. Reordered grid-line grammar such as zone 2, as well
as existing repeat/fit-content/subgrid limitations, remain outside this slice.
