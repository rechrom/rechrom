# smithay-clipboard

Vendored Rust sources from Smithay/smithay-clipboard tag v0.7.2:
https://github.com/Smithay/smithay-clipboard/tree/v0.7.2

License: MIT (see LICENSE). The manifest omits upstream example-only dev
dependencies. Used by the Linux embedding on winit's retained display
connection. Local changes add MIME snapshots and acknowledged standard
clipboard writes, retain bytes per native source until cancellation, and
remember the focused seat on keyboard enter. PRIMARY keeps its separate
upstream text interface. The MIME contract follows Chromium's
`ui/base/clipboard/clipboard_ozone.cc` and native offer/source lifetime follows
`ui/ozone/platform/wayland/host/wayland_clipboard.cc`.
