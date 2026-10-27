/// Backend-neutral paint-space rectangle.
// cpp: paint/paint_engine.h:70-76
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PaintRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl PaintRect {
    pub fn is_empty(self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }

    pub fn enclosing(self) -> Self {
        let x = self.x.floor();
        let y = self.y.floor();
        Self {
            x,
            y,
            width: if self.width <= 0.0 {
                0.0
            } else {
                (self.x + self.width).ceil() - x
            },
            height: if self.height <= 0.0 {
                0.0
            } else {
                (self.y + self.height).ceil() - y
            },
        }
    }

    pub fn enclosed(self) -> Self {
        let right = self.x + self.width;
        let bottom = self.y + self.height;
        if self.is_empty()
            || ![self.x, self.y, right, bottom]
                .iter()
                .all(|v| v.is_finite())
        {
            return Self::default();
        }
        let x = self.x.ceil();
        let y = self.y.ceil();
        Self {
            x,
            y,
            width: (right.floor() - x).max(0.0),
            height: (bottom.floor() - y).max(0.0),
        }
    }

    pub fn intersection(self, other: Self) -> Self {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = (self.x + self.width).min(other.x + other.width);
        let bottom = (self.y + self.height).min(other.y + other.height);
        Self {
            x,
            y,
            width: (right - x).max(0.0),
            height: (bottom - y).max(0.0),
        }
    }

    pub fn maximum_covered_rect(a: Self, b: Self) -> Self {
        if a.is_empty() {
            return b;
        }
        if b.is_empty() {
            return a;
        }
        let area = |r: Self| r.width * r.height;
        let mut maximum = a;
        let mut maximum_area = area(a);
        if area(b) > maximum_area {
            maximum = b;
            maximum_area = area(b);
        }
        let x = a.x.max(b.x);
        let y = a.y.max(b.y);
        let right = (a.x + a.width).min(b.x + b.width);
        let bottom = (a.y + a.height).min(b.y + b.height);
        if right >= x && bottom >= y && (right > x || bottom > y) {
            let top = a.y.min(b.y);
            let vertical = Self {
                x,
                y: top,
                width: right - x,
                height: (a.y + a.height).max(b.y + b.height) - top,
            };
            if area(vertical) > maximum_area {
                maximum = vertical;
                maximum_area = area(vertical);
            }
            let left = a.x.min(b.x);
            let horizontal = Self {
                x: left,
                y,
                width: (a.x + a.width).max(b.x + b.width) - left,
                height: bottom - y,
            };
            if area(horizontal) > maximum_area {
                maximum = horizontal;
            }
        }
        maximum
    }

    pub fn union(&mut self, other: Self) {
        if other.is_empty() {
            return;
        }
        if self.is_empty() {
            *self = other;
            return;
        }
        let right = (self.x + self.width).max(other.x + other.width);
        let bottom = (self.y + self.height).max(other.y + other.height);
        self.x = self.x.min(other.x);
        self.y = self.y.min(other.y);
        self.width = right - self.x;
        self.height = bottom - self.y;
    }

    pub fn contains(self, inner: Self) -> bool {
        let values = [
            self.x,
            self.y,
            self.width,
            self.height,
            inner.x,
            inner.y,
            inner.width,
            inner.height,
        ];
        values.iter().all(|value| value.is_finite())
            && self.x <= inner.x
            && self.y <= inner.y
            && self.x + self.width >= inner.x + inner.width
            && self.y + self.height >= inner.y + inner.height
    }
}
