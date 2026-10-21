// C++: src/foundation/text/native/text_direction.cc

use std::fmt;

use super::text_direction::{IsLtr, TextDirection};

// C++: text_direction.cc, operator<<.
impl fmt::Display for TextDirection {
    fn fmt(&self, ostream: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(ostream, "{}", if IsLtr(*self) { "LTR" } else { "RTL" })
    }
}
