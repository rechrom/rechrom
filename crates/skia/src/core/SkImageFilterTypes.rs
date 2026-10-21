#![allow(non_snake_case)]
use crate::include::core::SkScalar::{SkScalarCeilToInt, SkScalarFloorToInt};
use crate::include::core::SkSize::{SkISize, SkSize};
use crate::src::core::SkFloatingPoint::SkIsFinite;
use crate::src::core::SkMatrix::SkMatrix;
use crate::src::core::SkPoint::{SkIPoint, SkPoint};
use crate::src::core::SkRect::{SkIRect, SkRect};
use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

// Source: src/core/SkImageFilterTypes.h:63-81
// Default construction of uninitialized C++ POD is omitted; new() requires initialized coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IVector {
    pub fX: i32,
    pub fY: i32,
}
impl IVector {
    // Source: src/core/SkImageFilterTypes.h:67-67
    // Adapted: native POD default construction leaves every coordinate
    // indeterminate. Do not create an initialized Rust value or read those bytes.
    pub const fn uninitialized() -> core::mem::MaybeUninit<Self> {
        core::mem::MaybeUninit::uninit()
    }
    pub fn new(x: i32, y: i32) -> Self {
        Self { fX: x, fY: y }
    }
    pub fn from_vector(v: SkIPoint) -> Self {
        Self::new(v.fX, v.fY)
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vector {
    pub fX: f32,
    pub fY: f32,
}
impl Vector {
    // Source: src/core/SkImageFilterTypes.h:76-76
    // Adapted: native POD default construction leaves every coordinate
    // indeterminate. Do not create an initialized Rust value or read those bytes.
    pub const fn uninitialized() -> core::mem::MaybeUninit<Self> {
        core::mem::MaybeUninit::uninit()
    }
    pub fn new(x: f32, y: f32) -> Self {
        Self { fX: x, fY: y }
    }
    pub fn from_vector(v: SkPoint) -> Self {
        Self::new(v.fX, v.fY)
    }
    pub fn isFinite(&self) -> bool {
        SkIsFinite(&[self.fX, self.fY])
    }
}

// Source: src/core/SkImageFilterTypes.h:107-118
// Rust initialized-value constructor replaces C++ move/copy overloads. No Deref exposes untagged operations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParameterSpace<T> {
    fData: T,
}
impl<T> ParameterSpace<T> {
    // Source: src/core/SkImageFilterTypes.h:111-111
    // Adapted: Rust's named borrowed overload copies the actual wrapped value.
    pub fn from_ref(data: &T) -> Self
    where
        T: Clone,
    {
        Self {
            fData: data.clone(),
        }
    }
    pub fn new(data: T) -> Self {
        Self { fData: data }
    }
    pub fn geometry(&self) -> &T {
        &self.fData
    }
}

// Source: src/core/SkImageFilterTypes.h:126-137
// Rust initialized-value constructor replaces C++ move/copy overloads. No Deref exposes untagged operations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeviceSpace<T> {
    fData: T,
}
impl<T> DeviceSpace<T> {
    // Source: src/core/SkImageFilterTypes.h:130-130
    // Adapted: Rust's named borrowed overload copies the actual wrapped value.
    pub fn from_ref(data: &T) -> Self
    where
        T: Clone,
    {
        Self {
            fData: data.clone(),
        }
    }
    pub fn new(data: T) -> Self {
        Self { fData: data }
    }
    pub fn geometry(&self) -> &T {
        &self.fData
    }
}

// Source: src/core/SkImageFilterTypes.h:110,129
// Concrete native default-initialization mappings only. These POD geometry
// members have no native user constructor/member initializer: no initialized T
// exists until callers write it. No blanket Rust Default or zero fill is used.
// Unknown T has no default_construct method; it must supply an actual translated
// constructor rather than inherit a guessed generic policy.
macro_rules! native_trivial_space_default {
    ($($ty:ty),* $(,)?) => {$(
        impl ParameterSpace<$ty> {
            pub const fn default_construct() -> core::mem::MaybeUninit<Self> {
                core::mem::MaybeUninit::uninit()
            }
        }
        impl DeviceSpace<$ty> {
            pub const fn default_construct() -> core::mem::MaybeUninit<Self> {
                core::mem::MaybeUninit::uninit()
            }
        }
    )*};
}
native_trivial_space_default!(IVector, Vector, SkIPoint, SkPoint, SkISize, SkSize, SkIRect, SkRect);

// Source: src/core/SkImageFilterTypes.h:110,129
// These actual canonical geometry constructors perform native initialization:
// Matrix identity; RRect empty via native member initializers; Path default fill.
// Each field is constructed exactly once, without a new owner or provider.
macro_rules! native_constructed_space_default {
    ($($ty:ty),* $(,)?) => {$(
        impl ParameterSpace<$ty> {
            pub fn default_construct() -> Self {
                Self::new(<$ty>::default())
            }
        }
        impl DeviceSpace<$ty> {
            pub fn default_construct() -> Self {
                Self::new(<$ty>::default())
            }
        }
    )*};
}
native_constructed_space_default!(
    SkMatrix,
    crate::src::core::SkRRect::SkRRect,
    crate::src::core::SkPath::SkPath
);

// Source: src/core/SkImageFilterTypes.h:150-151
// Rust initialized-value constructor replaces C++ move/copy overloads. No Deref exposes untagged operations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayerSpace<T> {
    fData: T,
}
// Rust specialization adaptation: only upstream specialized geometry can be initialized/accessed.
mod sealed {
    pub trait Geometry {}
    impl Geometry for super::IVector {}
    impl Geometry for super::Vector {}
    impl Geometry for super::SkIPoint {}
    impl Geometry for super::SkPoint {}
    impl Geometry for super::SkISize {}
    impl Geometry for super::SkSize {}
    impl Geometry for super::SkIRect {}
    impl Geometry for super::SkRect {}
    impl Geometry for crate::src::core::SkMatrix::SkMatrix {}
}
impl<T: sealed::Geometry> LayerSpace<T> {
    // Source: src/core/SkImageFilterTypes.h:158-158
    // Source: src/core/SkImageFilterTypes.h:198-198
    // Source: src/core/SkImageFilterTypes.h:254-254
    // Source: src/core/SkImageFilterTypes.h:293-293
    // Source: src/core/SkImageFilterTypes.h:334-334
    // Source: src/core/SkImageFilterTypes.h:352-352
    // Source: src/core/SkImageFilterTypes.h:375-375
    // Source: src/core/SkImageFilterTypes.h:437-437
    // Source: src/core/SkImageFilterTypes.h:494-494
    // Adapted: one sealed generic overload expresses the native per-geometry
    // copying constructors without introducing other geometry specializations.
    pub fn from_ref(data: &T) -> Self
    where
        T: Clone,
    {
        Self {
            fData: data.clone(),
        }
    }
    pub fn new(data: T) -> Self {
        Self { fData: data }
    }
    pub fn geometry(&self) -> &T {
        &self.fData
    }
}

// Source: src/core/SkImageFilterTypes.h:154-191
impl LayerSpace<IVector> {
    // Source: src/core/SkImageFilterTypes.h:157-157
    // Adapted: native POD default construction leaves every coordinate
    // indeterminate. Do not create an initialized Rust value or read those bytes.
    pub const fn uninitialized() -> core::mem::MaybeUninit<Self> {
        core::mem::MaybeUninit::uninit()
    }
    pub fn x(&self) -> i32 {
        self.fData.fX
    }
    pub fn y(&self) -> i32 {
        self.fData.fY
    }
    pub fn as_vector(&self) -> SkIPoint {
        SkIPoint::make(self.x(), self.y())
    }
}
impl AddAssign for LayerSpace<IVector> {
    fn add_assign(&mut self, v: Self) {
        self.fData.fX = self.fData.fX.wrapping_add(v.fData.fX);
        self.fData.fY = self.fData.fY.wrapping_add(v.fData.fY);
    }
}
impl SubAssign for LayerSpace<IVector> {
    fn sub_assign(&mut self, v: Self) {
        self.fData.fX = self.fData.fX.wrapping_sub(v.fData.fX);
        self.fData.fY = self.fData.fY.wrapping_sub(v.fData.fY);
    }
}
impl Add for LayerSpace<IVector> {
    type Output = Self;
    fn add(mut self, v: Self) -> Self {
        self += v;
        self
    }
}
impl Sub for LayerSpace<IVector> {
    type Output = Self;
    fn sub(mut self, v: Self) -> Self {
        self -= v;
        self
    }
}
impl Neg for LayerSpace<IVector> {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(IVector::new(
            self.x().wrapping_neg(),
            self.y().wrapping_neg(),
        ))
    }
}

// Source: src/core/SkImageFilterTypes.h:194-247
impl LayerSpace<Vector> {
    // Source: src/core/SkImageFilterTypes.h:197-197
    // Adapted: native POD default construction leaves every coordinate
    // indeterminate. Do not create an initialized Rust value or read those bytes.
    pub const fn uninitialized() -> core::mem::MaybeUninit<Self> {
        core::mem::MaybeUninit::uninit()
    }
    pub fn x(&self) -> f32 {
        self.fData.fX
    }
    pub fn y(&self) -> f32 {
        self.fData.fY
    }
    pub fn as_vector(&self) -> SkPoint {
        SkPoint::make(self.x(), self.y())
    }
}
impl AddAssign for LayerSpace<Vector> {
    fn add_assign(&mut self, v: Self) {
        self.fData.fX = self.fData.fX + v.fData.fX;
        self.fData.fY = self.fData.fY + v.fData.fY;
    }
}
impl SubAssign for LayerSpace<Vector> {
    fn sub_assign(&mut self, v: Self) {
        self.fData.fX = self.fData.fX - v.fData.fX;
        self.fData.fY = self.fData.fY - v.fData.fY;
    }
}
impl Add for LayerSpace<Vector> {
    type Output = Self;
    fn add(mut self, v: Self) -> Self {
        self += v;
        self
    }
}
impl Sub for LayerSpace<Vector> {
    type Output = Self;
    fn sub(mut self, v: Self) -> Self {
        self -= v;
        self
    }
}
impl Neg for LayerSpace<Vector> {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(Vector::new(-self.x(), -self.y()))
    }
}
impl LayerSpace<Vector> {
    pub fn length(&self) -> f32 {
        SkPoint::length_xy(self.x(), self.y())
    }
}
impl MulAssign<f32> for LayerSpace<Vector> {
    fn mul_assign(&mut self, s: f32) {
        self.fData.fX *= s;
        self.fData.fY *= s;
    }
}
impl Mul<f32> for LayerSpace<Vector> {
    type Output = Self;
    fn mul(mut self, s: f32) -> Self {
        self *= s;
        self
    }
}
impl Mul<LayerSpace<Vector>> for f32 {
    type Output = LayerSpace<Vector>;
    fn mul(self, v: LayerSpace<Vector>) -> Self::Output {
        v * self
    }
}

// Source: src/core/SkImageFilterTypes.h:250-286
impl LayerSpace<SkIPoint> {
    // Source: src/core/SkImageFilterTypes.h:253-253
    // Adapted: native POD default construction leaves every coordinate
    // indeterminate. Do not create an initialized Rust value or read those bytes.
    pub const fn uninitialized() -> core::mem::MaybeUninit<Self> {
        core::mem::MaybeUninit::uninit()
    }
    pub fn x(&self) -> i32 {
        self.fData.fX
    }
    pub fn y(&self) -> i32 {
        self.fData.fY
    }
}
impl AddAssign<LayerSpace<IVector>> for LayerSpace<SkIPoint> {
    fn add_assign(&mut self, v: LayerSpace<IVector>) {
        self.fData += v.as_vector();
    }
}
impl SubAssign<LayerSpace<IVector>> for LayerSpace<SkIPoint> {
    fn sub_assign(&mut self, v: LayerSpace<IVector>) {
        self.fData -= v.as_vector();
    }
}
impl Add<LayerSpace<IVector>> for LayerSpace<SkIPoint> {
    type Output = Self;
    fn add(mut self, v: LayerSpace<IVector>) -> Self {
        self += v;
        self
    }
}
impl Sub<LayerSpace<IVector>> for LayerSpace<SkIPoint> {
    type Output = Self;
    fn sub(mut self, v: LayerSpace<IVector>) -> Self {
        self -= v;
        self
    }
}
impl Sub for LayerSpace<SkIPoint> {
    type Output = LayerSpace<IVector>;
    fn sub(self, p: Self) -> Self::Output {
        let v = self.fData - p.fData;
        LayerSpace::new(IVector::from_vector(v))
    }
}
impl Neg for LayerSpace<SkIPoint> {
    type Output = LayerSpace<IVector>;
    fn neg(self) -> Self::Output {
        LayerSpace::new(IVector::new(
            self.x().wrapping_neg(),
            self.y().wrapping_neg(),
        ))
    }
}

// Source: src/core/SkImageFilterTypes.h:289-327
impl LayerSpace<SkPoint> {
    // Source: src/core/SkImageFilterTypes.h:292-292
    // Adapted: native POD default construction leaves every coordinate
    // indeterminate. Do not create an initialized Rust value or read those bytes.
    pub const fn uninitialized() -> core::mem::MaybeUninit<Self> {
        core::mem::MaybeUninit::uninit()
    }
    pub fn x(&self) -> f32 {
        self.fData.fX
    }
    pub fn y(&self) -> f32 {
        self.fData.fY
    }
}
impl AddAssign<LayerSpace<Vector>> for LayerSpace<SkPoint> {
    fn add_assign(&mut self, v: LayerSpace<Vector>) {
        self.fData += v.as_vector();
    }
}
impl SubAssign<LayerSpace<Vector>> for LayerSpace<SkPoint> {
    fn sub_assign(&mut self, v: LayerSpace<Vector>) {
        self.fData -= v.as_vector();
    }
}
impl Add<LayerSpace<Vector>> for LayerSpace<SkPoint> {
    type Output = Self;
    fn add(mut self, v: LayerSpace<Vector>) -> Self {
        self += v;
        self
    }
}
impl Sub<LayerSpace<Vector>> for LayerSpace<SkPoint> {
    type Output = Self;
    fn sub(mut self, v: LayerSpace<Vector>) -> Self {
        self -= v;
        self
    }
}
impl Sub for LayerSpace<SkPoint> {
    type Output = LayerSpace<Vector>;
    fn sub(self, p: Self) -> Self::Output {
        let v = self.fData - p.fData;
        LayerSpace::new(Vector::from_vector(v))
    }
}
impl Neg for LayerSpace<SkPoint> {
    type Output = LayerSpace<Vector>;
    fn neg(self) -> Self::Output {
        LayerSpace::new(Vector::new(-self.x(), -self.y()))
    }
}
impl LayerSpace<SkPoint> {
    pub fn distanceToOrigin(&self) -> f32 {
        self.fData.distance_to_origin()
    }
}

// Source: src/core/SkImageFilterTypes.h:330-345
impl LayerSpace<SkISize> {
    // Source: src/core/SkImageFilterTypes.h:333-333
    // Adapted: native POD default construction leaves every coordinate
    // indeterminate. Do not create an initialized Rust value or read those bytes.
    pub const fn uninitialized() -> core::mem::MaybeUninit<Self> {
        core::mem::MaybeUninit::uninit()
    }
    pub fn width(&self) -> i32 {
        self.fData.width()
    }
    pub fn height(&self) -> i32 {
        self.fData.height()
    }
    pub fn isEmpty(&self) -> bool {
        self.fData.isEmpty()
    }
}

// Source: src/core/SkImageFilterTypes.h:348-368
impl LayerSpace<SkSize> {
    // Source: src/core/SkImageFilterTypes.h:351-351
    // Adapted: native POD default construction leaves every coordinate
    // indeterminate. Do not create an initialized Rust value or read those bytes.
    pub const fn uninitialized() -> core::mem::MaybeUninit<Self> {
        core::mem::MaybeUninit::uninit()
    }
    pub fn width(&self) -> f32 {
        self.fData.width()
    }
    pub fn height(&self) -> f32 {
        self.fData.height()
    }
    pub fn isEmpty(&self) -> bool {
        self.fData.isEmpty()
    }
}
impl LayerSpace<SkSize> {
    pub fn isZero(&self) -> bool {
        self.fData.isZero()
    }
}

// Source: src/core/SkImageFilterTypes.h:371-430
impl LayerSpace<SkIRect> {
    // Source: src/core/SkImageFilterTypes.h:374-374
    // Adapted: native POD default construction leaves every coordinate
    // indeterminate. Do not create an initialized Rust value or read those bytes.
    pub const fn uninitialized() -> core::mem::MaybeUninit<Self> {
        core::mem::MaybeUninit::uninit()
    }
    pub fn Empty() -> Self {
        Self::new(SkIRect::make_empty())
    }
    pub fn isEmpty(&self) -> bool {
        self.fData.is_empty64()
    }
    pub fn contains(&self, r: &Self) -> bool {
        self.fData.contains_rect(&r.fData)
    }
    pub fn left(&self) -> i32 {
        self.fData.fLeft
    }
    pub fn top(&self) -> i32 {
        self.fData.fTop
    }
    pub fn right(&self) -> i32 {
        self.fData.fRight
    }
    pub fn bottom(&self) -> i32 {
        self.fData.fBottom
    }
    pub fn width(&self) -> i32 {
        self.fData.width()
    }
    pub fn height(&self) -> i32 {
        self.fData.height()
    }
    pub fn topLeft(&self) -> LayerSpace<SkIPoint> {
        LayerSpace::new(SkIPoint::make(self.left(), self.top()))
    }
    pub fn size(&self) -> LayerSpace<SkISize> {
        LayerSpace::new(SkISize::Make(self.width(), self.height()))
    }
    pub fn intersect(&mut self, r: &Self) -> bool {
        self.fData.intersect(&r.fData)
    }
    pub fn join(&mut self, r: &Self) {
        self.fData.join(&r.fData)
    }
    pub fn outset(&mut self, d: LayerSpace<SkISize>) {
        self.fData.outset(d.width(), d.height());
    }
    pub fn inset(&mut self, d: LayerSpace<SkISize>) {
        self.fData.inset(d.width(), d.height());
    }
    pub fn offset(&mut self, v: LayerSpace<IVector>) {
        self.fData.offset(v.x(), v.y());
    }
    pub fn from_size(size: SkISize) -> Self {
        Self::new(SkIRect::make_size(size))
    }
    pub fn Unbounded() -> Option<Self> {
        None
    }
    pub fn Union<F: FnMut(i32) -> Self>(count: i32, mut bounds: F) -> Self {
        if count <= 0 {
            return Self::Empty();
        }
        let mut out = bounds(0);
        for i in 1..count {
            out.join(&bounds(i));
        }
        out
    }
    pub fn Intersects(a: &Self, b: &Self) -> bool {
        SkIRect::intersects(&a.fData, &b.fData)
    }
}

