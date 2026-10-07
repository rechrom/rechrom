use std::ops::Deref;

use font_engine::{FontBaseline, FontHeight};
use foundation::{EBaselineSource, EInlineBlockBaselineEdge, LayoutUnit, WritingDirectionMode};
use layoutng_geometry::geometry::box_strut::{BoxStrut, LineBoxStrut};
use layoutng_geometry::geometry::writing_mode_converter::WritingModeConverter;

use crate::logical_fragment::LogicalFragment;
use crate::physical_box_fragment::PhysicalBoxFragment;

// C++ derives from LogicalFragment and downcasts its physical reference. The
// typed borrowed reference retains that downcast invariant without a cast.
// cpp: layoutng_fragment_tree/logical_box_fragment.h:20-28
pub struct LogicalBoxFragment<'a> {
    logical_fragment_: LogicalFragment<'a>,
    physical_box_fragment_: &'a PhysicalBoxFragment,
}

impl<'a> Deref for LogicalBoxFragment<'a> {
    type Target = LogicalFragment<'a>;

    fn deref(&self) -> &Self::Target {
        &self.logical_fragment_
    }
}

#[allow(non_snake_case)]
impl<'a> LogicalBoxFragment<'a> {
    // PhysicalBoxFragment's base-class Deref is translated with that class.
    // cpp: layoutng_fragment_tree/logical_box_fragment.h:22-24
    pub fn new(
        writing_direction: WritingDirectionMode,
        physical_fragment: &'a PhysicalBoxFragment,
    ) -> Self {
        Self {
            logical_fragment_: LogicalFragment::new(writing_direction, physical_fragment),
            physical_box_fragment_: physical_fragment,
        }
    }

