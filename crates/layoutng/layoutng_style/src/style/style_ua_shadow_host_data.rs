use foundation::{AtomicString, Length, String, StyleAspectRatio};

// cpp: layoutng_style/style/style_ua_shadow_host_data.h:15-73
#[derive(Clone, PartialEq)]
pub struct StyleUAShadowHostData {
    width_: Length,
    height_: Length,
    max_width_: Length,
    max_height_: Length,
    aspect_ratio_: StyleAspectRatio,
    alt_text_: String,
    alt_attribute_: AtomicString,
    src_attribute_: AtomicString,
    has_appearance_: bool,
}

#[allow(non_snake_case)]
impl StyleUAShadowHostData {
    // cpp: layoutng_style/style/style_ua_shadow_host_data.h:22-39
    pub fn new(
        width: &Length,
        height: &Length,
        max_width: &Length,
        max_height: &Length,
        aspect_ratio: &StyleAspectRatio,
        alt_text: &String,
        alt_attribute: &AtomicString,
        src_attribute: &AtomicString,
        has_appearance: bool,
    ) -> Self {
        Self {
            width_: width.clone(),
            height_: height.clone(),
            max_width_: max_width.clone(),
            max_height_: max_height.clone(),
            aspect_ratio_: aspect_ratio.clone(),
            alt_text_: alt_text.clone(),
            alt_attribute_: alt_attribute.clone(),
            src_attribute_: src_attribute.clone(),
            has_appearance_: has_appearance,
        }
    }

    // cpp: layoutng_style/style/style_ua_shadow_host_data.h:41-43
    pub fn Clone(&self) -> Box<Self> {
        Box::new(Clone::clone(self))
    }

    // cpp: layoutng_style/style/style_ua_shadow_host_data.h:45-52
    pub fn Width(&self) -> &Length {
        &self.width_
    }
    pub fn Height(&self) -> &Length {
        &self.height_
    }
    pub fn MaxWidth(&self) -> &Length {
        &self.max_width_
    }
    pub fn MaxHeight(&self) -> &Length {
        &self.max_height_
    }
    pub fn AspectRatio(&self) -> &StyleAspectRatio {
        &self.aspect_ratio_
    }
    pub fn AltText(&self) -> &String {
        &self.alt_text_
    }
    pub fn AltAttribute(&self) -> &AtomicString {
        &self.alt_attribute_
    }
    pub fn SrcAttribute(&self) -> &AtomicString {
        &self.src_attribute_
    }
}

// cpp: layoutng_style/style/style_ua_shadow_host_data.h:54-61
// Derived equality compares all nine fields in the source's order.
