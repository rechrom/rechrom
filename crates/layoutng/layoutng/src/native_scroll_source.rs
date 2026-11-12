//! Borrowed source owner for focus and autoscroll. This port retains the live
//! LayoutObject identity and returns ordinary node/offset mutations.
use crate::internal::layout_box::LayoutBox;
pub use crate::internal::layout_box::{
    NativeScrollOffset, NativeScrollResult, ScrollAlignment, ScrollAlignmentBehavior,
    ScrollIntoViewParams,
};
use crate::internal::layout_input::Offset;
use crate::internal::layout_object::LayoutObject;
use crate::internal::layout_pass_scope::LayoutPassScope;
pub use crate::internal::layout_scrollable_area::{
    NativeLayoutSnapPlan, NativePageScrollCommand, NativePageScrollRequest,
    NativePendingSnapTarget, NativeSnapAnimationState, NativeSnapTargetNotification,
};
use crate::layout_engine::{LayoutEngine, LayoutRect};
use foundation::{gfx, LayoutUnit, PhysicalOffset, PhysicalRect, PhysicalSize};

/// Call-scoped facts from LocalFrameView's retained ScrollableArea registry.
/// The native area itself never crosses the LayoutNG package boundary.
#[derive(Clone, Copy, Debug)]
pub struct NativeCompositorScrollArea {
    pub target_node_id: u64,
    pub minimum: Offset,
    pub maximum: Offset,
}

#[derive(Clone, Copy, Debug)]
pub struct NativeCompositorScrollUpdate {
    pub target_node_id: u64,
    pub offset: Offset,
}

/// The translation between document absolute space and the local root-frame
/// space. Supplied by the frame embedding; this value never links two Pages.
#[derive(Clone, Copy, Debug, Default)]
pub struct NativeScrollFrameGeometry {
    pub document_to_root_frame: Offset,
}

pub struct NativeScrollSource<'a> {
    // Holding this borrow prevents tree retirement while the native identity
    // below is in use. No pointer leaves the port.
    _scope: LayoutPassScope,
    _engine: &'a mut LayoutEngine,
    object: *mut LayoutObject,
    frame: NativeScrollFrameGeometry,
}
fn Physical(offset: Offset) -> PhysicalOffset {
    PhysicalOffset::new(
        LayoutUnit::from_f64(offset.x),
        LayoutUnit::from_f64(offset.y),
    )
}
fn OffsetFromPhysical(offset: PhysicalOffset) -> Offset {
    Offset {
        x: offset.left.ToDouble(),
        y: offset.top.ToDouble(),
    }
}
impl LayoutEngine {
    /// LocalFrameView::ScrollableAreaWithElementId, local_frame_view.cc:4279-4289.
    ///
    /// The compositor element id names the resident PaintLayerScrollableArea,
    /// not necessarily the LayoutObject returned for its DOM node. In
    /// particular, the document element is the DOM-visible root scroller while
    /// LayoutView owns the native area. Resolve that original identity through
    /// LayoutView's creation/disposal registry instead of reconstructing it
    /// from exported Paint property nodes.
    pub fn CompositorScrollArea(
        &self,
        expected: foundation::CompositorElementId,
    ) -> Option<NativeCompositorScrollArea> {
        let tree = self.GetLayoutTree()?;
        let _scope = self.NativeScrollPassScope();
        let view = unsafe {
            foundation::DynamicTo::<crate::internal::layout_view::LayoutView>(
                tree.Root() as *const LayoutObject,
            )
            .as_ref()
        }?;
        let area = view.RegisteredScrollableAreas().into_iter().find(|area| {
            let pointer = area.Get();
            !pointer.is_null()
                && !unsafe { &*pointer }.HasBeenDisposed()
                && unsafe { &*pointer }.GetScrollElementId() == expected
        })?;
        let area = unsafe { &*area.Get() };
        let box_ = unsafe { area.GetLayoutBox().as_ref() }?;
        let node = unsafe { box_.GetNode().as_ref() }?;
        let minimum = area.MinimumScrollOffset();
        let maximum = area.MaximumScrollOffset();
        Some(NativeCompositorScrollArea {
            target_node_id: node.InputId(),
            minimum: Offset {
                x: f64::from(minimum.x()),
                y: f64::from(minimum.y()),
            },
            maximum: Offset {
                x: f64::from(maximum.x()),
                y: f64::from(maximum.y()),
            },
        })
    }