// Source: src/core/SkImageFilterTypes.h:433-483
impl LayerSpace<SkRect> {
    // Source: src/core/SkImageFilterTypes.h:436-436
    // Adapted: native POD default construction leaves every coordinate
    // indeterminate. Do not create an initialized Rust value or read those bytes.
    pub const fn uninitialized() -> core::mem::MaybeUninit<Self> {
        core::mem::MaybeUninit::uninit()
    }
    pub fn Empty() -> Self {
        Self::new(SkRect::MakeEmpty())
    }
    pub fn isEmpty(&self) -> bool {
        self.fData.isEmpty()
    }
    pub fn contains(&self, r: &Self) -> bool {
        self.fData.contains(&r.fData)
    }
    pub fn left(&self) -> f32 {
        self.fData.fLeft
    }
    pub fn top(&self) -> f32 {
        self.fData.fTop
    }
    pub fn right(&self) -> f32 {
        self.fData.fRight
    }
    pub fn bottom(&self) -> f32 {
        self.fData.fBottom
    }
    pub fn width(&self) -> f32 {
        self.fData.width()
    }
    pub fn height(&self) -> f32 {
        self.fData.height()
    }
    pub fn topLeft(&self) -> LayerSpace<SkPoint> {
        LayerSpace::new(SkPoint::make(self.left(), self.top()))
    }
    pub fn size(&self) -> LayerSpace<SkSize> {
        LayerSpace::new(SkSize::Make(self.width(), self.height()))
    }
    pub fn intersect(&mut self, r: &Self) -> bool {
        self.fData.intersect(&r.fData)
    }
    pub fn join(&mut self, r: &Self) {
        self.fData.join(&r.fData)
    }
    pub fn outset(&mut self, d: LayerSpace<SkSize>) {
        self.fData.outset(d.width(), d.height());
    }
    pub fn inset(&mut self, d: LayerSpace<SkSize>) {
        self.fData.inset(d.width(), d.height());
    }
    pub fn offset(&mut self, v: LayerSpace<Vector>) {
        self.fData.offset(v.x(), v.y());
    }
    pub fn from_irect(r: LayerSpace<SkIRect>) -> Self {
        Self::new(SkRect::Make(&r.fData))
    }
    pub fn clamp(&self, pt: LayerSpace<SkPoint>) -> LayerSpace<SkPoint> {
        use crate::include::private::SkTPin::SkTPin;
        LayerSpace::new(SkPoint::make(
            SkTPin(pt.x(), self.left(), self.right()),
            SkTPin(pt.y(), self.top(), self.bottom()),
        ))
    }
    pub fn center(&self) -> LayerSpace<SkPoint> {
        LayerSpace::new(self.fData.center())
    }
    pub fn round(&self) -> LayerSpace<SkIRect> {
        LayerSpace::new(self.fData.round())
    }
    pub fn roundIn(&self) -> LayerSpace<SkIRect> {
        LayerSpace::new(RoundIn(self.fData))
    }
    pub fn roundOut(&self) -> LayerSpace<SkIRect> {
        LayerSpace::new(RoundOut(self.fData))
    }
}

// Source: src/core/SkImageFilterTypes.h:490-496
// Matrix is the ninth specialization, in addition to eight geometric wrappers.
// Source: src/core/SkImageFilterTypes.h:516-534
// Source: src/core/SkImageFilterTypes.h:541-546
impl LayerSpace<SkMatrix> {
    pub fn preConcat(&mut self, m: &Self) -> &mut Self {
        let a = self.fData.clone();
        self.fData.setConcat(&a, &m.fData);
        self
    }
    pub fn postConcat(&mut self, m: &Self) -> &mut Self {
        let b = self.fData.clone();
        self.fData.setConcat(&m.fData, &b);
        self
    }
    pub fn invert(&self, inverse: Option<&mut Self>) -> bool {
        if let Some(inv) = self.fData.invert() {
            if let Some(out) = inverse {
                out.fData = inv;
            }
            true
        } else {
            false
        }
    }
    pub fn rc(&self, row: usize, col: usize) -> f32 {
        self.fData.rc(row, col)
    }
    pub fn get(&self, i: usize) -> f32 {
        self.fData.get(i)
    }
}

// Source: src/core/SkImageFilterTypes.h:493-493
// Only this specialization is initialized by native default construction.
// POD point/size/vector/rect default constructors remain uninitialized/pending.
impl Default for LayerSpace<SkMatrix> {
    fn default() -> Self {
        Self {
            fData: SkMatrix::default(),
        }
    }
}

// Source: src/core/SkImageFilterTypes.h:552-556
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatrixCapability {
    kTranslate,
    kScaleTranslate,
    kComplex,
}
// Source: src/core/SkImageFilterTypes.h:564-579
// Source: src/core/SkImageFilterTypes.h:651-656
#[derive(Clone, Copy, Debug, Default)]
pub struct Mapping {
    fLayerToDevMatrix: crate::src::core::SkM44::SkM44,
    fParamToLayerMatrix: crate::src::core::SkM44::SkM44,
    fDevToLayerMatrix: crate::src::core::SkM44::SkM44,
}
impl Mapping {
    pub fn new(param: crate::src::core::SkM44::SkM44) -> Self {
        Self {
            fParamToLayerMatrix: param,
            ..Self::default()
        }
    }
    pub fn from_decomposition(
        layer: crate::src::core::SkM44::SkM44,
        inverse: crate::src::core::SkM44::SkM44,
        param: crate::src::core::SkM44::SkM44,
    ) -> Self {
        Self {
            fLayerToDevMatrix: layer,
            fDevToLayerMatrix: inverse,
            fParamToLayerMatrix: param,
        }
    }
}
// Source: src/core/SkImageFilterTypes.h:595-595
impl Mapping {
    pub fn concatLocal(&mut self, local: &SkMatrix) {
        self.fParamToLayerMatrix.preConcat33(local);
    }
}
// Source: src/core/SkImageFilterTypes.h:607-622
impl Mapping {
    pub fn applyOrigin(&mut self, origin: LayerSpace<SkIPoint>) {
        let m = crate::src::core::SkM44::SkM44::Translate(
            origin.x().wrapping_neg() as f32,
            origin.y().wrapping_neg() as f32,
            0.0,
        );
        let okay = self.adjustLayerSpace(&m);
        crate::SkAssertResult!(@source b"src/core/SkImageFilterTypes.h\0".as_ptr(),608; @condition b"this->adjustLayerSpace(SkM44::Translate(-origin.x(), -origin.y()))\0".as_ptr(); okay);
    }
    pub fn layerToDeviceMatrix(&self) -> &crate::src::core::SkM44::SkM44 {
        &self.fLayerToDevMatrix
    }
    pub fn deviceToLayerMatrix(&self) -> &crate::src::core::SkM44::SkM44 {
        &self.fDevToLayerMatrix
    }
    pub fn layerMatrix(&self) -> &crate::src::core::SkM44::SkM44 {
        &self.fParamToLayerMatrix
    }
    pub fn totalMatrix(&self) -> crate::src::core::SkM44::SkM44 {
        self.fLayerToDevMatrix * self.fParamToLayerMatrix
    }
    pub fn paramToLayer<T: MapGeometry>(&self, param: &ParameterSpace<T>) -> LayerSpace<T> {
        LayerSpace::new(param.fData.map_geometry(&self.fParamToLayerMatrix.asM33()))
    }
}
// Source: src/core/SkImageFilterTypes.h:624-641
impl Mapping {
    pub fn deviceToLayer<T: EmptyGeometry>(&self, dev: &DeviceSpace<T>) -> LayerSpace<T> {
        if let Some(inv) = self.fLayerToDevMatrix.asM33().invert() {
            LayerSpace::new(dev.fData.map_geometry(&inv))
        } else {
            LayerSpace::new(T::empty_geometry())
        }
    }
    pub fn layerToDevice<T: MapGeometry>(&self, layer: &LayerSpace<T>) -> DeviceSpace<T> {
        DeviceSpace::new(layer.fData.map_geometry(&self.fLayerToDevMatrix.asM33()))
    }
}
// Source: src/core/SkImageFilterTypes.h:661-662
// Rust sealed traits express C++ specialization and only instantiate Empty() for rectangle geometry.
trait MapGeometry: sealed::Geometry {
    fn map_geometry(&self, matrix: &SkMatrix) -> Self;
}
trait EmptyGeometry: MapGeometry {
    fn empty_geometry() -> Self;
}
impl EmptyGeometry for SkRect {
    fn empty_geometry() -> Self {
        Self::MakeEmpty()
    }
}
impl EmptyGeometry for SkIRect {
    fn empty_geometry() -> Self {
        Self::make_empty()
    }
}

// Source: src/core/SkImageFilterTypes.h:838-842
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PixelBoundary {
    kUnknown,
    kTransparent,
    kInitialized,
}
// Source: src/core/SkImageFilterTypes.h:986-1003
// Reverse declaration order releases color-filter ownership before source-image ownership, as C++ does.
pub struct FilterResult {
    fLayerBounds: LayerSpace<SkIRect>,
    fColorFilter: Option<std::sync::Arc<dyn crate::src::core::SkColorFilter::SkColorFilter>>,
    fTransform: LayerSpace<SkMatrix>,
    fTileMode: crate::include::core::SkTileMode::SkTileMode,
    fSamplingOptions: crate::include::core::SkSamplingOptions::SkSamplingOptions,
    fBoundary: PixelBoundary,
    fImage: Option<std::sync::Arc<dyn super::SkSpecialImage::SkSpecialImage>>,
}
// Native implicit copy retains image then color-filter; explicit clone preserves construction order.
impl Clone for FilterResult {
    fn clone(&self) -> Self {
        let image = self.fImage.clone();
        let boundary = self.fBoundary;
        let sampling = self.fSamplingOptions;
        let tile = self.fTileMode;
        let transform = self.fTransform.clone();
        let color = self.fColorFilter.clone();
        let bounds = self.fLayerBounds;
        Self {
            fImage: image,
            fBoundary: boundary,
            fSamplingOptions: sampling,
            fTileMode: tile,
            fTransform: transform,
            fColorFilter: color,
            fLayerBounds: bounds,
        }
    }
    // Native implicit copy assignment follows declaration order, retaining then releasing each
    // owning field before advancing to the next. It differs from cloning the entire value first.
    fn clone_from(&mut self, source: &Self) {
        let image = source.fImage.clone();
        drop(core::mem::replace(&mut self.fImage, image));
        self.fBoundary = source.fBoundary;
        self.fSamplingOptions = source.fSamplingOptions;
        self.fTileMode = source.fTileMode;
        self.fTransform.clone_from(&source.fTransform);
        let color = source.fColorFilter.clone();
        drop(core::mem::replace(&mut self.fColorFilter, color));
        self.fLayerBounds = source.fLayerBounds;
    }
}
// Source: src/core/SkImageFilterTypes.h:716-722
impl Default for FilterResult {
    fn default() -> Self {
        Self::from_image(None)
    }
}
impl FilterResult {
    pub fn from_image(
        image: Option<std::sync::Arc<dyn super::SkSpecialImage::SkSpecialImage>>,
    ) -> Self {
        Self::new(image, LayerSpace::new(SkIPoint::make(0, 0)))
    }
    pub fn new(
        image: Option<std::sync::Arc<dyn super::SkSpecialImage::SkSpecialImage>>,
        origin: LayerSpace<SkIPoint>,
    ) -> Self {
        Self::with_boundary(image, origin, PixelBoundary::kUnknown)
    }
}
// Source: src/core/SkImageFilterTypes.h:751-766
impl FilterResult {
    pub const kDefaultSampling: crate::include::core::SkSamplingOptions::SkSamplingOptions =
        crate::include::core::SkSamplingOptions::SkSamplingOptions::with_filter(
            crate::include::core::SkSamplingOptions::SkFilterMode::kLinear,
        );
    pub fn has_image(&self) -> bool {
        self.fImage.is_some()
    }
    pub fn image(&self) -> Option<&dyn super::SkSpecialImage::SkSpecialImage> {
        self.fImage.as_deref()
    }
    pub fn refImage(&self) -> Option<std::sync::Arc<dyn super::SkSpecialImage::SkSpecialImage>> {
        self.fImage.clone()
    }
    pub fn layerBounds(&self) -> LayerSpace<SkIRect> {
        self.fLayerBounds
    }
    pub fn tileMode(&self) -> crate::include::core::SkTileMode::SkTileMode {
        self.fTileMode
    }
    pub fn sampling(&self) -> crate::include::core::SkSamplingOptions::SkSamplingOptions {
        self.fSamplingOptions
    }
    pub fn colorFilter(&self) -> Option<&dyn crate::src::core::SkColorFilter::SkColorFilter> {
        self.fColorFilter.as_deref()
    }
}
// Source: src/core/SkImageFilterTypes.h:844-855
impl FilterResult {
    fn with_boundary(
        image: Option<std::sync::Arc<dyn super::SkSpecialImage::SkSpecialImage>>,
        origin: LayerSpace<SkIPoint>,
        boundary: PixelBoundary,
    ) -> Self {
        let transform = LayerSpace::new(SkMatrix::Translate(origin.x() as f32, origin.y() as f32));
        let size = image
            .as_ref()
            .map_or(SkISize::Make(0, 0), |i| i.dimensions());
        let bounds = transform.mapIRect(&LayerSpace::from_size(size));
        Self {
            fImage: image,
            fBoundary: boundary,
            fSamplingOptions: Self::kDefaultSampling,
            fTileMode: crate::include::core::SkTileMode::SkTileMode::kDecal,
            fTransform: transform,
            fColorFilter: None,
            fLayerBounds: bounds,
        }
    }
}

// Source: src/core/SkImageFilterTypes.h:1140-1145
// Shared identity/lifetime belongs to Context's future Stats borrow, not a copied context counter set.
#[derive(Clone, Copy, Debug, Default)]
pub struct Stats {
    pub fNumVisitedImageFilters: i32,
    pub fNumCacheHits: i32,
    pub fNumOffscreenSurfaces: i32,
    pub fNumShaderClampedDraws: i32,
    pub fNumShaderBasedTilingDraws: i32,
}

// Source: src/core/SkImageFilterTypes.cpp:54-54
const kRoundEpsilon: f32 = 1e-3;
// Source: src/core/SkImageFilterTypes.cpp:56-80
fn are_axes_nearly_integer_aligned(
    m: &LayerSpace<SkMatrix>,
    out: Option<&mut LayerSpace<SkIPoint>>,
) -> (bool, bool) {
    use crate::include::core::SkScalar::{SkScalarNearlyEqual, SkScalarRoundToScalar};
    let invW = 1.0 / m.rc(2, 2);
    let tx = SkScalarRoundToScalar(m.rc(0, 2) * invW);
    let ty = SkScalarRoundToScalar(m.rc(1, 2) * invW);
    let affine = SkScalarNearlyEqual(m.rc(2, 0) * invW, 0.0, kRoundEpsilon)
        && SkScalarNearlyEqual(m.rc(2, 1) * invW, 0.0, kRoundEpsilon);
    if !affine {
        return (false, false);
    }
    let xAxis = SkScalarNearlyEqual(1.0, m.rc(0, 0) * invW, kRoundEpsilon)
        && SkScalarNearlyEqual(0.0, m.rc(0, 1) * invW, kRoundEpsilon)
        && SkScalarNearlyEqual(tx, m.rc(0, 2) * invW, kRoundEpsilon);
    let yAxis = SkScalarNearlyEqual(0.0, m.rc(1, 0) * invW, kRoundEpsilon)
        && SkScalarNearlyEqual(1.0, m.rc(1, 1) * invW, kRoundEpsilon)
        && SkScalarNearlyEqual(ty, m.rc(1, 2) * invW, kRoundEpsilon);
    if xAxis && yAxis {
        if let Some(out) = out {
            *out = LayerSpace::new(SkIPoint::make(tx as i32, ty as i32));
        }
    }
    (xAxis, yAxis)
}
// Source: src/core/SkImageFilterTypes.cpp:87-91
fn is_nearly_integer_translation(
    m: &LayerSpace<SkMatrix>,
    out: Option<&mut LayerSpace<SkIPoint>>,
) -> bool {
    let (x, y) = are_axes_nearly_integer_aligned(m, out);
    x && y
}
// Source: src/core/SkImageFilterTypes.cpp:118-181
fn periodic_axis_transform(
    tileMode: crate::include::core::SkTileMode::SkTileMode,
    crop: &LayerSpace<SkIRect>,
    output: &LayerSpace<SkIRect>,
) -> Option<LayerSpace<SkMatrix>> {
    use crate::include::core::SkScalar::SK_ScalarNearlyZero;
    use crate::include::core::SkTileMode::SkTileMode;
    use crate::src::core::SkFloatingPoint::sk_double_saturate2int;
    if tileMode == SkTileMode::kClamp || tileMode == SkTileMode::kDecal {
        return None;
    }
    let cropL = crop.left() as f64;
    let cropT = crop.top() as f64;
    let cropWidth = crop.right() as f64 - cropL;
    let cropHeight = crop.bottom() as f64 - cropT;
    let periodL = ((output.left() as f64 - cropL) / cropWidth).floor();
    let periodT = ((output.top() as f64 - cropT) / cropHeight).floor();
    let periodR = ((output.right() as f64 - cropL) / cropWidth).ceil();
    let periodB = ((output.bottom() as f64 - cropT) / cropHeight).ceil();
    if periodR - periodL <= 1.0 && periodB - periodT <= 1.0 {
        let mut sx = 1.0;
        let mut sy = 1.0;
        let mut tx = -cropL;
        let mut ty = -cropT;
        if tileMode == SkTileMode::kMirror {
            if periodL % 2.0 > SK_ScalarNearlyZero as f64 {
                sx = -1.0;
                tx = cropWidth - tx;
            }
            if periodT % 2.0 > SK_ScalarNearlyZero as f64 {
                sy = -1.0;
                ty = cropHeight - ty;
            }
        }
        tx += periodL * cropWidth + cropL;
        ty += periodT * cropHeight + cropT;
        if sk_double_saturate2int(tx) as f32 != tx as f32
            || sk_double_saturate2int(ty) as f32 != ty as f32
        {
            return None;
        }
        let mut matrix = SkMatrix::default();
        matrix.setScaleTranslate(sx, sy, tx as f32, ty as f32);
        Some(LayerSpace::new(matrix))
    } else {
        None
    }
}

