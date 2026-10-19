// Source: components/viz/service/surfaces/surface_reference.h:1-54
// Source: components/viz/service/surfaces/surface_reference.cc:1-34
#![allow(non_snake_case)]

use crate::common::surfaces::surface_id::SurfaceId;

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SurfaceReference {
    parent_id_: SurfaceId,
    child_id_: SurfaceId,
}
impl SurfaceReference {
    pub const fn new(parent: SurfaceId, child: SurfaceId) -> Self {
        Self {
            parent_id_: parent,
            child_id_: child,
        }
    }
    pub const fn parent_id(self) -> SurfaceId {
        self.parent_id_
    }
    pub const fn child_id(self) -> SurfaceId {
        self.child_id_
    }
    pub fn ToString(self) -> String {
        format!(
            "SurfaceReference(parent: {}, child: {})",
            self.parent_id_, self.child_id_
        )
    }
}