    /// ScrollingCoordinator::DidCompositorScroll and
    /// ScrollableArea::DidCompositorScroll. The resident native area commits
    /// first; the returned scalar mutation lets the embedding reflect the same
    /// value into its DOM owner without exporting the area itself.
    pub fn DidCompositorScroll(
        &mut self,
        expected: foundation::CompositorElementId,
        position: Offset,
        _source_type: scroll_snap::scroll_source_type::ScrollSourceType,
    ) -> Option<NativeCompositorScrollUpdate> {
        let tree = self.GetLayoutTree()?;
        let _scope = self.NativeScrollPassScope();
        let view = unsafe {
            foundation::DynamicTo::<crate::internal::layout_view::LayoutView>(
                tree.Root() as *const LayoutObject,
            )
            .as_ref()
        }?;
        let area = view.RegisteredScrollableAreas().into_iter().find(|area| {
            let pointer = area.Get();
            !pointer.is_null()
                && !unsafe { &*pointer }.HasBeenDisposed()
                && unsafe { &*pointer }.GetScrollElementId() == expected
        })?;
        let area = unsafe { &mut *area.Get() };
        let box_ = unsafe { area.GetLayoutBox().as_ref() }?;
        let node = unsafe { box_.GetNode().as_ref() }?;
        let position = gfx::PointF::new(position.x as f32, position.y as f32);
        let offset = area.ClampScrollOffset(&area.ScrollPositionToOffset(&position));
        area.UpdateScrollOffset(
            offset,
            layoutng_style::style::scroll_enums::mojom::blink::ScrollType::kCompositor,
        );
        Some(NativeCompositorScrollUpdate {
            target_node_id: node.InputId(),
            offset: Offset {
                x: f64::from(offset.x()),
                y: f64::from(offset.y()),
            },
        })
    }

    /// SelectionEditor::SetSelectionAndEndTyping keeps the actual anonymous
    /// containing-block identity while comparing old/new text-overflow owners.
    /// The opaque value is call-scoped to the synchronous FrameSelection
    /// transaction; no LayoutObject pointer is retained by the caller.
    pub fn SelectionTextOverflowOwnerIdentity(&self, node_id: u64) -> Option<usize> {
        let _scope = self.NativeScrollPassScope();
        let object = self.GetLayoutTree()?.GetLayoutObjectForNode(node_id)?;
        let owner = object.ContainingBlockForTextOverflow();
        (!owner.is_null()).then_some(owner as usize)
    }

    /// Consume the call-scoped identity returned immediately above. The Page
    /// holds this engine for the entire synchronous transaction, so the native
    /// tree cannot retire the anonymous owner between lookup and mutation.
    pub fn SetContainsSelectionFocusByIdentity(
        &mut self,
        owner_identity: usize,
        contains_focus: bool,
    ) -> Option<u64> {
        let _scope = self.NativeScrollPassScope();
        let owner = owner_identity as *mut LayoutObject;
        assert!(!owner.is_null(), "selection text-overflow owner");
        let owner = unsafe { &mut *owner };
        if owner.StyleRef().TextOverflow().IsClip()
            || owner.ContainsSelectionFocus() == contains_focus
        {
            return None;
        }
        owner.SetContainsSelectionFocus(contains_focus);
        unsafe { owner.GetNode().as_ref() }.map(|node| node.InputId())
    }

