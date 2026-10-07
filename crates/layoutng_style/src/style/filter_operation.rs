//! CSS filter value owners used by native computed style. Raster effect
//! construction remains with the renderer; these objects preserve the source
//! class values, equality, and GC lifetime rather than a filter-present flag.
#![allow(non_snake_case)]

use super::shadow_data::ShadowData;
use foundation::{Length, LengthPoint, Traceable, Visitor};
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

// The virtual interface dispatches by the immutable native operation type.
// Every concrete CSS operation retains this base at offset zero.
// cpp: core/style/filter_operation.h:58-149
#[repr(C)]
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
                OperationType::kGrayscale
                | OperationType::kSepia
                | OperationType::kSaturate
                | OperationType::kHueRotate
                | OperationType::kLuminanceToAlpha => {
                    (*(self as *const Self as *const BasicColorMatrixFilterOperation)).amount_
                        == (*(other as *const Self as *const BasicColorMatrixFilterOperation))
                            .amount_
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
                _ => unimplemented!("native equality for non-CSS filter class {:?}", self.type_),
            }
        }
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
    BasicColorMatrixFilterOperation,
    BasicComponentTransferFilterOperation,
    BlurFilterOperation,
    DropShadowFilterOperation
);

impl Traceable for BasicColorMatrixFilterOperation {
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
