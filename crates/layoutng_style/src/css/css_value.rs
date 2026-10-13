// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Class dispatch for the native computed CSS values carried by StyleImage.
#![allow(non_snake_case)]

use super::css_image_value::CSSImageValue;
use crate::style::generated_gradient_image::CSSGeneratedGradientValue;

#[repr(C)]
pub struct CSSValue {
    class_type: CSSValueClass,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CSSValueClass {
    Image,
    LinearGradient,
    RadialGradient,
    ConicGradient,
}

impl CSSValue {
    pub(crate) fn new(class_type: CSSValueClass) -> Self {
        Self { class_type }
    }

    pub(crate) fn gradient(&self) -> &CSSGeneratedGradientValue {
        assert!(self.IsImageGeneratorValue());
        // All native gradient allocations embed CSSValue first.
        unsafe { &*(self as *const Self).cast() }
    }

    pub fn AsImageValue(&self) -> Option<&CSSImageValue> {
        self.IsImageValue()
            .then(|| unsafe { &*(self as *const Self).cast() })
    }

    pub fn Hash(&self) -> u32 {
        if self.IsImageValue() {
            // cpp: css_value.cc:640-692. Image values use the source's
            // pointer-identity hash, independently of deep URL equality.
            self as *const Self as usize as u32
        } else {
            self.gradient().Data().Hash()
        }
    }

    pub fn IsImageValue(&self) -> bool {
        self.class_type == CSSValueClass::Image
    }

    pub fn IsImageGeneratorValue(&self) -> bool {
        !self.IsImageValue()
    }

    // cpp: css_value.cc:161-166. These native values admit images and
    // gradients; URL images are the only admitted class containing a URL.
    pub fn MayContainUrl(&self) -> bool {
        self.IsImageValue()
    }
}

impl PartialEq for CSSValue {
    fn eq(&self, other: &Self) -> bool {
        if self.class_type != other.class_type {
            return false;
        }
        match (self.AsImageValue(), other.AsImageValue()) {
            (Some(left), Some(right)) => left.UrlData() == right.UrlData(),
            _ => self.gradient().Data() == other.gradient().Data(),
        }
    }
}
