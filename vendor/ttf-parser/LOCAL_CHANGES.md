Local change to ttf-parser 0.25.1
================================
GPOS PairSet retains the full PairSet byte slice when resolving Device and
VariationIndex offsets. Previously the slice excluded the count and all bytes
after the pair records, losing valid variable-font kerning adjustments.

Reference: layoutng/src/harfbuzz/OT/Layout/GPOS/PairSet.hh (read-only oracle).
Upstream license files are retained. A synthetic PairSet regression and native
HarfBuzz comparisons cover this change.
