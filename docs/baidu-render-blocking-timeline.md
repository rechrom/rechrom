# Baidu stylesheet/parser readiness investigation, 2026-10-13

This is a request-level continuation of [the first-visible timeline](page-paint-timeline.md).
It retains the same completed-offscreen-content-frame endpoint, document generation
filtering, and native FCP/LCP limitations. No renderer readiness gate is bypassed.

## Instrumentation contract

Tracing is opt-in. The existing bounded recorder now supports optional textual
metadata, and exports it through the Performance snapshot and Chrome trace.
The HTTP transport allocates one ID per traced request and one URL mapping at
`HTTPRequestQueued`. The URL mapping strips userinfo and fragments. Subsequent
HTTP events carry numeric values, not body data or repeated URL logs. A request
captures its recording ID and owner context; async completion cannot cross into
a later recording or acquire the IO worker's unrelated context.

- `HTTPRequestQueued` / `HTTPRequestAdmitted`: scheduling admission, including
  queue wait. Admission is not connection completion.
- `HTTPRequestStart` / `HTTPResponseHeaders`: actual HTTP attempt/redirect work
  and delivered headers. DNS, TCP and TLS are not separately instrumented.
- `HTTPNetworkBodyComplete`: transport `Incoming` EOF, with encoded byte count.
  For streamed documents this observation can be delayed by bounded consumer
  backpressure; it is not an independently measured wire-arrival timestamp.
- `HTTPDecodeComplete` / `HTTPBodyComplete`: codec completion and joined body
  completion, with decoded and encoded sizes. The buffered CSS path completes
  on the IO worker before its consumer polls it.
- `HTTPResponseReady` / `HTTPResponseDelivered`: separate completion from
  consumer polling. `HTTPStreamComplete` precedes publishing the Finished marker;
  `HTTPStreamDelivered` observes consumption of that marker.
- Request failures, retries and cancellation remain visible.
- `ResourceRequestReused` identifies in-document coalescing/resident reuse.
  The transport does not implement an HTTP response cache. A same-process
  revisit is therefore not described as a warm HTTP-cache navigation.
- Parser events identify stylesheet owner nodes, queue/application/response
  times, parse spans, script IDs and waits. `DocumentLoader.Pump` records actual
  body bytes consumed, parser status and the paused script. Status values are
  0 need-input, 1 budget-yield, 2 script-wait, 3 parser-finished.

The existing stylesheet readiness policy and polling short circuit are unchanged.
No event emits response bodies; tracing disabled copies no event text.

## Source review

