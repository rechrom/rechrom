//! Call-scoped LayoutObject virtual adapters; no projected tree is retained.
//!
//! Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
//! Source: layout_object.cc:4524-4544,4602-4610; layout_block.cc:378-403;
//! layout_replaced.cc:129-237; layout_image.cc:377-392;
//! layout_inline.cc:597-697,834-844; layout_text_control_single_line.cc:39-75;
//! layout_text_control_multi_line.cc:31-53; layout_text_control.cc:79-93;
//! layout_embedded_content.cc:176-191,205-313.
use super::{ResidentFragment, ResidentHitTestServices};
use crate::{
    core::{
        layout::{
            hit_test_location::HitTestLocation,
            hit_test_phase::HitTestPhase,
            hit_test_request::HitTestStopNode,
            hit_test_result::{HitTestResult, HitTestResultRequest, NodeRef},
            layout_embedded_content_hit_test::{ChildFrameView, EmbeddedContentHitTestSource},
            layout_inline::{
                self, InlineHitTestItem, InlineHitTestParentCursor, InlineHitTestRegion,
                LayoutInlineHitTestSource,
            },
            layout_text_control_hit_test::TextControlHitTestSource,
            native_hit_test::{
                self, NativeHitTestRequiredPort, NativeHitTestResult, NativeLayoutHitTestSource,
                ReplacedHitTestSource,
            },
        },
        paint::box_fragment_painter::{
            BoxFragmentPainter, CulledInlineLayout, FragmentItemRef, InlineCursor, PhysicalFragment,
        },
    },
    platform::geometry::{contoured_rect::ContouredRectHitTest, path::PathHitTest},
};
use foundation::{
    layout_hit_test::LayoutHitTestKind as Kind, PhysicalOffset, PhysicalRect, PhysicalSize,
};
use layoutng_assembly::layout_engine::{
    FragmentCursor, NativeHitTestCursor, NativeHitTestInlineItem, NativeHitTestTarget,
    PaintFragmentCursor,
};

/// Source layout_box_hot.cc:35-68. Layout owns selection of the native
/// overflow rectangle; Interaction owns the HitTestLocation intersection.
/// `source` must be a LayoutBox, just as the source method requires.
pub fn MayIntersect<L: ?Sized>(
    source: NativeHitTestCursor<'_>,
    result: &HitTestResult<'_, L>,
    location: &HitTestLocation,
    offset: PhysicalOffset,
) -> bool {
    source
        .MayIntersectRect(result.GetHitTestRequest().IsHitTestVisualOverflow())
        .is_none_or(|mut overflow| {
            overflow.Move(&offset);
            location.Intersects(&overflow)
        })
}

