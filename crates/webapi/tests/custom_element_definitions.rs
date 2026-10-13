//! Autonomous custom element definitions and reactions through the real DOM/Window bindings.
//! Sources: https://html.spec.whatwg.org/multipage/custom-elements.html#dom-customelementregistry-define
//! https://html.spec.whatwg.org/multipage/custom-elements.html#custom-element-reactions
//! https://webidl.spec.whatwg.org/#es-sequence
//! These tests intentionally exercise the global registry, not scoped registries,
//! customized built-ins, or relative promise/reaction ordering. Promise completion
//! is checked only at an explicit runtime microtask checkpoint.
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
    run_with_checkpoint(script, None, &[]);
}

fn run_with_checkpoint(script: &str, after_checkpoint: Option<&str>, expected_errors: &[&str]) {
    let document = Rc::new(RefCell::new(html::html_parser::ParseHTML(
        "<!doctype html><html><head></head><body><main id=root></main></body></html>",
    )));
    let changed = document.clone();
    let dom = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
        document,
        Box::new(move |mutation| {
            ApplyDOMMutations(
                changed.borrow_mut().GetDocumentMut(),
                std::slice::from_ref(mutation),
            );
        }),
    )));
    let window = Rc::new(RefCell::new(WindowJavaScriptBindings::new(
        dom,
        Box::new(NoNetwork),
        "custom-element-definitions-fixture".into(),
        None,
    )));
    let mut runtime = QuickJsJavaScriptRuntime::new();
    let realm = runtime.CreateRealm(window.clone());
    window.borrow_mut().BindRuntime(&realm);
    for (source, name) in [
        (webapi::DOMBootstrapSource().to_owned(), "dom-webidl"),
        (webapi::WindowBootstrapSource(), "window-webidl"),
    ] {
        let result = runtime.Evaluate(&realm, &source, name);
        assert!(result.Succeeded(), "{name}: {:?}", result.exception);
    }
    let result = runtime.Evaluate(
        &realm,
        &format!(
            r#"function check(value, message) {{ if (!value) throw new Error(message); }}
                function same(actual, expected, message) {{
                    check(JSON.stringify(actual) === JSON.stringify(expected),
                          message + ': ' + JSON.stringify(actual));
                }}
                function throwsName(call, name, message) {{
                    let caught;
                    try {{ call(); }} catch (error) {{ caught = error; }}
                    check(caught && caught.name === name, message + ': ' + caught);
                }}
                {script}"#
        ),
        "custom-element-definitions.js",
    );
    assert!(result.Succeeded(), "{:?}", result.exception);
    runtime.PerformMicrotaskCheckpoint();
    let errors = runtime.TakePendingExceptions(&realm);
    assert_eq!(
        errors.len(),
        expected_errors.len(),
        "reported errors: {errors:?}"
    );
    for (error, expected) in errors.iter().zip(expected_errors) {
        assert!(
            error.message.contains(expected),
            "reported error: {error:?}"
        );
    }
    if let Some(script) = after_checkpoint {
        let result = runtime.Evaluate(&realm, script, "custom-element-checkpoint.js");
        assert!(result.Succeeded(), "{:?}", result.exception);
    }
}

