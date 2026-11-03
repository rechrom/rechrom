#![allow(non_snake_case)]

use foundation::{gfx, LayoutUnit, Member, PhysicalOffset, PhysicalRect, PhysicalSize, Visitor};
use layoutng_geometry::geometry::axis::{
    kPhysicalAxesHorizontal, kPhysicalAxesVertical, PhysicalAxes, PhysicalAxis,
};
use layoutng_geometry::geometry::box_edge::BoxEdge;

use super::super::layout_box::LayoutBox;
use super::super::layout_box_model_object::LayoutBoxModelObject;
use super::super::layout_object::LayoutObject;

fn RectToBoxEdge(axis: PhysicalAxis, rect: &PhysicalRect) -> BoxEdge {
    if axis == PhysicalAxis::kHorizontal {
        BoxEdge::new(rect.X(), rect.Width())
    } else {
        BoxEdge::new(rect.Y(), rect.Height())
    }
}

pub struct PerAxisData {
    pub axis: PhysicalAxis,
    pub min_inset: Option<LayoutUnit>,
    pub max_inset: Option<LayoutUnit>,
    pub min_inset_for_get_computed_style: Option<LayoutUnit>,
    pub max_inset_for_get_computed_style: Option<LayoutUnit>,
    pub scroll_container_relative_containing_block_range: BoxEdge,
    pub scroll_container_relative_sticky_box_range: BoxEdge,
    pub constraining_range: BoxEdge,
    pub container: Member<LayoutObject>,
    pub sticky_container: Member<LayoutBox>,
    pub containing_scroll_container: Member<LayoutBox>,
    pub is_fixed_to_view: bool,
    pub total_sticky_box_sticky_offset: LayoutUnit,
    pub total_containing_block_sticky_offset: LayoutUnit,
    pub sticky_offset: LayoutUnit,
}

impl PerAxisData {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        axis: PhysicalAxis,
        containing_block: &PhysicalRect,
        sticky_box: &PhysicalRect,
        constraining: &PhysicalRect,
        container: *const LayoutObject,
        sticky_container: *const LayoutBox,
        containing_scroll_container: *const LayoutBox,
        is_fixed_to_view: bool,
        min_inset: Option<LayoutUnit>,
        max_inset: Option<LayoutUnit>,
        min_inset_for_get_computed_style: Option<LayoutUnit>,
        max_inset_for_get_computed_style: Option<LayoutUnit>,
    ) -> Self {
        Self {
            axis,
            min_inset,
            max_inset,
            min_inset_for_get_computed_style,
            max_inset_for_get_computed_style,
            scroll_container_relative_containing_block_range: RectToBoxEdge(axis, containing_block),
            scroll_container_relative_sticky_box_range: RectToBoxEdge(axis, sticky_box),
            constraining_range: RectToBoxEdge(axis, constraining),
            container: Member::from_ptr(container as *mut LayoutObject),
            sticky_container: Member::from_ptr(sticky_container as *mut LayoutBox),
            containing_scroll_container: Member::from_ptr(
                containing_scroll_container as *mut LayoutBox,
            ),
            is_fixed_to_view,
            total_sticky_box_sticky_offset: LayoutUnit::default(),
            total_containing_block_sticky_offset: LayoutUnit::default(),
            sticky_offset: LayoutUnit::default(),
        }
    }

    pub fn ComputeOffset(&mut self, scroll_position: f32) {
        let mut sticky_box_range = self.scroll_container_relative_sticky_box_range;
        let mut containing_block_range = self.scroll_container_relative_containing_block_range;
        let ancestor_sticky_box_offset = self.AncestorStickyBoxOffset();
        let ancestor_containing_block_offset = self.AncestorContainingBlockOffset();
        sticky_box_range.Move(ancestor_sticky_box_offset + ancestor_containing_block_offset);
        containing_block_range.Move(ancestor_containing_block_offset);

        // Apply max first: the min inset wins on an over-constrained axis.
        let mut box_range = sticky_box_range;
        let mut content_box_range = self.constraining_range;
        if !self.is_fixed_to_view {
            content_box_range.Move(LayoutUnit::FromFloatFloor(scroll_position));
        }
        if let Some(max_inset) = self.max_inset {
            let limit = content_box_range.End() - max_inset;
            let mut delta = limit - sticky_box_range.End();
            let mut available_space = containing_block_range.offset - sticky_box_range.offset;
            delta = delta.ClampPositiveToZero();
            available_space = available_space.ClampPositiveToZero();
            if delta < available_space {
                delta = available_space;
            }
            box_range.Move(delta);
        }
        if let Some(min_inset) = self.min_inset {
            let limit = content_box_range.offset + min_inset;
            let mut delta = limit - sticky_box_range.offset;
            let mut available_space = containing_block_range.End() - sticky_box_range.End();
            delta = delta.ClampNegativeToZero();
            available_space = available_space.ClampNegativeToZero();
            if delta > available_space {
                delta = available_space;
            }
            box_range.Move(delta);
        }
        self.sticky_offset = box_range.offset - sticky_box_range.offset;
        self.total_sticky_box_sticky_offset = ancestor_sticky_box_offset + self.sticky_offset;
        self.total_containing_block_sticky_offset =
            ancestor_sticky_box_offset + ancestor_containing_block_offset + self.sticky_offset;
    }

    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.container);
        visitor.Trace(&self.sticky_container);
        visitor.Trace(&self.containing_scroll_container);
    }

    fn AncestorStickyBoxOffset(&self) -> LayoutUnit {
        let nearest = FindNearestStickyLayerShiftingStickyBox(self);
        if nearest.is_null() {
            return LayoutUnit::default();
        }
        let constraints = unsafe { &*nearest }.StickyConstraints();
        let ancestor_data = constraints.AxisData(self.axis);
        if !ancestor_data.is_null() {
            return unsafe { &*ancestor_data }.total_sticky_box_sticky_offset;
        }
        LayoutUnit::default()
    }

    fn AncestorContainingBlockOffset(&self) -> LayoutUnit {
        let nearest = FindNearestStickyLayerShiftingContainingBlock(self);
        if nearest.is_null() {
            return LayoutUnit::default();
        }
        let constraints = unsafe { &*nearest }.StickyConstraints();
        let ancestor_data = constraints.AxisData(self.axis);
        if !ancestor_data.is_null() {
            return unsafe { &*ancestor_data }.total_containing_block_sticky_offset;
        }
        LayoutUnit::default()
    }
}

