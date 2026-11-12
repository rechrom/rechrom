#![allow(non_snake_case)]

use foundation::{
    EPosition, LayoutUnit, Length, MakeGarbageCollected, Member, MinimumValueForLength,
    PhysicalOffset, PhysicalRect, RuntimeEnabledFeatures, To,
};
use layoutng_geometry::geometry::axis::{
    kPhysicalAxesBoth, kPhysicalAxesHorizontal, kPhysicalAxesNone, kPhysicalAxesVertical,
    PhysicalAxes, PhysicalAxis,
};
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;
use layoutng_style::style::computed_style::ComputedStyle;

use super::super::layout_block::LayoutBlock;
use super::super::layout_box::LayoutBox;
use super::super::layout_box_model_object::LayoutBoxModelObject;
use super::super::layout_object::LayoutObject;
use super::super::map_coordinates_flags::{MapCoordinatesFlags, MapCoordinatesMode};
use super::super::sticky_position_scrolling_constraints::{
    StickyConstraintsData, StickyPositionScrollingConstraints,
};

// LayoutInline's bounds and table-specific StickyContainer overrides are
// provided by their owning packages during hierarchy assembly.
unsafe extern "Rust" {
    fn DispatchLayoutInlinePhysicalLinesBoundingBox(inline: *const LayoutObject) -> PhysicalRect;
    fn DispatchLayoutBoxModelObjectStickyContainer(
        object: *const LayoutBoxModelObject,
    ) -> *mut LayoutBlock;
}

#[inline(never)]
fn MapRectToAncestorIgnoringPaintGeometry(
    object: &LayoutObject,
    mut rect: PhysicalRect,
    ancestor: *const LayoutBoxModelObject,
    mode: MapCoordinatesFlags,
) -> PhysicalRect {
    debug_assert!(mode.contains(MapCoordinatesMode::kIgnoreTransforms));
    let mut offset = PhysicalOffset::default();
    let mut current: *const LayoutObject = object;
    while current != ancestor as *const LayoutObject {
        let container = unsafe { &*current }.Container();
        assert!(!container.is_null());
        offset += unsafe { &*current }.OffsetFromContainer(container, mode);
        current = container;
    }
    rect.Move(&offset);
    rect
}

impl LayoutBoxModelObject {
    pub fn StickyConstraints(&self) -> StickyPositionScrollingConstraints {
        self.CheckIsNotDestroyed();
        self.FirstFragment().StickyConstraints()
    }

    pub fn HasStickyConstraints(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.FirstFragment().HasStickyConstraints()
    }

    pub fn SetStickyConstraints(&mut self, constraints: StickyConstraintsData) {
        self.CheckIsNotDestroyed();
        debug_assert!(!constraints.x_data.is_null() || !constraints.y_data.is_null());
        let has_changed = self
            .GetMutableForPainting()
            .FirstFragment()
            .SetStickyConstraints(constraints);
        if has_changed {
            self.SetNeedsPaintPropertyUpdate();
        }
    }

    pub fn ClearStickyConstraints(&mut self, axes_to_clear: PhysicalAxes) {
        self.CheckIsNotDestroyed();
        let has_changed = self
            .GetMutableForPainting()
            .FirstFragment()
            .ClearStickyConstraints(axes_to_clear);
        if has_changed {
            self.SetNeedsPaintPropertyUpdate();
        }
    }

    pub fn StickyContainer(&self) -> *mut LayoutBlock {
        self.CheckIsNotDestroyed();
        self.ContainingBlock()
    }

    pub fn StickyConstrainedAxes(style: &ComputedStyle) -> PhysicalAxes {
        if style.GetPosition() != EPosition::kSticky {
            return kPhysicalAxesNone;
        }
        let mut axes = kPhysicalAxesNone;
        if !style.Top().IsAuto() || !style.Bottom().IsAuto() {
            axes |= kPhysicalAxesVertical;
        }
        if !style.Left().IsAuto() || !style.Right().IsAuto() {
            axes |= kPhysicalAxesHorizontal;
        }
        if !RuntimeEnabledFeatures::SingleAxisScrollContainersEnabled() && axes.is_nonzero() {
            axes = kPhysicalAxesBoth;
        }
        axes
    }

