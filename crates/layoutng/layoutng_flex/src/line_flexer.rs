#![allow(non_snake_case)]

use foundation::LayoutUnit;

use crate::flex_item::{FlexItem, FlexerState};

// cpp: layoutng_flex/line_flexer.h:29-32
#[derive(Clone, Copy, PartialEq, Eq)]
enum FlexerMode {
    kGrow,
    kShrink,
}

// cpp: layoutng_flex/line_flexer.h:14-55
pub struct LineFlexer<'a> {
    line_items_: &'a mut [FlexItem],
    main_axis_inner_size_: LayoutUnit,
    gap_between_items_: LayoutUnit,
    mode_: FlexerMode,
    total_flex_factor_: f64,
    initial_free_space_: LayoutUnit,
    free_space_: LayoutUnit,
}

impl<'a> LineFlexer<'a> {
    // cpp: layoutng_flex/line_flexer.cc:9-42
    pub fn new(
        line_items: &'a mut [FlexItem],
        main_axis_inner_size: LayoutUnit,
        sum_hypothetical_main_size: LayoutUnit,
        gap_between_items: LayoutUnit,
    ) -> Self {
        let mode = if sum_hypothetical_main_size < main_axis_inner_size {
            FlexerMode::kGrow
        } else {
            FlexerMode::kShrink
        };
        let mut flexer = Self {
            line_items_: line_items,
            main_axis_inner_size_: main_axis_inner_size,
            gap_between_items_: gap_between_items,
            mode_: mode,
            total_flex_factor_: 0.0,
            initial_free_space_: LayoutUnit::default(),
            free_space_: LayoutUnit::default(),
        };
        for item in flexer.line_items_.iter() {
            debug_assert_eq!(item.flexed_content_size, item.hypothetical_content_size);
        }
        flexer.FreezeItems(|item| {
            let factor = if mode == FlexerMode::kGrow {
                item.flex_grow
            } else {
                item.flex_shrink
            };
            factor == 0.0
                || if mode == FlexerMode::kGrow {
                    item.base_content_size > item.hypothetical_content_size
                } else {
                    item.base_content_size < item.hypothetical_content_size
                }
        });
        flexer.initial_free_space_ = flexer.free_space_;
        flexer
    }

    // cpp: layoutng_flex/line_flexer.h:23-27
    pub fn Run(&mut self) {
        while self.ResolveFlexibleLengths() {}
    }

    // cpp: layoutng_flex/line_flexer.cc:44-88
    fn FreezeItems(&mut self, should_freeze: impl Fn(&FlexItem) -> bool) {
        self.total_flex_factor_ = 0.0;
        let mut total_weighted_flex_shrink = 0.0;
        self.free_space_ = self.main_axis_inner_size_;
        for item in self.line_items_.iter_mut() {
            item.state = if item.state == FlexerState::kFrozen || should_freeze(item) {
                FlexerState::kFrozen
            } else {
                FlexerState::kNone
            };
            if item.state == FlexerState::kFrozen {
                self.free_space_ -= item.FlexedMarginBoxSize() + self.gap_between_items_;
                continue;
            }
            item.flexed_content_size = item.hypothetical_content_size;
            let factor = if self.mode_ == FlexerMode::kGrow {
                item.flex_grow as f64
            } else {
                item.flex_shrink as f64
            };
            debug_assert_ne!(factor, 0.0);
            self.total_flex_factor_ += factor;
            if self.mode_ == FlexerMode::kGrow {
                item.free_space_fraction = factor / self.total_flex_factor_;
            } else {
                let weighted_flex_shrink = factor * item.base_content_size.ToDouble();
                total_weighted_flex_shrink += weighted_flex_shrink;
                item.free_space_fraction = weighted_flex_shrink / total_weighted_flex_shrink;
            }
            self.free_space_ -= item.FlexBaseMarginBoxSize() + self.gap_between_items_;
        }
        self.free_space_ += self.gap_between_items_;
    }