/// A stack-local view of the same resident LayoutObject and PhysicalFragment.
/// The remaining virtual subclasses require their specific source adapter;
/// construction reports that requirement before any result is mutated.
pub struct ResidentNativeHitTestSource<'a> {
    source: NativeHitTestCursor<'a>,
    root: PaintFragmentCursor<'a>,
    services: &'a dyn ResidentHitTestServices<()>,
    fragment: Option<ResidentFragment<'a, ()>>,
    inline_parent: Option<&'a ResidentInlineParent>,
    text_control: Option<ResidentTextControlNodes>,
    embedded_owner: Option<NodeRef>,
}
struct ResidentTextControlNodes {
    control: NodeRef,
    editor: NodeRef,
    container: Option<NodeRef>,
}
impl<'a> ResidentNativeHitTestSource<'a> {
    pub fn new(
        root: PaintFragmentCursor<'a>,
        source: NativeHitTestCursor<'a>,
        services: &'a dyn ResidentHitTestServices<()>,
    ) -> Result<Self, NativeHitTestRequiredPort> {
        let fragment = match source.Target() {
            NativeHitTestTarget::Base(_) | NativeHitTestTarget::Hidden(_) => None,
            NativeHitTestTarget::Physical {
                kind:
                    Kind::Block
                    | Kind::Replaced
                    | Kind::Image
                    | Kind::SvgRoot
                    | Kind::EmbeddedContent
                    | Kind::TextControlSingleLine
                    | Kind::TextControlMultiLine,
                fragment,
                ..
            } => fragment
                .map(|fragment| {
                    let expected: &layoutng_assembly::physical_fragment::PhysicalFragment =
                        fragment;
                    let cursor = root
                        .ForPaintObject(source.PaintIdentity())
                        .filter(|cursor| {
                            cursor
                                .Cursor()
                                .PhysicalFragment()
                                .is_some_and(|actual| std::ptr::eq(actual, expected))
                        })
                        .ok_or(NativeHitTestRequiredPort::MissingPaintFragment {
                            layout_identity: source.Identity(),
                        })?;
                    Ok(ResidentFragment::new(cursor, services))
                })
                .transpose()?,
            NativeHitTestTarget::Inline(_) | NativeHitTestTarget::Svg { .. } => None,
            target => return Err(NativeHitTestRequiredPort::Special(target.Kind())),
        };
        if matches!(
            source.Kind(),
            Kind::Replaced | Kind::Image | Kind::EmbeddedContent | Kind::SvgRoot
        ) && fragment.is_none()
        {
            return Err(NativeHitTestRequiredPort::MissingPaintFragment {
                layout_identity: source.Identity(),
            });
        }
        let text_control = if matches!(
            source.Kind(),
            Kind::TextControlSingleLine | Kind::TextControlMultiLine
        ) {
            let (control, editor, container) = source.TextControlNodes();
            let resolve = |node| {
                services
                    .ResolveNativeNode(node)
                    .ok_or(NativeHitTestRequiredPort::TextControl)
            };
            Some(ResidentTextControlNodes {
                control: resolve(control)?,
                editor: resolve(editor)?,
                container: container.map(resolve).transpose()?,
            })
        } else {
            None
        };
        let embedded_owner = if source.Kind() == Kind::EmbeddedContent {
            Some(
                source
                    .EmbeddedContentOwnerNode()
                    .and_then(|node| services.ResolveNativeNode(node))
                    .ok_or(NativeHitTestRequiredPort::EmbeddedContent)?,
            )
        } else {
            None
        };
        Ok(Self {
            source,
            root,
            services,
            fragment,
            inline_parent: None,
            text_control,
            embedded_owner,
        })
    }
    fn PaintCursor(&self) -> PaintFragmentCursor<'a> {
        self.fragment
            .as_ref()
            .expect("replaced geometry owner admitted at construction")
            .Cursor()
    }
    pub fn Cursor(&self) -> NativeHitTestCursor<'a> {
        self.source
    }
}

/// A parent cursor is scoped to the source inline adapter. The identity only
/// verifies that the translated method is using that same borrowed scope.
struct ResidentInlineParent(layoutng_assembly::inline_cursor::InlineCursor);
impl InlineHitTestParentCursor for ResidentInlineParent {
    fn Identity(&self) -> usize {
        self as *const _ as usize
    }
}

