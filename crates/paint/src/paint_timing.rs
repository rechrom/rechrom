//! PaintTiming's synchronous rendered milestones, Chromium paint_timing.cc:
//! 125-187,215-231,505-546. Presentation/PerformanceEntry delivery is separate.
use crate::PaintArtifact;
use std::{cell::Cell, time::Duration};
#[derive(Default)]
pub struct PaintTiming {
    first_paint: Cell<Option<Duration>>,
    first_contentful_paint: Cell<Option<Duration>>,
    first_image_paint: Cell<Option<Duration>>,
    first_eligible_to_paint: Cell<Option<Duration>>,
}
#[derive(Default)]
pub struct PaintTimingEffects {
    pub first_paint: bool,
    pub first_contentful_paint: bool,
    pub first_image_paint: bool,
}
impl PaintTiming {
    pub fn MarkFirstEligibleToPaint(&self, now: Duration) {
        if self.first_eligible_to_paint.get().is_none() {
            self.first_eligible_to_paint.set(Some(now));
        }
    }
    pub fn MarkIneligibleToPaint(&self) {
        if self.first_eligible_to_paint.get().is_some() && self.first_paint.get().is_none() {
            self.first_eligible_to_paint.set(None);
        }
    }
    pub fn MarkFirstContentfulPaint(&self, now: Duration) -> PaintTimingEffects {
        let mut effects = PaintTimingEffects::default();
        if self.first_contentful_paint.get().is_none() {
            self.first_contentful_paint.set(Some(now));
            effects.first_contentful_paint = true;
            if self.first_paint.get().is_none() {
                self.first_paint.set(Some(now));
                effects.first_paint = true;
            }
        }
        effects
    }
    pub fn FirstPaintRendered(&self) -> bool {
        self.first_paint.get().is_some()
    }
    pub fn FirstContentfulPaintRendered(&self) -> bool {
        self.first_contentful_paint.get().is_some()
    }
    pub fn NotifyPaint(&self, artifact: &PaintArtifact, now: Duration) -> PaintTimingEffects {
        let mut effects = PaintTimingEffects::default();
        // These are original PaintController/painter bits, not an artifact
        // bounds, color, display-item count, or DOM-content approximation.
        if artifact.first_painted
            || artifact.first_contentful_painted
            || artifact.text_painted
            || artifact.image_painted
        {
            if self.first_paint.get().is_none() {
                self.first_paint.set(Some(now));
                effects.first_paint = true;
            }
        }
        if artifact.first_contentful_painted || artifact.text_painted || artifact.image_painted {
            if self.first_contentful_paint.get().is_none() {
                self.first_contentful_paint.set(Some(now));
                effects.first_contentful_paint = true;
            }
        }
        if artifact.image_painted && self.first_image_paint.get().is_none() {
            self.first_image_paint.set(Some(now));
            effects.first_image_paint = true;
        }
        effects
    }
}
