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
