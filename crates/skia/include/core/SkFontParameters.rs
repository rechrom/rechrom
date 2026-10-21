// Source: include/core/SkFontParameters.h:15-18
// The native SkFontParameters::Variation::Axis nested value type is hoisted.
// Rust requires the field group to be contiguous; field order is unchanged.
// Source: include/core/SkFontParameters.h:24-30
// Source: include/core/SkFontParameters.h:38-38
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Axis {
    pub tag: u32,
    pub min: f32,
    pub def: f32,
    pub max: f32,
    flags: u16,
}
impl Axis {
    // Source: include/core/SkFontParameters.h:19-19
    pub const fn new() -> Self {
        Self {
            tag: 0,
            min: 0.0,
            def: 0.0,
            max: 0.0,
            flags: 0,
        }
    }
    // Source: include/core/SkFontParameters.h:20-21
    pub const fn with_values(tag: u32, min: f32, def: f32, max: f32, hidden: bool) -> Self {
        Self {
            tag,
            min,
            def,
            max,
            flags: if hidden { Self::HIDDEN } else { 0 },
        }
    }
    // Source: include/core/SkFontParameters.h:32-32
    pub const fn isHidden(&self) -> bool {
        self.flags & Self::HIDDEN != 0
    }
    // Source: include/core/SkFontParameters.h:34-34
    pub fn setHidden(&mut self, hidden: bool) {
        self.flags = if hidden {
            self.flags | Self::HIDDEN
        } else {
            self.flags & !Self::HIDDEN
        };
    }
    // Source: include/core/SkFontParameters.h:36-36
    const HIDDEN: u16 = 0x0001;
}
impl Default for Axis {
    fn default() -> Self {
        Self::new()
    }
}
