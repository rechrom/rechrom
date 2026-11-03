//! Styled stroke geometry and cc PaintFlags setup.

#![allow(non_snake_case)]

use crate::cc::paint::{
    paint_flags::{Cap, Join, PaintFlags, PaintFlagsTypes, Style},
    path_effect::PathEffect,
};
use crate::stroke_data::StrokeData;
use foundation::LineCap;

// The value crosses Layout ink-overflow and Paint recording. Foundation owns
// its single scalar identity; this module preserves the Paint package path.
pub use foundation::StrokeStyle;

// Source: third_party/blink/renderer/platform/graphics/styled_stroke_data.h:83-87.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GeometryInfo {
    pub path_length: i32,
    pub dash_thickness: i32,
    pub closed_path: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct DashDescription {
    intervals: [f32; 2],
    cap: Cap,
}

// Source: third_party/blink/renderer/platform/graphics/styled_stroke_data.cc:40-58.
fn SelectBestDashGap(
    stroke_length: f32,
    dash_length: f32,
    gap_length: f32,
    closed_path: bool,
) -> f32 {
    let available_length = if closed_path {
        stroke_length
    } else {
        stroke_length + gap_length
    };
    let min_num_dashes = (available_length / (dash_length + gap_length)).floor();
    let max_num_dashes = min_num_dashes + 1.0;
    let min_num_gaps = if closed_path {
        min_num_dashes
    } else {
        min_num_dashes - 1.0
    };
    let max_num_gaps = if closed_path {
        max_num_dashes
    } else {
        max_num_dashes - 1.0
    };
    let min_gap = (stroke_length - min_num_dashes * dash_length) / min_num_gaps;
    let max_gap = (stroke_length - max_num_dashes * dash_length) / max_num_gaps;
    if max_gap <= 0.0 || (min_gap - gap_length).abs() < (max_gap - gap_length).abs() {
        min_gap
    } else {
        max_gap
    }
}

// Source: third_party/blink/renderer/platform/graphics/styled_stroke_data.cc:64-66.
fn DashLengthRatio(thickness: f32) -> f32 {
    if thickness >= 3.0 {
        2.0
    } else {
        3.0
    }
}

// Source: third_party/blink/renderer/platform/graphics/styled_stroke_data.cc:72-74.
fn DashGapRatio(thickness: f32) -> f32 {
    if thickness >= 3.0 {
        1.0
    } else {
        2.0
    }
}

// Source: third_party/blink/renderer/platform/graphics/styled_stroke_data.h:51-107.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StyledStrokeData {
    style_: StrokeStyle,
    thickness_: f32,
}

impl Default for StyledStrokeData {
    fn default() -> Self {
        Self {
            style_: StrokeStyle::kSolidStroke,
            thickness_: 0.0,
        }
    }
}

impl StyledStrokeData {
    // Source: third_party/blink/renderer/platform/graphics/styled_stroke_data.h:61-65.
    pub fn Style(&self) -> StrokeStyle {
        self.style_
    }

    pub fn SetStyle(&mut self, style: StrokeStyle) {
        self.style_ = style;
    }

    pub fn Thickness(&self) -> f32 {
        self.thickness_
    }

    pub fn SetThickness(&mut self, thickness: f32) {
        self.thickness_ = thickness;
    }

    // Source: third_party/blink/renderer/platform/graphics/styled_stroke_data.cc:81-134.
    fn DashEffectFromStrokeStyle(&self, info: &GeometryInfo) -> Option<DashDescription> {
        let dash_width = if info.dash_thickness != 0 {
            info.dash_thickness as f32
        } else {
            self.Thickness()
        };
        if Self::StrokeIsDashed(dash_width, self.Style()) {
            let mut dash_length = dash_width;
            let mut gap_length = dash_length;
            if self.Style() == StrokeStyle::kDashedStroke {
                dash_length *= DashLengthRatio(dash_width);
                gap_length *= DashGapRatio(dash_width);
            }
            if info.path_length as f32 <= dash_length * 2.0 {
                return None;
            }
            let mut two_dashes_with_gap_length = 2.0 * dash_length + gap_length;
            if info.closed_path {
                two_dashes_with_gap_length += gap_length;
            }
            if info.path_length as f32 <= two_dashes_with_gap_length {
                let multiplier = info.path_length as f32 / two_dashes_with_gap_length;
                return Some(DashDescription {
                    intervals: [dash_length * multiplier, gap_length * multiplier],
                    cap: Cap::kButt_Cap,
                });
            }
            let gap = if self.Style() == StrokeStyle::kDashedStroke {
                SelectBestDashGap(
                    info.path_length as f32,
                    dash_length,
                    gap_length,
                    info.closed_path,
                )
            } else {
                gap_length
            };
            return Some(DashDescription {
                intervals: [dash_length, gap],
                cap: Cap::kButt_Cap,
            });
        }
        if self.Style() == StrokeStyle::kDottedStroke {
            let per_dot_length = dash_width * 2.0;
            if (info.path_length as f32) < per_dot_length {
                return Some(DashDescription {
                    intervals: [0.0, per_dot_length],
                    cap: Cap::kRound_Cap,
                });
            }
            let gap = SelectBestDashGap(
                info.path_length as f32,
                dash_width,
                dash_width,
                info.closed_path,
            );
            return Some(DashDescription {
                intervals: [0.0, gap + dash_width - 1.0e-2],
                cap: Cap::kRound_Cap,
            });
        }
        None
    }

