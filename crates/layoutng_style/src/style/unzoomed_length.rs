use foundation::Length;

// cpp: layoutng_style/style/unzoomed_length.h:36-54
#[derive(Clone, Debug, PartialEq)]
pub struct UnzoomedLength {
    length_: Length,
}

#[allow(non_snake_case)]
impl UnzoomedLength {
    // cpp: layoutng_style/style/unzoomed_length.h:42
    pub fn new(length: &Length) -> Self {
        Self {
            length_: length.clone(),
        }
    }

    // cpp: layoutng_style/style/unzoomed_length.h:44
    pub fn IsZero(&self) -> bool {
        self.length_.IsZero()
    }

    // cpp: layoutng_style/style/unzoomed_length.h:50
    pub fn length(&self) -> &Length {
        &self.length_
    }
}
