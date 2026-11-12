// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/layout/geometry/physical_rect.h:169-171,218-227
// Source: third_party/blink/renderer/core/layout/geometry/physical_rect.cc:41-51
// The source value storage is the existing shared foundation owner.
pub use foundation::PhysicalRect;
use foundation::{gfx, PhysicalOffset};

pub trait PhysicalRectHitTestExt {
    fn Center(&self) -> PhysicalOffset;
    fn Contains(&self, other: &Self) -> bool;
    fn Intersects(&self, other: &Self) -> bool;
    fn ToEnclosingRect(&self) -> gfx::Rect;
}

impl PhysicalRectHitTestExt for PhysicalRect {
    // Source: physical_rect.h:169-171
    fn Center(&self) -> PhysicalOffset {
        self.offset + PhysicalOffset::new(self.size.width / 2, self.size.height / 2)
    }
    // Source: physical_rect.cc:41-44
    fn Contains(&self, other: &Self) -> bool {
        self.offset.left <= other.offset.left
            && self.offset.top <= other.offset.top
            && self.Right() >= other.Right()
            && self.Bottom() >= other.Bottom()
    }
    // Source: physical_rect.cc:46-51
    fn Intersects(&self, other: &Self) -> bool {
        // Source tests both nonpositive dimensions (PhysicalRect::IsEmpty).
        self.Width() > foundation::LayoutUnit::default()
            && self.Height() > foundation::LayoutUnit::default()
            && other.Width() > foundation::LayoutUnit::default()
            && other.Height() > foundation::LayoutUnit::default()
            && self.offset.left < other.Right()
            && other.offset.left < self.Right()
            && self.offset.top < other.Bottom()
            && other.offset.top < self.Bottom()
    }
    // Source: physical_rect.h:218-227
    fn ToEnclosingRect(&self) -> gfx::Rect {
        let location = gfx::Point::new(self.offset.left.Floor(), self.offset.top.Floor());
        let max_point = self.MaxXMaxYCorner();
        gfx::Rect::new(
            location,
            gfx::Size::new(
                max_point.left.Ceil() - location.x(),
                max_point.top.Ceil() - location.y(),
            ),
        )
    }
}
