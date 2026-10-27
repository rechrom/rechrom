//! Typed geometry dependency boundary for translated cc mathematics. Transform
//! operations are supplied by the owning geometry engine; this module does not
//! introduce a second transform inversion or flattening algorithm.
pub use crate::input_geometry::{Point, PointF, RectF, Size, SizeF, Vector2dF};
use std::collections::BTreeMap;
use std::ops::{Add, AddAssign, Sub};

pub fn cpp_min<T: Copy + PartialOrd>(first: T, second: T) -> T {
    if second < first {
        second
    } else {
        first
    }
}
pub fn cpp_max<T: Copy + PartialOrd>(first: T, second: T) -> T {
    if first < second {
        second
    } else {
        first
    }
}
pub fn cpp_clamp<T: Copy + PartialOrd>(value: T, low: T, high: T) -> T {
    if value < low {
        low
    } else if high < value {
        high
    } else {
        value
    }
}

impl PointF {
    pub fn set_x(&mut self, x: f32) {
        *self = Self::new(x, self.y());
    }
    pub fn set_y(&mut self, y: f32) {
        *self = Self::new(self.x(), y);
    }
    pub fn Scale(&mut self, scale: f32) {
        *self = Self::new(self.x() * scale, self.y() * scale);
    }
}
impl Vector2dF {
    pub fn LengthSquared(self) -> f64 {
        self.x() as f64 * self.x() as f64 + self.y() as f64 * self.y() as f64
    }
    pub fn Length(self) -> f32 {
        self.x().hypot(self.y())
    }
    pub fn IsZero(self) -> bool {
        self.x() == 0.0 && self.y() == 0.0
    }
    pub fn Scale(&mut self, scale: f32) {
        self.set_x(self.x() * scale);
        self.set_y(self.y() * scale);
    }
    pub fn InvScale(&mut self, scale: f32) {
        self.set_x(self.x() / scale);
        self.set_y(self.y() / scale);
    }
    pub fn SetToMin(&mut self, other: Self) {
        self.set_x(cpp_min(self.x(), other.x()));
        self.set_y(cpp_min(self.y(), other.y()));
    }
    pub fn SetToMax(&mut self, other: Self) {
        self.set_x(cpp_max(self.x(), other.x()));
        self.set_y(cpp_max(self.y(), other.y()));
    }
}

