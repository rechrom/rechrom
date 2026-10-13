# Root fixed-position overflow clip

## Failure and owner

A classic-scrollbar Baidu baseline aborted during composition with
`external-clip-property-mismatch`. Its fixed-looking rounded control had a
1180×757 property clip, but the flat paint wrappers correctly retained the
1165×757 root scrollport clip. This was a baseline correctness failure, separate
from the selector performance optimization.

`PrePaintTreeWalk::ContextForChild` reset a `fixed_to_view` fragment to the full
host viewport properties. It correctly escaped document scroll translation,
but also discarded the root's resolved overflow clip. Paint owns this property
construction from the immutable Layout fragment snapshot. LayerTile's validator
correctly rejected the inconsistent artifact; it is unchanged.

## Pinned Chromium contract

Revision: `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`.

All references below are to
[paint_property_tree_builder.cc](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/paint/paint_property_tree_builder.cc):

- `UpdateOverflowClip`, lines 3284–3297 and 3339–3340, obtains the native
  overflow clip and installs it in the current containing-block context
- `UpdateForChildren`, lines 4362–4373, updates overflow and scroll properties
  before calling `UpdateOutOfFlowContext`
- `UpdateOutOfFlowContext`, lines 3837–3855, copies LayoutView's current context
  for fixed descendants, then restores its pre-scroll transform; it retains
  the current clip and root scroll-chain ownership
- `UpdatePaintOffset`, lines 3975–3984, selects this fixed context
- `SwitchToOOFContext`, lines 387–390, switches the containing-block context;
  effect ancestry is separately maintained in `current_effect`, as shown by
  `UpdateEffect` at line 2356 and filter effect handling at line 2715

The unchanged official source is 214320 bytes, SHA-256
`54ee6d5cd7a6fff2df2feeb9d79a635fe5e61b7ea6bb657e01c7705938f0176c`.
Both size and hash were rechecked against the existing pinned manifest under
`rechrom-evidence/mdn/mask-sources/`. Original attribution is retained in
`third_party/NOTICE.chromium-mask-lifecycle` and `third_party/LICENSE.chromium`.

## Bounded translation

`crates/paint/src/pre_paint_tree_walk.rs` now captures a private root fixed
context from the resolved root child state. It retains the root clip and
pre-scroll transform independently, instead of treating a full host viewport
as the scrollport. It does not inherit an unrelated nested box's overflow clip.
Native `fixed_to_view` remains the gate; fixed boxes contained by transforms
retain their existing geometry path.

Both `UpdateNode` and `UpdateContentInPlace` capture the context before walking
children. The ordinary retained-context comparison therefore sees updated
root clip ancestry after scrollbar add/remove or resize. Fully clean subtrees
continue to use their existing retained snapshots. No DOM, host preference,
network or environment lookup enters Paint; no interface expansion is needed.

The physical root snapshot is not the final logical effect ancestry.
`RestoreLogicalOutOfFlowParents` still invokes `RebindLogicalEffectState` to
restore an out-of-flow node's logical opacity/filter/effect ancestry after the
physical geometry walk. That mechanism and the flat root clip scopes are
unchanged.

## Verification

Three `fixed_control` tests were added to
`crates/browser/tests/root_scrollbar_viewport.rs`. Before the production repair,
all three failed with the same clip mismatch. Afterward, the complete suite
passed: **19 passed, 0 failed**.

The new cases exercise rounded fixed drawing through Page and the real
compositor, both scrollbar axes and overlay controls, fixed pixel stability
after native scrolling, scrollbar removal/reappearance, viewport resize,
escape from unrelated nested overflow, and transformed fixed containing
blocks. Rounded drawing deliberately prevents the rect-only redundant-clip
proof from hiding the property mismatch. The fixture uses injected scrollbar
metrics; production adds no fixed pixel adjustment.

Commands, with the workspace's normal build environment:

```sh
LAYOUTNG_LAYER_REPLAY_DIAGNOSTICS=1 cargo test -p browser --test root_scrollbar_viewport fixed_control -- --nocapture
LAYOUTNG_LAYER_REPLAY_DIAGNOSTICS=1 cargo test -p browser --test root_scrollbar_viewport -- --nocapture
```

Evidence: `rechrom-evidence/paint-timeline/fixed-control-before.log` and
`root-scrollbar-after.log`. Real-page baseline diagnostics are in
`film-before-baidu/runtime.log`, record 59.

Live Release verification also passed: all three `release-final-baidu-{1,2,3}`
trials and the 25-second `film-final-baidu` capture remained alive without a
compositor fatal or clip mismatch. The capture review showed the search
controls, hot list, fixed side widget and subsequently the Baidu logo.
This verifies that the formerly fatal real-page path reaches presentation;
it does not claim that every network request or script finished loading.
First-visible/largest-content timings are reported by the separate performance
investigation. No timing improvement or cross-browser pixel parity is inferred
from these regression tests.
