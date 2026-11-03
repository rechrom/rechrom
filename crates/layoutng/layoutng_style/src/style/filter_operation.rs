//! CSS filter value owners used by source computed style. Raster effect
//! construction remains with the renderer; these objects preserve the source
//! class values, equality, and GC lifetime rather than a filter-present flag.
#![allow(non_snake_case)]

use super::box_reflection::BoxReflection;
use super::shadow_data::ShadowData;
use foundation::{
    gfx::{Point, Size},
    Length, LengthPoint, String, Traceable, Visitor,
};
use std::ops::Deref;

// cpp: core/style/filter_operation.h:63-81
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperationType {
    kReference,
    kGrayscale,
    kSepia,
    kSaturate,
    kHueRotate,
    kLuminanceToAlpha,
    kInvert,
    kOpacity,
    kBrightness,
    kContrast,
    kBlur,
    kDropShadow,
    kBoxReflect,
    kColorMatrix,
    kComponentTransfer,
    kConvolveMatrix,
    kTurbulence,
}

// The virtual interface dispatches by the immutable source operation type.
// Every concrete CSS operation retains this base at offset zero.
// cpp: core/style/filter_operation.h:58-149
#[repr(C)]
#[derive(Debug)]
pub struct FilterOperation {
    type_: OperationType,
}

impl FilterOperation {
    fn new(type_: OperationType) -> Self {
        Self { type_ }
    }
    pub fn GetType(&self) -> OperationType {
        self.type_
    }
    pub fn IsSameType(&self, other: &Self) -> bool {
        self.type_ == other.type_
    }
    // cpp: core/style/filter_operation.h:83-106
    pub fn CanInterpolate(type_: OperationType) -> bool {
        !matches!(
            type_,
            OperationType::kReference
                | OperationType::kComponentTransfer
                | OperationType::kConvolveMatrix
                | OperationType::kBoxReflect
        )
    }
    // cpp: core/style/filter_operation.h:114-121,283-285,334-335,378-382
    pub fn AffectsOpacity(&self) -> bool {
        matches!(
            self.type_,
            OperationType::kReference
                | OperationType::kOpacity
                | OperationType::kBlur
                | OperationType::kDropShadow
                | OperationType::kBoxReflect
        )
    }
    pub fn MovesPixels(&self) -> bool {
        matches!(
            self.type_,
            OperationType::kReference
                | OperationType::kBlur
                | OperationType::kDropShadow
                | OperationType::kBoxReflect
        )
    }
    pub fn UsesCurrentColor(&self) -> bool {
        match self.type_ {
            OperationType::kReference => true,
            OperationType::kDropShadow => {
                unsafe { &*(self as *const Self as *const DropShadowFilterOperation) }
                    .Shadow()
                    .GetColor()
                    .DependsOnCurrentColor()
            }
            _ => false,
        }
    }
}

// cpp: core/style/filter_operation.h:112-114,214-219,288-292,347-352,391-396
impl PartialEq for FilterOperation {
    fn eq(&self, other: &Self) -> bool {
        if !self.IsSameType(other) {
            return false;
        }
        unsafe {
            match self.type_ {
                OperationType::kReference => {
                    (*(self as *const Self as *const ReferenceFilterOperation)).url_
                        == (*(other as *const Self as *const ReferenceFilterOperation)).url_
                }
                OperationType::kGrayscale
                | OperationType::kSepia
                | OperationType::kSaturate
                | OperationType::kHueRotate
                | OperationType::kLuminanceToAlpha => {
                    (*(self as *const Self as *const BasicColorMatrixFilterOperation)).amount_
                        == (*(other as *const Self as *const BasicColorMatrixFilterOperation))
                            .amount_
                }
                OperationType::kColorMatrix => {
                    (*(self as *const Self as *const ColorMatrixFilterOperation)).values_
                        == (*(other as *const Self as *const ColorMatrixFilterOperation)).values_
                }
                OperationType::kInvert
                | OperationType::kOpacity
                | OperationType::kBrightness
                | OperationType::kContrast => {
                    (*(self as *const Self as *const BasicComponentTransferFilterOperation)).amount_
                        == (*(other as *const Self as *const BasicComponentTransferFilterOperation))
                            .amount_
                }
                OperationType::kBlur => {
                    (*(self as *const Self as *const BlurFilterOperation)).std_deviation_
                        == (*(other as *const Self as *const BlurFilterOperation)).std_deviation_
                }
                OperationType::kDropShadow => {
                    (*(self as *const Self as *const DropShadowFilterOperation)).shadow_
                        == (*(other as *const Self as *const DropShadowFilterOperation)).shadow_
                }
                OperationType::kConvolveMatrix => {
                    *(self as *const Self as *const ConvolveMatrixFilterOperation)
                        == *(other as *const Self as *const ConvolveMatrixFilterOperation)
                }
                OperationType::kComponentTransfer => {
                    *(self as *const Self as *const ComponentTransferFilterOperation)
                        == *(other as *const Self as *const ComponentTransferFilterOperation)
                }
                OperationType::kTurbulence => {
                    *(self as *const Self as *const TurbulenceFilterOperation)
                        == *(other as *const Self as *const TurbulenceFilterOperation)
                }
                OperationType::kBoxReflect => {
                    *(self as *const Self as *const BoxReflectFilterOperation)
                        == *(other as *const Self as *const BoxReflectFilterOperation)
                }
            }
        }
    }
}

