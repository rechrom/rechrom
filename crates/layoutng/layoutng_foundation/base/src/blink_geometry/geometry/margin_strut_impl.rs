// C++: src/foundation/blink_geometry/geometry/margin_strut.cc

use super::layout_unit::LayoutUnit;
use super::margin_strut::MarginStrut;

impl MarginStrut {
    pub fn Append(&mut self, value: &LayoutUnit, is_quirky: bool) {
        if self.is_quirky_container_start && is_quirky {
            return;
        }

        if self.trim_leading_margins {
            return;
        }

        if *value < 0 {
            self.negative_margin = (*value).min(self.negative_margin);
        } else if is_quirky {
            debug_assert!(*value >= 0);
            self.quirky_positive_margin = (*value).max(self.quirky_positive_margin);
        } else {
            self.positive_margin = (*value).max(self.positive_margin);
        }
    }

    pub fn IsEmpty(&self) -> bool {
        if self.discard_margins {
            return true;
        }
        self.positive_margin == LayoutUnit::new()
            && self.negative_margin == LayoutUnit::new()
            && self.quirky_positive_margin == LayoutUnit::new()
    }
}

// C++: operator== compares all six members in source order.
impl PartialEq for MarginStrut {
    fn eq(&self, other: &Self) -> bool {
        self.positive_margin == other.positive_margin
            && self.negative_margin == other.negative_margin
            && self.quirky_positive_margin == other.quirky_positive_margin
            && self.discard_margins == other.discard_margins
            && self.trim_leading_margins == other.trim_leading_margins
            && self.is_quirky_container_start == other.is_quirky_container_start
    }
}

impl Eq for MarginStrut {}
