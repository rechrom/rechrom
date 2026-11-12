# `interaction` Chromium translation plan

`interaction` is the source-aligned production interaction facade. Each
Page retains one facade instance; the package retains its translated dispatch,
default-action and input-manager state and borrows the live DOM/Layout/Paint
owners needed by one transaction. Browser only routes those typed owners and
the resulting `InteractionOutput`; it does not own a second interaction object
graph. The former `interaction` crate was removed at the dependency cutover.
Source-owner cutover is tracked by the code paths themselves. The obsolete
node-id cursor entry and compatibility TextEditor/default-action path have been
removed. Cursor selection has one resident Layout/Paint/EventHandler route;
text editing uses the translated KeyboardEventManager/Editor/InputMethodController
route over the retained Page editing owners.

## Source baseline

- Chromium checkout: `/Users/zhenghuaiyu/chromium/src`
- Chromium revision: `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`
- Translation unit: one Chromium logical `.h`/`.cc` pair becomes one `.rs`
  module. Header-only sources become one `.rs` module.
- Rust modules preserve the Chromium relative directory below `src/`. For
  example, Blink's `core/dom/events/event.{h,cc}` maps to
  `interaction_dispatch/src/core/dom/events/event.rs`.

This plan is the scope definition, not a separate mapping/status ledger. Each
translated Rust file carries its own source revision and source line ranges.
Translated blocks remain in Chromium declaration/definition order and use a
source marker immediately before the corresponding Rust block:

```text
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/dom/events/event.h:123-151
// Source: third_party/blink/renderer/core/dom/events/event.cc:207-244
```

Mechanical Rust differences such as ownership wrappers, error types, and
traits may change the number of Rust lines. They must not change source method
order, lifecycle, state ownership, or algorithm branches. Rechrom adapters are
kept in the package facade and are never mixed into translated source modules.

## Package graph and directory shape

```text
interaction_input
    ^
    +-- interaction_dispatch
    +-- interaction_default ----> interaction_dispatch
    +-- interaction_scroll  ----> interaction_hit_test
    +-- interaction_hit_test
                  ^
                  |
            interaction
```

The root `interaction` crate is the Page-facing facade. Child crates do not
call Page, Style, Layout, Paint, Compositor, or Renderer lifecycle methods.
Hit testing and scrolling borrow the canonical resident Layout/Paint owners;
they do not build a second document or fragment tree.

```text
crates/interaction/
  Cargo.toml
  README.md
  src/
    lib.rs
    core/input/                         # translated orchestration managers
    core/events/web_input_event_conversion.rs
    adapter/                            # Rechrom-only boundary adapters
  interaction_input/src/
    public/common/input/
    common/input/
    platform/widget/input/
  interaction_dispatch/src/
    core/dom/events/
    core/events/
    core/input/
  interaction_hit_test/src/
    core/layout/
    core/layout/svg/
    core/paint/
    core/page/
  interaction_default/src/
    core/page/
    core/editing/
    core/html/
    core/html/forms/
  interaction_scroll/src/
    core/events/
    core/frame/
    core/input/
    core/scroll/
    core/paint/
```

## `interaction_input`

Owns host-event values, coalescing rules, timestamps/phases, and frame-aligned
main-thread admission. It has no DOM or rendering dependency.

Complete logical source pairs in the replacement scope:

| Rust module | Chromium sources |
| --- | --- |
| `public/common/input/web_input_event.rs` | `public/common/input/web_input_event.h`, `common/input/web_input_event.cc` |
| `public/common/input/pointer_id.rs` | `public/common/input/pointer_id.h` |
| `public/common/input/web_pointer_properties.rs` | `public/common/input/web_pointer_properties.h` |
| `public/common/input/web_mouse_event.rs` | `public/common/input/web_mouse_event.h`, `common/input/web_mouse_event.cc` |
| `public/common/input/web_mouse_wheel_event.rs` | `public/common/input/web_mouse_wheel_event.h`, `common/input/web_mouse_wheel_event.cc` |
| `public/common/input/web_pointer_event.rs` | `public/common/input/web_pointer_event.h`, `common/input/web_pointer_event.cc` |
| `public/common/input/web_keyboard_event.rs` | `public/common/input/web_keyboard_event.h`, `common/input/web_keyboard_event.cc` |
| `public/common/input/web_coalesced_input_event.rs` | `public/common/input/web_coalesced_input_event.h`, `common/input/web_coalesced_input_event.cc` |
| `platform/widget/input/main_thread_event_queue_task.rs` | `platform/widget/input/main_thread_event_queue_task.h` |
| `platform/widget/input/main_thread_event_queue_task_list.rs` | `platform/widget/input/main_thread_event_queue_task_list.{h,cc}` |
| `platform/widget/input/main_thread_event_queue.rs` | `platform/widget/input/main_thread_event_queue.{h,cc}` |

