use crate::{
    gfx, HeapVector, Length, MakeGarbageCollected, Member, Traceable, TransformOperationType,
    Visitor, WtfSizeT,
};

// cpp: foundation/blink_geometry/transforms/transform_operation.h:37-123
// The concrete operations belong to the remaining blink_geometry transform
// translation. This virtual interface preserves the operation-kind and deep
// comparison contract used by computed style without inventing an operation.
#[allow(non_snake_case)]
pub trait TransformOperation {
    fn GetType(&self) -> TransformOperationType;
    fn IsEqualAssumingSameType(&self, other: &dyn TransformOperation) -> bool;
    fn Apply(&self, transform: &mut gfx::Transform, border_box_size: &gfx::SizeF);

    fn IsSameType(&self, other: &dyn TransformOperation) -> bool {
        self.GetType() == other.GetType()
    }

    fn Is3DOperation(&self) -> bool {
        use TransformOperationType::*;
        matches!(
            self.GetType(),
            kScaleZ
                | kScale3D
                | kTranslateZ
                | kTranslate3D
                | kRotateX
                | kRotateY
                | kRotate3D
                | kMatrix3D
                | kPerspective
                | kInterpolated
        )
    }

    fn PreservesAxisAlignment(&self) -> bool {
        false
    }

    fn IsIdentityOrTranslation(&self) -> bool {
        false
    }

    fn HasNonTrivial3DComponent(&self) -> bool {
        self.Is3DOperation()
    }
}

// cpp: foundation/blink_geometry/transforms/matrix_3d_transform_operation.h:38-93
pub struct Matrix3DTransformOperation {
    matrix_: gfx::Transform,
}

impl Matrix3DTransformOperation {
    pub fn new(matrix: gfx::Transform) -> Self {
        Self { matrix_: matrix }
    }

    pub fn Matrix(&self) -> gfx::Transform {
        self.matrix_.clone()
    }
}

impl Traceable for Matrix3DTransformOperation {
    fn Trace(&self, _visitor: &mut Visitor<'_>) {}
}

impl TransformOperation for Matrix3DTransformOperation {
    fn GetType(&self) -> TransformOperationType {
        TransformOperationType::kMatrix3D
    }

    fn IsEqualAssumingSameType(&self, other: &dyn TransformOperation) -> bool {
        let other = unsafe { &*(other as *const dyn TransformOperation as *const Self) };
        self.matrix_ == other.matrix_
    }

    fn Apply(&self, transform: &mut gfx::Transform, _size: &gfx::SizeF) {
        transform.PreConcat(&self.matrix_);
    }

    fn IsIdentityOrTranslation(&self) -> bool {
        self.matrix_.IsIdentityOrTranslation()
    }

