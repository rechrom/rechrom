use foundation::Length;

use super::border_image_length::BorderImageLength;

// cpp: layoutng_style/style/border_image_length_box.h:43-79
#[derive(Clone)]
pub struct BorderImageLengthBox {
    left_: BorderImageLength,
    right_: BorderImageLength,
    top_: BorderImageLength,
    bottom_: BorderImageLength,
}

#[allow(non_snake_case)]
impl BorderImageLengthBox {
    // cpp: layoutng_style/style/border_image_length_box.h:47-48
    pub fn from_length(length: Length) -> Self {
        Self {
            left_: BorderImageLength::from_length(&length),
            right_: BorderImageLength::from_length(&length),
            top_: BorderImageLength::from_length(&length),
            bottom_: BorderImageLength::from_length(&length),
        }
    }

    // cpp: layoutng_style/style/border_image_length_box.h:50-51
    pub fn from_number(number: f64) -> Self {
        Self {
            left_: BorderImageLength::from_number(number),
            right_: BorderImageLength::from_number(number),
            top_: BorderImageLength::from_number(number),
            bottom_: BorderImageLength::from_number(number),
        }
    }

    // cpp: layoutng_style/style/border_image_length_box.h:53-57
    pub fn new(
        top: &BorderImageLength,
        right: &BorderImageLength,
        bottom: &BorderImageLength,
        left: &BorderImageLength,
    ) -> Self {
        Self {
            left_: left.clone(),
            right_: right.clone(),
            top_: top.clone(),
            bottom_: bottom.clone(),
        }
    }

    // cpp: layoutng_style/style/border_image_length_box.h:59-62
    pub fn Left(&self) -> &BorderImageLength {
        &self.left_
    }
    pub fn Right(&self) -> &BorderImageLength {
        &self.right_
    }
    pub fn Top(&self) -> &BorderImageLength {
        &self.top_
    }
    pub fn Bottom(&self) -> &BorderImageLength {
        &self.bottom_
    }

    // cpp: layoutng_style/style/border_image_length_box.h:69-72
    pub fn NonZero(&self) -> bool {
        !(self.left_.IsZero()
            && self.right_.IsZero()
            && self.top_.IsZero()
            && self.bottom_.IsZero())
    }
}

// cpp: layoutng_style/style/border_image_length_box.h:64-67
impl PartialEq for BorderImageLengthBox {
    fn eq(&self, other: &Self) -> bool {
        self.left_ == other.left_
            && self.right_ == other.right_
            && self.top_ == other.top_
            && self.bottom_ == other.bottom_
    }
}
