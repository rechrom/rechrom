# Connected shadow image resources

## Ownership and source alignment

Pinned Chromium revision: `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`.

Chromium resolves pending resources from each image/style consumer, including
native pseudo styles reached inside shadow roots. Rechrom retains its existing
`ResourceFetcher` adapter: Page assembles the typed resource/DOM/Style pipeline;
Style supplies native `StyleImage` dependencies; Document owns resource identity;
DocumentImage owns isolated SVG lifecycle; Paint/Raster consume immutable frames.
No core network I/O, SVG rewrite, fallback bitmap or second resource cache is added.

The adapter previously recursed through ordinary DOM children only. Those
children deliberately exclude shadow roots. A correctly styled, visible shadow
pseudo could therefore retain a pending SVG mask forever because no request was
admitted. The mask paint behavior was correct to suppress its pixels.

The existing discovery and completion walkers now include `ShadowRoot` before
ordinary `Children`, and completion's existing ancestor pruning index uses Parent
or ShadowHost. These three pieces agree on shadow-inclusive tree order. Ordinary
DOM Parent/Children and flat-tree slot ownership remain unchanged. Closed/nested
roots participate; slotted light nodes are visited once; unassigned light image
elements remain resource consumers. Completion still snapshots children after
each callback, preserving its existing mutation/checkpoint lifecycle.

Official owners:
- [Shadow style and pseudo recalc](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/dom/element.cc#5417)
- [Style pending resources](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/css/resolver/style_resolver.cc#1501)
- [Native mask resource loading](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/css/resolver/element_style_resources.cc#374)
- [Shadow-including tree order](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/dom/shadow_including_tree_order_traversal.h#57)
- [Image owner completion](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/loader/image_loader.cc#972)

Chromium does not literally perform this adapter's global discovery walk; its
per-consumer loading model supplies the behavior the existing boundary preserves.
The traversal utility is evidence for tree relation/order, not a new source claim
about where Chromium initiates image loads.

## Verification

`browser/tests/svg_mask_graphics.rs` exercises actual production raster pixels:
- Static stroked SVG mask, image, and inline controls paint real graphics
- A connected shadow pseudo mask fails before with no image request and zero ink
- Fixed shadow mask produces 172 graphic pixels, with a loaded canonical native
  resource and exact 24x24 equality to the ordinary mask control
- Closed nested roots, URL mutation, immutable old paint artifacts, shadow
  backgrounds, detached CSS waiting for attachment, and shared URL deduplication
- Shadow and slotted/unassigned light image load events occur once in shadow-
  inclusive order; failed shadow image dispatches one error event

The generic asset has a path plus circle and inherited currentColor stroke.
No test substitutes screenshots for SVG content or changes the renderer.

## Explicit limits

This is a bounded completion of connected consumer coverage. It does not implement
Chromium's full detached ImageLoader lifetime model, nor add display:none or
computed-style-only resource admission guards. Existing loading/CORS and URL
identity policies are unchanged. No claim is made that every shadow API is now
complete: direct slot IDL assignment is an existing separate binding limitation,
and fixtures use the actual name/slot attributes. The browser's pseudo computed-
style projection and FontFaceSet stub are not independent paint/font evidence.

Live-page evidence verifies MDN's search/theme/language icons and ad artwork now
appear. It is not complete cross-browser parity: the reference screenshot is a
scaled JPEG, and mandala, ad geometry and subpixel inner-column differences remain.