    fn PreservesAxisAlignment(&self) -> bool {
        self.matrix_.Preserves2dAxisAlignment()
    }
}

// cpp: third_party/blink/renderer/platform/transforms/matrix_transform_operation.h:34-65,76-85.
pub struct MatrixTransformOperation {
    matrix_: gfx::Transform,
}
impl MatrixTransformOperation {
    pub fn new(a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> Self {
        Self {
            matrix_: gfx::Transform::ColMajor(&[
                a, b, 0.0, 0.0, c, d, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, e, f, 0.0, 1.0,
            ]),
        }
    }
    pub fn Matrix(&self) -> &gfx::Transform {
        &self.matrix_
    }
}
impl Traceable for MatrixTransformOperation {
    fn Trace(&self, _: &mut Visitor<'_>) {}
}
impl TransformOperation for MatrixTransformOperation {
    fn GetType(&self) -> TransformOperationType {
        TransformOperationType::kMatrix
    }
    fn IsEqualAssumingSameType(&self, other: &dyn TransformOperation) -> bool {
        let other = unsafe { &*(other as *const dyn TransformOperation as *const Self) };
        self.matrix_ == other.matrix_
    }
    fn Apply(&self, transform: &mut gfx::Transform, _: &gfx::SizeF) {
        transform.PreConcat(&self.matrix_);
    }
    fn IsIdentityOrTranslation(&self) -> bool {
        self.matrix_.IsIdentityOrTranslation()
    }
    fn PreservesAxisAlignment(&self) -> bool {
        self.matrix_.Preserves2dAxisAlignment()
    }
}
// cpp: third_party/blink/renderer/platform/transforms/skew_transform_operation.h:33-57,67-69.
pub struct SkewTransformOperation {
    angle_x_: f64,
    angle_y_: f64,
    type_: TransformOperationType,
}
impl SkewTransformOperation {
    pub fn new(angle_x: f64, angle_y: f64, type_: TransformOperationType) -> Self {
        Self {
            angle_x_: angle_x,
            angle_y_: angle_y,
            type_,
        }
    }
}
impl Traceable for SkewTransformOperation {
    fn Trace(&self, _: &mut Visitor<'_>) {}
}
impl TransformOperation for SkewTransformOperation {
    fn GetType(&self) -> TransformOperationType {
        self.type_
    }
    fn IsEqualAssumingSameType(&self, other: &dyn TransformOperation) -> bool {
        let other = unsafe { &*(other as *const dyn TransformOperation as *const Self) };
        self.angle_x_ == other.angle_x_ && self.angle_y_ == other.angle_y_
    }
    fn Apply(&self, transform: &mut gfx::Transform, _: &gfx::SizeF) {
        transform.Skew(self.angle_x_, self.angle_y_);
    }
}
// cpp: third_party/blink/renderer/platform/transforms/perspective_transform_operation.h:37-74,84-89.
pub struct PerspectiveTransformOperation {
    p_: Option<f64>,
}
impl PerspectiveTransformOperation {
    pub fn new(p: Option<f64>) -> Self {
        Self { p_: p }
    }
    pub fn Perspective(&self) -> Option<f64> {
        self.p_
    }
}
impl Traceable for PerspectiveTransformOperation {
    fn Trace(&self, _: &mut Visitor<'_>) {}
}
impl TransformOperation for PerspectiveTransformOperation {
    fn GetType(&self) -> TransformOperationType {
        TransformOperationType::kPerspective
    }
    fn IsEqualAssumingSameType(&self, other: &dyn TransformOperation) -> bool {
        let other = unsafe { &*(other as *const dyn TransformOperation as *const Self) };
        self.p_ == other.p_
    }
    fn Apply(&self, transform: &mut gfx::Transform, _: &gfx::SizeF) {
        if let Some(p) = self.p_ {
            transform.ApplyPerspectiveDepth(p.max(1.0));
        }
    }
    fn HasNonTrivial3DComponent(&self) -> bool {
        false
    }
}

// cpp: foundation/blink_geometry/transforms/translate_transform_operation.h:36-124
pub struct TranslateTransformOperation {
    x_: Length,
    y_: Length,
    z_: f64,
    type_: TransformOperationType,
}

impl TranslateTransformOperation {
    pub fn new(x: Length, y: Length, z: f64, type_: TransformOperationType) -> Self {
        assert!(matches!(
            type_,
            TransformOperationType::kTranslate
                | TransformOperationType::kTranslateX
                | TransformOperationType::kTranslateY
                | TransformOperationType::kTranslateZ
                | TransformOperationType::kTranslate3D
        ));
        Self {
            x_: x,
            y_: y,
            z_: z,
            type_: type_,
        }
    }

    pub fn X(&self, size: &gfx::SizeF) -> f64 {
        crate::length_functions::FloatValueForLength(&self.x_, size.width()) as f64
    }

    pub fn Y(&self, size: &gfx::SizeF) -> f64 {
        crate::length_functions::FloatValueForLength(&self.y_, size.height()) as f64
    }

    pub fn Z(&self) -> f64 {
        self.z_
    }

    pub fn Apply(&self, transform: &mut gfx::Transform, size: &gfx::SizeF) {
        <Self as TransformOperation>::Apply(self, transform, size);
    }
}

impl Traceable for TranslateTransformOperation {
    fn Trace(&self, _visitor: &mut Visitor<'_>) {}
}

impl TransformOperation for TranslateTransformOperation {
    fn GetType(&self) -> TransformOperationType {
        self.type_
    }

    fn IsEqualAssumingSameType(&self, other: &dyn TransformOperation) -> bool {
        let other = unsafe { &*(other as *const dyn TransformOperation as *const Self) };
        self.x_ == other.x_ && self.y_ == other.y_ && self.z_ == other.z_
    }

    fn Apply(&self, transform: &mut gfx::Transform, size: &gfx::SizeF) {
        transform.Translate3d(self.X(size) as f32, self.Y(size) as f32, self.z_ as f32);
    }