Small upstream value dependencies are translated in the same crate rather than
redeclared without provenance:

| Rust module | Chromium sources |
| --- | --- |
| `public/mojom/input/input_event.rs` | `public/mojom/input/input_event.mojom` (`Button`, `DispatchType`, `EventType`, `RailsMode`) |
| `ui/events/types/event_type.rs` | `ui/events/types/event_type.h` |
| `ui/events/event_latency_metadata.rs` | `ui/events/event_latency_metadata.h` |
| `base/time/time.rs` | `base/time/time.h` (`TimeTicks` value ranges only; the embedding retains the clock) |

`web_gesture_event` and touch/drag platform events are follow-up translation
scope beyond the initial facade contract.

## `interaction_dispatch`

Owns DOM Event values, listener registration state, immutable dispatch paths,
retargeting contexts, capture/target/bubble propagation, cancellation, and
mouse/pointer boundary dispatch. DOM remains the tree/path-snapshot owner.

Event value modules:

| Rust module | Chromium sources |
| --- | --- |
| `core/dom/events/event.rs` | `core/dom/events/event.{h,cc}` |
| `core/dom/events/event_dispatch_result.rs` | `core/dom/events/event_dispatch_result.h` |
| `core/events/mouse_event.rs` | `core/events/mouse_event.{h,cc}` |
| `core/events/pointer_event.rs` | `core/events/pointer_event.{h,cc}` |
| `core/events/keyboard_event.rs` | `core/events/keyboard_event.{h,cc}` |
| `core/events/wheel_event.rs` | `core/events/wheel_event.{h,cc}` |
| `core/events/focus_event.rs` | `core/events/focus_event.{h,cc}` |
| `core/events/input_event.rs` | `core/events/input_event.{h,cc}` |
| `core/events/composition_event.rs` | `core/events/composition_event.{h,cc}` |

Listener and dispatch modules:

| Rust module | Chromium sources |
| --- | --- |
| `core/dom/events/event_listener.rs` | `core/dom/events/event_listener.h` |
| `core/dom/events/native_event_listener.rs` | `core/dom/events/native_event_listener.h` |
| `core/dom/events/registered_event_listener.rs` | `core/dom/events/registered_event_listener.{h,cc}` |
| `core/dom/events/event_listener_map.rs` | `core/dom/events/event_listener_map.{h,cc}` |
| `core/dom/events/event_target.rs` | `core/dom/events/event_target.{h,cc}` |
| `core/dom/events/event_target_impl.rs` | `core/dom/events/event_target_impl.{h,cc}` |
| `core/dom/events/node_event_context.rs` | `core/dom/events/node_event_context.{h,cc}` |
| `core/dom/events/tree_scope_event_context.rs` | `core/dom/events/tree_scope_event_context.{h,cc}` |
| `core/dom/events/window_event_context.rs` | `core/dom/events/window_event_context.{h,cc}` |
| `core/dom/events/event_path.rs` | `core/dom/events/event_path.{h,cc}` |
| `core/dom/events/event_dispatcher.rs` | `core/dom/events/event_dispatcher.{h,cc}` |
| `core/dom/events/event_dispatch_forbidden_scope.rs` | `core/dom/events/event_dispatch_forbidden_scope.{h,cc}` |
| `core/input/boundary_event_dispatcher.rs` | `core/input/boundary_event_dispatcher.{h,cc}` |
| `core/input/event_handling_util.rs` | `core/input/event_handling_util.{h,cc}` |

## `interaction_hit_test`

Owns hit-test request/location/result semantics and traversal decisions. It
borrows Layout's resident fragments and Paint's canonical property/transform
sidecars. It must not translate an entire painter merely to obtain its
`NodeAtPoint` methods.

Complete logical source pairs:

