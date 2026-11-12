/*
 * Copyright (C) 2003, 2006, 2007, 2008, 2009, 2010, 2011, 2012 Apple Inc.
 * All rights reserved.
 *
 * Portions are Copyright (C) 1998 Netscape Communications Corporation.
 *
 * Other contributors:
 *   Robert O'Callahan <roc+@cs.cmu.edu>
 *   David Baron <dbaron@dbaron.org>
 *   Christian Biesinger <cbiesinger@gmail.com>, <cbiesinger@web.de>
 *   Randall Jesup <rjesup@wgate.com>
 *   Roland Mainz <roland.mainz@informatik.med.uni-giessen.de>
 *   Josh Soref <timeless@mac.com>
 *   Boris Zbarsky <bzbarsky@mit.edu>
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2.1 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301, USA
 *
 * Alternatively, the contents of this file may be used under the terms
 * of either the Mozilla Public License Version 1.1, found at
 * http://www.mozilla.org/MPL/ (the "MPL") or the GNU General Public
 * License Version 2.0, found at http://www.fsf.org/copyleft/gpl.html
 * (the "GPL"), in which case the provisions of the MPL or the GPL are
 * applicable instead of those above.  If you wish to allow use of your
 * version of this file only under the terms of one of those two
 * licenses (the MPL or the GPL) and not to allow others to use your
 * version of this file under the LGPL, indicate your decision by
 * deletingthe provisions above and replace them with the notice and
 * other provisions required by the MPL or the GPL, as the case may be.
 * If you do not delete the provisions above, a recipient may use your
 * version of this file under any of the LGPL, the MPL or the GPL.
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/paint/paint_layer_scrollable_area.h:288,315-325,329-340,351,363,374-380,575,839-842
// Source: third_party/blink/renderer/core/paint/paint_layer_scrollable_area.cc:419-425,620-626,641-648,658-727,831-859,1009-1027,1399-1413,2515-2539,3751-3795
// Shared-source translation: only the scroll geometry and scrollability ranges
// below. Paint keeps its resident PaintLayerScrollableArea and LayoutBox owners.

use foundation::gfx::{Point, PointF, Rect, Size, Vector2d};
use foundation::length::Length;
use foundation::length_functions::MinimumValueForLength;
use foundation::style_constants::{EOverflow, EOverscrollBehavior, EResize, EVisibility};
use foundation::{
    LayoutUnit, PhysicalOffset, PhysicalRect, PhysicalSize, ToFlooredPoint, ToPixelSnappedRect,
    ToRoundedSize,
};
use interaction_input::ui::events::types::scroll_types::ScrollGranularity;

use crate::cc::trees::scroll_source_type::ScrollSourceType;
use crate::core::scroll::scroll_types::{
    ScrollDirectionPhysical, ScrollOffset, ScrollbarOrientation, ToScrollDelta,
};
use crate::core::scroll::scrollable_area::{IncludeScrollbarsInRect, ScrollBehavior, ScrollType};
pub use crate::public::mojom::scroll::scrollbar_mode::ScrollbarMode;

// Source: paint_layer_scrollable_area.cc:641-727,831-859,1399-1413,2515-2539,3751-3795.
// Each projection borrows the same resident LayoutBox/style/area. Only the
// fields actually read by these source methods cross the boundary.
pub trait PaintScrollGeometry {
    fn HasLayoutBox(&self) -> bool;
    fn IsScrollContainer(&self) -> bool;
    fn IsLayoutView(&self) -> bool;
    fn IsVerticalScrollEnforced(&self) -> bool;
    fn HasNonDocumentFullscreenElement(&self) -> bool;
    fn ViewScrollbarMode(&self, orientation: ScrollbarOrientation) -> ScrollbarMode;
    fn Overflow(&self, orientation: ScrollbarOrientation) -> EOverflow;
    fn IsOverflowValueScrollable(&self, orientation: ScrollbarOrientation) -> bool;
    fn Visibility(&self) -> EVisibility;
    fn OverscrollBehavior(&self, orientation: ScrollbarOrientation) -> EOverscrollBehavior;
    fn ScrollOrigin(&self) -> Point;
    fn GetScrollOffset(&self) -> ScrollOffset;
    fn SnapScrollOffsetToPhysicalPixels(&self, offset: ScrollOffset) -> Vector2d;
    fn Size(&self) -> PhysicalSize;
    // top, right, bottom, left; these are existing used-value outsets.
    fn BorderOutsets(&self) -> [LayoutUnit; 4];
    fn ScrollbarOutsets(&self) -> [LayoutUnit; 4];
    fn ScrollPadding(&self, side: usize) -> &Length;
    fn PhysicalPaddingBoxOffset(&self) -> PhysicalOffset;
    fn OverflowRectSize(&self) -> PhysicalSize;
    fn RootTransitionSnapshotSize(&self) -> Option<PhysicalSize>;
    fn IsGlobalRootScroller(&self) -> bool;
    fn RootScrollerVisibleArea(&self) -> Size;
    fn OverflowClipRectSize(&self) -> PhysicalSize;
}

// A borrowed readiness token. The assembly root constructs it at the source
// query boundary after the owning Layout/Paint state has been prepared.
pub struct PreparedScrollGeometry<'a> {
    geometry: &'a dyn PaintScrollGeometry,
}
impl<'a> PreparedScrollGeometry<'a> {
    pub fn new(geometry: &'a dyn PaintScrollGeometry) -> Self {
        Self { geometry }
    }
}

pub struct PaintLayerScrollableArea<'a> {
    geometry: &'a dyn PaintScrollGeometry,
}

// Source: paint_layer_scrollable_area.cc:1017-1021. The snap owner retains its
// strategy/target state; this projection neither duplicates nor caches it.
pub trait PageScrollSnapSelection {
    type Strategy;
    fn PageScrollSnapStrategy(&self, direction: ScrollDirectionPhysical) -> Self::Strategy;
    fn GetSnapPositionAndSetTarget(&self, strategy: &Self::Strategy) -> Option<PointF>;
}

#[derive(Clone, Copy, Debug)]
pub struct ScrollOffsetRequest {
    pub offset: ScrollOffset,
    pub scroll_type: ScrollType,
    pub source_type: ScrollSourceType,
    pub behavior: ScrollBehavior,
}
pub trait ScrollOffsetCommit {
    // Synchronously routes the typed request to the same resident scroll owner.
    fn SetScrollOffset(&self, request: ScrollOffsetRequest) -> bool;
}

/// Live area/box queries and the Element's synchronous CSSOM effects for the
/// source resize transaction. No LayoutObject or ComputedStyle is copied.
pub trait ResizeGeometry {
    fn InResizeMode(&self) -> bool;
    fn CanResize(&self) -> bool;
    fn HasElement(&self) -> bool;
    fn EffectiveZoom(&self) -> f32;
    fn StitchedSize(&self) -> PhysicalSize;
    fn ConvertFromRootFrame(&self, point: Point) -> Point;
    fn CornerSize(&self) -> Size;
    fn IsBorderBox(&self) -> bool;
    fn BorderPadding(&self) -> PhysicalSize;
    fn UsedResize(&self) -> EResize;
    fn SetInlineSizeProperty(&self, horizontal: bool, pixels: i32);
    fn UpdateStyleAndLayoutForSizeChange(&self);
}

// Source: paint_layer_scrollable_area.cc:2442-2509.
pub fn Resize(
    geometry: &dyn ResizeGeometry,
    point: Point,
    position_to_size: &foundation::gfx::Transform,
) {
    if !geometry.InResizeMode() || !geometry.CanResize() || !geometry.HasElement() {
        return;
    }
    let zoom = geometry.EffectiveZoom();
    let mut current = geometry.StitchedSize();
    current.ScaleFloat(1.0 / zoom);
    let frame = geometry.ConvertFromRootFrame(point);
    let local = foundation::gfx::point_conversions::ToRoundedPoint(
        position_to_size.ProjectPoint(PointF::new(frame.x() as f32, frame.y() as f32)),
    );
    let corner = geometry.CornerSize();
    let new_size = PhysicalSize::new(
        LayoutUnit::from_signed((local.x() as f32 / zoom) as i32)
            .max(LayoutUnit::from_f32(corner.width() as f32 / zoom)),
        LayoutUnit::from_signed((local.y() as f32 / zoom) as i32)
            .max(LayoutUnit::from_f32(corner.height() as f32 / zoom)),
    );
    let difference = new_size - current;
    let padding = if geometry.IsBorderBox() {
        PhysicalSize::default()
    } else {
        geometry.BorderPadding()
    };
    let stitched = geometry.StitchedSize();
    if geometry.UsedResize() != EResize::kVertical && difference.width != LayoutUnit::new() {
        let base = LayoutUnit::from_f32((stitched.width - padding.width).ToFloat() / zoom);
        geometry.SetInlineSizeProperty(true, (base + difference.width).ToFloat().round() as i32);
    }
    if geometry.UsedResize() != EResize::kHorizontal && difference.height != LayoutUnit::new() {
        let base = LayoutUnit::from_f32((stitched.height - padding.height).ToFloat() / zoom);
        geometry.SetInlineSizeProperty(false, (base + difference.height).ToFloat().round() as i32);
    }
    geometry.UpdateStyleAndLayoutForSizeChange();
}

impl<'a> PaintLayerScrollableArea<'a> {
    pub fn new(prepared: PreparedScrollGeometry<'a>) -> Self {
        Self {
            geometry: prepared.geometry,
        }
    }

    // Source: paint_layer_scrollable_area.h:315; .cc:419-425
    pub fn ScrollSize(&self, orientation: ScrollbarOrientation) -> i32 {
        let dimensions = self.MaximumScrollOffsetInt() - self.MinimumScrollOffsetInt();
        if orientation == ScrollbarOrientation::kHorizontalScrollbar {
            dimensions.x()
        } else {
            dimensions.y()
        }
    }

    // Source: paint_layer_scrollable_area.h:316-323,363
    pub fn ScrollOffsetToPosition(&self, offset: ScrollOffset) -> PointF {
        let origin = self.geometry.ScrollOrigin();
        PointF::new(origin.x() as f32, origin.y() as f32) + offset
    }
    pub fn ScrollPositionToOffset(&self, position: PointF) -> ScrollOffset {
        let origin = self.geometry.ScrollOrigin();
        position - PointF::new(origin.x() as f32, origin.y() as f32)
    }
    pub fn ScrollPosition(&self) -> PointF {
        self.ScrollOffsetToPosition(self.GetScrollOffset())
    }

    // Source: paint_layer_scrollable_area.h:324-325; .cc:620-626
    pub fn ScrollOffsetInt(&self) -> Vector2d {
        self.geometry
            .SnapScrollOffsetToPhysicalPixels(self.geometry.GetScrollOffset())
    }
    pub fn GetScrollOffset(&self) -> ScrollOffset {
        self.geometry.GetScrollOffset()
    }

    // Source: paint_layer_scrollable_area.h:329-330; .cc:641-648
    pub fn MinimumScrollOffsetInt(&self) -> Vector2d {
        let origin = self.geometry.ScrollOrigin();
        self.ClampNonScrollableAxesOffsets(Vector2d::new(-origin.x(), -origin.y()))
    }
    pub fn MaximumScrollOffsetInt(&self) -> Vector2d {
        let origin = self.geometry.ScrollOrigin();
        self.ClampNonScrollableAxesOffsets(
            Vector2d::new(-origin.x(), -origin.y()) + self.ComputeScrollableSize(),
        )
    }

    // Source: paint_layer_scrollable_area.h:331-333; .cc:658-682
    pub fn LayoutContentRect(&self, inclusion: IncludeScrollbarsInRect) -> PhysicalRect {
        let layer_size = self.geometry.Size();
        let border = self.geometry.BorderOutsets();
        let scrollbar = if inclusion == IncludeScrollbarsInRect::kExcludeScrollbars {
            self.geometry.ScrollbarOutsets()
        } else {
            [LayoutUnit::new(); 4]
        };
        let mut size = PhysicalSize::new(
            layer_size.width - border[1] - border[3] - scrollbar[1] - scrollbar[3],
            layer_size.height - border[0] - border[2] - scrollbar[0] - scrollbar[2],
        );
        size.ClampNegativeToZero();
        PhysicalRect::new(
            PhysicalOffset::FromPointFRound(&self.ScrollPosition()),
            size,
        )
    }
    pub fn VisibleContentRect(&self, inclusion: IncludeScrollbarsInRect) -> Rect {
        let rect = self.LayoutContentRect(inclusion);
        Rect::new(ToFlooredPoint(rect.offset), ToRoundedSize(rect.size))
    }

    // Source: paint_layer_scrollable_area.h:334-335; .cc:684-699
    pub fn VisibleScrollSnapportRect(&self, inclusion: IncludeScrollbarsInRect) -> PhysicalRect {
        let mut rect = self.LayoutContentRect(inclusion);
        let origin = self.geometry.ScrollOrigin();
        rect.Move(&PhysicalOffset::new(
            LayoutUnit::from_signed(-origin.x()),
            LayoutUnit::from_signed(-origin.y()),
        ));
        let top = MinimumValueForLength(self.geometry.ScrollPadding(0), rect.Height());
        let right = MinimumValueForLength(self.geometry.ScrollPadding(1), rect.Width());
        let bottom = MinimumValueForLength(self.geometry.ScrollPadding(2), rect.Height());
        let left = MinimumValueForLength(self.geometry.ScrollPadding(3), rect.Width());
        rect.ContractEdges(top, right, bottom, left);
        rect
    }

    // Source: paint_layer_scrollable_area.h:336,338-340; .cc:701-727
    pub fn ContentsSize(&self) -> Size {
        self.PixelSnappedContentsSize(self.geometry.PhysicalPaddingBoxOffset())
    }
    pub fn PixelSnappedContentsSize(&self, paint_offset: PhysicalOffset) -> Size {
        let mut size = self.geometry.OverflowRectSize();
        if self.geometry.IsLayoutView() {
            if let Some(container) = self.geometry.RootTransitionSnapshotSize() {
                size.width = size.width.max(container.width);
                size.height = size.height.max(container.height);
            }
        }
        ToPixelSnappedRect(PhysicalRect::new(paint_offset, size)).size()
    }

    // Source: paint_layer_scrollable_area.h:351; .cc:831-859
    pub fn UserInputScrollable(&self, orientation: ScrollbarOrientation) -> bool {
        if orientation == ScrollbarOrientation::kVerticalScrollbar
            && self.geometry.IsVerticalScrollEnforced()
        {
            return false;
        }
        if self.geometry.IsLayoutView() {
            if self.geometry.HasNonDocumentFullscreenElement() {
                return false;
            }
            let mode = self.geometry.ViewScrollbarMode(orientation);
            return matches!(mode, ScrollbarMode::kAuto | ScrollbarMode::kAlwaysOn);
        }
        matches!(
            self.geometry.Overflow(orientation),
            EOverflow::kScroll | EOverflow::kAuto | EOverflow::kOverlay
        )
    }

    // Source: paint_layer_scrollable_area.h:374-380; .cc:1009-1027
    pub fn ScrollByPageWithSnap<S: PageScrollSnapSelection>(
        &self,
        direction: ScrollDirectionPhysical,
        behavior: ScrollBehavior,
        page_steps: &crate::core::scroll::scrollable_area::ScrollableArea<'_>,
        snap: &S,
        commit: &dyn ScrollOffsetCommit,
    ) -> bool {
        let mut displacement = ToScrollDelta(direction, 1.0);
        displacement.Scale(
            page_steps.ScrollStep(
                ScrollGranularity::kScrollByPage,
                ScrollbarOrientation::kHorizontalScrollbar,
            ),
            page_steps.ScrollStep(
                ScrollGranularity::kScrollByPage,
                ScrollbarOrientation::kVerticalScrollbar,
            ),
        );
        let current = self.ScrollPosition();
        let strategy = snap.PageScrollSnapStrategy(direction);
        let position = snap
            .GetSnapPositionAndSetTarget(&strategy)
            .unwrap_or(current + displacement);
        commit.SetScrollOffset(ScrollOffsetRequest {
            offset: self.ScrollPositionToOffset(position),
            scroll_type: ScrollType::kProgrammatic,
            source_type: ScrollSourceType::kNone,
            behavior,
        })
    }

    // Source: paint_layer_scrollable_area.cc:1399-1413
    pub fn CanPropagateScroll(&self) -> bool {
        if !self.geometry.HasLayoutBox()
            || !self.geometry.IsScrollContainer()
            || self.geometry.Visibility() != EVisibility::kVisible
        {
            return true;
        }
        for orientation in [
            ScrollbarOrientation::kHorizontalScrollbar,
            ScrollbarOrientation::kVerticalScrollbar,
        ] {
            let behavior = self.geometry.OverscrollBehavior(orientation);
            if behavior != EOverscrollBehavior::kAuto && behavior != EOverscrollBehavior::kChain {
                return false;
            }
        }
        true
    }

    // Source: paint_layer_scrollable_area.h:288; .cc:2515-2539
    pub fn ScrollableAxes(&self) -> (bool, bool) {
        if !self.geometry.HasLayoutBox() || !self.geometry.IsScrollContainer() {
            return (false, false);
        }
        if self.geometry.IsLayoutView() {
            return (true, true);
        }
        (
            self.geometry
                .IsOverflowValueScrollable(ScrollbarOrientation::kHorizontalScrollbar),
            self.geometry
                .IsOverflowValueScrollable(ScrollbarOrientation::kVerticalScrollbar),
        )
    }

    // Source: paint_layer_scrollable_area.h:839; .cc:3751-3783
    pub fn ComputeScrollableSize(&self) -> Vector2d {
        if !self.geometry.HasLayoutBox() || !self.geometry.IsScrollContainer() {
            return Vector2d::default();
        }
        let contents = self.ContentsSize();
        let visible = if self.geometry.IsGlobalRootScroller() {
            self.geometry.RootScrollerVisibleArea()
        } else {
            ToRoundedSize(self.geometry.OverflowClipRectSize())
        };
        let width = contents.width().max(visible.width());
        let height = contents.height().max(visible.height());
        Vector2d::new(width - visible.width(), height - visible.height())
    }

    // Source: paint_layer_scrollable_area.h:842; .cc:3785-3795
    pub fn ClampNonScrollableAxesOffsets(&self, mut offset: Vector2d) -> Vector2d {
        let (horizontal, vertical) = self.ScrollableAxes();
        if !horizontal {
            offset.set_x(0);
        }
        if !vertical {
            offset.set_y(0);
        }
        offset
    }
}

use crate::core::scroll::scrollbar::ScrollbarPart;
use interaction_input::public::common::input::web_input_event::Type;

/// PaintLayerScrollableArea::DidScrollWithScrollbar's thumb branch,
/// paint_layer_scrollable_area.cc:3352-3364. DOM retains UseCounter storage.
#[derive(Clone, Copy, Debug)]
pub enum ScrollbarThumbUse {
    VerticalMouse,
    VerticalTouch,
    HorizontalMouse,
    HorizontalTouch,
    ReversedButtonDirection,
    VerticalButton,
    HorizontalButton,
    VerticalTrack,
    HorizontalTrack,
}
// Source paint_layer_scrollable_area.cc:3331-3379. The reversed-button
// feature is counted before the ordinary orientation-specific button feature.
pub fn PressedPartUseFeatures(
    part: ScrollbarPart,
    horizontal: bool,
    event_type: Type,
) -> [Option<ScrollbarThumbUse>; 2] {
    use ScrollbarPart::*;
    let mut first = None;
    let main = match part {
        kBackButtonEndPart | kForwardButtonStartPart => {
            first = Some(ScrollbarThumbUse::ReversedButtonDirection);
            if horizontal {
                ScrollbarThumbUse::HorizontalButton
            } else {
                ScrollbarThumbUse::VerticalButton
            }
        }
        kBackButtonStartPart | kForwardButtonEndPart => {
            if horizontal {
                ScrollbarThumbUse::HorizontalButton
            } else {
                ScrollbarThumbUse::VerticalButton
            }
        }
        kThumbPart => ThumbUseFeature(horizontal, event_type),
        kBackTrackPart | kForwardTrackPart => {
            if horizontal {
                ScrollbarThumbUse::HorizontalTrack
            } else {
                ScrollbarThumbUse::VerticalTrack
            }
        }
        _ => return [None, None],
    };
    [first, Some(main)]
}
pub fn ThumbUseFeature(horizontal: bool, event_type: Type) -> ScrollbarThumbUse {
    use interaction_input::public::common::input::web_input_event::WebInputEvent;
    match (horizontal, WebInputEvent::IsMouseEventType(event_type)) {
        (false, true) => ScrollbarThumbUse::VerticalMouse,
        (false, false) => ScrollbarThumbUse::VerticalTouch,
        (true, true) => ScrollbarThumbUse::HorizontalMouse,
        (true, false) => ScrollbarThumbUse::HorizontalTouch,
    }
}
