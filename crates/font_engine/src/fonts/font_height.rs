use foundation::LayoutUnit;

// cpp: font_engine/fonts/font_height.h:16-56
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FontHeight {
    pub ascent: LayoutUnit,
    pub descent: LayoutUnit,
}

impl FontHeight {
    // cpp: font_engine/fonts/font_height.h:19-20
    pub fn new(ascent: LayoutUnit, descent: LayoutUnit) -> Self {
        Self { ascent, descent }
    }

    // cpp: font_engine/fonts/font_height.h:23-25
    pub fn Empty() -> Self {
        Self::new(LayoutUnit::Min(), LayoutUnit::Min())
    }

    // cpp: font_engine/fonts/font_height.h:26-28
    pub fn IsEmpty(&self) -> bool {
        self.ascent == LayoutUnit::Min() && self.descent == LayoutUnit::Min()
    }

    // cpp: font_engine/fonts/font_height.h:30
    pub fn LineHeight(&self) -> LayoutUnit {
        self.ascent + self.descent
    }

    // cpp: font_engine/fonts/font_height.h:37-39
    pub fn Contains(&self, other: &Self) -> bool {
        other.ascent <= self.ascent && other.descent <= self.descent
    }

    // cpp: font_engine/fonts/font_height.cc:10-14
    pub fn AddLeading(&mut self, leading_space: &Self) {
        debug_assert!(!self.IsEmpty());
        self.ascent += leading_space.ascent;
        self.descent += leading_space.descent;
    }

    // cpp: font_engine/fonts/font_height.cc:16-20
    pub fn Move(&mut self, delta: LayoutUnit) {
        debug_assert!(!self.IsEmpty());
        self.ascent -= delta;
        self.descent += delta;
    }

    // cpp: font_engine/fonts/font_height.cc:22-25
    pub fn Unite(&mut self, other: &Self) {
        self.ascent = self.ascent.max(other.ascent);
        self.descent = self.descent.max(other.descent);
    }
}

// cpp: font_engine/fonts/font_height.cc:27-34
impl std::ops::AddAssign<&FontHeight> for FontHeight {
    fn add_assign(&mut self, other: &Self) {
        debug_assert!(self.ascent != LayoutUnit::Min() && self.descent != LayoutUnit::Min());
        debug_assert!(other.ascent != LayoutUnit::Min() && other.descent != LayoutUnit::Min());
        self.ascent += other.ascent;
        self.descent += other.descent;
    }
}
