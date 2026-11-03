//! Pre-cached container dimensions retained by generated style images.
//!
//! Chromium's full `CSSToLengthConversionData::ContainerSizes` can lazily walk
//! an `Element`. The Style package boundary receives the pre-cached copy, so it
//! retains only the two values which `StyleGeneratedImage` observes.

use foundation::{Traceable, Visitor};

// cpp: core/css/css_to_length_conversion_data.h:223-263
#[derive(Clone, Copy, Debug, Default)]
pub struct ContainerSizes {
    cached_width: Option<f64>,
    cached_height: Option<f64>,
}

#[allow(non_snake_case)]
impl ContainerSizes {
    pub fn PreCached(width: Option<f64>, height: Option<f64>) -> Self {
        Self {
            cached_width: width,
            cached_height: height,
        }
    }

    // cpp: core/css/css_to_length_conversion_data.cc:248-258
    pub fn PreCachedCopy(&self) -> Self {
        *self
    }

    // cpp: core/css/css_to_length_conversion_data.cc:264-267
    pub fn SizesEqual(&self, other: &Self) -> bool {
        self.Width() == other.Width() && self.Height() == other.Height()
    }

    // cpp: core/css/css_to_length_conversion_data.cc:269-278
    pub fn Width(&self) -> Option<f64> {
        self.cached_width
    }

    pub fn Height(&self) -> Option<f64> {
        self.cached_height
    }

    // cpp: core/css/css_to_length_conversion_data.cc:260-262
    pub fn Trace(&self, _: &mut Visitor) {}
}

impl Traceable for ContainerSizes {
    fn Trace(&self, visitor: &mut Visitor) {
        ContainerSizes::Trace(self, visitor);
    }
}
