// Source: include/core/SkFontMetrics.h:18-37
impl PartialEq for SkFontMetrics {
    fn eq(&self, that: &Self) -> bool {
        self.fFlags == that.fFlags
            && self.fTop == that.fTop
            && self.fAscent == that.fAscent
            && self.fDescent == that.fDescent
            && self.fBottom == that.fBottom
            && self.fLeading == that.fLeading
            && self.fAvgCharWidth == that.fAvgCharWidth
            && self.fMaxCharWidth == that.fMaxCharWidth
            && self.fXMin == that.fXMin
            && self.fXMax == that.fXMax
            && self.fXHeight == that.fXHeight
            && self.fCapHeight == that.fCapHeight
            && self.fUnderlineThickness == that.fUnderlineThickness
            && self.fUnderlinePosition == that.fUnderlinePosition
            && self.fStrikeoutThickness == that.fStrikeoutThickness
            && self.fStrikeoutPosition == that.fStrikeoutPosition
    }
}
// Source: include/core/SkFontMetrics.h:44-50
impl SkFontMetrics {
    pub const kUnderlineThicknessIsValid_Flag: u32 = 1 << 0;
    pub const kUnderlinePositionIsValid_Flag: u32 = 1 << 1;
    pub const kStrikeoutThicknessIsValid_Flag: u32 = 1 << 2;
    pub const kStrikeoutPositionIsValid_Flag: u32 = 1 << 3;
    pub const kBoundsInvalid_Flag: u32 = 1 << 4;
}
// Source: include/core/SkFontMetrics.h:52-67
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SkFontMetrics {
    pub fFlags: u32,
    pub fTop: f32,
    pub fAscent: f32,
    pub fDescent: f32,
    pub fBottom: f32,
    pub fLeading: f32,
    pub fAvgCharWidth: f32,
    pub fMaxCharWidth: f32,
    pub fXMin: f32,
    pub fXMax: f32,
    pub fXHeight: f32,
    pub fCapHeight: f32,
    pub fUnderlineThickness: f32,
    pub fUnderlinePosition: f32,
    pub fStrikeoutThickness: f32,
    pub fStrikeoutPosition: f32,
}
impl SkFontMetrics {
    // Source: include/core/SkFontMetrics.h:76-82
    pub fn hasUnderlineThickness(&self, value: &mut f32) -> bool {
        if self.fFlags & Self::kUnderlineThicknessIsValid_Flag != 0 {
            *value = self.fUnderlineThickness;
            return true;
        }
        false
    }
    // Source: include/core/SkFontMetrics.h:91-97
    pub fn hasUnderlinePosition(&self, value: &mut f32) -> bool {
        if self.fFlags & Self::kUnderlinePositionIsValid_Flag != 0 {
            *value = self.fUnderlinePosition;
            return true;
        }
        false
    }
    // Source: include/core/SkFontMetrics.h:106-112
    pub fn hasStrikeoutThickness(&self, value: &mut f32) -> bool {
        if self.fFlags & Self::kStrikeoutThicknessIsValid_Flag != 0 {
            *value = self.fStrikeoutThickness;
            return true;
        }
        false
    }
    // Source: include/core/SkFontMetrics.h:121-127
    pub fn hasStrikeoutPosition(&self, value: &mut f32) -> bool {
        if self.fFlags & Self::kStrikeoutPositionIsValid_Flag != 0 {
            *value = self.fStrikeoutPosition;
            return true;
        }
        false
    }
    // Source: include/core/SkFontMetrics.h:134-137
    pub fn hasBounds(&self) -> bool {
        self.fFlags & Self::kBoundsInvalid_Flag == 0
    }
}
