// C++: src/foundation/blink_geometry/geometry/physical_rect.h:30-81, 118-151
use crate::{gfx, LayoutUnit, PhysicalOffset, PhysicalSize};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PhysicalRect {
    pub offset: PhysicalOffset,
    pub size: PhysicalSize,
}

impl PhysicalRect {
    pub fn new(offset: PhysicalOffset, size: PhysicalSize) -> Self {
        Self { offset, size }
    }

    pub fn from_units(
        left: LayoutUnit,
        top: LayoutUnit,
        width: LayoutUnit,
        height: LayoutUnit,
    ) -> Self {
        Self::new(
            PhysicalOffset::new(left, top),
            PhysicalSize::new(width, height),
        )
    }

    pub fn IsEmpty(&self) -> bool {
        self.size.IsEmpty()
    }
    pub fn X(&self) -> LayoutUnit {
        self.offset.left
    }
    pub fn Y(&self) -> LayoutUnit {
        self.offset.top
    }
    pub fn Width(&self) -> LayoutUnit {
        self.size.width
    }
    pub fn Height(&self) -> LayoutUnit {
        self.size.height
    }
    pub fn Right(&self) -> LayoutUnit {
        self.offset.left + self.size.width
    }
    pub fn Bottom(&self) -> LayoutUnit {
        self.offset.top + self.size.height
    }

    // cpp: foundation/blink_geometry/geometry/physical_rect.h:59-68
    pub fn MinXMinYCorner(&self) -> PhysicalOffset {
        self.offset
    }
    pub fn MaxXMaxYCorner(&self) -> PhysicalOffset {
        PhysicalOffset::new(self.Right(), self.Bottom())
    }

    pub fn ExpandEdges(
        &mut self,
        top: LayoutUnit,
        right: LayoutUnit,
        bottom: LayoutUnit,
        left: LayoutUnit,
    ) {
        self.offset.top -= top;
        self.offset.left -= left;
        self.size.width += left + right;
        self.size.height += top + bottom;
    }

    pub fn Inflate(&mut self, amount: LayoutUnit) {
        self.ExpandEdges(amount, amount, amount, amount);
    }

    // cpp: foundation/blink_geometry/geometry/physical_rect.h:128-133
    pub fn ContractEdges(
        &mut self,
        top: LayoutUnit,
        right: LayoutUnit,
        bottom: LayoutUnit,
        left: LayoutUnit,
    ) {
        self.ExpandEdges(-top, -right, -bottom, -left);
    }

    pub fn Move(&mut self, offset: &PhysicalOffset) {
        self.offset += *offset;
    }

    // cpp: foundation/blink_geometry/geometry/physical_rect.h:137-153
    pub fn ShiftLeftEdgeTo(&mut self, edge: LayoutUnit) {
        let delta = edge - self.X();
        self.offset.left = edge;
        self.size.width = (self.Width() - delta).ClampNegativeToZero();
    }
    pub fn ShiftRightEdgeTo(&mut self, edge: LayoutUnit) {
        let delta = edge - self.Right();
        self.size.width = (self.Width() + delta).ClampNegativeToZero();
    }
    pub fn ShiftTopEdgeTo(&mut self, edge: LayoutUnit) {
        let delta = edge - self.Y();
        self.offset.top = edge;
        self.size.height = (self.Height() - delta).ClampNegativeToZero();
    }
    pub fn ShiftBottomEdgeTo(&mut self, edge: LayoutUnit) {
        let delta = edge - self.Bottom();
        self.size.height = (self.Height() + delta).ClampNegativeToZero();
    }

    // cpp: foundation/blink_geometry/geometry/physical_rect.h:177-183
    pub fn EnclosingRect(rect: &gfx::RectF) -> Self {
        let offset = PhysicalOffset::new(
            LayoutUnit::FromFloatFloor(rect.x()),
            LayoutUnit::FromFloatFloor(rect.y()),
        );
        let size = PhysicalSize::new(
            LayoutUnit::FromFloatCeil(rect.right()) - offset.left,
            LayoutUnit::FromFloatCeil(rect.bottom()) - offset.top,
        );
        Self::new(offset, size)
    }