    fn PreservesAxisAlignment(&self) -> bool {
        true
    }
    fn IsIdentityOrTranslation(&self) -> bool {
        true
    }
    fn HasNonTrivial3DComponent(&self) -> bool {
        self.z_ != 0.0
    }
}

impl PartialEq for TranslateTransformOperation {
    fn eq(&self, other: &Self) -> bool {
        self.x_ == other.x_ && self.y_ == other.y_ && self.z_ == other.z_
    }
}

// cpp: foundation/blink_geometry/transforms/rotation.h:18-23,49-53
// Vector3dF stores its components as float; the source rotation angle is double.
#[derive(Clone, Copy, Debug)]
pub struct Rotation {
    axis: [f32; 3],
    angle: f64,
}
// Existing Rotation payload exposed for the resolver converter; no second model.
impl Rotation {
    pub fn new(axis: [f32; 3], angle: f64) -> Self {
        Self { axis, angle }
    }
    pub fn Axis(&self) -> &[f32; 3] {
        &self.axis
    }
    pub fn Angle(&self) -> f64 {
        self.angle
    }
}
// cpp: foundation/blink_geometry/transforms/rotate_transform_operation.h:37-75,105-106
#[derive(Clone, Debug)]
pub struct RotateTransformOperation {
    rotation_: Rotation,
    type_: TransformOperationType,
}
#[allow(non_snake_case)]
impl RotateTransformOperation {
    pub fn new(angle: f64, type_: TransformOperationType) -> Self {
        Self::new_3d(0.0, 0.0, 1.0, angle, type_)
    }
    pub fn new_3d(x: f64, y: f64, z: f64, angle: f64, type_: TransformOperationType) -> Self {
        Self {
            rotation_: Rotation {
                axis: [x as f32, y as f32, z as f32],
                angle,
            },
            type_,
        }
    }
    pub fn X(&self) -> f64 {
        f64::from(self.rotation_.axis[0])
    }
    pub fn Y(&self) -> f64 {
        f64::from(self.rotation_.axis[1])
    }
    pub fn Z(&self) -> f64 {
        f64::from(self.rotation_.axis[2])
    }
    pub fn Angle(&self) -> f64 {
        self.rotation_.angle
    }
    pub fn Axis(&self) -> &[f32; 3] {
        &self.rotation_.axis
    }
    pub fn GetType(&self) -> TransformOperationType {
        self.type_
    }
    pub fn IsMatchingOperationType(kind: TransformOperationType) -> bool {
        use TransformOperationType::*;
        matches!(kind, kRotate | kRotateX | kRotateY | kRotateZ | kRotate3D)
    }
    pub fn Apply(&self, transform: &mut gfx::Transform, _size: &gfx::SizeF) {
        if self.type_ == TransformOperationType::kRotate {
            transform.Rotate(self.Angle());
        } else {
            transform.RotateAbout(self.X(), self.Y(), self.Z(), self.Angle());
        }
    }
}
impl Traceable for RotateTransformOperation {
    fn Trace(&self, _visitor: &mut Visitor<'_>) {}
}
impl TransformOperation for RotateTransformOperation {
    fn GetType(&self) -> TransformOperationType {
        self.type_
    }
    // cpp: foundation/blink_geometry/transforms/rotate_transform_operation.cc:47-53
    fn IsEqualAssumingSameType(&self, other: &dyn TransformOperation) -> bool {
        let other = unsafe { &*(other as *const dyn TransformOperation as *const Self) };
        self.rotation_.axis == other.rotation_.axis && self.rotation_.angle == other.rotation_.angle
    }
    fn Apply(&self, transform: &mut gfx::Transform, size: &gfx::SizeF) {
        Self::Apply(self, transform, size);
    }
    fn HasNonTrivial3DComponent(&self) -> bool {
        self.Angle() != 0.0 && (self.X() != 0.0 || self.Y() != 0.0)
    }
}
impl PartialEq for RotateTransformOperation {
    fn eq(&self, other: &Self) -> bool {
        self.type_ == other.type_
            && self.rotation_.axis == other.rotation_.axis
            && self.rotation_.angle == other.rotation_.angle
    }
}
// cpp: foundation/blink_geometry/transforms/scale_transform_operation.h:37-105
#[derive(Clone, Debug)]
pub struct ScaleTransformOperation {
    x_: f64,
    y_: f64,
    z_: f64,
    type_: TransformOperationType,
}
#[allow(non_snake_case)]
impl ScaleTransformOperation {
    pub fn new(x: f64, y: f64, z: f64, type_: TransformOperationType) -> Self {
        debug_assert!(Self::IsMatchingOperationType(type_));
        Self {
            x_: x,
            y_: y,
            z_: z,
            type_,
        }
    }
    pub fn new_2d(x: f64, y: f64, type_: TransformOperationType) -> Self {
        Self::new(x, y, 1.0, type_)
    }
    pub fn X(&self) -> f64 {
        self.x_
    }
    pub fn Y(&self) -> f64 {
        self.y_
    }
    pub fn Z(&self) -> f64 {
        self.z_
    }
    pub fn GetType(&self) -> TransformOperationType {
        self.type_
    }
    pub fn IsMatchingOperationType(kind: TransformOperationType) -> bool {
        use TransformOperationType::*;
        matches!(kind, kScale | kScaleX | kScaleY | kScaleZ | kScale3D)
    }
    pub fn Apply(&self, transform: &mut gfx::Transform, _size: &gfx::SizeF) {
        transform.Scale3d(self.x_ as f32, self.y_ as f32, self.z_ as f32);
    }
}
impl Traceable for ScaleTransformOperation {
    fn Trace(&self, _visitor: &mut Visitor<'_>) {}
}
impl TransformOperation for ScaleTransformOperation {
    fn GetType(&self) -> TransformOperationType {
        self.type_
    }
    fn IsEqualAssumingSameType(&self, other: &dyn TransformOperation) -> bool {
        let other = unsafe { &*(other as *const dyn TransformOperation as *const Self) };
        self.x_ == other.x_ && self.y_ == other.y_ && self.z_ == other.z_
    }
    fn Apply(&self, transform: &mut gfx::Transform, size: &gfx::SizeF) {
        Self::Apply(self, transform, size);
    }
    fn HasNonTrivial3DComponent(&self) -> bool {
        self.z_ != 1.0
    }
    fn PreservesAxisAlignment(&self) -> bool {
        true
    }
    fn IsIdentityOrTranslation(&self) -> bool {
        self.x_ == 1.0 && self.y_ == 1.0 && self.z_ == 1.0
    }
}
impl PartialEq for ScaleTransformOperation {
    fn eq(&self, other: &Self) -> bool {
        self.type_ == other.type_
            && self.x_ == other.x_
            && self.y_ == other.y_
            && self.z_ == other.z_
    }
}

// cpp: foundation/blink_geometry/transforms/transform_operation.h:72-74,94-100,120-121
impl PartialEq for dyn TransformOperation + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.IsSameType(other) && self.IsEqualAssumingSameType(other)
    }
}

// cpp: foundation/blink_geometry/transforms/transform_operations.h:41-43
#[derive(Clone, Copy, Default)]
pub struct EmptyTransformOperations;

// cpp: foundation/blink_geometry/transforms/transform_operations.h:45-155
// The source's GC vector contains Member<TransformOperation>. A Rust trait
// object carries the virtual table while Member remains the non-owning GC edge.
#[derive(Clone, Default)]
pub struct TransformOperations {
    operations_: HeapVector<Member<dyn TransformOperation>, 2>,
}

impl From<EmptyTransformOperations> for TransformOperations {
    fn from(_: EmptyTransformOperations) -> Self {
        Self::default()
    }
}

impl Traceable for TransformOperations {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.operations_);
    }
}

#[allow(non_snake_case)]
impl TransformOperations {
    // cpp: foundation/blink_geometry/transforms/transform_operations.h:50-60
    pub fn PushMatrix3D(&mut self, matrix: gfx::Transform) {
        let operation = MakeGarbageCollected(Matrix3DTransformOperation::new(matrix));
        self.operations_
            .push_back(Member::from_ptr(operation as *mut dyn TransformOperation));
    }

