use crate::src::core::SkFloatingPoint::{
    sk_float_ceil2int, sk_float_floor2int, sk_float_round2int,
};

// Source: include/core/SkSize.h:16-48
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SkISize {
    pub fWidth: i32,
    pub fHeight: i32,
}
impl SkISize {
    #[inline]
    pub const fn Make(w: i32, h: i32) -> Self {
        Self {
            fWidth: w,
            fHeight: h,
        }
    }
    #[inline]
    pub const fn MakeEmpty() -> Self {
        Self::Make(0, 0)
    }
    #[inline]
    pub fn set(&mut self, w: i32, h: i32) {
        *self = Self::Make(w, h);
    }
    #[inline]
    pub fn isZero(&self) -> bool {
        self.fWidth == 0 && self.fHeight == 0
    }
    #[inline]
    pub fn isEmpty(&self) -> bool {
        self.fWidth <= 0 || self.fHeight <= 0
    }
    #[inline]
    pub fn setEmpty(&mut self) {
        self.fWidth = 0;
        self.fHeight = 0;
    }
    #[inline]
    pub const fn width(&self) -> i32 {
        self.fWidth
    }
    #[inline]
    pub const fn height(&self) -> i32 {
        self.fHeight
    }
    #[inline]
    pub const fn area(&self) -> i64 {
        self.fWidth as i64 * self.fHeight as i64
    }
    #[inline]
    pub fn equals(&self, w: i32, h: i32) -> bool {
        self.fWidth == w && self.fHeight == h
    }
}

// Source: include/core/SkSize.h:52-92
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkSize {
    pub fWidth: f32,
    pub fHeight: f32,
}
impl SkSize {
    #[inline]
    pub const fn Make(w: f32, h: f32) -> Self {
        Self {
            fWidth: w,
            fHeight: h,
        }
    }
    #[inline]
    pub const fn FromISize(src: &SkISize) -> Self {
        Self::Make(src.fWidth as f32, src.fHeight as f32)
    }
    #[inline]
    pub const fn MakeEmpty() -> Self {
        Self::Make(0.0, 0.0)
    }
    #[inline]
    pub fn set(&mut self, w: f32, h: f32) {
        *self = Self::Make(w, h);
    }
    #[inline]
    pub fn isZero(&self) -> bool {
        self.fWidth == 0.0 && self.fHeight == 0.0
    }
    #[inline]
    pub fn isEmpty(&self) -> bool {
        self.fWidth <= 0.0 || self.fHeight <= 0.0
    }
    #[inline]
    pub fn setEmpty(&mut self) {
        *self = Self::Make(0.0, 0.0);
    }
    #[inline]
    pub fn width(&self) -> f32 {
        self.fWidth
    }
    #[inline]
    pub fn height(&self) -> f32 {
        self.fHeight
    }
    #[inline]
    pub fn equals(&self, w: f32, h: f32) -> bool {
        self.fWidth == w && self.fHeight == h
    }
    #[inline]
    pub fn toRound(&self) -> SkISize {
        SkISize::Make(
            sk_float_round2int(self.fWidth),
            sk_float_round2int(self.fHeight),
        )
    }
    #[inline]
    pub fn toCeil(&self) -> SkISize {
        SkISize::Make(
            sk_float_ceil2int(self.fWidth),
            sk_float_ceil2int(self.fHeight),
        )
    }
    #[inline]
    pub fn toFloor(&self) -> SkISize {
        SkISize::Make(
            sk_float_floor2int(self.fWidth),
            sk_float_floor2int(self.fHeight),
        )
    }
}
