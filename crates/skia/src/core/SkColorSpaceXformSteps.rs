use crate::include::core::SkAlphaType::{SkAlphaType, SkAlphaType::*};
use crate::modules::skcms::skcms::*;
use crate::modules::skcms::src::skcms_public::{
    skcms_Matrix3x3, skcms_TFType::*, skcms_TransferFunction,
};
use crate::src::core::SkColorSpace::{SkColorSpace, SkNamedGamut, SkNamedTransferFn};
// Source: src/core/SkColorSpaceXformSteps.h:21-28
// Source: src/core/SkColorSpaceXformSteps.h:30-39
#[derive(Default, Clone, Copy)]
pub struct Flags {
    pub unpremul: bool,
    pub linearize: bool,
    pub src_ootf: bool,
    pub gamut_transform: bool,
    pub dst_ootf: bool,
    pub encode: bool,
    pub premul: bool,
}
impl Flags {
    pub fn mask(&self) -> u32 {
        (self.unpremul as u32)
            | ((self.linearize as u32) * 2)
            | ((self.src_ootf as u32) * 32)
            | ((self.gamut_transform as u32) * 4)
            | ((self.dst_ootf as u32) * 64)
            | ((self.encode as u32) * 8)
            | ((self.premul as u32) * 16)
    }
}
// Source: src/core/SkColorSpaceXformSteps.h:41-41
// Inactive payload is initialized in Rust; only native flags determine whether it is accessed.
impl Default for SkColorSpaceXformSteps {
    fn default() -> Self {
        Self {
            fFlags: Flags::default(),
            fSrcTF: SkNamedTransferFn::kLinear,
            fDstTFInv: SkNamedTransferFn::kLinear,
            fSrcToDstMatrix: [0.0; 9],
            fSrcOotf: [0.0; 4],
            fDstOotf: [0.0; 4],
        }
    }
}
// Rust expression of native S/D duck typing: borrow the actual descriptor's
// color space and alpha type, without retaining or manufacturing another owner.
// Source: src/core/SkColorSpaceXformSteps.h:45-48
pub trait ColorSpaceAlpha {
    fn colorSpace(&self) -> Option<&SkColorSpace>;
    fn alphaType(&self) -> SkAlphaType;
}
impl ColorSpaceAlpha for super::SkImageInfo::SkImageInfo {
    fn colorSpace(&self) -> Option<&SkColorSpace> {
        self.colorSpace()
    }
    fn alphaType(&self) -> SkAlphaType {
        self.alphaType()
    }
}
impl ColorSpaceAlpha for super::SkImageInfo::SkColorInfo {
    fn colorSpace(&self) -> Option<&SkColorSpace> {
        self.colorSpace()
    }
    fn alphaType(&self) -> SkAlphaType {
        self.alphaType()
    }
}
impl ColorSpaceAlpha for super::SkPixmap::SkPixmap {
    fn colorSpace(&self) -> Option<&SkColorSpace> {
        self.colorSpace()
    }
    fn alphaType(&self) -> SkAlphaType {
        self.alphaType()
    }
}
impl SkColorSpaceXformSteps {
    // Source: src/core/SkColorSpaceXformSteps.h:45-48
    pub fn from_src_dst<S: ColorSpaceAlpha + ?Sized, D: ColorSpaceAlpha + ?Sized>(
        src: &S,
        dst: &D,
    ) -> Self {
        Self::new(
            src.colorSpace(),
            src.alphaType(),
            dst.colorSpace(),
            dst.alphaType(),
        )
    }
}
impl SkColorSpaceXformSteps {
    // Source: src/core/SkColorSpaceXformSteps.h:53-53
    pub fn is_active(&self) -> bool {
        self.fFlags.mask() != 0
    }
}
// Source: src/core/SkColorSpaceXformSteps.h:55-55
// Source: src/core/SkColorSpaceXformSteps.h:57-60
// Source: src/core/SkColorSpaceXformSteps.h:62-62
// Source: src/core/SkColorSpaceXformSteps.h:64-64
pub struct SkColorSpaceXformSteps {
    pub fFlags: Flags,
    pub fSrcTF: skcms_TransferFunction,
    pub fDstTFInv: skcms_TransferFunction,
    pub fSrcToDstMatrix: [f32; 9],
    pub fSrcOotf: [f32; 4],
    pub fDstOotf: [f32; 4],
}
// Source: src/core/SkColorSpaceXformSteps.cpp:27-39
fn set_ootf_Y(cs: &SkColorSpace, Y: &mut [f32; 4]) {
    let rec = SkColorSpace::MakeRGB(&SkNamedTransferFn::kLinear, &SkNamedGamut::kRec2020).unwrap();
    let mut m = skcms_Matrix3x3 {
        vals: [[0.0; 3]; 3],
    };
    cs.gamutTransformTo(&rec, &mut m);
    let yr = [0.262700, 0.678000, 0.059300];
    for i in 0..3 {
        Y[i] = 0.0;
        for j in 0..3 {
            Y[i] += m.vals[j][i] * yr[j];
        }
    }
}
// Source: src/core/SkColorSpaceXformSteps.cpp:41-42
// Source: src/core/SkColorSpaceXformSteps.cpp:45-47
// Source: src/core/SkColorSpaceXformSteps.cpp:51-52
// Source: src/core/SkColorSpaceXformSteps.cpp:54-57
// Source: src/core/SkColorSpaceXformSteps.cpp:59-62
// Source: src/core/SkColorSpaceXformSteps.cpp:66-66
// Source: src/core/SkColorSpaceXformSteps.cpp:70-76
// Source: src/core/SkColorSpaceXformSteps.cpp:79-79
// Source: src/core/SkColorSpaceXformSteps.cpp:81-84
// Source: src/core/SkColorSpaceXformSteps.cpp:87-88
// Source: src/core/SkColorSpaceXformSteps.cpp:90-91
// Source: src/core/SkColorSpaceXformSteps.cpp:93-105
// Source: src/core/SkColorSpaceXformSteps.cpp:107-109
// Source: src/core/SkColorSpaceXformSteps.cpp:111-116
// Source: src/core/SkColorSpaceXformSteps.cpp:118-135
// Source: src/core/SkColorSpaceXformSteps.cpp:137-140
// Source: src/core/SkColorSpaceXformSteps.cpp:142-144
// Source: src/core/SkColorSpaceXformSteps.cpp:146-148
// Source: src/core/SkColorSpaceXformSteps.cpp:150-152
// Source: src/core/SkColorSpaceXformSteps.cpp:154-157
// Source: src/core/SkColorSpaceXformSteps.cpp:159-162
// Source: src/core/SkColorSpaceXformSteps.cpp:164-164
// Source: src/core/SkColorSpaceXformSteps.cpp:167-169
// Source: src/core/SkColorSpaceXformSteps.cpp:172-172
// Source: src/core/SkColorSpaceXformSteps.cpp:174-178
// Source: src/core/SkColorSpaceXformSteps.cpp:181-187
// Source: src/core/SkColorSpaceXformSteps.cpp:190-196
// Source: src/core/SkColorSpaceXformSteps.cpp:198-200
// Source: src/core/SkColorSpaceXformSteps.cpp:203-211
impl SkColorSpaceXformSteps {
    pub fn new(
        src: Option<&SkColorSpace>,
        srcAT: SkAlphaType,
        dst: Option<&SkColorSpace>,
        mut dstAT: SkAlphaType,
    ) -> Self {
        let mut out = Self::default();
        if dstAT == kOpaque_SkAlphaType {
            dstAT = srcAT;
        }
        let srgb = SkColorSpace::MakeSRGB();
        let src = src.unwrap_or(&srgb);
        let dst = dst.unwrap_or(src);
        if src.hash() == dst.hash() && srcAT == dstAT {
            crate::SkASSERT!(@source b"src/core/SkColorSpaceXformSteps.cpp\0".as_ptr(),55; @condition b"SkColorSpace::Equals(src,dst)\0".as_ptr(); SkColorSpace::Equals(Some(src),Some(dst)));
            return out;
        }
        let mut st = SkNamedTransferFn::kLinear;
        src.transferFn(&mut st);
        let mut dt = st;
        dst.transferFn(&mut dt);
        let mut scale = 1.0;
        let pq = skcms_TransferFunction {
            g: -2.0,
            a: -107.0 / 128.0,
            b: 1.0,
            c: 32.0 / 2523.0,
            d: 2413.0 / 128.0,
            e: -2392.0 / 128.0,
            f: 8192.0 / 1305.0,
        };
        let hlg = skcms_TransferFunction {
            g: -3.0,
            a: 2.0,
            b: 2.0,
            c: 1.0 / 0.17883277,
            d: 0.28466892,
            e: 0.55991073,
            f: 0.0,
        };
        let stype = skcms_TransferFunction_getType(&st);
        match stype {
            skcms_TFType_PQ => {
                scale *= 10000.0 / st.a;
                out.fSrcTF = pq;
                out.fFlags.linearize = true;
            }
            skcms_TFType_HLG => {
                scale *= st.b / st.a;
                out.fFlags.linearize = true;
                out.fSrcTF = hlg;
                out.fSrcTF.f = 1.0 / 12.0 - 1.0;
                if st.c != 1.0 {
                    out.fFlags.src_ootf = true;
                    out.fSrcOotf[3] = st.c - 1.0;
                    set_ootf_Y(src, &mut out.fSrcOotf);
                }
            }
            _ => {
                out.fFlags.linearize = tf_bits(st) != tf_bits(SkNamedTransferFn::kLinear);
                if out.fFlags.linearize {
                    src.transferFn(&mut out.fSrcTF);
                }
            }
        }
        match skcms_TransferFunction_getType(&dt) {
            skcms_TFType_PQ => {
                scale /= 10000.0 / dt.a;
                out.fFlags.encode = true;
                out.fDstTFInv = pq;
                let input = out.fDstTFInv;
                skcms_TransferFunction_invert(&input, &mut out.fDstTFInv);
            }
            skcms_TFType_HLG => {
                scale /= dt.b / dt.a;
                out.fFlags.encode = true;
                out.fDstTFInv = hlg;
                out.fDstTFInv.f = 1.0 / 12.0 - 1.0;
                let input = out.fDstTFInv;
                skcms_TransferFunction_invert(&input, &mut out.fDstTFInv);
                if dt.c != 1.0 {
                    out.fFlags.dst_ootf = true;
                    out.fDstOotf[3] = 1.0 / dt.c - 1.0;
                    set_ootf_Y(dst, &mut out.fDstOotf);
                }
            }
            _ => {
                out.fFlags.encode = tf_bits(dt) != tf_bits(SkNamedTransferFn::kLinear);
                if out.fFlags.encode {
                    dst.invTransferFn(&mut out.fDstTFInv);
                }
            }
        }
        out.fFlags.unpremul = srcAT == kPremul_SkAlphaType;
        out.fFlags.gamut_transform = src.toXYZD50Hash() != dst.toXYZD50Hash() || scale != 1.0;
        out.fFlags.premul = srcAT != kOpaque_SkAlphaType && dstAT == kPremul_SkAlphaType;
        if out.fFlags.gamut_transform {
            let mut m = skcms_Matrix3x3 {
                vals: [[0.0; 3]; 3],
            };
            src.gamutTransformTo(dst, &mut m);
            for c in 0..3 {
                for r in 0..3 {
                    out.fSrcToDstMatrix[c * 3 + r] = m.vals[r][c] * scale;
                }
            }
        } else {
            #[cfg(debug_assertions)]
            {
                let mut sm = skcms_Matrix3x3 {
                    vals: [[0.0; 3]; 3],
                };
                let mut dm = sm;
                src.toXYZD50(&mut sm);
                dst.toXYZD50(&mut dm);
                crate::SkASSERT!(@source b"src/core/SkColorSpaceXformSteps.cpp\0".as_ptr(),162; @condition b"0 == memcmp(&srcM, &dstM, 9*sizeof(float)) && \"Hash collision\"\0".as_ptr(); sm.vals.iter().flatten().zip(dm.vals.iter().flatten()).all(|(x,y)|x.to_bits()==y.to_bits()));
            }
        }
        if out.fFlags.src_ootf && !out.fFlags.gamut_transform && out.fFlags.dst_ootf {
            crate::SkASSERT!(@source b"src/core/SkColorSpaceXformSteps.cpp\0".as_ptr(),172; @condition b"0 == memcmp(&this->fSrcOotf, &this->fDstOotf, 3*sizeof(float))\0".as_ptr(); (0..3).all(|i|out.fSrcOotf[i].to_bits()==out.fDstOotf[i].to_bits()));
            if (out.fSrcOotf[3] + 1.0) * (out.fDstOotf[3] + 1.0) == 1.0 {
                out.fFlags.src_ootf = false;
                out.fFlags.dst_ootf = false;
            }
        }
        if out.fFlags.linearize
            && !out.fFlags.src_ootf
            && !out.fFlags.gamut_transform
            && !out.fFlags.dst_ootf
            && out.fFlags.encode
            && src.transferFnHash() == dst.transferFnHash()
        {
            #[cfg(debug_assertions)]
            if stype != skcms_TFType_PQ && stype != skcms_TFType_HLG {
                let a = tf_values(out.fSrcTF);
                let b = tf_values(dt);
                for i in 0..7 {
                    crate::SkASSERT!(@source b"src/core/SkColorSpaceXformSteps.cpp\0".as_ptr(),194; @condition b"(&fSrcTF.g)[i] == (&dstTF.g)[i] && \"Hash collision\"\0".as_ptr(); a[i]==b[i]);
                }
            }
            out.fFlags.linearize = false;
            out.fFlags.encode = false;
        }
        if out.fFlags.unpremul && !out.fFlags.linearize && !out.fFlags.encode && out.fFlags.premul {
            out.fFlags.unpremul = false;
            out.fFlags.premul = false;
        }
        out
    }
}
// Native memcpy/memcmp payload projection.
fn tf_values(t: skcms_TransferFunction) -> [f32; 7] {
    [t.g, t.a, t.b, t.c, t.d, t.e, t.f]
}
fn tf_bits(t: skcms_TransferFunction) -> [u32; 7] {
    tf_values(t).map(f32::to_bits)
}
// Source: src/core/SkColorSpaceXformSteps.cpp:213-214
// Source: src/core/SkColorSpaceXformSteps.cpp:216-216
// Source: src/core/SkColorSpaceXformSteps.cpp:218-265
impl SkColorSpaceXformSteps {
    pub fn apply(&self, rgba: &mut [f32; 4]) {
        if self.fFlags.unpremul {
            let inv = 1.0 / rgba[3];
            let inv = if inv * 0.0 == 0.0 { inv } else { 0.0 };
            rgba[0] *= inv;
            rgba[1] *= inv;
            rgba[2] *= inv;
        }
        if self.fFlags.linearize {
            rgba[0] = skcms_TransferFunction_eval(&self.fSrcTF, rgba[0]);
            rgba[1] = skcms_TransferFunction_eval(&self.fSrcTF, rgba[1]);
            rgba[2] = skcms_TransferFunction_eval(&self.fSrcTF, rgba[2]);
        }
        if self.fFlags.src_ootf {
            let y = self.fSrcOotf[0] * rgba[0]
                + self.fSrcOotf[1] * rgba[1]
                + self.fSrcOotf[2] * rgba[2];
            let p = y.powf(self.fSrcOotf[3]);
            rgba[0] *= p;
            rgba[1] *= p;
            rgba[2] *= p;
        }
        if self.fFlags.gamut_transform {
            let temp = [rgba[0], rgba[1], rgba[2]];
            for i in 0..3 {
                rgba[i] = self.fSrcToDstMatrix[i] * temp[0]
                    + self.fSrcToDstMatrix[3 + i] * temp[1]
                    + self.fSrcToDstMatrix[6 + i] * temp[2];
            }
        }
        if self.fFlags.dst_ootf {
            let y = self.fDstOotf[0] * rgba[0]
                + self.fDstOotf[1] * rgba[1]
                + self.fDstOotf[2] * rgba[2];
            let p = y.powf(self.fDstOotf[3]);
            rgba[0] *= p;
            rgba[1] *= p;
            rgba[2] *= p;
        }
        if self.fFlags.encode {
            rgba[0] = skcms_TransferFunction_eval(&self.fDstTFInv, rgba[0]);
            rgba[1] = skcms_TransferFunction_eval(&self.fDstTFInv, rgba[1]);
            rgba[2] = skcms_TransferFunction_eval(&self.fDstTFInv, rgba[2]);
        }
        if self.fFlags.premul {
            rgba[0] *= rgba[3];
            rgba[1] *= rgba[3];
            rgba[2] *= rgba[3];
        }
    }
}
// Source: src/core/SkColorSpaceXformSteps.cpp:267-275
impl SkColorSpaceXformSteps {
    /// Self must remain live at a stable address for every pipeline retaining its stage context pointers.
    pub unsafe fn applyPipeline(
        &self,
        p: &mut crate::src::core::SkRasterPipeline::SkRasterPipeline,
    ) {
        use crate::src::core::SkRasterPipelineOpList::SkRasterPipelineOp as Op;
        if self.fFlags.unpremul {
            p.append(Op::unpremul, std::ptr::null_mut());
        }
        if self.fFlags.linearize {
            p.appendTransferFunction(&self.fSrcTF);
        }
        if self.fFlags.src_ootf {
            p.append(Op::ootf, self.fSrcOotf.as_ptr() as *mut std::ffi::c_void);
        }
        if self.fFlags.gamut_transform {
            p.append(
                Op::matrix_3x3,
                self.fSrcToDstMatrix.as_ptr() as *mut std::ffi::c_void,
            );
        }
        if self.fFlags.dst_ootf {
            p.append(Op::ootf, self.fDstOotf.as_ptr() as *mut std::ffi::c_void);
        }
        if self.fFlags.encode {
            p.appendTransferFunction(&self.fDstTFInv);
        }
        if self.fFlags.premul {
            p.append(Op::premul, std::ptr::null_mut());
        }
    }
}