| Rust module | Chromium sources |
| --- | --- |
| `core/layout/hit_test_phase.rs` | `core/layout/hit_test_phase.h` |
| `core/layout/hit_test_request.rs` | `core/layout/hit_test_request.{h,cc}` |
| `core/layout/hit_test_location.rs` | `core/layout/hit_test_location.{h,cc}` |
| `core/layout/hit_test_result.rs` | `core/layout/hit_test_result.{h,cc}` |
| `core/layout/hit_test_cache.rs` | `core/layout/hit_test_cache.{h,cc}` |
| `core/layout/svg/transformed_hit_test_location.rs` | `core/layout/svg/transformed_hit_test_location.{h,cc}` |
| `core/paint/hit_testing_transform_state.rs` | `core/paint/hit_testing_transform_state.{h,cc}` |
| `core/paint/paint_layer_paint_order_iterator.rs` | `core/paint/paint_layer_paint_order_iterator.{h,cc}` |
| `core/page/event_with_hit_test_results.rs` | `core/page/event_with_hit_test_results.h` |

Shared-source translated ranges, kept as modules with the same Chromium name:

- `core/paint/box_fragment_painter.rs`: declarations and definitions for
  `HitTestContext`, `NodeAtPoint`, `HitTestAllPhases`, child/item/text/SVG hit
  traversal, and hit-test visibility helpers only.
- `core/layout/layout_inline.rs`: `NodeAtPoint` and culled-inline hit-test
  declarations/definitions only.
- `core/paint/paint_layer_hit_test.rs`: the `PaintLayer::HitTest`, layer-child,
  fragment, transform, clip and border-shape hit-test declarations/definitions
  from `paint_layer.{h,cc}` only. The separate module name avoids overwriting
  Paint's resident `PaintLayer` owner.

Any required non-hit-test painter code stays in `paint`; this crate calls it
through borrowed canonical values rather than duplicating it.

## `interaction_default`

Owns focus transitions and source default actions for controls, activation,
text editing, selection, and form submission.

Focus and editing sources:

| Rust module | Chromium sources |
| --- | --- |
| `core/page/focus_controller.rs` | `core/page/focus_controller.{h,cc}` focus-transition ranges |
| `core/editing/editor.rs` | `core/editing/editor.{h,cc}` keyboard/text default-action ranges |
| `core/editing/frame_selection.rs` | `core/editing/frame_selection.{h,cc}` selection mutation ranges |
| `core/editing/selection_controller.rs` | `core/editing/selection_controller.{h,cc}` pointer-selection ranges |
| `core/editing/ime/input_method_controller.rs` | `core/editing/ime/input_method_controller.{h,cc}` composition/commit ranges |

Form-control foundations and concrete types:

| Rust module | Chromium sources |
| --- | --- |
| `core/html/forms/input_type.rs` | `core/html/forms/input_type.{h,cc}` relevant factory/base ranges |
| `core/html/forms/input_type_view.rs` | `core/html/forms/input_type_view.{h,cc}` |
| `core/html/forms/base_button_input_type.rs` | `core/html/forms/base_button_input_type.{h,cc}` |
| `core/html/forms/base_checkable_input_type.rs` | `core/html/forms/base_checkable_input_type.{h,cc}` |
| `core/html/forms/base_text_input_type.rs` | `core/html/forms/base_text_input_type.{h,cc}` |
| `core/html/forms/keyboard_clickable_input_type_view.rs` | `core/html/forms/keyboard_clickable_input_type_view.{h,cc}` |
| `core/html/forms/button_input_type.rs` | `core/html/forms/button_input_type.{h,cc}` |
| `core/html/forms/checkbox_input_type.rs` | `core/html/forms/checkbox_input_type.{h,cc}` |
| `core/html/forms/radio_input_type.rs` | `core/html/forms/radio_input_type.{h,cc}` |
| `core/html/forms/range_input_type.rs` | `core/html/forms/range_input_type.{h,cc}` |
| `core/html/forms/reset_input_type.rs` | `core/html/forms/reset_input_type.{h,cc}` |
| `core/html/forms/submit_input_type.rs` | `core/html/forms/submit_input_type.{h,cc}` |
| `core/html/forms/search_input_type.rs` | `core/html/forms/search_input_type.{h,cc}` |
| `core/html/forms/text_input_type.rs` | `core/html/forms/text_input_type.{h,cc}` |
| `core/html/forms/text_field_input_type.rs` | `core/html/forms/text_field_input_type.{h,cc}` |
| `core/html/forms/select_type.rs` | `core/html/forms/select_type.{h,cc}` reachable selection ranges |
| `core/html/forms/step_range.rs` | `core/html/forms/step_range.{h,cc}` |

