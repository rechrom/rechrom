# Local softbuffer 0.4.8 changes

Upstream library sources and MIT / Apache-2.0 licenses are retained. Cargo.toml.orig retains the upstream manifest. The local normalized manifest drops example/benchmark-only dev dependencies; this vendored copy is used as a library.

The opt-in macOS `native-iosurface` feature uses objc2-io-surface Rust bindings inside the existing Core Animation backend. Window / NSView / layer ownership and resize observers remain the softbuffer implementation. Default backends retain upstream behavior.

CPU targets use a bounded four-slot IOSurface pool, exclude the most recently submitted resource and resources reported in use by WindowServer, and lease slots until CPU mappings are released. Lock, full paint, unlock and CALayer.contents submission share the same retained resource. Storage is premultiplied BGRA with row and allocation sizes aligned by IOSurfaceAlignProperty; acquire validates dimensions, pitch and alignment. Buffer::pixel_format distinguishes this from the default XRGB encoding. Buffer::row_stride exposes the actual u32 row pitch, including padding; visible width remains unchanged. Default backends report their packed width as the stride.

The previous width*4 row pitch created CPU-readable IOSurfaces which Core Animation could not display at some window widths. The renderer now wraps the padded mapping directly and clips to the visible viewport, preserving the same mapping through presentation without a row-copy pass. Browser toolbar/content partitions also use the acquired stride.

Ownership references: Chromium components/viz/service/display_embedder/software_output_device_mac.cc and ui/accelerated_widget_mac/display_ca_layer_tree.mm. This is a restricted Rust adapter; no claim of complete Chromium compositor parity or no copies inside the operating system.

Regression tests cover odd widths, simultaneous mapping isolation, buffer reuse, submitted-buffer exclusion, resizing, bounded slots and invalid dimensions. Library tests can run with cargo test --manifest-path vendor/softbuffer/Cargo.toml --no-default-features --features native-iosurface --lib. Cargo's production patch is configured by the main workspace.
