// Copyright 2017 Google Inc. BSD-style license. UNVERIFIED native translation.
use crate::src::core::SkMatrix::SkMatrix;
// Source: include/codec/SkEncodedOrigin.h:14-25
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SkEncodedOrigin {
    kTopLeft_SkEncodedOrigin = 1,
    kTopRight_SkEncodedOrigin = 2,
    kBottomRight_SkEncodedOrigin = 3,
    kBottomLeft_SkEncodedOrigin = 4,
    kLeftTop_SkEncodedOrigin = 5,
    kRightTop_SkEncodedOrigin = 6,
    kRightBottom_SkEncodedOrigin = 7,
    kLeftBottom_SkEncodedOrigin = 8,
}
impl SkEncodedOrigin {
    pub const kDefault_SkEncodedOrigin: Self = Self::kTopLeft_SkEncodedOrigin;
    pub const kLast_SkEncodedOrigin: Self = Self::kLeftBottom_SkEncodedOrigin;
}
// Native unscoped-enum namespace projection; aliases hold no new state.
pub use SkEncodedOrigin::*;
pub const kDefault_SkEncodedOrigin: SkEncodedOrigin = SkEncodedOrigin::kDefault_SkEncodedOrigin;
pub const kLast_SkEncodedOrigin: SkEncodedOrigin = SkEncodedOrigin::kLast_SkEncodedOrigin;
// Source: include/codec/SkEncodedOrigin.h:32-44
pub fn SkEncodedOriginToMatrix(origin: SkEncodedOrigin, w: i32, h: i32) -> SkMatrix {
    let w = w as f32;
    let h = h as f32;
    match origin {
        kTopLeft_SkEncodedOrigin => SkMatrix::I().clone(),
        kTopRight_SkEncodedOrigin => SkMatrix::MakeAll(-1.0, 0.0, w, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
        kBottomRight_SkEncodedOrigin => {
            SkMatrix::MakeAll(-1.0, 0.0, w, 0.0, -1.0, h, 0.0, 0.0, 1.0)
        }
        kBottomLeft_SkEncodedOrigin => {
            SkMatrix::MakeAll(1.0, 0.0, 0.0, 0.0, -1.0, h, 0.0, 0.0, 1.0)
        }
        kLeftTop_SkEncodedOrigin => SkMatrix::MakeAll(0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0),
        kRightTop_SkEncodedOrigin => SkMatrix::MakeAll(0.0, -1.0, w, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0),
        kRightBottom_SkEncodedOrigin => {
            SkMatrix::MakeAll(0.0, -1.0, w, -1.0, 0.0, h, 0.0, 0.0, 1.0)
        }
        kLeftBottom_SkEncodedOrigin => {
            SkMatrix::MakeAll(0.0, 1.0, 0.0, -1.0, 0.0, h, 0.0, 0.0, 1.0)
        }
    }
}
// Source: include/codec/SkEncodedOrigin.h:49-61
pub fn SkEncodedOriginToMatrixInverse(origin: SkEncodedOrigin, w: i32, h: i32) -> SkMatrix {
    let w = w as f32;
    let h = h as f32;
    match origin {
        kTopLeft_SkEncodedOrigin => SkMatrix::I().clone(),
        kTopRight_SkEncodedOrigin => SkMatrix::MakeAll(-1.0, 0.0, w, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
        kBottomRight_SkEncodedOrigin => {
            SkMatrix::MakeAll(-1.0, 0.0, w, 0.0, -1.0, h, 0.0, 0.0, 1.0)
        }
        kBottomLeft_SkEncodedOrigin => {
            SkMatrix::MakeAll(1.0, 0.0, 0.0, 0.0, -1.0, h, 0.0, 0.0, 1.0)
        }
        kLeftTop_SkEncodedOrigin => SkMatrix::MakeAll(0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0),
        kRightTop_SkEncodedOrigin => SkMatrix::MakeAll(0.0, 1.0, 0.0, -1.0, 0.0, w, 0.0, 0.0, 1.0),
        kRightBottom_SkEncodedOrigin => {
            SkMatrix::MakeAll(0.0, -1.0, h, -1.0, 0.0, w, 0.0, 0.0, 1.0)
        }
        kLeftBottom_SkEncodedOrigin => {
            SkMatrix::MakeAll(0.0, -1.0, h, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0)
        }
    }
}
// Source: include/codec/SkEncodedOrigin.h:67-69
pub fn SkEncodedOriginSwapsWidthHeight(origin: SkEncodedOrigin) -> bool {
    origin as i32 >= kLeftTop_SkEncodedOrigin as i32
}