// Source: src/core/SkImageFilterTypes.cpp:183-210
struct RasterBackend {
    base: BackendState,
}
impl Backend for RasterBackend {
    fn state(&self) -> &BackendState {
        &self.base
    }
    unsafe fn makeDevice(
        &self,
        size: SkISize,
        colorSpace: Option<std::sync::Arc<crate::src::core::SkColorSpace::SkColorSpace>>,
        props: Option<&crate::include::core::SkSurfaceProps::SkSurfaceProps>,
    ) -> crate::include::core::SkRefCnt::sk_sp<dyn crate::src::core::SkDevice::SkDeviceVirtual>
    {
        let info = crate::src::core::SkImageInfo::SkImageInfo::MakeDimensionsWithColorSpace(
            size,
            self.colorType(),
            crate::include::core::SkAlphaType::SkAlphaType::kPremul_SkAlphaType,
            colorSpace,
        );
        let mut device = crate::src::core::SkBitmapDevice::SkBitmapDevice::Create(
            &info,
            props.unwrap_or_else(|| self.surfaceProps()),
            None,
        );
        match device.release() {
            Some(ptr) => crate::include::core::SkRefCnt::sk_sp::from_raw(
                ptr.as_ptr() as *mut dyn crate::src::core::SkDevice::SkDeviceVirtual
            ),
            None => crate::include::core::SkRefCnt::sk_sp::default(),
        }
    }
    fn makeImage(
        &self,
        subset: &SkIRect,
        image: Option<std::sync::Arc<dyn crate::src::image::SkImage::SkImage>>,
    ) -> Option<std::sync::Arc<dyn super::SkSpecialImage::SkSpecialImage>> {
        super::SkSpecialImage::MakeFromRasterImage(*subset, image, *self.surfaceProps())
    }
    unsafe fn getCachedBitmap(
        &self,
        data: &crate::src::core::SkBitmap::SkBitmap,
    ) -> Option<std::sync::Arc<dyn crate::src::image::SkImage::SkImage>> {
        crate::src::image::SkImage_RasterFactories::RasterFromBitmap(data)
    }
    fn getBlurEngine(&self) -> &dyn super::SkBlurEngine::SkBlurEngine {
        super::SkBlurEngine::GetRasterBlurEngine()
    }
}
// Source: src/core/SkImageFilterTypes.cpp:225-227
// Source: src/core/SkImageFilterTypes.h:1137-1137
pub fn MakeRasterBackend(
    surfaceProps: &crate::include::core::SkSurfaceProps::SkSurfaceProps,
    colorType: crate::include::core::SkColorType::SkColorType,
) -> Result<std::sync::Arc<dyn Backend>, crate::RasterError> {
    Ok(std::sync::Arc::new(RasterBackend {
        base: BackendState::new(
            super::SkImageFilterCache::SkImageFilterCache::Get(
                super::SkImageFilterCache::CreateIfNecessary::kYes,
            ),
            *surfaceProps,
            colorType,
        ),
    }))
}

impl Stats {
    // Source: src/core/SkImageFilterTypes.cpp:229-240
    // Source: src/core/SkImageFilterTypes.h:1147-1147
    pub fn dumpStats(&self) {
        unsafe {
            crate::SkDebugf!(@source b"src/core/SkImageFilterTypes.cpp\0".as_ptr(),230;
            concat!("ImageFilter Stats:\n",
            "      # visited filters: %d\n",
            "           # cache hits: %d\n",
            "   # offscreen surfaces: %d\n",
            " # shader-clamped draws: %d\n",
            "   # shader-tiled draws: %d\n\0").as_ptr(),
            self.fNumVisitedImageFilters as core::ffi::c_int,
            self.fNumCacheHits as core::ffi::c_int,
            self.fNumOffscreenSurfaces as core::ffi::c_int,
            self.fNumShaderClampedDraws as core::ffi::c_int,
            self.fNumShaderBasedTilingDraws as core::ffi::c_int);
        }
    }
    // Source: src/core/SkImageFilterTypes.cpp:243-251
    // Tracing adapter: the existing thread-local instant recorder supplies
    // TRACE_EVENT_SCOPE_THREAD and accepts lossless i32-to-f64 numeric fields.
    // Source: src/core/SkImageFilterTypes.h:1148-1148
    pub fn reportStats(&self) {
        browser_tracing::instant(
            "skia",
            "ImageFilter Graph Size",
            &[
                ("count", self.fNumVisitedImageFilters as f64),
                ("cache hits", self.fNumCacheHits as f64),
            ],
        );
        browser_tracing::instant(
            "skia",
            "ImageFilter Surfaces",
            &[("count", self.fNumOffscreenSurfaces as f64)],
        );
        browser_tracing::instant(
            "skia",
            "ImageFilter Shader Tiling",
            &[
                ("clamp", self.fNumShaderClampedDraws as f64),
                ("other", self.fNumShaderBasedTilingDraws as f64),
            ],
        );
    }
}

// Source: src/core/SkImageFilterTypes.cpp:255-255
// Source: src/core/SkImageFilterTypes.h:56-56
pub fn RoundOut(r: SkRect) -> SkIRect {
    r.makeInset(kRoundEpsilon, kRoundEpsilon).roundOut()
}
// Source: src/core/SkImageFilterTypes.h:57-57
// Source: src/core/SkImageFilterTypes.cpp:257-257
pub fn RoundIn(r: SkRect) -> SkIRect {
    r.makeOutset(kRoundEpsilon, kRoundEpsilon).roundIn()
}
// Source: src/core/SkImageFilterTypes.h:603-603
// Source: src/core/SkImageFilterTypes.cpp:309-318
impl Mapping {
    pub fn adjustLayerSpace(&mut self, layer: &crate::src::core::SkM44::SkM44) -> bool {
        let mut inverse = crate::src::core::SkM44::SkM44::default();
        if !layer.invert(&mut inverse) {
            return false;
        }
        self.fParamToLayerMatrix.postConcat(layer);
        self.fDevToLayerMatrix.postConcat(layer);
        self.fLayerToDevMatrix.preConcat(&inverse);
        true
    }
}
// Source: src/core/SkImageFilterTypes.cpp:321-324
impl MapGeometry for SkRect {
    fn map_geometry(&self, m: &SkMatrix) -> Self {
        if self.isEmpty() {
            Self::MakeEmpty()
        } else {
            let mut out = Self::MakeEmpty();
            m.mapRect(&mut out, self);
            out
        }
    }
}
// Source: src/core/SkImageFilterTypes.cpp:326-348
impl MapGeometry for SkIRect {
    fn map_geometry(&self, m: &SkMatrix) -> Self {
        use crate::src::core::SkFloatingPoint::sk_double_saturate2int;
        if self.is_empty() {
            return Self::make_empty();
        }
        if m.isScaleTranslate() {
            let l = m.getScaleX() as f64 * self.fLeft as f64 + m.getTranslateX() as f64;
            let r = m.getScaleX() as f64 * self.fRight as f64 + m.getTranslateX() as f64;
            let t = m.getScaleY() as f64 * self.fTop as f64 + m.getTranslateY() as f64;
            let b = m.getScaleY() as f64 * self.fBottom as f64 + m.getTranslateY() as f64;
            fn min(a: f64, b: f64) -> f64 {
                if b < a {
                    b
                } else {
                    a
                }
            }
            fn max(a: f64, b: f64) -> f64 {
                if a < b {
                    b
                } else {
                    a
                }
            }
            Self::make_ltrb(
                sk_double_saturate2int((min(l, r) + kRoundEpsilon as f64).floor()),
                sk_double_saturate2int((min(t, b) + kRoundEpsilon as f64).floor()),
                sk_double_saturate2int((max(l, r) - kRoundEpsilon as f64).ceil()),
                sk_double_saturate2int((max(t, b) - kRoundEpsilon as f64).ceil()),
            )
        } else {
            RoundOut(SkRect::Make(self).map_geometry(m))
        }
    }
}
// Source: src/core/SkImageFilterTypes.cpp:350-354
impl MapGeometry for SkIPoint {
    fn map_geometry(&self, m: &SkMatrix) -> Self {
        use crate::include::core::SkScalar::SkScalarRoundToInt;
        let p = m.mapPoint(SkPoint::make(self.fX as f32, self.fY as f32));
        Self::make(SkScalarRoundToInt(p.fX), SkScalarRoundToInt(p.fY))
    }
}
// Source: src/core/SkImageFilterTypes.cpp:356-359
impl MapGeometry for SkPoint {
    fn map_geometry(&self, m: &SkMatrix) -> Self {
        m.mapPoint(*self)
    }
}
// Source: src/core/SkImageFilterTypes.cpp:361-364
impl MapGeometry for Vector {
    fn map_geometry(&self, m: &SkMatrix) -> Self {
        let mut out = [SkPoint::make(0.0, 0.0)];
        m.mapVectors(&mut out, &[SkPoint::make(self.fX, self.fY)]);
        Self::from_vector(out[0])
    }
}
// Source: src/core/SkImageFilterTypes.cpp:366-370
impl MapGeometry for IVector {
    fn map_geometry(&self, m: &SkMatrix) -> Self {
        use crate::include::core::SkScalar::SkScalarRoundToInt;
        let v = Vector::new(self.fX as f32, self.fY as f32).map_geometry(m);
        Self::new(SkScalarRoundToInt(v.fX), SkScalarRoundToInt(v.fY))
    }
}
// Source: src/core/SkImageFilterTypes.cpp:375-386
impl MapGeometry for SkSize {
    fn map_geometry(&self, m: &SkMatrix) -> Self {
        if m.isScaleTranslate() {
            let s = Vector::new(self.fWidth, self.fHeight).map_geometry(m);
            return Self::Make(s.fX.abs(), s.fY.abs());
        }
        let x = Vector::new(self.fWidth, 0.0).map_geometry(m);
        let y = Vector::new(0.0, self.fHeight).map_geometry(m);
        Self::Make(
            SkPoint::length_xy(x.fX, x.fY),
            SkPoint::length_xy(y.fX, y.fY),
        )
    }
}
// Source: src/core/SkImageFilterTypes.cpp:388-393
impl MapGeometry for SkISize {
    fn map_geometry(&self, m: &SkMatrix) -> Self {
        let s = SkSize::FromISize(self).map_geometry(m);
        Self::Make(
            SkScalarCeilToInt(s.fWidth - kRoundEpsilon),
            SkScalarCeilToInt(s.fHeight - kRoundEpsilon),
        )
    }
}
// Source: src/core/SkImageFilterTypes.cpp:395-405
impl MapGeometry for SkMatrix {
    fn map_geometry(&self, m: &SkMatrix) -> Self {
        let mut inv = m.invert().unwrap_or_default();
        inv.postConcat(self);
        inv.postConcat(m);
        inv
    }
}

// Source: src/core/SkImageFilterTypes.cpp:410-429
impl LayerSpace<SkIRect> {
    pub fn relevantSubset(
        &self,
        dst: Self,
        tile: crate::include::core::SkTileMode::SkTileMode,
    ) -> Self {
        use crate::include::core::SkTileMode::SkTileMode;
        let mut fitted = *self;
        if tile == SkTileMode::kDecal || tile == SkTileMode::kClamp {
            if !fitted.intersect(&dst) {
                if tile == SkTileMode::kDecal {
                    fitted = Self::Empty();
                } else {
                    fitted = Self::new(
                        crate::src::core::SkRectPriv::SkRectPriv::ClosestDisjointEdge(
                            &fitted.fData,
                            &dst.fData,
                        ),
                    );
                }
            }
        }
        fitted
    }
}
// Source: src/core/SkImageFilterTypes.cpp:432-442
impl LayerSpace<SkSize> {
    pub fn round(&self) -> LayerSpace<SkISize> {
        LayerSpace::new(self.fData.toRound())
    }
    pub fn ceil(&self) -> LayerSpace<SkISize> {
        LayerSpace::new(SkISize::Make(
            SkScalarCeilToInt(self.width() - kRoundEpsilon),
            SkScalarCeilToInt(self.height() - kRoundEpsilon),
        ))
    }
    pub fn floor(&self) -> LayerSpace<SkISize> {
        LayerSpace::new(SkISize::Make(
            SkScalarFloorToInt(self.width() + kRoundEpsilon),
            SkScalarFloorToInt(self.height() + kRoundEpsilon),
        ))
    }
}

// Source: src/core/SkImageFilterTypes.h:504-514
// Source: src/core/SkImageFilterTypes.cpp:444-464
impl LayerSpace<SkMatrix> {
    pub fn mapRect(&self, r: &LayerSpace<SkRect>) -> LayerSpace<SkRect> {
        LayerSpace::new(r.fData.map_geometry(&self.fData))
    }
    pub fn mapIRect(&self, r: &LayerSpace<SkIRect>) -> LayerSpace<SkIRect> {
        LayerSpace::new(r.fData.map_geometry(&self.fData))
    }
    pub fn mapPoint(&self, p: &LayerSpace<SkPoint>) -> LayerSpace<SkPoint> {
        LayerSpace::new(p.fData.map_geometry(&self.fData))
    }
    pub fn mapVector(&self, v: &LayerSpace<Vector>) -> LayerSpace<Vector> {
        LayerSpace::new(v.fData.map_geometry(&self.fData))
    }
    pub fn mapSize(&self, s: &LayerSpace<SkSize>) -> LayerSpace<SkSize> {
        LayerSpace::new(s.fData.map_geometry(&self.fData))
    }
}

// Source: src/core/SkImageFilterTypes.h:538-539
// Source: src/core/SkImageFilterTypes.cpp:466-479
impl LayerSpace<SkMatrix> {
    pub fn inverseMapRect(&self, r: &LayerSpace<SkRect>, out: &mut LayerSpace<SkRect>) -> bool {
        let mut mapped = SkRect::MakeEmpty();
        if r.isEmpty() {
            *out = LayerSpace::<SkRect>::Empty();
            true
        } else if crate::src::core::SkMatrixPriv::SkMatrixPriv::InverseMapRect(
            &self.fData,
            &mut mapped,
            &r.fData,
        ) {
            *out = LayerSpace::new(mapped);
            true
        } else {
            false
        }
    }
}
// Source: src/core/SkImageFilterTypes.cpp:481-512
impl LayerSpace<SkMatrix> {
    pub fn inverseMapIRect(&self, r: &LayerSpace<SkIRect>, out: &mut LayerSpace<SkIRect>) -> bool {
        use crate::src::core::SkFloatingPoint::sk_double_saturate2int;
        if r.isEmpty() {
            *out = LayerSpace::<SkIRect>::Empty();
            true
        } else if self.fData.isScaleTranslate() {
            let m = &self.fData;
            if m.getScaleX() == 0.0 || m.getScaleY() == 0.0 {
                return false;
            }
            let l = (r.left() as f64 - m.getTranslateX() as f64) / m.getScaleX() as f64;
            let rr = (r.right() as f64 - m.getTranslateX() as f64) / m.getScaleX() as f64;
            let t = (r.top() as f64 - m.getTranslateY() as f64) / m.getScaleY() as f64;
            let b = (r.bottom() as f64 - m.getTranslateY() as f64) / m.getScaleY() as f64;
            fn min(a: f64, b: f64) -> f64 {
                if b < a {
                    b
                } else {
                    a
                }
            }
            fn max(a: f64, b: f64) -> f64 {
                if a < b {
                    b
                } else {
                    a
                }
            }
            let mapped = SkIRect::make_ltrb(
                sk_double_saturate2int((min(l, rr) + kRoundEpsilon as f64).floor()),
                sk_double_saturate2int((min(t, b) + kRoundEpsilon as f64).floor()),
                sk_double_saturate2int((max(l, rr) - kRoundEpsilon as f64).ceil()),
                sk_double_saturate2int((max(t, b) - kRoundEpsilon as f64).ceil()),
            );
            *out = LayerSpace::new(mapped);
            true
        } else {
            let mut mapped = SkRect::MakeEmpty();
            if crate::src::core::SkMatrixPriv::SkMatrixPriv::InverseMapRect(
                &self.fData,
                &mut mapped,
                &SkRect::Make(&r.fData),
            ) {
                *out = LayerSpace::new(mapped).roundOut();
                true
            } else {
                false
            }
        }
    }
}

impl FilterResult {
    // Source: src/core/SkImageFilterTypes.cpp:635-639
    /// Safety: preserve the same backing and backend leases as imageAndOffset.
    /// The output point is written only after successful resolution.
    // Source: src/core/SkImageFilterTypes.h:797-797
    pub unsafe fn imageAndOffset_into(
        &self,
        ctx: &Context<'_>,
        offset: &mut SkIPoint,
    ) -> Result<Option<std::sync::Arc<dyn super::SkSpecialImage::SkSpecialImage>>, crate::RasterError>
    {
        let (image, origin) = unsafe { self.imageAndOffset(ctx) }?;
        *offset = *origin.geometry();
        Ok(image)
    }
    // Source: src/core/SkImageFilterTypes.cpp:641-645
    /// Safety: retain Context's backend, source-image and destination draw leases.
    // Source: src/core/SkImageFilterTypes.h:803-803
    pub unsafe fn imageAndOffset(
        &self,
        ctx: &Context<'_>,
    ) -> Result<
        (
            Option<std::sync::Arc<dyn super::SkSpecialImage::SkSpecialImage>>,
            LayerSpace<SkIPoint>,
        ),
        crate::RasterError,
    > {
        let resolved = self.resolve(ctx, *ctx.desiredOutput(), false)?;
        let origin = resolved.layerBounds().topLeft();
        Ok((resolved.fImage, origin))
    }
}

