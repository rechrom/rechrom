// C++: src/foundation/text/native/writing_mode.cc

use std::fmt;

use super::writing_mode::WritingMode;

// C++: writing_mode.cc, operator<<.
impl fmt::Display for WritingMode {
    fn fmt(&self, ostream: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            WritingMode::kHorizontalTb => "horizontal-tb",
            WritingMode::kVerticalRl => "vertical-rl",
            WritingMode::kVerticalLr => "vertical-lr",
            WritingMode::kSidewaysRl => "sideways-rl",
            WritingMode::kSidewaysLr => "sideways-lr",
        };
        ostream.write_str(value)
    }
}
