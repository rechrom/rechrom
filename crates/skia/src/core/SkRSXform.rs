#![allow(non_snake_case)]

// Source: include/core/SkRSXform.h:24-27
impl SkRSXform {
    #[inline]
    pub fn Make(scos: f32, ssin: f32, tx: f32, ty: f32) -> Self {
        Self {
            fSCos: scos,
            fSSin: ssin,
            fTx: tx,
            fTy: ty,
        }
    }
}

// Source: include/core/SkRSXform.h:35-40
impl SkRSXform {
    #[inline]
    pub fn MakeFromRadians(scale: f32, radians: f32, tx: f32, ty: f32, ax: f32, ay: f32) -> Self {
        let s = crate::include::core::SkScalar::SkScalarSin(radians) * scale;
        let c = crate::include::core::SkScalar::SkScalarCos(radians) * scale;
        Self::Make(c, s, tx + -c * ax + s * ay, ty + -s * ax - c * ay)
    }
}

// Source: include/core/SkRSXform.h:42-45
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SkRSXform {
    pub fSCos: f32,
    pub fSSin: f32,
    pub fTx: f32,
    pub fTy: f32,
}

// Source: include/core/SkRSXform.h:47-49
impl SkRSXform {
    #[inline]
    pub fn rectStaysRect(&self) -> bool {
        self.fSCos == 0.0 || self.fSSin == 0.0
    }
}

// Source: include/core/SkRSXform.h:51-54
impl SkRSXform {
    #[inline]
    pub fn setIdentity(&mut self) {
        self.fSCos = 1.0;
        self.fSSin = 0.0;
        self.fTx = 0.0;
        self.fTy = 0.0;
    }
}

// Source: include/core/SkRSXform.h:56-61
impl SkRSXform {
    #[inline]
    pub fn set(&mut self, scos: f32, ssin: f32, tx: f32, ty: f32) {
        self.fSCos = scos;
        self.fSSin = ssin;
        self.fTx = tx;
        self.fTy = ty;
    }
}

// Source: include/core/SkRSXform.h:64-66
impl SkRSXform {
    #[inline]
    pub fn toQuadSize(
        &self,
        size: &crate::include::core::SkSize::SkSize,
        quad: &mut [crate::src::core::SkPoint::SkPoint; 4],
    ) {
        self.toQuad(size.fWidth, size.fHeight, quad);
    }
}

// Source: src/core/SkRSXform.cpp:9-31
impl SkRSXform {
    pub fn toQuad(
        &self,
        width: f32,
        height: f32,
        quad: &mut [crate::src::core::SkPoint::SkPoint; 4],
    ) {
        let m00 = self.fSCos;
        let m01 = -self.fSSin;
        let m02 = self.fTx;
        let m10 = -m01;
        let m11 = m00;
        let m12 = self.fTy;
        quad[0].set(m02, m12);
        quad[1].set(m00 * width + m02, m10 * width + m12);
        quad[2].set(
            m00 * width + m01 * height + m02,
            m10 * width + m11 * height + m12,
        );
        quad[3].set(m01 * height + m02, m11 * height + m12);
    }
}

// Source: src/core/SkRSXform.cpp:33-45
impl SkRSXform {
    pub fn toTriStrip(
        &self,
        width: f32,
        height: f32,
        strip: &mut [crate::src::core::SkPoint::SkPoint; 4],
    ) {
        let m00 = self.fSCos;
        let m01 = -self.fSSin;
        let m02 = self.fTx;
        let m10 = -m01;
        let m11 = m00;
        let m12 = self.fTy;
        strip[0].set(m02, m12);
        strip[1].set(m01 * height + m02, m11 * height + m12);
        strip[2].set(m00 * width + m02, m10 * width + m12);
        strip[3].set(
            m00 * width + m01 * height + m02,
            m10 * width + m11 * height + m12,
        );
    }
}
