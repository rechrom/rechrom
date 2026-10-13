# Production fill-layer finalization

The canonical ComputedStyle owns background and mask layer lists. Before Style
publishes its result, the production `AdjustComputedStyle` hook invokes
`ComputedStyleBuilder::AdjustBackgroundLayers` and `AdjustMaskLayers`, before
forced-colors adjustment. This is Chromium's pinned lifecycle at
`6c1d401fcca5e1b0030563a90c2f2bba168e0c15`,
[style_adjuster.cc:1314–1321](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/css/resolver/style_adjuster.cc#1314).

These existing methods cull layers beyond the image list and repeat each shorter
longhand list with the translated FillLayer implementation. The production
resolver previously skipped this step, so a transparent gradient before a URL
image could leave the second image at initial position/repeat/size. Paint and
raster correctly consumed those incorrect inputs. The repair connects existing
owners; it introduces no second geometry algorithm or resource/raster change.
Cold resolution and unadjusted matched-properties-cache hits use the same hook.
Copy-on-write preserves prior published styles and independently cached siblings.

Tests:

- `cargo test -p style --test production_fill_layers -- --test-threads=1`
  covers independent property lists, explicit none, background/mask culling,
  retained mutation, published-style immutability, and identical-style siblings
- `cargo test -p browser --test image_fill_layers -- --test-threads=1`
  covers fetched nested-SVG sprites, negative positioning, native PaintArtifact
  source/destination rectangles, scaled container requests, and exact full-frame
  RGBA equality of a transparent-gradient-plus-image with the single-image control

The independently scoped [SVG image container publication repair](svg-image-container-publication.md)
now covers the 20×40 SVG with a 40×80 background-size through exact resource
publication and blue-crop pixels. Its separate fixes resolve the embedding
scheme before key construction, supply the root container-size input to Layout,
and refresh retained resource snapshots. Fill-layer normalization itself remains
the earlier two-call Style lifecycle repair.

The pinned FillLayer position-copy algorithm is preserved exactly, including
its edge-origin/set-bit behavior when the copy pattern advances into already
filled suffix layers. Three-layer/two-position tests cover authored edge origins;
five-layer tests cover independent numeric patterns without redefining source
semantics. This is a bounded lifecycle repair, not a claim of complete Style,
SVG rendering, or browser pixel parity.
