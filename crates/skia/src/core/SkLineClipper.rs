// Copyright 2011 Google Inc. BSD-style license.
use crate::src::core::SkPoint::SkPoint;
use crate::src::core::SkRect::SkRect;
// Source: src/core/SkLineClipper.h:13-18
pub struct SkLineClipper;
impl SkLineClipper {
    pub const kMaxPoints: usize = 4;
    pub const kMaxClippedLineSegments: usize = 3;
}
// Source: src/core/SkLineClipper.cpp:20-34
fn pin_unsorted<T: PartialOrd>(mut value: T, mut limit0: T, mut limit1: T) -> T {
    if limit1 < limit0 {
        std::mem::swap(&mut limit0, &mut limit1);
    }
    crate::SkASSERT!(@source b"src/core/SkLineClipper.cpp\0".as_ptr(), 26; @condition b"limit0 <= limit1\0".as_ptr(); limit0 <= limit1);
    if value < limit0 {
        value = limit0;
    } else if value > limit1 {
        value = limit1;
    }
    value
}
// Source: src/core/SkLineClipper.cpp:37-55
fn sect_with_horizontal(src: &[SkPoint; 2], y: f32) -> f32 {
    let dy = src[1].fY - src[0].fY;
    if dy.abs() <= 1.0 / 4096.0 {
        (0.5 * (src[0].fX as f64 + src[1].fX as f64)) as f32
    } else {
        let x0 = src[0].fX as f64;
        let y0 = src[0].fY as f64;
        let x1 = src[1].fX as f64;
        let y1 = src[1].fY as f64;
        let result = x0 + (y as f64 - y0) * (x1 - x0) / (y1 - y0);
        pin_unsorted(result, x0, x1) as f32
    }
}
// Source: src/core/SkLineClipper.cpp:58-72
fn sect_with_vertical(src: &[SkPoint; 2], x: f32) -> f32 {
    let dx = src[1].fX - src[0].fX;
    if dx.abs() <= 1.0 / 4096.0 {
        (0.5 * (src[0].fY as f64 + src[1].fY as f64)) as f32
    } else {
        let x0 = src[0].fX as f64;
        let y0 = src[0].fY as f64;
        let x1 = src[1].fX as f64;
        let y1 = src[1].fY as f64;
        (y0 + (x as f64 - x0) * (y1 - y0) / (x1 - x0)) as f32
    }
}
// Source: src/core/SkLineClipper.cpp:74-81
fn sect_clamp_with_vertical(src: &[SkPoint; 2], x: f32) -> f32 {
    pin_unsorted(sect_with_vertical(src, x), src[0].fY, src[1].fY)
}
// Source: src/core/SkLineClipper.cpp:85-87
fn nestedLT(a: f32, b: f32, dim: f32) -> bool {
    a <= b && (a < b || dim > 0.0)
}
// Source: src/core/SkLineClipper.cpp:91-95
fn containsNoEmptyCheck(outer: &SkRect, inner: &SkRect) -> bool {
    outer.fLeft <= inner.fLeft
        && outer.fTop <= inner.fTop
        && outer.fRight >= inner.fRight
        && outer.fBottom >= inner.fBottom
}
impl SkLineClipper {
    // Source: src/core/SkLineClipper.cpp:97-167
    pub fn IntersectLine(src: &[SkPoint; 2], clip: &SkRect, dst: &mut [SkPoint; 2]) -> bool {
        // SkRect::set(point,point) uses ordered comparisons rather than f32::min/max,
        // retaining upstream NaN comparison behavior.
        let left = if src[1].fX < src[0].fX {
            src[1].fX
        } else {
            src[0].fX
        };
        let right = if src[0].fX < src[1].fX {
            src[1].fX
        } else {
            src[0].fX
        };
        let top = if src[1].fY < src[0].fY {
            src[1].fY
        } else {
            src[0].fY
        };
        let bottom = if src[0].fY < src[1].fY {
            src[1].fY
        } else {
            src[0].fY
        };
        let bounds = SkRect {
            fLeft: left,
            fRight: right,
            fTop: top,
            fBottom: bottom,
        };
        if containsNoEmptyCheck(clip, &bounds) {
            *dst = *src;
            return true;
        }
        if nestedLT(bounds.fRight, clip.fLeft, bounds.fRight - bounds.fLeft)
            || nestedLT(clip.fRight, bounds.fLeft, bounds.fRight - bounds.fLeft)
            || nestedLT(bounds.fBottom, clip.fTop, bounds.fBottom - bounds.fTop)
            || nestedLT(clip.fBottom, bounds.fTop, bounds.fBottom - bounds.fTop)
        {
            return false;
        }
        let (mut index0, mut index1) = if src[0].fY < src[1].fY {
            (0, 1)
        } else {
            (1, 0)
        };
        let mut tmp = *src;
        if tmp[index0].fY < clip.fTop {
            tmp[index0] = SkPoint {
                fX: sect_with_horizontal(src, clip.fTop),
                fY: clip.fTop,
            };
        }
        if tmp[index1].fY > clip.fBottom {
            tmp[index1] = SkPoint {
                fX: sect_with_horizontal(src, clip.fBottom),
                fY: clip.fBottom,
            };
        }
        (index0, index1) = if tmp[0].fX < tmp[1].fX {
            (0, 1)
        } else {
            (1, 0)
        };
        if tmp[index1].fX <= clip.fLeft || tmp[index0].fX >= clip.fRight {
            if tmp[0].fX != tmp[1].fX || tmp[0].fX < clip.fLeft || tmp[0].fX > clip.fRight {
                return false;
            }
        }
        if tmp[index0].fX < clip.fLeft {
            tmp[index0] = SkPoint {
                fX: clip.fLeft,
                fY: sect_with_vertical(&tmp, clip.fLeft),
            };
        }
        if tmp[index1].fX > clip.fRight {
            tmp[index1] = SkPoint {
                fX: clip.fRight,
                fY: sect_with_vertical(&tmp, clip.fRight),
            };
        }
        crate::SkASSERT!(@source b"src/core/SkLineClipper.cpp\0".as_ptr(), 163; @condition b"containsNoEmptyCheck(clip, bounds)\0".as_ptr(); tmp.iter().all(|p| p.fX >= clip.fLeft
            && p.fX <= clip.fRight
            && p.fY >= clip.fTop
            && p.fY <= clip.fBottom));
        *dst = tmp;
        true
    }
}
// Source: src/core/SkLineClipper.cpp:172-179
#[cfg(sk_debug)]
fn is_between_unsorted(value: f32, limit0: f32, limit1: f32) -> bool {
    if limit0 < limit1 {
        limit0 <= value && value <= limit1
    } else {
        limit1 <= value && value <= limit0
    }
}
impl SkLineClipper {
    // Source: src/core/SkLineClipper.cpp:182-283
    pub fn ClipLine(
        pts: &[SkPoint; 2],
        clip: &SkRect,
        lines: &mut [SkPoint; 4],
        can_cull_to_the_right: bool,
    ) -> usize {
        let (mut index0, mut index1) = if pts[0].fY < pts[1].fY {
            (0, 1)
        } else {
            (1, 0)
        };
        if pts[index1].fY <= clip.fTop {
            return 0;
        }
        if pts[index0].fY >= clip.fBottom {
            return 0;
        }
        let mut tmp = *pts;
        if pts[index0].fY < clip.fTop {
            tmp[index0] = SkPoint {
                fX: sect_with_horizontal(pts, clip.fTop),
                fY: clip.fTop,
            };
            crate::SkASSERT!(@source b"src/core/SkLineClipper.cpp\0".as_ptr(), 211; @condition b"is_between_unsorted(tmp[index0].fX, pts[0].fX, pts[1].fX)\0".as_ptr(); is_between_unsorted(tmp[index0].fX, pts[0].fX, pts[1].fX));
        }
        if tmp[index1].fY > clip.fBottom {
            tmp[index1] = SkPoint {
                fX: sect_with_horizontal(pts, clip.fBottom),
                fY: clip.fBottom,
            };
            crate::SkASSERT!(@source b"src/core/SkLineClipper.cpp\0".as_ptr(), 215; @condition b"is_between_unsorted(tmp[index1].fX, pts[0].fX, pts[1].fX)\0".as_ptr(); is_between_unsorted(tmp[index1].fX, pts[0].fX, pts[1].fX));
        }
        let mut storage = [tmp[0]; 4];
        let mut line_count = 1;
        let mut reverse;
        if pts[0].fX < pts[1].fX {
            index0 = 0;
            index1 = 1;
            reverse = false;
        } else {
            index0 = 1;
            index1 = 0;
            reverse = true;
        }
        if tmp[index1].fX <= clip.fLeft {
            tmp[0].fX = clip.fLeft;
            tmp[1].fX = clip.fLeft;
            storage[..2].copy_from_slice(&tmp);
            reverse = false;
        } else if tmp[index0].fX >= clip.fRight {
            if can_cull_to_the_right {
                return 0;
            }
            tmp[0].fX = clip.fRight;
            tmp[1].fX = clip.fRight;
            storage[..2].copy_from_slice(&tmp);
            reverse = false;
        } else {
            let mut r = 0;
            if tmp[index0].fX < clip.fLeft {
                storage[r] = SkPoint {
                    fX: clip.fLeft,
                    fY: tmp[index0].fY,
                };
                r += 1;
                storage[r] = SkPoint {
                    fX: clip.fLeft,
                    fY: sect_clamp_with_vertical(&tmp, clip.fLeft),
                };
                crate::SkASSERT!(@source b"src/core/SkLineClipper.cpp\0".as_ptr(), 255; @condition b"is_between_unsorted(r->fY, tmp[0].fY, tmp[1].fY)\0".as_ptr(); is_between_unsorted(storage[r].fY, tmp[0].fY, tmp[1].fY));
            } else {
                storage[r] = tmp[index0];
            }
            r += 1;
            if tmp[index1].fX > clip.fRight {
                storage[r] = SkPoint {
                    fX: clip.fRight,
                    fY: sect_clamp_with_vertical(&tmp, clip.fRight),
                };
                crate::SkASSERT!(@source b"src/core/SkLineClipper.cpp\0".as_ptr(), 263; @condition b"is_between_unsorted(r->fY, tmp[0].fY, tmp[1].fY)\0".as_ptr(); is_between_unsorted(storage[r].fY, tmp[0].fY, tmp[1].fY));
                r += 1;
                storage[r] = SkPoint {
                    fX: clip.fRight,
                    fY: tmp[index1].fY,
                };
            } else {
                storage[r] = tmp[index1];
            }
            line_count = r;
        }
        if reverse {
            for i in 0..=line_count {
                lines[line_count - i] = storage[i];
            }
        } else {
            lines[..line_count + 1].copy_from_slice(&storage[..line_count + 1]);
        }
        line_count
    }
}
