#![allow(non_snake_case)]

#[cfg(test)]
extern crate layoutng_replaced as _;

pub mod dom_bindings;
mod url_record;
pub mod web_api_engine;
pub mod window_bindings;

// The C++ Web API implementation evaluates these JavaScript source strings
// through its runtime. Keep their source text unchanged while the host bindings
// are ported to Rust; no C++ runtime code is linked or called.

// cpp: webapi/dom_webidl.inc:1-515
pub const DOM_WEBIDL_SOURCE: &str = include_str!("dom_webidl.js");
// cpp: webapi/window_webidl.inc:1-616
pub const WINDOW_WEBIDL_SOURCE: &str = include_str!("window_webidl.js");
// cpp: webapi/animation_webidl.inc:1-119
pub const ANIMATION_WEBIDL_SOURCE: &str = include_str!("animation_webidl.js");

pub fn DOMBootstrapSource() -> &'static str {
    DOM_WEBIDL_SOURCE
}

pub fn WindowBootstrapSource() -> String {
    let mut source =
        String::with_capacity(WINDOW_WEBIDL_SOURCE.len() + ANIMATION_WEBIDL_SOURCE.len());
    source.push_str(WINDOW_WEBIDL_SOURCE);
    source.push_str(ANIMATION_WEBIDL_SOURCE);
    source
}

#[cfg(test)]
mod tests {
    use super::*;
    use javascript::javascript_runtime::{
        HostCall, HostMethodRef, HostOperation, HostResult, HostValue, JavaScriptHostBindings,
        JavaScriptRuntime,
    };
    use javascript::quickjs_javascript_runtime::QuickJsJavaScriptRuntime;
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    struct BootstrapHost {
        installed: Rc<Cell<usize>>,
        wrapped: Rc<Cell<usize>>,
    }

    impl JavaScriptHostBindings for BootstrapHost {
        fn GlobalNames(&self) -> Vec<String> {
            vec![
                "__domInvoke".into(),
                "__domInstall".into(),
                "__domWrap".into(),
                "__request".into(),
                "__cancelRequest".into(),
                "__randomHex".into(),
                "__urlParts".into(),
                "__historyUpdate".into(),
                "performance".into(),
                "location".into(),
            ]
        }

        fn Invoke(&mut self, call: &HostCall<'_>) -> HostResult {
            if call.receiver == 101
                && call.operation == HostOperation::kGet
                && call.member == "href"
            {
                return HostResult {
                    value: HostValue::String("about:blank".into()),
                    ..Default::default()
                };
            }
            if call.receiver != 0 {
                return HostResult {
                    handled: false,
                    ..Default::default()
                };
            }
            if call.operation == HostOperation::kGet {
                if call.member == "performance" || call.member == "location" {
                    return HostResult {
                        value: HostValue::Object(javascript::javascript_runtime::HostObjectRef {
                            id: if call.member == "performance" {
                                100
                            } else {
                                101
                            },
                        }),
                        ..Default::default()
                    };
                }
                return HostResult {
                    value: HostValue::Method(HostMethodRef {
                        receiver: 0,
                        name: call.member.into(),
                    }),
                    ..Default::default()
                };
            }
            if call.operation == HostOperation::kCall {
                match call.member {
                    "__domInstall" => self.installed.set(self.installed.get() + 1),
                    "__domWrap" => self.wrapped.set(self.wrapped.get() + 1),
                    "__domInvoke" => panic!("bootstrap must not invoke a DOM method"),
                    _ => {
                        return HostResult {
                            handled: false,
                            ..Default::default()
                        }
                    }
                }
                return HostResult::default();
            }
            HostResult {
                handled: false,
                ..Default::default()
            }
        }
    }

    #[test]
    fn source_equivalent_dom_bootstrap_installs_prototypes_and_wrappers() {
        let installed = Rc::new(Cell::new(0));
        let wrapped = Rc::new(Cell::new(0));
        let host = Rc::new(RefCell::new(BootstrapHost {
            installed: installed.clone(),
            wrapped: wrapped.clone(),
        }));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(host);
        let result = runtime.Evaluate(&realm, DOMBootstrapSource(), "browser:dom-webidl");
        assert!(result.Succeeded(), "{:?}", result.exception);
        assert!(installed.get() >= 25);
        assert!(wrapped.get() >= 5);
        let constructors = runtime.Evaluate(
            &realm,
            "typeof Document === 'function' && typeof HTMLInputElement === 'function' && typeof NodeList === 'function'",
            "bootstrap-check.js",
        );
        assert!(constructors.Succeeded());
        assert_eq!(
            constructors
                .value
                .Implementation::<javascript::quickjs_javascript_runtime::JsValue>()
                .unwrap()
                .as_boolean(),
            Some(true)
        );
        let window = runtime.Evaluate(&realm, &WindowBootstrapSource(), "browser:window-webidl");
        assert!(window.Succeeded(), "{:?}", window.exception);
    }
}
