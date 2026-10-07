#![allow(non_snake_case)]
use layoutng_assembly::internal::layout_input::{ComputedStyle, Overflow};

// cpp: style_resolver/style_resolver.cc:4500-4509
pub(crate) fn NormalizeOverflow(style: &mut ComputedStyle) {
    let Some(extra) = &mut style.extended else {
        return;
    };
    if extra.overflow_x == Overflow::kVisible && extra.overflow_y != Overflow::kVisible {
        extra.overflow_x = Overflow::kAuto;
    }
    if extra.overflow_y == Overflow::kVisible && extra.overflow_x != Overflow::kVisible {
        extra.overflow_y = Overflow::kAuto;
    }
}

// cpp: style_resolver/style_resolver.cc:4511-4516
pub(crate) fn ResolveSVGCurrentColor(style: &mut ComputedStyle) {
    if style.paint.svg_fill_current_color {
        style.paint.svg_fill = Some(style.paint.color);
    }
    if style.paint.svg_stroke_current_color {
        style.paint.svg_stroke = Some(style.paint.color);
    }
}
