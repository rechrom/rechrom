//! Resident LayoutObject identity used by Blink's AutoscrollController.
//! Handles root the original object; each query borrows its owning engine and
//! enters the same LayoutPassScope as layout/scroll. No native tree is copied.
use crate::{
    internal::{
        layout_box::LayoutBox, layout_object::LayoutObject, layout_pass_scope::LayoutPassScope,
        scrollbar_orientation::ScrollbarOrientation,
    },
    layout_engine::LayoutEngine,
    native_scroll_source::{NativeScrollFrameGeometry, NativeScrollResult},
};
use foundation::{gfx, DynamicTo, LayoutUnit, Persistent, PhysicalOffset};

pub struct NativeAutoscrollLayout {
    object: Persistent<LayoutObject>,
}
pub enum NativeAutoscrollSearch {
    Found(NativeAutoscrollLayout),
    ParentFrame,
    NotFound,
}
impl Clone for NativeAutoscrollLayout {
    fn clone(&self) -> Self {
        Self {
            object: Persistent::from_ptr(self.object.Get()),
        }
    }
}
impl NativeAutoscrollLayout {
    fn FromObject(pointer: *mut LayoutObject) -> Option<Self> {
        (!pointer.is_null()).then(|| Self {
            object: Persistent::from_ptr(pointer),
        })
    }
    pub fn Identity(&self) -> usize {
        self.object.Get() as usize
    }
}
pub struct NativeAutoscrollSource<'a> {
    _scope: LayoutPassScope,
    _engine: &'a mut LayoutEngine,
    object: &'a NativeAutoscrollLayout,
    frame: NativeScrollFrameGeometry,
}
impl LayoutEngine {
    pub fn AutoscrollLayoutFor(&self, node_id: u64) -> Option<NativeAutoscrollLayout> {
        NativeAutoscrollLayout::FromObject(
            self.GetLayoutTree()?.OutputNode(node_id)? as *const LayoutObject as *mut LayoutObject
        )
    }
    /// Logical scrolling and autoscroll retain the same source LayoutObject.
    /// This alias documents the owner boundary without publishing a second
    /// node or containing-block tree.
    // Source: scroll_manager.cc:105-109,201-225.
    pub fn LogicalScrollLayoutFor(&self, node_id: u64) -> Option<NativeAutoscrollLayout> {
        self.AutoscrollLayoutFor(node_id)
    }
    /// Source fallback when neither focus nor mouse press retains layout.
    // Source: scroll_manager.cc:215-217.
    pub fn LogicalScrollLayoutView(&self) -> Option<NativeAutoscrollLayout> {
        NativeAutoscrollLayout::FromObject(
            self.GetLayoutTree()?.Root() as *const LayoutObject as *mut LayoutObject
        )
    }
    pub fn NativeAutoscrollSource<'a>(
        &'a mut self,
        object: &'a NativeAutoscrollLayout,
        frame: NativeScrollFrameGeometry,
    ) -> Option<NativeAutoscrollSource<'a>> {
        if unsafe { &*object.object.Get() }.WasDestroyedForTreeUpdate() {
            return None;
        }
        let scope = self.NativeScrollPassScope();
        Some(NativeAutoscrollSource {
            _scope: scope,
            _engine: self,
            object,
            frame,
        })
    }
}