    pub fn PushTranslate(&mut self, x: Length, y: Length) {
        let operation = MakeGarbageCollected(TranslateTransformOperation::new(
            x,
            y,
            0.0,
            TransformOperationType::kTranslate,
        ));
        self.operations_
            .push_back(Member::from_ptr(operation as *mut dyn TransformOperation));
    }

    pub fn PushTranslate3D(&mut self, x: Length, y: Length, z: f64) {
        let operation = MakeGarbageCollected(TranslateTransformOperation::new(
            x,
            y,
            z,
            TransformOperationType::kTranslate3D,
        ));
        self.operations_
            .push_back(Member::from_ptr(operation as *mut dyn TransformOperation));
    }

    // cpp: foundation/blink_geometry/transforms/transform_operations.h:56-60
    pub fn Apply(&self, size: &gfx::SizeF, transform: &mut gfx::Transform) {
        for operation in self.operations_.iter() {
            let pointer = operation.GetNonNull().expect("transform operation is null");
            unsafe { pointer.as_ref().Apply(transform, size) };
        }
    }

    // cpp: foundation/blink_geometry/transforms/transform_operations.h:53
    pub fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.operations_);
    }

    // cpp: foundation/blink_geometry/transforms/transform_operations.h:129-136
    pub fn clear(&mut self) {
        self.operations_.clear();
    }