// cpp: core/style/filter_operation.h:151-198. Resource ownership remains in
// the document projection; computed style retains the immutable IRI identity.
#[repr(C)]
pub struct ReferenceFilterOperation {
    base_: FilterOperation,
    url_: String,
}
impl ReferenceFilterOperation {
    pub fn new(url: String) -> Self {
        Self {
            base_: FilterOperation::new(OperationType::kReference),
            url_: url,
        }
    }
    pub fn Url(&self) -> &String {
        &self.url_
    }
}
impl Traceable for FilterOperation {
    fn Trace(&self, _visitor: &mut Visitor<'_>) {}
}

// cpp: core/style/filter_operation.h:200-225
#[repr(C)]
pub struct BasicColorMatrixFilterOperation {
    base_: FilterOperation,
    amount_: f64,
}
impl BasicColorMatrixFilterOperation {
    pub fn new(amount: f64, type_: OperationType) -> Self {
        assert!(matches!(
            type_,
            OperationType::kGrayscale
                | OperationType::kSepia
                | OperationType::kSaturate
                | OperationType::kHueRotate
                | OperationType::kLuminanceToAlpha
        ));
        Self {
            base_: FilterOperation::new(type_),
            amount_: amount,
        }
    }
    pub fn Amount(&self) -> f64 {
        self.amount_
    }
}

// cpp: core/style/filter_operation.h:221-244.
#[repr(C)]
pub struct ColorMatrixFilterOperation {
    base_: FilterOperation,
    values_: Vec<f32>,
}
impl ColorMatrixFilterOperation {
    pub fn new(values: Vec<f32>) -> Self {
        Self {
            base_: FilterOperation::new(OperationType::kColorMatrix),
            values_: values,
        }
    }
    pub fn Values(&self) -> &[f32] {
        &self.values_
    }
}

// cpp: core/style/filter_operation.h:272-299
#[repr(C)]
pub struct BasicComponentTransferFilterOperation {
    base_: FilterOperation,
    amount_: f64,
}
impl BasicComponentTransferFilterOperation {
    pub fn new(amount: f64, type_: OperationType) -> Self {
        assert!(matches!(
            type_,
            OperationType::kInvert
                | OperationType::kOpacity
                | OperationType::kBrightness
                | OperationType::kContrast
        ));
        Self {
            base_: FilterOperation::new(type_),
            amount_: amount,
        }
    }
    pub fn Amount(&self) -> f64 {
        self.amount_
    }
}

// cpp: core/style/filter_operation.h:320-358
#[repr(C)]
pub struct BlurFilterOperation {
    base_: FilterOperation,
    std_deviation_: LengthPoint,
}
impl BlurFilterOperation {
    pub fn new(std_deviation: &Length) -> Self {
        Self {
            base_: FilterOperation::new(OperationType::kBlur),
            std_deviation_: LengthPoint::new(std_deviation, std_deviation),
        }
    }
    pub fn StdDeviation(&self) -> &Length {
        self.std_deviation_.X()
    }
    pub fn StdDeviationXY(&self) -> &LengthPoint {
        &self.std_deviation_
    }
}

// cpp: core/style/filter_operation.h:368-401
#[repr(C)]
pub struct DropShadowFilterOperation {
    base_: FilterOperation,
    shadow_: ShadowData,
}

// cpp: core/style/filter_operation.h:398-416
#[repr(C)]
#[derive(Debug)]
pub struct BoxReflectFilterOperation {
    base_: FilterOperation,
    reflection_: BoxReflection,
}
impl BoxReflectFilterOperation {
    pub fn new(reflection: &BoxReflection) -> Self {
        Self {
            base_: FilterOperation::new(OperationType::kBoxReflect),
            reflection_: reflection.clone(),
        }
    }
    pub fn Reflection(&self) -> &BoxReflection {
        &self.reflection_
    }
}
// cpp: core/style/filter_operation.cc:107-111
impl PartialEq for BoxReflectFilterOperation {
    fn eq(&self, other: &Self) -> bool {
        self.reflection_ == other.reflection_
    }
}

// cpp: platform/graphics/filters/fe_convolve_matrix.h:34-50 and
// core/style/filter_operation.h:425-473.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConvolveMatrixEdgeMode {
    kUnknown,
    kDuplicate,
    kWrap,
    kNone,
}

