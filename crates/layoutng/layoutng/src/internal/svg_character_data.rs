// cpp: layoutng/internal/svg_character_data.h:29-50
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SvgCharacterData {
    pub x: f32,
    pub y: f32,
    pub dx: f32,
    pub dy: f32,
    pub rotate: f32,
    pub anchored_chunk: bool,
}

impl Default for SvgCharacterData {
    fn default() -> Self {
        Self {
            x: Self::EmptyValue(),
            y: Self::EmptyValue(),
            dx: Self::EmptyValue(),
            dy: Self::EmptyValue(),
            rotate: Self::EmptyValue(),
            anchored_chunk: false,
        }
    }
}

#[allow(non_snake_case)]
impl SvgCharacterData {
    // cpp: layoutng/internal/svg_character_data.h:33-38
    pub const fn EmptyValue() -> f32 {
        f32::NAN
    }

    pub fn IsEmptyValue(value: f32) -> bool {
        value.is_nan()
    }

    // cpp: layoutng/internal/svg_character_data.h:40-44
    pub fn HasX(&self) -> bool {
        !Self::IsEmptyValue(self.x)
    }
    pub fn HasY(&self) -> bool {
        !Self::IsEmptyValue(self.y)
    }
    pub fn HasDx(&self) -> bool {
        !Self::IsEmptyValue(self.dx)
    }
    pub fn HasDy(&self) -> bool {
        !Self::IsEmptyValue(self.dy)
    }
    pub fn HasRotate(&self) -> bool {
        !Self::IsEmptyValue(self.rotate)
    }
}

// cpp: layoutng/internal/svg_character_data.h:52-52
// The stream insertion operator is declared but not defined in the source tree.
