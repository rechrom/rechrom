//! EventHandler hit testing over Page's call-scoped resident Layout/Paint query.
//! Source: core/input/event_handler.h:282-288; event_handler.cc:371-451,
//! 888-891,2633-2674; core/dom/document.cc:5407-5443;
//! core/input/event_handling_util.cc:25-45,92-100.
use super::{
    page_event_handler_frame::PageEventHandlerFrameRegistry,
    page_event_handler_node::{PageEventHandlerNodes, RequiredEventHandlerPort},
    page_pointer_hit_test::{PagePointerGeometry, PointerHitResult},
    page_pointer_scroll_controls::PagePointerScrollControlOwners,
};
use crate::core::input::event_handler::{
    EventHandlerHitResult, EventHandlerHitTest, EventHandlerScrollbar, NodeRef,
};
use foundation::{gfx, PhysicalOffset, PhysicalRect, PhysicalSize};
use interaction_hit_test::core::layout::{
    hit_test_location::HitTestLocation, hit_test_request::RequestType,
    hit_test_result::HitTestResultRequest,
};
use interaction_input::public::common::input::web_mouse_event::WebMouseEvent;
use std::rc::Rc;

#[derive(Clone)]
pub struct PageEventHandlerHitResult {
    native: PointerHitResult,
    nodes: Rc<PageEventHandlerNodes>,
    controls: Rc<PagePointerScrollControlOwners>,
    /// Only original canonical document/node registries selected by this
    /// native result are retained, never a frame or copied node tree.
    foreign_nodes: Vec<Rc<PageEventHandlerNodes>>,
}
impl PageEventHandlerHitResult {
    pub fn new(
        native: PointerHitResult,
        nodes: Rc<PageEventHandlerNodes>,
        controls: Rc<PagePointerScrollControlOwners>,
    ) -> Self {
        Self {
            native,
            nodes,
            controls,
            foreign_nodes: Vec::new(),
        }
    }
    pub fn new_with_registry<'scope>(
        native: PointerHitResult,
        nodes: Rc<PageEventHandlerNodes>,
        controls: Rc<PagePointerScrollControlOwners>,
        registry: &dyn PageEventHandlerFrameRegistry<'scope>,
    ) -> Self {
        let mut result = Self::new(native, nodes, controls);
        // These are the existing result's target owners, including candidates
        // already collected by a list-based hit test. No fragment walk occurs.
        let mut admit = |node: &interaction_hit_test::core::layout::hit_test_result::NodeRef| {
            if result.nodes.OwnsHitNode(node)
                || result
                    .foreign_nodes
                    .iter()
                    .any(|owner| owner.OwnsHitNode(node))
            {
                return;
            }
            let owner = registry
                .NodesForHit(node)
                .expect("native hit target has a canonical frame/document owner");
            assert!(
                owner.OwnsHitNode(node),
                "frame registry validates native DOM identity before resolving its ID"
            );
            if !result
                .foreign_nodes
                .iter()
                .any(|existing| Rc::ptr_eq(existing, &owner))
            {
                result.foreign_nodes.push(owner);
            }
        };
        for node in [
            result.native.InnerNode(),
            result.native.InnerElement(),
            result.native.InnerPossiblyPseudoElement(),
        ]
        .into_iter()
        .flatten()
        {
            admit(node);
        }
        if result.native.GetHitTestRequest().ListBased() {
            for node in result.native.ListBasedTestResult().iter() {
                admit(node);
            }
        }
        result
    }
    pub fn Native(&self) -> &PointerHitResult {
        &self.native
    }
    pub fn NativeMut(&mut self) -> &mut PointerHitResult {
        &mut self.native
    }
    pub fn Nodes(&self) -> &Rc<PageEventHandlerNodes> {
        &self.nodes
    }
    fn NodeForHit(
        &self,
        node: &interaction_hit_test::core::layout::hit_test_result::NodeRef,
    ) -> Option<NodeRef> {
        self.nodes.NodeForHit(node).or_else(|| {
            self.foreign_nodes
                .iter()
                .find_map(|owner| owner.NodeForHit(node))
        })
    }
    fn HitTestNode(
        &self,
        node: &NodeRef,
    ) -> Option<interaction_hit_test::core::layout::hit_test_result::NodeRef> {
        self.nodes.HitTestNode(node).or_else(|| {
            self.foreign_nodes
                .iter()
                .find_map(|owner| owner.HitTestNode(node))
        })
    }
}
impl EventHandlerHitResult for PageEventHandlerHitResult {
    fn HitTestRequestType(&self) -> u32 {
        self.native.GetHitTestRequest().GetType()
    }
    fn InnerNode(&self) -> Option<NodeRef> {
        self.NodeForHit(self.native.InnerNode()?)
    }
    fn InnerElement(&self) -> Option<NodeRef> {
        self.NodeForHit(self.native.InnerElement()?)
    }
    fn InnerPossiblyPseudoElement(&self) -> Option<NodeRef> {
        self.NodeForHit(self.native.InnerPossiblyPseudoElement()?)
    }
    fn GetScrollbar(&self) -> Option<Rc<dyn EventHandlerScrollbar>> {
        Some(
            self.controls.EventHandlerScrollbar(
                self.native
                    .GetScrollbar()?
                    .NativeScrollbarOwner()
                    .expect("resident hit testing returns the native scrollbar owner")
                    .clone(),
            ),
        )
    }
    fn SetToShadowHostIfInUAShadowRoot(&mut self) {
        self.native.SetToShadowHostIfInUAShadowRoot();
    }
    fn SetNodeAndPosition(&mut self, node: NodeRef, point: &HitTestLocation) {
        self.native.SetNodeAndPosition(
            Some(
                self.HitTestNode(&node)
                    .expect("capture target belongs to the bound resident DOM owner"),
            ),
            *point.Point(),
        );
    }
    fn SetScrollbar(&mut self, scrollbar: Option<Rc<dyn EventHandlerScrollbar>>) {
        self.native.SetScrollbar(scrollbar.map(|bar| {
            self.controls
                .HitTestScrollbar(&bar)
                .expect("capture scrollbar belongs to the retained scroll-control owner")
        }));
    }
    fn SetURLElement(&mut self, node: Option<NodeRef>) {
        self.native.SetURLElement(node.map(|node| {
            self.HitTestNode(&node)
                .expect("URL element belongs to the bound resident DOM owner")
        }));
    }
    fn SetIsOverEmbeddedContentView(&mut self, value: bool) {
        self.native.SetIsOverEmbeddedContentView(value);
    }
}

