use foundation::{Member, Visitor};

use super::basic_shapes::BasicShape;
use super::computed_style_constants::ShapeBox;
use super::style_image::StyleImage;

// cpp: layoutng_style/style/shape_value.h:46-51
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShapeValueType {
    kShape,
    kBox,
    kImage,
}

// cpp: layoutng_style/style/shape_value.h:44-83
pub struct ShapeValue {
    type_: ShapeValueType,
    shape_: Option<Member<dyn BasicShape>>,
    image_: Member<StyleImage>,
    box_: ShapeBox,
}

#[allow(non_snake_case)]
impl ShapeValue {
    // cpp: layoutng_style/style/shape_value.h:53-54
    pub fn from_image(image: *mut StyleImage) -> Self {
        Self {
            type_: ShapeValueType::kImage,
            shape_: None,
            image_: Member::from_ptr(image),
            box_: ShapeBox::kContentBox,
        }
    }

    // cpp: layoutng_style/style/shape_value.h:55
    pub fn from_box(shape_box: ShapeBox) -> Self {
        Self {
            type_: ShapeValueType::kBox,
            shape_: None,
            image_: Member::default(),
            box_: shape_box,
        }
    }

    // cpp: layoutng_style/style/shape_value.h:56-57
    // The source stores a GC member, so the caller must provide a pointer to
    // a registered object that stays live through tracing, not a stack borrow.
    pub unsafe fn from_shape(shape: *const dyn BasicShape, shape_box: ShapeBox) -> Self {
        Self {
            type_: ShapeValueType::kShape,
            shape_: Some(Member::from_ptr(shape.cast_mut())),
            image_: Member::default(),
            box_: shape_box,
        }
    }

    // cpp: layoutng_style/style/shape_value.h:59-60
    pub fn GetType(&self) -> ShapeValueType {
        self.type_
    }
    pub fn Shape(&self) -> &dyn BasicShape {
        let shape = self
            .shape_
            .as_ref()
            .and_then(Member::GetNonNull)
            .expect("shape value required");
        unsafe { shape.as_ref() }
    }

    // cpp: layoutng_style/style/shape_value.h:62-69
    pub fn GetImage(&self) -> *mut StyleImage {
        self.image_.Get()
    }
    pub fn SetImage(&mut self, image: *mut StyleImage) {
        debug_assert_eq!(self.GetType(), ShapeValueType::kImage);
        if self.GetImage() != image {
            self.image_ = Member::from_ptr(image);
        }
    }
    pub fn CssBox(&self) -> ShapeBox {
        self.box_
    }

    // cpp: layoutng_style/style/shape_value.h:73-76
    pub fn Trace(&self, visitor: &mut Visitor) {
        if let Some(shape) = &self.shape_ {
            visitor.Trace(shape);
        }
        visitor.Trace(&self.image_);
    }
}

// cpp: layoutng_style/style/shape_value.h:85-100
impl PartialEq for ShapeValue {
    fn eq(&self, other: &Self) -> bool {
        if self.GetType() != other.GetType() {
            return false;
        }
        match self.GetType() {
            ShapeValueType::kShape => {
                self.CssBox() == other.CssBox() && self.Shape() == other.Shape()
            }
            ShapeValueType::kBox => self.CssBox() == other.CssBox(),
            ShapeValueType::kImage => {
                foundation::ValuesEquivalent(self.GetImage(), other.GetImage())
            }
        }
    }
}
