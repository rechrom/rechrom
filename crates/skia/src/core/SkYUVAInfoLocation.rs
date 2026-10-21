// Copyright 2021 Google LLC. BSD-style license. UNVERIFIED native translation.
use super::{
    SkColor::SkColorChannel,
    SkYUVAInfo::{kMaxPlanes, kYUVAChannelCount, YUVAChannels, YUVALocations},
};
// Source: src/core/SkYUVAInfoLocation.h:20-24
#[derive(Clone, Copy, Debug)]
pub struct YUVALocation {
    pub fPlane: i32,
    pub fChannel: SkColorChannel,
}
impl Default for YUVALocation {
    fn default() -> Self {
        Self {
            fPlane: -1,
            fChannel: SkColorChannel::kA,
        }
    }
}
// Source: src/core/SkYUVAInfoLocation.h:26-29
impl PartialEq for YUVALocation {
    fn eq(&self, that: &Self) -> bool {
        self.fPlane == that.fPlane && self.fChannel == that.fChannel
    }
    fn ne(&self, that: &Self) -> bool {
        !self.eq(that)
    }
}
impl Eq for YUVALocation {}
impl YUVALocation {
    // Source: src/core/SkYUVAInfoLocation.h:31-60
    pub fn AreValidLocations(locations: &YUVALocations, numPlanes: Option<&mut i32>) -> bool {
        let mut maxSlotUsed = -1;
        let mut used = [false; kMaxPlanes];
        let mut valid = true;
        for i in 0..kYUVAChannelCount {
            if locations[i].fPlane < 0 {
                if i != YUVAChannels::kA as usize {
                    valid = false;
                }
            } else if locations[i].fPlane >= kMaxPlanes as i32 {
                valid = false;
            } else {
                maxSlotUsed = locations[i].fPlane.max(maxSlotUsed);
                // Preserve original channel-index i assignment, not a rewritten plane index.
                used[i] = true;
            }
        }
        for i in 0..=maxSlotUsed {
            if !used[i as usize] {
                valid = false;
            }
        }
        if let Some(num) = numPlanes {
            *num = if valid { maxSlotUsed + 1 } else { 0 };
        }
        valid
    }
}
