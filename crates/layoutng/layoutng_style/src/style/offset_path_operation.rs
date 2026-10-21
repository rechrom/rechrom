use foundation::Visitor;

use super::computed_style_constants::CoordBox;

// cpp: layoutng_style/style/offset_path_operation.h:15
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum OperationType {
    kReference,
    kShape,
    kCoordBox,
}

// cpp: layoutng_style/style/offset_path_operation.h:36-39
pub struct OffsetPathOperationBase {
    coord_box_: CoordBox,
}

impl OffsetPathOperationBase {
    pub fn new(coord_box: CoordBox) -> Self {
        Self {
            coord_box_: coord_box,
        }
    }
}

// cpp: layoutng_style/style/offset_path_operation.h:13-40
#[allow(non_snake_case)]
pub trait OffsetPathOperation {
    // Rust exposes the inherited base field through this adapter method.
    fn base(&self) -> &OffsetPathOperationBase;

    // cpp: layoutng_style/style/offset_path_operation.h:21
    fn Trace(&self, _visitor: Option<&mut Visitor>) {}

    // cpp: layoutng_style/style/offset_path_operation.h:28
    fn GetType(&self) -> OperationType;

    // cpp: layoutng_style/style/offset_path_operation.h:29-31
    fn IsSameType(&self, other: &dyn OffsetPathOperation) -> bool {
        other.GetType() == self.GetType()
    }

    // cpp: layoutng_style/style/offset_path_operation.h:33
    fn GetCoordBox(&self) -> CoordBox {
        self.base().coord_box_
    }

    // cpp: layoutng_style/style/offset_path_operation.h:37
    fn IsEqualAssumingSameType(&self, other: &dyn OffsetPathOperation) -> bool;
}

// cpp: layoutng_style/style/offset_path_operation.h:23-26
impl PartialEq for dyn OffsetPathOperation + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.IsSameType(other)
            && self.IsEqualAssumingSameType(other)
            && self.GetCoordBox() == other.GetCoordBox()
    }
}

// cpp: core/style/coord_box_offset_path_operation.h:13-31.
pub struct CoordBoxOffsetPathOperation {
    base: OffsetPathOperationBase,
}
impl CoordBoxOffsetPathOperation {
    pub fn new(coord_box: CoordBox) -> Self {
        Self {
            base: OffsetPathOperationBase::new(coord_box),
        }
    }
}
#[allow(non_snake_case)]
impl OffsetPathOperation for CoordBoxOffsetPathOperation {
    fn base(&self) -> &OffsetPathOperationBase {
        &self.base
    }
    fn GetType(&self) -> OperationType {
        OperationType::kCoordBox
    }
    fn IsEqualAssumingSameType(&self, _: &dyn OffsetPathOperation) -> bool {
        true
    }
}
impl foundation::Traceable for CoordBoxOffsetPathOperation {
    fn Trace(&self, _: &mut Visitor<'_>) {}
}

// cpp: core/style/shape_offset_path_operation.h:15-49.
pub struct ShapeOffsetPathOperation {
    base: OffsetPathOperationBase,
    shape: foundation::Member<dyn super::basic_shapes::BasicShape>,
}
#[allow(non_snake_case)]
impl ShapeOffsetPathOperation {
    /// Shape is allocated in the active native layout heap.
    pub unsafe fn new(
        shape: *mut dyn super::basic_shapes::BasicShape,
        coord_box: CoordBox,
    ) -> Self {
        Self {
            base: OffsetPathOperationBase::new(coord_box),
            shape: foundation::Member::from_ptr(shape),
        }
    }
    pub fn GetBasicShape(&self) -> &dyn super::basic_shapes::BasicShape {
        unsafe {
            self.shape
                .GetNonNull()
                .expect("offset shape is non-null")
                .as_ref()
        }
    }
}
#[allow(non_snake_case)]
impl OffsetPathOperation for ShapeOffsetPathOperation {
    fn base(&self) -> &OffsetPathOperationBase {
        &self.base
    }
    fn GetType(&self) -> OperationType {
        OperationType::kShape
    }
    fn IsEqualAssumingSameType(&self, other: &dyn OffsetPathOperation) -> bool {
        debug_assert!(self.IsSameType(other));
        let other = unsafe { &*(other as *const dyn OffsetPathOperation as *const Self) };
        self.GetBasicShape() == other.GetBasicShape()
    }
    fn Trace(&self, visitor: Option<&mut Visitor>) {
        if let Some(visitor) = visitor {
            visitor.Trace(&self.shape);
        }
    }
}
impl foundation::Traceable for ShapeOffsetPathOperation {
    fn Trace(&self, v: &mut Visitor<'_>) {
        OffsetPathOperation::Trace(self, Some(v));
    }
}