    // Source: third_party/blink/renderer/platform/graphics/styled_stroke_data.cc:138-150.
    pub fn SetupPaint<B>(&self, flags: &mut PaintFlags<B>)
    where
        B: PaintFlagsTypes<PathEffect = PathEffect>,
    {
        self.SetupPaintWithGeometry(flags, &GeometryInfo::default());
    }

    pub fn SetupPaintWithGeometry<B>(&self, flags: &mut PaintFlags<B>, info: &GeometryInfo)
    where
        B: PaintFlagsTypes<PathEffect = PathEffect>,
    {
        flags.setStyle(Style::kStroke_Style);
        flags.setStrokeWidth(self.thickness_);
        flags.setStrokeCap(Cap::kButt_Cap);
        flags.setStrokeJoin(Join::kMiter_Join);
        flags.setStrokeMiter(4.0);
        self.SetupPaintDashPathEffect(flags, info);
    }

    // Source: third_party/blink/renderer/platform/graphics/styled_stroke_data.cc:152-161.
    fn SetupPaintDashPathEffect<B>(&self, flags: &mut PaintFlags<B>, info: &GeometryInfo)
    where
        B: PaintFlagsTypes<PathEffect = PathEffect>,
    {
        if let Some(dash) = self.DashEffectFromStrokeStyle(info) {
            flags.setPathEffect(Some(PathEffect::MakeDash(&dash.intervals, 2, 0.0)));
            flags.setStrokeCap(dash.cap);
        } else {
            flags.setPathEffect(None);
        }
    }

    // Source: third_party/blink/renderer/platform/graphics/styled_stroke_data.cc:163-172.
    pub fn ConvertToStrokeData(&self, info: &GeometryInfo) -> StrokeData {
        let mut stroke_data = StrokeData::default();
        stroke_data.SetThickness(self.thickness_);
        if let Some(dash) = self.DashEffectFromStrokeStyle(info) {
            stroke_data.SetDashEffect(Some(PathEffect::MakeDash(&dash.intervals, 2, 0.0)));
            stroke_data.SetLineCap(match dash.cap {
                Cap::kButt_Cap => LineCap::kButtCap,
                Cap::kRound_Cap => LineCap::kRoundCap,
                Cap::kSquare_Cap => LineCap::kSquareCap,
            });
        }
        stroke_data
    }

    // Source: third_party/blink/renderer/platform/graphics/styled_stroke_data.cc:174-176.
    pub fn StrokeIsDashed(width: f32, style: StrokeStyle) -> bool {
        style == StrokeStyle::kDashedStroke || (style == StrokeStyle::kDottedStroke && width <= 3.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stroke(style: StrokeStyle, thickness: f32) -> StyledStrokeData {
        let mut data = StyledStrokeData::default();
        data.SetStyle(style);
        data.SetThickness(thickness);
        data
    }

    #[test]
    fn dashed_geometry_fits_short_and_long_open_paths() {
        let data = stroke(StrokeStyle::kDashedStroke, 2.0);
        assert_eq!(
            data.DashEffectFromStrokeStyle(&GeometryInfo {
                path_length: 8,
                ..Default::default()
            }),
            None
        );
        assert_eq!(
            data.DashEffectFromStrokeStyle(&GeometryInfo {
                path_length: 13,
                ..Default::default()
            }),
            Some(DashDescription {
                intervals: [4.875, 3.25],
                cap: Cap::kButt_Cap,
            })
        );
        let long = data
            .DashEffectFromStrokeStyle(&GeometryInfo {
                path_length: 40,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(long.intervals[0], 6.0);
        assert!(long.intervals[1].is_finite());
    }

    #[test]
    fn dotted_geometry_switches_from_dashes_to_round_caps() {
        let thin = stroke(StrokeStyle::kDottedStroke, 2.0);
        assert!(StyledStrokeData::StrokeIsDashed(2.0, thin.Style()));
        assert_eq!(
            thin.DashEffectFromStrokeStyle(&GeometryInfo {
                path_length: 8,
                ..Default::default()
            })
            .unwrap()
            .cap,
            Cap::kButt_Cap
        );

        let thick = stroke(StrokeStyle::kDottedStroke, 4.0);
        assert!(!StyledStrokeData::StrokeIsDashed(4.0, thick.Style()));
        let dot = thick
            .DashEffectFromStrokeStyle(&GeometryInfo {
                path_length: 20,
                closed_path: true,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(dot.cap, Cap::kRound_Cap);
        assert_eq!(dot.intervals[0], 0.0);
        assert!(dot.intervals[1] > 0.0);

        let resolved = thick.ConvertToStrokeData(&GeometryInfo {
            path_length: 20,
            closed_path: true,
            ..Default::default()
        });
        assert_eq!(resolved.Thickness(), 4.0);
        assert!(resolved.DashEffect().is_some());
    }
}
