// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Native computed CSSImageValue. Fetching remains outside the Style owner.
#![allow(non_snake_case)]

use super::{
    css_url_data::CSSUrlData,
    css_value::{CSSValue, CSSValueClass},
};
use crate::style::style_image::StyleImage;
use foundation::{Member, Traceable, Visitor};

// cpp: core/css/css_image_value.h:37-102; css_image_value.cc:46-50.
#[repr(C)]
pub struct CSSImageValue {
    base: CSSValue,
    url_data: CSSUrlData,
    cached_image: Member<StyleImage>,
}

impl CSSImageValue {
    pub(crate) fn new(url_data: CSSUrlData, image: *mut StyleImage) -> Self {
        Self {
            base: CSSValue::new(CSSValueClass::Image),
            url_data,
            cached_image: Member::from_ptr(image),
        }
    }

    pub fn UrlData(&self) -> &CSSUrlData {
        &self.url_data
    }

    pub fn CachedImage(&self) -> *mut StyleImage {
        self.cached_image.Get()
    }
}

impl Traceable for CSSImageValue {
    // cpp: css_image_value.cc:177-182. URL strings own their immutable storage;
    // the cached image is a strong GC edge, not an untraced pointer or root.
    fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.cached_image);
    }
}
