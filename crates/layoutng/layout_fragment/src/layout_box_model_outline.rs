//! LayoutBoxModelObject outline traversal over retained LayoutObject geometry.

#![allow(non_snake_case)]

use foundation::{DynamicTo, PhysicalOffset};
use layoutng_style::style::outline_type::OutlineType;

use super::super::layout_box::LayoutBox;
use super::super::layout_box_model_object::LayoutBoxModelObject;
use super::super::layout_inline::LayoutInline;
use super::super::layout_object::LayoutObject;
use super::super::outline_rect_collector::OutlineRectCollector;

impl LayoutBoxModelObject {
    // cpp: core/layout/layout_box_model_object.cc:380-395
    pub fn AddOutlineRectsForNormalChildren(
        &self,
        collector: &mut dyn OutlineRectCollector,
        additional_offset: &PhysicalOffset,
        outline_type: OutlineType,
    ) {
        self.CheckIsNotDestroyed();
        let mut child = self.SlowFirstChild();
        while !child.is_null() {
            let child_ref = unsafe { &*child };
            if !child_ref.IsOutOfFlowPositioned() {
                self.AddOutlineRectsForDescendant(
                    child_ref,
                    collector,
                    additional_offset,
                    outline_type,
                );
            }
            child = child_ref.NextSibling();
        }
    }

    // cpp: core/layout/layout_box_model_object.cc:397-441
    pub fn AddOutlineRectsForDescendant(
        &self,
        descendant: &LayoutObject,
        collector: &mut dyn OutlineRectCollector,
        additional_offset: &PhysicalOffset,
        outline_type: OutlineType,
    ) {
        self.CheckIsNotDestroyed();
        if descendant.IsText() {
            return;
        }
        if descendant.HasLayer() {
            let mut descendant_collector = collector.ForDescendantCollector();
            descendant.AddOutlineRects(
                descendant_collector.as_mut(),
                std::ptr::null_mut(),
                &PhysicalOffset::default(),
                outline_type,
            );
            collector.CombineWithDescendant(
                descendant_collector.as_mut(),
                descendant,
                self,
                additional_offset,
            );
            return;
        }
        if descendant.IsBox() {
            let descendant_box = DynamicTo::<LayoutBox>(descendant);
            debug_assert!(!descendant_box.is_null());
            let offset = *additional_offset + unsafe { &*descendant_box }.PhysicalLocation();
            descendant.AddOutlineRects(collector, std::ptr::null_mut(), &offset, outline_type);
            return;
        }
        if descendant.IsLayoutInline() {
            let descendant_inline = DynamicTo::<LayoutInline>(descendant);
            debug_assert!(!descendant_inline.is_null());
            unsafe { &*descendant_inline }.AddOutlineRectsForNormalChildren(
                collector,
                additional_offset,
                outline_type,
            );
            return;
        }
        descendant.AddOutlineRects(
            collector,
            std::ptr::null_mut(),
            additional_offset,
            outline_type,
        );
    }
}
