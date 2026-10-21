use foundation::{Member, Point, Visitor};

use super::style_image::StyleImage;

impl foundation::Traceable for CursorData {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        CursorData::Trace(self, visitor);
    }
}

// cpp: layoutng_style/style/cursor_data.h:34-63
pub struct CursorData {
    image_: Member<StyleImage>,
    hot_spot_specified_: bool,
    hot_spot_: Point,
}

#[allow(non_snake_case)]
impl CursorData {
    // cpp: layoutng_style/style/cursor_data.h:38-43
    pub fn new(image: *mut StyleImage, hot_spot_specified: bool, hot_spot: &Point) -> Self {
        Self {
            image_: Member::from_ptr(image),
            hot_spot_specified_: hot_spot_specified,
            hot_spot_: hot_spot.clone(),
        }
    }

    // cpp: layoutng_style/style/cursor_data.h:49
    pub fn GetImage(&self) -> *mut StyleImage {
        self.image_.Get()
    }

    // cpp: layoutng_style/style/cursor_data.h:50
    pub fn SetImage(&mut self, image: *mut StyleImage) {
        self.image_ = Member::from_ptr(image);
    }

    // cpp: layoutng_style/style/cursor_data.h:52
    pub fn HotSpotSpecified(&self) -> bool {
        self.hot_spot_specified_
    }

    // cpp: layoutng_style/style/cursor_data.h:54-55
    pub fn HotSpot(&self) -> &Point {
        &self.hot_spot_
    }

    // cpp: layoutng_style/style/cursor_data.h:57
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.image_);
    }
}

// cpp: layoutng_style/style/cursor_data.h:45-47
impl PartialEq for CursorData {
    fn eq(&self, other: &Self) -> bool {
        self.hot_spot_ == other.hot_spot_
            && foundation::ValuesEquivalent(&self.image_, &other.image_)
    }
}

// cpp: layoutng_style/style/cursor_data.h:67
pub struct CursorDataVectorTraits;

#[allow(non_upper_case_globals)]
impl CursorDataVectorTraits {
    pub const kCanClearUnusedSlotsWithMemset: bool = true;
}
