# Exact SVG image container publication

The selected SVG image container is an input to isolated document layout, not
an instruction to stretch a natural-size recording. Resource identity remains
the fetched image id plus the complete `ContainerKey`: concrete viewport, zoom,
DPR, URL view and resolved embedding color scheme.

## Source-aligned boundary

This repair follows Chromium revision
`6c1d401fcca5e1b0030563a90c2f2bba168e0c15`:

- [StyleFetchedImage::GetImage](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/style/style_fetched_image.cc#230)
  resolves the embedding color scheme before handing the container to drawing
- [SVGImage::PaintRecordForCurrentFrame](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/svg/graphics/svg_image.cc#455)
  sets LayoutSVGRoot's container size before the isolated lifecycle update
- [LayoutSVGRoot::SetContainerSize](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/layout/svg/layout_svg_root.h#84)
  invalidates layout and full paint only when this input changes
- [LayoutSVGRoot::ViewportSize](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/layout/svg/layout_svg_root.cc#487)
  is an output of current layout, distinct from the container override

Page owns the host preference and projects it through immutable ConstraintSpace
and PaintResources inputs. Paint's existing image-container producer resolves
normal style against that preference; an explicit color-scheme uses the
canonical style's used scheme. The SVG pattern-image producer shares this same
key constructor. No consumer infers its environment from the resource's last
published variant. Page fulfills the exact requirements, including reflection
masks stored on effect nodes. Raster retains strict exact lookup.

DocumentImage supplies a typed `DocumentImageContainer { root_node_id, size }`
input to Layout. The root id identifies the isolated SVG root despite the
current inert HTML parser wrapper. The layout assembly routes it through an SVG
setter callback. Only that root gets an override; other roots get an empty one.
The existing replaced-size algorithm consumes the override before ordinary
inline/document-element fallback sizing. No DOM roles, CSS declarations, source
assets or source/destination rectangles are changed.

LayoutSVGRoot now retains the container input separately from its actual
viewport output. ViewBox transforms and percentage viewport lengths consume
normal layout geometry. Fixed child units without a viewBox remain fixed.
DocumentImage publishes an immutable record genuinely laid out at that size;
raster's existing intrinsic-source-to-container mapping remains unchanged.

## Retention and publication

Paint-only fragment export can carry an older resource catalog. Before recording,
Page installs the current immutable catalog even when all required variants are
already retained. Replacing a certified fragment's resource inputs clears its
PrePaint revision certificates, forcing retained nodes and reflection payloads
to consume the new inputs. This is invalidation, not a second resource cache or
a fabricated revision. Previously published artifacts retain their old Arcs.

This closes three observed failures in the same contract:

1. A normal-style key stayed unresolved in Paint but became resolved on publication
2. The selected viewport never reached the SVG root's existing sizing override
3. Retained fragment/PrePaint snapshots could keep older resources on resize-back
   or preference changes

## Verification and limits

`cargo test -p browser --test image_fill_layers -- --test-threads=1` covers
single/multi-layer sprite pixels, scaled and repeated tiles, simultaneous sizes
sharing one image id, mutation and resize-back, immutable old-frame pixels,
initial and changed host preferences with per-element overrides, fixed SVG units,
percentage nested viewports, and reflection-mask exact resource publication.
Reflection's downstream property-clip raster path is independently unsupported;
that test does not claim reflected pixels pass.

`cargo test -p paint --features translation_in_progress -- --test-threads=1`
also covers pattern-image key construction with zoom, DPR, view identity and
normal/explicit embedding schemes. An end-to-end inline pattern-image fixture
currently emits no document-image requirement, so that separate discovery/paint
path is not claimed verified here.

Fractional container residuals, arbitrary cropped-source origins, percentage
shape attributes, and complete browser pixel parity remain separate work.
