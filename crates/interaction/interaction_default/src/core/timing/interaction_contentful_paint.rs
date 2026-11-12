//! Chromium interaction_contentful_paint.h:20-52; .cc:17-58.
use super::{
    largest_contentful_paint::LargestContentfulPaint, paint_timing_record::DOMPaintTimingInfo,
};
use std::rc::Rc;
pub struct InteractionContentfulPaint {
    pub start_time: f64,
    pub duration: f64,
    pub navigation_id: u64,
    pub paint_timing_info: DOMPaintTimingInfo,
    largest_contentful_paint: Rc<LargestContentfulPaint>,
    interaction_id: u64,
}
impl InteractionContentfulPaint {
    pub fn new(
        start_time: f64,
        render_time: f64,
        largest_contentful_paint: Rc<LargestContentfulPaint>,
        navigation_id: u64,
        interaction_id: u64,
        paint_timing_info: DOMPaintTimingInfo,
    ) -> Rc<Self> {
        Rc::new(Self {
            start_time,
            duration: render_time - start_time,
            largest_contentful_paint,
            navigation_id,
            interaction_id,
            paint_timing_info,
        })
    }
    pub fn largestContentfulPaint(&self) -> Rc<LargestContentfulPaint> {
        self.largest_contentful_paint.clone()
    }
    pub fn interactionId(&self) -> u64 {
        self.interaction_id
    }
}
