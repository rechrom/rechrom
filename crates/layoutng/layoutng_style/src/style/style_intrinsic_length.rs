use foundation::Length;

// cpp: layoutng_style/style/style_intrinsic_length.h:20-22
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StyleIntrinsicLengthOptions {
    pub has_auto: bool,
}

// cpp: layoutng_style/style/style_intrinsic_length.h:16-48
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StyleIntrinsicLength {
    has_auto_: bool,
    length_: Option<Length>,
}

#[allow(non_snake_case)]
impl StyleIntrinsicLength {
    // cpp: layoutng_style/style/style_intrinsic_length.h:24-27
    pub fn new(length: &Option<Length>, options: StyleIntrinsicLengthOptions) -> Self {
        Self {
            has_auto_: options.has_auto,
            length_: length.clone(),
        }
    }

    // cpp: layoutng_style/style/style_intrinsic_length.h:31-33
    pub fn IsNoOp(&self) -> bool {
        !self.has_auto_ && self.length_.is_none()
    }

    // cpp: layoutng_style/style/style_intrinsic_length.h:35
    pub fn HasAuto(&self) -> bool {
        self.has_auto_
    }

    // cpp: layoutng_style/style/style_intrinsic_length.h:37
    pub fn SetHasAuto(&mut self) {
        self.has_auto_ = true;
    }

    // cpp: layoutng_style/style/style_intrinsic_length.h:39
    pub fn GetLength(&self) -> &Option<Length> {
        &self.length_
    }
}
