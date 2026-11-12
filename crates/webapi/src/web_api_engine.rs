//! Document-scoped Web API state and its typed Page boundary.
//!
//! JavaScript host calls remain synchronous inside the DOM and Window binding
//! implementations. Observable work for other engines is emitted as effects;
//! this object never reaches into Page, navigation, layout, paint or an OS
//! event loop.

use crate::{
    dom_bindings::{DOMBindingsHost, DOMEventListenerInterest, DOMJavaScriptBindings},
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
    SetPrefersReducedMotion(bool),
    SetForcedColors(style::media_queries::forced_colors::ForcedColors),
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
    emit_effect: Rc<dyn Fn(WebApiEffect)>,
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
        Self::WithPerformanceTiming(
            document,
            dom_host,
            xhr,
            user_agent,
            emit_effect,
            Rc::new(crate::window_bindings::WindowPerformanceTiming::new()),
        )
    }
    pub fn WithPerformanceTiming(
        document: Rc<RefCell<DOM>>,
        dom_host: DOMBindingsHost,
        xhr: Box<dyn XMLHttpRequestTransport>,
        user_agent: String,
        emit_effect: Rc<dyn Fn(WebApiEffect)>,
        performance_timing: Rc<crate::window_bindings::WindowPerformanceTiming>,
    ) -> Self {
        let dom_effects = emit_effect.clone();
        let dom = Rc::new(RefCell::new(DOMJavaScriptBindings::WithHost(
            document,
            Box::new(move |mutation| dom_effects(WebApiEffect::DocumentMutation(mutation.clone()))),
            dom_host,
        )));
        let animation_effects = emit_effect.clone();
        let window = Rc::new(RefCell::new(
            WindowJavaScriptBindings::WithPerformanceTiming(
                dom.clone(),
                xhr,
                user_agent,
                Some(Box::new(move |monotonic_time_ms| {
                    animation_effects(WebApiEffect::AnimationTick { monotonic_time_ms });
                })),
                performance_timing,
            ),
        ));
        let navigation_effects = emit_effect.clone();
        WindowJavaScriptBindings::BindPerformanceEffects(&window);
        Self::ConnectAbortTaskRunner(&dom, &window);
        window
            .borrow_mut()
            .SetLocationNavigator(Rc::new(move |navigation| {
                navigation_effects(WebApiEffect::Navigate(navigation));
            }));
        Self {
            dom,
            window,
            emit_effect,
        }
    }
    /// Prepare a new Document binding without constructing another Window or
    /// changing the existing Window association before publication.
    pub fn ForWindowReuse(
        document: Rc<RefCell<DOM>>,
        dom_host: DOMBindingsHost,
        previous: &Self,
        emit_effect: Rc<dyn Fn(WebApiEffect)>,
    ) -> Self {
        let effects = emit_effect.clone();
        let dom = Rc::new(RefCell::new(DOMJavaScriptBindings::WithHost(
            document,
            Box::new(move |mutation| effects(WebApiEffect::DocumentMutation(mutation.clone()))),
            dom_host,
        )));
        {
            let old = previous.dom.borrow();
            let mut new = dom.borrow_mut();
            new.CopyWindowIDLBindingsForReuse(&old);
        }
        Self::ConnectAbortTaskRunner(&dom, &previous.window);
        Self {
            dom,
            window: previous.window.clone(),
            emit_effect,
        }
    }
    pub fn PrepareHostObjectNamespaceForReuse(
        &self,
        runtime: &mut dyn JavaScriptRuntime,
        realm: &JavaScriptRealm,
    ) -> Result<(), JavaScriptException> {
        let namespace = runtime.CreateHostObjectNamespace(realm, self.dom.clone(), false)?;
        self.dom.borrow_mut().BindHostObjectNamespace(&namespace);
        self.dom.borrow_mut().BindRuntime(realm);
        Ok(())
    }
    pub fn PublishDocumentForReuse(
        &self,
        runtime: &mut dyn JavaScriptRuntime,
        realm: &JavaScriptRealm,
    ) {
        let namespace = self
            .dom
            .borrow()
            .HostObjectNamespace()
            .expect("prepared original namespace");
        let url = self.dom.borrow().DocumentURLForWindow();
        runtime.SelectDocumentHostObjectNamespace(realm, &namespace);
        self.dom.borrow().RebindForWindowReuse();
        let mut window = self.window.borrow_mut();
        window.InstallDocumentForReuse(self.dom.clone(), &namespace, url);
        let effects = self.emit_effect.clone();
        window.SetLocationNavigator(Rc::new(move |navigation| {
            effects(WebApiEffect::Navigate(navigation))
        }));
        let effects = self.emit_effect.clone();
        window.SetAnimationTick(Box::new(move |monotonic_time_ms| {
            effects(WebApiEffect::AnimationTick { monotonic_time_ms })
        }));
    }

    pub fn DOMBindings(&self) -> &Rc<RefCell<DOMJavaScriptBindings>> {
        &self.dom
    }
    fn ConnectAbortTaskRunner(
        dom: &Rc<RefCell<DOMJavaScriptBindings>>,
        window: &Rc<RefCell<WindowJavaScriptBindings>>,
    ) {
        let window = Rc::downgrade(window);
        dom.borrow().BindAbortTaskScheduler(Rc::new(
            move |milliseconds, high_nesting, callback| {
                debug_assert_eq!(high_nesting, milliseconds != 0);
                let mut callback = Some(callback);
                window
                    .upgrade()
                    .expect("live AbortSignal Window task runner")
                    .borrow_mut()
                    .EnqueueAbortTimeout(
                        std::time::Duration::from_millis(milliseconds),
                        Box::new(move |runtime, realm| {
                            runtime
                                .WithHostRuntime(realm, &mut |runtime| {
                                    callback.take().expect("one-shot abort timeout")(runtime);
                                })
                                .expect("native AbortSignal timeout realm");
                        }),
                    )
            },
        ));
    }

    pub fn EventListenerInterest(&self) -> DOMEventListenerInterest {
        self.dom.borrow().EventListenerInterest()
    }

    pub fn WindowBindings(&self) -> &Rc<RefCell<WindowJavaScriptBindings>> {
        &self.window
    }

    pub fn BindRuntime(&self, realm: &JavaScriptRealm) {
        self.dom.borrow_mut().BindRuntime(realm);
        self.window.borrow_mut().BindRuntime(realm);
    }

    pub fn BindInitialHostObjectNamespace(
        &self,
        runtime: &mut dyn JavaScriptRuntime,
        realm: &JavaScriptRealm,
    ) -> Result<(), JavaScriptException> {
        let namespace = runtime.CreateHostObjectNamespace(realm, self.dom.clone(), true)?;
        self.dom.borrow_mut().BindHostObjectNamespace(&namespace);
        self.window
            .borrow_mut()
            .BindInitialDocumentNamespace(&namespace);
        Ok(())
    }

    /// Lazy child ScriptController setup can run from a parent host callback.
    /// It uses that Agent's runtime lease without reacquiring ScriptEngine.
    pub fn BindInitialHostObjectNamespaceInHost(
        &self,
        runtime: &mut dyn javascript::javascript_runtime::JavaScriptHostRuntime,
    ) -> Result<(), JavaScriptException> {
        let namespace = runtime.CreateHostObjectNamespace(self.dom.clone(), true)?;
        self.dom.borrow_mut().BindHostObjectNamespace(&namespace);
        self.window
            .borrow_mut()
            .BindInitialDocumentNamespace(&namespace);
        Ok(())
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
            WebApiMutation::SetPrefersReducedMotion(preference) => {
                self.window.borrow_mut().SetPrefersReducedMotion(preference)
            }
            WebApiMutation::SetForcedColors(forced_colors) => {
                self.window.borrow_mut().SetForcedColors(forced_colors)
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
