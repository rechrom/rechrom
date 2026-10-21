// Native nested value types are hoisted to module scope. Raw pointer/count pairs
// retain their source representation and do not copy the caller's arrays.
// Consumers must retain valid coordinate/palette backing through each use.

// Source: include/core/SkFontArguments.h:21-29
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Coordinate {
    pub axis: u32,
    pub value: f32,
}
impl Coordinate {
    pub const wght: u32 = u32::from_be_bytes(*b"wght");
    pub const wdth: u32 = u32::from_be_bytes(*b"wdth");
    pub const slnt: u32 = u32::from_be_bytes(*b"slnt");
    pub const ital: u32 = u32::from_be_bytes(*b"ital");
    pub const opsz: u32 = u32::from_be_bytes(*b"opsz");
}

// Source: include/core/SkFontArguments.h:20-20
// Source: include/core/SkFontArguments.h:30-32
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VariationPosition {
    pub coordinates: *const Coordinate,
    pub coordinateCount: i32,
}

// Source: include/core/SkFontArguments.h:42-45
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PaletteOverride {
    pub index: u16,
    pub color: u32,
}

// Source: include/core/SkFontArguments.h:41-41
// Source: include/core/SkFontArguments.h:46-49
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Palette {
    pub index: i32,
    pub overrides: *const PaletteOverride,
    pub overrideCount: i32,
}

// Source: include/core/SkFontArguments.h:19-19
// Source: include/core/SkFontArguments.h:109-114
// Rust fields are grouped in source field order; Option<bool> represents the
// native optional value, without claiming C++ std::optional ABI identity.
#[derive(Clone, Copy)]
pub struct SkFontArguments {
    fCollectionIndex: i32,
    fVariationDesignPosition: VariationPosition,
    fPalette: Palette,
    fSyntheticBold: Option<bool>,
    fSyntheticOblique: Option<bool>,
}

impl SkFontArguments {
    // Source: include/core/SkFontArguments.h:51-54
    pub const fn new() -> Self {
        Self {
            fCollectionIndex: 0,
            fVariationDesignPosition: VariationPosition {
                coordinates: core::ptr::null(),
                coordinateCount: 0,
            },
            fPalette: Palette {
                index: 0,
                overrides: core::ptr::null(),
                overrideCount: 0,
            },
            fSyntheticBold: None,
            fSyntheticOblique: None,
        }
    }
    // Source: include/core/SkFontArguments.h:61-64
    pub fn setCollectionIndex(&mut self, collection_index: i32) -> &mut Self {
        self.fCollectionIndex = collection_index;
        self
    }
    // Source: include/core/SkFontArguments.h:73-77
    pub fn setVariationDesignPosition(&mut self, position: VariationPosition) -> &mut Self {
        self.fVariationDesignPosition.coordinates = position.coordinates;
        self.fVariationDesignPosition.coordinateCount = position.coordinateCount;
        self
    }
    // Source: include/core/SkFontArguments.h:79-81
    pub const fn getCollectionIndex(&self) -> i32 {
        self.fCollectionIndex
    }
    // Source: include/core/SkFontArguments.h:83-85
    pub const fn getVariationDesignPosition(&self) -> VariationPosition {
        self.fVariationDesignPosition
    }
    // Source: include/core/SkFontArguments.h:87-92
    pub fn setPalette(&mut self, palette: Palette) -> &mut Self {
        self.fPalette.index = palette.index;
        self.fPalette.overrides = palette.overrides;
        self.fPalette.overrideCount = palette.overrideCount;
        self
    }
    // Source: include/core/SkFontArguments.h:94-94
    pub const fn getPalette(&self) -> Palette {
        self.fPalette
    }
    // Source: include/core/SkFontArguments.h:96-99
    pub fn setSyntheticBold(&mut self, bold: Option<bool>) -> &mut Self {
        self.fSyntheticBold = bold;
        self
    }
    // Source: include/core/SkFontArguments.h:100-100
    pub const fn getSyntheticBold(&self) -> Option<bool> {
        self.fSyntheticBold
    }
    // Source: include/core/SkFontArguments.h:102-105
    pub fn setSyntheticOblique(&mut self, oblique: Option<bool>) -> &mut Self {
        self.fSyntheticOblique = oblique;
        self
    }
    // Source: include/core/SkFontArguments.h:106-106
    pub const fn getSyntheticOblique(&self) -> Option<bool> {
        self.fSyntheticOblique
    }
}
impl Default for SkFontArguments {
    fn default() -> Self {
        Self::new()
    }
}
