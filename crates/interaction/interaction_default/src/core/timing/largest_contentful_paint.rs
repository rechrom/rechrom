//! Chromium largest_contentful_paint.h:18-70; .cc:15-78.
use super::paint_timing_record::DOMPaintTimingInfo;
use foundation::AtomicString;
use interaction_dispatch::core::dom::events::event::{EventTarget, EventTargetRef};
use std::rc::{Rc, Weak};
pub trait LargestContentfulPaintElementFacts {
    fn IsConnected(&self, element: &EventTargetRef) -> bool;
    fn IsInShadowTree(&self, element: &EventTargetRef) -> bool;
    fn IsDocumentActive(&self, element: &EventTargetRef) -> bool;
    fn DocumentHasFrame(&self, element: &EventTargetRef) -> bool;
}
pub struct LargestContentfulPaint {
    pub start_time: f64,
    pub navigation_id: u64,
    pub paint_timing_info: DOMPaintTimingInfo,
    size: u64,
    render_time: f64,
    load_time: f64,
    id: AtomicString,
    url: String,
    element: Option<Weak<dyn EventTarget>>,
}
impl LargestContentfulPaint {
    pub fn new(
        start_time: f64,
        render_time: f64,
        size: u64,
        load_time: f64,
        id: AtomicString,
        url: String,
        element: Option<EventTargetRef>,
        navigation_id: u64,
        paint_timing_info: DOMPaintTimingInfo,
    ) -> Rc<Self> {
        Rc::new(Self {
            start_time,
            render_time,
            size,
            load_time,
            id,
            url,
            element: element.as_ref().map(Rc::downgrade),
            navigation_id,
            paint_timing_info,
        })
    }
    pub fn size(&self) -> u64 {
        self.size
    }
    pub fn renderTime(&self) -> f64 {
        self.render_time
    }
    pub fn loadTime(&self) -> f64 {
        self.load_time
    }
    pub fn id(&self) -> &AtomicString {
        &self.id
    }
    pub fn url(&self) -> &str {
        &self.url
    }
    pub fn element(
        &self,
        facts: &dyn LargestContentfulPaintElementFacts,
    ) -> Option<EventTargetRef> {
        let element = self.element.as_ref()?.upgrade()?;
        (facts.IsConnected(&element)
            && !facts.IsInShadowTree(&element)
            && facts.IsDocumentActive(&element)
            && facts.DocumentHasFrame(&element))
        .then_some(element)
    }
}
