#![allow(non_snake_case)]

use foundation::{DynamicTo, UnsupportedLayout};
use layoutng_style::style::anonymous_style::CreateAnonymousStyleBuilderWithDisplay;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::computed_style_constants::PseudoId;

use super::layout_input::NodeKind;
use super::layout_node_metadata::Element;
use super::layout_object::{LayoutObject, OverflowRecalcType};
use super::layout_pass_scope::LayoutObjectFactoryScope;
use super::layout_view::LayoutView;

impl LayoutObject {
    // cpp: layoutng/internal/layout_object_hot.cc:20-36
    pub fn MarkContainerChainForOverflowRecalcIfNeeded(
        &mut self,
        mark_container_chain_scrollable_overflow_recalc: bool,
    ) {
        self.CheckIsNotDestroyed();
        let mut object: *mut LayoutObject = self;
        loop {
            let current = unsafe { &mut *object };
            object = if current.IsTableCell() || current.IsTableRow() {
                current.Parent()
            } else {
                current.Container()
            };
            if object.is_null() {
                break;
            }
            if mark_container_chain_scrollable_overflow_recalc {
                let ancestor = unsafe { &mut *object };
                if ancestor.ChildNeedsScrollableOverflowRecalc() {
                    return;
                }
                ancestor.SetChildNeedsScrollableOverflowRecalc();
            }
        }
    }

    // cpp: layoutng/internal/layout_object.h:2879-2880
    pub fn SetNeedsOverflowRecalc(&mut self) {
        self.SetNeedsOverflowRecalcWithType(OverflowRecalcType::kLayoutAndVisualOverflowRecalc);
    }

    // cpp: layoutng/internal/layout_object_hot.cc:40-67
    pub fn SetNeedsOverflowRecalcWithType(&mut self, overflow_recalc_type: OverflowRecalcType) {
        self.CheckIsNotDestroyed();
        let mark_container_chain_scrollable_overflow_recalc =
            !self.SelfNeedsScrollableOverflowRecalc();
        if overflow_recalc_type == OverflowRecalcType::kLayoutAndVisualOverflowRecalc {
            self.SetSelfNeedsScrollableOverflowRecalc();
        }
        debug_assert!(
            overflow_recalc_type == OverflowRecalcType::kOnlyVisualOverflowRecalc
                || overflow_recalc_type == OverflowRecalcType::kLayoutAndVisualOverflowRecalc
        );
        if mark_container_chain_scrollable_overflow_recalc {
            self.MarkContainerChainForOverflowRecalcIfNeeded(
                overflow_recalc_type == OverflowRecalcType::kLayoutAndVisualOverflowRecalc,
            );
        }
    }

    // cpp: layoutng/internal/layout_object_hot.cc:69-128
    pub fn PropagateStyleToAnonymousChildren(&mut self) {
        self.CheckIsNotDestroyed();
        let mut child = self.SlowFirstChild();
        while !child.is_null() {
            let child_ref = unsafe { &mut *child };
            if !child_ref.IsAnonymous()
                || child_ref.StyleRef().StyleType() != PseudoId::kPseudoIdNone
                || child_ref.AnonymousHasStylePropagationOverride()
            {
                child = child_ref.NextSibling();
                continue;
            }

            let mut new_style_builder = CreateAnonymousStyleBuilderWithDisplay(
                self.StyleRef(),
                child_ref.StyleRef().Display(),
                self.StyleRef().AppliedTextDecorationData(),
            );
            if child_ref.IsLayoutTextCombine() {
                let factories = LayoutObjectFactoryScope::Objects();
                let update = if factories.is_null() {
                    None
                } else {
                    unsafe { &*factories }.update_anonymous_text_combine_style
                };
                let Some(update) = update else {
                    std::panic::panic_any(UnsupportedLayout::new(
                        "inline layout module is not installed",
                    ));
                };
                update(child_ref, &mut new_style_builder);
            }
            self.UpdateAnonymousChildStyle(child, &mut new_style_builder);
            child_ref.SetStyle(new_style_builder.TakeStyle());
            child = child_ref.NextSibling();
        }

        let pseudo_id = self.StyleRef().StyleType();
        if pseudo_id == PseudoId::kPseudoIdNone {
            return;
        }
        if pseudo_id == PseudoId::kPseudoIdMarker && self.StyleRef().ContentBehavesAsNormal() {
            return;
        }
        let mut child = self.NextInPreOrder(self);
        while !child.is_null() {
            let child_ref = unsafe { &mut *child };
            if !child_ref.IsAnonymous() {
                child = child_ref.NextInPreOrderAfterChildren(self);
                continue;
            }
            if child_ref.IsText() || child_ref.IsQuote() || child_ref.IsImage() {
                child_ref.SetPseudoElementStyle(self);
            }
            child = child_ref.NextInPreOrder(self);
        }
    }

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

    // cpp: layoutng/internal/layout_object_hot.cc:145-155
    pub fn ClearNeedsLayoutWithoutPaintInvalidation(&mut self) {
        self.CheckIsNotDestroyed();
        self.SetEverHadLayout();
        self.SetSelfNeedsFullLayout(false);
        if !self.ChildLayoutBlockedByDisplayLock() {
            self.SetChildNeedsFullLayout(false);
            self.SetNeedsSimplifiedLayout(false);
        }
        self.SetScrollAnchorDisablingStyleChanged(false);
        self.SetShouldSkipLayoutCache(false);
    }

    // cpp: layoutng/internal/layout_object_hot.cc:157-160
    pub fn ClearNeedsLayout(&mut self) {
        self.CheckIsNotDestroyed();
        self.ClearNeedsLayoutWithoutPaintInvalidation();
    }

    // cpp: layoutng/internal/layout_object_hot.cc:162-165
    pub fn ClearNeedsLayoutWithFullPaintInvalidation(&mut self) {
        self.CheckIsNotDestroyed();
        self.ClearNeedsLayoutWithoutPaintInvalidation();
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

    // cpp: layoutng/internal/layout_object_hot.cc:191-197
    pub fn View(&self) -> *mut LayoutView {
        self.CheckIsNotDestroyed();
        let mut root: *const LayoutObject = self;
        while !unsafe { &*root }.Parent().is_null() {
            root = unsafe { &*root }.Parent();
        }
        DynamicTo::<LayoutView>(root) as *mut LayoutView
    }

    // cpp: layoutng/internal/layout_object_hot.cc:199-202
    pub fn IsCanvasLayoutSubtreeContainer(&self) -> bool {
        let element = DynamicTo::<Element>(self.GetNode());
        !element.is_null() && unsafe { &*element }.GetUsedCanvasTransform().is_some()
    }
}