Owning element default-handler ranges:

- `core/html/html_element.rs`
- `core/html/forms/html_input_element.rs`
- `core/html/forms/html_button_element.rs`
- `core/html/forms/html_label_element.rs`
- `core/html/html_summary_element.rs`
- `core/html/html_details_element.rs`
- `core/html/forms/html_form_element.rs`
- `core/html/forms/html_select_element.rs`
- `core/html/forms/html_text_area_element.rs`
- `core/html/forms/text_control_element.rs`
- `core/html/forms/text_control_inner_elements.rs`

These large element sources are translated only for their dispatch/default
action ranges. DOM storage, parsing, style, and shadow-tree ownership remain in
their existing packages. Additional Chromium input types (date/time, color,
file, image and picker families) are the first post-cutover expansion.

## `interaction_scroll`

Owns the Blink main-thread wheel path, blocking-listener region decisions,
scroll chaining/default action, and typed scroll updates. Compositor scrolling
and pending/active-tree state remain in `compositor`.

Complete source pairs:

| Rust module | Chromium sources |
| --- | --- |
| `core/input/mouse_wheel_event_manager.rs` | `core/input/mouse_wheel_event_manager.{h,cc}` |
| `core/input/scroll_manager.rs` | `core/input/scroll_manager.{h,cc}` |
| `core/frame/event_handler_registry.rs` | `core/frame/event_handler_registry.{h,cc}` wheel-listener ranges |

Shared-source translated ranges:

- `core/input/event_handler.rs`: wheel dispatch and default-scroll routing.
- `core/scroll/scrollable_area.rs`: user-scrollability, clamping, chaining,
  and scroll-result calculations only.
- `core/paint/paint_layer_scrollable_area.rs`: canonical min/max offset and
  scroll-container decision ranges only.

`WheelEvent` itself lives in `interaction_dispatch`; the raw
`WebMouseWheelEvent` lives in `interaction_input`.

## Root `interaction` facade

The facade translates the input managers that orchestrate the five stages and
keeps Rechrom-only assembly in `src/adapter/`:

| Rust module | Chromium sources |
| --- | --- |
| `core/input/widget_event_handler.rs` | `core/input/widget_event_handler.{h,cc}` |
| `core/input/event_handler.rs` | `core/input/event_handler.{h,cc}` non-scroll orchestration ranges |
| `core/input/mouse_event_manager.rs` | `core/input/mouse_event_manager.{h,cc}` |
| `core/input/pointer_event_manager.rs` | `core/input/pointer_event_manager.{h,cc}` |
| `core/input/keyboard_event_manager.rs` | `core/input/keyboard_event_manager.{h,cc}` |
| `core/events/web_input_event_conversion.rs` | `core/events/web_input_event_conversion.{h,cc}` |

`adapter/` converts the existing host `InputEvent`, live DOM access,
`PaintFragmentCursor`, and output callbacks into the translated types. It does
not reimplement source decisions.

## Translation and cutover order

1. **Scaffold:** create the six crates and source-shaped directories without
   routing production traffic to them.
2. **Input:** translate raw event values and coalescing/queue behavior.
3. **Dispatch:** translate Event, listener/path state, propagation, and boundary
   events.
4. **Hit test:** translate query values first, then the resident-fragment
   traversal ranges.
5. **Default actions:** translate focus/editing foundations, then concrete
   controls in activation dependency order.
6. **Scroll:** translate wheel dispatch, listener regions, scroll-chain
   selection, and typed offset results.
7. **Facade:** translate the event managers and add only the minimal Rechrom
   adapters needed to retain the current `InteractionOutput` contract.
8. **Cutover:** point `browser` and other consumers at `interaction`, run the
   existing interaction/browser checks once as a focused batch, and remove all
   predecessor entry points so the facade has one production route.

Compilation is performed at coherent package boundaries rather than after each
source block. Browser-level verification is reserved for the integrated
cutover unless a concrete intermediate failure requires it.

## Progress report format

Every implementation report uses source lines from the pinned revision:

```text
current: interaction_input / public/common/input/web_input_event.rs
source:  web_input_event.h:1-455, web_input_event.cc:1-112
done:    web_input_event.h:1-218
active:  web_input_event.h:219-455
next:    web_input_event.cc:1-112
```

No separate status database is maintained. The source markers in `.rs` files,
the current diff, and the report above are the source of truth.
