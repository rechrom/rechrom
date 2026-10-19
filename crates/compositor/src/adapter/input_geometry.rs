//! Geometry dependency boundary for translated impl-input algorithms.
//! Rechrom adapter code: not included in Chromium translation coverage.
//! Native f32 values retain gfx::Vector2dF and SizeF rounding/size-clamp semantics.

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector2dF {
    x_: f32,
    y_: f32,
}

impl std::ops::AddAssign for Vector2dF {
    fn add_assign(&mut self, other: Self) {
        *self = Self::new(self.x() + other.x(), self.y() + other.y());
    }
}
impl Vector2dF {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x_: x, y_: y }
    }
    pub const fn x(self) -> f32 {
        self.x_
    }
    pub const fn y(self) -> f32 {
        self.y_
    }
    pub fn set_x(&mut self, x: f32) {
        self.x_ = x;
    }
    pub fn set_y(&mut self, y: f32) {
        self.y_ = y;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SizeF {
    width_: f32,
    height_: f32,
}
impl SizeF {
    fn clamp(value: f32) -> f32 {
        if value > 8.0 * f32::EPSILON {
            value
        } else {
            0.0
        }
    }
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width_: Self::clamp(width),
            height_: Self::clamp(height),
        }
    }
    pub const fn width(self) -> f32 {
        self.width_
    }
    pub const fn height(self) -> f32 {
        self.height_
    }
    pub fn set_width(&mut self, width: f32) {
        self.width_ = Self::clamp(width);
    }
    pub fn set_height(&mut self, height: f32) {
        self.height_ = Self::clamp(height);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Point {
    x_: i32,
    y_: i32,
}
impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x_: x, y_: y }
    }
    pub const fn x(self) -> i32 {
        self.x_
    }
    pub const fn y(self) -> i32 {
        self.y_
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Size {
    width_: i32,
    height_: i32,
}
impl Size {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width_: width.max(0),
            height_: height.max(0),
        }
    }
    pub const fn width(self) -> i32 {
        self.width_
    }
    pub const fn height(self) -> i32 {
        self.height_
    }
}
impl std::ops::AddAssign for Size {
    fn add_assign(&mut self, other: Self) {
        *self = Self::new(
            self.width().saturating_add(other.width()),
            self.height().saturating_add(other.height()),
        );
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PointF {
    x_: f32,
    y_: f32,
}
impl PointF {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x_: x, y_: y }
    }
    pub const fn x(self) -> f32 {
        self.x_
    }
    pub const fn y(self) -> f32 {
        self.y_
    }
    pub const fn OffsetFromOrigin(self) -> Vector2dF {
        Vector2dF::new(self.x_, self.y_)
    }
    pub fn SetToMin(&mut self, other: Self) {
        if other.x_ < self.x_ {
            self.x_ = other.x_;
        }
        if other.y_ < self.y_ {
            self.y_ = other.y_;
        }
    }
    pub fn SetToMax(&mut self, other: Self) {
        if self.x_ < other.x_ {
            self.x_ = other.x_;
        }
        if self.y_ < other.y_ {
            self.y_ = other.y_;
        }
    }
}
impl std::ops::Add<Vector2dF> for PointF {
    type Output = Self;
    fn add(self, rhs: Vector2dF) -> Self {
        Self::new(self.x_ + rhs.x_, self.y_ + rhs.y_)
    }
}
impl std::ops::Sub<Vector2dF> for PointF {
    type Output = Self;
    fn sub(self, rhs: Vector2dF) -> Self {
        Self::new(self.x_ - rhs.x_, self.y_ - rhs.y_)
    }
}
impl std::ops::Sub for PointF {
    type Output = Vector2dF;
    fn sub(self, rhs: Self) -> Vector2dF {
        Vector2dF::new(self.x_ - rhs.x_, self.y_ - rhs.y_)
    }
}
impl std::ops::Add for Vector2dF {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x_ + rhs.x_, self.y_ + rhs.y_)
    }
}
impl std::ops::Sub for Vector2dF {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x_ - rhs.x_, self.y_ - rhs.y_)
    }
}
impl SizeF {
    pub fn IsEmpty(self) -> bool {
        self.width_ == 0.0 || self.height_ == 0.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RectF {
    origin_: PointF,
    size_: SizeF,
}
impl RectF {
    pub const fn FromSize(size: SizeF) -> Self {
        Self {
            origin_: PointF::new(0.0, 0.0),
            size_: size,
        }
    }
    pub const fn FromOriginSize(origin: PointF, size: SizeF) -> Self {
        Self {
            origin_: origin,
            size_: size,
        }
    }
    pub const fn origin(self) -> PointF {
        self.origin_
    }
    pub const fn size(self) -> SizeF {
        self.size_
    }
    pub fn bottom_right(self) -> PointF {
        PointF::new(
            self.origin_.x_ + self.size_.width_,
            self.origin_.y_ + self.size_.height_,
        )
    }
}
pub fn PointAtOffsetFromOrigin(offset: Vector2dF) -> PointF {
    PointF::new(offset.x(), offset.y())
}
pub fn ScaleVector2d(vector: Vector2dF, x_scale: f32, y_scale: f32) -> Vector2dF {
    Vector2dF::new(vector.x() * x_scale, vector.y() * y_scale)
}
pub fn ScaleSize(size: SizeF, scale: f32) -> SizeF {
    SizeF::new(size.width() * scale, size.height() * scale)
}