Chromium intentionally waits to execute parser-inserted inline scripts when
previous relevant stylesheets still block scripts. See
[ScriptLoader](https://github.com/chromium/chromium/blob/main/third_party/blink/renderer/core/script/script_loader.cc#L1083-L1095)
and [Document readiness](https://github.com/chromium/chromium/blob/main/third_party/blink/renderer/core/dom/document.h#L668-L670).
A script need not visibly access CSS to participate in that ordering.

Three distinct local gaps were identified, without assuming they caused the
observed initial Baidu wait:

1. `PollParserStyleSheets` waits for a FIFO of all outstanding parser sheets;
   alternate and nonmatching-media sheets can incorrectly delay scripts or
   later completed sheets. Chromium's
   [LinkStyle](https://github.com/chromium/chromium/blob/main/third_party/blink/renderer/core/html/link_style.cc#L259-L282)
   classifies matching-media, nonalternate sheets as critical. Fixing this
   requires separate script readiness and minimal host media state, not
   ignoring all pending CSS. Connected sheets already have DOM-owner cascade
   order independently of completion order.
2. Rechrom has no speculative HTML preload scanner. It discovers resources
   only through the real parser and stops reading ahead while paused. Chromium's
   [HTMLDocumentParser::Append](https://github.com/chromium/chromium/blob/main/third_party/blink/renderer/core/html/parser/html_document_parser.cc#L922-L955)
   and [paused-parser path](https://github.com/chromium/chromium/blob/main/third_party/blink/renderer/core/html/parser/html_document_parser.cc#L796-L802)
   feed the scanner independently. This can serialize later CSS/script/image
   discovery. It does not shorten the transfer of the first already-discovered
   stylesheet by itself.
3. The local comment tying all render blockers to first rendering differs from
   current Chromium's post-body policy. Chromium's
   [RenderBlockingResourceManager](https://github.com/chromium/chromium/blob/main/third_party/blink/renderer/core/loader/render_blocking_resource_manager.cc#L195-L201)
   refuses new render blockers after body insertion, while
   [StyleEngine](https://github.com/chromium/chromium/blob/main/third_party/blink/renderer/core/css/style_engine.cc#L359-L380)
   and [HTMLDocumentParser](https://github.com/chromium/chromium/blob/main/third_party/blink/renderer/core/html/parser/html_document_parser.cc#L1283-L1303)
   maintain a separate parser-blocking-sheet pause. Changing the local boolean
   alone would be an incomplete translation.

These links were inspected on Chromium main, not verified against the older
pinned revision in prior repository documentation. Source inspection found the
production loading/parser/dynamic-resource pipeline shares one ResourceEngine;
no duplicated ownership bypass was identified.

## Evidence

Raw traces, request summaries, deterministic server logs, screenshots, binary
hashes and tests are retained in `rechrom-evidence/baidu-render-blocking/` beside
the repository. Numeric results, exclusions and interpretation are recorded below.

## Measured Baidu cause

All six valid Release windows identify the same first external sheet:
`https://pss.bdstatic.com/r/www/static/font/cosmic/pc/cos-icon_01c45ca.css`.
The filmstrip's DOM probe verifies `rel=stylesheet`, absent `media`, absent
`disabled`. It is not the alternate/nonmatching-media classification case.
Its decoded payload is 26,009 bytes in every sample; encoded DATA payload is
4,097 or 4,190 bytes. Encoded counts exclude HTTP headers and transfer framing.
All six CSS responses are HTTP/2 status 200, with no retry or redirect.

Three fresh processes each navigate from initialized about:blank to Baidu,
then revisit Baidu in that same process. The 1180 × 757 viewport, DPR 1,
classic scrollbars, Release executable, recorder and 25-second window are
identical. No build or other owned page workload overlaps the captures.
“Revisit” does not imply a warm HTTP cache: Page creates a new document resource
owner, and no cross-document HTTP response cache exists. OS/proxy/origin cache
state is uncontrolled. Cache-Control and Age were not retained by the current
response projection, so proxy/origin cacheability is not inferred.

Milliseconds, medians with min–max ranges:

| Measurement | Fresh process | Same-process revisit |
| --- | ---: | ---: |
| CSS attempt start → response headers | 5911.348 (5486.023–6941.996) | 4455.340 (3932.358–4567.713) |
| CSS queue wait | 0.064 (0.050–0.215) | 0.053 (0.050–0.085) |
| CSS ready → consumer Poll | 0.074 (0.067–0.094) | 0.086 (0.074–0.117) |
| CSS parsing | 6.139 (5.359–6.617) | 4.766 (4.418–9.319) |
| CSS body completion → first Style start | 18.780 (16.303–24.224) | 17.776 (17.328–32.118) |
| Navigation → completed content pixels | 13411.968 (12978.832–14512.863) | 11641.803 (8084.110–12031.539) |
| Document commit → completed content pixels | 6571.275 (6209.806–7556.636) | 5080.238 (4567.924–5171.681) |

The request-to-header interval accounts for **more than 99% of the interval
from document commit to first Style** in every valid sample. This interval
includes whatever connection, TLS, proxy and origin response work occurred;
it is not isolated origin-server time. The previous approximately 6.4-second
readiness gap is thus reproduced and explained by the first stylesheet's
HTTP response latency. This is not a demonstrated lost wake, saturated
request queue, slow CSS parsing or delayed resource polling.

Exact CSS request IDs, in sample order, are 11305, 39622, 12446, 38964, 8523,
45652. The raw per-request events, URLs and byte counts are in `comparison.json`
and each sample's `summary.json`/`trace.json`.

For example, fresh sample 1 commits at 7202.162 ms; CSS request 11305 starts
at 7203.100, receives headers at 12689.123, reaches body EOF at 12690.775,
and completes decoding at 12690.927. Its consumer receives it at 12691.024.
External parsing takes 6.617 ms. Sheet owner 13 and the six already-parsed
inline sheets are applied by 12698.515. Parser-blocking inline script node
37 resumes at 12698.533. Parsing proceeds through later scripts and buffered
HTML, and first Style starts at 12715.151. Completed content pixels appear at
13411.968.

The root-readiness fields are lifecycle observations, not exact body-insertion
timestamps. In that sample no renderable root is observed during the CSS wait;
parsing remains paused at script 37. After it resumes, a later script triggers
synchronous Style before the next outer lifecycle observation reports a
renderable root at 13338.415. That late observation must not be mistaken for
an additional 640 ms of missing DOM readiness. Exact body insertion is not
separately instrumented by this change.

No scheduling optimization was applied: bypassing this ordinary sheet would
change parser/CSS/script semantics. Revisit differences and network variation
are not claimed as code gains.

## Main-content observation and next gap

A separate instrumented filmstrip confirms:

- Completed first content pixels: 13360.649 ms; only top navigation links
- Search controls, hot list and fixed widget visible by 14332.910 ms
- Baidu logo visible by 15208.626 ms

Screenshot times are observation bounds with capture overhead, excluded from
the six timing windows. Native paint/LCP entry arrays remain empty. Neither
these observations nor the logo are relabeled as standards FCP or final LCP.
The filmstrip remains `readyState=loading`; full page load completion is not
claimed. No new Chromium screenshot or Chromium timing parity was established.

The logo request is not discovered until after the first CSS/script pause.
For fresh samples 1–3, it starts at 13348.248, 14470.880 and 12923.900 ms,
then completes its 15,444-byte body at 15233.490, 16416.906 and 15856.880 ms.
This, together with the fixture below, makes independent speculative resource
discovery the next concrete implementation gap to investigate. It requires
Chromium's scanner/tokenizer ownership and request reconciliation, not a
Baidu URL special case or unstyled early paint. Its real-page benefit has not
been implemented or measured here.

## Controlled ordering and overhead checks

The local server supplies a 397-byte HTML document, a 1200 ms delayed sheet,
a following inline script, a 400 ms delayed blocking external script, and a
600 ms delayed later sheet. In the first instrumented run:

1. Main-document body EOF is observed at 5.309 ms
2. slow.css starts at 5.431 and completes at 1206.325 ms
3. Inline script runs at 1206.804 ms, after the sheet is applied
4. gate.js is first requested at 1207.244 and completes at 1608.385 ms
5. late.css is first requested at 1608.870 and completes at 2209.964 ms
6. First completed content pixels arrive at 2245.998 ms

The final DOM is styled correctly. All six instrumented fixture windows retain
this ordering. The original executable's initial fixture first pixels are
2246.190/2242.617 ms for its fresh/revisit pair. This demonstrates serialization
of discovery despite the entire HTML having arrived; it does not demonstrate
an implemented preload optimization.

An alternating original/instrumented **tracing-off** check runs three fresh
processes plus three revisits for each executable. Since the recorder is off,
its endpoint is explicitly the existing first content *paint artifact* log,
not completed pixels:

| First artifact, ms | Original | Instrumented build, tracing off |
| --- | ---: | ---: |
| Fresh process | 2243.133 (2222.862–2246.631) | 2224.697 (2223.715–2225.266) |
| Revisit | 2234.712 (2224.626–2247.707) | 2243.109 (2229.339–2244.586) |

The ranges overlap, with no consistent directional effect. This network-delay
fixture does not prove zero CPU overhead or a statistically bounded overhead.
No speedup is claimed. All 19 strict fixture screenshot comparisons against
the original baseline have `changed_pixels: 0`. This parity statement applies
only to the controlled fixture, not to unrelated real-page/Chromium states.

## Verification and exclusions

- Release build succeeds (`rechrom_app`), final executable SHA-256
  `80041c2a940021b71c38d71f6318cd6c82011e31c40f4bcce256e85335f0aed2`
- New HTTP trace regression: 1/1; recorder: 1/1; open: 10/10;
  resource: 2/2; DevTools: 8/8; HTTP transport library: 11/11
- Document-loader: 19 passed, 4 failed. The retained pre-instrumentation test
  executable reproduces the exact four failures: missing source fixture,
  resource terminal-after-consumption cancellation, failure/cancellation
  operation release, and module graph cancellation release. They remain open;
  this is not a full all-green workspace claim
- One original second-cold attempt fails with a proxy CONNECT tunnel reset;
  capture orchestration ends before a trace is retained. It is explicitly
  excluded. The authorized retry supplies the remaining two complete pairs
- First pair's optional DOM probe failed on unsupported `innerText`; traces
  and screenshots are intact. The corrected subsequent probes report loading
  or interactive, never complete. Their timing windows remain valid
- All valid traces have zero dropped events. No native FCP/LCP, physical
  presentation, DNS/TCP/TLS partition or HTTP-cache hit rate is manufactured
- All owned capture/fixture subprocesses are terminated and joined; the
  interrupted capture's endpoint is closed. Ports 9517 and 9523 both return
  connection refused in the final cleanup check. No build remains running
- Existing working-tree changes are preserved; no commit or push was made
