//! RenderWidgetHostImpl::AutoscrollStart/Fling/End, source.cc:3289-3364.
//! Browser-side gesture protocol state, separate from Blink's controller.
use crate::{
    base::time::time::TimeTicks,
    public::common::input::{
        web_gesture_event::{GestureDevice, WebGestureEvent},
        web_input_event::Type,
    },
};
use foundation::gfx_geometry::PointF;

#[derive(Default)]
pub struct MiddleClickAutoscroll {
    position: Option<PointF>,
    sent_scroll_begin: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn autoscroll_keeps_one_scroll_begin_until_end() {
        let mut owner = MiddleClickAutoscroll::default();
        let now = TimeTicks::FromInternalValue(100);
        owner.Start((12.0, 34.0));
        let first = owner.Fling((0.0, 2.0), now);
        assert_eq!(first.len(), 2);
        assert_eq!(first[0].GetType(), Type::kGestureScrollBegin);
        assert_eq!(first[1].GetType(), Type::kGestureFlingStart);
        assert_eq!(first[0].PositionInWidget(), PointF::new(12.0, 34.0));
        assert_eq!(owner.Fling((0.0, 0.0), now).len(), 1);
        let end = owner.End(now);
        assert_eq!(end.len(), 1);
        assert_eq!(end[0].GetType(), Type::kGestureFlingCancel);
        assert!(end[0].data.fling_cancel.prevent_boosting);
        assert!(owner.End(now).is_empty());
    }
}
impl MiddleClickAutoscroll {
    pub fn Start(&mut self, position: (f32, f32)) {
        self.position = Some(PointF::new(position.0, position.1));
        self.sent_scroll_begin = false;
    }
    fn Event(&self, kind: Type, now: TimeTicks) -> WebGestureEvent {
        let mut event = WebGestureEvent::new(kind, 0, now, GestureDevice::kSyntheticAutoscroll);
        event.SetPositionInWidget(self.position.expect("active autoscroll target"));
        event
    }
    pub fn Fling(&mut self, velocity: (f32, f32), now: TimeTicks) -> Vec<WebGestureEvent> {
        assert!(self.position.is_some(), "autoscroll fling follows start");
        let mut events = Vec::new();
        if !self.sent_scroll_begin && velocity != (0.0, 0.0) {
            let mut begin = self.Event(Type::kGestureScrollBegin, now);
            begin.data.scroll_begin.delta_x_hint = velocity.0;
            begin.data.scroll_begin.delta_y_hint = velocity.1;
            events.push(begin);
            self.sent_scroll_begin = true;
        }
        let mut fling = self.Event(Type::kGestureFlingStart, now);
        fling.data.fling_start.velocity_x = velocity.0;
        fling.data.fling_start.velocity_y = velocity.1;
        events.push(fling);
        events
    }
    pub fn End(&mut self, now: TimeTicks) -> Vec<WebGestureEvent> {
        let mut events = Vec::new();
        if self.sent_scroll_begin {
            let mut cancel = self.Event(Type::kGestureFlingCancel, now);
            cancel.data.fling_cancel.prevent_boosting = true;
            events.push(cancel);
        }
        self.sent_scroll_begin = false;
        self.position = None;
        events
    }
}
