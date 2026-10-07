// C++: font_engine/fonts/shaping/font_features.h. The template
// FromFontDescription declaration has no definition in the supplied tree.
use std::ptr::NonNull;

// cpp: font_engine/fonts/shaping/font_features.h:21-34
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FontFeatureTag {
    pub tag: u32,
}

impl FontFeatureTag {
    pub const fn new(tag: u32) -> Self {
        Self { tag }
    }
    pub const fn from_chars(c1: u8, c2: u8, c3: u8, c4: u8) -> Self {
        Self {
            tag: (((((c1 as u32) << 8) | c2 as u32) << 8 | c3 as u32) << 8) | c4 as u32,
        }
    }
}

impl From<FontFeatureTag> for u32 {
    fn from(value: FontFeatureTag) -> Self {
        value.tag
    }
}

// cpp: font_engine/fonts/shaping/font_features.h:39-42
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FontFeatureValue {
    pub tag: u32,
    pub value: u32,
}

// Matches the four uint32_t fields of hb_feature_t.
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HarfBuzzFeature {
    pub tag: u32,
    pub value: u32,
    pub start: u32,
    pub end: u32,
}

// cpp: font_engine/fonts/shaping/font_features.h:47-68
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FontFeatureRange {
    pub tag: u32,
    pub value: u32,
    pub start: u32,
    pub end: u32,
}

impl Default for FontFeatureRange {
    fn default() -> Self {
        Self {
            tag: 0,
            value: 0,
            start: 0,
            end: u32::MAX,
        }
    }
}

impl FontFeatureRange {
    pub const kInitialSize: u32 = 1;
    pub const fn ToHarfBuzzData(features: *const Self) -> *const HarfBuzzFeature {
        features as *const HarfBuzzFeature
    }
}

const _: () =
    assert!(std::mem::size_of::<FontFeatureRange>() == std::mem::size_of::<HarfBuzzFeature>());
const _: () =
    assert!(std::mem::align_of::<FontFeatureRange>() == std::mem::align_of::<HarfBuzzFeature>());

// cpp: font_engine/fonts/shaping/font_features.h:72-73
// Rust Vec owns its backing; C++ inline capacity 6 is not a semantic field.
pub type FontFeatureRanges = Vec<FontFeatureRange>;

// cpp: font_engine/fonts/shaping/font_features.h:79-106
pub struct FontFeatureRangesSaver {
    features: NonNull<FontFeatureRanges>,
    num_features_before: usize,
}

impl FontFeatureRangesSaver {
    /// # Safety
    /// The vector must outlive the saver and remain at a stable address.
    pub unsafe fn new(features: &mut FontFeatureRanges) -> Self {
        Self {
            features: NonNull::from(&mut *features),
            num_features_before: features.len(),
        }
    }
}

impl Drop for FontFeatureRangesSaver {
    fn drop(&mut self) {
        let features = unsafe { self.features.as_mut() };
        if features.len() > self.num_features_before {
            features.truncate(self.num_features_before);
        }
    }
}
