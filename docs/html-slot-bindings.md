# Native HTMLSlotElement bindings

The script interface follows Blink's `HTMLSlotElement` IDL at revision
`6c1d401fcca5e1b0030563a90c2f2bba168e0c15`. An HTML-namespace `slot` receives the
native `HTMLSlotElement` prototype. Direct construction remains illegal, including
while another element is being upgraded; methods validate native identity rather
than trusting the mutable prototype or `instanceof` result.

- `name` reads canonical attributes and emits the existing typed attribute mutation
- `assignedNodes` and `assignedElements` return fresh arrays backed by the native
  assignment owner, with Web IDL dictionary/boolean conversion for `flatten`
- DOM owns recursive flattened assignment/fallback collection, translating
  `HTMLSlotElement::CollectFlattenedAssignedNodes`; only assignment-supporting
  slots recurse, and only Element/Text nodes participate
- `assign` validates every variadic Element-or-Text argument first, emits a single
  typed `kAssignSlotNodes` mutation, and uses existing `AssignSlotNodes` ownership,
  deduplication, ordering, previous-owner removal and invalidation
- Mutation payloads carry stable native IDs. Validation occurs before state is
  changed; DOM does not invoke Style/Layout/Paint or perform host I/O

No JavaScript distribution tree or site-specific branch is introduced. The public
binding's flattened sequence is not a layout child list: layout must retain slot
identity for its own style and potential box. Mutation trace names are derived
from actual enum discriminants, including the earlier import kind.

Tests live in `crates/webapi/tests/slot_bindings.rs` and
`crates/dom/tests/slot_assignment.rs`. They cover native brands after prototype
changes, invalid receivers, detached/light/closed/manual roots, first named slot,
comments, fallback nesting, snapshots, argument conversion/atomicity, reflected
name, mutable script projections and typed mutation validation.

## Explicit boundaries

The pre-existing custom-element registry does not implement complete customized
built-in construction/reactions. This slice does not add registry semantics or
slotchange delivery. Existing retained DOM assignment is authoritative; its
existing invalidation path is reused. General shadow layout publication and
synchronous redistribution geometry remain a separate integration task: the
retained projection still visits ordinary DOM children instead of the active
shadow/slot children, with attachment/measurement invalidation and specialized UA
control ownership needing coordinated fixtures. This API completion makes no
claim of complete Shadow DOM or Chromium pixel parity.

## Source links

- [HTMLSlotElement IDL](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/html/html_slot_element.idl)
- [HTMLSlotElement assignment and flattening](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/html/html_slot_element.cc)
- [AssignedNodesOptions IDL](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/html/assigned_nodes_options.idl)
- [V8 HTML constructor](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/bindings/core/v8/v8_html_constructor.cc)
- [SlotAssignment](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/dom/slot_assignment.cc)

Source notices are retained in `third_party/NOTICE.chromium-html-slot-bindings`.
