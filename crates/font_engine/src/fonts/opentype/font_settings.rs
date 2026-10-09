// C++: font_engine/fonts/opentype/font_settings.h.
// Four-byte OpenType tag conversion follows Chromium font_settings.cc.
// FontVariationSettings::GetHash remains outside this translated slice.
use std::sync::Arc;

// cpp: font_engine/fonts/opentype/font_settings.h:21-60
#[derive(Clone, Copy, Debug)]
pub struct FontTagValuePair<T> {
    tag: u32,
    value: T,
}

impl<T> FontTagValuePair<T> {
    pub fn new(tag: u32, value: T) -> Self {
        debug_assert!(tag == 0 || valid_tag(tag));
        Self { tag, value }
    }

    pub fn Tag(&self) -> u32 {
        self.tag
    }
}

impl<T: Copy> FontTagValuePair<T> {
    pub fn Value(&self) -> T {
        self.value
    }
}

impl<T: PartialEq> PartialEq for FontTagValuePair<T> {
    fn eq(&self, other: &Self) -> bool {
        self.tag == other.tag && self.value == other.value
    }
}

impl<T: Eq> Eq for FontTagValuePair<T> {}

impl<T> FontTagValuePair<T> {
    // C++ operator< compares tags only, even when values differ.
    pub fn LessThan(&self, other: &Self) -> bool {
        self.tag < other.tag
    }
}

fn valid_tag(tag: u32) -> bool {
    tag.to_be_bytes()
        .iter()
        .all(|byte| (0x20..0x7f).contains(byte))
}

// cpp: font_engine/fonts/opentype/font_settings.h:63-109
#[derive(Debug)]
pub struct FontSettings<T> {
    list: Vec<T>,
}

impl<T> Default for FontSettings<T> {
    fn default() -> Self {
        Self { list: Vec::new() }
    }
}

impl<T> FontSettings<T> {
    pub fn Append(&mut self, feature: T) {
        self.list.push(feature);
    }

    pub fn size(&self) -> u32 {
        u32::try_from(self.list.len()).expect("font settings exceed wtf_size_t")
    }

    pub fn at(&self, index: u32) -> &T {
        &self.list[index as usize]
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.list.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.list.iter_mut()
    }
}

impl<T: Copy + FontSettingTag> FontSettings<T> {
    pub fn FindPair(&self, tag: u32, found_pair: Option<&mut T>) -> bool {
        let Some(found_pair) = found_pair else {
            return false;
        };
        for pair in &self.list {
            if pair.tag() == tag {
                *found_pair = *pair;
                return true;
            }
        }
        false
    }
}

impl<T: PartialEq> PartialEq for FontSettings<T> {
    fn eq(&self, other: &Self) -> bool {
        self.list == other.list
    }
}

pub trait FontSettingTag {
    fn tag(&self) -> u32;
}

impl<T> FontSettingTag for FontTagValuePair<T> {
    fn tag(&self) -> u32 {
        self.tag
    }
}

pub type FontFeature = FontTagValuePair<i32>;
pub type FontVariationAxis = FontTagValuePair<f32>;

// cpp: font_engine/fonts/opentype/font_settings.h:111-145
pub type FontFeatureSettings = Arc<FontSettings<FontFeature>>;
pub type FontVariationSettings = Arc<FontSettings<FontVariationAxis>>;

pub fn CreateFontFeatureSettings() -> FontFeatureSettings {
    Arc::new(FontSettings::default())
}

pub fn CreateFontVariationSettings() -> FontVariationSettings {
    Arc::new(FontSettings::default())
}

// cpp: platform/fonts/opentype/font_settings.cc:18-22.
pub fn AtomicStringToFourByteTag(tag: &foundation::AtomicString) -> u32 {
    let units = tag.utf16_units().expect("OpenType tag is non-null");
    assert_eq!(units.len(), 4);
    ((units[0] as u32) << 24)
        | ((units[1] as u32) << 16)
        | ((units[2] as u32) << 8)
        | units[3] as u32
}
