//! LocalFrame/View leaves for the synchronous source EventHandler binding.
//! Chromium: local_frame_view.cc:735-741,4594-4673; event_handler.cc:412-442.
//! FrameTree and Layout retain ownership; these handles contain identity only.
use super::local_frame::{LocalFrameDomain, LocalFrameView};
use super::*;
use foundation::{gfx, LayoutUnit, PhysicalRect, PhysicalSize};
use interaction::adapter::event::EventTargetResolver;
use interaction::adapter::page_event_handler_frame::{
    PageEventHandlerFrameDomain, PageEventHandlerFrameInputClient, PageEventHandlerSubframe,
};
use interaction::adapter::page_event_handler_hit_test::{
    PageEventHandlerCoordinates, PageEventHandlerHitTestLifecycle,
};
use interaction::dispatch::core::dom::events::event::EventTargetRef;
use interaction::dispatch::core::input::event_handling_util::{
    self, FrameDiscardInput, FrameStability,
};
use interaction::hit_test::core::layout::hit_test_location::HitTestLocation;
use interaction::input::{
    base::time::time::TimeTicks,
    public::common::input::{
        web_gesture_event::WebGestureEvent, web_input_event::WebInputEvent,
        web_mouse_event::WebMouseEvent,
    },
};

pub(super) struct NativeEventHandlerFrame {
    pub domain: Weak<LocalFrameDomain>,
    pub token: u64,
}
impl NativeEventHandlerFrame {
    pub(super) fn new(domain: &Rc<LocalFrameDomain>, token: u64) -> Rc<Self> {
        assert!(token == domain.root || domain.Frame(token).is_some());
        Rc::new(Self {
            domain: Rc::downgrade(domain),
            token,
        })
    }
    fn Domain(&self) -> Rc<LocalFrameDomain> {
        self.domain
            .upgrade()
            .expect("live EventHandler Frame domain")
    }
    pub(super) fn View(&self) -> Option<Rc<LocalFrameView>> {
        let domain = self.domain.upgrade()?;
        if self.token == domain.root {
            domain.page.upgrade()?;
            Some(domain.root_view.clone())
        } else {
            let frame = domain.Frame(self.token)?;
            let view = frame.runtime.borrow().as_ref()?.view.clone();
            Some(view)
        }
    }
    pub(super) fn Identity(&self) -> Option<Rc<()>> {
        Some(
            self.domain
                .upgrade()?
                .FocusInteraction(self.token)?
                .SourceInputOwners()
                .pointer_frame
                .identity
                .clone(),
        )
    }
    pub(super) fn IsAttached(&self) -> bool {
        let Some(domain) = self.domain.upgrade() else {
            return false;
        };
        if self.token == domain.root {
            domain
                .page
                .upgrade()
                .is_some_and(|page| page.document.borrow().GetDocument().IsActive())
        } else {
            domain.Frame(self.token).is_some_and(|frame| {
                frame.state.Lifecycle().GetState()
                    == dom::frame_lifecycle::FrameLifecycleState::Attached
            })
        }
    }
    pub(super) fn IsDescendantOf(&self, identity: &Rc<()>) -> bool {
        let domain = self.Domain();
        let mut token = self.token;
        while let Some(frame) = domain.Frame(token) {
            token = frame.parent;
            if domain.FocusInteraction(token).is_some_and(|owner| {
                Rc::ptr_eq(identity, &owner.SourceInputOwners().pointer_frame.identity)
            }) {
                return true;
            }
        }
        false
    }
    pub(super) fn ParentLocalFrameIdentity(&self) -> Option<Rc<()>> {
        let domain = self.domain.upgrade()?;
        let parent = domain.Frame(self.token)?.parent;
        Some(
            domain
                .FocusInteraction(parent)?
                .SourceInputOwners()
                .pointer_frame
                .identity
                .clone(),
        )
    }
    pub(super) fn FocusedCoreFrameIdentity(&self) -> Option<Rc<()>> {
        self.FocusFrameIdentity(true)
    }
    pub(super) fn FocusedLocalFrameInWidgetIdentity(&self) -> Option<Rc<()>> {
        self.FocusFrameIdentity(false)
    }
    fn FocusFrameIdentity(&self, main_frame_fallback: bool) -> Option<Rc<()>> {
        let domain = self.domain.upgrade()?;
        let controller = domain.focus_controller.borrow().clone();
        let frame = match controller {
            Some(controller) if main_frame_fallback => controller.FocusedOrMainFrame()?,
            Some(controller) => controller.FocusedFrame()?,
            None if main_frame_fallback => domain.FocusFrame(domain.root)?,
            None => return None,
        };
        if !frame.IsLocalFrame()
            || !Rc::ptr_eq(&frame.LocalFrameRoot(), &domain.FocusFrame(domain.root)?)
        {
            return None;
        }
        let token = domain
            .focus_frames
            .borrow()
            .iter()
            .find_map(|(token, owner)| {
                let owner: interaction::default::core::page::focus_controller::FocusFrameRef =
                    owner.clone();
                Rc::ptr_eq(&owner, &frame).then_some(*token)
            })?;
        Some(
            domain
                .FocusInteraction(token)?
                .SourceInputOwners()
                .pointer_frame
                .identity
                .clone(),
        )
    }
    pub(super) fn OwnerElement(&self) -> Option<EventTargetRef> {
        let domain = self.domain.upgrade()?;
        let frame = domain.Frame(self.token)?;
        let document = domain.Document(frame.parent)?.Handle();
        domain
            .FocusInteraction(frame.parent)?
            .DOMEventTargets(&document)
            .ResolveTarget(frame.state.Owner()?.Id())
    }
    pub(super) fn NodeFrameIdentity(&self, target: &EventTargetRef) -> Option<Rc<()>> {
        let domain = self.domain.upgrade()?;
        let tokens = std::iter::once(domain.root)
            .chain(domain.frames.borrow().keys().copied().collect::<Vec<_>>())
            .collect::<Vec<_>>();
        for token in tokens {
            let Some(document) = domain.Document(token) else {
                continue;
            };
            let Some(engine) = domain.FocusInteraction(token) else {
                continue;
            };
            if engine
                .DOMEventTargets(&document.Handle())
                .NodeId(target)
                .is_some()
            {
                return Some(engine.SourceInputOwners().pointer_frame.identity.clone());
            }
        }
        None
    }
    pub(super) fn HitNodeFrameIdentity(
        &self,
        node: &interaction::hit_test::core::layout::hit_test_result::NodeRef,
    ) -> Option<Rc<()>> {
        let id = node.DOMNodeId()?;
        let identity = node.DOMIdentity();
        let domain = self.domain.upgrade()?;
        let tokens = std::iter::once(domain.root)
            .chain(domain.frames.borrow().keys().copied().collect::<Vec<_>>())
            .collect::<Vec<_>>();
        for token in tokens {
            let Some(document) = domain.Document(token) else {
                continue;
            };
            let matches = {
                let document = document.borrow();
                let document = document.GetDocument();
                document
                    .FindNodeById(id)
                    .is_some_and(|index| document.NodeHandle(index).Identity() == identity)
            };
            if matches {
                return Some(
                    domain
                        .FocusInteraction(token)?
                        .SourceInputOwners()
                        .pointer_frame
                        .identity
                        .clone(),
                );
            }
        }
        None
    }
    pub(super) fn AdjustmentScales(&self) -> (f32, f32, f32) {
        let page = self.Domain().page.upgrade().expect("attached frame scales");
        let device = page.constraints.borrow().device_pixel_ratio as f32;
        let page_scale = page.web_view_zoom.borrow().Scale();
        (device, 1.0, page_scale)
    }
    pub(super) fn NativeFrameSource(
        &self,
        identity: &Rc<()>,
    ) -> Option<Rc<interaction::adapter::page_event_handler_binding::PageEventHandlerSource>> {
        let domain = self.domain.upgrade()?;
        domain
            .FocusInteraction(domain.EventHandlerFrameToken(identity)?)?
            .NativeFrameSource()
    }
    pub(super) fn PointToRoot(&self, point: gfx::PointF) -> gfx::PointF {
        let domain = self.Domain();
        if self.token == domain.root {
            return point;
        }
        domain.MapChildPoint(
            &domain.Frame(self.token).expect("attached child Frame"),
            point,
        )
    }
    pub(super) fn PointFromRoot(&self, point: gfx::PointF) -> gfx::PointF {
        self.Domain()
            .RootPointToFrameFloat(self.token, point)
            .expect("resident FrameView coordinate mapping")
    }
    fn ShouldDiscard(&self, event: &WebInputEvent) -> bool {
        let domain = self.Domain();
        let document = domain
            .Document(self.token)
            .expect("attached Frame Document");
        let root = domain
            .Document(domain.root)
            .expect("attached local root Document");
        let cross_origin = !document
            .borrow()
            .GetDocument()
            .SecurityContext()
            .Origin()
            .CanAccess(
                root.borrow()
                    .GetDocument()
                    .SecurityContext()
                    .Origin()
                    .as_ref(),
            );
        event_handling_util::ShouldDiscardEventTargetingFrame(
            event,
            &FrameDiscardInput {
                is_cross_origin_to_outermost_main_frame: cross_origin,
                needs_occlusion_tracking: self
                    .View()
                    .is_some_and(|view| view.needs_occlusion_tracking.get()),
                view: self,
            },
            false,
        ) // features::kDiscardInputEventsToRecentlyMovedFrames defaults off.
    }
    fn MapLocation(
        &self,
        location: &HitTestLocation,
        map: impl Fn(gfx::PointF) -> gfx::PointF,
    ) -> HitTestLocation {
        let point = map(*location.TransformedPoint());
        if location.IsRectBasedTest() {
            let rect = location.TransformedRect();
            HitTestLocation::FromPointAndQuad(
                point,
                gfx::QuadF::new(
                    map(*rect.p1()),
                    map(*rect.p2()),
                    map(*rect.p3()),
                    map(*rect.p4()),
                ),
            )
        } else {
            HitTestLocation::FromPointF(point)
        }
    }
}
impl FrameStability for NativeEventHandlerFrame {
    fn RectInParentIsStableForIOv2(&self, timestamp: TimeTicks) -> bool {
        let Some(view) = self.View() else {
            return false;
        };
        if timestamp
            .ToInternalValue()
            .saturating_sub(view.rect_in_parent_stable_since_for_iov2.get())
            < 500_000
        {
            return false;
        }
        let domain = self.Domain();
        domain.Frame(self.token).is_none_or(|frame| {
            NativeEventHandlerFrame {
                domain: self.domain.clone(),
                token: frame.parent,
            }
            .RectInParentIsStableForIOv2(timestamp)
        })
    }
    fn RectInParentIsStable(&self, timestamp: TimeTicks) -> bool {
        let Some(view) = self.View() else {
            return false;
        };
        if timestamp.ToInternalValue() < view.rect_in_parent_stable_since.get() {
            return false;
        }
        let domain = self.Domain();
        domain.Frame(self.token).is_none_or(|frame| {
            NativeEventHandlerFrame {
                domain: self.domain.clone(),
                token: frame.parent,
            }
            .RectInParentIsStable(timestamp)
        })
    }
}
impl PageEventHandlerFrameInputClient for NativeEventHandlerFrame {
    fn HasClient(&self) -> bool {
        self.IsAttached()
            && self
                .domain
                .upgrade()
                .is_some_and(|domain| domain.page.upgrade().is_some())
    }
    fn SetMouseCapture(&self, capture: bool) {
        self.Domain()
            .page
            .upgrade()
            .expect("attached capture client")
            .client
            .borrow_mut()
            .CaptureMouseEventsToWidget(capture);
    }
    fn ShouldDiscardEventTargetingFrame(&self, event: &WebMouseEvent) -> bool {
        self.ShouldDiscard(event)
    }
    fn ShouldDiscardGestureEventTargetingFrame(&self, event: &WebGestureEvent) -> bool {
        self.ShouldDiscard(event)
    }
    fn NotifyUserActivation(&self) {
        use interaction::input::public::common::frame::user_activation_state::UserActivationNotificationType;
        let domain = self.Domain();
        let now = domain
            .page
            .upgrade()
            .expect("attached activation client")
            .InteractionEventClock()
            .Now();
        let activate = |token| {
            if let Some(engine) = domain.FocusInteraction(token) {
                engine
                    .SourceInputOwners()
                    .pointer_frame
                    .activation
                    .borrow_mut()
                    .Activate(UserActivationNotificationType::kInteraction, now);
            }
        };
        // Frame::NotifyUserActivationInFrameTree, frame.cc:306-344: first
        // ancestors, then all same-origin local frames in the same Page.
        let mut token = self.token;
        loop {
            activate(token);
            let Some(frame) = domain.Frame(token) else {
                break;
            };
            token = frame.parent;
        }
        let document = domain.Document(self.token).expect("activation Document");
        let origin = document
            .borrow()
            .GetDocument()
            .SecurityContext()
            .Origin()
            .clone();
        let tokens = std::iter::once(domain.root)
            .chain(domain.frames.borrow().keys().copied().collect::<Vec<_>>())
            .collect::<Vec<_>>();
        for token in tokens {
            if domain.Document(token).is_some_and(|document| {
                origin.CanAccess(
                    document
                        .borrow()
                        .GetDocument()
                        .SecurityContext()
                        .Origin()
                        .as_ref(),
                )
            }) {
                activate(token);
            }
        }
    }
    fn SetCaretBlinkingSuspended(&self, suspended: bool) {
        let domain = self.Domain();
        if self.token == domain.root {
            domain
                .page
                .upgrade()
                .expect("attached caret owner")
                .layout_editing
                .caret
                .borrow_mut()
                .SetCaretBlinkingSuspended(suspended);
        } else {
            domain
                .Frame(self.token)
                .expect("attached child")
                .runtime
                .borrow()
                .as_ref()
                .expect("attached child Document")
                .editing
                .caret
                .borrow_mut()
                .SetCaretBlinkingSuspended(suspended);
        }
    }
}
impl PageEventHandlerHitTestLifecycle for NativeEventHandlerFrame {
    fn HasContentLayoutObject(&self) -> bool {
        self.domain
            .upgrade()
            .and_then(|domain| {
                domain.WithLayout(self.token, |layout| layout.GetLayoutTree().is_some())
            })
            .unwrap_or(false)
    }
    fn HasView(&self) -> bool {
        self.IsAttached() && self.View().is_some()
    }
    fn DidFirstLayout(&self) -> bool {
        self.View().is_some_and(|view| view.DidFirstLayout())
    }
    fn LifecycleUpdatesActive(&self) -> bool {
        self.View()
            .is_some_and(|view| view.LifecycleUpdatesActive())
    }
    fn FrameViewSize(&self) -> PhysicalSize {
        let domain = self.Domain();
        if self.token == domain.root {
            let page = domain.page.upgrade().expect("live local root");
            let constraints = page.constraints.borrow();
            PhysicalSize::new(
                LayoutUnit::from_f64(constraints.available_size.width),
                LayoutUnit::from_f64(constraints.available_size.height),
            )
        } else {
            let size = self.View().expect("attached child view").size.get();
            PhysicalSize::new(
                LayoutUnit::from_f64(size.width() as f64),
                LayoutUnit::from_f64(size.height() as f64),
            )
        }
    }
    fn UnboundedElementEnabled(&self) -> bool {
        foundation::RuntimeEnabledFeatures::UnboundedElementEnabled()
    }
    fn HasActiveUnboundedElements(&self) -> bool {
        assert!(
            !self.UnboundedElementEnabled(),
            "unbounded FrameView owner is required when enabled"
        );
        false
    }
}
impl PageEventHandlerCoordinates for NativeEventHandlerFrame {
    fn ConvertLocationToRoot(
        &self,
        location: &HitTestLocation,
        visual_overflow: bool,
    ) -> HitTestLocation {
        if location.IsRectBasedTest() && !visual_overflow {
            let rect = location.BoundingBox();
            let point = self.PointToRoot(gfx::PointF::new(
                rect.offset.left.ToFloat(),
                rect.offset.top.ToFloat(),
            ));
            return HitTestLocation::FromPhysicalRect(PhysicalRect::new(
                foundation::PhysicalOffset::FromPointFFloor(&point),
                rect.size,
            ));
        }
        if location.IsRectBasedTest() {
            let domain = self.Domain();
            let rect = gfx::RectF::from(*location.BoundingBox());
            let rect = if self.token == domain.root {
                rect
            } else {
                domain.MapChildRect(
                    &domain.Frame(self.token).expect("attached child Frame"),
                    rect,
                )
            };
            return HitTestLocation::FromPhysicalRect(PhysicalRect::EnclosingRect(&rect));
        }
        self.MapLocation(location, |point| self.PointToRoot(point))
    }
    fn ConvertLocationFromRoot(&self, location: &HitTestLocation) -> HitTestLocation {
        self.MapLocation(location, |point| self.PointFromRoot(point))
    }
    fn ViewportToFrame(&self, location: &HitTestLocation) -> HitTestLocation {
        let domain = self.Domain();
        let page = domain.page.upgrade().expect("live local root");
        self.MapLocation(location, |point| {
            self.PointFromRoot(page.web_view_zoom.borrow().ViewportToRootFrame(point))
        })
    }
    fn ConvertPointFromRootFrame(&self, point: gfx::PointF) -> gfx::PointF {
        self.PointFromRoot(point)
    }
}

