use crate::include::core::SkAlphaType::SkAlphaType::*;
use crate::include::core::SkColorType::SkColorType::*;
use crate::include::core::{SkRefCnt::sk_sp, SkSize::SkISize};
use crate::include::private::SkTPin::SkTPin;
use crate::src::core::{
    SkArenaAlloc::SkSTArenaAlloc,
    SkColor::SkColor,
    SkColorData::SkPMColor4f,
    SkColorSpace::SkColorSpace,
    SkConvertPixels::SkConvertPixels,
    SkFloatingPoint::sk_float_floor2int_no_saturate,
    SkGeometry::SkCubicCoeff,
    SkImageInfo::SkImageInfo,
    SkMatrix::SkMatrix,
    SkPoint::SkPoint,
    SkRasterPipeline::MissingStage,
    SkVertices::{self, Builder, VertexMode},
};
use std::sync::Arc;

// Source: src/utils/SkPatchUtils.h:20-28
pub struct SkPatchUtils;
impl SkPatchUtils {
    pub const kNumCtrlPts: usize = 12;
    pub const kNumCorners: usize = 4;
    pub const kNumPtsCubic: usize = 4;
}
// Source: src/utils/SkPatchUtils.cpp:35-56
// Rust constants retain aliases where C++ enum discriminants share values.
const kTopP0_CubicCtrlPts: usize = 0;
const kTopP1_CubicCtrlPts: usize = 1;
const kTopP2_CubicCtrlPts: usize = 2;
const kTopP3_CubicCtrlPts: usize = 3;
const kRightP0_CubicCtrlPts: usize = 3;
const kRightP1_CubicCtrlPts: usize = 4;
const kRightP2_CubicCtrlPts: usize = 5;
const kRightP3_CubicCtrlPts: usize = 6;
const kBottomP0_CubicCtrlPts: usize = 9;
const kBottomP1_CubicCtrlPts: usize = 8;
const kBottomP2_CubicCtrlPts: usize = 7;
const kBottomP3_CubicCtrlPts: usize = 6;
const kLeftP0_CubicCtrlPts: usize = 0;
const kLeftP1_CubicCtrlPts: usize = 11;
const kLeftP2_CubicCtrlPts: usize = 10;
const kLeftP3_CubicCtrlPts: usize = 9;
// Source: src/utils/SkPatchUtils.cpp:59-64
const kTopLeft_Corner: usize = 0;
const kTopRight_Corner: usize = 1;
const kBottomRight_Corner: usize = 2;
const kBottomLeft_Corner: usize = 3;

