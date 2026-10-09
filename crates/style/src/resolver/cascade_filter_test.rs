// Copyright 2020 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// cpp: third_party/blink/renderer/core/css/resolver/cascade_filter_test.cc

use super::*;
use foundation::CSSPropertyID;

fn property(id: CSSPropertyID) -> &'static CSSProperty {
    CSSProperty::Get(id)
}

// cpp: cascade_filter_test.cc:13-20
#[test]
fn FilterNothing() {
    let filter = CascadeFilter::new();
    for id in [
        CSSPropertyID::kBackgroundColor,
        CSSPropertyID::kColor,
        CSSPropertyID::kDisplay,
        CSSPropertyID::kFloat,
        CSSPropertyID::kInternalVisitedColor,
    ] {
        assert!(filter.Accepts(property(id)));
    }
}

// cpp: cascade_filter_test.cc:22-25
#[test]
fn ConstructorBehavesLikeSingleAdd() {
    assert_eq!(
        CascadeFilter::new().Add(CSSProperty::kInherited),
        CascadeFilter::FromFlag(CSSProperty::kInherited),
    );
}

// cpp: cascade_filter_test.cc:27-30
#[test]
fn Equals() {
    assert_eq!(
        CascadeFilter::FromFlag(CSSProperty::kInherited),
        CascadeFilter::FromFlag(CSSProperty::kInherited),
    );
}

// cpp: cascade_filter_test.cc:32-38
#[test]
fn NotEqualsMask() {
    assert_ne!(
        CascadeFilter::FromFlag(CSSProperty::kInherited),
        CascadeFilter::FromFlag(CSSProperty::kVisited)
    );
    assert_ne!(
        CascadeFilter::FromFlag(CSSProperty::kInherited),
        CascadeFilter::FromFlag(CSSProperty::kInherited).Add(CSSProperty::kVisited)
    );
    assert_ne!(
        CascadeFilter::FromFlag(CSSProperty::kInherited),
        CascadeFilter::new()
    );
}

// cpp: cascade_filter_test.cc:40-48
#[test]
fn FilterNonInherited() {
    let filter = CascadeFilter::FromFlag(CSSProperty::kInherited);
    assert!(!filter.Accepts(property(CSSPropertyID::kBackgroundColor)));
    assert!(filter.Accepts(property(CSSPropertyID::kColor)));
    assert!(!filter.Accepts(property(CSSPropertyID::kDisplay)));
    assert!(!filter.Accepts(property(CSSPropertyID::kFloat)));
    assert!(filter.Accepts(property(CSSPropertyID::kInternalVisitedColor)));
}

// cpp: cascade_filter_test.cc:50-58
#[test]
fn FilterVisitedAndNonInherited() {
    let filter = CascadeFilter::new()
        .Add(CSSProperty::kNotVisited)
        .Add(CSSProperty::kInherited);
    assert!(!filter.Accepts(property(CSSPropertyID::kBackgroundColor)));
    assert!(filter.Accepts(property(CSSPropertyID::kColor)));
    assert!(!filter.Accepts(property(CSSPropertyID::kDisplay)));
    assert!(!filter.Accepts(property(CSSPropertyID::kFloat)));
    assert!(!filter.Accepts(property(CSSPropertyID::kInternalVisitedColor)));
}

// cpp: cascade_filter_test.cc:60-66
#[test]
fn RejectFlag() {
    let filter = CascadeFilter::new()
        .Add(CSSProperty::kVisited)
        .Add(CSSProperty::kInherited);
    assert!(filter.Requires(CSSProperty::kVisited));
    assert!(filter.Requires(CSSProperty::kInherited));
    assert!(!filter.Requires(CSSProperty::kNotVisited));
}

// cpp: cascade_filter_test.cc:68-86
#[test]
fn FilterLegacyOverlapping() {
    let filter = CascadeFilter::new().Add(CSSProperty::kNotLegacyOverlapping);
    for id in [
        CSSPropertyID::kWebkitTransformOriginX,
        CSSPropertyID::kWebkitTransformOriginY,
        CSSPropertyID::kWebkitTransformOriginZ,
        CSSPropertyID::kWebkitPerspectiveOriginX,
        CSSPropertyID::kWebkitPerspectiveOriginY,
        CSSPropertyID::kWebkitBorderImage,
    ] {
        assert!(!filter.Accepts(property(id)), "{id:?}");
    }
    for id in [
        CSSPropertyID::kTransformOrigin,
        CSSPropertyID::kPerspectiveOrigin,
        CSSPropertyID::kBorderImageSource,
        CSSPropertyID::kBorderImageSlice,
        CSSPropertyID::kBorderImageRepeat,
        CSSPropertyID::kBorderImageWidth,
        CSSPropertyID::kBorderImageOutset,
        CSSPropertyID::kColor,
        CSSPropertyID::kFloat,
    ] {
        assert!(filter.Accepts(property(id)), "{id:?}");
    }
}

// cpp: cascade_filter_test.cc:88-106
#[test]
fn FilterOverlapping() {
    let filter = CascadeFilter::new().Add(CSSProperty::kOverlapping);
    for id in [
        CSSPropertyID::kWebkitTransformOriginX,
        CSSPropertyID::kWebkitTransformOriginY,
        CSSPropertyID::kWebkitTransformOriginZ,
        CSSPropertyID::kWebkitPerspectiveOriginX,
        CSSPropertyID::kWebkitPerspectiveOriginY,
        CSSPropertyID::kWebkitBorderImage,
        CSSPropertyID::kTransformOrigin,
        CSSPropertyID::kPerspectiveOrigin,
        CSSPropertyID::kBorderImageSource,
        CSSPropertyID::kBorderImageSlice,
        CSSPropertyID::kBorderImageRepeat,
        CSSPropertyID::kBorderImageWidth,
        CSSPropertyID::kBorderImageOutset,
    ] {
        assert!(filter.Accepts(property(id)), "{id:?}");
    }
    assert!(!filter.Accepts(property(CSSPropertyID::kColor)));
    assert!(!filter.Accepts(property(CSSPropertyID::kFloat)));
}

// Additional verification of cascade_filter.h:59-67, using the mapped
// CSSProperty constructor rather than inventing generated longhand instances.
#[test]
fn RequiresAllBitsIncludingHighFlags() {
    let flags = CSSProperty::kInherited
        | CSSProperty::kNotVisited
        | CSSProperty::kNotAnimation
        | CSSProperty::kNotLegacyOverlapping;
    let property = CSSProperty::new(CSSPropertyID::kColor, flags, '\0');
    assert!(CascadeFilter::new().IsEmpty());
    assert!(CascadeFilter::new().Accepts(&property));
    let filter = CascadeFilter::FromFlag(CSSProperty::kInherited).Add(CSSProperty::kNotVisited);
    assert!(filter.Accepts(&property));
    assert_eq!(filter, filter.Add(CSSProperty::kInherited));
    assert!(!filter.IsEmpty());
    assert!(!filter.Add(CSSProperty::kVisited).Accepts(&property));
}
