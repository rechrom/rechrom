//! core/style/reference_clip_path_operation.h:31-65.
use super::clip_path_operation::{ClipPathOperation, OperationType};
use foundation::{String, Traceable, Visitor};

pub struct ReferenceClipPathOperation {
    url: String,
}

#[allow(non_snake_case)]
impl ReferenceClipPathOperation {
    pub fn new(url: String) -> Self {
        Self { url }
    }
    pub fn Url(&self) -> &String {
        &self.url
    }
}

#[allow(non_snake_case)]
impl ClipPathOperation for ReferenceClipPathOperation {
    fn GetType(&self) -> OperationType {
        OperationType::kReference
    }
    fn Equals(&self, other: &dyn ClipPathOperation) -> bool {
        if !self.IsSameType(other) {
            return false;
        }
        let other = unsafe { &*(other as *const dyn ClipPathOperation as *const Self) };
        self.url == other.url
    }
}
impl Traceable for ReferenceClipPathOperation {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        ClipPathOperation::Trace(self, visitor)
    }
}