fn FindNearestStickyLayerShiftingStickyBox(data: &PerAxisData) -> *const LayoutBoxModelObject {
    unsafe { &*data.container.Get() }.FindFirstStickyContainer(data.sticky_container.Get())
}

fn FindNearestStickyLayerShiftingContainingBlock(
    data: &PerAxisData,
) -> *const LayoutBoxModelObject {
    let scroll_container = data.containing_scroll_container.Get();
    if scroll_container.is_null() {
        return std::ptr::null();
    }
    unsafe { &*data.sticky_container.Get() }.FindFirstStickyContainer(scroll_container)
}

pub struct StickyPositionScrollingConstraints {
    x_data_: *mut PerAxisData,
    y_data_: *mut PerAxisData,
}

impl Default for StickyPositionScrollingConstraints {
    fn default() -> Self {
        Self {
            x_data_: std::ptr::null_mut(),
            y_data_: std::ptr::null_mut(),
        }
    }
}

impl StickyPositionScrollingConstraints {
    pub fn new(x_data: *mut PerAxisData, y_data: *mut PerAxisData) -> Self {
        Self {
            x_data_: x_data,
            y_data_: y_data,
        }
    }

    pub fn HasAnyConstraint(&self) -> bool {
        !self.x_data_.is_null() || !self.y_data_.is_null()
    }

    pub fn ComputeStickyOffset(
        &mut self,
        scroll_position: &gfx::PointF,
        scroll_axes: PhysicalAxes,
    ) {
        if !self.x_data_.is_null() && (scroll_axes & kPhysicalAxesHorizontal).is_nonzero() {
            unsafe { &mut *self.x_data_ }.ComputeOffset(scroll_position.x());
        }
        if !self.y_data_.is_null() && (scroll_axes & kPhysicalAxesVertical).is_nonzero() {
            unsafe { &mut *self.y_data_ }.ComputeOffset(scroll_position.y());
        }
    }

    pub fn AxisData(&self, axis: PhysicalAxis) -> *const PerAxisData {
        if axis == PhysicalAxis::kHorizontal {
            self.x_data_
        } else {
            self.y_data_
        }
    }

    pub fn StickyOffset(&self) -> PhysicalOffset {
        PhysicalOffset::new(
            if self.x_data_.is_null() {
                LayoutUnit::default()
            } else {
                unsafe { &*self.x_data_ }.sticky_offset
            },
            if self.y_data_.is_null() {
                LayoutUnit::default()
            } else {
                unsafe { &*self.y_data_ }.sticky_offset
            },
        )
    }

