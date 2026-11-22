# Vendored crate decisions (vendor plan P0)

Measured 2026-11-22 against pristine upstream sources (crates.io `.crate`
tarballs; CrabbyAvif GitHub archive at the pinned rev; Chromium patches from
chromium.googlesource.com at `6c1d401f`). No builds were run.

Method: `diff -ruN --strip-trailing-cr` upstream vs `vendor/<crate>`, excluding
`.cargo_vcs_info.json`, `Cargo.toml.orig` and the vendored notes. Upstream
files the vendored copy omits (CI, examples, tests, C API) are counted
separately as "dropped" and are not part of the patch. Several copies were
re-formatted with default rustfmt, which inflates raw line counts, so a second
"stmt" figure counts lines after removing whitespace and line breaks and
splitting on `; { }`. That is a formatting-insensitive estimate of real
change, not an exact line count.

## Summary

| Crate | Upstream | Functional patch | Decision | Effort |
|---|---|---|---|---|
| xml 1.4.0 | crates.io + Chromium `patches/xml-v1` | 3 files, +213/-4; vendored `src/` is byte-identical to upstream + Chromium patch | KEEP: Chromium parity (DTD ATTLIST default attributes) | none |
| sfv 0.15.0 | crates.io + Chromium `patches/sfv-v0_15` | 1 file, +93/-16; byte-identical to upstream + Chromium patch | KEEP: no lenient mode in sfv 0.16.1 | none |
| hyper-util 0.1.21 | crates.io | about 35 functional lines in 3 files (raw +65/-33 including rustfmt churn) | KEEP: an own client would be 1.5-2k lines | none |
| rav1d 1.1.0 | crates.io (tag v1.1.0, `782dab2`) | 2 files, +3/-3 (two upstream backports) | REPLACE with pinned git rev `6c6f463` (contains both fixes, version 1.1.0) | S |
| crabby-avif `8f2b74b` | GitHub (Chromium DEPS pin) | 1 new adapter (`codecs/rav1d.rs`, 542 lines) + about 20 lines of registration; decode-only manifest | KEEP: no external-codec API upstream | none |
| ttf-parser 0.25.1 | crates.io | 1 file, +24/-2 (GPOS PairSet) | REPLACE with stock 0.25.1: the patched code path is unreachable in Rechrom | XS |
| zune-jpeg 0.5.15 | crates.io | +606 src lines (+395 license text), all behind opt-in `source_compat` | KEEP: libjpeg-turbo/Skia pixel parity, no upstream equivalent | none |
| softbuffer 0.4.8 | crates.io | 5 files, +1,186/-11; behaviour change is macOS-only | REPLACE: stock softbuffer on Windows/Linux, with the macOS IOSurface presenter moved into `apps/rechrom` | M |
| smithay-clipboard 0.7.2 | crates.io | 3 files, about +194/-33 stmt (raw +352/-88) | KEEP for now; fold into the app-owned Wayland data-device module later (see winit) | none now |
| winit 0.30.13 | crates.io (latest 0.30) | 33 src files, raw +4,330/-943, about +2,550/-620 stmt; 2 new files | KEEP but shrink: move the A items to `apps/rechrom`, turn C items into small hooks, re-evaluate at winit 0.31 stable | M-L |

Things that surprised me:
- The ttf-parser patch fixes code Rechrom never calls. Only `font_engine` and
  `web_font` depend on ttf-parser (Cargo.lock), and neither touches GPOS.
  Shaping uses HarfRust/read-fonts.
- Two `winit/LOCAL_CHANGES.md` claims are stale:
  - The "high-resolution waitable timer" for `WaitUntil` is already upstream.
    The 0.30.0 changelog says "make `ControlFlow::WaitUntil` work more
    precisely using `CREATE_WAITABLE_TIMER_HIGH_RESOLUTION`", and the vendored
    `event_loop.rs` diff has no timer change.
  - The Wayland "cursor and window-state fixes" in `window/state.rs` are
    formatting only.
- rav1d main has moved the crate root to `src/lib.rs`, so `rav1d::src::lib::*`
  no longer exists there. The exported `dav1d_*` C API set (17 functions) is
  unchanged.

---

## xml 1.4.0 (crates/document/html)