// Source: src/core/SkImageFilterTypes.cpp:647-665
impl FilterResult {
    // Source: src/core/SkImageFilterTypes.h:814-814
    pub fn insetForSaveLayer(&self) -> Self {
        use crate::include::core::SkTileMode::SkTileMode;
        if self.fImage.is_none() {
            return Self::default();
        }
        crate::SkASSERT!(@source b"src/core/SkImageFilterTypes.cpp\0".as_ptr(),653; @condition b"fTileMode == SkTileMode::kDecal\0".as_ptr(); self.fTileMode == SkTileMode::kDecal);
        let mut inset = self.insetByPixel();
        crate::SkASSERT!(@source b"src/core/SkImageFilterTypes.cpp\0".as_ptr(),661; @condition b"inset.fBoundary == PixelBoundary::kInitialized && inset.fTileMode == SkTileMode::kDecal\0".as_ptr();
            inset.fBoundary == PixelBoundary::kInitialized && inset.fTileMode == SkTileMode::kDecal
        );
        inset.fBoundary = PixelBoundary::kTransparent;
        inset
    }
}
// Source: src/core/SkImageFilterTypes.cpp:667-675
impl FilterResult {
    // Source: src/core/SkImageFilterTypes.h:871-871
    fn insetByPixel(&self) -> Self {
        let mut inset = self.fLayerBounds;
        inset.inset(LayerSpace::new(SkISize::Make(1, 1)));
        crate::SkASSERT!(@source b"src/core/SkImageFilterTypes.cpp\0".as_ptr(),673; @condition b"!insetBounds.isEmpty()\0".as_ptr(); !inset.isEmpty());
        self.subset(self.fLayerBounds.topLeft(), inset, false)
    }
}
impl FilterResult {
    // Source: src/core/SkImageFilterTypes.cpp:1058-1122
    /// Safety: any required resolve obeys the actual offscreen image/draw leases.
    // Source: src/core/SkImageFilterTypes.h:779-781
    pub unsafe fn applyTransform(
        &self,
        ctx: &Context<'_>,
        transform: LayerSpace<SkMatrix>,
        sampling: crate::include::core::SkSamplingOptions::SkSamplingOptions,
    ) -> Result<Self, crate::RasterError> {
        if self.fImage.is_none() || ctx.desiredOutput().isEmpty() {
            crate::SkASSERT!(@source b"src/core/SkImageFilterTypes.cpp\0".as_ptr(),1063; @condition b"!fColorFilter\0".as_ptr(); self.fColorFilter.is_none());
            return Ok(Self::default());
        }
        if !transform.invert(None) {
            return Ok(Self::default());
        }
        let currentInteger = is_nearly_integer_translation(&self.fTransform, None);
        let nextInteger = is_nearly_integer_translation(&transform, None);
        crate::SkASSERT!(@source b"src/core/SkImageFilterTypes.cpp\0".as_ptr(),1078; @condition b"!currentXformIsInteger || fSamplingOptions == kDefaultSampling\0".as_ptr(); !currentInteger || self.fSamplingOptions==Self::kDefaultSampling);
        let mut nextSampling = if nextInteger {
            Self::kDefaultSampling
        } else {
            sampling
        };
        let isCropped = !nextInteger
            && self
                .analyzeBounds(
                    transform.geometry(),
                    ctx.desiredOutput().geometry(),
                    BoundsScope::kDeferred,
                )?
                .contains(BoundsAnalysis::kRequiresLayerCrop);
        let mut transformed = if !isCropped
            && compatible_sampling(
                &self.fSamplingOptions,
                currentInteger,
                &mut nextSampling,
                nextInteger,
            ) {
            self.clone()
        } else {
            let mut tight = LayerSpace::<SkIRect>::Empty();
            let resolved = if transform.inverseMapIRect(ctx.desiredOutput(), &mut tight) {
                self.resolve(ctx, tight, false)?
            } else {
                Self::default()
            };
            if resolved.fImage.is_none() {
                return Ok(Self::default());
            }
            resolved
        };
        transformed.fSamplingOptions = nextSampling;
        transformed.fTransform.postConcat(&transform);
        transformed.fLayerBounds = transform.mapIRect(&transformed.fLayerBounds);
        if !LayerSpace::<SkIRect>::Intersects(&transformed.fLayerBounds, ctx.desiredOutput()) {
            return Ok(Self::default());
        }
        Ok(transformed)
    }
    // Source: src/core/SkImageFilterTypes.cpp:1124-1155
    /// Safety: backend-created independent offscreen targets and all source
    /// image/shader pixels satisfy their synchronous draw and snapshot leases.
    // Source: src/core/SkImageFilterTypes.h:861-862
    unsafe fn resolve(
        &self,
        ctx: &Context<'_>,
        mut dstBounds: LayerSpace<SkIRect>,
        preserveDstBounds: bool,
    ) -> Result<Self, crate::RasterError> {
        if self.fImage.is_none() || (!preserveDstBounds && !dstBounds.intersect(&self.fLayerBounds))
        {
            return Ok(Self::default());
        }
        let compatible = self.fColorFilter.is_none()
            && self.fTileMode == crate::include::core::SkTileMode::SkTileMode::kDecal
            && !preserveDstBounds;
        let mut origin = LayerSpace::new(SkIPoint::make(0, 0));
        if compatible && is_nearly_integer_translation(&self.fTransform, Some(&mut origin)) {
            return Ok(self.subset(origin, dstBounds, false));
        }
        let props = crate::include::core::SkSurfaceProps::SkSurfaceProps::default();
        let boundary = if preserveDstBounds {
            PixelBoundary::kUnknown
        } else {
            PixelBoundary::kTransparent
        };
        let mut surface = AutoSurface::new(ctx, dstBounds, boundary, false, Some(&props))?;
        if surface.is_valid() {
            self.draw_internal(ctx, surface.device(), false, None)?;
        }
        surface.snap()
    }
}

// Source: src/core/SkImageFilterTypes.cpp:1157-1202
impl FilterResult {
    // Source: src/core/SkImageFilterTypes.h:867-869
    fn subset(
        &self,
        known: LayerSpace<SkIPoint>,
        bounds: LayerSpace<SkIRect>,
        clamp: bool,
    ) -> Self {
        use crate::include::core::SkTileMode::SkTileMode;
        #[cfg(sk_debug)]
        {
            let mut actual = LayerSpace::new(SkIPoint::make(0, 0));
            crate::SkASSERT!(@source b"src/core/SkImageFilterTypes.cpp\0".as_ptr(),1161; @condition b"is_nearly_integer_translation(fTransform, &actualOrigin) && SkIPoint(actualOrigin) == SkIPoint(knownOrigin)\0".as_ptr(); is_nearly_integer_translation(&self.fTransform,Some(&mut actual)) && actual==known);
        }
        let image = self.fImage.as_ref().expect("subset requires an image");
        let mut imageBounds = LayerSpace::new(SkIRect::make_xywh(
            known.x(),
            known.y(),
            image.width(),
            image.height(),
        ));
        imageBounds = imageBounds.relevantSubset(
            bounds,
            if clamp {
                SkTileMode::kClamp
            } else {
                SkTileMode::kDecal
            },
        );
        if imageBounds.isEmpty() {
            return Self::default();
        }
        // Preserve direct subtraction; negating INT_MIN origin before adding would overflow.
        let subset = SkIRect::make_ltrb(
            imageBounds.left().wrapping_sub(known.x()),
            imageBounds.top().wrapping_sub(known.y()),
            imageBounds.right().wrapping_sub(known.x()),
            imageBounds.bottom().wrapping_sub(known.y()),
        );
        crate::SkASSERT!(@source b"src/core/SkImageFilterTypes.cpp\0".as_ptr(),1180; @condition b"subset.fLeft >= 0 && subset.fTop >= 0 && subset.fRight <= fImage->width() && subset.fBottom <= fImage->height()\0".as_ptr(); subset.fLeft>=0&&subset.fTop>=0&&subset.fRight<=image.width()&&subset.fBottom<=image.height());
        let mut result = Self::new(image.makeSubset(&subset), imageBounds.topLeft());
        result.fColorFilter = self.fColorFilter.clone();
        crate::SkASSERT!(@source b"src/core/SkImageFilterTypes.cpp\0".as_ptr(),1187; @condition b"result.fBoundary == PixelBoundary::kUnknown\0".as_ptr(); result.fBoundary == PixelBoundary::kUnknown);
        let resultImage = result
            .fImage
            .as_ref()
            .expect("valid raster subset retains backing");
        if image.subset() == resultImage.subset() {
            result.fBoundary = self.fBoundary;
        } else {
            let mut safe = *image.subset();
            if self.fBoundary == PixelBoundary::kUnknown {
                safe.inset(1, 1);
            }
            if safe.contains_rect(resultImage.subset()) {
                result.fBoundary = PixelBoundary::kInitialized;
            }
        }
        result
    }
}

// Source: src/core/SkImageFilterTypes.cpp:93-116
fn decompose_transform(
    transform: &SkMatrix,
    point: SkPoint,
    mut post: Option<&mut SkMatrix>,
) -> SkMatrix {
    let mut scale = crate::include::core::SkSize::SkSize::MakeEmpty();
    if transform.decomposeScale(Some(&mut scale), post.as_deref_mut()) {
        SkMatrix::Scale(scale.fWidth, scale.fHeight)
    } else {
        let mut approx =
            crate::src::core::SkMatrixPriv::SkMatrixPriv::DifferentialAreaScale(transform, &point);
        if approx.is_finite()
            && !crate::include::core::SkScalar::SkScalarNearlyZero(
                approx,
                crate::include::core::SkScalar::SK_ScalarNearlyZero,
            )
        {
            approx = approx.sqrt();
        } else {
            approx = 1.0;
        }
        if let Some(post) = post {
            *post = transform.clone();
            let inverse = 1.0 / approx;
            post.preScale(inverse, inverse);
        }
        SkMatrix::Scale(approx, approx)
    }
}
impl Mapping {
    // Source: src/core/SkImageFilterTypes.h:587-589
    // Source: src/core/SkImageFilterTypes.cpp:259-298
    pub fn decomposeCTM(
        &mut self,
        ctm: &crate::src::core::SkM44::SkM44,
        capability: MatrixCapability,
        point: &ParameterSpace<SkPoint>,
    ) -> bool {
        use crate::src::core::{SkM44::SkM44, SkMatrixPriv::SkMatrixPriv};
        let (mut remainder, layer);
        if capability == MatrixCapability::kTranslate {
            remainder = *ctm;
            layer = SkM44::default();
        } else if SkMatrixPriv::IsScaleTranslateAsM33(ctm)
            || capability == MatrixCapability::kComplex
        {
            remainder = SkM44::default();
            layer = *ctm;
        } else {
            let layer33 = decompose_transform(&ctm.asM33(), point.fData, None);
            layer = SkM44::from(&layer33);
            remainder = *ctm;
            remainder.preScale(1.0 / layer.rc(0, 0), 1.0 / layer.rc(1, 1));
        }
        let mut inverse = SkM44::default();
        if !remainder.invert(&mut inverse) {
            false
        } else {
            self.fParamToLayerMatrix = layer;
            self.fLayerToDevMatrix = remainder;
            self.fDevToLayerMatrix = inverse;
            true
        }
    }
    // Source: src/core/SkImageFilterTypes.h:584-586
    // Source: src/core/SkImageFilterTypes.cpp:300-307
    pub fn decomposeCTMForFilter(
        &mut self,
        ctm: &crate::src::core::SkM44::SkM44,
        filter: Option<&dyn super::SkImageFilter::SkImageFilter>,
        point: &ParameterSpace<SkPoint>,
    ) -> bool {
        self.decomposeCTM(
            ctm,
            filter.map_or(MatrixCapability::kComplex, |f| f.getCTMCapability()),
            point,
        )
    }
}

// Source: src/core/SkImageFilterTypes.h:498-500
impl LayerSpace<SkMatrix> {
    pub fn RectToRect(from: &LayerSpace<SkRect>, to: &LayerSpace<SkRect>) -> Self {
        Self::new(SkMatrix::RectToRectOrIdentity(
            from.geometry(),
            to.geometry(),
            crate::src::core::SkMatrix::ScaleToFit::kFill_ScaleToFit,
        ))
    }
}

impl FilterResult {
    // Source: src/core/SkImageFilterTypes.cpp:836-931
    /// Safety: any required resolve obeys the actual offscreen image/draw leases.
    // Source: src/core/SkImageFilterTypes.h:772-774
    pub unsafe fn applyCrop(
        &self,
        ctx: &Context<'_>,
        crop: LayerSpace<SkIRect>,
        mut tileMode: crate::include::core::SkTileMode::SkTileMode,
    ) -> Result<Self, crate::RasterError> {
        use crate::include::core::SkTileMode::SkTileMode::*;
        if crop.isEmpty() || ctx.desiredOutput().isEmpty() {
            return Ok(Self::default());
        }
        let mut cropContent = crop;
        if self.fImage.is_none() || !cropContent.intersect(&self.fLayerBounds) {
            return Ok(Self::default());
        }
        let mut fittedCrop = crop.relevantSubset(*ctx.desiredOutput(), tileMode);
        if !cropContent.intersect(&fittedCrop) {
            return Ok(Self::default());
        }
        if let Some(transform) = periodic_axis_transform(tileMode, &fittedCrop, ctx.desiredOutput())
        {
            return self.applyTransform(ctx, transform, Self::kDefaultSampling);
        }
        let mut preserveTransparencyInCrop = false;
        if tileMode == kDecal {
            fittedCrop = cropContent;
        } else if fittedCrop.contains(ctx.desiredOutput()) {
            tileMode = kDecal;
            fittedCrop = *ctx.desiredOutput();
        } else if !cropContent.contains(&fittedCrop) {
            preserveTransparencyInCrop = true;
            if self.fTileMode == kDecal && tileMode == kClamp {
                cropContent.outset(LayerSpace::new(SkISize::Make(1, 1)));
                let valid = fittedCrop.intersect(&cropContent);
                crate::SkAssertResult!(@source b"src/core/SkImageFilterTypes.cpp\0".as_ptr(),887; @condition b"fittedCrop.intersect(cropContent)\0".as_ptr(); valid);
            }
        }
        let doubleClamp = self.fTileMode == kClamp && tileMode == kClamp;
        let mut origin = LayerSpace::new(SkIPoint::make(0, 0));
        if !preserveTransparencyInCrop
            && is_nearly_integer_translation(&self.fTransform, Some(&mut origin))
            && (doubleClamp
                || !self
                    .analyzeLayerBounds(&fittedCrop, BoundsScope::kDeferred)?
                    .contains(BoundsAnalysis::kHasLayerFillingEffect))
        {
            let mut restricted = self.subset(origin, fittedCrop, doubleClamp);
            restricted.updateTileMode(ctx, tileMode);
            if restricted.fBoundary == PixelBoundary::kInitialized || tileMode != kDecal {
                restricted.fBoundary = PixelBoundary::kUnknown;
            }
            Ok(restricted)
        } else if tileMode == kDecal {
            crate::SkASSERT!(@source b"src/core/SkImageFilterTypes.cpp\0".as_ptr(),920; @condition b"!preserveTransparencyInCrop\0".as_ptr(); !preserveTransparencyInCrop);
            let mut restricted = self.clone();
            restricted.fLayerBounds = fittedCrop;
            Ok(restricted)
        } else {
            let mut tiled = self.resolve(ctx, fittedCrop, true)?;
            tiled.updateTileMode(ctx, tileMode);
            Ok(tiled)
        }
    }
    // Source: src/core/SkImageFilterTypes.cpp:933-1002
    /// Safety: any required resolve obeys the actual offscreen image/draw leases.
    // Source: src/core/SkImageFilterTypes.h:786-787
    pub unsafe fn applyColorFilter(
        &self,
        ctx: &Context<'_>,
        colorFilter: Option<std::sync::Arc<dyn crate::src::core::SkColorFilter::SkColorFilter>>,
    ) -> Result<Self, crate::RasterError> {
        use crate::include::core::SkTileMode::SkTileMode::kClamp;
        let colorFilter = colorFilter.expect("native applyColorFilter requires nonnull filter");
        if ctx.desiredOutput().isEmpty() {
            return Ok(Self::default());
        }
        let mut newLayerBounds = self.fLayerBounds;
        if colorFilter.affectsTransparentBlack()? {
            if self.fImage.is_none() || !newLayerBounds.intersect(ctx.desiredOutput()) {
                let bounds = LayerSpace::new(SkIRect::make_xywh(
                    ctx.desiredOutput().left(),
                    ctx.desiredOutput().top(),
                    1,
                    1,
                ));
                let mut surface =
                    AutoSurface::new(ctx, bounds, PixelBoundary::kInitialized, false, None)?;
                if surface.is_valid() {
                    let mut paint = crate::src::core::SkPaint::SkPaint::default();
                    paint.setColor4f(&crate::src::core::SkColor::SkColors::kTransparent, None);
                    paint.setColorFilter(Some(colorFilter));
                    paint.setBlendMode(crate::src::core::SkBlendMode::SkBlendMode::kSrc);
                    crate::src::core::SkCanvas::SkCanvas::drawPaint_raw(surface.canvas(), &paint)?;
                }
                let mut solidColor = surface.snap()?;
                solidColor.updateTileMode(ctx, kClamp);
                return Ok(solidColor);
            }
            if self
                .analyzeLayerBounds(ctx.desiredOutput(), BoundsScope::kDeferred)?
                .contains(BoundsAnalysis::kRequiresLayerCrop)
            {
                newLayerBounds.outset(LayerSpace::new(SkISize::Make(1, 1)));
                let valid = newLayerBounds.intersect(ctx.desiredOutput());
                crate::SkAssertResult!(@source b"src/core/SkImageFilterTypes.cpp\0".as_ptr(),976; @condition b"newLayerBounds.intersect(ctx.desiredOutput())\0".as_ptr(); valid);
                let mut filtered = self.resolve(ctx, newLayerBounds, true)?;
                filtered.fColorFilter = Some(colorFilter);
                filtered.updateTileMode(ctx, kClamp);
                return Ok(filtered);
            }
            newLayerBounds = *ctx.desiredOutput();
        } else if self.fImage.is_none()
            || !LayerSpace::<SkIRect>::Intersects(&newLayerBounds, ctx.desiredOutput())
        {
            return Ok(Self::default());
        }
        let mut filtered = self.clone();
        filtered.fLayerBounds = newLayerBounds;
        filtered.fColorFilter =
            crate::src::core::SkColorFilter::Compose(Some(&colorFilter), self.fColorFilter.clone());
        Ok(filtered)
    }
}

// Source: src/core/SkImageFilterTypes.cpp:1004-1056
fn compatible_sampling(
    current: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
    current_nearest: bool,
    next: &mut crate::include::core::SkSamplingOptions::SkSamplingOptions,
    next_nearest: bool,
) -> bool {
    use crate::include::core::SkSamplingOptions::{
        SkFilterMode::{kLinear, kNearest},
        SkSamplingOptions,
    };
    if current.isAniso() && next.isAniso() {
        *next = SkSamplingOptions::Aniso(current.maxAniso().max(next.maxAniso()));
        true
    } else if current.isAniso() && next.filter() == kLinear {
        *next = *current;
        true
    } else if next.isAniso() && current.filter() == kLinear {
        true
    } else if current.useCubic()
        && (next.filter() == kLinear
            || (next.useCubic()
                && current.cubic().B == next.cubic().B
                && current.cubic().C == next.cubic().C))
    {
        *next = *current;
        true
    } else if next.useCubic() && current.filter() == kLinear {
        true
    } else if current.filter() == kLinear && next.filter() == kLinear {
        true
    } else if next.filter() == kNearest && current_nearest {
        crate::SkASSERT!(@source b"src/core/SkImageFilterTypes.cpp\0".as_ptr(),1044; @condition b"currentSampling.filter == SkFilterMode::kLinear\0".as_ptr(); current.filter() == kLinear);
        true
    } else if current.filter() == kNearest && next_nearest {
        crate::SkASSERT!(@source b"src/core/SkImageFilterTypes.cpp\0".as_ptr(),1048; @condition b"nextSampling->filter == SkFilterMode::kLinear\0".as_ptr(); next.filter() == kLinear);
        *next = *current;
        true
    } else {
        false
    }
}