    /// SelectionEditor::InvalidateTextOverflowLayoutForNode,
    /// selection_editor.cc:196-212. This is invoked by Page only after the
    /// deferred internal-default task has begun; the synchronous selection
    /// transaction never dirties Layout during hit testing.
    pub fn InvalidateTextOverflowLayoutForNode(&mut self, node_id: u64) -> bool {
        let _scope = self.NativeScrollPassScope();
        let Some(owner) = self
            .GetLayoutTree()
            .and_then(|tree| tree.GetLayoutObjectForNode(node_id))
            .map(|owner| owner as *const LayoutObject as *mut LayoutObject)
        else {
            return false;
        };
        let reason = std::ptr::addr_of!(crate::internal::layout_invalidation_reason::kStyleChange);
        unsafe { &mut *owner }.SetNeedsLayout(reason);
        let mut child = unsafe { &*owner }.SlowFirstChild();
        while let Some(current) = unsafe { child.as_mut() } {
            let next = current.NextSibling();
            if current.IsLayoutBlock() {
                current.SetNeedsLayout(reason);
            }
            child = next;
        }
        true
    }

    pub fn BoundingBoxForScrollIntoView(&self, id: u64) -> PhysicalRect {
        let _scope = self.NativeScrollPassScope();
        self.GetLayoutTree()
            .and_then(|t| t.GetLayoutObjectForNode(id))
            .map_or_else(PhysicalRect::default, |o| {
                o.AbsoluteBoundingBoxRectForScrollIntoView()
            })
    }
    /// Source: Scrollbar::ConvertFromRootFrame after its FrameView conversion,
    /// scrollbar.cc:808-830; paint_layer_scrollable_area.cc:388-398.
    pub fn ScrollbarPointFromContainingFrame(
        &self,
        scrollbar: &crate::internal::scrollbar::Scrollbar,
        point: gfx::Point,
    ) -> gfx::Point {
        if self.GetLayoutTree().is_none() {
            return point;
        }
        let _scope = self.NativeScrollPassScope();
        scrollbar.ConvertContainingFramePointToScrollbar(point)
    }
    pub fn NativeScrollSource(&mut self, id: u64) -> Option<NativeScrollSource<'_>> {
        self.NativeScrollSourceInFrame(id, NativeScrollFrameGeometry::default())
    }
    pub fn NativeScrollSourceInFrame(
        &mut self,
        id: u64,
        frame: NativeScrollFrameGeometry,
    ) -> Option<NativeScrollSource<'_>> {
        let object =
            self.GetLayoutTree()?.OutputNode(id)? as *const LayoutObject as *mut LayoutObject;
        let scope = self.NativeScrollPassScope();
        Some(NativeScrollSource {
            _scope: scope,
            _engine: self,
            object,
            frame,
        })
    }
}
impl NativeScrollSource<'_> {
    pub fn IsScrollContainerBox(&self) -> bool {
        let object = unsafe { &*self.object };
        object.IsBox() && unsafe { &*self.object.cast::<LayoutBox>() }.IsScrollContainer()
    }
    pub fn ScrollIntoOwnArea(
        &mut self,
        rect: PhysicalRect,
        margin: layoutng_geometry::geometry::box_strut::PhysicalBoxStrut,
        params: ScrollIntoViewParams,
    ) -> NativeScrollResult {
        assert!(unsafe { &*self.object }.IsBox());
        unsafe { &*self.object.cast::<LayoutBox>() }.ScrollIntoOwnArea(rect, margin, params)
    }
    fn Box(&self) -> Option<&LayoutBox> {
        unsafe { (&*self.object).EnclosingBox().as_ref() }
    }
    pub fn FindAutoscrollable(&self, middle_click: bool) -> Option<u64> {
        let box_ = LayoutBox::FindAutoscrollable(self.object, middle_click);
        let node = unsafe { box_.as_ref()? }.GetNode();
        Some(unsafe { node.as_ref()? }.InputId())
    }
    pub fn HasHorizontallyScrollableAncestor(&self) -> bool {
        LayoutBox::HasHorizontallyScrollableAncestor(self.object)
    }
    pub fn CalculateAutoscrollDirection(&self, point_in_root_frame: Offset) -> Offset {
        let Some(box_) = self.Box() else {
            return Offset::default();
        };
        OffsetFromPhysical(box_.CalculateAutoscrollDirectionInRootFrame(
            &gfx::PointF::new(point_in_root_frame.x as f32, point_in_root_frame.y as f32),
            Physical(self.frame.document_to_root_frame),
        ))
    }
    pub fn Autoscroll(&mut self, point_in_root_frame: Offset) -> NativeScrollResult {
        let Some(box_) = self.Box() else {
            return NativeScrollResult::default();
        };
        // LocalFrameView::ConvertFromRootFrame; host FrameView retains this
        // translation and Layout owns the scroll algorithm after conversion.
        let absolute = Physical(point_in_root_frame) - Physical(self.frame.document_to_root_frame);
        box_.AutoscrollWithResult(&absolute)
    }
    /// Element::UpdateSelectionOnFocus, element.cc:8741-8759; the default
    /// CreateScrollIntoViewParams aligns both axes CenterIfNeeded. Preserve
    /// partial horizontal visibility only for the ordinary Element focus path.
    /// BoundingBoxForScrollIntoView reads this same resident LayoutObject.
    pub fn ScrollFocusIntoView(
        &mut self,
        preserve_partial_horizontal_position: bool,
    ) -> NativeScrollResult {
        let mut params = ScrollIntoViewParams {
            align_x: ScrollAlignment::CenterIfNeeded(),
            align_y: ScrollAlignment::CenterIfNeeded(),
            ..ScrollIntoViewParams::default()
        };
        if preserve_partial_horizontal_position {
            params.align_x.rect_partial = ScrollAlignmentBehavior::NoScroll;
        }
        let absolute = unsafe { &*self.object }.AbsoluteBoundingBoxRectForScrollIntoView();
        self.ScrollPhysicalRectToVisible(absolute, params)
    }
    /// ElementFragmentAnchor's default ScrollIntoViewOptions: block Start,
    /// inline Nearest, behavior Auto. Alignment belongs to the live style.
    /// scroll_into_view_util.cc:693-739,763-800.
    pub fn ScrollFragmentAnchorIntoView(&mut self) -> NativeScrollResult {
        let style = unsafe { &*self.object }.StyleRef();
        let mut params = ScrollIntoViewParams::default();
        let alignment = match style.GetWritingDirection().BlockStart() {
            foundation::PhysicalDirection::kUp => ScrollAlignmentBehavior::Top,
            foundation::PhysicalDirection::kDown => ScrollAlignmentBehavior::Bottom,
            foundation::PhysicalDirection::kLeft => ScrollAlignmentBehavior::Left,
            foundation::PhysicalDirection::kRight => ScrollAlignmentBehavior::Right,
        };
        let always = ScrollAlignment {
            rect_visible: alignment,
            rect_hidden: alignment,
            rect_partial: alignment,
        };
        if style.IsHorizontalWritingMode() {
            params.align_y = always;
        } else {
            params.align_x = always;
        }
        self.ScrollElementBoundingBoxToVisible(params)
    }
    pub fn ScrollRectToVisible(
        &mut self,
        absolute_rect: LayoutRect,
        params: ScrollIntoViewParams,
    ) -> NativeScrollResult {
        self.ScrollPhysicalRectToVisible(
            PhysicalRect::new(
                Physical(Offset {
                    x: absolute_rect.x,
                    y: absolute_rect.y,
                }),
                PhysicalSize::new(
                    LayoutUnit::from_f64(absolute_rect.width),
                    LayoutUnit::from_f64(absolute_rect.height),
                ),
            ),
            params,
        )
    }
    /// Element::BoundingBoxForScrollIntoView consumes this exact resident
    /// object's source geometry. Alignment comes from the DOM/editing caller.
    pub fn ScrollElementBoundingBoxToVisible(
        &mut self,
        params: ScrollIntoViewParams,
    ) -> NativeScrollResult {
        let bounds = unsafe { &*self.object }.AbsoluteBoundingBoxRectForScrollIntoView();
        self.ScrollPhysicalRectToVisible(bounds, params)
    }
    // scroll_into_view_util.cc:574-604. The source target style supplies
    // scroll-margin even when its enclosing box belongs to another node.
    fn ScrollPhysicalRectToVisible(
        &mut self,
        absolute_rect: PhysicalRect,
        mut params: ScrollIntoViewParams,
    ) -> NativeScrollResult {
        let Some(box_) = self.Box() else {
            return NativeScrollResult::default();
        };
        // ScrollRectToVisible, scroll_into_view_util.cc:598-599.
        params.is_for_scroll_sequence |= params.scroll_type
            == layoutng_style::style::scroll_enums::mojom::blink::ScrollType::kProgrammatic;
        if self.object != box_ as *const LayoutBox as *mut LayoutObject {
            params.include_self = true;
        }
        box_.ScrollRectToVisibleWithMargin(
            absolute_rect,
            params,
            unsafe { &*self.object }.StyleRef().ScrollMarginStrut(),
        )
    }
}

