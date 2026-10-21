use foundation::Length;

// cpp: layoutng_style/style/border_image_length.h:88
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BorderImageLengthType {
    Length,
    Number,
}

// cpp: layoutng_style/style/border_image_length.h:43-89
#[derive(Clone)]
pub struct BorderImageLength {
    length_: Length,
    number_: f64,
    type_: BorderImageLengthType,
}

#[allow(non_snake_case)]
impl BorderImageLength {
    // cpp: layoutng_style/style/border_image_length.h:47
    pub fn from_number(number: f64) -> Self {
        Self {
            length_: Length::default(),
            number_: number,
            type_: BorderImageLengthType::Number,
        }
    }

    // cpp: layoutng_style/style/border_image_length.h:49-50
    pub fn from_length(length: &Length) -> Self {
        Self {
            length_: length.clone(),
            number_: 0.0,
            type_: BorderImageLengthType::Length,
        }
    }

    // cpp: layoutng_style/style/border_image_length.h:52-53
    pub fn IsNumber(&self) -> bool {
        self.type_ == BorderImageLengthType::Number
    }
    pub fn IsLength(&self) -> bool {
        self.type_ == BorderImageLengthType::Length
    }

    // cpp: layoutng_style/style/border_image_length.h:55-62
    pub fn length(&self) -> &Length {
        debug_assert!(self.IsLength());
        &self.length_
    }
    pub fn length_mut(&mut self) -> &mut Length {
        debug_assert!(self.IsLength());
        &mut self.length_
    }

    // cpp: layoutng_style/style/border_image_length.h:64-67
    pub fn Number(&self) -> f64 {
        debug_assert!(self.IsNumber());
        self.number_
    }

    // cpp: layoutng_style/style/border_image_length.h:74-81
    pub fn IsZero(&self) -> bool {
        if self.IsLength() {
            return self.length_.IsZero();
        }
        debug_assert!(self.IsNumber());
        self.number_ == 0.0
    }
}

// cpp: layoutng_style/style/border_image_length.h:69-72
impl PartialEq for BorderImageLength {
    fn eq(&self, other: &Self) -> bool {
        self.type_ == other.type_ && self.length_ == other.length_ && self.number_ == other.number_
    }
}