#[test]
fn define_finalizes_inherited_static_metadata_before_returning() {
    run(r#"
        const reads = [];
        class BaseElement extends HTMLElement {
            attributeChangedCallback() {}
        }
        BaseElement.elementStyles = [];
        class StyledElement extends BaseElement {
            static get observedAttributes() {
                reads.push(this);
                this.elementStyles = [this.styles];
                return ['data-state'];
            }
        }
        StyledElement.styles = 'button { width:120px; height:40px; }';
        check(StyledElement.elementStyles === BaseElement.elementStyles, 'inherited empty metadata');
        customElements.define('x-static-metadata', StyledElement);
        check(reads.length === 1 && reads[0] === StyledElement, 'define synchronously calls static getter with constructor receiver');
        same(StyledElement.elementStyles, [StyledElement.styles], 'getter finalized subclass styles');
        same(BaseElement.elementStyles, [], 'base metadata unchanged');
        check(document.createElement('x-static-metadata') instanceof StyledElement, 'registered constructor used');
        check(reads.length === 1, 'definition metadata is not reevaluated for instances');
    "#);
}

#[test]
fn define_captures_inherited_callbacks_in_order_and_keeps_the_snapshot() {
    run(r#"
        const reads = [], events = [], observed = ['data-state'];
        class BaseElement extends HTMLElement {}
        class CapturedElement extends BaseElement {}
        const callbacks = {
            connectedCallback() { events.push(['connected', this.localName]); },
            disconnectedCallback() { events.push(['disconnected', this.localName]); },
            connectedMoveCallback() {},
            adoptedCallback() {},
            attributeChangedCallback(name, oldValue, value, namespace) {
                events.push(['attribute', name, oldValue, value, namespace, this.localName]);
            }
        };
        const names = Object.keys(callbacks);
        for (const name of names) Object.defineProperty(BaseElement.prototype, name, {
            configurable: true,
            get() {
                check(this === CapturedElement.prototype, 'callback getter receiver: ' + name);
                reads.push(name);
                return callbacks[name];
            }
        });
        Object.defineProperty(CapturedElement, 'observedAttributes', {
            configurable: true,
            get() { reads.push('observedAttributes'); return observed; }
        });
        customElements.define('x-captured-callbacks', CapturedElement);
        same(reads, names.concat('observedAttributes'), 'definition getter order');
        for (const name of names) Object.defineProperty(BaseElement.prototype, name, {
            configurable: true,
            value() { events.push(['replacement', name]); }
        });
        Object.defineProperty(CapturedElement, 'observedAttributes', {
            get() { throw new Error('observedAttributes reread'); }
        });
        observed.push('data-late');
        const element = document.createElement('x-captured-callbacks');
        element.setAttribute('data-state', 'one');
        element.setAttribute('data-late', 'ignored');
        document.body.appendChild(element);
        document.body.removeChild(element);
        document.body.appendChild(element);
        same(events, [
            ['attribute', 'data-state', null, 'one', null, 'x-captured-callbacks'],
            ['connected', 'x-captured-callbacks'],
            ['disconnected', 'x-captured-callbacks'],
            ['connected', 'x-captured-callbacks']
        ], 'captured callbacks and observed attribute snapshot survive prototype mutation');
        same(reads, names.concat('observedAttributes'), 'callbacks read only during define');
    "#);
}

#[test]
fn observed_attributes_is_not_read_without_an_attribute_callback() {
    run(r#"
        class UnobservedElement extends HTMLElement {
            static get observedAttributes() { throw new Error('must not be read'); }
        }
        customElements.define('x-unobserved', UnobservedElement);
        check(customElements.get('x-unobserved') === UnobservedElement, 'definition succeeds without reading observedAttributes');
        const element = document.createElement('x-unobserved');
        element.setAttribute('data-state', 'one');
    "#);
}

#[test]
fn throwing_definition_getters_roll_back_the_name_constructor_and_running_flag() {
    run(r#"
        for (const phase of ['connectedCallback', 'observedAttributes']) {
            const sentinel = {phase};
            class RetryElement extends HTMLElement { attributeChangedCallback() {} }
            const target = phase === 'observedAttributes' ? RetryElement : RetryElement.prototype;
            Object.defineProperty(target, phase, {
                configurable: true,
                get() { throw sentinel; }
            });
            const name = 'x-retry-' + phase.toLowerCase();
            let caught;
            try { customElements.define(name, RetryElement); } catch (error) { caught = error; }
            check(caught === sentinel, 'original getter exception propagates: ' + phase);
            check(customElements.get(name) === undefined, 'failed definition not published');
            check(customElements.getName(RetryElement) === null, 'failed constructor not reserved');
            Object.defineProperty(target, phase, {
                value: phase === 'observedAttributes' ? [] : function() {}
            });
            customElements.define(name, RetryElement);
            check(customElements.get(name) === RetryElement, 'same name and constructor reusable after failure');
        }
    "#);
}

#[test]
fn lifecycle_callback_values_must_be_callable_or_undefined() {
    run(r#"
        let index = 0;
        for (const member of ['connectedCallback', 'disconnectedCallback', 'connectedMoveCallback', 'adoptedCallback', 'attributeChangedCallback']) {
            for (const value of [null, 7, {}]) {
                class InvalidCallback extends HTMLElement {}
                Object.defineProperty(InvalidCallback.prototype, member, {value, configurable: true});
                const name = 'x-invalid-callback-' + (++index);
                throwsName(() => customElements.define(name, InvalidCallback), 'TypeError', member + ' rejects non-callable');
                check(customElements.get(name) === undefined, 'invalid callback cannot publish definition');
                Object.defineProperty(InvalidCallback.prototype, member, {value: undefined});
                customElements.define(name, InvalidCallback);
            }
        }
    "#);
}

#[test]
fn reentrant_define_is_rejected_and_the_running_flag_is_always_cleared() {
    run(r#"
        class InnerElement extends HTMLElement {}
        class OuterElement extends HTMLElement {
            static get observedAttributes() {
                throwsName(() => customElements.define('x-reentrant-inner', InnerElement), 'NotSupportedError', 'reentrant definition guard');
                return [];
            }
            attributeChangedCallback() {}
        }
        customElements.define('x-reentrant-outer', OuterElement);
        check(customElements.get('x-reentrant-inner') === undefined, 'caught nested define did not publish');
        customElements.define('x-reentrant-inner', InnerElement);
        check(customElements.get('x-reentrant-inner') === InnerElement, 'guard clears after successful outer define');

        let recurse = true;
        class AnotherInner extends HTMLElement {}
        class ThrowingOuter extends HTMLElement {
            static get observedAttributes() {
                if (recurse) customElements.define('x-another-inner', AnotherInner);
                return [];
            }
            attributeChangedCallback() {}
        }
        throwsName(() => customElements.define('x-throwing-outer', ThrowingOuter), 'NotSupportedError', 'uncaught nested define propagates');
        check(customElements.get('x-throwing-outer') === undefined, 'outer registration rolled back');
        check(customElements.get('x-another-inner') === undefined, 'inner registration not published');
        recurse = false;
        customElements.define('x-throwing-outer', ThrowingOuter);
        customElements.define('x-another-inner', AnotherInner);
    "#);
}

#[test]
fn observed_attributes_uses_iterable_domstring_conversion_and_a_value_snapshot() {
    run(r#"
        const steps = [], changes = [];
        const source = [{toString() { steps.push('convert'); return 'data-state'; }}, null, undefined, true];
        class IterableElement extends HTMLElement {
            static get observedAttributes() {
                return {
                    *[Symbol.iterator]() {
                        steps.push('iterate');
                        for (const value of source) yield value;
                        steps.push('done');
                    }
                };
            }
            attributeChangedCallback(name, oldValue, value) { changes.push([name, oldValue, value]); }
        }
        customElements.define('x-iterable-observed', IterableElement);
        same(steps, ['iterate', 'convert', 'done'], 'iterable fully converted synchronously');
        source[0] = 'data-other';
        source.push('data-late');
        const element = document.createElement('x-iterable-observed');
        for (const name of ['data-state', 'null', 'undefined', 'true', 'data-other', 'data-late']) element.setAttribute(name, 'yes');
        same(changes, [
            ['data-state', null, 'yes'], ['null', null, 'yes'],
            ['undefined', null, 'yes'], ['true', null, 'yes']
        ], 'DOMStrings are retained independently of original iterable');
        same(steps, ['iterate', 'convert', 'done'], 'conversion not rerun for attribute changes');
    "#);
}

#[test]
fn invalid_sequences_and_conversion_exceptions_abort_definition() {
    run(r#"
        let index = 0;
        for (const value of [null, false, 1, 'data-state', {}, {0:'data-state', length:1}, [Symbol('attribute')]]) {
            class InvalidSequence extends HTMLElement {
                static get observedAttributes() { return value; }
                attributeChangedCallback() {}
            }
            const name = 'x-invalid-sequence-' + (++index);
            throwsName(() => customElements.define(name, InvalidSequence), 'TypeError', 'invalid sequence conversion');
            check(customElements.get(name) === undefined, 'sequence conversion failure not published');
            Object.defineProperty(InvalidSequence, 'observedAttributes', {value: []});
            customElements.define(name, InvalidSequence);
        }
        for (const phase of ['iterator', 'next', 'string']) {
            const sentinel = {phase};
            const iterable = phase === 'iterator' ? {
                get [Symbol.iterator]() { throw sentinel; }
            } : phase === 'next' ? {
                [Symbol.iterator]() { return {next() { throw sentinel; }}; }
            } : [{toString() { throw sentinel; }}];
            class ThrowingSequence extends HTMLElement {
                static get observedAttributes() { return iterable; }
                attributeChangedCallback() {}
            }
            const name = 'x-throwing-sequence-' + phase;
            let caught;
            try { customElements.define(name, ThrowingSequence); } catch (error) { caught = error; }
            check(caught === sentinel, 'original sequence exception propagates: ' + phase);
            check(customElements.get(name) === undefined, 'throwing sequence not published');
            Object.defineProperty(ThrowingSequence, 'observedAttributes', {value: []});
            customElements.define(name, ThrowingSequence);
        }
    "#);
}

#[test]
fn constructor_and_name_validation_does_not_silently_normalize_definitions() {
    run(r#"
        for (const constructor of [undefined, null, {}, () => {}, async function() {}, function*() {}])
            throwsName(() => customElements.define('x-invalid-constructor', constructor), 'TypeError', 'constructor must be constructable');
        function PrimitivePrototype() {}
        PrimitivePrototype.prototype = 7;
        throwsName(() => customElements.define('x-primitive-prototype', PrimitivePrototype), 'TypeError', 'constructor prototype must be object');
        for (const name of ['plain', 'X-uppercase', '-starts-with-hyphen', 'x-has space', 'x:colon', 'annotation-xml', 'font-face', 'missing-glyph']) {
            throwsName(() => customElements.define(name, class extends HTMLElement {}), 'SyntaxError', 'invalid custom element name ' + name);
        }
        class ValidElement extends HTMLElement {}
        customElements.define('x-valid-name', ValidElement);
        check(customElements.get('x-valid-name') === ValidElement, 'exact valid name resolves');
        check(customElements.get('X-VALID-NAME') === undefined, 'get is case-sensitive');
        throwsName(() => customElements.define('x-valid-name', class extends HTMLElement {}), 'NotSupportedError', 'duplicate name');
        throwsName(() => customElements.define('x-other-name', ValidElement), 'NotSupportedError', 'duplicate constructor');
        let constructions = 0;
        class LazyElement extends HTMLElement { constructor() { super(); ++constructions; } }
        customElements.define('x-lazy-constructor', LazyElement);
        check(constructions === 0, 'validation does not invoke constructor');
        const unicodeName = 'x-\u00b7valid';
        customElements.define(unicodeName, class extends HTMLElement {});
        check(customElements.get(unicodeName) !== undefined, 'valid non-ASCII custom element name');
    "#);
}

#[test]
fn connected_upgrade_delivers_initial_attributes_then_connection_and_later_mutations() {
    run(r#"
        const events = [];
        const element = document.createElement('x-upgrade-lifecycle');
        element.setAttribute('data-first', 'one');
        element.setAttribute('data-second', 'two');
        element.setAttribute('data-unobserved', 'ignored');
        document.body.appendChild(element);
        class UpgradeElement extends HTMLElement {
            static get observedAttributes() { return ['data-first', 'data-second']; }
            constructor() { super(); events.push(['constructor']); }
            attributeChangedCallback(name, oldValue, value, namespace) { events.push(['attribute', name, oldValue, value, namespace]); }
            connectedCallback() { events.push(['connected']); }
            disconnectedCallback() { events.push(['disconnected']); }
        }
        customElements.define('x-upgrade-lifecycle', UpgradeElement);
        check(element instanceof UpgradeElement, 'pre-existing connected element upgraded before define returns');
        same(events, [
            ['constructor'], ['attribute', 'data-first', null, 'one', null],
            ['attribute', 'data-second', null, 'two', null], ['connected']
        ], 'initial attributes precede connectedCallback');
        events.length = 0;
        element.setAttribute('data-first', 'changed');
        element.setAttribute('data-first', 'changed');
        element.removeAttribute('data-second');
        element.removeAttribute('data-second');
        element.setAttribute('data-unobserved', 'still ignored');
        document.body.removeChild(element);
        document.body.appendChild(element);
        same(events, [
            ['attribute', 'data-first', 'one', 'changed', null],
            ['attribute', 'data-first', 'changed', 'changed', null],
            ['attribute', 'data-second', 'two', null, null], ['disconnected'], ['connected']
        ], 'attribute mutation and detach/reinsert use captured lifecycle');
    "#);
}

#[test]
fn reentrant_reactions_drain_existing_element_reactions_in_fifo_order() {
    // The nested setAttribute [CEReactions] scope invokes this element's existing
    // reaction queue, including the remaining initial attribute and connection.
    // Source: HTML's invoke-custom-element-reactions and CEReactions algorithms.
    run(r#"
        const events = [];
        const element = document.createElement('x-reentrant-reactions');
        element.setAttribute('data-first', 'one');
        element.setAttribute('data-second', 'two');
        document.body.appendChild(element);
        class ReentrantElement extends HTMLElement {
            static get observedAttributes() { return ['data-first', 'data-second', 'data-nested']; }
            constructor() { super(); events.push('constructor'); }
            connectedCallback() { events.push('connected'); }
            attributeChangedCallback(name) {
                if (name === 'data-first') {
                    events.push('first:start');
                    this.setAttribute('data-nested', 'three');
                    events.push('first:end');
                } else events.push(name);
            }
        }
        customElements.define('x-reentrant-reactions', ReentrantElement);
        same(events, ['constructor', 'first:start', 'data-second', 'connected', 'data-nested', 'first:end'],
             'reentrant reaction queue ordering');
    "#);
}

#[test]
fn constructor_proxy_is_checked_without_executing_it_or_reading_prototype_early() {
    run(r#"
        const accesses = [];
        let executions = 0, proxy;
        function Candidate() { ++executions; }
        const prototype = Object.create(HTMLElement.prototype);
        const names = ['connectedCallback', 'disconnectedCallback', 'connectedMoveCallback', 'adoptedCallback', 'attributeChangedCallback'];
        for (const name of names) Object.defineProperty(prototype, name, {
            get() { accesses.push('prototype.' + name); return function() {}; }
        });
        Candidate.prototype = prototype;
        for (const name of ['observedAttributes', 'disabledFeatures', 'formAssociated']) Object.defineProperty(Candidate, name, {
            get() {
                check(this === proxy, 'static getter receiver: ' + name);
                return name === 'formAssociated' ? false : [];
            }
        });
        proxy = new Proxy(Candidate, {
            get(target, key, receiver) {
                accesses.push('constructor.' + String(key));
                return Reflect.get(target, key, receiver);
            },
            construct() { ++executions; throw new Error('constructor executed while defining'); }
        });
        customElements.define('x-proxy-order', proxy);
        check(executions === 0, 'neither construct trap nor constructor executes during define');
        same(accesses, [
            'constructor.prototype',
            ...names.map(name => 'prototype.' + name),
            'constructor.observedAttributes', 'constructor.disabledFeatures', 'constructor.formAssociated'
        ], 'one prototype read after constructibility check, then original properties');
        accesses.length = 0;
        throwsName(() => customElements.define('x-proxy-order', proxy), 'NotSupportedError', 'duplicate name preflight');
        throwsName(() => customElements.define('x-proxy-again', proxy), 'NotSupportedError', 'duplicate constructor preflight');
        same(accesses, [], 'duplicate validation does not inspect constructor properties');
    "#);
}

#[test]
fn argument_conversion_precedes_native_constructor_and_name_validation() {
    run(r#"
        const accesses = [];
        const name = {toString() { accesses.push('name'); return 'x-conversion-order'; }};
        const options = {
            get extends() {
                accesses.push('options.extends');
                return {toString() { accesses.push('extends.toString'); return 'button'; }};
            }
        };
        throwsName(() => customElements.define(name, {}, options), 'TypeError', 'callback conversion before options');
        same(accesses, ['name'], 'non-callable constructor prevents options conversion');
        accesses.length = 0;
        throwsName(() => customElements.define(name, () => {}, options), 'TypeError', 'native constructibility follows options conversion');
        same(accesses, ['name', 'options.extends', 'extends.toString'], 'WebIDL argument conversion order');
        accesses.length = 0;
        class ValidConstructor extends HTMLElement {}
        const noExtension = {get extends() { accesses.push('options.extends'); return null; }};
        throwsName(() => customElements.define('invalid', ValidConstructor, noExtension), 'SyntaxError', 'name validity is a native step');
        same(accesses, ['options.extends'], 'dictionary conversion before native name validation');
        const sentinel = {conversion: true};
        const throwingOptions = {get extends() { throw sentinel; }};
        let caught;
        try { customElements.define('x-conversion-retry', ValidConstructor, throwingOptions); } catch (error) { caught = error; }
        check(caught === sentinel, 'options getter exception preserved');
        check(customElements.get('x-conversion-retry') === undefined, 'options failure does not reserve name');
        customElements.define('x-conversion-retry', ValidConstructor, null);
        for (const options of [1, true, 'dictionary'])
            throwsName(() => customElements.define('x-invalid-options', class extends HTMLElement {}, options), 'TypeError', 'primitive dictionary rejected');
        throwsName(() => customElements.define(Symbol('name'), class extends HTMLElement {}), 'TypeError', 'name DOMString rejects Symbol');
        for (const name of ['x-col:on', 'x-eq=ual']) {
            class PunctuationElement extends HTMLElement {}
            customElements.define(name, PunctuationElement);
            check(customElements.get(name) === PunctuationElement, 'pinned valid-element-local-name rule: ' + name);
        }
    "#);
}

#[test]
fn array_sequence_conversion_uses_pinned_blink_index_and_live_length_branch() {
    // The pinned Blink NativeValueTraits array fast path ignores @@iterator and
    // reevaluates array length. This is deliberately source-aligned behavior.
    run(r#"
        const changes = [], conversions = [];
        const observed = [{toString() {
            conversions.push('first');
            delete observed[1];
            observed.push('data-appended');
            return 'data-first';
        }}, 'data-deleted'];
        Object.defineProperty(observed, Symbol.iterator, {
            get() { throw new Error('array iterator must not be inspected'); }
        });
        class ArrayObservedElement extends HTMLElement {
            static get observedAttributes() { return observed; }
            attributeChangedCallback(name) { changes.push(name); }
        }
        customElements.define('x-array-sequence', ArrayObservedElement);
        same(conversions, ['first'], 'array item is converted once');
        const element = document.createElement('x-array-sequence');
        for (const name of ['data-first', 'undefined', 'data-appended', 'data-deleted']) element.setAttribute(name, 'yes');
        same(changes, ['data-first', 'undefined', 'data-appended'], 'numeric access observes hole and appended item');
    "#);
}

#[test]
fn failed_constructor_reports_once_clears_callbacks_and_does_not_block_other_candidates() {
    run_with_checkpoint(
        r#"
        var events = [], attempts = 0;
        const failed = document.createElement('x-terminal-upgrade');
        failed.fixtureName = 'failed';
        failed.setAttribute('data-state', 'before');
        const succeeding = document.createElement('x-terminal-upgrade');
        succeeding.fixtureName = 'succeeding';
        succeeding.setAttribute('data-state', 'before');
        document.body.appendChild(failed);
        document.body.appendChild(succeeding);
        class TerminalElement extends HTMLElement {
            static get observedAttributes() { return ['data-state']; }
            constructor() {
                super();
                events.push('constructor:' + this.fixtureName);
                if (this.fixtureName === 'failed') {
                    ++attempts;
                    throw new Error('custom-upgrade-sentinel');
                }
            }
            attributeChangedCallback() { events.push('attribute:' + this.fixtureName); }
            connectedCallback() { events.push('connected:' + this.fixtureName); }
        }
        customElements.define('x-terminal-upgrade', TerminalElement);
        same(events, ['constructor:failed', 'constructor:succeeding', 'attribute:succeeding', 'connected:succeeding'],
             'failed constructor clears its callbacks without escaping define or blocking other candidates');
        check(customElements.get('x-terminal-upgrade') === TerminalElement, 'definition remains registered');
        failed.__customElementState = undefined;
        customElements.upgrade(failed);
        customElements.upgrade(document.body);
        failed.setAttribute('data-state', 'after');
        document.body.removeChild(failed);
        document.body.appendChild(failed);
        check(attempts === 1, 'failed native element remains terminal despite JS expando changes');
        same(events, ['constructor:failed', 'constructor:succeeding', 'attribute:succeeding', 'connected:succeeding'],
             'terminal element never receives later reactions');
        "#,
        None,
        &["custom-upgrade-sentinel"],
    );
}

#[test]
fn callback_exceptions_are_reported_without_stopping_remaining_reactions() {
    run_with_checkpoint(
        r#"
        const events = [];
        const element = document.createElement('x-throwing-callback');
        element.setAttribute('data-first', 'one');
        element.setAttribute('data-second', 'two');
        document.body.appendChild(element);
        class ThrowingCallback extends HTMLElement {
            static get observedAttributes() { return ['data-first', 'data-second']; }
            attributeChangedCallback(name) {
                events.push(name);
                if (name === 'data-first') throw new Error('custom-callback-sentinel');
            }
            connectedCallback() { events.push('connected'); }
        }
        customElements.define('x-throwing-callback', ThrowingCallback);
        same(events, ['data-first', 'data-second', 'connected'], 'remaining initial reactions run after callback throws');
        "#,
        None,
        &["custom-callback-sentinel"],
    );
}

#[test]
fn registry_and_node_brands_cannot_be_forged_or_replaced_by_author_projections() {
    run(r#"
        const registryPrototype = CustomElementRegistry.prototype;
        for (const receiver of [{}, Object.create(registryPrototype), document]) {
            throwsName(() => registryPrototype.define.call(receiver, 'x-fake-registry', class extends HTMLElement {}), 'TypeError', 'define registry brand');
            throwsName(() => registryPrototype.get.call(receiver, 'x-fake-registry'), 'TypeError', 'get registry brand');
            throwsName(() => registryPrototype.getName.call(receiver, HTMLElement), 'TypeError', 'getName registry brand');
            throwsName(() => registryPrototype.upgrade.call(receiver, document), 'TypeError', 'upgrade registry brand');
        }
        for (const root of [null, {}, Object.create(Node.prototype)])
            throwsName(() => customElements.upgrade(root), 'TypeError', 'upgrade requires a native Node');
        const events = [];
        const element = document.createElement('x-native-candidate');
        const setAttribute = element.setAttribute;
        element.setAttribute('data-state', 'native');
        document.body.appendChild(element);
        element.__customElementState = 'custom';
        Object.defineProperty(element, 'localName', {get() { throw new Error('author localName projection used'); }});
        element.getAttribute = function() { throw new Error('author attribute projection used'); };
        element.querySelectorAll = function() { throw new Error('author element traversal used'); };
        document.querySelectorAll = function() { throw new Error('author document traversal used'); };
        customElements._definitions = new Map();
        class NativeCandidate extends HTMLElement {
            static get observedAttributes() { return ['data-state']; }
            constructor() { super(); events.push('constructor'); }
            attributeChangedCallback(name, oldValue, value) { events.push(name + ':' + value); }
            connectedCallback() { events.push('connected'); }
        }
        customElements.define('x-native-candidate', NativeCandidate);
        same(events, ['constructor', 'data-state:native', 'connected'], 'native identity, attributes and traversal drive upgrade');
        check(element instanceof NativeCandidate, 'expando state cannot suppress upgrade');
        element.attributeChangedCallback = function() { throw new Error('instance callback override used'); };
        setAttribute.call(element, 'data-state', 'updated');
        same(events, ['constructor', 'data-state:native', 'connected', 'data-state:updated'], 'instance callback override does not replace captured callback');
    "#);
}

#[test]
fn upgrades_are_shadow_including_and_detached_candidates_require_explicit_upgrade_or_insertion() {
    run(r#"
        const events = [];
        function candidate(label, parent) {
            const element = document.createElement('x-shadow-candidate');
            element.fixtureName = label;
            if (parent) parent.appendChild(element);
            return element;
        }
        const connectedHost = document.createElement('div');
        const connectedShadow = connectedHost.attachShadow({mode:'closed'});
        const connectedShadowChild = candidate('connected-shadow', connectedShadow);
        const connectedLightChild = candidate('connected-light', connectedHost);
        document.body.appendChild(connectedHost);
        const detachedHost = document.createElement('div');
        const detachedShadow = detachedHost.attachShadow({mode:'closed'});
        const detachedShadowChild = candidate('detached-shadow', detachedShadow);
        const detachedLightChild = candidate('detached-light', detachedHost);
        const detachedSelf = candidate('detached-self');
        const insertedLater = candidate('inserted-later');
        class ShadowCandidate extends HTMLElement {
            constructor() { super(); events.push('constructor:' + this.fixtureName); }
            connectedCallback() { events.push('connected:' + this.fixtureName); }
        }
        customElements.define('x-shadow-candidate', ShadowCandidate);
        same(events, ['constructor:connected-shadow', 'connected:connected-shadow', 'constructor:connected-light', 'connected:connected-light'],
             'define visits connected author shadow before light descendants');
        check(!(detachedShadowChild instanceof ShadowCandidate) && !(detachedLightChild instanceof ShadowCandidate),
              'define leaves disconnected candidates undefined');
        events.length = 0;
        Object.setPrototypeOf(detachedHost, null);
        customElements.upgrade(detachedHost);
        same(events, ['constructor:detached-shadow', 'constructor:detached-light'], 'explicit upgrade uses native root brand and closed-shadow traversal');
        events.length = 0;
        Object.setPrototypeOf(detachedSelf, null);
        customElements.upgrade(detachedSelf);
        same(events, ['constructor:detached-self'], 'explicit upgrade includes the root itself despite prototype tampering');
        events.length = 0;
        document.body.appendChild(insertedLater);
        same(events, ['constructor:inserted-later', 'connected:inserted-later'], 'native insertion upgrades undefined candidate');
    "#);
}

#[test]
fn autonomous_definitions_match_native_html_name_and_ignore_later_is_attributes() {
    run(r#"
        const button = document.createElement('button');
        button.setAttribute('is', 'x-native-name');
        const svg = document.createElementNS('http://www.w3.org/2000/svg', 'x-native-name');
        const unrelated = document.createElement('x-unrelated-name');
        unrelated.setAttribute('is', 'x-native-name');
        const matching = document.createElement('x-native-name');
        for (const element of [button, svg, unrelated, matching]) document.body.appendChild(element);
        const upgraded = [];
        class MatchingElement extends HTMLElement { constructor() { super(); upgraded.push(this); } }
        customElements.define('x-native-name', MatchingElement);
        check(upgraded.length === 1 && upgraded[0] === matching, 'native HTML local name is the only autonomous match');
        check(!(button instanceof MatchingElement) && !(svg instanceof MatchingElement) && !(unrelated instanceof MatchingElement),
              'is content attribute and foreign namespace do not match autonomous definition');
    "#);
}

#[test]
fn when_defined_keeps_pending_identity_and_resolves_after_registration_at_checkpoint() {
    run_with_checkpoint(
        r#"
        var results = [], invalidName;
        var waiting = customElements.whenDefined('x-awaited-definition');
        check(waiting === customElements.whenDefined('x-awaited-definition'), 'one pending promise per exact name');
        waiting.then(value => results.push(value));
        var invalidPromise;
        try { invalidPromise = customElements.whenDefined('invalid'); }
        catch (error) { throw new Error('invalid name must reject the promise rather than throw synchronously'); }
        invalidPromise.catch(error => { invalidName = error.name; });
        var AwaitedElement = class extends HTMLElement {};
        customElements.define('x-awaited-definition', AwaitedElement);
        check(customElements.get('x-awaited-definition') === AwaitedElement, 'definition committed synchronously');
        check(results.length === 0, 'promise reaction waits for checkpoint');
        customElements.whenDefined('x-awaited-definition').then(value => results.push(value));
        "#,
        Some(
            r#"
            check(results.length === 2 && results.every(value => value === AwaitedElement), 'pending and already-defined promises resolve with constructor');
            check(invalidName === 'SyntaxError', 'invalid name rejects with SyntaxError');
        "#,
        ),
        &[],
    );
}

#[test]
fn proxy_wrapped_array_sequence_uses_the_iterable_branch() {
    // Pinned V8 Value::IsArray tests IsJSArray, unlike JS Array.isArray which
    // unwraps proxies. A Proxy-wrapped array follows Blink's generic iterable path.
    run(r#"
        const changes = [], reads = [];
        const target = ['data-index'];
        Object.defineProperty(target, Symbol.iterator, {
            get() {
                reads.push('iterator');
                return function*() { yield 'data-iterated'; };
            }
        });
        const observed = new Proxy(target, {});
        class ProxySequence extends HTMLElement {
            static get observedAttributes() { return observed; }
            attributeChangedCallback(name) { changes.push(name); }
        }
        customElements.define('x-proxy-sequence', ProxySequence);
        same(reads, ['iterator'], 'array Proxy reads the custom iterator');
        const element = document.createElement('x-proxy-sequence');
        element.setAttribute('data-index', 'ignored');
        element.setAttribute('data-iterated', 'observed');
        same(changes, ['data-iterated'], 'array Proxy uses yielded values instead of numeric indexes');
    "#);
}

#[test]
fn generic_sequence_caches_next_and_reads_value_before_done_at_each_step() {
    // Pinned ScriptIterator::Next reads value before done, even for the final
    // record. This intentionally checks the source branch rather than for-of.
    run(r#"
        const accesses = [], changes = [];
        let index = 0;
        const iterator = {
            get next() {
                accesses.push('next-method');
                return function() {
                    check(this === iterator, 'iterator next receiver');
                    const current = index++;
                    accesses.push('next-call');
                    return {
                        get value() {
                            accesses.push('value:' + current);
                            return current === 0 ? 'data-value' : {toString() { throw new Error('final value must not be converted'); }};
                        },
                        get done() { accesses.push('done:' + current); return current > 0; }
                    };
                };
            }
        };
        const observed = {[Symbol.iterator]() { accesses.push('iterator-call'); return iterator; }};
        class OrderedIterator extends HTMLElement {
            static get observedAttributes() { return observed; }
            attributeChangedCallback(name) { changes.push(name); }
        }
        customElements.define('x-iterator-get-order', OrderedIterator);
        same(accesses, ['iterator-call', 'next-method', 'next-call', 'value:0', 'done:0', 'next-call', 'value:1', 'done:1'],
             'pinned iterator extraction order');
        document.createElement('x-iterator-get-order').setAttribute('data-value', 'yes');
        same(changes, ['data-value'], 'only yielded, nonfinal value is converted');
    "#);
}

#[test]
fn generic_sequence_domstring_failure_does_not_close_the_pinned_iterator() {
    run(r#"
        const sentinel = {conversion: 'failed'};
        let returns = 0;
        const observed = {
            [Symbol.iterator]() {
                return {
                    next() { return {value: {toString() { throw sentinel; }}, done: false}; },
                    return() { ++returns; return {done: true}; }
                };
            }
        };
        class FailedIterator extends HTMLElement {
            static get observedAttributes() { return observed; }
            attributeChangedCallback() {}
        }
        let caught;
        try { customElements.define('x-iterator-no-close', FailedIterator); } catch (error) { caught = error; }
        check(caught === sentinel, 'item conversion error preserved');
        check(returns === 0, 'pinned NativeValueTraits error path does not call iterator.return');
        check(customElements.get('x-iterator-no-close') === undefined, 'failed sequence is not committed');
        Object.defineProperty(FailedIterator, 'observedAttributes', {value: []});
        customElements.define('x-iterator-no-close', FailedIterator);
    "#);
}
