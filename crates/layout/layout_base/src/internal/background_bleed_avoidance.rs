// cpp: layoutng/internal/background_bleed_avoidance.h:10-15
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum BackgroundBleedAvoidance {
    kBackgroundBleedNone = 0,
    kBackgroundBleedShrinkBackground = 1,
    kBackgroundBleedClipOnly = 2,
    kBackgroundBleedClipLayer = 3,
}

// cpp: layoutng/internal/background_bleed_avoidance.h:17-20
#[allow(non_snake_case)]
pub fn BleedAvoidanceIsClipping(bleed_avoidance: BackgroundBleedAvoidance) -> bool {
    matches!(
        bleed_avoidance,
        BackgroundBleedAvoidance::kBackgroundBleedClipOnly
            | BackgroundBleedAvoidance::kBackgroundBleedClipLayer
    )
}
