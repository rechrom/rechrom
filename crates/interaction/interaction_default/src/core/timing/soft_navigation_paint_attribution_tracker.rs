//! Chromium 6c1d401f soft_navigation_paint_attribution_tracker.h:93-236,
//! .cc:20-235. Per-node marked/propagated state belongs to this retained owner.
//! Native Layout/PrePaint route small facts and consume the returned flags;
//! this is not a second DOM or Layout tree.
use super::soft_navigation_context::SoftNavigationContext;
use interaction_dispatch::core::dom::events::event::{EventTarget, EventTargetRef};
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::{Rc, Weak},
};

struct NodeState {
    context: Rc<SoftNavigationContext>,
    modification_id: u64,
}
struct Entry {
    node: Weak<dyn EventTarget>,
    state: Rc<NodeState>,
}
fn Key(node: &EventTargetRef) -> usize {
    Rc::as_ptr(node) as *const () as usize
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrePaintUpdateResult {
    PropagateAncestorNode,
    PropagateCurrentNode,
}
/// Facts for the one LayoutObject currently visited by the original prepaint
/// walk. Anonymous objects retain `node == None`; no synthetic DOM identity.
pub struct SoftNavigationPrePaintFacts {
    pub node: Option<EventTargetRef>,
    pub is_layout_text: bool,
    pub node_is_text_type: bool,
    pub node_is_text_node: bool,
    pub is_paint_timing_image: bool,
    pub generating_node: Option<EventTargetRef>,
    pub contentful_node_has_layout: bool,
    pub contentful_node_is_paint_timing_image: bool,
}
pub struct SoftNavigationPrePaintUpdate {
    pub propagation: PrePaintUpdateResult,
    /// TextPaintTimingDetector::ResetPaintTrackingOnInteraction is needed for
    /// this contentful node, after its context actually changes.
    pub reset_text_tracking: Option<EventTargetRef>,
}
#[derive(Default)]
pub struct SoftNavigationPaintAttributionTracker {
    generation: Cell<u64>,
    last_context_id: Cell<u64>,
    marked: RefCell<HashMap<usize, Entry>>,
    propagated: RefCell<HashMap<usize, Entry>>,
}
impl SoftNavigationPaintAttributionTracker {
    fn State(map: &RefCell<HashMap<usize, Entry>>, node: &EventTargetRef) -> Option<Rc<NodeState>> {
        let key = Key(node);
        let mut map = map.borrow_mut();
        if map
            .get(&key)
            .is_some_and(|entry| entry.node.upgrade().is_none())
        {
            map.remove(&key);
        }
        map.get(&key).map(|entry| entry.state.clone())
    }
    /// cc:36-85. Caller supplies the actual UA-shadow host only for text-type
    /// nodes, using the canonical shadow-root owner. The returned node is the
    /// exact native `MarkSoftNavigationContextChanged` producer.
    pub fn MarkNodeAsDirectlyModified(
        &self,
        node: EventTargetRef,
        text_type: bool,
        user_agent_shadow_host: Option<EventTargetRef>,
        context: Rc<SoftNavigationContext>,
    ) -> Option<EventTargetRef> {
        let node = if text_type {
            user_agent_shadow_host.unwrap_or(node)
        } else {
            node
        };
        if context.ContextId() != self.last_context_id.get() {
            self.last_context_id.set(context.ContextId());
            self.generation.set(self.generation.get().wrapping_add(1));
        }
        let generation = self.generation.get();
        if Self::State(&self.marked, &node).is_some_and(|s| s.modification_id == generation) {
            return None;
        }
        self.marked.borrow_mut().insert(
            Key(&node),
            Entry {
                node: Rc::downgrade(&node),
                state: Rc::new(NodeState {
                    context: context.clone(),
                    modification_id: generation,
                }),
            },
        );
        context.AddModifiedNode();
        Some(node)
    }
    pub fn GetSoftNavigationContextForNode(
        &self,
        node: &EventTargetRef,
    ) -> Option<Rc<SoftNavigationContext>> {
        Self::State(&self.propagated, node).map(|s| s.context.clone())
    }
    pub fn IsAttributable(
        &self,
        node: &EventTargetRef,
        context: Option<&Rc<SoftNavigationContext>>,
    ) -> bool {
        context.is_some_and(|context| {
            self.GetSoftNavigationContextForNode(node)
                .is_some_and(|c| Rc::ptr_eq(&c, context))
        })
    }
    pub fn IsMarkedForTesting(
        &self,
        node: &EventTargetRef,
        context: &Rc<SoftNavigationContext>,
    ) -> bool {
        Self::State(&self.marked, node).is_some_and(|s| Rc::ptr_eq(&s.context, context))
    }
    /// cc:88-124. `has_layout` must query the contentful node itself (pseudo
    /// images can name a host without a LayoutObject), rather than the object
    /// currently traversed. Image contexts do not reset text tracking.
    fn MarkNodeForPaintTrackingIfNeeded(
        &self,
        node: EventTargetRef,
        state: Rc<NodeState>,
        has_layout: bool,
        is_image: bool,
    ) -> Option<EventTargetRef> {
        if !has_layout {
            return None;
        }
        let previous = Self::State(&self.propagated, &node);
        if previous
            .as_ref()
            .is_some_and(|p| p.modification_id >= state.modification_id)
        {
            return None;
        }
        let changed = previous
            .as_ref()
            .is_none_or(|p| !Rc::ptr_eq(&p.context, &state.context));
        self.propagated.borrow_mut().insert(
            Key(&node),
            Entry {
                node: Rc::downgrade(&node),
                state,
            },
        );
        (changed && !is_image).then_some(node)
    }
    /// cc:127-218. Runs exactly once for the original LayoutObject visit.
    pub fn UpdateOnPrePaint(
        &self,
        facts: SoftNavigationPrePaintFacts,
        context_container_root: Option<&EventTargetRef>,
        text_aggregator: Option<EventTargetRef>,
    ) -> SoftNavigationPrePaintUpdate {
        let mut inherited = context_container_root.and_then(|root| Self::State(&self.marked, root));
        let mut propagation = PrePaintUpdateResult::PropagateAncestorNode;
        if let Some(node) = &facts.node {
            if let Some(state) = Self::State(&self.marked, node) {
                if inherited
                    .as_ref()
                    .is_none_or(|ancestor| state.modification_id > ancestor.modification_id)
                {
                    inherited = Some(state);
                    if facts.node_is_text_type {
                        self.marked.borrow_mut().remove(&Key(node));
                    }
                    propagation = PrePaintUpdateResult::PropagateCurrentNode;
                } else {
                    self.marked.borrow_mut().remove(&Key(node));
                }
            }
        }
        let reset_text_tracking = inherited.and_then(|state| {
            let contentful = match facts.node {
                None if facts.is_layout_text => text_aggregator,
                Some(node) if facts.node_is_text_type || facts.is_paint_timing_image => {
                    if facts.node_is_text_node {
                        text_aggregator
                    } else {
                        Some(
                            facts
                                .generating_node
                                .expect("contentful LayoutObject generating node"),
                        )
                    }
                }
                _ => None,
            }?;
            self.MarkNodeForPaintTrackingIfNeeded(
                contentful,
                state,
                facts.contentful_node_has_layout,
                facts.contentful_node_is_paint_timing_image,
            )
        });
        SoftNavigationPrePaintUpdate {
            propagation,
            reset_text_tracking,
        }
    }
}