    pub fn Operations(&self) -> &HeapVector<Member<dyn TransformOperation>, 2> {
        &self.operations_
    }

    pub fn OperationsMut(&mut self) -> &mut HeapVector<Member<dyn TransformOperation>, 2> {
        &mut self.operations_
    }

    pub fn size(&self) -> WtfSizeT {
        self.operations_.size()
    }

    pub fn at(&self, index: WtfSizeT) -> Option<&dyn TransformOperation> {
        if index >= self.size() {
            return None;
        }
        let pointer = self.operations_.at(index).GetNonNull()?;
        Some(unsafe { pointer.as_ref() })
    }

    // cpp: foundation/blink_geometry/transforms/transform_operations.h:66-75
    pub fn Has3DOperation(&self) -> bool {
        self.operations_.iter().any(|operation| {
            let pointer = operation.GetNonNull().expect("transform operation is null");
            unsafe { pointer.as_ref().Is3DOperation() }
        })
    }

    // cpp: foundation/blink_geometry/transforms/transform_operations.h:77-87
    pub fn HasNonPerspective3DOperation(&self) -> bool {
        self.operations_.iter().any(|operation| {
            let pointer = operation.GetNonNull().expect("transform operation is null");
            let operation = unsafe { pointer.as_ref() };
            operation.Is3DOperation() && operation.GetType() != TransformOperationType::kPerspective
        })
    }

    // cpp: foundation/blink_geometry/transforms/transform_operations.h:89-105
    pub fn PreservesAxisAlignment(&self) -> bool {
        self.operations_.iter().all(|operation| {
            let pointer = operation.GetNonNull().expect("transform operation is null");
            unsafe { pointer.as_ref().PreservesAxisAlignment() }
        })
    }

    pub fn IsIdentityOrTranslation(&self) -> bool {
        self.operations_.iter().all(|operation| {
            let pointer = operation.GetNonNull().expect("transform operation is null");
            unsafe { pointer.as_ref().IsIdentityOrTranslation() }
        })
    }

    // cpp: foundation/blink_geometry/transforms/transform_operations.h:108-116
    pub fn HasNonTrivial3DComponent(&self) -> bool {
        self.operations_.iter().any(|operation| {
            let pointer = operation.GetNonNull().expect("transform operation is null");
            unsafe { pointer.as_ref().HasNonTrivial3DComponent() }
        })
    }