impl LocalFrameDomain {
    pub(super) fn BindChildEventHandler(self: &Rc<Self>, token: u64) {
        assert_ne!(token, self.root);
        let Some(engine) = self.FocusInteraction(token) else {
            return;
        };
        engine.SetAnchorNavigationHost(self.AnchorNavigationHost(token).map(|host| {
            host as Rc<
                dyn interaction::default::core::html::html_anchor_element::AnchorNavigationHost,
            >
        }));
        if engine.NativeFrameSource().is_some() {
            return;
        }
        let document = self
            .Document(token)
            .expect("current child input Document")
            .Handle();
        engine.SetEventHandlerEmbedding(
            &document,
            super::native_event_handler_host::NativeEventHandlerEmbedding::new(self, token),
        );
    }
    pub(super) fn EventHandlerFrameToken(&self, identity: &Rc<()>) -> Option<u64> {
        if self.FocusInteraction(self.root).is_some_and(|engine| {
            Rc::ptr_eq(identity, &engine.SourceInputOwners().pointer_frame.identity)
        }) {
            return Some(self.root);
        }
        self.frames.borrow().keys().copied().find(|token| {
            self.FocusInteraction(*token).is_some_and(|engine| {
                Rc::ptr_eq(identity, &engine.SourceInputOwners().pointer_frame.identity)
            })
        })
    }
    pub(super) fn EventHandlerFrame(
        self: &Rc<Self>,
        identity: &Rc<()>,
    ) -> Option<Rc<NativeEventHandlerFrame>> {
        Some(NativeEventHandlerFrame::new(
            self,
            self.EventHandlerFrameToken(identity)?,
        ))
    }
}
impl PageEventHandlerFrameDomain for NativeEventHandlerFrame {
    fn FocusedCoreFrameIdentity(&self) -> Option<Rc<()>> {
        NativeEventHandlerFrame::FocusedCoreFrameIdentity(self)
    }
    fn FocusedLocalFrameInWidgetIdentity(&self) -> Option<Rc<()>> {
        NativeEventHandlerFrame::FocusedLocalFrameInWidgetIdentity(self)
    }
    fn ParentLocalFrameIdentity(&self) -> Option<Rc<()>> {
        NativeEventHandlerFrame::ParentLocalFrameIdentity(self)
    }
    fn OwnerElement(&self) -> Option<EventTargetRef> {
        NativeEventHandlerFrame::OwnerElement(self)
    }
    fn NodeFrameIdentity(&self, target: &EventTargetRef) -> Option<Rc<()>> {
        NativeEventHandlerFrame::NodeFrameIdentity(self, target)
    }
    fn HitNodeFrameIdentity(
        &self,
        node: &interaction::hit_test::core::layout::hit_test_result::NodeRef,
    ) -> Option<Rc<()>> {
        NativeEventHandlerFrame::HitNodeFrameIdentity(self, node)
    }
    fn AdjustmentScales(&self) -> (f32, f32, f32) {
        NativeEventHandlerFrame::AdjustmentScales(self)
    }
    fn Identity(&self) -> Rc<()> {
        NativeEventHandlerFrame::Identity(self).expect("live EventHandler frame identity")
    }
    fn LocalFrameRootIdentity(&self) -> Rc<()> {
        let domain = self.Domain();
        domain
            .FocusInteraction(domain.root)
            .expect("local root input owner")
            .SourceInputOwners()
            .pointer_frame
            .identity
            .clone()
    }
    fn HasPage(&self) -> bool {
        self.domain
            .upgrade()
            .is_some_and(|domain| domain.page.upgrade().is_some())
    }
    fn IsAttached(&self) -> bool {
        NativeEventHandlerFrame::IsAttached(self)
    }
    fn IsOutermostMainFrame(&self) -> bool {
        self.domain
            .upgrade()
            .is_some_and(|domain| self.token == domain.root)
    }
    fn IsDescendantOf(&self, identity: &Rc<()>) -> bool {
        NativeEventHandlerFrame::IsDescendantOf(self, identity)
    }
    fn SubframeForTarget(&self, target: &EventTargetRef) -> PageEventHandlerSubframe {
        let Some(id) = target.NodeId() else {
            return PageEventHandlerSubframe::None;
        };
        let domain = self.Domain();
        let Some(token) = self
            .NodeFrameIdentity(target)
            .and_then(|identity| domain.EventHandlerFrameToken(&identity))
        else {
            return PageEventHandlerSubframe::None;
        };
        // EventHandler's canonical node registry has already admitted target
        // to this Document. Match event_handling_util.cc's LayoutEmbeddedContent
        // gate before asking the DOM-owned content-frame relation.
        let embedded = domain
            .WithLayout(token, |layout| {
                layout
                    .Output()
                    .and_then(|output| output.Node(id))
                    .and_then(|node| node.EmbeddedChildFrameSize())
                    .is_some()
            })
            .unwrap_or(false);
        if !embedded {
            return PageEventHandlerSubframe::None;
        }
        let Some(document) = domain.Document(token) else {
            return PageEventHandlerSubframe::None;
        };
        let frame = {
            let document = document.borrow();
            let document = document.GetDocument();
            document
                .FindNodeById(id)
                .and_then(|node| document.ContentFrame(node))
        };
        let Some(frame) = frame else {
            return PageEventHandlerSubframe::None;
        };
        if frame.IsRemoteFrame() {
            return PageEventHandlerSubframe::Remote;
        }
        let Some(owner) = domain.FocusInteraction(frame.Identity()) else {
            return PageEventHandlerSubframe::None;
        };
        PageEventHandlerSubframe::Local(owner.SourceInputOwners().pointer_frame.identity.clone())
    }
}
