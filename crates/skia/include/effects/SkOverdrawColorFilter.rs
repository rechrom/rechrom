// Copyright 2016 Google Inc. BSD-style license.
// Source: include/effects/SkOverdrawColorFilter.h:26-26
pub struct SkOverdrawColorFilter;
impl SkOverdrawColorFilter {
    // Source: include/effects/SkOverdrawColorFilter.h:28-28
    pub const kNumColors: i32 = 6;
    // Source: include/effects/SkOverdrawColorFilter.h:30-30
    pub fn MakeWithSkColors(
        colors: &[crate::src::core::SkColor::SkColor; Self::kNumColors as usize],
    ) -> Option<std::sync::Arc<dyn crate::src::core::SkColorFilter::SkColorFilter>> {
        crate::src::effects::colorfilters::SkRuntimeColorFilter::MakeWithSkColors(colors)
    }
}