impl NativeScrollSource<'_> {
    fn BoxForScrolling(&mut self) -> Option<&mut LayoutBox> {
        let object = unsafe { &mut *self.object };
        // HTMLInputElement::GetLayoutBoxForScrolling, cc:2613-2617. The
        // native text-field owner retains its actual InnerEditorElement.
        if object.IsTextField() {
            use crate::internal::form_node_metadata::TextControlElement;
            use crate::internal::layout_node_metadata::Node;
            let host =
                unsafe { foundation::DynamicTo::<TextControlElement>(object.GetNode()).as_ref() }
                    .expect("native text field retains its source TextControlElement");
            if let Some(editor) = unsafe { (host.InnerEditorElement() as *mut Node).as_mut() } {
                return unsafe { editor.GetLayoutBox().as_mut() };
            }
        }
        // ContainerNode::GetLayoutBoxForScrolling, cc:915-922.
        if !object.IsBox() {
            return None;
        }
        let box_ = unsafe { &mut *(self.object as *mut LayoutBox) };
        unsafe { box_.ContentLayoutBox().as_mut() }
    }
    /// ContainerNode::GetLayoutBoxForScrolling, container_node.cc:915-922;
    /// Element::GetPhysicalDirectionForCommand, element.cc:399-429 and
    /// Element::HandleScrollByPageCommand, element.cc:3172-3196. DOM's active
    /// document check and synchronous style/layout flush precede this port.
    pub fn ScrollByPageCommand(
        &mut self,
        command: NativePageScrollCommand,
    ) -> Option<NativePageScrollRequest> {
        // Exact source box; ordinary inline elements never page an ancestor.
        let content = self.BoxForScrolling()?;
        let area = unsafe { content.GetScrollableArea().as_mut()? };
        let writing = content.StyleRef().GetWritingDirection();
        use foundation::PhysicalDirection::*;
        use NativePageScrollCommand::*;
        let direction = match command {
            Up => kUp,
            Down => kDown,
            Left => kLeft,
            Right => kRight,
            InlineStart => writing.InlineStart(),
            InlineEnd => writing.InlineEnd(),
            BlockStart => writing.BlockStart(),
            BlockEnd => writing.BlockEnd(),
        };
        Some(area.ScrollByPageWithSnap(
            direction,
            foundation::RuntimeEnabledFeatures::FractionalScrollOffsetsEnabled(),
        ))
    }
}

