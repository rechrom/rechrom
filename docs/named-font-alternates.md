# Named font alternates

This slice connects the existing Chromium-translated `@font-feature-values`
parser and RuleSet owners to the production Style/FontEngine boundary. Its
reference is Blink revision `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`.

## Ownership and immutable inputs

Style retains native feature-value rules and builds the shared
`FontFeatureValuesRegistry` from active document-author RuleSets. Owner media is
filtered before collection; nested media stays in RuleSet. The registry reuses
`FontFeatureValuesStorage` and the canonical `CascadeLayerMap`, merging
individual aliases by normal layer priority and then source order. Family keys
use full ICU Unicode folding; alias names remain case-sensitive. Both the
translated ScopedStyleResolver and production DocumentStyleEngine use this
same registry.

The canonical FontDescription keeps the symbolic font-variant-alternates value.
Style resolves an immutable feature vector for each requested CSS family and
stages that additional typed value in the description. It does not select faces
or call Layout's font resolver. FontEngine carries requested CSS-family identity
alongside each selected face, preserving that identity through web-font aliases,
explicit family fallback and unicode-range faces. Platform/character fallback
never acquires an alias table from the selected file's internal family name.
Historical forms resolve even without an alias table. Shaping consumes selected
face features before the existing explicit element feature vector, preserving
explicit `font-feature-settings` override priority.

Description equality includes the family vectors and semantic alternate value.
Old FontDescriptions, Fonts, ShapeResults and frames retain their original
snapshots. No cache invalidation depends on a fresh Arc allocation alone.

## Lifecycle

Alias insertion/removal/edit, sheet order, layer order and media changes rebuild
and compare the effective registry, invalidate matched-style reuse, and visit
canonical styles even when no selector changed. The existing typed Style impact and Layout-equivalence boundary
project native `kReshape` to layout and paint, matching
ComputedStyle::VisualInvalidationDiff. This covers font-metric changes that are
absent from the smaller legacy family/size/weight projection.

DOM owns the active stylesheet ordering supplied to Style. Connected style/link
owners follow tree order; explicit ownerless sheets are the injected-author
prefix and adopted sheets preserve their requested suffix order.

## Explicit limits

At this Chromium pin, feature-value tables are collected only from
document-author sheets. Shadow-tree table fusion and user-sheet feature-value
support remain upstream TODOs; shadow text still uses the document table. No
new shadow/user alias-scoping model is invented here.

The existing initial/empty-family path uses the host default face without
claiming its internal family as CSS alias identity. The separately incomplete
preferred-standard `-webkit-standard` fallback stage and FontBuilder settings
substitution are not introduced by this slice.

`@font-face font-feature-settings` descriptor defaults are a separate incomplete
boundary and are not implemented by this slice. Element feature-setting
precedence is covered. Direct mutation of a stale CSSOM projection cannot
supply native feature-value rules: authoritative reparsed backing/revision is
required, and the existing stale-source guard remains intact.

FontEngine and Style remain pure compute. Resource download, font decoding,
platform discovery and file I/O remain outside this boundary. No alternative
shaper, font substitution, site-specific alias, pixel offset, or width correction
is added.

## Verification

Portable regressions cover parser recovery and native retention, all six alias
types, layer/source ordering, Unicode family folding and alias case sensitivity,
per-family fallback/unicode ranges, explicit overrides, stylesheet mutations,
retained snapshot identity and synthetic GSUB glyph/advance changes. An external
real-Inter Page diagnostic exercises the exact MDN hero/ad geometry without
checking a downloaded site font or absolute-path fixture into the repository.
Cross-browser screenshot claims require matched page states and raw pixels;
a JPEG browser reference cannot establish strict zero-pixel equality.

See `../third_party/NOTICE.chromium-font-alternates` for original source notices
and pinned URLs.
