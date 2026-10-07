#![allow(non_snake_case, non_camel_case_types)]

use foundation::{PhysicalOffset, PhysicalRect, Vector};

use super::layout_box_model_object::LayoutBoxModelObject;
use super::layout_object::LayoutObject;

// cpp: layoutng/internal/outline_rect_collector.h:21-21
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollectorType {
    kUnion,
    kVector,
}

// cpp: layoutng/internal/outline_rect_collector.h:19-46
pub trait OutlineRectCollector {
    fn GetType(&self) -> CollectorType;
    fn AddRect(&mut self, rect: &PhysicalRect);
    fn ForDescendantCollector(&self) -> Box<dyn OutlineRectCollector>;
    fn CombineWithDescendant(
        &mut self,
        collector: &mut dyn OutlineRectCollector,
        descendant: &LayoutObject,
        ancestor: *const LayoutBoxModelObject,
        post_offset: &PhysicalOffset,
    );
    fn CombineWithOffset(
        &mut self,
        collector: &mut dyn OutlineRectCollector,
        additional_offset: &PhysicalOffset,
    );
    fn IsEmpty(&self) -> bool;
}

// cpp: layoutng/internal/outline_rect_collector.h:48-72
#[derive(Default)]
pub struct UnionOutlineRectCollector {
    rect_: Option<PhysicalRect>,
}

impl UnionOutlineRectCollector {
    // cpp: layoutng/internal/outline_rect_collector.h:55-55
    pub fn Rect(&self) -> PhysicalRect {
        self.rect_.unwrap_or_default()
    }
}

impl OutlineRectCollector for UnionOutlineRectCollector {
    // cpp: layoutng/internal/outline_rect_collector.h:52-52
    fn GetType(&self) -> CollectorType {
        CollectorType::kUnion
    }

    // cpp: layoutng/internal/outline_rect_collector.h:54-54
    fn AddRect(&mut self, rect: &PhysicalRect) {
        unsafe { UnionOutlineRectCollectorAddRect(self, rect) }
    }

    // cpp: layoutng/internal/outline_rect_collector.h:57-59
    fn ForDescendantCollector(&self) -> Box<dyn OutlineRectCollector> {
        Box::new(Self::default())
    }

    // cpp: layoutng/internal/outline_rect_collector.h:61-66
    fn CombineWithDescendant(
        &mut self,
        collector: &mut dyn OutlineRectCollector,
        descendant: &LayoutObject,
        ancestor: *const LayoutBoxModelObject,
        post_offset: &PhysicalOffset,
    ) {
        unsafe {
            UnionOutlineRectCollectorCombineWithDescendant(
                self,
                collector,
                descendant,
                ancestor,
                post_offset,
            )
        }
    }

    fn CombineWithOffset(
        &mut self,
        collector: &mut dyn OutlineRectCollector,
        additional_offset: &PhysicalOffset,
    ) {
        unsafe { UnionOutlineRectCollectorCombineWithOffset(self, collector, additional_offset) }
    }

    // cpp: layoutng/internal/outline_rect_collector.h:68-68
    fn IsEmpty(&self) -> bool {
        self.rect_.is_none()
    }
}

// cpp: layoutng/internal/outline_rect_collector.h:74-98
#[derive(Default)]
pub struct VectorOutlineRectCollector {
    rects_: Vector<PhysicalRect>,
}

impl VectorOutlineRectCollector {
    // cpp: layoutng/internal/outline_rect_collector.h:81-81
    pub fn TakeRects(&mut self) -> Vector<PhysicalRect> {
        std::mem::take(&mut self.rects_)
    }
}

impl OutlineRectCollector for VectorOutlineRectCollector {
    // cpp: layoutng/internal/outline_rect_collector.h:78-78
    fn GetType(&self) -> CollectorType {
        CollectorType::kVector
    }

    // cpp: layoutng/internal/outline_rect_collector.h:80-80
    fn AddRect(&mut self, rect: &PhysicalRect) {
        self.rects_.push(*rect);
    }

    // cpp: layoutng/internal/outline_rect_collector.h:83-85
    fn ForDescendantCollector(&self) -> Box<dyn OutlineRectCollector> {
        Box::new(Self::default())
    }

    // cpp: layoutng/internal/outline_rect_collector.h:87-92
    fn CombineWithDescendant(
        &mut self,
        collector: &mut dyn OutlineRectCollector,
        descendant: &LayoutObject,
        ancestor: *const LayoutBoxModelObject,
        post_offset: &PhysicalOffset,
    ) {
        unsafe {
            VectorOutlineRectCollectorCombineWithDescendant(
                self,
                collector,
                descendant,
                ancestor,
                post_offset,
            )
        }
    }

    fn CombineWithOffset(
        &mut self,
        collector: &mut dyn OutlineRectCollector,
        additional_offset: &PhysicalOffset,
    ) {
        unsafe { VectorOutlineRectCollectorCombineWithOffset(self, collector, additional_offset) }
    }

    // cpp: layoutng/internal/outline_rect_collector.h:94-94
    fn IsEmpty(&self) -> bool {
        self.rects_.is_empty()
    }
}

// Five concrete methods have declarations but no bodies in the supplied
// source tree. These symbols keep the contracts without inventing behavior.
unsafe extern "Rust" {
    fn UnionOutlineRectCollectorAddRect(this: &mut UnionOutlineRectCollector, rect: &PhysicalRect);
    fn UnionOutlineRectCollectorCombineWithDescendant(
        this: &mut UnionOutlineRectCollector,
        collector: &mut dyn OutlineRectCollector,
        descendant: &LayoutObject,
        ancestor: *const LayoutBoxModelObject,
        post_offset: &PhysicalOffset,
    );
    fn UnionOutlineRectCollectorCombineWithOffset(
        this: &mut UnionOutlineRectCollector,
        collector: &mut dyn OutlineRectCollector,
        additional_offset: &PhysicalOffset,
    );
    fn VectorOutlineRectCollectorCombineWithDescendant(
        this: &mut VectorOutlineRectCollector,
        collector: &mut dyn OutlineRectCollector,
        descendant: &LayoutObject,
        ancestor: *const LayoutBoxModelObject,
        post_offset: &PhysicalOffset,
    );
    fn VectorOutlineRectCollectorCombineWithOffset(
        this: &mut VectorOutlineRectCollector,
        collector: &mut dyn OutlineRectCollector,
        additional_offset: &PhysicalOffset,
    );
}
