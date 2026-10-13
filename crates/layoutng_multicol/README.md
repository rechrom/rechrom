# Multi-column layout translation

This package owns the admitted `ColumnLayoutAlgorithm` path, translated from
Chromium revision `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`:

https://github.com/chromium/chromium/blob/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/layout/column_layout_algorithm.cc

The upstream copyright and BSD license are preserved in the source header and
`../../third_party/LICENSE.chromium`.

## Ownership and integration

`browser::CreateLayoutAssembly` installs the algorithm's existing layout and
intrinsic-sizing callbacks. Inputs remain canonical `ComputedStyle`, resident
LayoutObjects, and typed constraint spaces. Output is the existing immutable
fragment tree. No CSS parsing, DOM mutation, resource access, platform work,
Paint logic, or ordinary-block fallback lives here.

Chromium explicitly uses `BlockLayoutAlgorithm` to lay out each anonymous
`kColumnBox` fragmentainer, with the same multicol node, a column constraint
space and a continuation break token. That is also this package's path.
The outer multicol CSS box remains a distinct fragmentation-context root.

## Admitted source branches

- A single non-nested line of columns, with inline-direction overflow
- Used count/width, normal/length/percentage gaps and subpixel progression
- `column-fill:balance` content-run measurement, forced breaks, implicit-break
  distribution, minimum-shortage stretching and min/max block constraints
- `column-fill:auto`, including unconstrained block size
- Anonymous column fragments and block/inline continuation through the existing
  block and inline algorithms; logical coordinates and baseline propagation
- Intrinsic inline sizing through the upstream anonymous-column measurement path
- Existing native style and retained-tree invalidation paths

The shared physical-fragment translation must use the self-or-container owner
when upstream reads `layout_object_`. `GetLayoutObject()` intentionally stays
null for anonymous columns, so they do not acquire a DOM identity or duplicate
CSS decoration in the published tree.

## Explicit remaining gaps

The algorithm raises the existing typed `UnsupportedLayout` capability error
for nested/outer fragmentation, actual column spanners, wrapping or explicit
column-height, gap decorations, fragmented out-of-flow descendants, a list
marker belonging to the multicol container itself, table-cell finalization,
and column pseudo-elements/scroll markers. It does not silently claim those
branches work. Ordinary list items *inside* columns still use list layout.

Completing the upstream remainder needs its MulticolPartWalker,
LayoutSpanner/outer-fragmentation and wrapping paths, ColumnGapAccumulator,
OOF balancing and empty-column collaborator (owned with the translated
SimplifiedOofLayoutAlgorithm), marker placement, table-cell finalization, and
a typed DOM/pseudo-element lifecycle handoff rather than DOM writes in Layout.

The existing display-lock check in PhysicalBoxFragment::CheckIntegrity still
uses its pre-existing layout-lock adapter. Upstream's paint-lock provider is
unlinked here; that distinct limitation is not replaced with a fabricated value.
