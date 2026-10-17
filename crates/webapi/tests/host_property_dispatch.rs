//! Ordinary DOM properties resolve JavaScript descriptors before native accessors.
//! Sources: https://webidl.spec.whatwg.org/#es-attributes
//! https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-ordinarysetwithowndescriptor
//! https://html.spec.whatwg.org/multipage/input.html#dom-input-checked
//! All assertions use public DOM/ECMAScript operations through the real bindings.
use dom::dom_mutation::ApplyDOMMutations;
use javascript::{
    javascript_runtime::JavaScriptRuntime, quickjs_javascript_runtime::QuickJsJavaScriptRuntime,
};
use std::{cell::RefCell, rc::Rc};
use webapi::{dom_bindings::DOMJavaScriptBindings, window_bindings::WindowJavaScriptBindings};

struct NoNetwork;
impl xhr_transport::XMLHttpRequestTransport for NoNetwork {
    fn Start(
        &mut self,
        _: &xhr_transport::XMLHttpRequestData,
    ) -> std::io::Result<Box<dyn xhr_transport::XMLHttpRequestOperation>> {
        Err(std::io::Error::other("fixture has no network"))
    }
}

fn run(script: &str) {
    run_steps(&[script]);
}

fn run_steps(scripts: &[&str]) {
    let document = Rc::new(RefCell::new(html::html_parser::ParseHTML(
        "<!doctype html><html><head></head><body><main id=root></main></body></html>",
    )));
    let changed = document.clone();
    let dom = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
        document.clone(),
        Box::new(move |mutation| {
            ApplyDOMMutations(
                changed.borrow_mut().GetDocumentMut(),
                std::slice::from_ref(mutation),
            );
        }),
    )));
    // Use the same notification boundary as Page; no JS runs under DOM/host borrows.
    let scoped_document = document.clone();
    let scoped_bindings = Rc::downgrade(&dom);
    dom.borrow_mut()
        .SetScopedMutationEmitter(Rc::new(move |mutation, runtime| {
            let bindings = scoped_bindings.upgrade().unwrap();
            let notification = bindings.borrow().PrepareMutationNotification(mutation);
            ApplyDOMMutations(
                scoped_document.borrow_mut().GetDocumentMut(),
                std::slice::from_ref(mutation),
            );
            DOMJavaScriptBindings::DeliverMutationNotification(
                &bindings,
                notification,
                &mut |callback, args| {
                    runtime.Call(
                        callback,
                        &javascript::javascript_runtime::HostValue::Object(
                            javascript::javascript_runtime::HostObjectRef { id: 0 },
                        ),
                        args,
                    );
                },
            );
        }));
    // This bindings-only fixture has no Page. Supply its at-target dispatch
    // adapter through the same injected service used by the embedding.
    let weak_dom = Rc::downgrade(&dom);
    dom.borrow_mut()
        .SetSyntheticEventHandler(Some(Rc::new(move |event, target, runtime| {
            let dom = weak_dom.upgrade().unwrap();
            let invocation = interaction::event::EventListenerInvocation {
                event,
                target_node_id: target,
                current_target_node_id: target,
                phase: interaction::event::EventPhase::kAtTarget,
                capture_listeners: false,
                focused_node_id: None,
            };
            let result = DOMJavaScriptBindings::DispatchEventListenersScoped(
                &dom,
                &invocation,
                runtime,
                &mut |error| panic!("listener error: {error:?}"),
            );
            event.default_prevented |= result.prevent_default;
            *event.dispatch_state.borrow_mut() = interaction::event_path::EventDispatchState {
                path: None,
                final_target: Some(target),
                finished: true,
            };
        })));
    let window = Rc::new(RefCell::new(WindowJavaScriptBindings::new(
        dom.clone(),
        Box::new(NoNetwork),
        "host-property-dispatch-fixture".into(),
        None,
    )));
    let mut runtime = QuickJsJavaScriptRuntime::new();
    let realm = runtime.CreateRealm(window.clone());
    window.borrow_mut().BindRuntime(&realm);
    dom.borrow_mut().BindRuntime(&realm);
    for (source, name) in [
        (webapi::DOMBootstrapSource().to_owned(), "dom-webidl"),
        (webapi::WindowBootstrapSource(), "window-webidl"),
    ] {
        let result = runtime.Evaluate(&realm, &source, name);
        assert!(result.Succeeded(), "{name}: {:?}", result.exception);
    }
    for script in scripts {
        let result = runtime.Evaluate(
            &realm,
            &format!(
                r#"function check(value, message) {{ if (!value) throw new Error(message); }}
                function equal(actual, expected, message) {{
                    check(Object.is(actual, expected), message);
                }}
                function throwsSame(call, expected, message) {{
                    let didThrow = false, caught;
                    try {{ call(); }} catch (error) {{ didThrow = true; caught = error; }}
                    check(didThrow && caught === expected, message);
                }}
                function throwsTypeError(call, message) {{
                    let didThrow = false, caught;
                    try {{ call(); }} catch (error) {{ didThrow = true; caught = error; }}
                    check(didThrow && caught instanceof TypeError, message);
                }}
                {script}"#
            ),
            "host-property-dispatch.js",
        );
        assert!(result.Succeeded(), "{:?}", result.exception);
        runtime.PerformMicrotaskCheckpoint();
        let errors = runtime.TakePendingExceptions(&realm);
        assert!(errors.is_empty(), "unexpected reported errors: {errors:?}");
    }
}

