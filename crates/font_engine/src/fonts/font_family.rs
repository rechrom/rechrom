// C++: font_engine/fonts/font_family.h/.cc.
use foundation::AtomicString;
use std::ops::Deref;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// cpp: font_engine/fonts/font_family.h:43-44
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontFamilyType {
    kFamilyName,
    kGenericFamily,
}

// cpp: font_engine/fonts/font_family.h:39-95
#[derive(Debug)]
pub struct FontFamily {
    family_name_: AtomicString,
    next_: Option<Arc<SharedFontFamily>>,
    family_type_: FontFamilyType,
    is_prewarmed_: AtomicBool,
}

impl Default for FontFamily {
    fn default() -> Self {
        Self {
            family_name_: AtomicString::default(),
            next_: None,
            family_type_: FontFamilyType::kFamilyName,
            is_prewarmed_: AtomicBool::new(false),
        }
    }
}

impl Clone for FontFamily {
    fn clone(&self) -> Self {
        Self {
            family_name_: self.family_name_.clone(),
            next_: self.next_.clone(),
            family_type_: self.family_type_,
            is_prewarmed_: AtomicBool::new(self.IsPrewarmed()),
        }
    }
}

#[allow(non_snake_case)]
impl FontFamily {
    // cpp: font_engine/fonts/font_family.h:46-51
    pub fn new(
        family_name: AtomicString,
        family_type: FontFamilyType,
        next: Option<Arc<SharedFontFamily>>,
    ) -> Self {
        Self {
            family_name_: family_name,
            next_: next,
            family_type_: family_type,
            is_prewarmed_: AtomicBool::new(false),
        }
    }

    // cpp: font_engine/fonts/font_family.h:58-62
    pub fn FamilyName(&self) -> &AtomicString {
        &self.family_name_
    }
    pub fn FamilyIsGeneric(&self) -> bool {
        self.family_type_ == FontFamilyType::kGenericFamily
    }

    // cpp: font_engine/fonts/font_family.h:121-128
    pub fn Next(&self) -> *const FontFamily {
        self.next_
            .as_deref()
            .map_or(std::ptr::null(), |next| &next.family as *const FontFamily)
    }
    pub fn ReleaseNext(&mut self) -> Option<Arc<SharedFontFamily>> {
        self.next_.take()
    }

    // cpp: font_engine/fonts/font_family.h:66-67
    pub fn IsPrewarmed(&self) -> bool {
        self.is_prewarmed_.load(Ordering::Relaxed)
    }
    pub fn SetIsPrewarmed(&self) {
        self.is_prewarmed_.store(true, Ordering::Relaxed);
    }

    // cpp: font_engine/fonts/font_family.cc:22-30
    pub fn ToString(&self) -> String {
        let mut result = self.FamilyName().Utf8();
        let mut family = self.next_.as_deref();
        while let Some(current) = family {
            result.push_str(", ");
            result.push_str(&current.FamilyName().Utf8());
            family = current.family.next_.as_deref();
        }
        result
    }
}

// cpp: font_engine/fonts/font_family.h:114-119
impl Drop for FontFamily {
    fn drop(&mut self) {
        let mut reaper = self.next_.take();
        while let Some(shared) = reaper {
            match Arc::try_unwrap(shared) {
                Ok(mut node) => reaper = node.family.ReleaseNext(),
                Err(_) => break,
            }
        }
    }
}

// cpp: font_engine/fonts/font_family.cc:7-20
impl PartialEq for FontFamily {
    fn eq(&self, other: &Self) -> bool {
        if self.FamilyIsGeneric() != other.FamilyIsGeneric()
            || self.FamilyName() != other.FamilyName()
        {
            return false;
        }
        let mut left = self.next_.as_deref().map(|node| &node.family);
        let mut right = other.next_.as_deref().map(|node| &node.family);
        loop {
            match (left, right) {
                (None, None) => return true,
                (Some(a), Some(b)) if std::ptr::eq(a, b) => return true,
                (Some(a), Some(b))
                    if a.FamilyIsGeneric() == b.FamilyIsGeneric()
                        && a.FamilyName() == b.FamilyName() =>
                {
                    left = a.next_.as_deref().map(|node| &node.family);
                    right = b.next_.as_deref().map(|node| &node.family);
                }
                _ => return false,
            }
        }
    }
}
impl Eq for FontFamily {}

// cpp: font_engine/fonts/font_family.h:97-111
#[derive(Debug)]
pub struct SharedFontFamily {
    family: FontFamily,
}

#[allow(non_snake_case)]
impl SharedFontFamily {
    pub fn Create(
        family_name: AtomicString,
        family_type: FontFamilyType,
        next: Option<Arc<SharedFontFamily>>,
    ) -> Arc<Self> {
        Arc::new(Self {
            family: FontFamily::new(family_name, family_type, next),
        })
    }
}

impl Deref for SharedFontFamily {
    type Target = FontFamily;
    fn deref(&self) -> &Self::Target {
        &self.family
    }
}
