// Copyright 2019 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// cpp: third_party/blink/renderer/core/css/properties/css_exposure.h:9-28

// Describes whether a property is exposed to author/user style sheets,
// UA style sheets, or not at all.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CSSExposure {
    // The property can't be used anywhere, i.e. it's disabled.
    kNone,
    // The property may be used in UA stylesheets, but not in author and user
    // stylesheets, and is otherwise not visible to the author.
    kUA,
    // The property is web exposed and available everywhere.
    kWeb,
}

pub fn IsUAExposed(exposure: CSSExposure) -> bool {
    exposure >= CSSExposure::kUA
}

pub fn IsWebExposed(exposure: CSSExposure) -> bool {
    exposure == CSSExposure::kWeb
}

// cpp: third_party/blink/renderer/core/css/properties/css_exposure_test.cc:10-20
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn IsUAExposedTest() {
        assert!(!IsUAExposed(CSSExposure::kNone));
        assert!(IsUAExposed(CSSExposure::kUA));
        assert!(IsUAExposed(CSSExposure::kWeb));
    }

    #[test]
    fn IsWebExposedTest() {
        assert!(!IsWebExposed(CSSExposure::kNone));
        assert!(!IsWebExposed(CSSExposure::kUA));
        assert!(IsWebExposed(CSSExposure::kWeb));
    }
}
