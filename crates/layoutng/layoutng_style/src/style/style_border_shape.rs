use foundation::{Member, Visitor};

use super::basic_shapes::BasicShape;
use super::computed_style_constants::GeometryBox;

// cpp: layoutng_style/style/style_border_shape.h:17-56
pub struct StyleBorderShape {
    outer_: Member<dyn BasicShape>,
    inner_: Member<dyn BasicShape>,
    outer_box_: GeometryBox,
    inner_box_: GeometryBox,
}

#[allow(non_snake_case)]
impl StyleBorderShape {
    // cpp: layoutng_style/style/style_border_shape.h:21-28
    // SAFETY: C++ takes a BasicShape reference to a GC-managed object and
    // stores it in Member. Callers must supply live GC-managed shape pointers.
    pub unsafe fn new(
        outer: *mut dyn BasicShape,
        inner: Option<*mut dyn BasicShape>,
        outer_box: GeometryBox,
        inner_box: GeometryBox,
    ) -> Self {
        Self {
            outer_: Member::from_ptr(outer),
            inner_: Member::from_ptr(inner.unwrap_or(outer)),
            outer_box_: outer_box,
            inner_box_: inner_box,
        }
    }

    // cpp: layoutng_style/style/style_border_shape.h:30-33
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.outer_);
        visitor.Trace(&self.inner_);
    }

    // cpp: layoutng_style/style/style_border_shape.h:35-37
    pub fn HasSeparateInnerShape(&self) -> bool {
        !foundation::ValuesEquivalent(&self.inner_, &self.outer_)
            || self.inner_box_ != self.outer_box_
    }

    // cpp: layoutng_style/style/style_border_shape.h:39-43
    pub fn OuterShape(&self) -> &dyn BasicShape {
        let shape = self.outer_.GetNonNull().expect("outer shape required");
        unsafe { shape.as_ref() }
    }
    pub fn InnerShape(&self) -> &dyn BasicShape {
        let shape = self.inner_.GetNonNull().expect("inner shape required");
        unsafe { shape.as_ref() }
    }
    pub fn OuterBox(&self) -> GeometryBox {
        self.outer_box_
    }
    pub fn InnerBox(&self) -> GeometryBox {
        self.inner_box_
    }
}

// cpp: layoutng_style/style/style_border_shape.h:45-49
impl PartialEq for StyleBorderShape {
    fn eq(&self, other: &Self) -> bool {
        foundation::ValuesEquivalent(&self.outer_, &other.outer_)
            && foundation::ValuesEquivalent(&self.inner_, &other.inner_)
            && self.outer_box_ == other.outer_box_
            && self.inner_box_ == other.inner_box_
    }
}
