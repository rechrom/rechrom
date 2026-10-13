# Native custom-element definitions and reactions

This bounded autonomous-custom-element slice follows Chromium revision
`6c1d401fcca5e1b0030563a90c2f2bba168e0c15`. Focused native, binding and Page
regressions pass. Broader suites preserve their baseline failure identities, and
the real-page check establishes partial style delivery, not complete Custom Elements
support or Chromium parity.

## Ownership and production connection

`crates/dom/src/custom_elements.rs` owns registry definitions, opaque definition
identities, descriptors, observed-attribute metadata, element custom state,
candidate eligibility and native reaction queues. `PersistentDocument` connects
that owner to native element creation, attribute changes, attachment, removal,
and author-shadow traversal. Native identity remains authoritative after script
changes a wrapper's prototype.

`crates/webapi` retains constructor and captured lifecycle-function handles keyed
by definition identity. Its bindings perform Web IDL conversion, author getter
execution, constructor execution, promise resolution and exception reporting.
DOM receives typed metadata and constructor outcomes, not JavaScript objects.
DOM neither calls back into downstream rendering engines nor owns a second
script-visible tree. The existing Page assembly and DOM invalidation route carry
ordinary construction and callback mutations onward.

The old JavaScript registry map and delayed `querySelectorAll` upgrade scan are
not authoritative. The native registry/candidate path must be reached by the
public bindings. Framework-specific finalization hooks and site-name branches
are outside this contract.

## Definition transaction

Web IDL converts the name, callback argument and options dictionary before
Blink's native definition algorithm. `ElementDefinitionOptions.extends` is a
nullable DOMString with a null default. An options getter can execute author
code before the definition-running guard is entered.

After those conversions, preserve this validation order:

1. Require a constructible constructor, without invoking it or reading its
   prototype merely to test constructibility
2. Validate the exact case-sensitive custom-element name
3. Reject an already registered name, then an already registered constructor
4. Validate any `extends` descriptor
5. Reject an already-running definition in the same registry
6. Enter the guard, read and convert the original properties below, and always
   leave the guard on success or exception
7. Publish both definition indexes only after successful extraction; enqueue
   connected native upgrade candidates before resolving `whenDefined` waiters

Property extraction uses ordinary inherited Get semantics in this order:

- `constructor.prototype`, which must be an object
- `connectedCallback`, `disconnectedCallback`, `connectedMoveCallback`,
  `adoptedCallback`, `attributeChangedCallback` on that prototype
- `constructor.observedAttributes`, only when an attribute callback was captured
- `constructor.disabledFeatures`
- `constructor.formAssociated`
- If form-associated is truthy, the prototype's `formAssociatedCallback`,
  `formResetCallback`, `formDisabledCallback`, `formStateRestoreCallback`,
  `toolFillCallback`

Lifecycle values may be functions or undefined. Null and every other
non-function value throw TypeError. An abrupt getter/conversion exception is
preserved, later getters are not read, and no name or constructor is reserved.
Captured callback identities and the observed-name snapshot do not change when
script later replaces prototype methods, instance methods, static properties,
or the original observed-attributes array.

Undefined sequence metadata is empty. Other sequence values require Web IDL
`sequence<DOMString>` conversion, not permissive `Array.from` coercion: primitive
strings, null and plain arraylike objects are invalid, iterable objects are
accepted, and item conversion may itself throw. Convert all items before
deduplicating observed names. DOMString conversion preserves case and rejects
Symbols. The pinned Blink actual-array fast path ignores a custom iterator,
reads numeric indices, and reevaluates array length during iteration; generic
objects follow iterator conversion. V8's `Value::IsArray` tests native JSArray
identity, so a Proxy-wrapped array follows the generic iterator path and can
observe a custom iterator. JavaScript `Array.isArray` follows Proxy targets and
is not an equivalent actual-array predicate. The binding distinguishes native
proxies before taking the array fast path; the focused conversion regressions
pass.

The pinned generic iterator helper caches `next` once, reads each result's
`value` before `done` (including the terminal result), and does not close the
iterator when an item's DOMString conversion fails. An ordinary JavaScript
for-of loop has different observable getter/closing behavior, so it is not an
exact translation of this pinned helper. The binding implements the explicit
iterator path and its focused getter-order, closing and Proxy regressions pass.