// Source: src/utils/SkPatchUtils.cpp:83-86
// Source: src/utils/SkPatchUtils.cpp:140-143
// Rust requires contiguous fields; their native declaration order is retained.
struct FwDCubicEvaluator {
    fCoefs: SkCubicCoeff,
    fMax: i32,
    fCurrent: i32,
    fDivisions: i32,
    fFwDiff: [SkPoint; 4],
    fPoints: [SkPoint; 4],
}
impl FwDCubicEvaluator {
    // Source: src/utils/SkPatchUtils.cpp:92-97
    fn new(points: &[SkPoint; 4]) -> Self {
        // Zero temporaries are overwritten by restart before any evaluation.
        let mut out = Self {
            fCoefs: SkCubicCoeff::new(points),
            fPoints: *points,
            fMax: 0,
            fCurrent: 0,
            fDivisions: 0,
            fFwDiff: [SkPoint::default(); 4],
        };
        out.restart(1);
        out
    }
    // Source: src/utils/SkPatchUtils.cpp:102-114
    fn restart(&mut self, divisions: i32) {
        self.fDivisions = divisions;
        self.fCurrent = 0;
        self.fMax = self.fDivisions + 1;
        // Coefficient vectors are existing SkPoint two-lane values. Scalar
        // broadcasts retain each native lane's exact multiply/add grouping.
        let h = 1.0 / self.fDivisions as f32;
        let h2 = h * h;
        let h3 = h2 * h;
        let fwDiff3 = self.fCoefs.fA * 6.0 * h3;
        self.fFwDiff[3] = fwDiff3;
        self.fFwDiff[2] = fwDiff3 + (self.fCoefs.fB + self.fCoefs.fB) * h2;
        self.fFwDiff[1] = self.fCoefs.fA * h3 + self.fCoefs.fB * h2 + self.fCoefs.fC * h;
        self.fFwDiff[0] = self.fCoefs.fD;
    }
    // Source: src/utils/SkPatchUtils.cpp:119-121
    fn done(&self) -> bool {
        self.fCurrent > self.fMax
    }
    // Source: src/utils/SkPatchUtils.cpp:126-133
    fn next(&mut self) -> SkPoint {
        let point = self.fFwDiff[0];
        self.fFwDiff[0] = self.fFwDiff[0] + self.fFwDiff[1];
        self.fFwDiff[1] = self.fFwDiff[1] + self.fFwDiff[2];
        self.fFwDiff[2] = self.fFwDiff[2] + self.fFwDiff[3];
        self.fCurrent += 1;
        point
    }
    // Source: src/utils/SkPatchUtils.cpp:135-137
    fn getCtrlPoints(&self) -> &[SkPoint; 4] {
        &self.fPoints
    }
}
// Source: src/utils/SkPatchUtils.cpp:148-148
const kPartitionSize: i32 = 10;
// Source: src/utils/SkPatchUtils.cpp:154-163
fn approx_arc_length(points: &[SkPoint], count: i32) -> f32 {
    if count < 2 {
        return 0.0;
    }
    let mut arcLength = 0.0;
    for i in 0..count - 1 {
        arcLength += SkPoint::distance(points[i as usize], points[i as usize + 1]);
    }
    if arcLength.is_finite() {
        arcLength
    } else {
        -1.0
    }
}
// Source: src/utils/SkPatchUtils.cpp:165-170
fn bilerp(tx: f32, ty: f32, c00: f32, c10: f32, c01: f32, c11: f32) -> f32 {
    let a = c00 * (1.0 - tx) + c10 * tx;
    let b = c01 * (1.0 - tx) + c11 * tx;
    a * (1.0 - ty) + b * ty
}
// Source: src/utils/SkPatchUtils.cpp:172-180
// Four lane scalar expansion of the actual skvx operations, no color conversion.
fn bilerp4(
    tx: f32,
    ty: f32,
    c00: SkPMColor4f,
    c10: SkPMColor4f,
    c01: SkPMColor4f,
    c11: SkPMColor4f,
) -> SkPMColor4f {
    SkPMColor4f {
        fR: bilerp(tx, ty, c00.fR, c10.fR, c01.fR, c11.fR),
        fG: bilerp(tx, ty, c00.fG, c10.fG, c01.fG, c11.fG),
        fB: bilerp(tx, ty, c00.fB, c10.fB, c01.fB, c11.fB),
        fA: bilerp(tx, ty, c00.fA, c10.fA, c01.fA, c11.fA),
    }
}
impl SkPatchUtils {
    // Source: src/utils/SkPatchUtils.h:53-53
    // Source: src/utils/SkPatchUtils.cpp:182-210
    pub fn GetLevelOfDetail(cubics: &[SkPoint; 12], matrix: Option<&SkMatrix>) -> SkISize {
        let matrix = matrix.expect("native patch LOD requires a matrix");
        let mut pts = [SkPoint::default(); 4];
        Self::GetTopCubic(cubics, &mut pts);
        matrix.mapPointsInPlace(&mut pts);
        let topLength = approx_arc_length(&pts, 4);
        Self::GetBottomCubic(cubics, &mut pts);
        matrix.mapPointsInPlace(&mut pts);
        let bottomLength = approx_arc_length(&pts, 4);
        Self::GetLeftCubic(cubics, &mut pts);
        matrix.mapPointsInPlace(&mut pts);
        let leftLength = approx_arc_length(&pts, 4);
        Self::GetRightCubic(cubics, &mut pts);
        matrix.mapPointsInPlace(&mut pts);
        let rightLength = approx_arc_length(&pts, 4);
        if topLength < 0.0 || bottomLength < 0.0 || leftLength < 0.0 || rightLength < 0.0 {
            return SkISize::Make(0, 0);
        }
        // Native float-to-int conversion requires a representable result.
        let lodX = (topLength.max(bottomLength) / kPartitionSize as f32) as i32;
        let lodY = (leftLength.max(rightLength) / kPartitionSize as f32) as i32;
        SkISize::Make(8.max(lodX), 8.max(lodY))
    }
    // Source: src/utils/SkPatchUtils.h:33-33
    // Source: src/utils/SkPatchUtils.cpp:212-217
    pub fn GetTopCubic(cubics: &[SkPoint; 12], points: &mut [SkPoint; 4]) {
        points[0] = cubics[kTopP0_CubicCtrlPts];
        points[1] = cubics[kTopP1_CubicCtrlPts];
        points[2] = cubics[kTopP2_CubicCtrlPts];
        points[3] = cubics[kTopP3_CubicCtrlPts];
    }
    // Source: src/utils/SkPatchUtils.h:38-38
    // Source: src/utils/SkPatchUtils.cpp:219-224
    pub fn GetBottomCubic(cubics: &[SkPoint; 12], points: &mut [SkPoint; 4]) {
        points[0] = cubics[kBottomP0_CubicCtrlPts];
        points[1] = cubics[kBottomP1_CubicCtrlPts];
        points[2] = cubics[kBottomP2_CubicCtrlPts];
        points[3] = cubics[kBottomP3_CubicCtrlPts];
    }
    // Source: src/utils/SkPatchUtils.h:43-43
    // Source: src/utils/SkPatchUtils.cpp:226-231
    pub fn GetLeftCubic(cubics: &[SkPoint; 12], points: &mut [SkPoint; 4]) {
        points[0] = cubics[kLeftP0_CubicCtrlPts];
        points[1] = cubics[kLeftP1_CubicCtrlPts];
        points[2] = cubics[kLeftP2_CubicCtrlPts];
        points[3] = cubics[kLeftP3_CubicCtrlPts];
    }
    // Source: src/utils/SkPatchUtils.h:48-48
    // Source: src/utils/SkPatchUtils.cpp:233-238
    pub fn GetRightCubic(cubics: &[SkPoint; 12], points: &mut [SkPoint; 4]) {
        points[0] = cubics[kRightP0_CubicCtrlPts];
        points[1] = cubics[kRightP1_CubicCtrlPts];
        points[2] = cubics[kRightP2_CubicCtrlPts];
        points[3] = cubics[kRightP3_CubicCtrlPts];
    }
}
// Source: src/utils/SkPatchUtils.cpp:240-246
unsafe fn skcolor_to_float(
    dst: *mut SkPMColor4f,
    src: *const SkColor,
    count: i32,
    dstCS: Arc<SkColorSpace>,
) -> Result<(), MissingStage> {
    let srcInfo = SkImageInfo::MakeWithColorSpace(
        count,
        1,
        kBGRA_8888_SkColorType,
        kUnpremul_SkAlphaType,
        Some(SkColorSpace::MakeSRGB()),
    );
    let dstInfo = SkImageInfo::MakeWithColorSpace(
        count,
        1,
        kRGBA_F32_SkColorType,
        kPremul_SkAlphaType,
        Some(dstCS),
    );
    let success = SkConvertPixels(&dstInfo, dst.cast(), 0, &srcInfo, src.cast(), 0)?;
    crate::SkAssertResult!(@source b"src/utils/SkPatchUtils.cpp\0".as_ptr(), 245; @condition b"SkConvertPixels(dstInfo, dst, 0, srcInfo, src, 0)\0".as_ptr(); success);
    Ok(())
}
// Source: src/utils/SkPatchUtils.cpp:248-254
unsafe fn float_to_skcolor(
    dst: *mut SkColor,
    src: *const SkPMColor4f,
    count: i32,
    srcCS: Arc<SkColorSpace>,
) -> Result<(), MissingStage> {
    let srcInfo = SkImageInfo::MakeWithColorSpace(
        count,
        1,
        kRGBA_F32_SkColorType,
        kPremul_SkAlphaType,
        Some(srcCS),
    );
    let dstInfo = SkImageInfo::MakeWithColorSpace(
        count,
        1,
        kBGRA_8888_SkColorType,
        kUnpremul_SkAlphaType,
        Some(SkColorSpace::MakeSRGB()),
    );
    let success = SkConvertPixels(&dstInfo, dst.cast(), 0, &srcInfo, src.cast(), 0)?;
    crate::SkAssertResult!(@source b"src/utils/SkPatchUtils.cpp\0".as_ptr(), 253; @condition b"SkConvertPixels(dstInfo, dst, 0, srcInfo, src, 0)\0".as_ptr(); success);
    Ok(())
}
impl SkPatchUtils {
    // Source: src/utils/SkPatchUtils.h:55-57
    // Source: src/utils/SkPatchUtils.cpp:256-390
    // Real ConvertPixels is fallible in this crate. Propagate its actual error;
    // never replace missing color stages with a fabricated successful conversion.
    pub fn MakeVertices(
        cubics: Option<&[SkPoint; 12]>,
        srcColors: Option<&[SkColor; 4]>,
        srcTexCoords: Option<&[SkPoint; 4]>,
        mut lodX: i32,
        mut lodY: i32,
        colorSpace: Option<Arc<SkColorSpace>>,
    ) -> Result<sk_sp<SkVertices::SkVertices>, MissingStage> {
        if lodX < 1 || lodY < 1 || cubics.is_none() {
            return Ok(sk_sp::default());
        }
        let cubics = cubics.unwrap();
        // Native adds in int before widening. Do not silently alter its domain.
        let lodX64 = (lodX + 1) as i64;
        let lodY64 = (lodY + 1) as i64;
        let mult64 = lodX64 * lodY64;
        if mult64 > i32::MAX as i64 {
            return Ok(sk_sp::default());
        }
        let colorSpace = colorSpace
            .unwrap_or_else(|| crate::src::core::SkColorSpace::sk_srgb_singleton().clone());
        let mut vertexCount = mult64 as i32;
        if vertexCount > 10000 || lodX > 200 || lodY > 200 {
            let weightX = lodX as f32 / (lodX + lodY) as f32;
            let weightY = lodY as f32 / (lodX + lodY) as f32;
            lodX = 1.max(sk_float_floor2int_no_saturate(weightX * 200.0));
            lodY = 1.max(sk_float_floor2int_no_saturate(weightY * 200.0));
            vertexCount = (lodX + 1) * (lodY + 1);
        }
        let indexCount = lodX * lodY * 6;
        let mut flags = 0;
        if srcTexCoords.is_some() {
            flags |= SkVertices::kHasTexCoords_BuilderFlag;
        }
        if srcColors.is_some() {
            flags |= SkVertices::kHasColors_BuilderFlag;
        }
        // Pin the actual stack arena before exposing interior storage addresses.
        let mut alloc = std::pin::pin!(SkSTArenaAlloc::<2048>::new(0));
        let arena = unsafe { alloc.as_mut().arena() };
        let cornerColors = if srcColors.is_some() {
            arena.makeArrayDefault::<SkPMColor4f>(4)
        } else {
            std::ptr::null_mut()
        };
        let tmpColors = if srcColors.is_some() {
            arena.makeArrayDefault::<SkPMColor4f>(vertexCount as usize)
        } else {
            std::ptr::null_mut()
        };
        let mut builder = Builder::new(
            VertexMode::kTriangles_VertexMode,
            vertexCount,
            indexCount,
            flags,
        );
        let pos = builder.positions();
        let texs = builder.texCoords();
        let indices = builder.indices();
        unsafe {
            if !cornerColors.is_null() {
                skcolor_to_float(
                    cornerColors,
                    srcColors.unwrap().as_ptr(),
                    4,
                    colorSpace.clone(),
                )?;
            }
            let mut pts = [SkPoint::default(); 4];
            Self::GetBottomCubic(cubics, &mut pts);
            let mut fBottom = FwDCubicEvaluator::new(&pts);
            Self::GetTopCubic(cubics, &mut pts);
            let mut fTop = FwDCubicEvaluator::new(&pts);
            Self::GetLeftCubic(cubics, &mut pts);
            let mut fLeft = FwDCubicEvaluator::new(&pts);
            Self::GetRightCubic(cubics, &mut pts);
            let mut fRight = FwDCubicEvaluator::new(&pts);
            fBottom.restart(lodX);
            fTop.restart(lodX);
            let mut u = 0.0;
            let stride = lodY + 1;
            for x in 0..=lodX {
                let bottom = fBottom.next();
                let top = fTop.next();
                fLeft.restart(lodY);
                fRight.restart(lodY);
                let mut v = 0.0;
                for y in 0..=lodY {
                    let dataIndex = (x * (lodY + 1) + y) as usize;
                    let left = fLeft.next();
                    let right = fRight.next();
                    let s0 = SkPoint::make(
                        (1.0 - v) * top.x() + v * bottom.x(),
                        (1.0 - v) * top.y() + v * bottom.y(),
                    );
                    let s1 = SkPoint::make(
                        (1.0 - u) * left.x() + u * right.x(),
                        (1.0 - u) * left.y() + u * right.y(),
                    );
                    let s2 = SkPoint::make(
                        (1.0 - v)
                            * ((1.0 - u) * fTop.getCtrlPoints()[0].x()
                                + u * fTop.getCtrlPoints()[3].x())
                            + v * ((1.0 - u) * fBottom.getCtrlPoints()[0].x()
                                + u * fBottom.getCtrlPoints()[3].x()),
                        (1.0 - v)
                            * ((1.0 - u) * fTop.getCtrlPoints()[0].y()
                                + u * fTop.getCtrlPoints()[3].y())
                            + v * ((1.0 - u) * fBottom.getCtrlPoints()[0].y()
                                + u * fBottom.getCtrlPoints()[3].y()),
                    );
                    pos.add(dataIndex).write(s0 + s1 - s2);
                    if !cornerColors.is_null() {
                        tmpColors.add(dataIndex).write(bilerp4(
                            u,
                            v,
                            *cornerColors.add(kTopLeft_Corner),
                            *cornerColors.add(kTopRight_Corner),
                            *cornerColors.add(kBottomLeft_Corner),
                            *cornerColors.add(kBottomRight_Corner),
                        ));
                    }
                    if !texs.is_null() {
                        let src = srcTexCoords.unwrap();
                        texs.add(dataIndex).write(SkPoint::make(
                            bilerp(
                                u,
                                v,
                                src[kTopLeft_Corner].x(),
                                src[kTopRight_Corner].x(),
                                src[kBottomLeft_Corner].x(),
                                src[kBottomRight_Corner].x(),
                            ),
                            bilerp(
                                u,
                                v,
                                src[kTopLeft_Corner].y(),
                                src[kTopRight_Corner].y(),
                                src[kBottomLeft_Corner].y(),
                                src[kBottomRight_Corner].y(),
                            ),
                        ));
                    }
                    if x < lodX && y < lodY {
                        let i = (6 * (x * lodY + y)) as usize;
                        indices.add(i).write((x * stride + y) as u16);
                        indices.add(i + 1).write((x * stride + 1 + y) as u16);
                        indices.add(i + 2).write(((x + 1) * stride + 1 + y) as u16);
                        indices.add(i + 3).write(*indices.add(i));
                        indices.add(i + 4).write(*indices.add(i + 2));
                        indices.add(i + 5).write(((x + 1) * stride + y) as u16);
                    }
                    v = SkTPin(v + 1.0 / lodY as f32, 0.0, 1.0);
                }
                u = SkTPin(u + 1.0 / lodX as f32, 0.0, 1.0);
            }
            if !tmpColors.is_null() {
                float_to_skcolor(builder.colors(), tmpColors, vertexCount, colorSpace)?;
            }
            Ok(builder.detach())
        }
    }
}
