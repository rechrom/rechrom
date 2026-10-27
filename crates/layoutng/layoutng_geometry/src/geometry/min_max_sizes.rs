use foundation::LayoutUnit;
use std::ops::{AddAssign, DivAssign, SubAssign};

// cpp: layoutng/internal/min_max_sizes.h:19-22,56-58
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MinMaxSizes {
    pub min_size: LayoutUnit,
    pub max_size: LayoutUnit,
}

#[allow(non_snake_case)]
impl MinMaxSizes {
    // cpp: layoutng/internal/min_max_sizes.h:23-23
    pub fn IsEmpty(&self) -> bool {
        self.min_size == LayoutUnit::default() && self.max_size == LayoutUnit::Max()
    }

    // cpp: layoutng/internal/min_max_sizes.h:26-29
    pub fn Encompass(&mut self, other: &Self) {
        self.min_size = self.min_size.max(other.min_size);
        self.max_size = self.max_size.max(other.max_size);
    }

    // cpp: layoutng/internal/min_max_sizes.h:32-35
    pub fn EncompassValue(&mut self, value: LayoutUnit) {
        self.min_size = self.min_size.max(value);
        self.max_size = self.max_size.max(value);
    }

    // cpp: layoutng/internal/min_max_sizes.h:38-41
    pub fn Constrain(&mut self, value: LayoutUnit) {
        self.min_size = self.min_size.min(value);
        self.max_size = self.max_size.min(value);
    }

    // cpp: layoutng/internal/min_max_sizes.h:45-48
    pub fn ShrinkToFit(&self, available_size: LayoutUnit) -> LayoutUnit {
        debug_assert!(self.max_size >= self.min_size);
        self.max_size.min(self.min_size.max(available_size))
    }

    // cpp: layoutng/internal/min_max_sizes.h:52-54
    pub fn ClampSizeToMinAndMax(&self, size: LayoutUnit) -> LayoutUnit {
        self.min_size.max(size.min(self.max_size))
    }

    // cpp: layoutng/internal/min_max_sizes.h:60-60
    pub fn Assign(&mut self, value: LayoutUnit) {
        self.max_size = value;
        self.min_size = self.max_size;
    }
}

// cpp: layoutng/internal/min_max_sizes.h:61-65
impl AddAssign for MinMaxSizes {
    fn add_assign(&mut self, extra: Self) {
        self.min_size += extra.min_size;
        self.max_size += extra.max_size;
    }
}

// cpp: layoutng/internal/min_max_sizes.h:66-70
impl AddAssign<LayoutUnit> for MinMaxSizes {
    fn add_assign(&mut self, length: LayoutUnit) {
        self.min_size += length;
        self.max_size += length;
    }
}

// cpp: layoutng/internal/min_max_sizes.h:71-75
impl SubAssign<LayoutUnit> for MinMaxSizes {
    fn sub_assign(&mut self, length: LayoutUnit) {
        self.min_size -= length;
        self.max_size -= length;
    }
}

// cpp: layoutng/internal/min_max_sizes.h:76-80
impl DivAssign<f32> for MinMaxSizes {
    fn div_assign(&mut self, length: f32) {
        self.min_size /= length;
        self.max_size /= length;
    }
}

// cpp: layoutng/internal/min_max_sizes.h:88-109
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct MinMaxSizesResult {
    pub sizes: MinMaxSizes,
    pub depends_on_block_constraints: bool,
    pub applied_aspect_ratio: bool,
}

impl MinMaxSizesResult {
    pub fn new(sizes: MinMaxSizes, depends_on_block_constraints: bool) -> Self {
        Self {
            sizes,
            depends_on_block_constraints,
            applied_aspect_ratio: false,
        }
    }

    pub fn with_applied_aspect_ratio(
        sizes: MinMaxSizes,
        depends_on_block_constraints: bool,
        applied_aspect_ratio: bool,
    ) -> Self {
        Self {
            sizes,
            depends_on_block_constraints,
            applied_aspect_ratio,
        }
    }
}
