// cpp: layoutng_geometry/geometry/layout_unit_diffuser.h:8-9
// Pending connection to //src/foundation:blink_geometry_api.
use foundation::LayoutUnit;

// cpp: layoutng_geometry/geometry/layout_unit_diffuser.h:13-41
/// Distributes the raw-unit remainder across a fixed number of buckets.
#[derive(Clone, Copy, Debug)]
pub struct LayoutUnitDiffuser {
    base_: LayoutUnit,
    dx_: u64,
    dy_: u64,
    x_: u64,
    y_: u64,
    count_: u64,
}

#[allow(non_snake_case)]
impl LayoutUnitDiffuser {
    // cpp: layoutng_geometry/geometry/layout_unit_diffuser.h:43-46
    pub fn new(size: LayoutUnit, buckets: u32) -> Self {
        let result = Self::new_with_remainder(
            size / buckets,
            ((size.RawValue() as u32) % buckets) as u64,
            buckets as u64,
        );
        debug_assert!(size >= LayoutUnit::default());
        result
    }

    // cpp: layoutng_geometry/geometry/layout_unit_diffuser.h:47
    pub fn empty() -> Self {
        Self::new_with_remainder(LayoutUnit::default(), 0, 0)
    }

    // cpp: layoutng_geometry/geometry/layout_unit_diffuser.h:49
    pub fn BaseSize(&self) -> LayoutUnit {
        self.base_
    }

    // cpp: layoutng_geometry/geometry/layout_unit_diffuser.h:51-66
    pub fn Next(&mut self) -> LayoutUnit {
        // Epsilon is exactly one raw LayoutUnit tick (1/64 pixel).
        let epsilon = LayoutUnit::FromRawValue(1);
        if self.count_ == 0 {
            return LayoutUnit::default();
        }
        self.count_ -= 1;
        self.x_ = self.x_.wrapping_add(self.dx_);
        if self.x_ >= self.y_ {
            self.y_ = self.y_.wrapping_add(self.dy_);
            return self.base_ + epsilon;
        }
        self.base_
    }

    // cpp: layoutng_geometry/geometry/layout_unit_diffuser.h:68-84
    fn new_with_remainder(base: LayoutUnit, remainder: u64, buckets: u64) -> Self {
        let result = Self {
            base_: base,
            dx_: remainder.wrapping_mul(2),
            dy_: buckets.wrapping_mul(2),
            x_: 0,
            y_: buckets,
            count_: buckets,
        };
        debug_assert!(remainder <= u64::MAX / 2);
        debug_assert!(remainder <= buckets);
        result
    }
}

// cpp: layoutng_geometry/geometry/layout_unit_diffuser.h:47
impl Default for LayoutUnitDiffuser {
    fn default() -> Self {
        Self::empty()
    }
}
