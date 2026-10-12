// C++: src/foundation/text/native/tab_size.h

#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TabSizeValueType {
    kLength,
    kSpace,
}

// C++ struct TabSize -> Rust struct + impl.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TabSize {
    pub float_value_: f32,
    pub is_spaces_: bool,
}

impl TabSize {
    // C++: tab_size.h, constructor.
    pub fn new(num_or_length: f32, is_spaces: TabSizeValueType) -> Self {
        Self {
            float_value_: num_or_length,
            is_spaces_: is_spaces == TabSizeValueType::kSpace,
        }
    }

    // C++ default argument: TabSizeValueType::kSpace.
    pub fn spaces(num: f32) -> Self {
        Self::new(num, TabSizeValueType::kSpace)
    }

    // C++: tab_size.h, IsSpaces.
    pub fn IsSpaces(&self) -> bool {
        self.is_spaces_
    }
}

// C++: tab_size.h, operator==.
impl PartialEq for TabSize {
    fn eq(&self, other: &Self) -> bool {
        self.float_value_ == other.float_value_ && self.is_spaces_ == other.is_spaces_
    }
}
