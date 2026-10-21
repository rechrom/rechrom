pub struct SkMatrixPriv;

// Source: src/core/SkMatrixPriv.h:25-28
impl SkMatrixPriv {
    pub const kMaxFlattenSize: usize =
        9 * core::mem::size_of::<f32>() + core::mem::size_of::<u32>();
}
// Source: src/core/SkMatrixPriv.h:30-32
impl SkMatrixPriv {
    #[inline]
    pub fn WriteToMemory(
        matrix: &crate::src::core::SkMatrix::SkMatrix,
        buffer: Option<&mut [u8]>,
    ) -> usize {
        matrix.writeToMemory(buffer)
    }
}
// Source: src/core/SkMatrixPriv.h:34-36
impl SkMatrixPriv {
    #[inline]
    pub fn ReadFromMemory(
        matrix: &mut crate::src::core::SkMatrix::SkMatrix,
        buffer: &[u8],
    ) -> usize {
        matrix.readFromMemory(buffer)
    }
}
// Source: src/core/SkMatrixPriv.h:38-38
pub(crate) type MapPtsProc = crate::src::core::SkMatrix::MapPtsProc;
// Source: src/core/SkMatrixPriv.h:41-43
impl SkMatrixPriv {
    #[inline]
    pub(crate) fn GetMapPtsProc(matrix: &crate::src::core::SkMatrix::SkMatrix) -> MapPtsProc {
        matrix.getMapPtsProc()
    }
}
// Source: src/core/SkMatrixPriv.h:49-86
impl SkMatrixPriv {
    #[inline]
    pub fn InverseMapRect(
        mx: &crate::src::core::SkMatrix::SkMatrix,
        dst: &mut crate::src::core::SkRect::SkRect,
        src: &crate::src::core::SkRect::SkRect,
    ) -> bool {
        use crate::src::core::SkRect::SkRect;
        if mx.isScaleTranslate() {
            if mx.getScaleX() == 0.0 || mx.getScaleY() == 0.0 {
                return false;
            }
            let tx = mx.getTranslateX();
            let ty = mx.getTranslateY();
            let mut inverted = [
                src.fLeft - tx,
                src.fTop - ty,
                src.fRight - tx,
                src.fBottom - ty,
            ];
            if mx.getType() > 1 {
                let sx = 1.0 / mx.getScaleX();
                let sy = 1.0 / mx.getScaleY();
                inverted[0] *= sx;
                inverted[1] *= sy;
                inverted[2] *= sx;
                inverted[3] *= sy;
                if sx < 0.0 && sy < 0.0 {
                    inverted = [inverted[2], inverted[3], inverted[0], inverted[1]];
                } else if sx < 0.0 {
                    inverted.swap(0, 2);
                } else if sy < 0.0 {
                    inverted.swap(1, 3);
                }
            }
            *dst = SkRect::MakeLTRB(inverted[0], inverted[1], inverted[2], inverted[3]);
            return true;
        }
        if let Some(inverse) = mx.invert() {
            inverse.mapRect(dst, src);
            return true;
        }
        false
    }
}

// Source: src/core/SkMatrixPriv.h:106-139
impl SkMatrixPriv {
    /// Each strided record must contain a writable aligned SkPoint; records may not overlap.
    #[inline]
    pub unsafe fn MapPointsWithStride(
        mx: &crate::src::core::SkMatrix::SkMatrix,
        mut pts: *mut crate::src::core::SkPoint::SkPoint,
        stride: usize,
        count: usize,
    ) {
        debug_assert!(stride >= core::mem::size_of::<crate::src::core::SkPoint::SkPoint>());
        debug_assert_eq!(stride % core::mem::size_of::<f32>(), 0);
        let tm = mx.getType();
        if tm == 0 {
            return;
        }
        unsafe {
            if tm == 1 {
                let tx = mx.getTranslateX();
                let ty = mx.getTranslateY();
                for _ in 0..count {
                    let p = pts.read();
                    pts.write(crate::src::core::SkPoint::SkPoint::make(
                        p.fX + tx,
                        p.fY + ty,
                    ));
                    pts = pts.wrapping_byte_add(stride);
                }
                return;
            }
            if mx.hasPerspective() {
                for _ in 0..count {
                    pts.write(mx.mapPointPerspective(pts.read()));
                    pts = pts.wrapping_byte_add(stride);
                }
            } else {
                for _ in 0..count {
                    pts.write(mx.mapPointAffine(pts.read()));
                    pts = pts.wrapping_byte_add(stride);
                }
            }
        }
    }
}
// Source: src/core/SkMatrixPriv.h:161-172
impl SkMatrixPriv {
    /// Strided source/destination records must be aligned and valid; aliasing must obey the native forward loop.
    #[inline]
    pub unsafe fn MapPointsWithStrides(
        mx: &crate::src::core::SkMatrix::SkMatrix,
        mut dst: *mut crate::src::core::SkPoint::SkPoint,
        dstStride: usize,
        mut src: *const crate::src::core::SkPoint::SkPoint,
        srcStride: usize,
        count: usize,
    ) {
        debug_assert!(srcStride >= core::mem::size_of::<crate::src::core::SkPoint::SkPoint>());
        debug_assert!(dstStride >= core::mem::size_of::<crate::src::core::SkPoint::SkPoint>());
        debug_assert_eq!(srcStride % core::mem::size_of::<f32>(), 0);
        debug_assert_eq!(dstStride % core::mem::size_of::<f32>(), 0);
        unsafe {
            for _ in 0..count {
                dst.write(mx.mapPoint(src.read()));
                src = src.wrapping_byte_add(srcStride);
                dst = dst.wrapping_byte_add(dstStride);
            }
        }
    }
}
// Source: src/core/SkMatrixPriv.h:177-179
impl SkMatrixPriv {
    #[inline]
    pub fn PostIDiv(
        matrix: &mut crate::src::core::SkMatrix::SkMatrix,
        divx: i32,
        divy: i32,
    ) -> bool {
        matrix.postIDiv(divx, divy)
    }
}
// Source: src/core/SkMatrixPriv.h:181-183
impl SkMatrixPriv {
    #[inline]
    pub fn CheapEqual(
        a: &crate::src::core::SkMatrix::SkMatrix,
        b: &crate::src::core::SkMatrix::SkMatrix,
    ) -> bool {
        if core::ptr::eq(a, b) {
            return true;
        }
        let mut x = [0.0; 9];
        let mut y = [0.0; 9];
        a.get9(&mut x);
        b.get9(&mut y);
        x.iter().zip(y).all(|(u, v)| u.to_bits() == v.to_bits())
    }
}
// Source: src/core/SkMatrixPriv.h:185-185
impl SkMatrixPriv {
    pub fn M44ColMajor(m: &crate::src::core::SkM44::SkM44) -> &[f32; 16] {
        &m.fMat
    }
}
// Source: src/core/SkMatrixPriv.h:190-195
impl SkMatrixPriv {
    #[inline]
    pub fn IsScaleTranslateAsM33(m: &crate::src::core::SkM44::SkM44) -> bool {
        m.rc(1, 0) == 0.0
            && m.rc(3, 0) == 0.0
            && m.rc(0, 1) == 0.0
            && m.rc(3, 1) == 0.0
            && m.rc(3, 3) == 1.0
    }
}
