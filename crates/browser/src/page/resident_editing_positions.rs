//! Call-scoped assembly of native position and cursor frame queries. Results
//! retain source identities/DOM position values, never a Layout/Paint loan.
use super::*;
use foundation::{Persistent, PhysicalOffset};
use interaction::adapter::resident_editing_positions::{self, ResidentEditingPositionQueries};
use interaction::hit_test::core::layout::hit_test_result::{
    CaretPositionQuery, HitTestPositionData,
};
use layoutng_assembly::internal::layout_node_metadata::Node;
pub(super) struct PageResidentEditingPositions(pub(super) Weak<PageState>);
impl ResidentEditingPositionQueries for PageResidentEditingPositions {
    fn VisibleContentRectExcludingScrollbars(&self) -> foundation::gfx::RectF {
        let state = self.0.upgrade().expect("live cursor frame");
        if state.web_view_zoom.borrow().Settings().minimum_scale
            != state.web_view_zoom.borrow().Settings().maximum_scale
        {
            let facts = state
                .PageScaleViewportFacts()
                .expect("active viewport geometry");
            return state.web_view_zoom.borrow().VisibleRect(facts.size);
        }
        let layout = state.layout_engine.borrow();
        layout
            .Output()
            .expect("cursor layout view")
            .CursorVisibleContentRectExcludingScrollbars()
            .expect("cursor frame geometry")
    }
    fn SelectionContains(&self, point: &PhysicalOffset) -> bool {
        let state = self.0.upgrade().expect("live hit document");
        let selection = state
            .resident_frame_selection
            .borrow()
            .upgrade()
            .expect("canonical frame selection");
        let settings = state
            .interaction_text_settings
            .borrow()
            .clone()
            .expect("embedding interaction text settings");
        let has_layout_view = state.layout_engine.borrow().Output().is_some();
        selection
            .ContainsPoint(
                state.document.Handle(),
                Rc::new(super::pointer_geometry::PagePointerGeometryProvider(
                    self.0.clone(),
                )),
                settings,
                *point,
                has_layout_view,
            )
            .expect("resident FrameSelection::Contains")
    }
    fn UpdateCursorStyleAndLayout(&self) {
        self.0
            .upgrade()
            .expect("live cursor frame")
            .EnsureLayoutOutput();
    }
    fn UpdateSelectionStyleTree(&self, node: u64) {
        // Source node.cc:2038-2044; document.cc:2933-2958. Force only the
        // ancestor style/tree phase, then sync the existing attachment owner.
        let state = self.0.upgrade().expect("live selection document");
        let scope = {
            let mut owner = state.document.borrow_mut();
            let document = owner.GetDocumentMut();
            let Some(index) = document
                .FindNodeById(node)
                .filter(|&index| document.IsActive() && document.IsConnectedToDocument(index))
            else {
                return;
            };
            document.BeginNodeDisplayLockForcedScope(
                Some(index),
                dom::persistent_document::DisplayLockForcedPhase::StyleAndLayoutTree,
                false,
                false,
                true,
                false,
            )
        };
        state.ResolveStyles();
        {
            let mut layout = state.layout_engine.borrow_mut();
            let mut document = state.document.borrow_mut();
            crate::persistent_layout::SynchronizeLayoutTreeWithEngine(
                &mut layout,
                &mut document,
                &state.interaction.borrow(),
                &state.constraints.borrow(),
            );
        }
        dom::attribute_lifecycle::DrainAttributeLifecycle(&state.document.Handle());
        state
            .document
            .borrow_mut()
            .GetDocumentMut()
            .EndDisplayLockForcedUpdate(scope);
    }
    fn ContentLocalToRootAncestorPoint(
        &self,
        point: foundation::gfx::PointF,
    ) -> foundation::gfx::PointF {
        let state = self.0.upgrade().expect("live cursor frame");
        let layout = state.layout_engine.borrow();
        layout
            .Output()
            .expect("cursor layout view")
            .CursorContentLocalToRootAncestorPoint(point)
    }
    fn MapContentRectToRootAncestorSpace(
        &self,
        rect: foundation::gfx::RectF,
    ) -> foundation::gfx::RectF {
        let state = self.0.upgrade().expect("live cursor frame");
        let layout = state.layout_engine.borrow();
        layout
            .Output()
            .expect("cursor layout view")
            .CursorMapContentRectToRootAncestorSpace(rect)
    }

    fn NodeOwner(&self, node: u64) -> Option<Persistent<Node>> {
        let state = self.0.upgrade()?;
        let result = state
            .layout_engine
            .borrow()
            .CurrentLayoutNode(node)?
            .NativeNodeOwner();
        result
    }
    fn PositionForPoint(
        &self,
        node: &Persistent<Node>,
        point: &PhysicalOffset,
    ) -> Option<HitTestPositionData> {
        let state = self.0.upgrade()?;
        super::pointer_geometry::PagePointerGeometryProvider(self.0.clone())
            .Query(|query| {
                let owner = state.document.borrow();
                let settings = state.interaction_text_settings.borrow();
                let settings = settings
                    .as_ref()
                    .expect("embedding interaction text settings");
                let behavior = settings.editing_behavior;
                resident_editing_positions::QueryPositionForPoint(
                    owner.GetDocument(),
                    query.layout,
                    unsafe { &*node.Get() },
                    point,
                    layoutng_assembly::EditingPositionOptions {
                        move_caret_to_horizontal_boundary_when_past_top_or_bottom: behavior
                            .ShouldMoveCaretToHorizontalBoundaryWhenPastTopOrBottom(),
                        ..Default::default()
                    },
                    &settings.ui_locale,
                )
            })
            .expect("resident editing pre-paint query")
    }
    fn CaretPosition(&self, node: u64, kind: CaretPositionQuery) -> Option<HitTestPositionData> {
        let state = self.0.upgrade()?;
        super::pointer_geometry::PagePointerGeometryProvider(self.0.clone())
            .Query(|query| {
                let owner = state.document.borrow();
                let settings = state.interaction_text_settings.borrow();
                let settings = settings
                    .as_ref()
                    .expect("embedding interaction text settings");
                resident_editing_positions::QueryCaretPosition(
                    owner.GetDocument(),
                    query.layout,
                    node,
                    kind,
                    &settings.ui_locale,
                )
            })
            .expect("resident editing pre-paint query")
    }
}