impl FilterResult {
    // Source: src/core/SkImageFilterTypes.cpp:1204-1207
    /// Safety: target has an exclusive operation lease, live pixels and stable
    /// source-image backing. No callback may invalidate the target during draw.
    // Source: src/core/SkImageFilterTypes.h:808-808
    pub unsafe fn draw(
        &self,
        ctx: &Context<'_>,
        mut target: core::ptr::NonNull<dyn crate::src::core::SkDevice::SkDeviceVirtual + '_>,
        blender: Option<&dyn crate::include::core::SkBlender::SkBlender>,
    ) -> Result<(), crate::RasterError> {
        let previous =
            crate::src::core::SkM44::SkM44::from(target.as_ref().base().local_to_device());
        target
            .as_mut()
            .base_mut()
            .set_local_to_device(ctx.mapping().layerToDeviceMatrix());
        let result = self.draw_internal(ctx, target, true, blender);
        // Result propagation restores exactly as the native transform guard does.
        target.as_mut().base_mut().set_local_to_device(&previous);
        result
    }
    // Source: src/core/SkImageFilterTypes.cpp:1209-1325
    // Source: src/core/SkImageFilterTypes.h:955-958
    unsafe fn draw_internal(
        &self,
        ctx: &Context<'_>,
        mut device: core::ptr::NonNull<dyn crate::src::core::SkDevice::SkDeviceVirtual + '_>,
        preserveDeviceState: bool,
        blender: Option<&dyn crate::include::core::SkBlender::SkBlender>,
    ) -> Result<(), crate::RasterError> {
        use crate::src::core::SkBlendMode::SkBlendMode::kSrc;
        use crate::src::core::SkCanvas::SrcRectConstraint::*;
        use crate::src::core::SkPaint::SkPaint;
        let blendAffects = blender
            .is_some_and(|b| crate::src::core::SkBlenderBase::as_BB(b).affectsTransparentBlack());
        let retained_blender = || {
            let pointer = blender.map(|value| unsafe {
                core::mem::transmute::<
                    core::ptr::NonNull<dyn crate::include::core::SkBlender::SkBlender + '_>,
                    core::ptr::NonNull<dyn crate::include::core::SkBlender::SkBlender + 'static>,
                >(core::ptr::NonNull::from(value))
            });
            unsafe { crate::include::core::SkRefCnt::sk_ref_sp(pointer) }
        };
        if self.fImage.is_none() {
            if blendAffects {
                let mut clear = SkPaint::default();
                clear.setColor4f(&crate::src::core::SkColor::SkColors::kTransparent, None);
                clear.setBlender(retained_blender());
                device.as_mut().drawPaint(&clear)?;
            }
            return Ok(());
        }
        let scope = if blendAffects {
            BoundsScope::kShaderOnly
        } else {
            BoundsScope::kCanDrawDirectly
        };
        let analysis = self.analyzeBounds(
            device.as_ref().base().local_to_device(),
            &device.as_ref().devClipBounds(),
            scope,
        )?;
        if analysis.contains(BoundsAnalysis::kRequiresLayerCrop) {
            if blendAffects {
                let mut dstBounds = LayerSpace::<SkIRect>::Empty();
                if !LayerSpace::new(device.as_ref().base().local_to_device().clone())
                    .inverseMapIRect(
                        &LayerSpace::new(device.as_ref().devClipBounds()),
                        &mut dstBounds,
                    )
                {
                    return Ok(());
                }
                let clipped = self.resolve(ctx, dstBounds, false)?;
                return clipped.draw_internal(ctx, device, preserveDeviceState, blender);
            }
            if preserveDeviceState {
                device.as_mut().pushClipStack();
            }
            device.as_mut().clipRect(
                &SkRect::Make(self.fLayerBounds.geometry()),
                crate::include::core::SkClipOp::SkClipOp::kIntersect,
                true,
            );
        }
        // End each temporary target projection before calling a drawing hook.
        let result = (|| -> Result<(), crate::RasterError> {
            let pixelAligned = is_nearly_integer_translation(&self.fTransform, None)
                && is_nearly_integer_translation(
                    &LayerSpace::new(device.as_ref().base().local_to_device().clone()),
                    None,
                );
            let mut sampling = self.fSamplingOptions;
            if sampling == Self::kDefaultSampling && pixelAligned {
                sampling = crate::include::core::SkSamplingOptions::SkSamplingOptions::default();
            }
            if analysis.contains(BoundsAnalysis::kHasLayerFillingEffect)
                || (blendAffects && analysis.contains(BoundsAnalysis::kDstBoundsNotCovered))
            {
                let mut paint = SkPaint::default();
                if !preserveDeviceState && blender.is_none() {
                    paint.setBlendMode(kSrc);
                } else {
                    paint.setBlender(retained_blender());
                }
                paint.setShader(self.getAnalyzedShaderView(ctx, sampling, analysis)?);
                device.as_mut().drawPaint(&paint)?;
            } else {
                let mut paint = SkPaint::default();
                paint.setBlender(retained_blender());
                paint.setColorFilter(self.fColorFilter.clone());
                let mut net = SkMatrix::Concat(
                    device.as_ref().base().local_to_device(),
                    self.fTransform.geometry(),
                );
                let image = self.fImage.as_ref().unwrap();
                if self.canClampToTransparentBoundary(analysis)
                    && self.fSamplingOptions == Self::kDefaultSampling
                {
                    crate::SkASSERT!(@source b"src/core/SkImageFilterTypes.cpp\0".as_ptr(),1298; @condition b"!(analysis & BoundsAnalysis::kRequiresShaderTiling)\0".as_ptr(); !analysis.contains(BoundsAnalysis::kRequiresShaderTiling));
                    if !preserveDeviceState && blender.is_none() {
                        paint.setBlendMode(kSrc);
                    }
                    net.preTranslate(-1.0, -1.0);
                    let outset = image
                        .makePixelOutset()
                        .expect("native padded image requires valid outset");
                    device.as_mut().drawSpecialWithConstraint(
                        outset.as_ref(),
                        &net,
                        &sampling,
                        &paint,
                        kFast_SrcRectConstraint,
                    )?;
                } else {
                    paint.setAntiAlias(true);
                    let constraint = if analysis.contains(BoundsAnalysis::kRequiresShaderTiling) {
                        ctx.markShaderBasedTilingRequired(
                            crate::include::core::SkTileMode::SkTileMode::kClamp,
                        );
                        kStrict_SrcRectConstraint
                    } else {
                        kFast_SrcRectConstraint
                    };
                    device.as_mut().drawSpecialWithConstraint(
                        image.as_ref(),
                        &net,
                        &sampling,
                        &paint,
                        constraint,
                    )?;
                }
            }
            Ok(())
        })();
        if preserveDeviceState && analysis.contains(BoundsAnalysis::kRequiresLayerCrop) {
            device.as_mut().popClipStack();
        }
        result
    }
}

// Source: src/core/SkImageFilterTypes.h:818-832
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShaderFlags(u32);
impl ShaderFlags {
    pub const kNone: Self = Self(0);
    pub const kSampledRepeatedly: Self = Self(1 << 0);
    pub const kNonTrivialSampling: Self = Self(1 << 1);
    pub fn contains(self, flag: Self) -> bool {
        self.0 & flag.0 != 0
    }
}
impl core::ops::BitOr for ShaderFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
// Source: src/core/SkImageFilterTypes.h:1004-1004
// Adapted: ShaderFlags retains the native enum-mask bits in one Rust newtype.
// Operators follow SK_MAKE_BITMASK_OPS/SkEnumBitMask; no unknown bits are dropped.
impl core::ops::BitAnd for ShaderFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}
impl core::ops::BitXor for ShaderFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }
}
impl core::ops::Not for ShaderFlags {
    type Output = Self;
    fn not(self) -> Self {
        Self(!self.0)
    }
}
impl core::ops::BitOrAssign for ShaderFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}
impl core::ops::BitAndAssign for ShaderFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}
impl core::ops::BitXorAssign for ShaderFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs;
    }
}

impl FilterResult {
    // Source: src/core/SkImageFilterTypes.cpp:1327-1389
    /// Safety: retain source-image and Context's backend/pixel draw leases
    /// throughout resolution and all subsequent uses of the returned shader.
    // Source: src/core/SkImageFilterTypes.h:966-969
    pub unsafe fn asShader(
        &self,
        ctx: &Context<'_>,
        xtraSampling: crate::include::core::SkSamplingOptions::SkSamplingOptions,
        flags: ShaderFlags,
        sampleBounds: LayerSpace<SkIRect>,
    ) -> Result<
        Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>>,
        crate::RasterError,
    > {
        let Some(image) = self.fImage.as_ref() else {
            return Ok(None);
        };
        let currentInteger = is_nearly_integer_translation(&self.fTransform, None);
        let nextInteger = !flags.contains(ShaderFlags::kNonTrivialSampling);
        let mut mode = crate::src::core::SkBlendMode::SkBlendMode::kClear;
        let mut analysis = self.analyzeLayerBounds(&sampleBounds, BoundsScope::kShaderOnly)?;
        let mut sampling = xtraSampling;
        let needsResolve = (flags.contains(ShaderFlags::kSampledRepeatedly)
            && (self.fColorFilter.as_ref().is_some_and(|f| {
                !f.asAColorMode(None, Some(&mut mode))
                    || mode > crate::src::core::SkBlendMode::SkBlendMode::kLastCoeffMode
            }) || !crate::src::core::SkColorSpace::SkColorSpace::Equals(
                image.colorInfo().colorSpace(),
                ctx.colorSpace(),
            )))
            || !compatible_sampling(
                &self.fSamplingOptions,
                currentInteger,
                &mut sampling,
                nextInteger,
            )
            || analysis.contains(BoundsAnalysis::kRequiresLayerCrop);
        if sampling == Self::kDefaultSampling && nextInteger && (needsResolve || currentInteger) {
            sampling = crate::include::core::SkSamplingOptions::SkSamplingOptions::default();
        }
        if needsResolve {
            let resolved = self.resolve(ctx, sampleBounds, false)?;
            if resolved.has_image() {
                analysis = resolved.analyzeLayerBounds(&sampleBounds, BoundsScope::kShaderOnly)?;
                crate::SkASSERT!(@source b"src/core/SkImageFilterTypes.cpp\0".as_ptr(),1381; @condition b"!(analysis & ~kExpectedAnalysis)\0".as_ptr(); analysis.0 & !(BoundsAnalysis::kDstBoundsNotCovered.0|BoundsAnalysis::kRequiresShaderTiling.0)==0);
                return resolved.getAnalyzedShaderView(ctx, sampling, analysis);
            }
            Ok(None)
        } else {
            self.getAnalyzedShaderView(ctx, sampling, analysis)
        }
    }
}

// Private continuation storage for the two translated regions around the
// RuntimeShaderBuilder decal block (cpp1437-1453). The actual entry above
// evaluates that block before applying the existing suffix.
struct AnalyzedShaderParts {
    image_shader: Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>>,
    image_bounds: SkRect,
    pre_decal: SkMatrix,
    post_decal: SkMatrix,
}
impl FilterResult {
    // Source: src/core/SkImageFilterTypes.cpp:1391-1468
    // Prefix/suffix retain their existing translations; only cpp1437-1453 below
    // adds the previously missing actual RuntimeEffect builder branch.
    unsafe fn getAnalyzedShaderView(
        &self,
        ctx: &Context<'_>,
        sampling: crate::include::core::SkSamplingOptions::SkSamplingOptions,
        analysis: BoundsAnalysis,
    ) -> Result<
        Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>>,
        crate::RasterError,
    > {
        let mut parts = self.analyzed_shader_prefix(ctx, sampling, analysis);
        // Source: src/core/SkImageFilterTypes.cpp:1437-1453
        if analysis.contains(BoundsAnalysis::kRequiresDecalInLayerSpace) {
            crate::SkASSERT!(@source b"src/core/SkImageFilterTypes.cpp\0".as_ptr(),1438; @condition b"fTileMode == SkTileMode::kDecal\0".as_ptr(); self.fTileMode==crate::include::core::SkTileMode::SkTileMode::kDecal);
            let effect = crate::src::core::SkKnownRuntimeEffects::GetKnownRuntimeEffect(
                crate::src::core::SkKnownRuntimeEffects::StableKey::kDecal,
            )
            .expect("native known decal effect")
            .clone();
            let mut builder =
                crate::src::core::SkRuntimeEffect::SkRuntimeEffectBuilder::new(effect);
            builder.child(b"image").assign(
                parts
                    .image_shader
                    .take()
                    .map(crate::src::core::SkRuntimeEffect::ChildPtr::Shader)
                    .unwrap_or(crate::src::core::SkRuntimeEffect::ChildPtr::Empty),
            );
            let bounds = parts.pre_decal.mapRect_value(&parts.image_bounds);
            builder.uniform(b"decalBounds").assign(&bounds);
            parts.image_shader = builder.makeShader(None);
        }
        Ok(self.analyzed_shader_suffix(parts.image_shader, &parts.post_decal, analysis))
    }
}
impl FilterResult {
    // Source: src/core/SkImageFilterTypes.cpp:1391-1435
    unsafe fn analyzed_shader_prefix(
        &self,
        ctx: &Context<'_>,
        finalSampling: crate::include::core::SkSamplingOptions::SkSamplingOptions,
        analysis: BoundsAnalysis,
    ) -> AnalyzedShaderParts {
        use crate::include::core::SkTileMode::SkTileMode::*;
        let local = self.fTransform.geometry();
        let image = self
            .fImage
            .as_ref()
            .expect("native shader analysis requires an image");
        let imageBounds = SkRect::MakeISize(image.dimensions());
        let mut post = SkMatrix::default();
        let mut pre = if local.rectStaysRect()
            || !analysis.contains(BoundsAnalysis::kRequiresDecalInLayerSpace)
        {
            local.clone()
        } else {
            decompose_transform(local, imageBounds.center(), Some(&mut post))
        };
        let mut effective = self.fTileMode;
        let decalClamp = self.canClampToTransparentBoundary(analysis);
        let strict = analysis.contains(BoundsAnalysis::kRequiresShaderTiling);
        let shader;
        if strict && decalClamp {
            pre.preTranslate(-1.0, -1.0);
            shader = image
                .makePixelOutset()
                .expect("native known pixel outset")
                .asShader(kClamp, &finalSampling, &pre, strict);
            effective = kClamp;
        } else {
            if !analysis.contains(BoundsAnalysis::kDstBoundsNotCovered)
                || analysis.contains(BoundsAnalysis::kRequiresDecalInLayerSpace)
            {
                effective = kClamp;
            }
            shader = image.asShader(effective, &finalSampling, &pre, strict);
        }
        if strict {
            ctx.markShaderBasedTilingRequired(effective);
        }
        AnalyzedShaderParts {
            image_shader: shader,
            image_bounds: imageBounds,
            pre_decal: pre,
            post_decal: post,
        }
    }
    // Source: src/core/SkImageFilterTypes.cpp:1455-1468
    fn analyzed_shader_suffix(
        &self,
        mut imageShader: Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>>,
        postDecal: &SkMatrix,
        analysis: BoundsAnalysis,
    ) -> Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>> {
        if analysis.contains(BoundsAnalysis::kRequiresDecalInLayerSpace) {
            if let Some(shader) = imageShader.as_ref() {
                imageShader = Some(shader.makeWithLocalMatrix(postDecal));
            }
        }
        if self.fColorFilter.is_some() {
            if let Some(shader) = imageShader.as_ref() {
                imageShader = shader.makeWithColorFilter(self.fColorFilter.clone());
            }
        }
        imageShader
    }
}

// Source: src/core/SkImageFilterTypes.cpp:1476-1477
// Native local alias names the low-resolution pixel coordinates without creating a new space type.
type PixelSpace<T> = LayerSpace<T>;
// Source: src/core/SkImageFilterTypes.cpp:1479-1500
fn downscale_step_count(net_scale: f32) -> i32 {
    let mut steps = crate::src::core::SkMathPriv::SkNextLog2(
        crate::src::core::SkFloatingPoint::sk_float_ceil2int(1.0 / net_scale) as u32,
    );
    if steps > 0 {
        let final_scale = net_scale * (1i32 << (steps - 1)) as f32;
        let limit = if steps == 1 { 1.0 - kRoundEpsilon } else { 0.9 };
        if final_scale >= limit {
            steps -= 1;
        }
    }
    steps
}
// Source: src/core/SkImageFilterTypes.cpp:1502-1507
fn scale_about_center(src: PixelSpace<SkRect>, sx: f32, sy: f32) -> LayerSpace<SkRect> {
    let cx = if sx == 1.0 {
        0.0
    } else {
        0.5 * src.left() + 0.5 * src.right()
    };
    let cy = if sy == 1.0 {
        0.0
    } else {
        0.5 * src.top() + 0.5 * src.bottom()
    };
    LayerSpace::new(SkRect {
        fLeft: (src.left() - cx) * sx,
        fTop: (src.top() - cy) * sy,
        fRight: (src.right() - cx) * sx,
        fBottom: (src.bottom() - cy) * sy,
    })
}

