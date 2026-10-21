// C++: src/foundation/unsupported_layout.h

use std::error::Error;
use std::fmt;

// C++ class UnsupportedLayout : std::logic_error -> Rust struct + impl.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnsupportedLayout {
    message: String,
}

impl UnsupportedLayout {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for UnsupportedLayout {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for UnsupportedLayout {}
