//! EventHandler gesture targeting leaves over the canonical per-frame owners.
//! Source: event_handler.cc:1706-1749,1826-1868,1875-2026,2146-2182.
//! EventHandler retains routing/order; resident touch adjustment borrows Layout
//! only inside its existing native query, never in the handler's stored ports.
use super::*;
use crate::adapter::page_event_handler_frame::{
    PageEventHandlerFrameDomain, PageEventHandlerFrameRegistry, PageEventHandlerSubframe,
};
use crate::core::input::{
    event_handler::{
        gesture::{EventHandlerGestureTargeting, UnboundedGestureTarget},
        EventHandlerFrame,
    },
    touch_adjustment::{NodeRef as AdjustmentNode, TouchAdjustmentCandidateType},
};
use foundation::{gfx, PhysicalOffset};
use interaction_input::base::time::time::TimeTicks;

type Frame<'scope> = Rc<dyn EventHandlerFrame<'scope, Hit> + 'scope>;
pub struct PageEventHandlerGestureTargeting<'scope, 'listener, 'frame> {
    dispatcher: EventDispatcher<'listener, 'frame>,
    owners: ResidentCursorOwners,
    nodes: Rc<PageEventHandlerNodes>,
    geometry: Rc<dyn PagePointerGeometry>,
    controls: Rc<PagePointerScrollControlOwners>,
    registry: Rc<dyn PageEventHandlerFrameRegistry<'scope> + 'scope>,
    domain: Rc<dyn PageEventHandlerFrameDomain + 'scope>,
    errors: Rc<InputErrors>,
}
impl<'scope, 'listener, 'frame> PageEventHandlerGestureTargeting<'scope, 'listener, 'frame> {
    pub fn new(
        dispatcher: &EventDispatcher<'listener, 'frame>,
        owners: &ResidentCursorOwners,
        nodes: Rc<PageEventHandlerNodes>,
        geometry: Rc<dyn PagePointerGeometry>,
        controls: Rc<PagePointerScrollControlOwners>,
        registry: Rc<dyn PageEventHandlerFrameRegistry<'scope> + 'scope>,
        domain: Rc<dyn PageEventHandlerFrameDomain + 'scope>,
    ) -> Rc<Self> {
        assert!(Rc::ptr_eq(nodes.Targets(), &dispatcher.targets));
        Rc::new(Self {
            dispatcher: dispatcher.clone(),
            owners: owners.clone(),
            nodes,
            geometry,
            controls,
            registry,
            domain,
            errors: Rc::new(InputErrors::default()),
        })
    }
    pub fn Finish(&self) -> Result<(), PointerInputError> {
        self.errors.error.borrow_mut().take().map_or(Ok(()), Err)
    }
    fn Frame(&self, identity: &Rc<()>) -> Option<Frame<'scope>> {
        let frame = self.registry.FrameForIdentity(identity);
        if frame.is_none() {
            self.errors.Fault(PointerInputError::LocalFrameRouting);
        }
        frame
    }
    fn Target(&self, node: &NodeRef) -> Option<EventTargetRef> {
        let target = self.registry.EventTarget(node);
        if target.is_none() {
            self.errors.Fault(PointerInputError::DetachedTarget);
        }
        target
    }
}
impl<'scope> EventHandlerGestureTargeting<'scope, Hit>
    for PageEventHandlerGestureTargeting<'scope, '_, '_>
{
    fn Now(&self) -> TimeTicks {
        self.dispatcher.clock.Now()
    }
    fn InnerNodeFrame(&self, result: &Hit) -> Option<Frame<'scope>> {
        use crate::core::input::event_handler::EventHandlerHitResult;
        self.NodeFrame(&result.InnerNode()?)
    }
    fn NodeFrame(&self, node: &NodeRef) -> Option<Frame<'scope>> {
        let target = self.Target(node)?;
        self.domain
            .NodeFrameIdentity(&target)
            .and_then(|identity| self.Frame(&identity))
    }
    fn ParentLocalFrame(&self) -> Option<Frame<'scope>> {
        self.domain
            .ParentLocalFrameIdentity()
            .and_then(|identity| self.Frame(&identity))
    }
    fn OwnerElement(&self) -> Option<NodeRef> {
        let target = self.domain.OwnerElement()?;
        let node = self.registry.NodeForTarget(&target);
        if node.is_none() {
            self.errors.Fault(PointerInputError::DetachedTarget);
        }
        node
    }
    fn HoverElement(&self) -> Option<NodeRef> {
        let id = self.dispatcher.state.borrow().hovered_node_id?;
        let node = self.nodes.NodeForId(id);
        if node.is_none() {
            self.errors.Fault(PointerInputError::DetachedTarget);
        }
        node
    }
    fn ContentFrameForOwner(&self, node: &NodeRef) -> Option<Frame<'scope>> {
        let target = self.Target(node)?;
        match self.domain.SubframeForTarget(&target) {
            PageEventHandlerSubframe::None | PageEventHandlerSubframe::Remote => None,
            PageEventHandlerSubframe::Local(identity) => self.Frame(&identity),
        }
    }
    fn SubframeForActiveUnboundedElement(
        &self,
        _: gfx::PointF,
    ) -> Option<UnboundedGestureTarget<'scope, Hit>> {
        // This source branch has no owner when the source runtime feature is
        // disabled. If enabled, expose the missing live overlay connection.
        if foundation::RuntimeEnabledFeatures::UnboundedElementEnabled() {
            self.errors.Fault(PointerInputError::SourceLeaf(
                "EventHandler::GetTargetSubframeForActiveUnboundedElement",
            ));
        }
        None
    }
    fn AdjustmentScales(&self) -> (f32, f32, f32) {
        self.domain.AdjustmentScales()
    }
    fn IsOverResizer(&self, result: &Hit) -> bool {
        result.Native().IsOverResizer()
    }
    fn FindBestTouchAdjustmentCandidate(
        &self,
        candidate: TouchAdjustmentCandidateType,
        location: &HitTestLocation,
        result: &Hit,
    ) -> Option<(AdjustmentNode, gfx::Point)> {
        if self.errors.failed.get() {
            return None;
        }
        let query = self.dispatcher.WithPointerManager(
            &self.owners,
            self.geometry.clone(),
            self.controls.clone(),
            |_, pointer| {
                let value = self
                    .geometry
                    .BestTouchAdjustment(
                        candidate,
                        &crate::adapter::page_pointer_hit_test::PointerHit {
                            location: location.clone(),
                            result: result.Native().clone(),
                        },
                        pointer,
                    )
                    .map_err(PointerInputError::Geometry)?;
                let Some((id, point)) = value else {
                    return Ok(None);
                };
                let node = result
                    .Native()
                    .ListBasedTestResult()
                    .iter()
                    .find(|node| node.DOMNodeId() == Some(id))
                    .cloned()
                    .map(Ok)
                    .unwrap_or_else(|| {
                        self.geometry
                            .EventHandlerHitTestNode(id)
                            .map_err(PointerInputError::Geometry)
                            .and_then(|node| node.ok_or(PointerInputError::DetachedTarget))
                    })?;
                Ok(Some((node, point)))
            },
        );
        match query {
            Ok((value, _)) => value,
            Err(error) => {
                self.errors.Fault(error);
                None
            }
        }
    }
    fn ResolveRectBasedTest(
        &self,
        result: &mut Hit,
        node: &AdjustmentNode,
        point: PhysicalOffset,
    ) -> HitTestLocation {
        result.NativeMut().ResolveRectBasedTest(node, point)
    }
}