#[repr(C)]
#[derive(Debug)]
pub struct ConvolveMatrixFilterOperation {
    base_: FilterOperation,
    kernel_size_: Size,
    divisor_: f32,
    bias_: f32,
    target_offset_: Point,
    edge_mode_: ConvolveMatrixEdgeMode,
    preserve_alpha_: bool,
    kernel_matrix_: Vec<f32>,
}
impl ConvolveMatrixFilterOperation {
    pub fn new(
        kernel_size: Size,
        divisor: f32,
        bias: f32,
        target_offset: Point,
        edge_mode: ConvolveMatrixEdgeMode,
        preserve_alpha: bool,
        kernel_matrix: Vec<f32>,
    ) -> Self {
        Self {
            base_: FilterOperation::new(OperationType::kConvolveMatrix),
            kernel_size_: kernel_size,
            divisor_: divisor,
            bias_: bias,
            target_offset_: target_offset,
            edge_mode_: edge_mode,
            preserve_alpha_: preserve_alpha,
            kernel_matrix_: kernel_matrix,
        }
    }
    pub fn KernelSize(&self) -> Size {
        self.kernel_size_
    }
    pub fn Divisor(&self) -> f32 {
        self.divisor_
    }
    pub fn Bias(&self) -> f32 {
        self.bias_
    }
    pub fn TargetOffset(&self) -> Point {
        self.target_offset_
    }
    pub fn EdgeMode(&self) -> ConvolveMatrixEdgeMode {
        self.edge_mode_
    }
    pub fn PreserveAlpha(&self) -> bool {
        self.preserve_alpha_
    }
    pub fn KernelMatrix(&self) -> &[f32] {
        &self.kernel_matrix_
    }
}
impl PartialEq for ConvolveMatrixFilterOperation {
    fn eq(&self, other: &Self) -> bool {
        self.kernel_size_ == other.kernel_size_
            && self.divisor_ == other.divisor_
            && self.bias_ == other.bias_
            && self.target_offset_ == other.target_offset_
            && self.edge_mode_ == other.edge_mode_
            && self.preserve_alpha_ == other.preserve_alpha_
            && self.kernel_matrix_ == other.kernel_matrix_
    }
}

// cpp: platform/graphics/filters/fe_component_transfer.h:32-78 and
// core/style/filter_operation.h:482-514.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComponentTransferType {
    kUnknown,
    kIdentity,
    kTable,
    kDiscrete,
    kLinear,
    kGamma,
}
#[derive(Clone, Debug)]
pub struct ComponentTransferFunction {
    pub r#type: ComponentTransferType,
    pub slope: f32,
    pub intercept: f32,
    pub amplitude: f32,
    pub exponent: f32,
    pub offset: f32,
    pub table_values: Vec<f32>,
}
impl Default for ComponentTransferFunction {
    fn default() -> Self {
        Self {
            r#type: ComponentTransferType::kUnknown,
            slope: 0.0,
            intercept: 0.0,
            amplitude: 0.0,
            exponent: 0.0,
            offset: 0.0,
            table_values: Vec::new(),
        }
    }
}
impl PartialEq for ComponentTransferFunction {
    fn eq(&self, other: &Self) -> bool {
        if self.r#type != other.r#type {
            return false;
        }
        match self.r#type {
            ComponentTransferType::kIdentity => true,
            ComponentTransferType::kLinear => {
                self.slope == other.slope && self.intercept == other.intercept
            }
            ComponentTransferType::kGamma => {
                self.amplitude == other.amplitude
                    && self.exponent == other.exponent
                    && self.offset == other.offset
            }
            ComponentTransferType::kTable | ComponentTransferType::kDiscrete => {
                self.table_values == other.table_values
            }
            ComponentTransferType::kUnknown => false,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct ComponentTransferFilterOperation {
    base_: FilterOperation,
    functions_: [ComponentTransferFunction; 4],
}
impl PartialEq for ComponentTransferFilterOperation {
    fn eq(&self, other: &Self) -> bool {
        self.functions_ == other.functions_
    }
}
impl ComponentTransferFilterOperation {
    pub fn new(functions: [ComponentTransferFunction; 4]) -> Self {
        Self {
            base_: FilterOperation::new(OperationType::kComponentTransfer),
            functions_: functions,
        }
    }
    pub fn Functions(&self) -> &[ComponentTransferFunction; 4] {
        &self.functions_
    }
}

// cpp: platform/graphics/filters/fe_turbulence.h:32-40 and
// core/style/filter_operation.h:523-565.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TurbulenceType {
    kUnknown,
    kFractalNoise,
    kTurbulence,
}
#[repr(C)]
#[derive(Debug)]
pub struct TurbulenceFilterOperation {
    base_: FilterOperation,
    type_: TurbulenceType,
    base_frequency_x_: f32,
    base_frequency_y_: f32,
    num_octaves_: i32,
    seed_: f32,
    stitch_tiles_: bool,
}
impl PartialEq for TurbulenceFilterOperation {
    fn eq(&self, other: &Self) -> bool {
        self.type_ == other.type_
            && self.base_frequency_x_ == other.base_frequency_x_
            && self.base_frequency_y_ == other.base_frequency_y_
            && self.num_octaves_ == other.num_octaves_
            && self.seed_ == other.seed_
            && self.stitch_tiles_ == other.stitch_tiles_
    }
}
impl TurbulenceFilterOperation {
    pub fn new(
        type_: TurbulenceType,
        base_frequency_x: f32,
        base_frequency_y: f32,
        num_octaves: i32,
        seed: f32,
        stitch_tiles: bool,
    ) -> Self {
        Self {
            base_: FilterOperation::new(OperationType::kTurbulence),
            type_: type_,
            base_frequency_x_: base_frequency_x,
            base_frequency_y_: base_frequency_y,
            num_octaves_: num_octaves,
            seed_: seed,
            stitch_tiles_: stitch_tiles,
        }
    }
    pub fn Type(&self) -> TurbulenceType {
        self.type_
    }
    pub fn BaseFrequencyX(&self) -> f32 {
        self.base_frequency_x_
    }
    pub fn BaseFrequencyY(&self) -> f32 {
        self.base_frequency_y_
    }
    pub fn NumOctaves(&self) -> i32 {
        self.num_octaves_
    }
    pub fn Seed(&self) -> f32 {
        self.seed_
    }
    pub fn StitchTiles(&self) -> bool {
        self.stitch_tiles_
    }
}
impl DropShadowFilterOperation {
    pub fn new(shadow: ShadowData) -> Self {
        Self {
            base_: FilterOperation::new(OperationType::kDropShadow),
            shadow_: shadow,
        }
    }
    pub fn Shadow(&self) -> &ShadowData {
        &self.shadow_
    }
}