struct ResidentInlineItem<'a> {
    item: NativeHitTestInlineItem<'a>,
    root: PaintFragmentCursor<'a>,
    services: &'a dyn ResidentHitTestServices<()>,
}
impl<'r> InlineHitTestItem<'r, ()> for ResidentInlineItem<'_> {
    fn ContainerFragmentIndex(&self) -> usize {
        self.item.ContainerFragmentIndex()
    }
    fn HasBoxFragment(&self) -> bool {
        self.item.BoxFragment().is_some()
    }
    fn IsOpaqueBoxFragment(&self) -> bool {
        self.item
            .BoxFragment()
            .is_some_and(|fragment| fragment.IsOpaque())
    }
    fn OffsetInContainerFragment(&self) -> PhysicalOffset {
        self.item.OffsetInContainerFragment()
    }
    fn RectInContainerFragment(&self) -> PhysicalRect {
        self.item.RectInContainerFragment()
    }
    fn BoxFragmentNodeAtPoint(
        &self,
        result: &mut HitTestResult<'r, ()>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
        inline_root_offset: PhysicalOffset,
        phase: HitTestPhase,
    ) -> bool {
        let fragment = self
            .item
            .BoxFragment()
            .expect("inline item has a box fragment");
        let fragment =
            ResidentFragment::new(self.root.ForPhysicalFragment(fragment), self.services);
        let container = ResidentFragment::new(
            self.root.ForPhysicalFragment(self.item.Container()),
            self.services,
        );
        let item = ResidentFragment::new(
            self.root
                .ForInlineItem(self.item.Container(), self.item.ItemIndex()),
            self.services,
        );
        let cursor = InlineCursor::ForInlineBox(&container, self.item.ItemIndex());
        BoxFragmentPainter::WithInlineItem(&fragment, cursor, &item).NodeAtPointWithInlineRoot(
            result,
            location,
            offset,
            inline_root_offset,
            phase,
        )
    }
}
impl<'r> LayoutInlineHitTestSource<'r, ()> for ResidentNativeHitTestSource<'_> {
    fn IsInLayoutNGInlineFormattingContext(&self) -> bool {
        self.source.IsInLayoutNGInlineFormattingContext()
    }
    fn NeedsLayout(&self) -> bool {
        self.source.NeedsLayout()
    }
    fn HasSelfPaintingLayer(&self) -> bool {
        self.source.HasSelfPaintingLayer()
    }
    fn CanTraversePhysicalFragments(&self) -> bool {
        self.source.CanTraversePhysicalFragments()
    }
    fn IsFragmented(&self) -> bool {
        self.source.IsFragmented()
    }
    fn VisibleToHitTestRequest(&self, request: &HitTestResultRequest<'r, ()>) -> bool {
        self.source
            .VisibleToHitTestRequest(request.IgnorePointerEventsNone())
    }
    fn IsDescendantOfParentBlock(&self, parent: &dyn InlineHitTestParentCursor) -> bool {
        let native_parent = self
            .inline_parent
            .expect("culled inline retains its parent cursor");
        assert_eq!(parent.Identity(), native_parent.Identity());
        self.source.IsDescendantOfInlineParent(&native_parent.0)
    }
    fn SameLayoutObjectItems(
        &self,
    ) -> Box<dyn Iterator<Item = Box<dyn InlineHitTestItem<'r, ()> + '_>> + '_> {
        Box::new(self.source.SameLayoutObjectItems().map(|item| {
            Box::new(ResidentInlineItem {
                item,
                root: self.root,
                services: self.services,
            }) as Box<dyn InlineHitTestItem<'r, ()>>
        }))
    }
    fn CulledInlineItems<'s>(
        &'s self,
        parent: &'s dyn InlineHitTestParentCursor,
    ) -> Box<dyn Iterator<Item = Box<dyn InlineHitTestItem<'r, ()> + 's>> + 's> {
        let native_parent = self
            .inline_parent
            .expect("culled inline retains its parent cursor");
        assert_eq!(parent.Identity(), native_parent.Identity());
        Box::new(self.source.CulledInlineItems(&native_parent.0).map(|item| {
            Box::new(ResidentInlineItem {
                item,
                root: self.root,
                services: self.services,
            }) as Box<dyn InlineHitTestItem<'r, ()>>
        }))
    }
    fn NewRegion(&self) -> Box<dyn InlineHitTestRegion> {
        self.services.NewInlineHitTestRegion()
    }
    fn GetNode(&self) -> Option<NodeRef> {
        self.source
            .NodeId()
            .and_then(|id| self.services.ResolveNode(id))
    }
}

