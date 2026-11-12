//! Platform-neutral host routing sidecar for a source-generated canonical
//! wheel. This value never replaces or reconstructs its WebMouseWheelEvent.
use super::input_event::{
    EventModifiers, Offset, ScrollGranularity, WheelEvent, WheelMetadata, WheelPhase,
};
use interaction_input::public::common::input::{
    web_input_event::WebInputEvent,
    web_mouse_wheel_event::{Phase, WebMouseWheelEvent},
};

pub fn WheelHostSidecar(event: &WebMouseWheelEvent) -> WheelEvent {
    let position = event.pointer_properties.PositionInWidget();
    let modifiers = event.GetModifiers();
    WheelEvent {
        position: Offset {
            x: position.x() as f64,
            y: position.y() as f64,
        },
        delta: Offset {
            x: -event.delta_x as f64,
            y: -event.delta_y as f64,
        },
        phase: if event.momentum_phase == Phase::kPhaseBegan {
            WheelPhase::kBegan
        } else if event.momentum_phase == Phase::kPhaseEnded {
            WheelPhase::kEnded
        } else {
            WheelPhase::kChanged
        },
        delta_units: ScrollGranularity::kScrollByPrecisePixel,
        native: Some(WheelMetadata {
            timestamp_seconds: event.TimeStamp().ToInternalValue() as f64 / 1_000_000.,
            phase: event.phase.0 as u64,
            momentum_phase: event.momentum_phase.0 as u64,
        }),
        modifiers: EventModifiers {
            alt: modifiers & WebInputEvent::kAltKey != 0,
            control: modifiers & WebInputEvent::kControlKey != 0,
            meta: modifiers & WebInputEvent::kMetaKey != 0,
            shift: modifiers & WebInputEvent::kShiftKey != 0,
        },
        has_synthetic_phase: event.has_synthetic_phase,
        compositor_nonblocking: true,
        ..WheelEvent::default()
    }
}
