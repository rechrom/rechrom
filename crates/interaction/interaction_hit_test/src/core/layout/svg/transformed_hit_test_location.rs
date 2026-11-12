// Copyright 2018 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/layout/svg/transformed_hit_test_location.h:1-56
// Source: third_party/blink/renderer/core/layout/svg/transformed_hit_test_location.cc:1-76
// Rust modules replace include guards, includes, namespaces and export macros.

use std::borrow::Cow;
use std::ops::Deref;

use foundation::blink_geometry::transforms::affine_transform::AffineTransform;
use foundation::{gfx, PhysicalRect};

use super::super::hit_test_location::HitTestLocation;

// Source: transformed_hit_test_location.h:33-37
#[derive(Clone, Copy, Debug)]
pub enum InverseTag {
    kDontComputeInverse,
}

// Source: transformed_hit_test_location.h:17-23,49-51
// Cow preserves the original location identity on the identity-transform path
// and owns source optional storage only when a transformed location is needed.
pub struct TransformedHitTestLocation<'a> {
    location_: Option<Cow<'a, HitTestLocation>>,
}

// Source: transformed_hit_test_location.cc:13-34
fn LocationTransformHelper(
    location: &HitTestLocation,
    transform: &AffineTransform,
) -> HitTestLocation {
    let transformed_point = transform.MapPoint(location.TransformedPoint());
    if location.IsRectBasedTest() {
        HitTestLocation::FromPointAndQuad(
            transformed_point,
            transform.MapQuad(*location.TransformedRect()),
        )
    } else {
        let mapped_rect = transform.MapRect(gfx::RectF::from(*location.BoundingBox()));
        if mapped_rect.width() < 1.0 || mapped_rect.height() < 1.0 {
            HitTestLocation::FromPointAndBoundingBox(
                transformed_point,
                PhysicalRect::EnclosingRect(&mapped_rect),
            )
        } else {
            HitTestLocation::FromPointF(transformed_point)
        }
    }
}

// Source: transformed_hit_test_location.cc:36-49
fn InverseTransformLocationIfNeeded<'a>(
    location: &'a HitTestLocation,
    transform: &AffineTransform,
) -> Option<Cow<'a, HitTestLocation>> {
    if transform.IsIdentity() {
        return Some(Cow::Borrowed(location));
    }
    if !transform.IsInvertible() {
        return None;
    }
    let inverse = transform.Inverse();
    Some(Cow::Owned(LocationTransformHelper(location, &inverse)))
}

// Source: transformed_hit_test_location.cc:51-60
fn TransformLocationIfNeeded<'a>(
    location: &'a HitTestLocation,
    transform: &AffineTransform,
) -> Cow<'a, HitTestLocation> {
    if transform.IsIdentity() {
        return Cow::Borrowed(location);
    }
    Cow::Owned(LocationTransformHelper(location, transform))
}

impl<'a> TransformedHitTestLocation<'a> {
    // Source: transformed_hit_test_location.h:26-31; .cc:64-68
    pub fn new(location: &'a HitTestLocation, transform: &AffineTransform) -> Self {
        Self {
            location_: InverseTransformLocationIfNeeded(location, transform),
        }
    }

    // Source: transformed_hit_test_location.h:33-37; .cc:70-74
    pub fn WithInverseTag(
        location: &'a HitTestLocation,
        transform: &AffineTransform,
        _: InverseTag,
    ) -> Self {
        Self {
            location_: Some(TransformLocationIfNeeded(location, transform)),
        }
    }

    // Source: transformed_hit_test_location.h:47
    pub fn IsValid(&self) -> bool {
        self.location_.is_some()
    }

    // Source: transformed_hit_test_location.h:39-46
    pub fn Location(&self) -> &HitTestLocation {
        self.location_
            .as_deref()
            .expect("singular hit-test transform")
    }
}

// Source: transformed_hit_test_location.h:39-46
impl Deref for TransformedHitTestLocation<'_> {
    type Target = HitTestLocation;
    fn deref(&self) -> &Self::Target {
        self.Location()
    }
}
