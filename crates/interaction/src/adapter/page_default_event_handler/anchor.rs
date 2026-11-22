//! Production HTMLAnchorElementBase/HTMLAreaElement default-action projection.
//! Chromium: html_anchor_element.cc:137-206,366-430,441-531; node.cc:3407-3426.
//! Live DOM facts, canonical EventRef, resident Layout and retained loader host
//! reach the translated source directly. This adapter makes no navigation policy.
use super::*;
use crate::adapter::event::{EventProjection, EventTargetResolver};
use interaction_default::core::{
    html::{
        anchor_element_utils::{ParseRelAttribute, ReferrerPolicyForAttribute},
        html_anchor_element::*,
    },
    loader::{navigation_policy::NavigationEventModifiers, resource_request::ResourceRequest},
};
use interaction_dispatch::{
    bindings::script_invocation::ScriptInvocation,
    core::{
        dom::events::event::{EventRef, EventTargetRef},
        events::keyboard_event::KeyboardEvent,
    },
};
use std::{cell::RefCell, rc::Rc};

pub(super) fn IsLink(context: &InputTypeContext<'_>, node: u64) -> bool {
    ReadNode(&context.document, node, |_, node, _| {
        (node.IsHTMLElement("a") || node.IsHTMLElement("area"))
            && node.FindAttribute("href").is_some()
    })
    .unwrap_or(false)
}

pub(super) fn HandleDefaultEvent(
    context: &InputTypeContext<'_>,
    node: u64,
    event: &mut Event,
) -> bool {
    let owner = ReadNode(&context.document, node, |document, element, index| {
        (element.IsHTMLElement("a") || element.IsHTMLElement("area"))
            .then(|| document.NodeHandle(index))
    })
    .flatten();
    let Some(owner) = owner else {
        return false;
    };
    // HTMLElement/Node retain the fallback for non-links and ordinary keys.
    if !IsLink(context, node) {
        return false;
    }
    let Some(host) = context.anchor_navigation.clone() else {
        panic!("anchor default action requires its LocalFrame loader host");
    };
    let anchor = context.default_action_state.Anchor(owner, || {
        HTMLAnchorElementBase::new(Rc::new(AnchorEnvironment {
            document: context.document.clone(),
            node,
            targets: context.event_targets.clone(),
            state: context.interaction_state.clone(),
            host,
        }))
    });
    let native = event
        .canonical
        .clone()
        .expect("anchor defaults retain the canonical dispatch Event");
    anchor.DefaultEventHandler(
        &native,
        context.script_invocation,
        &DefaultActions {
            context,
            node,
            event,
        },
    );
    EventProjection::FromCanonical(native, context.event_targets.clone()).SyncTo(event);
    true
}

struct DefaultActions<'a, 'scope> {
    context: &'a InputTypeContext<'scope>,
    node: u64,
    event: &'a Event,
}
impl AnchorDefaultActions for DefaultActions<'_, '_> {
    fn DispatchSimulatedClick(&self, _: &EventRef) {
        (self.context.dispatch_simulated_click)(self.node, self.event);
    }
    fn BaseDefaultEventHandler(&self, event: &EventRef) {
        // Node::DefaultEventHandler only runs the click default for RawTarget.
        if self.event.r#type == EventType::kClick && self.event.target_node_id == self.node {
            if Node::HandleClickDefaultAction(|| {
                (self.context.dispatch_dom_activate)(self.node, self.event)
            }) {
                event.borrow_mut().BaseEventMut().SetDefaultHandled();
            }
        }
    }
    fn ServerMapPoint(&self, event: &EventRef) -> Option<(f32, f32, f32)> {
        let (target, point) = {
            let event = event.borrow();
            let mouse = event.AsMouseEvent()?;
            (
                self.context
                    .event_targets
                    .NodeId(&event.BaseEvent().RawTarget()?)?,
                *mouse.AbsoluteLocation(),
            )
        };
        if !ReadNode(&self.context.document, target, |_, node, _| {
            node.IsHTMLElement("img") && node.FindAttribute("ismap").is_some()
        })
        .unwrap_or(false)
        {
            return None;
        }
        self.context
            .anchor_navigation
            .as_ref()?
            .ServerMapContentPoint(target, &point)
    }
}

