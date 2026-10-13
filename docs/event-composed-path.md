# Event paths and composedPath

## Ownership and source alignment

The DOM owner exports an immutable `DOMEventPath` snapshot. It consumes its
existing `AssignedSlot`, `ShadowHost`, `TreeScopeRoot`, and shadow-mode state;
there is no second assignment map or JavaScript parent-chain reconstruction.
Entries carry retained node IDs and their scope-adjusted target. Snapshot-local
TreeScope contexts translate Blink's target inheritance, tree ordering, nearest
closed-tree tracking, and observer-relative visibility predicate.

Interaction wraps that snapshot with the Window endpoint and the node `load`
exception. It owns listener phases and dispatch lifetime. Both native input and
script `dispatchEvent` use the same dispatcher. The prior two Page adapters that
injected Window callbacks were removed; Page only forwards typed invocations.

The shared dispatch state keeps native retained Event wrappers current and drops
the path when dispatch ends. Constructed JavaScript Events preserve their own
identity while their private state delegates path, target, currentTarget and
phase to the native event owner during callbacks. Native wrapper redispatch
preserves identity and resets trust. Actions called on an outer Event from a
nested listener target that event's native wrapper, not the nested event.
Generic EventTargets use a private one-target dispatch state. XMLHttpRequest
reuses that owner rather than assigning public Event fields in a second loop.

`composedPath()` returns a fresh array, observes the current listener's closed
root visibility, and is empty before/after dispatch. Named/manual slot paths,
non-composed light-DOM events, host AT_TARGET capture/noncapture delivery,
nonbubbling composed events, mutation-stable paths, native receiver brands and
post-dispatch target clearing are covered by focused regressions.

## Pinned Chromium sources

Revision: `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`

- [Event::composedPath](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/dom/events/event.cc)
- [EventPath](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/dom/events/event_path.cc)
- [TreeScopeEventContext](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/dom/events/tree_scope_event_context.cc)
- [EventDispatcher](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/dom/events/event_dispatcher.cc)
- [WindowEventContext](https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/dom/events/window_event_context.cc)

The pinned source marks ClearCurrentTargetAfterDispatch,
ClearTargetOnlyIfInShadowTree and ComposedPathReturnTargetBeingDispatched as
stable. Attribution and applicable source headers are in
`third_party/NOTICE.chromium-event-path`.

## Verification and explicit scope

`cargo test -p dom --test event_path -p browser --test event_composed_path`:
14 DOM tests and 10 production Page tests pass. The first three Page cases were
recorded failing before implementation. The trusted-input fixture dispatches
`MouseEventType::kClick`; a down/up fixture independently hits the existing
`IsUnboundedElementActive` Style collaborator panic. Live MDN testing uses the
actual supported mouse move/press/release protocol separately.

This is the supported node/window dispatch-path slice, not complete DOM Events
or Shadow DOM conformance. Related-target path truncation, touch-list
retargeting, disabled-control path filtering, pseudo-element target rules,
KeepEventInNode, and stopped-document rules remain outside this change. No new
render scheduling or layout algorithm is included. A correct DOM state does not
by itself establish that an asynchronously captured frame has caught up.
