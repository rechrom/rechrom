//! Page assembly for Blink's single retained `ScrollManager` owner.
//!
//! LayoutNG keeps every `LayoutObject`/`ScrollableArea` identity. This module
//! projects only the operations consumed by `ScrollManager`; it does not copy
//! a containing-block tree, offsets, animator state, or scroll target state.
//! The same returned `Rc<ScrollManager>` is intended for EventHandler's wheel,
//! keyboard, mouse and gesture managers.

use std::{
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

use interaction_dispatch::core::dom::events::event::EventTargetRef;
use interaction_scroll::core::input::{
    event_handler::{
        EventHandlerMouseScrollState, EventHandlerScrollRouting, EventHandlerViewportCoordinates,
    },
    mouse_wheel_event_manager::{
        MouseWheelEventManagerContext, MouseWheelFeatures, WheelDocument, WheelFrame,
        WheelFrameView, WheelScrollableArea, WheelScrollableTraversal, WheelTargetRoute,
        WheelTargetRouting,
    },
    root_frame_viewport::{RootFrameLayoutViewport, RootFrameViewport},
    scroll_manager::{
        ScrollAutoscrollController, ScrollCompletion, ScrollCompletionMode, ScrollDirection,
        ScrollDirectionPhysical, ScrollEffect, ScrollEffectEmitter, ScrollFrame, ScrollFrameFacts,
        ScrollFrameServices, ScrollGranularity, ScrollKeyboardEventManager, ScrollLayoutBox,
        ScrollLayoutBoxFacts, ScrollLayoutBoxRef, ScrollManager, ScrollNode, ScrollNodeFacts,
        ScrollNodeRef, ScrollOffsetMutation, ScrollableArea, ScrollableAreaFacts,
        ScrollableAreaRef, SnapScrollKind, UserScrollRequest,
    },
};
use layoutng_assembly::{
    native_autoscroll_source::{
        NativeAutoscrollLayout, NativeLogicalScrollArea, NativeLogicalScrollBoxFacts,
        NativeLogicalScrollNodeFacts, NativeLogicalScrollingArea,
    },
    native_scroll_source::{NativeLayoutSnapPlan, NativeSnapAnimationState},
    native_user_scroll_source::{
        NativeScrollCompletionMode, NativeScrollFinishedPlan, NativeUserScrollAnimationRequest,
        NativeUserScrollAnimationState, NativeUserScrollAreaFacts, NativeUserScrollGranularity,
        NativeUserScrollPlan,
    },
    native_wheel_scrollable_traversal::NativeWheelScrollableArea,
};

/// Live LayoutNG owner required by the translated `ScrollManager`.
///
/// Implementations borrow their resident LayoutEngine for one method call and
/// enter its native scroll scope. Handles are the original persistent native
/// identities; no method may build or retain a parallel scroll tree.
pub trait PageLogicalScrollLayout {
    fn LayoutForNode(&self, node_id: u64) -> Option<NativeAutoscrollLayout>;
    fn LayoutView(&self) -> Option<NativeAutoscrollLayout>;
    fn NodeFacts(&self, layout: &NativeAutoscrollLayout) -> Option<NativeLogicalScrollNodeFacts>;
    fn EnclosingBox(&self, layout: &NativeAutoscrollLayout) -> Option<NativeAutoscrollLayout>;
    fn LayoutBox(&self, layout: &NativeAutoscrollLayout) -> Option<NativeAutoscrollLayout>;
    fn LayoutBoxForScrolling(
        &self,
        layout: &NativeAutoscrollLayout,
    ) -> Option<NativeAutoscrollLayout>;
    fn BoxFacts(&self, layout: &NativeAutoscrollLayout) -> NativeLogicalScrollBoxFacts;
    fn BoxNode(&self, layout: &NativeAutoscrollLayout) -> Option<NativeAutoscrollLayout>;
    fn ContainingBlock(&self, layout: &NativeAutoscrollLayout) -> Option<NativeAutoscrollLayout>;
    fn ScrollableArea(&self, layout: &NativeAutoscrollLayout) -> Option<NativeLogicalScrollArea>;
    fn AreaForScrolling(
        &self,
        layout: &NativeAutoscrollLayout,
    ) -> Option<NativeLogicalScrollingArea>;
    fn LayoutViewportArea(&self) -> Option<NativeLogicalScrollArea>;

    fn AreaFacts(&self, area: &NativeLogicalScrollArea) -> Option<NativeUserScrollAreaFacts>;
    fn AreaOffset(&self, area: &NativeLogicalScrollArea) -> Option<foundation::gfx::Vector2dF>;
    fn PlanSnapForUserScroll(
        &self,
        area: &NativeLogicalScrollArea,
        kind: layoutng_assembly::internal::layout_scrollable_area::NativeUserSnapScrollKind,
        direction: foundation::PhysicalDirection,
        line_step: f32,
        animation: NativeSnapAnimationState,
    ) -> Option<(bool, NativeLayoutSnapPlan)>;
    fn UserScroll(
        &self,
        area: &NativeLogicalScrollArea,
        granularity: NativeUserScrollGranularity,
        delta: foundation::gfx::Vector2dF,
        source_type: interaction_scroll::core::input::scroll_manager::ScrollSourceType,
        animation: NativeUserScrollAnimationState,
    ) -> Option<NativeUserScrollPlan>;
    fn WillConsumeUserScroll(
        &self,
        area: &NativeLogicalScrollArea,
        granularity: NativeUserScrollGranularity,
        delta: foundation::gfx::Vector2dF,
        animation: NativeUserScrollAnimationState,
    ) -> Option<bool>;
    fn OnScrollFinished(
        &self,
        area: &NativeLogicalScrollArea,
        enqueue_scrollend: bool,
    ) -> Option<NativeScrollFinishedPlan>;

    fn WheelScrollableAreas(
        &self,
        target_node_id: u64,
    ) -> Box<dyn Iterator<Item = NativeWheelScrollableArea>>;
}

/// Existing animator/DOM effect owners outside LayoutNG.
///
/// Acceptance is synchronous, matching `ScrollAnimator::UserScroll`. An
/// accepted request owns the completion until animation termination; rejection
/// returns it so the adapter can run the source fallback immediately.
pub trait PageScrollRuntime {
    fn UserAnimationState(&self, area: &NativeLogicalScrollArea) -> NativeUserScrollAnimationState;
    fn SnapAnimationState(&self, area: &NativeLogicalScrollArea) -> NativeSnapAnimationState;
    fn LineStep(&self) -> f32;
    fn ApplySnapPlan(&self, area: &NativeLogicalScrollArea, plan: NativeLayoutSnapPlan);
    fn ApplyImmediateUserScrollActions(
        &self,
        area: &NativeLogicalScrollArea,
        cancel_programmatic_animation: bool,
        finish_previous_animation: bool,
    );
    fn StartUserScrollAnimation(
        &self,
        area: &NativeLogicalScrollArea,
        request: NativeUserScrollAnimationRequest,
        completion: ScrollCompletion,
    ) -> Result<(), ScrollCompletion>;
    fn ApplyScrollFinished(&self, area: &NativeLogicalScrollArea, plan: NativeScrollFinishedPlan);
}

/// Live frame facts and source services not owned by LayoutNG.
pub trait PageScrollFrameState {
    fn FocusedNodeId(&self) -> Option<u64>;
    fn VisualViewportIsActive(&self) -> bool;
    fn MousePressed(&self) -> bool;
    fn BubbleLogicalScrollInParentFrame(
        &self,
        direction: ScrollDirection,
        granularity: ScrollGranularity,
    ) -> bool;
    fn AssertLayoutPrepared(&self, node_id: Option<u64>);
    fn RespectOverscrollBehaviorForScrollBubbling(&self) -> bool;
    fn AutoscrollController(&self) -> Option<Rc<dyn ScrollAutoscrollController>>;
    fn KeyboardEventManager(&self) -> Option<Rc<dyn ScrollKeyboardEventManager>>;
}

/// Canonical target identity resolver used by the wheel traversal. It must be
/// the same `DomEventTargets` registry used by wheel dispatch.
pub trait PageWheelTargetIdentity {
    fn NodeId(&self, target: &EventTargetRef) -> Option<u64>;
}

impl PageWheelTargetIdentity for super::dom_event::DomEventTargets {
    fn NodeId(&self, target: &EventTargetRef) -> Option<u64> {
        super::event::EventTargetResolver::NodeId(self, target)
    }
}

struct PageScrollOwner {
    layout: Rc<dyn PageLogicalScrollLayout>,
    runtime: Rc<dyn PageScrollRuntime>,
    frame: Rc<dyn PageScrollFrameState>,
    root_frame_viewport: RefCell<Option<ScrollableAreaRef>>,
    effects: ScrollEffectEmitter,
    areas: RefCell<HashMap<usize, Weak<PageScrollableArea>>>,
    wheel_areas: RefCell<HashMap<usize, Rc<dyn WheelScrollableArea>>>,
}

impl PageScrollOwner {
    fn Node(self: &Rc<Self>, layout: NativeAutoscrollLayout) -> ScrollNodeRef {
        Rc::new(PageScrollNode {
            owner: self.clone(),
            layout,
        })
    }

    fn Box(self: &Rc<Self>, layout: NativeAutoscrollLayout) -> ScrollLayoutBoxRef {
        Rc::new(PageScrollLayoutBox {
            owner: self.clone(),
            layout,
        })
    }

    fn NativeArea(self: &Rc<Self>, area: NativeLogicalScrollArea) -> Rc<PageScrollableArea> {
        let identity = area.Identity();
        if let Some(existing) = self.areas.borrow().get(&identity).and_then(Weak::upgrade) {
            return existing;
        }
        let wrapper = Rc::new(PageScrollableArea {
            owner: self.clone(),
            area,
        });
        self.areas
            .borrow_mut()
            .insert(identity, Rc::downgrade(&wrapper));
        wrapper
    }

    fn Area(self: &Rc<Self>, area: NativeLogicalScrollArea) -> ScrollableAreaRef {
        self.NativeArea(area)
    }

    fn WheelArea(self: &Rc<Self>, area: NativeWheelScrollableArea) -> Rc<dyn WheelScrollableArea> {
        let identity = area.Identity();
        if let Some(existing) = self.wheel_areas.borrow().get(&identity) {
            return existing.clone();
        }
        let wrapper: Rc<dyn WheelScrollableArea> = Rc::new(PageWheelScrollableArea { area });
        // Strong retention is intentional: MouseWheelEventManager stores
        // deferred fade owners as Weak across the complete wheel sequence.
        self.wheel_areas
            .borrow_mut()
            .insert(identity, wrapper.clone());
        wrapper
    }
}

struct PageScrollNode {
    owner: Rc<PageScrollOwner>,
    layout: NativeAutoscrollLayout,
}

/// DOM nodes without a LayoutObject still participate in ScrollManager's
/// focused-node ordering. The source manager sees the Node first, observes the
/// absent LayoutObject, then falls back to the layout viewport; it must not be
/// handed the viewport as though that were the focused node.
// Source: scroll_manager.cc:204-218.
struct PageNonLayoutScrollNode {
    node_id: u64,
}

impl ScrollNode for PageNonLayoutScrollNode {
    fn Facts(&self) -> ScrollNodeFacts {
        ScrollNodeFacts {
            dom_node_id: self.node_id,
            has_layout_object: false,
            is_element: false,
            is_effective_root_scroller: false,
            document_is_in_main_frame: true,
        }
    }

    fn EnclosingBox(&self) -> Option<ScrollLayoutBoxRef> {
        None
    }

    fn LayoutBox(&self) -> Option<ScrollLayoutBoxRef> {
        None
    }

    fn LayoutBoxForScrolling(&self) -> Option<ScrollLayoutBoxRef> {
        None
    }
}

impl ScrollNode for PageScrollNode {
    fn Facts(&self) -> ScrollNodeFacts {
        let facts = self
            .owner
            .layout
            .NodeFacts(&self.layout)
            .expect("retained logical-scroll node remains attached");
        NodeFacts(facts)
    }

    fn EnclosingBox(&self) -> Option<ScrollLayoutBoxRef> {
        self.owner
            .layout
            .EnclosingBox(&self.layout)
            .map(|layout| self.owner.Box(layout))
    }

    fn LayoutBox(&self) -> Option<ScrollLayoutBoxRef> {
        self.owner
            .layout
            .LayoutBox(&self.layout)
            .map(|layout| self.owner.Box(layout))
    }

    fn LayoutBoxForScrolling(&self) -> Option<ScrollLayoutBoxRef> {
        self.owner
            .layout
            .LayoutBoxForScrolling(&self.layout)
            .map(|layout| self.owner.Box(layout))
    }
}

fn NodeFacts(facts: NativeLogicalScrollNodeFacts) -> ScrollNodeFacts {
    ScrollNodeFacts {
        dom_node_id: facts.node_id,
        has_layout_object: true,
        is_element: facts.is_element,
        is_effective_root_scroller: facts.is_effective_root_scroller,
        document_is_in_main_frame: facts.document_is_in_main_frame,
    }
}

struct PageScrollLayoutBox {
    owner: Rc<PageScrollOwner>,
    layout: NativeAutoscrollLayout,
}

impl ScrollLayoutBox for PageScrollLayoutBox {
    fn Facts(&self) -> ScrollLayoutBoxFacts {
        let facts = self.owner.layout.BoxFacts(&self.layout);
        BoxFacts(facts)
    }

    fn Node(&self) -> Option<ScrollNodeRef> {
        self.owner
            .layout
            .BoxNode(&self.layout)
            .map(|layout| self.owner.Node(layout))
    }

    fn ContainingBlock(&self) -> Option<ScrollLayoutBoxRef> {
        self.owner
            .layout
            .ContainingBlock(&self.layout)
            .map(|layout| self.owner.Box(layout))
    }

    fn ScrollableArea(&self) -> Option<ScrollableAreaRef> {
        self.owner
            .layout
            .ScrollableArea(&self.layout)
            .map(|area| self.owner.Area(area))
    }

    fn AreaForScrolling(&self) -> Option<ScrollableAreaRef> {
        match self.owner.layout.AreaForScrolling(&self.layout)? {
            NativeLogicalScrollingArea::Area(area) => Some(self.owner.Area(area)),
            NativeLogicalScrollingArea::RootFrame => {
                self.owner.root_frame_viewport.borrow().as_ref().cloned()
            }
        }
    }
}

fn BoxFacts(facts: NativeLogicalScrollBoxFacts) -> ScrollLayoutBoxFacts {
    ScrollLayoutBoxFacts {
        is_horizontal_writing_mode: facts.is_horizontal_writing_mode,
        is_flipped_blocks_writing_mode: facts.is_flipped_blocks_writing_mode,
        is_global_root_scroller: facts.is_global_root_scroller,
        is_layout_view: facts.is_layout_view,
        overscroll_behavior_x: facts.overscroll_behavior_x,
        overscroll_behavior_y: facts.overscroll_behavior_y,
    }
}

struct PageScrollableArea {
    owner: Rc<PageScrollOwner>,
    area: NativeLogicalScrollArea,
}

impl PageScrollableArea {
    fn NativeFacts(&self) -> NativeUserScrollAreaFacts {
        self.owner
            .layout
            .AreaFacts(&self.area)
            .expect("retained logical ScrollableArea remains attached")
    }
}

impl ScrollableArea for PageScrollableArea {
    fn Facts(&self) -> ScrollableAreaFacts {
        let facts = self.NativeFacts();
        ScrollableAreaFacts {
            node_id: facts
                .node_id
                .expect("logical ScrollableArea in the DOM scroll chain owns a node"),
            scroll_element_id: facts.scroll_element_id,
            user_input_scrollable_horizontal: facts.user_input_scrollable_horizontal,
            user_input_scrollable_vertical: facts.user_input_scrollable_vertical,
            scrollable_horizontal: facts.scrollable_horizontal,
            scrollable_vertical: facts.scrollable_vertical,
            is_root_frame_viewport: facts.is_root_frame_viewport,
            scroll_affects_layout_viewport: facts.scroll_affects_layout_viewport,
            scroll_animator_enabled: self.owner.runtime.UserAnimationState(&self.area).enabled,
        }
    }

    fn SnapScroll(
        &self,
        kind: SnapScrollKind,
        direction: ScrollDirectionPhysical,
        _emit: &ScrollEffectEmitter,
    ) -> bool {
        use layoutng_assembly::internal::layout_scrollable_area::NativeUserSnapScrollKind;
        let kind = match kind {
            SnapScrollKind::Direction => NativeUserSnapScrollKind::Direction,
            SnapScrollKind::Page => NativeUserSnapScrollKind::Page,
            SnapScrollKind::Document => NativeUserSnapScrollKind::Document,
        };
        let direction = match direction {
            ScrollDirectionPhysical::kScrollUp => foundation::PhysicalDirection::kUp,
            ScrollDirectionPhysical::kScrollDown => foundation::PhysicalDirection::kDown,
            ScrollDirectionPhysical::kScrollLeft => foundation::PhysicalDirection::kLeft,
            ScrollDirectionPhysical::kScrollRight => foundation::PhysicalDirection::kRight,
        };
        let Some((handled, plan)) = self.owner.layout.PlanSnapForUserScroll(
            &self.area,
            kind,
            direction,
            self.owner.runtime.LineStep(),
            self.owner.runtime.SnapAnimationState(&self.area),
        ) else {
            return false;
        };
        self.owner.runtime.ApplySnapPlan(&self.area, plan);
        handled
    }

    fn UserScroll(
        &self,
        request: UserScrollRequest,
        completion: ScrollCompletion,
        emit: &ScrollEffectEmitter,
    ) -> interaction_scroll::core::scroll::scroll_types::ScrollConsumption {
        let granularity = NativeGranularity(request.granularity);
        let Some(plan) = self.owner.layout.UserScroll(
            &self.area,
            granularity,
            request.delta,
            request.source_type,
            self.owner.runtime.UserAnimationState(&self.area),
        ) else {
            completion(ScrollCompletionMode::kInterruptedByScroll);
            return interaction_scroll::core::scroll::scroll_types::ScrollConsumption::new(
                false,
                false,
                request.delta.x(),
                request.delta.y(),
            );
        };
        match plan {
            NativeUserScrollPlan::Completed {
                consumption,
                completion: mode,
                committed_offset,
                cancel_programmatic_animation,
                finish_previous_animation,
            } => {
                self.owner.runtime.ApplyImmediateUserScrollActions(
                    &self.area,
                    cancel_programmatic_animation,
                    finish_previous_animation,
                );
                if let (Some(node_id), Some(offset)) =
                    (self.NativeFacts().node_id, committed_offset)
                {
                    emit(ScrollEffect::SetOffset(ScrollOffsetMutation {
                        node_id,
                        offset,
                    }));
                }
                completion(CompletionMode(mode));
                Consumption(consumption)
            }
            NativeUserScrollPlan::Animate(request) => {
                let accepted = request.accepted_consumption;
                let rejected = request.rejected_consumption;
                let rejected_mode = request.rejected_completion;
                match self
                    .owner
                    .runtime
                    .StartUserScrollAnimation(&self.area, request, completion)
                {
                    Ok(()) => Consumption(accepted),
                    Err(completion) => {
                        completion(CompletionMode(rejected_mode));
                        Consumption(rejected)
                    }
                }
            }
        }
    }

    fn OnScrollFinished(&self, enqueue_scrollend: bool, emit: &ScrollEffectEmitter) {
        let Some(plan) = self
            .owner
            .layout
            .OnScrollFinished(&self.area, enqueue_scrollend)
        else {
            return;
        };
        self.owner.runtime.ApplyScrollFinished(&self.area, plan);
        if let Some(node_id) = plan.node_id {
            emit(ScrollEffect::ScrollFinished {
                node_id,
                scroll_element_id: plan.scroll_element_id,
                enqueue_scrollend: plan.enqueue_scrollend,
            });
        }
    }
}

impl RootFrameLayoutViewport for PageScrollableArea {
    fn Facts(&self) -> ScrollableAreaFacts {
        <Self as ScrollableArea>::Facts(self)
    }

    fn HasRunningUserScrollAnimation(&self) -> bool {
        self.owner
            .runtime
            .UserAnimationState(&self.area)
            .has_running_animation
    }

    fn WillConsumeUserScroll(&self, request: &UserScrollRequest) -> bool {
        self.owner
            .layout
            .WillConsumeUserScroll(
                &self.area,
                NativeGranularity(request.granularity),
                request.delta,
                self.owner.runtime.UserAnimationState(&self.area),
            )
            .unwrap_or(false)
    }

    fn SnapScroll(
        &self,
        kind: SnapScrollKind,
        direction: ScrollDirectionPhysical,
        emit: &ScrollEffectEmitter,
    ) -> bool {
        <Self as ScrollableArea>::SnapScroll(self, kind, direction, emit)
    }

    fn UserScroll(
        &self,
        request: UserScrollRequest,
        completion: ScrollCompletion,
        emit: &ScrollEffectEmitter,
    ) -> interaction_scroll::core::scroll::scroll_types::ScrollConsumption {
        <Self as ScrollableArea>::UserScroll(self, request, completion, emit)
    }

    fn OnScrollFinished(&self, enqueue_scrollend: bool, emit: &ScrollEffectEmitter) {
        <Self as ScrollableArea>::OnScrollFinished(self, enqueue_scrollend, emit)
    }
}

fn NativeGranularity(value: ScrollGranularity) -> NativeUserScrollGranularity {
    match value {
        ScrollGranularity::kScrollByPrecisePixel => NativeUserScrollGranularity::PrecisePixel,
        ScrollGranularity::kScrollByPixel => NativeUserScrollGranularity::Pixel,
        ScrollGranularity::kScrollByLine => NativeUserScrollGranularity::Line,
        ScrollGranularity::kScrollByPage => NativeUserScrollGranularity::Page,
        ScrollGranularity::kScrollByDocument => NativeUserScrollGranularity::Document,
    }
}

fn CompletionMode(value: NativeScrollCompletionMode) -> ScrollCompletionMode {
    match value {
        NativeScrollCompletionMode::Finished => ScrollCompletionMode::kFinished,
        NativeScrollCompletionMode::ZeroDelta => ScrollCompletionMode::kZeroDelta,
        NativeScrollCompletionMode::InterruptedByScroll => {
            ScrollCompletionMode::kInterruptedByScroll
        }
    }
}

fn Consumption(
    value: layoutng_assembly::native_user_scroll_source::NativeScrollConsumption,
) -> interaction_scroll::core::scroll::scroll_types::ScrollConsumption {
    interaction_scroll::core::scroll::scroll_types::ScrollConsumption::new(
        value.did_scroll_x,
        value.did_scroll_y,
        value.unused_scroll_delta_x,
        value.unused_scroll_delta_y,
    )
}

struct PageScrollFrame {
    owner: Rc<PageScrollOwner>,
}

impl ScrollFrame for PageScrollFrame {
    fn Facts(&self) -> ScrollFrameFacts {
        ScrollFrameFacts {
            focused_node: self
                .owner
                .frame
                .FocusedNodeId()
                .and_then(|id| self.owner.layout.LayoutForNode(id))
                .map(|layout| self.owner.Node(layout)),
            layout_view_node: self
                .owner
                .layout
                .LayoutView()
                .map(|layout| self.owner.Node(layout)),
            visual_viewport_is_active: self.owner.frame.VisualViewportIsActive(),
            mouse_pressed: self.owner.frame.MousePressed(),
        }
    }

    fn BubbleLogicalScrollInParentFrame(
        &self,
        direction: ScrollDirection,
        granularity: ScrollGranularity,
    ) -> bool {
        self.owner
            .frame
            .BubbleLogicalScrollInParentFrame(direction, granularity)
    }
}

struct PageScrollServices {
    owner: Rc<PageScrollOwner>,
}

impl ScrollFrameServices for PageScrollServices {
    fn NodeForId(&self, id: u64) -> Option<ScrollNodeRef> {
        self.owner
            .layout
            .LayoutForNode(id)
            .map(|layout| self.owner.Node(layout))
    }

    fn AutoscrollController(&self) -> Option<Rc<dyn ScrollAutoscrollController>> {
        self.owner.frame.AutoscrollController()
    }

    fn KeyboardEventManager(&self) -> Option<Rc<dyn ScrollKeyboardEventManager>> {
        self.owner.frame.KeyboardEventManager()
    }

    fn RespectOverscrollBehaviorForScrollBubbling(&self) -> bool {
        self.owner
            .frame
            .RespectOverscrollBehaviorForScrollBubbling()
    }

    fn ScrollEffects(&self) -> ScrollEffectEmitter {
        self.owner.effects.clone()
    }

    fn AssertScrollLayoutPrepared(&self, node: Option<&ScrollNodeRef>) {
        self.owner
            .frame
            .AssertLayoutPrepared(node.map(|node| node.Facts().dom_node_id));
    }
}

/// Result of the one-time EventHandler scroll-owner assembly.
pub struct PageScrollManagerOwners {
    manager: Rc<ScrollManager>,
    owner: Rc<PageScrollOwner>,
    layout_viewport_identity: usize,
}

impl PageScrollManagerOwners {
    pub fn new(
        layout: Rc<dyn PageLogicalScrollLayout>,
        runtime: Rc<dyn PageScrollRuntime>,
        frame: Rc<dyn PageScrollFrameState>,
        effects: ScrollEffectEmitter,
    ) -> Self {
        let owner = Rc::new(PageScrollOwner {
            layout,
            runtime,
            frame,
            root_frame_viewport: RefCell::new(None),
            effects,
            areas: RefCell::new(HashMap::new()),
            wheel_areas: RefCell::new(HashMap::new()),
        });
        let layout_viewport = owner
            .layout
            .LayoutViewportArea()
            .map(|area| owner.NativeArea(area))
            .expect("prepared local root owns a layout viewport ScrollableArea");
        let layout_viewport_identity = layout_viewport.area.Identity();
        let root = RootFrameViewport::new(layout_viewport);
        owner
            .root_frame_viewport
            .replace(Some(root.AsScrollableArea()));
        let scroll_frame: Rc<dyn ScrollFrame> = Rc::new(PageScrollFrame {
            owner: owner.clone(),
        });
        let services: Rc<dyn ScrollFrameServices> = Rc::new(PageScrollServices {
            owner: owner.clone(),
        });
        Self {
            manager: Rc::new(ScrollManager::new(scroll_frame, services)),
            owner,
            layout_viewport_identity,
        }
    }

    /// HitTestResult::GetScrollableContainer (hit_test_result.cc:304-314).
    pub fn ScrollableContainerForNode(
        &self,
        node_id: u64,
    ) -> Option<foundation::CompositorElementId> {
        self.owner
            .layout
            .WheelScrollableAreas(node_id)
            .next()
            .map(|area| area.GetScrollElementId())
    }

    pub fn LayoutViewportIdentity(&self) -> usize {
        self.layout_viewport_identity
    }

    /// This exact Rc must be passed to every EventHandler manager factory.
    pub fn ScrollManager(&self) -> Rc<ScrollManager> {
        self.manager.clone()
    }

    /// The frame and keyboard managers emit through the same Page-owned sink.
    // Source: scroll_manager.cc:310-311, keyboard_event_manager.cc:487,518.
    pub fn ScrollEffects(&self) -> ScrollEffectEmitter {
        self.owner.effects.clone()
    }

    /// Resolves the current live LayoutObject for a canonical DOM node. The
    /// no-layout wrapper preserves the Node identity needed by keyboard source
    /// ordering without fabricating a LayoutObject or copied layout tree.
    pub fn KeyboardScrollNode(&self, node_id: u64) -> ScrollNodeRef {
        self.owner
            .layout
            .LayoutForNode(node_id)
            .map(|layout| self.owner.Node(layout))
            .unwrap_or_else(|| Rc::new(PageNonLayoutScrollNode { node_id }))
    }

    pub fn WheelScrollableTraversal(
        &self,
        targets: Rc<dyn PageWheelTargetIdentity>,
    ) -> Rc<dyn WheelScrollableTraversal> {
        Rc::new(PageWheelScrollableTraversal {
            owner: self.owner.clone(),
            targets,
        })
    }

    /// Assembles the source EventHandler-owned wheel manager for the current
    /// local-root Page. Target hit testing/dispatch remains call-scoped; only
    /// target identity, phase latching and scrollbar fade state are retained.
    pub fn LocalRootWheelEventHandler(
        &self,
        targets: Rc<dyn PageWheelTargetIdentity>,
        mouse: Rc<crate::core::input::mouse_event_manager::MouseEventManagerState>,
        features: MouseWheelFeatures,
    ) -> Rc<EventHandlerScrollRouting> {
        let frame: Rc<dyn WheelFrame> = Rc::new(PageCallScopedLocalRootWheelFrame);
        let routing: Rc<dyn WheelTargetRouting> = Rc::new(PageLocalRootWheelTargetRouting);
        let context = MouseWheelEventManagerContext::CallScoped(
            routing,
            self.WheelScrollableTraversal(targets.clone()),
            features,
        );
        let mouse: Rc<dyn EventHandlerMouseScrollState> =
            Rc::new(PageEventHandlerMouseScrollState {
                owner: self.owner.clone(),
                targets,
                mouse,
            });
        let viewport: Rc<dyn EventHandlerViewportCoordinates> =
            Rc::new(PageLocalRootViewportCoordinates);
        Rc::new(EventHandlerScrollRouting::new(
            self.ScrollManager(),
            frame,
            context,
            mouse,
            viewport,
        ))
    }
}

struct PageCallScopedLocalRootWheelFrame;

impl WheelFrame for PageCallScopedLocalRootWheelFrame {
    fn Document(&self) -> Option<Rc<dyn WheelDocument>> {
        None
    }

    fn View(&self) -> Option<Rc<dyn WheelFrameView>> {
        None
    }

    fn PointerLockedElement(&self) -> Option<EventTargetRef> {
        // The current Browser Page has no PointerLockController owner.
        None
    }

    fn ParentLocalWheelRoute(&self) -> Option<Rc<dyn WheelTargetRoute>> {
        // Every Browser Page is an independent local root.
        None
    }
}

struct PageLocalRootWheelTargetRouting;

impl WheelTargetRouting for PageLocalRootWheelTargetRouting {
    fn SubframeForTargetNode(
        &self,
        _target: Option<&EventTargetRef>,
    ) -> Option<Rc<dyn WheelTargetRoute>> {
        // Rechrom does not yet embed child LocalFrames inside one Page domain.
        None
    }
}

struct PageEventHandlerMouseScrollState {
    owner: Rc<PageScrollOwner>,
    targets: Rc<dyn PageWheelTargetIdentity>,
    mouse: Rc<crate::core::input::mouse_event_manager::MouseEventManagerState>,
}

impl EventHandlerMouseScrollState for PageEventHandlerMouseScrollState {
    fn MousePressNode(&self) -> Option<ScrollNodeRef> {
        let target = self.mouse.MousePressNode()?;
        let node_id = self.targets.NodeId(&target)?;
        self.owner
            .layout
            .LayoutForNode(node_id)
            .map(|layout| self.owner.Node(layout))
    }

    fn LastKnownMousePositionInViewport(&self) -> foundation::gfx::PointF {
        self.mouse.LastKnownMousePositionInViewport()
    }

    fn LastKnownMouseScreenPosition(&self) -> foundation::gfx::PointF {
        self.mouse.LastKnownMouseScreenPosition()
    }

    fn InvalidateClick(&self) {
        self.mouse.InvalidateClick()
    }
}

struct PageLocalRootViewportCoordinates;

impl EventHandlerViewportCoordinates for PageLocalRootViewportCoordinates {
    fn ViewportToRootFrame(&self, point: foundation::gfx::PointF) -> foundation::gfx::PointF {
        point
    }
}

struct PageWheelScrollableArea {
    area: NativeWheelScrollableArea,
}

impl WheelScrollableArea for PageWheelScrollableArea {
    fn GetScrollElementId(&self) -> foundation::CompositorElementId {
        self.area.GetScrollElementId()
    }
    fn ScrollOffset(&self) -> (f64, f64) {
        self.area.ScrollOffset()
    }
    fn MinimumScrollOffset(&self) -> (f64, f64) {
        self.area.MinimumScrollOffset()
    }
    fn MaximumScrollOffset(&self) -> (f64, f64) {
        self.area.MaximumScrollOffset()
    }
    fn FadeInScrollbarIfExists(&self, horizontal: bool, vertical: bool) -> bool {
        self.area.FadeInScrollbarIfExists(horizontal, vertical)
    }
    fn FadeOutScrollbarIfNeeded(&self) {
        self.area.FadeOutScrollbarIfNeeded()
    }
}

struct PageWheelScrollableTraversal {
    owner: Rc<PageScrollOwner>,
    targets: Rc<dyn PageWheelTargetIdentity>,
}

impl WheelScrollableTraversal for PageWheelScrollableTraversal {
    fn ScrollableAreas<'a>(
        &'a self,
        target: Option<&EventTargetRef>,
    ) -> Box<dyn Iterator<Item = Rc<dyn WheelScrollableArea>> + 'a> {
        let Some(node_id) = target.and_then(|target| self.targets.NodeId(target)) else {
            return Box::new(std::iter::empty());
        };
        let owner = self.owner.clone();
        Box::new(
            self.owner
                .layout
                .WheelScrollableAreas(node_id)
                .map(move |area| owner.WheelArea(area)),
        )
    }
}
