#![allow(non_snake_case)]

use foundation::style_values::css::css_anchor_query_enums::{CSSAnchorSizeValue, CSSAnchorValue};
use foundation::{
    IsHorizontalWritingMode, LayoutUnit, PhysicalOffset, PhysicalRect, PhysicalSize,
    WritingDirectionMode, WritingMode,
};
use layoutng_geometry::geometry::logical_size::ToLogicalSize;

// The source types belong to //src/foundation/style_values. Their foundation
// module remains a dependency until that owner exposes them.

// cpp: layoutng_out_of_flow/anchor_query_calculations.cc:12-21
fn PhysicalAnchorValueUsing(
    x: CSSAnchorValue,
    flipped_x: CSSAnchorValue,
    y: CSSAnchorValue,
    flipped_y: CSSAnchorValue,
    writing_direction: WritingDirectionMode,
    is_y_axis: bool,
) -> CSSAnchorValue {
    if is_y_axis {
        if writing_direction.IsFlippedY() {
            flipped_y
        } else {
            y
        }
    } else if writing_direction.IsFlippedX() {
        flipped_x
    } else {
        x
    }
}

// cpp: layoutng_out_of_flow/anchor_query_calculations.cc:26-50
fn PhysicalAnchorValueFromLogicalOrAuto(
    anchor_value: CSSAnchorValue,
    writing_direction: WritingDirectionMode,
    self_writing_direction: WritingDirectionMode,
    is_y_axis: bool,
) -> CSSAnchorValue {
    match anchor_value {
        CSSAnchorValue::kSelfStart => PhysicalAnchorValueUsing(
            CSSAnchorValue::kLeft,
            CSSAnchorValue::kRight,
            CSSAnchorValue::kTop,
            CSSAnchorValue::kBottom,
            self_writing_direction,
            is_y_axis,
        ),
        CSSAnchorValue::kStart => PhysicalAnchorValueUsing(
            CSSAnchorValue::kLeft,
            CSSAnchorValue::kRight,
            CSSAnchorValue::kTop,
            CSSAnchorValue::kBottom,
            writing_direction,
            is_y_axis,
        ),
        CSSAnchorValue::kSelfEnd => PhysicalAnchorValueUsing(
            CSSAnchorValue::kRight,
            CSSAnchorValue::kLeft,
            CSSAnchorValue::kBottom,
            CSSAnchorValue::kTop,
            self_writing_direction,
            is_y_axis,
        ),
        CSSAnchorValue::kEnd => PhysicalAnchorValueUsing(
            CSSAnchorValue::kRight,
            CSSAnchorValue::kLeft,
            CSSAnchorValue::kBottom,
            CSSAnchorValue::kTop,
            writing_direction,
            is_y_axis,
        ),
        _ => anchor_value,
    }
}

// cpp: layoutng_out_of_flow/anchor_query_calculations.cc:54-77
fn PhysicalAnchorValueFromInsideOutside(
    anchor_value: CSSAnchorValue,
    is_y_axis: bool,
    is_right_or_bottom: bool,
) -> CSSAnchorValue {
    match anchor_value {
        CSSAnchorValue::kInside => {
            if is_y_axis {
                if is_right_or_bottom {
                    CSSAnchorValue::kBottom
                } else {
                    CSSAnchorValue::kTop
                }
            } else if is_right_or_bottom {
                CSSAnchorValue::kRight
            } else {
                CSSAnchorValue::kLeft
            }
        }
        CSSAnchorValue::kOutside => {
            if is_y_axis {
                if is_right_or_bottom {
                    CSSAnchorValue::kTop
                } else {
                    CSSAnchorValue::kBottom
                }
            } else if is_right_or_bottom {
                CSSAnchorValue::kLeft
            } else {
                CSSAnchorValue::kRight
            }
        }
        _ => anchor_value,
    }
}

