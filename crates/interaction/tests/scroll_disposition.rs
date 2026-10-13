use interaction::frame_aligned_input_queue::FrameAlignedInputQueue;
use interaction::input_event::{InputEvent, WheelEvent, WheelPhase};
use layoutng_assembly::internal::layout_input::Offset;
use std::time::Instant;
#[test]
fn wheel_coalescing_preserves_compositor_default_action_disposition() {
    let mut queue = FrameAlignedInputQueue::default();
    let now = Instant::now();
    for (compositor_handled, dy) in [(true, 20.), (true, -20.), (false, 10.)] {
        queue.Push(
            InputEvent::Wheel(WheelEvent {
                compositor_handled,
                phase: WheelPhase::kChanged,
                delta: Offset { x: 0., y: dy },
                ..Default::default()
            }),
            now,
            1,
        );
    }
    let entry = queue.PopFront().unwrap();
    let InputEvent::Wheel(wheel) = entry.input else {
        panic!("wheel")
    };
    assert!(wheel.compositor_handled);
    assert_eq!(wheel.delta.y, 0.);
    assert_eq!(entry.samples, 2);
    let InputEvent::Wheel(wheel) = queue.PopFront().unwrap().input else {
        panic!("wheel")
    };
    assert!(!wheel.compositor_handled);
    assert_eq!(wheel.delta.y, 10.);
    assert!(queue.IsEmpty());
}