impl LayoutEngine {
    /// Borrow the native FrameView-equivalent queue; no tree projection or walk.
    pub fn PendingPostLayoutSnapTargets(&self) -> Vec<NativePendingSnapTarget> {
        let Some(tree) = self.GetLayoutTree() else {
            return Vec::new();
        };
        let root = tree.Root();
        assert!(root.IsLayoutView());
        unsafe {
            &*(root as *const LayoutObject as *const crate::internal::layout_view::LayoutView)
        }
        .PendingPostLayoutSnapTargets()
    }
    pub fn ResolvePendingPostLayoutSnaps(
        &mut self,
        animation_states: &[NativeSnapAnimationState],
        should_defer_layout_snap: bool,
    ) -> Vec<NativeLayoutSnapPlan> {
        let Some(tree) = self.GetLayoutTree() else {
            return Vec::new();
        };
        let root = tree.Root() as *const LayoutObject;
        assert!(unsafe { &*root }.IsLayoutView());
        let _scope = self.NativeScrollPassScope();
        unsafe { &*(root as *const crate::internal::layout_view::LayoutView) }
            .ResolvePendingPostLayoutSnaps(animation_states, should_defer_layout_snap)
    }
}

impl LayoutEngine {
    /// Synchronize Element::SetHasFocusWithinUpToAncestor's DOM-owned editing
    /// input onto the same resident native Node before snap-area focus data is
    /// recomputed. This is a boundary update, not a copied interaction tree.
    pub fn UpdateEditingNodeFactsForFocus(
        &mut self,
        node_id: u64,
        facts: crate::EditingNodeFacts,
    ) -> bool {
        let _scope = self.NativeScrollPassScope();
        let Some(object) = self
            .GetLayoutTreeMut()
            .and_then(|tree| tree.GetLayoutObjectForNodeMut(node_id))
        else {
            return false;
        };
        let Some(node) = (unsafe { object.GetNode().as_mut() }) else {
            return false;
        };
        node.SetInputEditingFacts(facts);
        true
    }