// Source: src/core/SkImageFilterTypes.h:1102-1134
// Rust trait composition retains the actual cache/props/color metadata and
// native abstract CPU hooks. No default backend/blur implementation is supplied.
pub trait Backend {
    fn state(&self) -> &BackendState;
    /// Safety: returned actual device owns its offscreen backing; callers retain
    /// all native draw/COW leases. No parent recorder borrow may escape here.
    unsafe fn makeDevice(
        &self,
        size: SkISize,
        colorSpace: Option<std::sync::Arc<crate::src::core::SkColorSpace::SkColorSpace>>,
        props: Option<&crate::include::core::SkSurfaceProps::SkSurfaceProps>,
    ) -> crate::include::core::SkRefCnt::sk_sp<dyn crate::src::core::SkDevice::SkDeviceVirtual>;
    fn makeImage(
        &self,
        subset: &SkIRect,
        image: Option<std::sync::Arc<dyn crate::src::image::SkImage::SkImage>>,
    ) -> Option<std::sync::Arc<dyn super::SkSpecialImage::SkSpecialImage>>;
    /// Safety: data retains the complete native readable/copy-or-share image lease.
    unsafe fn getCachedBitmap(
        &self,
        data: &crate::src::core::SkBitmap::SkBitmap,
    ) -> Option<std::sync::Arc<dyn crate::src::image::SkImage::SkImage>>;
    fn getBlurEngine(&self) -> &dyn super::SkBlurEngine::SkBlurEngine;
    fn surfaceProps(&self) -> &crate::include::core::SkSurfaceProps::SkSurfaceProps {
        &self.state().fSurfaceProps
    }
    fn colorType(&self) -> crate::include::core::SkColorType::SkColorType {
        self.state().fColorType
    }
    fn cache(&self) -> Option<&super::SkImageFilterCache::SkImageFilterCache> {
        self.state().fCache.as_deref()
    }
}
// Source: src/core/SkImageFilterTypes.h:1131-1133
// Reverse native member teardown order; the cache owner drops last.
pub struct BackendState {
    fColorType: crate::include::core::SkColorType::SkColorType,
    fSurfaceProps: crate::include::core::SkSurfaceProps::SkSurfaceProps,
    fCache: Option<std::sync::Arc<super::SkImageFilterCache::SkImageFilterCache>>,
}
impl BackendState {
    // Source: src/core/SkImageFilterTypes.cpp:216-223
    pub fn new(
        cache: Option<std::sync::Arc<super::SkImageFilterCache::SkImageFilterCache>>,
        surfaceProps: crate::include::core::SkSurfaceProps::SkSurfaceProps,
        colorType: crate::include::core::SkColorType::SkColorType,
    ) -> Self {
        Self {
            fColorType: colorType,
            fSurfaceProps: surfaceProps,
            fCache: cache,
        }
    }
}
// Source: src/core/SkImageFilterTypes.h:1156-1270
// Rust borrow adaptation: all copied contexts share the same borrowed Cell of
// Stats. Each increment borrows it only for get/update/set, never clones a new
// counter owner. Color-space inputs are genuine retained Arcs, not reconstructed
// from a bare &SkColorSpace. The engine remains an actual required Backend hook.
pub struct Context<'stats> {
    fStats: Option<&'stats core::cell::Cell<Stats>>,
    fColorSpace: Option<std::sync::Arc<crate::src::core::SkColorSpace::SkColorSpace>>,
    fSource: FilterResult,
    fDesiredOutput: LayerSpace<SkIRect>,
    fMapping: Mapping,
    fBackend: std::sync::Arc<dyn Backend>,
}
impl Clone for Context<'_> {
    fn clone(&self) -> Self {
        // Native construction retains backend before source, then color space;
        // declaration order above separately preserves reverse destruction.
        let backend = self.fBackend.clone();
        let mapping = self.fMapping;
        let desired = self.fDesiredOutput;
        let source = self.fSource.clone();
        let color = self.fColorSpace.clone();
        Self {
            fBackend: backend,
            fMapping: mapping,
            fDesiredOutput: desired,
            fSource: source,
            fColorSpace: color,
            fStats: self.fStats,
        }
    }
}
impl<'stats> Context<'stats> {
    pub fn new(
        backend: std::sync::Arc<dyn Backend>,
        mapping: &Mapping,
        desiredOutput: LayerSpace<SkIRect>,
        source: &FilterResult,
        colorSpace: Option<std::sync::Arc<crate::src::core::SkColorSpace::SkColorSpace>>,
        stats: Option<&'stats core::cell::Cell<Stats>>,
    ) -> Self {
        Self {
            fStats: stats,
            fColorSpace: colorSpace,
            fSource: source.clone(),
            fDesiredOutput: desiredOutput,
            fMapping: *mapping,
            fBackend: backend,
        }
    }
    pub fn backend(&self) -> &dyn Backend {
        &*self.fBackend
    }
    pub fn mapping(&self) -> &Mapping {
        &self.fMapping
    }
    pub fn desiredOutput(&self) -> &LayerSpace<SkIRect> {
        &self.fDesiredOutput
    }
    pub fn colorSpace(&self) -> Option<&crate::src::core::SkColorSpace::SkColorSpace> {
        self.fColorSpace.as_deref()
    }
    pub fn refColorSpace(
        &self,
    ) -> Option<std::sync::Arc<crate::src::core::SkColorSpace::SkColorSpace>> {
        self.fColorSpace.clone()
    }
    pub fn source(&self) -> &FilterResult {
        &self.fSource
    }
    pub fn withNewMapping(&self, mapping: &Mapping) -> Self {
        let mut c = self.clone();
        c.fMapping = *mapping;
        c
    }
    pub fn withNewDesiredOutput(&self, desiredOutput: LayerSpace<SkIRect>) -> Self {
        let mut c = self.clone();
        c.fDesiredOutput = desiredOutput;
        c
    }
    pub fn withNewColorSpace(
        &self,
        cs: Option<std::sync::Arc<crate::src::core::SkColorSpace::SkColorSpace>>,
    ) -> Self {
        let mut c = self.clone();
        c.fColorSpace = cs;
        c
    }
    pub fn withNewSource(&self, source: &FilterResult) -> Self {
        let mut c = self.clone();
        c.fSource.clone_from(source);
        c
    }
    fn updateStats(&self, update: impl FnOnce(&mut Stats)) {
        if let Some(stats) = self.fStats {
            let mut value = stats.get();
            update(&mut value);
            stats.set(value);
        }
    }
    pub fn markVisitedImageFilter(&self) {
        self.updateStats(|s| s.fNumVisitedImageFilters += 1);
    }
    pub fn markCacheHit(&self) {
        self.updateStats(|s| s.fNumCacheHits += 1);
    }
    pub fn markNewSurface(&self) {
        self.updateStats(|s| s.fNumOffscreenSurfaces += 1);
    }
    pub fn markShaderBasedTilingRequired(
        &self,
        tileMode: crate::include::core::SkTileMode::SkTileMode,
    ) {
        self.updateStats(|s| {
            if tileMode == crate::include::core::SkTileMode::SkTileMode::kClamp {
                s.fNumShaderClampedDraws += 1;
            } else {
                s.fNumShaderBasedTilingDraws += 1;
            }
        });
    }
}

// Source: src/core/SkImageFilterTypes.h:873-895
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BoundsAnalysis(u32);
impl BoundsAnalysis {
    pub const kSimple: Self = Self(0);
    pub const kDstBoundsNotCovered: Self = Self(1 << 0);
    pub const kHasLayerFillingEffect: Self = Self(1 << 1);
    pub const kRequiresLayerCrop: Self = Self(1 << 2);
    pub const kRequiresShaderTiling: Self = Self(1 << 3);
    pub const kRequiresDecalInLayerSpace: Self = Self(1 << 4);
    pub fn contains(self, flag: Self) -> bool {
        self.0 & flag.0 != 0
    }
}
impl core::ops::BitOrAssign for BoundsAnalysis {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}
// Source: src/core/SkImageFilterTypes.h:1005-1005
// Adapted: same native mask operations; preserve every underlying 32-bit bit.
impl core::ops::BitOr for BoundsAnalysis {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
impl core::ops::BitAnd for BoundsAnalysis {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}
impl core::ops::BitXor for BoundsAnalysis {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }
}
impl core::ops::Not for BoundsAnalysis {
    type Output = Self;
    fn not(self) -> Self {
        Self(!self.0)
    }
}
impl core::ops::BitAndAssign for BoundsAnalysis {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}
impl core::ops::BitXorAssign for BoundsAnalysis {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs;
    }
}

// Source: src/core/SkImageFilterTypes.h:899-904
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoundsScope {
    kDeferred,
    kCanDrawDirectly,
    kShaderOnly,
    kRescale,
}
impl FilterResult {
    // Source: src/core/SkImageFilterTypes.cpp:677-825
    // Rust error adapter preserves the actual transparent-black query error;
    // analysis itself retains the native geometry, sampling and mask order.
    // Source: src/core/SkImageFilterTypes.h:911-913
    pub fn analyzeBounds(
        &self,
        xtraTransform: &SkMatrix,
        dstBounds: &SkIRect,
        scope: BoundsScope,
    ) -> Result<BoundsAnalysis, crate::RasterError> {
        use crate::include::core::SkScalar::SkScalarNearlyEqual;
        use crate::include::core::SkTileMode::SkTileMode::*;
        use crate::src::core::SkM44::SkM44;
        use crate::src::core::SkRectPriv::SkRectPriv;
        let nearest = crate::include::core::SkSamplingOptions::SkSamplingOptions::default();
        const HALF: f32 = 0.5;
        const CUBIC_RADIUS: f32 = 1.5;
        let mut analysis = BoundsAnalysis::kSimple;
        // Keep native short circuit: a non-decal tile never evaluates the CF.
        let fills = self.fTileMode != kDecal
            || match self.fColorFilter.as_ref() {
                Some(cf) => cf.affectsTransparentBlack()?,
                None => false,
            };
        let image = self
            .fImage
            .as_ref()
            .expect("native analyzeBounds requires an image");
        let mut pixelCenterBounds = SkRect::Make(dstBounds);
        if !SkRectPriv::QuadContainsIRect(
            xtraTransform,
            self.fLayerBounds.geometry(),
            dstBounds,
            kRoundEpsilon,
        ) {
            let requireLayerCrop = if fills {
                true
            } else {
                let imageBounds = self
                    .fTransform
                    .mapIRect(&LayerSpace::from_size(image.dimensions()));
                !self.fLayerBounds.contains(&imageBounds)
            };
            if requireLayerCrop {
                analysis |= BoundsAnalysis::kRequiresLayerCrop;
                let layerBoundsInDst = self.fLayerBounds.geometry().map_geometry(xtraTransform);
                let _ = pixelCenterBounds.intersect(&SkRect::Make(&layerBoundsInDst));
            }
        }
        let imageBounds = SkRect::MakeISize(image.dimensions());
        let mut netTransform = self.fTransform.clone();
        netTransform.postConcat(&LayerSpace::new(xtraTransform.clone()));
        let netM44 = SkM44::from(netTransform.geometry());
        let (xAxisAligned, yAxisAligned) = are_axes_nearly_integer_aligned(&netTransform, None);
        let isPixelAligned = xAxisAligned && yAxisAligned;
        let decalLeaks = scope != BoundsScope::kRescale
            && self.fTileMode == kDecal
            && self.fSamplingOptions != nearest
            && !isPixelAligned;
        let sampleRadius = if self.fSamplingOptions.useCubic() {
            CUBIC_RADIUS
        } else {
            HALF
        };
        let mut safeImageBounds = imageBounds;
        safeImageBounds.inset(sampleRadius, sampleRadius);
        if self.fSamplingOptions == Self::kDefaultSampling && !isPixelAligned {
            safeImageBounds.inset(
                if xAxisAligned { 0.0 } else { kRoundEpsilon },
                if yAxisAligned { 0.0 } else { kRoundEpsilon },
            );
        }
        let mut hasPixelPadding = self.fBoundary != PixelBoundary::kUnknown;
        if !SkRectPriv::QuadContainsRect(
            &netM44,
            if decalLeaks {
                &safeImageBounds
            } else {
                &imageBounds
            },
            &pixelCenterBounds,
            kRoundEpsilon,
        ) {
            analysis |= BoundsAnalysis::kDstBoundsNotCovered;
            if fills {
                analysis |= BoundsAnalysis::kHasLayerFillingEffect;
            }
            if decalLeaks {
                let mut scaleFactors = [0.0; 2];
                if !(netTransform.geometry().getMinMaxScales(&mut scaleFactors)
                    && SkScalarNearlyEqual(scaleFactors[0], 1.0, 0.2)
                    && SkScalarNearlyEqual(scaleFactors[1], 1.0, 0.2))
                {
                    analysis |= BoundsAnalysis::kRequiresDecalInLayerSpace;
                    if self.fBoundary == PixelBoundary::kTransparent {
                        hasPixelPadding = false;
                    }
                }
            }
        }
        if scope == BoundsScope::kDeferred {
            return Ok(analysis);
        } else if scope == BoundsScope::kCanDrawDirectly
            && !analysis.contains(BoundsAnalysis::kHasLayerFillingEffect)
        {
            let nnOrBilerp =
                self.fSamplingOptions == Self::kDefaultSampling || self.fSamplingOptions == nearest;
            if nnOrBilerp && (hasPixelPadding || isPixelAligned) {
                return Ok(analysis);
            }
        }
        if hasPixelPadding {
            safeImageBounds.outset(1.0, 1.0);
        }
        pixelCenterBounds.inset(HALF, HALF);
        let edgeMask = SkRectPriv::QuadContainsRectMask(
            &netM44,
            &safeImageBounds,
            &pixelCenterBounds,
            kRoundEpsilon,
        );
        if !edgeMask.iter().all(|&x| x != 0) {
            let subset = image.subset();
            let dimensions = image.backingStoreDimensions();
            // Native hwEdge constructor stores boolean 0/1 lanes, while the
            // QuadContainsRectMask comparisons produce full-bit lanes.
            let mut hwEdge = [
                (subset.fTop == 0) as i32,
                (subset.fRight == dimensions.fWidth) as i32,
                (subset.fBottom == dimensions.fHeight) as i32,
                (subset.fLeft == 0) as i32,
            ];
            if self.fTileMode == kRepeat || self.fTileMode == kMirror {
                let original = hwEdge;
                for i in 0..4 {
                    hwEdge[i] &= original[(i + 2) % 4];
                }
            }
            if !(0..4).all(|i| edgeMask[i] | hwEdge[i] != 0) {
                analysis |= BoundsAnalysis::kRequiresShaderTiling;
            }
        }
        Ok(analysis)
    }
    // Source: src/core/SkImageFilterTypes.h:914-917
    pub fn analyzeLayerBounds(
        &self,
        dstBounds: &LayerSpace<SkIRect>,
        scope: BoundsScope,
    ) -> Result<BoundsAnalysis, crate::RasterError> {
        self.analyzeBounds(&SkMatrix::default(), dstBounds.geometry(), scope)
    }
    // Source: src/core/SkImageFilterTypes.h:922-926
    pub fn canClampToTransparentBoundary(&self, analysis: BoundsAnalysis) -> bool {
        self.fTileMode == crate::include::core::SkTileMode::SkTileMode::kDecal
            && self.fBoundary == PixelBoundary::kTransparent
            && !analysis.contains(BoundsAnalysis::kRequiresDecalInLayerSpace)
    }
}

impl FilterResult {
    // Source: src/core/SkImageFilterTypes.cpp:827-834
    // Source: src/core/SkImageFilterTypes.h:981-981
    fn updateTileMode(
        &mut self,
        ctx: &Context<'_>,
        tileMode: crate::include::core::SkTileMode::SkTileMode,
    ) {
        if self.fImage.is_some() {
            self.fTileMode = tileMode;
            if tileMode != crate::include::core::SkTileMode::SkTileMode::kDecal {
                self.fLayerBounds = *ctx.desiredOutput();
            }
        }
    }
}

// Source: src/core/SkImageFilterTypes.cpp:528-629
// Rust owns the actual offscreen Canvas at a pinned address. All COW-capable
// calls use its raw entry and end temporary projections before callbacks.
struct AutoSurface {
    fBoundary: PixelBoundary,
    fDstBounds: LayerSpace<SkIRect>,
    fCanvas: Option<core::pin::Pin<Box<crate::src::core::SkCanvas::SkCanvas<'static>>>>,
}
impl AutoSurface {
    /// Safety: Backend's returned owner has the genuine native independent
    /// offscreen backing/recorder lease. All draws preserve Canvas pixel/COW
    /// contracts; no external mutable owner may alias this temporary surface.
    unsafe fn new(
        ctx: &Context<'_>,
        dstBounds: LayerSpace<SkIRect>,
        boundary: PixelBoundary,
        renderInParameterSpace: bool,
        props: Option<&crate::include::core::SkSurfaceProps::SkSurfaceProps>,
    ) -> Result<Self, crate::RasterError> {
        use crate::include::core::SkClipOp::SkClipOp;
        use crate::src::core::SkCanvas::SkCanvas;
        let mut surface = Self {
            fBoundary: boundary,
            fDstBounds: dstBounds,
            fCanvas: None,
        };
        let mut device = crate::include::core::SkRefCnt::sk_sp::default();
        if !dstBounds.isEmpty() {
            let padding = surface.padding();
            if padding != 0 {
                surface
                    .fDstBounds
                    .outset(LayerSpace::new(SkISize::Make(padding, padding)));
                if surface.fDstBounds.left() >= dstBounds.left()
                    || surface.fDstBounds.right() <= dstBounds.right()
                    || surface.fDstBounds.top() >= dstBounds.top()
                    || surface.fDstBounds.bottom() <= dstBounds.bottom()
                {
                    return Ok(surface);
                }
            }
            device = unsafe {
                ctx.backend().makeDevice(
                    *surface.fDstBounds.size().geometry(),
                    ctx.refColorSpace(),
                    props,
                )
            };
        }
        if device.get().is_none() {
            return Ok(surface);
        }
        ctx.markNewSurface();
        let mut canvas = unsafe { SkCanvas::with_device(device) };
        // Preserve original integer negation before conversion; INT_MIN is
        // outside native defined negation domain, not silently saturated here.
        unsafe {
            canvas.as_mut().translate(
                (-surface.fDstBounds.left()) as f32,
                (-surface.fDstBounds.top()) as f32,
            );
        }
        let raw = core::ptr::NonNull::from(unsafe { canvas.as_mut().get_unchecked_mut() });
        unsafe {
            SkCanvas::clear_raw(raw, 0)?;
        }
        let clip = if boundary == PixelBoundary::kTransparent {
            dstBounds.geometry()
        } else {
            surface.fDstBounds.geometry()
        };
        unsafe {
            SkCanvas::clipIRect_raw(raw, clip, SkClipOp::kIntersect);
        }
        if renderInParameterSpace {
            unsafe {
                canvas.as_mut().concat44(ctx.mapping().layerMatrix());
            }
        }
        surface.fCanvas = Some(canvas);
        Ok(surface)
    }
    fn padding(&self) -> i32 {
        if self.fBoundary == PixelBoundary::kUnknown {
            0
        } else {
            1
        }
    }
    fn is_valid(&self) -> bool {
        self.fCanvas.is_some()
    }
    fn canvas(&mut self) -> core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'static>> {
        let canvas = self
            .fCanvas
            .as_mut()
            .expect("native AutoSurface requires a canvas");
        core::ptr::NonNull::from(unsafe { canvas.as_mut().get_unchecked_mut() })
    }
    unsafe fn device(
        &mut self,
    ) -> core::ptr::NonNull<dyn crate::src::core::SkDevice::SkDeviceVirtual> {
        unsafe { crate::src::core::SkCanvas::SkCanvas::top_device_raw(self.canvas()) }
    }
    unsafe fn snap(&mut self) -> Result<FilterResult, crate::RasterError> {
        if self.fCanvas.is_none() {
            return Ok(FilterResult::default());
        }
        unsafe {
            crate::src::core::SkCanvas::SkCanvas::restoreToCount_raw(self.canvas(), 0)?;
        }
        let mut device = unsafe { self.device() };
        unsafe { device.as_mut() }.setImmutable();
        let mut subset = SkIRect::make_wh(self.fDstBounds.width(), self.fDstBounds.height());
        let image = unsafe { device.as_mut() }.snapSpecial(&subset, false)?;
        self.fCanvas = None;
        if let Some(image) = image
            .as_ref()
            .filter(|_| self.fBoundary != PixelBoundary::kUnknown)
        {
            let padding = self.padding();
            subset = SkIRect::make_size(image.dimensions()).make_inset(padding, padding);
            let origin = LayerSpace::new(SkIPoint::make(
                self.fDstBounds.left() + padding,
                self.fDstBounds.top() + padding,
            ));
            Ok(FilterResult::with_boundary(
                image.makeSubset(&subset),
                origin,
                self.fBoundary,
            ))
        } else {
            Ok(FilterResult::with_boundary(
                image,
                self.fDstBounds.topLeft(),
                PixelBoundary::kUnknown,
            ))
        }
    }
}

