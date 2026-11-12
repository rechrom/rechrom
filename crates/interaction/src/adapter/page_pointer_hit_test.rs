//! Call-scoped PointerEventManager geometry leaves. Every query releases its
//! Layout/Paint borrows before dispatch can reenter script or layout.
use super::page_cursor::{
    PreparedLocalChildFrameQueries, ResidentDOMCursorNodes, ResidentDOMNodeOwners,
};
use crate::ownership::InteractionDocument;
use foundation::{gfx, LayoutUnit, PhysicalOffset, PhysicalRect, PhysicalSize, TouchAction};
use interaction_hit_test::{
    core::{
        layout::{
            hit_test_location::HitTestLocation,
            hit_test_request::RequestType,
            hit_test_result::{HitTestResult, HitTestResultRequest, NodeRef},
        },
        page::touch_adjustment::{self, TouchAdjustmentCandidateType, TouchAdjustmentContext},
    },
    resident::{
        query_services::{ResidentQueryError, ResidentQueryServices, ResidentQueryViewport},
        ResidentLayer, ResidentLayerServices,
    },
};
use layoutng_assembly::layout_engine::{LayoutOutput, PaintFragmentCursor};

pub type PointerHitResult = HitTestResult<'static, ()>;
pub struct PointerHit {
    pub location: HitTestLocation,
    pub result: PointerHitResult,
}
#[derive(Clone, Copy, Debug)]
pub enum PointerGeometryError {
    Detached,
    NoLayout,
    Resident(ResidentQueryError),
    MissingTextSettings,
}
/// Settings::GetEditingBehaviorType and Platform::DefaultLocale inputs. The
/// embedding supplies its actual UI locale independently of document lang.
#[derive(Clone)]
pub struct InteractionTextSettings {
    pub smart_insert_delete_enabled: bool,
    pub editing_behavior: interaction_default::core::editing::editing_behavior::EditingBehaviorType,
    pub ui_locale: std::ffi::CString,
}
/// Page lends its resident owners for one method call and releases them before
/// this value returns to dispatch. Results retain only canonical source nodes.
pub trait PagePointerGeometry {
    fn PrepareLifecycleExceptPaint(&self) -> Result<(), PointerGeometryError>;
    /// LayoutView::HitTestNoLifecycleUpdate over the already prepared resident
    /// owners. This call must not invoke the normal lifecycle-preparing query.
    fn EventHandlerHitTestNoLifecycleUpdate(
        &self,
        location: HitTestLocation,
        request_bits: u32,
    ) -> Result<PointerHit, PointerGeometryError>;
    fn EventHandlerHitTestNode(&self, node: u64) -> Result<Option<NodeRef>, PointerGeometryError>;
    fn EventHandlerAbsoluteToLocalPoint(
        &self,
        node: u64,
        point: &HitTestLocation,
    ) -> Result<HitTestLocation, PointerGeometryError>;
    fn InteractionTextSettings(&self) -> Option<InteractionTextSettings>;
    fn ChildrenOrSiblingsAffectedByActiveOrHover(&self, node: u64) -> bool;
    fn HitTest(
        &self,
        location: HitTestLocation,
        request_bits: u32,
    ) -> Result<PointerHit, PointerGeometryError>;
    fn HitTestInLocalFrame(
        &self,
        location: HitTestLocation,
        request_bits: u32,
    ) -> Result<PointerHit, PointerGeometryError> {
        self.HitTest(location, request_bits)
    }
    fn HitTestAdjustmentRect(
        &self,
        point: gfx::PointF,
        size: PhysicalSize,
    ) -> Result<PointerHit, PointerGeometryError>;
    fn BestTouchAdjustment(
        &self,
        kind: TouchAdjustmentCandidateType,
        hit: &PointerHit,
        facts: &dyn PointerAdjustmentDOMFacts,
    ) -> Result<Option<(u64, gfx::Point)>, PointerGeometryError>;
    fn EffectiveTouchAction(&self, node: u64, stylus: bool) -> TouchAction;
    fn DeviceScaleFactor(&self) -> f32;
    fn PageScaleFactor(&self) -> f32;
    fn GetHitTestRectForAdjustment(&self, width: f32, height: f32) -> PhysicalSize;
}
/// Source DOM virtuals. Listener interest and focusability are supplied by
/// their existing owners; geometry and adjustment policy are not delegated.
pub trait PointerAdjustmentDOMFacts {
    fn WillRespondToMouseClickEvents(&self, node: u64) -> bool;
    fn WillRespondToMouseMoveEvents(&self, node: u64) -> bool;
    fn IsMouseFocusable(&self, node: u64) -> bool;
    fn ChildrenOrSiblingsAffectedByActiveOrHover(&self, node: u64) -> bool;
}
pub struct PreparedPointerHitTest<'a> {
    pub document: &'a InteractionDocument,
    pub layout: LayoutOutput<'a>,
    pub fragments: PaintFragmentCursor<'a>,
    pub paint: &'a paint::PaintEngine,
    pub paths: &'a dyn paint::path_hit_test::PaintPathHitTestFactory,
    pub child_frames: &'a dyn PreparedLocalChildFrameQueries,
    pub viewport: &'a dyn ResidentQueryViewport,
    pub text_settings: Option<&'a InteractionTextSettings>,
    pub native_paths: std::sync::Arc<dyn foundation::native_path::NativePathFactory>,
    pub editing_positions:
        std::rc::Rc<dyn super::resident_editing_positions::ResidentEditingPositionQueries>,
}
impl PreparedPointerHitTest<'_> {
    /// Source EventHandler capture and link projection reuse the same resident
    /// DOM resolver as ordinary hit testing, including native node ownership.
    pub fn EventHandlerHitTestNode(&self, id: u64) -> Option<NodeRef> {
        ResidentDOMCursorNodes::new(
            self.document.clone(),
            self.layout,
            self.fragments,
            self.child_frames,
            self.editing_positions.clone(),
            self.native_paths.clone(),
        )
        .NodeForId(id)
    }
    /// event_handler.cc:2654-2657: a target without a LayoutObject keeps the
    /// document point. This is the source fallback, not failed geometry.
    pub fn EventHandlerAbsoluteToLocalPoint(
        &self,
        id: u64,
        point: &HitTestLocation,
    ) -> HitTestLocation {
        HitTestLocation::FromPhysicalOffset(self.layout.Node(id).map_or(*point.Point(), |node| {
            node.AbsoluteToLocalPoint(point.Point())
        }))
    }
    /// The point is already in root-frame LayoutUnit coordinates, as produced
    /// by TransformWebPointerEvent. No CSS/physical conversion is repeated.
    pub fn HitTest(
        &self,
        location: HitTestLocation,
        request_bits: u32,
    ) -> Result<PointerHit, ResidentQueryError> {
        self.HitTestWithFramePolicy(location, request_bits, true)
    }
    /// LayoutView point queries stay in their originating frame. EventHandler
    /// pointer routing separately enables kAllowChildFrameContent above.
    pub fn HitTestInLocalFrame(
        &self,
        location: HitTestLocation,
        request_bits: u32,
    ) -> Result<PointerHit, ResidentQueryError> {
        self.HitTestWithFramePolicy(location, request_bits, false)
    }
    fn HitTestWithFramePolicy(
        &self,
        location: HitTestLocation,
        request_bits: u32,
        allow_child_frame_content: bool,
    ) -> Result<PointerHit, ResidentQueryError> {
        let order = self
            .paint
            .ResidentLayerOrder(
                self.fragments,
                paint::resident_hit_test::ResidentLayerSelection {
                    negative_z: true,
                    normal_flow: true,
                    positive_z: true,
                },
                true,
            )
            .ok_or(ResidentQueryError::MissingLayerOrder)?;
        drop(order);
        let nodes = ResidentDOMCursorNodes::new(
            self.document.clone(),
            self.layout,
            self.fragments,
            self.child_frames,
            self.editing_positions.clone(),
            self.native_paths.clone(),
        );
        let services = ResidentQueryServices::new(
            self.paint,
            self.fragments,
            &nodes,
            self.viewport,
            self.paths,
        );
        let layer = ResidentLayer::new(self.paint, self.fragments, self.fragments, &services)
            .ok_or(ResidentQueryError::MissingRootPaintLayer)?;
        // EventHandler::HitTestResultAtLocation cc:444-447.
        let request = HitTestResultRequest::<()>::new(
            request_bits
                | if allow_child_frame_content {
                    RequestType::kAllowChildFrameContent as u32
                } else {
                    0
                },
            None,
            None,
        );
        let mut result = PointerHitResult::new(&request, &location);
        let area = PhysicalRect::new(PhysicalOffset::default(), self.fragments.Cursor().Size());
        interaction_hit_test::core::paint::paint_layer_hit_test::HitTest(
            &layer,
            &location,
            &mut result,
            &area,
        );
        if let Some(error) = services.QueryError() {
            return Err(error);
        }
        Ok(PointerHit { location, result })
    }
    // Source pointer_event_manager.cc:516-522.
    pub fn HitTestAdjustmentRect(
        &self,
        point: gfx::PointF,
        size: PhysicalSize,
    ) -> Result<PointerHit, ResidentQueryError> {
        let point = PhysicalOffset::new(
            LayoutUnit::FromDoubleRound(point.x() as f64),
            LayoutUnit::FromDoubleRound(point.y() as f64),
        );
        let rect = PhysicalRect::new(
            PhysicalOffset::new(point.left - size.width / 2, point.top - size.height / 2),
            size,
        );
        self.HitTestInLocalFrame(
            HitTestLocation::FromPhysicalRect(rect),
            RequestType::kTouchEvent as u32
                | RequestType::kReadOnly as u32
                | RequestType::kActive as u32
                | RequestType::kListBased as u32,
        )
    }
    pub fn BestTouchAdjustment(
        &self,
        kind: TouchAdjustmentCandidateType,
        hit: &PointerHit,
        facts: &dyn PointerAdjustmentDOMFacts,
    ) -> Result<Option<(u64, gfx::Point)>, PointerGeometryError> {
        // EventHandler::BestNodeForHitTestResult, cc:1826-1867. Overflow
        // controls keep their original touch point so they remain draggable.
        assert!(hit.location.IsRectBasedTest());
        if kind != TouchAdjustmentCandidateType::kContextMenu
            && (hit.result.GetScrollbar().is_some() || hit.result.IsOverResizer())
        {
            return Ok(None);
        }
        if hit.location.ToEnclosingRect().IsEmpty() {
            return Ok(None);
        }
        if kind == TouchAdjustmentCandidateType::kContextMenu && self.text_settings.is_none() {
            return Err(PointerGeometryError::MissingTextSettings);
        }
        let nodes = ResidentDOMCursorNodes::new(
            self.document.clone(),
            self.layout,
            self.fragments,
            self.child_frames,
            self.editing_positions.clone(),
            self.native_paths.clone(),
        );
        let context = AdjustmentContext {
            query: self,
            nodes: &nodes,
            facts,
        };
        let candidates: Vec<_> = hit.result.ListBasedTestResult().iter().cloned().collect();
        let Some((node, point)) = touch_adjustment::FindBestTouchAdjustmentCandidate(
            &context,
            kind,
            self.viewport
                .LocalFramePointToRootFrame(hit.location.RoundedPoint()),
            self.viewport
                .LocalFrameRectToRootFrame(hit.location.ToEnclosingRect()),
            &candidates,
        ) else {
            return Ok(None);
        };
        Ok(node.DOMNodeId().map(|id| (id, point)))
    }
    // Source touch_action_util.cc:14-59.
    pub fn EffectiveTouchAction(&self, node: u64, stylus: bool) -> TouchAction {
        let Some(node) = self.layout.Node(node) else {
            return TouchAction::kAuto;
        };
        let (mut action, _, _) = node.PointerTouchActionFacts();
        if action & TouchAction::kPanX != TouchAction::kNone
            && node.PointerHasHorizontallyScrollableAncestor()
        {
            action |= TouchAction::kInternalPanXScrolls;
        }
        if action & TouchAction::kPan != TouchAction::kNone
            && (!stylus || action & TouchAction::kPan != TouchAction::kPan)
        {
            action |= TouchAction::kInternalNotWritable;
        }
        action
    }
}
struct AdjustmentContext<'a, 'b> {
    query: &'a PreparedPointerHitTest<'b>,
    nodes: &'a ResidentDOMCursorNodes<'b>,
    facts: &'a dyn PointerAdjustmentDOMFacts,
}
impl AdjustmentContext<'_, '_> {
    fn Read<T>(&self, node: &NodeRef, read: impl FnOnce(&dom::Document, usize) -> T) -> Option<T> {
        let owner = self.query.document.borrow();
        let document = owner.GetDocument();
        Some(read(document, document.FindNodeById(node.DOMNodeId()?)?))
    }
    fn Parent(&self, node: &NodeRef) -> Option<NodeRef> {
        let id = self.Read(node, |d, i| {
            d.Node(i)
                .Parent()
                .or_else(|| d.ShadowHost(i))
                .map(|i| d.Node(i).Id())
        })??;
        self.nodes.NodeForId(id)
    }
    fn Layout<'a>(
        &'a self,
        node: &NodeRef,
    ) -> Option<layoutng_assembly::layout_engine::LayoutNode<'a>> {
        self.query.layout.Node(node.DOMNodeId()?)
    }
}
impl TouchAdjustmentContext for AdjustmentContext<'_, '_> {
    type Node = NodeRef;
    fn Identity(&self, node: &NodeRef) -> u128 {
        node.DOMIdentity()
    }
    fn ParentOrShadowHost(&self, node: &NodeRef) -> Option<NodeRef> {
        self.Parent(node)
    }
    fn ParentShadowHostOrOwner(&self, node: &NodeRef) -> Option<NodeRef> {
        self.Parent(node)
    }
    fn WillRespondToMouseClickEvents(&self, node: &NodeRef) -> bool {
        node.DOMNodeId()
            .is_some_and(|id| self.facts.WillRespondToMouseClickEvents(id))
    }
    fn WillRespondToMouseMoveEvents(&self, node: &NodeRef) -> bool {
        node.DOMNodeId()
            .is_some_and(|id| self.facts.WillRespondToMouseMoveEvents(id))
    }
    fn IsElement(&self, node: &NodeRef) -> bool {
        self.Read(node, |d, i| d.Node(i).Type() == dom::DOMNodeType::kElement)
            .unwrap_or(false)
    }
    fn IsIFrameElement(&self, node: &NodeRef) -> bool {
        self.Read(node, |d, i| d.Node(i).IsHTMLElement("iframe"))
            .unwrap_or(false)
    }
    fn IsMouseFocusable(&self, node: &NodeRef) -> bool {
        node.DOMNodeId()
            .is_some_and(|id| self.facts.IsMouseFocusable(id))
    }
    fn ChildrenOrSiblingsAffectedByActiveOrHover(&self, node: &NodeRef) -> bool {
        node.DOMNodeId()
            .is_some_and(|id| self.facts.ChildrenOrSiblingsAffectedByActiveOrHover(id))
    }
    fn ComputedStyleAffectedByActiveOrHover(&self, node: &NodeRef) -> bool {
        self.Layout(node).is_some_and(|n| {
            let (_, active, hover) = n.PointerTouchActionFacts();
            active || hover
        })
    }
    fn EffectiveTouchAction(&self, node: &NodeRef) -> TouchAction {
        self.Layout(node)
            .map_or(TouchAction::kAuto, |n| n.PointerTouchActionFacts().0)
    }
    fn ContextMenuFacts(&self, node: &NodeRef) -> touch_adjustment::ContextMenuNodeFacts {
        // touch_adjustment.cc:126-168,214-259. Native LayoutSelection retains
        // the source selected text range, including first-letter fragments.
        let (is_text, is_shadow_root, is_link) = self
            .Read(node, |document, index| {
                (
                    document.Node(index).Type() == dom::DOMNodeType::kText,
                    document.ShadowHost(index).is_some(),
                    document.IsLinkForEditing(index),
                )
            })
            .unwrap_or((false, false, false));
        let layout = self.Layout(node);
        let facts = layout.as_ref().map(|node| node.HitTestFacts());
        let selected = layout.as_ref().is_some_and(|node| node.IsSelected());
        touch_adjustment::ContextMenuNodeFacts {
            is_text,
            is_shadow_root,
            is_layout_box: facts.as_ref().is_some_and(|facts| facts.box_),
            is_link,
            is_image_media_or_svg_image: facts
                .as_ref()
                .is_some_and(|facts| facts.image || facts.media || facts.svg_image),
            can_be_selection_leaf: layout
                .as_ref()
                .is_some_and(|node| node.CanBeSelectionLeaf()),
            should_select_on_contextual_menu_click: self
                .query
                .text_settings
                .expect("context-menu settings admitted before source query")
                .editing_behavior
                .ShouldSelectOnContextualMenuClick(),
            is_selected: selected,
        }
    }
    fn LayoutSelectionRange(&self, node: &NodeRef) -> std::ops::Range<u32> {
        self.Layout(node)
            .expect("source selected Text retains its LayoutText")
            .LayoutSelectionRange()
    }
    fn UpdateStyleAndLayoutTree(&self, _: &NodeRef) {} // Query admission already completed this synchronous barrier.
    fn IsEditable(&self, node: &NodeRef) -> bool {
        node.IsEditable()
    }
    fn HasLayoutObject(&self, node: &NodeRef) -> bool {
        self.Layout(node).is_some()
    }
    fn AbsoluteQuads(&self, node: &NodeRef) -> Vec<gfx::QuadF> {
        self.Layout(node)
            .map_or_else(Vec::new, |n| n.AbsoluteQuads())
    }
    fn ForEachWordRange(&self, node: &NodeRef, append: &mut dyn FnMut(std::ops::Range<u32>)) {
        let owner = self.query.document.borrow();
        let document = owner.GetDocument();
        let Some(index) = node.DOMNodeId().and_then(|id| document.FindNodeById(id)) else {
            return;
        };
        let settings = self
            .query
            .text_settings
            .expect("context-menu settings admitted before source query");
        let text = document.CharacterDataUTF16(index);
        let _ = font_engine::WordBreakIterator(
            text.as_ref(),
            settings.ui_locale.as_c_str(),
            |iterator| {
                let mut last = iterator.First();
                if last == font_engine::kTextBreakDone {
                    return;
                }
                loop {
                    let next = iterator.Next();
                    if next == font_engine::kTextBreakDone {
                        break;
                    }
                    if iterator.IsWordTextBreak() {
                        append(last as u32..next as u32);
                    }
                    last = next;
                }
            },
        );
    }
    fn AbsoluteQuadsForRange(
        &self,
        node: &NodeRef,
        range: std::ops::Range<u32>,
    ) -> Vec<gfx::QuadF> {
        self.Layout(node)
            .map_or_else(Vec::new, |node| node.AbsoluteQuadsForRange(range))
    }
    fn ConvertRectToRootFrame(&self, _: &NodeRef, rect: gfx::Rect) -> gfx::Rect {
        self.query.viewport.LocalFrameRectToRootFrame(rect)
    }
    fn ConvertPointToRootFrame(&self, _: &NodeRef, point: gfx::Point) -> gfx::Point {
        self.query.viewport.LocalFramePointToRootFrame(point)
    }
    fn IsDescendantOf(&self, node: &NodeRef, ancestor: &NodeRef) -> bool {
        self.Read(node, |document, index| {
            let Some(ancestor) = ancestor
                .DOMNodeId()
                .and_then(|id| document.FindNodeById(id))
            else {
                return false;
            };
            let mut parent = document.Node(index).Parent();
            while let Some(node) = parent {
                if node == ancestor {
                    return true;
                }
                parent = document.Node(node).Parent();
            }
            false
        })
        .unwrap_or(false)
    }
    fn IsPseudoElement(&self, node: &NodeRef) -> bool {
        node.Pseudo().is_some()
    }
    fn HasActivationBehavior(&self, node: &NodeRef) -> bool {
        self.Read(node, |d, i| {
            d.PseudoElementFacts(i)
                .is_some_and(|facts| facts.HasActivationBehavior())
        })
        .unwrap_or(false)
    }
}
