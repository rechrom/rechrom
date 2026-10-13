# Document-owned selector service

## Chromium comparison

Reference revision: `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`.
The inspected local source files match the SHA-256 values recorded when they
were retrieved from Chromium's official source mirror.

- [`core/css/style_engine.cc`, `StyleEngine::StyleEngine`, lines 251–264](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/css/style_engine.cc#251)
  creates and retains the document's `StyleResolver`; `ClearResolvers`, lines
  1042–1055, disposes it at the owning lifecycle boundary.
- [`core/css/resolver/style_resolver.cc`, `StyleResolver::ResolveStyle`, lines 1378–1404](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/css/resolver/style_resolver.cc#1378)
  creates per-element `StyleResolverState` and `StyleCascade`. The collector
  created at lines 1782–1785 consumes the retained resolver's `selector_filter_`.
- [`core/css/style_engine.cc`, `ApplyRuleSetInvalidationForSubtree`, lines 2580–2605](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/css/style_engine.cc#2580)
  constructs a stack-local `StyleScopeFrame`; lines 2844–2864 similarly scope
  selector filtering, scope activation and nth-index state to an invalidation
  operation. They are not permanent caches of mutable DOM matching results.

Source file hashes:

- `style_engine.cc`: `b04bcf323ee2415510d0fece9a11fa260f3b9745636d2b0b4ac9adeb1da49186`
- `style_resolver.cc`: `415d2c03f697b1afb1cafc56ab10b502eaa2fe8b4d6c3de4947e2c89d8b8eda2`

## Integration repair

The production adapter previously created a `PersistentSelectorService` for
every resolved element. Its first session interns a handle for every document
node, causing O(resolved elements × document nodes) handle-allocation work.
The document style engine already owned the service required by this path.

`DocumentStyleEngine` now shares that existing owner through the package-private
`ProductionBaseStyleCollaborators` for ordinary and highlight resolution. The
compatibility collectors use the same owner. Standalone resolver callers retain
their local default owner. No public cross-package contract or matching algorithm
changes, and no second cache is introduced.

Only existing arena-index handles and the parsed-selector cache persist. Each
`StyleMatchSession` still creates a new backend with current document and
interaction inputs, fresh atom/match state, and operation-local scope frames.
Document identity/compatibility-mode checks and DOM/style invalidation remain
unchanged. This removes the quadratic handle setup, not the inherent cost of
selector matching, cascade or layout.

## Verification

- Passed: `shared_selector_owner_reuses_handles_after_growth_mutation_and_interaction`
  checks pointer reuse, appended nodes, class/id changes, reparenting, `:has`,
  `:first-child`, hover and focus transitions
- Passed: `shared_selector_sessions_refresh_scope_and_shadow_tree_state`
  checks changed scope roots, reparenting across tree scopes, newly attached
  shadow roots and changed shadow stylesheet text
- Passed:
  `production_element_and_pseudo_hosts_share_selector_handles_but_not_sessions`
  checks exact service/handle sharing and fresh backend identities for ordinary
  and pseudo-element hosts; standalone callers remain independent
- Full Style unit suite: **657 passed, 14 failed**. All 14 failure names match
  the saved baseline; there are no new failures. This is not an all-green suite
- Both compositor provenance tests and the offscreen publication test passed

Three fresh-process Release measurements before and after, per real site, used
the same 1180 × 757 CSS viewport at DPR 1. The medians are:

| Measurement | MDN before → after | Wikipedia before → after |
| --- | --- | --- |
| Initial Style | 672.881 → 361.561 ms | 1363.486 → 655.876 ms |
| Response commit → first offscreen content pixels | 1882.574 → 1485.934 ms | 3487.148 → 2850.012 ms |
| Navigation → first offscreen content pixels | 6630.997 → 5204.847 ms | 6310.505 → 5984.451 ms |

The initial Style comparisons resolved the same work: MDN had 2414 document
nodes / 749 resolved elements; Wikipedia had 2755 / 1194. The full navigation
interval includes variable HTTP response time, so its entire reduction cannot
be attributed to selector ownership.

Before/after `Page.captureScreenshot` comparisons with `tools/pixel_diff.py`
reported **zero changed pixels** for MDN's matched-state run 3 pair and all three
Wikipedia pairs. MDN's other pairs differed by 999 pixels in top-ad copy; that
variation also occurred between baseline runs 1 and 2. This is not a claim that
all dynamic MDN states are identical. The captured images show the real pages,
not blank or error output.

The measured endpoint is the first completed offscreen content frame, confirmed
with the filmstrip. It is a proxy, not native presentation or a standard FCP/LCP
entry; those paint-timing APIs are not implemented. MDN's large ad appears later,
so the initial hero candidate cannot be described as the final LCP.

Evidence under `rechrom-evidence/paint-timeline/`: `style-failure-comparison.json`,
`style-shared-tests.log`, `style-host-test-rerun.log`, `provenance-tests.log`,
`offscreen-publication-tests-rerun.log`, `mdn-matched-state-pixels.json`,
`wikipedia-after-pixels.json`, and the `release-before-*` / `release-after-*`
trace and screenshot directories.

The existing Baidu `external-clip-property-mismatch` failure is outside this
Style-only ownership repair and must not count as a successful page comparison.