    // cpp: layoutng_fragment_tree/logical_box_fragment.h:26-28
    pub fn GetPhysicalBoxFragment(&self) -> &'a PhysicalBoxFragment {
        self.physical_box_fragment_
    }

    // cpp: layoutng_fragment_tree/logical_box_fragment.h:30-33
    pub fn IsWritingModeEqual(&self) -> bool {
        self.GetWritingDirection().GetWritingMode()
            == self.physical_box_fragment_.Style().GetWritingMode()
    }

    // cpp: layoutng_fragment_tree/logical_box_fragment.h:35-42
    pub fn SynthesizedBaseline(
        baseline_type: FontBaseline,
        is_flipped_lines: bool,
        block_size: LayoutUnit,
    ) -> LayoutUnit {
        if baseline_type == FontBaseline::kAlphabeticBaseline {
            if is_flipped_lines {
                LayoutUnit::default()
            } else {
                block_size
            }
        } else {
            block_size / 2
        }
    }

    // cpp: layoutng_fragment_tree/logical_box_fragment.h:44-53
    pub fn FirstBaseline(&self) -> Option<LayoutUnit> {
        if !self.IsWritingModeEqual() {
            return None;
        }
        let mut baseline = self.physical_box_fragment_.FirstBaseline();
        if let Some(value) = baseline.as_mut() {
            if self.physical_box_fragment_.IsScrollContainer() {
                *value = (*value).min(self.BlockSize()).max(LayoutUnit::default());
            }
        }
        baseline
    }

    // cpp: layoutng_fragment_tree/logical_box_fragment.h:55-61
    pub fn FirstBaselineOrSynthesize(&self, baseline_type: FontBaseline) -> LayoutUnit {
        self.FirstBaseline().unwrap_or_else(|| {
            Self::SynthesizedBaseline(
                baseline_type,
                self.GetWritingDirection().IsFlippedLines(),
                self.BlockSize(),
            )
        })
    }

    // cpp: layoutng_fragment_tree/logical_box_fragment.h:63-72
    pub fn LastBaseline(&self) -> Option<LayoutUnit> {
        if !self.IsWritingModeEqual() {
            return None;
        }
        let mut baseline = self.physical_box_fragment_.LastBaseline();
        if let Some(value) = baseline.as_mut() {
            if self.physical_box_fragment_.IsScrollContainer() {
                *value = (*value).min(self.BlockSize()).max(LayoutUnit::default());
            }
        }
        baseline
    }

    // cpp: layoutng_fragment_tree/logical_box_fragment.h:74-80
    pub fn LastBaselineOrSynthesize(&self, baseline_type: FontBaseline) -> LayoutUnit {
        self.LastBaseline().unwrap_or_else(|| {
            Self::SynthesizedBaseline(
                baseline_type,
                self.GetWritingDirection().IsFlippedLines(),
                self.BlockSize(),
            )
        })
    }

    // cpp: layoutng_fragment_tree/logical_box_fragment.h:88-91
    pub fn Borders(&self) -> BoxStrut {
        self.physical_box_fragment_
            .Borders()
            .ConvertToLogical(self.GetWritingDirection())
    }

    // cpp: layoutng_fragment_tree/logical_box_fragment.h:92-95
    pub fn Scrollbar(&self) -> BoxStrut {
        self.physical_box_fragment_
            .Scrollbar()
            .ConvertToLogical(self.GetWritingDirection())
    }

    // cpp: layoutng_fragment_tree/logical_box_fragment.h:96-99
    pub fn Padding(&self) -> BoxStrut {
        self.physical_box_fragment_
            .Padding()
            .ConvertToLogical(self.GetWritingDirection())
    }

    // cpp: layoutng_fragment_tree/logical_box_fragment.h:100-102
    pub fn BoxDecorations(&self) -> BoxStrut {
        self.Borders() + self.Scrollbar() + self.Padding()
    }

    // cpp: layoutng_fragment_tree/logical_box_fragment.h:104-106
    pub fn HasDescendantsForTablePart(&self) -> bool {
        self.physical_box_fragment_.HasDescendantsForTablePart()
    }

    // cpp: layoutng_fragment_tree/logical_box_fragment.h:82-86
    // cpp: layoutng_fragment_tree/logical_box_fragment.cc:14-84
    pub fn BaselineMetrics(
        &self,
        margins: &LineBoxStrut,
        baseline_type: FontBaseline,
    ) -> FontHeight {
        let fragment = self.GetPhysicalBoxFragment();
        let style = fragment.Style();
        let baseline = match style.BaselineSource() {
            EBaselineSource::kAuto => {
                let baseline = if fragment.UseLastBaselineForInlineBaseline() {
                    self.LastBaseline()
                } else {
                    self.FirstBaseline()
                };
                if fragment.ForceInlineBaselineSynthesis() {
                    None
                } else {
                    baseline
                }
            }
            EBaselineSource::kFirst => self.FirstBaseline(),
            EBaselineSource::kLast => self.LastBaseline(),
        };

        if let Some(baseline) = baseline {
            let (ascent, descent) = if self.GetWritingDirection().IsFlippedLines() {
                (self.BlockSize() - baseline, baseline)
            } else {
                (baseline, self.BlockSize() - baseline)
            };
            return FontHeight {
                ascent: ascent + margins.line_over,
                descent: descent + margins.line_under,
            };
        }

        let synthesize_metrics = |size: LayoutUnit| -> FontHeight {
            if baseline_type == FontBaseline::kAlphabeticBaseline {
                FontHeight {
                    ascent: size,
                    descent: LayoutUnit::default(),
                }
            } else {
                FontHeight {
                    ascent: size - size / 2,
                    descent: size / 2,
                }
            }
        };

        match style.InlineBlockBaselineEdge() {
            EInlineBlockBaselineEdge::kMarginBox => {
                synthesize_metrics(self.BlockSize() + margins.BlockSum())
            }
            EInlineBlockBaselineEdge::kBorderBox => {
                let mut metrics = synthesize_metrics(self.BlockSize());
                metrics.ascent += margins.line_over;
                metrics.descent += margins.line_under;
                metrics
            }
            EInlineBlockBaselineEdge::kContentBox => {
                let border_scrollbar_padding = LineBoxStrut::from_box(
                    &(self.Borders() + self.Scrollbar() + self.Padding()),
                    self.GetWritingDirection().IsFlippedLines(),
                );
                let content_size =
                    (self.BlockSize() - border_scrollbar_padding.BlockSum()).ClampNegativeToZero();
                let mut metrics = synthesize_metrics(content_size);
                metrics.ascent += margins.line_over + border_scrollbar_padding.line_over;
                metrics.descent += margins.line_under + border_scrollbar_padding.line_under;
                metrics
            }
        }
    }

    // cpp: layoutng_fragment_tree/logical_box_fragment.h:108
    // cpp: layoutng_fragment_tree/logical_box_fragment.cc:86-91
    pub fn BlockEndScrollableOverflow(&self) -> LayoutUnit {
        let converter = WritingModeConverter::new(
            self.GetWritingDirection(),
            self.physical_box_fragment_.Size(),
        );
        converter
            .ToLogicalRect(self.physical_box_fragment_.ScrollableOverflow())
            .BlockEndOffset()
    }
}
