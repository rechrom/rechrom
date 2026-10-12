# Rechrom

**Rebuilding Chromium. In Rust.**

Rechrom is a modular browser engine for browsers and apps. It rebuilds Chromium concepts piece by piece in Rust, with clear boundaries between DOM, layout, paint, tiling, rendering, and display.

[Website](https://rechrom.dev) · [GitHub](https://github.com/rechrom/rechrom)

> Rechrom is in active development. Version `0.0.1` establishes the engine architecture and a runnable desktop browser; web-platform coverage and public APIs are still evolving.

## Architecture

```text
DOM / CSSOM
    ↓ mutations
LayoutEngine
    ↓ layout result
PaintEngine
    ↓ PaintArtifact
LayerTileEngine
    ↓ FramePlan
Renderer / Display
```

The repository is organized around importable engine crates. [`crates/browser`](crates/browser) is the main integration library. [`apps/rechrom`](apps/rechrom) is the desktop browser shell and stays outside the reusable engine packages.

The current implementation includes:

- a persistent DOM-to-layout pipeline modeled on Chromium's LayoutNG;
- PaintArtifact, retained layer, tile, raster, and composition stages;
- Skia-aligned CPU rendering and native window presentation;
- QuickJS-based JavaScript execution and browser APIs;
- an F12 Performance panel for input, frame scheduling, raster, composition, and presentation timing.

## Build

The project uses the Rust toolchain pinned in [`rust-toolchain.toml`](rust-toolchain.toml).

```bash
cargo build --release -p rechrom_app
cargo run --release -p rechrom_app -- https://rechrom.dev
```

### Linux prerequisites

Install the pinned Rust toolchain through rustup and the platform development
libraries. For Debian/Ubuntu:

```bash
sudo apt-get install build-essential pkg-config libharfbuzz-dev libicu-dev \
  libfontconfig-dev libfreetype-dev libx11-dev libxkbcommon-dev libwayland-dev
cargo build --locked -p rechrom_app
cargo run --locked -p rechrom_app -- --url https://rechrom.dev
```

A visible window needs an X11 or Wayland session; use `--headless` below on
servers without a display. Rust registry dependencies still need downloading
on the first build.

The translated foundation primitives are owned by `crates/foundation/base`;
`crates/foundation` preserves the higher-level foundation interfaces. The
Roboto fallback/test font is bundled in `assets/fonts` with its license.
Linux and macOS use the internal Rust Skia CPU renderer. Text shaping uses
`rustybuzz`; OpenType metadata, glyph metrics, math and Unicode helpers still
use native HarfBuzz. Neither normal application build needs a sibling source checkout or
C++ Skia library. On macOS the translated Rust scaler calls Apple's system
CoreText/CoreGraphics APIs directly; that platform FFI is not a C++ Skia bridge.
System HarfBuzz and ICU development libraries are still required, so the whole
browser is not yet free of native C/C++ dependencies. macOS also needs Xcode
command-line tools (`brew install pkg-config harfbuzz icu4c` for dependencies).

Only explicit reference-oracle features (`raster/text`, `raster/font_backend`,
`raster/source_replay`, `browser/native_text`, `browser/source_png`, and
`rechrom_app/profile_native`) compile the retained C++ comparison bridge.
These are macOS-only verification tools, not production renderer switches.
They require `RECHROM_REFERENCE_SKIA_DIR` pointing to a Skia checkout with
headers, `lib/libskia.a`, and `lib/libskcms.a`; no sibling checkout is selected
implicitly. Some optional macOS Skia tests also need reference fixtures under
`../src/third_party/skia/resources`. Native comparison sources are retained to
check the Rust translation and are excluded from normal builds.

For a displayless browser process using the same Page, Compositor, Viz and
Renderer pipeline:

```bash
cargo run -p rechrom_app -- --headless --url https://www.baidu.com/ \
  --width 1280 --height 713 --devtools-port 9223 --exit-after 60
```

Headless mode needs no window or display server. Width and height describe the
page's CSS viewport at device-pixel ratio 1; toolbar pixels are excluded from
page screenshots. The existing localhost DevTools endpoint accepts HTTP JSON
requests at `/api`, including `Page.captureScreenshot`; it is not the full CDP
WebSocket transport. A screenshot is available only after a completed frame,
and may precede completion of network loading.

HTTPS keeps certificate and hostname verification enabled. When the host sets
`SSL_CERT_FILE`, its PEM certificates augment the public root set. An unreadable,
empty or invalid configured file is reported as an error rather than ignored.

On macOS, create an application bundle with:

```bash
apps/rechrom/package_macos.sh --release
open "target/Rechrom.app"
```

To build only the reusable integration crate:

```bash
cargo build -p browser
```

## Repository layout

```text
apps/rechrom/       desktop browser shell
crates/browser/     public engine integration crate
crates/dom/         DOM ownership and mutation routing
crates/layoutng_*/  layout algorithms and fragment output
crates/paint/       pre-paint state and PaintArtifact generation
crates/layer_tile/  retained layers, tiles, and frame planning
crates/renderer/    raster and composition
crates/devtools/    local Performance protocol and UI
vendor/             patched build dependencies
```

## Status

Rechrom `0.0.1` is an early development release. It is suitable for engine work, architecture experiments, and running the included desktop shell. It is not yet a general-purpose replacement for Chromium.