`formAssociated` uses ordinary boolean conversion. The two recognized
`disabledFeatures` strings are exactly `shadow` and `internals`; unknown strings
are ignored upstream. The unsupported-feature boundary below is explicit rather
than pretending these flags are enforced.

### Exact name rules at the pinned revision

The source uses the newer valid-element-local-name rule, not the old PCEN regex.
Names begin with ASCII lowercase, contain a hyphen after index zero, and contain
no ASCII uppercase. After the initial lowercase letter, the local-name check
rejects NUL, TAB, LF, FF, CR, SPACE, slash and greater-than. Punctuation such as
colon and equals is allowed. The reserved names are `annotation-xml`,
`color-profile`, `font-face`, `font-face-src`, `font-face-uri`,
`font-face-format`, `font-face-name`, and `missing-glyph`.

`get` does exact lookup; `getName` uses constructor identity. Invalid names passed
to `whenDefined` reject its promise. A pending name reuses its waiter promise.
Erase a resolved waiter before resolving with the constructor, because reading
a constructor's `then` property during promise resolution can execute author
code after the definition has been published.

## Native upgrade and reaction ordering

Definitions match native HTML namespace/local name and, for customized built-ins
upstream, the internal construction-time `is` value. Reading a mutable `is`
attribute is not equivalent. Definition-time candidate collection is connected,
registry-filtered and native-tree ordered. Explicit `upgrade(root)` also handles
detached subtrees, includes its root, visits author shadow roots before light
children, and excludes UA shadow roots. Closed author roots still participate.

An upgrade queues observed existing attributes in actual native attribute-list
order with null old values, followed by connectedCallback when appropriate. It
then executes the constructor under a definition-specific construction stack.
The constructor's native state is precustomized, so its mutations do not create
ordinary custom-state attribute/connection callbacks. Success installs the
definition and Custom state; failure clears that element's pending reactions and
makes subsequent upgrade attempts ineligible. Report constructor exceptions
without throwing from the outer define/upgrade call or preventing other
candidates from upgrading.

Subsequent native attribute mutations enqueue callbacks only for Custom state,
using the captured observed-name set. Equal old/new values still produce an
attribute reaction. Native attachment/removal produces connection reactions;
host JS does not rediscover these changes by periodically scanning wrappers.

The reaction owner has nested element-queue scopes and one shared FIFO reaction
queue per element. Increment its cursor before invoking a reaction. If an
attribute callback performs another CEReactions DOM operation, its nested scope
can drain the same element's already-pending reactions before returning to that
callback. A global non-reentrant batch drain would change this ordering.

`define` and `upgrade` are CEReactions operations: pending scoped reactions run
before returning to the caller. Only reactions created without an active scope
use the backup microtask queue. Captured callbacks run with the element as this
and typed snapshot arguments; report individual callback failures and continue
the remaining queue.

## Constructor and native-brand boundary

Direct `new` of a registered autonomous constructor remains explicitly
unsupported in this slice. The supported createElement/upgrade path is aligned
at its HTML constructor boundary, with focused native-brand and construction
regressions passing.
Blink's broader source contract is:

- Direct `new HTMLElement()` and an unregistered subclass are illegal
- Resolve the definition by NewTarget, using the construction stack first
- An autonomous definition invokes the HTMLElement interface, not an unrelated
  concrete HTML interface or SVG/MathML interface
- Read the current NewTarget.prototype when the HTML constructor runs; callback
  capture at registration does not freeze the future instance prototype
- A direct registered construction allocates a native element with its definition
- An upgrade consumes its existing native element once, replacing the stack
  entry with an already-constructed marker; nested/double super cannot reuse it
- Apply the prototype at the HTML constructor boundary and preserve native node
  identity; do not set it prematurely before invoking the custom constructor

Direct construction and synchronous `createElement` have different result
checks. Blink's synchronous creation additionally requires the expected native
HTMLElement, no attributes/children/parent, the same document, HTML namespace
and matching local name; reported failure yields a failed HTMLUnknownElement.
Broader parser/createElement/direct-new parity must not be inferred from passing
registration tests alone.

## Explicit boundaries and verification status

Direct `new` of registered constructors, customized built-ins, scoped registries,
form-associated custom elements and enforcement of disabled shadow/internals
features are explicitly unsupported in this bounded slice. Their admission
paths must reject unsupported requests; they must not silently store a flag and
imply working behavior. Preserve source property-read and exception order before
such admission errors where applicable.

