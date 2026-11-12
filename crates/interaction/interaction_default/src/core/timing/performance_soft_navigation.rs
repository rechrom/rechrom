//! Chromium performance_soft_navigation.h:19-57; .cc:19-76.
use super::{
    interaction_contentful_paint::InteractionContentfulPaint,
    paint_timing_record::DOMPaintTimingInfo, soft_navigation_context::SoftNavigationContext,
};
use crate::core::navigation_api::navigation_api::NavigationType;
use std::rc::Rc;
pub struct PerformanceSoftNavigation {
    pub start_time: f64,
    pub duration: f64,
    pub name: String,
    pub navigation_id: u64,
    pub paint_timing_info: DOMPaintTimingInfo,
    navigation_type: NavigationType,
    interaction_id: u64,
    context: Rc<SoftNavigationContext>,
}
impl PerformanceSoftNavigation {
    pub fn new(
        start_time: f64,
        paint_timing_info: DOMPaintTimingInfo,
        context: Rc<SoftNavigationContext>,
    ) -> Rc<Self> {
        Rc::new(Self {
            start_time,
            duration: paint_timing_info.presentation_time - start_time,
            name: context.AttributionUrl(),
            navigation_id: context.NavigationId().web_exposed_id,
            navigation_type: context.NavigationType(),
            interaction_id: context.InteractionId(),
            context,
            paint_timing_info,
        })
    }
    pub fn navigationType(&self) -> NavigationType {
        self.navigation_type
    }
    pub fn interactionId(&self) -> u64 {
        self.interaction_id
    }
    pub fn getLargestInteractionContentfulPaint(&self) -> Option<Rc<InteractionContentfulPaint>> {
        self.context.LargestIcpEntry()
    }
}
