//! Resolved stroke geometry consumed directly by cc PaintFlags.

#![allow(non_snake_case)]

use crate::cc::paint::{
    paint_flags::{Cap, Join, PaintFlags, PaintFlagsTypes, Style},
    path_effect::PathEffect,
};
use foundation::{LineCap, LineJoin};
use std::sync::Arc;

// Source: third_party/blink/renderer/platform/geometry/stroke_data.h:41-75.
#[derive(Clone, Debug)]
pub struct StrokeData {
    thickness_: f32,
    line_cap_: Cap,
    line_join_: Join,
    miter_limit_: f32,
    dash_: Option<Arc<PathEffect>>,
}

impl Default for StrokeData {
    fn default() -> Self {
        Self {
            thickness_: 0.0,
            line_cap_: Cap::kButt_Cap,
            line_join_: Join::kMiter_Join,
            miter_limit_: 4.0,
            dash_: None,
        }
    }
}

impl StrokeData {
    pub fn Thickness(&self) -> f32 {
        self.thickness_
    }

    pub fn SetThickness(&mut self, thickness: f32) {
        self.thickness_ = thickness;
    }

    pub fn SetLineCap(&mut self, cap: LineCap) {
        self.line_cap_ = match cap {
            LineCap::kButtCap => Cap::kButt_Cap,
            LineCap::kRoundCap => Cap::kRound_Cap,
            LineCap::kSquareCap => Cap::kSquare_Cap,
        };
    }

    pub fn SetLineJoin(&mut self, join: LineJoin) {
        self.line_join_ = match join {
            LineJoin::kMiterJoin => Join::kMiter_Join,
            LineJoin::kRoundJoin => Join::kRound_Join,
            LineJoin::kBevelJoin => Join::kBevel_Join,
        };
    }

    pub fn MiterLimit(&self) -> f32 {
        self.miter_limit_
    }

    pub fn SetMiterLimit(&mut self, miter_limit: f32) {
        self.miter_limit_ = miter_limit;
    }

    // Source: third_party/blink/renderer/platform/geometry/stroke_data.cc:35-50.
    pub fn SetLineDash(&mut self, dashes: &[f32], dash_offset: f32) {
        if dashes.is_empty() {
            self.dash_ = None;
            return;
        }
        let count = if dashes.len().is_multiple_of(2) {
            dashes.len()
        } else {
            dashes.len() * 2
        };
        let intervals: Vec<f32> = (0..count)
            .map(|index| dashes[index % dashes.len()])
            .collect();
        self.dash_ = Some(PathEffect::MakeDash(
            &intervals,
            i32::try_from(count).expect("source dash count fits i32"),
            dash_offset,
        ));
    }

    // Source: third_party/blink/renderer/platform/geometry/stroke_data.cc:52-54.
    pub fn SetDashEffect(&mut self, dash_effect: Option<Arc<PathEffect>>) {
        self.dash_ = dash_effect;
    }

    pub fn DashEffect(&self) -> Option<&Arc<PathEffect>> {
        self.dash_.as_ref()
    }

    // Source: third_party/blink/renderer/platform/geometry/stroke_data.cc:56-63.
    pub fn SetupPaint<B>(&self, flags: &mut PaintFlags<B>)
    where
        B: PaintFlagsTypes<PathEffect = PathEffect>,
    {
        flags.setStyle(Style::kStroke_Style);
        flags.setStrokeWidth(self.thickness_);
        flags.setStrokeCap(self.line_cap_);
        flags.setStrokeJoin(self.line_join_);
        flags.setStrokeMiter(self.miter_limit_);
        flags.setPathEffect(self.dash_.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn odd_dash_arrays_repeat_and_empty_arrays_clear() {
        let mut data = StrokeData::default();
        data.SetLineDash(&[1.0, 2.0, 3.0], 4.0);
        match data.DashEffect().unwrap().as_ref() {
            PathEffect::Dash { intervals, phase } => {
                assert_eq!(intervals, &[1.0, 2.0, 3.0, 1.0, 2.0, 3.0]);
                assert_eq!(*phase, 4.0);
            }
            _ => unreachable!("SetLineDash always creates a dash effect"),
        }
        data.SetLineDash(&[], 0.0);
        assert!(data.DashEffect().is_none());
    }
}
