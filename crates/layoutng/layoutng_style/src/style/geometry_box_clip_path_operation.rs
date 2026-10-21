// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Chromium core/style/geometry_box_clip_path_operation.h:13-40.
use super::{
    clip_path_operation::{ClipPathOperation, OperationType},
    computed_style_constants::GeometryBox,
};
use foundation::{Traceable, Visitor};
pub struct GeometryBoxClipPathOperation {
    geometry_box: GeometryBox,
}
#[allow(non_snake_case)]
impl GeometryBoxClipPathOperation {
    pub fn new(geometry_box: GeometryBox) -> Self {
        Self { geometry_box }
    }
    pub fn GetGeometryBox(&self) -> GeometryBox {
        self.geometry_box
    }
}
#[allow(non_snake_case)]
impl ClipPathOperation for GeometryBoxClipPathOperation {
    fn GetType(&self) -> OperationType {
        OperationType::kGeometryBox
    }
    fn Equals(&self, other: &dyn ClipPathOperation) -> bool {
        if !self.IsSameType(other) {
            return false;
        }
        let other = unsafe { &*(other as *const dyn ClipPathOperation as *const Self) };
        self.geometry_box == other.geometry_box
    }
}
impl Traceable for GeometryBoxClipPathOperation {
    fn Trace(&self, _: &mut Visitor<'_>) {}
}