/// Immutable facts read from the retained source Node/LayoutObject pair.
/// Anonymous boxes return no node facts and remain traversable as boxes.
// Source: scroll_manager.cc:105-198,201-225.
#[derive(Clone, Copy, Debug)]
pub struct NativeLogicalScrollNodeFacts {
    pub node_id: u64,
    pub is_element: bool,
    pub is_effective_root_scroller: bool,
    pub document_is_in_main_frame: bool,
}
impl NativeAutoscrollSource<'_> {
    fn Object(&self) -> &LayoutObject {
        unsafe { &*self.object.object.Get() }
    }
    fn Box(&self) -> Option<&LayoutBox> {
        unsafe { DynamicTo::<LayoutBox>(self.object.object.Get()).as_ref() }
    }
    pub fn HasViewport(&self) -> bool {
        self.Object()
            .InputOwnerForLayout()
            .InputViewport()
            .is_some()
    }
    pub fn IsBox(&self) -> bool {
        self.Object().IsBox()
    }
    pub fn NodeId(&self) -> Option<u64> {
        Some(unsafe { self.Object().GetNode().as_ref()? }.InputId())
    }
    pub fn IsDocumentNode(&self) -> bool {
        unsafe { self.Object().GetNode().as_ref() }.is_some_and(|node| node.IsDocumentNode())
    }
    pub fn Parent(&self) -> Option<NativeAutoscrollLayout> {
        NativeAutoscrollLayout::FromObject(self.Object().Parent())
    }
    pub fn FindAutoscrollable(&self, middle_click: bool) -> Option<NativeAutoscrollLayout> {
        NativeAutoscrollLayout::FromObject(
            LayoutBox::FindAutoscrollable(self.object.object.Get(), middle_click).cast(),
        )
    }
    pub fn FindAutoscrollableInFrame(&self, middle_click: bool) -> NativeAutoscrollSearch {
        let (object, parent_frame) =
            LayoutBox::FindAutoscrollableInFrame(self.object.object.Get(), middle_click);
        if let Some(object) = NativeAutoscrollLayout::FromObject(object.cast()) {
            NativeAutoscrollSearch::Found(object)
        } else if parent_frame {
            NativeAutoscrollSearch::ParentFrame
        } else {
            NativeAutoscrollSearch::NotFound
        }
    }
    pub fn HasScrollableOverflow(&self, horizontal: bool) -> bool {
        self.Box().is_some_and(|box_| {
            if horizontal {
                box_.HasScrollableOverflowX()
            } else {
                box_.HasScrollableOverflowY()
            }
        })
    }
    pub fn IsUserScrollable(&self) -> bool {
        self.Box().is_some_and(|box_| box_.IsUserScrollable())
    }
    pub fn PropagationFacts(
        &self,
    ) -> (
        bool,
        bool,
        foundation::style_constants::EOverscrollBehavior,
        foundation::style_constants::EOverscrollBehavior,
    ) {
        let area = self
            .Box()
            .and_then(|box_| unsafe { box_.GetScrollableArea().as_ref() });
        let horizontal = area.is_some_and(|area| {
            area.UserInputScrollable(ScrollbarOrientation::kHorizontalScrollbar)
        });
        let vertical = area
            .is_some_and(|area| area.UserInputScrollable(ScrollbarOrientation::kVerticalScrollbar));
        let style = self.Object().StyleRef();
        (
            horizontal,
            vertical,
            style.OverscrollBehaviorX(),
            style.OverscrollBehaviorY(),
        )
    }
    // Source: layout_box.cc:1627-1684. A Page's local root conversion is the
    // supplied frame translation; the native box computes the scroll belt.
    pub fn CalculateAutoscrollDirection(&self, point: (f64, f64)) -> (f64, f64) {
        let Some(box_) = self.Box() else {
            return (0.0, 0.0);
        };
        let offset = box_.CalculateAutoscrollDirectionInRootFrame(
            &gfx::PointF::new(point.0 as f32, point.1 as f32),
            PhysicalOffset::new(
                LayoutUnit::from_f64(self.frame.document_to_root_frame.x),
                LayoutUnit::from_f64(self.frame.document_to_root_frame.y),
            ),
        );
        (offset.left.ToDouble(), offset.top.ToDouble())
    }
    pub fn ScrollingBoxRect(&self) -> Option<gfx::RectF> {
        let rect = self.Box()?.AutoscrollScrollingBox()?;
        Some(gfx::RectF::new(
            gfx::PointF::new(rect.X().ToFloat(), rect.Y().ToFloat()),
            gfx::SizeF::new(rect.Width().ToFloat(), rect.Height().ToFloat()),
        ))
    }
    pub fn CalculateAutoscrollDirectionForRootRect(
        &self,
        point: (f64, f64),
        rect: gfx::RectF,
    ) -> (f64, f64) {
        let Some(box_) = self.Box() else {
            return (0.0, 0.0);
        };
        let rect = foundation::PhysicalRect::new(
            PhysicalOffset::new(
                LayoutUnit::FromFloatRound(rect.x()),
                LayoutUnit::FromFloatRound(rect.y()),
            ),
            foundation::PhysicalSize::new(
                LayoutUnit::FromFloatRound(rect.width()),
                LayoutUnit::FromFloatRound(rect.height()),
            ),
        );
        let delta = box_.CalculateAutoscrollDirectionForRootRect(
            &gfx::PointF::new(point.0 as f32, point.1 as f32),
            rect,
        );
        (delta.left.ToDouble(), delta.top.ToDouble())
    }
    pub fn Autoscroll(&self, point: (f64, f64)) -> NativeScrollResult {
        let Some(box_) = self.Box() else {
            return NativeScrollResult::default();
        };
        box_.AutoscrollWithResult(&PhysicalOffset::new(
            LayoutUnit::from_f64(point.0 - self.frame.document_to_root_frame.x),
            LayoutUnit::from_f64(point.1 - self.frame.document_to_root_frame.y),
        ))
    }
}

