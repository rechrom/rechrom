// Translation status: complete
// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/trees/occlusion.h:1-44
// Source: cc/trees/occlusion.cc:1-103
use crate::{
    adapter::{geometry::Rect, occlusion::OcclusionTransform},
    cc::base::{math_util::MathUtil, simple_enclosed_region::SimpleEnclosedRegion},
};

// Source: cc/trees/occlusion.h:17-39
// Source: cc/trees/occlusion.cc:13
#[derive(Clone, Default)]
pub struct Occlusion<T: OcclusionTransform> {
    draw_transform_: T,
    occlusion_from_outside_target_: SimpleEnclosedRegion,
    occlusion_from_inside_target_: SimpleEnclosedRegion,
}
impl<T: OcclusionTransform> Occlusion<T> {
    // Source: cc/trees/occlusion.cc:15-21
    pub fn new(
        draw_transform: &T,
        occlusion_from_outside_target: SimpleEnclosedRegion,
        occlusion_from_inside_target: SimpleEnclosedRegion,
    ) -> Self {
        Self {
            draw_transform_: draw_transform.clone(),
            occlusion_from_outside_target_: occlusion_from_outside_target,
            occlusion_from_inside_target_: occlusion_from_inside_target,
        }
    }
    // Source: cc/trees/occlusion.cc:23-27
    pub fn GetOcclusionWithGivenDrawTransform(&self, transform: &T) -> Self {
        Self::new(
            transform,
            self.occlusion_from_outside_target_,
            self.occlusion_from_inside_target_,
        )
    }
    // Source: cc/trees/occlusion.cc:29-32
    pub fn HasOcclusion(&self) -> bool {
        !self.occlusion_from_inside_target_.IsEmpty()
            || !self.occlusion_from_outside_target_.IsEmpty()
    }
    // Source: cc/trees/occlusion.cc:34-44
    pub fn IsOccluded(&self, content_rect: Rect) -> bool {
        if content_rect.IsEmpty() {
            return true;
        }
        if !self.HasOcclusion() {
            return false;
        }
        let unoccluded_rect_in_target_surface = self.GetUnoccludedRectInTargetSurface(content_rect);
        unoccluded_rect_in_target_surface.IsEmpty()
    }
    // Source: cc/trees/occlusion.cc:46-71
    pub fn GetUnoccludedContentRect(&self, content_rect: Rect) -> Rect {
        if content_rect.IsEmpty() {
            return content_rect;
        }
        if !self.HasOcclusion() {
            return content_rect;
        }
        let unoccluded_rect_in_target_surface = self.GetUnoccludedRectInTargetSurface(content_rect);
        if unoccluded_rect_in_target_surface.IsEmpty() {
            return Rect::default();
        }
        let mut inverse_draw_transform = T::default();
        let ok = self.draw_transform_.GetInverse(&mut inverse_draw_transform);
        if !ok {
            return content_rect;
        }
        let mut unoccluded_rect = MathUtil::ProjectEnclosingClippedRect(
            &inverse_draw_transform,
            unoccluded_rect_in_target_surface,
        );
        unoccluded_rect.Intersect(content_rect);
        unoccluded_rect
    }
    // Source: cc/trees/occlusion.cc:73-89
    fn GetUnoccludedRectInTargetSurface(&self, content_rect: Rect) -> Rect {
        let mut unoccluded_rect_in_target_surface =
            MathUtil::MapEnclosingClippedRect(&self.draw_transform_, content_rect);
        debug_assert!(self.occlusion_from_inside_target_.GetRegionComplexity() <= 1);
        debug_assert!(self.occlusion_from_outside_target_.GetRegionComplexity() <= 1);
        unoccluded_rect_in_target_surface.Subtract(self.occlusion_from_inside_target_.bounds());
        unoccluded_rect_in_target_surface.Subtract(self.occlusion_from_outside_target_.bounds());
        unoccluded_rect_in_target_surface
    }
    // Source: cc/trees/occlusion.cc:91-95
    pub fn IsEqual(&self, other: &Self) -> bool {
        self.draw_transform_ == other.draw_transform_
            && self.occlusion_from_inside_target_ == other.occlusion_from_inside_target_
            && self.occlusion_from_outside_target_ == other.occlusion_from_outside_target_
    }
    // Source: cc/trees/occlusion.cc:97-101
    pub fn ToString(&self) -> String {
        self.draw_transform_.ToString()
            + "outside("
            + &self.occlusion_from_outside_target_.ToString()
            + ") inside("
            + &self.occlusion_from_inside_target_.ToString()
            + ")"
    }
}
