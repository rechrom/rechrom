# Root scrollbar ownership and viewport geometry

## Contract

The application supplies `ConstraintSpace::scrollbar_theme` and CSS-pixel viewport
geometry. `ScrollbarThemeMetrics` is the existing typed, immutable host boundary:
normal/thin thickness, overlay policy, thumb/button sizes and colors. Layout never
queries environment variables, platform APIs, browser settings or website names.

The existing default in `CreateBrowserConstraints` remains overlay with configured
normal/thin thicknesses 15/7. The application can explicitly select classic or
overlay policy with `RECHROM_SCROLLBAR_MODE=classic|overlay`. This does not discover
OS preferences and is not a universal scrollbar width. Embedders may inject other
metric values directly; regression cases use 23/11 as well as 15/7. An invalid host
mode is rejected. The MDN comparison selects classic to match the measured reference.

Host available size, `window.innerWidth/innerHeight`, media-query geometry and
ordinary viewport units remain full-sized. Native viewport metadata is converted
once into layout-zoom coordinates, independently of device scale. The native root
fragment reserves scrollbar struts inside that full-size viewport. No pre-shrunk
page width, DOM offset, site branch or replacement scrollbar solver is involved.

## Native owners and the merged-root boundary

Chromium's LayoutView is separate from the document element and has propagated
viewport style. Rechrom's existing extracted root combines those identities while
retaining the document element's canonical native ComputedStyle. The repair adapts
the existing view gates to viewport semantics without overwriting authored style:

- LayoutView is a viewport scroll owner, including authored visible overflow
- Viewport scroll modes dispatch through `LayoutView::CalculateScrollbarModes`
- A propagated element cannot own a second local scrollport; the merged view is
  exempt from the descendant-only suppression
- Native tree relation finalization chooses the primary body when html overflow
  is visible on both axes and neither element applies containment; changed owner
  identity refreshes clipping and area ownership through native UpdateFromStyle
- Root scrollbar-width/gutter remain supplied by html even when body supplies
  overflow; viewport visible/clip mapping applies to root gutters
- The existing PaintLayerScrollableArea computes add/remove decisions and the
  existing BlockNode loop relayouts and freezes axes until scrollbar struts settle
- Native intrinsic inline contributions are invalidated when scrollbar changes
  affect the inline axis, before the existing relayout loop

A host resize explicitly invalidates document layout when viewport geometry changes,
even if computed CSS is identical. Equal-size resize retains the published layout.
This closes a lifecycle admission hole; it does not add a Page-side layout algorithm.

## Immutable client dimensions

`PaintProperties::client_size` carries optional native CSSOM client geometry.
The root uses `LayoutView::GetLayoutSize(kExcludeScrollbars)`; ordinary boxes use
`PhysicalPaddingBoxRect`, retaining Chromium's table border-box exception.
Browser's existing FragmentMetric projects those dimensions and adjusts zoom.
The optional fallback preserves non-native synthetic fragment callers. Paint's
existing FragmentNode content_size meaning is unchanged; client-only metadata
is explicitly excluded from paint-record equality.

## Root canvas clip parity

The existing root canvas selects contents properties for scrolling backgrounds.
With a classic gutter those properties clip at the native client edge, while the
old flat-record wrapper remained full viewport width. The compositor correctly
rejected that disagreement. Scrolling canvas color/image records now enter the
existing escapable geometry before recording and leave it afterward, matching
Chromium ViewPainter’s contents-property scope. Fixed backgrounds keep their
border-box state; mixed-background isolation stays outside the per-record clips.
No compositor validator or property-tree tolerance was changed.

## Verification and boundaries

Sixteen production Page tests cover classic/overlay/no overflow, both axes and
cross-axis induction, hidden/forced/thin/none, stable/both-edge gutters, add/remove
mutation and resize, no-op resize, body propagation and containment, arbitrary
injected metrics, page zoom 2/0.8, wrapping reflow, nested client sizes and retained
old snapshots. Canvas tests cover solid, scrolling-gradient, fixed-gradient and
mixed fixed/scrolling layers in both scrollbar modes, before and after wheel
scroll. The first classic vertical fixture and classic canvas compositor fixture
fail before their respective repairs. Scroll image pixel-position parity is not
claimed by the clip-contract tests.

This bounded slice does not split the merged HTML/LayoutView, implement quirks-body
CSSOM special casing, discover live OS theme preferences, implement native overlay
fade/interaction, visual-viewport/iframe scrollbar ownership, autosizing/printing,
or enable the separate unconditional-scrollbar viewport-unit feature. Those larger
branches must not be inferred from the standards-mode root tests. Custom scrollbar
style behavior remains at the pre-existing capability boundary. The pre-existing
merged-root viewport border-radius behavior is also outside this slice.

## Official source

Pinned Chromium revision `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`:

- [Viewport style propagation](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/css/resolver/style_resolver.cc#3626)
- [Document viewport-defining element](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/dom/document.cc#4212)
- [Scrollbar existence and removal](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/paint/paint_layer_scrollable_area.cc#1740)
- [Existing relayout and freeze loop](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/layout/block_node.cc#553)
- [Layout viewport geometry](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/layout/layout_view.cc#817)
- [Canvas contents-property recording](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/paint/view_painter.cc#151)
- [Client dimensions](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/dom/element.cc#2541)
- [Equal-size resize admission](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/frame/local_frame_view.cc#4075)

Original source headers and hashes are retained in
`third_party/NOTICE.chromium-root-scrollbars`. Source-specific LGPL/Apple notices
remain applicable and are not replaced by Chromium's root BSD license.
