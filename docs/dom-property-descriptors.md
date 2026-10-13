# Ordinary DOM property descriptors

The ordinary DOM-node binding surface follows Chromium/Blink revision
`6c1d401fcca5e1b0030563a90c2f2bba168e0c15`. This is a property-dispatch repair,
not a replacement DOM implementation or a claim of complete Web IDL support.

## Property resolution and native identity

After the Node prototype has been registered, `DOMJavaScriptBindings` declines
ordinary native-node Get/Set projection. `PrepareInvocation` also declines those
raw writes, so it cannot prepare a mutation before JavaScript property lookup.
The existing QuickJS wrapper then uses its retained `Reflect.get`/`Reflect.set`
functions with the original receiver. Its Has fallback performs ordinary
membership lookup without invoking a getter.

Consequently own descriptors and nearer author prototypes can shadow native
properties. Removing or replacing a configurable native descriptor affects
subsequent lookup; an absent property is not recreated by a native name switch.
Getter-only and nonwritable descriptors retain engine-owned rejection behavior.
Author accessor exceptions follow the existing JavaScript exception path,
including thrown-value identity, without a generic catch-and-ignore fallback.
The pre-bootstrap raw host protocol remains available to existing embedders.

`dom_webidl.js` installs enumerable, configurable native accessors on the
applicable interface prototypes. The selected accessor calls the existing
`__domInvoke` bridge with its declaring interface, member and actual receiver.
`dom_attributes.rs` validates native node identity, kind, namespace, tag or
shadow-root identity. JavaScript `instanceof` and prototype membership do not
establish a native brand. A genuine input still passes the input brand after
prototype replacement; a forged prototype does not. Input, textarea and select
value accessors have separate declaring-interface checks.

Setters validate the receiver before JavaScript value conversion. Converters
distinguish boolean, DOMString, nullable string and legacy null-to-empty-string
cases; DOMString conversion rejects Symbol and propagates conversion failures.
The native bridge checks the interface again and reuses the existing native
getter or setter. An unimplemented selected native attribute is an explicit
error rather than a request for expando fallback.

## Existing DOM ownership and reaction scheduling

The retained DOM remains the sole owner of node identity, attributes, control
state and tree mutation. The bridge emits the existing typed `DOMMutation`
through the existing scoped continuation, after releasing native borrows.
`browser::Page` continues to assemble the downstream lifecycle; no new DOM,
style, layout, rendering or presentation owner is introduced.

CEReactions metadata accompanies the selected accessor. Applicable setters use
the existing native reaction scope and callback drain; `checked`,
`indeterminate` and CharacterData `data` do not acquire an artificial scope merely
because they pass through the same bridge. This change does not duplicate the
custom-element registry, mutation logic or reaction queues.

Methods previously supplied by the removed ordinary-node projection are
installed on their applicable prototypes, including selector and attribute
methods. Existing EventTarget methods retain their prototype ownership.
ParentNode members are installed separately on Element, Document and
DocumentFragment; `getElementById` is on Document and DocumentFragment.
`innerHTML` is on Element and ShadowRoot, not generic DocumentFragment.
Event-handler descriptors use their applicable element/document interfaces,
including the lenient receiver behavior of Document `onreadystatechange`.

## Explicit boundaries

- Window-owned Document `URL`, `documentURI`, `cookie` and `domain` still preempt
  ordinary descriptor dispatch. Window/Location navigation, global/security and
  unforgeable behavior is outside this slice
- Collections, iterator records, bound Event objects and other genuine host
  projections retain their existing dispatch paths
- Existing JavaScript-only descriptor implementations and stubs are not all
  IDL-complete. Installing the migrated native descriptors does not establish
  full interface, conversion, security or extended-attribute conformance
- Document `body` and Element `outerHTML` have writable accessor shapes, but
  their native setter backends remain unsupported and explicitly reject writes
  instead of falling back to an expando
- Trusted Types and other previously unsupported DOM semantics are not added
  by this dispatch change

## Source chain and attribution

The binding generator selects member ownership, descriptor flags and receiver
checking. The installer creates signature-checked accessor functions on the
selected prototype:

- [Generator member location and flags](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/bindings/scripts/bind_gen/interface.py#4467)
- [IDLMemberInstaller attribute installation](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/platform/bindings/idl_member_installer.cc#202)
- [Native boolean conversion](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/bindings/core/v8/native_value_traits_impl.h#141)
- [Node IDL](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/dom/node.idl)
- [CharacterData IDL](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/dom/character_data.idl)
- [Element IDL](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/dom/element.idl)
- [HTMLElement IDL](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/html/html_element.idl)
- [Input IDL](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/html/forms/html_input_element.idl)
- [Textarea IDL](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/html/forms/html_text_area_element.idl)
- [Select IDL](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/html/forms/html_select_element.idl)

Ordinary lookup and receiver propagation follow
[ECMAScript OrdinarySetWithOwnDescriptor](https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-ordinarysetwithowndescriptor)
and [Web IDL attributes](https://webidl.spec.whatwg.org/#es-attributes).
The existing engine translation follows the vendored
[QuickJS 2026-06-04 release](https://bellard.org/quickjs/quickjs-2026-06-04.tar.xz);
its property read/write, Reflect and Proxy implementations continue to own those
language semantics.

`third_party/NOTICE.chromium-dom-property-descriptors` retains the original
source notices, pinned URLs and source-file SHA-256 hashes. The existing
`LICENSE.chromium`, `LICENSE-LGPL-2` and `LICENSE-LGPL-2.1` texts apply as identified
by each source file. The root Chromium BSD license does not replace source-specific
LGPL terms or relicense the remainder of Rechrom.

## Verification scope

`cargo test -p webapi --test host_property_dispatch` exercises author/native
shadowing, property ownership, receiver brands, conversion, distinct Reflect
receivers, deletion/replacement, membership, readonly rejection and exception
identity. Production Page and broader DOM/custom-element/rendering checks are
tracked separately. This document records the implementation contract and
remaining boundaries; it does not certify final test results or visual parity.