struct AnchorEnvironment {
    document: crate::ownership::InteractionDocument,
    node: u64,
    targets: Rc<crate::adapter::dom_event::DomEventTargets>,
    state: Rc<RefCell<dom::UserInteractionState>>,
    host: Rc<dyn AnchorNavigationHost>,
}
impl AnchorEnvironment {
    fn Attribute(&self, name: &str) -> Option<String> {
        ReadNode(&self.document, self.node, |_, node, _| {
            node.FindAttribute(name)
                .map(|attribute| attribute.value.clone())
        })
        .flatten()
    }
}
impl AnchorElementEnvironment for AnchorEnvironment {
    fn Target(&self) -> EventTargetRef {
        self.targets
            .ResolveTarget(self.node)
            .expect("resident anchor target")
    }
    fn IsLink(&self) -> bool {
        self.Attribute("href").is_some()
    }
    fn IsFocused(&self) -> bool {
        self.state.borrow().focused_node_id == Some(self.node)
    }
    fn IsEditable(&self) -> bool {
        ReadNode(&self.document, self.node, |document, _, index| {
            document.IsEditableForEditing(index)
        })
        .unwrap_or(false)
    }
    fn IsConnected(&self) -> bool {
        ReadNode(&self.document, self.node, |document, _, index| {
            document.IsConnectedToDocument(index)
        })
        .unwrap_or(false)
    }
    fn IsAreaElement(&self) -> bool {
        ReadNode(&self.document, self.node, |_, node, _| {
            node.IsHTMLElement("area")
        })
        .unwrap_or(false)
    }
    fn SetBaseActive(&self, active: bool) {
        let mut document = self.document.borrow_mut();
        let document = document.GetDocumentMut();
        if let Some(index) = document.FindNodeById(self.node) {
            document.SetElementActive(index, active);
        }
    }
    fn KeyboardEvent(&self, event: &EventRef) -> Option<Rc<RefCell<KeyboardEvent>>> {
        event.Typed()
    }
    fn MouseButton(&self, event: &EventRef) -> Option<i16> {
        event.borrow().AsMouseEvent().map(|mouse| mouse.button())
    }
    fn HasDOMWindow(&self) -> bool {
        self.targets.WindowDocument().IsAttached()
    }
    fn OnClickEvent(&self, _: &EventRef) { /* AnchorElementInteractionTracker metrics only. */
    }
    fn HrefAttribute(&self) -> String {
        self.Attribute("href").unwrap_or_default()
    }
    fn CompleteURL(&self, url: &str) -> String {
        self.document
            .borrow()
            .GetDocument()
            .BaseURL()
            .CompleteURL(url)
            .map(|url| url.into())
            .unwrap_or_default()
    }
    fn PingAttribute(&self) -> Option<String> {
        self.Attribute("ping")
    }
    fn SendPings(&self, invocation: Option<&dyn ScriptInvocation>, url: &str, ping: Option<&str>) {
        self.host.SendPings(invocation, url, ping);
    }
    fn CreateResourceRequest(&self, url: &str) -> Rc<dyn AnchorResourceRequest> {
        ResourceRequest::new(url)
    }
    fn HandleReferrerPolicyAttribute(&self, request: &Rc<dyn AnchorResourceRequest>) {
        let relations = self.LinkRelations();
        if let Some(policy) = self
            .Attribute("referrerpolicy")
            .and_then(|value| ReferrerPolicyForAttribute(&value, relations))
        {
            request
                .AsAny()
                .downcast_ref::<ResourceRequest>()
                .expect("source anchor ResourceRequest")
                .Mutate(|request| request.referrer_policy = policy);
        }
    }
    fn EventNavigationModifiers(&self, event: &EventRef) -> Option<NavigationEventModifiers> {
        let event = event.borrow();
        let keys = event.AsUIEventWithKeyState()?;
        let button = event.AsMouseEvent().map_or(0, |mouse| mouse.button());
        Some(NavigationEventModifiers {
            button,
            ctrl: keys.ctrlKey(),
            shift: keys.shiftKey(),
            alt: keys.altKey(),
            meta: keys.metaKey(),
        })
    }
    fn CurrentInputNavigationModifiers(&self) -> Option<NavigationEventModifiers> {
        self.host.CurrentInputNavigationModifiers()
    }
    fn IsMac(&self) -> bool {
        cfg!(target_os = "macos")
    }
    fn NewTabModifierSetFromIsolatedWorld(&self) -> bool {
        self.host.NewTabModifierSetFromIsolatedWorld()
    }
    fn DownloadAttribute(&self) -> Option<String> {
        self.Attribute("download")
    }
    fn CanReadContent(&self, url: &str) -> bool {
        self.host.CanReadContent(url)
    }
    fn HandleDownloadAttribute(
        &self,
        invocation: Option<&dyn ScriptInvocation>,
        download: &str,
        url: &str,
        trusted: bool,
        request: Rc<dyn AnchorResourceRequest>,
    ) {
        self.host.HandleDownloadAttribute(
            invocation,
            self.Target(),
            download,
            url,
            trusted,
            request,
        );
    }
    fn PostDelayedNavigationTask(
        &self,
        task: Box<dyn FnOnce()>,
    ) -> Rc<dyn AnchorNavigationTaskHandle> {
        self.host.PostDelayedNavigationTask(task)
    }
    fn SetDelayedNavigationTaskHandle(&self, handle: Rc<dyn AnchorNavigationTaskHandle>) {
        self.host.SetDelayedNavigationTaskHandle(handle);
    }
    fn CreateHyperlinkFrameLoadRequest(
        &self,
        invocation: Option<&dyn ScriptInvocation>,
        request: Rc<dyn AnchorResourceRequest>,
    ) -> Option<Rc<dyn AnchorFrameLoadRequest>> {
        self.host
            .CreateHyperlinkFrameLoadRequest(invocation, request)
    }
    fn EffectiveTarget(&self) -> String {
        self.Attribute("target").unwrap_or_else(|| {
            self.document
                .borrow()
                .GetDocument()
                .HTMLDocumentBaseTarget()
        })
    }
    fn LinkRelations(&self) -> u32 {
        ParseRelAttribute(&self.Attribute("rel").unwrap_or_default())
    }
    fn FindOrCreateFrameForNavigation(
        &self,
        request: &Rc<dyn AnchorFrameLoadRequest>,
        target: &str,
    ) -> AnchorNavigationLookup {
        self.host.FindOrCreateFrameForNavigation(request, target)
    }
    fn HrefTranslateEnabled(&self) -> bool {
        self.host.HrefTranslateEnabled()
    }
    fn HrefTranslateAttribute(&self) -> Option<String> {
        self.Attribute("hreftranslate")
    }
}
