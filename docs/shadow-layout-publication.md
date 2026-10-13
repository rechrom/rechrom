# Author shadow layout publication

## Contract and ownership

DOM retains shadow roots and named/manual slot assignment. Its direct
`FlatTreeChildren` view returns a host's root children, an assignment-supporting
slot's ordered assigned nodes (or slotable fallback), and ordinary children
otherwise. It deliberately retains slot elements, including `display:block`
slots, rather than using the recursively flattened script binding result.
`FlatTreeParentElement` remains the single active element-ancestry owner.

The live retained DOM-to-layout projection consumes that view in child attachment
and first-letter discovery. The host or slot stays the inherited-style context;
a synthetic root fragment never becomes a styled layout object. Existing native
ComputedStyle values and LayoutMutation::TreeUpdate remain the canonical boundary.
Initial construction, resident reconciliation, and paint-only export use this
same projection. LayoutObjectTree still owns identity reuse, reparenting,
reordering and removal; Page gains no tree/distribution algorithm.

Shadow attachment and changes to flat-tree position publish document-owned TREE
impact and revoke measurement geometry even when computed styles compare equal.
This covers empty-root attachment, child changes/manual assignment, slot names,
and a slottable element's slot attribute. No-op attributes and identical manual
assignments retain their current revision. Existing Style invalidation still
selects affected distributed hosts; no assignment cache is added.

## Pinned source contracts

Chromium revision `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`:

- [Element::AttachLayoutTree](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/dom/element.cc#4863): host/root, slot, and ordinary-child attachment choice
- [HTMLSlotElement::RecalcFlatTreeChildren](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/html/html_slot_element.cc#362) and `AttachLayoutTreeForSlotChildren` at line 405: ordered active child sequence, preserving slot attachment boundaries
- [Node::FlatTreeParentChanged](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/dom/node.cc#3757) and `RemovedFromFlatTree` at line 3820: style invalidation plus reattachment/detachment independently of CSS equality
- [Element::CreateAndAttachShadowRoot](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/dom/element.cc#6758): removing the old light children from attachment

The corresponding notices are preserved in
`third_party/NOTICE.chromium-shadow-layout-publication`.

## Deliberate integration boundaries

UA controls retain their existing layout adapter ownership. DOM's flat-tree view
is valid for UA roots too, but the current attachment adapter does not newly
traverse them: file inputs already project retained native button/status nodes;
text/search/textarea, range and select have dedicated projections; meter/progress
paint their native tracks and values at the host. Unconditionally traversing UA
roots would duplicate content or theme painting. A complete migration must also
translate the implementation-element appearance/LayoutObjectIsNeeded gates.
This slice makes no claim of complete UA shadow layout parity. A production
meter fixture also exposes a pre-existing unconditional `GetComputedStyle` panic
in the persistent selector collaborator for the UA host-appearance pseudo-class.
That diagnostic is explicitly ignored in the normal suite and can be run
individually; it is not counted as verified meter coverage.

Other pre-existing mechanisms remain separate: ShadowRoot.innerHTML's fragment
parser can create null-namespace nodes; SVG resource resolution by TreeScope;
slotchange/event retargeting and missing generic Element.slot/assignedSlot reflection (tests
use the supported native slot attribute mutation); and the general flex/grid separate-text whitespace
approximation (for example text nodes `A`, ` `, `B`). No site-specific compensation
is used. Screenshot improvement alone does not establish Chromium parity.

## Regression entry points

- `cargo test -p dom --test flat_tree_children --test slot_assignment`
- `cargo test -p browser --test shadow_layout_publication`

The Page fixtures use real QuickJS bindings, native createElement nodes, retained
frames and synchronous geometry reads rather than a separate parsed-document
builder or JavaScript DOM mock.