// cpp: layoutng_out_of_flow/anchor_query_calculations.h:17-26
// cpp: layoutng_out_of_flow/anchor_query_calculations.cc:83-177
pub fn ResolveAnchorValue(
    mut anchor_rect: PhysicalRect,
    mut anchor_value: CSSAnchorValue,
    mut percentage: f32,
    available_size: LayoutUnit,
    container_writing_direction: WritingDirectionMode,
    self_writing_direction: WritingDirectionMode,
    offset_to_padding_box: &PhysicalOffset,
    is_y_axis: bool,
    is_right_or_bottom: bool,
) -> Option<LayoutUnit> {
    anchor_rect.offset -= *offset_to_padding_box;
    anchor_value = PhysicalAnchorValueFromLogicalOrAuto(
        anchor_value,
        container_writing_direction,
        self_writing_direction,
        is_y_axis,
    );
    anchor_value =
        PhysicalAnchorValueFromInsideOutside(anchor_value, is_y_axis, is_right_or_bottom);
    let mut value;
    match anchor_value {
        CSSAnchorValue::kCenter => {
            let start = if is_y_axis {
                anchor_rect.Y()
            } else {
                anchor_rect.X()
            };
            let end = if is_y_axis {
                anchor_rect.Bottom()
            } else {
                anchor_rect.Right()
            };
            value = start + LayoutUnit::FromFloatRound((end - start) * 0.5f32);
        }
        CSSAnchorValue::kLeft => {
            if is_y_axis {
                return None;
            }
            value = anchor_rect.X();
        }
        CSSAnchorValue::kRight => {
            if is_y_axis {
                return None;
            }
            value = anchor_rect.Right();
        }
        CSSAnchorValue::kTop => {
            if !is_y_axis {
                return None;
            }
            value = anchor_rect.Y();
        }
        CSSAnchorValue::kBottom => {
            if !is_y_axis {
                return None;
            }
            value = anchor_rect.Bottom();
        }
        CSSAnchorValue::kPercentage => {
            let size;
            if is_y_axis {
                value = anchor_rect.Y();
                size = anchor_rect.Height();
                if container_writing_direction.IsFlippedY() {
                    percentage = 100.0 - percentage;
                }
            } else {
                value = anchor_rect.X();
                size = anchor_rect.Width();
                if container_writing_direction.IsFlippedX() {
                    percentage = 100.0 - percentage;
                }
            }
            value += LayoutUnit::FromFloatRound(size * percentage / 100.0f32);
        }
        CSSAnchorValue::kInside
        | CSSAnchorValue::kOutside
        | CSSAnchorValue::kStart
        | CSSAnchorValue::kEnd
        | CSSAnchorValue::kSelfStart
        | CSSAnchorValue::kSelfEnd => {
            unreachable!("logical anchor side should have been converted")
        }
    }
    if is_right_or_bottom {
        Some(available_size - value)
    } else {
        Some(value)
    }
}

// cpp: layoutng_out_of_flow/anchor_query_calculations.h:28-31
// cpp: layoutng_out_of_flow/anchor_query_calculations.cc:181-210
pub fn ResolveAnchorSizeValue(
    anchor_size: &PhysicalSize,
    anchor_size_value: CSSAnchorSizeValue,
    container_writing_mode: WritingMode,
    self_writing_mode: WritingMode,
) -> LayoutUnit {
    let logical_size = ToLogicalSize(*anchor_size, container_writing_mode);
    match anchor_size_value {
        CSSAnchorSizeValue::kInline => logical_size.inline_size,
        CSSAnchorSizeValue::kBlock => logical_size.block_size,
        CSSAnchorSizeValue::kWidth => anchor_size.width,
        CSSAnchorSizeValue::kHeight => anchor_size.height,
        CSSAnchorSizeValue::kSelfInline => {
            if IsHorizontalWritingMode(container_writing_mode)
                == IsHorizontalWritingMode(self_writing_mode)
            {
                logical_size.inline_size
            } else {
                logical_size.block_size
            }
        }
        CSSAnchorSizeValue::kSelfBlock => {
            if IsHorizontalWritingMode(container_writing_mode)
                == IsHorizontalWritingMode(self_writing_mode)
            {
                logical_size.block_size
            } else {
                logical_size.inline_size
            }
        }
        CSSAnchorSizeValue::kImplicit => unreachable!("implicit anchor size is NOTREACHED"),
    }
}