    // cpp: foundation/blink_geometry/transforms/transform_operations.h:118-126
    pub fn HasPerspective(&self) -> bool {
        self.operations_.iter().any(|operation| {
            let pointer = operation.GetNonNull().expect("transform operation is null");
            unsafe { pointer.as_ref().GetType() == TransformOperationType::kPerspective }
        })
    }
}

// cpp: foundation/blink_geometry/transforms/transform_operations_core.cc:10-18
impl PartialEq for TransformOperations {
    fn eq(&self, other: &Self) -> bool {
        if self.size() != other.size() {
            return false;
        }
        for index in 0..self.size() {
            let left = self.at(index).expect("transform operation is null");
            let right = other.at(index).expect("transform operation is null");
            if left != right {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestOperation {
        kind: TransformOperationType,
        value: i32,
    }

    impl TransformOperation for TestOperation {
        fn GetType(&self) -> TransformOperationType {
            self.kind
        }
        fn IsEqualAssumingSameType(&self, other: &dyn TransformOperation) -> bool {
            // Only TestOperation instances use this kind in this test.
            let other = unsafe { &*(other as *const dyn TransformOperation as *const Self) };
            self.value == other.value
        }
        fn Apply(&self, _transform: &mut gfx::Transform, _border_box_size: &gfx::SizeF) {
            unreachable!("the comparison test never applies an operation")
        }
    }

    #[test]
    fn compares_operation_values_and_detects_3d_types() {
        let mut left_op = TestOperation {
            kind: TransformOperationType::kTranslate3D,
            value: 3,
        };
        let mut right_op = TestOperation {
            kind: TransformOperationType::kTranslate3D,
            value: 3,
        };
        let mut left = TransformOperations::default();
        let mut right = TransformOperations::default();
        left.OperationsMut().push_back(Member::from_ptr(
            &mut left_op as &mut dyn TransformOperation,
        ));
        right.OperationsMut().push_back(Member::from_ptr(
            &mut right_op as &mut dyn TransformOperation,
        ));
        assert!(left == right);
        assert!(left.Has3DOperation());
        let mut different_op = TestOperation {
            kind: TransformOperationType::kTranslate3D,
            value: 4,
        };
        let mut different = TransformOperations::default();
        different.OperationsMut().push_back(Member::from_ptr(
            &mut different_op as &mut dyn TransformOperation,
        ));
        assert!(left != different);
    }

    #[test]
    fn concrete_rotate_and_scale_apply_in_source_order() {
        let size = gfx::SizeF::new(200.0, 100.0);
        let rotate = RotateTransformOperation::new(90.0, TransformOperationType::kRotate);
        let scale = ScaleTransformOperation::new(2.0, 3.0, 1.0, TransformOperationType::kScale);
        let mut transform = gfx::Transform::default();
        rotate.Apply(&mut transform, &size);
        scale.Apply(&mut transform, &size);
        assert_eq!(
            transform.GetColMajor(),
            [0.0, 2.0, 0.0, 0.0, -3.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]
        );
        let no_axis = RotateTransformOperation::new_3d(
            0.0,
            0.0,
            0.0,
            123.0,
            TransformOperationType::kRotate3D,
        );
        let mut identity = gfx::Transform::default();
        no_axis.Apply(&mut identity, &size);
        assert_eq!(identity, gfx::Transform::default());
    }

    #[test]
    fn two_dimensional_scale_leaves_third_column_bits_untouched() {
        let mut values = gfx::Transform::default().GetColMajor();
        let signaling_nan = f64::from_bits(0x7ff0000000000042);
        values[9] = signaling_nan;
        let mut transform = gfx::Transform::ColMajor(&values);
        transform.Scale3d(2.0, 3.0, 1.0);
        assert_eq!(
            transform.GetColMajor()[9].to_bits(),
            signaling_nan.to_bits()
        );
    }

    #[test]
    fn concrete_operation_values_and_identity_flags_follow_source() {
        let rotate =
            RotateTransformOperation::new_3d(1.0, 2.0, 3.0, 0.0, TransformOperationType::kRotate3D);
        assert!(!rotate.HasNonTrivial3DComponent());
        let scale = ScaleTransformOperation::new(1.0, 1.0, 1.0, TransformOperationType::kScale3D);
        assert!(scale.PreservesAxisAlignment());
        assert!(scale.IsIdentityOrTranslation());
        assert!(!scale.HasNonTrivial3DComponent());
        let same = ScaleTransformOperation::new(1.0, 1.0, 1.0, TransformOperationType::kScale3D);
        let different_kind =
            ScaleTransformOperation::new(1.0, 1.0, 1.0, TransformOperationType::kScale);
        assert_eq!(scale, same);
        assert_ne!(scale, different_kind);
        let nan = RotateTransformOperation::new(f64::NAN, TransformOperationType::kRotate);
        assert_ne!(nan, nan);
        let different_axis =
            RotateTransformOperation::new_3d(1.0, 0.0, 0.0, 0.0, TransformOperationType::kRotate3D);
        assert_ne!(rotate, different_axis);
    }

    #[test]
    fn translated_matrix_and_percentage_translate_apply_in_source_order() {
        let size = gfx::SizeF::new(200.0, 100.0);
        let translate = TranslateTransformOperation::new(
            Length::Percent(25.0),
            Length::Fixed(10.0),
            0.0,
            TransformOperationType::kTranslate,
        );
        let mut transform = gfx::Transform::default();
        translate.Apply(&mut transform, &size);
        assert_eq!(transform.GetColMajor()[12], 50.0);
        assert_eq!(transform.GetColMajor()[13], 10.0);

        let mut matrix_values = gfx::Transform::default().GetColMajor();
        matrix_values[12] = 4.0;
        let matrix = Matrix3DTransformOperation::new(gfx::Transform::ColMajor(&matrix_values));
        matrix.Apply(&mut transform, &size);
        assert_eq!(transform.GetColMajor()[12], 54.0);
        assert_eq!(transform.GetColMajor()[13], 10.0);
    }
}
