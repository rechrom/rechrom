//! core/style/reference_clip_path_operation.h:31-65.
use super::clip_path_operation::{ClipPathOperation, OperationType};
use foundation::{String, Traceable, Visitor};

/// Stable document/TreeScope identity supplied by the document resource owner.
/// Inheritance retains this binding with the canonical operation, just as
/// Blink's ReferenceClipPathOperation retains its SVGResource.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SvgClipResourceScope {
    pub document_id: u64,
    pub tree_scope_id: u64,
}

pub struct ReferenceClipPathOperation {
    url: String,
    scope: Option<SvgClipResourceScope>,
}

#[allow(non_snake_case)]
impl ReferenceClipPathOperation {
    pub fn new(url: String) -> Self {
        Self { url, scope: None }
    }
    pub fn with_scope(url: String, scope: Option<SvgClipResourceScope>) -> Self {
        Self { url, scope }
    }
    pub fn ResourceScope(&self) -> Option<SvgClipResourceScope> {
        self.scope
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
        self.url == other.url && self.scope == other.scope
    }
}
impl Traceable for ReferenceClipPathOperation {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        ClipPathOperation::Trace(self, visitor)
    }
}