#[test]
fn autonomous_element_constructor_uses_author_checked_accessor() {
    run(r#"
        const state = Symbol('checked'), calls = [], initial = {author: 'checked'};
        class ToggleElement extends HTMLElement {
            constructor() { super(); this.checked = initial; }
            get checked() { return this[state]; }
            set checked(value) { calls.push([this, value]); this[state] = value; }
        }
        customElements.define('x-author-checked', ToggleElement);
        const element = document.createElement('x-author-checked');
        equal(calls.length, 1, 'constructor calls the author setter once');
        equal(calls[0][0], element, 'setter receives the custom element');
        equal(calls[0][1], initial, 'setter receives the original value');
        equal(element.checked, initial, 'author getter returns symbol-backed value');
        element.checked = false;
        equal(calls.length, 2, 'subsequent assignment calls the same setter');
        equal(element.checked, false, 'author state changes');
        check(!Object.hasOwn(element, 'checked'), 'prototype setter does not create an own checked property');
        check(!element.hasAttribute('checked'), 'author property does not reflect a native attribute');
    "#);
}

#[test]
fn autonomous_element_constructor_uses_author_value_accessor() {
    run(r#"
        const state = Symbol('value'), calls = [], initial = {author: 'value'};
        class ValueElement extends HTMLElement {
            constructor() { super(); this.value = initial; }
            get value() { return this[state]; }
            set value(value) { calls.push([this, value]); this[state] = value; }
        }
        customElements.define('x-author-value', ValueElement);
        const element = document.createElement('x-author-value');
        equal(calls.length, 1, 'constructor calls the author value setter once');
        equal(calls[0][0], element, 'value setter receiver');
        equal(calls[0][1], initial, 'author value is not string-converted');
        equal(element.value, initial, 'author value getter');
        element.value = 17;
        equal(element.value, 17, 'later author value retains number type');
        equal(calls.length, 2, 'later value setter called once');
        check(!element.hasAttribute('value'), 'author value does not reflect a native attribute');
    "#);
}

#[test]
fn autonomous_element_data_accessor_is_not_silently_discarded() {
    run(r#"
        const state = Symbol('data'), calls = [], initial = {author: 'data'};
        class DataElement extends HTMLElement {
            constructor() { super(); this.data = initial; }
            get data() { return this[state]; }
            set data(value) { calls.push([this, value]); this[state] = value; }
        }
        customElements.define('x-author-data', DataElement);
        const element = document.createElement('x-author-data');
        equal(calls.length, 1, 'constructor invokes the author data setter');
        equal(calls[0][0], element, 'data setter receiver');
        equal(element.data, initial, 'data retains original object identity');
        equal(element.textContent, '', 'author data does not replace native child text');
    "#);
}

#[test]
fn unrelated_interfaces_accept_regular_expandos_without_native_interception() {
    run(r#"
        const cases = [
            [document.createElement('div'), ['checked', 'indeterminate', 'value', 'data']],
            [document.createTextNode('native text'), ['checked', 'value', 'id', 'className', 'innerHTML']],
            [document.createDocumentFragment(), ['checked', 'value', 'id', 'data', 'innerHTML']],
            [document, ['checked', 'value', 'id', 'data']]
        ];
        for (const [object, members] of cases) {
            for (const member of members) {
                check(!(member in object), 'unrelated interface has no native member ' + member);
                const value = {member};
                object[member] = value;
                equal(object[member], value, 'unrelated interface retains expando ' + member);
                const descriptor = Object.getOwnPropertyDescriptor(object, member);
                check(descriptor && descriptor.value === value, 'ordinary own data descriptor ' + member);
            }
        }
        equal(cases[1][0].data, 'native text', 'unrelated expandos preserve CharacterData');
        equal(cases[2][0].childNodes.length, 0, 'fragment innerHTML expando does not parse children');
    "#);
}

#[test]
fn own_data_property_shadows_native_input_accessor_until_deleted() {
    run(r#"
        const input = document.createElement('input');
        input.setAttribute('type', 'checkbox');
        input.checked = false;
        const initial = {author: 'own data'};
        Object.defineProperty(input, 'checked', {
            value: initial, writable: true, enumerable: true, configurable: true
        });
        equal(input.checked, initial, 'own checked data wins over native getter');
        input.checked = 'author update';
        equal(input.checked, 'author update', 'ordinary write updates own data');
        equal(Object.getOwnPropertyDescriptor(input, 'checked').value, 'author update', 'own value updated');
        check(delete input.checked, 'own override is deletable');
        equal(input.checked, false, 'deleting own data exposes unchanged native checked state');
        input.checked = true;
        equal(input.checked, true, 'native setter resumes after deletion');
    "#);
}

#[test]
fn own_accessor_shadows_native_input_getter_and_setter() {
    run(r#"
        const input = document.createElement('input'), calls = [];
        input.checked = false;
        let state = 'initial author state';
        Object.defineProperty(input, 'checked', {
            configurable: true,
            get() { calls.push(['get', this]); return state; },
            set(value) { calls.push(['set', this, value]); state = value; }
        });
        equal(input.checked, 'initial author state', 'own getter takes precedence');
        input.checked = 'next author state';
        equal(input.checked, 'next author state', 'own setter updates author state');
        equal(calls.length, 3, 'each own accessor executes once');
        check(calls.every(call => call[1] === input), 'own accessor receives input wrapper');
        equal(calls[1][2], 'next author state', 'own setter receives unconverted value');
        delete input.checked;
        equal(input.checked, false, 'own accessor writes did not mutate native checked state');
    "#);
}

#[test]
fn inherited_author_descriptor_shadows_native_input_accessor() {
    run(r#"
        const input = document.createElement('input'), state = Symbol('state'), calls = [];
        input.checked = false;
        const authorPrototype = Object.create(HTMLInputElement.prototype);
        Object.defineProperty(authorPrototype, 'checked', {
            configurable: true,
            get() { calls.push(['get', this]); return this[state]; },
            set(value) { calls.push(['set', this, value]); this[state] = value; }
        });
        Object.setPrototypeOf(input, authorPrototype);
        input.checked = 'author prototype value';
        equal(input.checked, 'author prototype value', 'nearer prototype accessor wins');
        equal(calls.length, 2, 'inherited getter and setter each execute once');
        check(calls.every(call => call[1] === input), 'inherited accessors keep receiver identity');
        check(!Object.hasOwn(input, 'checked'), 'inherited setter did not create own checked');
        delete authorPrototype.checked;
        equal(input.checked, false, 'removing nearer accessor restores unchanged native value');
    "#);
}

#[test]
fn custom_element_inherited_id_and_text_content_shadows_do_not_mutate_dom() {
    run(r#"
        const idState = Symbol('id'), textState = Symbol('text'), calls = [];
        class BaseElement extends HTMLElement {
            get id() { return this[idState]; }
            set id(value) { calls.push(['id', this, value]); this[idState] = value; }
            get textContent() { return this[textState]; }
            set textContent(value) { calls.push(['text', this, value]); this[textState] = value; }
        }
        class DerivedElement extends BaseElement {}
        customElements.define('x-inherited-properties', DerivedElement);
        const element = document.createElement('x-inherited-properties');
        element.setAttribute('id', 'native-id');
        const child = document.createTextNode('native text');
        element.appendChild(child);
        const id = {author: 'id'}, text = {author: 'text'};
        element.id = id;
        element.textContent = text;
        equal(element.id, id, 'inherited id getter returns author value');
        equal(element.textContent, text, 'inherited textContent getter returns author value');
        equal(calls.length, 2, 'inherited setters execute exactly once');
        check(calls.every(call => call[1] === element), 'inherited setters use custom element receiver');
        equal(element.getAttribute('id'), 'native-id', 'author id preserves native attribute');
        equal(element.firstChild, child, 'author textContent preserves native child identity');
        equal(child.data, 'native text', 'author textContent preserves native text');
        const idDescriptor = Object.getOwnPropertyDescriptor(Element.prototype, 'id');
        check(idDescriptor && typeof idDescriptor.set === 'function', 'native Element id accessor exists');
        idDescriptor.set.call(element, 'explicit native id');
        equal(element.getAttribute('id'), 'explicit native id', 'saved native setter explicitly updates attribute');
        equal(element.id, id, 'explicit native mutation does not replace author state');
    "#);
}

#[test]
fn author_getter_and_setter_exceptions_keep_identity_and_execute_once() {
    run(r#"
        const input = document.createElement('input');
        input.checked = false;
        const getterError = {from: 'getter'}, setterError = {from: 'setter'};
        let reads = 0, writes = 0;
        Object.defineProperty(input, 'checked', {
            configurable: true,
            get() { ++reads; throw getterError; },
            set(value) { ++writes; throw setterError; }
        });
        throwsSame(() => input.checked, getterError, 'getter preserves exact thrown object');
        equal(reads, 1, 'throwing getter executes once');
        throwsSame(() => { input.checked = true; }, setterError, 'setter preserves exact thrown object');
        equal(writes, 1, 'throwing setter executes once');
        check('checked' in input, 'membership recognizes own accessor');
        equal(reads, 1, 'membership does not call getter');
        equal(writes, 1, 'membership does not call setter');
        delete input.checked;
        equal(input.checked, false, 'throwing author setter leaves native state unchanged');
    "#);
}

#[test]
fn inherited_author_exceptions_are_not_swallowed_by_host_dispatch() {
    run(r#"
        const readError = Symbol('read error'), writeError = {from: 'write'};
        let reads = 0, writes = 0;
        class ThrowingElement extends HTMLElement {
            get value() { ++reads; throw readError; }
            set value(value) { ++writes; throw writeError; }
        }
        customElements.define('x-throwing-property', ThrowingElement);
        const element = document.createElement('x-throwing-property');
        throwsSame(() => element.value, readError, 'inherited getter preserves thrown symbol');
        throwsSame(() => { element.value = 'x'; }, writeError, 'inherited setter preserves thrown object');
        equal(reads, 1, 'inherited getter executes once');
        equal(writes, 1, 'inherited setter executes once');
        check('value' in element, 'inherited accessor membership');
        equal(reads, 1, 'inherited membership does not execute getter');
    "#);
}

#[test]
fn getter_only_and_nonwritable_properties_reject_writes_without_native_mutation() {
    run(r#"
        for (const inherited of [false, true]) {
            for (const accessor of [false, true]) {
                const input = document.createElement('input');
                input.checked = false;
                const owner = inherited ? Object.create(HTMLInputElement.prototype) : input;
                Object.defineProperty(owner, 'checked', accessor
                    ? {get() { return 'author getter'; }, configurable: true}
                    : {value: 'author data', writable: false, configurable: true});
                if (inherited) Object.setPrototypeOf(input, owner);
                equal(Reflect.set(input, 'checked', true), false, 'Reflect.set rejects read-only author property');
                throwsTypeError(() => { 'use strict'; input.checked = true; }, 'strict assignment rejects author property');
                equal(input.checked, accessor ? 'author getter' : 'author data', 'rejected writes preserve author value');
                delete owner.checked;
                equal(input.checked, false, 'rejected writes preserve native checked state');
            }
        }
    "#);
}

#[test]
fn checked_accessor_is_owned_by_input_prototype_with_webidl_flags() {
    run(r#"
        const input = document.createElement('input');
        const descriptor = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'checked');
        check(descriptor !== undefined, 'checked is installed on HTMLInputElement.prototype');
        equal(typeof descriptor.get, 'function', 'checked has a native getter');
        equal(typeof descriptor.set, 'function', 'checked has a native setter');
        equal(descriptor.get.name, 'get checked', 'native getter exposes Web IDL function name');
        equal(descriptor.set.name, 'set checked', 'native setter exposes Web IDL function name');
        equal(descriptor.get.length, 0, 'native getter arity');
        equal(descriptor.set.length, 1, 'native setter arity');
        equal(descriptor.enumerable, true, 'checked is enumerable');
        equal(descriptor.configurable, true, 'checked is configurable');
        check(!('value' in descriptor) && !('writable' in descriptor), 'checked is an accessor descriptor');
        check(!Object.hasOwn(input, 'checked'), 'input instance has no native own checked property');
        check(!Object.hasOwn(HTMLElement.prototype, 'checked'), 'checked is not an HTMLElement member');
        check(!Object.hasOwn(Element.prototype, 'checked'), 'checked is not an Element member');
        equal(Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'checked').get, descriptor.get, 'stable getter identity');
        equal(Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'checked').set, descriptor.set, 'stable setter identity');
    "#);
}

#[test]
fn saved_native_checked_accessors_enforce_native_brand_not_prototype_identity() {
    run(r#"
        const descriptor = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'checked');
        check(descriptor && descriptor.get && descriptor.set, 'native checked accessor exists');
        const forgedElement = document.createElement('div');
        Object.setPrototypeOf(forgedElement, HTMLInputElement.prototype);
        const invalid = [
            {}, document, document.createElement('div'), document.createElement('textarea'),
            document.createElementNS('http://www.w3.org/2000/svg', 'input'),
            Object.create(HTMLInputElement.prototype), forgedElement, null, undefined
        ];
        for (const receiver of invalid) {
            throwsTypeError(() => descriptor.get.call(receiver), 'native getter rejects non-input receiver');
            throwsTypeError(() => descriptor.set.call(receiver, true), 'native setter rejects non-input receiver');
        }
        const input = document.createElement('input');
        Object.setPrototypeOf(input, null);
        equal(descriptor.get.call(input), false, 'real input retains native brand after prototype removal');
        descriptor.set.call(input, true);
        equal(descriptor.get.call(input), true, 'saved native setter accepts real input with changed prototype');
    "#);
}

#[test]
fn native_input_booleans_use_webidl_boolean_conversion() {
    run(r#"
        const input = document.createElement('input');
        const conversions = [
            [false, false], [true, true], [0, false], [1, true], ['', false], ['false', true],
            [null, false], [undefined, false], [NaN, false], [{}, true], [Symbol('truthy'), true]
        ];
        let coercions = 0;
        conversions.push([{valueOf() { ++coercions; throw new Error('must not coerce'); },
                           toString() { ++coercions; throw new Error('must not coerce'); }}, true]);
        for (const member of ['checked', 'indeterminate']) {
            for (const [value, expected] of conversions) {
                input[member] = value;
                equal(input[member], expected, member + ' uses ToBoolean');
                check(!Object.hasOwn(input, member), member + ' remains a prototype accessor');
            }
        }
        equal(coercions, 0, 'ToBoolean never invokes object conversion hooks');
    "#);
}

#[test]
fn native_controls_keep_live_state_separate_from_content_attributes() {
    run(r#"
        const input = document.createElement('input');
        input.setAttribute('type', 'text');
        input.setAttribute('checked', '');
        input.setAttribute('value', 'seed');
        equal(input.checked, true, 'checked attribute initializes live native state');
        equal(input.value, 'seed', 'value attribute initializes live native state');
        input.checked = false;
        input.indeterminate = true;
        input.value = 'edited';
        equal(input.checked, false, 'native checked mutation');
        equal(input.indeterminate, true, 'native indeterminate mutation');
        equal(input.value, 'edited', 'native value mutation');
        check(input.hasAttribute('checked'), 'checked write preserves default attribute');
        equal(input.getAttribute('value'), 'seed', 'value write preserves default attribute');
        const textarea = document.createElement('textarea');
        textarea.textContent = 'initial text';
        equal(textarea.value, 'initial text', 'textarea initial native value');
        textarea.value = 'edited text';
        equal(textarea.value, 'edited text', 'textarea native value setter');
        equal(textarea.textContent, 'initial text', 'textarea live value does not replace default text');
        const select = document.createElement('select');
        for (const value of ['a', 'b']) {
            const option = document.createElement('option');
            option.setAttribute('value', value);
            option.appendChild(document.createTextNode(value.toUpperCase()));
            select.appendChild(option);
        }
        select.value = 'b';
        equal(select.value, 'b', 'select native value setter');
    "#);
}

#[test]
fn reflect_get_and_set_native_accessor_use_distinct_input_receiver() {
    run(r#"
        const target = document.createElement('input'), receiver = document.createElement('input');
        target.checked = false;
        receiver.checked = true;
        equal(Reflect.get(target, 'checked', receiver), true, 'native getter reads receiver rather than target');
        equal(Reflect.set(target, 'checked', false, receiver), true, 'native receiver write succeeds');
        equal(receiver.checked, false, 'Reflect.set updates receiver native state');
        equal(target.checked, false, 'Reflect.set preserves target native state');
        equal(Reflect.set(target, 'checked', true, receiver), true, 'second native receiver write succeeds');
        equal(receiver.checked, true, 'second write updates receiver');
        equal(target.checked, false, 'second write still does not mutate target');
        target.value = 'target value';
        receiver.value = 'receiver value';
        equal(Reflect.get(target, 'value', receiver), 'receiver value', 'native value getter reads receiver');
        equal(Reflect.set(target, 'value', 'receiver update', receiver), true, 'native value receiver write succeeds');
        equal(receiver.value, 'receiver update', 'native value writes receiver');
        equal(target.value, 'target value', 'native value does not mutate target');
    "#);
}

#[test]
fn reflect_native_accessor_rejects_non_input_receiver_without_mutating_target() {
    run(r#"
        const input = document.createElement('input');
        input.checked = false;
        for (const receiver of [{checked: 'own value'}, document.createElement('div'), Object.create(HTMLInputElement.prototype)]) {
            throwsTypeError(() => Reflect.get(input, 'checked', receiver), 'selected native getter brands actual receiver');
            throwsTypeError(() => Reflect.set(input, 'checked', true, receiver), 'selected native setter brands actual receiver');
            equal(input.checked, false, 'failed receiver write cannot mutate target');
        }
    "#);
}

#[test]
fn reflect_author_accessors_receive_the_explicit_receiver() {
    run(r#"
        const input = document.createElement('input'), receiver = {state: 'receiver state'}, calls = [];
        input.checked = false;
        Object.defineProperty(input, 'checked', {
            configurable: true,
            get() { calls.push(['get', this]); return this.state; },
            set(value) { calls.push(['set', this, value]); this.state = value; }
        });
        equal(Reflect.get(input, 'checked', receiver), 'receiver state', 'author getter uses explicit receiver');
        equal(Reflect.set(input, 'checked', 'updated receiver', receiver), true, 'author setter accepts plain receiver');
        equal(receiver.state, 'updated receiver', 'author setter updates explicit receiver');
        check(calls.length === 2 && calls.every(call => call[1] === receiver), 'both author accessors preserve receiver identity');
        check(!Object.hasOwn(input, 'state'), 'author receiver write does not mutate target expando');
        delete input.checked;
        equal(input.checked, false, 'author receiver write does not mutate target native state');
    "#);
}

#[test]
fn reflect_writable_data_descriptor_creates_receiver_property_without_native_setter() {
    run(r#"
        const target = document.createElement('input'), receiver = document.createElement('input');
        target.checked = false;
        receiver.checked = true;
        Object.defineProperty(target, 'checked', {value: 'target own value', writable: true, configurable: true});
        equal(Reflect.set(target, 'checked', 'receiver own value', receiver), true, 'ordinary data write succeeds');
        equal(target.checked, 'target own value', 'data write preserves target own value');
        const own = Object.getOwnPropertyDescriptor(receiver, 'checked');
        check(own && own.value === 'receiver own value', 'data write creates receiver own data');
        check(own.writable && own.enumerable && own.configurable, 'new receiver data has ordinary flags');
        delete receiver.checked;
        equal(receiver.checked, true, 'data write did not invoke receiver inherited native setter');
        delete target.checked;
        equal(target.checked, false, 'data write did not mutate target native state');
        Object.defineProperty(target, 'checked', {value: 'read-only target', writable: false, configurable: true});
        equal(Reflect.set(target, 'checked', false, receiver), false, 'nonwritable target rejects distinct receiver write');
        equal(receiver.checked, true, 'rejected data write leaves receiver native state unchanged');
    "#);
}

#[test]
fn deleting_or_replacing_native_prototype_accessor_changes_ordinary_lookup() {
    run(r#"
        const prototype = HTMLInputElement.prototype;
        const descriptor = Object.getOwnPropertyDescriptor(prototype, 'checked');
        check(descriptor && descriptor.get && descriptor.set, 'native checked descriptor exists');
        const input = document.createElement('input');
        input.checked = false;
        check(delete prototype.checked, 'native checked descriptor is configurable');
        check(!('checked' in input), 'deleted native property is absent from membership');
        equal(input.checked, undefined, 'raw native projection does not resurrect deleted getter');
        input.checked = 'ordinary after deletion';
        equal(Object.getOwnPropertyDescriptor(input, 'checked').value, 'ordinary after deletion', 'deleted native setter becomes ordinary data write');
        equal(descriptor.get.call(input), false, 'ordinary write leaves saved native state unchanged');
        delete input.checked;
        const state = Symbol('replacement');
        Object.defineProperty(prototype, 'checked', {
            configurable: true,
            get() { return this[state]; },
            set(value) { this[state] = value; }
        });
        input.checked = 'replacement value';
        equal(input.checked, 'replacement value', 'replacement prototype descriptor controls get and set');
        equal(descriptor.get.call(input), false, 'replacement setter leaves native state unchanged');
        Object.defineProperty(prototype, 'checked', descriptor);
        equal(input.checked, false, 'restored native descriptor exposes original native state');
    "#);
}

#[test]
fn saved_input_value_accessors_reject_other_control_interfaces_before_conversion() {
    run(r#"
        const descriptor = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value');
        check(descriptor && descriptor.get && descriptor.set, 'input owns native value accessors');
        let conversions = 0;
        const conversionError = {from: 'conversion'};
        const value = {toString() { ++conversions; throw conversionError; }};
        for (const receiver of [document.createElement('textarea'), document.createElement('select'),
                                document.createElement('div'), {}, Object.create(HTMLInputElement.prototype)]) {
            throwsTypeError(() => descriptor.get.call(receiver), 'input getter rejects other native interface');
            throwsTypeError(() => descriptor.set.call(receiver, value), 'input setter rejects wrong brand before conversion');
            equal(conversions, 0, 'wrong receiver cannot run author conversion code');
        }
        const input = document.createElement('input');
        input.value = 'original';
        throwsSame(() => descriptor.set.call(input, value), conversionError, 'valid receiver preserves conversion exception identity');
        equal(conversions, 1, 'valid receiver converts once');
        equal(input.value, 'original', 'failed conversion does not mutate native value');
    "#);
}

#[test]
fn own_id_and_text_content_descriptors_shadow_inherited_native_accessors() {
    run(r#"
        const element = document.createElement('div');
        element.setAttribute('id', 'native id');
        const child = document.createTextNode('native text');
        element.appendChild(child);
        const value = {author: 'id'}, calls = [];
        Object.defineProperty(element, 'id', {value, writable: true, configurable: true});
        Object.defineProperty(element, 'textContent', {
            configurable: true,
            get() { calls.push(['get', this]); return 'author text'; },
            set(value) { calls.push(['set', this, value]); }
        });
        equal(element.id, value, 'own id data shadows native getter');
        element.id = 'author id';
        equal(element.id, 'author id', 'own id write');
        element.textContent = 'discarded only by author setter';
        equal(element.textContent, 'author text', 'own textContent getter');
        check(calls.length === 2 && calls.every(call => call[1] === element), 'own textContent accessors execute with receiver');
        equal(element.getAttribute('id'), 'native id', 'own id preserves native attribute');
        equal(element.firstChild, child, 'own textContent preserves native child');
        delete element.id;
        delete element.textContent;
        equal(element.id, 'native id', 'deletion restores native id getter');
        equal(element.textContent, 'native text', 'deletion restores native textContent getter');
    "#);
}

#[test]
fn own_and_inherited_method_shadows_override_native_projection() {
    run(r#"
        for (const [member, arity] of [['getAttribute', 1], ['setAttribute', 2], ['querySelector', 1]]) {
            const descriptor = Object.getOwnPropertyDescriptor(Element.prototype, member);
            check(descriptor && descriptor.writable && descriptor.enumerable && descriptor.configurable,
                  member + ' has ordinary operation flags');
            equal(descriptor.value.name, member, member + ' operation name');
            equal(descriptor.value.length, arity, member + ' required argument count');
        }
        const element = document.createElement('div'), calls = [];
        element.setAttribute('data-state', 'native');
        const authorMethod = function(name) { calls.push(['call', this, name]); return 'author result'; };
        Object.defineProperty(element, 'getAttribute', {
            configurable: true,
            get() { calls.push(['get', this]); return authorMethod; }
        });
        equal(element.getAttribute('data-state'), 'author result', 'own getter shadows native method');
        check(calls.length === 2 && calls.every(call => call[1] === element), 'method getter and author call each execute with receiver');
        delete element.getAttribute;
        equal(element.getAttribute('data-state'), 'native', 'deleting own method shadow restores native method');
        const authorPrototype = Object.create(HTMLElement.prototype);
        Object.defineProperty(authorPrototype, 'getAttribute', {value: authorMethod, writable: true, configurable: true});
        Object.setPrototypeOf(element, authorPrototype);
        equal(element.getAttribute('data-state'), 'author result', 'nearer inherited method shadows native method');
        element.getAttribute = function(name) { return 'own replacement:' + name; };
        equal(element.getAttribute('data-state'), 'own replacement:data-state', 'ordinary assignment overrides inherited writable method');
        check(Object.hasOwn(element, 'getAttribute'), 'method assignment creates own property');
        delete element.getAttribute;
        delete authorPrototype.getAttribute;
        equal(element.getAttribute('data-state'), 'native', 'removing both method shadows restores native method');
    "#);
}

#[test]
fn node_child_nodes_is_readonly_same_object_on_node_prototype() {
    run(r#"
        const descriptor = Object.getOwnPropertyDescriptor(Node.prototype, 'childNodes');
        check(descriptor && typeof descriptor.get === 'function' && descriptor.set === undefined,
              'Node owns readonly childNodes accessor');
        check(descriptor.enumerable && descriptor.configurable, 'childNodes has ordinary IDL flags');
        equal(descriptor.get.name, 'get childNodes', 'readonly getter exposes Web IDL function name');
        equal(descriptor.get.length, 0, 'readonly getter arity');
        check(!Object.hasOwn(Element.prototype, 'childNodes'), 'Element inherits childNodes');
        const element = document.createElement('div'), child = document.createTextNode('text');
        const children = element.childNodes;
        equal(element.childNodes, children, 'childNodes preserves SameObject identity');
        equal(Reflect.set(element, 'childNodes', []), false, 'childNodes has no setter');
        throwsTypeError(() => { 'use strict'; element.childNodes = []; }, 'strict childNodes assignment rejects');
        check(!Object.hasOwn(element, 'childNodes'), 'rejected write creates no own property');
        element.appendChild(child);
        equal(children.length, 1, 'retained NodeList updates after native mutation');
        equal(children[0], child, 'retained NodeList contains original child');
        equal(descriptor.get.call(child).length, 0, 'childNodes also applies to CharacterData');
        throwsTypeError(() => descriptor.get.call({}), 'childNodes getter brands Node receiver');
        Object.defineProperty(element, 'childNodes', {value: 'author override', configurable: true});
        equal(element.childNodes, 'author override', 'readonly native accessor remains shadowable');
        delete element.childNodes;
        equal(element.childNodes, children, 'deleting shadow restores the same native NodeList');
    "#);
}

#[test]
fn parent_node_children_accessors_have_interface_ownership_and_readonly_semantics() {
    run(r#"
        check(!Object.hasOwn(Node.prototype, 'children'), 'children is not a Node member');
        for (const [prototype, receiver] of [
            [Element.prototype, document.createElement('div')],
            [Document.prototype, document],
            [DocumentFragment.prototype, document.createDocumentFragment()]
        ]) {
            const descriptor = Object.getOwnPropertyDescriptor(prototype, 'children');
            check(descriptor && typeof descriptor.get === 'function' && descriptor.set === undefined,
                  'ParentNode implementer owns readonly children');
            check(descriptor.enumerable && descriptor.configurable, 'children has ordinary IDL flags');
            const children = receiver.children;
            equal(receiver.children, children, 'children preserves SameObject identity');
            equal(Reflect.set(receiver, 'children', []), false, 'children readonly Reflect.set');
            throwsTypeError(() => { 'use strict'; receiver.children = []; }, 'children strict write rejects');
            equal(receiver.children, children, 'failed write retains native collection');
            throwsTypeError(() => descriptor.get.call(document.createTextNode('x')), 'children getter rejects CharacterData');
            throwsTypeError(() => descriptor.get.call({}), 'children getter rejects plain object');
        }
        const fragment = document.createDocumentFragment(), elements = fragment.children;
        fragment.appendChild(document.createTextNode('text'));
        const child = document.createElement('span');
        fragment.appendChild(child);
        equal(elements.length, 1, 'live children excludes text');
        equal(elements[0], child, 'live children retains native element identity');
        check(!('children' in document.createTextNode('x')), 'CharacterData does not inherit ParentNode children');
    "#);
}

#[test]
fn character_data_owns_data_accessor_and_keeps_native_brand_after_prototype_change() {
    run(r#"
        check(typeof CharacterData === 'function', 'CharacterData is exposed');
        equal(Object.getPrototypeOf(Text.prototype), CharacterData.prototype, 'Text inherits CharacterData');
        equal(Object.getPrototypeOf(Comment.prototype), CharacterData.prototype, 'Comment inherits CharacterData');
        check(!Object.hasOwn(Node.prototype, 'data'), 'data is not a Node member');
        check(!Object.hasOwn(Text.prototype, 'data'), 'Text inherits data');
        const descriptor = Object.getOwnPropertyDescriptor(CharacterData.prototype, 'data');
        check(descriptor && descriptor.get && descriptor.set && descriptor.enumerable && descriptor.configurable,
              'CharacterData owns ordinary native data accessor');
        for (const node of [document.createTextNode('initial'), document.createComment('initial')]) {
            equal(descriptor.get.call(node), 'initial', 'native CharacterData getter');
            descriptor.set.call(node, null);
            equal(node.data, '', 'CharacterData null converts to empty string');
            descriptor.set.call(node, undefined);
            equal(node.data, 'undefined', 'only null converts to empty string');
            Object.setPrototypeOf(node, null);
            descriptor.set.call(node, 'changed');
            equal(descriptor.get.call(node), 'changed', 'native CharacterData brand survives prototype removal');
        }
        const element = document.createElement('div');
        throwsTypeError(() => descriptor.get.call(element), 'data getter rejects Element');
        throwsTypeError(() => descriptor.set.call(element, 'x'), 'data setter rejects Element');
        element.data = 'author data';
        equal(element.data, 'author data', 'unrelated data remains an expando');
    "#);
}

#[test]
fn inner_html_belongs_to_element_and_shadow_root_but_not_document_fragment() {
    run(r#"
        const elementDescriptor = Object.getOwnPropertyDescriptor(Element.prototype, 'innerHTML');
        const shadowDescriptor = Object.getOwnPropertyDescriptor(ShadowRoot.prototype, 'innerHTML');
        for (const descriptor of [elementDescriptor, shadowDescriptor]) {
            check(descriptor && descriptor.get && descriptor.set, 'innerHTML has owning prototype accessors');
            check(descriptor.enumerable && descriptor.configurable, 'innerHTML has ordinary IDL flags');
        }
        check(!Object.hasOwn(DocumentFragment.prototype, 'innerHTML'), 'ordinary fragment has no innerHTML member');
        const element = document.createElement('div'), text = document.createTextNode('element text');
        element.appendChild(text);
        equal(elementDescriptor.get.call(element), 'element text', 'Element getter serializes native children');
        const root = element.attachShadow({mode: 'open'});
        root.appendChild(document.createTextNode('shadow text'));
        equal(shadowDescriptor.get.call(root), 'shadow text', 'ShadowRoot getter serializes native children');
        throwsTypeError(() => elementDescriptor.get.call(root), 'Element accessor rejects ShadowRoot receiver');
        throwsTypeError(() => shadowDescriptor.get.call(element), 'ShadowRoot accessor rejects Element receiver');
        const fragment = document.createDocumentFragment(), child = document.createTextNode('fragment text');
        fragment.appendChild(child);
        check(!('innerHTML' in fragment), 'ordinary fragment has no inherited innerHTML');
        const authorValue = {markup: '<b>author</b>'};
        fragment.innerHTML = authorValue;
        equal(fragment.innerHTML, authorValue, 'fragment retains own innerHTML expando without conversion');
        equal(fragment.firstChild, child, 'fragment expando never parses or changes children');
        let conversions = 0;
        const wrongReceiverValue = {toString() { ++conversions; return '<b>never parsed</b>'; }};
        for (const descriptor of [elementDescriptor, shadowDescriptor]) {
            throwsTypeError(() => descriptor.set.call(fragment, wrongReceiverValue), 'innerHTML setter rejects ordinary fragment');
        }
        equal(conversions, 0, 'wrong innerHTML receiver is rejected before string conversion');
    "#);
}

#[test]
fn nullable_and_dom_string_properties_apply_their_own_null_conversion() {
    run(r#"
        for (const tag of ['input', 'textarea']) {
            const control = document.createElement(tag);
            control.value = 'initial';
            control.value = null;
            equal(control.value, '', tag + ' value uses LegacyNullToEmptyString');
            control.value = undefined;
            equal(control.value, 'undefined', tag + ' value does not treat undefined as null');
        }
        const select = document.createElement('select');
        for (const value of ['initial', 'null', 'undefined']) {
            const option = document.createElement('option');
            option.setAttribute('value', value);
            option.appendChild(document.createTextNode(value));
            select.appendChild(option);
        }
        select.value = null;
        equal(select.value, 'null', 'select uses ordinary DOMString null conversion');
        select.value = undefined;
        equal(select.value, 'undefined', 'select uses ordinary DOMString undefined conversion');
        const element = document.createElement('div');
        for (const member of ['id', 'className']) {
            element[member] = null;
            equal(element[member], 'null', member + ' uses ordinary DOMString null conversion');
            element[member] = undefined;
            equal(element[member], 'undefined', member + ' uses ordinary DOMString undefined conversion');
        }
        const text = document.createTextNode('initial');
        for (const member of ['nodeValue', 'textContent']) {
            text[member] = null;
            equal(text.data, '', member + ' nullable null clears text');
            text.data = 'initial';
            text[member] = undefined;
            equal(text.data, '', member + ' nullable undefined clears text');
        }
    "#);
}

#[test]
fn native_dom_string_setters_reject_symbols_and_preserve_conversion_exception_identity() {
    run(r#"
        const input = document.createElement('input'), textarea = document.createElement('textarea');
        const select = document.createElement('select'), option = document.createElement('option');
        option.setAttribute('value', 'original');
        select.appendChild(option);
        const element = document.createElement('div'), text = document.createTextNode('original');
        const cases = [[input, 'value'], [textarea, 'value'], [select, 'value'],
                       [element, 'id'], [element, 'className'], [text, 'data'], [text, 'textContent'], [text, 'nodeValue']];
        for (const [receiver, member] of cases) {
            receiver[member] = 'original';
            throwsTypeError(() => { receiver[member] = Symbol('invalid'); }, member + ' rejects Symbol DOMString');
            equal(receiver[member], 'original', member + ' Symbol failure preserves native state');
            const sentinel = {member}, conversions = [];
            const value = {
                toString() { conversions.push('toString'); throw sentinel; },
                valueOf() { conversions.push('valueOf'); return 'unreachable'; }
            };
            throwsSame(() => { receiver[member] = value; }, sentinel, member + ' preserves thrown conversion object');
            check(conversions.length === 1 && conversions[0] === 'toString', member + ' converts once with string hint');
            equal(receiver[member], 'original', member + ' conversion failure preserves native state');
        }
        // A throwing conversion proves the valid innerHTML setter boundary
        // without invoking fragment parsing in this public-binding fixture.
        const root = element.attachShadow({mode: 'open'});
        for (const receiver of [element, root]) {
            const sentinel = Symbol('innerHTML conversion');
            throwsSame(() => { receiver.innerHTML = {toString() { throw sentinel; }}; }, sentinel,
                       'innerHTML preserves thrown conversion symbol before parsing');
            throwsTypeError(() => { receiver.innerHTML = Symbol('invalid'); }, 'innerHTML rejects Symbol before parsing');
            equal(receiver.childNodes.length, 0, 'failed innerHTML conversion never changes native children');
        }
    "#);
}

#[test]
fn author_event_handler_shadow_preserves_native_handler_identity() {
    run(r#"
        const element = document.createElement('button'), authorWrites = [];
        const nativeHandler = function(event) { return event; };
        element.onclick = nativeHandler;
        equal(element.onclick, nativeHandler, 'native handler getter retains function identity');
        let authorState = 'author initial';
        Object.defineProperty(element, 'onclick', {
            configurable: true,
            get() { return authorState; },
            set(value) { authorWrites.push([this, value]); authorState = value; }
        });
        const authorHandler = function() { throw new Error('author expando must not become a native listener'); };
        element.onclick = authorHandler;
        equal(element.onclick, authorHandler, 'own author getter and setter override native handler attribute');
        check(authorWrites.length === 1 && authorWrites[0][0] === element, 'own handler setter executes once with receiver');
        delete element.onclick;
        equal(element.onclick, nativeHandler, 'deleting shadow exposes retained native handler');
        element.onclick = null;
        equal(element.onclick, null, 'native null assignment removes handler');
        const text = document.createTextNode('text');
        check(!('onclick' in text), 'CharacterData does not inherit GlobalEventHandlers');
        text.onclick = authorHandler;
        equal(text.onclick, authorHandler, 'unrelated onclick property is an ordinary expando');
    "#);
}

#[test]
fn document_ready_state_handler_uses_source_lenient_this_and_document_only_ownership() {
    run(r#"
        const descriptor = Object.getOwnPropertyDescriptor(Document.prototype, 'onreadystatechange');
        check(descriptor && descriptor.get && descriptor.set && descriptor.enumerable && descriptor.configurable,
              'Document owns ordinary onreadystatechange accessor');
        for (const prototype of [Node.prototype, Element.prototype, HTMLElement.prototype]) {
            check(!Object.hasOwn(prototype, 'onreadystatechange'), 'readystatechange is not installed on unrelated prototype');
        }
        const handler = function() {};
        document.onreadystatechange = handler;
        equal(document.onreadystatechange, handler, 'document handler native getter');
        for (const receiver of [{}, document.createElement('div'), document.createTextNode('x'), null, undefined]) {
            equal(descriptor.get.call(receiver), undefined, 'LegacyLenientThis wrong receiver getter returns undefined');
            equal(descriptor.set.call(receiver, handler), undefined, 'LegacyLenientThis wrong receiver setter returns undefined');
        }
        equal(document.onreadystatechange, handler, 'wrong receiver calls cannot change document handler');
        const element = document.createElement('div');
        element.onreadystatechange = 'author value';
        equal(element.onreadystatechange, 'author value', 'ordinary unrelated assignment creates own expando');
        document.onreadystatechange = null;
    "#);
}

#[test]
fn fragment_get_element_by_id_uses_the_selected_receiver_tree() {
    run(r#"
        const fragment = document.createDocumentFragment(), host = document.createElement('div');
        const shadow = host.attachShadow({mode: 'open'});
        const documentChild = document.createElement('span'), fragmentChild = document.createElement('span');
        const shadowChild = document.createElement('span');
        for (const element of [documentChild, fragmentChild, shadowChild]) element.id = 'same-id';
        document.body.appendChild(documentChild);
        fragment.appendChild(fragmentChild);
        shadow.appendChild(shadowChild);
        equal(document.getElementById('same-id'), documentChild, 'document lookup searches document tree');
        equal(fragment.getElementById('same-id'), fragmentChild, 'fragment lookup searches its own tree');
        equal(shadow.getElementById('same-id'), shadowChild, 'shadow lookup searches its own tree');
        equal(fragment.getElementById('missing'), null, 'fragment missing id returns null');
        check(!Object.hasOwn(Element.prototype, 'getElementById'), 'Element does not implement NonElementParentNode');
    "#);
}

#[test]
fn event_listener_cycles_follow_target_and_dispatch_lifetimes() {
    run_steps(&[
        r#"
        globalThis.callbackWeakRefs = [];
        globalThis.gcTargetRefs = {plain:[], removed:[], cycle:[], handler:[], dispatched:[]};
        for (const kind of Object.keys(gcTargetRefs)) {
            for (let i=0;i<80;i++) {
                const target = document.createElement('i');
                if (kind === 'handler') target.onclick = () => target.nodeType;
                else if (kind !== 'plain') {
                    const callback = () => target.nodeType;
                    target.addEventListener('ping', callback);
                    if (kind === 'removed') target.removeEventListener('ping', callback);
                    if (kind === 'dispatched') target.dispatchEvent(new Event('ping'));
                }
                gcTargetRefs[kind].push(new WeakRef(target));
            }
        }
        globalThis.gcHeldTarget = document.createElement('button');
        globalThis.gcOnceCalls = 0; globalThis.gcHandlerCalls = 0;
        (() => {
            const once = () => { gcOnceCalls++; gcHeldTarget.dispatchEvent(new Event('ping')); };
            const handler = () => { gcHandlerCalls++; };
            callbackWeakRefs.push(new WeakRef(once), new WeakRef(handler));
            gcHeldTarget.addEventListener('ping', once, {once:true});
            gcHeldTarget.onclick = handler;
        })();
        globalThis.churnListenerGarbage = () => {
            for (let i=0;i<50000;i++) { const cycle = {}; cycle.self = cycle; }
        };
        "#,
        r#"for (let i=0;i<4;i++) churnListenerGarbage();"#,
        r#"
        for (const [kind, refs] of Object.entries(gcTargetRefs))
            equal(refs.filter(ref=>ref.deref()!==undefined).length, 0, kind+' targets are reclaimable');
        check(callbackWeakRefs.every(ref=>ref.deref()!==undefined), 'live target owns callbacks');
        gcHeldTarget.dispatchEvent(new Event('ping'));
        gcHeldTarget.dispatchEvent(new Event('ping'));
        equal(gcOnceCalls, 1, 'once callback survives edge removal and nested dispatch');
        gcHeldTarget.dispatchEvent(new Event('click'));
        equal(gcHandlerCalls, 1, 'attribute handler survives GC');
        gcHeldTarget.onclick = null;
        "#,
        r#"for (let i=0;i<4;i++) churnListenerGarbage();"#,
        r#"
        check(callbackWeakRefs.every(ref=>ref.deref()===undefined), 'removed callbacks are reclaimable');
        "#,
    ]);
}

#[test]
fn mutation_observer_interest_and_node_list_lifetime_follow_native_ownership() {
    run_steps(&[
        r#"
        const root=document.getElementById('root'), target=document.createElement('section');
        root.appendChild(target);
        const text=document.createTextNode('before');target.appendChild(text);
        const childOnly=new MutationObserver(()=>{});
        childOnly.observe(root,{childList:true,subtree:true});
        for(let i=0;i<100;i++)target.setAttribute('data-unused',String(i));
        equal(childOnly.takeRecords().length,0,'unrelated mutation type');
        const observer=new MutationObserver(()=>{});
        observer.observe(target,{attributes:true,attributeOldValue:true,attributeFilter:['data-state']});
        observer.observe(root,{attributes:true,subtree:true});
        target.setAttribute('data-state','one');target.setAttribute('data-state','two');
        target.setAttribute('data-other','value');
        globalThis.heldAttributeRecords=observer.takeRecords();
        equal(heldAttributeRecords.length,3,'one record per observer despite two matching registrations');
        equal(heldAttributeRecords[0].oldValue,null,'absent old attribute');
        equal(heldAttributeRecords[1].oldValue,'one','merged old-value interest');
        equal(heldAttributeRecords[2].oldValue,null,'other registration did not request old value');
        equal(heldAttributeRecords[0].attributeNamespace,null,'null attribute namespace');
        check(heldAttributeRecords[0].addedNodes instanceof NodeList,'lazy empty NodeList');
        equal(heldAttributeRecords[0].addedNodes,heldAttributeRecords[0].addedNodes,'stable added list');
        equal(heldAttributeRecords[0].removedNodes.length,0,'empty removed list');
        globalThis.emptyListRefs=[new WeakRef(heldAttributeRecords[0].addedNodes),new WeakRef(heldAttributeRecords[0].removedNodes)];
        observer.observe(root,{childList:true});
        target.setAttribute('data-other','updated');equal(observer.takeRecords().length,0,'reobserve replaces type options');
        const chars=new MutationObserver(()=>{});chars.observe(target,{characterData:true,subtree:true,characterDataOldValue:true});
        text.data='after';const changes=chars.takeRecords();
        equal(changes.length,1,'character-data interest');equal(changes[0].oldValue,'before','character-data old value');
        equal(changes[0].addedNodes.length,0,'character data has an empty added list');
        const host=document.createElement('div');root.appendChild(host);
        const shadow=host.attachShadow({mode:'open'}), inside=document.createElement('i');shadow.appendChild(inside);
        observer.takeRecords();observer.observe(root,{attributes:true,subtree:true});
        inside.setAttribute('data-state','hidden');equal(observer.takeRecords().length,0,'raw DOM ancestry stops at shadow root');
        const shadowObserver=new MutationObserver(()=>{});shadowObserver.observe(shadow,{attributes:true,subtree:true});
        inside.setAttribute('data-state','seen');equal(shadowObserver.takeRecords().length,1,'explicit shadow observation');
        let removed=document.createElement('em');removed.authorState=91;target.appendChild(removed);childOnly.takeRecords();
        globalThis.removedNodeRef=new WeakRef(removed);target.removeChild(removed);
        globalThis.heldRemovedList=childOnly.takeRecords()[0].removedNodes;removed=null;
        childOnly.disconnect();observer.disconnect();chars.disconnect();shadowObserver.disconnect();
        target.setAttribute('data-state','disconnected');equal(observer.takeRecords().length,0,'disconnected observer');
        globalThis.churnMutationGarbage=()=>{for(let i=0;i<50000;i++){const cycle={};cycle.self=cycle;}};
        "#,
        r#"for(let i=0;i<4;i++)churnMutationGarbage();"#,
        r#"
        equal(heldRemovedList[0].authorState,91,'retained removed NodeList traces its node wrapper');
        equal(removedNodeRef.deref(),heldRemovedList[0],'removed wrapper identity survives collection');
        equal(emptyListRefs[0].deref(),heldAttributeRecords[0].addedNodes,'record owns lazy list');
        globalThis.heldRemovedList=null;globalThis.heldAttributeRecords=null;
        "#,
        r#"for(let i=0;i<4;i++)churnMutationGarbage();"#,
        r#"
        check(emptyListRefs.every(ref=>ref.deref()===undefined),'unreferenced lazy lists are reclaimable');
        equal(removedNodeRef.deref(),undefined,'unreferenced removed wrapper is reclaimable');
        "#,
    ]);
}
