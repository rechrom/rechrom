//! Hit-test family: the resident Layout owners Chromium's hit testing walks
//! directly, plus the value enums its queries exchange.
//!
//! Plain facts a hit-test consumer needs from one fragment are accessors on
//! the borrowed [`PaintFragmentCursor`](crate::paint::PaintFragmentCursor)
//! (`ScrollableArea`, `OverflowClipMarginBorderOutsets`,
//! `SelfOrContainerScrollableArea`, `SelfOrContainerApplyFiltersToRect`).
//! The owner types below are re-exported deliberately and only for the
//! translated hit-test and cursor-selection code in `interaction` /
//! `interaction_hit_test`; they are not a general LayoutObject port.

// Values exchanged with hit-test queries (no Layout ownership).
/// Source core/layout/hit_test_phase.h.
pub use layout_assembly::internal::hit_test_phase::HitTestPhase;
/// Source LayoutObject::GetCursor's result (layout_object.h `CursorDirective`).
pub use layout_assembly::internal::layout_object::CursorDirective;

// Chromium reads these owners directly while hit testing:
// paint_layer.cc PaintLayer::HitTestLayer / HitTestFragmentsWithPhase walks
// PaintLayer and its LayoutBoxModelObject; layout_view.cc LayoutView::HitTest
// and HitTestNoLifecycleUpdate validate the LayoutView owner;
// box_fragment_painter.cc BoxFragmentPainter::NodeAtPoint and
// layout_inline.cc LayoutInline::HitTestCulledInline dispatch on the fragment's
// LayoutObject; hit_test_result.cc HitTestResult::Image reads the LayoutImage
// (LayoutInputReplaced) ImageResource; event_handler.cc
// EventHandler::SelectCursor / SelectAutoCursor read LayoutObject::GetCursor,
// EnclosingLayer()->GetLayoutBox(), GetScrollableArea() and LayoutBox
// geometry (AbsoluteContentQuad, PhysicalBorderBoxRect).
pub use layout_assembly::internal::layout_box::LayoutBox;
pub use layout_assembly::internal::layout_box_model_object::LayoutBoxModelObject;
pub use layout_assembly::internal::layout_object::LayoutObject;
pub use layout_assembly::internal::layout_replaced::LayoutInputReplaced;
pub use layout_assembly::internal::layout_view::LayoutView;
pub use layout_assembly::internal::paint_layer::PaintLayer;