// Source: src/core/SkImageFilterTypes.cpp:1509-1531
/// Safety: canvas is an exclusively callable live AutoSurface canvas with the
/// native pixel/COW lease. Any error aborts this offscreen rendering operation.
unsafe fn draw_color_filtered_border(
    canvas: core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'static>>,
    border: PixelSpace<SkIRect>,
    colorFilter: Option<std::sync::Arc<dyn crate::src::core::SkColorFilter::SkColorFilter>>,
) -> Result<(), crate::RasterError> {
    use crate::src::core::SkCanvas::SkCanvas;
    let mut cfOnly = crate::src::core::SkPaint::SkPaint::default();
    cfOnly.setColor4f(&crate::src::core::SkColor::SkColors::kTransparent, None);
    cfOnly.setColorFilter(colorFilter);
    cfOnly.setBlendMode(crate::src::core::SkBlendMode::SkBlendMode::kSrc);
    unsafe {
        SkCanvas::drawIRect_raw(
            canvas,
            &SkIRect::make_ltrb(
                border.left(),
                border.top(),
                border.right(),
                border.top() + 1,
            ),
            &cfOnly,
        )?;
        SkCanvas::drawIRect_raw(
            canvas,
            &SkIRect::make_ltrb(
                border.left(),
                border.bottom() - 1,
                border.right(),
                border.bottom(),
            ),
            &cfOnly,
        )?;
        SkCanvas::drawIRect_raw(
            canvas,
            &SkIRect::make_ltrb(
                border.left(),
                border.top() + 1,
                border.left() + 1,
                border.bottom() - 1,
            ),
            &cfOnly,
        )?;
        SkCanvas::drawIRect_raw(
            canvas,
            &SkIRect::make_ltrb(
                border.right() - 1,
                border.top() + 1,
                border.right(),
                border.bottom() - 1,
            ),
            &cfOnly,
        )?;
    }
    Ok(())
}
// Source: src/core/SkImageFilterTypes.cpp:1533-1613
/// Safety: same private offscreen lease as draw_color_filtered_border. Native
/// srcToDst is invertible and each draw follows the existing Canvas COW policy.
/// An error terminates the offscreen operation; owner drop restores its stack.
unsafe fn draw_tiled_border(
    canvas: core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'static>>,
    tileMode: crate::include::core::SkTileMode::SkTileMode,
    paint: &crate::src::core::SkPaint::SkPaint,
    srcToDst: &PixelSpace<SkMatrix>,
    mut srcBorder: PixelSpace<SkRect>,
    mut dstBorder: PixelSpace<SkRect>,
) -> Result<(), crate::RasterError> {
    use crate::include::core::SkTileMode::SkTileMode::*;
    use crate::src::core::SkCanvas::SkCanvas;
    crate::SkASSERT!(@source b"src/core/SkImageFilterTypes.cpp\0".as_ptr(),1539; @condition b"tileMode != SkTileMode::kDecal\0".as_ptr(); tileMode != kDecal);
    let drawEdge = |src: SkRect, dst: SkRect| -> Result<(), crate::RasterError> {
        unsafe {
            core::pin::Pin::new_unchecked(&mut *canvas.as_ptr()).save();
        }
        let matrix = SkMatrix::RectToRectOrIdentity(
            &src,
            &dst,
            crate::src::core::SkMatrix::ScaleToFit::kFill_ScaleToFit,
        );
        unsafe {
            core::pin::Pin::new_unchecked(&mut *canvas.as_ptr()).concat(&matrix);
        }
        unsafe {
            SkCanvas::drawRect_raw(canvas, &src, paint)?;
            SkCanvas::restore_raw(canvas)?;
        }
        Ok(())
    };
    let drawCorner = |src: SkPoint, dst: SkPoint| -> Result<(), crate::RasterError> {
        drawEdge(
            SkRect::MakeLTRB(src.fX, src.fY, src.fX + 1.0, src.fY + 1.0),
            SkRect::MakeLTRB(dst.fX, dst.fY, dst.fX + 1.0, dst.fY + 1.0),
        )
    };
    let mut dstSampleBounds = dstBorder;
    dstSampleBounds.inset(LayerSpace::new(SkSize::Make(1.0, 1.0)));
    let mut srcSampleBounds = LayerSpace::<SkRect>::Empty();
    let valid = srcToDst.inverseMapRect(&dstSampleBounds, &mut srcSampleBounds);
    crate::SkAssertResult!(@source b"src/core/SkImageFilterTypes.cpp\0".as_ptr(),1562; @condition b"srcToDst.inverseMapRect(dstSampleBounds, &srcSampleBounds)\0".as_ptr(); valid);
    if tileMode == kMirror || tileMode == kRepeat {
        srcBorder = dstSampleBounds;
        srcBorder.inset(LayerSpace::new(SkSize::Make(0.5, 0.5)));
        let original = srcBorder;
        let valid = srcToDst.inverseMapRect(&original, &mut srcBorder);
        crate::SkAssertResult!(@source b"src/core/SkImageFilterTypes.cpp\0".as_ptr(),1571; @condition b"srcToDst.inverseMapRect(srcBorder, &srcBorder)\0".as_ptr(); valid);
        srcBorder.outset(LayerSpace::new(SkSize::Make(0.5, 0.5)));
    }
    if tileMode == kRepeat {
        dstBorder = LayerSpace::new(SkRect::MakeLTRB(
            dstBorder.right() - 1.0,
            dstBorder.bottom() - 1.0,
            dstBorder.left() + 1.0,
            dstBorder.top() + 1.0,
        ));
    }
    drawEdge(
        SkRect::MakeLTRB(
            srcBorder.left(),
            srcSampleBounds.top(),
            srcBorder.left() + 1.0,
            srcSampleBounds.bottom(),
        ),
        SkRect::MakeLTRB(
            dstBorder.left(),
            dstSampleBounds.top(),
            dstBorder.left() + 1.0,
            dstSampleBounds.bottom(),
        ),
    )?;
    drawEdge(
        SkRect::MakeLTRB(
            srcBorder.right() - 1.0,
            srcSampleBounds.top(),
            srcBorder.right(),
            srcSampleBounds.bottom(),
        ),
        SkRect::MakeLTRB(
            dstBorder.right() - 1.0,
            dstSampleBounds.top(),
            dstBorder.right(),
            dstSampleBounds.bottom(),
        ),
    )?;
    drawEdge(
        SkRect::MakeLTRB(
            srcSampleBounds.left(),
            srcBorder.top(),
            srcSampleBounds.right(),
            srcBorder.top() + 1.0,
        ),
        SkRect::MakeLTRB(
            dstSampleBounds.left(),
            dstBorder.top(),
            dstSampleBounds.right(),
            dstBorder.top() + 1.0,
        ),
    )?;
    drawEdge(
        SkRect::MakeLTRB(
            srcSampleBounds.left(),
            srcBorder.bottom() - 1.0,
            srcSampleBounds.right(),
            srcBorder.bottom(),
        ),
        SkRect::MakeLTRB(
            dstSampleBounds.left(),
            dstBorder.bottom() - 1.0,
            dstSampleBounds.right(),
            dstBorder.bottom(),
        ),
    )?;
    drawCorner(
        SkPoint::make(srcBorder.left(), srcBorder.top()),
        SkPoint::make(dstBorder.left(), dstBorder.top()),
    )?;
    drawCorner(
        SkPoint::make(srcBorder.right() - 1.0, srcBorder.top()),
        SkPoint::make(dstBorder.right() - 1.0, dstBorder.top()),
    )?;
    drawCorner(
        SkPoint::make(srcBorder.right() - 1.0, srcBorder.bottom() - 1.0),
        SkPoint::make(dstBorder.right() - 1.0, dstBorder.bottom() - 1.0),
    )?;
    drawCorner(
        SkPoint::make(srcBorder.left(), srcBorder.bottom() - 1.0),
        SkPoint::make(dstBorder.left(), dstBorder.bottom() - 1.0),
    )?;
    Ok(())
}

impl FilterResult {
    // Source: src/core/SkImageFilterTypes.cpp:1617-1896
    /// Safety: preserve source immutable backing and all backend/offscreen
    /// Canvas pixel leases throughout every progressive resampling pass.
    // Source: src/core/SkImageFilterTypes.h:940-943
    pub unsafe fn rescale(
        &self,
        ctx: &Context<'_>,
        scale: LayerSpace<SkSize>,
        enforceDecal: bool,
        mut allowOverscaling: bool,
    ) -> Result<Self, crate::RasterError> {
        use crate::include::core::SkTileMode::SkTileMode::*;
        use crate::src::core::SkCanvas::SkCanvas;
        let mut visible = self.fLayerBounds;
        if self.fImage.is_none()
            || !visible.intersect(ctx.desiredOutput())
            || scale.width() <= 0.0
            || scale.height() <= 0.0
        {
            return Ok(Self::default());
        }
        let mut origin = LayerSpace::new(SkIPoint::make(0, 0));
        let pixelAligned = is_nearly_integer_translation(&self.fTransform, Some(&mut origin));
        let mut analysis = self.analyzeLayerBounds(ctx.desiredOutput(), BoundsScope::kRescale)?;
        let canDefer = pixelAligned
            && !analysis.contains(BoundsAnalysis::kRequiresLayerCrop)
            && !(enforceDecal && analysis.contains(BoundsAnalysis::kHasLayerFillingEffect));
        let source = self.fImage.as_ref().unwrap();
        let srcCS = source
            .colorInfo()
            .colorSpace()
            .unwrap_or_else(|| super::SkColorSpace::sk_srgb_singleton().as_ref());
        let dstCS = ctx.colorSpace().unwrap_or(srcCS);
        let hasEffects = !canDefer
            || self.fColorFilter.is_some()
            || source.colorInfo().colorType() != ctx.backend().colorType()
            || !super::SkColorSpace::SkColorSpace::Equals(Some(srcCS), Some(dstCS));
        let (mut xs, mut ys) = (
            downscale_step_count(scale.width()),
            downscale_step_count(scale.height()),
        );
        if xs == 0 && ys == 0 && !hasEffects {
            if analysis.contains(BoundsAnalysis::kHasLayerFillingEffect) {
                let mut noop = self.clone();
                noop.fLayerBounds = visible;
                return Ok(noop);
            }
            return Ok(self.subset(origin, visible, false));
        }
        let mut srcRect;
        let tileMode;
        let mut cfBorder = false;
        let mut deferPeriodic = false;
        if canDefer && analysis.contains(BoundsAnalysis::kHasLayerFillingEffect) {
            srcRect = LayerSpace::new(SkIRect::make_xywh(
                origin.x(),
                origin.y(),
                source.width(),
                source.height(),
            ));
            if self.fTileMode == kDecal && analysis.contains(BoundsAnalysis::kHasLayerFillingEffect)
            {
                tileMode = kClamp;
                cfBorder = true;
            } else {
                tileMode = self.fTileMode;
                deferPeriodic = tileMode == kRepeat || tileMode == kMirror;
            }
        } else {
            srcRect = visible;
            tileMode = kDecal;
        }
        srcRect = srcRect.relevantSubset(*ctx.desiredOutput(), tileMode);
        let mut stepBounds = LayerSpace::new(SkRect::Make(srcRect.geometry()));
        if stepBounds.isEmpty() {
            return Ok(Self::default());
        }
        let mut stepPixels = stepBounds;
        let mut image = self.clone();
        if !pixelAligned && (xs > 0 || ys > 0) {
            let net = image.fTransform.mapSize(&scale);
            let nextX = if net.width().is_finite() {
                downscale_step_count(net.width())
            } else {
                i32::MAX
            };
            let nextY = if net.height().is_finite() {
                downscale_step_count(net.height())
            } else {
                i32::MAX
            };
            if (xs > 0 && nextX > xs) || (ys > 0 && nextY > ys) {
                image = image.resolve(ctx, srcRect, false)?;
                if !image.has_image() {
                    return Ok(Self::default());
                }
                if !cfBorder {
                    image.fTileMode = tileMode;
                }
            }
        }
        if deferPeriodic {
            image.fTileMode = kClamp;
        } else {
            allowOverscaling = false;
        }
        let finalX = if xs > 0 {
            if allowOverscaling {
                1.0 / ((1i32 << xs) as f32)
            } else {
                scale.width()
            }
        } else {
            1.0
        };
        let finalY = if ys > 0 {
            if allowOverscaling {
                1.0 / ((1i32 << ys) as f32)
            } else {
                scale.height()
            }
        } else {
            1.0
        };
        loop {
            let mut sx = 1.0;
            if xs > 0 {
                sx = if xs > 1 {
                    0.5
                } else {
                    srcRect.width() as f32 * finalX / stepBounds.width()
                };
                xs -= 1;
            }
            let mut sy = 1.0;
            if ys > 0 {
                sy = if ys > 1 {
                    0.5
                } else {
                    srcRect.height() as f32 * finalY / stepBounds.height()
                };
                ys -= 1;
            }
            let mut dst = scale_about_center(stepBounds, sx, sy);
            let finalXStep = xs == 0 && sx != 1.0;
            let finalYStep = ys == 0 && sy != 1.0;
            if deferPeriodic && (finalXStep || finalYStep) {
                let pixels = dst.roundOut();
                dst = LayerSpace::new(SkRect::MakeLTRB(
                    if finalXStep {
                        pixels.left() as f32
                    } else {
                        dst.left()
                    },
                    if finalYStep {
                        pixels.top() as f32
                    } else {
                        dst.top()
                    },
                    if finalXStep {
                        pixels.right() as f32
                    } else {
                        dst.right()
                    },
                    if finalYStep {
                        pixels.bottom() as f32
                    } else {
                        dst.bottom()
                    },
                ));
            }
            let mut pixels = dst.roundOut();
            let samples = pixels;
            let boundary = if tileMode == kDecal {
                PixelBoundary::kTransparent
            } else {
                pixels.outset(LayerSpace::new(SkISize::Make(1, 1)));
                PixelBoundary::kUnknown
            };
            let mut surface = AutoSurface::new(ctx, pixels, boundary, false, None)?;
            if !surface.is_valid() {
                return Ok(Self::default());
            }
            let xform = LayerSpace::<SkMatrix>::RectToRect(&stepBounds, &dst);
            analysis =
                image.analyzeBounds(xform.geometry(), samples.geometry(), BoundsScope::kRescale)?;
            let mut paint = super::SkPaint::SkPaint::default();
            paint.setShader(image.getAnalyzedShaderView(ctx, image.sampling(), analysis)?);
            paint.setBlendMode(super::SkBlendMode::SkBlendMode::kSrc);
            let mut srcSampled = LayerSpace::<SkRect>::Empty();
            let valid = xform.inverseMapRect(
                &LayerSpace::new(SkRect::Make(samples.geometry())),
                &mut srcSampled,
            );
            crate::SkAssertResult!(@source b"src/core/SkImageFilterTypes.cpp\0".as_ptr(),1836; @condition b"scaleXform.inverseMapRect(PixelSpace<SkRect>(sampleBounds), &srcSampled)\0".as_ptr(); valid);
            let canvas = surface.canvas();
            core::pin::Pin::new_unchecked(&mut *canvas.as_ptr()).save();
            core::pin::Pin::new_unchecked(&mut *canvas.as_ptr()).concat(xform.geometry());
            let drawn = SkCanvas::drawRect_raw(canvas, srcSampled.geometry(), &paint);
            let restored = SkCanvas::restore_raw(canvas);
            drawn?;
            restored?;
            if cfBorder {
                crate::SkASSERT!(@source b"src/core/SkImageFilterTypes.cpp\0".as_ptr(),1848; @condition b"fColorFilter && as_CFB(fColorFilter)->affectsTransparentBlack()\0".as_ptr(); self.fColorFilter.as_ref().unwrap().affectsTransparentBlack()?);
                crate::SkASSERT!(@source b"src/core/SkImageFilterTypes.cpp\0".as_ptr(),1849; @condition b"tileMode == SkTileMode::kClamp\0".as_ptr(); tileMode==kClamp);
                draw_color_filtered_border(surface.canvas(), pixels, self.fColorFilter.clone())?;
                cfBorder = false;
            } else if tileMode != kDecal {
                draw_tiled_border(
                    surface.canvas(),
                    tileMode,
                    &paint,
                    &xform,
                    stepPixels,
                    LayerSpace::new(SkRect::Make(pixels.geometry())),
                )?;
            }
            image = surface.snap()?;
            image.fTileMode = if deferPeriodic { kClamp } else { tileMode };
            stepBounds = dst;
            stepPixels = LayerSpace::new(SkRect::Make(pixels.geometry()));
            if xs <= 0 && ys <= 0 {
                break;
            }
        }
        if deferPeriodic {
            image = image.insetByPixel();
        } else {
            crate::SkASSERT!(@source b"src/core/SkImageFilterTypes.cpp\0".as_ptr(),1881; @condition b"tileMode == SkTileMode::kDecal || tileMode == SkTileMode::kClamp\0".as_ptr(); tileMode==kDecal||tileMode==kClamp);
        }
        image.fTileMode = tileMode;
        image
            .fTransform
            .postConcat(&LayerSpace::<SkMatrix>::RectToRect(
                &stepBounds,
                &LayerSpace::new(SkRect::Make(srcRect.geometry())),
            ));
        image.fLayerBounds = visible;
        crate::SkASSERT!(@source b"src/core/SkImageFilterTypes.cpp\0".as_ptr(),1892; @condition b"!enforceDecal || image.fTileMode == SkTileMode::kDecal\0".as_ptr(); !enforceDecal||image.fTileMode==kDecal);
        crate::SkASSERT!(@source b"src/core/SkImageFilterTypes.cpp\0".as_ptr(),1893; @condition b"image.fTileMode != SkTileMode::kDecal || image.fBoundary == PixelBoundary::kTransparent\0".as_ptr(); image.fTileMode!=kDecal||image.fBoundary==PixelBoundary::kTransparent);
        crate::SkASSERT!(@source b"src/core/SkImageFilterTypes.cpp\0".as_ptr(),1895; @condition b"!deferPeriodicTiling || image.fBoundary == PixelBoundary::kInitialized\0".as_ptr(); !deferPeriodic||image.fBoundary==PixelBoundary::kInitialized);
        Ok(image)
    }
}

