use std::ops::Sub;

// cpp: layoutng/internal/shapes/shape_interval.h:36-74
#[derive(Clone, Copy, Debug)]
pub struct ShapeInterval<T> {
    x1_: T,
    x2_: T,
}

impl<T> Default for ShapeInterval<T>
where
    T: Copy + From<i8>,
{
    // cpp: layoutng/internal/shapes/shape_interval.h:41-41
    fn default() -> Self {
        Self {
            x1_: T::from(-1),
            x2_: T::from(-2),
        }
    }
}

#[allow(non_snake_case)]
impl<T> ShapeInterval<T>
where
    T: Copy + From<i8> + PartialOrd + PartialEq + Sub<Output = T>,
{
    // cpp: layoutng/internal/shapes/shape_interval.h:42-42
    pub fn new(x1: T, x2: T) -> Self {
        debug_assert!(x2 >= x1);
        Self { x1_: x1, x2_: x2 }
    }

    // cpp: layoutng/internal/shapes/shape_interval.h:44-48
    pub fn IsUndefined(&self) -> bool {
        self.x2_ < self.x1_
    }

    pub fn X1(&self) -> T {
        if self.IsUndefined() {
            T::from(0)
        } else {
            self.x1_
        }
    }

    pub fn X2(&self) -> T {
        if self.IsUndefined() {
            T::from(0)
        } else {
            self.x2_
        }
    }

    pub fn Width(&self) -> T {
        if self.IsUndefined() {
            T::from(0)
        } else {
            self.x2_ - self.x1_
        }
    }

    pub fn IsEmpty(&self) -> bool {
        self.IsUndefined() || self.x1_ == self.x2_
    }

    // cpp: layoutng/internal/shapes/shape_interval.h:50-54
    pub fn Set(&mut self, x1: T, x2: T) {
        debug_assert!(x2 >= x1);
        self.x1_ = x1;
        self.x2_ = x2;
    }

    // cpp: layoutng/internal/shapes/shape_interval.h:55-58
    pub fn Contains(&self, interval: &Self) -> bool {
        !self.IsUndefined()
            && !interval.IsUndefined()
            && self.X1() <= interval.X1()
            && self.X2() >= interval.X2()
    }

    // cpp: layoutng/internal/shapes/shape_interval.h:62-69
    pub fn Unite(&mut self, interval: &Self) {
        if interval.IsUndefined() {
            return;
        }
        if self.IsUndefined() {
            self.Set(interval.X1(), interval.X2());
        } else {
            let left = self.X1();
            let other_left = interval.X1();
            let right = self.X2();
            let other_right = interval.X2();
            self.Set(
                if other_left < left { other_left } else { left },
                if right < other_right {
                    other_right
                } else {
                    right
                },
            );
        }
    }
}

// cpp: layoutng/internal/shapes/shape_interval.h:59-61
impl<T> PartialEq for ShapeInterval<T>
where
    T: Copy + From<i8> + PartialOrd + PartialEq + Sub<Output = T>,
{
    fn eq(&self, other: &Self) -> bool {
        self.X1() == other.X1() && self.X2() == other.X2()
    }
}

impl<T> Eq for ShapeInterval<T> where T: Copy + From<i8> + Ord + Sub<Output = T> {}

// cpp: layoutng/internal/shapes/shape_interval.h:76-76
pub type IntShapeInterval = ShapeInterval<i32>;