    // cpp: foundation/blink_geometry/geometry/physical_rect.h:158-168
    pub fn PixelSnappedOffset(&self) -> gfx::Point {
        gfx::Point::new(self.offset.left.Round(), self.offset.top.Round())
    }

    pub fn PixelSnappedWidth(&self) -> i32 {
        SnapSizeToPixel(self.size.width, self.offset.left)
    }

    pub fn PixelSnappedHeight(&self) -> i32 {
        SnapSizeToPixel(self.size.height, self.offset.top)
    }

    pub fn PixelSnappedSize(&self) -> gfx::Size {
        gfx::Size::new(self.PixelSnappedWidth(), self.PixelSnappedHeight())
    }
}

// cpp: foundation/blink_geometry/geometry/layout_unit.h:807-815
pub fn SnapSizeToPixel(size: LayoutUnit, location: LayoutUnit) -> i32 {
    let fraction = location.Fraction();
    let result = (fraction + size).Round() - fraction.Round();
    if result == 0 && (size.RawValue() > 4 || size.RawValue() < -4) {
        return if size.RawValue() > 0 { 1 } else { -1 };
    }
    result
}

// cpp: foundation/blink_geometry/geometry/physical_rect.h:228-230
pub fn ToPixelSnappedRect(rect: PhysicalRect) -> gfx::Rect {
    gfx::Rect::new(rect.PixelSnappedOffset(), rect.PixelSnappedSize())
}

// cpp: foundation/blink_geometry/geometry/physical_rect.h:197-198
impl From<gfx::Rect> for PhysicalRect {
    fn from(rect: gfx::Rect) -> Self {
        Self::from_units(
            LayoutUnit::from_signed(rect.x()),
            LayoutUnit::from_signed(rect.y()),
            LayoutUnit::from_signed(rect.width()),
            LayoutUnit::from_signed(rect.height()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enclosing_rect_rounds_outer_edges() {
        let rect = PhysicalRect::EnclosingRect(&gfx::RectF::new(
            gfx::PointF::new(1.2, 2.8),
            gfx::SizeF::new(3.5, 4.1),
        ));
        assert!(rect.X().ToFloat() <= 1.2);
        assert!(rect.Y().ToFloat() <= 2.8);
        assert!(rect.Right().ToFloat() >= 4.7);
        assert!(rect.Bottom().ToFloat() >= 6.9);
    }

    #[test]
    fn shifting_an_edge_clamps_negative_size() {
        let mut rect = PhysicalRect::from_units(
            LayoutUnit::from_signed(2),
            LayoutUnit::from_signed(3),
            LayoutUnit::from_signed(4),
            LayoutUnit::from_signed(5),
        );
        rect.ShiftLeftEdgeTo(LayoutUnit::from_signed(8));
        assert_eq!(rect.X(), LayoutUnit::from_signed(8));
        assert_eq!(rect.Width(), LayoutUnit::default());
        rect.ShiftBottomEdgeTo(LayoutUnit::from_signed(4));
        assert_eq!(rect.Height(), LayoutUnit::from_signed(1));
    }

    #[test]
    fn fractional_box_snaps_edges_and_keeps_nonzero_extent() {
        let rect = PhysicalRect::from_units(
            LayoutUnit::from_f32(1.5),
            LayoutUnit::from_f32(2.5),
            LayoutUnit::from_f32(0.5),
            LayoutUnit::from_f32(0.5),
        );
        let snapped = ToPixelSnappedRect(rect);
        assert_eq!((snapped.x(), snapped.y()), (2, 3));
        assert_eq!((snapped.width(), snapped.height()), (1, 1));
    }
}

impl std::ops::Add<PhysicalOffset> for PhysicalRect {
    type Output = Self;
    fn add(self, offset: PhysicalOffset) -> Self {
        Self::new(self.offset + offset, self.size)
    }
}

impl std::ops::Sub<PhysicalOffset> for PhysicalRect {
    type Output = Self;
    fn sub(self, offset: PhysicalOffset) -> Self {
        Self::new(self.offset - offset, self.size)
    }
}