    pub fn UpdatePendingSnapData(&mut self) {
        let Some(tree) = self.GetLayoutTree() else {
            return;
        };
        let root = tree.Root() as *const LayoutObject;
        assert!(unsafe { &*root }.IsLayoutView());
        let _scope = self.NativeScrollPassScope();
        unsafe { &*(root as *const crate::internal::layout_view::LayoutView) }
            .ExecutePendingSnapDataUpdates();
    }
}

impl NativeScrollSource<'_> {
    /// Source Element::SetHasFocusWithinUpToAncestor's already selected snap
    /// container, element.cc:9168-9178; no selection or animation is triggered.
    pub fn IsSnapContainerForFocus(&mut self) -> bool {
        self.BoxForScrolling().is_some_and(|box_| {
            !box_.GetScrollableArea().is_null() && !box_.StyleRef().GetScrollSnapType().is_none
        })
    }
    pub fn UpdateFocusDataForSnapAreas(&mut self) {
        let Some(content) = self.BoxForScrolling() else {
            return;
        };
        if content.StyleRef().GetScrollSnapType().is_none {
            return;
        }
        if let Some(area) = unsafe { content.GetScrollableArea().as_mut() } {
            area.UpdateFocusDataForSnapAreas();
        }
    }
}

impl NativeScrollSource<'_> {
    pub fn ScrollElementIdentity(&mut self) -> Option<foundation::CompositorElementId> {
        let box_ = self.BoxForScrolling()?;
        Some(unsafe { box_.GetScrollableArea().as_ref()? }.GetScrollElementId())
    }
    pub fn ScrollFinishedSnapTargets(
        &mut self,
        expected: foundation::CompositorElementId,
    ) -> Option<NativeSnapTargetNotification> {
        let area = unsafe { self.BoxForScrolling()?.GetScrollableArea().as_mut()? };
        if area.GetScrollElementId() != expected {
            return None;
        }
        area.ScrollFinishedSnapTargets()
    }
}

impl NativeScrollSource<'_> {
    pub fn SetCompositorSnapTargetIds(
        &mut self,
        expected: foundation::CompositorElementId,
        targets: scroll_snap::scroll_snap_data::TargetSnapAreaElementIds,
    ) -> bool {
        let Some(box_) = self.BoxForScrolling() else {
            return false;
        };
        let Some(area) = (unsafe { box_.GetScrollableArea().as_mut() }) else {
            return false;
        };
        area.GetScrollElementId() == expected && area.SetTargetSnapAreaElementIds(targets)
    }
}

#[path = "native_overscroll_source.rs"]
mod overscroll_source;

