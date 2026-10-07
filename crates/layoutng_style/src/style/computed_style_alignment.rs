use std::sync::OnceLock;

use foundation::{
    g_null_atom, AtomicString, TextDecorationLine, TextEmphasisFill, TextEmphasisMark, WritingMode,
};

use super::computed_style::{ComputedStyle, ComputedStyleBuilder};
use super::computed_style_constants::{
    IsLeft, IsOver, IsRight, ItemPosition, LineLogicalSide, TextEmphasisPosition,
};
use super::style_self_alignment_data::StyleSelfAlignmentData;

// cpp: layoutng_style/style/computed_style_alignment.cc:20-35
fn resolve_self_alignment(
    value: &StyleSelfAlignmentData,
    normal_value_behavior: &StyleSelfAlignmentData,
    has_anchor_center_offset: bool,
) -> StyleSelfAlignmentData {
    if value.GetPosition() == ItemPosition::kLegacy
        || value.GetPosition() == ItemPosition::kNormal
        || value.GetPosition() == ItemPosition::kAuto
    {
        return *normal_value_behavior;
    }
    if !has_anchor_center_offset && value.GetPosition() == ItemPosition::kAnchorCenter {
        return StyleSelfAlignmentData::new(
            ItemPosition::kCenter,
            value.Overflow(),
            value.PositionType(),
        );
    }
    *value
}

