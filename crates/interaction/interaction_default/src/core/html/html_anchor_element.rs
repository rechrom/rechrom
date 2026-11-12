use crate::bindings::core::v8::script_promise::ScriptInvocation;
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/html/html_anchor_element.cc:137-206,366-430,441-531
// Source: core/html/anchor_element_utils.cc:279-287
// ResourceRequest/FrameLoadRequest remain with the loader owner. These narrow
// host calls preserve their source order and identity, including delayed tasks.

use crate::core::loader::navigation_policy::{
    NavigationEventModifiers, NavigationPolicy, NavigationPolicyFromEvent,
};
use interaction_dispatch::core::{
    dom::events::event::{EventRef, EventTargetRef},
    events::keyboard_event::KeyboardEvent,
};
use interaction_input::base::time::time::TimeTicks;
use std::{cell::RefCell, rc::Rc};

/// Opaque resident loader request. Mutations below run on that same owner.
pub trait AnchorResourceRequest {
    fn AsAny(&self) -> &dyn std::any::Any;
    fn IntoAny(self: Rc<Self>) -> Rc<dyn std::any::Any>;
}
pub trait AnchorNavigationTaskHandle {}
pub trait AnchorFrameLoadRequest {
    fn AsAny(&self) -> &dyn std::any::Any;
    fn IntoAny(self: Rc<Self>) -> Rc<dyn std::any::Any>;
    fn SetNavigationPolicy(&self, policy: NavigationPolicy);
    fn SetClientNavigationReasonAnchorClick(&self);
    fn SetSourceElement(&self, source: &EventTargetRef);
    fn CleanNavigationTarget(&self, target: &str) -> String;
    fn HandleRelAttribute(&self, target: &str, relations: u32);
    fn EnforceBlobUrlNoopenerIfNeeded(&self, completed_url: &str);
    fn SetTriggeringEventInfo(&self, is_trusted: bool);
    fn SetInputStartTime(&self, time: TimeTicks);
    fn Noopener(&self) -> bool;
    fn SetHrefTranslate(&self, value: &str);
}
pub enum AnchorNavigationLookup {
    Ready(Rc<dyn AnchorNavigationTarget>),
    Pending(Rc<dyn AnchorNavigationPending>),
    Denied,
}
pub trait AnchorNavigationPending {
    /// Resumes the source statements AFTER FindOrCreateFrameForNavigation.
    /// Pending never stands in for a fabricated Frame or an accepted window.
    fn Resume(&self, continuation: Box<dyn FnOnce(Option<Rc<dyn AnchorNavigationTarget>>)>);
}
pub trait AnchorNavigationTarget {
    fn IsCurrentFrame(&self) -> bool;
    fn NavigateStandard(&self, request: Rc<dyn AnchorFrameLoadRequest>);
    fn NavigateStandardWithInvocation(
        &self,
        request: Rc<dyn AnchorFrameLoadRequest>,
        invocation: Option<&dyn ScriptInvocation>,
    ) {
        let _ = invocation;
        self.NavigateStandard(request);
    }
}