/// Source PLSA.cc:3527-3567. The native style chooses the logical axes;
/// DOM alone resolves arena membership and pseudo parent retargeting.
#[derive(Clone, Copy, Debug)]
pub struct NativeSnapNotificationNodes {
    pub event_target: u64,
    pub block: Option<u64>,
    pub inline: Option<u64>,
    pub x: Option<u64>,
    pub y: Option<u64>,
}
impl NativeScrollSource<'_> {
    pub fn SnapNotificationNodes(
        &self,
        notification: NativeSnapTargetNotification,
    ) -> Option<NativeSnapNotificationNodes> {
        let box_ = self.Box()?;
        let area = unsafe { box_.GetScrollableArea().as_ref()? };
        if area.HasBeenDisposed()
            || area.GetScrollElementId() != notification.target.scroll_element_id
        {
            return None;
        }
        let mut node = unsafe { box_.GetNode().as_ref() };
        if node.is_none() {
            if let Some(parent) =
                unsafe { box_.Parent().as_ref() }.filter(|parent| parent.IsFieldset())
            {
                node = unsafe { parent.GetNode().as_ref() };
            }
        }
        let node = node?;
        // CompositorElementId carries a DOMNodeId namespace (source
        // compositor_element_id.cc:51-61); never interpret scroll/layer ids.
        let decode = |id: scroll_snap::ElementId| {
            if !id.IsValid() {
                return None;
            }
            let native = foundation::CompositorElementId::new(id.GetInternalValue());
            assert_eq!(
                foundation::NamespaceFromCompositorElementId(native),
                foundation::CompositorElementIdNamespace::kDOMNodeId
            );
            Some(
                id.GetInternalValue()
                    >> (foundation::compositor_element_id::kCompositorNamespaceBitCount
                        + foundation::compositor_element_id::kCompositorReservedBitCount),
            )
        };
        let x = notification.targets.and_then(|targets| decode(targets.x));
        let y = notification.targets.and_then(|targets| decode(targets.y));
        let horizontal = box_.StyleRef().GetWritingDirection().IsHorizontal();
        Some(NativeSnapNotificationNodes {
            event_target: node.InputId(),
            block: if horizontal { y } else { x },
            inline: if horizontal { x } else { y },
            x,
            y,
        })
    }
}

impl NativeScrollSource<'_> {
    pub fn CompositorSnapChangingNotification(
        &mut self,
        expected: foundation::CompositorElementId,
        targets: scroll_snap::scroll_snap_data::TargetSnapAreaElementIds,
    ) -> Option<NativeSnapTargetNotification> {
        let box_ = self.BoxForScrolling()?;
        let area = unsafe { box_.GetScrollableArea().as_mut() }?;
        if area.GetScrollElementId() != expected || area.HasBeenDisposed() {
            return None;
        }
        area.CompositorSnapChangingNotification(targets)
    }
}

impl NativeScrollSource<'_> {
    /// spatial_navigation.cc:356-375,403-437: the queried node's own box is
    /// used for overflow classification; its canonical scrolling box supplies
    /// the actual retained ScrollableArea identity.
    pub fn SpatialScrollableArea(
        &mut self,
    ) -> Option<(
        crate::native_autoscroll_source::NativeLogicalScrollArea,
        bool,
        bool,
        bool,
        bool,
        bool,
    )> {
        let object = unsafe { &*self.object };
        let original = unsafe { foundation::DynamicTo::<LayoutBox>(object).as_ref() }?;
        let facts = (
            original.HasScrollableOverflowX(),
            original.HasScrollableOverflowY(),
            original.IsUserScrollable(),
            object.StyleRef().OverflowX() == foundation::EOverflow::kHidden,
            object.StyleRef().OverflowY() == foundation::EOverflow::kHidden,
        );
        let scrolling = self.BoxForScrolling()?;
        let area = crate::native_autoscroll_source::NativeLogicalScrollArea::FromPointer(
            scrolling.GetScrollableArea(),
        )?;
        Some((area, facts.0, facts.1, facts.2, facts.3, facts.4))
    }
}