    pub fn StickyPositionOffset(&self) -> PhysicalOffset {
        self.CheckIsNotDestroyed();
        self.StickyConstraints().StickyOffset()
    }

    pub fn OffsetFromContainerInternal(
        &self,
        container: *const LayoutObject,
        mode: MapCoordinatesFlags,
    ) -> PhysicalOffset {
        self.CheckIsNotDestroyed();
        let mut offset = PhysicalOffset::default();
        if self.IsStickyPositioned() && !mode.contains(MapCoordinatesMode::kIgnoreStickyOffset) {
            offset += self.StickyPositionOffset();
        }
        offset + self.OffsetFromContainerInternalBase(container, mode)
    }

    #[inline(never)]
    pub fn ComputeStickyPositionConstraintsForLayout(
        &self,
        scroll_container: *const LayoutBox,
        is_fixed_to_view: bool,
        scroll_axes: PhysicalAxes,
    ) -> StickyConstraintsData {
        self.CheckIsNotDestroyed();
        debug_assert!(self.StyleRef().HasStickyConstrainedPosition());
        let mut sticky_container = unsafe { DispatchLayoutBoxModelObjectStickyContainer(self) };
        while unsafe { &*sticky_container }.IsAnonymous() {
            let parent = unsafe { &*sticky_container }.Parent();
            if !parent.is_null() && unsafe { &*parent }.IsFieldset() {
                break;
            }
            sticky_container = unsafe { &*sticky_container }.ContainingBlock();
        }

        debug_assert!(!scroll_container.is_null());
        let scroll_container_border_offset = unsafe { &*scroll_container }.BorderOutsets().Offset();
        let mut flags = MapCoordinatesFlags::from_mode(MapCoordinatesMode::kIgnoreTransforms);
        flags.insert(MapCoordinatesMode::kIgnoreScrollOffset);
        flags.insert(MapCoordinatesMode::kIgnoreStickyOffset);
        flags.insert(MapCoordinatesMode::kIgnoreScrollOriginAndOffset);

        let mut scroll_container_relative_containing_block_rect =
            if sticky_container as *const LayoutBox == scroll_container {
                unsafe { &*sticky_container }.ScrollableOverflowRect()
            } else {
                let local_rect = unsafe { &*sticky_container }.PhysicalPaddingBoxRect();
                MapRectToAncestorIgnoringPaintGeometry(
                    unsafe { &*sticky_container },
                    local_rect,
                    scroll_container as *const LayoutBoxModelObject,
                    flags,
                )
            };
        scroll_container_relative_containing_block_rect.Move(&-scroll_container_border_offset);
        scroll_container_relative_containing_block_rect
            .Contract(&unsafe { &*sticky_container }.PaddingOutsets());
        if !RuntimeEnabledFeatures::LayoutIgnoreMarginsForStickyEnabled() {
            let max_width = unsafe { &*sticky_container }.ContentLogicalWidth();
            scroll_container_relative_containing_block_rect.ContractEdges(
                MinimumValueForLength(self.StyleRef().MarginTop(), max_width),
                MinimumValueForLength(self.StyleRef().MarginRight(), max_width),
                MinimumValueForLength(self.StyleRef().MarginBottom(), max_width),
                MinimumValueForLength(self.StyleRef().MarginLeft(), max_width),
            );
        }

        let container = self.Container();
        let sticky_box_rect = if self.IsLayoutInline() {
            unsafe {
                DispatchLayoutInlinePhysicalLinesBoundingBox(
                    self as *const _ as *const LayoutObject,
                )
            }
        } else {
            let box_ = To::<LayoutBox>(self as *const _ as *const LayoutObject);
            let box_ = unsafe { &*box_ };
            PhysicalRect::new(box_.PhysicalLocation(), box_.StitchedSize())
        };
        let mut scroll_container_relative_sticky_box_rect = MapRectToAncestorIgnoringPaintGeometry(
            unsafe { &*container },
            sticky_box_rect,
            scroll_container as *const LayoutBoxModelObject,
            flags,
        );
        scroll_container_relative_sticky_box_rect.Move(&-scroll_container_border_offset);

        let constraining_rect = unsafe { &*scroll_container }.ComputeStickyConstrainingRect();
        let compute_axis_data = |axis: PhysicalAxis,
                                 min_length: &Length,
                                 max_length: &Length,
                                 available_size: LayoutUnit,
                                 sticky_box_size: LayoutUnit,
                                 is_flipped: bool| {
            let axes = if axis == PhysicalAxis::kHorizontal {
                kPhysicalAxesHorizontal
            } else {
                kPhysicalAxesVertical
            };
            if !(axes & scroll_axes).is_nonzero() {
                return std::ptr::null_mut();
            }
            let mut min_inset = None;
            let mut max_inset = None;
            let mut min_inset_for_get_computed_style = None;
            let mut max_inset_for_get_computed_style = None;
            if !min_length.IsAuto() {
                let value = MinimumValueForLength(min_length, available_size);
                min_inset = Some(value);
                min_inset_for_get_computed_style = Some(value);
            }
            if !max_length.IsAuto() {
                let value = MinimumValueForLength(max_length, available_size);
                max_inset = Some(value);
                max_inset_for_get_computed_style = Some(value);
            }
            if let (Some(min), Some(max)) = (min_inset.as_mut(), max_inset.as_mut()) {
                let free_space = available_size - sticky_box_size - *min - *max;
                if free_space < LayoutUnit::default() {
                    if is_flipped {
                        *min += free_space;
                    } else {
                        *max += free_space;
                    }
                }
            }
            MakeGarbageCollected(
                super::super::sticky_position_scrolling_constraints::PerAxisData::new(
                    axis,
                    &scroll_container_relative_containing_block_rect,
                    &scroll_container_relative_sticky_box_rect,
                    &constraining_rect,
                    container,
                    sticky_container as *const LayoutBox,
                    scroll_container,
                    is_fixed_to_view,
                    min_inset,
                    max_inset,
                    min_inset_for_get_computed_style,
                    max_inset_for_get_computed_style,
                ),
            )
        };

        let style = self.StyleRef();
        let sticky_container_writing_direction = unsafe { &*sticky_container }
            .StyleRef()
            .GetWritingDirection();
        StickyConstraintsData {
            x_data: compute_axis_data(
                PhysicalAxis::kHorizontal,
                style.Left(),
                style.Right(),
                constraining_rect.size.width,
                sticky_box_rect.Width(),
                sticky_container_writing_direction.IsFlippedX(),
            ),
            y_data: compute_axis_data(
                PhysicalAxis::kVertical,
                style.Top(),
                style.Bottom(),
                constraining_rect.size.height,
                sticky_box_rect.Height(),
                sticky_container_writing_direction.IsFlippedY(),
            ),
        }
    }
}

