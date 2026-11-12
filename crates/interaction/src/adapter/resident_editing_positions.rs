//! HitTestResult editing-position leaves using short resident query leases.
//! Source hit_test_result.cc:203-280; editing_utilities.cc:1437-1466.
use foundation::{Persistent, PhysicalOffset};
use interaction_hit_test::core::layout::hit_test_result::{
    self as hit, CaretPositionQuery, HitTestPositionData,
};
use layoutng_assembly::{internal::layout_node_metadata::Node, layout_engine::LayoutOutput};
use std::rc::Rc;
/// The provider borrows the current LayoutOutput for each call and returns only
/// the original DOM position value. Hit nodes retain neither a LayoutOutput
/// loan nor a geometry snapshot, including across script/lifecycle reentry.
pub trait ResidentEditingPositionQueries {
    /// Resolve another live DOM/native association (for example area→image)
    /// through Layout's existing attachment index, without keeping its loan.
    fn VisibleContentRectExcludingScrollbars(&self) -> foundation::gfx::RectF;
    fn SelectionContains(&self, point: &PhysicalOffset) -> bool;
    fn UpdateCursorStyleAndLayout(&self);
    fn UpdateSelectionStyleTree(&self, node: u64);
    fn ContentLocalToRootAncestorPoint(
        &self,
        point: foundation::gfx::PointF,
    ) -> foundation::gfx::PointF;
    fn MapContentRectToRootAncestorSpace(
        &self,
        rect: foundation::gfx::RectF,
    ) -> foundation::gfx::RectF;
    fn NodeOwner(&self, node: u64) -> Option<Persistent<Node>>;
    fn PositionForPoint(
        &self,
        node: &Persistent<Node>,
        point: &PhysicalOffset,
    ) -> Option<HitTestPositionData>;
    fn CaretPosition(&self, node: u64, query: CaretPositionQuery) -> Option<HitTestPositionData>;
}
struct ResidentPosition(HitTestPositionData);
impl hit::HitTestPosition for ResidentPosition {
    fn Value(&self) -> HitTestPositionData {
        self.0
    }
}
pub(super) fn Retain(value: Option<HitTestPositionData>) -> hit::PositionWithAffinity {
    value.map(|value| Rc::new(ResidentPosition(value)) as Rc<dyn hit::HitTestPosition>)
}
pub fn QueryPositionForPoint(
    document: &dom::Document,
    layout: LayoutOutput<'_>,
    node: &Node,
    point: &PhysicalOffset,
    options: layoutng_assembly::EditingPositionOptions,
    ui_locale: &std::ffi::CStr,
) -> Option<HitTestPositionData> {
    let native = layout.PositionForNativeNode(node, point, options)?;
    let adjusted = super::page_editing_visible_position::AdjustResidentPointPosition(
        document, layout, native, ui_locale,
    )?;
    Project(document, adjusted)
}
pub fn QueryCaretPosition(
    document: &dom::Document,
    layout: LayoutOutput<'_>,
    node: u64,
    query: CaretPositionQuery,
    ui_locale: &std::ffi::CStr,
) -> Option<HitTestPositionData> {
    Project(
        document,
        super::page_editing_visible_position::ResidentHitCaretPosition(
            document, layout, node, query, ui_locale,
        )?,
    )
}
pub fn QuerySelectionContains(
    document: &dom::Document,
    layout: LayoutOutput<'_>,
    selections: &layoutng_assembly::editing_state::SelectionState,
    locale: &std::ffi::CStr,
    point: &PhysicalOffset,
    hit: impl FnOnce() -> Option<layoutng_assembly::EditingPositionWithAffinity>,
) -> bool {
    super::page_editing_visible_position::ResidentSelectionContains(
        document, layout, selections, locale, point, hit,
    )
}
fn Project(
    document: &dom::Document,
    value: layoutng_assembly::EditingPositionWithAffinity,
) -> Option<HitTestPositionData> {
    use layoutng_assembly::{EditingPositionAnchorType as Anchor, EditingTextAffinity as Affinity};
    let node = value.position.node_id?;
    let index = document.FindNodeById(node)?;
    Some(HitTestPositionData {
        document: document.NodeDocumentHandle(index).Identity(),
        node_id: Some(node),
        offset: value.position.offset,
        anchor: match value.position.anchor {
            Anchor::OffsetInAnchor => hit::HitTestPositionAnchor::OffsetInAnchor,
            Anchor::BeforeAnchor => hit::HitTestPositionAnchor::BeforeAnchor,
            Anchor::AfterAnchor => hit::HitTestPositionAnchor::AfterAnchor,
            Anchor::AfterChildren => hit::HitTestPositionAnchor::AfterChildren,
        },
        affinity: match value.affinity {
            Affinity::Upstream => hit::HitTestTextAffinity::Upstream,
            Affinity::Downstream => hit::HitTestTextAffinity::Downstream,
        },
    })
}