This change does not implement parser-time synchronous custom construction,
declarative ShadowRoot parsing/hydration, complete innerHTML namespace handling,
adopted constructed stylesheet semantics, complete cross-document adoption, or
complete custom-element/Chromium pixel parity. Direct-new construction and
prototype-based native IDL dispatch are separate boundaries too.

Focused tests live in `crates/dom/tests/custom_elements.rs`,
`crates/webapi/tests/custom_element_definitions.rs`, and
`crates/browser/tests/custom_element_style_delivery.rs`. Their coverage includes
inherited metadata finalization during define, callback snapshots,
atomic rollback, conversion and exception order, reentrancy, native brands,
closed/detached shadow upgrades, constructor failures, scoped FIFO reaction
ordering, and style delivery through ordinary DOM operations.

Current verified results:

- New custom-element binding cases: 23 passed
- Existing slot and template binding cases: 7 passed each
- Native custom-element cases: 13 passed
- Page cases: 2 passed, covering real 120-by-40 fallback-style geometry and native
  backup-reaction microtask rearming
- Existing shadow-publication cases: 9 passed, with the same pre-existing ignored
  case
- Application build: passed

The 59 retained browser/rendering regression cases also pass. Broader results
are Style 638 passed/14 failed, WebAPI 23 passed/1 failed, application 37 passed/2
failed/3 ignored, and JavaScript 40 passed/2 failed/3 ignored. Every failure
identity matches its baseline, including a direct JavaScript source-baseline run.
This is baseline-equivalent verification, not an all-suite pass. The final
application build and diff check pass; the final executable hash matches the
captured build, and no owned build/browser/capture processes remain.

The genuine complete MDN page now has one finalized header style entry and one
delivered header shadow style; its search button measures 80 by 34. The modal
also has one shadow style. By final probe 6, the homepage has one finalized
style entry and one delivered shadow style, with button geometry
158.125 by 78.375 and host geometry 594.40625 by 78.375. One separate
host-property TypeError remains in bundle 9099: an author `checked` setter on an
autonomous custom element is intercepted by the native input-only projection.
Menus remain visibly uncollapsed. The captured page is a partial result with no
visual-parity claim. Screenshot artifact:
`libfile_9344a9dc4f508191880f3a4784af6bbc`.

That next boundary belongs to ordinary JavaScript property resolution versus
native IDL accessor dispatch. Regular own/custom-prototype properties must be
selected before native property conversion; explicitly invoking an input's
native accessor must still enforce the input brand. It is documented separately
as the next generic host/DOM integration task, with no workaround in this slice.

## Pinned source links

- [Registry and define transaction](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/html/custom/custom_element_registry.cc)
- [CustomElementRegistry IDL](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/html/custom/custom_element_registry.idl)
- [ElementDefinitionOptions IDL](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/dom/element_definition_options.idl)
- [Original property capture](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/bindings/core/v8/script_custom_element_definition_builder.cc)
- [Prototype and method retrieval](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/platform/bindings/callback_method_retriever.cc)
- [DOMString and sequence conversion](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/bindings/core/v8/native_value_traits_impl.h)
- [Generic iterator conversion](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/bindings/core/v8/script_iterator.cc#182)
- [Chromium's V8 revision pin](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/DEPS)
- [Pinned V8 actual-array predicate](https://chromium.googlesource.com/v8/v8/+/96550751177dbdb761b1708431616374148bc0eb/src/api/api.cc#3897)
- [Name validation](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/html/custom/custom_element.h)
- [Element local-name parsing](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/dom/document.cc#602)
- [Native definition and upgrade](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/html/custom/custom_element_definition.cc)
- [Construction and captured callback invocation](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/bindings/core/v8/script_custom_element_definition.cc)
- [HTML constructor](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/bindings/core/v8/v8_html_constructor.cc)
- [Native insertion, removal and attributes](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/dom/element.cc)
- [Reaction stack](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/html/custom/custom_element_reaction_stack.cc)
- [Per-element reaction queue](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/html/custom/custom_element_reaction_queue.cc)
- [CEReactions scope](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/html/custom/ce_reactions_scope.cc)
- [Shadow-inclusive upgrade sorter](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/html/custom/custom_element_upgrade_sorter.cc)

Original source-specific notices are retained in
`third_party/NOTICE.chromium-custom-elements`. The existing Chromium and Blink
license files remain applicable; the Chromium root license does not replace
the source-specific LGPL notices on the DOM integration files.
