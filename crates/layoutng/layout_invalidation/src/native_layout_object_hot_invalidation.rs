#![allow(non_snake_case)]

use foundation::UnsupportedLayout;
use layoutng_style::style::anonymous_style::CreateAnonymousStyleBuilderWithDisplay;
use layoutng_style::style::computed_style_constants::PseudoId;

use super::super::layout_object::{LayoutObject, OverflowRecalcType};
use super::super::layout_pass_scope::LayoutObjectFactoryScope;

impl LayoutObject {
    // cpp: layoutng/internal/layout_object_hot.cc:20-36
    #[inline(never)]
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
    #[inline(never)]
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
}
