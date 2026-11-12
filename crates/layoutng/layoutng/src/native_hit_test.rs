//! Borrow the resident owner selected by Chromium's NodeAtPoint virtual.
//! Source: layout_object.cc:4524-4544,4602-4610; layout_block.cc:378-403;
//! layout_inline.cc:597-697. Algorithms remain in Interaction over this cursor.
use super::PaintLayoutObjectIdentity;
#[path = "native_svg_hit_test.rs"]
mod svg_hit_test;
use crate::{
    fragment_item::FragmentItem,
    inline_cursor::InlineCursor,
    internal::{
        layout_box::LayoutBox,
        layout_box_model_object::LayoutBoxModelObject,
        layout_object::{LayoutObject, LayoutObjectClass},
    },
    physical_box_fragment::PhysicalBoxFragment,
    physical_fragment::PhysicalFragment,
};
use foundation::{
    layout_hit_test::LayoutHitTestKind, DynamicTo, EPointerEvents, EVisibility, PhysicalOffset,
    PhysicalRect,
};
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;
use layoutng_style::style::computed_style::ComputedStyle;
#[derive(Clone, Copy)]
pub struct NativeHitTestCursor<'a> {
    object: &'a LayoutObject,
}
pub enum NativeHitTestTarget<'a> {
    Base(NativeHitTestCursor<'a>),
    Hidden(NativeHitTestCursor<'a>),
    Physical {
        source: NativeHitTestCursor<'a>,
        kind: LayoutHitTestKind,
        fragment: Option<&'a PhysicalBoxFragment>,
    },
    Inline(NativeHitTestCursor<'a>),
    Svg {
        source: NativeHitTestCursor<'a>,
        kind: LayoutHitTestKind,
    },
}
impl<'a> NativeHitTestCursor<'a> {
    pub fn ForObject(object: &'a LayoutObject) -> Self {
        object.CheckIsNotDestroyed();
        Self { object }
    }
    pub fn ForFragment(fragment: &'a PhysicalFragment) -> Option<Self> {
        if fragment.IsLayoutObjectDestroyedOrMoved() {
            return None;
        }
        unsafe { fragment.GetLayoutObject().as_ref() }.map(Self::ForObject)
    }
    pub fn SvgObject(self) -> Option<&'a LayoutObject> {
        self.object.IsSVG().then_some(self.object)
    }
    pub fn Identity(self) -> usize {
        self.object as *const _ as usize
    }
    pub fn PaintIdentity(self) -> PaintLayoutObjectIdentity<'a> {
        PaintLayoutObjectIdentity {
            object: self.object,
        }
    }
    pub fn Style(self) -> &'a ComputedStyle {
        self.object.StyleRef()
    }
    pub fn NodeId(self) -> Option<u64> {
        unsafe { self.object.GetNode().as_ref() }
            .map(|n| n.InputHitTestNodeOwner().unwrap_or_else(|| n.InputId()))
    }
    pub fn NodeForHitTestId(self) -> Option<u64> {
        unsafe { self.object.NodeForHitTest().as_ref() }
            .map(|n| n.InputHitTestNodeOwner().unwrap_or_else(|| n.InputId()))
    }
    pub fn NeedsLayout(self) -> bool {
        self.object.NeedsLayout()
    }
    pub fn CanTraversePhysicalFragments(self) -> bool {
        self.object.CanTraversePhysicalFragments()
    }
    pub fn IsFragmented(self) -> bool {
        self.object.IsFragmented()
    }
    pub fn IsInLayoutNGInlineFormattingContext(self) -> bool {
        self.object.IsInLayoutNGInlineFormattingContext()
    }
    pub fn VisibleToHitTestRequest(self, ignore_pointer_events_none: bool) -> bool {
        let style = self.Style();
        style.Visibility() == EVisibility::kVisible
            && (ignore_pointer_events_none || style.UsedPointerEvents() != EPointerEvents::kNone)
    }
    /// Source layout_replaced.h:139-146 uses the element fact, independently
    /// of whether overflow is currently clipped by this LayoutObject.
    pub fn IsReplacedElementRespectingCSSOverflow(self) -> bool {
        use crate::internal::layout_node_metadata::Element;
        unsafe { DynamicTo::<Element>(self.object.GetNode()).as_ref() }
            .and_then(|element| element.InputElementData().as_ref())
            .is_some_and(|data| data.replaced_respects_css_overflow)
    }
    pub fn HasSelfPaintingLayer(self) -> bool {
        unsafe { DynamicTo::<LayoutBoxModelObject>(self.object).as_ref() }
            .is_some_and(LayoutBoxModelObject::HasSelfPaintingLayer)
    }
    pub fn FirstPhysicalFragment(self) -> Option<&'a PhysicalBoxFragment> {
        let box_ = unsafe { DynamicTo::<LayoutBox>(self.object).as_ref() }?;
        if box_.PhysicalFragmentCount() == 0 {
            None
        } else {
            unsafe { box_.GetPhysicalFragment(0).as_ref() }
        }
    }
    pub fn PhysicalLocation(self) -> Option<PhysicalOffset> {
        unsafe { DynamicTo::<LayoutBox>(self.object).as_ref() }.map(LayoutBox::PhysicalLocation)
    }
    pub fn ChildPaintBlockedByDisplayLock(self) -> bool {
        self.object.ChildPaintBlockedByDisplayLock()
    }
    pub fn PhysicalBorderBoxRect(self) -> PhysicalRect {
        unsafe { DynamicTo::<LayoutBox>(self.object).as_ref() }
            .expect("box hit cursor")
            .PhysicalBorderBoxRect()
    }
    pub fn PhysicalContentBoxRect(self) -> PhysicalRect {
        unsafe { DynamicTo::<LayoutBox>(self.object).as_ref() }
            .expect("embedded content box")
            .PhysicalContentBoxRect()
    }
    /// Source layout_embedded_content.cc:222,236: the original GetNode owner,
    /// independently of NodeForHitTest's result retargeting.
    pub fn EmbeddedContentOwnerNode(
        self,
    ) -> Option<&'a crate::internal::layout_node_metadata::Node> {
        unsafe { self.object.GetNode().as_ref() }
    }
    /// Source layout_embedded_content.cc:176-191.
    pub fn PointOverResizer(self, point: PhysicalOffset, touch: bool) -> bool {
        use crate::internal::layout_scrollable_area::ResizerHitTestType;
        let Some(box_) = (unsafe { DynamicTo::<LayoutBoxModelObject>(self.object).as_ref() })
        else {
            return false;
        };
        let Some(area) = (unsafe { box_.GetScrollableArea().as_ref() }) else {
            return false;
        };
        area.IsAbsolutePointInResizeControl(
            &foundation::ToRoundedPoint(&point),
            if touch {
                ResizerHitTestType::kResizerForTouch
            } else {
                ResizerHitTestType::kResizerForPointer
            },
        )
    }
    pub fn StitchedSize(self) -> foundation::PhysicalSize {
        unsafe { DynamicTo::<LayoutBox>(self.object).as_ref() }
            .expect("box hit cursor")
            .StitchedSize()
    }
    pub fn OverflowClipRect(self) -> PhysicalRect {
        unsafe { DynamicTo::<LayoutBox>(self.object).as_ref() }
            .expect("box hit cursor")
            .OverflowClipRectWithBehavior(
                foundation::OverlayScrollbarClipBehavior::kExcludeOverlayScrollbarSizeForHitTesting,
            )
    }
    pub fn ShouldClipOverflowAlongEitherAxis(self) -> bool {
        self.object.ShouldClipOverflowAlongEitherAxis()
    }
    pub fn VisualOverflowRectIncludingFilters(self) -> PhysicalRect {
        unsafe { DynamicTo::<LayoutBoxModelObject>(self.object).as_ref() }
            .expect("box model hit cursor")
            .VisualOverflowRectIncludingFilters()
    }
    /// Source layout_replaced.cc:107-110 and layout_box_visual_overflow.cc.
    /// Border-shape clipping uses the native LayoutBox outsets, independently
    /// of which physical fragment is currently being visited.
    pub fn ExpandBorderShapeRectForOverflowClipMargin(self, rect: &mut PhysicalRect) {
        let box_ = unsafe { DynamicTo::<LayoutBox>(self.object).as_ref() }
            .expect("border shape clipping is a LayoutBox query");
        if box_.ShouldApplyOverflowClipMargin() {
            rect.Expand(&box_.BorderOutsetsForClipping());
        }
    }
    /// Source layout_box_hot.cc:21-68. `None` is the effective root
    /// scroller's unconditional admission, not absent geometry. The caller
    /// performs the final HitTestLocation intersection in Interaction.
    /// This query is only valid for a LayoutBox subclass.
    pub fn MayIntersectRect(self, visual_overflow: bool) -> Option<PhysicalRect> {
        let box_ = unsafe { DynamicTo::<LayoutBox>(self.object).as_ref() }
            .expect("MayIntersect is a LayoutBox query");
        if box_.IsEffectiveRootScroller() {
            return None;
        }
        let hit_testable_overflow = box_.HasVisualOverflow()
            && (!box_.ShouldClipOverflowAlongBothAxis()
                || (box_.ShouldApplyOverflowClipMargin()
                    && box_
                        .StyleRef()
                        .OverflowClipMargin()
                        .as_ref()
                        .is_some_and(|margin| {
                            margin.GetMargin() > foundation::LayoutUnit::default()
                        })));
        let rect = if visual_overflow {
            box_.VisualOverflowRectIncludingFilters()
        } else if hit_testable_overflow {
            let mut overflow = box_.VisualOverflowRect();
            if box_.ShouldClipOverflowAlongEitherAxis() {
                overflow.Intersect(&box_.OverflowClipRect());
            }
            overflow.Unite(&box_.PhysicalBorderBoxRect());
            overflow
        } else {
            box_.PhysicalBorderBoxRect()
        };
        Some(rect)
    }
    pub fn TextControlFacts(self) -> foundation::layout_hit_test::TextControlHitFacts {
        let (_, editor, container) = self.TextControlNodes();
        foundation::layout_hit_test::TextControlHitFacts {
            inner_editor_node: editor.InputId(),
            container_node: container.map(|node| node.InputId()),
        }
    }
    /// Source layout_text_control_single_line.cc:62-69 and
    /// layout_text_control_multi_line.cc:44-47. Borrow the actual shadow
    /// nodes, whose identities must not be projected to the control host.
    pub fn TextControlNodes(
        self,
    ) -> (
        &'a crate::internal::layout_node_metadata::Node,
        &'a crate::internal::layout_node_metadata::Node,
        Option<&'a crate::internal::layout_node_metadata::Node>,
    ) {
        use crate::internal::{form_node_metadata::TextControlElement, layout_node_metadata::Node};
        let node = unsafe { self.object.GetNode().as_ref() }.expect("native text control node");
        let host =
            unsafe { DynamicTo::<TextControlElement>(node).as_ref() }.expect("native text control");
        let editor = unsafe { (host.InnerEditorElement() as *const Node).as_ref() }
            .expect("laid-out text control retains its actual inner editor");
        let container = unsafe { (host.HitContainerElement() as *const Node).as_ref() };
        (node, editor, container)
    }
    /// Source layout_text_control.cc:84-92, only evaluated after retargeting
    /// admission, using the same native coordinate-conversion owner.
    pub fn InnerEditorOffset(self) -> Option<PhysicalOffset> {
        use crate::internal::{
            form_node_metadata::TextControlElement, layout_node_metadata::Node,
            map_coordinates_flags::MapCoordinatesFlags,
        };
        let host = unsafe { DynamicTo::<TextControlElement>(self.object.GetNode()).as_ref() }
            .expect("native text control");
        let editor = unsafe { (host.InnerEditorElement() as *const Node).as_ref() }
            .expect("native inner editor");
        let object = unsafe { editor.GetLayoutObject().as_ref() }?;
        Some(object.LocalToAncestorPointPhysical(
            &PhysicalOffset::default(),
            DynamicTo::<LayoutBoxModelObject>(self.object),
            MapCoordinatesFlags::empty(),
        ))
    }
    pub fn HitNodeIsDescendantOfInnerEditor(
        self,
        node: &crate::internal::layout_node_metadata::Node,
    ) -> bool {
        let (_, editor, _) = self.TextControlNodes();
        let mut parent = node.parentNode();
        while let Some(node) = unsafe { parent.as_ref() } {
            if std::ptr::eq(&**node, editor) {
                return true;
            }
            parent = node.parentNode();
        }
        false
    }
    pub fn Target(self) -> NativeHitTestTarget<'a> {
        use LayoutHitTestKind as K;
        if self.object.IsSVGHiddenContainer() {
            return NativeHitTestTarget::Hidden(self);
        }
        if self.object.IsLayoutInline() {
            return NativeHitTestTarget::Inline(self);
        }
        let kind = match self.object.RuntimeClass() {
            LayoutObjectClass::SvgShape => K::SvgShape,
            LayoutObjectClass::SvgText => K::SvgText,
            LayoutObjectClass::SvgGroup => K::SvgContainer,
            LayoutObjectClass::SvgViewportContainer => K::SvgViewportContainer,
            LayoutObjectClass::SvgRoot => K::SvgRoot,
            LayoutObjectClass::TextControlSingleLine => K::TextControlSingleLine,
            LayoutObjectClass::TextControlMultiLine => K::TextControlMultiLine,
            LayoutObjectClass::Replaced => {
                if self.object.IsLayoutImage() {
                    K::Image
                } else if self.object.IsLayoutEmbeddedContent() {
                    K::EmbeddedContent
                } else {
                    K::Replaced
                }
            }
            LayoutObjectClass::Base
            | LayoutObjectClass::BoxModelObject
            | LayoutObjectClass::Box
            | LayoutObjectClass::Text
            | LayoutObjectClass::SvgInlineText
            | LayoutObjectClass::TableColumn => return NativeHitTestTarget::Base(self),
            _ => K::Block,
        };
        if matches!(
            kind,
            K::SvgShape | K::SvgText | K::SvgContainer | K::SvgViewportContainer
        ) {
            NativeHitTestTarget::Svg { source: self, kind }
        } else {
            NativeHitTestTarget::Physical {
                source: self,
                kind,
                fragment: self.FirstPhysicalFragment(),
            }
        }
    }
    pub fn Kind(self) -> LayoutHitTestKind {
        self.Target().Kind()
    }
    pub fn SameLayoutObjectItems(self) -> NativeHitTestInlineItems<'a> {
        let mut cursor = InlineCursor::default();
        cursor.MoveToLayoutObject(self.object);
        NativeHitTestInlineItems {
            cursor,
            owner: self,
        }
    }
    pub fn CulledInlineItems(self, parent: &InlineCursor) -> NativeHitTestInlineItems<'a> {
        let mut cursor = parent.clone();
        cursor.MoveToIncludingCulledInline(self.object);
        NativeHitTestInlineItems {
            cursor,
            owner: self,
        }
    }
    pub fn IsDescendantOfInlineParent(self, parent: &InlineCursor) -> bool {
        self.object
            .IsDescendantOf(parent.GetLayoutBlockFlow().cast())
    }
    /// Source HitTestResult::GetScrollableContainer, cc:299-314, delegates
    /// to the same resident ScrollableAreaTraversal used by wheel handling.
    pub fn ScrollableAreas(
        self,
    ) -> crate::native_wheel_scrollable_traversal::NativeWheelScrollableAreaTraversal {
        crate::native_wheel_scrollable_traversal::NativeWheelScrollableAreaTraversal::ForObject(
            self.object,
        )
    }
    pub fn ReverseChildren(self) -> NativeHitTestChildren<'a> {
        NativeHitTestChildren {
            next: self.object.SlowLastChild(),
            _owner: self,
        }
    }
}
impl<'a> NativeHitTestTarget<'a> {
    pub fn Kind(&self) -> LayoutHitTestKind {
        match self {
            Self::Base(_) => LayoutHitTestKind::Base,
            Self::Hidden(_) => LayoutHitTestKind::Hidden,
            Self::Inline(_) => LayoutHitTestKind::Inline,
            Self::Physical { kind, .. } | Self::Svg { kind, .. } => *kind,
        }
    }
    pub fn Cursor(&self) -> NativeHitTestCursor<'a> {
        match self {
            Self::Base(c) | Self::Hidden(c) | Self::Inline(c) => *c,
            Self::Physical { source, .. } | Self::Svg { source, .. } => *source,
        }
    }
}
pub struct NativeHitTestChildren<'a> {
    next: *const LayoutObject,
    _owner: NativeHitTestCursor<'a>,
}
impl<'a> Iterator for NativeHitTestChildren<'a> {
    type Item = NativeHitTestCursor<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let object = unsafe { self.next.as_ref() }?;
        self.next = object.PreviousSibling();
        Some(NativeHitTestCursor::ForObject(object))
    }
}
pub struct NativeHitTestInlineItems<'a> {
    cursor: InlineCursor,
    owner: NativeHitTestCursor<'a>,
}
#[derive(Clone)]
pub struct NativeHitTestInlineItem<'a> {
    cursor: InlineCursor,
    _owner: NativeHitTestCursor<'a>,
}
impl<'a> Iterator for NativeHitTestInlineItems<'a> {
    type Item = NativeHitTestInlineItem<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor.IsNull() {
            return None;
        }
        let item = NativeHitTestInlineItem {
            cursor: self.cursor.clone(),
            _owner: self.owner,
        };
        self.cursor.MoveToNextForSameLayoutObject();
        Some(item)
    }
}
impl<'a> NativeHitTestInlineItem<'a> {
    pub fn Identity(&self) -> usize {
        self.cursor.CurrentItem() as usize
    }
    pub fn Item(&self) -> &'a FragmentItem {
        unsafe { &*self.cursor.CurrentItem() }
    }
    pub fn Cursor(&self) -> InlineCursor {
        self.cursor.clone()
    }
    pub fn ContainerFragmentIndex(&self) -> usize {
        self.cursor.ContainerFragmentIndex() as usize
    }
    pub fn Container(&self) -> &'a PhysicalBoxFragment {
        unsafe { &*(self.cursor.ContainerFragment() as *const _) }
    }
    pub fn ItemIndex(&self) -> usize {
        unsafe {
            self.cursor
                .CurrentItem()
                .offset_from(self.cursor.Items().Items().as_ptr()) as usize
        }
    }
    pub fn BoxFragment(&self) -> Option<&'a PhysicalBoxFragment> {
        unsafe { self.cursor.Current().BoxFragment().as_ref() }
    }
    pub fn OffsetInContainerFragment(&self) -> PhysicalOffset {
        self.cursor.Current().OffsetInContainerFragment()
    }
    pub fn RectInContainerFragment(&self) -> PhysicalRect {
        self.cursor.Current().RectInContainerFragment()
    }
}
