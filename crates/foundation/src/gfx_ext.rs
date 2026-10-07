// C++: src/foundation/gfx_geometry/{rect,rect_f,outsets_f}.h, selected value API.
use foundation_base::gfx_geometry::{Point, PointF, Size, SizeF};
use foundation_base::gfx_geometry::{Vector2d, Vector2dF};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    origin: Point,
    size: Size,
}

impl Rect {
    fn clamped_dimension(origin: i32, dimension: i32) -> i32 {
        (((i64::from(origin) + i64::from(dimension))
            .clamp(i64::from(i32::MIN), i64::from(i32::MAX))
            - i64::from(origin))
        .clamp(i64::from(i32::MIN), i64::from(i32::MAX))) as i32
    }

    pub fn new(origin: Point, size: Size) -> Self {
        Self {
            origin,
            size: Size::new(
                Self::clamped_dimension(origin.x(), size.width()),
                Self::clamped_dimension(origin.y(), size.height()),
            ),
        }
    }
    pub fn origin(&self) -> Point {
        self.origin
    }
    // cpp: foundation/gfx_geometry/rect.h:73-74
    pub fn size(&self) -> Size {
        self.size
    }
    pub fn x(&self) -> i32 {
        self.origin.x()
    }
    pub fn y(&self) -> i32 {
        self.origin.y()
    }
    pub fn width(&self) -> i32 {
        self.size.width()
    }
    pub fn height(&self) -> i32 {
        self.size.height()
    }
    // cpp: foundation/gfx_geometry/rect.h:84-87
    pub fn right(&self) -> i32 {
        self.x() + self.width()
    }
    pub fn bottom(&self) -> i32 {
        self.y() + self.height()
    }
    // cpp: foundation/gfx_geometry/rect.h:152-152
    pub fn IsEmpty(&self) -> bool {
        self.size.IsEmpty()
    }
    // cpp: foundation/gfx_geometry/rect.h:195-195
    // cpp: foundation/gfx_geometry/rect.cc:167-181
    pub fn Union(&mut self, rect: Self) {
        if self.IsEmpty() {
            *self = rect;
            return;
        }
        if rect.IsEmpty() {
            return;
        }
        let left = self.x().min(rect.x());
        let top = self.y().min(rect.y());
        let right = self.right().max(rect.right());
        let bottom = self.bottom().max(rect.bottom());
        *self = Self::new(
            Point::new(left, top),
            Size::new(right.saturating_sub(left), bottom.saturating_sub(top)),
        );
    }
    // cpp: foundation/gfx_geometry/rect.h:50-67,142-145
    pub fn set_x(&mut self, x: i32) {
        *self = Self::new(Point::new(x, self.y()), self.size);
    }
    pub fn set_y(&mut self, y: i32) {
        *self = Self::new(Point::new(self.x(), y), self.size);
    }
    pub fn set_width(&mut self, width: i32) {
        *self = Self::new(self.origin, Size::new(width, self.height()));
    }
    pub fn set_height(&mut self, height: i32) {
        *self = Self::new(self.origin, Size::new(self.width(), height));
    }
    pub fn Offset(&mut self, horizontal: i32, vertical: i32) {
        *self = Self::new(
            Point::new(
                self.x().saturating_add(horizontal),
                self.y().saturating_add(vertical),
            ),
            self.size,
        );
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RectF {
    origin: PointF,
    size: SizeF,
}

impl RectF {
    pub fn new(origin: PointF, size: SizeF) -> Self {
        Self { origin, size }
    }
    pub fn origin(&self) -> PointF {
        self.origin
    }
    pub fn size(&self) -> SizeF {
        self.size
    }
    pub fn x(&self) -> f32 {
        self.origin.x()
    }
    pub fn y(&self) -> f32 {
        self.origin.y()
    }
    pub fn width(&self) -> f32 {
        self.size.width()
    }
    pub fn height(&self) -> f32 {
        self.size.height()
    }
    pub fn right(&self) -> f32 {
        self.x() + self.width()
    }
    pub fn bottom(&self) -> f32 {
        self.y() + self.height()
    }

    // cpp: foundation/gfx_geometry/rect_f.h:87-88
    // cpp: foundation/gfx_geometry/rect_f.cc:33-37
    pub fn Inset(&mut self, inset: f32) {
        self.origin = PointF::new(self.x() + inset, self.y() + inset);
        self.size = SizeF::new(self.width() - 2.0 * inset, self.height() - 2.0 * inset);
    }

    // cpp: foundation/gfx_geometry/rect_f.h:106-106
    pub fn IsEmpty(&self) -> bool {
        self.size.IsEmpty()
    }

    // cpp: foundation/gfx_geometry/rect_f.h:80-80
    pub fn OffsetFromOrigin(&self) -> Vector2dF {
        Vector2dF::new(self.x(), self.y())
    }

    // cpp: foundation/gfx_geometry/rect_f.cc:36-38
    pub fn Offset(&mut self, horizontal: f32, vertical: f32) {
        self.origin = PointF::new(self.x() + horizontal, self.y() + vertical);
    }

    // cpp: foundation/gfx_geometry/rect_f.h:211-219
    pub fn Scale(&mut self, x_scale: f32, y_scale: f32) {
        self.origin = PointF::new(self.x() * x_scale, self.y() * y_scale);
        self.size = SizeF::new(self.width() * x_scale, self.height() * y_scale);
    }

    // cpp: foundation/gfx_geometry/rect_f.cc:70-73
    pub fn InclusiveContains(&self, point: PointF) -> bool {
        point.x() >= self.x()
            && point.x() <= self.right()
            && point.y() >= self.y()
            && point.y() <= self.bottom()
    }

    // cpp: foundation/gfx_geometry/rect_f.cc:120-150
    pub fn Union(&mut self, rect: RectF) {
        if self.size.IsEmpty() {
            *self = rect;
            return;
        }
        if rect.size.IsEmpty() {
            return;
        }
        let left = self.x().min(rect.x());
        let top = self.y().min(rect.y());
        let right = self.right().max(rect.right());
        let bottom = self.bottom().max(rect.bottom());
        self.origin = PointF::new(left, top);
        self.size = SizeF::new(right - left, bottom - top);
        if self.right() < right && self.width() < f32::MAX {
            self.size.SetToNextWidth();
        }
        if self.bottom() < bottom && self.height() < f32::MAX {
            self.size.SetToNextHeight();
        }
    }
}

// cpp: foundation/blink_geometry/geometry/physical_rect.h:173-175
impl From<crate::PhysicalRect> for RectF {
    fn from(rect: crate::PhysicalRect) -> Self {
        Self::new(
            PointF::new(rect.X().ToFloat(), rect.Y().ToFloat()),
            SizeF::new(rect.Width().ToFloat(), rect.Height().ToFloat()),
        )
    }
}

// cpp: foundation/gfx_geometry/rect_f.h:31-31
impl From<SizeF> for RectF {
    fn from(size: SizeF) -> Self {
        Self::new(PointF::default(), size)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OutsetsF {
    top: f32,
    right: f32,
    bottom: f32,
    left: f32,
}

impl OutsetsF {
    pub fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }
    pub fn top(&self) -> f32 {
        self.top
    }
    pub fn right(&self) -> f32 {
        self.right
    }
    pub fn bottom(&self) -> f32 {
        self.bottom
    }
    pub fn left(&self) -> f32 {
        self.left
    }
    pub fn set_top(&mut self, value: f32) {
        self.top = value;
    }
    pub fn set_right(&mut self, value: f32) {
        self.right = value;
    }
    pub fn set_bottom(&mut self, value: f32) {
        self.bottom = value;
    }
    pub fn set_left(&mut self, value: f32) {
        self.left = value;
    }
}

pub fn TransposeSize(size: SizeF) -> SizeF {
    SizeF::new(size.height(), size.width())
}

// cpp: foundation/gfx_geometry/vector2d_conversions.cc:20-25
pub fn ToRoundedVector2d(vector: &Vector2dF) -> Vector2d {
    Vector2d::new(
        (vector.x() + 0.5).floor() as i32,
        (vector.y() + 0.5).floor() as i32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounded_vector_uses_floor_after_half_pixel_for_negative_offsets() {
        let rounded = ToRoundedVector2d(&Vector2dF::new(-1.5, -1.6));
        assert_eq!((rounded.x(), rounded.y()), (-1, -2));
    }

    #[test]
    fn rect_union_ignores_empty_rect_and_bounds_both_nonempty_rects() {
        let mut rect = RectF::new(PointF::new(5.0, 8.0), SizeF::new(4.0, 3.0));
        rect.Union(RectF::default());
        assert_eq!(
            (rect.x(), rect.y(), rect.width(), rect.height()),
            (5.0, 8.0, 4.0, 3.0)
        );
        rect.Union(RectF::new(PointF::new(2.0, 10.0), SizeF::new(5.0, 6.0)));
        assert_eq!(
            (rect.x(), rect.y(), rect.width(), rect.height()),
            (2.0, 8.0, 7.0, 8.0)
        );
    }
}
