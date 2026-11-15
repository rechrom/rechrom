// C++: src/foundation/blink_geometry/geometry/physical_size.cc

use crate::gfx_geometry::SizeF;

use super::layout_unit::LayoutUnit;
use super::physical_size::{AspectRatioFit, PhysicalSize};

impl PhysicalSize {
    pub fn FitToAspectRatio(&self, aspect_ratio: &PhysicalSize, fit: AspectRatioFit) -> Self {
        debug_assert!(aspect_ratio.width > 0);
        debug_assert!(aspect_ratio.height > 0);
        let constrained_height = self.width.MulDiv(aspect_ratio.height, aspect_ratio.width);
        let grow = fit == AspectRatioFit::kAspectRatioFitGrow;
        if (grow && constrained_height < self.height) || (!grow && constrained_height > self.height)
        {
            let constrained_width = self.height.MulDiv(aspect_ratio.width, aspect_ratio.height);
            return Self::new(constrained_width, self.height);
        }
        Self::new(self.width, constrained_height)
    }
}

pub fn LayoutRatioFromSizeF(ratio: SizeF) -> PhysicalSize {
    let width = LayoutUnit::from_f32(ratio.width());
    let height = LayoutUnit::from_f32(ratio.height());
    if (width.ToFloat() == ratio.width() && height.ToFloat() == ratio.height()) || ratio.IsEmpty() {
        return PhysicalSize::new(width, height);
    }
    if ratio.width() == ratio.height() {
        return PhysicalSize::new(LayoutUnit::from_signed(1), LayoutUnit::from_signed(1));
    }

    // Continued fractions, preserving the original 16-iteration bound.
    let initial = ratio.AspectRatio();
    let mut x = initial;
    let (mut h0, mut h1, mut k0, mut k1) = (0_i32, 1_i32, 1_i32, 0_i32);
    for _ in 0..16 {
        if !x.is_finite() {
            break;
        }
        let estimate = h1 as f32 / k1 as f32;
        if (initial - estimate).abs() < 0.000001_f32 {
            break;
        }
        let a = ClampFloorInt(x);
        let h2 = h1.saturating_mul(a).saturating_add(h0);
        let k2 = k1.saturating_mul(a).saturating_add(k0);
        if h2 == i32::MAX || k2 == i32::MAX {
            break;
        }
        h0 = h1;
        k0 = k1;
        h1 = h2;
        k1 = k2;
        x = 1.0 / (x - a as f32);
    }
    if h1 == 0 || k1 == 0 {
        return PhysicalSize::new(width, height);
    }
    PhysicalSize::new(LayoutUnit::FromRawValue(h1), LayoutUnit::FromRawValue(k1))
}

fn ClampFloorInt(value: f32) -> i32 {
    let floor = value.floor();
    if floor.is_nan() {
        0
    } else if floor >= i32::MAX as f32 {
        i32::MAX
    } else if floor <= i32::MIN as f32 {
        i32::MIN
    } else {
        floor as i32
    }
}