    pub fn ConstrainingRect(&self) -> PhysicalRect {
        Self::BuildRect(
            unsafe { self.x_data_.as_ref() }.map(|data| &data.constraining_range),
            unsafe { self.y_data_.as_ref() }.map(|data| &data.constraining_range),
        )
    }

    pub fn ScrollContainerRelativeContainingBlockRect(&self) -> PhysicalRect {
        Self::BuildRect(
            unsafe { self.x_data_.as_ref() }
                .map(|data| &data.scroll_container_relative_containing_block_range),
            unsafe { self.y_data_.as_ref() }
                .map(|data| &data.scroll_container_relative_containing_block_range),
        )
    }

    pub fn ScrollContainerRelativeStickyBoxRect(&self) -> PhysicalRect {
        Self::BuildRect(
            unsafe { self.x_data_.as_ref() }
                .map(|data| &data.scroll_container_relative_sticky_box_range),
            unsafe { self.y_data_.as_ref() }
                .map(|data| &data.scroll_container_relative_sticky_box_range),
        )
    }

    pub fn LeftInset(&self) -> Option<LayoutUnit> {
        unsafe { self.x_data_.as_ref() }.and_then(|data| data.min_inset)
    }
    pub fn RightInset(&self) -> Option<LayoutUnit> {
        unsafe { self.x_data_.as_ref() }.and_then(|data| data.max_inset)
    }
    pub fn TopInset(&self) -> Option<LayoutUnit> {
        unsafe { self.y_data_.as_ref() }.and_then(|data| data.min_inset)
    }
    pub fn BottomInset(&self) -> Option<LayoutUnit> {
        unsafe { self.y_data_.as_ref() }.and_then(|data| data.max_inset)
    }

    pub fn LeftInsetForGetComputedStyle(&self) -> Option<LayoutUnit> {
        unsafe { self.x_data_.as_ref() }.and_then(|data| data.min_inset_for_get_computed_style)
    }
    pub fn RightInsetForGetComputedStyle(&self) -> Option<LayoutUnit> {
        unsafe { self.x_data_.as_ref() }.and_then(|data| data.max_inset_for_get_computed_style)
    }
    pub fn TopInsetForGetComputedStyle(&self) -> Option<LayoutUnit> {
        unsafe { self.y_data_.as_ref() }.and_then(|data| data.min_inset_for_get_computed_style)
    }
    pub fn BottomInsetForGetComputedStyle(&self) -> Option<LayoutUnit> {
        unsafe { self.y_data_.as_ref() }.and_then(|data| data.max_inset_for_get_computed_style)
    }

    pub fn ContainingScrollContainer(&self) -> *const LayoutBox {
        let data = self.PreferredAxisData();
        if data.is_null() {
            std::ptr::null()
        } else {
            unsafe { &*data }.containing_scroll_container.Get()
        }
    }

    pub fn NearestStickyLayerShiftingStickyBox(&self) -> *const LayoutBoxModelObject {
        let data = self.PreferredAxisData();
        if data.is_null() {
            std::ptr::null()
        } else {
            FindNearestStickyLayerShiftingStickyBox(unsafe { &*data })
        }
    }

    pub fn NearestStickyLayerShiftingContainingBlock(&self) -> *const LayoutBoxModelObject {
        let data = self.PreferredAxisData();
        if data.is_null() {
            std::ptr::null()
        } else {
            FindNearestStickyLayerShiftingContainingBlock(unsafe { &*data })
        }
    }

    fn PreferredAxisData(&self) -> *const PerAxisData {
        if !self.x_data_.is_null() {
            self.x_data_
        } else {
            self.y_data_
        }
    }

    fn BuildRect(x: Option<&BoxEdge>, y: Option<&BoxEdge>) -> PhysicalRect {
        PhysicalRect::new(
            PhysicalOffset::new(
                x.map_or(LayoutUnit::default(), |axis| axis.offset),
                y.map_or(LayoutUnit::default(), |axis| axis.offset),
            ),
            PhysicalSize::new(
                x.map_or(LayoutUnit::default(), |axis| axis.size),
                y.map_or(LayoutUnit::default(), |axis| axis.size),
            ),
        )
    }
}

pub struct StickyConstraintsData {
    pub x_data: *mut PerAxisData,
    pub y_data: *mut PerAxisData,
}

impl Default for StickyConstraintsData {
    fn default() -> Self {
        Self {
            x_data: std::ptr::null_mut(),
            y_data: std::ptr::null_mut(),
        }
    }
}