/// LocalFrame's retained loader/navigation services. DOM facts and source
/// hyperlink decisions stay in HTMLAnchorElementBase; the embedding supplies
/// the live request, FrameTree, network and task owners at this small boundary.
pub trait AnchorNavigationHost {
    fn ServerMapContentPoint(
        &self,
        target: u64,
        absolute: &foundation::gfx::PointF,
    ) -> Option<(f32, f32, f32)>;
    fn CurrentInputNavigationModifiers(&self) -> Option<NavigationEventModifiers>;
    fn NewTabModifierSetFromIsolatedWorld(&self) -> bool;
    fn SendPings(
        &self,
        invocation: Option<&dyn ScriptInvocation>,
        completed_url: &str,
        ping_attribute: Option<&str>,
    );
    fn CanReadContent(&self, completed_url: &str) -> bool;
    fn HandleDownloadAttribute(
        &self,
        invocation: Option<&dyn ScriptInvocation>,
        source: EventTargetRef,
        download: &str,
        completed_url: &str,
        is_trusted: bool,
        request: Rc<dyn AnchorResourceRequest>,
    );
    fn PostDelayedNavigationTask(
        &self,
        task: Box<dyn FnOnce()>,
    ) -> Rc<dyn AnchorNavigationTaskHandle>;
    fn SetDelayedNavigationTaskHandle(&self, handle: Rc<dyn AnchorNavigationTaskHandle>);
    fn CreateHyperlinkFrameLoadRequest(
        &self,
        invocation: Option<&dyn ScriptInvocation>,
        request: Rc<dyn AnchorResourceRequest>,
    ) -> Option<Rc<dyn AnchorFrameLoadRequest>>;
    fn FindOrCreateFrameForNavigation(
        &self,
        request: &Rc<dyn AnchorFrameLoadRequest>,
        target: &str,
    ) -> AnchorNavigationLookup;
    fn HrefTranslateEnabled(&self) -> bool;
}
/// Borrowed dispatch services belong to this synchronous event/default scope;
/// neither HTMLAnchorElement nor its delayed navigation retains a runtime.
pub trait AnchorDefaultActions {
    fn DispatchSimulatedClick(&self, event: &EventRef);
    fn BaseDefaultEventHandler(&self, event: &EventRef);
    /// Live HTMLImageElement/LayoutBox query. Coordinates are AbsoluteToLocal
    /// minus the content origin; zoom is the source EffectiveZoom.
    fn ServerMapPoint(&self, event: &EventRef) -> Option<(f32, f32, f32)>;
}

pub trait AnchorElementEnvironment {
    fn Target(&self) -> EventTargetRef;
    fn IsLink(&self) -> bool;
    fn IsFocused(&self) -> bool;
    fn IsEditable(&self) -> bool;
    fn IsConnected(&self) -> bool;
    fn IsAreaElement(&self) -> bool;
    fn SetBaseActive(&self, active: bool);
    fn KeyboardEvent(&self, event: &EventRef) -> Option<Rc<RefCell<KeyboardEvent>>>;
    fn MouseButton(&self, event: &EventRef) -> Option<i16>;
    fn HasDOMWindow(&self) -> bool;
    fn OnClickEvent(&self, event: &EventRef);
    fn HrefAttribute(&self) -> String;
    fn CompleteURL(&self, url: &str) -> String;
    fn PingAttribute(&self) -> Option<String>;
    fn SendPings(
        &self,
        invocation: Option<&dyn ScriptInvocation>,
        completed_url: &str,
        ping_attribute: Option<&str>,
    );
    fn CreateResourceRequest(&self, completed_url: &str) -> Rc<dyn AnchorResourceRequest>;
    fn HandleReferrerPolicyAttribute(&self, request: &Rc<dyn AnchorResourceRequest>);
    fn EventNavigationModifiers(&self, event: &EventRef) -> Option<NavigationEventModifiers>;
    /// CurrentInputEvent projection, admitting MouseUp, Enter keyboard and
    /// gesture events at navigation_policy.cc:95-134, rather than JS trust.
    fn CurrentInputNavigationModifiers(&self) -> Option<NavigationEventModifiers>;
    fn IsMac(&self) -> bool;
    fn NewTabModifierSetFromIsolatedWorld(&self) -> bool;
    fn DownloadAttribute(&self) -> Option<String>;
    fn CanReadContent(&self, completed_url: &str) -> bool;
    fn HandleDownloadAttribute(
        &self,
        invocation: Option<&dyn ScriptInvocation>,
        download: &str,
        completed_url: &str,
        is_trusted: bool,
        request: Rc<dyn AnchorResourceRequest>,
    );
    /// Uses ui::kDoubleClickTimeMs and the current source task runner. The
    /// cancellable handle is then installed on the same frame EventHandler.
    fn PostDelayedNavigationTask(
        &self,
        task: Box<dyn FnOnce()>,
    ) -> Rc<dyn AnchorNavigationTaskHandle>;
    fn SetDelayedNavigationTaskHandle(&self, handle: Rc<dyn AnchorNavigationTaskHandle>);
    /// None is the source missing DOMWindow/Frame guard. The resulting request
    /// has RequestContextType::HYPERLINK and belongs to the existing loader.
    fn CreateHyperlinkFrameLoadRequest(
        &self,
        invocation: Option<&dyn ScriptInvocation>,
        request: Rc<dyn AnchorResourceRequest>,
    ) -> Option<Rc<dyn AnchorFrameLoadRequest>>;
    fn EffectiveTarget(&self) -> String;
    fn LinkRelations(&self) -> u32;
    fn FindOrCreateFrameForNavigation(
        &self,
        request: &Rc<dyn AnchorFrameLoadRequest>,
        target: &str,
    ) -> AnchorNavigationLookup;
    fn HrefTranslateEnabled(&self) -> bool;
    fn HrefTranslateAttribute(&self) -> Option<String>;
}

