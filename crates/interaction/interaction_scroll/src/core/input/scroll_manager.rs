// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/input/scroll_manager.h:1-120
// Source: third_party/blink/renderer/core/input/scroll_manager.cc:1-374
// Rust modules replace the includes/namespaces. Rc/Weak retain source Member
// and WeakPersistent identity; short RefCell borrows permit synchronous reentry.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use foundation::gfx::{Point, Transform, Vector2dF};
use foundation::style_constants::EOverscrollBehavior;
use interaction_input::public::common::input::web_input_event::Type;
use interaction_input::public::common::input::web_mouse_event::WebMouseEvent;
pub use interaction_input::ui::events::types::scroll_types::ScrollGranularity;

pub use crate::cc::trees::scroll_source_type::ScrollSourceType;
pub use crate::core::scroll::scroll_types::{
    ScrollConsumption, ScrollDirection, ScrollDirectionPhysical, ScrollOffset, ScrollbarOrientation,
};
use crate::core::scroll::scroll_types::{ToPhysicalDirection, ToScrollDelta};

// Source: scroll_manager.h:26-39
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollPropagationDirection {
    kHorizontal,
    kVertical,
    kBoth,
    kNone,
}
// Source: scroll_manager.cc:68-103. Shared by native LayoutBox adapters,
// including anonymous boxes that have no DOM scroll-node alias.
pub fn CanPropagateScroll(
    user_scrollable_horizontal: bool,
    user_scrollable_vertical: bool,
    overscroll_x: EOverscrollBehavior,
    overscroll_y: EOverscrollBehavior,
    direction: ScrollPropagationDirection,
) -> bool {
    if !user_scrollable_horizontal && !user_scrollable_vertical {
        return true;
    }
    let allows = |behavior| {
        matches!(
            behavior,
            EOverscrollBehavior::kAuto | EOverscrollBehavior::kChain
        )
    };
    match direction {
        ScrollPropagationDirection::kBoth => allows(overscroll_x) && allows(overscroll_y),
        ScrollPropagationDirection::kVertical => allows(overscroll_y),
        ScrollPropagationDirection::kHorizontal => allows(overscroll_x),
        ScrollPropagationDirection::kNone => true,
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogicalScrollResult {
    kScrolled,
    kContained,
    kBubbled,
}

pub type ScrollNodeRef = Rc<dyn ScrollNode>;
pub type ScrollLayoutBoxRef = Rc<dyn ScrollLayoutBox>;
pub type ScrollableAreaRef = Rc<dyn ScrollableArea>;
pub type ResizeScrollableAreaRef = Rc<dyn ResizeScrollableArea>;

// Source: scroll_manager.cc:109-151,159-198. These small live facts borrow the
// existing node/box/area owners. The containing-block chain is never copied.
pub struct ScrollNodeFacts {
    pub dom_node_id: u64,
    pub has_layout_object: bool,
    pub is_element: bool,
    pub is_effective_root_scroller: bool,
    pub document_is_in_main_frame: bool,
}
pub trait ScrollNode {
    fn Facts(&self) -> ScrollNodeFacts;
    fn EnclosingBox(&self) -> Option<ScrollLayoutBoxRef>;
    fn LayoutBox(&self) -> Option<ScrollLayoutBoxRef>;
    fn LayoutBoxForScrolling(&self) -> Option<ScrollLayoutBoxRef>;
}
pub struct ScrollLayoutBoxFacts {
    pub is_horizontal_writing_mode: bool,
    pub is_flipped_blocks_writing_mode: bool,
    pub is_global_root_scroller: bool,
    pub is_layout_view: bool,
    pub overscroll_behavior_x: EOverscrollBehavior,
    pub overscroll_behavior_y: EOverscrollBehavior,
}
pub trait ScrollLayoutBox {
    fn Facts(&self) -> ScrollLayoutBoxFacts;
    fn Node(&self) -> Option<ScrollNodeRef>;
    fn ContainingBlock(&self) -> Option<ScrollLayoutBoxRef>;
    fn ScrollableArea(&self) -> Option<ScrollableAreaRef>;
    // Source: scroll_manager.cc:241 (ScrollableArea::GetForScrolling).
    fn AreaForScrolling(&self) -> Option<ScrollableAreaRef>;
}

// Source: scroll_manager.cc:70-76,163,188-198,285-305.
#[derive(Clone, Copy, Debug)]
pub struct ScrollableAreaFacts {
    pub node_id: u64,
    pub scroll_element_id: foundation::CompositorElementId,
    pub user_input_scrollable_horizontal: bool,
    pub user_input_scrollable_vertical: bool,
    pub scrollable_horizontal: bool,
    pub scrollable_vertical: bool,
    pub is_root_frame_viewport: bool,
    pub scroll_affects_layout_viewport: bool,
    pub scroll_animator_enabled: bool,
}

// The receiving scroll owner computes and commits its own native scroll state.
// It emits only explicit offset results through this synchronous typed output;
// the manager never reads or retains a second scroll-offset store.
#[derive(Clone, Copy, Debug)]
pub struct ScrollOffsetMutation {
    pub node_id: u64,
    pub offset: ScrollOffset,
}
#[derive(Clone, Copy, Debug)]
pub enum ScrollEffect {
    /// The source UserScroll callback has completed on this exact area.
    AnimationCompleted {
        node_id: u64,
        scroll_element_id: foundation::CompositorElementId,
        source_type: ScrollSourceType,
        mode: ScrollCompletionMode,
    },
    SetOffset(ScrollOffsetMutation),
    ScrollFinished {
        node_id: u64,
        scroll_element_id: foundation::CompositorElementId,
        enqueue_scrollend: bool,
    },
}
pub type ScrollEffectEmitter = Rc<dyn Fn(ScrollEffect)>;

// Source: scrollable_area.h:97-104; scroll_manager.cc:275-314.
pub use crate::core::scroll::scrollable_area::ScrollCompletionMode;
pub type ScrollCompletion = Box<dyn FnOnce(ScrollCompletionMode)>;
#[derive(Clone, Copy, Debug)]
pub enum SnapScrollKind {
    Direction,
    Page,
    Document,
}
#[derive(Clone, Copy, Debug)]
pub struct UserScrollRequest {
    pub granularity: ScrollGranularity,
    pub delta: Vector2dF,
    pub source_type: ScrollSourceType,
}
pub trait ScrollableArea {
    fn Facts(&self) -> ScrollableAreaFacts;
    fn SnapScroll(
        &self,
        kind: SnapScrollKind,
        direction: ScrollDirectionPhysical,
        emit: &ScrollEffectEmitter,
    ) -> bool;
    fn UserScroll(
        &self,
        request: UserScrollRequest,
        completion: ScrollCompletion,
        emit: &ScrollEffectEmitter,
    ) -> ScrollConsumption;
    fn OnScrollFinished(&self, enqueue_scrollend: bool, emit: &ScrollEffectEmitter);
}

// Source: scroll_manager.cc:47-65,294-311. Each port is one existing owner;
// the keyboard manager retains the same weak scroll-area target until keyup.
pub trait ScrollKeyboardEventManager {
    fn HasPendingScrollendOnKeyUp(&self) -> bool;
    fn SetScrollendEventTarget(&self, area: &ScrollableAreaRef);
}
pub trait ScrollAutoscrollController {
    fn StopAutoscroll(&self);
    /// The current synchronous transaction supplies Node::StopAutoscroll's
    /// dispatch leaf; this callback is never retained by the scroll owner.
    fn StopAutoscrollWithNodeRelease(&self, _release: &dyn Fn(u64)) {
        self.StopAutoscroll();
    }
    fn StopMiddleClickAutoscroll(&self, frame: &Rc<dyn ScrollFrame>);
    fn MiddleClickAutoscrollInProgress(&self) -> bool;
}
pub struct ScrollFrameFacts {
    pub focused_node: Option<ScrollNodeRef>,
    pub layout_view_node: Option<ScrollNodeRef>,
    pub visual_viewport_is_active: bool,
    pub mouse_pressed: bool,
}
pub trait ScrollFrame {
    fn Facts(&self) -> ScrollFrameFacts;
    fn BubbleLogicalScrollInParentFrame(
        &self,
        direction: ScrollDirection,
        granularity: ScrollGranularity,
    ) -> bool;
}
pub trait ScrollFrameServices {
    fn NodeForId(&self, id: u64) -> Option<ScrollNodeRef>;
    fn AutoscrollController(&self) -> Option<Rc<dyn ScrollAutoscrollController>>;
    fn KeyboardEventManager(&self) -> Option<Rc<dyn ScrollKeyboardEventManager>>;
    fn RespectOverscrollBehaviorForScrollBubbling(&self) -> bool;
    fn ScrollEffects(&self) -> ScrollEffectEmitter;
    // Source: scroll_manager.cc:225,330-333. Page prepares the existing live
    // layout before entering this transaction. This is a readiness assertion,
    // not a callback that can trigger rendering lifecycle from this package.
    fn AssertScrollLayoutPrepared(&self, node: Option<&ScrollNodeRef>);
}

// Source: scroll_manager.cc:344-371. Resize remains on the original area.
pub trait ResizeScrollableArea {
    fn InResizeMode(&self) -> bool;
    fn SetInResizeMode(&self, value: bool);
    fn Resize(&self, position: Point, position_to_size: &Transform);
    fn InitializeResizeTransform(&self, position: Point) -> Transform;
}
pub trait ResizePaintLayer {
    fn ScrollableArea(&self) -> Option<ResizeScrollableAreaRef>;
}

// Source: scroll_manager.h:102-105.
struct ScrollChainResult {
    chain: VecDeque<u64>,
    can_bubble: bool,
}

// Source: scroll_manager.h:49-55,111-115.
// The absent Clone/Copy implementations express source deleted copying.
pub struct ScrollManager {
    frame_: Rc<dyn ScrollFrame>,
    services_: Rc<dyn ScrollFrameServices>,
    resize_scrollable_area_: RefCell<Option<ResizeScrollableAreaRef>>,
    resize_position_to_size_transform_: RefCell<Transform>,
}

impl ScrollManager {
    // Source: scroll_manager.cc:33-35.
    pub fn new(frame: Rc<dyn ScrollFrame>, services: Rc<dyn ScrollFrameServices>) -> Self {
        let result = Self {
            frame_: frame,
            services_: services,
            resize_scrollable_area_: RefCell::new(None),
            resize_position_to_size_transform_: RefCell::new(Transform::default()),
        };
        result.Clear();
        result
    }

    // Source: scroll_manager.h:57; scroll_manager.cc:37-40.
    pub fn Clear(&self) {
        *self.resize_scrollable_area_.borrow_mut() = None;
        *self.resize_position_to_size_transform_.borrow_mut() = Transform::default();
    }

    // Source: scroll_manager.h:54-55; scroll_manager.cc:42-45. Rc/Drop trace
    // and release the same frame and resize-area owners; no extra GC graph.

    // Source: scroll_manager.h:62; scroll_manager.cc:47-50.
    pub fn StopAutoscroll(&self) {
        if let Some(controller) = self.GetAutoscrollController() {
            controller.StopAutoscroll();
        }
    }

    pub fn StopAutoscrollWithNodeRelease(&self, release: &dyn Fn(u64)) {
        if let Some(controller) = self.GetAutoscrollController() {
            controller.StopAutoscrollWithNodeRelease(release);
        }
    }

    // Source: scroll_manager.h:60; scroll_manager.cc:52-55.
    pub fn StopMiddleClickAutoscroll(&self) {
        if let Some(controller) = self.GetAutoscrollController() {
            controller.StopMiddleClickAutoscroll(&self.frame_);
        }
    }

    // Source: scroll_manager.h:59; scroll_manager.cc:57-60.
    pub fn MiddleClickAutoscrollInProgress(&self) -> bool {
        self.GetAutoscrollController()
            .is_some_and(|controller| controller.MiddleClickAutoscrollInProgress())
    }

    // Source: scroll_manager.h:61; scroll_manager.cc:62-66.
    pub fn GetAutoscrollController(&self) -> Option<Rc<dyn ScrollAutoscrollController>> {
        self.services_.AutoscrollController()
    }

    // Source: scroll_manager.h:96-99; scroll_manager.cc:68-103.
    pub fn CanPropagate(
        layout_box: &dyn ScrollLayoutBox,
        direction: ScrollPropagationDirection,
    ) -> bool {
        let Some(area) = layout_box.ScrollableArea() else {
            return true;
        };
        let area = area.Facts();
        if !area.user_input_scrollable_horizontal && !area.user_input_scrollable_vertical {
            return true;
        }
        let style = layout_box.Facts();
        CanPropagateScroll(
            area.user_input_scrollable_horizontal,
            area.user_input_scrollable_vertical,
            style.overscroll_behavior_x,
            style.overscroll_behavior_y,
            direction,
        )
    }

    // Source: scroll_manager.h:106-108; scroll_manager.cc:105-155.
    fn RecomputeScrollChain(
        &self,
        start_node: &ScrollNodeRef,
        direction: ScrollDirection,
    ) -> ScrollChainResult {
        debug_assert!(start_node.Facts().has_layout_object);
        let mut current = start_node.EnclosingBox();
        let mut result = ScrollChainResult {
            chain: VecDeque::new(),
            can_bubble: true,
        };
        while let Some(layout_box) = current {
            if let Some(node) = layout_box.Node() {
                let style = layout_box.Facts();
                let physical = ToPhysicalDirection(
                    direction,
                    style.is_horizontal_writing_mode,
                    style.is_flipped_blocks_writing_mode,
                );
                let is_vertical = matches!(
                    physical,
                    ScrollDirectionPhysical::kScrollUp | ScrollDirectionPhysical::kScrollDown
                );
                let orientation = if is_vertical {
                    ScrollbarOrientation::kVerticalScrollbar
                } else {
                    ScrollbarOrientation::kHorizontalScrollbar
                };
                if self.CanScroll(&node, orientation) {
                    result.chain.push_front(node.Facts().dom_node_id);
                    if self.services_.RespectOverscrollBehaviorForScrollBubbling() {
                        let behavior = if is_vertical {
                            style.overscroll_behavior_y
                        } else {
                            style.overscroll_behavior_x
                        };
                        if !matches!(
                            behavior,
                            EOverscrollBehavior::kAuto | EOverscrollBehavior::kChain
                        ) {
                            result.can_bubble = false;
                            break;
                        }
                    }
                }
                if node.Facts().is_effective_root_scroller {
                    break;
                }
            }
            current = layout_box.ContainingBlock();
        }
        result
    }

    // Source: scroll_manager.h:109; scroll_manager.cc:157-199.
    fn CanScroll(&self, current_node: &ScrollNodeRef, orientation: ScrollbarOrientation) -> bool {
        let node_facts = current_node.Facts();
        let mut scrolling_box = current_node.LayoutBox();
        if node_facts.is_element {
            scrolling_box = current_node.LayoutBoxForScrolling().filter(|layout_box| {
                let area = layout_box
                    .ScrollableArea()
                    .expect("element scrolling box owns a scrollable area");
                let facts = area.Facts();
                facts.scrollable_horizontal || facts.scrollable_vertical
            });
        }
        let Some(scrolling_box) = scrolling_box else {
            return false;
        };
        let facts = scrolling_box.Facts();
        if facts.is_global_root_scroller {
            return true;
        }
        if facts.is_layout_view
            && node_facts.document_is_in_main_frame
            && self.frame_.Facts().visual_viewport_is_active
        {
            return true;
        }
        let Some(area) = scrolling_box.ScrollableArea() else {
            return false;
        };
        let area = area.Facts();
        match orientation {
            ScrollbarOrientation::kVerticalScrollbar => area.scrollable_vertical,
            ScrollbarOrientation::kHorizontalScrollbar => area.scrollable_horizontal,
        }
    }

    // Source: scroll_manager.h:64-78; scroll_manager.cc:201-323.
    pub fn LogicalScroll(
        &self,
        direction: ScrollDirection,
        granularity: ScrollGranularity,
        start_node: Option<ScrollNodeRef>,
        mouse_press_node: Option<ScrollNodeRef>,
        scrolling_via_key: bool,
    ) -> LogicalScrollResult {
        let mut node = start_node;
        if node.is_none() {
            node = self.frame_.Facts().focused_node;
        }
        if node.is_none() {
            node = mouse_press_node;
        }
        if node
            .as_ref()
            .is_none_or(|node| !node.Facts().has_layout_object)
        {
            if let Some(default_node) = self.frame_.Facts().layout_view_node {
                node = Some(default_node);
            }
        }
        let Some(node) = node else {
            return LogicalScrollResult::kBubbled;
        };
        // Source: scroll_manager.cc:223-225. The supplying Page has prepared
        // this document's live layout at the source synchronous query boundary.
        self.services_.AssertScrollLayoutPrepared(Some(&node));
        let mut chain_result = self.RecomputeScrollChain(&node, direction);
        while let Some(id) = chain_result.chain.pop_back() {
            let node = self
                .services_
                .NodeForId(id)
                .expect("scroll chain DOM id remains live");
            let layout_box = node
                .LayoutBox()
                .expect("scroll chain node owns a LayoutBox");
            let style = layout_box.Facts();
            let physical = ToPhysicalDirection(
                direction,
                style.is_horizontal_writing_mode,
                style.is_flipped_blocks_writing_mode,
            );
            let area = layout_box
                .AreaForScrolling()
                .expect("scroll chain box has its scrolling area");
            let emit = self.services_.ScrollEffects();
            // Source: scroll_manager.cc:244-273.
            let (snap_kind, source_type) = match granularity {
                ScrollGranularity::kScrollByLine => {
                    (SnapScrollKind::Direction, ScrollSourceType::kRelativeScroll)
                }
                ScrollGranularity::kScrollByPage => {
                    (SnapScrollKind::Page, ScrollSourceType::kRelativeScroll)
                }
                ScrollGranularity::kScrollByDocument => {
                    (SnapScrollKind::Document, ScrollSourceType::kAbsoluteScroll)
                }
                _ => unreachable!("LogicalScroll accepts line/page/document granularity"),
            };
            if area.SnapScroll(snap_kind, physical, &emit) {
                return LogicalScrollResult::kScrolled;
            }
            // Source: scroll_manager.cc:275-311. Both weak owners and the
            // same callback lifetime survive an asynchronous scroll animation.
            let weak_area = Rc::downgrade(&area);
            let weak_keyboard = self
                .services_
                .KeyboardEventManager()
                .map(|keyboard| Rc::downgrade(&keyboard));
            let completion_emit = emit.clone();
            let completion = Box::new(move |mode| {
                if let Some(area) = weak_area.upgrade() {
                    let mut enqueue_scrollend = mode == ScrollCompletionMode::kFinished;
                    let facts = area.Facts();
                    if enqueue_scrollend
                        && facts.is_root_frame_viewport
                        && !facts.scroll_affects_layout_viewport
                    {
                        enqueue_scrollend = false;
                    }
                    if scrolling_via_key && enqueue_scrollend {
                        if let Some(keyboard) = weak_keyboard
                            .as_ref()
                            .and_then(|keyboard| keyboard.upgrade())
                        {
                            if keyboard.HasPendingScrollendOnKeyUp()
                                || !area.Facts().scroll_animator_enabled
                            {
                                keyboard.SetScrollendEventTarget(&area);
                                enqueue_scrollend = false;
                            }
                        }
                    }
                    area.OnScrollFinished(enqueue_scrollend, &completion_emit);
                    // Source cc:281-311 callback lifetime, after OnScrollFinished
                    // effects. Do not infer completion from offset settling.
                    completion_emit(ScrollEffect::AnimationCompleted {
                        node_id: facts.node_id,
                        scroll_element_id: facts.scroll_element_id,
                        source_type,
                        mode,
                    });
                }
            });
            let consumption = area.UserScroll(
                UserScrollRequest {
                    granularity,
                    delta: ToScrollDelta(physical, 1.0),
                    source_type,
                },
                completion,
                &emit,
            );
            if consumption.DidScroll() {
                return LogicalScrollResult::kScrolled;
            }
        }
        if chain_result.can_bubble {
            LogicalScrollResult::kBubbled
        } else {
            LogicalScrollResult::kContained
        }
    }

    // Source: scroll_manager.h:80-86; scroll_manager.cc:325-342.
    pub fn BubblingScroll(
        &self,
        direction: ScrollDirection,
        granularity: ScrollGranularity,
        starting_node: Option<ScrollNodeRef>,
        mouse_press_node: Option<ScrollNodeRef>,
        scrolling_via_key: bool,
    ) -> bool {
        self.services_.AssertScrollLayoutPrepared(None);
        let result = self.LogicalScroll(
            direction,
            granularity,
            starting_node,
            mouse_press_node,
            scrolling_via_key,
        );
        if result != LogicalScrollResult::kBubbled {
            return true;
        }
        self.frame_
            .BubbleLogicalScrollInParentFrame(direction, granularity)
    }

    // Source: scroll_manager.h:88-90; scroll_manager.cc:344-356.
    pub fn InResizeMode(&self) -> bool {
        let area = self.resize_scrollable_area_.borrow().clone();
        area.is_some_and(|area| area.InResizeMode())
    }
    pub fn Resize(&self, event: &WebMouseEvent) {
        if event.GetType() == Type::kMouseMove {
            if !self.frame_.Facts().mouse_pressed {
                return;
            }
            let area = self
                .resize_scrollable_area_
                .borrow()
                .clone()
                .expect("Resize requires a live resize scrollable area");
            let transform = self.resize_position_to_size_transform_.borrow().clone();
            let point = event.PositionInRootFrame();
            // Source: scroll_manager.cc:353; ui/gfx/geometry/point_conversions.cc
            // ToFlooredPoint uses ClampFloor, including out-of-range saturation.
            let floor = |value: f32| value.floor() as i32;
            area.Resize(Point::new(floor(point.x()), floor(point.y())), &transform);
        }
    }

    // Source: scroll_manager.h:91-94; scroll_manager.cc:358-372.
    pub fn ClearResizeScrollableArea(&self, should_not_be_null: bool) {
        let area = self.resize_scrollable_area_.borrow().clone();
        if should_not_be_null {
            debug_assert!(area.is_some());
        }
        if let Some(area) = area {
            area.SetInResizeMode(false);
        }
        *self.resize_scrollable_area_.borrow_mut() = None;
    }
    pub fn SetResizeScrollableArea(&self, layer: &dyn ResizePaintLayer, position: Point) {
        let area = layer
            .ScrollableArea()
            .expect("resize layer owns a scrollable area");
        *self.resize_scrollable_area_.borrow_mut() = Some(area.clone());
        area.SetInResizeMode(true);
        let transform = area.InitializeResizeTransform(position);
        *self.resize_position_to_size_transform_.borrow_mut() = transform;
    }
}
