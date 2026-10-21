//! core/style/shape_clip_path_operation.h:45-92.
use super::basic_shapes::BasicShape;
use super::clip_path_operation::{ClipPathOperation, OperationType};
use super::computed_style_constants::GeometryBox;
use foundation::{gfx, Member, Path, Traceable, Visitor};

pub struct ShapeClipPathOperation {
    shape: Member<dyn BasicShape>,
    geometry_box: GeometryBox,
}

#[allow(non_snake_case)]
impl ShapeClipPathOperation {
    /// The shape must have been allocated in the current layout heap.
    pub unsafe fn new(shape: *mut dyn BasicShape, geometry_box: GeometryBox) -> Self {
        Self {
            shape: Member::from_ptr(shape),
            geometry_box,
        }
    }
    pub fn GetBasicShape(&self) -> &dyn BasicShape {
        unsafe {
            self.shape
                .GetNonNull()
                .expect("clip-path needs its shape")
                .as_ref()
        }
    }
    pub fn GetGeometryBox(&self) -> GeometryBox {
        self.geometry_box
    }
    pub fn GetPath(&self, rect: &gfx::RectF, zoom: f32, path_scale: f32) -> Path {
        self.GetBasicShape().GetPath(rect, zoom, path_scale)
    }
}

#[allow(non_snake_case)]
impl ClipPathOperation for ShapeClipPathOperation {
    fn GetType(&self) -> OperationType {
        OperationType::kShape
    }
    fn Equals(&self, other: &dyn ClipPathOperation) -> bool {
        if !self.IsSameType(other) {
            return false;
        }
        let other = unsafe { &*(other as *const dyn ClipPathOperation as *const Self) };
        self.GetBasicShape() == other.GetBasicShape() && self.geometry_box == other.geometry_box
    }
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.shape);
    }
}
impl Traceable for ShapeClipPathOperation {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        ClipPathOperation::Trace(self, visitor);
    }
}
