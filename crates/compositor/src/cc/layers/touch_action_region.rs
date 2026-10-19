// Translation status: complete
// Copyright 2017 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/layers/touch_action_region.h:1-54.
// Source: cc/layers/touch_action_region.cc:1-69.

use crate::{
    adapter::{geometry::Rect, region_backend::SkRegionBackend},
    cc::{base::region::Region, input::touch_action::TouchAction},
};
use std::collections::BTreeMap;

// Source: cc/layers/touch_action_region.h:12-50.
#[derive(Clone, Default)]
pub struct TouchActionRegion<B: SkRegionBackend> {
    map_: BTreeMap<TouchAction, Region<B>>,
}

impl<B: SkRegionBackend> TouchActionRegion<B> {
    // Source: cc/layers/touch_action_region.h:17-20;
    //         cc/layers/touch_action_region.cc:13-19.
    pub fn new() -> Self {
        Self::default()
    }

    // Source: cc/layers/touch_action_region.h:22;
    //         cc/layers/touch_action_region.cc:28-30.
    pub fn Union(&mut self, touch_action: TouchAction, rect: &Rect) {
        self.map_.entry(touch_action).or_default().Union(rect);
    }

    // Source: cc/layers/touch_action_region.h:23-24;
    //         cc/layers/touch_action_region.cc:21-26.
    pub fn GetAllRegions(&self) -> Region<B> {
        let mut all_regions = Region::default();
        for region in self.map_.values() {
            all_regions.Union(region);
        }
        all_regions
    }

    // Source: cc/layers/touch_action_region.h:25;
    //         cc/layers/touch_action_region.cc:32-39.
    pub fn GetRegionForTouchAction(&self, touch_action: TouchAction) -> &Region<B> {
        self.map_
            .get(&touch_action)
            .unwrap_or_else(|| Region::<B>::Empty())
    }

    // Source: cc/layers/touch_action_region.h:27-34.
    pub fn iter(&self) -> impl Iterator<Item = (&TouchAction, &Region<B>)> {
        self.map_.iter()
    }
    pub fn IsEmpty(&self) -> bool {
        self.map_.is_empty()
    }

    // Source: cc/layers/touch_action_region.h:36-42;
    //         cc/layers/touch_action_region.cc:41-54.
    pub fn GetAllowedTouchAction(&self, touch_rect: &Rect) -> TouchAction {
        let mut allowed_touch_action = TouchAction::kAuto;
        let non_empty_touch_rect = Rect::new(
            touch_rect.x,
            touch_rect.y,
            touch_rect.width.max(1),
            touch_rect.height.max(1),
        );
        for (touch_action, region) in &self.map_ {
            if !region.Intersects(&non_empty_touch_rect) {
                continue;
            }
            allowed_touch_action &= *touch_action;
        }
        allowed_touch_action
    }
}

// Source: cc/layers/touch_action_region.h:44-46;
//         cc/layers/touch_action_region.cc:56-67.
impl<B: SkRegionBackend> PartialEq for TouchActionRegion<B> {
    fn eq(&self, other: &Self) -> bool {
        self.map_.len() == other.map_.len()
            && self.map_.iter().all(|(action, region)| {
                other
                    .map_
                    .get(action)
                    .is_some_and(|other_region| region.Equals(other_region))
            })
    }
}
impl<B: SkRegionBackend> Eq for TouchActionRegion<B> {}

// Source: cc/layers/touch_action_region.h:52-54;
//         cc/layers/touch_action_region.cc:69.
// Namespace and include-guard closure map to the Rust module boundary.
