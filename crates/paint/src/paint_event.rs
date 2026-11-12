//! Paint milestones that receive compositor presentation times.
//!
//! Source: `third_party/blink/renderer/core/paint/paint_event.h`.

#![allow(non_camel_case_types)]

/// The source declaration is shared by PaintTiming and
/// FirstMeaningfulPaintDetector. Rechrom keeps the same ordered discriminants
/// at the Paint boundary so a future timing consumer cannot silently reorder
/// persisted or traced milestone values.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PaintEvent {
    kFirstPaint = 0,
    kFirstContentfulPaint = 1,
    kProvisionalFirstMeaningfulPaint = 2,
    kFirstImagePaint = 3,
}

#[cfg(test)]
mod tests {
    use super::PaintEvent;

    #[test]
    fn source_order_is_stable() {
        assert_eq!(PaintEvent::kFirstPaint as i32, 0);
        assert_eq!(PaintEvent::kFirstContentfulPaint as i32, 1);
        assert_eq!(PaintEvent::kProvisionalFirstMeaningfulPaint as i32, 2);
        assert_eq!(PaintEvent::kFirstImagePaint as i32, 3);
    }
}
