use crate::src::core::{
    SkColor::SkColor,
    SkPoint::SkPoint,
    SkVertices::{SkVertices, VertexMode},
};
// Source: src/core/SkVerticesPriv.h:18-18
#[repr(C)]
pub struct SkVertices_DeprecatedBone {
    pub values: [f32; 6],
}
// Source: src/core/SkVerticesPriv.h:23-24
// A borrowed privileged window, not an additional owner/refcount or copied arrays.
pub struct SkVerticesPriv<'a> {
    // Source: src/core/SkVerticesPriv.h:53-53
    pub(crate) fVertices: &'a SkVertices,
}
// Source: src/core/SkVerticesPriv.h:25-37
impl SkVerticesPriv<'_> {
    pub fn mode(&self) -> VertexMode {
        self.fVertices.fMode
    }
    pub fn hasColors(&self) -> bool {
        !self.fVertices.fColors.is_null()
    }
    pub fn hasTexCoords(&self) -> bool {
        !self.fVertices.fTexs.is_null()
    }
    pub fn hasIndices(&self) -> bool {
        !self.fVertices.fIndices.is_null()
    }
    pub fn vertexCount(&self) -> i32 {
        self.fVertices.fVertexCount
    }
    pub fn indexCount(&self) -> i32 {
        self.fVertices.fIndexCount
    }
    pub fn positions(&self) -> *const SkPoint {
        self.fVertices.fPositions
    }
    pub fn texCoords(&self) -> *const SkPoint {
        self.fVertices.fTexs
    }
    pub fn colors(&self) -> *const SkColor {
        self.fVertices.fColors
    }
    pub fn indices(&self) -> *const u16 {
        self.fVertices.fIndices
    }
}
// Source: src/core/SkVerticesPriv.h:40-40
impl Copy for SkVerticesPriv<'_> {}
impl Clone for SkVerticesPriv<'_> {
    fn clone(&self) -> Self {
        *self
    }
}
// Source: src/core/SkVerticesPriv.h:46-46
impl<'a> SkVerticesPriv<'a> {
    pub(crate) fn new(vertices: &'a SkVertices) -> Self {
        Self {
            fVertices: vertices,
        }
    }
}
// Source: src/core/SkVerticesPriv.h:58-62
impl SkVertices {
    // Source: include/core/SkVertices.h:104-105
    pub fn priv_(&self) -> SkVerticesPriv<'_> {
        SkVerticesPriv::new(self)
    }
}