/// Minimal immutable facts from the same source LayoutBox used by logical
/// scrolling. Styles and the native object itself remain inside Layout.
pub struct NativeLogicalScrollBoxFacts {
    pub is_horizontal_writing_mode: bool,
    pub is_flipped_blocks_writing_mode: bool,
    pub is_global_root_scroller: bool,
    pub is_layout_view: bool,
    pub overscroll_behavior_x: foundation::style_constants::EOverscrollBehavior,
    pub overscroll_behavior_y: foundation::style_constants::EOverscrollBehavior,
}
pub struct NativeLogicalScrollArea {
    area: Persistent<crate::internal::layout_scrollable_area::PaintLayerScrollableArea>,
}
impl Clone for NativeLogicalScrollArea {
    fn clone(&self) -> Self {
        Self {
            area: Persistent::from_ptr(self.area.Get()),
        }
    }
}
impl NativeLogicalScrollArea {
    pub(crate) fn FromPointer(
        area: *mut crate::internal::layout_scrollable_area::PaintLayerScrollableArea,
    ) -> Option<Self> {
        (!area.is_null()).then(|| Self {
            area: Persistent::from_ptr(area),
        })
    }
    pub fn Identity(&self) -> usize {
        self.area.Get() as usize
    }
    pub(crate) fn Area(
        &self,
    ) -> &crate::internal::layout_scrollable_area::PaintLayerScrollableArea {
        unsafe { &*self.area.Get() }
    }
    pub(crate) fn AreaPointer(
        &self,
    ) -> *mut crate::internal::layout_scrollable_area::PaintLayerScrollableArea {
        self.area.Get()
    }
    pub fn GetScrollElementId(&self) -> foundation::CompositorElementId {
        unsafe { &*self.area.Get() }.GetScrollElementId()
    }
}
pub enum NativeLogicalScrollingArea {
    Area(NativeLogicalScrollArea),
    RootFrame,
}
impl NativeAutoscrollSource<'_> {
    pub fn IsEffectiveRootScroller(&self) -> bool {
        self.Object().IsEffectiveRootScroller()
    }
    pub fn LogicalScrollNodeFacts(&self) -> Option<NativeLogicalScrollNodeFacts> {
        let object = self.Object();
        let node = unsafe { object.GetNode().as_ref()? };
        Some(NativeLogicalScrollNodeFacts {
            node_id: node.InputId(),
            is_element: node.IsElementNode(),
            is_effective_root_scroller: object.IsEffectiveRootScroller(),
            document_is_in_main_frame: object.ViewportGeometryForLayout().is_main_frame,
        })
    }
    /// Node::GetLayoutBox on the same retained native Node.
    pub fn LogicalLayoutBox(&self) -> Option<NativeAutoscrollLayout> {
        let node = unsafe { self.Object().GetNode().as_ref()? };
        NativeAutoscrollLayout::FromObject(node.GetLayoutBox().cast())
    }
    /// LayoutBox::GetNode resolved back to that Node's original LayoutObject.
    pub fn LogicalNode(&self) -> Option<NativeAutoscrollLayout> {
        let node = unsafe { self.Box()?.GetNode().as_ref()? };
        NativeAutoscrollLayout::FromObject(node.GetLayoutObject())
    }
    pub fn EnclosingBox(&self) -> Option<NativeAutoscrollLayout> {
        NativeAutoscrollLayout::FromObject(self.Object().EnclosingBox().cast())
    }
    pub fn ContainingBlock(&self) -> Option<NativeAutoscrollLayout> {
        NativeAutoscrollLayout::FromObject(self.Object().ContainingBlock().cast())
    }
    /// Source: ContainerNode::GetLayoutBoxForScrolling, container_node.cc:915-921.
    pub fn LayoutBoxForScrolling(&self) -> Option<NativeAutoscrollLayout> {
        let pointer = DynamicTo::<LayoutBox>(self.object.object.Get());
        let content = unsafe { pointer.as_mut()? }.ContentLayoutBox();
        let box_ = unsafe { content.as_ref()? };
        if box_.GetScrollableArea().is_null() {
            return None;
        }
        NativeAutoscrollLayout::FromObject(content.cast())
    }
    pub fn LogicalScrollBoxFacts(&self) -> NativeLogicalScrollBoxFacts {
        let box_ = self
            .Box()
            .expect("logical scroll query uses a source LayoutBox");
        let style = box_.StyleRef();
        NativeLogicalScrollBoxFacts {
            is_horizontal_writing_mode: box_.IsHorizontalWritingMode(),
            is_flipped_blocks_writing_mode: box_.HasFlippedBlocksWritingMode(),
            is_global_root_scroller: box_.IsGlobalRootScroller(),
            is_layout_view: box_.IsLayoutView(),
            overscroll_behavior_x: style.OverscrollBehaviorX(),
            overscroll_behavior_y: style.OverscrollBehaviorY(),
        }
    }
    pub fn LogicalScrollableArea(&self) -> Option<NativeLogicalScrollArea> {
        NativeLogicalScrollArea::FromPointer(self.Box()?.GetScrollableArea())
    }
    /// Source: ScrollableArea::GetForScrolling, scrollable_area.cc:1404-1421.
    pub fn LogicalAreaForScrolling(&self) -> Option<NativeLogicalScrollingArea> {
        let box_ = self.Box()?;
        if box_.IsGlobalRootScroller() {
            return Some(NativeLogicalScrollingArea::RootFrame);
        }
        if unsafe { box_.GetNode().as_ref() }.is_some_and(|node| node.IsElementNode()) {
            if let Some(content) = self.LayoutBoxForScrolling() {
                let content = unsafe { &*DynamicTo::<LayoutBox>(content.object.Get()) };
                return NativeLogicalScrollArea::FromPointer(content.GetScrollableArea())
                    .map(NativeLogicalScrollingArea::Area);
            }
        }
        self.LogicalScrollableArea()
            .map(NativeLogicalScrollingArea::Area)
    }
}
