use foundation::Visitor;

// cpp: layoutng_style/style/clip_path_operation.h:41
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperationType {
    kReference,
    kShape,
    kGeometryBox,
}

// cpp: layoutng_style/style/clip_path_operation.h:39-55
#[allow(non_snake_case)]
pub trait ClipPathOperation {
    // cpp: layoutng_style/style/clip_path_operation.h:44
    fn Trace(&self, _visitor: &mut Visitor) {}

    // cpp: layoutng_style/style/clip_path_operation.h:46
    fn Equals(&self, other: &dyn ClipPathOperation) -> bool;

    // cpp: layoutng_style/style/clip_path_operation.h:48
    fn GetType(&self) -> OperationType;

    // cpp: layoutng_style/style/clip_path_operation.h:49-51
    fn IsSameType(&self, other: &dyn ClipPathOperation) -> bool {
        other.GetType() == self.GetType()
    }
}