pub struct HTMLAnchorElementBase {
    environment: Rc<dyn AnchorElementEnvironment>,
}
impl HTMLAnchorElementBase {
    pub fn new(environment: Rc<dyn AnchorElementEnvironment>) -> Rc<Self> {
        Rc::new(Self { environment })
    }
    pub fn IsLiveLink(&self) -> bool {
        self.environment.IsLink() && !self.environment.IsEditable()
    }
    pub fn HasActivationBehavior(&self) -> bool {
        self.environment.IsLink()
    }
    pub fn SetActive(&self, active: bool) {
        if active && self.environment.IsEditable() {
            return;
        }
        self.environment.SetBaseActive(active);
    }
    pub fn DefaultEventHandler(
        self: &Rc<Self>,
        event: &EventRef,
        invocation: Option<&dyn ScriptInvocation>,
        actions: &dyn AnchorDefaultActions,
    ) {
        if self.environment.IsLink() {
            let keyboard = self.environment.KeyboardEvent(event);
            let enter = keyboard
                .as_ref()
                .is_some_and(|event| KeyboardEvent::IsEnterKeyKeydownEvent(Some(&event.borrow())));
            if self.environment.IsFocused() && enter && self.IsLiveLink() {
                event.borrow_mut().BaseEventMut().SetDefaultHandled();
                actions.DispatchSimulatedClick(event);
                return;
            }
            let is_click = {
                let event = event.borrow();
                matches!(event.BaseEvent().r#type(), "click" | "auxclick") && event.IsMouseEvent()
            };
            if is_click
                && matches!(self.environment.MouseButton(event), Some(0 | 1))
                && self.IsLiveLink()
            {
                self.HandleClick(event, invocation, actions);
                return;
            }
        }
        actions.BaseDefaultEventHandler(event);
    }
    pub fn HandleClick(
        self: &Rc<Self>,
        event: &EventRef,
        invocation: Option<&dyn ScriptInvocation>,
        actions: &dyn AnchorDefaultActions,
    ) {
        event.borrow_mut().BaseEventMut().SetDefaultHandled();
        if !self.environment.HasDOMWindow() {
            return;
        }
        if !self.environment.IsConnected() && self.environment.IsAreaElement() {
            return;
        }
        self.environment.OnClickEvent(event);
        let mut url = self
            .environment
            .HrefAttribute()
            .trim_matches(|c| matches!(c, '\t' | '\n' | '\u{000c}' | '\r' | ' '))
            .to_owned();
        if let Some((x, y, zoom)) = actions.ServerMapPoint(event) {
            // gfx::ToRoundedPoint and SetToMax(Point()) at source cc:165-168.
            let scale_factor = 1.0_f32 / zoom;
            url.push_str(&format!(
                "?{},{}",
                (x * scale_factor).round().max(0.0) as i32,
                (y * scale_factor).round().max(0.0) as i32
            ));
        }
        let completed_url = self.environment.CompleteURL(&url);
        self.environment.SendPings(
            invocation,
            &completed_url,
            self.environment.PingAttribute().as_deref(),
        );
        let request = self.environment.CreateResourceRequest(&completed_url);
        self.environment.HandleReferrerPolicyAttribute(&request);
        let policy = NavigationPolicyFromEvent(
            self.environment.EventNavigationModifiers(event),
            self.environment.CurrentInputNavigationModifiers(),
            self.environment.IsMac(),
            self.environment.NewTabModifierSetFromIsolatedWorld(),
        );
        let (is_trusted, time) = {
            let event = event.borrow();
            (
                event.BaseEvent().isTrusted(),
                event.BaseEvent().PlatformTimeStamp(),
            )
        };
        if let Some(download) = self.environment.DownloadAttribute() {
            if policy != NavigationPolicy::kNavigationPolicyDownload
                && self.environment.CanReadContent(&completed_url)
            {
                self.environment.HandleDownloadAttribute(
                    invocation,
                    &download,
                    &completed_url,
                    is_trusted,
                    request,
                );
                return;
            }
        }
        if policy == NavigationPolicy::kNavigationPolicyDownload {
            let weak = Rc::downgrade(self);
            let task = self
                .environment
                .PostDelayedNavigationTask(Box::new(move || {
                    if let Some(owner) = weak.upgrade() {
                        owner.NavigateToHyperlink(
                            None,
                            request,
                            policy,
                            is_trusted,
                            time,
                            completed_url,
                        );
                    }
                }));
            self.environment.SetDelayedNavigationTaskHandle(task);
        } else {
            self.NavigateToHyperlink(invocation, request, policy, is_trusted, time, completed_url);
        }
    }
    pub fn NavigateToHyperlink(
        self: &Rc<Self>,
        invocation: Option<&dyn ScriptInvocation>,
        request: Rc<dyn AnchorResourceRequest>,
        policy: NavigationPolicy,
        is_trusted: bool,
        time: TimeTicks,
        completed_url: String,
    ) {
        let Some(request) = self
            .environment
            .CreateHyperlinkFrameLoadRequest(invocation, request)
        else {
            return;
        };
        request.SetNavigationPolicy(policy);
        request.SetClientNavigationReasonAnchorClick();
        request.SetSourceElement(&self.environment.Target());
        let target = request.CleanNavigationTarget(&self.environment.EffectiveTarget());
        request.HandleRelAttribute(&target, self.environment.LinkRelations());
        request.EnforceBlobUrlNoopenerIfNeeded(&completed_url);
        request.SetTriggeringEventInfo(is_trusted);
        request.SetInputStartTime(time);
        match self
            .environment
            .FindOrCreateFrameForNavigation(&request, &target)
        {
            AnchorNavigationLookup::Ready(frame) => {
                self.FinishHyperlinkNavigation(request, Some(frame), invocation)
            }
            AnchorNavigationLookup::Denied => {
                self.FinishHyperlinkNavigation(request, None, invocation)
            }
            AnchorNavigationLookup::Pending(pending) => {
                let owner = Rc::downgrade(self);
                pending.Resume(Box::new(move |frame| {
                    if let Some(owner) = owner.upgrade() {
                        owner.FinishHyperlinkNavigation(request, frame, None);
                    }
                }));
            }
        }
    }
    fn FinishHyperlinkNavigation(
        &self,
        request: Rc<dyn AnchorFrameLoadRequest>,
        frame: Option<Rc<dyn AnchorNavigationTarget>>,
        invocation: Option<&dyn ScriptInvocation>,
    ) {
        if self.environment.HrefTranslateEnabled() {
            if let Some(value) = self.environment.HrefTranslateAttribute() {
                if frame.as_ref().is_some_and(|frame| frame.IsCurrentFrame()) || request.Noopener()
                {
                    request.SetHrefTranslate(&value);
                }
            }
        }
        if let Some(frame) = frame {
            frame.NavigateStandardWithInvocation(request, invocation);
        }
    }
}
