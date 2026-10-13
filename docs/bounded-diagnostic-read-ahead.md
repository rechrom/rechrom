# Bounded diagnostic HTML read-ahead, 2026-10-13

## Result

The missing opportunity is now demonstrated on live Baidu, not inferred from a separately downloaded response. With the real parser still blocked by the first ordinary stylesheet, bounded main-response read-ahead exposed the literal logo candidate **4.075–4.919 seconds before its actual request** in two Release navigations. It was observed **3.560–4.151 seconds before the blocked script executed**.

This is an opportunity bound, **not an implemented speedup**. No speculative request, preload admission/cache, CSS-readiness bypass, reordered script execution, or out-of-order DOM construction was added. The diagnostic remains off by default. Production policy/referrer/CORS/CSP and request reconciliation gaps from the previous audit remain open.

## Implementation and source contract

- The embedding explicitly enables `RECHROM_DIAGNOSTIC_READ_AHEAD=1` for content pages. The typed route is Page → OpenEngine → DocumentLoader; no engine reads environment variables
- `document_loader::DiagnosticReadAhead` consumes at most 16 KiB per paused-parser turn into an owned FIFO. Additional lifetime read-ahead is capped at 1 MiB. On normal parser demand, original bytes drain in order under the existing byte budget, including one-byte replay; only afterward can EOF/error be delivered
- Authoritative parser input is not appended during the diagnostic read-ahead. Its incremental decoder, tokenizer, tree builder, insertion point, callbacks and CSS/script scheduling remain authoritative. Read-ahead progress does not set `made_progress` or cause `DidPumpParser` style invalidation
- An independent `html::DiagnosticHTMLScanner` reuses the native tokenizer and existing incremental text-decoder implementation. It creates no DOM or requests. It has a 16 KiB append cap and 1 MiB cumulative source-input cap, and processes at most 512 emitted tokens per call. A native token is not character/time-preemptible; a token can span the bounded input. These are source-byte caps, not a claim that total heap usage is only 1 MiB
- The scanner is conservatively discarded at first script resume or document.write input insertion. Stop/failure destroys its queue and scanner. There are no asynchronous scanner callbacks that could outlive navigation ownership. Queued main bytes survive scan cancellation, but not navigation cancellation
- Split UTF-8/BOM/CRLF, windows-1252 and partial attributes preserve decoder/tokenizer state. Scanner-only decoding/cap errors disable scanning and do not fail the document early. Transport EOF/errors consumed ahead are held until authoritative demand
- Lexical base/referrer/CSP boundaries label an observation epoch; they do not claim policy acceptance. No URL is resolved or admitted. All template content is conservatively excluded, including declarative shadow templates. Picture context is lexical, not complete responsive-image eligibility. Raw-text/script/comment/noscript exclusions use native states and explicit scripting mode
- Trace output is capped at 128 candidate observations, with dropped-observation counts. Sources over 2048 bytes, credential-bearing spellings, or inline/non-HTTP schemes are not logged; fragments are removed. No response bodies or policy secrets are added to trace events

Pinned Chromium revision: `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`.