impl FilterResult {
    // Source: src/core/SkImageFilterTypes.cpp:1899-1924
    /// Safety: preserve the real offscreen Canvas/pixel lease through picture
    /// playback and snapshot; pic retains the original intrusive recording owner.
    pub unsafe fn MakeFromPicture(
        ctx: &Context<'_>,
        pic: crate::include::core::SkRefCnt::sk_sp<dyn crate::src::core::SkPicture::SkPicture>,
        cullRect: ParameterSpace<SkRect>,
    ) -> Result<Self, crate::RasterError> {
        crate::SkASSERT!(@source b"src/core/SkImageFilterTypes.cpp\0".as_ptr(),1902; @condition b"pic\0".as_ptr(); pic.is_some());
        let layerRect: LayerSpace<SkRect> = ctx.mapping().paramToLayer(&cullRect);
        let mut dstBounds = layerRect.roundOut();
        if !dstBounds.intersect(ctx.desiredOutput()) {
            return Ok(Self::default());
        }
        let props = ctx.backend().surfaceProps().cloneWithPixelGeometry(
            crate::include::core::SkSurfaceProps::SkPixelGeometry::kUnknown_SkPixelGeometry,
        );
        let mut surface =
            AutoSurface::new(ctx, dstBounds, PixelBoundary::kUnknown, true, Some(&props))?;
        if surface.is_valid() {
            // clipRect performs no COW callback. End its pinned projection before
            // the following raw draw can reenter the original complete Canvas.
            let raw = surface.canvas();
            core::pin::Pin::new_unchecked(&mut *raw.as_ptr())
                .clipRect_intersect(cullRect.geometry(), false);
            crate::src::core::SkCanvas::SkCanvas::drawPicture_sp_default_raw(
                surface.canvas(),
                &pic,
            )?;
        }
        surface.snap()
    }
}

impl FilterResult {
    // Source: src/core/SkImageFilterTypes.cpp:1926-1946
    /// Safety: retain shader/context backing for the complete synchronous draw
    /// and satisfy the Backend's actual offscreen Canvas pixel/COW contracts.
    // Source: src/core/SkImageFilterTypes.h:735-737
    pub unsafe fn MakeFromShader(
        ctx: &Context<'_>,
        shader: std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>,
        dither: bool,
    ) -> Result<Self, crate::RasterError> {
        let boundary = if dither {
            PixelBoundary::kUnknown
        } else {
            PixelBoundary::kTransparent
        };
        let mut surface =
            unsafe { AutoSurface::new(ctx, *ctx.desiredOutput(), boundary, true, None) }?;
        if surface.is_valid() {
            let mut paint = crate::src::core::SkPaint::SkPaint::default();
            paint.setShader(Some(shader));
            paint.setDither(dither);
            paint.setBlendMode(crate::src::core::SkBlendMode::SkBlendMode::kSrc);
            unsafe {
                crate::src::core::SkCanvas::SkCanvas::drawPaint_raw(surface.canvas(), &paint)
            }?;
        }
        unsafe { surface.snap() }
    }
}

// Rust propagation adaptation of h393-408 Union's same ascending-child loop.
// This helper adds no source credit and never converts a child error to bounds.
impl LayerSpace<SkIRect> {
    pub(crate) fn TryUnion(
        count: i32,
        mut bounds: impl FnMut(i32) -> Result<Self, crate::RasterError>,
    ) -> Result<Self, crate::RasterError> {
        if count <= 0 {
            return Ok(Self::Empty());
        }
        let mut out = bounds(0)?;
        for i in 1..count {
            out.join(&bounds(i)?);
        }
        Ok(out)
    }
}

impl FilterResult {
    // Source: src/core/SkImageFilterTypes.cpp:1948-2002
    /// Safety: source and every offscreen draw obey the real immutable-image,
    /// pixel bounds, Canvas callback and snapshot ownership leases.
    // Source: src/core/SkImageFilterTypes.h:742-746
    pub unsafe fn MakeFromImage(
        ctx: &Context<'_>,
        image: std::sync::Arc<dyn crate::src::image::SkImage::SkImage>,
        mut srcRect: SkRect,
        mut dstRect: ParameterSpace<SkRect>,
        sampling: crate::include::core::SkSamplingOptions::SkSamplingOptions,
    ) -> Result<Self, crate::RasterError> {
        let imageBounds = SkRect::MakeISize(image.dimensions());
        if !imageBounds.contains(&srcRect) {
            let srcToDst = SkMatrix::RectToRectOrIdentity(
                &srcRect,
                dstRect.geometry(),
                crate::src::core::SkMatrix::ScaleToFit::kFill_ScaleToFit,
            );
            if !srcRect.intersect(&imageBounds) {
                return Ok(Self::default());
            }
            let mut mapped = SkRect::MakeEmpty();
            srcToDst.mapRect(&mut mapped, &srcRect);
            dstRect = ParameterSpace::new(mapped);
        }
        if dstRect.geometry().isEmpty() {
            return Ok(Self::default());
        }
        let srcSubset = RoundOut(srcRect);
        if SkRect::Make(&srcSubset) == srcRect {
            let special = ctx.backend().makeImage(&srcSubset, Some(image));
            let subset = Self::new(special, LayerSpace::new(srcSubset.top_left()));
            let transform = *ctx.mapping().layerMatrix()
                * crate::src::core::SkM44::SkM44::RectToRect(&srcRect, dstRect.geometry());
            return subset.applyTransform(ctx, LayerSpace::new(transform.asM33()), sampling);
        }
        let mut dstBounds = ctx.mapping().paramToLayer(&dstRect).roundOut();
        if !dstBounds.intersect(ctx.desiredOutput()) {
            return Ok(Self::default());
        }
        let mut surface =
            AutoSurface::new(ctx, dstBounds, PixelBoundary::kTransparent, true, None)?;
        if surface.is_valid() {
            let mut paint = crate::src::core::SkPaint::SkPaint::default();
            paint.setAntiAlias(true);
            crate::src::core::SkCanvas::SkCanvas::drawImageRect_raw(
                surface.canvas(),
                &image,
                Some(&srcRect),
                dstRect.geometry(),
                &sampling,
                Some(&paint),
                crate::src::core::SkCanvas::SrcRectConstraint::kStrict_SrcRectConstraint,
            )?;
        }
        surface.snap()
    }
}

// Source: src/core/SkImageFilterTypes.h:1077-1082
struct SampledFilterResult {
    fImage: FilterResult,
    fSampleBounds: Option<LayerSpace<SkIRect>>,
    fFlags: ShaderFlags,
    fSampling: crate::include::core::SkSamplingOptions::SkSamplingOptions,
}
// Source: src/core/SkImageFilterTypes.h:1092-1096
// Vec substitutes STArray's owned sequence; Context remains a true borrow.
// Reverse field teardown drops generated shaders before sampled inputs.
pub struct Builder<'ctx, 'stats> {
    fInputShaders: Vec<Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>>>,
    fInputs: Vec<SampledFilterResult>,
    fContext: &'ctx Context<'stats>,
}
impl<'ctx, 'stats> Builder<'ctx, 'stats> {
    // Source: src/core/SkImageFilterTypes.cpp:2007-2008
    // Source: src/core/SkImageFilterTypes.h:1012-1012
    pub fn new(context: &'ctx Context<'stats>) -> Self {
        Self {
            fContext: context,
            fInputs: Vec::new(),
            fInputShaders: Vec::new(),
        }
    }
    // Source: src/core/SkImageFilterTypes.h:1027-1035
    pub fn add(&mut self, input: FilterResult) -> &mut Self {
        self.add_sampled(
            input,
            None,
            ShaderFlags::kNone,
            FilterResult::kDefaultSampling,
        )
    }
    pub fn add_sampled(
        &mut self,
        input: FilterResult,
        sampleBounds: Option<LayerSpace<SkIRect>>,
        flags: ShaderFlags,
        sampling: crate::include::core::SkSamplingOptions::SkSamplingOptions,
    ) -> &mut Self {
        self.fInputs.push(SampledFilterResult {
            fImage: input,
            fSampleBounds: sampleBounds,
            fFlags: flags,
            fSampling: sampling,
        });
        self
    }
    // Source: src/core/SkImageFilterTypes.h:1060-1073
    pub unsafe fn eval(
        &mut self,
        shaderFn: impl FnOnce(
            &[Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>>],
        ) -> Result<
            Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>>,
            crate::RasterError,
        >,
        explicitOutput: Option<LayerSpace<SkIRect>>,
    ) -> Result<FilterResult, crate::RasterError> {
        self.eval_in_space(shaderFn, explicitOutput, false)
    }
    pub unsafe fn eval_in_space(
        &mut self,
        shaderFn: impl FnOnce(
            &[Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>>],
        ) -> Result<
            Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>>,
            crate::RasterError,
        >,
        explicitOutput: Option<LayerSpace<SkIRect>>,
        evaluateInParameterSpace: bool,
    ) -> Result<FilterResult, crate::RasterError> {
        let bounds = self.outputBounds(explicitOutput);
        if bounds.isEmpty() {
            return Ok(FilterResult::default());
        }
        let shaders = self.createInputShaders(bounds, evaluateInParameterSpace)?;
        let shader = shaderFn(shaders)?;
        self.drawShader(shader, bounds, evaluateInParameterSpace)
    }
    // Source: src/core/SkImageFilterTypes.cpp:2010-2043
    // Source: src/core/SkImageFilterTypes.h:1084-1085
    unsafe fn createInputShaders(
        &mut self,
        bounds: LayerSpace<SkIRect>,
        parameterSpace: bool,
    ) -> Result<
        &[Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>>],
        crate::RasterError,
    > {
        let mut flags = ShaderFlags::kNone;
        let mut layerToParam = SkMatrix::default();
        if parameterSpace {
            layerToParam = self
                .fContext
                .mapping()
                .layerMatrix()
                .asM33()
                .invert()
                .unwrap_or_default();
            if !is_nearly_integer_translation(&LayerSpace::new(layerToParam.clone()), None) {
                flags = flags | ShaderFlags::kNonTrivialSampling;
            }
        }
        self.fInputShaders.reserve(self.fInputs.len());
        for input in &self.fInputs {
            let mut shader = input.fImage.asShader(
                self.fContext,
                input.fSampling,
                input.fFlags | flags,
                input.fSampleBounds.unwrap_or(bounds),
            )?;
            if parameterSpace {
                if let Some(s) = shader.as_ref() {
                    shader = Some(s.makeWithLocalMatrix(&layerToParam));
                }
            }
            self.fInputShaders.push(shader);
        }
        Ok(&self.fInputShaders)
    }
    // Source: src/core/SkImageFilterTypes.cpp:2045-2056
    // Source: src/core/SkImageFilterTypes.h:1087-1087
    fn outputBounds(&self, explicit: Option<LayerSpace<SkIRect>>) -> LayerSpace<SkIRect> {
        let mut output = *self.fContext.desiredOutput();
        if let Some(explicit) = explicit {
            if !output.intersect(&explicit) {
                return LayerSpace::<SkIRect>::Empty();
            }
        }
        output
    }
    // Source: src/core/SkImageFilterTypes.cpp:2058-2077
    // Source: src/core/SkImageFilterTypes.h:1089-1091
    unsafe fn drawShader(
        &self,
        shader: Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>>,
        bounds: LayerSpace<SkIRect>,
        parameterSpace: bool,
    ) -> Result<FilterResult, crate::RasterError> {
        crate::SkASSERT!(@source b"src/core/SkImageFilterTypes.cpp\0".as_ptr(),2061; @condition b"!outputBounds.isEmpty()\0".as_ptr(); !bounds.isEmpty());
        if shader.is_none() {
            return Ok(FilterResult::default());
        }
        let mut surface = AutoSurface::new(
            self.fContext,
            bounds,
            PixelBoundary::kTransparent,
            parameterSpace,
            None,
        )?;
        if surface.is_valid() {
            let mut paint = crate::src::core::SkPaint::SkPaint::default();
            paint.setShader(shader);
            paint.setBlendMode(crate::src::core::SkBlendMode::SkBlendMode::kSrc);
            crate::src::core::SkCanvas::SkCanvas::drawPaint_raw(surface.canvas(), &paint)?;
        }
        surface.snap()
    }
    // Source: src/core/SkImageFilterTypes.cpp:2079-2106
    // Source: src/core/SkImageFilterTypes.h:1036-1036
    pub unsafe fn merge(&self) -> Result<FilterResult, crate::RasterError> {
        crate::SkASSERT!(@source b"src/core/SkImageFilterTypes.cpp\0".as_ptr(),2082; @condition b"!fInputs.empty()\0".as_ptr(); !self.fInputs.is_empty());
        if self.fInputs.len() == 1 {
            crate::SkASSERT!(@source b"src/core/SkImageFilterTypes.cpp\0".as_ptr(),2084; @condition b"!fInputs[0].fSampleBounds.has_value() && fInputs[0].fSampling == kDefaultSampling && fInputs[0].fFlags == ShaderFlags::kNone\0".as_ptr(); self.fInputs[0].fSampleBounds.is_none()&&self.fInputs[0].fSampling==FilterResult::kDefaultSampling&&self.fInputs[0].fFlags==ShaderFlags::kNone);
            return Ok(self.fInputs[0].fImage.clone());
        }
        let merged = LayerSpace::<SkIRect>::Union(self.fInputs.len() as i32, |i| {
            self.fInputs[i as usize].fImage.layerBounds()
        });
        let bounds = self.outputBounds(Some(merged));
        let mut surface = AutoSurface::new(
            self.fContext,
            bounds,
            PixelBoundary::kTransparent,
            false,
            None,
        )?;
        if surface.is_valid() {
            for input in &self.fInputs {
                crate::SkASSERT!(@source b"src/core/SkImageFilterTypes.cpp\0".as_ptr(),2099; @condition b"!input.fSampleBounds.has_value() && input.fSampling == kDefaultSampling && input.fFlags == ShaderFlags::kNone\0".as_ptr(); input.fSampleBounds.is_none()&&input.fSampling==FilterResult::kDefaultSampling&&input.fFlags==ShaderFlags::kNone);
                input
                    .fImage
                    .draw_internal(self.fContext, surface.device(), true, None)?;
            }
        }
        surface.snap()
    }
    // Source: src/core/SkImageFilterTypes.cpp:2108-2246
    // Source: src/core/SkImageFilterTypes.h:1042-1042
    pub unsafe fn blur(
        &self,
        sigma: LayerSpace<SkSize>,
    ) -> Result<FilterResult, crate::RasterError> {
        use crate::include::core::SkTileMode::SkTileMode::*;
        crate::SkASSERT!(@source b"src/core/SkImageFilterTypes.cpp\0".as_ptr(),2109; @condition b"fInputs.size() == 1\0".as_ptr(); self.fInputs.len()==1);
        let engine = self.fContext.backend().getBlurEngine();
        let Some(algorithm) =
            engine.findAlgorithm(*sigma.geometry(), self.fContext.backend().colorType())
        else {
            return Ok(FilterResult::default());
        };
        let radii = LayerSpace::new(SkSize::Make(3.0 * sigma.width(), 3.0 * sigma.height())).ceil();
        let mut maxOutput = self.fInputs[0].fImage.layerBounds();
        maxOutput.outset(radii);
        let mut output = self.outputBounds(Some(maxOutput));
        if output.isEmpty() {
            return Ok(FilterResult::default());
        }
        let mut samples = output;
        samples.outset(radii);
        let sx = if sigma.width() > algorithm.maxSigma() {
            algorithm.maxSigma() / sigma.width()
        } else {
            1.0
        };
        let sy = if sigma.height() > algorithm.maxSigma() {
            algorithm.maxSigma() / sigma.height()
        } else {
            1.0
        };
        let low = self.fInputs[0].fImage.rescale(
            &self.fContext.withNewDesiredOutput(samples),
            LayerSpace::new(SkSize::Make(sx, sy)),
            algorithm.supportsOnlyDecalTiling(),
            true,
        )?;
        if !low.has_image() {
            return Ok(FilterResult::default());
        }
        crate::SkASSERT!(@source b"src/core/SkImageFilterTypes.cpp\0".as_ptr(),2153; @condition b"lowResImage.tileMode() == SkTileMode::kDecal || !algorithm->supportsOnlyDecalTiling()\0".as_ptr(); low.tileMode()==kDecal || !algorithm.supportsOnlyDecalTiling());
        let clamp_sigma = |v: f32| {
            if algorithm.maxSigma() < v {
                algorithm.maxSigma()
            } else {
                v
            }
        };
        let lowSigma = LayerSpace::new(SkSize::Make(
            clamp_sigma(sigma.width() * (1.0 / low.fTransform.rc(0, 0))),
            clamp_sigma(sigma.height() * (1.0 / low.fTransform.rc(1, 1))),
        ));
        let mut lowMax =
            LayerSpace::<SkIRect>::from_size(low.fImage.as_ref().unwrap().dimensions());
        let mut relative = LayerSpace::<SkIRect>::Empty();
        if low.tileMode() == kRepeat || low.tileMode() == kMirror {
            relative = lowMax;
        } else {
            let valid = low.fTransform.inverseMapIRect(&output, &mut relative);
            crate::SkAssertResult!(@source b"src/core/SkImageFilterTypes.cpp\0".as_ptr(),2176; @condition b"lowResImage.fTransform.inverseMapRect(outputBounds, &srcRelativeOutput)\0".as_ptr(); valid);
            lowMax.outset(
                LayerSpace::new(SkSize::Make(
                    3.0 * lowSigma.width(),
                    3.0 * lowSigma.height(),
                ))
                .ceil(),
            );
            relative = lowMax.relevantSubset(relative, low.tileMode());
            if relative.isEmpty() {
                return Ok(FilterResult::default());
            }
            relative.outset(LayerSpace::new(SkISize::Make(1, 1)));
        }
        let mut image = low.refImage().unwrap();
        let mut blurBounds = *relative.geometry();
        let mut tile = low.tileMode();
        if !algorithm.supportsOnlyDecalTiling()
            && low.canClampToTransparentBoundary(BoundsAnalysis::kSimple)
        {
            image = image.makePixelOutset().expect("native known pixel outset");
            blurBounds.offset(1, 1);
            tile = kClamp;
        }
        let bounds = SkIRect::make_size(image.dimensions());
        let image = algorithm.blur(*lowSigma.geometry(), image, &bounds, tile, &blurBounds)?;
        if image.is_none() {
            return Ok(FilterResult::default());
        }
        let mut result = FilterResult::new(image, relative.topLeft());
        if low.tileMode() == kClamp || low.tileMode() == kDecal {
            result = result.insetByPixel();
        }
        result.fTransform.postConcat(&low.fTransform);
        if low.tileMode() == kDecal {
            output = self.outputBounds(Some(result.fTransform.mapIRect(&LayerSpace::from_size(
                result.fImage.as_ref().unwrap().dimensions(),
            ))));
        }
        result.fLayerBounds = output;
        result.fTileMode = low.tileMode();
        Ok(result)
    }
}
