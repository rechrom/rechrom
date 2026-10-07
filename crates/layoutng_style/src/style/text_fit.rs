use foundation::{RuntimeEnabledFeatures, String};

// cpp: layoutng_style/style/text_fit.h:13-17
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum TextFitType {
    #[default]
    kNone,
    kGrow,
    kShrink,
}

// cpp: layoutng_style/style/text_fit.h:19-23
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum TextFitTarget {
    #[default]
    kConsistent,
    kPerLine,
    kPerLineAll,
}

// cpp: layoutng_style/style/text_fit.h:25-28
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum TextFitMethod {
    kScale,
    kFontSize,
}

// cpp: layoutng_style/style/text_fit.h:30-56
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextFit {
    type_: TextFitType,
    target_: TextFitTarget,
    scale_factor_limit_: Option<f32>,
}

#[allow(non_snake_case)]
impl TextFit {
    // cpp: layoutng_style/style/text_fit.h:35-37
    pub fn new(type_: TextFitType, target: TextFitTarget, limit: Option<f32>) -> Self {
        Self {
            type_,
            target_: target,
            scale_factor_limit_: limit,
        }
    }

    // cpp: layoutng_style/style/text_fit.h:43-44
    pub fn Type(&self) -> TextFitType {
        self.type_
    }
    pub fn Target(&self) -> TextFitTarget {
        self.target_
    }

    // cpp: layoutng_style/style/text_fit.h:45
    // cpp: layoutng_style/style/text_fit.cc:10-14
    pub fn Method(&self) -> TextFitMethod {
        if RuntimeEnabledFeatures::CssTextFitReshapingEnabled() {
            TextFitMethod::kFontSize
        } else {
            TextFitMethod::kScale
        }
    }

    // cpp: layoutng_style/style/text_fit.h:46-47
    pub fn ScaleFactorLimit(&self) -> Option<f32> {
        self.scale_factor_limit_
    }

    // cpp: layoutng_style/style/text_fit.h:49-50
    // Declared but not defined in the supplied source tree.
    pub fn ToString(&self) -> String {
        unsafe { TextFitToString(self) }
    }
}

unsafe extern "Rust" {
    fn TextFitToString(value: &TextFit) -> String;
}