1. [Append and independent paused scanning](https://github.com/chromium/chromium/blob/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/html/parser/html_document_parser.cc#L962-L1025), [paused-parser seed](https://github.com/chromium/chromium/blob/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/html/parser/html_document_parser.cc#L836-L843), and [independent tokenizer](https://github.com/chromium/chromium/blob/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/html/parser/html_preload_scanner.cc#L1194-L1337)
2. [Separate insertion scanner](https://github.com/chromium/chromium/blob/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/html/parser/html_document_parser.cc#L926-L958), [resume reset](https://github.com/chromium/chromium/blob/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/html/parser/html_document_parser.cc#L1204-L1224), and [detach cancellation](https://github.com/chromium/chromium/blob/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/html/parser/html_document_parser.cc#L487-L511)
3. [Single-decoder fan-out](https://github.com/chromium/chromium/blob/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/dom/decoded_data_document_parser.cc#L45-L105) and [body read fairness](https://github.com/chromium/chromium/blob/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/platform/loader/fetch/url_loader/navigation_body_loader.cc#L116-L153)

Deliberate diagnostic-only deviation: Chromium fans already-decoded text to both consumers; this slice duplicates the existing decoder instance for observations and retains raw bytes for unchanged authoritative consumption. It avoids moving decoder ownership for a measurement. It is not proposed as the production scanner architecture.

## Precise byte definitions

- `HTTPNetworkBodyChunk.encoded_bytes`: cumulative encoded DATA payload bytes actually returned by Hyper `Incoming`, before waiting to enqueue that frame to the content decoder. Excludes headers/transfer framing. This is transport consumption, not independent wire arrival; the next read can be delayed by backpressure
- `HTTPDecodedBodyChunk.decoded_bytes`: cumulative HTTP content-decoded bytes produced by the codec, before publishing the current output chunk. A current chunk may not yet be visible to the document loader
- Diagnostic `pumped_bytes`: cumulative content-decoded body bytes consumed through URLStreamOperation, including held FIFO bytes
- Diagnostic `appended_bytes` / ordinary `DocumentLoader.Pump.body_bytes`: bytes passed to the authoritative parser byte decoder, excluding bytes still held in the diagnostic FIFO. A pump event gives that turn's count; totals are summed only for the current navigation
- Scanner `bytes_fed`: raw source bytes fed into its separate incremental text decoder. It equals consumed/pumped source bytes while scanning is active; decoded text and a tokenizer cursor are different units
- `source_utf16_end` / `consumed_utf16`: source-text UTF-16 units consumed by the native tokenizer. They are not UTF-8/network byte offsets. BOM removal and legacy encoding expansion make that distinction necessary

The candidate event occurs inside a pump, before the end-of-pump totals event. `bytes_at_candidate.diagnostic_totals` in the JSON intentionally means the **last completed totals event** and can lag the candidate. The candidate's own `bytes_fed` and `appended_bytes` are the current counters. After first resume the scanner is canceled; final authoritative counts still continue to the full body.

## Controlled delayed-CSS fixture

The fixture delivers a 243-byte prefix containing CSS plus a following inline script, then after 50 ms delivers padding and a literal image beyond 540 KiB. CSS response is delayed 1200 ms. The inline script does document.write; a later script records `first,last`. A real colored PNG verifies image decode/render.

| Release mode | Script executes, ms | Candidate observed, ms | Actual image request, ms | First content frame, ms |
| --- | ---: | ---: | ---: | ---: |
| Original executable | 1206.674 | not enabled | 1215.683 | 1245.834 |
| New executable, off | 1207.866 | not enabled | 1213.746 | 1244.448 |
| New executable, on | 1212.065 | 63.061 | 1220.080 | 1261.642 |

At discovery, the scanner had been fed all 540,528 body bytes; only 243 bytes had been appended authoritatively. Main transport EOF was observed at 60.503 ms with diagnostics, versus 1210.385 ms with the new executable off. This is expected backpressure relief, not historical wire-arrival evidence.

All three runs have exactly `/`, `/slow.css`, `/logo.png`, once each, in the server log and transport trace. Zero added requests is verified for this deterministic fixture. All finish with `readyState=complete`, order `first,last`, and the image complete. Both original→new/off and new/off→new/on strict RGBA comparisons report `changed_pixels: 0` across 893,260 pixels. The inspected screenshot contains styled text, border, and colored image, not a blank/error page. Timing differences are diagnostic overhead/noise; no speedup is claimed.

## Live Baidu Release observations

All windows use `https://www.baidu.com/`, initialized about:blank, fresh processes, the same Release executable, 1180×757 CSS viewport, DPR 1, classic scrollbars, and 500,000-event recorder cap. The three primary windows last 25 seconds. No build/test workload overlaps their timing captures. All traces have zero dropped events. Times are milliseconds relative to navigation Open.

| Sample | First CSS/script wait | Logo candidate | Script executes | Actual logo request | Logo body complete | First completed content frame |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| baidu-off | 9073.560 | not enabled | 13789.404 | 14393.025 | 18355.565 | 14446.211 |
| baidu-on-1 | 7442.531 | 7525.928 | 11677.141 | 12444.797 | 14166.558 | 12311.082 |
| baidu-on-2 | 9934.940 | 9938.466 | 13498.577 | 14012.985 | 15850.882 | 14060.593 |

Counters at logo-candidate observation, from the same live navigation:

| Sample | Transport encoded consumed | HTTP decoded produced | Pumped/scanner bytes fed | Authoritative bytes appended | Candidate end, UTF-16 | Final body bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| baidu-on-1 | 121248 | 550111 | 543253 | 236928 | 527143 | 727968 |
| baidu-on-2 | 122458 | 553950 | 537566 | 236934 | 527153 | 728362 |

In on-1, the full 727,968-byte body is pumped/scanned while the parser stays at 236,928 bytes; 491,040 bytes are held. In on-2, 728,362 bytes are pumped/scanned with the parser at 236,934 bytes; 491,428 bytes are held. Both drain completely to final pumped=appended with zero queued bytes. Final source consumption is 719,006 and 719,148 UTF-16 units, respectively. One non-HTTP candidate spelling per run is omitted from trace output; no event-drop or candidate-cap exhaustion occurred.

The off navigation separately measures 233,514 bytes appended at the first wait, while the transport has consumed 116,698 encoded bytes and the HTTP decoder has produced 341,349 body bytes. This directly shows why appended bytes must not be labeled network received. Neither those compressed counts nor any prior response's literal offset proves historical logo availability.

Main transport EOF in on-1/on-2 occurs at 7730.365/10077.254 ms, before the first script resumes. The equivalent off EOF is 13794.010 ms, after its script resumes. The current diagnostic establishes that continuing to pump this response reaches the logo within the wait; it does not reconstruct earlier trials' transport occupancy.

The first completed content frame is the existing offscreen proxy, not native FCP/LCP, hardware presentation, or necessarily the frame containing the logo. Logo body completion also is not a paint timestamp. Final screenshots visibly contain logo, search controls, hot list and fixed widget; no exact logo-frame time is manufactured.

### Live-state parity qualification

The 25-second off run had queued 33 requests; each on run had queued 39. The extra six are later ordinary script/CSS/XHR resources, not scanner dispatches. The off trace was still waiting for later external script work at the window boundary (e.g. chat-input result.js starts at 24969.5 ms); the on traces had progressed farther through that sequence. Network response timing differs substantially, so the raw fixed-window totals cannot establish equal whole-navigation request counts or a performance gain.

The initial final screenshots differ by 3,320 pixels (0.3717%), confined to search-input bounds [200,195,973,274]: focus outline and later input icon state. No exact live-page before/after pixel parity is claimed from those unmatched script states. Controlled fixture parity above is exact. A final 40-second disabled-mode check addresses the settled visual-state comparison: its screenshot versus on-1 reports **changed_pixels: 0** across all 893,260 pixels. The longer ordinary run has 41 requests; all 39 diagnostic-run normalized scheme/host/path request identities appear in it, with two additional later ordinary hector script URLs only in the extended off run. This establishes no new resource path in the diagnostic samples, not equal end-to-end request counts or identical query strings. The controlled fixture is the exact request-count/order test. The longer capture is explicitly excluded from timing comparisons; it is a final-state validation, not evidence of speedup. Its readyState is interactive, while the 25-second captures report loading, so full load completion is not claimed.

## Verification

- Release application build passed; binary hashes in `binary-hashes.txt`
- Final focused tests: native scanner 8/8; document-loader streaming 6/6 (includes the two diagnostic integration cases); held-error test 1/1; HTTP transport 11/11; Open integration 10/10
- Scanner tests cover one-byte/multiple-size chunking, split UTF-8/attributes, BOM/CRLF offsets, windows-1252, native raw-text/noscript, templates, base/meta epochs, picture annotations, EOF, byte/source caps and token-budget continuation
- Loader integration checks read-ahead without DOM/progress change, held EOF, one-byte replay, document.write ordering, cap backpressure, stop/cancellation; held-error test verifies delayed error delivery
- No full all-green workspace claim. The previous investigation's four pre-existing document-loader failures were not fixed in this slice
- No Chromium DevTools, alternate socket route, security restriction bypass, HTTP cache, CSP/referrer/CORS policy, or speculative request owner was introduced
- All prior uncommitted work is preserved; this slice is uncommitted and unpushed

Machine-readable summaries, full traces, screenshots, fixture log, test/build logs and replay scripts reside beside this report. `analyze.py` regenerates the summary without a browser. Process cleanup and final file checks are recorded separately in `cleanup.json`.

Final cleanup verified: all seven owned capture processes terminated and joined, no matching Rechrom/build/fixture process remains, and ports 9517/9523 refuse connections. All 237 initial dirty status entries remain present; final status has 245 entries. No commit or push.
