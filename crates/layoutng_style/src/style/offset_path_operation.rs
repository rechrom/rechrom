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
