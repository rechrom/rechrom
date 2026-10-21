pub struct SkScaleToSides;

// Source: src/core/SkScaleToSides.h:23-62
impl SkScaleToSides {
    #[inline]
    pub fn AdjustRadii(limit: f64, scale: f64, a: &mut f32, b: &mut f32) {
        debug_assert!(scale < 1.0 && scale > 0.0, "scale: {scale}");
        *a = (*a as f64 * scale) as f32;
        *b = (*b as f64 * scale) as f32;
        if (*a + *b) as f64 > limit {
            let (min_radius, max_radius) = if *a > *b {
                (&mut *b, &mut *a)
            } else {
                (&mut *a, &mut *b)
            };
            let new_min_radius = *min_radius;
            let mut new_max_radius = (limit - new_min_radius as f64) as f32;
            while (new_max_radius + new_min_radius) as f64 > limit {
                new_max_radius = new_max_radius.next_down();
            }
            *max_radius = new_max_radius;
        }
        debug_assert!(*a >= 0.0 && *b >= 0.0);
        debug_assert!((*a + *b) as f64 <= limit);
    }
}
