#![allow(non_snake_case, non_camel_case_types)]

use foundation::{PhysicalOffset, PhysicalRect, Vector};
use layoutng::internal::map_coordinates_flags::MapCoordinatesFlags;
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;

use super::super::layout_box_model_object::LayoutBoxModelObject;
use super::super::layout_object::LayoutObject;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollectorType {
    kUnion,
    kVector,
}

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

#[derive(Default)]
pub struct UnionOutlineRectCollector {
    rect_: Option<PhysicalRect>,
}

impl UnionOutlineRectCollector {
    pub fn Rect(&self) -> PhysicalRect {
        self.rect_.unwrap_or_default()
    }
}

impl OutlineRectCollector for UnionOutlineRectCollector {
    fn GetType(&self) -> CollectorType {
        CollectorType::kUnion
    }

    fn AddRect(&mut self, rect: &PhysicalRect) {
        if let Some(current) = self.rect_.as_mut() {
            current.UniteEvenIfEmpty(rect);
        } else {
            self.rect_ = Some(*rect);
        }
    }

    fn ForDescendantCollector(&self) -> Box<dyn OutlineRectCollector> {
        Box::new(Self::default())
    }

    fn CombineWithDescendant(
        &mut self,
        collector: &mut dyn OutlineRectCollector,
        descendant: &LayoutObject,
        ancestor: *const LayoutBoxModelObject,
        post_offset: &PhysicalOffset,
    ) {
        assert_eq!(collector.GetType(), CollectorType::kUnion);
        if collector.IsEmpty() {
            return;
        }
        let collector = unsafe {
            &mut *(collector as *mut dyn OutlineRectCollector as *mut UnionOutlineRectCollector)
        };
        let mut rect = descendant.LocalToAncestorRect(
            &collector.Rect(),
            ancestor,
            MapCoordinatesFlags::default(),
        );
        rect.offset += *post_offset;
        self.AddRect(&rect);
    }

    fn CombineWithOffset(
        &mut self,
        collector: &mut dyn OutlineRectCollector,
        additional_offset: &PhysicalOffset,
    ) {
        assert_eq!(collector.GetType(), CollectorType::kUnion);
        if collector.IsEmpty() {
            return;
        }
        let collector = unsafe {
            &mut *(collector as *mut dyn OutlineRectCollector as *mut UnionOutlineRectCollector)
        };
        let mut rect = collector.Rect();
        rect.offset += *additional_offset;
        self.AddRect(&rect);
    }

    fn IsEmpty(&self) -> bool {
        self.rect_.is_none()
    }
}

#[derive(Default)]
pub struct VectorOutlineRectCollector {
    rects_: Vector<PhysicalRect>,
}

impl VectorOutlineRectCollector {
    pub fn TakeRects(&mut self) -> Vector<PhysicalRect> {
        std::mem::take(&mut self.rects_)
    }
}

impl OutlineRectCollector for VectorOutlineRectCollector {
    fn GetType(&self) -> CollectorType {
        CollectorType::kVector
    }

    fn AddRect(&mut self, rect: &PhysicalRect) {
        self.rects_.push(*rect);
    }

    fn ForDescendantCollector(&self) -> Box<dyn OutlineRectCollector> {
        Box::new(Self::default())
    }

    fn CombineWithDescendant(
        &mut self,
        collector: &mut dyn OutlineRectCollector,
        descendant: &LayoutObject,
        ancestor: *const LayoutBoxModelObject,
        post_offset: &PhysicalOffset,
    ) {
        assert_eq!(collector.GetType(), CollectorType::kVector);
        let collector = unsafe {
            &mut *(collector as *mut dyn OutlineRectCollector as *mut VectorOutlineRectCollector)
        };
        for source in collector.TakeRects() {
            let mut rect =
                descendant.LocalToAncestorRect(&source, ancestor, MapCoordinatesFlags::default());
            rect.offset += *post_offset;
            self.rects_.push(rect);
        }
    }

    fn CombineWithOffset(
        &mut self,
        collector: &mut dyn OutlineRectCollector,
        additional_offset: &PhysicalOffset,
    ) {
        assert_eq!(collector.GetType(), CollectorType::kVector);
        let collector = unsafe {
            &mut *(collector as *mut dyn OutlineRectCollector as *mut VectorOutlineRectCollector)
        };
        let mut rects = collector.TakeRects();
        if !additional_offset.IsZero() {
            for rect in &mut rects {
                rect.offset += *additional_offset;
            }
        }
        self.rects_.extend(rects);
    }

    fn IsEmpty(&self) -> bool {
        self.rects_.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use foundation::{LayoutUnit, PhysicalOffset, PhysicalRect, PhysicalSize};

    use super::{OutlineRectCollector, UnionOutlineRectCollector, VectorOutlineRectCollector};

    fn rect(x: i32, y: i32, width: i32, height: i32) -> PhysicalRect {
        PhysicalRect::new(
            PhysicalOffset::new(LayoutUnit::from_signed(x), LayoutUnit::from_signed(y)),
            PhysicalSize::new(
                LayoutUnit::from_signed(width),
                LayoutUnit::from_signed(height),
            ),
        )
    }

    #[test]
    fn union_collector_keeps_empty_outline_rects_and_unites_subsequent_rects() {
        let mut collector = UnionOutlineRectCollector::default();
        collector.AddRect(&rect(5, 7, 0, 0));
        assert!(!collector.IsEmpty());
        collector.AddRect(&rect(1, 2, 3, 4));
        assert_eq!(collector.Rect(), rect(1, 2, 4, 5));
    }

    #[test]
    fn vector_offset_combine_consumes_source_and_preserves_order() {
        let mut source = VectorOutlineRectCollector::default();
        source.AddRect(&rect(1, 2, 3, 4));
        source.AddRect(&rect(5, 6, 7, 8));
        let mut target = VectorOutlineRectCollector::default();
        target.CombineWithOffset(
            &mut source,
            &PhysicalOffset::new(LayoutUnit::from_signed(10), LayoutUnit::from_signed(20)),
        );
        assert!(source.IsEmpty());
        assert_eq!(target.rects_[0], rect(11, 22, 3, 4));
        assert_eq!(target.rects_[1], rect(15, 26, 7, 8));
    }
}
