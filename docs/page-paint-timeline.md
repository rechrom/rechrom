# Real-page first-visible timeline, 2026-10-13

## Result

A document-owned selector service was accidentally rebuilt for each resolved
element. Reusing the existing document owner removes repeated whole-DOM handle
setup. Initial Style time falls by 46.3% on MDN and 51.9% on Wikipedia with the
same node/resolution counts. The response-commit-to-first-content-frame interval
falls by 21.1% and 18.3%, respectively. Matching state screenshots remain exact.

Baidu exposed a separate fatal root/fixed-position clip inconsistency. The
source-aligned repair preserves the root overflow clip, passes all 19 root
scrollbar tests, and survives three real-page runs plus a visual filmstrip.

## What these two paint times mean

Chrome's [FCP definition](https://web.dev/articles/fcp) concerns the first
visible content after navigation. [LCP](https://web.dev/articles/lcp) concerns
the largest eligible visible content; later candidates can supersede earlier
ones. DOM completion, a display list, and the first large image are insufficient
to establish these metrics.

Rechrom currently produces no `paint` or `largest-contentful-paint` entries.
The supported read-only cloud Chromium evaluator does not expose `performance`
or `PerformanceObserver`, so there is no valid Chromium timing reference here.
No claim of equivalent Chrome speed, native FCP, physical screen presentation,
or final LCP is made.

Instead, the measured endpoint is **first completed offscreen content pixels**.
The new `OffscreenFramePublished` event is emitted after completed memory output
is published, and carries the activated document generation and recording
revision through compositor submission. Toolbar, empty and retained-old-document
frames are excluded. A filmstrip verifies what those content pixels contain.
This is a useful first-visible proxy, not a standards implementation.

## Repeated Release results

Three fresh processes per before/after/site, same cloud host, real URL, classic
scrollbars, 1180 × 757 CSS pixels, DPR 1, same font installation and Release
profile. Startup loads about:blank and initializes the runtime first. Site
state is cold within each process; OS font/DNS/proxy caches are not controlled.
No compiler or other owned page workload overlaps the trials. Trace collection
is identical; screenshots are captured only after the measurement window.
These are not warm-navigation or fully cold-machine measurements.

All values below are milliseconds; ranges are minimum–maximum, not confidence
intervals. Origin is the native navigation `Open` start.

| Site / measurement | Before median (range) | After median (range) |
| --- | ---: | ---: |
| MDN navigation → first content pixels | 6630.997 (4934.392–7955.945) | 5204.847 (4956.105–7209.092) |
| MDN document commit → first content pixels | 1882.574 (1748.361–1898.853) | 1485.934 (1458.253–1496.368) |
| MDN initial Style | 672.881 (661.911–708.240) | 361.561 (359.120–370.401) |
| Wikipedia navigation → first content pixels | 6310.505 (6105.259–9339.304) | 5984.451 (5683.755–8320.784) |
| Wikipedia document commit → first content pixels | 3487.148 (3429.347–3600.238) | 2850.012 (2682.955–2879.769) |
| Wikipedia initial Style | 1363.486 (1359.227–1460.548) | 655.876 (634.889–666.608) |

MDN resolves 749 of 2414 nodes; Wikipedia resolves 1194 of 2755 nodes, unchanged
across the compared initial Style passes. Main-document response timing varies
by seconds. The entire navigation median improvement must not be attributed to
the selector repair; the matched-work Style result is the strongest evidence.

Final Baidu first-content runs are 16315.131, 8692.135 and 11677.720 ms, median
11677.720 ms. They stay alive through the 25-second window. The first frame is
only the top navigation links, not the completed search page. The old build
crashed before a complete page and is not a valid successful-page baseline.
Baidu ends with `readyState=loading` in run 1 and `interactive` in runs 2–3;
none reaches `complete`, so full network/script load completion is not claimed.

## First large content and later candidates

Separate filmstrips sample screenshots approximately every 200 ms. Their times
are observation bounds, may include capture overhead, and are excluded from
the repeated timing comparison above. No largest-content eligibility algorithm
has been invented or substituted for Chromium's.

- MDN's initial frame already contains the hero heading and illustration. In
  the clean baseline filmstrip, first content pixels are published at 7331 ms;
  the real large blue advertisement image is first observed by 17389 ms.
  The blue placeholder in earlier frames is not counted as an image. In the
  clean selector-after filmstrip, first pixels are published at 5570 ms and the
  actual ad image is observed by 11818 ms. These single runs show why the hero
  is not safely called the final LCP; variable ad delivery prevents a causal
  LCP-speedup claim.
- Wikipedia's first visual frame contains the central globe, while some text
  appears later as fonts/resources arrive. It supplies a visible initial large
  image candidate, not proof of the final standards LCP. Its diagnostic
  filmstrips are visual evidence only, not additional benchmark trials.
- Baidu's final filmstrip publishes initial links at 13162 ms. Search controls,
  hot list and fixed side widget are visible by 14071 ms; the logo is visible by
  15168 ms. Later icons/text continue arriving. These are useful-content and
  image observations, not LCP entries.

All three pages still lack a valid final LCP value. Implementing Chromium's
paint-timing candidate lifecycle and presentation feedback is a separate
remaining engine capability, rather than relabeling these proxy timestamps.

## Timeline coverage and limits

The retained traces include document commit, parsing/loading work, Style,
font polling, layout, Paint, display-list publication, raster batches,
composition, candidate-ready, completed offscreen publication and document-load
completion when reached. They preserve the distinction between paint artifacts
and pixels; compositor provenance is taken from the activated snapshot.

For example, MDN baseline run 1 commits at 6073.371 ms, spends 708.240 ms in
initial Style, produces its first content artifact at 7693.664 ms, makes the
compositor candidate ready at 7954.738 ms, and publishes pixels at 7955.945 ms.
Layout and resource/font work occur between Style and the artifact; later
resource-driven frames are preserved, not bypassed to improve a number.

Current tracing does not separately provide DNS, TCP connect, TLS, HTML first
byte, a complete isolated parser interval, font-request wait versus processing,
or hardware presentation feedback. Document commit is not TTFB. Stage spans
can nest or overlap, especially raster workers, and must not be summed into a
serial critical path. No missing phase is silently reported as zero.

The next specific investigation is Baidu's blocking-resource/parser readiness,
not its 142–165 ms initial Style pass. Final run 3 commits at 4670 ms but does
not start Style until 11111 ms. During that gap, 758 `UpdateFrameIfNeeded`
observations report both pending render-blocking sheets and no renderable root;
the sheet flag clears around 11098 ms. Source entry points are
`crates/browser/src/page.rs:2292–2303` and
`crates/open/src/script_scheduler.rs:1098,1352`. This establishes the next
measurement target, not an assertion that the legitimate blocking policy is
wrong. Removing those checks would paint an incorrect intermediate document.

## Chromium alignment and guards

- [Document selector ownership](document-selector-service-lifetime.md) records
  pinned Chromium `StyleEngine`/`StyleResolver` ownership and operation-scoped
  mutable matching state, source hashes, production changes and exact tests.
- [Root fixed overflow clip](root-fixed-overflow-clip.md) records pinned
  `PaintPropertyTreeBuilder` fixed context, root overflow clip and logical
  effect rebinding. No compositor validator was weakened.
- Three new selector/session tests pass. Full Style suite is 657 passed and
  the same 14 pre-existing failures as baseline; it is not an all-green suite.
- Two compositor provenance tests and the offscreen-publication test pass.
- The three fixed-control regressions fail before the repair; the complete
  root-scrollbar suite passes 19/19 after it.
- Strict screenshot comparison changes zero pixels for the matched-state MDN
  run 3 pair and all three Wikipedia pairs. Other MDN pairs differ by 999 pixels
  of live ad text; the same variation also occurs between baseline runs.
- A pre-existing Release compilation guard around debug-only
  `CheckNoBlockFragmentation` was corrected; it adds no Release rendering logic.

Evidence is in the sibling `rechrom-evidence/paint-timeline/` directory:
`comparison.json`, all `release-before-*` / `release-after-*` runs,
`release-final-baidu-*`, `film-clean-before-mdn`, `film-clean-after-mdn`,
`film-final-baidu`, test logs, binary hashes and pixel-diff JSON. The baseline
and selector-after binaries preserve an isolated before/after comparison.
The final binary also includes the Baidu clip repair. No commit or push was made.

Final combined-build MDN and Wikipedia smoke captures also reach complete
documents with the expected page content. The first MDN smoke attempt lost its
HTTP endpoint before collection; its log has no fatal/panic diagnostic, so that
attempt is retained as inconclusive and excluded from timings. A fresh MDN
filmstrip retry and the Wikipedia run both completed. This is not a claim that
the unexplained lost sample has been diagnosed.

The request-level continuation is documented in [Baidu stylesheet/parser readiness](baidu-render-blocking-timeline.md).