    // cpp: layoutng_flex/line_flexer.cc:90-179
    fn ResolveFlexibleLengths(&mut self) -> bool {
        if self.total_flex_factor_ > 0.0 && self.total_flex_factor_ < 1.0 {
            let fractional =
                LayoutUnit::from_f64(self.initial_free_space_ * self.total_flex_factor_);
            if fractional.Abs() < self.free_space_.Abs() {
                self.free_space_ = fractional;
            }
        }
        if self.mode_ == FlexerMode::kGrow {
            if self.free_space_ <= LayoutUnit::default() {
                return false;
            }
        } else if self.free_space_ >= LayoutUnit::default() {
            return false;
        }
        let mut total_violation = LayoutUnit::default();
        for item in self.line_items_.iter_mut().rev() {
            if item.state == FlexerState::kFrozen {
                continue;
            }
            let extra_size = if item.free_space_fraction == 1.0 {
                self.free_space_
            } else {
                let extra = self.free_space_ * item.free_space_fraction;
                if extra.is_finite() {
                    LayoutUnit::FromDoubleRound(extra)
                } else {
                    LayoutUnit::default()
                }
            };
            self.free_space_ -= extra_size;
            let item_size = item.base_content_size + extra_size;
            let adjusted = item.main_axis_min_max_sizes.ClampSizeToMinAndMax(item_size);
            debug_assert!(adjusted >= LayoutUnit::default());
            item.flexed_content_size = adjusted;
            let violation = adjusted - item_size;
            if violation != LayoutUnit::default() {
                item.state = if violation < LayoutUnit::default() {
                    FlexerState::kMaxViolation
                } else {
                    FlexerState::kMinViolation
                };
            }
            total_violation += violation;
        }
        if total_violation != LayoutUnit::default() {
            let state = if total_violation < LayoutUnit::default() {
                FlexerState::kMaxViolation
            } else {
                FlexerState::kMinViolation
            };
            self.FreezeItems(|item| item.state == state);
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::WritingMode;
    use layoutng_assembly::internal::baseline_utils::BaselineGroup;
    use layoutng_assembly::internal::block_node::BlockNode;
    use layoutng_assembly::internal::min_max_sizes::MinMaxSizes;
    use layoutng_geometry::geometry::box_strut::{BoxStrut, PhysicalBoxStrut};
    use layoutng_style::style::computed_style_constants::ItemPosition;

    fn item(index: u32, basis: i32, grow: f32, shrink: f32) -> FlexItem {
        FlexItem::new(
            BlockNode::null(),
            index,
            grow,
            shrink,
            LayoutUnit::from_signed(basis),
            MinMaxSizes {
                min_size: LayoutUnit::default(),
                max_size: LayoutUnit::from_signed(1000),
            },
            LayoutUnit::default(),
            None,
            PhysicalBoxStrut::default(),
            BoxStrut::default(),
            0,
            ItemPosition::kNormal,
            WritingMode::kHorizontalTb,
            BaselineGroup::kMajor,
            false,
            false,
            false,
            true,
        )
    }

    #[test]
    fn distributes_grow_and_shrink_without_losing_free_space() {
        let mut growing = [item(0, 20, 1.0, 1.0), item(1, 20, 1.0, 1.0)];
        LineFlexer::new(
            &mut growing,
            LayoutUnit::from_signed(100),
            LayoutUnit::from_signed(40),
            LayoutUnit::default(),
        )
        .Run();
        assert_eq!(growing[0].flexed_content_size, LayoutUnit::from_signed(50));
        assert_eq!(growing[1].flexed_content_size, LayoutUnit::from_signed(50));

        let mut shrinking = [item(0, 70, 1.0, 1.0), item(1, 70, 1.0, 1.0)];
        LineFlexer::new(
            &mut shrinking,
            LayoutUnit::from_signed(100),
            LayoutUnit::from_signed(140),
            LayoutUnit::default(),
        )
        .Run();
        assert_eq!(
            shrinking[0].flexed_content_size,
            LayoutUnit::from_signed(50)
        );
        assert_eq!(
            shrinking[1].flexed_content_size,
            LayoutUnit::from_signed(50)
        );
    }

    #[test]
    fn fractional_flex_factor_limits_growth() {
        let mut items = [item(0, 0, 0.5, 1.0)];
        LineFlexer::new(
            &mut items,
            LayoutUnit::from_signed(100),
            LayoutUnit::default(),
            LayoutUnit::default(),
        )
        .Run();
        assert_eq!(items[0].flexed_content_size, LayoutUnit::from_signed(50));
    }
}