- Source: crates.io `xml-1.4.0.crate` (1.4.0 is the latest release) plus Chromium
  `third_party/rust/chromium_crates_io/patches/xml-v1/0001-Patch-XML-for-supporting-ATTLIST-default-attribute-v.patch`
  (drott@, 2026-08-12, references upstream kornelski/xml-rs PR #87).
- Patch: 3 files, +213/-4. `parser.rs` (+38/-2), `inside_doctype.rs`
  (+171/-1), `inside_reference.rs` (+4/-1). After applying Chromium's patch to
  pristine 1.4.0, `diff -r` against `vendor/xml/src` is empty.
- What it does:
  - Parses `<!ATTLIST elem attr TYPE default>` in the internal subset into a
    per-element default-attribute table.
  - `emit_start_element` adds missing defaults, including default
    `xmlns`/`xmlns:p` bindings, which go into the namespace stack.
  - Accepts `;` as a reference end inside DTD quoted values.
  - The `replace_unknown_entity_references` config already exists upstream.
- Rechrom use: `xml_document_parser.rs` (translation of
  `XMLDocumentParserRs`/`xml_ffi.rs`) drives `EventReader` with
  `ParserConfig`. DTD-defaulted attributes and namespaces change the DOM
  Chromium builds, so dropping the patch would lose parity.
- Alternatives: none. No later xml release exists, and upstream PR #87 is not
  released.
- Decision: KEEP, as an exact mirror of Chromium's vendored copy. When
  Chromium rebases, re-copy its patched copy.

## sfv 0.15.0 (crates/dom)

- Source: crates.io `sfv-0.15.0.crate` + Chromium `patches/sfv-v0_15/0001-sfv-v0_15-lenient-mode.patch`.
- Patch: `src/parser.rs` +93/-16. The vendored copy is byte-identical to
  upstream + Chromium patch. The manifest only drops dev/bench sections.
- What it does: `Parser::with_lenient_mode(bool)`. Lenient mode decodes byte
  sequences with Quiche-compatible base64 (synthesised padding, `.` as
  padding, ASCII whitespace skipped) and accepts a trailing decimal point
  (`scale == 100 && !lenient`).
- Rechrom use: `document_policy.rs` and `network_permissions_policy.rs` call
  `Parser::new(..).with_version(Rfc8941).with_lenient_mode(true)`.
- Alternatives: sfv 0.16.1 (latest) `parser.rs` has only `with_version`. It
  has no lenient flag and keeps the strict `scale == 100` check.
- Decision: KEEP. This is Chromium-parity behaviour with no stock
  equivalent.

## hyper-util 0.1.21 (crates/http_transport)

- Source: crates.io (0.1.21 is the latest).
- Patch: functional changes are in 3 files, about 35 lines:
  - `client/legacy/client.rs` +11: after `connection_for` and
    `conn.set(&pooled.conn_info)`, and before HTTP/1 URI normalisation and
    `try_send_request`, it awaits a `CheckConnection` request extension with
    the original `Connected`. The same check runs for new and pooled
    connections. It then stamps `CaptureSendStart` with `Instant::now()`
    immediately before `try_send_request`.
  - `connect/capture.rs` +15: the `ConnectionCheck` trait and `CheckConnection`
    extension.
  - `connect/mod.rs` +6: `CaptureSendStart` and re-exports.
  - The remaining hunks in `client.rs`, `http.rs`, `socks/mod.rs`,
    `tunnel.rs` and `capture.rs` tests are import reordering and rustfmt
    churn.
- Rechrom use:
  - `LocalNetworkAccessGate` runs the Local Network Access check on the
    selected connection's endpoint and proxy metadata, including pooled reuse.
  - `CaptureSendStart` feeds resource timing.
  - `proxy_connector.rs` uses upstream `HttpConnector`, `Tunnel` and `Socks*`.
- Alternatives:
  - A connector wrapper cannot do this, because pooled reuse never calls the
    connector.
  - Upstream `capture_connection` only observes metadata and cannot block the
    send.
  - A gating request body does not help, because HTTP/1 headers are written
    before the body is polled.
  - An own client on `hyper::client::conn` would have to reproduce:
    - the pool keyed by scheme+authority, with idle lists, idle timeout via
      the timer, and the max-idle-per-host limit;
    - checkout-vs-connect racing and HTTP/2 single-connection sharing, with
      connecting-lock dedupe;
    - ALPN h2 detection from `Connected`;
    - closed-connection detection;
    - `retry_canceled_requests` on reused connections;
    - origin-form/absolute-form URI and `Host` header handling.

    That is most of `legacy/client.rs` (about 1,000 non-test lines) plus
    `pool.rs` (1,125 lines), so 1.5-2k lines plus tests, to replace a
    35-line hook.
- Decision: KEEP. The patch is small, generic and needed. An own client is a
  large rewrite with no gain.

## rav1d 1.1.0 (crates/image_decoder, crabby-avif; opt-in `avif-backend`)

- Source: crates.io `rav1d-1.1.0.crate` = tag `v1.1.0` (`782dab2`).
- Patch: `src/decode.rs` +1/-1 (PR #1503, `6c6f463`: do not unwrap a frame
  header already taken by frame-exit cleanup in low-latency mode) and
  `src/obu.rs` +2/-2 (PR #1476, `9d78516`: signed T.35 payload-size
  arithmetic). Everything else in the diff is CI/packaging the vendored copy
  dropped.
- Upstream check (GitHub compare API):
  - `9d78516` is an ancestor of `6c6f463`.
  - `6c6f463` is 206 commits ahead of `v1.1.0` (behind 0) and on `main`
    (main HEAD `2e7216c`, 2026-10-27, is 6 commits further).
  - There is no tag after v1.1.0.
  - At `6c6f463`, `Cargo.toml` still says `version = "1.1.0"` with the same
    `bitdepth_8`/`bitdepth_16`/`asm` features. `build.rs` still compiles
    assembly only under `cfg(feature = "asm")`. The 17 exported
    `pub unsafe extern "C" fn dav1d_*` functions are the same set.
  - API delta: commit "Move crate root to src/lib.rs" removed the
    `rav1d::src::lib` module path. `rav1d::include::dav1d::{data, dav1d,
    picture}` still exist.
  - The extra commits are mostly dav1d 1.5.0 backports, perf work, lint and
    CI. For conformant streams AV1 output is bit-exact by spec.
- Decision: REPLACE with
  `rav1d = { git = "https://github.com/memorysafety/rav1d", rev = "6c6f463785a546dea18fd9ca0f14634c2ee0557c" }`
  in `[patch.crates-io]`. It still satisfies the `=1.1.0` requirement.
  - Required follow-up: in `vendor/crabby-avif/src/codecs/rav1d.rs`, change
    `use rav1d::src::lib::*;` to `use rav1d::*;`.
  - Conditions:
    - one `cargo check -p image_decoder --features avif-backend` to confirm
      that struct fields (`Dav1dSettings`, `Dav1dPicture`, `Dav1dResult`,
      `RawArc`) did not move;
    - one decode of the existing AVIF fixtures, comparing output hashes and
      decode time against the vendored copy.
  - If either check differs, keep the 3-line vendored backport.
- Effort: S (manifest + 1 import + vendor dir removal).

## crabby-avif `8f2b74b4` (crates/image_decoder, opt-in)

- Source: GitHub archive at the Chromium-pinned rev.
- Patch:
  - New `src/codecs/rav1d.rs` (542 lines, adapted from upstream `dav1d.rs`).
  - `codecs/mod.rs`, `decoder/mod.rs`, `image.rs` and `lib.rs`: about 20
    lines that register `CodecChoice::Rav1d` in place of the `dav1d` cfgs.
  - Decode-only manifest (no `build.rs`, C API, encoder or native codecs).
  - 101 upstream files dropped (C API tests, sys crates, CI).
  - The other 23 touched `src/` files are rustfmt churn only.
- Alternatives:
  - At the pinned rev and on current main (`2dc369b`, 2026-11-22),
    `trait Decoder` is `pub(crate)` and `CodecChoice` is a closed enum, so an
    external crate cannot supply a codec.
  - Upstream has no rav1d backend. Main has `dav1d` (C via `dav1d-sys`),
    `aom`, `avm`, Android MediaCodec, libjxl, and a new `native_av1`
    pure-Rust *keyframe-only* decoder (stills, grids, alpha, gain maps; no
    inter frames, so no animated AVIF).
  - Owning AVIF parsing would mean reimplementing crabby's HEIF/ISOBMFF
    parser, decoder and reformat modules (about 4.4k + 4.1k + 5.2k lines).
- Decision: KEEP, because the replacement is a large rewrite for no gain.
  Re-evaluate if Chromium moves to `native_av1`.

## ttf-parser 0.25.1 (crates/font_engine, crates/web_font)

- Source: crates.io (0.25.1 is the latest).
- Patch: `src/tables/gpos.rs` +24/-2. `PairSet` keeps the whole PairSet slice
  so Device/VariationIndex offsets resolve correctly. 17 of the lines are a
  unit test.
- Rechrom use (all ttf-parser call sites):
  - `font_engine`:
    - `clip_text_outline.rs`: `Face::parse`, `glyph_index`, `set_variation`,
      `units_per_em`, `outline_glyph` with `OutlineBuilder`.
    - `harfbuzz_shaper.rs`: `glyph_index`, `glyph_variation_index`,
      `raw_face().table(COLR/CBDT/sbix)`.
    - Test helpers in `engine.rs`, `font_fallback_iterator.rs` and
      `harfbuzz_shaper.rs`: `raw_face().table(cmap)`.
  - `web_font/ots_cff.rs`: `cff::Table::parse`, `number_of_glyphs`, `outline`
    and `CFFError::ZeroBBox` for the OTS CFF validation.
  - No GPOS, kern or OpenType-layout API is used anywhere.
  - Cargo.lock: only `font_engine` and `web_font` depend on ttf-parser.
    Shaping goes through `harfrust 0.16` on `read-fonts 0.47`.
- Decision: REPLACE with stock ttf-parser 0.25.1. Remove the
  `[patch.crates-io]` entry and `vendor/ttf-parser`. Behaviour cannot change,
  because the patched path is not reachable.
- Optional later step: drop ttf-parser entirely by migrating to
  read-fonts/skrifa.
  - read-fonts already in `font_engine` covers the charmap,
    `map_variant` (cmap14) and `table_data`.
  - Outlines with a variation location need skrifa, which `font_engine` does
    not depend on today. `skia` pins skrifa 0.46.2 on read-fonts 0.43.3, a
    different read-fonts version than harfrust's 0.47.
  - The OTS CFF validator would need re-basing on read-fonts' charstring
    evaluator, whose error taxonomy differs from `CFFError`.
  - Size: about 150-250 lines across 5 files. Medium risk in the CFF
    validation semantics. Not required by this decision.
- Effort: XS.

## zune-jpeg 0.5.15 (crates/image_decoder, feature `source_compat`)

- Source: crates.io. The latest is 0.5.16-rc3, whose features are `arith`,
  `x86`, `neon`, `portable_simd`, `std` and `log`, with no compat mode.
- Patch, all behind the new opt-in `source_compat` feature (the default
  build is unchanged):
  - +550 lines in 3 new kernel files: `idct/source_compat.rs` (ISLOW from
    `jidctint.c`), `color_convert/source_compat.rs` (`jdcolor.c`) and
    `upsampler/source_compat.rs` (`jdsample.c` fancy upsampling).
  - About 56 hook lines in `idct.rs`, `color_convert.rs`, `upsampler.rs`,
    `worker.rs`, `components.rs` and `decoder.rs`. These cover
    true-edge/last-row replication and the sampled width/height.
  - IJG license files.
- Alternatives: zune-jpeg has no public hook to swap the IDCT, upsampler or
  colour converter, and no released 0.5.x has a libjpeg-turbo-compatible
  path. The purpose is pixel parity with Skia's libjpeg-turbo JDCT_ISLOW +
  fancy-upsampling route; stock kernels round differently. Not re-measured
  here (no builds).
- Note: `source_compat` returns before the SIMD dispatch in all three
  choosers, so decoding uses scalar kernels. Profile JPEG decode cost when
  measuring image-heavy pages.
- Decision: KEEP, because no alternative gives the parity.

## softbuffer 0.4.8 (apps/rechrom)

- Source: crates.io (0.4.8 is the latest).
- Patch: 5 files, +1,186/-11.
  - New `backends/cg_iosurface.rs` (515): a bounded 4-slot IOSurface pool.
  - `backends/cg.rs` (+411/-5): prepared buffers, CALayer.contents
    submission, CA transaction observer, presentation feedback.
  - `lib.rs` (+203/-6): `PreparedBuffer`, `BufferFormat`, `PresentCallback`,
    `PresentationFeedback`, `NativeTransactionObserver`, `row_stride`,
    `pixel_format`, `present_with_presentation_feedback`.
  - `backend_interface.rs` (+38) and `backend_dispatch.rs` (+19): default
    impls.
  - New macOS-only `native-iosurface` feature.
- Windows/Linux use (`window_surface.rs`, non-macOS cfg): only
  `Buffer::row_stride()`, `Buffer::present_with_presentation_feedback()` and
  the `PresentCallback` / `PresentationCallback` / `PresentationFeedback` /
  `PresentationUnavailableReason` types. On these backends, `row_stride` is
  the packed width and the feedback path is `present()` followed by
  `swap_ack(start, end)` and `Unavailable(BackendHasNoTimestamp)`. That is
  trivial to express in the app with stock `buffer.present()`.
- macOS: the IOSurface/CALayer presenter owns the layer on winit's NSView. The
  same objc2 code can live in an `apps/rechrom` macOS module that obtains the
  NSView through raw-window-handle, as softbuffer does. That matches AGENTS
  §1.5 (platform code in the embedding, declared against the OS API) and the
  Chromium owners cited in `LOCAL_CHANGES.md`.
- Decision: REPLACE:
  - stock softbuffer 0.4.8 for Windows/Linux, with the callback/feedback
    types moved into the app;
  - the macOS presenter moved verbatim into `apps/rechrom` (no softbuffer on
    macOS, or stock softbuffer kept only for its non-IOSurface fallback).

  Behaviour, quality and performance stay the same because the code is
  relocated, not rewritten.
- Effort: M, about 1,000 lines moved plus about 150 lines of layer
  attach/resize observation. Needs macOS validation (headless + one visible
  window check).

## smithay-clipboard 0.7.2 (apps/rechrom, Linux)

- Source: crates.io. The latest, 0.7.3, still has only text
  `load`/`store`/`load_primary`/`store_primary`.
- Patch: `lib.rs` +49/-3 (`ClipboardData`, `load_formats`, `store_formats`
  with per-call reply channels and 5 s timeouts), `state.rs` +270/-75 and
  `worker.rs` +33/-10, about +194/-33 stmt with the rest rustfmt churn. It
  adds:
  - multi-MIME snapshots;
  - acknowledged writes (flush result returned);
  - bytes retained per source until cancellation;
  - focused-seat tracking on keyboard enter.

  PRIMARY stays the upstream text API.
- Rechrom use: `primary_clipboard.rs` (`load_formats(READ_FORMATS)`,
  `store_formats`, `load_primary`).
- Alternative: an own module on smithay-client-toolkit's data-device and
  primary-selection support (sctk is already in the graph through winit) is
  roughly the same size as smithay-clipboard itself (about 1k lines). There is
  no gain on its own.
- Decision: KEEP for now. When the Wayland drag code moves out of winit
  (winit A10 below), build one app-owned Wayland data-device module for DnD,
  selection and primary, and drop this crate then.

## winit 0.30.13 (apps/rechrom)

- Source: crates.io. 0.30.13 is the latest 0.30. 0.31.0-beta.4 is a split
  crate set (winit-core, winit-appkit, winit-win32, winit-wayland, winit-x11).
- Patch: 33 `src` files, raw +4,330/-943, of which a large share is rustfmt
  churn (the vendored copy was formatted with default rustfmt, not upstream's
  style). The formatting-insensitive estimate is about +2,550/-620. The two
  new files are `wayland/seat/data_device.rs` (524) and
  `windows/drop_handler/virtual_file.rs` (476). Big contributors:
  - X11 `event_processor.rs` (+635/-228) and `dnd.rs` (+115/-62)
  - macOS `window_delegate.rs` (+487/-115) and `view.rs` (+306/-55)
  - shared `event.rs` (+388/-27)
  - Windows `drop_handler.rs` (+305/-64)
- Rechrom use: `app.rs` (`open_urls`, `set_ime_text_state`,
  `NativeDragDrop`), `input.rs` (`MacOSMouseWheelMetadata`,
  `KeyEventExtMacOS`, `from_rgba_with_scale_factor`),
  `native_keyboard_modifiers.rs` (`caps_lock_state`),
  `native_drag/{windows,x11,wayland}.rs` (`drag_source_seat`),
  `primary_clipboard_windows.rs` (`clipboard_virtual_files`).

### Classification

Key: A = can move into `apps/rechrom` using raw-window-handle and direct OS
calls. B = already in stock winit 0.30.x. C = must stay in winit, so the table
names the smallest generic hook that would let the logic live in the app.

| # | Change | Platform | Size (raw) | Class | Evidence / proposed hook |
|---|---|---|---|---|---|
| 1 | `WaitUntil` high-resolution waitable timer | Windows | 0 (already upstream) | B | 0.30.0 changelog; `wait_for_messages_impl(&mut high_resolution_timer, ..)` is identical upstream. Remove the claim from `LOCAL_CHANGES.md`. |
| 2 | `native_key_code` on macOS key events | macOS | ~5 | B | `PhysicalKeyExtScancode::to_scancode()` returns the same virtual key code. |
| 3 | `unmodified_text` (`charactersIgnoringModifiers`) | macOS | ~5 | B, partly | `KeyEventExtModifierSupplement::key_without_modifiers()` exists but yields a `Key`, not the raw string. Keep it only if the exact string is needed, as part of hook C8. |
| 4 | Caps Lock in `Modifiers` | Windows | ~10 | A | Call `GetKeyState(VK_CAPITAL)` in the app's key/modifier handler. winit 0.30 Windows calls the handler synchronously from the window procedure, so the state is message-time. |
| 5 | Caps Lock | macOS | ~5 | A | Read `NSEvent.modifierFlags` (class property, current state) in the handler. |
| 6 | Caps Lock | X11 | shared xkb | A | Call `XkbGetState`/`XkbGetIndicatorState` on the Xlib display from raw-window-handle. |
| 7 | Caps Lock | Wayland | xkb `state.rs` +10 | C | Locked modifiers exist only in winit's wl_keyboard xkb state. Hook: expose lock state on `Modifiers` (the current 10-line `From<ModifiersState>`). |
| 8 | OLE `IDropTarget` destination, virtual files (`FileGroupDescriptor`/`FileContents`), `clipboard_virtual_files`, IDropTarget `POINTL` by-value ABI fix | Windows | ~790 | A | Upstream `WindowAttributesExtWindows::with_drag_and_drop(false)` stops winit's `OleInitialize`/`RegisterDragDrop`. The app calls `OleInitialize` and `RegisterDragDrop(hwnd)` with its own IDropTarget (move `drop_handler.rs` + `virtual_file.rs`). Paste already calls the reader directly. |
| 9 | `application:openURLs:` → `ApplicationHandler::open_urls` | macOS | ~60 | A (fallback C) | Register `NSAppleEventManager` handlers for `kInternetEventClass/kAEGetURL` (and `kAEOpenDocuments` for file URLs) before `run_app`, on the main thread. Fallback hook: winit's delegate forwards `application:openURLs:` to a `Box<dyn Fn(Vec<String>)>` set on the event-loop builder (about 25 lines). winit 0.30 installs its own NSApplicationDelegate, despite its doc comment. |
| 10 | `wl_data_device` drag destination + outgoing drag, button grab serial, `drag_source_seat` | Wayland | ~750 | A | Bind your own `wl_data_device_manager`, `wl_seat`, `wl_data_device` and a listen-only `wl_pointer` (for the button-press serial) on a private event queue of winit's `wl_display` from raw-window-handle, the same technique smithay-clipboard uses. Match events to winit's `wl_surface` pointer. winit 0.30 binds no data device, so nothing conflicts. Merge with smithay-clipboard. |
| 11 | XDND destination (XdndAware, Enter/Position/Status/Drop/Finished, action list, MIME selection with INCR) | X11 | ~760 | C | ClientMessage/SelectionNotify for winit's window are consumed by winit's event processor. Hook: `EventLoopBuilderExtX11::with_x11_event_hook(FnMut(*const XEvent) -> bool)` that sees raw events first and can consume them, mirroring upstream Windows `with_msg_hook`. About 20 lines in winit; the XDND logic moves to the app. |
| 12 | Drag destination: `NSDraggingDestination` on winit's window delegate, security-scoped NSURLs, `NSFilePromiseReceiver` promises, pasteboard MIME data, operation feedback | macOS | ~400 | C | The dragging messages go to winit's delegate class. Hook: `WindowExtMacOS::set_dragging_destination(Option<Retained<ProtocolObject<dyn NSDraggingDestination>>>)`, where winit forwards `draggingEntered/Updated/Exited/performDragOperation` when set and registers the requested types. About 30 lines in winit. |
| 13 | Wheel metadata (NSEvent phase/momentum/timestamp, CGEvent continuous + point deltas), PixelDelta/LineDelta by `IS_CONTINUOUS`, `BROWSER_PROFILE_NATIVE_WHEEL` eprintln | macOS | ~120 | A (risky) / C | A: `NSEvent addLocalMonitorForEventsMatchingMask:ScrollWheel` in the app records the raw fields and pairs them FIFO with winit's `MouseWheel` on the same thread; the app reclassifies deltas. The env-var profiler is app diagnostics and belongs in the app regardless. Cleaner C hook: `WindowEvent::MouseWheel` carries an opaque platform event pointer (`NSEvent*`, valid during dispatch) in place of the `MacOSMouseWheelMetadata` struct. |
| 14 | NSTextInputClient state: `set_ime_text_state`, `attributedSubstringForProposedRange`, `selectedRange`/`markedRange`/replacement ranges, insert-text commit length | macOS | ~200 | C | The text-input client is winit's NSView. Minimal hook: the current `set_ime_text_state(text, selection, composition)` setter; it has the same shape as winit 0.31 `ImeSurroundingText`. |
| 15 | Key-event extras: `doCommandBySelector` edit commands, `ime_processed`, `committed_text`, IME event ordering/flush around `keyDown:` | macOS | ~150 | C | Needs winit's `keyDown:`/`interpretKeyEvents:` sequence. Hook: per-KeyEvent `commands: Vec<String>` + `ime_handled: bool`. winit 0.31 offers `ApplicationHandlerExtMacOS::standard_key_binding`, but as a separate callback, not tied to the key event. |
| 16 | Custom cursor Retina scale (`from_rgba_with_scale_factor`) | macOS | ~20 | C | winit owns the NSCursor through cursor rects. Hook: the current scale field (minimal), or `CustomCursorExtMacOS::from_ns_cursor(Retained<NSCursor>)`. |
| 17 | Shared types: `NativeDragDrop*`, `Event::OpenUrls`, `MacOSMouseWheelMetadata`, `MouseWheel.native`, `Modifiers.caps_lock` | all | ~430 | follows | They become app types once 4-16 are done. Orbital and web changes (+104/-44, +1) exist only so these types compile, and disappear with them. |
| 18 | Formatting churn | all | large share of raw lines | n/a | Re-format the vendored files with upstream winit's rustfmt style so the real patch is reviewable. |

### Decision

KEEP, shrinking it in three steps:

1. Move the A items into `apps/rechrom` and delete them from the vendored
   copy: 4, 5, 6, 8, 9, 10, plus the wheel profiler in 13. That removes about
   1,700 lines, including both new files. Drop the stale claim in 1, and use
   the upstream APIs for 2 and 3. Effort: M. Needs Windows and Linux
   (X11 + Wayland) validation.
2. Replace the C logic with the generic hooks listed for 7, 11, 12, 13, 14, 15
   and 16. That leaves roughly 150-250 lines of hook code in winit, with the
   drag and IME logic in the app. Effort: M. Needs macOS validation.
3. When winit 0.31 is stable, re-evaluate. 0.31.0-beta.4 already has data
   transfer and DnD (incoming on appkit/win32/wayland/x11, outgoing on
   appkit/win32/wayland), macOS `standard_key_binding` and
   `ImeSurroundingText`. Most C hooks may then be B.
