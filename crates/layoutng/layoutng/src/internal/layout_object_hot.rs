#![allow(non_snake_case)]

use foundation::DynamicTo;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::computed_style_constants::PseudoId;

use super::layout_input::NodeKind;
use super::layout_node_metadata::Element;
use super::layout_object::LayoutObject;

#[path = "../../../layout_invalidation/src/native_layout_object_hot_invalidation.rs"]
mod layout_invalidation_native_layout_object_hot_invalidation;

impl LayoutObject {
    // cpp: layoutng/internal/layout_object_hot.cc:130-143
    pub fn FirstLineStyleRef(&self) -> &ComputedStyle {
        self.CheckIsNotDestroyed();
        if self.HasFirstLineStylesForLayout() {
            let owner = self.InputOwnerForLayout();
            let style = owner.InputFirstLineStyle();
            if !style.is_null() {
                return unsafe { &*style };
            }
            let parent = self.Parent();
            if owner.IsTextNode() && !parent.is_null() {
                return unsafe { &*parent }.FirstLineStyleRef();
            }
        }
        self.StyleRef()
    }

    // cpp: layoutng/internal/layout_object_hot.cc:167-172
    pub fn CanMatchSizeContainerQueries(&self) -> bool {
        self.CheckIsNotDestroyed();
        let element = DynamicTo::<Element>(self.GetNode());
        if !element.is_null() {
            // The style crate's Element forward type is the same native node
            // boundary, pending cross-crate type reconciliation.
            return self.StyleRef().CanMatchSizeContainerQueries(unsafe {
                &*(element as *const layoutng_style::style::forward::Element)
            });
        }
        false
    }

    // cpp: layoutng/internal/layout_object_hot.cc:174-177
    pub fn IsPseudoElement(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.StyleRef().StyleType() != PseudoId::kPseudoIdNone
    }

    // cpp: layoutng/internal/layout_object_hot.cc:179-182
    pub fn IsOverscrollAreaParent(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.StyleRef().StyleType() == PseudoId::kPseudoIdOverscrollAreaParent
    }

    // cpp: layoutng/internal/layout_object_hot.cc:184-189
    pub fn IsRenderedLegendCandidate(&self) -> bool {
        self.CheckIsNotDestroyed();
        let node = self.GetNode();
        !self.IsOutOfFlowPositioned()
            && !self.StyleRef().IsFloating()
            && !node.is_null()
            && unsafe { &*node }.InputKind() == NodeKind::kLegend
    }

    // cpp: layoutng/internal/layout_object_hot.cc:199-202
    pub fn IsCanvasLayoutSubtreeContainer(&self) -> bool {
        let element = DynamicTo::<Element>(self.GetNode());
        !element.is_null() && unsafe { &*element }.GetUsedCanvasTransform().is_some()
    }
}
