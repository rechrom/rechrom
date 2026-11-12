//! Chromium 6c1d401f: paint/timing/paint_timing_record.h:31-243,
//! .cc:18-102; timing/performance_entry.h:52-59. Records retain the original
//! weak Node and native Layout identity, never a DOM/Layout geometry copy.
use super::soft_navigation_context::SoftNavigationContext;
use foundation::gfx;
use interaction_dispatch::core::dom::events::event::{EventTarget, EventTargetRef};
use interaction_input::base::time::time::TimeTicks;
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DOMPaintTimingInfo {
    pub paint_time: f64,
    pub presentation_time: f64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PaintTimingLayoutIdentity(pub u64);
pub use image_resource::media_timing::{MediaTiming, RequestPriority, ResourceLoadTimings};
#[derive(Clone, Copy, Debug)]
pub struct EffectiveVisualSizeResult {
    pub size: u64,
    pub entropy: f64,
    pub is_min_entropy_met: bool,
    pub is_viewport_covered: bool,
}
impl Default for EffectiveVisualSizeResult {
    fn default() -> Self {
        Self {
            size: 0,
            entropy: 0.0,
            is_min_entropy_met: true,
            is_viewport_covered: false,
        }
    }
}
pub enum PaintTimingRecordKind {
    Text {
        effective_visual_size: u64,
        element_timing_rect: gfx::RectF,
        frame_index: Cell<u32>,
        needed_for_element_timing: Cell<bool>,
    },
    Image {
        media_timing: Weak<dyn MediaTiming>,
        hash: u64,
        effective_visual_size: EffectiveVisualSizeResult,
        load_time: Cell<TimeTicks>,
        first_animated_frame_time: Cell<TimeTicks>,
        sufficiently_loaded: Cell<bool>,
    },
}
pub struct PaintTimingRecord {
    node: Weak<dyn EventTarget>,
    layout_identity: PaintTimingLayoutIdentity,
    frame_visual_rect: gfx::Rect,
    root_visual_rect: gfx::RectF,
    needed_for_lcp: Cell<bool>,
    paint_time: Cell<TimeTicks>,
    paint_timing_info: Cell<DOMPaintTimingInfo>,
    soft_navigation_context: RefCell<Option<Rc<SoftNavigationContext>>>,
    pub(crate) kind: PaintTimingRecordKind,
}
impl PaintTimingRecord {
    fn new(
        node: &EventTargetRef,
        layout_identity: PaintTimingLayoutIdentity,
        frame_visual_rect: gfx::Rect,
        root_visual_rect: gfx::RectF,
        kind: PaintTimingRecordKind,
    ) -> Rc<Self> {
        Rc::new(Self {
            node: Rc::downgrade(node),
            layout_identity,
            frame_visual_rect,
            root_visual_rect,
            needed_for_lcp: Cell::new(false),
            paint_time: Cell::new(TimeTicks::new()),
            paint_timing_info: Cell::new(Default::default()),
            soft_navigation_context: RefCell::new(None),
            kind,
        })
    }
    pub fn NewText(
        node: &EventTargetRef,
        layout: PaintTimingLayoutIdentity,
        size: u64,
        element: gfx::RectF,
        frame: gfx::Rect,
        root: gfx::RectF,
    ) -> Rc<Self> {
        Self::new(
            node,
            layout,
            frame,
            root,
            PaintTimingRecordKind::Text {
                effective_visual_size: size,
                element_timing_rect: element,
                frame_index: Cell::new(0),
                needed_for_element_timing: Cell::new(false),
            },
        )
    }
    pub fn NewImage(
        node: &EventTargetRef,
        layout: PaintTimingLayoutIdentity,
        media: &Rc<dyn MediaTiming>,
        frame: gfx::Rect,
        root: gfx::RectF,
        hash: u64,
        size: EffectiveVisualSizeResult,
    ) -> Rc<Self> {
        assert!(size.size > 0);
        Self::new(
            node,
            layout,
            frame,
            root,
            PaintTimingRecordKind::Image {
                media_timing: Rc::downgrade(media),
                hash,
                effective_visual_size: size,
                load_time: Cell::new(TimeTicks::new()),
                first_animated_frame_time: Cell::new(TimeTicks::new()),
                sufficiently_loaded: Cell::new(false),
            },
        )
    }
    pub fn GetNode(&self) -> Option<EventTargetRef> {
        self.node.upgrade()
    }
    pub fn LayoutIdentity(&self) -> PaintTimingLayoutIdentity {
        self.layout_identity
    }
    pub fn RootVisualRect(&self) -> &gfx::RectF {
        &self.root_visual_rect
    }
    pub fn FrameVisualRect(&self) -> &gfx::Rect {
        &self.frame_visual_rect
    }
    pub fn IsImageRecord(&self) -> bool {
        matches!(self.kind, PaintTimingRecordKind::Image { .. })
    }
    pub fn IsTextRecord(&self) -> bool {
        matches!(self.kind, PaintTimingRecordKind::Text { .. })
    }
    pub fn EffectiveVisualSize(&self) -> u64 {
        match &self.kind {
            PaintTimingRecordKind::Text {
                effective_visual_size,
                ..
            } => *effective_visual_size,
            PaintTimingRecordKind::Image {
                effective_visual_size,
                ..
            } => effective_visual_size.size,
        }
    }
    pub fn HasPaintTime(&self) -> bool {
        !self.paint_time.get().is_null()
    }
    pub fn PaintTime(&self) -> TimeTicks {
        self.paint_time.get()
    }
    pub fn PaintTimingInfo(&self) -> DOMPaintTimingInfo {
        self.paint_timing_info.get()
    }
    pub fn SetPaintTime(&self, time: TimeTicks, info: DOMPaintTimingInfo) {
        self.paint_time.set(time);
        self.paint_timing_info.set(info);
    }
    pub fn GetSoftNavigationContext(&self) -> Option<Rc<SoftNavigationContext>> {
        self.soft_navigation_context.borrow().clone()
    }
    pub fn SetSoftNavigationContext(&self, context: Option<Rc<SoftNavigationContext>>) {
        *self.soft_navigation_context.borrow_mut() = context;
    }
    pub fn IsNeededForInteractionContentfulPaint(&self) -> bool {
        self.soft_navigation_context.borrow().is_some()
    }
    pub fn IsNeededForLargestContentfulPaint(&self) -> bool {
        self.needed_for_lcp.get()
    }
    pub fn SetIsNeededForLargestContentfulPaint(&self, value: bool) {
        self.needed_for_lcp.set(value);
    }
    pub fn IsEffectiveSizeLargerThan(&self, other: Option<&Self>) -> bool {
        self.EffectiveVisualSize() > other.map_or(0, Self::EffectiveVisualSize)
    }
    pub fn WasNodeRemoved(&self, context: &dyn PaintTimingNodeContext) -> bool {
        self.GetNode()
            .is_none_or(|node| context.CurrentLayoutIdentity(&node) != Some(self.layout_identity))
    }
    pub fn FrameIndex(&self) -> u32 {
        match &self.kind {
            PaintTimingRecordKind::Text { frame_index, .. } => frame_index.get(),
            _ => panic!("TextRecord"),
        }
    }
    pub fn SetFrameIndex(&self, index: u32) {
        match &self.kind {
            PaintTimingRecordKind::Text { frame_index, .. } => frame_index.set(index),
            _ => panic!("TextRecord"),
        }
    }
    pub fn IsNeededForElementTiming(&self) -> bool {
        match &self.kind {
            PaintTimingRecordKind::Text {
                needed_for_element_timing,
                ..
            } => needed_for_element_timing.get(),
            _ => false,
        }
    }
    pub fn SetIsNeededForElementTiming(&self, value: bool) {
        match &self.kind {
            PaintTimingRecordKind::Text {
                needed_for_element_timing,
                ..
            } => needed_for_element_timing.set(value),
            _ => panic!("TextRecord"),
        }
    }
    pub fn ElementTimingRect(&self) -> &gfx::RectF {
        match &self.kind {
            PaintTimingRecordKind::Text {
                element_timing_rect,
                ..
            } => element_timing_rect,
            _ => panic!("TextRecord"),
        }
    }
    pub fn GetMediaTiming(&self) -> Option<Rc<dyn MediaTiming>> {
        match &self.kind {
            PaintTimingRecordKind::Image { media_timing, .. } => media_timing.upgrade(),
            _ => None,
        }
    }
    pub fn GetEffectiveVisualSizeResult(&self) -> &EffectiveVisualSizeResult {
        match &self.kind {
            PaintTimingRecordKind::Image {
                effective_visual_size,
                ..
            } => effective_visual_size,
            _ => panic!("ImageRecord"),
        }
    }
    pub fn EntropyForLCP(&self) -> f64 {
        self.GetEffectiveVisualSizeResult().entropy
    }
    pub fn RequestPriority(&self) -> Option<RequestPriority> {
        self.GetMediaTiming()
            .and_then(|media| media.RequestPriority())
    }
    pub fn IsSufficientlyLoadedForReporting(&self) -> bool {
        match &self.kind {
            PaintTimingRecordKind::Image {
                sufficiently_loaded,
                ..
            } => sufficiently_loaded.get(),
            _ => panic!("ImageRecord"),
        }
    }
    pub fn SetIsSufficientlyLoadedForReporting(&self) {
        match &self.kind {
            PaintTimingRecordKind::Image {
                sufficiently_loaded,
                ..
            } => sufficiently_loaded.set(true),
            _ => panic!("ImageRecord"),
        }
    }
    pub fn LoadTime(&self) -> TimeTicks {
        match &self.kind {
            PaintTimingRecordKind::Image { load_time, .. } => load_time.get(),
            _ => panic!("ImageRecord"),
        }
    }
    pub fn SetLoadTime(&self, time: TimeTicks) {
        match &self.kind {
            PaintTimingRecordKind::Image { load_time, .. } => load_time.set(time),
            _ => panic!("ImageRecord"),
        }
    }
    pub fn FirstAnimatedFrameTime(&self) -> TimeTicks {
        match &self.kind {
            PaintTimingRecordKind::Image {
                first_animated_frame_time,
                ..
            } => first_animated_frame_time.get(),
            _ => panic!("ImageRecord"),
        }
    }
    pub fn HasFirstAnimatedFrameTime(&self) -> bool {
        !self.FirstAnimatedFrameTime().is_null()
    }
    pub fn SetFirstAnimatedFrameTime(&self, time: TimeTicks) {
        match &self.kind {
            PaintTimingRecordKind::Image {
                first_animated_frame_time,
                ..
            } => first_animated_frame_time.set(time),
            _ => panic!("ImageRecord"),
        }
    }
    pub fn Hash(&self) -> u64 {
        match &self.kind {
            PaintTimingRecordKind::Image { hash, .. } => *hash,
            _ => panic!("ImageRecord"),
        }
    }
}
/// Read the same native LayoutObject's lifetime identity. A reattach gets a
/// different identity even if an allocator reuses its previous address.
pub trait PaintTimingNodeContext {
    fn CurrentLayoutIdentity(&self, node: &EventTargetRef) -> Option<PaintTimingLayoutIdentity>;
}
