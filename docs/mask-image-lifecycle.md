# Fetched mask-image resource lifecycle

Reference revision: `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`.
All upstream paths below are under
https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/ .

## Source contract

- `core/style/style_fetched_image.cc:107–120` forwards resource queries independently
- `core/loader/resource/image_resource_content.cc:681–692` treats failure as terminal: loaded=true, loading=false, error=true, renderable=false
- `core/style/fill_layer.cc:466–496` aggregates pending state with `!IsLoaded && !ErrorOccurred`; all-none is not an active mask
- `core/paint/paint_property_tree_builder.cc:2187–2209` sets effect opacity to zero while any mask image is non-terminal; it also hides backdrop-filter effects when all mask images are invalid
- `core/paint/box_painter_base.cc:58–84,624–690,1286–1290,1352–1388,1777–1795` keeps native layer order and skips non-renderable image draws before selecting their composite operator; only the original bottom layer ignores its operation

## Ownership

ResourceFetcher already owns request/decode results and knows both the transport
URL and original document source binding. `ResourceLoadFailed.source` carries
that binding to Page for image failures. Page routes the fact to Document,
which keeps a single Loaded/Failed entry per source and invalidates its actual
Style consumers, including pseudo-elements. Absence is the current pipeline's
unpublished/pending state. A stale failure cannot revoke a published loaded
image; duplicate loaded publication still fails. No retry policy was added.

Style rebuilds immutable native `StyleFetchedImage` snapshots with terminal
error state. Failed URL images retain URL/CSS identity but publish no renderable
resource ID. Existing retained LayoutObject updates keep object identity;
previous frame artifacts retain their old style/resource/pixel state.

Layout's existing native paint-style adapter freezes `PaintMaskImageState`
(ready/loading/invalid) alongside each mask layer. A single/all-none chain is
not a mask. A none slot within an active chain stays a non-drawing slot, so it
cannot change which native layer is bottommost. Paint resolves only ready
resources, preserving strict missing/invalid-resource assertions for those
identities. It creates no substitute resource.

Paint owns temporary suppression in the effect property tree. The same helper
is used for flat effect wrappers and retained logical-effect rebinds, keeping
the property/command consistency validator strict. Invalid layers still
establish the mask effect, but provide no image drawing. Raster skips their
empty slots and determines bottom membership from the original index. Tile
lowering admits this zero-source slot while retaining all geometry validation.

No CSS reparsing, site condition, host I/O, cross-engine callbacks, or parallel
resource cache was introduced. Browser Page remains an assembly root; Style,
Layout, Paint, tile lowering and raster consume typed inputs.

## Important composite detail

For terminal failed ordinary images or none, let U be the non-drawing slot and
M a valid mask. Source-derived results at this revision:

- `U,M`: M for add/subtract/intersect/exclude, because U issues no operation
- `M,U`: M for add/subtract/exclude; empty for intersect
- One failed fetched image: empty mask, hiding its content
- One valid image: M, regardless of its bottom operator
- Only none: no masking

A failed/none top slot is not an explicit transparent image draw. Compacting
slots or synthesizing that draw would each change Chromium's non-default
operator behavior. Tests encode this pinned source behavior; these are not
independent Chromium browser-oracle pixel comparisons.

## Verification and scope

`crates/browser/tests/mask_image_lifecycle.rs` exercises production Page with
controlled loading, fetch error, decode error and successful SVG content.
It checks whole-element pending suppression, success/error transitions,
pseudo-consumer refresh, absence of fake resources, all four composite
operators and top/bottom none/failed slots, retained native identity and
immutable prior-frame pixels. StyleFetchedImage also tests query and CSS-value
identity. Document targeted-consumer tests are present; whether the complete
DOM unit harness compiles is recorded separately in task evidence.

The patch covers ordinary fetched images and the states the current complete-
decode resource pipeline publishes. Partial progressive images, SVG `<mask>`
reference resolution, broader mask geometry, and unrelated page failures are
not made complete by this work. A source-aligned fix is not a full-page visual
parity claim.