impl std::ops::SubAssign for Vector2dF {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl std::ops::AddAssign<Vector2dF> for PointF {
    fn add_assign(&mut self, rhs: Vector2dF) {
        *self = *self + rhs;
    }
}

pub fn RoundedVector2dF(value: Vector2dF) -> Vector2dF {
    Vector2dF::new(
        value.x().round() as i32 as f32,
        value.y().round() as i32 as f32,
    )
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OutsetsF {
    pub top: f32,
    pub left: f32,
    pub bottom: f32,
    pub right: f32,
}

impl RectF {
    pub fn Outset(&mut self, outsets: OutsetsF) {
        *self = Self::new(
            self.x() - outsets.left,
            self.y() - outsets.top,
            self.width() + outsets.left + outsets.right,
            self.height() + outsets.top + outsets.bottom,
        );
    }
}

impl Size {
    pub fn IsEmpty(self) -> bool {
        self.width() == 0 || self.height() == 0
    }
    pub fn Enlarge(&mut self, width: i32, height: i32) {
        *self = Self::new(
            self.width().saturating_add(width),
            self.height().saturating_add(height),
        );
    }
}

impl SizeF {
    pub fn Enlarge(&mut self, width: f32, height: f32) {
        *self = Self::new(self.width() + width, self.height() + height);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector2d {
    pub x: i32,
    pub y: i32,
}
impl Vector2d {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point3F {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl Point3F {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    pub const fn x(self) -> f32 {
        self.x
    }
    pub const fn y(self) -> f32 {
        self.y
    }
    pub const fn z(self) -> f32 {
        self.z
    }
    pub fn set_x(&mut self, x: f32) {
        self.x = x;
    }
    pub fn set_y(&mut self, y: f32) {
        self.y = y;
    }
    pub fn set_z(&mut self, z: f32) {
        self.z = z;
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector3dF {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl Vector3dF {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    pub const fn x(self) -> f32 {
        self.x
    }
    pub const fn y(self) -> f32 {
        self.y
    }
    pub const fn z(self) -> f32 {
        self.z
    }
    pub fn Length(&self) -> f32 {
        ((self.x as f64 * self.x as f64)
            + (self.y as f64 * self.y as f64)
            + (self.z as f64 * self.z as f64))
            .sqrt() as f32
    }
    pub fn Scale(&mut self, scale: f32) {
        self.x *= scale;
        self.y *= scale;
        self.z *= scale;
    }
    pub fn InvScale(&mut self, scale: f32) {
        self.x /= scale;
        self.y /= scale;
        self.z /= scale;
    }
}
impl Sub for Point3F {
    type Output = Vector3dF;
    fn sub(self, other: Self) -> Vector3dF {
        Vector3dF::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}
impl AddAssign for Vector3dF {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}
pub fn CrossProduct(first: Vector3dF, second: Vector3dF) -> Vector3dF {
    Vector3dF::new(
        (first.y as f64 * second.z as f64 - first.z as f64 * second.y as f64) as f32,
        (first.z as f64 * second.x as f64 - first.x as f64 * second.z as f64) as f32,
        (first.x as f64 * second.y as f64 - first.y as f64 * second.x as f64) as f32,
    )
}
pub fn DotProduct3(first: Vector3dF, second: Vector3dF) -> f32 {
    first.x * second.x + first.y * second.y + first.z * second.z
}
pub fn DotProduct2(first: Vector2dF, second: Vector2dF) -> f64 {
    first.x() as f64 * second.x() as f64 + first.y() as f64 * second.y() as f64
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}
impl Rect {
    pub fn set_x(&mut self, x: i32) {
        *self = Self::new(x, self.y, self.width, self.height);
    }
    pub fn set_y(&mut self, y: i32) {
        *self = Self::new(self.x, y, self.width, self.height);
    }
    pub fn set_width(&mut self, width: i32) {
        *self = Self::new(self.x, self.y, width, self.height);
    }
    pub fn set_height(&mut self, height: i32) {
        *self = Self::new(self.x, self.y, self.width, height);
    }
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width: x.saturating_add(width).saturating_sub(x).max(0),
            height: y.saturating_add(height).saturating_sub(y).max(0),
        }
    }
    pub fn FromBounds(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        fn saturated_range(min: i32, max: i32) -> (i32, i32) {
            if max < min {
                return (min, 0);
            }
            let effective_span = max.saturating_sub(min);
            let span_loss = max.saturating_sub(min + effective_span);
            if span_loss == 0 {
                return (min, effective_span);
            }
            if (max as i64).abs() < (i32::MAX / 2) as i64 {
                (max - effective_span, effective_span)
            } else if (min as i64).abs() < (i32::MAX / 2) as i64 {
                (min, effective_span)
            } else {
                (min + span_loss / 2, effective_span)
            }
        }
        let (x, width) = saturated_range(left, right);
        let (y, height) = saturated_range(top, bottom);
        Self {
            x,
            y,
            width,
            height,
        }
    }
    pub fn origin(self) -> Point {
        Point::new(self.x, self.y)
    }
    pub fn bottom_right(self) -> Point {
        Point::new(
            self.x.saturating_add(self.width),
            self.y.saturating_add(self.height),
        )
    }
    pub fn right(self) -> i32 {
        self.x + self.width
    }
    pub fn bottom(self) -> i32 {
        self.y + self.height
    }
    pub fn IsEmpty(self) -> bool {
        self.width == 0 || self.height == 0
    }
    pub fn ContainsPoint(self, point: Point) -> bool {
        point.x() >= self.x
            && point.x() < self.right()
            && point.y() >= self.y
            && point.y() < self.bottom()
    }
    pub fn Contains(self, rect: Self) -> bool {
        rect.x >= self.x
            && rect.right() <= self.right()
            && rect.y >= self.y
            && rect.bottom() <= self.bottom()
    }
    pub fn Intersects(self, rect: Self) -> bool {
        !(self.IsEmpty()
            || rect.IsEmpty()
            || rect.x >= self.right()
            || rect.right() <= self.x
            || rect.y >= self.bottom()
            || rect.bottom() <= self.y)
    }
    pub fn Intersect(&mut self, rect: Self) {
        if self.IsEmpty() || rect.IsEmpty() {
            *self = Self::default();
            return;
        }
        let left = self.x.max(rect.x);
        let top = self.y.max(rect.y);
        let right = self.right().min(rect.right());
        let bottom = self.bottom().min(rect.bottom());
        if left >= right || top >= bottom {
            *self = Self::default();
            return;
        }
        *self = Self::new(left, top, right - left, bottom - top);
    }
    pub fn ToString(self) -> String {
        format!("{},{} {}x{}", self.x, self.y, self.width, self.height)
    }
    // Source dependency semantics: ui/gfx/geometry/rect.cc:199-245.
    pub fn Union(&mut self, rect: Self) {
        if self.IsEmpty() {
            *self = rect;
            return;
        }
        if rect.IsEmpty() {
            return;
        }
        *self = Self::FromBounds(
            self.x.min(rect.x),
            self.y.min(rect.y),
            self.right().max(rect.right()),
            self.bottom().max(rect.bottom()),
        );
    }
    pub fn Subtract(&mut self, rect: Self) {
        if !self.Intersects(rect) {
            return;
        }
        if rect.Contains(*self) {
            *self = Self::default();
            return;
        }
        let mut rx = self.x;
        let mut ry = self.y;
        let mut rr = self.right();
        let mut rb = self.bottom();
        if rect.y <= self.y && rect.bottom() >= self.bottom() {
            if rect.x <= self.x {
                rx = rect.right();
            } else if rect.right() >= self.right() {
                rr = rect.x;
            }
        } else if rect.x <= self.x && rect.right() >= self.right() {
            if rect.y <= self.y {
                ry = rect.bottom();
            } else if rect.bottom() >= self.bottom() {
                rb = rect.y;
            }
        }
        *self = Self::FromBounds(rx, ry, rr, rb);
    }
    // Source dependency semantics: ui/gfx/geometry/rect.cc:119-123.
    pub fn Inset(&mut self, top: i32, left: i32, bottom: i32, right: i32) {
        *self = Self::new(
            self.x.saturating_add(left),
            self.y.saturating_add(top),
            self.width.saturating_sub(left.saturating_add(right)),
            self.height.saturating_sub(top.saturating_add(bottom)),
        );
    }
}
impl Add<Vector2d> for Rect {
    type Output = Self;
    fn add(self, offset: Vector2d) -> Self {
        Self::new(
            self.x.saturating_add(offset.x),
            self.y.saturating_add(offset.y),
            self.width,
            self.height,
        )
    }
}
impl From<Rect> for RectF {
    fn from(rect: Rect) -> Self {
        Self::new(
            rect.x as f32,
            rect.y as f32,
            rect.width as f32,
            rect.height as f32,
        )
    }
}
impl RectF {
    // Source: ui/gfx/geometry/rect_f.h:149
    // Source: ui/gfx/geometry/rect_f.cc:94-97
    pub fn Intersects(self, rect: Self) -> bool {
        !self.IsEmpty()
            && !rect.IsEmpty()
            && rect.x() < self.right()
            && rect.right() > self.x()
            && rect.y() < self.bottom()
            && rect.bottom() > self.y()
    }
    pub fn Contains(self, rect: Self) -> bool {
        rect.x() >= self.x()
            && rect.right() <= self.right()
            && rect.y() >= self.y()
            && rect.bottom() <= self.bottom()
    }
    pub fn IsEmpty(self) -> bool {
        self.width() == 0.0 || self.height() == 0.0
    }
    pub fn CenterPoint(self) -> PointF {
        PointF::new(
            self.x() + self.width() / 2.0,
            self.y() + self.height() / 2.0,
        )
    }
    pub fn set_height(&mut self, height: f32) {
        *self = Self::new(self.x(), self.y(), self.width(), height);
    }
    pub fn Scale(&mut self, x_scale: f32, y_scale: f32) {
        *self = Self::new(
            self.x() * x_scale,
            self.y() * y_scale,
            self.width() * x_scale,
            self.height() * y_scale,
        );
    }
    pub fn Intersect(&mut self, rect: Self) {
        if self.IsEmpty() || rect.IsEmpty() {
            *self = Self::default();
            return;
        }
        let rx = cpp_max(self.x(), rect.x());
        let ry = cpp_max(self.y(), rect.y());
        let rr = cpp_min(self.right(), rect.right());
        let rb = cpp_min(self.bottom(), rect.bottom());
        if rx >= rr || ry >= rb {
            *self = Self::default();
            return;
        }
        *self = Self::new(rx, ry, rr - rx, rb - ry);
    }
    pub fn Union(&mut self, rect: Self) {
        if self.IsEmpty() {
            *self = rect;
            return;
        }
        if rect.IsEmpty() {
            return;
        }
        let rx = cpp_min(self.x(), rect.x());
        let ry = cpp_min(self.y(), rect.y());
        let rr = cpp_max(self.right(), rect.right());
        let rb = cpp_max(self.bottom(), rect.bottom());
        *self = Self::new(rx, ry, rr - rx, rb - ry);
        let mut width = self.width();
        let mut height = self.height();
        if self.right() < rr && width < f32::MAX {
            width = cpp_max(8.0 * f32::EPSILON, width).next_up();
        }
        if self.bottom() < rb && height < f32::MAX {
            height = cpp_max(8.0 * f32::EPSILON, height).next_up();
        }
        *self = Self::new(self.x(), self.y(), width, height);
    }
    pub fn ToString(self) -> String {
        format!(
            "{},{} {}x{}",
            self.x(),
            self.y(),
            self.width(),
            self.height()
        )
    }
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self::FromOriginSize(PointF::new(x, y), SizeF::new(width, height))
    }
    pub fn x(self) -> f32 {
        self.origin().x()
    }
    pub fn y(self) -> f32 {
        self.origin().y()
    }
    pub fn width(self) -> f32 {
        self.size().width()
    }
    pub fn height(self) -> f32 {
        self.size().height()
    }
    pub fn right(self) -> f32 {
        self.x() + self.width()
    }
    pub fn bottom(self) -> f32 {
        self.y() + self.height()
    }
    pub fn top_right(self) -> PointF {
        PointF::new(self.right(), self.y())
    }
    pub fn bottom_left(self) -> PointF {
        PointF::new(self.x(), self.bottom())
    }
    pub fn Inset(&mut self, top: f32, left: f32, bottom: f32, right: f32) {
        *self = Self::new(
            self.x() + left,
            self.y() + top,
            self.width() - (left + right),
            self.height() - (top + bottom),
        );
    }
    pub fn Offset(&mut self, x: f32, y: f32) {
        *self = Self::FromOriginSize(self.origin() + Vector2dF::new(x, y), self.size());
    }
    pub fn set_width(&mut self, width: f32) {
        *self = Self::FromOriginSize(self.origin(), SizeF::new(width, self.height()));
    }
}
impl Add<Vector2dF> for RectF {
    type Output = Self;
    fn add(self, offset: Vector2dF) -> Self {
        Self::new(
            self.x() + offset.x(),
            self.y() + offset.y(),
            self.width(),
            self.height(),
        )
    }
}
pub fn BoundingRect(first: PointF, second: PointF) -> RectF {
    RectF::new(
        cpp_min(first.x(), second.x()),
        cpp_min(first.y(), second.y()),
        (first.x() - second.x()).abs(),
        (first.y() - second.y()).abs(),
    )
}
fn clamp_floor(value: f32) -> i32 {
    value.floor() as i32
}
fn clamp_ceil(value: f32) -> i32 {
    value.ceil() as i32
}
pub fn ToFlooredVector2d(vector: Vector2dF) -> Vector2d {
    Vector2d::new(clamp_floor(vector.x()), clamp_floor(vector.y()))
}
fn rounded_ignoring_error(value: f32, ignore_error: f32) -> f32 {
    let rounded = value.round();
    if (rounded - value).abs() < ignore_error {
        rounded
    } else {
        value
    }
}
pub fn ToEnclosingRectIgnoringError(rect: RectF, ignore_error: f32) -> Rect {
    let x = clamp_floor(rounded_ignoring_error(rect.x(), ignore_error));
    let y = clamp_floor(rounded_ignoring_error(rect.y(), ignore_error));
    let right = if rect.width() != 0.0 {
        clamp_ceil(rounded_ignoring_error(rect.right(), ignore_error))
    } else {
        x
    };
    let bottom = if rect.height() != 0.0 {
        clamp_ceil(rounded_ignoring_error(rect.bottom(), ignore_error))
    } else {
        y
    };
    Rect::FromBounds(x, y, right, bottom)
}
pub fn ToEnclosingRect(rect: RectF) -> Rect {
    ToEnclosingRectIgnoringError(rect, 0.0)
}
pub fn ToEnclosedRect(rect: RectF) -> Rect {
    let x = clamp_ceil(rect.x());
    let y = clamp_ceil(rect.y());
    Rect::FromBounds(x, y, clamp_floor(rect.right()), clamp_floor(rect.bottom()))
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct QuadF {
    pub points: [PointF; 4],
}
impl QuadF {
    pub const fn new(p1: PointF, p2: PointF, p3: PointF, p4: PointF) -> Self {
        Self {
            points: [p1, p2, p3, p4],
        }
    }
    pub fn FromRect(rect: RectF) -> Self {
        Self::new(
            rect.origin(),
            rect.top_right(),
            rect.bottom_right(),
            rect.bottom_left(),
        )
    }
    pub fn BoundingBox(self) -> RectF {
        let mut minimum = self.points[0];
        let mut maximum = minimum;
        for point in &self.points[1..] {
            minimum.SetToMin(*point);
            maximum.SetToMax(*point);
        }
        RectF::new(
            minimum.x(),
            minimum.y(),
            maximum.x() - minimum.x(),
            maximum.y() - minimum.y(),
        )
    }
}
impl AddAssign<Vector2dF> for QuadF {
    fn add_assign(&mut self, offset: Vector2dF) {
        for point in &mut self.points {
            *point = *point + offset;
        }
    }
}

pub trait Transform: Sized {
    fn rc(&self, row: usize, column: usize) -> f64;
    fn TransformVector4(&self, vector: &mut [f32; 4]);
    fn IsIdentityOrIntegerTranslation(&self) -> bool;
    fn IsIdentityOrTranslation(&self) -> bool;
    fn IsScaleOrTranslation(&self) -> bool;
    fn IsScale2d(&self) -> bool;
    fn Preserves2dAxisAlignment(&self) -> bool;
    fn IsFlat(&self) -> bool;
    fn GetCheckedInverse(&self) -> Self;
    fn To2dTranslation(&self) -> Vector2dF;
    fn To2dScale(&self) -> Vector2dF;
}
pub trait Matrix44 {
    fn rc(&self, row: usize, column: usize) -> f32;
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    None,
    Bool(bool),
    Int(i32),
    Double(f64),
    String(String),
    Binary(Vec<u8>),
    List(Vec<Value>),
    Dictionary(BTreeMap<String, Value>),
}
pub trait TracedValue {
    fn BeginDictionary(&mut self, name: &str);
    fn EndDictionary(&mut self);
    fn BeginArray(&mut self, name: &str);
    fn EndArray(&mut self);
    fn SetDouble(&mut self, name: &str, value: f64);
    fn SetInteger(&mut self, name: &str, value: i32);
    fn AppendDouble(&mut self, value: f64);
    fn AppendInteger(&mut self, value: i32);
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BoxF {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub width: f32,
    pub height: f32,
    pub depth: f32,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RRectF {
    pub rect: RectF,
    pub corner_radii: [Vector2dF; 4],
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GradientStep {
    pub fraction: f32,
    pub alpha: u8,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LinearGradient {
    pub angle: i16,
    pub steps: Vec<GradientStep>,
}

/// Typed live gfx::AxisTransform2d value operations. The geometry owner supplies
/// the actual scale/translation representation and its native mapping behavior.
/// Source dependency: ui/gfx/geometry/axis_transform2d.h.
pub trait AxisTransform2d: Clone + Default + PartialEq {
    // Native ui/gfx/geometry/axis_transform2d.h factory; actual value owner.
    fn FromScaleAndTranslation(scale: Vector2dF, translation: Vector2dF) -> Self;
    fn scale(&self) -> &Vector2dF;
    fn translation(&self) -> &Vector2dF;
    fn MapRect(&self, rect: RectF) -> RectF;
    fn InverseMapRect(&self, rect: RectF) -> RectF;
    fn PreScale(&mut self, scale: Vector2dF);
}

// Source dependency: ui/gfx/geometry/rect.h:308-324.
// Preserve edge scaling/rounding and the scale==1 integer-precision fast path.
pub fn ScaleToEnclosingRect(rect: Rect, scale: f32) -> Rect {
    if scale == 1.0 {
        return rect;
    }
    let x = clamp_floor(rect.x as f32 * scale);
    let y = clamp_floor(rect.y as f32 * scale);
    let right = if rect.width == 0 {
        x
    } else {
        clamp_ceil(rect.right() as f32 * scale)
    };
    let bottom = if rect.height == 0 {
        y
    } else {
        clamp_ceil(rect.bottom() as f32 * scale)
    };
    Rect::FromBounds(x, y, right, bottom)
}

// Source: ui/gfx/geometry/rect.h:350-363. ClampRound uses std::round then
// saturated_cast (base/numerics/safe_conversions.h:381-383).
pub fn ScaleToRoundedRect(rect: Rect, scale: f32) -> Rect {
    if scale == 1.0 {
        return rect;
    }
    let x = (rect.x as f32 * scale).round() as i32;
    let y = (rect.y as f32 * scale).round() as i32;
    let right = if rect.width == 0 {
        x
    } else {
        (rect.right() as f32 * scale).round() as i32
    };
    let bottom = if rect.height == 0 {
        y
    } else {
        (rect.bottom() as f32 * scale).round() as i32
    };
    Rect::FromBounds(x, y, right, bottom)
}

// External gfx primitive: ui/gfx/geometry/rect.cc:386-417. This is the
// geometry owner's maximum covered rectangle operation, not a cc hit-test rule.
pub fn MaximumCoveredRect(a: Rect, b: Rect) -> Rect {
    let area = |rect: Rect| rect.width as u64 * rect.height as u64;
    let mut maximum = a;
    let mut maximum_area = area(a);
    if area(b) > maximum_area {
        maximum = b;
        maximum_area = area(b);
    }
    let x = a.x.max(b.x);
    let y = a.y.max(b.y);
    let right = a.right().min(b.right());
    let bottom = a.bottom().min(b.bottom());
    let intersection = if x <= right && y <= bottom {
        Rect::FromBounds(x, y, right, bottom)
    } else {
        Rect::default()
    };
    if intersection.width != 0 || intersection.height != 0 {
        let vertical = Rect::FromBounds(
            intersection.x,
            a.y.min(b.y),
            intersection.right(),
            a.bottom().max(b.bottom()),
        );
        if area(vertical) > maximum_area {
            maximum = vertical;
            maximum_area = area(vertical);
        }
        let horizontal = Rect::FromBounds(
            a.x.min(b.x),
            intersection.y,
            a.right().max(b.right()),
            intersection.bottom(),
        );
        if area(horizontal) > maximum_area {
            maximum = horizontal;
            maximum_area = area(horizontal);
        }
    }
    let _ = maximum_area;
    maximum
}

impl Rect {
    // Source dependency: ui/gfx/geometry/rect.cc:305-312.
    pub fn ManhattanInternalDistance(self, rect: Self) -> i32 {
        let mut c = self;
        c.Union(rect);
        let x = (c.width - self.width - rect.width + 1).max(0);
        let y = (c.height - self.height - rect.height + 1).max(0);
        x + y
    }
}

impl Rect {
    pub fn size(self) -> Size {
        Size::new(self.width, self.height)
    }
}