fn emphasis_atom(cell: &'static OnceLock<AtomicString>, character: u16) -> &'static AtomicString {
    cell.get_or_init(|| AtomicString::from_utf16(&[character]))
}

#[allow(non_snake_case)]
impl ComputedStyle {
    // cpp: layoutng_style/style/computed_style.h:429-431
    // cpp: layoutng_style/style/computed_style_alignment.cc:39-48
    pub fn ResolvedAlignSelf(
        &self,
        normal_value_behavior: &StyleSelfAlignmentData,
        parent_style: *const ComputedStyle,
    ) -> StyleSelfAlignmentData {
        if parent_style.is_null() || self.AlignSelf().GetPosition() != ItemPosition::kAuto {
            return resolve_self_alignment(
                self.AlignSelf(),
                normal_value_behavior,
                self.AnchorCenterOffset().is_some(),
            );
        }
        resolve_self_alignment(
            unsafe { (&*parent_style).AlignItems() },
            normal_value_behavior,
            self.AnchorCenterOffset().is_some(),
        )
    }

    // cpp: layoutng_style/style/computed_style.h:432-434
    // cpp: layoutng_style/style/computed_style_alignment.cc:50-60
    pub fn ResolvedJustifySelf(
        &self,
        normal_value_behavior: &StyleSelfAlignmentData,
        parent_style: *const ComputedStyle,
    ) -> StyleSelfAlignmentData {
        if parent_style.is_null() || self.JustifySelf().GetPosition() != ItemPosition::kAuto {
            return resolve_self_alignment(
                self.JustifySelf(),
                normal_value_behavior,
                self.AnchorCenterOffset().is_some(),
            );
        }
        resolve_self_alignment(
            unsafe { (&*parent_style).JustifyItems() },
            normal_value_behavior,
            self.AnchorCenterOffset().is_some(),
        )
    }

    // cpp: layoutng_style/style/computed_style.h:792
    // cpp: layoutng_style/style/computed_style_alignment.cc:62-68
    pub fn GetTextEmphasisMark(&self) -> TextEmphasisMark {
        let mark = self.TextEmphasisMarkInternal();
        if mark != TextEmphasisMark::kAuto {
            return mark;
        }
        if self.IsHorizontalTypographicMode() {
            TextEmphasisMark::kDot
        } else {
            TextEmphasisMark::kSesame
        }
    }

    // cpp: layoutng_style/style/computed_style.h:793
    // cpp: layoutng_style/style/computed_style_alignment.cc:70-117
    pub fn TextEmphasisMarkString(&self) -> &AtomicString {
        static DOT_FILLED: OnceLock<AtomicString> = OnceLock::new();
        static DOT_OPEN: OnceLock<AtomicString> = OnceLock::new();
        static CIRCLE_FILLED: OnceLock<AtomicString> = OnceLock::new();
        static CIRCLE_OPEN: OnceLock<AtomicString> = OnceLock::new();
        static DOUBLE_CIRCLE_FILLED: OnceLock<AtomicString> = OnceLock::new();
        static DOUBLE_CIRCLE_OPEN: OnceLock<AtomicString> = OnceLock::new();
        static TRIANGLE_FILLED: OnceLock<AtomicString> = OnceLock::new();
        static TRIANGLE_OPEN: OnceLock<AtomicString> = OnceLock::new();
        static SESAME_FILLED: OnceLock<AtomicString> = OnceLock::new();
        static SESAME_OPEN: OnceLock<AtomicString> = OnceLock::new();

        match self.GetTextEmphasisMark() {
            TextEmphasisMark::kNone => &g_null_atom,
            TextEmphasisMark::kCustom => self.TextEmphasisCustomMark(),
            TextEmphasisMark::kDot => {
                let filled = emphasis_atom(&DOT_FILLED, 0x2022);
                let open = emphasis_atom(&DOT_OPEN, 0x25E6);
                if self.GetTextEmphasisFill() == TextEmphasisFill::kFilled {
                    filled
                } else {
                    open
                }
            }
            TextEmphasisMark::kCircle => {
                let filled = emphasis_atom(&CIRCLE_FILLED, 0x25CF);
                let open = emphasis_atom(&CIRCLE_OPEN, 0x25CB);
                if self.GetTextEmphasisFill() == TextEmphasisFill::kFilled {
                    filled
                } else {
                    open
                }
            }
            TextEmphasisMark::kDoubleCircle => {
                let filled = emphasis_atom(&DOUBLE_CIRCLE_FILLED, 0x25C9);
                let open = emphasis_atom(&DOUBLE_CIRCLE_OPEN, 0x25CE);
                if self.GetTextEmphasisFill() == TextEmphasisFill::kFilled {
                    filled
                } else {
                    open
                }
            }
            TextEmphasisMark::kTriangle => {
                let filled = emphasis_atom(&TRIANGLE_FILLED, 0x25B2);
                let open = emphasis_atom(&TRIANGLE_OPEN, 0x25B3);
                if self.GetTextEmphasisFill() == TextEmphasisFill::kFilled {
                    filled
                } else {
                    open
                }
            }
            TextEmphasisMark::kSesame => {
                let filled = emphasis_atom(&SESAME_FILLED, 0xFE45);
                let open = emphasis_atom(&SESAME_OPEN, 0xFE46);
                if self.GetTextEmphasisFill() == TextEmphasisFill::kFilled {
                    filled
                } else {
                    open
                }
            }
            TextEmphasisMark::kAuto => unreachable!("auto was resolved by GetTextEmphasisMark"),
        }
    }

    // cpp: layoutng_style/style/computed_style.h:794
    // cpp: layoutng_style/style/computed_style_alignment.cc:119-145
    pub fn GetTextEmphasisLineLogicalSide(&self) -> LineLogicalSide {
        let position: TextEmphasisPosition = self.GetTextEmphasisPosition();
        if foundation::RuntimeEnabledFeatures::TextEmphasisPositionAutoEnabled()
            && position == TextEmphasisPosition::kAuto
        {
            if self.IsHorizontalWritingMode() {
                let locale = self.GetFontDescription().Locale();
                return if !locale.is_null() && unsafe { (&*locale).IsMacrolanguageChinese() } {
                    LineLogicalSide::kUnder
                } else {
                    LineLogicalSide::kOver
                };
            }
            return match self.GetWritingMode() {
                WritingMode::kVerticalRl | WritingMode::kVerticalLr | WritingMode::kSidewaysRl => {
                    LineLogicalSide::kOver
                }
                WritingMode::kSidewaysLr => LineLogicalSide::kUnder,
                _ => unreachable!("auto emphasis position requires vertical writing mode"),
            };
        }
        if self.IsHorizontalWritingMode() {
            return if IsOver(position) {
                LineLogicalSide::kOver
            } else {
                LineLogicalSide::kUnder
            };
        }
        if self.GetWritingMode() != WritingMode::kSidewaysLr {
            return if IsRight(position) {
                LineLogicalSide::kOver
            } else {
                LineLogicalSide::kUnder
            };
        }
        if IsLeft(position) {
            LineLogicalSide::kOver
        } else {
            LineLogicalSide::kUnder
        }
    }

    // cpp: layoutng_style/style/computed_style.h:1871
    // cpp: layoutng_style/style/computed_style_alignment.cc:147-154
    pub fn TextDecorationsInEffect(&self) -> TextDecorationLine {
        let mut decorations = self.GetTextDecorationLine();
        let base = self.BaseTextDecorationData();
        if !base.is_null() {
            for decoration in unsafe { &*base } {
                decorations |= decoration.Lines();
            }
        }
        decorations
    }
}

#[allow(non_snake_case)]
impl ComputedStyleBuilder {
    // cpp: layoutng_style/style/computed_style.h:3252
    // cpp: layoutng_style/style/computed_style_alignment.cc:156-163
    pub fn UpdateFontOrientation(&mut self) {
        let orientation = self.ComputeFontOrientation();
        if self.GetFontDescription().Orientation() == orientation {
            return;
        }
        let mut description = self.GetFontDescription().clone();
        description.SetOrientation(orientation);
        self.SetFontDescription(&description);
    }
}
