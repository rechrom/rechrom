# Local winit 0.30.13 changes

Upstream: https://crates.io/crates/winit/0.30.13 (MIT / Apache-2.0; both
licenses retained). Only `src/` and the manifest differ from the published
crate. Upstream examples, tests and CI files are not vendored; the manifest's
example/test targets were removed with them. Compare against the published
crate to review every change (`diff -ruN --ignore-all-space`, excluding this
file: 27 `src` files plus the manifest, about 3,240 added and 770 removed
lines, much of it rustfmt churn).

These changes give the Rechrom embedding (`apps/rechrom`) the native facts
Chromium reads directly from the OS. The browser engines never call winit.

## Cross-platform API

- `event::NativeDragDrop` / `NativeDragDropEvent`: one native drag transaction
  per packet (enter, over, drop, leave) with position, modifiers, source
  operations, an atomic destination-operation feedback owner, MIME data, text
  and file offers. File offers carry an open descriptor (`NativeDragFileOffer`)
  rather than a path to reopen; promised and virtual files arrive through
  `NativeDragFilePromise` / `NativeDragVirtualFileOffer` read off the UI thread.
  macOS, X11 and Wayland produce these packets inside winit; on Windows the
  embedding's own OLE drop target builds the same values.
- `ApplicationHandler::open_urls` / `Event::OpenUrls`: OS requests to open URLs,
  passed through unparsed so the browser applies its own scheme policy.
- `Modifiers::caps_lock_state`: the native Caps Lock state (macOS, X11,
  Wayland; Windows returns `None` and the embedding reads `GetKeyState`).
- Mouse wheel events optionally carry `MacOSMouseWheelMetadata` (original
  NSEvent phase, momentum phase and timestamp) so finger lift and momentum
  scrolling stay distinguishable.

## Windows

- Only `native: None` on the two `MouseWheel` events so the shared wheel type
  compiles. The backend is otherwise stock.
- Moved to the embedding (no longer patched here): the OLE `IDropTarget`
  destination with virtual files (`FileGroupDescriptor` / `FileContents`) and
  clipboard virtual-file admission now live in
  `apps/rechrom/src/native_drag/windows_target.rs`; the window is created with
  upstream `with_drag_and_drop(false)`. Caps Lock is read with `GetKeyState`
  in `apps/rechrom/src/native_keyboard_modifiers.rs`.

## macOS

- Drag destination with security-scoped `NSURL` file access,
  `NSFilePromiseReceiver` promised files and pasteboard MIME data.
- `ImeTextState` / `set_ime_text_state`: renderer-owned editable text and
  selection for AppKit's synchronous `NSTextInputClient` range queries.
- `KeyEventExtMacOS`: the `doCommandBySelector` selectors, IME consumption and
  inserted character for each key event.
- `CustomCursorExtMacOS`: Retina-scaled custom cursor images and hotspots.
- Application URL-open requests and raw wheel phase metadata.

## Linux

- Wayland: `wl_data_device` drag destination and outgoing drag start
  (`seat/data_device.rs`; the focused seat and live pointer-grab serial are
  exposed through `platform::wayland`). `window/state.rs` differs only in
  formatting.
- X11: XDND destination with selection data, action negotiation and the same
  drag transaction event.

## Other backends

- Orbital and web: only the new event and modifier fields, so the shared types
  compile on every backend.