macro_rules! base_first_css_filter {
    ($($type:ty),+) => { $(
        impl Deref for $type {
            type Target = FilterOperation;
            fn deref(&self) -> &FilterOperation { &self.base_ }
        }
        const _: () = assert!(std::mem::offset_of!($type, base_) == 0);
    )+ };
}
base_first_css_filter!(
    ReferenceFilterOperation,
    BasicColorMatrixFilterOperation,
    ColorMatrixFilterOperation,
    BasicComponentTransferFilterOperation,
    BlurFilterOperation,
    DropShadowFilterOperation,
    BoxReflectFilterOperation,
    ConvolveMatrixFilterOperation,
    ComponentTransferFilterOperation,
    TurbulenceFilterOperation
);
impl Traceable for ReferenceFilterOperation {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.base_.Trace(visitor);
    }
}

impl Traceable for BasicColorMatrixFilterOperation {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.base_.Trace(visitor);
    }
}
impl Traceable for ColorMatrixFilterOperation {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.base_.Trace(visitor);
    }
}
impl Traceable for BasicComponentTransferFilterOperation {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.base_.Trace(visitor);
    }
}
impl Traceable for BlurFilterOperation {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.base_.Trace(visitor);
    }
}
impl Traceable for DropShadowFilterOperation {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.shadow_.Trace(visitor);
        self.base_.Trace(visitor);
    }
}
impl Traceable for BoxReflectFilterOperation {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.base_.Trace(visitor);
    }
}
impl Traceable for ConvolveMatrixFilterOperation {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.base_.Trace(visitor);
    }
}
impl Traceable for ComponentTransferFilterOperation {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.base_.Trace(visitor);
    }
}
impl Traceable for TurbulenceFilterOperation {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.base_.Trace(visitor);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canvas_filter_equality_keeps_source_class_rules() {
        let unknown = || ComponentTransferFunction::default();
        assert_ne!(unknown(), unknown(), "C++ UNKNOWN is never equal");

        let identity = || ComponentTransferFunction {
            r#type: ComponentTransferType::kIdentity,
            ..Default::default()
        };
        let left = ComponentTransferFilterOperation::new(std::array::from_fn(|_| identity()));
        let right = ComponentTransferFilterOperation::new(std::array::from_fn(|_| identity()));
        assert_eq!(&*left, &*right);

        let left =
            TurbulenceFilterOperation::new(TurbulenceType::kFractalNoise, 0.1, 0.2, 3, 4.0, true);
        let right =
            TurbulenceFilterOperation::new(TurbulenceType::kFractalNoise, 0.1, 0.2, 3, 4.0, true);
        assert_eq!(&*left, &*right);
    }
}
