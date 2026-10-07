use foundation::{HeapVector, MakeGarbageCollected, Member, Visitor};

use super::style_image::StyleImage;

impl foundation::Traceable for PaintImages {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        PaintImages::Trace(self, visitor);
    }
}

// cpp: layoutng_style/style/paint_images.h:21
pub type ImageList = HeapVector<Member<StyleImage>>;

// cpp: layoutng_style/style/paint_images.h:15-30
#[derive(Clone, Default)]
pub struct PaintImages {
    images_: ImageList,
}

#[allow(non_snake_case)]
impl PaintImages {
    // cpp: layoutng_style/style/paint_images.h:17-19
    pub fn Clone(&self) -> *mut Self {
        MakeGarbageCollected(self.clone())
    }

    // cpp: layoutng_style/style/paint_images.h:23
    pub fn ImagesMut(&mut self) -> &mut ImageList {
        &mut self.images_
    }

    // cpp: layoutng_style/style/paint_images.h:24
    pub fn Images(&self) -> &ImageList {
        &self.images_
    }

    // cpp: layoutng_style/style/paint_images.h:26
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.images_);
    }
}