impl LayoutObject {
    #[inline(never)]
    pub fn FindFirstStickyContainer(&self, below: *const LayoutBox) -> *const LayoutBoxModelObject {
        self.CheckIsNotDestroyed();
        debug_assert!(self.IsContainedBy(below as *const LayoutObject));
        let mut ancestor: *const LayoutObject = self;
        while ancestor != below as *const LayoutObject {
            let current = unsafe { &*ancestor };
            if current.StyleRef().HasStickyConstrainedPosition() {
                return To::<LayoutBoxModelObject>(ancestor);
            }
            ancestor = current.Container();
        }
        std::ptr::null()
    }
}

impl LayoutBox {
    pub fn ComputeStickyConstrainingRect(&self) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        // C++ uses DCHECK here; the release renderer still computes the
        // constraint rectangle for a styled scrolling ancestor.
        let mut constraining_rect = self.OverflowClipRect();
        constraining_rect.Move(&-self.BorderOutsets().Offset());
        constraining_rect.Contract(&self.PaddingOutsets());
        let origin = self.ScrollOrigin();
        constraining_rect.Move(&-PhysicalOffset::new(
            LayoutUnit::from_signed(origin.x()),
            LayoutUnit::from_signed(origin.y()),
        ));
        constraining_rect
    }
}