/// Reads readiness from the live LocalFrameView at each source call. In
/// particular DidFirstLayout is not inferred from an empty/nonempty fragment.
pub trait PageEventHandlerHitTestLifecycle {
    fn HasContentLayoutObject(&self) -> bool;
    fn HasView(&self) -> bool;
    fn DidFirstLayout(&self) -> bool;
    fn LifecycleUpdatesActive(&self) -> bool;
    fn FrameViewSize(&self) -> PhysicalSize;
    fn UnboundedElementEnabled(&self) -> bool;
    fn HasActiveUnboundedElements(&self) -> bool;
}
pub trait PageEventHandlerCoordinates {
    fn ConvertLocationToRoot(
        &self,
        location: &HitTestLocation,
        visual_overflow: bool,
    ) -> HitTestLocation;
    fn ConvertLocationFromRoot(&self, location: &HitTestLocation) -> HitTestLocation;
    fn ViewportToFrame(&self, location: &HitTestLocation) -> HitTestLocation;
    fn ConvertPointFromRootFrame(&self, point: gfx::PointF) -> gfx::PointF;
}
pub trait PageEventHandlerHoverState {
    fn UpdateHoverActiveState(
        &self,
        active: bool,
        update_active_chain: bool,
        node: Option<NodeRef>,
    );
}
pub struct PageEventHandlerHitTest<'scope> {
    geometry: Rc<dyn PagePointerGeometry>,
    lifecycle: Rc<dyn PageEventHandlerHitTestLifecycle + 'scope>,
    coordinates: Rc<dyn PageEventHandlerCoordinates + 'scope>,
    hover: Rc<dyn PageEventHandlerHoverState + 'scope>,
    nodes: Rc<PageEventHandlerNodes>,
    controls: Rc<PagePointerScrollControlOwners>,
    registry: std::rc::Weak<dyn PageEventHandlerFrameRegistry<'scope> + 'scope>,
}
impl<'scope> PageEventHandlerHitTest<'scope> {
    pub fn new(
        geometry: Option<Rc<dyn PagePointerGeometry>>,
        lifecycle: Option<Rc<dyn PageEventHandlerHitTestLifecycle + 'scope>>,
        coordinates: Option<Rc<dyn PageEventHandlerCoordinates + 'scope>>,
        hover: Option<Rc<dyn PageEventHandlerHoverState + 'scope>>,
        nodes: Rc<PageEventHandlerNodes>,
        controls: Option<Rc<PagePointerScrollControlOwners>>,
        registry: Option<Rc<dyn PageEventHandlerFrameRegistry<'scope> + 'scope>>,
    ) -> Result<Self, RequiredEventHandlerPort> {
        Ok(Self {
            geometry: geometry.ok_or(RequiredEventHandlerPort::HitTestGeometry)?,
            lifecycle: lifecycle.ok_or(RequiredEventHandlerPort::HitTestLifecycle)?,
            coordinates: coordinates.ok_or(RequiredEventHandlerPort::FrameCoordinates)?,
            hover: hover.ok_or(RequiredEventHandlerPort::HoverActiveState)?,
            nodes,
            controls: controls.ok_or(RequiredEventHandlerPort::ScrollControls)?,
            registry: Rc::downgrade(&registry.ok_or(RequiredEventHandlerPort::FrameRegistry)?),
        })
    }
    pub(crate) fn PrepareLifecycleExceptPaint(&self) {
        if let Err(error) = self.geometry.PrepareLifecycleExceptPaint() {
            panic!("pointer hit-test lifecycle: {error:?}");
        }
    }
    fn Query(
        &self,
        location: &HitTestLocation,
        bits: u32,
        no_lifecycle_update: bool,
    ) -> PageEventHandlerHitResult {
        if !no_lifecycle_update {
            if let Err(error) = self.geometry.PrepareLifecycleExceptPaint() {
                panic!("pointer hit-test lifecycle: {error:?}");
            }
        }
        let hit = match self
            .geometry
            .EventHandlerHitTestNoLifecycleUpdate(location.clone(), bits)
        {
            Ok(hit) => hit,
            Err(error) => panic!("pointer hit-test geometry: {error:?}"),
        };
        PageEventHandlerHitResult::new_with_registry(
            hit.result,
            self.nodes.clone(),
            self.controls.clone(),
            self.registry
                .upgrade()
                .expect("interaction binder retains the scoped frame registry")
                .as_ref(),
        )
    }
}
impl EventHandlerHitTest<PageEventHandlerHitResult> for PageEventHandlerHitTest<'_> {
    fn HasContentLayoutObject(&self) -> bool {
        self.lifecycle.HasContentLayoutObject()
    }
    fn HasView(&self) -> bool {
        self.lifecycle.HasView()
    }
    fn DidFirstLayout(&self) -> bool {
        self.lifecycle.DidFirstLayout()
    }
    fn LifecycleUpdatesActive(&self) -> bool {
        self.lifecycle.LifecycleUpdatesActive()
    }
    fn NewResult(&self, hit_type: u32, location: &HitTestLocation) -> PageEventHandlerHitResult {
        PageEventHandlerHitResult::new(
            PointerHitResult::new(&HitTestResultRequest::new(hit_type, None, None), location),
            self.nodes.clone(),
            self.controls.clone(),
        )
    }
    fn HitTest(
        &self,
        location: &HitTestLocation,
        result: &mut PageEventHandlerHitResult,
        no_lifecycle_update: bool,
    ) {
        *result = self.Query(location, result.HitTestRequestType(), no_lifecycle_update);
    }
    fn UpdateHoverActiveState(
        &self,
        active: bool,
        update_active_chain: bool,
        node: Option<NodeRef>,
    ) {
        self.hover
            .UpdateHoverActiveState(active, update_active_chain, node);
    }
    fn ConvertLocationToRoot(
        &self,
        location: &HitTestLocation,
        visual_overflow: bool,
    ) -> HitTestLocation {
        self.coordinates
            .ConvertLocationToRoot(location, visual_overflow)
    }
    fn ConvertLocationFromRoot(&self, location: &HitTestLocation) -> HitTestLocation {
        self.coordinates.ConvertLocationFromRoot(location)
    }
    fn ViewportToFrame(&self, location: &HitTestLocation) -> HitTestLocation {
        self.coordinates.ViewportToFrame(location)
    }
    fn ContentPointFromRootFrame(&self, event: &WebMouseEvent) -> HitTestLocation {
        let point = event.PositionInRootFrame();
        let point = gfx::PointF::new(point.x(), point.y());
        let point = if self.HasView() {
            self.coordinates.ConvertPointFromRootFrame(point)
        } else {
            point
        };
        HitTestLocation::FromPhysicalOffset(PhysicalOffset::FromPointFRound(&point))
    }
    fn DocumentPointForMousePress(&self, event: &WebMouseEvent) -> HitTestLocation {
        let point = event.PositionInRootFrame();
        self.ConvertLocationFromRoot(&HitTestLocation::FromPoint(gfx::Point::new(
            point.x().floor() as i32,
            point.y().floor() as i32,
        )))
    }
    fn HitTestResultInFrame(
        &self,
        hit_type: u32,
        location: &HitTestLocation,
    ) -> PageEventHandlerHitResult {
        debug_assert!(!location.IsRectBasedTest());
        let result = self.NewResult(hit_type, location);
        if !self.HasContentLayoutObject() {
            return result;
        }
        if self.HasView()
            && (!self.lifecycle.UnboundedElementEnabled()
                || !self.lifecycle.HasActiveUnboundedElements())
            && !location.Intersects(&PhysicalRect::new(
                PhysicalOffset::default(),
                self.lifecycle.FrameViewSize(),
            ))
        {
            return result;
        }
        self.Query(location, hit_type, false)
    }
    fn PerformMouseEventHitTest(
        &self,
        hit_type: u32,
        location: &HitTestLocation,
        _event: &WebMouseEvent,
    ) -> PageEventHandlerHitResult {
        if !self.HasContentLayoutObject() || !self.HasView() || !self.DidFirstLayout() {
            return self.NewResult(
                hit_type,
                &HitTestLocation::FromPointF(gfx::PointF::default()),
            );
        }
        let result = self.Query(location, hit_type, false);
        if hit_type & RequestType::kReadOnly as u32 == 0 {
            self.UpdateHoverActiveState(
                hit_type & RequestType::kActive as u32 != 0,
                hit_type & RequestType::kMove as u32 == 0,
                result.InnerPossiblyPseudoElement(),
            );
        }
        result
    }
}