/// Recreate only the source cursor position over the same resident item span.
/// The translated LayoutInline body retains region/target/list semantics.
pub fn HitTestCulledInline(
    root: PaintFragmentCursor<'_>,
    services: &dyn ResidentHitTestServices<()>,
    target: &dyn CulledInlineLayout,
    result: &mut HitTestResult<'_, ()>,
    location: &HitTestLocation,
    offset: PhysicalOffset,
    cursor: InlineCursor<'_, ()>,
) -> NativeHitTestResult {
    let object = (target as &dyn std::any::Any)
        .downcast_ref::<layoutng_assembly::internal::layout_object::LayoutObject>()
        .ok_or(NativeHitTestRequiredPort::MissingLayoutObject)?;
    let parent_item = match cursor.Current() {
        Some(FragmentItemRef::Resident(item)) => item,
        _ => return Err(NativeHitTestRequiredPort::Inline),
    };
    let container = match parent_item.Cursor().Cursor() {
        FragmentCursor::Inline { container, .. } => container,
        FragmentCursor::Physical { .. } => return Err(NativeHitTestRequiredPort::Inline),
    };
    let mut parent = layoutng_assembly::inline_cursor::InlineCursor::from_fragment(container);
    if let Some(index) = cursor.parent {
        let items = unsafe { container.Items().as_ref() }
            .expect("culled-inline parent retains FragmentItems");
        parent.MoveToFragmentItem(&items.Items()[index]);
        parent = parent.CursorForDescendants();
    }
    let parent = ResidentInlineParent(parent);
    let mut source =
        ResidentNativeHitTestSource::new(root, NativeHitTestCursor::ForObject(object), services)?;
    source.inline_parent = Some(&parent);
    Ok(layout_inline::HitTestCulledInline(
        &source, result, location, offset, &parent,
    ))
}
impl<'r> NativeLayoutHitTestSource<'r> for ResidentNativeHitTestSource<'_> {
    fn SpecialNodeAtPoint(
        &self,
        kind: Kind,
        result: &mut HitTestResult<'r, ()>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
        phase: HitTestPhase,
    ) -> NativeHitTestResult {
        if kind == Kind::SvgRoot {
            return native_hit_test::ReplacedNodeAtPoint(self, result, location, offset, phase);
        }
        debug_assert_eq!(offset, PhysicalOffset::default());
        super::svg_dispatch::NodeAtPoint(
            self.source,
            self.root,
            self.services,
            result,
            location,
            phase,
        )
    }
    fn Kind(&self) -> Kind {
        self.source.Kind()
    }
    fn Fragment(&self) -> Option<&dyn PhysicalFragment<()>> {
        self.fragment.as_ref().map(|fragment| fragment as _)
    }
    fn MayIntersect(
        &self,
        result: &HitTestResult<'r, ()>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
    ) -> bool {
        MayIntersect(self.source, result, location, offset)
    }
    fn Replaced(&self) -> &dyn ReplacedHitTestSource<'r> {
        self
    }
    fn Inline(&self) -> Result<&dyn LayoutInlineHitTestSource<'r, ()>, NativeHitTestRequiredPort> {
        Ok(self)
    }
    fn TextControl(&self) -> Result<&dyn TextControlHitTestSource<'r>, NativeHitTestRequiredPort> {
        self.text_control
            .as_ref()
            .map(|_| self as &dyn TextControlHitTestSource<'r>)
            .ok_or(NativeHitTestRequiredPort::TextControl)
    }
    fn Embedded(&self) -> Result<&dyn EmbeddedContentHitTestSource<'r>, NativeHitTestRequiredPort> {
        self.embedded_owner
            .as_ref()
            .map(|_| self as &dyn EmbeddedContentHitTestSource<'r>)
            .ok_or(NativeHitTestRequiredPort::EmbeddedContent)
    }
}
impl<'r> EmbeddedContentHitTestSource<'r> for ResidentNativeHitTestSource<'_> {
    fn Replaced(&self) -> &dyn ReplacedHitTestSource<'r> {
        self
    }
    fn ChildFrameView(&self) -> Result<ChildFrameView<'_>, NativeHitTestRequiredPort> {
        self.services.EmbeddedChildFrameView(
            self.source
                .EmbeddedContentOwnerNode()
                .expect("embedded content retains its original node owner"),
        )
    }
    fn MatchesStopNode(&self, result: &HitTestResult<'r, ()>) -> bool {
        result
            .GetHitTestRequest()
            .GetStopNode()
            .is_some_and(|stop| stop.Identity() == self.source.Identity())
    }
    fn PointOverResizer(
        &self,
        touch: bool,
        location: &HitTestLocation,
        offset: PhysicalOffset,
    ) -> bool {
        self.source
            .PointOverResizer(*location.Point() - offset, touch)
    }
    fn PhysicalContentBoxRect(&self) -> PhysicalRect {
        self.source.PhysicalContentBoxRect()
    }
    fn OwnerNode(&self) -> Option<NodeRef> {
        self.embedded_owner.clone()
    }
}
impl<'r> TextControlHitTestSource<'r> for ResidentNativeHitTestSource<'_> {
    fn BlockNodeAtPoint(
        &self,
        result: &mut HitTestResult<'r, ()>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
        phase: HitTestPhase,
    ) -> bool {
        // Qualified LayoutBlockFlow::NodeAtPoint uses LayoutBlock's translated
        // MayIntersect and first physical fragment, never virtual redispatch.
        native_hit_test::BlockNodeAtPoint(self, result, location, offset, phase)
    }
    fn ControlNode(&self) -> Option<NodeRef> {
        Some(
            self.text_control
                .as_ref()
                .expect("text control source")
                .control
                .clone(),
        )
    }
    fn InnerEditorNode(&self) -> NodeRef {
        self.text_control
            .as_ref()
            .expect("text control source")
            .editor
            .clone()
    }
    fn ContainerNode(&self) -> Option<NodeRef> {
        self.text_control
            .as_ref()
            .expect("text control source")
            .container
            .clone()
    }
    fn IsDescendantOfInnerEditor(&self, node: &NodeRef) -> bool {
        node.NativeNodeOwner()
            .and_then(|owner| unsafe { owner.Get().as_ref() })
            .is_some_and(|node| self.source.HitNodeIsDescendantOfInnerEditor(node))
    }
    fn InnerEditorOffset(&self) -> Option<PhysicalOffset> {
        self.source.InnerEditorOffset()
    }
}
impl<'r> ReplacedHitTestSource<'r> for ResidentNativeHitTestSource<'_> {
    fn IsInSelfHitTestingPhase(&self, phase: HitTestPhase) -> bool {
        if self.source.Kind() == Kind::SvgRoot {
            return phase == HitTestPhase::kSelfBlockBackground;
        }
        phase == HitTestPhase::kForeground
            || (phase == HitTestPhase::kSelfBlockBackground
                && self.source.IsReplacedElementRespectingCSSOverflow())
    }
    fn HasSelfPaintingLayer(&self) -> bool {
        self.source.HasSelfPaintingLayer()
    }
    fn HasBorderShape(&self) -> bool {
        self.source.Style().HasBorderShape()
    }
    fn HasBorderRadius(&self) -> bool {
        self.source.Style().HasBorderRadius()
    }
    fn MatchesStopNode(&self, stop: &dyn HitTestStopNode<()>) -> bool {
        stop.Identity() == self.source.Identity()
    }
    fn ChildPaintBlockedByDisplayLock(&self) -> bool {
        self.source.ChildPaintBlockedByDisplayLock()
    }
    fn ShouldClipOverflowAlongEitherAxis(&self) -> bool {
        self.source.ShouldClipOverflowAlongEitherAxis()
    }
    fn OverflowClipRect(&self) -> PhysicalRect {
        self.source.OverflowClipRect()
    }
    fn ExpandBorderShapeRectForOverflowClipMargin(&self, rect: &mut PhysicalRect) {
        self.source.ExpandBorderShapeRectForOverflowClipMargin(rect)
    }
    fn BorderShapeOuterPath(&self, rect: PhysicalRect) -> Box<dyn PathHitTest + '_> {
        self.services.BorderShapeOuterPath(self.PaintCursor(), rect)
    }
    fn HitTestOverflowControl(
        &self,
        result: &mut HitTestResult<'r, ()>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
    ) -> bool {
        self.services
            .HitTestOverflowControl(self.PaintCursor(), result, location, offset)
    }
    fn MayIntersect(
        &self,
        result: &HitTestResult<'r, ()>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
    ) -> bool {
        MayIntersect(self.source, result, location, offset)
    }
    fn VisibleToHitTestRequest(&self, ignore_pointer_events_none: bool) -> bool {
        self.source
            .VisibleToHitTestRequest(ignore_pointer_events_none)
    }
    fn IsReplacedElementRespectingCSSOverflow(&self) -> bool {
        self.source.IsReplacedElementRespectingCSSOverflow()
    }
    fn StitchedSize(&self) -> PhysicalSize {
        self.source.StitchedSize()
    }
    fn BorderBoxRect(&self) -> PhysicalRect {
        self.source.PhysicalBorderBoxRect()
    }
    fn VisualOverflowRectIncludingFilters(&self) -> PhysicalRect {
        self.source.VisualOverflowRectIncludingFilters()
    }
    fn PixelSnappedBorder(
        &self,
        rect: PhysicalRect,
        inner: bool,
    ) -> Box<dyn ContouredRectHitTest + '_> {
        self.services
            .ContouredBorder(self.PaintCursor(), rect, inner, false)
    }
    fn NodeForHitTest(&self) -> Option<NodeRef> {
        self.source
            .NodeForHitTestId()
            .and_then(|id| self.services.ResolveNode(id))
    }
    // Source layout_replaced.cc:214-237: walk the live LayoutObject siblings,
    // skip self-painting children and add only a LayoutBox's physical location.
    fn HitTestChildren(
        &self,
        result: &mut HitTestResult<'r, ()>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
        phase: HitTestPhase,
    ) -> NativeHitTestResult {
        if self.source.Kind() == Kind::SvgRoot {
            return super::svg_dispatch::RootHitTestChildren(
                self.source,
                self.root,
                self.services,
                result,
                location,
                offset,
                phase,
            );
        }
        for child in self.source.ReverseChildren() {
            if child.HasSelfPaintingLayer() {
                continue;
            }
            let child_offset = offset + child.PhysicalLocation().unwrap_or_default();
            if DispatchNativeNodeAtPoint(
                self.root,
                child,
                self.services,
                result,
                location,
                child_offset,
                phase,
            )? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

/// Recursive virtual dispatch from the live native sibling chain.
pub fn DispatchNativeNodeAtPoint(
    root: PaintFragmentCursor<'_>,
    cursor: NativeHitTestCursor<'_>,
    services: &dyn ResidentHitTestServices<()>,
    result: &mut HitTestResult<'_, ()>,
    location: &HitTestLocation,
    offset: PhysicalOffset,
    phase: HitTestPhase,
) -> NativeHitTestResult {
    let source = ResidentNativeHitTestSource::new(root, cursor, services)?;
    native_hit_test::NodeAtPoint(&source, result, location, offset, phase)
}

fn SourceCursor(
    cursor: PaintFragmentCursor<'_>,
) -> Result<NativeHitTestCursor<'_>, NativeHitTestRequiredPort> {
    let object = match cursor.Cursor() {
        FragmentCursor::Physical { fragment, .. } => fragment.GetSelfOrContainerLayoutObject(),
        FragmentCursor::Inline { item, .. } => item.GetLayoutObject(),
    };
    unsafe { object.as_ref() }
        .map(NativeHitTestCursor::ForObject)
        .ok_or(NativeHitTestRequiredPort::MissingLayoutObject)
}

/// LayoutObject::NodeAtPoint's virtual entry using the canonical owner selected
/// by the resident fragment. Errors must be handled as missing integration,
/// never converted to a successful no-hit result.
pub fn LegacyNodeAtPoint(
    cursor: PaintFragmentCursor<'_>,
    services: &dyn ResidentHitTestServices<()>,
    result: &mut HitTestResult<'_, ()>,
    location: &HitTestLocation,
    offset: PhysicalOffset,
    phase: HitTestPhase,
) -> NativeHitTestResult {
    let source = ResidentNativeHitTestSource::new(cursor, SourceCursor(cursor)?, services)?;
    native_hit_test::NodeAtPoint(&source, result, location, offset, phase)
}

/// LayoutObject::HitTestAllPhases, including LayoutBox's MayIntersect gate.
pub fn LegacyHitTestAllPhases(
    cursor: PaintFragmentCursor<'_>,
    services: &dyn ResidentHitTestServices<()>,
    result: &mut HitTestResult<'_, ()>,
    location: &HitTestLocation,
    offset: PhysicalOffset,
) -> NativeHitTestResult {
    let source = ResidentNativeHitTestSource::new(cursor, SourceCursor(cursor)?, services)?;
    native_hit_test::HitTestAllPhases(&source, result, location, offset)
}
