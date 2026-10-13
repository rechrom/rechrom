# DOM template primitives

`Element.hasAttributes()`, `Element.getAttributeNames()` and
`Document.importNode()` follow the owning DOM methods and Web IDL in Chromium
revision `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`:

- [Element IDL](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/dom/element.idl)
- [Element attribute storage/view](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/dom/element.cc)
- [Document IDL](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/dom/document.idl)
- [Document::importNode](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/dom/document.cc)
- [Node cloning](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/dom/node.cc)
- [Template cloning](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/html/html_template_element.cc)

Source-specific notices are retained in
`third_party/NOTICE.chromium-dom-template-primitives` and the existing
Chromium/Blink license files. This change does not relicense other code.

## Ownership and interfaces

The native attribute vector stays canonical. Attribute presence reads that
vector; names are copied in storage order with namespace prefixes intact and
cross the existing immutable `HostValue::Record` boundary. Web IDL materializes
a fresh ordinary array. Neither method uses an overridable public `attributes`
property or a persistent mirror of DOM data.

Import uses one typed `kImportNode` mutation, carrying the source node ID,
destination document ID, deep flag and expected allocated root ID. Validation
checks all identities before allocation. A shared DOM-owned clone helper is
parameterized by destination ownership; `cloneNode` retains source ownership.
Template content and regenerated UA shadow-tree roots are assigned ownership
before descendant creation. No HTML serialization/reparse, temporary insertion,
JavaScript tree-copy algorithm or renderer call is involved.

The binding checks native brands, including when genuine nodes have modified
JavaScript prototype chains. Public Document and ShadowRoot imports raise
`DOMException` with `NotSupportedError`; incompatible receiver/source values
raise TypeError. Boolean/default and `ImportNodeOptions.selfOnly` conversion
are covered. Imports return a detached new root without altering the source.

## Established scope and remaining limits

This completes the three missing primitives for ordinary nodes, descendants,
fragments and the existing retained template model. It is not a claim of full
Blink DOM/custom-element conformance:

- The existing template model has no separate inert template document;
  template content uses its retained owning document
- Explicit custom-element registry selection is rejected with a clear
  NotSupportedError until destination/scoped registry ownership is implemented
- Existing import/insertion custom-element reactions and clonable author shadow
  roots are not completed by this slice
- Existing DOM node-type, qualified element-name and doctype metadata limits
  are not broadened by reusing the clone engine

The observed MDN Lit stacks directly identified `hasAttributes` and
`importNode`; `getAttributeNames` was the immediately subsequent template call.
After implementation those first failures disappeared and search-related
shadow DOM was generated. Search remained visually absent, and later dropdown
execution exposed a separate `HTMLSlotElement` ReferenceError. Neither later
problem is claimed fixed here.

## Focused verification

`cargo test -p webapi --test dom_template_primitives` covers attribute order,
namespace prefixes, snapshot identity, wrapper overrides, deep/shallow imports,
source preservation, detached ownership, nested templates/fragments, options,
exceptions and prototype-mutated native brands.

`cargo test -p dom --test import_node` covers mutation validation before
allocation and imported control/UA-shadow destination ownership. Existing
browser SVG/mask/container/multicol/button regressions are rerun separately.
