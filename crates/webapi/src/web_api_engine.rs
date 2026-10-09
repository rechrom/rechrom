//! Document-scoped Web API state and its typed Page boundary.
//!
//! JavaScript host calls remain synchronous inside the DOM and Window binding
//! implementations. Observable work for other engines is emitted as effects;
//! this object never reaches into Page, navigation, layout, paint or an OS
//! event loop.

use crate::{
    dom_bindings::{DOMBindingsHost, DOMJavaScriptBindings},
    window_bindings::{LocationNavigation, WindowJavaScriptBindings},
};
use dom::{dom_mutation::DOMMutation, DOM};
use foundation::begin_frame::BeginFrameSource;
use javascript::{
    javascript_runtime::{JavaScriptException, JavaScriptRealm, JavaScriptRuntime},
    script_engine::ScriptSource,
};
use std::{cell::RefCell, rc::Rc, sync::Arc, time::Instant};
use xhr_transport::XMLHttpRequestTransport;

/// State changes supplied by Page or its event-loop adapter.
pub enum WebApiMutation {
    SetURL(String),
    SetViewport { width: f64, height: f64, scale: f64 },
    SetPreferredColorScheme(style::PreferredColorScheme),
    SetBeginFrameSource(Option<Arc<dyn BeginFrameSource>>),
    SetDocumentReadyState(String),
}

/// Work made observable outside the JavaScript/Web API boundary.
#[derive(Clone, Debug)]
pub enum WebApiEffect {
    DocumentMutation(DOMMutation),
    Navigate(LocationNavigation),
    AnimationTick { monotonic_time_ms: f64 },
}

/// Owns the document's DOM and Window host objects.
///
/// Runtime execution is injected for each operation. The engine therefore has
/// no thread, clock loop, Page reference or renderer dependency.
pub struct WebApiEngine {
    dom: Rc<RefCell<DOMJavaScriptBindings>>,
    window: Rc<RefCell<WindowJavaScriptBindings>>,
}

impl WebApiEngine {
    pub fn BootstrapSources() -> [ScriptSource; 2] {
        [
            ScriptSource {
                source: crate::DOMBootstrapSource().to_owned(),
                source_name: "browser:dom-webidl".into(),
            },
            ScriptSource {
                source: WindowJavaScriptBindings::BootstrapSource(),
                source_name: "browser:window-webidl".into(),
            },
        ]
    }

    pub fn new(
        document: Rc<RefCell<DOM>>,
        dom_host: DOMBindingsHost,
        xhr: Box<dyn XMLHttpRequestTransport>,
        user_agent: String,
        emit_effect: Rc<dyn Fn(WebApiEffect)>,
    ) -> Self {
        let dom_effects = emit_effect.clone();
        let dom = Rc::new(RefCell::new(DOMJavaScriptBindings::WithHost(
            document,
            Box::new(move |mutation| dom_effects(WebApiEffect::DocumentMutation(mutation.clone()))),
            dom_host,
        )));
        let animation_effects = emit_effect.clone();
        let window = Rc::new(RefCell::new(WindowJavaScriptBindings::new(
            dom.clone(),
            xhr,
            user_agent,
            Some(Box::new(move |monotonic_time_ms| {
                animation_effects(WebApiEffect::AnimationTick { monotonic_time_ms });
            })),
        )));
        let navigation_effects = emit_effect;
        window
            .borrow_mut()
            .SetLocationNavigator(Rc::new(move |navigation| {
                navigation_effects(WebApiEffect::Navigate(navigation));
            }));
        Self { dom, window }
    }

    pub fn DOMBindings(&self) -> &Rc<RefCell<DOMJavaScriptBindings>> {
        &self.dom
    }

    pub fn WindowBindings(&self) -> &Rc<RefCell<WindowJavaScriptBindings>> {
        &self.window
    }

    pub fn BindRuntime(&self, realm: &JavaScriptRealm) {
        self.dom.borrow_mut().BindRuntime(realm);
        self.window.borrow_mut().BindRuntime(realm);
    }

    pub fn ApplyMutation(&self, mutation: WebApiMutation) {
        match mutation {
            WebApiMutation::SetURL(url) => self.window.borrow_mut().SetURL(url),
            WebApiMutation::SetViewport {
                width,
                height,
                scale,
            } => self.window.borrow_mut().SetViewport(width, height, scale),
            WebApiMutation::SetPreferredColorScheme(preference) => {
                self.window.borrow_mut().SetPreferredColorScheme(preference)
            }
            WebApiMutation::SetBeginFrameSource(source) => {
                self.window.borrow_mut().SetBeginFrameSource(source)
            }
            WebApiMutation::SetDocumentReadyState(state) => {
                self.dom.borrow_mut().SetReadyState(state)
            }
        }
    }

    pub fn HasPendingAnimationFrames(&self) -> bool {
        self.window.borrow().HasPendingAnimationFrames()
    }

    pub fn NextTaskDeadline(&self, now: Instant) -> Option<Instant> {
        self.window.borrow().NextTaskDeadline(now)
    }

    pub fn SetTaskRegistrationFlusher(&self, flusher: Rc<dyn Fn()>) {
        self.window.borrow_mut().SetTaskRegistrationFlusher(flusher);
    }

    pub fn RunAnimationFrameCallbacks(
        &self,
        runtime: &mut dyn JavaScriptRuntime,
        realm: &JavaScriptRealm,
        frame_time: Instant,
        report_error: &mut dyn FnMut(&JavaScriptException),
    ) {
        WindowJavaScriptBindings::RunAnimationFrameCallbacks(
            &self.window,
            runtime,
            realm,
            frame_time,
            report_error,
        );
    }

    pub fn RunTaskTurn(
        &self,
        runtime: &mut dyn JavaScriptRuntime,
        realm: &JavaScriptRealm,
        milliseconds: f64,
        report_error: &mut dyn FnMut(&JavaScriptException),
    ) {
        WindowJavaScriptBindings::RunTaskTurn(
            &self.window,
            runtime,
            realm,
            milliseconds,
            report_error,
        );
    }
}
